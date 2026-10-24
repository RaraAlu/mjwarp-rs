"""Offline authoring: capture the S4 texture conversion/sampling reference (frozen 3.15).

OFFLINE ONLY: never run by product builds/tests/gates. The product test reads the
committed JSON; it never starts Python.

Reference semantics (frozen MJWarp 71da24d):
  - `render_util.create_warp_texture`: u8 `tex_data` -> RGBA f32 (`* f32(1/255)`), then
    `wp.Texture2D(..., filter_mode=LINEAR)` (normalized coords, CLAMP, one mip);
  - `render_util._convert_texture_data` is launched directly to capture the conversion
    bits per texture, and the same converted data feeds the texture sample capture;
  - `wp.texture_sample(tex, uv, dtype=vec4)` is the device path used by the render
    kernel, so the sample reference is the hardware LINEAR fetch;
  - `create_render_context(use_textures=False/True)` captures the registry toggle.
The Warp `cpu` device is used only to document the hardware-vs-software filter
difference; it is not the product acceptance reference.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import platform
import subprocess
import sys
import time

import mujoco
import numpy as np
import warp as wp

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
OUT = SCENE / "s4/textures.json"
MJB, XML, MODEL_312 = SCENE / "scene-315.mjb", SCENE / "scene.xml", SCENE / "model-312.json"
CACHE = ROOT / "target/render-bvh/s4-cache"
TASK_ID, SCHEMA = "2026-10-24-render-bvh-s4", 1
HASHES = {
    MJB: "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4",
    XML: "a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289",
    MODEL_312: "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
}
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME, GPU_ARCH, GPU_DRIVER = "NVIDIA GeForce RTX 4070 Ti SUPER", 89, "596.36"

wp.config.kernel_cache_dir = str(CACHE)
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from s4_texture_support import (  # noqa: E402
    bits,
    capture_registry_semantics,
    capture_zero_size_rejection,
    convert_reference,
    convert_synthetic,
    sample_reference,
    texture_bytes,
    texture_metadata,
    uv_grid,
)


def fail(message: str) -> None:
    raise SystemExit(f"S4 texture reference capture mismatch: {message}")


def digest(path: pathlib.Path) -> str:
    data = path.read_bytes()
    return hashlib.sha256(data.replace(b"\r\n", b"\n") if path.suffix == ".xml" else data).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def bits_digest(values_bits: list[int]) -> str:
    return hashlib.sha256(
        b"".join(int(value).to_bytes(4, "little") for value in values_bits)
    ).hexdigest()


def check_environment() -> dict:
    problems = []
    versions = {
        "python": platform.python_version(),
        "warp": wp.__version__,
        "mujoco": mujoco.__version__,
        "mujoco_warp": __import__("mujoco_warp").__version__,
        "numpy": np.__version__,
    }
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            problems.append(f"{name} {versions[name]} != {expected}")
    upstream = ROOT / "target/mjwarp-parallel/upstream"
    if run(["git", "-C", str(upstream), "rev-parse", "HEAD"]) != REVISION:
        problems.append("upstream revision mismatch")
    if run(["git", "-C", str(upstream), "status", "--porcelain"]):
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
    render_util = upstream / "mujoco_warp/_src/render_util.py"
    return {
        "versions": versions,
        "platform": platform.platform(),
        "gpu": device.name,
        "gpu_arch": int(device.arch),
        "gpu_driver": GPU_DRIVER,
        "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "device_reference_code": {
            "render_util.py": digest(render_util),
            "render.py": digest(upstream / "mujoco_warp/_src/render.py"),
            "types.py": digest(upstream / "mujoco_warp/_src/types.py"),
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def cross_check_312(mjm) -> dict:
    document = json.loads(MODEL_312.read_text(encoding="utf-8"))
    arrays = document["arrays"]
    problems = []
    for name in ("tex_width", "tex_height", "tex_nchannel", "tex_adr", "tex_type"):
        expected = [int(value) for value in getattr(mjm, name)]
        actual = [int(value) for value in arrays[name]["values"]]
        if expected != actual:
            problems.append(f"{name}: {actual} != {expected}")
    model_312_data = bytes(int(value) & 0xFF for value in arrays["tex_data"]["values"])
    model_315_data = bytes(np.asarray(mjm.tex_data, dtype=np.uint8))
    if model_312_data != model_315_data:
        problems.append("tex_data bytes differ")
    if problems:
        fail(f"3.12/3.15 texture cross-check failed: {problems}")
    return {
        "tex_metadata_equal": True,
        "tex_data_equal": True,
        "tex_data_bytes": len(model_315_data),
        "tex_data_sha256": hashlib.sha256(model_315_data).hexdigest(),
    }


def capture_pass(mjm, uv_bits: list[int]) -> dict:
    metadata = texture_metadata(mjm)
    textures = []
    gpu_samples = []
    cpu_samples = []
    for meta in metadata:
        data_bits, converted_bits = convert_reference(mjm, meta["tex_id"], "cuda:0")
        textures.append(
            dict(
                meta,
                data_sha256=hashlib.sha256(bytes(data_bits)).hexdigest(),
                converted_sha256=bits_digest(converted_bits),
                converted_texels=len(converted_bits) // 4,
            )
        )
        gpu = sample_reference(converted_bits, meta["width"], meta["height"], "cuda:0")
        cpu = sample_reference(converted_bits, meta["width"], meta["height"], "cpu")
        gpu_samples.append({"tex_id": meta["tex_id"], "rgba_bits": gpu})
        cpu_samples.append({"tex_id": meta["tex_id"], "rgba_bits": cpu})
    synthetic = [
        convert_synthetic(width, height, channels, "cuda:0")
        for (width, height, channels) in ((8, 6, 1), (6, 5, 2), (5, 4, 4))
    ]
    return {
        "textures": textures,
        "samples": {"uv_bits": uv_bits, "gpu": gpu_samples, "cpu": cpu_samples},
        "synthetic": synthetic,
    }


def hardware_software_diff(first: dict) -> dict:
    """Largest per-channel |GPU - CPU| (documented, not the acceptance reference)."""
    report = []
    for gpu, cpu in zip(first["samples"]["gpu"], first["samples"]["cpu"]):
        gpu_values = np.asarray(gpu["rgba_bits"], dtype=np.uint32).view(np.float32)
        cpu_values = np.asarray(cpu["rgba_bits"], dtype=np.uint32).view(np.float32)
        difference = np.abs(gpu_values.astype(np.float64) - cpu_values.astype(np.float64))
        report.append(
            {
                "tex_id": gpu["tex_id"],
                "max_abs_diff": float(difference.max()) if difference.size else 0.0,
                "unequal_channels": int(np.count_nonzero(gpu_values != cpu_values)),
            }
        )
    return {"per_texture": report, "max_abs_diff": max(item["max_abs_diff"] for item in report)}


def main() -> int:
    environment = check_environment()
    baseline_mib = memory_used_mib()
    print(f"S4 environment OK; gpu memory baseline={baseline_mib} MiB")
    wp.init()
    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    cross = cross_check_312(mjm)
    uvs = uv_grid()
    uv_bits = bits(uvs)
    print(f"uv grid: {len(uvs)} queries")

    started = time.perf_counter()
    first = capture_pass(mjm, uv_bits)
    elapsed = time.perf_counter() - started
    second = capture_pass(mjm, uv_bits)
    if first != second:
        fail("two capture passes differ")
    registry = capture_registry_semantics(mjm)
    if registry["registry_off"] != 0 or registry["registry_on"] != registry["ntex"]:
        fail(f"upstream registry toggle mismatch: {registry}")
    zero_size = capture_zero_size_rejection()
    if zero_size.get("constructible"):
        fail(f"upstream accepted a zero-sized texture: {zero_size}")
    difference = hardware_software_diff(first)

    document = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s4-texture-reference",
        "textures": first["textures"],
        "samples": {
            "uv_count": len(uvs),
            "uv_bits": uv_bits,
            "gpu": first["samples"]["gpu"],
        },
        "synthetic": first["synthetic"],
        "registry": registry,
        "rejections": {"zero_size": zero_size},
        "hardware_vs_software": difference,
        "cross_check_312": cross,
        "sources": {
            "mjb_315": {"path": "fixtures/render-bvh/scene-a/scene-315.mjb", "sha256": HASHES[MJB]},
            "scene_xml": {"path": "fixtures/render-bvh/scene-a/scene.xml", "sha256": HASHES[XML]},
            "model_312": {
                "path": "fixtures/render-bvh/scene-a/model-312.json",
                "sha256": HASHES[MODEL_312],
            },
        },
        "environment": environment,
        "determinism": {"repeat": 2, "bitwise_identical": True, "second_pass_seconds": round(elapsed, 6)},
        "gpu_memory_mib": {"baseline": baseline_mib, "after": memory_used_mib()},
        "commands": {"capture": f"{sys.executable} -B scripts/authoring/render-bvh-s4/capture_s4_textures.py"},
        "authoring_only": True,
        "reads_product_tree": False,
        "product_python_dependency": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(
        json.dumps(
            {
                "output": str(OUT.relative_to(ROOT)),
                "sha256": digest(OUT),
                "bytes": OUT.stat().st_size,
                "uvs": len(uvs),
                "textures": [texture["tex_id"] for texture in first["textures"]],
                "registry": registry,
                "zero_size": zero_size,
                "hardware_vs_software_max_abs_diff": difference["max_abs_diff"],
                "gpu_memory_mib": document["gpu_memory_mib"],
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
