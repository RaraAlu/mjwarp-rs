//! 流体椭球分支GPU参考测试。

#[cfg(feature = "cuda-probe")]
use super::tests_reference::*;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate::model::KinematicsParameter;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new G04 ellipsoid fluid branch"]
fn gpu_matches_independent_ellipsoid_reference() {
    const WORLDS: usize = 9;
    const TOLERANCE: f32 = 2.0e-6;
    let started = std::time::Instant::now();
    let session = crate::runtime::TransferSession::new(0).unwrap();
    let model = model();
    let parameters = kinematics_parameters();
    let plan =
        ComVelocityPlan::with_parameters(&session, model.clone(), parameters.clone()).unwrap();
    let input = ellipsoid_fluid();
    let fluid_plan = FluidForcePlan::new(&plan, input.clone()).unwrap();
    let mut source = plan.create_data(WORLDS).unwrap();
    let qpos = (0..WORLDS)
        .flat_map(|world| {
            let scale = world as f32;
            [0.07 * scale, -0.045 * scale]
        })
        .collect::<Vec<_>>();
    source.write_qpos(&qpos).unwrap();
    let qvel = (0..WORLDS)
        .flat_map(|world| {
            if world == 0 {
                [0.0, 0.0]
            } else {
                [0.12 + 0.025 * world as f32, -0.08 + 0.0175 * world as f32]
            }
        })
        .collect::<Vec<_>>();
    source.write_qvel(&qvel).unwrap();
    plan.update(&mut source).unwrap();
    let mut data = fluid_plan.create_data(WORLDS).unwrap();
    let output = fluid_plan
        .update(&mut source, &mut data)
        .unwrap()
        .snapshot()
        .unwrap();
    let fields = plan.fluid_snapshot_for_test(&source, true).unwrap();
    let mass = parameters.get(KinematicsParameter::BodyMass).unwrap();
    let inertia = parameters.get(KinematicsParameter::BodyInertia).unwrap();
    let mut compared = 0usize;
    let mut max_absolute = 0.0f32;
    let mut saw_nondegenerate = false;
    for world in 0..WORLDS {
        let (expected_body, expected_qfrc) = reference(
            &model,
            world,
            (&fields.0, &fields.1, &fields.2, fields.3.as_deref()),
            mass.world(world),
            inertia.world(world),
            &input,
        );
        let actual = output.world(world).unwrap();
        if world == 0 {
            assert_eq!(actual.body_force_torque, &[0.0; 30]);
            assert_eq!(actual.qfrc, &[0.0; 2]);
        }
        assert_eq!(&actual.body_force_torque[12..18], &[0.0; 6]);
        assert_eq!(&actual.body_force_torque[24..30], &[0.0; 6]);
        saw_nondegenerate |= actual.body_force_torque[6..12]
            .iter()
            .any(|value| *value != 0.0);
        for (index, (&actual, &expected)) in actual
            .body_force_torque
            .iter()
            .zip(expected_body.iter())
            .enumerate()
        {
            let absolute = (actual - expected).abs();
            max_absolute = max_absolute.max(absolute);
            assert!(
                absolute <= TOLERANCE,
                "world={world} body_component={index} actual={actual} expected={expected}"
            );
            compared += 1;
        }
        for (index, (&actual, &expected)) in
            actual.qfrc.iter().zip(expected_qfrc.iter()).enumerate()
        {
            let absolute = (actual - expected).abs();
            max_absolute = max_absolute.max(absolute);
            assert!(
                absolute <= TOLERANCE,
                "world={world} qfrc={index} actual={actual} expected={expected}"
            );
            compared += 1;
        }
    }
    assert!(saw_nondegenerate);
    assert_eq!(compared, 288);
    let report = serde_json::json!({
        "worlds": WORLDS,
        "compared_slots": compared,
        "absolute_tolerance": TOLERANCE,
        "max_absolute": max_absolute,
        "parameter_periods": {
            "wind": 2,
            "density": 3,
            "viscosity": 5,
            "body_mass": 7,
            "body_inertia": 11,
            "geom_size": 4,
            "geom_pos": 3,
            "geom_quat": 2
        },
        "active_geoms_on_one_body": 2,
        "zero_coefficient_skipped": true,
        "negative_coefficient_skipped": true,
        "low_mass_body_skipped": true,
        "zero_speed_boundary": true,
        "extra_lever_arm_torque": false,
        "frozen_upstream_device_output": false,
        "elapsed_ms": started.elapsed().as_millis(),
        "performance_acceptance": false
    });
    if let Some(path) = std::env::var_os("MJWARP_ELLIPSOID_FLUID_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
