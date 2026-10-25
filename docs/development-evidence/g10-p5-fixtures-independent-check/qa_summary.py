import json
from pathlib import Path

QA = Path(r"D:\Rust\mjwarp-rs\target\qa-g0b")
p = json.loads((QA / "parity_qa_report.json").read_text(encoding="utf-8"))
s = p["models"]["sdf-cylinder"]["octree_structure"]
print("octree:", json.dumps({k: v for k, v in s.items() if k != "depth_histogram"}, ensure_ascii=False))
print("hist:", s["depth_histogram"])
print("sdf field bytes:", {k: v["bytes_a"] for k, v in p["models"]["sdf-cylinder"]["field_rows"].items()})
print("hfield field bytes:", {k: v["bytes_a"] for k, v in p["models"]["hfield-box"]["field_rows"].items()})
print("parity overall:", p["overall"])
r = json.loads((QA / "references_qa_report.json").read_text(encoding="utf-8"))
rc = json.loads((QA / "recapture_qa_report.json").read_text(encoding="utf-8"))
print("ref overall:", r["overall"])
print("recapture overall:", rc["overall"])
print("classes:", {n: c.get("fixture_match_class") for n, c in rc["cases"].items()})
print("model flags:", json.dumps(r["model_flags"], ensure_ascii=False))
a = json.loads((QA / "audit_qa_report.json").read_text(encoding="utf-8"))
print("audit overall:", a["overall"])
print("case_set:", a["case_set"], "gaps:", len(a["delivered_meta"]["gaps"]))
e = json.loads((QA / "evidence_hash_report.json").read_text(encoding="utf-8"))
print("evidence checks:", len(e["checks"]), "failed:", len(e["failed"]))
print("gpu checks:", [x["check"] for x in e["checks"] if x["check"].startswith("gpu")])
