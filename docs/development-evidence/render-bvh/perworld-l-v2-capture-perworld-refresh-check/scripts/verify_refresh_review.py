"""QA 只读复核：P0-1 采集脚本每 world 场景刷新（refit_bvh 紧前于 render）。

复核目标 HEAD e34df3b7，基线 edc6019b。
只读：不采集、不跑 GPU、不改被复核脚本/证据/既有卡；仅在 --out 目录写证据。
脚本内“同树旧脚本对比”会临时把 edc6019b 的主脚本写回真实路径运行并在 finally 恢复校验。

用法（工作目录 = 仓库根）：
  python -B scripts/verify_refresh_review.py --out docs/development-evidence/render-bvh/perworld-l-v2-capture-perworld-refresh-check
"""
import argparse
import ast
import hashlib
import json
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

REVIEW_BASE = "e34df3b7"
BASE = "edc6019b"
SCRIPT = "scripts/capture_render_perworld.py"
META = "scripts/perworld_capture_meta.py"
PREP = "scripts/perworld_light_prep.py"
EVID = "docs/development-evidence/render-bvh/perworld-l-v2-capture-perworld-refresh/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v2-capture-perworld-refresh-check/"
CARD = ".normify/changes/2026-11-07-render-perworld-l-v2-capture-perworld-refresh.json"
CARD_CHECK = ".normify/changes/2026-11-07-render-perworld-l-v2-capture-perworld-refresh-check.json"
UPSTREAM = "target/mjwarp-parallel/upstream/mujoco_warp"
DEFAULT_OUT = "fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"
WORLDS1_OUT = "fixtures/render-bvh/scene-a/perworld/prep-worlds1-dryrun"
checks = []


def run(*args, **kw):
    return subprocess.run(list(args), capture_output=True, cwd=kw.get("cwd"), env=kw.get("env"))


def read(p):
    return open(os.path.join(os.getcwd(), p), "rb").read()


def sha(b):
    return hashlib.sha256(b).hexdigest()


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + cid + " :: " + str(detail)[:420])


def env_base():
    env = dict(os.environ)
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def leaf_map(obj, prefix=""):
    out = {}
    if isinstance(obj, dict):
        for k, v in obj.items():
            out.update(leaf_map(v, prefix + "/" + str(k)))
    elif isinstance(obj, list):
        if all(not isinstance(x, (dict, list)) for x in obj):
            out[prefix] = json.dumps(obj, ensure_ascii=False)
        else:
            for i, v in enumerate(obj):
                out.update(leaf_map(v, prefix + "/" + str(i)))
    else:
        out[prefix] = obj
    return out


def call_name(node):
    if isinstance(node, ast.Call):
        f = node.func
        if isinstance(f, ast.Attribute):
            return f.attr
        if isinstance(f, ast.Name):
            return f.id
    return None


def walk_calls(fn, name, parent_attr=None):
    hits = []
    for node in ast.walk(fn):
        if isinstance(node, ast.Call):
            n = call_name(node)
            if n == name:
                if parent_attr is None:
                    hits.append(node)
                elif isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name) \
                        and node.func.value.id == parent_attr:
                    hits.append(node)
    return hits


def args_of(call):
    return [a.id if isinstance(a, ast.Name) else (a.attr if isinstance(a, ast.Attribute) else None)
            for a in call.args]


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

    # 0) 复核目标可追溯
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    anc = run("git", "merge-base", "--is-ancestor", REVIEW_BASE, "HEAD").returncode
    after = run("git", "diff", "--name-only", REVIEW_BASE, "HEAD").stdout.decode().splitlines()
    mine_ok = all(("perworld-l-v1-reference-discriminability-check" in p)
                  or ("perworld-l-v1-reference-review" in p)
                  or ("perworld-l-v2-capture-perworld-refresh-check" in p) for p in after)
    add("review_target_traceable_and_only_qa_artifacts_after", anc == 0 and mine_ok,
        {"target": REVIEW_BASE, "current_head": head, "after": after})

    # 1) 变更面与零越界
    changed = run("git", "diff", "--name-only", BASE, REVIEW_BASE).stdout.decode().splitlines()
    expected = {CARD, ".normify/changes/2026-11-07-render-perworld-l-v1-reference-review.json", SCRIPT, META} | \
        {f".normify/runs/2026-11-07-render-perworld-l-v2-capture-perworld-refresh/{g}.json" for g in ["fmt", "clippy", "clippy-cuda", "unit"]} | \
        {EVID + rel for rel in ["README.md", "receipt.json", "manifest.txt",
                                "artifacts/default-dryrun-diff.json", "artifacts/perworld-refresh.json",
                                "artifacts/scripts.diff", "artifacts/worlds1-behavior.json",
                                "logs/after-default-dryrun.json", "logs/after-default-dryrun.stderr.log",
                                "logs/after-worlds1-dryrun.json", "logs/after-worlds1-dryrun.stderr.log",
                                "logs/before-default-dryrun.json", "logs/before-default-dryrun.stderr.log",
                                "logs/before-meta.txt", "logs/before-worlds1-dryrun.json",
                                "logs/before-worlds1-dryrun.stderr.log", "logs/call-order-assertion.log",
                                "logs/line-counts.txt", "scripts/verify_refresh_call_order.py"]}
    add("scope_27_paths_expected", set(changed) == expected,
        {"n": len(changed), "extra": sorted(set(changed) - expected),
         "missing": sorted(expected - set(changed))})
    probe = run("git", "diff", "--name-only", BASE, REVIEW_BASE, "--", "fixtures", "src", "tests",
                "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion").stdout.decode().splitlines()
    add("zero_diff_fixtures_src_tests_spec", probe == [], probe)
    dump("scope.json", {"base": BASE, "head": REVIEW_BASE, "changed": changed, "zero_diff_probe": probe})

    # 2) 行数
    lines = {}
    for p in (SCRIPT, META, PREP):
        b = run("git", "cat-file", "blob", f"{REVIEW_BASE}:{p}").stdout
        lines[p] = {"lines": b.count(b"\n"), "bytes": len(b), "sha256": sha(b)}
    old = run("git", "cat-file", "blob", f"{BASE}:{SCRIPT}").stdout
    old_meta = run("git", "cat-file", "blob", f"{BASE}:{META}").stdout
    add("line_ratchet_500_244_160_and_baseline_499_230",
        lines[SCRIPT]["lines"] == 500 and lines[META]["lines"] == 244 and lines[PREP]["lines"] == 160
        and old.count(b"\n") == 499 and old_meta.count(b"\n") == 230
        and all(v["lines"] <= 500 for v in lines.values()),
        {"lines": lines, "baseline": {"main": old.count(b"\n"), "meta": old_meta.count(b"\n")}})
    dump("line-counts.json", {"head": REVIEW_BASE, "files": lines,
                              "baseline": {"main": old.count(b"\n"), "meta": old_meta.count(b"\n")}})

    # 3) 独立 AST 检查（改法/覆盖）
    src = read(SCRIPT).decode("utf-8")
    meta_src = read(META).decode("utf-8")
    tree = ast.parse(src)
    meta_tree = ast.parse(meta_src)
    funcs = {n.name: n for n in tree.body if isinstance(n, ast.FunctionDef)}
    meta_funcs = {n.name: n for n in meta_tree.body if isinstance(n, ast.FunctionDef)}
    ri = funcs.get("render_images")
    render_calls = walk_calls(ri, "render", parent_attr="mjw") if ri else []
    refit_calls = walk_calls(ri, "refit_scene_before_render") if ri else []
    preceding = False
    if ri:
        body = [s for s in ri.body]
        for i, st in enumerate(body):
            if isinstance(st, ast.Expr) and isinstance(st.value, ast.Call) and call_name(st.value) == "render" \
                    and isinstance(st.value.func, ast.Attribute) and isinstance(st.value.func.value, ast.Name) \
                    and st.value.func.value.id == "mjw" and i > 0:
                prev = body[i - 1]
                preceding = (isinstance(prev, ast.Expr) and call_name(prev.value) == "refit_scene_before_render"
                             and args_of(prev.value) == ["m", "d", "rc"])
    helper = meta_funcs.get("refit_scene_before_render")
    helper_calls = walk_calls(helper, "refit_bvh", parent_attr="mjw") if helper else []
    helper_args = args_of(helper_calls[0]) if helper_calls else None
    control = [n for n in (ast.walk(helper) if helper else [])
               if isinstance(n, (ast.If, ast.For, ast.While, ast.Try))]
    imported = any(isinstance(n, ast.ImportFrom) and n.module == "perworld_capture_meta"
                   and any(a.name == "refit_scene_before_render" for a in n.names) for n in tree.body)
    add("ast_refit_precedes_render_single_call_and_helper_contract",
        len(render_calls) == 1 and len(refit_calls) == 1 and preceding
        and len(helper_calls) == 1 and helper_args == ["m", "d", "rc"] and control == [] and imported,
        {"render_calls": len(render_calls), "refit_calls_in_render_images": len(refit_calls),
         "refit_is_previous_statement": preceding, "helper_refit_calls": len(helper_calls),
         "helper_args": helper_args, "helper_control_nodes": len(control), "imported": imported})

    # 4) 覆盖：capture_once -> render_images 一次；main -> capture_once 两次且在 create_render_context 之后
    co = funcs.get("capture_once")
    mn = funcs.get("main")
    co_render = walk_calls(co, "render_images") if co else []
    mn_capture = walk_calls(mn, "capture_once") if mn else []
    mn_ctx = walk_calls(mn, "create_render_context") if mn else []
    after_ctx = bool(mn_capture) and bool(mn_ctx) and all(c.lineno > mn_ctx[0].lineno for c in mn_capture)
    add("coverage_one_render_images_in_capture_once_two_capture_once_in_main",
        len(co_render) == 1 and len(mn_capture) == 2 and after_ctx,
        {"capture_once_calls_render_images": len(co_render), "main_calls_capture_once": len(mn_capture),
         "main_ctx_lineno": mn_ctx[0].lineno if mn_ctx else None,
         "capture_once_linenos": [c.lineno for c in mn_capture]})
    dump("ast-checks.json", {"render_images": {"render_calls": len(render_calls),
                                               "refit_calls": len(refit_calls),
                                               "preceding": preceding},
                             "helper": {"calls": len(helper_calls), "args": helper_args,
                                        "control_nodes": len(control), "imported": imported},
                             "coverage": {"capture_once_render_images": len(co_render),
                                          "main_capture_once": len(mn_capture), "after_context": after_ctx}})

    # 5) 执行方断言器重跑
    av = run(sys.executable, "-B", os.path.join(EVID, "scripts/verify_refresh_call_order.py"), env=env_base())
    av_text = (av.stdout + av.stderr).decode("utf-8", "replace")
    ev_log = open(os.path.join(EVID, "logs/call-order-assertion.log"), encoding="utf-8").read()
    add("executed_assertion_script_10_of_10_reproduced",
        av.returncode == 0 and "SUMMARY PASS 10/10" in av_text and "SUMMARY PASS 10/10" in ev_log
        and sum(1 for l in av_text.splitlines() if l.startswith("PASS ")) >= 10 and "exit_code: 0" in ev_log,
        {"exit": av.returncode, "summary": "SUMMARY PASS 10/10" in av_text,
         "pass_lines": sum(1 for l in av_text.splitlines() if l.startswith("PASS "))})
    dump("call-order-assertion.json", {"exit": av.returncode,
                                       "summary_line": [l for l in av_text.splitlines() if l.startswith("SUMMARY")],
                                       "evidence_log_exit_code": [l for l in ev_log.splitlines() if "exit_code" in l]})

    # 6) 上游根因引用核对（pinned checkout 源码）
    def up(p):
        return read(os.path.join(UPSTREAM, p)).decode("utf-8")
    init_src = up("__init__.py")
    bvh_src = up("_src/bvh.py")
    ru_src = up("_src/render_util.py")
    bvh_tree = ast.parse(bvh_src)
    bvh_funcs = {n.name: n for n in ast.walk(bvh_tree) if isinstance(n, ast.FunctionDef)}
    rb = bvh_funcs.get("refit_bvh")
    rsb = bvh_funcs.get("refit_scene_bvh")
    rb_args = [a.arg for a in rb.args.args] if rb else None
    rsb_body = ast.get_source_segment(bvh_src, rsb) or "" if rsb else ""
    rsb_lines = rsb_body.splitlines()
    rsb_ok = ("d.nworld" in rsb_body and "bvh_ngeom" in rsb_body
              and "geom_xpos" in rsb_body and "geom_xmat" in rsb_body)
    ru_ok = "def create_render_context" in ru_src and "MjData(" in ru_src and "mj_forward(" in ru_src
    ru_lines = [i + 1 for i, l in enumerate(ru_src.splitlines())
                if "MjData(" in l or "mj_forward(" in l][:4]
    init_ok = "refit_bvh" in init_src
    add("upstream_root_cause_citations_verified",
        rb is not None and rb_args == ["m", "d", "rc"] and rsb is not None and rsb_ok and ru_ok and init_ok,
        {"refit_bvh_args": rb_args, "refit_scene_bvh_ok": rsb_ok,
         "create_render_context_mjdata_forward_lines": ru_lines,
         "init_exports_refit_bvh": init_ok,
         "refit_bvh_lineno": rb.lineno if rb else None,
         "refit_scene_bvh_lineno": rsb.lineno if rsb else None})
    dump("upstream-citations.json", {"pinned_upstream": "target/mjwarp-parallel/upstream (71da24d9)",
                                     "refit_bvh_args": rb_args, "refit_bvh_lineno": rb.lineno if rb else None,
                                     "refit_scene_bvh_lineno": rsb.lineno if rsb else None,
                                     "refit_scene_bvh_uses_nworld_geom": rsb_ok,
                                     "render_util_citations": ru_lines,
                                     "init_exports_refit_bvh": init_ok})

    # 7) 默认 dry-run 同树旧/新脚本逐叶对比
    cur = read(SCRIPT)
    old_bytes = run("git", "cat-file", "blob", f"{BASE}:{SCRIPT}").stdout
    restored = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old_bytes)
        r_old = run(sys.executable, "-B", "scripts/capture_render_perworld.py", "--dry-run",
                    "--out-dir", DEFAULT_OUT, env=env_base())
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored = read(SCRIPT) == cur
    r_new = run(sys.executable, "-B", "scripts/capture_render_perworld.py", "--dry-run",
                "--out-dir", DEFAULT_OUT, env=env_base())
    if r_old.returncode == 0 and r_new.returncode == 0:
        bo, bn = leaf_map(json.loads(r_old.stdout)), leaf_map(json.loads(r_new.stdout))
        added = sorted(set(bn) - set(bo))
        removed = sorted(set(bo) - set(bn))
        changed = sorted(k for k in set(bo) & set(bn) if bo[k] != bn[k])
        d = {"added": added, "removed": removed, "changed": changed,
             "legacy": [bo.get("/legacy_manifest_entries"), bn.get("/legacy_manifest_entries")]}
        ok = (added == [] and removed == [] and changed == ["/recipe/script/bytes", "/recipe/script/sha256"]
              and bo.get("/legacy_manifest_entries") == bn.get("/legacy_manifest_entries"))
    else:
        d, ok = None, False
    add("default_dryrun_only_script_sha_bytes_changed", ok and restored, {"diffs": d, "restored": restored})
    dump("default-behavior-diff.json", {"same_tree": True, "script_restored": restored,
                                        "old_sha256": sha(old_bytes), "new_sha256": sha(cur), "diffs": d})

    # 8) 证据 dry-run 日志复跑（backslash 命令行匹配证据）
    def dry_repr(script_arg, args):
        return run(sys.executable, "-B", script_arg, *args, env=env_base())
    repr_script = "scripts\\capture_render_perworld.py"
    repr_args = ["--dry-run", "--out-dir", "fixtures\\render-bvh\\scene-a\\perworld\\prep-default-dryrun"]
    restored2 = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old_bytes)
        ro2 = dry_repr(repr_script, repr_args)
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored2 = read(SCRIPT) == cur
    rn2 = dry_repr(repr_script, repr_args)
    ev_before = json.load(open(os.path.join(EVID, "logs/before-default-dryrun.json"), encoding="utf-8"))
    ev_after = json.load(open(os.path.join(EVID, "logs/after-default-dryrun.json"), encoding="utf-8"))

    def dmap(x, y):
        return {"added": sorted(set(y) - set(x)), "removed": sorted(set(x) - set(y)),
                "changed": sorted(k for k in set(x) & set(y) if x[k] != y[k])}
    logs_before = dmap(leaf_map(json.loads(ro2.stdout)), leaf_map(ev_before))
    logs_after = dmap(leaf_map(json.loads(rn2.stdout)), leaf_map(ev_after))
    add("evidence_dryrun_logs_reproduced",
        logs_before == {"added": [], "removed": [], "changed": []}
        and logs_after == {"added": [], "removed": [], "changed": []} and restored2,
        {"before_diff": logs_before, "after_diff": logs_after, "restored": restored2})
    dump("evidence-logs-reproduced.json", {"before_diff": logs_before, "after_diff": logs_after})

    # 9) --worlds 1 两版本一致拒绝
    w1_args = ["--dry-run", "--worlds", "1", "--hinge-deg", "0", "--out-dir", WORLDS1_OUT]
    restored3 = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old_bytes)
        w1_old = run(sys.executable, "-B", "scripts/capture_render_perworld.py", *w1_args, env=env_base())
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored3 = read(SCRIPT) == cur
    w1_new = run(sys.executable, "-B", "scripts/capture_render_perworld.py", *w1_args, env=env_base())
    e_old = (w1_old.stderr or b"").decode("utf-8", "replace").strip()
    e_new = (w1_new.stderr or b"").decode("utf-8", "replace").strip()
    w1_ok = (w1_old.returncode == 1 and w1_new.returncode == 1 and e_old == e_new
             and "must be >= 2" in e_new and not os.path.exists(WORLDS1_OUT))
    add("worlds1_rejected_identically_both_versions", w1_ok and restored3,
        {"old_exit": w1_old.returncode, "new_exit": w1_new.returncode,
         "stderr_equal": e_old == e_new, "stderr": e_new[:160],
         "out_dir_created": os.path.exists(WORLDS1_OUT)})
    dump("worlds1-behavior.json", {"old_exit": w1_old.returncode, "new_exit": w1_new.returncode,
                                   "stderr_equal": e_old == e_new, "stderr": e_new,
                                   "out_dir_created": os.path.exists(WORLDS1_OUT),
                                   "restored": restored3})

    # 10) 证据自核：manifest 18/18（19 文件）、编码、artifacts、scripts.diff
    mraw = read(EVID + "manifest.txt")
    mlines = mraw.decode("utf-8").splitlines()
    mrows = {}
    for ln in mlines:
        if not ln.strip() or ln.startswith("#"):
            continue
        p = ln.split()
        if len(p) == 3:
            mrows[p[2]] = {"sha256": p[0], "bytes": int(p[1])}
    files = []
    for root, _, fs in os.walk(os.path.join(os.getcwd(), EVID)):
        for f in fs:
            files.append(os.path.relpath(os.path.join(root, f), EVID).replace("\\", "/"))
    rows_ok = {}
    for rel, mm in mrows.items():
        b = read(EVID + rel)
        rows_ok[rel] = sha(b) == mm["sha256"] and len(b) == mm["bytes"]
    add("evidence_manifest_18_entries_19_files",
        len(mrows) == 18 and len(files) == 19 and all(rows_ok.values())
        and mlines[0].startswith("#") and "manifest.txt" not in mrows,
        {"entries": len(mrows), "files": len(files), "bad": [k for k, v in rows_ok.items() if not v]})
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
    add("evidence_encoding_hygiene", enc_bad == [], {"checked": len(files), "bad": enc_bad})
    diff_art = json.load(open(os.path.join(EVID, "artifacts/default-dryrun-diff.json"), encoding="utf-8"))
    w1_art = json.load(open(os.path.join(EVID, "artifacts/worlds1-behavior.json"), encoding="utf-8"))
    refresh_art = json.load(open(os.path.join(EVID, "artifacts/perworld-refresh.json"), encoding="utf-8"))
    actual_diff = run("git", "diff", "--text", f"{BASE}..{REVIEW_BASE}", "--", SCRIPT, META).stdout.decode("utf-8")
    stored_diff = open(os.path.join(EVID, "artifacts/scripts.diff"), encoding="utf-8").read()
    art_ok = (diff_art.get("unexpected_count") == 0 and len(diff_art.get("entries", [])) == 2
              and {e["path"] for e in diff_art["entries"]} == {"/recipe/script/sha256", "/recipe/script/bytes"}
              and w1_art.get("identical") is True
              and refresh_art.get("line_counts", {}).get("capture_render_perworld.py") == 500
              and refresh_art.get("line_counts", {}).get("perworld_capture_meta.py") == 244
              and refresh_art.get("call_order_assertion", {}).get("summary") == "SUMMARY PASS 10/10"
              and stored_diff.strip() == actual_diff.strip())
    add("evidence_artifacts_and_diff_consistent", art_ok,
        {"diff_unexpected": diff_art.get("unexpected_count"),
         "diff_paths": [e["path"] for e in diff_art.get("entries", [])],
         "worlds1_identical": w1_art.get("identical"),
         "stored_diff_matches_git": stored_diff.strip() == actual_diff.strip()})
    dump("evidence-selfcheck.json", {"manifest_entries": len(mrows), "files": len(files),
                                     "rows_ok": rows_ok, "encoding_bad": enc_bad,
                                     "diff_art_ok": art_ok})

    # 11) 被复核卡与 gate
    card = json.load(open(CARD, encoding="utf-8"))
    unit = json.load(open(os.path.join(".normify/runs/2026-11-07-render-perworld-l-v2-capture-perworld-refresh/unit.json"),
                          encoding="utf-8"))
    gates = [c.get("gate") for c in card["checks"] if c.get("kind") == "gate"]
    add("reviewed_card_verified_unit_1359_0",
        card["status"] == "verified" and card["revision"]["before"] == "27bbd8fb538c3cfad9b0a76e950d858d0c13d900"
        and gates == ["fmt", "clippy", "clippy-cuda", "unit"]
        and unit["tests"]["passed"] == 1359 and unit["tests"]["failed"] == 0,
        {"status": card["status"], "revision_before": card["revision"]["before"], "gates": gates,
         "unit": unit["tests"]})
    dump("reviewed-card.json", {"status": card["status"], "revision_before": card["revision"]["before"],
                                "gates": gates, "unit_tests": unit["tests"]})

    # 12) 依赖与非声明在位
    readme = open(os.path.join(EVID, "README.md"), encoding="utf-8").read()
    receipt = json.load(open(os.path.join(EVID, "receipt.json"), encoding="utf-8"))
    stmt = str(receipt.get("non_claims", {}).get("statement", ""))
    g_l1 = str(receipt.get("g_l1_dependency", ""))
    claims_ok = ("G-L1" in readme and "P0-2" in readme and "pass-margin" in readme
                 and "不宣称" in readme and "判别力恢复" in readme
                 and "no claim" in stmt.lower() and "discriminability" in stmt.lower()
                 and "P0-2" in g_l1 and "pass-margin" in g_l1)
    add("gl1_dependency_and_non_claims_present", claims_ok,
        {"readme_has_g_l1_pilot_margin": "G-L1" in readme and "pass-margin" in readme})

    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL", "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed, "target_head": REVIEW_BASE, "checks": checks}
    with open(os.path.join(logs, "verify-refresh-review.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())


