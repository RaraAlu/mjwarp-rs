//! 被动合力冻结顺序累加GPU测试。

use super::tests_support::*;
use super::*;
use crate::model::{CamLightParameters, KinematicsParameters};
use crate::physics::{
    FlexEdgePassiveInput, FlexEdgePassiveOptions, JointPassiveOptions, TendonPassiveOptions,
    TendonVelocityPlan,
};

#[test]
#[ignore = "new G04 ordered shared accumulation and f32 grouping certificate"]
fn gpu_accumulates_joint_tendon_flexedge_in_frozen_order() {
    let timer = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let model = kinematic([1.0, 0.0, 0.0]);
    let source_plan = super::super::KinematicsPlan::with_flex_positions(
        &session,
        flex_model(model.clone()),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let joint_plan = JointPassivePlan::new(
        &session,
        model.clone(),
        joint_parameters(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    let tendon_plan = TendonPassivePlan::new(
        &velocity_plan,
        tendon_parameters(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let flex_plan = FlexEdgePassivePlan::new(
        &source_plan,
        FlexEdgePassiveInput {
            length0: vec![0.0],
            stiffness: vec![-1.0],
            damping: vec![0.5],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions::default(),
    )
    .unwrap();
    let plan = PassiveForcePlan::new(&joint_plan, &tendon_plan, &flex_plan).unwrap();
    let worlds = 11;
    let mut source = source_plan.create_data(worlds).unwrap();
    let mut velocity = velocity_plan.create_data(worlds).unwrap();
    let mut joint = joint_plan.create_data(worlds).unwrap();
    let mut tendon = tendon_plan.create_data(worlds).unwrap();
    let mut flex = flex_plan.create_data(worlds).unwrap();
    let mut data = plan.create_data(worlds).unwrap();
    let zero = vec![0.0; worlds];
    source.write_qpos(&vec![1.0; worlds]).unwrap();
    source.write_qvel(&zero).unwrap();
    source_plan.update(&mut source).unwrap();
    velocity.write_qvel(&zero).unwrap();
    joint.write_qpos(&vec![1.0; worlds]).unwrap();
    joint.write_qvel(&zero).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    let joint_snapshot = joint.readback_joint().unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let tendon_snapshot = tendon_plan
        .update(&velocity_view, &mut tendon)
        .unwrap()
        .snapshot()
        .unwrap();
    let flex_snapshot = flex_plan
        .update(&source, &mut flex)
        .unwrap()
        .snapshot()
        .unwrap();
    let combined = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let snapshot = combined.snapshot().unwrap();
    let smooth = combined.smooth().unwrap();
    for world in 0..worlds {
        let spring = ordered_atomic_sources(
            joint_snapshot.spring(world).unwrap()[0],
            tendon_snapshot.spring(world).unwrap()[0],
            flex_snapshot.spring(world).unwrap()[0],
        );
        let damper = ordered_atomic_sources(
            joint_snapshot.damper(world).unwrap()[0],
            tendon_snapshot.damper(world).unwrap()[0],
            flex_snapshot.damper(world).unwrap()[0],
        );
        assert_eq!(
            snapshot.spring(world).unwrap()[0].to_bits(),
            spring.to_bits()
        );
        assert_eq!(
            snapshot.damper(world).unwrap()[0].to_bits(),
            damper.to_bits()
        );
        assert_eq!(
            smooth.world(world).unwrap()[0].to_bits(),
            (spring + damper).to_bits()
        );
    }
    assert_eq!(
        joint_snapshot.spring(0).unwrap()[0].to_bits(),
        16_777_216.0f32.to_bits()
    );
    assert_eq!(
        tendon_snapshot.spring(0).unwrap()[0].to_bits(),
        1.0f32.to_bits()
    );
    assert_eq!(
        flex_snapshot.spring(0).unwrap()[0].to_bits(),
        1.0f32.to_bits()
    );
    assert_eq!(
        snapshot.spring(0).unwrap()[0].to_bits(),
        16_777_216.0f32.to_bits()
    );
    assert_eq!(
        (joint_snapshot.spring(0).unwrap()[0]
            + (tendon_snapshot.spring(0).unwrap()[0] + flex_snapshot.spring(0).unwrap()[0]))
            .to_bits(),
        16_777_218.0f32.to_bits()
    );
    let _ = combined;
    data.corrupt_parts_guard_for_test(0, 99.0).unwrap();
    assert!(
        plan.update(&joint, &velocity_view, &source, &mut data)
            .is_err()
    );
    assert!(data.snapshot().is_err());
    data.corrupt_parts_guard_for_test(0, -131072.0).unwrap();
    plan.update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let _ = velocity_view;

    let moving = (0..worlds)
        .map(|world| (world + 1) as f32 / 16.0)
        .collect::<Vec<_>>();
    source.write_qpos(&vec![2.0; worlds]).unwrap();
    source.write_qvel(&moving).unwrap();
    source_plan.update(&mut source).unwrap();
    velocity.write_qvel(&moving).unwrap();
    joint.write_qpos(&vec![2.0; worlds]).unwrap();
    joint.write_qvel(&moving).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    let fresh_joint = joint.readback_joint().unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let fresh_tendon = tendon_plan
        .update(&velocity_view, &mut tendon)
        .unwrap()
        .snapshot()
        .unwrap();
    let fresh_flex = flex_plan
        .update(&source, &mut flex)
        .unwrap()
        .snapshot()
        .unwrap();
    let fresh_view = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let fresh = fresh_view.snapshot().unwrap();
    let fresh_smooth = fresh_view.smooth().unwrap();
    for world in 0..worlds {
        let spring = ordered_atomic_sources(
            fresh_joint.spring(world).unwrap()[0],
            fresh_tendon.spring(world).unwrap()[0],
            fresh_flex.spring(world).unwrap()[0],
        );
        let damper = ordered_atomic_sources(
            fresh_joint.damper(world).unwrap()[0],
            fresh_tendon.damper(world).unwrap()[0],
            fresh_flex.damper(world).unwrap()[0],
        );
        assert_eq!(fresh.spring(world).unwrap()[0].to_bits(), spring.to_bits());
        assert_eq!(fresh.damper(world).unwrap()[0].to_bits(), damper.to_bits());
        assert_eq!(
            fresh_smooth.world(world).unwrap()[0].to_bits(),
            (spring + damper).to_bits()
        );
    }
    assert!(
        fresh_flex
            .damper(0)
            .unwrap()
            .iter()
            .any(|value| *value != 0.0)
    );
    let _ = fresh_view;

    let foreign_model = kinematic([0.0, 1.0, 0.0]);
    let foreign_joint = JointPassivePlan::new(
        &session,
        foreign_model,
        joint_parameters(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    assert!(PassiveForcePlan::new(&foreign_joint, &tendon_plan, &flex_plan).is_err());
    let foreign_session = TransferSession::new(0).unwrap();
    let foreign_joint = JointPassivePlan::new(
        &foreign_session,
        model.clone(),
        joint_parameters(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    assert!(PassiveForcePlan::new(&foreign_joint, &tendon_plan, &flex_plan).is_err());

    let disabled = JointPassiveOptions {
        disable_spring: true,
        disable_damper: true,
    };
    let disabled_joint =
        JointPassivePlan::new(&session, model, joint_parameters(), disabled).unwrap();
    let disabled_tendon = TendonPassivePlan::new(
        &velocity_plan,
        tendon_parameters(),
        TendonPassiveOptions {
            disable_spring: true,
            disable_damper: true,
        },
    )
    .unwrap();
    let disabled_flex = FlexEdgePassivePlan::new(
        &source_plan,
        FlexEdgePassiveInput {
            length0: vec![f32::NAN],
            stiffness: vec![f32::NAN],
            damping: vec![f32::NAN],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions {
            disable_spring: true,
            disable_damper: true,
        },
    )
    .unwrap();
    let disabled_plan =
        PassiveForcePlan::new(&disabled_joint, &disabled_tendon, &disabled_flex).unwrap();
    let mut disabled_data = disabled_plan.create_data(3).unwrap();
    let cleared = disabled_plan.clear_disabled(&mut disabled_data).unwrap();
    for world in 0..3 {
        assert_eq!(cleared.snapshot().unwrap().spring(world).unwrap(), [0.0]);
        assert_eq!(cleared.snapshot().unwrap().damper(world).unwrap(), [0.0]);
        assert_eq!(cleared.smooth().unwrap().world(world).unwrap(), [0.0]);
    }
    let report = serde_json::json!({
        "worlds": worlds,
        "compared_slots": 2 * worlds * 3,
        "ordered_sources": ["joint_write", "tendon_atomic_add", "flex_edge_atomic_add"],
        "f32_grouping_certificate": true,
        "repeat_update_without_residue": true,
        "partial_publication_rejected": true,
        "dual_disabled_cold_clear": true,
        "sleep_filtering": false,
        "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
        "passive_force_owned_bytes": 4 * (plan.metadata.len() + plan.dummy.len() + data.parts.len() + data.status.len()),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_PASSIVE_FORCE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
