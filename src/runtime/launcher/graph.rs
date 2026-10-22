//! M6：W09 捕获隔离、W10 数值槽重绑、W11 预分配与 W21 捕获作用域入队。
//! 捕获以上下文为互斥单位；捕获期间只记录节点，不返回完成凭证。
//! 重绑只接受数值槽；整批校验通过才提交，旧资源进入待回收列表。

use super::{
    AbiSignature, ElemType, LauncherError, RetainedResources, SlotBinding, SlotRequirement,
    SlotRole, validate_bindings,
};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::sync::Mutex;

/// 图节点编号。
pub type NodeId = usize;

/// 图实例状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphState {
    Ready,
    Invalid,
}

/// 捕获阶段的预分配记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allocation {
    bytes: u64,
}

impl Allocation {
    pub fn bytes(self) -> u64 {
        self.bytes
    }
}

struct ContextState {
    capturing: bool,
    event_tracking: bool,
    allocated_bytes: u64,
}

/// 上下文级捕获互斥与预分配记账。
pub struct CaptureHost {
    state: Mutex<ContextState>,
}

impl Default for CaptureHost {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureHost {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ContextState {
                capturing: false,
                event_tracking: true,
                allocated_bytes: 0,
            }),
        }
    }

    pub fn is_capturing(&self) -> bool {
        self.lock().capturing
    }

    /// 捕获前预分配；捕获期间一律拒绝。
    pub fn allocate(&self, bytes: u64) -> Result<Allocation, LauncherError> {
        if bytes == 0 {
            return Err(LauncherError::InvalidSpan("预分配字节数为零"));
        }
        let mut state = self.lock();
        if state.capturing {
            return Err(LauncherError::Unsupported("捕获期间禁止分配"));
        }
        state.allocated_bytes = state
            .allocated_bytes
            .checked_add(bytes)
            .ok_or(LauncherError::Overflow("capture_allocation"))?;
        Ok(Allocation { bytes })
    }

    pub fn allocated_bytes(&self) -> u64 {
        self.lock().allocated_bytes
    }

    pub fn event_tracking(&self) -> bool {
        self.lock().event_tracking
    }

    /// 进入捕获；同一上下文同时只允许一个捕获。
    pub fn begin(&self) -> Result<CaptureGuard<'_>, LauncherError> {
        let mut state = self.lock();
        if state.capturing {
            return Err(LauncherError::CaptureBusy);
        }
        state.capturing = true;
        let previous_event_tracking = state.event_tracking;
        state.event_tracking = false;
        Ok(CaptureGuard {
            host: self,
            previous_event_tracking,
            nodes: Vec::new(),
            finished: false,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ContextState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn end(&self) {
        let mut state = self.lock();
        state.capturing = false;
    }

    fn restore_event_tracking(&self, enabled: bool) {
        self.lock().event_tracking = enabled;
    }
}

/// 捕获作用域内的单节点记录。
#[derive(Clone, Debug)]
pub struct CapturedNode {
    signature: AbiSignature,
    bindings: Vec<SlotBinding>,
    requirements: Vec<SlotRequirement>,
}

impl CapturedNode {
    pub fn name(&self) -> &'static str {
        self.signature.name()
    }

    pub fn signature(&self) -> &AbiSignature {
        &self.signature
    }

    pub fn bindings(&self) -> &[SlotBinding] {
        &self.bindings
    }
}

/// 捕获完成的静态图。
#[derive(Clone, Debug)]
pub struct CapturedGraph {
    nodes: Vec<CapturedNode>,
}

impl CapturedGraph {
    pub fn nodes(&self) -> &[CapturedNode] {
        &self.nodes
    }

    pub fn node(&self, node: NodeId) -> Result<&CapturedNode, LauncherError> {
        self.nodes
            .get(node)
            .ok_or(LauncherError::UnknownSubmission(node as u64))
    }

    /// 按当前绑定建立图实例。
    pub fn into_instance<R: RetainedResources>(self, resources: R) -> GraphInstance<R> {
        GraphInstance {
            graph: self,
            resources,
            pending: BTreeSet::new(),
            recycled: Vec::new(),
            started_replays: 0,
            next_recycle: 0,
            state: Cell::new(GraphState::Ready),
        }
    }
}

/// 捕获守卫；析构时恢复状态，恐慌路径同样结束捕获。
pub struct CaptureGuard<'a> {
    host: &'a CaptureHost,
    previous_event_tracking: bool,
    nodes: Vec<CapturedNode>,
    finished: bool,
}

impl CaptureGuard<'_> {
    /// 只记录节点；不执行内核，不返回完成凭证。
    pub fn enqueue(
        &mut self,
        signature: &AbiSignature,
        bindings: &[SlotBinding],
        requirements: &[SlotRequirement],
    ) -> Result<NodeId, LauncherError> {
        if self.finished {
            return Err(LauncherError::Unsupported("捕获作用域已经结束"));
        }
        validate_bindings(signature, bindings, requirements)?;
        self.nodes.push(CapturedNode {
            signature: signature.clone(),
            bindings: bindings.to_vec(),
            requirements: requirements.to_vec(),
        });
        Ok(self.nodes.len() - 1)
    }

    /// 捕获期间拒绝同步发射。
    pub fn reject_launch_sync(&self) -> Result<(), LauncherError> {
        Err(LauncherError::Unsupported("捕获期间禁止同步发射"))
    }

    /// 捕获期间拒绝异步提交。
    pub fn reject_submit(&self) -> Result<(), LauncherError> {
        Err(LauncherError::Unsupported("捕获期间禁止异步提交"))
    }

    /// 捕获期间拒绝无关流入队。
    pub fn reject_unrelated_enqueue(&self) -> Result<(), LauncherError> {
        Err(LauncherError::Unsupported("捕获期间拒绝无关流入队"))
    }

    /// 捕获期间拒绝流同步与事件查询。
    pub fn reject_synchronize(&self) -> Result<(), LauncherError> {
        Err(LauncherError::Unsupported("捕获期间禁止流同步"))
    }

    /// 捕获期间拒绝编译。
    pub fn reject_compile(&self) -> Result<(), LauncherError> {
        Err(LauncherError::Unsupported("捕获期间禁止编译"))
    }

    /// 结束捕获并返回静态图；空图直接销毁。
    pub fn finish(mut self) -> Result<CapturedGraph, LauncherError> {
        self.finished = true;
        if self.nodes.is_empty() {
            return Err(LauncherError::Unsupported("捕获图为空"));
        }
        Ok(CapturedGraph {
            nodes: std::mem::take(&mut self.nodes),
        })
    }
}

impl Drop for CaptureGuard<'_> {
    fn drop(&mut self) {
        self.host
            .restore_event_tracking(self.previous_event_tracking);
        self.host.end();
    }
}

/// 一次数值槽更新：节点加参数槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotUpdate {
    pub node: NodeId,
    pub binding: SlotBinding,
}

/// 图重放编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplayId(u64);

impl ReplayId {
    pub fn value(self) -> u64 {
        self.0
    }
}

/// 待回收批次编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecycleId(u64);

impl RecycleId {
    pub fn value(self) -> u64 {
        self.0
    }
}

struct RecycleRecord<R> {
    freed_after: u64,
    resources: R,
}

/// 图实例：持有当前绑定、资源与待回收列表。
pub struct GraphInstance<R: RetainedResources> {
    graph: CapturedGraph,
    resources: R,
    pending: BTreeSet<u64>,
    recycled: Vec<RecycleRecord<R>>,
    started_replays: u64,
    next_recycle: u64,
    state: Cell<GraphState>,
}

impl<R: RetainedResources> GraphInstance<R> {
    pub fn state(&self) -> GraphState {
        self.state.get()
    }

    pub fn nodes(&self) -> &[CapturedNode] {
        self.graph.nodes()
    }

    pub fn node(&self, node: NodeId) -> Result<&CapturedNode, LauncherError> {
        self.graph.node(node)
    }

    pub fn resources(&self) -> &R {
        &self.resources
    }

    pub fn pending_replays(&self) -> usize {
        self.pending.len()
    }

    pub fn recycled_len(&self) -> usize {
        self.recycled.len()
    }

    /// 作废图实例；作废后拒绝重放与重绑。
    pub fn invalidate(&self) {
        self.state.set(GraphState::Invalid);
    }

    fn require_ready(&self) -> Result<(), LauncherError> {
        if self.state.get() == GraphState::Ready {
            Ok(())
        } else {
            Err(LauncherError::GraphInvalid)
        }
    }

    /// 开始一次图重放，返回重放编号；完成凭证由重放路径持有。
    pub fn begin_replay(&mut self) -> Result<ReplayId, LauncherError> {
        self.require_ready()?;
        let id = self.started_replays;
        self.started_replays = id
            .checked_add(1)
            .ok_or(LauncherError::Overflow("replay_id"))?;
        self.pending.insert(id);
        Ok(ReplayId(id))
    }

    /// 标记一次重放完成并释放满足条件的旧资源。
    pub fn complete_replay(&mut self, id: ReplayId) -> Result<(), LauncherError> {
        self.require_ready()?;
        if !self.pending.remove(&id.0) {
            return Err(LauncherError::UnknownReplay(id.0));
        }
        self.release_recycled();
        Ok(())
    }

    /// 批量重绑数值槽。
    ///
    /// 先构造提案并逐节点联合校验，通过后才替换绑定与资源。
    /// 失败返回原资源并保持原绑定（事务回滚）。
    /// 旧资源待所有在途重放完成后释放。
    pub fn rebind_values(
        &mut self,
        updates: &[SlotUpdate],
        new_resources: R,
    ) -> Result<RecycleId, (LauncherError, R)> {
        if let Err(error) = self.require_ready() {
            return Err((error, new_resources));
        }
        if updates.is_empty() {
            return Err((LauncherError::Unsupported("空重绑批次"), new_resources));
        }
        let recycle_id = self.next_recycle;
        let next_recycle = match recycle_id.checked_add(1) {
            Some(next) => next,
            None => return Err((LauncherError::Overflow("recycle_id"), new_resources)),
        };
        let mut proposed: Vec<Vec<SlotBinding>> = self
            .graph
            .nodes()
            .iter()
            .map(|node| node.bindings.clone())
            .collect();
        let mut seen = BTreeSet::new();
        for update in updates {
            let Some(node) = self.graph.nodes().get(update.node) else {
                return Err((
                    LauncherError::UnknownNode(update.node as u64),
                    new_resources,
                ));
            };
            let slot = update.binding.slot;
            let Some(desc) = node.signature().slots().get(slot) else {
                return Err((LauncherError::UnexpectedBinding { slot }, new_resources));
            };
            if !desc.kind.is_buffer() {
                return Err((LauncherError::UnexpectedBinding { slot }, new_resources));
            }
            if matches!(desc.role, SlotRole::AddressBearing { .. }) {
                return Err((LauncherError::AddressSlotRebind { slot }, new_resources));
            }
            if !seen.insert((update.node, slot)) {
                return Err((LauncherError::DuplicateBinding { slot }, new_resources));
            }
            let Some(old) = proposed[update.node]
                .iter_mut()
                .find(|binding| binding.slot == slot)
            else {
                return Err((LauncherError::MissingBinding { slot }, new_resources));
            };
            if let Err(error) = check_rebind(desc.kind.elem(), old, &update.binding) {
                return Err((error, new_resources));
            }
            *old = update.binding;
        }
        for (node, bindings) in self.graph.nodes().iter().zip(proposed.iter()) {
            if let Err(error) = validate_bindings(node.signature(), bindings, &node.requirements) {
                return Err((error, new_resources));
            }
        }
        for (node, bindings) in self.graph.nodes.iter_mut().zip(proposed) {
            node.bindings = bindings;
        }
        let resources = std::mem::replace(&mut self.resources, new_resources);
        self.next_recycle = next_recycle;
        self.recycled.push(RecycleRecord {
            freed_after: self.started_replays,
            resources,
        });
        self.release_recycled();
        Ok(RecycleId(recycle_id))
    }

    fn release_recycled(&mut self) {
        let pending = &self.pending;
        let mut keep = Vec::new();
        for record in self.recycled.drain(..) {
            if pending.iter().any(|replay| *replay <= record.freed_after) {
                keep.push(record);
            } else {
                // 在途重放已全部完成，真正释放旧资源。
                let RecycleRecord {
                    freed_after,
                    resources,
                } = record;
                let _ = freed_after;
                drop(resources);
            }
        }
        self.recycled = keep;
    }
}

impl<R: RetainedResources> RetainedResources for GraphInstance<R> {
    fn quarantine(&self) {
        self.invalidate();
    }
}

fn check_rebind(
    expected: Option<ElemType>,
    old: &SlotBinding,
    new: &SlotBinding,
) -> Result<(), LauncherError> {
    let Some(elem) = expected else {
        return Err(LauncherError::UnexpectedBinding { slot: old.slot });
    };
    if new.elem != elem || new.elem != old.elem {
        return Err(LauncherError::ElementMismatch { slot: old.slot });
    }
    if new.span.context() != old.span.context() {
        return Err(LauncherError::ContextMismatch { slot: old.slot });
    }
    if new.span.bytes() < old.span.bytes() {
        return Err(LauncherError::RebindShrink { slot: old.slot });
    }
    Ok(())
}
