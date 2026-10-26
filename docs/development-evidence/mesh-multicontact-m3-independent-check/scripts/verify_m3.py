"""QA M3 independent verification.

- fixture + 32 bins: sha/count/dtype vs mesh-case-manifest.json; 9 mesh-poly slices and
  their sums/ranges/cumsum/value-domain; values vs a fresh pinned mjModel dump
- same source: xml sha256, qpos_bits, geom_xpos/geom_xmat bits recomputed from the pinned
  model/keyframe, compared bit-for-bit with the M3 GPU reference and with the bins
- meta.txt contact params vs the M3 reference first row; pinned CPU collision count
- manifest: 18 self-check rules, result/failures, cross_check actual/reference
- capture window: raw A/B/committed reference byte identity, captured_at vs window start
- validation window: guard.log + samples.csv aggregates (authoritative path; facts.txt
  guard_aggregate line is the known invalid line)

Read-only; pinned venv; the capture/validation windows are NOT re-run.
Output: target/qa-m3/verify_m3.json
"""
import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
M3 = ROOT / "docs/development-evidence/mesh-multicontact-m3"
CASES = M3 / "cases"
MANIFEST = M3 / "mesh-case-manifest.json"
RECEIPT = M3 / "receipt.json"
M3_REF = M3 / "upstream-mesh-multicontact-mesh-mesh-gpu.json"
M1_REF = ROOT / "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json"
FIXTURE = ROOT / "fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml"
CAPTURE_DIR = ROOT / "target/m3-mesh-mesh/capture-attempt1"
WINDOW_DIR = ROOT / "target/m3-mesh-mesh/attempt1"
OUT = ROOT / "target/qa-m3/verify_m3.json"

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


def fit32(values) -> list[float]:
    return [float(v) for v in np.asarray(values, dtype=np.float32).reshape(-1)]


def window_aggregates(csv_path: Path) -> dict:
    lines = csv_path.read_text(encoding="utf-8").strip().split("\n")
    header = lines[0].split(",")
    rows = [dict(zip(header, line.split(","))) for line in lines[1:]]
    used = [int(r["used_mib"]) for r in rows]
    free = [int(r["free_mib"]) for r in rows]
    util = [int(r["util_percent"]) for r in rows]
    streak = cur = 0
    for value in util:
        if value >= 80:
            cur += 1
            streak = max(streak, cur)
        else:
            cur = 0
    return {
        "samples": len(rows),
        "used_peak": max(used),
        "used_first": used[0],
        "used_min": min(used),
        "free_min": min(free),
        "util_max": max(util),
        "util_streak_80": streak,
        "first_ts": rows[0]["timestamp"],
        "last_ts": rows[-1]["timestamp"],
    }


def main():
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    receipt = json.loads(RECEIPT.read_text(encoding="utf-8"))
    reference = json.loads(M3_REF.read_text(encoding="utf-8"))
    report = {"kind": "qa-m3-verify", "failures": []}

    def fail(message: str):
        report["failures"].append(message)

    # fixture / model
    model = mujoco.MjModel.from_xml_path(str(FIXTURE))
    key = model.key("capture")
    qpos = np.asarray(model.key_qpos[key.id], dtype=np.float32)
    data = mujoco.MjData(model)
    data.qpos[:] = qpos
    mujoco.mj_forward(model, data)
    dataid = [int(v) for v in model.geom_dataid]
    report["fixture"] = {
        "path": "fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml",
        "sha256_measured": sha256(FIXTURE),
        "sha256_delivery": "bb9ffb2f979129bbac410e9959088528f35180d14f5572c9a52631fa2ed86d8d",
        "size": FIXTURE.stat().st_size,
        "nmesh": int(model.nmesh),
        "ngeom": int(model.ngeom),
        "geom_type": [int(v) for v in model.geom_type],
        "geom_dataid": dataid,
        "nmeshpoly": int(model.nmeshpoly),
        "nmeshvert": int(model.nmeshvert),
        "mesh_polynum": [int(v) for v in model.mesh_polynum],
        "mesh_polyadr": [int(v) for v in model.mesh_polyadr],
        "mesh_vertadr": [int(v) for v in model.mesh_vertadr],
        "mesh_vertnum": [int(v) for v in model.mesh_vertnum],
    }
    if sha256(FIXTURE) != "bb9ffb2f979129bbac410e9959088528f35180d14f5572c9a52631fa2ed86d8d":
        fail("fixture sha mismatch")
    if int(model.nmesh) != 2 or [int(v) for v in model.geom_type] != [7, 7] or dataid != [0, 1]:
        fail("fixture mesh/geom topology mismatch")

    case = "mesh-mesh"
    directory = CASES / case
    arrays = {}
    arrays_ok = 0
    for name in manifest["arrays_order"]:
        entry = manifest["cases"][case]["arrays"][name]
        path = directory / entry["file"]
        raw = path.read_bytes() if path.is_file() else b""
        ok = (
            path.is_file()
            and sha256(path) == entry["sha256"]
            and len(raw) // 4 == entry["count"]
            and entry["dtype"] == DTYPES[name]
            and len(raw) == entry["count"] * 4
        )
        arrays_ok += 1 if ok else 0
        arrays[name] = read_values(path, DTYPES[name]) if path.is_file() else []
        if not ok:
            fail(f"array {name}")
    report["bins"] = {"arrays_ok": arrays_ok, "array_count": len(manifest["arrays_order"]), "counts": {n: len(v) for n, v in arrays.items()}}
    if arrays_ok != 32:
        fail(f"arrays_ok={arrays_ok}")

    # manifest structure
    report["manifest"] = {
        "sha256_measured": sha256(MANIFEST),
        "sha256_receipt": receipt.get("bins", {}).get("manifest_sha256")
        or receipt.get("manifest", {}).get("sha256"),
        "size": MANIFEST.stat().st_size,
        "result": manifest.get("result"),
        "failures": manifest.get("failures"),
        "selfcheck_rule_count": len(manifest["cases"][case]["cpu_selfcheck"]["rules"]),
        "selfcheck_result": manifest["cases"][case]["cpu_selfcheck"]["result"],
        "cross_check": manifest["cases"][case]["cross_check"],
    }
    if report["manifest"]["sha256_measured"] != report["manifest"]["sha256_receipt"]:
        fail("manifest sha vs receipt")
    if manifest.get("result") != "PASS" or manifest.get("failures"):
        fail("manifest result/failures")
    if report["manifest"]["selfcheck_rule_count"] != 18:
        fail("selfcheck rule count != 18")
    if report["manifest"]["selfcheck_result"] != "PASS":
        fail("selfcheck result")

    # 9 mesh-poly arrays vs pinned model
    poly_names = ["mesh_polynum", "mesh_polyadr", "mesh_polynormal", "mesh_polyvertadr",
                  "mesh_polyvertnum", "mesh_polyvert", "mesh_polymapadr", "mesh_polymapnum",
                  "mesh_polymap"]
    model_poly = {
        "mesh_polynum": [int(v) for v in model.mesh_polynum],
        "mesh_polyadr": [int(v) for v in model.mesh_polyadr],
        "mesh_polynormal": fit32(model.mesh_polynormal),
        "mesh_polyvertadr": [int(v) for v in model.mesh_polyvertadr],
        "mesh_polyvertnum": [int(v) for v in model.mesh_polyvertnum],
        "mesh_polyvert": [int(v) for v in model.mesh_polyvert],
        "mesh_polymapadr": [int(v) for v in model.mesh_polymapadr],
        "mesh_polymapnum": [int(v) for v in model.mesh_polymapnum],
        "mesh_polymap": [int(v) for v in model.mesh_polymap],
    }
    lengths = [len(arrays[n]) for n in poly_names]
    nmesh = len(arrays["mesh_polynum"])
    nmeshpoly = len(arrays["mesh_polynormal"]) // 3
    nmeshvert = len(arrays["mesh_polymapadr"])
    checks = {
        "lengths_match": lengths == [2, 2, 36, 12, 12, 48, 16, 16, 48],
        "values_equal_to_mjmodel": all(arrays[n] == model_poly[n] for n in poly_names),
        "sum_polynum_eq_nmeshpoly": sum(arrays["mesh_polynum"]) == nmeshpoly,
        "sum_polyvertnum_eq_len": sum(arrays["mesh_polyvertnum"]) == len(arrays["mesh_polyvert"]),
        "sum_polymapnum_eq_len": sum(arrays["mesh_polymapnum"]) == len(arrays["mesh_polymap"]),
        "vertadr_cumsum": arrays["mesh_vertadr"] == [0] + [sum(arrays["mesh_vertnum"][: i + 1]) for i in range(nmesh - 1)],
        "polyadr_cumsum": arrays["mesh_polyadr"] == [0] + [sum(arrays["mesh_polynum"][: i + 1]) for i in range(nmesh - 1)],
        "polyvertadr_cumsum": arrays["mesh_polyvertadr"] == [0] + [sum(arrays["mesh_polyvertnum"][: i + 1]) for i in range(nmeshpoly - 1)],
        "polymapadr_cumsum": arrays["mesh_polymapadr"] == [0] + [sum(arrays["mesh_polymapnum"][: i + 1]) for i in range(nmeshvert - 1)],
    }
    # per-mesh ranges
    ranges_ok = True
    for mesh_index in range(nmesh):
        adr, num = arrays["mesh_polyadr"][mesh_index], arrays["mesh_polynum"][mesh_index]
        ranges_ok &= 0 <= adr and 0 <= num and adr + num <= nmeshpoly
        vadr, vnum = arrays["mesh_vertadr"][mesh_index], arrays["mesh_vertnum"][mesh_index]
        ranges_ok &= 0 <= vadr and 0 <= vnum and vadr + vnum <= nmeshvert
    for polygon in range(nmeshpoly):
        adr, num = arrays["mesh_polyvertadr"][polygon], arrays["mesh_polyvertnum"][polygon]
        ranges_ok &= 0 <= adr and 0 <= num and adr + num <= len(arrays["mesh_polyvert"])
    for vertex in range(nmeshvert):
        adr, num = arrays["mesh_polymapadr"][vertex], arrays["mesh_polymapnum"][vertex]
        ranges_ok &= 0 <= adr and 0 <= num and adr + num <= len(arrays["mesh_polymap"])
    checks["ranges_ok"] = bool(ranges_ok)
    # value domains (O7 semantics)
    domain_ok = True
    for mesh_index in range(nmesh):
        vertnum = arrays["mesh_vertnum"][mesh_index]
        polyadr, polynum = arrays["mesh_polyadr"][mesh_index], arrays["mesh_polynum"][mesh_index]
        for polygon in range(polyadr, polyadr + polynum):
            adr, num = arrays["mesh_polyvertadr"][polygon], arrays["mesh_polyvertnum"][polygon]
            for entry in range(adr, adr + num):
                domain_ok &= 0 <= arrays["mesh_polyvert"][entry] < vertnum
        vertadr = arrays["mesh_vertadr"][mesh_index]
        for vertex in range(vertadr, vertadr + arrays["mesh_vertnum"][mesh_index]):
            adr, num = arrays["mesh_polymapadr"][vertex], arrays["mesh_polymapnum"][vertex]
            for entry in range(adr, adr + num):
                domain_ok &= 0 <= arrays["mesh_polymap"][entry] < polynum
    checks["value_domains_ok"] = bool(domain_ok)
    checks["dataid_in_mesh_range"] = all(0 <= d < nmesh for t, d in zip([int(v) for v in model.geom_type], dataid) if t == 7)
    report["mesh_poly"] = {"lengths": lengths, "checks": checks, "nmesh": nmesh, "nmeshpoly": nmeshpoly, "nmeshvert": nmeshvert}
    for key, ok in checks.items():
        if not ok:
            fail(f"mesh_poly {key}")

    # same source
    ref_case = reference["cases"][case]
    ref_xpos = np.asarray([v for row in ref_case["output"]["geom_xpos"] for v in row], dtype=np.float32)
    ref_xmat = np.asarray([v for row in ref_case["output"]["geom_xmat"] for v in row], dtype=np.float32)
    geom_xpos = np.asarray(data.geom_xpos, dtype=np.float32).reshape(-1)
    geom_xmat = np.asarray(data.geom_xmat, dtype=np.float32).reshape(-1)
    same = {
        "xml_sha256_recomputed": sha256(FIXTURE),
        "xml_manifest": manifest["cases"][case]["xml_sha256"],
        "xml_reference": ref_case["xml_sha256"],
        "xml_match": sha256(FIXTURE) == manifest["cases"][case]["xml_sha256"] == ref_case["xml_sha256"],
        "qpos_bits_recomputed": bits32(qpos),
        "qpos_bits_manifest": manifest["cases"][case]["qpos_bits"],
        "qpos_bits_reference": ref_case["qpos_bits"],
        "qpos_match": bits32(qpos) == manifest["cases"][case]["qpos_bits"] == ref_case["qpos_bits"],
        "geom_xpos_bits_recomputed": bits32(geom_xpos),
        "geom_xpos_bits_bin": bits32(arrays["geom_xpos"]),
        "geom_xpos_bits_reference": bits32(ref_xpos),
        "geom_xpos_bit_match": bits32(geom_xpos) == bits32(arrays["geom_xpos"]) == bits32(ref_xpos),
        "geom_xmat_bits_recomputed": bits32(geom_xmat),
        "geom_xmat_bits_bin": bits32(arrays["geom_xmat"]),
        "geom_xmat_bits_reference": bits32(ref_xmat),
        "geom_xmat_bit_match": bits32(geom_xmat) == bits32(arrays["geom_xmat"]) == bits32(ref_xmat),
    }
    report["same_source"] = same
    for key in ["xml_match", "qpos_match", "geom_xpos_bit_match", "geom_xmat_bit_match"]:
        if not same[key]:
            fail(f"same source {key}")

    # meta.txt vs reference first row
    meta = {}
    for line in (directory / "meta.txt").read_text(encoding="utf-8").split("\n"):
        parts = line.split()
        if parts:
            meta[parts[0]] = parts[1:]
    ref_first = {name: ref_case["output"]["fields"][name][0] for name in ["friction", "solref", "solreffriction", "solimp", "adhesion"]}
    meta_checks = {
        "g1_g2": int(meta["g1"][0]) == 0 and int(meta["g2"][0]) == 1,
        "type1_type2_mesh": int(meta["type1"][0]) == 7 and int(meta["type2"][0]) == 7,
        "friction_bits": bits32([float(v) for v in meta["friction"]]) == bits32(ref_first["friction"]),
        "solref_bits": bits32([float(v) for v in meta["solref"]]) == bits32(ref_first["solref"]),
        "solreffriction_bits": bits32([float(v) for v in meta["solreffriction"]]) == bits32(ref_first["solreffriction"]),
        "solimp_bits": bits32([float(v) for v in meta["solimp"]]) == bits32(ref_first["solimp"]),
        "adhesion_bits": bits32([float(meta["adhesion"][0])]) == bits32(ref_first["adhesion"]),
    }
    report["meta_checks"] = meta_checks
    for key, ok in meta_checks.items():
        if not ok:
            fail(f"meta {key}")

    # CPU collision count cross evidence
    try:
        import mujoco_warp as mjw
        import warp as wp

        wp.set_device("cpu")
        host = mujoco.MjModel.from_xml_path(str(FIXTURE))
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
        report["cpu_collision"] = {"nacon": count, "reference": ref_case["output"]["count"], "match": count == ref_case["output"]["count"]}
        if count != ref_case["output"]["count"]:
            fail("cpu collision count")
    except Exception as error:  # noqa: BLE001
        report["cpu_collision"] = {"error": str(error)}

    # M3 reference content + old M1 reference untouched
    report["references"] = {
        "m3_sha256": sha256(M3_REF),
        "m3_size": M3_REF.stat().st_size,
        "m3_count": ref_case["output"]["count"],
        "m3_geom": ref_case["output"]["fields"]["geom"],
        "m3_dist": ref_case["output"]["fields"]["dist"],
        "m3_geomcollisionid": ref_case["output"]["fields"]["geomcollisionid"],
        "m3_two_captures": ref_case["two_captures_bitwise_identical"],
        "m3_captured_at": reference["provenance"]["captured_at"],
        "m3_geom_dataid": ref_case["geom_dataid"],
        "m1_sha256": sha256(M1_REF),
        "m1_size": M1_REF.stat().st_size,
    }
    if report["references"]["m3_sha256"] != "96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1":
        fail("m3 ref sha")
    if report["references"]["m1_sha256"] != "c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a":
        fail("m1 ref sha changed")

    # capture window raw identity
    raw_a = CAPTURE_DIR / "upstream-mesh-multicontact-mesh-mesh-A.json"
    raw_b = CAPTURE_DIR / "upstream-mesh-multicontact-mesh-mesh-B.json"
    report["capture_window"] = {
        "raw_a_sha256": sha256(raw_a),
        "raw_b_sha256": sha256(raw_b),
        "committed_ref_sha256": sha256(M3_REF),
        "raw_ab_ref_byte_identical": sha256(raw_a) == sha256(raw_b) == sha256(M3_REF),
        "captured_at_equals_window_start": reference["provenance"]["captured_at"] == "2026-10-27T00:48:22.577+08:00",
        "aggregates": window_aggregates(CAPTURE_DIR / "samples.csv"),
        "guard_tail": [
            line
            for line in (CAPTURE_DIR / "guard.log").read_text(encoding="utf-8").split("\n")
            if line.startswith("guard_baseline") or line.startswith("guard_aggregates")
        ],
    }
    if not report["capture_window"]["raw_ab_ref_byte_identical"]:
        fail("capture raw A/B/ref mismatch")
    capture = report["capture_window"]["aggregates"]
    if (
        capture["samples"] != 87
        or capture["used_peak"] != 2278
        or capture["free_min"] != 13784
        or capture["util_max"] != 80
        or capture["util_streak_80"] != 2
    ):
        fail("capture aggregates mismatch")

    # validation window (authoritative: guard.log + samples.csv)
    validation = window_aggregates(WINDOW_DIR / "samples.csv")
    report["validation_window"] = {
        "aggregates": validation,
        "guard_tail": [
            line
            for line in (WINDOW_DIR / "guard.log").read_text(encoding="utf-8").split("\n")
            if line.startswith("guard_baseline") or line.startswith("guard_aggregates")
            or line.startswith("guard_stop")
        ],
        "facts_invalid_line_present": "guard_aggregate=System.Collections.Specialized.OrderedDictionary"
        in (WINDOW_DIR / "facts.txt").read_text(encoding="utf-8"),
        "delta_vs_guard_baseline": validation["used_peak"] - 2227,
        "delta_vs_first_baseline_point": validation["used_peak"] - 2222,
        "delta_vs_sample_min": validation["used_peak"] - validation["used_min"],
        "floor_formula": "min(10240, 13840-3072)=10240",
    }
    if (
        validation["samples"] != 732
        or validation["used_peak"] != 2505
        or validation["free_min"] != 13557
        or validation["util_max"] != 60
        or validation["util_streak_80"] != 0
    ):
        fail("validation aggregates mismatch")
    if report["validation_window"]["delta_vs_guard_baseline"] != 278:
        fail("validation delta vs guard baseline != 278")
    if report["validation_window"]["delta_vs_first_baseline_point"] != 283 or validation["used_min"] != 2222:
        fail("validation delta 283 basis mismatch")
    if not report["validation_window"]["facts_invalid_line_present"]:
        fail("facts.txt invalid guard_aggregate line not present (claim mismatch)")

    # decode the 9 frozen binaries listed in the delivery evidence (frozen copies)
    freeze_dir = ROOT / "target/m3-freeze"
    frozen = {}
    if freeze_dir.is_dir():
        for path in freeze_dir.glob("*.exe"):
            frozen[path.name] = sha256(path)
    report["frozen_binaries"] = {"dir": str(freeze_dir), "count": len(frozen), "shas": frozen}

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print("arrays_ok=", arrays_ok, "mesh_poly=", checks)
    print("same_source=", {k: same[k] for k in ["xml_match", "qpos_match", "geom_xpos_bit_match", "geom_xmat_bit_match"]})
    print("capture=", capture)
    print("validation=", validation)
    print("deltas:", report["validation_window"]["delta_vs_guard_baseline"],
          report["validation_window"]["delta_vs_first_baseline_point"],
          report["validation_window"]["delta_vs_sample_min"])
    print("frozen binaries found:", len(frozen))
    print("failures:", report["failures"] or "none")
    print("wrote", OUT)
    sys.exit(1 if report["failures"] else 0)


if __name__ == "__main__":
    main()

