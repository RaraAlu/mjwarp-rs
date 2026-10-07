//! G01严格子集的常驻设备链。
//! 不提供冻结上游等价阶段。

use std::sync::Arc;

use super::attached::AttachedLayout;
use super::{
    AttachedKinematicsOutput, ComPositionLayout, ComPositionOutput, KinematicsLayout,
    KinematicsOutput, check_com_position, check_kinematics,
};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{
    AttachedModelInput, BatchLayout, KinematicsParameter, KinematicsParameters, MocapModelInput,
};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 一次上传并编译的运动学计划。
/// 一个计划可以创建多组独立状态。
/// 每项浮点字段独立按世界取模。
/// 模型拓扑仍只共享一份。
/// 默认状态使用检查后的qpos0。
/// 保留探针的严格输入限制。
/// 支持mocap与静态geom缓存。
/// 不处理相机或灯光。
/// 不冻结正式GPU编译路线。
pub struct KinematicsPlan {
    model: Arc<AttachedModelInput>,
    parameters: KinematicsParameters,
    body_mocapid: Vec<i32>,
    nmocap: usize,
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
    model: Arc<AttachedModelInput>,
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
}

/// 一次显式回读的完整子集结果。
/// 它独立拥有全部宿主缓冲。
/// 后续状态更新不修改此快照。
#[derive(Clone, Debug)]
pub struct KinematicsSnapshot {
    attached: AttachedKinematicsOutput,
    com: ComPositionOutput,
}

impl KinematicsSnapshot {
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
        let (model, body_mocapid, nmocap, static_geom) = model.into_parts();
        let (fk, com) =
            check_com_position(model.rigid(), 1, &model.rigid().kinematics().fields().qpos0)?;
        let attached = AttachedLayout::new(1, model.ngeom(), model.nsite())?;
        check_parameters(&model, &parameters)?;
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
            );
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let _ = (com, attached);
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
            let rigid = rigid.upload(session)?;
            let com = com.upload(session)?;
            let attached = attached.upload(session)?;
            let prefix = format!("{FIELD_BATCH_CUDA}\n#define MJWARP_MOCAP\n");
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
            Ok(Self {
                model: Arc::new(model),
                parameters,
                body_mocapid,
                nmocap,
                session: session.clone(),
                rigid,
                com,
                attached,
                rigid_kernel,
                com_kernel,
                attached_kernel,
                initialize_kernel,
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

    /// 创建独立世界与三个输出缓冲。
    /// 全部尺寸通过后才申请显存。
    /// 初始派生结果保持未就绪。
    pub fn create_data(&self, worlds: usize) -> Result<KinematicsData, TransferError> {
        let layout = ResidentLayout::new(&self.model, worlds, self.nmocap)?;
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
                model: Arc::clone(&self.model),
                layout,
                ready: ReadyStages::default(),
                state: self.session.upload(&state)?,
                rigid: allocate_output(&self.session, layout.rigid.guarded)?,
                com: allocate_output(&self.session, layout.com.guarded)?,
                attached: allocate_output(&self.session, layout.attached.guarded)?,
            };
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

    fn check_data(&self, data: &KinematicsData) -> Result<(), TransferError> {
        if Arc::ptr_eq(&self.model, &data.model) {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }

    /// 只更新七项刚体结果。
    /// 它废弃质心与附着结果。
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
    /// 本严格辅助要求刚体就绪。
    pub fn update_com(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready.require(data.ready.rigid, "rigid")?;
        data.ready.com = false;
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
        data.ready.require(data.ready.rigid, "rigid")?;
        data.ready.attached = false;
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

    /// 依次执行三个子集阶段。
    /// 不上传模型或编译内核。
    /// 不回读阶段间的宿主结果。
    /// 不等于上游fwd_kinematics。
    pub fn update(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.update_rigid(data)?;
        self.update_attached(data)?;
        self.update_com(data)
    }
}

impl KinematicsData {
    pub fn worlds(&self) -> usize {
        self.layout.qpos.worlds()
    }

    /// 完整检查后改写全部qpos。
    /// 输入错误保留已有设备结果。
    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        check_kinematics(self.model.rigid(), self.worlds(), qpos)?;
        self.write_checked_qpos(0, qpos)
    }

    /// 只改写指定世界的qpos。
    /// 它废弃整组派生结果。
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qpos.world_elements(world)?;
        check_kinematics(self.model.rigid(), 1, qpos)?;
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
        check_mocap(
            self.layout.mocap_pos.total_elements(),
            self.layout.mocap_quat.total_elements(),
            pos,
            quat,
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
        check_mocap(p.len(), q.len(), pos, quat)?;
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
        let lengths = [
            self.layout.rigid.output.total_elements() + 8,
            self.layout.com.output.total_elements() + 8,
            self.layout.attached.output.total_elements() + 8,
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
            Ok(KinematicsSnapshot {
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
            })
        }
    }
}

fn check_mocap(np: usize, nq: usize, pos: &[f32], quat: &[f32]) -> Result<(), InputError> {
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
        if !(1e-12..=1e12).contains(&squared) {
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
impl PackedModel {
    fn upload(self, session: &TransferSession) -> Result<super::DeviceModel<f32>, TransferError> {
        Ok(super::DeviceModel {
            metadata: session.upload(&self.metadata)?,
            parameters: session.upload(&self.parameters)?,
        })
    }
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
fn read_output(
    buffer: &TransferBuffer<f32>,
    length: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(length)?;
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
}
