"""QA judge: synthetic vertex-normal discrimination (independent replication).

For six extreme directions, QA replaces all four mesh vertex normals, renders the product with
`enable_vertex_normals=True/False` (spp=1, same pose/rays) and counts differing pixels.

Evidence strength (as required by Arch):
  * this proves the `vertex_normal` branch is reached and changes shading;
  * it is NOT a pinned reference comparison — no upstream capture exists for synthetic normals;
  * the scene-a reference itself is bitwise identical with the flag on/off (QA re-capture),
    so the variant has no discriminating power on the shipped fixture.
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
SYNTH = ROOT / "target/qa-s7b3/out/synthetic"


def main() -> int:
    cases = json.loads((SYNTH / "synthetic_normal.json").read_text(encoding="utf-8"))["cases"]
    by_case = {}
    for case in cases:
        by_case[case["case"]] = np.frombuffer(
            (SYNTH / f"rgb_{case['case']}.bin").read_bytes(), dtype=np.uint32
        )
    report = {"directions": []}
    for case in cases:
        if not case["vertex_normals"]:
            continue
        tag_on = case["case"]
        tag_off = tag_on[:-1] + "0"
        if tag_off not in by_case:
            continue
        left = by_case[tag_on]
        right = by_case[tag_off]
        changed = int(np.count_nonzero(left != right))
        report["directions"].append(
            {
                "direction": case["direction"],
                "vertex_on_vs_off_changed_pixels": changed,
                "total_pixels": int(left.size),
            }
        )
    report["any_direction_discriminates"] = any(
        entry["vertex_on_vs_off_changed_pixels"] > 0 for entry in report["directions"]
    )
    report["reference_has_discriminating_power_on_scene_a"] = False
    report["evidence_strength"] = (
        "branch-reachability only: no pinned reference exists for synthetic mesh normals; "
        "the six-direction sweep proves the vertex_normal path changes shading, not that the "
        "product matches upstream for such normals"
    )
    out = ROOT / "target/qa-s7b3/out/judge_synthetic.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
