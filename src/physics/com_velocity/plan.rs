//! 空间速度计划实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::model::{AttachedModelInput, KinematicsParameters, ParameterBatch};
#[cfg(all(test, feature = "cuda-probe"))]
use crate::runtime::TransferBuffer;
use crate::runtime::TransferSession;

#[cfg(feature = "cuda-probe")]
use super::cuda::COM_VELOCITY_CUDA;
use super::*;

impl ComVelocityPlan {
    /// 检查模型后建立严格常驻底座。
    /// 默认位置采用既有qpos0。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        Self::with_parameters(session, model, KinematicsParameters::default())
    }

    pub fn with_parameters(
        session: &TransferSession,
        model: AttachedModelInput,
        parameters: KinematicsParameters,
    ) -> Result<Self, TransferError> {
        let k = model.rigid().kinematics();
        let (nbody, njnt, nv) = (k.nbody(), k.njnt(), k.nv());
        ComVelocityLayout::new(1, nbody, njnt, nv)?;
        #[cfg(feature = "cuda-probe")]
        let metadata_values = {
            let f = k.fields();
            let mut values = crate::runtime::host_staging::<i32>(3 * nbody + 2 * njnt)?;
            let mut cursor = 0;
            for field in [
                &f.body_parentid,
                &f.body_jntnum,
                &f.body_jntadr,
                &f.jnt_type,
                &f.jnt_dofadr,
            ] {
                values[cursor..cursor + field.len()].copy_from_slice(field);
                cursor += field.len();
            }
            values
        };
        let position = KinematicsPlan::with_parameters(session, model, parameters)?;
        #[cfg(feature = "cuda-probe")]
        let metadata = session.upload(&metadata_values)?;
        #[cfg(feature = "cuda-probe")]
        let kernel = SynchronousKernel::compile(session, COM_VELOCITY_CUDA, "com_velocity")?;
        #[cfg(feature = "cuda-probe")]
        let position_kernels = position.compile_velocity_position()?;
        Ok(Self {
            position,
            nbody,
            njnt,
            nv,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernel,
            #[cfg(feature = "cuda-probe")]
            position_kernels,
        })
    }

    pub fn device(&self) -> usize {
        self.position.device()
    }

    pub(in crate::physics) fn check_data(
        &self,
        data: &ComVelocityData,
    ) -> Result<(), TransferError> {
        self.position.check_data(&data.position)
    }

    pub(in crate::physics) fn fluid_force_context(
        &self,
    ) -> Result<
        (
            TransferSession,
            std::sync::Arc<AttachedModelInput>,
            ParameterBatch,
            ParameterBatch,
        ),
        TransferError,
    > {
        self.position.fluid_force_context()
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn fluid_inputs<'a>(
        &self,
        data: &'a ComVelocityData,
        require_attached: bool,
    ) -> Result<FluidInputs<'a>, TransferError> {
        self.check_data(data)?;
        if !data.ready {
            return Err(TransferError::StageNotReady {
                stage: "com_velocity",
            });
        }
        Ok(FluidInputs {
            rigid: &data.rigid,
            com: &data.com,
            velocity: &data.output,
            attached: require_attached
                .then(|| self.position.fluid_force_attached_source(&data.position))
                .transpose()?,
        })
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn update_fluid_attached(
        &self,
        data: &mut ComVelocityData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.ready {
            return Err(TransferError::StageNotReady {
                stage: "com_velocity",
            });
        }
        self.position.update_rigid(&mut data.position)?;
        self.position.update_attached(&mut data.position)
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn fluid_snapshot_for_test(
        &self,
        data: &ComVelocityData,
        require_attached: bool,
    ) -> Result<FluidSnapshot, TransferError> {
        let inputs = self.fluid_inputs(data, require_attached)?;
        let read = |buffer: &TransferBuffer<f32>| -> Result<Vec<f32>, TransferError> {
            let mut values = crate::runtime::host_staging::<f32>(buffer.len())?;
            buffer.read_range_into(0, &mut values)?;
            Ok(values)
        };
        Ok((
            read(inputs.rigid)?,
            read(inputs.com)?,
            read(inputs.velocity)?,
            inputs.attached.map(read).transpose()?,
        ))
    }

    /// 创建独立世界与零qvel。
    /// 初始速度结果保持未就绪。
    pub fn create_data(&self, worlds: usize) -> Result<ComVelocityData, TransferError> {
        let layout = ComVelocityLayout::new(worlds, self.nbody, self.njnt, self.nv)?;
        let position = self.position.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, position);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut velocities = crate::runtime::host_staging::<f32>(layout.guarded_qvel)?;
            velocities.fill(-131072.0);
            velocities[4..layout.guarded_qvel - 4].fill(0.0);
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(ComVelocityData {
                position,
                layout,
                ready: false,
                qvel: self.session.upload(&velocities)?,
                output: self.session.upload(&values)?,
                rigid: velocity_workspace(&self.session, layout.guarded_rigid)?,
                com: velocity_workspace(&self.session, layout.guarded_com)?,
            })
        }
    }

    /// GPU重算位置、质心与空间速度。
    /// 不上传模型或重新编译内核。
    /// 阶段间不回读宿主中间量。
    /// 完成仍需显式回读数值检查。
    pub fn update(&self, data: &mut ComVelocityData) -> Result<(), TransferError> {
        self.position.check_data(&data.position)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.position.update_velocity_position(
                &data.position,
                &self.position_kernels,
                &mut data.rigid,
                &mut data.com,
            )?;
            let worlds = data.worlds() as u32;
            // SAFETY: Validated topology bounds joint/DOF indices and parent order.
            // The checked layouts bound all guarded offsets. Model, state and
            // private FK, COM and velocity workspaces all use frozen f32.
            // All buffers use this plan's session; one thread owns one world.
            // COM remains on-device with stride 14*nbody+6*nv. The adapter waits
            // before any buffer or compiled module can drop or be reused.
            unsafe {
                self.kernel.launch(
                    &self.metadata,
                    &data.qvel,
                    &data.com,
                    &mut data.output,
                    [self.nbody as u32, self.njnt as u32, self.nv as u32, worlds],
                )?;
            }
            data.ready = true;
            Ok(())
        }
    }
}
