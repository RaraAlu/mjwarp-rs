"""UNC-3 S3 独立复核复算脚本（CPU，不建 CUDA context、不跑捕获）。

只读输入：
- 13 个新 fixture + 2 个 authoring 脚本 + runner（本地 target，未入库）
- S3 证据：两个上游参考、precheck-cpu.json、logs/*
- 旧参考目录 fixtures/g10-p5/references/（只读 mtime/字节）

输出：target/qa-unc3-s3/verify_s3.json（本脚本自检，不写产品/证据目录）。
"""
import hashlib
import json
import re
import struct
import sys
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/g10-p5-branch-closure-capture"
OUT = Path(__file__).with_name("verify_s3.json")

WINDOW_START = "2026-10-27T09:37:42.194+08:00"
WINDOW_START_DT = "2026-10-27 09:37:42"
RUNNER = REPO / "target/s3-branch-closure/window/run-capture-attempt1.ps1"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def f32_bits(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", value))[0]


def ulp(a: float, b: float) -> int:
    ba, bb = f32_bits(a), f32_bits(b)
    if (ba >> 31) != (bb >> 31):
        # 跨 0 时按位串距离（简化：两侧绝对值求和 + 1）
        return abs(ba & 0x7FFFFFFF) + abs(bb & 0x7FFFFFFF) + 1
    return abs(ba - bb)


def parse_runner() -> dict:
    text = RUNNER.read_text(encoding="utf-8")
    fixtures = {}
    for line in text.splitlines():
        m = re.match(r"\s*'([^']+)'\s*=\s*'([0-9a-f]{64})'\s*$", line)
        if m:
            fixtures[m.group(1)] = m.group(2)
    script = re.search(r"\$scriptShaExpected\s*=\s*'([0-9a-f]{64})'", text)
    compare = re.search(r"\$compareShaExpected\s*=\s*'([0-9a-f]{64})'", text)
    return {
        "fixtures": fixtures,
        "capture_sha": script.group(1) if script else None,
        "compare_sha": compare.group(1) if compare else None,
    }


def xml_keyframe_qpos(path: Path):
    text = path.read_text(encoding="utf-8")
    m = re.search(r'<key[^>]*qpos="([^"]+)"', text)
    if not m:
        return None
    return [float(v) for v in m.group(1).split()]


def field_rows(field):
    """参考输出字段：dist/pos 为 [[...]] 行，取每行首元素/整行。"""
    return field


def case_ulp_compare(cpu_case, gpu_case):
    out = {"dist_ulp_max": None, "pos_ulp_max": None, "dist_bit_equal": None, "pos_bit_equal": None}
    cpu_dist = cpu_case.get("dist")
    gpu_dist = gpu_case["output"]["fields"].get("dist")
    cpu_pos = cpu_case.get("pos")
    gpu_pos = gpu_case["output"]["fields"].get("pos")
    if cpu_dist is not None and gpu_dist is not None:
        if len(cpu_dist) == len(gpu_dist):
            dmax = 0
            for row_c, row_g in zip(cpu_dist, gpu_dist):
                for a, b in zip(row_c, row_g):
                    dmax = max(dmax, ulp(a, b))
            out["dist_ulp_max"] = dmax
            out["dist_bit_equal"] = dmax == 0
    if cpu_pos is not None and gpu_pos is not None:
        if len(cpu_pos) == len(gpu_pos):
            pmax = 0
            for row_c, row_g in zip(cpu_pos, gpu_pos):
                for a, b in zip(row_c, row_g):
                    pmax = max(pmax, ulp(a, b))
            out["pos_ulp_max"] = pmax
            out["pos_bit_equal"] = pmax == 0
    return out


def main():
    result = {}

    # ---- A. 13 fixture + 2 脚本 sha：runner 内嵌 vs README 表 vs 实际 ----
    runner = parse_runner()
    readme = (EVID / "README.md").read_text(encoding="utf-8")
    readme_rows = {}
    for line in readme.splitlines():
        m = re.match(
            r"\|\s*(fixture|capture|precheck|compare|runner)\s*\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|\s*`([0-9a-f]{64})`",
            line,
        )
        if m:
            readme_rows[m.group(2)] = {"kind": m.group(1), "bytes": int(m.group(3)), "sha": m.group(4)}

    fixtures_check = []
    for path, expected in sorted(runner["fixtures"].items()):
        actual_sha = sha256(REPO / path)
        actual_bytes = (REPO / path).stat().st_size
        readme_entry = readme_rows.get(path)
        fixtures_check.append(
            {
                "path": path,
                "bytes": actual_bytes,
                "sha256": actual_sha,
                "runner_expected": expected,
                "runner_match": actual_sha == expected,
                "readme_match": bool(readme_entry and readme_entry["sha"] == actual_sha),
                "readme_bytes_match": bool(readme_entry and readme_entry["bytes"] == actual_bytes),
            }
        )
    result["fixtures"] = {
        "count": len(fixtures_check),
        "all_runner_match": all(row["runner_match"] for row in fixtures_check),
        "all_readme_match": all(row["readme_match"] for row in fixtures_check),
        "all_readme_bytes_match": all(row["readme_bytes_match"] for row in fixtures_check),
        "rows": fixtures_check,
    }

    scripts = {}
    for name, expected_sha, expected_bytes in [
        ("scripts/authoring/g10-p5/capture_branch_closure_gpu.py", runner["capture_sha"], 7403),
        ("scripts/authoring/g10-p5/precheck_branch_closure_cpu.py", "e26f936ec30b52ad57b18eeb92dba979540a82a0be527ca922e86806645d8387", 8624),
        ("scripts/compare_primitive_capture.py", runner["compare_sha"], 849),
        ("target/s3-branch-closure/window/run-capture-attempt1.ps1", "5f8f80c5c5629bb5567fdbad1bc3f933a9b9dfb01d5e2926fe1b59cf462fe3d2", 8745),
    ]:
        actual_sha = sha256(REPO / name)
        actual_bytes = (REPO / name).stat().st_size
        readme_entry = readme_rows.get(name)
        scripts[name] = {
            "bytes": actual_bytes,
            "sha256": actual_sha,
            "expected": expected_sha,
            "match": actual_sha == expected_sha,
            "bytes_match": actual_bytes == expected_bytes,
            "readme_match": bool(readme_entry and readme_entry["sha"] == actual_sha),
        }
    result["scripts"] = scripts

    # ---- B. 两份参考：字节/sha 三处（raw A、raw B、evidence） ----
    refs = {}
    for name, expected_sha, expected_bytes in [
        ("upstream-branch-closure-gpu.json", "24d89e9248cccece96baff2089954e438ca7956fe8a71ccc61df318208709db1", 16932),
        ("upstream-mesh-limits-gpu.json", "b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad", 4581),
    ]:
        copies = {
            "evidence": EVID / name,
            "run_a": REPO / "target/s3-branch-closure/capture-attempt1/run-a" / name,
            "run_b": REPO / "target/s3-branch-closure/capture-attempt1/run-b" / name,
        }
        shas = {key: sha256(path) for key, path in copies.items()}
        sizes = {key: path.stat().st_size for key, path in copies.items()}
        refs[name] = {
            "expected_sha256": expected_sha,
            "expected_bytes": expected_bytes,
            "shas": shas,
            "bytes": sizes,
            "all_three_same": len(set(shas.values())) == 1,
            "matches_expected": shas["evidence"] == expected_sha,
            "bytes_match": sizes["evidence"] == expected_bytes,
        }
    result["references"] = refs

    # ---- C. case 级：11 case、xml sha、qpos、count、two_captures、CPU vs GPU ULP ----
    branch = json.loads((EVID / "upstream-branch-closure-gpu.json").read_text(encoding="utf-8"))
    limits = json.loads((EVID / "upstream-mesh-limits-gpu.json").read_text(encoding="utf-8"))
    precheck = json.loads((EVID / "precheck-cpu.json").read_text(encoding="utf-8"))
    raw_a_branch = (REPO / "target/s3-branch-closure/capture-attempt1/run-a/upstream-branch-closure-gpu.json").read_bytes()
    raw_b_branch = (REPO / "target/s3-branch-closure/capture-attempt1/run-b/upstream-branch-closure-gpu.json").read_bytes()
    raw_a_limits = (REPO / "target/s3-branch-closure/capture-attempt1/run-a/upstream-mesh-limits-gpu.json").read_bytes()
    raw_b_limits = (REPO / "target/s3-branch-closure/capture-attempt1/run-b/upstream-mesh-limits-gpu.json").read_bytes()

    run_a_log = (EVID / "logs/capture-attempt1-run-a.log").read_text(encoding="utf-8")
    run_counts = {}
    for m in re.finditer(r"(?:branch|limits)/([a-z0-9\-]+): count=(\d+) two_captures=(\w+)", run_a_log):
        run_counts[m.group(1)] = {"count": int(m.group(2)), "two_captures": m.group(3) == "True"}

    rows = []
    for group, doc in (("branch", branch), ("limits", limits)):
        for name, case in doc["cases"].items():
            xml = REPO / case["xml"]
            xml_sha_actual = sha256(xml) if xml.exists() else None
            key_qpos = xml_keyframe_qpos(xml) if xml.exists() else None
            qpos_bits_recalc = [f32_bits(v) for v in case["qpos"]] if case.get("qpos") else None
            cpu = precheck["cases"].get(name)
            cmp = case_ulp_compare(cpu, case) if cpu else None
            rows.append(
                {
                    "case": name,
                    "group": group,
                    "xml": case["xml"],
                    "xml_exists": xml.exists(),
                    "xml_sha_match": xml_sha_actual == case["xml_sha256"],
                    "qpos_len": len(case.get("qpos") or []),
                    "qpos_matches_xml_keyframe": bool(
                        key_qpos is not None
                        and len(key_qpos) == len(case["qpos"])
                        and all(abs(a - b) <= 4e-7 for a, b in zip(key_qpos, case["qpos"]))
                    ),
                    "qpos_bits_recalc_match": qpos_bits_recalc == case.get("qpos_bits"),
                    "count": case["output"]["count"],
                    "two_captures": case.get("two_captures_bitwise_identical"),
                    "run_log_count": (run_counts.get(name) or {}).get("count"),
                    "run_log_two_captures": (run_counts.get(name) or {}).get("two_captures"),
                    "precheck_count": cpu.get("count") if cpu else None,
                    "precheck_qpos_bits_match": bool(cpu and cpu.get("qpos_bits") == case.get("qpos_bits")),
                    "cpu_vs_gpu": cmp,
                }
            )
    result["cases"] = {
        "branch_cases": len(branch["cases"]),
        "limits_cases": len(limits["cases"]),
        "total": len(rows),
        "raw_a_eq_raw_b_branch": raw_a_branch == raw_b_branch,
        "raw_a_eq_raw_b_limits": raw_a_limits == raw_b_limits,
        "all_two_captures": all(r["two_captures"] for r in rows),
        "all_run_log_counts_match": all(r["run_log_count"] == r["count"] for r in rows),
        "all_xml_sha_match": all(r["xml_sha_match"] for r in rows),
        "all_qpos_match_xml": all(r["qpos_matches_xml_keyframe"] for r in rows),
        "all_qpos_bits_recalc": all(r["qpos_bits_recalc_match"] for r in rows),
        "precheck_count_matches": sum(1 for r in rows if r["precheck_count"] == r["count"]),
        "precheck_qpos_bits_matches": sum(1 for r in rows if r["precheck_qpos_bits_match"]),
        "rows": rows,
        "provenance": {
            "branch": branch.get("provenance", {}),
            "limits": limits.get("provenance", {}),
            "branch_purpose": branch.get("purpose"),
            "limits_purpose": limits.get("purpose"),
        },
        "precheck_summary": {
            "result": precheck.get("result"),
            "failures": precheck.get("failures"),
            "u3_corner_union": precheck.get("u3_corner_union"),
            "u3_covers_8": precheck.get("u3_covers_8"),
            "mesh_stats": {
                name: precheck["cases"][name].get("mesh")
                for name in ("mesh-mesh-two-assets", "mesh-limits-cone-box", "mesh-limits-prism-box")
            },
        },
    }

    # ---- D. 采样重算（samples.csv + guard.log） ----
    samples = []
    with open(EVID / "logs/capture-attempt1-samples.csv", encoding="utf-8") as handle:
        header = handle.readline().strip().split(",")
        for line in handle:
            parts = line.strip().split(",")
            if len(parts) != len(header):
                continue
            samples.append(
                {
                    "seq": int(parts[0]),
                    "used": int(parts[2]),
                    "free": int(parts[3]),
                    "util": int(parts[4]),
                }
            )
    baseline_points = []
    for line in (EVID / "logs/capture-attempt1-baseline.txt").read_text(encoding="utf-8").splitlines():
        if re.match(r"^\d+, \d+, \d+, \d+, [\d.]+$", line.strip()):
            baseline_points.append([int(v.strip()) for v in line.split(",")[:4]])

    guard_text = (EVID / "logs/capture-attempt1-guard.log").read_text(encoding="utf-8")
    agg = dict(re.findall(r"(\w+)=([\d.\-]+)", guard_text.split("guard_aggregates")[-1]))

    used = [s["used"] for s in samples]
    free = [s["free"] for s in samples]
    util = [s["util"] for s in samples]
    streak = cur = 0
    for value in util:
        cur = cur + 1 if value >= 80 else 0
        streak = max(streak, cur)

    baseline_used = int(agg.get("baseline_used", 0))
    used_peak = max(used)
    first_baseline_used = baseline_points[0][0] if baseline_points else None
    computed = {
        "samples": len(samples),
        "baseline_used_guard": baseline_used,
        "used_peak": used_peak,
        "used_min": min(used),
        "delta_vs_guard_baseline": used_peak - baseline_used,
        "first_baseline_used": first_baseline_used,
        "delta_vs_first_baseline": used_peak - first_baseline_used if first_baseline_used is not None else None,
        "delta_vs_samples_min": used_peak - min(used),
        "free_min": min(free),
        "util_max": max(util),
        "util_streak_ge80": streak,
        "guard_floor_expected": min(10240, (baseline_points[0][1] - 3072)) if baseline_points else None,
        "free_min_above_floor": (min(free) > min(10240, (baseline_points[0][1] - 3072))) if baseline_points else None,
        "temp_max": max(s["util"] for s in samples) and None,
    }
    computed["temp_max"] = None
    result["samples"] = {
        "header": header,
        "computed": computed,
        "guard_aggregates_raw": agg,
        "baseline_points": baseline_points,
        "matches_guard_aggregates": {
            "samples": int(agg.get("samples", -1)) == computed["samples"],
            "baseline_used": int(agg.get("baseline_used", -1)) == computed["baseline_used_guard"],
            "used_peak": int(agg.get("used_peak", -1)) == computed["used_peak"],
            "delta_peak": int(agg.get("delta_peak", -1)) == computed["delta_vs_guard_baseline"],
            "free_min": int(agg.get("free_min", -1)) == computed["free_min"],
            "util_max": int(agg.get("util_max", -1)) == computed["util_max"],
            "util_streak": int(agg.get("util_streak", -1)) == computed["util_streak_ge80"],
        },
    }

    # ---- E. 旧参考零读写：11 文件 mtime + 字节 ----
    old = []
    for path in sorted((REPO / "fixtures/g10-p5/references").glob("*.json")):
        stat = path.stat()
        old.append(
            {
                "name": path.name,
                "bytes": stat.st_size,
                "mtime": __import__("datetime").datetime.fromtimestamp(stat.st_mtime).strftime("%Y-%m-%d %H:%M:%S"),
                "sha256_prefix16": sha256(path)[:16],
            }
        )
    result["old_references"] = {
        "count": len(old),
        "all_mtime_10_25": all(row["mtime"].startswith("2026-10-25") for row in old),
        "rows": old,
    }

    # ---- F. 窗口日志措辞/编译检查 ----
    all_logs = "\n".join(
        (EVID / "logs" / name).read_text(encoding="utf-8", errors="replace")
        for name in [
            "capture-attempt1-baseline.txt",
            "capture-attempt1-compare.log",
            "capture-attempt1-facts.txt",
            "capture-attempt1-guard.log",
            "capture-attempt1-run-a.log",
            "capture-attempt1-run-b.log",
        ]
    )
    result["window_checks"] = {
        "rust_compile_markers_in_logs": [
            marker
            for marker in ("Compiling ", "rustc ", "cargo build", "cargo test")
            if marker in all_logs
        ],
        "sha_ok_line_present_in_logs": "SHA_OK" in all_logs,
        "run_a_wall": re.search(r"runA_wall=([\d.]+)", all_logs).group(1) if re.search(r"runA_wall=([\d.]+)", all_logs) else None,
        "run_b_wall": re.search(r"runB_wall=([\d.]+)", all_logs).group(1) if re.search(r"runB_wall=([\d.]+)", all_logs) else None,
        "run_a_exit": re.search(r"runA_exit=(\d+)", all_logs).group(1) if re.search(r"runA_exit=(\d+)", all_logs) else None,
        "run_b_exit": re.search(r"runB_exit=(\d+)", all_logs).group(1) if re.search(r"runB_exit=(\d+)", all_logs) else None,
        "compare_pass_count": all_logs.count("RAW BYTE-IDENTICAL: PASS"),
    }

    OUT.write_text(json.dumps(result, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    summary = {
        "fixtures_13_all_match": result["fixtures"]["all_runner_match"] and result["fixtures"]["all_readme_match"],
        "scripts_all_match": all(v["match"] for v in result["scripts"].values()),
        "refs_three_way_same": all(v["all_three_same"] and v["matches_expected"] for v in result["references"].values()),
        "cases_total": result["cases"]["total"],
        "all_two_captures": result["cases"]["all_two_captures"],
        "all_run_counts_match": result["cases"]["all_run_log_counts_match"],
        "all_xml_sha": result["cases"]["all_xml_sha_match"],
        "all_qpos_xml": result["cases"]["all_qpos_match_xml"],
        "precheck_count_matches": result["cases"]["precheck_count_matches"],
        "precheck_qpos_bits_matches": result["cases"]["precheck_qpos_bits_matches"],
        "samples": result["samples"]["computed"],
        "guard_aggregates_match": result["samples"]["matches_guard_aggregates"],
        "old_refs_untouched_mtime": result["old_references"]["all_mtime_10_25"],
        "window": result["window_checks"],
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
