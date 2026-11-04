# -*- coding: utf-8 -*-
import csv
import hashlib
import io
import json
import os
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
A = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-attempt5")
AT = os.path.join(A, "attempt")


def sha(b):
    return hashlib.sha256(b).hexdigest()


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

print("== 0. evidence files (encoding/bytes/sha) ==")
for name in ("README.md", "attempt/canonical-rebuild.log", "attempt/entity-flex.log",
             "attempt/freeze.log", "attempt/guards-frozen.json", "attempt/monitor-pre-window.csv"):
    p = os.path.join(A, *name.split("/"))
    b = open(p, "rb").read()
    if b[:2] == b"\xff\xfe":
        enc = "UTF-16LE"
    elif b[:3] == b"\xef\xbb\xbf":
        enc = "UTF-8-BOM"
    else:
        enc = "UTF-8" if b.count(b"\x00") == 0 else "binary/utf16?"
    print("  %-34s %7d bytes enc=%-10s sha256=%s" % (name, len(b), enc, sha(b)[:16]))

print("\n== 1. frozen guards json ==")
gjb = open(os.path.join(AT, "guards-frozen.json"), "rb").read()
gj = json.loads(gjb)
print("  bytes=%d sha256=%s (freeze.log claims 7508 / 9b6a3535...)" % (len(gjb), sha(gjb)))
print("  suite=%s attempt=%s status=%s counts=%s" % (gj.get("suite"), gj.get("attempt"), gj.get("status"), json.dumps(gj.get("counts"))))
print("  aggregate_sha256=%s" % gj.get("aggregate_sha256"))
print("  aggregate_inputs_sha256=%s" % gj.get("aggregate_inputs_sha256"))
print("  aggregate_algorithm=%s" % gj.get("aggregate_algorithm"))
gi = gj.get("guard_inputs", [])
fe = gj.get("frozen_exes", [])
print("  guard_inputs=%d frozen_exes=%d entity_order=%s" % (len(gi), len(fe), json.dumps(gj.get("entity_order"))))
for e in fe:
    print("   %-58s bytes=%-10s sha=%s" % (e.get("path"), e.get("bytes"), (e.get("sha256") or "")[:16]))
    print("      entity=%r pending_freeze=%r resolve=%r stash_sha=%r stash_bytes=%r" %
          (e.get("entity"), e.get("pending_freeze"), e.get("resolve"),
           (e.get("stash_sha256") or "")[:16], e.get("stash_bytes")))
    print("      live==recorded:", e.get("resolve") == e.get("sha256"),
          "| stash==recorded:", e.get("stash_sha256") == e.get("sha256"))
# aggregate per documented algorithm: sha256 over sha256 hex strings of entries in repo-relative path-sorted order, no separator
def agg(entries):
    ordered = sorted(entries, key=lambda e: e["path"])
    cat = "".join(e["sha256"] for e in ordered)
    return hashlib.sha256(cat.encode()).hexdigest(), len(ordered)
a_in, n_in = agg(gi)
a_all, n_all = agg(gi + fe)
print("  recomputed aggregate_inputs (%d entries): %s | matches=%s" % (n_in, a_in, a_in == gj.get("aggregate_inputs_sha256")))
print("  recomputed aggregate_all    (%d entries): %s | matches=%s" % (n_all, a_all, a_all == gj.get("aggregate_sha256")))
# guard inputs vs current files
bad = 0
for e in gi:
    p = os.path.join(REPO, *e["path"].split("/"))
    b = open(p, "rb").read()
    if sha(b) != e["sha256"] or len(b) != e["bytes"]:
        bad += 1
        print("  INPUT MISMATCH", e["path"])
print("  guard_inputs vs current files: bad=%d/%d" % (bad, len(gi)))
print("  frozen exe entries == entity_order count:", len(fe) == len(gj.get("entity_order", [])))

print("\n== 2. freeze.log (UTF-16LE) ==")
fl = open(os.path.join(AT, "freeze.log"), "rb").read().decode("utf-16").strip()
print(" ", fl)

def read_text(name):
    b = open(os.path.join(AT, name), "rb").read()
    if b[:2] == b"\xff\xfe":
        return b.decode("utf-16")
    return b.decode("utf-8", "replace")


print("\n== 3. canonical rebuild log ==")
cr = read_text("canonical-rebuild.log")
print("  compile lines:", cr.count("Compiling mjwarp-rs"))
print("  finished line:", [l.strip() for l in cr.splitlines() if "Finished" in l])
print("  executable line:", [l.strip() for l in cr.splitlines() if "Executable" in l])
print("  contains 'sha256':", "sha256" in cr.lower())

print("\n== 4. entity run log ==")
el = read_text("entity-flex.log")
print("  'running 1 test':", "running 1 test" in el)
print("  result line:", [l.strip() for l in el.splitlines() if l.strip().startswith("test result:")])
print("  failure line:", [l.strip() for l in el.splitlines() if "render_flex_validate.rs:347" in l])
print("  left/right:", [l.strip() for l in el.splitlines() if l.strip().startswith(("left:", "right:"))])
print("  'FAILED':", "FAILED" in el, "| 'Compiling':", "Compiling" in el)
print("  finished times:", [l.strip().split("in ")[-1] for l in el.splitlines() if "Finished" in l][:2])
print("  D5 lines:", len([l for l in el.splitlines() if l.strip().startswith("D5 cam")]))

print("\n== 5. monitor csv ==")
raw = open(os.path.join(AT, "monitor-pre-window.csv"), "rb").read()
rows = list(csv.DictReader(io.StringIO(raw.decode("utf-8", "replace"))))
print("  columns:", list(rows[0].keys()) if rows else None)
print("  samples:", len(rows))
print("  first/last ts:", rows[0].get("ts"), "/", rows[-1].get("ts"))
free_key = next(k for k in rows[0] if "free" in k.lower())
util_key = next(k for k in rows[0] if "util" in k.lower())
comp_key = next((k for k in rows[0] if "compute_filtered" in k.lower()), None)
build_key = next((k for k in rows[0] if k.lower() in ("build", "build_proc", "build_processes")), None)
floor = 11688


def num(v):
    try:
        return float(v)
    except Exception:
        return None


clean = []
for r in rows:
    ok = True
    ok &= num(r[free_key]) is not None and num(r[free_key]) >= floor
    ok &= num(r[util_key]) is not None and num(r[util_key]) < 40
    if comp_key:
        ok &= num(r[comp_key]) == 0
    if build_key:
        ok &= num(r[build_key]) == 0
    clean.append(ok)
final = 0
for c in reversed(clean):
    if c:
        final += 1
    else:
        break
best = cur = 0
for c in clean:
    cur = cur + 1 if c else 0
    best = max(best, cur)
print("  keys: free=%s util=%s compute_filtered=%s build=%s" % (free_key, util_key, comp_key, build_key))
print("  final clean streak=%d (README says 60/60) | longest streak=%d" % (final, best))
dirty = [i for i, c in enumerate(clean) if not c]
if dirty:
    dv = [num(rows[i][free_key]) for i in dirty if num(rows[i][free_key]) is not None]
    du = [num(rows[i][util_key]) for i in dirty if num(rows[i][util_key]) is not None]
    print("  dirty samples=%d rows %d..%d | free_min=%s | util_max=%s" % (len(dirty), dirty[0] + 1, dirty[-1] + 1, min(dv), max(du)))
    print("  dirty ts range:", rows[dirty[0]].get("ts"), "..", rows[dirty[-1]].get("ts"))
print("  compute_filtered all zero:", all(num(r[comp_key]) == 0 for r in rows) if comp_key else "n/a")
print("  build all zero:", all(num(r[build_key]) == 0 for r in rows) if build_key else "n/a")

print("\n== 6. material signed floor ==")
mat = open(os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-validate", "materials", "attempt5-criteria.md"), encoding="utf-8").read()
print("  criteria material mentions 11688:", ("11,688" in mat) or ("11688" in mat))
import glob
for fp in glob.glob(os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-validate", "**", "*.md"), recursive=True) + \
          glob.glob(os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-validate", "**", "*.txt"), recursive=True) + \
          glob.glob(os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-validate", "*.md")):
    try:
        t = open(fp, encoding="utf-8").read()
    except Exception:
        continue
    if "11,688" in t or "11688" in t:
        for i, line in enumerate(t.splitlines(), 1):
            if "11,688" in line or "11688" in line:
                print("   %s:%d: %s" % (os.path.basename(fp), i, line.strip()[:160]))

print("\n== 7. test assert state at attempt5 commit (6e9aa585) ==")
_show = subprocess.run(["git", "show", "6e9aa585:tests/render_flex_validate.rs"], cwd=REPO,
                       capture_output=True)
_txt = _show.stdout.decode("utf-8", "replace").splitlines()
for i in range(339, 352):
    if i < len(_txt):
        print("  %d: %s" % (i + 1, _txt[i].strip()))
_cur = open(os.path.join(REPO, "tests", "render_flex_validate.rs"), encoding="utf-8").read()
print("  current worktree has allowlist implementation:", "assert_depth_allowlisted" in _cur)
src_support = os.path.join(REPO, "tests", "support", "render_bvh", "allowlist.rs")
print("  support module present now:", os.path.exists(src_support))
