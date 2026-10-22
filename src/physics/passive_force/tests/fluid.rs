//! 被动合力流体组合与失败边界GPU测试。

use super::tests_support::*;
use super::*;
use crate::model::{CamLightParameters, KinematicsParameter, KinematicsParameters};
use crate::model::{ParameterBatch, SleepTreeState, TendonWakeFields, TendonWakeModelInput};
use crate::physics::{
    FlexEdgePassiveInput, FlexEdgePassiveOptions, JointPassiveOptions, TendonPassiveOptions,
    TendonVelocityPlan,
};
#[test]
#[ignore = "new G04 resident fluid composition and failure boundaries"]
fn gpu_wires_resident_fluid_into_shared_passive_order() {
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
    let gravity = GravityCompensationParameters {
        gravity: ParameterBatch::new(2, 3, vec![-1.0, 0.0, 0.0, -2.0, 0.0, 0.0]).unwrap(),
        body_gravcomp: ParameterBatch::new(3, 2, vec![0.0, 1.0, 0.0, 0.5, 0.0, 0.25]).unwrap(),
        jnt_actgravcomp: vec![0],
    };
    let fluid = fluid_input(2);
    let base_plan = PassiveForcePlan::with_gravity_compensation(
        &joint_plan,
        &tendon_plan,
        &flex_plan,
        gravity.clone(),
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let plan = PassiveForcePlan::with_gravity_and_fluid(
        &joint_plan,
        &tendon_plan,
        &flex_plan,
        gravity,
        GravityCompensationOptions::default(),
        fluid.clone(),
    )
    .unwrap();
    let worlds = 5;
    let qpos = (0..worlds)
        .map(|world| 0.125 * (world + 1) as f32)
        .collect::<Vec<_>>();
    let qvel = (0..worlds)
        .map(|world| 0.25 + 0.125 * world as f32)
        .collect::<Vec<_>>();
    let mut source = source_plan.create_data(worlds).unwrap();
    let mut velocity = velocity_plan.create_data(worlds).unwrap();
    let mut joint = joint_plan.create_data(worlds).unwrap();
    let mut base_data = base_plan.create_data(worlds).unwrap();
    let mut data = plan.create_data(worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source.write_qvel(&qvel).unwrap();
    source_plan.update(&mut source).unwrap();
    velocity.write_qvel_from(&source, &qvel).unwrap();
    joint.write_qpos_from(&source, &qpos).unwrap();
    joint.write_qvel_from(&source, &qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let base = base_plan
        .update(&joint, &velocity_view, &source, &mut base_data)
        .unwrap();
    let base_snapshot = base.snapshot().unwrap();
    let base_smooth = base.smooth().unwrap();
    let view = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let snapshot = view.snapshot().unwrap();
    let smooth = view.smooth().unwrap();
    let fluid_snapshot = data.fluid.as_ref().unwrap().snapshot().unwrap();
    let mut fluid_nonzero = false;
    for world in 0..worlds {
        let fluid_value = fluid_snapshot.world(world).unwrap().qfrc[0];
        fluid_nonzero |= fluid_value != 0.0;
        assert_eq!(
            snapshot.spring(world).unwrap()[0].to_bits(),
            base_snapshot.spring(world).unwrap()[0].to_bits()
        );
        assert_eq!(
            snapshot.damper(world).unwrap()[0].to_bits(),
            base_snapshot.damper(world).unwrap()[0].to_bits()
        );
        assert_eq!(
            snapshot.gravity(world).unwrap()[0].to_bits(),
            base_snapshot.gravity(world).unwrap()[0].to_bits()
        );
        assert_eq!(
            smooth.world(world).unwrap()[0].to_bits(),
            (base_smooth.world(world).unwrap()[0] + fluid_value).to_bits()
        );
    }
    assert!(fluid_nonzero);

    data.fluid
        .as_mut()
        .unwrap()
        .corrupt_body_guard_for_test(0, 99.0)
        .unwrap();
    assert!(
        plan.update(&joint, &velocity_view, &source, &mut data)
            .is_err()
    );
    assert!(data.snapshot().is_err());
    assert!(data.smooth_output().is_err());
    data.fluid
        .as_mut()
        .unwrap()
        .corrupt_body_guard_for_test(0, -131072.0)
        .unwrap();
    plan.update(&joint, &velocity_view, &source, &mut data)
        .unwrap();

    let foreign_plan = super::super::KinematicsPlan::with_flex_positions(
        &session,
        flex_model(model.clone()),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let foreign_source = foreign_plan.create_data(worlds).unwrap();
    assert!(matches!(
        plan.update(&joint, &velocity_view, &foreign_source, &mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.snapshot().is_ok());
    let fluid_before = data.fluid.as_ref().unwrap().snapshot().unwrap();
    let changed_qvel = (0..worlds)
        .map(|world| -0.5 - 0.125 * world as f32)
        .collect::<Vec<_>>();
    source.write_qvel(&changed_qvel).unwrap();
    velocity.write_qvel(&changed_qvel).unwrap();
    joint.write_qvel(&changed_qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    assert!(matches!(
        plan.update(&joint, &velocity_view, &source, &mut data),
        Err(TransferError::StageNotReady {
            stage: "flex_edge_passive_source"
        })
    ));
    assert!(data.snapshot().is_err());
    assert!(data.smooth_output().is_err());
    source_plan.update_flex_edges(&mut source).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    plan.update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let fluid_after = data.fluid.as_ref().unwrap().snapshot().unwrap();
    assert!((0..worlds).any(|world| {
        fluid_before.world(world).unwrap().qfrc != fluid_after.world(world).unwrap().qfrc
    }));

    let disabled_options = JointPassiveOptions {
        disable_spring: true,
        disable_damper: true,
    };
    let disabled_joint =
        JointPassivePlan::new(&session, model, joint_parameters(), disabled_options).unwrap();
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
    let disabled_plan = PassiveForcePlan::with_fluid_force(
        &disabled_joint,
        &disabled_tendon,
        &disabled_flex,
        fluid,
    )
    .unwrap();
    let mut disabled_data = disabled_plan.create_data(worlds).unwrap();
    assert!(matches!(
        disabled_plan.update(&joint, &velocity_view, &source, &mut disabled_data),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "passive_force_disabled_path"
        }))
    ));
    let cleared = disabled_plan.clear_disabled(&mut disabled_data).unwrap();
    let cleared_snapshot = cleared.snapshot().unwrap();
    let cleared_smooth = cleared.smooth().unwrap();
    let cleared_fluid = disabled_data.fluid.as_ref().unwrap().snapshot().unwrap();
    for world in 0..worlds {
        assert!(
            cleared_snapshot
                .spring(world)
                .unwrap()
                .iter()
                .all(|&value| { value.to_bits() == 0.0f32.to_bits() })
        );
        assert!(
            cleared_snapshot
                .damper(world)
                .unwrap()
                .iter()
                .all(|&value| { value.to_bits() == 0.0f32.to_bits() })
        );
        assert!(
            cleared_smooth
                .world(world)
                .unwrap()
                .iter()
                .all(|&value| { value.to_bits() == 0.0f32.to_bits() })
        );
        assert!(
            cleared_fluid
                .world(world)
                .unwrap()
                .body_force_torque
                .iter()
                .chain(cleared_fluid.world(world).unwrap().qfrc.iter())
                .all(|&value| value.to_bits() == 0.0f32.to_bits())
        );
    }
    let sleep_model = sleep_kinematic();
    let wake = TendonWakeModelInput::with_flex_positions(
        sleep_flex_model(sleep_model.clone()),
        TendonWakeFields {
            body_treeid: vec![-1, 0, 1],
            tendon_limited: vec![false],
            tendon_range: ParameterBatch::new(1, 2, vec![-10.0, 10.0]).unwrap(),
            tendon_margin: ParameterBatch::new(1, 1, vec![0.0]).unwrap(),
            sleep_enabled: true,
            island_disabled: false,
        },
    )
    .unwrap();
    let mut sleep_parameters = KinematicsParameters::default();
    sleep_parameters.set(
        KinematicsParameter::BodyInertia,
        ParameterBatch::new(1, 9, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]).unwrap(),
    );
    let sleep_source_plan = super::super::KinematicsPlan::with_tendon_wake(
        &session,
        wake,
        sleep_parameters,
        CamLightParameters::default(),
    )
    .unwrap();
    let sleep_velocity_plan = TendonVelocityPlan::new(&sleep_source_plan).unwrap();
    let sleep_joint_plan = JointPassivePlan::new(
        &session,
        sleep_model.clone(),
        sleep_joint_parameters(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    let sleep_tendon_plan = TendonPassivePlan::new(
        &sleep_velocity_plan,
        tendon_parameters(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let sleep_flex_plan = FlexEdgePassivePlan::new(
        &sleep_source_plan,
        FlexEdgePassiveInput {
            length0: vec![0.5],
            stiffness: vec![-1.0],
            damping: vec![0.25],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions::default(),
    )
    .unwrap();
    let sleep_plan = PassiveForcePlan::with_fluid_force(
        &sleep_joint_plan,
        &sleep_tendon_plan,
        &sleep_flex_plan,
        fluid_input(3),
    )
    .unwrap();
    let sleep_worlds = 2;
    let sleep_qpos = vec![0.25, -0.5, 0.75, -1.0];
    let sleep_qvel = vec![0.5, -0.75, 1.0, -1.25];
    let mut sleep_source = sleep_source_plan.create_data(sleep_worlds).unwrap();
    let mut sleep_velocity = sleep_velocity_plan.create_data(sleep_worlds).unwrap();
    let mut sleep_joint = sleep_joint_plan.create_data(sleep_worlds).unwrap();
    let mut sleep_data = sleep_plan.create_data(sleep_worlds).unwrap();
    sleep_source.write_qpos(&sleep_qpos).unwrap();
    sleep_source.write_qvel(&sleep_qvel).unwrap();
    sleep_source_plan.update(&mut sleep_source).unwrap();
    for (world, tree_asleep) in [(0, vec![-11, 1]), (1, vec![0, -11])] {
        sleep_source
            .write_world_sleep(
                world,
                &SleepTreeState {
                    tree_asleep,
                    nbody_awake: 1,
                    nv_awake: 1,
                },
            )
            .unwrap();
    }
    sleep_source_plan
        .update_tendon_wake(&mut sleep_source)
        .unwrap();
    sleep_velocity.write_qvel(&sleep_qvel).unwrap();
    sleep_joint.write_qpos(&sleep_qpos).unwrap();
    sleep_joint.write_qvel(&sleep_qvel).unwrap();
    sleep_joint_plan.update_joint(&mut sleep_joint).unwrap();
    let sleep_velocity_view = sleep_velocity_plan
        .update(&sleep_source, &mut sleep_velocity)
        .unwrap();
    let sleep_view = sleep_plan
        .update(
            &sleep_joint,
            &sleep_velocity_view,
            &sleep_source,
            &mut sleep_data,
        )
        .unwrap();
    let sleep_snapshot = sleep_view.snapshot().unwrap();
    let sleep_smooth = sleep_view.smooth().unwrap();
    let sleep_fluid = sleep_data.fluid.as_ref().unwrap().snapshot().unwrap();
    assert!((0..sleep_worlds).any(|world| {
        sleep_fluid
            .world(world)
            .unwrap()
            .qfrc
            .iter()
            .any(|&value| value != 0.0)
    }));
    for (world, awake) in [(0, [true, false]), (1, [false, true])] {
        for (dof, &awake) in awake.iter().enumerate() {
            let passive = sleep_snapshot.spring(world).unwrap()[dof]
                + sleep_snapshot.damper(world).unwrap()[dof];
            let expected = passive + sleep_fluid.world(world).unwrap().qfrc[dof];
            let actual = sleep_smooth.world(world).unwrap()[dof];
            if awake {
                assert_eq!(actual.to_bits(), expected.to_bits());
            } else {
                assert_eq!(actual.to_bits(), 0.0f32.to_bits());
            }
        }
    }

    let disabled_sleep_joint = JointPassivePlan::new(
        &session,
        sleep_model,
        sleep_joint_parameters(),
        disabled_options,
    )
    .unwrap();
    let disabled_sleep_tendon = TendonPassivePlan::new(
        &sleep_velocity_plan,
        tendon_parameters(),
        TendonPassiveOptions {
            disable_spring: true,
            disable_damper: true,
        },
    )
    .unwrap();
    let disabled_sleep_flex = FlexEdgePassivePlan::new(
        &sleep_source_plan,
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
    let disabled_sleep_plan = PassiveForcePlan::with_fluid_force(
        &disabled_sleep_joint,
        &disabled_sleep_tendon,
        &disabled_sleep_flex,
        fluid_input(3),
    )
    .unwrap();
    let mut disabled_sleep_data = disabled_sleep_plan.create_data(sleep_worlds).unwrap();
    let cold = disabled_sleep_plan
        .clear_disabled(&mut disabled_sleep_data)
        .unwrap();
    assert!(cold.smooth().is_err());
    assert!(
        disabled_sleep_data
            .fluid
            .as_ref()
            .unwrap()
            .snapshot()
            .unwrap()
            .world(0)
            .unwrap()
            .qfrc
            .iter()
            .all(|&value| value.to_bits() == 0.0f32.to_bits())
    );
    sleep_source.write_qvel(&[-2.0, 3.0, -4.0, 5.0]).unwrap();
    let filtered = disabled_sleep_plan
        .clear_disabled_with_sleep(&sleep_source, &mut disabled_sleep_data)
        .unwrap();
    for world in 0..sleep_worlds {
        assert!(
            filtered
                .smooth()
                .unwrap()
                .world(world)
                .unwrap()
                .iter()
                .all(|&value| { value.to_bits() == 0.0f32.to_bits() })
        );
    }

    let report = serde_json::json!({
        "worlds": worlds,
        "sleep_worlds": sleep_worlds,
        "composition_order": ["spring", "damper", "allowed_gravity", "fluid"],
        "fluid_parameter_periods": [2, 3, 5],
        "has_fluid_false_control": true,
        "dual_disabled_fluid_clear": true,
        "sleep_filtering": true,
        "source_write_invalidation": true,
        "same_shape_foreign_source_rejected": true,
        "mid_device_failure_blocks_publication": true,
        "healthy_repair_recovered": true,
        "same_source_resident_kinematics": true,
        "joint_state_caller_synchronized": true,
        "full_chain_same_generation": true,
        "frozen_upstream_device_output": false,
        "absolute_tolerance": 2.0e-6,
        "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_PASSIVE_FLUID_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
