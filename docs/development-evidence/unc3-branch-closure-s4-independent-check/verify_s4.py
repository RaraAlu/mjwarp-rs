"""UNC-3 S4 独立复核复算脚本（只读；不重开窗、不跑 cargo/GPU）。

输入：S4 证据目录 docs/development-evidence/g10-p5-branch-closure-validation/、
      target/s4-freeze/、target/s4-branch-closure/attempt1/ 原始 guard.log（若存在）。
输出：target/qa-s4/verify_s4.json（本脚本自检数据，随后复制进证据目录）。

覆盖：窗口采样/guard/floor/delta 重算；sha-guards 25 行与冻结 exe/参考/fixture 独立哈希；
逐实体日志解析（exit/数值）；证据文件 sha 与 README §12 声明比对；BOM 关系；零编译 mtime 扫描；
window-receipt 数字一致性。
"""
import hashlib
import json
import re
import struct
import sys
from datetime import datetime, timedelta
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/g10-p5-branch-closure-validation"
FIXTURES = REPO / "fixtures"
FREEZE = REPO / "target/s4-freeze"
ATTEMPT = REPO / "target/s4-branch-closure/attempt1"
OUT = Path(__file__).with_name("verify_s4.json")

WINDOW_START = "2026-10-27T10:36:50"
WINDOW_END = "2026-10-27T10:41:10"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main():
    out = {}

    # ---------- 1. samples.csv 重算 ----------
    lines = read_text(EVID / "logs/validation-attempt1-samples.csv").splitlines()
    header = lines[0].split(",")
    rows = [line.split(",") for line in lines[1:] if line.strip()]
    idx = {name: i for i, name in enumerate(header)}
    used = [int(row[idx["used_mib"]]) for row in rows]
    free = [int(row[idx["free_mib"]]) for row in rows]
    util = [int(row[idx["util_percent"]]) for row in rows]
    temp = [int(row[idx["temperature_c"]]) for row in rows]
    streak = cur = 0
    for value in util:
        cur = cur + 1 if value >= 80 else 0
        streak = max(streak, cur)

    baseline_points = []
    for line in read_text(EVID / "logs/validation-attempt1-baseline.txt").splitlines():
        if re.match(r"^\d+, \d+, \d+, \d+, [\d.]+$", line.strip()):
            baseline_points.append([int(v.strip()) for v in line.split(",")[:4]])

    guard_text = read_text(EVID / "logs/validation-attempt1-guard.log")
    agg_line = [line for line in guard_text.splitlines() if line.startswith("guard_aggregates")][-1]
    agg = dict(re.findall(r"(\w+)=([\d.\-]+)", agg_line))
    guard_baseline = [line for line in guard_text.splitlines() if line.startswith("guard_baseline")][-1]
    gbase = dict(re.findall(r"(\w+)=([\d.\-]+)", guard_baseline))

    first_free = baseline_points[0][1]
    floor = min(10240, first_free - 3072)
    baseline_used = int(agg["baseline_used"])
    peak = max(used)
    samples = {
        "rows": len(rows),
        "header": header,
        "used_min": min(used),
        "used_peak": peak,
        "used_max": max(used),
        "free_min": min(free),
        "util_max": max(util),
        "util_streak_ge80": streak,
        "temp_max": max(temp),
        "first_baseline_point": baseline_points[0],
        "baseline_points": baseline_points,
        "guard_baseline_used": baseline_used,
        "guard_baseline_free": int(gbase["free_mib"]),
        "delta_vs_guard_baseline": peak - baseline_used,
        "delta_vs_first_baseline": peak - baseline_points[0][0],
        "delta_vs_samples_min": peak - min(used),
        "floor_recomputed": floor,
        "floor_formula": f"min(10240, {first_free}-3072)",
        "free_min_above_floor": min(free) > floor,
        "open_gate_free_ge_10240": first_free >= 10240,
        "guard_aggregates_line": agg_line,
        "guard_aggregates_match": {
            "samples": int(agg["samples"]) == len(rows),
            "baseline_used": int(agg["baseline_used"]) == baseline_used,
            "used_peak": int(agg["used_peak"]) == peak,
            "delta_peak": int(agg["delta_peak"]) == peak - baseline_used,
            "free_min": int(agg["free_min"]) == min(free),
            "util_max": int(agg["util_max"]) == max(util),
            "util_streak": int(agg["util_streak"]) == streak,
        },
    }
    facts_text = read_text(EVID / "logs/validation-attempt1-facts.txt")
    sampler = re.search(r"sampler_samples=(\d+) used_peak=(\d+) free_min=(\d+) util_max=(\d+) process_exited=(\w+) residual=(\d+)", facts_text)
    samples["facts_sampler"] = {
        "samples": int(sampler.group(1)),
        "used_peak": int(sampler.group(2)),
        "free_min": int(sampler.group(3)),
        "util_max": int(sampler.group(4)),
        "process_exited": sampler.group(5),
        "residual": int(sampler.group(6)),
    }
    samples["facts_matches_recompute"] = (
        samples["facts_sampler"]["samples"] == len(rows)
        and samples["facts_sampler"]["used_peak"] == peak
        and samples["facts_sampler"]["free_min"] == min(free)
        and samples["facts_sampler"]["util_max"] == max(util)
        and samples["facts_sampler"]["residual"] == 0
    )
    exit_codes = re.search(r"exit_codes: (.*)", facts_text).group(1)
    samples["exit_codes_line"] = exit_codes
    samples["exit_codes_all_zero"] = all(part.endswith("=0") for part in exit_codes.split())
    samples["failed_entities_zero"] = "failed_entities=0" in facts_text
    samples["guard_aggregate_defect_present"] = "guard_aggregate=System.Collections.Specialized.OrderedDictionary" in facts_text
    out["samples"] = samples

    # ---------- 2. sha guards: 25 行 + 独立重算 ----------
    guard_lines = [line for line in read_text(EVID / "logs/validation-attempt1-sha-guards.txt").splitlines() if line.strip()]
    guard_rows = []
    for line in guard_lines:
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        row = {"kind": line.split()[1], "match": fields.get("match") == "True"}
        if "target" in fields:
            row["target"] = fields["target"]
            row["file"] = fields.get("file")
            row["claimed"] = fields.get("sha256")
            row["actual"] = sha256(FREEZE / fields["file"]) if (FREEZE / fields["file"]).exists() else None
            row["bytes_claimed"] = int(fields.get("bytes", "0"))
            row["bytes_actual"] = (FREEZE / fields["file"]).stat().st_size if (FREEZE / fields["file"]).exists() else None
        elif "path" in fields:
            row["path"] = fields["path"]
            row["claimed"] = fields.get("sha256")
            row["actual"] = sha256(REPO / fields["path"]) if (REPO / fields["path"]).exists() else None
        row["recompute_match"] = row["claimed"] == row["actual"]
        guard_rows.append(row)
    fixture_paths = [row["path"] for row in guard_rows if row["kind"] == "fixture"]
    out["sha_guards"] = {
        "lines": len(guard_rows),
        "exes": sum(1 for row in guard_rows if row["kind"] == "exe"),
        "fixtures": len(fixture_paths),
        "references": sum(1 for row in guard_rows if row["kind"] == "reference"),
        "all_claimed_match": all(row["match"] for row in guard_rows),
        "all_recompute_match": all(row["recompute_match"] for row in guard_rows),
        "rows": guard_rows,
    }

    # ---------- 3. 窗后 binary-shas / runner ----------
    binary_lines = [line for line in read_text(EVID / "logs/validation-attempt1-binary-shas.txt").splitlines() if "|" in line]
    binary_rows = []
    for line in binary_lines:
        parts = [part.strip() for part in line.split("|")]
        if len(parts) < 3 or not parts[0].endswith(".exe"):
            continue
        binary_rows.append(
            {
                "file": parts[0],
                "bytes": int(parts[1]),
                "sha": parts[2],
                "actual": sha256(FREEZE / parts[0]) if (FREEZE / parts[0]).exists() else None,
            }
        )
    runner = REPO / "target/s4-branch-closure/window/run-validation-attempt1.ps1"
    out["binary_shas"] = {
        "rows": len(binary_rows),
        "all_match": all(row["actual"] == row["sha"] for row in binary_rows),
        "runs": binary_rows,
        "runner_sha": sha256(runner),
        "runner_bytes": runner.stat().st_size,
        "runner_claimed_in_readme": "e737966384a8077e15a0995d9565eff0448ea10fad683fb67ed1fb8446c62109",
        "runner_match": sha256(runner) == "e737966384a8077e15a0995d9565eff0448ea10fad683fb67ed1fb8446c62109",
    }

    # ---------- 4. 逐实体日志解析 ----------
    entity_logs = {}
    branch_text = read_text(EVID / "logs/validation-attempt1-g10_p5_branch_closure_cuda.log")
    cases = []
    for match in re.finditer(r"case=([a-z0-9\-]+) kernel=(\w+) nacon=(\d+) ref_count=(\d+) failures=(\d+)", branch_text):
        cases.append(
            {
                "case": match.group(1),
                "kernel": match.group(2),
                "nacon": int(match.group(3)),
                "ref_count": int(match.group(4)),
                "failures": int(match.group(5)),
            }
        )
    record = []
    for match in re.finditer(r"RECORD-ONLY limits case=([a-z0-9\-]+) device_count=(\d+) ref_count=(\d+)", branch_text):
        record.append({"case": match.group(1), "device_count": int(match.group(2)), "ref_count": int(match.group(3))})
    entity_logs["branch_closure"] = {
        "cases": cases,
        "case_count": len(cases),
        "all_counts_match": all(row["nacon"] == row["ref_count"] for row in cases),
        "all_failures_zero": all(row["failures"] == 0 for row in cases),
        "record_only": record,
        "has_test_ok": "test result: ok" in branch_text,
    }

    launcher_text = read_text(EVID / "logs/validation-attempt1-g10_p5_nvrtc_launcher.log")
    launcher = []
    for match in re.finditer(
        r"case=([a-z0-9\-]+) kind=(\w+) nacon=(\d+) nccd=(\d+) bit_equal=(\d+)/(\d+) max_dist_ulp=(\d+) max_pos_ulp=(\d+) max_frame_ulp=(\d+) failures=(\d+)",
        launcher_text,
    ):
        launcher.append(
            {
                "case": match.group(1),
                "kind": match.group(2),
                "nacon": int(match.group(3)),
                "bit_equal": f"{match.group(5)}/{match.group(6)}",
                "max_dist_ulp": int(match.group(7)),
                "max_pos_ulp": int(match.group(8)),
                "max_frame_ulp": int(match.group(9)),
                "failures": int(match.group(10)),
            }
        )
    entity_logs["launcher"] = {
        "cases": launcher,
        "case_count": len(launcher),
        "bit_full": all(row["bit_equal"].split("/")[0] == row["bit_equal"].split("/")[1] for row in launcher),
        "all_failures_zero": all(row["failures"] == 0 for row in launcher),
        "hfield_capsule_present": any(row["case"] == "hfield-capsule" for row in launcher),
        "hfield_capsule_bit": [row["bit_equal"] for row in launcher if row["case"] == "hfield-capsule"],
    }

    other = {}
    for name in [
        "g10_p5_multicontact_cuda",
        "g10_p5_primitive_cuda",
        "g10_p5_primitive_plane_cuda",
        "g10_p5_primitive_sphere_capsule_cuda",
        "g10_p5_primitive_branches_cuda",
        "g10_p5_mesh_multicontact_m2_cuda",
        "g10_p5_mesh_multicontact_m3_cuda",
        "stage2_equivalence_window",
    ]:
        text = read_text(EVID / f"logs/validation-attempt1-{name}.log")
        other[name] = {
            "has_test_ok": "test result: ok" in text,
            "failures_zero_lines": text.count("failures=0"),
            "nonzero_failure_lines": [
                line.strip()
                for line in text.splitlines()
                if re.search(r"failures=[1-9]", line)
            ],
            "true_count": text.count("true"),
        }
    entity_logs["other"] = other
    out["entity_logs"] = entity_logs

    # ---------- 5. 证据文件 sha 与 README §12 声明 ----------
    claimed = {
        "logs/validation-attempt1-facts.txt": "94c204bc",
        "logs/validation-attempt1-guard.log": "2bb90c90",
        "logs/validation-attempt1-samples.csv": "c5c9d302",
        "logs/validation-attempt1-baseline.txt": "9ae9eb2b",
        "logs/validation-attempt1-sha-guards.txt": "ca77a407",
        "logs/validation-attempt1-machine-load.txt": "9b58397c",
        "logs/validation-attempt1-binary-shas.txt": "07c3e4a1",
    }
    file_sha = {}
    for rel, prefix in claimed.items():
        actual = sha256(EVID / rel)
        file_sha[rel] = {"claimed_prefix": prefix, "actual": actual, "match": actual.startswith(prefix)}
    out["evidence_sha_claims"] = file_sha

    # ---------- 6. guard.log 原始 vs 证据副本（BOM 关系） ----------
    copy = (EVID / "logs/validation-attempt1-guard.log").read_bytes()
    original = (ATTEMPT / "guard.log").read_bytes() if (ATTEMPT / "guard.log").exists() else None
    out["guard_bom"] = {
        "copy_bytes": len(copy),
        "copy_sha": sha256(EVID / "logs/validation-attempt1-guard.log"),
        "original_exists": original is not None,
        "original_bytes": len(original) if original is not None else None,
        "original_sha": sha256(ATTEMPT / "guard.log") if original is not None else None,
        "copy_no_bom": not (copy[:3] == b"\xef\xbb\xbf"),
        "original_has_bom": bool(original is not None and original[:3] == b"\xef\xbb\xbf"),
        "content_equal_after_strip": bool(original is not None and original.lstrip(b"\xef\xbb\xbf") == copy),
    }

    # ---------- 7. 零编译扫描 target/debug（窗口时段，排除 s4 目录） ----------
    start = datetime.strptime(WINDOW_START, "%Y-%m-%dT%H:%M:%S")
    end = datetime.strptime(WINDOW_END, "%Y-%m-%dT%H:%M:%S")
    modified = []
    for path in (REPO / "target/debug").rglob("*"):
        if not path.is_file():
            continue
        if "s4" in path.parts:
            continue
        mtime = datetime.fromtimestamp(path.stat().st_mtime)
        if start - timedelta(minutes=5) <= mtime <= end + timedelta(minutes=5):
            modified.append(str(path.relative_to(REPO)))
            if len(modified) >= 20:
                break
    out["zero_compile"] = {"window": f"{WINDOW_START}..{WINDOW_END}", "modified_files": modified, "count": len(modified)}

    # ---------- 8. window-receipt 关键数字 ----------
    receipt = json.loads(read_text(EVID / "window-receipt.json"))
    out["window_receipt"] = {
        "task_id": receipt.get("task_id"),
        "top_keys": list(receipt.keys()),
    }
    for key in ("window", "samples", "sha_guard", "entities", "results", "u7_u8", "exits"):
        if key in receipt:
            out["window_receipt"][key] = receipt[key]

    OUT.write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    summary = {
        "samples": {k: samples[k] for k in ("rows", "used_min", "used_peak", "free_min", "util_max", "util_streak_ge80", "delta_vs_guard_baseline", "delta_vs_first_baseline", "delta_vs_samples_min", "floor_recomputed", "free_min_above_floor", "open_gate_free_ge_10240")},
        "guard_aggregates_match": samples["guard_aggregates_match"],
        "facts_matches_recompute": samples["facts_matches_recompute"],
        "exit_all_zero": samples["exit_codes_all_zero"],
        "sha_guards": {k: out["sha_guards"][k] for k in ("lines", "exes", "fixtures", "references", "all_claimed_match", "all_recompute_match")},
        "binary_shas": {k: out["binary_shas"][k] for k in ("rows", "all_match", "runner_match")},
        "branch": {k: out["entity_logs"]["branch_closure"][k] for k in ("case_count", "all_counts_match", "all_failures_zero")},
        "record": out["entity_logs"]["branch_closure"]["record_only"],
        "launcher": {k: out["entity_logs"]["launcher"][k] for k in ("case_count", "bit_full", "all_failures_zero", "hfield_capsule_present", "hfield_capsule_bit")},
        "evidence_sha_ok": all(v["match"] for v in file_sha.values()),
        "guard_bom": out["guard_bom"],
        "zero_compile": out["zero_compile"],
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1, default=str))
    return 0


if __name__ == "__main__":
    sys.exit(main())
