//! 被动合力数据访问与视图回读。

use super::*;

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

    pub(super) fn snapshot(&self) -> Result<PassiveForceSnapshot, TransferError> {
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
                gravity: read_parts(&self.gravity, self.layout.total_elements())?,
            })
        }
    }

    pub(super) fn smooth_output(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "passive_force",
            });
        }
        self.smooth.readback()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn corrupt_parts_guard_for_test(
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
