//! 上游 `_hfield_filter`：hfield 与其他 geom 的粗筛与紧 AABB 边界。

use super::super::gjk::{Geom, support};
use super::HfieldData;

/// 上游 `_hfield_filter` 返回的紧 AABB 边界（height field 局部帧）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HfieldBounds {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
    pub zmin: f32,
    pub zmax: f32,
}

/// 上游 `_hfield_filter`；`None` 表示粗筛拒绝（无可碰撞棱柱）。
pub(crate) fn hfield_filter(
    hfield: &HfieldData,
    geom2: &Geom<'_>,
    geomtype2: i32,
    other_rbound: f32,
    margin: f32,
) -> Option<HfieldBounds> {
    let size1 = hfield.size;
    let pos = geom2.pos;
    // box-sphere test: horizontal plane（size[0]、size[1] 为半宽）。
    for axis in 0..2 {
        if size1[axis] < pos[axis] - other_rbound - margin
            || -size1[axis] > pos[axis] + other_rbound + margin
        {
            return None;
        }
    }
    // box-sphere test: vertical direction（size[2] 顶、size[3] 底）。
    if size1[2] < pos[2] - other_rbound - margin {
        return None;
    }
    if -size1[3] > pos[2] + other_rbound + margin {
        return None;
    }

    // 用支撑函数取 geom2 在 hfield 帧内的紧 AABB。
    let xmax = support(geom2, geomtype2, [1.0, 0.0, 0.0]).point[0];
    let xmin = support(geom2, geomtype2, [-1.0, 0.0, 0.0]).point[0];
    let ymax = support(geom2, geomtype2, [0.0, 1.0, 0.0]).point[1];
    let ymin = support(geom2, geomtype2, [0.0, -1.0, 0.0]).point[1];
    let zmax = support(geom2, geomtype2, [0.0, 0.0, 1.0]).point[2];
    let zmin = support(geom2, geomtype2, [0.0, 0.0, -1.0]).point[2];

    // box-box test。
    if xmin - margin > size1[0]
        || xmax + margin < -size1[0]
        || ymin - margin > size1[1]
        || ymax + margin < -size1[1]
        || zmin - margin > size1[2]
        || zmax + margin < -size1[3]
    {
        return None;
    }
    Some(HfieldBounds {
        xmin,
        xmax,
        ymin,
        ymax,
        zmin,
        zmax,
    })
}
