//! R06-S2b：原生 flex 渲染字段（additive ABI `mjwarp_flex_render_*`）。
//!
//! 只读：不分配设备缓冲、不建 BVH、不使用 GPU、不改 `mjwarp_flex_position_*` 结构。
//! 只新导 5 个字段 + 3 个计数（`flex_dim` / `flex_radius`(mjtNum) / `flex_rgba`(float) /
//! `flex_edgeadr` / `flex_edgenum`）；`flex_edge` 由 G01 字段集提供（不重复导，避免双真相源）。
//!
//! 宿主边界：`mjtNum`(f64) → f32 只走 [`convert`] 一处（`NonFinite`/`ScalarOverflow` 带字段名）；
//! `flex_rgba` 原生即 float，只做有限性校验。转换点见
//! `docs/development-evidence/render-bvh/r06-flex-datasource/`。

use super::super::check_length;
use super::super::fields::native_staging as staging;
use super::super::kinematic::native::{check_info, convert, elements};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{NativeModelInfo, flex::FlexRenderFields},
};
use std::ffi::c_void;

/// 独占 flex 渲染字段快照（五个模型数组 + 三个计数）。
///
/// `flex_radius` 保留原生 f64 精度；转换在 [`NativeFlexRenderSnapshot::into_fields`] 集中完成。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeFlexRenderSnapshot {
    pub info: NativeModelInfo,
    pub nflex: i64,
    pub nflexvert: i64,
    pub nflexedge: i64,
    pub flex_dim: Vec<i32>,
    pub flex_radius: Vec<f64>,
    pub flex_rgba: Vec<f32>,
    pub flex_edgeadr: Vec<i32>,
    pub flex_edgenum: Vec<i32>,
}

#[repr(C)]
#[derive(Default)]
struct Counts {
    schema: u32,
    reserved: u32,
    nflex: i64,
    nflexvert: i64,
    nflexedge: i64,
}
#[repr(C)]
struct Targets {
    schema: u32,
    reserved: u32,
    nflex: u64,
    nflexvert: u64,
    nflexedge: u64,
    flex_dim: *mut i32,
    flex_radius: *mut f64,
    flex_rgba: *mut f32,
    flex_edgeadr: *mut i32,
    flex_edgenum: *mut i32,
}
const _: () = {
    assert!(size_of::<Counts>() == 32 && align_of::<Counts>() == 8);
    assert!(std::mem::offset_of!(Counts, nflex) == 8);
    assert!(size_of::<Targets>() == 72 && align_of::<Targets>() == 8);
    assert!(std::mem::offset_of!(Targets, nflexedge) == 24);
    assert!(std::mem::offset_of!(Targets, flex_dim) == 32);
    assert!(std::mem::offset_of!(Targets, flex_radius) == 40);
    assert!(std::mem::offset_of!(Targets, flex_rgba) == 48);
    assert!(std::mem::offset_of!(Targets, flex_edgenum) == 64);
};
unsafe extern "C" {
    fn mjwarp_native_flex_render_info(owner: *const c_void, counts: *mut Counts) -> i32;
    fn mjwarp_native_copy_flex_render(owner: *const c_void, targets: *const Targets) -> i32;
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
/// 计数预检：非负、不超 i32，且按数组展开后仍在宿主容量内。
fn check_counts(nflex: i64, verts: i64, edges: i64) -> Result<(), InputError> {
    for (field, count) in [("nflex", nflex), ("nflexvert", verts), ("nflexedge", edges)] {
        if count < 0 || count > i64::from(i32::MAX) {
            return Err(InputError::Overflow { field });
        }
        elements(count, 3)?;
    }
    // flex_rgba 展开 4 倍（float），其余计数展开 1/3 宽已由 elements 覆盖。
    nflex
        .checked_mul(4)
        .filter(|&n| n <= i64::from(i32::MAX))
        .ok_or(InputError::Overflow {
            field: "flex_render_fields",
        })?;
    Ok(())
}
impl NativeFlexRenderSnapshot {
    /// # Safety
    /// owner 须拥有对应只读模型；调用期间须保持其 DLL 存活。
    pub(in crate::io) unsafe fn capture(
        owner: *const c_void,
        info: NativeModelInfo,
    ) -> Result<Self, NativeProbeError> {
        check_info(info)?;
        let mut c = Counts::default();
        // SAFETY: caller retains the immutable model/DLL. The initialized count
        // DTO is aligned and writable; the bridge does not expose native pointers.
        status("flex_render_info", unsafe {
            mjwarp_native_flex_render_info(owner, &mut c)
        })?;
        if c.schema != 1 || c.reserved != 0 {
            return Err(NativeProbeError::AbiMismatch);
        }
        check_counts(c.nflex, c.nflexvert, c.nflexedge)?;
        let mut s = Self {
            info,
            nflex: c.nflex,
            nflexvert: c.nflexvert,
            nflexedge: c.nflexedge,
            flex_dim: staging(elements(c.nflex, 1)?)?,
            flex_radius: staging(elements(c.nflex, 1)?)?,
            flex_rgba: staging(elements(c.nflex, 4)?)?,
            flex_edgeadr: staging(elements(c.nflex, 1)?)?,
            flex_edgenum: staging(elements(c.nflex, 1)?)?,
        };
        let t = Targets {
            schema: 1,
            reserved: 0,
            nflex: c.nflex as u64,
            nflexvert: c.nflexvert as u64,
            nflexedge: c.nflexedge as u64,
            flex_dim: s.flex_dim.as_mut_ptr(),
            flex_radius: s.flex_radius.as_mut_ptr(),
            flex_rgba: s.flex_rgba.as_mut_ptr(),
            flex_edgeadr: s.flex_edgeadr.as_mut_ptr(),
            flex_edgenum: s.flex_edgenum.as_mut_ptr(),
        };
        // SAFETY: caller retains the immutable model/DLL through both calls.
        // Owned arrays are disjoint, aligned and capacity-checked. The bridge
        // checks every source/count/target before copying; empty arrays stay untouched.
        status("copy_flex_render", unsafe {
            mjwarp_native_copy_flex_render(owner, &t)
        })?;
        Ok(s)
    }

    /// 严格转换并校验 flex 渲染字段（窗口/映射不一致在 [`FlexRenderFields::validate`] 拒绝）。
    pub fn into_fields(self) -> Result<FlexRenderFields, NativeProbeError> {
        check_info(self.info)?;
        check_counts(self.nflex, self.nflexvert, self.nflexedge)?;
        for (field, len, count, width) in [
            ("flex_dim", self.flex_dim.len(), self.nflex, 1),
            ("flex_radius", self.flex_radius.len(), self.nflex, 1),
            ("flex_rgba", self.flex_rgba.len(), self.nflex, 4),
            ("flex_edgeadr", self.flex_edgeadr.len(), self.nflex, 1),
            ("flex_edgenum", self.flex_edgenum.len(), self.nflex, 1),
        ] {
            check_length(field, elements(count, width)?, len)?;
        }
        if let Some(index) = self.flex_rgba.iter().position(|value| !value.is_finite()) {
            return Err(InputError::NonFinite {
                field: "flex_rgba",
                index,
            }
            .into());
        }
        let fields = FlexRenderFields {
            nflex: elements(self.nflex, 1)?,
            nflexvert: elements(self.nflexvert, 1)?,
            nflexedge: elements(self.nflexedge, 1)?,
            flex_dim: self.flex_dim,
            flex_radius: convert("flex_radius", &self.flex_radius)?,
            flex_rgba: self.flex_rgba,
            flex_edgeadr: self.flex_edgeadr,
            flex_edgenum: self.flex_edgenum,
        };
        fields.validate()?;
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> NativeFlexRenderSnapshot {
        NativeFlexRenderSnapshot {
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
            nflexvert: 3,
            nflexedge: 2,
            flex_dim: vec![1, 2],
            flex_radius: vec![0.02, 0.03],
            flex_rgba: vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            flex_edgeadr: vec![0, 1],
            flex_edgenum: vec![1, 1],
        }
    }

    #[test]
    fn converts_flex_render_fields_with_single_cast_point() {
        let fields = snapshot().into_fields().unwrap();
        assert_eq!(fields.nflex, 2);
        assert_eq!(fields.nflexvert, 3);
        assert_eq!(fields.nflexedge, 2);
        assert_eq!(fields.flex_dim, [1, 2]);
        assert_eq!(fields.flex_radius[0].to_bits(), (0.02f64 as f32).to_bits());
        assert_eq!(fields.flex_edgeadr, [0, 1]);
        assert_eq!(fields.flex_edgenum, [1, 1]);
        // 1D flex 每边一个图元、2D flex 每 flex 一个图元（上游 render_util 语义）。
        let mapping = fields.geom_mapping().unwrap();
        assert_eq!(mapping.flexid, [0, 1]);
        assert_eq!(mapping.edgeid, [0, -1]);
    }

    #[test]
    fn rejects_every_mutated_flex_render_length_before_conversion() {
        for case in 0..5 {
            let mut s = snapshot();
            match case {
                0 => {
                    s.flex_dim.pop();
                }
                1 => {
                    s.flex_radius.pop();
                }
                2 => {
                    s.flex_rgba.pop();
                }
                3 => {
                    s.flex_edgeadr.pop();
                }
                _ => {
                    s.flex_edgenum.pop();
                }
            }
            assert!(matches!(
                s.into_fields(),
                Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
            ));
        }
    }

    #[test]
    fn labels_non_finite_rgba_and_rejects_open_bits() {
        let mut s = snapshot();
        s.flex_rgba[5] = f32::NAN;
        assert!(matches!(
            s.into_fields(),
            Err(NativeProbeError::Input(InputError::NonFinite {
                field: "flex_rgba",
                index: 5
            }))
        ));
        let mut s = snapshot();
        s.flex_radius[1] = f64::INFINITY;
        assert!(matches!(
            s.into_fields(),
            Err(NativeProbeError::Input(InputError::NonFinite {
                field: "flex_radius",
                index: 1
            }))
        ));
    }

    #[test]
    fn rejects_bad_counts_and_inconsistent_edge_addresses() {
        for case in 0..4 {
            let mut s = snapshot();
            match case {
                0 => s.nflex = -1,
                1 => s.nflexvert = -1,
                2 => s.flex_edgeadr = vec![0, 5],
                _ => s.flex_edgeadr = vec![0, 0],
            }
            assert!(s.into_fields().is_err(), "case {case}");
        }
    }

    /// 原生冒烟（默认忽略；S2b 证据命令显式跑）：读 `fixtures/flex-position/flex-tree.mjb`
    /// 的 flex render 桶，核对计数/映射与上游语义（1D 每边一个图元、2D/3D 每 flex 一个）。
    ///
    /// 前置：`MJWARP_MUJOCO_DLL` 指向受信 MuJoCo 3.12.0 DLL，构建 `--features native-model-probe`。
    #[cfg(feature = "native-model-probe")]
    #[test]
    #[ignore = "需要受信 MuJoCo DLL 与 native-model-probe 构建（证据命令显式跑）"]
    fn captures_flex_render_fields_from_flex_fixture() {
        const MJB: &[u8] = include_bytes!("../../../fixtures/flex-position/flex-tree.mjb");
        let dll = std::env::var_os("MJWARP_MUJOCO_DLL").expect("set MJWARP_MUJOCO_DLL");
        // SAFETY: 证据命令只传受信 MuJoCo 3.12.0 DLL 与仓库内已编译 MJB 夹具。
        let probe =
            unsafe { crate::io::NativeModelProbe::load_trusted(std::path::Path::new(&dll), MJB) }
                .expect("verified DLL + fixture must open");
        let fields = probe
            .flex_render_snapshot()
            .expect("flex render snapshot")
            .into_fields()
            .expect("flex render fields");
        assert_eq!(fields.nflex, 4, "flex-tree 夹具含 4 个柔体");
        assert!(fields.nflexvert > 0 && fields.nflexedge > 0);
        let mapping = fields.geom_mapping().expect("geom mapping");
        let expected: usize = (0..fields.nflex)
            .map(|flex| {
                if fields.flex_dim[flex] == 1 {
                    usize::try_from(fields.flex_edgenum[flex]).expect("edge count")
                } else {
                    1
                }
            })
            .sum();
        assert_eq!(
            mapping.len(),
            expected,
            "映射长度必须等于上游 bvh_nflexgeom 口径"
        );
        for (index, &flexid) in mapping.flexid.iter().enumerate() {
            let flex = usize::try_from(flexid).expect("flex id");
            assert!(flex < fields.nflex);
            if fields.flex_dim[flex] == 1 {
                assert!(mapping.edgeid[index] >= 0, "1D flex 图元必须带边号");
            } else {
                assert_eq!(mapping.edgeid[index], -1, "2D/3D flex 图元边号为 -1");
            }
        }
    }
}
