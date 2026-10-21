//! 常驻混合缓存故障与守卫测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new bridge source/status/primary guard and cached CRB lifetime purpose"]
fn cache_fault_bridge_rejects_full_source_and_target_guards_without_partial_publication() {
    let v = input::reference();
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
        .unwrap();
    let mut data = plan.create_data(2, 3).unwrap();
    data.write_qpos(&input::qpos(0).repeat(2)).unwrap();
    data.write_rhs(&input::rhs(&v, 0, 3).repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    // This cross-tree upper-triangular slot never enters the CSR. A cached
    // solve may ignore it; fresh bridge capture must validate and reject it.
    let unselected = 4 + 690 + 67;
    let mut saved = [0.0];
    data.matrix
        .matrix_input()
        .unwrap()
        .read_range_into(unselected, &mut saved)
        .unwrap();
    data.matrix
        .corrupt_output_for_test(unselected, f32::NAN)
        .unwrap();
    plan.update_solve(&mut data).unwrap();
    let cached = data.readback().unwrap();
    compare_solution(&cached, 0, &v, 0);
    assert!(data.readback_matrix().is_err());
    assert!(plan.update_gather(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_solve(&mut data).is_err());
    assert!(data.readback().is_err());
    data.matrix
        .corrupt_output_for_test(unselected, saved[0])
        .unwrap();
    plan.update_gather(&mut data).unwrap();
    plan.update_factor(&mut data).unwrap();
    plan.update_solve(&mut data).unwrap();
    for (offset, bad) in [
        (0, 0.0),
        (4 + 2 * 5314, 0.0),
        (4, f32::NAN),
        (4 + 690 + 67, f32::NAN),
    ] {
        let mut original = [0.0];
        data.matrix
            .matrix_input()
            .unwrap()
            .read_range_into(offset, &mut original)
            .unwrap();
        data.matrix.corrupt_output_for_test(offset, bad).unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        assert!(!data.gathered);
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix
            .corrupt_output_for_test(offset, original[0])
            .unwrap();
        plan.update_gather(&mut data).unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    for offset in [0, 4 + 2 * 2149] {
        data.solve.mass.matrix.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.solve
            .mass
            .matrix
            .write_range(offset, &[-131072.0])
            .unwrap();
        plan.update_gather(&mut data).unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    for offset in [0, data.worlds() + 4] {
        data.status.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.status.write_range(offset, &[-131072.0]).unwrap();
        plan.update_gather(&mut data).unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    let out = data.readback().unwrap();
    compare_solution(&out, 0, &v, 0);
    compare_solution(&out, 1, &v, 0);
}
