//! 上游 `collision_gjk.support()`：各 geom 类型的支撑点。
//!
//! 末尾统一对 `margin > 0` 追加 `dir * (0.5 * margin)`。

use super::geom_type;
use super::math::{
    add3, cw_mul3, dot3, mul_mat3_vec, neg3, normalize3, scale3, sign3, transpose_mul_mat3_vec,
};
use super::{FLOAT_MIN, Geom, MINVAL, SupportPoint};

/// 上游 `support(geom, geomtype, dir)`。
pub fn support(geom: &Geom<'_>, geomtype: i32, dir: [f32; 3]) -> SupportPoint {
    let mut point = SupportPoint::default();

    if geomtype == geom_type::SPHERE {
        point.point = add3(geom.pos, scale3(dir, geom.size[0] + 0.5 * geom.margin));
        return point;
    }

    let local_dir = transpose_mul_mat3_vec(&geom.rot, dir);

    if geomtype == geom_type::BOX {
        let tmp = sign3(local_dir);
        let res = cw_mul3(tmp, geom.size);
        point.point = add3(mul_mat3_vec(&geom.rot, res), geom.pos);
        let mut vertex_index = 0;
        if tmp[0] > 0.0 {
            vertex_index += 1;
        }
        if tmp[1] > 0.0 {
            vertex_index += 2;
        }
        if tmp[2] > 0.0 {
            vertex_index += 4;
        }
        point.vertex_index = vertex_index;
    } else if geomtype == geom_type::CAPSULE {
        let mut res = scale3(local_dir, geom.size[0]);
        // 圆柱贡献：z 方向按符号取半长。
        res[2] += if local_dir[2] < 0.0 {
            -geom.size[1]
        } else {
            geom.size[1]
        };
        point.point = add3(mul_mat3_vec(&geom.rot, res), geom.pos);
    } else if geomtype == geom_type::ELLIPSOID {
        let mut res = cw_mul3(local_dir, geom.size);
        res = normalize3(res);
        res = cw_mul3(res, geom.size);
        point.point = add3(mul_mat3_vec(&geom.rot, res), geom.pos);
    } else if geomtype == geom_type::CYLINDER {
        let mut res = [0.0_f32; 3];
        let d = (local_dir[0] * local_dir[0] + local_dir[1] * local_dir[1]).sqrt();
        if d > MINVAL {
            let scl = geom.size[0] / d;
            res[0] = local_dir[0] * scl;
            res[1] = local_dir[1] * scl;
        }
        let top = local_dir[2] >= 0.0;
        res[2] = if top { geom.size[1] } else { -geom.size[1] };
        point.vertex_index = if top { 0 } else { 1 };
        point.point = add3(mul_mat3_vec(&geom.rot, res), geom.pos);
    } else if geomtype == geom_type::MESH {
        support_mesh(geom, local_dir, &mut point);
    } else if geomtype == geom_type::HFIELD {
        point.vertex_index = if dir[2] < 0.0 { -2 } else { -3 };
        let mut max_dist = FLOAT_MIN;
        for vert in geom.polyvert.iter().take(6) {
            let dist = dot3(*vert, dir);
            if dist > max_dist {
                max_dist = dist;
                point.point = *vert;
            }
        }
    } else if geomtype == geom_type::TRIANGLE {
        let t1 = [geom.rot[0], geom.rot[1], geom.rot[2]];
        let t2 = [geom.rot[3], geom.rot[4], geom.rot[5]];
        let t3 = [geom.rot[6], geom.rot[7], geom.rot[8]];
        let d1 = dot3(t1, dir);
        let d2 = dot3(t2, dir);
        let d3 = dot3(t3, dir);
        point.point = if d1 > d2 && d1 > d3 {
            t1
        } else if d2 > d3 {
            t2
        } else {
            t3
        };
    } else if geomtype == geom_type::FLEX {
        let p0 = geom.polyvert[0];
        let p1 = geom.polyvert[1];
        let p2 = geom.polyvert[2];
        let p3 = geom.polyvert[3];
        let d0 = dot3(p0, dir);
        let d1 = dot3(p1, dir);
        let d2 = dot3(p2, dir);
        let d3 = dot3(p3, dir);
        point.point = if d0 > d1 && d0 > d2 && d0 > d3 {
            p0
        } else if d1 > d2 && d1 > d3 {
            p1
        } else if d2 > d3 {
            p2
        } else {
            p3
        };
    }

    if geom.margin > 0.0 {
        point.point = add3(point.point, scale3(dir, 0.5 * geom.margin));
    }
    point
}

/// 上游 MESH 支撑：无图或顶点数小于 10 时穷举，否则在图上 hillclimb。
fn support_mesh(geom: &Geom<'_>, local_dir: [f32; 3], point: &mut SupportPoint) {
    let Some(mesh) = geom.mesh else {
        return;
    };
    let vertadr = geom.vertadr.max(0) as usize;
    let vertnum = geom.vertnum.max(0) as usize;
    let mut max_dist = FLOAT_MIN;

    if geom.graphadr == -1 || geom.vertnum < 10 {
        if geom.index > -1 {
            let index = geom.index as usize;
            point.cached_index = geom.index;
            max_dist = dot3(mesh.vert[index], local_dir);
            point.point = mesh.vert[index];
        }
        for i in 0..vertnum {
            let vert = mesh.vert[vertadr + i];
            let dist = dot3(vert, local_dir);
            if dist > max_dist {
                max_dist = dist;
                point.point = vert;
                point.cached_index = (vertadr + i) as i32;
            }
        }
        point.vertex_index = point.cached_index - geom.vertadr;
    } else {
        let graphadr = geom.graphadr.max(0) as usize;
        let numvert = mesh.graph[graphadr] as usize;
        let vert_edgeadr = graphadr + 2;
        let vert_globalid = graphadr + 2 + numvert;
        let edge_localid = graphadr + 2 + 2 * numvert;
        let mut prev = -1_i32;
        let mut imax = if geom.index > -1 {
            geom.index as usize
        } else {
            0
        };
        max_dist = dot3(
            local_dir,
            mesh.vert[vertadr + mesh.graph[vert_globalid + imax] as usize],
        );

        // hillclimb until no change
        while imax as i32 != prev {
            prev = imax as i32;
            let mut i = mesh.graph[vert_edgeadr + imax] as usize;
            let mut subidx = mesh.graph[edge_localid + i];
            while subidx >= 0 {
                let idx = mesh.graph[vert_globalid + subidx as usize] as usize;
                let dist = dot3(local_dir, mesh.vert[vertadr + idx]);
                if dist > max_dist {
                    imax = subidx as usize;
                    max_dist = dist;
                }
                i += 1;
                subidx = mesh.graph[edge_localid + i];
            }
        }

        point.cached_index = imax as i32;
        point.vertex_index = mesh.graph[vert_globalid + imax];
        point.point = mesh.vert[vertadr + point.vertex_index as usize];
    }

    point.point = add3(mul_mat3_vec(&geom.rot, point.point), geom.pos);
}

/// 上游 `_epa_support` 需要的 geom1/geom2 顶点对；这里只返回两点与缓存索引。
pub(crate) fn support_pair(
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    dir: [f32; 3],
) -> (SupportPoint, SupportPoint) {
    (
        support(geom1, geomtype1, dir),
        support(geom2, geomtype2, neg3(dir)),
    )
}

/// 上游 `_discrete_geoms`。
pub(crate) fn discrete_geoms(type1: i32, type2: i32) -> bool {
    let discrete = |value: i32| {
        value == geom_type::MESH || value == geom_type::BOX || value == geom_type::HFIELD
    };
    discrete(type1) && discrete(type2)
}
