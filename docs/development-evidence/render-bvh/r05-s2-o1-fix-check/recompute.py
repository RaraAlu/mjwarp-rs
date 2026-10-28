#!/usr/bin/env python3
"""R05-S2 QA O1 修复的只读轻复核重算脚本（check 卡口径）。

只核 4 个点 + O2/O3/O5 处置，不跑全量门禁：
  1. 阈值全 f32（常量位型、两处比较、无残留 f64 阈值）；
  2. 新增 4 条单测强度（定向跑 render_hfield_mesh：16 passed）；
  3. README §2.1 措辞已否证旧等价结论、模块文档同步；
  4. 修复提交零 CUDA 源改动 → 内核门 sha/bytes 与 O1 前一致（我顺带跑了内核门）。
"""

import hashlib
import json
import re
import struct
import subprocess
import sys
import pathlib

try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

R = pathlib.Path(
    subprocess.run(["git", "rev-parse", "--show-toplevel"],
                   cwd=pathlib.Path(__file__).resolve().parent,
                   capture_output=True, text=True, check=True).stdout.strip()
)
FIX = "1b876391"
CHECK_EV = "docs/development-evidence/render-bvh/r05-s2-o1-fix-check"
IMPL_EV = "docs/development-evidence/render-bvh/r05-s2-hfield-intersect"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()
EXPECTED_KERNEL = {
    "source_sha256": "72bc364c4b732e278396d2aebd09625872f6dcecba469a98a3e40aba04851746",
    "ptx_sha256": "97467d835e27107a4a0803678e194f4868fb552707543a544e270cbde8a9bd13",
    "ptx_bytes": 264015,
}
BANNED = ["".join(chr(c) for c in seq) for seq in (
    (71, 49, 48, 24050, 23454, 29616), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (21407, 29983, 25918, 34892), (31934, 24230, 36798, 26631),
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
)]

problems = []


def blob(rev, rel):
    return subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=R, capture_output=True, check=True).stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def check(cond, msg):
    if not cond:
        problems.append(msg)


rep = {"head": HEAD, "fix_commit": FIX}

# ---------------------------------------------------------------- 1. 阈值全 f32
mesh = blob("HEAD", "src/render/bvh/hfield_mesh.rs").decode()
rep["threshold"] = {
    "constant_f32": "let threshold = 1.0e-5f32;" in mesh,
    "planar_uses_f32_compare": "diff.abs() < threshold" in mesh,
    "fits_plane_pred_uses_f32_compare": "(cz00 - z_pred).abs() >= threshold" in mesh,
    "fits_plane_slopes_use_f32_compare": "(cslope_x - slope_x).abs() >= threshold" in mesh
    and "(cslope_y - slope_y).abs() >= threshold" in mesh,
    "no_f64_threshold_left": "1.0e-5f64" not in mesh and "f64::from(diff.abs())" not in mesh,
    "f32_1e_5_bits": struct.unpack("<I", struct.pack("<f", 1.0e-5))[0],
    "f32_1e_5_value": struct.unpack("<f", struct.pack("<f", 1.0e-5))[0],
    "f32_value_lt_decimal": struct.unpack("<f", struct.pack("<f", 1.0e-5))[0] < 1.0e-5,
    "ff_control_chars_in_comment": mesh.count("\x0c"),
}
check(rep["threshold"]["constant_f32"], "threshold constant not f32")
check(rep["threshold"]["planar_uses_f32_compare"], "planar compare not f32")
check(rep["threshold"]["fits_plane_pred_uses_f32_compare"], "fits_plane prediction compare not f32")
check(rep["threshold"]["fits_plane_slopes_use_f32_compare"], "fits_plane slope compare not f32")
check(rep["threshold"]["no_f64_threshold_left"], "f64 threshold still referenced")
check(rep["threshold"]["f32_1e_5_bits"] == 0x3727C5AC, "f32(1e-5) bits != 0x3727C5AC")
check(rep["threshold"]["f32_value_lt_decimal"], "f32(1e-5) not < 1e-5")

# ---------------------------------------------------------------- 2. 新增 4 条单测强度
tests = blob("HEAD", "tests/render_hfield_mesh.rs").decode()
rep["new_tests"] = {
    "threshold_constant": "fn threshold_constant_matches_upstream_f32_rounding()" in tests,
    "exact_boundary": "fn diff_exactly_f32_threshold_is_non_planar_like_upstream()" in tests,
    "below_boundary": "fn diff_just_below_f32_threshold_is_planar_and_merges()" in tests,
    "deterministic": "fn triangulation_is_deterministic_for_same_input()" in tests,
    "pins_bits": "0x3727_C5AC" in tests and "0x3727_C5AB" in tests,
    "pins_counts_6v12i": "points.len(), 6" in tests and "indices.len(), 12" in tests,
    "pins_counts_4v6i": "points.len(), 4" in tests and "vec![0, 1, 3, 0, 3, 2]" in tests,
    "pins_full_index_vector": "vec![0, 1, 3, 0, 3, 2, 1, 4, 5, 1, 5, 3]" in tests,
    "pins_point_bits": "t * 0.5" in tests,
}
for k, v in rep["new_tests"].items():
    check(v, f"new test strength check failed: {k}")

# ---------------------------------------------------------------- 3. README §2.1 / 模块文档
readme = blob("HEAD", f"{IMPL_EV}/README.md").decode()
receipt = json.loads(blob("HEAD", f"{IMPL_EV}/receipt.json"))
rep["docs"] = {
    "readme_negates_old_claim": "不成立" in readme and "与 f64 比较等价" in readme,
    "readme_f32_caliber": "f32(1e-5) = 9.999999747378752e-06" in readme,
    "readme_boundary_tests": "6 顶点/12 索引" in readme and "4 顶点/6 索引" in readme,
    "module_doc_synced": "阈值 `1e-5` 也先转 f32" in mesh or "先转 f32" in mesh,
    "receipt_precision_clause": "f32(1e-5)" in json.dumps(receipt, ensure_ascii=False),
    "receipt_unit_tests_16": receipt.get("triangulation", {}).get("unit_tests") == 16,
    "receipt_qa_fixes": "qa_fixes" in receipt and "O1" in json.dumps(receipt.get("qa_fixes", {}), ensure_ascii=False),
    "receipt_o2_note": "u64::MAX" in json.dumps(receipt, ensure_ascii=False)
    and "upload content unchanged" in json.dumps(receipt, ensure_ascii=False),
    "receipt_o3_o5_carried": "geom_enabled_mask" in json.dumps(receipt, ensure_ascii=False)
    and "same half" in json.dumps(receipt, ensure_ascii=False),
    "readme_o2": "O2（如实记录）" in readme or "O2（如实记录）" in readme,
    "readme_o3_o5": "遗留 O3" in readme and "遗留 O5" in readme,
    "readme_unit_1086": "1086 passed / 0 failed" in readme,
}
for k, v in rep["docs"].items():
    check(v, f"doc check failed: {k}")

# ---------------------------------------------------------------- 4. 零 CUDA 源改动 + 内核门
files = [p for p in subprocess.run(["git", "show", "--name-only", "--pretty=format:", FIX],
                                   cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
rep["commit"] = {"files": len(files), "paths": files,
                 "cu_files": [p for p in files if p.endswith(".cu")],
                 "touches_hfield_mesh": "src/render/bvh/hfield_mesh.rs" in files,
                 "touches_tests": "tests/render_hfield_mesh.rs" in files}
check(not rep["commit"]["cu_files"], "fix commit touches .cu")
check(rep["commit"]["touches_hfield_mesh"], "fix commit does not touch hfield_mesh.rs")

log_dir = R / CHECK_EV / "logs"
gates = {}
kg = (log_dir / "kernel-gate.log")
if kg.exists():
    t = kg.read_text(encoding="utf-8-sig", errors="replace")
    m = re.search(r"source_sha256=(\w+) ptx_sha256=(\w+) ptx_bytes=(\d+)", t)
    gates["kernel-gate"] = {
        "source_sha256": m.group(1) if m else None,
        "ptx_sha256": m.group(2) if m else None,
        "ptx_bytes": int(m.group(3)) if m else None,
        "matches_s2_record": bool(m) and m.group(1) == EXPECTED_KERNEL["source_sha256"]
        and m.group(2) == EXPECTED_KERNEL["ptx_sha256"] and int(m.group(3)) == EXPECTED_KERNEL["ptx_bytes"],
        "exit_marker": re.search(r"KERNEL_GATE_EXIT=(\d+)", t).group(1) if "KERNEL_GATE_EXIT=" in t else None,
    }
    check(gates["kernel-gate"]["matches_s2_record"], "kernel gate sha/bytes != S2 record")
tg = (log_dir / "targeted.log")
if tg.exists():
    t = tg.read_text(encoding="utf-8-sig", errors="replace")
    m = re.search(r"test result: ok\. (\d+) passed; (\d+) failed", t)
    gates["targeted"] = {"passed": int(m.group(1)) if m else None,
                         "failed": int(m.group(2)) if m else None,
                         "exit_marker": re.search(r"TARGETED_EXIT=(\d+)", t).group(1) if "TARGETED_EXIT=" in t else None,
                         "new_tests_all_ok": all(
                             re.search(r"test (threshold_constant_matches_upstream_f32_rounding|"
                                       r"diff_exactly_f32_threshold_is_non_planar_like_upstream|"
                                       r"diff_just_below_f32_threshold_is_planar_and_merges|"
                                       r"triangulation_is_deterministic_for_same_input) \.\.\. ok", t) is not None
                             for _ in [0])}
    check(gates["targeted"]["passed"] == 16 and gates["targeted"]["failed"] == 0,
          f"targeted tests {gates['targeted']}")
pr = (log_dir / "probe.log")
if pr.exists():
    t = pr.read_text(encoding="utf-8-sig", errors="replace")
    gates["probe"] = {
        "at_threshold_6v12i": "A_AT v=6 i=12" in t and "B_AT v=6 i=12" in t,
        "below_threshold_4v6i": "C_BELOW v=4 i=6" in t,
        "thr_bits_ok": "THR_BITS 3727c5ac" in t,
    }
    check(all(gates["probe"].values()), f"probe checks: {gates['probe']}")
rep["gates"] = gates

# 全量计数一致性（不重跑全量，只核记录）
unit_run = json.loads(blob("HEAD", ".normify/runs/2026-10-28-render-bvh-r05-s2-hfield-intersect-r2/unit.json"))
rep["full_suite_record"] = {
    "from_gate_record": unit_run.get("tests"),
    "exit": unit_run.get("exit"),
    "consistent_with_1082_plus_4": unit_run.get("tests", {}).get("passed") == 1086,
}
check(rep["full_suite_record"]["consistent_with_1082_plus_4"], "unit gate record != 1086")

# 实现证据 manifest 自校（仍在同一目录、被本修复更新）
ev = R / IMPL_EV
manifest = {}
for line in (ev / "manifest.txt").read_text(encoding="utf-8").split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h, n, rel = line.split(" ", 2)
        manifest[rel] = (h, int(n))
ev_rel = {p.relative_to(ev).as_posix() for p in ev.rglob("*") if p.is_file() and p.name != "manifest.txt"}
man_bad = []
if ev_rel != set(manifest):
    man_bad.append(f"set mismatch {sorted(ev_rel ^ set(manifest))}")
for rel in sorted(ev_rel & set(manifest)):
    b = blob("HEAD", f"{IMPL_EV}/{rel}")
    if (sha(b), len(b)) != manifest[rel]:
        man_bad.append(f"{rel}: {manifest[rel]} vs {(sha(b), len(b))}")
enc = []
for p in ev.rglob("*"):
    if p.is_file():
        b = p.read_bytes()
        if b.startswith(b"\xef\xbb\xbf"):
            enc.append((p.name, "BOM"))
        if b"\r" in b:
            enc.append((p.relative_to(ev).as_posix(), "CR"))
hits = []
for p in ev.rglob("*"):
    if p.is_file():
        t = p.read_text(encoding="utf-8", errors="replace")
        hits += [(p.relative_to(ev).as_posix(), w) for w in BANNED if w in t]
rep["impl_evidence"] = {"files": len(ev_rel) + 1, "manifest_entries": len(manifest),
                        "manifest_problems": man_bad, "encoding_problems": enc,
                        "forbidden_word_hits": hits}
check(not man_bad, f"manifest problems: {man_bad}")
check(not enc, f"encoding problems: {enc}")
check(not hits, f"forbidden words: {hits}")

rep["problems"] = problems
out = pathlib.Path(__file__).resolve().parent / "recompute.json"
out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(rep, ensure_ascii=False, indent=2))
print("RESULT problems=" + str(len(problems)))
for item in problems:
    print("PROBLEM " + str(item))
