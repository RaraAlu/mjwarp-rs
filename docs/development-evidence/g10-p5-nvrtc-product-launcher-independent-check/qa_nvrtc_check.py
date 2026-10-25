"""G10@P5 NVRTC 产品 launcher 独立复核（纯 CPU 静态/结构核对 + 数值对照）。

只读交付源码/证据；数值对照需要 QA 自己的 dumper 落盘（`dumper/`，公开 API）：
当 `target/qa-nvrtc/prod-out/<case>/` 存在时，用本脚本自己的 Python 比较器逐字节
对照 G5 探针 `gpu-window/cases/<case>/out1/`，不依赖交付测试的 bit_equal 结论。

用法：python qa_nvrtc_check.py [--allow-missing-numeric]
"""

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
DELIVERY = REPO / "docs" / "development-evidence" / "g10-p5-nvrtc-product-launcher"
CASES = REPO / "docs" / "development-evidence" / "g10-p5-gpu-kernels" / "gpu-window" / "cases"
PROD_OUT = REPO / "target" / "qa-nvrtc" / "prod-out"
CUDA = REPO / "src" / "physics" / "collision" / "cuda"
SRC = REPO / "src" / "runtime" / "collision_gpu.rs"
SOURCE_RS = REPO / "src" / "runtime" / "collision_gpu" / "source.rs"
TESTS_RS = REPO / "src" / "runtime" / "collision_gpu" / "tests.rs"
ABI_RS = REPO / "src" / "runtime" / "collision_gpu" / "abi.rs"
ITEST = REPO / "tests" / "g10_p5_nvrtc_launcher.rs"
PRODUCT_CU = CUDA / "product.cu"

CLAIM = {
    "device_source_len": 121566,
    "device_source_sha256": "dc862ec884680349c6f38b40de0379082ec1b6dbf47d4dba9f22adfd8adcad43",
    "ptx_bytes": 1863043,
    "entries": 3,
    "adapter": {"hfield-sphere": ["Hfield", 8, 0, "314/314"],
                "sdf-box": ["Sdf", 80, 0, "3122/3122"],
                "ellipsoid-box": ["Contact", 2, 2, "80/80"]},
}

FIELDS_F32 = ["dist", "includemargin", "friction", "solref", "solreffriction", "solimp",
              "adhesion"]
FIELDS_I32 = ["dim", "geom", "efc_address", "worldid", "geomcollisionid", "type"]
MULT = {"dist": 1, "includemargin": 1, "friction": 5, "solref": 2, "solreffriction": 2,
        "solimp": 5, "adhesion": 1, "dim": 1, "geom": 2, "efc_address": None,
        "worldid": 1, "geomcollisionid": 1, "type": 1}


def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))


def read_bytes(p):
    return Path(p).read_bytes()


def sha256(p):
    return hashlib.sha256(read_bytes(p)).hexdigest()


def fn_defs(text):
    """粗提取顶层函数定义名（含 __device__/__global__/static 前缀；以 `) {` 结尾）。"""
    names = []
    for m in re.finditer(
            r"^(?:(?:__global__|__device__|__host__|static|inline|constexpr|template\s*<[^>]*>|\s)*)"
            r"[A-Za-z_][\w\s\*\&:<>,]*?\b([A-Za-z_]\w*)\s*\([^;{}]*\)\s*\{",
            text, re.M):
        names.append(m.group(1))
    return names


def strip_local_includes(text):
    out = []
    for line in text.splitlines(keepends=True):
        s = line.lstrip()
        if s.startswith("#include"):
            rest = s[len("#include"):].lstrip()
            if rest.startswith('"'):
                body = rest[1:]
                end = body.find('"')
                if end >= 0 and body[:end].endswith(".cu"):
                    continue
        out.append(line)
    return "".join(out)


def parse_assembly():
    src = SOURCE_RS.read_text(encoding="utf-8")
    order = re.findall(r'\("([\w.]+\.cu)",\s*[A-Z_0-9]+\)', src)
    files = {p.name: p.read_text(encoding="utf-8") for p in CUDA.glob("*.cu")}
    assembled = ""
    for name in order:
        assembled += strip_local_includes(files[name])
        if not assembled.endswith("\n"):
            assembled += "\n"
    assembled += files["product.cu"]
    raw = assembled.encode("utf-8")
    deps = {name: re.findall(r'#include\s+"([^"]+\.cu)"', files[name]) for name in order}
    pos = {n: i for i, n in enumerate(order)}
    violations = [{"file": n, "dep": d} for n, ds in deps.items() for d in ds
                  if d in pos and pos[d] > pos[n]]
    return {
        "order": order,
        "order_count": len(order),
        "cu_files": sorted(files),
        "device_cu_files": sorted(n for n in files if n != "product.cu"),
        "local_include_graph": deps,
        "topological_order_ok": violations == [],
        "topological_violations": violations,
        "assembled_len": len(raw),
        "assembled_sha256": hashlib.sha256(raw).hexdigest(),
        "matches_claim_len": len(raw) == CLAIM["device_source_len"],
        "matches_claim_sha256": hashlib.sha256(raw).hexdigest() == CLAIM["device_source_sha256"],
    }


def product_cu_boundary():
    prod = PRODUCT_CU.read_text(encoding="utf-8")
    defs = fn_defs(prod)
    kernels = re.findall(r"__global__\s+void\s+(\w+)", prod)
    cu_defs = {}
    for p in CUDA.glob("*.cu"):
        if p.name == "product.cu":
            continue
        cu_defs[p.name] = fn_defs(p.read_text(encoding="utf-8"))
    all_cu = {n for v in cu_defs.values() for n in v}
    called = sorted(set(re.findall(r"\b([a-z_]\w*)\s*\(", prod)))
    numeric_called = sorted(set(n for n in called if n in all_cu))
    glue_expected = {"init_polytope", "build_geom", "build_params", "sdf_type_of"}
    structs = re.findall(r"^struct\s+(\w+)\s*\{", prod, re.M)
    includes = re.findall(r'#include\s+([<"][^>"]+[>"])', prod)
    return {
        "product_top_level_defs": sorted(set(defs)),
        "product_kernels": kernels,
        "product_structs": structs,
        "product_includes": includes,
        "glue_defs_present": sorted(glue_expected & set(defs)),
        "glue_missing": sorted(glue_expected - set(defs)),
        "numeric_functions_called_and_defined_in_cuda": numeric_called,
        "product_defs_that_are_also_numeric_in_cuda": sorted(set(defs) & all_cu),
        "no_numeric_kernel_defined_in_product": sorted(set(defs) & all_cu) == [],
    }


def abi_layout():
    text = ABI_RS.read_text(encoding="utf-8")

    def parse(name):
        m = re.search(r"struct\s+" + name + r"\s*\{(.*?)\n\}", text, re.S)
        body = m.group(1)
        fields = []
        for line in body.splitlines():
            line = line.strip()
            if not line or line.startswith("//"):
                continue
            mm = re.match(r"pub\(crate\)\s+(\S+)\s*:\s*([^,]+),", line)
            if mm:
                fields.append((mm.group(1), mm.group(2).strip()))
        return fields

    def size_of(ty):
        if "*const" in ty or "*mut" in ty or ty.endswith("*"):
            return 8, 8
        if ty == "CollisionContactOut":
            return 136, 8
        m = re.match(r"\[(\w+);\s*(\d+)\]", ty)
        if m:
            inner = {"f32": 4, "i32": 4, "u32": 4}.get(m.group(1), 4)
            return inner * int(m.group(2)), inner
        return {"i32": (4, 4), "f32": (4, 4), "u32": (4, 4)}.get(ty, (4, 4))

    def layout(name):
        fields = parse(name)
        offset, align = 0, 1
        rows = []
        for fname, ty in fields:
            sz, al = size_of(ty)
            if offset % al:
                offset += al - offset % al
            rows.append({"field": fname, "type": ty, "offset": offset})
            offset += sz
            align = max(align, al)
        if offset % align:
            offset += align - offset % align
        return {"fields": rows, "size": offset, "align": align}

    args = layout("CollisionArgs")
    out = layout("CollisionContactOut")
    return {
        "collision_args": {"size": args["size"], "align": args["align"],
                           "field_count": len(args["fields"])},
        "collision_contact_out": {"size": out["size"], "align": out["align"],
                                  "field_count": len(out["fields"])},
        "matches_tests_rs_495_496": args["size"] == 496,
        "matches_tests_rs_136": out["size"] == 136,
        "args_field_names": [f["field"] for f in args["fields"]],
        "contact_out_field_names": [f["field"] for f in out["fields"]],
    }


def entry_authenticity():
    src = SRC.read_text(encoding="utf-8")
    runtime_mod = (REPO / "src" / "runtime" / "mod.rs").read_text(encoding="utf-8")
    physics_mod = (REPO / "src" / "physics" / "mod.rs").read_text(encoding="utf-8")
    cargo = (REPO / "Cargo.toml").read_text(encoding="utf-8")
    itest = ITEST.read_text(encoding="utf-8")
    return {
        "module_gated_by_feature": "#[cfg(feature = \"cuda-probe\")]" in runtime_mod
                                    and "pub mod collision_gpu;" in runtime_mod,
        "feature_declared": re.search(r"^cuda-probe\s*=", cargo, re.M) is not None,
        "feature_not_in_default": re.search(r"^default\s*=\s*\[([^\]]*)\]", cargo, re.M) is not None
                                   and "cuda-probe" not in
                                   re.search(r"^default\s*=\s*\[([^\]]*)\]", cargo, re.M).group(1),
        "new_uses_compile_program": "compile_program()?" in src,
        "compile_program_uses_cudarc_nvrtc": "compile_ptx_with_opts" in src
                                              and "cudarc::nvrtc" in src,
        "load_module_and_three_functions": "load_module(ptx)" in src
                                            and "load_function(entry)" in src,
        "run_launches_kernel": "launch_builder" in src or ".launch(" in src,
        "no_probe_binary_or_probe_cu_reference": "probe.exe" not in src
                                                 and "probe.cu" not in src,
        "integration_test_is_ignored": "#[ignore" in itest,
        "integration_test_uses_product_api": "CollisionGpuProgram::new" in itest
                                              and "program.run(" in itest,
        "integration_test_compares_to_out1": 'join("out1")' in itest,
    }


def compile_options_check():
    src = SRC.read_text(encoding="utf-8")
    block = src[src.find("pub(crate) fn compile_options"):]
    return {
        "fmad_true": "fmad: Some(true)" in block,
        "use_fast_math_false": "use_fast_math: Some(false)" in block,
        "std_cxx17": "--std=c++17" in block,
        "arch_sm89": "--gpu-architecture=sm_89" in block,
        "name_set": "mjwarp_collision_product.cu" in block,
        "include_paths_empty": "include_paths: vec![]" in block
                               or "include_paths: Vec::new()" in block,
    }


def error_path_review():
    tests = TESTS_RS.read_text(encoding="utf-8")
    names = re.findall(r"fn\s+(\w+)\(\)\s*\{", tests.split("#[cfg(test)]")[-1])
    test_names = re.findall(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+(\w+)", tests)
    return {
        "test_count": len(test_names),
        "test_names": test_names,
        "covers_compile_err_injection": "Err(\"nvrtc: boom\"" in tests,
        "covers_compile_panic_catch_unwind": "panic!(" in tests and "NVRTC" in tests,
        "covers_no_device": "Err(\"no cuda driver\"" in tests,
        "covers_compile_error_priority": "compile_first" in tests,
        "covers_invalid_meta": "InvalidArgument" in tests and "rejects_invalid_meta" in tests,
        "covers_mismatched_slices": "rejects_mismatched_slice_lengths" in tests,
        "covers_abi_layout_assertions": "size_of::<CollisionArgs>()" in tests
                                        and "offset_of!" in tests,
        "covers_error_display_all_variants": "error_display_covers_every_variant" in tests,
        "covers_source_assembly": "assembles_all_device_sources_in_dependency_order" in tests,
        "covers_deterministic_sha": "assembly_is_deterministic_and_hashed" in tests,
        "load_variant_display_covered": "CollisionGpuError::Load" in tests,
        "load_failure_not_injectable": "program_from_parts" in tests
                                        and "load_program(" not in tests,
        "uncovered": ["CollisionGpuError::Load（load_module/load_function 失败）无 CPU 可注入 seam，未单测"],
    }


def window_check():
    wl = (DELIVERY / "window-log.txt").read_text(encoding="utf-8")
    samples = [[int(x) for x in ln.split(",")]
               for ln in (DELIVERY / "window-memory-samples.txt").read_text(encoding="utf-8").splitlines()
               if ln.strip()]
    def val(key):
        m = re.findall(key + r"=([^\s]+)", wl)
        return m[-1] if m else None
    head = val("window_start_head")
    used0 = int(val("baseline_used_mib"))
    peak = int(val("peak_memory_used_mib"))
    after = int(val("after_used_mib"))
    git_head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"],
                              capture_output=True).stdout.decode().strip()
    parent = subprocess.run(["git", "-C", str(REPO), "rev-parse", "dfcc1bdc^"],
                            capture_output=True).stdout.decode().strip()
    changed = subprocess.run(["git", "-C", str(REPO), "diff-tree", "--no-commit-id",
                              "--name-only", "-r", "dfcc1bdc"],
                             capture_output=True).stdout.decode().splitlines()
    product_changed = [c for c in changed if c.startswith(("src/", "tests/", "Cargo.toml", "Cargo.lock"))]
    return {
        "window_head": head,
        "window_head_is_fix_parent": head.startswith(parent[:8]) or parent.startswith(head),
        "head_at_run": git_head,
        "baseline_used_mib": used0,
        "peak_used_mib": peak,
        "peak_delta_mib": peak - used0,
        "after_used_mib": after,
        "samples": len(samples),
        "samples_peak_matches": max(s[0] for s in samples) == peak,
        "last_sample_is_peak": samples[-1][0] == peak,
        "after_memory_measured_separately": after,
        "after_equals_baseline": after == used0,
        "guard_policy": val("guard_policy"),
        "guard_triggered": "guard triggered" in wl or "guard_trigger" in wl,
        "task_related_baseline": val("baseline_compute_apps_task_related"),
        "task_related_after": val("after_compute_apps_task_related"),
        "worktree_changes_count": val("window_worktree_changes_count"),
        "committed_file_count": len([c for c in changed if c.strip()]),
        "committed_product_file_count": len(product_changed),
        "entry_count_matches_commit": int(val("window_worktree_changes_count") or -1)
                                      == len(product_changed),
        "elapsed_seconds": val("elapsed_seconds"),
        "nvcc": val("nvcc"),
        "path_prepend": val("path_prepend"),
        "test_exe_sha_recorded": "exe" in wl and "sha256=" in wl,
        "artifacts_entries": len([ln for ln in
                                  (DELIVERY / "artifacts-sha256.txt").read_text(encoding="utf-8").splitlines()
                                  if ln.strip()]),
    }


def artifacts_check():
    rows = []
    for ln in (DELIVERY / "artifacts-sha256.txt").read_text(encoding="utf-8").splitlines():
        if not ln.strip():
            continue
        rel, want = ln.split("\t")
        f = REPO / rel
        rows.append({"path": rel, "exists": f.exists(),
                     "sha256_match": f.exists() and sha256(f) == want,
                     "sha256": want})
    return {"entries": len(rows), "all_match": all(r["sha256_match"] for r in rows),
            "rows": rows}


def line_counts():
    files = ["src/runtime/collision_gpu.rs", "src/runtime/collision_gpu/source.rs",
             "src/runtime/collision_gpu/error.rs", "src/runtime/collision_gpu/abi.rs",
             "src/runtime/collision_gpu/tests.rs", "src/runtime/collision_gpu/nvrtc_smoke.rs",
             "src/physics/collision/cuda/product.cu", "tests/g10_p5_nvrtc_launcher.rs"]
    out = {}
    for f in files:
        p = REPO / f
        if not p.exists():
            out[f] = None
            continue
        data = p.read_bytes()
        out[f] = data.count(b"\n")
    return {"files": out, "all_within_500": all(v is not None and v <= 500 for v in out.values())}


def numeric_compare():
    """QA 自 dump 的产品输出 vs G5 out1，逐字节；输出 bit_equal 计数（按交付字段口径）。"""
    cases = {}
    total_ok = True
    for case, (kind, nacon_claim, nccd_claim, bit_claim) in CLAIM["adapter"].items():
        out_dir = PROD_OUT / case
        ref_dir = CASES / case / "out1"
        entry = {"case": case, "kind": kind, "dump_exists": out_dir.is_dir()}
        # 结构核对（CPU 可做）：bit_equal 分母 = nworld(overflow) + nacon*(35+efc_width)
        ref_nacon_bytes = read_bytes(ref_dir / "nacon.bin")
        ref_nccd_bytes = read_bytes(ref_dir / "nccd.bin")
        meta = {}
        for line in (CASES / case / "meta.txt").read_text(encoding="utf-8").splitlines():
            parts = line.split()
            if len(parts) == 2:
                meta[parts[0]] = parts[1]
        nworld = int(meta.get("nworld", "0"))
        ref_nacon = int.from_bytes(ref_nacon_bytes, "little", signed=True)
        efc_width = int((ref_dir / "efc_width.txt").read_text(encoding="utf-8").strip())
        structure_total = nworld + ref_nacon * (35 + efc_width)
        entry.update({
            "structure_bit_total_recomputed": structure_total,
            "structure_matches_delivered_denominator": structure_total == int(bit_claim.split("/")[1]),
            "delivered_bit_equal": int(bit_claim.split("/")[0]),
            "delivered_bit_total": int(bit_claim.split("/")[1]),
        })
        if not out_dir.is_dir():
            entry["status"] = "pending_gpu_dump"
            cases[case] = entry
            total_ok = False
            continue
        efc_width = int((ref_dir / "efc_width.txt").read_text(encoding="utf-8").strip())
        ref_nacon = int.from_bytes(read_bytes(ref_dir / "nacon.bin"), "little", signed=True)
        ref_nccd = int.from_bytes(read_bytes(ref_dir / "nccd.bin"), "little", signed=True)
        dev_nacon = int.from_bytes(read_bytes(out_dir / "nacon.bin"), "little", signed=True)
        dev_nccd = int.from_bytes(read_bytes(out_dir / "nccd.bin"), "little", signed=True)
        entry.update({"nacon": [dev_nacon, ref_nacon], "nccd": [dev_nccd, ref_nccd]})
        bit_total = 0
        bit_equal = 0
        field_rows = {}
        # nacon / nccd 只做整值相等核对，不计入 bit_equal 计数（与交付口径一致：
        # 交付分母 = nworld(overflow) + nacon*(35+efc_width)）
        for name, d, r in (("nacon", dev_nacon, ref_nacon), ("nccd", dev_nccd, ref_nccd)):
            field_rows[name] = {"equal": d == r, "device": d, "reference": r}
        nacon = dev_nacon
        for name, mult in (("overflow", None), ("dist", 1), ("pos", 3), ("frame", 9),
                           ("includemargin", 1), ("friction", 5), ("solref", 2),
                           ("solreffriction", 2), ("solimp", 5), ("adhesion", 1),
                           ("dim", 1), ("geom", 2), ("efc_address", None),
                           ("worldid", 1), ("geomcollisionid", 1), ("type", 1)):
            dev = read_bytes(out_dir / f"{name}.bin")
            ref = read_bytes(ref_dir / f"{name}.bin")
            n = len(ref) // 4
            if name == "efc_address":
                n = nacon * efc_width
            elif name == "overflow":
                n = len(ref) // 4
            field_rows[name] = {"len_device": len(dev), "len_reference": len(ref),
                                "equal_bytes": dev == ref, "components": n}
            bit_total += n
            if dev == ref:
                bit_equal += n
            else:
                # 逐分量比较（f32/i32 都用原始 4 字节）
                a = [dev[i:i + 4] for i in range(0, len(dev), 4)]
                b = [ref[i:i + 4] for i in range(0, len(ref), 4)]
                bit_equal += sum(1 for x, y in zip(a, b) if x == y)
        delivered_bit = [int(x) for x in bit_claim.split("/")]
        entry.update({
            "status": "compared",
            "bit_equal": bit_equal,
            "bit_total": bit_total,
            "delivered_bit_equal": delivered_bit[0],
            "delivered_bit_total": delivered_bit[1],
            "bit_total_matches_structure": bit_total == delivered_bit[1],
            "bit_equal_matches_delivered": bit_equal == delivered_bit[0],
            "fields": field_rows,
        })
        cases[case] = entry
        total_ok = total_ok and entry["bit_equal_matches_delivered"] and entry["bit_total_matches_structure"]
    return {"cases": cases, "all_cases_compared": all(c["status"] == "compared" for c in cases.values()),
            "all_bit_equal": total_ok}


def own_window_check():
    """QA 自己两轮窗口日志（run1: hfield-sphere 0.5 s；run2: 其余 2 case 0.25 s）。"""
    runs = []
    for name in ("window-log.txt", "window-log2.txt"):
        p = EVIDENCE / name
        if not p.exists():
            continue
        text = p.read_text(encoding="utf-8")

        def val(key, default=None):
            m = re.findall(re.escape(key) + r"=([^\s]+)", text)
            return m[-1] if m else default

        case_results = re.findall(r"case_result=(.+bitwise identical[^\r\n]*)", text)
        runs.append({
            "file": name,
            "head": val("head"),
            "path_prepend": val("path_prepend"),
            "nvcc": val("nvcc"),
            "dumper_exe_sha256": val("dumper_exe_sha256"),
            "baseline_used_mib": val("baseline_used_mib"),
            "baseline_drift_mib": val("baseline_drift_mib"),
            "baseline_compute_apps_count": val("baseline_compute_apps_count"),
            "baseline_task_related": val("baseline_compute_apps_task_related"),
            "after_used_mib": val("after_used_mib"),
            "after_compute_apps_count": val("after_compute_apps_count"),
            "after_task_related": val("after_compute_apps_task_related"),
            "residual_qa_dumper_processes": val("residual_qa_dumper_processes"),
            "sample_count": val("sample_count"),
            "sample_peak_mib": val("sample_peak_mib"),
            "sample_peak_delta_mib": val("sample_peak_delta_mib"),
            "elapsed_seconds": val("elapsed_seconds"),
            "soft_threshold_exceeded": val("soft_threshold_exceeded"),
            "case_results": case_results,
            "window_status": "done" if "window=done" in text else "failed/stopped",
        })
    return runs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--allow-missing-numeric", action="store_true")
    ap.add_argument("--numeric-only", action="store_true",
                    help="只做数值对照（窗口内 per-case 早停用）")
    ap.add_argument("--case", default=None, help="--numeric-only 时只核一个 case")
    args = ap.parse_args()
    if args.numeric_only:
        num = numeric_compare()
        cases = num["cases"]
        selected = [args.case] if args.case else list(cases)
        out = {c: {k: v for k, v in cases.get(c, {}).items() if k != "fields"}
               for c in selected}
        print(json.dumps(out, ensure_ascii=False, indent=1))
        ok = all(cases.get(c, {}).get("status") == "compared"
                 and cases[c]["bit_equal_matches_delivered"]
                 and cases[c]["bit_total_matches_structure"] for c in selected)
        return 0 if ok else 2
    rep = {"schema": 1, "kind": "qa-nvrtc-launcher-check",
           "task_id": "2026-10-25-g10-p5-nvrtc-product-launcher-independent-check"}
    rep["entry_authenticity"] = entry_authenticity()
    rep["compile_options"] = compile_options_check()
    rep["assembly"] = parse_assembly()
    rep["product_cu_boundary"] = product_cu_boundary()
    rep["abi_layout"] = abi_layout()
    rep["error_paths"] = error_path_review()
    rep["window"] = window_check()
    rep["own_windows"] = own_window_check()
    rep["artifacts"] = artifacts_check()
    rep["line_counts"] = line_counts()
    rep["numeric"] = numeric_compare()
    v = {
        "entry_is_real_nvrtc": all([rep["entry_authenticity"][k] for k in
                                    ("module_gated_by_feature", "feature_declared",
                                     "new_uses_compile_program",
                                     "compile_program_uses_cudarc_nvrtc",
                                     "load_module_and_three_functions",
                                     "no_probe_binary_or_probe_cu_reference",
                                     "integration_test_is_ignored",
                                     "integration_test_uses_product_api",
                                     "integration_test_compares_to_out1")]),
        "compile_options_aligned": all(rep["compile_options"].values()),
        "assembly_fingerprint_matches": rep["assembly"]["matches_claim_len"]
                                        and rep["assembly"]["matches_claim_sha256"]
                                        and rep["assembly"]["topological_order_ok"]
                                        and rep["assembly"]["order_count"] == 14,
        "product_cu_is_glue_only": rep["product_cu_boundary"]["no_numeric_kernel_defined_in_product"]
                                   and rep["product_cu_boundary"]["glue_missing"] == [],
        "abi_layout_matches": rep["abi_layout"]["matches_tests_rs_495_496"]
                              and rep["abi_layout"]["matches_tests_rs_136"],
        "error_paths_substantive": all(rep["error_paths"][k] for k in
                                       ("covers_compile_err_injection",
                                        "covers_compile_panic_catch_unwind",
                                        "covers_no_device", "covers_compile_error_priority",
                                        "covers_invalid_meta", "covers_mismatched_slices",
                                        "covers_abi_layout_assertions",
                                        "covers_error_display_all_variants")),
        "window_self_consistent": rep["window"]["window_head_is_fix_parent"]
                                  and rep["window"]["samples_peak_matches"]
                                  and rep["window"]["last_sample_is_peak"]
                                  and rep["window"]["after_equals_baseline"]
                                  and not rep["window"]["guard_triggered"]
                                  and rep["window"]["task_related_baseline"] in ("<none>", "0")
                                  and rep["window"]["task_related_after"] in ("<none>", "0")
                                  and rep["window"]["entry_count_matches_commit"],
        "artifacts_sha_ok": rep["artifacts"]["all_match"],
        "line_limit_ok": rep["line_counts"]["all_within_500"],
        "numeric_verified": rep["numeric"]["all_cases_compared"] and rep["numeric"]["all_bit_equal"],
    }
    v["pass"] = all(x for k, x in v.items() if k != "numeric_verified")
    if not args.allow_missing_numeric:
        v["pass"] = v["pass"] and v["numeric_verified"]
    rep["verdict"] = v
    (EVIDENCE / "qa-nvrtc-check.json").write_text(
        json.dumps(rep, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(v, ensure_ascii=False, indent=1))
    print("assembly:", rep["assembly"]["assembled_len"], rep["assembly"]["assembled_sha256"])
    print("product.cu defs:", rep["product_cu_boundary"]["product_top_level_defs"])
    print("numeric called in product.cu:", rep["product_cu_boundary"]["numeric_functions_called_and_defined_in_cuda"][:10], "...")
    print("window:", json.dumps({k: rep["window"][k] for k in
                                 ("window_head", "baseline_used_mib", "peak_used_mib",
                                  "after_used_mib", "samples", "elapsed_seconds",
                                  "task_related_baseline", "task_related_after",
                                  "entry_count_matches_commit", "test_exe_sha_recorded")}, ensure_ascii=False))
    print("numeric:", json.dumps({c: {k: v2 for k, v2 in s.items() if k != "fields"}
                                  for c, s in rep["numeric"]["cases"].items()}, ensure_ascii=False))
    print("report:", EVIDENCE / "qa-nvrtc-check.json")
    return 0 if v["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
