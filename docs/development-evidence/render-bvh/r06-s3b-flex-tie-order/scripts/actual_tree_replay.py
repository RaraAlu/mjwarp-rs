"""Exact replay on the ACTUAL product scene BVH (dumped from product code via a CPU-only test),
to decide whether the LIFO traversal change can fix pixel 41094 and what the min-d tie rule does.

Tree (node index -> kind), from `dump_flex_scene_bvh_for_rca` (product `build_scene_bvh` output):
  leaf nodes: 0->leaf0, 1->leaf5, 2->leaf4, 3->leaf6, 6->leaf7, 8->leaf1, 9->leaf2, 10->leaf3
  internal : 4=(2,3), 5=(1,4), 7=(5,6), 11=(9,10), 12=(8,11), 13=(7,12), 14=(0,13)
"""
import json
from pathlib import Path

import numpy as np

F = np.float32
ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order"

nodes = {
    0: ("leaf", 0), 1: ("leaf", 5), 2: ("leaf", 4), 3: ("leaf", 6),
    4: ("int", 2, 3), 5: ("int", 1, 4), 6: ("leaf", 7),
    7: ("int", 5, 6), 8: ("leaf", 1), 9: ("leaf", 2), 10: ("leaf", 3),
    11: ("int", 9, 10), 12: ("int", 8, 11), 13: ("int", 7, 12), 14: ("int", 0, 13),
}
lo_bits = {
    0: ([3229635052, 3229635052, 3156465418], [1082151404, 1082151404, 1008981770]),
    1: ([3180052182, 3201341685, 1045423006], [1052672600, 1005602560, 1055737361]),
    2: ([3162969216, 3191047280, 1053794984], [1054122699, 1044371270, 1063156012]),
    3: ([1043567006, 3184061182, 1056232568], [1052689851, 1046184949, 1059194982]),
    4: ([3214262927, 1058306786, 1056293519], [3212501320, 1059313418, 1057300152]),
    5: ([3214262927, 1058642330, 1056293519], [3213927383, 1059313418, 1060320051]),
    6: ([3214262927, 1058306786, 1059648963], [3212501320, 1059313418, 1060320051]),
    7: ([3211830231, 1063004405, 1050589266], [3208810332, 1065185444, 1054951342]),
}


def fb(bits):
    return np.uint32(bits).view(np.float32)


leaf_bounds = {
    k: (np.array([fb(b) for b in v[0]], dtype=F), np.array([fb(b) for b in v[1]], dtype=F))
    for k, v in lo_bits.items()
}
int_bounds = {}
for idx in range(15):
    kind = nodes[idx]
    if kind[0] != "int":
        continue
    a = leaf_bounds[nodes[kind[1]][1]] if nodes[kind[1]][0] == "leaf" else int_bounds[kind[1]]
    b = leaf_bounds[nodes[kind[2]][1]] if nodes[kind[2]][0] == "leaf" else int_bounds[kind[2]]
    int_bounds[idx] = (
        np.array([min(a[0][t], b[0][t]) for t in range(3)], dtype=F),
        np.array([max(a[1][t], b[1][t]) for t in range(3)], dtype=F),
    )


def bounds(i):
    return leaf_bounds[nodes[i][1]] if nodes[i][0] == "leaf" else int_bounds[i]


def ray_aabb(origin, direction, b, t_limit):
    t0 = F(0.0)
    t1 = F(t_limit)
    for ax in range(3):
        o, d, lo, hi = F(origin[ax]), F(direction[ax]), F(b[0][ax]), F(b[1][ax])
        if d == 0.0:
            if o < lo or o > hi:
                return None
            continue
        near, far = F(F(lo - o) / d), F(F(hi - o) / d)
        if near > far:
            near, far = far, near
        t0 = near if near > t0 else t0
        t1 = far if far < t1 else t1
        if t0 > t1:
            return None
    return t0


def lifo(origin, direction, t_limit):
    stack = [14]
    order = []
    while stack:
        i = stack.pop()
        if ray_aabb(origin, direction, bounds(i), t_limit) is None:
            continue
        kind = nodes[i]
        if kind[0] == "int":
            stack.append(kind[1])
            stack.append(kind[2])
        else:
            order.append(kind[1])
    return order


class Heap:
    def __init__(self):
        self.items = []

    def push(self, entry, node):
        it = self.items
        it.append((entry, node))
        idx = len(it) - 1
        while idx > 0:
            parent = (idx - 1) // 2
            if it[parent][0] <= entry:
                break
            it[idx] = it[parent]
            idx = parent
        it[idx] = (entry, node)

    def pop(self):
        it = self.items
        top = it[0]
        count = len(it) - 1
        if count > 0:
            last = it[count]
            idx = 0
            while True:
                left = 2 * idx + 1
                if left >= count:
                    break
                right = left + 1
                child = right if (right < count and it[right][0] < it[left][0]) else left
                if it[child][0] >= last[0]:
                    break
                it[idx] = it[child]
                idx = child
            it[idx] = last
        it.pop()
        return top


def heap_order(origin, direction, t_limit):
    heap = Heap()
    entry = ray_aabb(origin, direction, bounds(14), t_limit)
    if entry is None:
        return []
    heap.push(entry, 14)
    order = []
    while heap.items:
        entry, i = heap.pop()
        if entry > t_limit:
            break
        kind = nodes[i]
        if kind[0] == "int":
            for child in (kind[1], kind[2]):
                ce = ray_aabb(origin, direction, bounds(child), t_limit)
                if ce is not None:
                    heap.push(ce, child)
        else:
            order.append(kind[1])
    return order


def winner(order, ds, t_limit, rule):
    hit_dist = float(t_limit)
    hit_geom = -1
    win = -1
    for leaf in order:
        slot = leaf - 4
        if slot < 0:
            continue
        d = float(ds[slot])
        if not d > 0.0:
            continue
        tol = 1.0e-6 * max(hit_dist, 1.0)
        if rule == "last":
            closer = (d < hit_dist - tol) or (abs(d - hit_dist) <= tol and hit_geom < 0)
        elif rule == "min-d-flex":
            flex_prev = hit_geom == -2
            closer = (d < hit_dist) if flex_prev else (d < hit_dist - tol)
        else:
            raise SystemExit(rule)
        if closer:
            hit_dist = d
            hit_geom = -2
            win = slot
    return win, hit_dist


alls = np.frombuffer((OUT / "cam2-flex-internals-allslots.f32le").read_bytes(), dtype="<f4").reshape(-1, 4, 30)
pix = np.frombuffer((OUT / "cam2-flex-pixel-indices.i32le").read_bytes(), dtype="<i4")
rays = np.frombuffer((OUT / "cam2-ray.f32le").read_bytes(), dtype="<f4").reshape(-1, 6)
depth_ref = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].reshape(-1).astype(F)
poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))["poses"]
f32of = lambda b: np.uint32(b).view(np.float32)  # noqa: E731
cam_xpos = np.array([float(f32of(b)) for b in poses["cam_xpos_bits"][6:9]], dtype=F)
cam_xmat = np.array([float(f32of(b)) for b in poses["cam_xmat_bits"][18:27]], dtype=F).reshape(3, 3)


def world_dir(local):
    return np.array(
        [F(F(cam_xmat[i, 0] * local[0]) + F(F(cam_xmat[i, 1] * local[1]) + F(cam_xmat[i, 2] * local[2]))) for i in range(3)],
        dtype=F,
    )


def ref_bits(px):
    return int(np.frombuffer(depth_ref[px:px + 1].tobytes(), dtype="<u4")[0])


print("\n=== exact replay on ACTUAL product tree ===")
for px in (41094, 31814):
    row = int(np.where(pix == px)[0][0])
    ds = alls[row, :, 29].astype(F)
    loc = rays[px, :3].astype(F)
    dirw = world_dir(loc)
    max_cam = F(125.0) / max(F(-loc[2]), F(1.0e-6))
    t_lim = float(F(F(max_cam * F(F(1.0) + F(1.0e-6))) + F(1.0e-9)))
    ol = lifo(cam_xpos, dirw, t_lim)
    oh = heap_order(cam_xpos, dirw, t_lim)
    print(f"\npixel {px}  slots d={[float(ds[s]) for s in range(4)]}")
    print("  LIFO order:", ol)
    print("  heap order:", oh)
    for name, order in (("LIFO", ol), ("heap", oh)):
        wl, dl = winner(order, ds, t_lim, "last")
        wm, dm = winner(order, ds, t_lim, "min-d-flex")
        pred_last = int(np.frombuffer(np.float32(F(dl) * F(-loc[2])).tobytes(), dtype="<u4")[0])
        pred_min = int(np.frombuffer(np.float32(F(dm) * F(-loc[2])).tobytes(), dtype="<u4")[0])
        print(f"  {name}: last-visit winner=slot{wl} bits=0x{pred_last:08x}   min-d winner=slot{wm} bits=0x{pred_min:08x}   ref=0x{ref_bits(px):08x}")
