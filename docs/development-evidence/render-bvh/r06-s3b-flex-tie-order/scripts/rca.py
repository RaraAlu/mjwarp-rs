"""Offline (CPU-only) reconstruction of the scene BVH for fixture `scene-flex`, used to
explain why the reference visits flex-0 edge0 last but the product visits edge1 last.

Inputs (all read-only): fixtures/render-bvh/scene-flex/{model-312.json, reference/base/state.npz}.
Builders replicated:
  * upstream: warp 1.15 host SAH (16 buckets, longest centroid axis, `bvh.cpp`)
  * product : mjwarp-rs `src/render/bvh/builder.rs` (12 bins, per-axis cost)
Traversals replicated:
  * upstream: warp `bvh_query_next` LIFO stack, right child popped first (`bvh.h`)
  * product : `render_common.cu` min-heap ordered by AABB entry distance
Only the *order* is compared (candidate pruning is irrelevant for the relative order of
two leaves whose AABBs both intersect the ray).
"""
import json
import math
from pathlib import Path

import numpy as np

F = np.float32
ROOT = Path(r"D:\Rust\mjwarp-rs")
MODEL = json.loads((ROOT / "fixtures/render-bvh/scene-flex/model-312.json").read_text(encoding="utf-8"))
STATE = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/state.npz")
CAPTURE = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/capture.json").read_text(encoding="utf-8"))

SAH_BUCKETS_WARP = 16
SAH_BINS_PRODUCT = 12
LEAF_SIZE = 1


def f32(x):
    return F(x)


def bits(x):
    return np.float32(x).view(np.uint32)


# ---------------------------------------------------------------- leaf bounds
def m(name):
    return MODEL["arrays"][name]["values"]


geom_type = [int(v) for v in m("geom_type")]
geom_dataid = [int(v) for v in m("geom_dataid")]
geom_group = [int(v) for v in m("geom_group")]
geom_size = np.array(m("geom_size"), dtype=F).reshape(-1, 3)
mesh_vert = np.array(m("mesh_vert"), dtype=F)
flex_radius = np.array(m("flex_radius"), dtype=F)
# `flex_edge` is a derived array (not dumped in model-312.json); the capture records the
# resolved values authored from the reference model.
flex_edge = [int(v) for v in CAPTURE["flex"]["flex_edge"]]
flex_vertadr = [int(v) for v in CAPTURE["flex"]["flex_vertadr"]]
print("flex_radius", flex_radius.tolist(), "flex_edge", flex_edge, "flex_vertadr", flex_vertadr)

geom_xpos = STATE["geom_xpos"][0].astype(F)
geom_xmat = STATE["geom_xmat"][0].astype(F)
flexvert_xpos = STATE["flexvert_xpos"][0].astype(F)

enabled = [g for g in range(len(geom_type)) if geom_group[g] in (0, 1, 2, 3)]
print("ngeom", len(geom_type), "enabled geom ids", enabled, "types", [geom_type[g] for g in enabled])
print("geom_size", geom_size.tolist())

MJ_MAXVAL = f32(1.0e10)


def bounds3(lower, upper):
    return (np.array(lower, dtype=F), np.array(upper, dtype=F))


def std_min(a, b):
    return b if (b < a) else a


def std_max(a, b):
    return b if (b > a) else a


def b_union(a, b):
    lo = np.array([std_min(a[0][i], b[0][i]) for i in range(3)], dtype=F)
    hi = np.array([std_max(a[1][i], b[1][i]) for i in range(3)], dtype=F)
    return (lo, hi)


def b_add_point(a, p):
    lo = np.array([std_min(a[0][i], p[i]) for i in range(3)], dtype=F)
    hi = np.array([std_max(a[1][i], p[i]) for i in range(3)], dtype=F)
    return (lo, hi)


def b_center(b):
    return (F(0.5) * (b[0] + b[1])).astype(F)


def b_edges(b):
    return (b[1] - b[0]).astype(F)


def b_area(b):
    e = b_edges(b)
    t = F(e[0] * e[1])
    t = F(t + F(e[0] * e[2]))
    t = F(t + F(e[1] * e[2]))
    return F(2.0) * t


def longest_axis(v):
    # warp vec.h: abs compare, first maximum wins (strict >)
    lmax = abs(F(v[0]))
    ret = 0
    for i in (1, 2):
        l = abs(F(v[i]))
        if l > lmax:
            ret = i
            lmax = l
    return ret


def box_bounds(pos, rot, size):
    lo = np.array([MJ_MAXVAL] * 3, dtype=F)
    hi = np.array([-MJ_MAXVAL] * 3, dtype=F)
    for i in range(2):
        for j in range(2):
            for k in range(2):
                local = np.array(
                    [
                        F(size[0]) * F(2.0 * i - 1.0),
                        F(size[1]) * F(2.0 * j - 1.0),
                        F(size[2]) * F(2.0 * k - 1.0),
                    ],
                    dtype=F,
                )
                world = np.array(
                    [
                        F(
                            pos[0]
                            + F(
                                F(rot[0, 0] * local[0])
                                + F(F(rot[0, 1] * local[1]) + F(rot[0, 2] * local[2]))
                            )
                        ),
                        F(
                            pos[1]
                            + F(
                                F(rot[1, 0] * local[0])
                                + F(F(rot[1, 1] * local[1]) + F(rot[1, 2] * local[2]))
                            )
                        ),
                        F(
                            pos[2]
                            + F(
                                F(rot[2, 0] * local[0])
                                + F(F(rot[2, 1] * local[1]) + F(rot[2, 2] * local[2]))
                            )
                        ),
                    ],
                    dtype=F,
                )
                lo = np.array([std_min(lo[z], world[z]) for z in range(3)], dtype=F)
                hi = np.array([std_max(hi[z], world[z]) for z in range(3)], dtype=F)
    return (lo, hi)


def sphere_bounds(pos, rot, size):
    r = F(size[0])
    inflate = np.array([r, r, r], dtype=F)
    return (np.array([pos[0] - r, pos[1] - r, pos[2] - r], dtype=F),
            np.array([pos[0] + r, pos[1] + r, pos[2] + r], dtype=F))


def plane_bounds(pos, rot, size):
    size_scale = F(std_max(F(size[0]), F(size[1])) * F(2.0))
    if size[0] <= 0.0 or size[1] <= 0.0:
        size_scale = F(1000.0)
    lo = np.array([MJ_MAXVAL] * 3, dtype=F)
    hi = np.array([-MJ_MAXVAL] * 3, dtype=F)
    for i in range(2):
        for j in range(2):
            local = np.array(
                [F(size_scale * F(2.0 * i - 1.0)), F(size_scale * F(2.0 * j - 1.0)), F(0.0)], dtype=F
            )
            world = np.array(
                [
                    F(pos[0] + F(F(rot[0, 0] * local[0]) + F(F(rot[0, 1] * local[1]) + F(rot[0, 2] * local[2])))),
                    F(pos[1] + F(F(rot[1, 0] * local[0]) + F(F(rot[1, 1] * local[1]) + F(rot[1, 2] * local[2])))),
                    F(pos[2] + F(F(rot[2, 0] * local[0]) + F(F(rot[2, 1] * local[1]) + F(rot[2, 2] * local[2])))),
                ],
                dtype=F,
            )
            lo = np.array([std_min(lo[z], world[z]) for z in range(3)], dtype=F)
            hi = np.array([std_max(hi[z], world[z]) for z in range(3)], dtype=F)
    lo = lo - np.array([0.01, 0.01, 0.01], dtype=F)
    hi = hi + np.array([0.01, 0.01, 0.01], dtype=F)
    return (lo, hi)


def capsule_bounds(pos, rot, size):
    radius = F(size[0])
    half_length = F(size[1])
    z = np.array([rot[0, 2], rot[1, 2], rot[2, 2]], dtype=F)
    e1 = np.array([pos[0] - z[0] * half_length, pos[1] - z[1] * half_length, pos[2] - z[2] * half_length], dtype=F)
    e2 = np.array([pos[0] + z[0] * half_length, pos[1] + z[1] * half_length, pos[2] + z[2] * half_length], dtype=F)
    seg_min = np.array([min(e1[i], e2[i]) for i in range(3)], dtype=F)
    seg_max = np.array([max(e1[i], e2[i]) for i in range(3)], dtype=F)
    inflate = np.array([radius, radius, radius], dtype=F)
    return (seg_min - inflate, seg_max + inflate)


def mesh_half_extents_upstream():
    # build_mesh_bvh: half computed from the raw (f64 in MuJoCo) mesh vertices, then cast to f32
    vstart = int(m("mesh_vertadr")[0])
    vnum = int(m("mesh_vertnum")[0])
    verts = np.array(m("mesh_vert")[3 * vstart:3 * (vstart + vnum)], dtype=np.float64).reshape(-1, 3)
    pmin = verts.min(axis=0)
    pmax = verts.max(axis=0)
    half = np.maximum(np.abs(pmin), np.abs(pmax))
    return np.array([f32(half[0]), f32(half[1]), f32(half[2])], dtype=F)


mesh_half = mesh_half_extents_upstream()
print("mesh_bounds_size (upstream f64->f32)", mesh_half.tolist(), "bits", [hex(int(bits(v))) for v in mesh_half])


def compute_leaf_bounds(geom_id):
    t = geom_type[geom_id]
    pos = geom_xpos[geom_id]
    rot = geom_xmat[geom_id]
    size = geom_size[geom_id]
    if t == 0:  # plane
        return plane_bounds(pos, rot, size)
    if t == 6:  # box
        return box_bounds(pos, rot, size)
    if t == 7:  # mesh -> box of mesh half extents
        return box_bounds(pos, rot, mesh_half)
    if t == 3:  # capsule
        return capsule_bounds(pos, rot, size)
    raise SystemExit(f"unhandled geom type {t}")


leaf_bounds = []
leaf_desc = []
for g in enabled:
    leaf_bounds.append(compute_leaf_bounds(g))
    leaf_desc.append(f"geom{g}(type {geom_type[g]})")

# flex leaves: flex_geom_flexid = [0,0,0,1], flex_geom_edgeid = [0,1,2,3]
flex_geom_flexid = [0, 0, 0, 1]
flex_geom_edgeid = [0, 1, 2, 3]
for flexid, edgeid in zip(flex_geom_flexid, flex_geom_edgeid):
    adr = flex_vertadr[flexid]
    a, b = flex_edge[2 * edgeid], flex_edge[2 * edgeid + 1]
    v0 = flexvert_xpos[adr + a]
    v1 = flexvert_xpos[adr + b]
    radius = flex_radius[flexid]
    lo = np.array([min(F(v0[i]), F(v1[i])) - radius for i in range(3)], dtype=F)
    hi = np.array([max(F(v0[i]), F(v1[i])) + radius for i in range(3)], dtype=F)
    leaf_bounds.append((lo, hi))
    leaf_desc.append(f"flex{flexid}edge{edgeid}")

N = len(leaf_bounds)
print(f"\n{N} leaves")
for i, (b, d) in enumerate(zip(leaf_bounds, leaf_desc)):
    print(f"  leaf {i} {d:12s} lo={[float(x) for x in b[0]]} hi={[float(x) for x in b[1]]}")
print("flex leaf ids:", [i for i, d in enumerate(leaf_desc) if d.startswith("flex")])

# ------------------------------------------------------------ warp SAH builder
def calc_bounds(subset):
    b = (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
    for it in subset:
        b = b_union(b, leaf_bounds[it])
    return b


def partition_sah(subset, range_bounds):
    centroid = (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
    for it in subset:
        centroid = b_add_point(centroid, b_center(leaf_bounds[it]))
    edges = b_edges(centroid)
    split_axis = longest_axis(edges)
    range_start = F(centroid[0][split_axis])
    range_end = F(centroid[1][split_axis])
    if range_end <= range_start:
        return range_start, split_axis
    span = F(range_end - range_start)
    counts = [0] * SAH_BUCKETS_WARP
    buckets = [None] * SAH_BUCKETS_WARP
    for it in subset:
        c = b_center(leaf_bounds[it])
        bidx = int(F(SAH_BUCKETS_WARP) * F(F(c[split_axis]) - range_start) / span)
        bindex = max(0, min(SAH_BUCKETS_WARP - 1, bidx))
        counts[bindex] += 1
        if buckets[bindex] is None:
            buckets[bindex] = leaf_bounds[it]
        else:
            buckets[bindex] = b_union(leaf_bounds[it], buckets[bindex])  # bounds_union(item, bucket)
    left = (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
    right = (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
    left_areas = [F(0)] * (SAH_BUCKETS_WARP - 1)
    right_areas = [F(0)] * (SAH_BUCKETS_WARP - 1)
    counts_l = [0] * (SAH_BUCKETS_WARP - 1)
    counts_r = [0] * (SAH_BUCKETS_WARP - 1)
    count_l = 0
    count_r = 0
    for i in range(SAH_BUCKETS_WARP - 1):
        bound_start = buckets[i] if buckets[i] is not None else (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
        bound_end = buckets[SAH_BUCKETS_WARP - i - 1] if buckets[SAH_BUCKETS_WARP - i - 1] is not None else (np.array([MJ_MAXVAL] * 3, dtype=F), np.array([-MJ_MAXVAL] * 3, dtype=F))
        left = b_union(left, bound_start)
        right = b_union(right, bound_end)
        left_areas[i] = b_area(left)
        right_areas[SAH_BUCKETS_WARP - i - 2] = b_area(right)
        count_l += counts[i]
        count_r += counts[SAH_BUCKETS_WARP - i - 1]
        counts_l[i] = count_l
        counts_r[SAH_BUCKETS_WARP - i - 2] = count_r
    total_area = b_area(range_bounds)
    inv_total = F(F(1.0) / total_area)
    min_split = 0
    min_cost = F(np.finfo(np.float32).max)
    for i in range(SAH_BUCKETS_WARP - 1):
        p_below = F(left_areas[i] * inv_total)
        p_above = F(right_areas[i] * inv_total)
        cost = F(F(p_below * F(counts_l[i])) + F(p_above * F(counts_r[i])))
        if cost < min_cost:
            min_cost = cost
            min_split = i
    split_point = F(
        range_start
        + F(F(F(min_split + 1) * F(range_end - range_start)) / F(SAH_BUCKETS_WARP))
    )
    return split_point, split_axis


def warp_build(subset):
    if len(subset) <= LEAF_SIZE:
        return {"leaf": subset, "bounds": calc_bounds(subset)}
    # NOTE: `calc_bounds` uses the node's own subset; the in-place permutation of
    # `primitive_indices` does not change which items end up on each side.
    b = calc_bounds(subset)
    split_point, split_axis = partition_sah(subset, b)
    left = [it for it in subset if F(F(0.5) * F(leaf_bounds[it][0][split_axis] + leaf_bounds[it][1][split_axis])) < split_point]
    right = [it for it in subset if it not in left]
    if not left or not right:
        mid = len(subset) // 2
        left = subset[:mid]
        right = subset[mid:]
    node = {"bounds": b, "children": (warp_build(left), warp_build(right))}
    return node


def warp_order(node, acc):
    if "leaf" in node:
        acc.extend(node["leaf"])
        return
    left, right = node["children"]
    warp_order(right, acc)  # stack pops right child first
    warp_order(left, acc)


# ------------------------------------------------------- product SAH builder
def product_bin_area(lo, hi):
    union_lo = np.array([np.inf] * 3, dtype=F)
    union_hi = np.array([-np.inf] * 3, dtype=F)
    for i in range(len(lo)):
        union_lo = np.array([min(union_lo[a], lo[i][a]) for a in range(3)], dtype=F)
        union_hi = np.array([max(union_hi[a], hi[i][a]) for a in range(3)], dtype=F)
    e = union_hi - union_lo
    return F(2.0) * F(F(F(e[0] * e[1]) + F(e[1] * e[2])) + F(e[2] * e[0]))


def product_choose_split(subset):
    count = len(subset)
    cmin = np.array([np.inf] * 3, dtype=F)
    cmax = np.array([-np.inf] * 3, dtype=F)
    for it in subset:
        c = b_center(leaf_bounds[it])
        cmin = np.array([min(cmin[a], c[a]) for a in range(3)], dtype=F)
        cmax = np.array([max(cmax[a], c[a]) for a in range(3)], dtype=F)
    extents = (cmax - cmin).astype(F)
    if all(e == 0.0 for e in extents):
        return count // 2
    best = None
    for axis in range(3):
        if extents[axis] == 0.0:
            continue
        inverse = F(F(SAH_BINS_PRODUCT) / extents[axis])
        bins = []
        for it in subset:
            c = F(b_center(leaf_bounds[it])[axis])
            bins.append(min(SAH_BINS_PRODUCT - 1, int(F(F(c - cmin[axis]) * inverse))))
        counts = [0] * SAH_BINS_PRODUCT
        lo = [np.array([np.inf] * 3, dtype=F) for _ in range(SAH_BINS_PRODUCT)]
        hi = [np.array([-np.inf] * 3, dtype=F) for _ in range(SAH_BINS_PRODUCT)]
        for slot, it in enumerate(subset):
            bi = bins[slot]
            counts[bi] += 1
            b = leaf_bounds[it]
            lo[bi] = np.array([min(lo[bi][a], b[0][a]) for a in range(3)], dtype=F)
            hi[bi] = np.array([max(hi[bi][a], b[1][a]) for a in range(3)], dtype=F)
        for split in range(1, SAH_BINS_PRODUCT):
            lc = sum(counts[:split])
            rc = sum(counts[split:])
            if lc == 0 or rc == 0:
                continue
            la = product_bin_area(lo[:split], hi[:split])
            ra = product_bin_area(lo[split:], hi[split:])
            cost = F(F(la * F(lc)) + F(ra * F(rc)))
            cand = (float(cost), axis, split)
            if best is None or cand < best:
                best = cand
    if best is None:
        return count // 2
    _, axis, split = best
    inverse = F(F(SAH_BINS_PRODUCT) / extents[axis])
    left, right = [], []
    for it in subset:
        bi = min(SAH_BINS_PRODUCT - 1, int(F(F(F(b_center(leaf_bounds[it])[axis]) - cmin[axis]) * inverse)))
        (left if bi < split else right).append(it)
    if not left or not right:
        return count // 2
    return len(left)


def pbounds(node):
    return node[2] if node[0] == "leaf" else node[3]


def product_build(subset):
    if len(subset) == 1:
        return ("leaf", subset[0], leaf_bounds[subset[0]])
    split = product_choose_split(subset)
    left_ids, right_ids = subset[:split], subset[split:]
    left = product_build(left_ids)
    right = product_build(right_ids)
    lb, rb = pbounds(left), pbounds(right)
    lo = np.array([min(lb[0][a], rb[0][a]) for a in range(3)], dtype=F)
    hi = np.array([max(lb[1][a], rb[1][a]) for a in range(3)], dtype=F)
    return ("node", left, right, (lo, hi))


# ------------------------------------------------------- product heap traversal
def product_ray_aabb(origin, direction, bounds, t_limit):
    t0 = F(0.0)
    t1 = F(t_limit)
    for axis in range(3):
        o = F(origin[axis])
        d = F(direction[axis])
        lo = F(bounds[0][axis])
        hi = F(bounds[1][axis])
        if d == 0.0:
            if o < lo or o > hi:
                return None
            continue
        near = F(F(lo - o) / d)
        far = F(F(hi - o) / d)
        if near > far:
            near, far = far, near
        t0 = near if near > t0 else t0
        t1 = far if far < t1 else t1
        if t0 > t1:
            return None
    return t0


class Heap:
    def __init__(self):
        self.items = []  # (entry, node)

    def push(self, entry, node):
        items = self.items
        items.append((entry, node))
        idx = len(items) - 1
        while idx > 0:
            parent = (idx - 1) // 2
            if items[parent][0] <= entry:
                break
            items[idx] = items[parent]
            idx = parent
        items[idx] = (entry, node)

    def pop(self):
        items = self.items
        top = items[0]
        count = len(items) - 1
        if count > 0:
            last = items[count]
            idx = 0
            while True:
                left = 2 * idx + 1
                if left >= count:
                    break
                right = left + 1
                child = right if (right < count and items[right][0] < items[left][0]) else left
                if items[child][0] >= last[0]:
                    break
                items[idx] = items[child]
                idx = child
            items[idx] = last
        items.pop()
        return top


def product_order(root, origin, direction, t_limit):
    heap = Heap()
    entry = product_ray_aabb(origin, direction, pbounds(root), t_limit)
    if entry is None:
        return []
    heap.push(entry, root)
    order = []
    while heap.items:
        entry, node = heap.pop()
        if entry > t_limit:
            break
        if node[0] == "node":
            for child in (node[1], node[2]):
                ce = product_ray_aabb(origin, direction, pbounds(child), t_limit)
                if ce is not None:
                    heap.push(ce, child)
        else:
            order.append(node[1])
    return order


# ---------------------------------------------------------------- ray for pixel
ray_raw = (ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order/cam2-ray.f32le").read_bytes()
rays = np.frombuffer(ray_raw, dtype="<f4").reshape(-1, 6)
PIXEL = 41094
local_dir = rays[PIXEL, :3].astype(F)
cam_poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
poses = cam_poses["poses"]


def f32of(b):
    return np.uint32(b).view(np.float32)


cam_xpos = np.array([float(f32of(b)) for b in poses["cam_xpos_bits"][6:9]], dtype=F)
cam_xmat = np.array([float(f32of(b)) for b in poses["cam_xmat_bits"][18:27]], dtype=F).reshape(3, 3)
world_dir = np.array(
    [
        F(F(cam_xmat[0, 0] * local_dir[0]) + F(F(cam_xmat[0, 1] * local_dir[1]) + F(cam_xmat[0, 2] * local_dir[2]))),
        F(F(cam_xmat[1, 0] * local_dir[0]) + F(F(cam_xmat[1, 1] * local_dir[1]) + F(cam_xmat[1, 2] * local_dir[2]))),
        F(F(cam_xmat[2, 0] * local_dir[0]) + F(F(cam_xmat[2, 1] * local_dir[1]) + F(cam_xmat[2, 2] * local_dir[2]))),
    ],
    dtype=F,
)
print("\nray", PIXEL, "origin", cam_xpos.tolist(), "local_dir", local_dir.tolist(), "world_dir", world_dir.tolist())

# ---------------------------------------------------------------- build + orders
warp_root = warp_build(list(range(N)))
warp_visit = []
warp_order(warp_root, warp_visit)

product_root = product_build(list(range(N)))
origin = cam_xpos
max_cam_dist = F(125.0) / max(F(-local_dir[2]), F(1.0e-6))
initial_bound = F(F(max_cam_dist * F(F(1.0) + F(1.0e-6))) + F(1.0e-9))
product_visit = product_order(product_root, origin, world_dir, initial_bound)

def product_lifo_order(node, acc):
    """Same LIFO stack discipline as warp (`push left`, `push right` -> pop right first)."""
    if node[0] == "leaf":
        acc.append(node[1])
        return
    product_lifo_order(node[2], acc)
    product_lifo_order(node[1], acc)


def warp_heap_order(node, origin, direction, t_limit):
    """Product-side min-heap traversal applied to the upstream (warp) tree."""
    heap = Heap()
    entry = product_ray_aabb(origin, direction, node["bounds"], t_limit)
    heap.push(entry, node)
    order = []
    while heap.items:
        entry, cur = heap.pop()
        if "leaf" in cur:
            order.append(cur["leaf"][0])
        else:
            for child in cur["children"]:
                ce = product_ray_aabb(origin, direction, child["bounds"], t_limit)
                if ce is not None:
                    heap.push(ce, child)
    return order


def rel(order, a, b):
    return order.index(a) < order.index(b)


warp_lifo_visit = warp_visit
product_lifo_visit = []
product_lifo_order(product_root, product_lifo_visit)
warp_heap_visit = warp_heap_order(warp_root, origin, world_dir, initial_bound)

print("\nwarp leaf visit order   :", warp_visit)
print("product leaf visit order:", product_visit)
print("warp tree + LIFO        :", warp_lifo_visit)
print("product tree + LIFO     :", product_lifo_visit)
print("warp tree + product heap:", warp_heap_visit)
flex_ids = [i for i, d in enumerate(leaf_desc) if d.startswith("flex")]
print("flex leaf ids:", flex_ids, " -> labels", [leaf_desc[i] for i in flex_ids])


def last_visited_of_pair(order, a, b):
    return leaf_desc[b] if rel(order, a, b) else leaf_desc[a]


print("\n== relative order (leaf4=flex0edge0, leaf5=flex0edge1) ==")
for name, order in (
    ("warp tree + LIFO (upstream)", warp_lifo_visit),
    ("product tree + heap (current)", product_visit),
    ("product tree + LIFO (cheap fix)", product_lifo_visit),
    ("warp tree + heap (control)", warp_heap_visit),
):
    print(f"{name:32s}: 5-before-4={rel(order, 5, 4)}  winner(last={last_visited_of_pair(order, 4, 5)})")

# sanity: node counts equal for both builders
def count_nodes(node):
    if "leaf" in node:
        return 1
    return 1 + count_nodes(node["children"][0]) + count_nodes(node["children"][1])


def count_product(node):
    if node[0] == "node":
        return 1 + count_product(node[1]) + count_product(node[2])
    return 1


print("\nwarp nodes:", count_nodes(warp_root), " product nodes:", count_product(product_root), " expected:", 2 * N - 1)

# print both trees (leaf sets) for review
def show_warp(node, depth=0):
    ind = "  " * depth
    if "leaf" in node:
        print(f"{ind}leaf {node['leaf']} {[leaf_desc[i] for i in node['leaf']]}")
    else:
        print(f"{ind}node")
        show_warp(node["children"][0], depth + 1)
        show_warp(node["children"][1], depth + 1)


def show_product(node, depth=0):
    ind = "  " * depth
    if node[0] == "leaf":
        print(f"{ind}leaf {node[1]} {leaf_desc[node[1]]}")
    else:
        print(f"{ind}node")
        show_product(node[1], depth + 1)
        show_product(node[2], depth + 1)


print("\n--- upstream (warp SAH) tree: left first, but traversal pops right first ---")
show_warp(warp_root)
print("\n--- product (builder.rs SAH) tree ---")
show_product(product_root)
