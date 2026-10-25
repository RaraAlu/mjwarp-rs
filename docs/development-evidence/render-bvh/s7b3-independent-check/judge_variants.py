"""QA judge: S7b3 flag variants and AA vs QA's own pinned-3.15 re-captures.

Compares product dumps (rgb_<variant>.bin / depth / seg) against QA reference captures
(target/qa-s7b3/ref/ref_<variant>.json) and, secondarily, against the delivered fixtures.
Also computes the discriminating-vs-base statistics independently (changed pixels per camera,
per-seg-geom distribution, depth/seg changes).
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s7b3/out"
REF = ROOT / "target/qa-s7b3/ref"
FLAGS = ROOT / "fixtures/render-bvh/scene-a/s7b/flags"

VARIANTS = [
    "base",
    "textures_off",
    "vertex_normals_off",
    "specular_off",
    "emission_off",
    "culling_off",
    "aa2",
]


def packed_bytes(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)),
            (values >> np.uint32(8) & np.uint32(0xFF)),
            (values & np.uint32(0xFF)),
        ],
        axis=-1,
    ).astype(np.int16)


def rgb_stats(actual: np.ndarray, expected: np.ndarray) -> dict:
    diff = np.abs(packed_bytes(actual) - packed_bytes(expected)).max(axis=1)
    return {
        "pixels": int(diff.size),
        "mismatched": int(np.count_nonzero(diff > 0)),
        "beyond_1_lsb": int(np.count_nonzero(diff > 1)),
        "max_byte_diff": int(diff.max()),
        "histogram_0_1_2_3_gt3": [int(np.count_nonzero(diff == v)) for v in (0, 1, 2, 3)]
        + [int(np.count_nonzero(diff > 3))],
    }


def ulp(a: np.ndarray, b: np.ndarray) -> np.ndarray:
    left = np.ascontiguousarray(a, dtype=np.int32).astype(np.int64)
    right = np.ascontiguousarray(b, dtype=np.int32).astype(np.int64)
    left = np.where(left < 0, np.int64(-2147483648) - left, left)
    right = np.where(right < 0, np.int64(-2147483648) - right, right)
    return np.abs(left - right)


def read_product(tag: str):
    rgb = np.frombuffer((OUT / f"rgb_{tag}.bin").read_bytes(), dtype=np.uint32)
    depth = np.frombuffer((OUT / f"depth_{tag}.bin").read_bytes(), dtype=np.uint32)
    seg = np.frombuffer((OUT / f"seg_{tag}.bin").read_bytes(), dtype=np.int32).reshape(-1, 2)
    return rgb, depth, seg


def camera_slices(reference: dict):
    starts = []
    offset = 0
    for camera in reference["cameras"]:
        starts.append(offset)
        offset += camera["pixels"]
    return starts


def compare_to_reference(tag: str) -> dict:
    ref = json.loads((REF / f"ref_{tag}.json").read_text(encoding="utf-8"))
    rgb, depth, seg = read_product(tag)
    starts = camera_slices(ref)
    cameras = []
    for index, camera in enumerate(ref["cameras"]):
        start = starts[index]
        pixels = camera["pixels"]
        expected_rgb = np.asarray(camera["rgb_bits"], dtype=np.uint32)
        expected_depth = np.asarray(camera["depth_bits"], dtype=np.uint32)
        expected_seg = np.asarray(camera["seg_pairs"], dtype=np.int32).reshape(-1, 2)
        actual_rgb = rgb[start : start + pixels]
        actual_depth = depth[start : start + pixels]
        actual_seg = seg[start : start + pixels]
        depth_ulp = ulp(actual_depth, expected_depth)
        cameras.append(
            {
                "index": index,
                "pixels": pixels,
                "rgb": rgb_stats(actual_rgb, expected_rgb),
                "depth_bitwise_equal": bool(np.array_equal(actual_depth, expected_depth)),
                "depth_max_ulp": int(depth_ulp.max()),
                "seg_mismatched": int(np.count_nonzero(np.any(actual_seg != expected_seg, axis=1))),
            }
        )
    return {"variant": tag, "reference": f"qa_ref_{tag}.json", "cameras": cameras}


def compare_to_delivered(tag: str) -> dict | None:
    path = FLAGS / f"{tag}_rgb.json"
    if not path.exists():
        return None
    delivered = json.loads(path.read_text(encoding="utf-8"))
    rgb, depth, seg = read_product(tag)
    starts = camera_slices(delivered)
    cameras = []
    for index, camera in enumerate(delivered["cameras"]):
        start = starts[index]
        pixels = camera["pixels"]
        expected_rgb = np.asarray(camera["rgb_bits_packed"], dtype=np.uint32)
        cameras.append(
            {
                "index": index,
                "rgb": rgb_stats(rgb[start : start + pixels], expected_rgb),
            }
        )
    return {"variant": tag, "cameras": cameras}


def base_relative(tag: str) -> dict:
    base_rgb, base_depth, base_seg = read_product("base")
    rgb, depth, seg = read_product(tag)
    # per-camera and per-seg-geom breakdown cam1 (mesh/primitive distribution)
    result = {"variant": tag, "cameras": []}
    starts = [0, 76800]
    for index, start in enumerate(starts):
        pixels = 76800 if index == 0 else 43200
        diff = np.abs(packed_bytes(rgb[start : start + pixels]) - packed_bytes(base_rgb[start : start + pixels])).max(axis=1)
        changed = diff > 0
        seg_cam = base_seg[start : start + pixels]
        by_geom: dict[int, int] = {}
        for geom in np.unique(seg_cam[changed][:, 0]):
            by_geom[int(geom)] = int(np.count_nonzero(changed & (seg_cam[:, 0] == geom)))
        depth_changed = int(np.count_nonzero(depth[start : start + pixels] != base_depth[start : start + pixels]))
        seg_changed = int(np.count_nonzero(np.any(seg[start : start + pixels] != base_seg[start : start + pixels], axis=1)))
        result["cameras"].append(
            {
                "index": index,
                "rgb_changed_pixels": int(np.count_nonzero(changed)),
                "rgb_changed_by_geom": by_geom,
                "depth_changed_pixels": depth_changed,
                "seg_changed_pixels": seg_changed,
            }
        )
    return result


def main() -> int:
    report = {"vs_qa_reference": [], "vs_delivered_fixture": [], "vs_base": []}
    for tag in VARIANTS:
        report["vs_qa_reference"].append(compare_to_reference(tag))
        delivered = compare_to_delivered(tag)
        if delivered:
            report["vs_delivered_fixture"].append(delivered)
        if tag != "base":
            report["vs_base"].append(base_relative(tag))
    out = OUT / "judge_variants.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    for entry in report["vs_qa_reference"]:
        print(
            entry["variant"],
            [
                {
                    "rgb_mismatched": camera["rgb"]["mismatched"],
                    "rgb_beyond": camera["rgb"]["beyond_1_lsb"],
                    "depth_equal": camera["depth_bitwise_equal"],
                    "depth_ulp": camera["depth_max_ulp"],
                    "seg_mismatch": camera["seg_mismatched"],
                }
                for camera in entry["cameras"]
            ],
        )
    print("--- vs base")
    for entry in report["vs_base"]:
        print(entry["variant"], entry["cameras"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
