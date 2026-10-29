#!/usr/bin/env python3
"""QA 独立重算：R08-RGB 逐相机通道掩码离线片复核（只读）。

复核对象：`2026-10-29-render-r08-rgb-hetero`
  - 工作提交 `883ee521`、收口提交 `01d1cb0b`；基线 `73738357`。
只读边界：不跑 GPU、不跑 `--ignored`、不改代码/证据；所有数字从仓库内容与 git 对象现算。

输出：`recompute.json`（逐项 check），并把表格打到 stdout（`logs/recompute.log`）。
"""
import hashlib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = HERE
while not os.path.isdir(os.path.join(REPO, ".git")) and REPO != os.path.dirname(REPO):
    REPO = os.path.dirname(REPO)

BASE = "73738357"
WORK = "883ee521"
CLOSE = "01d1cb0b"
EVID = "docs/development-evidence/render-bvh/r08-rgb-hetero"
SCOPE_SRC = [
    "src/render/kernel/megakernel.rs",
    "src/render/kernel/megakernel_arrays.rs",
    "tests/render_r08_cameras.rs",
    "tests/support/render_bvh/r08_gpu.rs",
]
KERNEL_SOURCES = [
    "src/render/bvh/cuda/scene.cu",
    "src/render/kernel/cuda/render_math.cu",
    "src/render/kernel/cuda/render_common.cu",
    "src/render/kernel/cuda/render_shadow.cu",
    "src/render/kernel/cuda/render_shade.cu",
    "src/render/kernel/cuda/render_megakernel.cu",
]
ZERO_CHANGE = [
    "src/render/kernel/cuda/**",
    "src/render/kernel/megakernel_uploads.rs",
    "src/render/context/camera_output.rs",
    "src/render/context.rs",
    "src/render/context/params.rs",
    "src/render/camera.rs",
]

BOUNDS = {
    "src/render/kernel/megakernel.rs": 420,
    "src/render/kernel/megakernel_arrays.rs": 365,
    "tests/render_r08_cameras.rs": 380,
    "tests/support/render_bvh/r08_gpu.rs": 420,
    "tests/support/render_bvh/r08.rs": 380,
}

checks = []


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})


def run(args, text=True):
    proc = subprocess.run(args, cwd=REPO, capture_output=True)
    out = proc.stdout.decode("utf-8-sig", "replace") if text else proc.stdout
    return proc.returncode, out, proc.stderr.decode("utf-8", "replace")


def git(*args):
    return run(["git", *args])


def blob(rev, path):
    code, out, err = run(["git", "show", f"{rev}:{path}"], text=False)
    if code != 0:
        return None
    return out


def disk(path):
    full = os.path.join(REPO, path)
    if not os.path.isfile(full):
        return None
    with open(full, "rb") as handle:
        return handle.read()


def text_of(path):
    data = disk(path)
    return None if data is None else data.decode("utf-8-sig", "replace")


def lines(data):
    return data.count(b"\n")


# ---- 1. host 填充：三通道逐相机、无全局布尔残留 ----
mk = text_of("src/render/kernel/megakernel.rs")
arr = text_of("src/render/kernel/megakernel_arrays.rs")
cam = text_of("src/render/camera.rs")
assign_ok = all(
    f"let render_{ch} = channel_flags(&resolved.plan.render_{ch});" in (mk or "")
    for ch in ("rgb", "depth", "seg")
)
global_gone = bool(mk) and ("is_enabled(ImageOutput" not in mk) and (
    "use crate::render::ImageOutput;" not in mk
)
push_ok = bool(mk) and all(
    re.search(
        rf"array\(&u\.render_{ch}, RenderElem::Bool, &\[nrender\]\)", mk
    )
    for ch in ("rgb", "depth", "seg")
)
fn_ok = bool(arr) and (
    "pub(crate) fn channel_flags(flags: &[bool]) -> Vec<i32>" in arr
    and "flags.iter().map(|enabled| i32::from(*enabled)).collect()" in arr
)
len_ok = bool(cam) and (
    "if resolved.len() != ncam {" in cam
    and re.search(r"pub fn nrender\(&self\) -> usize \{\s*self\.plan\.cameras\.len\(\)",  # noqa: E501
                  text_of("src/render/context.rs") or "") is not None
)
add(
    "host_fill_per_camera",
    assign_ok and global_gone and push_ok and fn_ok and len_ok,
    {
        "三条赋值用 channel_flags(&resolved.plan.render_*)": assign_ok,
        "megakernel.rs 无 is_enabled(ImageOutput / ImageOutput import": global_gone,
        "三处实参仍按 [nrender] 形状": push_ok,
        "channel_flags 纯逐项映射实现": fn_ok,
        "plan.render_* 长度由 resolve_targets 校验=ncam、nrender=plan.cameras.len()": len_ok,
    },
)

# ---- 1b. 缺口归因：兄弟设备路径本就走逐相机掩码，全局布尔只剩 megakernel 三行 ----
cast = text_of("src/render/kernel/cast.rs") or ""
renderer = text_of("src/render/kernel/renderer.rs") or ""
uploads = text_of("src/render/kernel/megakernel_uploads.rs") or ""
sibling_ok = all(
    re.search(
        rf"\.render_{ch}\s*\.push\(i32::from\(plan\.render_{ch}\[position\]\)\)", cast
    )
    is not None
    for ch in ("depth", "seg")
)
sibling_upload_ok = (
    "let render_depth = upload(&self.session, &bool_words(&camera.render_depth))?;" in renderer
    and "let render_seg = upload(&self.session, &bool_words(&camera.render_seg))?;" in renderer
)
plumbing_ok = (
    "upload(session, &bool_words(inputs.render_rgb))" in uploads
    and "upload(session, &bool_words(inputs.render_depth))" in uploads
    and "upload(session, &bool_words(inputs.render_seg))" in uploads
)
residual = []
for root, _dirs, files in os.walk(os.path.join(REPO, "src", "render")):
    for name in files:
        if not name.endswith(".rs"):
            continue
        full = os.path.join(root, name)
        body = open(full, encoding="utf-8").read()
        if "vec![i32::from(" in body:
            residual.append(os.path.relpath(full, REPO).replace(os.sep, "/"))
add(
    "gap_attribution_only_megakernel_host",
    sibling_ok and sibling_upload_ok and plumbing_ok and not residual,
    {
        "cast.rs 逐位置渲染掩码（depth/seg）": sibling_ok,
        "renderer.rs 上传 camera.render_depth/seg（S7b1 已逐相机）": sibling_upload_ok,
        "megakernel_uploads 只做 bool_words 打包（无全局折叠）":
            "upload(session, &bool_words(inputs.render_rgb))" in uploads,
        "src/render 无残留 vec![i32::from( 全局折叠": residual,
    },
)

# ---- 2. ABI：81 槽、51/52/53 = Array(Bool)；内核签名同位置 = render_rgb/depth/seg ----
args_src = text_of("src/runtime/launcher/render/args.rs") or ""
start = args_src.find("pub static MEGAKERNEL_SLOTS")
block = args_src[start:] if start >= 0 else ""
if start >= 0:
    open_brace = block.find("= {")
    close_brace = block.find("\n    ];", open_brace + 3)
    if open_brace >= 0 and close_brace > open_brace:
        block = block[open_brace:close_brace]
entries = []
for line in block.splitlines():
    token = line.split("//")[0].strip().rstrip(",").strip()
    if re.fullmatch(r"(?:B|A\(\w+\)|S\(\w+\))", token):
        entries.append(token)
slots_ok = len(entries) == 81 and entries[51:54] == ["A(Bool)", "A(Bool)", "A(Bool)"]
add(
    "abi_slots_81",
    slots_ok,
    {"len": len(entries), "slots_51_52_53": entries[51:54] if len(entries) == 81 else None},
)

cu = text_of("src/render/kernel/cuda/render_megakernel.cu") or ""
sig = cu[cu.find('extern "C" __global__ void render_megakernel('):]
sig = sig[: sig.find(") {") + 1]
params = [p.strip() for p in sig[sig.find("(") + 1: -1].split(",")]
names = [p.split()[-1] for p in params]
sig_ok = len(params) == 81 and names[51:54] == ["render_rgb", "render_depth", "render_seg"]
add(
    "kernel_signature_slots",
    sig_ok,
    {"params": len(params), "params_51_52_53": names[51:54] if len(params) == 81 else None},
)

# ---- 3. 内核源身份：6 个 .cu 拼接 sha 现算 + 与基线 blob 相等 ----
concat = b""
cu_facts = []
for path in KERNEL_SOURCES:
    data = disk(path)
    concat += data
    cu_facts.append(
        {
            "path": path,
            "bytes": len(data),
            "lf": lines(data),
            "crlf": data.count(b"\r\n"),
            "blob_base": git("rev-parse", f"{BASE}:{path}")[1].strip(),
            "blob_close": git("rev-parse", f"{CLOSE}:{path}")[1].strip(),
            "blob_disk": git("hash-object", path)[1].strip(),
        }
    )
source_sha = hashlib.sha256(concat).hexdigest()
blob_equal = all(
    row["blob_base"] == row["blob_close"] == row["blob_disk"] for row in cu_facts
)
add(
    "kernel_source_sha",
    source_sha == "e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c"
    and blob_equal,
    {"concat_bytes": len(concat), "source_sha256": source_sha, "cu_blob_base_eq_close_eq_disk": blob_equal,
     "files": cu_facts},
)

aa = disk("src/render/kernel/cuda/aa_resolve.cu")
aa_sha = hashlib.sha256(aa).hexdigest() if aa else None
add(
    "aa_resolve_source_sha",
    aa_sha == "0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76",
    {"bytes": len(aa) if aa else 0, "source_sha256": aa_sha},
)

# ---- 4. 范围：diff 文件集合 + 零改动文件 ----
code, out, _ = git("diff", "--name-status", f"{BASE}..{CLOSE}")
rows = [line.split("\t") for line in out.splitlines() if line.strip()]
expected_code = {
    "M .normify/changes/2026-10-28-render-per-camera-rgb-gap.json",
    "A .normify/changes/2026-10-29-render-r08-rgb-hetero-check.json",
    "A .normify/changes/2026-10-29-render-r08-rgb-hetero-validate.json",
    "A .normify/changes/2026-10-29-render-r08-rgb-hetero.json",
    "M .normify/modules/mjwarp/render.md",
    "A .normify/runs/2026-10-29-render-r08-rgb-hetero/clippy-cuda.json",
    "A .normify/runs/2026-10-29-render-r08-rgb-hetero/clippy.json",
    "A .normify/runs/2026-10-29-render-r08-rgb-hetero/fmt.json",
    "A .normify/runs/2026-10-29-render-r08-rgb-hetero/unit.json",
    f"A {EVID}/README.md",
    f"A {EVID}/logs/kernel-gate.txt",
    f"A {EVID}/logs/line-counts.txt",
    f"A {EVID}/logs/no-run.txt",
    f"A {EVID}/logs/scope-diff.txt",
    f"A {EVID}/logs/source-sha.txt",
    f"A {EVID}/logs/unit.txt",
    f"A {EVID}/receipt.json",
    "M src/render/kernel/megakernel.rs",
    "M src/render/kernel/megakernel_arrays.rs",
    "M tests/render_r08_cameras.rs",
    "M tests/support/render_bvh/r08_gpu.rs",
}
actual = {f"{row[0]} {row[1]}" for row in rows}
add(
    "scope_diff_exact",
    actual == expected_code,
    {"count": len(actual), "missing": sorted(expected_code - actual), "extra": sorted(actual - expected_code)},
)

zero_results = {}
for path in ZERO_CHANGE:
    code, out, _ = git("diff", "--name-only", f"{BASE}..{CLOSE}", "--", path)
    zero_results[path] = out.strip()
add("zero_change_files", all(v == "" for v in zero_results.values()), zero_results)

code, out, _ = git("diff", "--name-only", f"{BASE}..{CLOSE}", "--", "fixtures", "scripts/validate")
criteria_hits = [line for line in out.splitlines() if line.strip()]
add("criteria_reference_tolerance_untouched", not criteria_hits, {"hits": criteria_hits})

# ---- 5. 行数棘轮（LF 口径，before 取基线 blob、after 取现盘）----
receipt = json.loads(text_of(f"{EVID}/receipt.json"))
line_rows = {}
line_ok = True
for path, bound in BOUNDS.items():
    before = lines(blob(BASE, path))
    after = lines(disk(path))
    claimed = receipt["line_counts"][path]
    row_ok = (
        before == claimed["before"]
        and after == claimed["after"]
        and claimed["bound"] == bound
        and after <= bound
    )
    line_ok = line_ok and row_ok
    line_rows[path] = {"before": before, "after": after, "bound": bound, "match": row_ok}
add("line_counts", line_ok, line_rows)

# ---- 6. 证据：committed==现盘、receipt /task_id、LF 无 BOM、禁词 0 ----
code, out, _ = git("diff", "--quiet", CLOSE, "--", EVID)
code2, out2, _ = git("diff", "--quiet", "HEAD", "--", EVID)
add(
    "evidence_committed_eq_disk",
    code == 0 and code2 == 0,
    {"diff_vs_close_exit": code, "diff_vs_head_exit": code2},
)

task_ok = receipt.get("task_id") == "2026-10-29-render-r08-rgb-hetero"
add("receipt_task_id", task_ok, {"task_id": receipt.get("task_id")})

hygiene = {}
bad = []
for root, _dirs, files in os.walk(os.path.join(REPO, EVID)):
    for name in sorted(files):
        full = os.path.join(root, name)
        rel = os.path.relpath(full, REPO).replace(os.sep, "/")
        data = open(full, "rb").read()
        row = {"bytes": len(data), "crlf": data.count(b"\r\n"), "bom": data.startswith(b"\xef\xbb\xbf")}
        hygiene[rel] = row
        if row["crlf"] or row["bom"]:
            bad.append(rel)
add("evidence_lf_no_bom", not bad and len(hygiene) == 8, {"files": hygiene, "bad": bad})

# 禁词用码点构造，脚本本身不含字面量。
terms = [
    "".join(map(chr, [0x9010, 0x76F8, 0x673A, 0x20, 0x52, 0x47, 0x42, 0x20, 0x5DF2, 0x9A8C, 0x6536])),
    "".join(map(chr, [0x41, 0x42, 0x49, 0x20, 0x7B49, 0x4EF7])),
    "".join(map(chr, [0x63A5, 0x53E3, 0x7B49, 0x4EF7])),
]
hits = []
for rel in hygiene:
    body = text_of(rel)
    for term in terms:
        if term in body:
            hits.append({"file": rel, "term_codepoints": [hex(ord(c)) for c in term]})
add("evidence_forbidden_terms_zero", not hits, {"terms": len(terms), "hits": hits})

# ---- 7. 规划：缺口卡指向、S3 窗口卡存在且未 start ----
gap = json.loads(text_of(".normify/changes/2026-10-28-render-per-camera-rgb-gap.json"))
main = json.loads(text_of(".normify/changes/2026-10-29-render-r08-rgb-hetero.json"))
s3 = json.loads(text_of(".normify/changes/2026-10-29-render-r08-rgb-hetero-validate.json"))
plan_ok = (
    "2026-10-29-render-r08-rgb-hetero" in gap.get("note", "")
    and main.get("status") == "verified"
    and main.get("closed_at")
    and s3.get("id") == "2026-10-29-render-r08-rgb-hetero-validate"
    and s3.get("status") not in ("in_progress", "verified", "running")
)
add(
    "planning_links",
    plan_ok,
    {
        "gap_note_points_to_main": "2026-10-29-render-r08-rgb-hetero" in gap.get("note", ""),
        "main_status": main.get("status"),
        "main_closed_at": main.get("closed_at"),
        "s3_status": s3.get("status"),
    },
)

# ---- 8. 测试静态覆盖（只读代码，不执行 GPU 用例）----
gpu = text_of("tests/support/render_bvh/r08_gpu.rs")
entry = text_of("tests/render_r08_cameras.rs")
cases_ok = all(
    marker in gpu
    for marker in ('("cam0_on", [true, false])', '("cam1_on", [false, true])', '("all_off", [false, false])')
)
sha_ok = "assert_eq!(hashes[0], hashes[1]" in gpu
seg_start = gpu.find("pub fn heterogeneous_depth_seg_channels()")
seg_end = gpu.find("\n/// ", seg_start + 10)
seg_body = gpu[seg_start:seg_end] if seg_start >= 0 else ""
mega_cross_ok = "MegakernelRenderer" in seg_body and "mega_views.depth_raw(1).is_err()" in seg_body
entry_ok = bool(entry) and re.search(
    r"#\[ignore[^\]]*\]\s*fn gpu_heterogeneous_rgb_channels\(\)", entry
) is not None
unit_names = [
    "channel_flags_maps_each_camera_independent",
    "channel_flags_handles_all_false_and_empty",
]
unit_ok = all(name in (arr or "") for name in unit_names)
add(
    "tests_static_coverage",
    cases_ok and sha_ok and mega_cross_ok and entry_ok and unit_ok,
    {
        "rgb_three_states": cases_ok,
        "two_run_sha_assert": sha_ok,
        "depth_seg_mega_cross_mask": mega_cross_ok,
        "gpu_entry_ignored": entry_ok,
        "unit_tests_present": unit_ok,
    },
)

# ---- 9. QA 自跑门禁日志 ----
unit_log = text_of(f"{EVID}-check/logs/unit.log") or ""
res = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", unit_log)
passed = sum(int(r[1]) for r in res)
failed = sum(int(r[2]) for r in res)
ignored = sum(int(r[3]) for r in res)
new_tests_ok = all(
    f"test render::kernel::megakernel_arrays::tests::{name} ... ok" in unit_log for name in unit_names
)
gpu_ignored_ok = (
    "test gpu_heterogeneous_rgb_channels ... ignored" in unit_log
    and "test gpu_heterogeneous_depth_seg_channels ... ignored" in unit_log
)
add(
    "qa_unit_gate",
    len(res) > 0 and passed == 1106 and failed == 0 and ignored == 391 and new_tests_ok and gpu_ignored_ok,
    {
        "result_lines": len(res),
        "passed": passed,
        "failed": failed,
        "ignored": ignored,
        "new_unit_tests_ok": new_tests_ok,
        "gpu_entries_compiled_ignored": gpu_ignored_ok,
    },
)

gates_txt = text_of(f"{EVID}-check/logs/gates.txt") or ""
gate_lines = [
    line for line in gates_txt.splitlines()
    if re.match(r"^(fmt|clippy|clippy-cuda|unit):", line)
]
gates_ok = len(gate_lines) == 4 and all("exit=0" in line for line in gate_lines)
fmt_empty = (text_of(f"{EVID}-check/logs/fmt.log") or "").strip() == ""
clippy_logs = [
    text_of(f"{EVID}-check/logs/clippy.log") or "",
    text_of(f"{EVID}-check/logs/clippy-cuda.log") or "",
]
clippy_ok = all(("Finished" in body or body.strip() == "") and "error" not in body for body in clippy_logs)
add("qa_gate_logs", bool(gates_ok) and fmt_empty and clippy_ok,
    {"gates_txt_exit0": bool(gates_ok), "fmt_log_empty": fmt_empty, "clippy_logs_clean": clippy_ok})

# ---- 汇总 ----
summary = {
    "task": "2026-10-29-render-r08-rgb-hetero-check",
    "reviewed": {"base": BASE, "work": WORK, "close": CLOSE},
    "checks": checks,
    "problems": [c for c in checks if not c["ok"]],
}
with open(os.path.join(HERE, "recompute.json"), "w", encoding="utf-8", newline="\n") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

for check in checks:
    print(f"[{'PASS' if check['ok'] else 'FAIL'}] {check['id']}")
    print(json.dumps(check["detail"], ensure_ascii=False, sort_keys=True))
print(f"\nchecks={len(checks)} problems={len(summary['problems'])}")
sys.exit(0 if not summary["problems"] else 1)
