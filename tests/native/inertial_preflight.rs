//! 独立声明九字段ABI与复制前校验。
use super::{EMPTY, dll, inertial::TREE};
use mjwarp_rs::model::NativeModelInfo;
use std::{ffi::c_void, os::windows::ffi::OsStrExt};

#[repr(C)]
struct Targets {
    schema: u32,
    reserved: u32,
    nbody: u64,
    nv: u64,
    body_ipos: *mut f64,
    body_iquat: *mut f64,
    body_mass: *mut f64,
    body_inertia: *mut f64,
    dof_bodyid: *mut i32,
    dof_jntid: *mut i32,
    dof_parentid: *mut i32,
    dof_armature: *mut f64,
    dof_damping: *mut f64,
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
    fn mjwarp_native_copy_inertial(owner: *const c_void, targets: *const Targets) -> i32;
}
struct Owner(*mut c_void);
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: successful open gives this guard unique model/DLL ownership.
        unsafe { mjwarp_native_close(self.0) };
    }
}
fn open(bytes: &[u8]) -> Owner {
    let mut path: Vec<u16> = dll().as_os_str().encode_wide().collect();
    path.push(0);
    let mut raw = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: verified absolute DLL path and trusted fixed MJB, aligned live DTOs.
    assert_eq!(
        unsafe {
            mjwarp_native_open(
                path.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as i32,
                &mut raw,
                &mut info,
                &mut detail,
            )
        },
        0
    );
    assert!(!raw.is_null());
    Owner(raw)
}
#[test]
#[ignore = "requires verified Windows MuJoCo DLL"]
fn checks_all_inertial_targets_before_writing_and_skips_empty_dofs() {
    assert_eq!(size_of::<Targets>(), 96);
    assert_eq!(align_of::<Targets>(), 8);
    assert_eq!(std::mem::offset_of!(Targets, dof_damping), 88);
    let owner = open(TREE);
    let mut floats: [Vec<f64>; 6] = [
        vec![7.0; 24],
        vec![7.0; 32],
        vec![7.0; 8],
        vec![7.0; 24],
        vec![7.0; 13],
        vec![7.0; 13],
    ];
    let mut ints: [Vec<i32>; 3] = std::array::from_fn(|_| vec![7; 13]);
    for case in 0..15 {
        let mut t = Targets {
            schema: 1,
            reserved: 0,
            nbody: 8,
            nv: 13,
            body_ipos: floats[0].as_mut_ptr(),
            body_iquat: floats[1].as_mut_ptr(),
            body_mass: floats[2].as_mut_ptr(),
            body_inertia: floats[3].as_mut_ptr(),
            dof_armature: floats[4].as_mut_ptr(),
            dof_damping: floats[5].as_mut_ptr(),
            dof_bodyid: ints[0].as_mut_ptr(),
            dof_jntid: ints[1].as_mut_ptr(),
            dof_parentid: ints[2].as_mut_ptr(),
        };
        match case {
            0 => t.schema = 2,
            1 => t.reserved = 1,
            2 => t.nbody = 7,
            3 => t.nv = 12,
            4 => t.body_ipos = std::ptr::null_mut(),
            5 => t.body_iquat = std::ptr::null_mut(),
            6 => t.body_mass = std::ptr::null_mut(),
            7 => t.body_inertia = std::ptr::null_mut(),
            8 => t.dof_bodyid = std::ptr::null_mut(),
            9 => t.dof_jntid = std::ptr::null_mut(),
            10 => t.dof_parentid = std::ptr::null_mut(),
            11 => t.dof_armature = std::ptr::null_mut(),
            12 => t.dof_damping = std::ptr::null_mut(),
            _ => {}
        }
        let raw_owner = if case == 13 {
            std::ptr::null()
        } else {
            owner.0
        };
        let raw_targets = if case == 14 { std::ptr::null() } else { &t };
        // SAFETY: live unique model and disjoint adequately sized arrays. Invalid
        // count/null fields must trigger preflight before any read or write.
        assert_eq!(
            unsafe { mjwarp_native_copy_inertial(raw_owner, raw_targets) },
            8,
            "case {case}"
        );
        assert!(floats.iter().flatten().all(|&v| v == 7.0), "case {case}");
        assert!(ints.iter().flatten().all(|&v| v == 7), "case {case}");
    }
    let owner = open(EMPTY);
    let t = Targets {
        schema: 1,
        reserved: 0,
        nbody: 1,
        nv: 0,
        body_ipos: floats[0].as_mut_ptr(),
        body_iquat: floats[1].as_mut_ptr(),
        body_mass: floats[2].as_mut_ptr(),
        body_inertia: floats[3].as_mut_ptr(),
        dof_bodyid: std::ptr::null_mut(),
        dof_jntid: std::ptr::null_mut(),
        dof_parentid: std::ptr::null_mut(),
        dof_armature: std::ptr::null_mut(),
        dof_damping: std::ptr::null_mut(),
    };
    // SAFETY: zero DOFs allow null targets; body arrays remain disjoint and sized.
    assert_eq!(unsafe { mjwarp_native_copy_inertial(owner.0, &t) }, 0);
    assert_eq!(&floats[0][..3], &[0.0; 3]);
    assert_eq!(&floats[1][..4], &[1.0, 0.0, 0.0, 0.0]);
    assert_eq!(floats[2][0], 0.0);
    assert_eq!(&floats[3][..3], &[0.0; 3]);
    for (values, used) in floats.iter().zip([3, 4, 1, 3, 0, 0]) {
        assert!(values[used..].iter().all(|&v| v == 7.0));
    }
    assert!(ints.iter().flatten().all(|&v| v == 7));
}
