//! G01常驻设备链与等价入口。
//! 旧探针继续保留严格检查。

// 子模块：纯移动自原单文件，行为不变。
#[cfg(feature = "cuda-probe")]
#[path = "resident/attached_kernels.rs"]
mod attached_kernels;
#[cfg(feature = "cuda-probe")]
#[path = "resident/attached_launcher_abi.rs"]
mod attached_launcher_abi;
#[cfg(feature = "cuda-probe")]
#[path = "resident/attached_launcher_meta.rs"]
mod attached_launcher_meta;
#[cfg(feature = "cuda-probe")]
#[path = "resident/attached_launcher_specs.rs"]
mod attached_launcher_specs;
#[cfg(feature = "cuda-probe")]
#[path = "resident/attached_metadata_buffer.rs"]
mod attached_metadata_buffer;
mod checks;
#[cfg(feature = "cuda-probe")]
#[path = "resident/com_kernels.rs"]
mod com_kernels;
#[cfg(feature = "cuda-probe")]
#[path = "resident/com_launcher_abi.rs"]
mod com_launcher_abi;
#[cfg(feature = "cuda-probe")]
#[path = "resident/com_launcher_meta.rs"]
mod com_launcher_meta;
#[cfg(feature = "cuda-probe")]
#[path = "resident/com_launcher_specs.rs"]
mod com_launcher_specs;
#[cfg(feature = "cuda-probe")]
#[path = "resident/com_metadata_buffer.rs"]
mod com_metadata_buffer;
mod data;
mod layout;
mod output;
mod pack;
mod pack_batches;
mod plan_accessors;
mod plan_constructors;
mod plan_contexts;
mod plan_data;
mod plan_subsets;
mod plan_update_flex;
mod plan_update_forward;
mod plan_update_rigid;
mod public;
#[cfg(feature = "cuda-probe")]
mod rigid_upload;
mod sleep_copy;

#[cfg(all(test, feature = "cuda-probe"))]
mod rigid_kinematics_tests;
#[cfg(test)]
#[path = "resident/tests/edges_faces.rs"]
mod tests_edges_faces;
#[cfg(test)]
#[path = "resident/tests/g01.rs"]
mod tests_g01;
#[cfg(test)]
#[path = "resident/tests/hessian.rs"]
mod tests_hessian;
#[cfg(test)]
#[path = "resident/tests/kinematics.rs"]
mod tests_kinematics;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "resident/tests/launcher_attached.rs"]
mod tests_launcher_attached;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "resident/tests/launcher_com.rs"]
mod tests_launcher_com;
#[cfg(test)]
#[path = "resident/tests/parameters.rs"]
mod tests_parameters;
#[cfg(test)]
#[path = "resident/tests/passive.rs"]
mod tests_passive;
#[cfg(test)]
#[path = "resident/tests/reads.rs"]
mod tests_reads;
#[cfg(test)]
#[path = "resident/tests/stretch.rs"]
mod tests_stretch;
#[cfg(test)]
#[path = "resident/tests/support.rs"]
mod tests_support;
#[cfg(test)]
#[path = "resident/tests/tendons.rs"]
mod tests_tendons;

pub use self::public::{camlight, com_pos, flex, fwd_kinematics, kinematics, tendon};

use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use self::attached_kernels::{ResidentAttachedKernel, ResidentInitializeAttachedKernel};
#[cfg(feature = "cuda-probe")]
use self::attached_metadata_buffer::ResidentAttachedMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use self::com_kernels::ResidentComKernel;
#[cfg(feature = "cuda-probe")]
use self::com_metadata_buffer::ResidentComMetadataBuffer;
use super::attached::AttachedLayout;
use super::camlight::CamLightLayout;
use super::fixed_tendon::FixedTendonLayout;
use super::flex::FlexPositionLayout;
use super::flex_edge::FlexEdgeLayout;
use super::flex_face::FlexFaceLayout;
use super::flex_hessian::FlexHessianLayout;
use super::sleep::SleepTreeLayout;
use super::spatial_tendon::SpatialTendonLayout;
use super::tendon::TendonLayout;
use super::{
    AttachedKinematicsOutput, CamLightOutput, ComPositionLayout, ComPositionOutput,
    FixedTendonOutput, FlexEdgeOutput, FlexFaceOutput, FlexHessianOutput, FlexPositionOutput,
    KinematicsLayout, KinematicsOutput, SleepTreeOutput, SpatialTendonOutput, TendonOutput,
    check_com_position, check_kinematics,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::sleep::TendonWakeInfo;
use crate::model::{
    AttachedModelInput, BatchLayout, CamLightFields, CamLightModelInput, CamLightParameters,
    FixedTendonFields, FixedTendonModelInput, FixedTendonRows, FlexEdgeFields, FlexFaceFields,
    FlexHessianFields, FlexPositionFields, FlexPositionModelInput, KinematicsParameter,
    KinematicsParameters, MocapModelInput, ParameterBatch, SleepTreeState, SpatialTendonFields,
    SpatialTendonModelInput, SpatialTendonRows, TendonModelInput, TendonRows, TendonWakeModelInput,
};
#[cfg(feature = "cuda-probe")]
use crate::model::{CamLightParameter, SpatialTendonGeometry, TendonSubset};
use crate::runtime::TransferSession;
#[cfg(all(test, feature = "cuda-probe"))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 一次上传并编译的运动学计划。
/// 一个计划可以创建多组独立状态。
/// 既有模型参数独立按世界取模。
/// 模型拓扑仍只共享一份。
/// 默认状态使用检查后的qpos0。
/// 保留探针的严格输入限制。
/// 支持mocap与静态geom缓存。
/// 支持相机与光源位姿。
/// 不冻结正式GPU编译路线。
pub struct KinematicsPlan {
    equivalent: bool,
    model: Arc<AttachedModelInput>,
    parameters: KinematicsParameters,
    body_mocapid: Vec<i32>,
    nmocap: usize,
    ncam: usize,
    nlight: usize,
    camlight_parameters: CamLightParameters,
    fixed_tendon_rows: Arc<FixedTendonRows>,
    spatial_tendon_rows: Arc<SpatialTendonRows>,
    tendon_rows: Option<Arc<TendonRows>>,
    flex_fields: Option<Arc<FlexPositionFields>>,
    edge_fields: Option<Arc<FlexEdgeFields>>,
    face_fields: Option<Arc<FlexFaceFields>>,
    hessian_fields: Option<Arc<FlexHessianFields>>,
    sleep_info: Option<Arc<TendonWakeInfo>>,
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    rigid: super::rigid_kinematics::RigidModel,
    #[cfg(feature = "cuda-probe")]
    com: DeviceCom,
    #[cfg(feature = "cuda-probe")]
    attached: DeviceAttached,
    #[cfg(feature = "cuda-probe")]
    rigid_kernel: super::rigid_kinematics::RigidKernel,
    #[cfg(feature = "cuda-probe")]
    com_kernel: ResidentComKernel,
    #[cfg(feature = "cuda-probe")]
    attached_kernel: ResidentAttachedKernel,
    #[cfg(feature = "cuda-probe")]
    initialize_kernel: ResidentInitializeAttachedKernel,
    #[cfg(feature = "cuda-probe")]
    camlight: Option<DeviceCamLight>,
    #[cfg(feature = "cuda-probe")]
    fixed_tendon: Option<DeviceFixedTendon>,
    #[cfg(feature = "cuda-probe")]
    spatial_tendon: Option<DeviceSpatialTendon>,
    #[cfg(feature = "cuda-probe")]
    tendon: Option<DeviceTendon>,
    #[cfg(feature = "cuda-probe")]
    flex: Option<DeviceFlexPositions>,
    #[cfg(feature = "cuda-probe")]
    edges: Option<DeviceFlexEdges>,
    #[cfg(feature = "cuda-probe")]
    faces: Option<DeviceFlex>,
    #[cfg(feature = "cuda-probe")]
    hessian: Option<DeviceFlexHessian>,
    #[cfg(feature = "cuda-probe")]
    sleep: Option<DeviceSleep>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceCom {
    pub(super) metadata: ResidentComMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceAttached {
    pub(super) metadata: ResidentAttachedMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct VelocityPositionKernels {
    pub(super) com: ResidentComKernel,
}

#[derive(Clone, Copy, Debug)]
struct ResidentLayout {
    qpos: BatchLayout,
    mocap_pos: BatchLayout,
    mocap_quat: BatchLayout,
    state: BatchLayout,
    rigid: KinematicsLayout,
    com: ComPositionLayout,
    attached: AttachedLayout,
    camlight: CamLightLayout,
    fixed_tendon: FixedTendonLayout,
    spatial_tendon: SpatialTendonLayout,
    tendon: Option<TendonLayout>,
    flex: Option<FlexPositionLayout>,
    edges: Option<FlexEdgeLayout>,
    faces: Option<FlexFaceLayout>,
    hessian: Option<FlexHessianLayout>,
    sleep: Option<SleepTreeLayout>,
}

#[derive(Default)]
struct OptionalSubsets {
    equivalent: bool,
    tendon_rows: Option<TendonRows>,
    flex_fields: Option<FlexPositionFields>,
    edge_fields: Option<FlexEdgeFields>,
    face_fields: Option<FlexFaceFields>,
    hessian_fields: Option<FlexHessianFields>,
    sleep_info: Option<TendonWakeInfo>,
}

/// G01消费的调用方状态。
/// 矩阵缓存按世界连续展开。
/// world_pose采用xyz与wxyz。
/// 缺省缓存保留已有设备值。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct G01State {
    pub qpos: Vec<f32>,
    pub qvel: Vec<f32>,
    pub mocap_pos: Vec<f32>,
    pub mocap_quat: Vec<f32>,
    /// 每世界按七项刚体字段展开。
    pub rigid_cache: Option<Vec<f32>>,
    /// 每世界按质量、质心、惯量、cdof展开。
    pub com_cache: Option<Vec<f32>>,
    /// 本字段覆盖刚体缓存的世界位姿。
    pub world_pose: Option<Vec<[f32; 7]>>,
    pub attached_cache: Option<Vec<f32>>,
    pub flex_hessian_valid: Option<Vec<bool>>,
    pub sleep: Option<Vec<G01SleepState>>,
}

/// G01树状态保留独立活动标记。
/// 调用方负责活动标记一致性。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct G01SleepState {
    pub tree_asleep: Vec<i32>,
    pub tree_awake: Vec<i32>,
    pub ntree_awake: i32,
    pub nbody_awake: i32,
    pub nv_awake: i32,
}

#[derive(Default, Debug)]
struct ReadyStages {
    rigid: bool,
    com: bool,
    attached: bool,
    camlight: bool,
    fixed_tendon: bool,
    spatial_tendon: bool,
    tendon: bool,
    flex: bool,
    edges: bool,
    faces: bool,
    hessian: bool,
    sleep: bool,
}

/// 独占多世界状态与设备输出。
/// 它记录具体设备模型的身份。
/// 相同尺寸不允许跨计划使用。
/// 状态写入会废弃派生结果。
/// 更新完成只表示驱动执行成功。
/// 显式回读检查守卫与有限性。
/// 初始化缓存不代表阶段就绪。
pub struct KinematicsData {
    equivalent: bool,
    model: Arc<AttachedModelInput>,
    fixed_tendon_rows: Arc<FixedTendonRows>,
    spatial_tendon_rows: Arc<SpatialTendonRows>,
    tendon_rows: Option<Arc<TendonRows>>,
    edge_fields: Option<Arc<FlexEdgeFields>>,
    face_fields: Option<Arc<FlexFaceFields>>,
    hessian_fields: Option<Arc<FlexHessianFields>>,
    layout: ResidentLayout,
    ready: ReadyStages,
    #[cfg(feature = "cuda-probe")]
    state: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    rigid: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    com: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    attached: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    camlight: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    fixed_tendon: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    spatial_tendon: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    spatial_wrap: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    tendon: Option<ResidentTendon>,
    #[cfg(feature = "cuda-probe")]
    flex: Option<TransferBuffer<f32>>,
    #[cfg(feature = "cuda-probe")]
    flex_validity: Option<TransferBuffer<i32>>,
    #[cfg(feature = "cuda-probe")]
    edges: Option<ResidentFlexEdges>,
    #[cfg(feature = "cuda-probe")]
    faces: Option<TransferBuffer<f32>>,
    #[cfg(feature = "cuda-probe")]
    hessian: Option<TransferBuffer<f32>>,
    #[cfg(feature = "cuda-probe")]
    sleep: Option<ResidentSleep>,
}

/// 一次显式回读的完整子集结果。
/// 它独立拥有全部宿主缓冲。
/// 后续状态更新不修改此快照。
#[derive(Clone, Debug)]
pub struct KinematicsSnapshot {
    attached: AttachedKinematicsOutput,
    com: ComPositionOutput,
    camlight: CamLightOutput,
    fixed_tendon: FixedTendonOutput,
    spatial_tendon: SpatialTendonOutput,
    tendon: Option<TendonOutput>,
    flex: Option<FlexPositionOutput>,
    edges: Option<FlexEdgeOutput>,
    faces: Option<FlexFaceOutput>,
    hessian: Option<FlexHessianOutput>,
    sleep: Option<SleepTreeOutput>,
}

#[cfg(feature = "cuda-probe")]
use self::layout::{
    DeviceCamLight, DeviceFixedTendon, DeviceFlex, DeviceFlexEdges, DeviceFlexHessian,
    DeviceFlexPositions, DeviceSpatialTendon, DeviceTendon, ResidentFlexEdges, ResidentTendon,
};
#[cfg(feature = "cuda-probe")]
use self::sleep_copy::{DeviceSleep, ResidentSleep};
