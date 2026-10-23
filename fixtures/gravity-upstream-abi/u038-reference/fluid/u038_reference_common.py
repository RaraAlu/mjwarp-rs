"""Offline authoring common helpers for the U038 independent fluid/atomic reference.

OFFLINE ONLY. Not imported, built or run by product tests. Product tests read the
static fixtures and never start Python.
"""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import pathlib
import re
import shutil
import subprocess
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
FIXTURES = ROOT / "fixtures" / "gravity-upstream-abi"
OUT_ROOT = FIXTURES / "u038-reference"
CACHE_ROOT = ROOT / "target" / "mjwarp-parallel" / "u038-reference-cache"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
WARP_VERSION = "1.15.0"
EXPECTED_VERSIONS = {
    "python": "3.12.13",
    "mujoco": "3.15.0",
    "warp": "1.15.0",
    "numpy": "2.5.4",
    "mujoco-warp": "3.15.0",
}
WARP_DLL_SHA256 = "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6"
# smooth 模块冻结产物（fixtures/gravity-upstream-abi/independent-kinematics）
SMOOTH_CU_SHA256 = "844dca14a5932827729313a2500f343bdff950b8cdb6170aed1c5973e0ff290e"
SMOOTH_META_SHA256 = "2bb1bcdb12cbcce6143af26da5798c047ed628d4817c4b76358fc646db0e3cc8"
SMOOTH_PTX_SHA256 = "4179acd01fe8e7064fa97566cfe4158dd9ad4b89761dcab0da88001d9b521f57"
# passive 模块冻结产物（fixtures/gravity-upstream-abi/generated）
PASSIVE_CU_SHA256 = "dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d"
PASSIVE_META_SHA256 = "4a80327bc21a875a63b9cf3dfc43386ed40119012357560339f071ac50ea461c"
PASSIVE_PTX_SHA256 = "d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70"
SMOOTH_STEM = "wp_mujoco_warp._src.smooth_ca013d6"
PASSIVE_STEM = "wp_mujoco_warp._src.passive_5924789"
FLUID_ENTRY = "_fluid_force_83bf3546_cuda_kernel_forward"
GRAVITY_ENTRY = "_gravity_force_7c63ac28_cuda_kernel_forward"
MJ_MINVAL = 1.0e-15


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def digest_text(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def write_json(path: pathlib.Path, value) -> None:
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2) + "\n").encode())


def float_payload(field: str, values) -> dict:
    flat = np.asarray(values, dtype=np.float32).reshape(-1)
    raw = flat.astype("<f4", copy=False).tobytes(order="C")
    return {
        "field": field,
        "count": int(flat.size),
        "little_endian_sha256": hashlib.sha256(raw).hexdigest(),
        "bits": [f"{int(value):08x}" for value in flat.view(np.uint32)],
    }


def bits_of(values) -> list[str]:
    flat = np.asarray(values, dtype=np.float32).reshape(-1)
    return [f"{int(value):08x}" for value in flat.view(np.uint32)]


def check_pinned_env() -> dict:
    versions = {
        "python": sys.version.split()[0],
        "warp": None,
        "mujoco": None,
        "numpy": np.__version__,
        "mujoco-warp": None,
    }
    revision = subprocess.check_output(
        ["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != REVISION:
        raise RuntimeError(f"upstream revision mismatch: {revision}")
    if subprocess.check_output(
        ["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True
    ).strip():
        raise RuntimeError("upstream checkout is dirty")
    import mujoco

    import warp  # noqa: F401  (version probe only)
    import mujoco_warp

    versions["warp"] = warp.__version__
    versions["mujoco"] = mujoco.__version__
    versions["mujoco-warp"] = importlib.metadata.version("mujoco-warp")
    for key, expected in EXPECTED_VERSIONS.items():
        if versions[key] != expected:
            raise RuntimeError(f"version mismatch {key}: {versions[key]} != {expected}")
    site = pathlib.Path(warp.__file__).parent
    warp_dll = site / "bin" / "warp.dll"
    if digest(warp_dll) != WARP_DLL_SHA256:
        raise RuntimeError("warp.dll does not match the pinned wheel RECORD")
    for name in ("passive.py", "smooth.py", "support.py", "types.py", "io.py", "math.py"):
        if not (UPSTREAM / "mujoco_warp" / "_src" / name).is_file():
            raise RuntimeError(f"upstream source missing: {name}")
    gpu = subprocess.check_output(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        text=True,
    ).strip()
    return {"versions": versions, "upstream_revision": revision, "gpu": gpu}


def seed_frozen_cache() -> pathlib.Path:
    """Copy the frozen smooth/passive module artifacts into a fresh Warp cache.

    Warp 1.15 uses `<kernel_cache_dir>/<warp version>/<module identifier>/`.
    The copies are verified against the in-repo frozen hashes before use.
    """
    if CACHE_ROOT.exists():
        shutil.rmtree(CACHE_ROOT)
    base = CACHE_ROOT / "1.15.0"
    smooth_dir = base / SMOOTH_STEM
    passive_dir = base / PASSIVE_STEM
    smooth_dir.mkdir(parents=True)
    passive_dir.mkdir(parents=True)
    pairs = [
        (
            FIXTURES / "independent-kinematics" / "smooth.cu",
            smooth_dir / f"{SMOOTH_STEM}.cu",
            SMOOTH_CU_SHA256,
        ),
        (
            FIXTURES / "independent-kinematics" / "smooth.meta",
            smooth_dir / f"{SMOOTH_STEM}.meta",
            SMOOTH_META_SHA256,
        ),
        (
            FIXTURES / "independent-kinematics" / "smooth.ptx",
            smooth_dir / f"{SMOOTH_STEM}.sm89.ptx",
            SMOOTH_PTX_SHA256,
        ),
        (
            FIXTURES / "generated" / "module.cu",
            passive_dir / f"{PASSIVE_STEM}.cu",
            PASSIVE_CU_SHA256,
        ),
        (
            FIXTURES / "generated" / "module.meta",
            passive_dir / f"{PASSIVE_STEM}.meta",
            PASSIVE_META_SHA256,
        ),
        (
            FIXTURES / "generated" / "module.ptx",
            passive_dir / f"{PASSIVE_STEM}.sm89.ptx",
            PASSIVE_PTX_SHA256,
        ),
    ]
    for source, target, expected in pairs:
        if digest(source) != expected:
            raise RuntimeError(f"frozen artifact hash mismatch: {source.name}")
        shutil.copyfile(source, target)
        if digest(target) != expected:
            raise RuntimeError(f"copied cache artifact hash mismatch: {target.name}")
    return CACHE_ROOT


def bootstrap_warp():
    """Initialize Warp against the seeded frozen cache and import upstream modules."""
    cache = seed_frozen_cache()
    sys.path.insert(0, str(UPSTREAM))
    import warp as wp

    if wp.__version__ != WARP_VERSION:
        raise RuntimeError(f"unexpected Warp version: {wp.__version__}")
    wp.config.kernel_cache_dir = str(cache)
    wp.init()
    import mujoco_warp._src.passive as passive
    import mujoco_warp._src.smooth as smooth

    device = "cuda:0"
    for module in (smooth, passive):
        wp.load_module(module=wp.get_module(module.__name__), device=device)
    return wp, smooth, passive


def np_f32(values, shape) -> np.ndarray:
    return np.asarray(values, dtype=np.float32).reshape(shape)


def wp_array(wp, values, dtype, shape):
    return wp.array(np_f32(values, shape), dtype=dtype, device="cuda:0")


def build_native_model(config: dict):
    """Build the equivalent native model from the fixture config.

    Returns structural arrays from the pinned upstream converter. Used to derive
    branch/dof/tree structure and to natural-validate the fixture topology.
    Only topology is consumed; inertial values are irrelevant placeholders.
    """
    import mujoco
    import mujoco_warp

    def body_block(index: int) -> str:
        children = [
            b
            for b in range(1, config["nbody"])
            if config["body_parentid"][b] == index
        ]
        inner = "".join(body_block(child) for child in children)
        pos = " ".join(f"{float(v):.17g}" for v in config["body_pos"][index])
        quat = " ".join(f"{float(v):.17g}" for v in config["body_quat_wxyz"][index])
        joint = ""
        for j, joint_body in enumerate(config["jnt_bodyid"]):
            if joint_body != index:
                continue
            kind = {0: "free", 1: "ball", 2: "slide", 3: "hinge"}[config["jnt_type"][j]]
            jpos = " ".join(f"{float(v):.17g}" for v in config["jnt_pos"][j])
            axis = " ".join(f"{float(v):.17g}" for v in config["jnt_axis"][j])
            joint = f'<joint type="{kind}" pos="{jpos}" axis="{axis}"/>'
        geoms = ""
        for g, geom_body in enumerate(config.get("geom_bodyid", [])):
            if geom_body != index:
                continue
            kind = {2: "sphere", 3: "capsule", 4: "ellipsoid", 5: "cylinder", 6: "box"}[
                config["geom_type"][g]
            ]
            size = " ".join(f"{float(v):.17g}" for v in config["geom_size"][g])
            gpos = " ".join(f"{float(v):.17g}" for v in config["geom_pos"][g])
            gquat = " ".join(f"{float(v):.17g}" for v in config["geom_quat"][g])
            geoms += f'<geom type="{kind}" size="{size}" pos="{gpos}" quat="{gquat}" mass="0.1"/>'
        inertial = f'<inertial pos="0 0 0" mass="1.0" diaginertia="1 1 1"/>'
        return (
            f'<body name="b{index}" pos="{pos}" quat="{quat}">'
            f"{joint}{inertial}{geoms}{inner}</body>"
        )

    roots = [b for b in range(1, config["nbody"]) if config["body_parentid"][b] == 0]
    xml = "<mujoco><worldbody>" + "".join(body_block(b) for b in roots) + "</worldbody></mujoco>"
    mjm = mujoco.MjModel.from_xml_string(xml)
    converted = mujoco_warp.put_model(mjm)
    nbody, nv = config["nbody"], config["nv"]
    return {
        "mjm": mjm,
        "body_parentid": [int(v) for v in converted.body_parentid.numpy()],
        "body_rootid": [int(v) for v in converted.body_rootid.numpy()],
        "body_mocapid": [int(v) for v in converted.body_mocapid.numpy()],
        "body_weldid": [int(v) for v in converted.body_weldid.numpy()],
        "body_dofadr": [int(v) for v in converted.body_dofadr.numpy()],
        "body_jntnum": [int(v) for v in converted.body_jntnum.numpy()],
        "body_jntadr": [int(v) for v in converted.body_jntadr.numpy()],
        "body_branches": [int(v) for v in converted.body_branches.numpy()],
        "body_branch_start": [int(v) for v in converted.body_branch_start.numpy()],
        "body_geomnum": [int(v) for v in converted.body_geomnum.numpy()],
        "body_geomadr": [int(v) for v in converted.body_geomadr.numpy()],
        "jnt_type": [int(v) for v in converted.jnt_type.numpy()],
        "jnt_qposadr": [int(v) for v in converted.jnt_qposadr.numpy()],
        "jnt_dofadr": [int(v) for v in converted.jnt_dofadr.numpy()],
        "jnt_bodyid": [int(v) for v in converted.jnt_bodyid.numpy()],
        "dof_bodyid": [int(v) for v in converted.dof_bodyid.numpy()][:nv],
        "body_isdofancestor": [
            [int(v) for v in row[:nv]]
            for row in converted.body_isdofancestor.numpy()[:, :nv]
        ],
        "body_tree_levels": [
            [int(v) for v in level.numpy()] for level in converted.body_tree
        ],
        "mujoco_version": mujoco.__version__,
        "mujoco_warp_version": importlib.metadata.version("mujoco-warp"),
        "io_source_sha256": digest(UPSTREAM / "mujoco_warp" / "_src" / "io.py"),
        "support_source_sha256": digest(UPSTREAM / "mujoco_warp" / "_src" / "support.py"),
    }


def subtreemass_by_body(config: dict) -> list[float]:
    """Subtree mass per body from the base (period 1) mass row.

    Body indices follow MJCF order (parent index < child index), so one reverse
    sweep accumulates each subtree mass into its parent. World body includes all.
    """
    masses = np.asarray(config["body_mass"][0], dtype=np.float32)
    subtree = masses.astype(np.float64)
    for body in range(len(masses) - 1, 0, -1):
        subtree[config["body_parentid"][body]] += subtree[body]
    return [float(v) for v in np.asarray(subtree, dtype=np.float32)]
