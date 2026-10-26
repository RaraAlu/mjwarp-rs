# -*- coding: utf-8 -*-
"""QA check: three-build case_evidence matrix (GNU default / MSVC / GNU+fma).

Parses my own run logs, verifies each judge line has zero bit differences and that
the five `[判据]` lines are byte-identical across the three configurations.
Writes qa-three-build.json.
"""
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
LOGS = TASK / "logs"
MODES = ("gnu-default", "msvc", "gnu-fma")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def judge_lines(path: Path) -> list:
    return [line.strip() for line in read(path).splitlines() if line.strip().startswith("[判据]")]


def test_result_lines(path: Path) -> list:
    return [line.strip() for line in read(path).splitlines() if line.strip().startswith("test result:")]


def parse_judge(line: str) -> dict:
    match = re.match(
        r"\[判据\] (\S+)\s+count (\d+)/(\d+) \| dist 位等 (\d+)/(\d+) max (\d+) ULP \| "
        r"pos 位等接触 (\d+)/(\d+) max ([\d.]+) ULP@尺度 \| frame 位等分量 (\d+)/(\d+) max abs ([0-9.eE+-]+)",
        line)
    if not match:
        return {"raw": line, "parsed": False}
    return {
        "parsed": True, "case": match.group(1),
        "count": [int(match.group(2)), int(match.group(3))],
        "dist_equal": [int(match.group(4)), int(match.group(5))],
        "dist_max_ulp": int(match.group(6)),
        "pos_equal": [int(match.group(7)), int(match.group(8))],
        "pos_max_scaled_ulp": float(match.group(9)),
        "frame_equal": [int(match.group(10)), int(match.group(11))],
        "frame_max_abs": float(match.group(12)),
    }


result = {"modes": {}}
lines_by_mode = {}
for mode in MODES:
    lines = judge_lines(LOGS / f"{mode}-case-evidence.log")
    lines_by_mode[mode] = lines
    parsed = [parse_judge(line) for line in lines]
    entry = {
        "judge_line_count": len(lines),
        "all_parsed": all(item["parsed"] for item in parsed),
        "all_zero_diff": all(
            item["parsed"] and item["count"][0] == item["count"][1]
            and item["dist_equal"][0] == item["dist_equal"][1] and item["dist_max_ulp"] == 0
            and item["pos_equal"][0] == item["pos_equal"][1] and item["pos_max_scaled_ulp"] == 0.0
            and item["frame_equal"][0] == item["frame_equal"][1] and item["frame_max_abs"] == 0.0
            for item in parsed),
        "cases": {item.get("case"): item for item in parsed if item["parsed"]},
        "assert_test_result": test_result_lines(LOGS / f"{mode}-case-evidence-assert.log"),
        "print_test_result": test_result_lines(LOGS / f"{mode}-case-evidence.log"),
        "rustc_info": read(LOGS / f"toolchain-{mode}.txt").splitlines()[:8],
    }
    result["modes"][mode] = entry

result["judge_lines_identical_across_modes"] = (
    lines_by_mode["gnu-default"] == lines_by_mode["msvc"] == lines_by_mode["gnu-fma"])
result["all_modes_green"] = all(
    any("ok." in line and "0 failed" in line for line in entry["assert_test_result"])
    for entry in result["modes"].values())
result["rustflags_mechanism"] = read(LOGS / "gnu-fma-rustc-line.txt").splitlines() if (
    LOGS / "gnu-fma-rustc-line.txt").is_file() else []

(TASK / "qa-three-build.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"identical={result['judge_lines_identical_across_modes']} green={result['all_modes_green']}")
for mode, entry in result["modes"].items():
    print(mode, "lines=", entry["judge_line_count"], "zero=", entry["all_zero_diff"], entry["assert_test_result"])
