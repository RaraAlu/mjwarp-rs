//! G01物理辅助测试。

use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::InertialModelInput;

use super::*;

pub(super) fn test_model(joint: Option<i32>) -> InertialModelInput {
    use crate::model::{InertialFields, KinematicFields, KinematicModelInput};
    let (nq, nv) = match joint {
        Some(0) => (7, 6),
        Some(1) => (4, 3),
        Some(_) => (1, 1),
        None => (0, 0),
    };
    let mut k = KinematicFields {
        qpos0: vec![0.0; nq],
        body_parentid: vec![0, 0],
        body_jntadr: vec![-1, if joint.is_some() { 0 } else { -1 }],
        body_jntnum: vec![0, i32::from(joint.is_some())],
        body_pos: vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75],
        body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
        ..Default::default()
    };
    if let Some(ty) = joint {
        k.jnt_type.push(ty);
        k.jnt_bodyid.push(1);
        k.jnt_qposadr.push(0);
        k.jnt_dofadr.push(0);
        k.jnt_pos.extend([0.125, 0.25, 0.5]);
        k.jnt_axis.extend([0.0, 0.0, 1.0]);
        if ty < 2 {
            k.qpos0[if ty == 0 { 3 } else { 0 }] = 1.0;
        }
    }
    let i = InertialFields {
        body_ipos: vec![0.0; 6],
        body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
        body_mass: vec![0.0, 1.0],
        body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        dof_bodyid: vec![1; nv],
        dof_jntid: vec![0; nv],
        dof_parentid: (0..nv).map(|n| n as i32 - 1).collect(),
        dof_armature: vec![0.0; nv],
        dof_damping: vec![0.0; nv],
    };
    InertialModelInput::new(KinematicModelInput::new(nv, k).unwrap(), i).unwrap()
}

fn edited(
    model: &InertialModelInput,
    edit: impl FnOnce(&mut crate::model::KinematicFields, &mut crate::model::InertialFields),
) -> InertialModelInput {
    let mut k = model.kinematics().fields().clone();
    let mut i = model.fields().clone();
    edit(&mut k, &mut i);
    InertialModelInput::new(
        crate::model::KinematicModelInput::new(model.kinematics().nv(), k).unwrap(),
        i,
    )
    .unwrap()
}

#[test]
fn rejects_kinematics_dimensions_capacity_and_state_length() {
    // G01 subset: no full-stage acceptance.
    let m = test_model(Some(3));
    for worlds in [0, usize::MAX, u32::MAX as usize] {
        assert!(check_kinematics(&m, worlds, &[]).is_err());
    }
    for dimensions in [
        (1, usize::MAX, 2, 0),
        (1, 0, usize::MAX, 0),
        (1, 0, 1, usize::MAX),
        (1, 0, 0, 0),
    ] {
        assert!(
            KinematicsLayout::new(dimensions.0, dimensions.1, dimensions.2, dimensions.3).is_err()
        );
    }
    assert!(matches!(
        check_kinematics(&m, 2, &[0.0]),
        Err(InputError::LengthMismatch {
            expected: 2,
            actual: 1,
            ..
        })
    ));
    assert!(check_kinematics(&m, 2, &[0.0, 0.5]).is_ok());
}

#[test]
fn rejects_nonfinite_state_in_every_world() {
    let m = test_model(Some(3));
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            check_kinematics(&m, 2, &[0.0, bad]),
            Err(InputError::NonFinite { index: 1, .. })
        ));
    }
}

#[test]
fn rejects_noncanonical_world_and_nonunit_model_rotations() {
    let m = test_model(Some(3));
    for n in 0..6 {
        let changed = edited(&m, |k, i| match n {
            0 => k.body_pos[0] = 0.5,
            1 => k.body_quat[0] = -1.0,
            2 => i.body_ipos[0] = 0.5,
            3 => i.body_iquat[0] = -1.0,
            4 => k.body_quat[4] = 2.0,
            _ => i.body_iquat[4] = 0.0,
        });
        assert!(check_kinematics(&changed, 1, &[0.0]).is_err());
    }
}

#[test]
fn requires_unit_slide_and_hinge_axes() {
    for ty in [2, 3] {
        let m = test_model(Some(ty));
        let changed = edited(&m, |k, _| k.jnt_axis.fill(0.0));
        assert!(matches!(
            check_kinematics(&changed, 1, &[0.0]),
            Err(InputError::InvalidTopology {
                field: "jnt_axis",
                ..
            })
        ));
    }
}

#[test]
fn preserves_usable_free_and_ball_quaternions() {
    for ty in [0, 1] {
        let m = test_model(Some(ty));
        let mut q = m.kinematics().fields().qpos0.repeat(2);
        let a = m.kinematics().nq() + if ty == 0 { 3 } else { 0 };
        for bad in [0.0, 1e-8, 1e8] {
            q[a..a + 4].fill(0.0);
            q[a] = bad;
            assert!(
                matches!(check_kinematics(&m,2,&q),Err(InputError::InvalidTopology {index,..}) if index==a)
            );
        }
        q[a..a + 4].copy_from_slice(&[-2.0, 1.0, 0.0, 0.0]);
        let original = q.clone();
        check_kinematics(&m, 2, &q).unwrap();
        assert_eq!(q, original);
    }
}

#[test]
fn supports_static_zero_dof_shapes_and_checked_output_views() {
    let m = test_model(None);
    let layout = check_kinematics(&m, 2, &[]).unwrap();
    assert_eq!(layout.output.total_elements(), 112);
    let out = KinematicsOutput {
        layout,
        values: (0..120).map(|n| n as f32).collect(),
    };
    let w = out.world(1).unwrap();
    assert_eq!(w.xpos, &[60.0, 61.0, 62.0, 63.0, 64.0, 65.0]);
    assert_eq!(w.xquat.len(), 8);
    assert_eq!(w.xmat.len(), 18);
    assert_eq!(w.xipos.len(), 6);
    assert_eq!(w.ximat.len(), 18);
    assert!(w.xanchor.is_empty() && w.xaxis.is_empty());
    assert!(out.world(2).is_err());
}

#[test]
fn rejects_com_position_capacity_and_invalid_world_inertia() {
    for dims in [
        (0, 1, 0, 0),
        (1, 0, 0, 0),
        (usize::MAX, 1, 0, 0),
        (1, usize::MAX, 0, 0),
        (1, 1, usize::MAX, 0),
        (1, 1, 0, usize::MAX),
        (1, 1, 0, i32::MAX as usize),
    ] {
        assert!(ComPositionLayout::new(dims.0, dims.1, dims.2, dims.3).is_err());
    }
    let m = test_model(Some(3));
    for field in ["body_mass", "body_inertia"] {
        let changed = edited(&m, |_, i| {
            if field == "body_mass" {
                i.body_mass[0] = 1.0;
            } else {
                i.body_inertia[0] = 1.0;
            }
        });
        assert!(matches!(
            check_com_position(&changed, 1, &[0.0]),
            Err(InputError::InvalidTopology {
                reason: "nonzero_world_inertia",
                ..
            })
        ));
        // The narrower COM contract does not restrict the existing FK helper.
        assert!(check_kinematics(&changed, 1, &[0.0]).is_ok());
    }
}

#[test]
fn com_position_reuses_kinematics_checks_before_device_work() {
    let m = test_model(Some(3));
    assert!(check_com_position(&m, 2, &[0.0]).is_err());
    assert!(check_com_position(&m, 2, &[0.0, f32::NAN]).is_err());
    let changed = edited(&m, |k, _| k.jnt_axis.fill(0.0));
    assert!(check_com_position(&changed, 1, &[0.0]).is_err());
    assert!(check_com_position(&m, 2, &[0.0, 0.5]).is_ok());
}

#[test]
fn com_position_views_keep_world_body_and_dof_ranges_separate() {
    let layout = ComPositionLayout::new(2, 2, 1, 1).unwrap();
    let out = ComPositionOutput {
        layout,
        values: (0..76).map(|n| n as f32).collect(),
    };
    assert_eq!(out.worlds(), 2);
    assert_eq!(out.nbody(), 2);
    assert_eq!(out.nv(), 1);
    let w = out.world(1).unwrap();
    assert_eq!(w.subtree_mass, &[38.0, 39.0]);
    assert_eq!(w.subtree_com, &[40.0, 41.0, 42.0, 43.0, 44.0, 45.0]);
    assert_eq!(w.cinert.len(), 20);
    assert_eq!(w.cdof, &[66.0, 67.0, 68.0, 69.0, 70.0, 71.0]);
    assert!(out.world(2).is_err());
    let layout = ComPositionLayout::new(1, 1, 0, 0).unwrap();
    let out = ComPositionOutput {
        layout,
        values: vec![0.0; 22],
    };
    assert!(out.world(0).unwrap().cdof.is_empty());
}

#[test]
fn rejects_both_device_guards_and_incomplete_finite_payloads() {
    let mut v = vec![-131072.0; 12];
    v[4..8].fill(0.0);
    check_device_values(&v, "probe_output").unwrap();
    for index in [0, 3, 8, 11] {
        let mut changed = v.clone();
        changed[index] = 0.0;
        assert!(matches!(
            check_device_values(&changed, "probe_output"),
            Err(TransferError::Backend(ProbeError::Mismatch { .. }))
        ));
    }
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut changed = v.clone();
        changed[5] = bad;
        assert!(matches!(
            check_device_values(&changed, "probe_output"),
            Err(TransferError::Input(InputError::NonFinite { index: 1, .. }))
        ));
    }
}

#[test]
fn locates_flattened_history_samples() {
    // MC-38/MC-39：仅检查宿主范围。
    let layout = HistoryLayout::new(2, 3, 4).unwrap();
    assert_eq!(layout.samples(), 3);
    assert_eq!(layout.channels(), 4);
    assert_eq!(layout.batch().total_bytes(), 96);
    assert_eq!(layout.sample_range(1, 2).unwrap(), 20..24);
}

#[test]
fn rejects_history_dimensions_and_indices() {
    for dimensions in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1, usize::MAX, 2)] {
        assert!(HistoryLayout::new(dimensions.0, dimensions.1, dimensions.2).is_err());
    }
    let layout = HistoryLayout::new(2, 3, 4).unwrap();
    assert!(layout.sample_range(2, 0).is_err());
    assert_eq!(
        layout.sample_range(0, 3),
        Err(InputError::InvalidIndex {
            field: "sample",
            index: 3,
            limit: 3
        })
    );
}

#[test]
fn accepts_exact_minimum_interval_and_single_sample() {
    // MC-38：间隔不能小于原生阈值。
    let layout = HistoryLayout::new(2, 3, 1).unwrap();
    layout.validate_times(&[-0.25, 0.0, 0.5], 0.25).unwrap();
    HistoryLayout::new(1, 1, 1)
        .unwrap()
        .validate_times(&[0.0], 0.25)
        .unwrap();
}

#[test]
fn rejects_short_duplicate_decreasing_and_overflowing_intervals() {
    let layout = HistoryLayout::new(1, 2, 1).unwrap();
    for times in [[0.0, 0.125], [1.0, 1.0], [1.0, 0.0], [-f64::MAX, f64::MAX]] {
        assert_eq!(
            layout.validate_times(&times, 0.25),
            Err(InputError::InvalidHistoryTime { index: 1 })
        );
    }
}

#[test]
fn rejects_invalid_times_length_and_threshold() {
    let layout = HistoryLayout::new(1, 2, 1).unwrap();
    assert_eq!(
        layout.validate_times(&[0.0], 0.25),
        Err(InputError::LengthMismatch {
            field: "history_times",
            expected: 2,
            actual: 1
        })
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            layout.validate_times(&[0.0, invalid], 0.25),
            Err(InputError::NonFinite {
                field: "history_times",
                index: 1
            })
        );
    }
    for threshold in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(layout.validate_times(&[0.0, 1.0], threshold).is_err());
    }
}
