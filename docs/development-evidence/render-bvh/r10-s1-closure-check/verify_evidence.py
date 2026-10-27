"""QA evidence self-check for the R10-S1 observation-closure review.

Writes manifest.json (sha256/bytes/lf_lines for every evidence file except
manifest.json, the derived self-check log and the derived static-check log) and
logs/evidence-selfcheck.txt. Forbidden-claim terms come from the task record.
Read-only outside this directory.
"""
import hashlib
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
TASK = "2026-10-28-render-bvh-r10-s1-closure-check"
EXCLUDED = {"manifest.json", "logs/evidence-selfcheck.txt", "logs/static-check.log"}
README_LIMIT = 500
CRLF = b"\r\n"


def forbidden_terms():
    data = json.loads((REPO / ".normify/changes" / f"{TASK}.json").read_text(encoding="utf-8"))
    terms = []
    for check in data.get("checks", []):
        if check.get("kind") == "forbid_claim":
            terms.extend(check.get("terms", []))
    return terms


def main():
    terms = forbidden_terms()
    files = sorted(
        p for p in HERE.rglob("*")
        if p.is_file() and p.relative_to(HERE).as_posix() not in EXCLUDED
    )
    entries, problems = [], []
    for path in files:
        raw = path.read_bytes()
        rel = path.relative_to(HERE).as_posix()
        entries.append({"path": rel, "sha256": hashlib.sha256(raw).hexdigest(),
                        "bytes": len(raw), "lf_lines": raw.count(b"\n")})
        if raw[:3] == b"\xef\xbb\xbf" or CRLF in raw:
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
    if not receipt.is_file():
        problems.append("receipt.json missing")
    elif json.loads(receipt.read_text(encoding="utf-8")).get("task_id") != TASK:
        problems.append("receipt /task_id mismatch")
    manifest = {"schema": 1, "task_id": TASK, "kind": "qa_independent_check_manifest",
                "file_count": len(entries), "files": entries}
    (HERE / "manifest.json").write_bytes(
        (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    out = ["# evidence self-check (R10-S1 closure independent check)", f"task: {TASK}",
           f"files in manifest: {len(entries)}", f"README lines: {readme_lines}",
           f"forbidden terms checked (from task record): {len(terms)}",
           f"problems: {len(problems)}"] + [f"  {p}" for p in problems]
    (HERE / "logs/evidence-selfcheck.txt").write_bytes(("\n".join(out) + "\n").encode("utf-8"))
    print("\n".join(out))


if __name__ == "__main__":
    main()
