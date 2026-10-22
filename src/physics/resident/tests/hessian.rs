//! 柔体Hessian混合测试。

#[cfg(feature = "cuda-probe")]
use super::output::read_flex_validity;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_hessian_clears_all_mixed_sleeping_worlds_before_positions() {
    let session = TransferSession::new(0).unwrap();
    let plan = flex_hessian_plan(&session);
    let mut states = 0;
    let mut flags = 0;
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        assert_eq!(
            read_flex_validity(data.flex_validity.as_ref().unwrap(), 8 + 3 * worlds).unwrap(),
            vec![false; 3 * worlds]
        );
        let sleeping = SleepTreeState {
            tree_asleep: vec![0],
            nbody_awake: 1,
            nv_awake: 0,
        };
        for w in 0..worlds {
            data.write_world_sleep(w, &sleeping).unwrap();
        }
        for round in 0..4 {
            // Simulate a future cache producer privately; no public write API.
            let seed: Vec<_> = (0..3 * worlds)
                .map(|i| {
                    if round == 0 {
                        1
                    } else {
                        (i + round) as i32 % 2
                    }
                })
                .collect();
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(4, &seed)
                .unwrap();
            if round == 0 {
                assert!(matches!(
                    plan.update_flex_positions(&mut data),
                    Err(TransferError::StageNotReady { stage: "rigid" })
                ));
                assert_eq!(
                    read_flex_validity(data.flex_validity.as_ref().unwrap(), 8 + 3 * worlds)
                        .unwrap(),
                    vec![true; 3 * worlds]
                );
            }
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            assert_eq!(out.flex_positions().unwrap().nflex(), 3);
            for w in 0..worlds {
                assert_eq!(
                    out.flex_positions()
                        .unwrap()
                        .world(w)
                        .unwrap()
                        .flex_hessian_valid,
                    &[false; 3]
                );
                assert_eq!(
                    out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                    &[0]
                );
            }
            states += worlds;
            flags += 3 * worlds;
        }
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &vec![1; 3 * worlds])
            .unwrap();
        // Non-position stages must not clear this private simulated cache.
        plan.update_attached(&mut data).unwrap();
        plan.update_com(&mut data).unwrap();
        plan.update_camlight(&mut data).unwrap();
        data.write_world_qvel(worlds - 1, &[0.0; 6]).unwrap();
        plan.update_flex_edges(&mut data).unwrap();
        plan.update_flex_faces(&mut data).unwrap();
        plan.update_tendons(&mut data).unwrap();
        plan.update_tendon_wake(&mut data).unwrap();
        let old = data.readback().unwrap();
        assert_eq!(
            old.flex_positions()
                .unwrap()
                .world(worlds - 1)
                .unwrap()
                .flex_hessian_valid,
            &[true; 3]
        );
        assert!(data.write_world_qpos(worlds, &[]).is_err());
        assert!(data.write_world_qvel(0, &[f32::NAN; 6]).is_err());
        assert!(data.write_world_mocap(worlds, &[], &[]).is_err());
        assert_eq!(
            data.readback()
                .unwrap()
                .flex_positions()
                .unwrap()
                .world(0)
                .unwrap()
                .flex_hessian_valid,
            &[true; 3]
        );
        plan.update_flex_positions(&mut data).unwrap();
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady {
                stage: "flex_edges"
            })
        ));
        plan.update_flex_edges(&mut data).unwrap();
        plan.update_flex_faces(&mut data).unwrap();
        assert_eq!(
            data.readback()
                .unwrap()
                .flex_positions()
                .unwrap()
                .world(0)
                .unwrap()
                .flex_hessian_valid,
            &[false; 3]
        );
        assert_eq!(
            old.flex_positions()
                .unwrap()
                .world(0)
                .unwrap()
                .flex_hessian_valid,
            &[true; 3]
        );
    }
    assert_eq!((states, flags), (2084, 6252));
    println!("G01-flex-hessian states={states} flags={flags} exact_false=true");
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_hessian_rejects_guards_nonboolean_slots_and_late_launch_failure() {
    let session = TransferSession::new(0).unwrap();
    let mut plan = flex_hessian_plan(&session);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let length = data.layout.flex.unwrap().guarded_validity;
    for index in [0, 3, length - 4, length - 1] {
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(index, &[0])
            .unwrap();
        assert!(
            matches!(data.readback(), Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch {index: i,..})) if i == index)
        );
        // Clearing the payload must leave guard corruption visible.
        plan.update(&mut data).unwrap();
        assert!(data.readback().is_err());
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(index, &[-131072])
            .unwrap();
    }
    for index in [4, 4 + 3 * 256 + 1, length - 5] {
        for value in [-1, 2, i32::MIN, i32::MAX] {
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(index, &[value])
                .unwrap();
            assert!(
                matches!(data.readback(), Err(TransferError::Input(InputError::InvalidTopology {field: "resident_flex_hessian_valid",index: i,..})) if i == index - 4)
            );
            plan.update(&mut data).unwrap();
            data.readback().unwrap();
        }
    }
    let foreign = TransferSession::new(0).unwrap();
    let other = flex_hessian_plan(&session);
    data.flex_validity
        .as_mut()
        .unwrap()
        .write_range(4, &vec![1; 3 * 513])
        .unwrap();
    assert_eq!(
        other.update_flex_positions(&mut data),
        Err(TransferError::ModelMismatch)
    );
    assert!(
        read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
            .unwrap()
            .iter()
            .all(|&v| v)
    );
    let replacement = SynchronousKernel::compile(
        &foreign,
        super::super::flex::FLEX_POSITION_CUDA,
        "flex_positions",
    )
    .unwrap();
    let original = std::mem::replace(
        &mut plan.flex.as_mut().unwrap().kernel,
        FlexPositionsKernel::from_kernel(replacement),
    );
    assert_eq!(
        plan.update_flex_positions(&mut data),
        Err(TransferError::SessionMismatch)
    );
    // A later position-launch failure must not retain the valid flags.
    assert!(
        read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
            .unwrap()
            .iter()
            .all(|&v| !v)
    );
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "flex_positions"
        })
    ));
    plan.flex.as_mut().unwrap().kernel = original;
    plan.update(&mut data).unwrap();
    data.flex_validity
        .as_mut()
        .unwrap()
        .write_range(4, &vec![1; 3 * 513])
        .unwrap();
    let replacement = SynchronousKernel::compile(
        &foreign,
        super::super::flex::FLEX_POSITION_CUDA,
        "flex_invalidate",
    )
    .unwrap();
    let original = std::mem::replace(
        &mut plan.flex.as_mut().unwrap().invalidate_kernel,
        FlexInvalidateKernel::from_kernel(replacement),
    );
    assert_eq!(
        plan.update_flex_positions(&mut data),
        Err(TransferError::SessionMismatch)
    );
    assert!(
        read_flex_validity(data.flex_validity.as_ref().unwrap(), length)
            .unwrap()
            .iter()
            .all(|&v| v)
    );
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "flex_positions"
        })
    ));
    plan.flex.as_mut().unwrap().invalidate_kernel = original;
    let qpos = plan.model.rigid().kinematics().fields().qpos0.clone();
    data.write_world_qpos(512, &qpos).unwrap();
    assert!(matches!(
        plan.update_flex_positions(&mut data),
        Err(TransferError::StageNotReady { stage: "rigid" })
    ));
    plan.update(&mut data).unwrap();
    let snapshot = data.readback().unwrap();
    drop(plan);
    drop(other);
    drop(foreign);
    drop(session);
    assert_eq!(
        data.readback()
            .unwrap()
            .flex_positions()
            .unwrap()
            .world(512)
            .unwrap()
            .flex_hessian_valid,
        &[false; 3]
    );
    drop(data);
    assert_eq!(
        snapshot
            .flex_positions()
            .unwrap()
            .world(512)
            .unwrap()
            .flex_hessian_valid,
        &[false; 3]
    );
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn flex_hessian_handles_empty_and_static_direct_flex_routes() {
    let session = TransferSession::new(0).unwrap();
    let old = KinematicsPlan::new(&session, empty_model()).unwrap();
    let mut data = old.create_data(5).unwrap();
    assert!(data.flex_validity.is_none());
    old.update(&mut data).unwrap();
    assert!(data.readback().unwrap().flex_positions().is_none());
    for nf in [0, 1, 3] {
        let cam = CamLightModelInput::new(
            MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        let input = FlexPositionModelInput::new(
            TendonModelInput::new(cam, crate::model::TendonFields::default()).unwrap(),
            FlexPositionFields {
                flex_interp: vec![0; nf],
                flex_cellnum: vec![1; 3 * nf],
                flex_nodeadr: vec![0; nf],
                flex_nodenum: vec![0; nf],
                flex_vertadr: (0..nf as i32).collect(),
                flex_vertnum: vec![1; nf],
                flex_centered: vec![false; nf],
                flex_vertbodyid: vec![0; nf],
                flex_vert: vec![0.0; 3 * nf],
                flex_vert0: vec![0.0; 3 * nf],
                ..Default::default()
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
        assert_eq!(data.flex.as_ref().unwrap().len(), 8 + 3 * 513 * nf);
        assert_eq!(data.flex_validity.as_ref().unwrap().len(), 8 + 513 * nf);
        if nf != 0 {
            data.flex_validity
                .as_mut()
                .unwrap()
                .write_range(4, &vec![1; 513 * nf])
                .unwrap();
        }
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        let f = out.flex_positions().unwrap();
        assert_eq!(f.nflex(), nf);
        assert_eq!(f.world(512).unwrap().flex_hessian_valid, vec![false; nf]);
        assert_eq!(f.world(512).unwrap().flexvert_xpos, vec![0.0; 3 * nf]);
        assert!(f.world(513).is_err());
    }
}
