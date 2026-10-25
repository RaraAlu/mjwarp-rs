//! 上游 `collision_gjk._polygon_clip()`：面-面裁剪生成多点 witness。

use super::super::math::{add3, dot3, scale3, sub3};
use super::super::{FLOAT_MAX, INTERSECT_TOL};
use super::feature::{halfspace, plane_intersect, plane_normal, polygon_quad, witness_on_face};

/// 裁剪中间结果。
pub(crate) struct ClipResult {
    pub ncontact: usize,
    pub witness1: [[f32; 3]; 4],
    pub witness2: [[f32; 3]; 4],
    pub dists: [f32; 4],
}

/// 上游 `_polygon_clip`。
pub(crate) fn polygon_clip(
    face1: &[[f32; 3]],
    nface1: usize,
    face2: &[[f32; 3]],
    nface2: usize,
    n: [f32; 3],
    dir: [f32; 3],
) -> ClipResult {
    let mut result = ClipResult {
        ncontact: 0,
        witness1: [[0.0; 3]; 4],
        witness2: [[0.0; 3]; 4],
        dists: [0.0; 4],
    };

    // 裁剪面至少需要三角形。
    if nface1 < 3 {
        return result;
    }

    // 每个顶点的平面法线与平面常数。
    let mut pn = [[0.0_f32; 3]; 16];
    let mut pd = [0.0_f32; 16];
    for i in 0..nface1 - 1 {
        let (plane_d, plane_n) = plane_normal(face1[i], face1[i + 1], n);
        pd[i] = plane_d;
        pn[i] = plane_n;
    }
    let (plane_d, plane_n) = plane_normal(face1[nface1 - 1], face1[0], n);
    pd[nface1 - 1] = plane_d;
    pn[nface1 - 1] = plane_n;

    // 裁剪后的多边形最多 2 倍边数。
    let mut polygon = vec![[0.0_f32; 3]; 32];
    let mut clipped = vec![[0.0_f32; 3]; 32];
    let mut npolygon = nface2;
    let mut nclipped = 0_usize;

    for i in 0..nface2 {
        polygon[i] = face2[i];
    }

    // 逐边裁剪。
    for e in 0..nface1 {
        for i in 0..npolygon {
            let p = polygon[i];
            let q = polygon[(i + 1) % npolygon];

            let inside1 = halfspace(face1[e], pn[e], p);
            let inside2 = halfspace(face1[e], pn[e], q);

            // PQ 完全在外侧。
            if !inside1 && !inside2 {
                continue;
            }

            // PQ 完全在内侧，加入 Q。
            if inside1 && inside2 {
                clipped[nclipped] = q;
                nclipped += 1;
                continue;
            }

            // PQ 与裁剪边相交，加入交点。
            let t = plane_intersect(pn[e], pd[e], p, q);
            if t > -INTERSECT_TOL && t < 1.0 + INTERSECT_TOL {
                let t = t.clamp(0.0, 1.0);
                let d = sub3(q, p);
                clipped[nclipped] = add3(p, scale3(d, t));
                nclipped += 1;
            }

            // PQ 回到内侧，加入 Q。
            if inside2 {
                clipped[nclipped] = q;
                nclipped += 1;
            }
        }

        // 交换 polygon 与 clipped。
        std::mem::swap(&mut polygon, &mut clipped);
        npolygon = nclipped;
        nclipped = 0;
    }

    // 剔除相对该面有正距离的顶点。
    let m = npolygon;
    npolygon = 0;
    for i in 0..m {
        if dot3(sub3(polygon[i], face1[0]), n) <= 0.0 {
            if npolygon != i {
                polygon[npolygon] = polygon[i];
            }
            npolygon += 1;
        }
    }

    if npolygon < 1 {
        return result;
    }

    // 面是边时去掉重复点，取最长的一条。
    if nface2 == 2 && npolygon > 2 {
        let mut best1 = 0_usize;
        let mut best2 = 1_usize;
        let mut max_d = 0.0_f32;
        for i in 0..npolygon {
            for j in i + 1..npolygon {
                let diff = sub3(polygon[j], polygon[i]);
                let d2 = dot3(diff, diff);
                if d2 > max_d {
                    max_d = d2;
                    best1 = i;
                    best2 = j;
                }
            }
        }

        let (w1, w2, d) = witness_on_face(polygon[best1], face1[0], n, dir);
        result.witness1[0] = w1;
        result.witness2[0] = w2;
        result.dists[0] = d;

        let (w1, w2, d) = witness_on_face(polygon[best2], face1[0], n, dir);
        result.witness1[1] = w1;
        result.witness2[1] = w2;
        result.dists[1] = d;

        result.ncontact = 2;
        return result;
    }

    if npolygon > 4 {
        let quad = polygon_quad(&polygon, npolygon);
        for i in 0..4 {
            let (w1, w2, d) = witness_on_face(polygon[quad[i]], face1[0], n, dir);
            result.witness1[i] = w1;
            result.witness2[i] = w2;
            result.dists[i] = d;
        }
        result.ncontact = 4;
        return result;
    }

    for i in 0..npolygon {
        let (w1, w2, d) = witness_on_face(polygon[i], face1[0], n, dir);
        result.witness1[i] = w1;
        result.witness2[i] = w2;
        result.dists[i] = d;
    }
    result.ncontact = npolygon;
    result
}

/// 上游 `FLOAT_MAX` 在本模块内的可用性检查（裁剪平面参数用）。
#[allow(dead_code)]
pub(crate) fn max_parameter() -> f32 {
    FLOAT_MAX
}
