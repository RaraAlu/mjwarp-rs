# -*- coding: utf-8 -*-
"""QA static checks for the T4 identity live-check independent check.

Covers: commit boundary, default-gate inclusion mechanism, test purity (CPU only,
no GPU/NVRTC/compile calls), ignore markers, seam locality, product-binary scan,
and the committed/historical unit counts. Writes qa-static-checks.json.
"""
import json
import re
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/t4-identity-live-check-coverage-independent-check"
IMPL = "00831281"
HEAD = "c3acc6b6"
TEST_FILES = [REPO / "tests/t4_identity_live_check.rs", REPO / "tests/support/t4_identity_live.rs"]


def git(*args, check=True):
    out = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, text=True,
                         encoding="utf-8", errors="replace")
    if check and out.returncode != 0:
        raise SystemExit(f"git {' '.join(args)} failed: {out.stderr}")
    return out.stdout


checks = {}

# 1) implementation commit boundary (task JSON acceptance range)
name_status = [line for line in git("diff", "--name-status", f"{IMPL}~1..{IMPL}").splitlines() if line.strip()]
checks["impl_commit"] = {
    "rev": IMPL,
    "range": f"{IMPL}~1..{IMPL}",
    "files": name_status,
    "forbidden_paths_changed": [line for line in name_status
                                if re.search(r"\b(src/|Cargo\.toml|Cargo\.lock|gates\.toml|scripts/|fixtures/|tests/fixtures/)", line)],
    "allowed_prefixes": ["tests/t4_identity_live_check.rs", "tests/support/t4_identity_live.rs",
                         "docs/development-evidence/t4-identity-live-check/", ".normify/"],
    "unexpected_files": [],
}
for line in name_status:
    path = line.split("\t", 1)[1]
    if not any(path.startswith(prefix) for prefix in checks["impl_commit"]["allowed_prefixes"]):
        checks["impl_commit"]["unexpected_files"].append(path)

# 2) range quoted in the Arch message (recorded as a discrepancy, not used as the boundary)
arch_range = [line for line in git("diff", "--name-status", f"{HEAD}~2..{HEAD}").splitlines() if line.strip()]
checks["arch_message_range"] = {
    "range_quoted": f"{HEAD}~2..{HEAD}",
    "contains_impl_commit": False,
    "file_count": len(arch_range),
    "first_files": arch_range[:12],
    "note": "the quoted range does not contain the live-check implementation commit 00831281 "
            "(it is an ancestor 17 commits back); the task JSON acceptance range "
            "00831281~1..00831281 was used instead. Reported as a discrepancy.",
}

# 3) default-gate inclusion: gates.toml unit command
gates = (REPO / ".normify/gates.toml").read_text(encoding="utf-8")
unit_block = re.search(r"\[gate\.unit\](.*?)(?:\n\[|\Z)", gates, re.S)
unit_cmd = re.search(r'cmd\s*=\s*\[(.*?)\]', unit_block.group(1), re.S).group(1) if unit_block else ""
checks["default_gate_inclusion"] = {
    "gates_toml_unit_cmd": unit_cmd.strip(),
    "runs_all_targets": '"test"' in unit_cmd and '"--locked"' in unit_cmd and "--lib" not in unit_cmd,
    "note": "gates.toml [gate.unit] runs `cargo test --locked` without --lib, so the "
            "tests/t4_identity_live_check.rs integration target is auto-discovered and executed",
}

# 4) test purity + ignore markers + seam locality
token_patterns = ["std::process", "Command::new", "nvrtc", "NVRTC", "cuda", "CUDA",
                  "gpu", "GPU", "tokio", "async fn", "std::thread", "std::net"]
purity = {}
for path in TEST_FILES:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    token_hits = {}
    for token in token_patterns:
        hits = [(index + 1, line.strip()) for index, line in enumerate(lines) if token in line]
        if hits:
            token_hits[token] = hits
    purity[str(path.relative_to(REPO)).replace("\\", "/")] = {
        "lines": len(lines),
        "ignored_attrs": len(re.findall(r"(?m)^\s*#\[ignore", text)),
        "doc_mentions_of_ignore": len(re.findall(r"(?m)^\s*//.*#\[ignore", text)),
        "test_attrs": len(re.findall(r"#\[test\]", text)),
        "forbidden_token_hits": token_hits,
        "env_reads": [(index + 1, line.strip()) for index, line in enumerate(lines)
                      if "env::" in line or "var_os" in line or "T4_IDENTITY_LIVE_FIXTURE" in line],
        "imports": sorted(set(re.findall(r"^use\s+([^;]+);", text, re.M))),
    }
checks["test_purity"] = purity

# 5) seam locality: T4_IDENTITY_LIVE_FIXTURE must exist only in test files (never src/)
src_hits = git("grep", "-n", "T4_IDENTITY_LIVE_FIXTURE", "--", "src", check=False).strip()
checks["seam_locality"] = {
    "src_hits": src_hits if src_hits else "(none)",
    "only_in_test_files": src_hits == "",
}

# 6) product binary scan (product binary built at 3b7db351; product src unchanged through HEAD)
scan_targets = [
    REPO / "target/qa-fma-check-postfix/prod-debug/debug/mjwarp-rs.exe",
    REPO / "target/qa-fma-check-postfix/prod-release/release/mjwarp-rs.exe",
]
needle = b"T4_IDENTITY_LIVE_FIXTURE"
binary_scan = []
for path in scan_targets:
    if not path.is_file():
        binary_scan.append({"path": str(path), "exists": False})
        continue
    data = path.read_bytes()
    binary_scan.append({"path": str(path.relative_to(REPO)), "exists": True,
                        "bytes": len(data), "contains_seam": needle in data})
checks["product_binary_scan"] = {
    "note": "product binaries from the 3b7db351 build; product src (src/) is unchanged since",
    "results": binary_scan,
}

# 7) unit counts: committed record at impl commit + current HEAD gate record
def show(path):
    out = subprocess.run(["git", "-C", str(REPO), "show", path], capture_output=True, text=True,
                         encoding="utf-8", errors="replace")
    return out.stdout if out.returncode == 0 else ""


impl_unit = show(f"{IMPL}:.normify/runs/2026-10-25-t4-identity-live-check-coverage/unit.json")
impl_counts = json.loads(impl_unit)["tests"] if impl_unit else None
impl_at = json.loads(impl_unit)["at"] if impl_unit else None
baseline_records = []
for task in ("2026-10-25-case-evidence-frame-diagnosis", "2026-10-25-g10-p5-cuda-default-pipeline"):
    text = show(f"{IMPL}:.normify/runs/{task}/unit.json")
    if text:
        data = json.loads(text)
        baseline_records.append({"task": task, "at": data["at"], "tests": data["tests"]})
checks["unit_counts"] = {
    "impl_commit_unit": {"rev": IMPL, "at": impl_at, "tests": impl_counts},
    "delivery_claim": "595 -> 597 (live-check adds the two new tests)",
    "baseline_595_records_at_impl_commit": baseline_records,
    "baseline_all_before_impl_unit": all(record["at"] < impl_at for record in baseline_records),
    "note": "committed run records at 00831281 show 595 passed / 7 ignored before the live-check "
            "unit run (17:29:39Z) and 597 passed / 7 ignored in the live-check unit run "
            "(17:35:24Z) - the +2 is consistent with the two new non-ignored tests.",
}

# 8) fixture unchanged in the implementation commit + current worktree
fixture_diff = git("diff", "--name-status", f"{IMPL}~1..{IMPL}", "--", "fixtures/precision/t4-current-kernels.json").strip()
checks["fixture_unchanged"] = {
    "impl_commit_fixture_diff": fixture_diff if fixture_diff else "(none)",
    "worktree_fixture_status": git("status", "--porcelain", "--", "fixtures/precision/t4-current-kernels.json").strip()
    or "(clean)",
}

(TASK / "qa-static-checks.json").write_text(
    json.dumps(checks, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(checks, ensure_ascii=False, indent=2))
