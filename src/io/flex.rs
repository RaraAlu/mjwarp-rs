//! 原生柔体位置字段转换。
//! 本子集不替代完整put_model。
use super::{
    fields::native_staging as staging,
    kinematic::native::{check_info, convert, elements},
};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{FlexPositionFields, FlexPositionModelInput, NativeModelInfo, TendonModelInput},
};
use std::ffi::c_void;

/// 独占十二项原生柔体字段。
/// 标量保留原生f64精度。
/// 快照不借用模型或DLL。
/// 本子集不包含边面与弹性。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeFlexPositionSnapshot {
    pub info: NativeModelInfo,
    pub nflex: i64,
    pub nflexnode: i64,
    pub nflexvert: i64,
    pub flex_interp: Vec<i32>,
    pub flex_cellnum: Vec<i32>,
    pub flex_nodeadr: Vec<i32>,
    pub flex_nodenum: Vec<i32>,
    pub flex_vertadr: Vec<i32>,
    pub flex_vertnum: Vec<i32>,
    /// 仅接收零或一。
    pub flex_centered: Vec<u8>,
    pub flex_nodebodyid: Vec<i32>,
    pub flex_vertbodyid: Vec<i32>,
    pub flex_node: Vec<f64>,
    pub flex_vert: Vec<f64>,
    pub flex_vert0: Vec<f64>,
}

#[repr(C)]
#[derive(Default)]
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
const _: () = {
    assert!(size_of::<Counts>() == 32 && align_of::<Counts>() == 8);
    assert!(std::mem::offset_of!(Counts, nflex) == 8);
    assert!(size_of::<Targets>() == 128 && align_of::<Targets>() == 8);
    assert!(std::mem::offset_of!(Targets, flex_interp) == 32);
    assert!(std::mem::offset_of!(Targets, flex_centered) == 80);
    assert!(std::mem::offset_of!(Targets, flex_vert0) == 120);
};
unsafe extern "C" {
    fn mjwarp_native_flex_position_info(owner: *const c_void, counts: *mut Counts) -> i32;
    fn mjwarp_native_copy_flex_position(owner: *const c_void, targets: *const Targets) -> i32;
}
fn status(stage: &'static str, code: i32) -> Result<(), NativeProbeError> {
    if code != 0 {
        return Err(NativeProbeError::Native {
            stage,
            code: code as u32,
        });
    }
    Ok(())
}
fn check_counts(nflex: i64, nodes: i64, verts: i64) -> Result<(), InputError> {
    for (field, count) in [("nflex", nflex), ("nflexnode", nodes), ("nflexvert", verts)] {
        if count < 0 || count > i64::from(i32::MAX) {
            return Err(InputError::Overflow { field });
        }
        elements(count, 3)?;
    }
    // 先核对GPU展开布局。
    // 拒绝超限计数后才分配。
    let overflow = || InputError::Overflow {
        field: "flex_position_fields",
    };
    nflex
        .checked_mul(9)
        .and_then(|n| n.checked_add(5))
        .and_then(|n| n.checked_add(nodes))
        .and_then(|n| n.checked_add(verts))
        .filter(|&n| n <= i64::from(i32::MAX))
        .ok_or_else(overflow)?;
    verts
        .checked_mul(6)
        .and_then(|v| nodes.checked_mul(3).and_then(|n| v.checked_add(n)))
        .filter(|&n| n <= i64::from(i32::MAX))
        .ok_or_else(overflow)?;
    Ok(())
}
impl NativeFlexPositionSnapshot {
    /// # Safety
    /// owner须拥有对应只读模型。
    /// 调用期间须保持其DLL存活。
    pub(in crate::io) unsafe fn capture(
        owner: *const c_void,
        info: NativeModelInfo,
    ) -> Result<Self, NativeProbeError> {
        check_info(info)?;
        let mut c = Counts::default();
        // SAFETY: caller retains the immutable model/DLL. The initialized count
        // DTO is aligned and writable; the bridge does not expose native pointers.
        status("flex_position_info", unsafe {
            mjwarp_native_flex_position_info(owner, &mut c)
        })?;
        if c.schema != 1 || c.reserved != 0 {
            return Err(NativeProbeError::AbiMismatch);
        }
        check_counts(c.nflex, c.nflexnode, c.nflexvert)?;
        let mut s = Self {
            info,
            nflex: c.nflex,
            nflexnode: c.nflexnode,
            nflexvert: c.nflexvert,
            flex_interp: staging(elements(c.nflex, 1)?)?,
            flex_cellnum: staging(elements(c.nflex, 3)?)?,
            flex_nodeadr: staging(elements(c.nflex, 1)?)?,
            flex_nodenum: staging(elements(c.nflex, 1)?)?,
            flex_vertadr: staging(elements(c.nflex, 1)?)?,
            flex_vertnum: staging(elements(c.nflex, 1)?)?,
            flex_centered: staging(elements(c.nflex, 1)?)?,
            flex_nodebodyid: staging(elements(c.nflexnode, 1)?)?,
            flex_vertbodyid: staging(elements(c.nflexvert, 1)?)?,
            flex_node: staging(elements(c.nflexnode, 3)?)?,
            flex_vert: staging(elements(c.nflexvert, 3)?)?,
            flex_vert0: staging(elements(c.nflexvert, 3)?)?,
        };
        let t = Targets {
            schema: 1,
            reserved: 0,
            nflex: c.nflex as u64,
            nflexnode: c.nflexnode as u64,
            nflexvert: c.nflexvert as u64,
            flex_interp: s.flex_interp.as_mut_ptr(),
            flex_cellnum: s.flex_cellnum.as_mut_ptr(),
            flex_nodeadr: s.flex_nodeadr.as_mut_ptr(),
            flex_nodenum: s.flex_nodenum.as_mut_ptr(),
            flex_vertadr: s.flex_vertadr.as_mut_ptr(),
            flex_vertnum: s.flex_vertnum.as_mut_ptr(),
            flex_centered: s.flex_centered.as_mut_ptr(),
            flex_nodebodyid: s.flex_nodebodyid.as_mut_ptr(),
            flex_vertbodyid: s.flex_vertbodyid.as_mut_ptr(),
            flex_node: s.flex_node.as_mut_ptr(),
            flex_vert: s.flex_vert.as_mut_ptr(),
            flex_vert0: s.flex_vert0.as_mut_ptr(),
        };
        // SAFETY: caller retains the immutable model/DLL through both calls.
        // Owned arrays are disjoint, aligned and capacity-checked. The bridge
        // checks every source/count/target before copying; empty arrays stay untouched.
        status("copy_flex_position", unsafe {
            mjwarp_native_copy_flex_position(owner, &t)
        })?;
        Ok(s)
    }

    /// 严格转换并校验位置字段。
    /// 二次插值沿用上游拒绝语义。
    pub fn into_fields(self) -> Result<FlexPositionFields, NativeProbeError> {
        check_info(self.info)?;
        check_counts(self.nflex, self.nflexnode, self.nflexvert)?;
        for (field, len, count, width) in [
            ("flex_interp", self.flex_interp.len(), self.nflex, 1),
            ("flex_cellnum", self.flex_cellnum.len(), self.nflex, 3),
            ("flex_nodeadr", self.flex_nodeadr.len(), self.nflex, 1),
            ("flex_nodenum", self.flex_nodenum.len(), self.nflex, 1),
            ("flex_vertadr", self.flex_vertadr.len(), self.nflex, 1),
            ("flex_vertnum", self.flex_vertnum.len(), self.nflex, 1),
            ("flex_centered", self.flex_centered.len(), self.nflex, 1),
            (
                "flex_nodebodyid",
                self.flex_nodebodyid.len(),
                self.nflexnode,
                1,
            ),
            (
                "flex_vertbodyid",
                self.flex_vertbodyid.len(),
                self.nflexvert,
                1,
            ),
            ("flex_node", self.flex_node.len(), self.nflexnode, 3),
            ("flex_vert", self.flex_vert.len(), self.nflexvert, 3),
            ("flex_vert0", self.flex_vert0.len(), self.nflexvert, 3),
        ] {
            super::check_length(field, elements(count, width)?, len)?;
        }
        if let Some(index) = self.flex_centered.iter().position(|&v| v > 1) {
            return Err(InputError::InvalidTopology {
                field: "flex_centered",
                index,
                reason: "invalid_boolean",
            }
            .into());
        }
        let f = FlexPositionFields {
            flex_interp: self.flex_interp,
            flex_cellnum: self.flex_cellnum,
            flex_nodeadr: self.flex_nodeadr,
            flex_nodenum: self.flex_nodenum,
            flex_vertadr: self.flex_vertadr,
            flex_vertnum: self.flex_vertnum,
            flex_centered: self.flex_centered.into_iter().map(|v| v == 1).collect(),
            flex_nodebodyid: self.flex_nodebodyid,
            flex_vertbodyid: self.flex_vertbodyid,
            flex_node: convert("flex_node", &self.flex_node)?,
            flex_vert: convert("flex_vert", &self.flex_vert)?,
            flex_vert0: convert("flex_vert0", &self.flex_vert0)?,
        };
        f.validate(self.info.nbody as usize)?;
        // 已验证的范围保护切片。
        // 原生非零值不能靠下溢隐藏。
        for (index, &centered) in f.flex_centered.iter().enumerate() {
            if !centered {
                continue;
            }
            let node = f.flex_nodeadr[index].max(0) as usize;
            let count = f.flex_nodenum[index] as usize;
            let vert = f.flex_vertadr[index] as usize;
            let verts = f.flex_vertnum[index] as usize;
            if self.flex_node[3 * node..3 * (node + count)]
                .iter()
                .any(|&v| v != 0.0)
                || (f.flex_interp[index] == 0
                    && self.flex_vert[3 * vert..3 * (vert + verts)]
                        .iter()
                        .any(|&v| v != 0.0))
            {
                return Err(InputError::InvalidTopology {
                    field: "flex_centered",
                    index,
                    reason: "centered_native_position_has_local_offset",
                }
                .into());
            }
        }
        Ok(f)
    }

    /// 组合已准备的肌腱输入。
    /// 维度一致不证明模型同源。
    /// 调用方须保证身体编号一致。
    pub fn into_model(
        self,
        tendons: TendonModelInput,
    ) -> Result<FlexPositionModelInput, NativeProbeError> {
        check_info(self.info)?;
        let rigid = tendons
            .spatial()
            .fixed()
            .camlight()
            .mocap()
            .attached()
            .rigid();
        let k = rigid.kinematics();
        for (field, expected, actual) in [
            ("nbody", self.info.nbody as usize, k.nbody()),
            ("nq", self.info.nq as usize, k.nq()),
            ("nv", self.info.nv as usize, k.nv()),
        ] {
            super::check_length(field, expected, actual)?;
        }
        Ok(FlexPositionModelInput::new(tendons, self.into_fields()?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> NativeFlexPositionSnapshot {
        NativeFlexPositionSnapshot {
            info: NativeModelInfo {
                schema: 1,
                native_version: NativeModelInfo::VERSION,
                pointer_bytes: 8,
                num_bytes: 8,
                index_bytes: 4,
                size_bytes: 8,
                native_model_bytes: 1,
                nbody: 1,
                ..NativeModelInfo::default()
            },
            nflex: 2,
            nflexnode: 8,
            nflexvert: 2,
            flex_interp: vec![0, 1],
            flex_cellnum: vec![1; 6],
            flex_nodeadr: vec![-1, 0],
            flex_nodenum: vec![0, 8],
            flex_vertadr: vec![0, 1],
            flex_vertnum: vec![1, 1],
            flex_centered: vec![1, 1],
            flex_nodebodyid: vec![0; 8],
            flex_vertbodyid: vec![0, -1],
            flex_node: vec![0.0; 24],
            flex_vert: vec![0.0; 6],
            flex_vert0: vec![0.5; 6],
        }
    }
    #[test]
    fn checks_flex_counts_before_allocation() {
        for bad in [-1, i64::from(i32::MAX) + 1, i64::MAX] {
            for c in [(bad, 0, 0), (0, bad, 0), (0, 0, bad)] {
                assert!(check_counts(c.0, c.1, c.2).is_err());
            }
        }
        check_counts(0, 0, 0).unwrap();
        check_counts(4, 20, 80).unwrap();
        for c in [
            (i64::from(i32::MAX), 0, 0),
            (0, i64::from(i32::MAX), 0),
            (0, 0, i64::from(i32::MAX)),
        ] {
            assert!(check_counts(c.0, c.1, c.2).is_err());
        }
    }
    #[test]
    fn converts_flex_fields_and_preserves_finite_bits() {
        let mut s = snapshot();
        s.flex_node[0] = -0.0;
        s.flex_vert[0] = -0.0;
        let f = s.into_fields().unwrap();
        assert_eq!(f.flex_node[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(f.flex_vert[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(f.flex_centered, [true, true]);
    }
    #[test]
    fn rejects_every_mutated_flex_length_before_conversion() {
        for case in 0..12 {
            let mut s = snapshot();
            match case {
                0 => {
                    s.flex_interp.pop();
                }
                1 => {
                    s.flex_cellnum.pop();
                }
                2 => {
                    s.flex_nodeadr.pop();
                }
                3 => {
                    s.flex_nodenum.pop();
                }
                4 => {
                    s.flex_vertadr.pop();
                }
                5 => {
                    s.flex_vertnum.pop();
                }
                6 => {
                    s.flex_centered.pop();
                }
                7 => {
                    s.flex_nodebodyid.pop();
                }
                8 => {
                    s.flex_vertbodyid.pop();
                }
                9 => {
                    s.flex_node.pop();
                }
                10 => {
                    s.flex_vert.pop();
                }
                _ => {
                    s.flex_vert0.pop();
                }
            }
            assert!(
                matches!(
                    s.into_fields(),
                    Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
                ),
                "case {case}"
            );
        }
    }
    #[test]
    fn labels_native_flex_float_failures_and_catches_centered_underflow() {
        for field in 0..3 {
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
                let mut s = snapshot();
                match field {
                    0 => s.flex_node[0] = bad,
                    1 => s.flex_vert[0] = bad,
                    _ => s.flex_vert0[0] = bad,
                }
                let error = s.into_fields().unwrap_err();
                let expected_field = ["flex_node", "flex_vert", "flex_vert0"][field];
                assert_eq!(
                    error,
                    NativeProbeError::Input(if bad.is_finite() {
                        InputError::ScalarOverflow {
                            field: expected_field,
                            index: 0,
                        }
                    } else {
                        InputError::NonFinite {
                            field: expected_field,
                            index: 0,
                        }
                    })
                );
            }
        }
        for field in 0..2 {
            let mut s = snapshot();
            if field == 0 {
                s.flex_node[0] = f64::MIN_POSITIVE;
            } else {
                s.flex_vert[0] = f64::MIN_POSITIVE;
            }
            assert!(matches!(
                s.into_fields(),
                Err(NativeProbeError::Input(InputError::InvalidTopology {
                    field: "flex_centered",
                    ..
                }))
            ));
        }
    }
    #[test]
    fn rejects_bad_counts_boolean_indices_and_quadratic_modes() {
        for case in 0..9 {
            let mut s = snapshot();
            match case {
                0 => s.nflex = -1,
                1 => s.nflexnode = i64::MAX,
                2 => s.nflexvert = 3,
                3 => s.flex_centered[0] = 2,
                4 => s.flex_nodebodyid[0] = 1,
                5 => s.flex_vertbodyid[0] = -1,
                6 => s.flex_interp[1] = 2,
                7 => s.flex_interp[1] = -2,
                _ => s.info.nbody = 0,
            }
            assert!(s.into_fields().is_err(), "case {case}");
        }
    }
}
