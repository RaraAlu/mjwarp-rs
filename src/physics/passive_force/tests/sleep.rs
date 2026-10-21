//! 被动合力休眠过滤与恢复GPU测试。

use super::tests_support::*;
use super::*;
use crate::model::{CamLightParameters, KinematicsParameters};
use crate::model::{ParameterBatch, SleepTreeState, TendonWakeFields, TendonWakeModelInput};
use crate::physics::{
    FlexEdgePassiveInput, FlexEdgePassiveOptions, JointPassiveOptions, TendonPassiveOptions,
    TendonVelocityPlan,
};

#[test]
#[ignore = "new G04 resident sleep filtering and mid-chain recovery"]
fn gpu_filters_resident_sleep_and_recovers_after_midchain_failure() {
    let timer = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let model = sleep_kinematic();
    let wake = TendonWakeModelInput::with_flex_positions(
        sleep_flex_model(model.clone()),
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
    let source_plan = super::super::KinematicsPlan::with_tendon_wake(
        &session,
        wake,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let joint_plan = JointPassivePlan::new(
        &session,
        model.clone(),
        sleep_joint_parameters(),
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
            length0: vec![0.5],
            stiffness: vec![-1.0],
            damping: vec![0.25],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions::default(),
    )
    .unwrap();
    let plan = PassiveForcePlan::new(&joint_plan, &tendon_plan, &flex_plan).unwrap();
    assert!(plan.smooth.options().enable_sleep());
    assert_eq!(plan.smooth.input().ntree(), 2);
    let worlds = 2;
    let qpos = vec![0.25, -0.5, 0.75, -1.0];
    let qvel = vec![0.125, -0.25, 0.375, -0.5];
    let mut source = source_plan.create_data(worlds).unwrap();
    let mut velocity = velocity_plan.create_data(worlds).unwrap();
    let mut joint = joint_plan.create_data(worlds).unwrap();
    let mut data = plan.create_data(worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source.write_qvel(&qvel).unwrap();
    source_plan.update(&mut source).unwrap();
    for (world, tree_asleep) in [(0, vec![-11, 1]), (1, vec![0, -11])] {
        source
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
    source_plan.update_tendon_wake(&mut source).unwrap();
    velocity.write_qvel(&qvel).unwrap();
    joint.write_qpos(&qpos).unwrap();
    joint.write_qvel(&qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    for (field, value) in [
        (SmoothForceField::Bias, 1.0),
        (SmoothForceField::Actuator, 2.0),
        (SmoothForceField::Applied, 4.0),
    ] {
        data.write_other_force(field, &[value; 4]).unwrap();
    }
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let view = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let snapshot = view.snapshot().unwrap();
    let smooth = view.smooth().unwrap();
    for (world, awake) in [(0, [true, false]), (1, [false, true])] {
        for (dof, &awake) in awake.iter().enumerate() {
            let passive =
                snapshot.spring(world).unwrap()[dof] + snapshot.damper(world).unwrap()[dof];
            let expected = if awake {
                passive - 1.0 + 2.0 + 4.0
            } else {
                0.0
            };
            assert_eq!(
                smooth.world(world).unwrap()[dof].to_bits(),
                expected.to_bits()
            );
        }
    }
    let _ = view;
    let _ = velocity_view;

    for (world, tree_asleep) in [(0, vec![0, -11]), (1, vec![-11, 1])] {
        source
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
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    assert!(matches!(
        plan.update(&joint, &velocity_view, &source, &mut data),
        Err(TransferError::StageNotReady {
            stage: "passive_force_sleep_source"
        })
    ));
    assert!(data.snapshot().is_err());
    assert!(data.smooth_output().is_err());
    let _ = velocity_view;
    source_plan.update_tendon_wake(&mut source).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let view = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let snapshot = view.snapshot().unwrap();
    let smooth = view.smooth().unwrap();
    for (world, awake) in [(0, [false, true]), (1, [true, false])] {
        for (dof, &awake) in awake.iter().enumerate() {
            let passive =
                snapshot.spring(world).unwrap()[dof] + snapshot.damper(world).unwrap()[dof];
            let expected = if awake {
                passive - 1.0 + 2.0 + 4.0
            } else {
                0.0
            };
            assert_eq!(
                smooth.world(world).unwrap()[dof].to_bits(),
                expected.to_bits()
            );
        }
    }
    let _ = view;
    let _ = velocity_view;

    let changed_qvel = vec![-0.25, 0.5, -0.75, 1.0];
    source.write_qvel(&changed_qvel).unwrap();
    velocity.write_qvel(&changed_qvel).unwrap();
    joint.write_qvel(&changed_qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    data.parts.write_range(4, &[f32::NAN; 8]).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    assert!(matches!(
        plan.update(&joint, &velocity_view, &source, &mut data),
        Err(TransferError::StageNotReady {
            stage: "flex_edge_passive_source"
        })
    ));
    assert!(data.snapshot().is_err());
    assert!(data.smooth_output().is_err());
    let partial = read_parts(&data.parts, 8).unwrap();
    assert!(partial[4..12].iter().all(|value| value.is_finite()));
    let _ = velocity_view;
    source_plan.update_flex_edges(&mut source).unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let recovered = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let recovered_snapshot = recovered.snapshot().unwrap();
    let recovered_smooth = recovered.smooth().unwrap();
    assert!((0..worlds).any(|world| {
        recovered_snapshot.damper(world).unwrap() != snapshot.damper(world).unwrap()
    }));
    for (world, awake) in [(0, [false, true]), (1, [true, false])] {
        for (dof, &awake) in awake.iter().enumerate() {
            let passive = recovered_snapshot.spring(world).unwrap()[dof]
                + recovered_snapshot.damper(world).unwrap()[dof];
            let expected = if awake {
                passive - 1.0 + 2.0 + 4.0
            } else {
                0.0
            };
            assert_eq!(
                recovered_smooth.world(world).unwrap()[dof].to_bits(),
                expected.to_bits()
            );
        }
    }
    let _ = recovered;
    let _ = velocity_view;

    let internal_input = plan.smooth.input().clone();
    let internal_plan = SmoothForcePlan::new(
        &session,
        internal_input,
        SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        },
    )
    .unwrap();
    let mut internal = internal_plan.create_data(worlds).unwrap();
    for (field, value) in [
        (SmoothForceField::Passive, 5.0),
        (SmoothForceField::Bias, 1.0),
        (SmoothForceField::Actuator, 2.0),
        (SmoothForceField::Applied, 4.0),
    ] {
        internal.write_force(field, &[value; 4]).unwrap();
    }
    internal.write_awake(&[1, 0, 0, 1]).unwrap();
    internal_plan.update(&mut internal).unwrap();
    assert_eq!(internal.readback().unwrap().world(0).unwrap(), [10.0, 0.0]);
    assert_eq!(internal.readback().unwrap().world(1).unwrap(), [0.0, 10.0]);

    let tiny_sleep = session.upload(&[7i32]).unwrap();
    assert!(matches!(
        internal_plan.update_with_sleep_source(&mut internal, Some(&tiny_sleep)),
        Err(TransferError::Input(InputError::LengthMismatch {
            field: "smooth_force_sleep_source",
            ..
        }))
    ));
    assert!(internal.readback().is_err());
    internal_plan.update(&mut internal).unwrap();
    for options in [
        SmoothForceOptions {
            sleep_enabled: false,
            island_disabled: false,
        },
        SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: true,
        },
    ] {
        let disabled =
            SmoothForcePlan::new(&session, plan.smooth.input().clone(), options).unwrap();
        let mut disabled_data = disabled.create_data(worlds).unwrap();
        for (field, value) in [
            (SmoothForceField::Passive, 5.0),
            (SmoothForceField::Bias, 1.0),
            (SmoothForceField::Actuator, 2.0),
            (SmoothForceField::Applied, 4.0),
        ] {
            disabled_data.write_force(field, &[value; 4]).unwrap();
        }
        disabled
            .update_with_sleep_source(&mut disabled_data, Some(&tiny_sleep))
            .unwrap();
        assert!(
            disabled_data
                .readback()
                .unwrap()
                .world(0)
                .unwrap()
                .iter()
                .all(|&value| value == 10.0)
        );
    }
    let disabled_options = JointPassiveOptions {
        disable_spring: true,
        disable_damper: true,
    };
    let disabled_joint =
        JointPassivePlan::new(&session, model, sleep_joint_parameters(), disabled_options).unwrap();
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
    let mut disabled_data = disabled_plan.create_data(worlds).unwrap();
    for (field, value) in [
        (SmoothForceField::Bias, 1.0),
        (SmoothForceField::Actuator, 2.0),
        (SmoothForceField::Applied, 4.0),
    ] {
        disabled_data.write_other_force(field, &[value; 4]).unwrap();
    }
    let cold = disabled_plan.clear_disabled(&mut disabled_data).unwrap();
    for world in 0..worlds {
        assert_eq!(cold.snapshot().unwrap().spring(world).unwrap(), [0.0; 2]);
        assert_eq!(cold.snapshot().unwrap().damper(world).unwrap(), [0.0; 2]);
    }
    assert!(cold.smooth().is_err());
    source.write_qvel(&changed_qvel).unwrap();
    let filtered = disabled_plan
        .clear_disabled_with_sleep(&source, &mut disabled_data)
        .unwrap();
    assert_eq!(filtered.smooth().unwrap().world(0).unwrap(), [0.0, 5.0]);
    assert_eq!(filtered.smooth().unwrap().world(1).unwrap(), [5.0, 0.0]);
    let report = serde_json::json!({
        "worlds": worlds,
        "trees": 2,
        "awake_patterns": 4,
        "resident_device_sleep_source": true,
        "embedded_awake_path": true,
        "sleep_disabled_ignores_external_source": true,
        "island_disabled_ignores_external_source": true,
        "midchain_failure_stage": "flex_edge_passive_source",
        "partial_publication_rejected": true,
        "healthy_retry_recovered": true,
        "dual_disabled_cold_clear": true,
        "dual_disabled_sleep_merge": true,
        "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_PASSIVE_SLEEP_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
