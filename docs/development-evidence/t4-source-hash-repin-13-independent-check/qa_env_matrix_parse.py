# -*- coding: utf-8 -*-
"""Summarise QA-independent environment matrix raw logs into review evidence.

Raw logs live under target/qa-repin13/ (untracked); this script keeps the
summary in the evidence directory (small, <=500 lines).
"""
import json
import re
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = REPO / "docs/development-evidence/t4-source-hash-repin-13-independent-check"
RAW = REPO / "target/qa-repin13"

HEAD = bytes(Path(REPO / ".git/HEAD").read_bytes()).decode("utf-8", "replace").strip()
steps = [json.loads(line) for line in (RAW / "steps.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]

RESULT_RE = re.compile(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored")


def read_text_auto(path: Path) -> str:
    raw = path.read_bytes()
    if raw.startswith(b"\xff\xfe"):
        text = raw[2:].decode("utf-16-le", "replace")
    elif raw.startswith(b"\xef\xbb\xbf"):
        text = raw[3:].decode("utf-8", "replace")
    else:
        text = raw.decode("utf-8", "replace")
    return text.replace("\r\n", "\n")


def parse_log(rel_log: str):
    path = Path(rel_log)
    text = read_text_auto(path)
    results = [{"state": m.group(1), "passed": int(m.group(2)), "failed": int(m.group(3)), "ignored": int(m.group(4))}
               for m in RESULT_RE.finditer(text)]
    failed_names = re.findall(r"^---- (\S+) stdout ----$", text, re.M)
    frame = [l.strip() for l in text.splitlines() if "ellipsoid-box frame" in l]
    return {
        "log": str(path.relative_to(REPO)).replace("\\", "/"),
        "test_result_lines": len(results),
        "sum_passed": sum(r["passed"] for r in results),
        "sum_failed": sum(r["failed"] for r in results),
        "sum_ignored": sum(r["ignored"] for r in results),
        "failed_test_names": failed_names[:5],
        "frame_line": frame[:1],
        "compiling_lines": text.count("Compiling "),
        "finished_line": next((l.strip() for l in text.splitlines() if "Finished `test` profile" in l), ""),
    }


def exit_of(name: str, default=None):
    p = RAW / name
    if p.exists():
        m = re.search(r"exit=(\d+)", p.read_text(encoding="utf-8", errors="replace"))
        if m:
            return int(m.group(1))
    return default


def wall_of(name: str):
    p = RAW / name
    if p.exists():
        m = re.search(r"wall_s=([0-9.]+)", p.read_text(encoding="utf-8", errors="replace"))
        if m:
            return float(m.group(1))
    return None


runs = []
for s in steps:
    rec = {
        "name": s["name"], "cmd": s["cmd"], "target_dir": s["target_dir"],
        "exit": s["exit"], "wall_s": s["wall_s"],
        "started_utc": s["started_utc"], "ended_utc": s["ended_utc"],
        "deps_files_before": s["deps_files_before"], "deps_files_after": s["deps_files_after"],
        "target_mib": s["target_mib"],
    }
    rec.update(parse_log(s["log"]))
    runs.append(rec)

extra = [
    ("gnu-cuda-identity-targets", "gnu-cuda-identity-targets.log", "gnu-cuda-identity-exit.txt",
     "cargo test --locked --features cuda-probe --test t4_camlight_precision --test t4_gravity_3d_product_chain "
     "--test t4_gravity_nonzero_product_chain --test t4_input_precision --test t4_tendon_wake_precision",
     "target/verify-clean-qa-cuda"),
    ("msvc-identity-targets", "msvc-identity-targets.log", "msvc-identity-exit.txt",
     "cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe,cuda-probe --test "
     "t4_camlight_precision --test t4_gravity_3d_product_chain --test t4_gravity_nonzero_product_chain "
     "--test t4_input_precision --test t4_tendon_wake_precision",
     "target/verify-clean-qa-msvc"),
    ("msvc-all-features", "msvc-all-features.log", "msvc-all-features-exit.txt",
     "cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast",
     "target/verify-clean-qa-msvc-all"),
]
for name, log, exit_file, cmd, td in extra:
    if not (RAW / log).exists():
        continue
    rec = {"name": name, "cmd": cmd, "target_dir": td, "exit": exit_of(exit_file), "wall_s": wall_of(exit_file)}
    rec.update(parse_log(str((RAW / log))))
    runs.append(rec)

env_before = (RAW / "env-before.txt").read_text(encoding="utf-8").strip().splitlines()
env_after = (RAW / "env-after-clear.txt").read_text(encoding="utf-8").strip().splitlines()
cleared = (RAW / "env-cleared.txt").read_text(encoding="utf-8").strip().splitlines()
toolchain_default = (RAW / "toolchain-default.txt").read_text(encoding="utf-8").strip().splitlines()
toolchain_msvc = (RAW / "toolchain-msvc.txt").read_text(encoding="utf-8").strip().splitlines()
repo_facts = (RAW / "repo-facts.txt").read_text(encoding="utf-8").strip().splitlines()

result = {
    "task": "2026-10-25-t4-source-hash-repin-13-independent-check",
    "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "head_at_run": next((l.split("=", 1)[1] for l in repo_facts if l.startswith("head=")), ""),
    "product_worktree_changes_at_run": next((l.split("=", 1)[1] for l in repo_facts if l.startswith("product_worktree_changes_count=")), ""),
    "clean_criteria": {
        "dedicated_target_dirs": sorted({r["target_dir"] for r in runs}),
        "cargo_rust_env_before": env_before,
        "cargo_rust_env_after_clear": env_after,
        "cleared": cleared,
        "toolchain_repo_default": toolchain_default,
        "toolchain_explicit_msvc": toolchain_msvc,
    },
    "runs": runs,
}
text = json.dumps(result, ensure_ascii=False, indent=2)
(OUT / "qa-env-matrix.json").write_text(text + "\n", encoding="utf-8")
print(f"wrote qa-env-matrix.json ({len(text.splitlines())} lines), runs={len(runs)}")
for r in runs:
    print(f"{r['name']}: exit={r['exit']} wall={r['wall_s']} passed={r['sum_passed']} failed={r['sum_failed']} ignored={r['sum_ignored']} results={r['test_result_lines']} fail_names={r['failed_test_names']}")
