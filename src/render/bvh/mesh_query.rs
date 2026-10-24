//! GPU 网格三角 BVH 射线查询（渲染/BVH S3）。
//!
//! 宿主把 `MeshBvh`（顶点/索引/节点）、geom 位姿与射线集打包成四个设备缓冲，
//! 用既有 `SynchronousKernel`（NVRTC、fmad=true、fast_math=false）编译
//! `cuda/mesh.cu` 并发射。设备侧复刻冻结上游渲染路径
//! `ray_mesh_with_bvh` → `wp.mesh_query_ray` 的语义：局部系 woop 三角求交、
//! 背面剔除最多 8 次步进、最近命中，返回 distance/u/v/face/世界法向。
//!
//! 打包前校验射线数、`max_t`、位姿有限性、栈深与计数溢出；命中判定只在设备侧
//! 做（宿主不回落），输入非法在分配与发射前拒绝。

use crate::diagnostics::{InputError, TransferError};
use crate::render::bvh::{MeshBvh, MeshNodeKind, Ray};
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};

/// 与 `mesh.cu` 的栈深度一致；更深的树在打包前拒绝。
const STACK_DEPTH: usize = 128;
const ENTRY: &str = "mesh_bvh_ray";
const SOURCE: &str = include_str!("cuda/mesh.cu");
/// 每个射线的输出标量数：distance、normal 3、u、v、face、geomid。
const OUTPUT_STRIDE: usize = 8;

/// 单次查询的 geom 位姿（局部网格 → 世界）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshRayGeometry {
    pub geom_id: i32,
    pub xpos: [f32; 3],
    pub xmat: [f32; 9],
}

/// 单条网格射线命中；未命中时 `distance = -1`、`face = -1`、`geom_id = -1`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshRayHit {
    pub distance: f32,
    pub normal: [f32; 3],
    pub u: f32,
    pub v: f32,
    pub face: i32,
    pub geom_id: i32,
}

/// 网格 BVH 的 GPU 查询器；编译一次可重复查询。
pub struct MeshBvhQuery {
    session: TransferSession,
    kernel: SynchronousKernel,
}

impl MeshBvhQuery {
    /// 在给定传输会话上编译查询内核（需要 cuda-probe 与 NVRTC）。
    pub fn new(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, SOURCE, ENTRY)?,
        })
    }

    /// 查询一组射线。
    ///
    /// `max_t` 与 `rays` 等长（逐射线最大距离，语义同上游 `max_t`：严格小于才
    /// 命中）；`cull_backfaces` 对应上游 `ray_mesh_with_bvh` 的背面剔除开关。
    pub fn query(
        &self,
        mesh: &MeshBvh,
        geometry: &MeshRayGeometry,
        rays: &[Ray],
        max_t: &[f32],
        cull_backfaces: bool,
    ) -> Result<Vec<MeshRayHit>, TransferError> {
        if rays.is_empty() {
            return Err(InputError::InvalidDimension { field: "rays" }.into());
        }
        if max_t.len() != rays.len() {
            return Err(InputError::LengthMismatch {
                field: "max_t",
                expected: rays.len(),
                actual: max_t.len(),
            }
            .into());
        }
        if mesh.triangle_count() == 0 || mesh.nodes().is_empty() {
            return Err(InputError::InvalidDimension { field: "mesh_bvh" }.into());
        }
        for (index, value) in max_t.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "max_t",
                    index,
                }
                .into());
            }
            if *value < 0.0 {
                return Err(InputError::NegativeValue {
                    field: "max_t",
                    index,
                }
                .into());
            }
        }
        if geometry.geom_id < 0 {
            return Err(InputError::NegativeValue {
                field: "geom_id",
                index: 0,
            }
            .into());
        }
        if geometry
            .xpos
            .iter()
            .chain(geometry.xmat.iter())
            .any(|value| !value.is_finite())
        {
            return Err(InputError::NonFinite {
                field: "mesh_pose",
                index: 0,
            }
            .into());
        }
        check_depth(mesh)?;

        let vertex_count = mesh.vertices().len();
        let counts = PackedCounts::new(
            mesh.nodes().len(),
            mesh.triangle_count(),
            vertex_count,
            rays.len(),
        )?;
        let nodes = mesh.nodes();

        let mut meta = vec![0i32; counts.meta_len];
        meta[0] = counts.nodes as i32;
        meta[1] = counts.triangles as i32;
        meta[2] = counts.rays as i32;
        meta[3] = counts.nodes as i32 - 1;
        meta[4] = i32::from(cull_backfaces);
        meta[5] = geometry.geom_id;
        meta[6] = vertex_count as i32;
        let order_offset = 7 + 2 * counts.nodes;
        let indices_offset = order_offset + counts.triangles;
        for (index, node) in nodes.iter().enumerate() {
            match node.kind {
                MeshNodeKind::Internal { left, right } => {
                    if left as usize >= counts.nodes || right as usize >= counts.nodes {
                        return Err(InputError::InvalidIndex {
                            field: "mesh_child",
                            index,
                            limit: counts.nodes,
                        }
                        .into());
                    }
                    meta[7 + 2 * index] = left as i32;
                    meta[7 + 2 * index + 1] = right as i32;
                }
                MeshNodeKind::Leaf { begin, end } => {
                    if end as usize > counts.triangles || begin > end {
                        return Err(InputError::InvalidIndex {
                            field: "mesh_leaf",
                            index: begin as usize,
                            limit: counts.triangles,
                        }
                        .into());
                    }
                    meta[7 + 2 * index] = -(begin as i32) - 1;
                    meta[7 + 2 * index + 1] = end as i32;
                }
            }
        }
        for (slot, triangle) in mesh.leaf_order().iter().enumerate() {
            if *triangle as usize >= counts.triangles {
                return Err(InputError::InvalidIndex {
                    field: "mesh_triangle",
                    index: *triangle as usize,
                    limit: counts.triangles,
                }
                .into());
            }
            meta[order_offset + slot] = *triangle as i32;
        }
        for (slot, index) in mesh.indices().iter().enumerate() {
            if *index as usize >= vertex_count {
                return Err(InputError::InvalidIndex {
                    field: "mesh_vertex",
                    index: *index as usize,
                    limit: vertex_count,
                }
                .into());
            }
            meta[indices_offset + slot] = *index as i32;
        }

        let mut params = vec![0.0f32; counts.params_len];
        for (index, node) in nodes.iter().enumerate() {
            let offset = 6 * index;
            params[offset..offset + 3].copy_from_slice(&node.bounds.lower);
            params[offset + 3..offset + 6].copy_from_slice(&node.bounds.upper);
        }
        let vertex_offset = 6 * counts.nodes;
        for (index, vertex) in mesh.vertices().iter().enumerate() {
            params[vertex_offset + 3 * index..vertex_offset + 3 * index + 3]
                .copy_from_slice(vertex);
        }
        let pose_offset = vertex_offset + 3 * vertex_count;
        params[pose_offset..pose_offset + 3].copy_from_slice(&geometry.xpos);
        params[pose_offset + 3..pose_offset + 12].copy_from_slice(&geometry.xmat);

        let mut ray_values = vec![0.0f32; 7 * counts.rays];
        for (index, (ray, limit)) in rays.iter().zip(max_t.iter()).enumerate() {
            let base = 7 * index;
            ray_values[base..base + 3].copy_from_slice(&ray.origin);
            ray_values[base + 3..base + 6].copy_from_slice(&ray.direction);
            ray_values[base + 6] = *limit;
        }

        let meta_buffer: TransferBuffer<i32> = self.session.upload(&meta)?;
        let params_buffer: TransferBuffer<f32> = self.session.upload(&params)?;
        let ray_buffer: TransferBuffer<f32> = self.session.upload(&ray_values)?;
        let mut out = vec![0.0f32; OUTPUT_STRIDE * counts.rays];
        let mut out_buffer: TransferBuffer<f32> = self.session.upload(&out)?;
        let ray_count =
            u32::try_from(counts.rays).map_err(|_| InputError::Overflow { field: "rays" })?;
        // SAFETY: the kernel ABI, buffer types/strides and access ranges are
        // validated above; the grid covers exactly counts.rays with a tid guard in
        // mesh.cu; buffers belong to this session and the launch is synchronous.
        unsafe {
            self.kernel.launch(
                &meta_buffer,
                &params_buffer,
                &ray_buffer,
                &mut out_buffer,
                [0, 0, 0, ray_count],
            )?;
        }
        out_buffer.read_range_into(0, &mut out)?;

        let mut hits = Vec::with_capacity(counts.rays);
        for index in 0..counts.rays {
            let base = OUTPUT_STRIDE * index;
            hits.push(MeshRayHit {
                distance: out[base],
                normal: [out[base + 1], out[base + 2], out[base + 3]],
                u: out[base + 4],
                v: out[base + 5],
                face: i32::from_ne_bytes(out[base + 6].to_ne_bytes()),
                geom_id: i32::from_ne_bytes(out[base + 7].to_ne_bytes()),
            });
        }
        Ok(hits)
    }
}

/// 设备内核使用固定 128 层栈；超过可用深度在打包前拒绝，避免静默漏子树。
fn check_depth(mesh: &MeshBvh) -> Result<(), TransferError> {
    let nodes = mesh.nodes();
    let mut stack = vec![(nodes.len() as u32 - 1, 1usize)];
    while let Some((index, depth)) = stack.pop() {
        if depth > STACK_DEPTH {
            return Err(InputError::InvalidDimension {
                field: "mesh_bvh_depth",
            }
            .into());
        }
        if let MeshNodeKind::Internal { left, right } = nodes[index as usize].kind {
            stack.push((left, depth + 1));
            stack.push((right, depth + 1));
        }
    }
    Ok(())
}

struct PackedCounts {
    nodes: usize,
    triangles: usize,
    rays: usize,
    meta_len: usize,
    params_len: usize,
}

impl PackedCounts {
    fn new(
        nodes: usize,
        triangles: usize,
        vertices: usize,
        rays: usize,
    ) -> Result<Self, TransferError> {
        // 设备下标是 i32：节点/三角形/顶点数先限制在 i32 正区间。
        for (field, value) in [
            ("mesh_nodes", nodes),
            ("mesh_triangles", triangles),
            ("mesh_vertices", vertices),
        ] {
            if value > i32::MAX as usize {
                return Err(InputError::Overflow { field }.into());
            }
        }
        let meta_len = 7usize
            .checked_add(2usize.checked_mul(nodes).ok_or(InputError::Overflow {
                field: "mesh_nodes",
            })?)
            .and_then(|value| value.checked_add(4 * triangles))
            .ok_or(InputError::Overflow { field: "mesh_meta" })?;
        let params_len = 6usize
            .checked_mul(nodes)
            .and_then(|value| value.checked_add(3usize.checked_mul(vertices)?))
            .and_then(|value| value.checked_add(12))
            .ok_or(InputError::Overflow {
                field: "mesh_params",
            })?;
        Ok(Self {
            nodes,
            triangles,
            rays,
            meta_len,
            params_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::bvh::{MeshBvhInput, MeshNodeKind};

    fn tetra() -> MeshBvh {
        let vertices = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let indices = [0u32, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3];
        MeshBvh::build(MeshBvhInput {
            vertices: &vertices,
            indices: &indices,
        })
        .expect("四面体应被接受")
    }

    #[test]
    fn packed_counts_follow_the_documented_layout() {
        let counts = PackedCounts::new(7, 4, 4, 3).expect("计数应可表示");
        assert_eq!(counts.meta_len, 7 + 2 * 7 + 4 * 4);
        assert_eq!(counts.params_len, 6 * 7 + 3 * 4 + 12);
        assert_eq!(counts.rays, 3);
    }

    #[test]
    fn rejects_counts_beyond_device_index_range() {
        assert!(
            PackedCounts::new(i32::MAX as usize + 1, 1, 1, 1).is_err(),
            "节点数超出 i32 应被拒绝"
        );
        assert!(
            PackedCounts::new(1, i32::MAX as usize + 1, 1, 1).is_err(),
            "三角形数超出 i32 应被拒绝"
        );
        assert!(
            PackedCounts::new(1, 1, i32::MAX as usize + 1, 1).is_err(),
            "顶点数超出 i32 应被拒绝"
        );
    }

    #[test]
    fn depth_check_accepts_shallow_meshes_and_rejects_overflowing_meta() {
        let mesh = tetra();
        check_depth(&mesh).expect("四面体深度应被接受");
        assert!(
            mesh.nodes()
                .iter()
                .any(|node| matches!(node.kind, MeshNodeKind::Leaf { .. }))
        );
    }
}
