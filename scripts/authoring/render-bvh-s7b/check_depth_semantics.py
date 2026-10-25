"""S7b1 read-only prep: pin depth semantics (planar vs Euclidean) and seg object types
against the frozen S0b reference (depth.npz / seg.npz / rays.npz). CPU only, no GPU, no warp.
"""

from __future__ import annotations

import math
import pathlib

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[3]
REF = ROOT / "fixtures/render-bvh/scene-a/reference"
GRID = 8
ZN = np.float32(0.024999999441206455)
CAM_RES = [(320, 240), (240, 180)]
SENSOR = [(0.036, 0.024), (0.0, 0.0)]
INTRINSIC = (0.018, 0.016, 0.0, 0.0)
FOVY_DEG = 40.0


def local_cos(cam: int, px: int, py: int) -> float:
    """-ray_dir_local_cam[2] for the camera-space unit dir at pixel (px, py)."""
    width, height = CAM_RES[cam]
    aspect = np.float32(width / height)
    zn = ZN
    if SENSOR[cam][1] != 0.0:  # intrinsic branch
        fx, fy, cx, cy = (np.float32(v) for v in INTRINSIC)
        sensor_w, sensor_h = np.float32(SENSOR[cam][0]), np.float32(SENSOR[cam][1])
        sensor_aspect = sensor_w / sensor_h
        if aspect > sensor_aspect:
            sensor_h = sensor_w / aspect
        elif aspect < sensor_aspect:
            sensor_w = sensor_h * aspect
        inv_fx = zn / fx
        inv_fy = zn / fy
        half_w = np.float32(0.5) * sensor_w
        half_h = np.float32(0.5) * sensor_h
        left = -inv_fx * (half_w - cx)
        right = inv_fx * (half_w + cx)
        top = inv_fy * (half_h - cy)
        bottom = -inv_fy * (half_h + cy)
    else:  # fovy branch
        fovy_rad = np.float32(FOVY_DEG) * np.float32(math.pi / 180.0)
        half_h = zn * np.tan(np.float32(0.5) * fovy_rad)
        half_w = half_h * aspect
        left, right, top, bottom = -half_w, half_w, half_h, -half_h
    u = np.float32(px + 0.5) / np.float32(width)
    v = np.float32(py + 0.5) / np.float32(height)
    x = (right - left) * u + left
    y = (bottom - top) * v + top
    length = float(np.sqrt(np.float32(x * x) + np.float32(y * y) + np.float32(zn * zn)))
    return float(zn) / length


def main() -> None:
    depth = np.load(REF / "depth.npz")
    seg = np.load(REF / "seg.npz")
    rays = np.load(REF / "rays.npz")
    print("npz keys:", list(depth.keys()), list(seg.keys()), list(rays.keys()))

    for cam in range(2):
        d = np.asarray(depth[f"depth_cam{cam}"])
        s = np.asarray(seg[f"seg_cam{cam}"])
        print(
            f"cam{cam}: depth shape={d.shape} dtype={d.dtype} zeros={int((d == 0).sum())}/"
            f"{d.size} min_pos={float(d[d > 0].min()):.9g} max={float(d.max()):.9g}"
        )
        pairs, counts = np.unique(s.reshape(-1, 2), axis=0, return_counts=True)
        order = np.argsort(-counts)
        print(
            f"cam{cam}: seg pairs (top6) = "
            + ", ".join(f"{tuple(int(v) for v in pairs[i])}x{int(counts[i])}" for i in order[:6])
        )

    import mujoco

    print("mjOBJ values:", int(mujoco.mjtObj.mjOBJ_GEOM), int(mujoco.mjtObj.mjOBJ_FLEX))
    print("mjGEOM values:", [int(getattr(mujoco.mjtGeom, f"mjGEOM_{n}")) for n in ("PLANE", "SPHERE", "BOX", "MESH")])

    origin = np.asarray(rays["ray_origin"], dtype=np.float32)
    direction = np.asarray(rays["ray_direction"], dtype=np.float32)
    dist = np.asarray(rays["ray_dist"], dtype=np.float32)
    geomid = np.asarray(rays["ray_geomid"], dtype=np.int32)
    cat = np.asarray(rays["ray_category"], dtype=np.int8)
    print("camera_pixel ray count:", int((cat == 0).sum()))
    for cam in range(2):
        ids = [i for i in range(128) if cat[i] == 0][cam * 64 : (cam + 1) * 64]
        origin_unique = np.unique(origin[ids], axis=0)
        print(f"cam{cam}: unique origins among its rays = {len(origin_unique)} -> {origin_unique[0].tolist()}")

    # Compare reference depth against Euclidean vs planar prediction for camera-pixel hits.
    max_rel_euclid = 0.0
    max_rel_planar = 0.0
    max_abs_planar = 0.0
    compared = 0
    bit_euclid = 0
    bit_planar = 0
    for cam in range(2):
        dc = np.asarray(depth[f"depth_cam{cam}"])
        base = sum(64 for _ in range(cam))
        for gy in range(GRID):
            py = min(max(int((gy + 0.5) * CAM_RES[cam][1] / GRID), 0), CAM_RES[cam][1] - 1)
            for gx in range(GRID):
                px = min(max(int((gx + 0.5) * CAM_RES[cam][0] / GRID), 0), CAM_RES[cam][0] - 1)
                index = base + gy * GRID + gx
                if geomid[index] < 0:
                    continue
                value = np.float32(dc[py, px])
                if value == 0:
                    continue
                compared += 1
                euclid = np.float32(dist[index])
                cos = np.float32(local_cos(cam, px, py))
                planar = np.float32(euclid * cos)
                max_rel_euclid = max(max_rel_euclid, abs(float(value) - float(euclid)) / float(value))
                max_rel_planar = max(max_rel_planar, abs(float(value) - float(planar)) / float(value))
                max_abs_planar = max(max_abs_planar, abs(float(value) - float(planar)))
                if value.tobytes() == euclid.tobytes():
                    bit_euclid += 1
                if value.tobytes() == planar.tobytes():
                    bit_planar += 1
    print(
        f"camera-pixel hit pixels compared={compared}; max_rel euclidean={max_rel_euclid:.3e} "
        f"planar={max_rel_planar:.3e}; max_abs_planar={max_abs_planar:.3e}; "
        f"bit-equal euclidean={bit_euclid}/{compared} planar={bit_planar}/{compared}"
    )


if __name__ == "__main__":
    main()
