# -*- coding: utf-8 -*-
"""QA read-only verification: R01@P8 closeout (e492c979). Usage: [--out <log>]"""
import hashlib
import io
import json
import os
import re
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
C = "e492c979"
SUM = "docs/development-evidence/render-bvh/r01-p8-summary"
PLAN = "docs/development-evidence/render-bvh-precheck/plan.json"
PRE = "docs/development-evidence/render-bvh-precheck/README.md"

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
print("  HEAD:", git("rev-parse", "HEAD").strip().decode()[:12])

print("\n== 1. commit scope ==")
files = [f for f in git("show", "--name-only", "--format=", C).decode().splitlines() if f.strip()]
print("  files:", len(files))
for f in files:
    print("   ", f)
out = [f for f in files if not (f == PLAN or f == PRE or f.startswith(SUM + "/") or f.startswith(".normify/"))]
print("  out-of-scope files:", out if out else "none")
p = subprocess.run(["git", "diff", "--name-only", C + "^.." + C, "--", "src/", "tests/", "fixtures/"], cwd=REPO, capture_output=True)
print("  src/tests/fixtures diff empty:", p.stdout.strip() == b"")

print("\n== 2. summary evidence manifest/receipt ==")
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
rd = blob(C, SUM + "/README.md")
text = rd.decode("utf-8")
print("  README bytes=%d sha256=%s" % (len(rd), sha(rd)[:16]))
print("  receipt keys:", list(rc.keys()))

print("\n== 3. ten-item mapping consistency (plan vs README/receipt) ==")
pj = json.loads(blob(C, PLAN).decode("utf-8"))
rp = pj.get("r01_p8_mapping")
print("  r01_p8_mapping type:", type(rp).__name__)
if isinstance(rp, dict):
    items = rp.get("items", [])
else:
    items = rp or []
print("  mapping items:", len(items))
for it in items:
    print("   %-46s status=%-24s evidence=%s" % (it.get("item"), it.get("status"), it.get("evidence") or it.get("delivered")))
req_ten = ["worlds", "cameras", "assets", "bvh", "all_default_parameters", "lifecycle",
           "rgb_output", "depth_output", "segmentation_output"]
present = [i.get("item") for i in items]
print("  required items present:", [x for x in req_ten if x not in present] or "all")
print("  extra/supplementary:", [x for x in present if x not in req_ten])
print("  any planned/pending status:", [i.get("item") for i in items if str(i.get("status", "")).startswith("planned")] or "none")
print("  delivered-except-r07 item status:", [i.get("status") for i in items if "shadows" in str(i.get("item", ""))])
print("  R07 mentioned as out of scope:", "R07" in json.dumps(rp, ensure_ascii=False))

print("\n== 4. delivery row + N15 + updated_at ==")
dt = pj.get("delivery_tracking", {})
rows = [r for r in dt.get("rows", []) if str(r.get("slice")) == "R01-P8"]
print("  R01-P8 rows:", len(rows))
for r in rows:
    print("   task_id=%s normify_status=%s owner=%s" % (r.get("task_id"), r.get("normify_status"), r.get("owner")))
    print("   evidence=%s execution=%s" % (r.get("evidence"), r.get("execution")))
    print("   note:", str(r.get("note"))[:180])
print("  updated_at:", dt.get("updated_at"))
n15 = [x for x in dt.get("scheduling_notes", []) if isinstance(x, dict) and x.get("id") == "N15"]
print("  N15 present:", bool(n15))
if n15:
    note = n15[0].get("note", "")
    for kw in ("R01", "NP01", "NP02", "NP03", "NP04", "release", "R07", "Linux", "non-claim"):
        print("   N15 mentions %-10s %s" % (kw, kw.lower() in note.lower()))

print("\n== 5. precheck README addendum only ==")
d = git("diff", C + "^.." + C, "--", PRE).decode("utf-8", "replace").splitlines()
rem = [l for l in d if l.startswith("-") and not l.startswith("---")]
add = [l for l in d if l.startswith("+") and not l.startswith("+++")]
print("  addendum: added=%d removed=%d" % (len(add), len(rem)))
print("  removed lines:", rem[:5])
print("  addendum mentions 2026-11-04:", any("2026-11-04" in l for l in add))
print("  addendum says original conclusion unchanged:", any(("结论" in l and ("未改" in l or "不变" in l)) or "unchanged" in l.lower() for l in add))

print("\n== 6. plan.json diff locality ==")
pd = git("diff", C + "^.." + C, "--", PLAN).decode("utf-8", "replace").splitlines()
rem = [l for l in pd if l.startswith("-") and not l.startswith("---")]
add = [l for l in pd if l.startswith("+") and not l.startswith("+++")]
print("  plan diff: added=%d removed=%d" % (len(add), len(rem)))
print("  removed sample:", rem[:4])
print("  N15 added:", any("N15" in l for l in add))
print("  R01-P8 row added:", any("R01-P8" in l for l in add))
print("  updated_at change:", any("updated_at" in l for l in add + rem))
print("  R06 slices untouched by this commit:", not any("R06-S" in l for l in add + rem))

print("\n== 7. evidence pointers spot check ==")
refs = {
    "S6a": "docs/development-evidence/render-bvh/s6a-context-params/",
    "S7b1": "docs/development-evidence/render-bvh/s7b1-depth-seg/",
    "perf": "docs/development-evidence/render-bvh/perf/",
    "R10-S2": "docs/development-evidence/render-bvh/r10-s2-multi-world/",
    "R08-S1": "docs/development-evidence/render-bvh/r08-camera/",
    "r02-summary": "docs/development-evidence/render-bvh/r02-skybox-summary/",
    "r06-summary": "docs/development-evidence/render-bvh/r06-flex-summary/",
    "np01-e-recompute": "docs/development-evidence/np01-e-recompute/",
    "np02-new-kernel": "docs/development-evidence/np02-evidence/",
}
missing = 0
for k, dpath in refs.items():
    ok = os.path.isdir(os.path.join(REPO, *dpath.split("/")))
    if not ok:
        missing += 1
    print("  %-18s %s" % (k, "OK" if ok else "MISSING"))
print("  missing evidence dirs:", missing)
# reference dirs actually present under render-bvh
rdirs = sorted(os.listdir(os.path.join(REPO, "docs", "development-evidence", "render-bvh")))
print("  render-bvh evidence dir count:", len(rdirs))

print("\n== 8. NP check consistency ==")
npchk = rc.get("np_check", {})
for k in ("NP01", "NP02", "NP03", "NP04"):
    v = npchk.get(k, {})
    print("  %s verdict=%s" % (k, v.get("verdict")))
    if k == "NP01":
        print("    open_item:", v.get("open_item"))
        print("    forbidden:", v.get("forbidden_claims"))
    if k == "NP02":
        print("    evidence count:", len(v.get("evidence", [])))
print("  README NP01 open item wording:", "待用户签" in text)
print("  README NP03 Linux terminated:", "2026-10-24" in text and "Linux" in text)

print("\n== 9. known limits + non-claims ==")
for kw in ("release 全量构建未跑", "R07", "更高 spp", "Linux/T4", "与上游完全一致", "R06 单帧"):
    print("  limits contains %-18s %s" % (kw, kw in text))
for kw in ("不声明完整 R01", "不声明双平台", "不声明与上游完全一致", "不声明 release 全量", "对外发布依据"):
    print("  non-claim contains %-22s %s" % (kw, kw in text))
print("  receipt non_claims:", len(rc.get("non_claims", [])))
print("  receipt known_limits:", len(rc.get("known_limits", [])))

print("\n== 10. QA review card presence ==")
p = os.path.join(REPO, ".normify", "changes", "2026-11-04-render-r01-p8-closeout-check.json")
if os.path.exists(p):
    d = json.load(io.open(p, encoding="utf-8"))
    print("  card:", d["id"], "|", d["status"], "| checks:", [c.get("gate") or c.get("kind") for c in d.get("checks", [])])
print("  B closeout card status:", card_status("2026-11-04-render-r01-p8-closeout"))
