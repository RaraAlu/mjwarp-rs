# -*- coding: utf-8 -*-
"""QA read-only verification of the R06-S3b closure evidence (commit db082561).

Checks the six review items on the QA card:
  (1) B-class premises: frozen reference 6 files, GPU/recipe, attempt6 compile form, guards attempt=2 16/16
  (2) signed allowlist minimal scope verbatim / no expansion
  (3) D8 321 px vs 1 px separated
  (4) cross-round closure: src diff empty; two-run sha identical; r2 pixel detail
  (5) non-claims present
  (6) evidence index 11 items resolve to real commits
Plus closure manifest self-check (git blobs).

Usage: python qa_closure_check.py [--out <logfile>]
"""
import csv
import hashlib
import io
import json
import os
import subprocess
import sys

REPO = r"D:\Rust\mjwarp-rs"
CLO = "docs/development-evidence/render-bvh/r06-s3b-closure"
ATT6 = "docs/development-evidence/render-bvh/r06-s3b-flex-attempt6"
REFDIR = "fixtures/render-bvh/scene-flex/reference/base"

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


def git(*a, binary=False):
    p = subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s %s" % (" ".join(a), p.stderr.decode("utf-8", "replace")))
    return p.stdout if binary else p.stdout.decode("utf-8", "replace")


def blob(rev, path):
    return git("cat-file", "-p", "%s:%s" % (rev, path), binary=True)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def read_json(rev, path):
    return json.loads(blob(rev, path).decode("utf-8"))


print("== 0. objects ==")
print("  closure evidence commit:", git("rev-parse", "db082561").strip()[:12])
print("  closure card:", git("rev-parse", "9351a339").strip()[:12])
print("  HEAD:", git("rev-parse", "HEAD").strip()[:12])

print("\n== 1. closure manifest self-check (git blobs) ==")
man = blob("db082561", CLO + "/manifest.txt")
print("  manifest bytes=%d sha=%s first=%r" % (len(man), sha(man), chr(man[0])))
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob("db082561", rel if rel.startswith("docs/") else CLO + "/" + rel)
    if sha(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  entries=%d bad=%d" % (len(entries), bad))
rd = blob("db082561", CLO + "/README.md")
print("  README bytes=%d sha=%s" % (len(rd), sha(rd)[:16]))
fj = read_json("db082561", CLO + "/facts.json")
print("  facts keys:", list(fj.keys()))
print("  facts evidence_index size:", len(fj.get("evidence_index", {})))

print("\n== 2. P1 frozen reference 6 files (current worktree) ==")
p1 = fj["premises"]["P1_frozen_reference"]["files"]
bad1 = 0
for name, exp in p1.items():
    b = open(os.path.join(REPO, *REFDIR.split("/"), name), "rb").read()
    ok = sha(b) == exp["sha256"] and len(b) == exp["bytes"]
    bad1 += 0 if ok else 1
    print("  %-16s bytes=%-8d %s" % (name, len(b), "OK" if ok else "MISMATCH"))
print("  P1 bad=%d/%d" % (bad1, len(p1)))

print("\n== 3. P2/P3 GPU + recipe from capture provenance ==")
cap = read_json("db082561", REFDIR + "/capture.json")
def find(obj, keys):
    out = {}
    stack = [obj]
    while stack:
        cur = stack.pop()
        if isinstance(cur, dict):
            for k, v in cur.items():
                if k in keys:
                    out[k] = v
                if isinstance(v, (dict, list)):
                    stack.append(v)
        elif isinstance(cur, list):
            stack.extend(cur)
    return out
capkeys = find(cap, {"gpu", "gpu_name", "driver", "driver_version", "cuda", "cuda_version",
                     "use_fast_math", "use_shadows", "samples_per_pixel", "flex_render_smooth",
                     "use_precomputed_rays", "render_skybox", "enable_backface_culling"})
print("  capture.json found:", json.dumps(capkeys, ensure_ascii=False)[:400])
print("  claimed P2:", fj["premises"]["P2_gpu"])
print("  claimed P3:", json.dumps(fj["premises"]["P3_recipe"], ensure_ascii=False))

print("\n== 4. P4 attempt6 compile form + guards attempt=2 ==")
g = read_json("db082561", ATT6 + "/guards-after.json")
print("  guards: attempt=%s status=%s counts=%s" % (g.get("attempt"), g.get("status"), json.dumps(g.get("counts"))))
print("  guards sha256=%s bytes=%d" % (sha(blob("db082561", ATT6 + "/guards-after.json")), len(blob("db082561", ATT6 + "/guards-after.json"))))
print("  aggregate_all=%s" % g.get("aggregate_sha256"))
print("  aggregate_inputs=%s" % g.get("aggregate_inputs_sha256"))
fe = g.get("frozen_exes", [])
for e in fe:
    print("   %-58s bytes=%-10s sha=%s pending_freeze=%r stash_ok=%s"
          % (e.get("path"), e.get("bytes"), (e.get("sha256") or "")[:16], e.get("pending_freeze"),
             e.get("stash_sha256") == e.get("sha256")))
print("  claimed P4:", json.dumps(fj["premises"]["P4_compile_form"], ensure_ascii=False)[:400])
# aggregate recompute per documented algorithm
def agg(entries):
    cat = "".join(e["sha256"] for e in sorted(entries, key=lambda x: x["path"]))
    return sha(cat.encode())
gi = g.get("guard_inputs", [])
print("  recomputed aggregate_inputs:", agg(gi), "matches:", agg(gi) == g.get("aggregate_inputs_sha256"))
print("  recomputed aggregate_all   :", agg(gi + fe), "matches:", agg(gi + fe) == g.get("aggregate_sha256"))

print("\n== 5. attempt6 entity PASS + two-run sha + zero compile after freeze ==")
el = blob("db082561", ATT6 + "/entity-run.log")
txt = el.decode("utf-16") if el[:2] == b"\xff\xfe" else el.decode("utf-8", "replace")
print("  log encoding:", "UTF-16LE" if el[:2] == b"\xff\xfe" else "UTF-8")
print("  running 1 test:", "running 1 test" in txt)
print("  result lines:", [l.strip() for l in txt.splitlines() if l.strip().startswith("test result:")])
print("  Compiling lines:", txt.count("Compiling"))
print("  Finished lines:", [l.strip() for l in txt.splitlines() if "Finished" in l][:2])
tr = blob("db082561", ATT6 + "/two-run-sha.txt").decode("utf-8", "replace")
print("  two-run-sha.txt first lines:", tr.strip().splitlines()[:4])
print("  two-run values: rgb d84ecaed:", "d84ecaed" in tr, "| depth 98292153:", "98292153" in tr, "| seg c4be9853:", "c4be9853" in tr)

print("\n== 6. attempt6 monitor 60/60 clean vs floor ==")
mon = blob("db082561", ATT6 + "/monitor-official.csv")
rows = list(csv.DictReader(io.StringIO(mon.decode("utf-8", "replace"))))
print("  rows:", len(rows), "columns:", list(rows[0].keys())[:10])
fk = next(k for k in rows[0] if "free" in k.lower())
uk = next(k for k in rows[0] if "util" in k.lower())
ck = next((k for k in rows[0] if "compute_filtered" in k.lower()), None)
bk = next((k for k in rows[0] if k.lower() == "build"), None)
floor = 11688
clean = []
for r in rows:
    ok = float(r[fk]) >= floor and float(r[uk]) < 40
    if ck:
        ok &= float(r[ck]) == 0
    if bk:
        ok &= float(r[bk]) == 0
    clean.append(ok)
print("  all clean:", all(clean), "| util min/max:", min(float(r[uk]) for r in rows), max(float(r[uk]) for r in rows),
      "| free_min:", min(float(r[fk]) for r in rows))

print("\n== 7. cross-round chain a) src diff empty between r2 and assert impl ==")
d = git("diff", "--name-only", "d35cab68..8c15ad75", "--", "src/")
print("  git diff d35cab68..8c15ad75 -- src/ output:", repr(d))
print("  empty:", d.strip() == "")

print("\n== 8. cross-round chain b) two-run sha identical to r2 ==")
r2sum = read_json("d35cab68", "docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/artifacts/verification-summary.json")
print("  r2 two_run_sha:", json.dumps(r2sum.get("two_run_sha")))
print("  r2 two_run_sha_equal:", r2sum.get("two_run_sha_equal"))
print("  attempt6 values match r2:",
      all(v in tr for v in (r2sum["two_run_sha"]["rgb"], r2sum["two_run_sha"]["depth"], r2sum["two_run_sha"]["seg"])))

print("\n== 9. cross-round chain c) r2 pixel detail recompute ==")
ref_seg = np_load = None
import numpy as np
rb = np.load(io.BytesIO(blob("db082561", REFDIR + "/seg.npz")), allow_pickle=False)["seg_cam2"].reshape(-1, 2).astype(np.int32)
refdepth = np.load(io.BytesIO(blob("db082561", REFDIR + "/depth.npz")), allow_pickle=False)["depth_cam2"].reshape(-1).astype(np.float32)
ref_bits = refdepth.view(np.uint32)
art = np.load(io.BytesIO(blob("a3f690df", "docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/artifacts/cam2-depth-seg-fixed.npz")), allow_pickle=False)
pb = art["depth_bits"].astype(np.uint32)
ps = art["seg"].astype(np.int32)
flat = 128 * 320 + 134
def orded(bits):
    i = bits.astype(np.int64)
    return np.where(i & 0x80000000, 0x80000000 - i, i)
ulp = abs(int(orded(pb[flat:flat + 1])[0]) - int(orded(ref_bits[flat:flat + 1])[0]))
print("  pixel flat=%d prod_bits=0x%08x ref_bits=0x%08x ulp=%d beyond4=%s"
      % (flat, int(pb[flat]), int(ref_bits[flat]), ulp, ulp > 4))
print("  seg product=%s reference=%s | flexid=%d type=%d"
      % (ps[flat].tolist(), rb[flat].tolist(), int(rb[flat][0]), int(rb[flat][1])))
flex = rb[:, 1] == 9
print("  cam2 flex px=%d (flex0=%d flex1=%d)" % (int(flex.sum()), int((flex & (rb[:, 0] == 0)).sum()), int((flex & (rb[:, 0] == 1)).sum())))
print("  mismatch count on full cam2:", int((pb != ref_bits).sum()))

print("\n== 10. non-claims + index resolution ==")
rtext = rd.decode("utf-8")
for pat in ("上游等价", "parity pass", "对外发布", "内部验收"):
    print("  README contains %-8s %s" % (pat, pat in rtext))
idx = fj["evidence_index"]
badidx = 0
for k, v in idx.items():
    p = subprocess.run(["git", "cat-file", "-t", "%s^{commit}" % v], cwd=REPO, capture_output=True)
    ok = p.returncode == 0 and p.stdout.strip() == b"commit"
    if not ok:
        badidx += 1
    print("  index %-24s %-10s exists=%s" % (k, v, ok))
print("  index items=%d unresolved=%d" % (len(idx), badidx))
print("\n== 11. verdict inputs ==")
print("  closure README index rows(§7) count:", rtext.split("## 7.")[-1].count("| **") if "## 7." in rtext else "n/a")
