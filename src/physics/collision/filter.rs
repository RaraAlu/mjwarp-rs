//! 上游宽相过滤原语：`_plane_filter`/`_sphere_filter`/`_aabb_filter`/
//! `_obb_filter` 与 `_broadphase_filter` 的宿主侧 f32 转写。
//!
//! 公式逐分支对齐冻结上游 `mujoco_warp/_src/collision_driver.py`（提交
//! `71da24d956…`，Apache-2.0）：平面用第三列作法线；球用包围球距离；
//! AABB 用 Ericson《Real-time Collision Detection》4.2 的全局 AABB 分离；
//! OBB 用 Gottschalk 等（`mj_collideOBB`）的 6 轴分离测试。

use super::{BroadphaseFilter, BroadphaseModel, BroadphaseState};

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn sub3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

/// 行主序 3×3 矩阵乘向量（上游 `xmat @ v`）。
fn mat_vec3(matrix: &[f32; 9], vector: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * vector[0] + matrix[1] * vector[1] + matrix[2] * vector[2],
        matrix[3] * vector[0] + matrix[4] * vector[1] + matrix[5] * vector[2],
        matrix[6] * vector[0] + matrix[7] * vector[1] + matrix[8] * vector[2],
    ]
}

/// 列向量：上游 `wp.vec3(xmat[0, k], xmat[1, k], xmat[2, k])`。
fn mat_column(matrix: &[f32; 9], column: usize) -> [f32; 3] {
    [matrix[column], matrix[3 + column], matrix[6 + column]]
}

/// 上游 `_plane_filter`：`size1`/`size2` 实为两条 geom 的 `rbound`。
#[allow(clippy::too_many_arguments)]
pub(crate) fn plane_filter(
    size1: f32,
    size2: f32,
    margin1: f32,
    margin2: f32,
    xpos1: [f32; 3],
    xpos2: [f32; 3],
    xmat1: &[f32; 9],
    xmat2: &[f32; 9],
) -> bool {
    if size1 == 0.0 {
        let normal1 = mat_column(xmat1, 2);
        let dist = dot3(sub3(xpos2, xpos1), normal1);
        dist <= size2 + margin1 + margin2
    } else if size2 == 0.0 {
        let normal2 = mat_column(xmat2, 2);
        let dist = dot3(sub3(xpos1, xpos2), normal2);
        dist <= size1 + margin1 + margin2
    } else {
        true
    }
}

/// 上游 `_sphere_filter`：包围球重叠检测。
pub(crate) fn sphere_filter(
    size1: f32,
    size2: f32,
    margin1: f32,
    margin2: f32,
    xpos1: [f32; 3],
    xpos2: [f32; 3],
) -> bool {
    let bound = size1 + size2 + margin1 + margin2;
    let difference = sub3(xpos2, xpos1);
    dot3(difference, difference) <= bound * bound
}

/// 上游 `_aabb_filter`：全局 AABB 分离测试（Ericson 4.2）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn aabb_filter(
    center1: [f32; 3],
    center2: [f32; 3],
    size1: [f32; 3],
    size2: [f32; 3],
    margin1: f32,
    margin2: f32,
    xpos1: [f32; 3],
    xpos2: [f32; 3],
    xmat1: &[f32; 9],
    xmat2: &[f32; 9],
) -> bool {
    let center1 = {
        let local = mat_vec3(xmat1, center1);
        [
            local[0] + xpos1[0],
            local[1] + xpos1[1],
            local[2] + xpos1[2],
        ]
    };
    let center2 = {
        let local = mat_vec3(xmat2, center2);
        [
            local[0] + xpos2[0],
            local[1] + xpos2[1],
            local[2] + xpos2[2],
        ]
    };
    let margin = margin1 + margin2;
    let sign = [-1.0_f32, 1.0];

    let mut max1 = [-f32::MAX; 3];
    let mut min1 = [f32::MAX; 3];
    let mut max2 = [-f32::MAX; 3];
    let mut min2 = [f32::MAX; 3];
    for index in 0..2 {
        for inner in 0..2 {
            for outer in 0..2 {
                let corner1 = [
                    sign[index] * size1[0],
                    sign[inner] * size1[1],
                    sign[outer] * size1[2],
                ];
                let pos1 = mat_vec3(xmat1, corner1);
                let corner2 = [
                    sign[index] * size2[0],
                    sign[inner] * size2[1],
                    sign[outer] * size2[2],
                ];
                let pos2 = mat_vec3(xmat2, corner2);
                for axis in 0..3 {
                    if pos1[axis] > max1[axis] {
                        max1[axis] = pos1[axis];
                    }
                    if pos1[axis] < min1[axis] {
                        min1[axis] = pos1[axis];
                    }
                    if pos2[axis] > max2[axis] {
                        max2[axis] = pos2[axis];
                    }
                    if pos2[axis] < min2[axis] {
                        min2[axis] = pos2[axis];
                    }
                }
            }
        }
    }
    for axis in 0..3 {
        if center1[axis] + max1[axis] + margin < center2[axis] + min2[axis] {
            return false;
        }
        if center2[axis] + max2[axis] + margin < center1[axis] + min1[axis] {
            return false;
        }
    }
    true
}

/// 上游 `_obb_filter`：OBB 6 轴分离测试（Gottschalk 等）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn obb_filter(
    center1: [f32; 3],
    center2: [f32; 3],
    size1: [f32; 3],
    size2: [f32; 3],
    margin1: f32,
    margin2: f32,
    xpos1: [f32; 3],
    xpos2: [f32; 3],
    xmat1: &[f32; 9],
    xmat2: &[f32; 9],
) -> bool {
    let margin = margin1 + margin2;
    let xcenter = [
        {
            let local = mat_vec3(xmat1, center1);
            [
                local[0] + xpos1[0],
                local[1] + xpos1[1],
                local[2] + xpos1[2],
            ]
        },
        {
            let local = mat_vec3(xmat2, center2);
            [
                local[0] + xpos2[0],
                local[1] + xpos2[1],
                local[2] + xpos2[2],
            ]
        },
    ];
    let normal = [
        mat_column(xmat1, 0),
        mat_column(xmat1, 1),
        mat_column(xmat1, 2),
        mat_column(xmat2, 0),
        mat_column(xmat2, 1),
        mat_column(xmat2, 2),
    ];
    for selector in 0..2 {
        for axis in 0..3 {
            let direction = normal[3 * selector + axis];
            let projection = [dot3(xcenter[0], direction), dot3(xcenter[1], direction)];
            let radius0 = (size1[0] * dot3(normal[0], direction)).abs()
                + (size1[1] * dot3(normal[1], direction)).abs()
                + (size1[2] * dot3(normal[2], direction)).abs();
            let radius1 = (size2[0] * dot3(normal[3], direction)).abs()
                + (size2[1] * dot3(normal[4], direction)).abs()
                + (size2[2] * dot3(normal[5], direction)).abs();
            if radius0 + radius1 + margin < (projection[1] - projection[0]).abs() {
                return false;
            }
        }
    }
    true
}

/// 上游 `_broadphase_filter`：按位模式执行预筛。
///
/// 任一侧 `rbound == 0`（平面）时只走 plane filter（PLANE 位未置位则不检查）；
/// 否则按 SPHERE/AABB/OBB 位逐项检查，全部通过才返回 true。
pub(crate) fn passes(
    filter: BroadphaseFilter,
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    world: usize,
    geom1: usize,
    geom2: usize,
) -> bool {
    let index1 = world * model.ngeom + geom1;
    let index2 = world * model.ngeom + geom2;
    let rbound1 = model.rbound(world, geom1);
    let rbound2 = model.rbound(world, geom2);
    let margin1 = model.margin(world, geom1) + model.gap(world, geom1);
    let margin2 = model.margin(world, geom2) + model.gap(world, geom2);
    let xpos1 = state.geom_xpos[index1];
    let xpos2 = state.geom_xpos[index2];
    let xmat1 = &state.geom_xmat[index1];
    let xmat2 = &state.geom_xmat[index2];
    if rbound1 == 0.0 || rbound2 == 0.0 {
        if !filter.contains(BroadphaseFilter::PLANE) {
            return true;
        }
        return plane_filter(
            rbound1, rbound2, margin1, margin2, xpos1, xpos2, xmat1, xmat2,
        );
    }
    let (center1, size1) = model.aabb(world, geom1);
    let (center2, size2) = model.aabb(world, geom2);
    if filter.contains(BroadphaseFilter::SPHERE)
        && !sphere_filter(rbound1, rbound2, margin1, margin2, xpos1, xpos2)
    {
        return false;
    }
    if filter.contains(BroadphaseFilter::AABB)
        && !aabb_filter(
            center1, center2, size1, size2, margin1, margin2, xpos1, xpos2, xmat1, xmat2,
        )
    {
        return false;
    }
    if filter.contains(BroadphaseFilter::OBB)
        && !obb_filter(
            center1, center2, size1, size2, margin1, margin2, xpos1, xpos2, xmat1, xmat2,
        )
    {
        return false;
    }
    true
}
