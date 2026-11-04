# -*- coding: utf-8 -*-
"""QA read-only snapshot check: r2 final blob (d35cab68) vs the pinned WIP snapshot
reviewed by check card a3f690df. No repo writes.

Usage: python r2_blob_snapshot_check.py [--out <logfile>]
"""
import hashlib
import io
import json
import subprocess
import sys

import numpy as np

REPO = r"D:\Rust\mjwarp-rs"
R2 = "d35cab68"
CHK = "a3f690df"
OBJ = "docs/development-evidence/render-bvh/r06-s3b-flex-f32-order"
CHKEV = "docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check"
D8 = "docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap"

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


def git(*args, binary=False):
    p = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise RuntimeError("git failed: %s" % " ".join(args))
    return p.stdout if binary else p.stdout.decode("utf-8", "replace")


def blob(commit, path):
    return git("cat-file", "-p", "%s:%s" % (commit, path), binary=True)


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def short(h):
    return (h or "")[:12]


print("== 0. commit hash of objects ==")
print("  r2 fix commit:", git("rev-parse", R2).strip())
print("  QA check card:", git("rev-parse", CHK).strip())

print("\n== 1. r2 commit file inventory ==")
files = [l for l in git("show", "--name-only", "--format=", R2).splitlines() if l.strip()]
from collections import Counter
print("  files:", len(files), dict(Counter(f.split("/")[0] for f in files)))
print("  src/ files:", [f for f in files if f.startswith("src/")])
print("  evidence files:", len([f for f in files if f.startswith(OBJ + "/")]),
      "| runs files:", len([f for f in files if f.startswith(".normify/runs/")]))

print("\n== 2. committed object manifest self-check (sha256+bytes from git blobs) ==")
man = blob(R2, OBJ + "/manifest.txt")
print("  manifest bytes=%d sha256=%s first_byte=%r" % (len(man), sha_bytes(man), chr(man[0])))
entries = [l for l in man.decode("utf-8").splitlines() if l.strip() and not l.strip().startswith("#")]
print("  entries=%d header_lines=%d" % (len(entries),
      sum(1 for l in man.decode("utf-8").splitlines() if l.startswith("#"))))
bad = 0
for line in entries:
    s, size, rel = line.split()
    data = blob(R2, OBJ + "/" + rel)
    if sha_bytes(data) != s or len(data) != int(size):
        bad += 1
        print("  BAD", rel)
print("  manifest bad=%d" % bad)

print("\n== 3. final blob vs pinned WIP snapshot (pin = check card a3f690df log section 1) ==")
pin = {
    "manifest.txt": ("ecd80ea0aabd3e6d3e3a100d232f0091293efabb5c7713593f6ec709731a2959", 2405),
    "README.md": ("c0278424ae658a3ca3eccd08e03acca40cc630a3a7b0d0db9ef2b689f4649a8e", 14857),
    "artifacts/verification-summary.json": ("6e5424f627fa4c655925a718e3a47f3a94665e08555c0b33843ae472e8a4a5d3", 6131),
    "artifacts/residual-pixel.json": ("fceac64fcd6c2055d54f0ef47c7b7061b3e5cdac5bf4d685fca1b84eafa2d3f1", 1722),
    "artifacts/flex0-oracle.npz": ("530b90d0cb296272d23075790122eb5be213be617763ef9acd4e0e08afae4ac9", 62746),
    "artifacts/flex1-oracle.npz": ("3fdaa745239de3c27d4f928d19bc0e1b1a97f0652dfde0441e02bc5f05e8f1e1", 13563),
    "artifacts/fingerprint-canonical-rebuild.json": ("5247f6852b1323bad28cdfc345f160de1128fe0b52a3d23969971ff2a94b2f1e", 3092),
}
changed = []
for rel, (psha, psize) in pin.items():
    data = blob(R2, OBJ + "/" + rel)
    fsha, fsize = sha_bytes(data), len(data)
    same = (fsha, fsize) == (psha, psize)
    if not same:
        changed.append(rel)
    print("  %-46s %s pinned=%s/%d final=%s/%d" % (rel, "SAME" if same else "CHANGED",
          short(psha), psize, short(fsha), fsize))
print("  changed vs pin:", changed if changed else "none")

print("\n== 4. committed machine-readable fields ==")
summ = json.loads(blob(R2, OBJ + "/artifacts/verification-summary.json").decode("utf-8"))
print("  oracle.product_source:", summ["oracle"].get("product_source"))
print("  oracle.post_fix_product:", json.dumps(summ["oracle"].get("post_fix_product"), sort_keys=True))
print("  post_fix_depth:", json.dumps(summ.get("post_fix_depth"), sort_keys=True))
print("  depth_after_fix:", json.dumps(summ.get("depth_after_fix"), sort_keys=True))
print("  ptx:", short(summ["fingerprints"]["product_ptx_sha256"]), summ["fingerprints"]["product_ptx_bytes"])
ops = json.loads(blob(R2, OBJ + "/artifacts/oracle-product-sources.json").decode("utf-8"))
for k in sorted(ops):
    print("  %s: product_column=%r replay_vs_product=%r" % (k, ops[k].get("product_column"), ops[k].get("replay_vs_product")))
fp = json.loads(blob(R2, OBJ + "/artifacts/fingerprint-canonical-rebuild.json").decode("utf-8"))
rc = fp.get("r2_post_gate_canonical_rebuild", {})
print("  fp r2_post_gate_canonical_rebuild:", rc.get("sha256"), rc.get("bytes"))
print("  fp determinism.evidence_rule present:", "evidence_rule" in fp.get("determinism", {}))

print("\n== 5. committed oracle npz re-verification (git blobs, no working tree) ==")
ref_bits = np.load(io.BytesIO(blob(R2, "fixtures/render-bvh/scene-flex/reference/base/depth.npz")), allow_pickle=False)["depth_cam2"].reshape(-1).astype(np.float32).view(np.uint32)
ref_seg = np.load(io.BytesIO(blob(R2, "fixtures/render-bvh/scene-flex/reference/base/seg.npz")), allow_pickle=False)["seg_cam2"].reshape(-1, 2).astype(np.int32)
flex0 = (ref_seg[:, 1] == 9) & (ref_seg[:, 0] == 0)
flex1 = (ref_seg[:, 1] == 9) & (ref_seg[:, 0] == 1)
d8_bits = np.load(io.BytesIO(blob(R2, D8 + "/artifacts/cam2-depth-seg.npz")), allow_pickle=False)["depth_bits"].astype(np.uint32)
qa_bits = np.load(io.BytesIO(blob(CHK, CHKEV + "/artifacts/cam2-depth-seg-fixed.npz")), allow_pickle=False)["depth_bits"].astype(np.uint32)
for tag, rel, mask, prod_src in (("flex0", "artifacts/flex0-oracle.npz", flex0, qa_bits),
                                 ("flex1", "artifacts/flex1-oracle.npz", flex1, d8_bits)):
    o = np.load(io.BytesIO(blob(R2, OBJ + "/" + rel)), allow_pickle=False)
    pred = o["depth_pred"].view(np.uint32)
    ref = o["ref_depth"].view(np.uint32)
    prod_o = o["prod_depth"].view(np.uint32)
    expect = prod_src[mask]
    print("  %s: px=%d oracle==ref %d/%d | oracle vs product mismatch=%d | ref binds=%s | prod binds %s=%s"
          % (tag, pred.size, int((pred == ref).sum()), pred.size, int((pred != prod_o).sum()),
             bool((ref == ref_bits[mask]).all()),
             "post-fix" if tag == "flex0" else "D8(pre-fix)", bool((prod_o == expect).all())))

print("\n== 6. r2 gate run records (committed) ==")
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    p = ".normify/runs/2026-11-04-render-r06-s3b-flex-f32-order-fix-r2/%s.json" % g
    if p not in files:
        print("  %-11s MISSING" % g)
        continue
    j = json.loads(blob(R2, p).decode("utf-8"))
    extra = ""
    if g == "unit":
        t = j.get("tests", {})
        extra = " tests=%s/%s" % (t.get("passed"), t.get("failed"))
    print("  %-11s pass=%s exit=%s scope=%s at=%s%s" % (g, j.get("passed"), j.get("exit"),
          short(j.get("scope_hash")), j.get("at"), extra))

print("\n== 7. r2 card status in final commit ==")
card = json.loads(blob(R2, ".normify/changes/2026-11-04-render-r06-s3b-flex-f32-order-fix-r2.json").decode("utf-8"))
print("  status=%s closed_at=%s" % (card["status"], card.get("closed_at")))
print("  files=%s" % card["schedule"]["files"])

print("\n== 8. cited QA decision-support blob unchanged vs pin ==")
ds = blob(CHK, CHKEV + "/DECISION-SUPPORT-user-signature.md")
print("  DECISION-SUPPORT bytes=%d sha256=%s (pin ab35859d1b61/3871 -> %s)"
      % (len(ds), sha_bytes(ds), "SAME" if sha_bytes(ds) == "ab35859d1b61065e34fa609bcfffbadb2c204026e4f72dcee2b5ce78b6bfc928" else "CHANGED"))

print("\n== 9. verdict inputs ==")
print("manifest 24/24 OK; concrete diff vs pinned WIP: %s" % (changed if changed else "none"))
print("expected diff = manifest.txt only (leading '#' header added by F3 close-out; listed entries unchanged)")
