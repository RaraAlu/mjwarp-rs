"""Independent derivation of the S7b1 sphere arithmetic order (v2).

Exact product/upstream arithmetic chain (read from render.cu / render_math.cu):
  dir_local = compute_ray(...)            # fovy branch, explicit fmaf chain
  dir_world = mat_vec(cam_xmat, dir)      # plain expression (contraction calibrated)
  origin_world = cam_xpos
  sphere: dif = origin_world - sphere_pos (world); a = dot3_ch0(dir,dir),
          b = dot3_ba(dir,dif), dd = dot3_ba(dif,dif), det = fma(b,b,-mul(a,c)),
          sq = sqrt(det), den = rcp(a), x0 = den*(0-b-sq), x1 = den*(sq-b)
  depth = dist * -dir_local.z
Two variants are compared against the frozen reference:
  reference order: a=ch0, b=dd=ba, det fused, rcp
  product default order: all dots comp1-first (ba), det unfused
"""

from __future__ import annotations

import json
import math
import pathlib
import struct

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
BASE = ROOT / "target/qa-render-bvh-s7b1"
RECIPE = ROOT / "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json"
MODEL = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
S2B = ROOT / "fixtures/render-bvh/scene-a/s2/reference_hits.json"
OUT = BASE / "sphere-order-analysis.json"

F32 = np.float32
DEG2RAD = F32(0.017453292519943295)


def b2f(bits: int) -> np.float32:
    return np.frombuffer(struct.pack("<I", int(bits)), dtype=np.float32)[0]


def f2b(value) -> int:
    return int(np.frombuffer(struct.pack("<f", np.float32(value)), dtype=np.uint32)[0])


def fma(a, b, c):
    return F32(np.float64(F32(a)) * np.float64(F32(b)) + np.float64(F32(c)))


def mul(a, b):
    return F32(F32(a) * F32(b))


def add(a, b):
    return F32(F32(a) + F32(b))


def sub(a, b):
    return F32(F32(a) - F32(b))


def div(a, b):
    return F32(F32(a) / F32(b))


def sqrtf(a):
    return F32(np.sqrt(np.float64(F32(a))))


def mat_vec_orders():
    def build(name, first, rest):
        def mv(rot, vec):
            out = []
            for row in range(3):
                c = [F32(rot[row][k]) for k in range(3)]
                products = [mul(c[k], F32(vec[k])) for k in range(3)]
                acc = products[first]
                for k in rest:
                    acc = fma(c[k], F32(vec[k]), acc)
                out.append(acc)
            return out
        return name, mv

    return [
        build("mv_0_1_2", 0, (1, 2)),
        build("mv_0_2_1", 0, (2, 1)),
        build("mv_1_0_2", 1, (0, 2)),
        build("mv_1_2_0", 1, (2, 0)),
        build("mv_2_0_1", 2, (0, 1)),
        build("mv_2_1_0", 2, (1, 0)),
    ]


def normalize3_rays(a):
    length = sqrtf(fma(a[2], a[2], fma(a[1], a[1], mul(a[0], a[0]))))
    if float(length) > 0.0:
        return [div(a[k], length) for k in range(3)]
    return [F32(0.0)] * 3


def compute_ray_fovy(fovy, width, height, px, py, znear):
    inv_img_h = div(F32(1.0), F32(height))
    aspect = mul(F32(width), inv_img_h)
    fovy_rad = mul(F32(fovy), DEG2RAD)
    half_height = mul(F32(znear), F32(math.tan(float(mul(F32(0.5), fovy_rad)))))
    half_width = mul(half_height, aspect)
    left, right = F32(-float(half_width)), half_width
    top, bottom = half_height, F32(-float(half_height))
    u = div(F32(px + 0.5), F32(width))
    v = mul(F32(py + 0.5), inv_img_h)
    x = fma(sub(right, left), u, left)
    y = fma(sub(bottom, top), v, top)
    return normalize3_rays([x, y, F32(-znear)])


def dot_ch0(u, v):
    return fma(u[2], v[2], fma(u[1], v[1], mul(u[0], v[0])))


def dot_ba(u, v):
    return fma(u[2], v[2], fma(u[0], v[0], mul(u[1], v[1])))


def sphere_depth(origin_world, dir_world, dir_local, sphere_pos, radius, reference_order):
    dif = [sub(origin_world[k], sphere_pos[k]) for k in range(3)]
    if reference_order:
        a = dot_ch0(dir_world, dir_world)
        b = dot_ba(dir_world, dif)
        dd = dot_ba(dif, dif)
    else:
        a = dot_ba(dir_world, dir_world)
        b = dot_ba(dir_world, dif)
        dd = dot_ba(dif, dif)
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
    return mul(sol, F32(-float(dir_local[2])))


def main() -> int:
    recipe = json.loads(RECIPE.read_text(encoding="utf-8"))
    model = json.loads(MODEL.read_text(encoding="utf-8"))["arrays"]
    s2b = json.loads(S2B.read_text(encoding="utf-8"))
    harness = json.loads((BASE / "harness-report.json").read_text(encoding="utf-8"))
    cam_index = 1
    camera = recipe["cameras"][cam_index]
    width, height = camera["width"], camera["height"]
    depth_bits = [int(v) for v in camera["depth_bits"]]
    seg_flat = [int(v) for v in camera["seg_pairs"]]
    seg_pairs = [seg_flat[i:i + 2] for i in range(0, len(seg_flat), 2)]

    enabled = [int(v) for v in s2b["enabled_geom_ids"]]
    xpos_bits = [int(v) for v in s2b["cases"]["initial"]["geom_xpos_bits"]]
    xmat_bits = [int(v) for v in s2b["cases"]["initial"]["geom_xmat_bits"]]
    geom_type = [int(v) for v in model["geom_type"]["values"]]
    geom_size = [float(v) for v in model["geom_size"]["values"]]
    pose = {}
    for slot, geom in enumerate(enabled):
        pose[geom] = (
            [b2f(xpos_bits[3 * slot + k]) for k in range(3)],
            [[b2f(xmat_bits[9 * slot + 3 * r + c]) for c in range(3)] for r in range(3)],
        )

    fovy = float(model["cam_fovy"]["values"][cam_index])
    znear = F32(0.024999999441206455)
    zfar = F32(125.0)
    pose_bits = harness["poses"]["reference"][cam_index]
    cam_pos = [b2f(v) for v in pose_bits["xpos_bits"]]
    cam_mat = [[b2f(pose_bits["xmat_bits"][3 * r + c]) for c in range(3)] for r in range(3)]

    plane_pixels = [i for i, pair in enumerate(seg_pairs) if pair == [0, 5]]
    sphere_pixels = [i for i, pair in enumerate(seg_pairs) if pair == [2, 5]]

    def plane_depth(px, py, mv, origin, mat):
        local = compute_ray_fovy(fovy, width, height, px, py, znear)
        world = mv(mat, local)
        if float(world[2]) >= -1.0e-15:
            return F32(0.0)
        t = div(F32(-float(origin[2])), world[2])
        if float(t) < 0.0:
            return F32(0.0)
        depth = mul(t, F32(-float(local[2])))
        if float(depth) > float(zfar):
            return F32(0.0)
        return depth

    calibration = {}
    best_name, best_mv, best_equal = None, None, -1
    for name, mv in mat_vec_orders():
        equal = 0
        for pixel in plane_pixels:
            value = plane_depth(pixel % width, pixel // width, mv, cam_pos, cam_mat)
            if f2b(value) == depth_bits[pixel]:
                equal += 1
        calibration[name] = {"equal": equal, "total": len(plane_pixels)}
        if equal > best_equal:
            best_name, best_mv, best_equal = name, mv, equal

    # sphere world-space test with the calibrated camera transform
    sphere_pos = pose[2][0]
    sphere_radius = np.float32(geom_size[3 * enabled[enabled.index(2)] + 0])

    def evaluate(use_reference_order):
        equal = 0
        histogram = [0, 0, 0, 0, 0, 0]
        for pixel in sphere_pixels:
            local = compute_ray_fovy(fovy, width, height, pixel % width, pixel // width, znear)
            world = best_mv(cam_mat, local)
            value = sphere_depth(cam_pos, world, local, sphere_pos, sphere_radius, use_reference_order)
            expected = depth_bits[pixel]
            if value is None:
                continue
            delta = abs(f2b(value) - expected)
            histogram[min(delta, 5)] += 1
            if delta == 0:
                equal += 1
        return {
            "bit_equal": equal,
            "total": len(sphere_pixels),
            "histogram": histogram,
            "beyond_4": histogram[5],
        }

    reference_order = evaluate(True)
    product_order = evaluate(False)

    # new-pose generalization (shifted camera, product kernel dump)
    shifted_bits = harness["poses"]["shifted"][cam_index]
    shifted_pos = [b2f(v) for v in shifted_bits["xpos_bits"]]
    shifted_mat = [[b2f(shifted_bits["xmat_bits"][3 * r + c]) for c in range(3)] for r in range(3)]
    shifted_depth = list(struct.unpack("<%dI" % (len((BASE / "depth_shifted_pose.bin").read_bytes()) // 4),
                                       (BASE / "depth_shifted_pose.bin").read_bytes()))
    seg_raw = (BASE / "seg_shifted_pose.bin").read_bytes()
    seg_values = list(struct.unpack("<%di" % (len(seg_raw) // 4), seg_raw))
    shifted_seg = [seg_values[i:i + 2] for i in range(0, len(seg_values), 2)]
    new_pose = {"sphere_pixels": 0, "reference_order_bit_equal": 0, "product_order_gt4": 0, "reference_hist": [0, 0, 0, 0, 0, 0]}
    for pixel, pair in enumerate(shifted_seg):
        if pair != [2, 5]:
            continue
        new_pose["sphere_pixels"] += 1
        local = compute_ray_fovy(fovy, width, height, pixel % width, pixel // width, znear)
        world = best_mv(shifted_mat, local)
        expected = shifted_depth[pixel]
        reference = sphere_depth(shifted_pos, world, local, sphere_pos, sphere_radius, True)
        alternate = sphere_depth(shifted_pos, world, local, sphere_pos, sphere_radius, False)
        if reference is not None:
            delta = abs(f2b(reference) - expected)
            new_pose["reference_hist"][min(delta, 5)] += 1
            if delta == 0:
                new_pose["reference_order_bit_equal"] += 1
        if alternate is not None and abs(f2b(alternate) - expected) > 4:
            new_pose["product_order_gt4"] += 1

    report = {
        "camera": cam_index,
        "pixel_sets": {"plane_pixels": len(plane_pixels), "sphere_pixels": len(sphere_pixels)},
        "camera_transform_calibration": calibration,
        "chosen_camera_transform": best_name,
        "plane_bit_equal": best_equal,
        "sphere_reference_order": reference_order,
        "sphere_product_default_order": product_order,
        "new_pose_generalization": new_pose,
        "method": "f32 emulation: fma = exact f64 product+add then single f32 rounding; sqrt/div/rcp correctly rounded; camera ray and sphere chain copied from the product/upstream formulas; reference order vs product default order compared.",
    }
    OUT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
