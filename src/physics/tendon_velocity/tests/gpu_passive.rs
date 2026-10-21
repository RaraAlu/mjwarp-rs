//! 肌腱速度GPU被动消费测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new tendon passive consumer; does not rerun old tendon velocity acceptance"]
fn gpu_tendon_passive_consumes_actual_length_jacobian_and_velocity() {
    use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

    let session = TransferSession::new(0).unwrap();
    let source_plan = producer(&session, 3);
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let passive_plan = TendonPassivePlan::new(
        &velocity_plan,
        passive_parameters([2, 3, 5, 7, 11]),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let timer = std::time::Instant::now();
    let mut slots = 0;
    for worlds in [1, 2, 257] {
        let mut source = source_plan.create_data(worlds).unwrap();
        let mut velocity = velocity_plan.create_data(worlds).unwrap();
        let mut passive = passive_plan.create_data(worlds).unwrap();
        assert!(velocity_plan.update(&source, &mut velocity).is_err());
        source_plan.update(&mut source).unwrap();
        let mut qvel = vec![0.0; 3 * worlds];
        for row in qvel.chunks_mut(3) {
            row[0] = 0.25;
            row[1] = -0.25;
        }
        velocity.write_qvel(&qvel).unwrap();
        let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
        let velocity_snapshot = velocity_view.snapshot().unwrap();
        for world in 0..worlds {
            assert_eq!(velocity_snapshot.world(world).unwrap()[0], -0.5);
        }
        let snapshot = passive_plan
            .update(&velocity_view, &mut passive)
            .unwrap()
            .snapshot()
            .unwrap();
        for world in 0..worlds {
            assert_eq!(
                bits(snapshot.spring(world).unwrap()),
                bits(&[0.4375, -0.4375, 0.0])
            );
            assert_eq!(
                bits(snapshot.damper(world).unwrap()),
                bits(&[-0.1015625, 0.1015625, 0.0])
            );
            slots += 6;
        }

        let _ = velocity_view;
        let before = bits(snapshot.spring(0).unwrap());
        let source_buffer = source.tendon_velocity_source_for_test();
        let old_length = raw(source_buffer)[4];
        source_buffer.write_range(4, &[f32::NAN]).unwrap();
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        assert!(passive_plan.update(&view, &mut passive).is_err());
        assert_eq!(before, bits(snapshot.spring(0).unwrap()));
        let _ = view;
        source
            .tendon_velocity_source_for_test()
            .write_range(4, &[old_length])
            .unwrap();
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        passive_plan.update(&view, &mut passive).unwrap();

        let mut wrong_worlds = passive_plan.create_data(worlds + 1).unwrap();
        assert!(matches!(
            passive_plan.update(&view, &mut wrong_worlds),
            Err(TransferError::ModelMismatch)
        ));

        passive.corrupt_guard_for_test(true, 0, 7.0).unwrap();
        assert!(passive_plan.update(&view, &mut passive).is_err());
        passive.corrupt_guard_for_test(true, 0, -131072.0).unwrap();
        passive_plan.update(&view, &mut passive).unwrap();
        let end = passive.buffer_len_for_test(false) - 1;
        passive.corrupt_guard_for_test(false, end, 7.0).unwrap();
        assert!(passive_plan.update(&view, &mut passive).is_err());
        passive
            .corrupt_guard_for_test(false, end, -131072.0)
            .unwrap();
        passive_plan.update(&view, &mut passive).unwrap();
    }

    let foreign_source_plan = producer(&session, 3);
    let foreign_velocity_plan = TendonVelocityPlan::new(&foreign_source_plan).unwrap();
    let mut foreign_source = foreign_source_plan.create_data(1).unwrap();
    foreign_source_plan.update(&mut foreign_source).unwrap();
    let mut foreign_velocity = foreign_velocity_plan.create_data(1).unwrap();
    let foreign_view = foreign_velocity_plan
        .update(&foreign_source, &mut foreign_velocity)
        .unwrap();
    let mut passive = passive_plan.create_data(1).unwrap();
    assert!(matches!(
        passive_plan.update(&foreign_view, &mut passive),
        Err(TransferError::ModelMismatch)
    ));
    assert_eq!(slots, 1560);
    println!(
        "tendon_passive worlds=260 slots={slots} bit_difference=0 atomic_overlap_unproven=true phase_ms={}",
        timer.elapsed().as_millis()
    );
}
