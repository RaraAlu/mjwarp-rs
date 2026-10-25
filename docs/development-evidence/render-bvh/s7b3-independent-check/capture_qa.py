"""QA S7b3 independent reference capture (offline authoring, QA-owned; NOT product code).

Uses the frozen `PARAMS_28` reference recipe (the agreed contract) from the delivered authoring
script as plumbing only; every judgement is QA's own. Captures, with pinned 3.15:

  base                  : same-recipe default flags (spp=1)
  textures_off          : use_textures=False
  vertex_normals_off    : enable_vertex_normals=False
  specular_off          : enable_specular=False
  emission_off          : enable_emission=False
  culling_off           : enable_backface_culling=False (+ depth/seg)
  aa2                   : samples_per_pixel=2 -> nsamples=4 (+ depth/seg)

Writes per-variant RGB packed bits / depth bits / seg pairs / camera poses / light poses to
target/qa-s7b3/ref/ref_<variant>.json, and compares each capture to the delivered fixture
(fixtures/render-bvh/scene-a/s7b/flags/). Two renders per variant must be bitwise identical.

Run (pinned venv, repo root):
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qa-s7b3/capture_qa.py
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import sys
import time

import numpy as np
import warp as wp

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
SCENE = ROOT / "fixtures/render-bvh/scene-a"
S7B = SCENE / "s7b"
FLAGS = S7B / "flags"
OUT = ROOT / "target/qa-s7b3/ref"
CACHE = ROOT / "target/render-bvh/s7b3-flags-cache"
MJB = SCENE / "scene-315.mjb"

wp.config.kernel_cache_dir = str(CACHE)

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402

PARAMS_28 = {
    "nworld": 1,
    "cam_res": None,
    "render_rgb": True,
    "render_depth": True,
    "render_seg": True,
    "use_textures": True,
    "use_fast_math": False,
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
VARIANTS = [
    ("base", {}),
    ("textures_off", {"use_textures": False}),
    ("vertex_normals_off", {"enable_vertex_normals": False}),
    ("specular_off", {"enable_specular": False}),
    ("emission_off", {"enable_emission": False}),
    ("culling_off", {"enable_backface_culling": False}),
    ("aa2", {"samples_per_pixel": 2}),
]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def camera_pose_bits(d) -> list[dict]:
    xpos = np.asarray(d.cam_xpos.numpy())[0].astype(np.float32)
    xmat = np.asarray(d.cam_xmat.numpy())[0].astype(np.float32)
    return [
        {
            "xpos_bits": [int(v) for v in xpos[i].view(np.uint32)],
            "xmat_bits": [int(v) for v in xmat[i].reshape(-1).view(np.uint32)],
        }
        for i in range(xpos.shape[0])
    ]


def light_pose_bits(d) -> list[dict]:
    xpos = np.asarray(d.light_xpos.numpy())[0].astype(np.float32)
    xdir = np.asarray(d.light_xdir.numpy())[0].astype(np.float32)
    return [
        {
            "xpos_bits": [int(v) for v in xpos[i].view(np.uint32)],
            "xdir_bits": [int(v) for v in xdir[i].view(np.uint32)],
        }
        for i in range(xpos.shape[0])
    ]


def capture(mjm, m, d, override: dict, tag: str) -> dict:
    params = dict(PARAMS_28)
    params.update(override)
    rc = mjw.create_render_context(mjm, **params)
    mjw.render(m, d, rc)
    wp.synchronize()
    first_rgb = np.asarray(rc.rgb_data.numpy())[0].astype(np.uint32).copy()
    first_depth = np.asarray(rc.depth_data.numpy())[0].astype(np.float32).copy()
    first_seg = np.asarray(rc.seg_data.numpy())[0].astype(np.int32).copy()
    mjw.render(m, d, rc)
    wp.synchronize()
    repeat_ok = (
        np.array_equal(first_rgb, np.asarray(rc.rgb_data.numpy())[0])
        and np.array_equal(first_depth, np.asarray(rc.depth_data.numpy())[0])
        and np.array_equal(first_seg, np.asarray(rc.seg_data.numpy())[0])
    )
    cam_res = np.asarray(rc.cam_res.numpy()).astype(np.int64)
    rgb_adr = np.asarray(rc.rgb_adr.numpy()).astype(np.int64)
    depth_adr = np.asarray(rc.depth_adr.numpy()).astype(np.int64)
    seg_adr = np.asarray(rc.seg_adr.numpy()).astype(np.int64)
    cameras = []
    for index in range(int(rc.nrender)):
        width, height = int(cam_res[index][0]), int(cam_res[index][1])
        pixels = width * height
        start = int(rgb_adr[index])
        cameras.append(
            {
                "index": index,
                "width": width,
                "height": height,
                "pixels": pixels,
                "rgb_bits": [int(v) for v in first_rgb[start : start + pixels]],
                "depth_bits": [int(v) for v in first_depth[start : start + pixels].view(np.uint32)],
                "seg_pairs": [
                    [int(pair[0]), int(pair[1])]
                    for pair in first_seg[start : start + pixels].reshape(-1, 2)
                ],
            }
        )
    document = {
        "tag": tag,
        "override": override,
        "params": params,
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "environment": {
            "warp": wp.__version__,
            "mujoco": mujoco.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
            "numpy": np.__version__,
            "python": sys.executable,
        },
        "repeat_bitwise_equal": bool(repeat_ok),
        "poses": camera_pose_bits(d),
        "lights": light_pose_bits(d),
        "cameras": cameras,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / f"ref_{tag}.json"
    path.write_text(json.dumps(document, separators=(",", ":")), encoding="utf-8")
    print(
        f"{tag}: wrote {path.name} bytes={path.stat().st_size} sha256={sha256(path.read_bytes())} "
        f"repeat={repeat_ok} nsamples={int(np.asarray(rc.ray_data.numpy()).shape[0]) // 120000 if hasattr(rc, 'ray_data') else '?'}"
    )
    return document


def compare_to_fixture(document: dict, tag: str) -> None:
    rgb_path = FLAGS / f"{tag}_rgb.json"
    if not rgb_path.exists():
        print(f"  {tag}: no delivered RGB fixture")
        return
    delivered = json.loads(rgb_path.read_text(encoding="utf-8"))
    rgb_ok = [cam["rgb_bits"] == deliv["rgb_bits_packed"] for cam, deliv in zip(document["cameras"], delivered["cameras"])]
    print(f"  {tag}: qa_capture_equals_delivered_rgb={rgb_ok}")
    depth_path = FLAGS / f"{tag}_depth_seg.json"
    if depth_path.exists():
        depth_doc = json.loads(depth_path.read_text(encoding="utf-8"))
        depth_ok = []
        seg_ok = []
        for cam, deliv in zip(document["cameras"], depth_doc["cameras"]):
            depth_ok.append(cam["depth_bits"] == deliv["depth_bits"])
            flat = [value for pair in cam["seg_pairs"] for value in pair]
            seg_ok.append(flat == deliv["seg_pairs"])
        print(f"  {tag}: qa_capture_equals_delivered_depth={depth_ok} seg={seg_ok}")


def main() -> int:
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    documents = {}
    for tag, override in VARIANTS:
        documents[tag] = capture(mjm, m, d, override, tag)
    print("--- delivered fixture comparison")
    for tag, _ in VARIANTS:
        compare_to_fixture(documents[tag], tag)
    # vertex_normals on/off: QA capture base (on) vs off must be bitwise equal for scene-a
    base = documents["base"]
    off = documents["vertex_normals_off"]
    print(
        "--- vertex_normals on/off (QA capture) rgb bitwise equal:",
        [a["rgb_bits"] == b["rgb_bits"] for a, b in zip(base["cameras"], off["cameras"])],
    )
    # base capture vs S7b2 base capture (cross-check provenance of the shared base reference)
    s7b2_base = ROOT / "target/qa-s7b2/ref/ref_base.json"
    if s7b2_base.exists():
        prior = json.loads(s7b2_base.read_text(encoding="utf-8"))
        print(
            "--- S7b3 base vs S7b2 base capture rgb bitwise equal:",
            [a["rgb_bits"] == b["rgb_bits"] for a, b in zip(base["cameras"], prior["cameras"])],
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
