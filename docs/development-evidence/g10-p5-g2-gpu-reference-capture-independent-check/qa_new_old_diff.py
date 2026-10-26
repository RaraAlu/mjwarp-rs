# -*- coding: utf-8 -*-
"""QA check 2: same inputs (old CPU ref vs new GPU ref) + 17-field difference magnitude.

Self-written (does not import the delivery's compare_references.py / verify_inputs.py).
Rows are paired by their INT_FIELDS key, because the GPU capture's contact row order can
differ from the CPU capture while the contact set is identical (the product's comparison
sorts contacts by key; `case_compare.rs::sort_contacts`). Writes qa-new-old-diff.json.
"""
import hashlib
import json
import struct
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
OLD = REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"
NEW = REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json"
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def flatten(value):
    if isinstance(value, list):
        out = []
        for item in value:
            out.extend(flatten(item))
        return out
    return [value]


def f32_bits(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def ulp32(left: float, right: float) -> int:
    if left == right:
        return 0
    a, b = f32_bits(left), f32_bits(right)
    if (a >> 31) != (b >> 31):
        return 2 ** 32 - 1
    return abs(a - b)


def int_rows(output: dict) -> list:
    count = output["count"]
    rows = []
    for index in range(count):
        key = []
        for field in INT_FIELDS:
            values = output["fields"].get(field, [])
            key.extend(int(v) for v in (values[index] if index < len(values) else []))
        rows.append(tuple(key))
    return rows


old = json.loads(OLD.read_text(encoding="utf-8"))
new = json.loads(NEW.read_text(encoding="utf-8"))
result = {
    "old": {"path": str(OLD.relative_to(REPO)).replace("\\", "/"), "sha256": sha256(OLD),
            "bytes": OLD.stat().st_size},
    "new": {"path": str(NEW.relative_to(REPO)).replace("\\", "/"), "sha256": sha256(NEW),
            "bytes": NEW.stat().st_size},
    "revision": {"old": old.get("upstream_revision"), "new": new.get("upstream_revision"),
                 "same": old.get("upstream_revision") == new.get("upstream_revision")},
    "versions": {"old": old["provenance"].get("versions"), "new": new["provenance"].get("versions")},
    "case_names": {"old": list(old["cases"]), "new": list(new["cases"]),
                   "same_order": list(old["cases"]) == list(new["cases"])},
    "cases": {},
}
overall = {"float_bit_equal": 0, "float_total": 0, "frame_max_abs": 0.0,
           "int_mismatches": 0, "count_mismatches": 0, "structural_issues": []}

for name, old_case in old["cases"].items():
    new_case = new["cases"][name]
    o, n = old_case["output"], new_case["output"]
    rows_old, rows_new = int_rows(o), int_rows(n)
    entry = {
        "xml_same": old_case["xml"] == new_case["xml"],
        "xml_sha256_same": old_case["xml_sha256"] == new_case["xml_sha256"],
        "qpos_same": old_case["qpos"] == new_case["qpos"],
        "qpos_bits_same": old_case["qpos_bits"] == new_case["qpos_bits"],
        "two_captures_bitwise_identical_new": new_case["two_captures_bitwise_identical"],
        "count": {"old": o["count"], "new": n["count"]},
        "row_order_raw_same": rows_old == rows_new,
        "int_key_multiset_same": sorted(rows_old) == sorted(rows_new),
        "int_keys_unique": len(set(rows_old)) == len(rows_old) == len(set(rows_new)) == len(rows_new),
        "int_fields": {},
        "float_fields": {},
    }
    if entry["count"]["old"] != entry["count"]["new"]:
        overall["count_mismatches"] += 1
        overall["structural_issues"].append(f"{name}: count {entry['count']}")
    if not entry["int_key_multiset_same"]:
        overall["structural_issues"].append(f"{name}: int-key multiset differs")

    # per-int-field multiset comparison (row order independent)
    for field in INT_FIELDS:
        a = [flatten(row) for row in compute] if False else None
        va = [tuple(flatten(v)) for v in o["fields"].get(field, [])]
        vb = [tuple(flatten(v)) for v in n["fields"].get(field, [])]
        same_multiset = sorted(va) == sorted(vb)
        entry["int_fields"][field] = same_multiset
        if not same_multiset:
            overall["int_mismatches"] += 1
            overall["structural_issues"].append(f"{name}: int field {field} multiset differs")

    # row pairing by int key
    pair = None
    if entry["int_keys_unique"]:
        index_new = {key: idx for idx, key in enumerate(rows_new)}
        pair = [(idx, index_new[key]) for idx, key in enumerate(rows_old)]
        entry["paired_by"] = "int_key"
    elif rows_old == rows_new:
        pair = [(idx, idx) for idx in range(len(rows_old))]
        entry["paired_by"] = "raw_index"
    else:
        entry["paired_by"] = "unpaired_duplicate_keys"
        overall["structural_issues"].append(f"{name}: duplicate int keys, could not pair rows independently")

    if pair is not None:
        for field in FLOAT_FIELDS:
            old_rows = o["fields"].get(field, [])
            new_rows = n["fields"].get(field, [])
            pairs = []
            for old_idx, new_idx in pair:
                a = flatten(old_rows[old_idx]) if old_idx < len(old_rows) else []
                b = flatten(new_rows[new_idx]) if new_idx < len(new_rows) else []
                pairs.extend(zip(a, b))
            diffs = [abs(x - y) for x, y in pairs]
            bit_equal = sum(1 for x, y in pairs if x == y)
            ulps = [ulp32(x, y) for x, y in pairs]
            max_abs = max(diffs) if diffs else 0.0
            overall["float_bit_equal"] += bit_equal
            overall["float_total"] += len(pairs)
            if field == "frame":
                overall["frame_max_abs"] = max(overall["frame_max_abs"], max_abs)
            entry["float_fields"][field] = {
                "bit_equal": bit_equal, "total": len(pairs), "max_abs": max_abs,
            }

    for field in ("geom_xpos", "geom_xmat"):
        a = flatten(o.get(field, []))
        b = flatten(n.get(field, []))
        absd = [abs(x - y) for x, y in zip(a, b)]
        entry[field] = [sum(1 for x, y in zip(a, b) if x == y), len(a),
                        max(absd) if absd else 0.0]
    result["cases"][name] = entry

result["summary"] = overall
result["structural_fail"] = bool(overall["structural_issues"])
result["new_reference_two_captures_all_true"] = all(
    case["two_captures_bitwise_identical"] for case in new["cases"].values())
result["new_counts"] = {name: case["output"]["count"] for name, case in new["cases"].items()}
result["raw_row_order_differs"] = [name for name, entry in result["cases"].items()
                                   if not entry["row_order_raw_same"]]

(TASK / "qa-new-old-diff.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

print(f"old_sha={result['old']['sha256']}")
print(f"new_sha={result['new']['sha256']}")
print(f"revision_same={result['revision']['same']} structural_fail={result['structural_fail']}")
for name, entry in result["cases"].items():
    print(f"{name}: count={entry['count']} order_raw={entry['row_order_raw_same']} "
          f"int_multiset={entry['int_key_multiset_same']} paired_by={entry['paired_by']} "
          f"frame_max_abs={entry['float_fields']['frame']['max_abs']:.3e} "
          f"frame_bitequal={entry['float_fields']['frame']['bit_equal']}/{entry['float_fields']['frame']['total']}")
print(f"float_bit_equal={overall['float_bit_equal']}/{overall['float_total']} "
      f"frame_max_abs={overall['frame_max_abs']:.3e}")
