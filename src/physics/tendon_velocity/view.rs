//! 肌腱速度历史视图访问。

use super::*;

impl TendonVelocityView<'_, '_> {
    pub(in crate::physics) fn belongs_to(&self, plan: &TendonVelocityPlan<'_>) -> bool {
        std::ptr::eq(self.plan, plan)
    }
    pub(in crate::physics) fn source_data(&self) -> &KinematicsData {
        self.source
    }
    pub(in crate::physics) fn worlds(&self) -> usize {
        self.data.worlds()
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn tendon_passive_device(
        &self,
    ) -> Result<TendonVelocityDeviceView<'_>, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_velocity",
            });
        }
        self.plan.source.check_data(self.source)?;
        let source = self.plan.source.tendon_velocity_source(self.source)?;
        Ok(TendonVelocityDeviceView {
            source,
            velocity: &self.data.current,
            worlds: self.data.worlds(),
        })
    }
    pub fn snapshot(&self) -> Result<TendonVelocitySnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_velocity",
            });
        }
        self.plan.source.check_data(self.source)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.plan.source.tendon_velocity_source(self.source)?;
            Ok(TendonVelocitySnapshot {
                layout: self.data.output,
                values: read(
                    &self.data.current,
                    self.data.output.total_elements(),
                    "tendon_velocity",
                )?,
            })
        }
    }
}
