"""S7b3 authoring-only: flag 变体矩阵 + AA(spp=2) 的同配方参考（渲染/BVH S7b3）。

口径（Arch/Chief 2026-10-25）：
- 每组 flag 变体各自一份同配方（`use_fast_math=False`）参考，逐变体独立判据
  （RGB ≤1 LSB、depth ≤4 ULP、seg 全等）；不允许「开了不崩就算过」。
- 变体语义要可判别：texture off 不采样纹理、vertex-normal off 用面法线、
  culling off 背面命中、specular/emission off 对应贡献消失；本脚本在 index.json 里记录
  「变体 vs base」的差异统计（像素数、最大 byte 差、按 seg geom 的分布），供测试做
  独立断言；若某变体与 base 无差异（无判别力），把 `has_discriminating_effect=false`
  写进 index 并由测试报出。
- AA(spp=2)：`samples_per_pixel=2` → nsamples=4（2×2），每像素 4 条子样本射线
  （S5 布局 `[sample][camera][row][column]`，与上游 `for sample in range(nsamples)`
  的 `ray_base = sample * total_rays` 一致）；上游 `mjwarp.render` 内部完成
  `aa_accum` 累积与 `_aa_resolve`，`rc.rgb_data` 即合成后的 packed 结果。
- 只读 S0b 冻结夹具与 scene-315.mjb；不改 S2–S6 证据/夹具；不重采 rays。

输出（`fixtures/render-bvh/scene-a/s7b/flags/`）：
- `<variant>_rgb.json`：两相机 packed u32 ABGR 全图；
- `<variant>_depth_seg.json`（仅 culling_off / aa2）：f32 depth 位型 + seg 对；
- `index.json`：变体清单、override、文件 sha256、两次捕获位等、与 base 的差异统计、
  provenance 与 environment。

运行（pinned venv，仓库根）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/capture_flags_aa.py
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
BASE_DEPTH_SEG = S7B / "depth_seg_fast_math_false.json"
REFERENCE = SCENE / "reference"
CACHE = ROOT / "target/render-bvh/s7b3-flags-cache"
PRODUCER_TASK_ID = "2026-10-25-render-bvh-s7b3-flags"
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
# (变体名, override, 是否需要 depth/seg 参考)
VARIANTS = [
    ("textures_off", {"use_textures": False}, False),
    ("vertex_normals_off", {"enable_vertex_normals": False}, False),
    ("specular_off", {"enable_specular": False}, False),
    ("emission_off", {"enable_emission": False}, False),
    ("culling_off", {"enable_backface_culling": False}, True),
    ("aa2", {"samples_per_pixel": 2}, True),
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


def capture(rc, m, d) -> dict:
    """渲染一次并捕获两相机 rgb packed + depth + seg（上游 raw 缓冲语义）。"""
    mjw.render(m, d, rc)
    wp.synchronize()
    cam_res = np.asarray(rc.cam_res.numpy())
    rgb_adr = np.asarray(rc.rgb_adr.numpy())
    depth_adr = np.asarray(rc.depth_adr.numpy())
    seg_adr = np.asarray(rc.seg_adr.numpy())
    rgb_raw = np.asarray(rc.rgb_data.numpy())[0]
    depth_raw = np.asarray(rc.depth_data.numpy())[0].astype(np.float32)
    # `seg_data` 是 `(nworld, seg_slots, 2)`（每槽一个 i32 对）；`seg_adr` 以槽（pair）计。
    seg_numpy = np.asarray(rc.seg_data.numpy())
    seg_raw = seg_numpy.reshape(seg_numpy.shape[0], -1, 2)[0].astype(np.int32)
    cameras = []
    for index in range(int(rc.nrender)):
        width, height = int(cam_res[index][0]), int(cam_res[index][1])
        pixels = width * height
        rgb_start = int(rgb_adr[index])
        depth_start = int(depth_adr[index])
        seg_start = int(seg_adr[index])
        cameras.append(
            {
                "index": index,
                "width": width,
                "height": height,
                "pixels": pixels,
                "rgb_packed": rgb_raw[rgb_start : rgb_start + pixels].astype(np.uint32),
                "seg_pairs": seg_raw[seg_start : seg_start + pixels],
                "depth": depth_raw[depth_start : depth_start + pixels],
            }
        )
    return {"cam_res": [[int(w), int(h)] for w, h in cam_res.tolist()], "cameras": cameras}


def byte_diff(actual: np.ndarray, expected: np.ndarray) -> np.ndarray:
    """返回每像素三通道最大 byte 差（packed ABGR）。"""
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


def diff_stats(actual: dict, base: dict) -> list[dict]:
    """变体 vs base 的逐相机差异统计（含按 seg geom 的分布）。"""
    stats = []
    for camera, base_camera in zip(actual["cameras"], base["cameras"]):
        rgb_diff = byte_diff(camera["rgb_packed"], base_camera["rgb_packed"])
        beyond = np.count_nonzero(rgb_diff > 1)
        seg_changed = np.count_nonzero(np.any(camera["seg_pairs"] != base_camera["seg_pairs"], axis=1))
        depth_changed = np.count_nonzero(
            np.asarray(camera["depth"]).view(np.uint32) != np.asarray(base_camera["depth"]).view(np.uint32)
        )
        by_geom: dict[int, int] = {}
        for pixel in np.nonzero(rgb_diff > 0)[0].tolist():
            geom = int(camera["seg_pairs"][pixel][0])
            by_geom[geom] = by_geom.get(geom, 0) + 1
        stats.append(
            {
                "index": camera["index"],
                "rgb_changed_pixels": int(np.count_nonzero(rgb_diff > 0)),
                "rgb_beyond_1_lsb": int(beyond),
                "rgb_max_byte_diff": int(rgb_diff.max()) if rgb_diff.size else 0,
                "rgb_changed_by_geom": {str(key): value for key, value in sorted(by_geom.items())},
                "seg_changed_pixels": int(seg_changed),
                "depth_changed_pixels": int(depth_changed),
            }
        )
    return stats


def load_base() -> dict:
    rgb_doc = json.loads(BASE_RGB.read_text(encoding="utf-8"))
    ds_doc = json.loads(BASE_DEPTH_SEG.read_text(encoding="utf-8"))
    cameras = []
    for rgb_camera, ds_camera in zip(rgb_doc["cameras"], ds_doc["cameras"]):
        if rgb_camera["index"] != ds_camera["index"]:
            raise SystemExit("base fixture camera order mismatch")
        camera = {
            "index": rgb_camera["index"],
            "width": rgb_camera["width"],
            "height": rgb_camera["height"],
            "pixels": rgb_camera["pixels"],
            "rgb_packed": np.asarray(rgb_camera["rgb_bits_packed"], dtype=np.uint32),
            "seg_pairs": np.asarray(ds_camera["seg_pairs"], dtype=np.int32).reshape(-1, 2),
            "depth": np.asarray(ds_camera["depth_bits"], dtype=np.uint32).view(np.float32),
        }
        cameras.append(camera)
    return {"cameras": cameras}


def camera_rgb_documents(capture_doc: dict) -> list[dict]:
    return [
        {
            "index": camera["index"],
            "width": camera["width"],
            "height": camera["height"],
            "pixels": camera["pixels"],
            "rgb_bits_packed": [int(value) for value in camera["rgb_packed"].tolist()],
        }
        for camera in capture_doc["cameras"]
    ]


def camera_depth_seg_documents(capture_doc: dict) -> list[dict]:
    cameras = []
    depth_cursor = 0
    seg_cursor = 0
    for camera in capture_doc["cameras"]:
        bits = np.ascontiguousarray(camera["depth"], dtype=np.float32).view(np.uint32).tolist()
        positives = [value for value in bits if value != 0]
        pairs_2d = camera["seg_pairs"].reshape(-1, 2).tolist()
        cameras.append(
            {
                "index": camera["index"],
                "width": camera["width"],
                "height": camera["height"],
                "pixels": camera["pixels"],
                "depth_adr": depth_cursor,
                "seg_adr": seg_cursor,
                "depth_zero_pixels": int(len(bits) - len(positives)),
                "depth_min_positive_hex": f"0x{min(positives):08x}" if positives else None,
                "depth_max_hex": f"0x{max(bits):08x}",
                "depth_bits": bits,
                "seg_pairs": [int(value) for value in camera["seg_pairs"].reshape(-1).tolist()],
                "seg_unique": [list(pair) for pair in sorted({tuple(pair) for pair in pairs_2d})],
            }
        )
        depth_cursor += camera["pixels"]
        seg_cursor += camera["pixels"]
    return cameras


def camera_data_checks() -> list[dict]:
    """复用 S7b1 脚本的相机 data 位姿证据（同 schema；`parse_reference` 校验索引连续与复算误差）。"""
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    from convert_depth_seg import camera_data_checks as checks  # noqa: PLC0415

    return checks(
        np.load(REFERENCE / "rays.npz"),
        json.loads(MODEL_312.read_text(encoding="utf-8")),
    )


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
        "rgb.npz": REFERENCE / "rgb.npz",
        "rays.npz": REFERENCE / "rays.npz",
    }
    for name, expected in EXPECTED_SHA256.items():
        actual = sha256(paths[name])
        if actual != expected:
            print(f"FAIL: {name} sha256 {actual} != {expected}", file=sys.stderr)
            return 1
    for path in (BASE_RGB, BASE_DEPTH_SEG):
        if not path.exists():
            print(f"FAIL: missing base fixture {path}", file=sys.stderr)
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
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()

    base = load_base()
    entries = []
    for name, override, with_depth_seg in VARIANTS:
        params = {**PARAMS_28, **override}
        rc = mjw.create_render_context(mjm, **params)
        first = capture(rc, m, d)
        second = capture(rc, m, d)
        deterministic = all(
            np.array_equal(a["rgb_packed"], b["rgb_packed"])
            and np.array_equal(a["depth"], b["depth"])
            and np.array_equal(a["seg_pairs"], b["seg_pairs"])
            for a, b in zip(first["cameras"], second["cameras"])
        )
        if not deterministic:
            print(f"FAIL: {name} two captures differ", file=sys.stderr)
            return 1
        expected_res = [[320, 240], [240, 180]]
        if first["cam_res"] != expected_res:
            print(f"FAIL: {name} cam_res {first['cam_res']}", file=sys.stderr)
            return 1

        rgb_document = {
            "schema": 1,
            "task_id": PRODUCER_TASK_ID,
            "kind": "render-bvh-s7b3-flag-variant-rgb",
            "variant": name,
            "override": override,
            "encoding": {
                "rgb": "packed u32 ABGR-A8（a=255；int() 截断），row-major；解包 float32(byte)*float32(1/255)",
                "tolerance": "解包后每通道 <=1 LSB",
            },
            "cameras": camera_rgb_documents(first),
        }
        rgb_path = OUT / f"{name}_rgb.json"
        rgb_sha = write_json(rgb_path, rgb_document)
        files = {rgb_path.name: rgb_sha}
        if with_depth_seg:
            depth_docs = camera_depth_seg_documents(first)

            ds_document = {
                "schema": 1,
                "task_id": PRODUCER_TASK_ID,
                "kind": "render-bvh-s7b3-flag-variant-depth-seg",
                "variant": name,
                "encoding": {
                    "depth": "IEEE-754 binary32 bits as unsigned decimal u32, row-major",
                    "seg": "int32 [object_id, object_type] pairs, row-major (2 * pixels values)",
                },
                "recipe": {
                    "compile_fast_math": False,
                    "purpose": "S7b3 flag 变体/AA 的 depth ≤4 ULP 与 seg 全等判据（同配方参考）",
                },
                "resolved": {
                    "total_rays": sum(camera["pixels"] for camera in first["cameras"]),
                    "cam_res": first["cam_res"],
                    "depth_adr": [camera["depth_adr"] for camera in depth_docs],
                    "seg_adr": [camera["seg_adr"] for camera in depth_docs],
                },
                "camera_data_check": camera_data_checks(),
                "cameras": depth_docs,
            }
            ds_path = OUT / f"{name}_depth_seg.json"
            files[ds_path.name] = write_json(ds_path, ds_document)
        stats = diff_stats(first, base)
        result = {
            "variant": name,
            "override": override,
            "files": files,
            "determinism": {"repeat": 2, "bitwise_identical": True},
            "stats_vs_base": stats,
            "has_discriminating_effect": any(
                item["rgb_changed_pixels"] > 0
                or item["seg_changed_pixels"] > 0
                or item["depth_changed_pixels"] > 0
                for item in stats
            ),
        }
        entries.append(result)
        print(
            f"{name}: rgb_sha={rgb_sha} "
            f"changed={[item['rgb_changed_pixels'] for item in stats]} "
            f"seg_changed={[item['seg_changed_pixels'] for item in stats]} "
            f"depth_changed={[item['depth_changed_pixels'] for item in stats]}"
        )

    index_document = {
        "schema": 1,
        "task_id": PRODUCER_TASK_ID,
        "kind": "render-bvh-s7b3-flag-matrix-index",
        "authoring_only": True,
        "product_python_dependency": False,
        "recipe": {
            "use_fast_math": False,
            "base_params": "S0b PARAMS_28 的 28 参，仅 use_fast_math=False",
            "aa_note": "samples_per_pixel=2 → nsamples=4；上游 mjwarp.render 内部完成 aa_accum 累积与 _aa_resolve；rc.rgb_data 为合成结果",
            "view": "两相机 320×240 / 240×180，与 S0b 相同",
        },
        "variants": entries,
        "base": {
            "rgb": str(BASE_RGB.relative_to(ROOT)).replace("\\", "/"),
            "rgb_sha256": sha256(BASE_RGB),
            "depth_seg": str(BASE_DEPTH_SEG.relative_to(ROOT)).replace("\\", "/"),
            "depth_seg_sha256": sha256(BASE_DEPTH_SEG),
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
    }
    index_path = OUT / "index.json"
    index_sha = write_json(index_path, index_document)
    print(f"wrote {index_path.relative_to(ROOT)}: sha256={index_sha}, variants={len(entries)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
