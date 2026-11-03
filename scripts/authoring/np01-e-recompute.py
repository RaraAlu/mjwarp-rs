#!/usr/bin/env python3
"""NP01 A0：E 指标三候选重算（离线、只读输入、不启动产品链、零 GPU）。

输入（仓库内已归档证据，只读）：
  docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-native-f64.json
  docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json
  docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json
  docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json

输出：
  docs/development-evidence/np01-e-recompute/e_recompute.json（README 数字全部由本脚本产出）

口径：
  * E 严格按契约 t_i = atol + rtol*|native_i|，E = max_i |cand_i - native_i| / t_i。
  * 产品（设备）侧原始接触数组未归档，归档只有「设备 vs 上游」聚合（bit_equal/max_abs/max_ulp），
    因此 E_candidate 只给上下界（|dev-native| 介于 |up-native|-e 与 |up-native|+e，e=max|dev-up|），
    上下界显式标注，不冒充精确值。
  * 配对：count 相等才配对；同 world 内单接触直接配；多接触用位置最近邻贪心，
    并输出配对距离与最近邻间距基线用于判定可靠性。
"""

import json
import math
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
QA = os.path.join(ROOT, "docs", "development-evidence", "g10-p5-gpu-kernels-independent-check")
G5 = os.path.join(ROOT, "docs", "development-evidence", "g10-p5-gpu-kernels", "gpu-window")
OUT = os.path.join(ROOT, "docs", "development-evidence", "np01-e-recompute")

# 契约 docs/numerical-precision-contract.md L246-L254（用户 2026-10-27 拍板）：
# 值为 (atol, rtol)；frame/normal 的 rtol = 0。
TIERS = {
    ("dist", "CONVEX"): (1e-6, 1e-6),
    ("pos", "CONVEX"): (1e-6, 1e-6),
    ("frame", "CONVEX"): (2e-6, 0.0),
    ("dist", "SDF"): (2e-6, 1e-6),
    ("pos", "SDF"): (1e-6, 1e-6),
    ("frame", "SDF"): (5e-5, 0.0),
}
CASE_PATH = {"ellipsoid-box": "CONVEX", "sdf-box": "SDF"}
UNPAIRABLE = {"hfield-sphere": "HFIELD", "hfield-capsule": "HFIELD"}
F32_EPS = 2.0 ** -23


def load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def native_worlds(case):
    worlds = case["worlds"]
    groups = case["contacts"]
    assert len(groups) == worlds, "native contacts group count != worlds"
    return [
        [dict(dist=c["dist"], pos=list(c["pos"]), frame=list(c["frame"])) for c in group]
        for group in groups
    ]


def upstream_worlds(fields, worlds):
    groups = [[] for _ in range(worlds)]
    for i, w in enumerate(fields["worldid"]):
        groups[w].append(
            dict(
                dist=fields["dist"][i],
                pos=[fields["pos"][3 * i + k] for k in range(3)],
                frame=[fields["frame"][9 * i + k] for k in range(9)],
            )
        )
    return groups


def nn_spacing(points):
    out = []
    for i, p in enumerate(points):
        best = None
        for j, q in enumerate(points):
            if i != j:
                d = math.dist(p, q)
                best = d if best is None else min(best, d)
        if best is not None:
            out.append(best)
    out.sort()
    return out[len(out) // 2] if out else None


def pair_world(nat, up):
    if len(nat) != len(up):
        return None, dict(reason="count_mismatch", native=len(nat), upstream=len(up))
    if len(nat) == 1:
        return [(nat[0], up[0])], dict(method="1:1", native=1, upstream=1, pairing_reliable=True)
    remaining = list(range(len(up)))
    pairs, dists = [], []
    for n in nat:
        best = min(((math.dist(n["pos"], up[i]["pos"]), i) for i in remaining))
        dists.append(best[0])
        pairs.append((n, up[best[1]]))
        remaining.remove(best[1])
    dists.sort()
    spacing = nn_spacing([u["pos"] for u in up])
    median = dists[len(dists) // 2]
    return pairs, dict(
        method="nearest_neighbor_greedy",
        native=len(nat),
        upstream=len(up),
        pairing_distance_median=median,
        pairing_distance_max=dists[-1],
        nearest_neighbor_spacing_p50=spacing,
        pairing_reliable=(spacing is not None and median <= 0.5 * spacing),
    )


def field_components(pair, field):
    n, u = pair
    if field == "dist":
        return [(abs(n["dist"]), u["dist"], n["dist"])]
    return [(abs(n[field][k]), u[field][k], n[field][k]) for k in range(len(u[field]))]


def e_metric(pairs, field, path):
    atol, rtol = TIERS[(field, path)]
    worst = 0.0
    for pair in pairs:
        for nat_abs, up_val, nat_val in field_components(pair, field):
            worst = max(worst, abs(up_val - nat_val) / (atol + rtol * nat_abs))
    return worst


def field_components_flat(pairs, field):
    out = []
    for pair in pairs:
        out.extend(field_components(pair, field))
    return out


def main():
    native = load(os.path.join(QA, "qa-native-f64.json"))
    upstream = load(os.path.join(QA, "qa-upstream-gpu.json"))
    compare = load(os.path.join(QA, "qa-compare-gpu.json"))
    g5 = load(os.path.join(G5, "result.json"))

    dev_vs_up = {
        c["case"]: dict(
            dist_ulp=c.get("dist_max_ulp") or 0,
            pos_max_abs=c.get("pos_max_abs"),
            frame_max_abs=c.get("frame_max_abs"),
            dist_components=c.get("dist_components"),
        )
        for c in g5["cases"]
    }

    report = {
        "schema": 1,
        "task_id": "2026-11-04-precision-np01-e-recompute",
        "kind": "np01-e-three-candidates",
        "formula": "E = max_i |cand_i - native_i| / (atol + rtol*|native_i|); redline = 1.05 * E_mjwarp",
        "inputs": {
            "native_f64": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-native-f64.json",
            "upstream_f32_gpu": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json",
            "qa_compare": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json",
            "device_vs_upstream": "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json",
            "tier_table": "docs/numerical-precision-contract.md L246-L254 (user 2026-10-27)",
        },
        "candidates": {},
        "caveats": [
            "device raw contact arrays are not archived: E_candidate is bounded, not exact",
            "hfield cases: native 6/6 and 6/3 vs warp 8/7 contacts -> unpairable; no signed HFIELD tier",
            "sdf-box is 40:40 nearest-neighbour pairing; pos gap ~7e-2 > NN spacing p50 ~1.4e-2, so pairing reliability is doubtful and the conclusion is diagnostic only",
        ],
    }

    # ---- candidate 2: native f64 as reference ----
    cand2 = {"basis": "native f64 contact arrays (QA archive)", "cases": {}, "excluded": {}}
    for case in UNPAIRABLE:
        material = compare["g6_tolerance_candidates"]["native_f64_material"][case]
        cand2["excluded"][case] = dict(
            reason="HFIELD has no signed tier; native vs warp contact counts differ so pairing impossible",
            native_counts=material.get("native_counts"),
            warp_counts=material.get("warp_counts"),
            usable_as_tolerance_basis=material.get("usable_as_tolerance_basis"),
        )
    for case, path in CASE_PATH.items():
        worlds = native["cases"][case]["worlds"]
        nat_groups = native_worlds(native["cases"][case])
        up_groups = upstream_worlds(upstream["cases"][case]["fields"], worlds)
        entry = {"path": path, "worlds": worlds, "fields": {}, "pairing": []}
        pairs_all = []
        paired = True
        for w in range(worlds):
            pairs, meta = pair_world(nat_groups[w], up_groups[w])
            entry["pairing"].append(meta)
            if pairs is None:
                paired = False
                break
            pairs_all.append(pairs)
        if not paired:
            entry["pairing_failed"] = True
            cand2["cases"][case] = entry
            continue

        material = compare["g6_tolerance_candidates"]["native_f64_material"][case]["native_vs_warp_max"]
        dev = dev_vs_up[case]
        nat_dist_max = max(abs(p["dist"]) for groups in nat_groups for p in groups)
        dev_bound = {
            "dist": dev["dist_ulp"] * nat_dist_max * F32_EPS,
            "pos": dev["pos_max_abs"],
            "frame": dev["frame_max_abs"],
        }
        for field in ("dist", "pos", "frame"):
            atol, rtol = TIERS[(field, path)]
            e_up = max(e_metric(pairs, field, path) for pairs in pairs_all)
            d = material.get(f"{field}_abs")
            if d is None:
                continue
            e = dev_bound[field]
            nat_abs_max = max(
                nat_abs
                for pairs in pairs_all
                for nat_abs, _u, _n in field_components_flat(pairs, field)
            )
            denom_max = atol + rtol * nat_abs_max
            entry["fields"][field] = dict(
                E_mjwarp=e_up,
                E_candidate_lower_bound=max(0.0, d - e) / denom_max,
                E_candidate_upper_bound=(d + e) / atol,
                upstream_vs_native_max_abs=d,
                device_vs_upstream_bound=e,
                atol=atol,
                rtol=rtol,
                formula_note="lower bound uses max denominator, upper bound uses min denominator (atol); exact per-element value needs device raw arrays",
            )
        cand2["cases"][case] = entry
    report["candidates"]["native_f64"] = cand2

    # ---- redline ratio: device and upstream deviate from native nearly identically ----
    ratio = {}
    for case in CASE_PATH:
        entry = cand2["cases"].get(case, {})
        if not entry.get("fields"):
            continue
        ratio[case] = {
            field: dict(
                upstream_vs_native_max_abs=v["upstream_vs_native_max_abs"],
                device_vs_upstream_bound=v["device_vs_upstream_bound"],
                ratio_lower=max(0.0, v["upstream_vs_native_max_abs"] - v["device_vs_upstream_bound"])
                / v["upstream_vs_native_max_abs"],
                ratio_upper=(v["upstream_vs_native_max_abs"] + v["device_vs_upstream_bound"])
                / v["upstream_vs_native_max_abs"],
            )
            for field, v in entry["fields"].items()
        }
    report["redline_ratio"] = dict(
        note="redline E_candidate <= 1.05*E_mjwarp is a RELATIVE test: when device and upstream deviate from native almost identically (ratio ~ 1) the redline passes even though absolute E is huge; both facts must be reported separately",
        cases=ratio,
    )

    # ---- candidate 1: same-code cross-backend baseline + signed tier margins ----
    # 契约 L246-L254 的「实测 worst」列（分路径，来源 = native-contact-frame-normal-tiers 决策证据）；
    # QA 的 same_code_backend_baseline 不分路径，只作聚合交叉核对。
    baseline = compare["g6_tolerance_candidates"]["same_code_backend_baseline"]
    contract_worst = {
        ("dist", "CONVEX"): 9.31e-9,
        ("dist", "SDF"): 6.88e-7,
        ("pos", "CONVEX"): 1.86e-8,
        ("pos", "SDF"): 4.62e-7,
        ("frame", "CONVEX"): 8.19e-7,
        ("frame", "SDF"): 2.78e-5,
    }
    margins = {}
    for (field, path), (atol, _rtol) in TIERS.items():
        measured = contract_worst[(field, path)]
        margins[f"{field}/{path}"] = dict(
            atol=atol,
            measured_worst=measured,
            margin_factor=atol / measured if measured else None,
            within_tier=measured <= atol,
            worst_source="contract L246-L254 measured-worst column",
        )
    aggregate_check = {}
    for field in ("dist", "pos", "frame", "normal"):
        stats = baseline.get(field)
        if stats:
            aggregate_check[field] = dict(aggregate_max=stats["max"], note="baseline 不按路径拆分，仅作聚合交叉核对")
    tightest = min(margins.items(), key=lambda kv: kv[1]["margin_factor"] or float("inf"))
    report["candidates"]["same_code_backend"] = dict(
        basis="same-code cross-backend (warp CPU vs warp CUDA f32) error distribution; native f64 rejected as different engine semantics",
        baseline=baseline,
        baseline_not_split_by_path=aggregate_check,
        signed_tier_margin=margins,
        tightest_margin=dict(key=tightest[0], margin_factor=tightest[1]["margin_factor"]),
        redline_degenerate="E_mjwarp is 0 by definition here, so 1.05*E_mjwarp degenerates; the check becomes measured worst <= signed atol",
    )

    # ---- candidate 3: status quo ----
    report["candidates"]["status_quo"] = dict(
        basis="E not finalized; contact fields keep only upstream-f32 criteria (vector-scale ULP / frame absolute ULP / structural zero tolerance); tier table stays diagnostic on the native side",
        change_to_contract="none",
        risk="contact fields then have no numeric criterion on the native side, only the relative redline; reading the tier table as a native criterion would misjudge",
    )

    # ---- verdicts: what the archives can and cannot decide ----
    redline_check = {}
    for case, entry in cand2["cases"].items():
        if not entry.get("fields"):
            redline_check[case] = dict(pairable=False, pairing=entry.get("pairing"))
            continue
        redline_check[case] = dict(
            pairable=True,
            pairing_reliable=all(m.get("pairing_reliable", False) for m in entry["pairing"]),
            fields={},
        )
        for field, v in entry["fields"].items():
            redline = 1.05 * v["E_mjwarp"]
            low = v["E_candidate_lower_bound"]
            high = v["E_candidate_upper_bound"]
            if low > redline:
                verdict = "E_candidate > redline（界外）"
            elif high < redline:
                verdict = "E_candidate < redline（界内）"
            else:
                verdict = "界跨红线：归档数据不可判定"
            redline_check[case]["fields"][field] = dict(
                E_mjwarp=v["E_mjwarp"],
                redline=redline,
                E_candidate_bounds=[low, high],
                verdict=verdict,
                absolute_E_gt_1=v["E_mjwarp"] > 1.0,
            )
    report["verdicts"] = dict(
        native_f64=dict(
            decidability="归档数据只能判定 2/4 case（hfield 接触数与 warp 不等，不可配对）；且设备原始数组未归档 → 红线界跨，不能定稿",
            redline_check=redline_check,
            absolute_observation="ellipsoid-box 与 sdf-box 的 E_mjwarp 均可达数百~数千，说明分档 atol 不是 native 口径容差，而是 upstream-f32 口径",
            needed_for_finalize=[
                "补采 native f64 全量接触数组（离线 MuJoCo，CPU，无 GPU）",
                "补采设备（产品）原始接触数组以去掉上下界（小窗口或复用 QA 探针落盘）",
                "扩样到更多几何对/路径（当前 hfield 不可配对）",
            ],
        ),
        same_code_backend=dict(
            decidability="可用：6 个 (field,path) 全部实测 worst <= 已签 atol，最紧余量见 tightest_margin",
            caveat="E_mjwarp 恒 0 → redline 退化；定稿需把判据写成「实测 worst <= atol」核对，而不是 E 比值",
        ),
        status_quo=dict(
            decidability="可维持：零改契约，接触字段仍只有 upstream-f32 判据",
            caveat="native 口径无判据，只剩相对红线；分档表若被当成 native 判据会误判",
        ),
    )

    os.makedirs(OUT, exist_ok=True)
    out_path = os.path.join(OUT, "e_recompute.json")
    with open(out_path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)
        handle.write("\n")
    print("wrote", out_path)
    print("cand2:", json.dumps(cand2["cases"], ensure_ascii=False)[:600])
    print("margins:", json.dumps(margins, ensure_ascii=False))
    print("ratio:", json.dumps(ratio, ensure_ascii=False)[:600])


if __name__ == "__main__":
    main()
