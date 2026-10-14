//! 共享被动力累加链。
//! 本模块不代表完整被动力阶段。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::model::SmoothForceInput;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::SmoothForceOptions;
use super::{
    FlexEdgePassivePlan, JointPassiveData, JointPassivePlan, KinematicsData, SmoothForceData,
    SmoothForceField, SmoothForceOutput, SmoothForcePlan, TendonPassivePlan, TendonVelocityView,
};

pub struct PassiveForcePlan<'a, 'v, 'p, 'f> {
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    joint: &'a JointPassivePlan,
    tendon: &'a TendonPassivePlan<'v, 'p>,
    flex_edge: &'a FlexEdgePassivePlan<'f>,
    identity: Arc<()>,
    smooth: SmoothForcePlan,
    nv: usize,
    disable_spring: bool,
    disable_damper: bool,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    dummy: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}

pub struct PassiveForceData {
    identity: Arc<()>,
    layout: BatchLayout,
    ready: bool,
    smooth: SmoothForceData,
    #[cfg(feature = "cuda-probe")]
    parts: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct PassiveForceView<'a, 's, 'p> {
    data: &'a PassiveForceData,
    _joint: &'a JointPassiveData,
    _tendon_velocity: &'a TendonVelocityView<'s, 'p>,
    _flex_source: &'a KinematicsData,
}

pub struct ClearedPassiveForceView<'a> {
    data: &'a PassiveForceData,
}

pub struct SleepClearedPassiveForceView<'a> {
    data: &'a PassiveForceData,
    _sleep_source: &'a KinematicsData,
}

#[derive(Clone, Debug)]
pub struct PassiveForceSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl PassiveForceSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn spring(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }

    pub fn damper(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        let offset = 4 + self.layout.total_elements();
        Ok(&self.values[offset + range.start..offset + range.end])
    }
}

impl<'a, 'v, 'p, 'f> PassiveForcePlan<'a, 'v, 'p, 'f> {
    pub fn new(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
    ) -> Result<Self, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (joint, tendon, flex_edge);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let (session, smooth_input, nv, joint_options) = joint.shared_passive_context();
            let (tendon_session, tendon_nv, tendon_options) = tendon.shared_passive_context();
            let (flex_session, flex_nv, flex_options) = flex_edge.shared_passive_context();
            let same_options = joint_options.disable_spring == tendon_options.disable_spring
                && joint_options.disable_spring == flex_options.disable_spring
                && joint_options.disable_damper == tendon_options.disable_damper
                && joint_options.disable_damper == flex_options.disable_damper;
            if nv != tendon_nv
                || nv != flex_nv
                || !same_options
                || !flex_edge.matches_shared_model(joint.shared_passive_model())
                || !session.same_session(&tendon_session)
                || !session.same_session(&flex_session)
            {
                return Err(TransferError::ModelMismatch);
            }
            let (smooth_input, smooth_options) =
                if let Some((ntree, body_treeid, sleep_enabled, island_disabled)) =
                    flex_edge.shared_sleep_context()
                {
                    (
                        SmoothForceInput::new(
                            ntree,
                            body_treeid.to_vec(),
                            smooth_input.dof_bodyid().to_vec(),
                        )?,
                        SmoothForceOptions {
                            sleep_enabled,
                            island_disabled,
                        },
                    )
                } else {
                    (smooth_input, SmoothForceOptions::default())
                };
            let smooth = SmoothForcePlan::new(&session, smooth_input, smooth_options)?;
            let metadata = session.upload(&[-131072; 8])?;
            let dummy = session.upload(&[-131072.0; 8])?;
            let code = format!("#define PF_NV {nv}\n{CUDA}");
            Ok(Self {
                joint,
                tendon,
                flex_edge,
                identity: Arc::new(()),
                smooth,
                nv,
                disable_spring: joint_options.disable_spring,
                disable_damper: joint_options.disable_damper,
                metadata,
                dummy,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "passive_force_clear")?,
                    SynchronousKernel::compile(&session, &code, "passive_force_sum")?,
                ],
                session,
            })
        }
    }

    pub fn create_data(&self, worlds: usize) -> Result<PassiveForceData, TransferError> {
        let layout = BatchLayout::new(worlds, self.nv, 4)?;
        let count = layout
            .total_elements()
            .checked_mul(2)
            .ok_or(InputError::Overflow {
                field: "passive_force_parts",
            })?;
        let smooth = self.smooth.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (count, smooth);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, value: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(value);
                self.session.upload(&values)
            };
            Ok(PassiveForceData {
                identity: self.identity.clone(),
                layout,
                ready: false,
                smooth,
                parts: allocate(count, f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    /// 调用方须同步全部qpos与qvel副本。
    /// 本入口拒绝不同常驻源与不同世界数。
    /// 启用休眠时借用同一常驻源。
    pub fn update<'b, 's>(
        &'b self,
        joint: &'b JointPassiveData,
        tendon_velocity: &'b TendonVelocityView<'s, 'p>,
        flex_source: &'b KinematicsData,
        data: &'b mut PassiveForceData,
    ) -> Result<PassiveForceView<'b, 's, 'p>, TransferError> {
        if self.disable_spring && self.disable_damper {
            return Err(InputError::InvalidDimension {
                field: "passive_force_disabled_path",
            }
            .into());
        }
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || data.worlds() != joint.worlds()
            || data.worlds() != tendon_velocity.worlds()
            || data.worlds() != flex_source.worlds()
            || !self.tendon.accepts_shared_velocity(tendon_velocity)
            || !self.flex_edge.accepts_shared_source(flex_source)
            || !std::ptr::eq(tendon_velocity.source_data(), flex_source)
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = joint;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            let work = worlds.checked_mul(self.nv).ok_or(InputError::Overflow {
                field: "passive_force_work",
            })? as u32;
            self.joint.write_shared_passive_base(
                joint,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            self.tendon.accumulate_shared_passive(
                tendon_velocity,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            self.flex_edge.accumulate_shared_passive(
                flex_source,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            let sleep = if self.smooth.options().enable_sleep() {
                Some(self.flex_edge.shared_sleep_source(flex_source)?)
            } else {
                None
            };
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            self.finish(data, worlds, work, sleep)?;
            data.ready = true;
            Ok(PassiveForceView {
                data,
                _joint: joint,
                _tendon_velocity: tendon_velocity,
                _flex_source: flex_source,
            })
        }
    }

    /// 双禁用时仅清零共享分量。
    /// 本入口不读取任何生产者。
    pub fn clear_disabled<'b>(
        &'b self,
        data: &'b mut PassiveForceData,
    ) -> Result<ClearedPassiveForceView<'b>, TransferError> {
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || !self.disable_spring
            || !self.disable_damper
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            let work = worlds.checked_mul(self.nv).ok_or(InputError::Overflow {
                field: "passive_force_work",
            })? as u32;
            // SAFETY: 每线程清零一个分量槽。
            // 本分支不读取任何生产者设备缓冲。
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &self.dummy,
                    &self.dummy,
                    &mut data.parts,
                    [0, self.nv as u32, worlds as u32, work],
                )?;
            }
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            self.write_passive(data, worlds, work)?;
            if !self.smooth.options().enable_sleep() {
                self.smooth.update(&mut data.smooth)?;
            }
            data.ready = true;
            Ok(ClearedPassiveForceView { data })
        }
    }

    /// 双禁用时只借用树活动状态。
    /// 本入口不读取物理生产者。
    pub fn clear_disabled_with_sleep<'b>(
        &'b self,
        sleep_source: &'b KinematicsData,
        data: &'b mut PassiveForceData,
    ) -> Result<SleepClearedPassiveForceView<'b>, TransferError> {
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || !self.disable_spring
            || !self.disable_damper
            || !self.smooth.options().enable_sleep()
            || data.worlds() != sleep_source.worlds()
            || !self.flex_edge.accepts_shared_source(sleep_source)
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            let work = worlds.checked_mul(self.nv).ok_or(InputError::Overflow {
                field: "passive_force_work",
            })? as u32;
            // SAFETY: 每线程清零一个分量槽。
            // 本分支不读取任何物理生产者。
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &self.dummy,
                    &self.dummy,
                    &mut data.parts,
                    [0, self.nv as u32, worlds as u32, work],
                )?;
            }
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            let sleep = self.flex_edge.shared_sleep_source(sleep_source)?;
            self.finish(data, worlds, work, Some(sleep))?;
            data.ready = true;
            Ok(SleepClearedPassiveForceView {
                data,
                _sleep_source: sleep_source,
            })
        }
    }

    #[cfg(feature = "cuda-probe")]
    fn write_passive(
        &self,
        data: &mut PassiveForceData,
        worlds: usize,
        work: u32,
    ) -> Result<(), TransferError> {
        let passive = data.smooth.passive_target();
        // SAFETY: 分量缓冲含两个连续字段。
        // 目标仅写入平滑力的Passive字段。
        unsafe {
            self.kernels[1].launch(
                &self.metadata,
                &self.dummy,
                &data.parts,
                passive,
                [0, self.nv as u32, worlds as u32, work],
            )?;
        }
        Ok(())
    }

    #[cfg(feature = "cuda-probe")]
    fn finish(
        &self,
        data: &mut PassiveForceData,
        worlds: usize,
        work: u32,
        sleep: Option<&TransferBuffer<i32>>,
    ) -> Result<(), TransferError> {
        self.write_passive(data, worlds, work)?;
        self.smooth
            .update_with_sleep_source(&mut data.smooth, sleep)
    }
}

impl PassiveForceData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn write_other_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        if field == SmoothForceField::Passive {
            return Err(InputError::InvalidDimension {
                field: "shared_passive_owned",
            }
            .into());
        }
        self.ready = false;
        self.smooth.write_force(field, values)
    }

    fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "passive_force",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(PassiveForceSnapshot {
                layout: self.layout,
                values: read_parts(&self.parts, 2 * self.layout.total_elements())?,
            })
        }
    }

    fn smooth_output(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "passive_force",
            });
        }
        self.smooth.readback()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    fn corrupt_parts_guard_for_test(
        &mut self,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        self.parts.write_range(index, &[value])
    }
}

impl PassiveForceView<'_, '_, '_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

impl ClearedPassiveForceView<'_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

impl SleepClearedPassiveForceView<'_> {
    pub fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
        self.data.snapshot()
    }

    pub fn smooth(&self) -> Result<SmoothForceOutput, TransferError> {
        self.data.smooth_output()
    }
}

#[cfg(feature = "cuda-probe")]
fn read_parts(buffer: &TransferBuffer<f32>, count: usize) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, "passive_force_parts")?;
    Ok(values)
}

// 冻结顺序：passive.py::_spring_damper。
// 基线：71da24d，许可：Apache-2.0。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void passive_force_clear(const int* meta,const float* p,const float* state,float* out,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;out[4+(unsigned long long)work+tid]=0.0f;
}
extern "C" __global__ void passive_force_sum(const int* meta,const float* p,const float* parts,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);
}
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexEdgeFields,
        FlexPositionFields, FlexPositionModelInput, InertialFields, InertialModelInput,
        KinematicFields, KinematicModelInput, MocapModelInput, ParameterBatch, SleepTreeState,
        TendonFields, TendonModelInput, TendonWakeFields, TendonWakeModelInput,
    };
    use crate::model::{CamLightParameters, KinematicsParameters};
    use crate::physics::{
        FlexEdgePassiveInput, FlexEdgePassiveOptions, JointPassiveOptions, JointPassiveParameters,
        TendonPassiveOptions, TendonPassiveParameters, TendonVelocityPlan,
    };

    fn kinematic(axis: [f32; 3]) -> KinematicModelInput {
        KinematicModelInput::new(
            1,
            KinematicFields {
                qpos0: vec![0.0],
                body_parentid: vec![0, 0],
                body_jntadr: vec![-1, 0],
                body_jntnum: vec![0, 1],
                body_pos: vec![0.0; 6],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                jnt_type: vec![2],
                jnt_bodyid: vec![1],
                jnt_qposadr: vec![0],
                jnt_dofadr: vec![0],
                jnt_pos: vec![0.0; 3],
                jnt_axis: axis.to_vec(),
            },
        )
        .unwrap()
    }

    fn flex_model(model: KinematicModelInput) -> FlexPositionModelInput {
        let inertial = InertialModelInput::new(
            model,
            InertialFields {
                body_ipos: vec![0.0; 6],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                body_mass: vec![0.0, 1.0],
                body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                dof_bodyid: vec![1],
                dof_jntid: vec![0],
                dof_parentid: vec![-1],
                dof_armature: vec![0.0],
                dof_damping: vec![0.0],
            },
        )
        .unwrap();
        let attached = AttachedModelInput::new(inertial, AttachedFields::default()).unwrap();
        let camlight = CamLightModelInput::new(
            MocapModelInput::new(attached, vec![-1; 2]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let tendons = TendonModelInput::new(
            camlight,
            TendonFields {
                tendon_adr: vec![0],
                tendon_num: vec![1],
                wrap_type: vec![1],
                wrap_objid: vec![0],
                wrap_prm: vec![1.0],
                ten_j_rowadr: vec![0],
                ten_j_rownnz: vec![1],
                ten_j_colind: vec![0],
            },
        )
        .unwrap();
        FlexPositionModelInput::new(
            tendons,
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![2],
                flex_centered: vec![false],
                flex_nodebodyid: vec![],
                flex_vertbodyid: vec![0, 1],
                flex_node: vec![],
                flex_vert: vec![0.0; 6],
                flex_vert0: vec![0.0; 6],
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
        .unwrap()
    }

    fn sleep_kinematic() -> KinematicModelInput {
        KinematicModelInput::new(
            2,
            KinematicFields {
                qpos0: vec![0.0; 2],
                body_parentid: vec![0, 0, 0],
                body_jntadr: vec![-1, 0, 1],
                body_jntnum: vec![0, 1, 1],
                body_pos: vec![0.0; 9],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                jnt_type: vec![2, 2],
                jnt_bodyid: vec![1, 2],
                jnt_qposadr: vec![0, 1],
                jnt_dofadr: vec![0, 1],
                jnt_pos: vec![0.0; 6],
                jnt_axis: [1.0, 0.0, 0.0].repeat(2),
            },
        )
        .unwrap()
    }

    fn sleep_flex_model(model: KinematicModelInput) -> FlexPositionModelInput {
        let inertial = InertialModelInput::new(
            model,
            InertialFields {
                body_ipos: vec![0.0; 9],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                body_mass: vec![0.0, 1.0, 1.0],
                body_inertia: vec![0.0; 9],
                dof_bodyid: vec![1, 2],
                dof_jntid: vec![0, 1],
                dof_parentid: vec![-1, -1],
                dof_armature: vec![0.0; 2],
                dof_damping: vec![0.0; 2],
            },
        )
        .unwrap();
        let attached = AttachedModelInput::new(inertial, AttachedFields::default()).unwrap();
        let camlight = CamLightModelInput::new(
            MocapModelInput::new(attached, vec![-1; 3]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let tendons = TendonModelInput::new(
            camlight,
            TendonFields {
                tendon_adr: vec![0],
                tendon_num: vec![1],
                wrap_type: vec![1],
                wrap_objid: vec![0],
                wrap_prm: vec![1.0],
                ten_j_rowadr: vec![0],
                ten_j_rownnz: vec![1],
                ten_j_colind: vec![0],
            },
        )
        .unwrap();
        FlexPositionModelInput::new(
            tendons,
            FlexPositionFields {
                flex_interp: vec![0],
                flex_cellnum: vec![1; 3],
                flex_nodeadr: vec![0],
                flex_nodenum: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![2],
                flex_centered: vec![false],
                flex_nodebodyid: vec![],
                flex_vertbodyid: vec![1, 2],
                flex_node: vec![],
                flex_vert: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                flex_vert0: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            },
        )
        .unwrap()
        .with_edges(FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
            flex_edge: vec![0, 1],
            flexedge_j_rowadr: vec![0],
            flexedge_j_rownnz: vec![2],
            flexedge_j_colind: vec![0, 1],
        })
        .unwrap()
    }

    fn sleep_joint_parameters() -> JointPassiveParameters {
        JointPassiveParameters {
            qpos_spring: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
            stiffness: ParameterBatch::new(1, 2, vec![-2.0, -3.0]).unwrap(),
            stiffness_poly: ParameterBatch::new(1, 4, vec![0.0; 4]).unwrap(),
            damping: ParameterBatch::new(1, 2, vec![0.5, 0.75]).unwrap(),
            damping_poly: ParameterBatch::new(1, 4, vec![0.0; 4]).unwrap(),
        }
    }

    fn rows(period: usize, width: usize, mut row: impl FnMut(usize) -> Vec<f32>) -> ParameterBatch {
        let mut values = Vec::with_capacity(period * width);
        for index in 0..period {
            let row = row(index);
            assert_eq!(row.len(), width);
            values.extend(row);
        }
        ParameterBatch::new(period, width, values).unwrap()
    }

    fn joint_parameters() -> JointPassiveParameters {
        JointPassiveParameters {
            qpos_spring: rows(2, 1, |row| vec![row as f32 / 4.0]),
            stiffness: rows(3, 1, |row| {
                vec![if row == 0 {
                    -16_777_216.0
                } else {
                    -(8 << row) as f32
                }]
            }),
            stiffness_poly: rows(5, 2, |row| vec![0.0, row as f32 / 32.0]),
            damping: rows(7, 1, |row| vec![row as f32 / 16.0]),
            damping_poly: rows(11, 2, |row| vec![row as f32 / 64.0, 0.0]),
        }
    }

    fn tendon_parameters() -> TendonPassiveParameters {
        TendonPassiveParameters {
            stiffness: rows(2, 1, |row| vec![-1.0 - row as f32]),
            stiffness_poly: rows(3, 2, |row| vec![row as f32 / 32.0, 0.0]),
            damping: rows(5, 1, |row| vec![row as f32 / 16.0]),
            damping_poly: rows(7, 2, |row| vec![row as f32 / 64.0, 0.0]),
            length_spring: rows(11, 2, |row| vec![0.0, row as f32 / 64.0]),
        }
    }

    fn ordered_atomic_sources(mut value: f32, tendon: f32, flex_edge: f32) -> f32 {
        if tendon != 0.0 {
            value += tendon;
        }
        if flex_edge != 0.0 {
            value += flex_edge;
        }
        value
    }

    #[test]
    #[ignore = "new G04 ordered shared accumulation and f32 grouping certificate"]
    fn gpu_accumulates_joint_tendon_flexedge_in_frozen_order() {
        let timer = std::time::Instant::now();
        let session = TransferSession::new(0).unwrap();
        let model = kinematic([1.0, 0.0, 0.0]);
        let source_plan = super::super::KinematicsPlan::with_flex_positions(
            &session,
            flex_model(model.clone()),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let joint_plan = JointPassivePlan::new(
            &session,
            model.clone(),
            joint_parameters(),
            JointPassiveOptions::default(),
        )
        .unwrap();
        let tendon_plan = TendonPassivePlan::new(
            &velocity_plan,
            tendon_parameters(),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let flex_plan = FlexEdgePassivePlan::new(
            &source_plan,
            FlexEdgePassiveInput {
                length0: vec![0.0],
                stiffness: vec![-1.0],
                damping: vec![0.5],
                rigid: vec![false],
            },
            FlexEdgePassiveOptions::default(),
        )
        .unwrap();
        let plan = PassiveForcePlan::new(&joint_plan, &tendon_plan, &flex_plan).unwrap();
        let worlds = 11;
        let mut source = source_plan.create_data(worlds).unwrap();
        let mut velocity = velocity_plan.create_data(worlds).unwrap();
        let mut joint = joint_plan.create_data(worlds).unwrap();
        let mut tendon = tendon_plan.create_data(worlds).unwrap();
        let mut flex = flex_plan.create_data(worlds).unwrap();
        let mut data = plan.create_data(worlds).unwrap();
        let zero = vec![0.0; worlds];
        source.write_qpos(&vec![1.0; worlds]).unwrap();
        source.write_qvel(&zero).unwrap();
        source_plan.update(&mut source).unwrap();
        velocity.write_qvel(&zero).unwrap();
        joint.write_qpos(&vec![1.0; worlds]).unwrap();
        joint.write_qvel(&zero).unwrap();
        joint_plan.update_joint(&mut joint).unwrap();
        let joint_snapshot = joint.readback_joint().unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let tendon_snapshot = tendon_plan
            .update(&velocity_view, &mut tendon)
            .unwrap()
            .snapshot()
            .unwrap();
        let flex_snapshot = flex_plan
            .update(&source, &mut flex)
            .unwrap()
            .snapshot()
            .unwrap();
        let combined = plan
            .update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        let snapshot = combined.snapshot().unwrap();
        let smooth = combined.smooth().unwrap();
        for world in 0..worlds {
            let spring = ordered_atomic_sources(
                joint_snapshot.spring(world).unwrap()[0],
                tendon_snapshot.spring(world).unwrap()[0],
                flex_snapshot.spring(world).unwrap()[0],
            );
            let damper = ordered_atomic_sources(
                joint_snapshot.damper(world).unwrap()[0],
                tendon_snapshot.damper(world).unwrap()[0],
                flex_snapshot.damper(world).unwrap()[0],
            );
            assert_eq!(
                snapshot.spring(world).unwrap()[0].to_bits(),
                spring.to_bits()
            );
            assert_eq!(
                snapshot.damper(world).unwrap()[0].to_bits(),
                damper.to_bits()
            );
            assert_eq!(
                smooth.world(world).unwrap()[0].to_bits(),
                (spring + damper).to_bits()
            );
        }
        assert_eq!(
            joint_snapshot.spring(0).unwrap()[0].to_bits(),
            16_777_216.0f32.to_bits()
        );
        assert_eq!(
            tendon_snapshot.spring(0).unwrap()[0].to_bits(),
            1.0f32.to_bits()
        );
        assert_eq!(
            flex_snapshot.spring(0).unwrap()[0].to_bits(),
            1.0f32.to_bits()
        );
        assert_eq!(
            snapshot.spring(0).unwrap()[0].to_bits(),
            16_777_216.0f32.to_bits()
        );
        assert_eq!(
            (joint_snapshot.spring(0).unwrap()[0]
                + (tendon_snapshot.spring(0).unwrap()[0] + flex_snapshot.spring(0).unwrap()[0]))
                .to_bits(),
            16_777_218.0f32.to_bits()
        );
        drop(combined);
        data.corrupt_parts_guard_for_test(0, 99.0).unwrap();
        assert!(
            plan.update(&joint, &velocity_view, &source, &mut data)
                .is_err()
        );
        assert!(data.snapshot().is_err());
        data.corrupt_parts_guard_for_test(0, -131072.0).unwrap();
        plan.update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        drop(velocity_view);

        let moving = (0..worlds)
            .map(|world| (world + 1) as f32 / 16.0)
            .collect::<Vec<_>>();
        source.write_qpos(&vec![2.0; worlds]).unwrap();
        source.write_qvel(&moving).unwrap();
        source_plan.update(&mut source).unwrap();
        velocity.write_qvel(&moving).unwrap();
        joint.write_qpos(&vec![2.0; worlds]).unwrap();
        joint.write_qvel(&moving).unwrap();
        joint_plan.update_joint(&mut joint).unwrap();
        let fresh_joint = joint.readback_joint().unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let fresh_tendon = tendon_plan
            .update(&velocity_view, &mut tendon)
            .unwrap()
            .snapshot()
            .unwrap();
        let fresh_flex = flex_plan
            .update(&source, &mut flex)
            .unwrap()
            .snapshot()
            .unwrap();
        let fresh_view = plan
            .update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        let fresh = fresh_view.snapshot().unwrap();
        let fresh_smooth = fresh_view.smooth().unwrap();
        for world in 0..worlds {
            let spring = ordered_atomic_sources(
                fresh_joint.spring(world).unwrap()[0],
                fresh_tendon.spring(world).unwrap()[0],
                fresh_flex.spring(world).unwrap()[0],
            );
            let damper = ordered_atomic_sources(
                fresh_joint.damper(world).unwrap()[0],
                fresh_tendon.damper(world).unwrap()[0],
                fresh_flex.damper(world).unwrap()[0],
            );
            assert_eq!(fresh.spring(world).unwrap()[0].to_bits(), spring.to_bits());
            assert_eq!(fresh.damper(world).unwrap()[0].to_bits(), damper.to_bits());
            assert_eq!(
                fresh_smooth.world(world).unwrap()[0].to_bits(),
                (spring + damper).to_bits()
            );
        }
        assert!(
            fresh_flex
                .damper(0)
                .unwrap()
                .iter()
                .any(|value| *value != 0.0)
        );
        drop(fresh_view);

        let foreign_model = kinematic([0.0, 1.0, 0.0]);
        let foreign_joint = JointPassivePlan::new(
            &session,
            foreign_model,
            joint_parameters(),
            JointPassiveOptions::default(),
        )
        .unwrap();
        assert!(PassiveForcePlan::new(&foreign_joint, &tendon_plan, &flex_plan).is_err());
        let foreign_session = TransferSession::new(0).unwrap();
        let foreign_joint = JointPassivePlan::new(
            &foreign_session,
            model.clone(),
            joint_parameters(),
            JointPassiveOptions::default(),
        )
        .unwrap();
        assert!(PassiveForcePlan::new(&foreign_joint, &tendon_plan, &flex_plan).is_err());

        let disabled = JointPassiveOptions {
            disable_spring: true,
            disable_damper: true,
        };
        let disabled_joint =
            JointPassivePlan::new(&session, model, joint_parameters(), disabled).unwrap();
        let disabled_tendon = TendonPassivePlan::new(
            &velocity_plan,
            tendon_parameters(),
            TendonPassiveOptions {
                disable_spring: true,
                disable_damper: true,
            },
        )
        .unwrap();
        let disabled_flex = FlexEdgePassivePlan::new(
            &source_plan,
            FlexEdgePassiveInput {
                length0: vec![f32::NAN],
                stiffness: vec![f32::NAN],
                damping: vec![f32::NAN],
                rigid: vec![false],
            },
            FlexEdgePassiveOptions {
                disable_spring: true,
                disable_damper: true,
            },
        )
        .unwrap();
        let disabled_plan =
            PassiveForcePlan::new(&disabled_joint, &disabled_tendon, &disabled_flex).unwrap();
        let mut disabled_data = disabled_plan.create_data(3).unwrap();
        let cleared = disabled_plan.clear_disabled(&mut disabled_data).unwrap();
        for world in 0..3 {
            assert_eq!(cleared.snapshot().unwrap().spring(world).unwrap(), [0.0]);
            assert_eq!(cleared.snapshot().unwrap().damper(world).unwrap(), [0.0]);
            assert_eq!(cleared.smooth().unwrap().world(world).unwrap(), [0.0]);
        }
        let report = serde_json::json!({
            "worlds": worlds,
            "compared_slots": 2 * worlds * 3,
            "ordered_sources": ["joint_write", "tendon_atomic_add", "flex_edge_atomic_add"],
            "f32_grouping_certificate": true,
            "repeat_update_without_residue": true,
            "partial_publication_rejected": true,
            "dual_disabled_cold_clear": true,
            "sleep_filtering": false,
            "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
            "passive_force_owned_bytes": 4 * (plan.metadata.len() + plan.dummy.len() + data.parts.len() + data.status.len()),
            "performance_acceptance": false,
        });
        if let Some(path) = std::env::var_os("MJWARP_PASSIVE_FORCE_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }

    #[test]
    #[ignore = "new G04 resident sleep filtering and mid-chain recovery"]
    fn gpu_filters_resident_sleep_and_recovers_after_midchain_failure() {
        let timer = std::time::Instant::now();
        let session = TransferSession::new(0).unwrap();
        let model = sleep_kinematic();
        let wake = TendonWakeModelInput::with_flex_positions(
            sleep_flex_model(model.clone()),
            TendonWakeFields {
                body_treeid: vec![-1, 0, 1],
                tendon_limited: vec![false],
                tendon_range: ParameterBatch::new(1, 2, vec![-10.0, 10.0]).unwrap(),
                tendon_margin: ParameterBatch::new(1, 1, vec![0.0]).unwrap(),
                sleep_enabled: true,
                island_disabled: false,
            },
        )
        .unwrap();
        let source_plan = super::super::KinematicsPlan::with_tendon_wake(
            &session,
            wake,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let joint_plan = JointPassivePlan::new(
            &session,
            model.clone(),
            sleep_joint_parameters(),
            JointPassiveOptions::default(),
        )
        .unwrap();
        let tendon_plan = TendonPassivePlan::new(
            &velocity_plan,
            tendon_parameters(),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let flex_plan = FlexEdgePassivePlan::new(
            &source_plan,
            FlexEdgePassiveInput {
                length0: vec![0.5],
                stiffness: vec![-1.0],
                damping: vec![0.25],
                rigid: vec![false],
            },
            FlexEdgePassiveOptions::default(),
        )
        .unwrap();
        let plan = PassiveForcePlan::new(&joint_plan, &tendon_plan, &flex_plan).unwrap();
        assert!(plan.smooth.options().enable_sleep());
        assert_eq!(plan.smooth.input().ntree(), 2);
        let worlds = 2;
        let qpos = vec![0.25, -0.5, 0.75, -1.0];
        let qvel = vec![0.125, -0.25, 0.375, -0.5];
        let mut source = source_plan.create_data(worlds).unwrap();
        let mut velocity = velocity_plan.create_data(worlds).unwrap();
        let mut joint = joint_plan.create_data(worlds).unwrap();
        let mut data = plan.create_data(worlds).unwrap();
        source.write_qpos(&qpos).unwrap();
        source.write_qvel(&qvel).unwrap();
        source_plan.update(&mut source).unwrap();
        for (world, tree_asleep) in [(0, vec![-11, 1]), (1, vec![0, -11])] {
            source
                .write_world_sleep(
                    world,
                    &SleepTreeState {
                        tree_asleep,
                        nbody_awake: 1,
                        nv_awake: 1,
                    },
                )
                .unwrap();
        }
        source_plan.update_tendon_wake(&mut source).unwrap();
        velocity.write_qvel(&qvel).unwrap();
        joint.write_qpos(&qpos).unwrap();
        joint.write_qvel(&qvel).unwrap();
        joint_plan.update_joint(&mut joint).unwrap();
        for (field, value) in [
            (SmoothForceField::Bias, 1.0),
            (SmoothForceField::Actuator, 2.0),
            (SmoothForceField::Applied, 4.0),
        ] {
            data.write_other_force(field, &[value; 4]).unwrap();
        }
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let view = plan
            .update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        let snapshot = view.snapshot().unwrap();
        let smooth = view.smooth().unwrap();
        for (world, awake) in [(0, [true, false]), (1, [false, true])] {
            for (dof, &awake) in awake.iter().enumerate() {
                let passive =
                    snapshot.spring(world).unwrap()[dof] + snapshot.damper(world).unwrap()[dof];
                let expected = if awake {
                    passive - 1.0 + 2.0 + 4.0
                } else {
                    0.0
                };
                assert_eq!(
                    smooth.world(world).unwrap()[dof].to_bits(),
                    expected.to_bits()
                );
            }
        }
        drop(view);
        drop(velocity_view);

        for (world, tree_asleep) in [(0, vec![0, -11]), (1, vec![-11, 1])] {
            source
                .write_world_sleep(
                    world,
                    &SleepTreeState {
                        tree_asleep,
                        nbody_awake: 1,
                        nv_awake: 1,
                    },
                )
                .unwrap();
        }
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        assert!(matches!(
            plan.update(&joint, &velocity_view, &source, &mut data),
            Err(TransferError::StageNotReady {
                stage: "passive_force_sleep_source"
            })
        ));
        assert!(data.snapshot().is_err());
        assert!(data.smooth_output().is_err());
        drop(velocity_view);
        source_plan.update_tendon_wake(&mut source).unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let view = plan
            .update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        let snapshot = view.snapshot().unwrap();
        let smooth = view.smooth().unwrap();
        for (world, awake) in [(0, [false, true]), (1, [true, false])] {
            for (dof, &awake) in awake.iter().enumerate() {
                let passive =
                    snapshot.spring(world).unwrap()[dof] + snapshot.damper(world).unwrap()[dof];
                let expected = if awake {
                    passive - 1.0 + 2.0 + 4.0
                } else {
                    0.0
                };
                assert_eq!(
                    smooth.world(world).unwrap()[dof].to_bits(),
                    expected.to_bits()
                );
            }
        }
        drop(view);
        drop(velocity_view);

        let changed_qvel = vec![-0.25, 0.5, -0.75, 1.0];
        source.write_qvel(&changed_qvel).unwrap();
        velocity.write_qvel(&changed_qvel).unwrap();
        joint.write_qvel(&changed_qvel).unwrap();
        joint_plan.update_joint(&mut joint).unwrap();
        data.parts.write_range(4, &[f32::NAN; 8]).unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        assert!(matches!(
            plan.update(&joint, &velocity_view, &source, &mut data),
            Err(TransferError::StageNotReady {
                stage: "flex_edge_passive_source"
            })
        ));
        assert!(data.snapshot().is_err());
        assert!(data.smooth_output().is_err());
        let partial = read_parts(&data.parts, 8).unwrap();
        assert!(partial[4..12].iter().all(|value| value.is_finite()));
        drop(velocity_view);
        source_plan.update_flex_edges(&mut source).unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let recovered = plan
            .update(&joint, &velocity_view, &source, &mut data)
            .unwrap();
        let recovered_snapshot = recovered.snapshot().unwrap();
        let recovered_smooth = recovered.smooth().unwrap();
        assert!((0..worlds).any(|world| {
            recovered_snapshot.damper(world).unwrap() != snapshot.damper(world).unwrap()
        }));
        for (world, awake) in [(0, [false, true]), (1, [true, false])] {
            for (dof, &awake) in awake.iter().enumerate() {
                let passive = recovered_snapshot.spring(world).unwrap()[dof]
                    + recovered_snapshot.damper(world).unwrap()[dof];
                let expected = if awake {
                    passive - 1.0 + 2.0 + 4.0
                } else {
                    0.0
                };
                assert_eq!(
                    recovered_smooth.world(world).unwrap()[dof].to_bits(),
                    expected.to_bits()
                );
            }
        }
        drop(recovered);
        drop(velocity_view);

        let internal_input = plan.smooth.input().clone();
        let internal_plan = SmoothForcePlan::new(
            &session,
            internal_input,
            SmoothForceOptions {
                sleep_enabled: true,
                island_disabled: false,
            },
        )
        .unwrap();
        let mut internal = internal_plan.create_data(worlds).unwrap();
        for (field, value) in [
            (SmoothForceField::Passive, 5.0),
            (SmoothForceField::Bias, 1.0),
            (SmoothForceField::Actuator, 2.0),
            (SmoothForceField::Applied, 4.0),
        ] {
            internal.write_force(field, &[value; 4]).unwrap();
        }
        internal.write_awake(&[1, 0, 0, 1]).unwrap();
        internal_plan.update(&mut internal).unwrap();
        assert_eq!(internal.readback().unwrap().world(0).unwrap(), [10.0, 0.0]);
        assert_eq!(internal.readback().unwrap().world(1).unwrap(), [0.0, 10.0]);

        let tiny_sleep = session.upload(&[7i32]).unwrap();
        assert!(matches!(
            internal_plan.update_with_sleep_source(&mut internal, Some(&tiny_sleep)),
            Err(TransferError::Input(InputError::LengthMismatch {
                field: "smooth_force_sleep_source",
                ..
            }))
        ));
        assert!(internal.readback().is_err());
        internal_plan.update(&mut internal).unwrap();
        for options in [
            SmoothForceOptions {
                sleep_enabled: false,
                island_disabled: false,
            },
            SmoothForceOptions {
                sleep_enabled: true,
                island_disabled: true,
            },
        ] {
            let disabled =
                SmoothForcePlan::new(&session, plan.smooth.input().clone(), options).unwrap();
            let mut disabled_data = disabled.create_data(worlds).unwrap();
            for (field, value) in [
                (SmoothForceField::Passive, 5.0),
                (SmoothForceField::Bias, 1.0),
                (SmoothForceField::Actuator, 2.0),
                (SmoothForceField::Applied, 4.0),
            ] {
                disabled_data.write_force(field, &[value; 4]).unwrap();
            }
            disabled
                .update_with_sleep_source(&mut disabled_data, Some(&tiny_sleep))
                .unwrap();
            assert!(
                disabled_data
                    .readback()
                    .unwrap()
                    .world(0)
                    .unwrap()
                    .iter()
                    .all(|&value| value == 10.0)
            );
        }
        let disabled_options = JointPassiveOptions {
            disable_spring: true,
            disable_damper: true,
        };
        let disabled_joint =
            JointPassivePlan::new(&session, model, sleep_joint_parameters(), disabled_options)
                .unwrap();
        let disabled_tendon = TendonPassivePlan::new(
            &velocity_plan,
            tendon_parameters(),
            TendonPassiveOptions {
                disable_spring: true,
                disable_damper: true,
            },
        )
        .unwrap();
        let disabled_flex = FlexEdgePassivePlan::new(
            &source_plan,
            FlexEdgePassiveInput {
                length0: vec![f32::NAN],
                stiffness: vec![f32::NAN],
                damping: vec![f32::NAN],
                rigid: vec![false],
            },
            FlexEdgePassiveOptions {
                disable_spring: true,
                disable_damper: true,
            },
        )
        .unwrap();
        let disabled_plan =
            PassiveForcePlan::new(&disabled_joint, &disabled_tendon, &disabled_flex).unwrap();
        let mut disabled_data = disabled_plan.create_data(worlds).unwrap();
        for (field, value) in [
            (SmoothForceField::Bias, 1.0),
            (SmoothForceField::Actuator, 2.0),
            (SmoothForceField::Applied, 4.0),
        ] {
            disabled_data.write_other_force(field, &[value; 4]).unwrap();
        }
        let cold = disabled_plan.clear_disabled(&mut disabled_data).unwrap();
        for world in 0..worlds {
            assert_eq!(cold.snapshot().unwrap().spring(world).unwrap(), [0.0; 2]);
            assert_eq!(cold.snapshot().unwrap().damper(world).unwrap(), [0.0; 2]);
        }
        assert!(cold.smooth().is_err());
        source.write_qvel(&changed_qvel).unwrap();
        let filtered = disabled_plan
            .clear_disabled_with_sleep(&source, &mut disabled_data)
            .unwrap();
        assert_eq!(filtered.smooth().unwrap().world(0).unwrap(), [0.0, 5.0]);
        assert_eq!(filtered.smooth().unwrap().world(1).unwrap(), [5.0, 0.0]);
        let report = serde_json::json!({
            "worlds": worlds,
            "trees": 2,
            "awake_patterns": 4,
            "resident_device_sleep_source": true,
            "embedded_awake_path": true,
            "sleep_disabled_ignores_external_source": true,
            "island_disabled_ignores_external_source": true,
            "midchain_failure_stage": "flex_edge_passive_source",
            "partial_publication_rejected": true,
            "healthy_retry_recovered": true,
            "dual_disabled_cold_clear": true,
            "dual_disabled_sleep_merge": true,
            "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
            "performance_acceptance": false,
        });
        if let Some(path) = std::env::var_os("MJWARP_PASSIVE_SLEEP_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }
}
