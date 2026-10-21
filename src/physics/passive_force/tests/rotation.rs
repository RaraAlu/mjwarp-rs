//! 被动合力旋转、重力与肌腱组合GPU测试。

use super::tests_support::*;
use super::*;
use crate::model::ParameterBatch;
use crate::model::{CamLightParameters, KinematicsParameter, KinematicsParameters};
use crate::physics::{
    FlexEdgePassiveInput, FlexEdgePassiveOptions, GravityCompensationOptions,
    GravityCompensationParameters, GravityCompensationPlan, JointPassiveOptions,
    TendonPassiveOptions, TendonVelocityPlan,
};

#[test]
#[ignore = "new G04 gravity compensation and rotation composition"]
fn gpu_composes_rotation_tendon_flexedge_and_gravity() {
    const WORLDS: usize = 3;
    const SQRT_HALF: f32 = 0.70710677;
    let timer = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let model = rotation_kinematic();
    let mut kinematic_parameters = KinematicsParameters::default();
    let body_mass = ParameterBatch::new(
        3,
        5,
        vec![
            0.0, 1.0, 2.0, 1.0, 1.0, 0.0, 1.5, 1.0, 1.0, 1.0, 0.0, 0.5, 2.0, 1.0, 1.0,
        ],
    )
    .unwrap();
    kinematic_parameters.set(KinematicsParameter::BodyMass, body_mass.clone());
    let source_plan = super::super::KinematicsPlan::with_flex_positions(
        &session,
        rotation_flex_model(model.clone()),
        kinematic_parameters,
        CamLightParameters::default(),
    )
    .unwrap();
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let joint_plan = JointPassivePlan::new(
        &session,
        model.clone(),
        rotation_joint_parameters(),
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
            length0: vec![0.75],
            stiffness: vec![-0.5],
            damping: vec![0.25],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions::default(),
    )
    .unwrap();
    let gravity_parameters = GravityCompensationParameters {
        gravity: ParameterBatch::new(2, 3, vec![0.0, -1.0, 0.0, 0.0, -2.0, 0.0]).unwrap(),
        body_gravcomp: ParameterBatch::new(
            3,
            5,
            vec![
                0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.25, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            ],
        )
        .unwrap(),
        jnt_actgravcomp: vec![0, 1, 0, 0],
    };
    let mut gravity_plan = GravityCompensationPlan::new(
        &source_plan,
        gravity_parameters.clone(),
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let plan = PassiveForcePlan::with_gravity_compensation(
        &joint_plan,
        &tendon_plan,
        &flex_plan,
        gravity_parameters.clone(),
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut qpos = Vec::with_capacity(WORLDS * 13);
    let mut qvel = Vec::with_capacity(WORLDS * 11);
    for world in 0..WORLDS {
        let mut row = model.fields().qpos0.clone();
        row[0] = world as f32 * 0.125;
        row[1] = -(world as f32) * 0.0625;
        row[3..7].copy_from_slice(&[SQRT_HALF, 0.0, 0.0, SQRT_HALF]);
        row[7..11].copy_from_slice(&[SQRT_HALF, 0.0, SQRT_HALF, 0.0]);
        row[11] = 0.25 + world as f32 * 0.125;
        row[12] = -0.125 * (world + 1) as f32;
        qpos.extend(row);
        qvel.extend((0..11).map(|dof| (world * 11 + dof + 1) as f32 / 64.0));
    }
    let mut source = source_plan.create_data(WORLDS).unwrap();
    let mut velocity = velocity_plan.create_data(WORLDS).unwrap();
    let mut joint = joint_plan.create_data(WORLDS).unwrap();
    let mut tendon = tendon_plan.create_data(WORLDS).unwrap();
    let mut flex = flex_plan.create_data(WORLDS).unwrap();
    let mut gravity = gravity_plan.create_data(WORLDS).unwrap();
    let mut data = plan.create_data(WORLDS).unwrap();
    source.write_qpos(&qpos).unwrap();
    source.write_qvel(&qvel).unwrap();
    source_plan.update(&mut source).unwrap();
    source.write_world_qpos(0, &qpos[..13]).unwrap();
    assert!(matches!(
        gravity_plan.update(&source, &mut gravity),
        Err(TransferError::StageNotReady { .. })
    ));
    source_plan.update(&mut source).unwrap();
    velocity.write_qvel(&qvel).unwrap();
    joint.write_qpos(&qpos).unwrap();
    joint.write_qvel(&qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    let joint_output = joint.readback_joint().unwrap();
    let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
    let tendon_output = tendon_plan
        .update(&velocity_view, &mut tendon)
        .unwrap()
        .snapshot()
        .unwrap();
    let flex_output = flex_plan
        .update(&source, &mut flex)
        .unwrap()
        .snapshot()
        .unwrap();
    let gravity_output = gravity_plan
        .update(&source, &mut gravity)
        .unwrap()
        .snapshot()
        .unwrap();
    gravity_plan.test_write_root(2, i32::MAX).unwrap();
    assert!(matches!(
        gravity_plan.update(&source, &mut gravity),
        Err(TransferError::Input(InputError::InvalidTopology { .. }))
    ));
    gravity_plan.test_write_root(2, 1).unwrap();
    gravity_plan.test_write_ancestor(2, 0, 2).unwrap();
    assert!(matches!(
        gravity_plan.update(&source, &mut gravity),
        Err(TransferError::Input(InputError::InvalidTopology { .. }))
    ));
    gravity_plan.test_write_ancestor(2, 0, 1).unwrap();
    gravity_plan.test_write_dof_joint(0, i32::MAX).unwrap();
    assert!(matches!(
        gravity_plan.update(&source, &mut gravity),
        Err(TransferError::Input(InputError::InvalidTopology { .. }))
    ));
    gravity_plan.test_write_dof_joint(0, 0).unwrap();
    let recovered_gravity = gravity_plan
        .update(&source, &mut gravity)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..WORLDS {
        for (actual, expected) in recovered_gravity
            .world(world)
            .unwrap()
            .iter()
            .zip(gravity_output.world(world).unwrap())
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }
    let source_output = source.readback().unwrap();
    let view = plan
        .update(&joint, &velocity_view, &source, &mut data)
        .unwrap();
    let output = view.snapshot().unwrap();
    let smooth = view.smooth().unwrap();
    let dof_jntid = [0usize, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3];
    let dof_bodyid = [1usize, 1, 1, 1, 1, 1, 2, 2, 2, 3, 4];
    let parents = &model.fields().body_parentid;
    let mut rotation_nonzero = false;
    let mut gravity_rotation_nonzero = false;
    for world in 0..WORLDS {
        let rigid = source_output.rigid().world(world).unwrap();
        let com = source_output.com().world(world).unwrap();
        let gravity_row = gravity_parameters.gravity.world(world);
        let mass_row = body_mass.world(world);
        let gravcomp_row = gravity_parameters.body_gravcomp.world(world);
        let mut expected_gravity = [0.0f32; 11];
        let body = 2usize;
        let root = 1usize;
        let force = [
            (-gravity_row[0] * mass_row[body]) * gravcomp_row[body],
            (-gravity_row[1] * mass_row[body]) * gravcomp_row[body],
            (-gravity_row[2] * mass_row[body]) * gravcomp_row[body],
        ];
        let offset = [
            rigid.xipos[3 * body] - com.subtree_com[3 * root],
            rigid.xipos[3 * body + 1] - com.subtree_com[3 * root + 1],
            rigid.xipos[3 * body + 2] - com.subtree_com[3 * root + 2],
        ];
        for dof in 0..11 {
            let mut cursor = body;
            let mut affects = false;
            loop {
                if cursor == dof_bodyid[dof] {
                    affects = true;
                    break;
                }
                if cursor == 0 {
                    break;
                }
                cursor = parents[cursor] as usize;
            }
            if affects {
                let cdof = &com.cdof[6 * dof..6 * dof + 6];
                let jacobian = [
                    cdof[3] + (cdof[1] * offset[2] - cdof[2] * offset[1]),
                    cdof[4] + (cdof[2] * offset[0] - cdof[0] * offset[2]),
                    cdof[5] + (cdof[0] * offset[1] - cdof[1] * offset[0]),
                ];
                expected_gravity[dof] =
                    jacobian[0] * force[0] + jacobian[1] * force[1] + jacobian[2] * force[2];
            }
        }
        for dof in 0..11 {
            let shared_gravity = output.gravity(world).unwrap()[dof];
            assert_eq!(
                shared_gravity.to_bits(),
                gravity_output.world(world).unwrap()[dof].to_bits()
            );
            assert!((shared_gravity - expected_gravity[dof]).abs() <= 2.0e-6);
            let spring = ordered_atomic_sources(
                joint_output.spring(world).unwrap()[dof],
                tendon_output.spring(world).unwrap()[dof],
                flex_output.spring(world).unwrap()[dof],
            );
            let damper = ordered_atomic_sources(
                joint_output.damper(world).unwrap()[dof],
                tendon_output.damper(world).unwrap()[dof],
                flex_output.damper(world).unwrap()[dof],
            );
            assert_eq!(
                output.spring(world).unwrap()[dof].to_bits(),
                spring.to_bits()
            );
            assert_eq!(
                output.damper(world).unwrap()[dof].to_bits(),
                damper.to_bits()
            );
            let mut expected = spring + damper;
            if gravity_parameters.jnt_actgravcomp[dof_jntid[dof]] == 0 {
                expected += shared_gravity;
            }
            assert_eq!(
                smooth.world(world).unwrap()[dof].to_bits(),
                expected.to_bits()
            );
        }
        rotation_nonzero |= joint_output.spring(world).unwrap()[3..9]
            .iter()
            .any(|&value| value != 0.0);
        gravity_rotation_nonzero |= output.gravity(world).unwrap()[3..9]
            .iter()
            .any(|&value| value != 0.0);
    }
    assert!(rotation_nonzero);
    assert!(gravity_rotation_nonzero);
    assert!(tendon_output.spring(0).unwrap()[9] != 0.0);
    assert!(
        flex_output.spring(0).unwrap()[9..]
            .iter()
            .any(|&value| value != 0.0)
    );

    let disabled_gravity = GravityCompensationPlan::new(
        &source_plan,
        gravity_parameters.clone(),
        GravityCompensationOptions {
            disable_gravity: true,
        },
    )
    .unwrap();
    let cold_source = source_plan.create_data(WORLDS).unwrap();
    let mut disabled_gravity_data = disabled_gravity.create_data(WORLDS).unwrap();
    let disabled_gravity_output = disabled_gravity
        .update(&cold_source, &mut disabled_gravity_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..WORLDS {
        assert!(
            disabled_gravity_output
                .world(world)
                .unwrap()
                .iter()
                .all(|&value| value.to_bits() == 0.0f32.to_bits())
        );
    }

    let mut zero_parameters = gravity_parameters.clone();
    zero_parameters.body_gravcomp = ParameterBatch::new(1, 5, vec![0.0; 5]).unwrap();
    let zero_gravity = GravityCompensationPlan::new(
        &source_plan,
        zero_parameters,
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut zero_gravity_data = zero_gravity.create_data(WORLDS).unwrap();
    let zero_gravity_output = zero_gravity
        .update(&cold_source, &mut zero_gravity_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..WORLDS {
        assert!(
            zero_gravity_output
                .world(world)
                .unwrap()
                .iter()
                .all(|&value| value.to_bits() == 0.0f32.to_bits())
        );
    }

    let foreign_source_plan = super::super::KinematicsPlan::with_flex_positions(
        &session,
        rotation_flex_model(model.clone()),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let foreign_source = foreign_source_plan.create_data(WORLDS).unwrap();
    assert!(matches!(
        disabled_gravity.update(&foreign_source, &mut disabled_gravity_data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(matches!(
        zero_gravity.update(&foreign_source, &mut zero_gravity_data),
        Err(TransferError::ModelMismatch)
    ));

    let slots = WORLDS * 11;
    let mut negative_parts = vec![-131072.0f32; 2 * slots + 8];
    negative_parts[4..4 + 2 * slots].fill(-0.0);
    let mut positive_gravity = vec![-131072.0f32; slots + 8];
    positive_gravity[4..4 + slots].fill(0.0);
    let mut summed = vec![-131072.0f32; slots + 8];
    summed[4..4 + slots].fill(f32::NAN);
    let negative_parts = session.upload(&negative_parts).unwrap();
    let positive_gravity = session.upload(&positive_gravity).unwrap();
    let mut summed_buffer = session.upload(&summed).unwrap();
    disabled_gravity
        .sum_shared(
            &negative_parts,
            &positive_gravity,
            &mut summed_buffer,
            WORLDS,
        )
        .unwrap();
    summed_buffer.read_range_into(0, &mut summed).unwrap();
    assert!(
        summed[4..4 + slots]
            .iter()
            .all(|&value| value.to_bits() == (-0.0f32).to_bits())
    );

    let disabled_gravity_full = PassiveForcePlan::with_gravity_compensation(
        &joint_plan,
        &tendon_plan,
        &flex_plan,
        gravity_parameters.clone(),
        GravityCompensationOptions {
            disable_gravity: true,
        },
    )
    .unwrap();
    let mut disabled_gravity_full_data = disabled_gravity_full.create_data(WORLDS).unwrap();
    let disabled_gravity_view = disabled_gravity_full
        .update(
            &joint,
            &velocity_view,
            &source,
            &mut disabled_gravity_full_data,
        )
        .unwrap();
    let disabled_gravity_snapshot = disabled_gravity_view.snapshot().unwrap();
    let disabled_gravity_smooth = disabled_gravity_view.smooth().unwrap();
    let disabled_slot_zero = disabled_gravity_snapshot.spring(0).unwrap()[0]
        + disabled_gravity_snapshot.damper(0).unwrap()[0];
    assert_eq!(
        disabled_gravity_smooth.world(0).unwrap()[0].to_bits(),
        disabled_slot_zero.to_bits()
    );

    let disabled_options = JointPassiveOptions {
        disable_spring: true,
        disable_damper: true,
    };
    let disabled_joint = JointPassivePlan::new(
        &session,
        model,
        rotation_joint_parameters(),
        disabled_options,
    )
    .unwrap();
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
    let disabled_plan = PassiveForcePlan::with_gravity_compensation(
        &disabled_joint,
        &disabled_tendon,
        &disabled_flex,
        gravity_parameters,
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut disabled_data = disabled_plan.create_data(WORLDS).unwrap();
    let cleared = disabled_plan
        .clear_disabled(&mut disabled_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..WORLDS {
        assert!(
            cleared
                .gravity(world)
                .unwrap()
                .iter()
                .all(|&value| value.to_bits() == 0.0f32.to_bits())
        );
    }
    let report = serde_json::json!({
        "worlds": WORLDS,
        "compared_slots": WORLDS * 11,
        "rotation_tolerance": 2.0e-6,
        "gravity_tolerance": 2.0e-6,
        "gravity_atomic_path_executed": true,
        "multi_body_atomic_reduction_equivalence": false,
        "independent_periods": [2, 3, 3],
        "actuator_gravity_exclusion": true,
        "full_shared_rotation_composition": true,
        "disabled_gravity_preserves_negative_zero": true,
        "inactive_cold_source": true,
        "inactive_foreign_source_rejected": true,
        "active_stale_source_rejected_and_recovered": true,
        "invalid_metadata_rejected_without_oob": ["root", "dof_joint", "ancestor"],
        "dual_disabled_gravity_cleared": true,
        "reference_source": "resident candidate kinematic snapshot",
        "reference_scope": "gravity compensation and composition only",
        "frozen_upstream_device_output": false,
        "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_GRAVITY_COMPENSATION_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
