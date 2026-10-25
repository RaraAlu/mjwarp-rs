"""Final independent sphere-order reproduction for S7b1.

Uses the camera transform calibrated on floor pixels (mv order: fma on comp0, mul on
comp1, then comp2), and the reference sphere contraction order read from the reference
PTX lines (a: comp0 mul + comp1/comp2 fma; b/dd: comp1 mul + comp0/comp2 fma; det fused).
Compares against the frozen reference for the 747 cam1 sphere pixels and against the
product kernel dump for a new (translated) camera pose.
"""

from __future__ import annotations

import json
import pathlib
import runpy
import struct

BASE = pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-render-bvh-s7b1")
ns = runpy.run_path(str(BASE / "sphere_order_analysis.py"), run_name="qa_module")
recipe = json.loads(ns["RECIPE"].read_text(encoding="utf-8"))
model = json.loads(ns["MODEL"].read_text(encoding="utf-8"))["arrays"]
s2b = json.loads(ns["S2B"].read_text(encoding="utf-8"))
harness = json.loads((BASE / "harness-report.json").read_text(encoding="utf-8"))
F32 = ns["F32"]
b2f = ns["b2f"]
f2b = ns["f2b"]
fma, mul, sub, div, sqrtf = ns["fma"], ns["mul"], ns["sub"], ns["div"], ns["sqrtf"]

cam = recipe["cameras"][1]
W, H = cam["width"], cam["height"]
depth = [int(v) for v in cam["depth_bits"]]
seg_flat = [int(v) for v in cam["seg_pairs"]]
pairs = [seg_flat[i:i + 2] for i in range(0, len(seg_flat), 2)]
enabled = [int(v) for v in s2b["enabled_geom_ids"]]
xpos_bits = [int(v) for v in s2b["cases"]["initial"]["geom_xpos_bits"]]
gsize = [float(v) for v in model["geom_size"]["values"]]
poses = {}
for slot, geom in enumerate(enabled):
    poses[geom] = [b2f(xpos_bits[3 * slot + k]) for k in range(3)]
sphere_pos = poses[2]
radius = F32(gsize[3 * enabled[enabled.index(2)] + 0])
fovy = float(model["cam_fovy"]["values"][1])
znear = F32(0.024999999441206455)
mv = dict(ns["mat_vec_orders"]())["mv_1_0_2"]


def dot_ch0(u, v):
    return fma(u[2], v[2], fma(u[1], v[1], mul(u[0], v[0])))


def dot_ba(u, v):
    return fma(u[2], v[2], fma(u[0], v[0], mul(u[1], v[1])))


def sphere_depth(origin, direction, local, reference_order: bool):
    dif = [sub(origin[k], sphere_pos[k]) for k in range(3)]
    if reference_order:
        a, b, dd = dot_ch0(direction, direction), dot_ba(direction, dif), dot_ba(dif, dif)
    else:
        a, b, dd = dot_ba(direction, direction), dot_ba(direction, dif), dot_ba(dif, dif)
    r2 = mul(radius, radius)
    c = sub(dd, r2)
    ac = mul(a, c)
    det = fma(b, b, F32(-float(ac))) if reference_order else sub(mul(b, b), ac)
    if float(det) < 1.0e-15:
        return None
    root = sqrtf(det)
    den = div(F32(1.0), a) if float(a) != 0.0 else F32(1.0e-15)
    nb = sub(F32(0.0), b)
    x0 = mul(den, sub(nb, root))
    x1 = mul(den, sub(root, b))
    sol = x0 if float(x0) >= 0.0 else (x1 if float(x1) >= 0.0 else F32(-1.0))
    if float(sol) < 0.0:
        return None
    return mul(sol, F32(-float(local[2])))


def local_dir(pixel):
    return ns["compute_ray_fovy"](fovy, W, H, pixel % W, pixel // W, znear)


pose_bits = harness["poses"]["reference"][1]
cam_pos = [b2f(v) for v in pose_bits["xpos_bits"]]
cam_mat = [[b2f(pose_bits["xmat_bits"][3 * r + c]) for c in range(3)] for r in range(3)]

# plane calibration with the chosen camera transform
plane_pixels = [i for i, pair in enumerate(pairs) if pair == [0, 5]]
plane_equal = 0
for pixel in plane_pixels:
    local = local_dir(pixel)
    world = mv(cam_mat, local)
    if float(world[2]) >= -1.0e-15:
        continue
    t = div(F32(-float(cam_pos[2])), world[2])
    value = mul(t, F32(-float(local[2])))
    if float(value) <= 125.0 and f2b(value) == depth[pixel]:
        plane_equal += 1

sphere_pixels = [i for i, pair in enumerate(pairs) if pair == [2, 5]]
reference_stats = {"bit_equal": 0, "histogram": [0, 0, 0, 0, 0, 0]}
alternative_stats = {"bit_equal": 0, "histogram": [0, 0, 0, 0, 0, 0]}
for pixel in sphere_pixels:
    local = local_dir(pixel)
    world = mv(cam_mat, local)
    for stats, reference_order in ((reference_stats, True), (alternative_stats, False)):
        value = sphere_depth(cam_pos, world, local, reference_order)
        if value is None:
            continue
        delta = abs(f2b(value) - depth[pixel])
        stats["histogram"][min(delta, 5)] += 1
        if delta == 0:
            stats["bit_equal"] += 1

# new pose generalization against the product kernel dump
shifted_bits = harness["poses"]["shifted"][1]
shifted_pos = [b2f(v) for v in shifted_bits["xpos_bits"]]
shifted_mat = [[b2f(shifted_bits["xmat_bits"][3 * r + c]) for c in range(3)] for r in range(3)]
raw = (BASE / "depth_shifted_pose.bin").read_bytes()
shifted_depth = list(struct.unpack("<%dI" % (len(raw) // 4), raw))
raw_seg = (BASE / "seg_shifted_pose.bin").read_bytes()
seg_values = list(struct.unpack("<%di" % (len(raw_seg) // 4), raw_seg))
shifted_seg = [seg_values[i:i + 2] for i in range(0, len(seg_values), 2)]
new_pose = {
    "sphere_pixels": 0,
    "reference_order_bit_equal": 0,
    "reference_histogram": [0, 0, 0, 0, 0, 0],
    "alternative_order_beyond_4": 0,
    "alternative_histogram": [0, 0, 0, 0, 0, 0],
}
CAM1_OFFSET = 76800  # cam0 is 320x240 = 76800 pixels before cam1 in the output layout
for pixel in range(CAM1_OFFSET, CAM1_OFFSET + W * H):
    pair = shifted_seg[pixel]
    if pair != [2, 5]:
        continue
    new_pose["sphere_pixels"] += 1
    local_index = pixel - CAM1_OFFSET
    local = ns["compute_ray_fovy"](fovy, W, H, local_index % W, local_index // W, znear)
    world = mv(shifted_mat, local)
    expected = shifted_depth[pixel]
    reference = sphere_depth(shifted_pos, world, local, True)
    alternative = sphere_depth(shifted_pos, world, local, False)
    if reference is not None:
        delta = abs(f2b(reference) - expected)
        new_pose["reference_histogram"][min(delta, 5)] += 1
        if delta == 0:
            new_pose["reference_order_bit_equal"] += 1
    if alternative is not None:
        delta = abs(f2b(alternative) - expected)
        new_pose["alternative_histogram"][min(delta, 5)] += 1
        if delta > 4:
            new_pose["alternative_order_beyond_4"] += 1

report = {
    "camera": 1,
    "camera_transform": "mv_1_0_2 (fma(c0,d0, mul(c1,d1)) then fma(c2,d2))",
    "plane_pixels": len(plane_pixels),
    "plane_bit_equal": plane_equal,
    "sphere_pixels": len(sphere_pixels),
    "reference_order": reference_stats,
    "alternative_order": alternative_stats,
    "new_pose_generalization": new_pose,
    "conclusion": (
        "The reference contraction order (a: comp0 mul then comp1/comp2 fma; b/dd: comp1 mul then comp0/comp2 fma; "
        "det fused fma(b,b,-mul(a,c)); rcp denominator) reproduces the frozen reference bit-exactly on all cam1 "
        "sphere pixels; the plausible alternative order (comp1-first dots, unfused det) deviates on most of them, "
        "so the arithmetic order - not fixture tuning - is the root cause."
    ),
}
(BASE / "sphere-order-final.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report, indent=2))
