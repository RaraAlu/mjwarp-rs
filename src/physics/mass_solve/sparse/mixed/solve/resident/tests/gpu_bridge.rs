//! 常驻混合GPU桥接数值与生命周期测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
use crate as engine;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new qpos/CRB/CSR/three-route solve bridge numerical purpose"]
fn gpu_complete_bridge_copies_actual_dense_bits_and_keeps_original_field_tolerances() {
    let v = input::reference();
    let native: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/sparse-mass-factor/native.json"
    )))
    .unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
        .unwrap();
    let start = std::time::Instant::now();
    let mut matrix_fields = 0;
    let mut csr_slots = 0;
    let mut factor_slots = 0;
    let mut solution_slots = 0;
    let mut max_matrix = 0.0f64;
    let mut max_factor = 0.0f64;
    let mut max_solution = 0.0f64;
    let mut max_residual = 0.0f64;
    let mut known_bytes = 0;
    let mut batches = 0;
    for sample in 0..2 {
        for (worlds, nr) in [(1, 3), (2, 3), (257, 3), (2, 1), (2, 2), (2, 5)] {
            let mut data = plan.create_data(worlds, nr).unwrap();
            data.write_qpos(&input::qpos(sample).repeat(worlds))
                .unwrap();
            data.write_rhs(&input::rhs(&v, sample, nr).repeat(worlds))
                .unwrap();
            plan.update_position(&mut data).unwrap();
            plan.update_matrix(&mut data).unwrap();
            plan.update_gather(&mut data).unwrap();
            let dense = data.readback_matrix().unwrap();
            let mut csr = vec![0.0; data.solve.mass.matrix.len()];
            data.solve.mass.matrix.read_range_into(0, &mut csr).unwrap();
            engine::physics::check_device_values(&csr, "bridge_test_CSR").unwrap();
            let f = plan.input().fields();
            for w in 0..worlds {
                let source = dense.world(w).unwrap();
                for (i, &a) in source.crb.iter().chain(source.matrix).enumerate() {
                    let expected = if i < 690 {
                        v["cases"][sample]["crb"][i].as_f64().unwrap()
                    } else {
                        v["cases"][sample]["matrix"][i - 690].as_f64().unwrap()
                    };
                    max_matrix = max_matrix.max(close(a, expected, 2e-5));
                    matrix_fields += 1;
                }
                for row in 0..68 {
                    for slot in
                        f.m_rowadr[row] as usize..(f.m_rowadr[row] + f.m_rownnz[row]) as usize
                    {
                        assert_eq!(
                            csr[4 + w * 2149 + slot].to_bits(),
                            source.matrix[68 * row + f.m_colind[slot] as usize].to_bits()
                        );
                        csr_slots += 1;
                    }
                }
            }
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                let factors = out.world(w).unwrap().factor;
                let a = v["cases"][sample]["matrix"][68 * 65 + 65].as_f64().unwrap() as f32;
                let b = v["cases"][sample]["matrix"][68 * 65 + 66].as_f64().unwrap() as f32;
                let d = v["cases"][sample]["matrix"][68 * 66 + 66].as_f64().unwrap() as f32;
                // Unique positive 2x2 U, independent immutable native M only.
                let u = f64::from(a).sqrt();
                let off = f64::from(b) / u;
                let upper = [u, off, 0.0, (f64::from(d) - off * off).sqrt()];
                for (&x, &y) in factors.dense_upper.iter().zip(&upper) {
                    max_factor = max_factor.max(close(x, y, 2e-4));
                    factor_slots += 1;
                }
                assert_eq!(factors.dense_upper[2].to_bits(), 0);
                for i in 0..2149 {
                    let expected = if i < 2145 {
                        native["models"][0]["cases"][sample]["factor"][i]
                            .as_f64()
                            .unwrap()
                    } else {
                        f64::from(csr[4 + w * 2149 + i])
                    };
                    max_factor = max_factor.max(close(factors.sparse_factors[i], expected, 2e-4));
                    factor_slots += 1;
                }
                for i in 0..68 {
                    let expected = if i < 65 {
                        native["models"][0]["cases"][sample]["diagonal_inverse"][i]
                            .as_f64()
                            .unwrap()
                    } else {
                        f64::from(
                            1.0f32
                                / csr[4 + w * 2149 + (f.m_rowadr[i] + f.m_rownnz[i] - 1) as usize],
                        )
                    };
                    max_factor = max_factor.max(close(factors.diagonal_inverse[i], expected, 2e-4));
                    factor_slots += 1;
                }
                let (count, a, r) = compare_solution(&out, w, &v, sample);
                solution_slots += count;
                max_solution = max_solution.max(a);
                max_residual = max_residual.max(r);
            }
            // Exact capacities of the documented subset, not total/peak VRAM.
            let i = plan.input();
            let mixed_meta =
                2 + 2 * i.nv() + 3 * i.qld_all_updates().len() + i.qld_block_total() + 3 + 1;
            let solve_meta =
                4 + 2 * i.nv() + i.qld_level_offsets().len() + 3 * i.qld_all_updates().len() + 3;
            let mixed = 4
                * (worlds
                    * (5 * i.nc() + 2 * i.nv() + 3 * i.qld_block_total() + 2 + 2 * nr * i.nv())
                    + 88
                    + mixed_meta
                    + solve_meta);
            let source = 4 * (worlds * 5314 + 8);
            let producer_meta = 4 * (2 * (69 + 68) + 2 + 68);
            let bridge = 4 * (2 * 68 + 2149 + worlds + 8);
            known_bytes = known_bytes.max(mixed + source + producer_meta + bridge);
            batches += 1;
        }
    }
    assert_eq!(
        (
            batches,
            matrix_fields,
            csr_slots,
            factor_slots,
            solution_slots
        ),
        (12, 2827048, 1143268, 1181572, 108256)
    );
    let report = serde_json::json!({"batches":batches,"CRB_M_fields":matrix_fields,"CSR_bit_slots":csr_slots,"factor_slots":factor_slots,
            "solution_slots":solution_slots,"max_CRB_M_abs":max_matrix,"max_factor_abs":max_factor,"max_solution_abs":max_solution,
            "max_tree_residual":max_residual,"phase_ms":start.elapsed().as_millis(),"known_owned_buffer_subset_bytes":known_bytes,
            "excluded_buffer_accounting":"KinematicsPlan/Data and related model subsets","complete_native_model":false,"full_G03_NP":false});
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_RESIDENT_MIXED_REPORT").unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{report}");
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new qpos generation/RHS factor-reuse bridge lifecycle purpose"]
fn gpu_bridge_qpos_changes_invalidate_every_stage_and_rhs_only_reuses_completed_factor() {
    let v = input::reference();
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
        .unwrap();
    let other =
        SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
            .unwrap();
    let mut data = plan.create_data(257, 3).unwrap();
    let q = input::qpos(0);
    let rhs = input::rhs(&v, 0, 3);
    assert!(plan.update_matrix(&mut data).is_err());
    assert!(plan.update_gather(&mut data).is_err());
    assert!(data.readback().is_err());
    data.write_qpos(&q.repeat(257)).unwrap();
    data.write_rhs(&rhs.repeat(257)).unwrap();
    plan.update(&mut data).unwrap();
    let tokens = |d: &SparseMassMixedResidentData| {
        let state = d.matrix.state_values_for_test().unwrap();
        let mut rhs = vec![0.0; d.solve.rhs.len()];
        d.solve.rhs.read_range_into(0, &mut rhs).unwrap();
        let mut factor = vec![0.0; d.solve.mass.output.len()];
        d.solve.mass.output.read_range_into(0, &mut factor).unwrap();
        (
            state.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            rhs.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            factor.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        )
    };
    let unchanged = tokens(&data);
    assert!(matches!(
        other.update(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.readback().is_ok());
    assert_eq!(tokens(&data), unchanged);
    for (w, values) in [(257, &q[..]), (0, &q[..67])] {
        assert!(data.write_world_qpos(w, values).is_err());
        assert!(data.readback().is_ok());
        assert_eq!(tokens(&data), unchanged);
    }
    let mut bad = q.clone();
    bad[0] = f32::NAN;
    assert!(data.write_world_qpos(0, &bad).is_err());
    assert!(data.readback().is_ok());
    assert_eq!(tokens(&data), unchanged);
    for (w, values) in [(257, &rhs[..]), (0, &rhs[..rhs.len() - 1])] {
        assert!(data.write_world_rhs(w, values).is_err());
        assert!(data.readback().is_ok());
        assert_eq!(tokens(&data), unchanged);
    }
    let mut bad_rhs = rhs.clone();
    bad_rhs[0] = f32::NAN;
    assert!(data.write_world_rhs(0, &bad_rhs).is_err());
    assert!(data.readback().is_ok());
    assert_eq!(tokens(&data), unchanged);
    let old = data.matrix.state_values_for_test().unwrap();
    data.write_world_qpos(256, &input::qpos(1)).unwrap();
    let next = data.matrix.state_values_for_test().unwrap();
    // qpos状态没有结果缓冲的四槽守卫。
    assert_eq!(old.len(), 257 * 68);
    assert_eq!(next.len(), old.len());
    for i in (0..256 * 68).chain(257 * 68..old.len()) {
        assert_eq!(old[i].to_bits(), next[i].to_bits());
    }
    assert_eq!(
        next[256 * 68..257 * 68]
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        input::qpos(1)
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>()
    );
    assert!(plan.update_gather(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_solve(&mut data).is_err());
    data.write_world_rhs(256, &input::rhs(&v, 1, 3)).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    compare_solution(&out, 0, &v, 0);
    compare_solution(&out, 256, &v, 1);
    let mut factors = vec![0.0; data.solve.mass.output.len()];
    data.solve
        .mass
        .output
        .read_range_into(0, &mut factors)
        .unwrap();
    let raw_rhs = data.solve.rhs.len();
    let mut before = vec![0.0; raw_rhs];
    data.solve.rhs.read_range_into(0, &mut before).unwrap();
    data.write_world_rhs(256, &input::rhs(&v, 0, 3)).unwrap();
    assert!(data.readback().is_err());
    plan.update_solve(&mut data).unwrap();
    let mut after = vec![0.0; factors.len()];
    data.solve
        .mass
        .output
        .read_range_into(0, &mut after)
        .unwrap();
    assert_eq!(
        factors.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        after.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    let mut rhs_after = vec![0.0; raw_rhs];
    data.solve.rhs.read_range_into(0, &mut rhs_after).unwrap();
    for i in (0..4 + 256 * 204).chain(4 + 257 * 204..before.len()) {
        assert_eq!(before[i].to_bits(), rhs_after[i].to_bits());
    }
    data.write_world_rhs(256, &input::rhs(&v, 1, 3)).unwrap();
    plan.update_solve(&mut data).unwrap();
    let snapshot = data.readback().unwrap();
    drop(data);
    drop(plan);
    drop(other);
    drop(session);
    compare_solution(&snapshot, 256, &v, 1);
    assert!(snapshot.world(257).is_err());
}
