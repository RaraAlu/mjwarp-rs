"""Offline authoring: capture the S5 camera-ray reference (camera space, f32 bits).

OFFLINE ONLY: not run by product builds/tests/gates. The S5 product test reads the
committed JSON; it never starts Python.

Reference semantics: frozen upstream `render_util.create_render_context` (71da24d)
builds the precomputed ray buffers with `_build_rays` -> `compute_ray`, layout
`[sample][camera][row][column]` (x fastest), spp^2 sub-pixel samples. This capture
reads back `rc.ray` / `rc.ray_offset` f32 bits and the resolved camera metadata for a
matrix of camera-selection/resolution/output/samples_per_pixel/znear/projection
variants, plus the rejection surface. It also captures orthographic references by
copying the frozen 3.15 MjModel and overriding `cam_projection` (approved by Arch;
provenance records the override), and a dynamic-ray context
(`use_precomputed_rays=False`, spp=1) that must match the precomputed bits.

Full-grid bit equality is enforced by SHA-256 over the canonical byte order
(dir3 then offset3 per ray, LE f32); per-camera 128 evenly spaced pixels covering all
sub-samples are stored as raw f32 bits. AA image-level acceptance is deferred to
S7b/S8; this slice verifies the ray layer only.

Output: fixtures/render-bvh/scene-a/s5/camera_rays.json
"""

from __future__ import annotations

import gc
import json
import pathlib
import sys
import time

import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
OUT = ROOT / "fixtures/render-bvh/scene-a/s5/camera_rays.json"
TASK_ID = "2026-10-24-render-bvh-s5"
SCHEMA = 1

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402
from s5_capture_support import (  # noqa: E402
    CACHE, MJB, MJB_SHA256, XML_SHA256, bits_f32, bits_f32_scalar, bits_f64, capture_context,
    check_environment,
    fail, memory_used_mib,
)

wp.config.kernel_cache_dir = str(CACHE)

BASE_ARGS = dict(use_textures=False, use_shadows=False, enable_backface_culling=False)


def build_model(spec: dict):
    """从冻结 3.15 MJB 建模型并应用 authoring 覆盖（仅投影/znear，用于覆盖缺口）。"""
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    if spec.get("ortho_camera") is not None:
        camera = int(spec["ortho_camera"])
        mjm.cam_projection[camera] = int(mujoco.mjtProjection.mjPROJ_ORTHOGRAPHIC)
    if spec.get("znear_scale") is not None:
        mjm.vis.map.znear = float(np.float32(mjm.vis.map.znear * float(spec["znear_scale"])))
    return mjm


def model_document(mjm, spec: dict) -> dict:
    names = []
    for camera in range(mjm.ncam):
        name = mujoco.mj_id2name(mjm, mujoco.mjtObj.mjOBJ_CAMERA, camera)
        names.append(None if name is None else name)
    cameras = []
    for camera in range(mjm.ncam):
        cameras.append(
            {
                "projection": int(mjm.cam_projection[camera]),
                "fovy_bits": bits_f64(float(mjm.cam_fovy[camera])),
                "sensorsize_bits": bits_f32(mjm.cam_sensorsize[camera]),
                "intrinsic_bits": bits_f32(mjm.cam_intrinsic[camera]),
                "resolution": [int(mjm.cam_resolution[camera][0]), int(mjm.cam_resolution[camera][1])],
                "output": int(mjm.cam_output[camera]),
            }
        )
    return {
        "cameras": cameras,
        "names": names,
        "stat_extent_bits": bits_f64(float(mjm.stat.extent)),
        "vis_znear_bits": bits_f32_scalar(np.float32(mjm.vis.map.znear)),
        "vis_zfar_bits": bits_f32_scalar(np.float32(mjm.vis.map.zfar)),
        "overrides": spec.get("overrides", []),
    }


MODEL_SPECS = {
    "default": {},
    "ortho": {
        "ortho_camera": 1,
        "overrides": [
            {
                "field": "cam_projection[1]",
                "value": int(mujoco.mjtProjection.mjPROJ_ORTHOGRAPHIC),
                "reason": "scene-a 无正交相机；Arch 批准在 3.15 MjModel 副本上覆盖以覆盖正交分支",
            }
        ],
    },
    "znear_x2": {
        "znear_scale": 2.0,
        "overrides": [
            {"field": "vis.map.znear", "value": "x2 (f32 0.02)", "reason": "验证 znear 参与 f32 方向计算"}
        ],
    },
}


def context_request(spec: dict) -> dict:
    cam_res = spec.get("cam_res")
    if isinstance(cam_res, tuple):
        cam_res = [list(cam_res)]
    elif isinstance(cam_res, list):
        cam_res = [list(item) for item in cam_res]
    return {
        "active": spec["active"],
        "cam_res": cam_res,
        "render_rgb": spec.get("render_rgb"),
        "render_depth": spec.get("render_depth"),
        "render_seg": spec.get("render_seg"),
        "samples_per_pixel": spec.get("samples_per_pixel", 1),
        "use_precomputed_rays": spec.get("use_precomputed_rays", True),
        "mode": "dynamic" if spec.get("use_precomputed_rays", True) is False else "precomputed",
    }


def context_kwargs(spec: dict) -> dict:
    kwargs = dict(BASE_ARGS)
    kwargs.update(
        samples_per_pixel=spec.get("samples_per_pixel", 1),
        use_precomputed_rays=spec.get("use_precomputed_rays", True),
    )
    active = spec["active"]
    if active["kind"] == "all":
        pass
    elif active["kind"] == "empty":
        kwargs["cam_active"] = []
    elif active["kind"] == "mask":
        kwargs["cam_active"] = [bool(value) for value in active["values"]]
    elif active["kind"] == "names":
        kwargs["cam_active"] = [str(value) for value in active["values"]]
    elif active["kind"] == "indices":
        kwargs["cam_active"] = [int(value) for value in active["values"]]
    else:
        fail(f"unknown active kind {active['kind']}")
    for key in ("cam_res", "render_rgb", "render_depth", "render_seg"):
        if spec.get(key) is not None:
            kwargs[key] = spec[key]
    return kwargs


CONTEXTS = {
    "default_all": {"model": "default", "active": {"kind": "all"}},
    "mask_first": {"model": "default", "active": {"kind": "mask", "values": [True, False]}},
    "mask_second": {"model": "default", "active": {"kind": "mask", "values": [False, True]}},
    "names_track": {"model": "default", "active": {"kind": "names", "values": ["cam-track"]}},
    "indices_reversed": {"model": "default", "active": {"kind": "indices", "values": [1, 0]}},
    "empty_active": {"model": "default", "active": {"kind": "empty"}},
    "empty_mask": {"model": "default", "active": {"kind": "mask", "values": [False, False]}},
    "res_tuple": {"model": "default", "active": {"kind": "all"}, "cam_res": (100, 50)},
    "res_hetero": {"model": "default", "active": {"kind": "all"}, "cam_res": [[100, 50], [7, 9]]},
    "res_single_list": {"model": "default", "active": {"kind": "all"}, "cam_res": [[160, 120]]},
    "render_rgb_first": {
        "model": "default",
        "active": {"kind": "all"},
        "render_rgb": [True, False],
        "render_depth": [False, True],
    },
    "spp2_default": {"model": "default", "active": {"kind": "all"}, "samples_per_pixel": 2},
    "ortho_single": {"model": "ortho", "active": {"kind": "mask", "values": [False, True]}},
    "ortho_mixed": {"model": "ortho", "active": {"kind": "all"}},
    "znear_x2_default": {"model": "znear_x2", "active": {"kind": "all"}},
    "dynamic_default": {
        "model": "default",
        "active": {"kind": "all"},
        "use_precomputed_rays": False,
    },
    "dynamic_ortho": {
        "model": "ortho",
        "active": {"kind": "all"},
        "use_precomputed_rays": False,
    },
}

DYNAMIC_EQUALITIES = {"dynamic_default": "default_all", "dynamic_ortho": "ortho_mixed"}

REJECTIONS = [
    {
        "name": "spp_zero",
        "model": "default",
        "kwargs": {"samples_per_pixel": 0},
        "product_field": "samples_per_pixel",
    },
    {
        "name": "spp2_dynamic",
        "model": "default",
        "kwargs": {"samples_per_pixel": 2, "use_precomputed_rays": False},
        "product_field": "precomputed_rays_required",
    },
    {
        "name": "spp2_no_rgb",
        "model": "default",
        "kwargs": {"samples_per_pixel": 2, "render_rgb": False},
        "product_field": "render_rgb_required",
    },
    {
        "name": "res_count_mismatch",
        "model": "default",
        "kwargs": {"cam_res": [(50, 50), (60, 60), (70, 70)]},
        "product_field": "cam_res",
    },
    {
        "name": "mask_length_mismatch",
        "model": "default",
        "kwargs": {"cam_active": [True]},
        "product_field": "cam_active",
    },
    {
        "name": "unknown_camera_name",
        "model": "default",
        "kwargs": {"cam_active": ["no-such-camera"]},
        "product_field": "cam_active_name",
    },
    {
        "name": "camera_index_out_of_range",
        "model": "default",
        "kwargs": {"cam_active": [5]},
        "product_field": "cam_active_index",
    },
]


def capture_contexts() -> dict:
    captured = {}
    for name, spec in CONTEXTS.items():
        mjm = build_model(MODEL_SPECS[spec["model"]])
        kwargs = context_kwargs(spec)
        rc = mjw.create_render_context(mjm, **kwargs)
        wp.synchronize()
        data = capture_context(rc)
        data["has_orthographic_camera"] = bool(getattr(rc, "has_orthographic_camera", False))
        captured[name] = {"model": spec["model"], "request": context_request(spec), "expected": data}
        del rc
        gc.collect()
    return captured


def capture_rejections() -> list[dict]:
    captured = []
    for spec in REJECTIONS:
        mjm = build_model(MODEL_SPECS[spec["model"]])
        kwargs = dict(BASE_ARGS)
        kwargs.update(spec["kwargs"])
        record = {"name": spec["name"], "model": spec["model"], "kwargs": spec["kwargs"],
                  "product_field": spec["product_field"]}
        try:
            rc = mjw.create_render_context(mjm, **kwargs)
            del rc
            record["upstream_type"] = "none"
            record["upstream_message"] = "no error"
        except Exception as error:  # authoring records the frozen upstream behaviour
            record["upstream_type"] = type(error).__name__
            record["upstream_message"] = str(error)
        captured.append(record)
        gc.collect()
    return captured


def capture_pass() -> dict:
    return {"contexts": capture_contexts(), "rejections": capture_rejections()}


def pass_equal(first: dict, second: dict) -> list[str]:
    problems = []
    if list(first["contexts"]) != list(second["contexts"]):
        problems.append("context key set")
    for name, payload in first["contexts"].items():
        other = second["contexts"][name]
        if payload["expected"] != other["expected"]:
            problems.append(f"context {name}")
    if first["rejections"] != second["rejections"]:
        problems.append("rejections")
    return problems


def main() -> int:
    environment = check_environment(wp, mujoco, mjw)
    baseline_mib = memory_used_mib()
    print(f"S5 environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    models = {name: model_document(build_model(spec), spec) for name, spec in MODEL_SPECS.items()}
    first = capture_pass()
    started = time.perf_counter()
    second = capture_pass()
    elapsed = time.perf_counter() - started
    problems = pass_equal(first, second)
    if problems:
        fail(f"two captures differ: {problems}")
    for dynamic, static in DYNAMIC_EQUALITIES.items():
        dynamic_expected = first["contexts"][dynamic]["expected"]
        static_expected = first["contexts"][static]["expected"]
        if dynamic_expected["sha256"] != static_expected["sha256"]:
            fail(f"{dynamic} dynamic rays differ from {static}")
    document = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s5-camera-ray-reference",
        "scope": {
            "space": "camera-space local dir/offset only; world transform is S7b",
            "aa_image": "deferred to S7b/S8; this fixture verifies the ray layer only",
        },
        "models": models,
        "contexts": first["contexts"],
        "dynamic_equalities": DYNAMIC_EQUALITIES,
        "rejections": first["rejections"],
        "sources": {
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": XML_SHA256},
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": MJB_SHA256},
        },
        "environment": environment,
        "determinism": {"repeat": 2, "bitwise_identical": True, "second_pass_seconds": round(elapsed, 6)},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": memory_used_mib()},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh-s5/capture_s5_camera_rays.py"},
        "authoring_only": True,
        "reads_product_tree": False,
        "product_python_dependency": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({
        "output": str(OUT.relative_to(ROOT)),
        "contexts": {name: payload["expected"]["sha256"][:16] for name, payload in first["contexts"].items()},
        "rejections": {rejection["name"]: rejection["upstream_type"] for rejection in first["rejections"]},
        "gpu_memory_mib": document["gpu_memory_mib"],
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
