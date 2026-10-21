//! Small缓存与守卫故障测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA/NVRTC; new Small prefix cached-input/factor fault purpose"]
fn cache_ignores_old_csr_and_guarded_factor_stages_reject_faults() {
    let start = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let mut plan = SparseMassSmallPlan::new(&session, input::mixed_input()).unwrap();
    let mut data = plan.create_data(2).unwrap();
    let matrix = input::mixed_matrix(0);
    data.write_matrix(&matrix.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let mut stable = vec![0.0; data.factor.len()];
    data.factor.read_range_into(0, &mut stable).unwrap();
    plan.metadata.write_for_test(0, -1).unwrap();
    assert!(matches!(
        plan.update_factor(&mut data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "small_factor_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    let mut rejected = vec![0.0; data.factor.len()];
    data.factor.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(stable, rejected);
    plan.metadata.restore_for_test().unwrap();
    plan.update_factor(&mut data).unwrap();
    let before = data.readback().unwrap();
    data.source.matrix.write_range(4, &[f32::NAN]).unwrap();
    plan.update_factor(&mut data).unwrap();
    let after = data.readback().unwrap();
    for w in 0..2 {
        assert_eq!(
            after
                .world(w)
                .unwrap()
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            before
                .world(w)
                .unwrap()
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
    }
    assert!(plan.update_gather(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(data.readback().is_err());
    data.source.matrix.write_range(4, &[matrix[0]]).unwrap();
    plan.update(&mut data).unwrap();
    let tail = 4 + data.source.dense_layout.total_elements();
    for (offset, bad) in [(0, 0.0), (tail, 0.0), (4, f32::NAN)] {
        let mut saved = [0.0];
        data.source
            .dense
            .read_range_into(offset, &mut saved)
            .unwrap();
        data.source.dense.write_range(offset, &[bad]).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(!data.source.ready);
        assert!(data.readback().is_err());
        data.source.dense.write_range(offset, &saved).unwrap();
        plan.update(&mut data).unwrap();
    }
    for offset in [0, tail] {
        data.factor.write_range(offset, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.factor.write_range(offset, &[-131072.0]).unwrap();
        plan.update_factor(&mut data).unwrap();
    }
    for value in [f32::NAN, -1.0] {
        data.factor.write_range(4, &[value]).unwrap();
        assert!(data.readback().is_err());
        plan.update_factor(&mut data).unwrap();
    }
    for offset in [0, data.worlds() + 4] {
        data.source.status.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.source
            .status
            .write_range(offset, &[-131072.0])
            .unwrap();
        plan.update_factor(&mut data).unwrap();
    }
    assert!(data.readback().is_ok());
    println!(
        "small_factor_private metadata_faults=1 recovered=true phase_ms={} factor_capacity_bytes={}",
        start.elapsed().as_millis(),
        data.factor.len() * std::mem::size_of::<f32>()
    );
}
