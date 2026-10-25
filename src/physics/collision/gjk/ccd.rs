//! 上游 `collision_gjk` 的 `epa_phase()` 与 `ccd()` 入口。

use super::epa::epa;
use super::geom_type;
use super::phase::{CcdOptions, CcdResult, EpaPhase, gjk_phase};
use super::polytope_build::{polytope2, polytope3, polytope4};
use super::{FLOAT_MAX, Geom, GjkResult, Polytope};

impl Polytope {
    /// 上游 `epa_phase` 初始化：清零计数并保留缓冲容量。
    fn reset_for_epa(&mut self) {
        self.nface = 0;
        self.nvert = 0;
        self.nhorizon = 0;
        self.status = 0;
    }
}

/// 上游 `epa_phase`。
pub(crate) fn epa_phase<'a>(
    tolerance: f32,
    epa_iterations: usize,
    mut result: GjkResult,
    geom1: &Geom<'a>,
    geom2: &Geom<'a>,
    geomtype1: i32,
    geomtype2: i32,
) -> EpaPhase {
    let mut pt = Polytope::with_capacity(epa_iterations);
    pt.reset_for_epa();

    if result.dim == 2 {
        let (new_pt, new_result) = polytope2(
            pt,
            &result.simplex,
            &result.simplex1,
            &result.simplex2,
            &result.simplex_index1,
            &result.simplex_index2,
            geom1,
            geom2,
            geomtype1,
            geomtype2,
        );
        pt = new_pt;
        if pt.status == -1 {
            result.simplex = new_result.simplex;
            result.simplex1 = new_result.simplex1;
            result.simplex2 = new_result.simplex2;
            result.simplex_index1 = new_result.simplex_index1;
            result.simplex_index2 = new_result.simplex_index2;
            result.dim = 3;
        }
    } else if result.dim == 4 {
        let (new_pt, new_result) = polytope4(
            pt,
            &result.simplex,
            &result.simplex1,
            &result.simplex2,
            &result.simplex_index1,
            &result.simplex_index2,
        );
        pt = new_pt;
        if pt.status == -1 {
            result.simplex = new_result.simplex;
            result.simplex1 = new_result.simplex1;
            result.simplex2 = new_result.simplex2;
            result.simplex_index1 = new_result.simplex_index1;
            result.simplex_index2 = new_result.simplex_index2;
            result.dim = 3;
        }
    }

    // polytope2/polytope4 可能回退到 2-单纯形。
    if result.dim == 3 {
        pt = polytope3(
            pt,
            result.dist,
            &result.simplex,
            &result.simplex1,
            &result.simplex2,
            &result.simplex_index1,
            &result.simplex_index2,
            geom1,
            geom2,
            geomtype1,
            geomtype2,
        );
    }

    // 原点在边界上：不算穿透。
    if pt.status != 0 {
        return EpaPhase {
            dist: result.dist,
            ncontact: 1,
            x1: result.x1,
            x2: result.x2,
            multiccd_idx: -1,
            polytope: None,
            overflow: 0,
        };
    }

    let is_discrete = super::core::is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);
    let epa_result = epa(
        pt,
        tolerance,
        epa_iterations,
        geom1,
        geom2,
        geomtype1,
        geomtype2,
        is_discrete,
    );
    if epa_result.face_idx == -1 {
        return EpaPhase {
            dist: FLOAT_MAX,
            ncontact: 0,
            x1: [0.0; 3],
            x2: [0.0; 3],
            multiccd_idx: -1,
            polytope: None,
            overflow: epa_result.overflow,
        };
    }

    let mut multiccd_idx = epa_result.face_idx;
    // margin 非零时不支持 multicontact。
    if geom1.margin != 0.0 || geom2.margin != 0.0 {
        multiccd_idx = -1;
    }
    let supported = |value: i32| {
        value == geom_type::BOX || value == geom_type::MESH || value == geom_type::CYLINDER
    };
    if !supported(geomtype1) || !supported(geomtype2) {
        multiccd_idx = -1;
    }

    EpaPhase {
        dist: epa_result.dist,
        ncontact: 1,
        x1: epa_result.x1,
        x2: epa_result.x2,
        multiccd_idx,
        polytope: Some(epa_result.polytope),
        overflow: epa_result.overflow,
    }
}

/// 上游 `ccd()`：通用凸碰撞检测入口。
/// 上游 `ccd()`：通用凸碰撞检测入口。
pub fn ccd<'a>(
    geom1: &Geom<'a>,
    geom2: &Geom<'a>,
    geomtype1: i32,
    geomtype2: i32,
    x_1: [f32; 3],
    x_2: [f32; 3],
    options: CcdOptions,
) -> CcdResult {
    let internal = ccd_internal(geom1, geom2, geomtype1, geomtype2, x_1, x_2, options);
    CcdResult {
        dist: internal.dist,
        ncontact: internal.ncontact,
        x1: internal.x1,
        x2: internal.x2,
        multiccd_idx: internal.multiccd_idx,
        overflow: internal.overflow,
    }
}

/// `ccd()` 的内部形态。
pub(crate) struct CcdInternal {
    pub dist: f32,
    pub ncontact: i32,
    pub x1: [f32; 3],
    pub x2: [f32; 3],
    pub multiccd_idx: i32,
    pub overflow: u32,
}

pub(crate) fn ccd_internal<'a>(
    geom1: &Geom<'a>,
    geom2: &Geom<'a>,
    geomtype1: i32,
    geomtype2: i32,
    x_1: [f32; 3],
    x_2: [f32; 3],
    options: CcdOptions,
) -> CcdInternal {
    let phase = gjk_phase(
        options.tolerance,
        options.cutoff,
        options.gjk_iterations,
        geom1,
        geom2,
        geomtype1,
        geomtype2,
        x_1,
        x_2,
    );

    if !phase.needs_epa {
        return CcdInternal {
            dist: phase.dist,
            ncontact: phase.ncontact,
            x1: phase.x1,
            x2: phase.x2,
            multiccd_idx: -1,
            overflow: 0,
        };
    }

    // 上游在此处做 naccdmax 容量检查并计数；宿主侧由调用方承担。
    let epa_phase_result = epa_phase(
        options.tolerance,
        options.epa_iterations,
        phase.result,
        &phase.geom1,
        &phase.geom2,
        geomtype1,
        geomtype2,
    );

    let mut overflow = epa_phase_result.overflow;
    if epa_phase_result.dist == FLOAT_MAX {
        overflow |= super::overflow_type::CCD;
    }

    CcdInternal {
        dist: epa_phase_result.dist,
        ncontact: epa_phase_result.ncontact,
        x1: epa_phase_result.x1,
        x2: epa_phase_result.x2,
        multiccd_idx: epa_phase_result.multiccd_idx,
        overflow,
    }
}
