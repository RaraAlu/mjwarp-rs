# -*- coding: utf-8 -*-
"""Trim/derive evidence log helpers for the faithful impact measurement.

- move raw 250 ms CPU-process observation logs (huge) to target/ raw dir
- write dedup summaries (unique pid/name + first/last timestamp)
- extract rustc command lines (RUSTFLAGS vs .cargo/config.toml interaction)
"""
import re
import shutil
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = REPO / "docs/development-evidence/faithful-impact-measurement"
LOGS = EVIDENCE / "logs"
RAW = REPO / "target/faithful/raw-observations"
RAW.mkdir(parents=True, exist_ok=True)

obs_files = sorted(LOGS.glob("gpu-window-*-cpu-process-observations.txt"))
summary_lines = [
    "# GPU window CPU-process observations (deduped)",
    "# guard: CPU-only processes are recorded, never abort the window (approved rule).",
    "",
]
for path in obs_files:
    entries = {}
    order = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"(\d{2}:\d{2}:\d{2}\.\d{3}) (cpu_only|new_cuda_compute) pid=(\d+) name=(\S+)(.*)", line)
        if not m:
            continue
        ts, kind, pid, name, rest = m.groups()
        key = (kind, pid, name)
        if key not in entries:
            entries[key] = [ts, ts, 0]
            order.append(key)
        entries[key][1] = ts
        entries[key][2] += 1
    summary_lines.append(f"## {path.name}: unique_processes={len(order)}")
    for key in order:
        kind, pid, name = key
        first, last, count = entries[key]
        summary_lines.append(f"{kind} pid={pid} name={name} first={first} last={last} samples={count}")
    summary_lines.append("")
    shutil.move(str(path), str(RAW / path.name))
(LOGS / "cpu-process-observations-summary.txt").write_text("\n".join(summary_lines) + "\n", encoding="utf-8")

cmd_lines = ["# rustc command line for --crate-name mjwarp_rs (RUSTFLAGS vs .cargo/config.toml interaction)", ""]
for mode in ("gnu-default", "gnu-fma", "gnu-fma-ldc"):
    log = LOGS / f"build-{mode}.log"
    if not log.exists():
        continue
    text = log.read_text(encoding="utf-8", errors="replace")
    line = next((l for l in text.splitlines() if "--crate-name mjwarp_rs" in l), "")
    cmd_lines.append(f"## mode={mode}")
    cmd_lines.append(f"has_link_dead_code={'link-dead-code' in line}")
    cmd_lines.append(f"has_target_feature_fma={'target-feature=+fma' in line}")
    cmd_lines.append(line.strip()[:4000])
    cmd_lines.append("")
(LOGS / "rustc-command-lines.txt").write_text("\n".join(cmd_lines) + "\n", encoding="utf-8")
print("moved raw obs:", len(obs_files))
print("wrote cpu-process-observations-summary.txt and rustc-command-lines.txt")
