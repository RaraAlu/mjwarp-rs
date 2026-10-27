"""QA independent checks for S3: receipt s3 section, provenance, unchanged paths, scope, terms."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s3/verify_s3_checks.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout


def main():
    report = {}
    receipt = json.loads(
        (ROOT / "docs/development-evidence/product-collision-inputs-wiring/receipt.json").read_text(encoding="utf-8")
    )
    s3 = receipt.get("s3", {})
    check(receipt.get("task_id") == "2026-10-27-product-collision-inputs-s1-data-snapshot",
          "top-level receipt task_id changed")
    check(isinstance(receipt.get("s2"), dict) and receipt["s2"].get("task_id") ==
          "2026-10-27-product-collision-inputs-s2-adapter", "s2 task_id changed")
    check(s3.get("task_id") == "2026-10-27-product-collision-inputs-s3-entry-window", "s3 task_id")
    report["s3_keys"] = sorted(s3.keys())

    # any {path, sha256} entries inside the s3 section must match the files
    found, bad = [], []

    def walk(obj):
        if isinstance(obj, dict):
            if "path" in obj and "sha256" in obj:
                found.append(obj)
            for value in obj.values():
                walk(value)
        elif isinstance(obj, list):
            for value in obj:
                walk(value)

    walk(s3)
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
    check(not bad, f"receipt s3 deliverable mismatches: {bad}")
    report["s3_path_entries"] = len(found)

    # provenance.json: versions, MJB entries, 32 arrays x 3 cases
    prov = json.loads(
        (ROOT / "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/provenance.json")
        .read_text(encoding="utf-8")
    )
    report["provenance_keys"] = sorted(prov.keys())[:10]
    text = json.dumps(prov)
    check("3.12.0" in text, "provenance does not mention mujoco 3.12.0")
    cases = prov.get("cases") or prov.get("mjb_cases") or {}
    if isinstance(cases, dict):
        case_names = list(cases)
    elif isinstance(cases, list):
        case_names = [c.get("case") or c.get("name") for c in cases]
    else:
        case_names = []
    report["provenance_cases"] = case_names
    check(sorted(case_names) == ["mesh-box", "mesh-cylinder", "mesh-mesh"], f"provenance cases {case_names}")
    arrays = 0
    mis = []
    for name in case_names:
        case = cases[name]
        arrays += len(case.get("arrays", {}))
        for field, entry in case.get("arrays", {}).items():
            bin_path = ROOT / entry.get("bin", "") if entry.get("bin") else None
            if bin_path is not None and bin_path.is_file():
                raw = bin_path.read_bytes()
                if hashlib.sha256(raw).hexdigest() != entry.get("sha256"):
                    mis.append(f"{name}/{field}")
    report["provenance_arrays"] = arrays
    check(arrays == 96, f"provenance arrays {arrays}")
    check(not mis, f"provenance array sha mismatches: {mis}")

    # unchanged paths
    unchanged = [
        "include/mjwarp_native_probe.h", "native/g01_fields.inc", "native/model_probe.cpp",
        "src/io/fields.rs", "src/io/flex.rs", "src/io/g01.rs", "src/io/inertial.rs",
        "src/io/kinematic.rs", "src/model/attached.rs", "src/model/camlight.rs",
        "src/model/flex.rs", "src/model/inertial.rs", "src/model/mocap.rs",
        "src/model/parameters.rs", "src/model/sleep.rs", "src/model/spatial_tendon.rs",
        "src/model/tendon.rs", "src/model/topology.rs", "src/runtime/collision_gpu/abi.rs",
        "src/physics/collision/cuda/", "fixtures/g10-p5-mesh-multicontact/",
        "fixtures/g10-p5-primitive-branches/", "docs/development-evidence/mesh-multicontact-m0b/",
        "docs/development-evidence/mesh-multicontact-m1/", "docs/development-evidence/mesh-multicontact-m2/",
        "docs/development-evidence/mesh-multicontact-m3/",
    ]
    changed = git("diff", "41fa049c", "0c9497e4", "--name-only", "--", *unchanged).strip()
    check(changed == "", f"unchanged paths changed: {changed}")
    report["unchanged_paths"] = len(unchanged)
    report["unchanged_changed"] = len([line for line in changed.splitlines() if line.strip()])

    # scope of the S3 commit
    declared = [
        "src/runtime/collision_gpu.rs", "src/runtime/collision_gpu/", "src/runtime/backend.rs",
        "tests/g10_p5_product_collision_wiring_cuda.rs", "tests/support/",
        "docs/development-evidence/product-collision-inputs-wiring/",
        "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-entry-window.json",
    ]
    files = [line for line in git("show", "--name-only", "--format=", "0c9497e4").splitlines() if line.strip()]
    out_of_scope = [f for f in files if not (f.startswith(".normify/") or any(f == d or f.startswith(d) for d in declared))]
    check(not out_of_scope, f"out of scope: {out_of_scope}")
    report["commit_files"] = len(files)
    report["out_of_scope"] = out_of_scope

    # forbidden terms in files the S3 commit changed
    task = json.loads(
        (ROOT / ".normify/changes/2026-10-27-product-collision-inputs-s3-independent-check.json").read_text(encoding="utf-8")
    )
    terms = [t for c in task["checks"] if c["kind"] == "forbid_claim" for t in c["terms"]]
    hits = []
    for rel in files:
        if not rel.startswith(("src/", "tests/", "docs/")):
            continue
        if Path(rel).suffix not in (".rs", ".md", ".json", ".py", ".log"):
            continue
        path = ROOT / rel
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for term in terms:
            if term in text:
                hits.append(f"{rel}:{term}")
    report["term_hits_in_s3_changed_files"] = hits
    check(not hits, f"term hits: {hits}")

    # reference files still at their frozen shas (support constants)
    refs = {
        "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json":
            "c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a",
        "docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json":
            "96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1",
    }
    for rel, expected in refs.items():
        actual = hashlib.sha256((ROOT / rel).read_bytes()).hexdigest()
        check(actual == expected, f"reference sha changed: {rel}")

    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("s3 keys:", report["s3_keys"])
    print("provenance cases:", report["provenance_cases"], "arrays:", report["provenance_arrays"])
    print("unchanged:", report["unchanged_paths"], "commit files:", report["commit_files"])


if __name__ == "__main__":
    main()
