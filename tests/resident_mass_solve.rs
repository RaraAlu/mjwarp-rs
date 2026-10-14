//! 新常驻因子与RHS流；不调用旧一次性探针。
#![cfg(feature = "cuda-probe")]
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedFields, AttachedModelInput},
    physics::{MassFactorKind, MassSolveOutput, MassSolvePlan},
    runtime::TransferSession,
};
use serde_json::{Value, json};
#[path = "support/mass_solve_reference.rs"]
mod frozen;
#[path = "support/two_reference.rs"]
mod policy;
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
const ABS: f64 = 2e-4;
const REL: f64 = 2e-4;
const RESIDUAL: f64 = 2e-5;
const RECONSTRUCTION: f64 = 2e-6;
fn text(directory: &str, name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("fixtures/{directory}/{name}.json")),
    )
    .unwrap()
}
fn model(name: &str) -> AttachedModelInput {
    AttachedModelInput::new(
        reference::model(&serde_json::from_str(&text("mass-matrix", name)).unwrap()),
        AttachedFields::default(),
    )
    .unwrap()
}
fn rhs(case: &Value, n: usize, count: usize, phase: usize) -> Vec<f32> {
    let base = floats(case, "rhs");
    (0..count)
        .flat_map(|r| {
            base[((r + phase) % 3) * n..((r + phase) % 3 + 1) * n]
                .iter()
                .copied()
        })
        .collect()
}
fn properties(out: &MassSolveOutput, w: usize, case: &Value, phase: usize) {
    let n = out.nv();
    let view = out.world(w).unwrap();
    let m = case["matrix"].as_array().unwrap();
    let norm = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| m[i * n + j].as_f64().unwrap().abs())
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    let mut reconstruction = 0.0f64;
    for i in 0..n {
        let block = view
            .blocks
            .iter()
            .find(|b| (b.start..b.start + b.size).contains(&i))
            .unwrap();
        let mut error = 0.0;
        for j in 0..n {
            let value = if !(block.start..block.start + block.size).contains(&j) {
                0.0
            } else {
                match block.kind {
                    MassFactorKind::Diagonal => {
                        if i == j {
                            1.0 / f64::from(view.diagonal_inverse[i])
                        } else {
                            0.0
                        }
                    }
                    MassFactorKind::Cholesky => (block.start..=i.min(j))
                        .map(|k| f64::from(view.ld[k * n + i]) * f64::from(view.ld[k * n + j]))
                        .sum(),
                    MassFactorKind::ReverseLdl => (i.max(j)..block.start + block.size)
                        .map(|k| {
                            let a = if k == i {
                                1.0
                            } else {
                                f64::from(view.ld[k * n + i])
                            };
                            let b = if k == j {
                                1.0
                            } else {
                                f64::from(view.ld[k * n + j])
                            };
                            a * f64::from(view.ld[k * n + k]) * b
                        })
                        .sum(),
                }
            };
            error += (value - m[i * n + j].as_f64().unwrap()).abs();
        }
        reconstruction = reconstruction.max(if norm == 0.0 { error } else { error / norm });
    }
    assert!(
        reconstruction <= RECONSTRUCTION,
        "reconstruction={reconstruction}"
    );
    for r in 0..out.rhs_count() {
        let b = &case["rhs"].as_array().unwrap()[((r + phase) % 3) * n..((r + phase) % 3 + 1) * n];
        let x = &view.solution[r * n..(r + 1) * n];
        let denominator = norm * x.iter().map(|v| f64::from(*v).abs()).fold(0.0, f64::max)
            + b.iter()
                .map(|v| v.as_f64().unwrap().abs())
                .fold(0.0, f64::max);
        for i in 0..n {
            let error = ((0..n)
                .map(|j| m[i * n + j].as_f64().unwrap() * f64::from(x[j]))
                .sum::<f64>()
                - b[i].as_f64().unwrap())
            .abs();
            assert!(
                (if denominator == 0.0 {
                    error
                } else {
                    error / denominator
                }) <= RESIDUAL,
                "residual world={w} rhs={r}"
            );
        }
    }
}
fn compare(
    out: &MassSolveOutput,
    w: usize,
    case: &Value,
    source: &str,
    phase: usize,
    audit: &mut policy::Audit,
) {
    let (model, row) = frozen::row(source, out.worlds(), 3, w, case);
    let view = out.world(w).unwrap();
    let n = out.nv();
    assert_eq!(view.blocks.len(), model["blocks"].as_array().unwrap().len());
    for (b, r) in view.blocks.iter().zip(model["blocks"].as_array().unwrap()) {
        assert_eq!(r["start"], b.start);
        assert_eq!(r["size"], b.size);
        let kind = match r["route"].as_str().unwrap() {
            "compact" => MassFactorKind::Diagonal,
            "small_cholesky" | "tile_cholesky" => MassFactorKind::Cholesky,
            "sparse_ldl" => MassFactorKind::ReverseLdl,
            _ => panic!("route"),
        };
        assert_eq!(b.kind, kind);
    }
    for (field, actual, native) in [
        ("factor", view.ld, "native_factor"),
        (
            "diagonal_inverse",
            view.diagonal_inverse,
            "native_diagonal_inverse",
        ),
    ] {
        let expected = row[native]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let decision = policy::evaluate(
            actual,
            Some(policy::native_reference(
                format!("native-derived:{field}"),
                expected,
                ABS,
                REL,
            )),
            || {
                Ok(Some(policy::SecondaryReference {
                    identity: format!("frozen:{}:{field}", frozen::reference()["revision"]),
                    conditions: format!("same model/qpos/world={}, independent RHS factor; f32", w),
                    values: row[field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap() as f32)
                        .collect(),
                    relative_slack: Some(policy::MJWARP_RELATIVE_SLACK),
                }))
            },
        );
        audit.record(
            json!({"world":w,"case":case["id"],"phase":phase,"field":field}),
            decision,
        );
    }
    for r in 0..out.rhs_count() {
        let column = (r + phase) % 3;
        let range = column * n..(column + 1) * n;
        let expected = case["solution"].as_array().unwrap()[range.clone()]
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let decision = policy::evaluate(
            &view.solution[r * n..(r + 1) * n],
            Some(policy::native_reference(
                format!("native-solution:column{column}"),
                expected,
                ABS,
                REL,
            )),
            || {
                Ok(Some(policy::SecondaryReference {
                    identity: format!(
                        "frozen:{}:independent-column{column}",
                        frozen::reference()["revision"]
                    ),
                    conditions: format!(
                        "same model/qpos/world={w}; explicit RHS column projection from frozen count3 into count{}; phase={phase}; f32",
                        out.rhs_count()
                    ),
                    values: row["solution"].as_array().unwrap()[range]
                        .iter()
                        .map(|v| v.as_f64().unwrap() as f32)
                        .collect(),
                    relative_slack: Some(policy::MJWARP_RELATIVE_SLACK),
                }))
            },
        );
        audit.record(json!({"world":w,"case":case["id"],"phase":phase,"field":"solution","rhs":r,"source_column":column}),decision);
    }
    properties(out, w, case, phase);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn streams_rhs_updates_through_cached_factors_against_independent_references() {
    let session = TransferSession::new(0).unwrap();
    let mut audit = policy::Audit::new("new resident cached factor/RHS stream");
    let mut batches = Vec::new();
    for (index, name) in NAMES.iter().enumerate() {
        let source = text("mass-solve", name);
        let v: Value = serde_json::from_str(&source).unwrap();
        let cases = v["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        assert_eq!(v["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(v["relative_tolerance"].as_f64(), Some(REL));
        let plan = MassSolvePlan::new(&session, model(name)).unwrap();
        let native_model: Value = serde_json::from_str(&text("mass-matrix", name)).unwrap();
        let block_count = native_model["model"]["dof_parentid"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p.as_i64().unwrap() < 0)
            .count();
        let mut configurations = vec![(8, 3)];
        if index == 0 {
            configurations.extend([(8, 1), (8, 2), (8, 5)]);
        }
        if index == 2 {
            configurations.extend([1, 255, 256, 257, 513].map(|w| (w, 3)));
        }
        for (worlds, count) in configurations {
            let mut data = plan.create_data(worlds, count).unwrap();
            assert!(data.readback().is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(plan.update_factor(&mut data).is_err());
            data.write_qpos(
                &(0..worlds)
                    .flat_map(|w| floats(&cases[w % 8], "qpos"))
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            let started = std::time::Instant::now();
            plan.update_position(&mut data).unwrap();
            plan.update_matrix(&mut data).unwrap();
            plan.update_factor(&mut data).unwrap();
            let mut factor_bits = None;
            for phase in 0..2 {
                data.write_rhs(
                    &(0..worlds)
                        .flat_map(|w| rhs(&cases[w % 8], data.nv(), count, phase))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
                assert!(data.readback().is_err());
                plan.update_solve(&mut data).unwrap();
                let out = data.readback().unwrap();
                let bits: Vec<_> = (0..worlds)
                    .flat_map(|w| {
                        let v = out.world(w).unwrap();
                        v.ld.iter()
                            .chain(v.diagonal_inverse)
                            .map(|x| x.to_bits())
                            .collect::<Vec<_>>()
                    })
                    .collect();
                if let Some(before) = &factor_bits {
                    assert_eq!(&bits, before);
                } else {
                    factor_bits = Some(bits);
                }
                for w in 0..worlds {
                    compare(&out, w, &cases[w % 8], &source, phase, &mut audit);
                }
            }
            let n = data.nv();
            let owned_bytes =
                4 * (1 + 3 * block_count + worlds * (6 + 2 * n * n + 3 * n + 2 * count * n) + 24);
            batches.push(json!({"model":name,"worlds":worlds,"rhs_count":count,"factor_updates":1,"rhs_stream_updates":2,
                "elapsed_ms":started.elapsed().as_millis(),"new_solve_device_bytes":owned_bytes,
                "memory_scope":"new block metadata/factor/guarded RHS/solution only; excludes reused CRB/G01, JIT and driver"}));
        }
    }
    let report =
        json!({"numeric":audit.report(),"batches":batches,"fair_performance_acceptance":false});
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/resident-mass-solve/results.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
    audit.finish();
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn checks_world_rhs_isolation_foreign_plans_and_position_invalidation() {
    let session = TransferSession::new(0).unwrap();
    let name = NAMES[2];
    let v: Value = serde_json::from_str(&text("mass-solve", name)).unwrap();
    let cases = v["cases"].as_array().unwrap();
    let plan = MassSolvePlan::new(&session, model(name)).unwrap();
    let foreign = MassSolvePlan::new(&session, model(name)).unwrap();
    for (w, r) in [(0, 3), (1, 0), (usize::MAX, 3), (1, usize::MAX)] {
        assert!(plan.create_data(w, r).is_err());
    }
    let mut data = plan.create_data(513, 3).unwrap();
    data.write_qpos(
        &(0..513)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    data.write_rhs(
        &(0..513)
            .flat_map(|w| rhs(&cases[w % 8], data.nv(), 3, 0))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    for result in [
        foreign.update_position(&mut data),
        foreign.update_matrix(&mut data),
        foreign.update_factor(&mut data),
        foreign.update_solve(&mut data),
    ] {
        assert!(matches!(result, Err(TransferError::ModelMismatch)));
    }
    assert!(data.write_rhs(&[]).is_err());
    assert!(
        data.write_world_rhs(513, &rhs(&cases[0], data.nv(), 3, 0))
            .is_err()
    );
    let mut invalid = rhs(&cases[0], data.nv(), 3, 0);
    invalid[0] = f32::NAN;
    assert!(data.write_world_rhs(256, &invalid).is_err());
    assert!(data.write_qpos(&[]).is_err());
    let preserved = data.readback().unwrap();
    assert_eq!(
        before.world(256).unwrap().solution,
        preserved.world(256).unwrap().solution
    );
    data.write_world_rhs(256, &rhs(&cases[0], data.nv(), 3, 1))
        .unwrap();
    assert!(data.readback().is_err());
    plan.update_solve(&mut data).unwrap();
    let after = data.readback().unwrap();
    for w in 0..513 {
        if w != 256 {
            assert_eq!(
                before.world(w).unwrap().solution,
                after.world(w).unwrap().solution
            );
        } else {
            let native = cases[0]["solution"].as_array().unwrap();
            let actual = after.world(w).unwrap().solution;
            for (i, &a) in actual.iter().enumerate() {
                let e = native[((i / data.nv() + 1) % 3) * data.nv() + i % data.nv()]
                    .as_f64()
                    .unwrap();
                assert!((f64::from(a) - e).abs() <= ABS + REL * e.abs());
            }
        }
    }
    data.write_world_qpos(256, &floats(&cases[1], "qpos"))
        .unwrap();
    assert!(plan.update_solve(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_matrix(&mut data).is_err());
    data.write_world_rhs(256, &rhs(&cases[1], data.nv(), 3, 0))
        .unwrap();
    plan.update(&mut data).unwrap();
    let changed = data.readback().unwrap();
    for (a, e) in changed
        .world(256)
        .unwrap()
        .solution
        .iter()
        .zip(cases[1]["solution"].as_array().unwrap())
    {
        let e = e.as_f64().unwrap();
        assert!((f64::from(*a) - e).abs() <= ABS + REL * e.abs());
    }
    drop((data, plan, foreign, session));
    assert!(changed.world(512).is_ok());
    assert!(changed.world(513).is_err());
}
