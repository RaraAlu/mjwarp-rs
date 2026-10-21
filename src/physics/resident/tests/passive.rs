//! 边被动力测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_edge_passive_uses_resident_values_and_preserves_conditional_reads() {
    use crate::physics::{
        FlexEdgePassiveInput, FlexEdgePassiveOptions, FlexEdgePassivePlan, FlexEdgePassiveSnapshot,
    };

    fn passive_input(length: f32) -> FlexEdgePassiveInput {
        FlexEdgePassiveInput {
            length0: vec![length + 0.5, f32::NAN, length - 0.25, length, f32::NAN],
            stiffness: vec![2.0, 3.0, 4.0],
            damping: vec![0.5, 0.75, 1.0],
            rigid: vec![false, true, false, false, false],
        }
    }

    fn reference(
        snapshot: &KinematicsSnapshot,
        passive: &FlexEdgePassiveSnapshot,
        input: &FlexEdgePassiveInput,
        world: usize,
    ) {
        let edge = snapshot.flex_edges().unwrap().world(world).unwrap();
        let fields = snapshot.flex_edges().unwrap().fields();
        let flexid = [0usize, 0, 0, 1, 2];
        let mut spring = vec![0.0f32; 6];
        let mut damper = vec![0.0f32; 6];
        for (e, &f) in flexid.iter().enumerate() {
            if input.rigid[e] {
                continue;
            }
            let stiffness = input.stiffness[f];
            let damping = input.damping[f];
            if stiffness == 0.0 && damping == 0.0 {
                continue;
            }
            let count = fields.flexedge_j_rownnz[e] as usize;
            if count == 0 {
                continue;
            }
            let force_spring = if stiffness != 0.0 {
                stiffness * (input.length0[e] - edge.flexedge_length[e])
            } else {
                0.0
            };
            let force_damper = if damping != 0.0 {
                -damping * edge.flexedge_velocity[e]
            } else {
                0.0
            };
            if force_spring == 0.0 && force_damper == 0.0 {
                continue;
            }
            let start = fields.flexedge_j_rowadr[e] as usize;
            for i in 0..count {
                let column = fields.flexedge_j_colind[start + i] as usize;
                let jacobian = edge.flexedge_jacobian[start + i];
                if force_spring != 0.0 {
                    spring[column] += jacobian * force_spring;
                }
                if force_damper != 0.0 {
                    damper[column] += jacobian * force_damper;
                }
            }
        }
        for (actual, expected) in [
            (passive.spring(world).unwrap(), spring.as_slice()),
            (passive.damper(world).unwrap(), damper.as_slice()),
        ] {
            for (&actual, &expected) in actual.iter().zip(expected) {
                let tolerance = 2e-6 + 2e-6 * expected.abs();
                assert!(
                    actual.is_finite() && (actual - expected).abs() <= tolerance,
                    "world={world} actual={actual} expected={expected}"
                );
            }
        }
    }

    let session = TransferSession::new(0).unwrap();
    let cam = CamLightModelInput::new(
        MocapModelInput::new(parameter_model(true), vec![-1; 2]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let vertices = [2.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(3);
    let input = FlexPositionModelInput::new(
        TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
        FlexPositionFields {
            flex_interp: vec![0; 3],
            flex_cellnum: vec![1; 9],
            flex_nodeadr: vec![0; 3],
            flex_nodenum: vec![0; 3],
            flex_vertadr: vec![0, 2, 4],
            flex_vertnum: vec![2; 3],
            flex_centered: vec![false; 3],
            flex_vertbodyid: [0, 1].repeat(3),
            flex_vert: vertices,
            flex_vert0: vec![0.0; 18],
            ..Default::default()
        },
    )
    .unwrap()
    .with_edges(FlexEdgeFields {
        flex_edgeadr: vec![0, 3, 4],
        flex_edgenum: vec![3, 1, 1],
        flex_edge: [0, 1].repeat(5),
        flexedge_j_rowadr: vec![0, 6, 12, 18, 0],
        flexedge_j_rownnz: vec![6, 6, 6, 6, 0],
        flexedge_j_colind: (0..6).cycle().take(24).collect(),
    })
    .unwrap();
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let norm = 5.0_f32.sqrt();
    let parameters = passive_input(norm);
    let passive =
        FlexEdgePassivePlan::new(&plan, parameters.clone(), FlexEdgePassiveOptions::default())
            .unwrap();
    let disabled = FlexEdgePassivePlan::new(
        &plan,
        FlexEdgePassiveInput {
            length0: vec![f32::NAN; 5],
            stiffness: vec![f32::NAN; 3],
            damping: vec![f32::NAN; 3],
            rigid: vec![false; 5],
        },
        FlexEdgePassiveOptions {
            disable_spring: true,
            disable_damper: true,
        },
    )
    .unwrap();
    let mut data = plan.create_data(11).unwrap();
    let mut passive_data = passive.create_data(11).unwrap();
    assert!(matches!(
        passive.update(&data, &mut passive_data),
        Err(TransferError::StageNotReady {
            stage: "flex_edge_passive_source"
        })
    ));
    let mut wrong_worlds = passive.create_data(2).unwrap();
    for world in 0..11 {
        let scale = world as f32 * 0.125 - 0.5;
        let qvel: Vec<f32> = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
            .into_iter()
            .map(|v| v * scale)
            .collect();
        data.write_world_qvel(world, &qvel).unwrap();
    }
    plan.update(&mut data).unwrap();
    assert!(matches!(
        passive.update(&data, &mut wrong_worlds),
        Err(TransferError::ModelMismatch)
    ));
    let source = data.readback().unwrap();
    let output = passive
        .update(&data, &mut passive_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..11 {
        reference(&source, &output, &parameters, world);
    }

    data.write_qvel(&vec![0.0; 11 * 6]).unwrap();
    assert!(matches!(
        passive.update(&data, &mut passive_data),
        Err(TransferError::StageNotReady {
            stage: "flex_edge_passive_source"
        })
    ));
    plan.update_flex_edges(&mut data).unwrap();
    let zero_source = data.readback().unwrap();
    let stride = 2 * 5 + 24;
    for world in 0..11 {
        let base = 4 + world * stride;
        let values = &mut data.edges.as_mut().unwrap().values;
        values.write_range(base + 5 + 4, &[f32::NAN]).unwrap();
        values.write_range(base + 10 + 6, &[f32::NAN; 6]).unwrap();
        values.write_range(base + 10 + 18, &[f32::NAN; 6]).unwrap();
    }
    let conditional = passive
        .update(&data, &mut passive_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..11 {
        reference(&zero_source, &conditional, &parameters, world);
    }

    let first_active_j = 4 + 2 * 5;
    data.edges
        .as_mut()
        .unwrap()
        .values
        .write_range(first_active_j, &[f32::NAN])
        .unwrap();
    assert!(matches!(
        passive.update(&data, &mut passive_data),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "flex_edge_passive_source",
            ..
        }))
    ));

    data.edges
        .as_mut()
        .unwrap()
        .values
        .write_range(4, &vec![f32::NAN; 11 * stride])
        .unwrap();
    let mut disabled_data = disabled.create_data(11).unwrap();
    let output = disabled
        .update(&data, &mut disabled_data)
        .unwrap()
        .snapshot()
        .unwrap();
    for world in 0..11 {
        assert_eq!(output.spring(world).unwrap(), &[0.0; 6]);
        assert_eq!(output.damper(world).unwrap(), &[0.0; 6]);
    }
    assert!(output.spring(11).is_err());
    println!(
        "flex-edge-passive worlds=11 compared_slots={} self_owned_bytes=2960",
        3 * 2 * 11 * 6
    );
}
