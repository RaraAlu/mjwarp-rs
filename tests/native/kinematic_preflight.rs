//! 独立声明桥接ABI，检查复制前校验。
use super::{MIXED, NativePathChar, native_path};
use mjwarp_rs::model::NativeModelInfo;
use std::ffi::c_void;

#[repr(C)]
struct Targets {
    schema: u32,
    reserved: u32,
    nq: u64,
    nbody: u64,
    njnt: u64,
    qpos0: *mut f64,
    body_parentid: *mut i32,
    body_jntadr: *mut i32,
    body_jntnum: *mut i32,
    body_pos: *mut f64,
    body_quat: *mut f64,
    jnt_type: *mut i32,
    jnt_bodyid: *mut i32,
    jnt_qposadr: *mut i32,
    jnt_dofadr: *mut i32,
    jnt_pos: *mut f64,
    jnt_axis: *mut f64,
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
    fn mjwarp_native_copy_kinematic(owner: *const c_void, targets: *const Targets) -> i32;
}
struct Owner(*mut c_void);
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: the successful open gives this guard unique model/DLL ownership.
        unsafe { mjwarp_native_close(self.0) };
    }
}

#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn checks_all_bridge_counts_and_targets_before_any_output_write() {
    assert_eq!(size_of::<Targets>(), 128);
    assert_eq!(std::mem::offset_of!(Targets, jnt_axis), 120);
    let path = native_path();
    let mut raw = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: verified absolute DLL path and trusted fixed MJB; output DTOs are
    // aligned and live. No malformed native parsing occurs in this test.
    let status = unsafe {
        mjwarp_native_open(
            path.as_ptr(),
            MIXED.as_ptr().cast(),
            MIXED.len() as i32,
            &mut raw,
            &mut info,
            &mut detail,
        )
    };
    assert_eq!(status, 0);
    let owner = Owner(raw);
    assert!(!owner.0.is_null());
    assert_eq!((info.nq, info.nbody, info.njnt), (13, 6, 4));
    let mut floats: [Vec<f64>; 5] = [
        vec![7.0; 13],
        vec![7.0; 18],
        vec![7.0; 24],
        vec![7.0; 12],
        vec![7.0; 12],
    ];
    let mut ints: [Vec<i32>; 7] = std::array::from_fn(|i| vec![7; if i < 3 { 6 } else { 4 }]);
    for case in 0..17 {
        let mut t = Targets {
            schema: 1,
            reserved: 0,
            nq: 13,
            nbody: 6,
            njnt: 4,
            qpos0: floats[0].as_mut_ptr(),
            body_pos: floats[1].as_mut_ptr(),
            body_quat: floats[2].as_mut_ptr(),
            jnt_pos: floats[3].as_mut_ptr(),
            jnt_axis: floats[4].as_mut_ptr(),
            body_parentid: ints[0].as_mut_ptr(),
            body_jntadr: ints[1].as_mut_ptr(),
            body_jntnum: ints[2].as_mut_ptr(),
            jnt_type: ints[3].as_mut_ptr(),
            jnt_bodyid: ints[4].as_mut_ptr(),
            jnt_qposadr: ints[5].as_mut_ptr(),
            jnt_dofadr: ints[6].as_mut_ptr(),
        };
        match case {
            0 => t.schema = 2,
            1 => t.reserved = 1,
            2 => t.nq = 12,
            3 => t.nbody = 5,
            4 => t.njnt = 3,
            5 => t.qpos0 = std::ptr::null_mut(),
            6 => t.body_parentid = std::ptr::null_mut(),
            7 => t.body_jntadr = std::ptr::null_mut(),
            8 => t.body_jntnum = std::ptr::null_mut(),
            9 => t.body_pos = std::ptr::null_mut(),
            10 => t.body_quat = std::ptr::null_mut(),
            11 => t.jnt_type = std::ptr::null_mut(),
            12 => t.jnt_bodyid = std::ptr::null_mut(),
            13 => t.jnt_qposadr = std::ptr::null_mut(),
            14 => t.jnt_dofadr = std::ptr::null_mut(),
            15 => t.jnt_pos = std::ptr::null_mut(),
            _ => t.jnt_axis = std::ptr::null_mut(),
        }
        // SAFETY: live unique model and separate, adequately sized target arrays.
        // Deliberately invalid count/null fields must trigger the bridge's preflight
        // before it can dereference or copy any target.
        assert_eq!(
            unsafe { mjwarp_native_copy_kinematic(owner.0, &t) },
            8,
            "case {case}"
        );
        assert!(floats.iter().flatten().all(|&v| v == 7.0), "case {case}");
        assert!(ints.iter().flatten().all(|&v| v == 7), "case {case}");
    }
}
