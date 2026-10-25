"""QA independent G0b review: dump MuJoCo 3.15 side with QA's own code (no reuse of delivery scripts)."""

import hashlib
import json
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g0b"
FIX = ROOT / "fixtures/g10-p5"
MODELS = {
    "hfield-box": "fixtures/mjwarp-complex-physics/hfield-box.xml",
    "sdf-cylinder": "fixtures/mjwarp-complex-physics/sdf-cylinder.xml",
}


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def dump_fields(model, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    fields = {}

    def put(name, dtype, array):
        raw = np.ascontiguousarray(np.asarray(array).reshape(-1), dtype=dtype).tobytes()
        (out_dir / (name + ".bin")).write_bytes(raw)
        fields[name] = {"dtype": np.dtype(dtype).name, "count": len(raw) // np.dtype(dtype).itemsize,
                        "bytes": len(raw), "sha256": sha256_bytes(raw)}

    put("counts", "<i8", [model.ngeom, model.nhfield, model.nhfielddata, model.nmesh, model.noct, model.nbody])
    put("nq_nv", "<i8", [model.nq, model.nv])
    put("hfield_nrow", "<i4", model.hfield_nrow)
    put("hfield_ncol", "<i4", model.hfield_ncol)
    put("hfield_adr", "<i4", model.hfield_adr)
    put("hfield_size", "<f8", model.hfield_size)
    put("hfield_data", "<f4", model.hfield_data)
    put("oct_depth", "<i4", model.oct_depth)
    put("oct_child", "<i4", model.oct_child)
    put("oct_aabb", "<f8", model.oct_aabb)
    put("oct_coeff", "<f8", model.oct_coeff)
    put("mesh_octadr", "<i4", model.mesh_octadr)
    put("mesh_octnum", "<i4", model.mesh_octnum)
    put("opt_sdf", "<i4", [model.opt.sdf_iterations, model.opt.sdf_initpoints])
    put("geom_type", "<i4", model.geom_type)
    put("geom_dataid", "<i4", model.geom_dataid)
    put("geom_size", "<f8", model.geom_size)
    put("geom_margin", "<f8", model.geom_margin)
    doc = {
        "mujoco_version": mujoco.__version__,
        "counts": {"ngeom": int(model.ngeom), "nhfield": int(model.nhfield),
                   "nhfielddata": int(model.nhfielddata), "nmesh": int(model.nmesh),
                   "noct": int(model.noct), "nbody": int(model.nbody),
                   "nq": int(model.nq), "nv": int(model.nv)},
        "shapes": {name: list(np.asarray(getattr(model, name)).shape)
                   for name in ["oct_depth", "oct_child", "oct_aabb", "oct_coeff",
                                "hfield_data", "hfield_size"]},
        "fields": fields,
    }
    (out_dir / "fields.json").write_text(json.dumps(doc, indent=1), encoding="utf-8")
    return doc


def probe_load(path: Path):
    try:
        model = mujoco.MjModel.from_binary_path(str(path))
    except Exception as exc:  # noqa: BLE001 - report exact error text
        return {"ok": False, "error_type": type(exc).__name__, "error": str(exc)}
    result = {"ok": True, "ngeom": int(model.ngeom), "noct": int(model.noct), "nq": int(model.nq)}
    del model
    return result


def main():
    qa_mjb = QA / "mjb-315"
    report = {"mujoco_version": mujoco.__version__, "models": {}, "cross_version": {}}
    for name, rel_xml in MODELS.items():
        xml = ROOT / rel_xml
        fixture_mjb = FIX / "mjb-315" / (name + ".mjb")
        my_mjb = qa_mjb / (name + ".mjb")
        qa_mjb.mkdir(parents=True, exist_ok=True)

        # 1) same-version reload of the delivered artifact
        fixture_model = mujoco.MjModel.from_binary_path(str(fixture_mjb))
        frommjb = dump_fields(fixture_model, QA / "dump-315" / name / "frommjb")

        # 2) QA's own compile+save+reload from the source XML
        compiled = mujoco.MjModel.from_xml_path(str(xml))
        mujoco.mj_saveModel(compiled, str(my_mjb))
        reloaded = mujoco.MjModel.from_binary_path(str(my_mjb))
        fromxml = dump_fields(reloaded, QA / "dump-315" / name / "fromxml")

        report["models"][name] = {
            "source_xml": rel_xml,
            "source_xml_sha256": sha256_bytes(xml.read_bytes()),
            "fixture_mjb": {"path": str(fixture_mjb.relative_to(ROOT)).replace("\\", "/"),
                            "bytes": fixture_mjb.stat().st_size,
                            "sha256": sha256_bytes(fixture_mjb.read_bytes())},
            "qa_mjb": {"path": str(my_mjb.relative_to(ROOT)).replace("\\", "/"),
                       "bytes": my_mjb.stat().st_size,
                       "sha256": sha256_bytes(my_mjb.read_bytes())},
            "qa_save_equals_fixture_mjb": sha256_bytes(my_mjb.read_bytes()) == sha256_bytes(fixture_mjb.read_bytes()),
            "counts_frommjb": frommjb["counts"],
            "counts_fromxml": fromxml["counts"],
            "shapes": frommjb["shapes"],
        }
        print("315 %-13s noct=%d mjb_equal_fixture=%s" % (
            name, frommjb["counts"]["noct"], report["models"][name]["qa_save_equals_fixture_mjb"]))
        del fixture_model, compiled, reloaded

    # 3) cross-version load probes (3.15 must reject 3.12 artifacts)
    for name in MODELS:
        r312 = FIX / "mjb-312" / (name + ".mjb")
        r315 = FIX / "mjb-315" / (name + ".mjb")
        report["cross_version"][name] = {
            "load_312_under_315": probe_load(r312),
            "load_315_under_315": probe_load(r315),
        }
        print("cross %-13s 312_under_315=%s" % (name, json.dumps(report["cross_version"][name]["load_312_under_315"])[:160]))

    out = QA / "dump-315" / "qa_315_report.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("wrote", out)


if __name__ == "__main__":
    sys.exit(main())
