//! 肌腱速度GPU禁用与零系数测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new conditional-read coverage for frozen disable and zero-coefficient semantics"]
fn gpu_tendon_passive_skips_disabled_and_inactive_inputs() {
    use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

    let session = TransferSession::new(0).unwrap();
    let source_plan = producer(&session, 3);
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let parameters = passive_parameters([2, 3, 5, 7, 11]);
    let active = TendonPassivePlan::new(
        &velocity_plan,
        parameters.clone(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let disabled = TendonPassivePlan::new(
        &velocity_plan,
        parameters.clone(),
        TendonPassiveOptions {
            disable_spring: true,
            disable_damper: true,
        },
    )
    .unwrap();
    let damper = TendonPassivePlan::new(
        &velocity_plan,
        parameters.clone(),
        TendonPassiveOptions {
            disable_spring: true,
            disable_damper: false,
        },
    )
    .unwrap();
    let spring = TendonPassivePlan::new(
        &velocity_plan,
        parameters,
        TendonPassiveOptions {
            disable_spring: false,
            disable_damper: true,
        },
    )
    .unwrap();

    let mut source = source_plan.create_data(1).unwrap();
    source_plan.update(&mut source).unwrap();
    let mut velocity = velocity_plan.create_data(1).unwrap();
    velocity.write_qvel(&[0.25, -0.25, 0.0]).unwrap();
    let initial = velocity_plan.update(&source, &mut velocity).unwrap();
    let _ = initial;

    let source_buffer = source.tendon_velocity_source_for_test();
    let raw_source = raw(source_buffer);
    source_buffer.write_range(4, &[f32::NAN, f32::NAN]).unwrap();
    let view = velocity_plan.update(&source, &mut velocity).unwrap();
    let mut disabled_data = disabled.create_data(1).unwrap();
    let zero = disabled
        .update(&view, &mut disabled_data)
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(bits(zero.spring(0).unwrap()), bits(&[0.0; 3]));
    assert_eq!(bits(zero.damper(0).unwrap()), bits(&[0.0; 3]));

    let mut damper_data = damper.create_data(1).unwrap();
    let damper_snapshot = damper
        .update(&view, &mut damper_data)
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(
        bits(damper_snapshot.damper(0).unwrap()),
        bits(&[-0.1015625, 0.1015625, 0.0])
    );
    let mut active_data = active.create_data(1).unwrap();
    assert!(active.update(&view, &mut active_data).is_err());
    let _ = view;

    source
        .tendon_velocity_source_for_test()
        .write_range(4, &[raw_source[4]])
        .unwrap();
    let view = velocity_plan.update(&source, &mut velocity).unwrap();
    active.update(&view, &mut active_data).unwrap();
    let _ = view;
    source
        .tendon_velocity_source_for_test()
        .write_range(5, &[raw_source[5]])
        .unwrap();

    velocity.current.write_range(4, &[f32::NAN]).unwrap();
    let invalid_velocity = TendonVelocityView {
        plan: &velocity_plan,
        source: &source,
        data: &velocity,
    };
    let mut spring_data = spring.create_data(1).unwrap();
    spring.update(&invalid_velocity, &mut spring_data).unwrap();
    assert!(damper.update(&invalid_velocity, &mut damper_data).is_err());
    println!("tendon_passive_conditional disabled=2 inactive_length=1 disabled_velocity=1");
}
