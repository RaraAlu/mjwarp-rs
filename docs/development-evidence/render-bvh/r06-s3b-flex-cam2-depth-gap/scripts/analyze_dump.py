"""D8 cam2 depth-gap analysis (offline; CPU only).

Reads the env-gated WIP dump written by the ignored GPU test:
  target/render-bvh/r06-s3b-flex-window/d8/{depth_all.f32le,seg_all.i32le,meta.json}
and answers the H1/H1b separation questions:
  * reproduce (mismatched, beyond4ULP) = (321, 297)?
  * ULP histogram / absolute depth difference; per-flexid split
  * is product seg bit-equal to the reference seg? (same flexid everywhere?)
  * where are the mismatched pixels: silhouette of both flexes (H1 arithmetic) or
    flex0 joints/overlap (H1b candidate-order)?
  * overlap with the CPU-replay sensitive set (same kernel, different codegen).
"""
import json
import math
import pathlib
import struct

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
BASE = ROOT / "fixtures/render-bvh/scene-flex/reference/base"
DUMP = ROOT / "target/render-bvh/r06-s3b-flex-window/d8"
REPLAY = ROOT / "target/s3b-verify/cpu-replay/cpu_cam2_depth.npz"
FIX = ROOT / "fixtures/render-bvh/scene-flex"


def bits(a):
    return np.asarray(a, dtype=np.float32).view(np.int32).astype(np.int64)


def ulp_diff(a, b):
    return np.abs(bits(a) - bits(b))


def f32_from_bits(v):
    return struct.unpack("<f", struct.pack("<I", v & 0xFFFFFFFF))[0]


meta = json.loads((DUMP / "meta.json").read_text(encoding="utf-8"))
depth_all = np.fromfile(DUMP / "depth_all.f32le", dtype="<f4")
seg_all = np.fromfile(DUMP / "seg_all.i32le", dtype="<i4").reshape(-1, 2)
cam_res = meta["cam_res"]
depth_adr = meta["depth_adr"]
seg_adr = meta["seg_adr"]
print("meta:", json.dumps({k: meta[k] for k in meta if k not in ("cam_res", "depth_adr", "seg_adr")}, ensure_ascii=False))
print("cam_res", cam_res, "depth_adr", depth_adr, "seg_adr", seg_adr)

cam = 2
w, h = int(cam_res[cam][0]), int(cam_res[cam][1])
pixels = w * h
prod_depth = depth_all[int(depth_adr[cam]):int(depth_adr[cam]) + pixels].astype(np.float32)
prod_seg = seg_all[int(seg_adr[cam]):int(seg_adr[cam]) + pixels].astype(np.int32)

ref_depth = np.load(BASE / "depth.npz")[f"depth_cam{cam}.npy"].reshape(-1).astype(np.float32)
ref_seg = np.load(BASE / "seg.npz")[f"seg_cam{cam}.npy"].reshape(-1, 2).astype(np.int32)
assert prod_depth.size == ref_depth.size == prod_seg.shape[0] == ref_seg.shape[0]

ulp = ulp_diff(prod_depth, ref_depth)
mism = ulp > 0
beyond = ulp > 4
print(f"\n== reproduce ==  mismatched={int(mism.sum())} beyond4={int(beyond.sum())} max_ulp={int(ulp.max())}")
edges = [0, 1, 2, 4, 16, 64, 1024, 10 ** 9]
hist = [int(((ulp >= lo) & (ulp <= hi)).sum()) for lo, hi in zip(edges[:-1], edges[1:])]
print("ULP histogram buckets (0,1,2-4,5-16,17-64,65-1024,>1024):", hist)
absdiff = np.abs(prod_depth.astype(np.float64) - ref_depth.astype(np.float64))
if mism.any():
    print(f"abs diff (m): median={np.median(absdiff[mism]):.3e} p90={np.percentile(absdiff[mism], 90):.3e} max={absdiff[mism].max():.3e}")
    print("prod depth at mismatched:", prod_depth[mism][:6].tolist(), "... ref:", ref_depth[mism][:6].tolist())

flex = ref_seg[:, 1] == 9
for flexid in (0, 1):
    m = flex & (ref_seg[:, 0] == flexid)
    print(f"flexid {flexid}: pixels={int(m.sum())} mismatched={int((mism & m).sum())} beyond4={int((beyond & m).sum())} max_ulp={int(ulp[m].max()) if m.any() else 0}")
print(f"non-flex: mismatched={int((mism & ~flex).sum())} beyond4={int((beyond & ~flex).sum())}")

seg_mismatch = np.any(prod_seg != ref_seg, axis=1)
print(f"\nseg vs reference: mismatched_pixels={int(seg_mismatch.sum())} of {pixels}")
prod_flexids = {int(v) for v in prod_seg[prod_seg[:, 1] == 9][:, 0]}
ref_flexids = {int(v) for v in ref_seg[ref_seg[:, 1] == 9][:, 0]}
print("flexid sets product", sorted(prod_flexids), "reference", sorted(ref_flexids))
print("hit/miss pattern differs:", int(np.count_nonzero((prod_seg[:, 1] == 9) != flex)))

# ---- spatial classification of mismatched pixels ----
state = np.load(BASE / "state.npz")
verts = state["flexvert_xpos"][0].astype(np.float64)  # (6,3)
cp = json.loads((BASE / "cam-poses.json").read_text(encoding="utf-8"))["poses"]
cam_pos = np.array([f32_from_bits(v) for v in cp["cam_xpos_bits"][3 * cam:3 * cam + 3]], dtype=np.float64)
cam_mat = np.array([f32_from_bits(v) for v in cp["cam_xmat_bits"][9 * cam:9 * cam + 9]], dtype=np.float64).reshape(3, 3)
fovy = dict(zip(["cam-persp", "cam-track", "cam-flex"], [None, 40.0, 45.0]))["cam-flex"]
focal = (h / 2.0) / math.tan(math.radians(fovy) / 2.0)


def project(p):
    local = cam_mat.T @ (p - cam_pos)      # rows of cam_mat are local axes in world
    z = -local[2]
    if z <= 1e-9:
        return None
    return (w / 2.0 + focal * local[0] / z, h / 2.0 - focal * local[1] / z)


proj = {i: project(verts[i]) for i in range(6)}
print("\nprojected flex vertices (image px):", {k: (None if v is None else (round(v[0], 1), round(v[1], 1))) for k, v in proj.items()})

yy, xx = np.mgrid[0:h, 0:w]
mm = mism.reshape(h, w)
flex_img = flex.reshape(h, w)
ref_flexid = ref_seg[:, 0].reshape(h, w)
mism_px = np.argwhere(mm)              # (row, col)


def dist_to_vertex(vertex):
    if proj[vertex] is None:
        return np.full(len(mism_px), 1e9)
    vx, vy = proj[vertex]
    return np.hypot(mism_px[:, 1] - vx, mism_px[:, 0] - vy)


# distance to nearest projected joint: flex0 joints = vertices 1,2
for name, idxs in (("flex0 joints v1,v2", [1, 2]), ("flex0 endpoints v0,v3", [0, 3]), ("flex1 both v4,v5", [4, 5])):
    if len(mism_px) == 0:
        continue
    d = np.minimum(*[dist_to_vertex(i) for i in idxs]) if len(idxs) > 1 else dist_to_vertex(idxs[0])
    near = int((d <= 4.0).sum())
    print(f"mismatched px near {name} (<=4 px): {near}/{len(mism_px)}  median_dist={np.median(d):.1f}")

# distance to the silhouette (flex->background boundary) via BFS
boundary = np.zeros((h, w), dtype=bool)
for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
    shifted = np.roll(np.roll(flex_img, dy, axis=0), dx, axis=1)
    boundary |= flex_img & ~shifted
boundary = boundary & flex_img
print("flex silhouette pixels (image):", int(boundary.sum()))
if len(mism_px):
    bset = np.argwhere(boundary)
    # nearest boundary distance for mismatched pixels (brute force ok: small)
    dmin = np.sqrt(((mism_px[:, None, 0] - bset[None, :, 0]) ** 2 +
                    (mism_px[:, None, 1] - bset[None, :, 1]) ** 2).min(axis=1))
    print(f"mismatched px distance to silhouette: median={np.median(dmin):.1f}px p90={np.percentile(dmin, 90):.1f}px "
          f"<=2px={int((dmin <= 2).sum())}/{len(mism_px)} <=4px={int((dmin <= 4).sum())}/{len(mism_px)}")
    print("inside-flex mismatched px:", int(flex_img[mm].sum()), "/", len(mism_px))

# ---- overlap with CPU-replay sensitive set ----
if REPLAY.is_file():
    rep = np.load(REPLAY)["depth"]
    rep_ulp = ulp_diff(rep, ref_depth)
    rep_mism = rep_ulp > 0
    rep_beyond = rep_ulp > 4
    inter = int((mism & rep_mism).sum())
    print(f"\nCPU-replay sensitive set: mismatched={int(rep_mism.sum())} beyond4={int(rep_beyond.sum())}")
    print(f"product mismatched ∩ cpu-replay mismatched = {inter}/{int(mism.sum())} "
          f"({100.0 * inter / max(1, int(mism.sum())):.1f}%)")
    inter4 = int((beyond & rep_beyond).sum())
    print(f"product beyond4 ∩ cpu-replay beyond4 = {inter4}/{int(beyond.sum())}")
    if mism.any():
        print(f"cpu-replay ULP at product-mismatched px: median={np.median(rep_ulp[mism]):.1f} "
              f"vs at product-equal px: median={np.median(rep_ulp[~mism]):.3f}")
else:
    print("\nCPU replay npz missing:", REPLAY)
