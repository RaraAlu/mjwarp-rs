"""Offline authoring: capture the S2b GPU ray-query reference (device f32 poses + hits).

OFFLINE ONLY: not run by product builds/tests/gates. The S2b product test reads the
committed JSON; it never starts Python.

Why a new capture: the S0b/S2 ray hits were produced by the MJWarp `_ray_bvh` kernel
from *device-side f32 geom poses* (`d.geom_xpos`/`d.geom_xmat`), which can differ by
~1 ULP from the CPU MuJoCo poses that the S2 npz files store in `geom_xpos`. Feeding
CPU poses to the S2b product kernel shifts cylinder/capsule distances beyond the
4-ULP budget (observed 124 ULP on ray 92). This script records the exact device-side
f32 pose bits together with the reference hits, and cross-checks the hits against the
already committed S0b/S2 npz references so the capture is provably reproducing them.

Output: fixtures/render-bvh/scene-a/s2/reference_hits.json
"""

from __future__ import annotations

import gc
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
OUT = SCENE / "s2/reference_hits.json"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
RAYS = SCENE / "reference/rays.npz"
S0B = SCENE / "reference/rays.npz"
S2_INIT = SCENE / "s2/bvh_initial.npz"
S2_MOVED = SCENE / "s2/bvh_moved.npz"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/s2b-cache"
TASK_ID = "2026-10-24-render-bvh-s2b"
SCHEMA = 1
MJB_SHA256 = "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4"
XML_SHA256 = "a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
GPU_ARCH = 89
GPU_DRIVER = "596.36"
MOVE_TRANSLATION = [0.05, -0.04, 0.03]
MOVE_ANGLE_Z = 0.18
# Same 28 create_render_context parameters as S0b/S2 so the BVH geometry set matches.
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
    raise SystemExit(f"S2b reference capture mismatch: {message}")


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
            for name in ("bvh.py", "ray.py", "render_util.py", "types.py")
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def quat_multiply_z(q: np.ndarray, angle: float) -> np.ndarray:
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


def cast_rays(m, d, rc, pnt: np.ndarray, direction: np.ndarray) -> dict:
    from mujoco_warp._src import types as mjw_types

    nray = int(pnt.shape[0])
    pnt_w = wp.array(pnt.reshape(1, nray, 3).astype(np.float32), dtype=wp.vec3)
    vec_w = wp.array(direction.reshape(1, nray, 3).astype(np.float32), dtype=wp.vec3)
    bodyexclude = wp.array(np.full(nray, -1, dtype=np.int32), dtype=int)
    dist = wp.empty((1, nray), dtype=float)
    geomid = wp.empty((1, nray), dtype=int)
    normal = wp.empty((1, nray), dtype=wp.vec3)
    mjw.rays(
        m,
        d,
        pnt_w,
        vec_w,
        mjw_types.vec6(-1, -1, -1, -1, -1, -1),
        True,
        bodyexclude,
        dist,
        geomid,
        normal,
        rc,
    )
    wp.synchronize()
    return {
        "dist_bits": bits(np.asarray(dist.numpy(), dtype=np.float32)),
        "geomid": ints(np.asarray(geomid.numpy(), dtype=np.int32)),
        "normal_bits": bits(np.asarray(normal.numpy(), dtype=np.float32)),
    }


def bits(values: np.ndarray) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def ints(values: np.ndarray) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.int32).reshape(-1).tolist()


def move_qpos(mjm: mujoco.MjModel, qpos: np.ndarray) -> np.ndarray:
    moved = np.array(qpos, dtype=np.float64)
    moved[0:3] += np.asarray(MOVE_TRANSLATION, dtype=np.float64)
    moved[3:7] = quat_multiply_z(moved[3:7], MOVE_ANGLE_Z)
    return moved


def load_state(mjm: mujoco.MjModel, mjd_source: mujoco.MjData):
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd_source, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    return m, d


def capture_case(mjm, mjd, rc, refit: bool, pnt: np.ndarray, direction: np.ndarray, name: str) -> dict:
    m, d = load_state(mjm, mjd)
    if refit:
        mjw.refit_bvh(m, d, rc)
        wp.synchronize()
    device_xpos = np.asarray(d.geom_xpos.numpy(), dtype=np.float32)
    device_xmat = np.asarray(d.geom_xmat.numpy(), dtype=np.float32)
    hits = cast_rays(m, d, rc, pnt, direction)
    del m, d
    gc.collect()
    return {
        "name": name,
        "geom_xpos_bits": bits(device_xpos),
        "geom_xmat_bits": bits(device_xmat),
        **hits,
    }


def run_pass(mjm: mujoco.MjModel, pnt: np.ndarray, direction: np.ndarray) -> dict:
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    wp.synchronize()
    initial = capture_case(mjm, mjd, rc, False, pnt, direction, "initial")
    moved_mjd = mujoco.MjData(mjm)
    moved_mjd.qpos[:] = move_qpos(mjm, np.asarray(mjd.qpos, dtype=np.float64))
    mujoco.mj_forward(mjm, moved_mjd)
    moved = capture_case(mjm, moved_mjd, rc, True, pnt, direction, "moved")
    enabled = np.asarray(rc.enabled_geom_ids.numpy(), dtype=np.int32).tolist()
    del rc
    return {"initial": initial, "moved": moved, "enabled": enabled}


def case_bits_equal(left: dict, right: dict) -> list[str]:
    problems = []
    for key in ("geom_xpos_bits", "geom_xmat_bits", "dist_bits", "geomid", "normal_bits"):
        if left[key] != right[key]:
            problems.append(f"{left['name']}/{key}")
    return problems


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S2b environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    rays = np.load(RAYS)
    pnt = np.asarray(rays["ray_origin"], dtype=np.float32)
    direction = np.asarray(rays["ray_direction"], dtype=np.float32)
    if pnt.shape != (384, 3) or direction.shape != (384, 3):
        fail(f"unexpected ray set shape {pnt.shape} {direction.shape}")

    mjm_first = mujoco.MjModel.from_binary_path(str(MJB))
    mjm_second = mujoco.MjModel.from_binary_path(str(MJB))
    first = run_pass(mjm_first, pnt, direction)
    started = time.perf_counter()
    second = run_pass(mjm_second, pnt, direction)
    elapsed = time.perf_counter() - started
    if first["enabled"] != second["enabled"]:
        fail("enabled geom ids differ between passes")
    for name in ("initial", "moved"):
        problems = case_bits_equal(first[name], second[name])
        if problems:
            fail(f"two captures differ: {problems}")

    # Cross-check against the already committed references: the captured device poses
    # must reproduce the S0b initial hits and the S2 moved hits bit for bit.
    s0b = np.load(S0B)
    s2_initial = np.load(S2_INIT)
    s2_moved = np.load(S2_MOVED)
    cross_check = {
        "initial_geomid_vs_s0b": first["initial"]["geomid"] == ints(np.asarray(s0b["ray_geomid"])),
        "initial_dist_bits_vs_s0b": first["initial"]["dist_bits"] == bits(np.asarray(s0b["ray_dist"])),
        "initial_normal_bits_vs_s0b": first["initial"]["normal_bits"] == bits(np.asarray(s0b["ray_normal"])),
        "initial_geomid_vs_s2": first["initial"]["geomid"] == ints(np.asarray(s2_initial["hit_geomid"])),
        "moved_geomid_vs_s2": first["moved"]["geomid"] == ints(np.asarray(s2_moved["hit_geomid"])),
        "moved_dist_bits_vs_s2": first["moved"]["dist_bits"] == bits(np.asarray(s2_moved["hit_dist"])),
        "moved_normal_bits_vs_s2": first["moved"]["normal_bits"] == bits(np.asarray(s2_moved["hit_normal"])),
    }
    if not all(cross_check.values()):
        fail(f"S0b/S2 hit cross-check failed: {cross_check}")

    cases = {"initial": first["initial"], "moved": first["moved"]}
    document = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s2b-gpu-ray-reference",
        "ray_count": 384,
        "enabled_geom_ids": first["enabled"],
        "bvh_ngeom": len(first["enabled"]),
        "rays": {"origin_bits": bits(pnt), "direction_bits": bits(direction)},
        "cases": cases,
        "stats": {
            "hits": {name: sum(1 for value in case["geomid"] if value >= 0) for name, case in cases.items()},
            "mesh_hit_rays": sorted(index for index, value in enumerate(cases["initial"]["geomid"]) if value == 3),
            "mesh_hit_rays_moved": sorted(index for index, value in enumerate(cases["moved"]["geomid"]) if value == 3),
        },
        "cross_check_s0b_s2": cross_check,
        "sources": {
            "rays_npz": {"path": "fixtures/render-bvh/scene-a/reference/rays.npz", "sha256": digest(RAYS)},
            "s2_initial_npz": {
                "path": "fixtures/render-bvh/scene-a/s2/bvh_initial.npz",
                "sha256": digest(S2_INIT),
            },
            "s2_moved_npz": {"path": "fixtures/render-bvh/scene-a/s2/bvh_moved.npz", "sha256": digest(S2_MOVED)},
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": XML_SHA256},
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": MJB_SHA256},
        },
        "environment": environment,
        "determinism": {"repeat": 2, "bitwise_identical": True, "second_pass_seconds": round(elapsed, 6)},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": memory_used_mib()},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh/capture_s2b_query.py"},
        "authoring_only": True,
        "reads_product_tree": False,
        "product_python_dependency": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(
        json.dumps(
            {
                "output": str(OUT.relative_to(ROOT)),
                "sha256": digest(OUT),
                "hits": document["stats"]["hits"],
                "mesh_hit_rays": document["stats"]["mesh_hit_rays"],
                "cross_check": cross_check,
                "gpu_memory_mib": document["gpu_memory_mib"],
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
