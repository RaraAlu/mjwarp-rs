use super::super::{MassSolveOutput, check_result};
use super::*;
use crate::diagnostics::TransferError;
use crate::model::AttachedModelInput;

#[test]
fn splits_only_stage_boundaries_and_preserves_frozen_arithmetic() {
    let generated = sources().unwrap();
    let canonical = super::super::MASS_SOLVE_CUDA.replace("\r\n", "\n");
    assert_eq!(
        generated,
        split_sources(&canonical.replace('\n', "\r\n")).unwrap()
    );
    assert!(split_sources(&canonical.replace("  if(n==0) return;\n", "")).is_err());
    assert!(split_sources(&canonical.replace("void rigid_mass_solve(", "void unknown(")).is_err());
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
    let factor = SynchronousKernel::compile(&session, &source[0], "resident_mass_factor").unwrap();
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
