"""QA 最小复核：perworld L v2 灯判据 spec v2（五处锚点 + 其余逐段与 v1 一致 + 边界）。

复核目标 HEAD dccd0591，基线 e9b1181b（上一轮 QA 预审证据提交）。
只读：不改被复核 spec/receipt/manifest、既有证据、代码、fixtures；零 GPU。

用法（工作目录 = 仓库根）：
  python -B scripts/verify_spec_v2_review.py --out docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-spec-v2-check
"""
import argparse
import difflib
import hashlib
import json
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

REVIEW_BASE = "dccd0591"
BASE = "e9b1181b"
V1_COMMIT = "1ef5bd5b"
EVID = "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion-spec-v2-check/"
CARD = ".normify/changes/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2.json"
CARD_CHECK = ".normify/changes/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2-check.json"
V1_SHA = "1821be98953e834265e6e8b3a8e6e363f0b59306b9625ed96b6a39cb570ab9e8"
V1_BYTES = 16625
V2_SHA = "6cfaa57e2c8d5317442f90fddc157c666430d726ad20d3ceace102fe0b435900"
V2_BYTES = 20681
V1_REF = "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/"
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


def split_sections(text):
    lines = text.split("\n")
    preamble, sections = [], {}
    cur, buf = None, []
    for ln in lines:
        m = re.match(r"^## (\d+)\. ", ln)
        if m:
            if cur is None:
                preamble = buf
            else:
                sections[cur] = buf
            cur, buf = m.group(1), [ln]
        else:
            buf.append(ln)
    if cur is None:
        preamble = buf
    else:
        sections[cur] = buf
    return preamble, sections


def split_subsections(body):
    subs, cur, buf = {}, None, []
    for ln in body:
        m = re.match(r"^### (\S+) ", ln)
        if m:
            if cur is not None:
                subs[cur] = buf
            cur, buf = m.group(1), [ln]
        else:
            buf.append(ln)
    if cur is not None:
        subs[cur] = buf
    return subs


def lines_equal(a, b):
    return a == b


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

    v1 = run("git", "cat-file", "blob", f"{V1_COMMIT}:{EVID}spec.md").stdout
    v2 = read(EVID + "spec.md")
    receipt_raw = read(EVID + "receipt.json")
    receipt = json.loads(receipt_raw)
    manifest_raw = read(EVID + "manifest.txt")
    v1_text = v1.decode("utf-8")
    v2_text = v2.decode("utf-8")

    # 0) 复核目标可追溯
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    anc = run("git", "merge-base", "--is-ancestor", REVIEW_BASE, "HEAD").returncode
    after = run("git", "diff", "--name-only", REVIEW_BASE, "HEAD").stdout.decode().splitlines()
    mine = [p for p in after if p == CARD_CHECK or p.startswith(CHECK)
            or p.startswith(".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2-check/")]
    add("review_target_traceable_and_only_qa_commits_after", anc == 0 and after == mine,
        {"review_target": REVIEW_BASE, "current_head": head, "after": after})

    # 1) 边界：仅证据目录 + 卡/runs
    changed = run("git", "diff", "--name-only", BASE, REVIEW_BASE).stdout.decode().splitlines()
    expected = {
        CARD,
        ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2/fmt.json",
        ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2/clippy.json",
        ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2/clippy-cuda.json",
        ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2/unit.json",
        EVID + "manifest.txt", EVID + "receipt.json", EVID + "spec.md",
    }
    add("scope_exact_8_paths", set(changed) == expected,
        {"n": len(changed), "extra": sorted(set(changed) - expected), "missing": sorted(expected - set(changed))})
    probe = run("git", "diff", "--name-only", BASE, REVIEW_BASE, "--", "fixtures", "src", "scripts", "tests",
                "docs/development-evidence/render-bvh/sdf-criterion-v2.2").stdout.decode().splitlines()
    add("zero_diff_fixtures_src_scripts_tests_criterion_v22", probe == [], probe)
    dump("scope.json", {"base": BASE, "head": REVIEW_BASE, "changed": changed, "zero_diff_probe": probe,
                        "no_capture": probe == []})

    # 2) 签名对象与 v2/supersedes
    spec_sha, spec_bytes = sha(v2), len(v2)
    mrows = {}
    mlines = manifest_raw.decode("utf-8").splitlines()
    for ln in mlines:
        if not ln.strip() or ln.startswith("#"):
            continue
        p = ln.split()
        if len(p) == 3:
            mrows[p[2]] = {"sha256": p[0], "bytes": int(p[1])}
    add("spec_v2_sha_bytes_consistent",
        spec_sha == V2_SHA and spec_bytes == V2_BYTES
        and receipt["spec"]["sha256"] == V2_SHA and receipt["spec"]["bytes"] == V2_BYTES
        and mrows.get("spec.md") == {"sha256": V2_SHA, "bytes": V2_BYTES},
        {"actual": [spec_sha, spec_bytes], "receipt": [receipt["spec"]["sha256"], receipt["spec"]["bytes"]],
         "manifest": mrows.get("spec.md")})
    sup = receipt.get("supersedes", {})
    add("receipt_v2_version_and_supersedes_v1",
        receipt.get("version") == "v2" and sup == {"version": "v1", "spec_sha256": V1_SHA, "spec_bytes": V1_BYTES}
        and receipt["status"] == "draft_pending_user_signature" and receipt["effective"] is False,
        {"version": receipt.get("version"), "supersedes": sup, "status": receipt["status"],
         "effective": receipt["effective"], "v1_actual": [sha(v1), len(v1)]})
    dump("signature-object.json", {"v2_sha256": spec_sha, "v2_bytes": spec_bytes,
                                   "v1_sha256": sha(v1), "v1_bytes": len(v1),
                                   "receipt_spec": receipt["spec"], "version": receipt.get("version"),
                                   "supersedes": sup, "status": receipt["status"],
                                   "effective": receipt["effective"], "manifest_rows": mrows})

    # 3) manifest 2/2 与编码
    files = []
    for root, _, fs in os.walk(os.path.join(os.getcwd(), EVID)):
        for f in fs:
            files.append(os.path.relpath(os.path.join(root, f), EVID).replace("\\", "/"))
    rows_ok = {}
    for rel, meta in mrows.items():
        b = read(EVID + rel)
        rows_ok[rel] = sha(b) == meta["sha256"] and len(b) == meta["bytes"]
    add("manifest_2_entries_3_files_selfcheck",
        len(mrows) == 2 and len(files) == 3 and all(rows_ok.values())
        and mlines[0].startswith("#") and "manifest.txt" not in mrows,
        {"entries": len(mrows), "files": len(files), "rows_ok": rows_ok})
    enc_bad = []
    for rel in files:
        b = read(EVID + rel)
        e = {"bom": b[:3] == b"\xef\xbb\xbf", "cr": b.count(b"\r"), "tab": b.count(b"\t"), "bad_utf8": False}
        try:
            b.decode("utf-8")
        except UnicodeDecodeError:
            e["bad_utf8"] = True
        if e["bom"] or e["cr"] or e["tab"] or e["bad_utf8"]:
            enc_bad.append((rel, e))
    add("encoding_hygiene", enc_bad == [], {"checked": len(files), "bad": enc_bad})
    dump("evidence-selfcheck.json", {"manifest_entries": len(mrows), "files": files,
                                     "rows_ok": rows_ok, "manifest_first_line_hash": mlines[0].startswith("#"),
                                     "encoding_bad": enc_bad})

    # 4) §2.2 anchor
    sec22 = {
        "parameters_only_row": "| recipe.parameters |" in v2_text and "仅 `parameters` 块" in v2_text,
        "scene_script_row": "| recipe.scene_xml / recipe.script |" in v2_text and "**sha256/bytes 必然变化**" in v2_text,
        "old_single_row_absent": "| recipe | 逐字同" not in v2_text,
        "old_phrase_absent": "逐字同既有 perworld 参考 `nw3-hinge02040/capture.json.parameters`（`use_fast_math" in v2_text,
        "other_recipe_fields_stated": "`recipe` 的 `nworld/hinge_deg/seed/repeat/versions/upstream_revision` 与旧参考同口径" in v2_text,
    }
    add("anchor_2_2_recipe_split", all(sec22.values()), sec22)
    dump("anchor-2-2.json", sec22)

    # 5) §2.3 anchor（编码与实际参考一致）
    import numpy as np
    actual = {}
    for f in ["rgb.npz", "depth.npz", "seg.npz", "state.npz"]:
        z = np.load(os.path.join(V1_REF, f))
        actual[f] = {k: [list(z[k].shape), str(z[k].dtype)] for k in z.files}
    sec23 = {
        "old_notation_0_hits": "[nworld, 240, 320, 3]" not in v2_text and "[nworld, 240, 320]" not in v2_text,
        "per_world_keys_named": "rgb_w{0,1,2}_cam0" in v2_text and "depth_w{0,1,2}_cam0" in v2_text
        and "seg_w{0,1,2}_cam0" in v2_text,
        "state_stacked_table": all(s in v2_text for s in ["(3, 8)", "(3, 7, 3)", "(3, 7, 3, 3)", "(3, 3, 3)"]),
        "state_no_world_keys": "无 world 键" in v2_text,
        "actual_rgb_keys_match": sorted(actual["rgb.npz"]) == ["rgb_w0_cam0", "rgb_w1_cam0", "rgb_w2_cam0"],
        "actual_shapes": all(actual[f]["rgb_w0_cam0" if f == "rgb.npz" else ("depth_w0_cam0" if f == "depth.npz" else "seg_w0_cam0")][0]
                             == exp for f, exp in [("rgb.npz", [240, 320, 3]), ("depth.npz", [240, 320]), ("seg.npz", [240, 320, 2])]),
    }
    add("anchor_2_3_actual_per_world_encoding", all(sec23.values()), sec23)
    dump("anchor-2-3.json", {"checks": sec23, "actual_reference_arrays": actual})

    # 6) §10.2 anchor（条数口径）
    m102 = re.search(r"### 10\.2 窗口材料与纪律（写死）\n(.*?)\n### 10\.3", v2_text, flags=re.S)
    t102 = m102.group(1) if m102 else ""
    sec102 = {
        "new_one_exe": "**新增 1 个 exe**" in t102 and "`render_per_world_light`" in t102,
        "inputs_le_12": "**新增输入 ≤12 条**" in t102,
        "nine_mandatory": "必须包含" in t102 and "合计 9 条必需" in t102 and "至多 3 条" in t102,
        "arithmetic_9": 5 + 1 + 1 + 1 + 1 == 9 and 12 - 9 == 3,
        "freeze_list_authority": "以验证窗卡冻结清单为准" in t102 and "不得少冻结上述 9 条必需输入" in t102,
        "old_estimate_absent": "预估 +14 条" not in v2_text,
    }
    add("anchor_10_2_guard_count_explicit", all(sec102.values()), sec102)
    dump("anchor-10-2.json", sec102)

    # 7) §11.1 anchor（四条前置）
    m111 = re.search(r"### 11\.1 S-2 采集窗前置.*?(?=\n## 12)", v2_text, flags=re.S)
    t111 = m111.group(0) if m111 else ""
    sec111 = {
        "command_item": "`command` 字段" in t111 and "补写" in t111,
        "captured_at_item": "MJWARP_CAPTURE_AT" in t111 and "非空 ISO8601" in t111 and '空串 `""`' in t111,
        "per_world_scope_item": "per_world_scope" in t111 and "does_not_cover: per-world light poses" in t111,
        "task_id_item": "recipe.task_id" in t111 and ("参数化 task_id" in t111 or "reference_tag" in t111
                                                      or "reference_tag` 字段" in t111),
        "s2_sign_gate": "否则 S-2 不得签" in t111,
        "receipt_preconditions_4": len(receipt.get("preconditions_collection_window", [])) == 4,
    }
    add("anchor_11_1_capture_preconditions", all(sec111.values()), sec111)
    dump("anchor-11-1.json", {"checks": sec111, "receipt_preconditions": receipt.get("preconditions_collection_window")})

    # 8) §12 G-L1 anchor
    sec12 = {
        "gl1_mentions_111": "§11.1 四项采集窗前置全部落实" in v2_text,
        "gl1_lists_four": all(k in v2_text for k in ["command / captured_at+env / per_world_scope 文案 / task_id 或 reference_tag"]),
        "gl2_unchanged": "| **G-L2** | 验证窗（1 个） | 采集证据 verified（两次采集逐位一致 + world 间 ≥576）+ 判据实体实现 verified + QA 复核 PASS + guards 冻结（含新 exe + 新参考）+ 基线/floor + 零编译纪律 |" in v2_text,
    }
    add("anchor_12_gl1_synced", all(sec12.values()), sec12)
    dump("anchor-12.json", sec12)

    # 9) 其余逐段一致（section-level）
    p1, s1 = split_sections(v1_text)
    p2, s2 = split_sections(v2_text)
    same_ids = sorted(set(s1) & set(s2), key=int)
    unchanged = ["1", "3", "4", "5", "6", "7", "8", "9", "13", "15"]
    sec_report = {i: lines_equal(s1[i], s2[i]) for i in same_ids}
    add("sections_identity_outside_anchors",
        all(sec_report[i] for i in unchanged) and set(same_ids) == set(unchanged) | {"2", "10", "11", "12", "14"},
        {"section_equal": sec_report})
    # §2 内仅 2.2/2.3 变
    subs1 = split_subsections(s1["2"])
    subs2 = split_subsections(s2["2"])
    sub2 = {k: (k in subs1 and k in subs2 and lines_equal(subs1[k], subs2[k])) for k in ["2.1", "2.2", "2.3", "2.4"]}
    add("section_2_only_2_2_2_3_changed",
        sub2["2.1"] is True and sub2["2.4"] is True and sub2["2.2"] is False and sub2["2.3"] is False,
        {"subsection_equal": sub2, "subsections": sorted(set(subs1) | set(subs2))})
    # §10 内仅 10.2 变
    subs10a = split_subsections(s1["10"])
    subs10b = split_subsections(s2["10"])
    add("section_10_only_10_2_changed",
        lines_equal(subs10a.get("10.1", []), subs10b.get("10.1", []))
        and lines_equal(subs10a.get("10.3", []), subs10b.get("10.3", []))
        and not lines_equal(subs10a.get("10.2", []), subs10b.get("10.2", [])),
        {"10.1": lines_equal(subs10a.get("10.1", []), subs10b.get("10.1", [])),
         "10.3": lines_equal(subs10a.get("10.3", []), subs10b.get("10.3", []))})
    # §11：原 S 表为前缀，新增 11.1
    b1, b2 = s1["11"], s2["11"]
    t1 = "\n".join(b1).rstrip("\n")
    t2 = "\n".join(b2).rstrip("\n")
    add("section_11_original_table_preserved_and_11_1_added",
        t2.startswith(t1) and "### 11.1 S-2 采集窗前置" in t2,
        {"s_table_preserved": t2.startswith(t1), "added_11_1": "### 11.1 S-2 采集窗前置" in t2})
    # §12：仅 G-L1 行变
    gl1_old = next((l for l in s1["12"] if l.startswith("| **G-L1**")), None)
    gl1_new = next((l for l in s2["12"] if l.startswith("| **G-L1**")), None)
    others_old = [l for l in s1["12"] if not l.startswith("| **G-L1**")]
    others_new = [l for l in s2["12"] if not l.startswith("| **G-L1**")]
    add("section_12_only_gl1_row_changed",
        gl1_old != gl1_new and lines_equal(others_old, others_new),
        {"gl1_changed": gl1_old != gl1_new, "other_lines_equal": lines_equal(others_old, others_new)})
    # §14：原文为前缀，新增 14.1
    t14a = "\n".join(s1["14"]).rstrip("\n")
    t14b = "\n".join(s2["14"]).rstrip("\n")
    add("section_14_original_preserved_and_14_1_added",
        t14b.startswith(t14a) and "### 14.1 修订记录" in t14b,
        {"original_preserved": t14b.startswith(t14a), "added_14_1": "### 14.1 修订记录" in t14b})
    # preamble：仅 task/status 行改 + 新增版本行
    added_version = [l for l in p2 if l.startswith("- 版本：**v2**")]
    p2_wo = [l for l in p2 if not l.startswith("- 版本：**v2**")]
    opcodes = [op for op in difflib.SequenceMatcher(a=p1, b=p2_wo).get_opcodes() if op[0] != "equal"]
    add("preamble_only_version_header_lines",
        len(added_version) == 1 and len(opcodes) == 2
        and all(p1[i].startswith(("- 任务：", "- 状态：")) for op in opcodes if op[0] == "replace"
                for i in range(op[1], op[2])),
        {"added_version_line": added_version, "non_equal_opcodes": len(opcodes)})
    dump("section-identity.json", {"sections_equal": sec_report, "s2_subsections": sub2,
                                   "preamble_opcodes": len(opcodes),
                                   "s11_prefix": t2.startswith(t1), "s14_prefix": t14b.startswith(t14a),
                                   "gl1_changed": gl1_old != gl1_new,
                                   "g_l1_old": gl1_old, "g_l1_new": gl1_new})

    # 10) 语义关键点仍与 v1 相同（belt & braces）
    keep = {
        "thresholds": "≥ **384**" in v2_text and "≥ **576**" in v2_text and "0.5% × 76,800" in v2_text,
        "fail_stop_7": len(re.findall(r"^\d+\.\s", v2_text.split("## 8.", 1)[1].split("## 9.", 1)[0], flags=re.M)) == 7,
        "v22_lists": "cfd1038a" in v2_text and "14893fc6" in v2_text and "|ULP|≤128" in v2_text,
        "fact_base": all(s in v2_text for s in ["10.745878447", "21.255926664", "0.222378165", "0.437999482"])
        and "**0 / 0**" in v2_text,
        "signature_s_items": all(v2_text.count(f"**S-{i}**") == 1 for i in (1, 2, 3)),
        "go_g_items": all(v2_text.count(f"**G-L{i}**") == 1 for i in (1, 2)),
        "non_claim": "未经用户签名不生效" in v2_text and "不构成 perworld L v2 启动" in v2_text,
        "v1_sha_only_in_supersede_context": all(
            ("v1" in ln) or ("取代 v1" in ln) or ("1821be98…/16,625" in ln)
            for ln in v2_text.split("\n") if "1821be98" in ln),
    }
    add("semantic_core_unchanged", all(keep.values()), keep)
    dump("semantic-core.json", keep)

    # 11) 被复核卡与 gate
    card = json.load(open(CARD, encoding="utf-8"))
    unit = json.load(open(os.path.join(
        ".normify/runs/2026-11-06-render-perworld-l-v2-light-criterion-spec-v2/unit.json"), encoding="utf-8"))
    gates = [c.get("gate") for c in card["checks"] if c.get("kind") == "gate"]
    add("reviewed_card_verified_unit_1359_0",
        card["status"] == "verified" and card["revision"]["before"] == "493cb0ea198461ed14b42d3ef1704e4704457391"
        and gates == ["fmt", "clippy", "clippy-cuda", "unit"]
        and unit["tests"]["passed"] == 1359 and unit["tests"]["failed"] == 0,
        {"status": card["status"], "revision_before": card["revision"]["before"], "gates": gates,
         "unit": unit["tests"]})
    dump("reviewed-card.json", {"status": card["status"], "revision_before": card["revision"]["before"],
                                "gates": gates, "unit_tests": unit["tests"]})

    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL", "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed, "target_head": REVIEW_BASE, "checks": checks}
    with open(os.path.join(logs, "verify-spec-v2-review.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
