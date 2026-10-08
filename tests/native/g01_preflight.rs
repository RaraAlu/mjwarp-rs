//! G01字段ABI的独立声明与失败验收。
use super::{EMPTY, dll, load, mock};
use mjwarp_rs::{diagnostics::NativeProbeError, model::NativeModelInfo};
use std::{ffi::c_void, os::windows::ffi::OsStrExt};
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
struct FieldInfo {
    kind: u32,
    reserved: u32,
    count: i64,
}
unsafe extern "C" {
    fn mjwarp_native_open(
        path: *const u16,
        mjb: *const c_void,
        bytes: i32,
        owner: *mut *mut c_void,
        info: *mut NativeModelInfo,
        detail: *mut u32,
    ) -> i32;
    fn mjwarp_native_close(owner: *mut c_void);
    fn mjwarp_native_g01_field_info(owner: *const c_void, id: u32, info: *mut FieldInfo) -> i32;
    fn mjwarp_native_copy_g01_field(
        owner: *const c_void,
        id: u32,
        info: *const FieldInfo,
        target: *mut c_void,
    ) -> i32;
}
struct Owner(*mut c_void);
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: the successful open gave this guard exclusive owner lifetime.
        unsafe { mjwarp_native_close(self.0) };
    }
}
fn open() -> Owner {
    let bytes = include_bytes!("../../fixtures/g01/scene.mjb");
    let mut path: Vec<u16> = dll().as_os_str().encode_wide().collect();
    path.push(0);
    let mut owner = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: fixed trusted MJB, verified DLL and live aligned out parameters.
    assert_eq!(
        unsafe {
            mjwarp_native_open(
                path.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as i32,
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
#[test]
#[ignore = "需要可信MuJoCo DLL"]
fn g01_abi_preflights_every_id_kind_count_and_pointer_without_writes() {
    assert_eq!(size_of::<FieldInfo>(), 16);
    assert_eq!(align_of::<FieldInfo>(), 8);
    let owner = open();
    let null = std::ptr::null();
    let mut checks = 0;
    for id in 0..45 {
        let mut info = FieldInfo::default();
        // SAFETY: retained native owner, checked ID and aligned DTO.
        assert_eq!(
            unsafe { mjwarp_native_g01_field_info(owner.0, id, &mut info) },
            0
        );
        assert_eq!(
            info.kind,
            if (24..42).contains(&id) {
                2
            } else if id == 42 {
                3
            } else {
                1
            }
        );
        assert!(info.count >= 0);
        let bytes = info.count as usize
            * if info.kind == 2 {
                8
            } else if info.kind == 3 {
                1
            } else {
                4
            };
        let words = bytes.div_ceil(8).max(1);
        let mut buffer = vec![0x5a5a_5a5a_5a5a_5a5a_u64; words];
        let before = buffer.clone();
        for change in 0..7 {
            let mut bad = info;
            let mut target = buffer.as_mut_ptr().cast();
            let mut model = owner.0;
            let mut bad_id = id;
            match change {
                0 => bad.kind = 0,
                1 => bad.reserved = 1,
                2 => bad.count = -1,
                3 => bad.count = i64::MAX,
                4 => model = std::ptr::null_mut(),
                5 => bad_id = 45,
                _ => bad_id = u32::MAX,
            }
            // SAFETY: allocation covers the valid DTO; every mutation must fail
            // before touching the sentinel. The count mismatch cannot be trusted.
            assert_eq!(
                unsafe { mjwarp_native_copy_g01_field(model, bad_id, &bad, target) },
                8
            );
            assert_eq!(buffer, before);
            checks += 1;
            target = std::ptr::null_mut();
            if info.count > 0 {
                // SAFETY: a required null target must fail without copying.
                assert_eq!(
                    unsafe { mjwarp_native_copy_g01_field(owner.0, id, &info, target) },
                    8
                );
                checks += 1;
            }
        }
        // SAFETY: real DTO selects aligned capacity-checked scalar storage.
        assert_eq!(
            unsafe { mjwarp_native_copy_g01_field(owner.0, id, &info, buffer.as_mut_ptr().cast()) },
            0
        );
        // SAFETY: null info/target are explicitly rejected before memory access.
        assert_eq!(
            unsafe { mjwarp_native_copy_g01_field(owner.0, id, null, std::ptr::null_mut()) },
            8
        );
        assert_eq!(
            unsafe { mjwarp_native_g01_field_info(owner.0, id, std::ptr::null_mut()) },
            8
        );
    }
    assert!(checks >= 315);
}
#[test]
#[ignore = "需要原生测试DLL与独立日志"]
fn g01_missing_sources_and_bad_counts_preserve_native_ownership() {
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    for name in [
        "g01-missing-size",
        "g01-negative-site",
        "g01-oversized-camera",
    ] {
        std::fs::write(&path, []).unwrap();
        let model = load(&mock(name), EMPTY).unwrap();
        let error = model.g01_snapshot().unwrap_err();
        match name {
            "g01-missing-size" => assert_eq!(
                error,
                NativeProbeError::Native {
                    stage: "copy_g01_field",
                    code: 8
                }
            ),
            "g01-negative-site" => assert_eq!(
                error,
                NativeProbeError::Native {
                    stage: "g01_field_info",
                    code: 8
                }
            ),
            _ => assert_eq!(
                error,
                NativeProbeError::Native {
                    stage: "g01_field_info",
                    code: 8
                }
            ),
        }
        assert!(std::fs::read(&path).unwrap().is_empty());
        assert_eq!(model.kinematic_snapshot().unwrap().body_parentid, [0]);
        drop(model);
        assert_eq!(std::fs::read(&path).unwrap(), b"DU");
    }
}
