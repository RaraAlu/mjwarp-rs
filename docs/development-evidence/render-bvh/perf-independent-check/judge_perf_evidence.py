"""QA judge: perf evidence integrity + independence checks + 18ae715 (S8 fix) verification.

Independent checks:
  * delivered evidence file hashes vs README/receipt claims and git blobs (EOL normalized);
  * raw-timings windows == gpu-run/repeat-run log windows; warmup/iterations and series lengths;
  * official_summary values are exactly the s8perf[stat] values printed in the same run's log;
  * nvidia-smi-samples.txt claims (329 points, min/median/max, baseline/end);
  * "relative baseline only" scope: no absolute/cross-hardware/upstream comparison language, and
    the unmet list covers release build / sub-100 ms sampling / nworld>1;
  * diff scope of eadae87 and 18ae715 (no product src changes), size ratchet;
  * 18ae715 three fixes: discriminating tamper assertion in render_e2e.rs, fixture provenance
    without dangling reference, S8 README machine-forbidden terms gone; S8 numbers unchanged;
  * QA's own perf runs / memory cycles / exclusivity sampling (from target/qa-perf).
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import statistics
import subprocess

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
PERF = ROOT / "docs/development-evidence/render-bvh/perf"
QA = ROOT / "target/qa-perf/out"
BASE_PERF = "eadae87"
S8_FIX = "18ae715"
S8_EVIDENCE = ROOT / "docs/development-evidence/render-bvh/s8-e2e"


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> bytes:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=True).stdout


def norm(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def text_of(path: pathlib.Path) -> str:
    data = path.read_bytes()
    if data.startswith(b"\xef\xbb\xbf"):
        return data.decode("utf-8-sig", "replace")
    if data.startswith(b"\xff\xfe") or data.startswith(b"\xfe\xff"):
        return data.decode("utf-16", "replace")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("utf-16", "replace")


def main() -> int:
    receipt = json.loads((PERF / "receipt.json").read_text(encoding="utf-8"))
    readme = (PERF / "README.md").read_text(encoding="utf-8")
    raw = json.loads((PERF / "raw-timings.json").read_text(encoding="utf-8"))
    report: dict = {}

    # 1) evidence integrity
    readme_hashes = {
        "raw-timings.json": "731704df427a490ce77637062ba5a1d9763cf160b8a44373c609c787916a2498",
        "nvidia-smi-samples.txt": "7b45dc4b7825d97250831fd76cdbb64dcf1ed104a9d0269e4e28d322081808f4",
        "gpu-run.log": "3be587af67b0c4aaa6ae6a48b9574939e20139d59783119de99569453add30ea",
        "repeat-run.log": "e1da947c141cb40bbfe5cf3ee93f15855dba53a47b02bf7b91910466e4bb2e4b",
        "regression.log": "7b0d7abc2f6e7ee47945281a13c5dceebc039584a9792cca618f8bef14148a5b",
    }
    files = {}
    for name in receipt["evidence_files"]:
        path = PERF / name
        committed = git("show", f"{BASE_PERF}:docs/development-evidence/render-bvh/perf/{name}")
        files[name] = {
            "bytes": path.stat().st_size,
            "sha256": sha(path),
            "readme_sha256": readme_hashes.get(name),
            "readme_claim_match": readme_hashes.get(name) in (None, sha(path)),
            "equals_committed_blob": path.read_bytes() == committed,
            "equals_committed_blob_eol": norm(path.read_bytes()) == norm(committed),
        }
    report["evidence_files"] = files
    report["evidence_list_matches"] = sorted(receipt["evidence_files"]) == sorted(files)

    # 2) windows / series / summary-same-source
    gpu = text_of(PERF / "gpu-run.log")
    repeat = text_of(PERF / "repeat-run.log")
    windows = {
        "raw_a": raw["measurement_a"]["window"],
        "raw_b": raw["measurement_b"]["window"],
        "gpu_log_start": re.search(r"perf window: start (\S+)", gpu).group(1),
        "gpu_log_end": re.search(r"window: end (\S+)", gpu).group(1),
    }
    series = {}
    for pass_key in ("measurement_a", "measurement_b"):
        series[pass_key] = []
        for mode in raw[pass_key]["modes"]:
            series[pass_key].append(
                {
                    "mode": mode["mode"],
                    "frame_len": len(mode["frame_ms"]),
                    "launch_len": len(mode["launch_ms"]),
                    "warmup": mode["warmup"],
                    "iterations": mode["iterations"],
                }
            )
    stat_lines = re.findall(
        r"s8perf\[stat\] (\w+): rays=\d+ warmup=(\d+) iterations=(\d+) launch\(min,median,p95\)=\(([\d.]+),([\d.]+),([\d.]+)\)ms frame\(min,median,p95\)=\(([\d.]+),([\d.]+),([\d.]+)\)ms",
        gpu,
    )
    stat_lines_b = re.findall(
        r"s8perf\[stat\] (\w+): rays=\d+ warmup=(\d+) iterations=(\d+) launch\(min,median,p95\)=\(([\d.]+),([\d.]+),([\d.]+)\)ms frame\(min,median,p95\)=\(([\d.]+),([\d.]+),([\d.]+)\)ms",
        repeat,
    )

    def summary_from_log(lines):
        result = {}
        for line in lines:
            mode = line[0]
            result[mode] = {
                "warmup": int(line[1]),
                "iterations": int(line[2]),
                "launch_ms": {"min": float(line[3]), "median": float(line[4]), "p95": float(line[5])},
                "frame_ms": {"min": float(line[6]), "median": float(line[7]), "p95": float(line[8])},
            }
        return result

    official = {
        "measurement_a": raw["measurement_a"]["official_summary"],
        "measurement_b": raw["measurement_b"]["official_summary"],
    }
    log_summaries = {
        "measurement_a": summary_from_log(stat_lines),
        "measurement_b": summary_from_log(stat_lines_b),
    }
    report["protocol"] = {
        "windows": windows,
        "warmup": raw["warmup"],
        "iterations": raw["iterations"],
        "series": series,
        "log_summary_a": log_summaries["measurement_a"],
        "log_summary_b": log_summaries["measurement_b"],
        "official_summary_a": official["measurement_a"],
        "official_summary_b": official["measurement_b"],
    }
    summary_matches = True
    summary_diffs = []
    for pass_key in ("measurement_a", "measurement_b"):
        for mode, log_values in log_summaries[pass_key].items():
            recorded = official[pass_key].get(mode) or {}
            for metric in ("launch_ms", "frame_ms"):
                for stat in ("min", "median", "p95"):
                    recorded_value = (recorded.get(metric) or {}).get(stat)
                    log_value = log_values[metric][stat]
                    same = recorded_value is not None and abs(recorded_value - log_value) <= 1e-9
                    if not same:
                        summary_diffs.append((pass_key, mode, metric, stat, recorded_value, log_value))
                    summary_matches = summary_matches and same
    # ratios from the official medians must reproduce the recorded ratios
    ratio_checks = []
    for pass_key in ("measurement_a", "measurement_b"):
        medians = {
            mode: official[pass_key][mode]["frame_ms"]["median"]
            for mode in ("single_spp1", "aa2_spp2")
        }
        ratios = raw[pass_key]["ratios"]
        computed = {
            "aa_over_single": medians["aa2_spp2"] / medians["single_spp1"],
            "single_ms_per_120k_rays": medians["single_spp1"],
            "aa_ms_per_120k_rays": medians["aa2_spp2"] / 4.0,
        }
        ratio_checks.append(
            {
                "pass": pass_key,
                "computed": {key: round(value, 3) for key, value in computed.items()},
                "recorded": ratios,
                "match": all(abs(round(computed[key], 3) - ratios[key]) <= 1e-9 for key in computed),
            }
        )
    report["checks"] = {
        "official_summary_equals_log": summary_matches,
        "official_summary_diffs": summary_diffs,
        "ratios_derive_from_official_medians": all(item["match"] for item in ratio_checks),
        "ratio_checks": ratio_checks,
        "raw_window_a_equals_log": windows["raw_a"]["start"] == windows["gpu_log_start"]
        and windows["raw_a"]["end"] == windows["gpu_log_end"],
        "series_30_each": all(
            entry["frame_len"] == 30 and entry["launch_len"] == 30 and entry["warmup"] == 5
            for entries in series.values()
            for entry in entries
        ),
    }

    # 3) nvidia samples
    samples = [int(line.strip()) for line in text_of(PERF / "nvidia-smi-samples.txt").splitlines() if line.strip()]
    report["nvidia_samples"] = {
        "samples": len(samples),
        "min": min(samples),
        "median": int(statistics.median(samples)),
        "max": max(samples),
        "first": samples[0],
        "last": samples[-1],
        "receipt_claims": receipt["memory"]["nvidia_smi_samples"],
        "matches": (
            len(samples) == receipt["memory"]["nvidia_smi_samples"]["samples"]
            and min(samples) == receipt["memory"]["nvidia_smi_samples"]["min_mib"]
            and int(statistics.median(samples)) == receipt["memory"]["nvidia_smi_samples"]["median_mib"]
            and max(samples) == receipt["memory"]["nvidia_smi_samples"]["max_mib"]
            and samples[0] == receipt["memory"]["nvidia_smi_samples"]["desktop_baseline_mib"]
            and samples[-1] == receipt["memory"]["nvidia_smi_samples"]["window_end_mib"]
        ),
    }

    # 4) scope language (relative baseline only) and unmet coverage
    absolute_terms = ["绝对性能", "绝对性能指标", "固定性能指标", "跨硬件比较", "上游比较", "比上游快", "比上游慢"]
    combined = readme + json.dumps(receipt, ensure_ascii=False)
    report["scope"] = {
        "relative_baseline_declared": "相对基准" in readme,
        "absolute_claim_terms": {term: combined.count(term) for term in absolute_terms},
        "contains_invariant_line": "不宣称固定性能指标" in readme,
        "unmet_items": receipt["unmet"],
        "unmet_covers": {
            "release_build": any("release" in item for item in receipt["unmet"]),
            "sub_100ms_sampling": any("亚 100 ms" in item or "100 ms" in item for item in receipt["unmet"]),
            "nworld>1": any("nworld>1" in item for item in receipt["unmet"]),
            "single_scene": any("scene-a" in item for item in receipt["unmet"]),
            "no_upstream_comparison": any("上游" in item for item in receipt["unmet"]),
        },
    }

    # 5) diff scopes
    def changed(base: str, target: str):
        return git("diff", "--name-only", base, target).decode().splitlines()

    perf_changed = changed("eadae87^", "eadae87")
    fix_changed = changed(f"{S8_FIX}^", S8_FIX)
    report["diff"] = {
        "eadae87": {
            "changed": perf_changed,
            "src_changes": [path for path in perf_changed if path.startswith("src/")],
            "product_semantic_changes": [
                path for path in perf_changed if path.startswith(("src/", "fixtures/"))
            ],
        },
        "18ae715": {
            "changed": fix_changed,
            "src_changes": [path for path in fix_changed if path.startswith("src/")],
        },
    }
    render_perf = ROOT / "tests/render_perf.rs"
    report["size_ratchet"] = {
        "tests/render_perf.rs": {
            "lines": len(render_perf.read_text(encoding="utf-8").splitlines()),
            "bytes": render_perf.stat().st_size,
            "sha256": sha(render_perf),
            "receipt": receipt["size_ratchet"]["tests_render_perf_rs"],
            "matches": (
                len(render_perf.read_text(encoding="utf-8").splitlines())
                == receipt["size_ratchet"]["tests_render_perf_rs"]["lines"]
                and sha(render_perf) == receipt["size_ratchet"]["tests_render_perf_rs"]["sha256"]
            ),
            "under_500": len(render_perf.read_text(encoding="utf-8").splitlines()) < 500,
        }
    }

    # 6) 18ae715 fixes
    e2e = (ROOT / "tests/render_e2e.rs").read_text(encoding="utf-8")
    fixture = ROOT / "fixtures/render-bvh/scene-e2e/camlight-model.json"
    fixture_json = json.loads(fixture.read_text(encoding="utf-8"))
    s8_readme = (S8_EVIDENCE / "README.md").read_text(encoding="utf-8")
    s8_receipt = json.loads((S8_EVIDENCE / "receipt.json").read_text(encoding="utf-8"))
    script = ROOT / "scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py"
    report["s8_fix"] = {
        "discriminating_assertion_present": "tamper_diff" in e2e
        and "compare_pose_bits(&product.poses, &tampered.camera_data)" in e2e
        and "origin_mismatches.iter().any(|n| *n > 0)" in e2e,
        "fixture": {
            "lines": len(fixture.read_text(encoding="utf-8").splitlines()),
            "bytes": fixture.stat().st_size,
            "sha256": sha(fixture),
            "receipt": s8_receipt["input_fixture"],
            "matches_receipt": (
                len(fixture.read_text(encoding="utf-8").splitlines()) == s8_receipt["input_fixture"]["lines"]
                and fixture.stat().st_size == s8_receipt["input_fixture"]["bytes"]
                and sha(fixture) == s8_receipt["input_fixture"]["sha256"]
            ),
        },
        "provenance": {
            "sources": fixture_json["provenance"]["camera_data_reference"]["sources"],
            "all_sources_exist": all(
                (ROOT / source).exists() for source in fixture_json["provenance"]["camera_data_reference"]["sources"]
            ),
        },
        "script": {
            "lines": len(script.read_text(encoding="utf-8").splitlines()),
            "sha256": sha(script),
            "receipt_claim": s8_receipt["input_fixture"]["authoring_script"],
            "sha_matches": sha(script)
            == s8_receipt["input_fixture"]["authoring_script"]["sha256"],
            "lines_match": len(script.read_text(encoding="utf-8").splitlines())
            == s8_receipt["input_fixture"]["authoring_script"]["lines"],
        },
        "s8_readme_forbidden_terms": {
            term: s8_readme.count(term)
            for term in ("完整渲染通过", "完整R01通过", "完整 R01 通过", "产品渲染验收通过")
        },
        "s8_readme_upstream_equivalence_literal": [
            line.strip()
            for line in s8_readme.splitlines()
            if "上游完全等价" in line
        ],
        "s8_numbers_unchanged_from_qa_rerun": {
            "depth_seg_hash": "03efa95416bbd008",
            "rgb_hash": "ed50f750936c0275",
            "note": "QA reran render_e2e in this window: 5/5, 0-diff lines, perturbation 18,859/19,182/9,464",
        },
    }

    # 7) QA's own measurements
    def load(name):
        return json.loads((QA / name).read_text(encoding="utf-8"))

    qa_runs = []
    for name, tag in (("qa_perf_a.json", "A"), ("qa_perf_b.json", "B"), ("qa_perf_exclusive.json", "exclusive")):
        path = QA / name
        if not path.exists():
            continue
        data = load(name)
        single = data["single"]["summary"]["frame"]
        aa = data["aa"]["summary"]["frame"]
        qa_runs.append(
            {
                "tag": tag,
                "single_frame": single,
                "aa_frame": aa,
                "ratio": aa["median"] / single["median"],
                "single_ms_per_120k": single["median"],
                "aa_ms_per_120k": aa["median"] / 4.0,
                "window_s": (data["window"]["end_unix_ms"] - data["window"]["start_unix_ms"]) / 1000.0,
            }
        )
    mem = load("qa_mem.json")
    samples = [
        int(match.group(1))
        for match in (
            re.match(r"\d\d:\d\d:\d\d\.\d+ (\d+)", line)
            for line in (QA / "qa-window-mem.txt").read_text(encoding="ascii", errors="replace").splitlines()
        )
        if match
    ]
    apps_text = (QA / "qa-window-apps.txt").read_text(encoding="ascii", errors="replace")
    app_blocks = apps_text.split("t=")[1:]
    new_compute_names = set()
    for block in app_blocks:
        for line in block.splitlines():
            match = re.match(r"(\d+),\s*(.+)$", line.strip())
            if match:
                name = match.group(2).strip()
                if name.lower().endswith(".exe") and "windows" not in name.lower():
                    new_compute_names.add(name)
    report["qa_independent"] = {
        "perf_runs": qa_runs,
        "delivered_reference": {
            "single_median_a": 14.030,
            "single_median_b": 14.389,
            "aa_median_a": 36.174,
            "aa_median_b": 36.423,
            "ratio_a": 2.578,
            "ratio_b": 2.531,
        },
        "single_median_delta_vs_delivered_pct": [
            (run["single_frame"]["median"] - 14.030) / 14.030 * 100.0 for run in qa_runs
        ],
        "memory_cycles": {
            "free_process_start": mem["free_process_start"],
            "peak_bytes": mem["peak_bytes"],
            "residual_vs_process_start_bytes": mem["residual_vs_process_start_bytes"],
            "residual_between_cycles_bytes": mem["residual_between_cycles_bytes"],
            "cycle_peaks": [cycle["peak_bytes_vs_cycle_start"] for cycle in mem["cycles"]],
            "cycle_frame_free_growth": [cycle["frame_free_growth"] for cycle in mem["cycles"]],
            "cycle_destroy_free": [cycle["free_after_destroy"] for cycle in mem["cycles"]],
            "judgment": "one-time retention: first cycle peak includes kernel/module load, second "
            "cycle adds only the 2 MiB context, per-frame free series is constant and the free "
            "level after destroy is identical between cycles -> not a leak; the delivered "
            "52 MiB residual explanation is structurally supported (magnitude differs by process "
            "state/desktop baseline)",
        },
        "exclusivity_sample": {
            "samples": len(samples),
            "min": min(samples) if samples else None,
            "median": int(statistics.median(samples)) if samples else None,
            "max": max(samples) if samples else None,
            "first": samples[0] if samples else None,
            "last": samples[-1] if samples else None,
            "app_blocks": len(app_blocks),
            "qa_harness_seen_in_blocks": sum(1 for block in app_blocks if "qa-perf-harness" in block),
            "other_compute_processes": sorted(
                name
                for name in new_compute_names
                if "qa-perf-harness" not in name.lower()
                and any(key in name.lower() for key in ("python", "cargo", "test", "rustc", "mjwarp"))
            ),
        },
    }

    out = QA / "judge_perf_evidence.json"
    out.write_text(json.dumps(report, indent=1, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({k: report[k] for k in ("checks", "nvidia_samples", "scope", "diff", "s8_fix")}, indent=1, ensure_ascii=False)[:7000])
    print(json.dumps(report["qa_independent"], indent=1)[:4000])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
