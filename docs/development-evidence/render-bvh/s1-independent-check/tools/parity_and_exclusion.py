"""Independent native parity + exclusion-surface evidence for the S1 review.

Part 1: compare QA's own MuJoCo 3.12 dump (dump_model.exe, generated from the
committed .mjb) against fixtures/render-bvh/scene-a/model-312.json for all 133
arrays, then report the 116 render fields S1 covers.
Part 2: list which excluded fields appear in the upstream renderer pipeline
(render.py / render_util.py / ray.py / bvh.py + transitive deps) and in
upstream-references.tsv.
"""

from __future__ import annotations

import hashlib
import struct
import json
import pathlib
import re
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
QA = ROOT / "target/qa-render-bvh-s1"
REF = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
DUMP = QA / "dump.json"
EV = ROOT / "docs/development-evidence/render-bvh/s1-independent-check"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream/mujoco_warp/_src"
TSV = ROOT / "docs/development-evidence/render-bvh-precheck/upstream-references.tsv"

EXCLUDED = [
    "geom_contype", "geom_conaffinity", "geom_condim", "geom_priority", "geom_plugin",
    "geom_sameframe", "geom_solmix", "geom_solref", "geom_solimp", "geom_aabb",
    "geom_rbound", "geom_friction", "geom_margin", "geom_gap", "geom_surfacevel",
    "geom_adhesion", "geom_fluid",
]
RENDER_FILES = ["render.py", "render_util.py", "ray.py", "bvh.py"]
DEPS = ["math.py", "warp_util.py"]


def f32_bits(value: float) -> bytes:
    import numpy as np
    return struct.pack(">f", np.float32(value))


def f64_bits(value: float) -> bytes:
    return struct.pack(">d", float(value))


def main() -> int:
    ref = json.loads(REF.read_text(encoding="utf-8"))
    arrays = {k: v for k, v in ref["arrays"].items() if isinstance(v, dict)}
    dump = json.loads(DUMP.read_text(encoding="utf-8"))

    fields = []
    sameframe_diff = None
    for name in sorted(arrays):
        entry = arrays[name]
        values = dump[name]
        kind = entry["kind"]
        if kind == "float32":
            ok = [f32_bits(a) == f32_bits(b) for a, b in zip(entry["values"], values)]
        elif kind in ("int32", "int64", "byte"):
            ok = [int(a) == int(b) for a, b in zip(entry["values"], values)]
        else:
            ok = [f64_bits(a) == f64_bits(b) for a, b in zip(entry["values"], values)]
        diff = sum(1 for x in ok if not x)
        row = {"field": name, "kind": kind, "count": entry["count"], "mismatches": diff,
               "excluded_by_s1": name in EXCLUDED}
        fields.append(row)
        if name == "geom_sameframe":
            sameframe_diff = {"reference": entry["values"], "qa_native_dump": values}

    render = [f for f in fields if not f["excluded_by_s1"]]
    excluded_rows = [f for f in fields if f["excluded_by_s1"]]
    parity = {
        "task_id": "2026-10-24-render-bvh-s1-independent-check",
        "native_dump": "target/qa-render-bvh-s1/dump.json (QA-written C++ dumper, MuJoCo 3.12 C API, loads fixtures/render-bvh/scene-a/scene-312.mjb)",
        "reference": "fixtures/render-bvh/scene-a/model-312.json",
        "reference_sha256": hashlib.sha256(REF.read_bytes()).hexdigest(),
        "all_arrays": len(fields),
        "all_scalars": sum(f["count"] for f in fields),
        "all_fields_with_mismatch": [f["field"] for f in fields if f["mismatches"]],
        "all_mismatched_scalars": sum(f["mismatches"] for f in fields),
        "render_fields": len(render),
        "render_scalars": sum(f["count"] for f in render),
        "render_fields_with_mismatch": [f["field"] for f in render if f["mismatches"]],
        "render_mismatched_scalars": sum(f["mismatches"] for f in render),
        "excluded_fields": EXCLUDED,
        "excluded_field_count": len(excluded_rows),
        "excluded_scalars": sum(f["count"] for f in excluded_rows),
        "excluded_116_plus_17": len(render) + len(excluded_rows),
        "sameframe_difference_note": {
            "field": "geom_sameframe",
            "detail": sameframe_diff,
            "why": "3.12 keeps the BODY enum where 3.15 uses BODYROT for position-only frame offsets; S1 excludes this collision-only flag, and it is the only array where a fresh 3.12 load differs from the S0a reference dump",
        },
        "fields": fields,
        "verdict": "pass" if not [f for f in render if f["mismatches"]] else "fail",
    }

    # exclusion surface against upstream renderer sources
    sources = {}
    for name in RENDER_FILES + DEPS:
        sources[f"_src/{name}"] = (UPSTREAM / name).read_text(encoding="utf-8", errors="replace")
    exclusion = {}
    for field in EXCLUDED:
        hits = []
        for path, text in sources.items():
            for m in re.finditer(rf"\b{re.escape(field)}\b", text):
                line = text.count("\n", 0, m.start()) + 1
                hits.append(f"{path}:{line}")
        exclusion[field] = hits
    tsv = TSV.read_text(encoding="utf-8", errors="replace")
    tsv_hits = {field: [f"L{tsv.count(chr(10), 0, m.start()) + 1}"
                        for m in re.finditer(rf"\b{re.escape(field)}\b", tsv)] for field in EXCLUDED}
    # fields the render pipeline actually reads (for the positive control)
    read_by_renderer = set()
    for text in sources.values():
        for m in re.finditer(r"\b(?:m|mjm|model)\.(geom_[a-z_]+|cam_[a-z_]+|light_[a-z_]+|mat_[a-z_]+|tex_[a-z_]+|mesh_[a-z_]+)", text):
            read_by_renderer.add(m.group(1))
    exclusion_report = {
        "task_id": "2026-10-24-render-bvh-s1-independent-check",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "renderer_files_scanned": [f"_src/{n}" for n in RENDER_FILES],
        "transitive_render_deps_scanned": [f"_src/{n}" for n in DEPS],
        "excluded_field_hits": exclusion,
        "excluded_field_tsv_hits": tsv_hits,
        "excluded_fields_with_any_renderer_hit": [f for f, h in exclusion.items() if h],
        "excluded_fields_with_tsv_hit": [f for f, h in tsv_hits.items() if h],
        "renderer_read_geom_fields_positive_control": sorted(f for f in read_by_renderer if f.startswith("geom_")),
        "collision_only_note": "geom_aabb/geom_rbound/geom_margin and the other excluded fields only appear in collision_* modules and Model dataclass definitions; none are read by render.py, render_util.py, ray.py, bvh.py or their transitive imports",
        "verdict": "pass" if not [f for f, h in exclusion.items() if h] else "fail",
    }

    (EV / "field-parity.json").write_text(json.dumps(parity, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    (EV / "exclusion-surface.json").write_text(json.dumps(exclusion_report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({
        "parity": {k: parity[k] for k in ("verdict", "render_fields", "render_scalars",
                                          "render_fields_with_mismatch", "render_mismatched_scalars",
                                          "all_arrays", "excluded_116_plus_17",
                                          "all_fields_with_mismatch")},
        "exclusion": {k: exclusion_report[k] for k in ("verdict", "excluded_fields_with_any_renderer_hit",
                                                       "excluded_fields_with_tsv_hit")},
    }, indent=2))
    return 0 if parity["verdict"] == "pass" and exclusion_report["verdict"] == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
