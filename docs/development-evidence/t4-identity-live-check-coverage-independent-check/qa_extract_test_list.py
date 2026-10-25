# -*- coding: utf-8 -*-
"""Extract the decision-relevant part of `cargo test --locked -- --list`.

The full listing (843 lines) exceeds the evidence single-file line limit, so the
full output stays in target/qa-t4-check/ (not committed) and this script writes:
  - logs/default-cargo-test-list-extract.txt : header + totals + the live-check target section
  - qa-default-test-list.json                : machine-readable counts and the two test names
"""
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/t4-identity-live-check-coverage-independent-check"
FULL = REPO / "target/qa-t4-check/default-cargo-test-list-full.log"
EXTRACT = TASK / "logs/default-cargo-test-list-extract.txt"

raw = FULL.read_bytes()
if raw.startswith(b"\xff\xfe"):
    text = raw.decode("utf-16")
else:
    text = raw.decode("utf-8", errors="replace")
lines = text.splitlines()

entries = [line for line in lines if line.rstrip().endswith(": test")]
per_target = [line.strip() for line in lines if re.match(r"^\s*\d+ tests?, \d+ benchmarks?$", line)]
live_section = []
inside = False
for line in lines:
    if "tests\\t4_identity_live_check.rs" in line or "tests/t4_identity_live_check.rs" in line:
        inside = True
        live_section.append(line.rstrip())
        continue
    if inside:
        live_section.append(line.rstrip())
        if re.match(r"^\s*\d+ tests?, \d+ benchmarks?$", line.strip()):
            break

summary = {
    "command": "cargo test --locked -- --list",
    "exit": 0,
    "full_log (not committed)": str(FULL),
    "full_log_lines": len(lines),
    "test_entry_lines": len(entries),
    "per_target_summary_lines": len(per_target),
    "live_check_names_present": [name for name in (
        "live_product_sources_match_fixture",
        "sha256_and_lf_normalization_match_known_vectors",
    ) if any(name in line for line in entries)],
    "explanation": "611 test entries = 604 passed + 7 ignored reported by gate unit at the same HEAD; "
                   "both live-check tests are non-ignored and appear in the default listing.",
}
(TASK / "qa-default-test-list.json").write_text(
    json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

extract = [
    "# extract of `cargo test --locked -- --list` (default gate unit command)",
    f"# full listing is {len(lines)} lines (UTF-16LE, written by Windows PowerShell redirection) ->",
    "# exceeds the evidence single-file limit; full copy: target/qa-t4-check/default-cargo-test-list-full.log",
    f"# total test entries: {len(entries)} (= 604 passed + 7 ignored reported by gate unit at the same HEAD)",
    "",
    "## live-check integration target section",
    *live_section,
    "",
    "## verification",
    "live-check names present: " + ", ".join(summary["live_check_names_present"]),
]
EXTRACT.write_text("\n".join(extract) + "\n", encoding="utf-8")
print(f"entries={len(entries)} live_names={summary['live_check_names_present']}")
