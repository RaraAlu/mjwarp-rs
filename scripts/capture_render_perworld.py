"""Offline authoring: capture per-world (nworld > 1) render references.

OFFLINE ONLY (authoring). Not imported or run by product builds/tests/gates.
Reads only fixtures/ and the pinned authoring environment; writes only under
`--out-dir` (must stay inside fixtures/render-bvh/scene-a/perworld/).

Why: the architecture gap `2026-10-28-render-per-world-geom-light-gap` says our
device path indexes geom/light state with a single-world stride while the pinned
upstream render samples `geom_xpos_in[worldid, gi]`. This script captures a
reference where each world has a different body pose, using pinned upstream APIs
that support nworld natively (put_data(nworld=N), forward, create_render_context(
nworld=N), get_rgb/get_depth/get_seg -> (nworld, H, W)); see README for the exact
source lines. Lights are captured for completeness only: the device path still
indexes lights single-world, so per-world light poses are NOT claimed here.

Recipe (product same-recipe): `--use-fast-math false`, `--use-shadows true`,
samples_per_pixel=1, fixed cameras only (cam_mode == 0). Worlds: `--worlds N`
(default 3) with `--hinge-deg "0,20,40"` written into each world's hinge qpos.
Two captures per invocation (`--repeat 2`) must be byte-identical.

Outputs (per --out-dir): rgb.npz/depth.npz/seg.npz arrays w{w}_cam{c}, state.npz
(qpos, geom_xpos, geom_xmat, light_xpos, light_xdir), capture.json provenance.
Legacy references: sha256 manifest of fixtures/render-bvh except scene-a/perworld/,
snapshotted before/after; any change fails the run.

Usage:
  <pinned-python> -B scripts/capture_render_perworld.py --help
  <pinned-python> -B scripts/capture_render_perworld.py --dry-run \
      --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
  <pinned-python> -B scripts/capture_render_perworld.py --dry-run \
      --scene fixtures/render-bvh/scene-a-light/scene.xml \
      --out-root fixtures/render-bvh/scene-a-light/perworld \
      --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040
  <pinned-python> -B scripts/capture_render_perworld.py \
      --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
Env: MJWARP_CAPTURE_AT (ISO8601; share across invocations so capture.json is stable)
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

from perworld_light_prep import build_dry_run_plan
from perworld_capture_meta import capture_metadata, dry_run_echo, verify_capture_metadata

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCENE_DIR = ROOT / "fixtures/render-bvh/scene-a"
SCENE_XML = SCENE_DIR / "scene.xml"
RENDER_BVH = ROOT / "fixtures/render-bvh"
OUT_ROOT = SCENE_DIR / "perworld"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
DEFAULT_CACHE = ROOT / "target/render-bvh/perworld-cache"
TASK_ID = "2026-10-30-render-perworld-capture-authoring"
SCHEMA = 1
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0",
                     "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
GPU_ARCH = 89
DEFAULT_WORLDS = 3
DEFAULT_HINGE_DEG = "0,20,40"
# 28 create_render_context parameters (upstream signature); nworld/fast-math/shadows
# are overridden from the CLI so the recipe block always records what ran.
PARAMS_28 = {
    "nworld": 1, "cam_res": None, "render_rgb": True, "render_depth": True,
    "render_seg": True, "use_textures": True, "use_fast_math": True,
    "use_shadows": False, "use_ambient_lighting": True,
    "enabled_geom_groups": [0, 1, 2, 3], "cam_active": None,
    "background_color": [0.0, 0.0, 0.0, 1.0], "flex_render_smooth": True,
    "use_precomputed_rays": True, "render_skybox": False,
    "enable_backface_culling": True, "shadow_light_fraction": 0.3,
    "samples_per_pixel": 1, "enable_vertex_normals": True,
    "enable_specular": True, "enable_emission": True,
    "enable_per_light_ambient": True, "splat_position": None,
    "splat_rotation": None, "splat_scale": None, "splat_rgba": None,
    "splat_adr": None, "splat_group_id": None,
}
PARAMS = dict(PARAMS_28)

def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()

def fail(message: str) -> None:
    raise SystemExit(f"perworld capture environment/reference mismatch: {message}")

def sha_manifest(root: pathlib.Path, exclude: tuple[pathlib.Path, ...]) -> dict[str, str]:
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

def legacy_manifest(out_root: pathlib.Path) -> dict[str, str]:
    """Existing render-bvh references, excluding the effective per-world output subtree."""
    return sha_manifest(RENDER_BVH, (out_root.resolve(),))

def check_environment(*, want_cuda_device: bool, scene_path: pathlib.Path) -> dict:
    problems = []
    if pathlib.Path(sys.prefix) != VENV:
        problems.append(f"python prefix {sys.prefix} != {VENV}")
    expected_executable = VENV / "Scripts/python.exe"
    if pathlib.Path(sys.executable).resolve() != expected_executable.resolve():
        problems.append(f"python executable {sys.executable} != {expected_executable}")
    import inspect

    import mujoco
    import mujoco_warp as mjw
    import numpy as np
    import warp as wp

    versions = {"python": platform.python_version(), "warp": wp.__version__,
                "mujoco": mujoco.__version__, "mujoco_warp": mjw.__version__,
                "numpy": np.__version__}
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            problems.append(f"{name} {versions[name]} != {expected}")
    upstream_head = run(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"])
    if upstream_head != REVISION:
        problems.append(f"upstream revision {upstream_head} != {REVISION}")
    if run(["git", "-C", str(UPSTREAM), "status", "--porcelain"]):
        problems.append("upstream checkout is dirty")

    # Per-world capability contract: the whole authoring rests on these signatures.
    put_data_params = list(inspect.signature(mjw.put_data).parameters)
    if "nworld" not in put_data_params:
        problems.append("mujoco_warp.put_data has no nworld parameter")
    ctx_params = list(inspect.signature(mjw.create_render_context).parameters)
    if "nworld" not in ctx_params:
        problems.append("create_render_context has no nworld parameter")
    for name in ("get_rgb", "get_depth", "get_segmentation"):
        if not hasattr(mjw, name):
            problems.append(f"mujoco_warp.{name} missing")

    camera_modes = None
    resolved_scene = scene_path.resolve()
    if not resolved_scene.is_file():
        problems.append(f"scene xml missing: {resolved_scene}")
    else:
        mjm = mujoco.MjModel.from_xml_path(str(resolved_scene))
        camera_modes = [int(mode) for mode in mjm.cam_mode]
        if not any(mode == 0 for mode in camera_modes):
            problems.append("scene has no fixed camera (cam_mode == 0)")

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
        "scene_xml": {"path": resolved_scene.relative_to(ROOT).as_posix(),
                      "sha256": digest(resolved_scene), "bytes": resolved_scene.stat().st_size},
        "perworld_api": {
            "put_data_params": put_data_params,
            "create_render_context_params": ctx_params,
            "get_rgb_depth_seg": True,
        },
        "camera_modes": camera_modes,
    }
    if want_cuda_device:
        device = wp.get_device("cuda:0")
        if int(device.arch) != GPU_ARCH:
            fail(f"gpu arch {device.arch} != {GPU_ARCH}")
        if GPU_NAME not in device.name:
            fail(f"gpu name {device.name} != {GPU_NAME}")
        report["gpu"] = {
            "name": device.name, "arch": int(device.arch),
            "driver": run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]
                          ).splitlines()[0].strip(),
            "cuda_toolkit": list(wp.get_cuda_toolkit_version()),
            "cuda_driver": list(wp.get_cuda_driver_version()),
        }
    return report

def hinge_qpos(mjm):
    """Model default qpos (qpos0) plus the single hinge joint; FREE/BALL are not scalar."""
    import numpy as np
    scalar = [int(j) for j in range(int(mjm.njnt)) if int(mjm.jnt_type[j]) in (2, 3)]
    hinges = [j for j in scalar if int(mjm.jnt_type[j]) == 3]
    if len(hinges) != 1:
        fail(f"expected exactly one hinge joint, found scalar={scalar} hinge={hinges}")
    return np.array(mjm.qpos0, dtype=np.float64), hinges[0]

def build_world_qpos(mjm, world_deg: list[float]):
    import numpy as np
    base, joint = hinge_qpos(mjm)
    adr = int(mjm.jnt_qposadr[joint])
    if not 0 <= adr < base.size:
        fail(f"hinge qpos address {adr} outside qpos size {base.size}")
    out = np.repeat(base.astype(np.float32)[None, :], len(world_deg), axis=0)
    for world, deg in enumerate(world_deg):
        out[world][adr] = float(np.deg2rad(deg))
    return out

def camera_recipe(mjm, rc) -> list[dict]:
    cameras = []
    cam_res = rc.cam_res.numpy()
    for slot, cam_id in enumerate(int(v) for v in rc.cam_id_map.numpy()):
        name = mjm.camera(cam_id).name or f"cam{cam_id}"
        cameras.append({"slot": slot, "mujoco_cam_id": int(cam_id), "name": name,
                        "mode": int(mjm.cam_mode[cam_id]),
                        "xpos": [float(v) for v in mjm.cam_pos[cam_id]],
                        "xquat": [float(v) for v in mjm.cam_quat[cam_id]],
                        "fovy": float(mjm.cam_fovy[cam_id]),
                        "resolution": [int(cam_res[slot][0]), int(cam_res[slot][1])]})
    return cameras

def save_npz_deterministic(path: pathlib.Path, arrays: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in sorted(arrays):
            buffer = io.BytesIO()
            np = __import__("numpy")
            np.lib.format.write_array(buffer, np.ascontiguousarray(arrays[name]),
                                      allow_pickle=False)
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
    nworld = int(rc.nworld) if hasattr(rc, "nworld") else int(d.nworld)
    cam_res = np.asarray(rc.cam_res.numpy())
    cam_ids = np.asarray(rc.cam_id_map.numpy())
    depth_adr = np.asarray(rc.depth_adr.numpy())
    seg_adr = np.asarray(rc.seg_adr.numpy())
    depth_raw = np.asarray(rc.depth_data.numpy()).astype(np.float32)
    seg_raw = np.asarray(rc.seg_data.numpy()).astype(np.int32)
    images: dict[str, list] = {"rgb": [], "depth": [], "seg": []}
    for idx in range(int(rc.nrender)):
        width, height = int(cam_res[idx][0]), int(cam_res[idx][1])
        rgb = wp.zeros((nworld, height, width), dtype=wp.vec3)
        mjw.get_rgb(rc, idx, rgb)
        wp.synchronize()
        pixels = width * height
        images["rgb"].append(np.asarray(rgb.numpy()).astype(np.float32))
        depth_all = depth_raw.reshape(nworld, -1)
        seg_all = seg_raw.reshape(nworld, -1, 2)
        images["depth"].append(
            depth_all[:, int(depth_adr[idx]) : int(depth_adr[idx]) + pixels].reshape(
                nworld, height, width))
        images["seg"].append(
            seg_all[:, int(seg_adr[idx]) : int(seg_adr[idx]) + pixels].reshape(
                nworld, height, width, 2))
    meta = {
        "nworld": nworld,
        "cam_ids": [int(v) for v in cam_ids],
        "cam_res": [[int(w), int(h)] for w, h in cam_res],
        "depth_semantics": "raw rc.depth_data planar depth in meters (0 = no hit); no normalization",
        "seg_semantics": "(object_id, object_type) pairs; background = (-1, -1)",
        "rgb_semantics": "unpacked vec3 in [0, 1] via get_rgb, shape (nworld, H, W)",
    }
    return images, meta, elapsed

def state_arrays(d) -> dict:
    import numpy as np
    return {
        "qpos": np.asarray(d.qpos.numpy()).astype(np.float32),
        "geom_xpos": np.asarray(d.geom_xpos.numpy()).astype(np.float32),
        "geom_xmat": np.asarray(d.geom_xmat.numpy()).astype(np.float32),
        "light_xpos": np.asarray(d.light_xpos.numpy()).astype(np.float32),
        "light_xdir": np.asarray(d.light_xdir.numpy()).astype(np.float32),
    }

def npz_payload(capture: dict) -> dict:
    payload = {f"{kind}.npz": {k: v for k, v in capture["arrays"].items()
                               if k.startswith(f"{kind}_")}
               for kind in ("rgb", "depth", "seg")}
    payload["state.npz"] = capture["state"]
    return payload

def capture_once(mjm, m, d, rc) -> dict:
    import numpy as np
    images, image_meta, render_seconds = render_images(mjm, m, d, rc)
    nworld = len(images["rgb"][0])
    arrays = {}
    for cam_idx in range(len(images["rgb"])):
        for world in range(nworld):
            arrays[f"rgb_w{world}_cam{cam_idx}"] = images["rgb"][cam_idx][world]
            arrays[f"depth_w{world}_cam{cam_idx}"] = images["depth"][cam_idx][world]
            arrays[f"seg_w{world}_cam{cam_idx}"] = images["seg"][cam_idx][world]
    for name, values in arrays.items():
        if name.startswith(("rgb_", "depth_")) and not np.isfinite(values).all():
            fail(f"non-finite {name}")
    return {"arrays": arrays, "state": state_arrays(d), "image_meta": image_meta,
            "render_seconds": round(render_seconds, 6)}

def context_receipt(mjm, rc, files_sha256: dict, capture: dict, run_index: int) -> dict:
    return {
        "schema": SCHEMA, "task_id": TASK_ID, "kind": "render-perworld-context",
        "run_index": run_index,
        "parameters": dict(PARAMS),
        "resolved": {
            "nworld": int(PARAMS["nworld"]),
            "nrender": int(rc.nrender),
            "cam_id_map": [int(v) for v in rc.cam_id_map.numpy()],
            "cam_res": [[int(w), int(h)] for w, h in rc.cam_res.numpy()],
            "total_rays": int(rc.total_rays),
            "use_shadows": bool(rc.use_shadows),
            "use_fast_math": bool(rc.use_fast_math),
            "samples_per_pixel": int(rc.samples_per_pixel),
            "lights": {"type": [int(v) for v in mjm.light_type],
                       "castshadow": [int(v) for v in mjm.light_castshadow],
                       "active": [int(v) for v in mjm.light_active]},
        },
        "outputs": {
            "files_sha256": files_sha256,
            "rgb_shape": [list(a.shape) for k, a in capture["arrays"].items()
                          if k.startswith("rgb_")],
            "state_shape": {k: list(v.shape) for k, v in capture["state"].items()},
            "image_meta": capture["image_meta"],
        },
        "render_seconds": capture["render_seconds"],
    }

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out-dir", type=pathlib.Path, required=True,
                        help="new output directory (must stay inside --out-root)")
    parser.add_argument("--scene", type=pathlib.Path, default=SCENE_XML,
                        help="scene XML; default fixtures/render-bvh/scene-a/scene.xml")
    parser.add_argument("--out-root", type=pathlib.Path, default=OUT_ROOT,
                        help="output root; default fixtures/render-bvh/scene-a/perworld")
    parser.add_argument("--reference-tag", default=None,
                        help="reference tag; default = reference directory name (spec v2 §2.2)")
    parser.add_argument("--worlds", type=int, default=DEFAULT_WORLDS,
                        help="number of worlds (nworld); default 3")
    parser.add_argument("--hinge-deg", default=DEFAULT_HINGE_DEG,
                        help="per-world hinge angles in degrees, comma separated; default 0,20,40")
    parser.add_argument("--use-fast-math", choices=("true", "false"), default="false")
    parser.add_argument("--use-shadows", choices=("true", "false"), default="true")
    parser.add_argument("--seed", type=int, default=20261030)
    parser.add_argument("--repeat", type=int, choices=(1, 2), default=2,
                        help="capture repeats (2 requires byte-identical outputs)")
    parser.add_argument("--warp-cache", type=pathlib.Path, default=DEFAULT_CACHE)
    parser.add_argument("--dry-run", action="store_true",
                        help="environment + plan check only; no rendering, no cuda device")
    return parser.parse_args()

def main() -> int:
    args = parse_args()
    global PARAMS
    world_deg = [float(v) for v in args.hinge_deg.split(",") if v.strip()]
    if args.worlds < 2:
        fail("--worlds must be >= 2 (this is a per-world reference)")
    if len(world_deg) != args.worlds:
        fail(f"--hinge-deg has {len(world_deg)} values but --worlds is {args.worlds}")
    PARAMS = dict(PARAMS_28)
    PARAMS["nworld"] = args.worlds
    PARAMS["use_fast_math"] = args.use_fast_math == "true"
    PARAMS["use_shadows"] = args.use_shadows == "true"
    scene_path = args.scene.resolve()
    out_root = args.out_root.resolve()
    out_dir = args.out_dir.resolve()
    if not out_dir.is_relative_to(out_root):
        fail(f"--out-dir {out_dir} is outside --out-root {out_root}")
    if out_dir.exists() and any(out_dir.iterdir()):
        fail(f"--out-dir {out_dir} already exists and is not empty (references are append-only)")
    provenance = None if args.dry_run else capture_metadata([sys.executable, *sys.argv], pathlib.Path.cwd(), args.reference_tag, out_dir)
    import warp as wp

    wp.config.kernel_cache_dir = str(args.warp_cache)
    script_path = pathlib.Path(__file__).resolve()
    recipe = {
        "task_id": TASK_ID,
        "scene_xml": {"path": scene_path.relative_to(ROOT).as_posix(),
                      "sha256": digest(scene_path), "bytes": scene_path.stat().st_size},
        "script": {"path": script_path.relative_to(ROOT).as_posix(),
                   "sha256": digest(script_path), "bytes": script_path.stat().st_size},
        "npz_files": ["rgb.npz", "depth.npz", "seg.npz", "state.npz"],
        "nworld": args.worlds,
        "hinge_deg": world_deg,
        "use_fast_math": PARAMS["use_fast_math"], "use_shadows": PARAMS["use_shadows"],
        "seed": args.seed, "repeat": args.repeat,
        "python": {"executable": sys.executable, "version": platform.python_version()},
        "versions": EXPECTED_VERSIONS, "upstream_revision": REVISION,
        "capture_at_env": "",
    }
    if args.dry_run:
        environment = check_environment(want_cuda_device=False, scene_path=scene_path)
        import mujoco

        mjm = mujoco.MjModel.from_xml_path(str(scene_path))
        world_qpos = build_world_qpos(mjm, world_deg)
        plan = build_dry_run_plan(
            root=ROOT, task_id=TASK_ID, recipe=recipe, environment=environment,
            mjm=mjm, world_qpos=world_qpos, world_deg=world_deg, out_dir=out_dir,
            scene_path=scene_path, out_root=out_root, repeat=args.repeat, seed=args.seed,
            warp_cache=str(args.warp_cache),
            legacy_manifest_entries=len(legacy_manifest(out_root)))
        plan.update(dry_run_echo([sys.executable, *sys.argv], pathlib.Path.cwd(), args.reference_tag, out_dir))
        print(json.dumps(plan, indent=2, ensure_ascii=False))
        return 0

    environment = check_environment(want_cuda_device=True, scene_path=scene_path)
    legacy_before = legacy_manifest(out_root)
    import mujoco
    import mujoco_warp as mjw
    import numpy as np
    print(f"perworld environment OK; legacy reference files={len(legacy_before)}")
    mjm = mujoco.MjModel.from_xml_path(str(scene_path))
    m = mjw.put_model(mjm)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    d = mjw.put_data(mjm, mjd, nworld=args.worlds)
    world_qpos = build_world_qpos(mjm, world_deg)
    wp.copy(d.qpos, wp.array(world_qpos, dtype=float, device=d.qpos.device))
    mjw.forward(m, d)
    wp.synchronize()
    PARAMS["cam_active"] = [int(mode) == 0 for mode in mjm.cam_mode]
    rc = mjw.create_render_context(mjm, **PARAMS)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc) if args.repeat == 2 else first
    for name in sorted(first["arrays"]):
        if not np.array_equal(first["arrays"][name], second["arrays"][name]):
            fail(f"two captures differ in array {name}")
    for name in sorted(first["state"]):
        if not np.array_equal(first["state"][name], second["state"][name]):
            fail(f"two captures differ in state {name}")
    first_files = npz_payload(first)
    second_files = npz_payload(second)
    out_dir.mkdir(parents=True, exist_ok=True)
    first_hashes = {name: save_npz_deterministic(out_dir / name, data)
                    for name, data in first_files.items()}
    tmp_second = out_dir / ".second"
    second_hashes = {name: save_npz_deterministic(tmp_second / name, data)
                     for name, data in second_files.items()}
    if first_hashes != second_hashes:
        fail(f"two captures differ in npz bytes: {first_hashes} vs {second_hashes}")
    import shutil
    shutil.rmtree(tmp_second, ignore_errors=True)
    recipe["capture_at_env"] = provenance["captured_at"]
    recipe["cameras"] = camera_recipe(mjm, rc)
    capture = {
        **provenance,
        "schema": SCHEMA, "task_id": TASK_ID, "kind": "render-perworld-capture",
        "nworld": args.worlds, "hinge_deg": world_deg,
        "use_shadows": PARAMS["use_shadows"], "use_fast_math": PARAMS["use_fast_math"],
        "seed": args.seed, "repeat": args.repeat,
        "two_captures_bitwise_identical": first_hashes == second_hashes,
        "files_sha256": first_hashes,
        "environment": environment, "recipe": recipe, "parameters": PARAMS,
        "context": context_receipt(mjm, rc, first_hashes, first, 1),
        "legacy_reference_untouched": legacy_manifest(out_root) == legacy_before,
        "legacy_reference_files": len(legacy_before),
        "reads_product_tree": False, "authoring_only": True,
        "product_python_dependency": False,
    }
    if not capture["legacy_reference_untouched"]:
        fail("legacy references changed during capture")
    (out_dir / "capture.json").write_text(
        json.dumps(capture, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8", newline="\n")
    verify_capture_metadata(capture)
    print(json.dumps({
        "task_id": TASK_ID, "out_dir": out_dir.as_posix(),
        "files_sha256": first_hashes,
        "capture_json_sha256": digest(out_dir / "capture.json"),
        "script_sha256": recipe["script"]["sha256"],
        "scene_sha256": recipe["scene_xml"]["sha256"],
        "nworld": args.worlds, "hinge_deg": world_deg,
        "two_captures_bitwise_identical": capture["two_captures_bitwise_identical"],
        "legacy_reference_untouched": capture["legacy_reference_untouched"],
    }, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
