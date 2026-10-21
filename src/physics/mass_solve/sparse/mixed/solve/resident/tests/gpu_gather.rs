//! 常驻混合GPU聚集包装与陈旧元数据测试。

#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_dims;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::resident_mixed_gather_lengths;
#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedGatherKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::ResidentMixedGatherMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_resident_mixed_gather_wrapper_tracks_tail_and_metadata_revision() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let (nb, n, nc, worlds) = (2usize, 3usize, 6usize, 257usize);
    let values = vec![1, 2, 3, 0, 1, 3, 0, 0, 1, 0, 1, 2];
    let mut metadata =
        ResidentMixedGatherMetadataBuffer::upload(&session, values.clone(), n as u32, nc as u32)
            .unwrap();
    let kernel = ResidentMixedGatherKernel::compile(&session, &metadata, nb).unwrap();
    let dims = resident_mixed_gather_dims(nb as u32, n as u32, nc as u32, worlds as u32).unwrap();
    let (_, source_len, output_len) = resident_mixed_gather_lengths(&dims).unwrap();
    let stride = 10 * nb + n * n;
    let mut source_values = vec![-131072.0f32; source_len as usize];
    for world in 0..worlds {
        let dense = 4 + world * stride + 10 * nb;
        for row in 0..n {
            for column in 0..n {
                source_values[dense + row * n + column] = (1000 * world + 10 * row + column) as f32;
            }
        }
    }
    let output_values = vec![-131072.0f32; output_len as usize];
    let source = session.upload(&source_values).unwrap();
    let mut output = session.upload(&output_values).unwrap();
    kernel
        .launch(&metadata, &source, &source, &mut output, worlds)
        .unwrap();
    let mut result = vec![0.0; output.len()];
    output.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &output_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &output_values[output_values.len() - 4..]
    );
    let (rownnz, rest) = values.split_at(n);
    let (rowadr, colind) = rest.split_at(n);
    for world in [0, 256] {
        let dense = 4 + world * stride + 10 * nb;
        for row in 0..n {
            for slot in 0..rownnz[row] as usize {
                let index = rowadr[row] as usize + slot;
                let column = colind[index] as usize;
                assert_eq!(
                    result[4 + world * nc + index].to_bits(),
                    source_values[dense + row * n + column].to_bits()
                );
            }
        }
    }
    let stable = result
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>();

    metadata.write_for_test(0, 0).unwrap();
    assert!(
        kernel
            .launch(&metadata, &source, &source, &mut output, worlds)
            .is_err()
    );
    output.read_range_into(0, &mut result).unwrap();
    assert_eq!(
        result
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        stable
    );
    metadata.restore_for_test().unwrap();
    kernel
        .launch(&metadata, &source, &source, &mut output, worlds)
        .unwrap();
    output.read_range_into(0, &mut result).unwrap();
    assert_eq!(
        result
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        stable
    );

    let short_source = session
        .upload(&source_values[..source_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &short_source, &short_source, &mut output, worlds,)
            .is_err()
    );
    let foreign_unused = foreign.upload(&[0.0f32]).unwrap();
    assert!(
        kernel
            .launch(&metadata, &source, &foreign_unused, &mut output, worlds,)
            .is_err()
    );
    output.read_range_into(0, &mut result).unwrap();
    assert_eq!(
        result
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        stable
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA/NVRTC; resident mixed gather publication purpose"]
fn gpu_resident_mixed_gather_stale_metadata_stops_publication() {
    let v = input::reference();
    let session = TransferSession::new(0).unwrap();
    let mut plan =
        SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
            .unwrap();
    let mut data = plan.create_data(2, 3).unwrap();
    data.write_qpos(&input::qpos(0).repeat(2)).unwrap();
    data.write_rhs(&input::rhs(&v, 0, 3).repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let bits = |buffer: &TransferBuffer<f32>| {
        let mut values = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let stable_csr = bits(&data.solve.mass.matrix);
    let stable_factor = bits(&data.solve.mass.output);
    let stable_solution = bits(&data.solve.output);

    plan.metadata.write_for_test(0, 0).unwrap();
    assert!(plan.update_gather(&mut data).is_err());
    assert!(!data.gathered);
    assert!(!data.solve.ready);
    assert!(!data.solve.mass.ready);
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_solve(&mut data).is_err());
    assert!(data.readback().is_err());
    assert_eq!(bits(&data.solve.mass.matrix), stable_csr);
    assert_eq!(bits(&data.solve.mass.output), stable_factor);
    assert_eq!(bits(&data.solve.output), stable_solution);

    plan.metadata.restore_for_test().unwrap();
    plan.update_gather(&mut data).unwrap();
    assert_eq!(bits(&data.solve.mass.matrix), stable_csr);
    plan.update_factor(&mut data).unwrap();
    plan.update_solve(&mut data).unwrap();
    let restored = data.readback().unwrap();
    compare_solution(&restored, 0, &v, 0);
    compare_solution(&restored, 1, &v, 0);
}
