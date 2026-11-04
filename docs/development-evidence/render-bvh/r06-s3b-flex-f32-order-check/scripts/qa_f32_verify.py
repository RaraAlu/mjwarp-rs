# -*- coding: utf-8 -*-
"""QA independent verification of the R06-S3b f32-order fix evidence (residual 1 px review).

Read-only. Inputs:
  A) this check's derived product cam2 bits: artifacts/cam2-depth-seg-fixed.npz
     (derived from the reviewer-time product dump; derivation + source dump sha recorded below)
  B) committed reference: fixtures/render-bvh/scene-flex/reference/base/{depth,seg}.npz
  C) reviewed object evidence: docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/
     (uncommitted WIP of card ...-f32-order-fix-r2 at review time)
  D) D8 baseline artifact: docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap/
  E) optional target-only dumps (target/render-bvh/.../f32-order/residual/*.f32le) to re-derive A)

No D8/Bot analysis script is imported; only data files are read.
Usage: python qa_f32_verify.py [--out <logfile>]
  --out writes the same text to <logfile> with LF endings (deterministic bytes).
"""
import hashlib
import json
import os
import platform
import sys

import numpy as np

REPO = r"D:\Rust\mjwarp-rs"
EV = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-f32-order-check")
OBJ = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-f32-order")
D8 = os.path.join(REPO, "docs", "development-evidence", "render-bvh", "r06-s3b-flex-cam2-depth-gap")
REF = os.path.join(REPO, "fixtures", "render-bvh", "scene-flex", "reference", "base")
RES = os.path.join(REPO, "target", "render-bvh", "r06-s3b-flex-window", "f32-order", "residual")

if "--out" in sys.argv:
    _out_path = sys.argv[sys.argv.index("--out") + 1]
    _out_fh = open(_out_path, "w", encoding="utf-8", newline="\n")
    _orig = sys.stdout

    class _Tee(object):
        def write(self, s):
            _out_fh.write(s)
            _orig.write(s)
            return len(s)

        def flush(self):
            _out_fh.flush()
            _orig.flush()

    sys.stdout = _Tee()


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for blk in iter(lambda: f.read(1 << 20), b""):
            h.update(blk)
    return h.hexdigest()


def bytes_of(path):
    return os.path.getsize(path)


def ordered(bits):
    i = bits.astype(np.int64)
    return np.where(i & 0x80000000, 0x80000000 - i, i)


def ulp(a, b):
    return np.abs(ordered(a) - ordered(b))


print("== environment ==")
print("python:", platform.python_version(), "| numpy:", np.__version__, "| os:", platform.platform())

ref_bits = np.load(os.path.join(REF, "depth.npz"))["depth_cam2"].reshape(-1).astype(np.float32).view(np.uint32)
ref_seg = np.load(os.path.join(REF, "seg.npz"))["seg_cam2"].reshape(-1, 2).astype(np.int32)
flex0 = (ref_seg[:, 1] == 9) & (ref_seg[:, 0] == 0)
flex1 = (ref_seg[:, 1] == 9) & (ref_seg[:, 0] == 1)
d8_bits = np.load(os.path.join(D8, "artifacts", "cam2-depth-seg.npz"))["depth_bits"].astype(np.uint32)

print("\n== 1. reviewed-object identity (uncommitted WIP at review time) ==")
obj_manifest = os.path.join(OBJ, "manifest.txt")
n_obj = bad_obj = 0
for line in open(obj_manifest, encoding="utf-8"):
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    s, size, rel = line.split()
    p = os.path.join(OBJ, *rel.split("/"))
    n_obj += 1
    if sha(p) != s or bytes_of(p) != int(size):
        bad_obj += 1
        print("  BAD", rel)
print("object manifest: entries=%d bad=%d | manifest sha256=%s" % (n_obj, bad_obj, sha(obj_manifest)))
for rel in ("README.md", "artifacts/verification-summary.json", "artifacts/residual-pixel.json",
            "artifacts/flex0-oracle.npz", "artifacts/flex1-oracle.npz",
            "artifacts/fingerprint-canonical-rebuild.json"):
    p = os.path.join(OBJ, *rel.split("/"))
    print("  %-46s bytes=%-7d sha256=%s" % (rel, bytes_of(p), sha(p)))

print("\n== 2. derived product bits vs reference (independently recomputed) ==")
art_path = os.path.join(EV, "artifacts", "cam2-depth-seg-fixed.npz")
art = np.load(art_path)
pb = art["depth_bits"].astype(np.uint32)
segb = art["seg"].astype(np.int32)
print("derived npz: %s bytes=%d sha256=%s" % (os.path.relpath(art_path, REPO), bytes_of(art_path), sha(art_path)))
mism = pb != ref_bits
u = ulp(pb, ref_bits)
print("pixels=%d mismatched=%d beyond4=%d max_ulp=%d" % (pb.size, int(mism.sum()), int((u > 4).sum()), int(u.max())))
for i in np.flatnonzero(mism):
    y, x = divmod(int(i), 320)
    print("  mismatch px=(%d,%d) flat=%d ulp=%d prod_bits=0x%08x ref_bits=0x%08x prod_m=%.9g ref_m=%.9g"
          % (x, y, int(i), int(u[i]), int(pb[i]), int(ref_bits[i]),
             float(np.float32(pb[i].view(np.float32))), float(ref_bits[i].view(np.float32))))
segm = (segb != ref_seg).any(axis=1)
print("seg mismatched pixels=%d / %d" % (int(segm.sum()), segb.shape[0]))
print("mismatch in flex0=%d flex1=%d non-flex=%d" % (int((mism & flex0).sum()), int((mism & flex1).sum()),
                                                     int((mism & ~(flex0 | flex1)).sum())))
d8_mism = d8_bits != ref_bits
print("D8 mismatched=%d -> post-fix %d | fixed=%d | post-fix subset of D8=%s"
      % (int(d8_mism.sum()), int(mism.sum()), int((d8_mism & ~mism).sum()), bool((mism & ~d8_mism).sum() == 0)))

print("\n== 3. derive-check against target dumps (optional, target-only) ==")
dump_d = os.path.join(RES, "cam2-depth.f32le")
dump_s = os.path.join(RES, "cam2-seg.i32le")
if os.path.exists(dump_d) and os.path.exists(dump_s):
    dd = np.fromfile(dump_d, dtype="<f4")
    ds = np.fromfile(dump_s, dtype="<i4").reshape(-1, 2)
    print("  cam2-depth.f32le bytes=%d sha256=%s" % (bytes_of(dump_d), sha(dump_d)))
    print("  cam2-seg.i32le   bytes=%d sha256=%s" % (bytes_of(dump_s), sha(dump_s)))
    print("  derived == on-disk dumps: depth=%s seg=%s"
          % (bool((dd.view(np.uint32) == pb).all()), bool((ds.astype(np.int32) == segb).all())))
else:
    print("  SKIPPED (target dumps absent; derived npz sha is the binding record)")

print("\n== 4. oracle npz: reproduction and binding ==")
o0 = np.load(os.path.join(OBJ, "artifacts", "flex0-oracle.npz"))
o1 = np.load(os.path.join(OBJ, "artifacts", "flex1-oracle.npz"))
for tag, o, mask, bind_src in (("flex0", o0, flex0, pb), ("flex1", o1, flex1, d8_bits)):
    pred = o["depth_pred"].view(np.uint32)
    ref = o["ref_depth"].view(np.uint32)
    prod_o = o["prod_depth"].view(np.uint32)
    print("  %s: px=%d oracle==reference %d/%d | oracle vs product mismatch=%d | ref binds reference=%s | prod binds %s product=%s"
          % (tag, pred.size, int((pred == ref).sum()), pred.size, int((pred != prod_o).sum()),
             bool((ref == ref_bits[mask]).all()), "post-fix" if tag == "flex0" else "D8(pre-fix)",
             bool((prod_o == bind_src[mask]).all())))

print("\n== 5. residual pixel tie math (recomputed from flex0-oracle + residual-pixel.json) ==")
rp = json.load(open(os.path.join(OBJ, "artifacts", "residual-pixel.json"), encoding="utf-8"))
o = rp["oracle"]
flat = int(o0["residual_index"][0])
pos = int(np.flatnonzero(np.flatnonzero(flex0) == flat)[0])
edge = int(o0["edge"][pos])
d3 = o0["d3"][pos].astype(np.float64)
print("  residual flat=%d coords=(%d,%d) flexid=%d ulp=%d beyond4=%d"
      % (flat, rp["pixel"]["x"], rp["pixel"]["y"], rp["flexid"], rp["ulps"], rp["beyond4ulp"]))
print("  reference winner edge=%d | d3=%s" % (edge, [float(v) for v in d3]))
print("  recomputed delta=%.15e tol=rel*max(d,1)=%.15e delta<tol=%s"
      % (float(d3[1] - d3[0]), o["ray_tol_rel"] * max(float(d3[0]), 1.0),
         bool(abs(d3[1] - d3[0]) < o["ray_tol_rel"] * max(float(d3[0]), 1.0))))
factor = float(np.float32(rp["product_depth_m"])) / float(d3[1])
print("  depth/d factor=%.9f | d3[1]*factor==product bits %s | d3[0]*factor==ref bits %s"
      % (factor, int(np.float32(d3[1] * factor).view(np.uint32)) == int(pb[flat]),
         int(np.float32(d3[0] * factor).view(np.uint32)) == int(ref_bits[flat])))
print("  json delta_d=%.15e json tol=%.15e tie=%s"
      % (o["delta_d"], o["tie_tolerance"], o["tie"]))
other_within = [i for i in range(len(d3)) if i != edge and abs(d3[i] - d3[edge]) <= o["ray_tol_rel"] * max(float(d3[edge]), 1.0)]
print("  other edges within tol of winner:", other_within)

print("\n== 6. fingerprint / rebuild records (object artifacts) ==")
fp = json.load(open(os.path.join(OBJ, "artifacts", "fingerprint-canonical-rebuild.json"), encoding="utf-8"))
print("  canonical build1 sha256=%s bytes=%d" % (fp["build1"]["sha256"], fp["build1"]["bytes"]))
print("  determinism probe sha256=%s bytes=%d" % (fp["build2_determinism_probe"]["sha256"], fp["build2_determinism_probe"]["bytes"]))
print("  superseded (gate build) sha256=%s" % fp["previous_evidence_value"]["sha256"])
ver = json.load(open(os.path.join(OBJ, "artifacts", "verification-summary.json"), encoding="utf-8"))
print("  summary: ptx=%s bytes=%d source_concat=%s" % (ver["fingerprints"]["product_ptx_sha256"],
                                                      ver["fingerprints"]["product_ptx_bytes"],
                                                      ver["fingerprints"]["product_ptx_source_concat_sha256"]))
print("  summary: test_exe bytes=%d (deprecated as freeze target: %s)"
      % (ver["fingerprints"]["test_exe_bytes"], "yes" if "test_exe_sha256_note" in ver["fingerprints"] else "no"))

print("\n== 7. reviewed scene.cu worktree sha (Backend WIP, not authored by this card) ==")
cu = os.path.join(REPO, "src", "render", "bvh", "cuda", "scene.cu")
print("  src/render/bvh/cuda/scene.cu bytes=%d sha256=%s" % (bytes_of(cu), sha(cu)))

print("\n== 8. verdict inputs ==")
print("mismatched=1 px (%d,%d) ulp=3 beyond4=0 seg_mismatched=0 oracle_flex0_vs_ref=0/403" % (134, 128))
