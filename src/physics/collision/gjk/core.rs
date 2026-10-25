//! 上游 `collision_gjk.gjk()` 与 `_gjk_support()`。

use super::math::{cross3, dot3, neg3, scale3, sub3};
use super::support::{discrete_geoms, support};
use super::{FLOAT_MAX, Geom, GjkResult, MINVAL, MINVAL2, SupportPoint};

/// 上游 `_gjk_support`：按当前 `x_k` 取两侧支撑点，离散几何做方向调优。
fn gjk_support(
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    x_k: [f32; 3],
    x_norm: f32,
    simplex: &[[f32; 3]; 4],
    n: usize,
    is_discrete: bool,
) -> (SupportPoint, SupportPoint) {
    let mut dir_neg = [x_k[0] / x_norm, x_k[1] / x_norm, x_k[2] / x_norm];

    // 离散几何在方向噪声大时做调优。
    if is_discrete && x_norm < 1e-4 {
        if n == 2 {
            let edge = sub3(simplex[1], simplex[0]);
            let edge_norm2 = dot3(edge, edge);
            if edge_norm2 > MINVAL2 {
                let proj = dot3(dir_neg, edge) / edge_norm2;
                dir_neg = sub3(dir_neg, scale3(edge, proj));
                let dir_norm = super::math::length3(dir_neg);
                if dir_norm > MINVAL {
                    dir_neg = [
                        dir_neg[0] / dir_norm,
                        dir_neg[1] / dir_norm,
                        dir_neg[2] / dir_norm,
                    ];
                }
            }
        } else if n == 3 {
            let e1 = sub3(simplex[1], simplex[0]);
            let e2 = sub3(simplex[2], simplex[0]);
            let normal = cross3(e1, e2);
            let normal_norm = super::math::length3(normal);
            if normal_norm > MINVAL {
                let sign = if dot3(dir_neg, normal) < 0.0 {
                    -1.0_f32
                } else {
                    1.0
                };
                dir_neg = [
                    sign * normal[0] / normal_norm,
                    sign * normal[1] / normal_norm,
                    sign * normal[2] / normal_norm,
                ];
            }
        }
    }

    (
        support(geom1, geomtype1, neg3(dir_neg)),
        support(geom2, geomtype2, dir_neg),
    )
}

/// 上游 `gjk()`：求两凸几何的距离，返回 GJK 结果。
pub(crate) fn gjk(
    tolerance: f32,
    gjk_iterations: usize,
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    x1_0: [f32; 3],
    x2_0: [f32; 3],
    geomtype1: i32,
    geomtype2: i32,
    cutoff: f32,
    is_discrete: bool,
) -> GjkResult {
    let mut geom1 = *geom1;
    let mut geom2 = *geom2;
    let mut simplex = [[0.0_f32; 3]; 4];
    let mut simplex1 = [[0.0_f32; 3]; 4];
    let mut simplex2 = [[0.0_f32; 3]; 4];
    let mut simplex_index1 = [0_i32; 4];
    let mut simplex_index2 = [0_i32; 4];
    let mut n = 0_usize;
    let mut lmbda = [1.0_f32, 0.0, 0.0, 0.0];

    // 离散几何的 GJK 保证有限步收敛，忽略容差。
    let epsilon = if is_discrete {
        0.0
    } else {
        0.5 * tolerance * tolerance
    };
    let min_norm = if is_discrete { MINVAL } else { tolerance };

    let mut x_k = sub3(x1_0, x2_0);
    let mut xnorm = (dot3(x_k, x_k)).sqrt();
    let mut xnorm_prev = 0.0_f32;

    for _ in 0..gjk_iterations {
        if xnorm < min_norm || (xnorm_prev - xnorm).abs() < MINVAL {
            break;
        }

        let (sp1, sp2) = gjk_support(
            &geom1,
            &geom2,
            geomtype1,
            geomtype2,
            x_k,
            xnorm,
            &simplex,
            n,
            is_discrete,
        );
        simplex1[n] = sp1.point;
        geom1.index = sp1.cached_index;
        simplex_index1[n] = sp1.vertex_index;

        simplex2[n] = sp2.point;
        geom2.index = sp2.cached_index;
        simplex_index2[n] = sp2.vertex_index;

        simplex[n] = sub3(simplex1[n], simplex2[n]);

        // Frank-Wolfe 对偶间隙停止判据。
        if dot3(x_k, sub3(x_k, simplex[n])) < epsilon {
            break;
        }

        // 距离下界：lower > 0 表示分离。
        let lower = dot3(x_k, simplex[n]);
        if cutoff == 0.0 {
            if lower > 0.0 {
                let mut result = GjkResult::default();
                result.separated = true;
                result.dim = 0;
                result.dist = FLOAT_MAX;
                result.index1 = geom1.index;
                result.index2 = geom2.index;
                return result;
            }
        } else if cutoff < FLOAT_MAX && lower > 0.0 && lower >= cutoff * xnorm {
            let mut result = GjkResult::default();
            result.separated = true;
            result.dim = 0;
            result.dist = FLOAT_MAX;
            result.index1 = geom1.index;
            result.index2 = geom2.index;
            return result;
        }

        // 距离子算法：simplex 中离原点最近点的重心坐标。
        lmbda = super::simplex::subdistance(n + 1, &simplex);

        // 剔除不再需要的顶点。
        n = 0;
        for i in 0..4 {
            if lmbda[i] == 0.0 {
                continue;
            }
            simplex[n] = simplex[i];
            simplex1[n] = simplex1[i];
            simplex2[n] = simplex2[i];
            simplex_index1[n] = simplex_index1[i];
            simplex_index2[n] = simplex_index2[i];
            lmbda[n] = lmbda[i];
            n += 1;
        }

        // 不应发生。
        if n < 1 {
            break;
        }

        x_k = super::simplex::linear_combine(n, lmbda, &simplex);
        xnorm_prev = xnorm;
        xnorm = (dot3(x_k, x_k)).sqrt();

        // 四面体包含原点，提前返回。
        if n == 4 {
            break;
        }
    }

    let mut result = GjkResult::default();
    result.separated = false;

    // 近似 witness 点；n 为 0 时初始点即 witness。
    if n == 0 {
        result.x1 = x1_0;
        result.x2 = x2_0;
    } else {
        result.x1 = super::simplex::linear_combine(n, lmbda, &simplex1);
        result.x2 = super::simplex::linear_combine(n, lmbda, &simplex2);
    }

    if xnorm > 0.0 {
        let dir = [x_k[0] / xnorm, x_k[1] / xnorm, x_k[2] / xnorm];
        let sp1 = support(&geom1, geomtype1, neg3(dir));
        let sp2 = support(&geom2, geomtype2, dir);
        result.separated = dot3(x_k, sub3(sp1.point, sp2.point)) > 0.0;
    }

    // 3-单纯形且未分离：原点包含在单纯形中。
    result.dist = if n == 4 && !result.separated {
        0.0
    } else {
        xnorm
    };
    result.dim = n;
    result.simplex1 = simplex1;
    result.simplex2 = simplex2;
    result.simplex_index1 = simplex_index1;
    result.simplex_index2 = simplex_index2;
    result.simplex = simplex;
    result.index1 = geom1.index;
    result.index2 = geom2.index;
    result
}

/// 上游 `_gjk_support` 的离散几何判定（供 `phase` 复用）。
pub(crate) fn is_discrete(geomtype1: i32, geomtype2: i32, margin1: f32, margin2: f32) -> bool {
    discrete_geoms(geomtype1, geomtype2) && margin1 == 0.0 && margin2 == 0.0
}
