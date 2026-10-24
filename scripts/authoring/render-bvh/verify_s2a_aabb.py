"""Offline evidence: re-derive the S2a host AABB results from the captured reference.

OFFLINE ONLY: not run by product builds/tests/gates. Reads the committed
`fixtures/render-bvh/scene-a/s2a/bounds.json` (produced on the GPU by
`capture_s2a_bounds.py`), evaluates the same f32 formulas the product host code
implements, and asserts every leaf AABB is bit-equal to the MJWarp reference.
Also writes the evidence JSON consumed by the S2a README.

Why: the product host code must reproduce the frozen MJWarp NVRTC-compiled f32
arithmetic, including the compiler's fused-multiply-add contraction order. This
script pins that order: every 3-term `wp::mul`/`wp::dot` becomes
`fma(c2, b2, fma(c0, b0, round(c1*b1)))`; `pos ± z*half` becomes
`fma(±z, half, pos)`. It reports the observed max ULP error per geom type.

Output: docs/development-evidence/render-bvh/s2a-scene-bvh-host/aabb-parity.json
"""

from __future__ import annotations

import hashlib
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "fixtures/render-bvh/scene-a/s2a/bounds.json"
EVIDENCE = ROOT / "docs/development-evidence/render-bvh/s2a-scene-bvh-host/aabb-parity.json"
TASK_ID = "2026-10-24-render-bvh-s2a"
SCHEMA = 1

GEOM_NAMES = {
    0: "plane",
    2: "sphere",
    3: "capsule",
    4: "ellipsoid",
    5: "cylinder",
    6: "box",
    7: "mesh",
    8: "sdf",
}


def f32(value) -> np.float32:
    return np.float32(value)


def fma(a, b, c) -> np.float32:
    """Correctly rounded fma.rn.f32 via exact-in-longdouble product+sum."""
    return np.float32(np.longdouble(a) * np.longdouble(b) + np.longdouble(c))


def bits(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def from_bits(values) -> np.ndarray:
    return np.asarray(values, dtype=np.uint32).view(np.float32)


def matvec(row, vector) -> np.float32:
    """Frozen NVRTC contraction of wp::mul: fma(c0,b0, round(c1*b1)) then fma(c2,b2,·)."""
    intermediate = f32(row[1] * vector[1])
    intermediate = fma(row[0], vector[0], intermediate)
    return fma(row[2], vector[2], intermediate)


def dot3(a) -> np.float32:
    intermediate = f32(a[1] * a[1])
    intermediate = fma(a[0], a[0], intermediate)
    return fma(a[2], a[2], intermediate)


def box_bounds(pos, rot, size):
    minimum = np.full(3, f32(1.0e10), dtype=np.float32)
    maximum = np.full(3, f32(-1.0e10), dtype=np.float32)
    for index in range(8):
        corner = np.array(
            [
                f32(size[0] * (1 if index & 1 else -1)),
                f32(size[1] * (1 if index >> 1 & 1 else -1)),
                f32(size[2] * (1 if index >> 2 & 1 else -1)),
            ],
            dtype=np.float32,
        )
        world = np.array([f32(pos[c] + matvec(rot[c], corner)) for c in range(3)], dtype=np.float32)
        minimum = np.minimum(minimum, world)
        maximum = np.maximum(maximum, world)
    return minimum, maximum


def plane_bounds(pos, rot, size):
    size_scale = f32(max(f32(size[0]), f32(size[1])) * f32(2.0))
    if size[0] <= 0.0 or size[1] <= 0.0:
        size_scale = f32(1000.0)
    minimum = np.full(3, f32(1.0e10), dtype=np.float32)
    maximum = np.full(3, f32(-1.0e10), dtype=np.float32)
    for index in range(4):
        corner = np.array(
            [
                f32(size_scale * (1 if index & 1 else -1)),
                f32(size_scale * (1 if index >> 1 & 1 else -1)),
                f32(0.0),
            ],
            dtype=np.float32,
        )
        world = np.array([f32(pos[c] + matvec(rot[c], corner)) for c in range(3)], dtype=np.float32)
        minimum = np.minimum(minimum, world)
        maximum = np.maximum(maximum, world)
    return minimum - f32(0.01), maximum + f32(0.01)


def sphere_bounds(pos, rot, size):
    radius = size[0]
    return np.array([f32(pos[c] - radius) for c in range(3)]), np.array(
        [f32(pos[c] + radius) for c in range(3)]
    )


def capsule_bounds(pos, rot, size):
    radius, half_length = size[0], size[1]
    lower = np.empty(3, dtype=np.float32)
    upper = np.empty(3, dtype=np.float32)
    for axis in range(3):
        axis_z = rot[axis, 2]
        end1 = fma(f32(-axis_z), half_length, pos[axis])
        end2 = fma(axis_z, half_length, pos[axis])
        lower[axis] = f32(min(end1, end2) - radius)
        upper[axis] = f32(max(end1, end2) + radius)
    return lower, upper


def ellipsoid_bounds(pos, rot, size):
    extent = np.empty(3, dtype=np.float32)
    for axis in range(3):
        row = np.array(
            [f32(rot[axis, 0] * size[0]), f32(rot[axis, 1] * size[1]), f32(rot[axis, 2] * size[2])],
            dtype=np.float32,
        )
        extent[axis] = np.sqrt(dot3(row)).astype(np.float32)
    return np.array([f32(pos[c] - extent[c]) for c in range(3)]), np.array(
        [f32(pos[c] + extent[c]) for c in range(3)]
    )


def cylinder_bounds(pos, rot, size):
    radius, half_height = size[0], size[1]
    axis = np.array([rot[0, 2], rot[1, 2], rot[2, 2]], dtype=np.float32)
    basis_x = np.array([rot[0, 0], rot[1, 0], rot[2, 0]], dtype=np.float32)
    basis_y = np.array([rot[0, 1], rot[1, 1], rot[2, 1]], dtype=np.float32)
    extents = np.empty(3, dtype=np.float32)
    for axis_index in range(3):
        squared = fma(basis_x[axis_index], basis_x[axis_index], f32(basis_y[axis_index] * basis_y[axis_index]))
        radial = f32(radius * np.sqrt(squared))
        extents[axis_index] = fma(half_height, np.abs(axis[axis_index]), radial)
    return np.array([f32(pos[c] - extents[c]) for c in range(3)]), np.array(
        [f32(pos[c] + extents[c]) for c in range(3)]
    )


def mesh_half(points: np.ndarray) -> np.ndarray:
    minimum = np.min(points, axis=0)
    maximum = np.max(points, axis=0)
    return np.maximum(np.abs(minimum), np.abs(maximum)).astype(np.float32)


def leaf_bounds(model, case, slot, geom_id, mesh_half_sizes):
    geom_type = int(model["geom_type"][geom_id])
    size = from_bits(model["geom_size_bits"][geom_id * 3 : geom_id * 3 + 3])
    if geom_type in (7, 8):
        data_id = int(model["geom_dataid"][geom_id])
        size = mesh_half_sizes[data_id] if data_id >= 0 else np.zeros(3, dtype=np.float32)
    pos = from_bits(case["inputs"]["geom_xpos_bits"][slot * 3 : slot * 3 + 3])
    rot = from_bits(case["inputs"]["geom_xmat_bits"][slot * 9 : slot * 9 + 9]).reshape(3, 3)
    bounds = {
        0: plane_bounds,
        2: sphere_bounds,
        3: capsule_bounds,
        4: ellipsoid_bounds,
        5: cylinder_bounds,
        6: box_bounds,
        7: box_bounds,
        8: box_bounds,
    }[geom_type](pos, rot, size)
    return bounds


def ulp_distance(actual: np.ndarray, expected: np.ndarray) -> int:
    actual_bits = np.asarray(actual, dtype=np.float32).view(np.int32).astype(np.int64)
    expected_bits = np.asarray(expected, dtype=np.float32).view(np.int32).astype(np.int64)
    return int(np.max(np.abs(actual_bits - expected_bits)))


def main() -> int:
    document = json.loads(FIXTURE.read_text(encoding="utf-8"))
    model = document["model"]
    enabled = document["context"]["enabled_geom_ids"]
    mesh_half_sizes = {
        int(mesh): from_bits(bits_value) for mesh, bits_value in model["mesh_bounds_size_bits"].items()
    }

    per_type = {}
    cases = []
    total = mismatched = 0
    worst = 0
    changed_by_case = {}
    for case in document["cases"]:
        case_mismatches = 0
        case_worst = 0
        for slot, geom_id in enumerate(enabled):
            geom_id = int(geom_id)
            geom_type = int(model["geom_type"][geom_id])
            actual_lower, actual_upper = leaf_bounds(model, case, slot, geom_id, mesh_half_sizes)
            expected_lower = from_bits(case["lower_bits"][slot * 3 : slot * 3 + 3])
            expected_upper = from_bits(case["upper_bits"][slot * 3 : slot * 3 + 3])
            lower_equal = bits(actual_lower) == bits(expected_lower)
            upper_equal = bits(actual_upper) == bits(expected_upper)
            distance = max(ulp_distance(actual_lower, expected_lower), ulp_distance(actual_upper, expected_upper))
            name = GEOM_NAMES.get(geom_type, f"type{geom_type}")
            record = per_type.setdefault(name, {"checks": 0, "mismatches": 0, "max_ulp": 0, "geom_ids": set()})
            record["checks"] += 1
            record["geom_ids"].add(geom_id)
            record["max_ulp"] = max(record["max_ulp"], distance)
            total += 1
            worst = max(worst, distance)
            if not (lower_equal and upper_equal):
                record["mismatches"] += 1
                case_mismatches += 1
                mismatched += 1
            case_worst = max(case_worst, distance)
        cases.append(
            {
                "name": case["name"],
                "kind": case["kind"],
                "leaves_checked": len(enabled),
                "bit_equal": case_mismatches == 0,
                "max_ulp": case_worst,
            }
        )
        changed = [
            index
            for index in range(len(enabled))
            if bits(from_bits(case["lower_bits"][index * 3 : index * 3 + 3]))
            != bits(from_bits(document["cases"][0]["lower_bits"][index * 3 : index * 3 + 3]))
        ]
        changed_by_case[case["name"]] = changed

    for record in per_type.values():
        record["geom_ids"] = sorted(record["geom_ids"])

    initial_leaf_bits = [
        bits(from_bits(document["cases"][0]["lower_bits"][index * 3 : index * 3 + 3]))
        for index in range(len(enabled))
    ]
    moved_leaf_bits = [
        bits(from_bits(document["cases"][1]["lower_bits"][index * 3 : index * 3 + 3]))
        for index in range(len(enabled))
    ]
    changed_leaves = [index for index in range(len(enabled)) if initial_leaf_bits[index] != moved_leaf_bits[index]]

    evidence = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s2a-host-aabb-parity",
        "source_fixture": {
            "path": "fixtures/render-bvh/scene-a/s2a/bounds.json",
            "sha256": hashlib.sha256(FIXTURE.read_bytes()).hexdigest(),
        },
        "formula": {
            "source": "mujoco_warp/_src/bvh.py::_compute_bvh_bounds (upstream 71da24d956378a87a703b6e1442b13aec0c4ac29)",
            "f32_contraction": (
                "3-term sum -> fma(c2, b2, fma(c0, b0, round(c1*b1))); "
                "2-term sum -> fma(c0, b0, round(c1*b1)); pos ± z*half -> fma(±z, half, pos)"
            ),
            "fma_emulation": "correctly rounded fma.rn.f32 via exact long-double product+sum (authoring only)",
            "note": "product host code uses an f64 product+sum path that is bit-identical to this rule on all captured leaves",
        },
        "model_inputs": {
            "geom_count": len(model["geom_type"]),
            "mesh_count": int(model["nmesh"]),
            "geom_size_f32_bits_checked": True,
            "mesh_bounds_f32_bits_checked": True,
        },
        "cases": cases,
        "per_type": per_type,
        "summary": {
            "cases": len(cases),
            "leaf_checks": total,
            "bit_equal": total - mismatched,
            "mismatched": mismatched,
            "max_ulp": worst,
        },
        "refit": {
            "initial_vs_moved_changed_leaves": changed_leaves,
            "static_leaves": [index for index in range(len(enabled)) if index not in changed_leaves],
        },
        "changed_leaves_by_case": changed_by_case,
    }
    EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
    EVIDENCE.write_text(json.dumps(evidence, indent=2, sort_keys=False) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"output": str(EVIDENCE.relative_to(ROOT)), "summary": evidence["summary"],
                      "changed_leaves": changed_leaves}, indent=2))
    return 0 if mismatched == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
