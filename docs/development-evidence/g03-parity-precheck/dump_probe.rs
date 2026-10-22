//! G03 质量求解准入缺口离线诊断（不参与产品构建）。
//!
//! 用法：把本文件复制到临时 cargo 包（依赖 path 指向仓库根、features=["cuda-probe"]），
//! 以 `cargo +1.99.0-x86_64-pc-windows-msvc run --offline` 运行。
//! 只读取冻结 fixtures 与产品 GPU 探针输出，不读取测试内部状态。
use mjwarp_rs::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
use mjwarp_rs::physics::probe_mass_solve;
use mjwarp_rs::runtime::TransferSession;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}

fn f64s(v: &Value, key: &str) -> Vec<f64> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect()
}

fn integers(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}

fn model(v: &Value) -> InertialModelInput {
    let m = &v["model"];
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats(m, "qpos0"),
            body_parentid: integers(m, "body_parentid"),
            body_jntadr: integers(m, "body_jntadr"),
            body_jntnum: integers(m, "body_jntnum"),
            body_pos: floats(m, "body_pos"),
            body_quat: floats(m, "body_quat"),
            jnt_type: integers(m, "jnt_type"),
            jnt_bodyid: integers(m, "jnt_bodyid"),
            jnt_qposadr: integers(m, "jnt_qposadr"),
            jnt_dofadr: integers(m, "jnt_dofadr"),
            jnt_pos: floats(m, "jnt_pos"),
            jnt_axis: floats(m, "jnt_axis"),
        },
    )
    .unwrap();
    InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: floats(m, "body_ipos"),
            body_iquat: floats(m, "body_iquat"),
            body_mass: floats(m, "body_mass"),
            body_inertia: floats(m, "body_inertia"),
            dof_bodyid: integers(m, "dof_bodyid"),
            dof_jntid: integers(m, "dof_jntid"),
            dof_parentid: integers(m, "dof_parentid"),
            dof_armature: floats(m, "dof_armature"),
            dof_damping: floats(m, "dof_damping"),
        },
    )
    .unwrap()
}

fn bits(v: f32) -> u32 {
    v.to_bits()
}

fn abs_diff_bits(a: f32, b: f32) -> (f64, u64) {
    let abs = (f64::from(a) - f64::from(b)).abs();
    let ia = i64::from(bits(a) as i32);
    let ib = i64::from(bits(b) as i32);
    (abs, ia.abs_diff(ib))
}

/// 与 tests/support/two_reference.rs 相同的归一化误差统计。
fn score(actual: &[f32], native: &[f64], abs_tol: f64, rel_tol: f64) -> Value {
    let mut max_abs = 0.0_f64;
    let mut max_norm = 0.0_f64;
    let mut worst_abs = None;
    let mut worst_norm = None;
    let mut violations = 0usize;
    for (i, (&a, &e)) in actual.iter().zip(native).enumerate() {
        let error = (f64::from(a) - e).abs();
        if error > max_abs {
            max_abs = error;
            worst_abs = Some(i);
        }
        let limit = abs_tol + rel_tol * e.abs();
        if error > limit {
            violations += 1;
        }
        let normalized = if limit == 0.0 && error == 0.0 { 0.0 } else { error / limit };
        if normalized > max_norm || worst_norm.is_none() {
            max_norm = normalized;
            worst_norm = Some(i);
        }
    }
    json!({
        "max_absolute": max_abs,
        "max_normalized": max_norm,
        "worst_absolute_index": worst_abs,
        "worst_normalized_index": worst_norm,
        "violations": violations,
        "count": actual.len()
    })
}

fn compare_arrays(a: &[f32], b: &[f32]) -> Value {
    let mut max_abs = 0.0_f64;
    let mut max_ulp = 0u64;
    let mut different_bits = 0usize;
    for (&x, &y) in a.iter().zip(b) {
        let (abs, ulp) = abs_diff_bits(x, y);
        if abs > max_abs {
            max_abs = abs;
        }
        if ulp > max_ulp {
            max_ulp = ulp;
        }
        if bits(x) != bits(y) {
            different_bits += 1;
        }
    }
    json!({"max_absolute": max_abs, "max_ulp": max_ulp, "different_bits": different_bits, "count": a.clone().len()})
}

fn parse_trace(rows: &Value, world: usize, key: &str) -> Vec<f32> {
    rows[world][key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: dump_probe <repo-root> <output.json>"),
    );
    let out_path = PathBuf::from(std::env::args().nth(2).expect("output path"));
    let native_text = fs::read_to_string(root.join("fixtures/mass-solve/mixed-joints.json")).unwrap();
    let native_fixture: Value = serde_json::from_str(&native_text).unwrap();
    let matrix_text = fs::read_to_string(root.join("fixtures/mass-matrix/mixed-joints.json")).unwrap();
    let matrix_fixture: Value = serde_json::from_str(&matrix_text).unwrap();
    let reference_text =
        fs::read_to_string(root.join("fixtures/mjwarp-mass-solve/reference.json")).unwrap();
    let reference: Value = serde_json::from_str(&reference_text).unwrap();
    let trace_text =
        fs::read_to_string(root.join("fixtures/mjwarp-mass-solve/first-difference/ptx-aligned.json"))
            .unwrap();
    let trace: Value = serde_json::from_str(&trace_text).unwrap();

    let m = model(&matrix_fixture);
    let cases = native_fixture["cases"].as_array().unwrap();
    let worlds = 8usize;
    let rhs_count = 3usize;
    let q: Vec<f32> = (0..worlds)
        .flat_map(|w| floats(&cases[w % 8], "qpos"))
        .collect();
    let b: Vec<f32> = (0..worlds)
        .flat_map(|w| floats(&cases[w % 8], "rhs"))
        .collect();
    let session = TransferSession::new(0).unwrap();
    let out = probe_mass_solve(&session, &m, worlds, q.as_slice(), rhs_count, &b).unwrap();
    let n = out.nv();
    let world = 4usize;
    let w = out.world(world).unwrap();

    let case = &cases[4];
    let native_solution: Vec<f64> = f64s(case, "solution");
    let row = reference["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "mixed-joints")
        .unwrap()["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["worlds"] == 8 && r["rhs_count"] == 3 && r["world"] == world)
        .unwrap();
    let upstream_solution: Vec<f32> = row["solution"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect();
    let trace_rows = trace["rows"].as_array().unwrap();
    let trace_solution = parse_trace(&Value::Array(trace_rows.clone()), world, "solution");
    let trace_factor = parse_trace(&Value::Array(trace_rows.clone()), world, "factor");

    let third = native_solution.len() / 3;
    let mut our_scores = Vec::new();
    let mut upstream_scores = Vec::new();
    let mut element_rows = Vec::new();
    for r in 0..rhs_count {
        let range = r * n..(r + 1) * n;
        let our = score(&w.solution[range.clone()], &native_solution[range.clone()], 2e-4, 2e-4);
        let up = score(
            &upstream_solution[range.clone()],
            &native_solution[range.clone()],
            2e-4,
            2e-4,
        );
        our_scores.push(our);
        upstream_scores.push(up);
        for i in 0..n {
            let native = native_solution[r * n + i];
            let limit = 2e-4 + 2e-4 * native.abs();
            element_rows.push(json!({
                "rhs": r,
                "index": i,
                "native": native,
                "our": w.solution[r * n + i],
                "upstream": upstream_solution[r * n + i],
                "our_error": (f64::from(w.solution[r * n + i]) - native).abs(),
                "upstream_error": (f64::from(upstream_solution[r * n + i]) - native).abs(),
                "limit": limit,
                "our_normalized": (f64::from(w.solution[r * n + i]) - native).abs() / limit,
                "upstream_normalized": (f64::from(upstream_solution[r * n + i]) - native).abs() / limit,
            }));
        }
    }
    let _ = third;

    let report = json!({
        "task": "2026-10-22-g03-np03-admission-debt precheck",
        "model": "mixed-joints",
        "worlds": worlds,
        "rhs_count": rhs_count,
        "world": world,
        "nv": n,
        "our_solution": w.solution,
        "our_factor": w.ld,
        "our_diagonal_inverse": w.diagonal_inverse,
        "upstream_solution": upstream_solution,
        "trace_1012_solution": trace_solution,
        "trace_1012_factor": trace_factor,
        "our_scores_vs_native": our_scores,
        "upstream_scores_vs_native": upstream_scores,
        "our_vs_upstream_solution": compare_arrays(w.solution, &upstream_solution),
        "our_vs_trace_solution": compare_arrays(w.solution, &trace_solution),
        "our_vs_trace_factor": compare_arrays(w.ld, &trace_factor),
        "elements": element_rows,
    });
    fs::write(out_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("DUMP_OK");
}
