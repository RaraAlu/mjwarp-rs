//! 新原生捕获功能；不执行物理或GPU。
#![cfg(feature = "native-model-probe")]
use mjwarp_rs::{diagnostics::NativeProbeError, io::NativeModelProbe, model::NativeModelInfo};
use std::{
    ffi::c_void,
    path::{Path, PathBuf},
};

const MJB: &[u8] = include_bytes!("../fixtures/collision-reference-input/plane-sphere.mjb");
fn dll() -> PathBuf {
    std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into()
}
fn mock(name: &str) -> PathBuf {
    let suffix = if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };
    PathBuf::from(std::env::var_os("MJWARP_COLLISION_MOCKS").unwrap())
        .join(format!("{name}.{suffix}"))
}
fn load(path: &Path) -> Result<NativeModelProbe, NativeProbeError> {
    // SAFETY: this opt-in target uses the verified 3.12 library or its local
    // reader-only doubles, with the fixed, trusted frozen MJB.
    unsafe { NativeModelProbe::load_trusted(path, MJB) }
}

#[test]
#[ignore = "需要可信原生DLL与定向测试库"]
fn captures_all_thirteen_real_fields_and_outlives_native_owner() {
    let native = load(&dll()).unwrap();
    let snapshot = native.primitive_geometry_snapshot().unwrap();
    assert_eq!(
        (
            snapshot.info.nq,
            snapshot.info.nv,
            snapshot.info.nbody,
            snapshot.info.ngeom
        ),
        (7, 6, 2, 2)
    );
    // Geometry parameters come from the frozen XML and native 3.12 defaults;
    // no dynamics, contact generation, or rounded native TXT is a reference.
    assert_eq!(snapshot.geom_type, [0, 2]);
    assert_eq!(snapshot.geom_bodyid, [0, 1]);
    assert_eq!(snapshot.geom_contype, [1, 1]);
    assert_eq!(snapshot.geom_conaffinity, [1, 1]);
    assert_eq!(snapshot.geom_condim, [3, 3]);
    assert_eq!(snapshot.geom_priority, [0, 0]);
    assert_eq!(snapshot.geom_solmix, [1.0, 1.0]);
    assert_eq!(snapshot.geom_solref, [0.02, 1.0, 0.02, 1.0]);
    assert_eq!(
        snapshot.geom_solimp,
        [0.9, 0.95, 0.001, 0.5, 2.0, 0.9, 0.95, 0.001, 0.5, 2.0]
    );
    assert_eq!(snapshot.geom_size, [2.0, 2.0, 0.125, 0.25, 0.0, 0.0]);
    assert_eq!(
        snapshot.geom_friction,
        [1.0, 0.005, 0.0001, 1.0, 0.005, 0.0001]
    );
    assert_eq!(snapshot.geom_margin, [0.125, 0.125]);
    assert_eq!(snapshot.geom_gap, [0.0, 0.0]);
    let again = native.primitive_geometry_snapshot().unwrap();
    drop(native);
    assert_eq!(snapshot, again);
    let input = snapshot.into_input().unwrap();
    assert_eq!(input.ngeom(), 2);
    assert_eq!(input.fields().geom_margin, [0.125, 0.125]);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Targets {
    schema: u32,
    reserved: u32,
    ngeom: u64,
    geom_type: *mut i32,
    geom_bodyid: *mut i32,
    geom_contype: *mut i32,
    geom_conaffinity: *mut i32,
    geom_condim: *mut i32,
    geom_priority: *mut i32,
    geom_solmix: *mut f64,
    geom_solref: *mut f64,
    geom_solimp: *mut f64,
    geom_size: *mut f64,
    geom_friction: *mut f64,
    geom_margin: *mut f64,
    geom_gap: *mut f64,
}
unsafe extern "C" {
    fn mjwarp_native_open(
        path: *const NativePathChar,
        mjb: *const c_void,
        bytes: i32,
        owner: *mut *mut c_void,
        info: *mut NativeModelInfo,
        detail: *mut u32,
    ) -> i32;
    fn mjwarp_native_close(owner: *mut c_void);
    fn mjwarp_native_copy_primitive_geometry(owner: *const c_void, targets: *const Targets) -> i32;
}
#[cfg(target_os = "windows")]
type NativePathChar = u16;
#[cfg(target_os = "linux")]
type NativePathChar = std::ffi::c_char;
struct Owner(*mut c_void);
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: this guard owns exactly one successful native-open result.
        unsafe { mjwarp_native_close(self.0) };
    }
}
fn open(path: &Path) -> Owner {
    #[cfg(target_os = "windows")]
    use std::os::windows::ffi::OsStrExt;
    #[cfg(target_os = "windows")]
    let mut path: Vec<u16> = path.as_os_str().encode_wide().collect();
    #[cfg(target_os = "linux")]
    let mut path: Vec<NativePathChar> = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str()
            .as_bytes()
            .iter()
            .map(|&v| v as NativePathChar)
            .collect()
    };
    path.push(0);
    let mut owner = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: trusted opt-in DLL/MJB; initialized aligned out parameters live
    // for this call. The guard closes even a model with deliberately bad counts.
    assert_eq!(
        unsafe {
            mjwarp_native_open(
                path.as_ptr(),
                MJB.as_ptr().cast(),
                MJB.len() as i32,
                &mut owner,
                &mut info,
                &mut detail,
            )
        },
        0
    );
    assert!(!owner.is_null());
    Owner(owner)
}
fn targets(slots: &mut [Vec<u64>]) -> Targets {
    Targets {
        schema: 1,
        reserved: 0,
        ngeom: 2,
        geom_type: slots[0].as_mut_ptr().cast(),
        geom_bodyid: slots[1].as_mut_ptr().cast(),
        geom_contype: slots[2].as_mut_ptr().cast(),
        geom_conaffinity: slots[3].as_mut_ptr().cast(),
        geom_condim: slots[4].as_mut_ptr().cast(),
        geom_priority: slots[5].as_mut_ptr().cast(),
        geom_solmix: slots[6].as_mut_ptr().cast(),
        geom_solref: slots[7].as_mut_ptr().cast(),
        geom_solimp: slots[8].as_mut_ptr().cast(),
        geom_size: slots[9].as_mut_ptr().cast(),
        geom_friction: slots[10].as_mut_ptr().cast(),
        geom_margin: slots[11].as_mut_ptr().cast(),
        geom_gap: slots[12].as_mut_ptr().cast(),
    }
}

#[test]
#[ignore = "需要可信原生DLL与定向测试库"]
fn rejects_all_target_pointers_and_count_errors_before_any_write() {
    assert_eq!((size_of::<Targets>(), align_of::<Targets>()), (120, 8));
    let owner = open(&dll());
    let mut slots = vec![vec![0x5a5a_5a5a_5a5a_5a5a; 10]; 13];
    let original = slots.clone();
    let good = targets(&mut slots);
    macro_rules! null_fields {
        ($($field:ident),+ $(,)?) => { $(
            let mut bad = good; bad.$field = std::ptr::null_mut();
            // SAFETY: all other storage has valid capacity; the null field must
            // reject before writing any sentinel or dereferencing that field.
            assert_eq!(unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &bad) }, 8);
            assert_eq!(slots, original);
        )+ };
    }
    null_fields!(
        geom_type,
        geom_bodyid,
        geom_contype,
        geom_conaffinity,
        geom_condim,
        geom_priority,
        geom_solmix,
        geom_solref,
        geom_solimp,
        geom_size,
        geom_friction,
        geom_margin,
        geom_gap
    );
    for count in [0, 1, 3, u64::MAX] {
        let mut bad = good;
        bad.ngeom = count;
        // SAFETY: count mismatch must reject before accessing valid storage.
        assert_eq!(
            unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &bad) },
            8
        );
        assert_eq!(slots, original);
    }
    for (schema, reserved) in [(0, 0), (2, 0), (1, 1)] {
        let mut bad = good;
        bad.schema = schema;
        bad.reserved = reserved;
        // SAFETY: malformed DTO must reject before writing valid sentinels.
        assert_eq!(
            unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &bad) },
            8
        );
        assert_eq!(slots, original);
    }
    // SAFETY: null owner/DTO are explicitly checked before dereferencing.
    assert_eq!(
        unsafe { mjwarp_native_copy_primitive_geometry(std::ptr::null(), &good) },
        8
    );
    // SAFETY: same explicit null-DTO preflight.
    assert_eq!(
        unsafe { mjwarp_native_copy_primitive_geometry(owner.0, std::ptr::null()) },
        8
    );
    assert_eq!(slots, original);
    // SAFETY: all thirteen disjoint buffers have correct alignment and capacity.
    assert_eq!(
        unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &good) },
        0
    );
}

#[test]
#[ignore = "需要定向原生测试库"]
fn rejects_every_missing_source_and_invalid_native_count_transactionally() {
    for index in 0..13 {
        let path = mock(&format!("missing-{index}"));
        let native = load(&path).unwrap();
        assert!(matches!(
            native.primitive_geometry_snapshot(),
            Err(NativeProbeError::Native {
                stage: "copy_primitive_geometry",
                code: 8
            })
        ));
        let owner = open(&path);
        let mut slots = vec![vec![0x5a5a_5a5a_5a5a_5a5a; 10]; 13];
        let original = slots.clone();
        let t = targets(&mut slots);
        // SAFETY: trusted double has exactly one missing source; every target
        // is valid and preflight must reject before the first memcpy.
        assert_eq!(
            unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &t) },
            8
        );
        assert_eq!(slots, original);
    }
    for name in ["negative", "oversized"] {
        let path = mock(name);
        assert!(load(&path).is_err());
        let owner = open(&path);
        let mut slots = vec![vec![0x5a5a_5a5a_5a5a_5a5a; 10]; 13];
        let original = slots.clone();
        let mut t = targets(&mut slots);
        if name == "oversized" {
            t.ngeom = i64::MAX as u64;
        }
        // SAFETY: deliberately invalid counts must reject without reading
        // beyond the double's small arrays or changing any target.
        assert_eq!(
            unsafe { mjwarp_native_copy_primitive_geometry(owner.0, &t) },
            8
        );
        assert_eq!(slots, original);
    }
    let log = PathBuf::from(std::env::var_os("MJWARP_COLLISION_LIFETIME_LOG").unwrap());
    let before = std::fs::read(&log).unwrap_or_default();
    assert!(matches!(
        load(&mock("wrong-version")),
        Err(NativeProbeError::VersionMismatch { actual: 1234, .. })
    ));
    let after = std::fs::read(&log).unwrap();
    // The wrong-version loader would append L; only DLL unload may occur.
    assert_eq!(&after[before.len()..], b"U");
}

#[test]
#[ignore = "需要定向原生测试库"]
fn empty_capture_and_drop_order_leave_owned_snapshot_independent() {
    let native = load(&mock("empty")).unwrap();
    let snapshot = native.primitive_geometry_snapshot().unwrap();
    drop(native);
    assert_eq!(snapshot.info.ngeom, 0);
    assert_eq!(snapshot.into_input().unwrap().ngeom(), 0);
    let log = PathBuf::from(std::env::var_os("MJWARP_COLLISION_LIFETIME_LOG").unwrap());
    let before = std::fs::read(&log).unwrap_or_default();
    let native = load(&mock("valid")).unwrap();
    let snapshot = native.primitive_geometry_snapshot().unwrap();
    assert_eq!(std::fs::read(&log).unwrap_or_default(), before);
    drop(native);
    let after = std::fs::read(&log).unwrap();
    assert_eq!(&after[before.len()..], b"DU");
    assert_eq!(snapshot.geom_size, [2.0, 2.0, 0.125, 0.25, 0.0, 0.0]);
    assert_eq!(snapshot.into_input().unwrap().ngeom(), 2);
}
