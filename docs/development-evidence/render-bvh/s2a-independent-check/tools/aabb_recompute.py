"""Independent QA recomputation of the S2a leaf AABB reference (56 leaves).

Reimplements the frozen upstream MJWarp formulas (`_src/bvh.py`) in Python with
explicit f32 steps and the documented NVRTC fma contraction order, then compares
lower/upper bits with fixtures/render-bvh/scene-a/s2a/bounds.json. Also checks the
model-side inputs (geom_size, mesh bounds) against model-312.json.
"""

from __future__ import annotations

import json
import pathlib
import struct
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[2]
FIX = ROOT / "fixtures/render-bvh/scene-a/s2a/bounds.json"
REF = ROOT / "fixtures/render-bvh/scene-a/model-312.json"
OUT = ROOT / "docs/development-evidence/render-bvh/s2a-independent-check/aabb-recompute.json"

F32 = np.float32


def bits_to_f32(bits: int) -> np.float32:
    return np.frombuffer(struct.pack("<I", int(bits)), dtype=np.float32)[0]


def f32_bits(value: np.float32) -> int:
    return int(np.frombuffer(struct.pack("<f", np.float32(value)), dtype=np.uint32)[0])


def fma(a, b, c):
    """f32 fma: exact f32*f32 product (fits in f64) + f32 addend, single f32 round."""
    return F32(np.float64(F32(a)) * np.float64(F32(b)) + np.float64(F32(c)))


def mul(a, b):
    return F32(F32(a) * F32(b))


def add(a, b):
    return F32(F32(a) + F32(b))


def sub(a, b):
    return F32(F32(a) - F32(b))


def mat_vec(rot, vec):
    """warp mat33 @ vec3 with documented contraction: fma(c2,b2, fma(c0,b0, round(c1*b1)))."""
    out = []
    for r in range(3):
        c0, c1, c2 = F32(rot[r][0]), F32(rot[r][1]), F32(rot[r][2])
        b0, b1, b2 = F32(vec[0]), F32(vec[1]), F32(vec[2])
        out.append(fma(c2, b2, fma(c0, b0, mul(c1, b1))))
    return out


def dot3_fma(u, v):
    return fma(F32(u[2]), F32(v[2]), fma(F32(u[0]), F32(v[0]), mul(F32(u[1]), F32(v[1]))))


def length3(v):
    return F32(np.sqrt(np.float64(dot3_fma(v, v))))


def component_min_max(corners):
    lo = [min(c[i] for c in corners) for i in range(3)]
    hi = [max(c[i] for c in corners) for i in range(3)]
    return [F32(x) for x in lo], [F32(x) for x in hi]


def box_bounds(pos, rot, size):
    corners = []
    for i in (0, 1):
        for j in (0, 1):
            for k in (0, 1):
                local = [
                    mul(F32(size[0]), F32(2.0 * i - 1.0)),
                    mul(F32(size[1]), F32(2.0 * j - 1.0)),
                    mul(F32(size[2]), F32(2.0 * k - 1.0)),
                ]
                world = mat_vec(rot, local)
                corners.append([add(pos[t], world[t]) for t in range(3)])
    return component_min_max(corners)


def plane_bounds(pos, rot, size):
    scale = mul(max(F32(size[0]), F32(size[1])), F32(2.0))
    if size[0] <= 0.0 or size[1] <= 0.0:
        scale = F32(1000.0)
    corners = []
    for i in (0, 1):
        for j in (0, 1):
            local = [mul(scale, F32(2.0 * i - 1.0)), mul(scale, F32(2.0 * j - 1.0)), F32(0.0)]
            world = mat_vec(rot, local)
            corners.append([add(pos[t], world[t]) for t in range(3)])
    lo, hi = component_min_max(corners)
    return [sub(lo[t], F32(0.01)) for t in range(3)], [add(hi[t], F32(0.01)) for t in range(3)]


def sphere_bounds(pos, rot, size):
    r = F32(size[0])
    return [sub(pos[t], r) for t in range(3)], [add(pos[t], r) for t in range(3)]


def capsule_bounds(pos, rot, size):
    radius, half = F32(size[0]), F32(size[1])
    z = [F32(rot[0][2]), F32(rot[1][2]), F32(rot[2][2])]
    end1 = [fma(F32(-z[t]), half, pos[t]) for t in range(3)]
    end2 = [fma(z[t], half, pos[t]) for t in range(3)]
    lo = [min(end1[t], end2[t]) for t in range(3)]
    hi = [max(end1[t], end2[t]) for t in range(3)]
    return [sub(lo[t], radius) for t in range(3)], [add(hi[t], radius) for t in range(3)]


def cylinder_bounds(pos, rot, size):
    radius, half = F32(size[0]), F32(size[1])
    axis = [F32(rot[0][2]), F32(rot[1][2]), F32(rot[2][2])]
    axis_abs = [abs(axis[t]) for t in range(3)]
    bx = [F32(rot[0][0]), F32(rot[1][0]), F32(rot[2][0])]
    by = [F32(rot[0][1]), F32(rot[1][1]), F32(rot[2][1])]
    radial = []
    for t in range(3):
        s = fma(bx[t], bx[t], mul(by[t], by[t]))
        radial.append(mul(radius, F32(np.sqrt(np.float64(s)))))
    extent = [fma(half, axis_abs[t], radial[t]) for t in range(3)]
    return [sub(pos[t], extent[t]) for t in range(3)], [add(pos[t], extent[t]) for t in range(3)]


def ellipsoid_bounds(pos, rot, size):
    extent = []
    for r in range(3):
        row = [mul(F32(rot[r][0]), F32(size[0])), mul(F32(rot[r][1]), F32(size[1])), mul(F32(rot[r][2]), F32(size[2]))]
        extent.append(length3(row))
    return [sub(pos[t], extent[t]) for t in range(3)], [add(pos[t], extent[t]) for t in range(3)]


TYPES = {0: "plane", 2: "sphere", 3: "capsule", 4: "ellipsoid", 5: "cylinder", 6: "box", 7: "mesh"}


def main() -> int:
    fix = json.loads(FIX.read_text(encoding="utf-8"))
    model = fix["model"]
    ngeom = model["ngeom"]
    geom_type = model["geom_type"]
    geom_dataid = model["geom_dataid"]
    size_bits = model["geom_size_bits"]
    mesh_bounds_bits = model["mesh_bounds_size_bits"]
    sizes = [[bits_to_f32(size_bits[3 * i + k]) for k in range(3)] for i in range(ngeom)]
    mesh_bounds = {int(k): [bits_to_f32(v[k2]) for k2 in range(3)] for k, v in mesh_bounds_bits.items()}

    rows = []
    mismatches = 0
    max_ulp = 0
    for case in fix["cases"]:
        pos_bits = case["inputs"]["geom_xpos_bits"]
        mat_bits = case["inputs"]["geom_xmat_bits"]
        lower_ref = case["lower_bits"]
        upper_ref = case["upper_bits"]
        for g in range(ngeom):
            pos = [bits_to_f32(pos_bits[3 * g + t]) for t in range(3)]
            rot = [[bits_to_f32(mat_bits[9 * g + 3 * r + c]) for c in range(3)] for r in range(3)]
            kind = TYPES[geom_type[g]]
            size = sizes[g]
            if kind == "mesh":
                size = mesh_bounds[geom_dataid[g]]
                lo, hi = box_bounds(pos, rot, size)
            elif kind == "box":
                lo, hi = box_bounds(pos, rot, size)
            elif kind == "plane":
                lo, hi = plane_bounds(pos, rot, size)
            elif kind == "sphere":
                lo, hi = sphere_bounds(pos, rot, size)
            elif kind == "capsule":
                lo, hi = capsule_bounds(pos, rot, size)
            elif kind == "cylinder":
                lo, hi = cylinder_bounds(pos, rot, size)
            elif kind == "ellipsoid":
                lo, hi = ellipsoid_bounds(pos, rot, size)
            else:
                raise SystemExit(f"unsupported geom type {geom_type[g]}")
            row_diff = 0
            for t in range(3):
                for ref_value, computed in ((lower_ref[3 * g + t], lo[t]), (upper_ref[3 * g + t], hi[t])):
                    if f32_bits(computed) != int(ref_value):
                        row_diff += 1
                        mismatches += 1
                        ub = int(ref_value)
                        cb = f32_bits(computed)
                        max_ulp = max(max_ulp, abs(ub - cb))
            rows.append({"case": case["name"], "geom": g, "type": kind, "bit_mismatches": row_diff})

    # model-side input check: geom_size f64 -> f32 vs fixture bits, mesh bounds from mesh_vert
    ref = json.loads(REF.read_text(encoding="utf-8"))["arrays"]
    geom_size_ref = [F32(v) for v in ref["geom_size"]["values"]]
    size_input_diffs = sum(1 for i in range(len(geom_size_ref)) if f32_bits(geom_size_ref[i]) != int(size_bits[i]))
    mesh_vert = np.array(ref["mesh_vert"]["values"], dtype=np.float64).reshape(-1, 3)
    pmin = mesh_vert.min(axis=0)
    pmax = mesh_vert.max(axis=0)
    half = np.maximum(np.abs(pmin), np.abs(pmax))
    half32 = [F32(h) for h in half]
    mesh_bounds_fix = mesh_bounds[0]
    mesh_input_diffs = sum(1 for t in range(3) if f32_bits(half32[t]) != f32_bits(mesh_bounds_fix[t]))

    leaves = len(rows)
    report = {
        "task_id": "2026-10-24-render-bvh-s2a-independent-check",
        "fixture": "fixtures/render-bvh/scene-a/s2a/bounds.json",
        "method": "independent Python reimplementation of upstream _src/bvh.py bounds formulas with f32 steps and documented fma contraction; compared to fixture bits",
        "cases": len(fix["cases"]),
        "leaves": leaves,
        "leaf_aabbs_expected": 56,
        "bit_mismatches": mismatches,
        "max_ulp": max_ulp,
        "rows_with_mismatch": [r for r in rows if r["bit_mismatches"]],
        "inputs": {
            "geom_size_f64_to_f32_diffs": size_input_diffs,
            "mesh_bounds_diffs": mesh_input_diffs,
        },
        "rows": rows,
        "verdict": "pass" if mismatches == 0 and leaves == 56 and size_input_diffs == 0 and mesh_input_diffs == 0 else "fail",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({k: report[k] for k in ("verdict", "leaves", "bit_mismatches", "max_ulp", "rows_with_mismatch", "inputs")}, indent=2))
    return 0 if report["verdict"] == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
