"""QA verification v2: evidence copies vs window originals, delivery manifest, claims, fix diff."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/s3-product-wiring/attempt1"
WIN = ROOT / "target/s3-product-wiring/window"
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
LOGS = EV / "logs"
OUT = ROOT / "target/qa-s3win/verify_evidence.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def decode(raw: bytes):
    """Return (text, encoding) for the log encodings used by this window (utf-8 / utf-16-le)."""
    if raw.startswith(b"\xef\xbb\xbf"):
        return raw[3:].decode("utf-8", errors="replace"), "utf-8-sig"
    if len(raw) % 2 == 0 and raw.count(b"\x00") > len(raw) // 4:
        try:
            return raw.decode("utf-16-le"), "utf-16-le"
        except UnicodeDecodeError:
            pass
    try:
        return raw.decode("utf-8"), "utf-8"
    except UnicodeDecodeError:
        return raw.decode("utf-8", errors="replace"), "utf-8-invalid"


def normalized_text(raw: bytes) -> str:
    text, _ = decode(raw)
    return text.replace("\r\n", "\n")


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout


def main():
    pairs = []
    for name in ["facts.txt", "sha-guards.txt", "samples.csv", "guard.log", "baseline.txt", "machine-load.txt"]:
        pairs.append((ATT / name, LOGS / f"attempt1-{name}"))
    for log in sorted(ATT.glob("*.log")):
        if log.name == "guard-raw.log":  # intentionally not committed (BOM source kept raw)
            continue
        pairs.append((log, LOGS / f"attempt1-{log.name}"))
    pairs.append((WIN / "attempt1-runner-stdout.log", LOGS / "attempt1-runner-stdout.log"))
    mismatches = []
    encodings = {}
    for src, dst in pairs:
        if not dst.is_file():
            mismatches.append(f"missing evidence copy {dst.name}")
            continue
        encodings[dst.name] = decode(dst.read_bytes())[1]
        if normalized_text(src.read_bytes()) != normalized_text(dst.read_bytes()):
            mismatches.append(f"content differs {dst.name}")
    findings = []
    other_mismatches = [m for m in mismatches if "runner-stdout" not in m]
    check(not other_mismatches, f"evidence copy mismatches: {other_mismatches[:6]}")
    if any("runner-stdout" in m for m in mismatches):
        findings.append("logs/attempt1-runner-stdout.log: content differs from the raw original beyond line endings (blank-line doubling)")

    # evidence encoding: all should be UTF-8 without BOM (team rule); record actual state
    non_utf8 = []
    for path in [EV / "window-receipt.json", EV / "receipt.json", EV / "README.md"] + sorted(LOGS.glob("attempt1-*")):
        raw = path.read_bytes()
        encoding = decode(raw)[1]
        if encoding != "utf-8" or raw.startswith(b"\xef\xbb\xbf"):
            non_utf8.append(f"{path.name}: {encoding}")
    check(set(non_utf8) == {"attempt1-runner-stdout.log: utf-16-le"}, f"unexpected non-UTF-8 evidence files: {non_utf8}")
    findings.extend(non_utf8)

    # delivery record identity (sha/bytes exact; line count on decoded text)
    record = json.loads((ROOT / "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-window.json").read_text(encoding="utf-8"))
    bad_rows = []
    line_rows = []
    for entry in record.get("files", []):
        path = ROOT / entry["path"]
        if not path.is_file():
            bad_rows.append(f"missing {entry['path']}")
            continue
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != entry["sha256"] or len(raw) != entry.get("bytes", len(raw)):
            bad_rows.append(entry["path"])
        actual_lines = len(normalized_text(raw).splitlines())
        if actual_lines != entry.get("lines", actual_lines):
            line_rows.append(f"{entry['path']} record={entry['lines']} decoded={actual_lines}")
    check(not bad_rows, f"delivery record sha/bytes mismatches: {bad_rows[:6]}")
    findings.extend("delivery record line count differs: " + row for row in line_rows)
    check(record["task_id"] == "2026-10-27-product-collision-inputs-s3-window", "record task id")
    head_blob = git("rev-parse", "HEAD:docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-window.json").strip()
    work_blob = git("hash-object", "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-window.json").strip()
    check(head_blob == work_blob, "delivery record committed")

    window_receipt = json.loads((EV / "window-receipt.json").read_text(encoding="utf-8"))
    check(window_receipt["task_id"] == "2026-10-27-product-collision-inputs-s3-window", "window-receipt task_id")
    check(window_receipt["boundaries"]["window_pass_claimed"] is False, "window_pass_claimed false")
    receipt = json.loads((EV / "receipt.json").read_text(encoding="utf-8"))
    check(receipt["task_id"] == "2026-10-27-product-collision-inputs-s1-data-snapshot", "top-level task_id intact")
    check(receipt["s2"]["task_id"] == "2026-10-27-product-collision-inputs-s2-adapter", "s2 task_id intact")
    check(receipt["s3"]["task_id"] == "2026-10-27-product-collision-inputs-s3-entry-window", "s3 task_id intact")
    check(receipt["s3"]["window"]["attempt1"]["task_id"] == "2026-10-27-product-collision-inputs-s3-window",
          "receipt attempt1 task_id")

    claims_text = "\n".join((EV / name).read_text(encoding="utf-8") for name in ["README.md", "receipt.json", "window-receipt.json"]) + \
        (ROOT / "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-window.json").read_text(encoding="utf-8")
    # every 窗口通过 occurrence must sit inside a negation context
    negatives = ("不写", "不得", "不宣称", "不做", "no ", "never")
    positive = []
    for match in re.finditer("窗口通过", claims_text):
        context = claims_text[max(0, match.start() - 160): match.end() + 40]
        if not any(word in context for word in negatives):
            positive.append(context.replace(chr(10), " ")[:120])
    check(not positive, f"non-negated window-pass phrase: {positive[:3]}")
    task = json.loads((ROOT / ".normify/changes/2026-10-27-product-collision-inputs-s3-window-independent-check.json").read_text(encoding="utf-8"))
    terms = [t for c in task["checks"] if c["kind"] == "forbid_claim" for t in c["terms"]]
    term_hits = [t for t in terms if t in claims_text]
    check(not term_hits, f"regulated claim terms present: {term_hits}")
    check("非验证窗" in claims_text, "O1 dev self-test marked non-window")
    check("outside a signed window" in window_receipt["dev_self_test_O1"]["note"], "window receipt O1 note")

    fix_files = [line for line in git("show", "--name-only", "--format=", "fa704a6e").splitlines() if line.strip()]
    check(len(fix_files) == 4, f"fix commit files {len(fix_files)}")
    fix_diff = git("show", "fa704a6e", "--", *fix_files)
    numeric_removed = []
    for line in fix_diff.splitlines():
        if line.startswith("-") and not line.startswith("---"):
            if re.search(r"\b(4142|1051|3091|11920|9904|12976|867|1055)\b", line):
                numeric_removed.append(line.strip()[:120])
    check(not numeric_removed, f"fix commit removed quoted numbers: {numeric_removed[:5]}")
    check(git("log", "-1", "--format=%(trailers)", "fa704a6e").strip() ==
          "Normify-Fixes: 2026-10-27-product-collision-inputs-s3-window", "fix commit trailer")

    result = {
        "copied_files": len(pairs),
        "evidence_copy_mismatches": mismatches,
        "non_utf8_files": non_utf8,
        "findings": findings,
        "delivery_files": len(record.get("files", [])),
        "delivery_sha_bytes_mismatches": bad_rows,
        "delivery_line_count_mismatches": line_rows,
        "encodings": encodings,
        "fix_files": len(fix_files),
        "fix_numeric_removals": numeric_removed,
        "checks": checks,
        "problems": problems,
        "result": "PASS" if not problems else "FAIL",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={result['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("line-count mismatches:", line_rows)
    print("non-utf8:", non_utf8)


if __name__ == "__main__":
    main()
