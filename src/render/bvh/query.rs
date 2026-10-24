//! 宿主侧语义查询：从某个 world 的 group root 出发做射线-AABB 遍历。
//!
//! 这里只返回候选 leaf（是否被射线 AABB 相交），不计算 primitive 命中几何；
//! GPU 遍历与命中求交属 S2b。验收只用它检查「分组不跨 world」等语义。

use super::{BvhError, NodeKind, SceneBvh};

impl SceneBvh {
    /// 从 `world` 分组根出发的射线候选 leaf 索引（升序）。
    ///
    /// 遍历只经过该 group 的节点，因此候选 leaf 必属该 world；world 越界报错。
    pub fn ray_leaf_candidates(
        &self,
        world: usize,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Result<Vec<u32>, BvhError> {
        let mut stack = vec![self.group_root(world)?];
        let nodes = self.nodes();
        let mut candidates = Vec::new();
        while let Some(index) = stack.pop() {
            let node = nodes[index as usize];
            if !node.bounds.intersects_ray(origin, direction) {
                continue;
            }
            match node.kind {
                NodeKind::Leaf { leaf } => candidates.push(leaf),
                NodeKind::Internal { left, right } => {
                    stack.push(left);
                    stack.push(right);
                }
            }
        }
        candidates.sort_unstable();
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Aabb, NodeKind, SceneBvh, SceneBvhInput, build_scene_bvh};

    fn aabb(lower: [f32; 3], upper: [f32; 3]) -> Aabb {
        Aabb { lower, upper }
    }

    fn two_world_scene() -> (Vec<Aabb>, Vec<u32>, SceneBvh) {
        let bounds = vec![
            aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            aabb([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
            aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            aabb([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
        ];
        let groups = vec![0, 0, 1, 1];
        let bvh = build_scene_bvh(SceneBvhInput {
            bounds: &bounds,
            groups: &groups,
            geom_ids: &[0, 1, 0, 1],
            worlds: 2,
        })
        .unwrap();
        (bounds, groups, bvh)
    }

    #[test]
    fn candidates_never_cross_world_groups() {
        let (_, groups, bvh) = two_world_scene();
        for (world, origin) in [(0usize, [-1.0, 0.5, 0.5]), (1usize, [-1.0, 0.5, 0.5])] {
            let candidates = bvh
                .ray_leaf_candidates(world, origin, [1.0, 0.0, 0.0])
                .unwrap();
            assert_eq!(candidates.len(), 2, "world {world} 应命中两个 leaf");
            for candidate in candidates {
                assert_eq!(bvh.leaf_group(candidate), Some(world as u32));
                assert_eq!(bvh.leaf_group(candidate), Some(groups[candidate as usize]));
            }
        }
    }

    #[test]
    fn reports_miss_and_rejects_unknown_world() {
        let (_, _, bvh) = two_world_scene();
        assert!(
            bvh.ray_leaf_candidates(0, [0.5, 0.5, 5.0], [0.0, 0.0, 1.0])
                .unwrap()
                .is_empty()
        );
        assert!(matches!(
            bvh.ray_leaf_candidates(2, [0.0; 3], [1.0, 0.0, 0.0]),
            Err(super::super::BvhError::WorldOutOfRange {
                world: 2,
                worlds: 2
            })
        ));
    }

    #[test]
    fn traversal_visits_each_leaf_at_most_once() {
        let (_, _, bvh) = two_world_scene();
        let candidates = bvh
            .ray_leaf_candidates(0, [-5.0, 0.5, 0.5], [1.0, 0.0, 0.0])
            .unwrap();
        let unique: std::collections::BTreeSet<_> = candidates.iter().collect();
        assert_eq!(unique.len(), candidates.len());
        assert!(
            bvh.nodes()
                .iter()
                .any(|node| matches!(node.kind, NodeKind::Internal { .. }))
        );
    }
}
