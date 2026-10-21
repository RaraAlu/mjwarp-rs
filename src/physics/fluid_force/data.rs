//! 流体数据访问与视图回读。

use super::*;

impl FluidForceData {
    pub fn worlds(&self) -> usize {
        self.layout.qfrc.worlds()
    }

    pub(in crate::physics) fn snapshot(&self) -> Result<FluidForceSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "fluid_force",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(FluidForceSnapshot {
                layout: self.layout,
                body: read_output(
                    &self.body,
                    self.layout.body.total_elements(),
                    "fluid_force_body",
                )?,
                qfrc: read_output(&self.qfrc, self.layout.qfrc.total_elements(), "qfrc_fluid")?,
            })
        }
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn corrupt_body_guard_for_test(
        &mut self,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        self.body.write_range(index, &[value])
    }
}

impl FluidForceView<'_> {
    pub fn snapshot(&self) -> Result<FluidForceSnapshot, TransferError> {
        self.data.snapshot()
    }
}
