//! 质量矩阵乘法数据实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

use super::*;

impl MassMultiplyData {
    pub fn worlds(&self) -> usize {
        self.vector_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, false, 0, self.matrix_layout.total_elements(), values)
    }
    pub fn write_alternate_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, true, 0, self.matrix_layout.total_elements(), values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.matrix_layout.world_elements(world)?;
        self.write(false, false, r.start, r.len(), values)
    }
    pub fn write_vector(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, false, 0, self.vector_layout.total_elements(), values)
    }
    pub fn write_world_vector(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.vector_layout.world_elements(world)?;
        self.write(true, false, r.start, r.len(), values)
    }
    fn write(
        &mut self,
        vector: bool,
        alternate: bool,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        checked(
            values,
            expected,
            if vector {
                "mul_m_vector"
            } else {
                "mul_m_matrix"
            },
        )?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (alternate, offset);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let buffer = if vector {
                &mut self.vector
            } else if alternate {
                &mut self.alternate
            } else {
                &mut self.matrix
            };
            buffer.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut v = crate::runtime::host_staging::<f32>(self.vector_layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut v)?;
        crate::physics::check_device_values(&v, "mul_m_result")?;
        Ok(v)
    }
    pub fn readback(&self) -> Result<MassMultiplyOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mass_multiply",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMultiplyOutput {
                layout: self.vector_layout,
                values: self.checked_values()?,
            })
        }
    }
}
