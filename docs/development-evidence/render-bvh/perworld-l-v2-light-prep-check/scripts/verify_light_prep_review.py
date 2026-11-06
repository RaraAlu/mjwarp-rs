"""QA 只读复核：perworld L v2 灯备选准备片（默认行为 / 边界 / 行数 / dry-run 三自检 / 负例 / 证据）。

复核目标 HEAD b0b8cc02，基线 b0b8cc02^。
只读：不改被复核脚本、场景、证据；运行 dry-run（CPU、零 GPU、不写参考）。
脚本内“同树旧脚本对比”会临时把 b0b8cc02^ 的脚本内容写回 scripts/capture_render_perworld.py，
运行后在同一进程的 finally 中恢复并校验 sha；调用方需持有仓库写锁（QA 复核时由 qa-night 持有）。

用法（工作目录 = 仓库根）：
  python scripts/verify_light_prep_review.py --out docs/development-evidence/render-bvh/perworld-l-v2-light-prep-check
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

sys.dont_write_bytecode = True

BASE = "b0b8cc02^"
HEAD = "b0b8cc02"
REVIEW_BASE = "b0b8cc02"
SCRIPT = "scripts/capture_render_perworld.py"
MODULE = "scripts/perworld_light_prep.py"
SCENE_A = "fixtures/render-bvh/scene-a/scene.xml"
SCENE_L = "fixtures/render-bvh/scene-a-light/scene.xml"
EVID = "docs/development-evidence/render-bvh/perworld-l-v2-light-prep/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v2-light-prep-check/"
CARD = ".normify/changes/2026-11-06-render-perworld-l-v2-light-prep.json"
CARD_CHECK = ".normify/changes/2026-11-06-render-perworld-l-v2-light-prep-check.json"
EXPECTED_SCOPE = {
    CARD,
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-prep/clippy-cuda.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-prep/clippy.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-prep/fmt.json",
    ".normify/runs/2026-11-06-render-perworld-l-v2-light-prep/unit.json",
    EVID + "README.md",
    EVID + "manifest.txt",
    EVID + "receipt.json",
    EVID + "artifacts/default-dryrun-diff.json",
    EVID + "artifacts/prep-meta.json",
    EVID + "artifacts/scene.diff",
    EVID + "artifacts/script.diff",
    EVID + "logs/after-default-dryrun.json",
    EVID + "logs/after-default-dryrun.stderr.log",
    EVID + "logs/before-default-dryrun.json",
    EVID + "logs/before-default-dryrun.stderr.log",
    EVID + "logs/before-meta.txt",
    EVID + "logs/explicit-light-dryrun.json",
    EVID + "logs/explicit-light-dryrun.stderr.log",
    EVID + "logs/line-counts.txt",
    EVID + "logs/negative-outdir-nonempty.log",
    EVID + "logs/negative-outdir-outside-root.log",
    SCENE_L,
    SCRIPT,
    MODULE,
}
BACKLOG_OR_FROZEN = [
    "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/",
    "fixtures/render-bvh/scene-a-light/perworld/",
]
EXPECTED_EVIDENCE_FILES = 17
checks = []


def run(*args, **kw):
    return subprocess.run(list(args), capture_output=True, cwd=kw.get("cwd"), env=kw.get("env"))


def git_bytes(rev, path):
    p = run("git", "cat-file", "blob", f"{rev}:{path}")
    assert p.returncode == 0, (rev, path, p.stderr[:200])
    return p.stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def read(p):
    return open(os.path.join(os.getcwd(), p), "rb").read()


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + cid + " :: " + str(detail)[:400])


def pyenv():
    env = dict(os.environ)
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def run_dry(extra, env=None):
    return run(sys.executable, "-B", SCRIPT, "--dry-run", *extra, env=env or pyenv())


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
    rc_head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    rc_anc = run("git", "merge-base", "--is-ancestor", REVIEW_BASE, "HEAD").returncode
    after = run("git", "diff", "--name-only", REVIEW_BASE, "HEAD").stdout.decode().splitlines()
    mine = [p for p in after if p == CARD_CHECK or p.startswith(CHECK) or p.startswith(".normify/runs/2026-11-06-render-perworld-l-v2-light-prep-check/")]
    add("review_target_traceable_and_only_qa_commits_after",
        rc_anc == 0 and after == mine,
        {"review_target": REVIEW_BASE, "current_head": rc_head, "after": after})

    # 1) 变更面与零越界
    changed = run("git", "diff", "--name-only", BASE, HEAD).stdout.decode().splitlines()
    add("scope_exact_expected_set", set(changed) == EXPECTED_SCOPE,
        {"n_changed": len(changed), "extra": sorted(set(changed) - EXPECTED_SCOPE),
         "missing": sorted(EXPECTED_SCOPE - set(changed))})
    probe = run("git", "diff", "--name-only", BASE, HEAD, "--", "src",
                "fixtures/render-bvh/scene-a", "docs/development-evidence/render-bvh/sdf-criterion-v2.2",
                *BACKLOG_OR_FROZEN).stdout.decode().splitlines()
    add("zero_diff_src_scenea_perworld_criterion", probe == [], probe)
    ldir = sorted(os.listdir(os.path.dirname(SCENE_L)))
    add("scene_a_light_only_scene_xml", ldir == ["scene.xml"], ldir)
    dump("scope.json", {"base": BASE, "head": HEAD, "changed": changed,
                        "zero_diff_probe": probe, "scene_a_light_dir": ldir})

    # 2) 行数（git blob 换行口径）与版本基线
    lines = {}
    for p in (SCRIPT, MODULE, SCENE_L):
        b = git_bytes(HEAD, p)
        lines[p] = {"lines": b.count(b"\n"), "bytes": len(b), "sha256": sha(b), "ends_newline": b.endswith(b"\n")}
    base_script = git_bytes(BASE, SCRIPT)
    baseline = {"lines": base_script.count(b"\n"), "bytes": len(base_script), "sha256": sha(base_script)}
    add("line_ratchet_497_160_84_and_baseline_499",
        lines[SCRIPT]["lines"] == 497 and lines[MODULE]["lines"] == 160 and lines[SCENE_L]["lines"] == 84
        and baseline["lines"] == 499 and baseline["sha256"].startswith("67a760fa")
        and all(v["lines"] <= 500 for v in lines.values()),
        {"files": lines, "baseline_script": baseline})
    meta = json.load(open(os.path.join(EVID, "artifacts/prep-meta.json"), encoding="utf-8"))
    meta_ok = (meta["script"]["lines"] == lines[SCRIPT]["lines"] and meta["script"]["sha256"] == lines[SCRIPT]["sha256"]
               and meta["script"]["bytes"] == lines[SCRIPT]["bytes"] and meta["script"]["baseline_sha256"] == baseline["sha256"]
               and meta["script"]["baseline_lines"] == baseline["lines"]
               and meta["module"]["lines"] == lines[MODULE]["lines"] and meta["module"]["sha256"] == lines[MODULE]["sha256"]
               and meta["module"]["bytes"] == lines[MODULE]["bytes"]
               and meta["scene"]["lines"] == lines[SCENE_L]["lines"] and meta["scene"]["sha256"] == lines[SCENE_L]["sha256"]
               and meta["scene"]["bytes"] == lines[SCENE_L]["bytes"])
    add("prep_meta_matches_measured", meta_ok, {"script": meta["script"], "module": meta["module"], "scene": meta["scene"]})
    dump("line-counts.json", {"head": HEAD, "files": lines, "baseline_script": baseline,
                              "prep_meta_matches": meta_ok})

    # 3) legacy 计数算术：旧树 106 → 新树 107 = 新增 scene-a-light/scene.xml 一个文件
    def legacy_paths(rev):
        ls = run("git", "ls-tree", "-r", "--name-only", rev, "fixtures/render-bvh").stdout.decode().splitlines()
        excluded = "fixtures/render-bvh/scene-a/perworld/"
        return [p for p in ls if not p.startswith(excluded)]
    lb, lh = legacy_paths(BASE), legacy_paths(HEAD)
    added = sorted(set(lh) - set(lb))
    add("legacy_count_106_to_107_explained_by_new_fixture",
        len(lb) == 106 and len(lh) == 107 and added == [SCENE_L],
        {"before": len(lb), "after": len(lh), "added": added})
    dump("legacy-counts.json", {"before": len(lb), "after": len(lh), "added": added})

    # 4) 场景变体等价性
    ta = ET.fromstring(open(SCENE_A, encoding="utf-8").read())
    tb = ET.fromstring(open(SCENE_L, encoding="utf-8").read())

    def lights(root):
        return {e.get("name"): dict(e.attrib) for e in root.iter("light")}

    la, lb2 = lights(ta), lights(tb)
    parents = {}
    for e in tb.iter("light"):
        for p in tb.iter():
            if e in list(p):
                parents[e.get("name")] = p.get("name") or p.tag

    def canon(root):
        for p in root.iter():
            for c in list(p):
                if c.tag == "light":
                    p.remove(c)

        def walk(e):
            return (e.tag, tuple(sorted(e.attrib.items())), (e.text or "").strip(),
                    tuple(walk(c) for c in list(e)))
        return walk(root)

    struct_equal = canon(ET.fromstring(open(SCENE_A, encoding="utf-8").read())) == canon(
        ET.fromstring(open(SCENE_L, encoding="utf-8").read()))
    same_attrs = set(la) == set(lb2) and all(la[k] == lb2[k] for k in la)
    add("scene_lights_same_names_and_attributes", same_attrs and set(parents.values()) == {"tri_body"} and struct_equal,
        {"light_names": sorted(la), "attr_counts": {k: len(v) for k, v in la.items()},
         "parents": parents, "struct_equal_after_removing_lights": struct_equal})
    dump("scene-compare.json", {"light_names": sorted(la),
                                "attributes": la, "variant_parents": parents,
                                "struct_equal_after_removing_lights": struct_equal})

    # 5) 脚本默认参数（导入模块，不执行 main）
    sys.path.insert(0, os.path.join(os.getcwd(), "scripts"))
    import capture_render_perworld as cap  # noqa: E402
    old_argv = sys.argv
    sys.argv = ["capture_render_perworld.py", "--out-dir", "dummy-out"]
    try:
        ns = cap.parse_args()
    finally:
        sys.argv = old_argv
    defaults = {
        "scene_is_default": str(ns.scene) == str(cap.SCENE_XML),
        "out_root_is_default": str(ns.out_root) == str(cap.OUT_ROOT),
        "repeat": ns.repeat, "repeat_default_2": ns.repeat == 2,
        "worlds": ns.worlds, "hinge_deg": ns.hinge_deg, "seed": ns.seed,
        "task_id": cap.TASK_ID,
    }
    src = read(SCRIPT).decode("utf-8")
    add("script_defaults_unchanged",
        defaults["scene_is_default"] and defaults["out_root_is_default"] and defaults["repeat_default_2"]
        and ns.worlds == 3 and ns.hinge_deg == "0,20,40"
        and 'choices=(1, 2), default=2' in src and 'TASK_ID = "2026-10-30-render-perworld-capture-authoring"' in src,
        defaults)
    dump("script-semantics.json", defaults)

    # 6) 同树旧脚本 vs 新脚本默认 dry-run 逐叶对比
    cur = read(SCRIPT)
    old = git_bytes(BASE, SCRIPT)
    diffs = None
    restored = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old)
        r_old = run_dry(["--out-dir", "fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"])
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored = read(SCRIPT) == cur
    r_new = run_dry(["--out-dir", "fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"])
    if r_old.returncode == 0 and r_new.returncode == 0:
        bo = leaf_map(json.loads(r_old.stdout))
        bn = leaf_map(json.loads(r_new.stdout))
        added = sorted(set(bn) - set(bo))
        removed = sorted(set(bo) - set(bn))
        changed = sorted(k for k in set(bo) & set(bn) if bo[k] != bn[k])
        expected_added = {"/paths/default_scene", "/paths/default_out_root", "/paths/effective_scene",
                           "/paths/effective_out_root", "/paths/out_dir", "/paths/scene_explicit",
                           "/paths/out_root_explicit"}
        expected_changed = {"/recipe/script/sha256", "/recipe/script/bytes"}
        diffs = {"added": added, "removed": removed, "changed": changed,
                 "legacy_before": bo.get("/legacy_manifest_entries"), "legacy_after": bn.get("/legacy_manifest_entries"),
                 "repeat_before": bo.get("/repeat"), "repeat_after": bn.get("/repeat")}
        ok = (set(added) == expected_added and removed == [] and set(changed) == expected_changed
              and bo.get("/legacy_manifest_entries") == bn.get("/legacy_manifest_entries") == 107
              and bo.get("/repeat") == bn.get("/repeat") == 2)
    else:
        ok = False
    add("default_behavior_only_three_objective_diffs", ok and restored,
        {"diffs": diffs, "script_restored": restored})
    dump("default-behavior-diff.json", {"same_tree": True,
                                        "old_script_sha256": sha(old), "new_script_sha256": sha(cur),
                                        "script_restored": restored, "diffs": diffs})

    # 6b) 被复核证据内的 before/after 原始输出与本次复跑一致
    ev_before = leaf_map(json.load(open(os.path.join(EVID, "logs/before-default-dryrun.json"), encoding="utf-8")))
    ev_after = leaf_map(json.load(open(os.path.join(EVID, "logs/after-default-dryrun.json"), encoding="utf-8")))

    def dmap(x, y):
        return {"added": sorted(set(y) - set(x)), "removed": sorted(set(x) - set(y)),
                "changed": sorted(k for k in set(x) & set(y) if x[k] != y[k])}

    if r_old.returncode == 0 and r_new.returncode == 0:
        d_after = dmap(bn, ev_after)
        d_before = dmap(bo, ev_before)
        logs_ok = (d_after == {"added": [], "removed": [], "changed": []}
                   and d_before == {"added": [], "removed": [], "changed": ["/legacy_manifest_entries"]})
    else:
        d_after, d_before, logs_ok = None, None, False
    add("evidence_logs_reproduced", logs_ok,
        {"after_diff": d_after, "before_diff": d_before})
    dump("evidence-logs-reproduced.json", {
        "after_diff": d_after, "before_diff": d_before,
        "note": "reviewed before log was produced on the pre-slice tree (no scene-a-light), so its "
                "legacy_manifest_entries is 106 while the fresh same-tree run is 107; every other leaf is identical"})

    # 7) 显式 light dry-run：三自检数值与证据一致
    env = pyenv()
    expl = run_dry(["--scene", SCENE_L, "--out-root", "fixtures/render-bvh/scene-a-light/perworld",
                    "--out-dir", "fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040",
                    "--worlds", "3", "--hinge-deg", "0,20,40"], env=env)
    e = json.loads(expl.stdout) if expl.returncode == 0 else None
    evid_log = json.load(open(os.path.join(EVID, "logs/explicit-light-dryrun.json"), encoding="utf-8"))
    light_ok = bool(e) and all([
        e.get("light_pose_precheck") is not None,
        e["light_pose_precheck"]["diff_keys"] == evid_log["light_pose_precheck"]["diff_keys"],
        [x["attached_body"] for x in e["light_pose_precheck"]["lights"]] == ["tri_body", "tri_body", "tri_body"],
        len(e["light_pose_precheck"]["per_world"]) == 3,
        e["light_pose_precheck"]["verdict"]["nonzero"] is True,
        e["paths"]["scene_explicit"] is True,
        e["paths"]["out_root_explicit"] is True,
        e["legacy_manifest_entries"] == 112,
        e["repeat"] == 2,
        e["model_static"]["cameras"] == [[0, 0, "cam-persp"], [1, 3, "cam-track"]],
    ])
    default_plan = json.loads(r_new.stdout)
    add("dry_run_three_selfchecks_reproduced", light_ok,
        {"exit": expl.returncode,
         "diff_keys": e["light_pose_precheck"]["diff_keys"] if e else None,
         "legacy": e.get("legacy_manifest_entries") if e else None,
         "cameras": e["model_static"]["cameras"] if e else None})
    add("default_run_has_no_light_precheck", "light_pose_precheck" not in default_plan,
        sorted(default_plan.keys()))
    dump("light-precheck.json", {"exit": expl.returncode,
                                 "diff_keys": e["light_pose_precheck"]["diff_keys"] if e else None,
                                 "attached_bodies": [x["attached_body"] for x in e["light_pose_precheck"]["lights"]] if e else None,
                                 "per_world_count": len(e["light_pose_precheck"]["per_world"]) if e else None,
                                 "paths": e.get("paths") if e else None,
                                 "legacy": e.get("legacy_manifest_entries") if e else None,
                                 "repeat": e.get("repeat") if e else None,
                                 "cameras": e["model_static"]["cameras"] if e else None,
                                 "evidence_diff_keys_equal": bool(e) and e["light_pose_precheck"]["diff_keys"] == evid_log["light_pose_precheck"]["diff_keys"]})

    # 8) 负例：非空 out-dir、越界 out-root
    n1 = run_dry(["--out-dir", "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040"])
    n2 = run_dry(["--out-root", "target/qa-light-prep/outside-root",
                  "--out-dir", "fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"])
    msg1 = (n1.stdout + n1.stderr).decode("utf-8", "replace")
    msg2 = (n2.stdout + n2.stderr).decode("utf-8", "replace")
    add("negative_outdir_nonempty_exit1", n1.returncode == 1 and "already exists and is not empty" in msg1, msg1.strip()[:160])
    add("negative_outdir_outside_root_exit1", n2.returncode == 1 and "outside --out-root" in msg2, msg2.strip()[:160])
    no_side_effects = all([
        not os.path.exists("fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"),
        not os.path.exists("fixtures/render-bvh/scene-a-light/perworld"),
    ])
    add("negative_checks_no_side_effects", no_side_effects, {"prep_default_dir": False, "scene_a_light_perworld": False})
    dump("negative-checks.json", {"nonempty_exit": n1.returncode, "nonempty_msg": msg1.strip(),
                                  "outside_root_exit": n2.returncode, "outside_root_msg": msg2.strip(),
                                  "no_side_effects": bool(no_side_effects)})

    # 9) 零 GPU 静态证据：dry-run 分支 want_cuda_device=False，wp.get_device 只在 True 分支
    src_lines = src.splitlines()
    dry_idx = next(i for i, l in enumerate(src_lines) if "if args.dry_run:" in l)
    dry_block = "\n".join(src_lines[dry_idx:dry_idx + 15])
    getdev = [i for i, l in enumerate(src_lines) if "wp.get_device" in l]
    add("zero_gpu_dry_run_static", "want_cuda_device=False" in dry_block and len(getdev) == 1
        and "if want_cuda_device:" in src_lines[getdev[0] - 1],
        {"dry_run_uses_want_cuda_device_false": "want_cuda_device=False" in dry_block,
         "wp_get_device_line": getdev[0] + 1, "guard": src_lines[getdev[0] - 1].strip()})

    # 10) 证据自核：manifest / 编码 / task_id / 非声明 / 卡状态
    mpath = os.path.join(EVID, "manifest.txt")
    mraw = open(mpath, "rb").read()
    mlines = mraw.decode("utf-8").split("\n")
    rows, bad = [], []
    for ln in mlines:
        if not ln.strip() or ln.startswith("#"):
            continue
        parts = ln.split()
        if len(parts) != 3:
            bad.append(("format", ln))
            continue
        rows.append(parts)
    listed = set()
    for h, size, rel in rows:
        b = open(os.path.join(EVID, rel), "rb").read()
        listed.add(rel)
        if sha(b) != h or len(b) != int(size):
            bad.append((rel, sha(b), len(b), h, size))
    files_in_dir = []
    for root, _, fs in os.walk(os.path.join(os.getcwd(), EVID)):
        for f in fs:
            files_in_dir.append(os.path.relpath(os.path.join(root, f), EVID).replace("\\", "/"))
    add("evidence_manifest_selfcheck_16_entries_plus_self",
        len(rows) == 16 and len(files_in_dir) == EXPECTED_EVIDENCE_FILES and not bad
        and mlines[0].startswith("#") and "manifest.txt" not in listed,
        {"entries": len(rows), "files_total": len(files_in_dir), "bad": bad})
    enc_bad = []
    for rel in files_in_dir:
        b = open(os.path.join(EVID, rel), "rb").read()
        e = {"bom": b[:3] == b"\xef\xbb\xbf", "cr": b.count(b"\r"), "tab": b.count(b"\t")}
        try:
            b.decode("utf-8")
            e["bad_utf8"] = False
        except UnicodeDecodeError:
            e["bad_utf8"] = True
        if e["bom"] or e["cr"] or e["tab"] or e["bad_utf8"]:
            enc_bad.append((rel, e))
    add("evidence_encoding_hygiene", enc_bad == [], {"checked": len(files_in_dir), "bad": enc_bad})
    receipt = json.load(open(os.path.join(EVID, "receipt.json"), encoding="utf-8"))
    card = json.load(open(CARD, encoding="utf-8"))
    terms = ["灯位姿已覆盖", "per-world 灯验收通过", "上游等价"]
    term_hits = {}
    for rel in files_in_dir:
        t = open(os.path.join(EVID, rel), encoding="utf-8", errors="replace").read()
        for term in terms:
            if term in t:
                term_hits.setdefault(term, []).append(rel)
    claims_ok = (receipt["task_id"] == "2026-11-06-render-perworld-l-v2-light-prep"
                 and "not a perworld L v2 start" in json.dumps(receipt["non_claims"], ensure_ascii=False)
                 and term_hits == {})
    add("evidence_task_id_non_claims_and_forbidden_terms",
        claims_ok, {"task_id": receipt["task_id"], "forbidden_hits": term_hits})
    my_card = json.load(open(CARD_CHECK, encoding="utf-8"))
    gates = [c.get("gate") for c in my_card["checks"] if c.get("kind") == "gate"]
    add("reviewed_card_verified_and_my_card_ready",
        card["status"] == "verified" and card["revision"]["before"] == "b6b8bd8981f280d81387e1cac1bf5c320ec0ab3b"
        and gates == ["fmt", "clippy", "clippy-cuda", "unit"],
        {"status": card["status"], "revision_before": card["revision"]["before"], "my_gates": gates})
    dump("evidence-selfcheck.json", {"manifest_entries": len(rows), "files_total": len(files_in_dir),
                                     "manifest_bad": bad, "encoding_bad": enc_bad,
                                     "receipt_task_id": receipt["task_id"],
                                     "forbidden_hits": term_hits,
                                     "reviewed_card_status": card["status"],
                                     "reviewed_card_revision_before": card["revision"]["before"]})

    # 11) 工作树未被运行副作用污染（fixtures 无新增/改动）
    st = run("git", "status", "--porcelain", "fixtures/").stdout.decode().splitlines()
    add("fixtures_worktree_clean_after_runs", st == [], st)

    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL", "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed,
               "target_head": REVIEW_BASE, "checks": checks}
    with open(os.path.join(logs, "verify-light-prep.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
