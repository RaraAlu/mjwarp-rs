"""R09-S3b 阴影数值验收窗口独立复核（QA；只读 + 复算）。

复核对象：`2026-10-28-render-bvh-r09-s3b-shadow-validation`（evidence `7ce747fd` + closure `2d05433f`，
最终树 HEAD 415006d8）。本脚本独立重算，不引用 Backend/Arch 报告值：

  1. 证据完整性：52 文件、LF 无 BOM、manifest 51 条对 committed blob、无自引用、禁词 0；
  2. r2 阴影判据（从 r2 实体日志逐行核）：shadow-on 两相机 rgb/depth/seg 全 0、两次 sha、shadow-off 双重回归、
     可见性 4745==4745；
  3. r2 窗口事实（从 samples.csv / monitor CSV / facts / baseline / guard.log 独立重算）：采样数、peak、
     free_min、util_max、streak、三口径 delta、sustained、interval median/min/max、floor、body wall、
     attempt 记账、零编译（deps 快照）、守卫 56/56、窗后 9/9；
  4. 三次记账：attempt1 红项数字、attempt2 仪器红（exit_ok 空 + 2 条 failures）、r2 转绿；
  5. 编码根因与修复（从留档旧 runner 与修复版 runner 独立复算）：非 ASCII 字节、GBK/UTF-8 行数、`$exitOk`
     被吞、code-only 相同、新 runner ASCII-only；并**独立离线回放** attempt2 冻结日志的两条 S2b 用例判定；
  6. 证据副本口径：windows 副本 vs target 原件（36 逐字节相同 + 2 个 guard_aggregate EOL 归一化）、
     attempt1/attempt2 原始产物 sha；
  7. 判据/参考/红线：capture-exact 冻结 sha 与 shadow.rs 常量、test 源码只引用 capture-exact、`capture/` 未进断言、
     `git diff 415006d8..HEAD` 对红线路径为空、行数与计数。

说明：target/ 为瞬态目录（原始产物只读引用，不改不重跑窗口）。
"""
import hashlib
import json
import pathlib
import re
import subprocess
from datetime import datetime

import ctypes

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/render-bvh/r09-s3b-check"
VAL = REPO / "docs/development-evidence/render-bvh/r09-s3b-shadow-validation"
WIN = REPO / "target/s3b-validation-window"
BASE = "415006d8"
FINAL = "446fcb03"  # 勘误收口 + 门禁刷新（被复核树 2d05433f + 勘误 b20d754c/854ac0b4 + 446fcb03）

result = {"task_id": "2026-10-28-render-bvh-r09-s3b-check", "head": FINAL, "checks": {}, "problems": []}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def note(k, v):
    result["checks"][k] = v
    print(k, "=", json.dumps(v, ensure_ascii=False) if not isinstance(v, str) else v)


def fail(m):
    result["problems"].append(m)
    print("[FAIL]", m)


def blob(rel):
    return subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=REPO, capture_output=True).stdout


# 1. 证据完整性
files = sorted(p for p in VAL.rglob("*") if p.is_file())
note("validation_evidence_file_count", len(files))
if len(files) != 53:
    fail(f"证据目录文件数应为 53（含顶层 manifest），实测 {len(files)}")
eol = {}
for p in files:
    b = p.read_bytes()
    eol[p.relative_to(VAL).as_posix()] = {"bytes": len(b), "cr": b.count(b"\r"), "bom": b.startswith(b"\xef\xbb\xbf")}
bad_eol = {k: v for k, v in eol.items() if v["cr"] or v["bom"]}
note("validation_evidence_eol_bad", bad_eol)
for k in bad_eol:
    fail(f"证据文件 EOL/BOM 异常：{k}")

man_lines = (VAL / "manifest.txt").read_text(encoding="utf-8").splitlines()
man = []
for line in man_lines:
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    digest, size, rel = line.split()
    b = blob(
        (VAL.relative_to(REPO).as_posix() + "/" + rel) if (VAL / rel).exists() else rel
    )
    ok = sha(b) == digest and len(b) == int(size)
    man.append({"path": rel, "bytes": len(b), "claimed": int(size), "ok": ok})
    if not ok:
        fail(f"manifest 不符：{rel}")
note("validation_manifest_entries", len(man))
if len(man) != 52:
    fail(f"勘误后 manifest 条目数应为 52，实测 {len(man)}")
note("validation_manifest_all_ok", all(m["ok"] for m in man))
note("validation_manifest_self_reference", any(m["path"] == "manifest.txt" for m in man))
if any(m["path"] == "manifest.txt" for m in man):
    fail("manifest 自引用")

banned = ["".join(chr(c) for c in s) for s in (
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (31934, 24230, 36798, 26631), (21407, 29983, 25918, 34892), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807),
)]
hits = []
for p in files:
    t = p.read_text(encoding="utf-8", errors="replace")
    for w in banned:
        if w in t:
            hits.append({"path": p.relative_to(VAL).as_posix(), "term": w})
note("validation_banned_hits", hits)
for h in hits:
    fail(f"禁词命中 {h['term']} @ {h['path']}")

# 2. r2 阴影判据（实体日志）
r2 = VAL / "windows/attempt2-r2"
log = (r2 / "logs/render_shadow_reference.log").read_text(encoding="utf-8", errors="replace")
criteria = {
    "shadow_on_cam0_rgb_zero": "r09-s3b[shadow-on] cam0 rgb: pixels=76800 mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log,
    "shadow_on_cam1_rgb_zero": "r09-s3b[shadow-on] cam1 rgb: pixels=43200 mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log,
    "shadow_on_cam0_depth_seg_zero": "cam0 depth: compared=76800 mismatch=0 max_ulp=0 beyond_ulp=0 | seg_mismatch=0" in log,
    "shadow_on_cam1_depth_seg_zero": "cam1 depth: compared=43200 mismatch=0 max_ulp=0 beyond_ulp=0 | seg_mismatch=0" in log,
    "shadow_on_two_run_sha": "run sha256: rgb=f044f3fa1ead8cad0bf84ee49805422ba655ba9840ea55f835d00d92bcdfce9f depth=de732643aa3e9afb2a477b259dd1128139929c29ba7d8935d23096d767e42445 seg=065caf08c1759c34be2b9d9e2b2f8d62e874f219d8d6bf93c8d9e2408b293821" in log,
    "shadow_off_capture_exact_zero": "cam0 rgb(exact): mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log and "cam1 rgb(exact): mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log,
    "shadow_off_pre_unlock_zero": "cam0 pre-unlock: depth_mismatch=0 max_ulp=0 seg_mismatch=0" in log
    and "cam1 pre-unlock: depth_mismatch=0 max_ulp=0 seg_mismatch=0" in log
    and "cam0 pre-unlock rgb: mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log
    and "cam1 pre-unlock rgb: mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in log,
    "shadow_off_two_run_sha": "run sha256: rgb=af9df3317858f58460bca1ba7d99aaa49bb4fb2d26c600c4c5c5a94b695f5f8a" in log,
    "visibility_4745_eq_4745": "product: rgb_diff=4745 depth_diff=0 seg_diff=0 | reference: rgb_diff=4745 depth_diff=0 seg_diff=0" in log,
    "all_three_gpu_cases_pass": "test result: ok. 3 passed; 0 failed; 0 ignored" in log,
}
note("r2_criteria", criteria)
for k, v in criteria.items():
    if not v:
        fail(f"r2 判据缺失：{k}")

# 3. r2 窗口事实（独立重算）
facts_text = (r2 / "facts.txt").read_text(encoding="utf-8")
facts = {}
for line in facts_text.splitlines():
    if line.startswith(("entity ", "shadows_acceptance_")):
        continue
    for m in re.finditer(r"(\w+)=([^\s,]+)", line):
        facts[m.group(1)] = m.group(2)
facts["peak_seq"] = re.search(r"^used_peak_mib=\d+ peak_seq=(\d+)", facts_text, re.M).group(1)
rows = [l.split(",") for l in (r2 / "samples.csv").read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
samples = [
    {"seq": int(r[0]), "ts": datetime.fromisoformat(r[1]), "used": int(r[2]), "free": int(r[3]), "util": int(r[4])}
    for r in rows
]
uses = [s["used"] for s in samples]
intervals = [
    (samples[i]["ts"] - samples[i - 1]["ts"]).total_seconds() for i in range(1, len(samples))
]
intervals_sorted = sorted(intervals)
median = intervals_sorted[len(intervals_sorted) // 2]
streak = best = 0
for s in samples:
    streak = streak + 1 if s["util"] >= 80 else 0
    best = max(best, streak)
baseline_lines = (r2 / "baseline.txt").read_text(encoding="utf-8").splitlines()
base = {k: int(v) for k, v in (kv.split("=") for kv in baseline_lines[0].split())}
guard_log = (r2 / "guard.log").read_text(encoding="utf-8")
m_pre = re.search(r"open=(\d+)/(\d+)/(\d+)/(\d+)", guard_log)
r2_facts = {
    "samples": len(samples),
    "used_peak_mib": max(uses),
    "peak_seq": samples[uses.index(max(uses))]["seq"],
    "used_min_mib": min(uses),
    "free_min_mib": min(s["free"] for s in samples),
    "util_max_percent": max(s["util"] for s in samples),
    "util_streak_ge_80": best,
    "delta_vs_guard_baseline_mib": max(uses) - int(m_pre.group(1)),
    "delta_vs_first_sample_mib": max(uses) - uses[0],
    "delta_vs_sample_min_mib": max(uses) - min(uses),
    "sustained_delta_mib": max(uses) - int(m_pre.group(1)),
    "interval_median_s": round(median, 4),
    "interval_min_s": round(min(intervals), 4),
    "interval_max_s": round(max(intervals), 4),
    "baseline": base,
    "floor_mib": min(10240, base["free_mib"] - 3072),
    "guard_log_ok": "GUARDS_OK=56/56" in guard_log,
    "window_run_done": "WINDOW_RUN_DONE" in guard_log,
}
note("r2_window_recomputed", r2_facts)
expected = {
    "samples": 161, "used_peak_mib": 2822, "peak_seq": 5, "used_min_mib": 2548, "free_min_mib": 13240,
    "util_max_percent": 11, "util_streak_ge_80": 0, "delta_vs_guard_baseline_mib": 270,
    "delta_vs_first_sample_mib": 86, "delta_vs_sample_min_mib": 274, "sustained_delta_mib": 270,
    "floor_mib": 10240,
}
for k, v in expected.items():
    if r2_facts[k] != v:
        fail(f"r2 窗口事实不符：{k} 实测 {r2_facts[k]} 期望 {v}")
if abs(r2_facts["interval_median_s"] - 0.299) > 0.002:
    fail(f"r2 interval median 不符：{r2_facts['interval_median_s']}")

# facts 与重算交叉
facts_vs = {
    "samples": (int(facts["samples"]), r2_facts["samples"]),
    "used_peak_mib": (int(facts["used_peak_mib"]), r2_facts["used_peak_mib"]),
    "peak_seq": (int(facts["peak_seq"]), r2_facts["peak_seq"]),
    "used_min_mib": (int(facts["used_min_mib"]), r2_facts["used_min_mib"]),
    "free_min_mib": (int(facts["free_min_mib"]), r2_facts["free_min_mib"]),
    "util_max_percent": (int(facts["util_max_percent"]), r2_facts["util_max_percent"]),
    "util_streak_ge_80": (int(facts["util_streak_ge_80"]), r2_facts["util_streak_ge_80"]),
    "delta_vs_guard_baseline_mib": (int(facts["delta_vs_guard_baseline_mib"]), r2_facts["delta_vs_guard_baseline_mib"]),
    "delta_vs_first_sample_mib": (int(facts["delta_vs_first_sample_mib"]), r2_facts["delta_vs_first_sample_mib"]),
    "delta_vs_sample_min_mib": (int(facts["delta_vs_sample_min_mib"]), r2_facts["delta_vs_sample_min_mib"]),
    "sustained_delta_mib": (int(facts["sustained_delta_mib"]), r2_facts["sustained_delta_mib"]),
    "floor_mib": (int(facts["floor_mib"]), r2_facts["floor_mib"]),
}
note("r2_facts_vs_recomputed", {k: {"facts": a, "recomputed": b, "match": a == b} for k, (a, b) in facts_vs.items()})
for k, (a, b) in facts_vs.items():
    if a != b:
        fail(f"r2 facts 与重算不一致：{k} {a} != {b}")

# 4. 9 实体 exit 0 + body wall + attempt 记账
entity_lines = (r2 / "entity-results.csv").read_text(encoding="utf-8").splitlines()[1:]
exits = {}
for line in entity_lines:
    if not line.strip():
        continue
    parts = line.split(",")
    exits[parts[0]] = int(parts[2])
note("r2_entity_exits", exits)
if any(v != 0 for v in exits.values()) or len(exits) != 9:
    fail(f"r2 实体退出码异常：{exits}")
t_start = datetime.fromisoformat(facts["window_start"])
t_end = datetime.fromisoformat(facts["window_end"])
t_ent = datetime.fromisoformat(facts["entity_end"])
t_ent0 = datetime.fromisoformat(facts["entity_start"])
body = (t_ent - t_ent0).total_seconds()
note("r2_body_wall_s", round(body, 3))
if abs(body - 48.1) > 0.2:
    fail(f"r2 body wall 不符：{body}")
note("r2_attempt_valid_rounds", int(facts["attempt_valid_rounds"]))

if facts["deps_before_count"] != facts["deps_after_count"] or facts["deps_before_max_write"] != facts["deps_after_max_write"]:
    fail("r2 facts deps 快照前后不一致")
if facts["zero_rust_compile"] != "True":
    fail("r2 zero_rust_compile 不是 True")

# 采样监测 CSV（窗口前）
mon = [l.split(",") for l in (VAL / "windows/monitor-pre-window-attempt2-r2.csv").read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
mon_util = [int(r[3]) for r in mon]
mon_free = [int(r[2]) for r in mon]
mon_streak = [int(r[9]) for r in mon]
mon_build = sum(int(r[7]) for r in mon)
mon_clean = sum(1 for r in mon if r[8].strip() == "True")
mon_facts = {
    "samples": len(mon),
    "util_min": min(mon_util),
    "util_max": max(mon_util),
    "free_min": min(mon_free),
    "longest_streak": max(mon_streak),
    "last_streak": mon_streak[-1],
    "build_sum": mon_build,
    "clean_rows": mon_clean,
    "span": f"{mon[0][0]} -> {mon[-1][0]}",
}
note("r2_pre_window_monitor", mon_facts)
if mon_facts["samples"] != 60 or mon_facts["longest_streak"] != 60 or mon_facts["util_max"] > 4 or mon_build != 0:
    fail(f"r2 预窗监测不符：{mon_facts}")

# 5. 守卫 56/56（自行重哈希）
guards = []
for line in (r2 / "sha-guards.txt").read_text(encoding="utf-8").splitlines():
    if not line.startswith("SHA_OK"):
        continue
    _, digest, size, rel = line.split()
    p = REPO / rel
    if not p.exists():
        guards.append({"path": rel, "ok": False, "reason": "missing"})
        continue
    b = p.read_bytes()
    guards.append({"path": rel, "bytes": len(b), "claimed": int(size), "ok": sha(b) == digest and len(b) == int(size)})
note("r2_guards_recomputed", {"entries": len(guards), "all_ok": all(g["ok"] for g in guards), "bad": [g for g in guards if not g["ok"]]})
if len(guards) != 56 or not all(g["ok"] for g in guards):
    fail(f"守卫重算不符：{len(guards)} 条，bad={[g for g in guards if not g['ok']]}")

# 6. 两版 runner 编码根因与修复（留档旧版 + 修复版；Windows ANSI(936) 语义用 API 复现）
old_runner = WIN / "runner-history/run-window-attempt1-pre-ascii-53dc05ed.ps1"
new_runner = WIN / "run-window-attempt1.ps1"
k32 = ctypes.windll.kernel32


def ansi936_decode(b):
    """Windows MultiByteToWideChar(CP936)：与 PS 5.1 按 ANSI 读 .ps1 的解码语义一致。"""
    n = k32.MultiByteToWideChar(936, 0, b, len(b), None, 0)
    buf = ctypes.create_unicode_buffer(n)
    k32.MultiByteToWideChar(936, 0, b, len(b), buf, n)
    return buf.value


enc = {}
if old_runner.exists() and new_runner.exists():
    for tag, path in (("old", old_runner), ("new", new_runner)):
        b = path.read_bytes()
        ansi = ansi936_decode(b)
        utf8 = b.decode("utf-8", errors="replace")
        code_utf8 = [l for l in utf8.splitlines() if l.strip() and not l.strip().startswith("#")]

        def exitok_line(text):
            for i, l in enumerate(text.splitlines(), 1):
                if "$exitOk" in l:
                    return {"line": i, "starts_with_comment": l.strip().startswith("#"), "stripped": l.strip()[:60].encode("unicode_escape").decode("ascii")}
            return None

        enc[tag] = {
            "path": path.relative_to(REPO).as_posix(),
            "bytes": len(b),
            "sha256": sha(b),
            "non_ascii_bytes": sum(1 for x in b if x >= 128),
            "ansi_splitlines": len(ansi.splitlines()),
            "ansi_split_nl_elements": len(ansi.split("\n")),
            "utf8_splitlines": len(utf8.splitlines()),
            "utf8_split_nl_elements": len(utf8.split("\n")),
            "exitOk_under_ansi": exitok_line(ansi),
            "code_only_lines": len(code_utf8),
            "comment_with_dollar": sum(1 for l in utf8.splitlines() if re.match(r"^\s*#.*\$", l)),
        }
    enc["code_only_equal"] = enc["old"]["code_only_lines"] == enc["new"]["code_only_lines"]
    enc["old_bytes_matches_claim"] = enc["old"]["bytes"] == 16458 and enc["old"]["sha256"] == "53dc05ed01853d0042deeaf8e5f9cabe03413c34f0e56b9ab4b5ebba45057709"
    enc["new_bytes_matches_claim"] = enc["new"]["bytes"] == 16454 and enc["new"]["sha256"] == "8a73ce53e87f6b90e2a1f78eb54fac2a1958a28462b17224ce91d71bdecab926"
else:
    fail(f"runner 留档缺失：old={old_runner.exists()} new={new_runner.exists()}")
note("runner_encoding", enc)
if enc:
    if enc["old"]["non_ascii_bytes"] != 129:
        fail(f"旧 runner 非 ASCII 字节 实测 {enc['old']['non_ascii_bytes']} 期望 129")
    if not (enc["old"]["ansi_split_nl_elements"] == 342 and enc["old"]["utf8_split_nl_elements"] == 343):
        fail(f"旧 runner elements 行数不符 ANSI={enc['old']['ansi_split_nl_elements']} UTF8={enc['old']['utf8_split_nl_elements']}")
    if not (enc["old"]["ansi_splitlines"] == 341 and enc["old"]["utf8_splitlines"] == 342):
        fail(f"旧 runner splitlines 不符 ANSI={enc['old']['ansi_splitlines']} UTF8={enc['old']['utf8_splitlines']}")
    if not enc["old"]["exitOk_under_ansi"] or not enc["old"]["exitOk_under_ansi"]["starts_with_comment"]:
        fail("旧 runner 的 $exitOk 在 ANSI(936) 解码下未被并入注释行")
    if enc["new"]["non_ascii_bytes"] != 0:
        fail(f"新 runner 非 ASCII-only：{enc['new']['non_ascii_bytes']}")
    if not (
        enc["new"]["ansi_splitlines"] == enc["new"]["utf8_splitlines"]
        and enc["new"]["ansi_split_nl_elements"] == enc["new"]["utf8_split_nl_elements"]
    ):
        fail("新 runner ANSI 与 UTF-8 行数不一致")
    if not enc["new"]["exitOk_under_ansi"] or enc["new"]["exitOk_under_ansi"]["starts_with_comment"]:
        fail("新 runner 的 $exitOk 仍是注释行")
    if enc["new"]["comment_with_dollar"] != 0:
        fail("新 runner 仍存在注释行含 $")
    if not enc["code_only_equal"]:
        fail("runner 修复前后 code-only 行不一致")

# 7. 独立离线回放：用修复版四条件（另含多行正则）判定 attempt2 冻结日志
def replay(log_path, case, marker):
    text = log_path.read_text(encoding="utf-8", errors="replace")
    exit_ok = True  # attempt2 facts: entity exit=0
    m = re.search(rf"(?s)^test\s+{re.escape(case)}\s+\.\.\..*?\n\s*ok\b", text, re.M)
    name_found = bool(m)
    status_ok = bool(re.search(rf"^test\s+{re.escape(case)}\s+\.\.\.", text, re.M))
    return {
        "exit_ok": exit_ok,
        "name_found": name_found,
        "status_ok": status_ok,
        "marker_ok": marker in text,
        "all_true": bool(exit_ok and name_found and status_ok and marker in text),
    }


a2 = VAL / "windows/attempt2/logs"
replay_result = {
    "render_kernel_flags": replay(a2 / "render_kernel_flags.log", "aa_supported_and_remaining_rejections_pre_launch", "AA + shadows supported"),
    "render_kernel_rgb": replay(a2 / "render_kernel_rgb.log", "unsupported_configurations_are_rejected_before_launch", "shadows supported; precomputed=false/skybox rejected"),
}
note("offline_replay_attempt2_logs", replay_result)
for k, v in replay_result.items():
    if not v["all_true"]:
        fail(f"离线回放未全 True：{k} {v}")

# attempt2 仪器红与 r2 对比
attempt2_facts = (VAL / "windows/attempt2/facts.txt").read_text(encoding="utf-8")
note("attempt2_instrument_red", {
    "exit_ok_empty": "shadows_acceptance_render_kernel_flags exit_ok= name_found=True" in attempt2_facts
    and "shadows_acceptance_render_kernel_rgb exit_ok= name_found=True" in attempt2_facts,
    "failures": [l for l in attempt2_facts.splitlines() if l.startswith("failures=")],
    "samples": int(re.search(r"^samples=(\d+)$", attempt2_facts, re.M).group(1)),
    "no_window_run_done": "WINDOW_RUN_DONE" not in (VAL / "windows/attempt2/guard.log").read_text(encoding="utf-8"),
    "all_entities_exit_zero": all(int(l.split(",")[2]) == 0 for l in (VAL / "windows/attempt2/entity-results.csv").read_text(encoding="utf-8").splitlines()[1:] if l.strip()),
})
if attempt2_facts.count("exit_ok= ") == 0 or "shadows_acceptance_missing" not in attempt2_facts:
    fail("attempt2 仪器红事实不符")

# 8. 证据副本口径：windows vs target 原件
def compare_tree(win_dir, raw_dir):
    same = diff = 0
    details = []
    for p in sorted(win_dir.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(win_dir)
        raw = raw_dir / rel
        if not raw.exists():
            details.append({"path": rel.as_posix(), "reason": "target missing"})
            diff += 1
            continue
        a, b = p.read_bytes(), raw.read_bytes()
        if a == b:
            same += 1
        else:
            a_n = a.replace(b"\r\n", b"\n")
            b_n = b.replace(b"\r\n", b"\n")
            if a_n == b_n:
                details.append({"path": rel.as_posix(), "reason": "eol-normalized", "target_sha": sha(b), "target_bytes": len(b), "copy_sha": sha(a), "copy_bytes": len(a)})
                diff += 1
            else:
                details.append({"path": rel.as_posix(), "reason": "content differs", "target_sha": sha(b), "copy_sha": sha(a)})
                diff += 1
    return {"identical": same, "differing": diff, "details": details}


copies = {
    "attempt2": compare_tree(VAL / "windows/attempt2", WIN / "attempt2"),
    "attempt2-r2": compare_tree(VAL / "windows/attempt2-r2", WIN / "attempt2-r2"),
}
note("windows_vs_target", copies)

# 9. attempt1 原始产物（target 只读）：实际 sha 与 README/receipt 声称值对照
att1 = WIN / "attempt1"
a2raw = WIN / "attempt2"
att1_check = {
    "file_count": len([p for p in att1.rglob("*") if p.is_file()]),
    "facts": {"sha": sha((att1 / "facts.txt").read_bytes()), "bytes": (att1 / "facts.txt").stat().st_size},
    "entity_results": {"sha": sha((att1 / "entity-results.csv").read_bytes()), "bytes": (att1 / "entity-results.csv").stat().st_size},
    "guard_log": {"sha": sha((att1 / "guard.log").read_bytes()), "bytes": (att1 / "guard.log").stat().st_size},
}
note("attempt1_raw_actual", att1_check)
# 勘误后：README §5.1 / receipt.attempt1.raw_products 应为 attempt1 真值
att1_true = {k: att1_check[k]["sha"] for k in ("facts", "entity_results", "guard_log")}
att2_hashes = {
    "facts": sha((a2raw / "facts.txt").read_bytes()),
    "entity_results": sha((a2raw / "entity-results.csv").read_bytes()),
    "guard_log": sha((a2raw / "guard.log").read_bytes()),
}
readme_txt = (VAL / "README.md").read_text(encoding="utf-8")
receipt_txt = (VAL / "receipt.json").read_text(encoding="utf-8")
manifest_txt = (VAL / "manifest.txt").read_text(encoding="utf-8")
att1_true_check = {k: readme_txt.count(v[:16]) for k, v in att1_true.items()}
note("attempt1_true_sha_in_readme", att1_true_check)
if any(v < 1 for v in att1_true_check.values()):
    fail(f"README 未包含 attempt1 真值 sha：{att1_true_check}")
# 分区检查：README 勘误小节之外（< 勘误标题行）不得再出现 attempt2 错配值写进 attempt1 语境
readme_lines = readme_txt.splitlines()
errata_start = next((i for i, l in enumerate(readme_lines) if "证据勘误" in l and l.lstrip().startswith("#")), len(readme_lines))
pre_errata = "\n".join(readme_lines[:errata_start])
wrong_in_pre = {k: pre_errata.count(v[:16]) for k, v in att2_hashes.items()}
note("attempt2_sha_in_readme_before_errata_section", wrong_in_pre)
# 勘误小节内允许作为历史记录出现
wrong_after = {k: "\n".join(readme_lines[errata_start:]).count(v[:16]) for k, v in att2_hashes.items()}
note("attempt2_sha_in_readme_errata_section", wrong_after)
# receipt：attempt1 语境不得写 attempt2 值；errata 字段允许历史记录
receipt_obj = json.loads(receipt_txt)
raw_products = receipt_obj.get("rounds", {}).get("attempt1", {}).get("raw_products", "")
note("receipt_attempt1_raw_products", raw_products[:220])
if any(v[:16] in raw_products for v in att2_hashes.values()):
    fail("receipt attempt1.raw_products 仍含 attempt2 值")
if any(v[:16] not in raw_products for v in att1_true.values()):
    fail(f"receipt attempt1.raw_products 未含 attempt1 真值：{att1_true}")
errata_field = json.dumps(receipt_obj.get("errata", {}), ensure_ascii=False)
note("receipt_errata_field_present", bool(errata_field and errata_field != "{}"))
# manifest：attempt2 值只出现在 windows/attempt2/ 行；attempt1 值不应出现在 manifest
man_sha_attribution = []
for line in manifest_txt.splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    digest, size, rel = line.split()
    for tag, group in (("attempt1", att1_true), ("attempt2", att2_hashes)):
        for key, v in group.items():
            if digest == v:
                man_sha_attribution.append({"path": rel, "attribution": tag, "key": key})
note("manifest_sha_attribution", man_sha_attribution)
for a in man_sha_attribution:
    if a["attribution"] == "attempt1":
        fail(f"manifest 出现 attempt1 产物 sha：{a}")
    if a["attribution"] == "attempt2" and not a["path"].startswith("windows/attempt2/"):
        fail(f"manifest attempt2 sha 行前缀不对：{a}")
if not any(a["attribution"] == "attempt2" for a in man_sha_attribution):
    fail("manifest 未出现 windows/attempt2 三条应有 sha")

# attempt1 红项数字（从原始日志复算）
att1_log = (att1 / "logs/render_shadow_reference.log").read_text(encoding="utf-8", errors="replace")
m_att1 = re.search(r"cam1 rgb: pixels=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", att1_log)
vis1 = re.search(r"product: rgb_diff=(\d+) depth_diff=(\d+) seg_diff=(\d+) \| reference: rgb_diff=(\d+)", att1_log)
att1_numbers = {
    "cam1_mismatched": int(m_att1.group(2)),
    "cam1_max_byte_diff": int(m_att1.group(3)),
    "cam1_beyond_1_lsb": int(m_att1.group(4)),
    "product_shadow_pixels": int(vis1.group(1)),
    "reference_shadow_pixels": int(vis1.group(4)),
    "provenance": "target/s3b-validation-window/attempt1/logs/render_shadow_reference.log",
}
note("attempt1_red_numbers", att1_numbers)
if (att1_numbers["cam1_mismatched"], att1_numbers["cam1_beyond_1_lsb"], att1_numbers["cam1_max_byte_diff"]) != (1451, 1436, 77):
    fail(f"attempt1 红项数字与 README 不符：{att1_numbers}")
if (att1_numbers["product_shadow_pixels"], att1_numbers["reference_shadow_pixels"]) != (3537, 4745):
    fail(f"attempt1 可见性对照与 README 不符：{att1_numbers}")

# 9.1 材料冻结：deps 快照 + 9 exe 冻结表（对守卫/现文件复核）
ff = WIN / "attempt2-build/final-freeze"
deps_txt = (ff / "deps-snapshots.txt").read_text(encoding="utf-8", errors="replace")
snap_lines = [l for l in deps_txt.splitlines() if l.startswith("SNAPSHOT")]
snap_parsed = []
for line in snap_lines:
    m = re.search(
        r"SNAPSHOT time=(?P<t>\S+) deps_count=(?P<c>\d+) deps_max_write=(?P<m>\S+) build_procs=(?P<b>\d+) gpu=(?P<g>.+)$",
        line,
    )
    if m:
        snap_parsed.append({"time": m.group("t"), "count": int(m.group("c")), "max_write": m.group("m"),
                            "build_procs": int(m.group("b")), "gpu": m.group("g")})
last_two = snap_parsed[-2:]
def snap_interval(a, b):
    return (datetime.fromisoformat(b["time"]) - datetime.fromisoformat(a["time"])).total_seconds()

snap = {
    "snapshots": snap_parsed,
    "last_two_identical_count": last_two[0]["count"] == last_two[1]["count"],
    "last_two_identical_max_write": last_two[0]["max_write"] == last_two[1]["max_write"],
    "last_two_max_write_is_20_26_42_825": last_two[1]["max_write"].startswith("2026-10-28T20:26:42.8251842"),
    "interval_s": round(snap_interval(last_two[0], last_two[1]), 3),
    "build_procs_zero": all(s["build_procs"] == 0 for s in snap_parsed),
}
note("deps_snapshots", snap)
if not (snap["last_two_identical_count"] and snap["last_two_identical_max_write"] and snap["last_two_max_write_is_20_26_42_825"]
        and abs(snap["interval_s"] - 62.113) < 0.01 and snap["build_procs_zero"] and last_two[1]["count"] == 1826):
    fail(f"deps 快照与 README §5.3 不符：{snap}")
# 9 个冻结 exe：按 exes.txt（entity|sha|bytes|path）复核现盘字节
exes_txt = (ff / "exes.txt").read_text(encoding="utf-8", errors="replace")
frozen_files = []
for line in exes_txt.splitlines():
    if "|" not in line:
        continue
    entity, digest, size, rel = line.split("|")
    p = REPO / rel
    if p.exists():
        b = p.read_bytes()
        frozen_files.append({"entity": entity, "path": rel, "bytes": len(b), "ok": sha(b) == digest and len(b) == int(size)})
    else:
        frozen_files.append({"entity": entity, "path": rel, "ok": False, "reason": "missing"})
note("frozen_exes_recheck", {
    "total": len(frozen_files),
    "match": sum(1 for f in frozen_files if f.get("ok")),
    "bad": [f for f in frozen_files if not f.get("ok")],
})
if len(frozen_files) != 9 or any(not f.get("ok") for f in frozen_files):
    fail(f"9 个冻结 exe 现盘字节不符：{frozen_files}")


# 10. 参考/判据/红线
cap = REPO / "fixtures/render-bvh/scene-shadow/capture-exact"
npz_sha = {f"{g}/{n}": sha((cap / g / f"{n}.npz").read_bytes()) for g in ("shadow-on", "shadow-off") for n in ("rgb", "depth", "seg")}
shadow_rs = (REPO / "tests/support/render_bvh/shadow.rs").read_text(encoding="utf-8")
frozen = re.findall(r'"([0-9a-f]{64})"', shadow_rs[:4000])
note("reference_npz_sha", npz_sha)
note("shadow_rs_frozen_equal", frozen[:6] == list(npz_sha.values()))
if frozen[:6] != list(npz_sha.values()):
    fail("冻结参考 sha 与 shadow.rs 常量不符")

test_src = (REPO / "tests/render_shadow_reference.rs").read_text(encoding="utf-8")
shadow_rs = (REPO / "tests/support/render_bvh/shadow.rs").read_text(encoding="utf-8")


def code_lines_of(text):
    """剥离 // 与 //! 注释行后的代码行（用于判断 capture/ 是否进入断言）。"""
    out = []
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("//"):
            continue
        out.append(s)
    return "\n".join(out)


capture_teeth = re.search(r"capture(?!-exact)/", code_lines_of(test_src)) or re.search(
    r"capture(?!-exact)/", code_lines_of(shadow_rs)
)
note("legacy_capture_referenced_in_code", bool(capture_teeth))
if capture_teeth:
    fail("验收测试/解析器代码引用了 capture/（fast-math 组）")
note(
    "capture_exact_include_bytes_count",
    len(re.findall(r'include_bytes!\("\.\./\.\./\.\./fixtures/render-bvh/scene-shadow/capture-exact/', shadow_rs)),
)

diff = subprocess.run(
    ["git", "diff", "--numstat", f"{BASE}..{FINAL}", "--", "src", "tests", "fixtures", "Cargo.toml", "Cargo.lock",
     "docs/development-evidence/render-bvh/r09-s3b-check"],
    cwd=REPO, capture_output=True, text=True, encoding="utf-8",
).stdout
note("code_test_fixture_diff_since_base", diff.strip())
if diff.strip():
    fail(f"最终树对 src/tests/fixtures/Cargo 有改动：{diff.strip()}")
line_counts = {}
for rel, claim in {
    "src/render/kernel/cuda/render_common.cu": 393,
    "src/render/kernel/cuda/render_megakernel.cu": 340,
    "src/render/kernel/cuda/render_shadow.cu": 30,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "tests/render_shadow_reference.rs": 307,
    "tests/support/render_bvh/shadow.rs": 416,
    "tests/support/render_bvh/shadow_scene.rs": 238,
    "tests/render_e2e.rs": 500,
}.items():
    n = (REPO / rel).read_bytes().count(b"\n")
    line_counts[rel] = {"lf": n, "claimed": claim, "match": n == claim}
    if n != claim:
        fail(f"行数不符：{rel} {n} != {claim}")
note("line_counts", line_counts)
device = re.search(r"DEVICE_SOURCES: \[\(&str, &str\); (\d+)\]", (REPO / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8"))
note("collision_DEVICE_SOURCES", int(device.group(1)) if device else None)

# 参考夹具未变（自 S3a 捕获提交起）
cap_diff = subprocess.run(
    ["git", "diff", "--numstat", "93f983e5..HEAD", "--", "fixtures/render-bvh/scene-shadow/capture-exact",
     "fixtures/render-bvh/scene-shadow/capture", "fixtures/render-bvh/scene-shadow/scene.xml"],
    cwd=REPO, capture_output=True, text=True, encoding="utf-8",
).stdout
note("reference_fixture_diff_since_capture_commit", cap_diff.strip())
if cap_diff.strip():
    fail(f"参考/夹具自 S3a 捕获后有改动：{cap_diff}")

# 10.5 attempt2（仪器红轮）窗口事实独立重算（用于与 r2 对比）
a2rows = [l.split(",") for l in (VAL / "windows/attempt2/samples.csv").read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
a2s = [
    {"seq": int(r[0]), "ts": datetime.fromisoformat(r[1]), "used": int(r[2]), "free": int(r[3]), "util": int(r[4])}
    for r in a2rows
]
a2uses = [s["used"] for s in a2s]
a2inter = sorted((a2s[i]["ts"] - a2s[i - 1]["ts"]).total_seconds() for i in range(1, len(a2s)))
a2base_used = int(re.search(r"open=(\d+)/", (VAL / "windows/attempt2/guard.log").read_text(encoding="utf-8")).group(1))
a2_recomputed = {
    "samples": len(a2s),
    "used_peak_mib": max(a2uses),
    "peak_seq": a2s[a2uses.index(max(a2uses))]["seq"],
    "free_min_mib": min(s["free"] for s in a2s),
    "util_max_percent": max(s["util"] for s in a2s),
    "delta_vs_guard_baseline_mib": max(a2uses) - a2base_used,
    "delta_vs_first_sample_mib": max(a2uses) - a2uses[0],
    "delta_vs_sample_min_mib": max(a2uses) - min(a2uses),
    "interval_median_s": round(a2inter[len(a2inter) // 2], 4),
}
note("attempt2_window_recomputed", a2_recomputed)
for k, v in {"samples": 160, "used_peak_mib": 2849, "peak_seq": 5, "free_min_mib": 13213, "util_max_percent": 13,
             "delta_vs_guard_baseline_mib": 270, "delta_vs_first_sample_mib": 260, "delta_vs_sample_min_mib": 270,
             "interval_median_s": 0.2991}.items():
    if a2_recomputed[k] != v:
        fail(f"attempt2 窗口事实不符：{k} 实测 {a2_recomputed[k]} 期望 {v}")

# 11. README 自述数字与原始事实对照（含勘误后文字）
readme = (VAL / "README.md").read_text(encoding="utf-8")
attempt2_samples = int(re.search(r"^samples=(\d+)$", attempt2_facts, re.M).group(1))
attempt2_monitor = [l.split(",") for l in (VAL / "windows/monitor-pre-window-attempt2.csv").read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
readme_checks = {
    "attempt2_section_claims_160_samples": "0.25 s 采样 **160** 样本" in readme,
    "attempt2_facts_samples": attempt2_samples,
    "attempt2_samples_csv_rows": len((VAL / "windows/attempt2/samples.csv").read_text(encoding="utf-8").splitlines()) - 1,
    "r2_section_claims_161_samples": "0.25 s 采样 **161** 样本（peak 2 822" in readme or "0.25 s 采样 **161** 样本（peak 2 822、used_min 2 548" in readme,
    "attempt2_body_47_8_in_readme": "body **47.8 s**" in readme,
    "attempt2_body_from_facts": float(re.search(r"^body_wall_s=([\d.]+)$", attempt2_facts, re.M).group(1)),
    "attempt2_monitor_samples": len(attempt2_monitor),
    "attempt2_monitor_util_range": [min(int(r[3]) for r in attempt2_monitor), max(int(r[3]) for r in attempt2_monitor)],
    "attempt2_monitor_free_min": min(int(r[2]) for r in attempt2_monitor),
    "attempt2_monitor_span": f"{attempt2_monitor[0][0]} -> {attempt2_monitor[-1][0]}",
    "exit10_provenance_text": ("调用方" in readme) and ("返回码" in readme) and ("不可从产物" in readme or "无法从产物" in readme),
    "deps_two_chosen_and_first_discarded": all(s in readme for s in ("20:32:58", "20:34:00", "62.113", "20:31:29", "21%")),
    "deps_three_raw_lines_preserved": readme.count("deps_count=1826") >= 3 or "deps-snapshots.txt" in readme,
}
note("readme_vs_raw", readme_checks)
if not readme_checks["attempt2_section_claims_160_samples"]:
    fail("README §5.2 未改为 attempt2 160 样本")
if attempt2_samples != readme_checks["attempt2_samples_csv_rows"]:
    fail(f"attempt2 facts {attempt2_samples} 与 csv {readme_checks['attempt2_samples_csv_rows']} 行不符")
if not readme_checks["r2_section_claims_161_samples"]:
    fail("README §5.4/§5.2 的 r2 161 样本表述缺失")
if not readme_checks["exit10_provenance_text"]:
    fail("README 未写明 runner exit 10 的 provenance")
if not readme_checks["deps_two_chosen_and_first_discarded"]:
    fail("README §5.3 未写明 deps 快照取后两条/首条舍去口径")
if readme_checks["attempt2_monitor_samples"] != 60:
    fail(f"attempt2 预窗监测样本数不符：{readme_checks['attempt2_monitor_samples']}")

# 11.1 勘误清单抽查：erratum-grep-report.md 与独立遍历归属
report = VAL / "erratum-grep-report.md"
erratum = {
    "report_exists": report.exists(),
    "report_sha": sha(report.read_bytes()) if report.exists() else None,
    "report_bytes": report.stat().st_size if report.exists() else None,
    "report_in_manifest": any(
        l.strip().endswith("erratum-grep-report.md") and (report.exists() and l.split()[0] == sha(report.read_bytes()))
        for l in manifest_txt.splitlines() if l.strip() and not l.startswith("#")
    ),
}
six = {**{f"attempt1_{k}": v for k, v in att1_true.items()}, **{f"attempt2_{k}": v for k, v in att2_hashes.items()}}
occurrences = []
for p in sorted(VAL.rglob("*")):
    if not p.is_file() or p.suffix in (".npz", ".exe"):
        continue
    for lineno, line in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        for tag, digest in six.items():
            if digest[:16] in line:
                occurrences.append({"file": p.relative_to(VAL).as_posix(), "line": lineno, "sha": tag})
erratum["occurrences"] = occurrences
note("erratum_grep_self_check", erratum)
if not erratum["report_exists"] or not erratum["report_in_manifest"]:
    fail(f"erratum-grep-report.md 缺失或未按 sha 入 manifest：{erratum}")
# 归属检查：attempt1 sha 出现在 manifest 即错；attempt2 sha 出现在 README/receipt 只允许勘误语境
for o in occurrences:
    if o["file"] == "manifest.txt" and o["sha"].startswith("attempt1_"):
        fail(f"manifest 出现 attempt1 sha：{o}")
# 独立统计：修复后不应再有「attempt2 sha 写进 attempt1 语境」
bad_ctx = [
    o for o in occurrences
    if o["file"] == "README.md" and o["line"] < errata_start and o["sha"].startswith("attempt2_")
]
if bad_ctx:
    fail(f"README 勘误小节前仍有 attempt2 sha：{bad_ctx}")
note("erratum_grep_counts", {"occurrences_total": len(occurrences), "bad_context": bad_ctx})

# 12. r2 两条 S2b 解锁断言的日志级复核（四条件原样）
r2_logs = r2 / "logs"
s2b = {}
for entity, case, marker in (
    ("render_kernel_flags", "aa_supported_and_remaining_rejections_pre_launch", "AA + shadows supported"),
    ("render_kernel_rgb", "unsupported_configurations_are_rejected_before_launch", "shadows supported; precomputed=false/skybox rejected"),
):
    text = (r2_logs / f"{entity}.log").read_text(encoding="utf-8", errors="replace")
    s2b[entity] = {
        "exit_ok": f"entity {entity} exit=0" in facts_text,
        "name_found": bool(re.search(rf"^test\s+{re.escape(case)}\s+\.\.\.", text, re.M)),
        "status_ok": bool(re.search(rf"(?s)^test\s+{re.escape(case)}\s+\.\.\..*?\n\s*ok\b", text, re.M)),
        "marker_ok": marker in text,
        "facts_line": next((l for l in facts_text.splitlines() if l.startswith(f"shadows_acceptance_{entity}")), None),
    }
note("r2_s2b_acceptance", s2b)
for entity, v in s2b.items():
    if not all(v[k] for k in ("exit_ok", "name_found", "status_ok", "marker_ok")):
        fail(f"r2 S2b 四条件未全 True：{entity} {v}")
    if v["facts_line"] and "exit_ok=True name_found=True status_ok=True marker_ok=True" not in v["facts_line"]:
        fail(f"r2 facts 四条件行不符：{v['facts_line']}")


# 13. 勘误提交范围 + 两卡门禁记录（只读）
err_scope = subprocess.run(["git","diff","--numstat","2d05433f..b20d754c"],cwd=REPO,capture_output=True,text=True,encoding="utf-8").stdout
err_code = subprocess.run(["git","diff","--numstat","2d05433f..854ac0b4","--","src","tests","fixtures","Cargo.toml","Cargo.lock"],cwd=REPO,capture_output=True,text=True,encoding="utf-8").stdout
erratum_scope = {"b20d754c_files": [l.split("\t")[2] for l in err_scope.splitlines() if l.strip()], "code_test_fixture_diff": err_code.strip()}
note("erratum_scope", erratum_scope)
if len(erratum_scope["b20d754c_files"]) != 5 or err_code.strip():
    fail(f"勘误提交范围不符：{erratum_scope}")
gate_records = {}
for task in ("2026-10-28-render-bvh-r09-s3b-shadow-validation","2026-10-28-render-bvh-r09-s3b-evidence-erratum"):
    runs = REPO/".normify/runs"/task
    rec = {g: json.loads((runs/f"{g}.json").read_text(encoding="utf-8")) for g in ("fmt","clippy","clippy-cuda","unit") if (runs/f"{g}.json").exists()}
    gate_records[task] = {g: {"exit": v.get("exit"), "passed": v.get("tests",{}).get("passed"), "failed": v.get("tests",{}).get("failed")} for g,v in rec.items()}
note("gate_records", gate_records)
for task,rec in gate_records.items():
    if not rec or any(v["exit"] != 0 for v in rec.values()) or rec.get("unit",{}).get("passed") != 1050 or rec.get("unit",{}).get("failed") != 0:
        fail(f"{task} 门禁记录不符：{rec}")
val_sizes = {n: (VAL/n).stat().st_size for n in ("README.md","receipt.json","manifest.txt","erratum-grep-report.md")}
note("validation_evidence_sizes", val_sizes)


# 14. 勘误后 S3b 门禁记录刷新（446fcb03）只读复核
s3b_fix = subprocess.run(["git","diff","--numstat","854ac0b4..446fcb03"],cwd=REPO,capture_output=True,text=True,encoding="utf-8").stdout
s3b_fix_code = subprocess.run(["git","diff","--numstat","854ac0b4..446fcb03","--","src","tests","fixtures","Cargo.toml","Cargo.lock"],cwd=REPO,capture_output=True,text=True,encoding="utf-8").stdout
s3b_fix_files = [l.split("\t")[2] for l in s3b_fix.splitlines() if l.strip()]
note("s3b_gate_refresh_commit", {"files": s3b_fix_files, "code_test_fixture_diff": s3b_fix_code.strip()})
if len(s3b_fix_files) != 5 or s3b_fix_code.strip():
    fail(f"446fcb03 范围不符：{s3b_fix_files}")
refreshed = {}
runs = REPO/".normify/runs/2026-10-28-render-bvh-r09-s3b-shadow-validation"
for g in ("fmt","clippy","clippy-cuda","unit"):
    d = json.loads((runs/f"{g}.json").read_text(encoding="utf-8"))
    refreshed[g] = {"exit": d.get("exit"), "at": d.get("at"), "passed": d.get("tests",{}).get("passed"), "failed": d.get("tests",{}).get("failed")}
note("s3b_gate_records_after_refresh", refreshed)
if any(v["exit"] != 0 for v in refreshed.values()) or refreshed["unit"]["passed"] != 1050 or refreshed["unit"]["failed"] != 0:
    fail(f"S3b 刷新后门禁记录不符：{refreshed}")
if refreshed["unit"]["at"] <= "2026-10-28T13:12":
    fail("S3b 门禁记录未在勘误后重跑（at 未更新）")

note("problems", result["problems"])
print("problems =", len(result["problems"]))
EVID.mkdir(parents=True, exist_ok=True)
(EVID / "recompute.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
