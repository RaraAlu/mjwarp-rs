"""QA G0b review: detect duplicate contacts (same dist+pos+frame) in the new references."""

import json
import struct
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
REF = ROOT / "fixtures/g10-p5/references"
CASES = ["sphere-box-shallow", "sphere-box-deep", "capsule-box", "cylinder-box",
         "ellipsoid-box", "hfield-sphere", "hfield-capsule", "sdf-box",
         "mesh-mesh", "box-box-margin"]


def bits(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def rows(doc):
    fields = doc["output"]["fields"]
    count = doc["output"]["count"]
    out = []
    for i in range(count):
        row = {}
        for name, rec in fields.items():
            vals = rec["values"]
            width = len(vals) // count if count else 0
            row[name] = vals[i * width:(i + 1) * width]
        out.append(row)
    return out


def key(row):
    return (int(row["worldid"][0]), bits(row["dist"]), bits(row["pos"]), bits(row["frame"]),
            tuple(int(v) for v in row["geom"]))


def main():
    report = {}
    for name in CASES:
        doc = json.loads((REF / (name + ".json")).read_text(encoding="utf-8"))
        rs = rows(doc)
        groups = {}
        for r in rs:
            groups.setdefault(key(r), []).append(r)
        dup_groups = {k: len(v) for k, v in groups.items() if len(v) > 1}
        # dist+pos only (ignore frame) duplicates
        dp = {}
        for r in rs:
            dp.setdefault((int(r["worldid"][0]), bits(r["dist"]), bits(r["pos"])), []).append(r)
        dup_dp = {k: len(v) for k, v in dp.items() if len(v) > 1}
        report[name] = {
            "count": len(rs),
            "unique_rows_all_fields": len(groups),
            "duplicate_groups_same_dist_pos_frame": len(dup_groups),
            "duplicate_rows_extra_copy_count": sum(v - 1 for v in dup_groups.values()),
            "duplicate_groups_same_dist_pos_only": len(dup_dp),
        }
        print("%-19s count=%-2d unique=%-2d dup_(dist,pos,frame) groups=%-2d extra_copies=%-2d dup_(dist,pos) groups=%d" % (
            name, len(rs), len(groups), len(dup_groups),
            sum(v - 1 for v in dup_groups.values()), len(dup_dp)))
    (ROOT / "target/qa-g0b/duplicate_contacts_report.json").write_text(
        json.dumps(report, indent=1), encoding="utf-8")


if __name__ == "__main__":
    main()
