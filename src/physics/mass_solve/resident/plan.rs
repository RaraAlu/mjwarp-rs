//! 常驻分解计划与阶段更新。

#[cfg(feature = "cuda-probe")]
use super::super::factor::{check_block_intervals, factor_blocks};
#[cfg(feature = "cuda-probe")]
use super::super::probe::check_result;
use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::model::AttachedModelInput;
use crate::runtime::TransferSession;

impl MassSolvePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        let nv = model.rigid().kinematics().nv();
        MassSolveLayout::new(1, nv, 1)?;
        #[cfg(feature = "cuda-probe")]
        let (nbody, blocks, metadata, source) = {
            let blocks = factor_blocks(&model.rigid().fields().dof_parentid);
            check_block_intervals(&model.rigid().fields().dof_parentid, &blocks)?;
            let source = sources()?;
            let mut packed = vec![blocks.len() as i32];
            for block in &blocks {
                let kind = match block.kind {
                    super::super::MassFactorKind::Diagonal => 0,
                    super::super::MassFactorKind::Cholesky => 1,
                    super::super::MassFactorKind::ReverseLdl => 2,
                };
                packed.extend([block.start as i32, block.size as i32, kind]);
            }
            (
                model.rigid().kinematics().nbody(),
                blocks,
                session.upload(&packed)?,
                source,
            )
        };
        let matrix = MassMatrixPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let kernels = [
            SynchronousKernel::compile(session, &source[0], "resident_mass_factor")?,
            SynchronousKernel::compile(session, &source[1], "resident_mass_substitute")?,
        ];
        Ok(Self {
            matrix,
            nv,
            #[cfg(feature = "cuda-probe")]
            nbody,
            #[cfg(feature = "cuda-probe")]
            blocks,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernels,
        })
    }
    pub fn device(&self) -> usize {
        self.matrix.device()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<MassSolveData, TransferError> {
        let layout = MassSolveLayout::new(worlds, self.nv, rhs_count)?;
        let factor_layout = MassSolveLayout::new(worlds, self.nv, 1)?;
        let matrix = self.matrix.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, factor_layout, matrix);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |length, initial| {
                let mut values = crate::runtime::host_staging::<f32>(length)?;
                values.fill(-131072.0);
                values[4..length - 4].fill(initial);
                self.session.upload(&values)
            };
            Ok(MassSolveData {
                matrix,
                layout,
                factor_layout,
                factor_ready: false,
                ready: false,
                blocks: self.blocks.clone(),
                rhs: allocate(layout.rhs.total_elements() + 8, 0.0)?,
                factor: allocate(factor_layout.guarded, f32::NAN)?,
                output: allocate(layout.guarded, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut MassSolveData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_position(&self, data: &mut MassSolveData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        data.invalidate_factor();
        self.matrix.update_position(&mut data.matrix)
    }
    pub fn update_matrix(&self, data: &mut MassSolveData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        data.invalidate_factor();
        self.matrix.update_matrix(&mut data.matrix)
    }
    pub fn update_factor(&self, data: &mut MassSolveData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        data.invalidate_factor();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = data.matrix.matrix_input()?;
            // SAFETY: The same-plan CRB buffer has checked stride 10*nb+nv*nv.
            // Metadata partitions nv into validated intervals. Factor output uses
            // the checked one-RHS layout (3+nv*nv+2*nv) and 4+4 guards. All fields
            // are f32 and allocations are disjoint in one synchronized session.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &data.rhs,
                    input,
                    &mut data.factor,
                    [
                        self.nbody as u32,
                        self.nv as u32,
                        1,
                        data.layout.output.worlds() as u32,
                    ],
                )?;
            }
            data.check_factor()?;
            data.factor_ready = true;
            Ok(())
        }
    }
    /// 只读缓存因子；不再分解矩阵。
    pub fn update_solve(&self, data: &mut MassSolveData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        data.ready = false;
        if !data.factor_ready {
            return Err(TransferError::StageNotReady {
                stage: "mass_factor",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Err(error) = data.check_factor() {
                data.factor_ready = false;
                return Err(error);
            }
            // SAFETY: Read-only factor stride is 3+nv*nv+2*nv; guarded RHS stride
            // is nrhs*nv. Disjoint output stride is 3+nv*nv+nv+nrhs*nv. Layouts
            // bound all indices and metadata blocks, including zero-DOF states.
            // The fixed synchronized launch retains every same-session buffer.
            unsafe {
                self.kernels[1].launch(
                    &self.metadata,
                    &data.rhs,
                    &data.factor,
                    &mut data.output,
                    [
                        self.nbody as u32,
                        self.nv as u32,
                        data.layout.rhs_count as u32,
                        data.layout.output.worlds() as u32,
                    ],
                )?;
            }
            let mut values = crate::runtime::host_staging::<f32>(data.layout.guarded)?;
            data.output.read_range_into(0, &mut values)?;
            check_result(data.layout, &values)?;
            data.ready = true;
            Ok(())
        }
    }
}
