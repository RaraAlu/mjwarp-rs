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

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new G04 shared resident ellipsoid path"]
fn gpu_shared_resident_ellipsoid_matches_independent_reference() {
    use crate::model::{
        CamLightFields, CamLightModelInput, FlexEdgeFields, FlexPositionFields,
        FlexPositionModelInput, MocapModelInput, TendonFields, TendonModelInput,
    };
    use crate::physics::KinematicsPlan;
    use crate::physics::com_velocity::ResidentComVelocityPlan;

    const WORLDS: usize = 9;
    const TOLERANCE: f32 = 2.0e-6;
    let started = std::time::Instant::now();
    let session = crate::runtime::TransferSession::new(0).unwrap();
    let model = model();
    let parameters = kinematics_parameters();
    let input = ellipsoid_fluid();
    let nbody = model.rigid().kinematics().nbody();
    let flex = FlexPositionModelInput::new(
        TendonModelInput::new(
            CamLightModelInput::new(
                MocapModelInput::new(model.clone(), vec![-1; nbody]).unwrap(),
                CamLightFields::default(),
            )
            .unwrap(),
            TendonFields::default(),
        )
        .unwrap(),
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_nodebodyid: vec![],
            flex_node: vec![],
            flex_vert: vec![0.0; 6],
            flex_vert0: vec![0.0; 6],
            flex_vertbodyid: vec![0, 1],
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![1],
        flexedge_j_colind: vec![0],
    })
    .unwrap();
    let resident = KinematicsPlan::with_flex_positions(
        &session,
        flex,
        parameters.clone(),
        crate::model::CamLightParameters::default(),
    )
    .unwrap();
    let shared_plan = FluidForcePlan::from_resident(&resident, input.clone()).unwrap();
    let mut source = resident.create_data(WORLDS).unwrap();
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
    resident.update(&mut source).unwrap();

    // 独立参考输入取常驻速度子集自身，产品输出不反馈参考。
    let velocity_plan = ResidentComVelocityPlan::new(&resident).unwrap();
    let mut velocity_data = velocity_plan.create_data(WORLDS).unwrap();
    velocity_plan.update(&source, &mut velocity_data).unwrap();
    let inputs = velocity_plan
        .fluid_inputs(&source, &velocity_data, true)
        .unwrap();
    let read = |buffer: &crate::runtime::TransferBuffer<f32>| {
        let mut values = crate::runtime::host_staging::<f32>(buffer.len()).unwrap();
        buffer.read_range_into(0, &mut values).unwrap();
        values
    };
    let rigid = read(inputs.rigid);
    let com = read(inputs.com);
    let velocity = read(inputs.velocity);
    let attached = inputs.attached.map(read);

    let mut data = shared_plan.create_data(WORLDS).unwrap();
    shared_plan.update_resident(&source, &mut data).unwrap();
    let output = data.snapshot().unwrap();
    let mass = parameters.get(KinematicsParameter::BodyMass).unwrap();
    let inertia = parameters.get(KinematicsParameter::BodyInertia).unwrap();
    let mut compared = 0usize;
    let mut max_absolute = 0.0f32;
    let mut saw_ellipsoid_force = false;
    for world in 0..WORLDS {
        let (expected_body, expected_qfrc) = reference(
            &model,
            world,
            (&rigid, &com, &velocity, attached.as_deref()),
            mass.world(world),
            inertia.world(world),
            &input,
        );
        let actual = output.world(world).unwrap();
        saw_ellipsoid_force |= actual.body_force_torque.iter().any(|value| *value != 0.0);
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
    assert!(saw_ellipsoid_force);
    assert_eq!(compared, 288);
    let report = serde_json::json!({
        "worlds": WORLDS,
        "compared_slots": compared,
        "absolute_tolerance": TOLERANCE,
        "max_absolute": max_absolute,
        "shared_resident_source": true,
        "independent_reference": true,
        "ellipsoid_branch": true,
        "frozen_upstream_device_output": false,
        "elapsed_ms": started.elapsed().as_millis(),
        "performance_acceptance": false
    });
    if let Some(path) = std::env::var_os("MJWARP_SHARED_ELLIPSOID_FLUID_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
