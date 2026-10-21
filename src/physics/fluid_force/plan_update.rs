//! 流体计划更新与常驻源入口。

use super::*;

impl<'p> FluidForcePlan<'p> {
    pub fn update<'a>(
        &'a self,
        source: &'a mut ComVelocityData,
        data: &'a mut FluidForceData,
    ) -> Result<FluidForceView<'a>, TransferError> {
        let FluidForceSource::Independent(plan) = &self.source else {
            return Err(TransferError::ModelMismatch);
        };
        if !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != source.worlds()
            || plan.check_data(source).is_err()
            || data.resident_velocity.is_some()
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let has_ellipsoid = self
                .spec
                .input
                .body_fluid_ellipsoid
                .iter()
                .any(|&value| value);
            if has_ellipsoid {
                plan.update_fluid_attached(source)?;
            }
            let inputs = plan.fluid_inputs(source, has_ellipsoid)?;
            self.launch(inputs, data)?;
            data.ready = true;
            Ok(FluidForceView {
                data,
                _source: source,
            })
        }
    }

    pub(in crate::physics) fn accepts_resident_source(&self, source: &KinematicsPlan) -> bool {
        matches!(&self.source, FluidForceSource::Resident(plan) if plan.accepts_source(source))
    }

    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    pub(in crate::physics) fn update_resident(
        &self,
        source: &KinematicsData,
        data: &mut FluidForceData,
    ) -> Result<(), TransferError> {
        let FluidForceSource::Resident(plan) = &self.source else {
            return Err(TransferError::ModelMismatch);
        };
        if !Arc::ptr_eq(&self.spec, &data.spec) || data.worlds() != source.worlds() {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        let resident = data
            .resident_velocity
            .as_mut()
            .ok_or(TransferError::ModelMismatch)?;
        plan.update(source, resident)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let has_ellipsoid = self
                .spec
                .input
                .body_fluid_ellipsoid
                .iter()
                .any(|&value| value);
            let inputs = plan.fluid_inputs(source, resident, has_ellipsoid)?;
            let layout = data.layout;
            self.launch_buffers(
                inputs,
                layout,
                &mut data.body,
                &mut data.qfrc,
                &mut data.status,
            )?;
            data.ready = true;
            Ok(())
        }
    }
}
