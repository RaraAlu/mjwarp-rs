#!/usr/bin/env python3
"""QA 独立复核：SDF 渲染参考采集窗（U3，authoring-only util<60 例外）。

只读：所有结论由 committed blob（git）现算；不采信证据结论行。
被核 pin：1b185557（窗口本体 e0ab7ce8，util_min 更正在其后）
复核卡：2026-10-30-render-sdf-capture-check（revision.before = e0988285）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-capture-check/recompute.py
"""
import csv
import hashlib
import io
import itertools
import json
import pathlib
import re
import subprocess
import zipfile

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "1b185557"
WIN = "e0ab7ce8"
PRE = "dea7d0f1"
CARD = "2026-10-30-render-sdf-capture"
EV = "docs/development-evidence/render-bvh/sdf-capture"
EVW = "docs/development-evidence/render-bvh/sdf-capture-window-material"
PROD = "fixtures/render-bvh/scene-sdf/capture-sdf"

checks = []


def check(cid, ok, detail):
    checks.append({"id": cid, "passed": bool(ok), "detail": detail})


def git(*args):
    out = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    if out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {out.stderr.decode('utf-8', 'replace')[:300]}")
    return out.stdout


def blob(rev, p):
    return git("cat-file", "blob", f"{rev}:{p}")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def txt(rev, p):
    b = blob(rev, p)
    return (b.decode("utf-16-le") if b[:2] in (b"\xff\xfe",) else b.decode("utf-8", "replace")).lstrip("\ufeff")


def csv_rows(rel):
    return list(csv.DictReader(io.StringIO(txt(PIN, EV + "/" + rel))))


# 1. 例外五要素 + 原失败记录保留
r = json.loads(txt(PIN, EV + "/receipt.json"))
ex = r["exception"]
target = REPO / "target/render-bvh/sdf-capture-window"
retained = {}
for evp, tp in ((EV + "/monitor/monitor-pre-window-regular-failure.csv", target / "monitor-pre-window.csv"),
                (EV + "/monitor/pre-window-snapshot-regular.txt", target / "attempt1/pre-window-snapshot.txt"),
                (EV + "/monitor/monitor-failure-regular.txt", target / "attempt1/monitor-failure.txt"),
                (EV + "/monitor/pre-window-snapshot-exception.txt", target / "attempt1/pre-window-snapshot-attempt2.txt"),
                (EV + "/monitor/monitor-pre-window-attempt2-exception.csv", target / "monitor-pre-window-attempt2.csv")):
    retained[evp.split("/")[-1]] = bool(tp.exists() and sha(tp.read_bytes()) == sha(blob(PIN, evp)))
check("exception.five_elements_and_retention",
      ex["mode"] == "authoring_only_exception_util_lt_60"
      and ex["approved_by"] == "user via Chief 2026-10-31T00:00+08:00"
      and ex["not_a_normal_threshold_pass"] is True and ex["authoring_exception_used"] is True
      and ex["signature_unchanged"] is True
      and len(ex["original_normal_threshold_attempts_retained"]) == 3
      and all(retained.values()),
      {"exception": {k: ex[k] for k in ("mode", "approved_by", "not_a_normal_threshold_pass",
                                        "authoring_exception_used", "signature_unchanged")},
       "retained_list": ex["original_normal_threshold_attempts_retained"],
       "target_copies_byte_equal": retained})

# 2. 例外 monitor 机器重算（load-bearing = util>=40 的 clean 样本仍有 streak 继续）
rows = csv_rows("monitor/monitor-pre-window-attempt2-exception.csv")
util = [int(x["util_percent"]) for x in rows]
clean = [x["clean"].strip().lower() == "true" for x in rows]
streak = [int(x["streak"]) for x in rows]
geo = [i for i, u in enumerate(util) if u >= 40]
mon = {
    "samples": len(rows), "all_clean": all(clean), "max_streak": max(streak),
    "util_min": min(util), "util_max": max(util),
    "ge40": len(geo), "ge40_clean": sum(1 for i in geo if clean[i]),
    "ge60": sum(1 for u in util if u >= 60),
    "free_min": min(int(x["free_mib"]) for x in rows),
    "filtered_max": max(int(x["compute_filtered"]) for x in rows),
    "build_max": max(int(x["build"]) for x in rows),
    "clean_rule_matches_60": all(clean[i] == (util[i] < 60) for i in range(len(rows))),
}
check("exception.monitor_load_bearing",
      mon["samples"] == 60 and mon["all_clean"] and mon["max_streak"] == 60
      and mon["ge40"] == 17 and mon["ge40_clean"] == 17 and mon["ge60"] == 0
      and mon["util_max"] < 60 and mon["clean_rule_matches_60"]
      and mon["filtered_max"] == 0 and mon["build_max"] == 0,
      mon)
check("exception.maxutil_verbatim",
      "-MaxUtil 60" in r["window"]["exception_monitor"]["command"]
      and "-Required 60" in r["window"]["exception_monitor"]["command"]
      and "-MaxMinutes 30" in r["window"]["exception_monitor"]["command"]
      and "powershell" in r["window"]["exception_monitor"]["command"],
      {"command": r["window"]["exception_monitor"]["command"]})

# 3. 常规失败记录原样保留（未被覆盖、数字可复现）
rf = csv_rows("monitor/monitor-pre-window-regular-failure.csv")
ru = [int(x["util_percent"]) for x in rf]
rc = [x["clean"].strip().lower() == "true" for x in rf]
runs = [len(list(g)) for k, g in itertools.groupby(rc) if k]
fail = {
    "samples": len(rf), "longest_clean": max(runs) if runs else 0,
    "util_min": min(ru), "util_max": max(ru), "ge40": sum(1 for u in ru if u >= 40),
    "ge40_clean": sum(1 for u, c in zip(ru, rc) if u >= 40 and c),
    "free_min": min(int(x["free_mib"]) for x in rf),
    "receipt_util_min": r["window"]["regular_monitor"]["util_min"],
    "receipt_util_max": r["window"]["regular_monitor"]["util_max"],
    "receipt_samples": r["window"]["regular_monitor"]["samples"],
    "receipt_longest": r["window"]["regular_monitor"]["longest_clean"],
    "receipt_ge40": r["window"]["regular_monitor"]["util_ge40_samples"],
}
check("exception.regular_failure_reproduced",
      fail["samples"] == fail["receipt_samples"] == 354 and fail["longest_clean"] == fail["receipt_longest"] == 21
      and fail["util_min"] == fail["receipt_util_min"] == 5 and fail["util_max"] == fail["receipt_util_max"] == 50
      and fail["ge40"] == fail["receipt_ge40"] == 84 and fail["ge40_clean"] == 0,
      fail)

# 4. 两跑位级相等（capture.json 记录的两跑 hash 与磁盘三方一致）+ 数组结构
prod_report = {}
for g in ("sdf-mesh", "sdf-occluder"):
    cap_b = blob(PIN, f"{PROD}/{g}/capture.json")
    cap = json.loads(cap_b)
    row = {"capture_json": {"bytes": len(cap_b), "sha256": sha(cap_b)},
           "run1": cap["files_sha256"],
           "run2": (cap.get("context_repeat") or {}).get("outputs", {}).get("files_sha256"),
           "two_captures_bitwise_identical": cap["two_captures_bitwise_identical"],
           "legacy_reference_untouched": cap["legacy_reference_untouched"],
           "legacy_reference_files": cap["legacy_reference_files"],
           "script_sha": cap["recipe"]["script"]["sha256"][:16],
           "scene_sha": cap["recipe"]["scene_xml"]["sha256"][:16],
           "use_fast_math": cap["use_fast_math"], "use_shadows": cap["use_shadows"],
           "repeat": cap["repeat"], "nrender": cap["context"]["resolved"]["nrender"],
           "disk": {}}
    for kind in ("rgb", "depth", "seg"):
        raw = blob(PIN, f"{PROD}/{g}/{kind}.npz")
        with zipfile.ZipFile(io.BytesIO(raw)) as z:
            shapes = {}
            for n in sorted(z.namelist()):
                with z.open(n) as f:
                    a = np.load(f, allow_pickle=False)
                    shapes[n] = {"shape": list(a.shape), "dtype": str(a.dtype),
                                 "finite": bool(np.isfinite(a).all()) if a.dtype.kind == "f" else None}
        row["disk"][kind] = {"bytes": len(raw), "sha256": sha(raw), "arrays": shapes}
    row["run1_eq_run2"] = row["run1"] == row["run2"]
    row["disk_eq_run1"] = all(row["run1"][f"{k}.npz"] == row["disk"][k]["sha256"] for k in ("rgb", "depth", "seg"))
    prod_report[g] = row
check("products.two_run_and_arrays",
      all(v["run1_eq_run2"] and v["disk_eq_run1"] and v["two_captures_bitwise_identical"]
          and v["legacy_reference_untouched"] and v["legacy_reference_files"] == 69
          and v["script_sha"] == "8064596876aeec3e" and v["capture_json"]["bytes"] > 10000
          and v["use_fast_math"] is False and v["use_shadows"] is True and v["repeat"] == 2 and v["nrender"] == 2
          for v in prod_report.values())
      and prod_report["sdf-mesh"]["scene_sha"] == "376eb81ce200b293"
      and prod_report["sdf-occluder"]["scene_sha"] == "7869cfae5e16795e",
      prod_report)

# 5. legacy 69 未动 + fixture 字节不变
legacy_before = [p for p in git("ls-tree", "-r", "--name-only", PRE, "fixtures/render-bvh").decode().splitlines()
                 if not p.startswith("fixtures/render-bvh/scene-sdf/")]
legacy_pin = [p for p in git("ls-tree", "-r", "--name-only", PIN, "fixtures/render-bvh").decode().splitlines()
              if not p.startswith("fixtures/render-bvh/scene-sdf/")]
changed = [p for p in legacy_before if blob(PRE, p) != blob(PIN, p)]
scenes = {p: sha(blob(PIN, p))[:16] for p in ("fixtures/render-bvh/scene-sdf/scene.xml",
                                              "fixtures/render-bvh/scene-sdf/scene-mixed.xml")}
check("boundary.legacy_and_fixtures",
      len(legacy_pin) == 69 and legacy_before == legacy_pin and not changed
      and scenes == {"fixtures/render-bvh/scene-sdf/scene.xml": "376eb81ce200b293",
                     "fixtures/render-bvh/scene-sdf/scene-mixed.xml": "7869cfae5e16795e"},
      {"legacy_count": len(legacy_pin), "changed": changed, "scene_sha_prefix": scenes})

# 6. 范围（窗口提交只有证据/产物/卡；U3 自己的更正是 6 路径；夹在区间里的 per-world 提交单列）
win_paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{PRE}..{WIN}").decode().splitlines() if l.strip()]
bad = [p for p in win_paths if p.startswith(("src/", "tests/", "scripts/"))
       or (p.startswith("fixtures/") and not p.startswith(PROD + "/"))]
fix_paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{WIN}..{PIN}").decode().splitlines() if l.strip()]
u3_fix_paths = [p for p in fix_paths
                if p.startswith(EV) or p.startswith(EVW) or p.endswith("2026-10-30-render-sdf-capture.json")]
other_fix_paths = [p for p in fix_paths if p not in u3_fix_paths]
check("scope.window_and_correction",
      len(win_paths) == 25 and not bad and len(u3_fix_paths) == 6
      and not [p for p in fix_paths if p.startswith(("src/", "tests/", "fixtures/"))]
      and len(other_fix_paths) == 11
      and "scripts/capture_render_perworld.py" in other_fix_paths,
      {"window_paths": len(win_paths), "bad": bad, "u3_correction_paths": sorted(u3_fix_paths),
       "other_slice_paths_in_range": len(other_fix_paths),
       "note": "WIN..U3PIN 之间还夹了 per-world 切片提交（脚本修复+材料），不在 U3 范围；U3 自己的修正是 6 路径"})

# 7. 证据 manifest / util_min 更正 / 编码
man_report = {}
for d in (EV, EVW):
    man = txt(PIN, d + "/manifest.txt")
    rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
    ok = 0
    for s, n, p in rows:
        b = blob(PIN, p)
        if sha(b) == s and len(b) == int(n):
            ok += 1
    files = git("ls-tree", "-r", "--name-only", PIN, d).decode().splitlines()
    utf16 = [p.split("/")[-1] for p in files if blob(PIN, p)[:2] in (b"\xff\xfe", b"\xfe\xff")]
    man_report[d.split("/")[-1]] = {"files": len(files), "rows": len(rows), "match": ok, "utf16": utf16}
utmin = {p: ("util_min=5" in txt(PIN, p) or "util_min\": 5" in txt(PIN, p))
         for p in (EV + "/README.md", EV + "/receipt.json", EVW + "/materials/window-plan.txt")}
check("evidence.manifests_and_correction",
      man_report["sdf-capture"] == {"files": 12, "rows": 11, "match": 11,
                                    "utf16": ["capture-sdf-mesh.log", "capture-sdf-occluder.log"]}
      and man_report["sdf-capture-window-material"]["match"] == 3
      and all(utmin.values()),
      {"manifests": man_report, "util_min_5_in_three_places": utmin})

# 8. 卡面检查 / 当前 validate
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
check("card.checks_and_validate",
      chk.returncode == 0 and bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5),
      {"check_exit": chk.returncode, "validate": vm.groups() if vm else None})

problems = []
for c in checks:
    c["ok"] = c["passed"]

out = {"schema": 1, "task": CARD + "-check", "reviewed_task": CARD, "pin": PIN,
       "window_commit": WIN, "card_head": "e0988285",
       "checks": checks, "problems": problems,
       "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                   "problems": len(problems), "blockers": 0}}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
