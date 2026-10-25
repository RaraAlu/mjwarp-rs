//! 上游 `collision_gjk._epa()`：EPA 迭代与 horizon 扩展。

use super::math::{dot3, sub3};
use super::polytope::{
    attach_face, delete_face, epa_witness, get_face_verts, invalidate_face, is_face_deleted,
    is_invalid_face,
};
use super::{FLOAT_MAX, Geom, MIN_EPATOL, Polytope};

/// EPA 返回值。
pub(crate) struct EpaResult {
    pub dist: f32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    /// 选中面索引；`-1` 表示失败。
    pub face_idx: i32,
    pub overflow: u32,
    /// 迭代结束后的 polytope（供 multicontact 恢复 feature）。
    pub polytope: Polytope,
}

/// 上游 `_get_edge`。
#[inline]
fn get_edge(edge: i32) -> [i32; 2] {
    [edge & 0x3FF, (edge >> 10) & 0x3FF]
}

/// 上游 `_add_edge`：加入 horizon 边；重复边被移除（与末元素交换），返回新计数。
fn add_edge(pt: &mut Polytope, e1: i32, e2: i32) -> i32 {
    let n = pt.nhorizon;
    if n < 0 {
        return -1;
    }
    let edge = (e1.min(e2) << 10) | e1.max(e2);
    let n_usize = n as usize;
    for i in 0..n_usize {
        if edge == pt.horizon[i] {
            pt.horizon[i] = pt.horizon[n_usize - 1];
            return n - 1;
        }
    }
    // 缓冲用尽：强制 EPA 提前返回（无接触）。
    if n_usize == pt.horizon.len() {
        return -1;
    }
    pt.horizon[n_usize] = edge;
    n + 1
}

/// 上游 `_epa()`：从初始 polytope 求穿透距离与 witness。
#[allow(unused_assignments)]
pub(crate) fn epa(
    mut pt: Polytope,
    tolerance: f32,
    epa_iterations: usize,
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    is_discrete: bool,
) -> EpaResult {
    let mut upper = FLOAT_MAX;
    let mut upper2 = FLOAT_MAX;
    let mut idx: i32 = -1;
    let mut pidx: i32 = -1;
    let mut overflow = 0_u32;
    let epsilon = if is_discrete { MIN_EPATOL } else { tolerance };
    let mut nvalid = pt.nface as i32;

    // 面顶点用 10 bit 编码，迭代上限同时受顶点缓冲约束。
    let epa_iterations = epa_iterations.min(1000);
    for _ in 0..epa_iterations {
        pidx = idx;
        idx = -1;
        let mut lower2 = FLOAT_MAX;

        // 找离原点最近的面（穿透深度下界）。
        for i in 0..pt.nface {
            if !is_invalid_face(pt.face[i]) && pt.face_norm2[i] < lower2 {
                idx = i as i32;
                lower2 = pt.face_norm2[i];
            }
        }

        // 无有效面：回退到上一面。
        if lower2 > upper2 || idx < 0 {
            idx = pidx;
            break;
        }

        if lower2 <= 0.0 {
            break;
        }

        // 由最近面的法线取支撑点 w。
        let lower = lower2.sqrt();
        let wi = pt.nvert;
        let face_pr = pt.face_pr[idx as usize];
        let dir = [face_pr[0] / lower, face_pr[1] / lower, face_pr[2] / lower];
        let (support_index1, support_index2) =
            super::polytope::epa_support(&mut pt, wi, geom1, geom2, geomtype1, geomtype2, dir);
        let w = sub3(pt.vert[2 * wi], pt.vert[2 * wi + 1]);
        // 上游把缓存索引写回局部 geom 拷贝；本函数内不参与后续支撑。
        let _ = (support_index1, support_index2);
        pt.nvert += 1;

        // 第 k 次迭代上界（归一化前点积，精度更好）。
        let upper_k = dot3(face_pr, w) / lower;
        if upper_k < upper {
            upper = upper_k;
            upper2 = upper * upper;
        }

        if upper - lower < epsilon {
            break;
        }

        // 检查 wi 是否为重复支撑点。
        if is_discrete {
            let mut found_repeated = false;
            for i in 0..pt.nvert - 1 {
                if pt.vert_index[2 * i] == pt.vert_index[2 * wi]
                    && pt.vert_index[2 * i + 1] == pt.vert_index[2 * wi + 1]
                {
                    found_repeated = true;
                    break;
                }
            }
            if found_repeated {
                break;
            }
        }

        nvalid -= 1;
        pt.face[idx as usize] = delete_face(pt.face[idx as usize]);
        let face = get_face_verts(pt.face[idx as usize]);
        pt.nhorizon = add_edge(&mut pt, face[0] as i32, face[1] as i32);
        pt.nhorizon = add_edge(&mut pt, face[1] as i32, face[2] as i32);
        pt.nhorizon = add_edge(&mut pt, face[2] as i32, face[0] as i32);
        if pt.nhorizon == -1 {
            overflow |= super::overflow_type::EPA_HORIZON;
            idx = -1;
            break;
        }

        // 计算 w 的 horizon。
        for i in 0..pt.nface {
            if is_face_deleted(pt.face[i]) {
                continue;
            }
            if dot3(pt.face_pr[i], w) - pt.face_norm2[i] > 1e-10 {
                if !is_invalid_face(pt.face[i]) {
                    nvalid -= 1;
                }
                pt.face[i] = delete_face(pt.face[i]);
                let face = get_face_verts(pt.face[i]);
                pt.nhorizon = add_edge(&mut pt, face[0] as i32, face[1] as i32);
                pt.nhorizon = add_edge(&mut pt, face[1] as i32, face[2] as i32);
                pt.nhorizon = add_edge(&mut pt, face[2] as i32, face[0] as i32);
                if pt.nhorizon == -1 {
                    overflow |= super::overflow_type::EPA_HORIZON;
                    idx = -1;
                    break;
                }
            }
        }

        // 插入新顶点并沿 horizon 附面。
        let mut i = 0_usize;
        while (i as i32) < pt.nhorizon {
            let edge = get_edge(pt.horizon[i]);
            let nface = pt.nface;
            let dist2 = attach_face(&mut pt, nface, wi, edge[0] as usize, edge[1] as usize);
            if dist2 == 0.0 {
                idx = -1;
                break;
            }
            pt.nface += 1;
            if dist2 >= lower2 && dist2 <= upper2 {
                nvalid += 1;
            } else {
                pt.face[pt.nface - 1] = invalidate_face(pt.face[pt.nface - 1]);
            }
            i += 1;
        }

        // 没有候选面。
        if nvalid == 0 || idx == -1 {
            break;
        }

        pt.nhorizon = 0;
    }

    if idx > -1 {
        let (x1, x2, dist) = epa_witness(&pt, geom1, geom2, geomtype1, geomtype2, idx as usize);
        return EpaResult {
            dist,
            x1,
            x2,
            face_idx: idx,
            overflow,
            polytope: pt,
        };
    }
    EpaResult {
        dist: 0.0,
        x1: [0.0; 3],
        x2: [0.0; 3],
        face_idx: -1,
        overflow,
        polytope: pt,
    }
}
