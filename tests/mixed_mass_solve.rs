#![cfg(feature = "cuda-probe")]
//! 首次三路混合消费；只读静态参考，不运行任何作者。
use mjwarp_rs as engine;
#[path = "support/mixed_mass_solve.rs"]
mod input;
use engine::diagnostics::TransferError;
use engine::physics::{SparseMassMixedSolveOutput, SparseMassMixedSolvePlan};
use engine::runtime::TransferSession;
use serde_json::json;

fn compare(
    out: &SparseMassMixedSolveOutput,
    w: usize,
    model: &serde_json::Value,
    sample: usize,
) -> (usize, usize, f64, f64) {
    let actual = out.world(w).unwrap();
    let n = out.input().nv();
    let nr = out.rhs_count();
    let rhs = input::rhs(model, sample, nr);
    let reference = input::solution(model, sample, nr);
    let matrix = input::matrix(model, sample);
    let f = out.input().fields();
    let mut max = 0.0f64;
    let mut zero = 0;
    for (index, ((&a, &b), (&r, &s))) in actual
        .solution
        .iter()
        .zip(&reference)
        .zip(actual.rhs.iter().zip(&rhs))
        .enumerate()
    {
        assert_eq!(r.to_bits(), s.to_bits());
        let error = (f64::from(a) - b).abs();
        assert!(a.is_finite() && error <= 2e-4 + 2e-4 * b.abs());
        max = max.max(error);
        if index / n % 3 == 0 {
            assert_eq!(a, 0.0);
            zero += 1;
        }
    }
    let mut row_norm = vec![0.0f64; n];
    for i in 0..n {
        for slot in f.m_rowadr[i] as usize..(f.m_rowadr[i] + f.m_rownnz[i]) as usize {
            let k = f.m_colind[slot] as usize;
            let value = f64::from(matrix[slot]).abs();
            row_norm[i] += value;
            if k != i {
                row_norm[k] += value;
            }
        }
    }
    let mut worst = 0.0f64;
    for r in 0..nr {
        let x = &actual.solution[r * n..(r + 1) * n];
        let b = &rhs[r * n..(r + 1) * n];
        let mut ax = vec![0.0f64; n];
        for i in 0..n {
            for slot in f.m_rowadr[i] as usize..(f.m_rowadr[i] + f.m_rownnz[i]) as usize {
                let k = f.m_colind[slot] as usize;
                let value = f64::from(matrix[slot]);
                ax[i] += value * f64::from(x[k]);
                if k != i {
                    ax[k] += value * f64::from(x[i]);
                }
            }
        }
        // Check each tree independently: a high-norm Small matrix must never
        // dilute a Sparse residual through the full mixed-matrix denominator.
        for block in out.input().blocks() {
            let mut an = 0.0f64;
            let mut xn = 0.0f64;
            let mut bn = 0.0f64;
            let mut residual = 0.0f64;
            for i in block.start..block.start + block.size {
                an = an.max(row_norm[i]);
                xn = xn.max(f64::from(x[i]).abs());
                bn = bn.max(f64::from(b[i]).abs());
                residual = residual.max((ax[i] - f64::from(b[i])).abs());
            }
            let relative = residual / (an * xn + bn).max(1e-30);
            assert!(relative <= 2e-5);
            worst = worst.max(relative);
        }
    }
    (reference.len(), zero, max, worst)
}
fn buffer_bytes(i: &engine::model::SparseMassInput, worlds: usize, nr: usize) -> usize {
    let small = i
        .blocks()
        .iter()
        .filter(|b| b.route == engine::model::SparseMassRoute::SmallCholesky)
        .count();
    let factor_meta =
        2 + 2 * i.nv() + 3 * i.qld_all_updates().len() + i.qld_block_total() + 3 * small + 1;
    let solve_meta =
        4 + 2 * i.nv() + i.qld_level_offsets().len() + 3 * i.qld_all_updates().len() + 3 * small;
    4 * (worlds * (5 * i.nc() + 2 * i.nv() + 3 * i.qld_block_total() + 2 + 2 * nr * i.nv())
        + 88
        + factor_meta
        + solve_meta)
}
#[test]
#[ignore = "NVIDIA/NVRTC; new mixed solve route/stride/RHS consumer numeric purpose"]
fn gpu_mixed_consumes_all_three_routes_without_overwrite_and_crosses_128_groups() {
    let refs = input::reference();
    let session = TransferSession::new(0).unwrap();
    let mut slots = 0;
    let mut zeros = 0;
    let mut max = 0.0f64;
    let mut residual = 0.0f64;
    let mut milliseconds = 0u128;
    let mut bytes = 0;
    let mut batches = 0;
    for (m, model) in refs["models"].as_array().unwrap().iter().enumerate() {
        let plan = SparseMassMixedSolvePlan::new(&session, input::input(model)).unwrap();
        let runs = if m == 0 {
            vec![(1, 3), (2, 3), (257, 3), (2, 1), (2, 2), (2, 5)]
        } else {
            vec![(2, 3)]
        };
        let start = std::time::Instant::now();
        for sample in 0..2 {
            for &(worlds, nr) in &runs {
                let mut data = plan.create_data(worlds, nr).unwrap();
                data.write_matrix(&input::matrix(model, sample).repeat(worlds))
                    .unwrap();
                data.write_rhs(&input::rhs(model, sample, nr).repeat(worlds))
                    .unwrap();
                plan.update(&mut data).unwrap();
                let out = data.readback().unwrap();
                for w in 0..worlds {
                    let (s, z, a, r) = compare(&out, w, model, sample);
                    slots += s;
                    zeros += z;
                    max = max.max(a);
                    residual = residual.max(r);
                }
                bytes = bytes.max(buffer_bytes(plan.input(), worlds, nr));
                batches += 1;
            }
        }
        milliseconds += start.elapsed().as_millis();
    }
    assert_eq!((slots, zeros, batches), (141592, 47656, 14));
    let report = json!({"slots":slots,"zero_rhs_slots":zeros,"batches":batches,"max_abs":max,
        "max_block_residual":residual,"phase_ms":milliseconds,"max_owned_buffer_bytes":bytes,
        "sparse_max_dofs":131,"small_max_blocks":129,"full_G03_NP":false});
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_MIXED_SOLVE_REPORT").unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{report}");
}
fn factors(out: &SparseMassMixedSolveOutput) -> Vec<u32> {
    (0..out.worlds())
        .flat_map(|w| {
            let v = out.world(w).unwrap().factor;
            v.dense_upper
                .iter()
                .chain(v.sparse_factors)
                .chain(v.diagonal_inverse)
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        })
        .collect()
}
#[test]
#[ignore = "NVIDIA/NVRTC; new mixed factor reuse and RHS-only invalidation purpose"]
fn gpu_rhs_updates_preserve_factors_and_matrix_writes_require_new_same_generation_factor() {
    let refs = input::reference();
    let model = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedSolvePlan::new(&session, input::input(model)).unwrap();
    let foreign = SparseMassMixedSolvePlan::new(&session, input::input(model)).unwrap();
    let mut data = plan.create_data(257, 3).unwrap();
    let matrix = input::matrix(model, 0);
    let rhs = input::rhs(model, 0, 3);
    data.write_matrix(&matrix.repeat(257)).unwrap();
    data.write_rhs(&rhs.repeat(257)).unwrap();
    assert!(plan.update_solve(&mut data).is_err());
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    let bits = factors(&before);
    assert!(matches!(
        foreign.update_solve(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.readback().is_ok());
    for (world, bad) in [(257, &rhs[..]), (0, &rhs[..rhs.len() - 1])] {
        assert!(data.write_world_rhs(world, bad).is_err());
        assert!(data.readback().is_ok());
    }
    let mut bad = rhs.clone();
    bad[0] = f32::NAN;
    assert!(data.write_world_rhs(0, &bad).is_err());
    assert!(data.readback().is_ok());
    // Swap independent reference RHS columns, without touching the factor.
    let n = plan.input().nv();
    let mut swap = rhs.clone();
    swap[..n].copy_from_slice(&rhs[n..2 * n]);
    swap[n..2 * n].copy_from_slice(&rhs[..n]);
    data.write_world_rhs(256, &swap).unwrap();
    assert!(data.readback().is_err());
    plan.update_solve(&mut data).unwrap();
    let out = data.readback().unwrap();
    assert_eq!(factors(&out), bits);
    for w in [0, 255] {
        compare(&out, w, model, 0);
    }
    let expected = input::solution(model, 0, 3);
    let actual = out.world(256).unwrap();
    for r in 0..3 {
        let reference = if r == 0 {
            1
        } else if r == 1 {
            0
        } else {
            2
        };
        for i in 0..n {
            let a = f64::from(actual.solution[r * n + i]);
            let b = expected[reference * n + i];
            assert!((a - b).abs() <= 2e-4 + 2e-4 * b.abs());
        }
    }
    assert!(data.write_world_matrix(257, &matrix).is_err());
    assert!(data.readback().is_ok());
    data.write_world_matrix(256, &input::matrix(model, 1))
        .unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_solve(&mut data).is_err());
    data.write_world_rhs(256, &input::rhs(model, 1, 3)).unwrap();
    plan.update(&mut data).unwrap();
    let snapshot = data.readback().unwrap();
    compare(&snapshot, 256, model, 1);
    compare(&snapshot, 0, model, 0);
    drop(data);
    drop(plan);
    drop(foreign);
    drop(session);
    compare(&snapshot, 256, model, 1);
    assert!(snapshot.world(257).is_err());
}
