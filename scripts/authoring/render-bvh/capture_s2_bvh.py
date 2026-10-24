"""Offline authoring: capture the S2 scene-BVH reference (AABB/refit/ray hits).

OFFLINE ONLY: not imported or run by product builds/tests/gates; the product test
chain never starts Python. This script reads only fixtures/ plus the pinned
authoring environment and the frozen upstream checkout.

Why this exists: the S0b reference captured 384 ray hits at the initial pose but
not the scene-BVH AABBs or the refit-after-move behaviour that S2 acceptance
needs. This script captures, with the pinned MJWarp 3.15 stack and the committed
`scene-315.mjb`:

  - the scene BVH bounds (lower/upper), group ids and group roots,
  - the same 384-ray hit trace (geomid/distance/normal),
  - the geom world poses used to derive those AABBs,

at the initial pose and after moving the free base body, refitting the BVH the
same way the renderer does (`mjw.refit_bvh`). Every array is captured twice from
fresh model/data/context instances; bitwise-identical output is required.

Outputs (fixtures/render-bvh/scene-a/s2/):
  bvh_initial.npz bvh_moved.npz context.json
and fixtures/render-bvh/scene-a/s2/provenance.json.
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
OUT = SCENE / "s2"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
RAYS = SCENE / "reference/rays.npz"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/s2-cache"
TASK_ID = "2026-10-24-render-bvh-s2"
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
# Same camera/render parameters as the frozen S0b reference: the BVH enabled-geom
# subset and the precomputed-ray layout must match the reference geometry set.
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
# Deterministic move of the free `base` body (qpos[0:7]): small translation + a
# rotation about z. Any move would do; the values are recorded in context.json.
MOVE_TRANSLATION = [0.05, -0.04, 0.03]
MOVE_ANGLE_Z = 0.18

wp.config.kernel_cache_dir = str(CACHE)

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def nvidia_smi(field: str) -> str:
    return run(["nvidia-smi", f"--query-gpu={field}", "--format=csv,noheader"]).splitlines()[0].strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def fail(message: str) -> None:
    raise SystemExit(f"S2 reference capture mismatch: {message}")


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
    if nvidia_smi("driver_version") != GPU_DRIVER:
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
        "gpu_driver": nvidia_smi("driver_version"),
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


def load_state(mjd_source: mujoco.MjData, mjm: mujoco.MjModel):
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd_source, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    return m, d


def cast_rays(m, d, rc, pnt_np: np.ndarray, vec_np: np.ndarray):
    from mujoco_warp._src import types as mjw_types

    nray = int(pnt_np.shape[0])
    pnt = wp.array(pnt_np.reshape(1, nray, 3).astype(np.float32), dtype=wp.vec3)
    vec = wp.array(vec_np.reshape(1, nray, 3).astype(np.float32), dtype=wp.vec3)
    bodyexclude = wp.array(np.full(nray, -1, dtype=np.int32), dtype=int)
    dist = wp.empty((1, nray), dtype=float)
    geomid = wp.empty((1, nray), dtype=int)
    normal = wp.empty((1, nray), dtype=wp.vec3)
    mjw.rays(
        m,
        d,
        pnt,
        vec,
        mjw_types.vec6(-1, -1, -1, -1, -1, -1),
        True,
        bodyexclude,
        dist,
        geomid,
        normal,
        rc,
    )
    wp.synchronize()
    return (
        np.asarray(dist.numpy())[0].astype(np.float32),
        np.asarray(geomid.numpy())[0].astype(np.int32),
        np.asarray(normal.numpy())[0].astype(np.float32),
    )


def bvh_state(rc) -> dict:
    lower = np.asarray(rc.lower.numpy()).astype(np.float32)
    upper = np.asarray(rc.upper.numpy()).astype(np.float32)
    group = np.asarray(rc.group.numpy()).astype(np.int32)
    group_root = np.asarray(rc.group_root.numpy()).astype(np.int32)
    return {
        "lower": lower.reshape(-1, 3),
        "upper": upper.reshape(-1, 3),
        "group": group.reshape(-1),
        "group_root": group_root.reshape(-1),
        "bvh_ngeom": np.asarray([int(rc.bvh_ngeom)], dtype=np.int64),
        "enabled_geom_ids": np.asarray(rc.enabled_geom_ids.numpy(), dtype=np.int32),
    }


def run_once(mjm: mujoco.MjModel, mjd: mujoco.MjData, pnt: np.ndarray, vec: np.ndarray) -> dict:
    mujoco.mj_forward(mjm, mjd)
    m, d = load_state(mjd, mjm)
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    initial = {
        **bvh_state(rc),
        "geom_xpos": np.asarray(mjd.geom_xpos, dtype=np.float32).reshape(-1, 3),
        "geom_xmat": np.asarray(mjd.geom_xmat, dtype=np.float32).reshape(-1, 3, 3),
    }
    initial_hits = cast_rays(m, d, rc, pnt, vec)


def run_once(mjm: mujoco.MjModel, mjd: mujoco.MjData, pnt: np.ndarray, vec: np.ndarray) -> dict:
    mujoco.mj_forward(mjm, mjd)
    m, d = load_state(mjd, mjm)
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    initial = {
        **bvh_state(rc),
        "geom_xpos": np.asarray(mjd.geom_xpos, dtype=np.float32).reshape(-1, 3),
        "geom_xmat": np.asarray(mjd.geom_xmat, dtype=np.float32).reshape(-1, 3, 3),
    }
    initial_hits = cast_rays(m, d, rc, pnt, vec)

    moved = mujoco.MjData(mjm)
    moved.qpos[:] = mjd.qpos
    moved.qpos[0:3] += np.asarray(MOVE_TRANSLATION)
    moved.qpos[3:7] = quat_multiply_z(np.asarray(moved.qpos[3:7], dtype=np.float64), MOVE_ANGLE_Z)
    mujoco.mj_forward(mjm, moved)
    m2, d2 = load_state(moved, mjm)
    mjw.refit_bvh(m2, d2, rc)
    wp.synchronize()
    moved_state = {
        **bvh_state(rc),
        "geom_xpos": np.asarray(moved.geom_xpos, dtype=np.float32).reshape(-1, 3),
        "geom_xmat": np.asarray(moved.geom_xmat, dtype=np.float32).reshape(-1, 3, 3),
    }
    moved_hits = cast_rays(m2, d2, rc, pnt, vec)
    del rc, m, d, m2, d2
    gc.collect()
    return {
        "initial": {**initial, "hit_dist": initial_hits[0], "hit_geomid": initial_hits[1], "hit_normal": initial_hits[2]},
        "moved": {**moved_state, "hit_dist": moved_hits[0], "hit_geomid": moved_hits[1], "hit_normal": moved_hits[2]},
        "move_qpos7": np.asarray(moved.qpos[:7], dtype=np.float64),
    }


def save_npz_deterministic(path: pathlib.Path, arrays: dict) -> str:
    import io
    import zipfile

    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in sorted(arrays):
            buffer = io.BytesIO()
            np.lib.format.write_array(buffer, np.ascontiguousarray(arrays[name]), allow_pickle=False)
            info = zipfile.ZipInfo(f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            archive.writestr(info, buffer.getvalue())
    return digest(path)


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S2 environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    rays = np.load(RAYS)
    pnt = np.asarray(rays["ray_origin"], dtype=np.float32)
    vec = np.asarray(rays["ray_direction"], dtype=np.float32)
    if pnt.shape != (384, 3) or vec.shape != (384, 3):
        fail(f"unexpected S0b ray set shape {pnt.shape} {vec.shape}")

    mjm_first = mujoco.MjModel.from_binary_path(str(MJB))
    mjm_second = mujoco.MjModel.from_binary_path(str(MJB))
    first = run_once(mjm_first, mujoco.MjData(mjm_first), pnt, vec)
    started = time.perf_counter()
    second = run_once(mjm_second, mujoco.MjData(mjm_second), pnt, vec)
    elapsed = time.perf_counter() - started
    for pose in ("initial", "moved"):
        for key in first[pose]:
            if not np.array_equal(first[pose][key], second[pose][key]):
                fail(f"two captures differ at {pose}/{key}")
    if not np.array_equal(first["move_qpos7"], second["move_qpos7"]):
        fail("two captures differ at move_qpos7")

    initial_files = save_npz_deterministic(OUT / "bvh_initial.npz", first["initial"])
    moved_files = save_npz_deterministic(OUT / "bvh_moved.npz", first["moved"])
    # Cross-check the initial hit trace against the frozen S0b reference.
    s0b = np.load(SCENE / "reference/rays.npz")
    s0b_geomid = np.asarray(s0b["ray_geomid"], dtype=np.int32)
    s0b_dist = np.asarray(s0b["ray_dist"], dtype=np.float32)
    s0b_normal = np.asarray(s0b["ray_normal"], dtype=np.float32)
    initial_geomid = first["initial"]["hit_geomid"]
    cross_check = {
        "geomid_equal": bool(np.array_equal(first["initial"]["hit_geomid"], s0b_geomid)),
        "dist_bit_equal": bool(np.array_equal(first["initial"]["hit_dist"].view(np.uint32), s0b_dist.view(np.uint32))),
        "normal_bit_equal": bool(
            np.array_equal(first["initial"]["hit_normal"].view(np.uint32), s0b_normal.view(np.uint32))
        ),
        "geomid_mismatches": int(np.count_nonzero(initial_geomid != s0b_geomid)),
    }
    if not cross_check["geomid_equal"]:
        fail(f"initial geomid trace differs from S0b reference: {cross_check}")

    context = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s2-scene-bvh-reference",
        "parameters": {name: (list(value) if isinstance(value, list) else value) for name, value in PARAMS_28.items()},
        "resolved": {
            "nworld": int(PARAMS_28["nworld"]),
            "bvh_ngeom": int(first["initial"]["bvh_ngeom"][0]),
            "enabled_geom_ids": first["initial"]["enabled_geom_ids"].tolist(),
            "total_bvh_size": int(first["initial"]["lower"].shape[0]),
        },
        "move": {"body": "base", "qpos7": first["move_qpos7"].tolist(), "translation": MOVE_TRANSLATION, "angle_z": MOVE_ANGLE_Z},
        "rays": {"count": 384, "source": "fixtures/render-bvh/scene-a/reference/rays.npz", "sha256": digest(RAYS)},
        "initial": {"files": {"bvh_initial.npz": initial_files}, "cross_check_s0b": cross_check},
        "moved": {"files": {"bvh_moved.npz": moved_files}},
        "run": {"second_capture_seconds": round(elapsed, 6)},
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "context.json").write_text(json.dumps(context, indent=2) + "\n", encoding="utf-8", newline="\n")
    after_mib = memory_used_mib()
    provenance = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s2-scene-bvh-reference-provenance",
        "environment": environment,
        "fixtures": {
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": XML_SHA256},
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": MJB_SHA256},
            "s0b_rays": {"path": "fixtures/render-bvh/scene-a/reference/rays.npz", "sha256": digest(RAYS)},
        },
        "outputs": {"bvh_initial.npz": initial_files, "bvh_moved.npz": moved_files,
                    "context_json_sha256": digest(OUT / "context.json")},
        "determinism": {"repeat": 2, "bitwise_identical": True},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": after_mib},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh/capture_s2_bvh.py"},
        "reads_product_tree": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    (OUT / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"task_id": TASK_ID, "files": {"initial": initial_files, "moved": moved_files},
                      "cross_check_s0b": cross_check, "gpu_memory_mib": provenance["gpu_memory_mib"]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
