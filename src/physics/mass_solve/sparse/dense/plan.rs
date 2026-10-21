//! 稠密计划构造与更新入口。

use super::*;

impl SparseMassDensePlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        let input = Arc::new(input);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (input, session);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut sources = input.dense_block_sources()?;
            if sources.is_empty() {
                sources.push(0);
            }
            let nc = u32::try_from(input.nc()).map_err(|_| InputError::Overflow {
                field: "dense_validation_nc",
            })?;
            let prefix =
                u32::try_from(input.qld_block_total()).map_err(|_| InputError::Overflow {
                    field: "dense_validation_prefix",
                })?;
            let metadata = DenseGatherMetadataBuffer::upload(session, sources, nc, prefix)?;
            let validator = DenseValidationKernel::compile(session, nc, prefix)?;
            let gather = DenseGatherKernel::compile(session, &metadata)?;
            Ok(Self {
                input,
                session: session.clone(),
                metadata,
                validator,
                gather,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        &self.input
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassDenseData, TransferError> {
        let (matrix_layout, dense_layout) = layouts(&self.input, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix_layout, dense_layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, initial| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(initial);
                self.session.upload(&values)
            };
            Ok(SparseMassDenseData {
                input: self.input.clone(),
                matrix_layout,
                dense_layout,
                ready: false,
                matrix: allocate(matrix_layout.total_elements(), 0.0)?,
                dense: allocate(dense_layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassDenseData) -> Result<(), TransferError> {
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
            let p = self.validator.prefix;
            let worlds = u32::try_from(data.worlds()).map_err(|_| InputError::Overflow {
                field: "sparse_dense_worlds",
            })?;
            self.validator.launch(
                self.metadata.as_buffer(),
                &data.matrix,
                &data.matrix,
                &mut data.status,
                worlds as usize,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "sparse_dense_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 1.0 {
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 1.0,
                        actual,
                    }
                    .into());
                }
            }
            if p != 0 {
                self.gather.launch(
                    &self.metadata,
                    &data.matrix,
                    &data.matrix,
                    &mut data.dense,
                    worlds as usize,
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
