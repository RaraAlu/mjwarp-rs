#!/usr/bin/env python3
"""QA read-only review of the R06 multi-frame capture-window negative result.

Object: card 2026-11-05-render-r06-flex-multiframe-capture (verified;
commits 484852ca / dbe8d2b7 / 27474ac5) and evidence dir
docs/development-evidence/render-bvh/r06-flex-multiframe-capture/.

Read-only; no GPU, no capture, no writes outside <out>/<tmp> (target/).

Usage: python verify_capture_window.py [--out JSON]
Exit 0 = all FAIL checks passed; OBS lines are observations only.
"""
import argparse
import csv
import hashlib
import json
import os
import re
import subprocess
import sys

REPO = os.getcwd()
EVID = "docs/development-evidence/render-bvh/r06-flex-multiframe-capture"
CARD = "2026-11-05-render-r06-flex-multiframe-capture"
BASE_COMMIT = "b43d8295"   # QA review baseline before the window
SPEC = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SPEC_SHA = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
SPEC_BYTES = 10506
FLOOR = 11688
fails = []


def sh(*a):
    return subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)


def blob(rev, path):
    return sh("show", rev + ":" + path).stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def rb(p):
    return open(os.path.join(REPO, p), "rb").read()


def ok(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + (" :: " + detail if detail else ""))
    if not cond:
        fails.append(label)


def obs(label, detail=""):
    print("OBS  " + label + (" :: " + detail if detail else ""))


def decode(b):
    for e in ("utf-8", "cp936", "utf-16"):
        try:
            return b.decode(e), e
        except Exception:
            pass
    return b.decode("latin-1"), "latin-1"


def parse_json_from(text):
    idx = text.find("{")
    return json.JSONDecoder().raw_decode(text[idx:])[0]


ap = argparse.ArgumentParser()
ap.add_argument("--out", default="")
a = ap.parse_args()
res = {"head": sh("rev-parse", "--short", "HEAD").stdout.decode().strip(),
       "baseline_commit": BASE_COMMIT}
print("== HEAD", res["head"], "| status clean:", sh("status", "--porcelain").stdout.decode().strip() == "")

# ---- 1. commit scope / boundaries ----
print("\n[1] commit scope (b43d8295..HEAD) and boundaries")
rows = [l for l in sh("diff-tree", "-r", "--name-status", BASE_COMMIT, "HEAD").stdout.decode().splitlines()]
files = [r.split("\t")[-1] for r in rows]
res["commit_files"] = files
ok("window commits touch only the card + capture evidence dir",
   all(p.startswith((".normify/", EVID + "/")) for p in files),
   str([p for p in files if not p.startswith((".normify/", EVID + "/"))]))
ok("no src/tests/fixtures/scripts/spec changes in window commits",
   not any(p.startswith(("src/", "tests/", "fixtures/", "scripts/", SPEC)) for p in files))
ok("no npz/frame artifacts committed by window commits",
   not any(p.endswith(".npz") for p in files))
ok("existing base reference untouched by window commits",
   sh("diff", "--name-only", BASE_COMMIT, "HEAD", "--", "fixtures/").stdout.decode().strip() == "")
spec = blob("HEAD", SPEC)
ok("frozen spec sha/bytes unchanged", sha(spec) == SPEC_SHA and len(spec) == SPEC_BYTES,
   "%s / %dB" % (sha(spec)[:12], len(spec)))

# ---- 2. negative result: capture.log summary ----
print("\n[2] negative result (capture.log summary, cp936 decoded)")
capture_text, cap_enc = decode(rb(EVID + "/capture.log"))
cap = parse_json_from(capture_text)
res["capture_log"] = {"encoding": cap_enc, "summary": {
    "frames": cap.get("frames"), "frame_dz": cap.get("frame_dz"),
    "two_captures": cap.get("two_captures_bitwise_identical"),
    "legacy_untouched": cap.get("legacy_reference_untouched"),
    "delta_verdict": cap.get("delta_verdict"), "delta_report": cap.get("delta_report"),
    "files": len(cap.get("files_sha256") or {})}}
r_ = cap["delta_report"]
ok("capture summary has 3 frames / dz 0.1", cap.get("frames") == 3 and cap.get("frame_dz") == 0.1)
ok("000->001 = 697/616 pass-margin",
   r_[0] == {"pair": "000->001", "depth_changed": 697, "seg_changed": 616, "verdict": "pass-margin"}, str(r_[0]))
ok("001->002 = 244/183 non-discriminating",
   r_[1] == {"pair": "001->002", "depth_changed": 244, "seg_changed": 183, "verdict": "non-discriminating"}, str(r_[1]))
ok("overall = non-discriminating", cap.get("delta_verdict") == "non-discriminating")
ok("changes below 384 on the second pair (negative-result root cause)",
   r_[1]["depth_changed"] < 384 and r_[1]["seg_changed"] < 384)
ok("two captures bitwise identical = true", cap.get("two_captures_bitwise_identical") is True)
ok("legacy single-frame reference untouched = true", cap.get("legacy_reference_untouched") is True)
ok("'do not land' warning printed", "不得落库" in capture_text)
ok("single capture run (one summary, no retry)",
   capture_text.count('"two_captures_bitwise_identical"') == 1,
   "count=%d" % capture_text.count('"two_captures_bitwise_identical"'))
obs("capture.log is script stdout only: no command line and no explicit exit code; command/exit=3 are recorded in receipt/README and follow from the reviewed O3 code (delta_verdict=non-discriminating -> delta_exit_code=3)",
    "log has %d lines" % capture_text.count("\n"))

# ---- 3. dry-run ----
print("\n[3] dry-run")
dry_text, dry_enc = decode(rb(EVID + "/dry-run.log"))
dry = parse_json_from(dry_text)
res["dry_run"] = {"encoding": dry_enc, "mode": dry.get("mode"), "spec_sha": (dry.get("spec") or {}).get("sha256"),
                  "anchor_ids": dry.get("anchor_body_ids"), "base_z": dry.get("anchor_body_base_z"),
                  "repeat": dry.get("repeat"), "gpu_used": dry.get("gpu_used")}
ok("dry-run mode + spec sha + anchors + repeat=2 + gpu_used=false",
   dry.get("mode") == "multiframe-dry-run" and res["dry_run"]["spec_sha"] == SPEC_SHA
   and dry.get("anchor_body_ids") == [1, 2, 3] and dry.get("anchor_body_base_z") == [0.0, 0.0, 0.0]
   and dry.get("repeat") == 2 and dry.get("gpu_used") is False)
ok("dry-run expected layout = three frames + 3 top-level files",
   len(dry.get("expected_layout") or []) == 6, str(dry.get("expected_layout")))

# ---- 4. baseline / monitor ----
print("\n[4] baseline and monitor")
pts = []
for line in rb(EVID + "/baseline-three-points.txt").decode("utf-8-sig").splitlines():
    p = [x.strip() for x in line.split(",")]
    pts.append({"used": int(p[0].split()[0]), "free": int(p[1].split()[0]),
                "util": int(p[2].split()[0]), "temp": int(p[3])})
res["baseline"] = {"points": pts}
ok("baseline = 3 points within 5 min (file has 3 lines)", len(pts) == 3)
ok("baseline every point free >= floor 11688 and util < 40",
   all(x["free"] >= FLOOR and x["util"] < 40 for x in pts), str(pts))
floor = max(min(x["free"] for x in pts) - 3072, FLOOR)
ok("floor formula max(min_free-3072, 11688) = 11688", floor == FLOOR, str(floor))
mrows = list(csv.DictReader(open(os.path.join(REPO, EVID, "monitor-capture.csv"), encoding="utf-8-sig")))
res["monitor"] = {"rows": len(mrows),
                  "util_max": max(int(r["util_percent"]) for r in mrows),
                  "free_min": min(int(r["free_mib"]) for r in mrows),
                  "below_floor": sum(1 for r in mrows if int(r["free_mib"]) < FLOOR),
                  "compute_filtered_nonzero": sum(1 for r in mrows if int(r["compute_filtered"]) != 0),
                  "build_nonzero": sum(1 for r in mrows if int(r["build"]) != 0),
                  "clean_all": all(r["clean"].strip().lower() == "true" for r in mrows),
                  "streak_last": mrows[-1]["streak"], "first_ts": mrows[0]["ts"], "last_ts": mrows[-1]["ts"]}
ok("monitor = 60/60 clean rows", len(mrows) == 60 and res["monitor"]["clean_all"]
   and res["monitor"]["streak_last"] == "60")
ok("monitor util < 40 and every row free >= floor",
   res["monitor"]["util_max"] < 40 and res["monitor"]["below_floor"] == 0)
ok("monitor no compute/build processes", res["monitor"]["compute_filtered_nonzero"] == 0
   and res["monitor"]["build_nonzero"] == 0)
ok("S-2 exception not used (normal util<40 met)", res["monitor"]["util_max"] < 40)
post = rb(EVID + "/post-window-gpu.txt").decode("utf-8-sig").strip()
res["post_window_gpu"] = post
ok("post-window GPU state recorded", "MiB" in post and "%" in post, post)

# ---- 5. receipt vs diagnostic ----
print("\n[5] receipt vs capture.json diagnostic copy")
rec = json.loads(rb(EVID + "/receipt.json"))
diag = json.loads(rb(EVID + "/capture-diagnostic.json"))
dc = diag["delta_selfcheck"]
res["consistency"] = {
    "delta_match": rec["capture"]["delta_report"] == dc["report"],
    "overall": dc["overall"], "exit_code": rec["capture"]["exit_code"],
    "two_captures": diag["two_captures_bitwise_identical"],
    "spec_match": diag["spec"]["sha256"] == SPEC_SHA and diag["spec"]["bytes"] == SPEC_BYTES,
    "file_sha_match": rec["not_landed"]["retained_file_sha256_for_diagnosis"] == diag["files_sha256"],
    "frame0_vs_base_array_level": rec["not_landed"]["array_level_frame0_vs_base_comparison"],
}
ok("receipt delta report == capture.json.delta_selfcheck.report", res["consistency"]["delta_match"])
ok("overall non-discriminating and exit_code 3",
   dc["overall"] == "non-discriminating" and rec["capture"]["exit_code"] == 3)
ok("two-capture bitwise identity recorded", diag["two_captures_bitwise_identical"] is True)
ok("diagnostic spec block matches frozen spec", res["consistency"]["spec_match"])
ok("12 per-frame file sha256 recorded and receipt table matches",
   len(diag["files_sha256"]) == 12 and res["consistency"]["file_sha_match"], str(len(diag["files_sha256"])))
base_dir = "fixtures/render-bvh/scene-flex/reference/base"
base_shas = {n: sha(rb(base_dir + "/" + n + ".npz")) for n in ("rgb", "depth", "seg", "state")}
f000 = {k.split("/")[1].split(".")[0]: v for k, v in diag["files_sha256"].items() if k.startswith("frame-000/")}
res["frame0_vs_base_file_sha"] = {"equal": base_shas == f000, "base": base_shas}
ok("frame-000 file sha256 == existing base (diagnostic precursor)",
   base_shas == f000)
obs("array-level frame-000 vs base comparison NOT RUN (window stopped at self-check per spec section 8.5); "
    "recorded as such in receipt/README, to be run in a future spec window")

# ---- 6. not landed ----
print("\n[6] not landed")
status_lines = [l for l in sh("status", "--porcelain").stdout.decode().splitlines() if l.strip()]
own = (".normify/", "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-check/")
unrelated = [l for l in status_lines if not l[3:].lstrip().startswith(own)]
not_landed = {
    "multiframe_dir_exists": os.path.exists(os.path.join(REPO, "fixtures/render-bvh/scene-flex/multiframe")),
    "npz_tracked": [p for p in sh("ls-files", "fixtures").stdout.decode().splitlines() if "multiframe" in p],
    "git_status_clean_outside_review": unrelated == [],
    "unrelated_status_lines": unrelated,
}
tgt = os.path.join(REPO, "target", "r06-multiframe-capture")
not_landed["target_frames"] = []
if os.path.isdir(tgt):
    not_landed["target_frames"] = [f for f in os.listdir(tgt) if f.endswith(".npz") or "frame" in f]
res["not_landed"] = not_landed
ok("fixtures/.../multiframe does not exist", not not_landed["multiframe_dir_exists"])
ok("no multiframe artifact tracked in fixtures", not_landed["npz_tracked"] == [])
ok("working tree clean outside this review's own files", not_landed["git_status_clean_outside_review"],
   str(not_landed["unrelated_status_lines"]))
ok("no frame artifacts left in target scratch", not_landed["target_frames"] == [], str(not_landed["target_frames"]))

# ---- 7. evidence integrity / non-claims ----
print("\n[7] evidence manifest, encodings, non-claims")
man = rb(EVID + "/manifest.txt")
n = okc = 0
for line in man.decode().splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    data = rb(rel)
    okc += (sha(data) == h and str(len(data)) == b)
    n += 1
ok("evidence manifest self-check 9/9", n == 9 and okc == 9, "%d/%d" % (okc, n))
enc_bad = []
for name in sorted(os.listdir(os.path.join(REPO, EVID))):
    p = EVID + "/" + name
    b = rb(p)
    try:
        b.decode("utf-8")
        if b[:3] == b"\xef\xbb\xbf" or b"\r" in b:
            enc_bad.append(name)
    except UnicodeDecodeError:
        enc_bad.append(name)
res["encoding_deviations"] = enc_bad
ok("README/receipt/manifest/csv/json are UTF-8/LF/no BOM",
   all(n not in enc_bad for n in ("README.md", "receipt.json", "manifest.txt", "monitor-capture.csv",
                                  "capture-diagnostic.json", "cam-poses-diagnostic.json")))
obs("evidence encoding deviations vs card acceptance 'UTF-8/LF/no BOM': " + str(enc_bad),
    "capture.log/dry-run.log are PowerShell ANSI(cp936)+CRLF console captures; "
    "baseline-three-points.txt and post-window-gpu.txt carry UTF-8 BOM+CRLF")
texts = decode(rb(EVID + "/README.md"))[0] + decode(rb(EVID + "/receipt.json"))[0]
for term in ("多帧验收通过", "上游等价", "超窗写"):
    ok("forbidden claim absent: " + term, term not in texts)
ok("non-claims present (not multi-frame acceptance material / no coverage claim)",
   "不声明多帧已覆盖" in texts and "不构成" in texts and "不作对外发布依据" in texts)

# ---- 8. lock-timing record and card ----
print("\n[8] lock timing record and card")
readme = decode(rb(EVID + "/README.md"))[0]
ok("README section 10 records early acquire 11:59:25, corrective 12:00:07, no pre-window content write",
   "11:59:25" in readme and "12:00:07" in readme and "未做任何仓库内容写" in readme)
card = json.loads(blob("HEAD", ".normify/changes/" + CARD + ".json"))
res["card"] = {"status": card["status"], "addenda": [x.get("reason", "") for x in (card.get("addenda") or [])],
               "created_at": card["created_at"], "closed_at": card["closed_at"],
               "checks": len(card["checks"])}
ok("capture card verified with lock-timing addendum",
   card["status"] == "verified" and len(card.get("addenda") or []) == 1
   and "11:59:25" in (card["addenda"][0].get("reason", "")), str(res["card"]))
obs("window card created_at 12:01 is after baseline 12:00:26 / monitor 12:00:44; the card and all window "
    "evidence were committed together at 12:07:59 (commit 484852ca, after capture ended 12:06:44). "
    "Registration-before-execution ordering is a bookkeeping deviation; data/stop/not-landed conclusions are unaffected",
    "commit times: 484852ca 12:07:59, dbe8d2b7 12:08:07, 27474ac5 12:08:44")

res["fails"] = fails
if a.out:
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        json.dump(res, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print("wrote", a.out)
print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
