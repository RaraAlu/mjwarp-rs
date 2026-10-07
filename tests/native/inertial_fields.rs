//! 固定样本与独立原生打印件。
use super::{EMPTY, JOINTS, MIXED, dll, load, mock};
use mjwarp_rs::{
    diagnostics::{InputError, NativeProbeError},
    io::NativeInertialSnapshot,
    model::{InertialFields, KinematicFields},
};

pub(super) const TREE: &[u8] = include_bytes!("../../fixtures/native-probe/inertial-tree.mjb");

fn kinematic_expected() -> KinematicFields {
    KinematicFields {
        qpos0: vec![
            0.5, 0.0, 0.25, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, -0.5, 0.125, -0.25,
        ],
        body_parentid: vec![0, 0, 1, 1, 3, 1, 0, 6],
        body_jntadr: vec![-1, 0, 1, -1, 2, 4, -1, 5],
        body_jntnum: vec![0, 1, 1, 0, 2, 1, 0, 1],
        body_pos: vec![
            0.0, 0.0, 0.0, 0.5, 0.0, 0.25, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0, 0.25, 0.0, 0.0, -0.5, 0.0,
            0.0, 2.0, 0.0, 0.0, 0.0, 0.5, 0.0,
        ],
        body_quat: [1.0, 0.0, 0.0, 0.0].repeat(8),
        jnt_type: vec![0, 1, 2, 3, 3, 2],
        jnt_bodyid: vec![1, 2, 4, 4, 5, 7],
        jnt_qposadr: vec![0, 7, 11, 12, 13, 14],
        jnt_dofadr: vec![0, 6, 9, 10, 11, 12],
        jnt_pos: vec![0.0; 18],
        jnt_axis: vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            1.0,
        ],
    }
}
fn expected() -> InertialFields {
    // Authored XML supplies binary-exact constants; the separately generated
    // official printout confirms compiled structure, not exact floating precision.
    InertialFields {
        body_ipos: vec![
            0.0, 0.0, 0.0, 0.125, 0.0, 0.25, 0.0, 0.125, 0.0, 0.0, 0.0, 0.125, 0.125, 0.25, 0.5,
            0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.25, 0.0,
        ],
        body_iquat: vec![
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ],
        body_mass: vec![0.0, 2.0, 3.0, 1.0, 4.0, 5.0, 6.0, 7.0],
        body_inertia: vec![
            0.0, 0.0, 0.0, 0.5, 0.75, 1.0, 1.0, 1.25, 1.5, 0.25, 0.375, 0.5, 1.5, 1.75, 2.0, 2.0,
            2.5, 3.0, 2.5, 3.0, 3.5, 3.0, 3.5, 4.0,
        ],
        dof_bodyid: vec![1, 1, 1, 1, 1, 1, 2, 2, 2, 4, 4, 5, 7],
        dof_jntid: vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3, 4, 5],
        dof_parentid: vec![-1, 0, 1, 2, 3, 4, 5, 6, 7, 5, 9, 5, -1],
        dof_armature: vec![
            0.25, 0.25, 0.25, 0.25, 0.25, 0.25, 0.5, 0.5, 0.5, 0.75, 1.0, 1.25, 1.5,
        ],
        dof_damping: vec![
            0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.25, 0.25, 0.25, 0.375, 0.5, 0.625, 0.75,
        ],
    }
}
fn floats(s: &mut NativeInertialSnapshot) -> [(&'static str, &mut Vec<f64>); 6] {
    [
        ("body_ipos", &mut s.body_ipos),
        ("body_iquat", &mut s.body_iquat),
        ("body_mass", &mut s.body_mass),
        ("body_inertia", &mut s.body_inertia),
        ("dof_armature", &mut s.dof_armature),
        ("dof_damping", &mut s.dof_damping),
    ]
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn reads_owned_inertial_tree_against_independent_reference() {
    let p = load(&dll(), TREE).unwrap();
    let mut s = p.inertial_snapshot().unwrap();
    drop(p);
    let info = s.kinematics.info;
    assert_eq!((info.nq, info.nv, info.nbody, info.njnt), (15, 13, 8, 6));
    let e = expected();
    for ((_, native), values) in floats(&mut s).into_iter().zip([
        &e.body_ipos,
        &e.body_iquat,
        &e.body_mass,
        &e.body_inertia,
        &e.dof_armature,
        &e.dof_damping,
    ]) {
        assert_eq!(
            *native,
            values.iter().copied().map(f64::from).collect::<Vec<_>>()
        );
    }
    assert_eq!(s.dof_bodyid, e.dof_bodyid);
    assert_eq!(s.dof_jntid, e.dof_jntid);
    assert_eq!(s.dof_parentid, e.dof_parentid);
    let m = s.into_model().unwrap();
    assert_eq!(m.fields(), &e);
    assert_eq!(m.kinematics().fields(), &kinematic_expected());
}
#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn accepts_original_fixtures_as_inertial_subsets() {
    for bytes in [EMPTY, JOINTS, MIXED] {
        let p = load(&dll(), bytes).unwrap();
        let k = p.kinematic_snapshot().unwrap().into_model().unwrap();
        let old = p.snapshot().unwrap();
        let s = p.inertial_snapshot().unwrap();
        drop(p);
        assert_eq!(s.body_mass, old.body_mass);
        let m = s.into_model().unwrap();
        assert_eq!(m.kinematics(), &k);
        assert_eq!(m.fields().dof_bodyid.len(), k.nv());
    }
}
#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn rejects_mutated_inertial_lengths_values_and_ancestry() {
    let s = load(&dll(), TREE).unwrap().inertial_snapshot().unwrap();
    for case in 0..9 {
        let mut m = s.clone();
        let field = if case < 6 {
            let (field, v) = &mut floats(&mut m)[case];
            v.pop();
            *field
        } else {
            let (field, v) = match case {
                6 => ("dof_bodyid", &mut m.dof_bodyid),
                7 => ("dof_jntid", &mut m.dof_jntid),
                _ => ("dof_parentid", &mut m.dof_parentid),
            };
            v.pop();
            field
        };
        assert!(
            matches!(m.into_model(), Err(NativeProbeError::Input(InputError::LengthMismatch { field: name, .. })) if name == field)
        );
    }
    for case in 0..6 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
            let mut m = s.clone();
            let (field, v) = &mut floats(&mut m)[case];
            v[1] = value;
            let field = *field;
            let error = if value.is_finite() {
                InputError::ScalarOverflow { field, index: 1 }
            } else {
                InputError::NonFinite { field, index: 1 }
            };
            assert_eq!(m.into_model(), Err(NativeProbeError::Input(error)));
        }
    }
    for case in 2..6 {
        for value in [-0.25, -f64::MIN_POSITIVE] {
            let mut m = s.clone();
            let (field, v) = &mut floats(&mut m)[case];
            v[1] = value;
            let field = *field;
            assert_eq!(
                m.into_model(),
                Err(NativeProbeError::Input(InputError::NegativeValue {
                    field,
                    index: 1
                }))
            );
        }
    }
    for case in 0..3 {
        let mut m = s.clone();
        let field = match case {
            0 => {
                m.dof_bodyid[9] = 2;
                "dof_bodyid"
            }
            1 => {
                m.dof_jntid[9] = -1;
                "dof_jntid"
            }
            _ => {
                m.dof_parentid[9] = 8;
                "dof_parentid"
            }
        };
        assert!(
            matches!(m.into_model(), Err(NativeProbeError::Input(InputError::InvalidTopology { field: name, index: 9, .. })) if name == field)
        );
    }
    let mut m = s.clone();
    m.kinematics.info.nv = 12;
    assert!(matches!(
        m.into_model(),
        Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
    ));
    let mut m = s;
    m.kinematics.jnt_dofadr[1] = 5;
    assert!(matches!(
        m.into_model(),
        Err(NativeProbeError::Input(InputError::InvalidTopology {
            field: "jnt_dofadr",
            ..
        }))
    ));
}
#[test]
#[ignore = "requires native test DLLs and dedicated log path"]
fn rejects_missing_inertial_source_and_releases_owner_in_order() {
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    std::fs::write(&path, []).unwrap();
    let p = load(&mock("missing-inertial-pointer"), EMPTY).unwrap();
    assert!(matches!(
        p.inertial_snapshot(),
        Err(NativeProbeError::Native {
            stage: "copy_inertial",
            code: 8
        })
    ));
    assert!(std::fs::read(&path).unwrap().is_empty());
    assert_eq!(p.snapshot().unwrap().body_mass, [0.0]);
    p.kinematic_snapshot().unwrap().into_model().unwrap();
    drop(p);
    assert_eq!(std::fs::read(&path).unwrap(), b"DU");
}
#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires verified MuJoCo DLL and NVIDIA GPU"]
fn reads_all_gpu_inertial_fields_after_dropping_native_host_and_session() {
    use mjwarp_rs::{io::DeviceInertialModel, runtime::TransferSession};
    let p = load(&dll(), TREE).unwrap();
    let input = p.inertial_snapshot().unwrap().into_model().unwrap();
    let session = TransferSession::new(0).unwrap();
    let gpu = DeviceInertialModel::upload(&session, &input).unwrap();
    drop(p);
    drop(input);
    drop(session);
    let result = gpu.readback().unwrap();
    assert_eq!(result.fields(), &expected());
    assert_eq!(result.kinematics().fields(), &kinematic_expected());
}
#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires verified MuJoCo DLL and NVIDIA GPU"]
fn uploads_zero_dof_inertial_groups_and_preserves_finite_bits() {
    use mjwarp_rs::{io::DeviceInertialModel, model::InertialModelInput, runtime::TransferSession};
    let session = TransferSession::new(0).unwrap();
    for bytes in [EMPTY, MIXED] {
        let input = load(&dll(), bytes)
            .unwrap()
            .inertial_snapshot()
            .unwrap()
            .into_model()
            .unwrap();
        let mut f = input.fields().clone();
        f.body_ipos[0] = -0.0;
        f.body_iquat[0] = 2.0;
        f.body_mass[0] = -0.0;
        if !f.dof_damping.is_empty() {
            f.dof_damping[0] = -0.0;
        }
        let input = InertialModelInput::new(input.kinematics().clone(), f).unwrap();
        let gpu = DeviceInertialModel::upload(&session, &input).unwrap();
        let result = gpu.readback().unwrap();
        assert_eq!(result, input);
        assert_eq!(result.fields().body_ipos[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(result.fields().body_mass[0].to_bits(), (-0.0_f32).to_bits());
        if !result.fields().dof_damping.is_empty() {
            assert_eq!(
                result.fields().dof_damping[0].to_bits(),
                (-0.0_f32).to_bits()
            );
        }
    }
}
