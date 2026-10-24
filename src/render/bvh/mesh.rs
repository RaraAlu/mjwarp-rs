//! 网格三角形 BVH 的宿主构建、重拟合与顶点更新（渲染/BVH S3）。
//!
//! 输入是 S1 已验收的 `mesh_vert`/`mesh_face`（局部系），构建分箱 SAH 三角 BVH；
//! `refit`/`set_points` 支持动态顶点更新（上游 `mesh.points` 赋值后 `mesh.refit()`）。
//! 叶子容量 8、节点先子后父，重拟合按索引正序一遍完成；验收只做语义命中断言，
//! 设备遍历与 `wp.mesh_query_ray` 的等价翻译在 `cuda/mesh.cu`。

use super::Aabb;

/// 分箱 SAH 的箱数（只影响构建质量，不参与验收）。
const SAH_BINS: usize = 12;
/// 每个叶子的三角形上限；越小树越深，越大叶子线性扫描越慢。
const LEAF_CAPACITY: usize = 8;

/// 三角 BVH 节点类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshNodeKind {
    /// 内部节点：左右子节点索引。
    Internal { left: u32, right: u32 },
    /// 叶子：`order[begin..end]` 是该叶子包含的三角形编号。
    Leaf { begin: u32, end: u32 },
}

/// 三角 BVH 节点；`bounds` 是子树包围盒。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshNode {
    pub bounds: Aabb,
    pub kind: MeshNodeKind,
}

/// 构建输入：局部系顶点与三角形索引（`indices.len() % 3 == 0`）。
pub struct MeshBvhInput<'a> {
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [u32],
}

/// 三角 BVH 宿主侧错误；拒绝面在分配与构建前给出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshBvhError {
    /// 没有任何三角形。
    EmptyMesh,
    /// 索引长度不是 3 的整数倍。
    InvalidIndexCount { len: usize },
    /// 三角形数超过节点数组可表示范围。
    TriangleCountOverflow { triangles: usize },
    /// 三角形索引超出顶点数。
    IndexOutOfRange {
        corner: usize,
        index: u32,
        vertices: usize,
    },
    /// 顶点含非有限分量。
    NonFiniteVertex { index: usize },
    /// 更新顶点的长度与构建时不一致。
    VertexCountMismatch { expected: usize, actual: usize },
    /// 顶点或三角形包围盒非法（非有限/上下界颠倒）。
    InvalidBounds { triangle: usize },
}

/// 网格三角形 BVH；顶点与索引保持构建时的顺序。
#[derive(Debug, PartialEq)]
pub struct MeshBvh {
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    order: Vec<u32>,
    triangle_bounds: Vec<Aabb>,
    nodes: Vec<MeshNode>,
}

impl MeshBvh {
    /// 从局部系顶点与三角形索引构建三角 BVH。
    pub fn build(input: MeshBvhInput<'_>) -> Result<Self, MeshBvhError> {
        let MeshBvhInput { vertices, indices } = input;
        if !indices.len().is_multiple_of(3) {
            return Err(MeshBvhError::InvalidIndexCount { len: indices.len() });
        }
        let triangles = indices.len() / 3;
        if triangles == 0 {
            return Err(MeshBvhError::EmptyMesh);
        }
        let capacity = node_capacity(triangles)?;
        for (index, vertex) in vertices.iter().enumerate() {
            if !vertex.iter().all(|value| value.is_finite()) {
                return Err(MeshBvhError::NonFiniteVertex { index });
            }
        }
        for (corner, index) in indices.iter().enumerate() {
            if *index as usize >= vertices.len() {
                return Err(MeshBvhError::IndexOutOfRange {
                    corner,
                    index: *index,
                    vertices: vertices.len(),
                });
            }
        }
        let mut triangle_bounds = Vec::with_capacity(triangles);
        for triangle in 0..triangles {
            triangle_bounds.push(triangle_bounds_at(vertices, indices, triangle)?);
        }
        let mut order: Vec<u32> = (0..triangles as u32).collect();
        let mut nodes = Vec::with_capacity(capacity);
        build_subtree(&mut nodes, &mut order, &triangle_bounds, 0, triangles);
        Ok(Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            order,
            triangle_bounds,
            nodes,
        })
    }

    /// 局部系顶点（构建或最近一次 `set_points` 的值）。
    pub fn vertices(&self) -> &[[f32; 3]] {
        &self.vertices
    }

    /// 三角形索引（每三个一组）。
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// 三角形数。
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// 节点数组；最后一个节点是根。
    pub fn nodes(&self) -> &[MeshNode] {
        &self.nodes
    }

    /// 叶子中的三角形顺序；节点的 `Leaf { begin, end }` 引用它。
    pub fn leaf_order(&self) -> &[u32] {
        &self.order
    }

    /// 动态顶点更新：长度一致且全为有限数；对应上游 `mesh.points = new; mesh.refit()`。
    pub fn set_points(&mut self, vertices: &[[f32; 3]]) -> Result<(), MeshBvhError> {
        if vertices.len() != self.vertices.len() {
            return Err(MeshBvhError::VertexCountMismatch {
                expected: self.vertices.len(),
                actual: vertices.len(),
            });
        }
        for (index, vertex) in vertices.iter().enumerate() {
            if !vertex.iter().all(|value| value.is_finite()) {
                return Err(MeshBvhError::NonFiniteVertex { index });
            }
        }
        self.vertices.copy_from_slice(vertices);
        Ok(())
    }

    /// 用当前顶点重算三角形包围盒与自底向上的内部节点包围盒，不改树结构。
    pub fn refit(&mut self) {
        let vertices = &self.vertices;
        let indices = &self.indices;
        for triangle in 0..self.triangle_count() {
            self.triangle_bounds[triangle] =
                triangle_bounds_at(vertices, indices, triangle).expect("refit 前顶点已校验");
        }
        let bounds = &self.triangle_bounds;
        let order = &self.order;
        for index in 0..self.nodes.len() {
            self.nodes[index].bounds = match self.nodes[index].kind {
                MeshNodeKind::Internal { left, right } => self.nodes[left as usize]
                    .bounds
                    .union(self.nodes[right as usize].bounds),
                MeshNodeKind::Leaf { begin, end } => order[begin as usize..end as usize]
                    .iter()
                    .map(|triangle| bounds[*triangle as usize])
                    .reduce(Aabb::union)
                    .expect("叶子至少一个三角形"),
            };
        }
    }

    /// 宿主侧候选三角形（包围盒与射线相交），按三角形编号升序。
    ///
    /// 只用于主机测试语义检查（例如与暴力遍历对照），不参与设备命中判定。
    pub fn ray_candidates(&self, origin: [f32; 3], direction: [f32; 3]) -> Vec<u32> {
        let mut candidates = Vec::new();
        let mut stack = vec![self.nodes.len() as u32 - 1];
        while let Some(index) = stack.pop() {
            let node = self.nodes[index as usize];
            if !node.bounds.intersects_ray(origin, direction) {
                continue;
            }
            match node.kind {
                MeshNodeKind::Internal { left, right } => {
                    stack.push(left);
                    stack.push(right);
                }
                MeshNodeKind::Leaf { begin, end } => {
                    candidates.extend_from_slice(&self.order[begin as usize..end as usize]);
                }
            }
        }
        candidates.sort_unstable();
        candidates
    }
}

/// 节点数组容量 `2*leaves-1`；溢出在分配前拒绝。
fn node_capacity(triangles: usize) -> Result<usize, MeshBvhError> {
    if triangles > u32::MAX as usize / 2 {
        return Err(MeshBvhError::TriangleCountOverflow { triangles });
    }
    triangles
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .ok_or(MeshBvhError::TriangleCountOverflow { triangles })
}

/// 单个三角形的 AABB：三个顶点分量的 min/max，保持 f32 位型。
fn triangle_bounds_at(
    vertices: &[[f32; 3]],
    indices: &[u32],
    triangle: usize,
) -> Result<Aabb, MeshBvhError> {
    let mut lower = [f32::INFINITY; 3];
    let mut upper = [f32::NEG_INFINITY; 3];
    for corner in 0..3 {
        let vertex = vertices[indices[3 * triangle + corner] as usize];
        for axis in 0..3 {
            lower[axis] = lower[axis].min(vertex[axis]);
            upper[axis] = upper[axis].max(vertex[axis]);
        }
    }
    Aabb::new(lower, upper).map_err(|_| MeshBvhError::InvalidBounds { triangle })
}

/// 递归构建 `order[start..end]`，子节点先入数组，返回子树根索引。
fn build_subtree(
    nodes: &mut Vec<MeshNode>,
    order: &mut [u32],
    triangle_bounds: &[Aabb],
    start: usize,
    end: usize,
) -> u32 {
    if end - start <= LEAF_CAPACITY {
        let bounds = order[start..end]
            .iter()
            .map(|triangle| triangle_bounds[*triangle as usize])
            .reduce(Aabb::union)
            .expect("叶子至少一个三角形");
        let index = nodes.len() as u32;
        nodes.push(MeshNode {
            bounds,
            kind: MeshNodeKind::Leaf {
                begin: start as u32,
                end: end as u32,
            },
        });
        return index;
    }
    let split = choose_split(triangle_bounds, &mut order[start..end]) + start;
    let left = build_subtree(nodes, order, triangle_bounds, start, split);
    let right = build_subtree(nodes, order, triangle_bounds, split, end);
    let bounds = nodes[left as usize]
        .bounds
        .union(nodes[right as usize].bounds);
    let index = nodes.len() as u32;
    nodes.push(MeshNode {
        bounds,
        kind: MeshNodeKind::Internal { left, right },
    });
    index
}

/// 分箱 SAH 选切分点；退化时按中位切分。返回相对 `order` 的切分位置。
fn choose_split(triangle_bounds: &[Aabb], order: &mut [u32]) -> usize {
    let count = order.len();
    let mut centroid_min = [f32::INFINITY; 3];
    let mut centroid_max = [f32::NEG_INFINITY; 3];
    for triangle in order.iter() {
        let center = triangle_bounds[*triangle as usize].center();
        for axis in 0..3 {
            centroid_min[axis] = centroid_min[axis].min(center[axis]);
            centroid_max[axis] = centroid_max[axis].max(center[axis]);
        }
    }
    let extents = [
        centroid_max[0] - centroid_min[0],
        centroid_max[1] - centroid_min[1],
        centroid_max[2] - centroid_min[2],
    ];
    if extents.iter().all(|extent| *extent == 0.0) {
        return count / 2;
    }
    let mut bin_index = vec![0usize; count];
    let mut best: Option<(f32, usize, usize)> = None;
    for axis in 0..3 {
        if extents[axis] == 0.0 {
            continue;
        }
        let inverse = SAH_BINS as f32 / extents[axis];
        for (slot, triangle) in order.iter().enumerate() {
            let center = triangle_bounds[*triangle as usize].center()[axis];
            bin_index[slot] =
                (((center - centroid_min[axis]) * inverse) as usize).min(SAH_BINS - 1);
        }
        let mut counts = [0usize; SAH_BINS];
        let mut lower = [[f32::INFINITY; 3]; SAH_BINS];
        let mut upper = [[f32::NEG_INFINITY; 3]; SAH_BINS];
        for (slot, triangle) in order.iter().enumerate() {
            let bin = bin_index[slot];
            let bounds = triangle_bounds[*triangle as usize];
            counts[bin] += 1;
            for inner in 0..3 {
                lower[bin][inner] = lower[bin][inner].min(bounds.lower[inner]);
                upper[bin][inner] = upper[bin][inner].max(bounds.upper[inner]);
            }
        }
        for split in 1..SAH_BINS {
            let left_count: usize = counts[..split].iter().sum();
            let right_count: usize = counts[split..].iter().sum();
            if left_count == 0 || right_count == 0 {
                continue;
            }
            let left_area = bin_area(&lower[..split], &upper[..split]);
            let right_area = bin_area(&lower[split..], &upper[split..]);
            let cost = left_area * left_count as f32 + right_area * right_count as f32;
            let candidate = (cost, axis, split);
            best = Some(match best {
                Some(current) if current <= candidate => current,
                _ => candidate,
            });
        }
    }
    let Some((_, axis, split)) = best else {
        return count / 2;
    };
    let inverse = SAH_BINS as f32 / extents[axis];
    let mut left = Vec::with_capacity(count);
    let mut right = Vec::with_capacity(count);
    for triangle in order.iter() {
        let center = triangle_bounds[*triangle as usize].center()[axis];
        let bin = (((center - centroid_min[axis]) * inverse) as usize).min(SAH_BINS - 1);
        if bin < split {
            left.push(*triangle);
        } else {
            right.push(*triangle);
        }
    }
    if left.is_empty() || right.is_empty() {
        return count / 2;
    }
    let split_position = left.len();
    left.extend_from_slice(&right);
    order.copy_from_slice(&left);
    split_position
}

/// 箱并集的表面积（只用于启发式，不参与验收）。
fn bin_area(lower: &[[f32; 3]], upper: &[[f32; 3]]) -> f32 {
    let mut union_lower = [f32::INFINITY; 3];
    let mut union_upper = [f32::NEG_INFINITY; 3];
    for (lower, upper) in lower.iter().zip(upper.iter()) {
        for axis in 0..3 {
            union_lower[axis] = union_lower[axis].min(lower[axis]);
            union_upper[axis] = union_upper[axis].max(upper[axis]);
        }
    }
    let extents = [
        union_upper[0] - union_lower[0],
        union_upper[1] - union_lower[1],
        union_upper[2] - union_lower[2],
    ];
    2.0 * (extents[0] * extents[1] + extents[1] * extents[2] + extents[2] * extents[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetra() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3],
        )
    }

    fn build(vertices: &[[f32; 3]], indices: &[u32]) -> MeshBvh {
        MeshBvh::build(MeshBvhInput { vertices, indices }).expect("构建应接受输入")
    }

    #[test]
    fn builds_tree_and_refits_after_point_update() {
        let (vertices, indices) = tetra();
        let mut bvh = build(&vertices, &indices);
        assert_eq!(bvh.triangle_count(), 4);
        let mut sorted = bvh.leaf_order().to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);
        for (index, node) in bvh.nodes().iter().enumerate() {
            if let MeshNodeKind::Internal { left, right } = node.kind {
                assert!((left as usize) < index && (right as usize) < index);
                assert_eq!(
                    node.bounds,
                    bvh.nodes()[left as usize]
                        .bounds
                        .union(bvh.nodes()[right as usize].bounds)
                );
            }
        }
        let moved: Vec<[f32; 3]> = bvh
            .vertices()
            .iter()
            .map(|vertex| [vertex[0] + 2.0, vertex[1] - 1.0, vertex[2] + 0.5])
            .collect();
        bvh.set_points(&moved).unwrap();
        bvh.refit();
        let union = bvh
            .triangle_bounds
            .iter()
            .copied()
            .reduce(Aabb::union)
            .unwrap();
        assert_eq!(bvh.nodes().last().unwrap().bounds, union);
        assert_eq!(bvh.triangle_bounds[0].lower, [2.0, -1.0, 0.5]);
    }

    #[test]
    fn rejects_invalid_inputs_before_building_or_writing() {
        let (vertices, indices) = tetra();
        let build_result = |vertices: &[[f32; 3]], indices: &[u32]| {
            MeshBvh::build(MeshBvhInput { vertices, indices })
        };
        assert_eq!(
            build_result(&vertices, &[0, 1]),
            Err(MeshBvhError::InvalidIndexCount { len: 2 })
        );
        assert_eq!(build_result(&vertices, &[]), Err(MeshBvhError::EmptyMesh));
        assert_eq!(
            build_result(&vertices, &[0, 1, 9]),
            Err(MeshBvhError::IndexOutOfRange {
                corner: 2,
                index: 9,
                vertices: 4,
            })
        );
        let mut broken = vertices.clone();
        broken[2][1] = f32::NAN;
        assert_eq!(
            build_result(&broken, &indices),
            Err(MeshBvhError::NonFiniteVertex { index: 2 })
        );
        let mut bvh = build(&vertices, &indices);
        assert_eq!(
            bvh.set_points(&[[0.0; 3]; 2]),
            Err(MeshBvhError::VertexCountMismatch {
                expected: 4,
                actual: 2,
            })
        );
        let mut broken = bvh.vertices().to_vec();
        broken[0][2] = f32::INFINITY;
        assert_eq!(
            bvh.set_points(&broken),
            Err(MeshBvhError::NonFiniteVertex { index: 0 })
        );
        assert_eq!(bvh.vertices()[0], [0.0; 3]);
    }

    #[test]
    fn accepts_degenerate_triangles_without_panicking() {
        let vertices = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let indices = [0, 0, 0, 0, 1, 2, 0, 0, 1, 0, 1, 0];
        let mut bvh = build(&vertices, &indices);
        assert_eq!(bvh.triangle_count(), 4);
        assert_eq!(
            bvh.triangle_bounds[0],
            Aabb {
                lower: [0.0; 3],
                upper: [0.0; 3]
            }
        );
        bvh.refit();
        assert!(
            !bvh.ray_candidates([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0])
                .is_empty()
        );
    }
}
