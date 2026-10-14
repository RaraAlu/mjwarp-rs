//! 原生CSR六字段的独占捕获与校验。
//! 本模块不运行原生物理计算。
use crate::io::fields::{check_length, native_staging};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{NativeModelInfo, SparseMassFields, SparseMassInput},
};
use std::ffi::c_void;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeSparseMassSnapshot {
    pub info: NativeModelInfo,
    pub nc: usize,
    pub ntree: usize,
    pub fields: SparseMassFields,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Counts {
    schema: u32,
    reserved: u32,
    nv: u64,
    nc: u64,
    ntree: u64,
}
#[repr(C)]
struct Targets {
    info: Counts,
    dof_parentid: *mut i32,
    m_rownnz: *mut i32,
    m_rowadr: *mut i32,
    m_colind: *mut i32,
    tree_dofadr: *mut i32,
    tree_dofnum: *mut i32,
}
const _: () = {
    assert!(size_of::<Counts>() == 32 && align_of::<Counts>() == 8);
    assert!(std::mem::offset_of!(Counts, nv) == 8 && std::mem::offset_of!(Counts, ntree) == 24);
    assert!(size_of::<Targets>() == 80 && align_of::<Targets>() == 8);
    assert!(std::mem::offset_of!(Targets, dof_parentid) == 32);
    assert!(
        std::mem::offset_of!(Targets, m_colind) == 56
            && std::mem::offset_of!(Targets, tree_dofnum) == 72
    );
};
unsafe extern "C" {
    fn mjwarp_native_sparse_mass_info(owner: *const c_void, info: *mut Counts) -> i32;
    fn mjwarp_native_copy_sparse_mass(owner: *const c_void, targets: *const Targets) -> i32;
}
fn status(code: i32, stage: &'static str) -> Result<(), NativeProbeError> {
    if code == 0 {
        Ok(())
    } else {
        Err(NativeProbeError::Native {
            stage,
            code: code as u32,
        })
    }
}
fn counts(nv: u64, nc: u64, ntree: u64) -> Result<(usize, usize, usize), NativeProbeError> {
    let mut output = [0usize; 3];
    for (index, value) in [nv, nc, ntree].into_iter().enumerate() {
        if value > (i32::MAX as u64) / 4 {
            return Err(InputError::Overflow {
                field: "native_sparse_mass_count",
            }
            .into());
        }
        output[index] = usize::try_from(value).map_err(|_| InputError::Overflow {
            field: "native_sparse_mass_count",
        })?;
    }
    Ok((output[0], output[1], output[2]))
}
impl NativeSparseMassSnapshot {
    pub fn into_input(self) -> Result<SparseMassInput, NativeProbeError> {
        self.info.validate()?;
        let (nv, nc, ntree) = counts(self.info.nv as u64, self.nc as u64, self.ntree as u64)?;
        for (name, actual, expected) in [
            ("dof_parentid", self.fields.dof_parentid.len(), nv),
            ("M_rownnz", self.fields.m_rownnz.len(), nv),
            ("M_rowadr", self.fields.m_rowadr.len(), nv),
            ("M_colind", self.fields.m_colind.len(), nc),
            ("tree_dofadr", self.fields.tree_dofadr.len(), ntree),
            ("tree_dofnum", self.fields.tree_dofnum.len(), ntree),
        ] {
            check_length(name, expected, actual)?;
        }
        Ok(SparseMassInput::new(self.fields)?)
    }
}
/// # Safety
/// Caller retains the immutable native model and DLL for the full capture.
pub(in crate::io) unsafe fn capture(
    owner: *const c_void,
    info: NativeModelInfo,
) -> Result<NativeSparseMassSnapshot, NativeProbeError> {
    info.validate()?;
    let mut c = Counts::default();
    // SAFETY: Caller keeps owner live; c is initialized, aligned and writable.
    status(
        unsafe { mjwarp_native_sparse_mass_info(owner, &mut c) },
        "sparse_mass_info",
    )?;
    if c.schema != 1 || c.reserved != 0 || c.nv != info.nv as u64 {
        return Err(NativeProbeError::AbiMismatch);
    }
    let (nv, nc, ntree) = counts(c.nv, c.nc, c.ntree)?;
    let mut s = NativeSparseMassSnapshot {
        info,
        nc,
        ntree,
        fields: SparseMassFields {
            dof_parentid: native_staging(nv)?,
            m_rownnz: native_staging(nv)?,
            m_rowadr: native_staging(nv)?,
            m_colind: native_staging(nc)?,
            tree_dofadr: native_staging(ntree)?,
            tree_dofnum: native_staging(ntree)?,
        },
    };
    let t = Targets {
        info: c,
        dof_parentid: s.fields.dof_parentid.as_mut_ptr(),
        m_rownnz: s.fields.m_rownnz.as_mut_ptr(),
        m_rowadr: s.fields.m_rowadr.as_mut_ptr(),
        m_colind: s.fields.m_colind.as_mut_ptr(),
        tree_dofadr: s.fields.tree_dofadr.as_mut_ptr(),
        tree_dofnum: s.fields.tree_dofnum.as_mut_ptr(),
    };
    // SAFETY: Count-checked independent Vec<i32> arrays have exact ABI width.
    // All buffers and the retained owner survive this synchronous preflight/copy.
    status(
        unsafe { mjwarp_native_copy_sparse_mass(owner, &t) },
        "copy_sparse_mass",
    )?;
    Ok(s)
}
