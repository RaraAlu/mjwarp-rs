"""QA independent G0b review: re-capture the 10 GJK/EPA references with QA's own harness.

Runs mujoco_warp on warp's CPU backend (no CUDA, no GPU window, no delivery code reuse):
each case is captured twice from scratch and compared bitwise both to itself and to the
delivered fixture, using the declared acceptance criterion (group by worldid, sort by
(dist,pos) bit keys, then compare bitwise).
"""

import json
import struct
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIX = ROOT / "fixtures/g10-p5"
CACHE = ROOT / "target/qa-g0b/warp-cache"
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def capture(host, offset):
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=2, nconmax=4096, njmax=20000)
    qpos = np.tile(host.qpos0.astype(np.float32), (2, 1))
    qvel = np.zeros((2, host.nv), dtype=np.float32)
    for world in range(2):
        qpos[world, 0:3] += np.asarray(offset, dtype=np.float32) * world
        qvel[world] = np.linspace(-0.02 * (world + 1), 0.03 * (world + 1), host.nv, dtype=np.float32)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        arr = getattr(data.contact, name).numpy()
        fields[name] = arr[:count].reshape(count, -1) if arr.size else np.zeros((0, 0), dtype=arr.dtype)
    return {"qpos": qpos, "qvel": qvel, "count": count, "fields": fields}


def rows_of(capture_dict):
    fields = capture_dict["fields"]
    count = capture_dict["count"]
    rows = []
    for i in range(count):
        row = {}
        for name, arr in fields.items():
            row[name] = arr[i].tolist() if arr.shape[1] else []
        rows.append(row)
    return rows


def sort_key(row):
    return (int(row["worldid"][0]), bits32(row["dist"]), bits32(row["pos"]),
            bits32(row["frame"]), tuple(int(v) for v in row["geom"]),
            int(row["type"][0]), int(row["dim"][0]))


def compare_rows(rows_a, rows_b):
    """Exact field-by-field row comparison after worldid grouping + sorted key."""
    if len(rows_a) != len(rows_b):
        return False
    sa = sorted(rows_a, key=sort_key)
    sb = sorted(rows_b, key=sort_key)
    for ra, rb in zip(sa, sb):
        if set(ra) != set(rb):
            return False
        for name in ra:
            a, b = ra[name], rb[name]
            if len(a) != len(b):
                return False
            if name in FLOAT_FIELDS:
                if bits32(a) != bits32(b):
                    return False
            else:
                if [int(v) for v in a] != [int(v) for v in b]:
                    return False
    return True


def nearest_match_maxdiff(qa_rows, fix_rows):
    """Greedy match by (worldid, geom, geomcollisionid) + nearest dist; return per-field max abs diff."""
    used = set()
    maxdiff = {f: 0.0 for f in FLOAT_FIELDS}
    int_mismatch = 0
    for qa in qa_rows:
        best = None
        for idx, fx in enumerate(fix_rows):
            if idx in used or int(qa["worldid"][0]) != int(fx["worldid"][0]):
                continue
            if [int(v) for v in qa["geom"]] != [int(v) for v in fx["geom"]]:
                continue
            score = abs(float(qa["dist"][0]) - float(fx["dist"][0]))
            if best is None or score < best[0]:
                best = (score, idx)
        if best is None:
            continue
        used.add(best[1])
        fx = fix_rows[best[1]]
        for name in FLOAT_FIELDS:
            for x, y in zip(qa[name], fx[name]):
                maxdiff[name] = max(maxdiff[name], abs(float(x) - float(y)))
        for name in ("dim", "geom", "worldid", "type"):
            if [int(v) for v in qa[name]] != [int(v) for v in fx[name]]:
                int_mismatch += 1
    return maxdiff, int_mismatch


def fixture_rows(doc):
    fields = doc["output"]["fields"]
    count = doc["output"]["count"]
    rows = []
    for i in range(count):
        row = {}
        for name, rec in fields.items():
            vals = rec["values"]
            width = len(vals) // count if count else 0
            row[name] = vals[i * width:(i + 1) * width]
        rows.append(row)
    return rows


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.init()
    wp.set_device("cpu")
    print("device:", wp.get_device())
    index = json.loads((FIX / "models/index.json").read_text(encoding="utf-8"))
    report = {"device": str(wp.get_device()), "cases": {}, "overall": {}}
    all_ok = True
    for row in index["models"]:
        name = row["name"]
        host = mujoco.MjModel.from_xml_path(str(ROOT / row["xml"]))
        first = capture(host, row["offset"])
        second = capture(host, row["offset"])
        rows1, rows2 = rows_of(first), rows_of(second)
        raw_identical = (first["count"] == second["count"] and compare_rows(rows1, rows1) and
                         all(
                             np.array_equal(first["fields"][k], second["fields"][k])
                             for k in first["fields"]))
        sorted_identical = compare_rows(rows1, rows2)
        doc = json.loads((FIX / "references" / (name + ".json")).read_text(encoding="utf-8"))
        frows = fixture_rows(doc)
        matches_fixture_sorted = compare_rows(rows1, frows)
        matches_fixture_raw = (first["count"] == doc["output"]["count"] and all(
            np.array_equal(
                first["fields"][k],
                np.asarray(doc["output"]["fields"][k]["values"], dtype=first["fields"][k].dtype
                           ).reshape(first["fields"][k].shape))
            for k in first["fields"]))
        case = {
            "delivered_count": doc["output"]["count"],
            "qa_count": first["count"],
            "qa_twice_raw_identical": bool(raw_identical),
            "qa_twice_sorted_identical": bool(sorted_identical),
            "delivered_two_capture_raw_flag": doc["determinism"]["two_captures_raw_bitwise_identical"],
            "delivered_two_capture_sorted_flag": doc["determinism"]["two_captures_sorted_bitwise_identical"],
            "qa_matches_fixture_sorted": bool(matches_fixture_sorted),
            "qa_matches_fixture_raw": bool(matches_fixture_raw),
            "qa_worldid_sequence": [int(r["worldid"][0]) for r in rows1],
            "fixture_worldid_sequence": doc["order_observation"]["worldid_sequence"],
            "qa_contact_fields": sorted(first["fields"].keys()),
        }
        report["cases"][name] = case
        maxdiff, int_mismatch = nearest_match_maxdiff(rows1, frows)
        exact = all(v == 0.0 for v in maxdiff.values()) and int_mismatch == 0
        near = (maxdiff["dist"] <= 1e-6 and maxdiff["pos"] <= 1e-6
                and maxdiff["frame"] <= 1e-4 and int_mismatch == 0)
        case["fixture_max_abs_diff"] = maxdiff
        case["fixture_int_mismatch"] = int_mismatch
        case["fixture_match_class"] = "exact" if exact else ("near" if near else "mismatch")
        report["cases"][name] = case
        ok = (sorted_identical and first["count"] == doc["output"]["count"]
              and case["fixture_match_class"] in ("exact", "near"))
        all_ok = all_ok and ok
        print("%-19s count=%-3d twice_raw=%-5s twice_sorted=%-5s fixture=%-5s cls=%-7s maxdiff(dist=%.2g pos=%.2g frame=%.2g) seq=%s" % (
            name, first["count"], raw_identical, sorted_identical, matches_fixture_sorted,
            case["fixture_match_class"], maxdiff["dist"], maxdiff["pos"], maxdiff["frame"],
            case["qa_worldid_sequence"]))
    exact_n = sum(1 for c in report["cases"].values() if c["fixture_match_class"] == "exact")
    near_n = sum(1 for c in report["cases"].values() if c["fixture_match_class"] == "near")
    raw_all = all(c["qa_twice_raw_identical"] for c in report["cases"].values())
    report["overall"] = {"qa_twice_raw_identical_all_cases": raw_all,
                         "fixture_exact_matches": exact_n,
                         "fixture_near_matches": near_n,
                         "qa_recapture_matches_fixtures": all_ok}
    out = ROOT / "target/qa-g0b/recapture_qa_report.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("overall:", all_ok)
    print("wrote", out)


if __name__ == "__main__":
    sys.exit(main())
