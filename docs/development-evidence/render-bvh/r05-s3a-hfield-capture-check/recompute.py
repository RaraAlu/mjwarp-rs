#!/usr/bin/env python3
"""R05-S3a hfield 采集窗口独立复核重算脚本（check 卡口径）。

只读复核：签字物 / 8 个捕获文件 / capture.json / 例外留档 / 旧参考 / 窗口事实 /
证据清单。不开 GPU、不重跑采集、不做渲染数值结论。

说明：
- 仓库文件按 committed blob（`git show HEAD:<path>`，LF 归一口径）重算；旧参考基线按
  工作树字节重算（其历史口径是工作树，见观察 O-2）。
- 聚合值按上游口径（PowerShell Sort-Object 路径排序 + 小写 hex 拼接）由本目录
  logs/aggregate-repro.ps1 复现后比对。
"""

import hashlib
import json
import pathlib
import re
import struct
import subprocess
import sys
import zipfile

try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

R = pathlib.Path(
    subprocess.run(["git", "rev-parse", "--show-toplevel"],
                   cwd=pathlib.Path(__file__).resolve().parent,
                   capture_output=True, text=True, check=True).stdout.strip()
)
EV = "docs/development-evidence/render-bvh/r05-s3a-hfield-capture"
CHK = "docs/development-evidence/render-bvh/r05-s3a-hfield-capture-check"
CAP = "fixtures/render-bvh/scene-hfield/capture-hfield"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()
REVIEWED_BASE = "a6bcca80"  # 派工消息给出的 HEAD
WINDOW = R / "target/render-bvh/r05-s3a-window"

SIGNED = {
    "scripts/capture_render_hfield.py": ("cb1808f8c185df0e4695bb7107ec941c2e9e0aa8bb883fa1f630dc029d28a599", 20314, 425),
    "fixtures/render-bvh/scene-hfield/scene.xml": ("b3b9a6b4ee4844c92edec3f217d04b7e788a69f9ab1e864fe7dac765e9c6f53b", 3590, 63),
    "fixtures/render-bvh/scene-hfield/scene-mixed.xml": ("bf9bb966887466d1f526605875df3f9b8c6b244787e7f95be2eccc3cd3329bdf", 3180, 60),
}
GROUPS = {
    "hfield-only": {"scene_sha": "b3b9a6b4ee4844c92edec3f217d04b7e788a69f9ab1e864fe7dac765e9c6f53b", "scene_bytes": 3590},
    "hfield-primitive": {"scene_sha": "bf9bb966887466d1f526605875df3f9b8c6b244787e7f95be2eccc3cd3329bdf", "scene_bytes": 3180},
}
GLOBAL_WORDS = [
    (71, 49, 48, 24050, 23454, 29616), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (21407, 29983, 25918, 34892), (31934, 24230, 36798, 26631),
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
]
CARD_WORDS = [
    (31383, 21475, 36890, 36807), (31934, 24230, 36798, 26631),
    (19978, 28216, 31561, 20215), (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
]
BANNED = sorted({"".join(chr(c) for c in seq) for seq in GLOBAL_WORDS + CARD_WORDS})

problems = []
findings = []


def blob(rel):
    return subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=R, capture_output=True, check=True).stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lines(b):
    return b.count(b"\n") + (0 if b.endswith(b"\n") else 1)


def check(cond, msg):
    if not cond:
        problems.append(msg)


rep = {"head": HEAD, "reviewed_base_head": REVIEWED_BASE}

# ---------------------------------------------------------------- 0. HEAD 漂移范围
drift = [p for p in subprocess.run(["git", "diff", "--name-only", f"{REVIEWED_BASE}..HEAD"],
                                   cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
reviewed_prefixes = (EV + "/", "docs/development-evidence/render-bvh/r05-s3b-hfield-target-authoring/",
                     "fixtures/render-bvh/scene-hfield/", "scripts/capture_render_hfield.py",
                     "tests/render_hfield_reference.rs")
log_bytes = subprocess.run(["git", "log", "--oneline", f"{REVIEWED_BASE}..HEAD"], cwd=R, capture_output=True,
                           check=True).stdout
rep["head_drift"] = {"new_commits": log_bytes.decode("utf-8", "replace").strip().split("\n"),
                     "touches_reviewed_paths": [p for p in drift if p.startswith(reviewed_prefixes)]}
check(not rep["head_drift"]["touches_reviewed_paths"], "HEAD drift touches reviewed paths")

# ---------------------------------------------------------------- 1. 签字物未变
rep["signed"] = {}
for rel, (want_sha, want_bytes, want_lines) in SIGNED.items():
    b = blob(rel)
    non_ascii = sum(1 for byte in b if byte > 127)
    rep["signed"][rel] = {"sha256": sha(b), "bytes": len(b), "lf_lines": lines(b), "non_ascii": non_ascii,
                          "sha_ok": sha(b) == want_sha, "bytes_ok": len(b) == want_bytes, "lines_ok": lines(b) == want_lines}
    check(rep["signed"][rel]["sha_ok"] and rep["signed"][rel]["bytes_ok"] and rep["signed"][rel]["lines_ok"],
          f"signed artifact mismatch: {rel}")

# ---------------------------------------------------------------- 2. 8 个捕获文件 + npz 结构
manifest_rows = []
for line in (R / EV / "manifest.txt").read_text(encoding="utf-8").split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h, n, p = line.split(" ", 2)
        manifest_rows.append((h, int(n), p))
capture_rows = [(h, n, p) for h, n, p in manifest_rows if p.startswith("fixtures/")]
rep["captures"] = {"rows": len(capture_rows), "files": {}, "total_bytes": 0, "manifest_mismatch": [], "npz_shapes": {}}
for h, n, p in capture_rows:
    b = blob(p)
    rep["captures"]["files"][p] = {"sha256": sha(b), "bytes": len(b)}
    rep["captures"]["total_bytes"] += len(b)
    if (sha(b), len(b)) != (h, n):
        rep["captures"]["manifest_mismatch"].append(p)
    check((sha(b), len(b)) == (h, n), f"capture file manifest mismatch: {p}")
check(len(capture_rows) == 8, f"capture rows {len(capture_rows)} != 8")
check(rep["captures"]["total_bytes"] == 5784264, f"capture total bytes {rep['captures']['total_bytes']} != 5784264")


def npy_shapes(npz_path):
    """从 npz（zip）中读取每个 .npy 的 shape（只读结构，不做数值比较）。"""
    out = {}
    with zipfile.ZipFile(npz_path) as zf:
        for name in zf.namelist():
            if not name.endswith(".npy"):
                continue
            with zf.open(name) as fh:
                version = fh.read(8)
                if version[:6] != b"\x93NUMPY":
                    out[name] = "not-npy"
                    continue
                if version[6] == 1:
                    hlen = struct.unpack("<H", fh.read(2))[0]
                else:
                    hlen = struct.unpack("<I", fh.read(4))[0]
                header = fh.read(hlen).decode("latin1")
                m = re.search(r"'shape':\s*\(([^)]*)\)", header)
                out[name] = tuple(int(x) for x in m.group(1).split(",") if x.strip()) if m else "?"
    return out


for group in GROUPS:
    path = R / CAP / group / "rgb.npz"
    shapes = npy_shapes(path)
    rep["captures"]["npz_shapes"][group] = shapes
    # 结构核对：cam0 320x240、cam1 240x180（不比较像素值）
    expected_pairs = {(240, 320, 3), (180, 240, 3)}
    got = {(s[0], s[1], s[2]) for s in shapes.values() if isinstance(s, tuple) and len(s) == 3}
    check(expected_pairs == got, f"{group}: rgb npz shapes {got} != {expected_pairs}")

rep["capture_json"] = {}
captures = {}
for group, meta in GROUPS.items():
    cj = json.loads(blob(f"{CAP}/{group}/capture.json"))
    captures[group] = cj
    npz_ok = {}
    for name in ("rgb.npz", "depth.npz", "seg.npz"):
        b = blob(f"{CAP}/{group}/{name}")
        npz_ok[name] = sha(b) == cj["files_sha256"].get(name)
        check(npz_ok[name], f"{group}/{name}: capture.json sha mismatch")
    text = json.dumps(cj, ensure_ascii=True)
    rep["capture_json"][group] = {
        "group": cj.get("group"), "use_shadows": cj.get("use_shadows"),
        "use_fast_math": cj.get("use_fast_math"),
        "two_captures_bitwise_identical": cj.get("two_captures_bitwise_identical"),
        "legacy_reference_untouched": cj.get("legacy_reference_untouched"),
        "legacy_reference_files": cj.get("legacy_reference_files"),
        "captured_at": cj.get("captured_at"),
        "recipe_scene_xml": cj.get("recipe", {}).get("scene_xml"),
        "recipe_script": cj.get("recipe", {}).get("script"),
        "npz_sha_ok": npz_ok,
        "fast_math_false_occurrences": text.count('"use_fast_math": false'),
        "fast_math_true_occurrences": text.count('"use_fast_math": true'),
        "shadow_true_occurrences": text.count('"use_shadows": true'),
    }
    r = rep["capture_json"][group]
    check(r["group"] == group, f"{group}: group mismatch")
    check(r["use_shadows"] is True, f"{group}: use_shadows != true")
    check(r["use_fast_math"] is False, f"{group}: use_fast_math != false")
    check(r["two_captures_bitwise_identical"] is True, f"{group}: two-captures flag")
    check(r["legacy_reference_untouched"] is True and r["legacy_reference_files"] == 38, f"{group}: legacy flag")
    check(r["captured_at"] == "2026-10-29T00:00:00+08:00", f"{group}: captured_at")
    check(r["recipe_scene_xml"].get("sha256") == meta["scene_sha"], f"{group}: recipe scene sha")
    check(r["recipe_scene_xml"].get("bytes") == meta["scene_bytes"], f"{group}: recipe scene bytes")
    check(r["recipe_script"].get("sha256") == SIGNED["scripts/capture_render_hfield.py"][0], f"{group}: recipe script sha")
    check(r["fast_math_true_occurrences"] == 0, f"{group}: use_fast_math true occurrence")
    check(r["fast_math_false_occurrences"] == 7, f"{group}: use_fast_math false occurrences != 7")
check(rep["capture_json"]["hfield-only"]["captured_at"] == rep["capture_json"]["hfield-primitive"]["captured_at"],
      "captured_at differs across groups")

# cameras：capture.json 中 2 相机 + 分辨率；npz 结构已核
def find_pairs(obj, out):
    if isinstance(obj, dict):
        for v in obj.values():
            find_pairs(v, out)
    elif isinstance(obj, list):
        if len(obj) == 2 and all(isinstance(x, list) and len(x) == 2 for x in obj):
            out.append(obj)
        for v in obj:
            find_pairs(v, out)


def find_keys(obj, key, out):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k == key:
                out.append(v)
            find_keys(v, key, out)
    elif isinstance(obj, list):
        for v in obj:
            find_keys(v, key, out)


rep["cameras"] = {}
for group, cj in captures.items():
    pairs, ncams = [], []
    find_pairs(cj, pairs)
    find_keys(cj, "ncam", ncams)
    has_cams = any(p == [[320, 240], [240, 180]] for p in pairs) or (
        any(p == [320, 240] for p in pairs) and any(p == [240, 180] for p in pairs))
    rep["cameras"][group] = {"pairs": pairs[:6], "ncam": ncams, "has_320x240_and_240x180": has_cams}
    check(has_cams, f"{group}: camera resolutions not found")
    if ncams:
        check(all(int(v) == 2 for v in ncams), f"{group}: ncam != 2 {ncams}")

# ---------------------------------------------------------------- 3. 例外留档完整性
facts = (R / EV / "logs/window-facts.txt").read_text(encoding="utf-8")
readme = (R / EV / "README.md").read_text(encoding="utf-8")
receipt = json.loads((R / EV / "receipt.json").read_text(encoding="utf-8"))
exc = receipt.get("authoring_only_exception", {})
rep["exception"] = {
    "facts_attempt123_stop_count": facts.count("decision=STOP: normal threshold util<40 not met"),
    "facts_attempt3_numbers": all(s in facts for s in (
        "gate3_samples=354", "gate3_longest_clean=39", "gate3_util_min=2", "gate3_util_max=49",
        "gate3_free_min_mib=13109", "gate3_util_ge_40_samples=40", "gate3_util_ge_60_samples=0")),
    "facts_attempt4_exception": all(s in facts for s in (
        "authoring_only_exception=util_lt_60", "authoring_only_exception_approved_by=Chief",
        "authoring_only_exception_reason=load-attribution",
        "attempt4_monitor_script_sha256=689fd7eb9cf733a60d40b9bdccafe4119ee23e2e8e15a0df68638716a9c1871b",
        "attempt4_monitor_script_bytes=2529", "nonascii=0", "ansi_lines=34 lf_lines=34")),
    "facts_not_regular_pass": "this window is NOT reported as a normal-threshold pass" in facts,
    "facts_record_retained": "the original util<40 threshold record is retained" in facts,
    "readme_not_regular_pass": "attempt4 不记为常规门槛通过" in readme,
    "readme_record_retained": "原始 `util<40` 失败记录保留" in readme,
    "readme_approver_reason": "Chief 预批的 authoring-only 例外" in readme and "load-attribution" in readme,
    "readme_monitor_script_sha": "689fd7eb" in readme,
    "receipt_exception": {
        "name": exc.get("name"), "approved_by": exc.get("approved_by"),
        "reason_has_load_attribution": "load-attribution" in str(exc.get("reason")),
        "record_retained_text": "NOT reported as a normal-threshold pass"
        in str(exc.get("original_threshold_record_retained")),
        "monitor_script": exc.get("monitor_script"),
    },
    "receipt_attempts_stop_count": len([a for a in receipt.get("pre_window_monitor", {}).get("attempts", [])
                                        if a.get("decision", "").startswith("STOP")]),
}
check(rep["exception"]["facts_attempt123_stop_count"] == 3, "window-facts STOP count != 3")
check(rep["exception"]["facts_attempt3_numbers"], "window-facts attempt3 numbers missing")
check(rep["exception"]["facts_attempt4_exception"], "window-facts attempt4 fields missing")
check(rep["exception"]["facts_not_regular_pass"], "facts missing NOT-regular-pass")
check(rep["exception"]["facts_record_retained"], "facts missing record-retained")
check(rep["exception"]["readme_not_regular_pass"], "README missing NOT-regular-pass")
check(rep["exception"]["readme_record_retained"], "README missing record-retained")
check(rep["exception"]["readme_approver_reason"], "README missing approver/reason")
check(rep["exception"]["readme_monitor_script_sha"], "README missing monitor script sha")
check(rep["exception"]["receipt_exception"]["name"] == "util_lt_60", "receipt exception name")
check("Chief" in str(rep["exception"]["receipt_exception"]["approved_by"]), "receipt exception approver")
check(rep["exception"]["receipt_exception"]["reason_has_load_attribution"], "receipt reason")
check(rep["exception"]["receipt_exception"]["record_retained_text"], "receipt record note")
check(rep["exception"]["receipt_attempts_stop_count"] == 3, "receipt STOP attempts != 3")

# 不应把 attempt4 表述为常规通过（排除含否定词的句段）
suspicious_attempt4 = []
for m in re.finditer(r"attempt4[^\n]{0,240}", readme):
    seg = m.group(0)
    positive = any(p in seg for p in ("常规门槛通过", "常规通过"))
    negated = any(n in seg for n in ("不记为", "不表述为", "NOT", "不是", "非"))
    if positive and not negated:
        suspicious_attempt4.append(seg)
rep["exception"]["suspicious_attempt4_positive_claims"] = suspicious_attempt4
check(not suspicious_attempt4, f"attempt4 described as regular pass: {suspicious_attempt4}")

mon = (R / EV / "logs/monitor-until-clean-attempt4.ps1").read_bytes()
decoded = mon.decode("ascii", errors="replace")
rep["exception"]["monitor_script_recomputed"] = {
    "sha256": sha(mon), "bytes": len(mon), "non_ascii": sum(1 for b in mon if b > 127),
    "lf_lines": lines(mon), "ansi_lines": len(mon.decode("gbk", errors="replace").splitlines()),
    "comment_dollar_lines": sum(1 for l in decoded.split("\n") if l.lstrip().startswith("#") and "$" in l),
}
ms = rep["exception"]["monitor_script_recomputed"]
check(ms["sha256"] == "689fd7eb9cf733a60d40b9bdccafe4119ee23e2e8e15a0df68638716a9c1871b", "monitor script sha")
check(ms["bytes"] == 2529, "monitor script bytes != 2529")
check(ms["non_ascii"] == 0, "monitor script non-ASCII")
check(ms["lf_lines"] == ms["ansi_lines"] == 34, "monitor script line counts")
check(ms["comment_dollar_lines"] == 0, "monitor script comment-$ lines")

# 监测 CSV 独立重算
def csv_facts(rel):
    t = (R / EV / rel).read_text(encoding="utf-8")
    rows = [r for r in t.split("\n")[1:] if r.strip()]
    cells = [r.split(",") for r in rows]
    util = [int(c[3]) for c in cells]
    free = [int(c[2]) for c in cells]
    longest = cur = 0
    for c in cells:
        ok = c[8].strip() == "True"
        cur = cur + 1 if ok else 0
        longest = max(longest, cur)
    return {"samples": len(rows), "util_min": min(util), "util_max": max(util), "free_min": min(free),
            "longest_clean_streak": longest, "util_ge_40": sum(1 for u in util if u >= 40),
            "util_ge_60": sum(1 for u in util if u >= 60)}


rep["monitor_csv"] = {}
monitor_expect = {
    "logs/monitor.csv": {"samples": 177, "util_min": 2, "util_max": 43, "free_min": 13134, "longest_clean_streak": 24},
    "logs/monitor-attempt2.csv": {"samples": 177, "util_min": 2, "util_max": 43, "free_min": 13188, "longest_clean_streak": 34},
    "logs/monitor-attempt3.csv": {"samples": 354, "util_min": 2, "util_max": 49, "free_min": 13109,
                                   "longest_clean_streak": 39, "util_ge_40": 40, "util_ge_60": 0},
    "logs/monitor-attempt4.csv": {"samples": 60, "util_min": 4, "util_max": 48, "free_min": 13166,
                                   "longest_clean_streak": 60, "util_ge_60": 0},
}
for rel, want in monitor_expect.items():
    got = csv_facts(rel)
    rep["monitor_csv"][rel] = {"computed": got, "expected": want, "match": all(got[k] == v for k, v in want.items())}
    check(rep["monitor_csv"][rel]["match"], f"monitor csv mismatch: {rel}: {got} vs {want}")

# ---------------------------------------------------------------- 4. 旧参考零读写
legacy = (R / EV / "materials/legacy-baseline.txt").read_text(encoding="utf-8")
legacy_rows = []
for line in legacy.split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h, n, p = line.split(" ", 2)
        legacy_rows.append((h, int(n), p))
rep["legacy"] = {"files": len(legacy_rows), "bytes": 0, "mismatch": [], "crlf_text_files": []}
for h, n, p in legacy_rows:
    b = (R / p).read_bytes()   # 历史口径：工作树字节（见观察 O-2）
    rep["legacy"]["bytes"] += len(b)
    if (sha(b), len(b)) != (h, n):
        rep["legacy"]["mismatch"].append(p)
    if b"\r" in b and p.endswith((".json", ".md", ".txt")):
        rep["legacy"]["crlf_text_files"].append(p)
    check((sha(b), len(b)) == (h, n), f"legacy file changed: {p}")
rep["legacy"]["scene_a_files"] = sum(1 for _, _, p in legacy_rows if p.startswith("fixtures/render-bvh/scene-a/"))
rep["legacy"]["scene_e2e_files"] = sum(1 for _, _, p in legacy_rows if p.startswith("fixtures/render-bvh/scene-e2e/"))
check(rep["legacy"]["files"] == 38 and rep["legacy"]["bytes"] == 25320515, "legacy count/bytes mismatch")
check(rep["legacy"]["scene_a_files"] == 37 and rep["legacy"]["scene_e2e_files"] == 1, "legacy dir split")

# 聚合值：按上游口径用 PowerShell 复现
agg = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                      str(R / CHK / "logs" / "aggregate-repro.ps1")],
                     cwd=R, capture_output=True)
agg_text = (agg.stdout + agg.stderr).decode("utf-8", "replace")
rep["legacy"]["aggregate_repro"] = agg_text.strip().split("\n")
agg_hex = re.search(r"aggregate_hexconcat (\w+)", agg_text)
agg_lines = re.search(r"aggregate_lines (\w+)", agg_text)
rep["legacy"]["aggregate_hexconcat"] = agg_hex.group(1) if agg_hex else None
rep["legacy"]["aggregate_lines"] = agg_lines.group(1) if agg_lines else None
check(rep["legacy"]["aggregate_hexconcat"] == "b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b",
      "aggregate hexconcat mismatch")
check(rep["legacy"]["aggregate_lines"] == "983617f16b534f9cda743ff2b84bfed08e0154e88c369419b885bacb841cc260",
      "aggregate lines mismatch (R09 cross-check)")

# scene-shadow：窗口前快照（若仍存在）+ 目录总量 + git 不变
shadow_dir = R / "fixtures/render-bvh/scene-shadow"
shadow_files = sorted(p for p in shadow_dir.rglob("*") if p.is_file())
rep["scene_shadow"] = {"files": len(shadow_files), "bytes": sum(len(p.read_bytes()) for p in shadow_files)}
artifacts = [p for p in shadow_files if "capture" in p.relative_to(shadow_dir).as_posix()]
rep["scene_shadow"]["capture_artifacts"] = len(artifacts)
rep["scene_shadow"]["capture_artifact_bytes"] = sum(len(p.read_bytes()) for p in artifacts)
check(len(shadow_files) == 17 and rep["scene_shadow"]["bytes"] == 11566120, "scene-shadow totals")
check(len(artifacts) == 16 and rep["scene_shadow"]["capture_artifact_bytes"] == 11561363, "scene-shadow artifacts")
snap = WINDOW / "scene-shadow-before.txt"
if snap.exists():
    snap_rows = []
    for line in snap.read_text(encoding="utf-8-sig").split("\n"):
        line = line.strip()
        if line and not line.startswith("#"):
            h, n, p = line.split(" ", 2)
            snap_rows.append((h, int(n), p))
    mismatch = []
    for h, n, p in snap_rows:
        b = (R / p).read_bytes()
        if (sha(b), len(b)) != (h, n):
            mismatch.append(p)
    rep["scene_shadow"]["window_snapshot_rows"] = len(snap_rows)
    rep["scene_shadow"]["window_snapshot_artifact_rows"] = sum(1 for _, _, p in snap_rows if "capture" in p)
    rep["scene_shadow"]["window_snapshot_mismatch"] = mismatch
    check(len(snap_rows) == 17 and sum(1 for _, _, p in snap_rows if "capture" in p) == 16 and not mismatch,
          f"scene-shadow vs window snapshot mismatch: {mismatch}")
else:
    rep["scene_shadow"]["window_snapshot_rows"] = None
    findings.append("window snapshot scene-shadow-before.txt 不可用；改用 git 不变性核验")
for base in ("926b6488", HEAD):
    diff_names = [p for p in subprocess.run(
        ["git", "diff", "--name-only", base, "HEAD", "--", "fixtures/render-bvh/scene-shadow"],
        cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
    rep["scene_shadow"][f"diff_vs_{base[:8]}"] = diff_names
    check(not diff_names, f"scene-shadow changed since {base[:8]}: {diff_names[:5]}")
legacy_changed = [p for p in subprocess.run(
    ["git", "diff", "--name-only", "18c17d3a", "HEAD", "--", "fixtures/render-bvh/scene-a",
     "fixtures/render-bvh/scene-e2e"], cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
rep["legacy"]["git_diff_since_authoring"] = legacy_changed
check(not legacy_changed, f"legacy refs changed since authoring: {legacy_changed[:5]}")

# git status 受保护路径（当前工作树；若只剩与本片无关的未跟踪工具目录，记为观察）
gs = subprocess.run(["git", "status", "--short", "--", "scripts", "fixtures/render-bvh/scene-shadow",
                     "fixtures/render-bvh/scene-a", "fixtures/render-bvh/scene-e2e"],
                    cwd=R, capture_output=True, text=True, check=True).stdout.strip()
tracked_gs = [l for l in gs.split("\n") if l.strip() and not l.strip().startswith("??")]
untracked_gs = [l for l in gs.split("\n") if l.strip().startswith("??")]
rep["git_status"] = {"all": gs, "tracked_changes": tracked_gs, "untracked": untracked_gs}
check(not tracked_gs, f"tracked changes under protected paths: {tracked_gs}")
if untracked_gs:
    findings.append(f"受保护路径下存在未跟踪项（非本片提交，需确认归属）：{untracked_gs}")

# ---------------------------------------------------------------- 5. 窗口事实
def parse_facts(text):
    out = {}
    for line in text.split("\n"):
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            out.setdefault(k.strip(), []).append(v.strip())
    return out


ff = parse_facts(facts)
rep["window_facts"] = {
    "capture_cmd1": ff.get("capture_cmd1"), "capture_cmd2": ff.get("capture_cmd2"),
    "capture_total_wall_ms": ff.get("capture_total_wall_ms"),
    "window_open_at": ff.get("window_open_at"), "window_closed_at": ff.get("window_closed_at"),
    "gate4_free_min": ff.get("gate4_free_min_mib"), "gate4_floor": ff.get("gate4_floor_mib"),
    "gate4_compute_filtered": ff.get("gate4_compute_filtered"),
    "gate4_build": ff.get("gate4_build_processes"),
    "gpu_released": ff.get("gpu_released"), "residual_render_procs": ff.get("residual_render_procs"),
    "numeric_conclusion": ff.get("numeric_conclusion"),
    "signature_match": ff.get("gate1_signature_match"),
    "build_processes_after": ff.get("build_processes_after"),
}
check(rep["window_facts"]["capture_cmd1"] == ["hfield-only exit=0 wall_ms=9952"], "cmd1 fact")
check(rep["window_facts"]["capture_cmd2"] == ["hfield-primitive exit=0 wall_ms=4764"], "cmd2 fact")
check(rep["window_facts"]["capture_total_wall_ms"] == ["14716"], "total wall")
check(rep["window_facts"]["gate4_free_min"] == ["13268"], "gate4 free_min")
check(rep["window_facts"]["gate4_compute_filtered"] == ["0"], "gate4 filtered")
check(rep["window_facts"]["gate4_build"] == ["0"], "gate4 build")
check(rep["window_facts"]["residual_render_procs"] == ["0"], "residual procs")
check(rep["window_facts"]["build_processes_after"] == ["0"], "build processes after")
check(all(v == "True" for v in rep["window_facts"]["signature_match"]), "signature match not all True")
check(rep["window_facts"]["gpu_released"] is not None and rep["window_facts"]["gpu_released"][0].startswith("True"),
      "gpu_released not True")
check(rep["window_facts"]["numeric_conclusion"] == ["false (this slice captures only; no rgb/depth/seg numbers are compared or claimed)"],
      "numeric_conclusion fact unexpected")
# floor 公式自校（min(10240, baseline_free-3072)）
floors = {}
for attempt in range(1, 5):
    text = facts
    block = text.split(f"[attempt{attempt} ")[1].split("[", 1)[0] if f"[attempt{attempt} " in text else ""
    free = re.search(r"gate2_free_min_mib=(\d+)", block)
    floor = re.search(r"gate2_floor_mib=(\d+)", block)
    if free and floor:
        floors[f"attempt{attempt}"] = {"free": int(free.group(1)), "floor": int(floor.group(1)),
                                       "expected": min(10240, int(free.group(1)) - 3072),
                                       "ok": int(floor.group(1)) == min(10240, int(free.group(1)) - 3072)}
        check(floors[f"attempt{attempt}"]["ok"], f"floor formula mismatch attempt{attempt}")
gate4_block = facts.split("[gate4 ")[1].split("[", 1)[0] if "[gate4 " in facts else ""
if gate4_block:
    free = int(re.search(r"gate4_free_min_mib=(\d+)", gate4_block).group(1))
    floor = int(re.search(r"gate4_floor_mib=(\d+)", gate4_block).group(1))
    floors["gate4"] = {"free": free, "floor": floor, "expected": min(10240, free - 3072), "ok": floor == min(10240, free - 3072)}
    check(floors["gate4"]["ok"], "floor formula mismatch gate4")
rep["window_facts"]["floors"] = floors
# 证据日志 vs 窗口目录原始日志（若窗口目录仍存在）
identity = {}
if WINDOW.exists():
    for name in ("capture-hfield-only.log", "capture-hfield-primitive.log", "monitor.csv",
                 "monitor-attempt2.csv", "monitor-attempt3.csv", "monitor-attempt4.csv",
                 "monitor-until-clean-attempt4.ps1"):
        ev_path = R / EV / "logs" / name
        win_path = WINDOW / name
        if ev_path.exists() and win_path.exists():
            identity[name] = sha(ev_path.read_bytes()) == sha(win_path.read_bytes())
            check(identity[name], f"evidence log differs from window original: {name}")
rep["window_facts"]["evidence_vs_window_identity"] = identity

# ---------------------------------------------------------------- 6. 只采集不比数
numeric_hits = []
patterns = [r"beyond_1_lsb", r"beyond_one_lsb", r"max_ulp", r"mismatched", r"mismatch\s*=",
            r"≤\s*1\s*LSB", r"≤\s*4\s*ULP"]
whitelist = ("materials/reverse-calc", "materials/order-crosscheck", "materials/load-attribution")
criteria_markers = ("不写", "不宣称", "零数值", "口径", "判据", "数值验收（", "快照", "快照比对")
for p in sorted(x for x in (R / EV).rglob("*") if x.is_file()):
    rel = p.relative_to(R / EV).as_posix()
    t = p.read_text(encoding="utf-8", errors="replace")
    for pat in patterns:
        for m in re.finditer(pat, t):
            ln = t[:m.start()].count("\n") + 1
            line = t.split("\n")[ln - 1].strip()
            if rel.startswith(whitelist) or any(k in line for k in criteria_markers):
                continue
            numeric_hits.append((rel, ln, pat, line[:160]))
rep["numeric_conclusion_suspicious"] = numeric_hits
check(not numeric_hits, f"suspicious numeric-conclusion lines: {numeric_hits[:5]}")

# ---------------------------------------------------------------- 7. 证据清单 / 编码 / 禁词
ev_files = sorted(p for p in (R / EV).rglob("*") if p.is_file())
ev_rel = {p.relative_to(R / EV).as_posix() for p in ev_files if p.name != "manifest.txt"}
man_ev = {p: (h, n) for h, n, p in manifest_rows if not p.startswith("fixtures/")}
man_problems = []
if ev_rel != set(man_ev):
    man_problems.append(f"set mismatch {sorted(ev_rel ^ set(man_ev))}")
for rel in sorted(ev_rel & set(man_ev)):
    b = blob(f"{EV}/{rel}")
    if (sha(b), len(b)) != man_ev[rel]:
        man_problems.append(f"{rel}: {man_ev[rel]} vs {(sha(b), len(b))}")
bom, crlf, hits = [], [], []
for p in ev_files:
    b = p.read_bytes()
    rel = p.relative_to(R / EV).as_posix()
    if b.startswith(b"\xef\xbb\xbf"):
        bom.append(rel)
    if b"\r" in b:
        crlf.append(rel)
    t = p.read_text(encoding="utf-8", errors="replace")
    hits += [(rel, w) for w in BANNED if w in t]
rep["evidence"] = {
    "files": len(ev_files), "manifest_rows": len(manifest_rows), "capture_rows": len(capture_rows),
    "evidence_rows": len(man_ev), "manifest_problems": man_problems,
    "manifest_self_reference": any(p == "manifest.txt" for _, _, p in manifest_rows),
    "bom_files": bom, "crlf_files": crlf, "forbidden_word_hits": hits,
    "path_bases_documented": "repo-root-relative" in (R / EV / "manifest.txt").read_text(encoding="utf-8")
    and "relative to this evidence directory" in (R / EV / "manifest.txt").read_text(encoding="utf-8"),
}
check(not man_problems, f"manifest problems: {man_problems}")
check(not rep["evidence"]["manifest_self_reference"], "manifest self reference")
check(not bom, f"BOM files: {bom}")
check(not hits, f"forbidden word hits: {hits}")
check(rep["evidence"]["path_bases_documented"], "manifest path bases not documented")
for required in ("logs/window-facts.txt", "logs/monitor.csv", "logs/monitor-attempt2.csv",
                 "logs/monitor-attempt3.csv", "logs/monitor-attempt4.csv",
                 "logs/monitor-until-clean-attempt4.ps1", "materials/signature-record.md",
                 "materials/load-attribution.txt", "materials/legacy-baseline.txt",
                 "materials/reverse-calc.txt", "materials/reverse-calc.py",
                 "materials/order-crosscheck.txt", "README.md", "receipt.json"):
    check(required in man_ev, f"manifest missing evidence row: {required}")
if crlf:
    findings.append(f"实现证据中有 {len(crlf)} 个文件按 committed blob 含 CRLF（卡面要求 LF）：{crlf}")

rep["findings"] = findings
rep["problems"] = problems
out = pathlib.Path(CHK) / "recompute.json"
out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(rep, ensure_ascii=False, indent=2))
print("RESULT problems=" + str(len(problems)) + " findings=" + str(len(findings)))
for item in problems:
    print("PROBLEM " + str(item))
for item in findings:
    print("FINDING " + str(item))
