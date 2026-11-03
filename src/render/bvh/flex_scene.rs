//! R06-S3c：2D/3D flex 设备接线的宿主装配（渲染/BVH R06）。
//!
//! 输入是 S2c 的 `FlexTopologyFields`（真实 shell/elem 拓扑）+ render 桶的 `flex_dim` /
//! `flex_radius`、position 桶的 `flex_vertadr` / `flexvert_xpos`、以及 S2b 的 geom 映射
//! （`flex_geom_flexid/edgeid`）。输出两块：
//!
//! 1. **场景 flex 叶**（[`crate::render::bvh::FlexSceneLeaves`]）：1D = 边胶囊包围盒，
//!    2D/3D = 该 flex 顶点整体包围盒 ± 半径（上游 `bvh.py::_compute_flex_bvh_bounds` L265-311）；
//! 2. **设备 flex mesh 段**（[`FlexMeshDevice`]，布局细节见 `flex_scene/device.rs`）：
//!    每个 `(flex, world)` 一个世界空间三角 BVH，按 flex 主序/world 次序连续排布，宿主把它
//!    追加到 `MeshDevice` 末尾即可复用既有 blob 打包/上传路径（不改 `pack_blob` 签名）。
//!    设备侧条目索引 = `bvh_ids[flexid] + worldid`（1D/无 mesh 的 flex 为 `u64::MAX` 哨兵）。
//!
//! 与上游的对应关系（pinned `71da24d`）：
//! - `render_util.py` L490-512：2D/3D 每 flex 一个 mesh 图元，命中走 `ray_flex_with_bvh`；
//! - `bvh.py` L994-1109 / L1112-1179：flex mesh 构建/重拟合（面序移植见 `bvh::flex_mesh`）；
//! - `ray.py` L759-802：flex 已在世界空间、无 `_ray_map`、无多段 march；`group_root` = 该
//!   world 分组的子树根 —— 本实现把每个 `(flex, world)` 单独建树，`root` 即该树根，语义等价
//!   （三角 BVH 节点布局是自主分箱 SAH，验收只看语义命中）。
//!
//! 每帧从当前 `flexvert_xpos` 重建（上游每帧 `refit_flex_bvh` 语义的等价实现；`bvh::flex_mesh`
//! 的 `FlexMeshBvh::refit` 供未来持久 registry 用）。不跑 GPU、不生成参考。

use super::flex_mesh::FlexMeshError;
use super::{Aabb, FlexLeafError, FlexSceneLeaves};

pub use device::{FLEX_NO_MESH, FlexMeshDevice};

/// S3c host 装配错误；全部在分配/建树前给出显式错误（不 panic、不静默回退）。
#[derive(Clone, Debug, PartialEq)]
pub enum FlexSceneError {
    /// 维度非法（只接受 1/2/3）。
    UnsupportedDimension { flex: usize, dim: i32 },
    /// 长度/计数不一致（字段名 + 期望/实际）。
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 计数/索引非法或溢出。
    Count { field: &'static str },
    /// 非有限值（半径/顶点位置）。
    NonFinite { field: &'static str, index: usize },
    /// 2D/3D 元素/壳顶点宽度与 `dim` 不符（上游 `dim+1 per elem` / `dim per shell`）。
    TopologyWidth { flex: usize, field: &'static str },
    /// 1D 叶路径遇到未知映射（`flexid < 0`）。
    InvalidMapping { index: usize },
    /// 场景叶显式错误（复用 S3a 口径）。
    Leaf(FlexLeafError),
    /// flex mesh 面/树构建错误（复用 S3a 机制）。
    Mesh(FlexMeshError),
    /// S2c 拓扑字段自校验失败（`FlexTopologyFields::validate`；不 panic）。
    Topology(String),
}

impl std::fmt::Display for FlexSceneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDimension { flex, dim } => {
                write!(formatter, "flex[{flex}] 维度 {dim} 非法（只支持 1/2/3）")
            }
            Self::Length {
                field,
                expected,
                actual,
            } => write!(formatter, "{field} 长度 {actual} != {expected}"),
            Self::Count { field } => write!(formatter, "{field} 计数/索引非法"),
            Self::NonFinite { field, index } => write!(formatter, "{field}[{index}] 不是有限数"),
            Self::TopologyWidth { flex, field } => {
                write!(
                    formatter,
                    "flex[{flex}] 的 {field} 顶点宽度与 flex_dim 不符"
                )
            }
            Self::InvalidMapping { index } => {
                write!(formatter, "geom 映射[{index}] 的 flexid 非法")
            }
            Self::Leaf(error) => write!(formatter, "flex 场景叶：{error}"),
            Self::Mesh(error) => write!(formatter, "flex mesh：{error}"),
            Self::Topology(message) => write!(formatter, "flex 拓扑：{message}"),
        }
    }
}

impl std::error::Error for FlexSceneError {}

/// S3c 宿主装配输入（全部借用；`flexvert_xpos` 展平 `[world, vert, 3]`）。
pub struct FlexMeshChannel<'a> {
    pub topology: &'a crate::model::flex::FlexTopologyFields,
    pub flex_dim: &'a [i32],
    pub flex_vertadr: &'a [i32],
    pub flex_radius: &'a [f32],
    pub flexvert_xpos: &'a [f32],
    pub flex_edge: &'a [i32],
    pub geom_flexid: &'a [i32],
    pub geom_edgeid: &'a [i32],
    pub worlds: usize,
    pub nflexvert: usize,
}

/// 装配结果：场景 flex 叶 + 设备 flex mesh 段 + 逐 flex 条目基址。
#[derive(Clone, Debug)]
pub struct FlexMeshAssembly {
    pub leaves: FlexSceneLeaves,
    pub device: FlexMeshDevice,
    /// 长度 = nflex；build 期存**段内相对条目号**（1D/无 mesh 的 flex 为 `u64::MAX`）；
    /// [`Self::append_to_mesh_device`] 追加到 `MeshDevice` 后转成 blob 绝对条目号
    /// （设备条目索引 = `flex_bvh_id[flexid] + worldid`）。
    pub bvh_ids: Vec<u64>,
    /// 2D/3D mesh 条目数（= 该 flex 的 world 数累加）。
    pub mesh_count: usize,
}

impl FlexMeshAssembly {
    /// S3c host 入口：校验通道形状后构建 2D/3D 设备段；`None` = 无 mesh 通道。
    pub fn prepare(
        channel: Option<&FlexMeshChannel<'_>>,
        nworld: usize,
        nflex: usize,
    ) -> Result<Option<Self>, FlexSceneError> {
        let Some(channel) = channel else {
            return Ok(None);
        };
        if channel.worlds != nworld || channel.flex_dim.len() != nflex {
            return Err(FlexSceneError::Length {
                field: "flex_mesh_worlds",
                expected: nworld,
                actual: channel.worlds,
            });
        }
        Self::build(channel).map(Some)
    }

    /// 组装 2D/3D flex 的设备段与场景叶；`dim == 1` 的 flex 只产生胶囊场景叶。
    pub fn build(channel: &FlexMeshChannel<'_>) -> Result<Self, FlexSceneError> {
        let nflex = channel.flex_dim.len();
        if channel.worlds == 0 {
            return Err(FlexSceneError::Count { field: "worlds" });
        }
        if channel.topology.nflex != nflex {
            return Err(FlexSceneError::Length {
                field: "flex_topology_nflex",
                expected: nflex,
                actual: channel.topology.nflex,
            });
        }
        // R06-S3c：再用 S2c 拓扑字段自校验挡住短向量/负值（不 panic），随后按 flex 索引读取。
        channel
            .topology
            .validate()
            .map_err(|error| FlexSceneError::Topology(error.to_string()))?;
        for (field, len) in [
            ("flex_vertadr", channel.flex_vertadr.len()),
            ("flex_radius", channel.flex_radius.len()),
        ] {
            if len != nflex {
                return Err(FlexSceneError::Length {
                    field,
                    expected: nflex,
                    actual: len,
                });
            }
        }
        if channel.geom_flexid.len() != channel.geom_edgeid.len() {
            return Err(FlexSceneError::Length {
                field: "flex_geom_edgeid",
                expected: channel.geom_flexid.len(),
                actual: channel.geom_edgeid.len(),
            });
        }
        let world_verts = channel
            .nflexvert
            .checked_mul(3)
            .ok_or(FlexSceneError::Count { field: "nflexvert" })?;
        let expected_positions =
            channel
                .worlds
                .checked_mul(world_verts)
                .ok_or(FlexSceneError::Count {
                    field: "flexvert_xpos",
                })?;
        if channel.flexvert_xpos.len() != expected_positions {
            return Err(FlexSceneError::Length {
                field: "flexvert_xpos",
                expected: expected_positions,
                actual: channel.flexvert_xpos.len(),
            });
        }
        let vertnums = derive_vertnums(channel)?;
        for flex in 0..nflex {
            if !(1..=3).contains(&channel.flex_dim[flex]) {
                return Err(FlexSceneError::UnsupportedDimension {
                    flex,
                    dim: channel.flex_dim[flex],
                });
            }
            let radius = channel.flex_radius[flex];
            if !radius.is_finite() {
                return Err(FlexSceneError::NonFinite {
                    field: "flex_radius",
                    index: flex,
                });
            }
            if channel.flex_dim[flex] >= 2 {
                check_topology_width(channel, flex)?;
            }
        }
        let leaves = scene_leaves(channel, &vertnums)?;
        let (device, bvh_ids) = device::mesh_device(channel, &vertnums)?;
        let mesh_count = device.entries.len();
        Ok(Self {
            leaves,
            device,
            bvh_ids,
            mesh_count,
        })
    }

    /// 把设备段追加到既有 `MeshDevice` 布局，并把 `bvh_ids` 平移到追加后的绝对条目号。
    pub fn append_to_mesh_device(
        &self,
        meta: &mut Vec<i32>,
        params: &mut Vec<f32>,
        offsets: &mut Vec<[i32; 2]>,
    ) -> Result<Vec<u64>, FlexSceneError> {
        let base = self.device.append_to(meta, params, offsets)?;
        self.bvh_ids
            .iter()
            .map(|id| {
                if *id == FLEX_NO_MESH {
                    Ok(FLEX_NO_MESH)
                } else {
                    base.checked_add(*id).ok_or(FlexSceneError::Count {
                        field: "flex_mesh_entries",
                    })
                }
            })
            .collect()
    }
}

/// `flex_vertadr` 链推导 `flex_vertnum`（S3c host 入参不含 vertnum；与 S3pre 同口径校验）。
fn derive_vertnums(channel: &FlexMeshChannel<'_>) -> Result<Vec<usize>, FlexSceneError> {
    let nflex = channel.flex_dim.len();
    let mut vertnums = Vec::with_capacity(nflex);
    let mut next = 0usize;
    for flex in 0..nflex {
        let start =
            usize::try_from(channel.flex_vertadr[flex]).map_err(|_| FlexSceneError::Count {
                field: "flex_vertadr",
            })?;
        if start != next {
            return Err(FlexSceneError::Count {
                field: "flex_vertadr",
            });
        }
        let end = channel
            .flex_vertadr
            .get(flex + 1)
            .map(|value| usize::try_from(*value))
            .transpose()
            .map_err(|_| FlexSceneError::Count {
                field: "flex_vertadr",
            })?
            .unwrap_or(channel.nflexvert);
        if end < start || end > channel.nflexvert {
            return Err(FlexSceneError::Count {
                field: "flex_vertadr",
            });
        }
        vertnums.push(end - start);
        next = end;
    }
    if next != channel.nflexvert {
        return Err(FlexSceneError::Count { field: "nflexvert" });
    }
    Ok(vertnums)
}

/// 2D/3D 的逐 flex 拓扑宽度校验（上游 `mjmodel.h`：元素 `dim+1`、壳 `dim`）。
fn check_topology_width(channel: &FlexMeshChannel<'_>, flex: usize) -> Result<(), FlexSceneError> {
    let dim = usize::try_from(channel.flex_dim[flex])
        .map_err(|_| FlexSceneError::Count { field: "flex_dim" })?;
    let topology = channel.topology;
    let elems =
        usize::try_from(topology.flex_elemnum[flex]).map_err(|_| FlexSceneError::Count {
            field: "flex_elemnum",
        })?;
    let elem_start =
        usize::try_from(topology.flex_elemdataadr[flex]).map_err(|_| FlexSceneError::Count {
            field: "flex_elemdataadr",
        })?;
    let elem_end = topology
        .flex_elemdataadr
        .get(flex + 1)
        .map(|value| usize::try_from(*value))
        .transpose()
        .map_err(|_| FlexSceneError::Count {
            field: "flex_elemdataadr",
        })?
        .unwrap_or(topology.nflexelemdata);
    if elem_end.saturating_sub(elem_start) != elems * (dim + 1) {
        return Err(FlexSceneError::TopologyWidth {
            flex,
            field: "flex_elemdataadr",
        });
    }
    let shells =
        usize::try_from(topology.flex_shellnum[flex]).map_err(|_| FlexSceneError::Count {
            field: "flex_shellnum",
        })?;
    let shell_start =
        usize::try_from(topology.flex_shelldataadr[flex]).map_err(|_| FlexSceneError::Count {
            field: "flex_shelldataadr",
        })?;
    let shell_end = topology
        .flex_shelldataadr
        .get(flex + 1)
        .map(|value| usize::try_from(*value))
        .transpose()
        .map_err(|_| FlexSceneError::Count {
            field: "flex_shelldataadr",
        })?
        .unwrap_or(topology.nflexshelldata);
    if shell_end.saturating_sub(shell_start) != shells * dim {
        return Err(FlexSceneError::TopologyWidth {
            flex,
            field: "flex_shelldataadr",
        });
    }
    Ok(())
}

/// 场景 flex 叶：world-major、`group = worldid`。
fn scene_leaves(
    channel: &FlexMeshChannel<'_>,
    vertnums: &[usize],
) -> Result<FlexSceneLeaves, FlexSceneError> {
    let mut bounds = Vec::with_capacity(channel.worlds * channel.geom_flexid.len());
    let mut groups = Vec::with_capacity(channel.worlds * channel.geom_flexid.len());
    for world in 0..channel.worlds {
        for (index, (&flexid, &edgeid)) in channel
            .geom_flexid
            .iter()
            .zip(channel.geom_edgeid)
            .enumerate()
        {
            let flex =
                usize::try_from(flexid).map_err(|_| FlexSceneError::InvalidMapping { index })?;
            let radius = *channel.flex_radius.get(flex).ok_or(FlexSceneError::Count {
                field: "flex_radius",
            })?;
            let vert_adr =
                usize::try_from(channel.flex_vertadr[flex]).map_err(|_| FlexSceneError::Count {
                    field: "flex_vertadr",
                })?;
            let (lower, upper) = if edgeid >= 0 {
                let edge = usize::try_from(edgeid).map_err(|_| FlexSceneError::Count {
                    field: "flex_geom_edgeid",
                })?;
                let slot = edge
                    .checked_mul(2)
                    .ok_or(FlexSceneError::Count { field: "flex_edge" })?;
                let local0 = *channel
                    .flex_edge
                    .get(slot)
                    .ok_or(FlexSceneError::Count { field: "flex_edge" })?;
                let local1 = *channel
                    .flex_edge
                    .get(slot + 1)
                    .ok_or(FlexSceneError::Count { field: "flex_edge" })?;
                let v0 = position(channel, world, vert_adr, local0)?;
                let v1 = position(channel, world, vert_adr, local1)?;
                capsule_bounds(v0, v1, radius)
            } else {
                let count = *vertnums.get(flex).ok_or(FlexSceneError::Count {
                    field: "flex_vertnum",
                })?;
                mesh_bounds(channel, world, vert_adr, count, radius)?
            };
            bounds.push(Aabb::new(lower, upper).map_err(|_| FlexSceneError::Count {
                field: "flex_leaf_bounds",
            })?);
            groups
                .push(u32::try_from(world).map_err(|_| FlexSceneError::Count { field: "worlds" })?);
        }
    }
    Ok(FlexSceneLeaves { bounds, groups })
}

fn capsule_bounds(v0: [f32; 3], v1: [f32; 3], radius: f32) -> ([f32; 3], [f32; 3]) {
    let mut lower = [0.0f32; 3];
    let mut upper = [0.0f32; 3];
    for axis in 0..3 {
        lower[axis] = v0[axis].min(v1[axis]) - radius;
        upper[axis] = v0[axis].max(v1[axis]) + radius;
    }
    (lower, upper)
}

/// 2D/3D：该 flex 全部顶点的整体包围盒 ± 半径（上游 L299-311）。
fn mesh_bounds(
    channel: &FlexMeshChannel<'_>,
    world: usize,
    vert_adr: usize,
    count: usize,
    radius: f32,
) -> Result<([f32; 3], [f32; 3]), FlexSceneError> {
    let mut lower = [f32::INFINITY; 3];
    let mut upper = [f32::NEG_INFINITY; 3];
    for local in 0..count {
        let value = position(
            channel,
            world,
            vert_adr,
            i32::try_from(local).map_err(|_| FlexSceneError::Count {
                field: "flex_vertnum",
            })?,
        )?;
        for axis in 0..3 {
            lower[axis] = lower[axis].min(value[axis]);
            upper[axis] = upper[axis].max(value[axis]);
        }
    }
    for axis in 0..3 {
        lower[axis] -= radius;
        upper[axis] += radius;
    }
    Ok((lower, upper))
}

/// 取一个 flex 顶点的世界位置（`base` 是该 world 在展平数组里的起点）。
fn position(
    channel: &FlexMeshChannel<'_>,
    world: usize,
    vert_adr: usize,
    local: i32,
) -> Result<[f32; 3], FlexSceneError> {
    let local = usize::try_from(local).map_err(|_| FlexSceneError::Count { field: "flex_edge" })?;
    let world_base = world
        .checked_mul(channel.nflexvert)
        .and_then(|value| value.checked_add(vert_adr))
        .and_then(|value| value.checked_add(local))
        .and_then(|value| value.checked_mul(3))
        .ok_or(FlexSceneError::Count {
            field: "flexvert_xpos",
        })?;
    let mut out = [0.0f32; 3];
    for (axis, slot) in out.iter_mut().enumerate() {
        let value = *channel
            .flexvert_xpos
            .get(world_base + axis)
            .ok_or(FlexSceneError::Count {
                field: "flexvert_xpos",
            })?;
        if !value.is_finite() {
            return Err(FlexSceneError::NonFinite {
                field: "flexvert_xpos",
                index: world_base + axis,
            });
        }
        *slot = value;
    }
    Ok(out)
}

mod device;

#[cfg(test)]
#[path = "flex_scene/tests.rs"]
mod tests;
