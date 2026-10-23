//! 冻结上游 `_gravity_force` 设备输出对照（多体原子归约）。
//!
//! 参考来自离线 authoring 的冻结被动内核设备读回；测试只读夹具，不启动 Python。
//! 容差沿用 2e-6，证书断言与容差不动。

use super::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const TOLERANCE: f32 = 2.0e-6;
const FIXTURE: &str = "fixtures/gravity-upstream-abi/u038-reference/atomic";
const ENTRY: &str = "_gravity_force_7c63ac28_cuda_kernel_forward";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn digest(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

/// 回执覆盖全部夹具文件的 sha256、冻结入口与上游提交。
fn verify_receipt(directory: &Path) -> Value {
    let receipt = read_json(&directory.join("receipt.json"));
    assert_eq!(receipt["entry"], ENTRY);
    assert_eq!(
        receipt["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    for (name, expected) in receipt["files"].as_object().expect("receipt files") {
        assert_eq!(
            digest(&directory.join(name)),
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

/// 按关节布局推导 dof 三元组，并用显式 dof 地址交叉校验。
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
#[ignore = "U038 frozen upstream multi-body atomic reduction device comparison"]
fn gpu_matches_frozen_upstream_atomic_device_output() {
    let directory = fixture_dir();
    let _receipt = verify_receipt(&directory);
    let scenario = read_json(&directory.join("scenario.json"));
    let output = read_json(&directory.join("output.json"));
    assert_eq!(output["entry"], ENTRY);
    assert_eq!(output["field"], "qfrc_gravcomp");

    let worlds = scenario["worlds"].as_u64().unwrap() as usize;
    let nbody = scenario["nbody"].as_u64().unwrap() as usize;
    let njnt = scenario["njnt"].as_u64().unwrap() as usize;
    let nq = scenario["nq"].as_u64().unwrap() as usize;
    let nv = scenario["nv"].as_u64().unwrap() as usize;
    assert_eq!(nq, nv);
    let body_parentid = i32_values(&scenario["body_parentid"]);
    let jnt_type = i32_values(&scenario["jnt_type"]);
    let jnt_bodyid = i32_values(&scenario["jnt_bodyid"]);
    let jnt_dofadr = i32_values(&scenario["jnt_dofadr"]);
    assert_eq!(jnt_type.len(), njnt);
    assert_eq!(jnt_bodyid.len(), njnt);
    assert_eq!(jnt_dofadr.len(), njnt);
    let (dof_bodyid, dof_jntid, dof_parentid) =
        dof_layout(&jnt_type, &jnt_bodyid, &jnt_dofadr, &body_parentid, nv);
    let mut body_jntadr = vec![-1i32; nbody];
    let mut body_jntnum = vec![0i32; nbody];
    for (joint, &body) in jnt_bodyid.iter().enumerate() {
        let body = body as usize;
        if body_jntadr[body] < 0 {
            body_jntadr[body] = joint as i32;
        }
        body_jntnum[body] += 1;
    }
    let qpos = flatten_rows(&scenario["qpos"]);
    assert_eq!(qpos.len(), worlds * nq);

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
                    jnt_type,
                    jnt_bodyid,
                    jnt_qposadr: i32_values(&scenario["jnt_qposadr"]),
                    jnt_dofadr,
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
                dof_bodyid,
                dof_jntid,
                dof_parentid,
                dof_armature: vec![0.0; nv],
                dof_damping: vec![0.0; nv],
            },
        )
        .unwrap(),
        AttachedFields::default(),
    )
    .unwrap();

    let periods = &scenario["periods"];
    let period = |name: &str| periods[name].as_u64().unwrap() as usize;
    let body_mass = ParameterBatch::new(
        period("body_mass"),
        nbody,
        flatten_rows(&scenario["body_mass"]),
    )
    .unwrap();
    let mut source_parameters = KinematicsParameters::default();
    source_parameters.set(KinematicsParameter::BodyMass, body_mass);
    let timer = std::time::Instant::now();
    let session = TransferSession::new(0).unwrap();
    let source_plan = KinematicsPlan::with_parameters(&session, model, source_parameters).unwrap();
    let parameters = GravityCompensationParameters {
        gravity: ParameterBatch::new(period("gravity"), 3, flatten_rows(&scenario["gravity"]))
            .unwrap(),
        body_gravcomp: ParameterBatch::new(
            period("body_gravcomp"),
            nbody,
            flatten_rows(&scenario["body_gravcomp"]),
        )
        .unwrap(),
        jnt_actgravcomp: i32_values(&scenario["jnt_actgravcomp"]),
    };
    let plan = GravityCompensationPlan::new(
        &source_plan,
        parameters,
        GravityCompensationOptions::default(),
    )
    .unwrap();
    let mut source = source_plan.create_data(worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source_plan.update_rigid(&mut source).unwrap();
    source_plan.update_com(&mut source).unwrap();
    let mut data = plan.create_data(worlds).unwrap();
    let view = plan.update(&source, &mut data).unwrap().snapshot().unwrap();

    let expected = f32_values(&output["values"]);
    assert_eq!(expected.len(), worlds * nv);
    let natural = read_json(&directory.join("natural-validation.json"));
    let f64_recompute = natural["independent_f64_recompute_max_abs"]
        .as_f64()
        .expect("independent f64 recompute");
    assert!(f64_recompute <= 1.0e-3);
    let ancestor = natural["topology"]["body_isdofancestor"]
        .as_array()
        .expect("body_isdofancestor");
    let max_contributors = (0..nv)
        .map(|dof| {
            (1..nbody)
                .filter(|&body| ancestor[body][dof].as_i64() == Some(1))
                .count()
        })
        .max()
        .unwrap_or(0);
    assert!(
        max_contributors > 1,
        "scenario does not exercise multi-body atomic reduction"
    );
    let frozen_nonexact = f64_recompute > 0.0;
    let archived = output["bits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let converted = expected
        .iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>();
    assert_eq!(converted, archived, "archived bits differ from values");
    let variants = output["run_variants"]
        .as_array()
        .expect("run variants")
        .iter()
        .map(|variant| {
            variant["bits"]
                .as_array()
                .unwrap()
                .iter()
                .map(|bits| {
                    f32::from_bits(u32::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<_>>();
    assert!(!variants.is_empty());
    assert_eq!(
        output["deterministic_runs"].as_bool().unwrap(),
        variants.len() == 1
    );

    let (mut compared, mut bit_differences, mut max_ulp) = (0usize, 0usize, 0u32);
    let (mut max_absolute_first, mut max_absolute_any) = (0.0f32, 0.0f32);
    let mut first_difference: Option<Value> = None;
    for world in 0..worlds {
        let actual = view.world(world).unwrap();
        for (dof, &actual_value) in actual.iter().enumerate() {
            let index = world * nv + dof;
            let expected_value = expected[index];
            let absolute = (actual_value - expected_value).abs();
            let any_absolute = variants
                .iter()
                .map(|row| (row[index] - actual_value).abs())
                .fold(f32::INFINITY, f32::min);
            max_absolute_first = max_absolute_first.max(absolute);
            max_absolute_any = max_absolute_any.max(any_absolute);
            assert!(
                any_absolute <= TOLERANCE,
                "world={world} dof={dof} actual={actual_value} expected={expected_value} \
                 absolute={absolute} any_variant_absolute={any_absolute}"
            );
            if actual_value.to_bits() != expected_value.to_bits() {
                bit_differences += 1;
                let ulp = actual_value.to_bits().abs_diff(expected_value.to_bits());
                max_ulp = max_ulp.max(ulp);
                first_difference.get_or_insert(serde_json::json!({
                    "world": world,
                    "dof": dof,
                    "actual": actual_value,
                    "expected_first_variant": expected_value,
                    "actual_bits": format!("{:08x}", actual_value.to_bits()),
                    "expected_bits": format!("{:08x}", expected_value.to_bits()),
                    "ulp": ulp,
                }));
            }
            compared += 1;
        }
    }
    let report = serde_json::json!({
        "worlds": worlds,
        "nv": nv,
        "compared_slots": compared,
        "gravity_tolerance": TOLERANCE,
        "bit_differences_vs_first_variant": bit_differences,
        "max_ulp": max_ulp,
        "max_absolute_vs_first_variant": max_absolute_first,
        "max_absolute_vs_any_variant": max_absolute_any,
        "upstream_run_variants": variants.len(),
        "upstream_order_sensitive_runs": variants.len() > 1,
        "independent_f64_recompute_max_abs": f64_recompute,
        "frozen_scenario_nonexact": frozen_nonexact,
        "multi_body_contributors_per_dof_max": max_contributors,
        "first_difference": first_difference,
        "fixture": FIXTURE,
        "fixture_entry": ENTRY,
        "reference_source": "frozen upstream passive _gravity_force device readback (offline authoring)",
        "frozen_upstream_device_output": true,
        "nonexact_reduction_order_verified": frozen_nonexact && max_contributors > 1,
        "order_bit_differences_recorded": bit_differences,
        "candidate_snapshot_used_by_reference": false,
        "product_kernel_or_tolerance_changed": false,
        "python_started_by_test": false,
        "elapsed_ms": timer.elapsed().as_millis(),
        "performance_acceptance": false,
    });
    if let Some(path) = std::env::var_os("MJWARP_U038_ATOMIC_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
