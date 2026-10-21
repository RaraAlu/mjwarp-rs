//! 因子计划、数据与输出实现。

#[cfg(feature = "cuda-probe")]
use super::kernels::SparseMassCopyKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::SparseMassDiagKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::SparseMassUpdateKernel;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SparseMassCopyMetadataBuffer;
use super::*;

impl SparseMassFactorOutput {
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassFactorWorld<'_>, InputError> {
        let r = self.layout.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        Ok(SparseMassFactorWorld {
            sparse_factors: &v[..self.input.nc()],
            diagonal_inverse: &v[self.input.nc()..],
        })
    }
}

impl SparseMassFactorPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        // 无稀疏块时，上游不分配CSR因子尾区。
        // 不能用本工作区冒充纯紧凑qLD布局。
        if input.nv() != 0
            && !input
                .blocks()
                .iter()
                .any(|b| b.route == SparseMassRoute::SparseLdl)
        {
            return Err(InputError::InvalidTopology {
                field: "qLD_block_adr",
                index: 0,
                reason: "sparse_factor_region_required",
            }
            .into());
        }
        for block in input.blocks() {
            if !matches!(
                block.route,
                SparseMassRoute::SparseLdl | SparseMassRoute::Compact
            ) {
                return Err(InputError::InvalidTopology {
                    field: "qLD_block_adr",
                    index: block.start,
                    reason: "noncompact_mixed_factor_stage_not_implemented",
                }
                .into());
            }
        }
        Self::from_input(session, input)
    }
    // The mixed caller validates supported routes before using this factory.
    // Keep the public constructor's noncompact rejection above unchanged.
    pub(super) fn from_input(
        session: &TransferSession,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        let input = Arc::new(input);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, input);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nv = u32::try_from(input.nv()).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nv",
            })?;
            let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nc",
            })?;
            let nv_i32 = i32::try_from(nv).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nv",
            })?;
            let nc_i32 = i32::try_from(nc).map_err(|_| InputError::Overflow {
                field: "sparse_factor_nc",
            })?;
            let mut packed = vec![nv_i32, nc_i32];
            packed.extend(&input.fields().m_rownnz);
            packed.extend(&input.fields().m_rowadr);
            packed.extend(input.qld_all_updates().iter().flatten());
            if packed.len() > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "sparse_factor_metadata",
                }
                .into());
            }
            let metadata = SparseMassCopyMetadataBuffer::upload(
                session,
                packed,
                nv,
                nc,
                input.qld_level_offsets(),
            )?;
            let copy = SparseMassCopyKernel::compile(session, &metadata)?;
            let update = SparseMassUpdateKernel::compile(session, &metadata)?;
            let diag = SparseMassDiagKernel::compile(session, &metadata)?;
            Ok(Self {
                input,
                session: session.clone(),
                metadata,
                copy,
                update,
                diag,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassFactorData, TransferError> {
        let (matrix_layout, output_layout) = layouts(&self.input, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, output_layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |layout: BatchLayout, initial| {
                let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
                values.fill(-131072.0);
                values[4..4 + layout.total_elements()].fill(initial);
                self.session.upload(&values)
            };
            Ok(SparseMassFactorData {
                input: self.input.clone(),
                matrix_layout,
                output_layout,
                ready: false,
                matrix: allocate(matrix_layout, 0.0)?,
                output: allocate(output_layout, f32::NAN)?,
            })
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassFactorData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds_usize = data.worlds();
            self.copy.launch(
                &self.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds_usize,
            )?;
            for level in (0..self.input.qld_level_offsets().len() - 1).rev() {
                self.update.launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds_usize,
                    level,
                )?;
            }
            if self.input.nv() != 0 {
                self.diag.launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.output,
                    worlds_usize,
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
impl SparseMassFactorData {
    pub fn worlds(&self) -> usize {
        self.output_layout.worlds()
    }
    pub fn write_matrix(&mut self, values: &[f32]) -> Result<(), TransferError> {
        matrix_values(self.matrix_layout, values)?;
        self.write(0, values)
    }
    pub fn write_world_matrix(
        &mut self,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let range = self.matrix_layout.world_elements(world)?;
        matrix_values(BatchLayout::new(1, self.input.nc(), 4)?, values)?;
        self.write(range.start, values)
    }
    pub(super) fn write(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.matrix.write_range(4 + offset, values)
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut matrix =
            crate::runtime::host_staging::<f32>(self.matrix_layout.total_elements() + 8)?;
        self.matrix.read_range_into(0, &mut matrix)?;
        crate::physics::check_device_values(&matrix, "sparse_mass_M")?;
        let mut values =
            crate::runtime::host_staging::<f32>(self.output_layout.total_elements() + 8)?;
        self.output.read_range_into(0, &mut values)?;
        // 先定位主元，再统一检查有限值与守卫。
        for world in 0..self.worlds() {
            let start = 4 + self.output_layout.world_elements(world)?.start;
            for dof in 0..self.input.nv() {
                let f = self.input.fields();
                let diagonal = values[start + (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize];
                if !diagonal.is_finite() || diagonal <= 0.0 {
                    return Err(TransferError::InvalidPivot {
                        world,
                        dof,
                        value: f64::from(diagonal),
                    });
                }
            }
        }
        crate::physics::check_device_values(&values, "sparse_mass_factor")?;
        Ok(values)
    }
    pub fn readback(&self) -> Result<SparseMassFactorOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        Ok(SparseMassFactorOutput {
            input: self.input.clone(),
            layout: self.output_layout,
            values: self.checked_values()?,
        })
    }
}
