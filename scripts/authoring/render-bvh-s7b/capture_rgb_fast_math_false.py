"""S7b2 authoring-only: capture the **same-recipe** (`use_fast_math=False`) RGB reference.

Why: S0b froze `rgb.npz` with `PARAMS_28.use_fast_math=True`, while the product render recipe
(`WARP_1_15_RENDER`) is frozen at `fast_math=false`. The product RGB run against the S0b
fast-math reference showed 332 / 43,200 camera-1 pixels beyond 1 LSB (max 3 bytes, all on the
textured sphere/cylinder shading path), which is the same cross-recipe situation S7b1 resolved
for depth. Arch 2026-10-25 decision (S7b1 B-mode precedent, applied to RGB): the `<= 1 LSB`
metric is judged against this same-recipe reference; the S0b fast-math difference is recorded as
a **known cross-recipe deviation** (quantized histogram), never as a pass criterion and never by
relaxing the tolerance.

OFFLINE ONLY. Pinned 3.15 environment + the same `scene-315.mjb` / `PARAMS_28` as S0b, only
`use_fast_math` flipped to False. One `create_render_context` + two `mjwarp.render` calls (GPU,
seconds); no new scene, no new ray set, no product build, no Python in the product test path.

Output: `fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json`
  - `cameras[].rgb_bits_packed`: raw upstream `rc.rgb_data` packed u32 (ABGR, a=255, `int()`
    truncation), row-major; same schema as the S0b-derived `rgb.json`.
  - `cross_recipe`: vs S0b `reference/rgb.npz` (unpacked f32 -> bytes), per-camera byte-diff
    histogram `[0,1,2,3,>3]` and `>1 LSB` pixel count + geometry breakdown (deviation record).
  - `self_check`: `unpack(rgb_data)` vs upstream `get_rgb` bitwise; depth/seg bitwise vs the
    S7b1 `depth_seg_fast_math_false.json` same-recipe capture (regime proof).
  - `kernel_cache`: generated megakernel source/PTX hashes and div/sqrt op counts
    (`div.rn`/`sqrt.rn` must be non-zero, `div.approx`/`sqrt.approx` must be zero).
  - `determinism`: two renders bitwise identical.

Run (pinned venv, repo root):
  target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/capture_rgb_fast_math_false.py
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
REFERENCE = SCENE / "reference"
OUTPUT = S7B / "rgb_fast_math_false.json"
MJB = SCENE / "scene-315.mjb"
MODEL_312 = SCENE / "model-312.json"
S0B_RGB = REFERENCE / "rgb.npz"
S0B_DEPTH = REFERENCE / "depth.npz"
S0B_SEG = REFERENCE / "seg.npz"
S0B_RAYS = REFERENCE / "rays.npz"
S7B1_FMFALSE = S7B / "depth_seg_fast_math_false.json"
CACHE = ROOT / "target/render-bvh/s7b2-fastmath-cache"
PRODUCER_TASK_ID = "2026-10-25-render-bvh-s7b2-shading"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_SHA256 = {
    "scene-315.mjb": "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4",
    "model-312.json": "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
    "rgb.npz": "9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c",
    "depth.npz": "0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3",
    "seg.npz": "2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97",
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


def one_over_255() -> np.float32:
    return np.float32(1.0) / np.float32(255.0)


def packed_bytes(packed: np.ndarray) -> np.ndarray:
    """packed u32 ABGR -> uint8 (h, w, 3) r/g/b bytes (upstream `unpack_rgb_kernel` operands)."""
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)).astype(np.uint8),
            (values >> np.uint32(8) & np.uint32(0xFF)).astype(np.uint8),
            (values & np.uint32(0xFF)).astype(np.uint8),
        ],
        axis=-1,
    )


def unpack_float(packed: np.ndarray) -> np.ndarray:
    """upstream `unpack_rgb_kernel`: `float32(byte) * float32(1/255)`."""
    bytes_rgb = packed_bytes(packed).astype(np.float32)
    return bytes_rgb * one_over_255()


def byte_histogram(actual: np.ndarray, expected: np.ndarray) -> dict:
    """Per-pixel max channel byte diff histogram `[0,1,2,3,>3]` (packed row-major arrays)."""
    left = packed_bytes(actual).astype(np.int16).reshape(-1, 3)
    right = packed_bytes(expected).astype(np.int16).reshape(-1, 3)
    max_diff = np.abs(left - right).max(axis=1)
    beyond = int(np.count_nonzero(max_diff > 1))
    histogram = [int(np.count_nonzero(max_diff == value)) for value in (0, 1, 2, 3)]
    histogram.append(int(np.count_nonzero(max_diff > 3)))
    return {
        "histogram_0_1_2_3_gt3": histogram,
        "beyond_1_lsb": beyond,
        "max_byte_diff": int(max_diff.max()) if max_diff.size else 0,
        "mismatched_pixels": int(np.count_nonzero(max_diff > 0)),
    }


def render_once(m, d, rc) -> dict:
    mjw.render(m, d, rc)
    wp.synchronize()
    cam_res = np.asarray(rc.cam_res.numpy())
    rgb_adr = np.asarray(rc.rgb_adr.numpy())
    depth_adr = np.asarray(rc.depth_adr.numpy())
    seg_adr = np.asarray(rc.seg_adr.numpy())
    rgb_raw = np.asarray(rc.rgb_data.numpy())[0]
    depth_raw = np.asarray(rc.depth_data.numpy())[0].astype(np.float32)
    seg_raw = np.asarray(rc.seg_data.numpy())[0].astype(np.int32)
    cameras = []
    for index in range(int(rc.nrender)):
        width, height = int(cam_res[index][0]), int(cam_res[index][1])
        pixels = width * height
        rgb_start = int(rgb_adr[index])
        camera = {
            "index": index,
            "width": width,
            "height": height,
            "pixels": pixels,
            "rgb_adr": rgb_start,
            "rgb_packed": rgb_raw[rgb_start : rgb_start + pixels].astype(np.uint32),
            "depth": depth_raw[int(depth_adr[index]) : int(depth_adr[index]) + pixels],
            "seg": seg_raw[int(seg_adr[index]) : int(seg_adr[index]) + pixels],
        }
        # self-check: unpack(rgb_data) must equal upstream get_rgb bitwise.
        rgb = wp.zeros((1, height, width), dtype=wp.vec3)
        mjw.get_rgb(rc, index, rgb)
        wp.synchronize()
        get_rgb = np.asarray(rgb.numpy())[0].astype(np.float32).reshape(-1, 3)
        unpacked = unpack_float(camera["rgb_packed"])
        camera["unpack_matches_get_rgb"] = bool(
            np.array_equal(unpacked.view(np.uint32), get_rgb.view(np.uint32))
        )
        cameras.append(camera)
    return {
        "cam_res": [[int(w), int(h)] for w, h in cam_res.tolist()],
        "rgb_adr": [int(value) for value in rgb_adr],
        "depth_adr": [int(value) for value in depth_adr],
        "seg_adr": [int(value) for value in seg_adr],
        "rgb_pixels": int(np.asarray(rc.rgb_data.numpy()).shape[1]),
        "cameras": cameras,
    }


def kernel_cache_evidence() -> dict:
    candidates = sorted(CACHE.rglob("wp__build_megakernel__locals___render_megakernel_*.sm89.ptx"))
    if not candidates:
        raise SystemExit(f"no megakernel PTX under {CACHE}")
    ptx_path = candidates[-1]
    ptx_text = ptx_path.read_text(encoding="utf-8", errors="replace")
    source_path = ptx_path.with_suffix(".cu")
    return {
        "ptx_path": str(ptx_path.relative_to(ROOT)).replace("\\", "/"),
        "ptx_bytes": ptx_path.stat().st_size,
        "ptx_sha256": sha256(ptx_path),
        "source_bytes": source_path.stat().st_size if source_path.exists() else None,
        "source_sha256": sha256(source_path) if source_path.exists() else None,
        "div_rn": ptx_text.count("div.rn"),
        "div_approx": ptx_text.count("div.approx"),
        "sqrt_rn": ptx_text.count("sqrt.rn"),
        "sqrt_approx": ptx_text.count("sqrt.approx"),
        "rcp_approx": ptx_text.count("rcp.approx"),
    }


def depth_seg_regime_check(cameras: list[dict]) -> dict:
    """Bitwise self-proof that this capture shares the S7b1 fast_math=False render regime."""
    if not S7B1_FMFALSE.exists():
        return {"available": False, "reason": f"missing {S7B1_FMFALSE.name}"}
    document = json.loads(S7B1_FMFALSE.read_text(encoding="utf-8"))
    checks = []
    for camera, expected in zip(cameras, document["cameras"]):
        depth_bits = np.ascontiguousarray(camera["depth"], dtype=np.float32).view(np.uint32)
        seg_pairs = np.ascontiguousarray(camera["seg"], dtype=np.int32).reshape(-1)
        expected_bits = np.asarray(expected["depth_bits"], dtype=np.uint32)
        expected_pairs = np.asarray(expected["seg_pairs"], dtype=np.int32)
        checks.append(
            {
                "index": camera["index"],
                "depth_bitwise_equal": bool(np.array_equal(depth_bits, expected_bits)),
                "seg_bitwise_equal": bool(np.array_equal(seg_pairs, expected_pairs)),
            }
        )
    return {
        "available": True,
        "compared_with": str(S7B1_FMFALSE.relative_to(ROOT)).replace("\\", "/"),
        "s7b1_depth_seg_fast_math_false_sha256": sha256(S7B1_FMFALSE),
        "per_camera": checks,
        "all_equal": all(item["depth_bitwise_equal"] and item["seg_bitwise_equal"] for item in checks),
    }


def main() -> int:
    paths = {
        "scene-315.mjb": MJB,
        "model-312.json": MODEL_312,
        "rgb.npz": S0B_RGB,
        "depth.npz": S0B_DEPTH,
        "seg.npz": S0B_SEG,
        "rays.npz": S0B_RAYS,
    }
    for name, expected in EXPECTED_SHA256.items():
        actual = sha256(paths[name])
        if actual != expected:
            print(f"FAIL: {name} sha256 {actual} != {expected}", file=sys.stderr)
            return 1
    versions = installed_versions()
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            print(f"FAIL: {name} {versions[name]} != {expected}", file=sys.stderr)
            return 1
    gpu_name = subprocess.check_output(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"], text=True
    ).strip()
    if not gpu_name.startswith(GPU_NAME):
        print(f"FAIL: gpu {gpu_name} != {GPU_NAME}", file=sys.stderr)
        return 1

    s0b_rgb = np.load(S0B_RGB)
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    m = mjw.put_model(mjm)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    rc = mjw.create_render_context(mjm, **PARAMS_28)
    first = render_once(m, d, rc)
    second = render_once(m, d, rc)
    determinism = all(
        np.array_equal(a["rgb_packed"], b["rgb_packed"])
        and np.array_equal(a["depth"], b["depth"])
        and np.array_equal(a["seg"], b["seg"])
        for a, b in zip(first["cameras"], second["cameras"])
    )
    if not determinism:
        print("FAIL: two renders are not bitwise identical", file=sys.stderr)
        return 1
    if first["cam_res"] != [[320, 240], [240, 180]]:
        print(f"FAIL: cam_res {first['cam_res']}", file=sys.stderr)
        return 1
    for camera in first["cameras"]:
        if not camera["unpack_matches_get_rgb"]:
            print(
                f"FAIL: cam{camera['index']} unpack(rgb_data) != get_rgb bitwise",
                file=sys.stderr,
            )
            return 1

    cross_recipe = []
    cameras = []
    for camera, s0b in zip(first["cameras"], (s0b_rgb["rgb_cam0"], s0b_rgb["rgb_cam1"])):
        index = camera["index"]
        s0b_float = np.ascontiguousarray(s0b, dtype=np.float32).reshape(-1, 3)
        # S0b npz stores unpacked f32 -> exact bytes via rint(k * 255) (same rule as convert_rgb.py).
        s0b_bytes = np.rint(s0b_float.astype(np.float64) * 255.0).astype(np.int64)
        if s0b_bytes.min() < 0 or s0b_bytes.max() > 255:
            print(f"FAIL: cam{index} S0b byte out of range", file=sys.stderr)
            return 1
        s0b_packed = (
            (np.uint32(255) << np.uint32(24))
            | (s0b_bytes[:, 0].astype(np.uint32) << np.uint32(16))
            | (s0b_bytes[:, 1].astype(np.uint32) << np.uint32(8))
            | s0b_bytes[:, 2].astype(np.uint32)
        )
        cross = byte_histogram(camera["rgb_packed"], s0b_packed)
        cross["index"] = index
        cross_recipe.append(cross)
        packed = camera["rgb_packed"]
        cameras.append(
            {
                "index": index,
                "width": camera["width"],
                "height": camera["height"],
                "pixels": camera["pixels"],
                "rgb_adr": camera["rgb_adr"],
                "rgb_bits_packed": [int(value) for value in packed.tolist()],
                "unique_pixels": int(np.unique(packed).size),
                "black_pixels": int(np.count_nonzero(packed == np.uint32(0xFF000000))),
            }
        )

    regime = depth_seg_regime_check(first["cameras"])
    if regime.get("available") and not regime["all_equal"]:
        print(f"FAIL: depth/seg differ from S7b1 same-recipe capture: {regime}", file=sys.stderr)
        return 1
    cache = kernel_cache_evidence()
    if cache["div_rn"] == 0 or cache["div_approx"] != 0:
        print(
            f"FAIL: recipe is not fast_math=False: div_rn={cache['div_rn']} div_approx={cache['div_approx']}",
            file=sys.stderr,
        )
        return 1

    total_pixels = sum(camera["pixels"] for camera in cameras)
    document = {
        "schema": 1,
        "task_id": PRODUCER_TASK_ID,
        "kind": "render-bvh-s7b2-rgb-fast-math-false-reference",
        "authoring_only": True,
        "product_python_dependency": False,
        "recipe": {
            "create_render_context": "PARAMS_28 with use_fast_math=False (all other 27 params identical to S0b)",
            "purpose": "RGB <=1 LSB pass criterion for product WARP_1_15_RENDER (fast_math=false); "
                       "S0b fast-math RGB difference is a recorded known cross-recipe deviation",
            "compile_fast_math": False,
        },
        "encoding": {
            "rgb": "packed u32 ABGR-A8: a=255; channels = (a<<24)|(r<<16)|(g<<8)|b, "
                   "r/g/b = int(clamp(channel,0,1)*255) (truncation, not rounding); "
                   "unpack = float32(byte) * float32(1/255), upstream `unpack_rgb_kernel`",
            "pixel_order": "row-major (y outer, x fastest)",
            "tolerance": "unpacked per channel <= 1 LSB (1/255)",
        },
        "source": {
            "upstream": UPSTREAM,
            "scene_315_mjb_sha256": EXPECTED_SHA256["scene-315.mjb"],
            "model_312_sha256": EXPECTED_SHA256["model-312.json"],
            "s0b_rgb_npz_sha256": EXPECTED_SHA256["rgb.npz"],
            "s0b_depth_npz_sha256": EXPECTED_SHA256["depth.npz"],
            "s0b_seg_npz_sha256": EXPECTED_SHA256["seg.npz"],
            "s0b_rays_npz_sha256": EXPECTED_SHA256["rays.npz"],
            "capture_script_sha256": script_sha256(),
            "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "semantics": "raw rc.rgb_data (packed ABGR, a=255) and raw rc.depth_data/rc.seg_data",
        },
        "resolved": {
            "cam_res": first["cam_res"],
            "total_pixels": total_pixels,
            "rgb_pixels": first["rgb_pixels"],
            "rgb_adr": first["rgb_adr"],
            "depth_adr": first["depth_adr"],
            "seg_adr": first["seg_adr"],
        },
        "cameras": cameras,
        "cross_recipe": {
            "compared_with": "S0b rgb.npz (PARAMS_28.use_fast_math=True, unpacked f32 -> bytes)",
            "treatment": "byte-diff histogram is a known cross-recipe deviation record "
                         "(not a pass criterion); pass criterion is same-recipe <=1 LSB",
            "per_camera": cross_recipe,
        },
        "self_check": {
            "unpack_matches_get_rgb": True,
            "depth_seg_recipe": regime,
        },
        "kernel_cache": cache,
        "determinism": {"render_repeat": 2, "bitwise_identical": determinism},
        "environment": {
            "versions": versions,
            "gpu": gpu_name.split(",")[0].strip(),
            "gpu_driver": gpu_name.split(",")[1].strip(),
            "python_executable": sys.executable,
            "warp_kernel_cache_dir": str(CACHE.relative_to(ROOT)).replace("\\", "/"),
        },
    }
    text = json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n"
    parsed = json.loads(text)
    for camera, source in zip(parsed["cameras"], first["cameras"]):
        if camera["rgb_bits_packed"] != [int(value) for value in source["rgb_packed"].tolist()]:
            print(f"FAIL: round-trip cam{camera['index']}", file=sys.stderr)
            return 1
        if len(camera["rgb_bits_packed"]) != camera["width"] * camera["height"]:
            print(f"FAIL: length cam{camera['index']}", file=sys.stderr)
            return 1
    S7B.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(text.encode("utf-8"))
    written = OUTPUT.read_bytes()
    if sha256_bytes(written) != sha256_bytes(text.encode("utf-8")):
        print("FAIL: written bytes differ from serialized text", file=sys.stderr)
        return 1
    print(
        f"wrote {OUTPUT.relative_to(ROOT)}: {len(written)} bytes sha256={sha256_bytes(written)}, "
        f"total_pixels={total_pixels}, deterministic={determinism}, "
        f"ptx div_rn={cache['div_rn']} div_approx={cache['div_approx']} "
        f"sqrt_rn={cache['sqrt_rn']} sqrt_approx={cache['sqrt_approx']}, "
        f"cross_recipe={[{'cam': c['index'], 'hist': c['histogram_0_1_2_3_gt3'], 'beyond1': c['beyond_1_lsb'], 'max': c['max_byte_diff']} for c in cross_recipe]}, "
        f"depth_seg_recipe_equal={regime.get('all_equal')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
