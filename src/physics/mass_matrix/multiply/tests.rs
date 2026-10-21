//! 质量矩阵乘法单元测试。

use super::*;
use crate as engine;
mod source {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/sparse_mass_model.rs"
    ));
}
#[cfg(feature = "cuda-probe")]
mod mixed {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/mixed_mass_solve.rs"
    ));
}
fn sparse() -> SparseMassInput {
    source::sparse(&source::reference()["models"][0])
}
#[test]
fn mirrors_each_csr_address_in_original_append_order() {
    let v = source::reference();
    let i = source::sparse(&v["models"][0]);
    let _ = source::model(&v["models"][0]);
    let _ = source::floats(&v["models"][0]["cases"][0], "qpos");
    let m = metadata(&i).unwrap();
    let n = i.nv();
    let count = m[n] as usize;
    assert_eq!(count, 2 * i.nc() - n);
    for row in 0..n {
        let start = m[row] as usize;
        let end = m[row + 1] as usize;
        let cols = &m[n + 1 + start..n + 1 + end];
        assert_eq!(cols, &(0..n as i32).collect::<Vec<_>>()[..]);
        for k in start..end {
            let col = m[n + 1 + k] as usize;
            let adr = m[n + 1 + count + k] as usize;
            let r = row.max(col);
            assert_eq!(i.fields().m_colind[adr], row.min(col) as i32);
            assert!(adr >= i.fields().m_rowadr[r] as usize);
        }
    }
}
#[test]
fn rejects_empty_overflow_and_invalid_finite_inputs() {
    assert!(layouts(0, 0, 1).is_err());
    assert!(layouts(2, 4, 0).is_err());
    assert!(layouts(390, 4, usize::MAX).is_err());
    if let Some(mc) = 65536usize.checked_mul(65536) {
        assert!(layouts(65536, mc, 1).is_err());
    }
    assert!(checked(&[f32::NAN], 1, "mul_m").is_err());
    assert!(checked(&[], 1, "mul_m").is_err());
    let _ = sparse();
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new mul_m numerical/skip/alternate behavior, no old factor/solve calls"]
fn gpu_multiplies_dense_and_sparse_with_skip_and_independent_alternate() {
    let v = mixed::reference();
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0usize;
    let mut slots = 0usize;
    let mut max_abs = 0.0f64;
    let mut capacity = 0usize;
    for model in v["models"].as_array().unwrap() {
        let input = mixed::input(model);
        let n = input.nv();
        for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
            let plan = MassMultiplyPlan::new(&session, input.clone(), format).unwrap();
            for sample in 0..2 {
                let csr = mixed::matrix(model, sample);
                let full = full_matrix(&input, &csr);
                let matrix = if format == MassMultiplyFormat::Sparse {
                    csr
                } else {
                    full.clone()
                };
                let rhs = mixed::rhs(model, sample, 3);
                let solution = mixed::solution(model, sample, 3);
                for col in 0..3 {
                    let x = solution[col * n..(col + 1) * n]
                        .iter()
                        .map(|&v| v as f32)
                        .collect::<Vec<_>>();
                    let oracle = (0..n)
                        .map(|i| {
                            (0..n)
                                .map(|j| f64::from(full[i * n + j]) * f64::from(x[j]))
                                .sum::<f64>()
                        })
                        .collect::<Vec<_>>();
                    for worlds in [1, 2, 257] {
                        let mut data = plan.create_data(worlds).unwrap();
                        assert!(data.readback().is_err());
                        data.write_matrix(&matrix.repeat(worlds)).unwrap();
                        data.write_vector(&x.repeat(worlds)).unwrap();
                        plan.update(&mut data).unwrap();
                        let out = data.readback().unwrap();
                        for w in 0..worlds {
                            for (i, &a) in out.world(w).unwrap().iter().enumerate() {
                                close(a, oracle[i]);
                                close(a, f64::from(rhs[col * n + i]));
                                max_abs = max_abs.max((f64::from(a) - oracle[i]).abs());
                                slots += 1;
                            }
                        }
                        assert!(out.world(worlds).is_err());
                        capacity = capacity.max(
                            4 * (2 * data.matrix.len()
                                + data.vector.len()
                                + data.output.len()
                                + data.status.len()
                                + data.metadata.len()),
                        );
                        batches += 1;
                    }
                }
            }
        }
    }
    assert_eq!(batches, 72);
    assert_eq!(slots, 1485120);
    for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
        let plan = MassMultiplyPlan::new(&session, mixed::input(&v["models"][0]), format).unwrap();
        let other = MassMultiplyPlan::new(&session, mixed::input(&v["models"][0]), format).unwrap();
        let mut data = plan.create_data(257).unwrap();
        let n = 86;
        let csr = mixed::matrix(&v["models"][0], 0);
        let m = if format == MassMultiplyFormat::Sparse {
            csr.clone()
        } else {
            full_matrix(&mixed::input(&v["models"][0]), &csr)
        };
        let x = vec![1.0; n];
        data.write_matrix(&m.repeat(257)).unwrap();
        data.write_vector(&x.repeat(257)).unwrap();
        plan.update(&mut data).unwrap();
        let original = data.readback().unwrap();
        assert!(other.update(&mut data).is_err());
        assert!(data.write_world_vector(257, &x).is_err());
        assert!(data.write_world_vector(0, &x[..n - 1]).is_err());
        assert!(data.write_world_matrix(257, &m).is_err());
        assert!(data.write_alternate_matrix(&[f32::NAN]).is_err());
        assert!(plan.update_with(&mut data, Some(&[true]), false).is_err());
        assert_eq!(
            bits(&data.readback().unwrap().values),
            bits(&original.values)
        );
        let mut skip = vec![true; 257];
        skip[256] = false;
        let mut alternate = m.iter().map(|v| 2.0 * v).collect::<Vec<_>>();
        if format == MassMultiplyFormat::Dense {
            // Cross-tree asymmetric entry exercises the full row-major M API.
            alternate[n - 1] += 0.375;
        }
        data.write_alternate_matrix(&alternate.repeat(257)).unwrap();
        plan.update_with(&mut data, Some(&skip), true).unwrap();
        let alt = data.readback().unwrap();
        for w in 0..256 {
            assert_eq!(
                bits(alt.world(w).unwrap()),
                bits(original.world(w).unwrap())
            );
        }
        let full = if format == MassMultiplyFormat::Dense {
            alternate.clone()
        } else {
            full_matrix(&mixed::input(&v["models"][0]), &alternate)
        };
        for (i, &a) in alt.world(256).unwrap().iter().enumerate() {
            close(
                a,
                (0..n)
                    .map(|j| f64::from(full[i * n + j]) * f64::from(x[j]))
                    .sum(),
            );
        }
        // Selecting an alternate matrix never overwrites the default input.
        plan.update(&mut data).unwrap();
        assert_eq!(
            bits(&data.readback().unwrap().values),
            bits(&original.values)
        );
        let before_matrix = raw(&data.matrix);
        let mut selected = m.clone();
        selected[0] += 0.125;
        data.write_world_matrix(256, &selected).unwrap();
        let after_matrix = raw(&data.matrix);
        let mc = m.len();
        for i in (0..4 + 256 * mc).chain(4 + 257 * mc..before_matrix.len()) {
            assert_eq!(before_matrix[i].to_bits(), after_matrix[i].to_bits());
        }
        assert_eq!(
            bits(&after_matrix[4 + 256 * mc..4 + 257 * mc]),
            bits(&selected)
        );
        let before = raw(&data.vector);
        data.write_world_vector(256, &vec![0.0; n]).unwrap();
        let after = raw(&data.vector);
        for i in (0..4 + 256 * n).chain(4 + 257 * n..before.len()) {
            assert_eq!(before[i].to_bits(), after[i].to_bits());
        }
        assert!(data.readback().is_err());
        plan.update_with(&mut data, Some(&skip), false).unwrap();
        let snapshot = data.readback().unwrap();
        assert!(
            snapshot
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 0)
        );
        drop(data);
        drop(plan);
        drop(other);
        assert_eq!(
            bits(snapshot.world(0).unwrap()),
            bits(original.world(0).unwrap())
        );
    }
    drop(session);
    let report = serde_json::json!({"batches":batches,"slots":slots,"max_abs":max_abs,"phase_ms":start.elapsed().as_millis(),"owned_capacity_max_bytes":capacity,"abs":2e-5,"rel":2e-5,"native_or_upstream_equivalence":false});
    std::fs::write(
        std::env::var_os("MJWARP_MUL_M_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
#[cfg(feature = "cuda-probe")]
fn full_matrix(input: &SparseMassInput, csr: &[f32]) -> Vec<f32> {
    let n = input.nv();
    let f = input.fields();
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        let a = f.m_rowadr[i] as usize;
        let row = a..a + f.m_rownnz[i] as usize;
        for (&j, &value) in f.m_colind[row.clone()].iter().zip(&csr[row]) {
            let j = j as usize;
            out[i * n + j] = value;
            out[j * n + i] = value;
        }
    }
    out
}
#[cfg(feature = "cuda-probe")]
fn close(a: f32, b: f64) {
    assert!(
        (f64::from(a) - b).abs() <= 2e-5 + 2e-5 * b.abs(),
        "mul_m {a} vs {b}"
    );
}
#[cfg(feature = "cuda-probe")]
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
fn raw(v: &TransferBuffer<f32>) -> Vec<f32> {
    let mut out = vec![0.0; v.len()];
    v.read_range_into(0, &mut out).unwrap();
    out
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new mul_m source/output/status guards, skipped payload and no partial publish"]
fn gpu_rejects_source_guards_and_nonfinite_without_publishing_partial_result() {
    let session = TransferSession::new(0).unwrap();
    let i = sparse();
    let n = i.nv();
    let nc = i.nc();
    let plan = MassMultiplyPlan::new(&session, i, MassMultiplyFormat::Sparse).unwrap();
    let mut d = plan.create_data(2).unwrap();
    d.write_matrix(&vec![1.0; 2 * nc]).unwrap();
    d.write_vector(&vec![1.0; 2 * n]).unwrap();
    plan.update(&mut d).unwrap();
    let good = d.readback().unwrap();
    for (which, offset, bad) in [
        (0, 0, 0.0),
        (0, 4 + 2 * nc, 0.0),
        (0, 4, f32::NAN),
        (1, 0, 0.0),
        (1, 4 + 2 * n, 0.0),
        (1, 4, f32::INFINITY),
        (2, 0, 0.0),
        (2, 4 + 2 * n, 0.0),
        (3, 0, 0.0),
        (3, 6, 0.0),
    ] {
        let buf = match which {
            0 => &mut d.matrix,
            1 => &mut d.vector,
            2 => &mut d.output,
            _ => &mut d.status,
        };
        let mut old = [0.0];
        buf.read_range_into(offset, &mut old).unwrap();
        buf.write_range(offset, &[bad]).unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        let buf = match which {
            0 => &mut d.matrix,
            1 => &mut d.vector,
            2 => &mut d.output,
            _ => &mut d.status,
        };
        buf.write_range(offset, &old).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    }
    d.matrix.write_range(4, &[f32::NAN]).unwrap();
    plan.update_with(&mut d, Some(&[true, false]), false)
        .unwrap();
    assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    assert!(plan.update(&mut d).is_err());
    d.matrix.write_range(4, &[1.0]).unwrap();
    assert!(plan.update_with(&mut d, None, true).is_err());
    d.write_alternate_matrix(&vec![2.0; 2 * nc]).unwrap();
    plan.update_with(&mut d, None, true).unwrap();
    let alt = d.readback().unwrap();
    for (&a, &b) in alt.world(0).unwrap().iter().zip(good.world(0).unwrap()) {
        assert_eq!(a.to_bits(), (2.0 * b).to_bits());
    }
}
