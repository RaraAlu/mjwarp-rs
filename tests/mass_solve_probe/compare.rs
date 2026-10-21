use super::*;

pub(super) fn compare(
    out: &MassSolveOutput,
    world: usize,
    case: &Value,
    native_source: &str,
    audit: &mut Audit,
) {
    verify_native_references();
    let w = out.world(world).unwrap();
    let n = out.nv();
    let mut maximum: f64 = 0.0;
    let case_hash = format!("{:x}", Sha256::digest(serde_json::to_vec(case).unwrap()));
    let (reference_model, row) =
        block_reference::row(native_source, out.worlds(), out.rhs_count(), world, case);
    assert_eq!(
        w.blocks.len(),
        reference_model["blocks"].as_array().unwrap().len()
    );
    for (block, expected) in w
        .blocks
        .iter()
        .zip(reference_model["blocks"].as_array().unwrap())
    {
        assert_eq!(expected["start"], block.start);
        assert_eq!(expected["size"], block.size);
        let kind = match expected["route"].as_str().unwrap() {
            "compact" => MassFactorKind::Diagonal,
            "small_cholesky" | "tile_cholesky" => MassFactorKind::Cholesky,
            "sparse_ldl" => MassFactorKind::ReverseLdl,
            other => panic!("unknown route {other}"),
        };
        assert_eq!(block.kind, kind);
    }
    let secondary = |key: &str, range: std::ops::Range<usize>| {
        Ok(Some(SecondaryReference {
            identity: format!(
                "MJWarp:{}:{}:{key}",
                block_reference::reference()["revision"],
                reference_model["name"]
            ),
            conditions: format!(
                "native_case={case_hash};worlds={};rhs_count={};world={world};qpos={};rhs={};f32",
                out.worlds(),
                out.rhs_count(),
                row["qpos_batch_sha256"],
                row["rhs_batch_sha256"]
            ),
            values: row[key].as_array().unwrap()[range]
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect(),
            relative_slack: Some(MJWARP_RELATIVE_SLACK),
        }))
    };
    for (key, actual) in [("ld", w.ld), ("diagonal_inverse", w.diagonal_inverse)] {
        let field = if key == "ld" { "factor" } else { key };
        let expected = row[format!("native_{field}")].as_array().unwrap();
        let identity = format!(
            "MuJoCo3.12.0:matrix-derived-block-factor:case_sha256={case_hash}:field={field}"
        );
        let decision = evaluate(
            actual,
            Some(native_reference(
                identity,
                expected.iter().map(|v| v.as_f64().unwrap()).collect(),
                ABS,
                REL,
            )),
            || secondary(field, 0..actual.len()),
        );
        maximum = maximum.max(audit.record(serde_json::json!({"world": world, "case": case["id"], "case_sha256": case_hash, "field": key}), decision));
    }
    let matrix = case["matrix"].as_array().unwrap();
    let rownorm = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| matrix[i * n + j].as_f64().unwrap().abs())
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    let mut reconstruction: f64 = 0.0;
    for i in 0..n {
        let block = w
            .blocks
            .iter()
            .find(|b| (b.start..b.start + b.size).contains(&i))
            .unwrap();
        match block.kind {
            MassFactorKind::Diagonal => {
                assert!(w.diagonal_inverse[i] > 0.0);
                assert_eq!(w.ld[i * n + i], 0.0);
            }
            MassFactorKind::Cholesky => {
                assert!(w.ld[i * n + i] > 0.0);
                assert_eq!(w.diagonal_inverse[i], 0.0);
            }
            MassFactorKind::ReverseLdl => {
                assert!(w.ld[i * n + i] > 0.0 && w.diagonal_inverse[i] > 0.0);
                assert!(
                    (f64::from(w.ld[i * n + i]) * f64::from(w.diagonal_inverse[i]) - 1.0).abs()
                        <= 2e-6
                );
            }
        }
        let mut error = 0.0;
        for j in 0..n {
            if (block.kind == MassFactorKind::Cholesky && j < i)
                || (block.kind != MassFactorKind::Cholesky && j > i)
                || !(block.start..block.start + block.size).contains(&j)
            {
                assert_eq!(w.ld[i * n + j], 0.0);
            }
            let reconstructed = if !(block.start..block.start + block.size).contains(&j) {
                0.0
            } else if block.kind == MassFactorKind::Diagonal {
                if i == j {
                    1.0 / f64::from(w.diagonal_inverse[i])
                } else {
                    0.0
                }
            } else if block.kind == MassFactorKind::Cholesky {
                (block.start..=i.min(j))
                    .map(|k| f64::from(w.ld[k * n + i]) * f64::from(w.ld[k * n + j]))
                    .sum::<f64>()
            } else {
                (i.max(j)..block.start + block.size)
                    .map(|k| {
                        let lki = if k == i {
                            1.0
                        } else {
                            f64::from(w.ld[k * n + i])
                        };
                        let lkj = if k == j {
                            1.0
                        } else {
                            f64::from(w.ld[k * n + j])
                        };
                        lki * f64::from(w.ld[k * n + k]) * lkj
                    })
                    .sum::<f64>()
            };
            error += (reconstructed - matrix[i * n + j].as_f64().unwrap()).abs();
        }
        reconstruction = reconstruction.max(if rownorm == 0.0 {
            error
        } else {
            error / rownorm
        });
    }
    assert!(
        reconstruction <= RECONSTRUCTION,
        "G03 reconstruction={reconstruction:e}"
    );
    let mut residual: f64 = 0.0;
    assert_eq!(w.solution.len(), out.rhs_count() * n);
    for r in 0..out.rhs_count() {
        let x = &w.solution[r * n..(r + 1) * n];
        let expected = &case["solution"].as_array().unwrap()[r % 3 * n..(r % 3 + 1) * n];
        let rhs = &case["rhs"].as_array().unwrap()[r % 3 * n..(r % 3 + 1) * n];
        let identity = format!(
            "MuJoCo3.12.0:mass-solve:case_sha256={case_hash}:field=solution:rhs={}",
            r % 3
        );
        let decision = evaluate(
            x,
            Some(native_reference(
                identity,
                expected.iter().map(|v| v.as_f64().unwrap()).collect(),
                ABS,
                REL,
            )),
            || secondary("solution", r * n..(r + 1) * n),
        );
        maximum = maximum.max(audit.record(serde_json::json!({"world": world, "case": case["id"], "case_sha256": case_hash, "field": "solution", "rhs": r}), decision));
        let xnorm = x.iter().map(|&v| f64::from(v).abs()).fold(0.0, f64::max);
        let bnorm = rhs
            .iter()
            .map(|v| v.as_f64().unwrap().abs())
            .fold(0.0, f64::max);
        let denominator = rownorm * xnorm + bnorm;
        for i in 0..n {
            let actual = (0..n)
                .map(|j| matrix[i * n + j].as_f64().unwrap() * f64::from(x[j]))
                .sum::<f64>();
            let error = (actual - rhs[i].as_f64().unwrap()).abs();
            residual = residual.max(if denominator == 0.0 {
                error
            } else {
                error / denominator
            });
        }
        if r % 3 == 0 {
            assert!(x.iter().all(|&v| v == 0.0));
        }
    }
    assert!(residual <= RESIDUAL, "G03 residual={residual:e}");
    eprintln!(
        "G03-solve case={} world={world} max_abs_error={maximum:e} residual={residual:e} reconstruction={reconstruction:e}",
        case["id"]
    );
}
