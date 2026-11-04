"""Offline scope check for the proposed kernel-only fix (heap -> warp LIFO traversal).

Uses the cam2 per-pixel flex internals dump (all 4 flex slots' capsule distances) plus the
offline BVH reconstruction from `rca.py` to answer:
  1. how many cam2 flex pixels have two flex candidates within RAY_TOL_REL (order matters),
  2. for each of them, which edge the current heap order picks vs the warp LIFO order,
  3. whether the LIFO prediction matches the frozen reference depth bits.
"""
import sys
import json
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
import rca  # noqa: E402  (builds the two trees, prints its own report)

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order"
F = np.float32

allslots = np.frombuffer((OUT / "cam2-flex-internals-allslots.f32le").read_bytes(), dtype="<f4").reshape(-1, 4, 30)
pixel_indices = np.frombuffer((OUT / "cam2-flex-pixel-indices.i32le").read_bytes(), dtype="<i4")
rays = np.frombuffer((OUT / "cam2-ray.f32le").read_bytes(), dtype="<f4").reshape(-1, 6)
depth = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].reshape(-1).astype(F)

SLOTS = 4
D_FIELD = 29
N = len(pixel_indices)
print(f"\n=== cam2 flex pixels: {N}, allslots shape {allslots.shape} ===")

# leaf id for slot s = 4 + s (leaf order: 4 geoms then 4 flex leaves)
def simulate(order, ds, max_cam_dist):
    hit_dist = max_cam_dist
    hit_geom = -1
    winner = -1
    for leaf in order:
        slot = leaf - 4
        if slot < 0:
            continue  # rigid candidates are ignored in this flex-only simulation
        d = ds[slot]
        if not (d > 0.0):
            continue
        tol = F(F(1.0e-6) * (hit_dist if hit_dist > 1.0 else F(1.0)))
        within = abs(d - hit_dist) <= tol
        closer = (d < hit_dist - tol) or (within and (hit_geom < 0 or F(-2) > F(hit_geom)))
        if closer:
            hit_dist = d
            hit_geom = -2
            winner = slot
    return winner, hit_dist


def candidate_order(order_fn, origin, direction, t_limit):
    return order_fn(origin, direction, t_limit)


def lifo_order(root, origin, direction, t_limit):
    stack = [root]
    order = []
    while stack:
        node = stack.pop()
        b = rca.pbounds(node)
        if rca.product_ray_aabb(origin, direction, b, t_limit) is None:
            continue
        if node[0] == "node":
            stack.append(node[1])
            stack.append(node[2])  # right popped first == warp stack discipline
        else:
            order.append(node[1])
    return order


def heap_order_fn(node, origin, direction, t_limit):
    return rca.product_order(rca.product_root, origin, direction, t_limit)


cam_poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
poses = cam_poses["poses"]
f32of = lambda b: np.uint32(b).view(np.float32)  # noqa: E731
cam_xpos = np.array([float(f32of(b)) for b in poses["cam_xpos_bits"][6:9]], dtype=F)
cam_xmat = np.array([float(f32of(b)) for b in poses["cam_xmat_bits"][18:27]], dtype=F).reshape(3, 3)


def world_dir(local):
    return np.array(
        [
            F(F(cam_xmat[0, 0] * local[0]) + F(F(cam_xmat[0, 1] * local[1]) + F(cam_xmat[0, 2] * local[2]))),
            F(F(cam_xmat[1, 0] * local[0]) + F(F(cam_xmat[1, 1] * local[1]) + F(cam_xmat[1, 2] * local[2]))),
            F(F(cam_xmat[2, 0] * local[0]) + F(F(cam_xmat[2, 1] * local[1]) + F(cam_xmat[2, 2] * local[2]))),
        ],
        dtype=F,
    )


ties = []
mismatch_before = 0
mismatch_after = 0
changed = []
for row in range(N):
    px = int(pixel_indices[row])
    ds = allslots[row, :, D_FIELD]
    winners = [s for s in range(SLOTS) if ds[s] > 0.0]
    if len(winners) < 2:
        continue
    best = min(float(ds[s]) for s in winners)
    tol = 1.0e-6 * max(best, 1.0)
    pairs = [
        (a, b)
        for i, a in enumerate(winners)
        for b in winners[i + 1:]
        if abs(float(ds[a]) - float(ds[b])) <= tol
    ]
    if not pairs:
        continue
    local = rays[px, :3].astype(F)
    dirw = world_dir(local)
    max_cam_dist = F(125.0) / max(F(-local[2]), F(1.0e-6))
    t_limit = F(F(max_cam_dist * F(F(1.0) + F(1.0e-6))) + F(1.0e-9))
    order_heap = heap_order_fn(rca.product_root, cam_xpos, dirw, t_limit)
    order_lifo = lifo_order(rca.product_root, cam_xpos, dirw, t_limit)
    win_heap, dist_heap = simulate(order_heap, ds, max_cam_dist)
    win_lifo, dist_lifo = simulate(order_lifo, ds, max_cam_dist)

    # reference winner: match depth bits against f32(d * -local_z)
    ref_bits = int(np.frombuffer(depth[px:px + 1].tobytes(), dtype="<u4")[0])
    ref_winner = -1
    for s in winners:
        pred = F(F(ds[s]) * F(-local[2]))
        pred_bits = int(np.frombuffer(pred.tobytes(), dtype="<u4")[0])
        if pred_bits == ref_bits:
            ref_winner = s
    tie = {
        "pixel": px,
        "x": px % 320,
        "y": px // 320,
        "d": [float(ds[s]) for s in range(SLOTS)],
        "pairs": pairs,
        "order_heap": order_heap,
        "order_lifo": order_lifo,
        "winner_heap": win_heap,
        "winner_lifo": win_lifo,
        "ref_winner": ref_winner,
        "ref_bits": hex(ref_bits),
        "pred_bits_heap": hex(int(np.frombuffer(np.float32(F(dist_heap * F(-local[2]))).tobytes(), dtype="<u4")[0])),
        "pred_bits_lifo": hex(int(np.frombuffer(np.float32(F(dist_lifo * F(-local[2]))).tobytes(), dtype="<u4")[0])),
    }
    ties.append(tie)
    if win_heap != ref_winner:
        mismatch_before += 1
    if win_lifo != ref_winner:
        mismatch_after += 1
    if win_heap != win_lifo:
        changed.append((px, win_heap, win_lifo, ref_winner))

print(f"\n=== tie pixels (>=2 flex candidates within RAY_TOL_REL): {len(ties)} ===")
for tie in ties:
    print(json.dumps(tie, ensure_ascii=False))
print(f"\nheap order vs reference winner : {mismatch_before} mismatched")
print(f"LIFO order vs reference winner : {mismatch_after} mismatched")
print(f"pixels whose predicted winner changes: {changed}")
