# -*- coding: utf-8 -*-
"""G10@P5 · 深穿透 EPA witness 采样定性分析（纯 CPU，只用已有 QA 数据，不跑 GPU）。

数据源（均只读）：
  - 上游参考：docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-witness-gpu.json
    → sweep 中 case=cylinder-box 的 24 world（QA 姿态生成：rng seed 20251025，
      base qpos + 随机轴角（angle∈[-1.4,1.4]）+ free joint 位置 uniform(-0.08,0.08)）。
  - 设备输出：同目录 qa-gpu/out/cylinder-box-sweep/{dist,x1,x2,ncontact,index,overflow}[-run2].bin
    （QA 探针的未跟踪产物，本脚本只读、不复制 .bin；只把派生数字写入本片证据）。
判据同 G5 现行口径（qa_compare_gpu.py）：dist 按自身量级 ≤4 ULP；x1/x2 用 scale=max_i|ref_i|
的 4·ULP(scale)；ncontact/index/overflow 位级。
输出：analysis.json（派生数字）+ analysis.log（人读表）。
用法：python -B analyze_deep_epa.py
"""

import json
import hashlib
import math
import os
import struct
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
QA = HERE.parent / "g10-p5-gpu-kernels-independent-check"
WITNESS = QA / "qa-witness-gpu.json"
DEV = QA / "qa-gpu" / "out" / "cylinder-box-sweep"
CASE = "cylinder-box"
TASK = "2026-10-25-g10-p5-deep-epa-witness-sampling"
UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"


def read_bin(path, dtype, count):
    raw = Path(path).read_bytes()
    fmt = "<%df" % (len(raw) // 4) if dtype == "f32" else "<%di" % (len(raw) // 4)
    values = list(struct.unpack(fmt, raw))
    return values[:count]


def f32_bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def ulp_at(value):
    value = abs(float(value))
    if value == 0.0:
        return 2.0 ** -149
    a = np.float32(value)
    b = np.nextafter(a, np.float32(np.inf), dtype=np.float32)
    return float(b) - float(a)


def criterion_vector(dev, ref):
    scale = max(abs(v) for v in ref)
    if scale == 0.0:
        scale = 1.0
    bound = 4.0 * ulp_at(scale)
    worst = max(abs(d - r) for d, r in zip(dev, ref))
    return scale, bound, worst, worst <= bound


def norm3(v):
    return math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])


def dot3(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def sub3(a, b):
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]


def scale3(a, s):
    return [a[0] * s, a[1] * s, a[2] * s]


def main():
    witness = json.loads(WITNESS.read_text(encoding="utf-8"))
    sweep = next(item for item in witness["sweep"] if item["case"] == CASE)
    ref = sweep["results"]
    worlds = int(sweep["worlds"])

    dev = {
        "dist": read_bin(DEV / "dist.bin", "f32", worlds),
        "x1": read_bin(DEV / "x1.bin", "f32", worlds * 3),
        "x2": read_bin(DEV / "x2.bin", "f32", worlds * 3),
        "ncontact": read_bin(DEV / "ncontact.bin", "i32", worlds),
        "index": read_bin(DEV / "index.bin", "i32", worlds),
        "overflow": read_bin(DEV / "overflow.bin", "i32", worlds),
    }
    dev2 = {
        "dist": read_bin(DEV / "dist-run2.bin", "f32", worlds),
        "x1": read_bin(DEV / "x1-run2.bin", "f32", worlds * 3),
        "x2": read_bin(DEV / "x2-run2.bin", "f32", worlds * 3),
        "ncontact": read_bin(DEV / "ncontact-run2.bin", "i32", worlds),
        "index": read_bin(DEV / "index-run2.bin", "i32", worlds),
        "overflow": read_bin(DEV / "overflow-run2.bin", "i32", worlds),
    }
    two_runs_identical = all(dev[k] == dev2[k] for k in dev)
    bin_sha256 = {
        name: hashlib.sha256((DEV / name).read_bytes()).hexdigest()
        for name in sorted(os.listdir(DEV)) if name.endswith(".bin")
    }
    qa_summary = json.loads((QA / "qa-compare-gpu.json").read_text(encoding="utf-8"))
    qa_entry = qa_summary["witness_cases"][CASE + "-sweep"]

    rows = []
    for world in range(worlds):
        ref_dist = float(ref["dist"][world])
        ref_x1 = [float(v) for v in ref["x1"][world]]
        ref_x2 = [float(v) for v in ref["x2"][world]]
        d_dist = float(dev["dist"][world])
        d_x1 = [float(v) for v in dev["x1"][world * 3:(world + 1) * 3]]
        d_x2 = [float(v) for v in dev["x2"][world * 3:(world + 1) * 3]]

        dist_abs = abs(d_dist - ref_dist)
        dist_ulp = dist_abs / (ulp_at(ref_dist) if ref_dist != 0.0 else ulp_at(1.0))
        scale1, bound1, worst1, ok1 = criterion_vector(d_x1, ref_x1)
        scale2, bound2, worst2, ok2 = criterion_vector(d_x2, ref_x2)
        worst_axis = max(range(3), key=lambda axis: abs(d_x1[axis] - ref_x1[axis]))

        delta1 = sub3(d_x1, ref_x1)
        delta2 = sub3(d_x2, ref_x2)
        normal = None
        normal_component = None
        tangential_component = None
        delta1_vs_delta2 = None
        device_normal = None
        normal_angle_deg = None
        normal_delta = None
        witness_identity_residual = None
        if abs(ref_dist) > 1.0e-12:
            # 上游 witness：x2 - x1 = dist * n（dist<0 时为穿透），n 为单位法线。
            raw = scale3(sub3(ref_x2, ref_x1), 1.0 / ref_dist)
            length = norm3(raw)
            if length > 0.5:
                normal = scale3(raw, 1.0 / length)
                normal_component = abs(dot3(delta1, normal))
                tangential = sub3(delta1, scale3(normal, dot3(delta1, normal)))
                tangential_component = norm3(tangential)
                delta1_vs_delta2 = norm3(sub3(delta1, delta2))
        if abs(d_dist) > 1.0e-12:
            # 设备自算法线（用同一 witness 关系反解）与上游法线对比；
            # 以及自检 |(x2-x1) - dist*n|（witness 自洽残差，按 |dist| 归一）。
            raw_dev = scale3(sub3(d_x2, d_x1), 1.0 / d_dist)
            length_dev = norm3(raw_dev)
            if length_dev > 0.5:
                device_normal = scale3(raw_dev, 1.0 / length_dev)
                residual = norm3(sub3(sub3(d_x2, d_x1), scale3(device_normal, d_dist)))
                witness_identity_residual = residual / abs(d_dist)
                if normal is not None:
                    cosine = max(-1.0, min(1.0, dot3(normal, device_normal)))
                    normal_angle_deg = math.degrees(math.acos(cosine))
                    normal_delta = sub3(device_normal, normal)
        predicted_shift = None
        predicted_shift_ratio = None
        if normal_delta is not None and delta1_vs_delta2 is not None:
            # 若两实现只差一个绕切向的微小法线转动 δn，则 witness 点对的差应满足
            # |Δx1 - Δx2| ≈ |dist| · |δn|（x2 近似不动、x1 = x2 - dist·n）。
            predicted_shift = abs(ref_dist) * norm3(normal_delta)
            predicted_shift_ratio = delta1_vs_delta2 / predicted_shift if predicted_shift else None

        rows.append({
            "world": world,
            "ref_dist": ref_dist,
            "penetration_depth": -ref_dist,
            "dist": {
                "dev": d_dist,
                "abs": dist_abs,
                "ulp": dist_ulp,
                "delta": d_dist - ref_dist,
                "bit_equal": f32_bits(d_dist) == f32_bits(ref_dist),
                "ok": dist_abs <= (4.0 * ulp_at(ref_dist) if ref_dist != 0.0 else 4.0 * ulp_at(1.0)),
            },
            "x1": {
                "ref": ref_x1, "dev": d_x1, "delta": delta1,
                "scale": scale1, "bound": bound1, "worst_abs": worst1,
                "worst_axis": worst_axis, "ok": ok1,
                "worst_ulps_at_scale": worst1 / ulp_at(scale1),
                "worst_over_bound": worst1 / bound1 if bound1 else float("inf"),
                "normal_component": normal_component,
                "tangential_component": tangential_component,
                "delta_vs_delta2_gap": delta1_vs_delta2,
            },
            "x2": {
                "ref": ref_x2, "dev": d_x2, "delta": delta2,
                "scale": scale2, "bound": bound2, "worst_abs": worst2,
                "ok": ok2,
                "worst_ulps_at_scale": worst2 / ulp_at(scale2),
                "worst_over_bound": worst2 / bound2 if bound2 else float("inf"),
            },
            "normal_ref": normal,
            "normal_device": device_normal,
            "normal_delta": normal_delta,
            "normal_angle_deg": normal_angle_deg,
            "normal_delta_norm": norm3(normal_delta) if normal_delta is not None else None,
            "predicted_witness_shift": predicted_shift,
            "predicted_shift_ratio": predicted_shift_ratio,
            "witness_identity_residual_relative": witness_identity_residual,
            "ncontact": {"device": int(dev["ncontact"][world]),
                         "upstream": int(ref["ncontact"][world]),
                         "equal": int(dev["ncontact"][world]) == int(ref["ncontact"][world])},
            "index": {"device": int(dev["index"][world]),
                      "upstream": int(ref["index"][world]),
                      "equal": int(dev["index"][world]) == int(ref["index"][world])},
            "overflow": {"device": int(dev["overflow"][world]),
                         "upstream": int(ref["overflow"][world]),
                         "equal": int(dev["overflow"][world]) == int(ref["overflow"][world])},
        })

    # 干净法线（单位长度校验）与边界外 world 的汇总
    over_x1 = [row for row in rows if not row["x1"]["ok"]]
    over_x2 = [row for row in rows if not row["x2"]["ok"]]
    over = sorted({row["world"] for row in over_x1 + over_x2})
    depths = sorted(row["penetration_depth"] for row in rows)

    def mean(values):
        return float(sum(values) / len(values)) if values else None

    def pearson(xs, ys):
        if len(xs) < 3:
            return None
        mx, my = mean(xs), mean(ys)
        num = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
        dx = math.sqrt(sum((x - mx) ** 2 for x in xs))
        dy = math.sqrt(sum((y - my) ** 2 for y in ys))
        return num / (dx * dy) if dx and dy else None

    depths_all = [row["penetration_depth"] for row in rows]
    ratios_x1 = [row["x1"]["worst_over_bound"] for row in rows]
    ratios_x2 = [row["x2"]["worst_over_bound"] for row in rows]
    normal_comp = [row["x1"]["normal_component"] for row in rows]
    tangent_comp = [row["x1"]["tangential_component"] for row in rows]

    summary = {
        "worlds": worlds,
        "two_runs_bitwise_identical": two_runs_identical,
        "index_equal_worlds": sum(1 for row in rows if row["index"]["equal"]),
        "ncontact_equal_worlds": sum(1 for row in rows if row["ncontact"]["equal"]),
        "overflow_nonzero_worlds": sum(1 for row in rows if row["overflow"]["device"] != 0),
        "dist_bit_equal_worlds": sum(1 for row in rows if row["dist"]["bit_equal"]),
        "dist_max_ulp": max(row["dist"]["ulp"] for row in rows),
        "dist_over_bound_worlds": sum(1 for row in rows if not row["dist"]["ok"]),
        "x1_over_bound_worlds": [row["world"] for row in over_x1],
        "x2_over_bound_worlds": [row["world"] for row in over_x2],
        "over_bound_union_worlds": over,
        "x1_worst": max(rows, key=lambda row: row["x1"]["worst_abs"])["x1"],
        "x2_worst": max(rows, key=lambda row: row["x2"]["worst_abs"])["x2"],
        "penetration_depth_min": min(depths),
        "penetration_depth_max": max(depths),
        "penetration_depth_median": depths[len(depths) // 2],
        "over_bound_depths": {row["world"]: row["penetration_depth"] for row in rows if row["world"] in over},
        "normal_angle_max_deg": max(
            (row["normal_angle_deg"] for row in rows if row["normal_angle_deg"] is not None),
            default=None,
        ),
        "normal_angle_world_max": max(
            (row for row in rows if row["normal_angle_deg"] is not None),
            key=lambda row: row["normal_angle_deg"],
            default=None,
        )["world"] if any(row["normal_angle_deg"] is not None for row in rows) else None,
        "witness_identity_residual_max_relative": max(
            (row["witness_identity_residual_relative"] for row in rows
             if row["witness_identity_residual_relative"] is not None),
            default=None,
        ),
        "over_bound_details": [
            {
                "world": row["world"],
                "depth": row["penetration_depth"],
                "dist_delta": row["dist"]["delta"],
                "dist_ulp": row["dist"]["ulp"],
                "x1_ok": row["x1"]["ok"], "x1_worst_abs": row["x1"]["worst_abs"],
                "x1_bound": row["x1"]["bound"], "x1_over_bound": row["x1"]["worst_over_bound"],
                "x1_worst_axis": row["x1"]["worst_axis"],
                "x1_worst_ulps_at_scale": row["x1"]["worst_ulps_at_scale"],
                "x1_delta": row["x1"]["delta"],
                "x1_normal_component": row["x1"]["normal_component"],
                "x1_tangential_component": row["x1"]["tangential_component"],
                "x2_ok": row["x2"]["ok"], "x2_worst_abs": row["x2"]["worst_abs"],
                "x2_bound": row["x2"]["bound"], "x2_over_bound": row["x2"]["worst_over_bound"],
                "x2_delta": row["x2"]["delta"],
                "delta1_vs_delta2_gap": row["x1"]["delta_vs_delta2_gap"],
                "normal_angle_deg": row["normal_angle_deg"],
                "normal_delta": row["normal_delta"],
                "predicted_witness_shift": row["predicted_witness_shift"],
                "predicted_shift_ratio": row["predicted_shift_ratio"],
                "witness_identity_residual_relative": row["witness_identity_residual_relative"],
                "index_equal": row["index"]["equal"],
                "ncontact_equal": row["ncontact"]["equal"],
            }
            for row in rows if row["world"] in over
        ],
        "correlation_pearson": {
            "depth_vs_x1_over_bound": pearson(depths_all, ratios_x1),
            "depth_vs_x2_over_bound": pearson(depths_all, ratios_x2),
            "depth_vs_x1_normal_component": pearson(depths_all, normal_comp),
            "depth_vs_x1_tangential_component": pearson(depths_all, tangent_comp),
        },
    }

    cross_check = {
        "qa_summary_source": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json（witness_cases.cylinder-box-sweep）",
        "x1_bad_matches": qa_entry["x1"]["bad"] == len(summary["x1_over_bound_worlds"]),
        "x2_bad_matches": qa_entry["x2"]["bad"] == len(summary["x2_over_bound_worlds"]),
        "dist_bit_equal_matches": qa_entry["dist"]["bit_equal"] == summary["dist_bit_equal_worlds"],
        "dist_max_ulp_matches": abs(qa_entry["dist"]["max_ulp"] - summary["dist_max_ulp"]) < 1.0e-9,
        "dist_over_bound_matches": qa_entry["dist"]["bad"] == summary["dist_over_bound_worlds"],
        "ncontact_equal_matches": qa_entry["ncontact_equal"] == (summary["ncontact_equal_worlds"] == worlds),
        "index_pairs_equal_matches": qa_entry["index_device"] == qa_entry["index_upstream"],
        "overflow_zero_matches": summary["overflow_nonzero_worlds"] == 0,
        "two_runs_identical_matches": qa_entry["two_runs_bitwise_identical"] == two_runs_identical,
        "qa_numbers": {
            "x1_bad": qa_entry["x1"]["bad"], "x2_bad": qa_entry["x2"]["bad"],
            "x1_max_abs": qa_entry["x1"]["max_abs"], "x2_max_abs": qa_entry["x2"]["max_abs"],
            "dist_bit_equal": qa_entry["dist"]["bit_equal"],
            "dist_max_ulp": qa_entry["dist"]["max_ulp"],
            "dist_bad": qa_entry["dist"]["bad"],
        },
    }

    payload = {
        "schema": 1,
        "task_id": TASK,
        "kind": "deep-epa-witness-sampling-analysis",
        "upstream": UPSTREAM,
        "source": {
            "upstream_raw": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-witness-gpu.json（sweep cylinder-box）",
            "device_raw": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/out/cylinder-box-sweep/{dist,x1,x2,ncontact,index,overflow}[-run2].bin（QA 未跟踪产物，只读，不复制）",
            "device_bin_sha256": bin_sha256,
            "pose_generation": "QA qa_prepare_capture.py：np.random.default_rng(20251025)，base qpos + 随机轴角 angle∈[-1.4,1.4] + free joint 位置 uniform(-0.08,0.08)×3，共 24 world",
            "criterion": "G5 现行口径：dist 自身量级 ≤4 ULP；x1/x2 scale=max_i|ref_i| 的 4·ULP(scale)；ncontact/index/overflow 位级",
        },
        "cross_check_with_qa_summary": cross_check,
        "summary": summary,
        "worlds_file": "analysis-worlds.jsonl（每行一个 world 的完整派生字段；第 9 行 = world 8）",
        "notes": [
            "纯 CPU 定性分析：不改产品代码、不开 GPU 窗口、不复制 QA 的 .bin。",
            "法线由上游 witness 关系 x2-x1 = dist·n 反解（仅在 |dist|>1e-12 且反解模长>0.5 时使用）。",
            "normal/tangential 分解只对 x1 的偏差向量做，用来区分「深度/法线方向差」与「接触面内滑动」。",
            "predicted_witness_shift = |dist|·|normal_delta|：若两实现只差一个微小法线转动，witness 点对差应约等于该值。",
            "完整 24 world 明细见 analysis-worlds.jsonl（避免单个证据 JSON 超过 500 行棘轮）。",
        ],
    }

    out_path = HERE / "analysis.json"
    with open(out_path, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(payload, handle, ensure_ascii=False, indent=2)
        handle.write("\n")

    worlds_path = HERE / "analysis-worlds.jsonl"
    with open(worlds_path, "w", encoding="utf-8", newline="\n") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")

    lines = []
    lines.append(f"task={TASK}")
    lines.append(f"case={CASE} worlds={worlds} two_runs_bitwise_identical={two_runs_identical}")
    lines.append(f"dist: bit_equal {summary['dist_bit_equal_worlds']}/{worlds} max_ulp {summary['dist_max_ulp']:.3f} over_bound {summary['dist_over_bound_worlds']}")
    lines.append(f"ncontact_equal_worlds={summary['ncontact_equal_worlds']}/{worlds} index_equal={summary['index_equal_worlds']}/{worlds} overflow_nonzero_worlds={summary['overflow_nonzero_worlds']}")
    lines.append(f"x1_over_bound_worlds={summary['x1_over_bound_worlds']} x2_over_bound_worlds={summary['x2_over_bound_worlds']}")
    lines.append("cross_check_all_match=" + str(all(value for key, value in cross_check.items()
                                                   if key.endswith("_matches"))))
    lines.append("normal_angle_max_deg=" + str(summary["normal_angle_max_deg"])
                 + " witness_resid_max=" + str(summary["witness_identity_residual_max_relative"]))
    lines.append(f"depth min/median/max = {summary['penetration_depth_min']:.6f} / {summary['penetration_depth_median']:.6f} / {summary['penetration_depth_max']:.6f}")
    lines.append("")
    lines.append("world depth x1_worst x1_bound x1_ratio x2_worst x2_bound x2_ratio nrm_comp tan_comp d1-d2_gap idx_eq normal_angle_deg witness_resid pred_ratio")
    for row in rows:
        x1, x2 = row["x1"], row["x2"]
        def f(value):
            return "null" if value is None else f"{value:.3e}"
        angle = row["normal_angle_deg"]
        angle_text = "null" if angle is None else f"{angle:.6f}"
        pred = row["predicted_shift_ratio"]
        pred_text = "null" if pred is None else f"{pred:.4f}"
        lines.append(
            f"{row['world']:>2} {row['penetration_depth']:.6f} {f(x1['worst_abs'])} {f(x1['bound'])} {x1['worst_over_bound']:.3f} "
            f"{f(x2['worst_abs'])} {f(x2['bound'])} {x2['worst_over_bound']:.3f} "
            f"{f(x1['normal_component'])} {f(x1['tangential_component'])} {f(x1['delta_vs_delta2_gap'])} {row['index']['equal']} "
            f"{angle_text} {f(row['witness_identity_residual_relative'])} {pred_text}")
    lines.append("")
    lines.append("over_bound_details:")
    for detail in summary["over_bound_details"]:
        lines.append("  " + json.dumps(detail, ensure_ascii=False))
    lines.append("")
    lines.append("correlation_pearson=" + json.dumps(summary["correlation_pearson"], ensure_ascii=False))

    log_path = HERE / "analysis.log"
    with open(log_path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"wrote {out_path}")
    print(f"wrote {log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
