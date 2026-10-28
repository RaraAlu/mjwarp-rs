//! S7b1 网格三角 BVH 的 device 打包（R05 拆分②：纯移动自 `cast.rs`）。
//!
//! 本文件由 `cast.rs` 原 L284-372「MeshDevice（含类型文档）」段原样迁出，仅做归属调整：
//! 逻辑、打包顺序、类型、错误路径与布局文档**零变化**；`to_i32` 仍在 `cast.rs`
//! （`SceneDevice`/`CameraDevice` 共用），本文件经 `use super::to_i32;` 复用。
//!
//! 对外路径不变：`cast.rs` 保留 `pub use cast_mesh::MeshDevice;`。

use crate::render::bvh::{MeshBvh, MeshNodeKind};
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
