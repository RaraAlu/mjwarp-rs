"""QA independent G0b review: bitwise compare QA's own 3.12 vs 3.15 dumps and audit delivered parity claims."""

import hashlib
import json
import struct
from pathlib import Path

import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g0b"
FIX = ROOT / "fixtures/g10-p5"
MODELS = ["hfield-box", "sdf-cylinder"]
FIELDS = {
    "counts": ("i64", 6), "nq_nv": ("i64", 2),
    "hfield_nrow": ("i32", None), "hfield_ncol": ("i32", None), "hfield_adr": ("i32", None),
    "hfield_size": ("f64", None), "hfield_data": ("f32", None),
    "oct_depth": ("i32", None), "oct_child": ("i32", None),
    "oct_aabb": ("f64", None), "oct_coeff": ("f64", None),
    "mesh_octadr": ("i32", None), "mesh_octnum": ("i32", None),
    "opt_sdf": ("i32", 2), "geom_type": ("i32", None), "geom_dataid": ("i32", None),
    "geom_size": ("f64", None), "geom_margin": ("f64", None),
}
DT = {"i32": "<i4", "i64": "<i8", "f32": "<f4", "f64": "<f8"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_field(base, name):
    path = base / (name + ".bin")
    if not path.is_file():
        return None
    return path.read_bytes()


def first_diff(a, b):
    n = min(len(a), len(b))
    for i in range(n):
        if a[i] != b[i]:
            return i
    return n if len(a) != len(b) else None


def cmp_field(base_a, base_b, name, dtype):
    a = read_field(base_a, name)
    b = read_field(base_b, name)
    if a is None or b is None:
        return {"name": name, "status": "missing", "a_present": a is not None, "b_present": b is not None}
    out = {
        "name": name, "dtype": dtype, "bytes_a": len(a), "bytes_b": len(b),
        "count_a": len(a) // np.dtype(DT[dtype]).itemsize,
        "count_b": len(b) // np.dtype(DT[dtype]).itemsize,
        "sha256_a": sha(a), "equal": a == b,
    }
    if a != b:
        fd = first_diff(a, b)
        out["first_diff_byte"] = fd
        out["shape_equal"] = len(a) == len(b)
        if len(a) == len(b):
            va = np.frombuffer(a, dtype=DT[dtype])
            vb = np.frombuffer(b, dtype=DT[dtype])
            diff = va != vb
            out["diff_elements"] = int(diff.sum())
            out["diff_indices"] = np.nonzero(diff)[0][:20].tolist()
            fa, fb = va.astype(np.float64), vb.astype(np.float64)
            out["max_abs_diff"] = float(np.max(np.abs(fa - fb)))
    return out


def octree_structure(base):
    depth = np.frombuffer(read_field(base, "oct_depth"), dtype="<i4")
    child = np.frombuffer(read_field(base, "oct_child"), dtype="<i4")
    aabb = np.frombuffer(read_field(base, "oct_aabb"), dtype="<f8")
    coeff = np.frombuffer(read_field(base, "oct_coeff"), dtype="<f8")
    noct = len(depth)
    if noct == 0:
        return {"noct": 0, "note": "no octree (volume SDF absent)"}
    checks = {
        "noct": int(noct),
        "child_len_is_noct_x8": int(len(child)) == 8 * noct,
        "aabb_len_is_noct_x6": int(len(aabb)) == 6 * noct,
        "coeff_len_is_noct_x8": int(len(coeff)) == 8 * noct,
        "depth_values": sorted(set(int(v) for v in depth)),
        "depth_histogram": {int(d): int(c) for d, c in zip(*np.unique(depth, return_counts=True))},
        "root_depth_is_0": bool(depth[0] == 0),
        "root_count_at_depth0": int((depth == 0).sum()),
        "child_min": int(child.min()), "child_max": int(child.max()),
        "child_valid_or_negative": bool(((child >= -1) & (child < noct)).all()),
        "negative_children": int((child < 0).sum()),
        "referenced_nodes": int(len(np.unique(child[child >= 0]))),
        "unreferenced_nonroot_nodes": int(noct - 1 - len(np.unique(child[child >= 0]))),
        "depth_monotone_parent_child": None,
        "aabb_finite": bool(np.isfinite(aabb).all()),
        "coeff_finite": bool(np.isfinite(coeff).all()),
    }
    parent_child_ok = True
    bad = 0
    for i in range(noct):
        for j in range(8):
            c = int(child[8 * i + j])
            if c >= 0:
                if int(depth[c]) != int(depth[i]) + 1:
                    parent_child_ok = False
                    bad += 1
    checks["depth_monotone_parent_child"] = parent_child_ok
    checks["parent_child_depth_violations"] = bad
    # node AABB sanity: half-extents > 0, children inside parent
    half = aabb.reshape(noct, 2, 3)  # (center3, half3) as stored order center/half
    center, hext = aabb[0::6], None
    ctr = np.stack([aabb[0::6], aabb[1::6], aabb[2::6]], axis=1)
    hx = np.stack([aabb[3::6], aabb[4::6], aabb[5::6]], axis=1)
    checks["half_extents_positive"] = bool((hx > 0).all())
    checks["aabb_center_equals_root_center_first"] = bool(np.allclose(ctr[0], np.zeros(3), atol=1e-9))
    inside = True
    for i in range(min(noct, 5000)):
        for j in range(8):
            c = int(child[8 * i + j])
            if c >= 0:
                if not np.all(ctr[c] - hx[c] >= ctr[i] - hx[i] - 1e-9) or \
                   not np.all(ctr[c] + hx[c] <= ctr[i] + hx[i] + 1e-9):
                    inside = False
    checks["children_inside_parent_aabb"] = inside
    return checks


def main():
    report = {"models": {}, "parity_json_audit": {}, "overall": {}}
    delivered = json.loads((FIX / "octree-parity.json").read_text(encoding="utf-8"))
    all_equal = True
    for model in MODELS:
        b312 = QA / "dump-312" / model / "frommjb"
        b315 = QA / "dump-315" / model / "frommjb"
        rows = {}
        for name, (dtype, _n) in FIELDS.items():
            rows[name] = cmp_field(b312, b315, name, dtype)
        # cross-check: QA XML-compiled path vs delivered artifact path on the same side
        x312 = QA / "dump-312" / model / "fromxml"
        x315 = QA / "dump-315" / model / "fromxml"
        side_consistency = {
            "312_fromxml_equals_frommjb": all(
                read_field(x312, name) == read_field(b312, name) for name in FIELDS),
            "315_fromxml_equals_frommjb": all(
                read_field(x315, name) == read_field(b315, name) for name in FIELDS),
        }
        # delivery parity audit: every delivered field entry must match my recomputation
        dmodel = delivered["models"][model]
        entries = dmodel["f64_fields"] + dmodel["f32_fields"] + dmodel["struct_fields"]
        audit = []
        for entry in entries:
            name = entry["name"]
            if name not in rows:
                audit.append({"name": name, "matched": False, "reason": "not in QA field set"})
                continue
            mine = rows[name]
            matched = (
                mine.get("equal") == entry.get("equal")
                and mine.get("bytes_a") == entry.get("bytes")
                and mine.get("sha256_a") == entry.get("sha256_312")
                and read_field(b315, name) is not None
                and sha(read_field(b315, name)) == entry.get("sha256_315")
            )
            audit.append({"name": name, "matched": bool(matched),
                          "delivered_equal": entry.get("equal"), "qa_equal": mine.get("equal"),
                          "delivered_shas_ok": bool(
                              read_field(b315, name) is not None
                              and sha(read_field(b315, name)) == entry.get("sha256_315"))})
        parity_audit = {
            "all_delivered_fields_match_qa": all(a["matched"] for a in audit),
            "entries": audit,
        }
        structure = octree_structure(b312)
        hdata = np.frombuffer(read_field(b312, "hfield_data"), dtype="<f4") if read_field(b312, "hfield_data") else np.array([])
        hsize = np.frombuffer(read_field(b312, "hfield_size"), dtype="<f8") if read_field(b312, "hfield_size") else np.array([])
        equal_fields = [name for name, row in rows.items() if row.get("equal")]
        unequal_fields = [name for name, row in rows.items() if not row.get("equal")]
        report["models"][model] = {
            "counts_312": np.frombuffer(read_field(b312, "counts"), dtype="<i8").tolist(),
            "counts_315": np.frombuffer(read_field(b315, "counts"), dtype="<i8").tolist(),
            "field_rows": rows,
            "equal_fields": equal_fields,
            "unequal_fields": unequal_fields,
            "all_fields_equal": len(unequal_fields) == 0,
            "side_consistency": side_consistency,
            "delivered_parity_audit": parity_audit,
            "octree_structure": structure,
            "hfield_data_values": hdata.tolist(),
            "hfield_size_values": hsize.tolist(),
            "delivered_parity_claim": {
                "octree_f64_equal": dmodel.get("octree_f64_equal"),
                "hfield_equal": dmodel.get("hfield_equal"),
                "struct_equal": dmodel.get("struct_equal"),
                "noct_312": dmodel["counts_312"].get("noct"),
                "noct_315": dmodel["counts_315"].get("noct"),
            },
        }
        all_equal = all_equal and len(unequal_fields) == 0 and parity_audit["all_delivered_fields_match_qa"]
        print("%-13s fields_equal=%s unequal=%s delivered_audit=%s side=%s" % (
            model, len(unequal_fields) == 0, unequal_fields, parity_audit["all_delivered_fields_match_qa"],
            side_consistency))
        if model == "sdf-cylinder":
            print("  octree:", json.dumps({k: structure[k] for k in [
                "noct", "depth_histogram", "child_valid_or_negative", "referenced_nodes",
                "unreferenced_nonroot_nodes", "depth_monotone_parent_child",
                "half_extents_positive", "children_inside_parent_aabb"]}))
            print("  hfield_data:", hdata.tolist(), "hfield_size:", hsize.tolist())
        else:
            print("  hfield_data:", hdata.tolist(), "hfield_size:", hsize.tolist())

    report["overall"] = {
        "all_equal": bool(all_equal),
        "delivered_overall_claim": delivered.get("overall"),
    }
    out = QA / "parity_qa_report.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("overall all_equal =", all_equal, "| delivered claim =", delivered["overall"].get("all_equal"))
    print("wrote", out)


if __name__ == "__main__":
    main()
