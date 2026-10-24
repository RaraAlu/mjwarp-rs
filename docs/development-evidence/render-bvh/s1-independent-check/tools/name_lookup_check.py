"""Independent recomputation of the S1 mj_id2name parity (87 comparisons).

Oracle: mj_id2name from the loaded MuJoCo 3.12 library (name_oracle.json, produced
by an independently written C++ probe).
Product semantics: name_*adr table + `names` NUL-terminated buffer parsed exactly
as documented for `RenderFields::name_bytes` (negative/out-of-range id -> None,
negative/out-of-range address -> None, missing NUL -> None, empty name -> []).
"""

from __future__ import annotations

import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
REF = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
ORACLE = ROOT / "target/qa-render-bvh-s1/name_oracle.json"
OUT = ROOT / "docs/development-evidence/render-bvh/s1-independent-check/name-lookup-check.json"

TABLES = {
    "body": "name_bodyadr", "joint": "name_jntadr", "geom": "name_geomadr",
    "site": "name_siteadr", "camera": "name_camadr", "light": "name_lightadr",
    "mesh": "name_meshadr", "texture": "name_texadr", "material": "name_matadr",
}


def table_name(table: list[int], names: bytes, obj_id: int):
    if obj_id < 0 or obj_id >= len(table):
        return None
    adr = table[obj_id]
    if adr < 0 or adr >= len(names):
        return None
    tail = names[adr:]
    end = tail.find(b"\x00")
    if end < 0:
        return None
    return tail[:end]


def main() -> int:
    ref = json.loads(REF.read_text(encoding="utf-8"))
    arrays = {k: v for k, v in ref["arrays"].items() if isinstance(v, dict)}
    names = bytes(arrays["names"]["values"])
    oracle = json.loads(ORACLE.read_text(encoding="utf-8"))
    cases = []
    mismatches = []
    for obj, table_field in TABLES.items():
        table = arrays[table_field]["values"]
        count = len(table)
        # same comparison surface as the delivered native test: 7 boundary ids per
        # object plus every id in range (id 0/1/count-1 are counted twice, hence 87).
        sequence = [(-2, True), (-1, True), (0, True), (1, True), (count - 1, True),
                    (count, True), (count + 1, True)] + [(i, False) for i in range(count)]
        for obj_id, boundary in sequence:
            got = table_name(table, names, obj_id)
            key = f"{obj}:{obj_id}"
            want = oracle[key]
            want_bytes = None if want is None else want.encode("utf-8")
            ok = got == want_bytes
            if not ok:
                mismatches.append({"case": key, "rust_semantics": None if got is None else got.decode("utf-8", "replace"),
                                   "mj_id2name": want})
            cases.append({"case": key, "match": ok,
                          "value": None if want is None else want,
                          "boundary": boundary})
    report = {
        "task_id": "2026-10-24-render-bvh-s1-independent-check",
        "oracle": "mj_id2name from the loaded MuJoCo 3.12 library via an independent C++ probe",
        "semantics": "negative/out-of-range id -> None; negative/out-of-range name address -> None; missing NUL -> None; empty name -> empty bytes",
        "objects": list(TABLES),
        "comparisons": len(cases),
        "boundary_comparisons": sum(1 for c in cases if c["boundary"]),
        "mismatches": mismatches,
        "verdict": "pass" if not mismatches else "fail",
        "cases": cases,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"verdict": report["verdict"], "comparisons": len(cases),
                      "boundary": report["boundary_comparisons"], "mismatches": len(mismatches)}, indent=2))
    return 0 if not mismatches else 1


if __name__ == "__main__":
    sys.exit(main())
