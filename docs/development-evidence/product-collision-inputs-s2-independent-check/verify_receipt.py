"""QA checks: S2 receipt deliverable identities, M0b manifest array order, gate run json."""
import hashlib
import json
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def main():
    receipt = json.loads(
        (ROOT / "docs/development-evidence/product-collision-inputs-wiring/receipt.json").read_text(encoding="utf-8")
    )
    s2 = receipt["s2"]
    found = []

    def walk(obj):
        if isinstance(obj, dict):
            if "path" in obj and "sha256" in obj:
                found.append(obj)
            for value in obj.values():
                walk(value)
        elif isinstance(obj, list):
            for value in obj:
                walk(value)

    walk(s2)
    bad = []
    for entry in found:
        path = ROOT / entry["path"]
        raw = path.read_bytes()
        ok = hashlib.sha256(raw).hexdigest() == entry["sha256"]
        if "bytes" in entry:
            ok = ok and len(raw) == entry["bytes"]
        if "lines" in entry:
            ok = ok and raw.count(b"\n") == entry["lines"]
        if not ok:
            bad.append(entry["path"])
    check(not bad, f"receipt s2 deliverable mismatches: {bad}")
    check(receipt["task_id"] == "2026-10-27-product-collision-inputs-s1-data-snapshot",
          "receipt top-level task_id changed")
    check(s2["task_id"] == "2026-10-27-product-collision-inputs-s2-adapter", "receipt s2 task_id")

    # M0b manifest arrays_order == CollisionGpuInputs order
    manifest = json.loads(
        (ROOT / "docs/development-evidence/mesh-multicontact-m0b/mesh-cases-manifest.json").read_text(encoding="utf-8")
    )
    order = manifest["arrays_order"]
    check(len(order) == 32, f"M0b arrays_order len {len(order)}")
    report = {
        "receipt_s2_entries": len(found),
        "receipt_s2_mismatches": bad,
        "m0b_arrays_order": order,
        "gate_unit": json.loads(
            (ROOT / ".normify/runs/2026-10-27-product-collision-inputs-s2-adapter/unit.json").read_text(encoding="utf-8")
        ),
        "receipt_s2_gates": s2.get("gates"),
        "receipt_s2_parity": s2.get("parity"),
        "checks": 0,
        "problems": problems,
    }
    # M0b manifest per-case array shas vs the actual bins
    m0b_bad = []
    for case_name, case in manifest["cases"].items():
        for field, entry in case["arrays"].items():
            path = ROOT / "docs/development-evidence/mesh-multicontact-m0b/cases" / case_name / entry["file"]
            raw = path.read_bytes()
            check(len(raw) // 4 == entry["count"], f"{case_name}/{field} count")
            check(hashlib.sha256(raw).hexdigest() == entry["sha256"], f"{case_name}/{field} sha")
            if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
                m0b_bad.append(f"{case_name}/{field}")
    report["m0b_bin_sha_mismatches"] = m0b_bad
    report["checks"] = checks
    report["result"] = "PASS" if not problems else "FAIL"
    out = ROOT / "target/qa-s2/verify_receipt.json"
    out.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("receipt s2 entries:", len(found))
    print("gate unit:", report["gate_unit"]["tests"])
    print("m0b order first/last:", order[0], order[-1])


if __name__ == "__main__":
    main()
