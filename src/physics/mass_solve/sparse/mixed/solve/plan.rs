//! 混合求解计划构造与更新入口。

#[cfg(feature = "cuda-probe")]
use super::kernels::MixedSolveFusedKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::MixedSolveValidationKernel;
use super::layout::layout;
#[cfg(feature = "cuda-probe")]
use super::layout::metadata;
#[cfg(feature = "cuda-probe")]
use super::validation_buffer::MixedSolveValidationMetadataBuffer;
use super::*;

impl SparseMassMixedSolvePlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        let mass = SparseMassMixedFactorPlan::new(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = mass;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let stride = mass
                .input()
                .qld_block_total()
                .checked_add(mass.input().nc())
                .and_then(|value| value.checked_add(mass.input().nv()))
                .ok_or(InputError::Overflow {
                    field: "mixed_solve_validation_stride",
                })?;
            let metadata = MixedSolveValidationMetadataBuffer::upload(
                session,
                metadata(mass.input())?,
                mass.input().nv(),
                stride,
            )?;
            let validation = MixedSolveValidationKernel::compile(session, &metadata)?;
            let solve = MixedSolveFusedKernel::compile(session, &metadata)?;
            Ok(Self {
                mass,
                metadata,
                validation,
                solve,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.mass.input()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<SparseMassMixedSolveData, TransferError> {
        let layout = layout(worlds, self.input().nv(), rhs_count)?;
        let mass = self.mass.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (mass, layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, initial: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(initial);
                self.mass.session.upload(&values)
            };
            Ok(SparseMassMixedSolveData {
                mass,
                layout,
                rhs_count,
                ready: false,
                rhs: allocate(layout.total_elements(), 0.0)?,
                output: allocate(layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_gather(&mut data.mass)
    }
    pub fn update_factor(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_factor(&mut data.mass)
    }
    pub fn update_solve(&self, data: &mut SparseMassMixedSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        if !data.mass.ready {
            return Err(TransferError::StageNotReady {
                stage: "mixed_solve_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dimensions = self.solve.dimensions(data.rhs_count, data.worlds())?;
            self.validation.launch(
                &self.metadata,
                &data.rhs,
                &data.mass.output,
                &mut data.status,
                dimensions,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "mixed_solve_status")?;
            for (world, &value) in status[4..4 + data.worlds()].iter().enumerate() {
                if value != 0.0 {
                    if value == 1.0 || value == 3.0 {
                        data.mass.ready = false;
                    }
                    return Err(InputError::InvalidTopology {
                        field: "mixed_solve_sources",
                        index: world,
                        reason: "invalid_cached_factor_or_rhs",
                    }
                    .into());
                }
            }
            self.solve.launch(
                &self.metadata,
                &data.rhs,
                &data.mass.output,
                &mut data.output,
                dimensions,
            )?;
            data.checked_solution()?;
            data.ready = true;
            Ok(())
        }
    }
}
