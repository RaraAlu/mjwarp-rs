# -*- coding: utf-8 -*-
"""QA check: default `cargo test --locked` gate in a clean GNU target.

Parses my own summary extract. Writes qa-default-gate.json.
"""
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
LOGS = TASK / "logs"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace") if path.is_file() else ""


summary_lines = [line for line in read(LOGS / "default-gate-summary.txt").splitlines() if line.strip()]
results = []
for line in summary_lines:
    match = re.search(
        r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;.*?(\d+) filtered out",
        line)
    if match:
        results.append({
            "status": match.group(1),
            "passed": int(match.group(2)),
            "failed": int(match.group(3)),
            "ignored": int(match.group(4)),
            "filtered": int(match.group(5)),
        })
binary_count = None
for line in summary_lines:
    match = re.match(r"test_binaries_with_running_lines=(\d+)", line)
    if match:
        binary_count = int(match.group(1))

case_lines = [line.strip() for line in read(LOGS / "default-gate-case-evidence-lines.txt").splitlines()
              if line.strip()]
result = {
    "test_binaries": binary_count,
    "test_result_lines": len(results),
    "passed": sum(item["passed"] for item in results),
    "failed": sum(item["failed"] for item in results),
    "ignored": sum(item["ignored"] for item in results),
    "all_ok": all(item["status"] == "ok" and item["failed"] == 0 for item in results),
    "expected_counts_match": (sum(item["passed"] for item in results) == 604
                              and sum(item["failed"] for item in results) == 0
                              and sum(item["ignored"] for item in results) == 7),
    "case_evidence_lines": case_lines,
    "case_evidence_upstream_ulp_ran": any("upstream_f32_within_4ulp ... ok" in line for line in case_lines),
    "case_evidence_ours_ran": any("our_contacts_match_upstream_f32_within_4ulp ... ok" in line for line in case_lines),
    "raw_summary_tail": summary_lines[-5:],
}
(TASK / "qa-default-gate.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
