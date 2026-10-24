"""S3 authoring helper: static replay of upstream `ray_mesh_with_bvh` queries.

OFFLINE ONLY: imported by `capture_s3_mesh.py`, never by product builds/tests/gates.

The product mesh kernel is f32-only because the product SynchronousKernel path
enforces an f32-only PTX gate. Upstream's Woop triangle routine has one
double-precision fallback branch (when a barycentric determinant is exactly zero).
This replay emulates the device arithmetic (NVRTC fmad=true: `a - s*b` is a single
`fma(-s, b, a)`, `diff_product` uses two fmas) and additionally replays the upstream
fallback (exact products of the same f32 values, computed in f64) to count where the
f64 branch is reached and whether it would change the accepted hit.

The replay is a *static check only*: its numeric results are not compared bitwise
with the device; the reference comes from Warp itself (`capture_s3_mesh.py`).
"""

from __future__ import annotations

import numpy as np


def _f32(value) -> np.float32:
    return np.float32(value)


def _fma32(a, b, c) -> np.float32:
    # Correctly rounded f32 FMA: exact f64 product + sum, single rounding.
    return np.float32(np.float64(a) * np.float64(b) + np.float64(c))


def _diff_product32(a, b, c, d) -> np.float32:
    a, b, c, d = (_f32(value) for value in (a, b, c, d))
    cd = _f32(c * d)
    return _f32(_fma32(a, b, -cd) + _fma32(-c, d, cd))


def _fallback_product(a, b, c, d) -> np.float32:
    # Upstream: (float)((double)a*(double)b - (double)c*(double)d).
    return np.float32(np.float64(a) * np.float64(b) - np.float64(c) * np.float64(d))


def _woop(start, direction, v0, v1, v2):
    """Replay intersect_ray_tri_woop.

    Returns (hit, t, f64_branch, fallback_changed): `fallback_changed` is true when
    the upstream f64 fallback would flip the accept/reject decision relative to the
    pure-f32 computation (the product kernel has no f64 branch).
    """
    d = [_f32(value) for value in direction]
    kz = 0
    if abs(d[1]) > abs(d[0]):
        kz = 1
    if abs(d[2]) > abs(d[kz]):
        kz = 2
    kx = 0 if kz == 2 else kz + 1
    ky = 0 if kx == 2 else kx + 1
    if d[kz] < 0.0:
        kx, ky = ky, kx
    with np.errstate(all="ignore"):
        sx = _f32(d[kx] / d[kz])
        sy = _f32(d[ky] / d[kz])
        sz = _f32(_f32(1.0) / d[kz])
        rel = []
        for vertex in (v0, v1, v2):
            dif = [_f32(np.float64(vertex[i]) - np.float64(start[i])) for i in range(3)]
            # NVRTC fmad=true contracts `A[kx] - Sx * A[kz]` into one fma.
            rel.append((_fma32(-sx, dif[kz], dif[kx]), _fma32(-sy, dif[kz], dif[ky]), dif[kz]))
        (ax, ay, az), (bx, by, bz), (cx, cy, cz) = rel
        u = _diff_product32(cx, by, cy, bx)
        v = _diff_product32(ax, cy, ay, cx)
        w = _diff_product32(bx, ay, by, ax)
        branch = bool(u == 0.0 or v == 0.0 or w == 0.0)
        result_no_fallback = _finish(u, v, w, sx, sy, sz, az, bz, cz)
        if branch:
            u2 = _fallback_product(cx, by, cy, bx)
            v2 = _fallback_product(ax, cy, ay, cx)
            w2 = _fallback_product(bx, ay, by, ax)
            result_fallback = _finish(u2, v2, w2, sx, sy, sz, az, bz, cz)
            fallback_changed = result_fallback[0] != result_no_fallback[0]
            return result_fallback[0], result_fallback[1], True, fallback_changed
        return result_no_fallback[0], result_no_fallback[1], False, False


def _finish(u, v, w, sx, sy, sz, az, bz, cz):
    with np.errstate(all="ignore"):
        if (u < 0 or v < 0 or w < 0) and (u > 0 or v > 0 or w > 0):
            return False, _f32(0.0)
        det = _f32(u + v + w)
        if det == 0.0:
            return False, _f32(0.0)
        total = _f32(_f32(u * _f32(sz * az)) + _f32(v * _f32(sz * bz)) + _f32(w * _f32(sz * cz)))
        det_sign = int(np.int32(det.view(np.int32))) & -2147483648
        if (int(np.int32(total.view(np.int32))) ^ det_sign) < 0:
            return False, _f32(0.0)
        t = _f32(total * _f32(_f32(1.0) / det))
        return bool(t >= 0.0), t


def _normal(v0, v1, v2):
    normal = np.cross(v1 - v0, v2 - v0)
    length = _f32(np.sqrt(np.sum(np.asarray(normal, np.float32) ** 2)))
    return np.asarray(normal / length, np.float32) if length > 0 else np.zeros(3, np.float32)


def replay(points, indices, pnt, direction, max_t, cull) -> dict:
    """Replay every ray (including cull marches) and report branch statistics."""
    vertices = np.asarray(points, dtype=np.float32)
    triangles = np.asarray(indices, dtype=np.int32).reshape(-1, 3)
    branch_queries = 0
    fallback_changed = 0
    accepted = 0
    for ray in range(len(pnt)):
        lpnt = [_f32(value) for value in pnt[ray]]
        lvec = [_f32(value) for value in direction[ray]]
        offset = _f32(0.0)
        with np.errstate(all="ignore"):
            for _ in range(8):
                start = [_f32(lpnt[i] + _f32(lvec[i] * offset)) for i in range(3)]
                min_t = _f32(_f32(max_t[ray]) - offset)
                best = None
                for tri in triangles:
                    v0, v1, v2 = (vertices[int(index)] for index in tri)
                    hit, t, branch, changed = _woop(start, lvec, v0, v1, v2)
                    if branch:
                        branch_queries += 1
                    if changed:
                        fallback_changed += 1
                    if hit and t < min_t:
                        min_t = t
                        best = (t, _normal(v0, v1, v2))
                if best is None:
                    break
                dot = _f32(
                    _f32(_f32(lvec[0] * best[1][0]) + _f32(lvec[1] * best[1][1]))
                    + _f32(lvec[2] * best[1][2])
                )
                if not cull or dot < 0.0:
                    accepted += 1
                    break
                offset = _f32(offset + _f32(best[0] * _f32(1.0 + 1.0e-6)) + _f32(1.0e-9))
    return {
        "rays": int(len(pnt)),
        "accepted": accepted,
        "degenerate_woop_queries": int(branch_queries),
        "fallback_decision_changes": int(fallback_changed),
    }
