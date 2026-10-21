//! 流体惯性盒GPU参考测试。

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
#[ignore = "new G04 inertia-box fluid force and apply_ft projection"]
fn gpu_matches_independent_inertia_box_reference() {
    const WORLDS: usize = 13;
    const TOLERANCE: f32 = 2.0e-6;
    let started = std::time::Instant::now();
    let session = crate::runtime::TransferSession::new(0).unwrap();

    let simple_kinematics = simple_kinematics_parameters();
    let simple_model = simple_model();
    let simple_plan =
        ComVelocityPlan::with_parameters(&session, simple_model.clone(), simple_kinematics.clone())
            .unwrap();
    let simple_fluid = simple_fluid();
    let simple_fluid_plan = FluidForcePlan::new(&simple_plan, simple_fluid.clone()).unwrap();
    let mut simple_source = simple_plan.create_data(WORLDS).unwrap();
    simple_source.write_qpos(&[0.0; WORLDS]).unwrap();
    let simple_qvel = (0..WORLDS)
        .map(|world| (world as f32 - 6.0) * 0.125)
        .collect::<Vec<_>>();
    simple_source.write_qvel(&simple_qvel).unwrap();
    simple_plan.update(&mut simple_source).unwrap();
    let simple_fields = simple_plan
        .fluid_snapshot_for_test(&simple_source, false)
        .unwrap();
    let mut simple_data = simple_fluid_plan.create_data(WORLDS).unwrap();
    let simple_output = simple_fluid_plan
        .update(&mut simple_source, &mut simple_data)
        .unwrap()
        .snapshot()
        .unwrap();
    let simple_mass = simple_kinematics
        .get(KinematicsParameter::BodyMass)
        .unwrap();
    let simple_inertia = simple_kinematics
        .get(KinematicsParameter::BodyInertia)
        .unwrap();
    let mut compared = 0usize;
    let mut max_absolute = 0.0f32;
    for world in 0..WORLDS {
        let (expected_body, expected_qfrc) = reference(
            &simple_model,
            world,
            (
                &simple_fields.0,
                &simple_fields.1,
                &simple_fields.2,
                simple_fields.3.as_deref(),
            ),
            simple_mass.world(world),
            simple_inertia.world(world),
            &simple_fluid,
        );
        let actual = simple_output.world(world).unwrap();
        assert_eq!(&actual.body_force_torque[..6], &[0.0; 6]);
        assert_eq!(&actual.body_force_torque[12..18], &[0.0; 6]);
        for (index, (&actual, &expected)) in actual.body_force_torque[6..12]
            .iter()
            .zip(expected_body[6..12].iter())
            .enumerate()
        {
            let absolute = (actual - expected).abs();
            max_absolute = max_absolute.max(absolute);
            assert!(
                absolute <= TOLERANCE,
                "simple world={world} body_component={index} actual={actual} expected={expected}"
            );
            compared += 1;
        }
        let absolute = (actual.qfrc[0] - expected_qfrc[0]).abs();
        max_absolute = max_absolute.max(absolute);
        assert!(
            absolute <= TOLERANCE,
            "simple world={world} qfrc actual={} expected={}",
            actual.qfrc[0],
            expected_qfrc[0]
        );
        compared += 1;
    }
    assert_eq!(compared, 91);

    let kinematics = kinematics_parameters();
    let complex_model = model();
    let plan =
        ComVelocityPlan::with_parameters(&session, complex_model.clone(), kinematics.clone())
            .unwrap();
    let fluid_input = fluid();
    let mut fluid_plan = FluidForcePlan::new(&plan, fluid_input.clone()).unwrap();
    let mut source = plan.create_data(WORLDS).unwrap();
    let qpos = (0..WORLDS)
        .flat_map(|world| [(world as f32 - 6.0) * 0.11, (3.0 - world as f32) * 0.07])
        .collect::<Vec<_>>();
    source.write_qpos(&qpos).unwrap();
    let qvel = (0..WORLDS)
        .flat_map(|world| [0.2 + world as f32 * 0.03, -0.15 + world as f32 * 0.025])
        .collect::<Vec<_>>();
    source.write_qvel(&qvel).unwrap();
    plan.update(&mut source).unwrap();
    let source_fields = plan.fluid_snapshot_for_test(&source, false).unwrap();
    let mut data = fluid_plan.create_data(WORLDS).unwrap();
    let output = fluid_plan
        .update(&mut source, &mut data)
        .unwrap()
        .snapshot()
        .unwrap();

    let mass = kinematics.get(KinematicsParameter::BodyMass).unwrap();
    let inertia = kinematics.get(KinematicsParameter::BodyInertia).unwrap();
    for world in 0..WORLDS {
        let (expected_body, expected_qfrc) = reference(
            &complex_model,
            world,
            (
                &source_fields.0,
                &source_fields.1,
                &source_fields.2,
                source_fields.3.as_deref(),
            ),
            mass.world(world),
            inertia.world(world),
            &fluid_input,
        );
        let actual = output.world(world).unwrap();
        assert_eq!(&actual.body_force_torque[..6], &[0.0; 6]);
        assert_eq!(&actual.body_force_torque[24..30], &[0.0; 6]);
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
    assert_eq!(compared, 507);
    assert!(output.world(WORLDS).is_err());
    fluid_plan.corrupt_metadata_for_test(11, i32::MAX).unwrap();
    assert!(matches!(
        fluid_plan.update(&mut source, &mut data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "fluid_force_source",
            reason: "guard_mapping_or_nonfinite",
            ..
        }))
    ));
    let report = serde_json::json!({
        "worlds": WORLDS,
        "compared_slots": compared,
        "preserved_simple_slots": 91,
        "expanded_slots": 416,
        "absolute_tolerance": TOLERANCE,
        "max_absolute": max_absolute,
        "parameter_periods": [2, 3, 5, 7, 11],
        "mass_below_minval_body": 2,
        "wind_short_circuit_worlds": 7,
        "nonidentity_rotation": true,
        "nonzero_com_offset": true,
        "multi_body_projection": true,
        "invalid_root_rejected": true,
        "ellipsoid_case_in_this_test": false,
        "frozen_upstream_device_output": false,
        "elapsed_ms": started.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_FLUID_FORCE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
