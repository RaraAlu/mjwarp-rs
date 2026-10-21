//! Small校验产品路径测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA/NVRTC; Small validation product-call purpose"]
fn gpu_small_validation_real_update_rejects_and_recovers_without_factor_write() {
    let start = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let mut plan = SparseMassSmallPlan::new(&session, input::mixed_input()).unwrap();
    let mut data = plan.create_data(2).unwrap();
    let matrix = input::mixed_matrix(0);
    data.write_matrix(&matrix.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let bits = |values: &[f32]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let mut stable = vec![0.0; data.factor.len()];
    data.factor.read_range_into(0, &mut stable).unwrap();

    data.source.dense.write_range(0, &[0.0]).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    assert!(!data.ready);
    assert!(!data.source.ready);
    let mut rejected = vec![0.0; data.factor.len()];
    data.factor.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(bits(&stable), bits(&rejected));

    data.source.dense.write_range(0, &[-131072.0]).unwrap();
    plan.update_gather(&mut data).unwrap();
    plan.update_factor(&mut data).unwrap();
    data.factor.read_range_into(0, &mut stable).unwrap();
    data.source.status.write_range(4, &[0.0, 0.0]).unwrap();
    plan.metadata.write_for_test(0, -1).unwrap();
    assert!(matches!(
        plan.update_factor(&mut data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "small_factor_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    let mut status = vec![0.0; data.source.status.len()];
    data.source.status.read_range_into(0, &mut status).unwrap();
    assert_eq!(&status[4..6], &[1.0, 1.0]);
    data.factor.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(bits(&stable), bits(&rejected));

    plan.metadata.restore_for_test().unwrap();
    plan.update_factor(&mut data).unwrap();
    assert!(data.readback().is_ok());
    println!(
        "small_validation_private worlds=2 recovered=true phase_ms={} status_capacity_bytes={}",
        start.elapsed().as_millis(),
        data.source.status.len() * std::mem::size_of::<f32>()
    );
}
