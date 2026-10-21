//! 平滑力合并与守卫测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new U016 four-force merge, independent scalar bits, sleep/island and owned snapshots"]
fn gpu_merges_exact_f32_steps_and_zeroes_sleeping_dofs_in_all_option_combinations() {
    let v = reference();
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0usize;
    let mut slots = 0usize;
    let mut asleep = 0usize;
    let mut static_active = 0usize;
    let mut negative_zero = 0usize;
    let mut capacity = 0usize;
    let mut seen = [0usize; 18];
    for n in [1, 65, 513] {
        let input = model(n);
        for (sleep, island) in [(false, false), (false, true), (true, false), (true, true)] {
            let options = SmoothForceOptions {
                sleep_enabled: sleep,
                island_disabled: island,
            };
            let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
            assert_eq!(plan.options(), options);
            assert_eq!(plan.input(), &input);
            for sample in 0..2 {
                for worlds in [1, 2, 257] {
                    let mut d = plan.create_data(worlds).unwrap();
                    assert!(d.readback().is_err());
                    let mut values: [Vec<f32>; 4] =
                        std::array::from_fn(|_| Vec::with_capacity(worlds * n));
                    for w in 0..worlds {
                        for i in 0..n {
                            let (x, _) = row(&v, (i + 3 * w + 5 * sample) % 18);
                            for f in 0..4 {
                                values[f].push(x[f]);
                            }
                        }
                    }
                    let awake = (0..worlds * 129)
                        .map(|k| match (k % 129 + k / 129 + sample) % 5 {
                            0 => 0,
                            1 => -11,
                            2 => 2,
                            3 => 1,
                            _ => i32::MAX,
                        })
                        .collect::<Vec<_>>();
                    for (f, values) in FIELDS.iter().zip(&values) {
                        d.write_force(*f, values).unwrap();
                    }
                    d.write_awake(&awake).unwrap();
                    plan.update(&mut d).unwrap();
                    let out = d.readback().unwrap();
                    for w in 0..worlds {
                        for (i, &actual) in out.world(w).unwrap().iter().enumerate() {
                            let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
                            let index = (i + 3 * w + 5 * sample) % 18;
                            let sleeping = options.enable_sleep()
                                && tree >= 0
                                && awake[w * 129 + tree as usize] == 0;
                            let expected = if sleeping {
                                asleep += 1;
                                0
                            } else {
                                seen[index] += 1;
                                if tree < 0 {
                                    static_active += 1;
                                }
                                row(&v, index).1
                            };
                            assert_eq!(
                                actual.to_bits(),
                                expected,
                                "n{n}/w{w}/i{i}/sample{sample}/{options:?}"
                            );
                            if !sleeping && expected == 0x80000000 {
                                negative_zero += 1;
                            }
                            slots += 1;
                        }
                    }
                    assert!(out.world(worlds).is_err());
                    capacity = capacity.max(
                        4 * (d.forces.len() + d.output.len() + d.status.len() + d.metadata.len()),
                    );
                    batches += 1;
                }
            }
        }
    }
    assert_eq!((batches, slots), (72, 1204320));
    assert!(asleep > 0 && static_active > 0 && negative_zero > 0);
    assert!(seen.iter().all(|&c| c > 0));
    // New field/awake lifecycle only; no qpos, mass or RHS producer exists.
    let input = model(65);
    let options = SmoothForceOptions {
        sleep_enabled: true,
        island_disabled: false,
    };
    let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
    let other = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
    let mut d = plan.create_data(257).unwrap();
    for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
        d.write_force(*f, &vec![x; 257 * 65]).unwrap();
    }
    plan.update(&mut d).unwrap();
    let unchanged = tokens(&d);
    assert!(other.update(&mut d).is_err());
    assert_eq!(tokens(&d), unchanged);
    assert!(d.readback().is_ok());
    for f in FIELDS {
        for (w, length) in [(257, 65), (0, 64)] {
            assert!(d.write_world_force(f, w, &vec![1.0; length]).is_err());
            assert_eq!(tokens(&d), unchanged);
            assert!(d.readback().is_ok());
        }
        let mut bad = vec![1.0; 65];
        bad[3] = f32::NAN;
        assert!(d.write_world_force(f, 0, &bad).is_err());
        assert!(d.write_force(f, &[]).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
    }
    assert!(d.write_world_awake(257, &[1; 129]).is_err());
    assert!(d.write_world_awake(0, &[1; 128]).is_err());
    assert!(d.write_awake(&[]).is_err());
    assert_eq!(tokens(&d), unchanged);
    assert!(d.readback().is_ok());
    for (j, f) in FIELDS.iter().enumerate() {
        let before = raw_f32(&d.forces);
        let values = vec![[7.0, 1.0, 6.0, 8.0][j]; 65];
        let a = 4 + j * 257 * 65 + 256 * 65;
        d.write_world_force(*f, 256, &values).unwrap();
        let after = raw_f32(&d.forces);
        for i in (0..a).chain(a + 65..before.len()) {
            assert_eq!(before[i].to_bits(), after[i].to_bits());
        }
        assert_eq!(bits(&after[a..a + 65]), bits(&values));
        assert!(d.readback().is_err());
        plan.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..257 {
            let expected = if w == 256 {
                [12.0f32, 11.0, 13.0, 14.0][j]
            } else {
                10.0
            };
            assert!(
                out.world(w)
                    .unwrap()
                    .iter()
                    .all(|v| v.to_bits() == expected.to_bits())
            );
        }
        d.write_world_force(*f, 256, &vec![[5.0, 2.0, 3.0, 4.0][j]; 65])
            .unwrap();
        plan.update(&mut d).unwrap();
    }
    let before = raw_i32(d.metadata.as_buffer());
    let a = 4 + 273 + 65 + 256 * 129;
    d.write_world_awake(256, &[0; 129]).unwrap();
    let after = raw_i32(d.metadata.as_buffer());
    for i in (0..a).chain(a + 129..before.len()) {
        assert_eq!(before[i], after[i]);
    }
    assert!(after[a..a + 129].iter().all(|&v| v == 0));
    assert!(d.readback().is_err());
    plan.update(&mut d).unwrap();
    let sleeping = d.readback().unwrap();
    for w in 0..257 {
        for (i, &value) in sleeping.world(w).unwrap().iter().enumerate() {
            let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
            let expected = if w == 256 && tree >= 0 { 0.0f32 } else { 10.0 };
            assert_eq!(value.to_bits(), expected.to_bits());
        }
    }
    d.write_world_awake(256, &[-11; 129]).unwrap();
    plan.update(&mut d).unwrap();
    let snapshot = d.readback().unwrap();
    drop(d);
    drop(plan);
    drop(other);
    drop(session);
    assert!(
        snapshot
            .world(256)
            .unwrap()
            .iter()
            .all(|v| v.to_bits() == 10.0f32.to_bits())
    );
    // ntree0 has no awake payload; static DOFs still preserve active -0.
    let session = TransferSession::new(0).unwrap();
    let plan = SmoothForcePlan::new(
        &session,
        SmoothForceInput::new(0, vec![-1, -1], vec![1, 0]).unwrap(),
        options,
    )
    .unwrap();
    let mut d = plan.create_data(2).unwrap();
    let (x, _) = row(&v, 7);
    for (f, x) in FIELDS.iter().zip(x) {
        d.write_force(*f, &[x; 4]).unwrap();
    }
    d.write_world_awake(1, &[]).unwrap();
    plan.update(&mut d).unwrap();
    assert!(
        d.readback()
            .unwrap()
            .world(1)
            .unwrap()
            .iter()
            .all(|v| v.to_bits() == 0x80000000)
    );
    let report = serde_json::json!({"batches":batches,"slots":slots,"bit_differences":0,"asleep_positive_zero_slots":asleep,"static_active_slots":static_active,"active_negative_zero_slots":negative_zero,"active_case_coverage":seen,"phase_ms":start.elapsed().as_millis(),"owned_Data_max_bytes":capacity,"full_fwd_acceleration_NP":false});
    std::fs::write(
        std::env::var_os("MJWARP_SMOOTH_FORCE_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new smooth-force finite/overflow, mapping and full guarded-buffer rejection"]
fn gpu_rejects_guard_mapping_and_active_overflow_but_overwrites_sleeping_output_with_zero() {
    let session = TransferSession::new(0).unwrap();
    let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
    let options = SmoothForceOptions {
        sleep_enabled: true,
        island_disabled: false,
    };
    let plan = SmoothForcePlan::new(&session, input, options).unwrap();
    let mut d = plan.create_data(2).unwrap();
    for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
        d.write_force(*f, &[x; 6]).unwrap();
    }
    plan.update(&mut d).unwrap();
    let good = d.readback().unwrap();
    for (kind, offset, bad) in [
        (0, 0, 0.0),
        (0, 28, 0.0),
        (0, 4, f32::NAN),
        (0, 4 + 18 + 2, f32::INFINITY),
        (1, 0, 0.0),
        (1, 10, 0.0),
        (2, 0, 0.0),
        (2, 6, 0.0),
    ] {
        let buffer = match kind {
            0 => &mut d.forces,
            1 => &mut d.output,
            _ => &mut d.status,
        };
        let mut old = [0.0];
        buffer.read_range_into(offset, &mut old).unwrap();
        buffer.write_range(offset, &[bad]).unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        let buffer = match kind {
            0 => &mut d.forces,
            1 => &mut d.output,
            _ => &mut d.status,
        };
        buffer.write_range(offset, &old).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    }
    for (offset, bad) in [(0, 0), (14, 0), (5, -2), (5, 2), (7, -1), (7, 3)] {
        let mut old = [0];
        d.metadata
            .as_buffer()
            .read_range_into(offset, &mut old)
            .unwrap();
        d.metadata
            .corrupt_device_for_validator_test(offset, bad)
            .unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        d.metadata.restore_for_test().unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    }
    d.write_world_awake(0, &[0, 1]).unwrap();
    d.forces.write_range(4, &[f32::NAN]).unwrap();
    plan.update(&mut d).unwrap();
    let asleep = d.readback().unwrap();
    assert_eq!(asleep.world(0).unwrap()[0].to_bits(), 0);
    assert_eq!(asleep.world(0).unwrap()[1], 10.0);
    assert_eq!(asleep.world(0).unwrap()[2], 10.0);
    d.write_world_awake(0, &[2, -11]).unwrap();
    assert!(plan.update(&mut d).is_err());
    assert!(d.readback().is_err());
    d.forces.write_range(4, &[5.0]).unwrap();
    plan.update(&mut d).unwrap();
    d.write_world_force(SmoothForceField::Passive, 0, &[f32::MAX, 5.0, 5.0])
        .unwrap();
    d.write_world_force(SmoothForceField::Bias, 0, &[-f32::MAX, 2.0, 2.0])
        .unwrap();
    assert!(plan.update(&mut d).is_err());
    assert!(d.readback().is_err());
    d.write_world_awake(0, &[0, 1]).unwrap();
    plan.update(&mut d).unwrap();
    assert_eq!(d.readback().unwrap().world(0).unwrap()[0].to_bits(), 0);
    d.write_world_force(SmoothForceField::Passive, 0, &[5.0; 3])
        .unwrap();
    d.write_world_force(SmoothForceField::Bias, 0, &[2.0; 3])
        .unwrap();
    d.write_world_awake(0, &[1, 1]).unwrap();
    plan.update(&mut d).unwrap();
    assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
}
