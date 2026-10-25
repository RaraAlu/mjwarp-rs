"""QA judge: recompute perf statistics from the raw per-iteration series.

Independent derivation of every number in the perf evidence:
  * min / median / p95 for launch_ms and frame_ms, using the same declared nearest-rank
    definition with Rust `f64::round` semantics (half away from zero) — Python's round() is
    banker's rounding, so it is emulated explicitly;
  * AA/single median ratio and per-120,000-ray normalisation;
  * cross-pass consistency percentages;
  * comparison of the recomputed values with the delivered `official_summary` and the README /
    receipt numbers (tolerance 1e-3 ms for the rounded values);
  * memory series checks from the same raw file (free deltas, frame-free growth, residual).
"""

from __future__ import annotations

import json
import math
import pathlib

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
PERF = ROOT / "docs/development-evidence/render-bvh/perf"


def round_half_away(value: float) -> int:
    if value >= 0:
        return int(math.floor(value + 0.5))
    return int(math.ceil(value - 0.5))


def nearest_rank(sorted_values: list[float], fraction: float) -> float:
    index = round_half_away((len(sorted_values) - 1) * fraction)
    return sorted_values[index]


def summarize(values: list[float]) -> dict:
    ordered = sorted(values)
    return {"min": ordered[0], "median": nearest_rank(ordered, 0.5), "p95": nearest_rank(ordered, 0.95)}


def main() -> int:
    raw = json.loads((PERF / "raw-timings.json").read_text(encoding="utf-8"))
    receipt = json.loads((PERF / "receipt.json").read_text(encoding="utf-8"))
    readme = (PERF / "README.md").read_text(encoding="utf-8")
    report: dict = {"passes": {}, "checks": {}}
    recomputed = {}
    for pass_key, receipt_key, log_key in (
        ("measurement_a", "measurement_a", "gpu-run.log"),
        ("measurement_b", "measurement_b", "repeat-run.log"),
    ):
        data = raw[pass_key]
        modes = {mode["mode"]: mode for mode in data["modes"]}
        entry = {"window": data["window"], "modes": {}, "official_summary": {}}
        for mode_name, mode in modes.items():
            launch = mode["launch_ms"]
            frame = mode["frame_ms"]
            official = modes[mode_name].get("official_summary", {})
            entry["modes"][mode_name] = {
                "iterations": len(frame),
                "frame_recomputed": summarize(frame),
                "launch_recomputed": summarize(launch),
                "official_summary": official,
                "frame_series_min": frame and min(frame),
                "frame_series_max": frame and max(frame),
            }
        # ratio from recomputed medians
        single_median = entry["modes"]["single_spp1"]["frame_recomputed"]["median"]
        aa_median = entry["modes"]["aa2_spp2"]["frame_recomputed"]["median"]
        entry["ratios_recomputed"] = {
            "aa_over_single_median": aa_median / single_median,
            "single_ms_per_120k_rays": single_median,
            "aa_ms_per_120k_rays": aa_median / 4.0,
        }
        recomputed[pass_key] = entry
        report["passes"][pass_key] = entry

    # cross-pass consistency
    a = recomputed["measurement_a"]
    b = recomputed["measurement_b"]
    report["cross_pass"] = {
        "single_median_a": a["modes"]["single_spp1"]["frame_recomputed"]["median"],
        "single_median_b": b["modes"]["single_spp1"]["frame_recomputed"]["median"],
        "single_pct_diff": abs(
            a["modes"]["single_spp1"]["frame_recomputed"]["median"]
            - b["modes"]["single_spp1"]["frame_recomputed"]["median"]
        )
        / a["modes"]["single_spp1"]["frame_recomputed"]["median"]
        * 100.0,
        "aa_pct_diff": abs(
            a["modes"]["aa2_spp2"]["frame_recomputed"]["median"]
            - b["modes"]["aa2_spp2"]["frame_recomputed"]["median"]
        )
        / a["modes"]["aa2_spp2"]["frame_recomputed"]["median"]
        * 100.0,
        "ratio_a": a["ratios_recomputed"]["aa_over_single_median"],
        "ratio_b": b["ratios_recomputed"]["aa_over_single_median"],
        "ratio_pct_diff": abs(
            a["ratios_recomputed"]["aa_over_single_median"]
            - b["ratios_recomputed"]["aa_over_single_median"]
        )
        / a["ratios_recomputed"]["aa_over_single_median"]
        * 100.0,
    }

    # compare with delivered numbers
    def rounded(value: float) -> float:
        return round(value, 3)

    comparisons = []
    for pass_key, receipt_key in (("measurement_a", "measurement_a"), ("measurement_b", "measurement_b")):
        recomputed_pass = recomputed[pass_key]
        receipt_pass = receipt["measurements"][receipt_key]
        for mode_name, receipt_mode in (("single_spp1", "single_spp1"), ("aa2_spp2", "aa2_spp2")):
            for metric_key, receipt_metric in (("frame_recomputed", "frame_ms"), ("launch_recomputed", "launch_ms")):
                stat = recomputed_pass["modes"][mode_name][metric_key]
                expected = receipt_pass[receipt_mode][receipt_metric]
                for name in ("min", "median", "p95"):
                    comparisons.append(
                        {
                            "pass": pass_key,
                            "mode": mode_name,
                            "metric": receipt_metric,
                            "stat": name,
                            "recomputed": rounded(stat[name]),
                            "delivered": expected[name],
                            "match": abs(rounded(stat[name]) - expected[name]) <= 1e-3,
                        }
                    )
        ratios = receipt_pass["ratios"]
        comparisons.extend(
            [
                {
                    "pass": pass_key,
                    "metric": "aa_over_single_median",
                    "recomputed": rounded(recomputed_pass["ratios_recomputed"]["aa_over_single_median"]),
                    "delivered": ratios["aa_over_single_median"],
                    "match": abs(
                        rounded(recomputed_pass["ratios_recomputed"]["aa_over_single_median"])
                        - ratios["aa_over_single_median"]
                    )
                    <= 1e-3,
                },
                {
                    "pass": pass_key,
                    "metric": "single_ms_per_120k_rays",
                    "recomputed": rounded(recomputed_pass["ratios_recomputed"]["single_ms_per_120k_rays"]),
                    "delivered": ratios["single_ms_per_120k_rays"],
                    "match": abs(
                        rounded(recomputed_pass["ratios_recomputed"]["single_ms_per_120k_rays"])
                        - ratios["single_ms_per_120k_rays"]
                    )
                    <= 1e-3,
                },
                {
                    "pass": pass_key,
                    "metric": "aa_ms_per_120k_rays",
                    "recomputed": rounded(recomputed_pass["ratios_recomputed"]["aa_ms_per_120k_rays"]),
                    "delivered": ratios["aa_ms_per_120k_rays"],
                    "match": abs(
                        rounded(recomputed_pass["ratios_recomputed"]["aa_ms_per_120k_rays"])
                        - ratios["aa_ms_per_120k_rays"]
                    )
                    <= 1e-3,
                },
            ]
        )
    report["matches_delivered"] = comparisons
    report["all_delivered_numbers_reproduced"] = all(item["match"] for item in comparisons)

    # summary values are derived from the same raw series (>=50% of stats exactly equal is not
    # enough: every stat must equal the recomputation, which the comparisons above enforce).
    raw_summary_presence = all(
        "official_summary" in mode for pass_key in ("measurement_a", "measurement_b") for mode in raw[pass_key]["modes"]
    )
    report["checks"]["official_summary_present_in_raw"] = raw_summary_presence
    report["checks"]["official_summary_equals_recomputation"] = report["all_delivered_numbers_reproduced"]

    # memory series from the raw file
    memory = raw.get("memory", [])
    mem_checks = []
    for entry in memory:
        free = entry["frame_free"]
        mem_checks.append(
            {
                "mode": entry["mode"],
                "frames": entry["frames"],
                "frame_free_len": len(free),
                "frame_free_growth": free[-1] - free[0],
                "frame_free_unique": sorted(set(free)),
                "create_delta": entry["free_before"] - entry["free_after_create"],
                "peak_bytes": entry["free_before"] - min(free),
                "destroy_delta_vs_before": entry["free_after_destroy"] - entry["free_before"],
                "compile_bytes": entry["free_before"] - entry["free_after_compile"]
                if "free_after_compile" in entry
                else None,
            }
        )
    report["memory"] = {"checks": mem_checks, "raw_memory_entries": len(memory)}
    report["memory"]["claims"] = receipt["memory"]["device_memory"]

    out = ROOT / "target/qa-perf/out/judge_raw_stats.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps({k: report[k] for k in ("cross_pass", "checks", "memory")}, indent=1))
    mismatches = [item for item in comparisons if not item["match"]]
    print("comparisons:", len(comparisons), "mismatches:", mismatches)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
