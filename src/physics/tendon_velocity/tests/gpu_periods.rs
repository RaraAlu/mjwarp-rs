//! 肌腱速度GPU变周期与空行测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new varying-period and empty-CSR branch coverage"]
fn gpu_tendon_passive_maps_varying_periods_and_skips_empty_rows() {
    use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

    let session = TransferSession::new(0).unwrap();
    let source_plan = producer(&session, 3);
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let varying = TendonPassivePlan::new(
        &velocity_plan,
        varying_passive_parameters(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let empty = TendonPassivePlan::new(
        &velocity_plan,
        empty_row_passive_parameters(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let worlds = 11;
    let mut source = source_plan.create_data(worlds).unwrap();
    source_plan.update(&mut source).unwrap();
    let mut velocity = velocity_plan.create_data(worlds).unwrap();
    let qvel = [0.25, -0.25, 0.0].repeat(worlds);
    velocity.write_qvel(&qvel).unwrap();
    let view = velocity_plan.update(&source, &mut velocity).unwrap();
    let mut varying_data = varying.create_data(worlds).unwrap();
    let snapshot = varying
        .update(&view, &mut varying_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..worlds {
        let x = 1.0_f32 - (world % 11) as f32 / 32.0;
        let stiffness = 0.25_f32 + (world % 2) as f32 / 64.0;
        let stiffness_poly = (world % 3) as f32 / 64.0;
        let spring_force = -x * (stiffness + stiffness_poly * x);
        let damping = 0.125_f32 + (world % 5) as f32 / 64.0;
        let damping_poly = (world % 7) as f32 / 64.0;
        let damper_force = 0.5_f32 * (damping + damping_poly * 0.5_f32);
        assert_eq!(
            bits(snapshot.spring(world).unwrap()),
            bits(&[-spring_force, spring_force, 0.0])
        );
        assert_eq!(
            bits(snapshot.damper(world).unwrap()),
            bits(&[-damper_force, damper_force, 0.0])
        );
    }
    let _ = view;

    source
        .tendon_velocity_source_for_test()
        .write_range(8, &[f32::NAN])
        .unwrap();
    let ready = velocity_plan.update(&source, &mut velocity).unwrap();
    let _ = ready;
    for world in 0..worlds {
        velocity
            .current
            .write_range(4 + world * 5 + 4, &[f32::NAN])
            .unwrap();
    }
    let invalid_empty_values = TendonVelocityView {
        plan: &velocity_plan,
        source: &source,
        data: &velocity,
    };
    let mut empty_data = empty.create_data(worlds).unwrap();
    let empty_snapshot = empty
        .update(&invalid_empty_values, &mut empty_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..worlds {
        assert_eq!(bits(empty_snapshot.spring(world).unwrap()), bits(&[0.0; 3]));
        assert_eq!(bits(empty_snapshot.damper(world).unwrap()), bits(&[0.0; 3]));
    }
    println!("tendon_passive_periods worlds=11 periods=2,3,5,7,11 empty_rows=11");
}
