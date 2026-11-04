"""Classify the single residual flex pixel (134,128) after the mat_t_vec/mat_vec fix.

Runs offline: reconstructs the three flex-0 capsules for that pixel's ray and compares each
edge's f32 hit depth against the reference and product depth bits.
Read-only; writes target/s3b-rca/.
"""
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
F32 = np.float32
MINVAL = F32(1.0e-15)


def fma(a, b, c):
    return F32(np.float64(a) * np.float64(b) + np.float64(c))


def mul(a, b):
    return F32(F32(a) * F32(b))


def dot3(a, b):
    t1 = mul(a[1], b[1])
    return fma(a[2], b[2], fma(a[0], b[0], t1))


def dot2(a, b):
    t1 = mul(a[1], b[1])
    return fma(a[0], b[0], t1)


def mat_i(m, j):
    return [m[0 * 3 + j], m[1 * 3 + j], m[2 * 3 + j]]


def mat_from_quat(q):
    x, y, z, w = [F32(t) for t in q]
    q00, q01, q02, q03 = mul(x, x), mul(x, y), mul(x, z), mul(x, w)
    q11, q12, q13 = mul(y, y), mul(y, z), mul(y, w)
    q22, q23, q33 = mul(z, z), mul(z, w), mul(w, w)
    m = np.empty(9, dtype=np.float32)
    m[0] = F32(F32(F32(q00 + q11) - q22) - q33)
    m[1] = mul(F32(2.0), F32(q12 - q03))
    m[2] = mul(F32(2.0), F32(q13 + q02))
    m[3] = mul(F32(2.0), F32(q12 + q03))
    m[4] = F32(F32(F32(q00 - q11) + q22) - q33)
    m[5] = mul(F32(2.0), F32(q23 - q01))
    m[6] = mul(F32(2.0), F32(q13 - q02))
    m[7] = mul(F32(2.0), F32(q23 + q01))
    m[8] = F32(F32(F32(q00 - q11) - q22) + q33)
    return m


def quat_z2vec(vec):
    q = np.array([0.0, 0.0, 0.0, 1.0], dtype=np.float32)
    norm = F32(np.sqrt(np.float64(dot3(vec, vec))))
    if norm < MINVAL:
        return q
    u = np.array([F32(vec[i] / norm) for i in range(3)], dtype=np.float32)
    axis = np.array([F32(-u[1]), u[0], F32(0.0)], dtype=np.float32)
    a = F32(np.sqrt(np.float64(dot3(axis, axis))))
    if a < MINVAL:
        if u[2] < F32(0.0):
            q = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
        return q
    ax = np.array([F32(axis[i] / a) for i in range(3)], dtype=np.float32)
    angle = F32(np.arctan2(np.float64(a), np.float64(u[2])))  # approximation
    half = F32(angle * F32(0.5))
    s = F32(np.sin(np.float64(half)))
    c = F32(np.cos(np.float64(half)))
    return np.array([c, F32(ax[0] * s), F32(ax[1] * s), F32(ax[2] * s)], dtype=np.float32)


def ray_quad(a, b, c):
    det = fma(b, b, F32(-mul(a, c)))
    if det < MINVAL:
        return F32(-1.0), F32(-1.0), F32(-1.0)
    det = F32(np.sqrt(np.float64(det)))
    den = F32(F32(1.0) / a) if a != F32(0.0) else F32(F32(1.0) / MINVAL)
    nb = F32(-b)
    x0 = mul(F32(nb - det), den)
    x1 = mul(F32(nb + det), den)
    sol = x0 if x0 >= F32(0.0) else (x1 if x1 >= F32(0.0) else F32(-1.0))
    return sol, x0, x1


def capsule_hit(pos, mat, size, origin, dirw):
    ssz = F32(size[0] + size[1])
    dist_sqr = mul(ssz, ssz)
    dif = np.array([F32(origin[i] - pos[i]) for i in range(3)], dtype=np.float32)
    a_sp = dot3(dirw, dirw)
    b_sp = dot3(dirw, dif)
    c_sp = F32(dot3(dif, dif) - dist_sqr)
    sol, _, _ = ray_quad(a_sp, b_sp, c_sp)
    if sol < F32(0.0):
        return F32(-1.0)
    lp = np.array([dot3(mat_i(mat, j), dif) for j in range(3)], dtype=np.float32)
    lv = np.array([dot3(mat_i(mat, j), dirw) for j in range(3)], dtype=np.float32)
    x = F32(-1.0)
    sq = mul(size[0], size[0])
    a = dot2([lv[0], lv[1]], [lv[0], lv[1]])
    b = dot2([lv[0], lv[1]], [lp[0], lp[1]])
    c = F32(dot2([lp[0], lp[1]], [lp[0], lp[1]]) - sq)
    sol, x0, x1 = ray_quad(a, b, c)
    if sol >= F32(0.0) and abs(F32(lp[2] + mul(sol, lv[2]))) <= size[1]:
        if x < F32(0.0) or sol < x:
            x = sol
    ldif = np.array([lp[0], lp[1], F32(lp[2] - size[1])], dtype=np.float32)
    a2 = F32(a + mul(lv[2], lv[2]))
    b2 = dot3(lv, ldif)
    c2 = F32(dot3(ldif, ldif) - sq)
    sol, x0, x1 = ray_quad(a2, b2, c2)
    for xx in (x0, x1):
        if xx >= F32(0.0) and F32(lp[2] + mul(xx, lv[2])) >= size[1] and (x < F32(0.0) or xx < x):
            x = xx
    ldif2 = np.array([ldif[0], ldif[1], F32(lp[2] + size[1])], dtype=np.float32)
    b3 = dot3(lv, ldif2)
    c3 = F32(dot3(ldif2, ldif2) - sq)
    sol, x0, x1 = ray_quad(a2, b3, c3)
    for xx in (x0, x1):
        if xx >= F32(0.0) and F32(lp[2] + mul(xx, lv[2])) <= F32(-size[1]) and (x < F32(0.0) or xx < x):
            x = xx
    return x


def main():
    X, Y = 134, 128
    W = 320
    idx = Y * W + X
    rays = np.frombuffer((ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order/cam2-ray.f32le").read_bytes(), dtype="<f4").reshape(-1, 6)
    dir_local = rays[idx, :3].astype(np.float32)
    dirz = dir_local[2]
    ref_depth = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].reshape(-1).astype(np.float32)
    prod_depth = np.frombuffer((ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order/residual/cam2-depth.f32le").read_bytes(), dtype="<f4").astype(np.float32)
    state = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/state.npz")
    fv = state["flexvert_xpos"][0].astype(np.float32)
    poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
    f32of = lambda b: np.uint32(b).view(np.float32)
    cam_pos = np.array([f32of(b) for b in poses["poses"]["cam_xpos_bits"][6:9]], dtype=np.float32)
    cam_mat = np.array([f32of(b) for b in poses["poses"]["cam_xmat_bits"][18:27]], dtype=np.float32).reshape(3, 3)
    dirw = np.array([dot3([cam_mat[j][k] for k in range(3)], dir_local) for j in range(3)], dtype=np.float32)

    radius = np.array([0.02, 0.04], dtype=np.float32)
    edges = [(0, 1), (1, 2), (2, 3)]
    rbits = int(ref_depth[idx].view(np.uint32))
    pbits = int(prod_depth[idx].view(np.uint32))
    print(f"pixel ({X},{Y}) idx={idx} dir_local={dir_local} dirz={dirz!r}")
    print(f"ref_depth={ref_depth[idx]!r} bits={rbits:#010x}; prod_depth={prod_depth[idx]!r} bits={pbits:#010x}; ULP={abs(pbits-rbits)}")
    for e, (a0, a1) in enumerate(edges):
        v0, v1 = fv[a0], fv[a1]
        pos = np.array([mul(F32(v0[i] + v1[i]), F32(0.5)) for i in range(3)], dtype=np.float32)
        vec = np.array([F32(v1[i] - v0[i]) for i in range(3)], dtype=np.float32)
        length = F32(np.sqrt(np.float64(dot3(vec, vec))))
        quat = quat_z2vec(vec)
        mat = mat_from_quat(quat)
        size = np.array([radius[0], F32(F32(0.5) * length), 0.0], dtype=np.float32)
        d = capsule_hit(pos, mat, size, cam_pos, dirw)
        depth = mul(d, F32(-dirz))
        dbits = int(np.float32(depth).view(np.uint32))
        print(f"edge{e} verts({a0},{a1}) d={d!r} depth={depth!r} bits={dbits:#010x} "
              f"| vs ref ULP={abs(dbits-rbits)} vs prod ULP={abs(dbits-pbits)}")


if __name__ == "__main__":
    main()
