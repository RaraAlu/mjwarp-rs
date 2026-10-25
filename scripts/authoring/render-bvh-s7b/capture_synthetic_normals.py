"""S7b followups authoring-only: 合成顶点法线的 pinned 3.15 同配方参考。

背景（S7b3 复核缺口）：`vertex_normals_off` 在 scene-a 原法线下参考自身开/关逐位相同
（无判别力），S7b3 只用产品侧合成法线用例证明分支生效。本脚本补 pinned 参考对照：
把 `mjm.mesh_normal` 在内存里覆盖为 [1,0,0] / [0,0,1]（4 顶点同方向），用 pinned 3.15 +
同配方（`use_fast_math=False`）各采两相机全图 packed RGB；产品对合成参考须 ≤1 LSB，
且 on/off 路径可判别（合成法线下 on 与 off 渲染不同）。

口径：
- **不改 mjb 文件**：`mujoco.MjModel.from_binary_path` 后 in-memory 覆盖
  `mjm.mesh_normal[:]`，`mjwarp.put_model(mjm)` 上传覆盖值；provenance 记原 mjb sha、
  原法线数组 sha、覆盖方式、覆盖后数组 sha、脚本 sha，可复现；
- **生效断言**：覆盖后两相机渲染必须与默认同配方参考（`rgb_fast_math_false.json`）不同
  （否则覆盖没生效，参考无效 → 脚本 fail）；
- 两次捕获逐位一致；同配方 `PARAMS_28`（仅 `use_fast_math=False`）。

输出（`fixtures/render-bvh/scene-a/s7b/flags/`）：
- `synthetic_normals_x_rgb.json`：mesh_normal = [1,0,0]（4 顶点）的 pinned 参考；
- `synthetic_normals_z_rgb.json`：mesh_normal = [0,0,1]；
- `synthetic_normals_index.json`：provenance + 两位型 + 生效断言 + 两次位等。

运行（pinned venv，仓库根）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/capture_synthetic_normals.py
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import time

import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
S7B = SCENE / "s7b"
OUT = S7B / "flags"
MJB = SCENE / "scene-315.mjb"
MODEL_312 = SCENE / "model-312.json"
BASE_RGB = S7B / "rgb_fast_math_false.json"
CACHE = ROOT / "target/render-bvh/s7b-followups-cache"
PRODUCER_TASK_ID = "2026-10-25-render-bvh-s7b-followups"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_SHA256 = {
    "scene-315.mjb": "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4",
    "model-312.json": "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
    "rgb.npz": "9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c",
    "rays.npz": "83146ae50946537401be817a44bfe3af58320df475dabc49206aa47eec9d50ab",
}
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
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
# (文件后缀, 覆盖方向)
DIRECTIONS = [
    ("x", [1.0, 0.0, 0.0]),
    ("z", [0.0, 0.0, 1.0]),
]

wp.config.kernel_cache_dir = str(CACHE)

import importlib.metadata  # noqa: E402

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256(path: pathlib.Path) -> str:
    return sha256_bytes(path.read_bytes())


def script_sha256() -> str:
    return sha256_bytes(pathlib.Path(__file__).read_bytes().replace(b"\r\n", b"\n"))


def package_version(*names: str) -> str:
    for name in names:
        try:
            return importlib.metadata.version(name)
        except importlib.metadata.PackageNotFoundError:
            continue
    raise SystemExit(f"cannot resolve package version for {names}")


def installed_versions() -> dict:
    return {
        "warp": wp.__version__,
        "mujoco": mujoco.__version__,
        "mujoco_warp": getattr(mjw, "__version__", package_version("mujoco-warp", "mujoco_warp")),
        "numpy": np.__version__,
    }


def capture_once(rc, m, d) -> dict:
    """渲染一次并捕获两相机 rgb packed（上游 raw 缓冲语义）。"""
    mjw.render(m, d, rc)
    wp.synchronize()
    cam_res = np.asarray(rc.cam_res.numpy())
    rgb_adr = np.asarray(rc.rgb_adr.numpy())
    rgb_raw = np.asarray(rc.rgb_data.numpy())[0]
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
                "rgb_packed": rgb_raw[start : start + pixels].astype(np.uint32),
            }
        )
    return {"cam_res": [[int(w), int(h)] for w, h in cam_res.tolist()], "cameras": cameras}


def capture_direction(mjm, direction: list[float]) -> dict:
    """覆盖 mesh_normal 后重建 model/data/context 并两次捕获。"""
    mjm.mesh_normal[:] = np.asarray(direction, dtype=np.float32)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    first = capture_once(rc, m, d)
    second = capture_once(rc, m, d)
    return {"first": first, "second": second}


def byte_diff(actual: np.ndarray, expected: np.ndarray) -> np.ndarray:
    def channels(values: np.ndarray) -> np.ndarray:
        values = np.ascontiguousarray(values, dtype=np.uint32)
        return np.stack(
            [
                (values >> np.uint32(16) & np.uint32(0xFF)).astype(np.int16),
                (values >> np.uint32(8) & np.uint32(0xFF)).astype(np.int16),
                (values & np.uint32(0xFF)).astype(np.int16),
            ],
            axis=-1,
        )

    return np.abs(channels(actual) - channels(expected)).max(axis=1)


def load_base() -> list[np.ndarray]:
    document = json.loads(BASE_RGB.read_text(encoding="utf-8"))
    return [
        np.asarray(camera["rgb_bits_packed"], dtype=np.uint32) for camera in document["cameras"]
    ]


def write_json(path: pathlib.Path, document: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n"
    parsed = json.loads(text)
    if parsed != document:
        raise SystemExit(f"round-trip failed for {path.name}")
    path.write_bytes(text.encode("utf-8"))
    written = path.read_bytes()
    if sha256_bytes(written) != sha256_bytes(text.encode("utf-8")):
        raise SystemExit(f"written bytes differ for {path.name}")
    return sha256_bytes(written)


def main() -> int:
    paths = {
        "scene-315.mjb": MJB,
        "model-312.json": MODEL_312,
        "rgb.npz": SCENE / "reference/rgb.npz",
        "rays.npz": SCENE / "reference/rays.npz",
    }
    for name, expected in EXPECTED_SHA256.items():
        actual = sha256(paths[name])
        if actual != expected:
            print(f"FAIL: {name} sha256 {actual} != {expected}", file=sys.stderr)
            return 1
    if not BASE_RGB.exists():
        print(f"FAIL: missing base fixture {BASE_RGB}", file=sys.stderr)
        return 1
    versions = installed_versions()
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            print(f"FAIL: {name} {versions[name]} != {expected}", file=sys.stderr)
            return 1
    gpu = subprocess.check_output(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"], text=True
    ).strip()
    if not gpu.startswith(GPU_NAME):
        print(f"FAIL: gpu {gpu} != {GPU_NAME}", file=sys.stderr)
        return 1

    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    if mjm.mesh_normal.shape != (4, 3):
        print(f"FAIL: mesh_normal shape {mjm.mesh_normal.shape} != (4, 3)", file=sys.stderr)
        return 1
    if not mjm.mesh_normal.flags.writeable:
        print("FAIL: mesh_normal is not writeable", file=sys.stderr)
        return 1
    original = np.array(mjm.mesh_normal, copy=True)
    original_sha = sha256_bytes(original.tobytes())
    base = load_base()

    entries = []
    output_shas: dict[str, str] = {}
    for suffix, direction in DIRECTIONS:
        result = capture_direction(mjm, direction)
        first = result["first"]
        second = result["second"]
        deterministic = all(
            np.array_equal(a["rgb_packed"], b["rgb_packed"])
            for a, b in zip(first["cameras"], second["cameras"])
        )
        if not deterministic:
            print(f"FAIL: {suffix} two captures differ", file=sys.stderr)
            return 1
        if first["cam_res"] != [[320, 240], [240, 180]]:
            print(f"FAIL: {suffix} cam_res {first['cam_res']}", file=sys.stderr)
            return 1
        # 生效断言：覆盖后渲染必须与默认 scene-a 参考不同（否则覆盖无效/参考不可判别）。
        changed = [
            int(np.count_nonzero(byte_diff(camera["rgb_packed"], base[index]) > 0))
            for index, camera in enumerate(first["cameras"])
        ]
        if sum(changed) == 0:
            print(f"FAIL: {suffix} override has no rendering effect", file=sys.stderr)
            return 1
        overridden_sha = sha256_bytes(np.asarray(mjm.mesh_normal).tobytes())
        rgb_document = {
            "schema": 1,
            "task_id": PRODUCER_TASK_ID,
            "kind": "render-bvh-s7b-followups-synthetic-normals-rgb",
            "variant": f"synthetic_normals_{suffix}",
            "mesh_normal_override": direction,
            "encoding": {
                "rgb": "packed u32 ABGR-A8（a=255；int() 截断），row-major；解包 float32(byte)*float32(1/255)",
                "tolerance": "解包后每通道 <=1 LSB",
            },
            "cameras": [
                {
                    "index": camera["index"],
                    "width": camera["width"],
                    "height": camera["height"],
                    "pixels": camera["pixels"],
                    "rgb_bits_packed": [int(value) for value in camera["rgb_packed"].tolist()],
                }
                for camera in first["cameras"]
            ],
        }
        rgb_path = OUT / f"synthetic_normals_{suffix}_rgb.json"
        rgb_sha = write_json(rgb_path, rgb_document)
        output_shas[rgb_path.name] = rgb_sha
        entry = {
            "variant": f"synthetic_normals_{suffix}",
            "mesh_normal_override": direction,
            "file": rgb_path.name,
            "bytes": rgb_path.stat().st_size,
            "sha256": rgb_sha,
            "mesh_normal_sha256_before_override": overridden_sha,
            "effect_vs_default_scene_a": {
                "changed_pixels": changed,
                "note": "覆盖后渲染必须与默认 scene-a 同配方参考不同（生效断言）",
            },
            "determinism": {"repeat": 2, "bitwise_identical": True},
        }
        entries.append(entry)
        print(f"synthetic_normals_{suffix}: sha={rgb_sha} changed={changed}")

    index_document = {
        "schema": 1,
        "task_id": PRODUCER_TASK_ID,
        "kind": "render-bvh-s7b-followups-synthetic-normals-index",
        "authoring_only": True,
        "product_python_dependency": False,
        "purpose": "S7b3 复核缺口补采：vertex_normals 分支的 pinned 同配方参考（默认 scene-a 下该 flag 无判别力）",
        "override_method": "in-memory `mjm.mesh_normal[:] = direction`（4 顶点同方向），`mjwarp.put_model(mjm)` 上传；不改 mjb 文件",
        "recipe": {
            "use_fast_math": False,
            "base_params": "S0b PARAMS_28 的 28 参，仅 use_fast_math=False",
            "view": "两相机 320×240 / 240×180（与 S0b 相同）",
        },
        "mesh_normal": {
            "shape": [4, 3],
            "original_sha256": original_sha,
            "original_values": [float(value) for value in original.reshape(-1).tolist()],
        },
        "variants": entries,
        "base_reference": {
            "file": str(BASE_RGB.relative_to(ROOT)).replace("\\", "/"),
            "sha256": sha256(BASE_RGB),
        },
        "source": {
            "upstream": UPSTREAM,
            "scene_315_mjb_sha256": EXPECTED_SHA256["scene-315.mjb"],
            "model_312_sha256": EXPECTED_SHA256["model-312.json"],
            "s0b_rgb_npz_sha256": EXPECTED_SHA256["rgb.npz"],
            "s0b_rays_npz_sha256": EXPECTED_SHA256["rays.npz"],
            "capture_script_sha256": script_sha256(),
            "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        },
        "environment": {
            "versions": versions,
            "gpu": gpu.split(",")[0].strip(),
            "gpu_driver": gpu.split(",")[1].strip(),
            "python_executable": sys.executable,
            "warp_kernel_cache_dir": str(CACHE.relative_to(ROOT)).replace("\\", "/"),
        },
        "files": output_shas,
    }
    index_path = OUT / "synthetic_normals_index.json"
    index_sha = write_json(index_path, index_document)
    print(f"wrote {index_path.relative_to(ROOT)}: sha256={index_sha}, variants={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
