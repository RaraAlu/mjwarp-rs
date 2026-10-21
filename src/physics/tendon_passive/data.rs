//! 肌腱被动力数据与快照视图。

use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

impl TendonPassiveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn corrupt_guard_for_test(
        &mut self,
        current: bool,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        let buffer = if current {
            &mut self.current
        } else {
            &mut self.staged
        };
        buffer.write_range(index, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn buffer_len_for_test(&self, current: bool) -> usize {
        if current {
            self.current.len()
        } else {
            self.staged.len()
        }
    }
}

impl TendonPassiveView<'_, '_, '_> {
    pub fn snapshot(&self) -> Result<TendonPassiveSnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_passive",
            });
        }
        debug_assert!(self.velocity.belongs_to(self.plan.velocity));
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.tendon_passive_device()?;
            Ok(TendonPassiveSnapshot {
                layout: self.data.layout,
                values: read(
                    &self.data.current,
                    2 * self.data.layout.total_elements(),
                    "tendon_passive",
                )?,
            })
        }
    }
}
