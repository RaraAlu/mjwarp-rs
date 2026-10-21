//! 借用矩阵向量数据实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

use super::super::checked;
use super::*;

impl MassMatrixMultiplyData {
    pub fn worlds(&self) -> usize {
        self.vector_layout.worlds()
    }
    pub fn write_vector(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(0, self.vector_layout.total_elements(), values)
    }
    pub fn write_world_vector(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.vector_layout.world_elements(world)?;
        self.write(r.start, r.len(), values)
    }
    fn write(&mut self, offset: usize, count: usize, values: &[f32]) -> Result<(), TransferError> {
        checked(values, count, "borrowed_mul_m_vector")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.vector.write_range(4 + offset, values)
        }
    }
}
