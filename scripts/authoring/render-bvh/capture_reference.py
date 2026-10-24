"""Offline authoring: capture the frozen MuJoCo 3.15 / MJWarp reference for S0b.

OFFLINE ONLY. Not imported or run by product builds/tests/gates; the product
test chain never starts Python. This script reads only fixtures/ and the pinned
authoring environment (never the product tree), captures RGB/depth/seg images,
a fixed ray-hit trace set (camera pixels, seeded random, grazing, no-hit) and a
create_render_context 28-parameter receipt, captures twice and requires identical
sha256 for every output. Any environment mismatch fails loudly (no fallback).

Outputs (fixtures/render-bvh/scene-a/reference/):
  rgb.npz depth.npz seg.npz rays.npz context.json README.md
and fixtures/render-bvh/scene-a/provenance.json.
"""

from __future__ import annotations

import gc
import hashlib
import io
import json
import pathlib
import platform
import shutil
import subprocess
import sys
import time
import zipfile

import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
REFERENCE = SCENE / "reference"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
TMP = ROOT / "target/render-bvh/s0b-run"
CACHE = ROOT / "target/render-bvh/s0b-cache"
TASK_ID = "2026-10-24-render-bvh-s0b"
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
REFERENCE_CODE = ["render.py", "render_util.py", "ray.py", "types.py", "bvh.py"]
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
from capture_reference_rays import (  # noqa: E402
    CATEGORY_NAMES,
    SEED,
    build_ray_set,
    cast_rays,
    ray_stats,
)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    """LF-normalized text hash, matching model_parity.py's scene.xml hash."""
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def nvidia_smi(field: str) -> str:
    return run(["nvidia-smi", f"--query-gpu={field}", "--format=csv,noheader"]).splitlines()[0].strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def fail(message: str) -> None:
    raise SystemExit(f"S0b environment/reference mismatch: {message}")


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
    if digest(XML) != XML_SHA256 and digest_text(XML) != XML_SHA256:
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
        "python_executable_sha256": digest(pathlib.Path(sys.executable)),
        "python_prefix": sys.prefix,
        "platform": platform.platform(),
        "gpu": device.name,
        "gpu_arch": int(device.arch),
        "gpu_driver": nvidia_smi("driver_version"),
        "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "reference_code": {
            name: digest(UPSTREAM / "mujoco_warp" / "_src" / name) for name in REFERENCE_CODE
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def load_state():
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    return mjm, m, d


def create_context(mjm):
    return mjw.create_render_context(mjm, **PARAMS_28)


def render_images(m, d, rc):
    started = time.perf_counter()
    mjw.render(m, d, rc)
    wp.synchronize()
    elapsed = time.perf_counter() - started
    cam_res = np.asarray(rc.cam_res.numpy())
    cam_ids = np.asarray(rc.cam_id_map.numpy())
    depth_adr = np.asarray(rc.depth_adr.numpy())
    seg_adr = np.asarray(rc.seg_adr.numpy())
    depth_raw = np.asarray(rc.depth_data.numpy())[0].astype(np.float32)
    seg_raw = np.asarray(rc.seg_data.numpy())[0].astype(np.int32)
    images = {"rgb": [], "depth": [], "seg": []}
    for idx in range(int(rc.nrender)):
        width, height = int(cam_res[idx][0]), int(cam_res[idx][1])
        rgb = wp.zeros((1, height, width), dtype=wp.vec3)
        mjw.get_rgb(rc, idx, rgb)
        wp.synchronize()
        pixels = width * height
        images["rgb"].append(np.asarray(rgb.numpy())[0].astype(np.float32))
        images["depth"].append(
            depth_raw[int(depth_adr[idx]) : int(depth_adr[idx]) + pixels].reshape(height, width)
        )
        images["seg"].append(
            seg_raw[int(seg_adr[idx]) : int(seg_adr[idx]) + pixels].reshape(height, width, 2)
        )
    meta = {
        "cam_ids": [int(v) for v in cam_ids],
        "cam_res": [[int(w), int(h)] for w, h in cam_res],
        "depth_semantics": "raw rc.depth_data planar depth in meters (0 = no hit); "
        "no normalization or clamping applied",
        "seg_semantics": "(object_id, object_type) pairs; background = (-1, -1)",
        "rgb_semantics": "unpacked vec3 in [0, 1] via get_rgb",
    }
    return images, meta, elapsed


def capture_once(mjm, m, d, rc):
    images, image_meta, render_seconds = render_images(m, d, rc)
    pnt, vec, category, grazing_geom_id, camera_pixel_counts = build_ray_set(mjm, d, rc)
    dist, geomid, normal = cast_rays(m, d, rc, pnt, vec)
    stats = ray_stats(category, dist, geomid, normal, vec, grazing_geom_id)
    if not all(v > 0 for v in camera_pixel_counts) or sum(camera_pixel_counts) != stats["camera_pixels"]["rays"]:
        fail(
            f"camera pixel ray coverage invalid: per-camera={camera_pixel_counts}, "
            f"aggregate={stats['camera_pixels']}"
        )
    for name in ("rgb", "depth", "seg"):
        for array in images[name]:
            if not np.isfinite(array).all():
                fail(f"non-finite {name} image")
    arrays = {}
    for cam_idx in range(len(images["rgb"])):
        arrays[f"rgb_cam{cam_idx}"] = images["rgb"][cam_idx]
        arrays[f"depth_cam{cam_idx}"] = images["depth"][cam_idx]
        arrays[f"seg_cam{cam_idx}"] = images["seg"][cam_idx]
    arrays["ray_origin"] = pnt[0]
    arrays["ray_direction"] = vec[0]
    arrays["ray_category"] = category
    arrays["ray_geomid"] = geomid
    arrays["ray_dist"] = dist
    arrays["ray_normal"] = normal
    return {
        "arrays": arrays,
        "image_meta": image_meta,
        "ray_stats": stats,
        "grazing_geom_id": int(grazing_geom_id),
        "camera_pixel_counts": [int(v) for v in camera_pixel_counts],
        "render_seconds": round(render_seconds, 6),
    }


def save_npz_deterministic(path: pathlib.Path, arrays: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in sorted(arrays):
            buffer = io.BytesIO()
            np.lib.format.write_array(buffer, np.ascontiguousarray(arrays[name]), allow_pickle=False)
            info = zipfile.ZipInfo(f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            archive.writestr(info, buffer.getvalue())
    return digest(path)


def save_capture(directory: pathlib.Path, capture: dict) -> dict:
    directory.mkdir(parents=True, exist_ok=True)
    files = {
        "rgb.npz": {k: v for k, v in capture["arrays"].items() if k.startswith("rgb_")},
        "depth.npz": {k: v for k, v in capture["arrays"].items() if k.startswith("depth_")},
        "seg.npz": {k: v for k, v in capture["arrays"].items() if k.startswith("seg_")},
        "rays.npz": {k: v for k, v in capture["arrays"].items() if k.startswith("ray_")},
    }
    return {name: save_npz_deterministic(directory / name, data) for name, data in files.items()}


def jsonify(value):
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, (np.generic,)):
        return value.item()
    if isinstance(value, tuple):
        return list(value)
    return value


def camera_paths(mjm, rc) -> list[dict]:
    """Resolved per-camera projection path (intrinsic branch vs fovy branch)."""
    paths = []
    for cam in (int(v) for v in rc.cam_id_map.numpy()):
        sensorsize = [float(mjm.cam_sensorsize[cam, 0]), float(mjm.cam_sensorsize[cam, 1])]
        paths.append(
            {
                "cam_id": cam,
                "projection": int(mjm.cam_projection[cam]),
                "resolution": [int(mjm.cam_resolution[cam, 0]), int(mjm.cam_resolution[cam, 1])],
                "sensorsize": sensorsize,
                "intrinsic": [float(v) for v in mjm.cam_intrinsic[cam]],
                "fovy_deg": float(mjm.cam_fovy[cam]),
                "branch": "intrinsic" if sensorsize[1] != 0.0 else "fovy",
            }
        )
    return paths


def context_receipt(mjm, rc, files_sha256, capture) -> dict:
    background = int(rc.background_color)
    return {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-context-receipt",
        "create_render_context_parameters": {name: jsonify(value) for name, value in PARAMS_28.items()},
        "parameter_count": len(PARAMS_28),
        "resolved": {
            "nrender": int(rc.nrender),
            "cam_id_map": [int(v) for v in rc.cam_id_map.numpy()],
            "cam_res": [[int(w), int(h)] for w, h in rc.cam_res.numpy()],
            "render_rgb": [bool(v) for v in rc.render_rgb.numpy()],
            "render_depth": [bool(v) for v in rc.render_depth.numpy()],
            "render_seg": [bool(v) for v in rc.render_seg.numpy()],
            "rgb_adr": [int(v) for v in rc.rgb_adr.numpy()],
            "depth_adr": [int(v) for v in rc.depth_adr.numpy()],
            "seg_adr": [int(v) for v in rc.seg_adr.numpy()],
            "total_rays": int(rc.total_rays),
            "use_textures": bool(rc.use_textures),
            "use_fast_math": bool(rc.use_fast_math),
            "use_shadows": bool(rc.use_shadows),
            "use_ambient_lighting": bool(rc.use_ambient_lighting),
            "use_precomputed_rays": bool(rc.use_precomputed_rays),
            "render_skybox": bool(rc.render_skybox),
            "enable_backface_culling": bool(rc.enable_backface_culling),
            "shadow_light_fraction": float(rc.shadow_light_fraction),
            "samples_per_pixel": int(rc.samples_per_pixel),
            "enable_vertex_normals": bool(rc.enable_vertex_normals),
            "enable_specular": bool(rc.enable_specular),
            "enable_emission": bool(rc.enable_emission),
            "enable_per_light_ambient": bool(rc.enable_per_light_ambient),
            "background_color_unpacked": [
                float((background >> 16) & 0xFF),
                float((background >> 8) & 0xFF),
                float(background & 0xFF),
                float((background >> 24) & 0xFF),
            ],
            "bvh_ngeom": int(rc.bvh_ngeom),
            "enabled_geom_ids": [int(v) for v in rc.enabled_geom_ids.numpy()],
            "znear": float(rc.znear),
            "zfar": float(rc.zfar),
            "headlight_active": bool(rc.headlight_active),
            "headlight_ambient": [float(v) for v in rc.headlight_ambient],
            "headlight_diffuse": [float(v) for v in rc.headlight_diffuse],
            "headlight_specular": [float(v) for v in rc.headlight_specular],
            "splat_count": int(rc.splat_count),
            "geom_ray_types": [int(v) for v in rc.geom_ray_types],
        },
        "camera_paths": camera_paths(mjm, rc),
        "camera_pixel_rays_per_camera": capture["camera_pixel_counts"],
        "outputs": {
            "files_sha256": files_sha256,
            "rgb_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("rgb_")],
            "depth_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("depth_")],
            "seg_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("seg_")],
            "ray_arrays": {
                k: {"shape": list(a.shape), "dtype": str(a.dtype)}
                for k, a in capture["arrays"].items()
                if k.startswith("ray_")
            },
            "image_meta": capture["image_meta"],
            "ray_stats": capture["ray_stats"],
        },
        "seed": SEED,
    }


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S0b environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    mjm, m, d = load_state()
    rc = create_context(mjm)
    paths = camera_paths(mjm, rc)
    branches = [path["branch"] for path in paths]
    for required in ("intrinsic", "fovy"):
        if required not in branches:
            fail(f"camera branch {required} not covered by the frozen scene: {branches}")
    for path in paths:
        if path["branch"] == "intrinsic" and not (path["intrinsic"][0] > 0.0 and path["intrinsic"][1] > 0.0):
            fail(f"camera {path['cam_id']} intrinsic branch has non-positive focal: {path['intrinsic']}")
    print(f"S0b camera paths: {branches}")
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc)
    first_dir, second_dir = TMP / "first", TMP / "second"
    if first_dir.exists():
        shutil.rmtree(first_dir)
    if second_dir.exists():
        shutil.rmtree(second_dir)
    first_hashes = save_capture(first_dir, first)
    second_hashes = save_capture(second_dir, second)
    if first_hashes != second_hashes:
        fail(f"two captures differ: {first_hashes} vs {second_hashes}")
    first_context = context_receipt(mjm, rc, first_hashes, first)
    second_context = context_receipt(mjm, rc, second_hashes, second)
    if json.dumps(first_context, sort_keys=True) != json.dumps(second_context, sort_keys=True):
        fail("two context receipts differ")
    REFERENCE.mkdir(parents=True, exist_ok=True)
    for name in first_hashes:
        shutil.copyfile(first_dir / name, REFERENCE / name)
    (REFERENCE / "context.json").write_text(
        json.dumps(first_context, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    context_sha = digest(REFERENCE / "context.json")
    after_mib = memory_used_mib()
    provenance = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s0b-reference-capture-provenance",
        "environment": environment,
        "fixtures": {
            "scene_xml": {
                "path": "fixtures/render-bvh/scene-a/scene.xml",
                "sha256": XML_SHA256,
                "hash_note": "LF-normalized bytes (CRLF->LF), same algorithm as model_parity.py",
            },
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": MJB_SHA256},
        },
        "camera_paths": paths,
        "camera_pixel_rays_per_camera": first["camera_pixel_counts"],
        "script": {
            "path": "scripts/authoring/render-bvh/capture_reference.py",
            "sha256": digest(pathlib.Path(__file__).resolve()),
        },
        "parameters": {name: jsonify(value) for name, value in PARAMS_28.items()},
        "seed": SEED,
        "capture": {
            "repeat": 2,
            "identical_sha256": first_hashes == second_hashes,
            "outputs": first_hashes,
            "context_json_sha256": context_sha,
            "ray_categories": CATEGORY_NAMES,
            "ray_stats": first["ray_stats"],
            "grazing_geom_id": first["grazing_geom_id"],
            "image_meta": first["image_meta"],
            "render_seconds": {
                "first": first["render_seconds"],
                "second": second["render_seconds"],
            },
        },
        "gpu_memory_mib": {"baseline": baseline_mib, "after": after_mib},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh/capture_reference.py"},
        "reads_product_tree": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    (SCENE / "provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    wp.synchronize()
    del rc, m, d
    gc.collect()
    print(
        json.dumps(
            {
                "task_id": TASK_ID,
                "files_sha256": first_hashes,
                "context_sha256": context_sha,
                "ray_stats": first["ray_stats"],
                "image_meta": first["image_meta"],
                "gpu_memory_mib": {"baseline": baseline_mib, "after": after_mib},
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
