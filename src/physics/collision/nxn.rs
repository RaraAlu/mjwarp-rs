//! NXN 宽相：遍历冻结过滤后的 geom 对表，逐对执行上游过滤并写出候选。
//!
//! 对齐 `nxn_broadphase`/`_nxn_broadphase`（U007）：缺失 mesh 跳过、四位过滤、
//! `pairid[1] >= 0`（collision sensor）绕过过滤强制包含、按 geom 类型排序写出、
//! `naconmax` 容量截断。SLEEP 睡眠跳过与 incremental awake 复跑不在本片。

use super::{BroadphaseModel, BroadphaseOptions, BroadphaseOutput, BroadphaseState};
use super::{CandidateWriter, filter};

pub(crate) fn run(
    model: &BroadphaseModel<'_>,
    state: &BroadphaseState<'_>,
    options: BroadphaseOptions,
) -> BroadphaseOutput {
    let mut writer = CandidateWriter::new(options.capacity);
    for world in 0..state.nworld {
        for pair in model.pairs {
            let geom1 = pair.geom1 as usize;
            let geom2 = pair.geom2 as usize;
            if model.mesh_missing(world, geom1) || model.mesh_missing(world, geom2) {
                continue;
            }
            let forced = pair.collision_pairid >= 0;
            if forced || filter::passes(options.filter, model, state, world, geom1, geom2) {
                writer.add(
                    world,
                    pair.geom1,
                    pair.geom2,
                    model.geom_type[geom1],
                    model.geom_type[geom2],
                    [pair.contact_pairid, pair.collision_pairid],
                );
            }
        }
    }
    writer.finish()
}
