//! 上游 `collision_sdf.py::_sdf_narrowphase` 的宿主语义：单 pair 的
//! AABB 交集 + Halton 初始点 + 每初始点一次梯度下降 → 一个接触候选。
//! 不含 GPU 并行与 nacon 截断（G5 负责）。

use super::super::gjk::math::{add3, make_frame, mul_mat3_vec, sub3, transpose_mul_mat3_vec};
use super::{
    SdfShape,
    descent::{gradient_descent, relative_matrix},
    octree::halton,
};

/// 局部 AABB：`(中心, 半尺寸)`。
pub type LocalAabb = ([f32; 3], [f32; 3]);

/// 一个 SDF 窄相对（geom1 为普通几何，geom2 为体积 SDF）。
#[derive(Clone, Copy, Debug)]
pub struct SdfPair<'a> {
    pub shape1: SdfShape<'a>,
    pub shape2: SdfShape<'a>,
    pub pos1: [f32; 3],
    pub rot1: [f32; 9],
    pub pos2: [f32; 3],
    pub rot2: [f32; 9],
    /// geom1 的局部 AABB。
    pub aabb1: LocalAabb,
    /// geom2 的局部 AABB（= 根 octree 中心/半尺寸）。
    pub aabb2: LocalAabb,
    /// `m.opt.sdf_initpoints`。
    pub sdf_initpoints: usize,
    /// `m.opt.sdf_iterations`。
    pub sdf_iterations: usize,
}

/// 一个接触候选（每初始点一条）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfContact {
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
}

fn identity() -> [f32; 9] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
}

/// 上游 `transform_aabb`：把局部 AABB 的 8 角点变换后取包围盒。
fn transform_aabb(center: [f32; 3], size: [f32; 3], pos: [f32; 3], ori: &[f32; 9]) -> LocalAabb {
    let mut min = [1.0e9_f32; 3];
    let mut max = [-1.0e9_f32; 3];
    for index in 0..8 {
        let corner = [
            size[0] * if index & 1 != 0 { 1.0 } else { -1.0 },
            size[1] * if index & 2 != 0 { 1.0 } else { -1.0 },
            size[2] * if index & 4 != 0 { 1.0 } else { -1.0 },
        ];
        let frame = add3(mul_mat3_vec(ori, add3(corner, center)), pos);
        for axis in 0..3 {
            min[axis] = min[axis].min(frame[axis]);
            max[axis] = max[axis].max(frame[axis]);
        }
    }
    (min, max)
}

/// 单 pair 接触候选：AABB 不相交时返回空；否则每初始点一个候选。
pub fn sdf_pair_contacts(pair: &SdfPair) -> Vec<SdfContact> {
    let (aabb1_min, aabb1_max) = transform_aabb(pair.aabb1.0, pair.aabb1.1, [0.0; 3], &identity());
    let rel_mat = relative_matrix(&pair.rot1, &pair.rot2);
    let rel_pos = transpose_mul_mat3_vec(&pair.rot1, sub3(pair.pos2, pair.pos1));
    let (aabb2_min, aabb2_max) = transform_aabb(pair.aabb2.0, pair.aabb2.1, rel_pos, &rel_mat);
    let mut min = [0.0_f32; 3];
    let mut max = [0.0_f32; 3];
    for axis in 0..3 {
        min[axis] = aabb1_min[axis].max(aabb2_min[axis]);
        max[axis] = aabb1_max[axis].min(aabb2_max[axis]);
        if min[axis] > max[axis] {
            return Vec::new();
        }
    }
    let mut contacts = Vec::with_capacity(pair.sdf_initpoints);
    for index in 0..pair.sdf_initpoints {
        let x_g2 = [
            min[0] + (max[0] - min[0]) * halton(index as u32, 2),
            min[1] + (max[1] - min[1]) * halton(index as u32, 3),
            min[2] + (max[2] - min[2]) * halton(index as u32, 5),
        ];
        let x = add3(mul_mat3_vec(&pair.rot1, x_g2), pair.pos1);
        let x0_initial = transpose_mul_mat3_vec(&pair.rot2, sub3(x, pair.pos2));
        let (dist, pos, normal) = gradient_descent(
            &pair.shape1,
            &pair.shape2,
            x0_initial,
            pair.pos1,
            pair.rot1,
            pair.pos2,
            pair.rot2,
            pair.sdf_iterations,
        );
        contacts.push(SdfContact {
            dist,
            pos,
            frame: make_frame(normal),
        });
    }
    contacts
}
