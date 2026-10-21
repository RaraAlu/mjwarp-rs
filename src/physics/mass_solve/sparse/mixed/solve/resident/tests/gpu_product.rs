//! 常驻混合GPU产品路径测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA/NVRTC; resident mixed validation product path purpose"]
fn gpu_resident_mixed_validation_product_path_stops_publication() {
    let start = std::time::Instant::now();
    let v = input::reference();
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
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
    let source_tail = data.matrix.matrix_input().unwrap().len() - 4;
    let unselected_dense = 4 + 690 + 67;
    let assert_bits_unchanged = |label: &str, fault: &str, before: &[u32], after: &[u32]| {
        assert_eq!(before.len(), after.len());
        if let Some((index, (&expected, &actual))) = before
            .iter()
            .zip(after)
            .enumerate()
            .find(|(_, (expected, actual))| expected != actual)
        {
            panic!("{label} changed during rejected {fault} at {index}: {expected} -> {actual}");
        }
    };
    let mut recompute_drift = Vec::new();

    for (fault, offset, bad) in [
        ("head_guard", 0, 0.0),
        ("tail_guard", source_tail, 0.0),
        ("nonfinite_crb", 4, f32::NAN),
        ("nonfinite_unselected_dense", unselected_dense, f32::NAN),
    ] {
        let stable_factor = bits(&data.solve.mass.output);
        let stable_solution = bits(&data.solve.output);
        let mut original = [0.0];
        data.matrix
            .matrix_input()
            .unwrap()
            .read_range_into(offset, &mut original)
            .unwrap();
        data.matrix.corrupt_output_for_test(offset, bad).unwrap();

        assert!(plan.update_gather(&mut data).is_err());
        assert!(!data.gathered);
        assert!(!data.solve.ready);
        assert!(!data.solve.mass.ready);
        assert!(plan.update_factor(&mut data).is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.readback().is_err());
        assert_bits_unchanged("CSR", fault, &stable_csr, &bits(&data.solve.mass.matrix));
        assert_bits_unchanged(
            "factor",
            fault,
            &stable_factor,
            &bits(&data.solve.mass.output),
        );
        assert_bits_unchanged(
            "solution",
            fault,
            &stable_solution,
            &bits(&data.solve.output),
        );

        data.matrix
            .corrupt_output_for_test(offset, original[0])
            .unwrap();
        plan.update_gather(&mut data).unwrap();
        assert_bits_unchanged(
            "restored CSR",
            fault,
            &stable_csr,
            &bits(&data.solve.mass.matrix),
        );
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
        let restored = data.readback().unwrap();
        compare_solution(&restored, 0, &v, 0);
        compare_solution(&restored, 1, &v, 0);
        let recovered_factor = bits(&data.solve.mass.output);
        let recovered_solution = bits(&data.solve.output);
        recompute_drift.push((
            fault,
            stable_factor
                .iter()
                .zip(&recovered_factor)
                .filter(|(before, after)| before != after)
                .count(),
            stable_solution
                .iter()
                .zip(&recovered_solution)
                .filter(|(before, after)| before != after)
                .count(),
        ));
    }
    println!(
        "resident_mixed_validation_product worlds=2 faults=4 recovered=true recompute_drift={recompute_drift:?} phase_ms={}",
        start.elapsed().as_millis()
    );
}
