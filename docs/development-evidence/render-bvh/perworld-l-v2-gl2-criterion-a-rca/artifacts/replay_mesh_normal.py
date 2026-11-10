#!/usr/bin/env python3
"""Read-only CPU replay of the 5 G-L2 criterion-A rgb mismatched pixels.

Purpose: reproduce the mesh hit (face / bary / distance) from the diagnostic
pixel dump with f32 arithmetic, validate against the dumped depth bits, and
compare the two normal chains:

  upstream (pinned mujoco_warp 71da24d9, _src/ray.py::ray_mesh_with_bvh):
      n_local = cross(ab, ac)              # unnormalized
      normal  = normalize(mat @ n_local)   # ONE normalize
  product (mjwarp-rs):
      render_math.cu:    *out_normal = normalize3_mesh(min_normal)
      render_common.cu:  *normal = normalize3_mesh(mat_vec(mat, n_local))
      => normalize(mat @ normalize(cross))  # TWO normalizes

Reads only: the dump json, the G-L1 reference state.npz, the pinned model dump.
Writes nothing. Deterministic; no GPU.
"""
import json
import struct
from pathlib import Path

import numpy as np

F = np.float32
ROOT = Path(__file__).resolve().parents[5]


def M(a, b):
    return F(F(a) * F(b))


def A(a, b):
    return F(F(a) + F(b))


def S(a, b):
    return F(F(a) - F(b))


def D(a, b):
    return F(F(a) / F(b))


def FMA(a, b, c):
    return F(float(F(a)) * float(F(b)) + float(F(c)))


def bits2f(bits):
    return [struct.unpack("<f", struct.pack("<I", int(b)))[0] for b in bits]


def f2bits(x):
    return struct.unpack("<I", struct.pack("<f", float(F(x))))[0]


def ulp(a, b):
    return abs(int(f2bits(a)) - int(f2bits(b)))


def sub3(a, b):
    return [S(a[0], b[0]), S(a[1], b[1]), S(a[2], b[2])]


def dot_pinned(a, b):
    # pinned warp generated order: mul on index 1, fma index 0, fma index 2
    return FMA(a[2], b[2], FMA(a[0], b[0], M(a[1], b[1])))


def mat_vec(m, a):
    return [FMA(m[2], a[2], FMA(m[0], a[0], M(m[1], a[1]))),
            FMA(m[5], a[2], FMA(m[3], a[0], M(m[4], a[1]))),
            FMA(m[8], a[2], FMA(m[6], a[0], M(m[7], a[1])))]


def mat_t_vec(m, a):
    return [FMA(m[6], a[2], FMA(m[0], a[0], M(m[3], a[1]))),
            FMA(m[7], a[2], FMA(m[1], a[0], M(m[4], a[1]))),
            FMA(m[8], a[2], FMA(m[2], a[0], M(m[5], a[1])))]


def normalize3(a):
    l = F(np.sqrt(float(FMA(a[2], a[2], FMA(a[1], a[1], M(a[0], a[0]))))))
    if l > F(0):
        return [D(a[0], l), D(a[1], l), D(a[2], l)]
    return [F(0), F(0), F(0)]


def cross(a, b):
    return [S(M(a[1], b[2]), M(a[2], b[1])),
            S(M(a[2], b[0]), M(a[0], b[2])),
            S(M(a[0], b[1]), M(a[1], b[0]))]


def diff_product(a, b, c, d):
    cd = M(c, d)
    return A(FMA(a, b, F(-cd)), FMA(F(-c), d, cd))


def max_dim(d):
    xs = [abs(float(d[0])), abs(float(d[1])), abs(float(d[2]))]
    best = 0
    for i in (1, 2):
        if xs[i] > xs[best]:
            best = i
    return best


def intersect_tri(p, dr, a, b, c):
    """warp intersect_ray_tri_woop (include/warp-headers/intersect.h)."""
    kz = max_dim(dr)
    kx = kz + 1
    if kx == 3:
        kx = 0
    ky = kx + 1
    if ky == 3:
        ky = 0
    if float(dr[kz]) < 0:
        kx, ky = ky, kx
    sx = D(dr[kx], dr[kz])
    sy = D(dr[ky], dr[kz])
    sz = D(F(1), dr[kz])
    pa, pb, pc = sub3(a, p), sub3(b, p), sub3(c, p)

    def proj(v):
        return (S(v[kx], M(sx, v[kz])), S(v[ky], M(sy, v[kz])))

    ax, ay = proj(pa)
    bx, by = proj(pb)
    cx, cy = proj(pc)
    uu = diff_product(cx, by, cy, bx)
    vv = diff_product(ax, cy, ay, cx)
    ww = diff_product(bx, ay, by, ax)
    if ((uu < F(0) or vv < F(0) or ww < F(0))
            and (uu > F(0) or vv > F(0) or ww > F(0))):
        return None
    det = A(A(uu, vv), ww)
    if det == F(0):
        return None
    az = M(sz, pa[kz])
    bz = M(sz, pb[kz])
    cz = M(sz, pc[kz])
    t = A(A(M(uu, az), M(vv, bz)), M(ww, cz))
    det_sign = struct.unpack("<i", struct.pack("<f", float(det)))[0] & -2147483648
    t_bits = struct.unpack("<i", struct.pack("<f", float(t)))[0]
    xorf = struct.unpack("<f", struct.pack("<I", (t_bits ^ det_sign) & 0xFFFFFFFF))[0]
    if xorf < 0:
        return None
    rcp = D(F(1), det)
    return M(uu, rcp), M(vv, rcp), M(t, rcp), cross(sub3(b, a), sub3(c, a))


def main():
    dump_path = ROOT / "target/render-bvh/perworld-l-v2-gl2/diag/pixel-dump.json"
    ref = ROOT / "fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040"
    model_path = ROOT / "tests/support/render_bvh/light/model-315.json"

    raw = dump_path.read_bytes()
    dump = json.loads(raw)
    state = np.load(ref / "state.npz")
    gx = state["geom_xpos"].astype(np.float32).reshape(3, 7, 3)
    gxm = state["geom_xmat"].astype(np.float32).reshape(3, 7, 9)
    mod = json.loads(model_path.read_text(encoding="utf-8"))["arrays"]
    mesh_vert = np.asarray(mod["mesh_vert"]["values"], dtype=np.float32).reshape(-1, 3)
    mesh_face = np.asarray(mod["mesh_face"]["values"], dtype=np.int64).reshape(-1, 3)
    face_normal_tbl = np.asarray(mod["mesh_facenormal"]["values"], dtype=np.int64).reshape(-1, 3)
    mesh_normal = np.asarray(mod["mesh_normal"]["values"], dtype=np.float32).reshape(-1, 3)

    import hashlib
    print("dump path : target/render-bvh/perworld-l-v2-gl2/diag/pixel-dump.json")
    print("dump sha  :", hashlib.sha256(raw).hexdigest())
    print("dump bytes:", len(raw))
    print("mismatches:", len(dump["rgb_mismatches"]))
    print("")

    cam = dump["camera_poses"][1]
    cam_pos = [F(x) for x in bits2f(cam["xpos_bits"])]
    cam_mat = [F(x) for x in bits2f(cam["xmat_bits"])]

    for e in dump["rgb_mismatches"]:
        w = e["world"]
        geom_pos = [F(v) for v in gx[w, 3]]
        geom_mat = [F(v) for v in gxm[w, 3]]
        dir_local = [F(x) for x in bits2f(e["ray"]["direction_bits"])]
        dir_world = mat_vec(cam_mat, dir_local)
        lpnt = mat_t_vec(geom_mat, sub3(cam_pos, geom_pos))
        lvec = mat_t_vec(geom_mat, dir_world)

        best = None
        for face in range(4):
            idx = [int(i) for i in mesh_face[face]]
            a = [F(v) for v in mesh_vert[idx[0]]]
            b = [F(v) for v in mesh_vert[idx[1]]]
            c = [F(v) for v in mesh_vert[idx[2]]]
            hit = intersect_tri(lpnt, lvec, a, b, c)
            if hit is None:
                continue
            u, v, t, n_local = hit
            if t >= F(0) and (best is None or float(t) < float(best[3])):
                best = (face, u, v, t, n_local)
        if best is None:
            print(f"pixel {e['pixel']}: NO HIT")
            continue
        face, u, v, t, n_local = best
        depth = M(t, F(-dir_local[2]))
        depth_bits = f2bits(depth)
        dump_bits = int(e["depth"]["reference_bits"])

        # upstream: normalize(mat @ cross); product: normalize(mat @ normalize(cross))
        n_up = normalize3(mat_vec(geom_mat, n_local))
        n_prod = normalize3(mat_vec(geom_mat, normalize3(n_local)))

        def final_normal(n_world):
            face_vec = mat_t_vec(geom_mat, n_world)
            items = []
            for i in face_normal_tbl[face]:
                vn = [F(x) for x in mesh_normal[i]]
                items.append(face_vec if dot_pinned(vn, face_vec) < F(0.8) else vn)
            w0 = F(1) - u - v
            vec = [A(A(M(items[0][k], u), M(items[1][k], v)), M(items[2][k], w0))
                   for k in range(3)]
            return normalize3(mat_vec(geom_mat, vec))

        fin_up = final_normal(n_up)
        fin_prod = final_normal(n_prod)
        dots = [float(dot_pinned([F(x) for x in mesh_normal[i]],
                                 mat_t_vec(geom_mat, n_up))) for i in face_normal_tbl[face]]
        print(f"pixel {e['pixel']} world {w} cam {e['cam']} x {e['x']} y {e['y']}")
        print(f"  hit face {face} u {float(u):.9g} v {float(v):.9g} t {float(t):.9g}")
        print(f"  depth replay bits {depth_bits} dump bits {dump_bits} equal {depth_bits == dump_bits}")
        print(f"  vertex_normal dots {['%.4f' % d for d in dots]} (<0.8 -> face used)")
        print(f"  pre-normal ULP diff {[ulp(n_up[k], n_prod[k]) for k in range(3)]}")
        print(f"  final-normal ULP diff {[ulp(fin_up[k], fin_prod[k]) for k in range(3)]}")
        print(f"  product bytes {e['product']['bytes']} reference bytes {e['reference']['bytes']}")
        print("")


if __name__ == "__main__":
    main()
