"""QA judge: AA semantics from QA's own per-sample renders.

Product is rendered four ways in one harness process:
  * spp=1 context with sample s's own 120,000 rays (s = 0..3)  -> rgb_sample*, depth_sample*, seg_*
  * spp=2 (nsamples=4) context with the full 480,000 ray buffer -> rgb_aa / depth_aa / seg_aa

Independent judgements:
  1. depth/seg are single-sample outputs of the FIRST pass:
     depth_aa == depth_sample0 and seg_aa == seg_sample0 bitwise, while sample0 vs sample1
     actually differ (discriminating), and depth_aa != last-sample depth where they differ.
  2. RGB is the average of the four per-sample colours, quantised upstream-style
     (accumulate color*255, c = accum/(4), clamp[0,255], int truncation):
     per channel, |aa_byte - floor(mean(per-sample byte))| <= 1.
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
SPLIT = ROOT / "target/qa-s7b3/out/split"


def read_u32(name: str, shape=None) -> np.ndarray:
    values = np.frombuffer((SPLIT / name).read_bytes(), dtype=np.uint32)
    return values.reshape(shape) if shape else values


def read_f32(name: str) -> np.ndarray:
    return np.frombuffer((SPLIT / name).read_bytes(), dtype=np.uint32)


def read_seg(name: str) -> np.ndarray:
    return np.frombuffer((SPLIT / name).read_bytes(), dtype=np.int32).reshape(-1, 2)


def channels(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)),
            (values >> np.uint32(8) & np.uint32(0xFF)),
            (values & np.uint32(0xFF)),
        ],
        axis=-1,
    ).astype(np.int64)


def main() -> int:
    report = json.loads((SPLIT / "sample_split.json").read_text(encoding="utf-8"))
    nsamples = int(report["nsamples"])
    depth_aa = read_f32("depth_aa.bin")
    seg_aa = read_seg("seg_aa.bin")
    rgb_aa = read_u32("rgb_aa.bin")
    sample_depth = [read_f32(f"depth_sample{s}.bin") for s in range(nsamples)]
    sample_seg = [read_seg(f"seg_sample{s}.bin") for s in range(nsamples)]
    sample_rgb = [read_u32(f"rgb_sample{s}.bin") for s in range(nsamples)]
    result = {
        "nsamples": nsamples,
        "total_rays": report["total_rays"],
        "aa_accum_width": report["aa_accum_width"],
        "rgb_pixels": report["rgb_pixels"],
        "depth_first_sample_equal": bool(np.array_equal(depth_aa, sample_depth[0])),
        "seg_first_sample_equal": bool(np.array_equal(seg_aa, sample_seg[0])),
        "depth_last_sample_equal": bool(np.array_equal(depth_aa, sample_depth[-1])),
        "seg_last_sample_equal": bool(np.array_equal(seg_aa, sample_seg[-1])),
        "depth_sample0_vs_sample1_changed": int(
            np.count_nonzero(sample_depth[0] != sample_depth[1])
        ),
        "seg_sample0_vs_sample1_changed": int(
            np.count_nonzero(np.any(sample_seg[0] != sample_seg[1], axis=1))
        ),
        "depth_aa_vs_sample1_changed": int(np.count_nonzero(depth_aa != sample_depth[1])),
        "seg_aa_vs_sample1_changed": int(
            np.count_nonzero(np.any(seg_aa != sample_seg[1], axis=1))
        ),
    }
    # RGB average / quantisation check
    stack = np.stack([channels(rgb) for rgb in sample_rgb])  # (nsamples, pixels, 3) bytes
    mean = stack.astype(np.float64).mean(axis=0)
    floor_mean = np.floor(mean).astype(np.int64)
    aa = channels(rgb_aa)
    diff = np.abs(aa - floor_mean)
    per_channel_violations = [
        int(np.count_nonzero(diff[:, channel] > 1)) for channel in range(3)
    ]
    result["rgb_average"] = {
        "rule": "|aa_byte - floor(mean(per_sample_byte))| <= 1 per channel",
        "violations_per_channel": per_channel_violations,
        "max_abs_deviation": int(diff.max()),
        "pixels_with_any_violation": int(np.count_nonzero(np.any(diff > 1, axis=1))),
        "mean_byte_mean": [float(mean[:, c].mean()) for c in range(3)],
    }
    # alpha byte must stay 255
    result["aa_alpha_all_255"] = bool(np.all((rgb_aa >> np.uint32(24)) == np.uint32(255)))
    out = ROOT / "target/qa-s7b3/out/judge_aa.json"
    out.write_text(json.dumps(result, indent=1), encoding="utf-8")
    print(json.dumps(result, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
