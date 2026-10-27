"""Authoring-only MJB generator for the S3 product-collision end-to-end target.

Compiles the three frozen mesh fixtures (`mesh-box`, `mesh-cylinder`, `mesh-mesh`)
with the pinned MuJoCo 3.12.0 Python bindings, saves the `.mjb` files used by the S1
native probe at test time, and cross-checks the resulting 32 collision-input arrays
byte-for-byte against the frozen M0b/M3 case bins (authoring-only 3.15.0 capture).

The product test only reads the emitted `.mjb` files plus the frozen reference JSON;
it never runs Python (`AGENTS.md`: product tests must not start Python).

Pinned interpreter:
    target/render-bvh-precheck/venv312/Scripts/python.exe   (mujoco 3.12.0)

Usage (from the repository root):
    target/render-bvh-precheck/venv312/Scripts/python.exe \
        docs/development-evidence/product-collision-inputs-wiring/scripts/generate_collision_inputs_mjb.py \
        > docs/development-evidence/product-collision-inputs-wiring/logs/s3-mjb-generation.log

Exit code 0 = PASS (all three MJBs written, 96/96 arrays byte-identical to the bins).
"""

import hashlib
import json
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
OUT_DIR = EVIDENCE / "fixtures/mjb-312"
PROVENANCE = OUT_DIR / "provenance.json"

PINNED_MUJOCO = "3.12.0"
M1_REFERENCE = ROOT / "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json"
M3_REFERENCE = ROOT / (
    "docs/development-evidence/mesh-multicontact-m3/"
    "upstream-mesh-multicontact-mesh-mesh-gpu.json"
)

# 32 fields: (id, name, dtype of the host vector, width, mjModel attribute name or None
# for the two mjData fields). Mirrors include/mjwarp_collision_inputs_probe.h and
# src/model/collision_inputs.rs.
FIELDS = [
    (0, "geom_xpos", "f32", 3, None),
    (1, "geom_xmat", "f32", 9, None),
    (2, "geom_size", "f32", 3, "geom_size"),
    (3, "geom_margin", "f32", 1, "geom_margin"),
    (4, "geom_gap", "f32", 1, "geom_gap"),
    (5, "geom_rbound", "f32", 1, "geom_rbound"),
    (6, "geom_aabb", "f32", 6, "geom_aabb"),
    (7, "geom_type", "i32", 1, "geom_type"),
    (8, "geom_dataid", "i32", 1, "geom_dataid"),
    (9, "mesh_vertadr", "i32", 1, "mesh_vertadr"),
    (10, "mesh_vertnum", "i32", 1, "mesh_vertnum"),
    (11, "mesh_graphadr", "i32", 1, "mesh_graphadr"),
    (12, "mesh_octadr", "i32", 1, "mesh_octadr"),
    (13, "mesh_vert", "f32", 3, "mesh_vert"),
    (14, "mesh_graph", "i32", 1, "mesh_graph"),
    (15, "mesh_polynum", "i32", 1, "mesh_polynum"),
    (16, "mesh_polyadr", "i32", 1, "mesh_polyadr"),
    (17, "mesh_polynormal", "f32", 3, "mesh_polynormal"),
    (18, "mesh_polyvertadr", "i32", 1, "mesh_polyvertadr"),
    (19, "mesh_polyvertnum", "i32", 1, "mesh_polyvertnum"),
    (20, "mesh_polyvert", "i32", 1, "mesh_polyvert"),
    (21, "mesh_polymapadr", "i32", 1, "mesh_polymapadr"),
    (22, "mesh_polymapnum", "i32", 1, "mesh_polymapnum"),
    (23, "mesh_polymap", "i32", 1, "mesh_polymap"),
    (24, "hfield_size", "f32", 4, "hfield_size"),
    (25, "hfield_nrow", "i32", 1, "hfield_nrow"),
    (26, "hfield_ncol", "i32", 1, "hfield_ncol"),
    (27, "hfield_adr", "i32", 1, "hfield_adr"),
    (28, "hfield_data", "f32", 1, "hfield_data"),
    (29, "oct_aabb", "f32", 6, "oct_aabb"),
    (30, "oct_child", "i32", 8, "oct_child"),
    (31, "oct_coeff", "f32", 8, "oct_coeff"),
]

# Frozen case list: fixture XML + qpos source + case bins to cross-check against.
CASES = [
    {
        "name": "mesh-box",
        "xml": "fixtures/g10-p5-mesh-multicontact/mesh-box.xml",
        "reference": M1_REFERENCE,
        "bins": "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box",
        "geom_order": [1, 0],
    },
    {
        "name": "mesh-cylinder",
        "xml": "fixtures/g10-p5-mesh-multicontact/mesh-cylinder.xml",
        "reference": M1_REFERENCE,
        "bins": "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder",
        "geom_order": [1, 0],
    },
    {
        "name": "mesh-mesh",
        "xml": "fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml",
        "reference": M3_REFERENCE,
        "bins": "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh",
        "geom_order": [0, 1],
    },
]


def sha256_bytes(blob):
    return hashlib.sha256(blob).hexdigest()


def sha256_file(path):
    return sha256_bytes(path.read_bytes())


def host_bytes(values, dtype):
    if dtype == "f32":
        return np.asarray(values, dtype=np.float32).tobytes()
    return np.asarray(values, dtype=np.int32).tobytes()


def field_vectors(model, data):
    """Return {field name: (dtype, host byte blob)} for the 32 S1 fields."""
    vectors = {}
    for field_id, name, dtype, _width, attribute in FIELDS:
        if field_id == 0:
            values = np.asarray(data.geom_xpos, dtype=np.float64).ravel()
        elif field_id == 1:
            values = np.asarray(data.geom_xmat, dtype=np.float64).ravel()
        else:
            values = np.asarray(getattr(model, attribute)).ravel()
        vectors[name] = (dtype, host_bytes(values, dtype))
    return vectors


def first_difference(produced, expected, dtype):
    width = np.uint32 if dtype == "f32" else np.int32
    got = np.frombuffer(produced, dtype=width)
    want = np.frombuffer(expected, dtype=width)
    for index, (left, right) in enumerate(zip(got, want)):
        if left != right:
            return {"index": int(index), "mjb": int(left), "bin": int(right)}
    return {"index": None, "mjb": None, "bin": None, "length": [int(got.size), int(want.size)]}


def main():
    if mujoco.__version__ != PINNED_MUJOCO:
        print(f"FAIL: pinned mujoco {PINNED_MUJOCO} required, got {mujoco.__version__}")
        return 1
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    report = {
        "schema": 1,
        "kind": "s3-product-collision-mjb-fixtures",
        "authoring_only": True,
        "mujoco": mujoco.__version__,
        "numpy": np.__version__,
        "python": sys.version.split()[0],
        "purpose": "MJB files for the S3 end-to-end target (real model -> S1 snapshot -> S2 adapter -> device)",
        "cases": {},
        "failures": [],
    }

    for case in CASES:
        xml_path = ROOT / case["xml"]
        model = mujoco.MjModel.from_xml_path(str(xml_path))
        mjb_path = OUT_DIR / f"{case['name']}-312.mjb"
        mujoco.mj_saveModel(model, str(mjb_path))
        mjb_bytes = mjb_path.read_bytes()

        reference = json.loads(Path(case["reference"]).read_text(encoding="utf-8"))
        case_reference = reference["cases"][case["name"]]
        assert case_reference["xml"] == case["xml"], f"{case['name']}: xml path mismatch"
        assert case_reference["xml_sha256"] == sha256_file(xml_path), f"{case['name']}: xml sha mismatch"
        qpos = [float(value) for value in case_reference["qpos"]]

        data = mujoco.MjData(model)
        data.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mujoco.mj_kinematics(model, data)
        produced = field_vectors(model, data)

        # Reload the written MJB and re-dump: the file must round-trip the same arrays.
        reloaded = mujoco.MjModel.from_binary_path(str(mjb_path))
        reloaded_data = mujoco.MjData(reloaded)
        reloaded_data.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mujoco.mj_kinematics(reloaded, reloaded_data)
        round_trip = field_vectors(reloaded, reloaded_data)

        bins_dir = ROOT / case["bins"]
        arrays = {}
        case_failures = []
        for _field_id, name, dtype, _width, _attribute in FIELDS:
            blob = produced[name][1]
            bin_path = bins_dir / f"{name}.bin"
            expected = bin_path.read_bytes()
            entry = {
                "dtype": dtype,
                "count": int(len(blob) // (4)),
                "sha256": sha256_bytes(blob),
                "bin_sha256": sha256_bytes(expected),
                "bin_match": blob == expected,
                "round_trip_match": blob == round_trip[name][1],
            }
            if not entry["bin_match"]:
                entry["first_difference"] = first_difference(blob, expected, dtype)
                case_failures.append(name)
            if not entry["round_trip_match"]:
                case_failures.append(f"{name}:round_trip")
            arrays[name] = entry

        report["cases"][case["name"]] = {
            "xml": case["xml"],
            "xml_sha256": sha256_file(xml_path),
            "xml_bytes": xml_path.stat().st_size,
            "mjb": f"fixtures/mjb-312/{case['name']}-312.mjb",
            "mjb_sha256": sha256_bytes(mjb_bytes),
            "mjb_bytes": len(mjb_bytes),
            "qpos": qpos,
            "qpos_bits": [int(np.float32(value).view(np.uint32)) for value in qpos],
            "nq": int(model.nq),
            "ngeom": int(model.ngeom),
            "reference": str(Path(case["reference"]).relative_to(ROOT).as_posix()),
            "reference_sha256": sha256_file(Path(case["reference"])),
            "bins": case["bins"],
            "geom_order": case["geom_order"],
            "arrays": arrays,
            "field_mismatches": case_failures,
        }
        print(
            f"case={case['name']} mjb_bytes={len(mjb_bytes)} mjb_sha256={sha256_bytes(mjb_bytes)} "
            f"arrays={len(arrays)} field_mismatches={case_failures}"
        )
        report["failures"].extend(f"{case['name']}:{name}" for name in case_failures)

    report["result"] = "PASS" if not report["failures"] else "FAIL"
    report["array_checks"] = sum(len(case["arrays"]) for case in report["cases"].values())
    report["array_mismatches"] = len(report["failures"])
    PROVENANCE.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(
        f"array_checks={report['array_checks']} array_mismatches={report['array_mismatches']} "
        f"RESULT={report['result']} provenance={PROVENANCE.relative_to(ROOT)}"
    )
    return 0 if report["result"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
