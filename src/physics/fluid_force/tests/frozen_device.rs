//! 冻结上游流体内核设备输出对照（U038 `2026-10-16-next-u038-reference`）。
//!
//! 参考来自离线 authoring 生成的冻结上游设备输出（fixtures/.../u038-reference/fluid）；
//! 产品测试只读夹具，不启动 Python。容差沿用 2e-6，未改断言/内核/ABI。

use super::*;
use crate::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
    FlexEdgeFields, FlexPositionFields, FlexPositionModelInput, InertialFields, InertialModelInput,
    KinematicFields, KinematicModelInput, KinematicsParameter, KinematicsParameters,
    MocapModelInput, ParameterBatch, TendonFields, TendonModelInput,
};
use crate::physics::KinematicsPlan;
use crate::physics::com_velocity::ResidentComVelocityPlan;
use crate::runtime::TransferSession;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const TOLERANCE: f32 = 2.0e-6;
const FIXTURE: &str = "fixtures/gravity-upstream-abi/u038-reference/fluid";
const ENTRY: &str = "_fluid_force_83bf3546_cuda_kernel_forward";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn read_json(path: &Path) -> Value {
    let bytes =
        std::fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn digest(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

/// 校验夹具回执：每个文件的 sha256 与回执一致，入口名是冻结上游入口。
fn verify_receipt(directory: &Path) -> Value {
    let receipt = read_json(&directory.join("receipt.json"));
    assert_eq!(receipt["entry"], ENTRY);
    assert_eq!(
        receipt["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let files = receipt["files"].as_object().expect("receipt files");
    for (name, expected) in files {
        let actual = digest(&directory.join(name));
        assert_eq!(
            actual,
            expected.as_str().unwrap(),
            "fixture hash mismatch: {name}"
        );
    }
    receipt
}

fn f32_values(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item.as_f64().expect("number") as f32)
        .collect()
}

fn i32_values(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item.as_i64().expect("integer") as i32)
        .collect()
}

fn flatten_rows(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(f32_values)
        .collect()
}

const PER_JOINT_DOFS: [usize; 4] = [6, 3, 1, 1];

fn dof_layout(
    jnt_type: &[i32],
    jnt_bodyid: &[i32],
    jnt_dofadr: &[i32],
    body_parentid: &[i32],
    nv: usize,
) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut last_dof = vec![-1i32; body_parentid.len()];
    let mut dof_bodyid = vec![i32::MIN; nv];
    let mut dof_jntid = vec![i32::MIN; nv];
    let mut dof_parentid = vec![-1i32; nv];
    for (joint, &kind) in jnt_type.iter().enumerate() {
        let mut parent_body = body_parentid[jnt_bodyid[joint] as usize];
        let mut parent = -1i32;
        while parent_body > 0 {
            if last_dof[parent_body as usize] >= 0 {
                parent = last_dof[parent_body as usize];
                break;
            }
            parent_body = body_parentid[parent_body as usize];
        }
        let count = PER_JOINT_DOFS[kind as usize];
        let start = jnt_dofadr[joint] as usize;
        assert!(start + count <= nv);
        for step in 0..count {
            let dof = start + step;
            assert_eq!(dof_bodyid[dof], i32::MIN, "overlapping dof addresses");
            dof_bodyid[dof] = jnt_bodyid[joint];
            dof_jntid[dof] = joint as i32;
            dof_parentid[dof] = parent;
            parent = dof as i32;
        }
        last_dof[jnt_bodyid[joint] as usize] = (start + count - 1) as i32;
    }
    assert!(!dof_bodyid.contains(&i32::MIN), "uncovered dof addresses");
    (dof_bodyid, dof_jntid, dof_parentid)
}

#[test]
#[ignore = "U038 frozen upstream fluid device comparison (requires GPU and fixture)"]
fn gpu_shared_fluid_matches_frozen_upstream_device() {
    let directory = fixture_dir();
    let _receipt = verify_receipt(&directory);
    let scenario = read_json(&directory.join("scenario.json"));
    let output = read_json(&directory.join("output.json"));
    assert_eq!(output["entry"], ENTRY);
    assert!(output["deterministic_runs"].as_bool().unwrap());

    let worlds = scenario["worlds"].as_u64().unwrap() as usize;
    let nbody = scenario["nbody"].as_u64().unwrap() as usize;
    let njnt = scenario["njnt"].as_u64().unwrap() as usize;
    let nq = scenario["nq"].as_u64().unwrap() as usize;
    let nv = scenario["nv"].as_u64().unwrap() as usize;
    let ngeom = scenario["ngeom"].as_u64().unwrap() as usize;
    let body_parentid = i32_values(&scenario["body_parentid"]);
    let jnt_type = i32_values(&scenario["jnt_type"]);
    let jnt_bodyid = i32_values(&scenario["jnt_bodyid"]);
    let jnt_dofadr = i32_values(&scenario["jnt_dofadr"]);
    assert_eq!(jnt_type.len(), njnt);
    assert_eq!(jnt_bodyid.len(), njnt);
    assert_eq!(jnt_dofadr.len(), njnt);
    let (dof_bodyid, dof_jntid, dof_parentid) =
        dof_layout(&jnt_type, &jnt_bodyid, &jnt_dofadr, &body_parentid, nv);
    let qpos = flatten_rows(&scenario["qpos"]);
    let qvel = flatten_rows(&scenario["qvel"]);
    assert_eq!(qpos.len(), worlds * nq);
    assert_eq!(qvel.len(), worlds * nv);
    let mut body_jntadr = vec![-1i32; nbody];
    let mut body_jntnum = vec![0i32; nbody];
    for (joint, &body) in jnt_bodyid.iter().enumerate() {
        let body = body as usize;
        if body_jntadr[body] < 0 {
            body_jntadr[body] = joint as i32;
        }
        body_jntnum[body] += 1;
    }

    let model = AttachedModelInput::new(
        InertialModelInput::new(
            KinematicModelInput::new(
                nq,
                KinematicFields {
                    qpos0: f32_values(&scenario["qpos0"]),
                    body_parentid,
                    body_jntadr,
                    body_jntnum,
                    body_pos: flatten_rows(&scenario["body_pos"]),
                    body_quat: flatten_rows(&scenario["body_quat_wxyz"]),
                    jnt_type: jnt_type.clone(),
                    jnt_bodyid: jnt_bodyid.clone(),
                    jnt_qposadr: i32_values(&scenario["jnt_qposadr"]),
                    jnt_dofadr: jnt_dofadr.clone(),
                    jnt_pos: flatten_rows(&scenario["jnt_pos"]),
                    jnt_axis: flatten_rows(&scenario["jnt_axis"]),
                },
            )
            .unwrap(),
            InertialFields {
                body_ipos: flatten_rows(&scenario["body_ipos"]),
                body_iquat: flatten_rows(&scenario["body_iquat_wxyz"]),
                body_mass: flatten_rows(&scenario["body_mass"]),
                body_inertia: flatten_rows(&scenario["body_inertia"]),
                dof_bodyid: dof_bodyid.clone(),
                dof_jntid: dof_jntid.clone(),
                dof_parentid,
                dof_armature: vec![0.0; nv],
                dof_damping: vec![0.0; nv],
            },
        )
        .unwrap(),
        AttachedFields {
            geom_bodyid: i32_values(&scenario["geom_bodyid"]),
            geom_pos: flatten_rows(&scenario["geom_pos"]),
            geom_quat: flatten_rows(&scenario["geom_quat"]),
            ..Default::default()
        },
    )
    .unwrap();

    let periods = &scenario["periods"];
    let period = |name: &str| periods[name].as_u64().unwrap() as usize;
    let mut parameters = KinematicsParameters::default();
    parameters.set(
        KinematicsParameter::BodyMass,
        ParameterBatch::new(
            period("body_mass"),
            nbody,
            flatten_rows(&scenario["body_mass"]),
        )
        .unwrap(),
    );
    parameters.set(
        KinematicsParameter::BodyInertia,
        ParameterBatch::new(
            period("body_inertia"),
            3 * nbody,
            flatten_rows(&scenario["body_inertia"]),
        )
        .unwrap(),
    );
    parameters.set(
        KinematicsParameter::GeomPos,
        ParameterBatch::new(
            period("geom_pos"),
            3 * ngeom,
            flatten_rows(&scenario["geom_pos_periods"]),
        )
        .unwrap(),
    );
    parameters.set(
        KinematicsParameter::GeomQuat,
        ParameterBatch::new(
            period("geom_quat"),
            4 * ngeom,
            flatten_rows(&scenario["geom_quat_periods"]),
        )
        .unwrap(),
    );
    let input = FluidForceInput {
        wind: ParameterBatch::new(period("wind"), 3, flatten_rows(&scenario["wind"])).unwrap(),
        density: ParameterBatch::new(period("density"), 1, f32_values(&scenario["density"]))
            .unwrap(),
        viscosity: ParameterBatch::new(period("viscosity"), 1, f32_values(&scenario["viscosity"]))
            .unwrap(),
        geom_size: ParameterBatch::new(
            period("geom_size"),
            3 * ngeom,
            flatten_rows(&scenario["geom_size_periods"]),
        )
        .unwrap(),
        geom_type: i32_values(&scenario["geom_type"]),
        geom_fluid: f32_values(&scenario["geom_fluid"]),
        body_fluid_ellipsoid: scenario["body_fluid_ellipsoid"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_bool().unwrap())
            .collect(),
    };

    let session = TransferSession::new(0).unwrap();
    // 共享常驻链需要完整 G01 姿态/速度写入路径（边计算启用），柔体字段不参与流体输出。
    let flex = FlexPositionModelInput::new(
        TendonModelInput::new(
            CamLightModelInput::new(
                MocapModelInput::new(model, vec![-1; nbody]).unwrap(),
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
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        flex,
        parameters,
        CamLightParameters::default(),
    )
    .unwrap();
    let mut source = plan.create_data(worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source.write_qvel(&qvel).unwrap();
    plan.update(&mut source).unwrap();
    let velocity = ResidentComVelocityPlan::new(&plan).unwrap();
    let mut velocity_data = velocity.create_data(worlds).unwrap();
    velocity.update(&source, &mut velocity_data).unwrap();
    let fluid = FluidForcePlan::from_resident(&plan, input).unwrap();
    let mut data = fluid.create_data(worlds).unwrap();
    fluid.update_resident(&source, &mut data).unwrap();
    let snapshot = data.snapshot().unwrap();

    let expected = f32_values(&output["values"]);
    let expected_bits = output["bits"].as_array().unwrap();
    assert_eq!(expected.len(), worlds * nbody * 6);
    let (mut compared, mut bit_differences, mut max_absolute) = (0usize, 0usize, 0.0f32);
    let mut first_difference: Option<(usize, usize, usize, f32, f32)> = None;
    for world in 0..worlds {
        let actual = snapshot.world(world).unwrap();
        assert_eq!(actual.body_force_torque.len(), nbody * 6);
        for (index, (&actual_value, &expected_value)) in actual
            .body_force_torque
            .iter()
            .zip(expected[world * nbody * 6..(world + 1) * nbody * 6].iter())
            .enumerate()
        {
            let absolute = (actual_value - expected_value).abs();
            max_absolute = max_absolute.max(absolute);
            if actual_value.to_bits() != expected_value.to_bits() {
                bit_differences += 1;
                first_difference.get_or_insert((
                    world,
                    index / 6,
                    index % 6,
                    actual_value,
                    expected_value,
                ));
            }
            assert!(
                absolute <= TOLERANCE,
                "world={world} body={} component={} actual={actual_value} expected={expected_value} abs={absolute}",
                index / 6,
                index % 6
            );
            compared += 1;
        }
    }
    let expected_hex = expected
        .iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>();
    let archived_hex = expected_bits
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(expected_hex, archived_hex);
    let report = serde_json::json!({
        "worlds": worlds,
        "nbody": nbody,
        "ngeom": ngeom,
        "compared_slots": compared,
        "absolute_tolerance": TOLERANCE,
        "bit_differences": bit_differences,
        "max_absolute": max_absolute,
        "first_difference": first_difference.map(|(world, body, component, actual, expected)| serde_json::json!({
            "world": world,
            "body": body,
            "component": component,
            "actual": actual,
            "expected": expected,
            "actual_bits": format!("{:08x}", actual.to_bits()),
            "expected_bits": format!("{:08x}", expected.to_bits()),
        })),
        "fixture": FIXTURE,
        "fixture_entry": ENTRY,
        "reference_source": "frozen upstream passive _fluid_force device readback (offline authoring)",
        "frozen_upstream_device_output": true,
        "shared_resident_path": true,
        "python_started_by_test": false,
        "product_kernel_or_tolerance_changed": false,
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_U038_FLUID_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
