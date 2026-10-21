//! 混合求解布局与区域划分测试。

use super::layout::layout;
#[cfg(any(feature = "cuda-probe", test))]
use super::layout::metadata;
use super::tests_support::*;
#[cfg(any(feature = "cuda-probe", test))]
use super::validation_meta::StaticMixedSolveValidationMetadata;
use super::*;

#[test]
fn mixed_rhs_layout_rejects_zero_and_overflow_without_allocation() {
    for (w, n, r) in [
        (0, 86, 3),
        (1, 0, 3),
        (1, 86, 0),
        (usize::MAX, 86, 1),
        (1, usize::MAX, 3),
    ] {
        assert!(layout(w, n, r).is_err());
    }
    assert_eq!(layout(257, 86, 3).unwrap().total_elements(), 66306);
}
#[test]
fn packed_route_diagonals_and_small_groups_keep_distinct_regions() {
    let input = input();
    let m = metadata(&input).unwrap();
    let n = input.nv();
    let flags = &m[4..4 + n];
    assert_eq!(&m[..4], &[64, 90, 2201, 5]);
    assert!(flags[..65].iter().all(|&v| v == -1));
    assert_eq!(flags[85], -2);
    let diag = &m[4 + n..4 + 2 * n];
    assert_eq!(diag[65], 0);
    assert_eq!(diag[66], 3);
    assert_eq!(diag[85], 2290);
    let offsets = &m[4 + 2 * n..4 + 2 * n + 65];
    let count = offsets[64] as usize;
    let updates = &m[4 + 2 * n + 65..4 + 2 * n + 65 + 3 * count];
    assert!(
        updates
            .as_chunks::<3>()
            .0
            .iter()
            .all(|u| u[0] < 65 && u[1] < 65)
    );
    assert_eq!(&m[m.len() - 15..m.len() - 12], &[65, 2, 0]);
}
#[test]
fn mixed_solve_validation_metadata_proves_diag_and_inverse_regions() {
    let values = validation_metadata();
    assert!(StaticMixedSolveValidationMetadata::validate(&values, &[3, 10]).is_ok());
    assert!(StaticMixedSolveValidationMetadata::validate(&values[..9], &[3, 10]).is_err());

    let mut negative_diag = values.clone();
    negative_diag[7] = -1;
    assert!(StaticMixedSolveValidationMetadata::validate(&negative_diag, &[3, 10]).is_err());
    let mut large_diag = values.clone();
    large_diag[9] = 10;
    assert!(StaticMixedSolveValidationMetadata::validate(&large_diag, &[3, 10]).is_err());

    let mut signed_overflow = values.clone();
    signed_overflow[1] = i32::MAX;
    signed_overflow[2] = 1;
    assert!(StaticMixedSolveValidationMetadata::validate(&signed_overflow, &[3, 10]).is_err());
    let mut negative_inverse = values.clone();
    negative_inverse[1] = -5;
    assert!(StaticMixedSolveValidationMetadata::validate(&negative_inverse, &[3, 10]).is_err());
    let mut long_inverse = values;
    long_inverse[1] = 8;
    long_inverse[2] = 0;
    assert!(StaticMixedSolveValidationMetadata::validate(&long_inverse, &[3, 10]).is_err());
}
