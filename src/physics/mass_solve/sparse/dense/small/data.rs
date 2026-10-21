//! Small数据写入、检查与回读。

use super::*;

impl SparseMassSmallData {
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics::mass_solve::sparse) fn test_factor_buffer_mut(
        &mut self,
    ) -> &mut TransferBuffer<f32> {
        &mut self.factor
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics::mass_solve::sparse) fn copy_matrix_from(
        &mut self,
        source: &TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        self.invalidate();
        self.source
            .matrix
            .copy_range_from(0, source, 0, source.len())
    }
    pub(in crate::physics::mass_solve::sparse) fn invalidate(&mut self) {
        self.ready = false;
        self.source.ready = false;
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics::mass_solve::sparse) fn factor_buffer(
        &self,
    ) -> Result<&TransferBuffer<f32>, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_small_factor",
            });
        }
        Ok(&self.factor)
    }
    pub fn worlds(&self) -> usize {
        self.source.worlds()
    }
    pub(super) fn after_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.source.write_matrix(values);
        self.after_write(result)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let result = self.source.write_world_matrix(world, values);
        self.after_write(result)
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let layout = self.source.dense_layout;
        let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
        self.factor.read_range_into(0, &mut values)?;
        for world in 0..self.worlds() {
            let first = 4 + layout.world_elements(world)?.start;
            for block in self
                .source
                .input
                .blocks()
                .iter()
                .filter(|b| b.route == SparseMassRoute::SmallCholesky)
            {
                let address = self.source.input.qld_block_adr()[block.start] as usize;
                for i in 0..block.size {
                    let value = values[first + address + i * block.size + i];
                    if !value.is_finite() || value <= 0.0 {
                        return Err(TransferError::InvalidPivot {
                            world,
                            dof: block.start + i,
                            value: f64::from(value),
                        });
                    }
                }
            }
        }
        crate::physics::check_device_values(&values, "small_prefix_U")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassSmallOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "small_prefix_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SparseMassSmallOutput {
                input: self.source.input.clone(),
                layout: self.source.dense_layout,
                values: self.checked_values()?,
            })
        }
    }
}

// Frozen smooth.py:1206-1247, actual Small2 PTX: sqrt.rn/rcp.rn/mul/sub.
// Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
// Keep mul/sub permission to contract; do not force early FMA or explicit .rn.
