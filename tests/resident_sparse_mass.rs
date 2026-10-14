//! 新设备衔接目的；不重复固定CSR分解验收。
#![cfg(feature = "cuda-probe")]
use mjwarp_rs as engine;
#[path = "support/sparse_mass_model.rs"]
mod input;
use engine::{
    diagnostics::TransferError,
    physics::{SparseMassOutput, SparseMassPlan},
    runtime::TransferSession,
};
use serde_json::{Value, json};

fn close(actual: f32, expected: f64, tolerance: f64) {
    assert!(
        (f64::from(actual) - expected).abs() <= tolerance + tolerance * expected.abs(),
        "a={actual} e={expected}"
    );
}
fn compare(out: &SparseMassOutput, world: usize, case: &Value) -> (usize, usize, f64) {
    let f = out.input().fields();
    let n = out.input().nv();
    let v = out.world(world).unwrap();
    let mut matrix_count = 0;
    let mut factor_count = 0;
    let mut max = 0.0f64;
    let native = case["matrix"].as_array().unwrap();
    let mut lower = vec![0.0f64; n * n];
    let mut diag = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..f.m_rownnz[i] as usize {
            let a = f.m_rowadr[i] as usize + j;
            let c = f.m_colind[a] as usize;
            assert_eq!(
                v.csr_matrix[a].to_bits(),
                v.matrix.matrix[i * n + c].to_bits()
            );
            close(v.csr_matrix[a], native[a].as_f64().unwrap(), 2e-5);
            matrix_count += 1;
            close(
                v.factor.sparse_factors[a],
                case["factor"][a].as_f64().unwrap(),
                2e-4,
            );
            factor_count += 1;
            max = max.max(
                (f64::from(v.factor.sparse_factors[a]) - case["factor"][a].as_f64().unwrap()).abs(),
            );
            if c == i {
                lower[i * n + i] = 1.0;
                diag[i] = f64::from(v.factor.sparse_factors[a]);
            } else {
                lower[i * n + c] = f64::from(v.factor.sparse_factors[a]);
            }
        }
        close(
            v.factor.diagonal_inverse[i],
            case["diagonal_inverse"][i].as_f64().unwrap(),
            2e-4,
        );
        factor_count += 1;
    }
    let norm = v
        .matrix
        .matrix
        .chunks(n)
        .map(|row| row.iter().map(|&x| f64::from(x).abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let mut error = 0.0f64;
    for i in 0..n {
        let mut row = 0.0;
        for j in 0..n {
            let x = (0..n)
                .map(|k| lower[k * n + i] * diag[k] * lower[k * n + j])
                .sum::<f64>();
            row += (x - f64::from(v.matrix.matrix[i * n + j])).abs();
        }
        error = error.max(row);
    }
    assert!(error / norm <= 2e-6, "reconstruction={}", error / norm);
    (matrix_count, factor_count, max)
}
#[test]
fn ties_manual_model_fields_to_pinned_xml_and_builds_other_topology() {
    let refs = input::reference();
    for m in refs["models"].as_array().unwrap().iter().take(3) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "fixtures/sparse-mass-factor/{}.xml",
            m["name"].as_str().unwrap()
        ));
        let xml = std::fs::read_to_string(path).unwrap();
        let model = input::model(m);
        let n = model.rigid().kinematics().nv();
        assert_eq!(xml.matches("<body ").count(), n);
        for text in [
            "pos='.08 .01 .02'",
            "axis='0 0 1' armature='2'",
            "pos='.03 .01 0' mass='.01' diaginertia='.001 .001 .001'",
        ] {
            assert_eq!(xml.matches(text).count(), n);
        }
        assert_eq!(
            model.rigid().fields().dof_parentid,
            input::ints(m, "dof_parentid")
        );
    }
    // 同维但另一拓扑：保留65维，改为一根及64个子自由度。
    let model = input::model(&refs["models"][0]);
    let mut f = input::sparse(&refs["models"][0]).fields().clone();
    f.dof_parentid[2..].fill(0);
    f.m_rownnz = vec![2; 65];
    f.m_rownnz[0] = 1;
    f.m_rowadr = (0..65)
        .map(|i| if i == 0 { 0 } else { 2 * i - 1 })
        .collect();
    f.m_colind = vec![0];
    for i in 1..65 {
        f.m_colind.extend([0, i]);
    }
    let wrong = engine::model::SparseMassInput::new(f).unwrap();
    // 无需GPU会话；下一个设备测试将实际检查构造拒绝。
    assert_ne!(
        model.rigid().fields().dof_parentid,
        wrong.fields().dof_parentid
    );
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC; new CRB-to-CSR composition purpose"]
fn gathers_real_resident_device_matrix_and_factors_independent_native_cases() {
    use sha2::{Digest, Sha256};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/sparse-mass-factor/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(f["path"].as_str().unwrap())).unwrap())
            ),
            f["sha256"].as_str().unwrap()
        );
    }
    let refs = input::reference();
    let session = TransferSession::new(0).unwrap();
    let mut batches = vec![];
    let mut matrix_scalars = 0;
    let mut factor_scalars = 0;
    for m in refs["models"].as_array().unwrap().iter().take(3) {
        let sparse = input::sparse(m);
        let n = sparse.nv();
        let nc = sparse.nc();
        let plan = SparseMassPlan::new(&session, input::model(m), sparse).unwrap();
        for worlds in [1usize, 2, 257] {
            eprintln!("new bridge model={} worlds={worlds}", m["name"]);
            let start = std::time::Instant::now();
            let mut data = plan.create_data(worlds).unwrap();
            let q = (0..worlds)
                .flat_map(|w| input::floats(&m["cases"][w % 2], "qpos"))
                .collect::<Vec<_>>();
            data.write_qpos(&q).unwrap();
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            let mut max = 0.0f64;
            for w in 0..worlds {
                let (a, b, e) = compare(&out, w, &m["cases"][w % 2]);
                matrix_scalars += a;
                factor_scalars += b;
                max = max.max(e);
            }
            batches.push(json!({"model":m["name"],"worlds":worlds,"phase_elapsed_ms":start.elapsed().as_millis(),
                "matrix_scalars":worlds*nc,"factor_scalars":worlds*(nc+n),"additional_bridge_device_bytes":4*(2*n+nc+worlds+8),
                "max_native_factor_abs":max}));
        }
    }
    let report = json!({"scope":"new resident CRB-to-CSR composition","batches":batches,"matrix_scalars":matrix_scalars,
        "factor_scalars":factor_scalars,"CSR_matches_actual_GPU_dense_bits":true,"matrix_tolerance":2e-5,"factor_tolerance":2e-4,
        "reconstruction_limit":2e-6,"frozen_diagnostic_calls":0,"full_precision_equivalence":false,"solve_accepted":false});
    std::fs::create_dir_all(root.join("target/resident-sparse-mass")).unwrap();
    std::fs::write(
        root.join("target/resident-sparse-mass/results.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
#[test]
#[ignore = "requires NVIDIA driver and NVRTC; new gather freshness and ownership purpose"]
fn rejects_foreign_topology_stale_gather_and_preserves_valid_input_rejection() {
    let refs = input::reference();
    let m = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let mut wrong = input::sparse(m).fields().clone();
    wrong.dof_parentid[2..].fill(0);
    wrong.m_rownnz = vec![2; 65];
    wrong.m_rownnz[0] = 1;
    wrong.m_rowadr = (0..65)
        .map(|i| if i == 0 { 0 } else { 2 * i - 1 })
        .collect();
    wrong.m_colind = vec![0];
    for i in 1..65 {
        wrong.m_colind.extend([0, i]);
    }
    assert!(
        SparseMassPlan::new(
            &session,
            input::model(m),
            engine::model::SparseMassInput::new(wrong).unwrap()
        )
        .is_err()
    );
    let plan = SparseMassPlan::new(&session, input::model(m), input::sparse(m)).unwrap();
    let foreign = SparseMassPlan::new(&session, input::model(m), input::sparse(m)).unwrap();
    assert!(plan.create_data(0).is_err());
    let mut data = plan.create_data(257).unwrap();
    data.write_qpos(&input::floats(&m["cases"][0], "qpos").repeat(257))
        .unwrap();
    assert!(plan.update_gather(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    plan.update_position(&mut data).unwrap();
    plan.update_matrix(&mut data).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    plan.update_gather(&mut data).unwrap();
    assert!(data.readback().is_err());
    plan.update_factor(&mut data).unwrap();
    assert!(data.readback().is_ok());
    for result in [
        foreign.update_position(&mut data),
        foreign.update_matrix(&mut data),
        foreign.update_gather(&mut data),
        foreign.update_factor(&mut data),
    ] {
        assert!(matches!(result, Err(TransferError::ModelMismatch)));
    }
    assert!(data.write_qpos(&[]).is_err());
    assert!(
        data.write_world_qpos(257, &input::floats(&m["cases"][0], "qpos"))
            .is_err()
    );
    assert!(data.readback().is_ok());
    let mut nonfinite = input::floats(&m["cases"][0], "qpos");
    nonfinite[0] = f32::NAN;
    assert!(data.write_world_qpos(256, &nonfinite).is_err());
    assert!(data.readback().is_ok());
    data.write_world_qpos(256, &input::floats(&m["cases"][1], "qpos"))
        .unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_gather(&mut data).is_err());
    plan.update_position(&mut data).unwrap();
    plan.update_matrix(&mut data).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    plan.update_gather(&mut data).unwrap();
    plan.update_factor(&mut data).unwrap();
    let out = data.readback().unwrap();
    compare(&out, 256, &m["cases"][1]);
    compare(&out, 0, &m["cases"][0]);
    compare(&out, 255, &m["cases"][0]);
    plan.update_matrix(&mut data).unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_factor(&mut data).is_err());
    drop(data);
    drop(plan);
    drop(foreign);
    drop(session);
    assert!(out.world(256).is_ok());
    assert!(out.world(257).is_err());
}
