"""R06-S3b cam2 depth diagnosis (CPU-only, no GPU): replay the pinned upstream render on
the warp CPU backend and compare depth/seg/rgb against the GPU-captured reference.

Purpose: decide whether the "cam2 depth <= 4 ULP vs reference" criterion is achievable by
any non-bit-exact implementation, or whether reference/criterion premise is the issue.
Writes only under target/ (repo untouched). No CUDA device is opened.
"""
import json
import math
import pathlib
import struct

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
FIX = ROOT / "fixtures" / "render-bvh" / "scene-flex"
BASE = FIX / "reference" / "base"
OUT = ROOT / "target" / "s3b-verify" / "cpu-replay"
OUT.mkdir(parents=True, exist_ok=True)

cap = json.loads((BASE / "capture.json").read_text(encoding="utf-8"))
PARAMS = dict(cap["parameters"])
state = np.load(BASE / "state.npz")

import mujoco
import mujoco_warp as mjw
import warp as wp

wp.set_device("cpu")
wp.config.quiet = True
print("warp device:", wp.get_device(), flush=True)

mjm = mujoco.MjModel.from_xml_path(str(FIX / "scene.xml"))
mjd = mujoco.MjData(mjm)
mjd.qpos[:] = state["qpos"][0].astype(np.float64)
mujoco.mj_forward(mjm, mjd)
m = mjw.put_model(mjm)
d = mjw.put_data(mjm, mjd, nworld=1)
mjw.forward(m, d)
wp.synchronize()
print("forward done", flush=True)

# 1) CPU forward state vs GPU-captured state.npz (bit level)
for name in ["qpos", "geom_xpos", "geom_xmat", "light_xpos", "light_xdir", "flexvert_xpos"]:
    cpu = np.asarray(getattr(d, name).numpy(), dtype=np.float32)
    ref = np.asarray(state[name], dtype=np.float32)
    bits_equal = np.array_equal(cpu.view(np.uint32), ref.view(np.uint32))
    max_ulp = 0
    cb = cpu.reshape(-1).view(np.uint32).astype(np.int64)
    rb = ref.reshape(-1).view(np.uint32).astype(np.int64)
    diff = np.abs(cb - rb)
    max_ulp = int(diff.max()) if diff.size else 0
    print(f"state {name}: cpu_vs_gpu_bits_equal={bits_equal} max_ulp={max_ulp}", flush=True)

# 2) create render context + render on CPU
rc = mjw.create_render_context(mjm, **PARAMS)
print("render context created", flush=True)
mjwf = mjw
mjwf.render(m, d, rc)
wp.synchronize()
print("cpu render done", flush=True)

cam_res = np.asarray(rc.cam_res.numpy())
cam_ids = np.asarray(rc.cam_id_map.numpy())
depth_adr = np.asarray(rc.depth_adr.numpy())
seg_adr = np.asarray(rc.seg_adr.numpy())
rgb_adr = np.asarray(rc.rgb_adr.numpy())
depth_raw = np.asarray(rc.depth_data.numpy())[0].astype(np.float32)
seg_raw = np.asarray(rc.seg_data.numpy())[0].astype(np.int32)
rgb_raw = np.asarray(rc.rgb_data.numpy())[0].astype(np.uint32)


def f32_bits(arr):
    return np.asarray(arr, dtype=np.float32).view(np.uint32).astype(np.int64)


def ulp_of(a, b):
    return np.abs(f32_bits(a) - f32_bits(b))


ref_depth = np.load(BASE / "depth.npz")
ref_seg = np.load(BASE / "seg.npz")
ref_rgb = np.load(BASE / "rgb.npz")

print("\n== per-camera CPU-replay vs GPU reference ==", flush=True)
for idx in range(int(rc.nrender)):
    w, h = int(cam_res[idx][0]), int(cam_res[idx][1])
    muj = int(cam_ids[idx])
    pixels = w * h
    da, sa = int(depth_adr[idx]), int(seg_adr[idx])
    cpu_depth = depth_raw[da:da + pixels]
    cpu_seg = seg_raw[sa:sa + pixels].reshape(-1, 2)
    ref_d = np.load(BASE / "depth.npz")[f"depth_cam{idx}.npy"].reshape(-1).astype(np.float32)
    ref_s = np.load(BASE / "seg.npz")[f"seg_cam{idx}.npy"].reshape(-1, 2).astype(np.int32)
    ulp = ulp_of(cpu_depth, ref_d)
    mism = int(np.count_nonzero(ulp != 0))
    beyond = int(np.count_nonzero(ulp > 4))
    seg_mism = int(np.count_nonzero(np.any(cpu_seg != ref_s, axis=1)))
    flex_mask = ref_s[:, 1] == 9
    flex_mism = int(np.count_nonzero((ulp != 0) & flex_mask))
    print(f"cam{idx} {muj} {w}x{h}: depth mismatched={mism} beyond4ulp={beyond} "
          f"max_ulp={int(ulp.max())} seg_mismatch={seg_mism} flex_px={int(flex_mask.sum())} "
          f"flex_mismatch_px={flex_mism}", flush=True)
    np.savez_compressed(OUT / f"cpu_cam{idx}_depth.npz", depth=cpu_depth, seg=cpu_seg)
    if beyond:
        loc = np.argwhere((ulp > 4).reshape(h, w))
        sample = loc[:15].tolist()
        print(f"  cam{idx} beyond4 locations(sample)={sample}", flush=True)
        in_flex = int(np.count_nonzero(flex_mask.reshape(h, w)[loc[:, 0], loc[:, 1]]))
        print(f"  cam{idx} beyond4 in-flex={in_flex}/{beyond}", flush=True)

# RGB comparison (packed like the Rust judge: bytes)
print("\n== rgb (packed bytes) CPU-replay vs GPU reference ==", flush=True)
for idx in range(int(rc.nrender)):
    w, h = int(cam_res[idx][0]), int(cam_res[idx][1])
    pixels = w * h
    ra = int(rgb_adr[idx])
    cpu_rgb = rgb_raw[ra:ra + pixels]
    ref_vals = ref_rgb[f"rgb_cam{idx}.npy"].reshape(-1, 3).astype(np.float32)
    # reconstruct reference packed bytes the same way the Rust test does
    def packed(c):
        v = np.clip(c, 0.0, 1.0)
        return np.round(v * 255.0).astype(np.uint32)
    rb3 = packed(ref_vals)
    ref_packed = (np.uint32(0xFF) << 24) | (rb3[:, 0] << 16) | (rb3[:, 1] << 8) | rb3[:, 2]
    diff_bytes = np.stack([
        ((cpu_rgb >> 16) & 0xFF) - ((ref_packed >> 16) & 0xFF),
        ((cpu_rgb >> 8) & 0xFF) - ((ref_packed >> 8) & 0xFF),
        (cpu_rgb & 0xFF) - (ref_packed & 0xFF),
    ], axis=1).astype(np.int64)
    max_byte = int(np.abs(diff_bytes).max())
    beyond1 = int(np.count_nonzero(np.abs(diff_bytes) > 1))
    print(f"cam{idx}: max_byte_diff={max_byte} beyond_1_lsb={beyond1}", flush=True)
