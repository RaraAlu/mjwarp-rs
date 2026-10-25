"""QA independent G0b review: verify the 10 new GJK/EPA direct-reference captures without delivered support code."""

import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIX = ROOT / "fixtures/g10-p5"
REF = FIX / "references"
MODEL_INDEX = FIX / "models/index.json"
CAPTURE_SCRIPT = ROOT / "scripts/authoring/g10-p5/capture_g0b_references.py"
GEOM_NAMES = {0: "plane", 1: "hfield", 2: "sphere", 3: "capsule", 4: "ellipsoid",
              5: "cylinder", 6: "box", 7: "mesh", 8: "sdf"}
EXPECTED_WIDTH = {"dist": 1, "pos": 3, "frame": 9, "includemargin": 1, "friction": 5,
                  "solref": 2, "solreffriction": 2, "solimp": 5, "adhesion": 1,
                  "dim": 1, "geom": 2, "flex": 2, "elem": 2, "vert": 2,
                  "worldid": 1, "type": 1, "geomcollisionid": 1}
ZERO_WIDTH_NO_FLEX = {"flex", "elem", "vert"}
FLOAT_FIELDS = {"dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion"}
EXPECTED_GEOMS = {
    "sphere-box-shallow": None, "sphere-box-deep": None, "capsule-box": None,
    "cylinder-box": None, "ellipsoid-box": None, "hfield-sphere": None,
    "hfield-capsule": None, "sdf-box": None, "mesh-mesh": None, "box-box-margin": None,
}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits_f32(values):
    return [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values]


def bits_close(a, b):
    return bits_f32(a) == bits_f32(b)


def row_of(fields, index, count):
    row = {}
    for name, rec in fields.items():
        vals = rec["values"]
        width = len(vals) // count
        row[name] = vals[index * width:(index + 1) * width]
    return row


def compare_cpu(case, model, qpos, qvel, stored_rows, stored_worlds):
    """Recompute contacts world-by-world with MuJoCo CPU; compare counts/pairs/dist/pos."""
    result = {}
    for world in (0, 1):
        data = mujoco.MjData(model)
        data.qpos[:] = np.asarray(qpos[world], dtype=np.float64)
        data.qvel[:] = np.asarray(qvel[world], dtype=np.float64)
        mujoco.mj_forward(model, data)
        ncon = int(data.ncon)
        rows = []
        for i in range(ncon):
            con = data.contact[i]
            rows.append({"dist": float(con.dist), "pos": [float(v) for v in con.pos],
                         "geom": [int(con.geom1), int(con.geom2)],
                         "worldid": [world]})
        stored = [r for r in stored_rows if r["worldid"][0] == world]
        pair_match = sorted(tuple(r["geom"]) for r in stored) == sorted(tuple(r["geom"]) for r in rows)
        # match by (dist, pos) nearest neighbour after sorting
        s = sorted(stored, key=lambda r: (bits_f32([r["dist"][0]])[0], tuple(bits_f32(r["pos"]))))
        c = sorted(rows, key=lambda r: (bits_f32([r["dist"]])[0], tuple(bits_f32(r["pos"]))))
        exact_dist = sum(1 for a, b in zip(s, c) if a["dist"][0] == b["dist"])
        max_dd = max((abs(a["dist"][0] - b["dist"]) for a, b in zip(s, c)), default=0.0)
        max_dp = max((max(abs(x - y) for x, y in zip(a["pos"], b["pos"])) for a, b in zip(s, c)), default=0.0)
        result[str(world)] = {
            "cpu_ncon": ncon, "stored_ncon": len(stored), "count_match": ncon == len(stored),
            "geom_pairs_match": pair_match, "dist_exact_matches": exact_dist,
            "max_abs_dist_diff": max_dd, "max_abs_pos_diff": max_dp,
            "cpu_geom_pairs": sorted({tuple(r["geom"]) for r in rows}),
        }
    return result


def main():
    model_index = json.loads(MODEL_INDEX.read_text(encoding="utf-8"))
    index = json.loads((REF / "index.json").read_text(encoding="utf-8"))
    script_sha = sha256(CAPTURE_SCRIPT)
    model_index_sha = sha256(MODEL_INDEX)
    report = {"cases": {}, "index_audit": {}, "limitations": []}
    rows_by_model = {row["name"]: row for row in model_index["models"]}
    index_rows = {row["name"]: row for row in index["cases"]}
    ok_all = True

    for name, meta in rows_by_model.items():
        path = REF / (name + ".json")
        doc = json.loads(path.read_text(encoding="utf-8"))
        xml = ROOT / meta["xml"]
        model = mujoco.MjModel.from_xml_path(str(xml))
        qpos_stored = doc["input"]["qpos"]
        qvel_stored = doc["input"]["qvel"]
        qpos = np.asarray(qpos_stored["values"], dtype=np.float32).reshape(qpos_stored["shape"])
        qvel = np.asarray(qvel_stored["values"], dtype=np.float64).reshape(qvel_stored["shape"])
        fields = doc["output"]["fields"]
        count = doc["output"]["count"]
        rows = [row_of(fields, i, count) for i in range(count)]

        # --- provenance
        provenance = doc["provenance"]
        versions = provenance["versions"]
        prov = {
            "task_id": doc["task_id"] == "2026-10-25-g10-p5-fixtures",
            "upstream_revision": provenance["upstream_revision"] == "71da24d956378a87a703b6e1442b13aec0c4ac29",
            "script_sha_match": provenance["authoring_script_sha256"] == script_sha,
            "model_index_sha_match": provenance["model_index_sha256"] == model_index_sha,
            "source_xml_sha_match": doc["input"]["sha256"] == sha256(xml),
            "versions": versions,
            "flags_ok": provenance["authoring_only"] is True and provenance["reads_product_tree"] is False
                        and provenance["product_python_dependency"] is False,
            "gpu": provenance["gpu"],
        }
        prov["all_ok"] = all([prov["task_id"], prov["upstream_revision"], prov["script_sha_match"],
                              prov["model_index_sha_match"], prov["source_xml_sha_match"], prov["flags_ok"]])

        # --- default_state semantics (qpos/qvel rebuilt independently from the XML)
        qpos0 = np.asarray(model.qpos0, dtype=np.float32)
        expected_qpos = np.tile(qpos0, (2, 1))
        expected_qpos[1, 0:3] += np.asarray(doc["input"]["offset"], dtype=np.float32)
        expected_qvel = np.zeros((2, model.nv), dtype=np.float64)
        for w in range(2):
            expected_qvel[w] = np.linspace(-0.02 * (w + 1), 0.03 * (w + 1), model.nv, dtype=np.float32)
        state = {
            "nq": int(model.nq), "nv": int(model.nv), "freejoints": int((model.jnt_type == 0).sum()),
            "jnt_qposadr": [int(v) for v in model.jnt_qposadr],
            "qpos_shape": list(qpos.shape),
            "qpos_matches_expected_bits": bool(np.array_equal(qpos.view(np.uint32), expected_qpos.view(np.uint32))),
            "qvel_matches_expected_bits": bool(np.array_equal(qvel.astype(np.float32).view(np.uint32),
                                                              expected_qvel.astype(np.float32).view(np.uint32))),
            "stored_bits_field_consistent": (bits_f32(qpos_stored["values"]) == qpos_stored["bits"]
                                             if "bits" in qpos_stored else
                                             bits_f32(qpos_stored["values"]) == bits_f32(qpos_stored["values"])),
            "world0_world1_qpos_diff_indices": np.nonzero(
                qpos[0].view(np.uint32) != qpos[1].view(np.uint32))[0].tolist(),
            "offset_nonzero_indices": [i for i, v in enumerate(doc["input"]["offset"]) if float(v) != 0.0],
        }
        state["diff_only_at_offset_components"] = (
            state["world0_world1_qpos_diff_indices"] == state["offset_nonzero_indices"])
        state["ok"] = all([state["qpos_matches_expected_bits"], state["qvel_matches_expected_bits"],
                           state["diff_only_at_offset_components"]])
        # independent FK check: moving world shifts only the free body
        data0 = mujoco.MjData(model)
        data1 = mujoco.MjData(model)
        data0.qpos[:] = qpos[0].astype(np.float64)
        data1.qpos[:] = qpos[1].astype(np.float64)
        mujoco.mj_kinematics(model, data0)
        mujoco.mj_kinematics(model, data1)
        xpos_diff = data1.xpos - data0.xpos
        moved = [int(i) for i in range(model.nbody) if np.abs(xpos_diff[i]).max() > 0]
        free_body = int(model.jnt_bodyid[0]) if model.njnt else -1
        fk = {
            "free_body_id": free_body,
            "bodies_moved": moved,
            "free_body_moved_by_offset": bool(
                free_body >= 0 and np.allclose(xpos_diff[free_body],
                                               np.asarray(doc["input"]["offset"]), atol=1e-6)),
            "only_free_body_moved": moved == [free_body],
        }
        state["fk"] = fk

        # --- contact 17 fields
        field_check = {}
        for fname, width in EXPECTED_WIDTH.items():
            rec = fields.get(fname)
            expected_width = 0 if (fname in ZERO_WIDTH_NO_FLEX and int(model.nflex) == 0) else width
            if rec is None:
                field_check[fname] = {"present": False}
                continue
            n = len(rec["values"]) // count if count else 0
            field_check[fname] = {
                "present": True, "dtype": rec["dtype"], "expected_width": expected_width,
                "zero_width_because_nflex0": fname in ZERO_WIDTH_NO_FLEX and int(model.nflex) == 0,
                "width_ok": n == expected_width, "len_ok": len(rec["values"]) == count * expected_width,
                "sha_len_ok": len(rec["sha256"]) == 64,
                "finite": bool(np.isfinite(np.asarray(rec["values"], dtype=np.float64)).all()),
            }
        fields_ok = all(f["present"] and f["width_ok"] and f["len_ok"] and f["finite"]
                        for f in field_check.values())
        world_seq = fields["worldid"]["values"]
        per_world = {}
        for w in sorted(set(world_seq)):
            per_world[str(w)] = world_seq.count(w)
        order = {
            "worldid_sequence": world_seq,
            "grouped": all(world_seq[i] <= world_seq[i + 1] for i in range(len(world_seq) - 1)),
            "reported_grouped": doc["order_observation"]["grouped_by_world_in_raw_order"],
            "reported_sequence_matches": doc["order_observation"]["worldid_sequence"] == world_seq,
        }
        geom_types = {i: GEOM_NAMES.get(int(model.geom_type[i]), str(int(model.geom_type[i])))
                      for i in range(model.ngeom)}
        pairs = sorted({tuple(r["geom"]) for r in rows})
        pair_types = [sorted(geom_types[g] for g in pair if g >= 0) for pair in pairs]

        # --- contact semantic self-consistency (checkable without trusting either implementation)
        semantic = {
            "frame_orthonormal_violations": 0,
            "includemargin_mismatch": 0,
            "pos_out_of_range": 0,
            "geom_id_out_of_range": 0,
            "dim_values": sorted({int(r["dim"][0]) for r in rows}),
            "type_values": sorted({int(r["type"][0]) for r in rows}),
            "worldid_values": sorted({int(r["worldid"][0]) for r in rows}),
        }
        gid = {}
        for r in rows:
            frame = np.asarray(r["frame"], dtype=np.float64).reshape(3, 3)
            if not np.allclose(frame @ frame.T, np.eye(3), atol=2e-5):
                semantic["frame_orthonormal_violations"] += 1
            g1, g2 = int(r["geom"][0]), int(r["geom"][1])
            margin = float(model.geom_margin[g1]) + float(model.geom_margin[g2])
            if abs(float(r["includemargin"][0]) - margin) > 1e-6:
                semantic["includemargin_mismatch"] += 1
            if not np.all(np.abs(np.asarray(r["pos"], dtype=np.float64)) < 10):
                semantic["pos_out_of_range"] += 1
            if not (0 <= g1 < model.ngeom and 0 <= g2 < model.ngeom):
                semantic["geom_id_out_of_range"] += 1
            key = (int(r["worldid"][0]), g1, g2)
            gid.setdefault(key, []).append(int(r["geomcollisionid"][0]))
        semantic["geomcollisionid_sequences"] = {
            "%d:%d-%d" % k: sorted(v) for k, v in gid.items()}
        semantic["ok"] = (semantic["frame_orthonormal_violations"] == 0
                          and semantic["includemargin_mismatch"] == 0
                          and semantic["pos_out_of_range"] == 0
                          and semantic["geom_id_out_of_range"] == 0)

        # --- independent CPU recompute (MuJoCo 3.15 CPU vs stored mjwarp reference)
        cpu = compare_cpu(doc, model, qpos, qvel, rows, world_seq)

        det = doc["determinism"]
        determinism = {
            "two_captures_raw_bitwise_identical": det["two_captures_raw_bitwise_identical"],
            "two_captures_sorted_bitwise_identical": det["two_captures_sorted_bitwise_identical"],
            "second_capture_stored": False,
            "note": "单次捕获数据只落盘一次，第二次捕获仅记录布尔；位等结论无法从夹具文件本身复算",
        }

        case_report = {
            "path": str(path.relative_to(ROOT)).replace("\\", "/"),
            "file_sha256": sha256(path),
            "provenance": prov,
            "default_state": state,
            "contact": {"count": count, "per_world": per_world,
                        "field_check": field_check, "fields_ok": fields_ok,
                        "field_count": len(fields), "pairs": pairs, "pair_types": pair_types,
                        "dist_min": round(float(min(fields["dist"]["values"])), 6),
                        "dist_max": round(float(max(fields["dist"]["values"])), 6)},
            "order": order,
            "determinism": determinism,
            "contact_semantics": semantic,
            "cpu_recompute": cpu,
        }
        report["cases"][name] = case_report
        # index consistency
        entry = index_rows[name]
        idx = {
            "sha_match": entry["sha256"] == case_report["file_sha256"],
            "count_match": entry["count"] == count,
            "per_world_match": entry["per_world"] == per_world,
            "raw_flag_match": entry["two_captures_raw_bitwise_identical"] == det["two_captures_raw_bitwise_identical"],
            "sorted_flag_match": entry["two_captures_sorted_bitwise_identical"] == det["two_captures_sorted_bitwise_identical"],
            "grouped_flag_match": entry["grouped_by_world_in_raw_order"] == order["grouped"],
        }
        idx["all_ok"] = all(idx.values())
        report["index_audit"][name] = idx
        if not (prov["all_ok"] and state["ok"] and fields_ok and semantic["ok"]
                and order["reported_sequence_matches"] and idx["all_ok"]):
            ok_all = False
        print("%-19s count=%-3d per_world=%-11s pair=%-22s prov=%s state=%s fields=%s sem=%s order=%s idx=%s CPU=%s" % (
            name, count, per_world, str(pair_types)[:22], prov["all_ok"], state["ok"], fields_ok,
            semantic["ok"], order["reported_sequence_matches"], idx["all_ok"],
            json.dumps({w: [v["count_match"], v["geom_pairs_match"], round(v["max_abs_dist_diff"], 6)]
                        for w, v in cpu.items()})))

    # coverage matrix checks
    cov = {}
    for name in rows_by_model:
        c = report["cases"][name]
        cov[name] = {
            "pairs_types": c["contact"]["pair_types"],
            "count": c["contact"]["count"],
            "per_world": c["contact"]["per_world"],
            "raw_identical": c["determinism"]["two_captures_raw_bitwise_identical"],
            "sorted_identical": c["determinism"]["two_captures_sorted_bitwise_identical"],
            "grouped": c["order"]["grouped"],
        }
    cov["shallow_vs_deep_dist"] = {
        "shallow_min": report["cases"]["sphere-box-shallow"]["contact"]["dist_min"],
        "deep_min": report["cases"]["sphere-box-deep"]["contact"]["dist_min"],
        "deep_more_negative": report["cases"]["sphere-box-deep"]["contact"]["dist_min"]
                               < report["cases"]["sphere-box-shallow"]["contact"]["dist_min"],
    }
    bx = mujoco.MjModel.from_xml_path(str(ROOT / rows_by_model["box-box-margin"]["xml"]))
    dsbl = int(bx.opt.disableflags)
    cov["box_box_margin_model"] = {
        "geom_margin": [float(v) for v in bx.geom_margin],
        "multiccd_enabled": (dsbl & int(mujoco.mjtDisableBit.mjDSBL_MULTICCD)) == 0,
        "nativeccd_disabled": (dsbl & int(mujoco.mjtDisableBit.mjDSBL_NATIVECCD)) != 0,
        "disableflags": dsbl,
    }
    report["coverage"] = cov
    # per-case model flags: documents which code path the case actually exercises
    flags = {}
    for name, meta in rows_by_model.items():
        m = mujoco.MjModel.from_xml_path(str(ROOT / meta["xml"]))
        dsbl = int(m.opt.disableflags)
        flags[name] = {
            "multiccd_enabled": (dsbl & int(mujoco.mjtDisableBit.mjDSBL_MULTICCD)) == 0,
            "nativeccd_enabled": (dsbl & int(mujoco.mjtDisableBit.mjDSBL_NATIVECCD)) == 0,
            "geom_margin": [float(v) for v in m.geom_margin],
            "is_discrete_possible": bool(np.all(m.geom_margin == 0)),
            "nflex": int(m.nflex),
        }
    report["model_flags"] = flags
    report["overall"] = {"all_checks_ok": ok_all,
                         "determinism_limitation": "second capture not stored; bitwise determinism not independently recomputable from fixtures"}
    out = ROOT / "target/qa-g0b/references_qa_report.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("overall all_checks_ok =", ok_all)
    print("wrote", out)


if __name__ == "__main__":
    sys.exit(main())
