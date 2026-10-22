//! 常驻借用空间速度实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

#[cfg(feature = "cuda-probe")]
use super::cuda::COM_VELOCITY_CUDA;
use super::*;

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
impl<'p> ResidentComVelocityPlan<'p> {
    pub(in crate::physics) fn new(source: &'p KinematicsPlan) -> Result<Self, TransferError> {
        let (session, model, _, _) = source.fluid_force_context()?;
        let k = model.rigid().kinematics();
        let (nbody, njnt, nv) = (k.nbody(), k.njnt(), k.nv());
        ComVelocityLayout::new(1, nbody, njnt, nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, model);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
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
            let static_dims =
                ComVelocityStaticDims::new(nbody, njnt, nv).map_err(com_velocity_launch_error)?;
            Ok(Self {
                source,
                identity: std::sync::Arc::new(()),
                nbody,
                njnt,
                nv,
                metadata: Box::new(ComVelocityMetadataBuffer::upload(
                    &session,
                    values,
                    static_dims,
                )?),
                kernel: ComVelocityKernel::compile(&session, COM_VELOCITY_CUDA)?,
                position_kernels: source.compile_velocity_position()?,
                session,
            })
        }
    }

    pub(in crate::physics) fn accepts_source(&self, source: &KinematicsPlan) -> bool {
        std::ptr::eq(self.source, source)
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn fluid_inputs<'a>(
        &self,
        source: &'a KinematicsData,
        data: &'a ResidentComVelocityData,
        require_attached: bool,
    ) -> Result<FluidInputs<'a>, TransferError> {
        if !data.ready {
            return Err(TransferError::StageNotReady {
                stage: "resident_com_velocity",
            });
        }
        Ok(FluidInputs {
            rigid: &data.rigid,
            com: &data.com,
            velocity: &data.output,
            attached: require_attached
                .then(|| self.source.fluid_force_attached_source(source))
                .transpose()?,
        })
    }

    pub(in crate::physics) fn create_data(
        &self,
        worlds: usize,
    ) -> Result<ResidentComVelocityData, TransferError> {
        let layout = ComVelocityLayout::new(worlds, self.nbody, self.njnt, self.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(ResidentComVelocityData {
                identity: self.identity.clone(),
                layout,
                ready: false,
                output: velocity_workspace(&self.session, layout.guarded)?,
                rigid: velocity_workspace(&self.session, layout.guarded_rigid)?,
                com: velocity_workspace(&self.session, layout.guarded_com)?,
            })
        }
    }

    pub(in crate::physics) fn update(
        &self,
        source: &KinematicsData,
        data: &mut ResidentComVelocityData,
    ) -> Result<(), TransferError> {
        if !std::sync::Arc::ptr_eq(&self.identity, &data.identity)
            || data.layout.output.worlds() != source.worlds()
        {
            return Err(TransferError::ModelMismatch);
        }
        self.source.check_data(source)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let qvel = self.source.fluid_force_qvel_source(source)?;
            self.source.update_velocity_position(
                source,
                &self.position_kernels,
                &mut data.rigid,
                &mut data.com,
            )?;
            let worlds = source.worlds() as u32;
            self.kernel
                .launch(&self.metadata, qvel, &data.com, &mut data.output, worlds)?;
            data.ready = true;
            Ok(())
        }
    }
}

impl ResidentComVelocityData {
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn invalidate(&mut self) {
        self.ready = false;
    }
}
