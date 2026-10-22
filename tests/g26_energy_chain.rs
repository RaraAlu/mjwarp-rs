//! G26：常驻 CRB→M→M·qvel→动能全链调度与冻结上游 energy_vel 参考。
//! 上游参考由离线 authoring 生成（fixtures/kinetic-energy/upstream-energy-vel.json）；
//! 本测试只读静态 fixture，不启动 Python。
#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedFields, AttachedModelInput, SparseMassInput},
    physics::{
        MassMatrixEnergyPlan, MassMatrixMultiplyData, MassMatrixMultiplyPlan, MassMatrixPlan,
        MassMultiplyFormat, schedule_energy_chain,
    },
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "support/rigid_reference.rs"]
mod reference;
#[path = "support/g26_energy.rs"]
mod support;

const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;
const NAMES: [&str; 6] = [
    "mixed-joints",
    "inertial-tree",
    "rotated-tree",
    "zero-dof",
    "massless-tree",
    "armature-chain",
];

fn read(path: &str) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(root.join(path)).unwrap()).unwrap()
}

fn fixture() -> Value {
    read("fixtures/kinetic-energy/upstream-energy-vel.json")
}

fn mass_matrix_fixture(name: &str) -> Value {
    read(&format!("fixtures/mass-matrix/{name}.json"))
}

fn model_for(scenario: &Value) -> (AttachedModelInput, SparseMassInput) {
    if scenario["kind"] == "fixture" {
        let name = scenario["name"].as_str().unwrap();
        let fixture = mass_matrix_fixture(name);
        (
            AttachedModelInput::new(reference::model(&fixture), AttachedFields::default()).unwrap(),
            support::sparse(&scenario["sparse"]),
        )
    } else {
        let nv = scenario["nv"].as_u64().unwrap() as usize;
        assert_eq!(scenario["name"], format!("synthetic-chain-{nv}"));
        support::synthetic(nv)
    }
}

fn potential(values: usize) -> Vec<f32> {
    (0..values)
        .map(|w| ((w * 11 + 3) % 17) as f32 * 0.0625 - 0.5)
        .collect()
}

#[test]
fn freezes_static_upstream_energy_reference_and_manifest_hashes() {
    let manifest = read("fixtures/kinetic-energy/manifest.json");
    assert_eq!(
        manifest["upstream_revision"].as_str(),
        Some("71da24d956378a87a703b6e1442b13aec0c4ac29")
    );
    assert_eq!(manifest["absolute_tolerance"].as_f64(), Some(ABS));
    assert_eq!(manifest["relative_tolerance"].as_f64(), Some(REL));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for row in manifest["files"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(row["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            row["sha256"].as_str().unwrap()
        );
    }
    let value = fixture();
    assert_eq!(value["upstream_revision"], manifest["upstream_revision"]);
    let scenarios = value["scenarios"].as_array().unwrap();
    assert_eq!(scenarios.len(), 9);
    let mut fixture_models = 0;
    let mut synthetic = 0;
    let mut records = 0;
    for scenario in scenarios {
        let nv = scenario["nv"].as_u64().unwrap() as usize;
        let cases = scenario["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        match scenario["kind"].as_str().unwrap() {
            "fixture" => {
                fixture_models += 1;
                assert!(NAMES.contains(&scenario["name"].as_str().unwrap()));
            }
            "synthetic-chain" => {
                synthetic += 1;
                assert!([3usize, 33, 65].contains(&nv));
            }
            other => panic!("unexpected scenario kind {other}"),
        }
        for case in cases {
            assert_eq!(
                support::hex_floats(&case["qpos_hex"]).len(),
                scenario["nq"].as_u64().unwrap() as usize
            );
            assert_eq!(support::hex_floats(&case["qvel_hex"]).len(), nv);
            assert_eq!(support::hex_floats(&case["energy_vel_hex"]).len(), 2);
            records += 1;
        }
        let sparse = &scenario["sparse"];
        assert_eq!(support::ints(sparse, "dof_parentid").len(), nv);
        assert_eq!(support::ints(sparse, "m_rownnz").len(), nv);
        assert_eq!(support::ints(sparse, "m_rowadr").len(), nv);
        assert_eq!(
            support::ints(sparse, "tree_dofadr").len(),
            support::ints(sparse, "tree_dofnum").len()
        );
    }
    assert_eq!(fixture_models, 6);
    assert_eq!(synthetic, 3);
    assert_eq!(records, 144);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC; frozen upstream energy reference"]
fn schedules_crb_to_kinetic_energy_against_frozen_upstream_reference() {
    let session = TransferSession::new(0).unwrap();
    let value = fixture();
    let scenarios = value["scenarios"].as_array().unwrap();
    let mut scalars = 0usize;
    let mut failures = Vec::new();
    let mut batches = Vec::new();
    let mut worst = 0.0_f64;
    let mut worst_record: Option<Value> = None;
    let start = std::time::Instant::now();
    for scenario in scenarios {
        let name = scenario["name"].as_str().unwrap();
        let nv = scenario["nv"].as_u64().unwrap() as usize;
        let (model, sparse) = model_for(scenario);
        let producer = MassMatrixPlan::new(&session, model).unwrap();
        if nv == 0 {
            // 完整调度要求 n>=1；零自由度模型在乘积/动能布局处必须被拒绝。
            assert!(matches!(
                MassMatrixMultiplyPlan::new(&producer, sparse.clone(), MassMultiplyFormat::Sparse),
                Err(TransferError::Input(_))
            ));
            assert!(matches!(
                MassMatrixMultiplyPlan::new(&producer, sparse, MassMultiplyFormat::Dense),
                Err(TransferError::Input(_))
            ));
            continue;
        }
        let cases = scenario["cases"].as_array().unwrap();
        for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
            let multiply = MassMatrixMultiplyPlan::new(&producer, sparse.clone(), format).unwrap();
            let energy = MassMatrixEnergyPlan::new(&multiply).unwrap();
            let mut source = producer.create_data(1).unwrap();
            let mut multiply_data: MassMatrixMultiplyData = multiply.create_data(1).unwrap();
            let mut energy_data = energy.create_data(1).unwrap();
            for case in cases {
                let qpos = support::hex_floats(&case["qpos_hex"]);
                let qvel = support::hex_floats(&case["qvel_hex"]);
                let expected = support::hex_floats(&case["energy_vel_hex"])[1];
                let p = potential(1);
                source.write_qpos(&qpos).unwrap();
                let before = source.source_state();
                let snapshot = schedule_energy_chain(
                    &producer,
                    &mut source,
                    &multiply,
                    &mut multiply_data,
                    &energy,
                    &mut energy_data,
                    &qvel,
                    &p,
                )
                .unwrap();
                assert_eq!(snapshot.source_state(), before);
                assert_eq!(snapshot.source_state().token(), before.token());
                assert_eq!(snapshot.worlds(), 1);
                let result = snapshot.world(0).unwrap();
                let actual = result[1];
                let expected = f64::from(expected);
                let normalized =
                    (f64::from(actual) - expected).abs() / (ABS + REL * expected.abs());
                scalars += 2;
                worst = worst.max(normalized);
                if normalized > 1.0 {
                    failures.push(serde_json::json!({
                        "scenario": name, "format": format!("{format:?}"), "state": case["state"],
                        "vector": case["vector"], "actual": actual, "expected": expected,
                        "normalized": normalized,
                    }));
                } else if normalized >= worst - 1e-12 {
                    worst_record = Some(serde_json::json!({
                        "scenario": name, "format": format!("{format:?}"), "state": case["state"],
                        "vector": case["vector"], "actual": actual, "expected": expected,
                        "normalized": normalized,
                    }));
                }
            }
        }
        // 非默认世界数：同一场景 5 个世界，逐世界用对应案例期望。
        if matches!(name, "mixed-joints" | "synthetic-chain-33") {
            let multiply =
                MassMatrixMultiplyPlan::new(&producer, sparse, MassMultiplyFormat::Dense).unwrap();
            let energy = MassMatrixEnergyPlan::new(&multiply).unwrap();
            let worlds = 5;
            let mut source = producer.create_data(worlds).unwrap();
            let mut multiply_data = multiply.create_data(worlds).unwrap();
            let mut energy_data = energy.create_data(worlds).unwrap();
            let mut qpos = Vec::new();
            let mut qvel = Vec::new();
            for w in 0..worlds {
                let case = &cases[w % cases.len()];
                qpos.extend(support::hex_floats(&case["qpos_hex"]));
                qvel.extend(support::hex_floats(&case["qvel_hex"]));
            }
            source.write_qpos(&qpos).unwrap();
            let before = source.source_state();
            let snapshot = schedule_energy_chain(
                &producer,
                &mut source,
                &multiply,
                &mut multiply_data,
                &energy,
                &mut energy_data,
                &qvel,
                &potential(worlds),
            )
            .unwrap();
            assert_eq!(snapshot.source_state(), before);
            for w in 0..worlds {
                let expected = support::hex_floats(&cases[w % cases.len()]["energy_vel_hex"])[1];
                let actual = snapshot.world(w).unwrap()[1];
                let expected = f64::from(expected);
                let normalized =
                    (f64::from(actual) - expected).abs() / (ABS + REL * expected.abs());
                scalars += 2;
                worst = worst.max(normalized);
                if normalized > 1.0 {
                    failures.push(serde_json::json!({
                        "scenario": name, "format": "Dense", "worlds": worlds, "world": w,
                        "actual": actual, "expected": expected, "normalized": normalized,
                    }));
                }
            }
        }
        batches.push(serde_json::json!({"scenario": name, "nv": nv, "formats": 2}));
    }
    let elapsed_ms = start.elapsed().as_millis();
    let report = serde_json::json!({
        "scope": "full CRB->M->Mqvel->kinetic scheduling vs frozen upstream MJWarp energy_vel; no full G02 sensor scheduling",
        "scalars": scalars,
        "failed_blocks": failures.len(),
        "failures": failures,
        "worst_normalized_error": worst,
        "worst_record": worst_record,
        "batches": batches,
        "elapsed_ms": elapsed_ms,
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "process_gpu_memory_peak_measured": false,
        "fair_performance_acceptance": false,
    });
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("target/g26-complete");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("energy-chain-report.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!(
        "G26 energy chain scenarios=9 scalars={scalars} failed={} worst_normalized={worst:.3e} elapsed_ms={elapsed_ms}",
        if failures.is_empty() { 0 } else { 1 }
    );
    assert!(
        report["failures"].as_array().unwrap().is_empty(),
        "G26 energy-chain upstream tolerance failures recorded in target/g26-complete/energy-chain-report.json"
    );
}
