//! 柔体位置与休眠回读测试。

#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyKernel;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_position_readback_rejects_guards_and_nonfinite_payloads() {
    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let input = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0, 1],
            flex_cellnum: vec![1; 6],
            flex_nodeadr: vec![0, 0],
            flex_nodenum: vec![0, 8],
            flex_vertadr: vec![0, 1],
            flex_vertnum: vec![1, 1],
            flex_centered: vec![false, true],
            flex_nodebodyid: vec![0; 8],
            flex_vertbodyid: vec![0, -1],
            flex_node: vec![0.0; 24],
            flex_vert: vec![0.2, 0.3, 0.4, 0.0, 0.0, 0.0],
            flex_vert0: vec![0.5; 6],
        },
    )
    .unwrap();
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let len = data.layout.flex.unwrap().guarded;
    for index in [0, 3, len - 4, len - 1] {
        data.flex
            .as_mut()
            .unwrap()
            .write_range(index, &[0.0])
            .unwrap();
        assert!(
            matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch { index:i,.. })) if i==index)
        );
        data.flex
            .as_mut()
            .unwrap()
            .write_range(index, &[-131072.0])
            .unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [
            4,
            4 + 24,
            4 + 29,
            4 + 512 * 30,
            4 + 512 * 30 + 24,
            4 + 512 * 30 + 29,
        ] {
            data.flex
                .as_mut()
                .unwrap()
                .write_range(index, &[bad])
                .unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite {field:"resident_flex_position_output",index:i})) if i==index-4)
            );
            plan.update_flex_positions(&mut data).unwrap();
        }
    }
    let out = data.readback().unwrap();
    assert_eq!(
        out.flex_positions()
            .unwrap()
            .world(512)
            .unwrap()
            .flexvert_xpos,
        &[0.2, 0.3, 0.4, 0.0, 0.0, 0.0]
    );
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn gpu_m3_sleep_copy_uses_real_resident_buffers_and_recovers() {
    fn raw(buffer: &TransferBuffer<i32>) -> Vec<i32> {
        let mut values = crate::runtime::host_staging::<i32>(buffer.len()).unwrap();
        buffer.read_range_into(0, &mut values).unwrap();
        values
    }

    let session = TransferSession::new(0).unwrap();
    let mut plan = flex_hessian_plan(&session);
    let mut data = plan.create_data(3).unwrap();
    plan.update(&mut data).unwrap();
    let states = [
        SleepTreeState {
            tree_asleep: vec![-11],
            nbody_awake: 2,
            nv_awake: 6,
        },
        SleepTreeState {
            tree_asleep: vec![0],
            nbody_awake: 0,
            nv_awake: 0,
        },
        SleepTreeState {
            tree_asleep: vec![-3],
            nbody_awake: 1,
            nv_awake: 2,
        },
    ];
    for (world, state) in states.iter().enumerate() {
        data.write_world_sleep(world, state).unwrap();
    }
    plan.update_tendon_wake(&mut data).unwrap();
    let output = data.readback().unwrap();
    for (world, expected) in states.iter().enumerate() {
        let actual = output.sleep_trees().unwrap().world(world).unwrap();
        assert_eq!(actual.tree_asleep, expected.tree_asleep);
        assert_eq!(actual.tree_awake, &[i32::from(expected.tree_asleep[0] < 0)]);
        assert_eq!(
            (actual.ntree_awake, actual.nbody_awake, actual.nv_awake),
            (
                i32::from(expected.tree_asleep[0] < 0),
                expected.nbody_awake,
                expected.nv_awake,
            )
        );
    }
    let resident = data.sleep.as_ref().unwrap();
    for buffer in [&resident.state, &resident.scratch] {
        let values = raw(buffer);
        assert_eq!(&values[..4], &[-131072; 4]);
        assert_eq!(&values[values.len() - 4..], &[-131072; 4]);
    }

    let state_before = raw(&data.sleep.as_ref().unwrap().state);
    let scratch_before = raw(&data.sleep.as_ref().unwrap().scratch);
    plan.sleep
        .as_mut()
        .unwrap()
        .metadata
        .overwrite_for_test(0, 7)
        .unwrap();
    assert!(matches!(
        plan.update_tendon_wake(&mut data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "sleep_copy_metadata",
            ..
        }))
    ));
    assert!(!data.ready.sleep);
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "tendon_wake"
        })
    ));
    assert_eq!(raw(&data.sleep.as_ref().unwrap().state), state_before);
    assert_eq!(raw(&data.sleep.as_ref().unwrap().scratch), scratch_before);
    plan.sleep
        .as_mut()
        .unwrap()
        .metadata
        .restore_for_test()
        .unwrap();
    plan.update_tendon_wake(&mut data).unwrap();
    data.readback().unwrap();

    let foreign = TransferSession::new(0).unwrap();
    let replacement = SleepCopyKernel::compile(&foreign).unwrap();
    let original = std::mem::replace(&mut plan.sleep.as_mut().unwrap().copy_kernel, replacement);
    let state_before = raw(&data.sleep.as_ref().unwrap().state);
    let scratch_before = raw(&data.sleep.as_ref().unwrap().scratch);
    assert_eq!(
        plan.update_tendon_wake(&mut data),
        Err(TransferError::SessionMismatch)
    );
    assert!(!data.ready.sleep);
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "tendon_wake"
        })
    ));
    assert_eq!(raw(&data.sleep.as_ref().unwrap().state), state_before);
    assert_eq!(raw(&data.sleep.as_ref().unwrap().scratch), scratch_before);
    plan.sleep.as_mut().unwrap().copy_kernel = original;
    plan.update_tendon_wake(&mut data).unwrap();
    data.readback().unwrap();

    let report = serde_json::json!({
        "entry": "sleep_copy",
        "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "worlds": 3,
        "ntree": 1,
        "state_elements": 23,
        "real_resident_call": true,
        "valid_rows_exact": true,
        "guards_preserved": true,
        "stale_real_buffer_write_rejected_without_output_write": true,
        "foreign_session_rejected_without_output_write": true,
        "same_plan_recovery": true,
        "np01_to_np04_accepted": false,
        "performance_accepted": false
    });
    if let Some(path) = std::env::var_os("MJWARP_SLEEP_COPY_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn tendon_wake_readback_rejects_guards_with_static_flex_and_zero_trees() {
    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let flex = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![1],
            flex_centered: vec![false],
            flex_vertbodyid: vec![0],
            flex_vert: vec![0.2, 0.3, 0.4],
            flex_vert0: vec![0.5; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let fields = crate::model::TendonWakeFields {
        body_treeid: vec![-1],
        tendon_limited: vec![],
        tendon_range: crate::model::ParameterBatch::new(3, 0, vec![]).unwrap(),
        tendon_margin: crate::model::ParameterBatch::new(2, 0, vec![]).unwrap(),
        sleep_enabled: true,
        island_disabled: false,
    };
    let input = TendonWakeModelInput::with_flex_positions(flex, fields).unwrap();
    let plan = KinematicsPlan::with_tendon_wake(
        &session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let len = data.layout.sleep.unwrap().guarded;
    for index in [0, 3, len - 4, len - 1] {
        data.sleep
            .as_mut()
            .unwrap()
            .state
            .write_range(index, &[0])
            .unwrap();
        assert!(
            matches!(data.readback(), Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i == index)
        );
        data.sleep
            .as_mut()
            .unwrap()
            .state
            .write_range(index, &[-131072])
            .unwrap();
    }
    let out = data.readback().unwrap();
    assert_eq!(
        out.flex_positions()
            .unwrap()
            .world(512)
            .unwrap()
            .flexvert_xpos,
        [0.2, 0.3, 0.4]
    );
    let row = out.sleep_trees().unwrap().world(512).unwrap();
    assert!(row.tree_asleep.is_empty() && row.tree_awake.is_empty());
    assert_eq!((row.ntree_awake, row.nbody_awake, row.nv_awake), (0, 1, 0));
    assert!(out.sleep_trees().unwrap().world(513).is_err());
    assert!(plan.create_data(0).is_err());
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn spatial_tendon_readback_rejects_overflow_and_recovers_from_coincident_sites() {
    let session = TransferSession::new(0).unwrap();
    let plan = spatial_tendon_plan(&session);
    let mut data = plan.create_data(513).unwrap();
    data.write_world_qpos(512, &[-1.0]).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let row = out.spatial_tendon().world(512).unwrap();
    assert_eq!(row.ten_length, &[0.0]);
    assert_eq!(row.ten_jacobian, &[1.0]);
    // Isolate spatial norm overflow without overflowing COM inertia first.
    let position = 4 + 512 * 36 + 12 + 3;
    data.attached.write_range(position, &[1e20]).unwrap();
    plan.update_spatial_tendons(&mut data).unwrap();
    let error = data.readback().expect_err("spatial norm must overflow");
    let start = data
        .layout
        .spatial_tendon
        .output
        .world_elements(512)
        .unwrap()
        .start;
    assert!(
        matches!(
            error,
            TransferError::Input(InputError::NonFinite {
                field: "resident_spatial_tendon_output",
                index
            }) if index == start
        ),
        "{error:?}"
    );
    data.write_world_qpos(512, &[2.0]).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let row = out.spatial_tendon().world(512).unwrap();
    assert_eq!(row.ten_length, &[3.0]);
    assert_eq!(row.ten_jacobian, &[1.0]);
}
