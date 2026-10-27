//! S1碰撞输入探针的原始ABI预检：非法参数必须返回8且不写半成品。
//! 只被`tests/g10_p5_collision_inputs_snapshot.rs`包含；不进入产品路径。

use mjwarp_rs::model::NativeModelInfo;
use std::{ffi::c_void, path::Path};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct FieldInfo {
    kind: u32,
    reserved: u32,
    count: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct InfoData {
    schema: u32,
    reserved: u32,
    nq: i64,
    ngeom: i64,
    nmesh: i64,
    nmeshvert: i64,
    nmeshgraph: i64,
    nmeshpoly: i64,
    nmeshpolyvert: i64,
    nmeshpolymap: i64,
    nhfield: i64,
    nhfielddata: i64,
    noct: i64,
}

#[cfg(target_os = "windows")]
type NativePathChar = u16;
#[cfg(target_os = "linux")]
type NativePathChar = std::ffi::c_char;

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
    fn mjwarp_native_collision_inputs_info(owner: *const c_void, info: *mut InfoData) -> i32;
    fn mjwarp_native_collision_inputs_field_info(
        owner: *const c_void,
        id: u32,
        info: *mut FieldInfo,
    ) -> i32;
    fn mjwarp_native_copy_collision_inputs_field(
        owner: *const c_void,
        id: u32,
        info: *const FieldInfo,
        target: *mut c_void,
    ) -> i32;
    fn mjwarp_native_collision_inputs_create(owner: *const c_void, output: *mut *mut c_void)
    -> i32;
    fn mjwarp_native_collision_inputs_set_qpos(
        owner: *const c_void,
        data: *mut c_void,
        qpos: *const f64,
        nq: u64,
    ) -> i32;
    fn mjwarp_native_collision_inputs_copy_xpos_xmat(
        owner: *const c_void,
        data: *const c_void,
        geom_xpos: *mut f32,
        geom_xmat: *mut f32,
        ngeom: u64,
    ) -> i32;
    fn mjwarp_native_collision_inputs_destroy(owner: *const c_void, data: *mut c_void) -> i32;
}

fn path_bytes(path: &Path) -> Vec<NativePathChar> {
    #[cfg(target_os = "windows")]
    let mut bytes: Vec<NativePathChar> = {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str().encode_wide().collect()
    };
    #[cfg(target_os = "linux")]
    let mut bytes: Vec<NativePathChar> = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str()
            .as_bytes()
            .iter()
            .map(|&byte| byte as NativePathChar)
            .collect()
    };
    bytes.push(0);
    bytes
}

struct Owner(*mut c_void);
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: this guard owns exactly one successful native-open result.
        unsafe { mjwarp_native_close(self.0) };
    }
}

fn open_raw(bytes: &[u8]) -> Owner {
    let path = path_bytes(&super::oracle_support::dll());
    let mut owner = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: trusted opt-in DLL/MJB; initialized aligned out parameters.
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

/// 运行全部原始ABI负例；句柄与所有者在返回前按序释放。
pub fn run(mjb: &[u8]) {
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!((size_of::<FieldInfo>(), align_of::<FieldInfo>()), (16, 8));
    assert_eq!(offset_of!(FieldInfo, count), 8);
    assert_eq!((size_of::<InfoData>(), align_of::<InfoData>()), (96, 8));
    assert_eq!(offset_of!(InfoData, noct), 88);
    let owner = open_raw(mjb);
    let foreign = open_raw(mjb);
    let mut info = InfoData::default();
    // SAFETY: explicit null-argument preflight cases.
    unsafe {
        assert_eq!(
            mjwarp_native_collision_inputs_info(std::ptr::null(), &mut info),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_info(owner.0, std::ptr::null_mut()),
            8
        );
        assert_eq!(mjwarp_native_collision_inputs_info(owner.0, &mut info), 0);
    }
    assert_eq!((info.schema, info.reserved), (1, 0));
    let mut field = FieldInfo::default();
    // SAFETY: unknown field IDs and null DTOs must not dereference native data.
    unsafe {
        for id in [32_u32, 33, u32::MAX] {
            assert_eq!(
                mjwarp_native_collision_inputs_field_info(owner.0, id, &mut field),
                8
            );
        }
        assert_eq!(
            mjwarp_native_collision_inputs_field_info(owner.0, 2, std::ptr::null_mut()),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_field_info(std::ptr::null(), 2, &mut field),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_field_info(owner.0, 2, &mut field),
            0
        );
    }
    assert_eq!(field.kind, 2);
    assert_eq!(field.count, info.ngeom * 3);
    // Copy preflight: data fields, null targets, wrong kind/count/reserved.
    let expected = usize::try_from(field.count).unwrap();
    let mut target = vec![0x5a5a_5a5a_i64; expected];
    let original = target.clone();
    // SAFETY: every rejected case must leave the sentinel buffer untouched.
    unsafe {
        for id in [0_u32, 1] {
            assert_eq!(
                mjwarp_native_copy_collision_inputs_field(
                    owner.0,
                    id,
                    &field,
                    target.as_mut_ptr().cast(),
                ),
                8
            );
        }
        let mut bad = field;
        bad.count += 1;
        assert_eq!(
            mjwarp_native_copy_collision_inputs_field(owner.0, 2, &bad, target.as_mut_ptr().cast()),
            8
        );
        let mut bad = field;
        bad.kind = 1;
        assert_eq!(
            mjwarp_native_copy_collision_inputs_field(owner.0, 2, &bad, target.as_mut_ptr().cast()),
            8
        );
        let mut bad = field;
        bad.reserved = 1;
        assert_eq!(
            mjwarp_native_copy_collision_inputs_field(owner.0, 2, &bad, target.as_mut_ptr().cast()),
            8
        );
        assert_eq!(
            mjwarp_native_copy_collision_inputs_field(owner.0, 2, &field, std::ptr::null_mut()),
            8
        );
    }
    assert_eq!(target, original);
    // SAFETY: the real copy writes exactly ngeom*3 doubles into the sentinel buffer.
    unsafe {
        assert_eq!(
            mjwarp_native_copy_collision_inputs_field(
                owner.0,
                2,
                &field,
                target.as_mut_ptr().cast()
            ),
            0
        );
    }
    assert_ne!(target, original);
    // mjData handle: null outputs, wrong qpos lengths and foreign-owner destroy.
    let mut handle = std::ptr::null_mut();
    // SAFETY: explicit null-argument preflight cases.
    unsafe {
        assert_eq!(
            mjwarp_native_collision_inputs_create(std::ptr::null(), &mut handle),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_create(owner.0, std::ptr::null_mut()),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_create(owner.0, &mut handle),
            0
        );
    }
    assert!(!handle.is_null());
    let nq = usize::try_from(info.nq).unwrap();
    let ngeom = usize::try_from(info.ngeom).unwrap();
    let qpos = vec![0.0_f64; nq];
    let mut xpos = vec![0.0_f32; ngeom * 3];
    let mut xmat = vec![0.0_f32; ngeom * 9];
    // SAFETY: every rejected call keeps the handle valid for the final destroy.
    unsafe {
        assert_eq!(
            mjwarp_native_collision_inputs_set_qpos(owner.0, handle, std::ptr::null(), nq as u64),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_set_qpos(
                owner.0,
                handle,
                qpos.as_ptr(),
                (nq + 1) as u64
            ),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_set_qpos(foreign.0, handle, qpos.as_ptr(), nq as u64),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_set_qpos(owner.0, handle, qpos.as_ptr(), nq as u64),
            0
        );
        assert_eq!(
            mjwarp_native_collision_inputs_copy_xpos_xmat(
                owner.0,
                handle,
                std::ptr::null_mut(),
                xmat.as_mut_ptr(),
                ngeom as u64,
            ),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_copy_xpos_xmat(
                owner.0,
                handle,
                xpos.as_mut_ptr(),
                xmat.as_mut_ptr(),
                (ngeom + 1) as u64,
            ),
            8
        );
        assert_eq!(
            mjwarp_native_collision_inputs_copy_xpos_xmat(
                owner.0,
                handle,
                xpos.as_mut_ptr(),
                xmat.as_mut_ptr(),
                ngeom as u64,
            ),
            0
        );
        assert_eq!(mjwarp_native_collision_inputs_destroy(foreign.0, handle), 8);
        assert_eq!(mjwarp_native_collision_inputs_destroy(owner.0, handle), 0);
        assert_eq!(
            mjwarp_native_collision_inputs_destroy(owner.0, std::ptr::null_mut()),
            0
        );
    }
    drop(foreign);
}
