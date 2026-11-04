# -*- coding: utf-8 -*-
"""Generic QA read-only verification for the C-series family closeouts.

Usage: python qa_c_check.py <C1|C2|C3> [--out <log>]
"""
import hashlib
import io
import json
import os
import re
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
PLAN = "docs/development-evidence/render-bvh-precheck/plan.json"
PRE = "docs/development-evidence/render-bvh-precheck/README.md"

CFG = {
    "C1": {
        "commit": "948eddc6",
        "card": "2026-11-04-render-r03-r04-output-closeout",
        "ev": "docs/development-evidence/render-bvh/r03-r04-summary",
        "rows": ["R03", "R04"],
        "note": "N16",
        "keywords": ["120,000/120,000", "0 ULP", "精确相等", "不声明", "已知限制", "AA", "spp", "nworld", "对外发布依据", "不替其他家族"],
    },
    "C2": {
        "commit": "77c7ce81",
        "card": "2026-11-04-render-r02-rgb-closeout",
        "ev": "docs/development-evidence/render-bvh/r02-rgb-summary",
        "rows": ["R02", "R02-skybox"],
        "note": "N17",
        "keywords": ["120,000/120,000", "beyond_1_lsb=0", "不声明", "已知限制", "spp", "Linux/T4", "对外发布依据", "不替其他家族", "validation-only", "INVALID_ROUND"],
    },
    "C3": {
        "commit": "0f49dba9",
        "card": "2026-11-04-render-r05-r09-r10-family-closeout",
        "ev": "docs/development-evidence/render-bvh/r05-r09-r10-summary",
        "rows": ["R05", "R09", "R10"],
        "note": "N18",
        "keywords": ["不声明", "已知限制", "authoring-only", "validation-only", "INVALID_ROUND", "spp", "Linux/T4", "对外发布依据", "不替其他家族", "nworld"],
    },
}

which = sys.argv[1]
cfg = CFG[which]
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

C, CARD, EV = cfg["commit"], cfg["card"], cfg["ev"]


def git(*a):
    p = subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s %s" % (" ".join(a), p.stderr.decode("utf-8", "replace")))
    return p.stdout


def blob(rev, path):
    return git("cat-file", "-p", rev + ":" + path)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def card_status(task_id):
    p = os.path.join(REPO, ".normify", "changes", task_id + ".json")
    if not os.path.exists(p):
        return "missing"
    try:
        return json.load(io.open(p, encoding="utf-8"))["status"]
    except Exception:
        return "unreadable"


print("== 0. objects (%s) ==" % which)
print("  commit:", git("rev-parse", C).strip().decode()[:12])
print("  card:", CARD, "| status:", card_status(CARD))
print("  evidence dir:", EV)

print("\n== 1. commit scope ==")
files = [f for f in git("show", "--name-only", "--format=", C).decode().splitlines() if f.strip()]
print("  files:", len(files))
for f in files:
    print("   ", f)
out = [f for f in files if not (f == PLAN or f.startswith(EV + "/") or f.startswith(".normify/"))]
print("  out-of-scope files:", out if out else "none")
p = subprocess.run(["git", "diff", "--name-only", C + "^.." + C, "--", "src/", "tests/", "fixtures/"], cwd=REPO, capture_output=True)
print("  src/tests/fixtures diff empty:", p.stdout.strip() == b"")
p = subprocess.run(["git", "diff", "--name-only", C + "^.." + C, "--", PRE], cwd=REPO, capture_output=True)
print("  precheck README untouched by this commit:", p.stdout.strip() == b"")

print("\n== 2. evidence manifest/receipt ==")
man = blob(C, EV + "/manifest.txt")
print("  manifest bytes=%d first=%r" % (len(man), chr(man[0])))
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob(C, rel if rel.startswith("docs/") else EV + "/" + rel)
    if sha(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  manifest entries=%d bad=%d" % (len(entries), bad))
rc = json.loads(blob(C, EV + "/receipt.json").decode("utf-8"))
rd = blob(C, EV + "/README.md")
text = rd.decode("utf-8")
print("  README bytes=%d sha256=%s" % (len(rd), sha(rd)[:16]))
print("  receipt keys:", list(rc.keys()))
for k in ("gpu_used", "product_code_changed", "criteria_changed", "reference_changed", "status", "task_id"):
    print("  receipt.%-24s %r" % (k, rc.get(k)))

print("\n== 3. evidence pointers -> verified cards ==")
ids = set(re.findall(r"2026-\d{2}-\d{2}-(?:render|precision)[a-z0-9\-]*", json.dumps(rc, ensure_ascii=False) + text))
print("  task-id pointers:", len(ids))
missing = [i for i in ids if card_status(i) == "missing"]
notver = [i for i in ids if card_status(i) not in ("verified", "missing")]
print("  missing cards:", len(missing), missing[:5])
print("  non-verified cards:", len(notver), notver[:5])
dirmiss = 0
for m in re.findall(r"……/([a-z0-9\-]+)/", text):
    pth = os.path.join(REPO, "docs", "development-evidence", "render-bvh", m)
    if not os.path.isdir(pth):
        dirmiss += 1
        print("  MISSING evidence dir (abbrev):", m)
print("  abbreviated evidence dirs missing:", dirmiss)

print("\n== 4. keyword coverage in README ==")
for kw in cfg["keywords"]:
    print("  %-20s %s" % (kw, kw in text))

print("\n== 5. plan.json bookkeeping ==")
cur = json.loads(blob(C, PLAN).decode("utf-8"))
par = json.loads(blob(C + "^", PLAN).decode("utf-8"))
changed = [k for k in set(list(cur.keys()) + list(par.keys()))
           if json.dumps(par.get(k), sort_keys=True) != json.dumps(cur.get(k), sort_keys=True)]
print("  changed top-level keys:", changed)
pr = par["delivery_tracking"]["rows"]
cr = cur["delivery_tracking"]["rows"]
added = [r for r in cr if json.dumps(r, sort_keys=True) not in [json.dumps(x, sort_keys=True) for x in pr]]
print("  rows added:", len(added))
for r in added:
    print("   +row slice=%-12s task=%-46s status=%-9s evidence=%s"
          % (r.get("slice"), r.get("task_id"), r.get("normify_status"), r.get("evidence")))
    print("     card_file_status:", card_status(r.get("task_id", "")))
print("  expected row slices present:", all(any(r.get("slice") == s for r in added) for s in cfg["rows"]))
pn = [x.get("id") for x in par["delivery_tracking"].get("scheduling_notes", []) if isinstance(x, dict)]
cn = [x.get("id") for x in cur["delivery_tracking"].get("scheduling_notes", []) if isinstance(x, dict)]
print("  notes added:", [x for x in cn if x not in pn])
print("  expected note %s added: %s" % (cfg["note"], cfg["note"] in cn and cfg["note"] not in pn))
print("  updated_at:", par["delivery_tracking"].get("updated_at"), "->", cur["delivery_tracking"].get("updated_at"))
print("  r01_p8_mapping untouched:", json.dumps(par.get("r01_p8_mapping"), sort_keys=True) == json.dumps(cur.get("r01_p8_mapping"), sort_keys=True))
prs = [s for s in par["slices"] if str(s.get("id", "")).startswith(("R01", "R06"))]
crs = [s for s in cur["slices"] if str(s.get("id", "")).startswith(("R01", "R06"))]
print("  R01/R06 slice definitions untouched:", json.dumps(prs, sort_keys=True) == json.dumps(crs, sort_keys=True))

print("\n== 6. roadmap nodes not started by this card ==")
for nid in ("roadmap-r03-p8", "roadmap-r04-p8", "roadmap-r02-p8", "roadmap-r05-p8", "roadmap-r09-p8", "roadmap-r10-p8"):
    fp = os.path.join(REPO, ".normify", "changes", nid + ".json")
    if os.path.exists(fp):
        st = json.load(io.open(fp, encoding="utf-8"))["status"]
        print("  %-18s %s" % (nid, st))
