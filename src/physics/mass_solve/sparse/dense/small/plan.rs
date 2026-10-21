//! Small计划构造、写入与更新入口。

#[cfg(feature = "cuda-probe")]
use super::buffer::SmallFactorMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::kernels::SmallFactorKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::SmallValidationKernel;
use super::*;

impl SparseMassSmallPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        check_routes(&input)?;
        let source = SparseMassDensePlan::new(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut meta = Vec::new();
            let mut group_specs = Vec::new();
            for size in 2..=6 {
                let first = meta.len() / 3;
                for block in source
                    .input
                    .blocks()
                    .iter()
                    .filter(|b| b.route == SparseMassRoute::SmallCholesky && b.size == size)
                {
                    meta.extend([
                        block.start as i32,
                        source.input.qld_block_adr()[block.start],
                        size as i32,
                    ]);
                }
                let count = meta.len() / 3 - first;
                if count != 0 {
                    group_specs.push((size as u32, first as u32, count as u32));
                }
            }
            if meta.is_empty() {
                meta.push(0);
            }
            let prefix = u32::try_from(source.input.qld_block_total()).map_err(|_| {
                InputError::Overflow {
                    field: "small_factor_prefix",
                }
            })?;
            let metadata = SmallFactorMetadataBuffer::upload(session, meta, prefix)?;
            let groups = group_specs
                .into_iter()
                .map(|(size, first, count)| {
                    SmallFactorKernel::compile(session, &metadata, size, first, count)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Self {
                source,
                metadata,
                groups,
                validator: SmallValidationKernel::compile(session, prefix)?,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.source.input()
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassSmallData, TransferError> {
        let source = self.source.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let len = source.dense_layout.total_elements();
            let mut values = crate::runtime::host_staging::<f32>(len + 8)?;
            values.fill(-131072.0);
            values[4..4 + len].fill(f32::NAN);
            Ok(SparseMassSmallData {
                source,
                ready: false,
                factor: self.source.session.upload(&values)?,
            })
        }
    }
    pub(super) fn check_data(&self, data: &SparseMassSmallData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.source.input, &data.source.input) {
            return Err(TransferError::ModelMismatch);
        }
        Ok(())
    }
    pub fn update(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        self.source.update(&mut data.source)
    }
    pub fn update_factor(&self, data: &mut SparseMassSmallData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        if !data.source.ready {
            return Err(TransferError::StageNotReady {
                stage: "small_prefix_gather",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            self.validator.launch(
                self.metadata.as_buffer(),
                &data.source.dense,
                &data.source.dense,
                &mut data.source.status,
                worlds,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.source.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "small_prefix_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 1.0 {
                    data.source.ready = false;
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 1.0,
                        actual,
                    }
                    .into());
                }
            }
            for kernel in &self.groups {
                kernel.launch(
                    &self.metadata,
                    &data.source.dense,
                    &data.source.dense,
                    &mut data.factor,
                    u32::try_from(worlds).map_err(|_| InputError::Overflow {
                        field: "small_factor_worlds",
                    })?,
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
