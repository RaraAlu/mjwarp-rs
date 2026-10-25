"""QA judge: RGB fixture decode / packed-ABGR semantics / cross-recipe record / provenance.

Independent decode of the two RGB fixtures plus the S7b1 depth/seg fixtures:
  * packed u32 must be ABGR-A8 (alpha byte == 255) and length == w*h;
  * `rgb.json` (S0b-derived) must equal npz f32 -> rint(k*255) bytes -> pack ABGR;
  * `rgb_fast_math_false.json` (same-recipe, raw `rc.rgb_data`) cross_recipe histograms must be
    reproducible from the two fixtures;
  * its `self_check.depth_seg_recipe` must match the S7b1 same-recipe depth/seg fixture bits;
  * fixture byte sizes / sha256 must match the receipt/README claims.
"""

from __future__ import annotations

import hashlib
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"
SCENE = ROOT / "fixtures/render-bvh/scene-a"
RECEIPT = ROOT / "docs/development-evidence/render-bvh/s7b2-shading/receipt.json"
README = ROOT / "docs/development-evidence/render-bvh/s7b2-shading/README.md"
REF = ROOT / "target/qa-s7b2/ref"

CLAIMS = {
    "rgb.json": {
        "bytes": 1322081,
        "sha256": "57f85a049c25be3c98dccc37c304fb6db6232215266fdfe1b616a2202a7126d2",
    },
    "rgb_fast_math_false.json": {
        "bytes": 1323726,
        "sha256": "d0de2b6069241518513a8b6ec1f65dfb76ce43d4c86b663374e8cf32c6e85b4a",
    },
}


def packed_bytes(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)).astype(np.uint32),
            (values >> np.uint32(8) & np.uint32(0xFF)).astype(np.uint32),
            (values & np.uint32(0xFF)).astype(np.uint32),
        ],
        axis=-1,
    )


def byte_histogram(actual: np.ndarray, expected: np.ndarray) -> list[int]:
    left = packed_bytes(actual).astype(np.int16)
    right = packed_bytes(expected).astype(np.int16)
    max_diff = np.abs(left - right).max(axis=1)
    return [int(np.count_nonzero(max_diff == value)) for value in (0, 1, 2, 3)] + [
        int(np.count_nonzero(max_diff > 3))
    ]


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    report = {"files": {}, "cameras": {}, "cross_recipe": {}, "checks": {}}
    documents = {}
    for name in ("rgb.json", "rgb_fast_math_false.json"):
        path = S7B / name
        data = path.read_bytes()
        claim = CLAIMS[name]
        entry = {
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
            "bytes_match_claim": len(data) == claim["bytes"],
            "sha256_match_claim": hashlib.sha256(data).hexdigest() == claim["sha256"],
        }
        report["files"][name] = entry
        documents[name] = json.loads(data.decode("utf-8"))

    npz = np.load(SCENE / "reference/rgb.npz")
    report["checks"]["npz_keys"] = sorted(npz.files)
    s0b = documents["rgb.json"]
    same = documents["rgb_fast_math_false.json"]
    for index in (0, 1):
        camera = s0b["cameras"][index]
        packed = np.asarray(camera["rgb_bits_packed"], dtype=np.uint32)
        alpha = packed >> np.uint32(24)
        rebuilt_rgb = np.rint(
            np.asarray(npz[f"rgb_cam{index}"], dtype=np.float64) * 255.0
        ).astype(np.uint32)
        expected_packed = (
            (np.uint32(255) << np.uint32(24))
            | (rebuilt_rgb[..., 0] << np.uint32(16))
            | (rebuilt_rgb[..., 1] << np.uint32(8))
            | rebuilt_rgb[..., 2]
        ).reshape(-1)
        same_camera = same["cameras"][index]
        same_packed = np.asarray(same_camera["rgb_bits_packed"], dtype=np.uint32)
        cross = same["cross_recipe"]["per_camera"][index]
        recorded_hist = cross["histogram_0_1_2_3_gt3"]
        computed_hist = byte_histogram(same_packed, packed)
        report["cameras"][index] = {
            "width": camera["width"],
            "height": camera["height"],
            "pixels": camera["pixels"],
            "length_ok": packed.size == camera["width"] * camera["height"],
            "alpha_all_255": bool(np.all(alpha == np.uint32(255))),
            "unique_pixels_recorded": camera["unique_pixels"],
            "unique_pixels_recomputed": int(np.unique(packed).size),
            "black_pixels_recorded": camera.get("black_pixels", camera.get("zero_pixels")),
            "black_pixels_recomputed": int(np.count_nonzero(packed == np.uint32(0xFF000000))),
            "rgb_json_equals_npz_rint_pack": bool(np.array_equal(packed, expected_packed)),
            "same_recipe_length_ok": same_packed.size == same_camera["width"] * same_camera["height"],
            "same_recipe_alpha_all_255": bool(np.all(same_packed >> np.uint32(24) == np.uint32(255))),
            "cross_recipe_hist_recorded": recorded_hist,
            "cross_recipe_hist_recomputed": computed_hist,
            "cross_recipe_hist_matches": recorded_hist == computed_hist,
        }
    report["cross_recipe"] = {
        "note": same["cross_recipe"].get("treatment"),
        "compared_with": same["cross_recipe"].get("compared_with"),
    }

    # depth/seg fixtures: the same-recipe fixture stores raw capture; the S7b1 `depth_seg.json`
    # is the S0b *fast-math* reference, so they are expected to differ (known cross-recipe
    # record). The delivered self_check claim is that its own capture equals the S7b1 *same
    # recipe* fixture; QA verifies the equivalent with its own pinned capture (`ref_base`).
    depth_recipe = json.loads((S7B / "depth_seg_fast_math_false.json").read_text(encoding="utf-8"))
    depth_s7b1 = json.loads((S7B / "depth_seg.json").read_text(encoding="utf-8"))
    qa_capture = json.loads((REF / "ref_base.json").read_text(encoding="utf-8"))
    report["depth_seg"] = []
    for index in (0, 1):
        camera = depth_recipe["cameras"][index]
        base = depth_s7b1["cameras"][index]
        capture = qa_capture["cameras"][index]
        depth_bits = np.asarray(camera["depth_bits"], dtype=np.uint32)
        seg_pairs = np.asarray(camera["seg_pairs"], dtype=np.int32)
        base_bits = np.asarray(base["depth_bits"], dtype=np.uint32)
        qa_bits = np.asarray(capture["depth_bits"], dtype=np.uint32)
        qa_seg = np.asarray(capture["seg_pairs"], dtype=np.int32).reshape(-1)
        cross_ulp = np.abs(
            np.where(
                base_bits.astype(np.int64) < 0,
                np.int64(-2147483648) - base_bits.astype(np.int64),
                base_bits.astype(np.int64),
            )
            - np.where(
                depth_bits.astype(np.int64) < 0,
                np.int64(-2147483648) - depth_bits.astype(np.int64),
                depth_bits.astype(np.int64),
            )
        )
        report["depth_seg"].append(
            {
                "index": index,
                "pixels": camera["width"] * camera["height"],
                "depth_len_ok": depth_bits.size == camera["width"] * camera["height"],
                "seg_len_ok": seg_pairs.size == 2 * camera["width"] * camera["height"],
                "qa_capture_depth_equals_recipe_fixture": bool(np.array_equal(qa_bits, depth_bits)),
                "qa_capture_seg_equals_recipe_fixture": bool(np.array_equal(qa_seg, seg_pairs)),
                "recipe_vs_s0b_fastmath_depth": {
                    "known_cross_recipe": True,
                    "mismatched": int(np.count_nonzero(cross_ulp > 0)),
                    "max_ulp": int(cross_ulp.max()),
                },
            }
        )
    recorded_self = same["self_check"]["depth_seg_recipe"]["per_camera"]
    report["checks"]["self_check_claim_matches_independent"] = [
        {
            "index": index,
            "recorded_depth_equal": recorded_self[index]["depth_bitwise_equal"],
            "recorded_seg_equal": recorded_self[index]["seg_bitwise_equal"],
            "qa_capture_depth_equal": entry["qa_capture_depth_equals_recipe_fixture"],
            "qa_capture_seg_equal": entry["qa_capture_seg_equals_recipe_fixture"],
        }
        for index, entry in enumerate(report["depth_seg"])
    ]

    # kernel cache record (same-recipe capture): file exists and hashes match the record
    cache = same["kernel_cache"]
    cache_path = ROOT / cache["ptx_path"]
    report["kernel_cache"] = {
        "ptx_path": cache["ptx_path"],
        "exists": cache_path.exists(),
        "ptx_bytes": cache_path.stat().st_size if cache_path.exists() else None,
        "ptx_sha256": sha256(cache_path) if cache_path.exists() else None,
        "recorded_bytes": cache["ptx_bytes"],
        "recorded_sha256": cache["ptx_sha256"],
        "div_rn": cache["div_rn"],
        "div_approx": cache["div_approx"],
    }
    report["kernel_cache"]["matches"] = (
        report["kernel_cache"]["exists"]
        and report["kernel_cache"]["ptx_bytes"] == cache["ptx_bytes"]
        and report["kernel_cache"]["ptx_sha256"] == cache["ptx_sha256"]
    )

    out = ROOT / "target/qa-s7b2/out/judge_fixtures.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1)[:6000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
