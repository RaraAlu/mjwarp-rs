//! R06-S3c：flex mesh 设备段（与 `MeshDevice` 同布局的 meta/params + 逐条目偏移）。
//!
//! 布局（消费方 `render_common.cu::ray_mesh_leaf`，与 `cast_mesh.rs::MeshDevice` 逐字段一致）：
//!   meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//!   params = [node bounds 6*n_nodes, vertices 3*n_vert]
//!
//! 条目索引 = `bvh_ids[flexid] + worldid`（flex 主序、world 次序）；1D/无 mesh 的 flex 记
//! [`FLEX_NO_MESH`] 哨兵，设备侧据此跳过 mesh 查询（走 S3a 胶囊路径）。

use super::super::flex_mesh::{FlexMeshBvh, FlexMeshInput};
use super::super::{MeshBvh, MeshNodeKind};
use super::{FlexMeshChannel, FlexSceneError};

/// 设备 flex mesh 段：与 `MeshDevice` 同布局的 meta/params 与逐条目 (meta, params) 偏移。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlexMeshDevice {
    pub meta: Vec<i32>,
    pub params: Vec<f32>,
    /// 每 `(flex, world)` 条目的 `(meta_base, params_base)`（flex 主序、world 次序）。
    pub entries: Vec<[i32; 2]>,
}

/// 哨兵：该 flex 没有设备 mesh 条目（1D 走边胶囊）。
pub const FLEX_NO_MESH: u64 = u64::MAX;

impl FlexMeshDevice {
    /// 把本段追加到既有 `MeshDevice` 同布局的三条缓冲，返回本段首条目的绝对条目号。
    ///
    /// 既有条目的 `(meta, params)` 偏移不变；本段条目偏移加上追加后的基址。
    pub fn append_to(
        &self,
        meta: &mut Vec<i32>,
        params: &mut Vec<f32>,
        offsets: &mut Vec<[i32; 2]>,
    ) -> Result<u64, FlexSceneError> {
        let base = u64::try_from(offsets.len()).map_err(|_| FlexSceneError::Count {
            field: "flex_mesh_entries",
        })?;
        let meta_base = i32::try_from(meta.len()).map_err(|_| FlexSceneError::Count {
            field: "flex_mesh_meta",
        })?;
        let params_base = i32::try_from(params.len()).map_err(|_| FlexSceneError::Count {
            field: "flex_mesh_params",
        })?;
        meta.extend_from_slice(&self.meta);
        params.extend_from_slice(&self.params);
        for entry in &self.entries {
            let meta_slot = entry[0]
                .checked_add(meta_base)
                .ok_or(FlexSceneError::Count {
                    field: "flex_mesh_meta",
                })?;
            let params_slot = entry[1]
                .checked_add(params_base)
                .ok_or(FlexSceneError::Count {
                    field: "flex_mesh_params",
                })?;
            offsets.push([meta_slot, params_slot]);
        }
        Ok(base)
    }
}

/// 构建每个 `(flex, world)` 的 MeshBvh 并打包成 `MeshDevice` 同布局的 device 段。
pub(super) fn mesh_device<'a>(
    channel: &'a FlexMeshChannel<'a>,
    vertnums: &[usize],
) -> Result<(FlexMeshDevice, Vec<u64>), FlexSceneError> {
    let nflex = channel.flex_dim.len();
    let mut device = FlexMeshDevice::default();
    let mut bvh_ids = vec![FLEX_NO_MESH; nflex];
    let mut world_vertices = Vec::with_capacity(channel.worlds);
    for world in 0..channel.worlds {
        world_vertices.push(world_positions(channel, world)?);
    }
    // flex 主序、world 次序：设备条目索引 = `bvh_ids[flexid] + worldid`。
    for flex in 0..nflex {
        if channel.flex_dim[flex] < 2 {
            continue;
        }
        let input = mesh_input(channel, flex, vertnums[flex])?;
        bvh_ids[flex] = u64::try_from(device.entries.len())
            .map_err(|_| FlexSceneError::Count { field: "flex_mesh" })?;
        for vertices in &world_vertices {
            let mesh = FlexMeshBvh::build(&input, vertices).map_err(FlexSceneError::Mesh)?;
            pack_mesh(&mut device, mesh.bvh());
        }
    }
    Ok((device, bvh_ids))
}

/// 单个 world 的 flex 顶点表（`[[f32; 3]]`，全局顶点索引口径）。
fn world_positions(
    channel: &FlexMeshChannel<'_>,
    world: usize,
) -> Result<Vec<[f32; 3]>, FlexSceneError> {
    let base = world
        .checked_mul(channel.nflexvert)
        .and_then(|value| value.checked_mul(3))
        .ok_or(FlexSceneError::Count {
            field: "flexvert_xpos",
        })?;
    let mut positions = Vec::with_capacity(channel.nflexvert);
    for vertex in 0..channel.nflexvert {
        let offset = base + 3 * vertex;
        let mut value = [0.0f32; 3];
        for (axis, slot) in value.iter_mut().enumerate() {
            let component =
                *channel
                    .flexvert_xpos
                    .get(offset + axis)
                    .ok_or(FlexSceneError::Count {
                        field: "flexvert_xpos",
                    })?;
            if !component.is_finite() {
                return Err(FlexSceneError::NonFinite {
                    field: "flexvert_xpos",
                    index: offset + axis,
                });
            }
            *slot = component;
        }
        positions.push(value);
    }
    Ok(positions)
}

/// 组装 `FlexMeshInput`（S3a 机制的显式拓扑入参；偏移量来自 S2c topology）。
fn mesh_input<'a>(
    channel: &'a FlexMeshChannel<'a>,
    flex: usize,
    vert_num: usize,
) -> Result<FlexMeshInput<'a>, FlexSceneError> {
    let topology = channel.topology;
    Ok(FlexMeshInput {
        dim: channel.flex_dim[flex],
        radius: channel.flex_radius[flex],
        vert_adr: channel.flex_vertadr[flex],
        vert_num: i32::try_from(vert_num).map_err(|_| FlexSceneError::Count {
            field: "flex_vertnum",
        })?,
        elem_data_adr: topology.flex_elemdataadr[flex],
        elem_num: topology.flex_elemnum[flex],
        elem: &topology.flex_elem,
        shell_data_adr: topology.flex_shelldataadr[flex],
        shell_num: topology.flex_shellnum[flex],
        shell: &topology.flex_shell,
    })
}

/// 把一个 `MeshBvh` 追加到 device 段（布局与 `MeshDevice::pack` 逐字段一致）。
fn pack_mesh(device: &mut FlexMeshDevice, mesh: &MeshBvh) {
    let nodes = mesh.nodes();
    let meta_base = device.meta.len() as i32;
    let params_base = device.params.len() as i32;
    device.meta.push(nodes.len() as i32);
    device.meta.push(mesh.triangle_count() as i32);
    device.meta.push(mesh.vertices().len() as i32);
    device.meta.push((nodes.len() - 1) as i32);
    for node in nodes {
        match node.kind {
            MeshNodeKind::Internal { left, right } => {
                device.meta.push(left as i32);
                device.meta.push(right as i32);
            }
            MeshNodeKind::Leaf { begin, end } => {
                device.meta.push(-(begin as i32) - 1);
                device.meta.push(end as i32);
            }
        }
    }
    for triangle in mesh.leaf_order() {
        device.meta.push(*triangle as i32);
    }
    for index in mesh.indices() {
        device.meta.push(*index as i32);
    }
    for node in nodes {
        device.params.extend_from_slice(&node.bounds.lower);
        device.params.extend_from_slice(&node.bounds.upper);
    }
    for vertex in mesh.vertices() {
        device.params.extend_from_slice(vertex);
    }
    device.entries.push([meta_base, params_base]);
}
