//! 稠密数据写入、检查与回读。

use super::*;

impl SparseMassDenseData {
    pub fn worlds(&self) -> usize {
        self.dense_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        check_input(values, self.matrix_layout.total_elements())?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.matrix_layout.world_elements(world)?;
        check_input(values, r.len())?;
        self.write(r.start, values)
    }
    pub(super) fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.matrix.write_range(4 + offset, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut values =
            crate::runtime::host_staging::<f32>(self.dense_layout.total_elements() + 8)?;
        self.dense.read_range_into(0, &mut values)?;
        crate::physics::check_device_values(&values, "sparse_dense_matrix")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassDenseOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_dense_gather",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SparseMassDenseOutput {
                input: self.input.clone(),
                layout: self.dense_layout,
                values: self.checked_values()?,
            })
        }
    }
}
