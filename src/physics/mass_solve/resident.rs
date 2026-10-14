//! 常驻分解与多右端项严格辅助。
//! 完整G03仍待验收。
#[cfg(feature = "cuda-probe")]
use super::{MassFactorBlock, check_block_intervals, check_result, factor_blocks};
use super::{MassSolveLayout, MassSolveOutput};
use crate::diagnostics::{ProbeError, TransferError};
use crate::model::AttachedModelInput;
use crate::physics::{MassMatrixData, MassMatrixPlan};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 独立保留CRB、因子与回代内核。
pub struct MassSolvePlan {
    matrix: MassMatrixPlan,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    nbody: usize,
    #[cfg(feature = "cuda-probe")]
    blocks: Vec<MassFactorBlock>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}

/// RHS写入保留因子，只废弃解。
pub struct MassSolveData {
    matrix: MassMatrixData,
    layout: MassSolveLayout,
    #[cfg(feature = "cuda-probe")]
    factor_layout: MassSolveLayout,
    factor_ready: bool,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    blocks: Vec<MassFactorBlock>,
    #[cfg(feature = "cuda-probe")]
    rhs: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    factor: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;

    #[test]
    fn splits_only_stage_boundaries_and_preserves_frozen_arithmetic() {
        let generated = sources().unwrap();
        let canonical = super::super::MASS_SOLVE_CUDA.replace("\r\n", "\n");
        assert_eq!(
            generated,
            split_sources(&canonical.replace('\n', "\r\n")).unwrap()
        );
        assert!(split_sources(&canonical.replace("  if(n==0) return;\n", "")).is_err());
        assert!(
            split_sources(&canonical.replace("void rigid_mass_solve(", "void unknown(")).is_err()
        );
        assert!(generated[0].contains("void resident_mass_factor("));
        assert!(!generated[0].contains("for(unsigned r=0;r<nr;r++)"));
        assert!(generated[1].contains("const float* ld=saved+3;"));
        assert!(!generated[1].contains("sqrtf("));
        assert!(!generated[1].contains("rsqrt.approx"));
        let (_, solve) = canonical.split_once("  if(n==0) return;\n").unwrap();
        assert!(generated[1].ends_with(&solve.replace(
            "  const float* b=rhs+(unsigned long long)w*nr*n;",
            "  const float* b=rhs+4+(unsigned long long)w*nr*n;"
        )));
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn cached_factor_survives_poisoned_matrix_and_invalid_worlds_recover() {
        let session = TransferSession::new(0).unwrap();
        let m = crate::physics::tests::test_model(Some(3));
        let m = AttachedModelInput::new(m, crate::model::AttachedFields::default()).unwrap();
        let plan = MassSolvePlan::new(&session, m).unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_rhs(&[1.0, 2.0, 3.0, -1.0, -2.0, -3.0]).unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        let mut factors = vec![0.0; data.factor_layout.guarded];
        data.factor.read_range_into(0, &mut factors).unwrap();
        data.matrix.poison_matrix_for_test(1).unwrap();
        data.write_world_rhs(1, &[-2.0, -1.0, -3.0]).unwrap();
        plan.update_solve(&mut data).unwrap();
        let mut after = vec![0.0; factors.len()];
        data.factor.read_range_into(0, &mut after).unwrap();
        assert!(
            factors
                .iter()
                .zip(&after)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
        let solved = data.readback().unwrap();
        assert_eq!(
            solved.world(0).unwrap().solution,
            before.world(0).unwrap().solution
        );
        assert_eq!(
            solved.world(1).unwrap().solution[0],
            before.world(1).unwrap().solution[1]
        );
        assert_eq!(
            solved.world(1).unwrap().solution[1],
            before.world(1).unwrap().solution[0]
        );
        assert!(matches!(
            plan.update_factor(&mut data),
            Err(TransferError::InvalidPivot { world: 1, .. })
        ));
        assert!(!data.factor_ready);
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        plan.update(&mut data).unwrap();
        data.readback().unwrap();
        for target in 0..3 {
            match target {
                0 => data.factor.write_range(0, &[0.0]).unwrap(),
                1 => data.rhs.write_range(0, &[0.0]).unwrap(),
                _ => data.output.write_range(0, &[0.0]).unwrap(),
            }
            assert!(data.readback().is_err());
            match target {
                0 => data.factor.write_range(0, &[-131072.0]).unwrap(),
                1 => data.rhs.write_range(0, &[-131072.0]).unwrap(),
                _ => data.output.write_range(0, &[-131072.0]).unwrap(),
            }
        }
        data.write_rhs(&[f32::MAX; 6]).unwrap();
        // Whether overflow occurs depends on the model; finite inputs remain legal.
        let _ = plan.update_solve(&mut data);
        data.write_rhs(&[0.0; 6]).unwrap();
        plan.update_solve(&mut data).unwrap();
        data.readback().unwrap();
    }

    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn independent_diagonal_65_fixture_checks_cached_reverse_ldl_without_upstream_claim() {
        let session = TransferSession::new(0).unwrap();
        let source = sources().unwrap();
        let factor =
            SynchronousKernel::compile(&session, &source[0], "resident_mass_factor").unwrap();
        let solve =
            SynchronousKernel::compile(&session, &source[1], "resident_mass_substitute").unwrap();
        let n = 65usize;
        let layout = MassSolveLayout::new(2, n, 3).unwrap();
        let fl = MassSolveLayout::new(2, n, 1).unwrap();
        let metadata = session.upload(&[1, 0, 65, 2]).unwrap();
        let mut matrix = vec![0.0f32; 8 + 2 * (10 + n * n)];
        matrix[..4].fill(-131072.0);
        let end = matrix.len() - 4;
        matrix[end..].fill(-131072.0);
        for w in 0..2 {
            for i in 0..n {
                matrix[4 + w * (10 + n * n) + 10 + i * n + i] = 2.0;
            }
        }
        let matrix = session.upload(&matrix).unwrap();
        let mut rhs = vec![0.0f32; layout.rhs.total_elements() + 8];
        rhs[..4].fill(-131072.0);
        let end = rhs.len() - 4;
        rhs[end..].fill(-131072.0);
        for (i, v) in rhs[4..end].iter_mut().enumerate() {
            *v = (i % 17) as f32;
        }
        let rhs = session.upload(&rhs).unwrap();
        let init = |length| {
            let mut v = vec![f32::NAN; length];
            v[..4].fill(-131072.0);
            v[length - 4..].fill(-131072.0);
            session.upload(&v).unwrap()
        };
        let mut f = init(fl.guarded);
        let mut out = init(layout.guarded);
        // SAFETY: Independent SPD fixture is exactly 2*I_65. Metadata has one
        // contiguous 65-DOF reverse-LDL block. Both worlds and all guarded strides
        // match the private f32 ABI; allocations are disjoint in one session.
        unsafe {
            factor
                .launch(&metadata, &rhs, &matrix, &mut f, [1, 65, 1, 2])
                .unwrap();
            solve
                .launch(&metadata, &rhs, &f, &mut out, [1, 65, 3, 2])
                .unwrap();
        }
        let mut values = vec![0.0; layout.guarded];
        out.read_range_into(0, &mut values).unwrap();
        check_result(layout, &values).unwrap();
        let snapshot = MassSolveOutput {
            layout,
            values,
            blocks: vec![MassFactorBlock {
                start: 0,
                size: n,
                kind: super::super::MassFactorKind::ReverseLdl,
            }],
        };
        for w in 0..2 {
            let view = snapshot.world(w).unwrap();
            for i in 0..n {
                assert_eq!(view.ld[i * n + i], 2.0);
                assert_eq!(view.diagonal_inverse[i], 0.5);
            }
            for (i, &v) in view.solution.iter().enumerate() {
                assert_eq!(v, ((w * 3 * n + i) % 17) as f32 * 0.5);
            }
        }
    }
}

// Retain the existing frozen-derived f32 arithmetic verbatim.
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for license and warranty terms.
#[cfg(feature = "cuda-probe")]
fn sources() -> Result<[String; 2], TransferError> {
    split_sources(super::MASS_SOLVE_CUDA)
}

#[cfg(feature = "cuda-probe")]
fn split_sources(source: &str) -> Result<[String; 2], TransferError> {
    const CUT: &str = "  if(n==0) return;\n";
    const ENTRY: &str = "void rigid_mass_solve(";
    const INIT: &str = "  for(unsigned i=0;i<n;i++) inv[i]=0.0f;";
    // Windows检出可能改换行；先统一再核对唯一锚点。
    let source = source.replace("\r\n", "\n");
    if source.matches(CUT).count() != 1
        || source.matches(ENTRY).count() != 1
        || source.matches(INIT).count() != 1
    {
        return Err(ProbeError::InvalidArgument("mass_solve_source_split").into());
    }
    let (factor, solve) = source.split_once(CUT).unwrap();
    let factor = format!(
        "{}\n}}\n",
        factor.replace(ENTRY, "void resident_mass_factor(").replace(
            INIT,
            &format!("{INIT}\n  for(unsigned i=0;i<n;i++) x[i]=0.0f;")
        )
    );
    // Factors use the existing single-RHS layout. Its unused x slot is zero.
    // Solutions have their own variable-RHS layout and never overwrite factors.
    let prefix = r#"
extern "C" __global__ void resident_mass_substitute(const int* blocks,const float* rhs,
    const float* state,float* output,unsigned nb,unsigned n,unsigned nr,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const float* saved=state+4+(unsigned long long)w*(3ull+n*n+2ull*n);
  float* status=output+4+(unsigned long long)w*(3ull+n*n+n+nr*n);
  for(unsigned i=0;i<3+n*n+n;++i) status[i]=saved[i];
  const float* ld=saved+3;const float* inv=ld+n*n;float* x=status+3+n*n+n;
  if(n==0) return;
"#;
    let old_rhs = "  const float* b=rhs+(unsigned long long)w*nr*n;";
    if solve.matches(old_rhs).count() != 1 {
        return Err(ProbeError::InvalidArgument("mass_solve_rhs_split").into());
    }
    let solve = format!(
        "{prefix}{}",
        solve.replace(
            old_rhs,
            "  const float* b=rhs+4+(unsigned long long)w*nr*n;"
        )
    )
    .replace("\r\n", "\n");
    Ok([factor, solve])
}

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
                    super::MassFactorKind::Diagonal => 0,
                    super::MassFactorKind::Cholesky => 1,
                    super::MassFactorKind::ReverseLdl => 2,
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

impl MassSolveData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn rhs_count(&self) -> usize {
        self.layout.rhs_count
    }
    fn invalidate_factor(&mut self) {
        self.factor_ready = false;
        self.ready = false;
    }
    fn after_position_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate_factor();
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_qpos(values);
        self.after_position_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_world_qpos(world, values);
        self.after_position_write(result)
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.layout.check_rhs(values)?;
        self.write_checked_rhs(0, values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.rhs.world_elements(world)?;
        MassSolveLayout::new(1, self.nv(), self.rhs_count())?.check_rhs(values)?;
        self.write_checked_rhs(range.start, values)
    }
    fn write_checked_rhs(&mut self, offset: usize, values: &[f32]) -> Result<(), TransferError> {
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, values);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.rhs.write_range(4 + offset, values)
    }
    #[cfg(feature = "cuda-probe")]
    fn check_factor(&self) -> Result<(), TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.factor_layout.guarded)?;
        self.factor.read_range_into(0, &mut values)?;
        check_result(self.factor_layout, &values)
    }
    pub fn readback(&self) -> Result<MassSolveOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "resident_mass_solve",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.matrix.readback()?;
            self.check_factor()?;
            let mut rhs =
                crate::runtime::host_staging::<f32>(self.layout.rhs.total_elements() + 8)?;
            self.rhs.read_range_into(0, &mut rhs)?;
            crate::physics::check_device_values(&rhs, "resident_mass_solve_rhs")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            check_result(self.layout, &values)?;
            Ok(MassSolveOutput {
                layout: self.layout,
                values,
                blocks: self.blocks.clone(),
            })
        }
    }
}
