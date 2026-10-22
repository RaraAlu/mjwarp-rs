//! 流体计划数据分配与测试破坏入口。

use super::*;

impl<'p> FluidForcePlan<'p> {
    pub fn create_data(&self, worlds: usize) -> Result<FluidForceData, TransferError> {
        let layout = layout(worlds, self.spec.nb, self.spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout;
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
            Ok(FluidForceData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                resident_velocity: match &self.source {
                    FluidForceSource::Independent(_) => None,
                    FluidForceSource::Resident(source) => Some(source.create_data(worlds)?),
                },
                body: allocate(layout.body.total_elements(), f32::NAN)?,
                qfrc: allocate(layout.qfrc.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn corrupt_metadata_for_test(
        &mut self,
        index: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata
            .corrupt_device_for_validator_test(index, &[value])
    }
}
