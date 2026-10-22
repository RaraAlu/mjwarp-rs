"""Offline authoring scenario config for the U038 3D reference (OFFLINE ONLY)."""

from __future__ import annotations

import math

import numpy as np

from u038_3d_common import *

def quat_from_axis_angle(axis, angle) -> list[float]:
    axis = np.asarray(axis, dtype=np.float64)
    axis = axis / np.linalg.norm(axis)
    half = angle / 2.0
    q = np.concatenate(([math.cos(half)], axis * math.sin(half)))
    return [float(v) for v in q]


def qpos_world(index: int) -> list[float]:
    # free translation: all nonzero, distinct per world
    pos = [0.25 + 0.05 * index, -0.35 + 0.03 * index, 0.45 - 0.02 * index]
    free_q = quat_from_axis_angle((1.0, 0.6, -0.8), 0.7 + 0.11 * index)
    ball_q = quat_from_axis_angle((-0.4, 1.0, 0.75), -0.9 + 0.13 * index)
    hinge = 0.6 - 0.07 * index
    slide = 0.3 + 0.05 * index
    values = pos + free_q + ball_q + [hinge, slide]
    assert len(values) == NQ and all(v != 0.0 for v in values)
    return values


def body_rotation_quat(index: int) -> list[float]:
    axes = [(1.0, 1.0, 1.0), (1.0, -1.0, 1.0), (2.0, 1.0, -2.0), (-1.0, 2.0, 1.0)]
    return quat_from_axis_angle(axes[index], 0.55 + 0.17 * index)


def build_config() -> dict:
    body_parentid = [0, 0, 1, 2, 0]
    body_rootid = [0, 1, 1, 1, 4]
    body_pos = [
        [0.0, 0.0, 0.0],
        [0.2, 0.1, -0.1],
        [0.3, -0.2, 0.15],
        [-0.1, 0.25, 0.2],
        [0.4, 0.05, -0.2],
    ]
    body_quat = [[1.0, 0.0, 0.0, 0.0]] + [body_rotation_quat(i) for i in range(4)]
    joint = {
        "jnt_type": [0, 1, 3, 2],
        "jnt_bodyid": [1, 2, 3, 4],
        "jnt_qposadr": [0, 7, 11, 12],
        "jnt_dofadr": [0, 6, 9, 10],
        "jnt_pos": [[0.0, 0.0, 0.0], [0.05, -0.02, 0.03], [0.1, 0.02, -0.05], [0.02, 0.06, 0.04]],
        "jnt_axis": [[0.0, 0.0, 1.0], [0.36, 0.48, 0.8], [0.6, 0.8, 0.0], [1.0, 0.0, 0.0]],
    }
    qpos0 = [[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]]
    base_mass = np.asarray([0.0, 1.5, 2.5, 3.5, 5.0], dtype=np.float32)
    mass_rows = [base_mass, base_mass, base_mass]
    body_mass = np.concatenate(mass_rows).astype(np.float32)
    base_gc = np.asarray([0.0, 1.0, 0.5, 2.0, 0.25], dtype=np.float32)
    gravcomp_rows = [base_gc, base_gc, base_gc, base_gc, base_gc]
    body_gravcomp = np.concatenate(gravcomp_rows).astype(np.float32)
    subtree = np.zeros(NBODY, dtype=np.float32)
    for b in range(NBODY):
        node = b
        while True:
            subtree[node] += float(base_mass[b])
            parent = body_parentid[node]
            if parent == node or node == 0:
                break
            node = parent
    body_ipos = [[0.0, 0.0, 0.0]] + [
        [0.02 + 0.01 * i, -0.03 + 0.02 * i, 0.04 - 0.01 * i] for i in range(4)
    ]
    return {
        "schema": 1,
        "scope": "frozen-upstream-3d-qpos-kinematics-com-cdof",
        "upstream_revision": REVISION,
        "warp_version": "1.15.0",
        "worlds": WORLDS,
        "nbody": NBODY,
        "njnt": NJNT,
        "nq": NQ,
        "nv": NV,
        "nbranch": NBRANCH,
        "nmocap": 0,
        "qpos0": qpos0,
        "body_parentid": body_parentid,
        "body_rootid": body_rootid,
        "body_mocapid": [-1] * NBODY,
        "body_jntnum": [0, 1, 1, 1, 1],
        "body_jntadr": [-1, 0, 1, 2, 3],
        "body_pos": body_pos,
        "body_quat_wxyz": body_quat,
        **joint,
        "body_branches": [1, 2, 3, 4],
        "body_branch_start": [0, 3, 4],
        "qpos": [qpos_world(i) for i in range(WORLDS)],
        "mocap_pos": [],
        "mocap_quat_wxyz": [],
        "body_ipos": body_ipos,
        "body_iquat_wxyz": [[1.0, 0.0, 0.0, 0.0]] * NBODY,
        "body_mass": [row.tolist() for row in mass_rows],
        "body_subtreemass": [subtree.tolist()],
        "body_tree_levels": [[1, 4], [2], [3]],
        "legacy_static_input": "fixtures/gravity-upstream-abi/nonzero-kinematics/input.json",
        "legacy_static_input_sha256": "7b100efb0065eabe786c0d9abb202d23c83f66875c913fb7fa8d9ef13b3cfdad",
        "candidate_kinematics_inputs": False,
        "extra_authoring": {
            "opt_gravity": [[0.3, -9.81, 1.2], [0.3, -9.81, 1.2]],
            "body_mass_periods": [row.tolist() for row in mass_rows],
            "body_gravcomp_periods": [row.tolist() for row in gravcomp_rows],
        },
    }


def body_tree_levels(config) -> list[list[int]]:
    return config["body_tree_levels"]

