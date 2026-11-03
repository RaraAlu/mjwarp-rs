//! R06-S3a：1D flex 的场景 BVH 叶包围盒（渲染/BVH R06）。
//!
//! 翻译冻结上游 `71da24d` 的 `bvh.py::_compute_flex_bvh_bounds`（L265-311）：1D flex 的
//! 场景图元是「边胶囊」，叶包围盒 = `min(v0,v1) - inflate` / `max(v0,v1) + inflate`
//! （`inflate = (radius, radius, radius)`）；叶序 world-major，`group = worldid`。
//! 2D/3D flex 的图元是 per-flex mesh（`edgeid = -1`），本片不接产品数据，显式拒绝而不是
//! 静默跳过（数据层归 S2c，mesh 机制在 `bvh::flex_mesh`）。
//!
//! 场景叶的 device 编码：`leaf_geom = -1 - flex_local`（`flex_local` = `bvh_nflexgeom` 内
//! 局部号），设备见 `kernel/cuda/render_flex.cu::flex_leaf_hit`。

use super::Aabb;

/// 1D flex 场景叶计算错误；全部在分配前拒绝。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlexLeafError {
    /// 数组长度与计数不一致。
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 索引越界/计数非法。
    Count { field: &'static str },
    /// 顶点位置含非有限分量。
    NonFinite { field: &'static str, index: usize },
    /// 1D 叶路径遇到 2D/3D mesh 图元（`edgeid < 0`）。
    UnsupportedMeshPrimitive { index: usize },
}

impl std::fmt::Display for FlexLeafError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Length {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} 长度不匹配：期望 {expected}，实际 {actual}"
            ),
            Self::Count { field } => write!(formatter, "{field} 计数/索引非法"),
            Self::NonFinite { field, index } => write!(formatter, "{field}[{index}] 不是有限数"),
            Self::UnsupportedMeshPrimitive { index } => write!(
                formatter,
                "1D flex 叶只支持胶囊图元：flex_geom_edgeid[{index}] < 0（2D/3D 归 S2c）"
            ),
        }
    }
}

impl std::error::Error for FlexLeafError {}

/// 1D flex 场景叶输入（flex 通道的宿主切片；`flexvert_xpos` 展平 `[world, vert, 3]`）。
pub struct OneDimensionalLeafInput<'a> {
    pub worlds: usize,
    pub nflexvert: usize,
    pub flex_vertadr: &'a [i32],
    pub flex_edge: &'a [i32],
    pub flex_radius: &'a [f32],
    pub flexvert_xpos: &'a [f32],
    pub geom_flexid: &'a [i32],
    pub geom_edgeid: &'a [i32],
}

/// 场景 BVH 的 flex 叶块：bound 与 group 一一对应，顺序 world-major。
#[derive(Clone, Debug, PartialEq)]
pub struct FlexSceneLeaves {
    pub bounds: Vec<Aabb>,
    pub groups: Vec<u32>,
}

/// 1D flex 的场景叶包围盒（上游 `bvh.py::_compute_flex_bvh_bounds` L284-311）。
pub fn one_dimensional_scene_leaves(
    input: OneDimensionalLeafInput<'_>,
) -> Result<FlexSceneLeaves, FlexLeafError> {
    let OneDimensionalLeafInput {
        worlds,
        nflexvert,
        flex_vertadr,
        flex_edge,
        flex_radius,
        flexvert_xpos,
        geom_flexid,
        geom_edgeid,
    } = input;
    if worlds == 0 || geom_flexid.is_empty() {
        return Err(FlexLeafError::Count {
            field: "bvh_nflexgeom",
        });
    }
    if geom_flexid.len() != geom_edgeid.len() {
        return Err(FlexLeafError::Length {
            field: "flex_geom_edgeid",
            expected: geom_flexid.len(),
            actual: geom_edgeid.len(),
        });
    }
    let world_entries = nflexvert
        .checked_mul(3)
        .ok_or(FlexLeafError::Count { field: "nflexvert" })?;
    let expected_positions = worlds
        .checked_mul(world_entries)
        .ok_or(FlexLeafError::Count {
            field: "flexvert_xpos",
        })?;
    if flexvert_xpos.len() != expected_positions {
        return Err(FlexLeafError::Length {
            field: "flexvert_xpos",
            expected: expected_positions,
            actual: flexvert_xpos.len(),
        });
    }
    let mut bounds = Vec::with_capacity(worlds * geom_flexid.len());
    let mut groups = Vec::with_capacity(worlds * geom_flexid.len());
    for world in 0..worlds {
        let base = world * world_entries;
        for (index, (&flexid, &edgeid)) in geom_flexid.iter().zip(geom_edgeid).enumerate() {
            if edgeid < 0 {
                return Err(FlexLeafError::UnsupportedMeshPrimitive { index });
            }
            let flex = usize::try_from(flexid).map_err(|_| FlexLeafError::Count {
                field: "flex_geom_flexid",
            })?;
            let vert_adr = *flex_vertadr.get(flex).ok_or(FlexLeafError::Count {
                field: "flex_vertadr",
            })?;
            let vert_adr = usize::try_from(vert_adr).map_err(|_| FlexLeafError::Count {
                field: "flex_vertadr",
            })?;
            let radius = *flex_radius.get(flex).ok_or(FlexLeafError::Count {
                field: "flex_radius",
            })?;
            if !radius.is_finite() {
                return Err(FlexLeafError::NonFinite {
                    field: "flex_radius",
                    index: flex,
                });
            }
            let edge = usize::try_from(edgeid).map_err(|_| FlexLeafError::Count {
                field: "flex_geom_edgeid",
            })?;
            let slot = edge
                .checked_mul(2)
                .ok_or(FlexLeafError::Count { field: "flex_edge" })?;
            let local0 = *flex_edge
                .get(slot)
                .ok_or(FlexLeafError::Count { field: "flex_edge" })?;
            let local1 = *flex_edge
                .get(slot + 1)
                .ok_or(FlexLeafError::Count { field: "flex_edge" })?;
            let v0 = vertex(flexvert_xpos, base, vert_adr, local0, world)?;
            let v1 = vertex(flexvert_xpos, base, vert_adr, local1, world)?;
            let mut lower = [0.0f32; 3];
            let mut upper = [0.0f32; 3];
            for axis in 0..3 {
                // 上游 `wp.min(v0,v1) - inflate` / `wp.max(v0,v1) + inflate`，逐分量同序。
                lower[axis] = v0[axis].min(v1[axis]) - radius;
                upper[axis] = v0[axis].max(v1[axis]) + radius;
            }
            bounds.push(Aabb::new(lower, upper).map_err(|_| FlexLeafError::Count {
                field: "flex_leaf_bounds",
            })?);
            groups
                .push(u32::try_from(world).map_err(|_| FlexLeafError::Count { field: "worlds" })?);
        }
    }
    Ok(FlexSceneLeaves { bounds, groups })
}

/// 取一个 flex 边端点的世界位置（`base` 是该 world 在展平数组里的起点）。
fn vertex(
    positions: &[f32],
    base: usize,
    vert_adr: usize,
    local: i32,
    world: usize,
) -> Result<[f32; 3], FlexLeafError> {
    let local = usize::try_from(local).map_err(|_| FlexLeafError::Count { field: "flex_edge" })?;
    let index = vert_adr
        .checked_add(local)
        .and_then(|value| value.checked_mul(3))
        .and_then(|value| value.checked_add(base))
        .ok_or(FlexLeafError::Count { field: "flex_edge" })?;
    let mut out = [0.0f32; 3];
    for (axis, slot) in out.iter_mut().enumerate() {
        let value = *positions.get(index + axis).ok_or(FlexLeafError::Count {
            field: "flexvert_xpos",
        })?;
        if !value.is_finite() {
            return Err(FlexLeafError::NonFinite {
                field: "flexvert_xpos",
                index: world,
            });
        }
        *slot = value;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{FlexLeafError, OneDimensionalLeafInput, one_dimensional_scene_leaves};
    use crate::render::bvh::Aabb;

    /// 单世界单边：包围盒 = 两端点逐分量 min/max 再按半径外扩。
    #[test]
    fn one_dimensional_leaf_inflates_the_capsule_bounds() {
        let positions = [0.0f32, 0.0, 0.0, 1.0, 2.0, -1.0];
        let leaves = one_dimensional_scene_leaves(OneDimensionalLeafInput {
            worlds: 1,
            nflexvert: 2,
            flex_vertadr: &[0],
            flex_edge: &[0, 1],
            flex_radius: &[0.5],
            flexvert_xpos: &positions,
            geom_flexid: &[0],
            geom_edgeid: &[0],
        })
        .expect("输入合法");
        assert_eq!(leaves.bounds.len(), 1);
        assert_eq!(leaves.groups, [0]);
        assert_eq!(
            leaves.bounds[0],
            Aabb {
                lower: [-0.5, -0.5, -1.5],
                upper: [1.5, 2.5, 0.5],
            }
        );
    }

    /// 多 world：叶序 world-major，group = worldid，同一图元用各自 world 的位置。
    #[test]
    fn one_dimensional_leaves_are_world_major_and_grouped() {
        let positions = [
            0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, // world 0
            10.0, 0.0, 0.0, 11.0, 0.0, 0.0, // world 1
        ];
        let leaves = one_dimensional_scene_leaves(OneDimensionalLeafInput {
            worlds: 2,
            nflexvert: 2,
            flex_vertadr: &[0],
            flex_edge: &[0, 1],
            flex_radius: &[1.0],
            flexvert_xpos: &positions,
            geom_flexid: &[0],
            geom_edgeid: &[0],
        })
        .expect("输入合法");
        assert_eq!(leaves.groups, [0, 1]);
        assert_eq!(leaves.bounds[0].lower, [-1.0, -1.0, -1.0]);
        assert_eq!(leaves.bounds[0].upper, [2.0, 1.0, 1.0]);
        assert_eq!(leaves.bounds[1].lower, [9.0, -1.0, -1.0]);
        assert_eq!(leaves.bounds[1].upper, [12.0, 1.0, 1.0]);
    }

    /// 2D/3D mesh 图元（edgeid = -1）在 1D 叶路径上显式拒绝（不是静默跳过）。
    #[test]
    fn one_dimensional_leaves_reject_mesh_primitives() {
        let error = one_dimensional_scene_leaves(OneDimensionalLeafInput {
            worlds: 1,
            nflexvert: 2,
            flex_vertadr: &[0],
            flex_edge: &[0, 1],
            flex_radius: &[0.1],
            flexvert_xpos: &[0.0; 6],
            geom_flexid: &[0],
            geom_edgeid: &[-1],
        })
        .expect_err("2D/3D 图元必须拒绝");
        assert_eq!(error, FlexLeafError::UnsupportedMeshPrimitive { index: 0 });
    }

    /// 长度/映射不一致在构建前拒绝。
    #[test]
    fn one_dimensional_leaves_reject_inconsistent_inputs() {
        let error = one_dimensional_scene_leaves(OneDimensionalLeafInput {
            worlds: 1,
            nflexvert: 2,
            flex_vertadr: &[0],
            flex_edge: &[0, 1],
            flex_radius: &[0.1],
            flexvert_xpos: &[0.0; 3],
            geom_flexid: &[0],
            geom_edgeid: &[0],
        })
        .expect_err("位置长度必须匹配");
        assert!(matches!(error, FlexLeafError::Length { .. }));
        let error = one_dimensional_scene_leaves(OneDimensionalLeafInput {
            worlds: 1,
            nflexvert: 2,
            flex_vertadr: &[0],
            flex_edge: &[0, 1],
            flex_radius: &[0.1],
            flexvert_xpos: &[0.0; 6],
            geom_flexid: &[0],
            geom_edgeid: &[],
        })
        .expect_err("映射长度必须一致");
        assert!(matches!(error, FlexLeafError::Length { .. }));
    }
}
