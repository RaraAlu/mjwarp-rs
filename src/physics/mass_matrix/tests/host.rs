//! 质量矩阵布局与宿主视图测试。

use super::*;

#[test]
fn rejects_mass_matrix_dimensions_square_capacity_and_kernel_indices() {
    for dims in [
        (0, 1, 0),
        (1, 0, 0),
        (usize::MAX, 1, 0),
        (1, usize::MAX, 0),
        (1, 1, usize::MAX),
        (1, 1, 46341),
        (1, i32::MAX as usize, 1),
    ] {
        assert!(MassMatrixLayout::new(dims.0, dims.1, dims.2).is_err());
    }
    assert!(MassMatrixLayout::new(1, 1, 46340).is_ok());
}
#[test]
fn mass_matrix_views_separate_worlds_inertias_and_dense_rows() {
    let layout = MassMatrixLayout::new(2, 2, 3).unwrap();
    let out = MassMatrixOutput {
        layout,
        values: (0..66).map(|n| n as f32).collect(),
    };
    assert_eq!(out.worlds(), 2);
    assert_eq!(out.nbody(), 2);
    assert_eq!(out.nv(), 3);
    let w = out.world(1).unwrap();
    assert_eq!(w.crb.len(), 20);
    assert_eq!(w.crb[0], 33.0);
    assert_eq!(
        w.matrix,
        &[53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0]
    );
    assert!(out.world(2).is_err());
    let out = MassMatrixOutput {
        layout: MassMatrixLayout::new(1, 1, 0).unwrap(),
        values: vec![0.0; 18],
    };
    assert!(out.world(0).unwrap().matrix.is_empty());
}
#[test]
fn preflight_keeps_existing_com_and_state_rejections() {
    let m = super::super::tests::test_model(Some(3));
    assert!(super::super::check_com_position(&m, 2, &[0.0]).is_err());
    assert!(super::super::check_com_position(&m, 2, &[0.0, f32::NAN]).is_err());
    assert!(super::super::check_com_position(&m, 2, &[0.0, 0.5]).is_ok());
}
