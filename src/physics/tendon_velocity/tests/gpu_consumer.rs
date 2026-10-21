//! 肌腱速度GPU真实雅可比消费测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new actual J device consumer, not old tendon/G01 acceptance"]
fn gpu_connects_real_mixed_jacobian_to_owned_velocities() {
    let session = TransferSession::new(0).unwrap();
    let timer = std::time::Instant::now();
    let (mut batches, mut slots, mut capacity) = (0, 0, 0);
    for nv in [3, 33, 65] {
        let source_plan = producer(&session, nv);
        let plan = TendonVelocityPlan::new(&source_plan).unwrap();
        for worlds in [1, 2, 257] {
            let mut source = source_plan.create_data(worlds).unwrap();
            let mut data = plan.create_data(worlds).unwrap();
            let mut last_snapshot = None;
            assert!(plan.update(&source, &mut data).is_err());
            for pose in 0..2 {
                let q: Vec<f32> = (0..nv * worlds)
                    .map(|i| ((i / nv + pose) % 3) as f32 / 4.0)
                    .collect();
                source.write_qpos(&q).unwrap();
                assert!(plan.update(&source, &mut data).is_err());
                source_plan.update(&mut source).unwrap();
                let j = raw(source_plan.tendon_velocity_source(&source).unwrap());
                for w in 0..worlds {
                    for i in 0..2 * nv + 2 {
                        let e = if i < nv {
                            match i {
                                0 => -1.0,
                                1 => 1.0,
                                _ => 0.0,
                            }
                        } else if i < 2 * nv {
                            coefficient(i - nv)
                        } else if i == 2 * nv {
                            0.0
                        } else {
                            1.0
                        };
                        assert_eq!(j[4 + w * plan.spec.stride + 5 + i], e);
                    }
                }
                for vector in 0..2 {
                    let v: Vec<f32> = (0..nv * worlds)
                        .map(|i| {
                            if vector == 1 && i < nv {
                                -0.0
                            } else {
                                ((i + pose + vector) % 9) as f32 / 8.0 - 0.5
                            }
                        })
                        .collect();
                    data.write_qvel(&v).unwrap();
                    assert!(!data.ready);
                    let snapshot = plan.update(&source, &mut data).unwrap().snapshot().unwrap();
                    for w in 0..worlds {
                        assert_eq!(
                            bits(snapshot.world(w).unwrap()),
                            bits(&expected(&v[w * nv..(w + 1) * nv]))
                        );
                        slots += 5;
                    }
                    let state = bits(&raw(&data.qvel));
                    let result = bits(&raw(&data.current));
                    let mut bad = v.clone();
                    bad[0] = f32::NAN;
                    assert!(data.write_qvel(&bad).is_err());
                    bad[0] = f32::INFINITY;
                    assert!(data.write_qvel(&bad).is_err());
                    assert!(data.write_world_qvel(worlds, &v[..nv]).is_err());
                    assert!(data.write_qvel(&v[..nv - 1]).is_err());
                    assert!(data.ready);
                    assert_eq!(state, bits(&raw(&data.qvel)));
                    assert_eq!(result, bits(&raw(&data.current)));
                    assert_eq!(snapshot.worlds(), worlds);
                    batches += 1;
                    last_snapshot = Some(snapshot);
                }
            }
            if nv == 3 && worlds == 2 {
                let old = bits(&raw(&data.current));
                let foreign_plan = producer(&session, nv);
                let foreign = foreign_plan.create_data(worlds).unwrap();
                assert!(matches!(
                    plan.update(&foreign, &mut data),
                    Err(TransferError::ModelMismatch)
                ));
                let wrong = source_plan.create_data(worlds + 1).unwrap();
                assert!(matches!(
                    plan.update(&wrong, &mut data),
                    Err(TransferError::ModelMismatch)
                ));
                let other = TendonVelocityPlan::new(&source_plan).unwrap();
                let mut other_data = other.create_data(worlds).unwrap();
                assert!(matches!(
                    plan.update(&source, &mut other_data),
                    Err(TransferError::ModelMismatch)
                ));
                assert!(data.ready);
                assert_eq!(old, bits(&raw(&data.current)));
                let history = last_snapshot.as_ref().unwrap();
                let historical_bits = bits(history.world(1).unwrap());
                data.write_world_qvel(1, &[0.25; 3]).unwrap();
                assert!(!data.ready);
                let s = plan.update(&source, &mut data).unwrap().snapshot().unwrap();
                assert_eq!(bits(s.world(1).unwrap()), bits(&expected(&[0.25; 3])));
                assert_eq!(historical_bits, bits(history.world(1).unwrap()));
            }
            capacity = capacity.max(
                4 * (plan.metadata.len()
                    + data.qvel.len()
                    + data.current.len()
                    + data.staged.len()
                    + data.status.len()),
            );
        }
    }
    assert_eq!((batches, slots, capacity), (36, 15600, 78856));
    println!(
        "tendon_velocity batches={batches} slots={slots} bit_difference=0 source_updates=18 extra_world_write_consumer=1 phase_ms={} owned_capacity_bytes={capacity}",
        timer.elapsed().as_millis()
    );
}
