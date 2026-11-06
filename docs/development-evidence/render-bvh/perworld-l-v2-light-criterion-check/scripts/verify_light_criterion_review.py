"""QA 读前预审：perworld L v2 灯判据可签草案。

复核目标 HEAD 1ef5bd5b，基线 b6f5b724。
只读：不改被复核 spec/receipt/manifest、既有证据、代码、fixtures；零 GPU。

用法（工作目录 = 仓库根）：
  python -B scripts/verify_light_criterion_review.py --out docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-check
"""
import argparse
import glob
import hashlib
import json
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

REVIEW_BASE = "1ef5bd5b"
BASE = "b6f5b724"
EVID = "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-check/"
CARD = ".normify/changes/2026-11-06-render-perworld-l-v2-light-criterion.json"
CARD_CHECK = ".normify/changes/2026-11-06-render-perworld-l-v2-light-criterion-check.json"
PREP = "docs/development-evidence/render-bvh/perworld-l-v2-light-prep/"
V1_REF = "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/"
SPEC_SHA = "1821be98953e834265e6e8b3a8e6e363f0b59306b9625ed96b6a39cb570ab9e8"
SPEC_BYTES = 16625
EXPECTED_SCOPE = {
    ".normify/changes/2026-11-06-render-perworld-l-v2-light-criterion.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion/clippy-cuda.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion/clippy.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion/fmt.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion/unit.json",
    EVID + "manifest.txt",
    EVID + "receipt.json",
    EVID + "spec.md",
}
checks = []


def run(*args):
    return subprocess.run(list(args), capture_output=True)


def read(p):
    return open(os.path.join(os.getcwd(), p), "rb").read()


def sha(b):
    return hashlib.sha256(b).hexdigest()


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

    spec = read(EVID + "spec.md")
    spec_text = spec.decode("utf-8")
    receipt_raw = read(EVID + "receipt.json")
    receipt = json.loads(receipt_raw)
    manifest_raw = read(EVID + "manifest.txt")

    # 0) 复核目标可追溯
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    anc = run("git", "merge-base", "--is-ancestor", REVIEW_BASE, "HEAD").returncode
    after = run("git", "diff", "--name-only", REVIEW_BASE, "HEAD").stdout.decode().splitlines()
    mine = [p for p in after if p == CARD_CHECK or p.startswith(CHECK)
            or p.startswith(".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-check/")]
    add("review_target_traceable_and_only_qa_commits_after", anc == 0 and after == mine,
        {"review_target": REVIEW_BASE, "current_head": head,
         "my_card_registered": os.path.exists(CARD_CHECK), "after": after})

    # 1) 变更面与零越界
    changed = run("git", "diff", "--name-only", BASE, REVIEW_BASE).stdout.decode().splitlines()
    add("scope_exact_8_paths", set(changed) == EXPECTED_SCOPE,
        {"n": len(changed), "extra": sorted(set(changed) - EXPECTED_SCOPE),
         "missing": sorted(EXPECTED_SCOPE - set(changed))})
    probe = run("git", "diff", "--name-only", BASE, REVIEW_BASE, "--", "fixtures", "src", "scripts", "tests",
                "docs/development-evidence/render-bvh/sdf-criterion-v2.2").stdout.decode().splitlines()
    add("zero_diff_fixtures_src_scripts_tests_criterion_v22", probe == [], probe)
    dump("scope.json", {"base": BASE, "head": REVIEW_BASE, "changed": changed, "zero_diff_probe": probe})

    # 2) 签名对象与机读状态
    spec_sha, spec_bytes = sha(spec), len(spec)
    mlines = manifest_raw.decode("utf-8").splitlines()
    mrows = {}
    for ln in mlines:
        if not ln.strip() or ln.startswith("#"):
            continue
        parts = ln.split()
        if len(parts) == 3:
            mrows[parts[2]] = {"sha256": parts[0], "bytes": int(parts[1])}
    sig_ok = (spec_sha == SPEC_SHA and spec_bytes == SPEC_BYTES
              and receipt["spec"]["sha256"] == SPEC_SHA and receipt["spec"]["bytes"] == SPEC_BYTES
              and mrows.get("spec.md") == {"sha256": SPEC_SHA, "bytes": SPEC_BYTES})
    add("signature_object_spec_sha_bytes_consistent", sig_ok,
        {"actual": [spec_sha, spec_bytes], "receipt": [receipt["spec"]["sha256"], receipt["spec"]["bytes"]],
         "manifest": mrows.get("spec.md")})
    status_ok = (receipt["status"] == "draft_pending_user_signature" and receipt["effective"] is False
                 and receipt["kind"] == "criterion-spec-draft"
                 and receipt["task_id"] == "2026-11-06-render-perworld-l-v2-light-criterion")
    add("receipt_machine_readable_draft_not_effective", status_ok,
        {"status": receipt["status"], "effective": receipt["effective"], "kind": receipt["kind"]})
    dump("signature-object.json", {"spec_sha256": spec_sha, "spec_bytes": spec_bytes,
                                   "receipt_status": receipt["status"], "receipt_effective": receipt["effective"],
                                   "receipt_spec": receipt["spec"], "manifest_rows": mrows})

    # 3) manifest 2/2 与编码卫生
    files = []
    for root, _, fs in os.walk(os.path.join(os.getcwd(), EVID)):
        for f in fs:
            files.append(os.path.relpath(os.path.join(root, f), EVID).replace("\\", "/"))
    rec = {}
    for rel, meta in mrows.items():
        b = read(EVID + rel)
        rec[rel] = {"ok": sha(b) == meta["sha256"] and len(b) == meta["bytes"]}
    m_ok = (len(mrows) == 2 and len(files) == 3 and all(v["ok"] for v in rec.values())
            and mlines[0].startswith("#") and "manifest.txt" not in mrows)
    add("manifest_2_entries_3_files_selfcheck", m_ok,
        {"entries": len(mrows), "files": len(files), "rows_ok": rec})
    enc_bad = []
    for rel in files:
        b = read(EVID + rel)
        e = {"bom": b[:3] == b"\xef\xbb\xbf", "cr": b.count(b"\r"), "tab": b.count(b"\t"),
             "bad_utf8": False}
        try:
            b.decode("utf-8")
        except UnicodeDecodeError:
            e["bad_utf8"] = True
        if e["bom"] or e["cr"] or e["tab"] or e["bad_utf8"]:
            enc_bad.append((rel, e))
    add("evidence_encoding_hygiene", enc_bad == [], {"checked": len(files), "bad": enc_bad})
    dump("evidence-selfcheck.json", {"manifest_entries": len(mrows), "files": files,
                                     "manifest_first_line_hash": mlines[0].startswith("#"),
                                     "rows_ok": rec, "encoding_bad": enc_bad})

    # 4) 内容自洽：节数、阈值算术、参考范围
    sections = re.findall(r"^## (\d+)\. ", spec_text, flags=re.M)
    add("spec_15_sections", [int(x) for x in sections] == list(range(1, 16)), sections)
    th_ok = (0.005 * 76800 == 384 and 76800 == 320 * 240 and 1.5 * 384 == 576
             and "≥ **384**" in spec_text and "≥ **576**" in spec_text
             and "[384, 576)" in spec_text)
    add("threshold_arithmetic_384_576_76800", th_ok,
        {"0.5pct_x_76800": 0.005 * 76800, "px_per_world": 320 * 240, "margin": 1.5 * 384})
    scope_keys = {
        "tag": "nw3-light-hinge02040" in spec_text,
        "out_root": "fixtures/render-bvh/scene-a-light/perworld/" in spec_text,
        "worlds_3": "**3**（nworld=3）" in spec_text,
        "hinge": "**0 / 20 / 40**" in spec_text,
        "camera": "cam-persp" in spec_text and "cam-track" in spec_text,
        "scene_variant": "fixtures/render-bvh/scene-a-light/scene.xml" in spec_text,
    }
    add("reference_scope_written_down", all(scope_keys.values()), scope_keys)
    dump("content-consistency.json", {"sections": sections, "thresholds_ok": th_ok, "scope_keys": scope_keys})

    # 5) recipe 行与 v1 capture.json.parameters 对齐
    v1 = json.load(open(os.path.join(V1_REF, "capture.json"), encoding="utf-8"))
    params = v1["parameters"]
    expected_forms = {
        "use_fast_math": "use_fast_math=false",
        "use_shadows": "use_shadows=true",
        "samples_per_pixel": "samples_per_pixel=1",
        "enabled_geom_groups": "enabled_geom_groups=[0,1,2,3]",
        "background_color": "background_color=[0,0,0,1]",
        "shadow_light_fraction": "shadow_light_fraction=0.3",
        "flex_render_smooth": "flex_render_smooth=true",
        "use_precomputed_rays": "use_precomputed_rays=true",
    }
    forms_ok = all(v in spec_text for v in expected_forms.values())
    values_ok = (params["use_fast_math"] is False and params["use_shadows"] is True
                 and params["samples_per_pixel"] == 1 and params["enabled_geom_groups"] == [0, 1, 2, 3]
                 and params["background_color"] == [0.0, 0.0, 0.0, 1.0]
                 and params["shadow_light_fraction"] == 0.3 and params["flex_render_smooth"] is True
                 and params["use_precomputed_rays"] is True and params["cam_active"] == [True, False])
    add("recipe_parameters_match_v1_reference", forms_ok and values_ok,
        {"forms_in_spec": expected_forms, "v1_values_match": values_ok,
         "cam_active": params["cam_active"]})
    cam = v1.get("context", {}).get("resolved", {})
    cam_ok = cam.get("cam_res") == [[320, 240]] and cam.get("total_rays") == 76800
    add("v1_reference_resolution_320x240", cam_ok, {"cam_res": cam.get("cam_res"), "total_rays": cam.get("total_rays")})
    dump("recipe-parameters.json", {"expected_forms": expected_forms, "forms_ok": forms_ok,
                                    "v1_values_ok": values_ok, "cam_res": cam.get("cam_res"),
                                    "total_rays": cam.get("total_rays")})

    # 6) 事实地基与 prep 证据逐值对齐
    prep = json.load(open(os.path.join(PREP, "logs/explicit-light-dryrun.json"), encoding="utf-8"))
    prep_keys = prep["light_pose_precheck"]["diff_keys"]
    prep_values = {}
    for d in prep_keys:
        for k in d["keys"]:
            prep_values[(d["world"], k["light"], k["key"])] = k["value"]
    fact_strings = ["10.745878447", "21.255926664", "0.222378165", "0.437999482"]
    facts_ok = all(s in spec_text for s in fact_strings) and "**0 / 0**" in spec_text
    spot_reason = ("z 旋转轴" in spec_text) or ("z-rotation axis" in spec_text)
    add("fact_base_numbers_match_prep_evidence", facts_ok and spot_reason,
        {"prep_values": {f"{k[0]}/{k[1]}/{k[2]}": v for k, v in prep_values.items()},
         "strings_in_spec": {s: (s in spec_text) for s in fact_strings},
         "spot_zero_reason_present": spot_reason})
    dump("fact-base-vs-prep.json", {"prep_values": {f"{k[0]}/{k[1]}/{k[2]}": v for k, v in prep_values.items()},
                                    "spec_contains": {s: (s in spec_text) for s in fact_strings},
                                    "spot_reason_present": spot_reason})

    # 7) 不继承 criterion v2.2
    legacy_entities = ["render_e2e", "render_kernel_rgb", "render_kernel_depth_seg", "render_kernel_flags",
                       "render_r10_multi_world", "render_skybox_reference", "render_context_lifecycle",
                       "render_mesh_bvh", "render_hfield_reference", "render_shadow_reference"]
    v22 = {
        "not_inherit_sentence": "不继承" in spec_text,
        "recipe_list_sha": "cfd1038a" in spec_text,
        "aa2_list_sha": "14893fc6" in spec_text,
        "rows_183_172": "183" in spec_text and "172" in spec_text,
        "ulp128": "|ULP|≤128" in spec_text,
        "all_10_legacy_entities_named": all(e in spec_text for e in legacy_entities),
    }
    add("criterion_v22_not_inherited_and_lists_named", all(v22.values()), v22)
    dump("v22-non-inheritance.json", v22)

    # 8) 失败即停 7 条
    sec8 = spec_text.split("## 8.", 1)[1].split("## 9.", 1)[0]
    items = re.findall(r"^(\d+)\.\s", sec8, flags=re.M)
    add("fail_stop_7_items_present", items == [str(i) for i in range(1, 8)], items)
    dump("fail-stop.json", {"section8_items": items})

    # 9) 签名 / GO 清单
    lists_ok = (spec_text.count("**S-1**") == 1 and spec_text.count("**S-2**") == 1
                and spec_text.count("**S-3**") == 1 and spec_text.count("**G-L1**") == 1
                and spec_text.count("**G-L2**") == 1
                and "authoring-only `util<60`" in spec_text
                and "基线" in spec_text and "floor" in spec_text)
    add("signature_S_and_GO_G_checklists_unambiguous", lists_ok,
        {"S1": spec_text.count("**S-1**"), "S2": spec_text.count("**S-2**"),
         "S3": spec_text.count("**S-3**"), "GL1": spec_text.count("**G-L1**"), "GL2": spec_text.count("**G-L2**"),
         "authoring_exception": "authoring-only `util<60`" in spec_text})
    dump("checklists.json", {"s1": spec_text.count("**S-1**"), "s2": spec_text.count("**S-2**"),
                             "s3": spec_text.count("**S-3**"), "gl1": spec_text.count("**G-L1**"),
                             "gl2": spec_text.count("**G-L2**")})

    # 10) 既有 perworld suite 96 条事实（归档 guards 本体）
    guards_paths = glob.glob(os.path.join("docs/development-evidence/render-bvh/perworld-impl-s1", "**",
                                          "guards-perworld-s1.json"), recursive=True)
    gp = sorted(guards_paths)[0] if guards_paths else None
    suite = {}
    if gp:
        g = json.load(open(gp, encoding="utf-8"))
        counts = g.get("counts", {})
        suite = {"path": gp.replace("\\", "/"), "counts": counts,
                 "entity_order": g.get("entity_order"),
                 "spec_says_96": "11 exe + 83 输入 + 2" in spec_text,
                 "legacy_entities_match": [e for e in g.get("entity_order", []) if e != "render_per_world"] == legacy_entities}
    suite_ok = bool(suite) and suite["counts"].get("total") == 96 and suite["counts"].get("frozen_exes") == 11 \
        and suite["counts"].get("guard_inputs") == 83 and suite["counts"].get("outline_lists") == 2 \
        and suite["spec_says_96"] and suite["legacy_entities_match"]
    add("perworld_suite_96_fact_matches_archived_guards", suite_ok, suite)
    dump("perworld-suite-fact.json", suite)

    # 11) 规格要求 vs 现有采集脚本输出（接口差，observation 数据）
    script_src = read("scripts/capture_render_perworld.py").decode("utf-8")
    v1_capture = v1
    gaps = {
        "capture_command_field_absent": "command" not in v1_capture and '"command"' not in script_src,
        "captured_at_depends_on_env": "MJWARP_CAPTURE_AT" in script_src and v1_capture.get("captured_at") == "",
        "per_world_scope_old_text": v1_capture["per_world_scope"]["does_not_cover"].startswith("per-world light poses")
        and "does_not_cover" in script_src,
        "spec_requires_command": "命令。" in spec_text or "命令" in spec_text,
        "spec_requires_capture_time": "采集时间" in spec_text,
        "spec_requires_scope_covers_light": "per_world_scope" in spec_text and "per-world 灯位姿" in spec_text,
    }
    add("capture_interface_gaps_identified",
        gaps["capture_command_field_absent"] and gaps["captured_at_depends_on_env"]
        and gaps["per_world_scope_old_text"] and gaps["spec_requires_command"]
        and gaps["spec_requires_capture_time"] and gaps["spec_requires_scope_covers_light"], gaps)
    dump("capture-interface-gaps.json", {"gaps": gaps,
                                         "v1_capture_keys": list(v1_capture.keys()),
                                         "captured_at": v1_capture.get("captured_at"),
                                         "per_world_scope": v1_capture.get("per_world_scope")})

    # 12) §2.3 形状描述 vs 既有参考实际键编码
    import numpy as np
    shapes = {}
    for f in ["rgb.npz", "depth.npz", "seg.npz", "state.npz"]:
        z = np.load(os.path.join(V1_REF, f))
        shapes[f] = {k: [list(z[k].shape), str(z[k].dtype)] for k in z.files}
    stacked_notation = "[nworld, 240, 320, 3]" in spec_text and "[nworld, 240, 320]" in spec_text
    per_world_keys = all(len(shapes[f]) == 3 for f in ["rgb.npz", "depth.npz", "seg.npz"])
    add("spec_23_shape_notation_vs_actual_encoding",
        stacked_notation and per_world_keys,
        {"spec_stacked_notation_present": stacked_notation, "actual_per_world_keys": per_world_keys,
         "actual_keys": {f: list(v) for f, v in shapes.items()}})
    dump("npz-shape-vs-spec.json", {"spec_stacked_notation_present": stacked_notation,
                                    "actual": shapes,
                                    "note": "spec §2.3 用堆叠形状记法，但既有参考按 world 存独立键（rgb_w0_cam0 等）；同节又写『同编码』"})

    # 13) 体积估算
    sizes = {f: os.path.getsize(os.path.join(V1_REF, f)) for f in
             ["rgb.npz", "depth.npz", "seg.npz", "state.npz", "capture.json"]}
    total = sum(sizes.values())
    add("volume_estimate_consistent", abs(total - 5.55e6) < 2e4,
        {"sizes": sizes, "total_bytes": total, "mb_decimal": round(total / 1e6, 4),
         "spec_says": "≈5.55 MB"})
    dump("volume.json", {"sizes": sizes, "total_bytes": total, "mb": round(total / 1e6, 4),
                         "spec_text": "≈5.55 MB/tag"})

    # 14) 被复核卡与 gates
    card = json.load(open(CARD, encoding="utf-8"))
    unit = json.load(open(os.path.join(".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion/unit.json"),
                          encoding="utf-8"))
    gates = [c.get("gate") for c in card["checks"] if c.get("kind") == "gate"]
    card_ok = (card["status"] == "verified"
               and card["revision"]["before"] == "42748ac70aa5b6cd8c2802bc9d1084857862ced5"
               and gates == ["fmt", "clippy", "clippy-cuda", "unit"]
               and unit["tests"]["passed"] == 1359 and unit["tests"]["failed"] == 0)
    add("reviewed_card_verified_and_gate_unit_1359_0", card_ok,
        {"status": card["status"], "revision_before": card["revision"]["before"], "gates": gates,
         "unit": unit["tests"]})
    dump("reviewed-card.json", {"status": card["status"], "revision_before": card["revision"]["before"],
                                "gates": gates, "unit_tests": unit["tests"]})

    passed = sum(1 for c in checks if c["ok"])
    total_checks = len(checks)
    summary = {"conclusion": "PASS" if passed == total_checks else "FAIL", "passed": passed,
               "total": total_checks, "blocking": 0 if passed == total_checks else total_checks - passed,
               "target_head": REVIEW_BASE, "checks": checks}
    with open(os.path.join(logs, "verify-criterion-review.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total_checks))
    return 0 if passed == total_checks else 1


if __name__ == "__main__":
    sys.exit(main())
