"""S7b2 authoring-only: derive the RGB reference fixture + light data-pose check.

Input (read-only, frozen by S0b):
  fixtures/render-bvh/scene-a/reference/rgb.npz   (`get_rgb` 解包后的线性 RGB，float32 [0,1])
Output:
  fixtures/render-bvh/scene-a/s7b/rgb.json
    - cameras[].rgb_bits_packed：上游 `rgb_data` 的 packed u32（ABGR，a=255，`int()` 截断），
      row-major；测试侧按 `byte/255.0`（f32 乘 1/255）解包后比较「每通道 ≤1 LSB」。
    - light_data_check：pinned 3.15 `mjwarp.forward` 产出的 data `light_xpos`/`light_xdir`
      f32 位型（scene-a 三盏灯 `light_mode=0` 固定灯），用于断言 host 侧推导位等。
  provenance：npz/mjb sha256、上游 commit、脚本 sha256、生成时间；两次 forward 逐位一致。

不做任何重新渲染、不改 S0b/S2–S6 夹具。

Run (pinned venv, repo root):
  target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/convert_rgb.py
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import time

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCENE = ROOT / "fixtures/render-bvh/scene-a"
S7B = SCENE / "s7b"
REFERENCE = SCENE / "reference"
OUTPUT = S7B / "rgb.json"
MJB = SCENE / "scene-315.mjb"
MODEL_312 = SCENE / "model-312.json"
RGB_NPZ = REFERENCE / "rgb.npz"
TASK_ID = "2026-10-25-render-bvh-s7b2-shading"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED_SHA256 = {
    "scene-315.mjb": "8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4",
    "model-312.json": "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
    "rgb.npz": "9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c",
}
EXPECTED_VERSIONS = {"warp": "1.15.0", "mujoco": "3.15.0", "mujoco_warp": "3.15.0", "numpy": "2.5.4"}
GPU_NAME = "NVIDIA GeForce RTX 4070 Ti SUPER"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256(path: pathlib.Path) -> str:
    return sha256_bytes(path.read_bytes())


def script_sha256() -> str:
    return sha256_bytes(pathlib.Path(__file__).read_bytes().replace(b"\r\n", b"\n"))


def package_version(*names: str) -> str:
    import importlib.metadata  # noqa: PLC0415

    for name in names:
        try:
            return importlib.metadata.version(name)
        except importlib.metadata.PackageNotFoundError:
            continue
    raise SystemExit(f"cannot resolve package version for {names}")


def unpack_bytes(array: np.ndarray) -> np.ndarray:
    """npz 的 f32 `k/255` → 原始字节（0..255）。"""
    bytes_value = np.rint(np.asarray(array, dtype=np.float64) * 255.0).astype(np.int64)
    if bytes_value.min() < 0 or bytes_value.max() > 255:
        raise SystemExit(f"rgb byte out of range: {bytes_value.min()}..{bytes_value.max()}")
    return bytes_value.astype(np.uint32)


def pack_abgr(byte_r: np.ndarray, byte_g: np.ndarray, byte_b: np.ndarray) -> np.ndarray:
    """上游 `pack_rgba_to_uint32(r,g,b,255)`：`(a<<24)|(r<<16)|(g<<8)|b`，`int()` 截断。"""
    alpha = np.uint32(255)
    return (
        (alpha << np.uint32(24))
        | (byte_r.astype(np.uint32) << np.uint32(16))
        | (byte_g.astype(np.uint32) << np.uint32(8))
        | byte_b.astype(np.uint32)
    )


def unpack_float(packed: np.ndarray) -> np.ndarray:
    """上游 `unpack_rgb_kernel`：`float32(byte) * float32(1/255)`。"""
    one_over_255 = np.float32(1.0) / np.float32(255.0)
    values = packed.astype(np.uint32)
    red = np.float32(values >> np.uint32(16) & np.uint32(0xFF)) * one_over_255
    green = np.float32(values >> np.uint32(8) & np.uint32(0xFF)) * one_over_255
    blue = np.float32(values & np.uint32(0xFF)) * one_over_255
    return np.stack([red, green, blue], axis=-1)


def camera_documents(npz: np.lib.npyio.NpzFile) -> list[dict]:
    cameras = []
    for index, key in enumerate(("rgb_cam0", "rgb_cam1")):
        array = np.asarray(npz[key], dtype=np.float32)
        if array.ndim != 3 or array.shape[2] != 3:
            raise SystemExit(f"{key}: shape {array.shape} != (h, w, 3)")
        height, width, _ = array.shape
        bytes_rgb = unpack_bytes(array)
        packed = pack_abgr(bytes_rgb[..., 0], bytes_rgb[..., 1], bytes_rgb[..., 2]).reshape(-1)
        if packed.size != width * height:
            raise SystemExit(f"{key}: pixel count {packed.size} != {width * height}")
        # round-trip 自查：packed → 解包 f32 必须与 npz 原始位型逐位一致。
        rebuilt = unpack_float(packed).reshape(height, width, 3)
        if not np.array_equal(rebuilt.view(np.uint32), array.view(np.uint32)):
            raise SystemExit(f"{key}: packed round-trip 与 npz 位型不一致")
        unique = np.unique(packed)
        cameras.append(
            {
                "index": index,
                "width": width,
                "height": height,
                "pixels": width * height,
                "rgb_bits_packed": [int(value) for value in packed.tolist()],
                "unique_pixels": int(unique.size),
                "zero_pixels": int(np.count_nonzero(packed == np.uint32(0xFF000000))),
            }
        )
    return cameras


def light_data_check() -> list[dict]:
    """pinned 3.15 `mjwarp.forward` 的 data light_xpos/xdir f32 位型（scene-a 固定灯）。"""
    import mujoco  # noqa: PLC0415
    import mujoco_warp as mjw  # noqa: PLC0415
    import warp as wp  # noqa: PLC0415

    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    model = mjw.put_model(mjm)
    data = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(model, data)
    wp.synchronize()
    xpos = np.ascontiguousarray(np.asarray(data.light_xpos.numpy())[0], dtype=np.float32)
    xdir = np.ascontiguousarray(np.asarray(data.light_xdir.numpy())[0], dtype=np.float32)
    modes = [int(value) for value in np.asarray(mjm.light_mode)]
    bodyids = [int(value) for value in np.asarray(mjm.light_bodyid)]
    if any(mode != 0 for mode in modes):
        raise SystemExit(f"light_mode {modes} != all 0（本片只支持固定灯推导）")
    if any(body != 0 for body in bodyids):
        raise SystemExit(f"light_bodyid {bodyids} != all 0（需要 data body 位姿）")
    checks = []
    for index in range(xpos.shape[0]):
        checks.append(
            {
                "index": index,
                "mode": modes[index],
                "bodyid": bodyids[index],
                "xpos_bits": [int(value) for value in xpos[index].view(np.uint32)],
                "xdir_bits": [int(value) for value in xdir[index].view(np.uint32)],
            }
        )
    return checks


def main() -> int:
    for name, expected in EXPECTED_SHA256.items():
        path = {"scene-315.mjb": MJB, "model-312.json": MODEL_312, "rgb.npz": RGB_NPZ}[name]
        actual = sha256(path)
        if actual != expected:
            print(f"FAIL: {name} sha256 {actual} != {expected}", file=sys.stderr)
            return 1
    versions = {
        "warp": None,
        "mujoco": None,
        "mujoco_warp": None,
        "numpy": None,
    }
    import mujoco  # noqa: PLC0415
    import mujoco_warp as mjw  # noqa: PLC0415
    import warp as wp  # noqa: PLC0415

    versions = {
        "warp": wp.__version__,
        "mujoco": mujoco.__version__,
        "mujoco_warp": getattr(mjw, "__version__", package_version("mujoco-warp", "mujoco_warp")),
        "numpy": np.__version__,
    }
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

    npz = np.load(RGB_NPZ)
    cameras = camera_documents(npz)
    checks = light_data_check()
    second = light_data_check()
    if checks != second:
        print("FAIL: two light forward passes are not bitwise identical", file=sys.stderr)
        return 1

    document = {
        "schema": 1,
        "task_id": TASK_ID,
        "kind": "render-bvh-s7b2-rgb-reference",
        "authoring_only": True,
        "product_python_dependency": False,
        "encoding": {
            "rgb": "packed u32 ABGR-A8：a=255；通道 = (a<<24)|(r<<16)|(g<<8)|b，"
                   "r/g/b = int(clamp(channel,0,1)*255)（截断，不是四舍五入）；"
                   "解包 = float32(byte) * float32(1/255)，与上游 `unpack_rgb_kernel` 一致",
            "pixel_order": "row-major (y outer, x fastest)",
            "tolerance": "解包后每通道 ≤1 LSB（1/255）",
            "light_data": "data light_xpos/light_xdir 的 f32 位型（u32 十进制）",
        },
        "source": {
            "upstream": UPSTREAM,
            "sim_reference": "S0b PARAMS_28（use_fast_math=true）单帧 get_rgb 解包结果",
            "rgb_npz_sha256": EXPECTED_SHA256["rgb.npz"],
            "scene_315_mjb_sha256": EXPECTED_SHA256["scene-315.mjb"],
            "model_312_sha256": EXPECTED_SHA256["model-312.json"],
            "convert_script_sha256": script_sha256(),
            "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        },
        "resolved": {
            "cam_res": [[camera["width"], camera["height"]] for camera in cameras],
            "total_pixels": sum(int(camera["pixels"]) for camera in cameras),
        },
        "cameras": cameras,
        "light_data_check": checks,
        "determinism": {"light_forward_repeat": 2, "bitwise_identical": True},
        "environment": {
            "versions": versions,
            "gpu": gpu.split(",")[0].strip(),
            "gpu_driver": gpu.split(",")[1].strip(),
            "python_executable": sys.executable,
        },
    }
    text = json.dumps(document, ensure_ascii=False, separators=(",", ":")) + "\n"
    parsed = json.loads(text)
    for index, camera in enumerate(parsed["cameras"]):
        array = np.asarray(npz[f"rgb_cam{index}"], dtype=np.float32)
        packed = np.asarray(camera["rgb_bits_packed"], dtype=np.uint32)
        if not np.array_equal(
            unpack_float(packed).reshape(array.shape).view(np.uint32), array.view(np.uint32)
        ):
            print(f"FAIL: round-trip cam{index}", file=sys.stderr)
            return 1
        if len(camera["rgb_bits_packed"]) != camera["width"] * camera["height"]:
            print(f"FAIL: length cam{index}", file=sys.stderr)
            return 1
    S7B.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(text.encode("utf-8"))
    written = OUTPUT.read_bytes()
    if sha256_bytes(written) != sha256_bytes(text.encode("utf-8")):
        print("FAIL: written bytes differ from serialized text", file=sys.stderr)
        return 1
    print(
        f"wrote {OUTPUT.relative_to(ROOT)}: {len(written)} bytes sha256={sha256_bytes(written)}, "
        f"total_pixels={sum(int(c['pixels']) for c in cameras)}, lights={len(checks)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
