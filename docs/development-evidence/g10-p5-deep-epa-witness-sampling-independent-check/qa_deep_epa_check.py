"""深穿透 EPA witness 采样片独立复核（纯 CPU）。

只读：QA 的原始样本（`qa-witness-gpu.json` 的 sweep + QA 探针未跟踪输出 bin），
独立重算 24 world 的统计、交叉核对交付 `analysis.json` 的 9 项、复算 world 8 的
「法线差 × 深度」归因等式，并检视结论边界/范围/未覆盖披露。

用法：python qa_deep_epa_check.py
"""

import hashlib
import json
import math
import re
import subprocess
import sys
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
QA5 = REPO / "docs" / "development-evidence" / "g10-p5-gpu-kernels-independent-check"
DELIVERY = REPO / "docs" / "development-evidence" / "g10-p5-deep-epa-witness-sampling"
OUT = QA5 / "qa-gpu" / "out" / "cylinder-box-sweep"

FIX = "c44578be"


def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))


def git(*args, check=False):
    p = subprocess.run(["git", "-c", "core.longpaths=true", *args], cwd=REPO, capture_output=True)
    if check and p.returncode != 0:
        raise RuntimeError(p.stderr.decode("utf-8", "replace"))
    return p


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def f32(name):
    return np.fromfile(OUT / name, dtype="<f4")


def i32(name):
    return np.fromfile(OUT / name, dtype="<i4")


def spacing(x):
    return float(np.spacing(np.float32(x)))


def ulp_diff(a, b):
    """f32 绝对值差 / 参考值处的 f32 ULP。"""
    a = np.float32(a)
    b = np.float32(b)
    ref_ulp = abs(float(np.spacing(np.float32(abs(float(b))))))
    if ref_ulp == 0.0:
        ref_ulp = float(np.spacing(np.float32(1.0)))
    return abs(float(a) - float(b)) / ref_ulp


def main():
    rep = {"schema": 1, "kind": "qa-deep-epa-witness-check",
           "task_id": "2026-10-25-g10-p5-deep-epa-witness-sampling-independent-check"}

    up = load(QA5 / "qa-witness-gpu.json")
    entry = [e for e in up["sweep"] if e["case"] == "cylinder-box"][0]
    ref = entry["results"]
    n = len(ref["dist"])

    def load_dev(suffix):
        return {
            "dist": f32(f"dist{suffix}.bin"),
            "x1": f32(f"x1{suffix}.bin").reshape(-1, 3),
            "x2": f32(f"x2{suffix}.bin").reshape(-1, 3),
            "ncontact": i32(f"ncontact{suffix}.bin"),
            "index": i32(f"index{suffix}.bin"),
            "overflow": i32(f"overflow{suffix}.bin"),
        }

    dev = load_dev("")
    dev2 = load_dev("-run2")
    ref_x1 = np.asarray(ref["x1"], dtype=np.float32)
    ref_x2 = np.asarray(ref["x2"], dtype=np.float32)
    ref_dist = np.asarray(ref["dist"], dtype=np.float32)

    # 派生法线（与交付同口径：n = normalize((x2-x1)/dist)）
    normals_dev, normals_ref, angle_deg = [], [], []
    for i in range(n):
        d = float(dev["dist"][i]); dr = float(ref_dist[i])
        vd = (dev["x2"][i].astype(np.float64) - dev["x1"][i].astype(np.float64)) / d
        vr = (ref_x2[i].astype(np.float64) - ref_x1[i].astype(np.float64)) / dr
        nd = vd / np.linalg.norm(vd)
        nref = vr / np.linalg.norm(vr)
        normals_dev.append(nd)
        normals_ref.append(nref)
        angle_deg.append(math.degrees(math.acos(float(np.clip(np.dot(nd, nref), -1.0, 1.0)))))

    # ---------- 1) 统计独立重算 ----------
    dist_bit_equal = 0
    dist_ulps = []
    dist_bad = []
    for w in range(n):
        a = ref_dist[w]
        b = dev["dist"][w]
        if a.tobytes() == b.tobytes():
            dist_bit_equal += 1
        u = ulp_diff(b, a)
        dist_ulps.append({"world": w, "ulp": u})
        if u > 4.0:
            dist_bad.append(w)
    dist_max_ulp = max(x["ulp"] for x in dist_ulps)

    def axis_stats(dev_pts, ref_pts):
        rows = []
        for w in range(n):
            scale = float(np.max(np.abs(ref_pts[w])))
            bound = 4.0 * spacing(scale)
            delta = np.asarray(dev_pts[w] - ref_pts[w], dtype=np.float64)
            worst = float(np.max(np.abs(delta)))
            worst_axis = int(np.argmax(np.abs(delta)))
            normal = np.asarray(ref["normal_ref"][w], dtype=np.float64) if "normal_ref" in ref else None
            rows.append({"world": w, "scale": scale, "bound": bound, "delta": delta.tolist(),
                         "worst_abs": worst, "worst_axis": worst_axis,
                         "over_bound": worst / bound if bound else float("inf"),
                         "ok": worst <= bound})
        return rows

    x1_rows = axis_stats(dev["x1"], ref_x1)
    x2_rows = axis_stats(dev["x2"], ref_x2)
    x1_bad = [r["world"] for r in x1_rows if not r["ok"]]
    x2_bad = [r["world"] for r in x2_rows if not r["ok"]]
    x1_worst = max(x1_rows, key=lambda r: r["worst_abs"])
    x2_worst = max(x2_rows, key=lambda r: r["worst_abs"])
    ncontact_equal = sum(1 for w in range(n) if dev["ncontact"][w] == ref["ncontact"][w])
    index_equal = sum(1 for w in range(n) if dev["index"][w] == ref["index"][w])
    overflow_nonzero = sum(1 for w in range(n) if dev["overflow"][w] != 0 or ref["overflow"][w] != 0)
    two_runs_fields = ["dist", "x1", "x2", "ncontact", "index", "overflow"]
    two_runs_identical = all(
        np.array_equal(dev[f], dev2[f], equal_nan=True) for f in two_runs_fields)

    depths = [-float(ref_dist[w]) for w in range(n)]
    stats = {
        "worlds": n,
        "two_runs_bitwise_identical": bool(two_runs_identical),
        "dist_bit_equal": dist_bit_equal,
        "dist_max_ulp": dist_max_ulp,
        "dist_bad_worlds": dist_bad,
        "x1_bad_worlds": x1_bad,
        "x2_bad_worlds": x2_bad,
        "x1_worst": {"world": x1_worst["world"], "worst_abs": x1_worst["worst_abs"],
                     "bound": x1_worst["bound"], "over_bound": x1_worst["over_bound"]},
        "x2_worst": {"world": x2_worst["world"], "worst_abs": x2_worst["worst_abs"],
                     "bound": x2_worst["bound"], "over_bound": x2_worst["over_bound"]},
        "ncontact_equal_worlds": ncontact_equal,
        "index_equal_worlds": index_equal,
        "overflow_nonzero_worlds": overflow_nonzero,
        "depth_min": min(depths), "depth_max": max(depths),
        "depth_median": float(np.median(depths)),
        "over_bound_depths": {str(w): depths[w] for w in sorted(set(x1_bad) | set(x2_bad))},
    }
    rep["recompute"] = stats

    # 交付声明数字（从交付 analysis.json 读取，避免手抄；只作为对照）
    analysis = load(DELIVERY / "analysis.json")
    a_sum = analysis["summary"]
    a_details = {d["world"]: d for d in a_sum["over_bound_details"]}
    delivered = {
        "dist_bit_equal": a_sum["dist_bit_equal_worlds"],
        "dist_max_ulp": a_sum["dist_max_ulp"],
        "dist_bad": (len(a_sum["dist_over_bound_worlds"])
                     if isinstance(a_sum["dist_over_bound_worlds"], (list, dict))
                     else int(a_sum["dist_over_bound_worlds"])),
        "x1_bad": a_sum["x1_over_bound_worlds"],
        "x2_bad": a_sum["x2_over_bound_worlds"],
        "x1_max_abs": a_sum["x1_worst"]["worst_abs"],
        "x2_max_abs": a_sum["x2_worst"]["worst_abs"],
        "depth_min": a_sum["penetration_depth_min"],
        "depth_max": a_sum["penetration_depth_max"],
        "depth_median": a_sum["penetration_depth_median"],
        "x2_world10": a_details[10]["x2_worst_abs"],
        "x2_world15": a_details[15]["x2_worst_abs"],
        "x1_world10": a_details[10]["x1_worst_abs"],
        "x1_world15": a_details[15]["x1_worst_abs"],
        "depth_world8": a_details[8]["depth"],
    }
    upper_median = sorted(depths)[n // 2]
    rep["delivery_claim_check"] = {
        "dist_bit_equal_matches": dist_bit_equal == delivered["dist_bit_equal"],
        "dist_max_ulp_matches": abs(dist_max_ulp - delivered["dist_max_ulp"]) < 1e-9,
        "dist_bad_matches": len(dist_bad) == delivered["dist_bad"],
        "x1_bad_matches": x1_bad == delivered["x1_bad"],
        "x2_bad_matches": x2_bad == delivered["x2_bad"],
        "x1_max_abs_matches": abs(x1_worst["worst_abs"] - delivered["x1_max_abs"]) < 1e-15,
        "x2_max_abs_matches": abs(x2_worst["worst_abs"] - delivered["x2_max_abs"]) < 1e-15,
        "depth_range_matches": abs(min(depths) - delivered["depth_min"]) < 1e-12
                               and abs(max(depths) - delivered["depth_max"]) < 1e-12,
        "depth_median_matches_true_median": abs(stats["depth_median"] - delivered["depth_median"]) < 1e-12,
        "x2_world10_matches": abs(float(x2_rows[10]["worst_abs"]) - delivered["x2_world10"]) < 1e-15,
        "x2_world15_matches": abs(float(x2_rows[15]["worst_abs"]) - delivered["x2_world15"]) < 1e-15,
        "x1_world10_matches": abs(float(x1_rows[10]["worst_abs"]) - delivered["x1_world10"]) < 1e-15,
        "x1_world15_matches": abs(float(x1_rows[15]["worst_abs"]) - delivered["x1_world15"]) < 1e-15,
        "depth_world8_matches": abs(depths[8] - delivered["depth_world8"]) < 1e-12,
        "ncontact_index_overflow_matches": ncontact_equal == 24 and index_equal == 24
                                           and overflow_nonzero == 0,
        "two_runs_identical_matches": bool(two_runs_identical),
    }
    # 中位数口径：交付用 depths[len//2]（排序后上中位），非两中位平均
    rep["numeric_discrepancies"] = [{
        "field": "penetration_depth_median",
        "delivered": delivered["depth_median"],
        "recomputed_true_median": stats["depth_median"],
        "recomputed_upper_median": upper_median,
        "delivered_definition": "sorted_depths[len//2]（上中位数，n=24 时取第 13 小）"
                                if abs(delivered["depth_median"] - upper_median) < 1e-15 else "unknown",
        "impact": "仅影响 README 的「中位 0.1101」表述（真实两中位平均 0.1089）；"
                  "不影响越界集合、极值、相关性或任何结论。",
        "severity": "suggestion",
    }]

    # 与 QA 汇总 qa-compare-gpu.json 的 9 项交叉核对（独立重算 ↔ 我方汇总 ↔ 交付声明）
    qa_cmp = load(QA5 / "qa-compare-gpu.json")["witness_cases"]["cylinder-box-sweep"]
    qa_nine = {
        "x1_bad": qa_cmp["x1"]["bad"], "x2_bad": qa_cmp["x2"]["bad"],
        "x1_max_abs": qa_cmp["x1"]["max_abs"], "x2_max_abs": qa_cmp["x2"]["max_abs"],
        "dist_bit_equal": qa_cmp["dist"]["bit_equal"], "dist_max_ulp": qa_cmp["dist"]["max_ulp"],
        "dist_bad": qa_cmp["dist"]["bad"],
        "ncontact_equal": qa_cmp["ncontact_equal"],
        "two_runs_identical": qa_cmp["two_runs_bitwise_identical"],
        "index_equal": qa_cmp["index_device"] == qa_cmp["index_upstream"],
        "overflow_zero": all(v == 0 for v in qa_cmp["overflow_device"]),
    }
    mine_nine = {
        "x1_bad": len(x1_bad), "x2_bad": len(x2_bad),
        "x1_max_abs": x1_worst["worst_abs"], "x2_max_abs": x2_worst["worst_abs"],
        "dist_bit_equal": dist_bit_equal, "dist_max_ulp": dist_max_ulp, "dist_bad": len(dist_bad),
        "ncontact_equal": ncontact_equal == n, "two_runs_identical": bool(two_runs_identical),
        "index_equal": index_equal == n, "overflow_zero": overflow_nonzero == 0,
    }
    rep["cross_check_9"] = {
        "qa_summary": qa_nine,
        "qa_json_self_consistency": {
            "x1_bad": qa_nine["x1_bad"] == mine_nine["x1_bad"],
            "x2_bad": qa_nine["x2_bad"] == mine_nine["x2_bad"],
            "x1_max_abs": abs(qa_nine["x1_max_abs"] - mine_nine["x1_max_abs"]) < 1e-15,
            "x2_max_abs": abs(qa_nine["x2_max_abs"] - mine_nine["x2_max_abs"]) < 1e-15,
            "dist_bit_equal": qa_nine["dist_bit_equal"] == mine_nine["dist_bit_equal"],
            "dist_max_ulp": abs(qa_nine["dist_max_ulp"] - mine_nine["dist_max_ulp"]) < 1e-9,
            "dist_bad": qa_nine["dist_bad"] == mine_nine["dist_bad"],
            "ncontact_equal": qa_nine["ncontact_equal"] == mine_nine["ncontact_equal"],
            "two_runs_identical": qa_nine["two_runs_identical"] == mine_nine["two_runs_identical"],
            "index_equal": qa_nine["index_equal"] == mine_nine["index_equal"],
            "overflow_zero": qa_nine["overflow_zero"] == mine_nine["overflow_zero"],
        },
    }
    rep["cross_check_9"]["all_match"] = all(rep["cross_check_9"]["qa_json_self_consistency"].values())

    # 数据源 sha256（交付记录了 12 个未跟踪 bin 的 sha256；独立复算）
    recorded = analysis["source"]["device_bin_sha256"]
    sha_rows = {}
    for name, want in recorded.items():
        f = OUT / name
        sha_rows[name] = {"exists": f.exists(),
                          "sha256_match": f.exists() and sha256(f) == want}
    rep["device_bin_sha256"] = {
        "recorded_count": len(recorded),
        "all_match": all(v["sha256_match"] for v in sha_rows.values()),
        "rows": sha_rows,
        "delivery_has_bin_files": sorted(p.name for p in DELIVERY.rglob("*.bin")),
    }

    # pose 生成方法与 world 数（独立核）
    prep_src = (QA5 / "qa_prepare_capture.py").read_text(encoding="utf-8")
    rep["pose_generation_check"] = {
        "rng_seed_20251025_in_source": "default_rng(20251025)" in prep_src,
        "angle_range_pm_1_4": "uniform(-1.4, 1.4)" in prep_src,
        "free_joint_uniform_pm_0_08": "uniform(-0.08, 0.08)" in prep_src,
        "worlds_claimed": entry["worlds"],
        "worlds_in_data": n,
    }

    # ---------- 2) world 8 归因复算 + 循环性检查 ----------
    w = 8
    d_dev = float(dev["dist"][w])
    d_ref = float(ref_dist[w])
    x1d, x2d = dev["x1"][w].astype(np.float64), dev["x2"][w].astype(np.float64)
    x1r, x2r = ref_x1[w].astype(np.float64), ref_x2[w].astype(np.float64)
    # witness 关系：n = (x2-x1)/dist（交付 notes 明确该反解口径）
    n_dev_raw = (x2d - x1d) / d_dev
    n_ref_raw = (x2r - x1r) / d_ref
    n_dev = n_dev_raw / np.linalg.norm(n_dev_raw)
    n_ref = n_ref_raw / np.linalg.norm(n_ref_raw)
    cosang = float(np.clip(np.dot(n_dev, n_ref), -1.0, 1.0))
    dtheta = math.acos(cosang)
    predicted = abs(d_dev) * dtheta
    measured = float(np.linalg.norm((x1d - x2d) - (x1r - x2r)))
    ratio = measured / predicted if predicted else float("nan")
    # 自洽残差：|(x2-x1) - dist*n̂| / |dist|，n̂ 为归一化派生法线
    residual = float(np.linalg.norm((x2d - x1d) - d_dev * n_dev) / abs(d_dev))
    # 恒等式检查：Δx1-Δx2 与 d_dev*n_dev - d_ref*n_ref 是同一个量（定义使然）
    delta_pair = (x1d - x2d) - (x1r - x2r)
    delta_pair_from_normals = d_dev * n_dev - d_ref * n_ref
    identity_gap = float(np.linalg.norm(delta_pair - delta_pair_from_normals))
    delta_d = abs(d_dev - d_ref)
    # 对比量：若只差 dist（同法线），预期平移 = |Δdist|；若只差法线，预期 = |dist|·δθ
    rep["attribution_world8"] = {
        "dist_dev": d_dev, "dist_ref": d_ref, "dist_abs_diff": delta_d,
        "normal_angle_rad": dtheta, "normal_angle_deg": math.degrees(dtheta),
        "normal_delta_norm": float(np.linalg.norm(n_dev - n_ref)),
        "predicted_shift": predicted, "measured_point_pair_shift": measured,
        "ratio_measured_over_predicted": ratio,
        "witness_identity_residual_relative": residual,
        "identity_gap_between_two_forms": identity_gap,
        "delta_pair": delta_pair.tolist(),
        "delta_pair_from_normals": delta_pair_from_normals.tolist(),
        "normal_angle_rank_among_24": sorted(angle_deg, reverse=True).index(math.degrees(dtheta)) + 1,
        "identity_is_definitional": "n 由 (x2-x1)/dist 反解，故 Δx1-Δx2 ≡ d_dev·n_dev - d_ref·n_ref；"
                                    "实测点对差 = |Δx1-Δx2| 只是该恒等式的数值重述（Δdist 仅占 0.011%），"
                                    "说明「差=法线差×深度」在 d 相等时必然成立，不能据此判定哪一方正确或根因。",
        "delta_dist_share": delta_d / abs(d_dev),
        "alternative_consistent_hypotheses": [
            "同一接触面、EPA 法线数值差（同顶点集不同迭代/精度）",
            "不同接触面（选面 tie-break）但同 index/ncontact",
        ],
        "discriminating_data_available": False,
        "discriminating_data_needed": ["face_idx", "EPA 三顶点", "重心权重"],
    }

    # 相关性（与交付同定义：depth vs x1/x2 over_bound、x1 法向/切向分量）
    normal_refs = np.asarray(normals_ref, dtype=np.float64)
    x1_ratio = np.array([r["over_bound"] for r in x1_rows])
    x2_ratio = np.array([r["over_bound"] for r in x2_rows])
    x1_norm_comp = np.array([abs(float(np.dot(x1_rows[i]["delta"], normal_refs[i]))) for i in range(n)])
    x1_tan_comp = np.array([float(np.linalg.norm(
        np.asarray(x1_rows[i]["delta"]) - np.dot(np.asarray(x1_rows[i]["delta"]), normal_refs[i]) * normal_refs[i]))
        for i in range(n)])

    def pearson(a, b):
        return float(np.corrcoef(a, b)[0, 1])

    corr = {
        "depth_vs_x1_over_bound": pearson(depths, x1_ratio),
        "depth_vs_x2_over_bound": pearson(depths, x2_ratio),
        "depth_vs_x1_normal_component": pearson(depths, x1_norm_comp),
        "depth_vs_x1_tangential_component": pearson(depths, x1_tan_comp),
    }
    def pearson_p(r, nn):
        # 双侧 t 检验，数值积分 t 分布（不依赖 scipy）
        if abs(r) >= 1.0:
            return 0.0
        df = nn - 2
        t = abs(r) * math.sqrt(df / (1.0 - r * r))

        def pdf(x):
            return (math.gamma((df + 1) / 2.0) /
                    (math.sqrt(df * math.pi) * math.gamma(df / 2.0)) *
                    (1.0 + x * x / df) ** (-(df + 1) / 2.0))

        steps = 2000
        integral = 0.0
        for i in range(steps + 1):
            u = i / steps
            w = 1.0 if i in (0, steps) else (4.0 if i % 2 else 2.0)
            integral += w * pdf(t * u)
        integral *= t / (3.0 * steps)
        return 2.0 * (0.5 - integral)

    pvals = {k: pearson_p(r, n) for k, r in {
        "x1_over_bound": corr["depth_vs_x1_over_bound"],
        "x2_over_bound": corr["depth_vs_x2_over_bound"],
        "x1_normal_component": corr["depth_vs_x1_normal_component"],
        "x1_tangential_component": corr["depth_vs_x1_tangential_component"]}.items()}
    rep["correlation"] = {
        "computed": corr,
        "delivered": analysis["summary"].get("correlation_pearson"),
        "matches_delivered": all(
            abs(corr[k] - analysis["summary"]["correlation_pearson"][k]) < 1e-9
            for k in corr if k in analysis["summary"].get("correlation_pearson", {})),
        "p_values_approx": pvals,
        "note": "|r|≤0.25、n=24：仅说明该样本下无显著线性趋势（p 约 0.24–0.68），"
                "不能作为「不存在系统性深度依赖」的强证据；结论应标注样本量限制。",
    }

    # ---------- 3) 结论边界 / 未覆盖披露 / 范围 ----------
    readme = (DELIVERY / "README.md").read_text(encoding="utf-8")
    receipt = load(DELIVERY / "receipt.json")
    forbidden = ["G10已实现", "完整物理通过", "双平台通过", "上游完全等价", "原生放行", "精度达标"]
    rep["boundary_check"] = {
        "forbidden_claim_hits": [w_ for w_ in forbidden if w_ in readme or w_ in json.dumps(receipt, ensure_ascii=False)],
        "fix_needed_phrased_as_recommendation": ("不需要" in readme and "建议" in readme),
        "decision_owner_stated": "Arch→Chief→用户" in readme or "Arch" in readme and "Chief" in readme and "用户" in readme,
        "judgement_change_needs_approval": "任何判据/声明面调整都需" in readme,
        "no_self_relaxation": "本片不自行放宽" in readme,
        "uncovered_face_data": "face_idx" in readme and "三顶点" in readme and "权重" in readme,
        "uncovered_frame_normal": "frame" in readme and "normal" in readme and "无" in readme,
        "uncovered_bins_untracked": "未跟踪" in readme and "sha256" in readme,
        "states_cannot_prove_device_face_equivalence": "不能" in readme and "等价" in readme,
        "g5_conclusion_unchanged": "G5 交付结论不变" in readme,
    }
    fix_files = [ln.split("\t")[-1] for ln in
                 git("show", "--name-status", "--format=", FIX).stdout.decode("utf-8", "replace").splitlines()
                 if ln.strip()]
    product_changes = [f for f in fix_files
                       if f.startswith(("src/", "tests/", "Cargo", "fixtures/", "scripts/"))]
    rep["scope"] = {
        "fix_commit": FIX,
        "files": fix_files,
        "product_or_fixture_changes": product_changes,
        "only_evidence_and_normify": all(
            f.startswith(("docs/development-evidence/", ".normify/")) for f in fix_files),
        "delivery_has_bin": False,
        "gpu_window": False,
    }

    # ---------- 判定 ----------
    all_claim = {k: v for k, v in rep["delivery_claim_check"].items()
                 if k != "depth_median_matches_true_median"}
    rep["verdict"] = {
        "recompute_matches_delivery": all(all_claim.values()),
        "median_caliber_observation": len(rep["numeric_discrepancies"]) == 1,
        "cross_check_9_all_match": rep["cross_check_9"]["all_match"],
        "device_bin_sha256_ok": rep["device_bin_sha256"]["all_match"],
        "attribution_arithmetic_correct": abs(ratio - 1.0) < 1e-3,
        "attribution_circularity_flagged": True,
        "boundary_ok": (rep["boundary_check"]["forbidden_claim_hits"] == []
                        and rep["boundary_check"]["fix_needed_phrased_as_recommendation"]
                        and rep["boundary_check"]["decision_owner_stated"]
                        and rep["boundary_check"]["judgement_change_needs_approval"]
                        and rep["boundary_check"]["states_cannot_prove_device_face_equivalence"]),
        "scope_ok": rep["scope"]["only_evidence_and_normify"]
                    and rep["scope"]["product_or_fixture_changes"] == [],
        "pass": False,
    }
    v = rep["verdict"]
    v["pass"] = all([v["recompute_matches_delivery"], v["cross_check_9_all_match"],
                     v["device_bin_sha256_ok"], v["attribution_arithmetic_correct"],
                     v["boundary_ok"], v["scope_ok"]])

    (EVIDENCE / "qa-deep-epa-check.json").write_text(
        json.dumps(rep, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("== recompute ==")
    print(json.dumps({k: stats[k] for k in ("worlds", "dist_bit_equal", "dist_max_ulp",
                                            "dist_bad_worlds", "x1_bad_worlds", "x2_bad_worlds",
                                            "x1_worst", "x2_worst", "ncontact_equal_worlds",
                                            "index_equal_worlds", "overflow_nonzero_worlds",
                                            "depth_min", "depth_max", "depth_median",
                                            "two_runs_bitwise_identical")}, ensure_ascii=False))
    print("== delivery claim ==")
    print(json.dumps(all_claim, ensure_ascii=False))
    print("== cross check 9 ==")
    print(json.dumps({k: v_ for k, v_ in rep["cross_check_9"].items() if k != "qa_summary"}, ensure_ascii=False))
    print("== data source sha ==")
    print(json.dumps({k: v_ for k, v_ in rep["device_bin_sha256"].items() if k != "rows"}, ensure_ascii=False))
    print("== attribution w8 ==")
    print(json.dumps({k: v_ for k, v_ in rep["attribution_world8"].items()
                      if k not in ("delta_pair", "delta_pair_from_normals", "alternative_consistent_hypotheses",
                                   "discriminating_data_needed")}, ensure_ascii=False))
    print("== correlation ==")
    print(json.dumps({k: v_ for k, v_ in rep["correlation"].items() if k != "note"}, ensure_ascii=False))
    print("== numeric discrepancies ==")
    print(json.dumps(rep["numeric_discrepancies"], ensure_ascii=False))
    print("== boundary ==")
    print(json.dumps(rep["boundary_check"], ensure_ascii=False))
    print("== scope ==")
    print(json.dumps(rep["scope"], ensure_ascii=False))
    print("== verdict ==")
    print(json.dumps(v, ensure_ascii=False))
    print("report:", EVIDENCE / "qa-deep-epa-check.json")
    return 0 if v["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
