//! 场景 BVH 的宿主重拟合。
//!
//! 重拟合只刷新 leaf AABB 与自底向上的内部节点包围盒，不改树结构；节点数组
//! 由构建保证子节点索引小于父节点，因此按索引顺序一遍即可完成。

use super::{Aabb, BvhError, NodeKind, SceneBvh};

/// 用新的 leaf AABB 重拟合 `bvh`；长度不符或 AABB 非法时在写入前拒绝。
pub fn refit_bvh(bvh: &mut SceneBvh, bounds: &[Aabb]) -> Result<(), BvhError> {
    if bounds.len() != bvh.leaf_count() {
        return Err(BvhError::LengthMismatch {
            field: "bounds",
            expected: bvh.leaf_count(),
            actual: bounds.len(),
        });
    }
    for (leaf, bounds) in bounds.iter().enumerate() {
        if !bounds.is_valid() {
            return Err(BvhError::InvalidAabb { leaf });
        }
    }
    bvh.leaves_mut().copy_from_slice(bounds);
    let nodes = bvh.nodes_mut();
    for index in 0..nodes.len() {
        match nodes[index].kind {
            NodeKind::Leaf { leaf } => {
                nodes[index].bounds = bounds[leaf as usize];
            }
            NodeKind::Internal { left, right } => {
                let left_bounds = nodes[left as usize].bounds;
                let right_bounds = nodes[right as usize].bounds;
                nodes[index].bounds = left_bounds.union(right_bounds);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{NodeKind as Kind, SceneBvhInput, build_scene_bvh};
    use super::*;

    fn aabb(lower: [f32; 3], upper: [f32; 3]) -> Aabb {
        Aabb { lower, upper }
    }

    fn scene() -> SceneBvh {
        let bounds = [
            aabb([0.0; 3], [1.0; 3]),
            aabb([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
            aabb([0.0, 2.0, 0.0], [1.0, 3.0, 1.0]),
        ];
        build_scene_bvh(SceneBvhInput {
            bounds: &bounds,
            groups: &[0, 0, 0],
            geom_ids: &[0, 1, 2],
            worlds: 1,
        })
        .unwrap()
    }

    #[test]
    fn refits_leaves_and_internal_nodes_to_new_bounds() {
        let mut bvh = scene();
        let moved = [
            aabb([10.0; 3], [11.0; 3]),
            aabb([12.0, 10.0, 10.0], [13.0, 11.0, 11.0]),
            aabb([10.0, 12.0, 10.0], [11.0, 13.0, 11.0]),
        ];
        refit_bvh(&mut bvh, &moved).unwrap();
        assert_eq!(bvh.leaf_bounds(), &moved);
        assert_eq!(
            bvh.group_bounds(0).unwrap(),
            aabb([10.0; 3], [13.0, 13.0, 11.0])
        );
        let union = moved
            .iter()
            .copied()
            .reduce(|left, right| left.union(right))
            .unwrap();
        assert_eq!(
            bvh.nodes()[bvh.group_root(0).unwrap() as usize].bounds,
            union
        );
        for node in bvh.nodes() {
            match node.kind {
                Kind::Leaf { leaf } => assert_eq!(node.bounds, moved[leaf as usize]),
                Kind::Internal { left, right } => {
                    assert_eq!(
                        node.bounds,
                        bvh.nodes()[left as usize]
                            .bounds
                            .union(bvh.nodes()[right as usize].bounds)
                    );
                }
            }
        }
    }

    #[test]
    fn rejects_length_mismatch_and_invalid_bounds_before_writing() {
        let mut bvh = scene();
        assert_eq!(
            refit_bvh(&mut bvh, &[]),
            Err(BvhError::LengthMismatch {
                field: "bounds",
                expected: 3,
                actual: 0,
            })
        );
        let invalid = [
            aabb([0.0; 3], [1.0; 3]),
            aabb([2.0; 3], [1.0; 3]),
            aabb([0.0; 3], [1.0; 3]),
        ];
        assert_eq!(
            refit_bvh(&mut bvh, &invalid),
            Err(BvhError::InvalidAabb { leaf: 1 })
        );
        assert_eq!(bvh.leaf_bounds()[0], aabb([0.0; 3], [1.0; 3]));
    }
}
