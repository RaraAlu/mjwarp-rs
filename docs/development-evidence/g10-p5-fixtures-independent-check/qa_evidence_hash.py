"""QA independent G0b review: verify delivered evidence hashes, declared artifact digests and scope claims."""

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/g10-p5-fixtures"
FIX = ROOT / "fixtures/g10-p5"
COMMIT = "f0e43bb"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT)] + list(args), capture_output=True)


def git_bytes(path, commit=COMMIT):
    res = git("show", "%s:%s" % (commit, path))
    if res.returncode != 0:
        return None
    return res.stdout


def check(reg, name, ok, detail=None):
    reg.append({"check": name, "ok": bool(ok), "detail": detail})
    if not ok:
        print("FAIL:", name, detail)


def main():
    receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
    reg = []
    # 1) evidence files byte-identical to the commit blobs
    for name in ["README.md", "receipt.json", "gpu-run.log"]:
        blob = git_bytes("docs/development-evidence/g10-p5-fixtures/" + name)
        local = (EV / name).read_bytes()
        check(reg, "evidence_blob_equal:" + name, blob == local,
              None if blob == local else {"local_bytes": len(local), "blob_bytes": None if blob is None else len(blob)})
    # 2) receipt-declared artifact digests
    for rel, declared in [
        ("fixtures/g10-p5/octree-parity.json", receipt["mjb_and_parity"]["octree_parity_sha256"]),
        ("fixtures/g10-p5/existing-reference-audit.json", receipt["existing_reference_audit"]["sha256"]),
    ]:
        actual = sha256(ROOT / rel)
        check(reg, "receipt_sha:" + rel, actual == declared, {"declared": declared, "actual": actual})
    models = receipt["mjb_and_parity"]["models"]
    for model, row in models.items():
        for version, meta in [("mjb_312", row["mjb_312"]), ("mjb_315", row["mjb_315"])]:
            path = ROOT / meta["path"]
            check(reg, "mjb:%s:%s:bytes" % (model, version), path.stat().st_size == meta["bytes"],
                  {"declared": meta["bytes"], "actual": path.stat().st_size})
            check(reg, "mjb:%s:%s:sha" % (model, version), sha256(path) == meta["sha256"],
                  {"declared": meta["sha256"], "actual": sha256(path)})
        check(reg, "source_xml_sha:" + model, sha256(ROOT / row["source_xml"]) == row["source_xml_sha256"],
              {"declared": row["source_xml_sha256"], "actual": sha256(ROOT / row["source_xml"])})
        noct = row["noct"]
        check(reg, "noct:%s" % model, noct["312"] == noct["315"] == (63497 if model == "sdf-cylinder" else 0), noct)
    # 3) references index
    idx = receipt["captures"]["index"]
    refidx = FIX / "references/index.json"
    check(reg, "references_index:bytes", refidx.stat().st_size == idx["bytes"],
          {"declared": idx["bytes"], "actual": refidx.stat().st_size})
    check(reg, "references_index:sha", sha256(refidx) == idx["sha256"],
          {"declared": idx["sha256"], "actual": sha256(refidx)})
    doc = json.loads(refidx.read_text(encoding="utf-8"))
    for row in doc["cases"]:
        path = ROOT / row["file"]
        check(reg, "ref_case_sha:" + row["name"], sha256(path) == row["sha256"],
              {"declared": row["sha256"], "actual": sha256(path)})
    # 4) models index
    midx = receipt["artifacts"]["models_index"]
    check(reg, "models_index:bytes", (FIX / "models/index.json").stat().st_size == midx["bytes"],
          {"declared": midx["bytes"], "actual": (FIX / "models/index.json").stat().st_size})
    check(reg, "models_index:sha", sha256(FIX / "models/index.json") == midx["sha256"],
          {"declared": midx["sha256"], "actual": sha256(FIX / "models/index.json")})
    # 5) scripts: lines/bytes/sha
    script_dir = ROOT / "scripts/authoring/g10-p5"
    for name, meta in receipt["artifacts"]["scripts"].items():
        path = script_dir / name
        raw = path.read_bytes()
        lines = raw.count(b"\n") + (0 if raw.endswith(b"\n") else 1)
        check(reg, "script:lines:" + name, lines == meta["lines"], {"declared": meta["lines"], "actual": lines})
        check(reg, "script:bytes:" + name, len(raw) == meta["bytes"], {"declared": meta["bytes"], "actual": len(raw)})
        check(reg, "script:sha:" + name, sha256(path) == meta["sha256"],
              {"declared": meta["sha256"], "actual": sha256(path)})
    # 6) README MJB table prefixes
    readme = (EV / "README.md").read_text(encoding="utf-8")
    for model, row in models.items():
        for version in ["mjb_312", "mjb_315"]:
            prefix = row[version]["sha256"][:8]
            check(reg, "readme_prefix:%s:%s" % (model, version), prefix in readme, prefix)
        check(reg, "readme_noct:%s" % model,
              ("%d" % row["noct"]["312"]) in readme or format(row["noct"]["312"], ",") in readme,
              row["noct"]["312"])
    # 7) gpu log: memory baseline and no task compute process
    log = (EV / "gpu-run.log").read_text(encoding="utf-8", errors="replace")
    gpu = receipt["gpu"]
    check(reg, "gpu:start_memory", ("%d MiB" % gpu["window"]["start_memory_mib"]) in log, gpu["window"])
    before, after = log.split("=== window: end", 1)[0], log.split("=== window: end", 1)[1]
    check(reg, "gpu:no_python_in_log", "python" not in after.lower(), None)
    # process list equality: before == after on the pid/name lines that appear in both blocks
    def proc_lines(block):
        return [ln.strip() for ln in block.splitlines() if ln.strip() and "," in ln and "MiB" not in ln
                and not ln.startswith("===") and not ln.startswith("name,") and not ln.startswith("pid,")
                and "command" not in ln and not ln.startswith("[g0b]")]
    try:
        head = log.split("=== command:", 1)[0]
        tail = log.split("=== device after ===", 1)[1]
        import re as _re

        def proc_lines(block):
            return sorted(ln.strip().replace(", [N/A]", "") for ln in block.splitlines()
                          if _re.match(r"^\d+,\s", ln.strip()))

        a_pre = proc_lines(head)
        a_post = proc_lines(tail)
        check(reg, "gpu:process_list_before_after_equal", a_pre == a_post,
              {"before_n": len(a_pre), "after_n": len(a_post),
               "only_before": sorted(set(a_pre) - set(a_post))[:5],
               "only_after": sorted(set(a_post) - set(a_pre))[:5]})
    except Exception as exc:  # noqa: BLE001
        check(reg, "gpu:process_list_before_after_equal", False, str(exc))
    # 8) unmet items
    check(reg, "receipt_unmet_count", len(receipt.get("unmet", [])) == 7, len(receipt.get("unmet", [])))
    check(reg, "receipt_task_id", receipt.get("task_id") == "2026-10-25-g10-p5-fixtures", receipt.get("task_id"))
    # 9) protected paths untouched by the commit
    diff = git("diff", "--name-only", "87bc042.." + COMMIT).stdout.decode("utf-8", "replace").splitlines()
    allowed = ("scripts/authoring/g10-p5/", "fixtures/g10-p5/", "docs/development-evidence/g10-p5-fixtures/", ".normify/")
    outside = [p for p in diff if p and not p.startswith(allowed)]
    check(reg, "commit_scope_only_g0b_paths", outside == [], outside)
    check(reg, "commit_no_src_tests_cargo",
          not any(p.startswith(("src/", "tests/", "Cargo")) for p in diff), diff[:5])
    check(reg, "commit_no_d04_evidence",
          not any(p.startswith("docs/evidence/d04-complex-physics/") for p in diff), None)
    check(reg, "commit_no_render_evidence",
          not any(p.startswith("docs/development-evidence/render-bvh/") for p in diff), None)
    # 10) complex-physics fixtures unmodified in the commit and in worktree vs HEAD
    check(reg, "commit_no_complex_physics_xml",
          not any(p.startswith("fixtures/mjwarp-complex-physics/") for p in diff), None)
    status = git("status", "--porcelain", "fixtures/mjwarp-complex-physics").stdout.decode().strip()
    check(reg, "worktree_complex_physics_clean", status == "", status)
    # summary
    failed = [r for r in reg if not r["ok"]]
    out = {"checks": reg, "failed": failed, "all_ok": not failed}
    (ROOT / "target/qa-g0b/evidence_hash_report.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    print("checks=%d failed=%d" % (len(reg), len(failed)))
    for r in failed:
        print("  ", r["check"], r["detail"])
    return 0 if not failed else 1


def declared_key(receipt):
    return "existing_reference_audit"


if __name__ == "__main__":
    sys.exit(main())
