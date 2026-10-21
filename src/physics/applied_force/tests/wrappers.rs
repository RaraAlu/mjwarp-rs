//! 施加外力包装恢复测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3外力增量类型化包装、同步位型与陈旧证明恢复"]
fn gpu_m3_delta_wrapper_preserves_bits_and_recovers_stale_metadata() {
    let session = TransferSession::new(0).unwrap();
    let v = reference();
    let mut plan = AppliedForcePlan::new(
        &session,
        ints(&v, "parents"),
        ints(&v, "roots"),
        ints(&v, "dof_bodyid"),
    )
    .unwrap();
    let mut data = plan.create_data(1).unwrap();
    for (field, key) in [
        (AppliedForceGeometry::Xipos, "xipos"),
        (AppliedForceGeometry::SubtreeCom, "subtree_com"),
        (AppliedForceGeometry::Cdof, "cdof"),
    ] {
        data.write_geometry(field, &floats(&v, key)).unwrap();
    }
    data.write_ft(&floats(&v, "ft_force_torque")).unwrap();
    data.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
    plan.replace(&mut data).unwrap();
    let expected = v["delta_bits"].as_array().unwrap();
    for (actual, expected) in data
        .readback()
        .unwrap()
        .world(0)
        .unwrap()
        .iter()
        .zip(expected)
    {
        assert_eq!(
            actual.to_bits(),
            u32::from_str_radix(expected.as_str().unwrap(), 16).unwrap()
        );
    }

    let before = bits(&raw(&data.current));
    plan.metadata.write_for_test(4, 0).unwrap();
    assert!(plan.replace(&mut data).is_err());
    assert_eq!(bits(&raw(&data.current)), before);
    assert!(data.readback().is_err());
    plan.metadata.restore_for_test().unwrap();
    plan.replace(&mut data).unwrap();
    assert_eq!(bits(&raw(&data.current)), before);
    assert!(data.readback().is_ok());
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3外力验证类型化包装、设备故障拒绝与同计划恢复"]
fn gpu_m3_validate_wrapper_preserves_bits_and_recovers_faults() {
    let session = TransferSession::new(0).unwrap();
    let v = reference();
    let parents = ints(&v, "parents");
    let roots = ints(&v, "roots");
    let bodies = ints(&v, "dof_bodyid");
    let nb = parents.len();
    let metadata_len = 2 * nb + bodies.len() + 8;
    let parent_offset = 4;
    let root_offset = parent_offset + nb;
    let body_offset = root_offset + nb;
    let mut plan = AppliedForcePlan::new(&session, parents, roots, bodies).unwrap();
    let mut data = plan.create_data(1).unwrap();
    for (field, key) in [
        (AppliedForceGeometry::Xipos, "xipos"),
        (AppliedForceGeometry::SubtreeCom, "subtree_com"),
        (AppliedForceGeometry::Cdof, "cdof"),
    ] {
        data.write_geometry(field, &floats(&v, key)).unwrap();
    }
    data.write_ft(&floats(&v, "ft_force_torque")).unwrap();
    data.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
    plan.replace(&mut data).unwrap();
    let expected = v["delta_bits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::from_str_radix(value.as_str().unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);

    let before = bits(&raw(&data.current));
    plan.metadata.write_for_test(4, 0).unwrap();
    assert!(matches!(
        plan.replace(&mut data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "applied_force_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    assert_eq!(bits(&raw(&data.current)), before);
    assert!(data.readback().is_err());
    plan.metadata.restore_for_test().unwrap();
    plan.replace(&mut data).unwrap();
    assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);

    for (offset, bad) in [
        (0, 0),
        (metadata_len - 1, 0),
        (parent_offset, 1),
        (parent_offset + 1, 1),
        (root_offset, 1),
        (root_offset + 2, 2),
        (body_offset, -1),
        (body_offset, nb as i32),
    ] {
        let before = bits(&raw(&data.current));
        plan.metadata
            .corrupt_device_for_validator_test(offset, bad)
            .unwrap();
        assert!(matches!(
            plan.replace(&mut data),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "applied_force_source",
                reason: "nonfinite_guard_or_mapping",
                ..
            }))
        ));
        assert_eq!(bits(&raw(&data.current)), before);
        assert!(data.readback().is_err());
        plan.metadata.restore_for_test().unwrap();
        plan.replace(&mut data).unwrap();
        assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);
    }

    for (kind, bad) in [(0, f32::NAN), (1, f32::INFINITY)] {
        let buffer = if kind == 0 {
            &mut data.geometry
        } else {
            &mut data.ft
        };
        let mut saved = [0.0];
        buffer.read_range_into(4, &mut saved).unwrap();
        buffer.write_range(4, &[bad]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(matches!(
            plan.replace(&mut data),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "applied_force_source",
                reason: "nonfinite_guard_or_mapping",
                ..
            }))
        ));
        assert_eq!(bits(&raw(&data.current)), before);
        assert!(data.readback().is_err());
        let buffer = if kind == 0 {
            &mut data.geometry
        } else {
            &mut data.ft
        };
        buffer.write_range(4, &saved).unwrap();
        plan.replace(&mut data).unwrap();
        assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);
    }
}
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "M3外力发布类型化包装、真实累加替换、失败不交换与恢复"]
fn gpu_m3_publish_wrapper_preserves_bits_and_recovers_faults() {
    let session = TransferSession::new(0).unwrap();
    let started = std::time::Instant::now();
    let v = reference();
    let plan = AppliedForcePlan::new(
        &session,
        ints(&v, "parents"),
        ints(&v, "roots"),
        ints(&v, "dof_bodyid"),
    )
    .unwrap();
    let mut data = plan.create_data(1).unwrap();
    for (field, key) in [
        (AppliedForceGeometry::Xipos, "xipos"),
        (AppliedForceGeometry::SubtreeCom, "subtree_com"),
        (AppliedForceGeometry::Cdof, "cdof"),
    ] {
        data.write_geometry(field, &floats(&v, key)).unwrap();
    }
    data.write_ft(&floats(&v, "ft_force_torque")).unwrap();
    data.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
    data.current.write_range(4, &[f32::NAN]).unwrap();
    plan.replace(&mut data).unwrap();
    let expected = v["delta_bits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| u32::from_str_radix(value.as_str().unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);

    plan.accumulate(&mut data).unwrap();
    for (&actual, &single) in data
        .readback()
        .unwrap()
        .world(0)
        .unwrap()
        .iter()
        .zip(&expected)
    {
        assert_eq!(actual.to_bits(), (2.0 * f32::from_bits(single)).to_bits());
    }

    for kind in 0..3 {
        let (offset, saved) = match kind {
            0 => {
                let mut saved = [0.0];
                data.current.read_range_into(0, &mut saved).unwrap();
                data.current.write_range(0, &[0.0]).unwrap();
                (0, saved[0])
            }
            1 => {
                let mut saved = [0.0];
                data.delta.read_range_into(0, &mut saved).unwrap();
                data.delta.write_range(0, &[0.0]).unwrap();
                (0, saved[0])
            }
            _ => {
                let offset = data.staged.len() - 1;
                let mut saved = [0.0];
                data.staged.read_range_into(offset, &mut saved).unwrap();
                data.staged.write_range(offset, &[0.0]).unwrap();
                (offset, saved[0])
            }
        };
        let expected_index = if kind == 2 { data.staged.len() - 1 } else { 0 };
        let before = bits(&raw(&data.current));
        let error = plan.replace(&mut data).unwrap_err();
        assert!(matches!(
            error,
            TransferError::Backend(crate::diagnostics::ProbeError::Mismatch {
                index,
                expected,
                actual,
            }) if index == expected_index && expected == -131_072.0 && actual == 0.0
        ));
        assert_eq!(bits(&raw(&data.current)), before);
        assert!(data.readback().is_err());
        match kind {
            0 => data.current.write_range(offset, &[saved]).unwrap(),
            1 => data.delta.write_range(offset, &[saved]).unwrap(),
            _ => data.staged.write_range(offset, &[saved]).unwrap(),
        }
        plan.replace(&mut data).unwrap();
        assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);
    }

    let nb = plan.input.parent.len();
    let n = plan.input.body.len();
    let dof = plan.input.body.iter().position(|&body| body > 0).unwrap();
    let body = plan.input.body[dof] as usize;
    data.write_geometry(AppliedForceGeometry::Xipos, &vec![0.0; 3 * nb])
        .unwrap();
    data.write_geometry(AppliedForceGeometry::SubtreeCom, &vec![0.0; 3 * nb])
        .unwrap();
    let mut cdof = vec![0.0; 6 * n];
    cdof[6 * dof + 3] = 1.0;
    data.write_geometry(AppliedForceGeometry::Cdof, &cdof)
        .unwrap();
    let mut ft = vec![0.0; 6 * nb];
    ft[6 * body] = f32::MAX;
    data.write_ft(&ft).unwrap();
    let mut current = vec![0.0; n];
    current[dof] = f32::MAX;
    data.write_qfrc(&current).unwrap();
    let before = bits(&raw(&data.current));
    let error = plan.accumulate(&mut data).unwrap_err();
    assert!(matches!(
        error,
        TransferError::Input(InputError::NonFinite {
            field: "qfrc_staged",
            index,
        }) if index == dof
    ));
    assert_eq!(bits(&raw(&data.current)), before);
    assert!(data.readback().is_err());
    plan.replace(&mut data).unwrap();
    assert_eq!(data.readback().unwrap().world(0).unwrap()[dof], f32::MAX);

    for (field, key) in [
        (AppliedForceGeometry::Xipos, "xipos"),
        (AppliedForceGeometry::SubtreeCom, "subtree_com"),
        (AppliedForceGeometry::Cdof, "cdof"),
    ] {
        data.write_geometry(field, &floats(&v, key)).unwrap();
    }
    data.write_ft(&floats(&v, "ft_force_torque")).unwrap();
    data.write_qfrc(&floats(&v, "initial_qfrc")).unwrap();
    plan.replace(&mut data).unwrap();
    assert_eq!(bits(data.readback().unwrap().world(0).unwrap()), expected);

    let report = serde_json::json!({
        "entry": "applied_force_publish",
        "replace_ignores_old_nan": true,
        "accumulate_bitwise": true,
        "guard_failures": 3,
        "nonfinite_publish_rejected": true,
        "current_unchanged_on_failure": true,
        "same_plan_recovered": true,
        "phase_ms": started.elapsed().as_millis(),
        "task_gpu_memory": "unknown",
        "full_U075_NP": false
    });
    std::fs::write(
        std::env::var_os("MJWARP_APPLIED_FORCE_PUBLISH_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}
