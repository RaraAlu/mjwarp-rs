"""QA 只读复核：per-world L v1 参考「输入位姿不同、输出逐位相同」事实 + 机器判据分类 + 限定说明。

复核目标基线 HEAD e34df3b7（本卡登记 2746db06 之后只允许新增本卡证据）。
只读：不重跑窗口/GPU、不改历史证据/卡状态/fixtures/源码；仅写本卡证据目录。

用法（工作目录 = 仓库根）：
  python -B scripts/verify_l1_reference_review.py --out docs/development-evidence/render-bvh/perworld-l-v1-reference-discriminability-check
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys

import numpy as np

sys.dont_write_bytecode = True

BASE = "e34df3b7"
EVID_REF = "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/"
EVID_MAIN = "fixtures/render-bvh/scene-a/reference/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v1-reference-discriminability-check/"
CARD = ".normify/changes/2026-11-07-render-perworld-l-v1-reference-discriminability-check.json"
CARD_CHECK = CARD
STANDARD = "docs/development-evidence/criteria-purpose-audit/standard.md"
HISTORICAL = [
    "docs/development-evidence/render-bvh/perworld-impl-s1/",
    "docs/development-evidence/render-bvh/perworld-capture-run/",
    "docs/development-evidence/render-bvh/perworld-capture-check/",
    "docs/development-evidence/render-bvh/perworld-s1-window-material/",
    "docs/development-evidence/render-bvh/criteria-purpose-audit/",
]
checks = []


def run(*args):
    return subprocess.run(list(args), capture_output=True)


def sha_file(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + cid + " :: " + str(detail)[:420])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=CHECK)
    args = ap.parse_args()
    out = args.out
    arts = os.path.join(out, "artifacts")
    logs = os.path.join(out, "logs")
    os.makedirs(arts, exist_ok=True)
    os.makedirs(logs, exist_ok=True)

    def dump(name, obj):
        with open(os.path.join(arts, name), "w", encoding="utf-8", newline="\n") as f:
            json.dump(obj, f, ensure_ascii=False, indent=1)
            f.write("\n")

    # 0) 复核目标可追溯：BASE 之后只允许本卡登记/证据
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    anc = run("git", "merge-base", "--is-ancestor", BASE, "HEAD").returncode
    after = run("git", "diff", "--name-only", BASE, "HEAD").stdout.decode().splitlines()
    qa_names = ("perworld-l-v1-reference-discriminability-check",
                "perworld-l-v1-reference-review",
                "perworld-l-v2-capture-perworld-refresh-check")
    mine = [p for p in after if any(n in p for n in qa_names)]
    add("review_target_traceable_and_only_own_artifacts_after", anc == 0 and after == mine,
        {"base": BASE, "current_head": head, "after": after})

    # 1) 参考文件与 capture.json 哈希绑定
    cap = json.load(open(os.path.join(EVID_REF, "capture.json"), encoding="utf-8"))
    files = {f: sha_file(os.path.join(EVID_REF, f)) for f in ["rgb.npz", "depth.npz", "seg.npz", "state.npz"]}
    bound = cap.get("files_sha256", {})
    add("reference_hashes_bound_to_capture_json",
        files == bound and cap.get("two_captures_bitwise_identical") is True
        and cap.get("repeat") == 2 and cap.get("nworld") == 3
        and cap.get("per_world_scope", {}).get("covers", "").startswith("per-world geom poses"),
        {"files": files, "bound_equal": files == bound,
         "two_captures_bitwise_identical": cap.get("two_captures_bitwise_identical"),
         "per_world_scope": cap.get("per_world_scope")})

    # 2) rgb 三 world 逐位相同
    rgb = np.load(os.path.join(EVID_REF, "rgb.npz"))
    keys = ["rgb_w0_cam0", "rgb_w1_cam0", "rgb_w2_cam0"]
    rgb_arr = [rgb[k] for k in keys]
    rgb_eq = all(np.array_equal(rgb_arr[0], a) for a in rgb_arr[1:]) and \
        rgb_arr[0].tobytes() == rgb_arr[1].tobytes() == rgb_arr[2].tobytes()
    max_abs_rgb = float(max(np.max(np.abs(rgb_arr[0].astype(np.float64) - a.astype(np.float64))) for a in rgb_arr[1:]))
    add("rgb_three_worlds_bitwise_identical_max_abs_diff_0", rgb_eq and max_abs_rgb == 0.0,
        {"sha256": {k: hashlib.sha256(rgb[k].tobytes()).hexdigest() for k in keys},
         "max_abs_diff": max_abs_rgb})

    # 3) depth 三 world 逐位相同 + 变化像素 0/0
    depth = np.load(os.path.join(EVID_REF, "depth.npz"))
    dk = ["depth_w0_cam0", "depth_w1_cam0", "depth_w2_cam0"]
    d_arr = [depth[k] for k in dk]
    bits = lambda a, b: int(np.count_nonzero(a.view(np.uint32) != b.view(np.uint32)))
    depth_eq = all(np.array_equal(d_arr[0], a) for a in d_arr[1:])
    depth_changed = [bits(d_arr[0], d_arr[1]), bits(d_arr[1], d_arr[2])]
    add("depth_three_worlds_bitwise_identical_changed_pixels_0_0",
        depth_eq and depth_changed == [0, 0],
        {"sha256": {k: hashlib.sha256(depth[k].tobytes()).hexdigest() for k in dk},
         "bit_changed_pixels": depth_changed})

    # 4) seg 三 world 逐位相同 + 变化像素 0/0
    seg = np.load(os.path.join(EVID_REF, "seg.npz"))
    sk = ["seg_w0_cam0", "seg_w1_cam0", "seg_w2_cam0"]
    s_arr = [seg[k] for k in sk]

    def seg_changed(a, b):
        return int(np.count_nonzero((a[..., 0] != b[..., 0]) | (a[..., 1] != b[..., 1])))
    seg_eq = all(np.array_equal(s_arr[0], a) for a in s_arr[1:])
    seg_changed_px = [seg_changed(s_arr[0], s_arr[1]), seg_changed(s_arr[1], s_arr[2])]
    add("seg_three_worlds_bitwise_identical_changed_pixels_0_0",
        seg_eq and seg_changed_px == [0, 0],
        {"sha256": {k: hashlib.sha256(seg[k].tobytes()).hexdigest() for k in sk},
         "changed_pixels": seg_changed_px})
    dump("reference-facts.json", {
        "capture": {"task_id": cap.get("task_id"), "nworld": cap.get("nworld"), "hinge_deg": cap.get("hinge_deg"),
                    "repeat": cap.get("repeat"), "two_captures_bitwise_identical": cap.get("two_captures_bitwise_identical"),
                    "captured_at": cap.get("captured_at"), "per_world_scope": cap.get("per_world_scope")},
        "files_sha256": files,
        "rgb": {"sha256": {k: hashlib.sha256(rgb[k].tobytes()).hexdigest() for k in keys},
                "bitwise_identical": rgb_eq, "max_abs_diff": max_abs_rgb},
        "depth": {"sha256": {k: hashlib.sha256(depth[k].tobytes()).hexdigest() for k in dk},
                  "bitwise_identical": depth_eq, "bit_changed_pixels": depth_changed},
        "seg": {"sha256": {k: hashlib.sha256(seg[k].tobytes()).hexdigest() for k in sk},
                "bitwise_identical": seg_eq, "pair_changed_pixels": seg_changed_px}})

    # 5) state：输入位姿逐 world 不同（含 hinge 角）
    st = np.load(os.path.join(EVID_REF, "state.npz"))
    qpos = st["qpos"]
    hinge = [float(qpos[w][7]) for w in range(3)]
    gx = st["geom_xpos"]
    gxm = st["geom_xmat"]
    state_facts = {
        "qpos_hinge_rad": hinge,
        "qpos_all_equal": bool(np.array_equal(qpos[0], qpos[1]) and np.array_equal(qpos[1], qpos[2])),
        "geom_xpos_all_equal": bool(np.array_equal(gx[0], gx[1]) and np.array_equal(gx[1], gx[2])),
        "geom_xpos_max_abs_diff_0_1": float(np.max(np.abs(gx[0].astype(np.float64) - gx[1].astype(np.float64)))),
        "geom_xpos_max_abs_diff_0_2": float(np.max(np.abs(gx[0].astype(np.float64) - gx[2].astype(np.float64)))),
        "geom_xmat_all_equal": bool(np.array_equal(gxm[0], gxm[1]) and np.array_equal(gxm[1], gxm[2])),
        "geom_xmat_max_abs_diff_0_1": float(np.max(np.abs(gxm[0].astype(np.float64) - gxm[1].astype(np.float64)))),
        "light_xpos_all_equal": bool(np.array_equal(st["light_xpos"][0], st["light_xpos"][1])
                                     and np.array_equal(st["light_xpos"][1], st["light_xpos"][2])),
        "light_xdir_all_equal": bool(np.array_equal(st["light_xdir"][0], st["light_xdir"][1])
                                     and np.array_equal(st["light_xdir"][1], st["light_xdir"][2])),
    }
    add("state_inputs_differ_per_world_but_lights_equal",
        not state_facts["qpos_all_equal"] and not state_facts["geom_xpos_all_equal"]
        and not state_facts["geom_xmat_all_equal"] and state_facts["light_xpos_all_equal"]
        and state_facts["light_xdir_all_equal"]
        and round(hinge[1], 6) == round(np.deg2rad(20), 6) and round(hinge[2], 6) == round(np.deg2rad(40), 6),
        state_facts)
    dump("state-facts.json", state_facts)

    # 6) 结构性成因线索：参考 seg 只含 floor/背景；同场景 cam-persp 同构，cam-track 含几何
    def seg_pairs(arr):
        pairs = np.stack([arr[..., 0].ravel(), arr[..., 1].ravel()], axis=1)
        uniq, counts = np.unique(pairs, axis=0, return_counts=True)
        return [{"id": int(i), "type": int(t), "pixels": int(c)} for (i, t), c in zip(uniq.tolist(), counts.tolist())]
    ref_pairs = seg_pairs(s_arr[0])
    main_seg = np.load(os.path.join(EVID_MAIN, "seg.npz"))
    main_cam0 = seg_pairs(main_seg["seg_cam0"])
    main_cam1 = seg_pairs(main_seg["seg_cam1"])
    ref_ids = {p["id"] for p in ref_pairs}
    cam1_ids = {p["id"] for p in main_cam1}
    visibility = {
        "perworld_cam0_seg_pairs": ref_pairs,
        "scene_a_reference_cam0_seg_pairs": main_cam0,
        "scene_a_reference_cam1_seg_pairs": main_cam1,
        "perworld_cam0_only_floor_and_bg": ref_ids <= {0, -1},
        "main_cam0_only_floor_and_bg": {p["id"] for p in main_cam0} <= {0, -1},
        "cam1_sees_bodies": {1, 2, 3, 4, 5}.issubset(cam1_ids),
        "note": "cam-persp 视野内只有 geom 0（floor）与背景；world 变化的 body/tri 几何只在 cam-track 可见 → 参考图像对 world 位姿变化结构性不敏感（至少部分成因），不单是采集刷新问题。",
    }
    add("structural_clue_campersp_sees_only_floor",
        visibility["perworld_cam0_only_floor_and_bg"] and visibility["main_cam0_only_floor_and_bg"]
        and visibility["cam1_sees_bodies"],
        {"perworld_cam0": ref_pairs, "reference_cam0": main_cam0, "reference_cam1_ids": sorted(cam1_ids)})
    dump("visibility.json", visibility)

    # 7) L v1 历史声明清单（引用原文，不改卡）
    impl = json.load(open(".normify/changes/2026-10-30-render-perworld-impl-s1.json", encoding="utf-8"))
    capture = json.load(open(".normify/changes/2026-10-30-render-perworld-capture.json", encoding="utf-8"))
    abandoned = json.load(open(".normify/changes/2026-11-03-render-perworld-s1-check.json", encoding="utf-8"))
    win_csv = open("docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/entity-results.csv",
                   encoding="utf-8").read()
    win_per_world = open("docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/logs/render_per_world.log",
                         encoding="utf-8").read()
    claims = {
        "impl_s1_card_status": impl["status"],
        "impl_s1_claim_per_world_correctness": [a for a in impl["acceptance"] if "逐 world 正确性" in a][:1],
        "impl_s1_claim_scope": [a for a in impl["acceptance"] if "per-world 范围" in a][:1],
        "capture_card_status": capture["status"],
        "capture_claim_determinism": [a for a in capture["acceptance"] if "确定性" in a][:1],
        "window_attempt3_render_per_world_exit0": ("render_per_world,new,0" in win_csv
                                                    and "gpu_per_world_matches_captured_reference ... ok" in win_per_world),
        "abandoned_qa_check_status": abandoned["status"],
        "abandoned_qa_check_note_excerpt": abandoned.get("note", "")[:400],
    }
    add("l1_claim_inventory_loaded",
        claims["impl_s1_card_status"] == "verified" and claims["capture_card_status"] == "verified"
        and claims["window_attempt3_render_per_world_exit0"] and bool(claims["impl_s1_claim_per_world_correctness"]),
        claims)
    dump("l1-claims.json", claims)

    # 8) 分类（按标准原文分类与前提条款）
    std = open(STANDARD, encoding="utf-8").read()
    classification = {
        "standard_path": STANDARD,
        "standard_categories_quoted": {
            "A": "A 本质-推导型：可作发布依据。",
            "B": "B 参考-前提型：仅在声明前提下有效；跨场景/跨形态即降级；不得单独作发布依据。",
            "C": "C 完整性-哈希型：只证明留痕/字节一致，不涉及数值正确性。",
        },
        "standard_premise_rule_present": "参考型判据必须写明前提" in std and "前提不成立" in std,
        "assigned_class": "B",
        "reference_premise": "参考必须具备 world 间判别力（reference-side discriminability），否则逐 world 对照是空转",
        "premise_holds": False,
        "verdict": "该参考上的逐 world 位姿对照判据降级/不适用；不得作 per-world 位姿正确性依据",
        "still_valid": [
            "C 类：参考字节/哈希绑定、两跑 bitwise 一致、legacy 未被改（完整性，不涉数值正确性）",
            "数据面事实：state.npz 逐 world 记录了不同 geom 位姿输入（不等于渲染正确性）",
            "产品侧 world 隔离断言（窗口日志 attempt3 通过）：仅窄信号，位姿变化影响产品输出；不替代参考对照",
        ],
        "requires_downgrade_or_qualification": [
            "「逐 world 位姿级对照通过（mismatched=0 / max_ulp<=4 / seg 全等）」在该参考上空转，不得作正确性/发布依据",
            "「该参考可用于后续 per-world 位姿对照」→ 不可用于位姿正确性对照；仅可作布局/哈希/回归留痕（回归价值低）",
        ],
    }
    add("classification_under_standard",
        classification["standard_premise_rule_present"] and classification["assigned_class"] == "B"
        and classification["premise_holds"] is False and "降级" in classification["verdict"],
        classification)
    dump("classification.json", classification)

    # 9) 历史不改：BASE 之后历史证据/卡零 diff（只允许本卡）
    hist_probe = run("git", "diff", "--name-only", BASE, "HEAD", "--", "fixtures", "src", "tests", "scripts", *HISTORICAL)
    add("historical_evidence_and_cards_untouched", hist_probe.stdout.strip() == b"", hist_probe.stdout.decode().splitlines())
    dump("unchanged.json", {"base": BASE, "probe_hits": hist_probe.stdout.decode().splitlines(),
                            "historical_paths": HISTORICAL})

    # 10) 最小结论句与限定说明在位（README 由本卡提供，运行时会先写 README 再跑；用文本检查）
    readme_path = os.path.join(out, "README.md")
    if os.path.exists(readme_path):
        readme = open(readme_path, encoding="utf-8").read()
        minimal = "L v1 参考无 world 间判别力，不能作为 per-world 位姿正确性的对照依据；待 P0-1 修复 + pilot 通过后新参考再验证。"
        add("minimal_conclusion_and_qualification_present",
            minimal in readme and "B 类" in readme and "空转" in readme and "P0-1" in readme,
            {"minimal_sentence_present": minimal in readme})
    else:
        add("minimal_conclusion_and_qualification_present", False, "README missing")

    # 13) P0-3 补充（Arch 请求，fix 轮）：证据链锚点映射 / 面向用户话术 / P0-1·pilot 独立建议
    chain = {
        "note": "哪些 L v1 断言直接依赖该参考的逐 world 像素对照、哪些只依赖产品侧隔离/实现面/完整性。",
        "items": [
            {"path": ".normify/changes/2026-10-30-render-perworld-impl-s1.json", "anchor": "acceptance #2「逐 world 正确性」",
             "claim": "逐 world 位级比对 rgb（mismatched=0/max_byte_diff=0）、depth（mismatched=0/max_ulp<=4）、seg 全等；world 隔离；两跑 SHA 相同",
             "depends_on_reference_pixels": True,
             "still_valid_part": "world 隔离（产品侧）、两跑 SHA（确定性）",
             "downgraded_part": "「rgb/depth/seg 逐 world 位级对照通过」在该参考上空转", "class": "B"},
            {"path": "docs/development-evidence/render-bvh/perworld-impl-s1/README.md", "anchor": "「实现机制」1–4 节",
             "claim": "adr 位移输出定位、内核索引点不变、逐 world 编排、leaf 槽位校验",
             "depends_on_reference_pixels": False,
             "still_valid_part": "实现机制与 CPU 单测/门禁事实", "downgraded_part": "无（该节不依赖参考像素）", "class": "实现面事实"},
            {"path": "docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/entity-results.csv",
             "anchor": "行 `render_per_world,new,0`；logs/render_per_world.log `gpu_per_world_matches_captured_reference ... ok`",
             "claim": "实体通过（含参考逐 world 对照 + 两跑 + 隔离）",
             "depends_on_reference_pixels": True,
             "still_valid_part": "产品侧 world 隔离与两跑确定性（日志内同用例）",
             "downgraded_part": "参考对照部分空转；不得作为 per-world 正确性依据", "class": "B"},
            {"path": "tests/render_per_world.rs", "anchor": ":378-420 参考逐 world 对照 / :422-435 两跑 SHA / :437-469 world 隔离",
             "claim": "参考对照 + 确定性 + 隔离断言",
             "depends_on_reference_pixels": True,
             "still_valid_part": ":422-435 确定性、:437-469 隔离（产品侧）",
             "downgraded_part": ":378-420 参考对照（参考无判别力）", "class": "B（部分）"},
            {"path": "docs/development-evidence/render-bvh/perworld-s1-window-material/materials/entity-table.json",
             "anchor": "render_per_world 实体条目（判据/覆盖理由）",
             "claim": "窗口内用该参考做逐 world 对照",
             "depends_on_reference_pixels": True, "still_valid_part": "材料/冻结/窗口纪律部分",
             "downgraded_part": "参考对照判据在该参考上不适用", "class": "B"},
            {"path": "docs/development-evidence/render-bvh/perworld-capture-run/receipt.json（及 manifest）",
             "anchor": "capture.json files_sha256 / manifest 逐文件 sha256+bytes",
             "claim": "参考文件存在、哈希绑定、两跑 bitwise 一致",
             "depends_on_reference_pixels": False,
             "still_valid_part": "完整性/确定性（C 类）", "downgraded_part": "不涉数值正确性", "class": "C"},
            {"path": "docs/development-evidence/render-bvh/perworld-capture-check/", "anchor": "QA 采集复核证据",
             "claim": "采集产物完整性/确定性/边界复核",
             "depends_on_reference_pixels": False,
             "still_valid_part": "C 类完整性结论", "downgraded_part": "不覆盖 world 判别力（当时未要求）", "class": "C"},
        ],
    }
    add("evidence_chain_anchor_map_recorded", len(chain["items"]) == 7
        and any(i["depends_on_reference_pixels"] for i in chain["items"])
        and any(not i["depends_on_reference_pixels"] for i in chain["items"]),
        {"items": len(chain["items"])})
    dump("evidence-chain.json", chain)

    user_sentence = ("L v1 per-world geom 位姿覆盖中，「以 nw3-hinge02040 参考做逐 world 像素对照并通过」这部分为空转："
                     "该参考三 world 输出逐位相同、无 world 间判别力，不能作为 per-world 位姿正确性的对照依据；"
                     "仍成立的是参考的确定性/哈希完整性、逐 world 位姿数据面记录，以及产品侧 world 隔离断言——"
                     "后者只说明位姿变化会影响产品输出，不等于与参考一致。"
                     "待 P0-1 修复 + pilot 通过后的新参考再验证。")
    advice = {
        "supports_refit_direction": True,
        "why": "refit_bvh 是实现正确的必要修复（调用序/上游语义核对通过）；但它只解决「render 前未刷新场景 BVH」这一根因假设，不改变视野内看不到 world 变化几何的事实。",
        "pilot_expectation": "按现 scene-a-light + cam-persp（视野内仅 floor），depth/seg 变化很可能仍为 0；即便 rgb 因灯光位姿变化而有差，§6 的 depth/seg 门槛仍不可达。",
        "pilot_must_record": ["per-world rgb/depth/seg 参考与产品侧变化像素", "seg 可见 geom id 集合", "rgb 差异来源（灯光/阴影面 vs 几何面）", "refit 是否实际执行（调用计数/日志）"],
        "recommendation": "pilot 若 non-discriminating：先核视野（seg 是否仍只有 floor）、再决定改场景/相机（新 spec + 重签）或改判别力口径（签名级）；不要只在采集脚本上继续试。",
        "not_claiming": "不替实现卡/判据下结论；refit 修复仍按 P0-1 卡与本 QC 复核结论有效（PASS）。",
    }
    add("pilot_advice_recorded", advice["supports_refit_direction"] is True
        and len(advice["pilot_must_record"]) >= 3 and "non-discriminating" in advice["recommendation"],
        {"supports_refit_direction": advice["supports_refit_direction"]})
    dump("pilot-advice.json", advice)

    if os.path.exists(readme_path):
        readme2 = open(readme_path, encoding="utf-8").read()
        add("p0_3_supplement_sections_present",
            "## 9. 证据链锚点映射" in readme2 and "## 10. 面向用户的限定话术" in readme2
            and "## 11. 对 P0-1 / pilot 的独立建议" in readme2 and "该参考三 world 输出逐位相同" in readme2,
            {"has_9_10_11": all(s in readme2 for s in ["## 9. 证据链锚点映射", "## 10. 面向用户的限定话术",
                                                       "## 11. 对 P0-1 / pilot 的独立建议"])})
    else:
        add("p0_3_supplement_sections_present", False, "README missing")

    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL", "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed,
               "fact_confirmed": True, "assigned_class": "B", "premise_holds": False,
               "checks": checks}
    with open(os.path.join(logs, "verify-l1-reference-review.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())

