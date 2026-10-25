"""QA judge: S7b3 flag/AA fixtures (hashes, schema, provenance, discriminating records).

Independently:
  * file bytes/sha256 vs the receipt claims and vs QA's own pinned-3.15 re-captures;
  * every RGB fixture decodes as packed ABGR-A8 (alpha 255), length == w*h; depth 4 bytes/pixel,
    seg 2 int32/pixel;
  * `index.json` per-variant `stats_vs_base` / `has_discriminating_effect` / determinism records
    vs QA's own variant-vs-base statistics;
  * `depth_seg` fixtures carry `compile_fast_math=false` and the same `camera_data_check` bits as
    the S7b1 fixture.
"""

from __future__ import annotations

import hashlib
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
FLAGS = ROOT / "fixtures/render-bvh/scene-a/s7b/flags"
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"
RECEIPT = ROOT / "docs/development-evidence/render-bvh/s7b3-flags/receipt.json"
REF = ROOT / "target/qa-s7b3/ref"
OUT = ROOT / "target/qa-s7b3/out"
VARIANTS = ["textures_off", "vertex_normals_off", "specular_off", "emission_off", "culling_off", "aa2"]


def main() -> int:
    receipt = json.loads(RECEIPT.read_text(encoding="utf-8"))
    claims = {}
    for variant in receipt["variants"]:
        reference = variant["reference"]
        if "file" in reference:
            claims[reference["file"].rsplit("/", 1)[-1]] = (reference["bytes"], reference["sha256"])
        else:
            for entry in reference.values():
                claims[entry["file"].rsplit("/", 1)[-1]] = (entry["bytes"], entry["sha256"])
    claims["index.json"] = (receipt["flag_index"]["bytes"], receipt["flag_index"]["sha256"])
    report = {"files": {}, "checks": {}, "fixtures": {}}
    for name, (expected_bytes, expected_sha) in claims.items():
        data = (FLAGS / name).read_bytes()
        report["files"][name] = {
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
            "bytes_match": len(data) == expected_bytes,
            "sha256_match": hashlib.sha256(data).hexdigest() == expected_sha,
        }
    script = ROOT / "scripts/authoring/render-bvh-s7b/capture_flags_aa.py"
    report["checks"]["capture_script_sha256"] = hashlib.sha256(
        script.read_bytes().replace(b"\r\n", b"\n")
    ).hexdigest()
    report["checks"]["capture_script_matches_receipt"] = (
        report["checks"]["capture_script_sha256"] == receipt["fixture_provenance"]["capture_script_sha256"]
    )

    s7b1_camera_data = json.loads((S7B / "depth_seg.json").read_text(encoding="utf-8"))["camera_data_check"]
    qa_vs_base = {
        entry["variant"]: entry
        for entry in json.loads((OUT / "judge_variants.json").read_text(encoding="utf-8"))["vs_base"]
    }
    for variant in VARIANTS:
        rgb = json.loads((FLAGS / f"{variant}_rgb.json").read_text(encoding="utf-8"))
        qa_ref = json.loads((REF / f"ref_{variant}.json").read_text(encoding="utf-8"))
        cameras = []
        rgb_equal = True
        for camera, qa_camera in zip(rgb["cameras"], qa_ref["cameras"]):
            packed = np.asarray(camera["rgb_bits_packed"], dtype=np.uint32)
            cameras.append(
                {
                    "index": camera["index"],
                    "length_ok": int(packed.size) == int(camera["width"]) * int(camera["height"]),
                    "alpha_all_255": bool(np.all(packed >> np.uint32(24) == np.uint32(255))),
                }
            )
            rgb_equal = rgb_equal and camera["rgb_bits_packed"] == qa_camera["rgb_bits"]
        entry = {"cameras": cameras, "equals_qa_capture_rgb": rgb_equal}
        depth_seg_path = FLAGS / f"{variant}_depth_seg.json"
        if depth_seg_path.exists():
            depth_seg = json.loads(depth_seg_path.read_text(encoding="utf-8"))
            depth_ok = True
            seg_ok = True
            for camera, qa_camera in zip(depth_seg["cameras"], qa_ref["cameras"]):
                pixels = int(camera["width"]) * int(camera["height"])
                depth_bits = np.asarray(camera["depth_bits"], dtype=np.uint32)
                seg_pairs = np.asarray(camera["seg_pairs"], dtype=np.int32)
                depth_ok = depth_ok and depth_bits.size == pixels and camera["depth_bits"] == qa_camera["depth_bits"]
                seg_ok = (
                    seg_ok
                    and seg_pairs.size == 2 * pixels
                    and camera["seg_pairs"] == [value for pair in qa_camera["seg_pairs"] for value in pair]
                )
            entry["depth_seg"] = {
                "compile_fast_math": depth_seg.get("recipe", {}).get("compile_fast_math"),
                "camera_data_matches_s7b1": depth_seg.get("camera_data_check") == s7b1_camera_data,
                "depth_len_and_bits_equal_qa": depth_ok,
                "seg_len_and_values_equal_qa": seg_ok,
            }
        report["fixtures"][variant] = entry

    index = json.loads((FLAGS / "index.json").read_text(encoding="utf-8"))
    records = {entry["variant"]: entry for entry in index["variants"]}
    report["index"] = {"variants": sorted(records), "records": {}}
    for variant in VARIANTS:
        record = records[variant]
        stats = qa_vs_base.get(variant, {"cameras": []})["cameras"]
        recorded = record["stats_vs_base"]
        stats_match = len(recorded) == len(stats) == 2
        if stats_match:
            for i in range(2):
                # totals must agree; per-geom attribution may differ because the delivered
                # index attributes changed pixels by the variant/AA segmentation while QA uses
                # the base segmentation (AA/culling change the seg of those very pixels).
                stats_match = stats_match and (
                    recorded[i]["rgb_changed_pixels"] == stats[i]["rgb_changed_pixels"]
                    and recorded[i]["seg_changed_pixels"] == stats[i]["seg_changed_pixels"]
                    and recorded[i]["depth_changed_pixels"] == stats[i]["depth_changed_pixels"]
                )
        report["index"]["records"][variant] = {
            "determinism": record["determinism"],
            "has_discriminating_effect": record["has_discriminating_effect"],
            "qa_has_effect": any(camera["rgb_changed_pixels"] > 0 for camera in stats),
            "stats_match_qa": stats_match,
        }
    out = OUT / "judge_fixtures.json"
    out.write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf-8")
    print(json.dumps(report["files"], indent=1))
    print(json.dumps(report["checks"], indent=1))
    print(json.dumps(report["fixtures"], indent=1))
    print(json.dumps(report["index"], indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
