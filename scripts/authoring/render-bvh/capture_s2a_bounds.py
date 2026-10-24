"""Offline authoring: capture the S2a host-side AABB/refit reference (bit-level JSON).

OFFLINE ONLY: not imported or run by product builds/tests/gates; the product test
chain never starts Python. Reads fixtures/ plus the pinned authoring environment
and the frozen upstream checkout only.

Why this exists: S2a verifies on the host (no GPU) that the product's f32 AABB
computation for primitive geoms reproduces the frozen MJWarp reference bit for
bit, at the initial pose and after moving the free base body and refitting. The
reference AABBs are produced by the real MJWarp kernels (`_compute_bvh_bounds`
plus `Bvh.refit`), but the *inputs* used by those kernels must be recorded too,
because the refit reads MJWarp's own f32 device poses (which are not always
bit-equal to the CPU MuJoCo poses). This script therefore records, for every
case, the exact f32 geom poses fed to the kernel and the resulting lower/upper
leaf bounds as u32 bit patterns, plus a set of extra deterministic calibration
poses that pin the f32 operation order of every geom-type formula.

Outputs (fixtures/render-bvh/scene-a/s2a/):
  bounds.json          (schema/provenance + model inputs + cases)
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import platform
import subprocess
import sys
import time

import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
OUT = SCENE / "s2a"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
S2_INIT = SCENE / "s2" / "bvh_initial.npz"
S2_MOVED = SCENE / "s2" / "bvh_moved.npz"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/s2a-cache"
TASK_ID = "2026-10-24-render-bvh-s2a"
SCHEMA = 1
MJB_SHA256 = "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4"
XML_SHA256 = "a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {
    "warp": "1.15.0",
    "mujoco": "3.15.0",
    "mujoco_warp": "3.15.0",
    "numpy": "2.5.4",
}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
GPU_ARCH = 89
GPU_DRIVER = "596.36"

# Same render parameters as the frozen S0b/S2 references: the enabled-geom subset
# must match so the scene BVH holds exactly the 7 leaves compared here.
PARAMS_28 = {
    "nworld": 1,
    "cam_res": None,
    "render_rgb": True,
    "render_depth": True,
    "render_seg": True,
    "use_textures": True,
    "use_fast_math": True,
    "use_shadows": False,
    "use_ambient_lighting": True,
    "enabled_geom_groups": [0, 1, 2, 3],
    "cam_active": None,
    "background_color": [0.0, 0.0, 0.0, 1.0],
    "flex_render_smooth": True,
    "use_precomputed_rays": True,
    "render_skybox": False,
    "enable_backface_culling": True,
    "shadow_light_fraction": 0.3,
    "samples_per_pixel": 1,
    "enable_vertex_normals": True,
    "enable_specular": True,
    "enable_emission": True,
    "enable_per_light_ambient": True,
    "splat_position": None,
    "splat_rotation": None,
    "splat_scale": None,
    "splat_rgba": None,
    "splat_adr": None,
    "splat_group_id": None,
}
# Same deterministic move as the S2 capture (so the `moved-base` case must
# reproduce the S2 moved leaf bounds bit for bit and can be cross-checked).
MOVE_TRANSLATION = [0.05, -0.04, 0.03]
MOVE_ANGLE_Z = 0.18
# Extra calibration states: (translation, raw quat (w,x,y,z), hinge angle).
# They are literals, not RNG output, so the capture is reproducible.
CALIBRATION = [
    ([0.12, -0.07, 0.05], [0.9, 0.2, -0.3, 0.1], 0.35),
    ([-0.2, 0.15, 0.08], [0.7, -0.4, 0.2, 0.5], -0.6),
    ([0.02, 0.3, -0.1], [0.5, 0.5, 0.5, 0.5], 1.2),
    ([0.35, 0.05, 0.02], [0.95, -0.1, 0.05, 0.28], 0.75),
    ([-0.05, -0.25, 0.2], [0.6, 0.3, -0.5, 0.55], -1.1),
    ([0.18, 0.22, 0.14], [0.85, 0.35, 0.12, -0.38], 2.0),
]

wp.config.kernel_cache_dir = str(CACHE)

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def fail(message: str) -> None:
    raise SystemExit(f"S2a reference capture mismatch: {message}")


def check_environment() -> dict:
    problems = []
    if pathlib.Path(sys.prefix) != VENV:
        problems.append(f"python prefix {sys.prefix} != {VENV}")
    versions = {
        "python": platform.python_version(),
        "warp": wp.__version__,
        "mujoco": mujoco.__version__,
        "mujoco_warp": mjw.__version__,
        "numpy": np.__version__,
    }
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            problems.append(f"{name} {versions[name]} != {expected}")
    if run(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"]) != REVISION:
        problems.append("upstream revision mismatch")
    if run(["git", "-C", str(UPSTREAM), "status", "--porcelain"]):
        problems.append("upstream checkout is dirty")
    if digest(MJB) != MJB_SHA256:
        problems.append("scene-315.mjb hash mismatch")
    if digest_text(XML) != XML_SHA256:
        problems.append("scene.xml hash mismatch")
    device = wp.get_device("cuda:0")
    if int(device.arch) != GPU_ARCH:
        problems.append(f"gpu arch {device.arch} != {GPU_ARCH}")
    if GPU_NAME not in device.name:
        problems.append(f"gpu name {device.name} != {GPU_NAME}")
    if run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]) != GPU_DRIVER:
        problems.append("gpu driver mismatch")
    if problems:
        fail("; ".join(problems))
    return {
        "versions": versions,
        "python_executable": sys.executable,
        "python_prefix": sys.prefix,
        "platform": platform.platform(),
        "gpu": device.name,
        "gpu_arch": int(device.arch),
        "gpu_driver": GPU_DRIVER,
        "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "device_reference_code": {
            name: digest(UPSTREAM / "mujoco_warp" / "_src" / name)
            for name in ("bvh.py", "render_util.py", "types.py")
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def quat_multiply_z(q: np.ndarray, angle: float) -> np.ndarray:
    """q * quat(z axis, angle), normalized, with MuJoCo q = (w, x, y, z)."""
    half = 0.5 * angle
    rz = np.array([np.cos(half), 0.0, 0.0, np.sin(half)])
    w1, x1, y1, z1 = q
    w2, x2, y2, z2 = rz
    out = np.array(
        [
            w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
            w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
            w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
            w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
        ]
    )
    return out / np.linalg.norm(out)


def f32_bits(values: np.ndarray) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def case_state(mjm: mujoco.MjModel, qpos: np.ndarray) -> mujoco.MjData:
    mjd = mujoco.MjData(mjm)
    mjd.qpos[:] = qpos
    mujoco.mj_forward(mjm, mjd)
    return mjd


def capture_case(mjm, m, rc, qpos, name, kind):
    mjd = case_state(mjm, qpos)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    mjw.refit_bvh(m, d, rc)
    wp.synchronize()
    xpos = np.asarray(d.geom_xpos.numpy(), dtype=np.float32)
    xmat = np.asarray(d.geom_xmat.numpy(), dtype=np.float32)
    return {
        "name": name,
        "kind": kind,
        "qpos": np.asarray(mjd.qpos, dtype=np.float64).reshape(-1).tolist(),
        "inputs": {
            "geom_xpos_bits": f32_bits(xpos),
            "geom_xmat_bits": f32_bits(xmat),
        },
        "lower_bits": f32_bits(np.asarray(rc.lower.numpy(), dtype=np.float32)),
        "upper_bits": f32_bits(np.asarray(rc.upper.numpy(), dtype=np.float32)),
    }


def run_pass(mjm) -> dict:
    mjd_default = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd_default)
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    wp.synchronize()
    # The build path in create_render_context derives its AABB inputs from a
    # fresh MjData after mj_forward, cast f64 -> f32 when uploaded. Reproduce
    # exactly those inputs (initial pose = default pose).
    initial_inputs = {
        "geom_xpos_bits": f32_bits(np.asarray(mjd_default.geom_xpos, dtype=np.float32)),
        "geom_xmat_bits": f32_bits(np.asarray(mjd_default.geom_xmat, dtype=np.float32)),
    }
    initial = {
        "name": "initial",
        "kind": "build",
        "qpos": np.asarray(mjd_default.qpos, dtype=np.float64).reshape(-1).tolist(),
        "inputs": initial_inputs,
        "lower_bits": f32_bits(np.asarray(rc.lower.numpy(), dtype=np.float32)),
        "upper_bits": f32_bits(np.asarray(rc.upper.numpy(), dtype=np.float32)),
    }
    m = mjw.put_model(mjm)
    default_qpos = np.asarray(mjd_default.qpos, dtype=np.float64).copy()
    moved_qpos = default_qpos.copy()
    moved_qpos[0:3] += np.asarray(MOVE_TRANSLATION, dtype=np.float64)
    moved_qpos[3:7] = quat_multiply_z(default_qpos[3:7], MOVE_ANGLE_Z)
    hinge = int(mjm.jnt_qposadr[mujoco.mj_name2id(mjm, mujoco.mjtObj.mjOBJ_JOINT, "tri-hinge")])
    cases = [initial, capture_case(mjm, m, rc, moved_qpos, "moved-base", "refit")]
    for index, (translation, quat, angle) in enumerate(CALIBRATION):
        qpos = default_qpos.copy()
        qpos[0:3] += np.asarray(translation, dtype=np.float64)
        qpos[3:7] = np.asarray(quat, dtype=np.float64) / np.linalg.norm(quat)
        qpos[hinge] = angle
        cases.append(capture_case(mjm, m, rc, qpos, f"calibration-{index}", "refit"))
    state = {
        "bvh_ngeom": int(rc.bvh_ngeom),
        "enabled_geom_ids": np.asarray(rc.enabled_geom_ids.numpy(), dtype=np.int32).tolist(),
        "group": np.asarray(rc.group.numpy(), dtype=np.int32).tolist(),
        "group_root": np.asarray(rc.group_root.numpy(), dtype=np.int32).tolist(),
    }
    del rc, m
    return {"cases": cases, "context": state}


def model_inputs(mjm) -> dict:
    mesh_half = {}
    for mid in range(int(mjm.nmesh)):
        adr = int(mjm.mesh_vertadr[mid])
        num = int(mjm.mesh_vertnum[mid])
        points = np.asarray(mjm.mesh_vert[adr:adr + num], dtype=np.float64)
        half = np.maximum(np.abs(points.min(axis=0)), np.abs(points.max(axis=0)))
        mesh_half[str(mid)] = f32_bits(half)
    return {
        "ngeom": int(mjm.ngeom),
        "nmesh": int(mjm.nmesh),
        "geom_type": np.asarray(mjm.geom_type, dtype=np.int32).tolist(),
        "geom_dataid": np.asarray(mjm.geom_dataid, dtype=np.int32).tolist(),
        "geom_size_bits": f32_bits(np.asarray(mjm.geom_size, dtype=np.float32)),
        "mesh_bounds_size_bits": mesh_half,
    }


def bits_diff(a: dict, b: dict) -> list[str]:
    problems = []
    for left, right in zip(a["cases"], b["cases"]):
        if left["name"] != right["name"]:
            problems.append("case order")
        for key in ("qpos", "inputs", "lower_bits", "upper_bits"):
            if left[key] != right[key]:
                problems.append(f"{left['name']}/{key}")
    if a["context"] != b["context"]:
        problems.append("context")
    return problems


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S2a environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    mjm_first = mujoco.MjModel.from_binary_path(str(MJB))
    mjm_second = mujoco.MjModel.from_binary_path(str(MJB))
    first = run_pass(mjm_first)
    started = time.perf_counter()
    second = run_pass(mjm_second)
    elapsed = time.perf_counter() - started
    problems = bits_diff(first, second)
    if problems:
        fail(f"two captures differ: {problems}")
    initial = first["cases"][0]
    moved = first["cases"][1]
    # Cross-check the reference leaf bounds against the already committed S2
    # capture: same context parameters and the same deterministic move must give
    # the same AABB bits.
    s2_initial = np.load(S2_INIT)
    s2_moved = np.load(S2_MOVED)
    cross_check = {
        "initial_lower_bits_equal": initial["lower_bits"] == f32_bits(s2_initial["lower"]),
        "initial_upper_bits_equal": initial["upper_bits"] == f32_bits(s2_initial["upper"]),
        "moved_lower_bits_equal": moved["lower_bits"] == f32_bits(s2_moved["lower"]),
        "moved_upper_bits_equal": moved["upper_bits"] == f32_bits(s2_moved["upper"]),
    }
    if not all(cross_check.values()):
        fail(f"S2 capture cross-check failed: {cross_check}")
    document = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s2a-host-aabb-refit-reference",
        "model": model_inputs(mjm_first),
        "context": first["context"],
        "cases": first["cases"],
        "cross_check_s2": cross_check,
        "environment": environment,
        "fixtures": {
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": XML_SHA256},
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": MJB_SHA256},
            "s2_initial": {"path": "fixtures/render-bvh/scene-a/s2/bvh_initial.npz", "sha256": digest(S2_INIT)},
            "s2_moved": {"path": "fixtures/render-bvh/scene-a/s2/bvh_moved.npz", "sha256": digest(S2_MOVED)},
        },
        "determinism": {"repeat": 2, "bitwise_identical": True, "second_pass_seconds": round(elapsed, 6)},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": memory_used_mib()},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh/capture_s2a_bounds.py"},
        "reads_product_tree": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "bounds.json").write_text(json.dumps(document, indent=2, sort_keys=False) + "\n", encoding="utf-8",
                                     newline="\n")
    print(json.dumps({
        "task_id": TASK_ID,
        "outputs": {"bounds.json": digest(OUT / "bounds.json")},
        "cases": [case["name"] for case in first["cases"]],
        "cross_check_s2": cross_check,
        "gpu_memory_mib": document["gpu_memory_mib"],
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
