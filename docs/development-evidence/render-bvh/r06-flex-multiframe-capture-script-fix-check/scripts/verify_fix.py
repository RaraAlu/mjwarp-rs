#!/usr/bin/env python3
"""QA minimal review of the R06 multiframe capture-script O3 fix (and O1/O2/O4/O5).

Objects: fix commit 3e5fc677 (module + scripts/authoring/README.md) and
old-evidence fix commit 1678545e (Normify-Fixes on the implementation card).
Read-only, CPU-only (no GPU, no capture). Writes only <tmp> (target/...).

Usage: python verify_fix.py [--out JSON] [--tmp DIR]
Exit 0 = all FAIL checks passed; OBS lines are observations only.
"""
import argparse
import ast
import contextlib
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import subprocess
import sys

REPO = os.getcwd()
BASE = "e1d93aca"          # previous QA review commit (main script baseline)
FIX = "3e5fc677"           # O3 + O2 fix
OLDFIX = "1678545e"        # O1/O4/O5 old-evidence fix (Normify-Fixes)
MAIN = "scripts/authoring/capture_render_flex.py"
MODULE = "scripts/authoring/capture_render_flex_multiframe.py"
OLD_EVID = "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-script"
FIX_EVID = "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-script-fix"
SPEC = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SPEC_SHA = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
SPEC_BYTES = 10506
fails = []


def sh(*a, cwd=REPO):
    return subprocess.run(["git"] + list(a), cwd=cwd, capture_output=True)


def blob(rev, path):
    return sh("show", rev + ":" + path).stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def ok(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + (" :: " + detail if detail else ""))
    if not cond:
        fails.append(label)


def obs(label, detail=""):
    print("OBS  " + label + (" :: " + detail if detail else ""))


def run_cli(args):
    r = subprocess.run([sys.executable, "-B", MAIN] + args, cwd=REPO,
                       capture_output=True, text=True, encoding="utf-8")
    return r.returncode, (r.stdout or ""), (r.stderr or "")


ap = argparse.ArgumentParser()
ap.add_argument("--out", default="")
ap.add_argument("--tmp", default=os.path.join(REPO, "target", "qa-multiframe-fix-rerun"))
a = ap.parse_args()
tmp = os.path.abspath(a.tmp)
res = {"base_commit": BASE, "fix_commit": FIX, "old_evidence_fix_commit": OLDFIX,
       "head": sh("rev-parse", "--short", "HEAD").stdout.decode().strip()}
print("== base", BASE, "fix", FIX, "oldfix", OLDFIX, "HEAD", res["head"])

# ---- 1. commit scope + trailers ----
print("\n[1] commit scope and trailers")
body_fix = sh("show", "-s", "--format=%B", FIX).stdout.decode()
body_old = sh("show", "-s", "--format=%B", OLDFIX).stdout.decode()
res["trailers"] = {
    "fix": [l for l in body_fix.splitlines() if l.startswith("Normify-")],
    "old": [l for l in body_old.splitlines() if l.startswith("Normify-")],
}
ok("fix commit carries exactly one Normify-Task (its own card), no Normify-Fixes",
   res["trailers"]["fix"] == ["Normify-Task: 2026-11-05-render-r06-flex-multiframe-capture-script-fix"],
   str(res["trailers"]["fix"]))
ok("old-evidence commit carries exactly one Normify-Fixes (old card), no Normify-Task",
   res["trailers"]["old"] == ["Normify-Fixes: 2026-11-05-render-r06-flex-multiframe-capture-script"],
   str(res["trailers"]["old"]))
fix_files = [p for p in sh("show", "--name-only", "--format=", FIX).stdout.decode().splitlines() if p]
old_files = [p for p in sh("show", "--name-only", "--format=", OLDFIX).stdout.decode().splitlines() if p]
res["fix_files"], res["old_files"] = fix_files, old_files
ok("fix commit touches only scripts/authoring(+module/README) and fix evidence/card/runs",
   all(p.startswith((".normify/", FIX_EVID + "/")) or p in (MAIN, MODULE, "scripts/authoring/README.md")
       for p in fix_files), str([p for p in fix_files if not (p.startswith((".normify/", FIX_EVID + "/")) or p in (MAIN, MODULE, "scripts/authoring/README.md"))]))
ok("fix commit does not touch the old evidence dir", not any(p.startswith(OLD_EVID + "/") for p in fix_files))
ok("old-evidence commit touches only the old evidence dir + card/runs",
   all(p.startswith((".normify/", OLD_EVID + "/")) for p in old_files),
   str([p for p in old_files if not p.startswith((".normify/", OLD_EVID + "/"))]))
ok("neither commit touches src/tests/fixtures/Cargo",
   not any(p.startswith(("src/", "tests/", "fixtures/", "Cargo")) for p in fix_files + old_files))
ok("no npz/capture artifact committed by either commit",
   not any(p.endswith(".npz") for p in fix_files + old_files))

# ---- 2. 500-line ratchet / unchanged main script / spec ----
print("\n[2] line-count ratchet, main script unchanged, frozen spec")
main_base = blob(BASE, MAIN)
main_head = blob("HEAD", MAIN)
mod_head = blob("HEAD", MODULE)
res["lines"] = {MAIN: main_head.count(b"\n"), MODULE: mod_head.count(b"\n")}
ok("capture_render_flex.py byte-identical to QA baseline %s" % BASE, main_base == main_head)
ok("capture_render_flex.py = 498 lines (git blob / wc -l)", main_head.count(b"\n") == 498,
   str(main_head.count(b"\n")))
ok("extension module = 479 lines", mod_head.count(b"\n") == 479, str(mod_head.count(b"\n")))
num = [l for l in sh("show", "--numstat", "--format=", FIX, "--", "scripts/authoring/README.md").stdout.decode().splitlines()]
ok("scripts/authoring/README.md +4/-0", num == ["4\t0\tscripts/authoring/README.md"], str(num))
spec = blob("HEAD", SPEC)
ok("frozen spec sha/bytes unchanged", sha(spec) == SPEC_SHA and len(spec) == SPEC_BYTES,
   "%s / %dB" % (sha(spec)[:12], len(spec)))
ok("spec not touched by either commit", SPEC not in fix_files + old_files)
ok("fixtures/.../scene-flex/multiframe absent",
   not os.path.exists("fixtures/render-bvh/scene-flex/multiframe"))

# ---- 3. O3: static wiring in capture_multiframe (AST) ----
print("\n[3] O3 static wiring (AST)")
mod_text = mod_head.decode()
tree = ast.parse(mod_text)
fn = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef) and n.name == "capture_multiframe")
calls = [n for n in ast.walk(fn) if isinstance(n, ast.Call)]
call_names = [c.func.id if isinstance(c.func, ast.Name) else getattr(c.func, "attr", None) for c in calls]
ok("capture_multiframe calls _delta_report", "_delta_report" in call_names, str(sorted(set(x for x in call_names if x))))
ok("capture_multiframe calls _print_delta_report", "_print_delta_report" in call_names)
ok("capture_multiframe calls delta_exit_code", "delta_exit_code" in call_names)
stream_false = any(isinstance(c, ast.Call) and getattr(c.func, "id", "") == "_delta_report"
                   and any(k.arg == "stream" and getattr(k.value, "value", None) is False for k in c.keywords)
                   for c in calls)
ok("_delta_report is called with stream=False in capture path", stream_false)
last = fn.body[-1]
ok("capture_multiframe ends with 'return delta_exit_code(...)'",
   isinstance(last, ast.Return) and isinstance(last.value, ast.Call)
   and getattr(last.value.func, "id", "") == "delta_exit_code")
bare_zero = [n for n in ast.walk(fn) if isinstance(n, ast.Return) and isinstance(n.value, ast.Constant)
             and n.value.value == 0]
ok("no unconditional 'return 0' left in capture_multiframe", bare_zero == [], str(len(bare_zero)))
selfcheck = None
for n in ast.walk(fn):
    if isinstance(n, ast.Dict) and any(isinstance(k, ast.Constant) and k.value == "delta_selfcheck" for k in n.keys):
        selfcheck = n
subkeys = [k.value for k in selfcheck.keys] if selfcheck else []
ok("capture.json dict has 'delta_selfcheck' key", selfcheck is not None)
val = next((v for k, v in zip(selfcheck.keys, selfcheck.values)
            if isinstance(k, ast.Constant) and k.value == "delta_selfcheck"), None)
vkeys = [k.value for k in val.keys] if isinstance(val, ast.Dict) else []
ok("delta_selfcheck = {thresholds, cam2_pixels, report, overall}",
   vkeys == ["thresholds", "cam2_pixels", "report", "overall"], str(vkeys))
res["delta_selfcheck_keys"] = vkeys

# ---- 4. O3: functional demonstration (same functions the capture path runs) ----
print("\n[4] O3 functional: warning + exit code per verdict (synthetic data, no GPU)")
spec_imp = importlib.util.spec_from_file_location("mfm", os.path.join(REPO, MODULE))
mfm = importlib.util.module_from_spec(spec_imp)
spec_imp.loader.exec_module(mfm)
import numpy as np

shutil.rmtree(tmp, ignore_errors=True)
os.makedirs(tmp, exist_ok=True)
res["o3_functional"] = {}
for name, p1, p2, want in (("pass", 600, 600, 0), ("insufficient", 600, 500, 3),
                           ("non-discriminating", 600, 300, 3)):
    root = os.path.join(tmp, name)
    depth = [np.zeros((240, 320), dtype=np.float32) for _ in range(3)]
    seg = [np.zeros((240, 320, 2), dtype=np.int32) for _ in range(3)]
    depth[1].reshape(-1)[:p1] = 1.0
    seg[1].reshape(-1, 2)[:p1, 0] = 5
    depth[2] = depth[1].copy()
    seg[2] = seg[1].copy()
    depth[2].reshape(-1)[:p2] = 2.0
    seg[2].reshape(-1, 2)[:p2, 0] = 7
    for f in range(3):
        d = os.path.join(root, "frame-%03d" % f)
        os.makedirs(d, exist_ok=True)
        np.savez(os.path.join(d, "depth.npz"), depth_cam2=depth[f])
        np.savez(os.path.join(d, "seg.npz"), seg_cam2=seg[f])
    report = mfm._delta_report(__import__("pathlib").Path(root), None, stream=False)
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        mfm._print_delta_report(report)
    code = mfm.delta_exit_code(report)
    res["o3_functional"][name] = {"verdict": report["overall"], "exit": code,
                                  "warning": "不得落库" in buf.getvalue()}
    ok("%s: verdict=%s -> exit %d, warning=%s" % (name, report["overall"], want,
                                                  "不得落库" in buf.getvalue()),
       report["overall"] in ("pass-margin", "insufficient-margin", "non-discriminating")
       and code == want and (("不得落库" in buf.getvalue()) == (name != "pass")))
    c, out, err = run_cli(["--check-deltas", "--out-dir", root])
    ok("--check-deltas standalone keeps exit %d for %s" % (want, name), c == want, "exit=%d" % c)

# ---- 5. O2: --frames whitelist ----
print("\n[5] O2: --frames whitelist")
never = os.path.join(tmp, "never")
res["o2"] = {}
for v in ("0", "-1", "2"):
    c, out, err = run_cli(["--frames", v, "--out-dir", never, "--dry-run"])
    res["o2"]["frames" + v] = {"exit": c, "stderr_has_reason": "只接受" in (out + err)}
    ok("--frames %s exit 1 with reason" % v, c == 1 and "只接受" in (out + err),
       "exit=%d" % c)
c, out, err = run_cli(["--frames", "1", "--dry-run"])
res["o2"]["frames1"] = {"exit": c, "single_frame_path": "multiframe-dry-run" not in out
                        and "render-flex-context" in out or "r06-s3b" in out}
ok("--frames 1 still takes the original single-frame path (exit 0, no multiframe plan)",
   c == 0 and "multiframe-dry-run" not in out, "exit=%d" % c)
ok("--frames 1 dry-run output is not the multiframe plan", '"mode": "multiframe-dry-run"' not in out)

# ---- 6. O1/O4/O5 old-evidence fixes ----
print("\n[6] O1/O4/O5 old-evidence fixes (commit %s)" % OLDFIX)
try:
    dry = json.loads(blob("HEAD", OLD_EVID + "/logs/dry-run-multiframe.json").decode())
    ok("O1: dry-run-multiframe.json is pure JSON (json.load OK)",
       dry.get("mode") == "multiframe-dry-run" and dry.get("gpu_used") is False)
except Exception as e:
    ok("O1: dry-run-multiframe.json is pure JSON (json.load OK)", False, str(e))
ok("O1: stderr split into dry-run-multiframe.stderr.txt",
   b"WARNING" in blob("HEAD", OLD_EVID + "/logs/dry-run-multiframe.stderr.txt"))
old_readme = blob("HEAD", OLD_EVID + "/README.md").decode()
ok("O4: old README artifact row now says '+7 行 / 0 删除'",
   "+7 行 / 0 删除" in old_readme)
ok("O4: any remaining '+5 行' is only the historical quote in the revision record",
   old_readme.count("+5 行") == 1 and "由「+5 行」更正为「+7 行 / 0 删除」" in old_readme,
   "occurrences=%d" % old_readme.count("+5 行"))
old_receipt = json.loads(blob("HEAD", OLD_EVID + "/receipt.json").decode())
ok("O5: implementation receipt status=verified, commit=5a99047f",
   old_receipt.get("status") == "verified" and old_receipt.get("commit") == "5a99047f",
   "%r" % old_receipt.get("status"))
for label, d in (("old implementation evidence", OLD_EVID), ("fix evidence", FIX_EVID)):
    man = blob("HEAD", d + "/manifest.txt")
    n = okc = 0
    for line in man.decode().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        h, b, rel = line.split()
        data = blob("HEAD", d + "/" + rel)
        okc += (sha(data) == h and str(len(data)) == b)
        n += 1
    ok("%s manifest self-check %d/%d" % (label, okc, n), n > 0 and okc == n)

# ---- 7. cards ----
print("\n[7] cards and evidence")
for cid in ("2026-11-05-render-r06-flex-multiframe-capture-script-fix",
            "2026-11-05-render-r06-flex-multiframe-capture-script"):
    card = json.loads(blob("HEAD", ".normify/changes/" + cid + ".json").decode())
    ok("card %s verified" % cid, card["status"] == "verified", card["status"])
obs("capture-path exit code is verified statically + by the shared functions; no GPU capture was run "
    "(card allows synthetic/stub demonstration); dynamic exit-3 on a real capture is exercised in the capture window")

res["fails"] = fails
if a.out:
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        json.dump(res, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print("wrote", a.out)
print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
