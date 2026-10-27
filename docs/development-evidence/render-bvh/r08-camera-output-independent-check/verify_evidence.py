"""QA evidence self-check for the R08-S1 independent check.

Scans this evidence directory, writes manifest.json (sha256/bytes/lf_lines for
every evidence file except manifest.json and the derived self-check log), and
writes logs/evidence-selfcheck.txt with encoding / forbidden-term / size checks.

The forbidden-claim term list is read from this task's Normify record (no literal
terms are stored in this file). Read-only outside this directory.
"""
import hashlib
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
TASK = "2026-10-28-render-bvh-r08-camera-output-check"
SELFCHECK_LOG = "logs/evidence-selfcheck.txt"
README_LIMIT = 500


def forbidden_terms():
    record = REPO / ".normify" / "changes" / f"{TASK}.json"
    data = json.loads(record.read_text(encoding="utf-8"))
    terms = []
    for check in data.get("checks", []):
        if check.get("kind") == "forbid_claim":
            terms.extend(check.get("terms", []))
    return terms


def main():
    terms = forbidden_terms()
    files = sorted(
        p
        for p in HERE.rglob("*")
        if p.is_file()
        and p.name != "manifest.json"
        and p.relative_to(HERE).as_posix() != SELFCHECK_LOG
    )
    entries = []
    problems = []
    for path in files:
        raw = path.read_bytes()
        rel = path.relative_to(HERE).as_posix()
        entries.append(
            {
                "path": rel,
                "sha256": hashlib.sha256(raw).hexdigest(),
                "bytes": len(raw),
                "lf_lines": raw.count(b"\n"),
            }
        )
        if raw[:3] == b"\xef\xbb\xbf" or b"\r\n" in raw:
            problems.append(f"encoding: {rel}")
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            problems.append(f"not utf-8: {rel}")
            continue
        for term in terms:
            if term in text:
                problems.append(f"forbidden term in {rel}")
    readme = HERE / "README.md"
    readme_lines = readme.read_bytes().count(b"\n") if readme.is_file() else -1
    if readme_lines > README_LIMIT:
        problems.append(f"README lines {readme_lines} > {README_LIMIT}")
    receipt = HERE / "receipt.json"
    if receipt.is_file():
        data = json.loads(receipt.read_text(encoding="utf-8"))
        if data.get("task_id") != TASK:
            problems.append("receipt /task_id mismatch")
    else:
        problems.append("receipt.json missing")
    manifest = {
        "schema": 1,
        "task_id": TASK,
        "kind": "qa_independent_check_manifest",
        "file_count": len(entries),
        "files": entries,
    }
    (HERE / "manifest.json").write_bytes(
        (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )
    out = [
        "# evidence self-check (R08-S1 independent check)",
        f"task: {TASK}",
        f"files in manifest: {len(entries)}",
        f"README lines: {readme_lines}",
        f"forbidden terms checked (from task record): {len(terms)}",
        f"problems: {len(problems)}",
    ] + [f"  {problem}" for problem in problems]
    (HERE / SELFCHECK_LOG).write_bytes(("\n".join(out) + "\n").encode("utf-8"))
    print("\n".join(out))


if __name__ == "__main__":
    main()
