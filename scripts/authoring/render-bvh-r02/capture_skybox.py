"""Offline authoring: capture the R02-S1 skybox reference (MuJoCo 3.15 / MJWarp 3.15).

OFFLINE ONLY (GPU stage A of the r02-skybox window). Not imported or run by product
builds/tests/gates; the product test chain never starts Python. Reads only fixtures/
and the pinned authoring environment (never the product tree).

The only difference from `scripts/authoring/render-bvh/capture_reference.py` (S0b,
frozen) is `render_skybox=True` plus a separate output directory, so the existing
S0b reference stays byte-identical and is NOT re-captured.

Outputs (fixtures/render-bvh/scene-a/reference-skybox/):
  rgb.npz depth.npz seg.npz capture.json
and prints one CATCH_DONE line. Every environment mismatch fails loudly (no fallback).

Usage:
  # offline self-check only (no GPU, no imports of warp/mujoco_warp):
  python scripts/authoring/render-bvh-r02/capture_skybox.py --dry-run
  # GPU authoring (stage A, only inside the signed window):
  <pinned venv python> scripts/authoring/render-bvh-r02/capture_skybox.py
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import pathlib
import platform
import sys
import time
import zipfile

try:  # numpy is only needed for the GPU path; --dry-run must work without it
    import numpy as np
except ImportError:  # pragma: no cover - dry-run only
    np = None  # type: ignore[assignment]

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
OUT_DIR = SCENE / "reference-skybox"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/r02-skybox-cache"
TASK_ID = "2026-10-29-render-bvh-r02-skybox-s2-capture-authoring"
CAPTURE_SCHEMA = 1

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

# S0b PARAMS_28 with the single intended change: render_skybox=True.
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
    "render_skybox": True,
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
EXPECTED_PIXELS = 320 * 240 + 240 * 180  # 120000 = cam-persp + cam-track


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    """LF-normalized text hash (same convention as model_parity.py)."""
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def fail(message: str) -> None:
    raise SystemExit(f"R02-S1 skybox capture mismatch: {message}")


def check_static_assets() -> dict:
    """Asset-level checks that need no GPU and no pinned imports (dry-run + real run)."""
    problems = []
    if not VENV.is_dir():
        problems.append(f"pinned venv missing: {VENV}")
    if not UPSTREAM.is_dir():
        problems.append(f"upstream checkout missing: {UPSTREAM}")
    if not MJB.is_file():
        problems.append(f"missing {MJB}")
    elif digest(MJB) != MJB_SHA256:
        problems.append("scene-315.mjb hash mismatch")
    if not XML.is_file():
        problems.append(f"missing {XML}")
    elif digest(XML) != XML_SHA256 and digest_text(XML) != XML_SHA256:
        problems.append("scene.xml hash mismatch")
    if PARAMS_28.get("render_skybox") is not True:
        problems.append("render_skybox must be True")
    if PARAMS_28["splat_position"] is not None:
        problems.append("splat params must stay None in this slice")
    if PARAMS_28["nworld"] != 1 or PARAMS_28["samples_per_pixel"] != 1:
        problems.append("nworld/spp must stay 1")
    if problems:
        fail("; ".join(problems))
    return {
        "scene_mjb_sha256": MJB_SHA256,
        "scene_xml_sha256": XML_SHA256,
        "params28_sha256": hashlib.sha256(
            json.dumps(PARAMS_28, sort_keys=True).encode("utf-8")
        ).hexdigest(),
        "expected_pixels": EXPECTED_PIXELS,
    }


def check_environment() -> dict:
    import warp as wp  # noqa: PLC0415
    import mujoco  # noqa: PLC0415
    import mujoco_warp as mjw  # noqa: PLC0415

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
    up = subprocess_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"])
    if up != REVISION:
        problems.append("upstream revision mismatch")
    if subprocess_output(["git", "-C", str(UPSTREAM), "status", "--porcelain"]):
        problems.append("upstream checkout is dirty")
    device = wp.get_device("cuda:0")
    if int(device.arch) != GPU_ARCH:
        problems.append(f"gpu arch {device.arch} != {GPU_ARCH}")
    if GPU_NAME not in device.name:
        problems.append(f"gpu name {device.name} != {GPU_NAME}")
    driver = subprocess_output(
        ["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]
    ).splitlines()[0].strip()
    if driver != GPU_DRIVER:
        problems.append(f"gpu driver {driver} != {GPU_DRIVER}")
    if problems:
        fail("; ".join(problems))
    return {
        "versions": versions,
        "python_executable": sys.executable,
        "python_prefix": sys.prefix,
        "platform": platform.platform(),
        "gpu": device.name,
        "gpu_arch": int(device.arch),
        "gpu_driver": driver,
        "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "reference_code": {
            name: digest(UPSTREAM / "mujoco_warp" / "_src" / name) for name in REFERENCE_CODE
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def subprocess_output(command: list[str]) -> str:
    import subprocess  # noqa: PLC0415

    return subprocess.check_output(command, text=True).strip()


def load_state():
    import mujoco  # noqa: PLC0415
    import mujoco_warp as mjw  # noqa: PLC0415
    import warp as wp  # noqa: PLC0415

    wp.config.kernel_cache_dir = str(CACHE)
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    return mjm, m, d


def skybox_meta(mjm) -> dict:
    import mujoco  # noqa: PLC0415

    ids = np.nonzero(mjm.tex_type == mujoco.mjtTexture.mjTEXTURE_SKYBOX)[0]
    if ids.size != 1:
        fail(f"expected exactly 1 skybox texture, got {int(ids.size)}")
    tex_id = int(ids[0])
    return {
        "tex_type": [int(v) for v in np.asarray(mjm.tex_type).reshape(-1)],
        "tex_width": [int(v) for v in np.asarray(mjm.tex_width).reshape(-1)],
        "tex_height": [int(v) for v in np.asarray(mjm.tex_height).reshape(-1)],
        "tex_nchannel": [int(v) for v in np.asarray(mjm.tex_nchannel).reshape(-1)],
        "skybox_tex_id": tex_id,
        "skybox_face_width": int(np.asarray(mjm.tex_width).reshape(-1)[tex_id]),
        "skybox_tex_height": int(np.asarray(mjm.tex_height).reshape(-1)[tex_id]),
        "layout": "vertical 2D strip: tex_height = 6 * tex_width, faces +X,-X,+Y,-Y,+Z,-Z",
    }


def render_images(mjm, m, d, rc) -> tuple[dict, dict, float]:
    import mujoco_warp as mjw  # noqa: PLC0415
    import warp as wp  # noqa: PLC0415

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
        "depth_semantics": "raw rc.depth_data planar depth in meters (0 = no hit)",
        "seg_semantics": "(object_id, object_type) pairs; background = (-1, -1)",
        "rgb_semantics": "unpacked vec3 in [0, 1] via get_rgb",
    }
    return images, meta, elapsed


def npz_bytes(arrays: dict) -> tuple[dict, bytes]:
    payload = {}
    for name in ("rgb", "depth", "seg"):
        buffer = io.BytesIO()
        for key in sorted(arrays):
            if not key.startswith(f"{name}_"):
                continue
            np.lib.format.write_array(
                buffer, np.ascontiguousarray(arrays[key]), allow_pickle=False
            )
        payload[f"{name}.npz"] = buffer.getvalue()
    return payload


def write_npz(path: pathlib.Path, arrays: dict, prefix: str) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for key in sorted(arrays):
            if not key.startswith(f"{prefix}_"):
                continue
            buffer = io.BytesIO()
            np.lib.format.write_array(
                buffer, np.ascontiguousarray(arrays[key]), allow_pickle=False
            )
            info = zipfile.ZipInfo(f"{key}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            archive.writestr(info, buffer.getvalue())
    return digest(path)


def capture_once(mjm, m, d) -> dict:
    import mujoco_warp as mjw  # noqa: PLC0415

    rc = mjw.create_render_context(mjm, **PARAMS_28)
    if not bool(rc.render_skybox):
        fail("pinned create_render_context did not enable render_skybox")
    images, image_meta, render_seconds = render_images(mjm, m, d, rc)
    arrays = {}
    for cam_idx in range(len(images["rgb"])):
        arrays[f"rgb_cam{cam_idx}"] = images["rgb"][cam_idx]
        arrays[f"depth_cam{cam_idx}"] = images["depth"][cam_idx]
        arrays[f"seg_cam{cam_idx}"] = images["seg"][cam_idx]
    total_pixels = sum(int(w) * int(h) for w, h in image_meta["cam_res"])
    if total_pixels != EXPECTED_PIXELS:
        fail(f"pixel count {total_pixels} != {EXPECTED_PIXELS}")
    for name in ("rgb", "depth", "seg"):
        for array in images[name]:
            if not np.isfinite(array).all():
                fail(f"non-finite {name} image")
    return {
        "arrays": arrays,
        "image_meta": image_meta,
        "render_seconds": round(render_seconds, 6),
        "rc": {
            "render_skybox": bool(rc.render_skybox),
            "skybox_tex_id": [int(v) for v in np.asarray(rc.skybox_tex_id.numpy()).reshape(-1)],
            "skybox_face_width": [
                int(v) for v in np.asarray(rc.skybox_face_width.numpy()).reshape(-1)
            ],
            "splat_count": int(rc.splat_count),
            "use_textures": bool(PARAMS_28["use_textures"]),
        },
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="capture R02-S1 skybox reference")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="asset/param self-check only (no GPU, no warp/mujoco_warp import)",
    )
    args = parser.parse_args(argv)
    static = check_static_assets()
    if args.dry_run:
        print(
            "DRY_RUN_OK task_id={} render_skybox=True expected_pixels={} "
            "params28_sha256={}".format(TASK_ID, static["expected_pixels"], static["params28_sha256"])
        )
        return 0

    environment = check_environment()
    mjm, m, d = load_state()
    meta = skybox_meta(mjm)
    first = capture_once(mjm, m, d)
    second = capture_once(mjm, m, d)
    payload_first, payload_second = npz_bytes(first["arrays"]), npz_bytes(second["arrays"])
    first_shas = {
        name: hashlib.sha256(blob).hexdigest() for name, blob in payload_first.items()
    }
    second_shas = {
        name: hashlib.sha256(blob).hexdigest() for name, blob in payload_second.items()
    }
    if first_shas != second_shas:
        fail(f"two-run sha mismatch: {first_shas} vs {second_shas}")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    file_shas = {
        "rgb.npz": write_npz(OUT_DIR / "rgb.npz", first["arrays"], "rgb"),
        "depth.npz": write_npz(OUT_DIR / "depth.npz", first["arrays"], "depth"),
        "seg.npz": write_npz(OUT_DIR / "seg.npz", first["arrays"], "seg"),
    }
    for name, expected in first_shas.items():
        if file_shas[name] != expected:
            fail(f"written {name} sha differs from in-memory sha")
    capture = {
        "schema": CAPTURE_SCHEMA,
        "task_id": TASK_ID,
        "kind": "r02-s1-skybox-reference",
        "captured_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "scene": {
            "mjb": "fixtures/render-bvh/scene-a/scene-315.mjb",
            "mjb_sha256": static["scene_mjb_sha256"],
            "xml": "fixtures/render-bvh/scene-a/scene.xml",
            "xml_sha256": static["scene_xml_sha256"],
            "model_315": "fixtures/render-bvh/scene-a/model-315.json",
        },
        "params28": PARAMS_28,
        "params28_sha256": static["params28_sha256"],
        "skybox": meta,
        "rc": first["rc"],
        "cameras": first["image_meta"],
        "expected_pixels": EXPECTED_PIXELS,
        "files": [
            {"path": f"fixtures/render-bvh/scene-a/reference-skybox/{name}", "sha256": sha}
            for name, sha in file_shas.items()
        ],
        "determinism": {
            "two_runs": True,
            "sha_equal": True,
            "per_file_sha256": file_shas,
        },
        "render_seconds": {"run1": first["render_seconds"], "run2": second["render_seconds"]},
        "environment": environment,
        "not_product_acceptance": True,
    }
    capture_path = OUT_DIR / "capture.json"
    capture_path.write_text(
        json.dumps(capture, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n"
    )
    print(
        "CATCH_DONE task_id={} files={} capture_sha256={} two_run_sha_equal=True".format(
            TASK_ID, len(file_shas), digest(capture_path)
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
