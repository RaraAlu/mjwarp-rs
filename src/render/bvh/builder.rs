//! 分组场景 BVH 的宿主构建。
//!
//! 每个 world 是独立的 group（上游 `group == worldid`），各 group 单独建树，
//! group root 指向该树根，因此查询从某个 group root 出发时天然不会跨 world。
//! 构建用分箱 SAH（12 箱，逐轴择优）；节点先子后父追加，使 refit 可按索引
//! 正序自底向上刷新。节点布局是本实现细节，验收只做语义断言。

use super::{Aabb, BvhError};

const SAH_BINS: usize = 12;

/// 节点类型：内部节点或 leaf。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Internal { left: u32, right: u32 },
    Leaf { leaf: u32 },
}

/// BVH 节点；`bounds` 是子树包围盒。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhNode {
    pub bounds: Aabb,
    pub kind: NodeKind,
}

/// 构建输入：leaf 的 AABB、world 分组与 geom id，`worlds` 为 world 数。
pub struct SceneBvhInput<'a> {
    pub bounds: &'a [Aabb],
    pub groups: &'a [u32],
    pub geom_ids: &'a [i32],
    pub worlds: usize,
}

/// 分组场景 BVH。
#[derive(Debug, PartialEq)]
pub struct SceneBvh {
    nodes: Vec<BvhNode>,
    leaf_bounds: Vec<Aabb>,
    leaf_groups: Vec<u32>,
    leaf_geom_ids: Vec<i32>,
    group_roots: Vec<u32>,
}

impl SceneBvh {
    pub fn worlds(&self) -> usize {
        self.group_roots.len()
    }

    pub fn leaf_count(&self) -> usize {
        self.leaf_bounds.len()
    }

    pub fn nodes(&self) -> &[BvhNode] {
        &self.nodes
    }

    pub fn leaf_bounds(&self) -> &[Aabb] {
        &self.leaf_bounds
    }

    pub fn leaf_group(&self, leaf: u32) -> Option<u32> {
        self.leaf_groups.get(leaf as usize).copied()
    }

    pub fn leaf_geom_id(&self, leaf: u32) -> Option<i32> {
        self.leaf_geom_ids.get(leaf as usize).copied()
    }

    pub fn group_root(&self, world: usize) -> Result<u32, BvhError> {
        self.group_roots
            .get(world)
            .copied()
            .ok_or(BvhError::WorldOutOfRange {
                world,
                worlds: self.group_roots.len(),
            })
    }

    /// 该 world 分组的语义包围盒（group 内所有 leaf 的并集）。
    pub fn group_bounds(&self, world: usize) -> Result<Aabb, BvhError> {
        let root = self.group_root(world)?;
        Ok(self.nodes[root as usize].bounds)
    }

    pub(crate) fn leaves_mut(&mut self) -> &mut [Aabb] {
        &mut self.leaf_bounds
    }

    pub(crate) fn nodes_mut(&mut self) -> &mut [BvhNode] {
        &mut self.nodes
    }
}

/// 节点数组容量 `2*leaves-1`；溢出在分配前拒绝。
fn node_capacity(leaves: usize) -> Result<usize, BvhError> {
    if leaves == 0 || leaves > u32::MAX as usize / 2 {
        return Err(BvhError::LeafCountOverflow { leaves });
    }
    leaves
        .checked_mul(2)
        .and_then(|value| value.checked_sub(1))
        .ok_or(BvhError::LeafCountOverflow { leaves })
}

/// 构建场景 BVH；空场景、越界 group、缺失 world 与非法计数在分配前拒绝。
pub fn build_scene_bvh(input: SceneBvhInput<'_>) -> Result<SceneBvh, BvhError> {
    let SceneBvhInput {
        bounds,
        groups,
        geom_ids,
        worlds,
    } = input;
    if worlds == 0 {
        return Err(BvhError::InvalidWorldCount { worlds });
    }
    if groups.len() != bounds.len() {
        return Err(BvhError::LengthMismatch {
            field: "groups",
            expected: bounds.len(),
            actual: groups.len(),
        });
    }
    if geom_ids.len() != bounds.len() {
        return Err(BvhError::LengthMismatch {
            field: "geom_ids",
            expected: bounds.len(),
            actual: geom_ids.len(),
        });
    }
    if bounds.is_empty() {
        return Err(BvhError::EmptyScene);
    }
    let capacity = node_capacity(bounds.len())?;
    for (leaf, bounds) in bounds.iter().enumerate() {
        if !bounds.is_valid() {
            return Err(BvhError::InvalidAabb { leaf });
        }
    }

    let mut per_world: Vec<Vec<u32>> = vec![Vec::new(); worlds];
    for (leaf, group) in groups.iter().enumerate() {
        let group = *group as usize;
        if group >= worlds {
            return Err(BvhError::GroupOutOfRange {
                leaf,
                group,
                worlds,
            });
        }
        per_world[group].push(leaf as u32);
    }
    for (world, leaves) in per_world.iter().enumerate() {
        if leaves.is_empty() {
            return Err(BvhError::EmptyWorld { world });
        }
    }

    let mut nodes: Vec<BvhNode> = Vec::with_capacity(capacity);
    let mut group_roots = Vec::with_capacity(worlds);
    for mut leaves in per_world {
        let count = leaves.len();
        let root = build_subtree(&mut nodes, bounds, &mut leaves, 0, count);
        group_roots.push(root);
    }
    Ok(SceneBvh {
        nodes,
        leaf_bounds: bounds.to_vec(),
        leaf_groups: groups.to_vec(),
        leaf_geom_ids: geom_ids.to_vec(),
        group_roots,
    })
}

/// 递归构建 `ids[start..end]`，子节点先入数组，返回子树根索引。
fn build_subtree(
    nodes: &mut Vec<BvhNode>,
    leaf_bounds: &[Aabb],
    ids: &mut [u32],
    start: usize,
    end: usize,
) -> u32 {
    if end - start == 1 {
        let leaf = ids[start];
        let index = nodes.len() as u32;
        nodes.push(BvhNode {
            bounds: leaf_bounds[leaf as usize],
            kind: NodeKind::Leaf { leaf },
        });
        return index;
    }
    let split = choose_split(leaf_bounds, &mut ids[start..end]) + start;
    let left = build_subtree(nodes, leaf_bounds, ids, start, split);
    let right = build_subtree(nodes, leaf_bounds, ids, split, end);
    let bounds = nodes[left as usize]
        .bounds
        .union(nodes[right as usize].bounds);
    let index = nodes.len() as u32;
    nodes.push(BvhNode {
        bounds,
        kind: NodeKind::Internal { left, right },
    });
    index
}

/// 分箱 SAH 选切分点；退化时按中位切分。返回相对 `ids` 的切分位置。
fn choose_split(leaf_bounds: &[Aabb], ids: &mut [u32]) -> usize {
    let count = ids.len();
    let mut centroid_min = [f32::INFINITY; 3];
    let mut centroid_max = [f32::NEG_INFINITY; 3];
    for &leaf in ids.iter() {
        let center = leaf_bounds[leaf as usize].center();
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
        for (slot, &leaf) in ids.iter().enumerate() {
            let center = leaf_bounds[leaf as usize].center()[axis];
            let bin = (((center - centroid_min[axis]) * inverse) as usize).min(SAH_BINS - 1);
            bin_index[slot] = bin;
        }
        let mut counts = [0usize; SAH_BINS];
        let mut lower = [[f32::INFINITY; 3]; SAH_BINS];
        let mut upper = [[f32::NEG_INFINITY; 3]; SAH_BINS];
        for (slot, &leaf) in ids.iter().enumerate() {
            let bin = bin_index[slot];
            let bounds = leaf_bounds[leaf as usize];
            counts[bin] += 1;
            for axis_inner in 0..3 {
                lower[bin][axis_inner] = lower[bin][axis_inner].min(bounds.lower[axis_inner]);
                upper[bin][axis_inner] = upper[bin][axis_inner].max(bounds.upper[axis_inner]);
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
    let mut left_ids = Vec::with_capacity(count);
    let mut right_ids = Vec::with_capacity(count);
    for &leaf in ids.iter() {
        let center = leaf_bounds[leaf as usize].center()[axis];
        let bin = (((center - centroid_min[axis]) * inverse) as usize).min(SAH_BINS - 1);
        if bin < split {
            left_ids.push(leaf);
        } else {
            right_ids.push(leaf);
        }
    }
    if left_ids.is_empty() || right_ids.is_empty() {
        return count / 2;
    }
    let split_position = left_ids.len();
    left_ids.extend_from_slice(&right_ids);
    ids.copy_from_slice(&left_ids);
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

    fn aabb(lower: [f32; 3], upper: [f32; 3]) -> Aabb {
        Aabb { lower, upper }
    }

    #[test]
    fn rejects_empty_and_mismatched_inputs_before_building() {
        let empty: [Aabb; 0] = [];
        assert_eq!(
            build_scene_bvh(SceneBvhInput {
                bounds: &empty,
                groups: &[],
                geom_ids: &[],
                worlds: 1,
            }),
            Err(BvhError::EmptyScene)
        );
        let bounds = [aabb([0.0; 3], [1.0; 3])];
        assert_eq!(
            build_scene_bvh(SceneBvhInput {
                bounds: &bounds,
                groups: &[0, 1],
                geom_ids: &[0],
                worlds: 1,
            }),
            Err(BvhError::LengthMismatch {
                field: "groups",
                expected: 1,
                actual: 2,
            })
        );
        assert_eq!(
            build_scene_bvh(SceneBvhInput {
                bounds: &bounds,
                groups: &[0],
                geom_ids: &[0],
                worlds: 0,
            }),
            Err(BvhError::InvalidWorldCount { worlds: 0 })
        );
        assert_eq!(
            build_scene_bvh(SceneBvhInput {
                bounds: &bounds,
                groups: &[2],
                geom_ids: &[0],
                worlds: 2,
            }),
            Err(BvhError::GroupOutOfRange {
                leaf: 0,
                group: 2,
                worlds: 2,
            })
        );
        assert_eq!(
            build_scene_bvh(SceneBvhInput {
                bounds: &bounds,
                groups: &[0],
                geom_ids: &[0],
                worlds: 2,
            }),
            Err(BvhError::EmptyWorld { world: 1 })
        );
    }

    #[test]
    fn node_capacity_rejects_overflow_without_allocating() {
        assert_eq!(node_capacity(3), Ok(5));
        assert_eq!(
            node_capacity(usize::MAX),
            Err(BvhError::LeafCountOverflow { leaves: usize::MAX })
        );
        assert_eq!(
            node_capacity(0),
            Err(BvhError::LeafCountOverflow { leaves: 0 })
        );
    }

    #[test]
    fn builds_group_roots_with_children_before_parents() {
        let bounds = [
            aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            aabb([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
            aabb([0.0, 2.0, 0.0], [1.0, 3.0, 1.0]),
        ];
        let world = build_scene_bvh(SceneBvhInput {
            bounds: &bounds,
            groups: &[0, 0, 0],
            geom_ids: &[0, 1, 2],
            worlds: 1,
        })
        .unwrap();
        assert_eq!(world.nodes().len(), 5);
        for (index, node) in world.nodes().iter().enumerate() {
            if let NodeKind::Internal { left, right } = node.kind {
                assert!((left as usize) < index && (right as usize) < index);
                assert_eq!(
                    node.bounds,
                    world.nodes()[left as usize]
                        .bounds
                        .union(world.nodes()[right as usize].bounds)
                );
            }
        }
        assert_eq!(
            world.group_bounds(0).unwrap(),
            aabb([0.0; 3], [3.0, 3.0, 1.0])
        );
    }

    #[test]
    fn splits_worlds_into_independent_trees() {
        let bounds = [
            aabb([0.0; 3], [1.0; 3]),
            aabb([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]),
        ];
        let worlds = build_scene_bvh(SceneBvhInput {
            bounds: &bounds,
            groups: &[0, 1],
            geom_ids: &[0, 1],
            worlds: 2,
        })
        .unwrap();
        assert_eq!(worlds.group_bounds(0).unwrap(), bounds[0]);
        assert_eq!(worlds.group_bounds(1).unwrap(), bounds[1]);
        assert_eq!(worlds.leaf_group(0), Some(0));
        assert_eq!(worlds.leaf_group(1), Some(1));
        assert_eq!(worlds.leaf_geom_id(1), Some(1));
    }
}
