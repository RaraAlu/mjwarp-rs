# -*- coding: utf-8 -*-
"""G6-2 性能证据聚合（离线 authoring；不占 GPU、不启动产品测试）。

输入（同目录）：
  device/<case>.json      perf_probe.exe --perf 5 50 产出的设备两栏原始样本
  host/<case>.json        cargo test --locked --lib perf_ -- --ignored 产出的 host 单线程样本
  upstream-cpu.json       冻结上游 f32 CPU 参考计时（离线 Python）
  window-log.txt          窗口基线/环境日志（可选，用于摘 CPU/驱动/时间）
输出：
  perf-summary.json       统一汇总（每 case 四列 + 覆盖度 + 环境 + 口径说明）
用法（仓库根目录）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/g10-p5-acceptance-map/perf/aggregate_perf.py
"""

from __future__ import annotations

import json
import os
import statistics

HERE = os.path.dirname(os.path.abspath(__file__))
TASK_ID = "2026-10-25-g10-p5-acceptance-perf-native"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"


def load_json(path: str):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def percentile(samples, fraction: float) -> float:
    values = sorted(float(value) for value in samples)
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    index = max(0, min(len(values) - 1, int(round(fraction * (len(values) - 1)))))
    return values[index]


def summarize(samples, ncontacts: int):
    values = [float(value) for value in samples]
    median = statistics.median(values) if values else 0.0
    return {
        "samples": len(values),
        "median_ns": median,
        "p95_ns": percentile(values, 0.95),
        "min_ns": min(values) if values else 0.0,
        "max_ns": max(values) if values else 0.0,
        "ncontacts": ncontacts,
        "contacts_per_s": (ncontacts * 1.0e9 / median) if median > 0 else None,
    }


def load_series(directory: str):
    series = {}
    if not os.path.isdir(directory):
        return series
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".json"):
            continue
        payload = load_json(os.path.join(directory, name))
        case = payload.get("case") or os.path.splitext(name)[0]
        series[case] = payload
    return series


def read_window_context() -> dict:
    path = os.path.join(HERE, "window-log.txt")
    context = {}
    if not os.path.isfile(path):
        return context
    first_token_keys = {
        "baseline_used_mib=", "baseline_compute_apps_count=", "task_related_count=",
        "desktop_process_count=", "window_worktree_changes_count=",
        "peak_memory_used_mib=", "peak_delta_mib=", "sample_count=",
        "util_max_percent=", "util_avg_percent=", "temp_max_c=",
        "elapsed_seconds=", "after_used_mib=", "after_compute_apps_count=",
    }
    with open(path, "r", encoding="utf-8", errors="replace") as handle:
        for line in handle:
            for key in ("window_start_head=", "head_revision=", "baseline_memory=",
                        "baseline_used_mib=", "baseline_compute_apps_count=",
                        "task_related_count=", "cpu=", "desktop_process_top5=",
                        "desktop_process_count=", "product_diff_6510317f_to_head=",
                        "window_worktree_changes_count=", "peak_memory_used_mib=",
                        "peak_delta_mib=", "sample_count=", "util_max_percent=",
                        "util_avg_percent=", "temp_max_c=", "elapsed_seconds=",
                        "after_used_mib=", "after_compute_apps_count="):
                marker = " " + key
                if marker in line:
                    raw = line.split(marker, 1)[1].strip()
                    context[key.rstrip("=")] = raw.split()[0] if key in first_token_keys else raw
    return context


def main() -> int:
    device = load_series(os.path.join(HERE, "device"))
    host = load_series(os.path.join(HERE, "host"))
    upstream_path = os.path.join(HERE, "upstream-cpu.json")
    upstream = load_json(upstream_path) if os.path.isfile(upstream_path) else {}

    upstream_cases = {row.get("case") for row in upstream.get("cases", [])} - {None}
    cases = sorted(set(device) | set(host) | upstream_cases)
    rows = []
    for case in cases:
        row = {"case": case}
        if case in host:
            payload = host[case]
            row["host_cpu_single_thread"] = summarize(payload.get("samples_ns", []),
                                                      int(payload.get("ncontacts", 0)))
        if case in device:
            payload = device[case]
            nacon = int(payload.get("nacon", 0))
            kernel = summarize(payload.get("kernel_ns", []), nacon)
            e2e = summarize(payload.get("e2e_ns", []), nacon)
            row["device_kernel_only"] = kernel
            row["device_end_to_end"] = e2e
            row["device_case_meta"] = {
                "kernel_kind": payload.get("kernel_kind"),
                "nworld": payload.get("nworld"),
                "naconmax": payload.get("naconmax"),
                "efc_width": payload.get("efc_width"),
                "nacon": payload.get("nacon"),
                "nccd": payload.get("nccd"),
                "overflow": payload.get("overflow"),
            }
        upstream_rows = {item.get("case"): item for item in upstream.get("cases", [])}
        if case in upstream_rows:
            payload = upstream_rows[case]
            row["upstream_f32_cpu"] = summarize(payload.get("samples_ns", []),
                                                int(payload.get("ncontacts", 0)))
        rows.append(row)

    coverage = {
        "host_cpu": sorted(set(cases) & set(host)),
        "device": sorted(set(cases) & set(device)),
        "upstream_f32_cpu": sorted(upstream_cases),
        "missing": {
            "host_cpu": sorted(set(cases) - set(host)),
            "device": sorted(set(cases) - set(device)),
            "upstream_f32_cpu": sorted(set(cases) - upstream_cases),
        },
    }

    summary = {
        "schema": 1,
        "task_id": TASK_ID,
        "kind": "g6-2-perf-summary",
        "upstream": upstream.get("upstream", UPSTREAM),
        "columns": {
            "host_cpu_single_thread": "Rust host 产品路径，纯 CPU 单线程，warmup 20 + 200 次，中位/p95/contacts/s",
            "device_kernel_only": "设备内核单次发射，CUDA events（warmup 5 + 50），清零在 start 事件前",
            "device_end_to_end": "设备一次完整调用：geom 位姿 H2D + 输出清零 + kernel + 全部输出 D2H + 同步（host steady_clock，warmup 5 + 50）",
            "upstream_f32_cpu": "冻结上游 f32 CPU 同机同版本参考（离线 Python，warmup 20 + 200，中位/p95/contacts/s）",
        },
        "environment": {
            "device": {
                key: device[next(iter(device))].get(key)
                for key in ("device", "compute_capability", "cudart_version")
            } if device else {},
            "upstream": upstream.get("environment", {}),
            "window": read_window_context(),
        },
        "notes": [
            "性能数据不进精度判据、不跨硬件外推；编译/JIT 时间不计入上述任何列。",
            "device 列只覆盖冻结 G5 case 目录存在的 7 个 case；host 列覆盖 G2 五 case + G3 sdf-box + G4 hfield 三例。",
            "upstream_f32_cpu 为同机同版本上游 CPU 参考，不是上游 GPU 计时（上游 GPU 计时未纳入本任务口径）。",
            "contacts/s = 每次调用接触点数 × 1e9 / 中位耗时(ns)；只作量级参考，不作门槛。",
            "G6-2 的独占窗口口径见 window-log.txt：跑前跑后 query-compute-apps、0.5 s 显存采样、守卫 3072 MiB/50%。",
        ],
        "coverage": coverage,
        "cases": rows,
    }

    out_path = os.path.join(HERE, "perf-summary.json")
    with open(out_path, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(summary, handle, ensure_ascii=False, indent=2)
        handle.write("\n")

    print(f"wrote {out_path}")
    print(f"cases={len(rows)} host={len(coverage['host_cpu'])} device={len(coverage['device'])} "
          f"upstream={len(coverage['upstream_f32_cpu'])}")
    for row in rows:
        device_row = row.get("device_kernel_only") or {}
        print(f"  {row['case']:<16} kernel_median={device_row.get('median_ns')} "
              f"host_median={(row.get('host_cpu_single_thread') or {}).get('median_ns')} "
              f"upstream_median={(row.get('upstream_f32_cpu') or {}).get('median_ns')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
