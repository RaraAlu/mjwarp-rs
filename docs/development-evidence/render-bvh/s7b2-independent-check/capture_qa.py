"""QA S7b2 independent reference capture (offline authoring helper, NOT product code).

Reuses the delivered authoring capture script's frozen PARAMS_28 + pinned environment as
*plumbing only* (the reference recipe is an agreed contract), and adds three QA experiments:

  tag=base  : default pose + default materials  -> must reproduce the delivered
              `rgb_fast_math_false.json` / `depth_seg_fast_math_false.json` packed bits
              (independent provenance check of the fixtures).
  tag=pose  : both cameras moved (mjm.cam_pos shifted); reference re-rendered; writes the
              exact f32 cam_xpos/cam_xmat bits so the product harness can be driven at the
              same new pose.
  tag=shin  : `mat_shininess[1]` replaced (headlight specular exponent = mat*128 != 64);
              reference re-rendered at the default pose. Proves the product reads the
              material exponent (semantic fix) rather than the fixed 64 constant.

Also dumps `d.light_xpos`/`d.light_xdir` f32 bits from `mjwarp.forward` (pinned 3.15) so QA
can check light poses three ways: pinned forward / QA f32 recompute / product light_data_poses.

Output per tag (JSON): poses, rgb packed bits, depth bits, seg pairs, light bits.
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
OUT = ROOT / "target/qa-s7b2/ref"
CACHE = ROOT / "target/render-bvh/s7b2-fastmath-cache"
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


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def camera_pose_bits(d) -> list[dict]:
    xpos = np.asarray(d.cam_xpos.numpy())[0].astype(np.float32)
    xmat = np.asarray(d.cam_xmat.numpy())[0].astype(np.float32)
    return [
        {
            "xpos_bits": [int(value) for value in xpos[index].view(np.uint32)],
            "xmat_bits": [int(value) for value in xmat[index].reshape(-1).view(np.uint32)],
        }
        for index in range(xpos.shape[0])
    ]


def light_pose_bits(d) -> list[dict]:
    xpos = np.asarray(d.light_xpos.numpy())[0].astype(np.float32)
    xdir = np.asarray(d.light_xdir.numpy())[0].astype(np.float32)
    return [
        {
            "xpos_bits": [int(value) for value in xpos[index].view(np.uint32)],
            "xdir_bits": [int(value) for value in xdir[index].view(np.uint32)],
        }
        for index in range(xpos.shape[0])
    ]


def capture(tag: str, cam_shift=None, shininess=None) -> dict:
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    if cam_shift is not None:
        mjm.cam_pos[:] = mjm.cam_pos[:] + np.asarray(cam_shift, dtype=mjm.cam_pos.dtype)
    if shininess is not None:
        for index, value in shininess.items():
            mjm.mat_shininess[index] = value
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    rc = mjw.create_render_context(mjm, **PARAMS_28)
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
                "rgb_bits": [int(value) for value in first_rgb[start : start + pixels]],
                "depth_bits": [
                    int(value) for value in first_depth[start : start + pixels].view(np.uint32)
                ],
                "seg_pairs": [
                    [int(pair[0]), int(pair[1])]
                    for pair in first_seg[start : start + pixels].reshape(-1, 2)
                ],
            }
        )
    document = {
        "tag": tag,
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "environment": {
            "warp": wp.__version__,
            "mujoco": mujoco.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
            "numpy": np.__version__,
            "python": sys.executable,
            "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
        },
        "cam_shift": cam_shift,
        "shininess": None
        if shininess is None
        else {str(key): value for key, value in shininess.items()},
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
        f"repeat={repeat_ok}"
    )
    return document


def compare_to_delivered(document: dict) -> None:
    delivered = json.loads((S7B / "rgb_fast_math_false.json").read_text(encoding="utf-8"))
    depth_doc = json.loads((S7B / "depth_seg_fast_math_false.json").read_text(encoding="utf-8"))
    for camera, expected_rgb, expected_depth in zip(
        document["cameras"], delivered["cameras"], depth_doc["cameras"]
    ):
        rgb_equal = camera["rgb_bits"] == expected_rgb["rgb_bits_packed"]
        depth_equal = camera["depth_bits"] == expected_depth["depth_bits"]
        seg_flat = [value for pair in camera["seg_pairs"] for value in pair]
        seg_equal = seg_flat == expected_depth["seg_pairs"]
        print(
            f"  cam{camera['index']}: rgb_bits_equal={rgb_equal} depth_bits_equal={depth_equal} "
            f"seg_equal={seg_equal}"
        )


def main() -> int:
    base = capture("base")
    compare_to_delivered(base)
    capture("pose", cam_shift=(0.05, -0.03, 0.02))
    capture("shin", shininess={1: 0.05})
    capture("shin2", shininess={1: 0.3})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
