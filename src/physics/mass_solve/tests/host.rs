//! 质量求解主机维度、视图与状态测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::factor::factor_blocks;
#[cfg(any(feature = "cuda-probe", test))]
use super::probe::check_result;
use super::*;

#[test]
fn rejects_solve_dimensions_square_and_rhs_capacity() {
    for (w, n, r) in [
        (0, 1, 1),
        (usize::MAX, 1, 1),
        (1, 1, 0),
        (1, 1, usize::MAX),
        (1, usize::MAX, 1),
        (1, 46341, 1),
        (1, 1, i32::MAX as usize),
    ] {
        assert!(MassSolveLayout::new(w, n, r).is_err());
    }
    assert!(MassSolveLayout::new(1, 0, 3).is_ok());
    assert!(MassSolveLayout::new(1, 46340, 1).is_err());
    assert!(MassSolveLayout::new(1, 46339, 1).is_ok());
    let layout = MassSolveLayout::new(513, 2, 3).unwrap();
    assert_eq!(layout.output.total_bytes(), 513 * 15 * size_of::<f32>());
    assert_eq!(layout.rhs.total_bytes(), 513 * 6 * size_of::<f32>());
}
#[test]
fn rejects_rhs_length_and_nonfinite_values() {
    let l = MassSolveLayout::new(2, 2, 3).unwrap();
    assert!(matches!(
        l.check_rhs(&[0.0]),
        Err(InputError::LengthMismatch { expected: 12, .. })
    ));
    let mut rhs = [0.0; 12];
    rhs[7] = f32::INFINITY;
    assert_eq!(
        l.check_rhs(&rhs),
        Err(InputError::NonFinite {
            field: "mass_solve_rhs",
            index: 7
        })
    );
    assert!(l.check_rhs(&[0.0; 12]).is_ok());
}
#[test]
fn solve_views_skip_status_and_separate_worlds_rhs_and_factors() {
    let l = MassSolveLayout::new(2, 2, 3).unwrap();
    let out = MassSolveOutput {
        layout: l,
        values: (0..l.guarded).map(|n| n as f32).collect(),
        blocks: factor_blocks(&[-1, 0]),
    };
    assert_eq!(out.worlds(), 2);
    assert_eq!(out.nv(), 2);
    assert_eq!(out.rhs_count(), 3);
    let w = out.world(1).unwrap();
    assert_eq!(w.ld, &[22.0, 23.0, 24.0, 25.0]);
    assert_eq!(w.diagonal_inverse, &[26.0, 27.0]);
    assert_eq!(w.solution, &[28.0, 29.0, 30.0, 31.0, 32.0, 33.0]);
    assert!(out.world(2).is_err());
    let l = MassSolveLayout::new(1, 0, 3).unwrap();
    let out = MassSolveOutput {
        layout: l,
        values: vec![0.0; l.guarded],
        blocks: vec![],
    };
    let w = out.world(0).unwrap();
    assert!(w.ld.is_empty() && w.diagonal_inverse.is_empty() && w.solution.is_empty());
}
#[test]
fn statuses_precede_partial_payload_but_never_skip_guards() {
    let l = MassSolveLayout::new(2, 2, 1).unwrap();
    let mut v = vec![f32::NAN; l.guarded];
    v[..4].fill(-131072.0);
    v[l.guarded - 4..].fill(-131072.0);
    v[4..7].copy_from_slice(&[0.0, -1.0, 0.0]);
    let start = 4 + l.output.elements_per_world();
    v[start..start + 3].copy_from_slice(&[1.0, 0.0, -3.0]);
    assert_eq!(
        check_result(l, &v),
        Err(TransferError::InvalidPivot {
            world: 1,
            dof: 0,
            value: -3.0
        })
    );
    v[0] = 0.0;
    assert!(matches!(
        check_result(l, &v),
        Err(TransferError::Backend(ProbeError::Mismatch {
            index: 0,
            ..
        }))
    ));
}
