"""Offline exhaustive contraction enumeration of the 1D flex capsule hit chain.

Oracle inputs come from upstream-flex1-chain.npz (warp GPU replay that reproduces the
reference cam2 depth bit-exactly for all 251 flex-1 pixels). We rebuild the capsule
matrix from quat, then enumerate plausible NVRTC contraction variants of the
sum-of-products / quadratic expressions, and check which variant reproduces
(a) the reference depth bits and (b) the product depth bits.

Read-only; writes target/s3b-rca/.
"""
import itertools
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target" / "s3b-rca"
F32 = np.float32
MINVAL = F32(1.0e-15)


def fma(a, b, c):
    return F32(np.float64(a) * np.float64(b) + np.float64(c))


def mul(a, b):
    return F32(F32(a) * F32(b))


def dot3(a, b, rounded):
    t = [mul(a[i], b[i]) for i in range(3)]
    s = fma(a[1], b[1], t[0]) if rounded == 0 else (
        fma(a[0], b[0], t[1]) if rounded == 1 else fma(a[0], b[0], t[2]))
    if rounded == 0:
        return fma(a[2], b[2], s)
    if rounded == 1:
        return fma(a[2], b[2], s)
    return fma(a[1], b[1], s)


def dot2(a, b, rounded):
    t0 = mul(a[0], b[0])
    t1 = mul(a[1], b[1])
    return fma(a[0], b[0], t1) if rounded == 1 else fma(a[1], b[1], t0)


def det_of(a, b, c, variant):
    if variant == 0:
        return F32(mul(b, b) - mul(a, c))
    if variant == 1:
        return fma(b, b, F32(-mul(a, c)))
    return fma(F32(-a), c, mul(b, b))


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


def ray_quad(a, b, c, cfg):
    det = det_of(a, b, c, cfg["det"])
    if det < MINVAL:
        return F32(-1.0), F32(-1.0), F32(-1.0)
    det = F32(np.sqrt(np.float64(det)))
    den = F32(F32(1.0) / a) if a != F32(0.0) else F32(F32(1.0) / MINVAL)
    if cfg["root"] == 0:
        nb = F32(-b)
        x0 = mul(F32(nb - det), den)
        x1 = mul(F32(nb + det), den)
    else:
        x0 = mul(F32(F32(0.0) - F32(b + det)), den)
        x1 = mul(F32(det - b), den)
    sol = x0 if x0 >= F32(0.0) else (x1 if x1 >= F32(0.0) else F32(-1.0))
    return sol, x0, x1


def chain(pos, mat, size, origin, dirw, cfg):
    ssz = F32(size[0] + size[1])
    dist_sqr = mul(ssz, ssz)
    dif = np.array([F32(origin[i] - pos[i]) for i in range(3)], dtype=np.float32)
    a_sp = dot3(dirw, dirw, cfg["d3"])
    b_sp = dot3(dirw, dif, cfg["d3"])
    c_sp = F32(dot3(dif, dif, cfg["d3"]) - dist_sqr)
    sol, x0_sp, x1_sp = ray_quad(a_sp, b_sp, c_sp, cfg)
    if sol < F32(0.0):
        return F32(-1.0)
    # ray_map: local frame
    lp = np.array([dot3([mat[k * 3 + j] for k in range(3)], dif, cfg["d3m_lp"]) for j in range(3)], dtype=np.float32)
    lv = np.array([dot3([mat[k * 3 + j] for k in range(3)], dirw, cfg["d3m_lv"]) for j in range(3)], dtype=np.float32)
    x = F32(-1.0)
    sq = mul(size[0], size[0])
    a = dot2([lv[0], lv[1]], [lv[0], lv[1]], cfg["d2"])
    b = dot2([lv[0], lv[1]], [lp[0], lp[1]], cfg["d2"])
    c = F32(dot2([lp[0], lp[1]], [lp[0], lp[1]], cfg["d2"]) - sq)
    sol, x0, x1 = ray_quad(a, b, c, cfg)
    if sol >= F32(0.0) and abs(F32(lp[2] + mul(sol, lv[2]))) <= size[1]:
        if x < F32(0.0) or sol < x:
            x = sol
    ldif0 = np.array([lp[0], lp[1], F32(lp[2] - size[1])], dtype=np.float32)
    if cfg["add"] == 0:
        a2 = F32(a + mul(lv[2], lv[2]))
    else:
        a2 = fma(lv[2], lv[2], a)
    b2 = dot3(lv, ldif0, cfg["d3"])
    c2 = F32(dot3(ldif0, ldif0, cfg["d3"]) - sq)
    sol, x0, x1 = ray_quad(a2, b2, c2, cfg)
    for xx in (x0, x1):
        if xx >= F32(0.0) and F32(lp[2] + mul(xx, lv[2])) >= size[1] and (x < F32(0.0) or xx < x):
            x = xx
    ldif1 = np.array([ldif0[0], ldif0[1], F32(lp[2] + size[1])], dtype=np.float32)
    b3 = dot3(lv, ldif1, cfg["d3"])
    c3 = F32(dot3(ldif1, ldif1, cfg["d3"]) - sq)
    sol, x0, x1 = ray_quad(a2, b3, c3, cfg)
    for xx in (x0, x1):
        if xx >= F32(0.0) and F32(lp[2] + mul(xx, lv[2])) <= F32(-size[1]) and (x < F32(0.0) or xx < x):
            x = xx
    return x


def run(cfg, data):
    dirs = data["dirw"]
    pos, mat, size = data["pos"], data["mat"], data["size"]
    origin = data["origin"]
    return np.array(
        [chain(pos, mat, size, origin, dirs[k].astype(np.float32), cfg) for k in range(len(dirs))],
        dtype=np.float32,
    )


def main():
    z = np.load(ROOT / "target/render-bvh/r06-s3b-flex-window/f32-order/upstream-flex1-chain.npz")
    v = z["values"].astype(np.float32)
    data = dict(
        pos=v[0, 0:3],
        dirw=v[:, 14:17].astype(np.float32),
        size=v[0, 11:14],
        origin=z["cam_xpos"].astype(np.float32),
    )
    data["mat"] = mat_from_quat(v[0, 7:11])
    ref_depth = z["ref_depth"].astype(np.float32)
    prod_depth = z["prod_depth"].astype(np.float32)
    dirz = z["dirs"].astype(np.float32)[:, 2]
    n = len(dirz)

    base = dict(d3=1, d3m_lp=1, d3m_lv=1, d2=1, det=1, add=0, root=0)
    grid = []
    for d3 in (1,):
        for d3m_lp in (0, 1, 2):
            for d3m_lv in (0, 1, 2):
                for d2 in (1,):
                    for det in (1,):
                        for add in (0, 1):
                            for root in (0, 1):
                                grid.append(dict(d3=d3, d3m_lp=d3m_lp, d3m_lv=d3m_lv, d2=d2, det=det, add=add, root=root))

    results = []
    for cfg in grid:
        d = run(cfg, data)
        depth = np.array([mul(d[k], F32(-dirz[k])) for k in range(n)], dtype=np.float32)
        ref_ok = int((depth.view(np.uint32) == ref_depth.view(np.uint32)).sum())
        prod_ok = int((depth.view(np.uint32) == prod_depth.view(np.uint32)).sum())
        results.append((ref_ok, prod_ok, cfg))
    results.sort(key=lambda t: (-t[0], -t[1]))
    print("variants:", len(results))
    for ref_ok, prod_ok, cfg in results[:12]:
        print(f"  ref={ref_ok}/{n} prod={prod_ok}/{n}  {cfg}")
    # best for product
    by_prod = sorted(results, key=lambda t: -t[1])[:8]
    print("--- best for PRODUCT ---")
    for ref_ok, prod_ok, cfg in by_prod:
        print(f"  ref={ref_ok}/{n} prod={prod_ok}/{n}  {cfg}")
    np.savez(OUT / "contract-enum.npz", results=np.array([[a, b] for a, b, _ in results], dtype=np.int32))
    print("base cfg results:", [r for r in results if r[2] == base])


if __name__ == "__main__":
    main()
