"""R09-S3a offline authoring: capture shadow-on / shadow-off RGB/depth/seg raw reference.

OFFLINE ONLY (authoring). Not imported or run by product builds/tests/gates; the
product test chain never starts Python. This script reads only fixtures/ and the
pinned authoring environment (never the product tree), and only writes under
`--out-dir` (default: fixtures/render-bvh/scene-shadow/capture/<group>).

Scene: fixtures/render-bvh/scene-shadow/scene.xml
  Derived from scene-a/scene.xml with exactly one semantic change: the point light
  "lamp" has castshadow="true" (scene-a: false). The scene therefore has >=1
  directional light (sun) and >=1 point light (lamp), both casting shadows.
  scene-a and every older reference are read-only here.

Groups (parametric `--use-shadows`):
  shadow-on  -> --use-shadows true   (out-dir .../capture/shadow-on)
  shadow-off -> --use-shadows false  (out-dir .../capture/shadow-off)

Recipe (parametric `--use-fast-math`, default false = product same recipe):
  The S3a capture froze the fast-math recipe (true); the R09-S3a-capture-exact
  task re-captures the same scene with the product same recipe (false). Only the
  `use_fast_math` parameter changes; scene, cameras, seed and layout are identical.

Two independent captures per invocation (`--repeat 2`, default): the same process
renders twice and every raw output (npz bytes + capture.json payload) must be
byte-identical; otherwise the script exits non-zero with the mismatch details.

Output layout (per --out-dir):
  rgb.npz      arrays rgb_cam0..N    float32 [H, W, 3]
  depth.npz    arrays depth_cam0..N  float32 [H, W]  (raw planar depth, 0 = no hit)
  seg.npz      arrays seg_cam0..N    int32   [H, W, 2] ((object_id, object_type); bg = (-1,-1))
  capture.json provenance + parameters + per-run shas + two-run bitwise verdict

Legacy references: a sha256 manifest of every file under fixtures/render-bvh except
`scene-shadow/` is snapshotted before and after the capture; the run fails if any
of them changed (`legacy_reference_untouched`). `--out-dir` must stay outside the
legacy reference files.

Usage:
  <pinned-python> -B scripts/capture_render_shadow.py --help
  <pinned-python> -B scripts/capture_render_shadow.py --dry-run
  <pinned-python> -B scripts/capture_render_shadow.py --use-shadows true \
      --out-dir fixtures/render-bvh/scene-shadow/capture/shadow-on
  <pinned-python> -B scripts/capture_render_shadow.py --use-shadows true \
      --use-fast-math false \
      --out-dir fixtures/render-bvh/scene-shadow/capture-exact/shadow-on
Env: MJWARP_CAPTURE_AT (ISO8601; share across both groups so capture.json is stable)
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import pathlib
import platform
import subprocess
import sys
import time
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCENE_DIR = ROOT / "fixtures/render-bvh/scene-shadow"
SCENE_XML = SCENE_DIR / "scene.xml"
RENDER_BVH = ROOT / "fixtures/render-bvh"
FIXTURES = ROOT / "fixtures"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
DEFAULT_CACHE = ROOT / "target/render-bvh/s3a-shadow-cache"
TASK_ID = "2026-10-28-render-bvh-r09-s3a-shadow-capture"
SCHEMA = 1
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {
    "warp": "1.15.0",
    "mujoco": "3.15.0",
    "mujoco_warp": "3.15.0",
    "numpy": "2.5.4",
}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
GPU_ARCH = 89
# 28 create_render_context parameters (upstream signature); use_shadows overridden per group.
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


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    """LF-normalized text hash (same algorithm as the S0b fixture hashes)."""
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def fail(message: str) -> None:
    raise SystemExit(f"R09-S3a capture environment/reference mismatch: {message}")


def sha_manifest(root: pathlib.Path, exclude: tuple[pathlib.Path, ...]) -> dict[str, str]:
    """sha256 of every file under root, skipping any root/... path in `exclude`."""
    manifest: dict[str, str] = {}
    if not root.is_dir():
        return manifest
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        resolved = path.resolve()
        if any(resolved.is_relative_to(item) for item in exclude):
            continue
        manifest[path.relative_to(ROOT).as_posix()] = digest(path)
    return manifest


def legacy_manifest() -> dict[str, str]:
    """All existing render-bvh references (scene-a/scene-e2e), excluding scene-shadow."""
    return sha_manifest(RENDER_BVH, (SCENE_DIR.resolve(),))


def check_environment(*, want_cuda_device: bool) -> dict:
    problems = []
    if pathlib.Path(sys.prefix) != VENV:
        problems.append(f"python prefix {sys.prefix} != {VENV}")
    expected_executable = VENV / "Scripts/python.exe"
    if pathlib.Path(sys.executable).resolve() != expected_executable.resolve():
        problems.append(f"python executable {sys.executable} != {expected_executable}")
    import mujoco
    import mujoco_warp as mjw
    import numpy as np
    import warp as wp

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
    upstream_head = run(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"])
    if upstream_head != REVISION:
        problems.append(f"upstream revision {upstream_head} != {REVISION}")
    if run(["git", "-C", str(UPSTREAM), "status", "--porcelain"]):
        problems.append("upstream checkout is dirty")
    if problems:
        fail("; ".join(problems))
    report = {
        "versions": versions,
        "python_executable": sys.executable,
        "python_executable_sha256": digest(pathlib.Path(sys.executable)),
        "python_prefix": sys.prefix,
        "platform": platform.platform(),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "scene_xml": {"path": SCENE_XML.relative_to(ROOT).as_posix(), "sha256": digest(SCENE_XML)},
    }
    if want_cuda_device:
        device = wp.get_device("cuda:0")
        if int(device.arch) != GPU_ARCH:
            fail(f"gpu arch {device.arch} != {GPU_ARCH}")
        if GPU_NAME not in device.name:
            fail(f"gpu name {device.name} != {GPU_NAME}")
        report["gpu"] = {
            "name": device.name,
            "arch": int(device.arch),
            "driver": run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]).splitlines()[0].strip(),
            "cuda_toolkit": list(wp.get_cuda_toolkit_version()),
            "cuda_driver": list(wp.get_cuda_driver_version()),
        }
    return report


def save_npz_deterministic(path: pathlib.Path, arrays: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in sorted(arrays):
            buffer = io.BytesIO()
            np = __import__("numpy")
            np.lib.format.write_array(buffer, np.ascontiguousarray(arrays[name]), allow_pickle=False)
            info = zipfile.ZipInfo(f"{name}.npy", date_time=(1980, 1, 1, 0, 0, 0))
            archive.writestr(info, buffer.getvalue())
    return digest(path)


def render_images(mjm, m, d, rc) -> tuple[dict, dict, float]:
    import numpy as np
    import warp as wp

    started = time.perf_counter()
    import mujoco_warp as mjw

    mjw.render(m, d, rc)
    wp.synchronize()
    elapsed = time.perf_counter() - started
    cam_res = np.asarray(rc.cam_res.numpy())
    cam_ids = np.asarray(rc.cam_id_map.numpy())
    depth_adr = np.asarray(rc.depth_adr.numpy())
    seg_adr = np.asarray(rc.seg_adr.numpy())
    depth_raw = np.asarray(rc.depth_data.numpy())[0].astype(np.float32)
    seg_raw = np.asarray(rc.seg_data.numpy())[0].astype(np.int32)
    images: dict[str, list] = {"rgb": [], "depth": [], "seg": []}
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
        "depth_semantics": "raw rc.depth_data planar depth in meters (0 = no hit); no normalization",
        "seg_semantics": "(object_id, object_type) pairs; background = (-1, -1)",
        "rgb_semantics": "unpacked vec3 in [0, 1] via get_rgb",
    }
    return images, meta, elapsed


def capture_once(mjm, m, d, rc) -> dict:
    import numpy as np

    images, image_meta, render_seconds = render_images(mjm, m, d, rc)
    arrays = {}
    for cam_idx in range(len(images["rgb"])):
        arrays[f"rgb_cam{cam_idx}"] = images["rgb"][cam_idx]
        arrays[f"depth_cam{cam_idx}"] = images["depth"][cam_idx]
        arrays[f"seg_cam{cam_idx}"] = images["seg"][cam_idx]
    for name, values in arrays.items():
        if name.startswith(("rgb_", "depth_")) and not np.isfinite(values).all():
            fail(f"non-finite {name}")
    return {
        "arrays": arrays,
        "image_meta": image_meta,
        "render_seconds": round(render_seconds, 6),
    }


def context_receipt(mjm, rc, files_sha256: dict, capture: dict, run_index: int) -> dict:
    background = int(rc.background_color)
    return {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-shadow-reference",
        "run_index": run_index,
        "parameters": {name: value for name, value in PARAMS.items()},
        "resolved": {
            "nrender": int(rc.nrender),
            "cam_id_map": [int(v) for v in rc.cam_id_map.numpy()],
            "cam_res": [[int(w), int(h)] for w, h in rc.cam_res.numpy()],
            "total_rays": int(rc.total_rays),
            "use_shadows": bool(rc.use_shadows),
            "shadow_light_fraction": float(rc.shadow_light_fraction),
            "use_fast_math": bool(rc.use_fast_math),
            "samples_per_pixel": int(rc.samples_per_pixel),
            "background_color_unpacked": [
                float((background >> 16) & 0xFF),
                float((background >> 8) & 0xFF),
                float(background & 0xFF),
                float((background >> 24) & 0xFF),
            ],
            "znear": float(rc.znear),
            "zfar": float(rc.zfar),
        },
        "lights": {
            "type": [int(v) for v in mjm.light_type],
            "castshadow": [int(v) for v in mjm.light_castshadow],
            "active": [int(v) for v in mjm.light_active],
        },
        "outputs": {
            "files_sha256": files_sha256,
            "rgb_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("rgb_")],
            "depth_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("depth_")],
            "seg_shape": [list(a.shape) for k, a in capture["arrays"].items() if k.startswith("seg_")],
            "image_meta": capture["image_meta"],
        },
        "render_seconds": capture["render_seconds"],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="R09-S3a authoring-only shadow reference capture (pinned MJWarp 3.15).",
        epilog="Outputs rgb.npz/depth.npz/seg.npz/capture.json under --out-dir; legacy references are sha-snapshotted and must stay untouched.",
    )
    parser.add_argument(
        "--use-shadows",
        choices=("true", "false"),
        required=True,
        help="true = shadow-on group, false = shadow-off group",
    )
    parser.add_argument(
        "--use-fast-math",
        choices=("true", "false"),
        default="false",
        help="warp render recipe fast-math switch (default false = product same recipe)",
    )
    parser.add_argument(
        "--out-dir",
        type=pathlib.Path,
        required=True,
        help="new output directory (must not be inside older reference dirs)",
    )
    parser.add_argument("--seed", type=int, default=20261028, help="recorded capture seed")
    parser.add_argument(
        "--repeat",
        type=int,
        choices=(1, 2),
        default=2,
        help="independent capture repeats (2 requires byte-identical raw outputs)",
    )
    parser.add_argument("--scene", type=pathlib.Path, default=SCENE_XML, help="scene XML (default scene-shadow)")
    parser.add_argument("--warp-cache", type=pathlib.Path, default=DEFAULT_CACHE, help="warp kernel cache dir")
    parser.add_argument("--dry-run", action="store_true", help="environment + plan check only; no rendering")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    global PARAMS
    PARAMS = dict(PARAMS_28)
    PARAMS["use_shadows"] = args.use_shadows == "true"
    PARAMS["use_fast_math"] = args.use_fast_math == "true"
    out_dir = args.out_dir.resolve()
    # 旧参考零写入：out-dir 不得落在既有参考目录内。
    if out_dir.is_relative_to(RENDER_BVH) and not out_dir.is_relative_to(SCENE_DIR.resolve()):
        fail(f"--out-dir {out_dir} is inside legacy reference tree {RENDER_BVH}")
    import warp as wp

    wp.config.kernel_cache_dir = str(args.warp_cache)
    if args.dry_run:
        environment = check_environment(want_cuda_device=False)
        plan = {
            "task_id": TASK_ID,
            "dry_run": True,
            "scene": args.scene.resolve().as_posix(),
            "scene_sha256": digest(args.scene.resolve()),
            "use_shadows": PARAMS["use_shadows"],
            "use_fast_math": PARAMS["use_fast_math"],
            "out_dir": out_dir.as_posix(),
            "repeat": args.repeat,
            "seed": args.seed,
            "warp_cache": str(args.warp_cache),
            "expected_outputs": ["rgb.npz", "depth.npz", "seg.npz", "capture.json"],
            "legacy_manifest_entries": len(legacy_manifest()),
            "environment": environment,
        }
        print(json.dumps(plan, indent=2, ensure_ascii=False))
        return 0
    environment = check_environment(want_cuda_device=True)
    legacy_before = legacy_manifest()
    import mujoco
    import mujoco_warp as mjw
    import numpy as np

    print(f"R09-S3a environment OK; legacy reference files={len(legacy_before)}")
    mjm = mujoco.MjModel.from_xml_path(str(args.scene.resolve()))
    m = mjw.put_model(mjm)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    rc = mjw.create_render_context(mjm, **PARAMS)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc) if args.repeat == 2 else first
    # 两次独立捕获的 raw 数组必须逐字节一致（先比数组，再比 npz 字节）。
    for name in sorted(first["arrays"]):
        if not np.array_equal(first["arrays"][name], second["arrays"][name]):
            fail(f"two captures differ in array {name}")
    first_files = {
        "rgb.npz": {k: v for k, v in first["arrays"].items() if k.startswith("rgb_")},
        "depth.npz": {k: v for k, v in first["arrays"].items() if k.startswith("depth_")},
        "seg.npz": {k: v for k, v in first["arrays"].items() if k.startswith("seg_")},
    }
    second_files = {
        "rgb.npz": {k: v for k, v in second["arrays"].items() if k.startswith("rgb_")},
        "depth.npz": {k: v for k, v in second["arrays"].items() if k.startswith("depth_")},
        "seg.npz": {k: v for k, v in second["arrays"].items() if k.startswith("seg_")},
    }
    out_dir.mkdir(parents=True, exist_ok=True)
    first_hashes = {name: save_npz_deterministic(out_dir / name, data) for name, data in first_files.items()}
    tmp_second = out_dir / ".second"
    second_hashes = {name: save_npz_deterministic(tmp_second / name, data) for name, data in second_files.items()}
    if first_hashes != second_hashes:
        fail(f"two captures differ in npz bytes: {first_hashes} vs {second_hashes}")
    import shutil

    shutil.rmtree(tmp_second, ignore_errors=True)
    groups = {
        "shadow-on": args.use_shadows == "true",
        "shadow-off": args.use_shadows == "false",
    }
    capture = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-shadow-capture",
        "group": [name for name, enabled in groups.items() if enabled][0],
        "use_shadows": PARAMS["use_shadows"],
        "seed": args.seed,
        "repeat": args.repeat,
        "two_captures_bitwise_identical": first_hashes == second_hashes,
        "files_sha256": first_hashes,
        "captured_at": __import__("os").environ.get("MJWARP_CAPTURE_AT", ""),
        "environment": environment,
        "parameters": PARAMS,
        "context": context_receipt(mjm, rc, first_hashes, first, 1),
        "context_repeat": context_receipt(mjm, rc, first_hashes, second, 2),
        "legacy_reference_untouched": legacy_manifest() == legacy_before,
        "legacy_reference_files": len(legacy_before),
        "reads_product_tree": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    if not capture["legacy_reference_untouched"]:
        fail("legacy references changed during capture")
    (out_dir / "capture.json").write_text(
        json.dumps(capture, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n"
    )
    print(
        json.dumps(
            {
                "task_id": TASK_ID,
                "group": capture["group"],
                "out_dir": out_dir.as_posix(),
                "files_sha256": first_hashes,
                "capture_json_sha256": digest(out_dir / "capture.json"),
                "two_captures_bitwise_identical": capture["two_captures_bitwise_identical"],
                "legacy_reference_untouched": capture["legacy_reference_untouched"],
            },
            indent=2,
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
