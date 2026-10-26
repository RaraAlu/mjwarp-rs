# -*- coding: utf-8 -*-
"""QA check 6: GPU window discipline (one window, samples, guard, pre/post, no residue).

Parses the delivery's window logs independently and compares with the Arch-approved
record. Writes qa-window.json.
"""
import json
import re
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EV = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture"
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
STAMP = "20261026-012956"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def git(*args):
    out = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                         text=True, encoding="utf-8", errors="replace")
    return out.stdout.strip(), out.returncode


def parse_snapshot(path: Path) -> dict:
    text = read(path)
    gpu = None
    apps = []
    for line in text.splitlines():
        line = line.strip()
        m = re.match(r"^(NVIDIA .+), (\d+) MiB, (\d+) MiB, (\d+) MiB, (\d+) %, (\d+), ([\d.]+), ([\d.]+)$", line)
        if m:
            gpu = {"name": m.group(1), "total_mib": int(m.group(2)), "used_mib": int(m.group(3)),
                   "free_mib": int(m.group(4)), "util": int(m.group(5)), "temp_c": int(m.group(6)),
                   "driver": m.group(7), "compute_cap": m.group(8)}
        m = re.match(r"^(\d+), (.+), (.+)$", line)
        if m and m.group(1).isdigit() and m.group(2).strip().lower() != "process_name":
            apps.append({"pid": int(m.group(1)), "name": m.group(2).strip(),
                         "used": m.group(3).strip()})
    return {"gpu": gpu, "compute_apps": apps, "compute_apps_count": len(apps)}


def parse_samples(path: Path) -> dict:
    rows = []
    for line in read(path).splitlines():
        line = line.strip()
        m = re.match(r"^(\d\d:\d\d:\d\d\.\d\d\d);(\d+), (\d+), (\d+), (\d+)$", line)
        if m:
            rows.append({"time": m.group(1), "used": int(m.group(2)), "free": int(m.group(3)),
                         "util": int(m.group(4)), "temp": int(m.group(5))})
    return rows


pre = parse_snapshot(EV / f"logs/window-{STAMP}-pre.txt")
post = parse_snapshot(EV / f"logs/window-{STAMP}-post.txt")
samples = parse_samples(EV / f"logs/window-{STAMP}-samples.txt")
guard = read(EV / f"logs/window-{STAMP}-guard.log")
meta = json.loads((EV / "capture-meta.json").read_text(encoding="utf-8"))

# sample statistics
used = [s["used"] for s in samples]
free = [s["free"] for s in samples]
util = [s["util"] for s in samples]
temp = [s["temp"] for s in samples]
intervals = []
for a, b in zip(samples, samples[1:]):
    try:
        to = sum(float(x) * f for x, f in zip(a["time"].split(":"), (3600, 60, 1)))
        tb = sum(float(x) * f for x, f in zip(b["time"].split(":"), (3600, 60, 1)))
        intervals.append(round(tb - to, 3))
    except ValueError:
        pass
intervals.sort()
median_interval = intervals[len(intervals) // 2] if intervals else None
consecutive_80 = 0
max_consecutive_80 = 0
for u in util:
    if u >= 80:
        consecutive_80 += 1
        max_consecutive_80 = max(max_consecutive_80, consecutive_80)
    else:
        consecutive_80 = 0

pre_names = {a["name"] for a in pre["compute_apps"]}
post_names = {a["name"] for a in post["compute_apps"]}
pre_pids = {a["pid"] for a in pre["compute_apps"]}
post_pids = {a["pid"] for a in post["compute_apps"]}

# aborted attempts: must be zero-GPU (no CUDA device init / no rounds)
aborted = {}
for name in ("window-attempt-1-aborted.txt", "window-attempt-2-aborted.txt"):
    text = read(EV / "logs" / name)
    aborted[name] = {
        "bytes": len(text),
        "mentions_cuda_device": ("device: cuda" in text or "cuda:0" in text),
        "mentions_compiled_module": ("(compiled)" in text),
        "mentions_round_output": ("rounds/round" in text),
        "first_lines": text.splitlines()[:6],
    }

# one window only: the final meta and round logs share the same stamp
round1 = read(EV / f"logs/window-{STAMP}-round1.txt")
round2 = read(EV / f"logs/window-{STAMP}-round2.txt")
wall_ms = []
for text in (round1, round2):
    m = re.search(r"EXIT=0 WALL_MS=(\d+)", text)
    wall_ms.append(int(m.group(1)) if m else None)
preflight = read(EV / f"logs/window-{STAMP}-preflight.txt")

# guard template unchanged since the pre-capture baseline commit
guard_diff, _ = git("diff", "--name-status", "209b1039..HEAD", "--", "scripts/gpu-window-guard.ps1")
finalize = read(EV / "finalize_capture.ps1")

result = {
    "window_stamp": STAMP,
    "pre_gpu": pre["gpu"],
    "post_gpu": post["gpu"],
    "samples": {
        "count": len(samples),
        "median_interval_s": median_interval,
        "first_time": samples[0]["time"] if samples else None,
        "last_time": samples[-1]["time"] if samples else None,
        "used_mib": {"min": min(used) if used else None, "max": max(used) if used else None},
        "free_mib": {"min": min(free) if free else None, "max": max(free) if free else None},
        "util_percent": {"min": min(util) if util else None, "max": max(util) if util else None},
        "temp_c": {"min": min(temp) if temp else None, "max": max(temp) if temp else None},
        "max_consecutive_util_ge_80": max_consecutive_80,
    },
    "guard_log": guard,
    "guard": {
        "aborted_false": "aborted=False" in guard,
        "residual_context_pids_empty": "residual_context_pids=" in guard and "residual_context_pids= " in guard,
    },
    "compute_apps": {
        "pre_count": pre["compute_apps_count"],
        "post_count": post["compute_apps_count"],
        "same_name_set": pre_names == post_names,
        "same_pid_set": pre_pids == post_pids,
        "pre_has_python": any("python" in n.lower() for n in pre_names),
        "post_has_python": any("python" in n.lower() for n in post_names),
        "same_name_multiset": sorted(pre_names) == sorted(post_names),
    },
    "aborted_attempts": aborted,
    "timing": {"round1_wall_ms": wall_ms[0], "round2_wall_ms": wall_ms[1],
               "preflight_log": preflight.splitlines()},
    "guard_template_unchanged_since_209b1039": guard_diff if guard_diff else "(none)",
    "finalize_script": {
        "mentions_execute_or_capture_recipe": any(token in finalize for token in
                                                  ("run_capture_gpu", "-Execute", "capture_mjwarp_f32_gpu")),
        "mentions_warp_or_cuda": any(token in finalize.lower() for token in
                                     ("warp", "cuda", "nvrtc")),
    },
    "arch_approved_record": {
        "used_peak_mib": 4417,
        "used_delta_mib": 339,
        "free_min_mib": 11645,
        "util_max_percent": 80,
        "temp_max_c": 30,
        "compute_apps_each": 35,
        "preflight_s": 1.24,
        "round1_s": 28.08,
        "round2_s": 4.47,
    },
    "matches_approved_record": {
        "used_peak": (max(used) if used else None) == 4417,
        "free_min": (min(free) if free else None) == 11645,
        "util_max": (max(util) if util else None) == 80,
        "temp_max": (max(temp) if temp else None) == 30,
        "compute_apps_35_each": pre["compute_apps_count"] == post["compute_apps_count"] == 35,
        "round1_s": wall_ms[0] is not None and abs(wall_ms[0] / 1000 - 28.08) < 0.05,
        "round2_s": wall_ms[1] is not None and abs(wall_ms[1] / 1000 - 4.47) < 0.05,
    },
    "meta_sampler": meta.get("sampler"),
    "meta_guard": meta.get("guard"),
    "meta_pre_post": {"pre": meta.get("pre_snapshot"), "post": meta.get("post_snapshot")},
}
(TASK / "qa-window.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
