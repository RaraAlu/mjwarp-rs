//! 常驻混合计划拓扑校验与构造、更新入口。

#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedGatherKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedGatherMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedValidationKernel;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_dimension;
use super::*;

pub(super) fn topology(
    model: &AttachedModelInput,
    input: &SparseMassInput,
) -> Result<(), InputError> {
    if model.rigid().kinematics().nv() != input.nv()
        || model.rigid().fields().dof_parentid != input.fields().dof_parentid
    {
        return Err(InputError::InvalidTopology {
            field: "resident_mixed_topology",
            index: 0,
            reason: "CSR_and_inertial_dof_topology_must_match",
        });
    }
    Ok(())
}
impl SparseMassMixedResidentPlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        topology(&model, &input)?;
        #[cfg(feature = "cuda-probe")]
        let nbody = model.rigid().kinematics().nbody();
        // The mixed constructor rejects Tile before a physics producer exists.
        let solve = SparseMassMixedSolvePlan::new(session, input)?;
        let matrix = MassMatrixPlan::new(session, model)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (solve, matrix);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let f = solve.input().fields();
            let mut meta = f.m_rownnz.clone();
            meta.extend(&f.m_rowadr);
            meta.extend(&f.m_colind);
            if meta.len() > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "resident_mixed_metadata",
                }
                .into());
            }
            let n =
                resident_mixed_validation_dimension(solve.input().nv(), "resident_mixed_gather_n")?;
            let nc = resident_mixed_validation_dimension(
                solve.input().nc(),
                "resident_mixed_gather_nc",
            )?;
            let metadata = ResidentMixedGatherMetadataBuffer::upload(session, meta, n, nc)?;
            let validation = ResidentMixedValidationKernel::compile(
                session,
                nbody,
                solve.input().nv(),
                solve.input().nc(),
            )?;
            let gather = ResidentMixedGatherKernel::compile(session, &metadata, nbody)?;
            Ok(Self {
                matrix,
                solve,
                metadata,
                validation,
                gather,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.solve.input()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<SparseMassMixedResidentData, TransferError> {
        let matrix = self.matrix.create_data(worlds)?;
        let solve = self.solve.create_data(worlds, rhs_count)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix, solve);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut status = crate::runtime::host_staging::<f32>(worlds + 8)?;
            status.fill(-131072.0);
            status[4..4 + worlds].fill(f32::NAN);
            Ok(SparseMassMixedResidentData {
                matrix,
                solve,
                gathered: false,
                status: self.solve.mass.session.upload(&status)?,
            })
        }
    }
    pub(super) fn check_data(
        &self,
        data: &SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        self.solve.mass.check_data(&data.solve.mass)
    }
    pub fn update(&self, data: &mut SparseMassMixedResidentData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_position(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_position(&mut data.matrix)
    }
    pub fn update_matrix(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_matrix(&mut data.matrix)
    }
    pub fn update_gather(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = data.matrix.matrix_input()?;
            let world_count = data.worlds();
            self.validation.launch(
                self.metadata.as_buffer(),
                source,
                source,
                &mut data.status,
                world_count,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "resident_mixed_source_status")?;
            for (world, &value) in status[4..4 + data.worlds()].iter().enumerate() {
                if value != 1.0 {
                    return Err(InputError::InvalidTopology {
                        field: "resident_mixed_source",
                        index: world,
                        reason: "invalid_CRB_or_dense_matrix",
                    }
                    .into());
                }
            }
            self.gather.launch(
                &self.metadata,
                source,
                source,
                &mut data.solve.mass.matrix,
                world_count,
            )?;
            let mut guards = [0.0f32; 8];
            data.solve
                .mass
                .matrix
                .read_range_into(0, &mut guards[..4])?;
            data.solve.mass.matrix.read_range_into(
                4 + data.solve.mass.sparse.matrix_layout.total_elements(),
                &mut guards[4..],
            )?;
            crate::physics::check_device_guards(&guards)?;
            // This captures the GPU-produced primary CSR once for both actual
            // factor producers. No host physical readback is uploaded again.
            self.solve.update_gather(&mut data.solve)?;
            data.gathered = true;
            Ok(())
        }
    }
    pub fn update_factor(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_mixed_gather",
            });
        }
        self.solve.update_factor(&mut data.solve)
    }
    pub fn update_solve(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_mixed_gather",
            });
        }
        self.solve.update_solve(&mut data.solve)
    }
}
