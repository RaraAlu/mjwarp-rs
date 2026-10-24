"""Offline S0b ray-set construction and validation (authoring helper).

OFFLINE ONLY: imported by capture_reference.py only; never by product code.
The ray set covers four categories required by plan S0b:
camera pixels (primary rays from the frozen render context), seeded random rays,
shallow-incidence ("grazing") rays that hit the floor plane near-tangentially,
and no-hit rays pointing away from the scene. Panels are placed so that the
grazing rays travel just above the floor plane at a height far below every other
geom, which makes the grazing hit unambiguous (plane hits are exact).
Ray casting uses the pinned MJWarp `rays` API with the frozen BVH (RenderContext).
"""

from __future__ import annotations

import numpy as np
import warp as wp

from mujoco_warp._src import types as mjw_types

CATEGORY_NAMES = ["camera_pixels", "seeded_random", "grazing", "no_hit"]
CAMERA_PIXEL_GRID = 8
RANDOM_RAYS = 128
GRAZING_RAYS = 64
NO_HIT_RAYS = 64
SEED = 20261024


def normalize(vector: np.ndarray) -> np.ndarray:
    length = float(np.linalg.norm(vector))
    if length == 0.0:
        raise RuntimeError("zero-length ray direction")
    return vector / length


def build_ray_set(mjm, d, rc, seed: int = SEED):
    cam_res = np.asarray(rc.cam_res.numpy())
    cam_ids = np.asarray(rc.cam_id_map.numpy())
    cam_xpos = np.asarray(d.cam_xpos.numpy())[0].astype(np.float64)
    cam_xmat = np.asarray(d.cam_xmat.numpy())[0].astype(np.float64)
    cam_rays = np.asarray(rc.ray.numpy()).astype(np.float64)
    offsets, total = [], 0
    for width, height in cam_res:
        offsets.append(total)
        total += int(width) * int(height)
    origins, directions, categories = [], [], []
    camera_pixel_counts = []

    for cam_idx, (width, height) in enumerate(cam_res):
        width, height = int(width), int(height)
        cam_id = int(cam_ids[cam_idx])
        pixel_count = 0
        for gy in range(CAMERA_PIXEL_GRID):
            py = min(max(int((gy + 0.5) * height / CAMERA_PIXEL_GRID), 0), height - 1)
            for gx in range(CAMERA_PIXEL_GRID):
                px = min(max(int((gx + 0.5) * width / CAMERA_PIXEL_GRID), 0), width - 1)
                local = cam_rays[offsets[cam_idx] + px + py * width]
                origins.append(cam_xpos[cam_id])
                directions.append(normalize(cam_xmat[cam_id] @ local))
                categories.append(0)
                pixel_count += 1
        camera_pixel_counts.append(pixel_count)

    rng = np.random.default_rng(seed)
    center = np.asarray(mjm.stat.center, dtype=np.float64)
    extent = float(mjm.stat.extent)
    geom_xpos = np.asarray(d.geom_xpos.numpy())[0].astype(np.float64)
    geom_xmat = np.asarray(d.geom_xmat.numpy())[0].astype(np.float64)
    lower = geom_xpos.min(axis=0) - 0.5 * extent
    upper = geom_xpos.max(axis=0) + 0.5 * extent
    for _ in range(RANDOM_RAYS):
        origins.append(rng.uniform(lower, upper))
        directions.append(normalize(rng.normal(size=3)))
        categories.append(1)

    import mujoco

    floor_id = mujoco.mj_name2id(mjm, mujoco.mjtObj.mjOBJ_GEOM, "floor")
    if floor_id < 0:
        raise RuntimeError("scene has no floor geom for grazing rays")
    floor = geom_xpos[floor_id]
    axes = geom_xmat[floor_id]
    normal = axes[:, 2] / np.linalg.norm(axes[:, 2])
    axis_u = axes[:, 0] / np.linalg.norm(axes[:, 0])
    axis_v = np.cross(normal, axis_u)
    run_length = 1.2
    height = 0.006
    for index in range(GRAZING_RAYS):
        angle = 2.0 * np.pi * index / GRAZING_RAYS
        radial = np.cos(angle) * axis_u + np.sin(angle) * axis_v
        target = floor + (0.45 + 0.02 * index) * radial
        origin = target + run_length * radial + height * normal
        origins.append(origin)
        directions.append(normalize(target - origin))
        categories.append(2)

    top = center + np.array([0.0, 0.0, 1.5 * extent])
    for index in range(NO_HIT_RAYS):
        angle = 2.0 * np.pi * index / NO_HIT_RAYS
        origin = top + np.array([np.cos(angle), np.sin(angle), 0.0]) * 0.1 * extent
        direction = np.array([np.cos(angle), np.sin(angle), 2.0])
        origins.append(origin)
        directions.append(normalize(direction))
        categories.append(3)

    pnt = np.asarray(origins, dtype=np.float32).reshape(1, -1, 3)
    vec = np.asarray(directions, dtype=np.float32).reshape(1, -1, 3)
    category = np.asarray(categories, dtype=np.int8)
    return pnt, vec, category, floor_id, camera_pixel_counts


def cast_rays(m, d, rc, pnt_np, vec_np):
    nray = int(pnt_np.shape[1])
    pnt = wp.array(pnt_np, dtype=wp.vec3)
    vec = wp.array(vec_np, dtype=wp.vec3)
    bodyexclude = wp.array(np.full(nray, -1, dtype=np.int32), dtype=int)
    dist = wp.empty((1, nray), dtype=float)
    geomid = wp.empty((1, nray), dtype=int)
    normal = wp.empty((1, nray), dtype=wp.vec3)
    import mujoco_warp as mjw

    mjw.rays(
        m,
        d,
        pnt,
        vec,
        mjw_types.vec6(-1, -1, -1, -1, -1, -1),
        True,
        bodyexclude,
        dist,
        geomid,
        normal,
        rc,
    )
    wp.synchronize()
    return (
        np.asarray(dist.numpy())[0].astype(np.float32),
        np.asarray(geomid.numpy())[0].astype(np.int32),
        np.asarray(normal.numpy())[0].astype(np.float32),
    )


def ray_stats(category, dist, geomid, normal, vec, grazing_geom_id):
    stats = {}
    for code, name in enumerate(CATEGORY_NAMES):
        mask = category == code
        hits = int(np.count_nonzero(geomid[mask] >= 0))
        dots = np.abs(np.einsum("ij,ij->i", normal[mask], vec[0][mask]))
        stats[name] = {
            "rays": int(mask.sum()),
            "hits": hits,
            "no_hits": int(mask.sum()) - hits,
            "max_abs_normal_dot_direction": float(dots.max()) if dots.size else None,
        }
    camera = stats["camera_pixels"]
    if camera["hits"] < camera["rays"] // 2:
        raise RuntimeError(f"camera-pixel rays mostly miss: {camera}")
    grazing = stats["grazing"]
    grazing_mask = category == 2
    wrong_geom = int(np.count_nonzero(geomid[grazing_mask] != grazing_geom_id))
    if grazing["hits"] != grazing["rays"] or wrong_geom or grazing["max_abs_normal_dot_direction"] > 0.05:
        raise RuntimeError(
            f"grazing rays invalid: {grazing} (geom_id={grazing_geom_id}, wrong_geom_hits={wrong_geom})"
        )
    no_hit = stats["no_hit"]
    if no_hit["hits"] != 0:
        raise RuntimeError(f"no-hit rays intersect geometry: {no_hit}")
    return stats
