"""QA judge: product dumps vs QA-decoded reference (and delivered fixtures).

Compares, per camera:
  * packed u32 equality (ABGR-A8) + byte-diff histogram [0,1,2,3,>3],
  * unpacked f32 = float32(byte) * float32(1/255) and max per-channel LSB diff,
  * depth f32 bits equality + ULP histogram,
  * seg pairs equality,
  * cross-recipe record vs the S0b fast-math fixture.
"""

from __future__ import annotations

import json
import pathlib
import struct
import sys

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s7b2/out"
REF = ROOT / "target/qa-s7b2/ref"
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"


def read_bin(path: pathlib.Path, dtype) -> np.ndarray:
    return np.frombuffer(path.read_bytes(), dtype=dtype)


def packed_bytes(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)).astype(np.uint8),
            (values >> np.uint32(8) & np.uint32(0xFF)).astype(np.uint8),
            (values & np.uint32(0xFF)).astype(np.uint8),
        ],
        axis=-1,
    )


def unpack(packed: np.ndarray) -> np.ndarray:
    return packed_bytes(packed).astype(np.float32) * np.float32(1.0 / 255.0)


def byte_histogram(actual: np.ndarray, expected: np.ndarray) -> dict:
    left = packed_bytes(actual).astype(np.int16).reshape(-1, 3)
    right = packed_bytes(expected).astype(np.int16).reshape(-1, 3)
    diff = np.abs(left - right)
    max_diff = diff.max(axis=1)
    return {
        "pixels": int(max_diff.size),
        "mismatched_pixels": int(np.count_nonzero(max_diff > 0)),
        "beyond_1_lsb": int(np.count_nonzero(max_diff > 1)),
        "max_byte_diff": int(max_diff.max()) if max_diff.size else 0,
        "histogram_0_1_2_3_gt3": [
            int(np.count_nonzero(max_diff == value)) for value in (0, 1, 2, 3)
        ]
        + [int(np.count_nonzero(max_diff > 3))],
        "max_unpacked_lsb_diff": float(diff.max()) if diff.size else 0.0,
    }


def ulp_distance(left_bits: np.ndarray, right_bits: np.ndarray) -> np.ndarray:
    left = np.ascontiguousarray(left_bits, dtype=np.int32).astype(np.int64)
    right = np.ascontiguousarray(right_bits, dtype=np.int32).astype(np.int64)
    left = np.where(left < 0, np.int64(-2147483648) - left, left)
    right = np.where(right < 0, np.int64(-2147483648) - right, right)
    return np.abs(left - right)


def compare(tag: str, ref_doc: dict, product_tag: str | None = None) -> dict:
    product_tag = product_tag or tag
    rgb = read_bin(OUT / f"rgb_{product_tag}.bin", np.uint32)
    depth = read_bin(OUT / f"depth_{product_tag}.bin", np.uint32)
    seg = read_bin(OUT / f"seg_{product_tag}.bin", np.int32).reshape(-1, 2)
    s0b = json.loads((S7B / "rgb.json").read_text(encoding="utf-8"))
    report = {"tag": tag, "product_tag": product_tag, "cameras": []}
    for index, camera in enumerate(ref_doc["cameras"]):
        start = int(sum(item["pixels"] for item in ref_doc["cameras"][:index]))
        expected_rgb = np.asarray(camera["rgb_bits"], dtype=np.uint32)
        actual_rgb = np.ascontiguousarray(rgb[start : start + camera["pixels"]])
        expected_depth = np.asarray(camera["depth_bits"], dtype=np.uint32)
        actual_depth = np.ascontiguousarray(depth[start : start + camera["pixels"]])
        expected_seg = np.asarray(camera["seg_pairs"], dtype=np.int32).reshape(-1, 2)
        actual_seg = np.ascontiguousarray(seg[start : start + camera["pixels"]])
        ulp = ulp_distance(actual_depth, expected_depth)
        hist = [int(np.count_nonzero(ulp == value)) for value in (0, 1, 2, 3, 4)]
        hist.append(int(np.count_nonzero(ulp > 4)))
        # cross recipe record vs S0b fixture
        s0b_expected = np.asarray(s0b["cameras"][index]["rgb_bits_packed"], dtype=np.uint32)
        entry = {
            "index": index,
            "pixels": int(camera["pixels"]),
            "rgb": byte_histogram(actual_rgb, expected_rgb),
            "rgb_bitwise_equal": bool(np.array_equal(actual_rgb, expected_rgb)),
            "depth_bitwise_equal": bool(np.array_equal(actual_depth, expected_depth)),
            "depth_max_ulp": int(ulp.max()) if ulp.size else 0,
            "depth_ulp_histogram_0_1_2_3_4_gt4": hist,
            "seg_bitwise_equal": bool(np.array_equal(actual_seg, expected_seg)),
            "seg_mismatched": int(np.count_nonzero(np.any(actual_seg != expected_seg, axis=1))),
            "cross_recipe_vs_s0b": byte_histogram(actual_rgb, s0b_expected),
        }
        report["cameras"].append(entry)
    report["totals"] = {
        "rgb_mismatched_pixels": sum(c["rgb"]["mismatched_pixels"] for c in report["cameras"]),
        "rgb_beyond_1_lsb": sum(c["rgb"]["beyond_1_lsb"] for c in report["cameras"]),
        "depth_mismatched_pixels": sum(
            0 if c["depth_bitwise_equal"] else c["pixels"] - c["depth_ulp_histogram_0_1_2_3_4_gt4"][0]
            for c in report["cameras"]
        ),
        "seg_mismatched": sum(c["seg_mismatched"] for c in report["cameras"]),
        "unpacked_max_lsb": max(c["rgb"]["max_unpacked_lsb_diff"] for c in report["cameras"]),
    }
    return report


def main() -> int:
    tag = sys.argv[1] if len(sys.argv) > 1 else "base"
    product_tag = sys.argv[2] if len(sys.argv) > 2 else tag
    ref_path = REF / f"ref_{tag}.json"
    if tag in ("base", "base_qa"):
        # hard-number comparison: always against the delivered same-recipe fixture
        delivered = json.loads((S7B / "rgb_fast_math_false.json").read_text(encoding="utf-8"))
        depth_doc = json.loads(
            (S7B / "depth_seg_fast_math_false.json").read_text(encoding="utf-8")
        )
        cameras = []
        for rgb_camera, depth_camera in zip(delivered["cameras"], depth_doc["cameras"]):
            cameras.append(
                {
                    "index": rgb_camera["index"],
                    "pixels": rgb_camera["pixels"],
                    "rgb_bits": rgb_camera["rgb_bits_packed"],
                    "depth_bits": depth_camera["depth_bits"],
                    "seg_pairs": depth_camera["seg_pairs"],
                }
            )
        ref_doc = {"cameras": cameras}
        source = "delivered-fixture"
    else:
        ref_doc = json.loads(ref_path.read_text(encoding="utf-8"))
        source = f"QA-reference({ref_path.name})"
    report = compare(tag, ref_doc, product_tag)
    report["reference_source"] = source
    out = OUT / f"judge_{tag}.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report["totals"], indent=1))
    for camera in report["cameras"]:
        print(
            f"cam{camera['index']}: rgb_equal={camera['rgb_bitwise_equal']} "
            f"hist={camera['rgb']['histogram_0_1_2_3_gt3']} max_byte={camera['rgb']['max_byte_diff']} "
            f"depth_equal={camera['depth_bitwise_equal']} depth_max_ulp={camera['depth_max_ulp']} "
            f"seg_equal={camera['seg_bitwise_equal']} "
            f"cross_s0b={camera['cross_recipe_vs_s0b']['histogram_0_1_2_3_gt3']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
