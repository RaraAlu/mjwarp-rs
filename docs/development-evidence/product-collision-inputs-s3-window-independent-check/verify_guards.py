"""QA independent verification of the S3 window guards, freeze sources and scope."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/s3-product-wiring/attempt1"
FREEZE = ROOT / "target/s3-product-wiring-freeze"
OUT = ROOT / "target/qa-s3win/verify_guards.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout


def main():
    guard_lines = (ATT / "sha-guards.txt").read_text(encoding="utf-8").splitlines()
    fact_lines = (ATT / "facts.txt").read_text(encoding="utf-8").splitlines()
    exe_rows, input_rows = [], []
    bad = []
    for line in guard_lines:
        if not line.startswith("SHA_OK "):
            bad.append(f"malformed line: {line[:80]}")
            continue
        fields = dict(item.split("=", 1) for item in line.split() if "=" in item)
        if line.startswith("SHA_OK exe "):
            path = FREEZE / fields["file"]
            row = {"kind": "exe", "target": fields["target"], "file": fields["file"],
                   "bytes": int(fields["bytes"]), "sha256": fields["sha256"], "expected": fields["expected"],
                   "match": fields["match"] == "True"}
            exe_rows.append(row)
        else:
            path = ROOT / fields["path"]
            row = {"kind": "input", "path": fields["path"], "sha256": fields["sha256"],
                   "expected": fields["expected"], "match": fields["match"] == "True"}
            input_rows.append(row)
        if not path.is_file():
            bad.append(f"missing {path}")
            continue
        actual = sha(path)
        if actual != fields["sha256"] or actual != fields["expected"] or fields["match"] != "True":
            bad.append(f"sha mismatch {path}")
        if row["kind"] == "exe" and path.stat().st_size != row["bytes"]:
            bad.append(f"exe bytes mismatch {path}")

    fact_guard = [line for line in fact_lines if line.startswith("SHA_OK ")]
    check(len(guard_lines) == 133, f"sha-guards lines {len(guard_lines)}")
    check(len(exe_rows) == 10, f"exe rows {len(exe_rows)}")
    check(len(input_rows) == 123, f"input rows {len(input_rows)}")
    check(not bad, f"guard problems: {bad[:5]} (total {len(bad)})")
    check(len(fact_guard) == 133, f"facts SHA_OK lines {len(fact_guard)}")
    check(set(fact_guard) == set(guard_lines), "facts SHA_OK lines identical to sha-guards.txt")
    check(all(row["match"] for row in exe_rows + input_rows), "all match=True")

    # post-window exe hashes (current freeze dir) vs guards / receipt
    post_bad = []
    for row in exe_rows:
        path = FREEZE / row["file"]
        if sha(path) != row["expected"] or path.stat().st_size != row["bytes"]:
            post_bad.append(row["file"])
    check(not post_bad, f"post-window exe mismatches: {post_bad}")

    # frozen-shas.txt sources and bytes
    source_counts = {}
    frozen_rows = {}
    for line in (FREEZE / "frozen-shas.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or "|" not in line:
            continue
        parts = [p.strip() for p in line.split("|")]
        file, size, mtime, digest, copy_match = parts[0], int(parts[1]), parts[2], parts[3], parts[4]
        source = next((p.split("=", 1)[1] for p in parts if p.startswith("source=")), "")
        source_counts[source] = source_counts.get(source, 0) + 1
        frozen_rows[file] = {"bytes": size, "sha256": digest, "source": source, "copy_match": copy_match}
    check(len(frozen_rows) == 10, f"frozen-shas rows {len(frozen_rows)}")
    check(source_counts.get("s3-build(all-features)") == 1, f"s3-build count {source_counts}")
    check(source_counts.get("s4-build(2026-10-27T10:17:56,all-features)") == 1, "s4-build count")
    check(source_counts.get("m3-freeze(M3-signed,all-features)") == 8, "m3-freeze count")
    check(all(row["copy_match"] == "copy_match=True" for row in frozen_rows.values()), "copy_match all True")
    for row in exe_rows:
        frozen = frozen_rows.get(row["file"], {})
        check(frozen.get("sha256") == row["expected"] and frozen.get("bytes") == row["bytes"],
              f"frozen row mismatch {row['file']}")

    # runner identity
    runner = ROOT / "target/s3-product-wiring/window/run-validation-attempt1.ps1"
    receipt = json.loads((ROOT / "docs/development-evidence/product-collision-inputs-wiring/window-receipt.json").read_text(encoding="utf-8"))
    check(runner.stat().st_size == 33221, f"runner bytes {runner.stat().st_size}")
    check(sha(runner) == "c7a387a423312e4db55d0619a20341426b71561f7f1b1ab3129ed2f3c19a103d", "runner sha")
    check(receipt["runner"]["bytes"] == 33221 and receipt["runner"]["sha256"] == sha(runner), "receipt runner matches")

    # scope: no code changes between the S3 implementation commit and HEAD (the window commits)
    code_paths = ["src", "tests", "native", "include", "fixtures", "Cargo.toml", "Cargo.lock", "build.rs"]
    changed = [line for line in git("diff", "0c9497e4", "HEAD", "--name-only", "--", *code_paths).splitlines() if line.strip()]
    check(not changed, f"code changed after 0c9497e4: {changed[:5]}")
    window_commit_files = [line for line in git("show", "--name-only", "--format=", "45e07485").splitlines() if line.strip()]
    fix_commit_files = [line for line in git("show", "--name-only", "--format=", "fa704a6e").splitlines() if line.strip()]
    code_hits = [f for f in window_commit_files + fix_commit_files
                 if f.startswith(("src/", "tests/", "native/", "include/", "fixtures/")) or f in ("Cargo.toml", "Cargo.lock", "build.rs")]
    check(not code_hits, f"code files inside window commits: {code_hits}")

    # guarded inputs must not have been modified by the window commits
    guarded = {row["path"] for row in input_rows}
    touched = set(window_commit_files) | set(fix_commit_files)
    overlap = sorted(guarded & touched)
    check(not overlap, f"guarded inputs touched by window commits: {overlap}")

    result = {
        "sha_guard_lines": len(guard_lines),
        "exe_rows": len(exe_rows),
        "input_rows": len(input_rows),
        "guard_problems": bad[:10],
        "facts_sha_ok_lines": len(fact_guard),
        "post_window_exe_mismatches": post_bad,
        "source_counts": source_counts,
        "frozen_rows": len(frozen_rows),
        "runner": {"bytes": runner.stat().st_size, "sha256": sha(runner)},
        "code_changed_since_impl": changed,
        "window_commit_files": len(window_commit_files),
        "fix_commit_files": len(fix_commit_files),
        "guarded_touched_overlap": overlap,
        "checks": checks,
        "problems": problems,
        "result": "PASS" if not problems else "FAIL",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={result['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("guards:", len(exe_rows), "exe +", len(input_rows), "inputs =", len(guard_lines), "lines; facts SHA_OK", len(fact_guard))
    print("sources:", source_counts)
    print("window commit files:", len(window_commit_files), "fix:", len(fix_commit_files), "code hits: none" if not code_hits else code_hits)


if __name__ == "__main__":
    main()
