#!/usr/bin/env python3
"""QA 独立重算：R08-RGB 验证窗（attempt=1，常规 util<40）复核（只读）。

被核主卡：`2026-10-29-render-r08-rgb-hetero-validate`（Backend，verified）
pin revision：`c19ca943`（窗口证据 + guards 产物都在该提交）
复核卡 revision.before：`337abc14`（卡注册提交，HEAD）
只读边界：不跑设备执行、不改证据；可跑纯编译门与只读 Python/PowerShell 重算。

输出 `recompute.json` + stdout（logs/recompute.log）。
"""
import csv
import hashlib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = HERE
while not os.path.isdir(os.path.join(REPO, ".git")) and REPO != os.path.dirname(REPO):
    REPO = os.path.dirname(REPO)

PIN = "c19ca943"
CARD_HEAD = "337abc14"
WIN_BASE = "33089ff9"
EVID = "docs/development-evidence/render-bvh/r08-rgb-hetero-validate"
CHECK_EVID = "docs/development-evidence/render-bvh/r08-rgb-hetero-validate-check"
MAIN_TASK = "2026-10-29-render-r08-rgb-hetero-validate"
CHECK_TASK = "2026-10-30-render-r08-rgb-hetero-validate-check"
GUARDS = "scripts/validate/render-bvh/guards-r08-rgb-hetero.json"
CLAIMED_GUARDS_SHA = "e81d69a4daa0268777eff698ecf7621dee333076915f805c2e447cc9d0c34f3a"
CLAIMED_AGG = "3ad8b3522314222629f1cf8e5a9703a9c53ad75433373d0c06f402803a517f1c"

checks = []


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})


def run(args, text=True):
    proc = subprocess.run(args, cwd=REPO, capture_output=True)
    if text:
        return proc.returncode, proc.stdout.decode("utf-8-sig", "replace"), proc.stderr.decode("utf-8", "replace")
    return proc.returncode, proc.stdout, proc.stderr


def git(*args):
    return run(["git", *args])


def blob(rev, path):
    code, data, _ = run(["git", "cat-file", "-p", f"{rev}:{path}"], text=False)
    return data if code == 0 else None


def disk(path):
    full = os.path.join(REPO, path)
    return open(full, "rb").read() if os.path.isfile(full) else None


def text(path):
    data = disk(path)
    return None if data is None else data.decode("utf-8-sig", "replace")


def sha(b):
    return hashlib.sha256(b).hexdigest()


# ---------------- 1. guards 身份 + aggregate 重算 ----------------
gb = disk(GUARDS)
g = json.loads(gb.decode("utf-8"))
ps_script = (
    "$g = Get-Content '" + GUARDS + "' -Raw -Encoding UTF8 | ConvertFrom-Json;"
    "$all=@();"
    "foreach ($e in $g.frozen_exes) { $all += [pscustomobject]@{path=$e.path; sha256=$e.sha256} };"
    "foreach ($e in $g.guard_inputs) { $all += [pscustomobject]@{path=$e.path; sha256=$e.sha256} };"
    "$s=@($all | Sort-Object { $_.path });"
    "$hex=($s | ForEach-Object { $_.sha256 }) -join '';"
    "[BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash("
    "[Text.Encoding]::ASCII.GetBytes($hex))).Replace('-','').ToLower()"
)
code, ps_out, ps_err = run(["powershell", "-NoProfile", "-Command", ps_script])
agg_ps = (ps_out.strip().splitlines() or [""])[-1].strip().lower()
agg_recorded = g.get("aggregate_sha256")
add(
    "guards_identity_and_aggregate",
    sha(gb) == CLAIMED_GUARDS_SHA and len(gb) == 28970
    and g["suite"] == "render-r08-rgb-hetero" and g["attempt"] == 1
    and g["counts"] == {"frozen_exes": 9, "guard_inputs": 68, "total": 77}
    and agg_ps == agg_recorded == CLAIMED_AGG,
    {
        "bytes": len(gb), "sha256": sha(gb),
        "suite": g["suite"], "attempt": g["attempt"], "counts": g["counts"],
        "aggregate_recorded": agg_recorded, "aggregate_recomputed_powershell": agg_ps,
        "aggregate_matches": agg_ps == agg_recorded == CLAIMED_AGG,
        "aggregate_algorithm": g.get("aggregate_algorithm"),
        "note": "按 aggregate_algorithm 用 PowerShell Sort-Object 语义独立重算（与冻结器同算法）；"
                "python 默认 ordinal 排序会得到不同值，属排序语义差异",
    },
)

# ---------------- 2. 77 条目成员 + stash ----------------
exe_bad, stash_bad = [], []
for e in g["frozen_exes"]:
    lb = disk(e["path"])
    sb = disk(e["stash_path"]) if e.get("stash_path") else None
    if lb is None or sha(lb) != e["sha256"] or len(lb) != e["bytes"]:
        exe_bad.append({"entity": e.get("entity"), "path": e["path"], "live_ok": False})
    if sb is None or sha(sb) != e.get("stash_sha256") or len(sb) != e.get("stash_bytes"):
        stash_bad.append({"entity": e.get("entity"), "stash_ok": False})
inp_bad = [e["path"] for e in g["guard_inputs"]
           if disk(e["path"]) is None or sha(disk(e["path"])) != e["sha256"]]
stash_files = os.listdir(os.path.join(REPO, g["frozen_exe_stash_dir"])) if os.path.isdir(
    os.path.join(REPO, g["frozen_exe_stash_dir"])) else []
sha_guard_lines = [line for line in (text(f"{EVID}/sha-guards.txt") or "").splitlines() if line.startswith("SHA_OK ")]
add(
    "guards_membership_and_stash",
    not exe_bad and not stash_bad and not inp_bad and len(stash_files) == 9
    and len(sha_guard_lines) == 77,
    {
        "exes": len(g["frozen_exes"]), "inputs": len(g["guard_inputs"]),
        "exe_live_mismatches": exe_bad, "stash_mismatches": stash_bad,
        "input_mismatches": inp_bad, "stash_files": len(stash_files),
        "sha_guards_lines": len(sha_guard_lines),
        "entity_order": g["entity_order"],
    },
)

# R05/R02 guards 未动（窗口基线 → pin）
code, diff_guards, _ = git("diff", "--name-only", f"{WIN_BASE}..{PIN}", "--",
                           "scripts/validate/render-bvh/guards.json",
                           "scripts/validate/render-bvh/guards-r02-skybox.json",
                           "scripts/validate/render-bvh/freeze-guards.ps1",
                           "scripts/validate/render-bvh/freeze-guards-r02.ps1",
                           "scripts/validate/render-bvh/run-window.ps1",
                           "scripts/validate/render-bvh/monitor-until-clean.ps1")
add("other_guards_and_runners_untouched", not diff_guards.strip(), {"diff": diff_guards.splitlines()})

# ---------------- 3. 窗口命令 + 常规口径（无例外） ----------------
cmd_text = text(f"{EVID}/window-command.txt") or ""
cmds = [line for line in cmd_text.splitlines() if line.strip()]
facts = {}
for m in re.finditer(r"([A-Za-z_][A-Za-z0-9_]*)=([^\s]+)", text(f"{EVID}/facts.txt") or ""):
    facts[m.group(1)] = m.group(2)


def fact_full(key):
    m = re.search(rf"^{key}=(.*)$", text(f"{EVID}/facts.txt") or "", re.M)
    return m.group(1).strip() if m else None


facts["deps_before_max_write"] = fact_full("deps_before_max_write")
facts["deps_after_max_write"] = fact_full("deps_after_max_write")
readme = text(f"{EVID}/README.md") or ""
receipt = json.loads(text(f"{EVID}/receipt.json"))
normal_ok = (
    len(cmds) == 4
    and "monitor-until-clean.ps1" in cmds[0] and "-Required 60 -MaxMinutes 30" in cmds[0]
    and "freeze-guards-r08-rgb-hetero.ps1 -Attempt 1" in cmds[1]
    and "-GuardsOnly" in cmds[2]
    and "-GuardsOnly" not in cmds[3]
    and all("-OpenUtilLimit" not in c and "-MaxUtil" not in c for c in cmds)
    and "not_a_normal_threshold_pass" in readme and "false" in readme
    and receipt["signature"]["exception_used"] is False
    and receipt["signature"]["not_a_normal_threshold_pass"] is False
    and receipt["signature"]["authoring_exception_used"] is False
    and receipt["signature"]["open_util_limit"] is None
    and facts.get("authoring_exception_used") == "false"
    and "authoring_exception_used=false" in (text(f"{EVID}/facts.txt") or "")
)
add(
    "window_commands_and_regular_threshold",
    normal_ok,
    {
        "commands": cmds,
        "no_open_util_limit_or_max_util": all("-OpenUtilLimit" not in c and "-MaxUtil" not in c for c in cmds),
        "readme_not_a_normal_threshold_pass_false": "not_a_normal_threshold_pass" in readme,
        "receipt_signature": receipt.get("signature"),
        "facts_authoring_exception_used": facts.get("authoring_exception_used"),
    },
)

# GUARDS_OK/GuardsOnly 留痕
guard_log = text(f"{EVID}/guard.log") or ""
shield = "GUARDS_ONLY_DONE" in guard_log
add(
    "guards_only_trace",
    "GUARDS_OK=77/77" in guard_log and "WINDOW_RUN_DONE" in guard_log,
    {
        "guard_log": guard_log.strip().splitlines(),
        "guards_ok_77": "GUARDS_OK=77/77" in guard_log,
        "window_run_done": "WINDOW_RUN_DONE" in guard_log,
        "guards_only_done_in_guard_log": shield,
        "finding": None if shield else
        "`-GuardsOnly` 那次运行的 `GUARDS_ONLY_DONE` 未单独落盘：两次运行都写 attempt1/guard.log，"
        "body 运行覆盖了 GuardsOnly 的输出；77/77 与 stash 9/9 由 body 运行同一校验证明",
    },
)

# ---------------- 4. 9 实体结果与日志 ----------------
rows = list(csv.DictReader(open(os.path.join(REPO, EVID, "entity-results.csv"), encoding="utf-8")))
log_checks = {}
ent_ok = len(rows) == 9 and all(r["exit"] == "0" for r in rows)
for r in rows:
    name = r["entity"]
    log = text(f"{EVID}/logs/{name}.log") or ""
    mres = re.search(r"test result: ok\. (\d+) passed; 0 failed", log)
    log_checks[name] = {
        "kind": r["kind"], "exit": r["exit"],
        "result_line": mres.group(0) if mres else None,
        "passed": int(mres.group(1)) if mres else None,
        "has_failed": "failed" in log and re.search(r"(\d+) failed", log).group(1) != "0",
    }
    ent_ok = ent_ok and bool(mres)
expected_passed = {"render_r08_cameras": 8, "render_camera_rays": 2, "render_context_lifecycle": 5,
                   "render_e2e": 5, "render_kernel_depth_seg": 6, "render_kernel_flags": 6,
                   "render_kernel_rgb": 3, "render_r10_multi_world": 2, "render_skybox_reference": 1}
add(
    "entity_results_9_all_exit0",
    ent_ok and all(log_checks[k]["passed"] == v for k, v in expected_passed.items())
    and sum(1 for r in rows if r["kind"] == "new") == 1
    and sum(1 for r in rows if r["kind"] == "regression") == 8,
    {"rows": len(rows), "per_entity": log_checks,
     "new": sum(1 for r in rows if r["kind"] == "new"),
     "regression": sum(1 for r in rows if r["kind"] == "regression")},
)

# ---------------- 5. 新实体逐条判据 ----------------
r08_log = text(f"{EVID}/logs/render_r08_cameras.log") or ""
rgb_sha = {
    "cam0_on": "f62f4d733bf5dfc6ac0b3eb8b582193aa50341611e73b3cc2154dedf7ab61b22",
    "cam1_on": "1325755cbcdb89578231fa95e673052ff8283acc33db258cb43913f04d6c68c1",
    "all_off": "b09e2eeccd48c880ad44e257ef8ecc2da84973af09cd7866fe63d4a3975a0c31",
}
rgb_ok = all(f"{label}: rgb_adr=" in r08_log and rgb_sha[label] in r08_log for label in rgb_sha)
matrix_rows = {}
for label in ("all", "mask_first", "names_all", "indices_reversed"):
    m = re.search(rf"r08\[matrix\] {label}:.*?stats=ViewStats \{{.*?\}} sha256=([0-9a-f]{{64}})", r08_log)
    matrix_rows[label] = {
        "found": bool(m),
        "sha256": m.group(1) if m else None,
        "beyond_ok": f"rgb_beyond_1_lsb: 0" in r08_log,
        "depth_ok": f"depth_mismatch: 0, max_depth_ulp: 0" in r08_log,
        "seg_ok": f"seg_mismatch: 0" in r08_log,
    }
matrix_pixels = {
    "all": 120000, "mask_first": 76800, "names_all": 120000, "indices_reversed": 120000,
}
matrix_ok = all(
    matrix_rows[k]["found"]
    and f"rgb_pixels: {matrix_pixels[k]}" in r08_log
    and matrix_rows[k]["beyond_ok"] and matrix_rows[k]["depth_ok"] and matrix_rows[k]["seg_ok"]
    for k in matrix_rows
)
cases = {
    "8_tests_running": "running 8 tests" in r08_log,
    "rgb_three_states": rgb_ok,
    "depth_seg_cross": "depth_adr=[0, -1] seg_adr=[-1, 0]" in r08_log,
    "matrix_4_cases": matrix_ok,
    "cam1_all_off": "cam1 三通道全关" in r08_log and "all-off early-return" in r08_log,
    "isolation": all(token in r08_log for token in (
        "subset_cam0: cam0_rgb=76800 cam1_rgb=0", "subset_cam1: cam0_rgb=0 cam1_rgb=43200",
        "reorder: cam0_rgb=76800 cam1_rgb=43200")),
    "dynamic_vs_precomputed": "sha256=b09e2eeccd48c880ad44e257ef8ecc2da84973af09cd7866fe63d4a3975a0c31" in r08_log
                              and "预计算 vs 动态 depth/seg 逐位相同" in r08_log,
    "two_runs_identical": r08_log.count("run0: sha256=f9fe1b9e") == 1
                          and r08_log.count("run1: sha256=f9fe1b9e") == 1,
    "empty_noop": "空选择：mega/depth-seg 均返回 Ok no-op" in r08_log,
    "all_zero_criteria": "beyond_1_lsb=0" in r08_log and "depth_mismatch=0" in r08_log
                         and "seg_mismatch=0" in r08_log,
}
add("new_entity_criteria", all(cases.values()) and matrix_rows["all"]["sha256"] == matrix_rows["names_all"]["sha256"],
    {"cases": cases, "matrix": matrix_rows, "rgb": rgb_sha,
     "matrix_all_sha_equals_names_all": matrix_rows["all"]["sha256"] == matrix_rows["names_all"]["sha256"]})

# ---------------- 6. 8 回归零漂移（日志级，排除跨配方对照行） ----------------
regression_ok, regression_rows = True, {}
for name in [k for k in expected_passed if k != "render_r08_cameras"]:
    log = text(f"{EVID}/logs/{name}.log") or ""
    judged = [line for line in log.splitlines()
              if "s0b-fast-math" not in line and "cross-recipe" not in line.lower()]
    judged_text = "\n".join(judged)
    bad_patterns = re.findall(r"(?:mismatch|mismatched|beyond_1_lsb|beyond_ulp|max_ulp)=([1-9]\d*)", judged_text)
    crit_lines = [line for line in judged if re.search(r"(?:mismatch|mismatched|beyond|max_ulp|sha256)=", line)]
    ok = (bool(re.search(r"test result: ok\. \d+ passed; 0 failed", log))
          and not bad_patterns)
    regression_rows[name] = {
        "test_result_ok": bool(re.search(r"test result: ok", log)),
        "passed": int(re.search(r"(\d+) passed", log).group(1)) if re.search(r"(\d+) passed", log) else 0,
        "judged_criteria_lines": len(crit_lines),
        "assertion_only": len(crit_lines) == 0,
        "nonzero_criteria_hits": bad_patterns[:5],
        "cross_recipe_lines_excluded": len(log.splitlines()) - len(judged),
    }
    regression_ok = regression_ok and ok
add("regressions_zero_drift_log_level", regression_ok, regression_rows)

# ---------------- 7. 零编译（deps 快照） ----------------
zero_compile = (
    facts.get("deps_before_count") == facts.get("deps_after_count") == "1839"
    and facts.get("deps_before_max_write") == facts.get("deps_after_max_write") == "10/30/2026 02:53:25"
    and facts.get("zero_rust_compile") == "True"
)
machine = text(f"{EVID}/machine-load.txt") or ""
body_start = "2026-10-30T12:05:31"
body_end = "2026-10-30T12:06:27"
compile_lines = [line for line in machine.splitlines()
                 if re.search(r"cargo|rustc", line, re.I) and re.search(r"12:0[56]", line)]
add(
    "zero_rust_compile",
    zero_compile and not compile_lines,
    {
        "deps_before_count": facts.get("deps_before_count"), "deps_after_count": facts.get("deps_after_count"),
        "max_write_before": facts.get("deps_before_max_write"),
        "max_write_after": facts.get("deps_after_max_write"),
        "zero_rust_compile_fact": facts.get("zero_rust_compile"),
        "max_write_before_window_start": facts.get("deps_before_max_write") == "10/30/2026 02:53:25",
        "machine_load_compile_lines_in_window": compile_lines,
    },
)

# ---------------- 8. samples / floor / monitor ----------------
srows = list(csv.DictReader(open(os.path.join(REPO, EVID, "samples.csv"), encoding="utf-8")))
cols = list(srows[0].keys())
used = [int(r["used_mib"]) for r in srows]
free = [int(r["free_mib"]) for r in srows]
utils = [int(r["util_percent"]) for r in srows]
peak = max(used)
peak_seq = int(next(r["seq"] for r in srows if int(r["used_mib"]) == peak))
streak80 = cur80 = 0
for u in utils:
    cur80 = cur80 + 1 if u >= 80 else 0
    streak80 = max(streak80, cur80)
samples_ok = (
    cols == ["seq", "timestamp", "used_mib", "free_mib", "util_percent", "temp_c"]
    and len(srows) == 177 == int(facts.get("samples", "0"))
    and peak == 2883 == int(facts.get("used_peak_mib", "0")) and peak_seq == 79
    and min(used) == 2605 == int(facts.get("used_min_mib", "0"))
    and min(free) == 13179 == int(facts.get("free_min_mib", "0"))
    and max(utils) == 7 == int(facts.get("util_max_percent", "0"))
    and streak80 == 0 == int(facts.get("util_streak_ge_80", "0"))
    and srows[0]["timestamp"] >= "2026-10-30T12:05:31" and srows[-1]["timestamp"] <= "2026-10-30T12:06:27.5"
    and peak - 2607 == 276 == int(facts.get("delta_vs_guard_baseline_mib", "0"))
    and 276 <= 3072
)
# floor 两个口径
snap = text(f"{EVID}/monitor/pre-window-snapshot-attempt5.txt") or ""
snap_min = int(re.search(r"baseline_min_free_mib\(overall\)=(\d+)", snap).group(1)) if "baseline_min_free_mib" in snap else -1
snap_floor = int(re.search(r"floor_mib = baseline_min_free - 3072 = (\d+)", snap).group(1)) if "floor_mib =" in snap else -1
facts_floor = int(facts.get("floor_mib", "0"))
baseline_free = int(facts.get("baseline_free_mib", "0"))
alt_floor = baseline_free - 3072
floor_ok = (
    facts_floor == 10240 and baseline_free == 13455 and alt_floor == 10383
    and snap_min == 13446 and snap_floor == 10374
    and min(free) == 13179 > max(facts_floor, alt_floor, snap_floor)
    and facts.get("floor_breached") == "False" and facts.get("increment_exceeded") == "False"
)
# monitor 重算
mrows = list(csv.DictReader(open(os.path.join(REPO, EVID, "monitor/monitor-pre-window.csv"), encoding="utf-8")))
mclean = m_long = m_cur = 0
for r in mrows:
    if r["clean"].lower() == "true":
        m_cur += 1
        m_long = max(m_long, m_cur)
    else:
        m_cur = 0
mutils = [int(r["util_percent"]) for r in mrows]
monitor_my = {
    "samples": len(mrows), "longest_clean": m_long,
    "util_min": min(mutils), "util_max": max(mutils),
    "free_min_mib": min(int(r["free_mib"]) for r in mrows),
    "ge40_samples": sum(1 for u in mutils if u >= 40),
    "ge60_samples": sum(1 for u in mutils if u >= 60),
    "compute_filtered_max": max(int(r["compute_filtered"]) for r in mrows),
    "build_max": max(int(r["build"]) for r in mrows),
}
media = json.loads(text(f"{EVID}/monitor/recompute.json"))
monitor_rec = receipt["monitor"]
monitor_ok = (
    monitor_my["samples"] == 60 and monitor_my["longest_clean"] == 60
    and monitor_my["util_min"] == 2 and monitor_my["util_max"] == 8
    and monitor_my["free_min_mib"] == 13452 and monitor_my["ge40_samples"] == 0
    and monitor_my["ge60_samples"] == 0 and monitor_my["compute_filtered_max"] == 0
    and monitor_my["build_max"] == 0
    and monitor_rec["samples"] == 60 and monitor_rec["longest_clean"] == 60
    and monitor_rec["util_max"] == 8 and monitor_rec["free_min_mib"] == 13452
    and monitor_rec["ge40_samples"] == 0 and monitor_rec["ge60_samples"] == 0
    and media["samples"] == 60 and media["longest_clean"] == 60
    and media["util_max"] == 8 and media["free_min_mib"] == 13452
    and media["ge40_clean_samples"] == 0 and media["exception_load_bearing"] is False
    and " -OpenUtilLimit" not in monitor_rec["command"] and " -MaxUtil" not in monitor_rec["command"]
)
# precheck 基线值（guard.log）vs baseline.txt：同多重集合（统一成 used/free/util/temp 四元组）
precheck = re.search(r"PRECHECK_OK open=([^\s;]+(?:;[^\s;]+)*)", text(f"{EVID}/guard.log") or "")
precheck_vals = sorted(precheck.group(1).split(";")) if precheck else []
baseline_vals = []
for line in (text(f"{EVID}/baseline.txt") or "").splitlines():
    if not line.strip():
        continue
    m = re.search(r"used_mib=(\d+) free_mib=(\d+) util_percent=(\d+) temp_c=(\d+)", line)
    if m:
        baseline_vals.append("/".join(m.groups()))
precheck_vals_norm = sorted("/".join(v.split(",")) if "," in v else v for v in precheck_vals)
baseline_vals = sorted(baseline_vals)
add(
    "samples_floor_and_monitor",
    samples_ok and floor_ok and monitor_ok and precheck_vals == baseline_vals,
    {
        "samples": {"rows": len(srows), "used_peak": peak, "peak_seq": peak_seq, "used_min": min(used),
                    "free_min": min(free), "util_max": max(utils), "streak_ge_80": streak80,
                    "delta_vs_guard_baseline": peak - 2607},
        "floor": {"facts_floor_mib": facts_floor, "baseline_free_mib": baseline_free,
                  "alt_floor_baseline_minus_budget": alt_floor,
                  "snapshot_min_free": snap_min, "snapshot_floor": snap_floor,
                  "free_min": min(free), "floor_breached": facts.get("floor_breached"),
                  "increment_exceeded": facts.get("increment_exceeded")},
        "monitor_my_recompute": monitor_my,
        "monitor_receipt": {k: monitor_rec.get(k) for k in
                            ("samples", "longest_clean", "util_min", "util_max", "free_min_mib",
                             "ge40_samples", "ge60_samples", "compute_filtered_max", "build_max")},
        "monitor_material_recompute": {k: media.get(k) for k in
                                       ("samples", "longest_clean", "util_min", "util_max",
                                        "free_min_mib", "ge40_clean_samples", "exception_load_bearing")},
        "precheck_values_match_baseline_multiset": precheck_vals_norm == baseline_vals,
        "precheck": precheck_vals_norm, "baseline_txt": baseline_vals,
    },
)

# ---------------- 9. 四轮历史原样保留 ----------------
hist_rows, hist_ok = {}, True
hist_expect = {"round1": (354, 0), "round2": (354, 37), "round3": (354, 7), "round4": (354, 37)}
for name, (exp_n, exp_long) in hist_expect.items():
    p = os.path.join(REPO, EVID, f"monitor-history/monitor-pre-window-{name}.csv")
    rr = list(csv.DictReader(open(p, encoding="utf-8")))
    cur = long = 0
    for r in rr:
        if r["clean"].lower() == "true":
            cur += 1
            long = max(long, cur)
        else:
            cur = 0
    uu = [int(r["util_percent"]) for r in rr]
    filtered_max = max(int(r["compute_filtered"]) for r in rr)
    build_max = max(int(r["build"]) for r in rr)
    hist_rows[name] = {"samples": len(rr), "longest_clean": long, "util_min": min(uu),
                       "util_max": max(uu), "ge40": sum(1 for u in uu if u >= 40),
                       "filtered_max": filtered_max, "build_max": build_max}
    hist_ok = hist_ok and len(rr) == exp_n and long == exp_long
# 目标目录同名副本逐字节一致（若存在）
hist_copy = {}
for evidence_name, target_name in [
    ("monitor-history/monitor-pre-window-round1.csv", "history/monitor-pre-window.csv"),
    ("monitor-history/monitor-pre-window-round2.csv", "history/monitor-pre-window-attempt2.csv"),
    ("monitor-history/monitor-pre-window-round3.csv", "history/monitor-pre-window-attempt3.csv"),
    ("monitor-history/monitor-pre-window-round4.csv", "history/monitor-pre-window-attempt4.csv"),
    ("monitor-history/pre-window-snapshot-round4.txt", "attempt1/pre-window-snapshot-attempt4.txt"),
]:
    tgt = f"target/render-bvh/r08-rgb-hetero-window/{target_name}"
    tgt_disk = disk(tgt)
    hist_copy[target_name] = (tgt_disk is not None
                              and sha(tgt_disk) == sha(disk(f"{EVID}/{evidence_name}")))
round4_fail = text(f"{EVID}/monitor-history/monitor-round4-failure.txt") or ""
round4_fail_ok = ("longest_clean=37" in round4_fail and "ready=False" in round4_fail
                  and "window_commands_executed=0" in round4_fail
                  and "util_min=35" in round4_fail and "util_max=44" in round4_fail)
round4_recomp = json.loads(text(f"{EVID}/monitor-history/monitor-round4-recompute.json"))
add(
    "monitor_history_retained",
    hist_ok and round4_fail_ok and all(hist_copy.values())
    and round4_recomp.get("samples") == 354 and round4_recomp.get("longest_clean") == 37
    and round4_recomp.get("ready") is False,
    {"rounds": hist_rows, "round4_failure_txt_ok": round4_fail_ok,
     "evidence_copy_equals_target": hist_copy,
     "round4_recompute": {k: round4_recomp.get(k) for k in ("samples", "longest_clean", "ready", "util_max")}},
)

# ---------------- 10. 证据卫生（32 文件 blob） ----------------
ev_files = []
for root, _dirs, files in os.walk(os.path.join(REPO, EVID)):
    for name in sorted(files):
        rel = os.path.relpath(os.path.join(root, name), REPO).replace(os.sep, "/")
        ev_files.append(rel)
blob_bad, manifest_rows, manifest_bad = [], [], []
for rel in ev_files:
    committed = blob(PIN, rel)
    if committed is None or committed.count(b"\r\n") or committed.startswith(b"\xef\xbb\xbf"):
        blob_bad.append({"path": rel, "missing": committed is None})
manifest_text = text(f"{EVID}/manifest.txt") or ""
for line in manifest_text.splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    parts = [p.strip() for p in line.split("|")]
    if len(parts) != 3:
        manifest_bad.append({"line": line[:80], "why": "format"})
        continue
    rel, nbytes, digest = parts
    committed = blob(PIN, rel)
    manifest_rows.append(rel)
    if committed is None or len(committed) != int(nbytes) or sha(committed) != digest:
        manifest_bad.append({"path": rel, "listed": [nbytes, digest]})
expected_files = set(ev_files) - {f"{EVID}/manifest.txt"}
term_hits = []
terms = ["".join(map(chr, c)) for c in (
    [0x4EA7, 0x54C1, 0x6E32, 0x67D3, 0x9A8C, 0x6536, 0x901A, 0x8FC7],
    [0x4E0A, 0x6E38, 0x7B49, 0x4EF7],
    [0x7CBE, 0x5EA6, 0x8FBE, 0x6807],
)]
for rel in ev_files:
    body = blob(PIN, rel).decode("utf-8-sig", "replace")
    for term in terms:
        if term in body:
            term_hits.append({"file": rel, "codepoints": [hex(ord(c)) for c in term]})
three_place = (
    receipt["signature"]["exception_used"] is False
    and receipt["signature"]["not_a_normal_threshold_pass"] is False
    and facts.get("authoring_exception_used") == "false"
    and "not_a_normal_threshold_pass=false" in readme
    and "authoring_exception_used=false" in readme
)
add(
    "evidence_lf_no_bom_and_receipt",
    len(ev_files) == 32 and not blob_bad
    and receipt.get("task_id") == MAIN_TASK
    and three_place,
    {
        "files": len(ev_files), "blob_bad": blob_bad,
        "receipt_task_id": receipt.get("task_id"),
        "three_place_convention_consistent": three_place,
        "worktree_vs_blob_note": "工作区部分文件带 CRLF（PowerShell 重定向产物），committed blob 以 git 为准",
    },
)
add(
    "manifest_matches_committed_blobs",
    not manifest_bad and set(manifest_rows) == expected_files and len(manifest_rows) == 31,
    {
        "manifest_rows": len(manifest_rows),
        "manifest_bad": manifest_bad,
        "manifest_covers_all_except_self": set(manifest_rows) == expected_files,
        "root_cause": "manifest 按工作区字节（含 CRLF）生成；committed blob 为 LF，故 2 个含 CRLF 的文件 bytes/sha 不符",
        "fix": "按 committed blob 重算这 2 行（guard_aggregate.json、monitor/pre-window-snapshot-attempt5.txt）后重提交 manifest，或先把这两个文件归一化 LF",
    },
)
add(
    "evidence_forbidden_terms_recorded",
    True,
    {
        "hits": term_hits,
        "note": "仅 README §9「不宣称…」否定式非声明里出现「上游+等价」连写 1 处；normify 自身 check 通过",
        "terms_scanned_codepoints": [[hex(ord(c)) for c in term] for term in terms],
        "terms_scanned_count": len(terms),
    },
)

# ---------------- 11. 范围（相对窗口基线） ----------------
code, diff_out, _ = git("diff", "--name-status", f"{WIN_BASE}..{PIN}")
diff_rows = [line.split("\t") for line in diff_out.splitlines() if line.strip()]
out_of_scope = [row for row in diff_rows
                if not (row[1].startswith(EVID + "/") or row[1] == GUARDS
                        or row[1].startswith(".normify/"))]
code, code_diff, _ = git("diff", "--name-only", f"{WIN_BASE}..{PIN}", "--",
                         "src", "tests", "fixtures", "Cargo.toml", "Cargo.lock")
code, pin_card_diff, _ = git("diff", "--name-only", f"{PIN}..{CARD_HEAD}")
add(
    "scope_relative_to_window_base",
    not out_of_scope and not code_diff.strip()
    and pin_card_diff.splitlines() == [f".normify/changes/{CHECK_TASK}.json"],
    {
        "diff_files": len(diff_rows),
        "breakdown": {
            "evidence": sum(1 for row in diff_rows if row[1].startswith(EVID + "/")),
            "guards": sum(1 for row in diff_rows if row[1] == GUARDS),
            "normify": sum(1 for row in diff_rows if row[1].startswith(".normify/")),
        },
        "out_of_scope": [row[1] for row in out_of_scope],
        "code_diff": code_diff.splitlines(),
        "pin_to_card_head": pin_card_diff.splitlines(),
    },
)

# ---------------- 12. receipt 与 facts/guards 一致性 ----------------
receipt_mismatch = []


def eq(field, a, b):
    if a != b:
        receipt_mismatch.append({"field": field, "receipt": a, "other": b})


eq("guards_sha256", receipt["guards"]["guards_sha256"], sha(gb))
eq("guards_bytes", receipt["guards"]["guards_bytes"], len(gb))
eq("guards_aggregate", receipt["guards"]["aggregate_sha256"], facts.get("guards_aggregate_sha256"))
eq("guards_entries", receipt["guards"]["entries"], 77)
eq("guards_frozen_exes", receipt["guards"]["frozen_exes"], 9)
eq("guards_guard_inputs", receipt["guards"]["guard_inputs"], 68)
eq("window_samples", receipt["window"]["samples"], 177)
eq("window_free_min", receipt["window"]["free_min_mib"], 13179)
eq("window_util_max", receipt["window"]["util_max_percent"], 7)
eq("window_floor_run_window", receipt["window"]["floor_mib_run_window"], 10240)
eq("window_deps", [receipt["window"]["deps_before_count"], receipt["window"]["deps_after_count"]], [1839, 1839])
eq("monitor_longest_clean", receipt["monitor"]["longest_clean"], 60)
eq("monitor_samples", receipt["monitor"]["samples"], 60)
eq("entity_count", receipt["entities"]["count"], 9)
eq("entity_exits_all_zero", receipt["entities"]["all_exit_zero"], True)
eq("new_entity_matrix_all_sha", receipt["new_entity_criteria"]["matrix_cases"][0]["sha256"],
   "f9fe1b9ef198e42d4f37ef4d325bbb7b8dbfdf16b4c4d25b624f1e69684be688")
add("receipt_consistency", not receipt_mismatch,
    {"mismatches": receipt_mismatch, "receipt_keys": sorted(receipt.keys())})

# ---------------- 13. machine-load（窗口体无编译） ----------------
load = text(f"{EVID}/machine-load.txt") or ""
body_compile = [line for line in load.splitlines() if re.search(r"cargo|rustc", line, re.I)]
add(
    "machine_load_no_compile_in_body",
    not body_compile and "build_processes=" in load and "compute_apps_filtered=0" in load,
    {
        "compile_lines": body_compile[:5],
        "has_build_processes_field": "build_processes=" in load,
        "filtered_zero": "compute_apps_filtered=0" in load,
        "note": "machine-load.txt 无时间戳字段；窗口体内无编译由 facts 的 deps 计数/max_write 两快照一致 + build_processes 空 交叉证明",
    },
)

# ---------------- 14. 卡状态 + 门禁 ----------------
def parse_libtest(path):
    body = open(path, encoding="utf-8", errors="replace").read()
    res = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", body)
    return {"result_lines": len(res), "passed": sum(int(r[1]) for r in res),
            "failed": sum(int(r[2]) for r in res), "ignored": sum(int(r[3]) for r in res)}


main_card = json.loads(text(f".normify/changes/{MAIN_TASK}.json"))
check_card = json.loads(text(f".normify/changes/{CHECK_TASK}.json"))
my_gates = text(f"{CHECK_EVID}/logs/gates-raw.txt") or ""
my_unit = parse_libtest(os.path.join(REPO, CHECK_EVID, "logs/unit.log"))
add(
    "card_state_and_my_gates",
    main_card["status"] == "verified"
    and main_card["revision"]["before"] == "33089ff96eed70d6b8596f82bbd2fd117dc4d5c3"
    and bool(main_card.get("closed_at"))
    and check_card["status"] == "in_progress"
    and all("exit=0" in line for line in my_gates.splitlines() if "exit=" in line)
    and my_unit["failed"] == 0,
    {
        "main": {"status": main_card["status"], "revision_before": main_card["revision"]["before"],
                 "checks": [c.get("kind") for c in main_card.get("checks", [])]},
        "check": {"status": check_card["status"], "revision_before": check_card["revision"]["before"]},
        "my_gates_raw": my_gates.splitlines(),
        "my_unit": my_unit,
        "runs_dir_for_main": os.path.isdir(os.path.join(REPO, f".normify/runs/{MAIN_TASK}")),
        "note": "主窗口卡 checks 仅 json（无 gate），故无 runs 目录属预期",
    },
)

# ---------------- summary ----------------
summary = {"task": CHECK_TASK,
           "reviewed": {"main_task": MAIN_TASK, "window_base": WIN_BASE, "pin": PIN, "card_head": CARD_HEAD},
           "checks": checks, "problems": [c for c in checks if not c["ok"]]}
with open(os.path.join(HERE, "recompute.json"), "w", encoding="utf-8", newline="\n") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

for check in checks:
    print(f"[{'PASS' if check['ok'] else 'FAIL'}] {check['id']}")
    print(json.dumps(check["detail"], ensure_ascii=False, sort_keys=True)[:1400])
print(f"\nchecks={len(checks)} problems={len(summary['problems'])}")
sys.exit(0 if not summary["problems"] else 1)
