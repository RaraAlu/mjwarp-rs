#!/usr/bin/env python3
"""QA 独立复核：per-world 采集窗（attempt1 失败留档 + 窗口期修复 + attempt2 采集）。

只读：所有结论由 committed blob（git）现算；不采信证据结论行。
被核 pin：3478bbab（修复链 e8339052 authoring + 787eb48a 材料）
复核卡：2026-10-30-render-perworld-capture-check（revision.before = e0988285）

用法（仓库根目录）：python docs/development-evidence/render-bvh/perworld-capture-check/recompute.py
"""
import csv
import hashlib
import io
import json
import pathlib
import re
import subprocess
import zipfile

import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "3478bbab"
PRE = "8e40c7ea"
CARD = "2026-10-30-render-perworld-capture"
EV = "docs/development-evidence/render-bvh/perworld-capture-run"
EVA = "docs/development-evidence/render-bvh/perworld-capture-authoring"
EVW = "docs/development-evidence/render-bvh/perworld-capture-window-material"
PROD = "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040"
SCRIPT = "scripts/capture_render_perworld.py"
NEW_SCRIPT_SHA = "67a760fa400f2cf89c19a700e49d08c619a78008664f03cd69fab21bd2077815"

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


# 1. 脚本身份 + 修复点（代码级）
s = blob(PIN, SCRIPT)
code = s.decode("utf-8")
fix = {
    "bytes": len(s), "sha256": sha(s), "lines": s.count(b"\n"), "non_ascii": sum(1 for b in s if b > 127),
    "cr": s.count(b"\r"),
    "scalar_only_slide_hinge": "in (2, 3)" in code and "in (0, 1, 2, 3)" not in code,
    "requires_single_hinge": "expected exactly one hinge joint" in code,
    "free_not_counted": "FREE/BALL are not scalar" in code,
    "base_from_qpos0": "np.array(mjm.qpos0, dtype=np.float64)" in code,
    "qposadr_bound_check": "outside qpos size" in code,
    "float32_repeat": "np.repeat(base.astype(np.float32)[None, :]" in code,
    "dryrun_static_model": "model_static" in code and "jnt_qposadr" in code,
    "two_run_compare": "two captures differ in array" in code and "two captures differ in npz bytes" in code,
    "outdir_nonempty_guard": "already exists and is not empty (references are append-only)" in code,
}
check("script.identity_and_fix",
      fix["bytes"] == 24008 and fix["sha256"] == NEW_SCRIPT_SHA and fix["lines"] == 499
      and fix["non_ascii"] == 0 and fix["cr"] == 0
      and all(v is True for k, v in fix.items() if k.startswith(("scalar", "requires", "free", "base",
                                                                  "qposadr", "float32", "dryrun", "two_run", "outdir"))),
      fix)

# 2. dry-run 离线复跑（CUDA 掩码 99，无设备）vs 证据 attempt2 计划（唯一允许差异 out_dir）
qa = json.loads((HERE / "materials" / "dryrun-qa.json").read_text(encoding="utf-8"))
evplan = json.loads(blob(PIN, EV + "/materials/dryrun-plan-attempt2.json"))
diffs = []


def walk(a, b, path=""):
    if type(a) is not type(b):
        diffs.append([path, "type"])
        return
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                diffs.append([path + "." + k, "missing"])
            else:
                walk(a[k], b[k], path + "." + k)
    elif isinstance(a, list):
        if len(a) != len(b):
            diffs.append([path, "len"])
            return
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, f"{path}[{i}]")
    elif a != b:
        diffs.append([path, a, b])


walk(evplan, qa)
diffs = [d for d in diffs if d[0] != ".out_dir"]
model = qa["model_static"]
wq = qa["per_world_plan"]["world_qpos"]
check("dryrun.reproduced",
      not diffs and qa["dry_run"] is True and qa["repeat"] == 2 and qa["legacy_manifest_entries"] == 79
      and model["njnt"] == 2 and model["nq"] == 8 and model["jnt_type"] == [0, 3]
      and model["jnt_qposadr"] == [0, 7] and model["qpos0"][3] != 0.0
      and len(wq) == 3 and all(len(r) == 8 for r in wq)
      and wq[0][7] == 0.0 and abs(wq[1][7] - 0.3490658402442932) < 1e-12
      and qa["cam_active_fixed_only"] == [True, False]
      and "not claimed" in qa["per_world_plan"]["note"],
      {"diffs_excluding_out_dir": diffs, "model_static": model,
       "world_qpos_shape": [len(wq), len(wq[0])], "world_qpos_hinge_rad": [r[7] for r in wq],
       "legacy_manifest_entries": qa["legacy_manifest_entries"],
       "cam_active_fixed_only": qa["cam_active_fixed_only"]})

# 3. 产物：两跑相等（脚本自报 + 磁盘一致）、数组结构、per_world_scope 不宣称灯位姿
cap_b = blob(PIN, PROD + "/capture.json")
cap = json.loads(cap_b)
prod = {"capture_json": {"bytes": len(cap_b), "sha256": sha(cap_b)},
        "task_id": cap["task_id"], "kind": cap["kind"],
        "two_captures_bitwise_identical": cap["two_captures_bitwise_identical"],
        "legacy_reference_untouched": cap["legacy_reference_untouched"],
        "legacy_reference_files": cap["legacy_reference_files"],
        "script_sha": cap["recipe"]["script"]["sha256"][:16],
        "scene_sha": cap["recipe"]["scene_xml"]["sha256"][:16],
        "nworld": cap["recipe"]["nworld"], "hinge_deg": cap["recipe"]["hinge_deg"],
        "use_fast_math": cap["recipe"]["use_fast_math"], "use_shadows": cap["recipe"]["use_shadows"],
        "repeat": cap["recipe"]["repeat"],
        "per_world_scope": cap.get("per_world_scope"),
        "recorded_equals_disk": {},
        "arrays": {}}
for kind in ("rgb", "depth", "seg", "state"):
    raw = blob(PIN, f"{PROD}/{kind}.npz")
    with zipfile.ZipFile(io.BytesIO(raw)) as z:
        shapes = {}
        for n in sorted(z.namelist()):
            with z.open(n) as f:
                a = np.load(f, allow_pickle=False)
                shapes[n] = {"shape": list(a.shape), "dtype": str(a.dtype),
                             "finite": bool(np.isfinite(a).all()) if a.dtype.kind == "f" else None}
    prod["arrays"][kind] = {"bytes": len(raw), "sha256": sha(raw), "arrays": shapes}
    prod["recorded_equals_disk"][kind] = cap["files_sha256"][f"{kind}.npz"] == sha(raw)
check("products.and_scope",
      all(prod["recorded_equals_disk"].values()) and prod["two_captures_bitwise_identical"] is True
      and prod["legacy_reference_untouched"] is True and prod["legacy_reference_files"] == 79
      and prod["script_sha"] == "67a760fa400f2cf8" and prod["scene_sha"] == "a9f1f0a35464f4df"
      and prod["nworld"] == 3 and prod["hinge_deg"] == [0.0, 20.0, 40.0]
      and prod["use_fast_math"] is False and prod["use_shadows"] is True and prod["repeat"] == 2
      and prod["per_world_scope"]["does_not_cover"].startswith("per-world light poses")
      and prod["arrays"]["state"]["arrays"]["qpos.npy"]["shape"] == [3, 8]
      and prod["arrays"]["rgb"]["arrays"]["rgb_w0_cam0.npy"]["shape"] == [240, 320, 3],
      prod)

# 4. legacy 79 未动（窗前后逐条 sha 一致）+ 零 Rust 编译（mtime 证据）
legacy_before = [p for p in git("ls-tree", "-r", "--name-only", PRE, "fixtures/render-bvh").decode().splitlines()
                 if not p.startswith("fixtures/render-bvh/scene-a/perworld/")]
legacy_pin = [p for p in git("ls-tree", "-r", "--name-only", PIN, "fixtures/render-bvh").decode().splitlines()
              if not p.startswith("fixtures/render-bvh/scene-a/perworld/")]
changed = [p for p in legacy_before if blob(PRE, p) != blob(PIN, p)]
check("boundary.legacy_unchanged",
      len(legacy_pin) == 79 and legacy_before == legacy_pin and not changed,
      {"legacy_count": len(legacy_pin), "changed": changed})

# 5. attempt2 例外 monitor 重算 + 诚实口径（load_bearing=false，不得称 60 门槛生效）
rows = list(csv.DictReader(io.StringIO(txt(PIN, EV + "/monitor/monitor-pre-window-attempt2.csv"))))
util = [int(r["util_percent"]) for r in rows]
clean = [r["clean"].strip().lower() == "true" for r in rows]
rec = json.loads(txt(PIN, EV + "/monitor/exception-monitor-recompute.json"))
r_receipt = json.loads(txt(PIN, EV + "/receipt.json"))
exw = r_receipt["window"]["exception"]
mon = {"samples": len(rows), "util_min": min(util), "util_max": max(util),
       "ge40": sum(1 for u in util if u >= 40), "ge60": sum(1 for u in util if u >= 60),
       "all_clean": all(clean), "max_streak": max(int(r["streak"]) for r in rows),
       "free_min": min(int(r["free_mib"]) for r in rows),
       "filtered_max": max(int(r["compute_filtered"]) for r in rows),
       "build_max": max(int(r["build"]) for r in rows),
       "clean_rule_matches_60": all(clean[i] == (util[i] < 60) for i in range(len(rows))),
       "recompute_load_bearing": rec["exception_load_bearing"],
       "receipt_load_bearing": exw.get("exception_load_bearing"),
       "receipt_effectively_not_load_bearing": exw.get("exception_effectively_not_load_bearing"),
       "receipt_maxutil": "-MaxUtil 60" in exw.get("monitor_command", "")}
check("exception.monitor_honest",
      mon["samples"] == 60 and mon["all_clean"] and mon["max_streak"] == 60
      and mon["util_min"] == 2 and mon["util_max"] == 39 and mon["ge40"] == 0 and mon["ge60"] == 0
      and mon["filtered_max"] == 0 and mon["build_max"] == 0 and mon["clean_rule_matches_60"]
      and mon["recompute_load_bearing"] is False
      and mon["receipt_load_bearing"] is False and mon["receipt_effectively_not_load_bearing"] is True
      and mon["receipt_maxutil"],
      mon)

# 6. attempt1 失败留档原样保留（target 副本可对拍）
target = REPO / "target/render-bvh/perworld-capture-window"
retained = {}
for evp, tp in ((EV + "/attempt1/capture-failure.txt", target / "attempt1/capture-failure.txt"),
                (EV + "/attempt1/monitor-pre-window-attempt1.csv", target / "monitor-pre-window-attempt1.csv"),
                (EV + "/attempt1/exception-monitor-recompute.json", target / "attempt1/exception-monitor-recompute.json"),
                (EV + "/attempt1/pre-window-snapshot.txt", target / "attempt1/pre-window-snapshot.txt"),
                (EV + "/monitor/monitor-pre-window-attempt2.csv", target / "monitor-pre-window-attempt2.csv"),
                (EV + "/monitor/pre-window-snapshot-attempt2.txt", target / "attempt2/pre-window-snapshot.txt"),
                (EV + "/materials/dryrun-plan-attempt2.json", target / "attempt2/dryrun-plan.json"),
                (EV + "/monitor/exception-monitor-recompute.json", target / "attempt2/exception-monitor-recompute.json")):
    retained[evp.split("/")[-1]] = bool(tp.exists() and sha(tp.read_bytes()) == sha(blob(PIN, evp)))
faillog = txt(PIN, EV + "/attempt1/capture-failure.txt")
check("attempt1.retained",
      all(retained.values()) and "capture_exit=1" in faillog
      and "expected exactly one scalar hinge joint" in faillog
      and "products_written=none" in faillog,
      {"target_copies_byte_equal": retained,
       "failure_excerpt": [l for l in faillog.splitlines() if "capture_exit" in l or "failure_message" in l or "products_written" in l]})

# 7. 范围 / 材料同步 / 证据 manifest
paths_347 = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{PRE}..{PIN}").decode().splitlines() if l.strip()]
scripts = [p for p in paths_347 if p.startswith("scripts/")]
fixtures = [p for p in paths_347 if p.startswith("fixtures/")]
bad = [p for p in paths_347 if p.startswith(("src/", "tests/"))]
sha_sync = {}
for p in (EVW + "/README.md", EVW + "/materials/window-plan.txt", EVW + "/materials/expected-products.json",
          EVA + "/receipt.json"):
    t = txt(PIN, p)
    sha_sync[p.split("/")[-1]] = (NEW_SCRIPT_SHA in t)
man = txt(PIN, EV + "/manifest.txt")
mrows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
mok = 0
for s2, n2, p2 in mrows:
    b = blob(PIN, p2)
    if sha(b) == s2 and len(b) == int(n2):
        mok += 1
evw_files = git("ls-tree", "-r", "--name-only", PIN, EVW).decode().splitlines()
check("scope.materials_manifest",
      not bad and scripts == [SCRIPT] and sorted(fixtures) == sorted(
          [f"{PROD}/{k}" for k in ("rgb.npz", "depth.npz", "seg.npz", "state.npz", "capture.json")])
      and all(sha_sync.values()) and len(mrows) == 14 and mok == 14
      and len(evw_files) == 4,
      {"scripts": scripts, "fixtures": sorted(fixtures), "bad": bad,
       "script_sha_sync_in_4_files": sha_sync,
       "evidence_manifest": {"rows": len(mrows), "match": mok},
       "window_material_files": len(evw_files),
       "note": "per-world 窗口材料目录没有 manifest.txt（只有 README/materials/receipt），记为观察"})

# 8. 卡面检查 / validate
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
       "card_head": "e0988285", "checks": checks, "problems": problems,
       "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                   "problems": len(problems), "blockers": 0}}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
