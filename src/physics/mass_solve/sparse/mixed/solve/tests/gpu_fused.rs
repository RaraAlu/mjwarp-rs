//! 混合求解GPU融合分组与恢复测试。

#[cfg(feature = "cuda-probe")]
use super::fused_meta::MixedSolveFusedDims;
#[cfg(feature = "cuda-probe")]
use super::fused_spec::mixed_solve_fused_lengths;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_mixed_solve_fused_wrapper_checks_groups_spans_and_recovery() {
    let refs = samples::reference();
    let model = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
    let mut data = plan.create_data(1, 129).unwrap();
    data.write_matrix(&samples::matrix(model, 0)).unwrap();
    data.write_rhs(&samples::rhs(model, 0, 129)).unwrap();
    plan.update(&mut data).unwrap();
    let expected = samples::solution(model, 0, 129);
    let actual = data.readback().unwrap();
    for (&value, &reference) in actual.world(0).unwrap().solution.iter().zip(&expected) {
        assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
    }

    let bits = |buffer: &TransferBuffer<f32>| {
        let mut values = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let stable = bits(&data.output);
    let dimensions = plan.solve.dimensions(129, 1).unwrap();
    data.ready = false;
    plan.metadata.write_for_test(0, -1).unwrap();
    assert!(
        plan.solve
            .launch(
                &plan.metadata,
                &data.rhs,
                &data.mass.output,
                &mut data.output,
                dimensions,
            )
            .is_err()
    );
    assert!(!data.ready);
    assert!(data.readback().is_err());
    assert_eq!(bits(&data.output), stable);
    plan.metadata.restore_for_test().unwrap();

    let fused_dims = MixedSolveFusedDims {
        abi: dimensions,
        metadata_elements: plan.metadata.element_count().unwrap(),
    };
    let (_, rhs_len, factor_len, output_len) = mixed_solve_fused_lengths(&fused_dims).unwrap();
    let short_rhs = session.upload(&vec![0.0f32; rhs_len as usize - 1]).unwrap();
    let long_rhs = session.upload(&vec![0.0f32; rhs_len as usize + 1]).unwrap();
    for candidate in [&short_rhs, &long_rhs] {
        assert!(
            plan.solve
                .launch(
                    &plan.metadata,
                    candidate,
                    &data.mass.output,
                    &mut data.output,
                    dimensions,
                )
                .is_err()
        );
    }
    let short_factor = session
        .upload(&vec![0.0f32; factor_len as usize - 1])
        .unwrap();
    let long_factor = session
        .upload(&vec![0.0f32; factor_len as usize + 1])
        .unwrap();
    for candidate in [&short_factor, &long_factor] {
        assert!(
            plan.solve
                .launch(
                    &plan.metadata,
                    &data.rhs,
                    candidate,
                    &mut data.output,
                    dimensions,
                )
                .is_err()
        );
    }
    let mut short_output = session
        .upload(&vec![0.0f32; output_len as usize - 1])
        .unwrap();
    let mut long_output = session
        .upload(&vec![0.0f32; output_len as usize + 1])
        .unwrap();
    for candidate in [&mut short_output, &mut long_output] {
        assert!(
            plan.solve
                .launch(
                    &plan.metadata,
                    &data.rhs,
                    &data.mass.output,
                    candidate,
                    dimensions,
                )
                .is_err()
        );
    }
    let foreign_factor = foreign.upload(&vec![0.0f32; factor_len as usize]).unwrap();
    assert!(matches!(
        plan.solve.launch(
            &plan.metadata,
            &data.rhs,
            &foreign_factor,
            &mut data.output,
            dimensions,
        ),
        Err(TransferError::SessionMismatch)
    ));
    assert!(plan.solve.dimensions(usize::MAX, 1).is_err());
    assert!(plan.solve.dimensions(65536, 65536).is_err());
    assert_eq!(bits(&data.output), stable);

    plan.update_solve(&mut data).unwrap();
    let recovered = data.readback().unwrap();
    for (&value, &reference) in recovered.world(0).unwrap().solution.iter().zip(&expected) {
        assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
    }
}
