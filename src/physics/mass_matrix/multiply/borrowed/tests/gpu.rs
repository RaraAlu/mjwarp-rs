//! 借用矩阵 GPU 消费者测试。

use super::super::super::MassMultiplyFormat;
use super::super::MassMatrixMultiplyPlan;
use super::{input, model};
use crate::physics::MassMatrixPlan;
use crate::runtime::{TransferBuffer, TransferSession};

#[cfg(feature = "cuda-probe")]
fn raw(b: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; b.len()];
    b.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new live CRB/M borrowed consumer; conditional M-dot oracle, not old CRB/mul_m/bridge admission"]
fn gpu_consumes_the_actual_resident_matrix_without_m_copy_or_host_feedback() {
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0;
    let mut slots = 0;
    let mut dependencies = 0;
    let mut max_abs = 0.0f64;
    let mut capacity = 0;
    let mut last_snapshot = None;
    for n in [3, 33, 65] {
        let i = input(n, true);
        let producer = MassMatrixPlan::new(&session, model(&i)).unwrap();
        let plans = [
            MassMatrixMultiplyPlan::new(&producer, i.clone(), MassMultiplyFormat::Sparse).unwrap(),
            MassMatrixMultiplyPlan::new(&producer, i.clone(), MassMultiplyFormat::Dense).unwrap(),
        ];
        assert!(
            MassMatrixMultiplyPlan::new(&producer, input(n, false), MassMultiplyFormat::Sparse)
                .is_err()
        );
        let foreign = MassMatrixPlan::new(&session, model(&i)).unwrap();
        for worlds in [1, 2, 257] {
            let mut source = producer.create_data(worlds).unwrap();
            let foreign_source = foreign.create_data(worlds).unwrap();
            let mut data = [
                plans[0].create_data(worlds).unwrap(),
                plans[1].create_data(worlds).unwrap(),
            ];
            for index in 0..2 {
                assert!(
                    plans[index]
                        .update(&mut data[index], &source, None)
                        .is_err()
                );
                assert!(
                    plans[index]
                        .update(&mut data[index], &foreign_source, None)
                        .is_err()
                );
            }
            for state in 0..2 {
                let positions: Vec<f32> = (0..worlds * n)
                    .map(|j| ((j + state) % 9) as f32 * 0.07 - 0.28)
                    .collect();
                source.write_qpos(&positions).unwrap();
                // Existing producers execute only as dependencies of the
                // new live-allocation connection; no old gold is assessed.
                producer.update(&mut source).unwrap();
                dependencies += 1;
                let m = source.readback().unwrap();
                let source_bits = bits(&raw(source.matrix_input().unwrap()));
                // Freshness is per-call, not inferred from initial zeros,
                // old outputs or a new state of the same source allocation.
                for index in 0..2 {
                    let before = bits(&raw(&data[index].current));
                    let view = plans[index]
                        .update(&mut data[index], &source, Some(&vec![true; worlds]))
                        .unwrap();
                    assert!(view.fresh_snapshot().is_err());
                    assert!(view.is_current(worlds).is_err());
                    for w in 0..worlds {
                        assert!(!view.is_current(w).unwrap());
                    }
                    assert_eq!(before, bits(&view.snapshot().unwrap().values));
                }
                for vector in 0..2 {
                    let q: Vec<f32> = (0..worlds * n)
                        .map(|j| ((j + 3 * vector + state) % 9) as f32 / 8.0 - 0.5)
                        .collect();
                    for index in 0..2 {
                        data[index].write_vector(&q).unwrap();
                        // Vector changes must not invalidate/reallocate M.
                        assert_eq!(source_bits, bits(&raw(source.matrix_input().unwrap())));
                        let view = plans[index]
                            .update(&mut data[index], &source, None)
                            .unwrap();
                        assert_eq!(view.worlds(), worlds);
                        for w in 0..worlds {
                            assert!(view.is_current(w).unwrap());
                        }
                        let out = view.fresh_snapshot().unwrap();
                        assert!(out.world(worlds).is_err());
                        for w in 0..worlds {
                            let world_matrix = m.world(w).unwrap().matrix;
                            for (r, &actual) in out.world(w).unwrap().iter().enumerate() {
                                let expected = (0..n)
                                    .map(|c| world_matrix[r * n + c] as f64 * q[w * n + c] as f64)
                                    .sum::<f64>();
                                let err = (actual as f64 - expected).abs();
                                assert!(
                                    err <= 2e-5 + 2e-5 * expected.abs(),
                                    "n={n} w={w} row={r} actual={actual} expected={expected}"
                                );
                                max_abs = max_abs.max(err);
                                slots += 1;
                            }
                        }
                        drop(view);
                        capacity = capacity.max(
                            4 * (data[index].metadata.len()
                                + data[index].vector.len()
                                + data[index].current.len()
                                + data[index].staged.len()
                                + data[index].status.len()),
                        );
                        last_snapshot = Some(out);
                        batches += 1;
                    }
                }
            }
            for index in 0..2 {
                let before = bits(&raw(&data[index].current));
                let q = vec![0.375; n];
                data[index].write_world_vector(worlds - 1, &q).unwrap();
                let mut skip = vec![true; worlds];
                skip[worlds - 1] = false;
                let view = plans[index]
                    .update(&mut data[index], &source, Some(&skip))
                    .unwrap();
                let out = view.snapshot().unwrap();
                for w in 0..worlds {
                    assert_eq!(view.is_current(w).unwrap(), w == worlds - 1);
                }
                if worlds > 1 {
                    assert!(view.fresh_snapshot().is_err());
                }
                let a = 4 + (worlds - 1) * n;
                assert_eq!(&before[..a], &bits(&out.values)[..a]);
                drop(view);
                let before = bits(&raw(&data[index].current));
                assert!(
                    plans[1 - index]
                        .update(&mut data[index], &source, None)
                        .is_err()
                );
                assert!(
                    plans[index]
                        .update(&mut data[index], &source, Some(&[]))
                        .is_err()
                );
                assert!(data[index].write_world_vector(worlds, &q).is_err());
                assert!(data[index].write_vector(&[f32::NAN]).is_err());
                assert_eq!(before, bits(&raw(&data[index].current)));
            }
            if n == 3 && worlds == 2 {
                let mut another = producer.create_data(worlds).unwrap();
                another.write_qpos(&[0.2; 6]).unwrap();
                producer.update(&mut another).unwrap();
                dependencies += 1;
                let matrix = another.readback().unwrap();
                for index in 0..2 {
                    let before = bits(&raw(&data[index].current));
                    let q = vec![0.25; 6];
                    data[index].write_vector(&q).unwrap();
                    let view = plans[index]
                        .update(&mut data[index], &another, Some(&[true, false]))
                        .unwrap();
                    assert!(!view.is_current(0).unwrap());
                    assert!(view.is_current(1).unwrap());
                    assert!(view.fresh_snapshot().is_err());
                    let out = view.snapshot().unwrap();
                    assert_eq!(&before[4..7], &bits(out.world(0).unwrap()));
                    for r in 0..3 {
                        let expected = (0..3)
                            .map(|c| matrix.world(1).unwrap().matrix[3 * r + c] as f64 * 0.25)
                            .sum::<f64>();
                        assert!(
                            (out.world(1).unwrap()[r] as f64 - expected).abs()
                                <= 2e-5 + 2e-5 * expected.abs()
                        );
                    }
                }
            }
            // This tests fresh consumer admission, not the frozen bridge's
            // failed raw-qpos range or independent cached-factor lifecycle.
            source.write_qpos(&vec![0.0; n * worlds]).unwrap();
            for index in 0..2 {
                let old = bits(&raw(&data[index].current));
                assert!(
                    plans[index]
                        .update(&mut data[index], &source, None)
                        .is_err()
                );
                assert_eq!(old, bits(&raw(&data[index].current)));
            }
        }
    }
    drop(session);
    assert!(
        last_snapshot
            .unwrap()
            .world(256)
            .unwrap()
            .iter()
            .all(|v| v.is_finite())
    );
    assert_eq!((batches, slots, dependencies), (72, 210080, 19));
    println!(
        "borrowed_mul_m batches={batches} slots={slots} producer_dependency_updates={dependencies} max_abs={max_abs:.17e} phase_ms={} consumer_owned_buffer_capacity_bytes={capacity} M_buffer_copies=0",
        start.elapsed().as_millis()
    );
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new borrowed source/metadata/guard/overflow publication failures; no old producer numerical reference"]
fn gpu_rejects_corrupt_live_sources_and_never_partially_publishes_results() {
    let session = TransferSession::new(0).unwrap();
    let i = input(3, true);
    let producer = MassMatrixPlan::new(&session, model(&i)).unwrap();
    let mut source = producer.create_data(257).unwrap();
    producer.update(&mut source).unwrap();
    let source_original = raw(source.matrix_input().unwrap());
    let mut faults = 0;
    for format in [MassMultiplyFormat::Sparse, MassMultiplyFormat::Dense] {
        let plan = MassMatrixMultiplyPlan::new(&producer, i.clone(), format).unwrap();
        let mut d = plan.create_data(257).unwrap();
        d.write_vector(&vec![0.5; 771]).unwrap();
        drop(plan.update(&mut d, &source, None).unwrap());
        for field in 0..4 {
            for edge in 0..8 {
                let b = match field {
                    0 => &mut d.vector,
                    1 => &mut d.current,
                    2 => &mut d.staged,
                    _ => &mut d.status,
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[77.0]).unwrap();
                let before = bits(&raw(&d.current));
                assert!(plan.update(&mut d, &source, None).is_err());
                assert_eq!(before, bits(&raw(&d.current)));
                let b = match field {
                    0 => &mut d.vector,
                    1 => &mut d.current,
                    2 => &mut d.staged,
                    _ => &mut d.status,
                };
                b.write_range(index, &[-131072.0]).unwrap();
                drop(plan.update(&mut d, &source, None).unwrap());
                faults += 1;
            }
        }
        for index in [
            0,
            3,
            d.metadata.len() - 4,
            d.metadata.len() - 1,
            4,
            5,
            6,
            7,
            6 + 3 + 1,
        ] {
            let mut old = [0];
            d.metadata.read_range_into(index, &mut old).unwrap();
            d.metadata.write_range(index, &[i32::MAX]).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update(&mut d, &source, None).is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            d.metadata.write_range(index, &old).unwrap();
            drop(plan.update(&mut d, &source, None).unwrap());
            faults += 1;
        }
        for index in [
            0,
            3,
            source_original.len() - 4,
            source_original.len() - 1,
            4,
            4 + 40,
            4 + 256 * 49 + 40,
        ] {
            let bad = if index < 4 || index >= source_original.len() - 4 {
                0.0
            } else {
                f32::NAN
            };
            source.corrupt_output_for_test(index, bad).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update(&mut d, &source, None).is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            source
                .corrupt_output_for_test(index, source_original[index])
                .unwrap();
            drop(plan.update(&mut d, &source, None).unwrap());
            faults += 1;
        }
        d.vector.write_range(4, &[f32::NAN]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d, &source, None).is_err());
        let mut skip = vec![false; 257];
        skip[0] = true;
        let out = plan
            .update(&mut d, &source, Some(&skip))
            .unwrap()
            .snapshot()
            .unwrap();
        assert_eq!(&before[4..7], &bits(out.world(0).unwrap()));
        d.vector.write_range(4, &[0.5]).unwrap();
        source
            .corrupt_output_for_test(4 + 40, f32::INFINITY)
            .unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d, &source, None).is_err());
        drop(plan.update(&mut d, &source, Some(&skip)).unwrap());
        assert_eq!(&before[4..7], &bits(&raw(&d.current))[4..7]);
        source
            .corrupt_output_for_test(4 + 40, source_original[4 + 40])
            .unwrap();
        d.write_world_vector(256, &[f32::MAX; 3]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d, &source, None).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        d.write_world_vector(256, &[0.5; 3]).unwrap();
        drop(plan.update(&mut d, &source, None).unwrap());
        faults += 3;
    }
    assert_eq!(faults, 102);
    println!(
        "borrowed_mul_m_private faults={faults} formats=2 producer_dependency_updates=1 no_partial_publication=true recovered=true"
    );
}
