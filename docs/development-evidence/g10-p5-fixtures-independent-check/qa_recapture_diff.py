"""QA independent G0b review: quantify CPU-recapture vs delivered CUDA fixture differences."""

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


def f32_bits(v):
    return struct.unpack("<I", struct.pack("<f", float(v)))[0]


def ulp(a, b):
    return abs(f32_bits(a) - f32_bits(b))


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
    return {"count": count, "fields": fields}


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


def match_rows(rows_qa, rows_fix):
    """Greedy match by (worldid, geom, geomcollisionid) and nearest dist."""
    used = set()
    pairs = []
    for qa in rows_qa:
        best = None
        for idx, fx in enumerate(rows_fix):
            if idx in used:
                continue
            if int(qa["worldid"][0]) != int(fx["worldid"][0]):
                continue
            if [int(v) for v in qa["geom"]] != [int(v) for v in fx["geom"]]:
                continue
            score = abs(float(qa["dist"][0]) - float(fx["dist"][0]))
            if best is None or score < best[0]:
                best = (score, idx)
        if best is None:
            pairs.append((qa, None))
        else:
            used.add(best[1])
            pairs.append((qa, rows_fix[best[1]]))
    return pairs


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.init()
    wp.set_device("cpu")
    index = json.loads((FIX / "models/index.json").read_text(encoding="utf-8"))
    report = {"cases": {}, "overall": {}}
    worst_dist_ulp = 0
    worst_pos_ulp = 0
    all_matched = True
    for row in index["models"]:
        name = row["name"]
        host = mujoco.MjModel.from_xml_path(str(ROOT / row["xml"]))
        qa = capture(host, row["offset"])
        doc = json.loads((FIX / "references" / (name + ".json")).read_text(encoding="utf-8"))
        frows = fixture_rows(doc)
        qrows = []
        for i in range(qa["count"]):
            r = {f: (arr[i].tolist() if arr.shape[1] else []) for f, arr in qa["fields"].items()}
            qrows.append(r)
        pairs = match_rows(qrows, frows)
        unmatched = sum(1 for _, fx in pairs if fx is None)
        per_field = {f: {"max_abs": 0.0, "max_ulp": 0} for f in FLOAT_FIELDS}
        int_mismatch = {f: 0 for f in INT_FIELDS}
        for a, b in pairs:
            if b is None:
                continue
            for f in FLOAT_FIELDS:
                if not a[f] or not b[f]:
                    continue
                for x, y in zip(a[f], b[f]):
                    per_field[f]["max_abs"] = max(per_field[f]["max_abs"], abs(float(x) - float(y)))
                    per_field[f]["max_ulp"] = max(per_field[f]["max_ulp"], ulp(x, y))
            for f in INT_FIELDS:
                if [int(v) for v in a[f]] != [int(v) for v in b[f]]:
                    int_mismatch[f] += 1
        # order-insensitive count of distinct geom pairs and world counts
        report["cases"][name] = {
            "qa_count": qa["count"], "delivered_count": doc["output"]["count"],
            "unmatched_rows": unmatched,
            "float_field_diffs": per_field,
            "int_field_mismatches": int_mismatch,
            "qa_geomcollisionid_per_world": {
                str(w): sorted(int(r["geomcollisionid"][0]) for r in qrows if int(r["worldid"][0]) == w)
                for w in (0, 1)},
            "delivered_geomcollisionid_per_world": {
                str(w): sorted(int(r["geomcollisionid"][0]) for r in frows if int(r["worldid"][0]) == w)
                for w in (0, 1)},
        }
        if unmatched:
            all_matched = False
        worst_dist_ulp = max(worst_dist_ulp, per_field["dist"]["max_ulp"])
        worst_pos_ulp = max(worst_pos_ulp, per_field["pos"]["max_ulp"])
        print("%-19s unmatched=%-2d dist(max_abs=%.3g,max_ulp=%d) pos(max_abs=%.3g,max_ulp=%d) frame_ulp=%d int_mismatch=%s" % (
            name, unmatched, per_field["dist"]["max_abs"], per_field["dist"]["max_ulp"],
            per_field["pos"]["max_abs"], per_field["pos"]["max_ulp"],
            per_field["frame"]["max_ulp"],
            {k: v for k, v in int_mismatch.items() if v}))
    report["overall"] = {"all_rows_matched": all_matched,
                         "worst_dist_ulp": worst_dist_ulp, "worst_pos_ulp": worst_pos_ulp}
    out = ROOT / "target/qa-g0b/recapture_diff_report.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("overall:", json.dumps(report["overall"]))
    print("wrote", out)


if __name__ == "__main__":
    sys.exit(main())
