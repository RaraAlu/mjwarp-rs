#![cfg(feature = "native-model-probe")]

use mjwarp_rs::{diagnostics::NativeProbeError, io::NativeModelProbe, model::NativeModelInfo};
use std::path::PathBuf;

const JOINTS: &[u8] = include_bytes!("../fixtures/native-probe/two-joints.mjb");
const EMPTY: &[u8] = include_bytes!("../fixtures/native-probe/zero-dof.mjb");
const MIXED: &[u8] = include_bytes!("../fixtures/native-probe/mixed-joints.mjb");

#[path = "native/kinematic_preflight.rs"]
mod preflight;

fn mixed_expected() -> mjwarp_rs::model::KinematicFields {
    use mjwarp_rs::model::KinematicFields;
    // XML supplies exact binary fractions. Official compile.exe's separate
    // printout independently confirms counts, order, addresses and default axes.
    KinematicFields {
        qpos0: vec![
            0.5, 0.25, 0.125, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.25, -0.5,
        ],
        body_parentid: vec![0, 0, 1, 2, 3, 0],
        body_jntadr: vec![-1, 0, 1, 2, 3, -1],
        body_jntnum: vec![0, 1, 1, 1, 1, 0],
        body_pos: vec![
            0.0, 0.0, 0.0, 0.5, 0.25, 0.125, 0.0, 0.0, 0.5, 0.25, 0.0, 0.0, 0.0, 0.5, 0.0, 2.0,
            0.0, 0.0,
        ],
        body_quat: [1.0, 0.0, 0.0, 0.0].repeat(6),
        jnt_type: vec![0, 1, 2, 3],
        jnt_bodyid: vec![1, 2, 3, 4],
        jnt_qposadr: vec![0, 7, 11, 12],
        jnt_dofadr: vec![0, 6, 9, 10],
        jnt_pos: vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.25, 0.0,
        ],
        jnt_axis: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    }
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn reads_twelve_mixed_joint_fields_against_independent_reference() {
    let probe = load(&dll(), MIXED).unwrap();
    let snapshot = probe.kinematic_snapshot().unwrap();
    drop(probe);
    let i = snapshot.info;
    assert_eq!((i.nq, i.nv, i.nbody, i.njnt), (13, 11, 6, 4));
    let expected = mixed_expected();
    for (native, values) in [
        (&snapshot.qpos0, &expected.qpos0),
        (&snapshot.body_pos, &expected.body_pos),
        (&snapshot.body_quat, &expected.body_quat),
        (&snapshot.jnt_pos, &expected.jnt_pos),
        (&snapshot.jnt_axis, &expected.jnt_axis),
    ] {
        assert_eq!(
            *native,
            values.iter().copied().map(f64::from).collect::<Vec<_>>()
        );
    }
    for (native, values) in [
        (&snapshot.body_parentid, &expected.body_parentid),
        (&snapshot.body_jntadr, &expected.body_jntadr),
        (&snapshot.body_jntnum, &expected.body_jntnum),
        (&snapshot.jnt_type, &expected.jnt_type),
        (&snapshot.jnt_bodyid, &expected.jnt_bodyid),
        (&snapshot.jnt_qposadr, &expected.jnt_qposadr),
        (&snapshot.jnt_dofadr, &expected.jnt_dofadr),
    ] {
        assert_eq!(native, values);
    }
    let model = snapshot.into_model().unwrap();
    assert_eq!(model.fields(), &expected);
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn accepts_original_fixtures_as_validated_kinematic_inputs() {
    for (bytes, nq, nv, nb, nj) in [(JOINTS, 2, 2, 3, 2), (EMPTY, 0, 0, 1, 0)] {
        let probe = load(&dll(), bytes).unwrap();
        let snapshot = probe.kinematic_snapshot().unwrap();
        drop(probe);
        let m = snapshot.into_model().unwrap();
        assert_eq!((m.nq(), m.nv(), m.nbody(), m.njnt()), (nq, nv, nb, nj));
        assert_eq!(m.fields().body_quat, [1.0, 0.0, 0.0, 0.0].repeat(nb));
        assert_eq!(m.fields().body_jntadr[0], -1);
    }
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn rejects_mutated_snapshot_lengths_indices_and_float_failures() {
    use mjwarp_rs::diagnostics::InputError;
    let snapshot = load(&dll(), MIXED).unwrap().kinematic_snapshot().unwrap();
    for case in 0..12 {
        let mut s = snapshot.clone();
        match case {
            0 => {
                s.qpos0.pop();
            }
            1 => {
                s.body_parentid.pop();
            }
            2 => {
                s.body_jntadr.pop();
            }
            3 => {
                s.body_jntnum.pop();
            }
            4 => {
                s.body_pos.pop();
            }
            5 => {
                s.body_quat.pop();
            }
            6 => {
                s.jnt_type.pop();
            }
            7 => {
                s.jnt_bodyid.pop();
            }
            8 => {
                s.jnt_qposadr.pop();
            }
            9 => {
                s.jnt_dofadr.pop();
            }
            10 => {
                s.jnt_pos.pop();
            }
            _ => {
                s.jnt_axis.pop();
            }
        }
        assert!(matches!(
            s.into_model(),
            Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
        ));
    }
    for case in 0..5 {
        let mut s = snapshot.clone();
        match case {
            0 => s.body_parentid[1] = 1,
            1 => s.jnt_bodyid[0] = 0,
            2 => s.jnt_qposadr[1] = 6,
            3 => s.jnt_dofadr[1] = -1,
            _ => s.jnt_type[1] = 4,
        }
        assert!(matches!(
            s.into_model(),
            Err(NativeProbeError::Input(InputError::InvalidTopology { .. }))
        ));
    }
    for case in 0..5 {
        for invalid in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            -f64::MAX,
        ] {
            let mut s = snapshot.clone();
            let (field, values) = match case {
                0 => ("qpos0", &mut s.qpos0),
                1 => ("body_pos", &mut s.body_pos),
                2 => ("body_quat", &mut s.body_quat),
                3 => ("jnt_pos", &mut s.jnt_pos),
                _ => ("jnt_axis", &mut s.jnt_axis),
            };
            values[1] = invalid;
            let error = if invalid.is_finite() {
                InputError::ScalarOverflow { field, index: 1 }
            } else {
                InputError::NonFinite { field, index: 1 }
            };
            assert_eq!(s.into_model(), Err(NativeProbeError::Input(error)));
        }
    }
    let mut s = snapshot;
    s.info.nv = i64::from(i32::MAX) + 1;
    assert_eq!(
        s.into_model(),
        Err(NativeProbeError::Input(InputError::Overflow {
            field: "nv"
        }))
    );
}

#[test]
#[ignore = "requires native test DLLs and dedicated log path"]
fn rejects_missing_kinematic_source_and_releases_owner_in_order() {
    let path = PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    std::fs::write(&path, []).unwrap();
    let probe = load(&mock("missing-kinematic-pointer"), EMPTY).unwrap();
    assert!(matches!(
        probe.kinematic_snapshot(),
        Err(NativeProbeError::Native {
            stage: "copy_kinematic",
            code: 8
        })
    ));
    assert!(std::fs::read(&path).unwrap().is_empty());
    // A copy failure neither mutates nor prematurely deletes the model.
    assert_eq!(probe.snapshot().unwrap().body_parentid, [0]);
    drop(probe);
    assert_eq!(std::fs::read(&path).unwrap(), b"DU");
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires verified MuJoCo DLL and NVIDIA GPU"]
fn reads_all_gpu_kinematic_fields_after_dropping_native_host_and_session() {
    use mjwarp_rs::{io::DeviceKinematicModel, runtime::TransferSession};
    let probe = load(&dll(), MIXED).unwrap();
    let input = probe.kinematic_snapshot().unwrap().into_model().unwrap();
    let session = TransferSession::new(0).unwrap();
    let gpu = DeviceKinematicModel::upload(&session, &input).unwrap();
    drop(probe);
    drop(input);
    drop(session);
    let result = gpu.readback().unwrap();
    assert_eq!(
        (result.nq(), result.nv(), result.nbody(), result.njnt()),
        (13, 11, 6, 4)
    );
    assert_eq!(result.fields(), &mixed_expected());
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires verified MuJoCo DLL and NVIDIA GPU"]
fn uploads_zero_joint_groups_and_preserves_float_bits() {
    use mjwarp_rs::{
        io::DeviceKinematicModel, model::KinematicModelInput, runtime::TransferSession,
    };
    let session = TransferSession::new(0).unwrap();
    for bytes in [EMPTY, JOINTS] {
        let input = load(&dll(), bytes)
            .unwrap()
            .kinematic_snapshot()
            .unwrap()
            .into_model()
            .unwrap();
        let mut fields = input.fields().clone();
        fields.body_pos[0] = -0.0;
        fields.body_quat[0] = 2.0; // Finite bits pass through; no normalization.
        let input = KinematicModelInput::new(input.nv(), fields).unwrap();
        let gpu = DeviceKinematicModel::upload(&session, &input).unwrap();
        let result = gpu.readback().unwrap();
        assert_eq!(result, input);
        assert_eq!(result.fields().body_pos[0].to_bits(), (-0.0_f32).to_bits());
    }
}

fn dll() -> PathBuf {
    PathBuf::from(std::env::var_os("MJWARP_MUJOCO_DLL").expect("set verified MuJoCo DLL path"))
}
fn mock(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("MJWARP_NATIVE_MOCKS").expect("build native test DLLs"))
        .join(format!("{name}.dll"))
}
fn load(path: &std::path::Path, bytes: &[u8]) -> Result<NativeModelProbe, NativeProbeError> {
    // SAFETY: explicit opt-in tests use the verified official DLL, our compiled
    // test doubles and fixed trusted fixtures. Invalid cases only change headers.
    unsafe { NativeModelProbe::load_trusted(path, bytes) }
}

#[test]
#[ignore = "requires verified Windows MuJoCo 3.12.0 DLL"]
fn reads_owned_native_fields_and_keeps_snapshot_after_drop() {
    let probe = load(&dll(), JOINTS).unwrap();
    let info = probe.info();
    info.validate().unwrap();
    assert_eq!(info.native_version, NativeModelInfo::VERSION);
    assert_eq!((info.nq, info.nv, info.nu, info.na), (2, 2, 0, 0));
    assert_eq!(
        (info.nbody, info.njnt, info.ngeom, info.nsensordata),
        (3, 2, 2, 0)
    );
    assert_eq!(
        (info.num_bytes, info.index_bytes, info.size_bytes),
        (8, 4, 8)
    );
    let snapshot = probe.snapshot().unwrap();
    drop(probe);
    assert_eq!(snapshot.qpos0, [0.25, -0.5]);
    assert_eq!(snapshot.body_mass, [0.0, 2.0, 3.0]);
    assert_eq!(snapshot.body_parentid, [0, 0, 1]);
    assert_eq!(snapshot.jnt_type, [2, 3]);
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn accepts_zero_dof_without_dereferencing_empty_fields() {
    let probe = load(&dll(), EMPTY).unwrap();
    let snapshot = probe.snapshot().unwrap();
    assert_eq!(
        (snapshot.info.nq, snapshot.info.nv, snapshot.info.njnt),
        (0, 0, 0)
    );
    assert!(snapshot.qpos0.is_empty());
    assert!(snapshot.jnt_type.is_empty());
    assert_eq!(snapshot.body_mass, [0.0]);
    assert_eq!(snapshot.body_parentid, [0]);
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn rejects_each_output_length_before_any_write() {
    let probe = load(&dll(), JOINTS).unwrap();
    for invalid in 0..4 {
        let mut q = vec![7.0; if invalid == 0 { 1 } else { 2 }];
        let mut m = vec![7.0; if invalid == 1 { 2 } else { 3 }];
        let mut p = vec![7; if invalid == 2 { 2 } else { 3 }];
        let mut j = vec![7; if invalid == 3 { 1 } else { 2 }];
        assert!(matches!(
            probe.copy_core_into(&mut q, &mut m, &mut p, &mut j),
            Err(NativeProbeError::Input(_))
        ));
        assert!(q.iter().chain(&m).all(|&v| v == 7.0));
        assert!(p.iter().chain(&j).all(|&v| v == 7));
    }
}

#[test]
#[ignore = "requires native test DLLs"]
fn rejects_version_and_missing_symbol_before_model_calls() {
    assert!(matches!(
        load(&mock("wrong-version"), JOINTS),
        Err(NativeProbeError::VersionMismatch { actual: 123, .. })
    ));
    assert!(matches!(
        load(&mock("missing-load"), JOINTS),
        Err(NativeProbeError::Native {
            stage: "mj_loadModelBuffer",
            ..
        })
    ));
    assert!(matches!(
        load(&mock("missing-version"), JOINTS),
        Err(NativeProbeError::Native {
            stage: "mj_version",
            ..
        })
    ));
    assert!(matches!(
        load(&mock("missing-delete"), JOINTS),
        Err(NativeProbeError::Native {
            stage: "mj_deleteModel",
            ..
        })
    ));
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn rejects_missing_library_and_safe_invalid_mjb_cases() {
    assert!(matches!(
        load(&dll().with_file_name("does-not-exist.dll"), JOINTS),
        Err(NativeProbeError::Native {
            stage: "load_library",
            ..
        })
    ));
    assert!(matches!(
        load(&dll(), &[]),
        Err(NativeProbeError::InvalidMjbHeader)
    ));
    let mut invalid = JOINTS.to_vec();
    invalid[12..16].copy_from_slice(&123_i32.to_le_bytes());
    assert!(matches!(
        load(&dll(), &invalid),
        Err(NativeProbeError::VersionMismatch { .. })
    ));
    // Truncate before the native allocator: only the validated MJB header remains.
    assert!(matches!(
        load(&dll(), &JOINTS[..20]),
        Err(NativeProbeError::Native {
            stage: "load_mjb",
            ..
        })
    ));
}

#[test]
#[ignore = "requires tracking test DLL and dedicated log path"]
fn deletes_model_before_unloading_its_library() {
    let path = PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    std::fs::write(&path, []).unwrap();
    let probe = load(&mock("tracking"), EMPTY).unwrap();
    let snapshot = probe.snapshot().unwrap();
    assert!(std::fs::read(&path).unwrap().is_empty());
    drop(probe);
    assert_eq!(std::fs::read(&path).unwrap(), b"DU");
    assert_eq!(snapshot.body_parentid, [0]);
}

#[test]
#[ignore = "requires invalid-counts test DLL and dedicated log path"]
fn releases_native_owner_when_count_validation_fails() {
    let path = PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    std::fs::write(&path, []).unwrap();
    assert!(matches!(
        load(&mock("invalid-counts"), EMPTY),
        Err(NativeProbeError::Input(_))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), b"DU");
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires verified MuJoCo DLL and NVIDIA GPU"]
fn uploads_native_snapshot_after_releasing_model_and_library() {
    use mjwarp_rs::{
        io::{DeviceBatch, upload_f64_batch},
        model::BatchLayout,
        runtime::TransferSession,
    };
    let probe = load(&dll(), JOINTS).unwrap();
    let snapshot = probe.snapshot().unwrap();
    drop(probe);
    let session = TransferSession::new(0).unwrap();
    let field = upload_f64_batch(
        &session,
        BatchLayout::new(1, 3, 4).unwrap(),
        &snapshot.body_mass,
    )
    .unwrap();
    let mut mass = [0.0; 3];
    field.read_into(&mut mass).unwrap();
    assert_eq!(mass, [0.0, 2.0, 3.0]);
    let field = upload_f64_batch(
        &session,
        BatchLayout::new(1, 2, 4).unwrap(),
        &snapshot.qpos0,
    )
    .unwrap();
    let mut q = [0.0; 2];
    field.read_into(&mut q).unwrap();
    assert_eq!(q, [0.25, -0.5]);
    let field = DeviceBatch::upload(
        &session,
        BatchLayout::new(1, 3, 4).unwrap(),
        &snapshot.body_parentid,
    )
    .unwrap();
    let mut p = [0; 3];
    field.read_into(&mut p).unwrap();
    assert_eq!(p, [0, 0, 1]);
    let field = DeviceBatch::upload(
        &session,
        BatchLayout::new(1, 2, 4).unwrap(),
        &snapshot.jnt_type,
    )
    .unwrap();
    let mut j = [0; 2];
    field.read_into(&mut j).unwrap();
    assert_eq!(j, [2, 3]);
}
