# -*- coding: utf-8 -*-
"""QA independent 5-case bitwise verification (option-A fix).

Reads the QA dump produced from a copy tree (r2 + test-only dump of expected/actual
contact rows, both sorted by the product's own sort key) and the CPU reference JSON.
Independently re-sorts/re-pairs in Python and verifies bit equality of dist/pos/frame.
Writes qa-5case-bitwise.json.
"""
import json
import struct
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/qa-a-check"
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
DUMP = WORK / "qa-contacts-dump.json"
REF = REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"


def f32(value) -> float:
    return struct.unpack("<f", struct.pack("<f", float(value)))[0]


def bits(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def f32_key(value: float) -> int:
    raw = bits(value)
    return raw | 0x8000_0000 if raw < 0x8000_0000 else 0xFFFF_FFFF - raw


def raw_key(row):
    """row = [worldid, geomcollisionid, dist_bits, pos3 bits..., frame9 bits...]."""
    return (f32_key(struct.unpack("<f", struct.pack("<I", row[2]))[0]),
            f32_key(struct.unpack("<f", struct.pack("<I", row[3]))[0]),
            f32_key(struct.unpack("<f", struct.pack("<I", row[4]))[0]),
            f32_key(struct.unpack("<f", struct.pack("<I", row[5]))[0]),
            row[1])


def bits_to_float(raw: int) -> float:
    return struct.unpack("<f", struct.pack("<I", raw))[0]


def decode(rows):
    return [{"worldid": r[0], "geomcollisionid": r[1], "dist_bits": r[2],
             "pos_bits": r[3:6], "frame_bits": r[6:15]} for r in rows]


def reference_rows(ref_case):
    out = ref_case["output"]
    count = out["count"]
    f = out["fields"]
    dist = f["dist"]                     # list of rows, each row a list
    pos = f["pos"]
    frame = f["frame"]
    worldid = f["worldid"]
    collisionid = f["geomcollisionid"]
    rows = []
    for index in range(count):
        rows.append((int(worldid[index][0]), int(collisionid[index][0]), bits(dist[index][0]),
                     [bits(pos[index][axis]) for axis in range(3)],
                     [bits(frame[index][cell]) for cell in range(9)]))
    return rows


dump = json.loads(DUMP.read_text(encoding="utf-8"))
ref = json.loads(REF.read_text(encoding="utf-8"))
result = {"dump": str(DUMP), "reference": str(REF), "cases": {}}
overall = {"dist_bit_equal": 0, "dist_total": 0, "pos_bit_equal": 0, "pos_total": 0,
           "frame_bit_equal": 0, "frame_total": 0, "max_frame_abs": 0.0,
           "structural_issues": []}

for name, worlds in dump["cases"].items():
    ref_case = ref["cases"][name]
    ref_rows_all = reference_rows(ref_case)
    entry = {"worlds": len(worlds), "expected_reference_json_mismatches": [],
             "count": {"expected": 0, "actual": 0}, "per_world": []}
    for world_entry in worlds:
        world = world_entry["world"]
        expected_raw = sorted([list(row) for row in world_entry["expected"]], key=raw_key)
        actual_raw = sorted([list(row) for row in world_entry["actual"]], key=raw_key)
        ref_rows = sorted([[row[0], row[1], row[2], *row[3], *row[4]]
                           for row in ref_rows_all if row[0] == world], key=raw_key)
        expected = decode(expected_raw)
        actual = decode(actual_raw)
        ref_decoded = decode(ref_rows)
        if expected != ref_decoded:
            entry["expected_reference_json_mismatches"].append(world)
        if len(expected) != len(actual):
            overall["structural_issues"].append(f"{name} world {world}: count {len(expected)} vs {len(actual)}")
        entry["count"]["expected"] += len(expected)
        entry["count"]["actual"] += len(actual)
        wstats = {"world": world, "contacts": len(expected), "dist_bit_equal": 0,
                  "pos_bit_equal": 0, "pos_total": 0, "frame_bit_equal": 0, "frame_total": 0,
                  "max_frame_abs": 0.0}
        for exp, act in zip(expected, actual):
            wstats["dist_bit_equal"] += int(exp["dist_bits"] == act["dist_bits"])
            for axis in range(3):
                wstats["pos_bit_equal"] += int(exp["pos_bits"][axis] == act["pos_bits"][axis])
                wstats["pos_total"] += 1
            for cell in range(9):
                wstats["frame_bit_equal"] += int(exp["frame_bits"][cell] == act["frame_bits"][cell])
                wstats["frame_total"] += 1
                diff = abs(bits_to_float(exp["frame_bits"][cell]) - bits_to_float(act["frame_bits"][cell]))
                wstats["max_frame_abs"] = max(wstats["max_frame_abs"], diff)
        entry["per_world"].append(wstats)
        overall["dist_bit_equal"] += wstats["dist_bit_equal"]
        overall["dist_total"] += wstats["contacts"]
        overall["pos_bit_equal"] += wstats["pos_bit_equal"]
        overall["pos_total"] += wstats["pos_total"]
        overall["frame_bit_equal"] += wstats["frame_bit_equal"]
        overall["frame_total"] += wstats["frame_total"]
        overall["max_frame_abs"] = max(overall["max_frame_abs"], wstats["max_frame_abs"])
    entry["count_equal"] = entry["count"]["expected"] == entry["count"]["actual"]
    result["cases"][name] = entry

result["summary"] = overall
result["bitwise_zero_diff"] = (
    not overall["structural_issues"]
    and overall["dist_bit_equal"] == overall["dist_total"]
    and overall["pos_bit_equal"] == overall["pos_total"]
    and overall["frame_bit_equal"] == overall["frame_total"]
    and overall["max_frame_abs"] == 0.0
    and all(not c["expected_reference_json_mismatches"] for c in result["cases"].values()))
result["expected_rows_match_cpu_reference_json"] = all(
    not c["expected_reference_json_mismatches"] for c in result["cases"].values())

(TASK / "qa-5case-bitwise.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"bitwise_zero_diff={result['bitwise_zero_diff']}")
print(f"dist {overall['dist_bit_equal']}/{overall['dist_total']} pos {overall['pos_bit_equal']}/{overall['pos_total']} "
      f"frame {overall['frame_bit_equal']}/{overall['frame_total']} max_frame_abs={overall['max_frame_abs']}")
for name, entry in result["cases"].items():
    print(name, entry["count"], entry["count_equal"], entry["expected_reference_json_mismatches"])
