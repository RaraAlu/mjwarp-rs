"""Offline authoring: capture the S3 mesh-BVH ray-query reference (device f32 bits).

OFFLINE ONLY: not run by product builds/tests/gates. The S3 product test reads the
committed JSON; it never starts Python.

Reference semantics: the frozen upstream render path calls
`mujoco_warp._src.ray.ray_mesh_with_bvh` (a `@wp.func`), which maps the world ray
into the local mesh frame and calls Warp's `wp.mesh_query_ray` (Woop triangle
intersection, nearest hit, u/v barycentrics, face index, 8-step back-face culling).
This script captures that exact function on the same mesh BVH the render context
builds (`build_mesh_bvh(..., constructor="cubql")`) with device-side f32 geom poses
(the S2b discipline: CPU poses can differ by ~1 ULP).

Cross-checks against frozen references (their files are not rewritten):
- initial/moved pose bits equal the S2b device-pose capture; mesh hit/miss on the 384
  S0b rays equals S2b `geomid == mesh geom`, with distance/normal bits equal for hits;
- 3.12 `mesh_vert` f64 -> f32 equals the captured 3.15 mesh points f32 bits; and
- `s3_mesh_replay.replay` proves the f64 fallback branch never changes a decision on
  the fixture (the product kernel is f32-only), so the translation is parity-valid.

Output: fixtures/render-bvh/scene-a/s3/mesh_query.json
"""

from __future__ import annotations

import gc
import hashlib
import json
import pathlib
import platform
import subprocess
import sys
import time

import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
OUT = SCENE / "s3/mesh_query.json"
MJB, XML, MODEL_312 = SCENE / "scene-315.mjb", SCENE / "scene.xml", SCENE / "model-312.json"
RAYS, S2_HITS = SCENE / "reference/rays.npz", SCENE / "s2/reference_hits.json"
VENV, UPSTREAM = ROOT / "target/mjwarp-parallel/venv", ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/s3-cache"
TASK_ID, SCHEMA, GEOM_ID = "2026-10-24-render-bvh-s3", 1, 3
WORLD1_GEOM_ID = GEOM_ID + 100
HASHES = {
    MJB: "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4",
    XML: "a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289",
    MODEL_312: "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
    RAYS: "83146ae50946537401be817a44bfe3af58320df475dabc49206aa47eec9d50ab",
    S2_HITS: "609ffc527e8216654cb7b6576f998659026d0e5324429c31c7a67f058c225f84",
}
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME, GPU_ARCH, GPU_DRIVER = "NVIDIA GeForce RTX 4070 Ti SUPER", 89, "596.36"
MOVE_TRANSLATION, MOVE_ANGLE_Z = [0.05, -0.04, 0.03], 0.18
# Same 28 create_render_context parameters as S0b/S2/S2b so the mesh BVH matches.
PARAMS_28 = dict(
    nworld=1, cam_res=None, render_rgb=True, render_depth=True, render_seg=True, use_textures=True,
    use_fast_math=True, use_shadows=False, use_ambient_lighting=True, enabled_geom_groups=[0, 1, 2, 3],
    cam_active=None, background_color=[0.0, 0.0, 0.0, 1.0], flex_render_smooth=True,
    use_precomputed_rays=True, render_skybox=False, enable_backface_culling=True, shadow_light_fraction=0.3,
    samples_per_pixel=1, enable_vertex_normals=True, enable_specular=True, enable_emission=True,
    enable_per_light_ambient=True, splat_position=None, splat_rotation=None, splat_scale=None,
    splat_rgba=None, splat_adr=None, splat_group_id=None,
)
DEFORM_DELTAS = np.array(
    [[0.011, -0.007, 0.005], [0.004, 0.009, -0.006], [-0.008, 0.003, 0.007], [0.006, -0.005, -0.004]],
    dtype=np.float32,
)
wp.config.kernel_cache_dir = str(CACHE)
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402
from s3_mesh_replay import replay  # noqa: E402
from s3_mesh_capture_support import (  # noqa: E402
    DEGENERATE_INDICES, DEGENERATE_VERTICES, cast, checked_degenerate_coverage, degenerate_rays,
    design_mesh_rays, world_to_local,
)


def fail(message: str) -> None:
    raise SystemExit(f"S3 mesh reference capture mismatch: {message}")


def digest(path: pathlib.Path) -> str:
    data = path.read_bytes()
    return hashlib.sha256(data.replace(b"\r\n", b"\n") if path.suffix == ".xml" else data).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def bits(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def ints(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.int32).reshape(-1).tolist()


def f32s(values) -> np.ndarray:
    return np.frombuffer(np.asarray(values, dtype=np.uint32).tobytes(), dtype=np.float32).copy()


def check_environment() -> dict:
    problems = []
    if pathlib.Path(sys.prefix) != VENV:
        problems.append(f"python prefix {sys.prefix} != {VENV}")
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
    if run(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"]) != REVISION:
        problems.append("upstream revision mismatch")
    if run(["git", "-C", str(UPSTREAM), "status", "--porcelain"]):
        problems.append("upstream checkout is dirty")
    for path, expected in HASHES.items():
        if digest(path) != expected:
            problems.append(f"{path.name} hash mismatch")
    device = wp.get_device("cuda:0")
    if int(device.arch) != GPU_ARCH or GPU_NAME not in device.name:
        problems.append(f"gpu {device.name} arch {device.arch} mismatch")
    if run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]) != GPU_DRIVER:
        problems.append("gpu driver mismatch")
    if problems:
        fail("; ".join(problems))
    return {
        "versions": versions, "python_executable": sys.executable, "python_prefix": sys.prefix,
        "platform": platform.platform(), "gpu": device.name, "gpu_arch": int(device.arch),
        "gpu_driver": GPU_DRIVER, "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()), "upstream_revision": REVISION,
        "upstream_clean": True, "mesh_bvh_constructor": "cubql (render_util.create_render_context -> bvh.build_mesh_bvh)",
        "device_reference_code": {
            name: hashlib.sha256((UPSTREAM / "mujoco_warp" / "_src" / name).read_bytes()).hexdigest()
            for name in ("bvh.py", "ray.py", "render.py", "render_util.py", "types.py")
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def quat_multiply_z(q: np.ndarray, angle: float) -> np.ndarray:
    half = 0.5 * angle
    rz = np.array([np.cos(half), 0.0, 0.0, np.sin(half)])
    w1, x1, y1, z1 = q
    w2, x2, y2, z2 = rz
    out = np.array([
        w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2, w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
        w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2, w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
    ])
    return out / np.linalg.norm(out)


def moved_qpos(mjm: mujoco.MjModel) -> np.ndarray:
    qpos = np.array(mujoco.MjData(mjm).qpos, dtype=np.float64)
    qpos[0:3] += np.asarray(MOVE_TRANSLATION, dtype=np.float64)
    qpos[3:7] = quat_multiply_z(qpos[3:7], MOVE_ANGLE_Z)
    return qpos


def make_state(mjm: mujoco.MjModel, moved: bool):
    mjd = mujoco.MjData(mjm)
    if moved:
        mjd.qpos[:] = moved_qpos(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    return m, d


def mesh_handle(rc, mesh_id: int):
    return rc.mesh_registry[int(rc.mesh_bvh_id.numpy()[mesh_id])]


CASE_SPECS = [
    # name, pose, deform, cull, geom_id, max_t mode
    ("initial", "initial", False, True, GEOM_ID, "default"),
    ("no_cull", "initial", False, False, GEOM_ID, "default"),
    ("tight_above", "initial", False, True, GEOM_ID, "above"),
    ("tight_below", "initial", False, True, GEOM_ID, "below"),
    ("dynamic", "initial", True, True, GEOM_ID, "default"),
    ("moved", "moved", False, True, WORLD1_GEOM_ID, "default"),
    ("dynamic_moved", "moved", True, True, WORLD1_GEOM_ID, "default"),
]


def capture_group(mjm, mesh_id: int, pose: str, deform_points: bool, rays) -> dict:
    """Capture one pose/deform group.

    The dynamic `set_points` + `refit` case uses a standalone `sah` mesh (the default
    constructor of upstream's dynamic refit path `refit_flex_bvh`); the frozen render
    context builds static mesh BVHs with `cubql`, which is not the refit reference path.
    """
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    m, d = make_state(mjm, pose == "moved")
    if pose == "moved":
        mjw.refit_bvh(m, d, rc)
        wp.synchronize()
    if deform_points:
        base = mesh_handle(rc, mesh_id)
        points = np.asarray(base.points.numpy(), dtype=np.float32)
        indices = np.asarray(base.indices.numpy(), dtype=np.int32)
        mesh = wp.Mesh(
            points=wp.array(points, dtype=wp.vec3),
            indices=wp.array(indices, dtype=wp.int32),
            bvh_constructor="sah",
            bvh_leaf_size=2,
        )
        mesh_bvh_id = wp.array([mesh.id], dtype=wp.uint64)
        mesh_dataid = 0
    else:
        mesh = mesh_handle(rc, mesh_id)
        mesh_bvh_id = rc.mesh_bvh_id
        mesh_dataid = mesh_id
    vertices_initial = np.asarray(mesh.points.numpy(), dtype=np.float32)
    vertices_used = vertices_initial
    if deform_points:
        vertices_used = np.asarray(vertices_initial + DEFORM_DELTAS, dtype=np.float32)
        mesh.points = wp.array(vertices_used, dtype=wp.vec3)
        mesh.refit()
        wp.synchronize()
    xpos = np.asarray(d.geom_xpos.numpy(), dtype=np.float32)[0, GEOM_ID]
    xmat = np.asarray(d.geom_xmat.numpy(), dtype=np.float32)[0, GEOM_ID]
    pnt, direction = rays
    default_max = np.full(pnt.shape[0], np.float32(mujoco.mjMAXVAL), dtype=np.float32)
    output = {
        "vertices_initial_bits": bits(vertices_initial),
        "vertices_used_bits": bits(vertices_used),
        "geom_xpos_bits": bits(xpos),
        "geom_xmat_bits": bits(xmat),
        "cases": {},
    }
    distance = np.zeros(pnt.shape[0], dtype=np.float32)
    hit = np.zeros(pnt.shape[0], dtype=bool)
    for name, group_pose, group_deform, cull, geom_id, max_mode in CASE_SPECS:
        if group_pose != pose or group_deform != deform_points:
            continue
        max_t = default_max
        if max_mode in ("above", "below"):
            factor = np.float32(1.01) if max_mode == "above" else np.float32(0.99)
            max_t = default_max.copy()
            max_t[hit] = np.asarray(distance[hit] * factor, dtype=np.float32)
            max_t = np.maximum(max_t, np.float32(0.0))
        result = cast(mesh_bvh_id, xpos, xmat, mesh_dataid, geom_id, pnt, direction, max_t, cull)
        result.update({
            "cull_backfaces": int(cull), "geom_id": geom_id, "max_t_bits": bits(max_t),
            "geom_xpos_bits": bits(xpos), "geom_xmat_bits": bits(xmat),
        })
        output["cases"][name] = result
        if name == "initial":
            distance, hit = f32s(result["distance_bits"]), np.asarray(result["hit"], dtype=bool)
    del rc, m, d, mesh
    gc.collect()
    return output


def capture_pass(mjm, mesh_id: int, pnt, direction, degenerate) -> dict:
    groups = {}
    for pose, deform in (("initial", False), ("initial", True), ("moved", False), ("moved", True)):
        groups[f"{pose}_{'deformed' if deform else 'rest'}"] = capture_group(
            mjm, mesh_id, pose, deform, (pnt, direction)
        )
    deg_origins, deg_directions = degenerate
    deg_max = np.full(deg_origins.shape[0], np.float32(mujoco.mjMAXVAL), dtype=np.float32)
    mesh = wp.Mesh(
        points=wp.array(DEGENERATE_VERTICES, dtype=wp.vec3),
        indices=wp.array(DEGENERATE_INDICES, dtype=wp.int32),
        bvh_constructor="sah", bvh_leaf_size=2,
    )
    deg_case = cast(wp.array([mesh.id], dtype=wp.uint64), np.zeros(3, np.float32),
                    np.eye(3, dtype=np.float32).reshape(-1), 0, 0,
                    deg_origins, deg_directions, deg_max, True)
    deg_case.update({
        "vertices_bits": bits(DEGENERATE_VERTICES), "indices": ints(DEGENERATE_INDICES),
        "origin_bits": bits(deg_origins), "direction_bits": bits(deg_directions),
        "max_t_bits": bits(deg_max), "cull_backfaces": 1, "geom_id": 0,
        "geom_xpos_bits": bits(np.zeros(3, np.float32)),
        "geom_xmat_bits": bits(np.eye(3, dtype=np.float32).reshape(-1)),
    })
    return {"groups": groups, "degenerate": deg_case}


def case_bits_equal(first: dict, second: dict) -> list[str]:
    problems = []
    for group, payload in first["groups"].items():
        for name, case in payload["cases"].items():
            for key in ("hit", "distance_bits", "normal_bits", "u_bits", "v_bits", "face"):
                if case[key] != second["groups"][group]["cases"][name][key]:
                    problems.append(f"{group}/{name}/{key}")
        for key in ("vertices_initial_bits", "vertices_used_bits", "geom_xpos_bits", "geom_xmat_bits"):
            if payload[key] != second["groups"][group][key]:
                problems.append(f"{group}/{key}")
    for key in ("hit", "distance_bits", "normal_bits", "u_bits", "v_bits", "face"):
        if first["degenerate"][key] != second["degenerate"][key]:
            problems.append(f"degenerate/{key}")
    return problems


def find_case(pass_data: dict, name: str) -> dict:
    for group in pass_data["groups"].values():
        if name in group["cases"]:
            return group["cases"][name]
    fail(f"case {name} not captured")
    raise AssertionError


def s2b_cross_check(first: dict) -> dict:
    s2 = json.loads(S2_HITS.read_text(encoding="utf-8"))
    slot = [int(value) for value in s2["enabled_geom_ids"]].index(GEOM_ID)
    initial, moved = find_case(first, "initial"), find_case(first, "moved")
    s2_initial, s2_moved = s2["cases"]["initial"], s2["cases"]["moved"]
    xpos = f32s(s2_initial["geom_xpos_bits"]).reshape(-1, 3)[slot]
    xmat = f32s(s2_initial["geom_xmat_bits"]).reshape(-1, 9)[slot]
    moved_xpos = f32s(s2_moved["geom_xpos_bits"]).reshape(-1, 3)[slot]
    moved_xmat = f32s(s2_moved["geom_xmat_bits"]).reshape(-1, 9)[slot]
    for case, name, pose, mat in ((initial, "initial", xpos, xmat), (moved, "moved", moved_xpos, moved_xmat)):
        if list(case["geom_xpos_bits"]) != bits(pose) or list(case["geom_xmat_bits"]) != bits(mat):
            fail(f"{name} pose bits differ from the S2b device-pose capture")
    mesh_initial = [i for i, geom in enumerate(ints(s2_initial["geomid"])) if geom == GEOM_ID]
    mesh_moved = [i for i, geom in enumerate(ints(s2_moved["geomid"])) if geom == GEOM_ID]
    own_initial = [i for i, hit in enumerate(initial["hit"][:384]) if hit]
    own_moved = [i for i, hit in enumerate(moved["hit"][:384]) if hit]
    if own_initial != mesh_initial or own_moved != mesh_moved:
        fail(f"S2b hit/miss cross-check failed: {own_initial} vs {mesh_initial}, {own_moved} vs {mesh_moved}")
    s2_initial_dist = f32s(s2_initial["dist_bits"])
    s2_moved_dist = f32s(s2_moved["dist_bits"])
    s2_initial_normal = f32s(s2_initial["normal_bits"]).reshape(-1, 3)
    s2_moved_normal = f32s(s2_moved["normal_bits"]).reshape(-1, 3)
    dist_ok = all(initial["distance_bits"][i] == int(np.float32(s2_initial_dist[i]).view(np.uint32)) for i in mesh_initial) \
        and all(moved["distance_bits"][i] == int(np.float32(s2_moved_dist[i]).view(np.uint32)) for i in mesh_moved)
    normal_ok = all(initial["normal_bits"][3 * i:3 * i + 3] == bits(s2_initial_normal[i]) for i in mesh_initial) \
        and all(moved["normal_bits"][3 * i:3 * i + 3] == bits(s2_moved_normal[i]) for i in mesh_moved)
    if not (dist_ok and normal_ok):
        fail("S2b distance/normal bit cross-check failed")
    return {
        "initial_pose_bits_equal_s2b": True, "moved_pose_bits_equal_s2b": True,
        "mesh_hit_rays_s2b_initial": mesh_initial, "mesh_hit_rays_s2b_moved": mesh_moved,
        "product_hit_rays_initial": own_initial, "product_hit_rays_moved": own_moved,
        "hit_miss_match_initial": True, "hit_miss_match_moved": True,
        "mesh_hit_distance_bits_equal_s2b": True, "mesh_hit_normal_bits_equal_s2b": True,
    }


def changed_rays(dynamic: dict, initial: dict) -> list[int]:
    changed = []
    for index in range(len(initial["hit"])):
        if dynamic["hit"][index] != initial["hit"][index] or dynamic["face"][index] != initial["face"][index]:
            changed.append(index)
            continue
        delta = abs(int(np.int64(dynamic["distance_bits"][index]) - np.int64(initial["distance_bits"][index])))
        if delta > 4:
            changed.append(index)
    return changed


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S3 environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    rays_npz = np.load(RAYS)
    s0b_pnt = np.asarray(rays_npz["ray_origin"], dtype=np.float32)
    s0b_dir = np.asarray(rays_npz["ray_direction"], dtype=np.float32)
    if s0b_pnt.shape != (384, 3):
        fail(f"unexpected S0b ray set shape {s0b_pnt.shape}")

    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    m, d = make_state(mjm, False)
    rc0 = mjw.create_render_context(mjm, **PARAMS_28)
    mesh_id = int(mjm.geom_dataid[GEOM_ID])
    if int(mjm.geom_type[GEOM_ID]) != 7:
        fail(f"geom {GEOM_ID} is not a mesh")
    mesh0 = mesh_handle(rc0, mesh_id)
    vertices0 = np.asarray(mesh0.points.numpy(), dtype=np.float32)
    indices0 = np.asarray(mesh0.indices.numpy(), dtype=np.int32)
    xpos0 = np.asarray(d.geom_xpos.numpy(), dtype=np.float32)[0, GEOM_ID]
    xmat0 = np.asarray(d.geom_xmat.numpy(), dtype=np.float32)[0, GEOM_ID]
    design_pnt, design_dir, tags = design_mesh_rays(vertices0, indices0, xpos0, xmat0)
    pnt = np.concatenate([s0b_pnt, design_pnt], axis=0)
    direction = np.concatenate([s0b_dir, design_dir], axis=0)
    degenerate = degenerate_rays()
    model_312 = json.loads(MODEL_312.read_text(encoding="utf-8"))["arrays"]["mesh_vert"]["values"]
    model_312_vertices = np.asarray(np.asarray(model_312, dtype=np.float64), dtype=np.float32)
    if bits(model_312_vertices) != bits(vertices0):
        fail("3.12 model mesh_vert f64 -> f32 does not match the captured 3.15 mesh points")
    del rc0, mesh0, m, d
    gc.collect()

    first = capture_pass(mjm, mesh_id, pnt, direction, degenerate)
    started = time.perf_counter()
    second = capture_pass(mjm, mesh_id, pnt, direction, degenerate)
    elapsed = time.perf_counter() - started
    problems = case_bits_equal(first, second)
    if problems:
        fail(f"two captures differ: {problems}")
    coverage = checked_degenerate_coverage(first["degenerate"])

    cross = s2b_cross_check(first)
    initial_case, dynamic_case = find_case(first, "initial"), find_case(first, "dynamic")
    changed = changed_rays(dynamic_case, initial_case)
    default_max = np.full(pnt.shape[0], np.float32(mujoco.mjMAXVAL), dtype=np.float32)
    replay_info = {}
    for group, payload in first["groups"].items():
        lpnt, lvec = world_to_local(
            pnt, direction,
            f32s(payload["geom_xpos_bits"]).reshape(-1),
            f32s(payload["geom_xmat_bits"]).reshape(-1),
        )
        replay_info[group] = replay(
            f32s(payload["vertices_used_bits"]).reshape(-1, 3), indices0, lpnt, lvec, default_max, True
        )
    lpnt, lvec = degenerate
    replay_info["degenerate"] = replay(
        DEGENERATE_VERTICES, DEGENERATE_INDICES, lpnt, lvec,
        np.full(lpnt.shape[0], np.float32(mujoco.mjMAXVAL)), True,
    )
    scene_branches = sum(
        item["degenerate_woop_queries"] for name, item in replay_info.items() if name != "degenerate"
    )
    degenerate_branches = replay_info["degenerate"]["degenerate_woop_queries"]
    decision_changes = sum(item["fallback_decision_changes"] for item in replay_info.values())
    # scene-a must never reach upstream's f64 branch (f32-only product kernel); the synthetic
    # degenerate mesh reaches it only where both paths reject, so no decision changes.
    if scene_branches != 0 or decision_changes != 0:
        fail(f"fixture rays reach the upstream f64 fallback branch: {replay_info}")

    document = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s3-mesh-query-reference",
        "mesh": {
            "mesh_id": mesh_id, "geom_id": GEOM_ID, "world1_geom_id": WORLD1_GEOM_ID,
            "vertex_count": int(vertices0.shape[0]), "triangle_count": int(indices0.shape[0] // 3),
            "vertex_bits_initial": bits(vertices0),
            "vertex_bits_dynamic": first["groups"]["initial_deformed"]["vertices_used_bits"],
            "indices": ints(indices0), "model_312_vertex_bits": bits(model_312_vertices),
            "model_312_vertex_bits_equal": True,
        },
        "rays": {
            "count": int(pnt.shape[0]), "s0b_count": 384,
            "origin_bits": bits(pnt), "direction_bits": bits(direction),
            "tags": ["s0b"] * 384 + tags,
        },
        "cases": {
            name: dict(case, vertices=("dynamic" if "deformed" in group else "initial"))
            for group, payload in first["groups"].items()
            for name, case in payload["cases"].items()
        },
        "degenerate": first["degenerate"], "degenerate_coverage": coverage,
        "stats": {
            "hits": {name: int(sum(case["hit"])) for group in first["groups"].values()
                     for name, case in group["cases"].items()},
            "set_points_changed_rays": changed,
            "replay": replay_info,
            "degenerate_woop_queries_scene": scene_branches,
            "degenerate_woop_queries_degenerate_mesh": degenerate_branches,
            "fallback_decision_changes": decision_changes,
        },
        "cross_check_s2b": cross,
        "sources": {
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": HASHES[XML]},
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": HASHES[MJB]},
            "model_312": {"path": "fixtures/render-bvh/scene-a/model-312.json", "sha256": HASHES[MODEL_312]},
            "rays_npz": {"path": "fixtures/render-bvh/scene-a/reference/rays.npz", "sha256": HASHES[RAYS]},
            "s2b_reference_hits": {"path": "fixtures/render-bvh/scene-a/s2/reference_hits.json",
                                   "sha256": HASHES[S2_HITS]},
        },
        "environment": environment,
        "determinism": {"repeat": 2, "bitwise_identical": True, "second_pass_seconds": round(elapsed, 6)},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": memory_used_mib()},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh/capture_s3_mesh.py"},
        "authoring_only": True, "reads_product_tree": False, "product_python_dependency": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({
        "output": str(OUT.relative_to(ROOT)), "sha256": digest(OUT),
        "hits": document["stats"]["hits"], "changed_rays": changed, "cross_check": cross,
        "gpu_memory_mib": document["gpu_memory_mib"],
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
