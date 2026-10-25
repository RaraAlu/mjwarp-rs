//! 上游 `collision_gjk` 的多接触（multicontact）辅助：feature 维度、法线匹配与多边形取点。

use super::super::math::{dot3, length3, normalize3, sub3};
use super::super::{EDGE_TOL, FACE_TOL, MINVAL};

/// 上游 `_feature_dim`：返回 (维度, 顶点索引三元组, 顶点三元组矩阵行)。
pub(crate) fn feature_dim(
    face: [usize; 3],
    vert_index: &[i32],
    vert: &[[f32; 3]],
    offset: usize,
) -> (usize, [i32; 3], [[f32; 3]; 3]) {
    let v1i = vert_index[2 * face[0] + offset];
    let v2i = vert_index[2 * face[1] + offset];
    let v3i = vert_index[2 * face[2] + offset];

    let feature_index = [v1i, v2i, v3i];
    let feature_vertex = [
        vert[2 * face[0] + offset],
        vert[2 * face[1] + offset],
        vert[2 * face[2] + offset],
    ];

    if v1i != v2i {
        let dim = if v3i == v1i || v3i == v2i { 2 } else { 3 };
        return (dim, feature_index, feature_vertex);
    }

    let mut feature_index = feature_index;
    let mut feature_vertex = feature_vertex;
    feature_index[1] = v3i;
    feature_vertex[1] = vert[2 * face[2] + offset];
    let dim = if v1i != v3i { 2 } else { 1 };
    (dim, feature_index, feature_vertex)
}

/// 上游 `_aligned_faces`：找一对相互朝向（点积 < -FACE_TOL）的法线。
pub(crate) fn aligned_faces(
    vert1: &[[f32; 3]],
    len1: usize,
    vert2: &[[f32; 3]],
    len2: usize,
) -> (bool, [usize; 2]) {
    for i in 0..len1 {
        for j in 0..len2 {
            if dot3(vert1[i], vert2[j]) < -FACE_TOL {
                return (true, [i, j]);
            }
        }
    }
    (false, [0, 0])
}

/// 上游 `_aligned_face_edge`：找一条与面垂直（`|dot| < EDGE_TOL`）的边。
pub(crate) fn aligned_face_edge(
    edge: &[[f32; 3]],
    nedge: usize,
    face: &[[f32; 3]],
    nface: usize,
    dir: [f32; 3],
) -> (bool, [usize; 2]) {
    for i in 0..nface {
        // 忽略背向碰撞方向的面（负点积）。
        if dot3(face[i], dir) <= MINVAL {
            continue;
        }
        for j in 0..nedge {
            if dot3(edge[j], face[i]).abs() < EDGE_TOL {
                return (true, [j, i]);
            }
        }
    }
    (false, [0, 0])
}

/// 上游 `_intersect1`：两段整数数组的最多 2 个公共值。
pub(crate) fn intersect1(
    a1: &[i32],
    a2: &[i32],
    start1: usize,
    start2: usize,
    len1: usize,
    len2: usize,
) -> (usize, [i32; 2]) {
    let mut count = 0;
    let mut res = [0_i32; 2];
    for i in start1..start1 + len1 {
        for j in start2..start2 + len2 {
            if a1[i] == a2[j] {
                res[count] = a1[i];
                count += 1;
                if count == 2 {
                    return (2, res);
                }
            }
        }
    }
    (count, res)
}

/// 上游 `_intersect2`：2 元组与数组的最多 2 个公共值。
pub(crate) fn intersect2(
    a1: &[i32; 2],
    a2: &[i32],
    start2: usize,
    len1: usize,
    len2: usize,
) -> (usize, [i32; 2]) {
    let mut count = 0;
    let mut res = [0_i32; 2];
    for i in 0..len1 {
        for j in start2..start2 + len2 {
            if a1[i] == a2[j] {
                res[count] = a1[i];
                count += 1;
                if count == 2 {
                    return (2, res);
                }
            }
        }
    }
    (count, res)
}

/// 上游 `_set_edge`：由 EPA 顶点与 endvert 组成 2 点“面”。
pub(crate) fn set_edge(
    vert1: &[[f32; 3]],
    vert2: &[[f32; 3]],
    start: usize,
    end: usize,
    offset: usize,
) -> Vec<[f32; 3]> {
    vec![vert1[2 * start + offset], vert2[end]]
}

/// 上游 `_area4`：空间四边形面积。
pub(crate) fn area4(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    0.5 * length3(super::super::math::cross3(sub3(a, c), sub3(b, d)))
}

/// 上游 `_polygon_quad`：凸多边形中面积最大的四边形索引。
pub(crate) fn polygon_quad(polygon: &[[f32; 3]], npolygon: usize) -> [usize; 4] {
    let mut b = 1_usize;
    let mut c = 2_usize;
    let mut d = 3_usize;
    let mut res = [0_usize, b, c, d];
    let mut m = area4(polygon[0], polygon[b], polygon[c], polygon[d]);

    for a in 0..npolygon {
        loop {
            let m_next = area4(
                polygon[a],
                polygon[b],
                polygon[c],
                polygon[(d + 1) % npolygon],
            );
            if m_next <= m {
                break;
            }
            m = m_next;
            d = (d + 1) % npolygon;
            res = [a, b, c, d];
            loop {
                let m_next = area4(
                    polygon[a],
                    polygon[b],
                    polygon[(c + 1) % npolygon],
                    polygon[d],
                );
                if m_next <= m {
                    break;
                }
                m = m_next;
                c = (c + 1) % npolygon;
                res = [a, b, c, d];
                loop {
                    let m_next = area4(
                        polygon[a],
                        polygon[(b + 1) % npolygon],
                        polygon[c],
                        polygon[d],
                    );
                    if m_next <= m {
                        break;
                    }
                    m = m_next;
                    b = (b + 1) % npolygon;
                    res = [a, b, c, d];
                }
            }
        }
        if b == a {
            b = (b + 1) % npolygon;
            if c == b {
                c = (c + 1) % npolygon;
                if d == c {
                    d = (d + 1) % npolygon;
                }
            }
        }
    }
    res
}

/// 上游 `_plane_normal`：返回 (平面常数, 平面法线)。
pub(crate) fn plane_normal(v1: [f32; 3], v2: [f32; 3], n: [f32; 3]) -> (f32, [f32; 3]) {
    let v3 = [v1[0] + n[0], v1[1] + n[1], v1[2] + n[2]];
    let res = super::super::math::cross3(sub3(v2, v1), sub3(v3, v1));
    (dot3(res, v1), res)
}

/// 上游 `_halfspace`。
pub(crate) fn halfspace(a: [f32; 3], n: [f32; 3], p: [f32; 3]) -> bool {
    dot3(sub3(p, a), n) > -1e-10
}

/// 上游 `_plane_intersect`：直线 `a + t(b-a)` 与平面交点参数。
pub(crate) fn plane_intersect(pn: [f32; 3], pd: f32, a: [f32; 3], b: [f32; 3]) -> f32 {
    let dot = dot3(pn, sub3(b, a));
    if dot.abs() < 1e-10 {
        return super::super::FLOAT_MAX;
    }
    (pd - dot3(pn, a)) / dot
}

/// 上游 `_witness_on_face`。
pub(crate) fn witness_on_face(
    v: [f32; 3],
    p: [f32; 3],
    n: [f32; 3],
    dir: [f32; 3],
) -> ([f32; 3], [f32; 3], f32) {
    let d = sub3(v, p);
    let dist = dot3(d, n);
    let w1 = [
        v[0] - dir[0] * dist.abs(),
        v[1] - dir[1] * dist.abs(),
        v[2] - dir[2] * dist.abs(),
    ];
    (w1, v, dist)
}

/// 上游 `_normalize_vec`（仅在多点面裁剪内部使用）。
#[allow(dead_code)]
pub(crate) fn normalized(vector: [f32; 3]) -> [f32; 3] {
    normalize3(vector)
}
