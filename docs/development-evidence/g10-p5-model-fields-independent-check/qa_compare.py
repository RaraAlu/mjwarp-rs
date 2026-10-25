"""QA G1 independent review: compare QA's ABI export, QA's own 3.12 MJB dump and the G0b parity fixture.

Inputs (all produced by QA):
  target/qa-g1/ref-312/<model>/*.bin          QA's MuJoCo 3.12 C dumper (G0b tool)
  target/qa-g1/abi-pre/<model>/*.bin          QA probe compiled against pre-G1 source (fec926f5)
  target/qa-g1/abi-post/<model>/*.bin         QA probe compiled against reviewed G1 source (ef47d450)
  fixtures/g10-p5/octree-parity.json          G0b parity fixture (expected sha256_312)
"""

import hashlib
import json
import struct
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g1"
FIX = ROOT / "fixtures/g10-p5"
MODELS = ["hfield-box", "sdf-cylinder"]
FIXTURE_FIELDS = {
    "oct_aabb": ("f64", 6), "oct_coeff": ("f64", 8), "hfield_size": ("f64", 4),
    "geom_size": ("f64", 3), "geom_margin": ("f64", 1),
    "hfield_data": ("f32", 1),
    "counts": ("i64", 1), "nq_nv": ("i64", 1), "oct_depth": ("i32", 1),
    "oct_child": ("i32", 8), "mesh_octadr": ("i32", 1), "mesh_octnum": ("i32", 1),
    "hfield_nrow": ("i32", 1), "hfield_ncol": ("i32", 1), "hfield_adr": ("i32", 1),
    "opt_sdf": ("i32", 1), "geom_type": ("i32", 1), "geom_dataid": ("i32", 1),
}
REF_FILES = {name: name + ".bin" for name in FIXTURE_FIELDS}
VOLUME_IDS = {
    45: "hfield_nrow", 46: "hfield_ncol", 47: "hfield_size", 48: "hfield_adr",
    49: "hfield_data", 51: "oct_depth", 52: "oct_child", 53: "oct_aabb",
    54: "oct_coeff", 55: "mesh_octadr", 56: "mesh_octnum",
}
WIDTH = {"f64": 8, "f32": 4, "i32": 4, "i64": 8}
DTYPE = {"f64": "<f8", "f32": "<f4", "i32": "<i4", "i64": "<i8"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_manifest(path):
    fields = {}
    for line in path.read_text(encoding="utf-8").splitlines()[1:]:
        parts = line.split(",")
        if len(parts) != 5:
            continue
        fields[int(parts[0])] = {
            "kind": int(parts[1]), "count": int(parts[2]), "bytes": int(parts[3]),
            "present": parts[4] == "1",
        }
    return fields


def main():
    fixture = json.loads((FIX / "octree-parity.json").read_text(encoding="utf-8"))
    report = {"schema": 1, "kind": "qa-g1-parity-and-abi", "models": {}, "overall": {}}
    all_ok = True
    for model in MODELS:
        parity = fixture["models"][model]
        entries = parity["f64_fields"] + parity["f32_fields"] + parity["struct_fields"]
        expected = {entry["name"]: entry for entry in entries}

        ref_dir = QA / "ref-312" / model
        pre_dir = QA / "abi-pre" / model
        post_dir = QA / "abi-post" / model
        ref = {}
        for name in FIXTURE_FIELDS:
            path = ref_dir / REF_FILES[name]
            ref[name] = path.read_bytes() if path.is_file() else None

        # 1) QA reference dump vs G0b fixture sha256_312 (all fixture-listed fields)
        ref_checks = {}
        for name, entry in expected.items():
            data = ref.get(name)
            if data is None:
                ref_checks[name] = {"ok": False, "reason": "missing QA ref dump"}
                continue
            ref_checks[name] = {
                "bytes": len(data), "expected_bytes": entry.get("bytes"),
                "sha256": sha(data), "expected_sha256_312": entry.get("sha256_312"),
                "ok": sha(data) == entry.get("sha256_312"),
            }
            if not ref_checks[name]["ok"]:
                all_ok = False

        # 2) ABI post export vs fixture sha (12 volume fields) and vs QA reference dump
        pre_manifest = read_manifest(pre_dir / "manifest.csv")
        post_manifest = read_manifest(post_dir / "manifest.csv")
        abi_checks = {}
        for field_id, name in VOLUME_IDS.items():
            data = (post_dir / (str(field_id) + ".bin")).read_bytes()
            entry = expected[name]
            same_ref = ref[name] is not None and data == ref[name]
            sha_ok = sha(data) == entry.get("sha256_312")
            meta = post_manifest[field_id]
            count_ok = meta["bytes"] == len(data) == entry.get("bytes")
            abi_checks[name] = {
                "id": field_id, "kind": meta["kind"], "abi_bytes": len(data),
                "fixture_bytes": entry.get("bytes"), "sha256": sha(data),
                "fixture_sha256_312": entry.get("sha256_312"),
                "sha_ok": sha_ok, "bytes_equal_ref": same_ref,
                "count_bytes_ok": count_ok,
            }
            if not (sha_ok and same_ref and count_ok):
                all_ok = False
        # opt_sdf = ids 57 + 58 concatenated
        opt = (post_dir / "57.bin").read_bytes() + (post_dir / "58.bin").read_bytes()
        opt_entry = expected["opt_sdf"]
        opt_ok = (sha(opt) == opt_entry.get("sha256_312") and opt == ref["opt_sdf"])
        abi_checks["opt_sdf"] = {
            "ids": [57, 58], "abi_bytes": len(opt), "sha256": sha(opt),
            "fixture_sha256_312": opt_entry.get("sha256_312"),
            "sha_ok": sha(opt) == opt_entry.get("sha256_312"),
            "bytes_equal_ref": opt == ref["opt_sdf"],
        }
        if not opt_ok:
            all_ok = False
        # noct scalar (id 50)
        noct_bytes = (post_dir / "50.bin").read_bytes()
        noct_value = struct.unpack("<q", noct_bytes)[0]
        noct_expected = parity["counts_312"]["noct"]
        noct_ok = noct_value == noct_expected
        if not noct_ok:
            all_ok = False

        # 3) non-regression: ids 0..44 pre vs post (bytes, kind, count)
        ids = sorted(set(range(0, 45)))
        regress = {"ids_checked": len(ids), "byte_mismatches": [], "kind_mismatches": [],
                   "count_mismatches": [], "missing_post": [], "missing_pre": []}
        for field_id in ids:
            pre_meta = pre_manifest.get(field_id)
            post_meta = post_manifest.get(field_id)
            if not pre_meta:
                regress["missing_pre"].append(field_id)
                continue
            if not post_meta:
                regress["missing_post"].append(field_id)
                continue
            if pre_meta["kind"] != post_meta["kind"]:
                regress["kind_mismatches"].append(field_id)
            if pre_meta["count"] != post_meta["count"] or pre_meta["bytes"] != post_meta["bytes"]:
                regress["count_mismatches"].append(field_id)
            pre_bytes = (pre_dir / (str(field_id) + ".bin")).read_bytes()
            post_bytes = (post_dir / (str(field_id) + ".bin")).read_bytes()
            if pre_bytes != post_bytes:
                regress["byte_mismatches"].append(field_id)
        regress["ok"] = not (regress["byte_mismatches"] or regress["kind_mismatches"]
                             or regress["count_mismatches"] or regress["missing_post"]
                             or regress["missing_pre"])
        if not regress["ok"]:
            all_ok = False

        # 4) ABI struct constants pre vs post
        def abi_lines(directory):
            return (directory / "abi.txt").read_text(encoding="utf-8").strip().splitlines()
        abi_pre = {line.split("=")[0]: int(line.split("=")[1]) for line in abi_lines(pre_dir)
                   if "=" in line}
        abi_post = {line.split("=")[0]: int(line.split("=")[1]) for line in abi_lines(post_dir)
                    if "=" in line}
        abi_struct = {
            "pre": abi_pre, "post": abi_post,
            "struct_unchanged": all(abi_pre.get(key) == abi_post.get(key) for key in
                                    ("field_info_size", "field_info_align",
                                     "field_info_off_count", "native_info_size",
                                     "native_info_align")),
            "expected_sizes": (abi_post.get("field_info_size") == 16
                               and abi_post.get("field_info_align") == 8
                               and abi_post.get("field_info_off_count") == 8
                               and abi_post.get("native_info_size") == 96
                               and abi_post.get("native_info_align") == 8),
        }
        if not (abi_struct["struct_unchanged"] and abi_struct["expected_sizes"]):
            all_ok = False

        # 5) kind set check: pre only 1/2/3; post adds 4/5 only for new ids
        pre_kinds = sorted({meta["kind"] for meta in pre_manifest.values() if meta["present"]})
        post_kinds = sorted({meta["kind"] for meta in post_manifest.values() if meta["present"]})
        new_kinds = sorted({post_manifest[fid]["kind"] for fid in range(45, 59)})
        kind_check = {
            "pre_kinds": pre_kinds, "post_kinds": post_kinds,
            "new_field_kinds": new_kinds,
            "pre_only_1_2_3": pre_kinds == [1, 2, 3],
            "post_kinds_are_1_2_3_4_5": post_kinds == [1, 2, 3, 4, 5],
            "no_extra_kind": new_kinds == [1, 2, 4, 5],
        }
        if not all(kind_check[key] for key in ("pre_only_1_2_3", "post_kinds_are_1_2_3_4_5",
                                               "no_extra_kind")):
            all_ok = False

        # anchors
        def as_f32(data):
            return [struct.unpack("<f", data[i:i + 4])[0] for i in range(0, len(data), 4)]
        def as_f64(data):
            return [struct.unpack("<d", data[i:i + 8])[0] for i in range(0, len(data), 8)]
        anchors = {
            "hfield_nrow": [int(v) for v in struct.unpack("<i", ref["hfield_nrow"])] if ref["hfield_nrow"] else [],
            "hfield_ncol": [int(v) for v in struct.unpack("<i", ref["hfield_ncol"])] if ref["hfield_ncol"] else [],
            "hfield_adr": [int(v) for v in struct.unpack("<i", ref["hfield_adr"])] if ref["hfield_adr"] else [],
            "hfield_size": as_f64(ref["hfield_size"]) if ref["hfield_size"] else [],
            "hfield_data": as_f32(ref["hfield_data"]) if ref["hfield_data"] else [],
            "opt_sdf": [int(v) for v in struct.unpack("<2i", ref["opt_sdf"])],
            "mesh_octadr": [int(v) for v in struct.unpack("<i", ref["mesh_octadr"])] if ref["mesh_octadr"] else [],
            "mesh_octnum": [int(v) for v in struct.unpack("<i", ref["mesh_octnum"])] if ref["mesh_octnum"] else [],
        }
        report["models"][model] = {
            "noct": noct_value, "noct_expected": noct_expected, "noct_ok": noct_ok,
            "ref_dump_vs_fixture": ref_checks,
            "abi_export_vs_fixture_and_ref": abi_checks,
            "non_regression_ids_0_44": regress,
            "abi_struct": abi_struct,
            "kind_check": kind_check,
            "anchors": anchors,
        }
        print("%-13s noct=%s(%s) ref_vs_fixture=%s abi_vs_fixture=%s regress=%s struct=%s kinds=%s" % (
            model, noct_value, noct_ok,
            all(c["ok"] for c in ref_checks.values()),
            all(c.get("sha_ok") and c.get("bytes_equal_ref", True) for c in abi_checks.values()),
            regress["ok"], abi_struct["struct_unchanged"] and abi_struct["expected_sizes"],
            kind_check["no_extra_kind"]))

    report["overall"] = {"all_checks_ok": all_ok}
    (QA / "parity_report.json").write_text(json.dumps(report, ensure_ascii=False, indent=1),
                                           encoding="utf-8")
    print("overall:", all_ok)
    print("wrote", QA / "parity_report.json")


if __name__ == "__main__":
    main()
