//! 常驻混合GPU校验拒绝测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedValidationKernel;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::resident_mixed_validation_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_resident_mixed_validation_wrapper_rejects_before_launch() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let (nb, n, nc, worlds) = (2usize, 3usize, 5usize, 257usize);
    let dims = [nb as u32, n as u32, nc as u32, worlds as u32];
    let (source_len, status_len) = resident_mixed_validation_lengths(&dims).unwrap();
    let mut source_values = vec![-131072.0f32; source_len as usize];
    let source_end = source_values.len() - 4;
    source_values[4..source_end].fill(0.25);
    let status_values = vec![-131072.0f32; status_len as usize];
    let mut metadata = session.upload(&[17i32]).unwrap();
    let mut unused = session.upload(&[7.0f32]).unwrap();
    let source = session.upload(&source_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();
    let kernel = ResidentMixedValidationKernel::compile(&session, nb, n, nc).unwrap();

    kernel
        .launch(&metadata, &source, &unused, &mut status, worlds)
        .unwrap();
    let mut result = vec![0.0; status.len()];
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &status_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &status_values[status_values.len() - 4..]
    );
    assert!(result[4..4 + worlds].iter().all(|&value| value == 1.0));

    metadata.write_range(0, &[-19]).unwrap();
    unused.write_range(0, &[f32::NAN]).unwrap();
    kernel
        .launch(&metadata, &source, &unused, &mut status, worlds)
        .unwrap();
    let stable = result.clone();
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    let short_source = session
        .upload(&source_values[..source_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &short_source, &unused, &mut status, worlds)
            .is_err()
    );
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    let mut short_status = session
        .upload(&status_values[..status_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &source, &unused, &mut short_status, worlds)
            .is_err()
    );
    let mut short_after = vec![0.0; short_status.len()];
    short_status.read_range_into(0, &mut short_after).unwrap();
    assert_eq!(short_after, status_values[..status_values.len() - 1]);

    let foreign_unused = foreign.upload(&[7.0f32]).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &source, &foreign_unused, &mut status, worlds),
        Err(TransferError::SessionMismatch)
    ));
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    assert!(
        kernel
            .launch_dims(
                &metadata,
                &source,
                &unused,
                &mut status,
                [dims[0] + 1, dims[1], dims[2], dims[3]],
            )
            .is_err()
    );
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(result, stable);

    let mut source_after = vec![0.0; source.len()];
    source.read_range_into(0, &mut source_after).unwrap();
    assert_eq!(
        source_after
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        source_values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
}
