//! 冻结工件诊断启动入口。

use super::support::artifact;
use super::support::cuda_error;
use super::support::f32_arg;
use super::support::flatten;
use super::support::i32_arg;
use super::support::launch2;
use super::support::repeat_rows;
use super::validate_input::validate_input;
use super::*;

impl FrozenWarpKinematicsArtifact {
    pub fn launch_diagnostic(
        &self,
        device: usize,
    ) -> Result<FrozenWarpKinematicsReadback, ProbeError> {
        let input = &self.input;
        validate_input(input)?;
        let stream = crate::runtime::cuda::transfer_stream(device)?;
        let cache = ModuleCache::new(stream.context().clone());
        let handle = |entry| {
            cache
                .frozen_function(stream.context(), &self.frozen, entry)
                .map_err(super::support::launcher_error(
                    "frozen-warp-kinematics-entry",
                ))
        };
        let kinematics = handle(KIN_ENTRY)?;
        let matrices = handle(MATRIX_ENTRY)?;
        let inertial = handle(INERTIAL_ENTRY)?;
        let init = handle(INIT_ENTRY)?;
        let acc = handle(ACC_ENTRY)?;
        let div = handle(DIV_ENTRY)?;
        let cdof_kernel = handle(CDOF_ENTRY)?;

        let qpos0_values = flatten(&input.qpos0);
        let body_pos_values = flatten(&input.body_pos);
        let body_quat_values = flatten(&input.body_quat_wxyz);
        let jnt_pos_values = flatten(&input.jnt_pos);
        let jnt_axis_values = flatten(&input.jnt_axis);
        let qpos_values = flatten(&input.qpos);
        let mocap_pos_values = repeat_rows(&input.mocap_pos, input.worlds);
        let mocap_quat_values = repeat_rows(&input.mocap_quat_wxyz, input.worlds);
        let body_ipos_values = flatten(&input.body_ipos);
        let body_iquat_values = flatten(&input.body_iquat_wxyz);
        let body_mass_values = flatten(&input.body_mass);
        let body_subtreemass_values = flatten(&input.body_subtreemass);
        let tree_values = input
            .body_tree_levels
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();

        let qpos0 = stream
            .clone_htod(&qpos0_values)
            .map_err(cuda_error("upload"))?;
        let parent = stream
            .clone_htod(&input.body_parentid)
            .map_err(cuda_error("upload"))?;
        let roots = stream
            .clone_htod(&input.body_rootid)
            .map_err(cuda_error("upload"))?;
        let mocapid = stream
            .clone_htod(&input.body_mocapid)
            .map_err(cuda_error("upload"))?;
        let jntnum = stream
            .clone_htod(&input.body_jntnum)
            .map_err(cuda_error("upload"))?;
        let jntadr = stream
            .clone_htod(&input.body_jntadr)
            .map_err(cuda_error("upload"))?;
        let body_pos = stream
            .clone_htod(&body_pos_values)
            .map_err(cuda_error("upload"))?;
        let body_quat = stream
            .clone_htod(&body_quat_values)
            .map_err(cuda_error("upload"))?;
        let jnt_type = stream
            .clone_htod(&input.jnt_type)
            .map_err(cuda_error("upload"))?;
        let jnt_qposadr = stream
            .clone_htod(&input.jnt_qposadr)
            .map_err(cuda_error("upload"))?;
        let jnt_dofadr = stream
            .clone_htod(&input.jnt_dofadr)
            .map_err(cuda_error("upload"))?;
        let jnt_bodyid = stream
            .clone_htod(&input.jnt_bodyid)
            .map_err(cuda_error("upload"))?;
        let jnt_pos = stream
            .clone_htod(&jnt_pos_values)
            .map_err(cuda_error("upload"))?;
        let jnt_axis = stream
            .clone_htod(&jnt_axis_values)
            .map_err(cuda_error("upload"))?;
        let branches = stream
            .clone_htod(&input.body_branches)
            .map_err(cuda_error("upload"))?;
        let branch_start = stream
            .clone_htod(&input.body_branch_start)
            .map_err(cuda_error("upload"))?;
        let qpos = stream
            .clone_htod(&qpos_values)
            .map_err(cuda_error("upload"))?;
        let mocap_pos = stream
            .clone_htod(&mocap_pos_values)
            .map_err(cuda_error("upload"))?;
        let mocap_quat = stream
            .clone_htod(&mocap_quat_values)
            .map_err(cuda_error("upload"))?;
        let body_ipos = stream
            .clone_htod(&body_ipos_values)
            .map_err(cuda_error("upload"))?;
        let body_iquat = stream
            .clone_htod(&body_iquat_values)
            .map_err(cuda_error("upload"))?;
        let body_mass = stream
            .clone_htod(&body_mass_values)
            .map_err(cuda_error("upload"))?;
        let body_subtreemass = stream
            .clone_htod(&body_subtreemass_values)
            .map_err(cuda_error("upload"))?;
        let tree = stream
            .clone_htod(&tree_values)
            .map_err(cuda_error("upload"))?;

        let mut xpos_seed = vec![f32::NAN; input.worlds * input.nbody * 3];
        let mut xquat_seed = vec![f32::NAN; input.worlds * input.nbody * 4];
        for world in 0..input.worlds {
            xpos_seed[world * input.nbody * 3..world * input.nbody * 3 + 3].fill(0.0);
            xquat_seed[world * input.nbody * 4] = 1.0;
            xquat_seed[world * input.nbody * 4 + 1..world * input.nbody * 4 + 4].fill(0.0);
        }
        let mut xpos = stream
            .clone_htod(&xpos_seed)
            .map_err(cuda_error("output"))?;
        let mut xquat = stream
            .clone_htod(&xquat_seed)
            .map_err(cuda_error("output"))?;
        let mut xanchor = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.njnt * 3])
            .map_err(cuda_error("output"))?;
        let mut xaxis = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.njnt * 3])
            .map_err(cuda_error("output"))?;
        let mut xmat = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 9])
            .map_err(cuda_error("output"))?;
        let mut xipos = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 3])
            .map_err(cuda_error("output"))?;
        let mut ximat = stream
            .clone_htod(&vec![f32::NAN; input.worlds * input.nbody * 9])
            .map_err(cuda_error("output"))?;
        let mut subtree = stream
            .alloc_zeros::<f32>(input.worlds * input.nbody * 3)
            .map_err(cuda_error("output"))?;
        let mut cdof = stream
            .alloc_zeros::<f32>(input.worlds * input.nv * 6)
            .map_err(cuda_error("output"))?;

        let (qpos0_ptr, qpos0_record) = qpos0.device_ptr(&stream);
        let (parent_ptr, parent_record) = parent.device_ptr(&stream);
        let (roots_ptr, roots_record) = roots.device_ptr(&stream);
        let (mocapid_ptr, mocapid_record) = mocapid.device_ptr(&stream);
        let (jntnum_ptr, jntnum_record) = jntnum.device_ptr(&stream);
        let (jntadr_ptr, jntadr_record) = jntadr.device_ptr(&stream);
        let (body_pos_ptr, body_pos_record) = body_pos.device_ptr(&stream);
        let (body_quat_ptr, body_quat_record) = body_quat.device_ptr(&stream);
        let (jnt_type_ptr, jnt_type_record) = jnt_type.device_ptr(&stream);
        let (jnt_qposadr_ptr, jnt_qposadr_record) = jnt_qposadr.device_ptr(&stream);
        let (jnt_dofadr_ptr, jnt_dofadr_record) = jnt_dofadr.device_ptr(&stream);
        let (jnt_bodyid_ptr, jnt_bodyid_record) = jnt_bodyid.device_ptr(&stream);
        let (jnt_pos_ptr, jnt_pos_record) = jnt_pos.device_ptr(&stream);
        let (jnt_axis_ptr, jnt_axis_record) = jnt_axis.device_ptr(&stream);
        let (branches_ptr, branches_record) = branches.device_ptr(&stream);
        let (branch_start_ptr, branch_start_record) = branch_start.device_ptr(&stream);
        let (qpos_ptr, qpos_record) = qpos.device_ptr(&stream);
        let (mocap_pos_ptr, mocap_pos_record) = mocap_pos.device_ptr(&stream);
        let (mocap_quat_ptr, mocap_quat_record) = mocap_quat.device_ptr(&stream);
        let (body_ipos_ptr, body_ipos_record) = body_ipos.device_ptr(&stream);
        let (body_iquat_ptr, body_iquat_record) = body_iquat.device_ptr(&stream);
        let (body_mass_ptr, body_mass_record) = body_mass.device_ptr(&stream);
        let (body_subtreemass_ptr, body_subtreemass_record) = body_subtreemass.device_ptr(&stream);
        let (tree_ptr, tree_record) = tree.device_ptr(&stream);

        let cap = |elements: usize| u64::try_from(elements).unwrap_or(u64::MAX) * 4;
        let qpos0_cap = cap(qpos0.len());
        let xpos_cap = cap(xpos.len());
        let xquat_cap = cap(xquat.len());
        let xanchor_cap = cap(xanchor.len());
        let xaxis_cap = cap(xaxis.len());
        let xmat_cap = cap(xmat.len());
        let xipos_cap = cap(xipos.len());
        let ximat_cap = cap(ximat.len());
        let subtree_cap = cap(subtree.len());
        let cdof_cap = cap(cdof.len());
        let (xpos_ptr, xpos_record) = xpos.device_ptr_mut(&stream);
        let (xquat_ptr, xquat_record) = xquat.device_ptr_mut(&stream);
        let (xanchor_ptr, xanchor_record) = xanchor.device_ptr_mut(&stream);
        let (xaxis_ptr, xaxis_record) = xaxis.device_ptr_mut(&stream);
        let (xmat_ptr, xmat_record) = xmat.device_ptr_mut(&stream);
        let (xipos_ptr, xipos_record) = xipos.device_ptr_mut(&stream);
        let (ximat_ptr, ximat_record) = ximat.device_ptr_mut(&stream);
        let (subtree_ptr, subtree_record) = subtree.device_ptr_mut(&stream);
        let (cdof_ptr, cdof_record) = cdof.device_ptr_mut(&stream);

        launch2(
            &stream,
            &kinematics,
            [input.worlds, input.nbranch],
            &[
                f32_arg(qpos0_ptr, &[input.qpos0.len(), input.nq], 4, qpos0_cap)?,
                i32_arg(parent_ptr, &[input.nbody], cap(parent.len()))?,
                i32_arg(mocapid_ptr, &[input.nbody], cap(mocapid.len()))?,
                i32_arg(jntnum_ptr, &[input.nbody], cap(jntnum.len()))?,
                i32_arg(jntadr_ptr, &[input.nbody], cap(jntadr.len()))?,
                f32_arg(
                    body_pos_ptr,
                    &[input.body_pos.len(), input.nbody],
                    12,
                    cap(body_pos.len()),
                )?,
                f32_arg(
                    body_quat_ptr,
                    &[input.body_quat_wxyz.len(), input.nbody],
                    16,
                    cap(body_quat.len()),
                )?,
                i32_arg(jnt_type_ptr, &[input.njnt], cap(jnt_type.len()))?,
                i32_arg(jnt_qposadr_ptr, &[input.njnt], cap(jnt_qposadr.len()))?,
                f32_arg(
                    jnt_pos_ptr,
                    &[input.jnt_pos.len(), input.njnt],
                    12,
                    cap(jnt_pos.len()),
                )?,
                f32_arg(
                    jnt_axis_ptr,
                    &[input.jnt_axis.len(), input.njnt],
                    12,
                    cap(jnt_axis.len()),
                )?,
                i32_arg(
                    branches_ptr,
                    &[input.body_branches.len()],
                    cap(branches.len()),
                )?,
                i32_arg(
                    branch_start_ptr,
                    &[input.body_branch_start.len()],
                    cap(branch_start.len()),
                )?,
                f32_arg(qpos_ptr, &[input.worlds, input.nq], 4, cap(qpos.len()))?,
                f32_arg(
                    mocap_pos_ptr,
                    &[input.worlds, input.nmocap],
                    12,
                    cap(mocap_pos.len()),
                )?,
                f32_arg(
                    mocap_quat_ptr,
                    &[input.worlds, input.nmocap],
                    16,
                    cap(mocap_quat.len()),
                )?,
                f32_arg(xpos_ptr, &[input.worlds, input.nbody], 12, xpos_cap)?,
                f32_arg(xquat_ptr, &[input.worlds, input.nbody], 16, xquat_cap)?,
                f32_arg(xanchor_ptr, &[input.worlds, input.njnt], 12, xanchor_cap)?,
                f32_arg(xaxis_ptr, &[input.worlds, input.njnt], 12, xaxis_cap)?,
            ],
            "frozen-warp-kinematics-launch",
        )?;
        launch2(
            &stream,
            &matrices,
            [input.worlds, input.nbody],
            &[
                f32_arg(xquat_ptr, &[input.worlds, input.nbody], 16, xquat_cap)?,
                f32_arg(xmat_ptr, &[input.worlds, input.nbody], 36, xmat_cap)?,
            ],
            "frozen-warp-matrices-launch",
        )?;
        launch2(
            &stream,
            &inertial,
            [input.worlds, input.nbody],
            &[
                f32_arg(
                    body_ipos_ptr,
                    &[input.body_ipos.len(), input.nbody],
                    12,
                    cap(body_ipos.len()),
                )?,
                f32_arg(
                    body_iquat_ptr,
                    &[input.body_iquat_wxyz.len(), input.nbody],
                    16,
                    cap(body_iquat.len()),
                )?,
                f32_arg(xpos_ptr, &[input.worlds, input.nbody], 12, xpos_cap)?,
                f32_arg(xquat_ptr, &[input.worlds, input.nbody], 16, xquat_cap)?,
                f32_arg(xipos_ptr, &[input.worlds, input.nbody], 12, xipos_cap)?,
                f32_arg(ximat_ptr, &[input.worlds, input.nbody], 36, ximat_cap)?,
            ],
            "frozen-warp-inertial-launch",
        )?;
        launch2(
            &stream,
            &init,
            [input.worlds, input.nbody],
            &[
                f32_arg(
                    body_mass_ptr,
                    &[input.body_mass.len(), input.nbody],
                    4,
                    cap(body_mass.len()),
                )?,
                f32_arg(xipos_ptr, &[input.worlds, input.nbody], 12, xipos_cap)?,
                f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
            ],
            "frozen-warp-com-init-launch",
        )?;
        let mut tree_offset = tree_values.len();
        for level in input.body_tree_levels.iter().rev() {
            tree_offset -= level.len();
            let byte_offset = u64::try_from(tree_offset)
                .ok()
                .and_then(|value| value.checked_mul(4))
                .ok_or_else(|| artifact("input", "tree offset overflow"))?;
            let level_ptr = tree_ptr
                .checked_add(byte_offset)
                .ok_or_else(|| artifact("input", "tree pointer overflow"))?;
            launch2(
                &stream,
                &acc,
                [input.worlds, level.len()],
                &[
                    i32_arg(parent_ptr, &[input.nbody], cap(parent.len()))?,
                    f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
                    i32_arg(level_ptr, &[level.len()], cap(level.len()))?,
                    f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
                ],
                "frozen-warp-com-acc-launch",
            )?;
        }
        launch2(
            &stream,
            &div,
            [input.worlds, input.nbody],
            &[
                f32_arg(
                    body_subtreemass_ptr,
                    &[input.body_subtreemass.len(), input.nbody],
                    4,
                    cap(body_subtreemass.len()),
                )?,
                f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
                f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
            ],
            "frozen-warp-com-div-launch",
        )?;
        launch2(
            &stream,
            &cdof_kernel,
            [input.worlds, input.njnt],
            &[
                i32_arg(roots_ptr, &[input.nbody], cap(roots.len()))?,
                i32_arg(jnt_type_ptr, &[input.njnt], cap(jnt_type.len()))?,
                i32_arg(jnt_dofadr_ptr, &[input.njnt], cap(jnt_dofadr.len()))?,
                i32_arg(jnt_bodyid_ptr, &[input.njnt], cap(jnt_bodyid.len()))?,
                f32_arg(xmat_ptr, &[input.worlds, input.nbody], 36, xmat_cap)?,
                f32_arg(xanchor_ptr, &[input.worlds, input.njnt], 12, xanchor_cap)?,
                f32_arg(xaxis_ptr, &[input.worlds, input.njnt], 12, xaxis_cap)?,
                f32_arg(subtree_ptr, &[input.worlds, input.nbody], 12, subtree_cap)?,
                f32_arg(cdof_ptr, &[input.worlds, input.nv], 24, cdof_cap)?,
            ],
            "frozen-warp-cdof-launch",
        )?;

        drop(cdof_record);
        drop(subtree_record);
        drop(ximat_record);
        drop(xipos_record);
        drop(xmat_record);
        drop(xaxis_record);
        drop(xanchor_record);
        drop(xquat_record);
        drop(xpos_record);
        drop(tree_record);
        drop(body_subtreemass_record);
        drop(body_mass_record);
        drop(body_iquat_record);
        drop(body_ipos_record);
        drop(mocap_quat_record);
        drop(mocap_pos_record);
        drop(qpos_record);
        drop(branch_start_record);
        drop(branches_record);
        drop(jnt_axis_record);
        drop(jnt_pos_record);
        drop(jnt_bodyid_record);
        drop(jnt_dofadr_record);
        drop(jnt_qposadr_record);
        drop(jnt_type_record);
        drop(body_quat_record);
        drop(body_pos_record);
        drop(jntadr_record);
        drop(jntnum_record);
        drop(mocapid_record);
        drop(roots_record);
        drop(parent_record);
        drop(qpos0_record);

        let fields = [
            stream.clone_dtoh(&xpos).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xquat).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xipos).map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xmat).map_err(cuda_error("readback"))?,
            stream
                .clone_dtoh(&xanchor)
                .map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&xaxis).map_err(cuda_error("readback"))?,
            stream
                .clone_dtoh(&subtree)
                .map_err(cuda_error("readback"))?,
            stream.clone_dtoh(&cdof).map_err(cuda_error("readback"))?,
        ];
        let bits = fields
            .iter()
            .map(|values| values.iter().map(|value| value.to_bits()).collect())
            .collect();
        let [
            xpos,
            xquat_wxyz,
            xipos,
            xmat,
            xanchor,
            xaxis,
            subtree_com,
            cdof,
        ] = fields;
        Ok(FrozenWarpKinematicsReadback {
            xpos,
            xquat_wxyz,
            xipos,
            xmat,
            xanchor,
            xaxis,
            subtree_com,
            cdof,
            bits,
        })
    }
}
