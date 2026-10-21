//! 柔体边与面测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_edge_readback_rejects_guards_nonfinite_and_foreign_inputs() {
    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(false), vec![-1; 2]).unwrap(),
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
            flex_vert: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
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
        flexedge_j_rownnz: vec![1],
        flexedge_j_colind: vec![0],
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
    plan.update(&mut data).unwrap();
    let l = data.layout.edges.unwrap();
    for qvel in [false, true] {
        let len = if qvel { l.guarded_qvel } else { l.guarded };
        for index in [0, 3, len - 4, len - 1] {
            let e = data.edges.as_mut().unwrap();
            let b = if qvel { &mut e.qvel } else { &mut e.values };
            b.write_range(index, &[0.0]).unwrap();
            assert!(matches!(data.readback(),Err(TransferError::Backend(
                    crate::diagnostics::ProbeError::Mismatch {index:i,..})) if i==index));
            let e = data.edges.as_mut().unwrap();
            let b = if qvel { &mut e.qvel } else { &mut e.values };
            b.write_range(index, &[-131072.0]).unwrap();
        }
        for index in [4, 4 + (len - 8) / 2, len - 5] {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let e = data.edges.as_mut().unwrap();
                let b = if qvel { &mut e.qvel } else { &mut e.values };
                b.write_range(index, &[value]).unwrap();
                let field = if qvel {
                    "resident_qvel_input"
                } else {
                    "resident_flex_edge_output"
                };
                assert!(matches!(data.readback(),Err(TransferError::Input(
                        InputError::NonFinite {field:f,index:i})) if f==field && i==index-4));
                if qvel {
                    data.edges
                        .as_mut()
                        .unwrap()
                        .qvel
                        .write_range(index, &[0.0])
                        .unwrap();
                }
                plan.update_flex_edges(&mut data).unwrap();
            }
        }
    }
    data.flex
        .as_mut()
        .unwrap()
        .write_range(4, &[f32::MAX, 0.0, 0.0])
        .unwrap();
    plan.update_flex_edges(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::Input(InputError::NonFinite {
            field: "resident_flex_edge_output",
            index: 0
        }))
    ));
    plan.update_flex_positions(&mut data).unwrap();
    plan.update_flex_edges(&mut data).unwrap();
    let device = plan.edges.as_ref().unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let foreign_qvel = foreign.upload(&vec![0.0; l.guarded_qvel]).unwrap();
    let flex = data.flex.as_ref().unwrap();
    let edges = data.edges.as_mut().unwrap();
    // SAFETY: These invalid calls exit before launching. Other buffers keep
    // their checked resident layouts and exact private kernel ABI.
    unsafe {
        assert!(matches!(
            device.kernel.launch_flex_edges(
                &device.metadata,
                &foreign_qvel,
                &data.com,
                flex,
                &mut edges.values,
                513
            ),
            Err(TransferError::SessionMismatch)
        ));
        for worlds in [0, u32::MAX] {
            assert!(matches!(
                device.kernel.launch_flex_edges(
                    &device.metadata,
                    &edges.qvel,
                    &data.com,
                    flex,
                    &mut edges.values,
                    worlds
                ),
                Err(TransferError::Input(InputError::InvalidDimension { .. }))
            ));
        }
    }
    data.readback().unwrap();
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_face_checks_analytic_rotations_shear_interior_and_degenerate_states() {
    let session = TransferSession::new(0).unwrap();
    for (angle, shear, zero) in [
        (0.0f32, 0.0, false),
        (std::f32::consts::FRAC_PI_4, 0.0, false),
        (0.0, 0.4, false),
        (0.0, 0.0, true),
    ] {
        let mut nodes = Vec::new();
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let (x, y, z) = (x as f32, y as f32, z as f32);
                    let (a, b) = (x + shear * y, y);
                    nodes.extend(if zero {
                        [0.0; 3]
                    } else {
                        [
                            angle.cos() * a - angle.sin() * b + 1.0,
                            angle.sin() * a + angle.cos() * b - 2.0,
                            z + 3.0,
                        ]
                    });
                }
            }
        }
        // Frozen nodal semantics retain a distinct body-attached interior.
        if angle == 0.0 && shear == 0.0 && !zero {
            nodes[3 * 13..3 * 13 + 3].copy_from_slice(&[9.0, 8.0, 7.0]);
        }
        let plan = flex_face_plan(&session, nodes, [2, 2, 2]);
        let mut data = plan.create_data(513).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let f = out.flex_faces().unwrap().world(512).unwrap();
        let theta = if shear != 0.0 {
            -(shear / 2.0).atan()
        } else {
            angle
        };
        for face in 0..24 {
            assert!(
                f.face_xpos[27 * face + 12..27 * face + 27]
                    .iter()
                    .all(|&v| v == 0.0)
            );
            if shear != 0.0 && face < 16 {
                continue;
            }
            let q = &f.face_quat[4 * face..4 * face + 4];
            let expected = if zero {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, (theta / 2.0).sin(), (theta / 2.0).cos()]
            };
            for (&a, &e) in q.iter().zip(&expected) {
                assert!(
                    (a - e).abs() < 2e-5,
                    "face={face} actual={q:?} expected={expected:?}"
                );
            }
        }
        if angle == 0.0 && shear == 0.0 && !zero {
            assert_eq!(
                out.flex_positions()
                    .unwrap()
                    .world(512)
                    .unwrap()
                    .flexvert_xpos,
                &[9.0, 8.0, 7.0]
            );
        }
    }
    // Preserve the frozen iteration's stationary 180-degree limitation.
    let mut nodes = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                nodes.extend([-(x as f32), -(y as f32), z as f32]);
            }
        }
    }
    let plan = flex_face_plan(&session, nodes, [1, 1, 1]);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    assert!(
        data.readback()
            .unwrap()
            .flex_faces()
            .unwrap()
            .world(512)
            .unwrap()
            .face_quat
            .as_chunks::<4>()
            .0
            .iter()
            .all(|q| *q == [0.0, 0.0, 0.0, 1.0])
    );
}
#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow() {
    let session = TransferSession::new(0).unwrap();
    let mut nodes = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                nodes.extend([x as f32, y as f32, z as f32]);
            }
        }
    }
    let plan = flex_face_plan(&session, nodes, [1, 1, 1]);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let l = data.layout.faces.unwrap();
    for i in [0, 3, l.guarded - 4, l.guarded - 1] {
        data.faces.as_mut().unwrap().write_range(i, &[0.0]).unwrap();
        assert!(
            matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch{index,..})) if index==i)
        );
        data.faces
            .as_mut()
            .unwrap()
            .write_range(i, &[-131072.0])
            .unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for i in [4, 4 + 512 * 186 + 100, l.guarded - 5] {
            data.faces.as_mut().unwrap().write_range(i, &[bad]).unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{field:"resident_flex_face_output",index})) if index==i-4)
            );
            plan.update_flex_faces(&mut data).unwrap();
        }
    }
    // Overflowed polar arithmetic follows frozen Warp normalize: NaN length returns identity.
    let mut huge = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                huge.extend([x as f32 * 1e20, y as f32 * 1e20, z as f32 * 1e20]);
            }
        }
    }
    data.flex
        .as_mut()
        .unwrap()
        .write_range(4 + 512 * 27, &huge)
        .unwrap();
    plan.update_flex_faces(&mut data).unwrap();
    let out = data.readback().unwrap();
    let world = out.flex_faces().unwrap().world(512).unwrap();
    assert!(world.face_xpos.iter().all(|x| x.is_finite()));
    assert!(world.face_xpos.contains(&1e20));
    assert!(
        world
            .face_quat
            .as_chunks::<4>()
            .0
            .iter()
            .all(|q| *q == [0.0, 0.0, 0.0, 1.0])
    );
    plan.update_flex_positions(&mut data).unwrap();
    plan.update_flex_faces(&mut data).unwrap();
    data.readback().unwrap();
}
