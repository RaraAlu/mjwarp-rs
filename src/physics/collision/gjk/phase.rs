//! 上游 `collision_gjk` 的 `ccd()` 两阶段驱动：`gjk_phase` / `epa_phase` / `_inflate`。

use super::core::gjk;
use super::geom_type;
use super::math::{dot3, length3, normalize3, scale3, sub3};
use super::support::support;
use super::{FLOAT_MAX, Geom, GjkResult, Polytope};

/// CCD 选项（上游 `eval_ccd_write_contact` 的 `tolerance`/`cutoff` 与迭代上限）。
#[derive(Clone, Copy, Debug)]
pub struct CcdOptions {
    /// `m.opt.ccd_tolerance`。
    pub tolerance: f32,
    /// 分离判据用的 cutoff（非 sensor 对为 pair gap；sensor 对为 `1e32`）。
    pub cutoff: f32,
    /// `m.opt.ccd_iterations`。
    pub gjk_iterations: usize,
    /// hfield/builder 侧派生值（全 box-box 对被钳到 16）。
    pub epa_iterations: usize,
}

impl Default for CcdOptions {
    fn default() -> Self {
        Self {
            tolerance: 1.0e-6,
            cutoff: 0.0,
            gjk_iterations: 35,
            epa_iterations: 16,
        }
    }
}

/// `ccd()` 的宿主侧返回。
#[derive(Clone, Copy, Debug)]
pub struct CcdResult {
    pub dist: f32,
    /// `0` = 分离、`1` = 有接触候选。
    pub ncontact: i32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    /// `-1` 表示不支持（或 EPA 失败）。
    pub multiccd_idx: i32,
    /// 上游 `OverflowType` 位（`CCD`/`EPA_HORIZON`）。
    pub overflow: u32,
}

/// `gjk_phase` 的中间结果（`geom1/geom2` 为 shrink/恢复后的副本）。
pub(crate) struct GjkPhase<'a> {
    pub needs_epa: bool,
    pub dist: f32,
    pub ncontact: i32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    pub result: GjkResult,
    pub geom1: Geom<'a>,
    pub geom2: Geom<'a>,
}

/// `epa_phase` 的中间结果（保留 polytope 供 multicontact 使用）。
pub(crate) struct EpaPhase {
    pub dist: f32,
    pub ncontact: i32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    pub multiccd_idx: i32,
    pub polytope: Option<Polytope>,
    pub overflow: u32,
}

/// 上游 `_inflate`：浅穿透把 witness 沿法线膨胀回真实表面。
fn inflate(
    result: &GjkResult,
    geom1: &Geom<'_>,
    geom2: &Geom<'_>,
    geomtype1: i32,
    geomtype2: i32,
    margin1: f32,
    margin2: f32,
) -> (f32, [f32; 3], [f32; 3]) {
    let mut dist = result.dist;
    let mut x1 = result.x1;
    let mut x2 = result.x2;

    if geomtype1 == geom_type::HFIELD {
        let v = result.simplex_index1[0];
        let mut is_side = false;
        for i in 0..result.dim {
            if result.simplex_index1[i] != v {
                is_side = true;
                break;
            }
        }

        if is_side {
            let n = [0.0_f32, 0.0, 1.0];
            let sp = support(geom2, geomtype2, x2);
            x2 = sub3(sp.point, scale3(n, margin2));
            let a = geom1.polyvert[3];
            let b = geom1.polyvert[4];
            let c = geom1.polyvert[5];
            let coordinates = super::simplex::tri_affine_coord(a, b, c, x2);
            if coordinates[0] > 0.0 && coordinates[1] > 0.0 && coordinates[2] > 0.0 {
                x1 = [
                    coordinates[0] * a[0] + coordinates[1] * b[0] + coordinates[2] * c[0],
                    coordinates[0] * a[1] + coordinates[1] * b[1] + coordinates[2] * c[1],
                    coordinates[0] * a[2] + coordinates[1] * b[2] + coordinates[2] * c[2],
                ];
            } else {
                let mut p = c;
                if coordinates[1] > 0.0 {
                    p = b;
                }
                if coordinates[0] > 0.0 {
                    p = a;
                }
                let d = dot3(sub3(x2, p), n);
                x1 = [x2[0] - d * n[0], x2[1] - d * n[1], x2[2] - d * n[2]];
            }
            dist = -length3(sub3(x1, x2));
            return (dist, x1, x2);
        }
    }

    let n = normalize3(sub3(x2, x1));
    if margin1 > 0.0 {
        x1 = [
            x1[0] + margin1 * n[0],
            x1[1] + margin1 * n[1],
            x1[2] + margin1 * n[2],
        ];
    }
    if margin2 > 0.0 {
        x2 = [
            x2[0] - margin2 * n[0],
            x2[1] - margin2 * n[1],
            x2[2] - margin2 * n[2],
        ];
    }
    dist -= margin1 + margin2;
    (dist, x1, x2)
}

/// 上游 `gjk_phase`。
pub(crate) fn gjk_phase<'a>(
    tolerance: f32,
    cutoff: f32,
    gjk_iterations: usize,
    geom1: &Geom<'a>,
    geom2: &Geom<'a>,
    geomtype1: i32,
    geomtype2: i32,
    x_1: [f32; 3],
    x_2: [f32; 3],
) -> GjkPhase<'a> {
    let mut geom1 = *geom1;
    let mut geom2 = *geom2;
    let orig_margin1 = geom1.margin;
    let orig_margin2 = geom2.margin;
    let orig_size1 = geom1.size;
    let orig_size2 = geom2.size;
    let mut full_margin1 = 0.0_f32;
    let mut full_margin2 = 0.0_f32;
    let mut size1 = 0.0_f32;
    let mut size2 = 0.0_f32;
    let empty = GjkResult::default();
    let mut cutoff = cutoff;

    // 离散几何判定：mesh/box/hfield 组合且两侧 margin 都为 0。
    let is_discrete = super::core::is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);

    // 球/胶囊 shrink 为点/线段。
    if geomtype1 == geom_type::SPHERE || geomtype1 == geom_type::CAPSULE {
        size1 = geom1.size[0];
        full_margin1 = size1 + 0.5 * orig_margin1;
        geom1.margin = 0.0;
        geom1.size = [0.0, geom1.size[1], geom1.size[2]];
    }
    if geomtype2 == geom_type::SPHERE || geomtype2 == geom_type::CAPSULE {
        size2 = geom2.size[0];
        full_margin2 = size2 + 0.5 * orig_margin2;
        geom2.margin = 0.0;
        geom2.size = [0.0, geom2.size[1], geom2.size[2]];
    }

    if size1 + size2 > 0.0 {
        cutoff += full_margin1 + full_margin2;
        let result = gjk(
            tolerance,
            gjk_iterations,
            &geom1,
            &geom2,
            x_1,
            x_2,
            geomtype1,
            geomtype2,
            cutoff,
            is_discrete,
        );
        geom1.index = result.index1;
        geom2.index = result.index2;

        // 浅穿透：膨胀接触。
        if result.dist > tolerance {
            if result.dist == FLOAT_MAX {
                return GjkPhase {
                    needs_epa: false,
                    dist: result.dist,
                    ncontact: 1,
                    x1: result.x1,
                    x2: result.x2,
                    result: empty,
                    geom1,
                    geom2,
                };
            }
            let (dist, x1, x2) = inflate(
                &result,
                &geom1,
                &geom2,
                geomtype1,
                geomtype2,
                full_margin1,
                full_margin2,
            );
            return GjkPhase {
                needs_epa: false,
                dist,
                ncontact: 1,
                x1,
                x2,
                result: empty,
                geom1,
                geom2,
            };
        }

        // 深穿透：恢复初始条件后重跑。
        geom1.margin = orig_margin1;
        geom1.size = orig_size1;
        geom2.margin = orig_margin2;
        geom2.size = orig_size2;
        cutoff -= full_margin1 + full_margin2;
    }

    let result = gjk(
        tolerance,
        gjk_iterations,
        &geom1,
        &geom2,
        x_1,
        x_2,
        geomtype1,
        geomtype2,
        cutoff,
        is_discrete,
    );
    geom1.index = result.index1;
    geom2.index = result.index2;

    // 无穿透深度可恢复。
    if result.dist > tolerance || result.dim < 2 || result.separated {
        return GjkPhase {
            needs_epa: false,
            dist: result.dist,
            ncontact: 1,
            x1: result.x1,
            x2: result.x2,
            result: empty,
            geom1,
            geom2,
        };
    }

    GjkPhase {
        needs_epa: true,
        dist: result.dist,
        ncontact: 1,
        x1: result.x1,
        x2: result.x2,
        result,
        geom1,
        geom2,
    }
}
