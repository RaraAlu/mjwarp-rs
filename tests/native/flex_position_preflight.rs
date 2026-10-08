//! 独立声明柔体字段ABI。
//! 失败路径不得写入任何目标。
use super::{EMPTY, dll, flex_position::FLEX};
use mjwarp_rs::model::NativeModelInfo;
use std::{ffi::c_void, os::windows::ffi::OsStrExt};
#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq)]
struct Counts {
    schema: u32,
    reserved: u32,
    nflex: i64,
    nflexnode: i64,
    nflexvert: i64,
}
#[repr(C)]
struct Targets {
    schema: u32,
    reserved: u32,
    nflex: u64,
    nflexnode: u64,
    nflexvert: u64,
    flex_interp: *mut i32,
    flex_cellnum: *mut i32,
    flex_nodeadr: *mut i32,
    flex_nodenum: *mut i32,
    flex_vertadr: *mut i32,
    flex_vertnum: *mut i32,
    flex_centered: *mut u8,
    flex_nodebodyid: *mut i32,
    flex_vertbodyid: *mut i32,
    flex_node: *mut f64,
    flex_vert: *mut f64,
    flex_vert0: *mut f64,
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
    fn mjwarp_native_flex_position_info(owner: *const c_void, info: *mut Counts) -> i32;
    fn mjwarp_native_copy_flex_position(owner: *const c_void, targets: *const Targets) -> i32;
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
    let mut owner = std::ptr::null_mut();
    let mut info = NativeModelInfo::default();
    let mut detail = 0;
    // SAFETY: trusted fixed MJB, verified absolute DLL path and live aligned DTOs.
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
fn preflights_all_flex_targets_before_writing_and_skips_zero_counts() {
    assert_eq!(size_of::<Counts>(), 32);
    assert_eq!(size_of::<Targets>(), 128);
    assert_eq!(std::mem::offset_of!(Targets, flex_centered), 80);
    assert_eq!(std::mem::offset_of!(Targets, flex_vert0), 120);
    let owner = open(FLEX);
    let mut c = Counts::default();
    // SAFETY: live owner and unique aligned output DTO.
    assert_eq!(
        unsafe { mjwarp_native_flex_position_info(owner.0, &mut c) },
        0
    );
    assert_eq!(
        c,
        Counts {
            schema: 1,
            reserved: 0,
            nflex: 4,
            nflexnode: 20,
            nflexvert: 80
        }
    );
    let saved = Counts { ..c };
    // SAFETY: null owner must trigger preflight before touching the valid DTO.
    assert_eq!(
        unsafe { mjwarp_native_flex_position_info(std::ptr::null(), &mut c) },
        8
    );
    assert_eq!(c, saved);
    // SAFETY: null target must trigger preflight before touching model data.
    assert_eq!(
        unsafe { mjwarp_native_flex_position_info(owner.0, std::ptr::null_mut()) },
        8
    );
    let mut ints: [Vec<i32>; 8] = std::array::from_fn(|_| vec![7; 81]);
    let mut floats: [Vec<f64>; 3] = std::array::from_fn(|_| vec![7.0; 241]);
    let mut flags = [7_u8; 5];
    for case in 0..22 {
        let mut t = Targets {
            schema: 1,
            reserved: 0,
            nflex: 4,
            nflexnode: 20,
            nflexvert: 80,
            flex_interp: ints[0].as_mut_ptr(),
            flex_cellnum: ints[1].as_mut_ptr(),
            flex_nodeadr: ints[2].as_mut_ptr(),
            flex_nodenum: ints[3].as_mut_ptr(),
            flex_vertadr: ints[4].as_mut_ptr(),
            flex_vertnum: ints[5].as_mut_ptr(),
            flex_centered: flags.as_mut_ptr(),
            flex_nodebodyid: ints[6].as_mut_ptr(),
            flex_vertbodyid: ints[7].as_mut_ptr(),
            flex_node: floats[0].as_mut_ptr(),
            flex_vert: floats[1].as_mut_ptr(),
            flex_vert0: floats[2].as_mut_ptr(),
        };
        match case {
            0 => t.schema = 2,
            1 => t.reserved = 1,
            2 => t.nflex = 3,
            3 => t.nflexnode = 19,
            4 => t.nflexvert = 79,
            5 => t.flex_interp = std::ptr::null_mut(),
            6 => t.flex_cellnum = std::ptr::null_mut(),
            7 => t.flex_nodeadr = std::ptr::null_mut(),
            8 => t.flex_nodenum = std::ptr::null_mut(),
            9 => t.flex_vertadr = std::ptr::null_mut(),
            10 => t.flex_vertnum = std::ptr::null_mut(),
            11 => t.flex_centered = std::ptr::null_mut(),
            12 => t.flex_nodebodyid = std::ptr::null_mut(),
            13 => t.flex_vertbodyid = std::ptr::null_mut(),
            14 => t.flex_node = std::ptr::null_mut(),
            15 => t.flex_vert = std::ptr::null_mut(),
            16 => t.flex_vert0 = std::ptr::null_mut(),
            19 => t.nflex = u64::MAX,
            20 => t.nflexnode = u64::MAX,
            21 => t.nflexvert = u64::MAX,
            _ => {}
        }
        let o = if case == 17 {
            std::ptr::null()
        } else {
            owner.0
        };
        let target = if case == 18 { std::ptr::null() } else { &t };
        // SAFETY: disjoint sized arrays and live owner. Invalid fields must fail
        // preflight before the bridge performs any reads or writes.
        assert_eq!(
            unsafe { mjwarp_native_copy_flex_position(o, target) },
            8,
            "case {case}"
        );
        assert!(ints.iter().flatten().all(|&v| v == 7), "case {case}");
        assert!(floats.iter().flatten().all(|&v| v == 7.0), "case {case}");
        assert_eq!(flags, [7; 5], "case {case}");
    }
    let owner = open(EMPTY);
    let t = Targets {
        schema: 1,
        reserved: 0,
        nflex: 0,
        nflexnode: 0,
        nflexvert: 0,
        flex_interp: std::ptr::null_mut(),
        flex_cellnum: std::ptr::null_mut(),
        flex_nodeadr: std::ptr::null_mut(),
        flex_nodenum: std::ptr::null_mut(),
        flex_vertadr: std::ptr::null_mut(),
        flex_vertnum: std::ptr::null_mut(),
        flex_centered: std::ptr::null_mut(),
        flex_nodebodyid: std::ptr::null_mut(),
        flex_vertbodyid: std::ptr::null_mut(),
        flex_node: std::ptr::null_mut(),
        flex_vert: std::ptr::null_mut(),
        flex_vert0: std::ptr::null_mut(),
    };
    // SAFETY: zero counts allow null pointers; the owner retains a valid empty model.
    assert_eq!(unsafe { mjwarp_native_copy_flex_position(owner.0, &t) }, 0);
}
