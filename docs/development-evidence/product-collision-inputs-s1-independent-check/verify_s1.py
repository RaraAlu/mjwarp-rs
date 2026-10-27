"""QA independent verification of the S1 collision-inputs oracle and field tables.

Read-only: loads the three frozen MJBs with the pinned MuJoCo 3.12.0 python
bindings, recomputes every count/field/qpos case from mjModel/mjData, and
compares against tests/support/collision_inputs_oracle.json. Also cross-checks
the three duplicated field tables (Rust spec, C++ switch, CollisionGpuInputs).

Writes only target/qa-s1/verify_s1.json (outside the repo tree of evidence).
"""
import hashlib
import json
import re
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s1/verify_s1.json"
ORACLE = ROOT / "tests/support/collision_inputs_oracle.json"

COUNT_NAMES = ["nq", "ngeom", "nmesh", "nmeshvert", "nmeshgraph", "nmeshpoly",
               "nmeshpolyvert", "nmeshpolymap", "nhfield", "nhfielddata", "noct"]

# id -> (name, kind, unit, width) mirrored from include/mjwarp_collision_inputs_probe.h
FIELD_SPEC = {
    0: ("geom_xpos", 4, "ngeom", 3), 1: ("geom_xmat", 4, "ngeom", 9),
    2: ("geom_size", 2, "ngeom", 3), 3: ("geom_margin", 2, "ngeom", 1),
    4: ("geom_gap", 2, "ngeom", 1), 5: ("geom_rbound", 2, "ngeom", 1),
    6: ("geom_aabb", 2, "ngeom", 6), 7: ("geom_type", 1, "ngeom", 1),
    8: ("geom_dataid", 1, "ngeom", 1), 9: ("mesh_vertadr", 1, "nmesh", 1),
    10: ("mesh_vertnum", 1, "nmesh", 1), 11: ("mesh_graphadr", 1, "nmesh", 1),
    12: ("mesh_octadr", 1, "nmesh", 1), 13: ("mesh_vert", 4, "nmeshvert", 3),
    14: ("mesh_graph", 1, "nmeshgraph", 1), 15: ("mesh_polynum", 1, "nmesh", 1),
    16: ("mesh_polyadr", 1, "nmesh", 1), 17: ("mesh_polynormal", 2, "nmeshpoly", 3),
    18: ("mesh_polyvertadr", 1, "nmeshpoly", 1),
    19: ("mesh_polyvertnum", 1, "nmeshpoly", 1),
    20: ("mesh_polyvert", 1, "nmeshpolyvert", 1),
    21: ("mesh_polymapadr", 1, "nmeshvert", 1),
    22: ("mesh_polymapnum", 1, "nmeshvert", 1),
    23: ("mesh_polymap", 1, "nmeshpolymap", 1),
    24: ("hfield_size", 2, "nhfield", 4), 25: ("hfield_nrow", 1, "nhfield", 1),
    26: ("hfield_ncol", 1, "nhfield", 1), 27: ("hfield_adr", 1, "nhfield", 1),
    28: ("hfield_data", 4, "nhfielddata", 1),
    29: ("oct_aabb", 2, "noct", 6), 30: ("oct_child", 1, "noct", 8),
    31: ("oct_coeff", 2, "noct", 8),
}
ATTR = {  # id -> mjModel attribute for the value-bearing field (None for mjData ids)
    0: None, 1: None,
}
for _id, (_n, _k, _u, _w) in FIELD_SPEC.items():
    if _id >= 2:
        ATTR[_id] = _n

problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def bits32(values):
    return np.asarray(values, dtype=np.float32).view(np.uint32).tolist()


def bits64(values):
    return np.asarray(values, dtype=np.float64).view(np.uint64).tolist()


def main():
    raw_oracle = ORACLE.read_bytes()
    oracle = json.loads(raw_oracle)
    report = {
        "oracle_sha256": hashlib.sha256(raw_oracle).hexdigest(),
        "oracle_bytes": len(raw_oracle),
        "mujoco": mujoco.__version__,
        "numpy": np.__version__,
        "python": sys.version.split()[0],
        "models": {},
        "field_table": {},
    }
    check(oracle["schema"] == 1, "oracle schema")

    by_name = {m["name"]: m for m in oracle["models"]}
    check(len(by_name) == 3, "oracle model count")

    for rel in ["fixtures/g10-p5/mjb-312/hfield-box.mjb",
                "fixtures/render-bvh/scene-a/scene-312.mjb",
                "fixtures/g10-p5/mjb-312/sdf-cylinder.mjb"]:
        path = ROOT / rel
        raw = path.read_bytes()
        name = path.stem
        entry = by_name[name]
        check(len(raw) == entry["mjb_bytes"], f"{name} bytes")
        check(hashlib.sha256(raw).hexdigest() == entry["mjb_sha256"], f"{name} sha")
        model = mujoco.MjModel.from_binary_path(str(path))
        data = mujoco.MjData(model)
        counts = {k: int(getattr(model, k)) for k in COUNT_NAMES}
        check(counts == entry["counts"], f"{name} counts {counts} != {entry['counts']}")

        # qpos cases: default qpos0 plus per-entry small offset, mj_kinematics each.
        qpos0 = np.array(model.qpos0, dtype=np.float64)
        qpos1 = qpos0 + np.array([0.01 * ((i % 3) - 1) for i in range(model.nq)])
        cases = entry["qpos_cases"]
        check(len(cases) == 2, f"{name} qpos cases")
        recases = []
        for qpos, case in zip((qpos0, qpos1), cases):
            check(bits64(qpos) == case["qpos_bits"], f"{name} qpos bits")
            data.qpos[:] = qpos
            mujoco.mj_kinematics(model, data)
            xs = bits32(np.asarray(data.geom_xpos).reshape(-1))
            xm = bits32(np.asarray(data.geom_xmat).reshape(-1))
            check(xs == case["geom_xpos_bits"], f"{name} geom_xpos bits")
            check(xm == case["geom_xmat_bits"], f"{name} geom_xmat bits")
            recases.append({"qpos_bits_ok": bits64(qpos) == case["qpos_bits"],
                            "n_xpos": len(xs), "n_xmat": len(xm)})

        field_report = {}
        for fid in range(2, 32):
            spec = FIELD_SPEC[fid]
            expected_count = counts[spec[2]] * spec[3]
            got = entry["fields"][str(fid)]
            check(got["kind"] == spec[1], f"{name} field {fid} kind")
            check(got["count"] == expected_count, f"{name} field {fid} count")
            if got.get("volume_oracle"):
                check(counts["noct"] > 0, f"{name} field {fid} volume only when noct>0")
                field_report[fid] = {"count": got["count"], "volume_only": True}
                continue
            values = np.asarray(getattr(model, ATTR[fid])).reshape(-1)
            check(values.size == expected_count, f"{name} field {fid} native size")
            if spec[1] == 1:
                check(got["i32"] == values.astype(np.int32).tolist(), f"{name} field {fid} i32")
            elif spec[1] == 4:
                check(got["f32_bits"] == bits32(values), f"{name} field {fid} f32")
            else:
                check(got["f64_bits"] == bits64(values), f"{name} field {fid} f64")
                check(got["f32_bits"] == bits32(values), f"{name} field {fid} f32")
            field_report[fid] = {"count": got["count"], "checked": True}
        report["models"][name] = {"counts": counts, "qpos_cases": recases,
                                  "fields": field_report}

    # ---- duplicated field tables: Rust spec, C++ switch, CollisionGpuInputs ----
    rust_src = (ROOT / "src/model/collision_inputs.rs").read_text(encoding="utf-8")
    rust_rows = re.findall(
        r'field\("([a-z0-9_]+)",\s*(Float32|Index|Num),\s*(\w+),\s*(\d+),\s*(Data|Model)\)',
        rust_src)
    check(len(rust_rows) == 32, f"rust field rows {len(rust_rows)}")
    kind_of = {"Index": 1, "Num": 2, "Float32": 4}
    unit_name = {"Ngeom": "ngeom", "Nmesh": "nmesh", "Nmeshvert": "nmeshvert",
                 "Nmeshgraph": "nmeshgraph", "Nmeshpoly": "nmeshpoly",
                 "Nmeshpolyvert": "nmeshpolyvert", "Nmeshpolymap": "nmeshpolymap",
                 "Nhfield": "nhfield", "Nhfielddata": "nhfielddata", "Noct": "noct"}
    for fid, (name, kind, unit, width, source) in enumerate(rust_rows):
        spec = FIELD_SPEC[fid]
        check((name, kind_of[kind], unit_name[unit], int(width)) == spec,
              f"rust row {fid} != header spec: {(name, kind, unit, width)}")
        check((source == "Data") == (fid < 2), f"rust row {fid} source")

    cpp_src = (ROOT / "native/model_probe_collision_inputs.cpp").read_text(encoding="utf-8")
    cpp_rows = re.findall(
        r"case (\d+): (?:source = m->([a-z0-9_]+); )?unit = m->([a-z0-9_]+);"
        r" width = (\d+); kind = (\d+);( data_sourced = true;)?", cpp_src)
    check(len(cpp_rows) == 32, f"cpp rows {len(cpp_rows)}")
    for cid, source, unit, width, kind, data_sourced in cpp_rows:
        fid = int(cid)
        spec = FIELD_SPEC[fid]
        check((spec[1], spec[2], spec[3]) == (int(kind), unit, int(width)),
              f"cpp case {fid} != header spec")
        if fid >= 2:
            check(source == spec[0], f"cpp case {fid} source attr")
        check(bool(data_sourced) == (fid < 2), f"cpp case {fid} data flag")

    types_src = (ROOT / "src/runtime/collision_gpu/types.rs").read_text(encoding="utf-8")
    block = re.search(r"pub struct CollisionGpuInputs<'a> \{(.*?)\n\}", types_src, re.S).group(1)
    gpu_fields = re.findall(r"pub (\w+): &'a \[", block)
    check(len(gpu_fields) == 32, f"CollisionGpuInputs fields {len(gpu_fields)}")
    check(gpu_fields == [FIELD_SPEC[i][0] for i in range(32)],
          f"CollisionGpuInputs order != header spec: {gpu_fields}")

    report["field_table"] = {
        "rust_rows": len(rust_rows),
        "cpp_cases": len(cpp_rows),
        "gpu_input_fields": len(gpu_fields),
        "gpu_order_matches_header": gpu_fields == [FIELD_SPEC[i][0] for i in range(32)],
    }
    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    print(f"oracle sha={report['oracle_sha256']} bytes={report['oracle_bytes']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
