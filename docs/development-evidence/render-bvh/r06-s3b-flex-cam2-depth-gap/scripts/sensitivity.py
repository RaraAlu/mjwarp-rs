"""cam2 depth diagnosis part 2 (CPU only): depth-ULP sensitivity analysis.

For the 654 flex pixels: compare reference depth local gradient (per-pixel depth change in
ULP) with the counts observed in the GPU window (321 mismatched / 297 beyond 4 ULP) and in
the CPU warp replay (597 beyond 4 ULP, all in flex region). Also check rigid regions for
the same sensitivity (they pass on GPU because the product is bit-exact there).
"""
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
BASE = ROOT / "fixtures/render-bvh/scene-flex/reference/base"
OUT = ROOT / "target/s3b-verify/cpu-replay"

depth = np.load(BASE / "depth.npz")
seg = np.load(BASE / "seg.npz")
rgb = np.load(BASE / "rgb.npz")


def bits(a):
    return np.asarray(a, dtype=np.float32).view(np.int32)


def ulp_map(a, b):
    return np.abs(bits(a).astype(np.int64) - bits(b).astype(np.int64))


for cam in (0, 1, 2):
    d = depth[f"depth_cam{cam}.npy"].astype(np.float32)
    s = seg[f"seg_cam{cam}.npy"]
    flex = s[:, :, 1] == 9
    reach = d != 0.0
    h, w = d.shape
    # local max neighbor depth change expressed in ULP of the smaller-magnitude value
    grad = np.zeros((h, w), dtype=np.float64)
    for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        shifted = np.roll(np.roll(d, dy, axis=0), dx, axis=1)
        valid = reach & np.roll(np.roll(reach, dy, axis=0), dx, axis=1)
        diff = np.abs(d.astype(np.float64) - shifted.astype(np.float64))
        ref = np.minimum(np.abs(d.astype(np.float64)), np.abs(shifted.astype(np.float64)))
        # ULP at that depth (f32 exponent-based), avoid log2(0)
        with np.errstate(divide="ignore", invalid="ignore"):
            ulp_unit = 2.0 ** (np.floor(np.log2(np.where(ref > 0, ref, 1.0))) - 23.0)
        ulp_diff = np.where(valid, diff / ulp_unit, 0.0)
        grad = np.maximum(grad, ulp_diff)
    print(f"cam{cam}: pixels={h*w} hit_px={int(reach.sum())} flex_px={int(flex.sum())}")
    print(f"  rigid/hit region: neighbor-ULP>4 pixels = {int(np.count_nonzero(reach & (grad > 4.0)))}"
          f" of {int(reach.sum())}")
    if flex.sum():
        fg = grad[flex]
        print(f"  flex region: neighbor-ULP>4 pixels = {int(np.count_nonzero(fg > 4.0))} of {int(flex.sum())}")
        print(f"  flex depth range = {float(d[flex].min()):.8f} .. {float(d[flex].max()):.8f} m")
        for thr in (1, 2, 4, 8, 16, 64):
            print(f"    flex pixels with neighbor depth jump > {thr:>2} ULP: {int(np.count_nonzero(fg > thr))}")

# CPU replay vs reference on flex pixels
for cam in (2,):
    rep = np.load(OUT / f"cpu_cam{cam}_depth.npz")["depth"]
    ref = depth[f"depth_cam{cam}.npy"].reshape(-1).astype(np.float32)
    s = seg[f"seg_cam{cam}.npy"].reshape(-1, 2)
    flex = s[:, 1] == 9
    u = ulp_map(rep, ref)
    sens = np.zeros(ref.shape[0], dtype=np.float64)
    d = depth[f"depth_cam{cam}.npy"].astype(np.float32)
    h, w = d.shape
    grad = np.zeros((h, w), dtype=np.float64)
    reach = d != 0.0
    for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        shifted = np.roll(np.roll(d, dy, axis=0), dx, axis=1)
        valid = reach & np.roll(np.roll(reach, dy, axis=0), dx, axis=1)
        diff = np.abs(d.astype(np.float64) - shifted.astype(np.float64))
        ref2 = np.minimum(np.abs(d.astype(np.float64)), np.abs(shifted.astype(np.float64)))
        with np.errstate(divide="ignore", invalid="ignore"):
            ulp_unit = 2.0 ** (np.floor(np.log2(np.where(ref2 > 0, ref2, 1.0))) - 23.0)
        grad = np.maximum(grad, np.where(valid, diff / ulp_unit, 0.0))
    sens = grad.reshape(-1)
    beyond = (u > 4) & flex
    print(f"\ncam{cam} CPU-replay vs reference on flex pixels: beyond4={int(beyond.sum())}/{int(flex.sum())}")
    print(f"  beyond4 pixels mean neighbor-gradient(ULP) = {float(sens[beyond].mean()):.1f} "
          f"(median {float(np.median(sens[beyond])):.1f})")
    within = (u != 0) & (u <= 4) & flex
    print(f"  mismatch<=4ULP pixels mean neighbor-gradient = {float(sens[within].mean()):.1f} "
          f"(median {float(np.median(sens[within])):.1f})")
    equal = (u == 0) & flex
    print(f"  bit-equal pixels        mean neighbor-gradient = {float(sens[equal].mean()):.1f} "
          f"(median {float(np.median(sens[equal])):.1f})")
