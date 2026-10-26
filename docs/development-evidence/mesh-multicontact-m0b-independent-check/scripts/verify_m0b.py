"""QA M0b independent verification.

- 32 bins per case: file/dtype/count/sha256 vs mesh-cases-manifest.json + receipt
- same-source: xml sha256, qpos_bits, geom_xpos/geom_xmat bits recomputed from the
  pinned mujoco 3.15 model/keyframe and compared bit-for-bit with the M1 GPU reference
- 9 mesh-poly arrays: pinned dims/lengths, sums, per-entity ranges, strong consistency,
  cumsum layout, and value equality with a fresh mjModel dump
- meta.txt scalars: pair/meta values and contact params vs the M1 reference first row
- collision count cross evidence on the pinned CPU backend (fixture validity only)

Read-only, pinned venv, CPU device only (no CUDA context is created by this script).
Output: target/qa-m0b/verify_m0b.json
"""
import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
M0B = ROOT / "docs/development-evidence/mesh-multicontact-m0b"
CASES = M0B / "cases"
MANIFEST = M0B / "mesh-cases-manifest.json"
RECEIPT = M0B / "receipt.json"
M1_REF = ROOT / "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json"
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
OUT = ROOT / "target/qa-m0b/verify_m0b.json"

DTYPES = {
    "geom_xpos": "f32", "geom_xmat": "f32", "geom_size": "f32", "geom_margin": "f32",
    "geom_gap": "f32", "geom_rbound": "f32", "geom_aabb": "f32", "geom_type": "i32",
    "geom_dataid": "i32", "mesh_vertadr": "i32", "mesh_vertnum": "i32", "mesh_graphadr": "i32",
    "mesh_octadr": "i32", "mesh_vert": "f32", "mesh_graph": "i32",
    "mesh_polynum": "i32", "mesh_polyadr": "i32", "mesh_polynormal": "f32",
    "mesh_polyvertadr": "i32", "mesh_polyvertnum": "i32", "mesh_polyvert": "i32",
    "mesh_polymapadr": "i32", "mesh_polymapnum": "i32", "mesh_polymap": "i32",
    "hfield_size": "f32", "hfield_nrow": "i32", "hfield_ncol": "i32", "hfield_adr": "i32",
    "hfield_data": "f32", "oct_aabb": "f32", "oct_child": "i32", "oct_coeff": "f32",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_values(path: Path, dtype: str):
    raw = path.read_bytes()
    fmt = "<" + ("f" if dtype == "f32" else "i") * (len(raw) // 4)
    return list(struct.unpack(fmt, raw))


def bits32(values) -> list[int]:
    arr = np.asarray(values, dtype=np.float32).reshape(-1)
    return [int(v) for v in arr.view(np.uint32)]


def model_arrays(xml: Path) -> dict:
    model = mujoco.MjModel.from_xml_path(str(xml))
    key = model.key("capture")
    qpos = np.asarray(model.key_qpos[key.id], dtype=np.float32)
    data = mujoco.MjData(model)
    data.qpos[:] = qpos
    mujoco.mj_forward(model, data)
    return {"model": model, "qpos": qpos, "data": data}


def main():
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    receipt = json.loads(RECEIPT.read_text(encoding="utf-8"))
    reference = json.loads(M1_REF.read_text(encoding="utf-8"))
    report = {
        "kind": "qa-m0b-verify",
        "manifest_sha256_measured": sha256(MANIFEST),
        "manifest_sha256_receipt": receipt["outputs"]["manifest"]["sha256"],
        "manifest_size_measured": MANIFEST.stat().st_size,
        "manifest_size_receipt": receipt["outputs"]["manifest"]["size_bytes"],
        "m1_reference_sha256_measured": sha256(M1_REF),
        "authoring_script_sha256_measured": sha256(M0B / "scripts/prepare_mesh_cases.py"),
        "authoring_script_sha256_manifest": manifest["provenance"]["authoring_script_sha256"],
        "cases": {},
        "failures": [],
    }
    if report["manifest_sha256_measured"] != report["manifest_sha256_receipt"]:
        report["failures"].append("manifest sha vs receipt")
    if report["authoring_script_sha256_measured"] != report["authoring_script_sha256_manifest"]:
        report["failures"].append("authoring script sha vs manifest")

    # independent collision-count cross evidence on the pinned CPU backend (fixture validity only)
    try:
        import mujoco_warp as mjw
        import warp as wp

        wp.set_device("cpu")
        cpu_available = True
    except Exception as error:  # noqa: BLE001
        cpu_available = False
        wp = None
        mjw = None
        report["cpu_backend_error"] = str(error)

    for case in ["mesh-box", "mesh-cylinder"]:
        directory = CASES / case
        row = {"arrays": {}, "checks": {}, "failures": []}
        arrays_ok = 0
        for name in manifest["arrays_order"]:
            entry = manifest["cases"][case]["arrays"][name]
            path = directory / entry["file"]
            dtype = DTYPES[name]
            actual_sha = sha256(path) if path.is_file() else None
            raw = path.read_bytes() if path.is_file() else b""
            actual_count = len(raw) // 4
            ok = (
                actual_sha == entry["sha256"]
                and actual_count == entry["count"]
                and entry["dtype"] == dtype
                and len(raw) == entry["count"] * 4
            )
            row["arrays"][name] = {
                "ok": ok,
                "count": actual_count,
                "manifest_count": entry["count"],
                "dtype": dtype,
                "manifest_dtype": entry["dtype"],
                "size_bytes": len(raw),
                "sha_matches": actual_sha == entry["sha256"],
            }
            arrays_ok += 1 if ok else 0
            if not ok:
                row["failures"].append(f"array {name}")
        row["checks"]["array_count"] = manifest["cases"][case]["arrays"].__len__()
        row["checks"]["arrays_ok"] = arrays_ok
        if arrays_ok != 32:
            report["failures"].append(f"{case}: arrays_ok={arrays_ok}/32")

        # same source: xml / qpos / geom pose / geom xmat
        xml = FIXTURES / f"{case}.xml"
        md = model_arrays(xml)
        model = md["model"]
        qpos_bits = bits32(md["qpos"])
        geom_xpos = np.asarray(md["data"].geom_xpos, dtype=np.float32).reshape(-1)
        geom_xmat = np.asarray(md["data"].geom_xmat, dtype=np.float32).reshape(-1)
        reference_case = reference["cases"][case]
        reference_xpos = np.asarray(
            [v for row_ in reference_case["output"]["geom_xpos"] for v in row_], dtype=np.float32
        )
        reference_xmat = np.asarray(
            [v for row_ in reference_case["output"]["geom_xmat"] for v in row_], dtype=np.float32
        )
        bin_xpos = read_values(directory / "geom_xpos.bin", "f32")
        bin_xmat = read_values(directory / "geom_xmat.bin", "f32")
        row["same_source"] = {
            "xml_sha256_recomputed": sha256(xml),
            "xml_sha256_manifest": manifest["cases"][case]["xml_sha256"],
            "xml_sha256_reference": reference_case["xml_sha256"],
            "xml_match": sha256(xml) == manifest["cases"][case]["xml_sha256"] == reference_case["xml_sha256"],
            "qpos_bits_recomputed": qpos_bits,
            "qpos_bits_manifest": manifest["cases"][case]["qpos_bits"],
            "qpos_bits_reference": reference_case["qpos_bits"],
            "qpos_match": qpos_bits == manifest["cases"][case]["qpos_bits"] == reference_case["qpos_bits"],
            "geom_xpos_bits_recomputed": bits32(geom_xpos),
            "geom_xpos_bits_bin": bits32(bin_xpos),
            "geom_xpos_bits_reference": bits32(reference_xpos),
            "geom_xpos_bit_match": bits32(geom_xpos) == bits32(bin_xpos) == bits32(reference_xpos),
            "geom_xmat_bits_recomputed": bits32(geom_xmat),
            "geom_xmat_bits_bin": bits32(bin_xmat),
            "geom_xmat_bits_reference": bits32(reference_xmat),
            "geom_xmat_bit_match": bits32(geom_xmat) == bits32(bin_xmat) == bits32(reference_xmat),
        }
        for key in ["xml_match", "qpos_match", "geom_xpos_bit_match", "geom_xmat_bit_match"]:
            if not row["same_source"][key]:
                row["failures"].append(f"same source {key}")
                report["failures"].append(f"{case}: {key}")

        # 9 mesh-poly arrays vs pinned dims and a fresh mjModel dump
        cm = model_arrays(xml)["model"]
        dataid = int(model.geom_dataid[0])
        poly_arrays = {
            "mesh_polynum": list(cm.mesh_polynum),
            "mesh_polyadr": list(cm.mesh_polyadr),
            "mesh_polynormal": [float(v) for v in np.asarray(cm.mesh_polynormal, dtype=np.float32).reshape(-1)],
            "mesh_polyvertadr": list(cm.mesh_polyvertadr),
            "mesh_polyvertnum": list(cm.mesh_polyvertnum),
            "mesh_polyvert": list(cm.mesh_polyvert),
            "mesh_polymapadr": list(cm.mesh_polymapadr),
            "mesh_polymapnum": list(cm.mesh_polymapnum),
            "mesh_polymap": list(cm.mesh_polymap),
        }
        bin_poly = {}
        for name in ["mesh_polynum", "mesh_polyadr", "mesh_polynormal", "mesh_polyvertadr",
                     "mesh_polyvertnum", "mesh_polyvert", "mesh_polymapadr", "mesh_polymapnum",
                     "mesh_polymap"]:
            bin_poly[name] = read_values(directory / f"{name}.bin", DTYPES[name])
        lengths = [len(bin_poly[name]) for name in [
            "mesh_polynum", "mesh_polyadr", "mesh_polynormal", "mesh_polyvertadr",
            "mesh_polyvertnum", "mesh_polyvert", "mesh_polymapadr", "mesh_polymapnum",
            "mesh_polymap"]]
        nmesh = len(bin_poly["mesh_polynum"])
        nmeshpoly = len(bin_poly["mesh_polynormal"]) // 3
        nmeshvert = len(bin_poly["mesh_polymapadr"])
        sums = {
            "sum_polynum": sum(bin_poly["mesh_polynum"]),
            "nmeshpoly": nmeshpoly,
            "sum_polyvertnum": sum(bin_poly["mesh_polyvertnum"]),
            "len_polyvert": len(bin_poly["mesh_polyvert"]),
            "sum_polymapnum": sum(bin_poly["mesh_polymapnum"]),
            "len_polymap": len(bin_poly["mesh_polymap"]),
        }
        ranges_ok = (
            all(0 <= a and 0 <= n and a + n <= nmeshpoly for a, n in zip(bin_poly["mesh_polyadr"], bin_poly["mesh_polynum"]))
            and all(0 <= a and 0 <= n and a + n <= len(bin_poly["mesh_polyvert"]) for a, n in zip(bin_poly["mesh_polyvertadr"], bin_poly["mesh_polyvertnum"]))
            and all(0 <= a and 0 <= n and a + n <= len(bin_poly["mesh_polymap"]) for a, n in zip(bin_poly["mesh_polymapadr"], bin_poly["mesh_polymapnum"]))
        )
        cumsum_ok = all(
            bin_poly[adr_name] == [0] + [sum(bin_poly[num_name][: i + 1]) for i in range(len(bin_poly[num_name]) - 1)]
            for adr_name, num_name in [
                ("mesh_polyadr", "mesh_polynum"),
                ("mesh_polyvertadr", "mesh_polyvertnum"),
                ("mesh_polymapadr", "mesh_polymapnum"),
            ]
        )
        row["mesh_poly"] = {
            "lengths": lengths,
            "expected_lengths": [1, 1, 18, 6, 6, 24, 8, 8, 24],
            "lengths_match": lengths == [1, 1, 18, 6, 6, 24, 8, 8, 24],
            "sums": sums,
            "sums_match": sums["sum_polynum"] == nmeshpoly and sums["sum_polyvertnum"] == sums["len_polyvert"] and sums["sum_polymapnum"] == sums["len_polymap"],
            "ranges_ok": ranges_ok,
            "adr_equals_cumsum": cumsum_ok,
            "values_equal_to_mjmodel": all(bin_poly[name] == poly_arrays[name] for name in bin_poly),
            "manifest_selfcheck_result": manifest["cases"][case]["cpu_selfcheck"]["result"],
        }
        for key in ["lengths_match", "sums_match", "ranges_ok", "adr_equals_cumsum", "values_equal_to_mjmodel"]:
            if not row["mesh_poly"][key]:
                row["failures"].append(f"mesh_poly {key}")
                report["failures"].append(f"{case}: mesh_poly {key}")

        # meta.txt: parse and check pair identity + contact params against the M1 reference first row
        meta = {}
        for line in (directory / "meta.txt").read_text(encoding="utf-8").split("\n"):
            parts = line.split()
            if parts:
                meta[parts[0]] = parts[1:]
        ref_fields = reference_case["output"]["fields"]
        ref_first = {name: ref_fields[name][0] for name in ["friction", "solref", "solreffriction", "solimp", "adhesion", "includemargin"]}
        meta_checks = {
            "nworld": int(meta["nworld"][0]) == reference_case["nworld"],
            "ngeom": int(meta["ngeom"][0]) == reference_case["ngeom"] if "ngeom" in reference_case else int(meta["ngeom"][0]) == model.ngeom,
            "g1": int(meta["g1"][0]) == 1,
            "g2": int(meta["g2"][0]) == 0,
            "type1": int(meta["type1"][0]) == int(model.geom_type[1]),
            "type2": int(meta["type2"][0]) == int(model.geom_type[0]),
            "friction_bits": bits32([float(v) for v in meta["friction"]]) == bits32(ref_first["friction"]),
            "solref_bits": bits32([float(v) for v in meta["solref"]]) == bits32(ref_first["solref"]),
            "solreffriction_bits": bits32([float(v) for v in meta["solreffriction"]]) == bits32(ref_first["solreffriction"]),
            "solimp_bits": bits32([float(v) for v in meta["solimp"]]) == bits32(ref_first["solimp"]),
            "adhesion_bits": bits32([float(meta["adhesion"][0])]) == bits32(ref_first["adhesion"]),
        }
        row["meta_checks"] = meta_checks
        for key, ok in meta_checks.items():
            if not ok:
                row["failures"].append(f"meta {key}")
                report["failures"].append(f"{case}: meta {key}")

        # CPU collision count cross evidence (fixture validity only, same M1 precheck recipe)
        if cpu_available:
            try:
                host = mujoco.MjModel.from_xml_path(str(xml))
                key_id = host.key("capture").id
                qpos_host = np.asarray(host.key_qpos[key_id], dtype=np.float32)
                model_w = mjw.put_model(host)
                data_w = mjw.make_data(host, nworld=1, nconmax=4096, njmax=20000)
                data_w.qpos.assign(qpos_host.reshape(1, host.nq))
                data_w.qvel.assign(np.zeros((1, host.nv), dtype=np.float32))
                mjw.kinematics(model_w, data_w)
                mjw.collision(model_w, data_w)
                wp.synchronize_device()
                count = int(data_w.nacon.numpy()[0])
                row["cpu_collision"] = {"nacon": count, "reference": reference_case["output"]["count"], "match": count == reference_case["output"]["count"]}
                if count != reference_case["output"]["count"]:
                    report["failures"].append(f"{case}: cpu collision count {count}")
            except Exception as error:  # noqa: BLE001
                row["cpu_collision"] = {"error": str(error)}
        report["cases"][case] = row

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    for case, row in report["cases"].items():
        print(case, "arrays_ok=", row["checks"]["arrays_ok"], "/32", "same_source=",
              {k: row["same_source"][k] for k in ["xml_match", "qpos_match", "geom_xpos_bit_match", "geom_xmat_bit_match"]},
              "mesh_poly=", {k: row["mesh_poly"][k] for k in ["lengths_match", "sums_match", "ranges_ok", "adr_equals_cumsum", "values_equal_to_mjmodel"]},
              "cpu_collision=", row.get("cpu_collision"))
    print("failures:", report["failures"] or "none")
    print("wrote", OUT)
    sys.exit(1 if report["failures"] else 0)


if __name__ == "__main__":
    main()
