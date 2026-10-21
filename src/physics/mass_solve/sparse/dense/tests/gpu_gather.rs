//! 稠密GPU聚集包装与产品路径测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA/NVRTC; dense gather wrapper purpose"]
fn gpu_dense_gather_wrapper_checks_tail_spans_and_proof_lifecycle() {
    let start = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let nc = 3u32;
    let prefix = 257u32;
    let worlds = 2usize;
    let sources = (0..prefix)
        .map(|slot| (slot % (nc + 1)) as i32)
        .collect::<Vec<_>>();
    let mut metadata =
        DenseGatherMetadataBuffer::upload(&session, sources.clone(), nc, prefix).unwrap();
    let mut kernel = DenseGatherKernel::compile(&session, &metadata).unwrap();
    let mut matrix_values = vec![-131072.0f32; nc as usize * worlds + 8];
    matrix_values[4..10].copy_from_slice(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    let matrix = session.upload(&matrix_values).unwrap();
    let unused = session.upload(&[7.0f32]).unwrap();
    let output_values = vec![-131072.0f32; prefix as usize * worlds + 8];
    let mut output = session.upload(&output_values).unwrap();

    kernel
        .launch(&metadata, &matrix, &unused, &mut output, worlds)
        .unwrap();
    let mut actual = vec![0.0; output.len()];
    output.read_range_into(0, &mut actual).unwrap();
    assert_eq!(&actual[..4], &output_values[..4]);
    assert_eq!(
        &actual[actual.len() - 4..],
        &output_values[output_values.len() - 4..]
    );
    for world in 0..worlds {
        for (slot, &source) in sources.iter().enumerate() {
            let expected = if source == nc as i32 {
                0.0
            } else {
                matrix_values[4 + world * nc as usize + source as usize]
            };
            assert_eq!(
                actual[4 + world * prefix as usize + slot].to_bits(),
                expected.to_bits()
            );
        }
    }
    let stable = actual.clone();

    let short_matrix = session
        .upload(&matrix_values[..matrix_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &short_matrix, &unused, &mut output, worlds)
            .is_err()
    );
    let mut short_output = session
        .upload(&output_values[..output_values.len() - 1])
        .unwrap();
    assert!(
        kernel
            .launch(&metadata, &matrix, &unused, &mut short_output, worlds,)
            .is_err()
    );
    let mut short_after = vec![0.0; short_output.len()];
    short_output.read_range_into(0, &mut short_after).unwrap();
    assert_eq!(short_after, output_values[..output_values.len() - 1]);
    let foreign_unused = foreign.upload(&[7.0f32]).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &matrix, &foreign_unused, &mut output, worlds),
        Err(TransferError::SessionMismatch)
    ));
    kernel.nc += 1;
    assert!(
        kernel
            .launch(&metadata, &matrix, &unused, &mut output, worlds)
            .is_err()
    );
    kernel.nc = nc;
    kernel.prefix -= 1;
    assert!(
        kernel
            .launch(&metadata, &matrix, &unused, &mut output, worlds)
            .is_err()
    );
    kernel.prefix = prefix;
    output.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, stable);
    metadata.write_for_test(0, -1).unwrap();
    assert!(
        kernel
            .launch(&metadata, &matrix, &unused, &mut output, worlds)
            .is_err()
    );
    output.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, stable);

    metadata.restore_for_test().unwrap();
    kernel
        .launch(&metadata, &matrix, &unused, &mut output, worlds)
        .unwrap();
    output.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, stable);
    println!(
        "dense_gather_wrapper worlds={worlds} prefix={prefix} phase_ms={}",
        start.elapsed().as_millis()
    );
}

#[test]
#[ignore = "requires NVIDIA/NVRTC; dense gather product-call purpose"]
fn gpu_dense_gather_real_update_handles_zero_prefix_and_stale_proof() {
    let start = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();

    let zero_plan = SparseMassDensePlan::new(&session, input::fork(1)).unwrap();
    assert_eq!(zero_plan.input().qld_block_total(), 0);
    let mut zero_data = zero_plan.create_data(2).unwrap();
    zero_data.write_matrix(&[2.0, 3.0]).unwrap();
    zero_plan.update(&mut zero_data).unwrap();
    let zero = zero_data.readback().unwrap();
    assert!(zero.world(0).unwrap().is_empty());
    assert!(zero.world(1).unwrap().is_empty());

    let refs = input::reference();
    let model = &refs["models"][3];
    let mut plan = SparseMassDensePlan::new(&session, input::native(model)).unwrap();
    let mut data = plan.create_data(2).unwrap();
    let matrix = input::matrix(&model["cases"][0]);
    data.write_matrix(&matrix.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let stable = data.readback().unwrap().values;

    plan.metadata.write_for_test(0, -1).unwrap();
    assert!(plan.update(&mut data).is_err());
    assert!(!data.ready);
    assert!(data.readback().is_err());
    let mut rejected = vec![0.0; data.dense.len()];
    data.dense.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(
        rejected.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        stable.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );

    plan.metadata.restore_for_test().unwrap();
    plan.update(&mut data).unwrap();
    let recovered = data.readback().unwrap().values;
    assert_eq!(
        recovered.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        stable.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    println!(
        "dense_gather_real worlds=2 zero_prefix=true recovered=true phase_ms={}",
        start.elapsed().as_millis()
    );
}
