//! GPU 场景 BVH 射线查询（渲染/BVH S2b）。
//!
//! 宿主把 S2a 构建/重拟合好的 `SceneBvh`、每个 leaf 的 geom 类型与位姿、射线集
//! 打包成四个设备缓冲，用既有 `SynchronousKernel`（NVRTC、fmad=true、
//! fast_math=false）编译 `scene.cu` 并发射；设备侧是真 `fma.rn.f32`，不使用宿主
//! f64 复刻。一次调用查询一个 world 分组，命中语义为「最近的 primitive 命中」，
//! 输出 geomid / distance / normal；mesh 命中留 S3（内核按 miss 处理）。

use crate::diagnostics::{InputError, TransferError};
use crate::render::bvh::{NodeKind, SceneBvh};
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};

/// 与 `scene.cu` 的栈深度一致；越深的分组在打包前拒绝。
const STACK_DEPTH: usize = 128;
const ENTRY: &str = "scene_bvh_ray";
const SOURCE: &str = include_str!("scene.cu");

/// 世界坐标射线；方向不要求归一化（distance 是方向参数）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

/// 单个 leaf 的求交输入；mesh/SDF 的 leaf 命中按 S3 延后（内核返回 miss）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafRayGeometry {
    pub geom_type: i32,
    pub size: [f32; 3],
    pub xpos: [f32; 3],
    pub xmat: [f32; 9],
}

/// 单条射线的查询结果；未命中时 `distance = -1`、`geom_id = -1`、法向为零。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub distance: f32,
    pub geom_id: i32,
    pub normal: [f32; 3],
}

/// 场景 BVH 的 GPU 查询器；编译一次可重复查询。
pub struct SceneBvhQuery {
    session: TransferSession,
    kernel: SynchronousKernel,
}

impl SceneBvhQuery {
    /// 在给定传输会话上编译查询内核（需要 cuda-probe 与 NVRTC）。
    pub fn new(session: &TransferSession) -> Result<Self, TransferError> {
        Ok(Self {
            session: session.clone(),
            kernel: SynchronousKernel::compile(session, SOURCE, ENTRY)?,
        })
    }

    /// 查询 `world` 分组。
    ///
    /// `geometry` 按 leaf 顺序给出每个 leaf 的 geom 信息（长度必须等于
    /// `bvh.leaf_count()`）；`rays` 非空。返回顺序与 `rays` 一致。
    pub fn query(
        &self,
        world: usize,
        bvh: &SceneBvh,
        geometry: &[LeafRayGeometry],
        rays: &[Ray],
    ) -> Result<Vec<RayHit>, TransferError> {
        if rays.is_empty() {
            return Err(InputError::InvalidDimension { field: "rays" }.into());
        }
        if geometry.len() != bvh.leaf_count() {
            return Err(InputError::LengthMismatch {
                field: "leaf_geometry",
                expected: bvh.leaf_count(),
                actual: geometry.len(),
            }
            .into());
        }
        if world >= bvh.worlds() {
            return Err(InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: bvh.worlds(),
            }
            .into());
        }
        let nodes = bvh.nodes();
        if nodes.is_empty() || bvh.leaf_count() == 0 {
            return Err(InputError::InvalidDimension { field: "scene_bvh" }.into());
        }
        check_depth(nodes, bvh)?;
        for leaf_geometry in geometry.iter() {
            match leaf_geometry.geom_type {
                0 | 2 | 3 | 4 | 5 | 6 | 7 | 8 => {}
                1 => {
                    return Err(InputError::InvalidDimension {
                        field: "geom_type_hfield",
                    }
                    .into());
                }
                other => {
                    return Err(InputError::InvalidIndex {
                        field: "geom_type",
                        index: other as usize,
                        limit: 9,
                    }
                    .into());
                }
            }
        }

        let counts = PackedCounts::new(nodes.len(), bvh.leaf_count(), rays.len())?;
        let root = bvh
            .group_root(world)
            .map_err(|_| InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: bvh.worlds(),
            })?;

        let mut meta = vec![0i32; counts.meta_len];
        meta[0] = counts.nodes as i32;
        meta[1] = counts.leaves as i32;
        meta[2] = counts.rays as i32;
        meta[3] = root as i32;
        let leaf_geom_offset = 4 + 2 * counts.nodes;
        for (index, node) in nodes.iter().enumerate() {
            match node.kind {
                NodeKind::Internal { left, right } => {
                    if left as usize >= counts.nodes || right as usize >= counts.nodes {
                        return Err(InputError::InvalidIndex {
                            field: "bvh_child",
                            index,
                            limit: counts.nodes,
                        }
                        .into());
                    }
                    meta[4 + 2 * index] = left as i32;
                    meta[4 + 2 * index + 1] = right as i32;
                }
                NodeKind::Leaf { leaf } => {
                    if leaf as usize >= counts.leaves {
                        return Err(InputError::InvalidIndex {
                            field: "bvh_leaf",
                            index: leaf as usize,
                            limit: counts.leaves,
                        }
                        .into());
                    }
                    meta[4 + 2 * index] = -(leaf as i32) - 1;
                    meta[4 + 2 * index + 1] = 0;
                }
            }
        }
        for leaf in 0..counts.leaves {
            meta[leaf_geom_offset + leaf] =
                bvh.leaf_geom_id(leaf as u32)
                    .ok_or(InputError::InvalidIndex {
                        field: "leaf_geom_id",
                        index: leaf,
                        limit: counts.leaves,
                    })?;
        }
        for (leaf, leaf_geometry) in geometry.iter().enumerate() {
            meta[leaf_geom_offset + counts.leaves + leaf] = leaf_geometry.geom_type;
        }

        let mut params = vec![0.0f32; counts.params_len];
        for (index, node) in nodes.iter().enumerate() {
            let offset = 6 * index;
            params[offset..offset + 3].copy_from_slice(&node.bounds.lower);
            params[offset + 3..offset + 6].copy_from_slice(&node.bounds.upper);
        }
        let leaf_size_offset = 6 * counts.nodes;
        let leaf_xpos_offset = leaf_size_offset + 3 * counts.leaves;
        let leaf_xmat_offset = leaf_xpos_offset + 3 * counts.leaves;
        for (leaf, leaf_geometry) in geometry.iter().enumerate() {
            params[leaf_size_offset + 3 * leaf..leaf_size_offset + 3 * leaf + 3]
                .copy_from_slice(&leaf_geometry.size);
            params[leaf_xpos_offset + 3 * leaf..leaf_xpos_offset + 3 * leaf + 3]
                .copy_from_slice(&leaf_geometry.xpos);
            params[leaf_xmat_offset + 9 * leaf..leaf_xmat_offset + 9 * leaf + 9]
                .copy_from_slice(&leaf_geometry.xmat);
        }

        let mut ray_values = vec![0.0f32; 6 * counts.rays];
        for (index, ray) in rays.iter().enumerate() {
            ray_values[6 * index..6 * index + 3].copy_from_slice(&ray.origin);
            ray_values[6 * index + 3..6 * index + 6].copy_from_slice(&ray.direction);
        }

        let meta_buffer: TransferBuffer<i32> = self.session.upload(&meta)?;
        let params_buffer: TransferBuffer<f32> = self.session.upload(&params)?;
        let ray_buffer: TransferBuffer<f32> = self.session.upload(&ray_values)?;
        let mut out = vec![0.0f32; 5 * counts.rays];
        let mut out_buffer: TransferBuffer<f32> = self.session.upload(&out)?;
        let ray_count =
            u32::try_from(counts.rays).map_err(|_| InputError::Overflow { field: "rays" })?;
        // SAFETY: the kernel ABI, buffer types/strides and access ranges are
        // validated above; grid covers exactly counts.rays with a tid guard in
        // scene.cu; buffers belong to this session and the launch is synchronous.
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
            let base = 5 * index;
            hits.push(RayHit {
                distance: out[base],
                geom_id: i32::from_ne_bytes(out[base + 1].to_ne_bytes()),
                normal: [out[base + 2], out[base + 3], out[base + 4]],
            });
        }
        Ok(hits)
    }
}

struct PackedCounts {
    nodes: usize,
    leaves: usize,
    rays: usize,
    meta_len: usize,
    params_len: usize,
}

impl PackedCounts {
    fn new(nodes: usize, leaves: usize, rays: usize) -> Result<Self, TransferError> {
        let meta_len = 4usize
            .checked_add(
                2usize
                    .checked_mul(nodes)
                    .ok_or(InputError::Overflow { field: "bvh_nodes" })?,
            )
            .and_then(|value| value.checked_add(2 * leaves))
            .ok_or(InputError::Overflow { field: "bvh_meta" })?;
        let params_len = 6usize
            .checked_mul(nodes)
            .and_then(|value| value.checked_add(15 * leaves))
            .ok_or(InputError::Overflow {
                field: "bvh_params",
            })?;
        let _ = rays;
        Ok(Self {
            nodes,
            leaves,
            rays,
            meta_len,
            params_len,
        })
    }
}

/// 内核使用固定 128 层栈；超过可用深度在打包前拒绝，避免静默漏子树。
fn check_depth(nodes: &[crate::render::bvh::BvhNode], bvh: &SceneBvh) -> Result<(), TransferError> {
    for world in 0..bvh.worlds() {
        let root = bvh
            .group_root(world)
            .map_err(|_| InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: bvh.worlds(),
            })?;
        let mut stack = vec![(root, 1usize)];
        while let Some((index, depth)) = stack.pop() {
            if depth + 1 > STACK_DEPTH {
                return Err(InputError::InvalidDimension { field: "bvh_depth" }.into());
            }
            if let NodeKind::Internal { left, right } = nodes[index as usize].kind {
                stack.push((left, depth + 1));
                stack.push((right, depth + 1));
            }
        }
    }
    Ok(())
}
