"""G10@P5 M0b: mesh multicontact case production (32 input arrays, authoring only, CPU).

Produces the 32 host input arrays for the M1 mesh-multicontact fixtures
(`fixtures/g10-p5-mesh-multicontact/`, the same fixtures used for the M1 GPU capture) as
little-endian `.bin` files plus `meta.txt`, into a **new** directory
`docs/development-evidence/mesh-multicontact-m0b/cases/<case>/`.

Same-source requirements (Chief hard requirement 1):
  - the model is loaded from the exact fixture XML used by M1 (bitwise `xml_sha256`);
  - qpos is the deterministic `capture` keyframe, compared bitwise to the M1 reference
    `qpos_bits`; CPU kinematics is also compared bitwise to the M1 reference
    `geom_xpos`/`geom_xmat` (identity poses, so this is expected to be exact).

CPU self-check gate (Chief hard requirement 2), mirroring the corrected M0a
`validate_mesh_poly` rules (nmesh / nmeshpoly / nmeshvert):
  - all 9 polygon arrays present; lengths / dims follow pinned
    `mujoco_warp/_src/types.py` (rev 71da24d9) L1329-1337 / L1864-1872;
  - `sum(polynum) == nmeshpoly`; per-mesh, per-polygon and per-vertex ranges in bounds;
  - strong consistency `sum(polyvertnum) == len(mesh_polyvert)` and
    `sum(polymapnum) == len(mesh_polymap)`.

No CUDA context is created (`wp.set_device("cpu")`); this script never touches product
code and product tests never run Python. Usage (pinned venv):
    python -B prepare_mesh_cases.py
"""
from __future__ import annotations

import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
M1_REF = ROOT / "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json"
OUT = ROOT / "docs/development-evidence/mesh-multicontact-m0b/cases"
MANIFEST = ROOT / "docs/development-evidence/mesh-multicontact-m0b/mesh-cases-manifest.json"
CASES = ("mesh-box", "mesh-cylinder")

GEOM_BOX = 6
POLY_ARRAYS = (
    "mesh_polynum",
    "mesh_polyadr",
    "mesh_polynormal",
    "mesh_polyvertadr",
    "mesh_polyvertnum",
    "mesh_polyvert",
    "mesh_polymapadr",
    "mesh_polymapnum",
    "mesh_polymap",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values) -> list[int]:
    return [int(v) for v in np.asarray(values, dtype=np.float32).reshape(-1).view(np.uint32)]


def squeeze(array):
    array = np.asarray(array)
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def as_f32(array) -> np.ndarray:
    return np.ascontiguousarray(squeeze(np.asarray(array)), dtype=np.float32)


def as_i32(array) -> np.ndarray:
    return np.ascontiguousarray(squeeze(np.asarray(array)), dtype=np.int32)


def write_bin(directory: Path, name: str, values: np.ndarray, dtype) -> dict:
    values = np.ascontiguousarray(values.reshape(-1), dtype=dtype)
    path = directory / f"{name}.bin"
    path.write_bytes(values.tobytes())
    return {
        "file": path.name,
        "dtype": "f32" if dtype == np.float32 else "i32",
        "count": int(values.size),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def mesh_poly_selfcheck(arrays: dict[str, np.ndarray]) -> tuple[list[dict], list[str]]:
    """Mirror of the corrected M0a `validate_mesh_poly` rules (same order/messages)."""
    rules: list[dict] = []
    failures: list[str] = []

    def check(rule_id: str, rule: str, condition: bool, detail: str = "") -> None:
        rules.append({"id": rule_id, "rule": rule, "result": "PASS" if condition else "FAIL", "detail": detail})
        if not condition:
            failures.append(f"{rule_id}: {rule} ({detail})")

    polynum = arrays["mesh_polynum"]
    polyadr = arrays["mesh_polyadr"]
    polynormal = arrays["mesh_polynormal"]
    polyvertadr = arrays["mesh_polyvertadr"]
    polyvertnum = arrays["mesh_polyvertnum"]
    polyvert = arrays["mesh_polyvert"]
    polymapadr = arrays["mesh_polymapadr"]
    polymapnum = arrays["mesh_polymapnum"]
    polymap = arrays["mesh_polymap"]

    check("1", "9 polygon arrays present", all(a.size > 0 for a in (polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap)))
    nmesh = int(polynum.size)
    check("2", "polynum.len == polyadr.len (per mesh)", int(polyadr.size) == nmesh, f"nmesh={nmesh}")
    check("3", "polynormal.len % 3 == 0", int(polynormal.size) % 3 == 0)
    nmeshpoly = int(polynormal.size) // 3
    check(
        "4",
        "polyvertadr/polyvertnum.len == nmeshpoly (per polygon)",
        int(polyvertadr.size) == nmeshpoly and int(polyvertnum.size) == nmeshpoly,
        f"nmeshpoly={nmeshpoly}",
    )
    check(
        "5",
        "polymapadr.len == polymapnum.len (per vertex)",
        int(polymapadr.size) == int(polymapnum.size),
        f"nmeshvert={polymapadr.size}",
    )
    nmeshvert = int(polymapadr.size)

    total_polygons = 0
    ranges_ok = True
    negative = False
    for index in range(nmesh):
        adr, num = int(polyadr[index]), int(polynum[index])
        if adr < 0 or num < 0:
            negative = True
        elif adr + num > nmeshpoly:
            ranges_ok = False
        total_polygons += max(num, 0)
    check("6", "per mesh: polyadr+polynum <= nmeshpoly", ranges_ok)
    check("7", "sum(polynum) == nmeshpoly", total_polygons == nmeshpoly, f"{total_polygons} vs {nmeshpoly}")

    total_vertices = 0
    vert_ranges_ok = True
    for index in range(nmeshpoly):
        adr, num = int(polyvertadr[index]), int(polyvertnum[index])
        if adr < 0 or num < 0:
            negative = True
        elif adr + num > int(polyvert.size):
            vert_ranges_ok = False
        total_vertices += max(num, 0)
    check("8", "per polygon: polyvertadr+polyvertnum <= len(mesh_polyvert)", vert_ranges_ok)
    check("9", "sum(polyvertnum) == len(mesh_polyvert)", total_vertices == int(polyvert.size), f"{total_vertices} vs {polyvert.size}")

    total_maps = 0
    map_ranges_ok = True
    for index in range(nmeshvert):
        adr, num = int(polymapadr[index]), int(polymapnum[index])
        if adr < 0 or num < 0:
            negative = True
        elif adr + num > int(polymap.size):
            map_ranges_ok = False
        total_maps += max(num, 0)
    check("10", "per vertex: polymapadr+polymapnum <= len(mesh_polymap)", map_ranges_ok)
    check("11", "sum(polymapnum) == len(mesh_polymap)", total_maps == int(polymap.size), f"{total_maps} vs {polymap.size}")
    check("12", "all adr/num non-negative", not negative)
    return rules, failures


def prepare_case(name: str, reference_sha: str) -> dict:
    xml = FIXTURES / f"{name}.xml"
    reference = json.loads(M1_REF.read_text(encoding="utf-8"))["cases"][name]
    host = mujoco.MjModel.from_xml_path(str(xml))
    key_id = host.key("capture").id
    qpos_flat = np.asarray(host.key_qpos[key_id], dtype=np.float32)
    nworld = 1
    ngeom = int(host.ngeom)

    cross: dict[str, object] = {}
    cross["xml_sha256"] = {"actual": sha256(xml), "reference": reference["xml_sha256"]}
    cross["xml_sha256"]["match"] = cross["xml_sha256"]["actual"] == cross["xml_sha256"]["reference"]
    qpos_bits = bits32(qpos_flat)
    cross["qpos_bits"] = {"actual": qpos_bits, "reference": reference["qpos_bits"], "match": qpos_bits == reference["qpos_bits"]}

    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=nworld, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos_flat.reshape(nworld, host.nq))
    data.qvel.assign(np.zeros((nworld, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    cross["collision_count"] = {"actual": count, "reference": int(reference["output"]["count"]), "match": count == int(reference["output"]["count"])}
    geom_xpos = np.asarray(data.geom_xpos.numpy(), dtype=np.float32)
    geom_xmat = np.asarray(data.geom_xmat.numpy(), dtype=np.float32)
    ref_xpos = np.asarray(reference["output"]["geom_xpos"], dtype=np.float32)
    ref_xmat = np.asarray(reference["output"]["geom_xmat"], dtype=np.float32)
    cross["geom_xpos_bits"] = {"match": bits32(geom_xpos) == bits32(ref_xpos)}
    cross["geom_xmat_bits"] = {"match": bits32(geom_xmat) == bits32(ref_xmat)}

    arrays: dict[str, tuple[np.ndarray, object]] = {}
    arrays["geom_xpos"] = (as_f32(geom_xpos), np.float32)
    arrays["geom_xmat"] = (as_f32(geom_xmat), np.float32)
    arrays["geom_size"] = (as_f32(model.geom_size.numpy()), np.float32)
    arrays["geom_margin"] = (as_f32(model.geom_margin.numpy()), np.float32)
    arrays["geom_gap"] = (as_f32(model.geom_gap.numpy()), np.float32)
    arrays["geom_rbound"] = (as_f32(model.geom_rbound.numpy()), np.float32)
    arrays["geom_aabb"] = (as_f32(model.geom_aabb.numpy()), np.float32)
    arrays["geom_type"] = (as_i32(model.geom_type.numpy()), np.int32)
    arrays["geom_dataid"] = (as_i32(model.geom_dataid.numpy()), np.int32)
    arrays["mesh_vertadr"] = (as_i32(model.mesh_vertadr.numpy()), np.int32)
    arrays["mesh_vertnum"] = (as_i32(model.mesh_vertnum.numpy()), np.int32)
    arrays["mesh_graphadr"] = (as_i32(model.mesh_graphadr.numpy()), np.int32)
    arrays["mesh_octadr"] = (as_i32(model.mesh_octadr.numpy()), np.int32)
    arrays["mesh_vert"] = (as_f32(model.mesh_vert.numpy()), np.float32)
    arrays["mesh_graph"] = (as_i32(model.mesh_graph.numpy()), np.int32)
    arrays["hfield_size"] = (as_f32(model.hfield_size.numpy()), np.float32)
    arrays["hfield_nrow"] = (as_i32(model.hfield_nrow.numpy()), np.int32)
    arrays["hfield_ncol"] = (as_i32(model.hfield_ncol.numpy()), np.int32)
    arrays["hfield_adr"] = (as_i32(model.hfield_adr.numpy()), np.int32)
    arrays["hfield_data"] = (as_f32(model.hfield_data.numpy()), np.float32)
    arrays["oct_aabb"] = (as_f32(model.oct_aabb.numpy()), np.float32)
    arrays["oct_child"] = (as_i32(model.oct_child.numpy()), np.int32)
    arrays["oct_coeff"] = (as_f32(model.oct_coeff.numpy()), np.float32)
    arrays["mesh_polynum"] = (as_i32(model.mesh_polynum.numpy()), np.int32)
    arrays["mesh_polyadr"] = (as_i32(model.mesh_polyadr.numpy()), np.int32)
    arrays["mesh_polynormal"] = (as_f32(model.mesh_polynormal.numpy()), np.float32)
    arrays["mesh_polyvertadr"] = (as_i32(model.mesh_polyvertadr.numpy()), np.int32)
    arrays["mesh_polyvertnum"] = (as_i32(model.mesh_polyvertnum.numpy()), np.int32)
    arrays["mesh_polyvert"] = (as_i32(model.mesh_polyvert.numpy()), np.int32)
    arrays["mesh_polymapadr"] = (as_i32(model.mesh_polymapadr.numpy()), np.int32)
    arrays["mesh_polymapnum"] = (as_i32(model.mesh_polymapnum.numpy()), np.int32)
    arrays["mesh_polymap"] = (as_i32(model.mesh_polymap.numpy()), np.int32)

    poly_values = {key: arrays[key][0] for key in POLY_ARRAYS}
    poly_rules, poly_failures = mesh_poly_selfcheck(poly_values)

    # meta.txt: scalars needed by CollisionGpuInputs.meta (same keys as the G5 prepare script).
    # Contact parameter rows are identical for every row of a pair; take the M1 reference row.
    fields = reference["output"]["fields"]
    row = {key: (values[0] if values else []) for key, values in fields.items()}
    g1, g2 = int(row["geom"][0]), int(row["geom"][1])
    geom_type = as_i32(model.geom_type.numpy())
    geom_gap_values = as_f32(model.geom_gap.numpy())
    geom_condim = as_i32(model.geom_condim.numpy())
    nmaxcondim = max(int(v) for v in geom_condim)
    efc_width = max(1, 2 * (nmaxcondim - 1))
    type1, type2 = int(geom_type[g1]), int(geom_type[g2])
    meta_lines = [
        ("case", name),
        ("nworld", nworld),
        ("ngeom", ngeom),
        ("naconmax", 4096 * nworld),
        ("naccdmax", 4096 * nworld),
        ("efc_width", efc_width),
        ("g1", g1),
        ("g2", g2),
        ("type1", type1),
        ("type2", type2),
        ("tolerance", float(host.opt.ccd_tolerance)),
        ("gjk_iterations", int(host.opt.ccd_iterations)),
        ("epa_iterations", 16 if (type1 == GEOM_BOX and type2 == GEOM_BOX) else int(host.opt.ccd_iterations)),
        ("margin", float(row["includemargin"][0])),
        ("gap", float(geom_gap_values[g1] + geom_gap_values[g2])),
        ("condim", int(row["dim"][0])),
        ("friction", [float(v) for v in row["friction"]]),
        ("solref", [float(v) for v in row["solref"]]),
        ("solreffriction", [float(v) for v in row["solreffriction"]]),
        ("solimp", [float(v) for v in row["solimp"]]),
        ("adhesion", float(row["adhesion"][0])),
        ("pairid0", -1),
        ("pairid1", -1),
        ("sdf_initpoints", int(host.opt.sdf_initpoints)),
        ("sdf_iterations", int(host.opt.sdf_iterations)),
        ("hfield_id", -1),
        ("oct_root", -1),
        ("noct", 0),
    ]
    meta_text = "\n".join(
        f"{key} " + " ".join(repr(value) for value in values) if isinstance(values, list) else f"{key} {values}"
        for key, values in meta_lines
    ) + "\n"

    directory = OUT / name
    directory.mkdir(parents=True, exist_ok=True)
    written = {}
    for key in sorted(arrays):
        written[key] = write_bin(directory, key, arrays[key][0], arrays[key][1])
    (directory / "meta.txt").write_text(meta_text, encoding="utf-8")

    return {
        "xml": xml.relative_to(ROOT).as_posix(),
        "xml_sha256": sha256(xml),
        "qpos": [float(v) for v in qpos_flat],
        "qpos_bits": qpos_bits,
        "nworld": nworld,
        "ngeom": ngeom,
        "meta": "meta.txt",
        "arrays": written,
        "cross_check": cross,
        "cpu_selfcheck": {"rules": poly_rules, "failures": poly_failures, "result": "PASS" if not poly_failures else "FAIL"},
    }, poly_failures


def main() -> int:
    wp.set_device("cpu")
    print("device:", wp.get_device())
    reference_sha = sha256(M1_REF)
    report = {
        "schema": 1,
        "task_id": "2026-10-26-mesh-multicontact-m0b-case-production",
        "kind": "m0b-mesh-multicontact-case-bins",
        "purpose": "32-array host inputs for the M1 mesh-multicontact fixtures (mesh-box / mesh-cylinder); same pinned model + same fixtures as the M1 GPU capture",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "arrays_order": [
            "geom_xpos", "geom_xmat", "geom_size", "geom_margin", "geom_gap", "geom_rbound",
            "geom_aabb", "geom_type", "geom_dataid", "mesh_vertadr", "mesh_vertnum",
            "mesh_graphadr", "mesh_octadr", "mesh_vert", "mesh_graph", "mesh_polynum",
            "mesh_polyadr", "mesh_polynormal", "mesh_polyvertadr", "mesh_polyvertnum",
            "mesh_polyvert", "mesh_polymapadr", "mesh_polymapnum", "mesh_polymap",
            "hfield_size", "hfield_nrow", "hfield_ncol", "hfield_adr", "hfield_data",
            "oct_aabb", "oct_child", "oct_coeff",
        ],
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
                "numpy": np.__version__,
            },
            "device": "cpu (no CUDA context)",
            "authoring_script": "docs/development-evidence/mesh-multicontact-m0b/scripts/prepare_mesh_cases.py",
            "authoring_script_sha256": sha256(Path(__file__)),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
            "m1_reference": "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json",
            "m1_reference_sha256": reference_sha,
        },
        "cases": {},
        "failures": [],
    }
    for name in CASES:
        record, failures = prepare_case(name, reference_sha)
        report["cases"][name] = record
        report["failures"].extend(f"{name}: {failure}" for failure in failures)
        mismatched = [key for key, value in record["cross_check"].items() if isinstance(value, dict) and "match" in value and not value["match"]]
        if mismatched:
            report["failures"].extend(f"{name}: cross_check mismatch {mismatched}")
        print(
            f"{name}: arrays={len(record['arrays'])} selfcheck={record['cpu_selfcheck']['result']} "
            f"xml={record['cross_check']['xml_sha256']['match']} qpos_bits={record['cross_check']['qpos_bits']['match']} "
            f"geom_xpos={record['cross_check']['geom_xpos_bits']['match']} geom_xmat={record['cross_check']['geom_xmat_bits']['match']} "
            f"count={record['cross_check']['collision_count']['actual']}/{record['cross_check']['collision_count']['reference']}"
        )
        for entry in record["cpu_selfcheck"]["rules"]:
            print(f"  selfcheck[{entry['id']}] {entry['result']}: {entry['rule']} {entry['detail']}")
    report["result"] = "PASS" if not report["failures"] else "FAIL"
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("RESULT:", report["result"], "failures:", report["failures"])
    print("wrote", MANIFEST, "and", OUT)
    return 0 if not report["failures"] else 1


if __name__ == "__main__":
    sys.exit(main())
