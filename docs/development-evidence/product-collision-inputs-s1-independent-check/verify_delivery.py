"""QA check of the S1 delivery record: every declared file's blob/sha256/bytes/lines."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
REC = ROOT / "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s1-data-snapshot.json"
OUT = ROOT / "target/qa-s1/verify_delivery.json"


def git_blob(path: Path) -> str:
    return subprocess.run(["git", "hash-object", str(path)], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()


def main():
    rec = json.loads(REC.read_text(encoding="utf-8"))
    report = {"task_id": rec["task_id"], "head_at_start": rec["head_at_start"], "files": []}
    problems = []
    for entry in rec["files"]:
        path = ROOT / entry["path"]
        raw = path.read_bytes()
        sha = hashlib.sha256(raw).hexdigest()
        lines = raw.count(b"\n")
        blob = git_blob(path)
        row = {"path": entry["path"], "sha256_ok": sha == entry["sha256"],
               "bytes_ok": len(raw) == entry["bytes"],
               "lines_ok": lines == entry["lines"], "blob_ok": blob == entry["blob"],
               "actual": {"sha256": sha, "bytes": len(raw), "lines": lines, "blob": blob}}
        if not all((row["sha256_ok"], row["bytes_ok"], row["lines_ok"], row["blob_ok"])):
            problems.append(row)
        report["files"].append(row)
    report["count"] = len(report["files"])
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"

    # git-blob identity of the delivery record itself vs HEAD tree
    head_blob = subprocess.run(["git", "rev-parse", "HEAD:" + str(REC.relative_to(ROOT)).replace("\\", "/")],
                               cwd=ROOT, capture_output=True, text=True).stdout.strip()
    report["record_blob_in_head"] = head_blob
    report["record_blob_worktree"] = git_blob(REC)
    OUT.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"files={report['count']} problems={len(problems)} result={report['result']}")
    for p in problems:
        print("PROBLEM:", p["path"], p["actual"])
    print("record blob head/worktree:", head_blob, report["record_blob_worktree"])
    print("wrote", OUT)


if __name__ == "__main__":
    main()
