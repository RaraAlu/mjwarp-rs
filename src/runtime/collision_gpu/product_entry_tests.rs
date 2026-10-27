//! S3：产品碰撞入口的三态回退单测（纯 CPU：不创建 CUDA context、不编译 NVRTC）。
//!
//! 覆盖：①未编译 `cuda`（`FeatureDisabled`）②设备/程序不可用（各 `BackendUnavailable`
//! 原因，以及探测通过但程序缺失）③运行期失败（注入执行器返回 `Launch`）；成功路径把请求
//! 原样交给执行器（内核种类透传）；`nworld>1` 与非法快照在任何设备工作之前被拒绝；Host
//! 选择从不触碰设备。真实设备路径由 GPU 窗口 target 覆盖，本模块不依赖硬件。

use super::{
    CollisionDeviceRunner, CollisionGpuOutput, CollisionInputsSnapshot, ProductCollisionOutcome,
    ProductCollisionRequest, ProductHostReason, prepare_inputs, run_device_or_fallback,
    run_product_collision,
};
use crate::runtime::backend::{BackendChoice, BackendUnavailable};
use crate::runtime::collision_gpu::{
    CollisionGpuError, CollisionGpuInputs, CollisionGpuMeta, CollisionKernel,
};
use std::cell::Cell;

/// 单 geom、无 mesh/hfield/oct 数据的最小合法快照（9 项 mesh_poly 全空）。
struct MinimalInputs {
    geom_xpos: [f32; 3],
    geom_xmat: [f32; 9],
    geom_size: [f32; 3],
    geom_margin: [f32; 1],
    geom_gap: [f32; 1],
    geom_rbound: [f32; 1],
    geom_aabb: [f32; 6],
    geom_type: [i32; 1],
    geom_dataid: [i32; 1],
}

impl MinimalInputs {
    fn new() -> Self {
        Self {
            geom_xpos: [0.0; 3],
            geom_xmat: [0.0; 9],
            geom_size: [0.0; 3],
            geom_margin: [0.0; 1],
            geom_gap: [0.0; 1],
            geom_rbound: [0.0; 1],
            geom_aabb: [0.0; 6],
            geom_type: [6; 1],
            geom_dataid: [0; 1],
        }
    }

    fn snapshot(&self) -> CollisionInputsSnapshot<'_> {
        CollisionInputsSnapshot {
            geom_xpos: &self.geom_xpos,
            geom_xmat: &self.geom_xmat,
            geom_size: &self.geom_size,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_rbound: &self.geom_rbound,
            geom_aabb: &self.geom_aabb,
            geom_type: &self.geom_type,
            geom_dataid: &self.geom_dataid,
            mesh_vertadr: &[],
            mesh_vertnum: &[],
            mesh_graphadr: &[],
            mesh_octadr: &[],
            mesh_vert: &[],
            mesh_graph: &[],
            mesh_polynum: &[],
            mesh_polyadr: &[],
            mesh_polynormal: &[],
            mesh_polyvertadr: &[],
            mesh_polyvertnum: &[],
            mesh_polyvert: &[],
            mesh_polymapadr: &[],
            mesh_polymapnum: &[],
            mesh_polymap: &[],
            hfield_size: &[],
            hfield_nrow: &[],
            hfield_ncol: &[],
            hfield_adr: &[],
            hfield_data: &[],
            oct_aabb: &[],
            oct_child: &[],
            oct_coeff: &[],
        }
    }

    /// 与 M0b/M3 case `meta.txt` 同口径的配对参数（本片只接受 `nworld=1`）。
    fn meta(nworld: i32) -> CollisionGpuMeta {
        CollisionGpuMeta {
            nworld,
            ngeom: 1,
            g1: 0,
            g2: 0,
            type1: 6,
            type2: 7,
            geomgeomid: 0,
            naccdmax: 4,
            hfield_id: -1,
            oct_root: -1,
            noct: 0,
            tolerance: 1e-6,
            margin: 0.0,
            gap: 0.0,
            gjk_iterations: 35,
            epa_iterations: 35,
            condim: 3,
            pairid0: -1,
            pairid1: -1,
            friction: [1.0, 1.0, 0.005, 1e-4, 1e-4],
            solref: [0.02, 1.0],
            solreffriction: [0.0, 0.0],
            solimp: [0.9, 0.95, 0.001, 0.5, 2.0],
            adhesion: 0.0,
            sdf_initpoints: 40,
            sdf_iterations: 10,
            naconmax: 4,
            efc_width: 4,
        }
    }

    fn request(&self, nworld: i32) -> ProductCollisionRequest<'_> {
        ProductCollisionRequest {
            snapshot: self.snapshot(),
            meta: Self::meta(nworld),
            kernel: CollisionKernel::ContactMulti,
        }
    }
}

/// 准备合法输入（经真实 S2 adapter 校验后的 `CollisionGpuInputs`）。
fn prepared<'a>(inputs: &'a MinimalInputs, nworld: i32) -> CollisionGpuInputs<'a> {
    let request = inputs.request(nworld);
    prepare_inputs(&request).expect("最小合法快照应通过 S2 adapter 校验")
}

/// 模拟设备执行器：记录调用次数与内核种类，并按注入结果返回。
struct MockDevice {
    calls: Cell<usize>,
    last_kernel: Cell<Option<CollisionKernel>>,
    result: Result<CollisionGpuOutput, CollisionGpuError>,
}

impl MockDevice {
    fn succeeding() -> Self {
        Self {
            calls: Cell::new(0),
            last_kernel: Cell::new(None),
            result: Ok(CollisionGpuOutput {
                nacon: 1,
                nccd: 0,
                overflow: vec![0],
                dist: vec![-0.01],
                pos: Vec::new(),
                frame: Vec::new(),
                includemargin: Vec::new(),
                friction: Vec::new(),
                solref: Vec::new(),
                solreffriction: Vec::new(),
                solimp: Vec::new(),
                adhesion: Vec::new(),
                dim: Vec::new(),
                geom: Vec::new(),
                efc_address: Vec::new(),
                worldid: Vec::new(),
                contact_type: Vec::new(),
                geomcollisionid: Vec::new(),
            }),
        }
    }

    fn failing(error: CollisionGpuError) -> Self {
        Self {
            result: Err(error),
            ..Self::succeeding()
        }
    }
}

impl CollisionDeviceRunner for MockDevice {
    fn run_device(
        &self,
        kernel: CollisionKernel,
        _inputs: &CollisionGpuInputs<'_>,
    ) -> Result<CollisionGpuOutput, CollisionGpuError> {
        self.calls.set(self.calls.get() + 1);
        self.last_kernel.set(Some(kernel));
        self.result.clone()
    }
}

fn fallback(
    inputs: &CollisionGpuInputs<'_>,
    availability: Result<(), BackendUnavailable>,
    device: Option<&MockDevice>,
) -> ProductCollisionOutcome {
    run_device_or_fallback(
        CollisionKernel::ContactMulti,
        inputs,
        availability,
        device.map(|device| device as &dyn CollisionDeviceRunner),
    )
    .unwrap()
}

/// Host 选择：生产路径直接返回 `Selected`，不探测设备、不编译 NVRTC。
#[test]
fn host_choice_returns_selected_without_touching_device() {
    let inputs = MinimalInputs::new();
    let outcome = run_product_collision(BackendChoice::Host, inputs.request(1)).unwrap();
    assert_eq!(
        outcome.host_reason(),
        Some(&ProductHostReason::Selected),
        "Host 选择应返回 Selected"
    );
    assert!(outcome.device().is_none());
}

/// ①未编译 `cuda`：Device 选择回退 Host，原因 `FeatureDisabled`（不 panic、不跑设备）。
#[test]
fn missing_cuda_feature_falls_back_with_feature_disabled() {
    let inputs = MinimalInputs::new();
    let prepared = prepared(&inputs, 1);
    let device = MockDevice::failing(CollisionGpuError::Launch("不应被调用".to_owned()));
    let outcome = fallback(
        &prepared,
        Err(BackendUnavailable::FeatureDisabled),
        Some(&device),
    );
    assert_eq!(
        outcome.host_reason(),
        Some(&ProductHostReason::FeatureDisabled)
    );
    assert_eq!(device.calls.get(), 0);
}

/// ①编译期回退：`cuda` 特性未编译时，生产可用性判定恒 `FeatureDisabled`
/// （`--no-default-features --features cuda-probe` 构建下由本用例覆盖）。
#[cfg(not(feature = "cuda"))]
#[test]
fn production_availability_is_feature_disabled_without_cuda_feature() {
    assert_eq!(
        crate::runtime::backend::product_collision_availability(),
        Err(BackendUnavailable::FeatureDisabled)
    );
}

/// `cuda` 构建下产品入口的可用性结论由缓存窄相状态派生（后端接线一致性）。
#[cfg(feature = "cuda")]
#[test]
fn production_availability_follows_cached_narrowphase_status() {
    use crate::runtime::backend::{BackendStatus, narrowphase_backend_status};

    match (
        crate::runtime::backend::product_collision_availability(),
        narrowphase_backend_status(),
    ) {
        (Ok(()), BackendStatus::CudaReady) => {}
        (Err(reason), BackendStatus::HostOnly(status_reason)) => {
            assert_eq!(&reason, status_reason, "不可用原因须与缓存窄相状态一致");
        }
        (availability, status) => {
            panic!("产品入口可用性 {availability:?} 与缓存窄相状态 {status:?} 不一致")
        }
    }
}

/// ②设备/程序不可用：各探测原因原样保留为可诊断的 `DeviceUnavailable`。
#[test]
fn device_unavailable_reasons_survive_as_diagnostics() {
    let inputs = MinimalInputs::new();
    let prepared = prepared(&inputs, 1);
    let reasons = [
        BackendUnavailable::DriverUnavailable,
        BackendUnavailable::NoDevice("无可用 CUDA 设备".to_owned()),
        BackendUnavailable::NvrtcUnavailable,
        BackendUnavailable::Compile("注入编译失败".to_owned()),
    ];
    for reason in reasons {
        let device = MockDevice::failing(CollisionGpuError::Launch("不应被调用".to_owned()));
        let outcome = fallback(&prepared, Err(reason.clone()), Some(&device));
        assert_eq!(
            outcome.host_reason(),
            Some(&ProductHostReason::DeviceUnavailable(reason)),
            "不可用原因须原样保留"
        );
        assert_eq!(device.calls.get(), 0);
    }
}

/// ②探测通过但未取得设备程序：回退 Host 并带明确诊断（不 panic）。
#[test]
fn missing_device_program_is_reported() {
    let inputs = MinimalInputs::new();
    let prepared = prepared(&inputs, 1);
    let outcome = fallback(&prepared, Ok(()), None);
    match outcome.host_reason() {
        Some(ProductHostReason::DeviceFailure(detail)) => {
            assert!(detail.contains("设备程序不可用"), "detail={detail}");
        }
        other => panic!("期望 DeviceFailure，得到 {other:?}"),
    }
}

/// ③运行期失败：执行器错误转成 Host 回退原因（保留错误文本、不 panic）。
#[test]
fn runtime_failure_falls_back_to_host_with_error_text() {
    let inputs = MinimalInputs::new();
    let prepared = prepared(&inputs, 1);
    let device = MockDevice::failing(CollisionGpuError::Launch("注入内核执行失败".to_owned()));
    let outcome = fallback(&prepared, Ok(()), Some(&device));
    match outcome.host_reason() {
        Some(ProductHostReason::DeviceFailure(detail)) => {
            assert!(detail.contains("注入内核执行失败"), "detail={detail}");
        }
        other => panic!("期望 DeviceFailure，得到 {other:?}"),
    }
    assert_eq!(device.calls.get(), 1, "运行期失败应已调用一次设备执行器");
}

/// 成功路径：设备输出原样返回，内核种类透传给执行器。
#[test]
fn device_success_returns_device_output_and_passes_kernel() {
    let inputs = MinimalInputs::new();
    let prepared = prepared(&inputs, 1);
    let device = MockDevice::succeeding();
    let outcome = fallback(&prepared, Ok(()), Some(&device));
    let output = outcome.device().expect("成功路径应返回 Device 输出");
    assert_eq!(output.nacon, 1);
    assert_eq!(output.dist, vec![-0.01]);
    assert!(outcome.host_reason().is_none());
    assert_eq!(
        device.last_kernel.get(),
        Some(CollisionKernel::ContactMulti),
        "内核种类须原样透传"
    );
}

/// nworld 口径：只接受 1；`nworld>1` 与 `nworld=0` 都在任何设备探测之前被明确拒绝。
#[test]
fn rejects_non_single_world_before_device_work() {
    let inputs = MinimalInputs::new();
    for nworld in [0, 2] {
        let error = run_product_collision(BackendChoice::Device, inputs.request(nworld))
            .expect_err("非单 world 必须在设备工作之前被拒绝");
        match error {
            CollisionGpuError::InvalidArgument(message) => {
                assert!(message.contains("nworld=1"), "message={message}");
                assert!(message.contains("另立"), "message={message}");
            }
            other => panic!("期望 InvalidArgument，得到 {other:?}"),
        }
    }
}

/// 非法快照（geom_xpos 长度不符）在设备探测之前由 S2 adapter 校验拒绝。
#[test]
fn invalid_snapshot_is_rejected_before_device_work() {
    let inputs = MinimalInputs::new();
    let mut request = inputs.request(1);
    request.snapshot.geom_xpos = &[];
    let error =
        run_product_collision(BackendChoice::Host, request).expect_err("非法快照必须被拒绝");
    match error {
        CollisionGpuError::InvalidArgument(message) => {
            assert!(message.contains("geom_xpos"), "message={message}");
        }
        other => panic!("期望 InvalidArgument，得到 {other:?}"),
    }
}
