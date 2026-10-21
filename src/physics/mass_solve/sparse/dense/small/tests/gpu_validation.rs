//! Small校验GPU包装测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::SmallValidationKernel;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_small_validation_wrapper_preserves_zero_prefix_and_unused_semantics() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut metadata = session.upload(&[0i32]).unwrap();
    let unused = session.upload(&[7.0f32]).unwrap();
    let worlds = 257usize;
    let prefix = 3usize;
    let mut matrix_values = vec![-131072.0; prefix * worlds + 8];
    let matrix_end = matrix_values.len() - 4;
    matrix_values[4..matrix_end].fill(0.25);
    let status_values = vec![-131072.0; worlds + 8];
    let matrix = session.upload(&matrix_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();
    let kernel = SmallValidationKernel::compile(&session, prefix as u32).unwrap();

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
            .launch(&metadata, &matrix, &unused, &mut short_status, worlds,)
            .is_err()
    );
    let mut short_after = vec![0.0; short_status.len()];
    short_status.read_range_into(0, &mut short_after).unwrap();
    assert_eq!(short_after, status_values[..status_values.len() - 1]);
    let wrong_prefix_matrix = session
        .upload(&vec![-131072.0f32; (prefix + 1) * worlds + 8])
        .unwrap();
    assert!(
        kernel
            .launch(
                &metadata,
                &wrong_prefix_matrix,
                &unused,
                &mut status,
                worlds,
            )
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

    let zero_kernel = SmallValidationKernel::compile(&session, 0).unwrap();
    let zero_matrix = session.upload(&[-131072.0f32; 8]).unwrap();
    let mut zero_status = session.upload(&[-131072.0f32; 11]).unwrap();
    zero_kernel
        .launch(&metadata, &zero_matrix, &unused, &mut zero_status, 3)
        .unwrap();
    let mut zero_result = vec![0.0; zero_status.len()];
    zero_status.read_range_into(0, &mut zero_result).unwrap();
    assert_eq!(&zero_result[4..7], &[1.0, 1.0, 1.0]);
}
