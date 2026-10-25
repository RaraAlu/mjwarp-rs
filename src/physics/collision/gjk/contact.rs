//! 上游 `collision_convex.eval_ccd_write_contact` 的宿主侧接触装配子集。
//!
//! 语义：原点平移到 geom1 位置（数值精度与上游一致）→ `ccd()` → margin 膨胀修正
//! → multicontact（在 box/mesh/cylinder 组合且未禁用时）→ frame 由 witness 差分
//! 生成 → 依次写接触。

use super::Geom;
use super::ccd::epa_phase;
use super::geom_type;
use super::math::{add3, make_frame, sub3};
use super::multicontact::multicontact;
use super::phase::{CcdOptions, gjk_phase};

/// 单个接触（上游 `contact` 的数值字段子集）。
#[derive(Clone, Copy, Debug)]
pub struct CcdContact {
    pub dist: f32,
    pub pos: [f32; 3],
    /// 行优先 3×3 接触坐标系。
    pub frame: [f32; 9],
}

/// `ccd_pair` 返回。
#[derive(Clone, Debug, Default)]
pub struct CcdOutput {
    pub contacts: Vec<CcdContact>,
    /// 上游 `OverflowType` 位（`CCD`/`EPA_HORIZON`）。
    pub overflow: u32,
    /// `ccd()` 原始接触候选数（`0` 分离、`1` 候选）。
    pub ncontact: i32,
    /// margin 修正后的原始距离（上游 `ccd` 输出）。
    pub dist: f32,
}

/// 接触装配选项。
#[derive(Clone, Copy, Debug)]
pub struct ContactOptions {
    pub ccd: CcdOptions,
    /// 上游 `contact_margin_gap`：pair margin 或两 geom margin 之和。
    pub margin: f32,
    /// 上游 `contact_margin_gap`：pair gap 或两 geom gap 之和。
    pub gap: f32,
    /// `pairid[1] >= 0`：collision sensor 对。
    pub collision_sensor: bool,
    /// `m.opt.disableflags & MULTICCD == 0`。
    pub use_multiccd: bool,
}

impl Default for ContactOptions {
    fn default() -> Self {
        Self {
            ccd: CcdOptions::default(),
            margin: 0.0,
            gap: 0.0,
            collision_sensor: false,
            use_multiccd: true,
        }
    }
}

/// 上游 `eval_ccd_write_contact`：两 geom 的 CCD 接触输出。
pub fn ccd_pair<'a>(
    geom1: &Geom<'a>,
    geom2: &Geom<'a>,
    geomtype1: i32,
    geomtype2: i32,
    options: ContactOptions,
) -> CcdOutput {
    let mut geom1 = *geom1;
    let mut geom2 = *geom2;
    geom1.margin = options.margin;
    geom2.margin = options.margin;

    let tolerance = options.ccd.tolerance;
    let cutoff = if options.collision_sensor {
        1.0e32_f32
    } else {
        options.gap
    };

    // 原点平移到 geom1 位置，保证平移不变性下的数值精度。
    let origin = geom1.pos;
    geom1.pos = [0.0; 3];
    geom2.pos = sub3(geom2.pos, origin);

    let phase = gjk_phase(
        tolerance,
        cutoff,
        options.ccd.gjk_iterations,
        &geom1,
        &geom2,
        geomtype1,
        geomtype2,
        geom1.pos,
        geom2.pos,
    );

    let mut dist = phase.dist;
    let mut ncontact = phase.ncontact;
    let (mut w1, mut w2) = (phase.x1, phase.x2);
    let mut multiccd_idx = -1;
    let mut overflow = 0_u32;
    let mut polytope = None;
    let mut epa_face = 0_u32;

    if phase.needs_epa {
        let epa = epa_phase(
            tolerance,
            options.ccd.epa_iterations,
            phase.result,
            &phase.geom1,
            &phase.geom2,
            geomtype1,
            geomtype2,
        );
        dist = epa.dist;
        ncontact = epa.ncontact;
        w1 = epa.x1;
        w2 = epa.x2;
        multiccd_idx = epa.multiccd_idx;
        overflow |= epa.overflow;
        if let Some(pt) = epa.polytope.as_ref() {
            if multiccd_idx > -1 && (multiccd_idx as usize) < pt.face.len() {
                epa_face = pt.face[multiccd_idx as usize];
            }
            polytope = epa.polytope;
        }
        if dist == 1.0e30_f32 {
            overflow |= super::overflow_type::CCD;
        }
    }

    // 超出 gap：无接触。
    if dist >= options.gap && !options.collision_sensor {
        return CcdOutput {
            contacts: Vec::new(),
            overflow,
            ncontact,
            dist,
        };
    }

    // CCD 在 margin 膨胀形状上工作，把距离修正回真实表面距离。
    dist += options.margin;

    let mut witness1 = [[0.0_f32; 3]; 4];
    let mut witness2 = [[0.0_f32; 3]; 4];
    let mut dists = [0.0_f32; 4];
    witness1[0] = w1;
    witness2[0] = w2;
    let mut ncollision = ncontact.max(0) as usize;

    let supported = |value: i32| {
        value == geom_type::BOX || value == geom_type::MESH || value == geom_type::CYLINDER
    };
    let multicontact_enabled =
        options.use_multiccd || (geomtype1 == geom_type::BOX && geomtype2 == geom_type::BOX);
    if multicontact_enabled && supported(geomtype1) && supported(geomtype2) {
        if geomtype1 == geom_type::MESH && geom1.mesh.is_none() {
            multiccd_idx = -1;
        }
        if geomtype2 == geom_type::MESH && geom2.mesh.is_none() {
            multiccd_idx = -1;
        }

        dists = [0.0; 4];
        if multiccd_idx > -1 {
            let pt = polytope
                .as_ref()
                .expect("multiccd_idx 有效时必须存在 EPA polytope");
            let mc = multicontact(pt, epa_face, w1, w2, &geom1, &geom2, geomtype1, geomtype2);
            ncollision = mc.ncontact;
            witness1 = mc.witness1;
            witness2 = mc.witness2;
            dists = mc.dists;

            // 裁剪可能产生 0 接触，回退到单个 EPA witness。
            if ncollision < 1 {
                ncollision = 1;
                witness1[0] = w1;
                witness2[0] = w2;
            }
        }
    }

    // 重叠时 witness1 越过 witness2，方向需按距离符号取反。
    let frame = if dist <= options.margin {
        make_frame(sub3(witness1[0], witness2[0]))
    } else {
        make_frame(sub3(witness2[0], witness1[0]))
    };

    let mut contacts = Vec::with_capacity(ncollision);
    for i in 0..ncollision {
        let contact_dist = if ncollision > 1 { dists[i] } else { dist };
        // 上游 `write_contact`：`detected = dist < margin + gap`。
        if contact_dist < options.margin + options.gap {
            let sum = add3(witness1[i], witness2[i]);
            let pos = add3([0.5 * sum[0], 0.5 * sum[1], 0.5 * sum[2]], origin);
            contacts.push(CcdContact {
                dist: contact_dist,
                pos,
                frame,
            });
        }
    }

    CcdOutput {
        contacts,
        overflow,
        ncontact: ncontact.max(0),
        dist,
    }
}
