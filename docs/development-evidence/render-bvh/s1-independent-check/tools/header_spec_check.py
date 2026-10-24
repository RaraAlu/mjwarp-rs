"""Independent check of the S1 render field spec (kind/width) against MuJoCo 3.12 headers.

For the 116 render fields S1 reads, derive expected scalar type and count from
MuJoCo's own mjmodel.h/mjvisualize.h declarations and comments, then compare with
model-312.json metadata (which S1's Rust spec must match element for element).
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
PKG = ROOT / "target/toolchains/mujoco-3.12.0/package/include/mujoco"
REF = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
OUT = ROOT / "target/qa-render-bvh-s1/header-spec-check.json"
EV = ROOT / "docs/development-evidence/render-bvh/s1-independent-check"

EXCLUDED = {
    "geom_contype", "geom_conaffinity", "geom_condim", "geom_priority", "geom_plugin",
    "geom_sameframe", "geom_solmix", "geom_solref", "geom_solimp", "geom_aabb",
    "geom_rbound", "geom_friction", "geom_margin", "geom_gap", "geom_surfacevel",
    "geom_adhesion", "geom_fluid",
}
CTYPE_KIND = {
    "int": "int32",
    "mjtNum": "float64",
    "float": "float32",
    "mjtByte": "byte",
    "mjtSize": "int64",
    "mjtBool": "byte",
    "char": "byte",
}
UNIT_ALIASES = {"mjNTEXROLE": 10}


def main() -> int:
    ref = json.loads(REF.read_text(encoding="utf-8"))
    arrays = {k: v for k, v in ref["arrays"].items() if isinstance(v, dict)}
    counts = ref["counts"]
    fields = [k for k in arrays if k not in EXCLUDED]

    header = {}
    for name in ("mjmodel.h", "mjvisualize.h"):
        text = (PKG / name).read_text(encoding="utf-8", errors="replace")
        for line in text.splitlines():
            m = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*\*?\s+([A-Za-z_][A-Za-z0-9_]*)(\[[0-9]+\])?\s*;\s*//(.*)$", line)
            if m:
                header.setdefault(m.group(2), (m.group(1), m.group(4)))

    failures = []
    rows = []
    for field in fields:
        entry = arrays[field]
        if field.startswith("vis.") or field.startswith("stat.") or field == "names":
            rows.append({"field": field, "kind": entry["kind"], "count": entry["count"],
                         "source": "mjvisualize/stat special", "ok": True})
            continue
        if field not in header:
            failures.append(f"{field}: not declared in MuJoCo headers")
            continue
        ctype, comment = header[field]
        expected_kind = CTYPE_KIND.get(ctype)
        unit_match = re.search(r"\(([A-Za-z_0-9]+) x (\d+)\)", comment)
        if unit_match:
            unit_token, width = unit_match.group(1), int(unit_match.group(2))
            if unit_token in UNIT_ALIASES:
                unit = UNIT_ALIASES[unit_token]
            else:
                unit = counts.get(unit_token)
            expected_count = None if unit is None else unit * width
        else:
            unit_token = None
            expected_count = None
        ok = entry["kind"] == expected_kind and (expected_count is None or entry["count"] == expected_count)
        if not ok:
            failures.append(
                f"{field}: header {ctype}/{comment.strip()} -> kind {expected_kind}, count {expected_count}; "
                f"reference kind {entry['kind']}, count {entry['count']}"
            )
        rows.append({
            "field": field,
            "header_type": ctype,
            "header_comment": comment.strip(),
            "kind": entry["kind"],
            "expected_kind": expected_kind,
            "count": entry["count"],
            "expected_count": expected_count,
            "ok": ok,
        })

    # specials
    specials = {
        "vis.headlight.ambient": ("float", 3), "vis.headlight.diffuse": ("float", 3),
        "vis.headlight.specular": ("float", 3), "vis.headlight.active": ("int", 1),
        "vis.map.znear": ("float", 1), "vis.map.zfar": ("float", 1),
        "stat.extent": ("mjtNum", 1), "stat.center": ("mjtNum", 3), "names": ("char", None),
    }
    for field, (ctype, width) in specials.items():
        entry = arrays[field]
        expected_kind = CTYPE_KIND[ctype]
        count = counts["nnames"] if field == "names" else width
        ok = entry["kind"] == expected_kind and entry["count"] == count
        if not ok:
            failures.append(f"{field}: special {ctype}/{width} -> kind {expected_kind}, count {count}; "
                            f"reference kind {entry['kind']}, count {entry['count']}")

    report = {
        "task_id": "2026-10-24-render-bvh-s1-independent-check",
        "fields_checked": len(rows),
        "excluded_fields": sorted(EXCLUDED),
        "116_plus_17": len(fields) + len(EXCLUDED),
        "reference_arrays": len(arrays),
        "kind_count_mismatches": failures,
        "rows": rows,
        "verdict": "pass" if not failures else "fail",
    }
    out = EV / "header-spec-check.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"verdict": report["verdict"], "fields": len(rows), "116_plus_17": report["116_plus_17"],
                      "reference_arrays": report["reference_arrays"], "failures": failures}, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
