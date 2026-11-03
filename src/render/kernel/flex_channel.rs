//! R06-S2：宿主 flex 通道（9 个 flex 数组 + `bvh_nflexgeom` 标量）、显式校验与设备上传。
//!
//! 上游语义（pinned `71da24d`：`render_util.py` L490-512 / L740-748，`render.py` L1520-1583）：
//! - `flex_vertadr`：`[nflex]`；`flex_edge`：`[nflexedge]`（`vec2i`）；`flex_radius`：`[nflex]`；
//!   `flexvert_xpos_in`：`[nworld, nflexvert]`（data 顶点世界位姿，`vec3`）；`flex_rgba`：
//!   `[nflex]`（`vec4`）；`flex_bvh_id`：`[nflex]`（flex BVH 句柄；S3 填真值，本片由调用方供）；
//!   `flex_group_root`：`[nworld, nflex]`（上游初值 -1，S3 由 flex BVH 填）；
//!   `flex_geom_flexid` / `flex_geom_edgeid`：scene BVH flex 图元映射，长度 = `bvh_nflexgeom`。
//! - 本文件只建宿主通道：类型、显式校验（不 panic、不静默回退）、上传与形状口径；
//!   **model/context 的真实 flex 数据源归 R06-S2b**，设备 cast/BVH 分支归 S3（`.cu` 与其声明零改动）。
//! - 空通道（`nflex == 0` 且全部数组为空）与 `None` 等价：保持 S7b2 的 1 元素占位行为；仅
//!   `flex_group_root` / `flexvert_xpos` 两槽形状按上游改 2-D。81 槽槽数/槽类型不变
//!   ⇒ **不构成 ABI 变更**（S0 对表见 `docs/development-evidence/render-bvh/r06-flex-precheck/`）。

use crate::runtime::launcher::LauncherError;
use crate::runtime::{TransferBuffer, TransferSession};

use super::super::megakernel_arrays::upload;

/// 宿主 flex 通道输入（借用切片；长度口径见 [`FlexHostInputs::validate`]）。
///
/// 供数侧（S2b）负责把原生模型/数据转成本结构的形状与精度：上游 MjModel 的 `flex_radius`
/// 是 `mjtNum`（f64），按上游 `wp.array[float]` 口径转 f32；`flex_rgba` 是 MjModel 的
/// `float`（f32）；`flexvert_xpos` 取 flex 物理输出的 f32 世界位姿。本片不做任何近似或回退。
#[derive(Clone, Copy, Debug)]
pub struct FlexHostInputs<'a> {
    /// 发射世界数（须等于 `resolved.nworld`）。
    pub nworld: usize,
    /// 柔体数（上游 `mjm.nflex`）。
    pub nflex: usize,
    /// 全部柔体顶点数（上游 `mjm.nflexvert`）。
    pub nflexvert: usize,
    /// `[nflex]`：每柔体首顶点地址。
    pub flex_vertadr: &'a [i32],
    /// 展平 `[nflexedge, 2]`：每条边的两端顶点号（柔体局部）。
    pub flex_edge: &'a [i32],
    /// `[nflex]`：每柔体图元半径。
    pub flex_radius: &'a [f32],
    /// 展平 `[nworld, nflexvert, 3]`：data 顶点世界位姿。
    pub flexvert_xpos: &'a [f32],
    /// 展平 `[nflex, 4]`：每柔体 rgba。
    pub flex_rgba: &'a [f32],
    /// `[nflex]`：flex BVH 句柄（S3 填真值；S2 由调用方供 0）。
    pub flex_bvh_id: &'a [u64],
    /// 展平 `[nworld, nflex]`：上游初值 -1，S3 由 flex BVH 填。
    pub flex_group_root: &'a [i32],
    /// `[bvh_nflexgeom]`：scene BVH flex 图元 → 柔体 id。
    pub flex_geom_flexid: &'a [i32],
    /// `[bvh_nflexgeom]`：scene BVH flex 图元 → 边号；2D/3D 图元为 -1。
    pub flex_geom_edgeid: &'a [i32],
}

/// 校验后的 flex 计数（`bvh_nflexgeom` 标量与各槽形状的口径来源）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FlexCounts {
    pub(crate) nflex: usize,
    pub(crate) nflexvert: usize,
    pub(crate) nflexedge: usize,
    pub(crate) bvh_nflexgeom: usize,
}

impl FlexHostInputs<'_> {
    /// 零 flex 空通道：计数为 0 且全部数组为空（与 `None` 等价）。
    pub fn is_empty(&self) -> bool {
        self.nflex == 0
            && self.nflexvert == 0
            && self.flex_vertadr.is_empty()
            && self.flex_edge.is_empty()
            && self.flex_radius.is_empty()
            && self.flexvert_xpos.is_empty()
            && self.flex_rgba.is_empty()
            && self.flex_bvh_id.is_empty()
            && self.flex_group_root.is_empty()
            && self.flex_geom_flexid.is_empty()
            && self.flex_geom_edgeid.is_empty()
    }

    /// 校验通道口径（含与发射侧 `nworld` 的一致性）。
    ///
    /// 与 [`Self::validate_counts`] 同一错误面：世界数不一致、计数与数组长度不一致、
    /// `flex_vertadr` 非 0 起点 / 非降 / 越界、`flex_edge` 奇数长度或端点越界、
    /// geom 映射长度不一致或 id 越界，全部返回显式错误（不 panic、不静默回退）。
    pub fn validate(&self, nworld: usize) -> Result<(), LauncherError> {
        self.validate_counts(nworld).map(|_| ())
    }

    /// 校验通道口径并给出计数（crate 内供上传与实参形状使用）。
    ///
    /// 拒绝面（返回 [`LauncherError::InvalidArray`] / [`LauncherError::Overflow`]，不 panic）：
    /// 世界数不一致、计数与数组长度不一致、`flex_vertadr` 非 0 起点 / 非降 / 越界、
    /// `flex_edge` 奇数长度或端点越界、geom 映射长度不一致或 id 越界。
    ///
    /// 本通道不携带、因而无法校验的两条上游语义（由 S2b/S3 负责）：`flex_vertadr[f] +
    /// flex_vertnum[f] == nflexvert` 的完整累加、1D/2D/3D 与 `flex_geom_edgeid == -1` 的对应关系
    /// （需要 `flex_vertnum` / `flex_dim` / `flex_edgenum`）。
    pub(crate) fn validate_counts(&self, nworld: usize) -> Result<FlexCounts, LauncherError> {
        if self.is_empty() {
            return Ok(FlexCounts::default());
        }
        if self.nworld != nworld {
            return Err(LauncherError::InvalidArray("flex_nworld"));
        }
        if self.nflex == 0 {
            return Err(LauncherError::InvalidArray("flex_nflex"));
        }
        if self.nflexvert == 0 {
            return Err(LauncherError::InvalidArray("flex_nflexvert"));
        }
        let nflex = self.nflex;
        let nflexvert = self.nflexvert;
        if self.flex_vertadr.len() != nflex {
            return Err(LauncherError::InvalidArray("flex_vertadr"));
        }
        if self.flex_radius.len() != nflex {
            return Err(LauncherError::InvalidArray("flex_radius"));
        }
        let rgba_len = nflex
            .checked_mul(4)
            .ok_or(LauncherError::Overflow("flex_rgba"))?;
        if self.flex_rgba.len() != rgba_len {
            return Err(LauncherError::InvalidArray("flex_rgba"));
        }
        if self.flex_bvh_id.len() != nflex {
            return Err(LauncherError::InvalidArray("flex_bvh_id"));
        }
        let verts_per_world = nflexvert
            .checked_mul(3)
            .ok_or(LauncherError::Overflow("flexvert_xpos"))?;
        let xpos_len = self
            .nworld
            .checked_mul(verts_per_world)
            .ok_or(LauncherError::Overflow("flexvert_xpos"))?;
        if self.flexvert_xpos.len() != xpos_len {
            return Err(LauncherError::InvalidArray("flexvert_xpos"));
        }
        let group_root_len = self
            .nworld
            .checked_mul(nflex)
            .ok_or(LauncherError::Overflow("flex_group_root"))?;
        if self.flex_group_root.len() != group_root_len {
            return Err(LauncherError::InvalidArray("flex_group_root"));
        }
        if !self.flex_edge.len().is_multiple_of(2) {
            return Err(LauncherError::InvalidArray("flex_edge"));
        }
        if self.flex_geom_flexid.len() != self.flex_geom_edgeid.len() {
            return Err(LauncherError::InvalidArray("flex_geom_edgeid"));
        }
        // flex_vertadr：首项必须为 0、非降、每项落在 [0, nflexvert)。
        let mut previous: i32 = -1;
        for (flex, &address) in self.flex_vertadr.iter().enumerate() {
            let in_range = usize::try_from(address).is_ok_and(|value| value < nflexvert);
            let starts_at_zero = flex != 0 || address == 0;
            if !in_range || !starts_at_zero || address < previous {
                return Err(LauncherError::InvalidArray("flex_vertadr"));
            }
            previous = address;
        }
        let nflexedge = self.flex_edge.len() / 2;
        // 边端点：柔体局部顶点号，必须落在 [0, nflexvert)。
        for &vertex in self.flex_edge {
            if !usize::try_from(vertex).is_ok_and(|value| value < nflexvert) {
                return Err(LauncherError::InvalidArray("flex_edge"));
            }
        }
        // geom 映射：flex id 落在 [0, nflex)；边号 -1（2D/3D 图元）或落在 [0, nflexedge)。
        for (&flexid, &edgeid) in self.flex_geom_flexid.iter().zip(self.flex_geom_edgeid) {
            if !usize::try_from(flexid).is_ok_and(|value| value < nflex) {
                return Err(LauncherError::InvalidArray("flex_geom_flexid"));
            }
            let edge_ok =
                edgeid == -1 || usize::try_from(edgeid).is_ok_and(|value| value < nflexedge);
            if !edge_ok {
                return Err(LauncherError::InvalidArray("flex_geom_edgeid"));
            }
        }
        Ok(FlexCounts {
            nflex,
            nflexvert,
            nflexedge,
            bvh_nflexgeom: self.flex_geom_flexid.len(),
        })
    }
}

/// `flex_vertadr` 槽形状：`[max(nflex, 1)]`（渲染 ABI 拒绝零维，空通道落 1 元素占位）。
pub(crate) fn flex_vertadr_shape(counts: FlexCounts) -> [usize; 1] {
    [counts.nflex.max(1)]
}

/// `flex_edge` 槽形状：`[max(nflexedge, 1)]`。
pub(crate) fn flex_edge_shape(counts: FlexCounts) -> [usize; 1] {
    [counts.nflexedge.max(1)]
}

/// `flex_radius` / `flex_rgba` / `flex_bvh_id` 槽形状：`[max(nflex, 1)]`。
pub(crate) fn flex_model_shape(counts: FlexCounts) -> [usize; 1] {
    [counts.nflex.max(1)]
}

/// `flexvert_xpos` 槽形状：2-D `[nworld, max(nflexvert, 1)]`（上游 `wp.array2d`）。
pub(crate) fn flexvert_xpos_shape(nworld: usize, nflexvert: usize) -> [usize; 2] {
    [nworld.max(1), nflexvert.max(1)]
}

/// `flex_group_root` 槽形状：2-D `[nworld, max(nflex, 1)]`（上游 `wp.array2d`）。
pub(crate) fn flex_group_root_shape(nworld: usize, nflex: usize) -> [usize; 2] {
    [nworld.max(1), nflex.max(1)]
}

/// `flex_geom_flexid` / `flex_geom_edgeid` 槽形状：`[max(bvh_nflexgeom, 1)]`。
pub(crate) fn flex_geom_shape(counts: FlexCounts) -> [usize; 1] {
    [counts.bvh_nflexgeom.max(1)]
}

/// 上传后的 flex 家族设备缓冲与计数（`megakernel_uploads.rs` 持有，实参装配读形状）。
pub(super) struct FlexUploads {
    pub(super) flex_vertadr: TransferBuffer<i32>,
    pub(super) flex_edge: TransferBuffer<i32>,
    pub(super) flex_radius: TransferBuffer<f32>,
    pub(super) flexvert_xpos: TransferBuffer<f32>,
    pub(super) flex_bvh_id: TransferBuffer<u32>,
    pub(super) flex_group_root: TransferBuffer<i32>,
    pub(super) flex_rgba: TransferBuffer<f32>,
    pub(super) flex_geom_flexid: TransferBuffer<i32>,
    pub(super) flex_geom_edgeid: TransferBuffer<i32>,
    pub(super) counts: FlexCounts,
}

/// 上传 flex 家族；`None` 或空通道落 S7b2 等价占位（1 元素占位 + `max(1)` 形状口径）。
pub(super) fn upload_flex_arrays(
    session: &TransferSession,
    nworld: usize,
    flex: Option<&FlexHostInputs<'_>>,
) -> Result<FlexUploads, LauncherError> {
    let counts = match flex {
        Some(inputs) => inputs.validate_counts(nworld)?,
        None => FlexCounts::default(),
    };
    let (vertadr, edge, radius, xpos, rgba, bvh_id, group_root, geom_flexid, geom_edgeid) =
        match flex {
            Some(inputs) if counts.nflex != 0 => (
                inputs.flex_vertadr.to_vec(),
                inputs.flex_edge.to_vec(),
                inputs.flex_radius.to_vec(),
                inputs.flexvert_xpos.to_vec(),
                inputs.flex_rgba.to_vec(),
                bvh_id_words(inputs.flex_bvh_id),
                inputs.flex_group_root.to_vec(),
                inputs.flex_geom_flexid.to_vec(),
                inputs.flex_geom_edgeid.to_vec(),
            ),
            // 零 flex 等价占位：数据按 `max(1)` 形状展平（2-D 槽按 nworld 展平）。
            _ => (
                vec![0i32],
                vec![0i32; 2],
                vec![0.0f32],
                vec![0.0f32; 3 * nworld],
                vec![0.0f32; 4],
                bvh_id_words(&[]),
                vec![0i32; nworld],
                vec![0i32],
                vec![0i32],
            ),
        };
    Ok(FlexUploads {
        flex_vertadr: upload(session, &vertadr)?,
        flex_edge: upload(session, &edge)?,
        flex_radius: upload(session, &radius)?,
        flexvert_xpos: upload(session, &xpos)?,
        flex_bvh_id: upload(session, &bvh_id)?,
        flex_group_root: upload(session, &group_root)?,
        flex_rgba: upload(session, &rgba)?,
        flex_geom_flexid: upload(session, &geom_flexid)?,
        flex_geom_edgeid: upload(session, &geom_edgeid)?,
        counts,
    })
}

/// `u64` 数组 → 小端 u32 字数对（`TransferElement` 无 u64；与 hfield 槽同款承载）。
fn bvh_id_words(values: &[u64]) -> Vec<u32> {
    if values.is_empty() {
        return vec![0u32, 0];
    }
    let mut words = Vec::with_capacity(2 * values.len());
    for id in values {
        words.push(*id as u32);
        words.push((*id >> 32) as u32);
    }
    words
}

// R06-S2：单测拆到子文件（守「代码文件 ≤ 500 行」棘轮；沿 `src/model/flex/tests.rs` 先例）。
#[cfg(test)]
#[path = "flex_channel/tests.rs"]
mod tests;
