"""QA independent dump: real mjModel mesh polygon arrays for the M1 cube fixture.

Reads fixtures/g10-p5-mesh-multicontact/mesh-box.xml with the pinned mujoco 3.15
(in the pinned venv) and dumps the 9 mesh polygon arrays exactly as stored in
mjModel, so the independent review can compare them against
`validate_tests::CubeMeshPoly` and against the pinned `types.py` dims.

Pure CPU, no warp/mujoco_warp needed: mjModel arrays are the host-side truth for
the sidecar (the sidecar uploads these arrays verbatim).
"""
import hashlib
import json
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
OUT = ROOT / "target/qa-m0afix/cube-poly-model.json"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def f32_bits(values):
    a = np.asarray(values, dtype=np.float32).reshape(-1)
    return [int(v) for v in a.view(np.uint32)]


def main():
    report = {
        "kind": "qa-m0afix-mesh-poly-model-dump",
        "note": "independent dump of mjModel mesh polygon arrays for the real cube fixture",
        "mujoco_version": mujoco.__version__,
        "interpreter": sys.executable,
        "py_version": ".".join(map(str, sys.version_info[:3])),
        "fixtures": {},
    }
    for name in ["mesh-box", "mesh-cylinder"]:
        xml = FIXTURES / f"{name}.xml"
        model = mujoco.MjModel.from_xml_path(str(xml))
        dataid = int(model.geom_dataid[0])
        assert dataid >= 0, f"{name}: geom 0 is not a mesh"
        polynum = np.asarray(model.mesh_polynum, dtype=np.int32)
        polyadr = np.asarray(model.mesh_polyadr, dtype=np.int32)
        polynormal = np.asarray(model.mesh_polynormal, dtype=np.float64)
        polyvertadr = np.asarray(model.mesh_polyvertadr, dtype=np.int32)
        polyvertnum = np.asarray(model.mesh_polyvertnum, dtype=np.int32)
        polyvert = np.asarray(model.mesh_polyvert, dtype=np.int32)
        polymapadr = np.asarray(model.mesh_polymapadr, dtype=np.int32)
        polymapnum = np.asarray(model.mesh_polymapnum, dtype=np.int32)
        polymap = np.asarray(model.mesh_polymap, dtype=np.int32)
        # Per-mesh slice of each (nmesh,) array; flat arrays are global and, with a
        # single mesh asset, equal to the mesh's own data.
        i = dataid
        mesh = {
            "xml_sha256": sha256(xml),
            "nmesh": int(model.nmesh),
            "mesh_dataid": dataid,
            "nmeshpoly": int(model.nmeshpoly),
            "nmeshvert": int(model.nmeshvert),
            "nmeshface": int(model.nmeshface),
            "vertnum": int(model.mesh_vertnum[i]),
            "graphadr": int(model.mesh_graphadr[i]),
            "polynum": int(polynum[i]),
            "polyadr": int(polyadr[i]),
            "array_lengths": {
                "mesh_polynum": int(polynum.size),
                "mesh_polyadr": int(polyadr.size),
                "mesh_polynormal_flat": int(polynormal.size),
                "mesh_polyvertadr": int(polyvertadr.size),
                "mesh_polyvertnum": int(polyvertnum.size),
                "mesh_polyvert": int(polyvert.size),
                "mesh_polymapadr": int(polymapadr.size),
                "mesh_polymapnum": int(polymapnum.size),
                "mesh_polymap": int(polymap.size),
            },
            "arrays": {
                "polynum": [int(v) for v in polynum],
                "polyadr": [int(v) for v in polyadr],
                "polynormal": [float(v) for v in polynormal.reshape(-1)],
                "polynormal_bits": f32_bits(polynormal),
                "polyvertadr": [int(v) for v in polyvertadr],
                "polyvertnum": [int(v) for v in polyvertnum],
                "polyvert": [int(v) for v in polyvert],
                "polymapadr": [int(v) for v in polymapadr],
                "polymapnum": [int(v) for v in polymapnum],
                "polymap": [int(v) for v in polymap],
            },
            "sums": {
                "sum_polynum": int(polynum.sum()),
                "nmeshpoly_from_polynormal": int(polynormal.size // 3),
                "sum_polyvertnum": int(polyvertnum.sum()),
                "len_polyvert": int(polyvert.size),
                "sum_polymapnum": int(polymapnum.sum()),
                "len_polymap": int(polymap.size),
            },
            "polyadr_equals_cumsum_polynum": bool(
                np.array_equal(polyadr, np.concatenate(([0], np.cumsum(polynum)[:-1])))
            ),
            "polyvertadr_equals_cumsum_polyvertnum": bool(
                np.array_equal(
                    polyvertadr, np.concatenate(([0], np.cumsum(polyvertnum)[:-1]))
                )
            ),
            "polymapadr_equals_cumsum_polymapnum": bool(
                np.array_equal(polymapadr, np.concatenate(([0], np.cumsum(polymapnum)[:-1])))
            ),
            "polyvert_in_vertex_range": [bool(0 <= int(v) < int(model.mesh_vertnum[i])) for v in polyvert],
            "polymap_in_polygon_range": [bool(0 <= int(v) < int(model.nmeshpoly)) for v in polymap],
        }
        report["fixtures"][name] = mesh

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    for name, mesh in report["fixtures"].items():
        print(name, "lengths:", mesh["array_lengths"])
        print(name, "sums:", mesh["sums"])
    print("wrote", OUT)


if __name__ == "__main__":
    main()
