use super::*;

#[test]
fn solve_references_freeze_shapes_tolerances_and_matching_model_states() {
    verify_native_references();
    for (m, s) in MODELS.into_iter().zip(SOLVES) {
        let m: Value = serde_json::from_str(m).unwrap();
        let s: Value = serde_json::from_str(s).unwrap();
        let model = model(&m);
        let n = model.kinematics().nv();
        assert_eq!(s["native_version"], 3012000);
        assert_eq!(s["seed"], 1789);
        assert_eq!(s["rhs_seed"], 9701);
        assert_eq!(s["ntendon"], 0);
        assert_eq!(s["nu"], 0);
        assert_eq!(s["rhs_count"], 3);
        assert_eq!(s["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(s["relative_tolerance"].as_f64(), Some(REL));
        assert_eq!(s["residual_tolerance"].as_f64(), Some(RESIDUAL));
        assert_eq!(s["reconstruction_tolerance"].as_f64(), Some(RECONSTRUCTION));
        for key in ["nq", "nv", "nbody"] {
            assert_eq!(m[key], s[key]);
        }
        let cases = s["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 8);
        for (i, c) in cases.iter().enumerate() {
            assert_eq!(c["id"], i);
            assert_eq!(c["qpos"], m["cases"][i]["qpos"]);
            assert_eq!(c["matrix"], m["cases"][i]["matrix"]);
            for (key, len) in [
                ("ld", n * n),
                ("diagonal_inverse", n),
                ("rhs", 3 * n),
                ("solution", 3 * n),
            ] {
                let values = c[key].as_array().unwrap();
                assert_eq!(values.len(), len);
                assert!(values.iter().all(|v| v.as_f64().unwrap().is_finite()));
            }
        }
    }
}

#[test]
fn block_references_bind_all_configurations_and_native_factor_semantics() {
    let reference = block_reference::reference();
    assert_eq!(reference["models"].as_array().unwrap().len(), MODELS.len());
    for (index, model) in reference["models"].as_array().unwrap().iter().enumerate() {
        assert_eq!(
            model["model_sha256"],
            format!("{:x}", Sha256::digest(MODELS[index].as_bytes()))
        );
        let native: Value = serde_json::from_str(SOLVES[index]).unwrap();
        let n = native["nv"].as_u64().unwrap() as usize;
        let mut configurations = std::collections::BTreeSet::new();
        for row in model["rows"].as_array().unwrap() {
            let worlds = row["worlds"].as_u64().unwrap() as usize;
            let count = row["rhs_count"].as_u64().unwrap() as usize;
            let world = row["world"].as_u64().unwrap() as usize;
            let case = &native["cases"][world % 8];
            configurations.insert((worlds, count));
            let (_, bound) = block_reference::row(SOLVES[index], worlds, count, world, case);
            assert_eq!(row, bound);
            for (field, size) in [
                ("factor", n * n),
                ("native_factor", n * n),
                ("diagonal_inverse", n),
                ("native_diagonal_inverse", n),
                ("solution", n * count),
            ] {
                let values = row[field].as_array().unwrap();
                assert_eq!(values.len(), size);
                assert!(values.iter().all(|v| v.as_f64().unwrap().is_finite()));
            }
            // 只验证独立参考语义；f64不进入产品计算。
            let f: Vec<_> = row["native_factor"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let matrix: Vec<_> = case["matrix"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let inv: Vec<_> = row["native_diagonal_inverse"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            for i in 0..n {
                for j in 0..n {
                    let value = if i == j && inv[i] > 0.0 {
                        1.0 / inv[i]
                    } else {
                        (0..n).map(|k| f[k * n + i] * f[k * n + j]).sum::<f64>()
                    };
                    assert!(
                        (value - matrix[i * n + j]).abs()
                            <= 1e-12 * (1.0 + matrix[i * n + j].abs())
                    );
                }
            }
        }
        let mut expected = std::collections::BTreeSet::from([(8, 3)]);
        if index == 0 {
            expected.extend([(8, 1), (8, 2), (8, 5)]);
        }
        if index == 2 {
            expected.extend([1, 255, 256, 257, 513].map(|w| (w, 3)));
        }
        if index == 5 {
            expected.insert((1, 3));
        }
        assert_eq!(configurations, expected);
        for (worlds, count) in expected {
            assert_eq!(
                model["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["worlds"] == worlds && r["rhs_count"] == count)
                    .count(),
                worlds
            );
        }
    }
}
