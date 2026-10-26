"""QA independent CPU recompute of the 6 plane-family contacts from the pinned mujoco_warp formulas.

CPU only: imports mujoco (CPU library) + numpy; never imports warp/mujoco_warp, never creates a CUDA context.
Implements the pinned `collision_primitive.py` / `collision_primitive_core.py` math in float32 (step-wise,
left-associative, matching Python source order) and compares nacon/dist/pos/geomcollisionid against the
B0 GPU reference. Also computes the Rust .cu grouping variant for plane-cylinder pos3/pos4 to check whether
the device add-order differs from the pinned order.
"""
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
REF = ROOT / "docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json"
FIX = ROOT / "fixtures/g10-p5-primitive"
MJ_MAXVAL = np.float32(1.0e10)
HUGE = np.float32(-1.0e6)


def f32(x):
    return np.float32(x)


def v3(x, y, z):
    return np.array([x, y, z], dtype=np.float32)


def add(a, b):
    return (a + b).astype(np.float32)


def sub(a, b):
    return (a - b).astype(np.float32)


def scale(a, s):
    return (a * np.float32(s)).astype(np.float32)


def dot(a, b):
    r = a[0] * b[0]
    r = r + a[1] * b[1]
    r = r + a[2] * b[2]
    return np.float32(r)


def cross(a, b):
    return np.array([
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ], dtype=np.float32)


def matvec(rot9, v):
    m = np.asarray(rot9, dtype=np.float32).reshape(3, 3)
    return np.array([
        m[0, 0] * v[0] + m[0, 1] * v[1] + m[0, 2] * v[2],
        m[1, 0] * v[0] + m[1, 1] * v[1] + m[1, 2] * v[2],
        m[2, 0] * v[0] + m[2, 1] * v[1] + m[2, 2] * v[2],
    ], dtype=np.float32)


def transpose_matvec(rot9, v):
    m = np.asarray(rot9, dtype=np.float32).reshape(3, 3)
    return np.array([
        m[0, 0] * v[0] + m[1, 0] * v[1] + m[2, 0] * v[2],
        m[0, 1] * v[0] + m[1, 1] * v[1] + m[2, 1] * v[2],
        m[0, 2] * v[0] + m[1, 2] * v[1] + m[2, 2] * v[2],
    ], dtype=np.float32)


def plane_sphere(n, plane_pos, sphere_pos, radius):
    dist = dot(sub(sphere_pos, plane_pos), n) - np.float32(radius)
    pos = sub(sphere_pos, scale(n, np.float32(radius) + np.float32(0.5) * dist))
    return np.float32(dist), pos


def plane_ellipsoid(n, plane_pos, pos_e, rot_e, size_e):
    local_n = transpose_matvec(rot_e, n)
    scaled = local_n * size_e
    length = np.float32(np.sqrt(dot(scaled, scaled)))
    normalized = scaled / length
    support = -normalized
    local = support * size_e
    pos = add(pos_e, matvec(rot_e, local))
    dist = dot(n, sub(pos, plane_pos))
    pos = sub(pos, scale(scale(n, dist), np.float32(0.5)))
    return np.float32(dist), pos


def plane_box(n, plane_pos, pos_b, rot_b, size_b):
    center_dist = dot(sub(pos_b, plane_pos), n)
    d = np.full(8, MJ_MAXVAL, dtype=np.float32)
    p = np.zeros((8, 3), dtype=np.float32)
    for i in range(8):
        corner = v3(size_b[0] if (i & 1) else -size_b[0],
                    size_b[1] if (i & 2) else -size_b[1],
                    size_b[2] if (i & 4) else -size_b[2])
        corner = matvec(rot_b, corner)
        ldist = dot(n, corner)
        cdist = center_dist + ldist
        d[i] = cdist
        p[i] = sub(add(corner, pos_b), scale(scale(n, np.float32(0.5)), cdist))
    return d, p


def plane_capsule(n, plane_pos, pos_c, axis_c, radius, half_length):
    perpendicular = sub(axis_c, scale(n, dot(n, axis_c)))
    norm = np.float32(np.sqrt(dot(perpendicular, perpendicular)))
    if norm == 0.0:
        b = perpendicular
    else:
        b = perpendicular / norm
    if norm < np.float32(0.5):
        if np.float32(-0.5) < n[1] and n[1] < np.float32(0.5):
            b = v3(0.0, 1.0, 0.0)
        else:
            b = v3(0.0, 0.0, 1.0)
    c = cross(n, b)
    frame = np.array([n[0], n[1], n[2], b[0], b[1], b[2], c[0], c[1], c[2]], dtype=np.float32)
    segment = scale(axis_c, half_length)
    d1, p1 = plane_sphere(n, plane_pos, add(pos_c, segment), radius)
    d2, p2 = plane_sphere(n, plane_pos, sub(pos_c, segment), radius)
    return np.array([d1, d2], dtype=np.float32), np.array([p1, p2], dtype=np.float32), frame


def plane_cylinder(n, plane_pos, center, axis_c, radius, half_height):
    axis = axis_c
    prjaxis = dot(n, axis)
    if prjaxis > 0:
        axis = scale(axis, -1.0)
        prjaxis = -prjaxis
    dist0 = dot(sub(center, plane_pos), n)
    vec = sub(scale(axis, prjaxis), n)
    len_sqr = dot(vec, vec)
    if len_sqr >= np.float32(1e-12):
        vec = scale(vec, np.float32(radius) / np.float32(np.sqrt(len_sqr)))
    else:
        vec = v3(radius, 0.0, 0.0)
    prjvec = dot(vec, n)
    axis = scale(axis, half_height)
    prjaxis = prjaxis * half_height
    dist1 = dist0 + prjaxis + prjvec
    pos1 = sub(add(add(center, vec), axis), scale(n, dist1 * np.float32(0.5)))
    dist2 = dist0 - prjaxis + prjvec
    pos2 = sub(sub(add(center, vec), axis), scale(n, dist2 * np.float32(0.5)))
    prjvec1 = -prjvec * np.float32(0.5)
    dist3 = dist0 + prjaxis + prjvec1
    vec1 = cross(vec, axis)
    vec1 = (vec1 / np.float32(np.sqrt(dot(vec1, vec1)))) * (np.float32(radius) * np.float32(np.sqrt(3.0)) * np.float32(0.5))
    pos3 = sub(sub(sub(add(add(center, vec1), axis), scale(vec, np.float32(0.5))), scale(n, dist3 * np.float32(0.5))), v3(0, 0, 0))
    pos4 = sub(sub(sub(add(sub(center, vec1), axis), scale(vec, np.float32(0.5))), scale(n, dist3 * np.float32(0.5))), v3(0, 0, 0))
    # Rust .cu variant for pos3/pos4: base = (center+axis) - 0.5*vec; pos = ±vec1 + base - n*(dist3/2)
    base = sub(add(center, axis), scale(vec, np.float32(0.5)))
    rust_pos3 = sub(add(base, vec1), scale(n, dist3 * np.float32(0.5)))
    rust_pos4 = sub(sub(base, vec1), scale(n, dist3 * np.float32(0.5)))
    return (np.array([dist1, dist2, dist3, dist3], dtype=np.float32),
            np.array([pos1, pos2, pos3, pos4], dtype=np.float32),
            np.array([rust_pos3, rust_pos4], dtype=np.float32))


def plane_convex(n, plane_pos, pos_c, rot_c, verts):
    local_pp = transpose_matvec(rot_c, sub(plane_pos, pos_c))
    local_n = transpose_matvec(rot_c, n)
    count = len(verts)
    indices = [-1, -1, -1, -1]
    max_support = HUGE
    a = v3(0, 0, 0)
    for i in range(count):
        support = dot(sub(local_pp, verts[i]), local_n)
        if support > max_support:
            max_support = support
            indices[0] = i
            a = verts[i]
    if max_support < 0:
        return np.zeros(0, dtype=np.float32), np.zeros((0, 3), dtype=np.float32)
    threshold = max_support - np.float32(1e-3)
    b_dist = HUGE
    b = v3(0, 0, 0)
    for i in range(count):
        support = dot(sub(local_pp, verts[i]), local_n)
        mask = np.float32(0.0) if support > threshold else HUGE
        d = sub(a, verts[i])
        dist = dot(d, d) + mask
        if dist > b_dist:
            indices[1] = i
            b_dist = dist
            b = verts[i]
    ab = cross(local_n, sub(a, b))
    c_dist = HUGE
    c = v3(0, 0, 0)
    for i in range(count):
        support = dot(sub(local_pp, verts[i]), local_n)
        mask = np.float32(0.0) if support > threshold else HUGE
        dist = abs(dot(sub(a, verts[i]), ab)) + mask
        if dist > c_dist:
            indices[2] = i
            c_dist = dist
            c = verts[i]
    ac = cross(local_n, sub(a, c))
    bc = cross(local_n, sub(b, c))
    d_dist = HUGE
    for i in range(count):
        support = dot(sub(local_pp, verts[i]), local_n)
        mask = np.float32(0.0) if support > threshold else HUGE
        dist_ap = abs(dot(sub(a, verts[i]), ac)) + mask
        dist_bp = abs(dot(sub(b, verts[i]), bc)) + mask
        if dist_ap + dist_bp > d_dist:
            indices[3] = i
            d_dist = dist_ap + dist_bp
    dists = []
    poss = []
    ids = []
    for i in range(3, -1, -1):
        idx = indices[i]
        ncount = sum(1 for j in range(i + 1) if indices[j] == idx)
        if ncount == 1:
            local = verts[idx]
            world = add(pos_c, matvec(rot_c, local))
            support = dot(sub(local_pp, local), local_n)
            d = -support
            poss.append(sub(world, scale(n, np.float32(0.5) * d)))
            dists.append(np.float32(d))
            # geomcollisionid is the contact write ordinal (0..n-1), not the vertex index
            ids.append(len(dists) - 1)
    return np.array(dists, dtype=np.float32), np.array(poss, dtype=np.float32), np.array(ids, dtype=np.int32)


def ulp(a, b):
    def sbits(x):
        return struct.unpack('<i', struct.pack('<f', float(np.float32(x))))[0]
    ai = sbits(a)
    bi = sbits(b)
    def key(v):
        return -int(int(v) & 0x7FFFFFFF) if v < 0 else int(v)
    return abs(key(ai) - key(bi))


def rows_of(field):
    # B0 JSON fields: scalar field is [[v00],[v01],...] per contact row; vec3 is [[x,y,z],...]
    out = []
    for row in field:
        out.append(np.array(row, dtype=np.float32) if len(row) > 1 else np.array([row[0]], dtype=np.float32))
    return out


def bits(x):
    return int(np.float32(x).view(np.int32))


ref = json.loads(REF.read_text(encoding="utf-8"))
cases = ["plane-sphere", "plane-capsule", "plane-ellipsoid", "plane-cylinder", "plane-box", "plane-mesh"]
rows = []
for name in cases:
    case = ref["cases"][name]
    out = case["output"]
    xpos = np.array(out["geom_xpos"][0], dtype=np.float32)
    xmat = np.array(out["geom_xmat"][0], dtype=np.float32)
    model = mujoco.MjModel.from_xml_path(str(FIX / (name + ".xml")))
    gtype = model.geom_type.tolist()
    size = np.array(model.geom_size, dtype=np.float32)
    plane_idx = gtype.index(0)
    other_idx = 1 - plane_idx
    n = v3(xmat[plane_idx][2], xmat[plane_idx][5], xmat[plane_idx][8])
    plane_pos = xpos[plane_idx]
    pos_o = xpos[other_idx]
    rot_o = xmat[other_idx]
    size_o = size[other_idx]
    t = gtype[other_idx]
    if t == 2:
        dist, pos = plane_sphere(n, plane_pos, pos_o, size_o[0])
        dists = np.array([dist], dtype=np.float32); poss = np.array([pos], dtype=np.float32); ids = np.array([0], dtype=np.int32)
    elif t == 3:
        axis = v3(rot_o[2], rot_o[5], rot_o[8])
        dists, poss, _frame = plane_capsule(n, plane_pos, pos_o, axis, size_o[0], size_o[1]); ids = np.array([0, 1], dtype=np.int32)
    elif t == 4:
        dist, pos = plane_ellipsoid(n, plane_pos, pos_o, rot_o, size_o)
        dists = np.array([dist], dtype=np.float32); poss = np.array([pos], dtype=np.float32); ids = np.array([0], dtype=np.int32)
    elif t == 5:
        axis = v3(rot_o[2], rot_o[5], rot_o[8])
        dists, poss, _rust = plane_cylinder(n, plane_pos, pos_o, axis, size_o[0], size_o[1]); ids = np.array([0, 1, 2, 3], dtype=np.int32)
    elif t == 6:
        dists, poss = plane_box(n, plane_pos, pos_o, rot_o, size_o); ids = np.arange(8, dtype=np.int32)
    else:
        verts = np.array(model.mesh_vert, dtype=np.float32)
        dists, poss, ids = plane_convex(n, plane_pos, pos_o, rot_o, verts)
    # write gate: detected = dist < margin + gap; fixtures use default margin 0 (verified by reference counts)
    keep = [i for i in range(len(dists)) if dists[i] < np.float32(0.0)]
    mine_d = dists[keep]; mine_p = poss[keep]; mine_id = ids[keep]
    ref_rows_d = rows_of(out["fields"]["dist"])
    ref_rows_p = rows_of(out["fields"]["pos"])
    ref_rows_id = rows_of(out["fields"]["geomcollisionid"])
    ref_d = np.array([row[0] for row in ref_rows_d], dtype=np.float32)
    ref_p = np.array([np.array(row) for row in ref_rows_p], dtype=np.float32).reshape(-1, 3)
    ref_id = np.array([int(row[0]) for row in ref_rows_id], dtype=np.int32)
    max_dup = 0
    max_pp_ulp = 0
    max_pos_diff = 0.0
    max_pos_bound = 0.0
    max_pos_ratio = 0.0
    max_dist_diff = 0.0
    max_dist_bound = 0.0
    max_dist_ratio = 0.0
    detail = []
    if len(mine_d) == len(ref_d):
        for i in range(len(ref_d)):
            j = int(np.where(mine_id == ref_id[i])[0][0]) if np.any(mine_id == ref_id[i]) else i
            du = ulp(mine_d[j], ref_d[i])
            pu = max(ulp(mine_p[j][k], ref_p[i][k]) for k in range(3))
            # criterion as implemented in tests: dist <= 4 ULP of ref (ref==0 -> 4 ULP @ 1.0)
            dscale = np.float32(abs(ref_d[i])) if ref_d[i] != 0 else np.float32(1.0)
            dbound = np.float32(4.0) * np.spacing(dscale)
            ddiff = np.float32(abs(mine_d[j] - ref_d[i]))
            dratio = float(ddiff / dbound) if dbound > 0 else float("inf")
            # pos: 4 * ULP(scale) where scale = max absolute ref component of the contact
            pscale = np.float32(max(abs(float(x)) for x in ref_p[i]))
            pbound = np.float32(4.0) * np.spacing(pscale)
            pdiff = np.float32(max(abs(mine_p[j][k] - ref_p[i][k]) for k in range(3)))
            pratio = float(pdiff / pbound) if pbound > 0 else float("inf")
            max_dup = max(max_dup, du)
            max_pp_ulp = max(max_pp_ulp, pu)
            max_pos_diff = max(max_pos_diff, float(pdiff)); max_pos_bound = max(max_pos_bound, float(pbound)); max_pos_ratio = max(max_pos_ratio, pratio)
            max_dist_diff = max(max_dist_diff, float(ddiff)); max_dist_bound = max(max_dist_bound, float(dbound)); max_dist_ratio = max(max_dist_ratio, dratio)
            detail.append(f"id{ref_id[i]}:dup={du},pup={pu},pratio={pratio:.3f},dratio={dratio:.3f}")
    rows.append({"case": name, "mine_count": int(len(mine_d)), "ref_count": int(len(ref_d)),
                 "ids_mine": mine_id.tolist(), "ids_ref": ref_id.tolist(),
                 "max_dist_ulp": int(max_dup), "max_pos_ulp": int(max_pp_ulp),
                 "max_pos_diff": max_pos_diff, "max_pos_bound": max_pos_bound, "max_pos_bound_ratio": max_pos_ratio,
                 "max_dist_diff": max_dist_diff, "max_dist_bound": max_dist_bound, "max_dist_bound_ratio": max_dist_ratio,
                 "detail": detail})

out = {"mujoco_version": mujoco.__version__, "python": ".".join(map(str, sys.version_info[:3])), "cases": rows}
(ROOT / "target/qa-plane").mkdir(parents=True, exist_ok=True)
(ROOT / "target/qa-plane/recompute-plane.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
for r in rows:
    print(r["case"], "mine", r["mine_count"], "ref", r["ref_count"], "max_dist_ulp", r["max_dist_ulp"], "max_pos_ulp", r["max_pos_ulp"], "ids", r["ids_mine"], r["ids_ref"])
print("mujoco", out["mujoco_version"])
