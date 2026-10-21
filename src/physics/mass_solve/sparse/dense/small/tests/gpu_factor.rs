//! Small因子GPU包装与路由测试。

#[cfg(feature = "cuda-probe")]
use super::buffer::SmallFactorMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::kernels::SmallFactorKernel;
use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_small_factor_wrapper_binds_real_spans_and_metadata() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut metadata = SmallFactorMetadataBuffer::upload(&session, vec![0, 0, 2], 4).unwrap();
    let wrong_metadata = SmallFactorMetadataBuffer::upload(&session, vec![0, 0, 2], 5).unwrap();
    let kernel = SmallFactorKernel::compile(&session, &metadata, 2, 0, 1).unwrap();
    let worlds = 257u32;
    let mut matrix_values = vec![-131072.0; worlds as usize * 4 + 8];
    for world in 0..worlds as usize {
        matrix_values[4 + 4 * world..8 + 4 * world].copy_from_slice(&[4.0, 0.0, 0.0, 9.0]);
    }
    let output_values = vec![-131072.0; matrix_values.len()];
    let matrix = session.upload(&matrix_values).unwrap();
    let mut output = session.upload(&output_values).unwrap();

    kernel
        .launch(&metadata, &matrix, &matrix, &mut output, worlds)
        .unwrap();
    let mut result = vec![0.0; output.len()];
    output.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &output_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &output_values[output_values.len() - 4..]
    );
    for world in 0..worlds as usize {
        assert_eq!(&result[4 + 4 * world..8 + 4 * world], &[2.0, 0.0, 0.0, 3.0]);
    }

    let short_matrix = session
        .upload(&matrix_values[..matrix_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &short_matrix, &short_matrix, &mut output, worlds,)
            .is_err()
    );
    let foreign_matrix = foreign.upload(&matrix_values).unwrap();
    assert!(matches!(
        kernel.launch(
            &metadata,
            &foreign_matrix,
            &foreign_matrix,
            &mut output,
            worlds,
        ),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        kernel.launch(&wrong_metadata, &matrix, &matrix, &mut output, worlds,),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "small_factor_static_group"
        }))
    ));

    metadata.write_for_test(0, -1).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &matrix, &matrix, &mut output, worlds),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "small_factor_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    let mut after = vec![0.0; output.len()];
    output.read_range_into(0, &mut after).unwrap();
    assert_eq!(result, after);
    metadata.restore_for_test().unwrap();
    kernel
        .launch(&metadata, &matrix, &matrix, &mut output, worlds)
        .unwrap();
}

#[test]
fn routes_keep_mathdx_tile_rejected_without_reclassifying_small() {
    assert!(check_routes(&input::mixed_input()).is_ok());
    assert!(check_routes(&input::sparse(&input::base()["models"][3])).is_err());
}
