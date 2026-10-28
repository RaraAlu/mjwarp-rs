//! S7b1 网格三角 BVH 的 device 打包（R05 拆分②：纯移动自 `cast.rs`）。
//!
//! 本文件由 `cast.rs` 原 L284-372「MeshDevice（含类型文档）」段原样迁出，仅做归属调整：
//! 逻辑、打包顺序、类型、错误路径与布局文档**零变化**；`to_i32` 仍在 `cast.rs`
//! （`SceneDevice`/`CameraDevice` 共用），本文件经 `use super::to_i32;` 复用。
//!
//! 对外路径不变：`cast.rs` 保留 `pub use cast_mesh::MeshDevice;`。
//!
//! R05-S2 追加：`HfieldDevice`（hfield 三角化 + `hfield_bvh_id` 槽值）也落在此文件
//! （`cast_mesh.rs` 已登记 `mjwarp.render` 模块源；见 S2 证据）。

use std::collections::BTreeSet;

use crate::render::bvh::{HfieldMesh, HfieldMeshInput, MeshBvh, MeshNodeKind, build_hfield_mesh};
use crate::render::context::params::hfield::HfieldContextInput;
use crate::render::context::{RenderContext, ResolvedContext};
use crate::render::kernel::cast::ModelRenderArrays;
use crate::runtime::launcher::LauncherError;

use super::to_i32;

/// 网格三角 BVH 的 device 打包（`render.cu::ray_mesh_leaf` 消费）。
///
/// 逐网格段（`mesh_offsets[i] = (meta 基址, params 基址)`）：
///
/// ```text
/// meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
/// params = [node bounds 6*n_nodes, vertices 3*n_vert]
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDevice {
    pub meta: Vec<i32>,
    pub params: Vec<f32>,
    /// 每网格 `(meta 基址, params 基址)`，按 `Vec2I` 元素上传。
    pub offsets: Vec<[i32; 2]>,
}

impl MeshDevice {
    /// 按 context 拥有的网格 BVH 顺序打包（含 `set_points` + 显式 `refit` 后的节点）。
    pub fn pack(meshes: &[&MeshBvh]) -> Result<Self, LauncherError> {
        let mut meta = Vec::new();
        let mut params = Vec::new();
        let mut offsets = Vec::with_capacity(meshes.len());
        for mesh in meshes {
            let nodes = mesh.nodes();
            let n_nodes = nodes.len();
            let triangles = mesh.triangle_count();
            let vertices = mesh.vertices();
            if n_nodes == 0 || triangles == 0 || vertices.is_empty() {
                return Err(LauncherError::InvalidArray("mesh_bvh"));
            }
            offsets.push([
                to_i32(meta.len(), "mesh_meta_offset")?,
                to_i32(params.len(), "mesh_params_offset")?,
            ]);
            meta.push(to_i32(n_nodes, "mesh_nodes")?);
            meta.push(to_i32(triangles, "mesh_triangles")?);
            meta.push(to_i32(vertices.len(), "mesh_vertices")?);
            meta.push(to_i32(n_nodes, "mesh_root")? - 1);
            for node in nodes {
                match node.kind {
                    MeshNodeKind::Internal { left, right } => {
                        meta.push(to_i32(left as usize, "mesh_child")?);
                        meta.push(to_i32(right as usize, "mesh_child")?);
                    }
                    MeshNodeKind::Leaf { begin, end } => {
                        meta.push(-to_i32(begin as usize, "mesh_leaf")? - 1);
                        meta.push(to_i32(end as usize, "mesh_leaf")?);
                    }
                }
            }
            for triangle in mesh.leaf_order() {
                meta.push(to_i32(*triangle as usize, "mesh_order")?);
            }
            for index in mesh.indices() {
                meta.push(to_i32(*index as usize, "mesh_index")?);
            }
            for node in nodes {
                params.extend_from_slice(&node.bounds.lower);
                params.extend_from_slice(&node.bounds.upper);
            }
            for vertex in vertices {
                params.extend_from_slice(vertex);
            }
        }
        Ok(Self {
            meta,
            params,
            offsets,
        })
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// 展平的 `(meta, params)` 偏移（`Vec2I` 数组内容）。
    pub fn flat_offsets(&self) -> Vec<i32> {
        let mut flat = Vec::with_capacity(2 * self.offsets.len());
        for [meta, params] in &self.offsets {
            flat.push(*meta);
            flat.push(*params);
        }
        flat
    }
}

/// R05-S2 hfield 设备胶水：把注入的 hfield 静态数据三角化成 mesh BVH，并给出
/// `hfield_bvh_id` 槽值（**blob 内 mesh 索引**，未用项 `u64::MAX` 哨兵）。
///
/// 语义（与上游 `render_util.py` L473-488 对照）：
/// - 只对**被启用 hfield geom 引用**的 hfield 建 mesh（上游 `used_hfield_id` 同义）；
/// - hfield mesh **追加在既有 mesh blob 末尾**（`mesh_count + order`），既有 mesh 索引不变；
/// - `bvh_ids.len() == nhfield`；未被引用的 hfield 槽位填 `u64::MAX`（设备侧跳过）；
/// - 无 hfield geom 时返回 `Ok(None)`（`hfield_bvh_id` 上传 1 个哨兵占位）。
pub struct HfieldDevice {
    meshes: Vec<HfieldMesh>,
    bvh_ids: Vec<u64>,
}

impl HfieldDevice {
    /// 构建 hfield mesh BVH 与槽值表。
    ///
    /// 错误面：hfield 三角化/网格 BVH 失败 → [`LauncherError::Driver`]；
    /// `geom_dataid` 越界或 mesh 索引溢出 → [`LauncherError::InvalidArray`]/[`LauncherError::Overflow`]；
    /// 模型含 hfield geom 但未注入 hfield 数据 → [`LauncherError::Unsupported`]。
    pub fn build(
        input: Option<&HfieldContextInput>,
        model: &ModelRenderArrays,
        mesh_count: usize,
    ) -> Result<Option<Self>, LauncherError> {
        let mut used: BTreeSet<usize> = BTreeSet::new();
        for (geom, kind) in model.geom_type.iter().enumerate() {
            if *kind != 1 {
                continue;
            }
            let dataid = *model
                .geom_dataid
                .get(geom)
                .ok_or(LauncherError::InvalidArray("hfield_dataid"))?;
            if dataid < 0 {
                continue;
            }
            used.insert(
                usize::try_from(dataid)
                    .map_err(|_| LauncherError::InvalidArray("hfield_dataid"))?,
            );
        }
        if used.is_empty() {
            return Ok(None);
        }
        let input = input.ok_or(LauncherError::Unsupported(
            "S7b3 的 hfield 求交需要注入 hfield 数据（ResolvedContext::set_hfield）",
        ))?;
        let nhfield = input.len();
        let mut bvh_ids = vec![u64::MAX; nhfield];
        let mut meshes = Vec::with_capacity(used.len());
        for (order, hid) in used.iter().enumerate() {
            let hfield = input
                .hfields()
                .get(*hid)
                .ok_or(LauncherError::InvalidArray("hfield_dataid"))?;
            let size = hfield.size();
            let mesh = build_hfield_mesh(&HfieldMeshInput {
                nrow: hfield.nrow(),
                ncol: hfield.ncol(),
                size: [size[0], size[1], size[2]],
                data: hfield.data(),
            })
            .map_err(|error| LauncherError::Driver(format!("hfield-mesh: {error}")))?;
            let index = mesh_count
                .checked_add(order)
                .ok_or(LauncherError::Overflow("hfield_mesh_index"))?;
            let index = to_i32(index, "hfield_mesh_index")?;
            bvh_ids[*hid] = index as u64;
            meshes.push(mesh);
        }
        Ok(Some(Self { meshes, bvh_ids }))
    }

    /// 追加到 mesh blob 末尾的 hfield mesh BVH（顺序 = 被引用 hfield id 升序）。
    pub fn meshes(&self) -> &[HfieldMesh] {
        &self.meshes
    }

    /// `hfield_bvh_id` 槽值：长度 `nhfield`，值为 blob 内 mesh 索引，未用项 `u64::MAX`。
    pub fn bvh_ids(&self) -> &[u64] {
        &self.bvh_ids
    }
}

/// 每帧 mesh 设备打包：既有 mesh BVH（context 拥有）+ 追加 hfield mesh（R05-S2）。
///
/// 既有 mesh 索引不变；hfield mesh 依次追加在末尾（blob 内索引 = `mesh_count + order`）。
/// `hfield` 为 `None` 表示模型无 hfield geom（`hfield_bvh_id` 槽落 1 个哨兵占位）。
///
/// 放在 `cast_mesh.rs` 而非 `megakernel.rs`：`megakernel.rs` 行数口径按 S2 rebase
/// 尽量零改动（见 S2 证据 README）。
pub fn pack_mesh_devices(
    context: &RenderContext,
    resolved: &ResolvedContext,
    model: &ModelRenderArrays,
) -> Result<(MeshDevice, Option<HfieldDevice>), LauncherError> {
    let hfield = HfieldDevice::build(resolved.hfield(), model, context.mesh_count())?;
    let mut mesh_bvhs = Vec::with_capacity(
        context.mesh_count() + hfield.as_ref().map_or(0, |device| device.meshes().len()),
    );
    for index in 0..context.mesh_count() {
        mesh_bvhs.push(
            context
                .mesh_bvh(index)
                .ok_or(LauncherError::InvalidArray("mesh_index"))?,
        );
    }
    if let Some(device) = &hfield {
        mesh_bvhs.extend(device.meshes().iter().map(|mesh| mesh.bvh()));
    }
    Ok((MeshDevice::pack(&mesh_bvhs)?, hfield))
}
