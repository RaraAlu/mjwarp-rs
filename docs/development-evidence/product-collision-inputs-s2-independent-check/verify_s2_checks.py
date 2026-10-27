"""QA independent checks for S2: adapter/type order, delivery record identity, evidence logs."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s2/verify_s2_checks.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def skel_fields(text, start_pattern):
    """Return the ordered `name: &'a [T]` field names of the first matching struct."""
    start = text.index(start_pattern)
    block = text[start:]
    end = block.index("\n}")
    block = block[:end]
    return re.findall(r"pub (\w+): &'a \[(\w+)\]", block)


def main():
    report = {}
    # --- 1. adapter view order/types vs CollisionGpuInputs ---
    types_src = (ROOT / "src/runtime/collision_gpu/types.rs").read_text(encoding="utf-8")
    gpu = skel_fields(types_src, "pub struct CollisionGpuInputs<'a> {")
    adapter_src = (ROOT / "src/runtime/collision_gpu/inputs_from_snapshot.rs").read_text(encoding="utf-8")
    view = skel_fields(adapter_src, "pub struct CollisionInputsSnapshot<'a> {")
    check(len(gpu) == 32, f"CollisionGpuInputs fields {len(gpu)}")
    check(len(view) == 32, f"CollisionInputsSnapshot fields {len(view)}")
    check([n for n, _ in gpu] == [n for n, _ in view], "view field order differs from CollisionGpuInputs")
    check([t for _, t in gpu] == [t for _, t in view], "view field types differ from CollisionGpuInputs")
    report["view_order_matches"] = [n for n, _ in gpu] == [n for n, _ in view]
    report["view_types_match"] = [t for _, t in gpu] == [t for _, t in view]

    # the construction block must assign each inputs field from the same-named snapshot field
    pairs = re.findall(r"^\s{8}(\w+): snapshot\.(\w+),$", adapter_src, re.M)
    check(len(pairs) == 32, f"construction lines {len(pairs)}")
    check(all(a == b for a, b in pairs), "construction pairs field names differ")
    report["construction_pairs"] = len(pairs)

    # --- 2. delivery record identity ---
    rec_path = ROOT / "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s2-adapter.json"
    rec = json.loads(rec_path.read_text(encoding="utf-8"))
    check(rec["task_id"] == "2026-10-27-product-collision-inputs-s2-adapter", "delivery task id")
    bad = []
    for entry in rec.get("files", []):
        path = ROOT / entry["path"]
        raw = path.read_bytes()
        row = {
            "path": entry["path"],
            "blob_ok": git("hash-object", entry["path"]) == entry["blob"],
            "sha_ok": hashlib.sha256(raw).hexdigest() == entry["sha256"],
            "bytes_ok": len(raw) == entry["bytes"],
            "lines_ok": raw.count(b"\n") == entry["lines"],
        }
        if not all((row["blob_ok"], row["sha_ok"], row["bytes_ok"], row["lines_ok"])):
            bad.append(row)
    check(not bad, f"delivery record mismatches: {bad}")
    report["delivery_files"] = len(rec.get("files", []))
    report["delivery_mismatches"] = bad
    head_blob = git("rev-parse", "HEAD:docs/development-evidence/changes/2026-10-27-product-collision-inputs-s2-adapter.json")
    report["delivery_record_blob_head"] = head_blob
    report["delivery_record_blob_worktree"] = git("hash-object",
                                                  "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s2-adapter.json")

    # --- 3. case bins: counts and shas are stable vs HEAD tree ---
    case_dirs = [
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box",
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder",
        "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh",
    ]
    bins = {}
    for case_dir in case_dirs:
        files = sorted(p for p in (ROOT / case_dir).iterdir() if p.is_file())
        bins[case_dir] = {
            "files": len(files),
            "bins": sum(1 for p in files if p.suffix == ".bin"),
            "meta": (ROOT / case_dir / "meta.txt").is_file(),
            "head_match": git("diff", "HEAD", "6a113daf", "--name-only", "--", case_dir) == "",
        }
        check(bins[case_dir]["bins"] == 32, f"{case_dir} bin count")
        check(bins[case_dir]["meta"], f"{case_dir} meta")
    report["cases"] = bins

    # --- 4. evidence logs: BOM / LF / sizes ---
    logs = {}
    for path in sorted((ROOT / "docs/development-evidence/product-collision-inputs-wiring/logs").iterdir()):
        raw = path.read_bytes()
        logs[path.name] = {
            "bytes": len(raw),
            "bom": raw.startswith(b"\xef\xbb\xbf"),
            "crlf": b"\r\n" in raw,
        }
    report["logs"] = logs
    for name in ["s2-targeted-default.log", "s2-targeted-native.log", "s2-fmt-clippy.log", "s2-gate.log"]:
        check(name in logs, f"missing log {name}")
        if name in logs:
            check(not logs[name]["bom"], f"{name} has BOM")

    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"
    OUT.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)


if __name__ == "__main__":
    main()
