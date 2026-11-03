//! R06-S2c：原生 flex shell/elem 拓扑（additive ABI `mjwarp_flex_topology_*`）。
//!
//! 只读：不分配设备缓冲、不建 BVH、不使用 GPU、不改 `mjwarp_flex_position_*` /
//! `mjwarp_flex_render_*` 结构。只新导 7 个字段 + 4 个计数：
//! `flex_elemadr` / `flex_elemnum` / `flex_elemdataadr` / `flex_elem`（扁平
//! `nflexelemdata` 个）/ `flex_shellnum` / `flex_shelldataadr` / `flex_shell`（扁平
//! `nflexshelldata` 个）。`flex_vertadr`/`flex_vertnum` 仍在 `flex_position` 桶，
//! 不重复导（单一真相源）。
//!
//! 宿主边界：本桶全是整数，**无 f64→f32 转换点**（转换只在 `flex_render` 桶的
//! `flex_radius` 一处，见 `docs/development-evidence/render-bvh/r06-flex-datasource/`）。
//! 上游字段语义见 `mujoco/mjmodel.h`（`flex_elem` 每元素 `dim+1` 个顶点 id、
//! `flex_shell` 每片段 `dim` 个）。
//!
//! `NativeModelProbe::flex_topology_snapshot` 访问器也落在本文件：R06-S2c 卡片
//! `schedule.files` 不含 `src/io/mod.rs`，同 crate inherent impl 保持 scope 不越界。

use super::super::check_length;
use super::super::fields::native_staging as staging;
use super::super::kinematic::native::{check_info, elements};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{
        NativeModelInfo,
        flex::{FlexRenderFields, FlexTopologyFields},
    },
};
use std::ffi::c_void;

/// 独占 flex shell/elem 拓扑快照（七个模型数组 + 四个计数）。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeFlexTopologySnapshot {
    pub info: NativeModelInfo,
    pub nflex: i64,
    pub nflexelem: i64,
    pub nflexelemdata: i64,
    pub nflexshelldata: i64,
    pub flex_elemadr: Vec<i32>,
    pub flex_elemnum: Vec<i32>,
    pub flex_elemdataadr: Vec<i32>,
    pub flex_elem: Vec<i32>,
    pub flex_shellnum: Vec<i32>,
    pub flex_shelldataadr: Vec<i32>,
    pub flex_shell: Vec<i32>,
}

#[repr(C)]
#[derive(Default)]
struct Counts {
    schema: u32,
    reserved: u32,
    nflex: i64,
    nflexelem: i64,
    nflexelemdata: i64,
    nflexshelldata: i64,
}
#[repr(C)]
struct Targets {
    schema: u32,
    reserved: u32,
    nflex: u64,
    nflexelem: u64,
    nflexelemdata: u64,
    nflexshelldata: u64,
    flex_elemadr: *mut i32,
    flex_elemnum: *mut i32,
    flex_elemdataadr: *mut i32,
    flex_elem: *mut i32,
    flex_shellnum: *mut i32,
    flex_shelldataadr: *mut i32,
    flex_shell: *mut i32,
}
const _: () = {
    assert!(size_of::<Counts>() == 40 && align_of::<Counts>() == 8);
    assert!(std::mem::offset_of!(Counts, nflex) == 8);
    assert!(std::mem::offset_of!(Counts, nflexshelldata) == 32);
    assert!(size_of::<Targets>() == 96 && align_of::<Targets>() == 8);
    assert!(std::mem::offset_of!(Targets, nflexshelldata) == 32);
    assert!(std::mem::offset_of!(Targets, flex_elemadr) == 40);
    assert!(std::mem::offset_of!(Targets, flex_elemnum) == 48);
    assert!(std::mem::offset_of!(Targets, flex_elemdataadr) == 56);
    assert!(std::mem::offset_of!(Targets, flex_elem) == 64);
    assert!(std::mem::offset_of!(Targets, flex_shellnum) == 72);
    assert!(std::mem::offset_of!(Targets, flex_shelldataadr) == 80);
    assert!(std::mem::offset_of!(Targets, flex_shell) == 88);
};
unsafe extern "C" {
    fn mjwarp_native_flex_topology_info(owner: *const c_void, counts: *mut Counts) -> i32;
    fn mjwarp_native_copy_flex_topology(owner: *const c_void, targets: *const Targets) -> i32;
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
fn check_counts(nflex: i64, elem: i64, elemdata: i64, shelldata: i64) -> Result<(), InputError> {
    for (field, count) in [
        ("nflex", nflex),
        ("nflexelem", elem),
        ("nflexelemdata", elemdata),
        ("nflexshelldata", shelldata),
    ] {
        if count < 0 || count > i64::from(i32::MAX) {
            return Err(InputError::Overflow { field });
        }
        elements(count, 1)?;
    }
    Ok(())
}

impl NativeFlexTopologySnapshot {
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
        status("flex_topology_info", unsafe {
            mjwarp_native_flex_topology_info(owner, &mut c)
        })?;
        if c.schema != 1 || c.reserved != 0 {
            return Err(NativeProbeError::AbiMismatch);
        }
        check_counts(c.nflex, c.nflexelem, c.nflexelemdata, c.nflexshelldata)?;
        let mut s = Self {
            info,
            nflex: c.nflex,
            nflexelem: c.nflexelem,
            nflexelemdata: c.nflexelemdata,
            nflexshelldata: c.nflexshelldata,
            flex_elemadr: staging(elements(c.nflex, 1)?)?,
            flex_elemnum: staging(elements(c.nflex, 1)?)?,
            flex_elemdataadr: staging(elements(c.nflex, 1)?)?,
            flex_elem: staging(elements(c.nflexelemdata, 1)?)?,
            flex_shellnum: staging(elements(c.nflex, 1)?)?,
            flex_shelldataadr: staging(elements(c.nflex, 1)?)?,
            flex_shell: staging(elements(c.nflexshelldata, 1)?)?,
        };
        let t = Targets {
            schema: 1,
            reserved: 0,
            nflex: c.nflex as u64,
            nflexelem: c.nflexelem as u64,
            nflexelemdata: c.nflexelemdata as u64,
            nflexshelldata: c.nflexshelldata as u64,
            flex_elemadr: s.flex_elemadr.as_mut_ptr(),
            flex_elemnum: s.flex_elemnum.as_mut_ptr(),
            flex_elemdataadr: s.flex_elemdataadr.as_mut_ptr(),
            flex_elem: s.flex_elem.as_mut_ptr(),
            flex_shellnum: s.flex_shellnum.as_mut_ptr(),
            flex_shelldataadr: s.flex_shelldataadr.as_mut_ptr(),
            flex_shell: s.flex_shell.as_mut_ptr(),
        };
        // SAFETY: caller retains the immutable model/DLL through both calls.
        // Owned arrays are disjoint, aligned and capacity-checked. The bridge
        // checks every source/count/target before copying; empty arrays stay untouched.
        status("copy_flex_topology", unsafe {
            mjwarp_native_copy_flex_topology(owner, &t)
        })?;
        Ok(s)
    }

    /// 严格转换并校验拓扑字段（地址链/长度不一致在 [`FlexTopologyFields::validate`] 拒绝）。
    pub fn into_fields(self) -> Result<FlexTopologyFields, NativeProbeError> {
        check_info(self.info)?;
        check_counts(
            self.nflex,
            self.nflexelem,
            self.nflexelemdata,
            self.nflexshelldata,
        )?;
        for (field, len, count) in [
            ("flex_elemadr", self.flex_elemadr.len(), self.nflex),
            ("flex_elemnum", self.flex_elemnum.len(), self.nflex),
            ("flex_elemdataadr", self.flex_elemdataadr.len(), self.nflex),
            ("flex_elem", self.flex_elem.len(), self.nflexelemdata),
            ("flex_shellnum", self.flex_shellnum.len(), self.nflex),
            (
                "flex_shelldataadr",
                self.flex_shelldataadr.len(),
                self.nflex,
            ),
            ("flex_shell", self.flex_shell.len(), self.nflexshelldata),
        ] {
            check_length(field, elements(count, 1)?, len)?;
        }
        let fields = FlexTopologyFields {
            nflex: elements(self.nflex, 1)?,
            nflexelem: elements(self.nflexelem, 1)?,
            nflexelemdata: elements(self.nflexelemdata, 1)?,
            nflexshelldata: elements(self.nflexshelldata, 1)?,
            flex_elemadr: self.flex_elemadr,
            flex_elemnum: self.flex_elemnum,
            flex_elemdataadr: self.flex_elemdataadr,
            flex_elem: self.flex_elem,
            flex_shellnum: self.flex_shellnum,
            flex_shelldataadr: self.flex_shelldataadr,
            flex_shell: self.flex_shell,
        };
        fields.validate()?;
        Ok(fields)
    }
}

/// 与 [`FlexRenderFields`] 联合校验：元素/壳顶点宽度 = `dim+1` / `dim`（上游 `mjmodel.h`）。
///
/// 拓扑桶没有 `flex_dim`（在 render 桶），逐维宽度只能联合判定；元素空间地址链
/// （首 0、逐 flex 连续、总和 = `nflexelem`）也在此完整校验。
pub fn validate_topology_against_render(
    topology: &FlexTopologyFields,
    render: &FlexRenderFields,
) -> Result<(), InputError> {
    // R06-S2c 修正（QA `3b8e0d7f`）：先校验 render 侧长度，短向量不得进入按 flex 索引的判定。
    render.validate()?;
    topology.validate()?;
    if topology.nflex != render.nflex {
        return Err(InputError::LengthMismatch {
            field: "flex_topology_nflex",
            expected: render.nflex,
            actual: topology.nflex,
        });
    }
    let invalid = |field, index, reason| InputError::InvalidTopology {
        field,
        index,
        reason,
    };
    let mut next = 0usize;
    for flex in 0..topology.nflex {
        let start = usize::try_from(topology.flex_elemadr[flex])
            .map_err(|_| invalid("flex_elemadr", flex, "negative_address"))?;
        if start != next {
            return Err(invalid("flex_elemadr", flex, "address_gap_or_overlap"));
        }
        let items = usize::try_from(topology.flex_elemnum[flex])
            .map_err(|_| invalid("flex_elemnum", flex, "negative_count"))?;
        next = start.checked_add(items).ok_or(InputError::Overflow {
            field: "flex_topology_fields",
        })?;
    }
    if next != topology.nflexelem {
        return Err(invalid("flex_elemadr", 0, "total_mismatch"));
    }
    for flex in 0..topology.nflex {
        let dim = usize::try_from(render.flex_dim[flex])
            .map_err(|_| invalid("flex_dim", flex, "negative_dimension"))?;
        let elems = usize::try_from(topology.flex_elemnum[flex])
            .map_err(|_| invalid("flex_elemnum", flex, "negative_count"))?;
        let elem_start = usize::try_from(topology.flex_elemdataadr[flex])
            .map_err(|_| invalid("flex_elemdataadr", flex, "negative_address"))?;
        if flex == 0 && elem_start != 0 {
            return Err(invalid("flex_elemdataadr", 0, "first_address_not_zero"));
        }
        let next_elem_data = topology
            .flex_elemdataadr
            .get(flex + 1)
            .map(|value| usize::try_from(*value))
            .transpose()
            .map_err(|_| invalid("flex_elemdataadr", flex + 1, "negative_address"))?
            .unwrap_or(topology.nflexelemdata);
        let elem_data = next_elem_data
            .checked_sub(elem_start)
            .ok_or_else(|| invalid("flex_elemdataadr", flex, "address_not_monotonic"))?;
        if elem_data != elems * (dim + 1) {
            return Err(invalid("flex_elemdataadr", flex, "element_width_mismatch"));
        }
        let shells = usize::try_from(topology.flex_shellnum[flex])
            .map_err(|_| invalid("flex_shellnum", flex, "negative_count"))?;
        let shell_start = usize::try_from(topology.flex_shelldataadr[flex])
            .map_err(|_| invalid("flex_shelldataadr", flex, "negative_address"))?;
        if flex == 0 && shell_start != 0 {
            return Err(invalid("flex_shelldataadr", 0, "first_address_not_zero"));
        }
        let next_shell_data = topology
            .flex_shelldataadr
            .get(flex + 1)
            .map(|value| usize::try_from(*value))
            .transpose()
            .map_err(|_| invalid("flex_shelldataadr", flex + 1, "negative_address"))?
            .unwrap_or(topology.nflexshelldata);
        let shell_data = next_shell_data
            .checked_sub(shell_start)
            .ok_or_else(|| invalid("flex_shelldataadr", flex, "address_not_monotonic"))?;
        if shell_data != shells * dim {
            return Err(invalid("flex_shelldataadr", flex, "shell_width_mismatch"));
        }
    }
    Ok(())
}

impl super::super::NativeModelProbe {
    /// 复制 flex shell/elem 拓扑快照（additive `mjwarp_flex_topology_*` 桶）。
    pub fn flex_topology_snapshot(&self) -> Result<NativeFlexTopologySnapshot, NativeProbeError> {
        // SAFETY: Self retains the immutable owner and its DLL on this thread;
        // the capture owns every output buffer and the bridge preflights counts.
        unsafe { NativeFlexTopologySnapshot::capture(self.owner_ptr(), self.info()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> NativeFlexTopologySnapshot {
        NativeFlexTopologySnapshot {
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
            nflexelem: 2,
            nflexelemdata: 7,
            nflexshelldata: 3,
            flex_elemadr: vec![0, 1],
            flex_elemnum: vec![1, 1],
            flex_elemdataadr: vec![0, 3],
            flex_elem: vec![0, 1, 2, 0, 1, 3, 2],
            flex_shellnum: vec![0, 1],
            flex_shelldataadr: vec![0, 0],
            flex_shell: vec![0, 1, 2],
        }
    }

    #[test]
    fn converts_flex_topology_fields() {
        let fields = snapshot().into_fields().unwrap();
        assert_eq!(fields.nflex, 2);
        assert_eq!(fields.nflexelem, 2);
        assert_eq!(fields.nflexelemdata, 7);
        assert_eq!(fields.nflexshelldata, 3);
        assert_eq!(fields.flex_elemadr, [0, 1]);
        assert_eq!(fields.flex_elem, [0, 1, 2, 0, 1, 3, 2]);
        assert_eq!(fields.flex_shell, [0, 1, 2]);
    }

    #[test]
    fn rejects_every_mutated_flex_topology_length_before_conversion() {
        for case in 0..7 {
            let mut s = snapshot();
            match case {
                0 => s.flex_elemadr.pop(),
                1 => s.flex_elemnum.pop(),
                2 => s.flex_elemdataadr.pop(),
                3 => s.flex_elem.pop(),
                4 => s.flex_shellnum.pop(),
                5 => s.flex_shelldataadr.pop(),
                _ => s.flex_shell.pop(),
            };
            assert!(matches!(
                s.into_fields(),
                Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
            ));
        }
    }

    #[test]
    fn rejects_bad_counts_and_negative_addresses() {
        for case in 0..4 {
            let mut s = snapshot();
            match case {
                0 => s.nflex = -1,
                1 => s.nflexelemdata = -1,
                2 => s.flex_elemadr = vec![0, -1],
                _ => s.flex_shelldataadr = vec![-1, 0],
            }
            assert!(s.into_fields().is_err(), "case {case}");
        }
    }

    fn render_for(dims: &[i32]) -> FlexRenderFields {
        FlexRenderFields {
            nflex: dims.len(),
            nflexvert: 10,
            nflexedge: 0,
            flex_dim: dims.to_vec(),
            flex_radius: vec![0.1; dims.len()],
            flex_rgba: vec![1.0; 4 * dims.len()],
            flex_edgeadr: vec![0; dims.len()],
            flex_edgenum: vec![0; dims.len()],
        }
    }

    #[test]
    fn cross_validates_element_and_shell_widths_against_render() {
        let fields = snapshot().into_fields().unwrap();
        // 2D：元素宽度 3；3D：元素宽度 4、壳宽度 3。
        validate_topology_against_render(&fields, &render_for(&[2, 3])).expect("宽度一致");
        assert!(validate_topology_against_render(&fields, &render_for(&[2, 2])).is_err());
        assert!(validate_topology_against_render(&fields, &render_for(&[3, 3])).is_err());
        let mut broken = fields.clone();
        broken.flex_elemadr = vec![0, 2];
        assert!(validate_topology_against_render(&broken, &render_for(&[2, 3])).is_err());
    }

    #[test]
    fn rejects_short_render_vectors_before_indexing() {
        let fields = snapshot().into_fields().unwrap();
        // 短 render 向量在入口被拦（不进按 flex 索引判定），错误含具体字段/期望/实际。
        let mut render = render_for(&[2, 3]);
        render.flex_dim.pop();
        assert!(matches!(
            validate_topology_against_render(&fields, &render),
            Err(InputError::LengthMismatch {
                field: "flex_dim",
                expected: 2,
                actual: 1,
            })
        ));
        let mut render = render_for(&[2, 3]);
        render.flex_rgba.pop();
        assert!(matches!(
            validate_topology_against_render(&fields, &render),
            Err(InputError::LengthMismatch {
                field: "flex_rgba",
                expected: 8,
                actual: 7,
            })
        ));
    }

    #[test]
    fn accepts_empty_topology() {
        let mut s = snapshot();
        s.nflex = 0;
        s.nflexelem = 0;
        s.nflexelemdata = 0;
        s.nflexshelldata = 0;
        s.flex_elemadr.clear();
        s.flex_elemnum.clear();
        s.flex_elemdataadr.clear();
        s.flex_elem.clear();
        s.flex_shellnum.clear();
        s.flex_shelldataadr.clear();
        s.flex_shell.clear();
        let fields = s.into_fields().expect("空拓扑合法");
        assert_eq!(fields.nflex, 0);
        assert!(fields.validate().is_ok());
    }

    /// 原生冒烟（默认忽略；S2c 证据命令显式跑）：读 `fixtures/flex-position/flex-tree.mjb`
    /// 的 flex topology 桶，核对 2D/3D 元素与 shell 拓扑（flex-tree = 2×dim1 + 2×dim3）。
    ///
    /// 2D 元素每项 3 个顶点 id、3D 每项 4 个（上游 `mjmodel.h`：`dim+1 per elem`）；
    /// 地址链连续性由 [`FlexTopologyFields::validate`] 保证，这里再核对单 flex 宽度。
    #[cfg(feature = "native-model-probe")]
    #[test]
    #[ignore = "需要受信 MuJoCo DLL 与 native-model-probe 构建（证据命令显式跑）"]
    fn captures_flex_topology_from_flex_fixture() {
        const MJB: &[u8] = include_bytes!("../../../fixtures/flex-position/flex-tree.mjb");
        let dll = std::env::var_os("MJWARP_MUJOCO_DLL").expect("set MJWARP_MUJOCO_DLL");
        // SAFETY: 证据命令只传受信 MuJoCo 3.12.0 DLL 与仓库内已编译 MJB 夹具。
        let probe =
            unsafe { crate::io::NativeModelProbe::load_trusted(std::path::Path::new(&dll), MJB) }
                .expect("verified DLL + fixture must open");
        let render = probe
            .flex_render_snapshot()
            .expect("flex render snapshot")
            .into_fields()
            .expect("flex render fields");
        let fields = probe
            .flex_topology_snapshot()
            .expect("flex topology snapshot")
            .into_fields()
            .expect("flex topology fields");
        println!(
            "flex-tree topology: nflex={} nflexelem={} nflexelemdata={} nflexshelldata={} elemadr={:?} elemnum={:?} elemdataadr={:?} shellnum={:?} shelldataadr={:?}",
            fields.nflex,
            fields.nflexelem,
            fields.nflexelemdata,
            fields.nflexshelldata,
            fields.flex_elemadr,
            fields.flex_elemnum,
            fields.flex_elemdataadr,
            fields.flex_shellnum,
            fields.flex_shelldataadr
        );
        validate_topology_against_render(&fields, &render).expect("拓扑与 render 联合校验通过");
        assert_eq!(fields.nflex, 4, "flex-tree 夹具含 4 个柔体");
        assert_eq!(fields.nflex, render.nflex);
        assert!(fields.nflexelem > 0 && fields.nflexelemdata >= fields.nflexelem);
    }
}
