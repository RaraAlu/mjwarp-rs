"""QA 只读复核：perworld L v2 采集脚本 §11.1 前置（command/captured_at/per_world_scope/reference_tag）。

复核目标 HEAD b0af1be4，基线 91d628c3。
只读：不改被复核脚本/证据、既有证据、代码、fixtures；CPU-only、零 GPU、不采集。
脚本内“同树旧脚本对比”会临时把 91d628c3 的脚本内容写回 scripts/capture_render_perworld.py，
运行后同一进程 finally 恢复并校验 sha；调用方需持有仓库写锁（QA 复核时 qa-night 持有）。

用法（工作目录 = 仓库根）：
  python -B scripts/verify_capture_meta_review.py --out docs/development-evidence/render-bvh/perworld-l-v2-capture-meta-check
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

REVIEW_BASE = "9275e6e6"
FIX_BASE = "b0af1be4"
BASE = "91d628c3"
SCRIPT = "scripts/capture_render_perworld.py"
MODULE = "scripts/perworld_capture_meta.py"
EVID = "docs/development-evidence/render-bvh/perworld-l-v2-capture-meta/"
CHECK = "docs/development-evidence/render-bvh/perworld-l-v2-capture-meta-check/"
CARD = ".normify/changes/2026-11-07-render-perworld-l-v2-capture-meta.json"
CARD_CHECK = ".normify/changes/2026-11-07-render-perworld-l-v2-capture-meta-check.json"
SPEC = "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md"
NEG_OUT_DIR = "fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040"
DEFAULT_OUT_DIR = "fixtures/render-bvh/scene-a/perworld/prep-default-dryrun"
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


def env_base(extra=None):
    env = dict(os.environ)
    env.pop("MJWARP_CAPTURE_AT", None)
    env["PYTHONIOENCODING"] = "utf-8"
    if extra:
        env.update(extra)
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


def run_script(args, env):
    return run(sys.executable, "-B", SCRIPT, *args, env=env)


def dry(args=None, env=None):
    return run_script(["--dry-run", "--out-dir", DEFAULT_OUT_DIR] + (args or []), env or env_base())


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
    mine = [p for p in after if p == CARD_CHECK or p.startswith(CHECK)
            or p.startswith(".normify/runs/2026-11-07-render-perworld-l-v2-capture-meta-check/")]
    add("review_target_traceable_and_only_qa_commits_after", anc == 0 and after == mine,
        {"review_target": REVIEW_BASE, "current_head": head, "after": after})

    # 1) 变更面与零越界
    changed = run("git", "diff", "--name-only", BASE, REVIEW_BASE).stdout.decode().splitlines()
    expected = {CARD, SCRIPT, MODULE} | {f".normify/runs/2026-11-07-render-perworld-l-v2-capture-meta/{g}.json"
                                       for g in ["fmt", "clippy", "clippy-cuda", "unit"]} | \
        {EVID + f for f in ["README.md", "receipt.json", "manifest.txt",
                            "artifacts/capture-meta.json", "artifacts/default-dryrun-diff.json",
                            "artifacts/negative-cases.json", "artifacts/script.diff",
                            "artifacts/synthetic-capture-metadata.json",
                            "logs/after-default-dryrun.json", "logs/after-default-dryrun.stderr.log",
                            "logs/before-default-dryrun.json", "logs/before-default-dryrun.stderr.log",
                            "logs/before-meta.txt", "logs/line-counts.txt", "logs/module-selfcheck.log",
                            "logs/negative-captured-at-empty.log", "logs/negative-captured-at-missing.log"]}
    add("scope_exact_24_paths", set(changed) == expected,
        {"n": len(changed), "extra": sorted(set(changed) - expected), "missing": sorted(expected - set(changed))})
    probe = run("git", "diff", "--name-only", BASE, REVIEW_BASE, "--", "fixtures", "src", "tests",
                "docs/development-evidence/render-bvh/perworld-l-v2-light-criterion",
                "docs/development-evidence/render-bvh/sdf-criterion-v2.2").stdout.decode().splitlines()
    add("zero_diff_fixtures_src_tests_spec_criterion", probe == [], probe)
    dump("scope.json", {"base": BASE, "head": REVIEW_BASE, "changed": changed, "zero_diff_probe": probe,
                        "no_capture_no_reference": probe == []})

    # 2) 行数 / sha / bytes
    lines = {}
    for p in (SCRIPT, MODULE):
        b = run("git", "cat-file", "blob", f"{REVIEW_BASE}:{p}").stdout
        lines[p] = {"lines": b.count(b"\n"), "bytes": len(b), "sha256": sha(b), "ends_newline": b.endswith(b"\n")}
    old_script = run("git", "cat-file", "blob", f"{BASE}:{SCRIPT}").stdout
    add("line_ratchet_499_230_and_matches_evidence",
        lines[SCRIPT]["lines"] == 499 and lines[MODULE]["lines"] == 230
        and lines[MODULE]["lines"] <= 500 and lines[SCRIPT]["lines"] <= 500
        and old_script.count(b"\n") == 497,
        {"files": lines, "baseline_script_lines": old_script.count(b"\n")})
    meta = json.load(open(os.path.join(EVID, "artifacts/capture-meta.json"), encoding="utf-8"))
    add("capture_meta_artifact_matches_measured",
        meta["script"]["lines_after"] == lines[SCRIPT]["lines"] and meta["script"]["sha256"] == lines[SCRIPT]["sha256"]
        and meta["script"]["bytes"] == lines[SCRIPT]["bytes"] and meta["script"]["lines_before"] == 497
        and meta["module"]["lines"] == lines[MODULE]["lines"] and meta["module"]["sha256"] == lines[MODULE]["sha256"]
        and meta["module"]["bytes"] == lines[MODULE]["bytes"],
        {"script": meta["script"], "module": meta["module"]})
    dump("line-counts.json", {"head": REVIEW_BASE, "files": lines, "baseline_script_lines": old_script.count(b"\n"),
                              "artifact_script": meta["script"], "artifact_module": meta["module"]})

    # 3) 模块自检 14/14（独立重跑）
    sc = run(sys.executable, "-B", MODULE, env=env_base())
    sc_text = (sc.stdout + sc.stderr).decode("utf-8", "replace")
    m = re.search(r"SELFCHECK (PASS|FAIL) (\d+)/(\d+)", sc_text)
    ev_sc = open(os.path.join(EVID, "logs/module-selfcheck.log"), encoding="utf-8").read()
    add("module_selfcheck_14_of_14_reproduced",
        sc.returncode == 0 and m is not None and m.group(1) == "PASS" and m.group(2) == "14"
        and m.group(3) == "14" and "SELFCHECK PASS 14/14" in ev_sc,
        {"exit": sc.returncode, "summary": m.group(0) if m else None,
         "evidence_log_has_summary": "SELFCHECK PASS 14/14" in ev_sc})
    dump("module-selfcheck.json", {"exit": sc.returncode, "summary": m.group(0) if m else None,
                                   "output_tail": sc_text.splitlines()[-3:]})

    # 4) dry-run 回显（command / reference_tag / captured_at_env；不消费 env）
    d0 = dry(env=env_base())
    o0 = json.loads(d0.stdout) if d0.returncode == 0 else None
    cmd = (o0 or {}).get("command", {})
    echo_ok = bool(o0) and isinstance(cmd.get("argv"), list) and len(cmd["argv"]) > 0 \
        and bool(str(cmd.get("rendered") or "").strip()) and bool(str(cmd.get("cwd") or "").strip()) \
        and o0.get("reference_tag") == "prep-default-dryrun" \
        and o0.get("captured_at_env") == {"present": False, "non_empty": False} \
        and "captured_at" not in o0 and o0.get("legacy_manifest_entries") == 107 and o0.get("repeat") == 2
    add("dryrun_echoes_command_tag_and_env_state", echo_ok,
        {"exit": d0.returncode, "command": cmd, "reference_tag": (o0 or {}).get("reference_tag"),
         "captured_at_env": (o0 or {}).get("captured_at_env"),
         "legacy": (o0 or {}).get("legacy_manifest_entries")})
    dv = dry(env=env_base({"MJWARP_CAPTURE_AT": "2026-11-07T13:00:00+08:00"}))
    ov = json.loads(dv.stdout) if dv.returncode == 0 else {}
    di = dry(env=env_base({"MJWARP_CAPTURE_AT": "not-a-time"}))
    oi = json.loads(di.stdout) if di.returncode == 0 else {}
    dt = dry(["--reference-tag", "tag-x"], env=env_base())
    ot = json.loads(dt.stdout) if dt.returncode == 0 else {}
    env_echo_ok = (dv.returncode == 0 and ov.get("captured_at_env") == {"present": True, "non_empty": True}
                   and "captured_at" not in ov
                   and di.returncode == 0 and oi.get("captured_at_env") == {"present": True, "non_empty": True}
                   and dt.returncode == 0 and ot.get("reference_tag") == "tag-x")
    add("dryrun_env_echo_and_reference_tag_override", env_echo_ok,
        {"valid_env_echo": ov.get("captured_at_env"), "invalid_env_echo": oi.get("captured_at_env"),
         "override_tag": ot.get("reference_tag")})
    dump("dryrun-echo.json", {"default": {"exit": d0.returncode, "command": cmd,
                                          "reference_tag": (o0 or {}).get("reference_tag"),
                                          "captured_at_env": (o0 or {}).get("captured_at_env")},
                              "valid_env": {"exit": dv.returncode, "captured_at_env": ov.get("captured_at_env")},
                              "invalid_env": {"exit": di.returncode, "captured_at_env": oi.get("captured_at_env")},
                              "override": {"exit": dt.returncode, "reference_tag": ot.get("reference_tag")}})

    # 5) 负例：capture 路径在 GPU 之前早失败（缺/空/非 ISO env），out_dir 不创建
    neg_args = ["--scene", "fixtures/render-bvh/scene-a-light/scene.xml",
                "--out-root", "fixtures/render-bvh/scene-a-light/perworld", "--out-dir", NEG_OUT_DIR]
    negs = {}
    for name, env in [("missing", env_base()),
                      ("empty", env_base({"MJWARP_CAPTURE_AT": ""})),
                      ("not_iso", env_base({"MJWARP_CAPTURE_AT": "not-a-time"}))]:
        import time
        t0 = time.time()
        r = run_script(neg_args, env)
        elapsed = time.time() - t0
        msg = (r.stdout + r.stderr).decode("utf-8", "replace")
        exists = os.path.exists(NEG_OUT_DIR)
        negs[name] = {"exit": r.returncode, "elapsed_s": round(elapsed, 3), "out_dir_created": exists,
                      "message": msg.strip().splitlines()[-1] if msg.strip() else "",
                      "gpu_strings": [s for s in ["CUDA", "cuda", "warp device", "wp.get_device"] if s in msg]}
    neg_ok = (negs["missing"]["exit"] == 1 and "is missing" in negs["missing"]["message"]
              and negs["empty"]["exit"] == 1 and "is empty" in negs["empty"]["message"]
              and negs["not_iso"]["exit"] == 1 and "not ISO8601" in negs["not_iso"]["message"]
              and not any(v["out_dir_created"] for v in negs.values())
              and not any(v["gpu_strings"] for v in negs.values())
              and all(v["elapsed_s"] < 5 for v in negs.values()))
    add("capture_path_negative_cases_exit_before_gpu", neg_ok, negs)
    dump("negative-cases.json", negs)

    # 6) 采集元数据接口（CPU 直调模块，覆盖 capture.json 顶层形状）
    sys.path.insert(0, os.path.join(os.getcwd(), "scripts"))
    import perworld_capture_meta as pcm  # noqa: E402
    md = pcm.capture_metadata(["py", "scripts/capture_render_perworld.py", "--dry-run"],
                              os.getcwd(), None, NEG_OUT_DIR, env={"MJWARP_CAPTURE_AT": "2026-11-07T13:00:00+08:00"})
    synth = {**md, "recipe": {"task_id": pcm.TASK_ID}}
    pcm.verify_capture_metadata(synth)
    iface = {
        "keys": sorted(md.keys()),
        "command_nonempty": bool(md["command"]["argv"]) and bool(md["command"]["rendered"]),
        "captured_at": md["captured_at"],
        "reference_tag": md["reference_tag"],
        "task_id": md["task_id"],
        "task_id_source": md["task_id_source"],
        "scope_covers_light_and_geom": "per-world light poses" in md["per_world_scope"]["covers"]
        and "geom poses" in md["per_world_scope"]["covers"],
        "scope_has_103_premise": all(k in md["per_world_scope"]["covers"]
                                      for k in ["host-only", "1-D", "worldid", ".cu unchanged", "§10.3"]),
        "scope_does_not_cover": md["per_world_scope"]["does_not_cover"],
        "verify_passes": True,
    }
    add("capture_metadata_interface_complete",
        iface["command_nonempty"] and iface["captured_at"] == "2026-11-07T13:00:00+08:00"
        and iface["reference_tag"] == "nw3-light-hinge02040" and iface["task_id"] == pcm.TASK_ID
        and iface["task_id_source"] == "script-constant" and iface["scope_covers_light_and_geom"]
        and iface["scope_has_103_premise"] and iface["verify_passes"],
        iface)
    # 负例：verify 拒绝空 command / 旧 scope 文案（接口面）
    def expect_fail(body):
        try:
            body()
            return False
        except SystemExit:
            return True
    verify_neg = {
        "empty_command_rejected": expect_fail(lambda: pcm.verify_capture_metadata(
            {**synth, "command": {"argv": [], "rendered": ""}})),
        "old_scope_rejected": expect_fail(lambda: pcm.verify_capture_metadata(
            {**synth, "per_world_scope": {"covers": pcm.SCOPE_COVERS, "does_not_cover": pcm.OLD_SCOPE_DOES_NOT_COVER}})),
        "empty_captured_at_rejected": expect_fail(lambda: pcm.verify_capture_metadata({**synth, "captured_at": ""})),
    }
    add("capture_metadata_verify_negative_cases", all(verify_neg.values()), verify_neg)
    dump("capture-metadata-interface.json", {"interface": iface, "verify_negative": verify_neg})

    # 7) 默认行为：同树旧脚本 vs 新脚本逐叶 diff
    cur = read(SCRIPT)
    old = run("git", "cat-file", "blob", f"{BASE}:{SCRIPT}").stdout
    restored = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old)
        r_old = dry(env=env_base())
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored = read(SCRIPT) == cur
    r_new = dry(env=env_base())
    if r_old.returncode == 0 and r_new.returncode == 0:
        bo, bn = leaf_map(json.loads(r_old.stdout)), leaf_map(json.loads(r_new.stdout))
        added = sorted(set(bn) - set(bo))
        removed = sorted(set(bo) - set(bn))
        changed = sorted(k for k in set(bo) & set(bn) if bo[k] != bn[k])
        expected_added = {"/command/argv", "/command/cwd", "/command/rendered",
                          "/reference_tag", "/captured_at_env/present", "/captured_at_env/non_empty"}
        expected_changed = {"/recipe/script/sha256", "/recipe/script/bytes"}
        d = {"added": added, "removed": removed, "changed": changed,
             "legacy": [bo.get("/legacy_manifest_entries"), bn.get("/legacy_manifest_entries")],
             "repeat": [bo.get("/repeat"), bn.get("/repeat")]}
        ok = (set(added) == expected_added and removed == [] and set(changed) == expected_changed
              and bo.get("/legacy_manifest_entries") == bn.get("/legacy_manifest_entries") == 107
              and bo.get("/repeat") == bn.get("/repeat") == 2)
    else:
        d, ok = None, False
    add("default_behavior_only_allowed_diffs", ok and restored, {"diffs": d, "script_restored": restored})
    dump("default-behavior-diff.json", {"same_tree": True, "old_script_sha256": sha(old),
                                        "new_script_sha256": sha(cur), "script_restored": restored, "diffs": d})

    # 8) 被复核证据日志与本次复跑一致（按证据记录的 backslash 命令行复刻）
    ev_before = json.load(open(os.path.join(EVID, "logs/before-default-dryrun.json"), encoding="utf-8"))
    ev_after = json.load(open(os.path.join(EVID, "logs/after-default-dryrun.json"), encoding="utf-8"))
    def dmap(x, y):
        return {"added": sorted(set(y) - set(x)), "removed": sorted(set(x) - set(y)),
                "changed": sorted(k for k in set(x) & set(y) if x[k] != y[k])}
    repr_script = "scripts\\capture_render_perworld.py"
    repr_args = ["--dry-run", "--out-dir", "fixtures\\render-bvh\\scene-a\\perworld\\prep-default-dryrun"]
    restored2 = False
    try:
        with open(SCRIPT, "wb") as f:
            f.write(old)
        r_old2 = run(sys.executable, "-B", repr_script, *repr_args, env=env_base())
    finally:
        with open(SCRIPT, "wb") as f:
            f.write(cur)
        restored2 = read(SCRIPT) == cur
    r_new2 = run(sys.executable, "-B", repr_script, *repr_args, env=env_base())
    if r_old2.returncode == 0 and r_new2.returncode == 0:
        logs_after = dmap(leaf_map(json.loads(r_new2.stdout)), leaf_map(ev_after))
        logs_before = dmap(leaf_map(json.loads(r_old2.stdout)), leaf_map(ev_before))
        logs_ok = logs_after == {"added": [], "removed": [], "changed": []} and \
            logs_before == {"added": [], "removed": [], "changed": []}
    else:
        logs_after = logs_before = None
        logs_ok = False
    add("evidence_dryrun_logs_reproduced", logs_ok and restored2,
        {"after_diff": logs_after, "before_diff": logs_before, "script_restored": restored2})
    dump("evidence-logs-reproduced.json", {"after_diff": logs_after, "before_diff": logs_before})

    # 9) 证据自核：manifest 16 条/17 文件、编码、diff artifact、negative artifact、合成标记
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
    add("evidence_manifest_16_entries_17_files",
        len(mrows) == 16 and len(files) == 17 and all(rows_ok.values())
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
    neg_art = json.load(open(os.path.join(EVID, "artifacts/negative-cases.json"), encoding="utf-8"))
    synth_text = open(os.path.join(EVID, "artifacts/synthetic-capture-metadata.json"), encoding="utf-8").read()
    synth_lines = synth_text.splitlines()
    synth_header = synth_lines[0] if synth_lines else ""
    synth_art = json.loads("\n".join(synth_lines[1:]))
    neg_cases = {c["case"]: c for c in neg_art.get("cases", [])}
    art_ok = (diff_art.get("unexpected_count") == 0 and len(diff_art.get("entries", [])) == 5
              and {e["path"] for e in diff_art["entries"]} ==
              {"/captured_at_env", "/command", "/reference_tag", "/recipe/script/sha256", "/recipe/script/bytes"}
              and neg_cases.get("missing", {}).get("returncode") == 1
              and neg_cases.get("empty", {}).get("returncode") == 1
              and synth_header.upper().startswith("SYNTHETIC")
              and all(k in synth_art for k in ["command", "captured_at", "reference_tag",
                                               "per_world_scope", "task_id", "task_id_source"]))
    add("evidence_artifacts_consistent", art_ok,
        {"diff_entries": [e["path"] for e in diff_art.get("entries", [])],
         "unexpected": diff_art.get("unexpected_count"),
         "negatives": {k: neg_cases.get(k, {}).get("returncode") for k in ["missing", "empty"]},
         "synthetic_marked": synth_header.upper().startswith("SYNTHETIC"),
         "synthetic_header": synth_header[:60]})
    dump("evidence-selfcheck.json", {"manifest_entries": len(mrows), "files": len(files),
                                     "rows_ok": rows_ok, "encoding_bad": enc_bad,
                                     "diff_unexpected": diff_art.get("unexpected_count"),
                                     "negative_returns": {k: neg_cases.get(k, {}).get("returncode") for k in ["missing", "empty"]},
                                     "synthetic_marked": synth_header.upper().startswith("SYNTHETIC")})

    # 10a) 9275e6e6 证据层修正（Arch 旁证 O1/O2/O4）
    readme = open(os.path.join(EVID, "README.md"), encoding="utf-8").read()
    receipt2 = json.load(open(os.path.join(EVID, "receipt.json"), encoding="utf-8"))
    cat = receipt2["prerequisites"]["captured_at"]
    fix_changed = sorted(run("git", "diff", "--name-only", FIX_BASE, REVIEW_BASE).stdout.decode().splitlines())
    expected_fix = sorted({CARD} | {f".normify/runs/2026-11-07-render-perworld-l-v2-capture-meta/{g}.json"
                                   for g in ["fmt", "clippy", "clippy-cuda", "unit"]} |
                          {EVID + "README.md", EVID + "manifest.txt", EVID + "receipt.json"})
    rec_missing = cat["negative_missing"]["elapsed_s"]
    rec_empty = cat["negative_empty"]["elapsed_s"]
    log_missing = open(os.path.join(EVID, "logs/negative-captured-at-missing.log"), encoding="utf-8").read()
    log_empty = open(os.path.join(EVID, "logs/negative-captured-at-empty.log"), encoding="utf-8").read()
    lm = re.search(r"elapsed_s:\s*([0-9.]+)", log_missing)
    le = re.search(r"elapsed_s:\s*([0-9.]+)", log_empty)
    log_s = [float(lm.group(1)) if lm else None, float(le.group(1)) if le else None]
    fix = {
        "fix_scope_only_evidence_gates_card": fix_changed == expected_fix,
        "fix_changed": fix_changed,
        "o1_receipt_timings_match_logs": rec_missing == log_s[0] and rec_empty == log_s[1],
        "o1_readme_timings_match_logs": f"{log_s[0]:.3f} s / {log_s[1]:.3f} s" in readme,
        "o1_stale_timings_absent": "0.075" not in readme and "0.111" not in readme
        and "0.075" not in json.dumps(cat) and "0.111" not in json.dumps(cat),
        "o2_typo_fixed": "**显式非空环境变量**" in readme and "显式非空 JSON" not in readme,
        "o4_known_item_added": "已知项（Arch 旁证 O4）" in readme and "command.argv" in readme
        and "规范化" in readme,
    }
    add("fix_commit_9275e6e6_three_observations", all(fix.values()),
        {"checks": fix, "receipt_timings": [rec_missing, rec_empty], "log_timings": log_s})
    dump("fix-commit.json", fix)

    # 10) 实现与 spec v2 §11.1 对齐（静态检查）
    src = read(SCRIPT).decode("utf-8")
    mod = read(MODULE).decode("utf-8")
    spec = read(SPEC).decode("utf-8")
    prov_pos = src.index("provenance = None if args.dry_run")
    prov_next = [l.strip() for l in src[prov_pos:].splitlines()[1:5] if l.strip()][:1]
    gaps = {
        "spec_has_11_1_four_items": all(k in spec for k in ["`command` 字段", "MJWARP_CAPTURE_AT", "per_world_scope", "reference_tag"]),
        "command_before_warp_import": prov_next == ["import warp as wp"],
        "capture_spreads_provenance": "{**provenance" in src or "**provenance," in src,
        "verify_called_after_write": src.index("verify_capture_metadata(capture)") > src.index('(out_dir / "capture.json").write_text'),
        "dry_run_uses_echo_only": "dry_run_echo(" in src and "None if args.dry_run else capture_metadata" in src,
        "task_id_kept": 'TASK_ID = "2026-10-30-render-perworld-capture-authoring"' in mod
        and 'TASK_ID_SOURCE = "script-constant"' in mod,
        "reference_tag_cli": "--reference-tag" in src,
        "old_scope_wording_defined_for_rejection": 'OLD_SCOPE_DOES_NOT_COVER = "per-world light poses' in mod,
        "spec_103_premise_in_scope": "§10.3" in mod and "host-only" in mod,
    }
    add("implementation_matches_spec_v2_11_1", all(gaps.values()), gaps)
    dump("spec-alignment.json", gaps)

    # 11) 被复核卡与 gate
    card = json.load(open(CARD, encoding="utf-8"))
    unit = json.load(open(os.path.join(".normify/runs/2026-11-07-render-perworld-l-v2-capture-meta/unit.json"),
                          encoding="utf-8"))
    gates = [c.get("gate") for c in card["checks"] if c.get("kind") == "gate"]
    add("reviewed_card_verified_unit_1359_0",
        card["status"] == "verified" and card["revision"]["before"] == "262c034a066a538134c716281b6cd14761e01115"
        and gates == ["fmt", "clippy", "clippy-cuda", "unit"]
        and unit["tests"]["passed"] == 1359 and unit["tests"]["failed"] == 0,
        {"status": card["status"], "revision_before": card["revision"]["before"], "gates": gates,
         "unit": unit["tests"]})
    dump("reviewed-card.json", {"status": card["status"], "revision_before": card["revision"]["before"],
                                "gates": gates, "unit_tests": unit["tests"]})
    addenda = card.get("addenda", [])
    add("reviewed_card_fix_addenda_recorded",
        len(addenda) == 1 and "O1/O2/O4" in addenda[0].get("reason", "")
        and "0.079/0.078" in addenda[0].get("reason", "")
        and str(addenda[0].get("hash", "")).startswith("sha256:"),
        {"addenda_count": len(addenda), "reason": addenda[0].get("reason", "")[:160] if addenda else None})

    passed = sum(1 for c in checks if c["ok"])
    total = len(checks)
    summary = {"conclusion": "PASS" if passed == total else "FAIL", "passed": passed, "total": total,
               "blocking": 0 if passed == total else total - passed, "target_head": REVIEW_BASE, "checks": checks}
    with open(os.path.join(logs, "verify-capture-meta-review.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("\nSUMMARY {} {}/{}".format(summary["conclusion"], passed, total))
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
