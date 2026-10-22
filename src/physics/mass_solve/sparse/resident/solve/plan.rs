//! 常驻稀疏求解计划。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::{SparseMassPlan, SparseMassSolveData, SparseMassSolvePlan, layout};
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::model::{AttachedModelInput, SparseMassInput};
use crate::runtime::TransferSession;

impl SparseMassSolvePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        let mass = SparseMassPlan::new(session, model, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = mass;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = mass.input();
            let offsets = input.qld_level_offsets();
            let mut metadata = vec![(offsets.len() - 1) as i32];
            metadata.extend(input.qld_block_adr());
            let f = input.fields();
            metadata.extend((0..input.nv()).map(|i| f.m_rowadr[i] + f.m_rownnz[i] - 1));
            metadata.extend(offsets);
            metadata.extend(input.qld_all_updates().iter().flatten());
            let static_dims = super::launcher_abi::SparseSolveStaticDims::new(
                input.nv(),
                input.nc(),
                offsets.len() - 1,
                input.qld_all_updates().len(),
            )
            .map_err(super::launcher_abi::sparse_solve_launch_error)?;
            let metadata = super::launcher_abi::SparseSolveMetadataBuffer::from_values(
                session,
                metadata,
                static_dims,
            )?;
            Ok(Self {
                mass,
                metadata,
                kernels: super::kernels::SparseSolveKernels::compile(session, CUDA)?,
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
    ) -> Result<SparseMassSolveData, TransferError> {
        let layout = layout(worlds, self.input().nv(), rhs_count)?;
        let mass = self.mass.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (mass, layout);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
            values.fill(-131072.0);
            values[4..4 + layout.total_elements()].fill(0.0);
            let rhs = self.mass.session.upload(&values)?;
            values[4..4 + layout.total_elements()].fill(f32::NAN);
            Ok(SparseMassSolveData {
                mass,
                layout,
                rhs_count,
                ready: false,
                rhs,
                output: self.mass.session.upload(&values)?,
            })
        }
    }
    pub fn update(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_position(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_position(&mut data.mass)
    }
    pub fn update_matrix(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_matrix(&mut data.mass)
    }
    pub fn update_gather(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_gather(&mut data.mass)
    }
    pub fn update_factor(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        self.mass.update_factor(&mut data.mass)
    }
    pub fn update_solve(&self, data: &mut SparseMassSolveData) -> Result<(), TransferError> {
        self.mass.check_data(&data.mass)?;
        data.ready = false;
        if !data.mass.gathered || !data.mass.factor.ready {
            return Err(TransferError::StageNotReady {
                stage: "sparse_mass_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds() as u32;
            let rhs_count = data.rhs_count as u32;
            self.kernels.launch_validate(
                &self.metadata,
                &data.rhs,
                &data.mass.factor.output,
                &mut data.mass.status,
                worlds,
                rhs_count,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.mass.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "sparse_solve_status")?;
            for (index, &actual) in status[4..4 + data.worlds()].iter().enumerate() {
                if actual != 0.0 {
                    if actual == 1.0 || actual == 3.0 {
                        data.mass.factor.ready = false;
                    }
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 0.0,
                        actual,
                    }
                    .into());
                }
            }
            // 每块独占一个世界/右端项，128线程全部到达层同步。
            self.kernels.launch_fused(
                &self.metadata,
                &data.rhs,
                &data.mass.factor.output,
                &mut data.output,
                worlds,
                rhs_count,
            )?;
            data.checked_solution()?;
            data.ready = true;
            Ok(())
        }
    }
}
