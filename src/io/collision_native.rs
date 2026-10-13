//! 原生几何字段的独占捕获。
//! 本模块不调用原生物理计算。
use super::{NativePrimitiveGeometrySnapshot, fields::native_staging};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::NativeModelInfo,
};
use std::ffi::c_void;

#[repr(C)]
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
const _: () = {
    assert!(size_of::<Targets>() == 120 && align_of::<Targets>() == 8);
    assert!(std::mem::offset_of!(Targets, ngeom) == 8);
    assert!(std::mem::offset_of!(Targets, geom_type) == 16);
    assert!(std::mem::offset_of!(Targets, geom_solmix) == 64);
    assert!(std::mem::offset_of!(Targets, geom_gap) == 112);
};
unsafe extern "C" {
    fn mjwarp_native_copy_primitive_geometry(owner: *const c_void, targets: *const Targets) -> i32;
}

/// 调用方须持有同一原生模型与库。
pub(super) unsafe fn capture(
    owner: *const c_void,
    info: NativeModelInfo,
) -> Result<NativePrimitiveGeometrySnapshot, NativeProbeError> {
    info.validate()?;
    let n = usize::try_from(info.ngeom).map_err(|_| InputError::Overflow { field: "ngeom" })?;
    if n > i32::MAX as usize / 5 {
        return Err(InputError::Overflow {
            field: "primitive_geometry",
        }
        .into());
    }
    // The five-wide bound also protects all smaller field multiplications.
    let mut s = NativePrimitiveGeometrySnapshot {
        info,
        geom_type: native_staging(n)?,
        geom_bodyid: native_staging(n)?,
        geom_contype: native_staging(n)?,
        geom_conaffinity: native_staging(n)?,
        geom_condim: native_staging(n)?,
        geom_priority: native_staging(n)?,
        geom_solmix: native_staging(n)?,
        geom_solref: native_staging(n * 2)?,
        geom_solimp: native_staging(n * 5)?,
        geom_size: native_staging(n * 3)?,
        geom_friction: native_staging(n * 3)?,
        geom_margin: native_staging(n)?,
        geom_gap: native_staging(n)?,
    };
    let t = Targets {
        schema: 1,
        reserved: 0,
        ngeom: n as u64,
        geom_type: s.geom_type.as_mut_ptr(),
        geom_bodyid: s.geom_bodyid.as_mut_ptr(),
        geom_contype: s.geom_contype.as_mut_ptr(),
        geom_conaffinity: s.geom_conaffinity.as_mut_ptr(),
        geom_condim: s.geom_condim.as_mut_ptr(),
        geom_priority: s.geom_priority.as_mut_ptr(),
        geom_solmix: s.geom_solmix.as_mut_ptr(),
        geom_solref: s.geom_solref.as_mut_ptr(),
        geom_solimp: s.geom_solimp.as_mut_ptr(),
        geom_size: s.geom_size.as_mut_ptr(),
        geom_friction: s.geom_friction.as_mut_ptr(),
        geom_margin: s.geom_margin.as_mut_ptr(),
        geom_gap: s.geom_gap.as_mut_ptr(),
    };
    // SAFETY: caller retains the owner; all arrays are independent owned vectors,
    // correctly aligned and sized. Native code preflights every field before copy.
    let status = unsafe { mjwarp_native_copy_primitive_geometry(owner, &t) };
    if status != 0 {
        return Err(NativeProbeError::Native {
            stage: "copy_primitive_geometry",
            code: status as u32,
        });
    }
    Ok(s)
}
