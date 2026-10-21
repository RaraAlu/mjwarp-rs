//! 参数、模型身份、mocap与相机测试。

use super::checks::check_mocap;
use super::checks::check_parameters;
#[cfg(feature = "cuda-probe")]
use super::output::read_output;
#[cfg(feature = "cuda-probe")]
use super::pack::ATTACHED_PARAMETERS;
#[cfg(feature = "cuda-probe")]
use super::pack::COM_PARAMETERS;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::pack_model;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::packed_length;
use super::tests_support::*;
use super::*;

#[test]
fn checks_all_layouts_before_device_allocation() {
    let m = empty_model();
    for worlds in [0, u32::MAX as usize, usize::MAX] {
        assert!(matches!(
            ResidentLayout::new(&m, worlds, 0),
            Err(InputError::InvalidDimension { .. })
        ));
    }
    let l = ResidentLayout::new(&m, 513, 0).unwrap();
    assert_eq!(l.qpos.total_elements(), 0);
    assert_eq!(l.rigid.output.total_elements(), 513 * 28);
    assert_eq!(l.com.output.total_elements(), 513 * 14);
    assert_eq!(l.attached.output.total_elements(), 0);
    assert!(l.qpos.world_elements(512).is_ok());
    assert!(l.qpos.world_elements(513).is_err());
}

#[test]
fn rejects_unready_and_stale_stage_combinations() {
    let mut s = ReadyStages::default();
    assert_eq!(
        s.require_all(),
        Err(TransferError::StageNotReady { stage: "rigid" })
    );
    s.rigid = true;
    assert_eq!(
        s.require_all(),
        Err(TransferError::StageNotReady { stage: "attached" })
    );
    s.attached = true;
    assert_eq!(
        s.require_all(),
        Err(TransferError::StageNotReady { stage: "com" })
    );
    s.com = true;
    assert!(s.require_all().is_ok());
    s = ReadyStages::default();
    assert!(s.require_all().is_err());
}

#[test]
fn model_identity_distinguishes_equal_inputs() {
    let a = Arc::new(empty_model());
    let b = Arc::new((*a).clone());
    assert_eq!(a, b);
    assert!(!Arc::ptr_eq(&a, &b));
    assert!(Arc::ptr_eq(&a, &Arc::clone(&a)));
}

#[test]
fn validates_all_parameter_rows_and_model_dimensions() {
    use crate::model::ParameterBatch;
    let model = parameter_model(false);
    let mut parameters = KinematicsParameters::default();
    for (i, field) in KinematicsParameter::ALL.into_iter().enumerate() {
        let shared = field.shared(&model);
        parameters.set(
            field,
            ParameterBatch::new(i + 1, shared.len(), shared.repeat(i + 1)).unwrap(),
        );
        check_parameters(&model, &parameters).unwrap();
        let mut bad = parameters.clone();
        bad.set(
            field,
            ParameterBatch::new(2, shared.len() + 1, vec![0.0; 2 * (shared.len() + 1)]).unwrap(),
        );
        assert!(
            matches!(check_parameters(&model, &bad), Err(InputError::LengthMismatch { field: f, .. }) if f == field.name())
        );
    }
}

#[test]
fn rejects_invalid_physical_parameters_in_later_rows() {
    use crate::model::{KinematicsParameter as F, ParameterBatch};
    let model = parameter_model(false);
    for (field, local, value, reason) in [
        (F::BodyPos, 0, 1.0, "noncanonical_world"),
        (F::BodyIpos, 1, 1.0, "noncanonical_world"),
        (F::BodyQuat, 0, -1.0, "noncanonical_world"),
        (F::BodyIquat, 0, -1.0, "noncanonical_world"),
        (F::BodyQuat, 4, 0.5, "nonunit_model_rotation"),
        (F::BodyIquat, 4, 0.5, "nonunit_model_rotation"),
        (F::GeomQuat, 0, 0.5, "nonunit_model_rotation"),
        (F::SiteQuat, 0, 0.5, "nonunit_model_rotation"),
        (F::JointAxis, 0, 0.5, "nonunit_joint_axis"),
        (F::BodyMass, 0, 1.0, "nonzero_world_inertia"),
        (F::BodyInertia, 2, 1.0, "nonzero_world_inertia"),
    ] {
        let shared = field.shared(&model);
        let mut rows = shared.repeat(3);
        rows[2 * shared.len() + local] = value;
        let mut parameters = KinematicsParameters::default();
        parameters.set(field, ParameterBatch::new(3, shared.len(), rows).unwrap());
        assert!(
            matches!(check_parameters(&model, &parameters), Err(InputError::InvalidTopology { field: f, reason: r, .. }) if f == field.name() && r == reason)
        );
    }
    for (field, local) in [(F::BodyMass, 1), (F::BodyInertia, 4)] {
        let shared = field.shared(&model);
        let mut rows = shared.repeat(3);
        rows[2 * shared.len() + local] = -0.1;
        let mut p = KinematicsParameters::default();
        p.set(field, ParameterBatch::new(3, shared.len(), rows).unwrap());
        assert_eq!(
            check_parameters(&model, &p),
            Err(InputError::NegativeValue {
                field: field.name(),
                index: 2 * shared.len() + local
            })
        );
    }
    let free = parameter_model(true);
    let mut rows = F::Qpos0.shared(&free).repeat(3);
    rows[17] = 0.0;
    let mut p = KinematicsParameters::default();
    p.set(F::Qpos0, ParameterBatch::new(3, 7, rows).unwrap());
    assert!(matches!(
        check_parameters(&free, &p),
        Err(InputError::InvalidTopology {
            field: "qpos0",
            index: 17,
            reason: "unusable_state_quaternion"
        })
    ));
}

#[cfg(feature = "cuda-probe")]
#[test]
fn packs_independent_periods_and_checks_aggregate_capacity() {
    use crate::model::{KinematicsParameter as F, ParameterBatch};
    let model = parameter_model(false);
    let mut parameters = KinematicsParameters::default();
    parameters.set(
        F::BodyMass,
        ParameterBatch::new(3, 2, vec![0.0, 1.0, 0.0, 2.0, 0.0, 3.0]).unwrap(),
    );
    let packed = pack_model(&model, &parameters, &[&[0, 0]], &COM_PARAMETERS).unwrap();
    assert_eq!(packed.metadata, [0, 0, 0, 3, 6, 1]);
    assert_eq!(&packed.parameters[..6], &[0.0, 1.0, 0.0, 2.0, 0.0, 3.0]);
    assert_eq!(packed.parameters.len(), 12);
    assert_eq!(
        packed_length([i32::MAX as usize].into_iter()).unwrap(),
        i32::MAX as usize
    );
    for lengths in [[i32::MAX as usize, 1], [usize::MAX, 1]] {
        assert!(packed_length(lengths.into_iter()).is_err());
    }
    let empty = pack_model(&empty_model(), &parameters, &[&[28]], &ATTACHED_PARAMETERS).unwrap();
    assert_eq!(empty.parameters, [0.0]);
    assert_eq!(empty.metadata, [28, 0, 1, 0, 1, 0, 1, 0, 1]);
}
#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn static_geom_cache_survives_changed_body_frames_while_sites_update() {
    let session = TransferSession::new(0).unwrap();
    for (ng, ns) in [(1, 1), (1, 0), (0, 1), (0, 0)] {
        let model = AttachedModelInput::new(
            empty_model().rigid().clone(),
            crate::model::AttachedFields {
                geom_bodyid: vec![0; ng],
                geom_pos: [1.0, 2.0, 3.0].repeat(ng),
                geom_quat: [1.0, 0.0, 0.0, 0.0].repeat(ng),
                site_bodyid: vec![0; ns],
                site_pos: [4.0, 5.0, 6.0].repeat(ns),
                site_quat: [1.0, 0.0, 0.0, 0.0].repeat(ns),
            },
        )
        .unwrap();
        let plan = KinematicsPlan::new(&session, model).unwrap();
        let mut data = plan.create_data(513).unwrap();
        let initial = read_output(
            &data.attached,
            data.layout.attached.guarded,
            "initial_static",
        )
        .unwrap();
        if ng != 0 {
            assert_eq!(&initial[4..7], &[1.0, 2.0, 3.0]);
        }
        assert!(data.write_mocap(&[1.0], &[]).is_err());
        data.write_mocap(&[], &[]).unwrap();
        data.write_world_mocap(512, &[], &[]).unwrap();
        plan.update_rigid(&mut data).unwrap();
        // Test-only corruption distinguishes skip semantics from recomputing
        // an unchanged static transform. No public API exposes this write.
        for world in 0..513 {
            data.rigid
                .write_range(4 + world * 28, &[9.0, 8.0, 7.0])
                .unwrap();
        }
        plan.update_attached(&mut data).unwrap();
        let updated = read_output(
            &data.attached,
            data.layout.attached.guarded,
            "updated_static",
        )
        .unwrap();
        for world in 0..513 {
            let start = 4 + world * 12 * (ng + ns);
            assert_eq!(
                &initial[start..start + 12 * ng],
                &updated[start..start + 12 * ng]
            );
            if ns != 0 {
                assert_eq!(&updated[start + 12 * ng..start + 12 * ng + 3], &[13.0; 3]);
            }
        }
    }
}

#[test]
fn validates_mocap_lengths_finiteness_and_quaternion_capacity() {
    assert!(check_mocap(0, 0, &[], &[]).is_ok());
    assert!(check_mocap(3, 4, &[0.0; 3], &[-2.0, -0.0, 0.0, 0.0]).is_ok());
    assert!(check_mocap(3, 4, &[0.0; 3], &[]).is_err());
    assert!(check_mocap(0, 0, &[1.0], &[]).is_err());
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            check_mocap(3, 4, &[bad, 0.0, 0.0], &[1.0, 0.0, 0.0, 0.0]),
            Err(InputError::NonFinite {
                field: "mocap_pos",
                index: 0
            })
        ));
        assert!(matches!(
            check_mocap(3, 4, &[0.0; 3], &[bad, 0.0, 0.0, 0.0]),
            Err(InputError::NonFinite {
                field: "mocap_quat",
                index: 0
            })
        ));
    }
    for bad in [0.0, 1e-7, 1e7, f32::MAX] {
        assert!(matches!(
            check_mocap(6, 8, &[0.0; 6], &[1.0, 0.0, 0.0, 0.0, bad, 0.0, 0.0, 0.0]),
            Err(InputError::InvalidTopology {
                field: "mocap_quat",
                index: 4,
                ..
            })
        ));
    }
}

#[test]
fn checks_packed_mocap_state_capacity_before_allocation() {
    let m = empty_model();
    let l = ResidentLayout::new(&m, 513, 2).unwrap();
    assert_eq!(l.state.total_elements(), 513 * 14);
    assert_eq!(l.mocap_pos.world_elements(512).unwrap(), 3072..3078);
    assert_eq!(l.mocap_quat.world_elements(512).unwrap(), 4096..4104);
    for nm in [i32::MAX as usize / 7 + 1, usize::MAX] {
        assert!(matches!(
            ResidentLayout::new(&m, 1, nm),
            Err(InputError::Overflow {
                field: "resident_state"
            })
        ));
    }
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn camlight_readback_rejects_device_guards_and_nonfinite_payloads() {
    let session = TransferSession::new(0).unwrap();
    let model = CamLightModelInput::new(
        MocapModelInput::new(empty_model(), vec![-1]).unwrap(),
        CamLightFields {
            cam_mode: vec![0],
            cam_bodyid: vec![0],
            cam_targetbodyid: vec![-1],
            cam_pos: vec![1.0, 2.0, 3.0],
            cam_quat: vec![1.0, 0.0, 0.0, 0.0],
            cam_poscom0: vec![0.0; 3],
            cam_pos0: vec![0.0; 3],
            cam_mat0: vec![0.0; 9],
            light_mode: vec![0],
            light_bodyid: vec![0],
            light_targetbodyid: vec![-1],
            light_pos: vec![1.0, 2.0, 3.0],
            light_dir: vec![0.0, 0.0, 1.0],
            light_poscom0: vec![0.0; 3],
            light_pos0: vec![0.0; 3],
            light_dir0: vec![0.0; 3],
        },
    )
    .unwrap();
    let plan = KinematicsPlan::with_camlight(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let len = data.layout.camlight.guarded;
    for index in [0, 3, len - 4, len - 1] {
        data.camlight.write_range(index, &[0.0]).unwrap();
        assert!(
            matches!(data.readback(),Err(TransferError::Backend(crate::diagnostics::ProbeError::Mismatch{index:i,..})) if i==index)
        );
        data.camlight.write_range(index, &[-131072.0]).unwrap();
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [4, 4 + 512 * 18 + 3, 4 + 512 * 18 + 12, 4 + 512 * 18 + 15] {
            data.camlight.write_range(index, &[bad]).unwrap();
            assert!(
                matches!(data.readback(),Err(TransferError::Input(InputError::NonFinite{field:"resident_camlight_output",index:i})) if i==index-4)
            );
            plan.update_camlight(&mut data).unwrap();
        }
    }
    data.readback().unwrap();
}
