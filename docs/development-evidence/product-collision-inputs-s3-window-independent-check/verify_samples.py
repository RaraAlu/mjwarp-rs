"""QA independent recomputation of the S3 window attempt1 sampler facts."""
import csv
import json
import statistics
from datetime import datetime
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
RAW = ROOT / "target/s3-product-wiring/attempt1"
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
OUT = ROOT / "target/qa-s3win/verify_samples.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def parse_ts(text: str) -> datetime:
    main, _, offset = text.partition("+")
    head, _, frac = main.partition(".")
    return datetime.fromisoformat(f"{head}.{(frac + '000000')[:6]}+{offset}")


def main():
    rows = list(csv.DictReader((RAW / "samples.csv").read_text(encoding="utf-8-sig").splitlines()))
    used = [int(row["used_mib"]) for row in rows]
    free = [int(row["free_mib"]) for row in rows]
    util = [int(row["util_percent"]) for row in rows]
    seqs = [int(row["seq"]) for row in rows]
    stamps = [parse_ts(row["timestamp"]) for row in rows]

    guard_lines = (RAW / "guard.log").read_text(encoding="utf-8", errors="replace").splitlines()
    guard_base = None
    guard_init = None
    for line in guard_lines:
        if line.startswith("guard_baseline"):
            guard_base = dict(item.split("=", 1) for item in line.split() if "=" in item)
        if line.startswith("guard_init"):
            guard_init = dict(item.split("=", 1) for item in line.split() if "=" in item)

    baseline_used = int(guard_base["used_mib"])
    peak = max(used)
    peak_index = used.index(peak)
    sample_min = min(used)
    free_min = min(free)
    util_max = max(util)
    streak = best = 0
    for value in util:
        streak = streak + 1 if value >= 80 else 0
        best = max(best, streak)
    intervals = [(stamps[i + 1] - stamps[i]).total_seconds() for i in range(len(stamps) - 1)]
    max_excluding_peak = max(used[:peak_index] + used[peak_index + 1:])

    baseline_rows = []
    for line in (RAW / "baseline.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split(",")]
        if len(parts) == 5 and parts[0].isdigit():
            baseline_rows.append(tuple(int(x) for x in parts[:4]))
    baseline_free_min = min(row[1] for row in baseline_rows)
    floor = min(10240, baseline_free_min - 3072)
    open_free = baseline_rows[0][1]

    receipt = json.loads((EV / "window-receipt.json").read_text(encoding="utf-8"))
    observed = {
        "seq": seqs[peak_index],
        "timestamp": rows[peak_index]["timestamp"],
        "used_mib": peak,
        "prev_sample": used[peak_index - 1],
        "next_sample": used[peak_index + 1],
        "next_next_sample": used[peak_index + 2],
    }
    receipt_neighbours = receipt["sampler"]["observation_OW1"]["neighbor_samples_used_mib"]
    receipt_budget_neighbours = receipt["budget_deviation"]["peak"]["neighbors_used_mib"]

    check(len(rows) == 867, f"samples {len(rows)}")
    check(baseline_used == 3091, f"guard baseline {baseline_used}")
    check(peak == 4142, f"used_peak {peak}")
    check(peak - baseline_used == 1051, "delta vs guard baseline")
    check(peak - used[0] == 1051, "delta vs first sample")
    check(peak - sample_min == 1055, "delta vs sample min")
    check(sample_min == 3087, f"used_min {sample_min}")
    check(free_min == 11920, f"free_min {free_min}")
    check(util_max == 31, f"util_max {util_max}")
    check(best == 0, f"util>=80 streak {best}")
    check(seqs[peak_index] == 776, f"spike seq {seqs[peak_index]}")
    check(rows[peak_index]["timestamp"] == "2026-10-27T17:03:36.7225390+08:00", "spike timestamp")
    check(sum(1 for value in used if value > 3120) == 1, "samples_gt_3120")
    check(max_excluding_peak == 3109 and max_excluding_peak - baseline_used == 18,
          f"sustained delta {max_excluding_peak - baseline_used}")
    check(int(statistics.median(used)) == 3091, "median used")
    check(abs(statistics.median(intervals) - 0.25) < 0.05,
          f"sampling cadence near nominal 0.25 s (median {statistics.median(intervals):.4f})")
    check(floor == 9904, f"floor {floor}")
    check(open_free >= 10240, "open free gate")
    check(int(guard_init["free_floor_mib"]) == floor, "guard_init floor matches formula")
    check(any("guard_aggregates samples=867 baseline_used=3091 used_peak=4142 delta_peak=1051 limit=3072 free_min=11920 util_max=31 util_streak=0" in line for line in guard_lines),
          "guard_aggregates line")
    end_line = next(line for line in guard_lines if line.startswith("guard_end"))
    check("residual_context_pids= " in end_line, "guard_end residual empty")
    check(observed["prev_sample"] == 3092 and observed["next_sample"] == 3092,
          f"observed immediate neighbours {observed}")
    check(receipt_neighbours == [3092, 3091],
          f"receipt observation_OW1 neighbours {receipt_neighbours} (observed prev/next {observed['prev_sample']}/{observed['next_sample']})")
    check(receipt_budget_neighbours == [3092, 4142, 3091],
          f"receipt budget peak neighbours {receipt_budget_neighbours}")

    result = {
        "samples": len(rows),
        "baseline_used_mib": baseline_used,
        "used_peak_mib": peak,
        "used_min_mib": sample_min,
        "delta_vs_guard_baseline_mib": peak - baseline_used,
        "delta_vs_first_sample_mib": peak - used[0],
        "delta_vs_sample_min_mib": peak - sample_min,
        "free_min_mib": free_min,
        "util_max_percent": util_max,
        "util_ge_80_streak_max": best,
        "median_used_mib": int(statistics.median(used)),
        "p99_sorted_index_mib": sorted(used)[int(len(used) * 0.99)],
        "sampling_interval": {"median_s": round(statistics.median(intervals), 4),
                              "min_s": round(min(intervals), 4),
                              "max_s": round(max(intervals), 4)},
        "spike": observed,
        "peak_excluded_max_mib": max_excluding_peak,
        "sustained_delta_mib": max_excluding_peak - baseline_used,
        "receipt_neighbours_OW1": receipt_neighbours,
        "receipt_neighbours_budget": receipt_budget_neighbours,
        "prewindow_baseline_points": baseline_rows,
        "floor_mib": floor,
        "open_free_mib": open_free,
        "checks": checks,
        "problems": problems,
        "result": "PASS" if not problems else "FAIL",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={result['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print(json.dumps({k: result[k] for k in ["samples", "baseline_used_mib", "used_peak_mib",
                                             "delta_vs_guard_baseline_mib", "delta_vs_first_sample_mib",
                                             "delta_vs_sample_min_mib", "free_min_mib", "util_max_percent",
                                             "util_ge_80_streak_max", "floor_mib"]}, ensure_ascii=False))
    print("interval:", result["sampling_interval"])
    print("spike:", json.dumps(observed, ensure_ascii=False), "receipt:", receipt_neighbours, receipt_budget_neighbours)


if __name__ == "__main__":
    main()
