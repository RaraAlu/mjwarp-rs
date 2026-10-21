//! 重力补偿单元测试。

use super::*;
use crate::model::{
    AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput,
};
#[cfg(feature = "cuda-probe")]
use crate::model::{KinematicsParameter, KinematicsParameters};

fn source_model() -> AttachedModelInput {
    let kinematic = KinematicModelInput::new(
        1,
        KinematicFields {
            qpos0: vec![0.0],
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, 0],
            body_jntnum: vec![0, 1],
            body_pos: vec![0.0; 6],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            jnt_type: vec![2],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0; 3],
            jnt_axis: vec![1.0, 0.0, 0.0],
        },
    )
    .unwrap();
    let inertial = InertialModelInput::new(
        kinematic,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0, 1.0],
            body_inertia: vec![0.0; 6],
            dof_bodyid: vec![1],
            dof_jntid: vec![0],
            dof_parentid: vec![-1],
            dof_armature: vec![0.0],
            dof_damping: vec![0.0],
        },
    )
    .unwrap();
    AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn multi_body_model() -> AttachedModelInput {
    let kinematic = KinematicModelInput::new(
        4,
        KinematicFields {
            qpos0: vec![0.0; 4],
            body_parentid: vec![0, 0, 1, 2, 0],
            body_jntadr: vec![-1, 0, 1, 2, 3],
            body_jntnum: vec![0, 1, 1, 1, 1],
            body_pos: vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 4.0, 0.0, 0.0, -2.0, 0.0, 0.0,
            ],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            jnt_type: vec![3; 4],
            jnt_bodyid: vec![1, 2, 3, 4],
            jnt_qposadr: vec![0, 1, 2, 3],
            jnt_dofadr: vec![0, 1, 2, 3],
            jnt_pos: vec![0.0; 12],
            jnt_axis: [0.0, 0.0, 1.0].repeat(4),
        },
    )
    .unwrap();
    let inertial = InertialModelInput::new(
        kinematic,
        InertialFields {
            body_ipos: vec![
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            ],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            body_mass: vec![0.0, 1.0, 2.0, 4.0, 8.0],
            body_inertia: vec![0.0; 15],
            dof_bodyid: vec![1, 2, 3, 4],
            dof_jntid: vec![0, 1, 2, 3],
            dof_parentid: vec![-1, 0, 1, -1],
            dof_armature: vec![0.0; 4],
            dof_damping: vec![0.0; 4],
        },
    )
    .unwrap();
    AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
}

#[test]
fn host_derives_roots_ancestry_and_independent_periods() {
    let model = Arc::new(source_model());
    let params = GravityCompensationParameters {
        gravity: ParameterBatch::new(2, 3, vec![0.0; 6]).unwrap(),
        body_gravcomp: ParameterBatch::new(5, 2, vec![0.5; 10]).unwrap(),
        jnt_actgravcomp: vec![1],
    };
    let body_mass = ParameterBatch::new(3, 2, vec![1.0; 6]).unwrap();
    let value = spec(
        model,
        body_mass,
        params,
        GravityCompensationOptions::default(),
    )
    .unwrap();
    assert_eq!(value.offsets, [0, 6, 12]);
    assert_eq!(value.parameter_len, 22);
    assert_eq!(&value.metadata[4..6], &[0, 1]);
    assert_eq!(&value.metadata[6..8], &[0, 1]);
    assert!(value.active);
    assert!(layout(257, 1).is_ok());
    assert!(layout(0, 1).is_err());
    assert!(layout(1, usize::MAX).is_err());
}

#[test]
fn host_rejects_shapes_and_noncanonical_actuator_flags() {
    let model = Arc::new(source_model());
    let body_mass = ParameterBatch::new(1, 2, vec![0.0, 1.0]).unwrap();
    for bad in [
        GravityCompensationParameters {
            gravity: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
            body_gravcomp: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
            jnt_actgravcomp: vec![0],
        },
        GravityCompensationParameters {
            gravity: ParameterBatch::new(1, 3, vec![0.0; 3]).unwrap(),
            body_gravcomp: ParameterBatch::new(1, 1, vec![0.0]).unwrap(),
            jnt_actgravcomp: vec![0],
        },
        GravityCompensationParameters {
            gravity: ParameterBatch::new(1, 3, vec![0.0; 3]).unwrap(),
            body_gravcomp: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
            jnt_actgravcomp: vec![2],
        },
    ] {
        assert!(
            spec(
                model.clone(),
                body_mass.clone(),
                bad,
                GravityCompensationOptions::default(),
            )
            .is_err()
        );
    }
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "new G04 multi-body gravity reduction certificate"]
fn gpu_matches_independent_multi_body_reduction_certificate() {
    const WORLDS: usize = 7;
    const EXPECTED: [[f32; 4]; WORLDS] = [
        [35.0, 22.0, 4.0, 8.0],
        [5.5, 3.0, 0.5, 0.5],
        [10.5, 4.5, 0.5, 2.0],
        [8.25, 5.25, 1.0, 4.0],
        [10.0, 4.5, 0.5, 2.0],
        [13.25, 7.0, 1.0, 0.5],
        [62.5, 42.0, 8.0, 2.0],
    ];

    let timer = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let body_mass = ParameterBatch::new(
        3,
        5,
        vec![
            0.0, 1.0, 2.0, 4.0, 8.0, 0.0, 2.0, 1.0, 0.5, 4.0, 0.0, 0.5, 4.0, 2.0, 1.0,
        ],
    )
    .unwrap();
    let mut source_parameters = KinematicsParameters::default();
    source_parameters.set(KinematicsParameter::BodyMass, body_mass);
    let source_plan =
        KinematicsPlan::with_parameters(&session, multi_body_model(), source_parameters).unwrap();
    let parameters = GravityCompensationParameters {
        gravity: ParameterBatch::new(2, 3, vec![0.0, -1.0, 0.0, 0.0, -0.5, 0.0]).unwrap(),
        body_gravcomp: ParameterBatch::new(
            5,
            5,
            vec![
                0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.5, 1.0, 2.0, 0.25, 0.0, 2.0, 0.5, 0.25, 2.0, 0.0,
                1.0, 0.25, 0.5, 1.0, 0.0, 0.25, 2.0, 1.0, 0.5,
            ],
        )
        .unwrap(),
        jnt_actgravcomp: vec![0; 4],
    };
    let plan = GravityCompensationPlan::new(
        &source_plan,
        parameters,
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut source = source_plan.create_data(WORLDS).unwrap();
    source.write_qpos(&[0.0; WORLDS * 4]).unwrap();
    source_plan.update_rigid(&mut source).unwrap();
    source_plan.update_com(&mut source).unwrap();
    let mut data = plan.create_data(WORLDS).unwrap();
    let output = plan.update(&source, &mut data).unwrap().snapshot().unwrap();
    let (_, com_buffer) = source_plan.gravity_compensation_source(&source).unwrap();
    let com_stride = 14 * 5 + 6 * 4;
    let mut com_values = crate::runtime::host_staging::<f32>(WORLDS * com_stride + 8).unwrap();
    com_buffer.read_range_into(0, &mut com_values).unwrap();
    super::super::check_device_values(&com_values, "gravity_multi_body_com_diagnostic").unwrap();

    let mut bit_differences = 0usize;
    let mut max_absolute = 0.0f32;
    let mut max_ulp = 0u32;
    let mut comparisons = Vec::with_capacity(WORLDS * 4);
    for (world, expected) in EXPECTED.iter().enumerate() {
        let actual = output.world(world).unwrap();
        for (dof, (&actual, &expected)) in actual.iter().zip(expected).enumerate() {
            let absolute = (actual - expected).abs();
            let ulp = actual.to_bits().abs_diff(expected.to_bits());
            max_absolute = max_absolute.max(absolute);
            assert!(
                absolute <= 2.0e-6,
                "world={world} dof={dof} actual={actual} expected={expected}"
            );
            if actual.to_bits() != expected.to_bits() {
                bit_differences += 1;
                max_ulp = max_ulp.max(ulp);
            }
            comparisons.push(serde_json::json!({
                "world": world,
                "dof": dof,
                "actual": actual,
                "expected": expected,
                "absolute": absolute,
                "actual_bits": format!("{:08x}", actual.to_bits()),
                "expected_bits": format!("{:08x}", expected.to_bits()),
                "ulp": ulp,
            }));
        }
    }
    let candidate_com = (0..WORLDS)
        .map(|world| {
            let row = &com_values[4 + world * com_stride..4 + (world + 1) * com_stride];
            serde_json::json!({
                "world": world,
                "subtree_com": &row[5..20],
                "cdof": &row[70..94],
            })
        })
        .collect::<Vec<_>>();

    let report = serde_json::json!({
        "worlds": WORLDS,
        "compared_slots": WORLDS * 4,
        "gravity_tolerance": 2.0e-6,
        "bit_differences": bit_differences,
        "max_ulp": max_ulp,
        "max_absolute": max_absolute,
        "comparisons": comparisons,
        "candidate_com_diagnostics": candidate_com,
        "same_dof_contributors": 3,
        "independent_roots": 2,
        "cross_root_ancestor_exclusion": true,
        "nonzero_rotational_moments": true,
        "independent_periods": [2, 3, 5],
        "reference_source": "frozen analytic lever-arm certificate",
        "candidate_snapshot_used_by_reference": false,
        "frozen_upstream_device_output": false,
        "nonexact_reduction_order_verified": false,
        "diagnostic_elapsed_ms": timer.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_GRAVITY_MULTI_BODY_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
    assert_eq!(
        bit_differences, 0,
        "frozen bit certificate differs; see the diagnostic report"
    );
}
