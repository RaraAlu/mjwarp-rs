//! 固定与空间肌腱测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn global_tendon_readback_rejects_guards_and_nonfinite_values() {
    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let model = TendonModelInput::new(
        cam,
        crate::model::TendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![1],
            wrap_type: vec![1],
            wrap_objid: vec![0],
            wrap_prm: vec![-2.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![1],
            ten_j_colind: vec![0],
        },
    )
    .unwrap();
    let plan = KinematicsPlan::with_tendons(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let layout = data.layout.tendon.unwrap();
    for index in [0, 3, layout.guarded - 4, layout.guarded - 1] {
        data.tendon
            .as_mut()
            .unwrap()
            .values
            .write_range(index, &[0.0])
            .unwrap();
        assert!(matches!(data.readback(),Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index));
        data.tendon
            .as_mut()
            .unwrap()
            .values
            .write_range(index, &[-131072.0])
            .unwrap();
    }
    for index in [0, 3, layout.guarded_indices - 4, layout.guarded_indices - 1] {
        data.tendon
            .as_mut()
            .unwrap()
            .indices
            .write_range(index, &[0])
            .unwrap();
        assert!(matches!(data.readback(),Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index));
        data.tendon
            .as_mut()
            .unwrap()
            .indices
            .write_range(index, &[-131072])
            .unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [4, 4 + 512 * 8, 4 + 512 * 8 + 1, layout.guarded - 5] {
            data.tendon
                .as_mut()
                .unwrap()
                .values
                .write_range(index, &[bad])
                .unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{
                    field:"resident_tendon_output",index:i})) if i==index-4)
            );
            plan.update_tendons(&mut data).unwrap();
        }
    }
    data.readback().unwrap();
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn fixed_tendon_readback_rejects_device_guards_and_nonfinite_payloads() {
    let session = TransferSession::new(0).unwrap();
    let plan = fixed_tendon_plan(&session, -2.0);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let length = data.layout.fixed_tendon.guarded;
    for index in [0, 3, length - 4, length - 1] {
        data.fixed_tendon.write_range(index, &[0.0]).unwrap();
        assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
        data.fixed_tendon.write_range(index, &[-131072.0]).unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [4, 5, 4 + 512 * 2, 5 + 512 * 2] {
            data.fixed_tendon.write_range(index, &[bad]).unwrap();
            assert!(matches!(data.readback(), Err(TransferError::Input(
                    InputError::NonFinite { field: "resident_fixed_tendon_output", index: i })) if i==index-4));
            plan.update_fixed_tendons(&mut data).unwrap();
        }
    }
    data.readback().unwrap();
    // An empty zero-DOF model still exposes checked empty tendon views.
    let plan = KinematicsPlan::new(&session, empty_model()).unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    assert!(
        data.readback()
            .unwrap()
            .fixed_tendon()
            .world(512)
            .unwrap()
            .ten_jacobian
            .is_empty()
    );
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn fixed_tendon_readback_rejects_arithmetic_overflow_and_recovers() {
    let session = TransferSession::new(0).unwrap();
    let plan = fixed_tendon_plan(&session, f32::MAX);
    let mut data = plan.create_data(513).unwrap();
    data.write_world_qpos(512, &[2.0]).unwrap();
    plan.update(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::Input(InputError::NonFinite {
            field: "resident_fixed_tendon_output",
            index: 1024
        }))
    ));
    data.write_world_qpos(512, &[0.5]).unwrap();
    plan.update(&mut data).unwrap();
    assert_eq!(
        data.readback()
            .unwrap()
            .fixed_tendon()
            .world(512)
            .unwrap()
            .ten_length,
        &[f32::MAX * 0.5]
    );
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn spatial_tendon_readback_rejects_both_device_guards_and_nonfinite_payloads() {
    let session = TransferSession::new(0).unwrap();
    let plan = spatial_tendon_plan(&session);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    for index in [
        0,
        3,
        data.layout.spatial_tendon.guarded - 4,
        data.layout.spatial_tendon.guarded - 1,
    ] {
        data.spatial_tendon.write_range(index, &[0.0]).unwrap();
        assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
        data.spatial_tendon
            .write_range(index, &[-131072.0])
            .unwrap();
    }
    for index in [
        0,
        3,
        data.layout.spatial_tendon.guarded_indices - 4,
        data.layout.spatial_tendon.guarded_indices - 1,
    ] {
        data.spatial_wrap.write_range(index, &[0]).unwrap();
        assert!(matches!(data.readback(), Err(TransferError::Backend(
                crate::diagnostics::ProbeError::Mismatch { index: i, .. })) if i==index));
        data.spatial_wrap.write_range(index, &[-131072]).unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [
            4,
            5,
            6,
            4 + 512 * 28,
            5 + 512 * 28,
            6 + 512 * 28,
            18 + 512 * 28,
            31 + 512 * 28,
        ] {
            data.spatial_tendon.write_range(index, &[bad]).unwrap();
            assert!(matches!(data.readback(), Err(TransferError::Input(
                    InputError::NonFinite { field: "resident_spatial_tendon_output", index: i })) if i==index-4));
            plan.update_spatial_tendons(&mut data).unwrap();
        }
    }
    // Every update clears unused integer and position slots too.
    data.spatial_wrap
        .write_range(4, &vec![i32::MAX; 513 * 6])
        .unwrap();
    plan.update_spatial_tendons(&mut data).unwrap();
    let out = data.readback().unwrap();
    let row = out.spatial_tendon().world(512).unwrap();
    assert_eq!(row.wrap_obj, &[-1, -1, 0, 0]);
    assert_eq!(&row.wrap_xpos[6..], &[0.0; 6]);
    let empty = KinematicsPlan::new(&session, empty_model()).unwrap();
    let mut data = empty.create_data(513).unwrap();
    empty.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let row = out.spatial_tendon().world(512).unwrap();
    assert!(row.ten_length.is_empty() && row.wrap_obj.is_empty());
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_edge_free_body_matches_translation_rotation_and_velocity() {
    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let input = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_vertbodyid: vec![0, 1],
            flex_vert: vec![2.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            flex_vert0: vec![0.0; 6],
            ..Default::default()
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![6],
        flexedge_j_colind: (0..6).collect(),
    })
    .unwrap();
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    data.write_qvel(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0].repeat(513))
        .unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let norm = 5.0_f32.sqrt();
    for w in 0..513 {
        let f = out.flex_edges().unwrap().world(w).unwrap();
        assert!((f.flexedge_length[0] - norm).abs() < 1e-6);
        for (&a, e) in
            f.flexedge_jacobian
                .iter()
                .zip([-2.0 / norm, 1.0 / norm, 0.0, 0.0, 0.0, 2.0 / norm])
        {
            assert!((a - e).abs() < 1e-6);
        }
        assert!((f.flexedge_velocity[0] - 12.0 / norm).abs() < 1e-6);
    }
    let cam = CamLightModelInput::new(
        MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let input = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0],
            flex_cellnum: vec![1; 3],
            flex_nodeadr: vec![0],
            flex_nodenum: vec![0],
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_centered: vec![false],
            flex_vertbodyid: vec![0; 2],
            flex_vert: vec![0.0, 0.0, 0.0, 3.0, 4.0, 0.0],
            flex_vert0: vec![0.0; 6],
            ..Default::default()
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![1],
        flex_edge: vec![0, 1],
        flexedge_j_rowadr: vec![0],
        flexedge_j_rownnz: vec![0],
        flexedge_j_colind: vec![],
    })
    .unwrap();
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    data.write_qvel(&[]).unwrap();
    data.write_world_qvel(512, &[]).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for w in 0..513 {
        let f = out.flex_edges().unwrap().world(w).unwrap();
        assert_eq!(f.flexedge_length, &[5.0]);
        assert_eq!(f.flexedge_velocity, &[0.0]);
        assert!(f.flexedge_jacobian.is_empty());
    }
}
