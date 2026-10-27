"""QA independent recompute for R08-S1 (python/numpy cross-implementation).

Reads the S7b frozen references (authoring-time pinned-upstream GPU captures) and:
  1) reconciles the declared layout/pixel counts;
  2) rebuilds the whole buffers in adr order;
  3) computes the expected per-camera outputs with an independent numpy float32
     implementation of the upstream formulas (unpack_rgb / depth clamp / seg copy);
  4) recomputes the cross-check counts claimed by the slice;
  5) dumps the expected values as raw little-endian binaries so a separate Rust
     harness can bit-compare the product outputs against this implementation.

Read-only w.r.t. the repository; writes only under target/qa-r08/.
"""
import json
import struct
from pathlib import Path

import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-r08"
EXPECTED = OUT / "expected"
RGB_REF = ROOT / "fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json"
DEPTH_SEG_REF = ROOT / "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json"

SCALES = [1.0, 0.1, 100.0]


def f32(value: float) -> np.float32:
    return np.float32(value)


def inv255_bits() -> str:
    return f"0x{np.float32(1.0 / 255.0).view(np.uint32):08X}"


def unpack_rgb(packed: np.ndarray) -> np.ndarray:
    """Independent impl of upstream unpack_rgb_kernel -> (n, 3) f32 [r, g, b]."""
    value = packed.astype(np.uint32)
    inv = np.float32(np.float64(1.0) / np.float64(255.0))
    b = (value & np.uint32(0xFF)).astype(np.float32) * inv
    g = ((value >> np.uint32(8)) & np.uint32(0xFF)).astype(np.float32) * inv
    r = ((value >> np.uint32(16)) & np.uint32(0xFF)).astype(np.float32) * inv
    return np.stack([r, g, b], axis=1)


def fmaxf(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """CUDA fmaxf: a NaN operand is ignored (returns the other operand)."""
    return np.where(np.isnan(a), b, np.where(np.isnan(b), a, np.maximum(a, b)))


def fminf(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    """CUDA fminf: a NaN operand is ignored (returns the other operand)."""
    return np.where(np.isnan(a), b, np.where(np.isnan(b), a, np.minimum(a, b)))


def clamp_unit(value: np.ndarray) -> np.ndarray:
    """Upstream wp.clamp(x, 0, 1) = fminf(fmaxf(x, 0), 1)."""
    zero = np.float32(0.0)
    one = np.float32(1.0)
    return fminf(fmaxf(value, zero), one)


def main():
    EXPECTED.mkdir(parents=True, exist_ok=True)
    rgb = json.loads(RGB_REF.read_text(encoding="utf-8"))
    depth_seg = json.loads(DEPTH_SEG_REF.read_text(encoding="utf-8"))

    summary = {"checks": [], "counts": {}, "dumps": {}}

    def check(cond, message):
        summary["checks"].append({"ok": bool(cond), "message": message})
        if not cond:
            print("PROBLEM:", message)

    # ---- declared lookup tables (fixtures) ----
    cam_res = rgb["resolved"]["cam_res"]
    check(cam_res == depth_seg["resolved"]["cam_res"], f"cam_res equal: {cam_res}")
    check(rgb["resolved"]["total_pixels"] == 120_000, "total_pixels 120000")
    check(rgb["resolved"]["rgb_pixels"] == 120_000, "rgb_pixels 120000")
    check(rgb["resolved"]["rgb_adr"] == [0, 76_800], f"rgb_adr {rgb['resolved']['rgb_adr']}")
    check(rgb["resolved"]["depth_adr"] == [0, 76_800], "depth_adr [0,76800]")
    check(rgb["resolved"]["seg_adr"] == [0, 76_800], "seg_adr [0,76800]")
    check(depth_seg["resolved"]["total_rays"] == 120_000, "total_rays 120000")
    check(rgb["encoding"]["pixel_order"].startswith("row-major"), "pixel order row-major")
    check(rgb["source"]["upstream"] == "71da24d956378a87a703b6e1442b13aec0c4ac29", "upstream pin")
    check(depth_seg["source"]["upstream"] == rgb["source"]["upstream"], "depth/seg same pin")
    check(rgb["self_check"]["unpack_matches_get_rgb"] is True, "unpack == get_rgb self check")

    # ---- per-camera reference arrays ----
    resolutions = [(camera["width"], camera["height"]) for camera in rgb["cameras"]]
    check(resolutions == [tuple(r) for r in cam_res], f"camera resolutions {resolutions}")
    check(rgb["self_check"]["depth_seg_recipe"]["all_equal"] is True, "depth/seg recipe all_equal")

    cameras = []
    for index, camera in enumerate(rgb["cameras"]):
        resolution = (camera["width"], camera["height"])
        pixels = resolution[0] * resolution[1]
        check(camera["pixels"] == pixels, f"camera {index} pixels {camera['pixels']} == {pixels}")
        check(camera["rgb_adr"] == sum(r[0] * r[1] for r in resolutions[:index]),
              f"camera {index} rgb_adr contiguous")
        packed = np.array(camera["rgb_bits_packed"], dtype=np.uint32)
        check(packed.size == pixels, f"camera {index} packed len {packed.size} == {pixels}")
        cameras.append({
            "resolution": resolution,
            "pixels": pixels,
            "packed": packed,
            "adr": camera["rgb_adr"],
        })
    for index, camera in enumerate(depth_seg["cameras"]):
        resolution = resolutions[index]
        pixels = resolution[0] * resolution[1]
        check(camera["width"], f"depth/seg camera {index} width")
        bits = np.array(camera["depth_bits"], dtype=np.uint32)
        pairs = np.array(camera["seg_pairs"], dtype=np.int32)
        check(bits.size == pixels, f"camera {index} depth len {bits.size} == {pixels}")
        check(pairs.size == pixels * 2, f"camera {index} seg len {pairs.size} == {pixels * 2}")
        check(camera["depth_adr"] == cameras[index]["adr"], f"camera {index} depth_adr matches")
        check(camera["seg_adr"] == cameras[index]["adr"], f"camera {index} seg_adr matches")
        cameras[index]["depth_bits"] = bits
        cameras[index]["seg_pairs"] = pairs.reshape(-1, 2)

    total_pixels = sum(c["pixels"] for c in cameras)
    check(total_pixels == 120_000, f"total pixels {total_pixels}")

    # ---- expected per-camera values (independent implementation) ----
    counts = {"packed": 0, "unpacked": 0, "depth_raw": 0, "depth_scaled": 0, "seg": 0}
    clamp_high = {}
    raw_max = {}
    seg_background = {}
    zero_depth = {}
    minus_zero_depth = {}
    for index, camera in enumerate(cameras):
        packed = camera["packed"]
        expected_rgb = unpack_rgb(packed)
        counts["packed"] += packed.size
        counts["unpacked"] += packed.size * 3
        expected_rgb.astype("<f4").tofile(EXPECTED / f"rgb_c{index}.f32.bin")
        summary["dumps"][f"rgb_c{index}"] = int(packed.size * 3)

        raw = camera["depth_bits"].view(np.float32)
        counts["depth_raw"] += raw.size
        camera["depth_bits"].astype("<u4").tofile(EXPECTED / f"depth_raw_c{index}.bits.bin")
        summary["dumps"][f"depth_raw_c{index}"] = int(raw.size)
        check(not np.isnan(raw).any(), f"camera {index} reference depth has no NaN")

        for scale in SCALES:
            # division first, then clamp, both in float32
            scaled = clamp_unit(raw / np.float32(scale))
            counts["depth_scaled"] += scaled.size
            tag = f"s{str(scale).replace('.', '_')}"
            scaled.astype("<f4").tofile(EXPECTED / f"depth_{tag}_c{index}.f32.bin")
            summary["dumps"][f"depth_{tag}_c{index}"] = int(scaled.size)
            if scale == 1.0:
                clamp_high[index] = int(np.isclose(scaled, np.float32(1.0), rtol=0, atol=0).sum())
                raw_max[index] = float(raw.max())

        seg = camera["seg_pairs"]
        counts["seg"] += seg.shape[0]
        seg.astype("<i4").tofile(EXPECTED / f"seg_c{index}.i32x2.bin")
        summary["dumps"][f"seg_c{index}"] = int(seg.size)
        background = int(((seg[:, 0] == -1) & (seg[:, 1] == -1)).sum())
        seg_background[index] = background
        zero_depth[index] = int((camera["depth_bits"] == 0).sum())
        minus_zero_depth[index] = int((camera["depth_bits"] == np.uint32(0x80000000)).sum())
        check(background == zero_depth[index],
              f"camera {index} background {background} == depth zero {zero_depth[index]}")

    # ---- claimed counts (from the slice receipt) ----
    claimed = {
        "rgb_packed": 120_000, "unpacked": 360_000, "depth_raw": 120_000,
        "depth_scaled": 360_000, "seg": 120_000,
        "background": {0: 16_995, 1: 27_498}, "clamp_high": {0: 12_890, 1: 15_702},
        "raw_max": {0: 1.6825796365737915, 1: 3.703162670135498},
    }
    check(counts["packed"] == claimed["rgb_packed"], f"packed {counts['packed']}")
    check(counts["unpacked"] == claimed["unpacked"], f"unpacked {counts['unpacked']}")
    check(counts["depth_raw"] == claimed["depth_raw"], f"depth_raw {counts['depth_raw']}")
    check(counts["depth_scaled"] == claimed["depth_scaled"], f"depth_scaled {counts['depth_scaled']}")
    check(counts["seg"] == claimed["seg"], f"seg {counts['seg']}")
    check(seg_background == claimed["background"], f"background {seg_background}")
    check(clamp_high == claimed["clamp_high"], f"clamp_high {clamp_high}")
    for index in (0, 1):
        check(abs(raw_max[index] - claimed["raw_max"][index]) <= 0.0, f"raw max {index} {raw_max[index]}")

    # ---- INV_255 identity ----
    value = struct.unpack("<I", struct.pack("<f", 1.0 / 255.0))[0]
    check(value == 0x3B80_8081, f"float32(1/255) bits 0x{value:08X}")

    summary["fixture"] = {
        "cam_res": cam_res,
        "resolutions": resolutions,
        "total_pixels": total_pixels,
        "rgb_adr": rgb["resolved"]["rgb_adr"],
        "depth_adr": rgb["resolved"]["depth_adr"],
        "seg_adr": rgb["resolved"]["seg_adr"],
        "upstream": rgb["source"]["upstream"],
        "rgb_sha256_input": None,
    }
    summary["counts"] = counts
    summary["seg_background"] = seg_background
    summary["depth_zero_plus0"] = zero_depth
    summary["depth_zero_minus0"] = minus_zero_depth
    summary["clamp_high_at_scale_1"] = clamp_high
    summary["depth_raw_max"] = raw_max
    summary["inv_255_bits"] = inv255_bits()
    summary["scales"] = SCALES
    problems = [c for c in summary["checks"] if not c["ok"]]
    summary["result"] = "PASS" if not problems else "FAIL"
    (OUT / "python_expected.json").write_bytes(
        (json.dumps(summary, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print("checks:", len(summary["checks"]), "problems:", len(problems), "result:", summary["result"])
    print("counts:", counts)
    print("background:", seg_background, "zero:", zero_depth, "minus0:", minus_zero_depth)
    print("clamp_high:", clamp_high, "raw_max:", raw_max)
    print("INV_255 bits:", summary["inv_255_bits"])
    print("dumps:", sum(summary["dumps"].values()), "values")


if __name__ == "__main__":
    main()
