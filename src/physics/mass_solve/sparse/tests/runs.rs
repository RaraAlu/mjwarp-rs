//! 稀疏质量矩阵与守卫测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC; new sparse buffer fault purpose"]
fn matrix_write_is_bit_isolated_and_both_guards_reject_corruption() {
    let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/sparse-mass-factor/native.json"
    )))
    .unwrap();
    let m = &refs["models"][2];
    let ints = |name: &str| {
        m[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as i32)
            .collect()
    };
    let input = SparseMassInput::new(SparseMassFields {
        dof_parentid: ints("dof_parentid"),
        m_rownnz: ints("m_rownnz"),
        m_rowadr: ints("m_rowadr"),
        m_colind: ints("m_colind"),
        tree_dofadr: ints("tree_dofadr"),
        tree_dofnum: ints("tree_dofnum"),
    })
    .unwrap();
    let nc = input.nc();
    let values = |case: usize| {
        m["cases"][case]["matrix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<_>>()
    };
    let session = TransferSession::new(0).unwrap();
    let compact = SparseMassInput::new(SparseMassFields {
        dof_parentid: vec![-1],
        m_rownnz: vec![1],
        m_rowadr: vec![0],
        m_colind: vec![0],
        tree_dofadr: vec![0],
        tree_dofnum: vec![1],
    })
    .unwrap();
    assert!(SparseMassFactorPlan::new(&session, compact).is_err());
    let mut plan = SparseMassFactorPlan::new(&session, input).unwrap();
    let mut data = plan.create_data(3).unwrap();
    data.write_matrix(&values(0).repeat(3)).unwrap();
    let mut before = vec![0.0; 3 * nc + 8];
    data.matrix.read_range_into(0, &mut before).unwrap();
    data.write_world_matrix(1, &values(1)).unwrap();
    let mut after = vec![0.0; before.len()];
    data.matrix.read_range_into(0, &mut after).unwrap();
    for index in (0..4 + nc).chain(4 + 2 * nc..after.len()) {
        assert_eq!(before[index].to_bits(), after[index].to_bits());
    }
    assert!(
        before[4 + nc..4 + 2 * nc]
            .iter()
            .zip(&after[4 + nc..4 + 2 * nc])
            .any(|(a, b)| a.to_bits() != b.to_bits())
    );
    let worlds = data.worlds();
    plan.copy
        .launch(
            &plan.metadata,
            &data.matrix,
            &data.matrix,
            &mut data.output,
            worlds,
        )
        .unwrap();
    assert_copy_matches_matrix(&data);
    assert!(!data.ready);
    let foreign = TransferSession::new(0).unwrap();
    let foreign_matrix = foreign.upload(&vec![0.0f32; data.matrix.len()]).unwrap();
    assert!(matches!(
        plan.copy.launch(
            &plan.metadata,
            &foreign_matrix,
            &foreign_matrix,
            &mut data.output,
            worlds,
        ),
        Err(TransferError::SessionMismatch)
    ));
    assert_copy_matches_matrix(&data);
    assert!(!data.ready);
    let mut before_update_reject = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut before_update_reject)
        .unwrap();
    assert!(matches!(
        plan.update.launch(
            &plan.metadata,
            &foreign_matrix,
            &foreign_matrix,
            &mut data.output,
            worlds,
            0,
        ),
        Err(TransferError::SessionMismatch)
    ));
    let mut after_update_reject = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut after_update_reject)
        .unwrap();
    assert_eq!(
        before_update_reject
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        after_update_reject
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    let first = plan.metadata.values[0];
    plan.metadata.write_for_test(0, first + 1).unwrap();
    assert!(
        plan.update
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
                0,
            )
            .is_err()
    );
    let mut after_stale_reject = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut after_stale_reject)
        .unwrap();
    assert_eq!(
        after_update_reject
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        after_stale_reject
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    plan.metadata.restore_for_test().unwrap();
    plan.copy
        .launch(
            &plan.metadata,
            &data.matrix,
            &data.matrix,
            &mut data.output,
            worlds,
        )
        .unwrap();
    for level in (0..plan.input.qld_level_offsets().len() - 1).rev() {
        plan.update
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
                level,
            )
            .unwrap();
    }
    let mut before_diag_reject = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut before_diag_reject)
        .unwrap();
    assert!(matches!(
        plan.diag.launch(
            &plan.metadata,
            &foreign_matrix,
            &foreign_matrix,
            &mut data.output,
            worlds,
        ),
        Err(TransferError::SessionMismatch)
    ));
    let mut after_diag_foreign = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut after_diag_foreign)
        .unwrap();
    assert_eq!(
        before_diag_reject
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        after_diag_foreign
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    let first = plan.metadata.values[0];
    plan.metadata.write_for_test(0, first + 1).unwrap();
    assert!(
        plan.diag
            .launch(
                &plan.metadata,
                &data.matrix,
                &data.matrix,
                &mut data.output,
                worlds,
            )
            .is_err()
    );
    let mut after_diag_stale = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut after_diag_stale)
        .unwrap();
    assert_eq!(
        after_diag_foreign
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        after_diag_stale
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    plan.metadata.restore_for_test().unwrap();
    plan.diag
        .launch(
            &plan.metadata,
            &data.matrix,
            &data.matrix,
            &mut data.output,
            worlds,
        )
        .unwrap();
    let mut after_diag = vec![0.0; data.output.len()];
    data.output.read_range_into(0, &mut after_diag).unwrap();
    assert_diag_matches_factors_bitwise(&data, &before_diag_reject, &after_diag);
    assert!(!data.ready);

    plan.update_factor(&mut data).unwrap();
    assert_factor_matches_native(&data, m);
    let first = plan.metadata.values[0];
    plan.metadata.write_for_test(0, first + 1).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    assert!(!data.ready);
    assert!(data.readback().is_err());
    plan.metadata.restore_for_test().unwrap();
    plan.update_factor(&mut data).unwrap();
    assert_factor_matches_native(&data, m);
    data.output.write_range(0, &[0.0]).unwrap();
    assert!(data.readback().is_err());
    data.output.write_range(0, &[-131072.0]).unwrap();
    data.matrix.write_range(0, &[0.0]).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    assert!(data.readback().is_err());
    data.matrix.write_range(0, &[-131072.0]).unwrap();
    plan.update_factor(&mut data).unwrap();
    assert_factor_matches_native(&data, m);
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC; zero-stride sparse copy purpose"]
fn empty_sparse_mass_copy_preserves_zero_stride_and_guards() {
    let session = TransferSession::new(0).unwrap();
    let input = SparseMassInput::new(SparseMassFields::default()).unwrap();
    let plan = SparseMassFactorPlan::new(&session, input).unwrap();
    let mut data = plan.create_data(2).unwrap();
    plan.update_factor(&mut data).unwrap();
    let output = data.readback().unwrap();
    assert_eq!(output.worlds(), 2);
    assert!(output.world(0).unwrap().sparse_factors.is_empty());
    assert!(output.world(0).unwrap().diagonal_inverse.is_empty());
    assert!(output.world(1).unwrap().sparse_factors.is_empty());
    assert!(output.world(1).unwrap().diagonal_inverse.is_empty());
    let mut raw = vec![0.0; data.output.len()];
    data.output.read_range_into(0, &mut raw).unwrap();
    assert_eq!(raw.len(), 8);
    for value in raw {
        assert_eq!(value.to_bits(), (-131072.0f32).to_bits());
    }
}
