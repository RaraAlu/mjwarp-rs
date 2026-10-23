//! G08@P5 基本几何窄相切片：plane-sphere 与 sphere-sphere 接触生成。
//!
//! 对齐冻结上游 `mujoco_warp._src.collision_primitive`（G08；P5，提交
//! `71da24d956…`，Apache-2.0）的两类基础几何对：
//!
//! - `plane_sphere`：`dist = dot(sphere_pos - plane_pos, normal) - radius`，
//!   `pos = sphere_pos - normal * (radius + 0.5*dist)`，法线取平面旋转矩阵第三列。
//! - `sphere_sphere`：中心距减半径和，`pos` 在连心线上，法线为单位连心向量。
//! - `plane_box`（G09）：`collision_primitive_core.plane_box` 的 8 角点距离/位置；
//!   wrapper 对 8 个角点依次 `write_contact`，按 margin/gap 检测后最多写出 8 个
//!   候选（面接触通常 4 个、棱 2 个、角 1 个、间隙外 0 个）。
//! - `make_frame`：法线为第一行，`orthogonals` 构造正交基（b 取 y 或 z 轴投影后
//!   归一化，c = a × b）。
//! - `write_contact`/`contact_params`：`margin/gap` 检测、`solmix`/`priority`
//!   材质混合、`condim` 取大、`adhesion` 降维、`MJ_MINMU` 摩擦下限、
//!   `contact_type` 位（CONSTRAINT=1、SENSOR=2，本片 sensor 恒 0）。
//!
//! 输入为 G07 宽相候选（`BroadphaseOutput`）与冻结 `geom_xpos/xmat`；输出接触
//! 结构（`dist/pos/frame/includemargin/dim/geom/worldid/friction/solref/
//! solreffriction/solimp/adhesion/type/geomcollisionid`）。
//!
//! 未验收边界：capsule/ellipsoid/cylinder/mesh/hfield/SDF、box-box、convex/CCD/
//! multiccd、flex 接触、显式 `<pair>` 覆盖（pair_dim/pair_solref/pair_friction 等）、
//! collision sensor 接触类型位、EFC 行地址与约束求解、世界批量参数、GPU 内核与图执行。

use super::{BroadphaseOutput, GEOM_BOX, GEOM_PLANE, GEOM_SPHERE};

/// 上游 `mjMINVAL` 与 `mjMINMU`。
pub const MJ_MINVAL: f32 = 1.0e-15;
pub const MJ_MINMU: f32 = 1.0e-5;
/// 上游 `ContactType` 位。
pub const CONTACT_CONSTRAINT: i32 = 1 << 0;
pub const CONTACT_SENSOR: i32 = 1 << 1;

/// 窄相模型参数（`geom_*` 冻结字段）。
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveModel<'a> {
    pub ngeom: usize,
    pub geom_type: &'a [i32],
    pub geom_condim: &'a [i32],
    pub geom_priority: &'a [i32],
    pub geom_solmix: &'a [f32],
    pub geom_solref: &'a [[f32; 2]],
    pub geom_solimp: &'a [[f32; 5]],
    pub geom_friction: &'a [[f32; 3]],
    pub geom_margin: &'a [f32],
    pub geom_gap: &'a [f32],
    pub geom_adhesion: &'a [f32],
    pub geom_size: &'a [[f32; 3]],
}

/// 冻结几何位姿。
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveState<'a> {
    pub nworld: usize,
    pub geom_xpos: &'a [[f32; 3]],
    pub geom_xmat: &'a [[f32; 9]],
}

/// 接触结构（`d.contact` 子集）。
#[derive(Clone, Debug, PartialEq)]
pub struct GeomContact {
    pub worldid: i32,
    pub geom: [i32; 2],
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
    pub includemargin: f32,
    pub dim: i32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
    pub adhesion: f32,
    pub contact_type: i32,
    pub geomcollisionid: i32,
}

/// 窄相错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NarrowphaseError {
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    UnsupportedPair {
        index: usize,
        geom_type1: i32,
        geom_type2: i32,
    },
    InvalidIndex {
        field: &'static str,
        index: usize,
        limit: usize,
    },
}

impl std::fmt::Display for NarrowphaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "长度不匹配：{field} 期望 {expected}，实际 {actual}"
            ),
            Self::UnsupportedPair {
                index,
                geom_type1,
                geom_type2,
            } => write!(
                formatter,
                "不支持的几何对：candidate[{index}] = ({geom_type1}, {geom_type2})"
            ),
            Self::InvalidIndex {
                field,
                index,
                limit,
            } => write!(formatter, "索引越界：{field}[{index}]，上限 {limit}"),
        }
    }
}

impl std::error::Error for NarrowphaseError {}

mod primitive;
use primitive::column;
pub use primitive::{make_frame, plane_box, plane_sphere, sphere_sphere};

fn safe_div(numerator: f32, denominator: f32) -> f32 {
    numerator
        / if denominator != 0.0 {
            denominator
        } else {
            MJ_MINVAL
        }
}

fn geom_slice(
    model: &PrimitiveModel<'_>,
    geom1: usize,
    geom2: usize,
) -> Result<(), NarrowphaseError> {
    for geom in [geom1, geom2] {
        if geom >= model.ngeom {
            return Err(NarrowphaseError::InvalidIndex {
                field: "geom",
                index: geom,
                limit: model.ngeom,
            });
        }
    }
    Ok(())
}

fn validate(
    model: &PrimitiveModel<'_>,
    state: &PrimitiveState<'_>,
) -> Result<(), NarrowphaseError> {
    let ngeom = model.ngeom;
    for (field, actual) in [
        ("geom_type", model.geom_type.len()),
        ("geom_condim", model.geom_condim.len()),
        ("geom_priority", model.geom_priority.len()),
        ("geom_solmix", model.geom_solmix.len()),
        ("geom_solref", model.geom_solref.len()),
        ("geom_solimp", model.geom_solimp.len()),
        ("geom_friction", model.geom_friction.len()),
        ("geom_margin", model.geom_margin.len()),
        ("geom_gap", model.geom_gap.len()),
        ("geom_adhesion", model.geom_adhesion.len()),
        ("geom_size", model.geom_size.len()),
    ] {
        if actual != ngeom {
            return Err(NarrowphaseError::InvalidLength {
                field,
                expected: ngeom,
                actual,
            });
        }
    }
    for (field, actual) in [
        ("geom_xpos", state.geom_xpos.len()),
        ("geom_xmat", state.geom_xmat.len()),
    ] {
        if actual != state.nworld * ngeom {
            return Err(NarrowphaseError::InvalidLength {
                field,
                expected: state.nworld * ngeom,
                actual,
            });
        }
    }
    Ok(())
}

fn material_params(
    model: &PrimitiveModel<'_>,
    geom1: usize,
    geom2: usize,
) -> (i32, [f32; 5], [f32; 2], [f32; 5], f32) {
    let solmix1 = model.geom_solmix[geom1];
    let solmix2 = model.geom_solmix[geom2];
    let condim1 = model.geom_condim[geom1];
    let condim2 = model.geom_condim[geom2];
    let p1 = model.geom_priority[geom1];
    let p2 = model.geom_priority[geom2];
    let a1 = model.geom_adhesion[geom1];
    let a2 = model.geom_adhesion[geom2];
    let (mix, condim, max_friction, adhesion) = if p1 > p2 {
        (1.0_f32, condim1, model.geom_friction[geom1], a1)
    } else if p2 > p1 {
        (0.0_f32, condim2, model.geom_friction[geom2], a2)
    } else {
        let mut mix = safe_div(solmix1, solmix1 + solmix2);
        if solmix1 < MJ_MINVAL && solmix2 < MJ_MINVAL {
            mix = 0.5;
        }
        if solmix1 < MJ_MINVAL && solmix2 >= MJ_MINVAL {
            mix = 0.0;
        }
        if solmix1 >= MJ_MINVAL && solmix2 < MJ_MINVAL {
            mix = 1.0;
        }
        let condim = condim1.max(condim2);
        let friction1 = model.geom_friction[geom1];
        let friction2 = model.geom_friction[geom2];
        let max_friction = [
            friction1[0].max(friction2[0]),
            friction1[1].max(friction2[1]),
            friction1[2].max(friction2[2]),
        ];
        (mix, condim, max_friction, a1 + a2)
    };
    let friction = [
        max_friction[0],
        max_friction[0],
        max_friction[1],
        max_friction[2],
        max_friction[2],
    ];
    let solref1 = model.geom_solref[geom1];
    let solref2 = model.geom_solref[geom2];
    let solref = if solref1[0] > 0.0 && solref2[0] > 0.0 {
        [
            mix * solref1[0] + (1.0 - mix) * solref2[0],
            mix * solref1[1] + (1.0 - mix) * solref2[1],
        ]
    } else {
        [solref1[0].min(solref2[0]), solref1[1].min(solref2[1])]
    };
    let solimp1 = model.geom_solimp[geom1];
    let solimp2 = model.geom_solimp[geom2];
    let mut solimp = [0.0_f32; 5];
    for index in 0..5 {
        solimp[index] = mix * solimp1[index] + (1.0 - mix) * solimp2[index];
    }
    let mut friction = friction;
    for value in &mut friction {
        *value = value.max(MJ_MINMU);
    }
    (condim, friction, solref, solimp, adhesion)
}

/// 上游 `write_contact`：按 margin/gap 检测与 pairid 语义决定是否写出。
#[allow(clippy::too_many_arguments)]
fn write_contact(
    output: &mut Vec<GeomContact>,
    worldid: usize,
    geom: [i32; 2],
    pairid: [i32; 2],
    dist: f32,
    pos: [f32; 3],
    normal: [f32; 3],
    margin: f32,
    gap: f32,
    condim: i32,
    friction: [f32; 5],
    solref: [f32; 2],
    solimp: [f32; 5],
    adhesion: f32,
) {
    let active = (dist < margin) || ((adhesion != 0.0) && (dist < margin + gap));
    let detected = dist < margin + gap;
    let dim = if adhesion != 0.0 && dist >= margin {
        1
    } else {
        condim
    };
    if (pairid[0] == -2 || !detected) && pairid[1] == -1 {
        return;
    }
    let mut contact_type = 0;
    if pairid[0] >= -1 && detected {
        contact_type |= CONTACT_CONSTRAINT;
    }
    if pairid[1] >= 0 {
        contact_type |= CONTACT_SENSOR;
    }
    let _ = active;
    output.push(GeomContact {
        worldid: worldid as i32,
        geom,
        dist,
        pos,
        frame: make_frame(normal),
        includemargin: margin,
        dim,
        friction,
        solref,
        solreffriction: [0.0, 0.0],
        solimp,
        adhesion,
        contact_type,
        geomcollisionid: 0,
    });
}

/// 上游 `primitive_narrowphase` 的两类基础几何对（plane-sphere / sphere-sphere）。
pub fn primitive_narrowphase(
    model: &PrimitiveModel<'_>,
    state: &PrimitiveState<'_>,
    candidates: &BroadphaseOutput,
) -> Result<Vec<GeomContact>, NarrowphaseError> {
    validate(model, state)?;
    let mut output = Vec::new();
    for index in 0..candidates.pairs.len() {
        let worldid = candidates.worldids[index] as usize;
        if worldid >= state.nworld {
            return Err(NarrowphaseError::InvalidIndex {
                field: "worldid",
                index: worldid,
                limit: state.nworld,
            });
        }
        let geom1 = candidates.pairs[index][0] as usize;
        let geom2 = candidates.pairs[index][1] as usize;
        geom_slice(model, geom1, geom2)?;
        let pairid = candidates.pairids[index];
        let type1 = model.geom_type[geom1];
        let type2 = model.geom_type[geom2];
        let index_base = worldid * model.ngeom;
        let pos1 = state.geom_xpos[index_base + geom1];
        let pos2 = state.geom_xpos[index_base + geom2];
        let margin = model.geom_margin[geom1] + model.geom_margin[geom2];
        let gap = model.geom_gap[geom1] + model.geom_gap[geom2];
        let (condim, friction, solref, solimp, adhesion) = material_params(model, geom1, geom2);
        match (type1, type2) {
            (GEOM_PLANE, GEOM_SPHERE) => {
                let normal = column(&state.geom_xmat[index_base + geom1], 2);
                let radius = model.geom_size[geom2][0];
                let (dist, pos) = plane_sphere(normal, pos1, pos2, radius);
                write_contact(
                    &mut output,
                    worldid,
                    [geom1 as i32, geom2 as i32],
                    pairid,
                    dist,
                    pos,
                    normal,
                    margin,
                    gap,
                    condim,
                    friction,
                    solref,
                    solimp,
                    adhesion,
                );
            }
            (GEOM_SPHERE, GEOM_SPHERE) => {
                let (dist, pos, normal) = sphere_sphere(
                    pos1,
                    model.geom_size[geom1][0],
                    pos2,
                    model.geom_size[geom2][0],
                );
                write_contact(
                    &mut output,
                    worldid,
                    [geom1 as i32, geom2 as i32],
                    pairid,
                    dist,
                    pos,
                    normal,
                    margin,
                    gap,
                    condim,
                    friction,
                    solref,
                    solimp,
                    adhesion,
                );
            }
            (GEOM_PLANE, GEOM_BOX) => {
                let normal = column(&state.geom_xmat[index_base + geom1], 2);
                let (dists, positions) = plane_box(
                    normal,
                    pos1,
                    pos2,
                    state.geom_xmat[index_base + geom2],
                    model.geom_size[geom2],
                );
                for index in 0..8 {
                    write_contact(
                        &mut output,
                        worldid,
                        [geom1 as i32, geom2 as i32],
                        pairid,
                        dists[index],
                        positions[index],
                        normal,
                        margin,
                        gap,
                        condim,
                        friction,
                        solref,
                        solimp,
                        adhesion,
                    );
                }
            }
            (GEOM_BOX, GEOM_PLANE) => {
                let normal = column(&state.geom_xmat[index_base + geom2], 2);
                let (dists, positions) = plane_box(
                    normal,
                    pos2,
                    pos1,
                    state.geom_xmat[index_base + geom1],
                    model.geom_size[geom1],
                );
                for index in 0..8 {
                    write_contact(
                        &mut output,
                        worldid,
                        [geom1 as i32, geom2 as i32],
                        pairid,
                        dists[index],
                        positions[index],
                        normal,
                        margin,
                        gap,
                        condim,
                        friction,
                        solref,
                        solimp,
                        adhesion,
                    );
                }
            }
            _ => {
                return Err(NarrowphaseError::UnsupportedPair {
                    index,
                    geom_type1: type1,
                    geom_type2: type2,
                });
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
