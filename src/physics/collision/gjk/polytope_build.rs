//! 上游 `collision_gjk` 的 EPA polytope 初始构造：`_polytope2/_polytope3/_polytope4`。

use super::math::{add3, cross3, length3, mul_mat3_vec, neg3, sub3};
use super::polytope::{attach_face, epa_support, ray_triangle, replace_simplex3, rotmat};
use super::simplex::tri_point_intersect;
use super::{FLOAT_MAX, Geom, GjkResult, MINVAL, Polytope};
use super::{MIN_DIST2, MIN_DIST3, MIN_DIST4};

/// 上游 `_polytope2`：由 GJK 的 1-单纯形构造 hexahedron。
pub(crate) fn polytope2(
    mut pt: Polytope,
    simplex: &[[f32; 3]; 4],
    simplex1: &[[f32; 3]; 4],
    simplex2: &[[f32; 3]; 4],
    simplex_index1: &[i32; 4],
    simplex_index2: &[i32; 4],
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
) -> (Polytope, GjkResult) {
    let diff = sub3(simplex[1], simplex[0]);

    // polytope 中心。
    pt.center = [
        0.5 * (simplex[0][0] + simplex[1][0]),
        0.5 * (simplex[0][1] + simplex[1][1]),
        0.5 * (simplex[0][2] + simplex[1][2]),
    ];

    // 找分量最小（叉积最大）的轴。
    let mut value = FLOAT_MAX;
    let mut index = 0_usize;
    for i in 0..3 {
        if diff[i].abs() < value {
            value = diff[i].abs();
            index = i;
        }
    }
    let mut e = [0.0_f32; 3];
    e[index] = 1.0;
    let d1 = cross3(e, diff);

    // 绕线段旋转 120° 得到另外三个点。
    let r = rotmat(diff);
    let d2 = mul_mat3_vec(&r, d1);
    let d3 = mul_mat3_vec(&r, d2);

    pt.vert[0] = simplex1[0];
    pt.vert[1] = simplex2[0];
    pt.vert[2] = simplex1[1];
    pt.vert[3] = simplex2[1];
    pt.vert_index[0] = simplex_index1[0];
    pt.vert_index[1] = simplex_index2[0];
    pt.vert_index[2] = simplex_index1[1];
    pt.vert_index[3] = simplex_index2[1];

    let n1 = length3(d1);
    epa_support(
        &mut pt,
        2,
        geom1,
        geom2,
        geomtype1,
        geomtype2,
        [d1[0] / n1, d1[1] / n1, d1[2] / n1],
    );
    let n2 = length3(d2);
    epa_support(
        &mut pt,
        3,
        geom1,
        geom2,
        geomtype1,
        geomtype2,
        [d2[0] / n2, d2[1] / n2, d2[2] / n2],
    );
    let n3 = length3(d3);
    epa_support(
        &mut pt,
        4,
        geom1,
        geom2,
        geomtype1,
        geomtype2,
        [d3[0] / n3, d3[1] / n3, d3[2] / n3],
    );

    // 构造 hexahedron。
    if attach_face(&mut pt, 0, 0, 2, 3) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 2, 3);
            (pt, next)
        };
    }
    if attach_face(&mut pt, 1, 0, 4, 2) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 4, 2);
            (pt, next)
        };
    }
    if attach_face(&mut pt, 2, 0, 3, 4) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 3, 4);
            (pt, next)
        };
    }
    if attach_face(&mut pt, 3, 1, 3, 2) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 1, 3, 2);
            (pt, next)
        };
    }
    if attach_face(&mut pt, 4, 1, 2, 4) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 1, 2, 4);
            (pt, next)
        };
    }
    if attach_face(&mut pt, 5, 1, 4, 3) < MIN_DIST2 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 1, 4, 3);
            (pt, next)
        };
    }

    // 检查 hexahedron 凸性。
    let v2 = sub3(pt.vert[4], pt.vert[5]);
    let v3 = sub3(pt.vert[6], pt.vert[7]);
    let v4 = sub3(pt.vert[8], pt.vert[9]);
    if ray_triangle(simplex[0], simplex[1], v2, v3, v4) == 0 {
        pt.status = 1;
        return (pt, GjkResult::default());
    }

    pt.nvert = 5;
    pt.nface = 6;
    pt.status = 0;
    (pt, GjkResult::default())
}

/// 上游 `_polytope3`：由 GJK 的 2-单纯形构造 hexahedron。
pub(crate) fn polytope3(
    mut pt: Polytope,
    dist: f32,
    simplex: &[[f32; 3]; 4],
    simplex1: &[[f32; 3]; 4],
    simplex2: &[[f32; 3]; 4],
    simplex_index1: &[i32; 4],
    simplex_index2: &[i32; 4],
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
) -> Polytope {
    let sum1 = add3(add3(simplex[0], simplex[1]), simplex[2]);
    pt.center = [
        sum1[0] * (1.0_f32 / 3.0),
        sum1[1] * (1.0_f32 / 3.0),
        sum1[2] * (1.0_f32 / 3.0),
    ];

    // 双向法线。
    let n = cross3(sub3(simplex[1], simplex[0]), sub3(simplex[2], simplex[0]));
    let norm = length3(n);
    if norm < MINVAL {
        pt.status = 2;
        return pt;
    }
    let n = [n[0] / norm, n[1] / norm, n[2] / norm];

    pt.vert[0] = simplex1[0];
    pt.vert[1] = simplex2[0];
    pt.vert[2] = simplex1[1];
    pt.vert[3] = simplex2[1];
    pt.vert[4] = simplex1[2];
    pt.vert[5] = simplex2[2];
    pt.vert_index[0] = simplex_index1[0];
    pt.vert_index[1] = simplex_index2[0];
    pt.vert_index[2] = simplex_index1[1];
    pt.vert_index[3] = simplex_index2[1];
    pt.vert_index[4] = simplex_index1[2];
    pt.vert_index[5] = simplex_index2[2];

    epa_support(&mut pt, 3, geom1, geom2, geomtype1, geomtype2, neg3(n));
    epa_support(&mut pt, 4, geom1, geom2, geomtype1, geomtype2, n);

    let v1 = simplex[0];
    let v2 = simplex[1];
    let v3 = simplex[2];
    let v4 = sub3(pt.vert[6], pt.vert[7]);
    let v5 = sub3(pt.vert[8], pt.vert[9]);

    if tri_point_intersect(v1, v2, v3, v4) {
        pt.status = 3;
        return pt;
    }
    if tri_point_intersect(v1, v2, v3, v5) {
        pt.status = 4;
        return pt;
    }

    // 原点不在单纯形上时，检查 hexahedron 是否包含原点。
    if dist > 1e-5
        && !super::simplex::test_tetra(v1, v2, v3, v4)
        && !super::simplex::test_tetra(v1, v2, v3, v5)
    {
        pt.status = 5;
        return pt;
    }

    if attach_face(&mut pt, 0, 4, 0, 1) < MIN_DIST3 {
        pt.status = 6;
        return pt;
    }
    if attach_face(&mut pt, 1, 4, 2, 0) < MIN_DIST3 {
        pt.status = 7;
        return pt;
    }
    if attach_face(&mut pt, 2, 4, 1, 2) < MIN_DIST3 {
        pt.status = 8;
        return pt;
    }
    if attach_face(&mut pt, 3, 3, 1, 0) < MIN_DIST3 {
        pt.status = 9;
        return pt;
    }
    if attach_face(&mut pt, 4, 3, 0, 2) < MIN_DIST3 {
        pt.status = 10;
        return pt;
    }
    if attach_face(&mut pt, 5, 3, 2, 1) < MIN_DIST3 {
        pt.status = 11;
        return pt;
    }

    pt.nvert = 5;
    pt.nface = 6;
    pt.status = 0;
    pt
}

/// 上游 `_polytope4`：由 GJK 的 3-单纯形构造 tetrahedron。
pub(crate) fn polytope4(
    mut pt: Polytope,
    simplex: &[[f32; 3]; 4],
    simplex1: &[[f32; 3]; 4],
    simplex2: &[[f32; 3]; 4],
    simplex_index1: &[i32; 4],
    simplex_index2: &[i32; 4],
) -> (Polytope, GjkResult) {
    let sum = add3(add3(simplex[0], simplex[1]), add3(simplex[2], simplex[3]));
    pt.center = [0.25 * sum[0], 0.25 * sum[1], 0.25 * sum[2]];

    for i in 0..4 {
        pt.vert[2 * i] = simplex1[i];
        pt.vert[2 * i + 1] = simplex2[i];
        pt.vert_index[2 * i] = simplex_index1[i];
        pt.vert_index[2 * i + 1] = simplex_index2[i];
    }

    let mut dist = [0.0_f32; 4];
    #[allow(unused_assignments)]
    let mut idx = 0_usize;

    // 原点在某面上时用 2-单纯形替换 3-单纯形。
    dist[0] = attach_face(&mut pt, 0, 0, 1, 2);
    if dist[0] < MIN_DIST4 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 1, 2);
            (pt, next)
        };
    }
    dist[1] = attach_face(&mut pt, 1, 0, 3, 1);
    if dist[1] < MIN_DIST4 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 3, 1);
            (pt, next)
        };
    }
    idx = if dist[0] < dist[1] { 0 } else { 1 };

    dist[2] = attach_face(&mut pt, 2, 0, 2, 3);
    if dist[2] < MIN_DIST4 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 0, 2, 3);
            (pt, next)
        };
    }
    if dist[2] < dist[idx] {
        idx = 2;
    }

    dist[3] = attach_face(&mut pt, 3, 3, 2, 1);
    if dist[3] < MIN_DIST4 {
        pt.status = -1;
        return {
            let next = replace_simplex3(&pt, 3, 2, 1);
            (pt, next)
        };
    }
    if dist[3] < dist[idx] {
        idx = 3;
    }

    if !super::simplex::test_tetra(simplex[0], simplex[1], simplex[2], simplex[3]) {
        if dist[idx] > MINVAL {
            pt.status = 12;
            return (pt, GjkResult::default());
        }
        // 回退到最近面。
        pt.status = -1;
        return match idx {
            0 => {
                let next = replace_simplex3(&pt, 0, 1, 2);
                (pt, next)
            }
            1 => {
                let next = replace_simplex3(&pt, 0, 3, 1);
                (pt, next)
            }
            2 => {
                let next = replace_simplex3(&pt, 0, 2, 3);
                (pt, next)
            }
            _ => {
                let next = replace_simplex3(&pt, 3, 2, 1);
                (pt, next)
            }
        };
    }

    pt.nvert = 4;
    pt.nface = 4;
    pt.status = 0;
    (pt, GjkResult::default())
}
