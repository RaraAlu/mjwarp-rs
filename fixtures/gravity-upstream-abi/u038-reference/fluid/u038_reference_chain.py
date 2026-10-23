"""Offline authoring: frozen-upstream smooth chain and passive kernel launchers (OFFLINE ONLY)."""

from __future__ import annotations

import numpy as np

from u038_reference_common import *


def run_kinematics_chain(wp, smooth, config: dict, native: dict, qpos: np.ndarray, qvel: np.ndarray):
    """Run the frozen-upstream smooth chain for the fixture scenario.

    Returns numpy float32 arrays for xpos/xquat/xmat/xipos/ximat/xanchor/xaxis/
    subtree_com/cdof/geom_xpos/geom_xmat/cvel.
    """
    device = "cuda:0"
    worlds, nbody, nv = config["worlds"], config["nbody"], config["nv"]
    njnt = config["njnt"]
    nq = config["nq"]
    qpos0 = wp_array(wp, config["qpos0"], float, (-1, nq))
    body_pos = wp_array(wp, config["body_pos"], wp.vec3, (-1, nbody, 3))
    body_quat = wp_array(wp, config["body_quat_wxyz"], wp.quat, (-1, nbody, 4))
    jnt_pos = wp_array(wp, config["jnt_pos"], wp.vec3, (-1, njnt, 3))
    jnt_axis = wp_array(wp, config["jnt_axis"], wp.vec3, (-1, njnt, 3))
    body_ipos = wp_array(wp, config["body_ipos"], wp.vec3, (-1, nbody, 3))
    body_iquat = wp_array(wp, config["body_iquat_wxyz"], wp.quat, (-1, nbody, 4))
    body_mass = wp_array(wp, config["body_mass"], float, (-1, nbody))
    body_subtreemass = wp_array(wp, config["body_subtreemass"], float, (1, nbody))
    qpos_array = wp_array(wp, qpos, float, (worlds, nq))
    qvel_array = wp_array(wp, qvel, float, (worlds, nv))
    zeros_mocap_pos = wp.zeros((worlds, 0), dtype=wp.vec3, device=device)
    zeros_mocap_quat = wp.zeros((worlds, 0), dtype=wp.quat, device=device)
    xpos_seed = np.full((worlds, nbody, 3), np.nan, dtype=np.float32)
    xpos_seed[:, 0, :] = 0.0
    xquat_seed = np.full((worlds, nbody, 4), np.nan, dtype=np.float32)
    xquat_seed[:, 0, :] = np.asarray([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    xpos = wp.array(xpos_seed, dtype=wp.vec3, device=device)
    xquat = wp.array(xquat_seed, dtype=wp.quat, device=device)
    xanchor = wp.full((worlds, njnt), value=np.nan, dtype=wp.vec3, device=device)
    xaxis = wp.full((worlds, njnt), value=np.nan, dtype=wp.vec3, device=device)
    xmat = wp.full((worlds, nbody), value=np.nan, dtype=wp.mat33, device=device)
    xipos = wp.full((worlds, nbody), value=np.nan, dtype=wp.vec3, device=device)
    ximat = wp.full((worlds, nbody), value=np.nan, dtype=wp.mat33, device=device)
    subtree_com = wp.zeros((worlds, nbody), dtype=wp.vec3, device=device)
    cdof = wp.zeros((worlds, nv), dtype=wp.spatial_vector, device=device)
    cdof_dot = wp.zeros((worlds, nv), dtype=wp.spatial_vector, device=device)
    cvel = wp.zeros((worlds, nbody), dtype=wp.spatial_vector, device=device)
    geom_xpos = wp.zeros((worlds, config["ngeom"]), dtype=wp.vec3, device=device)
    geom_xmat = wp.zeros((worlds, config["ngeom"]), dtype=wp.mat33, device=device)

    wp.launch(
        smooth._kinematics_branch,
        dim=(worlds, len(native["body_branch_start"]) - 1),
        inputs=[
            qpos0,
            wp.array(native["body_parentid"], dtype=int, device=device),
            wp.array(native["body_mocapid"], dtype=int, device=device),
            wp.array(native["body_jntnum"], dtype=int, device=device),
            wp.array(native["body_jntadr"], dtype=int, device=device),
            body_pos,
            body_quat,
            wp.array(native["jnt_type"], dtype=int, device=device),
            wp.array(native["jnt_qposadr"], dtype=int, device=device),
            jnt_pos,
            jnt_axis,
            wp.array(native["body_branches"], dtype=int, device=device),
            wp.array(native["body_branch_start"], dtype=int, device=device),
            qpos_array,
            zeros_mocap_pos,
            zeros_mocap_quat,
        ],
        outputs=[xpos, xquat, xanchor, xaxis],
        device=device,
    )
    wp.launch(
        smooth._compute_body_matrices,
        dim=(worlds, nbody),
        inputs=[xquat],
        outputs=[xmat],
        device=device,
    )
    wp.launch(
        smooth._compute_body_inertial_frames,
        dim=(worlds, nbody),
        inputs=[body_ipos, body_iquat, xpos, xquat],
        outputs=[xipos, ximat],
        device=device,
    )
    wp.launch(
        smooth._subtree_com_init,
        dim=(worlds, nbody),
        inputs=[body_mass, xipos],
        outputs=[subtree_com],
        device=device,
    )
    for level in reversed(native["body_tree_levels"]):
        tree = wp.array(level, dtype=int, device=device)
        wp.launch(
            smooth._subtree_com_acc,
            dim=(worlds, len(level)),
            inputs=[
                wp.array(native["body_parentid"], dtype=int, device=device),
                subtree_com,
                tree,
            ],
            outputs=[subtree_com],
            device=device,
        )
    wp.launch(
        smooth._subtree_div,
        dim=(worlds, nbody),
        inputs=[body_subtreemass, subtree_com],
        outputs=[subtree_com],
        device=device,
    )
    wp.launch(
        smooth._cdof,
        dim=(worlds, njnt),
        inputs=[
            wp.array(native["body_rootid"], dtype=int, device=device),
            wp.array(native["jnt_type"], dtype=int, device=device),
            wp.array(native["jnt_dofadr"], dtype=int, device=device),
            wp.array(native["jnt_bodyid"], dtype=int, device=device),
            xmat,
            xanchor,
            xaxis,
            subtree_com,
        ],
        outputs=[cdof],
        device=device,
    )
    if config["ngeom"]:
        wp.launch(
            smooth._geom_local_to_global,
            dim=(worlds, config["ngeom"]),
            inputs=[
                wp.array(native["body_rootid"], dtype=int, device=device),
                wp.array(native["body_weldid"], dtype=int, device=device),
                wp.array(native["body_mocapid"], dtype=int, device=device),
                wp.array(config["geom_bodyid"], dtype=int, device=device),
                wp_array(wp, config["geom_pos_periods"], wp.vec3, (-1, config["ngeom"], 3)),
                wp_array(wp, config["geom_quat_periods"], wp.quat, (-1, config["ngeom"], 4)),
                xpos,
                xquat,
            ],
            outputs=[geom_xpos, geom_xmat],
            device=device,
        )
    wp.launch(smooth._comvel_root, dim=(worlds, 6), inputs=[], outputs=[cvel], device=device)
    wp.launch(
        smooth._comvel_branch,
        dim=(worlds, len(native["body_branch_start"]) - 1),
        inputs=[
            wp.array(native["body_parentid"], dtype=int, device=device),
            wp.array(native["body_jntnum"], dtype=int, device=device),
            wp.array(native["body_jntadr"], dtype=int, device=device),
            wp.array(native["body_dofadr"], dtype=int, device=device),
            wp.array(native["jnt_type"], dtype=int, device=device),
            wp.array(native["body_branches"], dtype=int, device=device),
            wp.array(native["body_branch_start"], dtype=int, device=device),
            qvel_array,
            cdof,
        ],
        outputs=[cvel, cdof_dot],
        device=device,
    )
    wp.synchronize_device(device)
    fields = {
        "xpos": xpos,
        "xquat_wxyz": xquat,
        "xmat": xmat,
        "xipos": xipos,
        "ximat": ximat,
        "xanchor": xanchor,
        "xaxis": xaxis,
        "subtree_com": subtree_com,
        "cdof": cdof,
        "geom_xpos": geom_xpos,
        "geom_xmat": geom_xmat,
        "cvel": cvel,
    }
    return {
        name: np.asarray(array.numpy(), dtype=np.float32) for name, array in fields.items()
    }


def launch_fluid(wp, passive, config: dict, intermediates: dict):
    device = "cuda:0"
    worlds, nbody = config["worlds"], config["nbody"]
    periods = config["periods"]
    wind = wp_array(wp, config["wind"], wp.vec3, (-1, 3))
    density = wp_array(wp, config["density"], float, (-1,))
    viscosity = wp_array(wp, config["viscosity"], float, (-1,))
    body_rootid = wp.array(config["body_rootid"], dtype=int, device=device)
    body_geomnum = wp.array(config["body_geomnum"], dtype=int, device=device)
    body_geomadr = wp.array(config["body_geomadr"], dtype=int, device=device)
    body_mass = wp_array(wp, config["body_mass"], float, (periods["body_mass"], nbody))
    body_inertia = wp_array(wp, config["body_inertia"], wp.vec3, (periods["body_inertia"], nbody, 3))
    geom_type = wp.array(config["geom_type"], dtype=int, device=device)
    geom_size = wp_array(wp, config["geom_size_periods"], wp.vec3, (periods["geom_size"], config["ngeom"], 3))
    geom_fluid = wp.array(
        np_f32(config["geom_fluid"], (config["ngeom"], 12)), dtype=float, device=device
    )
    body_fluid_ellipsoid = wp.array(
        np.asarray(config["body_fluid_ellipsoid"], dtype=bool), dtype=bool, device=device
    )
    xipos = wp_array(wp, intermediates["xipos"], wp.vec3, (worlds, nbody, 3))
    ximat = wp_array(wp, intermediates["ximat"], wp.mat33, (worlds, nbody, 3, 3))
    geom_xpos = wp_array(wp, intermediates["geom_xpos"], wp.vec3, (worlds, config["ngeom"], 3))
    geom_xmat = wp_array(wp, intermediates["geom_xmat"], wp.mat33, (worlds, config["ngeom"], 3, 3))
    subtree_com = wp_array(wp, intermediates["subtree_com"], wp.vec3, (worlds, nbody, 3))
    cvel = wp_array(wp, intermediates["cvel"], wp.spatial_vector, (worlds, nbody, 6))
    out = wp.zeros((worlds, nbody), dtype=wp.spatial_vector, device=device)
    wp.launch(
        passive._fluid_force,
        dim=(worlds, nbody),
        inputs=[
            wind,
            density,
            viscosity,
            body_rootid,
            body_geomnum,
            body_geomadr,
            body_mass,
            body_inertia,
            geom_type,
            geom_size,
            geom_fluid,
            body_fluid_ellipsoid,
            xipos,
            ximat,
            geom_xpos,
            geom_xmat,
            subtree_com,
            cvel,
        ],
        outputs=[out],
        device=device,
    )
    wp.synchronize_device(device)
    return np.asarray(out.numpy(), dtype=np.float32).reshape(worlds, nbody, 6)


def launch_gravity(wp, passive, config: dict, intermediates: dict):
    device = "cuda:0"
    worlds, nbody, nv = config["worlds"], config["nbody"], config["nv"]
    periods = config["periods"]
    gravity = wp_array(wp, config["gravity"], wp.vec3, (-1,))
    body_parentid = wp.array(config["body_parentid"], dtype=int, device=device)
    body_rootid = wp.array(config["body_rootid"], dtype=int, device=device)
    body_mass = wp_array(wp, config["body_mass"], float, (periods["body_mass"], nbody))
    body_gravcomp = wp_array(
        wp, config["body_gravcomp"], float, (periods["body_gravcomp"], nbody)
    )
    dof_bodyid = wp.array(config["dof_bodyid"], dtype=int, device=device)
    ancestor = wp.array(
        np.asarray(config["body_isdofancestor"], dtype=np.int32), dtype=int, device=device
    )
    xipos = wp_array(wp, intermediates["xipos"], wp.vec3, (worlds, nbody, 3))
    subtree_com = wp_array(wp, intermediates["subtree_com"], wp.vec3, (worlds, nbody, 3))
    cdof = wp_array(wp, intermediates["cdof"], wp.spatial_vector, (worlds, nv, 6))
    out = wp.zeros((worlds, nv), dtype=float, device=device)
    wp.launch(
        passive._gravity_force,
        dim=(worlds, nbody - 1, nv),
        inputs=[
            gravity,
            body_parentid,
            body_rootid,
            body_mass,
            body_gravcomp,
            dof_bodyid,
            ancestor,
            xipos,
            subtree_com,
            cdof,
        ],
        outputs=[out],
        device=device,
    )
    wp.synchronize_device(device)
    return np.asarray(out.numpy(), dtype=np.float32).reshape(worlds, nv)
