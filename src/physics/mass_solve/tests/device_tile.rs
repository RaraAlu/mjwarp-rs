//! 质量求解GPU首差证据与Tile因子测试。

#[cfg(feature = "cuda-probe")]
use super::probe::solve_device;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
fn first_difference_evidence_binds_inputs_and_upstream() {
    use sha2::{Digest, Sha256};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/mjwarp-mass-solve/first-difference/manifest.json"
    ))
    .unwrap();
    for (path, hash) in manifest["files"].as_object().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(path)).unwrap())
            ),
            hash.as_str().unwrap(),
            "{path}"
        );
    }
    let upstream: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/mjwarp-mass-solve/first-difference/upstream.json"
    ))
    .unwrap();
    let baseline: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/mjwarp-mass-solve/first-difference/baseline.json"
    ))
    .unwrap();
    assert_eq!(
        upstream["revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    assert_eq!(upstream["worlds"], 8);
    assert_eq!(upstream["rhs_count"], 3);
    for field in [
        "xpos",
        "xquat",
        "xmat",
        "xipos",
        "ximat",
        "xanchor",
        "xaxis",
        "subtree_com",
        "cinert",
        "cdof",
        "crb",
        "matrix",
    ] {
        let bits = |v: &serde_json::Value| {
            v.as_array()
                .unwrap()
                .iter()
                .map(|x| (x.as_f64().unwrap() as f32).to_bits())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            bits(&baseline["rows"][1][field]),
            bits(&upstream["rows"][1][field]),
            "{field}"
        );
    }
    assert_ne!(
        baseline["rows"][1]["factor"][0],
        upstream["rows"][1]["factor"][0]
    );
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn device_tile_factor_matches_frozen_matrix_before_back_substitution() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/mjwarp-mass-solve/first-difference/upstream.json"
    ))
    .unwrap();
    let rows = reference["rows"].as_array().unwrap();
    let s = TransferSession::new(0).unwrap();
    let n = 11;
    let stride = 10 + n * n;
    let mut state = vec![-131072.0; 8 + rows.len() * stride];
    for (w, row) in rows.iter().enumerate() {
        let start = 4 + w * stride;
        state[start..start + 10].fill(0.0);
        for (out, v) in state[start + 10..start + stride]
            .iter_mut()
            .zip(row["matrix"].as_array().unwrap())
        {
            *out = v.as_f64().unwrap() as f32;
        }
    }
    let parents: Vec<_> = (0..n as i32).map(|i| i - 1).collect();
    let layout = MassSolveLayout::new(rows.len(), n, 1).unwrap();
    let out = solve_device(
        &s,
        1,
        layout,
        &s.upload(&state).unwrap(),
        &vec![0.0; rows.len() * n],
        factor_blocks(&parents),
    )
    .unwrap();
    for (w, row) in rows.iter().enumerate() {
        let factor = out.world(w).unwrap().ld;
        for (i, (a, b)) in factor
            .iter()
            .zip(row["factor"].as_array().unwrap())
            .enumerate()
        {
            let b = b.as_f64().unwrap() as f32;
            // Same-matrix trace bounds the observed MathDx rounding only.
            // It does not replace the full native-first solution gates.
            assert!(
                a.to_bits().abs_diff(b.to_bits()) <= 1,
                "same-matrix factor world={w} index={i}: {a} != {b}"
            );
        }
        assert!(out.world(w).unwrap().solution.iter().all(|&x| x == 0.0));
    }
    assert_eq!(
        out.world(1).unwrap().ld[0].to_bits(),
        3.741_657_3_f32.to_bits()
    );

    // Exercise invalid tile pivots, then recover on the same session.
    let layout = MassSolveLayout::new(1, 7, 1).unwrap();
    let parents: Vec<_> = (0..7).map(|i| i - 1).collect();
    let mut state = vec![-131072.0; 67];
    state[4..63].fill(0.0);
    for i in 0..7 {
        state[14 + i * 8] = 4.0;
    }
    for pivot in [0.0, -1.0, f32::INFINITY, f32::NAN] {
        state[14] = pivot;
        assert!(matches!(
            solve_device(
                &s,
                1,
                layout,
                &s.upload(&state).unwrap(),
                &[1.0; 7],
                factor_blocks(&parents)
            ),
            Err(TransferError::InvalidPivot { dof: 0, .. })
        ));
    }
    state[14] = 4.0;
    let out = solve_device(
        &s,
        1,
        layout,
        &s.upload(&state).unwrap(),
        &[1.0; 7],
        factor_blocks(&parents),
    )
    .unwrap();
    assert_eq!(out.world(0).unwrap().solution, &[0.25; 7]);
}
