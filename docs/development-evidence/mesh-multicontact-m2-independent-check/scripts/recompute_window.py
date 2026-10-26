"""QA M2 independent window aggregate recomputation from the raw nvidia-smi samples.

Reads target/m2/attempt{1,2,3,4}/samples.csv + guard.log + facts.txt and recomputes
peak used / min free / max util / 80%-streak / residual-relevant counts. Read-only.
Output: target/qa-m2/window-recompute.json
"""
import json
import re
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/m2"
OUT = ROOT / "target/qa-m2/window-recompute.json"


def read_text(path: Path) -> str:
    raw = path.read_bytes()
    if raw[:2] == b"\xff\xfe":
        return raw[2:].decode("utf-16-le")
    if raw[:3] == b"\xef\xbb\xbf":
        return raw[3:].decode("utf-8")
    return raw.decode("utf-8")


def fetch(path: Path) -> list[dict]:
    lines = read_text(path).strip().split("\n")
    header = lines[0].split(",")
    rows = []
    for line in lines[1:]:
        cells = line.split(",")
        rows.append(dict(zip(header, cells)))
    return rows


report = {"kind": "qa-m2-window-recompute", "attempts": {}}
for name in ["attempt1", "attempt2", "attempt3", "attempt4"]:
    directory = ATT / name
    rows = fetch(directory / "samples.csv")
    used = [int(row["used_mib"]) for row in rows]
    free = [int(row["free_mib"]) for row in rows]
    util = [int(row["util_percent"]) for row in rows]
    streak = 0
    current = 0
    for value in util:
        if value >= 80:
            current += 1
            streak = max(streak, current)
        else:
            current = 0
    guard = read_text(directory / "guard.log")
    aggregates = re.search(
        r"guard_aggregates .*?baseline_used=(\d+).*?used_peak=(\d+).*?delta_peak=(\d+).*?"
        r"free_min=(\d+).*?util_max=(\d+).*?util_streak=(\d+)",
        guard,
        re.S,
    )
    stopped = re.search(r"guard_stop reason=(\S+)(.*)", guard)
    facts = {}
    facts_path = directory / "facts.txt"
    if facts_path.is_file():
        for line in read_text(facts_path).split("\n"):
            if ":" in line:
                key, value = line.split(":", 1)
                facts[key.strip()] = value.strip()
    report["attempts"][name] = {
        "samples": len(rows),
        "used_peak_by_csv": max(used),
        "free_min_by_csv": min(free),
        "util_max_by_csv": max(util),
        "util_streak_80_by_csv": streak,
        "first_timestamp": rows[0]["timestamp"],
        "last_timestamp": rows[-1]["timestamp"],
        "guard_aggregates": {
            "baseline_used": int(aggregates.group(1)),
            "used_peak": int(aggregates.group(2)),
            "delta_peak": int(aggregates.group(3)),
            "free_min": int(aggregates.group(4)),
            "util_max": int(aggregates.group(5)),
            "util_streak": int(aggregates.group(6)),
        }
        if aggregates
        else None,
        "guard_stop": {"reason": stopped.group(1), "detail": stopped.group(2).strip()}
        if stopped
        else None,
        "facts": facts,
        "delta_vs_guard_baseline": max(used) - int(aggregates.group(1)) if aggregates else None,
    }

OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
for name, row in report["attempts"].items():
    print(
        name,
        "samples=", row["samples"],
        "csv_peak=", row["used_peak_by_csv"],
        "csv_free_min=", row["free_min_by_csv"],
        "csv_util_max=", row["util_max_by_csv"],
        "streak=", row["util_streak_80_by_csv"],
        "guard_delta=", row["guard_aggregates"]["delta_peak"] if row["guard_aggregates"] else None,
        "stop=", row["guard_stop"]["reason"] if row["guard_stop"] else None,
    )
print("wrote", OUT)
