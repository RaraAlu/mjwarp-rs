//! 新常驻CRB的阶段复用与状态契约。
//! 不调用旧一次性质量矩阵探针。
#![cfg(feature = "cuda-probe")]
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedFields, AttachedModelInput},
    physics::{MassMatrixOutput, MassMatrixPlan},
    runtime::TransferSession,
};
use serde_json::Value;
use std::time::Instant;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const NAMES: [&str; 6] = [
    "mixed-joints",
    "inertial-tree",
    "rotated-tree",
    "zero-dof",
    "massless-tree",
    "armature-chain",
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;
fn fixture(name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("fixtures/mass-matrix/{name}.json")),
        )
        .unwrap(),
    )
    .unwrap()
}
fn model(v: &Value) -> AttachedModelInput {
    AttachedModelInput::new(reference::model(v), AttachedFields::default()).unwrap()
}
fn compare(out: &MassMatrixOutput, world: usize, case: &Value) -> usize {
    let values = out.world(world).unwrap();
    let mut count = 0;
    for (name, actual) in [("crb", values.crb), ("matrix", values.matrix)] {
        let native = case[name].as_array().unwrap();
        assert_eq!(actual.len(), native.len());
        for (i, (&a, e)) in actual.iter().zip(native).enumerate() {
            let e = e.as_f64().unwrap();
            assert!(
                a.is_finite() && (f64::from(a) - e).abs() <= ABS + REL * e.abs(),
                "resident CRB case={} world={world} field={name} index={i} actual={a} native={e}",
                case["id"]
            );
        }
        count += actual.len();
    }
    let nv = out.nv();
    for i in 0..nv {
        assert!(values.matrix[i * nv + i] >= 0.0);
        for j in 0..nv {
            assert_eq!(values.matrix[i * nv + j], values.matrix[j * nv + i]);
        }
    }
    count
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn reuses_resident_position_and_matrix_stages_across_all_frozen_models() {
    let session = TransferSession::new(0).unwrap();
    let mut count = 0;
    let mut batches = Vec::new();
    for name in NAMES {
        let v = fixture(name);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        let plan = MassMatrixPlan::new(&session, model(&v)).unwrap();
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for worlds in [1, 255, 256, 257, 513] {
            let mut data = plan.create_data(worlds).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_matrix(&mut data).is_err());
            let start = Instant::now();
            for round in 0..8 {
                let qpos: Vec<_> = (0..worlds)
                    .flat_map(|w| floats(&cases[(w + round) % 8], "qpos"))
                    .collect();
                data.write_qpos(&qpos).unwrap();
                assert!(data.readback().is_err());
                assert!(plan.update_matrix(&mut data).is_err());
                plan.update_position(&mut data).unwrap();
                assert!(data.readback().is_err());
                for _ in 0..2 {
                    plan.update_matrix(&mut data).unwrap();
                    let snapshot = data.readback().unwrap();
                    for w in 0..worlds {
                        count += compare(&snapshot, w, &cases[(w + round) % 8]);
                    }
                }
            }
            let nb = v["nbody"].as_u64().unwrap() as usize;
            let nv = v["nv"].as_u64().unwrap() as usize;
            let bytes = 4 * (2 + 2 * nb + 2 * nv + nv.max(1) + worlds * (10 * nb + nv * nv) + 8);
            batches.push(serde_json::json!({"model":name,"worlds":worlds,"states":8,"matrix_updates_per_state":2,
                "elapsed_ms":start.elapsed().as_millis(),"new_crb_device_bytes":bytes,
                "memory_scope":"new metadata/armature/output only; excludes reused G01 buffers, JIT and driver"}));
        }
    }
    let report = serde_json::json!({"purpose":"new resident stage reuse, not one-shot probe replay","scalars":count,"batches":batches,
        "absolute_tolerance":ABS,"relative_tolerance":REL,"fair_performance_acceptance":false,"process_gpu_memory_peak_measured":false});
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/resident-crb/batches.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!(
        "resident CRB six models, eight states, five world sizes, two matrix updates/state: {count} scalars"
    );
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn checks_resident_qpos_invalidation_foreign_plans_and_snapshot_ownership() {
    let session = TransferSession::new(0).unwrap();
    let v = fixture(NAMES[0]);
    let plan = MassMatrixPlan::new(&session, model(&v)).unwrap();
    let foreign = MassMatrixPlan::new(&session, model(&v)).unwrap();
    for worlds in [0, usize::MAX, u32::MAX as usize] {
        assert!(plan.create_data(worlds).is_err());
    }
    let mut data = plan.create_data(513).unwrap();
    let cases = v["cases"].as_array().unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    for result in [
        foreign.update(&mut data),
        foreign.update_position(&mut data),
        foreign.update_matrix(&mut data),
    ] {
        assert!(matches!(result, Err(TransferError::ModelMismatch)));
    }
    assert!(data.write_qpos(&[]).is_err());
    assert!(
        data.write_world_qpos(513, &floats(&cases[0], "qpos"))
            .is_err()
    );
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut values = floats(&cases[0], "qpos");
        values[0] = x;
        assert!(data.write_world_qpos(256, &values).is_err());
    }
    let preserved = data.readback().unwrap();
    for w in 0..513 {
        assert_eq!(
            preserved.world(w).unwrap().matrix,
            before.world(w).unwrap().matrix
        );
    }
    data.write_world_qpos(256, &floats(&cases[7], "qpos"))
        .unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_matrix(&mut data).is_err());
    plan.update_position(&mut data).unwrap();
    plan.update_matrix(&mut data).unwrap();
    let after = data.readback().unwrap();
    for w in 0..513 {
        compare(&after, w, &cases[if w == 256 { 7 } else { 0 }]);
    }
    compare(&before, 256, &cases[0]);
    assert!(after.world(513).is_err());
    drop((data, plan, foreign, session));
    compare(&after, 256, &cases[7]);
}
