//! SAP 宽相：固定方向投影 + 排序 + 扫描，复用上游同一过滤与写出语义。
//!
//! 对齐 `sap_broadphase`/`_sap_project`/`sap_range`/`_sap_broadphase`（U008）：
//! 方向为上游固定随机方向 `normalize(0.5935, 0.7790, 0.1235)`（f32）；平面
//! `rbound == 0` 投影半径取 `MJ_MAXVAL`；缺失 mesh 投影为反转区间（下界
//! `MJ_MAXVAL`、上界 `-MJ_MAXVAL`）并在扫描时再次跳过；扫描范围取排序后
//! 首个 `lower > upper_i` 的位置（含边界位置，对齐上游 `sap_binary_search`
//! 的 `>` 与 `sap_range` 的 `limit`）。`SAP_TILE`/`SAP_SEGMENTED` 只差上游
//! 排序策略，本片两条走同一宿主稳定排序，候选集合一致。

use super::{BroadphaseModel, BroadphaseOptions, BroadphaseOutput, BroadphaseState};
use super::{CandidateWriter, MJ_MAXVAL, filter, upper_tri_index};

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

/// 上游 `wp.normalize(wp.vec3(0.5935, 0.7790, 0.1235))`（f32）。
fn direction() -> [f32; 3] {
    let raw = [0.5935_f32, 0.7790, 0.1235];
    let norm = dot3(raw, raw).sqrt();
    [raw[0] / norm, raw[1] / norm, raw[2] / norm]
}

/// 逐 geom 投影区间（对齐 `_sap_project`）。
fn project(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    world: usize,
    direction: [f32; 3],
) -> (Vec<f32>, Vec<f32>) {
    let mut lower = Vec::with_capacity(model.ngeom);
    let mut upper = Vec::with_capacity(model.ngeom);
    for geom in 0..model.ngeom {
        if model.mesh_missing(world, geom) {
            lower.push(MJ_MAXVAL);
            upper.push(-MJ_MAXVAL);
            continue;
        }
        let xpos = state.geom_xpos[world * model.ngeom + geom];
        let center = dot3(direction, xpos);
        let rbound = model.rbound(world, geom);
        let radius = if rbound == 0.0 {
            MJ_MAXVAL + model.margin(world, geom) + model.gap(world, geom)
        } else {
            rbound + model.margin(world, geom) + model.gap(world, geom)
        };
        if center.is_nan() {
            lower.push(MJ_MAXVAL);
            upper.push(MJ_MAXVAL);
        } else {
            lower.push(center - radius);
            upper.push(center + radius);
        }
    }
    (lower, upper)
}

/// 首个 `values[index] > value` 的位置（`values` 升序），无则 `values.len()`。
fn first_greater(values: &[f32], value: f32, start: usize) -> usize {
    let mut low = start;
    let mut high = values.len();
    while low < high {
        let middle = (low + high) / 2;
        if values[middle] > value {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    high
}

pub(crate) fn run(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    options: BroadphaseOptions,
) -> BroadphaseOutput {
    let direction = direction();
    let ngeom = model.ngeom;
    let mut writer = CandidateWriter::new(options.capacity);
    for world in 0..state.nworld {
        let (lower, upper) = project(model, state, world, direction);
        let mut order: Vec<usize> = (0..ngeom).collect();
        order.sort_by(|left, right| lower[*left].total_cmp(&lower[*right]));
        let sorted_lower: Vec<f32> = order.iter().map(|geom| lower[*geom]).collect();
        for position in 0..ngeom {
            let geom1 = order[position];
            let limit = first_greater(&sorted_lower, upper[geom1], position + 1)
                .min(ngeom.saturating_sub(1));
            if limit <= position {
                continue;
            }
            for &geom2 in &order[(position + 1)..=limit] {
                if model.mesh_missing(world, geom1) || model.mesh_missing(world, geom2) {
                    continue;
                }
                let pairid = model.full_pairid[upper_tri_index(ngeom, geom1, geom2)];
                if pairid[0] < -1 && pairid[1] < 0 {
                    continue;
                }
                let forced = pairid[1] >= 0;
                if forced || filter::passes(options.filter, model, state, world, geom1, geom2) {
                    writer.add(
                        world,
                        geom1 as i32,
                        geom2 as i32,
                        model.geom_type[geom1],
                        model.geom_type[geom2],
                        pairid,
                    );
                }
            }
        }
    }
    writer.finish()
}
