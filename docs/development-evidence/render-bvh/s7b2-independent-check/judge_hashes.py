"""QA: independent recomputation of the delivered run hashes + source/PTX identity.

  * FNV-1a 64 of RGB words and of (depth bits, seg pairs) from QA dumps must equal the hashes
    printed in the delivered `gpu-run.log` (`rgb_hash`, `depth_seg_hash`);
  * `source_sha256` claim must equal sha256 of the concatenated kernel sources (include_str!
    concat), and the dumped PTX sha256 must equal the artifact/receipt ptx_sha256;
  * the S7b1 camera data poses used by the delivered fixture must equal the cam_xpos/cam_xmat
    bits that the pinned forward produced in the QA capture;
  * the delivered capture.log kernel-cache PTX facts are checked against the real cache file.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s7b2/out"
REF = ROOT / "target/qa-s7b2/ref"
EVIDENCE = ROOT / "docs/development-evidence/render-bvh/s7b2-shading"
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"
CACHE = "target/render-bvh/s7b2-fastmath-cache"

FILES = [
    "src/render/bvh/cuda/scene.cu",
    "src/render/kernel/cuda/render_math.cu",
    "src/render/kernel/cuda/render_common.cu",
    "src/render/kernel/cuda/render_shade.cu",
    "src/render/kernel/cuda/render_megakernel.cu",
]


def fnv1a_64(values, basis: int = 0xCBF29CE484222325) -> int:
    hash_value = basis
    for value in values:
        hash_value = ((hash_value ^ (value & 0xFFFFFFFFFFFFFFFF)) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return hash_value


def main() -> int:
    report: dict = {}
    rgb = pathlib.Path(OUT / "rgb_base.bin").read_bytes()
    depth = pathlib.Path(OUT / "depth_base.bin").read_bytes()
    seg = pathlib.Path(OUT / "seg_base.bin").read_bytes()
    rgb_words = [int.from_bytes(rgb[offset : offset + 4], "little") for offset in range(0, len(rgb), 4)]
    depth_bits = [int.from_bytes(depth[offset : offset + 4], "little") for offset in range(0, len(depth), 4)]
    seg_words = [int.from_bytes(seg[offset : offset + 4], "little") for offset in range(0, len(seg), 4)]
    report["qa_dump"] = {
        "rgb_words": len(rgb_words),
        "depth_bits": len(depth_bits),
        "seg_words": len(seg_words),
        # delivered test folds RGB from a zero seed (`fold(0u64, ...)`), depth/seg use the
        # FNV-1a offset basis (`output_hash`); both are recomputed here from QA dumps.
        "rgb_hash_fold_from_zero": f"{fnv1a_64(rgb_words, 0):016x}",
        "depth_seg_hash_fnv1a64": f"{fnv1a_64(depth_bits + seg_words):016x}",
    }
    gpu_log = (pathlib.Path(EVIDENCE) / "gpu-run.log").read_bytes().decode("utf-16")
    match = re.search(r"depth_seg_hash=([0-9a-f]+) rgb_hash=([0-9a-f]+)", gpu_log)
    report["delivered_log"] = {"depth_seg_hash": match.group(1), "rgb_hash": match.group(2)}
    report["hashes_match"] = (
        report["qa_dump"]["rgb_hash_fold_from_zero"] == match.group(2)
        and report["qa_dump"]["depth_seg_hash_fnv1a64"] == match.group(1)
    )

    concatenated = b"".join(pathlib.Path(ROOT / path).read_bytes() for path in FILES)
    source_sha = hashlib.sha256(concatenated).hexdigest()
    harness = json.loads((OUT / "harness_base.json").read_text(encoding="utf-8"))
    ptx_bytes = pathlib.Path(OUT / "megakernel.ptx").read_bytes()
    report["kernel_artifact"] = {
        "source_concat_bytes": len(concatenated),
        "source_concat_sha256": source_sha,
        "harness_source_sha256": harness["ptx"]["source_sha256"],
        "source_sha256_matches": source_sha == harness["ptx"]["source_sha256"],
        "ptx_bytes": len(ptx_bytes),
        "ptx_sha256": hashlib.sha256(ptx_bytes).hexdigest(),
        "harness_ptx_sha256": harness["ptx"]["ptx_sha256"],
        "ptx_sha256_matches": hashlib.sha256(ptx_bytes).hexdigest() == harness["ptx"]["ptx_sha256"],
        "entries": harness["ptx"]["entries"],
        "f64_lines": harness["ptx"]["f64_lines"],
    }

    # camera data poses used by the delivered S7b1 fixture vs QA pinned-forward capture
    depth_seg = json.loads((S7B / "depth_seg.json").read_text(encoding="utf-8"))
    base_capture = json.loads((REF / "ref_base.json").read_text(encoding="utf-8"))
    report["camera_poses_bits_equal"] = [
        {
            "index": index,
            "xpos_equal": fixture["origin_bits"] == capture["xpos_bits"],
            "xmat_equal": fixture["xmat_bits"] == capture["xmat_bits"],
        }
        for index, (fixture, capture) in enumerate(
            zip(depth_seg["camera_data_check"], base_capture["poses"])
        )
    ]
    # product base harness used exactly these bits
    product_poses = json.loads((OUT / "harness_base.json").read_text(encoding="utf-8"))["poses"]
    report["product_pose_bits_equal_fixture"] = [
        {
            "index": index,
            "xpos_equal": product["xpos_bits"] == fixture["origin_bits"],
            "xmat_equal": product["xmat_bits"] == fixture["xmat_bits"],
        }
        for index, (product, fixture) in enumerate(zip(product_poses, depth_seg["camera_data_check"]))
    ]

    # capture.log / kernel cache facts
    capture_log = (pathlib.Path(EVIDENCE) / "capture.log").read_bytes().decode("utf-16")
    same = json.loads((S7B / "rgb_fast_math_false.json").read_text(encoding="utf-8"))
    cache_ptx = pathlib.Path(ROOT / same["kernel_cache"]["ptx_path"])
    report["capture_log"] = {
        "div_rn": re.search(r"ptx div_rn=(\d+) div_approx=(\d+)", capture_log).groups(),
        "cache_ptx_exists": cache_ptx.exists(),
        "cache_ptx_bytes": cache_ptx.stat().st_size if cache_ptx.exists() else None,
        "cache_ptx_sha256": hashlib.sha256(cache_ptx.read_bytes()).hexdigest()
        if cache_ptx.exists()
        else None,
        "recorded_ptx_bytes": same["kernel_cache"]["ptx_bytes"],
        "recorded_ptx_sha256": same["kernel_cache"]["ptx_sha256"],
    }
    report["capture_log"]["cache_matches_record"] = (
        report["capture_log"]["cache_ptx_bytes"] == report["capture_log"]["recorded_ptx_bytes"]
        and report["capture_log"]["cache_ptx_sha256"] == report["capture_log"]["recorded_ptx_sha256"]
    )

    out = OUT / "judge_hashes.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
