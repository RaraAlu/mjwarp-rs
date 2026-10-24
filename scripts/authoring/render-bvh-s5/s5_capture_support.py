"""S5 authoring support: environment checks, device-array capture and SHA-256 digests.

OFFLINE ONLY: imported by `capture_s5_camera_rays.py`, never by product builds/tests.

The ray arrays are read back from the frozen upstream RenderContext (`rc.ray`,
`rc.ray_offset`, f32 vec3, layout `[sample][camera][row][column]`) and hashed with the
canonical S5 byte order: dir3 then offset3 per ray, little-endian f32.
"""

from __future__ import annotations

import hashlib
import pathlib
import platform
import subprocess
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
MJB = SCENE / "scene-315.mjb"
XML = SCENE / "scene.xml"
VENV = ROOT / "target/mjwarp-parallel/venv"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
CACHE = ROOT / "target/render-bvh/s5-cache"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
MJB_SHA256 = "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4"
XML_SHA256 = "a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289"
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"
GPU_ARCH = 89
GPU_DRIVER = "596.36"


def fail(message: str) -> None:
    raise SystemExit(f"S5 camera-ray reference capture mismatch: {message}")


def digest(path: pathlib.Path) -> str:
    data = path.read_bytes()
    return hashlib.sha256(data.replace(b"\r\n", b"\n") if path.suffix == ".xml" else data).hexdigest()


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def memory_used_mib() -> int:
    return int(run(["nvidia-smi", "--query-gpu=memory.used", "--format=csv,noheader,nounits"]).splitlines()[0])


def bits_f32(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def bits_f32_scalar(value: float) -> int:
    return int(np.float32(value).view(np.uint32))


def bits_f64(value: float) -> int:
    return int(np.float64(value).view(np.uint64))


def check_environment(wp, mujoco, mjw) -> dict:
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
    if digest(MJB) != MJB_SHA256:
        problems.append("scene-315.mjb hash mismatch")
    if digest(XML) != XML_SHA256:
        problems.append("scene.xml hash mismatch")
    device = wp.get_device("cuda:0")
    if int(device.arch) != GPU_ARCH or GPU_NAME not in device.name:
        problems.append(f"gpu {device.name} arch {device.arch} mismatch")
    if run(["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"]) != GPU_DRIVER:
        problems.append("gpu driver mismatch")
    if problems:
        fail("; ".join(problems))
    return {
        "versions": versions,
        "python_executable": sys.executable,
        "python_prefix": sys.prefix,
        "platform": platform.platform(),
        "gpu": device.name,
        "gpu_arch": int(device.arch),
        "gpu_driver": GPU_DRIVER,
        "warp_cuda_toolkit": list(wp.get_cuda_toolkit_version()),
        "warp_cuda_driver": list(wp.get_cuda_driver_version()),
        "upstream_revision": REVISION,
        "upstream_clean": True,
        "device_reference_code": {
            name: digest(UPSTREAM / "mujoco_warp" / "_src" / name)
            for name in ("render_util.py", "render.py", "types.py")
        },
        "warp_kernel_cache_dir": str(CACHE),
    }


def canonical_sha256(directions: np.ndarray, offsets: np.ndarray) -> str:
    """SHA-256 over dir3 then offset3 per ray (LE f32), 与产品 `RayBuffer::canonical_bytes` 一致。"""
    directions = np.ascontiguousarray(directions, dtype="<f4")
    offsets = np.ascontiguousarray(offsets, dtype="<f4")
    digest_value = hashlib.sha256()
    digest_value.update(directions.tobytes())
    digest_value.update(offsets.tobytes())
    return digest_value.hexdigest()


def sample_entries(directions: np.ndarray, offsets: np.ndarray, cam_res, spp: int, total: int) -> list[dict]:
    """每相机 min(128, w*h) 个等距像素、覆盖全部子样本的位型抽样。"""
    entries: list[dict] = []
    cam_offsets = []
    accumulated = 0
    for width, height in cam_res:
        cam_offsets.append(accumulated)
        accumulated += int(width) * int(height)
    for position, (width, height) in enumerate(cam_res):
        width = int(width)
        height = int(height)
        pixels = width * height
        count = min(128, pixels)
        if count == 0:
            continue
        for sample in range(spp * spp):
            for slot in range(count):
                local = slot * pixels // count
                x, y = local % width, local // width
                index = sample * total + cam_offsets[position] + y * width + x
                entries.append(
                    {
                        "camera": position,
                        "sample": sample,
                        "x": x,
                        "y": y,
                        "index": index,
                        "dir_bits": bits_f32(directions[index]),
                        "offset_bits": bits_f32(offsets[index]),
                    }
                )
    return entries


def capture_context(rc) -> dict:
    """读取渲染 context 的射线缓冲与相机元数据（f32 位型 + 规范哈希）。"""
    directions = np.asarray(rc.ray.numpy(), dtype=np.float32).reshape(-1, 3)
    offsets = np.asarray(rc.ray_offset.numpy(), dtype=np.float32).reshape(-1, 3)
    cam_res = [[int(w), int(h)] for w, h in np.asarray(rc.cam_res.numpy(), dtype=np.int64)]
    spp = int(rc.samples_per_pixel)
    total = int(rc.total_rays)
    expected_count = total * spp * spp
    if directions.shape[0] != expected_count or offsets.shape[0] != expected_count:
        fail(f"ray count {directions.shape[0]}/{offsets.shape[0]} != {expected_count}")
    return {
        "nrender": int(rc.nrender),
        "cameras": [int(camera) for camera in np.asarray(rc.cam_id_map.numpy(), dtype=np.int64)],
        "cam_res": cam_res,
        "render_rgb": [bool(value) for value in np.asarray(rc.render_rgb.numpy())],
        "render_depth": [bool(value) for value in np.asarray(rc.render_depth.numpy())],
        "render_seg": [bool(value) for value in np.asarray(rc.render_seg.numpy())],
        "total_rays": total,
        "rgb_pixels": int(sum(w * h for (w, h), enabled in zip(cam_res, rc.render_rgb.numpy()) if bool(enabled))),
        "samples_per_pixel": spp,
        "samples": spp * spp,
        "znear_bits": bits_f32_scalar(np.float32(rc.znear)),
        "zfar_bits": bits_f32_scalar(np.float32(rc.zfar)),
        "count": int(directions.shape[0]),
        "sha256": canonical_sha256(directions, offsets),
        "samples_entries": sample_entries(directions, offsets, cam_res, spp, total),
    }
