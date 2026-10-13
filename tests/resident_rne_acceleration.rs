//! G02加速度辅助的独立新行为。
//! 测试只读静态原生参考。
#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedFields, AttachedModelInput},
    physics::{RnePlan, RneSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Instant;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const NAMES: [&str; 5] = [
    "mixed-joints",
    "inertial-tree",
    "rotated-tree",
    "zero-dof",
    "massless-tree",
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn fixture(name: &str, acceleration: bool) -> Value {
    let directory = if acceleration {
        "rne-acceleration"
    } else {
        "rne-bias"
    };
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("fixtures/{directory}/{name}.json")),
        )
        .unwrap(),
    )
    .unwrap()
}
fn model(name: &str) -> AttachedModelInput {
    AttachedModelInput::new(
        reference::model(&fixture(name, false)),
        AttachedFields::default(),
    )
    .unwrap()
}
fn gravity(v: &Value, g: usize) -> [f32; 3] {
    v["gravities"][g]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
fn compare(snapshot: &RneSnapshot, w: usize, case: &Value, g: usize, flag: bool) -> usize {
    let result = snapshot.world(w).unwrap();
    assert_eq!(result.cacc.len(), 6 * snapshot.nbody());
    assert_eq!(result.cfrc_int.len(), 6 * snapshot.nbody());
    let expected = case["qfrc"][usize::from(flag)][g].as_array().unwrap();
    assert_eq!(result.qfrc_bias.len(), expected.len());
    for (d, (&actual, expected)) in result.qfrc_bias.iter().zip(expected).enumerate() {
        let expected = expected.as_f64().unwrap();
        let error = (f64::from(actual) - expected).abs();
        assert!(
            actual.is_finite() && error <= ABS + REL * expected.abs(),
            "G02-RNE-ACC flag={flag} world={w} dof={d} actual={actual} native={expected} error={error}"
        );
    }
    expected.len()
}
fn same(a: &RneSnapshot, b: &RneSnapshot) {
    for w in 0..a.worlds() {
        assert_eq!(a.world(w).unwrap().qfrc_bias, b.world(w).unwrap().qfrc_bias);
        assert_eq!(a.world(w).unwrap().cacc, b.world(w).unwrap().cacc);
        assert_eq!(a.world(w).unwrap().cfrc_int, b.world(w).unwrap().cfrc_int);
    }
}

#[test]
fn freezes_new_acceleration_inputs_and_existing_tolerances() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("fixtures/rne-acceleration/manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["native_version"], 3012000);
    assert_eq!(manifest["absolute_tolerance"].as_f64(), Some(ABS));
    assert_eq!(manifest["relative_tolerance"].as_f64(), Some(REL));
    for row in manifest["files"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(row["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            row["sha256"].as_str().unwrap()
        );
    }
    let mut joint_types = [false; 4];
    let mut changed = false;
    for name in NAMES {
        let v = fixture(name, true);
        let old = fixture(name, false);
        let m = model(name);
        assert_eq!(v["cases"].as_array().unwrap().len(), 8);
        assert_eq!(v["gravities"], old["gravities"]);
        for &kind in &m.rigid().kinematics().fields().jnt_type {
            joint_types[kind as usize] = true;
        }
        for c in v["cases"].as_array().unwrap() {
            for key in ["qpos", "qvel", "qacc"] {
                for number in c[key].as_array().unwrap() {
                    let n = number.as_f64().unwrap();
                    assert!(n.is_finite());
                    // serde_json十进制解析可能差一个f64尾位。
                    // 此界限只诊断序列化，不改变物理容差。
                    assert!((f64::from(n as f32) - n).abs() <= f64::EPSILON * n.abs());
                }
            }
            changed |= c["qfrc"][0] != c["qfrc"][1];
        }
    }
    assert!(joint_types.into_iter().all(|v| v));
    assert!(changed);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn computes_both_flags_for_nonzero_accelerations_without_old_bias_replay() {
    let session = TransferSession::new(0).unwrap();
    let mut scalars = 0;
    let mut failures = Vec::new();
    let mut batches = Vec::new();
    let start = Instant::now();
    for name in NAMES {
        let v = fixture(name, true);
        let cases = v["cases"].as_array().unwrap();
        for g in 0..3 {
            let plan = RnePlan::new(&session, model(name), gravity(&v, g)).unwrap();
            for worlds in [1, 2, 5, 513] {
                let mut data = plan.create_data(worlds).unwrap();
                let batch_start = Instant::now();
                for round in 0..8 {
                    for key in ["qpos", "qvel", "qacc"] {
                        let values: Vec<_> = (0..worlds)
                            .flat_map(|w| floats(&cases[(w + round) % 8], key))
                            .collect();
                        match key {
                            "qpos" => data.write_qpos(&values),
                            "qvel" => data.write_qvel(&values),
                            _ => data.write_qacc(&values),
                        }
                        .unwrap();
                    }
                    plan.update_velocity(&mut data).unwrap();
                    for flag in [false, true] {
                        plan.update_rne(&mut data, flag).unwrap();
                        let snapshot = data.readback().unwrap();
                        assert_eq!(snapshot.includes_acceleration(), flag);
                        for w in 0..worlds {
                            let case = &cases[(w + round) % 8];
                            let actual = snapshot.world(w).unwrap().qfrc_bias;
                            let native = case["qfrc"][usize::from(flag)][g].as_array().unwrap();
                            assert_eq!(actual.len(), native.len());
                            let mut maximum = 0.0_f64;
                            let mut worst = 0;
                            for (d, (&a, e)) in actual.iter().zip(native).enumerate() {
                                let e = e.as_f64().unwrap();
                                let normalized = (f64::from(a) - e).abs() / (ABS + REL * e.abs());
                                if normalized > maximum {
                                    maximum = normalized;
                                    worst = d;
                                }
                            }
                            scalars += actual.len();
                            if maximum > 1.0 {
                                failures.push(serde_json::json!({"model":name,"gravity":g,"worlds":worlds,"round":round,"world":w,
                                    "state":(w+round)%8,"flag":flag,"max_normalized_error":maximum,"worst_dof":worst,
                                    "actual":actual[worst],"native":native[worst],"admitted":false}));
                            }
                        }
                    }
                }
                let nb = v["nbody"].as_u64().unwrap() as usize;
                let nv = v["nv"].as_u64().unwrap() as usize;
                let owned_rne_device_bytes =
                    4 * (5 + 4 * nb + nv + worlds * (12 * nb + 2 * nv) + 16);
                batches.push(serde_json::json!({"model":name,"gravity":g,"worlds":worlds,"rounds":8,"flags":2,
                    "elapsed_ms":batch_start.elapsed().as_millis(),"owned_rne_device_bytes":owned_rne_device_bytes,
                    "memory_scope":"metadata/qacc/output actual allocation sizes only; excludes reused velocity/G01 buffers, JIT and driver"}));
            }
        }
    }
    eprintln!(
        "G02-RNE-ACC models=5 gravities=3 states=8 worlds=[1,2,5,513] flags=2 scalars={scalars} elapsed_ms={}",
        start.elapsed().as_millis()
    );
    let report = serde_json::json!({"scope":"new qacc native tolerance gate; no MJWarp/redline admission", "scalars":scalars,
        "failed_world_field_blocks":failures.len(),"failures":failures,"batches":batches,"elapsed_ms":start.elapsed().as_millis(),
        "absolute_tolerance":ABS,"relative_tolerance":REL,"process_gpu_memory_peak_measured":false,"fair_performance_acceptance":false});
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/rne-acceleration-validation/full-numerical.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "G02-RNE-ACC {} native-tolerance blocks failed; complete coverage recorded",
        failures.len()
    );
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn qacc_writes_preserve_velocity_but_invalidate_force_and_reject_bad_inputs() {
    let session = TransferSession::new(0).unwrap();
    let v = fixture(NAMES[0], true);
    let plan = RnePlan::new(&session, model(NAMES[0]), gravity(&v, 0)).unwrap();
    let foreign = RnePlan::new(&session, model(NAMES[0]), gravity(&v, 0)).unwrap();
    for worlds in [0, usize::MAX, u32::MAX as usize] {
        assert!(plan.create_data(worlds).is_err());
    }
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(RnePlan::new(&session, model(NAMES[0]), [x, 0.0, 0.0]).is_err());
    }
    let mut data = plan.create_data(513).unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_rne(&mut data, true).is_err());
    let first = &v["cases"][0];
    data.write_qacc(
        &(0..513)
            .flat_map(|_| floats(first, "qacc"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    plan.update(&mut data, true).unwrap();
    let before = data.readback().unwrap();
    for operation in [
        foreign.update(&mut data, true),
        foreign.update_velocity(&mut data),
        foreign.update_rne(&mut data, true),
    ] {
        assert!(matches!(operation, Err(TransferError::ModelMismatch)));
    }
    for result in [
        data.write_qacc(&[]),
        data.write_world_qacc(513, &floats(first, "qacc")),
        data.write_qpos(&[]),
        data.write_qvel(&[]),
    ] {
        assert!(matches!(result, Err(TransferError::Input(_))));
    }
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut values = floats(first, "qacc");
        values[0] = x;
        assert!(data.write_world_qacc(256, &values).is_err());
    }
    same(&before, &data.readback().unwrap());
    let mut qacc = floats(first, "qacc");
    qacc.fill(0.0);
    data.write_world_qacc(256, &qacc).unwrap();
    assert!(data.readback().is_err());
    plan.update_rne(&mut data, true).unwrap();
    let after = data.readback().unwrap();
    compare(&after, 256, first, 0, false);
    for w in 0..513 {
        if w != 256 {
            compare(&after, w, first, 0, true);
        }
    }
    compare(&before, 256, first, 0, true);
    assert!(before.world(513).is_err());
    data.write_world_qvel(0, &floats(first, "qvel")).unwrap();
    assert!(plan.update_rne(&mut data, true).is_err());
    plan.update_velocity(&mut data).unwrap();
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();
    data.write_world_qpos(0, &floats(first, "qpos")).unwrap();
    assert!(plan.update_rne(&mut data, true).is_err());
    plan.update(&mut data, true).unwrap();
    data.readback().unwrap();
}
