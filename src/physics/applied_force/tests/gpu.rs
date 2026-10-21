//! 施加外力GPU子树与故障测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new U075 projection/add/replace, independent lever reference and exact dyadic coverage"]
fn gpu_projects_subtrees_and_repeatedly_accumulates_current_force_without_old_producers() {
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0;
    let mut slots = 0;
    let mut delta_slots = 0;
    let mut capacity = 0;
    let v = reference();
    // Fixed expected bits independently certify sign, component and root use.
    let plan = AppliedForcePlan::new(
        &session,
        ints(&v, "parents"),
        ints(&v, "roots"),
        ints(&v, "dof_bodyid"),
    )
    .unwrap();
    let mut d = plan.create_data(1).unwrap();
    for (field, key) in [
        (AppliedForceGeometry::Xipos, "xipos"),
        (AppliedForceGeometry::SubtreeCom, "subtree_com"),
        (AppliedForceGeometry::Cdof, "cdof"),
    ] {
        d.write_geometry(field, &floats(&v, key)).unwrap();
    }
    d.write_ft(&floats(&v, "ft_force_torque")).unwrap();
    d.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
    plan.accumulate(&mut d).unwrap();
    let added = d.readback().unwrap();
    for (i, &x) in added.world(0).unwrap().iter().enumerate() {
        assert_eq!(
            x.to_bits(),
            u32::from_str_radix(v["added_bits"][i].as_str().unwrap(), 16).unwrap()
        );
    }
    plan.replace(&mut d).unwrap();
    for (i, &x) in d.readback().unwrap().world(0).unwrap().iter().enumerate() {
        assert_eq!(
            x.to_bits(),
            u32::from_str_radix(v["delta_bits"][i].as_str().unwrap(), 16).unwrap()
        );
    }
    plan.accumulate(&mut d).unwrap();
    for (&x, &a) in d
        .readback()
        .unwrap()
        .world(0)
        .unwrap()
        .iter()
        .zip([4.0, -12.0, -40.0, 0.0].iter())
    {
        assert_eq!(x, 2.0 * a);
    }
    for nb in [8, 273] {
        for n in [1, 65, 513] {
            let t = model(nb, n);
            let plan =
                AppliedForcePlan::new(&session, t.parent.clone(), t.root.clone(), t.body.clone())
                    .unwrap();
            for sample in 0..2 {
                for worlds in [1, 2, 257] {
                    let mut d = plan.create_data(worlds).unwrap();
                    assert!(d.readback().is_err());
                    let expected = fill(&plan, &mut d, sample);
                    for add in [false, true] {
                        let seed = (0..worlds)
                            .flat_map(|w| scene(&t, w, sample).4)
                            .collect::<Vec<_>>();
                        d.write_qfrc(&seed).unwrap();
                        if add {
                            plan.accumulate(&mut d).unwrap();
                        } else {
                            plan.replace(&mut d).unwrap();
                        }
                        let out = d.readback().unwrap();
                        let delta = raw(&d.delta);
                        for w in 0..worlds {
                            for i in 0..n {
                                let e = expected[w][i];
                                assert_eq!(
                                    f64::from(e as f32),
                                    e,
                                    "dyadic reference must be exactly representable"
                                );
                                assert_eq!(delta[4 + w * n + i].to_bits(), (e as f32).to_bits());
                                delta_slots += 1;
                                let final_e = if add {
                                    f64::from(seed[w * n + i]) + e
                                } else {
                                    e
                                };
                                assert_eq!(f64::from(final_e as f32), final_e);
                                assert_eq!(
                                    out.world(w).unwrap()[i].to_bits(),
                                    (final_e as f32).to_bits()
                                );
                                slots += 1;
                            }
                        }
                        assert!(out.world(worlds).is_err());
                        capacity = capacity.max(
                            4 * (plan.metadata.len()
                                + d.geometry.len()
                                + d.ft.len()
                                + d.current.len()
                                + d.delta.len()
                                + d.staged.len()
                                + d.status.len()),
                        );
                        batches += 1;
                    }
                }
            }
        }
    }
    assert_eq!((batches, slots, delta_slots), (72, 1204320, 1204320));
    // FMA side is a compiler-order diagnostic, not frozen Warp acceptance.
    let c = &v["fma_control"];
    let plan = AppliedForcePlan::new(
        &session,
        ints(c, "parents"),
        ints(c, "roots"),
        ints(c, "dof_bodyid"),
    )
    .unwrap();
    let mut d = plan.create_data(1).unwrap();
    let from_bits = |key: &str| {
        c[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| f32::from_bits(u32::from_str_radix(v.as_str().unwrap(), 16).unwrap()))
            .collect::<Vec<_>>()
    };
    d.write_geometry(AppliedForceGeometry::Xipos, &from_bits("xipos_bits"))
        .unwrap();
    d.write_geometry(
        AppliedForceGeometry::SubtreeCom,
        &from_bits("subtree_com_bits"),
    )
    .unwrap();
    d.write_geometry(AppliedForceGeometry::Cdof, &floats(c, "cdof"))
        .unwrap();
    d.write_ft(&from_bits("ft_bits")).unwrap();
    plan.replace(&mut d).unwrap();
    let fma = d.readback().unwrap().world(0).unwrap()[0];
    let fma_expected = c["independent_f64_delta"].as_f64().unwrap();
    assert!((f64::from(fma) - fma_expected).abs() <= 2e-7 + 2e-7 * fma_expected.abs());
    assert!(
        c["compiler_order_diagnostic_bits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| u32::from_str_radix(b.as_str().unwrap(), 16).unwrap() == fma.to_bits())
    );
    // New geometry/FT/current-force lifecycle, no old qpos/RHS stage calls.
    let t = model(8, 65);
    let plan =
        AppliedForcePlan::new(&session, t.parent.clone(), t.root.clone(), t.body.clone()).unwrap();
    let other =
        AppliedForcePlan::new(&session, t.parent.clone(), t.root.clone(), t.body.clone()).unwrap();
    let mut d = plan.create_data(257).unwrap();
    let expected = fill(&plan, &mut d, 0);
    plan.replace(&mut d).unwrap();
    let before = d.readback().unwrap();
    plan.accumulate(&mut d).unwrap();
    // 同一下标同时读取回读与参考。
    #[allow(clippy::needless_range_loop)]
    for w in 0..257 {
        for (i, &value) in d.readback().unwrap().world(w).unwrap().iter().enumerate() {
            assert_eq!(value.to_bits(), (2.0 * expected[w][i] as f32).to_bits());
        }
    }
    plan.replace(&mut d).unwrap();
    assert_eq!(bits(&before.values), bits(&d.readback().unwrap().values));
    let unchanged = tokens(&d);
    assert!(other.accumulate(&mut d).is_err());
    assert_eq!(tokens(&d), unchanged);
    assert!(d.readback().is_ok());
    for field in [
        AppliedForceGeometry::Xipos,
        AppliedForceGeometry::SubtreeCom,
        AppliedForceGeometry::Cdof,
    ] {
        let width = d.layout.geometry[field.index()].elements_per_world();
        assert!(
            d.write_world_geometry(field, 257, &vec![0.0; width])
                .is_err()
        );
        assert!(
            d.write_world_geometry(field, 0, &vec![0.0; width - 1])
                .is_err()
        );
        let mut nan = vec![0.0; width];
        nan[0] = f32::NAN;
        assert!(d.write_world_geometry(field, 0, &nan).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
    }
    assert!(d.write_world_ft(257, &[0.0; 48]).is_err());
    assert!(d.write_world_ft(0, &[0.0; 47]).is_err());
    assert!(d.write_world_qfrc(257, &[0.0; 65]).is_err());
    assert!(d.write_world_qfrc(0, &[0.0; 64]).is_err());
    assert!(d.write_ft(&[f32::INFINITY]).is_err());
    assert_eq!(tokens(&d), unchanged);
    assert!(d.readback().is_ok());
    for field in [
        AppliedForceGeometry::Xipos,
        AppliedForceGeometry::SubtreeCom,
        AppliedForceGeometry::Cdof,
    ] {
        let old = raw(&d.geometry);
        let l = d.layout.geometry[field.index()];
        let r = l.world_elements(256).unwrap();
        let a = 4 + d.layout.offsets[field.index()] + r.start;
        let changed = old[a..a + r.len()]
            .iter()
            .map(|v| v + 0.125)
            .collect::<Vec<_>>();
        d.write_world_geometry(field, 256, &changed).unwrap();
        let now = raw(&d.geometry);
        for i in (0..a).chain(a + r.len()..old.len()) {
            assert_eq!(old[i].to_bits(), now[i].to_bits());
        }
        assert_eq!(bits(&now[a..a + r.len()]), bits(&changed));
        assert!(d.readback().is_err());
        d.write_world_geometry(field, 256, &old[a..a + r.len()])
            .unwrap();
        plan.replace(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&before.values));
    }
    let old = raw(&d.ft);
    let a = 4 + 256 * 48;
    d.write_world_ft(256, &[-0.0; 48]).unwrap();
    let now = raw(&d.ft);
    for i in (0..a).chain(a + 48..old.len()) {
        assert_eq!(old[i].to_bits(), now[i].to_bits());
    }
    assert!(d.readback().is_err());
    plan.replace(&mut d).unwrap();
    assert!(
        d.readback()
            .unwrap()
            .world(256)
            .unwrap()
            .iter()
            .all(|v| v.to_bits() == 0)
    );
    for w in 0..256 {
        assert_eq!(
            bits(d.readback().unwrap().world(w).unwrap()),
            bits(before.world(w).unwrap())
        );
    }
    let old = raw(&d.current);
    let a = 4 + 256 * 65;
    d.write_world_qfrc(256, &vec![7.0; 65]).unwrap();
    let now = raw(&d.current);
    for i in (0..a).chain(a + 65..old.len()) {
        assert_eq!(old[i].to_bits(), now[i].to_bits());
    }
    assert!(d.readback().is_err());
    plan.accumulate(&mut d).unwrap();
    let snapshot = d.readback().unwrap();
    assert!(snapshot.world(256).unwrap().iter().all(|&v| v == 7.0));
    drop(d);
    drop(plan);
    drop(other);
    drop(session);
    assert!(snapshot.world(256).unwrap().iter().all(|&v| v == 7.0));
    let report = serde_json::json!({"batches":batches,"qfrc_slots":slots,"delta_slots":delta_slots,"dyadic_bit_differences":0,"fma_control_bits":format!("{:08x}",fma.to_bits()),"fma_f64_error":(f64::from(fma)-fma_expected).abs(),"phase_ms":start.elapsed().as_millis(),"max_owned_plan_Data_bytes":capacity,"full_U075_NP":false});
    std::fs::write(
        std::env::var_os("MJWARP_APPLIED_FORCE_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new U075 guard/index/finite failures must preserve current qfrc before atomic publication"]
fn gpu_rejects_faults_without_partial_current_mutation_and_replace_ignores_old_nan_payload() {
    let session = TransferSession::new(0).unwrap();
    let mut plan = AppliedForcePlan::new(&session, vec![0, 0], vec![0, 1], vec![1]).unwrap();
    let mut d = plan.create_data(2).unwrap();
    d.write_geometry(
        AppliedForceGeometry::Cdof,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0].repeat(2),
    )
    .unwrap();
    d.write_ft(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0].repeat(2))
        .unwrap();
    d.write_qfrc(&[3.0, 4.0]).unwrap();
    plan.accumulate(&mut d).unwrap();
    assert_eq!(d.readback().unwrap().world(0).unwrap(), &[5.0]);
    for (kind, offset, bad) in [
        (0, 0, 0.0),
        (0, 40, 0.0),
        (0, 4, f32::NAN),
        (1, 0, 0.0),
        (1, 28, 0.0),
        (1, 10, f32::INFINITY),
        (2, 0, 0.0),
        (2, 6, 0.0),
        (2, 4, f32::NAN),
        (3, 0, 0.0),
        (3, 6, 0.0),
        (4, 0, 0.0),
        (4, 6, 0.0),
        (5, 0, 0.0),
        (5, 6, 0.0),
    ] {
        let buffer = match kind {
            0 => &mut d.geometry,
            1 => &mut d.ft,
            2 => &mut d.current,
            3 => &mut d.delta,
            4 => &mut d.staged,
            _ => &mut d.status,
        };
        let mut saved = [0.0];
        buffer.read_range_into(offset, &mut saved).unwrap();
        buffer.write_range(offset, &[bad]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.accumulate(&mut d).is_err());
        assert_eq!(bits(&raw(&d.current)), before);
        assert!(d.readback().is_err());
        let buffer = match kind {
            0 => &mut d.geometry,
            1 => &mut d.ft,
            2 => &mut d.current,
            3 => &mut d.delta,
            4 => &mut d.staged,
            _ => &mut d.status,
        };
        buffer.write_range(offset, &saved).unwrap();
        d.write_qfrc(&[3.0, 4.0]).unwrap();
        plan.accumulate(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap(), &[5.0]);
    }
    for (offset, bad) in [
        (0, 0),
        (9, 0),
        (5, 1),
        (5, -1),
        (7, 0),
        (7, 2),
        (8, -1),
        (8, 2),
    ] {
        let mut saved = [0];
        plan.metadata
            .as_buffer()
            .read_range_into(offset, &mut saved)
            .unwrap();
        plan.metadata.write_for_test(offset, bad).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.accumulate(&mut d).is_err());
        assert_eq!(bits(&raw(&d.current)), before);
        assert!(d.readback().is_err());
        plan.metadata.restore_for_test().unwrap();
        d.write_qfrc(&[3.0, 4.0]).unwrap();
        plan.accumulate(&mut d).unwrap();
    }
    d.current.write_range(4, &[f32::NAN]).unwrap();
    plan.replace(&mut d).unwrap();
    assert_eq!(d.readback().unwrap().world(0).unwrap(), &[2.0]);
    d.write_geometry(
        AppliedForceGeometry::Cdof,
        &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0].repeat(2),
    )
    .unwrap();
    d.write_ft(
        &[
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ]
        .repeat(2),
    )
    .unwrap();
    let before = bits(&raw(&d.current));
    assert!(plan.accumulate(&mut d).is_err());
    assert_eq!(bits(&raw(&d.current)), before);
    assert!(d.readback().is_err());
    d.write_geometry(
        AppliedForceGeometry::Cdof,
        &[0.0, 0.0, 0.0, 0.5, 0.0, 0.0].repeat(2),
    )
    .unwrap();
    d.write_qfrc(&[f32::MAX; 2]).unwrap();
    let before = bits(&raw(&d.current));
    assert!(plan.accumulate(&mut d).is_err());
    assert_eq!(bits(&raw(&d.current)), before);
    assert!(d.readback().is_err());
    plan.replace(&mut d).unwrap();
    assert_eq!(d.readback().unwrap().world(0).unwrap(), &[f32::MAX * 0.5]);
    let _ = raw_meta(plan.metadata.as_buffer());
}
