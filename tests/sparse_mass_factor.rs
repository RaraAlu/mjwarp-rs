//! 新增耦合CSR行为；不重放既有G03辅助验收。
use mjwarp_rs::model::{SparseMassFields, SparseMassInput, SparseMassRoute};
use serde_json::Value;
const NATIVE: &str = include_str!("../fixtures/sparse-mass-factor/native.json");
fn reference() -> Value {
    serde_json::from_str(NATIVE).unwrap()
}
fn ints(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap() as i32)
        .collect()
}
fn fields(v: &Value) -> SparseMassFields {
    SparseMassFields {
        dof_parentid: ints(v, "dof_parentid"),
        m_rownnz: ints(v, "m_rownnz"),
        m_rowadr: ints(v, "m_rowadr"),
        m_colind: ints(v, "m_colind"),
        tree_dofadr: ints(v, "tree_dofadr"),
        tree_dofnum: ints(v, "tree_dofnum"),
    }
}
#[test]
fn matches_independent_native_metadata_and_frozen_routes() {
    for m in reference()["models"].as_array().unwrap() {
        let input = SparseMassInput::new(fields(m)).unwrap();
        assert_eq!(input.qld_block_adr(), ints(m, "qld_block_adr"));
        assert_eq!(
            input.qld_block_total(),
            m["qld_block_total"].as_u64().unwrap() as usize
        );
        assert_eq!(input.qld_level_offsets(), ints(m, "qld_level_offsets"));
        assert_eq!(
            input
                .qld_all_updates()
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            ints(m, "qld_updates_flat")
        );
        assert_eq!(input.blocks()[0].route, SparseMassRoute::SparseLdl);
        assert!(
            input
                .qld_updates(input.qld_level_offsets().len() - 1)
                .is_err()
        );
    }
    let empty = SparseMassInput::new(SparseMassFields::default()).unwrap();
    assert_eq!(empty.qld_all_updates(), &[[0, 0, 0]]);
    assert_eq!(empty.qld_level_offsets(), &[0]);
    // 冻结过滤只关心num；空树前、中、后都不影响布局。
    let base = fields(&reference()["models"][2]);
    let expected = SparseMassInput::new(base.clone()).unwrap();
    let mut padded = base;
    padded.tree_dofadr = vec![0, 0, 65, 65, 66];
    padded.tree_dofnum = vec![0, 65, 0, 1, 0];
    let actual = SparseMassInput::new(padded).unwrap();
    assert_eq!(actual.qld_block_adr(), expected.qld_block_adr());
    assert_eq!(actual.qld_all_updates(), expected.qld_all_updates());
    for (rownnz, rowadr, colind, route) in [
        (vec![1, 1], vec![0, 1], vec![0, 1], SparseMassRoute::Compact),
        (
            vec![1, 2],
            vec![0, 1],
            vec![0, 0, 1],
            SparseMassRoute::SmallCholesky,
        ),
    ] {
        let input = SparseMassInput::new(SparseMassFields {
            dof_parentid: vec![-1, 0],
            m_rownnz: rownnz,
            m_rowadr: rowadr,
            m_colind: colind,
            tree_dofadr: vec![0],
            tree_dofnum: vec![2],
        })
        .unwrap();
        assert_eq!(input.blocks()[0].route, route);
    }
}
#[test]
fn rejects_bad_csr_trees_ancestry_and_compressed_sparse_rows() {
    let base = fields(&reference()["models"][0]);
    for case in 0..9 {
        let mut f = base.clone();
        match case {
            0 => {
                f.m_rownnz.pop();
            }
            1 => f.m_rowadr[1] = 0,
            2 => f.m_rownnz[2] = i32::MAX,
            3 => f.m_colind[1] = -1,
            4 => f.m_colind[2] = 0,
            5 => f.dof_parentid[2] = 2,
            6 => f.tree_dofnum[0] = 64,
            7 => f.tree_dofadr[0] = 1,
            _ => {
                // 合法祖先子集不能冒充稀疏完整前缀。
                f.m_colind.remove(1);
                f.m_rownnz[1] = 1;
                for a in &mut f.m_rowadr[2..] {
                    *a -= 1;
                }
            }
        }
        assert!(SparseMassInput::new(f).is_err(), "case={case}");
    }
}

#[cfg(feature = "cuda-probe")]
#[path = "support/sparse_mass_frozen.rs"]
mod frozen;
#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::{
        diagnostics::TransferError, physics::SparseMassFactorPlan, runtime::TransferSession,
    };
    use serde_json::json;
    fn floats(v: &Value, key: &str) -> Vec<f32> {
        v[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect()
    }
    fn compare(actual: &[f32], expected: &[f64]) -> f64 {
        assert_eq!(actual.len(), expected.len());
        let mut max = 0.0f64;
        for (&a, &e) in actual.iter().zip(expected) {
            let err = (f64::from(a) - e).abs();
            max = max.max(err);
            assert!(err <= 2e-4 + 2e-4 * e.abs(), "a={a} e={e} error={err}");
        }
        max
    }
    fn reconstruction(input: &SparseMassInput, factor: &[f32], matrix: &[f32]) -> f64 {
        let n = input.nv();
        let f = input.fields();
        let mut l = vec![0.0f64; n * n];
        let mut m = vec![0.0f64; n * n];
        let mut d = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..f.m_rownnz[i] as usize {
                let a = f.m_rowadr[i] as usize + j;
                let c = f.m_colind[a] as usize;
                m[i * n + c] = f64::from(matrix[a]);
                m[c * n + i] = f64::from(matrix[a]);
                if c == i {
                    d[i] = f64::from(factor[a]);
                    l[i * n + i] = 1.0;
                } else {
                    l[i * n + c] = f64::from(factor[a]);
                }
            }
        }
        let norm = m
            .chunks(n)
            .map(|row| row.iter().map(|x| x.abs()).sum::<f64>())
            .fold(0.0, f64::max);
        let mut error = 0.0f64;
        for i in 0..n {
            let mut row = 0.0;
            for j in 0..n {
                let r = (0..n)
                    .map(|k| l[k * n + i] * d[k] * l[k * n + j])
                    .sum::<f64>();
                row += (r - m[i * n + j]).abs();
            }
            error = error.max(row);
        }
        let result = error / norm;
        assert!(result <= 2e-6, "reconstruction={result}");
        result
    }
    #[test]
    #[ignore = "requires NVIDIA driver, NVRTC and pinned frozen sparse PTX"]
    fn coupled_large_chain_branch_and_compact_match_two_independent_factors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest: Value =
            serde_json::from_str(include_str!("../fixtures/sparse-mass-factor/manifest.json"))
                .unwrap();
        use sha2::{Digest, Sha256};
        for file in manifest["files"].as_array().unwrap() {
            let bytes = std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap();
            assert_eq!(
                format!("{:x}", Sha256::digest(bytes)),
                file["sha256"].as_str().unwrap()
            );
        }
        let session = TransferSession::new(0).unwrap();
        let mut batches = vec![];
        let mut scalars = 0;
        let mut bit_differences = Vec::new();
        let mut changed_bits = 0usize;
        for m in reference()["models"].as_array().unwrap().iter().take(3) {
            let input = SparseMassInput::new(fields(m)).unwrap();
            let plan = SparseMassFactorPlan::new(&session, input.clone()).unwrap();
            let cases = m["cases"].as_array().unwrap();
            let nc = input.nc();
            let n = input.nv();
            assert!(
                cases.iter().any(
                    |c| floats(c, "matrix").iter().enumerate().any(
                        |(a, &v)| input.fields().m_colind[a] as usize
                            != input
                                .fields()
                                .m_rowadr
                                .partition_point(|&r| r as usize <= a)
                                - 1
                            && v != 0.0
                    )
                )
            );
            for worlds in [1usize, 2, 257] {
                eprintln!("sparse batch model={} worlds={worlds}: product", m["name"]);
                let matrix: Vec<_> = (0..worlds)
                    .flat_map(|w| floats(&cases[w % 2], "matrix"))
                    .collect();
                let expected: Vec<_> = (0..worlds)
                    .flat_map(|w| {
                        cases[w % 2]["factor"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap())
                    })
                    .collect();
                let expected_d: Vec<_> = (0..worlds)
                    .flat_map(|w| {
                        cases[w % 2]["diagonal_inverse"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap())
                    })
                    .collect();
                let started = std::time::Instant::now();
                let mut data = plan.create_data(worlds).unwrap();
                data.write_matrix(&matrix).unwrap();
                plan.update_factor(&mut data).unwrap();
                let out = data.readback().unwrap();
                let elapsed = started.elapsed().as_millis();
                eprintln!("sparse batch model={} worlds={worlds}: frozen", m["name"]);
                let (up, up_d) = frozen::factor(
                    &ints(m, "m_rownnz"),
                    &ints(m, "m_rowadr"),
                    &ints(m, "qld_updates_flat"),
                    &ints(m, "qld_level_offsets"),
                    &matrix,
                    worlds,
                );
                let mut native_error = 0.0f64;
                eprintln!(
                    "sparse batch model={} worlds={worlds}: compare and reconstruct",
                    m["name"]
                );
                let mut upstream_error = 0.0f64;
                let mut recon = 0.0f64;
                for w in 0..worlds {
                    let v = out.world(w).unwrap();
                    for (field, actual, expected) in [
                        ("factor", v.sparse_factors, &up[w * nc..(w + 1) * nc]),
                        (
                            "diagonal_inverse",
                            v.diagonal_inverse,
                            &up_d[w * n..(w + 1) * n],
                        ),
                    ] {
                        let count = actual
                            .iter()
                            .zip(expected)
                            .filter(|(a, b)| a.to_bits() != b.to_bits())
                            .count();
                        changed_bits += count;
                        bit_differences.push(json!({"model":m["name"],"worlds":worlds,"world":w,"field":field,"changed_bits":count,"scalars":actual.len()}));
                    }
                    native_error = native_error
                        .max(compare(v.sparse_factors, &expected[w * nc..(w + 1) * nc]));
                    native_error = native_error
                        .max(compare(v.diagonal_inverse, &expected_d[w * n..(w + 1) * n]));
                    upstream_error = upstream_error.max(compare(
                        v.sparse_factors,
                        &up[w * nc..(w + 1) * nc]
                            .iter()
                            .map(|&x| f64::from(x))
                            .collect::<Vec<_>>(),
                    ));
                    upstream_error = upstream_error.max(compare(
                        v.diagonal_inverse,
                        &up_d[w * n..(w + 1) * n]
                            .iter()
                            .map(|&x| f64::from(x))
                            .collect::<Vec<_>>(),
                    ));
                    recon = recon.max(reconstruction(
                        &input,
                        v.sparse_factors,
                        &matrix[w * nc..(w + 1) * nc],
                    ));
                }
                // 独立冻结产物也须过同一原生容差。
                compare(&up, &expected);
                compare(&up_d, &expected_d);
                scalars += worlds * (nc + n);
                let metadata_words = 2 + 2 * n + 3 * input.qld_all_updates().len();
                let bytes = 4 * (metadata_words + worlds * (2 * nc + n) + 16);
                batches.push(json!({"model":m["name"],"worlds":worlds,"elapsed_ms":elapsed,"new_device_bytes":bytes,
                    "native_max_abs":native_error,"frozen_max_abs":upstream_error,"reconstruction":recon,"scalars":worlds*(nc+n)}));
                eprintln!("sparse batch model={} worlds={worlds}: complete", m["name"]);
            }
        }
        let report = json!({"scope":"new coupled sparse factor only","scalars":scalars,"batches":batches,
            "bit_differences":bit_differences,"changed_bits":changed_bits,"bitwise_equal_this_run":changed_bits==0,
            "absolute_tolerance":2e-4,"relative_tolerance":2e-4,"upstream_CU":frozen::CU_HASH,"upstream_PTX":frozen::PTX_HASH,
            "solve_accepted":false,"fair_performance_accepted":false});
        std::fs::create_dir_all(root.join("target/sparse-mass-factor")).unwrap();
        std::fs::write(
            root.join("target/sparse-mass-factor/results.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn rejects_dense_mixed_factors_and_checks_isolation_invalidation_and_bad_pivot() {
        let refs = reference();
        let m = &refs["models"][2];
        let session = TransferSession::new(0).unwrap();
        let empty = SparseMassFactorPlan::new(
            &session,
            SparseMassInput::new(SparseMassFields::default()).unwrap(),
        )
        .unwrap();
        let mut zero = empty.create_data(2).unwrap();
        zero.write_matrix(&[]).unwrap();
        empty.update_factor(&mut zero).unwrap();
        assert!(
            zero.readback()
                .unwrap()
                .world(1)
                .unwrap()
                .sparse_factors
                .is_empty()
        );
        assert!(
            SparseMassFactorPlan::new(
                &session,
                SparseMassInput::new(fields(&refs["models"][3])).unwrap()
            )
            .is_err()
        );
        let plan =
            SparseMassFactorPlan::new(&session, SparseMassInput::new(fields(m)).unwrap()).unwrap();
        let foreign =
            SparseMassFactorPlan::new(&session, SparseMassInput::new(fields(m)).unwrap()).unwrap();
        assert!(plan.create_data(0).is_err());
        assert!(plan.create_data(usize::MAX).is_err());
        let mut data = plan.create_data(2).unwrap();
        let a = floats(&m["cases"][0], "matrix");
        let b = floats(&m["cases"][1], "matrix");
        data.write_matrix(&a.repeat(2)).unwrap();
        plan.update_factor(&mut data).unwrap();
        let before = data.readback().unwrap();
        assert!(matches!(
            foreign.update_factor(&mut data),
            Err(TransferError::ModelMismatch)
        ));
        assert!(data.write_world_matrix(2, &a).is_err());
        assert!(data.write_matrix(&[]).is_err());
        let mut bad = a.clone();
        bad[0] = f32::NAN;
        assert!(data.write_world_matrix(1, &bad).is_err());
        assert!(data.readback().is_ok());
        data.write_world_matrix(1, &b).unwrap();
        assert!(data.readback().is_err());
        plan.update_factor(&mut data).unwrap();
        let after = data.readback().unwrap();
        // 重算含原子加；这里只核对原容差，不声称重算逐位保持。
        compare(
            after.world(0).unwrap().sparse_factors,
            &before
                .world(0)
                .unwrap()
                .sparse_factors
                .iter()
                .map(|&x| f64::from(x))
                .collect::<Vec<_>>(),
        );
        data.write_world_matrix(1, &vec![0.0; a.len()]).unwrap();
        assert!(matches!(
            plan.update_factor(&mut data),
            Err(TransferError::InvalidPivot { world: 1, .. })
        ));
        assert!(data.readback().is_err());
        data.write_world_matrix(1, &a).unwrap();
        plan.update_factor(&mut data).unwrap();
        let snapshot = data.readback().unwrap();
        drop(data);
        drop(plan);
        drop(foreign);
        drop(session);
        assert_eq!(snapshot.world(0).unwrap().sparse_factors.len(), a.len());
        assert!(snapshot.world(2).is_err());
    }
}
