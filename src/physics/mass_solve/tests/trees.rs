//! 质量求解独立树分块测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::factor::check_block_intervals;
#[cfg(any(feature = "cuda-probe", test))]
use super::factor::factor_blocks;
use super::*;

#[test]
fn partitions_independent_trees_and_freezes_factor_formats() {
    assert!(factor_blocks(&[]).is_empty());
    let blocks = factor_blocks(&[-1, 0, -1]);
    assert!(check_block_intervals(&[-1, 0, -1], &blocks).is_ok());
    let interleaved = [-1, -1, 0];
    assert!(check_block_intervals(&interleaved, &factor_blocks(&interleaved)).is_err());
    assert_eq!(
        blocks,
        vec![
            MassFactorBlock {
                start: 0,
                size: 2,
                kind: MassFactorKind::Cholesky
            },
            MassFactorBlock {
                start: 2,
                size: 1,
                kind: MassFactorKind::Diagonal
            },
        ]
    );
    for (size, kind) in [
        (64, MassFactorKind::Cholesky),
        (65, MassFactorKind::ReverseLdl),
    ] {
        let parents: Vec<i32> = (0..size).map(|i| i - 1).collect();
        assert_eq!(factor_blocks(&parents)[0].kind, kind);
    }
}
