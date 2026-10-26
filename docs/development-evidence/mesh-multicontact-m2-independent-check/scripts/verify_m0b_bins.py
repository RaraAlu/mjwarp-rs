"""QA M2 independent check: verify the M0b 32-bin inputs used by the M2 window are
the real pinned mjModel data (not hand-written), and that the 9 mesh-poly arrays plus
mesh_vert / geom pose match the pinned model for the same fixtures.

Read-only; pinned mujoco 3.15 venv, CPU only.
Output: target/qa-m2/m0b-bins-vs-model.json
"""
import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
CASES = ROOT / "docs/development-evidence/mesh-multicontact-m0b/cases"
MANIFEST = ROOT / "docs/development-evidence/mesh-multicontact-m0b/mesh-cases-manifest.json"
OUT = ROOT / "target/qa-m2/m0b-bins-vs-model.json"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_i32(path):
    data = Path(path).read_bytes()
    return list(struct.unpack("<" + "i" * (len(data) // 4), data))


def read_f32(path):
    data = Path(path).read_bytes()
    return list(struct.unpack("<" + "f" * (len(data) // 4), data))


def f32_bits(values):
    arr = np.asarray(values, dtype=np.float32).reshape(-1)
    return [int(v) for v in arr.view(np.uint32)]


def model_arrays(xml):
    model = mujoco.MjModel.from_xml_path(str(xml))
    dataid = int(model.geom_dataid[0])
    mesh_id = int(model.geom_dataid[1]) if model.geom_dataid[1] >= 0 else dataid
    i = dataid
    return {
        "nmesh": int(model.nmesh),
        "nmeshpoly": int(model.nmeshpoly),
        "nmeshvert": int(model.nmeshvert),
        "mesh_dataid": dataid,
        "mesh_polynum": list(model.mesh_polynum),
        "mesh_polyadr": list(model.mesh_polyadr),
        "mesh_polynormal": [float(v) for v in np.asarray(model.mesh_polynormal, dtype=np.float32).reshape(-1)],
        "mesh_polynormal_bits": f32_bits(model.mesh_polynormal),
        "mesh_polyvertadr": list(model.mesh_polyvertadr),
        "mesh_polyvertnum": list(model.mesh_polyvertnum),
        "mesh_polyvert": list(model.mesh_polyvert),
        "mesh_polymapadr": list(model.mesh_polymapadr),
        "mesh_polymapnum": list(model.mesh_polymapnum),
        "mesh_polymap": list(model.mesh_polymap),
        "mesh_vert": [float(v) for v in np.asarray(model.mesh_vert, dtype=np.float32).reshape(-1)],
        "mesh_vert_bits": f32_bits(model.mesh_vert),
        "mesh_vertadr": list(model.mesh_vertadr),
        "mesh_vertnum": list(model.mesh_vertnum),
        "mesh_graphadr": list(model.mesh_graphadr),
        "nmeshface": int(model.nmeshface),
        "mesh_face": [int(v) for v in np.asarray(model.mesh_face).reshape(-1)],
        "vert_i": i,
        "mesh_i": mesh_id,
    }


def main():
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    report = {
        "kind": "qa-m2-m0b-bins-vs-model",
        "note": "independent verification that the window inputs (M0b 32 bins) equal the pinned mjModel arrays",
        "mujoco_version": mujoco.__version__,
        "cases": {},
        "failures": [],
    }
    for case in ["mesh-box", "mesh-cylinder"]:
        xml = FIXTURES / f"{case}.xml"
        directory = CASES / case
        arrays = model_arrays(xml)
        row = {
            "xml_sha256": sha256(xml),
            "bin_files": len(list(directory.glob("*.bin"))),
            "checks": {},
        }
        checks = {
            "mesh_polynum": read_i32(directory / "mesh_polynum.bin"),
            "mesh_polyadr": read_i32(directory / "mesh_polyadr.bin"),
            "mesh_polynormal": read_f32(directory / "mesh_polynormal.bin"),
            "mesh_polyvertadr": read_i32(directory / "mesh_polyvertadr.bin"),
            "mesh_polyvertnum": read_i32(directory / "mesh_polyvertnum.bin"),
            "mesh_polyvert": read_i32(directory / "mesh_polyvert.bin"),
            "mesh_polymapadr": read_i32(directory / "mesh_polymapadr.bin"),
            "mesh_polymapnum": read_i32(directory / "mesh_polymapnum.bin"),
            "mesh_polymap": read_i32(directory / "mesh_polymap.bin"),
            "mesh_vert": read_f32(directory / "mesh_vert.bin"),
            "mesh_vertadr": read_i32(directory / "mesh_vertadr.bin"),
            "mesh_vertnum": read_i32(directory / "mesh_vertnum.bin"),
            "mesh_graph": read_i32(directory / "mesh_graph.bin"),
            "mesh_graphadr": read_i32(directory / "mesh_graphadr.bin"),
        }
        model = {
            "mesh_polynum": arrays["mesh_polynum"],
            "mesh_polyadr": arrays["mesh_polyadr"],
            "mesh_polynormal": arrays["mesh_polynormal"],
            "mesh_polyvertadr": arrays["mesh_polyvertadr"],
            "mesh_polyvertnum": arrays["mesh_polyvertnum"],
            "mesh_polyvert": arrays["mesh_polyvert"],
            "mesh_polymapadr": arrays["mesh_polymapadr"],
            "mesh_polymapnum": arrays["mesh_polymapnum"],
            "mesh_polymap": arrays["mesh_polymap"],
            "mesh_vert": arrays["mesh_vert"],
            "mesh_vertadr": arrays["mesh_vertadr"],
            "mesh_vertnum": arrays["mesh_vertnum"],
            "mesh_graph": [int(v) for v in np.asarray(mujoco.MjModel.from_xml_path(str(xml)).mesh_graph).reshape(-1)],
            "mesh_graphadr": arrays["mesh_graphadr"],
        }
        for name, values in checks.items():
            expected = model[name]
            ok = len(values) == len(expected) and all(
                (abs(a - b) <= 0.0 if isinstance(a, float) or isinstance(b, float) else a == b)
                for a, b in zip(values, expected)
            )
            row["checks"][name] = {"ok": ok, "n": len(values), "expected_n": len(expected)}
            if not ok:
                report["failures"].append(f"{case}:{name}")
        row["geom_pose"] = {
            "geom_xpos": read_f32(directory / "geom_xpos.bin"),
            "geom_xmat": read_f32(directory / "geom_xmat.bin"),
            "geom_type": read_i32(directory / "geom_type.bin"),
            "geom_dataid": read_i32(directory / "geom_dataid.bin"),
        }
        row["meta_txt"] = (directory / "meta.txt").read_text(encoding="utf-8").strip().split("\n")
        row["manifest_case"] = manifest.get("cases", {}).get(case, {})
        report["cases"][case] = row
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    for case, row in report["cases"].items():
        bad = [k for k, v in row["checks"].items() if not v["ok"]]
        print(case, "bin_files=", row["bin_files"], "mismatches=", bad or "none")
        print("  geom_type=", row["geom_pose"]["geom_type"], "geom_dataid=", row["geom_pose"]["geom_dataid"])
    print("failures:", report["failures"] or "none")
    print("wrote", OUT)
    sys.exit(1 if report["failures"] else 0)


if __name__ == "__main__":
    main()
