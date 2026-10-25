import json
from pathlib import Path

QA = Path(r"D:\Rust\mjwarp-rs\target\qa-g0b")
r = json.loads((QA / "references_qa_report.json").read_text(encoding="utf-8"))
rc = json.loads((QA / "recapture_qa_report.json").read_text(encoding="utf-8"))
p = json.loads((QA / "parity_qa_report.json").read_text(encoding="utf-8"))
for name, c in r["cases"].items():
    ct = c["contact"]
    print("%-19s count=%-2d per_world=%-11s pairs=%-18s dist=[%.6f,%.6f] types=%s" % (
        name, ct["count"], str(ct["per_world"]), str(ct["pairs"]),
        ct["dist_min"], ct["dist_max"], ct["pair_types"]))
print()
for name, c in rc["cases"].items():
    md = c["fixture_max_abs_diff"]
    print("%-19s class=%-7s exact=%s maxdiff dist=%.2g pos=%.2g frame=%.2g qa_seq=%s" % (
        name, c["fixture_match_class"], c["qa_matches_fixture_sorted"],
        md["dist"], md["pos"], md["frame"], c["qa_worldid_sequence"]))
print()
for model, row in p["models"].items():
    print(model, "all_fields_equal=", row["all_fields_equal"],
          "unequal=", row["unequal_fields"],
          "side=", row["side_consistency"],
          "delivered_audit=", row["delivered_parity_audit"]["all_delivered_fields_match_qa"])
