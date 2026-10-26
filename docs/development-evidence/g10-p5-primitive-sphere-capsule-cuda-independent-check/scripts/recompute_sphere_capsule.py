"""Independent CPU recomputation of pinned f32 sphere/capsule primitive semantics
against the B0 upstream GPU reference (4 cases). No warp / mujoco_warp import.
Mirrors frozen collision_primitive_core.py formulas operand-by-operand in float32.
"""
import json
import math
import struct
import sys
from pathlib import Path

import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
REF = ROOT / "docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json"

F = np.float32


def f32(x):
    return F(x)


def v3(x, y, z):
    return np.array([x, y, z], dtype=np.float32)


def add(a, b):
    return np.array([F(a[0] + b[0]), F(a[1] + b[1]), F(a[2] + b[2])], dtype=np.float32)


def sub(a, b):
    return np.array([F(a[0] - b[0]), F(a[1] - b[1]), F(a[2] - b[2])], dtype=np.float32)


def scale(a, s):
    s = F(s)
    return np.array([F(a[0] * s), F(a[1] * s), F(a[2] * s)], dtype=np.float32)


def dot(a, b):
    return F(F(a[0] * b[0]) + F(F(a[1] * b[1]) + F(a[2] * b[2])))


def length(a):
    return F(np.sqrt(dot(a, a), dtype=np.float32))


def cross(a, b):
    return np.array(
        [
            F(F(a[1] * b[2]) - F(a[2] * b[1])),
            F(F(a[2] * b[0]) - F(a[0] * b[2])),
            F(F(a[0] * b[1]) - F(a[1] * b[0])),
        ],
        dtype=np.float32,
    )


def clamp(x, lo, hi):
    return F(min(max(F(x), F(lo)), F(hi)))


def sign(x):
    return F(1.0) if x > 0 else (F(-1.0) if x < 0 else F(0.0))


def safe_div(a, b):
    mj_minval = F(1e-15)
    return F(F(a) / F(b)) if abs(F(b)) >= mj_minval else F(0.0)


def make_frame(normal):
    a = normal / length(normal) if length(normal) != 0 else v3(0, 0, 0)
    selector = v3(0.0, 1.0, 0.0)
    if not (-0.5 < a[1] < 0.5):
        selector = v3(0.0, 0.0, 1.0)
    b = selector - scale(a, dot(a, selector))
    b = b / length(b) if length(b) != 0 else v3(0, 0, 0)
    c = cross(a, b)
    return np.concatenate([a, b, c]).astype(np.float32)


def sphere_sphere(pos1, r1, pos2, r2):
    d = sub(pos2, pos1)
    dist = length(d)
    if dist == 0:
        n = v3(1.0, 0.0, 0.0)
    else:
        n = np.array([F(d[0] / dist), F(d[1] / dist), F(d[2] / dist)], dtype=np.float32)
    dist = F(dist - F(r1 + r2))
    pos = add(pos1, scale(n, F(r1 + F(F(0.5) * dist))))
    return dist, pos, n


def sphere_capsule(sphere_pos, sr, cap_pos, cap_axis, cr, chalf):
    off = sub(sphere_pos, cap_pos)
    proj = clamp(dot(cap_axis, off), -chalf, chalf)
    pt = add(cap_pos, scale(cap_axis, proj))
    return sphere_sphere(sphere_pos, sr, pt, cr)


def sphere_cylinder(sphere_pos, sr, cyl_pos, cyl_axis, cr, chalf):
    vec = sub(sphere_pos, cyl_pos)
    x = dot(vec, cyl_axis)
    a_proj = scale(cyl_axis, x)
    p_proj = sub(vec, a_proj)
    p2 = dot(p_proj, p_proj)
    collide_side = abs(x) < chalf
    collide_cap = p2 < F(cr * cr)
    if collide_side and collide_cap:
        dist_cap = F(chalf - abs(x))
        dist_radius = F(cr - F(np.sqrt(p2, dtype=np.float32)))
        if dist_cap < dist_radius:
            collide_side = False
        else:
            collide_cap = False
    if collide_side:
        return sphere_sphere(sphere_pos, sr, add(cyl_pos, a_proj), cr)
    if collide_cap:
        if x > 0:
            pos_cap = add(cyl_pos, scale(cyl_axis, chalf))
            plane_normal = cyl_axis
        else:
            pos_cap = sub(cyl_pos, scale(cyl_axis, chalf))
            plane_normal = scale(cyl_axis, -1.0)
        dist = F(dot(sub(sphere_pos, pos_cap), plane_normal) - sr)
        pos = sub(sphere_pos, scale(plane_normal, F(sr + F(F(0.5) * dist))))
        return dist, pos, scale(plane_normal, -1.0)
    inv_len = safe_div(1.0, F(np.sqrt(p2, dtype=np.float32)))
    p_proj = scale(p_proj, F(cr * inv_len))
    cap_offset = scale(cyl_axis, F(sign(x) * chalf))
    pos_corner = add(add(cyl_pos, cap_offset), p_proj)
    return sphere_sphere(sphere_pos, sr, pos_corner, F(0.0))


def fma(a, b, c):
    """Emulate CUDA fma.rn contraction (single rounding): a*b + c in one rounding."""
    return F(float(a) * float(b) + float(c))


def capsule_capsule(pa, a1, r1, h1, pb, a2, r2, h2, margin):
    contacts = []
    ax1 = scale(a1, h1)
    ax2 = scale(a2, h2)
    dif = sub(pa, pb)
    ma = dot(ax1, ax1)
    mb = -dot(ax1, ax2)
    mc = dot(ax2, ax2)
    u = -dot(ax1, dif)
    v = dot(ax2, dif)
    # NVCC fmad=true contracts `ma*mc - mb*mb` into fma(ma, mc, -(mb*mb)) (single rounding).
    # Without fusion this is exactly 0.0 for the capsule-capsule fixture, flipping the branch
    # (parallel -> 2 contacts) and diverging from the GPU reference (nacon=1).
    det_fused = fma(ma, mc, -F(mb * mb))
    det_unfused = F(F(ma * mc) - F(mb * mb))
    det = det_fused if USE_FUSED_DET else det_unfused
    mj_minval = F(1e-15)
    if abs(det) >= mj_minval:
        inv_det = F(F(1.0) / det)
        x1 = F(F(F(mc * u) - F(mb * v)) * inv_det)
        x2 = F(F(F(ma * v) - F(mb * u)) * inv_det)
        if x1 > 1:
            x1 = F(1.0)
            x2 = F(F(v - mb) / mc)
        elif x1 < -1:
            x1 = F(-1.0)
            x2 = F(F(v + mb) / mc)
        if x2 > 1:
            x2 = F(1.0)
            x1 = clamp(F(F(u - mb) / ma), -1.0, 1.0)
        elif x2 < -1:
            x2 = F(-1.0)
            x1 = clamp(F(F(u + mb) / ma), -1.0, 1.0)
        vec1 = add(pa, scale(ax1, x1))
        vec2 = add(pb, scale(ax2, x2))
        d, p, n = sphere_sphere(vec1, r1, vec2, r2)
        if d <= margin:
            contacts.append((d, p, n))
    else:
        cnt = 0
        vec1 = add(pa, ax1)
        x2 = clamp(F(F(v - mb) / mc), -1.0, 1.0)
        vec2 = add(pb, scale(ax2, x2))
        d, p, n = sphere_sphere(vec1, r1, vec2, r2)
        if d <= margin:
            contacts.append((d, p, n))
            cnt += 1
        vec1 = sub(pa, ax1)
        x2 = clamp(F(F(v + mb) / mc), -1.0, 1.0)
        vec2 = add(pb, scale(ax2, x2))
        d, p, n = sphere_sphere(vec1, r1, vec2, r2)
        if d <= margin:
            contacts.append((d, p, n))
            cnt += 1
        if cnt < 2:
            vec2 = add(pb, ax2)
            x1 = clamp(F(F(u - mb) / ma), -1.0, 1.0)
            vec1 = add(pa, scale(ax1, x1))
            d, p, n = sphere_sphere(vec1, r1, vec2, r2)
            if d <= margin:
                contacts.append((d, p, n))
                cnt += 1
        if cnt < 2:
            vec2 = sub(pb, ax2)
            x1 = clamp(F(F(u + mb) / ma), -1.0, 1.0)
            vec1 = add(pa, scale(ax1, x1))
            d, p, n = sphere_sphere(vec1, r1, vec2, r2)
            if d <= margin:
                contacts.append((d, p, n))
    return contacts


CASES = {
    "sphere-sphere": {"geom_size": [[0.1, 0, 0], [0.1, 0, 0]], "geom_type": [2, 2]},
    "sphere-capsule": {"geom_size": [[0.1, 0, 0], [0.05, 0.12, 0]], "geom_type": [2, 3]},
    "sphere-cylinder": {"geom_size": [[0.1, 0, 0], [0.06, 0.1, 0]], "geom_type": [2, 5]},
    "capsule-capsule": {"geom_size": [[0.05, 0.12, 0], [0.05, 0.1, 0]], "geom_type": [3, 3]},
}

# Device code is compiled by NVCC with default fmad=true, so `ma*mc - mb*mb` is contracted
# to a single-rounding FMA. Set False to show the non-fused (CPU-ish) branch divergence.
USE_FUSED_DET = "--unfused" not in sys.argv


def ulp_key(v):
    b = struct.unpack("<I", struct.pack("<f", float(v)))[0]
    return -int(b & 0x7FFFFFFF) if b & 0x80000000 else int(b)


def ulp_distance(a, b):
    return abs(ulp_key(a) - ulp_key(b))


def main():
    ref = json.loads(REF.read_text(encoding="utf-8"))
    out = {"python": ".".join(map(str, sys.version_info[:3])), "numpy": np.__version__, "cases": []}
    for name, meta in CASES.items():
        case = ref["cases"][name]
        xpos = case["output"]["geom_xpos"]
        # JSON shape: [[[x,y,z] for geom] for world]
        pos = [v3(*xpos[0][0]), v3(*xpos[0][1])]
        sizes = meta["geom_size"]
        t1, t2 = meta["geom_type"]
        if name == "sphere-sphere":
            contacts = [sphere_sphere(pos[0], sizes[0][0], pos[1], sizes[1][0])]
        elif name == "sphere-capsule":
            axis = v3(0, 0, 1)  # identity xmat -> third column
            contacts = [sphere_capsule(pos[0], sizes[0][0], pos[1], axis, sizes[1][0], sizes[1][1])]
        elif name == "sphere-cylinder":
            axis = v3(0, 0, 1)
            contacts = [
                sphere_cylinder(pos[0], sizes[0][0], pos[1], axis, sizes[1][0], sizes[1][1])
            ]
        else:
            axis = v3(0, 0, 1)
            contacts = capsule_capsule(
                pos[0], axis, sizes[0][0], sizes[0][1], pos[1], axis, sizes[1][0], sizes[1][1], F(0.0)
            )
        contacts = [(d, p, n) for (d, p, n) in contacts if d < F(0.0)]  # detected = dist < margin+gap
        ref_rows = case["output"]["fields"]
        ref_dist = [F(row[0]) for row in ref_rows["dist"]]
        ref_pos = [[F(c) for c in row] for row in ref_rows["pos"]]
        ref_frame = [[F(c) for c in row] for row in ref_rows["frame"]]
        ref_ids = [int(row[0]) for row in ref_rows["geomcollisionid"]]
        dist_ulp = 0
        pos_ulp = 0
        frame_ulp = 0
        detail = []
        for i, (d, p, n) in enumerate(contacts):
            if i >= len(ref_dist):
                break
            dist_ulp = max(dist_ulp, ulp_distance(d, ref_dist[i]))
            pos_ulp = max(pos_ulp, max(ulp_distance(p[k], ref_pos[i][k]) for k in range(3)))
            frame = make_frame(n)
            frame_ulp = max(frame_ulp, max(ulp_distance(frame[k], ref_frame[i][k]) for k in range(9)))
            detail.append(f"row{i}:dup={ulp_distance(d, ref_dist[i])}")
        out["cases"].append(
            {
                "case": name,
                "mine_count": len(contacts),
                "ref_count": len(ref_dist),
                "ids_mine": list(range(len(contacts))),
                "ids_ref": ref_ids,
                "max_dist_ulp": dist_ulp,
                "max_pos_ulp": pos_ulp,
                "max_frame_ulp": frame_ulp,
                "fused_det_emulation": USE_FUSED_DET,
                "detail": detail,
            }
        )
    (ROOT / "target/qa-b2").mkdir(parents=True, exist_ok=True)
    (ROOT / "target/qa-b2/recompute-sphere-capsule.json").write_text(
        json.dumps(out, indent=1), encoding="utf-8"
    )
    for c in out["cases"]:
        print(
            c["case"],
            "mine",
            c["mine_count"],
            "ref",
            c["ref_count"],
            "dist_ulp",
            c["max_dist_ulp"],
            "pos_ulp",
            c["max_pos_ulp"],
            "frame_ulp",
            c["max_frame_ulp"],
            "ids",
            c["ids_mine"],
            c["ids_ref"],
        )
    print("numpy", np.__version__)


if __name__ == "__main__":
    main()
