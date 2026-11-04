# -*- coding: utf-8 -*-
"""QA independent recompute for the D8 review (QA card r06-s3b-flex-d8-review).

Inputs (all read-only):
  1) committed D8 evidence: artifacts/cam2-depth-seg.npz  (product cam2 depth bits + seg)
  2) committed reference:   fixtures/render-bvh/scene-flex/reference/base/{depth.npz,seg.npz}
  3) CPU-warp replay artifact (NOT committed; target-only): target/s3b-verify/cpu-replay/cpu_cam2_depth.npz
     -> its sha256/bytes are recorded in the QA evidence so the 98.8% overlap claim stays auditable.

No D8 script is imported or executed; only the two committed inputs + the replay artifact are read.
Usage: python qa_recompute.py [--no-replay] [--out <logfile>]
  --out writes the same text to <logfile> with LF line endings (deterministic bytes for the manifest).
"""
import hashlib
import json
import os
import platform
import sys

import numpy as np

REPO = r"D:\Rust\mjwarp-rs"
D8 = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-cam2-depth-gap")
REF = os.path.join(REPO, "fixtures", "render-bvh", "scene-flex", "reference", "base")
REPLAY = os.path.join(REPO, "target", "s3b-verify", "cpu-replay", "cpu_cam2_depth.npz")

if "--out" in sys.argv:
    _out_path = sys.argv[sys.argv.index("--out") + 1]
    _out_fh = open(_out_path, "w", encoding="utf-8", newline="\n")
    _orig_stdout = sys.stdout

    class _Tee(object):
        def write(self, s):
            _out_fh.write(s)
            _orig_stdout.write(s)
            return len(s)

        def flush(self):
            _out_fh.flush()
            _orig_stdout.flush()

    sys.stdout = _Tee()


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for blk in iter(lambda: f.read(1 << 20), b""):
            h.update(blk)
    return h.hexdigest()


def ordered(bits):
    i = bits.astype(np.int64)
    return np.where(i & 0x80000000, 0x80000000 - i, i)


def ulp_of(a_bits, b_bits):
    return np.abs(ordered(a_bits) - ordered(b_bits))


print("== environment ==")
print("python:", platform.python_version(), "| numpy:", np.__version__, "| platform:", platform.platform())
print("inputs:")
for p in (os.path.join(D8, "artifacts", "cam2-depth-seg.npz"),
          os.path.join(REF, "depth.npz"),
          os.path.join(REF, "seg.npz")):
    print("  %s  bytes=%d  sha256=%s" % (os.path.relpath(p, REPO), os.path.getsize(p), sha256(p)))

art = np.load(os.path.join(D8, "artifacts", "cam2-depth-seg.npz"))
prod_bits = art["depth_bits"].astype(np.uint32)
prod_seg = art["seg"].astype(np.int32)
ref_depth = np.load(os.path.join(REF, "depth.npz"))["depth_cam2"].reshape(-1).astype(np.float32)
ref_seg = np.load(os.path.join(REF, "seg.npz"))["seg_cam2"].reshape(-1, 2).astype(np.int32)
ref_bits = ref_depth.view(np.uint32)
assert prod_bits.size == ref_bits.size == 76800

ulp = ulp_of(prod_bits, ref_bits)
mism = prod_bits != ref_bits
beyond = ulp > 4
flex_mask = ref_seg[:, 1] == 9
flexid = ref_seg[:, 0]

print("\n== headline (product cam2 depth vs reference, bitwise) ==")
print("mismatched=%d beyond4=%d max_ulp=%d" % (int(mism.sum()), int(beyond.sum()), int(ulp.max())))

print("\n== ULP histogram: D8 committed inclusive-bounds reproduction vs QA disjoint ==")
edges = [0, 1, 2, 4, 16, 64, 1024, 10 ** 9]
inclusive = [int(((ulp >= lo) & (ulp <= hi)).sum()) for lo, hi in zip(edges[:-1], edges[1:])]
# explicit disjoint buckets: [0], [1], [2], [3..4], [5..16], [17..64], [65..1024], (>1024)
disjoint = [int((ulp == 0).sum()), int((ulp == 1).sum())]
for lo, hi in ((2, 2), (3, 4), (5, 16), (17, 64), (65, 1024)):
    disjoint.append(int(((ulp >= lo) & (ulp <= hi)).sum()))
disjoint.append(int((ulp > 1024).sum()))
print("D8 inclusive (labels 0,1,2-4,5-16,17-64,65-1024,>1024):", inclusive)
print("   sum =", sum(inclusive), "(vs 76800, double-counted =", sum(inclusive) - 76800, ")")
print("QA disjoint [0,1,2,3-4,5-16,17-64,65-1024,>1024]:", disjoint)
print("   sum =", sum(disjoint))
print("counts at shared endpoints 1/2/4/16/64:", [int((ulp == v).sum()) for v in (1, 2, 4, 16, 64)])

print("\n== distribution by region ==")
for fid in (0, 1):
    m = flex_mask & (flexid == fid)
    print("flexid%d: pixels=%d mismatched=%d beyond4=%d max_ulp=%d"
          % (fid, int(m.sum()), int((mism & m).sum()), int((beyond & m).sum()), int(ulp[m].max())))
print("non-flex mismatched=%d beyond4=%d" % (int((mism & ~flex_mask).sum()), int((beyond & ~flex_mask).sum())))
print("mismatch inside flex=%d / %d" % (int((mism & flex_mask).sum()), int(mism.sum())))
print("seg mismatched pixels=%d / %d | product==reference: %s"
      % (int((prod_seg != ref_seg).any(axis=1).sum()), prod_seg.shape[0], bool((prod_seg == ref_seg).all())))

absd = np.abs(prod_bits.view(np.float32)[mism].astype(np.float64) - ref_bits.view(np.float32)[mism].astype(np.float64))
print("abs diff (m): median=%.6e p90=%.6e max=%.6e" % (np.median(absd), np.percentile(absd, 90), absd.max()))

print("\n== region semantics ==")
ref0 = ref_bits.view(np.float32) == 0.0
print("reference depth==0 pixels=%d | all non-flex=%s | flex depth all nonzero=%s"
      % (int(ref0.sum()), bool((ref0 & flex_mask).sum() == 0),
         bool((ref_bits.view(np.float32)[flex_mask] > 0).all())))

print("\n== distance of mismatched pixels to flex silhouette ==")
H, W = 240, 320
mask = flex_mask.reshape(H, W)
pts = np.argwhere(mism.reshape(H, W))
padded = np.pad(mask, 1, constant_values=False)
boundary = mask & (~padded[:-2, 1:-1] | ~padded[2:, 1:-1] | ~padded[1:-1, :-2] | ~padded[1:-1, 2:])
bpts = np.argwhere(boundary)
d = np.sqrt(((pts[:, None, :] - bpts[None, :, :]) ** 2).sum(axis=2)).min(axis=1)
print("silhouette_px=%d mismatched_px=%d dist_median=%.2f <=2px=%d/%d <=4px=%d/%d"
      % (len(bpts), len(pts), np.median(d), int((d <= 2).sum()), len(d), int((d <= 4).sum()), len(d)))

print("\n== D8 manifest self-check (sha256 + bytes of every listed file) ==")
mpath = os.path.join(D8, "manifest.txt")
entries = 0
bad = 0
for line in open(mpath, encoding="utf-8"):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    sha, size, rel = line.split()
    full = os.path.join(REPO, rel.replace("/", os.sep))
    got_sha, got_size = sha256(full), os.path.getsize(full)
    ok = got_sha == sha and got_size == int(size)
    bad += 0 if ok else 1
    entries += 1
print("manifest entries=%d | mismatches=%d" % (entries, bad))

print("\n== D8 summary.json vs recomputed numbers ==")
summary = json.load(open(os.path.join(D8, "artifacts", "summary.json"), encoding="utf-8"))
checks = {
    "depth_mismatched": int(mism.sum()),
    "depth_beyond4ulp": int(beyond.sum()),
    "depth_max_ulp": int(ulp.max()),
    "non_flex_mismatched": int((mism & ~flex_mask).sum()),
    "seg_mismatched_pixels_vs_reference": int((prod_seg != ref_seg).any(axis=1).sum()),
    "mismatch_inside_flex": int((mism & flex_mask).sum()),
    "reference_cam2_depth_zero_pixels": int(ref0.sum()),
}
all_ok = True
for k, v in checks.items():
    ok = summary.get(k) == v
    all_ok &= ok
    print("  %-40s summary=%-6s recompute=%-6s %s" % (k, summary.get(k), v, "OK" if ok else "MISMATCH"))
pf = summary.get("per_flexid", {})
for fid in (0, 1):
    m = flex_mask & (flexid == fid)
    ok = (pf[str(fid)]["pixels"] == int(m.sum()) and pf[str(fid)]["mismatched"] == int((mism & m).sum())
          and pf[str(fid)]["beyond4"] == int((beyond & m).sum()) and pf[str(fid)]["max_ulp"] == int(ulp[m].max()))
    all_ok &= ok
    print("  per_flexid[%d] %s" % (fid, "OK" if ok else "MISMATCH"))
dist = summary.get("mismatch_dist_to_flex_silhouette_px", {})
ok = (dist.get("<=2px") == int((d <= 2).sum()) and dist.get("<=4px") == int((d <= 4).sum())
      and abs(dist.get("median", 0) - float(np.median(d))) < 1e-9)
all_ok &= ok
print("  dist_to_silhouette %s" % ("OK" if ok else "MISMATCH"))
hist_sum = sum(summary.get("ulp_histogram", {}).values())
print("  ulp_histogram sum=%d (expected 76800; known inclusive-bounds defect, see README section 4)" % hist_sum)
print("  summary consistency:", "ALL OK" if all_ok else "HAS MISMATCH")

print("\n== D8 dump files on disk + two-run hash identity (target-only, optional) ==")
DUMP = os.path.join(REPO, "target", "render-bvh", "r06-s3b-flex-window", "d8")
expected = {
    "depth_all.f32le": (787200, "fa4cceaaf7eeca70f3307ead99ad24f095727fdbb904d7de1c33e9eb9ffef20e"),
    "seg_all.i32le": (1574400, "c4be985324b883abcbaa63fa44888b016fd28d2d266858aaa493877b49a64185"),
    "meta.json": (1080, "a347b64461bf93e4ac427fcf5beb16d001df6e8acd763cc529c6dd8564b10b2b"),
}
if not os.path.isdir(DUMP):
    print("  SKIPPED (dump dir absent: %s)" % os.path.relpath(DUMP, REPO))
else:
    for name, (exp_bytes, exp_sha) in expected.items():
        p = os.path.join(DUMP, name)
        ok = os.path.exists(p) and os.path.getsize(p) == exp_bytes and sha256(p) == exp_sha
        print("  %-16s %s" % (name, "OK" if ok else "MISMATCH/absent"))
    meta = json.load(open(os.path.join(DUMP, "meta.json"), encoding="utf-8"))
    trh = meta.get("two_run_hashes", [])
    same = len(trh) == 6 and trh[:3] == trh[3:]
    print("  two_run_hashes identical=%s (rgb=%s... depth=%s... seg=%s...)"
          % (same, trh[0][:8] if trh else "-", trh[1][:8] if len(trh) > 1 else "-",
             trh[2][:8] if len(trh) > 2 else "-"))

print("\n== D8 accounting quotes (verbatim, file:line) ==")
facts = os.path.join(D8, "logs", "d8-pre-window-facts.txt")
pats = ("test_exe ", "guards_sha256=", "guards_status=", "guards_exe_", "wip_test_exe",
        "committed_test_exe", "compute_filtered_count=", "fresh three-point baseline",
        "run_exit=", "dump depth_all", "dump seg_all", "dump meta", "wip_restored=")
for ln, line in enumerate(open(facts, encoding="utf-8"), 1):
    if any(p in line for p in pats):
        print("d8-pre-window-facts.txt:%d: %s" % (ln, line.rstrip("\n")))
dumpsha = os.path.join(D8, "logs", "dump-sha256.txt")
for ln, line in enumerate(open(dumpsha, encoding="utf-8"), 1):
    print("dump-sha256.txt:%d: %s" % (ln, line.rstrip("\n")))

if "--no-replay" in sys.argv or not os.path.exists(REPLAY):
    print("\n== CPU replay cross-check: SKIPPED (artifact absent) ==")
    sys.exit(0)

print("\n== CPU-warp replay cross-check (target-only artifact) ==")
print("replay artifact: %s bytes=%d sha256=%s"
      % (os.path.relpath(REPLAY, REPO), os.path.getsize(REPLAY), sha256(REPLAY)))
rep = np.load(REPLAY)
rep_bits = rep["depth"].astype(np.float32).view(np.uint32)
rep_seg = rep["seg"].astype(np.int32)
rulp = ulp_of(rep_bits, ref_bits)
rmism = rep_bits != ref_bits
print("replay mismatched=%d beyond4=%d | replay seg==reference: %s"
      % (int(rmism.sum()), int((rulp > 4).sum()), bool((rep_seg == ref_seg).all())))
inter = int((mism & rmism).sum())
print("product mismatched=%d | replay mismatched=%d | intersection=%d -> %.1f%%"
      % (int(mism.sum()), int(rmism.sum()), inter, 100.0 * inter / int(mism.sum())))
print("replay ULP at product-equal pixels: median=%.1f max=%d"
      % (np.median(rulp[~mism]), int(rulp[~mism].max())))
print("replay ULP at product-mismatch pixels: median=%.1f max=%d"
      % (np.median(rulp[mism]), int(rulp[mism].max())))
print("product beyond4=%d | among them replay beyond4=%d"
      % (int(beyond.sum()), int((beyond & (rulp > 4)).sum())))
print("replay beyond4 in flex region=%d/%d"
      % (int(((rulp > 4) & flex_mask).sum()), int(flex_mask.sum())))
