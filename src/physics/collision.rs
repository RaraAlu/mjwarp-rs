//! G07@P5 宽相碰撞切片的宿主侧实现：`nxn_broadphase` / `sap_broadphase`。
//!
//! 对齐冻结上游 `mujoco_warp._src.collision_driver`（U007/U008；G07～G11；
//! P5/P7，提交 `71da24d956…`，Apache-2.0）的候选生成子集：
//!
//! - 输入：冻结模型字段（geom 类型/`geom_dataid`/AABB/rbound/margin/gap）、
//!   上游 `put_model` 预计算的过滤后 geom 对表与全量 `nxn_pairid` 上三角、
//!   冻结状态（每世界 `geom_xpos`/`geom_xmat`）与选项（`BroadphaseFilter`
//!   位模式、`naconmax` 容量）。
//! - 输出：候选对（按 geom 类型排序的 pair）、`[contact_id, collision_id]`、
//!   世界编号与 `ncollision`。`ncollision` 统计全部通过候选（可超容量），
//!   写出列表截断到容量（对齐上游 `_add_geom_pair` 的原子计数）。
//! - 语义：平面 `rbound == 0` 只走 plane filter；非平面按 SPHERE/AABB/OBB
//!   位逐项预筛（未置位则不检查）；排除对由调用方对表承担（NXN 用过滤后
//!   对表，SAP 用全量上三角并跳过 `pairid[0] < -1` 且 `pairid[1] < 0`）；
//!   collision sensor 对（`pairid[1] >= 0`）绕过过滤强制包含；世界批量
//!   `geom_dataid` 缺失 mesh（`-1`）在两条路径都跳过。
//!
//! 未验收边界：SLEEP 睡眠跳过、incremental awake 复跑、AABB/rbound/margin/
//! gap 的世界批量语义（本片按 `world % batches` 取批但不做批量语义验收）、
//! flex 窄相、SDF GPU 内核（宿主数学已由 G3 落地）与图条件、完整 `collision()` 驱动与接触输出。

use std::error::Error;
use std::fmt;

mod filter;
mod narrowphase;
pub use narrowphase::{
    GeomContact, MJ_MINMU, MJ_MINVAL, NarrowphaseError, PrimitiveModel, PrimitiveState, make_frame,
    plane_box, plane_sphere, primitive_narrowphase, sphere_sphere,
};
#[allow(dead_code)]
// GJK/EPA 在 G5 GPU 内核片接入前只由测试与后续片使用。
mod gjk;
mod nxn;
mod sap;
#[cfg(test)]
mod tests;

// GJK/EPA 凸碰撞（G10@P5 前置片）：`ccd`/`ccd_pair` 与 geom/mesh 视图。
// 目前只由 lib 内测试（`gjk/case_compare.rs`）与后续 G3–G5 片使用。
#[allow(unused_imports)]
pub use gjk::geom_type;
#[allow(unused_imports)]
pub use gjk::{
    CcdContact, CcdOptions, CcdOutput, CcdResult, ContactOptions, Geom, MeshView, ccd, ccd_pair,
};
// G3 宿主体积 SDF 数学（mujoco_warp/_src/collision_sdf.py 子集）。
// G5 GPU 内核片接入前只由测试与后续片使用。
#[allow(dead_code)]
pub mod sdf;

// G4 hfield 宿主窄相（mujoco_warp/_src/collision_convex.py::ccd_hfield_kernel 子集）。
// G5 GPU 内核片接入前只由测试与后续片使用。
#[allow(dead_code)]
pub mod hfield;

// G5：`relative_matrix` 的上游语义（transpose(a)*b，行优先）与 CPU 可验证单测。
#[allow(dead_code)]
pub(crate) mod relmat;

// G5 碰撞 GPU 内核（NVRTC 设备源码 + 宿主编译/接线）；运行需 cuda-probe 与 GPU 窗口。
#[cfg(feature = "cuda-probe")]
pub mod cuda;

/// 上游 `mjtGeom` 的平面、球、盒与网格编号。
pub const GEOM_PLANE: i32 = 0;
pub const GEOM_SPHERE: i32 = 2;
pub const GEOM_BOX: i32 = 6;
pub const GEOM_MESH: i32 = 7;
/// 上游 `mjMAXVAL`（f32）。
pub(crate) const MJ_MAXVAL: f32 = 1.0e10;

/// 上游 `BroadphaseType`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BroadphaseType {
    Nxn,
    SapTile,
    SapSegmented,
}

impl BroadphaseType {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Nxn),
            1 => Some(Self::SapTile),
            2 => Some(Self::SapSegmented),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            Self::Nxn => 0,
            Self::SapTile => 1,
            Self::SapSegmented => 2,
        }
    }
}

/// 上游 `BroadphaseFilter` 位模式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BroadphaseFilter(pub u32);

impl BroadphaseFilter {
    pub const PLANE: Self = Self(1 << 0);
    pub const SPHERE: Self = Self(1 << 1);
    pub const AABB: Self = Self(1 << 2);
    pub const OBB: Self = Self(1 << 3);
    pub const ALL: Self = Self(15);

    pub fn bits(self) -> u32 {
        self.0
    }

    pub(crate) fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// 冻结 geom 对记录：上游 `nxn_geom_pair_filtered` 与 `nxn_pairid_filtered`
/// 的逐行合并（NXN 路径使用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeomPair {
    pub geom1: i32,
    pub geom2: i32,
    pub contact_pairid: i32,
    pub collision_pairid: i32,
}

/// 宽相模型字段。
///
/// AABB/rbound/margin/gap 为世界批量数组（`*_batches × ngeom` 展开，按
/// `world % batches` 取批）；`geom_dataid` 与上游一致支持世界批量，
/// 批量大于 1 时 `dataid < 0` 的 mesh geom 视为该世界缺失。
#[derive(Clone, Copy, Debug)]
pub struct BroadphaseModel<'a> {
    pub ngeom: usize,
    pub geom_type: &'a [i32],
    pub geom_dataid_batches: usize,
    pub geom_dataid: &'a [i32],
    pub aabb_batches: usize,
    pub geom_aabb_center: &'a [[f32; 3]],
    pub geom_aabb_size: &'a [[f32; 3]],
    pub rbound_batches: usize,
    pub geom_rbound: &'a [f32],
    pub margin_batches: usize,
    pub geom_margin: &'a [f32],
    pub gap_batches: usize,
    pub geom_gap: &'a [f32],
    /// 上游 `nxn_geom_pair_filtered` + `nxn_pairid_filtered`。
    pub pairs: &'a [GeomPair],
    /// 上游 `nxn_pairid` 全量上三角（`i < j` 行优先，见 `upper_tri_index`）。
    pub full_pairid: &'a [[i32; 2]],
}

impl BroadphaseModel<'_> {
    pub(crate) fn aabb(&self, world: usize, geom: usize) -> ([f32; 3], [f32; 3]) {
        let index = (world % self.aabb_batches) * self.ngeom + geom;
        (self.geom_aabb_center[index], self.geom_aabb_size[index])
    }

    pub(crate) fn rbound(&self, world: usize, geom: usize) -> f32 {
        self.geom_rbound[(world % self.rbound_batches) * self.ngeom + geom]
    }

    pub(crate) fn margin(&self, world: usize, geom: usize) -> f32 {
        self.geom_margin[(world % self.margin_batches) * self.ngeom + geom]
    }

    pub(crate) fn gap(&self, world: usize, geom: usize) -> f32 {
        self.geom_gap[(world % self.gap_batches) * self.ngeom + geom]
    }

    pub(crate) fn mesh_missing(&self, world: usize, geom: usize) -> bool {
        self.geom_dataid_batches > 1
            && self.geom_type[geom] == GEOM_MESH
            && self.geom_dataid[(world % self.geom_dataid_batches) * self.ngeom + geom] < 0
    }
}

/// 冻结每世界 geom 位姿（世界坐标，`mat` 行主序 3×3）。
#[derive(Clone, Copy, Debug)]
pub struct BroadphaseState<'a> {
    pub nworld: usize,
    pub geom_xpos: &'a [[f32; 3]],
    pub geom_xmat: &'a [[f32; 9]],
}

/// 宽相选项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BroadphaseOptions {
    pub filter: BroadphaseFilter,
    pub capacity: usize,
}

/// 宽相候选输出。
///
/// `pairs`/`pairids`/`worldids` 长度同为 `min(ncollision, capacity)`；
/// 上游写出顺序由原子计数决定（非确定性），比较时按集合规范化。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BroadphaseOutput {
    pub ncollision: usize,
    pub pairs: Vec<[i32; 2]>,
    pub pairids: Vec<[i32; 2]>,
    pub worldids: Vec<i32>,
}

/// 宽相错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BroadphaseError {
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
    InvalidBatch {
        field: &'static str,
        batches: usize,
    },
    UnsupportedBroadphase {
        value: i32,
    },
}

impl fmt::Display for BroadphaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(formatter, "索引越界：{field}[{index}]，上限 {limit}"),
            Self::InvalidBatch { field, batches } => {
                write!(formatter, "批量不合法：{field} = {batches}")
            }
            Self::UnsupportedBroadphase { value } => {
                write!(formatter, "不支持的宽相类型：{value}")
            }
        }
    }
}

impl Error for BroadphaseError {}

/// 候选写出器：统计全部通过候选，只写出前 `capacity` 条（对齐上游原子计数）。
#[derive(Debug)]
pub(crate) struct CandidateWriter {
    output: BroadphaseOutput,
    capacity: usize,
}

impl CandidateWriter {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            output: BroadphaseOutput::default(),
            capacity,
        }
    }

    pub(crate) fn add(
        &mut self,
        world: usize,
        geom1: i32,
        geom2: i32,
        type1: i32,
        type2: i32,
        pairid: [i32; 2],
    ) {
        let collisionid = self.output.ncollision;
        self.output.ncollision += 1;
        if collisionid >= self.capacity {
            return;
        }
        let pair = if type1 > type2 {
            [geom2, geom1]
        } else {
            [geom1, geom2]
        };
        self.output.pairs.push(pair);
        self.output.pairids.push(pairid);
        self.output.worldids.push(world as i32);
    }

    pub(crate) fn finish(self) -> BroadphaseOutput {
        self.output
    }
}

/// 上游 `upper_tri_index`：`ngeom × ngeom` 上三角行优先编号。
pub(crate) fn upper_tri_index(ngeom: usize, geom1: usize, geom2: usize) -> usize {
    let (i, j) = if geom2 < geom1 {
        (geom2, geom1)
    } else {
        (geom1, geom2)
    };
    i * (2 * ngeom - i - 3) / 2 + j - 1
}

/// 校验冻结输入形状；不改动输入。
pub fn validate(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
) -> Result<(), BroadphaseError> {
    let ngeom = model.ngeom;
    for (field, batches) in [
        ("geom_dataid", model.geom_dataid_batches),
        ("geom_aabb", model.aabb_batches),
        ("geom_rbound", model.rbound_batches),
        ("geom_margin", model.margin_batches),
        ("geom_gap", model.gap_batches),
    ] {
        if batches == 0 {
            return Err(BroadphaseError::InvalidBatch { field, batches });
        }
    }
    for (field, actual, width) in [
        ("geom_type", model.geom_type.len(), 1),
        (
            "geom_dataid",
            model.geom_dataid.len(),
            model.geom_dataid_batches,
        ),
        (
            "geom_aabb_center",
            model.geom_aabb_center.len(),
            model.aabb_batches,
        ),
        (
            "geom_aabb_size",
            model.geom_aabb_size.len(),
            model.aabb_batches,
        ),
        ("geom_rbound", model.geom_rbound.len(), model.rbound_batches),
        ("geom_margin", model.geom_margin.len(), model.margin_batches),
        ("geom_gap", model.geom_gap.len(), model.gap_batches),
    ] {
        let expected = ngeom
            .checked_mul(width)
            .ok_or(BroadphaseError::InvalidLength {
                field,
                expected: usize::MAX,
                actual,
            })?;
        if actual != expected {
            return Err(BroadphaseError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    let expected_pairs = ngeom
        .checked_mul(ngeom.saturating_sub(1))
        .map(|value| value / 2)
        .ok_or(BroadphaseError::InvalidLength {
            field: "full_pairid",
            expected: usize::MAX,
            actual: model.full_pairid.len(),
        })?;
    if model.full_pairid.len() != expected_pairs {
        return Err(BroadphaseError::InvalidLength {
            field: "full_pairid",
            expected: expected_pairs,
            actual: model.full_pairid.len(),
        });
    }
    for pair in model.pairs {
        for geom in [pair.geom1, pair.geom2] {
            let geom = geom as usize;
            if pair.geom1 < 0 || pair.geom2 < 0 || geom >= ngeom {
                return Err(BroadphaseError::InvalidIndex {
                    field: "pairs",
                    index: geom,
                    limit: ngeom,
                });
            }
        }
    }
    for (field, actual, width) in [
        ("geom_xpos", state.geom_xpos.len(), 3),
        ("geom_xmat", state.geom_xmat.len(), 9),
    ] {
        let _ = width;
        let expected = state
            .nworld
            .checked_mul(ngeom)
            .ok_or(BroadphaseError::InvalidLength {
                field,
                expected: usize::MAX,
                actual,
            })?;
        if actual != expected {
            return Err(BroadphaseError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

/// NXN 宽相：遍历上游过滤后的 geom 对表逐个预筛并写出候选。
///
/// 对齐 `nxn_broadphase`（U007）：缺失 mesh 跳过、四位过滤、collision
/// sensor 强制包含、按 geom 类型排序写出、`naconmax` 容量截断。
pub fn nxn_broadphase(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    options: BroadphaseOptions,
) -> Result<BroadphaseOutput, BroadphaseError> {
    validate(model, state)?;
    Ok(nxn::run(model, state, options))
}

/// SAP 宽相：固定方向投影 + 排序 + 扫描，再复用同一过滤与写出语义。
///
/// 对齐 `sap_broadphase`（U008）；`SAP_TILE` 与 `SAP_SEGMENTED` 只差上游
/// 排序策略，本片两条都走同一宿主排序但候选集合相同（上游写出顺序由
/// 原子计数决定，非确定性）。
pub fn sap_broadphase(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    broadphase: BroadphaseType,
    options: BroadphaseOptions,
) -> Result<BroadphaseOutput, BroadphaseError> {
    if !matches!(
        broadphase,
        BroadphaseType::SapTile | BroadphaseType::SapSegmented
    ) {
        return Err(BroadphaseError::UnsupportedBroadphase {
            value: broadphase.as_i32(),
        });
    }
    validate(model, state)?;
    Ok(sap::run(model, state, options))
}
