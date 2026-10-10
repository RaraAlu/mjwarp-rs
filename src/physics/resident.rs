//! G01常驻设备链与等价入口。
//! 旧探针继续保留严格检查。

use std::sync::Arc;

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
    KinematicsParameters, MocapModelInput, SleepTreeState, SpatialTendonFields,
    SpatialTendonModelInput, SpatialTendonRows, TendonModelInput, TendonRows, TendonWakeModelInput,
};
#[cfg(feature = "cuda-probe")]
use crate::model::{CamLightParameter, SpatialTendonGeometry, TendonSubset};
use crate::runtime::TransferSession;
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
    rigid: super::DeviceModel<f32>,
    #[cfg(feature = "cuda-probe")]
    com: super::DeviceModel<f32>,
    #[cfg(feature = "cuda-probe")]
    attached: super::DeviceModel<f32>,
    #[cfg(feature = "cuda-probe")]
    rigid_kernel: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    com_kernel: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    attached_kernel: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    initialize_kernel: SynchronousKernel,
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
pub(super) struct VelocityPositionKernels {
    rigid: SynchronousKernel,
    com: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceSleep {
    model: super::DeviceModel<f32>,
    copy_kernel: SynchronousKernel,
    wake_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct ResidentSleep {
    state: TransferBuffer<i32>,
    scratch: TransferBuffer<i32>,
}

#[cfg(feature = "cuda-probe")]
struct DeviceFlex {
    model: super::DeviceModel<f32>,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceFlexPositions {
    model: super::DeviceModel<f32>,
    kernel: SynchronousKernel,
    invalidate_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceFlexHessian {
    model: super::DeviceModel<f32>,
    kernel: SynchronousKernel,
    validate_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceFlexEdges {
    metadata: TransferBuffer<i32>,
    kernel: SynchronousKernel,
}
#[cfg(feature = "cuda-probe")]
struct ResidentFlexEdges {
    qvel: TransferBuffer<f32>,
    values: TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
struct DeviceTendon {
    model: super::DeviceModel<f32>,
    values_kernel: SynchronousKernel,
    indices_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct ResidentTendon {
    values: TransferBuffer<f32>,
    indices: TransferBuffer<i32>,
}

#[cfg(feature = "cuda-probe")]
struct DeviceSpatialTendon {
    model: super::DeviceModel<f32>,
    site_kernel: SynchronousKernel,
    moment_kernel: SynchronousKernel,
    wrap_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceFixedTendon {
    model: super::DeviceModel<f32>,
    kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
struct DeviceCamLight {
    model: super::DeviceModel<f32>,
    rigid_kernel: SynchronousKernel,
    com_kernel: SynchronousKernel,
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
impl G01SleepState {
    fn validate(&self, ntree: usize, nbody: usize, nv: usize) -> Result<(), InputError> {
        SleepTreeState {
            tree_asleep: self.tree_asleep.clone(),
            nbody_awake: self.nbody_awake,
            nv_awake: self.nv_awake,
        }
        .validate(ntree, nbody, nv)?;
        check_size("tree_awake", ntree, self.tree_awake.len())?;
        if self.ntree_awake < 0 || self.ntree_awake as usize > ntree {
            return Err(InputError::InvalidDimension {
                field: "ntree_awake",
            });
        }
        for (index, &value) in self.tree_awake.iter().enumerate() {
            if value != 0 && value != 1 {
                return Err(InputError::InvalidTopology {
                    field: "tree_awake",
                    index,
                    reason: "noncanonical_boolean",
                });
            }
        }
        Ok(())
    }
    #[cfg(feature = "cuda-probe")]
    fn packed(&self) -> Vec<i32> {
        let mut result = self.tree_asleep.clone();
        result.extend_from_slice(&self.tree_awake);
        result.extend_from_slice(&[self.ntree_awake, self.nbody_awake, self.nv_awake]);
        result
    }
}

/// 完整G01入口不要求新鲜度票据。
/// 调用方提供一致的模型与状态。
/// 本入口不执行碰撞或质量阶段。
pub fn fwd_kinematics(
    plan: &KinematicsPlan,
    data: &mut KinematicsData,
) -> Result<(), TransferError> {
    kinematics(plan, data)?;
    com_pos(plan, data)?;
    camlight(plan, data)?;
    flex(plan, data)?;
    tendon(plan, data)?;
    plan.update_tendon_wake(data)
}

/// 重算刚体与附着位姿。
/// 保留世界位姿与静态几何缓存。
pub fn kinematics(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_rigid(data)?;
    plan.update_attached(data)
}

/// 读取已有刚体设备字段。
/// 调用方负责输入的物理一致性。
pub fn com_pos(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_com(data)
}

/// 读取已有刚体与质心字段。
/// 本入口不隐式重算前置阶段。
pub fn camlight(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_camlight(data)
}

/// 重算柔体位置、边与壳体面。
/// 读取已有刚体、质心与qvel。
/// 先清除Hessian有效标志。
pub fn flex(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_flex_positions(data)?;
    plan.update_flex_edges(data)?;
    plan.update_flex_faces(data)
}

/// 重算全局肌腱与包裹字段。
/// 读取已有附着、质心与qpos。
/// 本入口不执行肌腱唤醒。
pub fn tendon(plan: &KinematicsPlan, data: &mut KinematicsData) -> Result<(), TransferError> {
    plan.check_g01_data(data)?;
    plan.update_tendons(data)
}

impl ResidentLayout {
    fn new(model: &AttachedModelInput, worlds: usize, nmocap: usize) -> Result<Self, InputError> {
        let k = model.rigid().kinematics();
        let stride = nmocap
            .checked_mul(7)
            .and_then(|n| n.checked_add(k.nq()))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "resident_state",
            })?;
        Ok(Self {
            rigid: KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?,
            com: ComPositionLayout::new(worlds, k.nbody(), k.njnt(), k.nv())?,
            attached: AttachedLayout::new(worlds, model.ngeom(), model.nsite())?,
            camlight: CamLightLayout::new(worlds, 0, 0)?,
            fixed_tendon: FixedTendonLayout::new(worlds, 0, 0)?,
            spatial_tendon: SpatialTendonLayout::new(worlds, 0, 0, 0)?,
            tendon: None,
            flex: None,
            edges: None,
            faces: None,
            hessian: None,
            sleep: None,
            qpos: BatchLayout::new(worlds, k.nq(), 4)?,
            mocap_pos: BatchLayout::new(worlds, nmocap * 3, 4)?,
            mocap_quat: BatchLayout::new(worlds, nmocap * 4, 4)?,
            state: BatchLayout::new(worlds, stride, 4)?,
        })
    }
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

impl ReadyStages {
    fn require(&self, ready: bool, stage: &'static str) -> Result<(), TransferError> {
        if ready {
            Ok(())
        } else {
            Err(TransferError::StageNotReady { stage })
        }
    }
    fn require_all(&self) -> Result<(), TransferError> {
        self.require(self.rigid, "rigid")?;
        self.require(self.attached, "attached")?;
        self.require(self.com, "com")
    }
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

impl KinematicsSnapshot {
    /// 显式计算完成才提供矩阵。
    /// 位置刷新后返回None。
    pub fn flex_hessian(&self) -> Option<&FlexHessianOutput> {
        self.hessian.as_ref()
    }
    pub fn flex_faces(&self) -> Option<&FlexFaceOutput> {
        self.faces.as_ref()
    }
    pub fn flex_edges(&self) -> Option<&FlexEdgeOutput> {
        self.edges.as_ref()
    }
    pub fn sleep_trees(&self) -> Option<&SleepTreeOutput> {
        self.sleep.as_ref()
    }
    /// 柔体入口提供位置结果。
    /// 旧入口返回None。
    pub fn flex_positions(&self) -> Option<&FlexPositionOutput> {
        self.flex.as_ref()
    }
    /// 混合入口提供全局结果。
    /// 旧局部入口返回None。
    pub fn tendon(&self) -> Option<&TendonOutput> {
        self.tendon.as_ref()
    }
    pub fn spatial_tendon(&self) -> &SpatialTendonOutput {
        &self.spatial_tendon
    }
    pub fn fixed_tendon(&self) -> &FixedTendonOutput {
        &self.fixed_tendon
    }
    pub fn camlight(&self) -> &CamLightOutput {
        &self.camlight
    }
    pub fn rigid(&self) -> &KinematicsOutput {
        self.attached.rigid()
    }
    pub fn attached(&self) -> &AttachedKinematicsOutput {
        &self.attached
    }
    pub fn com(&self) -> &ComPositionOutput {
        &self.com
    }
}

impl KinematicsPlan {
    pub fn flex_hessian_fields(&self) -> Option<&FlexHessianFields> {
        self.hessian_fields.as_deref()
    }
    pub fn flex_face_fields(&self) -> Option<&FlexFaceFields> {
        self.face_fields.as_deref()
    }
    pub fn flex_edge_fields(&self) -> Option<&FlexEdgeFields> {
        self.edge_fields.as_deref()
    }
    /// 检查模型及qpos0后上传。
    /// 本辅助不接受非法默认姿态。
    /// 需要cuda-probe与NVRTC。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        Self::with_parameters(session, model, KinematicsParameters::default())
    }

    /// 先检查全部参数行，再上传。
    /// 缺省字段使用原模型共享行。
    /// 参数周期无需整除世界数。
    /// 计划创建后不允许改写参数。
    pub fn with_parameters(
        session: &TransferSession,
        model: AttachedModelInput,
        parameters: KinematicsParameters,
    ) -> Result<Self, TransferError> {
        let ids = vec![-1; model.rigid().kinematics().nbody()];
        Self::with_mocap(session, MocapModelInput::new(model, ids)?, parameters)
    }

    /// 先检查映射与全部参数行。
    /// 初始mocap使用体参数行。
    /// 静态geom使用GPU初始化。
    /// 模型参数与拓扑保持只读。
    pub fn with_mocap(
        session: &TransferSession,
        model: MocapModelInput,
        parameters: KinematicsParameters,
    ) -> Result<Self, TransferError> {
        Self::with_camlight(
            session,
            CamLightModelInput::new(model, CamLightFields::default())?,
            parameters,
            CamLightParameters::default(),
        )
    }

    /// 接收已编译的跟踪常量。
    /// 十项参数独立按世界取模。
    /// 相机与光源只计算位姿。
    pub fn with_camlight(
        session: &TransferSession,
        model: CamLightModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_fixed_tendons(
            session,
            FixedTendonModelInput::new(model, FixedTendonFields::default())?,
            parameters,
            camlight_parameters,
        )
    }

    /// 接收关节项与原生CSR。
    /// 系数与拓扑共享一份。
    /// 长度直接使用当前qpos。
    pub fn with_fixed_tendons(
        session: &TransferSession,
        model: FixedTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_spatial_tendons(
            session,
            SpatialTendonModelInput::new(model, SpatialTendonFields::default())?,
            parameters,
            camlight_parameters,
        )
    }

    /// 接收site、滑轮与球柱路径。
    /// 两种肌腱集合各用局部编号。
    pub fn with_spatial_tendons(
        session: &TransferSession,
        model: SpatialTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets::default(),
        )
    }

    /// 保留原生全局编号与CSR。
    /// GPU合并长度、力臂与包裹。
    /// 旧局部接口保持原编号。
    pub fn with_tendons(
        session: &TransferSession,
        model: TendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                ..Default::default()
            },
        )
    }

    /// 另加柔体节点与顶点位置。
    /// 可选边复用质心与qvel。
    /// 可选面只复用设备节点。
    /// 位置阶段清除Hessian标志。
    /// 保留混合肌腱全局编号。
    pub fn with_flex_positions(
        session: &TransferSession,
        model: FlexPositionModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, fields, edges, faces, hessian) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                flex_fields: Some(fields),
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: None,
                equivalent: false,
            },
        )
    }

    /// 另加肌腱唤醒副作用。
    /// 不实现完整休眠状态机。
    pub fn with_tendon_wake(
        session: &TransferSession,
        model: TendonWakeModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, flex, edges, faces, hessian, info) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                flex_fields: flex,
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: Some(info),
                equivalent: false,
            },
        )
    }

    /// 完整G01保留等价数值路径。
    /// 模型必须包含全部边面字段。
    /// 空集合也采用显式空字段。
    /// 旧构造入口继续严格检查。
    pub fn for_g01(
        session: &TransferSession,
        model: TendonWakeModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        if model.flex_positions().is_none()
            || model.flex_edges().is_none()
            || model.flex_faces().is_none()
        {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_complete_flex_fields",
            }
            .into());
        }
        let (model, flex, edges, faces, hessian, info) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                equivalent: true,
                tendon_rows: Some(rows),
                flex_fields: flex,
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: Some(info),
            },
        )
    }

    /// 先检查全部长度与引用。
    /// 输入错误不改写设备状态。
    /// 传输失败废弃就绪票据。
    /// 低层回读仍反映当前缓冲。
    pub fn import_g01_state(
        &self,
        data: &mut KinematicsData,
        state: &G01State,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        check_g01_qpos(self.model.rigid(), data.worlds(), &state.qpos)?;
        let e = data.edge_layout()?;
        check_finite_field("qvel", e.qvel.total_elements(), &state.qvel)?;
        check_mocap_mode(
            data.layout.mocap_pos.total_elements(),
            data.layout.mocap_quat.total_elements(),
            &state.mocap_pos,
            &state.mocap_quat,
            false,
        )?;
        for (cache, field, expected) in [
            (
                &state.rigid_cache,
                "rigid_cache",
                data.layout.rigid.output.total_elements(),
            ),
            (
                &state.com_cache,
                "com_cache",
                data.layout.com.output.total_elements(),
            ),
        ] {
            if let Some(cache) = cache {
                check_finite_field(field, expected, cache)?;
            }
        }
        if let Some(poses) = &state.world_pose {
            check_size("world_pose", data.worlds(), poses.len())?;
            for pose in poses {
                check_finite_field("world_pose", 7, pose)?;
            }
        }
        if let Some(cache) = &state.attached_cache {
            check_finite_field(
                "attached_cache",
                data.layout.attached.output.total_elements(),
                cache,
            )?;
        }
        if let Some(flags) = &state.flex_hessian_valid {
            check_size(
                "flex_hessian_valid",
                data.layout
                    .flex
                    .expect("complete G01 flex")
                    .validity
                    .total_elements(),
                flags.len(),
            )?;
        }
        if let Some(sleep) = &state.sleep {
            check_size("sleep", data.worlds(), sleep.len())?;
            let l = data.layout.sleep.expect("complete G01 sleep");
            let k = self.model.rigid().kinematics();
            for row in sleep {
                row.validate(l.ntree, k.nbody(), k.nv())?;
            }
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            data.ready = ReadyStages::default();
            data.write_checked_qpos(0, &state.qpos)?;
            data.write_qvel(&state.qvel)?;
            data.write_checked_mocap(0, 0, &state.mocap_pos, &state.mocap_quat)?;
            if let Some(cache) = &state.rigid_cache {
                data.rigid.write_range(4, cache)?;
            }
            if let Some(cache) = &state.com_cache {
                data.com.write_range(4, cache)?;
            }
            if let Some(poses) = &state.world_pose {
                for (world, pose) in poses.iter().enumerate() {
                    let start = 4 + world * data.layout.rigid.output.elements_per_world();
                    data.rigid.write_range(start, &pose[..3])?;
                    data.rigid
                        .write_range(start + 3 * data.layout.rigid.nbody, &pose[3..])?;
                }
            }
            if let Some(cache) = &state.attached_cache {
                data.attached.write_range(4, cache)?;
            }
            if let Some(flags) = &state.flex_hessian_valid {
                let values: Vec<i32> = flags.iter().map(|&v| i32::from(v)).collect();
                data.flex_validity
                    .as_mut()
                    .expect("complete G01 flex")
                    .write_range(4, &values)?;
            }
            if let Some(sleep) = &state.sleep {
                let l = data.layout.sleep.expect("complete G01 sleep");
                for (world, row) in sleep.iter().enumerate() {
                    let start = 4 + l.output.world_elements(world)?.start;
                    data.sleep
                        .as_mut()
                        .expect("complete G01 sleep")
                        .state
                        .write_range(start, &row.packed())?;
                }
            }
            Ok(())
        }
    }

    fn with_tendon_subsets(
        session: &TransferSession,
        model: SpatialTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
        subsets: OptionalSubsets,
    ) -> Result<Self, TransferError> {
        let OptionalSubsets {
            equivalent,
            tendon_rows,
            flex_fields,
            edge_fields,
            face_fields,
            hessian_fields,
            sleep_info,
        } = subsets;
        if let Some(rows) = &tendon_rows {
            TendonLayout::new(1, rows.ntendon(), rows.nnz(), rows.nwrap())?;
        }
        let (model, spatial_fields, spatial_rows, spatial_geometry) = model.into_parts();
        let spatial_layout = SpatialTendonLayout::new(
            1,
            spatial_rows.ntendon(),
            spatial_rows.nnz(),
            spatial_rows.nwrap(),
        )?;
        let (model, fixed_tendon_fields, fixed_tendon_rows) = model.into_parts();
        let fixed_tendon_layout =
            FixedTendonLayout::new(1, fixed_tendon_rows.ntendon(), fixed_tendon_rows.nnz())?;
        if equivalent {
            camlight_parameters.validate_mode(&model, false)?;
        } else {
            camlight_parameters.validate(&model)?;
        }
        let ncam = model.ncam();
        let nlight = model.nlight();
        let camlight_layout = CamLightLayout::new(1, ncam, nlight)?;
        let (model, camlight_fields) = model.into_parts();
        let (model, body_mocapid, nmocap, static_geom) = model.into_parts();
        let k = model.rigid().kinematics();
        let (fk, com) = if equivalent {
            (
                check_g01_qpos(model.rigid(), 1, &k.fields().qpos0)?,
                ComPositionLayout::new(1, k.nbody(), k.njnt(), k.nv())?,
            )
        } else {
            check_com_position(model.rigid(), 1, &k.fields().qpos0)?
        };
        let attached = AttachedLayout::new(1, model.ngeom(), model.nsite())?;
        if equivalent {
            parameters.validate(&model)?;
        } else {
            check_parameters(&model, &parameters)?;
        }
        let flex_layout = flex_fields
            .as_ref()
            .map(|f| {
                FlexPositionLayout::new(
                    1,
                    f.flex_interp.len(),
                    f.flex_nodebodyid.len(),
                    f.flex_vertbodyid.len(),
                )
            })
            .transpose()?;
        let sleep_layout = sleep_info
            .as_ref()
            .map(|i| SleepTreeLayout::new(1, i.ntree))
            .transpose()?;
        let edge_layout = edge_fields
            .as_ref()
            .map(|f| {
                FlexEdgeLayout::new(1, f.nflexedge(), f.nnz(), model.rigid().kinematics().nv())
            })
            .transpose()?;
        let face_layout = face_fields
            .as_ref()
            .map(|f| FlexFaceLayout::new(1, f.nflexface()))
            .transpose()?;
        let hessian_layout = hessian_fields
            .as_ref()
            .map(|_| {
                FlexHessianLayout::new(
                    1,
                    flex_fields
                        .as_ref()
                        .expect("checked flex fields")
                        .flex_vertbodyid
                        .len(),
                    edge_fields
                        .as_ref()
                        .expect("checked edge fields")
                        .nflexedge(),
                )
            })
            .transpose()?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (
                session,
                fk,
                com,
                attached,
                body_mocapid,
                nmocap,
                static_geom,
                camlight_fields,
                camlight_layout,
                fixed_tendon_layout,
                fixed_tendon_fields,
                fixed_tendon_rows,
                spatial_fields,
                spatial_rows,
                spatial_layout,
                spatial_geometry,
                tendon_rows,
                flex_fields,
                flex_layout,
                sleep_info,
                sleep_layout,
                edge_fields,
                edge_layout,
                face_fields,
                face_layout,
                hessian_fields,
                hessian_layout,
            );
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let com_stride = com.output.elements_per_world();
            let _ = (attached, sleep_layout);
            let k = model.rigid().kinematics().fields();
            let a = model.fields();
            // Pack and check all descriptors before any device allocation.
            let rigid = pack_model(
                &model,
                &parameters,
                &[
                    &k.body_parentid,
                    &k.body_jntadr,
                    &k.body_jntnum,
                    &k.jnt_type,
                    &k.jnt_qposadr,
                    &[nmocap as i32],
                    &body_mocapid,
                ],
                &RIGID_PARAMETERS,
            )?;
            let com = pack_model(
                &model,
                &parameters,
                &[&k.body_parentid, &k.jnt_type, &k.jnt_bodyid, &k.jnt_dofadr],
                &COM_PARAMETERS,
            )?;
            let static_geom: Vec<i32> = static_geom.into_iter().map(i32::from).collect();
            let attached = pack_model(
                &model,
                &parameters,
                &[
                    &[fk.output.elements_per_world() as i32],
                    &a.geom_bodyid,
                    &a.site_bodyid,
                    &static_geom,
                ],
                &ATTACHED_PARAMETERS,
            )?;
            let camlight = pack_camlight(
                &camlight_fields,
                &camlight_parameters,
                fk.output.elements_per_world(),
                com_stride,
            )?;
            let fixed_tendon = pack_fixed_tendon(&model, &fixed_tendon_fields)?;
            let spatial_tendon =
                pack_spatial_tendon(&model, &spatial_fields, spatial_geometry.as_ref())?;
            let tendon = tendon_rows
                .as_ref()
                .map(|rows| pack_tendon(rows, &fixed_tendon_rows, &spatial_rows))
                .transpose()?;
            let flex = flex_fields
                .as_ref()
                .map(|f| pack_flex(f, fk.output.elements_per_world(), fk.nbody))
                .transpose()?;
            let sleep = sleep_info
                .as_ref()
                .map(|i| pack_sleep(i, tendon_rows.as_ref().expect("checked global tendons")))
                .transpose()?;
            let edges = edge_fields
                .as_ref()
                .map(|f| {
                    pack_flex_edges(
                        &model,
                        flex_fields.as_ref().expect("checked flex positions"),
                        f,
                        com_stride,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let faces = face_fields
                .as_ref()
                .map(|f| {
                    pack_flex_faces(
                        flex_fields.as_ref().expect("checked flex positions"),
                        f,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let hessian = hessian_fields
                .as_ref()
                .map(|h| {
                    pack_flex_hessian(
                        flex_fields.as_ref().expect("checked flex fields"),
                        edge_fields.as_ref().expect("checked edge fields"),
                        h,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                        edge_layout
                            .expect("checked edge layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let rigid = rigid.upload(session)?;
            let com = com.upload(session)?;
            let attached = attached.upload(session)?;
            let mode = if equivalent {
                "#define MJWARP_G01_EQUIVALENT\n"
            } else {
                ""
            };
            let prefix = format!("{FIELD_BATCH_CUDA}\n#define MJWARP_MOCAP\n{mode}");
            let source = format!("{prefix}{}", super::KINEMATICS_CUDA);
            let rigid_kernel = SynchronousKernel::compile(session, &source, "rigid_kinematics")?;
            let source = format!("{FIELD_BATCH_CUDA}\n{}", super::COM_POSITION_CUDA);
            let com_kernel = SynchronousKernel::compile(session, &source, "rigid_com_position")?;
            let source = format!(
                "{prefix}{}\n{}",
                super::KINEMATICS_CUDA,
                super::attached::ATTACHED_CUDA
            );
            let attached_kernel =
                SynchronousKernel::compile(session, &source, "attached_kinematics")?;
            let initialize_kernel =
                SynchronousKernel::compile(session, &source, "initialize_attached")?;
            let camlight = if camlight_layout.is_empty() {
                None
            } else {
                let source = format!(
                    "{FIELD_BATCH_CUDA}\n{}\n{}",
                    super::KINEMATICS_CUDA,
                    super::camlight::CAMLIGHT_CUDA
                );
                Some(DeviceCamLight {
                    model: camlight.upload(session)?,
                    rigid_kernel: SynchronousKernel::compile(session, &source, "camlight_rigid")?,
                    com_kernel: SynchronousKernel::compile(session, &source, "camlight_com")?,
                })
            };
            let fixed_tendon = if fixed_tendon_layout.is_empty() {
                None
            } else {
                Some(DeviceFixedTendon {
                    model: fixed_tendon.upload(session)?,
                    kernel: SynchronousKernel::compile(
                        session,
                        super::fixed_tendon::FIXED_TENDON_CUDA,
                        "fixed_tendon",
                    )?,
                })
            };
            let spatial_tendon = if spatial_layout.is_empty() {
                None
            } else {
                let source = format!(
                    "{}\n{}",
                    super::tendon_wrap::TENDON_WRAP_CUDA,
                    super::spatial_tendon::SPATIAL_TENDON_CUDA
                );
                Some(DeviceSpatialTendon {
                    model: spatial_tendon.upload(session)?,
                    site_kernel: SynchronousKernel::compile(session, &source, "spatial_site")?,
                    moment_kernel: SynchronousKernel::compile(session, &source, "spatial_moment")?,
                    wrap_kernel: SynchronousKernel::compile(session, &source, "spatial_wrap")?,
                })
            };
            let tendon = if let Some(tendon) = tendon {
                Some(DeviceTendon {
                    model: tendon.upload(session)?,
                    values_kernel: SynchronousKernel::compile(
                        session,
                        super::tendon::TENDON_CUDA,
                        "tendon_values",
                    )?,
                    indices_kernel: SynchronousKernel::compile(
                        session,
                        super::tendon::TENDON_CUDA,
                        "tendon_indices",
                    )?,
                })
            } else {
                None
            };
            let flex = if flex_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlexPositions {
                    model: flex.expect("checked flex fields").upload(session)?,
                    invalidate_kernel: SynchronousKernel::compile(
                        session,
                        super::flex::FLEX_POSITION_CUDA,
                        "flex_invalidate",
                    )?,
                    kernel: SynchronousKernel::compile(
                        session,
                        super::flex::FLEX_POSITION_CUDA,
                        "flex_positions",
                    )?,
                })
            } else {
                None
            };
            let sleep = if let Some(sleep) = sleep {
                Some(DeviceSleep {
                    model: sleep.upload(session)?,
                    copy_kernel: SynchronousKernel::compile(
                        session,
                        super::sleep::TENDON_WAKE_CUDA,
                        "sleep_copy",
                    )?,
                    wake_kernel: SynchronousKernel::compile(
                        session,
                        super::sleep::TENDON_WAKE_CUDA,
                        "tendon_wake",
                    )?,
                })
            } else {
                None
            };
            let edges = if edge_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlexEdges {
                    metadata: session.upload(&edges.expect("checked edge fields"))?,
                    kernel: SynchronousKernel::compile(
                        session,
                        super::flex_edge::FLEX_EDGE_CUDA,
                        "flex_edges",
                    )?,
                })
            } else {
                None
            };
            let faces = if face_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlex {
                    model: faces.expect("checked face fields").upload(session)?,
                    kernel: SynchronousKernel::compile(
                        session,
                        super::flex_face::FLEX_FACE_CUDA,
                        "flex_faces",
                    )?,
                })
            } else {
                None
            };
            let hessian = if hessian_layout.is_some_and(|l| l.output.elements_per_world() != 0) {
                Some(DeviceFlexHessian {
                    model: hessian.expect("checked hessian fields").upload(session)?,
                    kernel: SynchronousKernel::compile(
                        session,
                        super::flex_hessian::FLEX_HESSIAN_CUDA,
                        "flex_hessian",
                    )?,
                    validate_kernel: SynchronousKernel::compile(
                        session,
                        super::flex_hessian::FLEX_HESSIAN_CUDA,
                        "flex_hessian_validate",
                    )?,
                })
            } else {
                None
            };
            Ok(Self {
                equivalent,
                model: Arc::new(model),
                parameters,
                body_mocapid,
                nmocap,
                ncam,
                nlight,
                camlight_parameters,
                fixed_tendon_rows: Arc::new(fixed_tendon_rows),
                spatial_tendon_rows: Arc::new(spatial_rows),
                tendon_rows: tendon_rows.map(Arc::new),
                flex_fields: flex_fields.map(Arc::new),
                edge_fields: edge_fields.map(Arc::new),
                face_fields: face_fields.map(Arc::new),
                hessian_fields: hessian_fields.map(Arc::new),
                sleep_info: sleep_info.map(Arc::new),
                session: session.clone(),
                rigid,
                com,
                attached,
                rigid_kernel,
                com_kernel,
                attached_kernel,
                initialize_kernel,
                camlight,
                fixed_tendon,
                spatial_tendon,
                tendon,
                flex,
                edges,
                faces,
                hessian,
                sleep,
            })
        }
    }

    pub fn device(&self) -> usize {
        self.session.device()
    }

    pub fn parameters(&self) -> &KinematicsParameters {
        &self.parameters
    }

    pub fn nmocap(&self) -> usize {
        self.nmocap
    }

    pub fn body_mocapid(&self) -> &[i32] {
        &self.body_mocapid
    }

    pub fn camlight_parameters(&self) -> &CamLightParameters {
        &self.camlight_parameters
    }
    pub fn ncam(&self) -> usize {
        self.ncam
    }
    pub fn nlight(&self) -> usize {
        self.nlight
    }
    pub fn fixed_tendon_rows(&self) -> &FixedTendonRows {
        &self.fixed_tendon_rows
    }
    pub fn spatial_tendon_rows(&self) -> &SpatialTendonRows {
        &self.spatial_tendon_rows
    }
    pub fn tendon_rows(&self) -> Option<&TendonRows> {
        self.tendon_rows.as_deref()
    }

    /// 创建独立世界与设备输出。
    /// 全部尺寸通过后才申请显存。
    /// 初始派生结果保持未就绪。
    pub fn create_data(&self, worlds: usize) -> Result<KinematicsData, TransferError> {
        let mut layout = ResidentLayout::new(&self.model, worlds, self.nmocap)?;
        layout.camlight = CamLightLayout::new(worlds, self.ncam, self.nlight)?;
        layout.fixed_tendon = FixedTendonLayout::new(
            worlds,
            self.fixed_tendon_rows.ntendon(),
            self.fixed_tendon_rows.nnz(),
        )?;
        layout.spatial_tendon = SpatialTendonLayout::new(
            worlds,
            self.spatial_tendon_rows.ntendon(),
            self.spatial_tendon_rows.nnz(),
            self.spatial_tendon_rows.nwrap(),
        )?;
        layout.tendon = self
            .tendon_rows
            .as_ref()
            .map(|r| TendonLayout::new(worlds, r.ntendon(), r.nnz(), r.nwrap()))
            .transpose()?;
        layout.flex = self
            .flex_fields
            .as_ref()
            .map(|f| {
                FlexPositionLayout::new(
                    worlds,
                    f.flex_interp.len(),
                    f.flex_nodebodyid.len(),
                    f.flex_vertbodyid.len(),
                )
            })
            .transpose()?;
        layout.sleep = self
            .sleep_info
            .as_ref()
            .map(|i| SleepTreeLayout::new(worlds, i.ntree))
            .transpose()?;
        layout.edges = self
            .edge_fields
            .as_ref()
            .map(|f| {
                FlexEdgeLayout::new(
                    worlds,
                    f.nflexedge(),
                    f.nnz(),
                    self.model.rigid().kinematics().nv(),
                )
            })
            .transpose()?;
        layout.faces = self
            .face_fields
            .as_ref()
            .map(|f| FlexFaceLayout::new(worlds, f.nflexface()))
            .transpose()?;
        layout.hessian = self
            .hessian_fields
            .as_ref()
            .map(|_| {
                FlexHessianLayout::new(
                    worlds,
                    self.flex_fields
                        .as_ref()
                        .expect("checked flex fields")
                        .flex_vertbodyid
                        .len(),
                    self.edge_fields
                        .as_ref()
                        .expect("checked edge fields")
                        .nflexedge(),
                )
            })
            .transpose()?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout.state;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Three contiguous field blocks preserve independent world writes.
            // Kernel wide offsets match these checked batch lengths exactly.
            let mut state =
                crate::runtime::host_staging::<f32>(layout.state.total_elements().max(1))?;
            let nq = layout.qpos.elements_per_world();
            if nq != 0 {
                for (world, row) in state[..layout.qpos.total_elements()]
                    .chunks_exact_mut(nq)
                    .enumerate()
                {
                    row.copy_from_slice(self.parameters.world(
                        KinematicsParameter::Qpos0,
                        &self.model,
                        world,
                    ));
                }
            }
            let pos_start = layout.qpos.total_elements();
            let quat_start = pos_start + layout.mocap_pos.total_elements();
            for world in 0..worlds {
                let bp = self
                    .parameters
                    .world(KinematicsParameter::BodyPos, &self.model, world);
                let bq = self
                    .parameters
                    .world(KinematicsParameter::BodyQuat, &self.model, world);
                for (body, &id) in self
                    .body_mocapid
                    .iter()
                    .enumerate()
                    .filter(|&(_, &id)| id >= 0)
                {
                    let p = pos_start + world * 3 * self.nmocap + id as usize * 3;
                    let q = quat_start + world * 4 * self.nmocap + id as usize * 4;
                    state[p..p + 3].copy_from_slice(&bp[body * 3..body * 3 + 3]);
                    state[q..q + 4].copy_from_slice(&bq[body * 4..body * 4 + 4]);
                }
            }
            let mut data = KinematicsData {
                equivalent: self.equivalent,
                model: Arc::clone(&self.model),
                fixed_tendon_rows: Arc::clone(&self.fixed_tendon_rows),
                spatial_tendon_rows: Arc::clone(&self.spatial_tendon_rows),
                tendon_rows: self.tendon_rows.clone(),
                edge_fields: self.edge_fields.clone(),
                face_fields: self.face_fields.clone(),
                hessian_fields: self.hessian_fields.clone(),
                layout,
                ready: ReadyStages::default(),
                flex_validity: layout
                    .flex
                    .map(|l| {
                        let mut values = crate::runtime::host_staging::<i32>(l.guarded_validity)?;
                        values.fill(-131072);
                        values[4..l.guarded_validity - 4].fill(0);
                        self.session.upload(&values)
                    })
                    .transpose()?,
                state: self.session.upload(&state)?,
                rigid: allocate_output(&self.session, layout.rigid.guarded)?,
                com: allocate_output(&self.session, layout.com.guarded)?,
                attached: allocate_output(&self.session, layout.attached.guarded)?,
                camlight: allocate_output(&self.session, layout.camlight.guarded)?,
                fixed_tendon: allocate_output(&self.session, layout.fixed_tendon.guarded)?,
                spatial_tendon: allocate_output(&self.session, layout.spatial_tendon.guarded)?,
                spatial_wrap: allocate_integer_output(
                    &self.session,
                    layout.spatial_tendon.guarded_indices,
                )?,
                tendon: layout
                    .tendon
                    .map(|l| -> Result<_, TransferError> {
                        Ok(ResidentTendon {
                            values: allocate_output(&self.session, l.guarded)?,
                            indices: allocate_integer_output(&self.session, l.guarded_indices)?,
                        })
                    })
                    .transpose()?,
                flex: layout
                    .flex
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                faces: layout
                    .faces
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                hessian: layout
                    .hessian
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                edges: layout
                    .edges
                    .map(|l| -> Result<_, TransferError> {
                        let mut qvel = crate::runtime::host_staging::<f32>(l.guarded_qvel)?;
                        qvel.fill(-131072.0);
                        qvel[4..l.guarded_qvel - 4].fill(0.0);
                        Ok(ResidentFlexEdges {
                            qvel: self.session.upload(&qvel)?,
                            values: allocate_output(&self.session, l.guarded)?,
                        })
                    })
                    .transpose()?,
                sleep: layout
                    .sleep
                    .map(|l| -> Result<_, TransferError> {
                        let k = self.model.rigid().kinematics();
                        let initial = SleepTreeState {
                            tree_asleep: vec![crate::model::TREE_FULLY_AWAKE; l.ntree],
                            nbody_awake: k.nbody() as i32,
                            nv_awake: k.nv() as i32,
                        }
                        .packed();
                        let mut values = crate::runtime::host_staging::<i32>(l.guarded)?;
                        values.fill(-131072);
                        for row in values[4..l.guarded - 4].chunks_exact_mut(initial.len()) {
                            row.copy_from_slice(&initial);
                        }
                        Ok(ResidentSleep {
                            state: self.session.upload(&values)?,
                            scratch: allocate_integer_output(&self.session, l.guarded)?,
                        })
                    })
                    .transpose()?,
            };
            if self.equivalent {
                for world in 0..worlds {
                    let start = 4 + world * layout.rigid.output.elements_per_world();
                    data.rigid.write_range(start, &[0.0; 3])?;
                    data.rigid
                        .write_range(start + 3 * layout.rigid.nbody, &[1.0, 0.0, 0.0, 0.0])?;
                }
            }
            self.update_rigid(&mut data)?;
            // SAFETY: The initialization entry shares the checked attached ABI.
            // Metadata includes ngeom static flags, followed by four descriptors.
            // FK is synchronized; each thread writes only its guarded world.
            unsafe {
                self.initialize_kernel.launch(
                    &self.attached.metadata,
                    &self.attached.parameters,
                    &data.rigid,
                    &mut data.attached,
                    [
                        layout.rigid.nbody as u32,
                        self.model.ngeom() as u32,
                        self.model.nsite() as u32,
                        worlds as u32,
                    ],
                )?;
            }
            data.ready = ReadyStages::default();
            Ok(data)
        }
    }

    pub(super) fn check_data(&self, data: &KinematicsData) -> Result<(), TransferError> {
        if Arc::ptr_eq(&self.model, &data.model) {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn compile_velocity_position(
        &self,
    ) -> Result<VelocityPositionKernels, TransferError> {
        // Specialize only the private fixed shaders; model/state uploads stay f32.
        let mut rigid = super::rigid_source::<f64>(super::KINEMATICS_CUDA).into_owned();
        for name in [
            "model", "state", "mp", "mq", "q0", "bp", "bq", "ip", "iq", "jp", "axis", "q",
        ] {
            rigid = rigid.replace(
                &format!("const double* {name}"),
                &format!("const float* {name}"),
            );
        }
        rigid = rigid
            .replace(
                "__device__ V load_v(const double* p)",
                "template<class T> __device__ V load_v(const T* p)",
            )
            .replace(
                "__device__ Q load_q(const double* p)",
                "template<class T> __device__ Q load_q(const T* p)",
            );
        let source = format!("{FIELD_BATCH_CUDA}\n#define MJWARP_MOCAP\n{rigid}");
        let rigid = SynchronousKernel::compile(&self.session, &source, "rigid_kinematics")?;
        let com = super::rigid_source::<f64>(super::COM_POSITION_CUDA)
            .replace("const double* p,", "const float* p,")
            .replace("const double* mass=", "const float* mass=")
            .replace("const double* inertia=", "const float* inertia=")
            .replace("const double* d=inertia", "const float* d=inertia");
        let source = format!("{FIELD_BATCH_CUDA}\n{com}");
        Ok(VelocityPositionKernels {
            rigid,
            com: SynchronousKernel::compile(&self.session, &source, "rigid_com_position")?,
        })
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn update_velocity_position(
        &self,
        data: &KinematicsData,
        kernels: &VelocityPositionKernels,
        rigid: &mut TransferBuffer<f64>,
        com: &mut TransferBuffer<f64>,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        let l = data.layout;
        for (actual, expected) in [
            (rigid.len(), l.rigid.output.total_elements() + 8),
            (com.len(), l.com.output.total_elements() + 8),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field: "velocity_position_workspace",
                    expected,
                    actual,
                }
                .into());
            }
        }
        let k = self.model.rigid().kinematics();
        let worlds = data.worlds() as u32;
        // SAFETY: This typed kernel pair fixes both ABIs. Existing validated
        // metadata and f32 model/state allocations bound all reads; the checked
        // f64 workspaces have identical element strides and disjoint ownership.
        // The adapter enforces one session and waits after each launch.
        unsafe {
            kernels.rigid.launch(
                &self.rigid.metadata,
                &self.rigid.parameters,
                &data.state,
                rigid,
                [k.nq() as u32, k.nbody() as u32, k.njnt() as u32, worlds],
            )?;
            kernels.com.launch(
                &self.com.metadata,
                &self.com.parameters,
                rigid,
                com,
                [k.nbody() as u32, k.njnt() as u32, k.nv() as u32, worlds],
            )?;
        }
        Ok(())
    }

    fn check_g01_data(&self, data: &KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        Ok(())
    }

    fn require_input(
        &self,
        data: &KinematicsData,
        ready: bool,
        stage: &'static str,
    ) -> Result<(), TransferError> {
        if !self.equivalent {
            data.ready.require(ready, stage)?;
        }
        Ok(())
    }

    /// 只更新七项刚体结果。
    /// 它废弃全部派生结果。
    pub fn update_rigid(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let l = data.layout.rigid;
            let worlds = data.worlds() as u32;
            // SAFETY: Model validation and checked layouts prove every metadata,
            // parameter and qpos offset. All allocations belong to this plan's
            // session. Guarded output is disjoint and each thread owns one world.
            // The fixed f32 ABI matches rigid_kinematics; launch waits before reuse.
            unsafe {
                self.rigid_kernel.launch(
                    &self.rigid.metadata,
                    &self.rigid.parameters,
                    &data.state,
                    &mut data.rigid,
                    [l.nq as u32, l.nbody as u32, l.njnt as u32, worlds],
                )?;
            }
            data.ready.rigid = true;
            Ok(())
        }
    }

    /// 读取已有刚体设备结果。
    /// 严格计划要求刚体就绪。
    pub fn update_com(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.com = false;
        data.ready.edges = false;
        data.ready.camlight = false;
        data.ready.spatial_tendon = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let k = self.model.rigid().kinematics();
            let worlds = data.worlds() as u32;
            // SAFETY: Immutable topology proves parent and DOF ranges. Checked FK
            // and COM layouts bound every guarded stride. All f32 buffers use the
            // same session and distinct allocations; each thread owns one world.
            // This shader reads the synchronized FK buffer without a host copy.
            unsafe {
                self.com_kernel.launch(
                    &self.com.metadata,
                    &self.com.parameters,
                    &data.rigid,
                    &mut data.com,
                    [k.nbody() as u32, k.njnt() as u32, k.nv() as u32, worlds],
                )?;
            }
            data.ready.com = true;
            Ok(())
        }
    }

    /// 读取已有刚体设备结果。
    /// 跳过静态geom缓存。
    /// 每次重算全部site。
    pub fn update_attached(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.attached = false;
        data.ready.spatial_tendon = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dimensions = [
                data.layout.rigid.nbody as u32,
                self.model.ngeom() as u32,
                self.model.nsite() as u32,
                data.worlds() as u32,
            ];
            // SAFETY: Validated attachment IDs and checked packed lengths prove
            // all reads. Empty sets retain nonempty metadata/parameter/guard
            // allocations. The rigid stride is fixed in metadata; shader ABI is
            // f32. Outputs are disjoint, same-session and synchronized on return.
            unsafe {
                self.attached_kernel.launch(
                    &self.attached.metadata,
                    &self.attached.parameters,
                    &data.rigid,
                    &mut data.attached,
                    dimensions,
                )?;
            }
            data.ready.attached = true;
            Ok(())
        }
    }

    /// 复用刚体与质心设备结果。
    /// 两段执行不回读中间结果。
    pub fn update_camlight(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        self.require_input(data, data.ready.com, "com")?;
        data.ready.camlight = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.camlight {
                let dimensions = [
                    data.layout.rigid.nbody as u32,
                    self.ncam as u32,
                    self.nlight as u32,
                    data.worlds() as u32,
                ];
                // SAFETY: Checked topology, field descriptors and layouts bound
                // all reads/writes. The fixed f32 ABI matches both entry points.
                // Each thread owns one guarded world. The COM launch reads that
                // world's output only after the synchronous FK launch completes.
                // All allocations share this plan's session; no owners can drop.
                unsafe {
                    device.rigid_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.rigid,
                        &mut data.camlight,
                        dimensions,
                    )?;
                    device.com_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.com,
                        &mut data.camlight,
                        dimensions,
                    )?;
                }
            }
            data.ready.camlight = true;
            Ok(())
        }
    }

    /// 只读取常驻qpos状态。
    /// 本子集不依赖刚体或质心。
    pub fn update_fixed_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready.fixed_tendon = false;
        data.ready.sleep = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.fixed_tendon {
                let l = data.layout.fixed_tendon;
                let dimensions = [
                    data.layout.qpos.elements_per_world() as u32,
                    l.ntendon as u32,
                    l.nnz as u32,
                    data.worlds() as u32,
                ];
                // SAFETY: Validated scalar joint references and CSR coverage prove
                // every packed qpos address and Jacobian slot. Checked layouts
                // bound the wide world strides. Buffers share this session and
                // distinct allocations; each thread owns one guarded output row.
                unsafe {
                    device.kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.state,
                        &mut data.fixed_tendon,
                        dimensions,
                    )?;
                }
            }
            data.ready.fixed_tendon = true;
            Ok(())
        }
    }

    /// 复用site与质心设备结果。
    /// 三次执行不回读中间结果。
    pub fn update_spatial_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.attached, "attached")?;
        self.require_input(data, data.ready.com, "com")?;
        data.ready.spatial_tendon = false;
        data.ready.sleep = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.spatial_tendon {
                let dimensions = [
                    self.model.rigid().kinematics().nv() as u32,
                    0,
                    0,
                    data.worlds() as u32,
                ];
                // SAFETY: Validated paths, CSR and derived body descriptors bound
                // all accesses. Checked layouts bound each wide world offset.
                // The first two entries use f32; the third reads f32 and writes i32.
                // All outputs are distinct allocations from their immutable inputs.
                // Synchronous launches preserve site -> moment ordering and owners.
                unsafe {
                    device.site_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.attached,
                        &mut data.spatial_tendon,
                        dimensions,
                    )?;
                    device.moment_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.com,
                        &mut data.spatial_tendon,
                        dimensions,
                    )?;
                    device.wrap_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.spatial_tendon,
                        &mut data.spatial_wrap,
                        dimensions,
                    )?;
                }
            }
            data.ready.spatial_tendon = true;
            Ok(())
        }
    }

    /// 重算两类并合并全局结果。
    /// 空间集合需要附着与质心。
    pub fn update_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.update_fixed_tendons(data)?;
        self.update_spatial_tendons(data)?;
        self.assemble_tendons(data)
    }

    fn assemble_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready.tendon = false;
        data.ready.sleep = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(output), Some(layout)) =
                (&self.tendon, &mut data.tendon, data.layout.tendon)
                && !layout.is_empty()
            {
                let dimensions = [0, 0, 0, data.layout.qpos.worlds() as u32];
                // SAFETY: 映射覆盖全部CSR行。
                // 空间内核限定动态点数量。
                // 同一会话持有全部缓冲。
                // 布局限定各缓冲的世界偏移。
                // 输入与输出独立分配。
                // 各线程只写一个世界。
                // 浮点段采用i32索引参数。
                // 整数段读取i32空间结果。
                // 同步调用保留全部缓冲寿命。
                unsafe {
                    device.values_kernel.launch(
                        &device.model.metadata,
                        &data.spatial_wrap,
                        &data.fixed_tendon,
                        &mut output.values,
                        dimensions,
                    )?;
                    device.values_kernel.launch(
                        &device.model.metadata,
                        &data.spatial_wrap,
                        &data.spatial_tendon,
                        &mut output.values,
                        [1, 0, 0, dimensions[3]],
                    )?;
                    device.indices_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.spatial_wrap,
                        &mut output.indices,
                        dimensions,
                    )?;
                }
            }
            data.ready.tendon = true;
            Ok(())
        }
    }

    /// 只更新柔体位置子集。
    /// 它只依赖刚体结果。
    /// 先清除每世界的柔体标志。
    /// 随后计算节点与顶点位置。
    /// 严格入口先检查刚体就绪。
    /// 本辅助不计算Hessian矩阵。
    pub fn update_flex_positions(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.flex = false;
        data.ready.edges = false;
        data.ready.faces = false;
        data.ready.hessian = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out)) = (&self.flex, &mut data.flex) {
                // SAFETY: This private entry shares the checked flex metadata.
                // It ignores float inputs and writes only world*nf i32 slots.
                // Guarded validity owns the same session and checked dimensions.
                // Clear before position work, even when positions later fail.
                unsafe {
                    device.invalidate_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.rigid,
                        data.flex_validity.as_mut().expect("checked flex validity"),
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
                // SAFETY: Checked immutable fields partition nodes and vertices.
                // Each synchronized thread reads an initialized rigid world and owns
                // its guarded output; wide strides match checked batch layouts.
                unsafe {
                    device.kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.rigid,
                        out,
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
            }
            data.ready.flex = true;
            Ok(())
        }
    }

    /// 显式缓存拉伸投影矩阵。
    /// 本阶段不执行矩阵乘法。
    pub fn update_flex_hessian(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if self.hessian_fields.is_some() {
            data.ready.require(data.ready.flex, "flex_positions")?;
            data.ready.require(data.ready.edges, "flex_edges")?;
        }
        data.ready.hessian = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(output)) = (&self.hessian, &mut data.hessian) {
                // SAFETY: Validated elements, edge pairs and 21-value ranges
                // bound all accesses. Checked node offsets select vertices.
                // Buffers share one session and disjoint guarded allocations.
                // Synchronization finishes blocks before publishing valid flags.
                unsafe {
                    device.kernel.launch_flex_hessian(
                        (&device.model.metadata, &device.model.parameters),
                        data.flex.as_ref().expect("checked flex positions"),
                        &data.edges.as_ref().expect("checked flex edges").values,
                        data.flex_validity.as_ref().expect("checked flex validity"),
                        output,
                        data.layout.qpos.worlds() as u32,
                    )?;
                    device.validate_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        output,
                        data.flex_validity.as_mut().expect("checked flex validity"),
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
            }
            data.ready.hessian = true;
            Ok(())
        }
    }

    /// 唤醒后只刷新树活动标记。
    /// 它不推进自动休眠计数。
    pub fn update_tendon_wake(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if self.sleep_info.is_some() {
            data.ready.require(data.ready.tendon, "tendon")?;
        }
        data.ready.sleep = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out), Some(tendon)) =
                (&self.sleep, &mut data.sleep, &data.tendon)
            {
                let dims = [0, 0, 0, data.layout.qpos.worlds() as u32];
                // SAFETY: Checked tree cycles and global paths bound all accesses.
                // Separate guarded buffers never alias. Synchronized copy seeds
                // scratch before each world-owned wake pass; failures keep state.
                unsafe {
                    device.copy_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &out.state,
                        &mut out.scratch,
                        dims,
                    )?;
                    device.wake_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &tendon.values,
                        &mut out.scratch,
                        dims,
                    )?;
                }
                std::mem::swap(&mut out.state, &mut out.scratch);
            }
            data.ready.sleep = true;
            Ok(())
        }
    }

    /// 复用设备顶点与质心。
    /// qvel写入只废弃本阶段。
    pub fn update_flex_edges(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if data.layout.edges.is_some_and(|l| !l.is_empty()) {
            self.require_input(data, data.ready.com, "com")?;
            self.require_input(data, data.ready.flex, "flex_positions")?;
        }
        data.ready.edges = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out), Some(flex)) =
                (&self.edges, &mut data.edges, &data.flex)
            {
                // SAFETY: Checked endpoints, disjoint CSR rows, ancestry and
                // guarded wide strides bound every read/write. Inputs retain
                // this model/session. The private f32 ABI waits before reuse.
                unsafe {
                    device.kernel.launch_flex_edges(
                        &device.metadata,
                        &out.qvel,
                        &data.com,
                        flex,
                        &mut out.values,
                        data.layout.qpos.worlds() as u32,
                    )?;
                }
            }
            data.ready.edges = true;
            Ok(())
        }
    }

    /// 面阶段只读取设备节点。
    /// 它不依赖质心或qvel。
    pub fn update_flex_faces(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if data.layout.faces.is_some_and(|l| !l.is_empty()) {
            self.require_input(data, data.ready.flex, "flex_positions")?;
        }
        data.ready.faces = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(nodes), Some(out)) =
                (&self.faces, &data.flex, &mut data.faces)
            {
                // SAFETY: Canonical checked face rows bound all node references.
                // Guarded layouts bound each world. Buffers share this session.
                // The private f32 ABI synchronizes before output reuse.
                unsafe {
                    device.kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        nodes,
                        out,
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
            }
            data.ready.faces = true;
            Ok(())
        }
    }

    /// 依次执行六个基础子集。
    /// 混合入口另做GPU合并。
    /// 柔体入口另加位置更新。
    /// 可选边面紧随位置更新。
    /// 不上传模型或编译内核。
    /// 不回读阶段间的宿主结果。
    /// 不提供完整等价阶段。
    pub fn update(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.update_rigid(data)?;
        self.update_attached(data)?;
        self.update_com(data)?;
        self.update_camlight(data)?;
        self.update_flex_positions(data)?;
        self.update_flex_edges(data)?;
        self.update_flex_faces(data)?;
        self.update_tendons(data)?;
        self.update_tendon_wake(data)
    }
}

impl KinematicsData {
    pub fn flex_hessian_fields(&self) -> Option<&FlexHessianFields> {
        self.hessian_fields.as_deref()
    }
    pub fn flex_face_fields(&self) -> Option<&FlexFaceFields> {
        self.face_fields.as_deref()
    }
    pub fn flex_edge_fields(&self) -> Option<&FlexEdgeFields> {
        self.edge_fields.as_deref()
    }
    /// 只写入启用边计算的状态。
    /// 输入错误保留已有结果。
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        let l = self.edge_layout()?;
        self.write_checked_qvel(0, l.qvel.total_elements(), qvel)
    }
    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let r = self.edge_layout()?.qvel.world_elements(world)?;
        self.write_checked_qvel(r.start, r.len(), qvel)
    }
    fn edge_layout(&self) -> Result<FlexEdgeLayout, TransferError> {
        self.layout.edges.ok_or(
            InputError::InvalidDimension {
                field: "flex_edges_disabled",
            }
            .into(),
        )
    }
    fn write_checked_qvel(
        &mut self,
        offset: usize,
        expected: usize,
        qvel: &[f32],
    ) -> Result<(), TransferError> {
        if qvel.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "qvel",
                expected,
                actual: qvel.len(),
            }
            .into());
        }
        if let Some(index) = qvel.iter().position(|v| !v.is_finite()) {
            return Err(InputError::NonFinite {
                field: "qvel",
                index,
            }
            .into());
        }
        self.ready.edges = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.edges
            .as_mut()
            .expect("checked edge state")
            .qvel
            .write_range(4 + offset, qvel)
    }
    /// 检查全部状态后写入一个世界。
    /// 它只废弃树副作用结果。
    pub fn write_world_sleep(
        &mut self,
        world: usize,
        state: &SleepTreeState,
    ) -> Result<(), TransferError> {
        let l = self.layout.sleep.ok_or(InputError::DisabledOutput)?;
        let range = l.output.world_elements(world)?;
        let k = self.model.rigid().kinematics();
        state.validate(l.ntree, k.nbody(), k.nv())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = range;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.sleep
                .as_mut()
                .expect("checked sleep layout")
                .state
                .write_range(4 + range.start, &state.packed())?;
            self.ready.sleep = false;
            Ok(())
        }
    }
    pub fn tendon_rows(&self) -> Option<&TendonRows> {
        self.tendon_rows.as_deref()
    }
    pub fn spatial_tendon_rows(&self) -> &SpatialTendonRows {
        &self.spatial_tendon_rows
    }
    pub fn fixed_tendon_rows(&self) -> &FixedTendonRows {
        &self.fixed_tendon_rows
    }
    pub fn worlds(&self) -> usize {
        self.layout.qpos.worlds()
    }

    /// 完整检查后改写全部qpos。
    /// 输入错误保留已有设备结果。
    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        if self.equivalent {
            check_g01_qpos(self.model.rigid(), self.worlds(), qpos)?;
        } else {
            check_kinematics(self.model.rigid(), self.worlds(), qpos)?;
        }
        self.write_checked_qpos(0, qpos)
    }

    /// 只改写指定世界的qpos。
    /// 它废弃整组派生结果。
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qpos.world_elements(world)?;
        if self.equivalent {
            check_g01_qpos(self.model.rigid(), 1, qpos)?;
        } else {
            check_kinematics(self.model.rigid(), 1, qpos)?;
        }
        self.write_checked_qpos(range.start, qpos)
    }

    fn write_checked_qpos(&mut self, offset: usize, qpos: &[f32]) -> Result<(), TransferError> {
        self.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, qpos);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.state.write_range(offset, qpos)
    }

    pub fn nmocap(&self) -> usize {
        self.layout.mocap_pos.elements_per_world() / 3
    }

    /// 检查后写入全部mocap状态。
    /// 四元数顺序采用wxyz。
    /// GPU更新归一化四元数。
    /// 输入错误保留已有结果。
    pub fn write_mocap(&mut self, pos: &[f32], quat: &[f32]) -> Result<(), TransferError> {
        check_mocap_mode(
            self.layout.mocap_pos.total_elements(),
            self.layout.mocap_quat.total_elements(),
            pos,
            quat,
            !self.equivalent,
        )?;
        self.write_checked_mocap(0, 0, pos, quat)
    }

    /// 只写入指定世界的mocap。
    /// 它废弃整组派生结果。
    pub fn write_world_mocap(
        &mut self,
        world: usize,
        pos: &[f32],
        quat: &[f32],
    ) -> Result<(), TransferError> {
        let p = self.layout.mocap_pos.world_elements(world)?;
        let q = self.layout.mocap_quat.world_elements(world)?;
        check_mocap_mode(p.len(), q.len(), pos, quat, !self.equivalent)?;
        self.write_checked_mocap(p.start, q.start, pos, quat)
    }

    fn write_checked_mocap(
        &mut self,
        p: usize,
        q: usize,
        pos: &[f32],
        quat: &[f32],
    ) -> Result<(), TransferError> {
        self.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (p, q, pos, quat);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let start = self.layout.qpos.total_elements();
            self.state.write_range(start + p, pos)?;
            self.state
                .write_range(start + self.layout.mocap_pos.total_elements() + q, quat)
        }
    }

    /// 回读并检查全部子集结果。
    /// 失败不发布部分宿主快照。
    /// 结果未就绪时明确返回错误。
    pub fn readback(&self) -> Result<KinematicsSnapshot, TransferError> {
        self.ready.require_all()?;
        self.ready.require(
            self.layout.camlight.is_empty() || self.ready.camlight,
            "camlight",
        )?;
        self.ready.require(
            self.layout.fixed_tendon.is_empty() || self.ready.fixed_tendon,
            "fixed_tendon",
        )?;
        self.ready.require(
            self.layout.spatial_tendon.is_empty() || self.ready.spatial_tendon,
            "spatial_tendon",
        )?;
        self.ready.require(
            self.layout.tendon.is_none_or(|l| l.is_empty()) || self.ready.tendon,
            "tendon",
        )?;
        self.ready.require(
            self.layout.flex.is_none_or(|l| l.is_empty()) || self.ready.flex,
            "flex_positions",
        )?;
        self.ready.require(
            self.layout.sleep.is_none() || self.ready.sleep,
            "tendon_wake",
        )?;
        self.ready.require(
            self.layout.edges.is_none_or(|l| l.is_empty()) || self.ready.edges,
            "flex_edges",
        )?;
        self.ready.require(
            self.layout.faces.is_none_or(|l| l.is_empty()) || self.ready.faces,
            "flex_faces",
        )?;
        self.readback_fields()
    }

    /// 回读G01当前字段与缓存。
    /// 本入口不要求新鲜度票据。
    /// 未计算字段保留已有设备值。
    /// 它仍检查哨兵与有限性。
    pub fn readback_g01(&self) -> Result<KinematicsSnapshot, TransferError> {
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        self.readback_fields()
    }

    fn readback_fields(&self) -> Result<KinematicsSnapshot, TransferError> {
        let lengths = [
            self.layout.rigid.output.total_elements() + 8,
            self.layout.com.output.total_elements() + 8,
            self.layout.attached.output.total_elements() + 8,
            self.layout.camlight.output.total_elements() + 8,
            self.layout.fixed_tendon.output.total_elements() + 8,
            self.layout.spatial_tendon.output.total_elements() + 8,
        ];
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = lengths;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let rigid = read_output(&self.rigid, lengths[0], "resident_rigid_output")?;
            let com = read_output(&self.com, lengths[1], "resident_com_output")?;
            let attached = read_output(&self.attached, lengths[2], "resident_attached_output")?;
            let camlight = read_output(&self.camlight, lengths[3], "resident_camlight_output")?;
            let fixed_tendon = read_output(
                &self.fixed_tendon,
                lengths[4],
                "resident_fixed_tendon_output",
            )?;
            let spatial_tendon = read_output(
                &self.spatial_tendon,
                lengths[5],
                "resident_spatial_tendon_output",
            )?;
            let spatial_wrap = read_output(
                &self.spatial_wrap,
                self.layout.spatial_tendon.indices.total_elements() + 8,
                "resident_spatial_wrap_output",
            )?;
            let tendon = match (&self.tendon, self.layout.tendon, &self.tendon_rows) {
                (Some(out), Some(layout), Some(rows)) => Some(TendonOutput {
                    layout,
                    values: read_output(&out.values, layout.guarded, "resident_tendon_output")?,
                    indices: read_output(
                        &out.indices,
                        layout.guarded_indices,
                        "resident_tendon_indices",
                    )?,
                    rows: Arc::clone(rows),
                }),
                _ => None,
            };
            Ok(KinematicsSnapshot {
                hessian: match (&self.hessian, self.layout.hessian) {
                    (Some(out), Some(layout)) if self.ready.hessian => Some(FlexHessianOutput {
                        layout,
                        fields: Arc::clone(
                            self.hessian_fields
                                .as_ref()
                                .expect("checked hessian fields"),
                        ),
                        edges: Arc::clone(self.edge_fields.as_ref().expect("checked edge fields")),
                        values: read_output(out, layout.guarded, "resident_flex_hessian_output")?,
                    }),
                    _ => None,
                },
                faces: match (&self.faces, self.layout.faces) {
                    (Some(out), Some(layout)) => Some(FlexFaceOutput {
                        layout,
                        fields: Arc::clone(self.face_fields.as_ref().expect("checked face fields")),
                        values: read_output(out, layout.guarded, "resident_flex_face_output")?,
                    }),
                    _ => None,
                },
                edges: match (&self.edges, self.layout.edges) {
                    (Some(out), Some(layout)) => {
                        read_output(&out.qvel, layout.guarded_qvel, "resident_qvel_input")?;
                        Some(FlexEdgeOutput {
                            layout,
                            fields: Arc::clone(
                                self.edge_fields.as_ref().expect("checked edge fields"),
                            ),
                            values: read_output(
                                &out.values,
                                layout.guarded,
                                "resident_flex_edge_output",
                            )?,
                        })
                    }
                    _ => None,
                },
                sleep: match (&self.sleep, self.layout.sleep) {
                    (Some(out), Some(layout)) => Some(SleepTreeOutput {
                        layout,
                        values: read_output(
                            &out.state,
                            layout.guarded,
                            "resident_sleep_tree_output",
                        )?,
                    }),
                    _ => None,
                },
                flex: match (&self.flex, self.layout.flex) {
                    (Some(out), Some(layout)) => Some(FlexPositionOutput {
                        layout,
                        values: read_output(out, layout.guarded, "resident_flex_position_output")?,
                        validity: read_flex_validity(
                            self.flex_validity.as_ref().expect("checked flex validity"),
                            layout.guarded_validity,
                        )?,
                    }),
                    _ => None,
                },
                attached: AttachedKinematicsOutput {
                    rigid: KinematicsOutput {
                        layout: self.layout.rigid,
                        values: rigid,
                    },
                    layout: self.layout.attached,
                    values: attached,
                },
                com: ComPositionOutput {
                    layout: self.layout.com,
                    values: com,
                },
                camlight: CamLightOutput {
                    layout: self.layout.camlight,
                    values: camlight,
                },
                fixed_tendon: FixedTendonOutput {
                    layout: self.layout.fixed_tendon,
                    values: fixed_tendon,
                    rows: Arc::clone(&self.fixed_tendon_rows),
                },
                spatial_tendon: SpatialTendonOutput {
                    layout: self.layout.spatial_tendon,
                    values: spatial_tendon,
                    indices: spatial_wrap,
                    rows: Arc::clone(&self.spatial_tendon_rows),
                },
                tendon,
            })
        }
    }
}

#[cfg(test)]
fn check_mocap(np: usize, nq: usize, pos: &[f32], quat: &[f32]) -> Result<(), InputError> {
    check_mocap_mode(np, nq, pos, quat, true)
}

fn check_size(field: &'static str, expected: usize, actual: usize) -> Result<(), InputError> {
    if actual == expected {
        Ok(())
    } else {
        Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        })
    }
}
fn check_finite_field(
    field: &'static str,
    expected: usize,
    values: &[f32],
) -> Result<(), InputError> {
    check_size(field, expected, values.len())?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite { field, index });
        }
    }
    Ok(())
}
fn check_g01_qpos(
    model: &crate::model::InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsLayout, InputError> {
    let k = model.kinematics();
    let l = KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?;
    let expected = worlds.checked_mul(k.nq()).ok_or(InputError::Overflow {
        field: "kinematics_qpos",
    })?;
    check_finite_field("kinematics_qpos", expected, qpos)?;
    Ok(l)
}
fn check_mocap_mode(
    np: usize,
    nq: usize,
    pos: &[f32],
    quat: &[f32],
    strict: bool,
) -> Result<(), InputError> {
    for (field, expected, values) in [("mocap_pos", np, pos), ("mocap_quat", nq, quat)] {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual: values.len(),
            });
        }
        for (index, v) in values.iter().enumerate() {
            if !v.is_finite() {
                return Err(InputError::NonFinite { field, index });
            }
        }
    }
    for (index, rotation) in quat.as_chunks::<4>().0.iter().enumerate() {
        let squared: f64 = rotation.iter().map(|&x| f64::from(x).powi(2)).sum();
        if strict && !(1e-12..=1e12).contains(&squared) {
            return Err(InputError::InvalidTopology {
                field: "mocap_quat",
                index: 4 * index,
                reason: "unusable_state_quaternion",
            });
        }
    }
    Ok(())
}

fn check_parameters(
    model: &AttachedModelInput,
    parameters: &KinematicsParameters,
) -> Result<(), InputError> {
    use KinematicsParameter as F;
    parameters.validate(model)?;
    let k = model.rigid().kinematics();
    for field in F::ALL {
        let width = field.shared(model).len();
        if width == 0 {
            continue;
        }
        for (row, values) in parameters
            .values(field, model)
            .chunks_exact(width)
            .enumerate()
        {
            let fail = |index, reason| InputError::InvalidTopology {
                field: field.name(),
                index,
                reason,
            };
            let canonical: Option<&[f32]> = match field {
                F::BodyPos | F::BodyIpos => Some(&[0.0; 3]),
                F::BodyQuat | F::BodyIquat => Some(&[1.0, 0.0, 0.0, 0.0]),
                F::BodyMass => Some(&[0.0]),
                F::BodyInertia => Some(&[0.0; 3]),
                _ => None,
            };
            if let Some(required) = canonical
                && &values[..required.len()] != required
            {
                let reason = if matches!(field, F::BodyMass | F::BodyInertia) {
                    "nonzero_world_inertia"
                } else {
                    "noncanonical_world"
                };
                return Err(fail(row * width, reason));
            }
            match field {
                F::BodyQuat | F::BodyIquat | F::GeomQuat | F::SiteQuat => {
                    for (index, q) in values.as_chunks::<4>().0.iter().enumerate() {
                        if (super::norm_squared(q) - 1.0).abs() > 2e-6 {
                            return Err(fail(row * width / 4 + index, "nonunit_model_rotation"));
                        }
                    }
                }
                F::BodyMass | F::BodyInertia => {
                    for (index, &v) in values.iter().enumerate() {
                        if v < 0.0 {
                            return Err(InputError::NegativeValue {
                                field: field.name(),
                                index: row * width + index,
                            });
                        }
                    }
                }
                F::JointAxis => {
                    for (j, axis) in values.as_chunks::<3>().0.iter().enumerate() {
                        if k.fields().jnt_type[j] >= 2
                            && (super::norm_squared(axis) - 1.0).abs() > 2e-6
                        {
                            return Err(fail(row * k.njnt() + j, "nonunit_joint_axis"));
                        }
                    }
                }
                F::Qpos0 => {
                    for j in 0..k.njnt() {
                        let ty = k.fields().jnt_type[j];
                        if ty <= 1 {
                            let start =
                                k.fields().jnt_qposadr[j] as usize + if ty == 0 { 3 } else { 0 };
                            if !(1e-12..=1e12)
                                .contains(&super::norm_squared(&values[start..start + 4]))
                            {
                                return Err(fail(row * width + start, "unusable_state_quaternion"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
const RIGID_PARAMETERS: [KinematicsParameter; 7] = [
    KinematicsParameter::Qpos0,
    KinematicsParameter::BodyPos,
    KinematicsParameter::BodyQuat,
    KinematicsParameter::BodyIpos,
    KinematicsParameter::BodyIquat,
    KinematicsParameter::JointPos,
    KinematicsParameter::JointAxis,
];
#[cfg(feature = "cuda-probe")]
const COM_PARAMETERS: [KinematicsParameter; 2] = [
    KinematicsParameter::BodyMass,
    KinematicsParameter::BodyInertia,
];
#[cfg(feature = "cuda-probe")]
const ATTACHED_PARAMETERS: [KinematicsParameter; 4] = [
    KinematicsParameter::GeomPos,
    KinematicsParameter::GeomQuat,
    KinematicsParameter::SitePos,
    KinematicsParameter::SiteQuat,
];

#[cfg(feature = "cuda-probe")]
struct PackedModel {
    metadata: Vec<i32>,
    parameters: Vec<f32>,
}

#[cfg(feature = "cuda-probe")]
fn pack_flex_hessian(
    positions: &FlexPositionFields,
    edges: &FlexEdgeFields,
    fields: &FlexHessianFields,
    position_stride: usize,
    edge_stride: usize,
) -> Result<PackedModel, TransferError> {
    let nf = positions.flex_interp.len();
    let elem_offset = 9 + 11 * nf;
    let elemedge_offset = elem_offset + fields.flex_elem.len();
    let edge_offset = elemedge_offset + fields.flex_elemedge.len();
    let mut metadata = crate::runtime::host_staging::<i32>(edge_offset + edges.flex_edge.len())?;
    metadata[..9].copy_from_slice(&[
        nf as i32,
        positions.flex_vertbodyid.len() as i32,
        edges.nflexedge() as i32,
        position_stride as i32,
        edge_stride as i32,
        elem_offset as i32,
        elemedge_offset as i32,
        edge_offset as i32,
        (3 * positions.flex_nodebodyid.len()) as i32,
    ]);
    for f in 0..nf {
        metadata[9 + 11 * f..20 + 11 * f].copy_from_slice(&[
            fields.flex_dim[f],
            i32::from(fields.active(positions, f)),
            positions.flex_vertadr[f],
            positions.flex_vertnum[f],
            edges.flex_edgeadr[f],
            edges.flex_edgenum[f],
            fields.flex_elemadr[f],
            fields.flex_elemnum[f],
            fields.flex_elemdataadr[f],
            fields.flex_elemedgeadr[f],
            fields.flex_stiffnessadr[f],
        ]);
    }
    metadata[elem_offset..elemedge_offset].copy_from_slice(&fields.flex_elem);
    metadata[elemedge_offset..edge_offset].copy_from_slice(&fields.flex_elemedge);
    metadata[edge_offset..].copy_from_slice(&edges.flex_edge);
    let mut parameters = crate::runtime::host_staging::<f32>(
        (fields.flexedge_length0.len() + fields.flex_stiffness.len()).max(1),
    )?;
    let n = fields.flexedge_length0.len();
    parameters[..n].copy_from_slice(&fields.flexedge_length0);
    parameters[n..n + fields.flex_stiffness.len()].copy_from_slice(&fields.flex_stiffness);
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_flex_faces(
    positions: &FlexPositionFields,
    faces: &FlexFaceFields,
    stride: usize,
) -> Result<PackedModel, TransferError> {
    let length = packed_length([2, faces.nflexface() * 10].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    metadata[..2].copy_from_slice(&[faces.nflexface() as i32, stride as i32]);
    for face in 0..faces.nflexface() {
        let flex = faces.flex_face_map[2 * face] as usize;
        let local = faces.flex_face_map[2 * face + 1] as usize;
        let axis =
            crate::model::flex::face_nodes(&positions.flex_cellnum[3 * flex..3 * flex + 3], local)
                .0;
        metadata[2 + 10 * face] = axis;
        metadata[3 + 10 * face..12 + 10 * face]
            .copy_from_slice(&faces.flex_face[9 * face..9 * face + 9]);
    }
    Ok(PackedModel {
        metadata,
        parameters: vec![0.0],
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_sleep(info: &TendonWakeInfo, rows: &TendonRows) -> Result<PackedModel, TransferError> {
    let f = &info.fields;
    let l = TendonLayout::new(1, rows.ntendon(), rows.nnz(), rows.nwrap())?;
    let mut metadata = vec![
        info.ntree as i32,
        rows.ntendon() as i32,
        info.wrap_treeid.len() as i32,
        f.tendon_range.batches() as i32,
        f.tendon_margin.batches() as i32,
        l.output.elements_per_world() as i32,
        i32::from(f.sleep_enabled && !f.island_disabled),
    ];
    metadata.extend(&info.tendon_adr);
    metadata.extend(&info.tendon_num);
    metadata.extend(f.tendon_limited.iter().copied().map(i32::from));
    metadata.extend(&info.wrap_treeid);
    let mut parameters = f.tendon_range.values().to_vec();
    parameters.extend(f.tendon_margin.values());
    if parameters.is_empty() {
        parameters.push(0.0);
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_flex_edges(
    model: &AttachedModelInput,
    f: &FlexPositionFields,
    e: &FlexEdgeFields,
    com_stride: usize,
    flex_stride: usize,
) -> Result<Vec<i32>, TransferError> {
    let rigid = model.rigid();
    let k = rigid.kinematics();
    let nb = k.nbody();
    let ne = e.nflexedge();
    let nnz = e.nnz();
    let nvert = f.flex_vertbodyid.len();
    let length = packed_length([7, 4 * ne, nnz, nvert, nb, nb, k.nv()].into_iter())?;
    let mut m = crate::runtime::host_staging::<i32>(length)?;
    m[..7].copy_from_slice(&[
        nb as i32,
        k.nv() as i32,
        ne as i32,
        nnz as i32,
        com_stride as i32,
        flex_stride as i32,
        nvert as i32,
    ]);
    for flex in 0..f.flex_interp.len() {
        let start = e.flex_edgeadr[flex] as usize;
        let end = start + e.flex_edgenum[flex] as usize;
        for edge in start..end {
            m[7 + 4 * edge..7 + 4 * edge + 4].copy_from_slice(&[
                f.flex_vertadr[flex] + e.flex_edge[2 * edge],
                f.flex_vertadr[flex] + e.flex_edge[2 * edge + 1],
                e.flexedge_j_rowadr[edge],
                e.flexedge_j_rownnz[edge],
            ]);
        }
    }
    let mut cursor = 7 + 4 * ne;
    for values in [
        &e.flexedge_j_colind,
        &f.flex_vertbodyid,
        &k.fields().body_parentid,
    ] {
        m[cursor..cursor + values.len()].copy_from_slice(values);
        cursor += values.len();
    }
    for body in 1..nb {
        let parent = k.fields().body_parentid[body] as usize;
        m[cursor + body] = if parent == 0 {
            body as i32
        } else {
            m[cursor + parent]
        };
    }
    cursor += nb;
    m[cursor..].copy_from_slice(&rigid.fields().dof_bodyid);
    Ok(m)
}

#[cfg(feature = "cuda-probe")]
fn pack_flex(
    f: &FlexPositionFields,
    rigid_stride: usize,
    nb: usize,
) -> Result<PackedModel, TransferError> {
    let (nf, nn, nv) = (
        f.flex_interp.len(),
        f.flex_nodebodyid.len(),
        f.flex_vertbodyid.len(),
    );
    let mut metadata = crate::runtime::host_staging::<i32>(5 + 9 * nf + nn + nv)?;
    metadata[..5].copy_from_slice(&[
        nf as i32,
        nn as i32,
        nv as i32,
        nb as i32,
        rigid_stride as i32,
    ]);
    for i in 0..nf {
        metadata[5 + 9 * i..5 + 9 * i + 9].copy_from_slice(&[
            f.flex_interp[i],
            f.flex_cellnum[3 * i],
            f.flex_cellnum[3 * i + 1],
            f.flex_cellnum[3 * i + 2],
            f.flex_nodeadr[i],
            f.flex_nodenum[i],
            f.flex_vertadr[i],
            f.flex_vertnum[i],
            i32::from(f.flex_centered[i]),
        ]);
    }
    metadata[5 + 9 * nf..5 + 9 * nf + nn].copy_from_slice(&f.flex_nodebodyid);
    metadata[5 + 9 * nf + nn..].copy_from_slice(&f.flex_vertbodyid);
    let length = (6 * nv + 3 * nn).max(1);
    let mut parameters = crate::runtime::host_staging::<f32>(length)?;
    parameters[..3 * nv].copy_from_slice(&f.flex_vert);
    parameters[3 * nv..6 * nv].copy_from_slice(&f.flex_vert0);
    parameters[6 * nv..6 * nv + 3 * nn].copy_from_slice(&f.flex_node);
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_tendon(
    rows: &TendonRows,
    fixed: &FixedTendonRows,
    spatial: &SpatialTendonRows,
) -> Result<PackedModel, TransferError> {
    let length = packed_length([8, rows.ntendon() * 5].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    metadata[..8].copy_from_slice(&[
        rows.ntendon() as i32,
        rows.nnz() as i32,
        rows.nwrap() as i32,
        fixed.ntendon() as i32,
        fixed.nnz() as i32,
        spatial.ntendon() as i32,
        spatial.nnz() as i32,
        spatial.nwrap() as i32,
    ]);
    for t in 0..rows.ntendon() {
        let (kind, local, offset) = match rows.subset(t)? {
            TendonSubset::Fixed(local) => (0, local, fixed.rowadr()[local]),
            TendonSubset::Spatial(local) => (1, local, spatial.rowadr()[local]),
        };
        metadata[8 + 5 * t..13 + 5 * t].copy_from_slice(&[
            kind,
            local as i32,
            offset,
            rows.rowadr()[t],
            rows.rownnz()[t],
        ]);
    }
    Ok(PackedModel {
        metadata,
        parameters: vec![0.0],
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_camlight(
    fields: &CamLightFields,
    parameters: &CamLightParameters,
    rigid_stride: usize,
    com_stride: usize,
) -> Result<PackedModel, TransferError> {
    let header = [rigid_stride as i32, com_stride as i32];
    let topology: [&[i32]; 7] = [
        &header,
        &fields.cam_mode,
        &fields.cam_bodyid,
        &fields.cam_targetbodyid,
        &fields.light_mode,
        &fields.light_bodyid,
        &fields.light_targetbodyid,
    ];
    let metadata_len = packed_length(topology.iter().map(|t| t.len()).chain(std::iter::once(20)))?;
    let parameters_len = packed_length(
        CamLightParameter::ALL
            .iter()
            .map(|&f| parameters.values(f, fields).len()),
    )?;
    let mut metadata = crate::runtime::host_staging::<i32>(metadata_len)?;
    let mut values = crate::runtime::host_staging::<f32>(parameters_len.max(1))?;
    let mut cursor = 0;
    for t in topology {
        metadata[cursor..cursor + t.len()].copy_from_slice(t);
        cursor += t.len();
    }
    let mut offset = 0;
    for f in CamLightParameter::ALL {
        let v = parameters.values(f, fields);
        metadata[cursor] = offset as i32;
        metadata[cursor + 1] = parameters.batches(f) as i32;
        values[offset..offset + v.len()].copy_from_slice(v);
        offset += v.len();
        cursor += 2;
    }
    Ok(PackedModel {
        metadata,
        parameters: values,
    })
}

#[cfg(feature = "cuda-probe")]
impl PackedModel {
    fn upload(self, session: &TransferSession) -> Result<super::DeviceModel<f32>, TransferError> {
        Ok(super::DeviceModel {
            metadata: session.upload(&self.metadata)?,
            parameters: session.upload(&self.parameters)?,
        })
    }
}

#[cfg(feature = "cuda-probe")]
fn pack_fixed_tendon(
    model: &AttachedModelInput,
    fields: &FixedTendonFields,
) -> Result<PackedModel, TransferError> {
    let nt = fields.tendon_adr.len();
    let nw = fields.wrap_objid.len();
    let length = packed_length([1, nt, nt, nw, nw].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    let mut parameters = crate::runtime::host_staging::<f32>(nw.max(1))?;
    metadata[0] = nw as i32;
    metadata[1..1 + nt].copy_from_slice(&fields.tendon_adr);
    metadata[1 + nt..1 + 2 * nt].copy_from_slice(&fields.tendon_num);
    parameters[..nw].copy_from_slice(&fields.wrap_prm);
    let k = model.rigid().kinematics().fields();
    for tendon in 0..nt {
        let row = fields.ten_j_rowadr[tendon] as usize;
        let columns = &fields.ten_j_colind[row..row + fields.ten_j_rownnz[tendon] as usize];
        let start = fields.tendon_adr[tendon] as usize;
        for i in start..start + fields.tendon_num[tendon] as usize {
            let joint = fields.wrap_objid[i] as usize;
            metadata[1 + 2 * nt + i] = k.jnt_qposadr[joint];
            // Model validation proved each reference exists in this sorted row.
            let slot = columns.binary_search(&k.jnt_dofadr[joint]).map_err(|_| {
                InputError::InvalidTopology {
                    field: "ten_J_colind",
                    index: row,
                    reason: "missing_tendon_dof_column",
                }
            })?;
            metadata[1 + 2 * nt + nw + i] = (row + slot) as i32;
        }
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
fn packed_length(mut lengths: impl Iterator<Item = usize>) -> Result<usize, InputError> {
    lengths.try_fold(0usize, |sum, len| {
        sum.checked_add(len)
            .filter(|&end| end <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "resident_parameter_pack",
            })
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_spatial_tendon(
    model: &AttachedModelInput,
    fields: &SpatialTendonFields,
    geometry: Option<&SpatialTendonGeometry>,
) -> Result<PackedModel, TransferError> {
    let k = model.rigid().kinematics();
    let f = k.fields();
    let nb = k.nbody();
    let nt = fields.tendon_adr.len();
    let nw = fields.wrap_type.len();
    let length = packed_length(
        [
            9,
            nb,
            nb,
            nb,
            nb,
            nt,
            nt,
            nt,
            nt,
            nw,
            nw,
            nw,
            nw,
            fields.ten_j_colind.len(),
        ]
        .into_iter(),
    )?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    let size_count = geometry.map_or(0, |g| g.geom_size.values().len());
    let param_count = packed_length([nw, size_count].into_iter())?;
    let mut parameters = crate::runtime::host_staging::<f32>(param_count.max(1))?;
    let layout = AttachedLayout::new(1, model.ngeom(), model.nsite())?;
    metadata[..9].copy_from_slice(&[
        nt as i32,
        nw as i32,
        fields.ten_j_colind.len() as i32,
        nb as i32,
        layout.output.elements_per_world() as i32,
        (12 * model.ngeom()) as i32,
        model.ngeom() as i32,
        geometry.map_or(1, |g| g.geom_size.batches()) as i32,
        nw as i32,
    ]);
    metadata[9..9 + nb].copy_from_slice(&f.body_parentid);
    for body in 1..nb {
        let parent = f.body_parentid[body] as usize;
        metadata[9 + nb + body] = if parent == 0 {
            body as i32
        } else {
            metadata[9 + nb + parent]
        };
        let count = f.body_jntnum[body] as usize;
        if count > 0 {
            let start = f.body_jntadr[body] as usize;
            metadata[9 + 2 * nb + body] = f.jnt_dofadr[start];
            for joint in start..start + count {
                metadata[9 + 3 * nb + body] +=
                    crate::model::topology::JointType::try_from(f.jnt_type[joint])?.dof_width()
                        as i32;
            }
        }
    }
    let mut cursor = 9 + 4 * nb;
    for values in [
        &fields.tendon_adr,
        &fields.tendon_num,
        &fields.ten_j_rowadr,
        &fields.ten_j_rownnz,
        &fields.wrap_type,
        &fields.wrap_objid,
    ] {
        metadata[cursor..cursor + values.len()].copy_from_slice(values);
        cursor += values.len();
    }
    for (i, (&kind, &id)) in fields.wrap_type.iter().zip(&fields.wrap_objid).enumerate() {
        metadata[cursor + i] = match kind {
            3 => model.fields().site_bodyid[id as usize],
            4 | 5 => model.fields().geom_bodyid[id as usize],
            _ => -1,
        };
    }
    cursor += nw;
    for (i, &kind) in fields.wrap_type.iter().enumerate() {
        metadata[cursor + i] = if matches!(kind, 4 | 5) && fields.wrap_prm[i].round() >= 0.0 {
            fields.wrap_prm[i].round() as i32
        } else {
            -1
        };
    }
    cursor += nw;
    metadata[cursor..].copy_from_slice(&fields.ten_j_colind);
    for tendon in 0..nt {
        let mut scale = 1.0;
        let start = fields.tendon_adr[tendon] as usize;
        for (i, value) in parameters
            .iter_mut()
            .enumerate()
            .skip(start)
            .take(fields.tendon_num[tendon] as usize)
        {
            if fields.wrap_type[i] == 2 {
                scale = 1.0 / fields.wrap_prm[i];
            }
            *value = scale;
        }
    }
    if let Some(g) = geometry {
        parameters[nw..nw + size_count].copy_from_slice(g.geom_size.values());
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
fn pack_model(
    model: &AttachedModelInput,
    parameters: &KinematicsParameters,
    topology: &[&[i32]],
    fields: &[KinematicsParameter],
) -> Result<PackedModel, TransferError> {
    let metadata_len = packed_length(
        topology
            .iter()
            .map(|t| t.len())
            .chain(std::iter::once(2 * fields.len())),
    )?;
    let parameters_len = packed_length(fields.iter().map(|&f| parameters.values(f, model).len()))?;
    let mut metadata = crate::runtime::host_staging::<i32>(metadata_len)?;
    let mut values = crate::runtime::host_staging::<f32>(parameters_len.max(1))?;
    let mut cursor = 0;
    for t in topology {
        metadata[cursor..cursor + t.len()].copy_from_slice(t);
        cursor += t.len();
    }
    let mut offset = 0;
    for &f in fields {
        let v = parameters.values(f, model);
        metadata[cursor] = offset as i32;
        metadata[cursor + 1] = parameters.batches(f) as i32;
        values[offset..offset + v.len()].copy_from_slice(v);
        offset += v.len();
        cursor += 2;
    }
    Ok(PackedModel {
        metadata,
        parameters: values,
    })
}

// Descriptors hold a checked scalar offset and a positive field period.
// Wide pointer arithmetic also protects offsets across CUDA blocks.
#[cfg(feature = "cuda-probe")]
const FIELD_BATCH_CUDA: &str = r#"
#define MJWARP_FIELD_BATCHES
__device__ const float* field_parameter(const float* p,const int* desc,unsigned world,unsigned width) {
  return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;
}
"#;

#[cfg(feature = "cuda-probe")]
fn allocate_output(
    session: &TransferSession,
    length: usize,
) -> Result<TransferBuffer<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(length)?;
    values.fill(-131072.0);
    values[4..length - 4].fill(f32::NAN);
    session.upload(&values)
}

#[cfg(feature = "cuda-probe")]
fn read_flex_validity(
    buffer: &TransferBuffer<i32>,
    length: usize,
) -> Result<Vec<bool>, TransferError> {
    let values = read_output(buffer, length, "resident_flex_hessian_valid")?;
    decode_flex_validity(&values[4..length - 4])
}

#[cfg(any(feature = "cuda-probe", test))]
fn decode_flex_validity(values: &[i32]) -> Result<Vec<bool>, TransferError> {
    for (index, &value) in values.iter().enumerate() {
        if value != 0 && value != 1 {
            return Err(InputError::InvalidTopology {
                field: "resident_flex_hessian_valid",
                index,
                reason: "expected 0 or 1",
            }
            .into());
        }
    }
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact(values.len())
        .map_err(|_| TransferError::HostAllocation {
            bytes: values.len(),
        })?;
    decoded.extend(values.iter().map(|&v| v == 1));
    Ok(decoded)
}

#[cfg(feature = "cuda-probe")]
fn allocate_integer_output(
    session: &TransferSession,
    length: usize,
) -> Result<TransferBuffer<i32>, TransferError> {
    let mut values = crate::runtime::host_staging::<i32>(length)?;
    values.fill(-131072);
    values[4..length - 4].fill(i32::MAX);
    session.upload(&values)
}

#[cfg(feature = "cuda-probe")]
fn read_output<T: crate::runtime::TransferElement + Into<f64>>(
    buffer: &TransferBuffer<T>,
    length: usize,
    field: &'static str,
) -> Result<Vec<T>, TransferError> {
    let mut values = crate::runtime::host_staging::<T>(length)?;
    buffer.read_range_into(0, &mut values)?;
    super::check_device_values(&values, field)?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
    };

    #[test]
    fn flex_hessian_decodes_only_canonical_boolean_slots() {
        assert_eq!(decode_flex_validity(&[]).unwrap(), Vec::<bool>::new());
        assert_eq!(
            decode_flex_validity(&[0, 1, 0, 1]).unwrap(),
            [false, true, false, true]
        );
        for value in [-131072, -1, 2, i32::MIN, i32::MAX] {
            assert!(matches!(
                decode_flex_validity(&[0, value, 1]),
                Err(TransferError::Input(InputError::InvalidTopology {
                    field: "resident_flex_hessian_valid",
                    index: 1,
                    ..
                }))
            ));
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn flex_hessian_plan(session: &TransferSession) -> KinematicsPlan {
        let cam = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let mut nodes = Vec::new();
        for _ in 0..2 {
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        nodes.extend([x as f32, y as f32, z as f32]);
                    }
                }
            }
        }
        let mut faces = FlexFaceFields::default();
        for f in 0..6 {
            faces.flex_face_map.extend([2, f]);
            faces.flex_face.extend(
                crate::model::flex::face_nodes(&[1; 3], f as usize)
                    .1
                    .map(|n| n + 8),
            );
            faces.flex_face.extend([-1; 5]);
        }
        let flex = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0, 1, -1],
                flex_cellnum: vec![1; 9],
                flex_nodeadr: vec![0, 0, 8],
                flex_nodenum: vec![0, 8, 8],
                flex_vertadr: vec![0, 2, 4],
                flex_vertnum: vec![2; 3],
                flex_centered: vec![false; 3],
                flex_nodebodyid: vec![1; 16],
                flex_vertbodyid: vec![1; 6],
                flex_node: nodes,
                flex_vert: [0.2, 0.3, 0.4, 0.7, 0.8, 0.9].repeat(3),
                flex_vert0: [0.2, 0.3, 0.4, 0.7, 0.8, 0.9].repeat(3),
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0, 1, 2],
            flex_edgenum: vec![1; 3],
            flex_edge: [0, 1].repeat(3),
            flexedge_j_rowadr: vec![0; 3],
            flexedge_j_rownnz: vec![0; 3],
            flexedge_j_colind: vec![],
        })
        .unwrap()
        .with_faces(faces)
        .unwrap()
        .with_hessian(FlexHessianFields {
            flex_dim: vec![1; 3],
            flex_rigid: vec![true, false, false],
            flex_elemadr: vec![0, 1, 2],
            flex_elemnum: vec![1; 3],
            flex_elemdataadr: vec![0, 2, 4],
            flex_elemedgeadr: vec![0, 1, 2],
            flex_stiffnessadr: vec![-1; 3],
            flex_elem: [0, 1].repeat(3),
            flex_elemedge: vec![0; 3],
            flexedge_length0: vec![1.0; 3],
            flex_stiffness: vec![],
        })
        .unwrap();
        let wake = TendonWakeModelInput::with_flex_positions(
            flex,
            crate::model::TendonWakeFields {
                body_treeid: vec![-1, 0],
                tendon_limited: vec![],
                tendon_range: crate::model::ParameterBatch::new(3, 0, vec![]).unwrap(),
                tendon_margin: crate::model::ParameterBatch::new(2, 0, vec![]).unwrap(),
                sleep_enabled: true,
                island_disabled: false,
            },
        )
        .unwrap();
        KinematicsPlan::with_tendon_wake(
            session,
            wake,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }

    #[cfg(feature = "cuda-probe")]
    fn stretch_plan(session: &TransferSession) -> KinematicsPlan {
        let cam = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let mut stiffness = vec![0.0; 42];
        for base in [0, 21] {
            for k in [0, 3, 5] {
                stiffness[base + k] = 1.0;
            }
        }
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0; 2],
                flex_cellnum: vec![1; 6],
                flex_nodeadr: vec![0; 2],
                flex_nodenum: vec![0; 2],
                flex_vertadr: vec![0, 3],
                flex_vertnum: vec![3; 2],
                flex_centered: vec![false; 2],
                flex_nodebodyid: vec![],
                flex_node: vec![],
                flex_vertbodyid: [0, 1, 1].repeat(2),
                flex_vert: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2),
                flex_vert0: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2),
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0, 3],
            flex_edgenum: vec![3; 2],
            flex_edge: [1, 2, 2, 0, 0, 1].repeat(2),
            flexedge_j_rowadr: vec![0; 6],
            flexedge_j_rownnz: vec![0; 6],
            flexedge_j_colind: vec![],
        })
        .unwrap()
        .with_hessian(FlexHessianFields {
            flex_dim: vec![2; 2],
            flex_rigid: vec![false; 2],
            flex_elemadr: vec![0, 1],
            flex_elemnum: vec![1; 2],
            flex_elemdataadr: vec![0, 3],
            flex_elemedgeadr: vec![0, 3],
            flex_stiffnessadr: vec![0, 21],
            flex_elem: [0, 1, 2].repeat(2),
            flex_elemedge: [0, 1, 2].repeat(2),
            flexedge_length0: [2.0f32.sqrt(), 1.0, 1.0].repeat(2),
            flex_stiffness: stiffness,
        })
        .unwrap();
        KinematicsPlan::with_flex_positions(
            session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn stretch_hessian_checks_analytic_compression_and_partial_cache() {
        let session = TransferSession::new(0).unwrap();
        let plan = stretch_plan(&session);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let vertices = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2);
        let lengths = [2.0f32.sqrt(), 1.0, 1.0].repeat(2);
        let expected_diag = [
            2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 4.0, -2.0, 0.0, 2.0, 0.0, 0.0, 2.0, -2.0, 0.0, 4.0, 0.0,
            0.0,
        ];
        let expected_edge = [
            -2.0, 2.0, 0.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0, 0.0, 0.0, 0.0,
            0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        for scale in [1.0, 0.5] {
            for w in 0..513 {
                data.flex
                    .as_mut()
                    .unwrap()
                    .write_range(
                        4 + 18 * w,
                        &vertices.iter().map(|v| v * scale).collect::<Vec<_>>(),
                    )
                    .unwrap();
                data.edges
                    .as_mut()
                    .unwrap()
                    .values
                    .write_range(
                        4 + 12 * w,
                        &lengths.iter().map(|v| v * scale).collect::<Vec<_>>(),
                    )
                    .unwrap();
            }
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(4, &vec![0; 1026])
                .unwrap();
            plan.update_flex_hessian(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in [0, 256, 512] {
                let h = out.flex_hessian().unwrap().world(w).unwrap();
                for (actual, expected) in h.flexvert_hessian.iter().zip(expected_diag.repeat(2)) {
                    assert!((actual - expected * scale * scale).abs() < 2e-6);
                }
                for (actual, expected) in h.flexedge_hessian.iter().zip(expected_edge.repeat(2)) {
                    assert!((actual - expected * scale * scale).abs() < 2e-6);
                }
            }
        }
        let width = data.layout.hessian.unwrap().output.elements_per_world();
        let payload = vec![37.0; 513 * width];
        data.hessian
            .as_mut()
            .unwrap()
            .write_range(4, &payload)
            .unwrap();
        let flags: Vec<i32> = (0..1026).map(|i| i % 2).collect();
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &flags)
            .unwrap();
        plan.update_flex_hessian(&mut data).unwrap();
        let out = data.readback().unwrap();
        for w in 0..513 {
            let h = out.flex_hessian().unwrap().world(w).unwrap();
            assert!(
                h.flexvert_hessian[18..]
                    .iter()
                    .chain(&h.flexedge_hessian[27..])
                    .all(|&v| v == 37.0)
            );
            assert!(
                h.flexvert_hessian[..18]
                    .iter()
                    .chain(&h.flexedge_hessian[..27])
                    .all(|&v| v != 37.0)
            );
        }
        // A valid cache ignores even private geometry changes; G01 clears it.
        data.flex
            .as_mut()
            .unwrap()
            .write_range(4, &[7.0; 18])
            .unwrap();
        plan.update_flex_hessian(&mut data).unwrap();
        assert_eq!(
            out.flex_hessian().unwrap().values,
            data.readback().unwrap().flex_hessian().unwrap().values
        );
        plan.update(&mut data).unwrap();
        assert!(data.readback().unwrap().flex_hessian().is_none());
        plan.update_flex_hessian(&mut data).unwrap();
        assert!(
            !data.readback().unwrap().flex_hessian().unwrap().values[4..4 + width].contains(&37.0)
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn stretch_hessian_rejects_guards_nonfinite_and_late_publish_failure() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let mut plan = stretch_plan(&session);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        plan.update_flex_hessian(&mut data).unwrap();
        let len = data.layout.hessian.unwrap().guarded;
        for index in (0..4).chain(len - 4..len) {
            data.hessian
                .as_mut()
                .unwrap()
                .write_range(index, &[0.0])
                .unwrap();
            assert!(data.readback().is_err());
            data.hessian
                .as_mut()
                .unwrap()
                .write_range(index, &[-131072.0])
                .unwrap();
        }
        for index in [4, 4 + 90 * 256, len - 5] {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                data.hessian
                    .as_mut()
                    .unwrap()
                    .write_range(index, &[value])
                    .unwrap();
                assert!(matches!(
                    data.readback(),
                    Err(TransferError::Input(InputError::NonFinite {
                        field: "resident_flex_hessian_output",
                        ..
                    }))
                ));
                plan.update(&mut data).unwrap();
                plan.update_flex_hessian(&mut data).unwrap();
                data.readback().unwrap();
            }
        }
        for publish in [false, true] {
            plan.update(&mut data).unwrap();
            let replacement = SynchronousKernel::compile(
                &foreign,
                super::super::flex_hessian::FLEX_HESSIAN_CUDA,
                if publish {
                    "flex_hessian_validate"
                } else {
                    "flex_hessian"
                },
            )
            .unwrap();
            let device = plan.hessian.as_mut().unwrap();
            let slot = if publish {
                &mut device.validate_kernel
            } else {
                &mut device.kernel
            };
            let original = std::mem::replace(slot, replacement);
            assert_eq!(
                plan.update_flex_hessian(&mut data),
                Err(TransferError::SessionMismatch)
            );
            assert!(data.readback().unwrap().flex_hessian().is_none());
            assert!(
                read_flex_validity(data.flex_validity.as_ref().unwrap(), 1034)
                    .unwrap()
                    .iter()
                    .all(|&v| !v)
            );
            let device = plan.hessian.as_mut().unwrap();
            *if publish {
                &mut device.validate_kernel
            } else {
                &mut device.kernel
            } = original;
            plan.update_flex_hessian(&mut data).unwrap();
            data.readback().unwrap();
        }
        // Finite positions/edge lengths can still overflow material arithmetic.
        data.flex
            .as_mut()
            .unwrap()
            .write_range(4, &[f32::MAX, -f32::MAX, 0.0, 0.0, f32::MAX, 0.0].repeat(3))
            .unwrap();
        data.edges
            .as_mut()
            .unwrap()
            .values
            .write_range(4, &[f32::MAX; 6])
            .unwrap();
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &[0; 2])
            .unwrap();
        plan.update_flex_hessian(&mut data).unwrap();
        assert!(data.readback().is_err());
        let mixed = flex_hessian_plan(&session);
        let mut state = mixed.create_data(2).unwrap();
        mixed.update(&mut state).unwrap();
        mixed.update_flex_hessian(&mut state).unwrap();
        assert!(state.readback().unwrap().flex_faces().is_some());
        assert!(state.readback().unwrap().sleep_trees().is_some());
        assert_eq!(
            state
                .readback()
                .unwrap()
                .flex_positions()
                .unwrap()
                .world(1)
                .unwrap()
                .flex_hessian_valid,
            &[true; 3]
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_hessian_clears_all_mixed_sleeping_worlds_before_positions() {
        let session = TransferSession::new(0).unwrap();
        let plan = flex_hessian_plan(&session);
        let mut states = 0;
        let mut flags = 0;
        for worlds in [1, 2, 5, 513] {
            let mut data = plan.create_data(worlds).unwrap();
            assert_eq!(
                read_flex_validity(data.flex_validity.as_ref().unwrap(), 8 + 3 * worlds).unwrap(),
                vec![false; 3 * worlds]
            );
            let sleeping = SleepTreeState {
                tree_asleep: vec![0],
                nbody_awake: 1,
                nv_awake: 0,
            };
            for w in 0..worlds {
                data.write_world_sleep(w, &sleeping).unwrap();
            }
            for round in 0..4 {
                // Simulate a future cache producer privately; no public write API.
                let seed: Vec<_> = (0..3 * worlds)
                    .map(|i| {
                        if round == 0 {
                            1
                        } else {
                            (i + round) as i32 % 2
                        }
                    })
                    .collect();
                data.flex_validity
                    .as_mut()
                    .unwrap()
                    .write_range(4, &seed)
                    .unwrap();
                if round == 0 {
                    assert!(matches!(
                        plan.update_flex_positions(&mut data),
                        Err(TransferError::StageNotReady { stage: "rigid" })
                    ));
                    assert_eq!(
                        read_flex_validity(data.flex_validity.as_ref().unwrap(), 8 + 3 * worlds)
                            .unwrap(),
                        vec![true; 3 * worlds]
                    );
                }
                plan.update(&mut data).unwrap();
                let out = data.readback().unwrap();
                assert_eq!(out.flex_positions().unwrap().nflex(), 3);
                for w in 0..worlds {
                    assert_eq!(
                        out.flex_positions()
                            .unwrap()
                            .world(w)
                            .unwrap()
                            .flex_hessian_valid,
                        &[false; 3]
                    );
                    assert_eq!(
                        out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                        &[0]
                    );
                }
                states += worlds;
                flags += 3 * worlds;
            }
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(4, &vec![1; 3 * worlds])
                .unwrap();
            // Non-position stages must not clear this private simulated cache.
            plan.update_attached(&mut data).unwrap();
            plan.update_com(&mut data).unwrap();
            plan.update_camlight(&mut data).unwrap();
            data.write_world_qvel(worlds - 1, &[0.0; 6]).unwrap();
            plan.update_flex_edges(&mut data).unwrap();
            plan.update_flex_faces(&mut data).unwrap();
            plan.update_tendons(&mut data).unwrap();
            plan.update_tendon_wake(&mut data).unwrap();
            let old = data.readback().unwrap();
            assert_eq!(
                old.flex_positions()
                    .unwrap()
                    .world(worlds - 1)
                    .unwrap()
                    .flex_hessian_valid,
                &[true; 3]
            );
            assert!(data.write_world_qpos(worlds, &[]).is_err());
            assert!(data.write_world_qvel(0, &[f32::NAN; 6]).is_err());
            assert!(data.write_world_mocap(worlds, &[], &[]).is_err());
            assert_eq!(
                data.readback()
                    .unwrap()
                    .flex_positions()
                    .unwrap()
                    .world(0)
                    .unwrap()
                    .flex_hessian_valid,
                &[true; 3]
            );
            plan.update_flex_positions(&mut data).unwrap();
            assert!(matches!(
                data.readback(),
                Err(TransferError::StageNotReady {
                    stage: "flex_edges"
                })
            ));
            plan.update_flex_edges(&mut data).unwrap();
            plan.update_flex_faces(&mut data).unwrap();
            assert_eq!(
                data.readback()
                    .unwrap()
                    .flex_positions()
                    .unwrap()
                    .world(0)
                    .unwrap()
                    .flex_hessian_valid,
                &[false; 3]
            );
            assert_eq!(
                old.flex_positions()
                    .unwrap()
                    .world(0)
                    .unwrap()
                    .flex_hessian_valid,
                &[true; 3]
            );
        }
        assert_eq!((states, flags), (2084, 6252));
        println!("G01-flex-hessian states={states} flags={flags} exact_false=true");
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_hessian_rejects_guards_nonboolean_slots_and_late_launch_failure() {
        let session = TransferSession::new(0).unwrap();
        let mut plan = flex_hessian_plan(&session);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let length = data.layout.flex.unwrap().guarded_validity;
        for index in [0, 3, length - 4, length - 1] {
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(index, &[0])
                .unwrap();
            assert!(
                matches!(data.readback(), Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch {index: i,..})) if i == index)
            );
            // Clearing the payload must leave guard corruption visible.
            plan.update(&mut data).unwrap();
            assert!(data.readback().is_err());
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(index, &[-131072])
                .unwrap();
        }
        for index in [4, 4 + 3 * 256 + 1, length - 5] {
            for value in [-1, 2, i32::MIN, i32::MAX] {
                data.flex_validity
                    .as_mut()
                    .unwrap()
                    .write_range(index, &[value])
                    .unwrap();
                assert!(
                    matches!(data.readback(), Err(TransferError::Input(InputError::InvalidTopology {field: "resident_flex_hessian_valid",index: i,..})) if i == index - 4)
                );
                plan.update(&mut data).unwrap();
                data.readback().unwrap();
            }
        }
        let foreign = TransferSession::new(0).unwrap();
        let other = flex_hessian_plan(&session);
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &vec![1; 3 * 513])
            .unwrap();
        assert_eq!(
            other.update_flex_positions(&mut data),
            Err(TransferError::ModelMismatch)
        );
        assert!(
            read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
                .unwrap()
                .iter()
                .all(|&v| v)
        );
        let replacement = SynchronousKernel::compile(
            &foreign,
            super::super::flex::FLEX_POSITION_CUDA,
            "flex_positions",
        )
        .unwrap();
        let original = std::mem::replace(&mut plan.flex.as_mut().unwrap().kernel, replacement);
        assert_eq!(
            plan.update_flex_positions(&mut data),
            Err(TransferError::SessionMismatch)
        );
        // A later position-launch failure must not retain the valid flags.
        assert!(
            read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
                .unwrap()
                .iter()
                .all(|&v| !v)
        );
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
        plan.flex.as_mut().unwrap().kernel = original;
        plan.update(&mut data).unwrap();
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &vec![1; 3 * 513])
            .unwrap();
        let replacement = SynchronousKernel::compile(
            &foreign,
            super::super::flex::FLEX_POSITION_CUDA,
            "flex_invalidate",
        )
        .unwrap();
        let original = std::mem::replace(
            &mut plan.flex.as_mut().unwrap().invalidate_kernel,
            replacement,
        );
        assert_eq!(
            plan.update_flex_positions(&mut data),
            Err(TransferError::SessionMismatch)
        );
        assert!(
            read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
                .unwrap()
                .iter()
                .all(|&v| v)
        );
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_positions"
            })
        ));
        plan.flex.as_mut().unwrap().invalidate_kernel = original;
        let qpos = plan.model.rigid().kinematics().fields().qpos0.clone();
        data.write_world_qpos(512, &qpos).unwrap();
        assert!(matches!(
            plan.update_flex_positions(&mut data),
            Err(TransferError::StageNotReady { stage: "rigid" })
        ));
        plan.update(&mut data).unwrap();
        let snapshot = data.readback().unwrap();
        drop(plan);
        drop(other);
        drop(foreign);
        drop(session);
        assert_eq!(
            data.readback()
                .unwrap()
                .flex_positions()
                .unwrap()
                .world(512)
                .unwrap()
                .flex_hessian_valid,
            &[false; 3]
        );
        drop(data);
        assert_eq!(
            snapshot
                .flex_positions()
                .unwrap()
                .world(512)
                .unwrap()
                .flex_hessian_valid,
            &[false; 3]
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_hessian_handles_empty_and_static_direct_flex_routes() {
        let session = TransferSession::new(0).unwrap();
        let old = KinematicsPlan::new(&session, empty_model()).unwrap();
        let mut data = old.create_data(5).unwrap();
        assert!(data.flex_validity.is_none());
        old.update(&mut data).unwrap();
        assert!(data.readback().unwrap().flex_positions().is_none());
        for nf in [0, 1, 3] {
            let cam = CamLightModelInput::new(
                MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
                CamLightFields::default(),
            )
            .unwrap();
            let input = FlexPositionModelInput::new(
                TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
                FlexPositionFields {
                    flex_interp: vec![0; nf],
                    flex_cellnum: vec![1; 3 * nf],
                    flex_nodeadr: vec![0; nf],
                    flex_nodenum: vec![0; nf],
                    flex_vertadr: (0..nf as i32).collect(),
                    flex_vertnum: vec![1; nf],
                    flex_centered: vec![false; nf],
                    flex_vertbodyid: vec![0; nf],
                    flex_vert: vec![0.0; 3 * nf],
                    flex_vert0: vec![0.0; 3 * nf],
                    ..Default::default()
                },
            )
            .unwrap();
            let plan = KinematicsPlan::with_flex_positions(
                &session,
                input,
                KinematicsParameters::default(),
                CamLightParameters::default(),
            )
            .unwrap();
            let mut data = plan.create_data(513).unwrap();
            assert_eq!(data.flex.as_ref().unwrap().len(), 8 + 3 * 513 * nf);
            assert_eq!(data.flex_validity.as_ref().unwrap().len(), 8 + 513 * nf);
            if nf != 0 {
                data.flex_validity
                    .as_mut()
                    .unwrap()
                    .write_range(4, &vec![1; 513 * nf])
                    .unwrap();
            }
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            let f = out.flex_positions().unwrap();
            assert_eq!(f.nflex(), nf);
            assert_eq!(f.world(512).unwrap().flex_hessian_valid, vec![false; nf]);
            assert_eq!(f.world(512).unwrap().flexvert_xpos, vec![0.0; 3 * nf]);
            assert!(f.world(513).is_err());
        }
    }

    fn empty_model() -> AttachedModelInput {
        let k = KinematicModelInput::new(
            0,
            KinematicFields {
                body_parentid: vec![0],
                body_jntadr: vec![-1],
                body_jntnum: vec![0],
                body_pos: vec![0.0; 3],
                body_quat: vec![1.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        )
        .unwrap();
        let i = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 3],
                body_iquat: vec![1.0, 0.0, 0.0, 0.0],
                body_mass: vec![0.0],
                body_inertia: vec![0.0; 3],
                ..Default::default()
            },
        )
        .unwrap();
        AttachedModelInput::new(i, AttachedFields::default()).unwrap()
    }

    #[test]
    fn checks_all_layouts_before_device_allocation() {
        let m = empty_model();
        for worlds in [0, u32::MAX as usize, usize::MAX] {
            assert!(matches!(
                ResidentLayout::new(&m, worlds, 0),
                Err(InputError::InvalidDimension { .. })
            ));
        }
        let l = ResidentLayout::new(&m, 513, 0).unwrap();
        assert_eq!(l.qpos.total_elements(), 0);
        assert_eq!(l.rigid.output.total_elements(), 513 * 28);
        assert_eq!(l.com.output.total_elements(), 513 * 14);
        assert_eq!(l.attached.output.total_elements(), 0);
        assert!(l.qpos.world_elements(512).is_ok());
        assert!(l.qpos.world_elements(513).is_err());
    }

    #[test]
    fn rejects_unready_and_stale_stage_combinations() {
        let mut s = ReadyStages::default();
        assert_eq!(
            s.require_all(),
            Err(TransferError::StageNotReady { stage: "rigid" })
        );
        s.rigid = true;
        assert_eq!(
            s.require_all(),
            Err(TransferError::StageNotReady { stage: "attached" })
        );
        s.attached = true;
        assert_eq!(
            s.require_all(),
            Err(TransferError::StageNotReady { stage: "com" })
        );
        s.com = true;
        assert!(s.require_all().is_ok());
        s = ReadyStages::default();
        assert!(s.require_all().is_err());
    }

    #[test]
    fn model_identity_distinguishes_equal_inputs() {
        let a = Arc::new(empty_model());
        let b = Arc::new((*a).clone());
        assert_eq!(a, b);
        assert!(!Arc::ptr_eq(&a, &b));
        assert!(Arc::ptr_eq(&a, &Arc::clone(&a)));
    }

    fn parameter_model(free: bool) -> AttachedModelInput {
        let k = KinematicModelInput::new(
            if free { 6 } else { 1 },
            KinematicFields {
                qpos0: if free {
                    vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
                } else {
                    vec![0.0]
                },
                body_parentid: vec![0, 0],
                body_jntadr: vec![-1, 0],
                body_jntnum: vec![0, 1],
                body_pos: vec![0.0; 6],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                jnt_type: vec![if free { 0 } else { 2 }],
                jnt_bodyid: vec![1],
                jnt_qposadr: vec![0],
                jnt_dofadr: vec![0],
                jnt_pos: vec![0.0; 3],
                jnt_axis: vec![1.0, 0.0, 0.0],
            },
        )
        .unwrap();
        let nv = k.nv();
        let rigid = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 6],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                body_mass: vec![0.0, 2.0],
                body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                dof_bodyid: vec![1; nv],
                dof_jntid: vec![0; nv],
                dof_parentid: (0..nv).map(|i| i as i32 - 1).collect(),
                dof_armature: vec![0.0; nv],
                dof_damping: vec![0.0; nv],
            },
        )
        .unwrap();
        AttachedModelInput::new(
            rigid,
            AttachedFields {
                geom_bodyid: vec![1],
                geom_pos: vec![0.0; 3],
                geom_quat: vec![1.0, 0.0, 0.0, 0.0],
                site_bodyid: vec![1],
                site_pos: vec![0.0; 3],
                site_quat: vec![1.0, 0.0, 0.0, 0.0],
            },
        )
        .unwrap()
    }

    #[test]
    fn validates_all_parameter_rows_and_model_dimensions() {
        use crate::model::ParameterBatch;
        let model = parameter_model(false);
        let mut parameters = KinematicsParameters::default();
        for (i, field) in KinematicsParameter::ALL.into_iter().enumerate() {
            let shared = field.shared(&model);
            parameters.set(
                field,
                ParameterBatch::new(i + 1, shared.len(), shared.repeat(i + 1)).unwrap(),
            );
            check_parameters(&model, &parameters).unwrap();
            let mut bad = parameters.clone();
            bad.set(
                field,
                ParameterBatch::new(2, shared.len() + 1, vec![0.0; 2 * (shared.len() + 1)])
                    .unwrap(),
            );
            assert!(
                matches!(check_parameters(&model, &bad), Err(InputError::LengthMismatch { field: f, .. }) if f == field.name())
            );
        }
    }

    #[test]
    fn rejects_invalid_physical_parameters_in_later_rows() {
        use crate::model::{KinematicsParameter as F, ParameterBatch};
        let model = parameter_model(false);
        for (field, local, value, reason) in [
            (F::BodyPos, 0, 1.0, "noncanonical_world"),
            (F::BodyIpos, 1, 1.0, "noncanonical_world"),
            (F::BodyQuat, 0, -1.0, "noncanonical_world"),
            (F::BodyIquat, 0, -1.0, "noncanonical_world"),
            (F::BodyQuat, 4, 0.5, "nonunit_model_rotation"),
            (F::BodyIquat, 4, 0.5, "nonunit_model_rotation"),
            (F::GeomQuat, 0, 0.5, "nonunit_model_rotation"),
            (F::SiteQuat, 0, 0.5, "nonunit_model_rotation"),
            (F::JointAxis, 0, 0.5, "nonunit_joint_axis"),
            (F::BodyMass, 0, 1.0, "nonzero_world_inertia"),
            (F::BodyInertia, 2, 1.0, "nonzero_world_inertia"),
        ] {
            let shared = field.shared(&model);
            let mut rows = shared.repeat(3);
            rows[2 * shared.len() + local] = value;
            let mut parameters = KinematicsParameters::default();
            parameters.set(field, ParameterBatch::new(3, shared.len(), rows).unwrap());
            assert!(
                matches!(check_parameters(&model, &parameters), Err(InputError::InvalidTopology { field: f, reason: r, .. }) if f == field.name() && r == reason)
            );
        }
        for (field, local) in [(F::BodyMass, 1), (F::BodyInertia, 4)] {
            let shared = field.shared(&model);
            let mut rows = shared.repeat(3);
            rows[2 * shared.len() + local] = -0.1;
            let mut p = KinematicsParameters::default();
            p.set(field, ParameterBatch::new(3, shared.len(), rows).unwrap());
            assert_eq!(
                check_parameters(&model, &p),
                Err(InputError::NegativeValue {
                    field: field.name(),
                    index: 2 * shared.len() + local
                })
            );
        }
        let free = parameter_model(true);
        let mut rows = F::Qpos0.shared(&free).repeat(3);
        rows[17] = 0.0;
        let mut p = KinematicsParameters::default();
        p.set(F::Qpos0, ParameterBatch::new(3, 7, rows).unwrap());
        assert!(matches!(
            check_parameters(&free, &p),
            Err(InputError::InvalidTopology {
                field: "qpos0",
                index: 17,
                reason: "unusable_state_quaternion"
            })
        ));
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    fn packs_independent_periods_and_checks_aggregate_capacity() {
        use crate::model::{KinematicsParameter as F, ParameterBatch};
        let model = parameter_model(false);
        let mut parameters = KinematicsParameters::default();
        parameters.set(
            F::BodyMass,
            ParameterBatch::new(3, 2, vec![0.0, 1.0, 0.0, 2.0, 0.0, 3.0]).unwrap(),
        );
        let packed = pack_model(&model, &parameters, &[&[0, 0]], &COM_PARAMETERS).unwrap();
        assert_eq!(packed.metadata, [0, 0, 0, 3, 6, 1]);
        assert_eq!(&packed.parameters[..6], &[0.0, 1.0, 0.0, 2.0, 0.0, 3.0]);
        assert_eq!(packed.parameters.len(), 12);
        assert_eq!(
            packed_length([i32::MAX as usize].into_iter()).unwrap(),
            i32::MAX as usize
        );
        for lengths in [[i32::MAX as usize, 1], [usize::MAX, 1]] {
            assert!(packed_length(lengths.into_iter()).is_err());
        }
        let empty =
            pack_model(&empty_model(), &parameters, &[&[28]], &ATTACHED_PARAMETERS).unwrap();
        assert_eq!(empty.parameters, [0.0]);
        assert_eq!(empty.metadata, [28, 0, 1, 0, 1, 0, 1, 0, 1]);
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn static_geom_cache_survives_changed_body_frames_while_sites_update() {
        let session = TransferSession::new(0).unwrap();
        for (ng, ns) in [(1, 1), (1, 0), (0, 1), (0, 0)] {
            let model = AttachedModelInput::new(
                empty_model().rigid().clone(),
                crate::model::AttachedFields {
                    geom_bodyid: vec![0; ng],
                    geom_pos: [1.0, 2.0, 3.0].repeat(ng),
                    geom_quat: [1.0, 0.0, 0.0, 0.0].repeat(ng),
                    site_bodyid: vec![0; ns],
                    site_pos: [4.0, 5.0, 6.0].repeat(ns),
                    site_quat: [1.0, 0.0, 0.0, 0.0].repeat(ns),
                },
            )
            .unwrap();
            let plan = KinematicsPlan::new(&session, model).unwrap();
            let mut data = plan.create_data(513).unwrap();
            let initial = read_output(
                &data.attached,
                data.layout.attached.guarded,
                "initial_static",
            )
            .unwrap();
            if ng != 0 {
                assert_eq!(&initial[4..7], &[1.0, 2.0, 3.0]);
            }
            assert!(data.write_mocap(&[1.0], &[]).is_err());
            data.write_mocap(&[], &[]).unwrap();
            data.write_world_mocap(512, &[], &[]).unwrap();
            plan.update_rigid(&mut data).unwrap();
            // Test-only corruption distinguishes skip semantics from recomputing
            // an unchanged static transform. No public API exposes this write.
            for world in 0..513 {
                data.rigid
                    .write_range(4 + world * 28, &[9.0, 8.0, 7.0])
                    .unwrap();
            }
            plan.update_attached(&mut data).unwrap();
            let updated = read_output(
                &data.attached,
                data.layout.attached.guarded,
                "updated_static",
            )
            .unwrap();
            for world in 0..513 {
                let start = 4 + world * 12 * (ng + ns);
                assert_eq!(
                    &initial[start..start + 12 * ng],
                    &updated[start..start + 12 * ng]
                );
                if ns != 0 {
                    assert_eq!(&updated[start + 12 * ng..start + 12 * ng + 3], &[13.0; 3]);
                }
            }
        }
    }

    #[test]
    fn validates_mocap_lengths_finiteness_and_quaternion_capacity() {
        assert!(check_mocap(0, 0, &[], &[]).is_ok());
        assert!(check_mocap(3, 4, &[0.0; 3], &[-2.0, -0.0, 0.0, 0.0]).is_ok());
        assert!(check_mocap(3, 4, &[0.0; 3], &[]).is_err());
        assert!(check_mocap(0, 0, &[1.0], &[]).is_err());
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                check_mocap(3, 4, &[bad, 0.0, 0.0], &[1.0, 0.0, 0.0, 0.0]),
                Err(InputError::NonFinite {
                    field: "mocap_pos",
                    index: 0
                })
            ));
            assert!(matches!(
                check_mocap(3, 4, &[0.0; 3], &[bad, 0.0, 0.0, 0.0]),
                Err(InputError::NonFinite {
                    field: "mocap_quat",
                    index: 0
                })
            ));
        }
        for bad in [0.0, 1e-7, 1e7, f32::MAX] {
            assert!(matches!(
                check_mocap(6, 8, &[0.0; 6], &[1.0, 0.0, 0.0, 0.0, bad, 0.0, 0.0, 0.0]),
                Err(InputError::InvalidTopology {
                    field: "mocap_quat",
                    index: 4,
                    ..
                })
            ));
        }
    }

    #[test]
    fn checks_packed_mocap_state_capacity_before_allocation() {
        let m = empty_model();
        let l = ResidentLayout::new(&m, 513, 2).unwrap();
        assert_eq!(l.state.total_elements(), 513 * 14);
        assert_eq!(l.mocap_pos.world_elements(512).unwrap(), 3072..3078);
        assert_eq!(l.mocap_quat.world_elements(512).unwrap(), 4096..4104);
        for nm in [i32::MAX as usize / 7 + 1, usize::MAX] {
            assert!(matches!(
                ResidentLayout::new(&m, 1, nm),
                Err(InputError::Overflow {
                    field: "resident_state"
                })
            ));
        }
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn camlight_readback_rejects_device_guards_and_nonfinite_payloads() {
        let session = TransferSession::new(0).unwrap();
        let model = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields {
                cam_mode: vec![0],
                cam_bodyid: vec![0],
                cam_targetbodyid: vec![-1],
                cam_pos: vec![1.0, 2.0, 3.0],
                cam_quat: vec![1.0, 0.0, 0.0, 0.0],
                cam_poscom0: vec![0.0; 3],
                cam_pos0: vec![0.0; 3],
                cam_mat0: vec![0.0; 9],
                light_mode: vec![0],
                light_bodyid: vec![0],
                light_targetbodyid: vec![-1],
                light_pos: vec![1.0, 2.0, 3.0],
                light_dir: vec![0.0, 0.0, 1.0],
                light_poscom0: vec![0.0; 3],
                light_pos0: vec![0.0; 3],
                light_dir0: vec![0.0; 3],
            },
        )
        .unwrap();
        let plan = KinematicsPlan::with_camlight(
            &session,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let len = data.layout.camlight.guarded;
        for index in [0, 3, len - 4, len - 1] {
            data.camlight.write_range(index, &[0.0]).unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index)
            );
            data.camlight.write_range(index, &[-131072.0]).unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for index in [4, 4 + 512 * 18 + 3, 4 + 512 * 18 + 12, 4 + 512 * 18 + 15] {
                data.camlight.write_range(index, &[bad]).unwrap();
                assert!(
                    matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{field:"resident_camlight_output",index:i})) if i==index-4)
                );
                plan.update_camlight(&mut data).unwrap();
            }
        }
        data.readback().unwrap();
    }

    #[cfg(feature = "cuda-probe")]
    fn fixed_tendon_plan(session: &TransferSession, coefficient: f32) -> KinematicsPlan {
        let camlight = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let model = FixedTendonModelInput::new(
            camlight,
            FixedTendonFields {
                tendon_adr: vec![0],
                tendon_num: vec![1],
                wrap_type: vec![1],
                wrap_objid: vec![0],
                wrap_prm: vec![coefficient],
                ten_j_rowadr: vec![0],
                ten_j_rownnz: vec![1],
                ten_j_colind: vec![0],
            },
        )
        .unwrap();
        KinematicsPlan::with_fixed_tendons(
            session,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn global_tendon_readback_rejects_guards_and_nonfinite_values() {
        let session = TransferSession::new(0).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let model = TendonModelInput::new(
            cam,
            crate::model::TendonFields {
                tendon_adr: vec![0],
                tendon_num: vec![1],
                wrap_type: vec![1],
                wrap_objid: vec![0],
                wrap_prm: vec![-2.0],
                ten_j_rowadr: vec![0],
                ten_j_rownnz: vec![1],
                ten_j_colind: vec![0],
            },
        )
        .unwrap();
        let plan = KinematicsPlan::with_tendons(
            &session,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let layout = data.layout.tendon.unwrap();
        for index in [0, 3, layout.guarded - 4, layout.guarded - 1] {
            data.tendon
                .as_mut()
                .unwrap()
                .values
                .write_range(index, &[0.0])
                .unwrap();
            assert!(matches!(data.readback(),Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index));
            data.tendon
                .as_mut()
                .unwrap()
                .values
                .write_range(index, &[-131072.0])
                .unwrap();
        }
        for index in [0, 3, layout.guarded_indices - 4, layout.guarded_indices - 1] {
            data.tendon
                .as_mut()
                .unwrap()
                .indices
                .write_range(index, &[0])
                .unwrap();
            assert!(matches!(data.readback(),Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index));
            data.tendon
                .as_mut()
                .unwrap()
                .indices
                .write_range(index, &[-131072])
                .unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for index in [4, 4 + 512 * 8, 4 + 512 * 8 + 1, layout.guarded - 5] {
                data.tendon
                    .as_mut()
                    .unwrap()
                    .values
                    .write_range(index, &[bad])
                    .unwrap();
                assert!(
                    matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{
                    field:"resident_tendon_output",index:i})) if i==index-4)
                );
                plan.update_tendons(&mut data).unwrap();
            }
        }
        data.readback().unwrap();
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn fixed_tendon_readback_rejects_device_guards_and_nonfinite_payloads() {
        let session = TransferSession::new(0).unwrap();
        let plan = fixed_tendon_plan(&session, -2.0);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let length = data.layout.fixed_tendon.guarded;
        for index in [0, 3, length - 4, length - 1] {
            data.fixed_tendon.write_range(index, &[0.0]).unwrap();
            assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
            data.fixed_tendon.write_range(index, &[-131072.0]).unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for index in [4, 5, 4 + 512 * 2, 5 + 512 * 2] {
                data.fixed_tendon.write_range(index, &[bad]).unwrap();
                assert!(matches!(data.readback(), Err(TransferError::Input(
                    InputError::NonFinite { field: "resident_fixed_tendon_output", index: i })) if i==index-4));
                plan.update_fixed_tendons(&mut data).unwrap();
            }
        }
        data.readback().unwrap();
        // An empty zero-DOF model still exposes checked empty tendon views.
        let plan = KinematicsPlan::new(&session, empty_model()).unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        assert!(
            data.readback()
                .unwrap()
                .fixed_tendon()
                .world(512)
                .unwrap()
                .ten_jacobian
                .is_empty()
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn fixed_tendon_readback_rejects_arithmetic_overflow_and_recovers() {
        let session = TransferSession::new(0).unwrap();
        let plan = fixed_tendon_plan(&session, f32::MAX);
        let mut data = plan.create_data(513).unwrap();
        data.write_world_qpos(512, &[2.0]).unwrap();
        plan.update(&mut data).unwrap();
        assert!(matches!(
            data.readback(),
            Err(TransferError::Input(InputError::NonFinite {
                field: "resident_fixed_tendon_output",
                index: 1024
            }))
        ));
        data.write_world_qpos(512, &[0.5]).unwrap();
        plan.update(&mut data).unwrap();
        assert_eq!(
            data.readback()
                .unwrap()
                .fixed_tendon()
                .world(512)
                .unwrap()
                .ten_length,
            &[f32::MAX * 0.5]
        );
    }

    #[cfg(feature = "cuda-probe")]
    fn spatial_tendon_plan(session: &TransferSession) -> KinematicsPlan {
        let base = parameter_model(false);
        let mut fields = base.fields().clone();
        fields.site_bodyid = vec![0, 1];
        fields.site_pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        fields.site_quat = [1.0, 0.0, 0.0, 0.0].repeat(2);
        let attached = AttachedModelInput::new(base.rigid().clone(), fields).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(attached, vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let fixed = FixedTendonModelInput::new(cam, FixedTendonFields::default()).unwrap();
        let model = SpatialTendonModelInput::new(
            fixed,
            SpatialTendonFields {
                tendon_adr: vec![0],
                tendon_num: vec![2],
                wrap_type: vec![3, 3],
                wrap_objid: vec![0, 1],
                wrap_prm: vec![0.0; 2],
                ten_j_rowadr: vec![0],
                ten_j_rownnz: vec![1],
                ten_j_colind: vec![0],
            },
        )
        .unwrap();
        KinematicsPlan::with_spatial_tendons(
            session,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn spatial_tendon_readback_rejects_both_device_guards_and_nonfinite_payloads() {
        let session = TransferSession::new(0).unwrap();
        let plan = spatial_tendon_plan(&session);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        for index in [
            0,
            3,
            data.layout.spatial_tendon.guarded - 4,
            data.layout.spatial_tendon.guarded - 1,
        ] {
            data.spatial_tendon.write_range(index, &[0.0]).unwrap();
            assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
            data.spatial_tendon
                .write_range(index, &[-131072.0])
                .unwrap();
        }
        for index in [
            0,
            3,
            data.layout.spatial_tendon.guarded_indices - 4,
            data.layout.spatial_tendon.guarded_indices - 1,
        ] {
            data.spatial_wrap.write_range(index, &[0]).unwrap();
            assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
            data.spatial_wrap.write_range(index, &[-131072]).unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for index in [
                4,
                5,
                6,
                4 + 512 * 28,
                5 + 512 * 28,
                6 + 512 * 28,
                18 + 512 * 28,
                31 + 512 * 28,
            ] {
                data.spatial_tendon.write_range(index, &[bad]).unwrap();
                assert!(matches!(data.readback(), Err(TransferError::Input(
                    InputError::NonFinite { field: "resident_spatial_tendon_output", index: i })) if i==index-4));
                plan.update_spatial_tendons(&mut data).unwrap();
            }
        }
        // Every update clears unused integer and position slots too.
        data.spatial_wrap
            .write_range(4, &vec![i32::MAX; 513 * 6])
            .unwrap();
        plan.update_spatial_tendons(&mut data).unwrap();
        let out = data.readback().unwrap();
        let row = out.spatial_tendon().world(512).unwrap();
        assert_eq!(row.wrap_obj, &[-1, -1, 0, 0]);
        assert_eq!(&row.wrap_xpos[6..], &[0.0; 6]);
        let empty = KinematicsPlan::new(&session, empty_model()).unwrap();
        let mut data = empty.create_data(513).unwrap();
        empty.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let row = out.spatial_tendon().world(512).unwrap();
        assert!(row.ten_length.is_empty() && row.wrap_obj.is_empty());
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_edge_free_body_matches_translation_rotation_and_velocity() {
        let session = TransferSession::new(0).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![2],
                flex_centered: vec![false],
                flex_vertbodyid: vec![0, 1],
                flex_vert: vec![2.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                flex_vert0: vec![0.0; 6],
                ..Default::default()
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
            flex_edge: vec![0, 1],
            flexedge_j_rowadr: vec![0],
            flexedge_j_rownnz: vec![6],
            flexedge_j_colind: (0..6).collect(),
        })
        .unwrap();
        let plan = KinematicsPlan::with_flex_positions(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        data.write_qvel(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0].repeat(513))
            .unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let norm = 5.0_f32.sqrt();
        for w in 0..513 {
            let f = out.flex_edges().unwrap().world(w).unwrap();
            assert!((f.flexedge_length[0] - norm).abs() < 1e-6);
            for (&a, e) in
                f.flexedge_jacobian
                    .iter()
                    .zip([-2.0 / norm, 1.0 / norm, 0.0, 0.0, 0.0, 2.0 / norm])
            {
                assert!((a - e).abs() < 1e-6);
            }
            assert!((f.flexedge_velocity[0] - 12.0 / norm).abs() < 1e-6);
        }
        let cam = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![2],
                flex_centered: vec![false],
                flex_vertbodyid: vec![0; 2],
                flex_vert: vec![0.0, 0.0, 0.0, 3.0, 4.0, 0.0],
                flex_vert0: vec![0.0; 6],
                ..Default::default()
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
            flex_edge: vec![0, 1],
            flexedge_j_rowadr: vec![0],
            flexedge_j_rownnz: vec![0],
            flexedge_j_colind: vec![],
        })
        .unwrap();
        let plan = KinematicsPlan::with_flex_positions(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        data.write_qvel(&[]).unwrap();
        data.write_world_qvel(512, &[]).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for w in 0..513 {
            let f = out.flex_edges().unwrap().world(w).unwrap();
            assert_eq!(f.flexedge_length, &[5.0]);
            assert_eq!(f.flexedge_velocity, &[0.0]);
            assert!(f.flexedge_jacobian.is_empty());
        }
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_edge_readback_rejects_guards_nonfinite_and_foreign_inputs() {
        let session = TransferSession::new(0).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![2],
                flex_centered: vec![false],
                flex_vertbodyid: vec![0, 1],
                flex_vert: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                flex_vert0: vec![0.0; 6],
                ..Default::default()
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
            flex_edge: vec![0, 1],
            flexedge_j_rowadr: vec![0],
            flexedge_j_rownnz: vec![1],
            flexedge_j_colind: vec![0],
        })
        .unwrap();
        let plan = KinematicsPlan::with_flex_positions(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let l = data.layout.edges.unwrap();
        for qvel in [false, true] {
            let len = if qvel { l.guarded_qvel } else { l.guarded };
            for index in [0, 3, len - 4, len - 1] {
                let e = data.edges.as_mut().unwrap();
                let b = if qvel { &mut e.qvel } else { &mut e.values };
                b.write_range(index, &[0.0]).unwrap();
                assert!(matches!(data.readback(),Err(TransferError::Backend(
                    crate::diagnostics::ProbeError::Mismatch {index:i,..})) if i==index));
                let e = data.edges.as_mut().unwrap();
                let b = if qvel { &mut e.qvel } else { &mut e.values };
                b.write_range(index, &[-131072.0]).unwrap();
            }
            for index in [4, 4 + (len - 8) / 2, len - 5] {
                for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                    let e = data.edges.as_mut().unwrap();
                    let b = if qvel { &mut e.qvel } else { &mut e.values };
                    b.write_range(index, &[value]).unwrap();
                    let field = if qvel {
                        "resident_qvel_input"
                    } else {
                        "resident_flex_edge_output"
                    };
                    assert!(matches!(data.readback(),Err(TransferError::Input(
                        InputError::NonFinite {field:f,index:i})) if f==field && i==index-4));
                    if qvel {
                        data.edges
                            .as_mut()
                            .unwrap()
                            .qvel
                            .write_range(index, &[0.0])
                            .unwrap();
                    }
                    plan.update_flex_edges(&mut data).unwrap();
                }
            }
        }
        data.flex
            .as_mut()
            .unwrap()
            .write_range(4, &[f32::MAX, 0.0, 0.0])
            .unwrap();
        plan.update_flex_edges(&mut data).unwrap();
        assert!(matches!(
            data.readback(),
            Err(TransferError::Input(InputError::NonFinite {
                field: "resident_flex_edge_output",
                index: 0
            }))
        ));
        plan.update_flex_positions(&mut data).unwrap();
        plan.update_flex_edges(&mut data).unwrap();
        let device = plan.edges.as_ref().unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let foreign_qvel = foreign.upload(&vec![0.0; l.guarded_qvel]).unwrap();
        let flex = data.flex.as_ref().unwrap();
        let edges = data.edges.as_mut().unwrap();
        // SAFETY: These invalid calls exit before launching. Other buffers keep
        // their checked resident layouts and exact private kernel ABI.
        unsafe {
            assert!(matches!(
                device.kernel.launch_flex_edges(
                    &device.metadata,
                    &foreign_qvel,
                    &data.com,
                    flex,
                    &mut edges.values,
                    513
                ),
                Err(TransferError::SessionMismatch)
            ));
            for worlds in [0, u32::MAX] {
                assert!(matches!(
                    device.kernel.launch_flex_edges(
                        &device.metadata,
                        &edges.qvel,
                        &data.com,
                        flex,
                        &mut edges.values,
                        worlds
                    ),
                    Err(TransferError::Input(InputError::InvalidDimension { .. }))
                ));
            }
        }
        data.readback().unwrap();
    }

    #[cfg(feature = "cuda-probe")]
    fn flex_face_plan(
        session: &TransferSession,
        nodes: Vec<f32>,
        cells: [i32; 3],
    ) -> KinematicsPlan {
        let cam = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let tendons = TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap();
        let count = nodes.len() / 3;
        let mut faces = FlexFaceFields::default();
        let nf = 2 * (cells[0] * cells[1] + cells[0] * cells[2] + cells[1] * cells[2]);
        for f in 0..nf {
            faces.flex_face_map.extend([0, f]);
            faces
                .flex_face
                .extend(crate::model::flex::face_nodes(&cells, f as usize).1);
            faces.flex_face.extend([-1; 5]);
        }
        let model = FlexPositionModelInput::new(
            tendons,
            FlexPositionFields {
                flex_interp: vec![-1],
                flex_cellnum: cells.to_vec(),
                flex_nodeadr: vec![0],
                flex_nodenum: vec![count as i32],
                flex_vertadr: vec![0],
                flex_vertnum: vec![1],
                flex_centered: vec![false],
                flex_nodebodyid: vec![0; count],
                flex_vertbodyid: vec![-1],
                flex_node: nodes,
                flex_vert: vec![0.0; 3],
                flex_vert0: vec![0.5; 3],
            },
        )
        .unwrap()
        .with_faces(faces)
        .unwrap();
        KinematicsPlan::with_flex_positions(
            session,
            model,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_face_checks_analytic_rotations_shear_interior_and_degenerate_states() {
        let session = TransferSession::new(0).unwrap();
        for (angle, shear, zero) in [
            (0.0f32, 0.0, false),
            (std::f32::consts::FRAC_PI_4, 0.0, false),
            (0.0, 0.4, false),
            (0.0, 0.0, true),
        ] {
            let mut nodes = Vec::new();
            for x in 0..3 {
                for y in 0..3 {
                    for z in 0..3 {
                        let (x, y, z) = (x as f32, y as f32, z as f32);
                        let (a, b) = (x + shear * y, y);
                        nodes.extend(if zero {
                            [0.0; 3]
                        } else {
                            [
                                angle.cos() * a - angle.sin() * b + 1.0,
                                angle.sin() * a + angle.cos() * b - 2.0,
                                z + 3.0,
                            ]
                        });
                    }
                }
            }
            // Frozen nodal semantics retain a distinct body-attached interior.
            if angle == 0.0 && shear == 0.0 && !zero {
                nodes[3 * 13..3 * 13 + 3].copy_from_slice(&[9.0, 8.0, 7.0]);
            }
            let plan = flex_face_plan(&session, nodes, [2, 2, 2]);
            let mut data = plan.create_data(513).unwrap();
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            let f = out.flex_faces().unwrap().world(512).unwrap();
            let theta = if shear != 0.0 {
                -(shear / 2.0).atan()
            } else {
                angle
            };
            for face in 0..24 {
                assert!(
                    f.face_xpos[27 * face + 12..27 * face + 27]
                        .iter()
                        .all(|&v| v == 0.0)
                );
                if shear != 0.0 && face < 16 {
                    continue;
                }
                let q = &f.face_quat[4 * face..4 * face + 4];
                let expected = if zero {
                    [0.0, 0.0, 0.0, 1.0]
                } else {
                    [0.0, 0.0, (theta / 2.0).sin(), (theta / 2.0).cos()]
                };
                for (&a, &e) in q.iter().zip(&expected) {
                    assert!(
                        (a - e).abs() < 2e-5,
                        "face={face} actual={q:?} expected={expected:?}"
                    );
                }
            }
            if angle == 0.0 && shear == 0.0 && !zero {
                assert_eq!(
                    out.flex_positions()
                        .unwrap()
                        .world(512)
                        .unwrap()
                        .flexvert_xpos,
                    &[9.0, 8.0, 7.0]
                );
            }
        }
        // Preserve the frozen iteration's stationary 180-degree limitation.
        let mut nodes = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    nodes.extend([-(x as f32), -(y as f32), z as f32]);
                }
            }
        }
        let plan = flex_face_plan(&session, nodes, [1, 1, 1]);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        assert!(
            data.readback()
                .unwrap()
                .flex_faces()
                .unwrap()
                .world(512)
                .unwrap()
                .face_quat
                .as_chunks::<4>()
                .0
                .iter()
                .all(|q| *q == [0.0, 0.0, 0.0, 1.0])
        );
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow() {
        let session = TransferSession::new(0).unwrap();
        let mut nodes = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    nodes.extend([x as f32, y as f32, z as f32]);
                }
            }
        }
        let plan = flex_face_plan(&session, nodes, [1, 1, 1]);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let l = data.layout.faces.unwrap();
        for i in [0, 3, l.guarded - 4, l.guarded - 1] {
            data.faces.as_mut().unwrap().write_range(i, &[0.0]).unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch{index,..})) if index==i)
            );
            data.faces
                .as_mut()
                .unwrap()
                .write_range(i, &[-131072.0])
                .unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for i in [4, 4 + 512 * 186 + 100, l.guarded - 5] {
                data.faces.as_mut().unwrap().write_range(i, &[bad]).unwrap();
                assert!(
                    matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{field:"resident_flex_face_output",index})) if index==i-4)
                );
                plan.update_flex_faces(&mut data).unwrap();
            }
        }
        // Finite nodal values can overflow cross products and polar arithmetic.
        let mut huge = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    huge.extend([x as f32 * 1e20, y as f32 * 1e20, z as f32 * 1e20]);
                }
            }
        }
        data.flex
            .as_mut()
            .unwrap()
            .write_range(4 + 512 * 27, &huge)
            .unwrap();
        plan.update_flex_faces(&mut data).unwrap();
        assert!(matches!(
            data.readback(),
            Err(TransferError::Input(InputError::NonFinite {
                field: "resident_flex_face_output",
                ..
            }))
        ));
        plan.update_flex_positions(&mut data).unwrap();
        plan.update_flex_faces(&mut data).unwrap();
        data.readback().unwrap();
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn flex_position_readback_rejects_guards_and_nonfinite_payloads() {
        let session = TransferSession::new(0).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0, 1],
                flex_cellnum: vec![1; 6],
                flex_nodeadr: vec![0, 0],
                flex_nodenum: vec![0, 8],
                flex_vertadr: vec![0, 1],
                flex_vertnum: vec![1, 1],
                flex_centered: vec![false, true],
                flex_nodebodyid: vec![0; 8],
                flex_vertbodyid: vec![0, -1],
                flex_node: vec![0.0; 24],
                flex_vert: vec![0.2, 0.3, 0.4, 0.0, 0.0, 0.0],
                flex_vert0: vec![0.5; 6],
            },
        )
        .unwrap();
        let plan = KinematicsPlan::with_flex_positions(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let len = data.layout.flex.unwrap().guarded;
        for index in [0, 3, len - 4, len - 1] {
            data.flex
                .as_mut()
                .unwrap()
                .write_range(index, &[0.0])
                .unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch { index:i,.. })) if i==index)
            );
            data.flex
                .as_mut()
                .unwrap()
                .write_range(index, &[-131072.0])
                .unwrap();
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for index in [
                4,
                4 + 24,
                4 + 29,
                4 + 512 * 30,
                4 + 512 * 30 + 24,
                4 + 512 * 30 + 29,
            ] {
                data.flex
                    .as_mut()
                    .unwrap()
                    .write_range(index, &[bad])
                    .unwrap();
                assert!(
                    matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite {field:"resident_flex_position_output",index:i})) if i==index-4)
                );
                plan.update_flex_positions(&mut data).unwrap();
            }
        }
        let out = data.readback().unwrap();
        assert_eq!(
            out.flex_positions()
                .unwrap()
                .world(512)
                .unwrap()
                .flexvert_xpos,
            &[0.2, 0.3, 0.4, 0.0, 0.0, 0.0]
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn tendon_wake_readback_rejects_guards_with_static_flex_and_zero_trees() {
        let session = TransferSession::new(0).unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let flex = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![1],
                flex_centered: vec![false],
                flex_vertbodyid: vec![0],
                flex_vert: vec![0.2, 0.3, 0.4],
                flex_vert0: vec![0.5; 3],
                ..Default::default()
            },
        )
        .unwrap();
        let fields = crate::model::TendonWakeFields {
            body_treeid: vec![-1],
            tendon_limited: vec![],
            tendon_range: crate::model::ParameterBatch::new(3, 0, vec![]).unwrap(),
            tendon_margin: crate::model::ParameterBatch::new(2, 0, vec![]).unwrap(),
            sleep_enabled: true,
            island_disabled: false,
        };
        let input = TendonWakeModelInput::with_flex_positions(flex, fields).unwrap();
        let plan = KinematicsPlan::with_tendon_wake(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let len = data.layout.sleep.unwrap().guarded;
        for index in [0, 3, len - 4, len - 1] {
            data.sleep
                .as_mut()
                .unwrap()
                .state
                .write_range(index, &[0])
                .unwrap();
            assert!(
                matches!(data.readback(), Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i == index)
            );
            data.sleep
                .as_mut()
                .unwrap()
                .state
                .write_range(index, &[-131072])
                .unwrap();
        }
        let out = data.readback().unwrap();
        assert_eq!(
            out.flex_positions()
                .unwrap()
                .world(512)
                .unwrap()
                .flexvert_xpos,
            [0.2, 0.3, 0.4]
        );
        let row = out.sleep_trees().unwrap().world(512).unwrap();
        assert!(row.tree_asleep.is_empty() && row.tree_awake.is_empty());
        assert_eq!((row.ntree_awake, row.nbody_awake, row.nv_awake), (0, 1, 0));
        assert!(out.sleep_trees().unwrap().world(513).is_err());
        assert!(plan.create_data(0).is_err());
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "需要NVIDIA驱动与NVRTC"]
    fn spatial_tendon_readback_rejects_overflow_and_recovers_from_coincident_sites() {
        let session = TransferSession::new(0).unwrap();
        let plan = spatial_tendon_plan(&session);
        let mut data = plan.create_data(513).unwrap();
        data.write_world_qpos(512, &[-1.0]).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let row = out.spatial_tendon().world(512).unwrap();
        assert_eq!(row.ten_length, &[0.0]);
        assert_eq!(row.ten_jacobian, &[1.0]);
        // Isolate spatial norm overflow without overflowing COM inertia first.
        let position = 4 + 512 * 36 + 12 + 3;
        data.attached.write_range(position, &[1e20]).unwrap();
        plan.update_spatial_tendons(&mut data).unwrap();
        let error = data.readback().expect_err("spatial norm must overflow");
        let start = data
            .layout
            .spatial_tendon
            .output
            .world_elements(512)
            .unwrap()
            .start;
        assert!(
            matches!(
                error,
                TransferError::Input(InputError::NonFinite {
                    field: "resident_spatial_tendon_output",
                    index
                }) if index == start
            ),
            "{error:?}"
        );
        data.write_world_qpos(512, &[2.0]).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let row = out.spatial_tendon().world(512).unwrap();
        assert_eq!(row.ten_length, &[3.0]);
        assert_eq!(row.ten_jacobian, &[1.0]);
    }
}
