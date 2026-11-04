#!/usr/bin/env python3
"""G10@P5 family closeout (commit e5cdc542 + NP01 wording fix afd5e6cc) QA check.

Read-only. Reproduces the numbers quoted in the QA review README:
  1. summary manifest self-check (sha256 + bytes, header, EOL/BOM)
  2. map.json requirement accounting vs receipt + README table
  3. evidence/card references resolve to verified cards
  4. the "54 subcards" claim: no written index; candidate derivations
  5. device/host evidence boundary (paths exist, numbers traceable)
  6. zero src/tests/fixtures/acceptance-map diff; fix touches NP01 wording only
Run: python g10_closeout_check.py   (from repo root)
"""
import glob
import hashlib
import json
import os
import re
import subprocess

REPO = "D:/Rust/mjwarp-rs"
SUMMARY = "docs/development-evidence/g10-p5-summary"
MAP = "docs/development-evidence/g10-p5-acceptance-map/map.json"
BASE = "65fc778a"
CLOSEOUT = "e5cdc542"
FIX = "afd5e6cc"
fails = []


def ok(label, cond, detail=""):
    print(("PASS " if cond else "FAIL ") + label + (" :: " + detail if detail else ""))
    if not cond:
        fails.append(label)


def git(*args):
    return subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True,
                          text=True, encoding="utf-8").stdout


def sha256(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def card_statuses(rev):
    out = {}
    names = git("ls-tree", "-r", "--name-only", rev, ".normify/changes/").splitlines()
    for p in names:
        if "g10-p5" not in p:
            continue
        t = git("show", rev + ":" + p)
        m = re.search(r'"status":\s*"(\w+)"', t)
        out[p] = m.group(1) if m else "?"
    return out


os.chdir(REPO)
print("== repo", git("rev-parse", "--short", "HEAD").strip())

# 1. manifest
print("\n[1] summary manifest")
raw = open(SUMMARY + "/manifest.txt", "rb").read()
ok("manifest header", raw.startswith(b"#") and b"\xef\xbb\xbf" not in raw and b"\r\n" not in raw,
   "first line '#', no BOM, LF")
ok("manifest does not list itself", b"manifest.txt" not in raw.split(b"\n", 2)[2])
n = 0
for line in raw.decode("utf-8").splitlines():
    if not line.strip() or line.startswith("#"):
        continue
    h, b, rel = line.split()
    ok("manifest entry " + rel, sha256(rel) == h and str(os.path.getsize(rel)) == b)
    n += 1
ok("manifest entry count", n == 2, str(n))

# 2. accounting
print("\n[2] requirement accounting")
m = json.load(open(MAP, encoding="utf-8"))
r = json.load(open(SUMMARY + "/receipt.json", encoding="utf-8"))
counts = {k: m["summary"][k] for k in ("requirements_total", "met", "partial", "unmet", "uncovered_total")}
want = {"requirements_total": 12, "met": 8, "partial": 4, "unmet": 0, "uncovered_total": 6}
ok("map.json summary", counts == want, json.dumps(counts))
ok("receipt.requirement_counts", r["requirement_counts"] ==
   {"total": 12, "met": 8, "partial": 4, "unmet": 0, "uncovered_extra": 6})
req = {x["requirement_id"]: x["status"] for x in m["requirements"]}
exp = {"G6-1.1": "met", "G6-1.2": "partial", "G6-1.3": "met", "G6-1.4": "met",
       "G6-1.5": "met", "G6-1.6": "met", "G6-1.7": "partial", "G6-2.1": "partial",
       "G6-2.2": "met", "G6-2.3": "met", "G6-2.4": "partial", "G6-3.1": "met"}
ok("map.json per-requirement status", req == exp, json.dumps(req, ensure_ascii=False))
rec_rows = {x["id"]: x["status"] for x in r["requirements"]}
ok("receipt requirement rows", rec_rows == exp, json.dumps(rec_rows, ensure_ascii=False))
readme = open(SUMMARY + "/README.md", encoding="utf-8").read()
bad = []
for rid, st in exp.items():
    row = [l for l in readme.splitlines() if l.startswith("| " + rid + " ")]
    if len(row) != 1 or ("**" + st) not in row[0]:
        bad.append(rid)
ok("README 12-row status column", not bad, "mismatch=" + ",".join(bad))
unc = {x["requirement_id"]: x.get("status") for x in m["uncovered"]}
ok("uncovered ids", sorted(unc) == ["UNC-%d" % i for i in range(1, 7)], json.dumps(unc))
ok("UNC-1 partial / UNC-2 closed / UNC-3 partial",
   (unc["UNC-1"], unc["UNC-2"], unc["UNC-3"]) == ("partial", "closed", "partial"))

# 3. references
print("\n[3] evidence/card references")
pat = re.compile(r"2026-\d\d-\d\d-[a-z0-9-]+")
status = {}
for p in glob.glob(".normify/changes/*.json"):
    d = json.load(open(p, encoding="utf-8"))
    status[d["id"]] = d["status"]
for f in (SUMMARY + "/README.md", SUMMARY + "/receipt.json"):
    t = open(f, encoding="utf-8").read()
    for cid in sorted(set(pat.findall(t))):
        st = status.get(cid, "MISSING")
        print("  ref", os.path.basename(f), st, cid)
        if cid.endswith("-check"):
            continue  # the QA check card itself
        ok("ref verified " + cid, st == "verified", st)
gpu_window = "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases"
for case in ("hfield-capsule", "hfield-sphere", "sdf-box"):
    ok("device window case " + case, os.path.isdir(gpu_window + "/" + case))
for d in ("g10-p5-hfield-host", "g10-p5-sdf-host", "g10-p5-sdf-relmat-fix",
          "g10-p5-host-ccd-overflow-bit-fix"):
    ok("host evidence dir " + d, os.path.isdir("docs/development-evidence/" + d))
for plan, host in (("4834", "g10-p5-acceptance-map/perf/perf-summary.json"),):
    ok("perf number " + plan + " in " + host,
       plan in open("docs/development-evidence/" + host, encoding="utf-8").read())

# 4. the 54 claim
print("\n[4] '54 subcards' claim")
dirs = sorted(os.path.basename(d) for d in glob.glob("docs/development-evidence/g10-p5-*")
              if os.path.isdir(d) and os.path.basename(d) != "g10-p5-summary-check")
print("  g10-p5 evidence dirs:", len(dirs))
ok("dir count is 54", len(dirs) == 54, str(len(dirs)))
allcards = [json.load(open(p, encoding="utf-8")) for p in glob.glob(".normify/changes/*.json")]
owners = {b: [] for b in dirs}
for c in allcards:
    for f in (c.get("schedule") or {}).get("files") or []:
        b = os.path.basename((f or "").replace("\\", "/").rstrip("/"))
        if b in owners:
            owners[b].append((c["id"], c["status"]))
ok("every dir maps to a verified card",
   all(any(s == "verified" for _, s in v) for v in owners.values()),
   "unmapped=" + ",".join(b for b, v in owners.items()
                          if not any(s == "verified" for _, s in v)))
for rev in (CLOSEOUT, FIX):
    st = card_statuses(rev)
    from collections import Counter
    c = Counter(st.values())
    print("  snapshot", rev, "files", len(st), dict(c))
    if rev == FIX:
        ok("snapshot verified==52", c["verified"] == 52, str(c["verified"]))
        ok("snapshot abandoned==4", c["abandoned"] == 4, str(c["abandoned"]))
        ok("snapshot proposed==2", c["proposed"] == 2, str(c["proposed"]))
        ok("58-4==54 (non-abandoned incl. 2 proposed)", len(st) - c["abandoned"] == 54)
t = open(SUMMARY + "/README.md", encoding="utf-8").read()
ok("README claims 54 verified subcards", "54 张子卡全部 verified" in t)
ok("no 54-item index file in summary dir",
   len(glob.glob(SUMMARY + "/*")) == 3, ",".join(os.path.basename(p) for p in glob.glob(SUMMARY + "/*")))

# 5. boundaries
print("\n[5] boundaries")
diff = git("diff", "--stat", BASE + "..HEAD", "--", "src", "tests", "fixtures",
           "docs/development-evidence/g10-p5-acceptance-map")
ok("zero diff src/tests/fixtures/acceptance-map since " + BASE, diff.strip() == "", diff.strip())
names = git("show", "--name-only", "--format=", CLOSEOUT).split()
ok("closeout touches only summary+card", all(p.startswith(("docs/development-evidence/g10-p5-summary/",
                                                          ".normify/changes/")) for p in names), ",".join(names))
names = git("show", "--name-only", "--format=", FIX).split()
ok("fix touches only summary+card", all(p.startswith(("docs/development-evidence/g10-p5-summary/",
                                                      ".normify/changes/")) for p in names), ",".join(names))
num = {}
for line in git("show", "--numstat", "--format=", FIX).splitlines():
    a, d, p = line.split("\t")
    num[p] = (int(a), int(d))
    print("  fix numstat", a, d, p)
ok("fix numstat exactly NP01 wording + addenda", num == {
    ".normify/changes/2026-11-04-g10-p5-family-closeout.json": (9, 1),
    "docs/development-evidence/g10-p5-summary/README.md": (1, 1),
    "docs/development-evidence/g10-p5-summary/manifest.txt": (2, 2),
    "docs/development-evidence/g10-p5-summary/receipt.json": (1, 1)}, json.dumps(num))
add = [l for l in git("show", "--format=", FIX, "--",
                      SUMMARY + "/README.md", SUMMARY + "/receipt.json").splitlines()
       if l.startswith(("+", "-")) and not l.startswith(("+++", "---"))]
ok("fix changed lines are NP01 only",
   all(("NP01" in l) or ("E " in l) or ("E指标" in l) or ("A2" in l) or ("metric" in l)
       or ("E criterion" in l) for l in add), str(len(add)) + " lines")
ok("fix does not change accounting tokens",
   "met 8" in t and "partial 4" in t and "unmet 0" in t)
terms = ["完整 G10 通过", "产品碰撞验收通过", "parity pass", "设备全路径覆盖"]
for term in terms:
    ok("forbidden claim absent: " + term,
       term not in readme and term not in json.dumps(r, ensure_ascii=False))

# 6. encoding audit of this evidence dir (logs are written by run_logged.py as UTF-8/LF)
print("\n[6] encoding audit (evidence files except logs)")
for p in sorted(glob.glob("docs/development-evidence/g10-p5-summary-check/**/*", recursive=True)):
    if not os.path.isfile(p) or "/logs/" in p.replace("\\", "/"):
        continue
    b = open(p, "rb").read()
    ok("utf8 no BOM no CR " + os.path.basename(p),
       not b.startswith((b"\xef\xbb\xbf", b"\xff\xfe", b"\xfe\xff")) and b"\r" not in b)

print("\nRESULT:", "PASS" if not fails else "FAIL", "fails=" + ",".join(fails))
raise SystemExit(0 if not fails else 1)
