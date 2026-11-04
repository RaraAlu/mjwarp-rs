#!/usr/bin/env python3
"""QA read-only check of the R06 multi-frame capture-script implementation.

Object: commit 5a99047f (capture_render_flex.py additive dispatch + new
capture_render_flex_multiframe.py) against the frozen spec
`r06-flex-multiframe-criterion/spec.md` (sha256 1d9fa5ce... / 10506 B).

Runs CPU-only checks (help / dry-run / parameter errors / delta self-check on
synthetic data); no GPU, no capture, no repo writes outside <tmp> (target/).

Usage: python verify_capture_script.py [--out JSON] [--tmp DIR]
Exit 0 = all FAIL checks passed; OBS lines are observations, not failures.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

REPO = os.getcwd()
COMMIT = "5a99047f"
SPEC_REL = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SPEC_SHA = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
SPEC_BYTES = 10506
SCRIPT = "scripts/authoring/capture_render_flex.py"
MODULE = "scripts/authoring/capture_render_flex_multiframe.py"
EVID = "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-script"
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
    r = subprocess.run([sys.executable, "-B", SCRIPT] + args, cwd=REPO,
                       capture_output=True, text=True, encoding="utf-8")
    return r.returncode, (r.stdout or "") + (r.stderr or "")


def parse_json_from(out):
    idx = out.find("{")
    if idx < 0:
        return None
    return json.JSONDecoder().raw_decode(out[idx:])[0]


ap = argparse.ArgumentParser()
ap.add_argument("--out", default="")
ap.add_argument("--tmp", default=os.path.join(REPO, "target", "qa-multiframe-rerun"))
a = ap.parse_args()
res = {"object_commit": COMMIT, "repo_head": sh("rev-parse", "--short", "HEAD").stdout.decode().strip()}

print("== object", COMMIT, "HEAD", res["repo_head"])

# ---- 1. script identity / additive diff ----
print("\n[1] scripts: identity, line counts, additive-only diff")
base = blob("HEAD", SCRIPT)
mod = blob("HEAD", MODULE)
res["line_counts_git_blob"] = {SCRIPT: base.count(b"\n"), MODULE: mod.count(b"\n")}
ok("capture_render_flex.py = 498 lines (git blob / wc -l)",
   base.count(b"\n") == 498 and base.endswith(b"\n"), str(base.count(b"\n")))
ok("capture_render_flex_multiframe.py = 446 lines (git blob / wc -l)",
   mod.count(b"\n") == 446 and mod.endswith(b"\n"), str(mod.count(b"\n")))
ok("both scripts LF-only, no BOM",
   b"\r" not in base and b"\r" not in mod and not base.startswith(b"\xef\xbb\xbf"))
receipt = json.loads(blob("HEAD", EVID + "/receipt.json").decode())
for rel, data in ((SCRIPT, base), (MODULE, mod)):
    expect = receipt["scripts"]["baseline" if rel == SCRIPT else "module"]["sha256"]
    ok("sha256 matches receipt: " + rel, sha(data) == expect)
num = [l for l in sh("show", "--numstat", "--format=", COMMIT, "--", "scripts/authoring/").stdout.decode().splitlines()]
res["numstat"] = num
ok("additive only: +7/-0 on capture_render_flex.py", "7\t0\t" + SCRIPT in num, str(num))
added = [l for l in sh("diff", COMMIT + "^", COMMIT, "--", SCRIPT, "--text").stdout.decode().splitlines()
         if l.startswith("+") and not l.startswith("+++")]
removed = [l for l in sh("diff", COMMIT + "^", COMMIT, "--", SCRIPT, "--text").stdout.decode().splitlines()
           if l.startswith("-") and not l.startswith("---")]
ok("no deleted lines in baseline script", removed == [], str(removed))
ok("dispatch block present (returns None on single-frame path)",
   any("dispatch_multiframe" in l for l in added) and any("add_frame_args(parser)" in l for l in added))

# ---- 2. spec correspondence (static) ----
print("\n[2] frozen spec correspondence")
spec = blob("HEAD", SPEC_REL)
res["spec"] = {"sha256": sha(spec), "bytes": len(spec)}
ok("spec sha256/bytes match frozen values", sha(spec) == SPEC_SHA and len(spec) == SPEC_BYTES,
   "%s / %d B" % (sha(spec)[:12], len(spec)))
mod_text = mod.decode()
for token in ["FRAMES = 3", "FRAME_DZ = 0.10", 'ANCHOR_BODIES = ("flex-anchor0", "flex-anchor1", "flex-anchor2")',
              "DELTA_ACCEPT_MIN = 384", "DELTA_MARGIN_MIN = 576", "CAM2_INDEX = 2",
              "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6", "10506"]:
    ok("module constant: " + token, token in mod_text)
ok("frame offset k*FRAME_DZ applied to anchor body_pos.z",
   "offset = frame * FRAME_DZ" in mod_text and "body_pos[body, 2] = mjm.body_pos[body, 2] + offset" in mod_text)
ok("no physics stepping (no mj_step / mjw.step)",
   "mj_step" not in mod_text and "mjw.step" not in mod_text and "mj_forward" in mod_text)
ok("per-frame layout frame-xxx/{rgb,depth,seg,state}.npz",
   'f"frame-{frame:03d}"' in mod_text and '{name}.npz' in mod_text)
ok("top-level capture.json / cam-poses.json / README.md",
   all(t in mod_text for t in ['capture.json', 'cam-poses.json', 'README.md']))
ok("only 3 new CLI options and no threshold CLI",
   len(re.findall(r"add_argument\(", mod_text)) == 3 and
   all(t not in mod_text for t in ["--delta", "--threshold", "--accept-min", "--margin"]))
ok("no skip/force switch", not re.search(r"add_argument\([^)]*(skip|force|no-check)", mod_text))

# ---- 3. commit scope / zero capture ----
print("\n[3] commit scope and zero-capture boundary")
names = [p for p in sh("show", "--name-only", "--format=", COMMIT).stdout.decode().splitlines() if p]
res["commit_files"] = names
bad = [p for p in names if not (p.startswith(".normify/") or p.startswith(EVID + "/")
                                or p in (SCRIPT, MODULE, "scripts/authoring/README.md"))]
ok("commit touches only scripts/authoring + evidence + card/runs", bad == [], str(bad))
ok("no src/tests/fixtures/Cargo change",
   not any(p.startswith(("src/", "tests/", "fixtures/", "Cargo")) for p in names))
ok("no npz/capture artifact committed", not any(p.endswith(".npz") for p in names))
ok("fixtures/.../scene-flex/multiframe absent",
   not os.path.exists("fixtures/render-bvh/scene-flex/multiframe"))
ok("spec untouched by commit", SPEC_REL not in names)

# ---- 4. implementation evidence ----
print("\n[4] implementation evidence self-check")
man = blob("HEAD", EVID + "/manifest.txt")
n = okc = 0
for line in man.decode().splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    data = blob("HEAD", EVID + "/" + rel)
    okc += (sha(data) == h and str(len(data)) == b)
    n += 1
ok("implementation manifest self-check 16/16", n == 16 and okc == 16, "%d/%d" % (okc, n))
readme = blob("HEAD", EVID + "/README.md").decode()
ok("evidence documents static single-frame basis (no dynamic claim)", "静态判定" in readme)
ok("evidence non-claims: implementation != capture/acceptance",
   "不产生任何验收结论" in readme and "未执行采集" in readme)
enc_bad = []
for rel in names:
    if rel.startswith(EVID + "/") and rel.endswith((".md", ".json", ".txt", ".diff")):
        b = blob("HEAD", rel)
        try:
            b.decode("utf-8")
            badenc = b[:3] == b"\xef\xbb\xbf" or b[:2] in (b"\xff\xfe", b"\xfe\xff") or b"\x00" in b
        except UnicodeDecodeError:
            badenc = True
        if badenc:
            enc_bad.append(rel)
ok("evidence text files UTF-8, no BOM/NUL", enc_bad == [], str(enc_bad))
obs("logs/dry-run-multiframe.json has a PowerShell ErrorRecord preamble before the JSON object",
    "first '{' at line 13; body parses fine from there, file itself is not valid JSON")
obs("evidence README says capture-script.diff is '+5 行'; actual diff is +7/-0 (artifact content is correct)",
    "README section 6")
obs("receipt.json status is 'implemented_pending_commit' (card is verified/committed)")

# ---- 5. CPU reruns ----
print("\n[5] CPU reruns (no GPU, no capture)")
tmp = os.path.abspath(a.tmp)
shutil.rmtree(tmp, ignore_errors=True)
os.makedirs(tmp, exist_ok=True)
code, out = run_cli(["--help"])
ok("--help exit 0 and lists 3 new options",
   code == 0 and all(t in out for t in ("--frames", "--frame-dz", "--check-deltas")))
never = os.path.join(tmp, "never")
code, out = run_cli(["--frames", "3", "--frame-dz", "0.1", "--out-dir", never, "--dry-run"])
plan = parse_json_from(out) if code == 0 else None
res["dry_run"] = {"exit": code, "spec_sha": (plan or {}).get("spec", {}).get("sha256"),
                  "anchor_ids": (plan or {}).get("anchor_body_ids"),
                  "base_z": (plan or {}).get("anchor_body_base_z"),
                  "delta_constants": (plan or {}).get("delta_constants"),
                  "gpu_used": (plan or {}).get("gpu_used"),
                  "wrote_out_dir": os.path.exists(never)}
ok("dry-run exit 0, spec sha/bytes and frozen values echoed",
   code == 0 and res["dry_run"]["spec_sha"] == SPEC_SHA and res["dry_run"]["delta_constants"] == {"accept_min": 384, "margin_min": 576})
ok("dry-run wrote nothing and reports gpu_used=false",
   res["dry_run"]["wrote_out_dir"] is False and res["dry_run"]["gpu_used"] is False)

param_cases = [
    ("frames=2", ["--frames", "2", "--out-dir", never, "--dry-run"]),
    ("dz=0.2", ["--frames", "3", "--frame-dz", "0.2", "--out-dir", never, "--dry-run"]),
    ("no out-dir", ["--frames", "3", "--frame-dz", "0.1", "--dry-run"]),
    ("repeat=1", ["--frames", "3", "--frame-dz", "0.1", "--out-dir", never, "--repeat", "1"]),
    ("check-deltas no out-dir", ["--check-deltas"]),
    ("check-deltas + frames", ["--check-deltas", "--out-dir", never, "--frames", "3"]),
    ("check-deltas missing dir", ["--check-deltas", "--out-dir", os.path.join(tmp, "nope")]),
    ("check-deltas empty dir", ["--check-deltas", "--out-dir", os.path.join(tmp, "empty")]),
]
os.makedirs(os.path.join(tmp, "empty"), exist_ok=True)
res["param_exits"] = {}
for label, argv in param_cases:
    c, _ = run_cli(argv)
    res["param_exits"][label] = c
    ok("rejects " + label + " (exit 1)", c == 1, "exit=%d" % c)

frames0, _ = run_cli(["--frames", "0", "--dry-run"])
res["frames_zero_exit"] = frames0
obs("--frames 0 falls back to the single-frame path with exit 0 (no explicit rejection)",
    "frames<=1 are not validated against (1,3); no spec change possible via this path")

# synth delta data (shape/keys must match base cam2 arrays)
import numpy as np


def synth(name, pair1, pair2):
    root = os.path.join(tmp, name)
    depth = [np.zeros((240, 320), dtype=np.float32) for _ in range(3)]
    seg = [np.zeros((240, 320, 2), dtype=np.int32) for _ in range(3)]
    depth[1].reshape(-1)[:pair1] = 1.0
    seg[1].reshape(-1, 2)[:pair1, 0] = 5
    depth[2] = depth[1].copy()
    seg[2] = seg[1].copy()
    depth[2].reshape(-1)[:pair2] = 2.0
    seg[2].reshape(-1, 2)[:pair2, 0] = 7
    for f in range(3):
        d = os.path.join(root, "frame-%03d" % f)
        os.makedirs(d, exist_ok=True)
        np.savez(os.path.join(d, "depth.npz"), depth_cam2=depth[f])
        np.savez(os.path.join(d, "seg.npz"), seg_cam2=seg[f])
    return root


def counts(root):
    d = [np.load(os.path.join(root, "frame-%03d" % f, "depth.npz"))["depth_cam2"] for f in range(3)]
    s = [np.load(os.path.join(root, "frame-%03d" % f, "seg.npz"))["seg_cam2"] for f in range(3)]
    out = []
    for x, y in ((0, 1), (1, 2)):
        out.append((int(np.count_nonzero(d[x].view(np.uint32) != d[y].view(np.uint32))),
                    int(np.count_nonzero(np.any(s[x] != s[y], axis=2)))))
    return out


res["delta"] = {}
for name, p1, p2, want_exit, want_verdict in (
        ("synth-pass", 600, 600, 0, "pass-margin"),
        ("synth-insufficient", 600, 500, 3, "insufficient-margin"),
        ("synth-non-discriminating", 600, 300, 3, "non-discriminating")):
    root = synth(name, p1, p2)
    c, out = run_cli(["--check-deltas", "--out-dir", root])
    verdict = next((l.split("overall:")[1].strip() for l in out.splitlines() if "overall:" in l), "")
    res["delta"][name] = {"exit": c, "counts": counts(root), "verdict": verdict}
    ok("%s: exit %d / %s" % (name, want_exit, want_verdict), c == want_exit and verdict == want_verdict,
       "counts=%s verdict=%s" % (counts(root), verdict))
ok("check-deltas prints stop warning when not pass-margin",
   "不得落库" in run_cli(["--check-deltas", "--out-dir", os.path.join(tmp, "synth-insufficient")])[1])
obs("capture_multiframe computes the delta report with stream=False and returns 0 even when the "
    "verdict is not pass-margin (explicit warning/exit only via --check-deltas); evidence README "
    "section 2/§8.5 claims a printed warning in the capture path")

# ---- 6. line-count convention ----
print("\n[6] line-count convention (resolves Arch vs Backend numbers)")
ok("authoritative counts = git blob newline count = wc -l = 498 / 446",
   res["line_counts_git_blob"] == {SCRIPT: 498, MODULE: 446})
scan = blob("HEAD", "docs/development-evidence/split-program/scan-oversized.ps1").decode()
m = re.search(r"\$exts\s*=\s*@\(([^)]*)\)", scan)
exts = m.group(1) if m else ""
res["oversized_scan_exts"] = exts
ok("documented 500-line scan covers rs/cu/c/h/cpp/sh/ps1 only (.py not scanned)",
   all(t in exts for t in (".rs", ".cu", ".c", ".h", ".cpp", ".sh", ".ps1")) and ".py" not in exts,
   exts)
obs("PS 5.1 Get-Content default ANSI decoding undercounts UTF-8 files (observed 496/432); "
    "use wc -l / git blob / [IO.File]::ReadAllLines; capture_render_flex.py 498 is 2 lines below 500")

res["fails"] = fails
if a.out:
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        json.dump(res, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print("wrote", a.out)
print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
