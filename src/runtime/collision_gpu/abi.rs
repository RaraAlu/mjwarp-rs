//! G10@P5：产品内核参数 ABI（与 `product.cu` 的 `mjwarp_collision_args` 逐字段对应）。
//!
//! 布局按 64 位 C ABI 推导（指针 8 字节、`int`/`float` 4 字节、嵌套结构按 8 字节对齐）：
//!
//! ```text
//! 偏移  字段                 偏移  字段
//! 0     geom_xpos            344   out（d_contact_out，136 字节）
//! 8     geom_xmat            480   nacon
//! ...                       488   nccd
//! 176   oct_coeff            496   overflow
//! 184   mesh_poly_args（sidecar 指针）
//! 192   nworld
//! ...                       总大小 504，对齐 8
//! ```
//!
//! M0a（mesh multicontact 方案 A sidecar）：24 个指针（原 23 + `mesh_poly_args`）占 0–191；
//! 标量块 192–343（全部较 496 版本 +8）；`out` 344–479；尾 3 指针 480/488/496；总 504。
//! 唯一对齐关系：标量止于 344（8 对齐），故无隐式填充；`mesh_poly_args` 在 i32 块之前。
//! sidecar 结构 `MjwarpMeshPolyArgs` 为 9 个指针（72 字节，align 8，顺序与 pinned
//! `collision_convex.py` L197-205 一致）；9 个 mesh polygon 数组全空时该指针为 null
//! （无 polygon 数据、mesh 多接触功能不启用、行为与改动前一致）。9 个数组的形状语义与
//! pinned `mujoco_warp/_src/types.py` L1329-1337/L1864-1872 一致（逐 mesh / 逐 polygon /
//! 逐 vertex 三段），由 [`validate_mesh_poly`] 的 CPU 校验写死（M0a 修正片更正）。
//!
//! 完整偏移表由 `tests.rs`/`abi_tests.rs` 的 `abi_*` 测试逐项锁死。M0a 起 `CollisionArgs`
//! 是冻结探针 `ProbeArgs`（496 字节、23 指针）的**超集**：前 23 个指针与 i32/f32 标量块
//! 顺序/类型一致，仅在指针块末尾插入 `mesh_poly_args`（总 504 字节）；探针 `ProbeArgs`/
//! 测试侧 `DeviceArrays` 保持 23 项输入语义（9 个 mesh_poly 数组以空 slice 传入），
//! 冻结探针与旧 G5 证据零改动。
//!
//! 本模块同时持有**纯 CPU** 的输入校验 [`validate_inputs`]：宿主 slice 长度与 `meta`
//! 给出的 ABI 容量必须匹配，否则在分配/发射前返回 `InvalidArgument`。

use super::{CollisionGpuError, CollisionGpuInputs};

/// 上游 `contact_*` 输出数组视图（对应 `contact.cu::d_contact_out`）。
///
/// `pos` 在设备侧是 `v3*`（12 字节步长），这里用 `*mut f32` 表示同一指针；`frame` 是
/// `[naconmax][9]` 行优先。
#[repr(C)]
pub(crate) struct CollisionContactOut {
    pub(crate) dist: *mut f32,
    pub(crate) pos: *mut f32,
    pub(crate) frame: *mut f32,
    pub(crate) includemargin: *mut f32,
    pub(crate) friction: *mut f32,
    pub(crate) solref: *mut f32,
    pub(crate) solreffriction: *mut f32,
    pub(crate) solimp: *mut f32,
    pub(crate) dim: *mut i32,
    pub(crate) geom: *mut i32,
    pub(crate) efc_address: *mut i32,
    pub(crate) worldid: *mut i32,
    /// 上游 `contact_type`（`contact.cu::d_contact_out::type`）。
    pub(crate) contact_type: *mut i32,
    pub(crate) geomcollisionid: *mut i32,
    pub(crate) adhesion: *mut f32,
    /// 全局 `nacon` 计数器（与 `CollisionArgs::nacon` 同址）。
    pub(crate) nacon: *mut i32,
    pub(crate) efc_address_width: i32,
}

/// mesh polygon 输入的设备侧 sidecar（对应 `product.cu::MjwarpMeshPolyArgs`，方案 A）。
///
/// 9 个指针顺序与 pinned `collision_convex.py` L197-205 一致；结构 72 字节、align 8。
/// 9 个宿主数组全空时不构造 sidecar（`CollisionArgs::mesh_poly_args` 为 null）。
///
/// 数组形状语义与 pinned `mujoco_warp/_src/types.py` L1329-1337/L1864-1872 一致：
/// `mesh_polynum`/`mesh_polyadr` 逐 mesh（`nmesh`）；`mesh_polynormal`（`vec3`）/
/// `mesh_polyvertadr`/`mesh_polyvertnum` 逐 polygon（`nmeshpoly`）；`mesh_polyvert` 为全部
/// polygon 顶点索引（`nmeshpolyvert`）；`mesh_polymapadr`/`mesh_polymapnum` 逐 vertex
/// （`nmeshvert`）；`mesh_polymap` 为全部 vertex→polygon 映射索引（`nmeshpolymap`）。
#[repr(C)]
pub(crate) struct MjwarpMeshPolyArgs {
    /// 逐 mesh：polygon 数（`nmesh`）。
    pub(crate) mesh_polynum: *const i32,
    /// 逐 mesh：首个 polygon 地址（`nmesh`）。
    pub(crate) mesh_polyadr: *const i32,
    /// 逐 polygon：法线 `(nmeshpoly, 3)` 展平。
    pub(crate) mesh_polynormal: *const f32,
    /// 逐 polygon：顶点首地址（`nmeshpoly`）。
    pub(crate) mesh_polyvertadr: *const i32,
    /// 逐 polygon：顶点数（`nmeshpoly`）。
    pub(crate) mesh_polyvertnum: *const i32,
    /// 全部 polygon 顶点索引（`nmeshpolyvert`）。
    pub(crate) mesh_polyvert: *const i32,
    /// 逐 vertex：映射首地址（`nmeshvert`）。
    pub(crate) mesh_polymapadr: *const i32,
    /// 逐 vertex：映射项数（`nmeshvert`）。
    pub(crate) mesh_polymapnum: *const i32,
    /// 全部 vertex→polygon 映射索引（`nmeshpolymap`）。
    pub(crate) mesh_polymap: *const i32,
}

// SAFETY: `MjwarpMeshPolyArgs` 是纯指针结构（72 字节、align 8、无 Rust 侧隐式填充），
// 只作为设备指针目标整体上传；字段顺序由 `abi_tests.rs` 偏移测试锁死。
unsafe impl cudarc::driver::DeviceRepr for MjwarpMeshPolyArgs {}

/// 产品内核参数（对应 `product.cu::mjwarp_collision_args`，按值传入）。
#[repr(C)]
pub(crate) struct CollisionArgs {
    pub(crate) geom_xpos: *const f32,
    pub(crate) geom_xmat: *const f32,
    pub(crate) geom_size: *const f32,
    pub(crate) geom_margin: *const f32,
    pub(crate) geom_gap: *const f32,
    pub(crate) geom_rbound: *const f32,
    pub(crate) geom_aabb: *const f32,
    pub(crate) geom_type: *const i32,
    pub(crate) geom_dataid: *const i32,
    pub(crate) mesh_vertadr: *const i32,
    pub(crate) mesh_vertnum: *const i32,
    pub(crate) mesh_graphadr: *const i32,
    pub(crate) mesh_octadr: *const i32,
    pub(crate) mesh_vert: *const f32,
    pub(crate) mesh_graph: *const i32,
    pub(crate) hfield_size: *const f32,
    pub(crate) hfield_nrow: *const i32,
    pub(crate) hfield_ncol: *const i32,
    pub(crate) hfield_adr: *const i32,
    pub(crate) hfield_data: *const f32,
    pub(crate) oct_aabb: *const f32,
    pub(crate) oct_child: *const i32,
    pub(crate) oct_coeff: *const f32,
    /// B3/M0a：mesh polygon sidecar 指针；9 个 mesh polygon 数组全空时为 null。
    pub(crate) mesh_poly_args: *const MjwarpMeshPolyArgs,

    pub(crate) nworld: i32,
    pub(crate) ngeom: i32,
    pub(crate) g1: i32,
    pub(crate) g2: i32,
    pub(crate) type1: i32,
    pub(crate) type2: i32,
    pub(crate) geomgeomid: i32,
    pub(crate) naccdmax: i32,
    pub(crate) hfield_id: i32,
    pub(crate) oct_root: i32,
    pub(crate) noct: i32,
    pub(crate) tolerance: f32,
    pub(crate) margin: f32,
    pub(crate) gap: f32,
    pub(crate) gjk_iterations: i32,
    pub(crate) epa_iterations: i32,
    pub(crate) condim: i32,
    pub(crate) pairid0: i32,
    pub(crate) pairid1: i32,
    pub(crate) friction: [f32; 5],
    pub(crate) solref: [f32; 2],
    pub(crate) solreffriction: [f32; 2],
    pub(crate) solimp: [f32; 5],
    pub(crate) adhesion: f32,
    pub(crate) sdf_initpoints: i32,
    pub(crate) sdf_iterations: i32,
    pub(crate) naconmax: i32,
    pub(crate) efc_width: i32,

    pub(crate) out: CollisionContactOut,
    pub(crate) nacon: *mut i32,
    pub(crate) nccd: *mut i32,
    pub(crate) overflow: *mut u32,
}

// SAFETY: `CollisionArgs` 与设备侧 `mjwarp_collision_args` 逐字段对应（指针 + i32/f32
// 固定宽度整数），字段顺序由 `tests.rs` 的偏移测试锁死；无 Rust 侧隐式填充。
unsafe impl cudarc::driver::DeviceRepr for CollisionArgs {}

fn checked_len(factors: &[usize]) -> Option<usize> {
    factors
        .iter()
        .try_fold(1usize, |total, &factor| total.checked_mul(factor))
}

/// 纯 CPU 参数校验：容量/维度为正、`g1`/`g2` 在范围内、geom 基础数组长度匹配。
pub(crate) fn validate_inputs(inputs: &CollisionGpuInputs<'_>) -> Result<(), CollisionGpuError> {
    let meta = &inputs.meta;
    if meta.nworld <= 0 {
        return Err(CollisionGpuError::InvalidArgument("nworld 必须为正"));
    }
    if meta.ngeom <= 0 {
        return Err(CollisionGpuError::InvalidArgument("ngeom 必须为正"));
    }
    if meta.naconmax <= 0 {
        return Err(CollisionGpuError::InvalidArgument("naconmax 必须为正"));
    }
    if meta.efc_width <= 0 {
        return Err(CollisionGpuError::InvalidArgument("efc_width 必须为正"));
    }
    if meta.g1 < 0 || meta.g1 >= meta.ngeom {
        return Err(CollisionGpuError::InvalidArgument("g1 超出 geom 范围"));
    }
    if meta.g2 < 0 || meta.g2 >= meta.ngeom {
        return Err(CollisionGpuError::InvalidArgument("g2 超出 geom 范围"));
    }
    let nworld = meta.nworld as usize;
    let ngeom = meta.ngeom as usize;
    let Some(nworld_ngeom) = checked_len(&[nworld, ngeom]) else {
        return Err(CollisionGpuError::InvalidArgument("nworld*ngeom 溢出"));
    };
    let checks: [(&'static str, usize, Option<usize>); 9] = [
        (
            "geom_xpos 长度须为 nworld*ngeom*3",
            inputs.geom_xpos.len(),
            nworld_ngeom.checked_mul(3),
        ),
        (
            "geom_xmat 长度须为 nworld*ngeom*9",
            inputs.geom_xmat.len(),
            nworld_ngeom.checked_mul(9),
        ),
        (
            "geom_size 长度须为 ngeom*3",
            inputs.geom_size.len(),
            ngeom.checked_mul(3),
        ),
        (
            "geom_margin 长度须为 ngeom",
            inputs.geom_margin.len(),
            Some(ngeom),
        ),
        (
            "geom_gap 长度须为 ngeom",
            inputs.geom_gap.len(),
            Some(ngeom),
        ),
        (
            "geom_rbound 长度须为 ngeom",
            inputs.geom_rbound.len(),
            Some(ngeom),
        ),
        (
            "geom_aabb 长度须为 ngeom*6",
            inputs.geom_aabb.len(),
            ngeom.checked_mul(6),
        ),
        (
            "geom_type 长度须为 ngeom",
            inputs.geom_type.len(),
            Some(ngeom),
        ),
        (
            "geom_dataid 长度须为 ngeom",
            inputs.geom_dataid.len(),
            Some(ngeom),
        ),
    ];
    for (message, actual, expected) in checks {
        if expected != Some(actual) {
            return Err(CollisionGpuError::InvalidArgument(message));
        }
    }
    validate_mesh_poly(inputs)?;
    Ok(())
}

/// mesh polygon sidecar 的 CPU 校验（数组形状语义与 pinned
/// `mujoco_warp/_src/types.py` L1329-1337/L1864-1872 一致，写死）：
///
/// - `nmesh = polynum.len() == polyadr.len()`（逐 mesh）；
/// - `nmeshpoly = polynormal.len()/3 == polyvertadr.len() == polyvertnum.len()`（逐 polygon）；
/// - `nmeshvert = polymapadr.len() == polymapnum.len()`（逐 vertex）；
/// - `sum(polynum) == nmeshpoly`；逐 mesh `polyadr[i]+polynum[i] <= nmeshpoly`；
/// - 逐 polygon `polyvertadr[i]+polyvertnum[i] <= polyvert.len()`；
/// - 逐 vertex `polymapadr[i]+polymapnum[i] <= polymap.len()`；
/// - 强一致：`sum(polyvertnum) == polyvert.len()`、`sum(polymapnum) == polymap.len()`。
///
/// 全空/部分空语义（M0a 写死，本修正片不变）：
/// - 9 个输入 slice 全空 → 不构造 sidecar（`mesh_poly_args = null`），语义为「无 mesh polygon
///   数据、mesh 多接触功能不启用、行为与改动前完全一致」；
/// - 部分为空或长度不自洽 → `InvalidArgument`（不静默禁用，避免掩盖宿主上传 bug）。
fn validate_mesh_poly(inputs: &CollisionGpuInputs<'_>) -> Result<(), CollisionGpuError> {
    let present = [
        !inputs.mesh_polynum.is_empty(),
        !inputs.mesh_polyadr.is_empty(),
        !inputs.mesh_polynormal.is_empty(),
        !inputs.mesh_polyvertadr.is_empty(),
        !inputs.mesh_polyvertnum.is_empty(),
        !inputs.mesh_polyvert.is_empty(),
        !inputs.mesh_polymapadr.is_empty(),
        !inputs.mesh_polymapnum.is_empty(),
        !inputs.mesh_polymap.is_empty(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if present == 0 {
        return Ok(());
    }
    if present != 9 {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 输入不完整：9 个数组必须同时提供或同时为空",
        ));
    }
    // 9 项齐全：按 pinned 形状分段核对长度（逐 mesh / 逐 polygon / 逐 vertex）。
    let nmesh = inputs.mesh_polynum.len();
    if inputs.mesh_polyadr.len() != nmesh {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 mesh 数组长度须一致（polynum/polyadr）",
        ));
    }
    if !inputs.mesh_polynormal.len().is_multiple_of(3) {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_polynormal 长度须为 3 的倍数",
        ));
    }
    let nmeshpoly = inputs.mesh_polynormal.len() / 3;
    if inputs.mesh_polyvertadr.len() != nmeshpoly || inputs.mesh_polyvertnum.len() != nmeshpoly {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 polygon 数组长度须为 nmeshpoly（polyvertadr/polyvertnum）",
        ));
    }
    if inputs.mesh_polymapadr.len() != inputs.mesh_polymapnum.len() {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 vertex 数组长度须一致（polymapadr/polymapnum）",
        ));
    }
    let nmeshvert = inputs.mesh_polymapadr.len();

    // 逐 mesh：polyadr+polynum 落在 nmeshpoly 内，且 sum(polynum) == nmeshpoly。
    let mut total_polygons = 0usize;
    for index in 0..nmesh {
        let (adr, num) = (inputs.mesh_polyadr[index], inputs.mesh_polynum[index]);
        if adr < 0 || num < 0 {
            return Err(CollisionGpuError::InvalidArgument(
                "mesh_polyadr/mesh_polynum 不得为负",
            ));
        }
        if (adr as usize)
            .checked_add(num as usize)
            .is_none_or(|end| end > nmeshpoly)
        {
            return Err(CollisionGpuError::InvalidArgument(
                "mesh_poly 区间超出 mesh_polynormal 行数",
            ));
        }
        total_polygons += num as usize;
    }
    if total_polygons != nmeshpoly {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_polynum 之和须等于 mesh_polynormal 行数",
        ));
    }

    // 逐 polygon：polyvertadr+polyvertnum 落在 mesh_polyvert 内，且 sum(polyvertnum) 吻合。
    let mut total_polygon_vertices = 0usize;
    for index in 0..nmeshpoly {
        let (adr, num) = (
            inputs.mesh_polyvertadr[index],
            inputs.mesh_polyvertnum[index],
        );
        if adr < 0 || num < 0 {
            return Err(CollisionGpuError::InvalidArgument(
                "mesh_polyvertadr/mesh_polyvertnum 不得为负",
            ));
        }
        if (adr as usize)
            .checked_add(num as usize)
            .is_none_or(|end| end > inputs.mesh_polyvert.len())
        {
            return Err(CollisionGpuError::InvalidArgument("mesh_polyvert 区间越界"));
        }
        total_polygon_vertices += num as usize;
    }
    if total_polygon_vertices != inputs.mesh_polyvert.len() {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_polyvertnum 之和须等于 mesh_polyvert 长度",
        ));
    }

    // 逐 vertex：polymapadr+polymapnum 落在 mesh_polymap 内，且 sum(polymapnum) 吻合。
    let mut total_map_entries = 0usize;
    for index in 0..nmeshvert {
        let (adr, num) = (inputs.mesh_polymapadr[index], inputs.mesh_polymapnum[index]);
        if adr < 0 || num < 0 {
            return Err(CollisionGpuError::InvalidArgument(
                "mesh_polymapadr/mesh_polymapnum 不得为负",
            ));
        }
        if (adr as usize)
            .checked_add(num as usize)
            .is_none_or(|end| end > inputs.mesh_polymap.len())
        {
            return Err(CollisionGpuError::InvalidArgument("mesh_polymap 区间越界"));
        }
        total_map_entries += num as usize;
    }
    if total_map_entries != inputs.mesh_polymap.len() {
        return Err(CollisionGpuError::InvalidArgument(
            "mesh_polymapnum 之和须等于 mesh_polymap 长度",
        ));
    }
    Ok(())
}
