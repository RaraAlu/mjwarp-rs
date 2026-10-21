use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new device-only liveM->Mqvel->energy connection, independent onehot exact bits; no old selector replay"]
fn gpu_links_live_matrix_energy_with_exact_one_hot_and_device_pair_bits() {
    use super::super::super::tests::{input, model};
    use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut sources = 0;
    let mut products = 0;
    let mut batches = 0;
    let mut worlds_checked = 0;
    let mut pair_slots = 0;
    let mut capacity = 0;
    let mut last = None;
    for n in [3, 33, 65] {
        let i = input(n, true);
        let producer = MassMatrixPlan::new(&session, model(&i)).unwrap();
        let multiply = [
            MassMatrixMultiplyPlan::new(&producer, i.clone(), MassMultiplyFormat::Sparse).unwrap(),
            MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Dense).unwrap(),
        ];
        let plans = [
            MassMatrixEnergyPlan::new(&multiply[0]).unwrap(),
            MassMatrixEnergyPlan::new(&multiply[1]).unwrap(),
        ];
        for worlds in [1, 2, 257] {
            let mut source = producer.create_data(worlds).unwrap();
            let mut md = [
                multiply[0].create_data(worlds).unwrap(),
                multiply[1].create_data(worlds).unwrap(),
            ];
            let mut ed = [
                plans[0].create_data(worlds).unwrap(),
                plans[1].create_data(worlds).unwrap(),
            ];
            for state in 0..2 {
                let pose: Vec<f32> = (0..worlds * n)
                    .map(|j| ((j + state) % 9) as f32 * 0.07 - 0.28)
                    .collect();
                source.write_qpos(&pose).unwrap();
                producer.update(&mut source).unwrap();
                sources += 1;
                let matrix = source.readback().unwrap();
                if n == 3 && worlds == 2 && state == 0 {
                    for f in 0..2 {
                        let view = multiply[f]
                            .update(&mut md[f], &source, Some(&[true, true]))
                            .unwrap();
                        products += 1;
                        let before = bits(&raw(&ed[f].current));
                        assert!(plans[f].update(&mut ed[f], &view).is_err());
                        assert_eq!(before, bits(&raw(&ed[f].current)));
                    }
                }
                for vector in 0..2 {
                    let mut q = vec![0.0; worlds * n];
                    let hot: Vec<usize> = (0..worlds).map(|w| (w + vector + state) % n).collect();
                    if vector == 1 {
                        for (w, &h) in hot.iter().enumerate() {
                            assert_ne!(h, (w + state) % n);
                        }
                    }
                    for w in 0..worlds {
                        q[w * n + hot[w]] = 1.0;
                    }
                    let p: Vec<f32> = (0..worlds)
                        .map(|w| {
                            if (w + state) % 7 == 0 {
                                -0.0
                            } else {
                                ((w + vector + state) % 19) as f32 * 0.25
                            }
                        })
                        .collect();
                    for f in 0..2 {
                        md[f].write_vector(&q).unwrap();
                        let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                        products += 1;
                        ed[f].write_potential(&p).unwrap();
                        let energy = plans[f].update(&mut ed[f], &mv).unwrap();
                        assert_eq!(energy.worlds(), worlds);
                        let out = energy.snapshot().unwrap();
                        assert_eq!(out.worlds(), worlds);
                        assert!(out.world(worlds).is_err());
                        let packed = raw(&energy.data.pair);
                        let mq = raw(&mv.data.current);
                        let srcq = raw(&mv.data.vector);
                        for w in 0..worlds {
                            assert_eq!(
                                bits(&packed[4 + 2 * w * n..4 + (2 * w + 1) * n]),
                                bits(&srcq[4 + w * n..4 + (w + 1) * n])
                            );
                            assert_eq!(
                                bits(&packed[4 + (2 * w + 1) * n..4 + 2 * (w + 1) * n]),
                                bits(&mq[4 + w * n..4 + (w + 1) * n])
                            );
                            pair_slots += 2 * n;
                            let diagonal = matrix.world(w).unwrap().matrix[hot[w] * n + hot[w]];
                            let word = diagonal.to_bits();
                            assert!(
                                diagonal > 0.0 && diagonal.is_normal() && ((word >> 23) & 255) > 1
                            );
                            // Independent binary-exponent certificate for
                            // onehot q: half the source diagonal, no energy
                            // candidate or Mqvel feeds this expected word.
                            let expected = word - 0x0080_0000;
                            assert_eq!(out.world(w).unwrap()[1].to_bits(), expected);
                            assert_eq!(out.world(w).unwrap()[0].to_bits(), p[w].to_bits());
                        }
                        let _ = energy;
                        if n == 3 && worlds == 2 && state == 0 && vector == 0 {
                            let mut wrong = plans[f].create_data(1).unwrap();
                            assert!(plans[f].update(&mut wrong, &mv).is_err());
                            let foreign = MassMatrixEnergyPlan::new(&multiply[f]).unwrap();
                            let before = bits(&raw(&ed[f].current));
                            assert!(foreign.update(&mut ed[f], &mv).is_err());
                            assert_eq!(before, bits(&raw(&ed[f].current)));
                            if f == 1 {
                                let before = bits(&raw(&ed[0].current));
                                assert!(plans[0].update(&mut ed[0], &mv).is_err());
                                assert_eq!(before, bits(&raw(&ed[0].current)));
                            }
                        }
                        capacity = capacity.max(
                            4 * (ed[f].pair.len()
                                + ed[f].current.len()
                                + ed[f].staged.len()
                                + ed[f].status.len()
                                + plans[f].metadata.len()),
                        );
                        last = Some(out);
                        batches += 1;
                        worlds_checked += worlds;
                    }
                }
            }
            if n == 3 && worlds == 2 {
                for f in 0..2 {
                    md[f].write_world_vector(1, &[0.25; 3]).unwrap();
                    let mv = multiply[f]
                        .update(&mut md[f], &source, Some(&[true, false]))
                        .unwrap();
                    products += 1;
                    let before = bits(&raw(&ed[f].current));
                    assert!(plans[f].update(&mut ed[f], &mv).is_err());
                    assert_eq!(before, bits(&raw(&ed[f].current)));
                    drop(mv);
                    md[f].write_vector(&[-0.0; 6]).unwrap();
                    let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                    products += 1;
                    let out = plans[f]
                        .update(&mut ed[f], &mv)
                        .unwrap()
                        .snapshot()
                        .unwrap();
                    for w in 0..2 {
                        assert_eq!(out.world(w).unwrap()[1].to_bits(), 0x80000000);
                    }
                }
            }
        }
    }
    drop(session);
    assert!(
        last.unwrap()
            .world(256)
            .unwrap()
            .iter()
            .all(|v| v.is_finite())
    );
    assert_eq!(
        (batches, worlds_checked, sources, products),
        (72, 6240, 18, 78)
    );
    assert_eq!(pair_slots, 420160);
    assert_eq!(capacity, 138912);
    println!(
        "device_energy batches={batches} worlds={worlds_checked} bit_difference=0 pair_bit_slots={pair_slots} negative_zero_worlds=4 source_dependency_updates={sources} Mqvel_dependency_updates={products} phase_ms={} energy_owned_capacity_bytes={capacity} pair_D2D_copies_per_update=2W M_host_feedback=0",
        start.elapsed().as_millis()
    );
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new device-energy private raw-source/pair/output guards and finite/overflow transactional faults"]
fn gpu_rejects_corrupted_device_inputs_and_stale_source_without_partial_energy() {
    use super::super::super::tests::{input, model};
    use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
    let s = TransferSession::new(0).unwrap();
    let i = input(3, true);
    let producer = MassMatrixPlan::new(&s, model(&i)).unwrap();
    let mut source = producer.create_data(2).unwrap();
    producer.update(&mut source).unwrap();
    let multiply = MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Sparse).unwrap();
    let mut md = multiply.create_data(2).unwrap();
    md.write_vector(&[0.5; 6]).unwrap();
    drop(multiply.update(&mut md, &source, None).unwrap());
    let mut plan = MassMatrixEnergyPlan::new(&multiply).unwrap();
    let mut data = plan.create_data(2).unwrap();
    // Test-only forged tickets let fault injection emulate corrupted GPU
    // buffers without rerunning a frozen producer or claiming that public
    // callers can construct/forge the sealed current-view provenance.
    fn ticket<'a>(
        source: &'a crate::physics::MassMatrixData,
        data: &'a super::super::super::MassMatrixMultiplyData,
    ) -> MassMatrixMultiplyView<'a> {
        MassMatrixMultiplyView {
            source,
            data,
            computed: vec![true; 2],
        }
    }
    let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
    let mut faults = 0;
    for field in 0..4 {
        for edge in 0..8 {
            let b = match field {
                0 => &mut data.pair,
                1 => &mut data.current,
                2 => &mut data.staged,
                _ => &mut data.status,
            };
            let index = if edge < 4 { edge } else { b.len() - 8 + edge };
            b.write_range(index, &[99.0]).unwrap();
            let before = bits(&raw(&data.current));
            assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            let b = match field {
                0 => &mut data.pair,
                1 => &mut data.current,
                2 => &mut data.staged,
                _ => &mut data.status,
            };
            b.write_range(index, &[-131072.0]).unwrap();
            let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
            faults += 1;
        }
    }
    for field in 0..2 {
        for edge in 0..8 {
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            let index = if edge < 4 { edge } else { b.len() - 8 + edge };
            b.write_range(index, &[99.0]).unwrap();
            let before = bits(&raw(&data.current));
            assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            b.write_range(index, &[-131072.0]).unwrap();
            let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
            faults += 1;
        }
    }
    for (field, index, bad) in [
        (0, 4, f32::NAN),
        (0, 9, f32::INFINITY),
        (1, 4, f32::NAN),
        (1, 9, f32::INFINITY),
    ] {
        let b = if field == 0 {
            &mut md.vector
        } else {
            &mut md.current
        };
        let old = raw(b)[index];
        b.write_range(index, &[bad]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        let b = if field == 0 {
            &mut md.vector
        } else {
            &mut md.current
        };
        b.write_range(index, &[old]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        faults += 1;
    }
    plan.metadata.write_for_test(0).unwrap();
    let before = bits(&raw(&data.current));
    assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    plan.metadata.restore_for_test().unwrap();
    let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
    faults += 1;
    let old = raw(&data.current)[4];
    data.current.write_range(4, &[f32::NAN]).unwrap();
    let before = bits(&raw(&data.current));
    assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    data.current.write_range(4, &[old]).unwrap();
    faults += 1;
    data.current.write_range(5, &[f32::NAN]).unwrap();
    data.pair.write_range(4, &[f32::NAN]).unwrap();
    let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
    let q = raw(&md.vector);
    let mq = raw(&md.current);
    md.vector.write_range(7, &[f32::MAX]).unwrap();
    md.current.write_range(7, &[f32::MAX]).unwrap();
    let before = bits(&raw(&data.current));
    assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    md.vector.write_range(7, &q[7..8]).unwrap();
    md.current.write_range(7, &mq[7..8]).unwrap();
    let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
    faults += 1;
    let before = bits(&raw(&data.current));
    assert!(data.write_potential(&[f32::NAN; 2]).is_err());
    assert!(data.write_world_potential(2, 1.0).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    source.write_qpos(&[0.0; 6]).unwrap();
    let stale = MassMatrixMultiplyView {
        source: &source,
        data: &md,
        computed: vec![true; 2],
    };
    assert!(plan.update(&mut data, &stale).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    faults += 1;
    assert_eq!(faults, 56);
    println!(
        "device_energy_private faults={faults} source_dependency_updates=1 Mqvel_dependency_updates=1 no_partial_publication=true cold_source_rejected=true"
    );
}
