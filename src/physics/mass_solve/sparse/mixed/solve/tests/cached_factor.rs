//! 混合求解缓存因子与消费者故障测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new mixed consumer cached-only factor/source/guard failure purpose"]
fn cached_merged_factor_ignores_old_sources_and_rejects_each_consumer_fault() {
    let refs = samples::reference();
    let model = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let mut plan = SparseMassMixedSolvePlan::new(&session, samples::input(model)).unwrap();
    let mut data = plan.create_data(2, 3).unwrap();
    data.write_matrix(&samples::matrix(model, 0).repeat(2))
        .unwrap();
    let rhs = samples::rhs(model, 0, 3);
    data.write_rhs(&rhs.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    let expected = samples::solution(model, 0, 3);
    let bits = |buffer: &TransferBuffer<f32>| {
        let mut values = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let stable_solution = bits(&data.output);
    let stable_status = bits(&data.status);
    plan.metadata.write_for_test(0, -1).unwrap();
    data.write_rhs(&rhs.repeat(2)).unwrap();
    assert!(plan.update_solve(&mut data).is_err());
    assert!(data.mass.ready);
    assert!(data.readback().is_err());
    assert_eq!(bits(&data.output), stable_solution);
    assert_eq!(bits(&data.status), stable_status);
    plan.metadata.restore_for_test().unwrap();
    plan.update_solve(&mut data).unwrap();
    let mut factor = vec![0.0; data.mass.output.len()];
    data.mass.output.read_range_into(0, &mut factor).unwrap();
    // Both stale primary and producer CSR can be invalid; the published
    // merged factor alone owns this solve's lifetime and physics inputs.
    let mut a = [0.0];
    let mut b = [0.0];
    data.mass.matrix.read_range_into(4, &mut a).unwrap();
    data.mass.sparse.matrix.read_range_into(4, &mut b).unwrap();
    data.mass.matrix.write_range(4, &[f32::NAN]).unwrap();
    data.mass.sparse.matrix.write_range(4, &[f32::NAN]).unwrap();
    data.write_rhs(&rhs.repeat(2)).unwrap();
    plan.update_solve(&mut data).unwrap();
    let after = data.readback().unwrap();
    let mut unchanged = vec![0.0; factor.len()];
    data.mass.output.read_range_into(0, &mut unchanged).unwrap();
    assert_eq!(
        factor.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        unchanged.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    for w in 0..2 {
        for (&actual, &reference) in after.world(w).unwrap().solution.iter().zip(&expected) {
            assert!((f64::from(actual) - reference).abs() <= 2e-4 + 2e-4 * reference.abs());
        }
    }
    assert!(plan.update_factor(&mut data).is_err());
    assert!(data.readback().is_err());
    assert!(plan.update_gather(&mut data).is_err());
    data.mass.matrix.write_range(4, &a).unwrap();
    data.mass.sparse.matrix.write_range(4, &b).unwrap();
    plan.update(&mut data).unwrap();
    // A poisoned old Small producer must not invalidate a completed merge.
    data.mass
        .small
        .test_factor_buffer_mut()
        .write_range(0, &[0.0])
        .unwrap();
    data.write_rhs(&rhs.repeat(2)).unwrap();
    plan.update_solve(&mut data).unwrap();
    assert!(data.readback().is_ok());
    data.mass
        .small
        .test_factor_buffer_mut()
        .write_range(0, &[-131072.0])
        .unwrap();
    let prefix = plan.input().qld_block_total();
    let inverse = 4 + prefix + plan.input().nc();
    for (offset, bad) in [
        (0, 0.0),
        (factor.len() - 4, 0.0),
        (4, -1.0),
        (5, f32::NAN),
        (4 + prefix, -1.0),
        (inverse, -1.0),
        (inverse + 65, f32::INFINITY),
    ] {
        data.mass.output.write_range(offset, &[bad]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(!data.mass.ready);
        data.mass
            .output
            .write_range(offset, &[factor[offset]])
            .unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    let len = data.rhs.len();
    for (offset, bad) in [(0, 0.0), (len - 4, 0.0), (4, f32::NAN)] {
        let mut saved = [0.0];
        data.rhs.read_range_into(offset, &mut saved).unwrap();
        data.rhs.write_range(offset, &[bad]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.ready);
        data.rhs.write_range(offset, &saved).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    for offset in [0, len - 4] {
        data.output.write_range(offset, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.ready);
        data.output.write_range(offset, &[-131072.0]).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    data.output.write_range(4, &[f32::NAN]).unwrap();
    assert!(data.readback().is_err());
    plan.update_solve(&mut data).unwrap();
    for offset in [0, data.worlds() + 4] {
        data.status.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.ready);
        assert!(data.readback().is_err());
        data.status.write_range(offset, &[-131072.0]).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    let mut raw = vec![0.0; len];
    data.rhs.read_range_into(0, &mut raw).unwrap();
    data.write_world_rhs(1, &samples::rhs(model, 1, 3)).unwrap();
    let mut next = vec![0.0; len];
    data.rhs.read_range_into(0, &mut next).unwrap();
    for i in (0..4 + 258).chain(4 + 516..len) {
        assert_eq!(raw[i].to_bits(), next[i].to_bits());
    }
    plan.update_solve(&mut data).unwrap();
    assert!(data.readback().is_ok());
    assert_eq!(before.rhs_count(), 3);
}
