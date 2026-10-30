"""QA independent re-enumeration for the sphere `a` order fix (read-only).

Run (from the repo root):
    target\\mjwarp-parallel\\venv\\Scripts\\python.exe \
        docs\\development-evidence\\render-bvh\\sphere-dot-a-order-fix-check\\materials\\recompute-rca-variants.py

It reuses the f32 primitives and the diagnostic parser of the (uncommitted) RCA script
target/rca/sphere_depth_rca.py, but performs its own full enumeration over
6^3 dot-order combinations x {'fused','nonfused'} det = 432 variants and scores each variant
against the product bytes and the reference bytes of the 44 diagnosed pixels.

Expected (2026-10-31):
    total variants: 432
    max ref matches: 44
    winners(44/44): 2
      ((1, 0, 2), (1, 0, 2), (0, 1, 2), 'fused')  product_matches= 0
      ((1, 0, 2), (1, 0, 2), (1, 0, 2), 'fused')  product_matches= 0
    product-before-fix variant ((0,1,2),(1,0,2),(1,0,2),'fused') -> (44, 0)
    fixed variant ((1,0,2),(1,0,2),(1,0,2),'fused') -> (0, 44)
    next after winners: 37/44
"""

from __future__ import annotations

import importlib.util
import itertools
import json
import pathlib
import sys

import numpy as np


def repo_root() -> pathlib.Path:
    here = pathlib.Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "Cargo.toml").exists():
            return parent
    raise SystemExit("repo root (Cargo.toml) not found above this script")


ROOT = repo_root()
HERE = ROOT / "target" / "rca"
spec = importlib.util.spec_from_file_location("rca3", HERE / "sphere_depth_rca.py")
rca = importlib.util.module_from_spec(spec)
sys.modules["rca3"] = rca
spec.loader.exec_module(rca)

f32 = np.float32
MUL, FMA = rca.mul32, rca.fma32


def dot_order(parts, order):
    acc = MUL(parts[order[0]][0], parts[order[0]][1])
    for index in order[1:]:
        acc = FMA(parts[index][0], parts[index][1], acc)
    return acc


def sphere_variant(origin, vec, center, r2, a_order, b_order, det_mode, dd_order):
    d = [rca.sub32(origin[i], center[i]) for i in range(3)]
    a = dot_order([(vec[i], vec[i]) for i in range(3)], a_order)
    b = dot_order([(vec[i], d[i]) for i in range(3)], b_order)
    dd = dot_order([(d[i], d[i]) for i in range(3)], dd_order)
    c = rca.sub32(dd, r2)
    if det_mode == "fused":
        det = FMA(b, b, rca.sub32(f32(0.0), MUL(a, c)))
    else:
        det = rca.sub32(MUL(b, b), MUL(a, c))
    if det < 1.0e-15:
        return f32(-1.0)
    sq = f32(np.sqrt(np.float64(det)))
    den = f32(np.float64(1.0) / np.float64(a))
    x0 = MUL(den, rca.sub32(rca.sub32(f32(0.0), b), sq))
    x1 = MUL(den, rca.sub32(sq, b))
    return x0 if x0 >= 0 else (x1 if x1 >= 0 else f32(-1.0))


def main() -> int:
    model = json.loads(rca.DUMP.read_text(encoding="utf-8"))
    arrays = model["arrays"]
    cam_pos = arrays["cam_pos"]["values"]
    cam_quat = arrays["cam_quat"]["values"]
    geom_type = arrays["geom_type"]["values"]
    geom_size = arrays["geom_size"]["values"]
    geom_pos = arrays["geom_pos"]["values"]
    sphere_index = [i for i, t in enumerate(geom_type) if t == 2][0]
    center = np.asarray([f32(geom_pos[3 * sphere_index + k]) for k in range(3)], dtype=np.float32)
    r2 = MUL(f32(geom_size[3 * sphere_index]), f32(geom_size[3 * sphere_index]))

    rows = rca.parse_diag(limit=50)
    pixels = []
    for row in rows:
        cam_index = 0 if row["cam"] == "cam0" else 1
        origin = np.asarray([f32(cam_pos[3 * cam_index + k]) for k in range(3)], dtype=np.float32)
        local_dir = np.asarray(row["dir"], dtype=np.float32)
        quat = cam_quat[4 * cam_index: 4 * cam_index + 4]
        mat = rca.warp_quat_to_mat(quat)
        world = np.asarray(rca.mat_vec_fma_2_0_1(mat, local_dir), dtype=np.float32)
        pixels.append((origin, world, local_dir, row["p_bits"], row["r_bits"]))

    total = len(pixels)
    orders = list(itertools.permutations(range(3)))
    results = {}
    for a_order, b_order, dd_order, det_mode in itertools.product(
        orders, orders, orders, ("fused", "nonfused")
    ):
        product_hits = ref_hits = 0
        for origin, world, local_dir, p_bits, r_bits in pixels:
            t = sphere_variant(origin, world, center, r2, a_order, b_order, det_mode, dd_order)
            depth = f32(0.0) if t < 0 else MUL(t, rca.sub32(f32(0.0), local_dir[2]))
            if rca.bits(depth) == p_bits:
                product_hits += 1
            if rca.bits(depth) == r_bits:
                ref_hits += 1
        results[(a_order, b_order, dd_order, det_mode)] = (product_hits, ref_hits)

    print(f"pixels: {total}")
    print(f"total variants: {len(results)}")
    winners = [k for k, v in results.items() if v[1] == total]
    print(f"winners({total}/{total}): {len(winners)}")
    for w in winners:
        print(f"  a{w[0]} b{w[1]} dd{w[2]} det={w[3]} product_matches={results[w][0]}")
    for name, key in (
        ("product-before-fix", ((0, 1, 2), (1, 0, 2), (1, 0, 2), "fused")),
        ("fixed-code", ((1, 0, 2), (1, 0, 2), (1, 0, 2), "fused")),
    ):
        print(f"{name} variant -> product_matches,ref_matches = {results.get(key)}")
    ranked = sorted(((v[1], k) for k, v in results.items()), reverse=True)
    print("next after winners:", [(s, k) for s, k in ranked if s < total][:3])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
