//! 混合求解GPU校验边界与拒绝测试。

#[cfg(feature = "cuda-probe")]
use super::fused_meta::StaticMixedSolveFusedMetadata;
#[cfg(feature = "cuda-probe")]
use super::kernels::MixedSolveValidationKernel;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::validation_buffer::MixedSolveValidationMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::StaticMixedSolveValidationMetadata;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::mixed_solve_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_mixed_solve_validation_wrapper_checks_tail_and_rejections() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut metadata =
        MixedSolveValidationMetadataBuffer::upload(&session, fused_metadata(), 4, 10).unwrap();
    let kernel = MixedSolveValidationKernel::compile(&session, &metadata).unwrap();
    let dims = kernel.dimensions(2, 257).unwrap();
    let (_, rhs_len, factor_len, status_len) = mixed_solve_validation_lengths(&dims).unwrap();
    let mut rhs_values = vec![-131072.0f32; rhs_len as usize];
    let rhs_end = rhs_values.len() - 4;
    rhs_values[4..rhs_end].fill(0.25);
    let mut factor_values = vec![-131072.0f32; factor_len as usize];
    let factor_end = factor_values.len() - 4;
    factor_values[4..factor_end].fill(1.0);
    let mut status_values = vec![-131072.0f32; status_len as usize];
    status_values[4..4 + 257].fill(f32::NAN);
    let rhs = session.upload(&rhs_values).unwrap();
    let factor = session.upload(&factor_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();

    kernel
        .launch(&metadata, &rhs, &factor, &mut status, dims)
        .unwrap();
    let mut result = vec![0.0; status.len()];
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &status_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &status_values[status_values.len() - 4..]
    );
    assert!(result[4..4 + 257].iter().all(|&value| value == 0.0));
    let stable = result.clone();

    metadata.write_for_test(7, 1).unwrap();
    assert!(
        kernel
            .launch(&metadata, &rhs, &factor, &mut status, dims)
            .is_err()
    );
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);
    metadata.restore_for_test().unwrap();
    kernel
        .launch(&metadata, &rhs, &factor, &mut status, dims)
        .unwrap();

    let short_rhs = session.upload(&rhs_values[..rhs_values.len() - 1]).unwrap();
    let mut long_rhs_values = rhs_values.clone();
    long_rhs_values.push(0.0);
    let long_rhs = session.upload(&long_rhs_values).unwrap();
    for candidate in [&short_rhs, &long_rhs] {
        assert!(
            kernel
                .launch(&metadata, candidate, &factor, &mut status, dims)
                .is_err()
        );
    }
    let short_factor = session
        .upload(&factor_values[..factor_values.len() - 1])
        .unwrap();
    let mut long_factor_values = factor_values.clone();
    long_factor_values.push(0.0);
    let long_factor = session.upload(&long_factor_values).unwrap();
    for candidate in [&short_factor, &long_factor] {
        assert!(
            kernel
                .launch(&metadata, &rhs, candidate, &mut status, dims)
                .is_err()
        );
    }
    let mut short_status = session
        .upload(&status_values[..status_values.len() - 1])
        .unwrap();
    let mut long_status_values = status_values.clone();
    long_status_values.push(-131072.0);
    let mut long_status = session.upload(&long_status_values).unwrap();
    for candidate in [&mut short_status, &mut long_status] {
        assert!(
            kernel
                .launch(&metadata, &rhs, &factor, candidate, dims)
                .is_err()
        );
    }
    let foreign_factor = foreign.upload(&factor_values).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &rhs, &foreign_factor, &mut status, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(kernel.dimensions(usize::MAX, 257).is_err());
    assert!(kernel.dimensions(1, u32::MAX as usize - 254).is_err());
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_mixed_solve_fused_rejects_in_bounds_small_overlap_without_state_change() {
    let refs = samples::reference();
    let model = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
    let mut data = plan.create_data(1, 129).unwrap();
    data.write_matrix(&samples::matrix(model, 0)).unwrap();
    data.write_rhs(&samples::rhs(model, 0, 129)).unwrap();
    plan.update(&mut data).unwrap();

    let f32_bits = |buffer: &TransferBuffer<f32>| {
        let mut values = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let i32_bits = |buffer: &TransferBuffer<i32>| {
        let mut values = vec![0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
    };
    let metadata_before = i32_bits(plan.metadata.as_buffer());
    let rhs_before = f32_bits(&data.rhs);
    let factor_before = f32_bits(&data.mass.output);
    let output_before = f32_bits(&data.output);
    let status_before = f32_bits(&data.status);
    let ready_before = data.ready;
    let factor_ready_before = data.mass.ready;
    let proof_span = plan.metadata.fused_checked.span();
    let proof_dims = *plan.metadata.fused_checked.dims();
    let proof_digest = plan.metadata.fused_checked.digest();

    let mut overlap = plan.metadata.values.clone();
    let n = usize::try_from(plan.metadata.n).unwrap();
    let levels = usize::try_from(overlap[0]).unwrap();
    let diagonal_end = 4 + 2 * n;
    let offsets_end = diagonal_end + levels + 1;
    let update_count = usize::try_from(overlap[offsets_end - 1]).unwrap();
    let small_base = offsets_end + 3 * update_count.max(1);
    assert!(overlap[3] >= 2);
    let first = [
        overlap[small_base],
        overlap[small_base + 1],
        overlap[small_base + 2],
    ];
    overlap[small_base + 3..small_base + 6].copy_from_slice(&first);

    let start = usize::try_from(first[0]).unwrap();
    let size = usize::try_from(first[1]).unwrap();
    let address = usize::try_from(first[2]).unwrap();
    let prefix = usize::try_from(overlap[1]).unwrap();
    assert!((2..=6).contains(&size));
    assert!(start + size <= n);
    assert!(address + size * size <= prefix);
    assert_eq!(
        &overlap[small_base..small_base + 3],
        &overlap[small_base + 3..small_base + 6]
    );
    assert!(
        overlap[4 + start..4 + start + size]
            .iter()
            .all(|&flag| flag == first[2])
    );
    assert!(
        StaticMixedSolveValidationMetadata::validate(
            &overlap,
            &[plan.metadata.n, plan.metadata.stride],
        )
        .is_ok()
    );
    assert!(matches!(
        StaticMixedSolveFusedMetadata::validate(&overlap, &[plan.metadata.n, plan.metadata.stride],),
        Err(LauncherError::InvalidMetadata("混合融合Small分区冲突"))
    ));

    let rejection = match MixedSolveValidationMetadataBuffer::upload(
        &session,
        overlap,
        n,
        usize::try_from(plan.metadata.stride).unwrap(),
    ) {
        Ok(_) => panic!("Small重叠元数据意外通过"),
        Err(error) => error,
    };
    assert!(matches!(
        rejection,
        TransferError::Input(InputError::InvalidTopology {
            field: "mixed_solve_fused_metadata",
            index: 0,
            reason: "invalid_or_stale_proof",
        })
    ));

    assert_eq!(
        plan.metadata.fused_checked.span().context(),
        proof_span.context()
    );
    assert_eq!(plan.metadata.fused_checked.span().addr(), proof_span.addr());
    assert_eq!(
        plan.metadata.fused_checked.span().bytes(),
        proof_span.bytes()
    );
    assert_eq!(*plan.metadata.fused_checked.dims(), proof_dims);
    assert_eq!(plan.metadata.fused_checked.digest(), proof_digest);
    assert_eq!(i32_bits(plan.metadata.as_buffer()), metadata_before);
    assert_eq!(f32_bits(&data.rhs), rhs_before);
    assert_eq!(f32_bits(&data.mass.output), factor_before);
    assert_eq!(f32_bits(&data.output), output_before);
    assert_eq!(f32_bits(&data.status), status_before);
    assert_eq!(data.ready, ready_before);
    assert_eq!(data.mass.ready, factor_ready_before);

    plan.update_solve(&mut data).unwrap();
    assert!(data.ready);
    let expected = samples::solution(model, 0, 129);
    let recovered = data.readback().unwrap();
    for (&value, &reference) in recovered.world(0).unwrap().solution.iter().zip(&expected) {
        assert!((f64::from(value) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
    }
}
