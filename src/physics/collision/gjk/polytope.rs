//! 上游 `collision_gjk` 的 EPA polytope 构造、面操作与 witness 恢复。

use super::math::{dot3, length3, neg3, normalize3, sub3};
use super::simplex::{det3, tri_affine_coord};
use super::support::support;
use super::{
    FACE_DELETED_BIT, FACE_INVALID_BIT, FACE_INVALID_OR_DELETED_MASK, Geom, GjkResult, Polytope,
    geom_type,
};

/// 上游 `_get_face_verts`：三顶点索引（每 10 bit）。
#[inline]
pub(crate) fn get_face_verts(face: u32) -> [usize; 3] {
    [
        (face & 0x3FF) as usize,
        ((face >> 10) & 0x3FF) as usize,
        ((face >> 20) & 0x3FF) as usize,
    ]
}

/// 上游 `_delete_face`。
#[inline]
pub(crate) fn delete_face(face: u32) -> u32 {
    face | FACE_DELETED_BIT
}

/// 上游 `_is_face_deleted`。
#[inline]
pub(crate) fn is_face_deleted(face: u32) -> bool {
    face & FACE_DELETED_BIT != 0
}

/// 上游 `_invalidate_face`。
#[inline]
pub(crate) fn invalidate_face(face: u32) -> u32 {
    face | FACE_INVALID_BIT
}

/// 上游 `_is_invalid_face`：invalid 或 deleted。
#[inline]
pub(crate) fn is_invalid_face(face: u32) -> bool {
    face & FACE_INVALID_OR_DELETED_MASK != 0
}

/// 上游 `_attach_face`：加入三角形面并返回原点投影距离平方（0 表示失败/退化）。
pub(crate) fn attach_face(pt: &mut Polytope, idx: usize, v1: usize, v2: usize, v3: usize) -> f32 {
    // 面缓冲用尽：返回 0 让 EPA 提前退出（无接触）。
    if pt.nface == pt.face.len() {
        return 0.0;
    }

    let p1 = sub3(pt.vert[2 * v1], pt.vert[2 * v1 + 1]);
    let p2 = sub3(pt.vert[2 * v2], pt.vert[2 * v2 + 1]);
    let p3 = sub3(pt.vert[2 * v3], pt.vert[2 * v3 + 1]);
    let (mut r, ret) = super::simplex::project_origin_plane(p3, p2, p1);
    if ret {
        return 0.0;
    }

    // 投影要朝 polytope 外侧。
    if dot3(r, sub3(p1, pt.center)) < 0.0 {
        r = neg3(r);
    }

    pt.face[idx] = (v1 as u32) + ((v2 as u32) << 10) + ((v3 as u32) << 20);
    pt.face_pr[idx] = r;
    // 对齐 pinned ccd PTX 的 FMA 收缩：dot(r,r) 为「首项乘积 + 逐项 mul_add」链。
    let head = r[0] * r[0];
    let tail = r[1].mul_add(r[1], head);
    pt.face_norm2[idx] = r[2].mul_add(r[2], tail);
    pt.face_norm2[idx]
}

/// 上游 `_epa_support`：向两个 geom 取支撑点写入 polytope 的第 `idx` 对顶点。
pub(crate) fn epa_support(
    pt: &mut Polytope,
    idx: usize,
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    dir: [f32; 3],
) -> (i32, i32) {
    let sp = support(geom1, geomtype1, dir);
    pt.vert[2 * idx] = sp.point;
    pt.vert_index[2 * idx] = sp.vertex_index;
    let index1 = sp.cached_index;

    let sp = support(geom2, geomtype2, neg3(dir));
    pt.vert[2 * idx + 1] = sp.point;
    pt.vert_index[2 * idx + 1] = sp.vertex_index;
    let index2 = sp.cached_index;
    (index1, index2)
}

/// 上游 `_rotmat`：绕轴旋转 120° 的矩阵。
pub(crate) fn rotmat(axis: [f32; 3]) -> [f32; 9] {
    let n = length3(axis);
    let u1 = axis[0] / n;
    let u2 = axis[1] / n;
    let u3 = axis[2] / n;

    let sin = 0.866_025_403_78_f32;
    let cos = -0.5_f32;
    let one_minus = 1.0 - cos;
    [
        cos + u1 * u1 * one_minus,
        u1 * u2 * one_minus - u3 * sin,
        u1 * u3 * one_minus + u2 * sin,
        u2 * u1 * one_minus + u3 * sin,
        cos + u2 * u2 * one_minus,
        u2 * u3 * one_minus - u1 * sin,
        u1 * u3 * one_minus - u2 * sin,
        u2 * u3 * one_minus + u1 * sin,
        cos + u3 * u3 * one_minus,
    ]
}

/// 上游 `_ray_triangle`。
pub(crate) fn ray_triangle(
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    v4: [f32; 3],
    v5: [f32; 3],
) -> i32 {
    let vol1 = det3(sub3(v3, v1), sub3(v4, v1), sub3(v2, v1));
    let vol2 = det3(sub3(v4, v1), sub3(v5, v1), sub3(v2, v1));
    let vol3 = det3(sub3(v5, v1), sub3(v3, v1), sub3(v2, v1));

    if vol1 >= 0.0 && vol2 >= 0.0 && vol3 >= 0.0 {
        return 1;
    }
    if vol1 <= 0.0 && vol2 <= 0.0 && vol3 <= 0.0 {
        return -1;
    }
    0
}

/// 上游 `_replace_simplex3`：用 polytope 的 3 顶点重置 GJK 结果。
pub(crate) fn replace_simplex3(pt: &Polytope, v1: usize, v2: usize, v3: usize) -> GjkResult {
    let mut result = GjkResult::default();
    let mut simplex1 = [[0.0_f32; 3]; 4];
    simplex1[0] = pt.vert[2 * v1];
    simplex1[1] = pt.vert[2 * v2];
    simplex1[2] = pt.vert[2 * v3];
    let mut simplex2 = [[0.0_f32; 3]; 4];
    simplex2[0] = pt.vert[2 * v1 + 1];
    simplex2[1] = pt.vert[2 * v2 + 1];
    simplex2[2] = pt.vert[2 * v3 + 1];
    let mut simplex = [[0.0_f32; 3]; 4];
    simplex[0] = sub3(simplex1[0], simplex2[0]);
    simplex[1] = sub3(simplex1[1], simplex2[1]);
    simplex[2] = sub3(simplex1[2], simplex2[2]);

    let mut simplex_index1 = [0_i32; 4];
    simplex_index1[0] = pt.vert_index[2 * v1];
    simplex_index1[1] = pt.vert_index[2 * v2];
    simplex_index1[2] = pt.vert_index[2 * v3];
    let mut simplex_index2 = [0_i32; 4];
    simplex_index2[0] = pt.vert_index[2 * v1 + 1];
    simplex_index2[1] = pt.vert_index[2 * v2 + 1];
    simplex_index2[2] = pt.vert_index[2 * v3 + 1];

    result.simplex = simplex;
    result.simplex1 = simplex1;
    result.simplex2 = simplex2;
    result.simplex_index1 = simplex_index1;
    result.simplex_index2 = simplex_index2;
    result
}

/// 上游 `_epa_witness`：从选中面恢复接触 witness 点与穿透距离。
pub(crate) fn epa_witness(
    pt: &Polytope,
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    face_idx: usize,
) -> ([f32; 3], [f32; 3], f32) {
    let face = get_face_verts(pt.face[face_idx]);
    // 面上的重心坐标。
    let v1 = sub3(pt.vert[2 * face[0]], pt.vert[2 * face[0] + 1]);
    let v2 = sub3(pt.vert[2 * face[1]], pt.vert[2 * face[1] + 1]);
    let v3 = sub3(pt.vert[2 * face[2]], pt.vert[2 * face[2] + 1]);
    let coordinates = tri_affine_coord(v1, v2, v3, pt.face_pr[face_idx]);
    let (l1, l2, l3) = (coordinates[0], coordinates[1], coordinates[2]);

    // geom2 侧的面。
    let v1 = pt.vert[2 * face[0] + 1];
    let v2 = pt.vert[2 * face[1] + 1];
    let v3 = pt.vert[2 * face[2] + 1];
    let x2 = [
        v1[0] * l1 + v2[0] * l2 + v3[0] * l3,
        v1[1] * l1 + v2[1] * l2 + v3[1] * l3,
        v1[2] * l1 + v2[2] * l2 + v3[2] * l3,
    ];

    // hfield 的 witness 修正。
    let i1 = pt.vert_index[2 * face[0]];
    let i2 = pt.vert_index[2 * face[1]];
    let i3 = pt.vert_index[2 * face[2]];
    if geomtype1 == geom_type::HFIELD && (i1 != i2 || i1 != i3) {
        let n = [0.0_f32, 0.0, 1.0];
        let a = geom1.polyvert[3];
        let b = geom1.polyvert[4];
        let c = geom1.polyvert[5];

        let x2 = if geomtype2 == geom_type::CAPSULE || geomtype2 == geom_type::SPHERE {
            let radius = geom2.size[0];
            let margin = geom2.margin;
            let mut geom2 = *geom2;
            geom2.margin = 0.0;
            geom2.size = [0.0, geom2.size[1], geom2.size[2]];
            let sp = support(&geom2, geomtype2, x2);
            let offset = 0.5 * margin + radius;
            [
                sp.point[0] - offset * n[0],
                sp.point[1] - offset * n[1],
                sp.point[2] - offset * n[2],
            ]
        } else {
            let x2n = normalize3(x2);
            support(geom2, geomtype2, x2n).point
        };

        let coordinates2 = tri_affine_coord(a, b, c, x2);
        let x1 = if coordinates2[0] > 0.0 && coordinates2[1] > 0.0 && coordinates2[2] > 0.0 {
            [
                coordinates2[0] * a[0] + coordinates2[1] * b[0] + coordinates2[2] * c[0],
                coordinates2[0] * a[1] + coordinates2[1] * b[1] + coordinates2[2] * c[1],
                coordinates2[0] * a[2] + coordinates2[1] * b[2] + coordinates2[2] * c[2],
            ]
        } else {
            let mut p = c;
            if coordinates2[1] > 0.0 {
                p = b;
            }
            if coordinates2[0] > 0.0 {
                p = a;
            }
            let d = dot3(sub3(x2, p), n);
            [x2[0] - d * n[0], x2[1] - d * n[1], x2[2] - d * n[2]]
        };
        let dist = -length3(sub3(x1, x2));
        return (x1, x2, dist);
    }

    // geom1 侧的面。
    let v1 = pt.vert[2 * face[0]];
    let v2 = pt.vert[2 * face[1]];
    let v3 = pt.vert[2 * face[2]];
    let x1 = [
        v1[0] * l1 + v2[0] * l2 + v3[0] * l3,
        v1[1] * l1 + v2[1] * l2 + v3[1] * l3,
        v1[2] * l1 + v2[2] * l2 + v3[2] * l3,
    ];

    (x1, x2, -(pt.face_norm2[face_idx]).sqrt())
}
