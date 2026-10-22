"""Offline authoring reference generation and packages for U038 3D (OFFLINE ONLY)."""

from __future__ import annotations

import importlib.metadata
import json
import pathlib
import re

import numpy as np

from u038_3d_common import *

def float_payload(name, values):
    flat = np.asarray(values, dtype=np.float32).reshape(-1)
    raw = flat.astype("<f4", copy=False).tobytes(order="C")
    return {
        "field": name,
        "count": int(flat.size),
        "little_endian_sha256": hashlib.sha256(raw).hexdigest(),
        "bits": [f"{int(value):08x}" for value in flat.view(np.uint32)],
    }


def run_upstream(config):
    import warp as wp
    import mujoco_warp._src.smooth as smooth

    device = "cuda:0"
    nbody, njnt, nq, nv = config["nbody"], config["njnt"], config["nq"], config["nv"]
    qpos_np = np.asarray(config["qpos"], dtype=np.float32).reshape(WORLDS, nq)
    qpos0 = wp.array(np.asarray(config["qpos0"], dtype=np.float32).reshape(-1, nq), dtype=float, device=device)
    body_pos = wp.array(np.asarray(config["body_pos"], dtype=np.float32).reshape(-1, nbody, 3), dtype=wp.vec3, device=device)
    body_quat = wp.array(np.asarray(config["body_quat_wxyz"], dtype=np.float32).reshape(-1, nbody, 4), dtype=wp.quat, device=device)
    jnt_pos = wp.array(np.asarray(config["jnt_pos"], dtype=np.float32).reshape(-1, njnt, 3), dtype=wp.vec3, device=device)
    jnt_axis = wp.array(np.asarray(config["jnt_axis"], dtype=np.float32).reshape(-1, njnt, 3), dtype=wp.vec3, device=device)
    body_ipos = wp.array(np.asarray(config["body_ipos"], dtype=np.float32).reshape(-1, nbody, 3), dtype=wp.vec3, device=device)
    body_iquat = wp.array(np.asarray(config["body_iquat_wxyz"], dtype=np.float32).reshape(-1, nbody, 4), dtype=wp.quat, device=device)
    body_mass = wp.array(np.asarray(config["body_mass"][0][:nbody], dtype=np.float32).reshape(1, nbody), dtype=float, device=device)
    body_subtreemass = wp.array(np.asarray(config["body_subtreemass"][0], dtype=np.float32).reshape(1, nbody), dtype=float, device=device)
    arrays = {
        "body_parentid": wp.array(config["body_parentid"], dtype=int, device=device),
        "body_rootid": wp.array(config["body_rootid"], dtype=int, device=device),
        "body_mocapid": wp.array(config["body_mocapid"], dtype=int, device=device),
        "body_jntnum": wp.array(config["body_jntnum"], dtype=int, device=device),
        "body_jntadr": wp.array(config["body_jntadr"], dtype=int, device=device),
        "jnt_type": wp.array(config["jnt_type"], dtype=int, device=device),
        "jnt_qposadr": wp.array(config["jnt_qposadr"], dtype=int, device=device),
        "jnt_dofadr": wp.array(config["jnt_dofadr"], dtype=int, device=device),
        "jnt_bodyid": wp.array(config["jnt_bodyid"], dtype=int, device=device),
        "body_branches": wp.array(config["body_branches"], dtype=int, device=device),
        "body_branch_start": wp.array(config["body_branch_start"], dtype=int, device=device),
        "qpos": wp.array(qpos_np, dtype=float, device=device),
        "mocap_pos": wp.array(np.zeros((WORLDS, 0, 3), dtype=np.float32), dtype=wp.vec3, device=device),
        "mocap_quat": wp.array(np.zeros((WORLDS, 0, 4), dtype=np.float32), dtype=wp.quat, device=device),
    }
    xpos_seed = np.full((WORLDS, nbody, 3), np.nan, dtype=np.float32)
    xpos_seed[:, 0, :] = 0.0
    xquat_seed = np.full((WORLDS, nbody, 4), np.nan, dtype=np.float32)
    xquat_seed[:, 0, :] = np.asarray([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    xpos = wp.array(xpos_seed, dtype=wp.vec3, device=device)
    xquat = wp.array(xquat_seed, dtype=wp.quat, device=device)
    xanchor = wp.full((WORLDS, njnt), value=np.nan, dtype=wp.vec3, device=device)
    xaxis = wp.full((WORLDS, njnt), value=np.nan, dtype=wp.vec3, device=device)
    xmat = wp.full((WORLDS, nbody), value=np.nan, dtype=wp.mat33, device=device)
    xipos = wp.full((WORLDS, nbody), value=np.nan, dtype=wp.vec3, device=device)
    ximat = wp.full((WORLDS, nbody), value=np.nan, dtype=wp.mat33, device=device)
    subtree_com = wp.zeros((WORLDS, nbody), dtype=wp.vec3, device=device)
    cdof = wp.zeros((WORLDS, nv), dtype=wp.spatial_vector, device=device)
    wp.launch(
        smooth._kinematics_branch,
        dim=(WORLDS, config["nbranch"]),
        inputs=[
            qpos0, arrays["body_parentid"], arrays["body_mocapid"], arrays["body_jntnum"],
            arrays["body_jntadr"], body_pos, body_quat, arrays["jnt_type"], arrays["jnt_qposadr"],
            jnt_pos, jnt_axis, arrays["body_branches"], arrays["body_branch_start"], arrays["qpos"],
            arrays["mocap_pos"], arrays["mocap_quat"],
        ],
        outputs=[xpos, xquat, xanchor, xaxis],
        device=device,
    )
    wp.launch(smooth._compute_body_matrices, dim=(WORLDS, nbody), inputs=[xquat], outputs=[xmat], device=device)
    wp.launch(smooth._compute_body_inertial_frames, dim=(WORLDS, nbody), inputs=[body_ipos, body_iquat, xpos, xquat], outputs=[xipos, ximat], device=device)
    wp.launch(smooth._subtree_com_init, dim=(WORLDS, nbody), inputs=[body_mass, xipos], outputs=[subtree_com], device=device)
    for level in reversed(config["body_tree_levels"]):
        tree = wp.array(level, dtype=int, device=device)
        wp.launch(smooth._subtree_com_acc, dim=(WORLDS, len(level)), inputs=[arrays["body_parentid"], subtree_com, tree], outputs=[subtree_com], device=device)
    wp.launch(smooth._subtree_div, dim=(WORLDS, nbody), inputs=[body_subtreemass, subtree_com], outputs=[subtree_com], device=device)
    wp.launch(smooth._cdof, dim=(WORLDS, njnt), inputs=[arrays["body_rootid"], arrays["jnt_type"], arrays["jnt_dofadr"], arrays["jnt_bodyid"], xmat, xanchor, xaxis, subtree_com], outputs=[cdof], device=device)
    wp.synchronize_device(device)
    return {
        "xpos": np.asarray(xpos.numpy(), dtype=np.float32),
        "xquat_wxyz": np.asarray(xquat.numpy(), dtype=np.float32),
        "xipos": np.asarray(xipos.numpy(), dtype=np.float32),
        "xmat": np.asarray(xmat.numpy(), dtype=np.float32),
        "xanchor": np.asarray(xanchor.numpy(), dtype=np.float32),
        "xaxis": np.asarray(xaxis.numpy(), dtype=np.float32),
        "subtree_com": np.asarray(subtree_com.numpy(), dtype=np.float32),
        "cdof": np.asarray(cdof.numpy(), dtype=np.float32),
    }


PER_JOINT_DOFS = {0: 6, 1: 3, 2: 1, 3: 1}


def dof_layout(config):
    """Serialized DOF layout with MuJoCo-style `dof_parentid` chain.

    The first DOF of a joint points at the last DOF of the nearest ancestor
    body that owns DOFs (-1 when none); the remaining DOFs point at the previous
    DOF of the same joint. This is the `dof_parentid` chain consumed by the
    pinned upstream `body_isdofancestor` precompute.
    """
    bodies = config["jnt_bodyid"]
    types = config["jnt_type"]
    body_last_dof = {}
    dof_bodyid = []
    dof_jntid = []
    dof_parentid = []
    for j, (body, kind) in enumerate(zip(bodies, types)):
        parent_body = config["body_parentid"][body]
        parent = -1
        while parent_body > 0:
            if parent_body in body_last_dof:
                parent = body_last_dof[parent_body]
                break
            parent_body = config["body_parentid"][parent_body]
        for _ in range(PER_JOINT_DOFS[kind]):
            dof_bodyid.append(body)
            dof_jntid.append(j)
            dof_parentid.append(parent)
            parent = len(dof_parentid) - 1
        body_last_dof[body] = len(dof_bodyid) - 1
    return dof_bodyid, dof_jntid, dof_parentid


def body_isdofancestor_rows(config):
    """Pinned upstream `put_model` semantics: which DOFs affect each body.

    Mirrors `mujoco_warp/_src/io.py` (revision 71da24d): for each body, walk from
    its last own DOF through `dof_parentid`; every visited DOF affects the body.
    The fixture previously stored the transposed relation (body ancestor of the
    DOF owner), which produced 70/77 gravity mismatches while the product chain
    was correct; the reversed direction is a fixture error.
    """
    dof_bodyid, _, dof_parentid = dof_layout(config)
    nbody, nv = config["nbody"], config["nv"]
    body_dofadr = [-1] * nbody
    body_dofnum = [0] * nbody
    for j, body in enumerate(config["jnt_bodyid"]):
        body_dofadr[body] = config["jnt_dofadr"][j]
        body_dofnum[body] = PER_JOINT_DOFS[config["jnt_type"][j]]
    rows = [[0] * nv for _ in range(nbody)]
    for bodyid in range(nbody):
        b = bodyid
        while b > 0 and body_dofnum[b] == 0:
            b = config["body_parentid"][b]
        if body_dofnum[b] == 0:
            continue
        dofid = body_dofadr[b] + body_dofnum[b] - 1
        while dofid >= 0:
            rows[bodyid][dofid] = 1
            dofid = dof_parentid[dofid]
    return rows


def native_isdofancestor(config):
    """Authoritative matrix from pinned upstream `mujoco_warp.put_model`.

    Builds the equivalent native MuJoCo model from the same config, converts it
    with the pinned upstream converter and returns `body_isdofancestor` (nv
    columns). Used as the natural validation of the fixture matrix; generation
    aborts if the two disagree.
    """
    import mujoco
    import mujoco_warp

    def block(body):
        children = [b for b in range(1, config["nbody"]) if config["body_parentid"][b] == body]
        inner = "".join(block(child) for child in children)
        pos = " ".join(f"{v:.17g}" for v in config["body_pos"][body])
        quat = " ".join(f"{v:.17g}" for v in config["body_quat_wxyz"][body])
        joint = ""
        for j, joint_body in enumerate(config["jnt_bodyid"]):
            if joint_body != body:
                continue
            kind = config["jnt_type"][j]
            if kind == 0:
                joint = "<freejoint/>"
            else:
                name = {1: "ball", 2: "hinge", 3: "slide"}[kind]
                jpos = " ".join(f"{v:.17g}" for v in config["jnt_pos"][j])
                axis = " ".join(f"{v:.17g}" for v in config["jnt_axis"][j])
                joint = f'<joint type="{name}" pos="{jpos}" axis="{axis}"/>'
        mass = float(config["body_mass"][0][body])
        inertial = (
            f'<inertial pos="{" ".join(f"{v:.17g}" for v in config["body_ipos"][body])}"'
            f' quat="1 0 0 0" mass="{mass:.17g}" diaginertia="1 1 1"/>'
        )
        return f'<body name="b{body}" pos="{pos}" quat="{quat}">{joint}{inertial}{inner}</body>'

    roots = [b for b in range(1, config["nbody"]) if config["body_parentid"][b] == 0]
    xml = "<mujoco><worldbody>" + "".join(block(b) for b in roots) + "</worldbody></mujoco>"
    mjm = mujoco.MjModel.from_xml_string(xml)
    if [int(v) for v in mjm.dof_parentid] != list(dof_layout(config)[2]):
        raise RuntimeError("native dof_parentid does not match the fixture layout")
    converted = mujoco_warp.put_model(mjm)
    matrix = converted.body_isdofancestor.numpy()[:, : config["nv"]]
    provenance = {
        "mujoco": mujoco.__version__,
        "mujoco_warp": importlib.metadata.version("mujoco-warp"),
        "converter": "mujoco_warp.put_model (body_isdofancestor precompute)",
        "io_source_sha256": digest(UPSTREAM / "mujoco_warp" / "_src" / "io.py"),
        "dof_parentid": [int(v) for v in mjm.dof_parentid],
    }
    return [[int(v) for v in row] for row in matrix], provenance


def build_model_package(config, ancestor):
    dof_bodyid, _, _ = dof_layout(config)
    nbody, nv = config["nbody"], config["nv"]
    extra = config["extra_authoring"]
    return {
        "schema": 1,
        "scope": "gravity-upstream-3d-model-package",
        "upstream_revision": REVISION,
        "warp_version": "1.15.0",
        "dimensions": {"worlds": WORLDS, "nbody": nbody, "njnt": NJNT, "nq": NQ, "nv": nv, "branches": NBRANCH},
        "model": {
            "opt_gravity": [float(np.float32(value)) for row in extra["opt_gravity"] for value in row],
            "body_parentid": config["body_parentid"],
            "body_rootid": config["body_rootid"],
            "body_mass": [value for row in extra["body_mass_periods"] for value in row],
            "body_gravcomp": [value for row in extra["body_gravcomp_periods"] for value in row],
            "dof_bodyid": dof_bodyid,
            "body_isdofancestor": [value for row in ancestor for value in row],
        },
        "parameter_periods": {"gravity": GRAVITY_PERIODS, "body_mass": MASS_PERIODS, "body_gravcomp": GRAVCOMP_PERIODS},
        "boundaries": {"three_d_rotation_coverage": True, "full_state_space_accepted": False, "np03_complete": False},
    }


def anchor_digest(anchor_rel: str) -> str:
    anchor = ROOT / anchor_rel
    text = anchor.read_text(encoding="utf-8")
    rels = re.findall(r'include_str!\("([^"]+)"\)', text)
    if not rels:
        raise RuntimeError(f"no include_str anchors: {anchor_rel}")
    parts = []
    for rel in rels:
        parts.append((anchor.parent / rel).read_bytes().decode("utf-8"))
    return digest_text("\n".join(parts).replace("\r\n", "\n"))


def build_product_package(config, model_package, output):
    world, nbody, njnt, nq, nv = config["worlds"], config["nbody"], config["njnt"], config["nq"], config["nv"]
    expected = {
        "rigid_values": world * (nbody * 3 + nbody * 4 + nbody * 9 + nbody * 3 + njnt * 3 + njnt * 3),
        "subtree_com_values": world * nbody * 3,
        "cdof_values": world * nv * 6,
        "gravity_values": world * nv,
    }
    expected["total_values"] = sum(expected.values())
    package = {
        "schema": 1,
        "scope": "gravity-live-product-3d-qpos-chain",
        "upstream_revision": REVISION,
        "warp_version": "1.15.0",
        "model_package": {
            "path": "fixtures/gravity-upstream-abi/three-d-model-package.json",
            "sha256": digest(ROOT / "fixtures" / "gravity-upstream-abi" / "three-d-model-package.json"),
        },
        "kinematics": {
            "directory": "fixtures/gravity-upstream-abi/three-d-kinematics",
            "receipt_sha256": digest(HERE / "receipt.json"),
            "input_sha256": digest(HERE / "input.json"),
            "output_sha256": digest(HERE / "output.json"),
            "raw_bits_sha256": digest(HERE / "raw-bits.json"),
        },
        "gravity": {
            "directory": "fixtures/gravity-upstream-abi/generated",
            "receipt_sha256": "547f9881030a15838d108900b39f50c0691b8e085beb0bab9d48684d9eff5eea",
        },
        "product_sources": {
            "physics_core": {"path": "src/physics/mod.rs", "sha256": anchor_digest("docs/development-evidence/t4-resident-anchors/physics.rs")},
            "resident": {"path": "src/physics/resident.rs", "sha256": anchor_digest("docs/evidence/t4-g03-retest/resident-source-anchor.rs")},
            "gravity_compensation": {"path": "src/physics/gravity_compensation.rs", "sha256": digest(ROOT / "src/physics/gravity_compensation.rs")},
        },
        "dimensions": {"worlds": world, "nbody": nbody, "njnt": njnt, "nq": nq, "nv": nv, "branches": NBRANCH},
        "parameter_periods": {"gravity": GRAVITY_PERIODS, "body_mass": MASS_PERIODS, "body_gravcomp": GRAVCOMP_PERIODS},
        "expected": {**expected, "absolute_tolerance": 0.000002},
        "boundaries": {
            "all_qpos_nonzero": True,
            "nonzero_rotation_coverage": True,
            "three_d_rotation_coverage": True,
            "combined_joint_types": True,
            "same_model_and_qpos": True,
            "same_resident_source": True,
            "candidate_kinematics_inputs": False,
            "product_python_dependency": False,
            "full_state_space_accepted": False,
            "np03_complete": False,
            "full_regression_run": False,
            "performance_accepted": False,
        },
    }
    write_json(ROOT / "fixtures" / "gravity-upstream-abi" / "three-d-package.json", package)


