//! 冻结输入校验。

use super::support::artifact;
use super::support::rows;
use super::*;

pub(super) fn validate_input(input: &FrozenWarpKinematicsInput) -> Result<(), ProbeError> {
    if input.schema != 1
        || input.scope != "gravity-upstream-independent-qpos-kinematics-com-cdof"
        || input.upstream_revision != REVISION
        || input.warp_version != WARP_VERSION
        || (input.worlds, input.nbody, input.njnt, input.nq, input.nv) != (7, 5, 4, 4, 4)
        || input.nbranch != 2
        || input.nmocap != 1
        || input.legacy_static_input != "../independent-inputs/input.json"
        || input.legacy_static_input_sha256
            != "5136b09e1d727b7799d527ebed42b1f7ad05b0a0040928a78a69137b2400ec0c"
        || input.candidate_kinematics_inputs
    {
        return Err(artifact("input", "frozen identity mismatch"));
    }
    let exact = [
        (input.body_parentid.len(), input.nbody),
        (input.body_rootid.len(), input.nbody),
        (input.body_mocapid.len(), input.nbody),
        (input.body_jntnum.len(), input.nbody),
        (input.body_jntadr.len(), input.nbody),
        (input.jnt_type.len(), input.njnt),
        (input.jnt_qposadr.len(), input.njnt),
        (input.jnt_dofadr.len(), input.njnt),
        (input.jnt_bodyid.len(), input.njnt),
        (input.body_branches.len(), input.nbody - 1),
        (input.body_branch_start.len(), input.nbranch + 1),
        (input.qpos.len(), input.worlds),
        (input.mocap_pos.len(), input.nmocap),
        (input.mocap_quat_wxyz.len(), input.nmocap),
    ];
    if exact.iter().any(|&(actual, expected)| actual != expected)
        || !rows(&input.qpos0, input.nq)
        || !rows(&input.body_pos, input.nbody * 3)
        || !rows(&input.body_quat_wxyz, input.nbody * 4)
        || !rows(&input.jnt_pos, input.njnt * 3)
        || !rows(&input.jnt_axis, input.njnt * 3)
        || !rows(&input.qpos, input.nq)
        || !rows(&input.mocap_pos, 3)
        || !rows(&input.mocap_quat_wxyz, 4)
        || !rows(&input.body_ipos, input.nbody * 3)
        || !rows(&input.body_iquat_wxyz, input.nbody * 4)
        || !rows(&input.body_mass, input.nbody)
        || !rows(&input.body_subtreemass, input.nbody)
    {
        return Err(artifact("input", "field length mismatch"));
    }
    if [
        &input.qpos0,
        &input.body_pos,
        &input.body_quat_wxyz,
        &input.jnt_pos,
        &input.jnt_axis,
        &input.qpos,
        &input.mocap_pos,
        &input.mocap_quat_wxyz,
        &input.body_ipos,
        &input.body_iquat_wxyz,
        &input.body_mass,
        &input.body_subtreemass,
    ]
    .into_iter()
    .flat_map(|rows| rows.iter().flatten())
    .any(|value| !value.is_finite())
    {
        return Err(artifact("input", "non-finite value"));
    }
    if input
        .qpos
        .iter()
        .flatten()
        .any(|value| value.to_bits() != 0)
        || input.body_parentid[0] != 0
        || input.body_rootid[0] != 0
        || input.body_branch_start.first() != Some(&0)
        || input.body_branch_start.last() != Some(&(input.body_branches.len() as i32))
    {
        return Err(artifact("input", "topology or state mismatch"));
    }
    let mut bodies = input
        .body_tree_levels
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    bodies.sort_unstable();
    if bodies != (1..input.nbody as i32).collect::<Vec<_>>() {
        return Err(artifact("input", "body tree mismatch"));
    }
    Ok(())
}
