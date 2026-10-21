//! 质量求解GPU因子分解与Cholesky测试。

#[cfg(feature = "cuda-probe")]
use super::probe::solve_device;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn device_factorization_rejects_indefinite_world_and_recovers() {
    let s = TransferSession::new(0).unwrap();
    let l = MassSolveLayout::new(3, 2, 1).unwrap();
    let mut state = vec![-131072.0; 50];
    for w in 0..3 {
        state[4 + 14 * w..4 + 14 * w + 10].fill(0.0);
        state[14 + 14 * w..18 + 14 * w].copy_from_slice(&[2.0, 0.0, 0.0, 3.0]);
    }
    state[28..32].copy_from_slice(&[1.0, 2.0, 2.0, 1.0]);
    let b = s.upload(&state).unwrap();
    assert_eq!(
        solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap_err(),
        TransferError::InvalidPivot {
            world: 1,
            dof: 0,
            value: -3.0
        }
    );
    for pivot in [-1.0 - 2.0f32.powi(-23), 0.0, f32::INFINITY, f32::NAN] {
        state[28..32].copy_from_slice(&[pivot, 0.0, 0.0, 3.0]);
        let b = s.upload(&state).unwrap();
        let TransferError::InvalidPivot { world, dof, value } =
            solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap_err()
        else {
            panic!("expected invalid pivot")
        };
        assert_eq!((world, dof), (1, 0));
        if pivot.is_nan() {
            assert!(value.is_nan());
        } else {
            assert_eq!(value.to_bits(), f64::from(pivot).to_bits());
        }
    }
    state[28..32].copy_from_slice(&[2.0, 0.0, 0.0, 3.0]);
    let b = s.upload(&state).unwrap();
    let out = solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap();
    assert_eq!(out.world(2).unwrap().solution, &[0.5, 1.0 / 3.0]);
}
#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn device_reciprocal_and_solution_overflow_do_not_poison_session() {
    let s = TransferSession::new(0).unwrap();
    let l = MassSolveLayout::new(1, 1, 1).unwrap();
    let mut state = vec![-131072.0; 19];
    state[4..14].fill(0.0);
    state[14] = 1e-39;
    let b = s.upload(&state).unwrap();
    assert!(matches!(
        solve_device(&s, 1, l, &b, &[1.0], factor_blocks(&[-1])),
        Err(TransferError::Input(InputError::NonFinite {
            field: "mass_solve_diagonal_inverse",
            index: 0
        }))
    ));
    state[14] = 1e-30;
    let b = s.upload(&state).unwrap();
    assert!(matches!(
        solve_device(&s, 1, l, &b, &[1e20], factor_blocks(&[-1])),
        Err(TransferError::Input(InputError::NonFinite {
            field: "mass_solve_output",
            ..
        }))
    ));
    state[14] = 2.0;
    let b = s.upload(&state).unwrap();
    assert_eq!(
        solve_device(&s, 1, l, &b, &[1.0], factor_blocks(&[-1]))
            .unwrap()
            .world(0)
            .unwrap()
            .solution,
        &[0.5]
    );
}

#[cfg(feature = "cuda-probe")]
fn reverse_test_block() -> Vec<MassFactorBlock> {
    vec![MassFactorBlock {
        start: 0,
        size: 2,
        kind: MassFactorKind::ReverseLdl,
    }]
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn device_cholesky_rejects_bad_pivots_and_recovers() {
    let s = TransferSession::new(0).unwrap();
    let l = MassSolveLayout::new(1, 2, 1).unwrap();
    let mut state = vec![-131072.0; 22];
    state[4..14].fill(0.0);
    for pivot in [0.0, -1.0, f32::INFINITY, f32::NAN] {
        state[14..18].copy_from_slice(&[pivot, 0.0, 0.0, 3.0]);
        let b = s.upload(&state).unwrap();
        assert!(matches!(
            solve_device(&s, 1, l, &b, &[1.0; 2], factor_blocks(&[-1, 0])),
            Err(TransferError::InvalidPivot {
                world: 0,
                dof: 0,
                ..
            })
        ));
    }
    state[14..18].copy_from_slice(&[1.0, 2.0, 2.0, 1.0]);
    let b = s.upload(&state).unwrap();
    assert_eq!(
        solve_device(&s, 1, l, &b, &[1.0; 2], factor_blocks(&[-1, 0])).unwrap_err(),
        TransferError::InvalidPivot {
            world: 0,
            dof: 1,
            value: -3.0
        }
    );
    state[14..18].copy_from_slice(&[4.0, 2.0, 2.0, 5.0]);
    let b = s.upload(&state).unwrap();
    let out = solve_device(&s, 1, l, &b, &[8.0, 12.0], factor_blocks(&[-1, 0])).unwrap();
    let w = out.world(0).unwrap();
    assert_eq!(w.ld, &[2.0, 1.0, 0.0, 2.0]);
    assert_eq!(w.diagonal_inverse, &[0.0, 0.0]);
    assert_eq!(w.solution, &[1.0, 2.0]);
}
