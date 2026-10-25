"""QA G1 independent review: evidence/scope/conversion/world-contract consistency checks."""

import json
import re
import struct
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g1"
EV = ROOT / "docs/development-evidence/g10-p5-model-fields"
REV = "ef47d450"
PRE = "ef47d450^"


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT)] + list(args), capture_output=True)


def blob(path, commit=REV):
    res = git("show", "%s:%s" % (commit, path))
    return res.stdout if res.returncode == 0 else None


def sha(data):
    import hashlib
    return hashlib.sha256(data).hexdigest()


def main():
    report = {"checks": [], "failed": []}

    def check(name, ok, detail=None):
        report["checks"].append({"check": name, "ok": bool(ok), "detail": detail})
        if not ok:
            report["failed"].append({"check": name, "detail": detail})

    # 1) evidence blobs stay identical to the reviewed commit (and HEAD)
    for name in ["README.md", "receipt.json", "parity-run.log"]:
        rel = "docs/development-evidence/g10-p5-model-fields/" + name
        head_blob = blob(rel, "HEAD")
        rev_blob = blob(rel, REV)
        check("evidence_unchanged:" + name, head_blob == rev_blob and rev_blob is not None,
              {"rev": sha(rev_blob or b""), "head": sha(head_blob or b"")})
    receipt = json.loads(blob("docs/development-evidence/g10-p5-model-fields/receipt.json", REV)
                         .decode(encoding="utf-8"))
    readme = blob("docs/development-evidence/g10-p5-model-fields/README.md", REV).decode(encoding="utf-8")
    parity_log = blob("docs/development-evidence/g10-p5-model-fields/parity-run.log",
                      REV).decode(encoding="utf-8")

    # 2) scope: only declared paths changed, no Cargo/fixtures/S2-S8
    diff = git("diff", "--name-only", PRE + ".." + REV).stdout.decode().splitlines()
    allowed = ("native/", "include/", "src/io/", "src/model/",
               "tests/g10_p5_model_fields.rs", "docs/development-evidence/g10-p5-model-fields/",
               ".normify/")
    outside = [p for p in diff if p and not p.startswith(allowed)]
    check("scope_declared_paths_only", outside == [], outside)
    check("no_cargo_changes", not any(p.startswith("Cargo") or p == "rust-toolchain.toml" for p in diff))
    check("no_fixtures_changes", not any(p.startswith("fixtures/") for p in diff))
    check("no_s2_s8_or_d04_evidence",
          not any(p.startswith("docs/development-evidence/render-bvh/")
                  or p.startswith("docs/evidence/") for p in diff))
    check("g01_reader_untouched", "src/io/g01.rs" not in diff)
    check("build_rs_untouched", "build.rs" not in diff)
    check("test_file_in_scope", "tests/g10_p5_model_fields.rs" in diff)

    # 3) ABI header: comments-only diff
    header_diff = git("diff", "-U0", PRE + ".." + REV, "--", "include/mjwarp_native_probe.h")
    header_lines = header_diff.stdout.decode(encoding="utf-8").splitlines()
    changed = [line for line in header_lines
               if (line.startswith("+") or line.startswith("-"))
               and not line.startswith(("+++", "---"))]
    check("abi_header_comments_only",
          all(line[1:].lstrip().startswith("//") for line in changed) and len(changed) > 0,
          changed)

    # 4) existing native table entries / kinds unchanged; new kinds only 4/5
    inc_diff = git("diff", "-U0", PRE + ".." + REV, "--", "native/g01_fields.inc").stdout.decode()
    removed = [line for line in inc_diff.splitlines()
               if line.startswith("-") and not line.startswith("---")]
    # Only the two scalar-width expressions were refactored into g01_scalar_bytes; their
    # kinds 1/2/3 semantics are unchanged (and the pre/post export bytes prove it).
    refactor_ok = len(removed) == 2 and all("kind == 2" in line and "kind == 3" in line
                                            for line in removed)
    check("native_table_only_width_refactor_removed", refactor_ok, removed)
    check("native_table_adds_ids_45_58",
          all(("case %d:" % i) in inc_diff for i in range(45, 59)), None)
    check("native_table_kind_branches",
          "case 4: return sizeof(float)" in inc_diff and "case 5: return sizeof(int64_t)" in inc_diff)

    # 5) conversion point: single `as f32`, no double quantization in volume path
    model_volume = blob("src/model/volume.rs", REV).decode(encoding="utf-8")
    volume_input = blob("src/model/volume_input.rs", REV).decode(encoding="utf-8")
    io_volume = blob("src/io/volume.rs", REV).decode(encoding="utf-8")
    as_f32_model = len(re.findall(r"as f32", model_volume))
    as_f32_input = len(re.findall(r"as f32", volume_input))
    as_f32_io = len(re.findall(r"as f32", io_volume))
    check("single_conversion_site", as_f32_model == 1 and as_f32_input == 0 and as_f32_io == 0,
          {"model_volume": as_f32_model, "volume_input": as_f32_input, "io_volume": as_f32_io})
    check("num_to_f32_definition", "pub fn num_to_f32" in model_volume
          and "value as f32" in model_volume)
    check("no_double_quantization_roundtrip",
          "as f64" not in model_volume and "as f64" not in volume_input and "as f64" not in io_volume,
          {"model_volume_as_f64": model_volume.count("as f64"),
           "volume_input_as_f64": volume_input.count("as f64"),
           "io_volume_as_f64": io_volume.count("as f64")})
    check("conversion_entrypoints", all(name in volume_input for name in
          ("oct_aabb_f32", "oct_coeff_f32", "hfield_size_f32")))
    check("not_converted_asserted", "hfield_data(原生即f32)" in receipt["conversion"]["not_converted"][0]
          or "hfield_data" in receipt["conversion"]["not_converted"][0])

    # 6) bit-level conversion examples recomputed from QA's own 3.12 dump
    ref = QA / "ref-312" / "hfield-box"
    size_raw = (ref / "hfield_size.bin").read_bytes()
    size_f64 = list(struct.unpack("<4d", size_raw))
    size_bits = ["0x%016X" % struct.unpack("<Q", size_raw[i:i + 8])[0] for i in range(0, 32, 8)]
    f32_examples = {}
    for value in size_f64:
        raw = struct.pack("<f", value)
        f32_examples[repr(value)] = "0x%08X" % struct.unpack("<I", raw)[0]
    check("conversion_example_0_1", f32_examples.get("0.1") == "0x3DCCCCCD",
          {"examples": f32_examples, "f64_bits": size_bits})
    check("conversion_example_1_0", f32_examples.get("1.0") == "0x3F800000", f32_examples)
    data_raw = (ref / "hfield_data.bin").read_bytes()
    data_bits = ["0x%08X" % struct.unpack("<I", data_raw[i:i + 4])[0] for i in range(0, len(data_raw), 4)]
    check("hfield_data_native_f32_no_conversion", data_bits[1] == "0x3F000000", data_bits)

    # 7) world-dim contract: spec-only, no positive multi-world claim
    world = receipt["world_dim_contract"]
    check("world_contract_spec_only", world["status"] == "spec_only_not_implemented",
          world["status"])
    check("world_contract_policy",
          ("共享" in world["policy"] or "广播" in world["policy"])
          and "naconmax" in world["naconmax_reserve"])
    positive = []
    for match in re.finditer(r"[^。\n]*(多世界|nworld)[^。\n]*", readme):
        line = match.group(0)
        if "验证" in line and "未" not in line and "不" not in line:
            positive.append(line)
    check("no_positive_multiworld_claim", positive == [], positive)
    check("unmet_lists_gpu_world_linux",
          all(key in json.dumps(receipt["unmet"], ensure_ascii=False)
              for key in ("GPU", "nworld>1", "Linux")))

    # 8) evidence counts vs measured
    unit_log = (QA / "unit-full.log").read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m.group(1)) for m in re.finditer(r"test result: ok\. (\d+) passed", unit_log))
    failed = sum(int(m.group(1)) for m in re.finditer(r"; (\d+) failed", unit_log))
    ignored = sum(int(m.group(1)) for m in re.finditer(r"; (\d+) ignored;", unit_log))
    check("unit_count_matches_receipt", (passed, failed, ignored) == (565, 0, 1),
          {"measured": [passed, failed, ignored], "receipt": receipt["verification"]["unit"]})
    check("readme_unit_count", "565" in readme)
    parity_test = (QA / "parity-test.log").read_text(encoding="utf-8", errors="replace")
    check("delivered_parity_test_passed",
          "reads_volume_fields_bitwise_equal_to_g0b_parity ... ok" in parity_test
          and "1 passed; 0 failed" in parity_test)
    check("parity_log_matches_command",
          "--features native-model-probe" in parity_log and "1 passed" in parity_log)
    # field counts: 14 new fields, 45 existing ids
    check("new_field_count_14", "45..58" in readme and "14 个字段" in readme)
    g01 = blob("src/io/g01.rs", REV).decode(encoding="utf-8")
    check("existing_field_count_45", "FIELDS: [(&str, u32); 45]" in g01)
    # line counts of new files
    lines = {}
    for path in ["src/model/volume.rs", "src/model/volume_input.rs", "src/io/volume.rs",
                 "tests/g10_p5_model_fields.rs"]:
        lines[path] = len(blob(path, REV).decode(encoding="utf-8").splitlines())
    check("new_files_under_500", all(count <= 500 for count in lines.values()), lines)
    parity_report = json.loads((QA / "parity_report.json").read_text(encoding="utf-8"))
    check("measured_parity_report_ok", parity_report["overall"]["all_checks_ok"],
          parity_report["overall"])
    check("measured_each_model_ok",
          all(m["ref_dump_vs_fixture"] and m["abi_export_vs_fixture_and_ref"]
              and m["non_regression_ids_0_44"]["ok"] and m["abi_struct"]["struct_unchanged"]
              and m["kind_check"]["no_extra_kind"]
              for m in parity_report["models"].values()), None)

    # 9) forbidden claims absent from G1 evidence
    banned = [
        "".join(chr(int(piece, 16)) for piece in group.split())
        for group in (
            "0047 0031 0030 5df2 5b9e 73b0",
            "5b8c 6574 7269 7406 901a 8fc7",
            "53cc 5e73 53f0 901a 8fc7",
            "4e0a 6e38 5b8c 5168 7b49 4ef7",
            "539f 751f 653e 884c",
            "7cbe 5ea6 8fbe 6807",
        )
    ]
    evidence_text = readme + "\n" + json.dumps(receipt, ensure_ascii=False) + "\n" + parity_log
    hits = [term for term in banned if term in evidence_text]
    check("no_forbidden_claims", hits == [], hits)

    # 10) anchors from QA dump
    report["anchors"] = {
        "hfield_size": size_f64,
        "hfield_size_f64_bits": size_bits,
        "hfield_data_bits": data_bits,
        "conversion_f32": f32_examples,
        "unit": {"passed": passed, "failed": failed, "ignored": ignored},
        "line_counts": lines,
    }
    report["overall"] = {"all_checks_ok": not report["failed"]}
    (QA / "evidence_check_report.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("checks=%d failed=%d" % (len(report["checks"]), len(report["failed"])))
    for item in report["failed"]:
        print("  FAIL", item["check"], item["detail"])
    print("anchors:", json.dumps({k: report["anchors"][k] for k in
                                  ("hfield_size", "hfield_data_bits", "conversion_f32", "unit")},
                                 ensure_ascii=False))


if __name__ == "__main__":
    main()
