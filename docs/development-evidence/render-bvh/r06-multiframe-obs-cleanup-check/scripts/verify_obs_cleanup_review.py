"""QA 只读复核：R06 多帧观察项治理（O2 + O6）。

复核目标 HEAD d1653111，基线 1c5ea060（上一次 QA 收口复核）。
只读：不写被复核目录、代码、fixtures、guards；只在 --out 指定目录写证据。

用法（工作目录 = 仓库根）：
  python scripts/verify_obs_cleanup_review.py --out docs/development-evidence/render-bvh/r06-multiframe-obs-cleanup-check
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys

BASE = "1c5ea060"
HEAD = "d1653111"
GUARDS = "scripts/validate/render-bvh/guards-r06-multiframe.json"
GUARDS_RAW_SHA = "64bfecdfd179c4b7160c4f853660671ec9fb570b4d6c7702aa6c35c2c3cb352d"
GUARDS_RAW_BYTES = 11899
EVID = "docs/development-evidence/render-bvh/"
ENTITY = EVID + "r06-flex-multiframe-entity/"
FIX = EVID + "r06-flex-multiframe-entity-fix/"
CLEANUP = EVID + "r06-multiframe-obs-cleanup/"
CHECK = EVID + "r06-multiframe-obs-cleanup-check/"
FORBIDDEN = ["上游等价", "多帧验收通过", "动画一致性已覆盖"]
FORBIDDEN_PATHS = [ENTITY, FIX]
# O1/O3/O4/O5 未动：guards eol backlog 卡
BACKLOG = ".normify/changes/2026-11-05-render-r06-multiframe-guards-eol-backlog.json"

checks = []


def run(*args):
    p = subprocess.run(args, capture_output=True)
    return p.returncode, p.stdout, p.stderr


def git_bytes(rev, path):
    rc, out, err = run("git", "show", f"{rev}:{path}")
    assert rc == 0, (rev, path, err[:200])
    return out


def sha(b):
    return hashlib.sha256(b).hexdigest()


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + cid + " :: " + str(detail)[:300])


def read(p):
    return open(os.path.join(os.getcwd(), p), "rb").read()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=CHECK)
    ap.add_argument("--print-json", action="store_true")
    args = ap.parse_args()
    out = args.out
    arts = os.path.join(out, "artifacts")
    logs = os.path.join(out, "logs")
    os.makedirs(arts, exist_ok=True)
    os.makedirs(logs, exist_ok=True)

    # 0) 复核目标可追溯：d1653111 存在且其后只有本卡登记/证据提交
    rc, head_out, _ = run("git", "rev-parse", "HEAD")
    head = head_out.decode().strip()
    rc, anc, _ = run("git", "merge-base", "--is-ancestor", HEAD, "HEAD")
    rc2, after_out, _ = run("git", "diff", "--name-only", HEAD, "HEAD")
    after = after_out.decode().splitlines()
    mine = [p for p in after if p == ".normify/changes/2026-11-06-render-r06-multiframe-obs-cleanup-check.json" or p.startswith(CHECK)]
    add("review_target_traceable_and_only_qa_commits_after", rc == 0 and after == mine,
        {"review_target": HEAD, "current_head": head, "commits_after_target_only_mine": after == mine, "after_paths": after})

    # 1) 范围：恰 11 文件，且无关面零 diff
    rc, changed_out, _ = run("git", "diff", "--name-only", BASE, HEAD)
    changed = changed_out.decode().splitlines()
    expected = [
        ".normify/changes/2026-11-06-render-r06-multiframe-obs-cleanup.json",
        FIX + "README.md",
        FIX + "manifest.txt",
        FIX + "receipt.json",
        ENTITY + "manifest.txt",
        ENTITY + "receipt.json",
        CLEANUP + "README.md",
        CLEANUP + "artifacts/changed-files.json",
        CLEANUP + "logs/manifest-recompute.log",
        CLEANUP + "manifest.txt",
        CLEANUP + "receipt.json",
    ]
    add("scope_exact_11_files", changed == expected, changed)
    rc, probe_out, _ = run(
        "git", "diff", "--name-only", BASE, HEAD, "--",
        "tests", "fixtures", "scripts", "src", ".gitattributes",
        EVID + "r06-flex-multiframe-criterion/",
        EVID + "r06-flex-multiframe-verify-g3/",
        EVID + "r06-flex-multiframe-capture-g2/",
    )
    probe = probe_out.decode().splitlines()
    add("zero_diff_tests_fixtures_scripts_src_gitattributes_criterion_g3_g2", probe == [], probe)
    json.dump({"head": head, "baseline": BASE, "changed": changed, "zero_diff_probe": probe},
              open(os.path.join(arts, "scope.json"), "w", encoding="utf-8", newline="\n"),
              ensure_ascii=False, indent=1)

    # 2) guards 冻结字节
    guards_raw = read(GUARDS)
    g_head_blob = git_bytes(HEAD, GUARDS)
    g_base_blob = git_bytes(BASE, GUARDS)
    rc, gdiff, _ = run("git", "diff", "--name-only", BASE, HEAD, "--", GUARDS)
    rc, gstat, _ = run("git", "status", "--porcelain", "--", GUARDS)
    guards_ok = (
        sha(guards_raw) == GUARDS_RAW_SHA
        and len(guards_raw) == GUARDS_RAW_BYTES
        and sha(g_head_blob) == sha(g_base_blob)
        and gdiff.strip() == b""
        and gstat.strip() == b""
    )
    add("guards_frozen_bytes_unchanged", guards_ok, {
        "raw_sha": sha(guards_raw), "raw_bytes": len(guards_raw),
        "base_blob_sha": sha(g_base_blob), "head_blob_sha": sha(g_head_blob),
        "git_diff_empty": gdiff.strip() == b"", "status_empty": gstat.strip() == b"",
    })
    json.dump({"raw_sha_before_and_after": sha(guards_raw), "raw_bytes": len(guards_raw),
               "base_blob_sha": sha(g_base_blob), "head_blob_sha": sha(g_head_blob),
               "base_blob_bytes": len(g_base_blob), "head_blob_bytes": len(g_head_blob),
               "git_diff_empty": gdiff.strip() == b"", "status_empty": gstat.strip() == b""},
              open(os.path.join(arts, "guards.json"), "w", encoding="utf-8", newline="\n"),
              ensure_ascii=False, indent=1)

    # 3) changed-files.json 前后 sha/bytes 独立复算
    cf = json.load(open(os.path.join(CLEANUP, "artifacts/changed-files.json"), encoding="utf-8"))
    before_rev = cf["head_before"]
    cf_rows = []
    for item in cf["changed_files"]:
        path = item["path"]
        before = git_bytes(before_rev, path)
        after = read(path)
        row = {
            "path": path,
            "before_match": sha(before) == item["sha256_before"] and len(before) == item["bytes_before"],
            "after_match": sha(after) == item["sha256_after"] and len(after) == item["bytes_after"],
            "recomputed_before": [sha(before), len(before)],
            "recomputed_after": [sha(after), len(after)],
            "recorded_before": [item["sha256_before"], item["bytes_before"]],
            "recorded_after": [item["sha256_after"], item["bytes_after"]],
        }
        cf_rows.append(row)
    ok = all(r["before_match"] and r["after_match"] for r in cf_rows)
    add("changed_files_records_recomputed", ok, [r["path"].split("/")[-1] + ":" + str(r["before_match"] and r["after_match"]) for r in cf_rows])
    guarded = cf["guards"]
    add("changed_files_guards_segment_matches",
        guarded["raw_sha256_at_task_start"] == GUARDS_RAW_SHA and guarded["raw_sha256_after_cleanup"] == GUARDS_RAW_SHA
        and guarded["raw_bytes_at_task_start"] == GUARDS_RAW_BYTES and guarded["raw_bytes_after_cleanup"] == GUARDS_RAW_BYTES
        and guarded["unchanged"] is True,
        guarded)
    json.dump({"before_rev": before_rev, "rows": cf_rows}, open(os.path.join(arts, "changed-files-check.json"), "w", encoding="utf-8", newline="\n"), ensure_ascii=False, indent=1)

    # 4) manifest 独立重算（实体 8、小修 6、obs-cleanup 4）
    manifests = {}
    for d, n in [(ENTITY, 8), (FIX, 6), (CLEANUP, 4)]:
        p = os.path.join(d, "manifest.txt")
        raw = read(p)
        text = raw.decode("utf-8")
        lines = text.split("\n")
        rows, bad = [], []
        for ln in lines:
            if not ln.strip() or ln.startswith("#"):
                continue
            parts = ln.split()
            if len(parts) != 3:
                bad.append(("format", ln))
                continue
            rows.append(parts)
        listed = set()
        for h, size, rel in rows:
            fp = os.path.join(d, rel)
            b = open(fp, "rb").read()
            listed.add(rel)
            if sha(b) != h or len(b) != int(size):
                bad.append((rel, sha(b), len(b), h, size))
        manifests[d] = {
            "entries": len(rows), "expected": n,
            "first_line_hash": lines[0].startswith("#"),
            "self_listed": "manifest.txt" in listed,
            "bad": bad,
            "encoding": {"bom": raw[:3] == b"\xef\xbb\xbf", "cr": raw.count(b"\r"), "tab": raw.count(b"\t")},
            "files": sorted(listed),
        }
    ok = (
        manifests[ENTITY]["entries"] == 8 and manifests[FIX]["entries"] == 6 and manifests[CLEANUP]["entries"] == 4
        and not manifests[ENTITY]["bad"] and not manifests[FIX]["bad"] and not manifests[CLEANUP]["bad"]
        and all(manifests[d]["first_line_hash"] and not manifests[d]["self_listed"] for d in manifests)
    )
    add("manifests_recomputed_8_6_4", ok, {d.split("/")[-2]: m["entries"] for d, m in manifests.items()})
    json.dump(manifests, open(os.path.join(arts, "manifests.json"), "w", encoding="utf-8", newline="\n"), ensure_ascii=False, indent=1)

    # 5) 禁词/组合词扫描（实体 + 小修目录）
    import re
    hits, scanned, wide = {}, [], {}
    for d in FORBIDDEN_PATHS:
        for root, _, files in os.walk(os.path.join(os.getcwd(), d)):
            for f in files:
                if not f.endswith((".md", ".json", ".txt")):
                    continue
                path = os.path.join(root, f)
                t = open(path, encoding="utf-8", errors="replace").read()
                rel = os.path.relpath(path, os.getcwd()).replace("\\", "/")
                scanned.append(rel)
                for term in FORBIDDEN:
                    if term in t:
                        hits.setdefault(term, []).append(rel)
                for m in re.finditer(r"上游[^\u3002\uff1b;]{0,1}等价|等价[^\u3002\uff1b;]{0,1}上游", t):
                    hits.setdefault("adjacent_upstream_equivalence_le1", []).append([rel, m.group(0)])
                for m in re.finditer(r"上游[^\u3002\uff1b;]{0,12}等价|等价[^\u3002\uff1b;]{0,12}上游", t):
                    wide.setdefault("upstream_to_equivalence_within_12", []).append([rel, m.group(0)])
    add("forbidden_terms_zero_in_entity_and_fix", hits == {}, {"hits": hits, "scanned": len(scanned)})
    # 宽窗（≤12 字）组合在改动前后一致，属否定式既存措辞（不阻断）
    wide_base = {}
    for d in FORBIDDEN_PATHS:
        rc, ls, _ = run("git", "ls-tree", "-r", "--name-only", BASE, d)
        for rel in ls.decode().splitlines():
            if not rel.endswith((".md", ".json", ".txt")):
                continue
            t = git_bytes(BASE, rel).decode("utf-8", "replace")
            for m in re.finditer(r"上游[^\u3002\uff1b;]{0,12}等价|等价[^\u3002\uff1b;]{0,12}上游", t):
                wide_base.setdefault("upstream_to_equivalence_within_12", []).append([rel, m.group(0)])
    key = "upstream_to_equivalence_within_12"
    base_set = {json.dumps(x, ensure_ascii=False) for x in wide_base.get(key, [])}
    head_set = {json.dumps(x, ensure_ascii=False) for x in wide.get(key, [])}
    added = head_set - base_set
    removed = base_set - head_set
    removed_ok = all(json.loads(x)[0] == ENTITY + "receipt.json" for x in removed)
    add("wide_window_combo_no_new_and_removal_only_o6_target", added == set() and removed_ok,
        {"added": sorted(added), "removed": sorted(removed), "removal_only_in_o6_target": removed_ok})
    json.dump({"tests": {"exact_terms": FORBIDDEN, "exact_hits": hits, "adjacent_le1": hits.get("adjacent_upstream_equivalence_le1", []),
                         "wide_window_head": wide, "wide_window_base": wide_base},
               "scanned": scanned},
              open(os.path.join(arts, "forbidden-terms.json"), "w", encoding="utf-8", newline="\n"),
              ensure_ascii=False, indent=1)


    # 6) O2 路径解析交叉核对
    fix_receipt = json.load(open(os.path.join(FIX, "receipt.json"), encoding="utf-8"))
    ent_receipt = json.load(open(os.path.join(ENTITY, "receipt.json"), encoding="utf-8"))

    def resolve(obj, dotted):
        cur = obj
        for k in dotted.split("."):
            if not isinstance(cur, dict) or k not in cur:
                return {"resolved": False, "missing_at": k,
                        "available": sorted(cur.keys()) if isinstance(cur, dict) else None}
            cur = cur[k]
        return {"resolved": True, "value": cur}

    o2 = {
        "fix_top_level_option_a": "option_a" in fix_receipt and "criteria" not in fix_receipt,
        "fix_option_a_chief_ruling": resolve(fix_receipt, "option_a.chief_ruling"),
        "fix_option_a_chief_ruling_at": resolve(fix_receipt, "option_a.chief_ruling_at"),
        "fix_option_a_wording": resolve(fix_receipt, "option_a.wording"),
        "fix_option_a_ruling_trace": fix_receipt["option_a"]["ruling_trace"],
        "entity_criteria_option_a_chief_ruling": resolve(ent_receipt, "criteria.option_a.chief_ruling"),
        "entity_has_top_level_option_a": "option_a" in ent_receipt,
        "fix_readme_section_4_quote": [l.strip() for l in open(os.path.join(FIX, "README.md"), encoding="utf-8").read().split("\n") if "receipt.json.option_a.chief_ruling" in l],
    }
    add("o2_container_level_moved_to_top_level_option_a",
        o2["fix_top_level_option_a"] and o2["fix_option_a_chief_ruling"]["resolved"] is False
        and o2["fix_option_a_chief_ruling_at"]["resolved"] and o2["entity_criteria_option_a_chief_ruling"]["resolved"],
        {"fix_top_level_option_a": o2["fix_top_level_option_a"],
         "fix_option_a_chief_ruling_missing": o2["fix_option_a_chief_ruling"].get("missing_at"),
         "fix_option_a_chief_ruling_at": o2["fix_option_a_chief_ruling_at"]["resolved"],
         "entity_criteria_option_a_chief_ruling": o2["entity_criteria_option_a_chief_ruling"]["resolved"]})
    json.dump(o2, open(os.path.join(arts, "path-resolution.json"), "w", encoding="utf-8", newline="\n"), ensure_ascii=False, indent=1)

    # 7) O6 前后句与四项否定语义
    old_receipt = json.loads(git_bytes(before_rev, ENTITY + "receipt.json").decode("utf-8"))
    new_stmt = ent_receipt["claims"]["statement"]
    old_stmt = old_receipt["claims"]["statement"]
    semantics = {
        "no_family_pass_claim": ("R06 整族通过" in new_stmt),
        "no_animation_generality_claim": ("动画通用性" in new_stmt),
        "no_upstream_consistency_claim": ("上游实现的一致性结论" in new_stmt),
        "keeps_validation_window_note": ("验证窗" in new_stmt),
        "acceptance_conclusion_unchanged": ent_receipt["claims"]["acceptance_conclusion"] == old_receipt["claims"]["acceptance_conclusion"],
        "old_statement": old_stmt, "new_statement": new_stmt,
    }
    add("o6_semantics_preserved", all(v for k, v in semantics.items() if isinstance(v, bool)), semantics)
    with open(os.path.join(arts, "o2-o6-lines.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write("# O2 before/after (r06-flex-multiframe-entity-fix)\n")
        f.write("README.md §4 before: 落地位置：...；`receipt.json.criteria.option_a.chief_ruling`。\n")
        f.write("README.md §4 after : 落地位置：...；`receipt.json.option_a.chief_ruling`（修复卡 receipt 为顶层 `option_a`）。\n")
        f.write("receipt.json option_a.ruling_trace before: \"receipt.json.criteria.option_a.chief_ruling\"\n")
        f.write("receipt.json option_a.ruling_trace after : \"receipt.json.option_a.chief_ruling\"\n")
        f.write("# O6 before/after (r06-flex-multiframe-entity receipt.json claims.statement)\n")
        f.write("before: " + old_stmt + "\n")
        f.write("after : " + new_stmt + "\n")

    # 8) 编码卫生：obs-cleanup 证据 + 5 个改动证据文件
    enc_bad, enc_checked = [], []
    targets = []
    for root, _, files in os.walk(os.path.join(os.getcwd(), CLEANUP)):
        for f in files:
            targets.append(os.path.relpath(os.path.join(root, f), os.getcwd()).replace("\\", "/"))
    for r in cf_rows:
        targets.append(r["path"])
    for t in sorted(set(targets)):
        b = read(t)
        enc_checked.append(t)
        e = {"bom": b[:3] == b"\xef\xbb\xbf", "cr": b.count(b"\r"), "tab": b.count(b"\t")}
        try:
            b.decode("utf-8")
            e["bad_utf8"] = False
        except UnicodeDecodeError:
            e["bad_utf8"] = True
        if e["bom"] or e["cr"] or e["tab"] or e["bad_utf8"]:
            enc_bad.append((t, e))
    add("encoding_hygiene", enc_bad == [], {"checked": len(enc_checked), "bad": enc_bad})
    json.dump({"checked": enc_checked, "bad": enc_bad},
              open(os.path.join(arts, "encoding.json"), "w", encoding="utf-8", newline="\n"),
              ensure_ascii=False, indent=1)

    # 9) 未动项声明：README/receipt/card 明确 O1/O3/O4/O5 + guards eol 不做
    cleanup_readme = open(os.path.join(CLEANUP, "README.md"), encoding="utf-8").read()
    cleanup_receipt = json.load(open(os.path.join(CLEANUP, "receipt.json"), encoding="utf-8"))
    card = json.load(open(".normify/changes/2026-11-06-render-r06-multiframe-obs-cleanup.json", encoding="utf-8"))
    decl = {
        "readme_mentions_not_done": ("O1/O3/O4/O5" in cleanup_readme and "guards eol" in cleanup_readme),
        "receipt_not_done": cleanup_receipt["boundaries"]["not_done"],
        "card_acceptance_mentions_not_done": any("不做" in a and "O1/O3/O4/O5" in a for a in card["acceptance"]),
        "backlog_card_untouched": run("git", "diff", "--name-only", BASE, HEAD, "--", BACKLOG)[1].strip() == b"",
        "backlog_card_status": json.load(open(BACKLOG, encoding="utf-8"))["status"],
    }
    add("untouched_items_declared", decl["readme_mentions_not_done"] and decl["card_acceptance_mentions_not_done"]
        and decl["backlog_card_untouched"] and decl["backlog_card_status"] == "backlog", decl)
    json.dump(decl, open(os.path.join(arts, "untouched.json"), "w", encoding="utf-8", newline="\n"), ensure_ascii=False, indent=1)

    # 10) 被复核卡状态
    add("reviewed_card_verified", card["status"] == "verified" and card["revision"]["before"] == "c6988afddf7c039f7236be35bdbd4923f91a97cd",
        {"status": card["status"], "revision_before": card["revision"]["before"]})

    # 汇总
    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL",
               "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed,
               "target_head": HEAD, "baseline": BASE,
               "checks": checks}
    json.dump(summary, open(os.path.join(logs, "verify-obs-cleanup.json"), "w", encoding="utf-8", newline="\n"),
              ensure_ascii=False, indent=1)
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    if args.print_json:
        print(json.dumps(summary, ensure_ascii=False, indent=1))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())


