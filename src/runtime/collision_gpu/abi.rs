//! G10@P5：产品内核参数 ABI（与 `product.cu` 的 `mjwarp_collision_args` 逐字段对应）。
//!
//! 布局按 64 位 C ABI 推导（指针 8 字节、`int`/`float` 4 字节、嵌套结构按 8 字节对齐）：
//!
//! ```text
//! 偏移  字段                 偏移  字段
//! 0     geom_xpos            184   nworld
//! 8     geom_xmat            228   tolerance
//! ...                        ...
//! 176   oct_coeff            332   efc_width
//!                           336   out（d_contact_out，136 字节）
//!                            472   nacon
//!                            480   nccd
//!                            488   overflow
//! 总大小 496，对齐 8
//! ```
//!
//! 完整偏移表由 `tests.rs` 的 `abi_*` 测试逐项锁死；字段顺序**必须**与探针 `ProbeArgs`
//! 完全一致（同 frozen 上游 revision 71da24d）。
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
    Ok(())
}
