//! G03新回代目的；静态独立LU参考不启动任何作者。
#![cfg(feature = "cuda-probe")]
use mjwarp_rs as engine;
#[path = "support/sparse_mass_model.rs"]
mod input;
use engine::{
    physics::{SparseMassSolveOutput, SparseMassSolvePlan},
    runtime::TransferSession,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Instant;

fn reference() -> Value {
    serde_json::from_str(include_str!("../fixtures/sparse-mass-solve/reference.json")).unwrap()
}
fn hash(path: &str) -> String {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    format!("{:x}", Sha256::digest(bytes))
}
fn rhs(case: &Value, n: usize, nr: usize, phase: usize) -> Vec<f32> {
    let values = input::floats(case, "rhs");
    (0..nr)
        .flat_map(|r| values[((r + phase) % 3) * n..((r + phase) % 3 + 1) * n].to_vec())
        .collect()
}
fn bits(out: &SparseMassSolveOutput) -> Vec<u32> {
    (0..out.worlds())
        .flat_map(|w| {
            let v = out.world(w).unwrap().factor;
            v.sparse_factors
                .iter()
                .chain(v.diagonal_inverse)
                .map(|x| x.to_bits())
                .collect::<Vec<_>>()
        })
        .collect()
}
fn compare(
    out: &SparseMassSolveOutput,
    world: usize,
    model: &Value,
    matrix_case: &Value,
    solve_case: &Value,
    phase: usize,
) -> (usize, f64, f64) {
    let n = out.input().nv();
    let nnz = input::ints(model, "m_rownnz");
    let adr = input::ints(model, "m_rowadr");
    let col = input::ints(model, "m_colind");
    let m = input::floats(matrix_case, "matrix");
    let mut dense = vec![0.0f64; n * n];
    for i in 0..n {
        for a in adr[i] as usize..(adr[i] + nnz[i]) as usize {
            dense[i * n + col[a] as usize] = f64::from(m[a]);
            dense[col[a] as usize * n + i] = f64::from(m[a]);
        }
    }
    let norm = dense
        .chunks(n)
        .map(|v| v.iter().map(|x| x.abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let b = rhs(solve_case, n, out.rhs_count(), phase);
    let x = out.world(world).unwrap().solution;
    let mut max = 0.0f64;
    let mut max_residual = 0.0f64;
    for r in 0..out.rhs_count() {
        let mut xn = 0.0f64;
        let mut bn = 0.0f64;
        let mut residual = 0.0f64;
        for i in 0..n {
            let a = f64::from(x[r * n + i]);
            let expected = solve_case["solution"][((r + phase) % 3) * n + i]
                .as_f64()
                .unwrap();
            let error = (a - expected).abs();
            assert!(
                error <= 2e-4 + 2e-4 * expected.abs(),
                "world={world} rhs={r} dof={i} actual={a} expected={expected}"
            );
            max = max.max(error);
            xn = xn.max(a.abs());
            bn = bn.max(f64::from(b[r * n + i]).abs());
            let ax = (0..n)
                .map(|j| dense[i * n + j] * f64::from(x[r * n + j]))
                .sum::<f64>();
            residual = residual.max((ax - f64::from(b[r * n + i])).abs());
        }
        let relative = residual / (norm * xn + bn).max(1e-30);
        assert!(
            relative <= 2e-5,
            "world={world} rhs={r} residual={relative}"
        );
        max_residual = max_residual.max(relative);
    }
    (n * out.rhs_count(), max, max_residual)
}
#[test]
fn pins_independent_lu_reference_and_frozen_tolerances() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/sparse-mass-solve/manifest.json")).unwrap();
    let contract: Value =
        serde_json::from_str(include_str!("../fixtures/sparse-mass-solve/contract.json")).unwrap();
    assert_eq!(
        hash("fixtures/sparse-mass-solve/reference.json"),
        manifest["output_sha256"]
    );
    assert_eq!(
        hash("fixtures/sparse-mass-solve/contract.json"),
        manifest["contract_sha256"]
    );
    // 可移植测试只核对冻结输入；作者脚本另核对自身源码。
    let pin = contract["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "fixtures/sparse-mass-factor/native.json")
        .unwrap();
    assert_eq!(
        hash("fixtures/sparse-mass-factor/native.json"),
        pin["sha256"]
    );
    assert_eq!(contract["absolute"], 2e-4);
    assert_eq!(contract["relative"], 2e-4);
    assert_eq!(contract["product_residual"], 2e-5);
    let native = input::reference();
    let refs = reference();
    for (m, s) in native["models"]
        .as_array()
        .unwrap()
        .iter()
        .zip(refs["models"].as_array().unwrap())
    {
        assert_eq!(m["name"], s["name"]);
        assert_eq!(m["nv"], s["nv"]);
        let n = m["nv"].as_u64().unwrap() as usize;
        for case in s["cases"].as_array().unwrap() {
            assert_eq!(case["rhs"].as_array().unwrap().len(), 3 * n);
            assert_eq!(case["solution"].as_array().unwrap().len(), 3 * n);
            assert!(case["reference_residual"].as_f64().unwrap() <= 1e-12);
            assert!(
                case["rhs"].as_array().unwrap()[..n]
                    .iter()
                    .all(|v| v.as_f64() == Some(0.0))
            );
        }
    }
}
#[test]
#[ignore = "requires NVIDIA driver/NVRTC; new coupled fused solve purpose"]
fn coupled_128_thread_multi_rhs_streams_match_independent_lu_and_residual() {
    let native = input::reference();
    let refs = reference();
    let session = TransferSession::new(0).unwrap();
    let mut total = 0;
    let mut max = 0.0f64;
    let mut residual = 0.0f64;
    let mut batches = Vec::new();
    let mut max_extra = 0;
    for mi in 0..3 {
        let m = &native["models"][mi];
        let s = &refs["models"][mi];
        let n = m["nv"].as_u64().unwrap() as usize;
        let plan = SparseMassSolvePlan::new(&session, input::model(m), input::sparse(m)).unwrap();
        let mut configs = vec![(1, 3), (2, 3), (257, 3)];
        if mi == 0 {
            configs.extend([(2, 1), (2, 2), (2, 5)]);
        }
        for (worlds, nr) in configs {
            let mut data = plan.create_data(worlds, nr).unwrap();
            let start = Instant::now();
            let mut count = 0;
            for sample in 0..2 {
                data.write_qpos(&input::floats(&m["cases"][sample], "qpos").repeat(worlds))
                    .unwrap();
                data.write_rhs(&rhs(&s["cases"][sample], n, nr, 0).repeat(worlds))
                    .unwrap();
                plan.update(&mut data).unwrap();
                let before = data.readback().unwrap();
                let factor_bits = bits(&before);
                for phase in 0..2 {
                    let out = if phase == 0 {
                        before.clone()
                    } else {
                        data.write_rhs(&rhs(&s["cases"][sample], n, nr, phase).repeat(worlds))
                            .unwrap();
                        assert!(data.readback().is_err());
                        plan.update_solve(&mut data).unwrap();
                        data.readback().unwrap()
                    };
                    assert_eq!(bits(&out), factor_bits);
                    for world in 0..worlds {
                        let (c, a, r) = compare(
                            &out,
                            world,
                            m,
                            &m["cases"][sample],
                            &s["cases"][sample],
                            phase,
                        );
                        count += c;
                        max = max.max(a);
                        residual = residual.max(r);
                    }
                }
            }
            total += count;
            let extra = 4
                * (2 * (worlds * nr * n + 8)
                    + 1
                    + 2 * n
                    + plan.input().qld_level_offsets().len()
                    + 3 * plan.input().qld_all_updates().len());
            max_extra = max_extra.max(extra);
            batches.push(json!({"model":m["name"],"worlds":worlds,"rhs":nr,"states":2,"streams":2,"scalars":count,"phase_elapsed_ms":start.elapsed().as_millis(),"additional_device_bytes":extra}));
        }
    }
    let result = json!({"schema":1,"purpose":"new fused sparse solve; independent LU of frozen native f32 CSR","batches":batches,"solution_scalars":total,"max_absolute":max,"max_normalized_residual":residual,"max_additional_device_bytes":max_extra,"timing_includes_host_compare":true,"upstream_bit_difference":"unmeasured; old 395007 not inherited"});
    if let Some(dir) = std::env::var_os("MJWARP_SPARSE_SOLVE_RESULTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            std::path::Path::new(&dir).join("results.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    }
    println!("{result}");
}
#[test]
#[ignore = "requires NVIDIA driver/NVRTC; new sparse solve ownership/stale-state purpose"]
fn rejected_inputs_preserve_cache_and_world_writes_remain_independent() {
    let refs = input::reference();
    let m = &refs["models"][0];
    let n = 65;
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassSolvePlan::new(&session, input::model(m), input::sparse(m)).unwrap();
    let foreign = SparseMassSolvePlan::new(&session, input::model(m), input::sparse(m)).unwrap();
    let mut data = plan.create_data(257, 3).unwrap();
    data.write_qpos(&input::floats(&m["cases"][0], "qpos").repeat(257))
        .unwrap();
    let solve_refs = reference();
    let solve_case = &solve_refs["models"][0]["cases"][0];
    data.write_rhs(&rhs(solve_case, n, 3, 0).repeat(257))
        .unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    let factor_bits = bits(&before);
    assert!(foreign.update_solve(&mut data).is_err());
    assert!(data.write_rhs(&[]).is_err());
    assert!(data.write_world_rhs(257, &vec![1.0; 3 * n]).is_err());
    assert!(data.write_world_rhs(0, &vec![f32::NAN; 3 * n]).is_err());
    assert!(data.write_world_qpos(0, &vec![f32::NAN; n]).is_err());
    assert!(data.write_world_qpos(257, &vec![0.0; n]).is_err());
    assert_eq!(bits(&data.readback().unwrap()), factor_bits);
    data.write_world_rhs(256, &vec![0.0; 3 * n]).unwrap();
    plan.update_solve(&mut data).unwrap();
    let out = data.readback().unwrap();
    assert_eq!(bits(&out), factor_bits);
    for w in [0, 255] {
        // 原子累加不保证重算位型；数值仍须满足原容差。
        for (&a, &b) in out
            .world(w)
            .unwrap()
            .solution
            .iter()
            .zip(before.world(w).unwrap().solution)
        {
            assert!((a - b).abs() <= 2e-4 + 2e-4 * b.abs());
        }
        compare(&out, w, m, &m["cases"][0], solve_case, 0);
    }
    assert!(out.world(256).unwrap().solution.iter().all(|&v| v == 0.0));
    data.write_world_qpos(256, &input::floats(&m["cases"][1], "qpos"))
        .unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_solve(&mut data).is_err());
    plan.update_position(&mut data).unwrap();
    plan.update_matrix(&mut data).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    plan.update_gather(&mut data).unwrap();
    plan.update_factor(&mut data).unwrap();
    plan.update_solve(&mut data).unwrap();
    let snapshot = data.readback().unwrap();
    drop(data);
    drop(plan);
    drop(foreign);
    drop(session);
    assert_eq!(snapshot.worlds(), 257);
    assert!(snapshot.world(257).is_err());
    assert_eq!(snapshot.world(256).unwrap().solution.len(), 195);
    let mut empty = m.clone();
    for key in [
        "dof_parentid",
        "m_rownnz",
        "m_rowadr",
        "m_colind",
        "tree_dofadr",
        "tree_dofnum",
    ] {
        empty[key] = json!([]);
    }
    let session = TransferSession::new(0).unwrap();
    let plan =
        SparseMassSolvePlan::new(&session, input::model(&empty), input::sparse(&empty)).unwrap();
    assert!(plan.create_data(0, 3).is_err());
    assert!(plan.create_data(1, 0).is_err());
    let mut data = plan.create_data(2, 3).unwrap();
    data.write_rhs(&[]).unwrap();
    plan.update(&mut data).unwrap();
    assert!(
        data.readback()
            .unwrap()
            .world(1)
            .unwrap()
            .solution
            .is_empty()
    );
}
