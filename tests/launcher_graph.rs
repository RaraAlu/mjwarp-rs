//! M6：捕获隔离、数值槽重绑与待回收列表的宿主验证。
//! 不启动真实图捕获；真实探针图仍属既有`cuda/graph.rs`路径。

use mjwarp_rs::runtime::launcher::{
    AbiSignature, Access, BlockRule, CaptureHost, DeviceSpan, ElemType, Fence, GraphInstance,
    GraphState, LauncherError, RetainedResources, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole, SlotUpdate, SubmitQueue,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

const VALUE_SLOTS: &[SlotDesc] = &[
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
];

const MIXED_SLOTS: &[SlotDesc] = &[
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::ReadWrite,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: mjwarp_rs::runtime::launcher::ProofKind("test.graph.metadata.v1"),
        },
    },
];

fn signature(slots: &'static [SlotDesc]) -> AbiSignature {
    AbiSignature::check("test_graph", BlockRule::Fixed(256), slots, &[]).unwrap()
}

fn binding(slot: usize, addr: u64, bytes: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: DeviceSpan::new(1, addr, bytes).unwrap(),
        elem,
    }
}

fn value_binding(slot: usize, addr: u64, bytes: u64) -> SlotBinding {
    binding(slot, addr, bytes, ElemType::F32)
}

fn requirements() -> Vec<SlotRequirement> {
    vec![SlotRequirement {
        slot: 0,
        min_elements: 32,
    }]
}

#[derive(Debug)]
struct Tracked {
    alive: Arc<AtomicBool>,
    quarantined: Arc<AtomicBool>,
}

impl Tracked {
    fn new() -> (Self, Arc<AtomicBool>, Arc<AtomicBool>) {
        let alive = Arc::new(AtomicBool::new(true));
        let quarantined = Arc::new(AtomicBool::new(false));
        (
            Self {
                alive: Arc::clone(&alive),
                quarantined: Arc::clone(&quarantined),
            },
            alive,
            quarantined,
        )
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}

impl RetainedResources for Tracked {
    fn quarantine(&self) {
        self.quarantined.store(true, Ordering::SeqCst);
    }
}

fn captured(graph_signature: &AbiSignature, bindings: &[SlotBinding]) -> GraphInstance<Tracked> {
    let host = CaptureHost::new();
    let mut guard = host.begin().unwrap();
    guard
        .enqueue(graph_signature, bindings, &requirements())
        .unwrap();
    let graph = guard.finish().unwrap();
    let (tracked, _, _) = Tracked::new();
    graph.into_instance(tracked)
}

#[test]
fn capture_is_context_exclusive_and_nested_is_rejected() {
    let host = CaptureHost::new();
    assert!(!host.is_capturing());
    let guard = host.begin().unwrap();
    assert!(host.is_capturing());
    assert!(matches!(host.begin(), Err(LauncherError::CaptureBusy)));
    drop(guard);
    assert!(!host.is_capturing());
    // 异常路径同样恢复互斥。
    assert!(
        std::panic::catch_unwind(|| {
            let _guard = host.begin().unwrap();
            panic!("test capture unwind");
        })
        .is_err()
    );
    assert!(!host.is_capturing());
    let guard = host.begin().unwrap();
    drop(guard);
    assert!(!host.is_capturing());
}

#[test]
fn capture_scope_records_nodes_and_rejects_unrelated_work() {
    let host = CaptureHost::new();
    let sig = signature(VALUE_SLOTS);
    let bindings = [value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)];
    let mut guard = host.begin().unwrap();
    let node = guard.enqueue(&sig, &bindings, &requirements()).unwrap();
    assert_eq!(node, 0);
    assert!(guard.reject_launch_sync().is_err());
    assert!(guard.reject_submit().is_err());
    assert!(guard.reject_unrelated_enqueue().is_err());
    assert!(guard.reject_synchronize().is_err());
    assert!(guard.reject_compile().is_err());
    // 捕获期间禁止分配。
    assert_eq!(
        host.allocate(64).unwrap_err(),
        LauncherError::Unsupported("捕获期间禁止分配")
    );
    let graph = guard.finish().unwrap();
    assert_eq!(graph.nodes().len(), 1);
    assert_eq!(graph.node(0).unwrap().name(), "test_graph");
    // 捕获结束后可继续预分配。
    host.allocate(64).unwrap();
    assert_eq!(host.allocated_bytes(), 64);
}

#[test]
fn capture_restores_event_tracking_and_rejects_empty_graph() {
    let host = CaptureHost::new();
    assert!(host.event_tracking());
    let guard = host.begin().unwrap();
    assert!(!host.event_tracking());
    drop(guard);
    assert!(host.event_tracking());
    let guard = host.begin().unwrap();
    assert_eq!(
        guard.finish().unwrap_err(),
        LauncherError::Unsupported("捕获图为空")
    );
    assert!(host.event_tracking());
    assert!(!host.is_capturing());
}

#[test]
fn bindings_are_validated_inside_the_capture_scope() {
    let host = CaptureHost::new();
    let sig = signature(VALUE_SLOTS);
    let mut guard = host.begin().unwrap();
    assert!(guard.enqueue(&sig, &[], &requirements()).is_err());
    let wrong_elem = [
        binding(0, 0x1000, 256, ElemType::I32),
        value_binding(1, 0x2000, 256),
    ];
    assert!(guard.enqueue(&sig, &wrong_elem, &requirements()).is_err());
    let overlapping = [value_binding(0, 0x1000, 256), value_binding(1, 0x1080, 256)];
    assert!(guard.enqueue(&sig, &overlapping, &requirements()).is_err());
    assert!(guard.finish().is_err());
}

#[test]
fn rebind_rejects_address_slots_and_mismatched_buffers() {
    let sig = signature(MIXED_SLOTS);
    let mut graph = captured(
        &sig,
        &[
            value_binding(0, 0x1000, 256),
            binding(1, 0x2000, 256, ElemType::I32),
        ],
    );
    let (tracked, alive, _) = Tracked::new();
    let address_slot = SlotUpdate {
        node: 0,
        binding: binding(1, 0x3000, 256, ElemType::I32),
    };
    let (error, returned) = graph.rebind_values(&[address_slot], tracked).unwrap_err();
    assert_eq!(error, LauncherError::AddressSlotRebind { slot: 1 });
    assert!(alive.load(Ordering::SeqCst));
    drop(returned);
    let (tracked, _, _) = Tracked::new();
    let (error, _) = graph
        .rebind_values(
            &[SlotUpdate {
                node: 0,
                binding: binding(0, 0x3000, 256, ElemType::I32),
            }],
            tracked,
        )
        .unwrap_err();
    assert_eq!(error, LauncherError::ElementMismatch { slot: 0 });
    let (tracked, _, _) = Tracked::new();
    let (error, _) = graph
        .rebind_values(
            &[SlotUpdate {
                node: 5,
                binding: value_binding(0, 0x3000, 256),
            }],
            tracked,
        )
        .unwrap_err();
    assert_eq!(error, LauncherError::UnknownNode(5));
    let (tracked, _, _) = Tracked::new();
    let (error, _) = graph
        .rebind_values(
            &[SlotUpdate {
                node: 0,
                binding: value_binding(9, 0x3000, 256),
            }],
            tracked,
        )
        .unwrap_err();
    assert_eq!(error, LauncherError::UnexpectedBinding { slot: 9 });
}

#[test]
fn rebind_checks_context_capacity_and_duplicates() {
    let sig = signature(VALUE_SLOTS);
    let mut graph = captured(
        &sig,
        &[value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)],
    );
    let other_context = SlotBinding {
        slot: 0,
        span: DeviceSpan::new(2, 0x3000, 256).unwrap(),
        elem: ElemType::F32,
    };
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        graph
            .rebind_values(
                &[SlotUpdate {
                    node: 0,
                    binding: other_context
                }],
                tracked
            )
            .unwrap_err()
            .0,
        LauncherError::ContextMismatch { slot: 0 }
    );
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        graph
            .rebind_values(
                &[SlotUpdate {
                    node: 0,
                    binding: value_binding(0, 0x3000, 128),
                }],
                tracked
            )
            .unwrap_err()
            .0,
        LauncherError::RebindShrink { slot: 0 }
    );
    let (tracked, _, _) = Tracked::new();
    let duplicate = SlotUpdate {
        node: 0,
        binding: value_binding(0, 0x3000, 256),
    };
    assert_eq!(
        graph
            .rebind_values(&[duplicate, duplicate], tracked)
            .unwrap_err()
            .0,
        LauncherError::DuplicateBinding { slot: 0 }
    );
    let (tracked, _, _) = Tracked::new();
    assert_eq!(
        graph.rebind_values(&[], tracked).unwrap_err().0,
        LauncherError::Unsupported("空重绑批次")
    );
}

#[test]
fn rebind_rolls_back_alias_conflicts_and_keeps_old_bindings() {
    let sig = signature(VALUE_SLOTS);
    let mut graph = captured(
        &sig,
        &[value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)],
    );
    let before = graph.node(0).unwrap().bindings().to_vec();
    let (tracked, alive, _) = Tracked::new();
    // 把可读写槽改绑到写槽区间内：整批联合校验失败。
    let (error, returned) = graph
        .rebind_values(
            &[SlotUpdate {
                node: 0,
                binding: value_binding(0, 0x2080, 256),
            }],
            tracked,
        )
        .unwrap_err();
    assert_eq!(
        error,
        LauncherError::Alias {
            first: 0,
            second: 1
        }
    );
    assert!(alive.load(Ordering::SeqCst));
    drop(returned);
    assert_eq!(graph.node(0).unwrap().bindings(), before.as_slice());
    assert_eq!(graph.recycled_len(), 0);
    assert_eq!(graph.state(), GraphState::Ready);
}

#[test]
fn rebind_holds_old_resources_until_pending_replays_complete() {
    let sig = signature(VALUE_SLOTS);
    let (old, old_alive, _) = Tracked::new();
    let graph = {
        let host = CaptureHost::new();
        let mut guard = host.begin().unwrap();
        guard
            .enqueue(
                &sig,
                &[value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)],
                &requirements(),
            )
            .unwrap();
        guard.finish().unwrap().into_instance(old)
    };
    let mut graph = graph;
    let replay = graph.begin_replay().unwrap();
    let (new, new_alive, _) = Tracked::new();
    let recycle = graph
        .rebind_values(
            &[SlotUpdate {
                node: 0,
                binding: value_binding(0, 0x3000, 256),
            }],
            new,
        )
        .unwrap();
    assert_eq!(graph.recycled_len(), 1);
    assert!(old_alive.load(Ordering::SeqCst));
    assert!(new_alive.load(Ordering::SeqCst));
    assert_eq!(graph.node(0).unwrap().bindings()[0].span.addr(), 0x3000);
    // 在途重放未完成前旧资源不得释放。
    assert_eq!(graph.pending_replays(), 1);
    graph.complete_replay(replay).unwrap();
    assert!(!old_alive.load(Ordering::SeqCst));
    assert!(new_alive.load(Ordering::SeqCst));
    assert_eq!(graph.recycled_len(), 0);
    assert_eq!(recycle.value(), 0);
    assert_eq!(
        graph.complete_replay(replay).unwrap_err(),
        LauncherError::UnknownReplay(0)
    );
}

#[test]
fn invalidated_graph_rejects_replay_and_rebind() {
    let sig = signature(VALUE_SLOTS);
    let mut graph = captured(
        &sig,
        &[value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)],
    );
    graph.invalidate();
    assert_eq!(graph.state(), GraphState::Invalid);
    assert_eq!(
        graph.begin_replay().unwrap_err(),
        LauncherError::GraphInvalid
    );
    let (tracked, alive, _) = Tracked::new();
    let (error, returned) = graph
        .rebind_values(
            &[SlotUpdate {
                node: 0,
                binding: value_binding(0, 0x3000, 256),
            }],
            tracked,
        )
        .unwrap_err();
    assert_eq!(error, LauncherError::GraphInvalid);
    assert!(alive.load(Ordering::SeqCst));
    drop(returned);
}

#[derive(Default)]
struct FenceState {
    ready: bool,
    error: Option<LauncherError>,
}

struct MockFence {
    state: Arc<Mutex<FenceState>>,
}

impl Fence for MockFence {
    fn query(&self) -> Result<bool, LauncherError> {
        let state = self.state.lock().unwrap();
        match &state.error {
            Some(error) => Err(error.clone()),
            None => Ok(state.ready),
        }
    }

    fn wait(&self) -> Result<(), LauncherError> {
        let mut state = self.state.lock().unwrap();
        match &state.error {
            Some(error) => Err(error.clone()),
            None => {
                state.ready = true;
                Ok(())
            }
        }
    }
}

#[test]
fn graph_instance_plugs_into_m5_retained_resource_slot() {
    let sig = signature(VALUE_SLOTS);
    let bindings = [value_binding(0, 0x1000, 256), value_binding(1, 0x2000, 256)];
    // 图实例实现`RetainedResources`：隔离即作废图。
    let graph = captured(&sig, &bindings);
    graph.quarantine();
    assert_eq!(graph.state(), GraphState::Invalid);

    let graph = captured(&sig, &bindings);
    let mut queue: SubmitQueue<MockFence, GraphInstance<Tracked>> = SubmitQueue::new();
    let fence_state = Arc::new(Mutex::new(FenceState::default()));
    let fence = MockFence {
        state: Arc::clone(&fence_state),
    };
    let stream = mjwarp_rs::runtime::launcher::StreamKey::new(0, 0, 1);
    let id = queue.submit(stream, &[], &[], graph, fence).unwrap();
    fence_state.lock().unwrap().ready = true;
    assert!(queue.query(id).unwrap());
    let graph = queue.wait(id).unwrap();
    assert_eq!(graph.state(), GraphState::Ready);
    assert_eq!(graph.nodes().len(), 1);
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires an NVIDIA GPU"]
fn gpu_preallocates_before_capture_and_rejects_during_capture() {
    use mjwarp_rs::runtime::TransferSession;
    let session = TransferSession::new(0).unwrap();
    let host = CaptureHost::new();
    host.allocate(64 * 4).unwrap();
    let buffer = session.preallocate::<f32>(64).unwrap();
    assert_eq!(buffer.len(), 64);
    assert_eq!(buffer.byte_len(), 256);
    assert_eq!(session.device(), 0);
    let guard = host.begin().unwrap();
    assert!(host.allocate(16).is_err());
    drop(guard);
    host.allocate(16).unwrap();
    assert_eq!(host.allocated_bytes(), 64 * 4 + 16);
    drop(buffer);
}
