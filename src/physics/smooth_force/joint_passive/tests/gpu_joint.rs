//! 关节被动GPU关节力测试。

#[cfg(feature = "cuda-probe")]
use super::tests_oracle::*;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new joint passive producer and explicit joint-only GPU smooth connection, no old selectors"]
fn gpu_produces_joint_forces_with_periodic_poly_only_coefficients_and_disable_flags() {
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0;
    let mut slots = 0;
    let mut capacity = 0;
    let mut last = None;
    for periods in [[1; 5], [2, 3, 5, 7, 11]] {
        for flags in 0..4 {
            let options = JointPassiveOptions {
                disable_spring: flags & 1 != 0,
                disable_damper: flags & 2 != 0,
            };
            let plan = JointPassivePlan::new(&session, model(), params(periods), options).unwrap();
            for worlds in [1, 2, 257] {
                let mut d = plan.create_data(worlds).unwrap();
                assert!(d.readback_joint().is_err());
                for state in 0..2 {
                    let q: Vec<f32> = (0..13 * worlds)
                        .map(|j| ((j + state) % 9) as f32 / 8.0 - 0.5)
                        .collect();
                    let v: Vec<f32> = (0..11 * worlds)
                        .map(|j| {
                            let x = ((j + 3 * state) % 9) as f32 / 8.0 - 0.5;
                            if x == 0.0 && j % 2 == 0 { -0.0 } else { x }
                        })
                        .collect();
                    let bias: Vec<f32> = (0..11 * worlds)
                        .map(|j| 0.125 + ((j + state) % 3) as f32 / 16.0)
                        .collect();
                    let act: Vec<f32> = (0..11 * worlds)
                        .map(|j| ((j + 2) % 5) as f32 / 16.0 - 0.125)
                        .collect();
                    let applied: Vec<f32> = (0..11 * worlds)
                        .map(|j| ((j + state) % 3) as f32 / 32.0 - 0.03125)
                        .collect();
                    let other = [
                        (SmoothForceField::Bias, &bias),
                        (SmoothForceField::Actuator, &act),
                        (SmoothForceField::Applied, &applied),
                    ];
                    for k in 0..3 {
                        if state == 1 && k == 0 {
                            assert!(d.readback_joint_only_smooth().is_ok());
                        }
                        let (field, values) = other[(k + flags as usize % 3) % 3];
                        d.write_other_force(field, values).unwrap();
                        assert!(d.readback_joint_only_smooth().is_err());
                    }
                    d.write_qpos(&q).unwrap();
                    d.write_qvel(&v).unwrap();
                    plan.update_joint_only_smooth(&mut d).unwrap();
                    let parts = d.readback_joint().unwrap();
                    let smooth = d.readback_joint_only_smooth().unwrap();
                    for w in 0..worlds {
                        let (s, da) = oracle(
                            &plan.input,
                            w,
                            &q[13 * w..13 * (w + 1)],
                            &v[11 * w..11 * (w + 1)],
                            options,
                        );
                        assert_eq!(bits(parts.spring(w).unwrap()), bits(&s));
                        assert_eq!(bits(parts.damper(w).unwrap()), bits(&da));
                        let sum: Vec<f32> = s
                            .iter()
                            .zip(&da)
                            .enumerate()
                            .map(|(r, (&a, &b))| {
                                let s = a as f64 + b as f64;
                                assert_eq!(s as f32 as f64, s);
                                let result = ((s - bias[w * 11 + r] as f64)
                                    + act[w * 11 + r] as f64)
                                    + applied[w * 11 + r] as f64;
                                assert_eq!(result as f32 as f64, result);
                                result as f32
                            })
                            .collect();
                        assert_eq!(bits(smooth.world(w).unwrap()), bits(&sum));
                        slots += 11;
                    }
                    batches += 1;
                    last = Some(parts);
                    capacity = capacity.max(
                        4 * (plan.metadata.len()
                            + plan.parameters.len()
                            + d.state.len()
                            + d.current.len()
                            + d.staged.len()
                            + d.status.len()
                            + d.smooth.metadata.len()
                            + d.smooth.forces.len()
                            + d.smooth.output.len()
                            + d.smooth.status.len()),
                    );
                }
                let before = bits(&raw(&d.current));
                assert!(d.write_world_qvel(worlds, &[0.0; 11]).is_err());
                let old_state = bits(&raw(&d.state));
                let mut bad_q = vec![0.0; 13 * worlds];
                bad_q[0] = f32::NAN;
                let mut bad_v = vec![0.0; 11 * worlds];
                bad_v[11 * worlds - 1] = f32::INFINITY;
                assert!(d.write_qpos(&bad_q).is_err());
                assert!(d.write_qvel(&bad_v).is_err());
                assert_eq!(old_state, bits(&raw(&d.state)));
                assert!(d.readback_joint().is_ok());
                assert!(d.readback_joint_only_smooth().is_ok());
                assert!(
                    d.write_other_force(SmoothForceField::Passive, &vec![0.0; 11 * worlds])
                        .is_err()
                );
                assert_eq!(before, bits(&raw(&d.current)));
                d.write_world_qvel(worlds - 1, &[0.25; 11]).unwrap();
                assert!(d.readback_joint().is_err());
                assert!(d.readback_joint_only_smooth().is_err());
                plan.update_joint(&mut d).unwrap();
            }
        }
    }
    drop(session);
    assert_eq!(last.unwrap().worlds(), 257);
    assert_eq!((batches, slots), (48, 45760));
    assert_eq!(capacity, 130648);
    println!(
        "joint_passive batches={batches} spring_slots={slots} damper_slots={slots} joint_smooth_slots={slots} bit_difference=0 smooth_dependency_updates=48 phase_ms={} owned_buffer_capacity_bytes={capacity}",
        start.elapsed().as_millis()
    );
}
