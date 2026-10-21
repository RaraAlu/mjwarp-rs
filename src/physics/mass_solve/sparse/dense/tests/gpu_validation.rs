//! 稠密GPU校验包装与产品路径测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_dense_validation_wrapper_preserves_zero_prefix_and_unused_semantics() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut metadata = session.upload(&[0i32]).unwrap();
    let unused = session.upload(&[7.0f32]).unwrap();
    let worlds = 257usize;
    let nc = 3usize;
    let mut matrix_values = vec![-131072.0; nc * worlds + 8];
    let matrix_end = matrix_values.len() - 4;
    matrix_values[4..matrix_end].fill(0.25);
    let status_values = vec![-131072.0; worlds + 8];
    let matrix = session.upload(&matrix_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();
    let kernel = DenseValidationKernel::compile(&session, nc as u32, 0).unwrap();

    kernel
        .launch(&metadata, &matrix, &unused, &mut status, worlds)
        .unwrap();
    let mut result = vec![0.0; status.len()];
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &status_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &status_values[status_values.len() - 4..]
    );
    assert!(result[4..4 + worlds].iter().all(|&value| value == 1.0));
    let stable = result.clone();

    metadata.write_range(0, &[-1]).unwrap();
    kernel
        .launch(&metadata, &matrix, &unused, &mut status, worlds)
        .unwrap();
    let short_matrix = session
        .upload(&matrix_values[..matrix_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &short_matrix, &unused, &mut status, worlds)
            .is_err()
    );
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    let mut short_status = session
        .upload(&status_values[..status_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &matrix, &unused, &mut short_status, worlds)
            .is_err()
    );
    let mut short_after = vec![0.0; short_status.len()];
    short_status.read_range_into(0, &mut short_after).unwrap();
    assert_eq!(short_after, status_values[..status_values.len() - 1]);

    let wrong_nc_matrix = session
        .upload(&vec![-131072.0f32; (nc + 1) * worlds + 8])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &wrong_nc_matrix, &unused, &mut status, worlds,)
            .is_err()
    );
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    let foreign_unused = foreign.upload(&[7.0f32]).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &matrix, &foreign_unused, &mut status, worlds),
        Err(TransferError::SessionMismatch)
    ));
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);
}

#[test]
#[ignore = "requires NVIDIA/NVRTC; dense validation product-call purpose"]
fn gpu_dense_validation_real_update_rejects_before_gather_and_recovers() {
    let start = std::time::Instant::now();
    let refs = input::reference();
    let model = &refs["models"][3];
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassDensePlan::new(&session, input::native(model)).unwrap();
    let mut data = plan.create_data(2).unwrap();
    let matrix = input::matrix(&model["cases"][0]);
    data.write_matrix(&matrix.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let bits = |values: &[f32]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let mut stable = vec![0.0; data.dense.len()];
    data.dense.read_range_into(0, &mut stable).unwrap();

    data.matrix.write_range(0, &[0.0]).unwrap();
    assert!(plan.update(&mut data).is_err());
    assert!(!data.ready);
    assert!(data.readback().is_err());
    let mut rejected = vec![0.0; data.dense.len()];
    data.dense.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(bits(&stable), bits(&rejected));

    data.matrix.write_range(0, &[-131072.0]).unwrap();
    plan.update(&mut data).unwrap();
    assert!(data.readback().is_ok());
    println!(
        "dense_validation_private worlds=2 recovered=true phase_ms={} status_capacity_bytes={}",
        start.elapsed().as_millis(),
        data.status.len() * std::mem::size_of::<f32>()
    );
}
