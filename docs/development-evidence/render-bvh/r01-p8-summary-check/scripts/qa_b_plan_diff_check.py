# -*- coding: utf-8 -*-
import json
import os
import subprocess

REPO = r"D:\Rust\mjwarp-rs"
C = "e492c979"
PLAN = "docs/development-evidence/render-bvh-precheck/plan.json"


def blob(rev, path):
    return subprocess.run(["git", "cat-file", "-p", rev + ":" + path], cwd=REPO, capture_output=True).stdout


cur = json.loads(blob(C, PLAN).decode("utf-8"))
par = json.loads(blob(C + "^", PLAN).decode("utf-8"))

import sys

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

print("== top-level keys changed ==")
for k in sorted(set(list(cur.keys()) + list(par.keys()))):
    a = json.dumps(par.get(k), sort_keys=True, ensure_ascii=False)
    b = json.dumps(cur.get(k), sort_keys=True, ensure_ascii=False)
    if a != b:
        print("  CHANGED:", k, "| parent len", len(a), "-> now", len(b))
    else:
        print("  same:   ", k)

print("\n== r01_p8_mapping items ==")
pm, cm = par.get("r01_p8_mapping"), cur.get("r01_p8_mapping")
print("  parent items:", len(pm), "current items:", len(cm))
for a, b in zip(pm, cm):
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("  refreshed:", a.get("item"), "| parent status:", a.get("status"), "-> now:", b.get("status"))
        print("    parent slices:", a.get("slices"))
        print("    now slices:   ", b.get("slices"))

print("\n== delivery_tracking rows diff ==")
pr, cr = par["delivery_tracking"]["rows"], cur["delivery_tracking"]["rows"]
print("  rows:", len(pr), "->", len(cr))
added = [r for r in cr if json.dumps(r, sort_keys=True) not in [json.dumps(x, sort_keys=True) for x in pr]]
removed = [r for r in pr if json.dumps(r, sort_keys=True) not in [json.dumps(x, sort_keys=True) for x in cr]]
for r in added:
    print("  +row:", r.get("slice"), r.get("task_id"), r.get("normify_status"))
for r in removed:
    print("  -row:", r.get("slice"), r.get("task_id"))

print("\n== scheduling_notes diff ==")
pn = [x.get("id") for x in par["delivery_tracking"].get("scheduling_notes", []) if isinstance(x, dict)]
cn = [x.get("id") for x in cur["delivery_tracking"].get("scheduling_notes", []) if isinstance(x, dict)]
print("  parent notes:", pn)
print("  current notes:", cn)
print("  updated_at:", par["delivery_tracking"].get("updated_at"), "->", cur["delivery_tracking"].get("updated_at"))

print("\n== R06 slice definitions unchanged by B commit ==")
prs = [s for s in par["slices"] if str(s.get("id", "")).startswith("R06")]
crs = [s for s in cur["slices"] if str(s.get("id", "")).startswith("R06")]
print("  R06 slices count:", len(prs), "->", len(crs))
print("  identical:", json.dumps(prs, sort_keys=True) == json.dumps(crs, sort_keys=True))

print("\n== mapping note pointers resolve (sample task ids) ==")
import re
ids = set()
for it in cm:
    for m in re.findall(r"2026-\d\d-\d\d-(?:render|precision)-[a-z0-9\-]+", json.dumps(it, ensure_ascii=False)):
        ids.add(m)
print("  pointer ids found:", len(ids))
bad = 0
for i in sorted(ids):
    p = os.path.join(REPO, ".normify", "changes", i + ".json")
    if not os.path.exists(p):
        bad += 1
        print("  MISSING card:", i)
print("  missing:", bad)

print("\n== family evidence dirs for r08/r10 ==")
dirs = sorted(d for d in os.listdir(os.path.join(REPO, "docs", "development-evidence", "render-bvh")) if d.startswith(("r08", "r10")))
print("  ", dirs)
