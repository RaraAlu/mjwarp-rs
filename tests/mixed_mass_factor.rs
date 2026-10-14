#![cfg(feature = "cuda-probe")]
//! 首次混合生产结果及跨阶段状态；不运行旧测试或作者。
use mjwarp_rs as engine;
#[path = "support/small_mass_prefix.rs"]
mod input;
use engine::diagnostics::{InputError, TransferError};
use engine::physics::{SparseMassMixedFactorOutput, SparseMassMixedFactorPlan};
use engine::runtime::TransferSession;
use serde_json::json;

fn expected(value: &serde_json::Value) -> Vec<f64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect()
}
fn compare(out: &SparseMassMixedFactorOutput, w: usize, sample: usize) -> (usize, f64, f64) {
    let base = input::base();
    let analytic = input::analytic();
    let matrix = input::mixed_matrix(sample);
    let world = out.world(w).unwrap();
    let native = &base["models"][0]["cases"][sample];
    let mut tail = expected(&native["factor"]);
    tail.extend(matrix[tail.len()..].iter().map(|&v| f64::from(v)));
    let prefix = expected(&analytic["cases"][sample]["upper"]);
    let mut diagonal = expected(&native["diagonal_inverse"]);
    let f = out.input().fields();
    for dof in diagonal.len()..out.input().nv() {
        let slot = (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize;
        diagonal.push(f64::from(1.0f32 / matrix[slot]));
    }
    let mut max = 0.0f64;
    let mut slots = 0;
    for (actual, reference) in [
        (world.dense_upper, &prefix),
        (world.sparse_factors, &tail),
        (world.diagonal_inverse, &diagonal),
    ] {
        assert_eq!(actual.len(), reference.len());
        for (&a, &b) in actual.iter().zip(reference) {
            let error = (f64::from(a) - b).abs();
            assert!(a.is_finite() && error <= 2e-4 + 2e-4 * b.abs());
            max = max.max(error);
            slots += 1;
        }
    }
    // Sparse updates never own Small or Compact rows. Copy their original M,
    // not their U. Full-nv reciprocal must use those original M diagonals.
    for i in 2145..matrix.len() {
        assert_eq!(world.sparse_factors[i].to_bits(), matrix[i].to_bits());
    }
    for dof in 65..out.input().nv() {
        let slot = (f.m_rowadr[dof] + f.m_rownnz[dof] - 1) as usize;
        assert_eq!(
            world.diagonal_inverse[dof].to_bits(),
            (1.0f32 / matrix[slot]).to_bits()
        );
    }
    let mut reconstruction = 0.0f64;
    // Independent reconstruction is diagnostic f64, never product input.
    for i in 0..65 {
        for j in 0..65 {
            let value = (i.max(j)..65)
                .map(|k| {
                    let start = f.m_rowadr[k] as usize;
                    let a = if k == i {
                        1.0
                    } else {
                        f64::from(world.sparse_factors[start + i])
                    };
                    let b = if k == j {
                        1.0
                    } else {
                        f64::from(world.sparse_factors[start + j])
                    };
                    a * f64::from(world.sparse_factors[start + k]) * b
                })
                .sum::<f64>();
            let m = f64::from(matrix[f.m_rowadr[i.max(j)] as usize + i.min(j)]);
            let error = (value - m).abs() / m.abs().max(1.0);
            assert!(error <= 2e-6);
            reconstruction = reconstruction.max(error);
        }
    }
    for block in out
        .input()
        .blocks()
        .iter()
        .filter(|b| b.start >= 65 && b.size > 1)
    {
        let address = out.input().qld_block_adr()[block.start] as usize;
        for i in 0..block.size {
            for j in 0..block.size {
                let value = (0..block.size)
                    .map(|k| {
                        f64::from(world.dense_upper[address + k * block.size + i])
                            * f64::from(world.dense_upper[address + k * block.size + j])
                    })
                    .sum::<f64>();
                let row = block.start + i.max(j);
                let column = (block.start + i.min(j)) as i32;
                let start = f.m_rowadr[row] as usize;
                let slot = start
                    + f.m_colind[start..start + f.m_rownnz[row] as usize]
                        .binary_search(&column)
                        .unwrap();
                let m = f64::from(matrix[slot]);
                let error = (value - m).abs() / m.abs().max(1.0);
                assert!(error <= 2e-6);
                reconstruction = reconstruction.max(error);
                if i > j {
                    assert_eq!(world.dense_upper[address + i * block.size + j].to_bits(), 0);
                }
            }
        }
    }
    (slots, max, reconstruction)
}

#[test]
#[ignore = "NVIDIA/NVRTC; new complete three-route mixed output purpose"]
fn gpu_mixed_prefix_csr_tail_and_all_dof_inverse_follow_frozen_stage_order() {
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedFactorPlan::new(&session, input::mixed_input()).unwrap();
    let start = std::time::Instant::now();
    let mut count = 0;
    let mut max = 0.0f64;
    let mut reconstruction = 0.0f64;
    for sample in 0..2 {
        let matrix = input::mixed_matrix(sample);
        for worlds in [1, 2, 257] {
            let mut data = plan.create_data(worlds).unwrap();
            data.write_matrix(&matrix.repeat(worlds)).unwrap();
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            assert_eq!(out.worlds(), worlds);
            for w in 0..worlds {
                let (n, a, r) = compare(&out, w, sample);
                count += n;
                max = max.max(a);
                reconstruction = reconstruction.max(r);
            }
        }
    }
    assert_eq!(count, 1236040);
    let i = plan.input();
    let metadata = 2 + 2 * i.nv() + 3 * i.qld_all_updates().len() + i.qld_block_total() + 3 * 5 + 1;
    let device_bytes =
        4 * (257 * (5 * i.nc() + 2 * i.nv() + 3 * i.qld_block_total() + 1) + 64 + metadata);
    let report = json!({"slots":count,"max_abs":max,"max_reconstruction":reconstruction,
        "phase_ms":start.elapsed().as_millis(),"max_owned_device_buffer_bytes":device_bytes,
        "batches":6,"lower_positive_zero_slots":18200,"copied_small_compact_csr_slots":29120,
        "non_sparse_inverse_bit_slots":10920,"full_G03_NP":false});
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_MIXED_FACTOR_REPORT").unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{report}");
}

#[test]
#[ignore = "NVIDIA/NVRTC; new mixed ownership/invalidations and pivot failure purpose"]
fn gpu_mixed_world_writes_invalidate_all_stages_and_failures_never_publish_partial_results() {
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedFactorPlan::new(&session, input::mixed_input()).unwrap();
    let other = SparseMassMixedFactorPlan::new(&session, input::mixed_input()).unwrap();
    let mut data = plan.create_data(257).unwrap();
    let original = input::mixed_matrix(0);
    let changed = input::mixed_matrix(1);
    assert!(data.readback().is_err());
    data.write_matrix(&original.repeat(257)).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    plan.update(&mut data).unwrap();
    assert!(matches!(
        other.update(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.readback().is_ok());
    for (world, values) in [(257, &original[..]), (0, &original[..original.len() - 1])] {
        assert!(matches!(
            data.write_world_matrix(world, values),
            Err(TransferError::Input(_))
        ));
        assert!(data.readback().is_ok());
    }
    let mut invalid = original.clone();
    invalid[0] = f32::NAN;
    assert!(matches!(
        data.write_world_matrix(0, &invalid),
        Err(TransferError::Input(InputError::NonFinite { .. }))
    ));
    assert!(data.readback().is_ok());
    data.write_world_matrix(256, &changed).unwrap();
    assert!(data.readback().is_err());
    assert!(plan.update_factor(&mut data).is_err());
    plan.update(&mut data).unwrap();
    let after = data.readback().unwrap();
    compare(&after, 0, 0);
    compare(&after, 255, 0);
    compare(&after, 256, 1);
    // Selected world changes primary matrix for each of the three factor routes.
    for (slot, dof) in [(0, 0), (2145, 65), (2200, 85)] {
        let mut bad = changed.clone();
        bad[slot] = -1.0;
        data.write_world_matrix(256, &bad).unwrap();
        let result = plan.update(&mut data);
        assert!(
            matches!(result, Err(TransferError::InvalidPivot { world:256,dof:bad_dof,.. }) if bad_dof == dof)
        );
        assert!(data.readback().is_err());
        data.write_world_matrix(256, &changed).unwrap();
        plan.update(&mut data).unwrap();
    }
    let snapshot = data.readback().unwrap();
    drop(data);
    drop(plan);
    drop(other);
    drop(session);
    compare(&snapshot, 256, 1);
    assert!(snapshot.world(257).is_err());
}
