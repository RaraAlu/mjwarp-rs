# -*- coding: utf-8 -*-
"""QA read-only verification: R06 flex family closeout (1de0321a) + stats-print (a0118271).

Usage: python qa_flex_closeout_check.py [--out <log>]
"""
import hashlib
import io
import json
import os
import re
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
C = "1de0321a"
S = "a0118271"
SUM = "docs/development-evidence/render-bvh/r06-flex-summary"
PLAN = "docs/development-evidence/render-bvh-precheck/plan.json"
ATT6 = "docs/development-evidence/render-bvh/r06-s3b-flex-attempt6"

if "--out" in sys.argv:
    _fh = open(sys.argv[sys.argv.index("--out") + 1], "w", encoding="utf-8", newline="\n")
    _orig = sys.stdout

    class _Tee(object):
        def write(self, s):
            _fh.write(s)
            _orig.write(s)
            return len(s)

        def flush(self):
            _fh.flush()
            _orig.flush()

    sys.stdout = _Tee()


def git(*a):
    p = subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s %s" % (" ".join(a), p.stderr.decode("utf-8", "replace")))
    return p.stdout


def blob(rev, path):
    return git("cat-file", "-p", rev + ":" + path)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def commit_ok(rev):
    p = subprocess.run(["git", "cat-file", "-t", rev + "^{commit}"], cwd=REPO, capture_output=True)
    return p.returncode == 0 and p.stdout.strip() == b"commit"


def card_status(task_id):
    p = os.path.join(REPO, ".normify", "changes", task_id + ".json")
    if not os.path.exists(p):
        return "missing"
    try:
        return json.load(io.open(p, encoding="utf-8"))["status"]
    except Exception:
        return "unreadable"


print("== 0. objects ==")
print("  closeout commit:", git("rev-parse", C).strip().decode()[:12])
print("  stats-print commit:", git("rev-parse", S).strip().decode()[:12])
print("  HEAD:", git("rev-parse", "HEAD").strip().decode()[:12])

print("\n== 1. closeout commit scope ==")
files = [f for f in git("show", "--name-only", "--format=", C).decode().splitlines() if f.strip()]
print("  files:", len(files))
for f in files:
    print("   ", f)
out = [f for f in files if not (f == PLAN or f.startswith(SUM + "/") or f.startswith(".normify/"))]
print("  out-of-scope files:", out if out else "none")
p = subprocess.run(["git", "diff", "--name-only", C + "^.." + C, "--", "src/", "tests/", "fixtures/"],
                   cwd=REPO, capture_output=True)
print("  src/tests/fixtures diff empty:", p.stdout.strip() == b"")

print("\n== 2. summary evidence manifest + receipt ==")
man = blob(C, SUM + "/manifest.txt")
print("  manifest bytes=%d first=%r" % (len(man), chr(man[0])))
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob(C, rel if rel.startswith("docs/") else SUM + "/" + rel)
    if sha(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  manifest entries=%d bad=%d" % (len(entries), bad))
rc = json.loads(blob(C, SUM + "/receipt.json").decode("utf-8"))
print("  receipt keys:", list(rc.keys()))
print("  receipt bytes=%d" % len(blob(C, SUM + "/receipt.json")))

print("\n== 3. wording: single-frame / multi-frame + non-claims + allowlist + D8 ==")
rd = blob(C, SUM + "/README.md")
text = rd.decode("utf-8")
print("  README bytes=%d sha256=%s" % (len(rd), sha(rd)[:16]))
checks = {
    "single_frame_ok": "单帧位级成立" in text,
    "multiframe_not_covered": "多帧演化未覆盖" in text,
    "no_animation_claim": ("不声明「动画与几何一致性」" in text) or ("不声明动画与几何一致性" in text),
    "no_upstream_equivalence": "上游完全等价" in text,
    "no_parity": "parity 通过" in text,
    "no_external_release": "对外发布依据" in text,
    "allowlist_cam2": "cam2" in text,
    "allowlist_pixel": "(x=134,y=128)" in text or "(x=134, y=128)" in text,
    "allowlist_flexid0": "flexid0" in text,
    "allowlist_ulp": "ULP ≤ 4" in text,
    "allowlist_beyond4_zero": "beyond4 = 0" in text,
    "allowlist_beyond4_fail": "beyond4 ≠ 0" in text,
    "allowlist_no_expand": ("范围不扩大" in text) or ("不扩大" in text),
    "d8_321": "321" in text,
    "d8_beyond4_297": "297" in text,
    "d8_maxulp_259": "259" in text,
    "d8_separate": "不混算" in text,
    "multiframe_tracker": "2026-11-04-render-r06-flex-multiframe" in text,
}
for k, v in checks.items():
    print("  %-30s %s" % (k, v))
sig = blob(C, "docs/development-evidence/render-bvh/r06-s3b-flex-allowlist/README.md").decode("utf-8")
print("  allowlist scope matches signature card wording:", all(
    s in sig for s in ("cam2", "(x=134, y=128)", "flexid0", "ULP ≤ 4", "beyond4 = 0", "beyond4 不为 0")))

print("\n== 4. plan.json accounting ==")
pj = json.loads(blob(C, PLAN).decode("utf-8"))
r06slices = [s for s in pj["slices"] if str(s.get("id", "")).startswith("R06")]
print("  R06 slices:", len(r06slices))
badc = 0
for s in r06slices:
    commits = s.get("delivered_commits", [])
    ok = s.get("status") == "delivered" and bool(commits) and all(commit_ok(x) for x in commits)
    badc += 0 if ok else 1
    print("   %-9s status=%-9s commits=%d ok=%s" % (s.get("id"), s.get("status"), len(commits), ok))
print("  slices bad=%d" % badc)
dt = pj["delivery_tracking"]
rows = [r for r in dt["rows"] if str(r.get("slice", "")).startswith("R06")]
print("  R06 rows:", len(rows), "| updated_at:", dt.get("updated_at"))
badr = 0
for r in rows:
    st = card_status(r.get("task_id", ""))
    ok = st == "verified" and r.get("normify_status") == "verified"
    badr += 0 if ok else 1
    print("   %-14s %-52s file_status=%-9s row_status=%s" % (r.get("slice"), r.get("task_id"), st, r.get("normify_status")))
print("  rows bad=%d" % badr)
n14 = [x for x in dt.get("scheduling_notes", []) if isinstance(x, dict) and x.get("id") == "N14"]
print("  N14 present:", bool(n14))
if n14:
    note = n14[0].get("note", "")
    for kw in ("valid_round", "VALID_ROUND", "allowlist", "D8", "multiframe", "非声明", "non_claim"):
        print("   N14 mentions %-12s %s" % (kw, kw.lower() in note.lower()))
print("  roadmap-r06-p8 present:", "roadmap-r06-p8" in json.dumps(pj))
for s in pj["slices"]:
    if s.get("id") == "roadmap-r06-p8":
        print("   roadmap-r06-p8 status:", s.get("status"))
mut = [s for s in pj["slices"] if s.get("id") == "roadmap-r01-p8"]
if mut:
    print("   roadmap-r01-p8 status:", mut[0].get("status"))
print("  non_claims plan count:", len(pj.get("non_claims", [])))

print("\n== 5. plan.json diff is additive for R06 accounting ==")
d = git("diff", C + "^.." + C, "--", PLAN).decode("utf-8", "replace").splitlines()
rem = [l for l in d if l.startswith("-") and not l.startswith("---")]
add = [l for l in d if l.startswith("+") and not l.startswith("+++")]
print("  added lines=%d removed lines=%d" % (len(add), len(rem)))
print("  removed lines content:", rem[:5])

print("\n== 6. closeout README window numbers vs attempt6 evidence ==")
g = json.loads(blob(C, ATT6 + "/guards-after.json").decode("utf-8"))
print("  guards counts:", json.dumps(g.get("counts")), "| attempt:", g.get("attempt"), "| status:", g.get("status"))
print("  aggregate_all:", g.get("aggregate_sha256"))
print("  aggregate_inputs:", g.get("aggregate_inputs_sha256"))
for e in g.get("frozen_exes", []):
    print("   exe %-52s %-10s %s" % (e.get("path"), e.get("bytes"), (e.get("sha256") or "")[:16]))
tr = blob(C, ATT6 + "/two-run-sha.txt").decode("utf-8", "replace")
print("  two-run hashes in evidence:", all(x in tr for x in ("d84ecaed", "98292153", "c4be9853")))
el = blob(C, ATT6 + "/entity-run.log")
etxt = el.decode("utf-16") if el[:2] == b"\xff\xfe" else el.decode("utf-8", "replace")
print("  entity: running 1 test:", "running 1 test" in etxt, "| passed line:",
      [l.strip() for l in etxt.splitlines() if "test result:" in l])

print("\n== 7. stats-print commit scope (test-only) ==")
sfiles = [f for f in git("show", "--name-only", "--format=", S).decode().splitlines() if f.strip()]
print("  files:", sfiles)
sd = git("diff", S + "^.." + S, "--", "tests/").decode("utf-8", "replace").splitlines()
srem = [l for l in sd if l.startswith("-") and not l.startswith("---")]
sadd = [l for l in sd if l.startswith("+") and not l.startswith("+++")]
print("  tests diff: added=%d removed=%d" % (len(sadd), len(srem)))
print("  removed lines:", srem)
p = subprocess.run(["git", "diff", "--name-only", S + "^.." + S, "--", "src/", "fixtures/"], cwd=REPO, capture_output=True)
print("  src/fixtures diff empty:", p.stdout.strip() == b"")
print("  added println/format lines:", len([l for l in sadd if "println!" in l or "format!" in l]))
print("  added assert lines (new test only):", len([l for l in sadd if "assert" in l]))
print("  MAX_DEPTH_ULP mentions in additions:", len([l for l in sadd if "MAX_DEPTH_ULP" in l]))
print("  allowlist constants changed:", any(("ALLOWLIST_PIXEL_X: usize = 134" in l and l.startswith("-")) for l in sd))
print("  assert_depth_allowlisted body changed:", any(("assert_depth_allowlisted" in l and l.startswith("-")) for l in sd))

print("\n== 8. A QA review card presence ==")
cards = [f for f in os.listdir(os.path.join(REPO, ".normify", "changes")) if "flex-closeout" in f or "closeout-check" in f]
print("  closeout-related cards:", cards)
for c in cards:
    try:
        j = json.load(io.open(os.path.join(REPO, ".normify", "changes", c), encoding="utf-8"))
        print("   %s | %s | owner %s" % (j["id"], j["status"], j["schedule"]["owner"]))
    except Exception:
        pass

print("\n== 9. receipt fields (no-side-effects declaration) ==")
rc2 = json.loads(blob(C, SUM + "/receipt.json").decode("utf-8"))
for k in ("gpu_used", "product_code_changed", "criteria_changed", "reference_changed",
          "multi_frame_evolution_covered", "status", "date"):
    print("  %-32s %r" % (k, rc2.get(k)))
print("  multi_frame_tracking_card:", rc2.get("multi_frame_tracking_card"))
print("  plan_json_changes:", json.dumps(rc2.get("plan_json_changes"), ensure_ascii=False)[:300])
print("  reviews:", json.dumps(rc2.get("reviews"), ensure_ascii=False)[:300])
print("  non_claims count:", len(rc2.get("non_claims", [])))

print("\n== 10. slice implementation cards / commits / evidence dirs ==")
readme = text
slice_cards = {
    "R06-S0": ("2026-11-03-render-r06-flex-precheck", ["7e8ec8c8"], "r06-flex-precheck"),
    "R06-S1a": ("2026-11-03-render-r06-s1a-megakernel-args-split", ["0ac566a5"], None),
    "R06-S2": ("2026-11-03-render-r06-s2-flex-host-channel", ["7c0bb8f9"], "r06-s2-flex-host-channel"),
    "R06-S2b": ("2026-11-04-render-r06-s2b-flex-datasource-t4-merged", ["1f199737", "88e19c44"], "r06-flex-datasource"),
    "R06-S2c": ("2026-11-04-render-r06-s2c-flex-shell-data", ["df6d74af", "b4a7e827"], "r06-s2c-flex-shell"),
    "R06-S3pre": ("2026-11-04-render-r06-s3pre-flexhost-wiring", ["0b82c676"], "r06-s3pre-flexhost-wiring"),
    "R06-S3a": ("2026-11-04-render-r06-s3a-flex-device", ["544d02b9", "735dcc6f"], "r06-s3a-flex-device"),
    "R06-S3b": ("2026-11-04-render-r06-s3b-closure", ["71ec7c83", "d35cab68", "3406f1ca", "7a48205d", "db082561", "8aec9619"], "r06-s3b-closure"),
    "R06-S3c": ("2026-11-04-render-r06-s3c-flex-mesh-device", ["181eaa7a", "e9e198cd", "ddd06292", "5d52ebeb"], "r06-s3c-flex-mesh-device"),
}
bad = 0
for sl, (card, commits, evdir) in slice_cards.items():
    st = card_status(card)
    cok = all(commit_ok(x) for x in commits)
    d = os.path.join(REPO, "docs", "development-evidence", "render-bvh", evdir) if evdir else None
    dok = (os.path.isdir(d) if d else True)
    ok = st == "verified" and cok and dok
    bad += 0 if ok else 1
    print("  %-9s card=%s commits_ok=%s evidence_dir=%s" % (sl, st, cok, (evdir and (dok or "MISSING")) or "n/a"))
print("  slice checks bad=%d" % bad)
print("  README rows mention all 9 slice ids:", all(x in readme for x in slice_cards))

print("\n== 11. stats-print spot check evidence ==")
ST = "docs/development-evidence/render-bvh/r06-s3b-flex-attempt6-stats-print"
sm = blob(S, ST + "/manifest.txt")
se = [l for l in sm.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
sbad = 0
for line in se:
    x, size, rel = line.split()
    data = blob(S, rel if rel.startswith("docs/") else ST + "/" + rel)
    if sha(data) != x or len(data) != int(size):
        sbad += 1
print("  stats evidence manifest entries=%d bad=%d" % (len(se), sbad))
sc = json.loads(blob(S, ".normify/changes/2026-11-04-render-r06-s3b-flex-attempt6-stats-print.json").decode("utf-8"))
print("  stats card status in commit:", sc["status"])
su = json.loads(blob(S, ".normify/runs/2026-11-04-render-r06-s3b-flex-attempt6-stats-print/unit.json").decode("utf-8"))
print("  stats unit gate tests:", su.get("tests"), "passed:", su.get("passed"))
sreadme = blob(S, ST + "/README.md").decode("utf-8")
print("  README claims 19 tests/18 passed/1 ignored:", ("19 tests" in sreadme) and ("18 passed" in sreadme))
print("  test-only claim:", "只读打印" in sreadme or "只读" in sreadme)

