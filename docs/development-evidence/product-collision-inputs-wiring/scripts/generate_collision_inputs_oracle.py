"""Authoring-only oracle generator for the S1 collision-inputs snapshot test.

Reads three frozen 3.12.0 MJB fixtures with the pinned MuJoCo 3.12.0 Python
bindings, emits every collision-input field (IDs 2..31), the mjData
geom_xpos/geom_xmat f32 bit patterns after mj_kinematics for fixed qpos cases,
and the model counts into tests/support/collision_inputs_oracle.json.

The product test only reads this JSON plus the MJB bytes; it never runs Python.

Usage (pinned venv):
    target/render-bvh-precheck/venv312/Scripts/python.exe \
        docs/development-evidence/product-collision-inputs-wiring/scripts/generate_collision_inputs_oracle.py
"""

import hashlib
import json
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / "tests/support/collision_inputs_oracle.json"

MODELS = [
    "fixtures/g10-p5/mjb-312/hfield-box.mjb",
    "fixtures/render-bvh/scene-a/scene-312.mjb",
    "fixtures/g10-p5/mjb-312/sdf-cylinder.mjb",
]

# (field id, name, kind, unit name, width, attribute) - mirrors
# include/mjwarp_collision_inputs_probe.h and src/model/collision_inputs.rs.
KIND_INDEX, KIND_NUM, KIND_FLOAT = 1, 2, 4
FIELDS = [
    (0, "geom_xpos", KIND_FLOAT, "ngeom", 3, None),
    (1, "geom_xmat", KIND_FLOAT, "ngeom", 9, None),
    (2, "geom_size", KIND_NUM, "ngeom", 3, "geom_size"),
    (3, "geom_margin", KIND_NUM, "ngeom", 1, "geom_margin"),
    (4, "geom_gap", KIND_NUM, "ngeom", 1, "geom_gap"),
    (5, "geom_rbound", KIND_NUM, "ngeom", 1, "geom_rbound"),
    (6, "geom_aabb", KIND_NUM, "ngeom", 6, "geom_aabb"),
    (7, "geom_type", KIND_INDEX, "ngeom", 1, "geom_type"),
    (8, "geom_dataid", KIND_INDEX, "ngeom", 1, "geom_dataid"),
    (9, "mesh_vertadr", KIND_INDEX, "nmesh", 1, "mesh_vertadr"),
    (10, "mesh_vertnum", KIND_INDEX, "nmesh", 1, "mesh_vertnum"),
    (11, "mesh_graphadr", KIND_INDEX, "nmesh", 1, "mesh_graphadr"),
    (12, "mesh_octadr", KIND_INDEX, "nmesh", 1, "mesh_octadr"),
    (13, "mesh_vert", KIND_FLOAT, "nmeshvert", 3, "mesh_vert"),
    (14, "mesh_graph", KIND_INDEX, "nmeshgraph", 1, "mesh_graph"),
    (15, "mesh_polynum", KIND_INDEX, "nmesh", 1, "mesh_polynum"),
    (16, "mesh_polyadr", KIND_INDEX, "nmesh", 1, "mesh_polyadr"),
    (17, "mesh_polynormal", KIND_NUM, "nmeshpoly", 3, "mesh_polynormal"),
    (18, "mesh_polyvertadr", KIND_INDEX, "nmeshpoly", 1, "mesh_polyvertadr"),
    (19, "mesh_polyvertnum", KIND_INDEX, "nmeshpoly", 1, "mesh_polyvertnum"),
    (20, "mesh_polyvert", KIND_INDEX, "nmeshpolyvert", 1, "mesh_polyvert"),
    (21, "mesh_polymapadr", KIND_INDEX, "nmeshvert", 1, "mesh_polymapadr"),
    (22, "mesh_polymapnum", KIND_INDEX, "nmeshvert", 1, "mesh_polymapnum"),
    (23, "mesh_polymap", KIND_INDEX, "nmeshpolymap", 1, "mesh_polymap"),
    (24, "hfield_size", KIND_NUM, "nhfield", 4, "hfield_size"),
    (25, "hfield_nrow", KIND_INDEX, "nhfield", 1, "hfield_nrow"),
    (26, "hfield_ncol", KIND_INDEX, "nhfield", 1, "hfield_ncol"),
    (27, "hfield_adr", KIND_INDEX, "nhfield", 1, "hfield_adr"),
    (28, "hfield_data", KIND_FLOAT, "nhfielddata", 1, "hfield_data"),
    (29, "oct_aabb", KIND_NUM, "noct", 6, "oct_aabb"),
    (30, "oct_child", KIND_INDEX, "noct", 8, "oct_child"),
    (31, "oct_coeff", KIND_NUM, "noct", 8, "oct_coeff"),
]

COUNTS = ["nq", "ngeom", "nmesh", "nmeshvert", "nmeshgraph", "nmeshpoly",
          "nmeshpolyvert", "nmeshpolymap", "nhfield", "nhfielddata", "noct"]


def bits32(values):
    return np.asarray(values, dtype=np.float32).view(np.uint32).tolist()


def bits64(values):
    """Exact f64 payload as u64: serde_json keeps u64 exact without float_roundtrip."""
    return np.asarray(values, dtype=np.float64).view(np.uint64).tolist()


def main():
    if mujoco.__version__ != "3.12.0":
        raise SystemExit(f"pinned mujoco 3.12.0 required, got {mujoco.__version__}")
    print(f"mujoco={mujoco.__version__} python={sys.version.split()[0]} numpy={np.__version__}")
    models = []
    for rel in MODELS:
        path = ROOT / rel
        raw = path.read_bytes()
        model = mujoco.MjModel.from_binary_path(str(path))
        data = mujoco.MjData(model)
        counts = {name: int(getattr(model, name)) for name in COUNTS}
        # Deterministic qpos cases: the model default plus one small per-entry offset.
        qpos0 = np.array(model.qpos0, dtype=np.float64)
        qpos1 = qpos0 + np.array([0.01 * ((index % 3) - 1) for index in range(model.nq)])
        cases = []
        for qpos in (qpos0, qpos1):
            data.qpos[:] = qpos
            mujoco.mj_kinematics(model, data)
            cases.append({
                "qpos_bits": bits64(qpos),
                "geom_xpos_bits": bits32(np.asarray(data.geom_xpos).reshape(-1)),
                "geom_xmat_bits": bits32(np.asarray(data.geom_xmat).reshape(-1)),
            })
        fields = {}
        oct_counts_only = counts["noct"] > 0
        for field_id, name, kind, unit, width, attribute in FIELDS:
            if field_id < 2:
                continue
            expected = counts[unit] * width
            if oct_counts_only and field_id >= 29:
                fields[str(field_id)] = {
                    "kind": kind,
                    "count": expected,
                    "volume_oracle": True,
                }
                continue
            values = np.asarray(getattr(model, attribute)).reshape(-1)
            if values.size != expected:
                raise SystemExit(f"{rel} {name}: size {values.size} != expected {expected}")
            if kind == KIND_INDEX:
                entry = {"kind": kind, "count": expected, "i32": values.astype(np.int32).tolist()}
            elif kind == KIND_FLOAT:
                entry = {"kind": kind, "count": expected,
                         "f32_bits": np.asarray(values, dtype=np.float32).view(np.uint32).tolist()}
            else:
                entry = {"kind": kind, "count": expected, "f64_bits": bits64(values),
                         "f32_bits": bits32(values)}
            fields[str(field_id)] = entry
        models.append({
            "name": path.stem,
            "mjb": rel,
            "mjb_bytes": len(raw),
            "mjb_sha256": hashlib.sha256(raw).hexdigest(),
            "counts": counts,
            "qpos_cases": cases,
            "fields": fields,
        })
        print(f"model {path.stem}: counts={counts} qpos_cases={len(cases)} fields={len(fields)}")
    payload = {
        "schema": 1,
        "kind": "s1-collision-inputs-oracle",
        "provenance": {
            "generated_by": "docs/development-evidence/product-collision-inputs-wiring/scripts/generate_collision_inputs_oracle.py",
            "mujoco_python": mujoco.__version__,
            "numpy": np.__version__,
            "python": sys.version.split()[0],
            "device": "cpu",
        },
        "models": models,
    }
    text = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    OUT.write_bytes(text.encode("utf-8"))
    print(f"wrote {OUT.relative_to(ROOT)} bytes={len(text.encode('utf-8'))} sha256={hashlib.sha256(text.encode('utf-8')).hexdigest()}")


if __name__ == "__main__":
    main()
