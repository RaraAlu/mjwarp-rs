"""R06-S3b offline authoring: capture 1D flex render reference (rgb/depth/seg + state).

OFFLINE ONLY (authoring): not imported or run by product builds/tests/gates; reads only
fixtures/ + pinned authoring env (never the product tree); writes only under --out-dir
(default fixtures/render-bvh/scene-flex/reference/<group>).
Scene: 1D flex + rigid geometry (flex never wins a near-coplanar tie, so clearance >=0.3;
1D reference contract: geom_id=-2, mesh_id=flexid, seg=(flexid,9) first sample,
depth = dist * -ray_dir_local_cam[2], color = flex_rgba[mesh_id]).
Recipe: pinned editable upstream (PyPI wheels rejected: 5 .py files differ); product-same
recipe = use_fast_math false + use_shadows true. `--repeat 2` requires byte-identical runs.
Outputs: rgb.npz / depth.npz / seg.npz / state.npz (qpos/geom_xpos/geom_xmat/light_xpos/
light_xdir/flexvert_xpos) / capture.json / cam-poses.json (data camera poses, f32 bit provenance).
Legacy fixtures sha-snapshotted before/after.
Usage: <pinned-python> -B scripts/authoring/capture_render_flex.py --dry-run | --group base
       <pinned-python> -B scripts/authoring/capture_render_flex.py --cam-poses-only
Env: MJWARP_CAPTURE_AT (ISO8601; fixes capture.json provenance)
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import pathlib
import platform
import subprocess
import sys
import time
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCENE_DIR = ROOT / "fixtures/render-bvh/scene-flex"
SCENE_XML = SCENE_DIR / "scene.xml"
RENDER_BVH = ROOT / "fixtures/render-bvh"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
DEFAULT_CACHE = ROOT / "target/render-bvh/s3b-flex-cache"
TASK_ID = "2026-11-04-render-r06-s3b-flex-capture"
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
FLEX_FIELDS = [
    "flex_dim", "flex_radius", "flex_vertadr", "flex_vertnum", "flex_vertbodyid",
    "flex_edgeadr", "flex_edgenum", "flex_edge", "flex_elemadr", "flex_elemnum",
    "flex_elemdataadr", "flex_elem", "flex_shellnum", "flex_shelldataadr", "flex_shell",
]

PARAMS = {
    "render_rgb": True, "render_depth": True, "render_seg": True, "use_textures": True,
    "use_fast_math": False, "use_shadows": True, "use_ambient_lighting": True, "enabled_geom_groups": [0, 1, 2, 3],
    "cam_active": None, "background_color": [0.0, 0.0, 0.0, 1.0], "flex_render_smooth": True, "use_precomputed_rays": True,
    "render_skybox": False, "enable_backface_culling": True, "shadow_light_fraction": 0.3, "samples_per_pixel": 1,
    "enable_vertex_normals": True, "enable_specular": True, "enable_emission": True, "enable_per_light_ambient": True,
    "splat_position": None, "splat_rotation": None, "splat_scale": None, "splat_rgba": None,
    "splat_adr": None, "splat_group_id": None,
}


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def fail(message: str) -> None:
    raise SystemExit(f"R06-S3b flex capture environment/reference mismatch: {message}")


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
    """All existing render-bvh references, excluding the scene-flex fixture itself."""
    return sha_manifest(RENDER_BVH, (SCENE_DIR.resolve(),))


def flex_model_report(mjm, rc) -> dict:
    """Flex model arrays (3.15 side) + resolved render-context flex fields."""
    import numpy as np

    def to_int_list(value):
        array = value.numpy() if hasattr(value, "numpy") else value
        return np.asarray(array).reshape(-1).astype(int).tolist()

    report = {"nflex": int(mjm.nflex), "nflexvert": int(mjm.nflexvert)}

    for name in FLEX_FIELDS:
        values = np.asarray(getattr(mjm, name)).reshape(-1)
        report[name] = [float(v) for v in values] if values.dtype.kind == "f" else [int(v) for v in values]
    report["rgba"] = [[float(v) for v in row] for row in mjm.flex_rgba[: mjm.nflex]]
    report["resolved"] = {
        "bvh_nflexgeom": int(rc.bvh_nflexgeom),
        "flex_dim_np": to_int_list(rc.flex_dim_np),
        "flex_geom_flexid": to_int_list(rc.flex_geom_flexid),
        "flex_geom_edgeid": to_int_list(rc.flex_geom_edgeid),
    }
    return report


def check_environment(*, want_cuda_device: bool) -> dict:
    problems = []
    if pathlib.Path(sys.prefix) != VENV:
        problems.append(f"python prefix {sys.prefix} != {VENV}")
    if pathlib.Path(sys.executable).resolve() != (VENV / "Scripts/python.exe").resolve():
        problems.append(f"python executable {sys.executable} != {VENV}/Scripts/python.exe")
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
    module_path = pathlib.Path(mjw.__file__).resolve()
    if not module_path.is_relative_to(UPSTREAM.resolve()):
        problems.append(f"mujoco_warp imported from {module_path} (not the pinned editable checkout)")
    if problems:
        fail("; ".join(problems))
    import mujoco as mjc

    report = {"versions": versions, "python_executable": sys.executable, "python_prefix": sys.prefix,
              "python_executable_sha256": digest(pathlib.Path(sys.executable)), "platform": platform.platform(),
              "upstream_revision": REVISION, "upstream_clean": True, "upstream_module": module_path.as_posix(),
              "mujoco_package": pathlib.Path(mjc.__file__).resolve().as_posix(),
              "scene_xml": {"path": SCENE_XML.relative_to(ROOT).as_posix(), "sha256": digest(SCENE_XML)}}
    if want_cuda_device:
        device = wp.get_device("cuda:0")
        if int(device.arch) != GPU_ARCH:
            fail(f"gpu arch {device.arch} != {GPU_ARCH}")
        if GPU_NAME not in device.name:
            fail(f"gpu name {device.name} != {GPU_NAME}")
        report["gpu"] = {"name": device.name, "arch": int(device.arch), "cuda_toolkit": list(wp.get_cuda_toolkit_version()),
                         "cuda_driver": list(wp.get_cuda_driver_version()),
                         "driver": run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]).splitlines()[0].strip()}
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
        "seg_semantics": "(object_id, object_type) pairs; background = (-1, -1); flex = (flexid, 9)",
        "rgb_semantics": "unpacked vec3 in [0, 1] via get_rgb",
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
        "flexvert_xpos": np.asarray(d.flexvert_xpos.numpy()).astype(np.float32),
    }


CAM_POSES = "cam-poses.json"
EXISTING_REFERENCE_FILES = ("rgb.npz", "depth.npz", "seg.npz", "state.npz", "capture.json")


def flex_cam_pose_record(d) -> dict:
    """Data poses (`d.cam_xpos`/`d.cam_xmat`) as f32 bit patterns; same forward as the npz outputs."""
    import numpy as np

    pos = np.asarray(d.cam_xpos.numpy(), dtype=np.float32)
    mat = np.asarray(d.cam_xmat.numpy(), dtype=np.float32)
    if pos.ndim != 3 or pos.shape[0] != 1:
        fail(f"camera pose world layout {pos.shape}")
    bits = lambda a: [int(v) for v in np.ascontiguousarray(a).view(np.uint32).reshape(-1)]
    return {"nworld": int(pos.shape[0]), "ncam": int(pos.shape[1]), "cam_xpos_bits": bits(pos[0]),
            "cam_xmat_bits": bits(mat[0]), "cam_xpos": [float(v) for v in pos[0].reshape(-1)],
            "cam_xmat": [float(v) for v in mat[0].reshape(-1)]}


def state_bitwise_equal(live: dict, saved_path: pathlib.Path) -> dict:
    """Six state arrays vs state.npz, f32 bit for f32 bit (same-frame self-proof)."""
    import numpy as np

    def bits(a):
        return np.ascontiguousarray(np.asarray(a, dtype=np.float32)).view(np.uint32)

    with np.load(saved_path) as archive:
        saved = {name: archive[name] for name in archive.files}
    return {name: bool(bits(live[name]).shape == bits(saved[name]).shape
                       and np.array_equal(bits(live[name]), bits(saved[name]))) for name in sorted(live)}


def write_cam_poses(out_dir, mjm, poses: dict, environment: dict, state_equal: dict):
    import mujoco

    path = out_dir / CAM_POSES
    record = {"schema": SCHEMA, "task_id": "2026-11-04-render-r06-s3b-cam-pose-source-fix",
              "kind": "render-flex-camera-pose-provenance", "captured_at": os.environ.get("MJWARP_CAPTURE_AT", ""),
              "command": [sys.executable, "-B"] + sys.argv, "environment": environment,
              "cameras": {"names": [mujoco.mj_id2name(mjm, mujoco.mjtObj.mjOBJ_CAMERA, i) or "" for i in range(int(mjm.ncam))],
                          "cam_mode": [int(v) for v in mjm.cam_mode], "cam_targetbodyid": [int(v) for v in mjm.cam_targetbodyid]},
              "poses": poses, "state_matches_saved_npz": state_equal}
    out_dir.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    return path



def cpu_preflight(scene: pathlib.Path) -> dict:
    """CPU-only preflight for the flex report/state paths that failed on GPU runs (no CUDA)."""
    import mujoco
    import mujoco_warp as mjw
    import numpy as np
    import warp as wp
    class Stub:
        """Minimal RenderContext view: upstream create_render_context derives these from mjm."""
        def __init__(self, mjm):
            flexid, edgeid = [], []
            for flex in range(int(mjm.nflex)):
                if int(mjm.flex_dim[flex]) == 1:
                    start, count = int(mjm.flex_edgeadr[flex]), int(mjm.flex_edgenum[flex])
                    flexid.extend([flex] * count)
                    edgeid.extend(range(start, start + count))
                else:
                    flexid.append(flex)
                    edgeid.append(-1)
            self.bvh_nflexgeom = len(flexid)
            self.flex_dim_np = np.asarray(mjm.flex_dim)
            self.flex_geom_flexid = np.asarray(flexid)
            self.flex_geom_edgeid = np.asarray(edgeid)
    wp.set_device("cpu")
    mjm = mujoco.MjModel.from_xml_path(str(scene))
    m = mjw.put_model(mjm)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    state = state_arrays(d)
    positions = np.asarray(state["flexvert_xpos"])
    if positions.shape[1:] != (int(mjm.nflexvert), 3) or not np.isfinite(positions).all():
        fail(f"cpu preflight flexvert_xpos {positions.shape}")
    report = flex_model_report(mjm, Stub(mjm))
    return {"device": "cpu", "report_ok": True, "state_ok": True, "nflex": report["nflex"],
            "bvh_nflexgeom": report["resolved"]["bvh_nflexgeom"],
            "flex_geom_edgeid": report["resolved"]["flex_geom_edgeid"],
            "flexvert_xpos_shape": list(positions.shape), "flexvert_xpos_sum": float(positions.sum())}


def capture_once(mjm, m, d, rc) -> dict:
    import numpy as np

    images, image_meta, render_seconds = render_images(mjm, m, d, rc)
    arrays: dict[str, object] = {}
    for cam_idx in range(len(images["rgb"])):
        arrays[f"rgb_cam{cam_idx}"] = images["rgb"][cam_idx]
        arrays[f"depth_cam{cam_idx}"] = images["depth"][cam_idx]
        arrays[f"seg_cam{cam_idx}"] = images["seg"][cam_idx]
    for name, values in arrays.items():
        if name.startswith(("rgb_", "depth_")) and not np.isfinite(values).all():
            fail(f"non-finite {name}")
    return {"arrays": arrays, "state": state_arrays(d), "image_meta": image_meta,
            "render_seconds": round(render_seconds, 6)}


def context_receipt(mjm, rc, files_sha256: dict, capture: dict, run_index: int) -> dict:
    background = int(rc.background_color)
    resolved = {"nrender": int(rc.nrender), "total_rays": int(rc.total_rays), "cam_id_map": [int(v) for v in rc.cam_id_map.numpy()],
                "cam_res": [[int(w), int(h)] for w, h in rc.cam_res.numpy()], "use_shadows": bool(rc.use_shadows),
                "use_fast_math": bool(rc.use_fast_math), "shadow_light_fraction": float(rc.shadow_light_fraction),
                "flex_render_smooth": bool(rc.flex_render_smooth), "samples_per_pixel": int(rc.samples_per_pixel),
                "background_color_unpacked": [float((background >> 16) & 0xFF), float((background >> 8) & 0xFF), float(background & 0xFF), float((background >> 24) & 0xFF)],
                "znear": float(rc.znear), "zfar": float(rc.zfar)}
    shapes = lambda prefix: [list(a.shape) for k, a in capture["arrays"].items() if k.startswith(prefix)]
    return {"schema": SCHEMA, "task_id": TASK_ID, "kind": "render-flex-context", "run_index": run_index,
            "parameters": dict(PARAMS), "resolved": resolved,
            "lights": {"type": [int(v) for v in mjm.light_type], "castshadow": [int(v) for v in mjm.light_castshadow],
                       "active": [int(v) for v in mjm.light_active]},
            "flex": flex_model_report(mjm, rc),
            "outputs": {"files_sha256": files_sha256, "rgb_shape": shapes("rgb_"), "depth_shape": shapes("depth_"), "seg_shape": shapes("seg_"),
                        "state_shape": {k: list(v.shape) for k, v in capture["state"].items()}, "image_meta": capture["image_meta"]},
            "render_seconds": capture["render_seconds"]}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="R06-S3b authoring-only 1D flex render reference capture (pinned MJWarp 3.15).",
        epilog="Outputs rgb/depth/seg/state.npz + capture.json under --out-dir; legacy references are sha-snapshotted and must stay untouched.")
    parser.add_argument("--group", default="base", help="output group name (default base)")
    parser.add_argument("--use-fast-math", choices=("true", "false"), default="false")
    parser.add_argument("--use-shadows", choices=("true", "false"), default="true")
    parser.add_argument("--out-dir", type=pathlib.Path, default=None, help="output directory (default: fixtures/render-bvh/scene-flex/reference/<group>)")
    parser.add_argument("--seed", type=int, default=20261104, help="recorded capture seed")
    parser.add_argument("--repeat", type=int, choices=(1, 2), default=2, help="independent capture repeats (2 requires byte-identical raw outputs)")
    parser.add_argument("--scene", type=pathlib.Path, default=SCENE_XML, help="scene XML (default scene-flex)")
    parser.add_argument("--warp-cache", type=pathlib.Path, default=DEFAULT_CACHE, help="warp kernel cache dir")
    parser.add_argument("--dry-run", action="store_true", help="environment + plan check only; no rendering")
    parser.add_argument("--cam-poses-only", action="store_true", help="forward-only data camera-pose provenance (writes cam-poses.json, nothing else)")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    out_dir = (args.out_dir or (SCENE_DIR / "reference" / args.group)).resolve()
    if out_dir.is_relative_to(RENDER_BVH) and not out_dir.is_relative_to(SCENE_DIR.resolve()):
        fail(f"--out-dir {out_dir} is inside legacy reference tree {RENDER_BVH}")
    PARAMS["use_shadows"] = args.use_shadows == "true"
    PARAMS["use_fast_math"] = args.use_fast_math == "true"
    import warp as wp

    wp.config.kernel_cache_dir = str(args.warp_cache)
    if args.cam_poses_only and args.dry_run:
        fail("--cam-poses-only 不能与 --dry-run 同用")
    if args.dry_run:
        environment = check_environment(want_cuda_device=False)
        preflight = cpu_preflight(args.scene.resolve())
        plan = {
            "task_id": TASK_ID, "dry_run": True, "group": args.group,
            "scene": args.scene.resolve().as_posix(), "scene_sha256": digest(args.scene.resolve()),
            "parameters": dict(PARAMS), "out_dir": out_dir.as_posix(),
            "repeat": args.repeat, "seed": args.seed, "warp_cache": str(args.warp_cache),
            "expected_outputs": ["rgb.npz", "depth.npz", "seg.npz", "state.npz", "capture.json"],
            "legacy_manifest_entries": len(legacy_manifest()),
            "environment": environment, "cpu_preflight": preflight,
        }
        print(json.dumps(plan, indent=2, ensure_ascii=False))
        return 0
    preflight = cpu_preflight(args.scene.resolve())
    wp.set_device("cuda:0")
    environment = check_environment(want_cuda_device=True)
    environment["cpu_preflight"] = preflight
    legacy_before = legacy_manifest()
    pose_before = {name: digest(out_dir / name) for name in EXISTING_REFERENCE_FILES if (out_dir / name).is_file()}
    import mujoco
    import mujoco_warp as mjw
    import numpy as np

    print(f"R06-S3b environment OK; legacy reference files={len(legacy_before)}")
    mjm = mujoco.MjModel.from_xml_path(str(args.scene.resolve()))
    m = mjw.put_model(mjm)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    poses = flex_cam_pose_record(d)
    if args.cam_poses_only:
        state_equal = state_bitwise_equal(state_arrays(d), out_dir / "state.npz")
        if not all(state_equal.values()): fail(f"state arrays differ from state.npz: {state_equal}")
        path = write_cam_poses(out_dir, mjm, poses, environment, state_equal)
        after = {name: digest(out_dir / name) for name in EXISTING_REFERENCE_FILES if (out_dir / name).is_file()}
        if pose_before != after: fail(f"existing reference files changed during pose dump: {pose_before} vs {after}")
        print(json.dumps({"task_id": "2026-11-04-render-r06-s3b-cam-pose-source-fix", "mode": "cam-poses-only", "cam_poses": path.as_posix(),
                          "cam_poses_sha256": digest(path), "ncam": poses["ncam"], "state_bitwise_equal": state_equal,
                          "existing_files_sha256": after, "existing_files_unchanged": True}, indent=2, ensure_ascii=False))
        return 0
    rc = mjw.create_render_context(mjm, **PARAMS)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc) if args.repeat == 2 else first
    for name in sorted(first["arrays"]):
        if not np.array_equal(first["arrays"][name], second["arrays"][name]):
            fail(f"two captures differ in array {name}")
    for name in sorted(first["state"]):
        if not np.array_equal(first["state"][name], second["state"][name]):
            fail(f"two captures differ in state {name}")
    first_files: dict[str, dict] = {f"{kind}.npz": {k: v for k, v in first["arrays"].items() if k.startswith(f"{kind}_")}
                                    for kind in ("rgb", "depth", "seg")}
    second_files = {f"{kind}.npz": {k: v for k, v in second["arrays"].items() if k.startswith(f"{kind}_")}
                    for kind in ("rgb", "depth", "seg")}
    first_files["state.npz"], second_files["state.npz"] = first["state"], second["state"]
    out_dir.mkdir(parents=True, exist_ok=True)
    first_hashes = {name: save_npz_deterministic(out_dir / name, data) for name, data in first_files.items()}
    tmp_second = out_dir / ".second"
    second_hashes = {name: save_npz_deterministic(tmp_second / name, data) for name, data in second_files.items()}
    if first_hashes != second_hashes:
        fail(f"two captures differ in npz bytes: {first_hashes} vs {second_hashes}")
    import shutil

    shutil.rmtree(tmp_second, ignore_errors=True)
    flex = flex_model_report(mjm, rc)
    capture = {"schema": SCHEMA, "task_id": TASK_ID, "kind": "render-flex-capture", "group": args.group,
               "parameters": dict(PARAMS), "seed": args.seed, "repeat": args.repeat,
               "two_captures_bitwise_identical": first_hashes == second_hashes, "files_sha256": first_hashes,
               "captured_at": os.environ.get("MJWARP_CAPTURE_AT", ""), "command": [sys.executable, "-B"] + sys.argv,
               "environment": environment, "flex": flex,
               "reference_semantics": {"flex_hit": "geom_id=-2, mesh_id=flexid, seg=(flexid, 9) first sample only",
                                       "depth": "dist * -ray_dir_local_cam[2] (same as geom hits)", "color": "flex_rgba[mesh_id]"},
               "context": context_receipt(mjm, rc, first_hashes, first, 1),
               "context_repeat": context_receipt(mjm, rc, first_hashes, second, 2),
               "legacy_reference_untouched": legacy_manifest() == legacy_before, "legacy_reference_files": len(legacy_before),
               "reads_product_tree": False, "authoring_only": True, "product_python_dependency": False}
    if not capture["legacy_reference_untouched"]:
        fail("legacy references changed during capture")
    (out_dir / "capture.json").write_text(
        json.dumps(capture, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n"
    )
    # D5：同一次 forward 的 data 相机位姿 provenance（f32 位型），供 GPU 用例逐位消费。
    state_equal = state_bitwise_equal(first["state"], out_dir / "state.npz")
    cam_poses_path = write_cam_poses(out_dir, mjm, poses, environment, state_equal)
    summary = {"task_id": TASK_ID, "group": args.group, "out_dir": out_dir.as_posix(),
               "files_sha256": first_hashes, "capture_json_sha256": digest(out_dir / "capture.json"),
               "cam_poses_sha256": digest(cam_poses_path),
               "state_matches_saved_npz": state_equal,
               "two_captures_bitwise_identical": capture["two_captures_bitwise_identical"],
               "legacy_reference_untouched": capture["legacy_reference_untouched"],
               "flex": {"nflex": flex["nflex"], "bvh_nflexgeom": flex["resolved"]["bvh_nflexgeom"]}}
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
