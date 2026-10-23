"""Offline authoring: U038 reference fixture scenarios and topology validation (OFFLINE ONLY)."""

from __future__ import annotations

import numpy as np

import u038_reference_common as common


def quat_identity() -> list[float]:
    return [1.0, 0.0, 0.0, 0.0]


def fluid_config(worlds: int = 9) -> dict:
    nbody, njnt, nq, nv, ngeom = 5, 2, 2, 2, 5
    body_parentid = [0, 0, 1, 2, 1]
    body_pos = [
        [0.0, 0.0, 0.0],
        [0.4, -0.2, 0.3],
        [0.8, 0.1, -0.15],
        [0.35, -0.25, 0.2],
        [-0.3, 0.45, 0.1],
    ]
    body_ipos = [
        [0.0, 0.0, 0.0],
        [0.2, 0.05, 0.0],
        [-0.1, 0.15, 0.05],
        [0.05, -0.1, 0.12],
        [0.0, 0.0, 0.0],
    ]
    body_mass = [0.0, 6.0, 4.0, 2.0, 3.0]
    body_inertia = [
        [0.0, 0.0, 0.0],
        [5.0, 4.0, 3.0],
        [3.0, 2.5, 2.0],
        [2.0, 1.5, 1.0],
        [0.5, 0.4, 0.3],
    ]
    geom_bodyid = [1, 1, 1, 2, 4]
    geom_pos = [
        [0.2, 0.0, 0.1],
        [-0.15, 0.05, 0.0],
        [0.0, -0.2, 0.15],
        [0.1, 0.0, -0.1],
        [0.0, 0.0, 0.0],
    ]
    geom_quat = [
        [1.0, 0.0, 0.0, 0.0],
        [0.9238795, 0.0, 0.38268343, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.70710677, 0.0, 0.0, 0.70710677],
        [1.0, 0.0, 0.0, 0.0],
    ]
    geom_type = [2, 3, 4, 5, 6]
    geom_size_base = [
        [0.2, 0.2, 0.2],
        [0.1, 0.25, 0.1],
        [0.3, 0.2, 0.15],
        [0.12, 0.22, 0.12],
        [0.18, 0.14, 0.1],
    ]
    geom_size_periods = []
    for row in range(4):
        delta = row * 0.015625
        geom_size_periods.append(
            [
                0.2 + delta, 0.2, 0.2,
                0.1, 0.25 + delta, 0.1,
                0.3, 0.2, 0.15 + delta,
                0.12, 0.22, 0.12,
                0.18, 0.14, 0.1 + delta,
            ]
        )
    geom_pos_periods = []
    for row in range(3):
        delta = row * 0.03125
        geom_pos_periods.append(
            [
                0.2 + delta, 0.0, 0.1,
                -0.15, 0.05 + delta, 0.0,
                0.0, -0.2, 0.15,
                0.1, 0.0, -0.1,
                0.0, 0.0, 0.0,
            ]
        )
    geom_quat_periods = [
        [1.0, 0.0, 0.0, 0.0, 0.9238795, 0.0, 0.38268343, 0.0, 1.0, 0.0, 0.0, 0.0,
         0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0, 0.8660254, 0.0, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0,
         0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0],
    ]
    coefficients = [1.0, 0.75, 0.0, 1.0, 0.5]
    base = [0.8, 0.35, 0.2, 0.45, 0.3, 0.04, 0.06, 0.08, 0.02, 0.03, 0.05]
    geom_fluid = []
    for coefficient in coefficients:
        field = [coefficient] + base
        if coefficient == 0.0:
            field = [coefficient] + [1.0e6] * 11
        geom_fluid.extend(field)
    qpos = [[0.07 * w, -0.045 * w] for w in range(worlds)]
    qvel = [
        [0.0, 0.0] if w == 0 else [0.12 + 0.025 * w, -0.08 + 0.0175 * w]
        for w in range(worlds)
    ]
    config = {
        "scenario": "fluid",
        "worlds": worlds,
        "nbody": nbody,
        "njnt": njnt,
        "nq": nq,
        "nv": nv,
        "ngeom": ngeom,
        "qpos0": [0.0, 0.0],
        "body_parentid": body_parentid,
        "body_pos": body_pos,
        "body_quat_wxyz": [quat_identity() for _ in range(nbody)],
        "body_ipos": body_ipos,
        "body_iquat_wxyz": [quat_identity() for _ in range(nbody)],
        "jnt_type": [3, 3],
        "jnt_bodyid": [1, 2],
        "jnt_qposadr": [0, 1],
        "jnt_dofadr": [0, 1],
        "jnt_pos": [[0.1, 0.0, 0.0], [0.0, 0.15, -0.05]],
        "jnt_axis": [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
        "geom_bodyid": geom_bodyid,
        "geom_pos": geom_pos,
        "geom_quat": geom_quat,
        "geom_type": geom_type,
        "geom_size": geom_size_base,
        "geom_size_periods": geom_size_periods,
        "geom_pos_periods": geom_pos_periods,
        "geom_quat_periods": geom_quat_periods,
        "body_mass": [body_mass],
        "body_inertia": [[v for body in body_inertia for v in body]],
        "wind": [[0.0, 0.0, 0.0], [0.25, -0.125, 0.0625]],
        "density": [1.0, 0.5, 1.25],
        "viscosity": [0.2, 0.125, 0.25, 0.375, 0.5],
        "geom_fluid": geom_fluid,
        "body_fluid_ellipsoid": [False, True, True, True, True],
        "qpos": qpos,
        "qvel": qvel,
        "periods": {
            "body_mass": 1,
            "body_inertia": 1,
            "geom_size": 4,
            "geom_pos": 3,
            "geom_quat": 2,
            "wind": 2,
            "density": 3,
            "viscosity": 5,
        },
    }
    config["body_subtreemass"] = common.subtreemass_by_body(config)
    return config


def atomic_config(worlds: int = 7) -> dict:
    nbody, njnt, nq, nv = 5, 4, 4, 4
    config = {
        "scenario": "atomic",
        "worlds": worlds,
        "nbody": nbody,
        "njnt": njnt,
        "nq": nq,
        "nv": nv,
        "ngeom": 0,
        "qpos0": [0.0, 0.0, 0.0, 0.0],
        "body_parentid": [0, 0, 1, 2, 0],
        "body_pos": [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.87, -0.19, 0.13],
            [0.71, 0.23, -0.11],
            [-0.63, 0.31, 0.17],
        ],
        "body_quat_wxyz": [quat_identity() for _ in range(nbody)],
        "body_ipos": [
            [0.0, 0.0, 0.0],
            [0.37, 0.11, 0.0],
            [0.53, -0.19, 0.07],
            [0.29, 0.13, -0.11],
            [-0.41, 0.23, 0.05],
        ],
        "body_iquat_wxyz": [quat_identity() for _ in range(nbody)],
        "jnt_type": [3, 3, 3, 3],
        "jnt_bodyid": [1, 2, 3, 4],
        "jnt_qposadr": [0, 1, 2, 3],
        "jnt_dofadr": [0, 1, 2, 3],
        "jnt_pos": [[0.0, 0.0, 0.0] for _ in range(njnt)],
        "jnt_axis": [[0.0, 0.0, 1.0] for _ in range(njnt)],
        # 非精确归约：质量/重补偿为二进制不可精确表示的十进制值，
        # 多体贡献之和依赖加法顺序（上游 atom.add 与产品固定顺序可不同）。
        "body_mass": [[0.0, 0.3, 0.7, 1.1, 0.5]],
        "body_inertia": [[0.0] * 15],
        "gravity": [[0.03, -0.29, 0.07], [-0.05, -0.13, 0.02]],
        "body_gravcomp": [
            [0.0, 0.6, 0.9, 1.3, 0.4],
            [0.0, 1.1, 0.3, 0.7, 1.7],
            [0.0, 0.8, 1.4, 0.2, 0.9],
            [0.0, 1.5, 0.4, 1.1, 0.3],
            [0.0, 0.2, 1.2, 0.6, 1.4],
        ],
        "jnt_actgravcomp": [0, 0, 0, 0],
        "qpos": [[0.0] * 4 for _ in range(worlds)],
        "qvel": [[0.0] * 4 for _ in range(worlds)],
        "periods": {"body_mass": 1, "body_gravcomp": 5, "gravity": 2},
    }
    config["body_subtreemass"] = common.subtreemass_by_body(config)
    return config


PER_JOINT_DOFS = {0: 6, 1: 3, 2: 1, 3: 1}


def expected_dof_bodyid(config: dict) -> list[int]:
    dof_bodyid = [-1] * config["nv"]
    for joint, kind in enumerate(config["jnt_type"]):
        for step in range(PER_JOINT_DOFS[kind]):
            dof_bodyid[config["jnt_dofadr"][joint] + step] = config["jnt_bodyid"][joint]
    if any(value < 0 for value in dof_bodyid):
        raise RuntimeError(f"uncovered dofs in fixture jnt layout: {dof_bodyid}")
    return dof_bodyid


def validate_topology(config: dict, native: dict) -> dict:
    expected = {
        "body_parentid": config["body_parentid"],
        "jnt_type": config["jnt_type"],
        "jnt_qposadr": config["jnt_qposadr"],
        "jnt_dofadr": config["jnt_dofadr"],
        "jnt_bodyid": config["jnt_bodyid"],
        "dof_bodyid": expected_dof_bodyid(config),
    }
    failures = {
        key: {"fixture": config.get(key), "native": native.get(key)}
        for key in expected
        if expected[key] is not None and native.get(key) != expected[key]
    }
    if config["ngeom"]:
        geom_count = [0] * config["nbody"]
        for body in config["geom_bodyid"]:
            geom_count[body] += 1
        if native["body_geomnum"] != geom_count:
            failures["body_geomnum"] = {
                "fixture": geom_count,
                "native": native["body_geomnum"],
            }
    if failures:
        raise RuntimeError(f"fixture topology differs from upstream put_model: {failures}")
    return {
        "exact_match": True,
        "mujoco_version": native["mujoco_version"],
        "mujoco_warp_version": native["mujoco_warp_version"],
        "io_source_sha256": native["io_source_sha256"],
        "source_anchor": "mujoco_warp/_src/io.py: put_model structural precompute",
        "body_branches": native["body_branches"],
        "body_branch_start": native["body_branch_start"],
        "body_tree_levels": native["body_tree_levels"],
        "body_isdofancestor": native["body_isdofancestor"],
        "dof_parentid": [int(v) for v in native["mjm"].dof_parentid],
        "body_subtreemass": config["body_subtreemass"],
        "body_subtreemass_source": "derived from fixture base body_mass row (MJCF order, parent<child)",
    }


def finalize_config(config: dict, native: dict) -> dict:
    config["body_rootid"] = native["body_rootid"]
    config["body_geomnum"] = native["body_geomnum"]
    config["body_geomadr"] = native["body_geomadr"]
    config["dof_bodyid"] = native["dof_bodyid"]
    config["body_isdofancestor"] = native["body_isdofancestor"]
    config["body_branches"] = native["body_branches"]
    config["body_branch_start"] = native["body_branch_start"]
    config["body_tree_levels"] = native["body_tree_levels"]
    return config


def geom_semiaxes(size, geom_type: int):
    if geom_type == 2:
        return np.array([size[0], size[0], size[0]], dtype=np.float64)
    if geom_type == 3:
        return np.array([size[0], size[0], size[1] + size[0]], dtype=np.float64)
    if geom_type == 5:
        return np.array([size[0], size[0], size[1]], dtype=np.float64)
    return np.asarray(size, dtype=np.float64)
