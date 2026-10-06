#![cfg(feature = "native-model-probe")]

use mjwarp_rs::{diagnostics::NativeProbeError, io::NativeModelProbe, model::NativeModelInfo};
use std::path::PathBuf;

const JOINTS: &[u8] = include_bytes!("../fixtures/native-probe/two-joints.mjb");
const EMPTY: &[u8] = include_bytes!("../fixtures/native-probe/zero-dof.mjb");

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
