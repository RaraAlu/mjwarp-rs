#!/usr/bin/env python3
"""R05 拆分①（megakernel 上传段纯移动）独立复核重算脚本。

口径：
- 证据目录内文件按工作树（LF 落盘后即 committed blob）；仓库内被复核文件按 `git show HEAD:<path>`（LF 归一）。
- 所有数字由本脚本现算，不采信任务卡 / 实现证据 / Arch / Backend 给出的数字（仅作为"待核声明"）。
- 只读：不写工作树、不跑 GPU、不重开窗口。
"""

import hashlib
import json
import pathlib
import re
import subprocess
import sys

try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

R = pathlib.Path(  # repo root（按 git 顶层解析，脚本放哪都能跑）
    subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=pathlib.Path(__file__).resolve().parent,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
)
EV = "docs/development-evidence/render-bvh/r05-split-megakernel"
CHECK_EV = "docs/development-evidence/render-bvh/r05-split-megakernel-check"
PRE = "00ed9efb"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()

BASELINE_SOURCE_SHA = "2d2c27df985882c2b571514579dff19f9103bfc903ec786db42a592ce1bd8c2a"
BASELINE_PTX_SHA = "e955bb4b0280228639f3e544b459873c83d1dae672662a43a96cccc8dabc4ffa"
BASELINE_PTX_BYTES = 258660

problems = []
report = {"head": HEAD, "pre_split": PRE}


def blob(rel):
    return subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=R, capture_output=True, check=True).stdout


def old_blob(rel):
    return subprocess.run(["git", "show", f"{PRE}:{rel}"], cwd=R, capture_output=True, check=True).stdout


def sha(b):
    return hashlib.sha256(b).hexdigest()


def text(b):
    return b.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")


def line_count(b):
    return b.count(b"\n") + (0 if b.endswith(b"\n") else 1)


# ---------------------------------------------------------------- 1. 纯移动：内核源 / PTX
REPORT = report
chain = [
    "src/render/bvh/cuda/scene.cu",
    "src/render/kernel/cuda/render_math.cu",
    "src/render/kernel/cuda/render_common.cu",
    "src/render/kernel/cuda/render_shadow.cu",
    "src/render/kernel/cuda/render_shade.cu",
    "src/render/kernel/cuda/render_megakernel.cu",
]
h = hashlib.sha256()
cuda_same = True
for rel in chain:
    h.update(blob(rel))
    cuda_same = cuda_same and (sha(blob(rel)) == sha(old_blob(rel)))
concat_sha = h.hexdigest()
report["kernel_source"] = {
    "concat_files": chain,
    "concat_sha256": concat_sha,
    "baseline_source_sha256": BASELINE_SOURCE_SHA,
    "source_sha_equals_baseline": concat_sha == BASELINE_SOURCE_SHA,
    "cuda_files_byte_identical_to_pre_split": cuda_same,
    "note": "source_sha256 = sha256(concat of 6 include_str files)（src/runtime/launcher/artifact.rs:259）；"
            "6 个文件与拆分前逐字节相同 ⇒ 拆分前后 source 字符串相同，PTX 输入同一",
}
if concat_sha != BASELINE_SOURCE_SHA:
    problems.append("concat source sha != baseline")
if not cuda_same:
    problems.append("cuda sources changed")

# ---------------------------------------------------------------- 2. 行数
lines = {}
for rel, claim in {
    "src/render/kernel/megakernel.rs": 406,
    "src/render/kernel/megakernel_uploads.rs": 384,
    "src/render/kernel/mod.rs": 474,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "src/runtime/launcher/render/args.rs": 494,
}.items():
    got = line_count(blob(rel))
    lines[rel] = {"lines": got, "claim": claim, "match": got == claim}
    if got != claim:
        problems.append(f"line count mismatch {rel}: {got} != {claim}")
report["lines"] = lines
report["lines"]["megakernel_rs_before"] = line_count(old_blob("src/render/kernel/megakernel.rs"))
report["lines_limits"] = {
    "megakernel_rs_target_le_430": lines["src/render/kernel/megakernel.rs"]["lines"] <= 430,
    "both_new_files_le_500": (
        lines["src/render/kernel/megakernel.rs"]["lines"] <= 500
        and lines["src/render/kernel/megakernel_uploads.rs"]["lines"] <= 500
    ),
}

# ---------------------------------------------------------------- 3. 移动段逐行对照（旧 L203-344 → 新模块）
old_mk = text(old_blob("src/render/kernel/megakernel.rs"))
new_mk = text(blob("src/render/kernel/megakernel.rs"))
new_up = text(blob("src/render/kernel/megakernel_uploads.rs"))

old_lines = old_mk.split("\n")
seg_start = next(i for i, l in enumerate(old_lines) if "---- 纹理数组" in l)  # 0-based -> L203
seg_end = next(i for i, l in enumerate(old_lines) if "let seg_scratch" in l)
while not old_lines[seg_end].rstrip().endswith(";"):
    seg_end += 1
moved = old_lines[seg_start : seg_end + 1]
report["moved_segment"] = {
    "old_line_range": f"L{seg_start + 1}-{seg_end + 1}",
    "line_count": len(moved),
    "kept_after_segment": old_lines[seg_end + 1 : seg_end + 4],
}
if (seg_start + 1, seg_end + 1) != (203, 344):
    problems.append(f"moved range {seg_start + 1}-{seg_end + 1} != L203-344")

# UploadInputs 字面量：字段 -> 传入表达式
lit_src = re.search(r"UploadInputs \{(.+?)\n        \}\)", new_mk, re.S).group(1)
bindings = {}
for m in re.finditer(r"([A-Za-z_][A-Za-z0-9_]*): (&?[A-Za-z_][A-Za-z0-9_.]*),", lit_src):
    bindings[m.group(1)] = m.group(2)
for m in re.finditer(r"^\s{12}([A-Za-z_][A-Za-z0-9_]*),$", lit_src, re.M):
    bindings[m.group(1)] = "&" + m.group(1)

# 70 个 upload(...) 调用：顺序 + 实参
def upload_calls(src):
    out = []
    pat = re.compile(r"(?:let\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*upload\s*\(")
    i = 0
    while True:
        m = pat.search(src, i)
        if not m:
            break
        j = m.end() - 1
        depth = 0
        while j < len(src):
            if src[j] == "(":
                depth += 1
            elif src[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        inner = src[m.end() : j]
        norm = re.sub(r"\s+", "", inner.replace("&self.session", "session").replace("self.session", "session"))
        out.append((m.group(1), norm))
        i = j
    return out


old_calls = upload_calls(old_mk)
new_calls = upload_calls(new_up)
call_mismatch = []
for k in range(max(len(old_calls), len(new_calls))):
    a = old_calls[k] if k < len(old_calls) else None
    b = new_calls[k] if k < len(new_calls) else None
    if a is None or b is None or a[0] != b[0]:
        call_mismatch.append({"index": k, "old": a, "new": b})
        continue
    # 旧实参引用局部 X；新实参为 inputs.X（须在 UploadInputs 字面量里绑定到同名局部 &X）
    o_arg = a[1].split(",", 1)[1] if "," in a[1] else ""
    n_arg = b[1].split(",", 1)[1] if "," in b[1] else ""
    def aggressive(s):
        s = re.sub(r"&?inputs\.([A-Za-z_][A-Za-z0-9_]*)", r"\1", s)
        s = s.replace("resolved.output.", "").replace("layout.", "").replace("&", "").replace("self.", "")
        return re.sub(r"\s+", "", s)

    o_norm, n_norm = aggressive(o_arg), aggressive(n_arg)
    fields = re.findall(r"inputs\.([A-Za-z_][A-Za-z0-9_]*)", n_arg)
    bind_ok = all(bindings.get(f, "").lstrip("&") == f for f in fields)
    if o_norm != n_norm or not bind_ok:
        call_mismatch.append(
            {"index": k, "old": a, "new": b, "old_norm": o_norm, "new_norm": n_norm,
             "bindings": {f: bindings.get(f) for f in fields}}
        )
report["upload_calls"] = {
    "count_old": len(old_calls),
    "count_new": len(new_calls),
    "sequence_and_args_equivalent": not call_mismatch,
    "mismatches": call_mismatch,
}
if call_mismatch:
    problems.append(f"upload call mismatches: {len(call_mismatch)}")

# 81 个 args.push：逐槽顺序与内容（参考前缀 u. 归一）
def pushes(src):
    out = []
    for m in re.finditer(r"args\.push\((.+?)\);\n", src, re.S):
        b = re.sub(r"\s+", "", m.group(1))
        b = b.replace("&u.", "&").replace("u.", "").replace(",)", ")")
        out.append(b)
    return out


old_push = pushes(old_mk)
new_push = pushes(new_mk)
report["args_push"] = {
    "count_old": len(old_push),
    "count_new": len(new_push),
    "slot_sequence_identical": old_push == new_push,
    "slots": len(new_push),
}
if old_push != new_push:
    problems.append("args.push sequence differs")

# 移动段整体：归一后逐行比对（把 inputs.X 还原成 &X / resolved.output 还原成 layout）
def norm(line):
    s = line.strip()
    s = s.replace("&self.session", "session").replace("self.session", "session")
    s = re.sub(r"&?inputs\.([A-Za-z_][A-Za-z0-9_]*)", r"\1", s)
    s = s.replace("resolved.output.", "layout.")
    s = s.replace("&", "").replace("self.", "")
    s = re.sub(r"\s+", "", s).replace(",)", ")")
    return s


up_lines = new_up.split("\n")
s2 = next(i for i, l in enumerate(up_lines) if "---- 纹理数组" in l and "----" in l)
e2 = next(i for i, l in enumerate(up_lines) if "let seg_scratch" in l)
while not up_lines[e2].rstrip().endswith(";"):
    e2 += 1
new_moved = up_lines[s2 : e2 + 1]
diffs = []
for old_l, new_l in zip(moved, new_moved):
    a, b = norm(old_l), norm(new_l)
    if a != b:
        diffs.append({"old": a, "new": b})
report["moved_body"] = {
    "old_nonblank": len([l for l in moved if l.strip()]),
    "new_nonblank": len([l for l in new_moved if l.strip()]),
    "normalized_diff_count": len(diffs),
    "diffs": diffs,
    "note": "归一规则：去掉 self.session 借用、inputs.X 还原为 &X、resolved.output 还原为 layout、空白/逗号尾差；"
            "差异仅出现在引用接线处，逐条人工判定等价（见 README §4 O3）",
}

# ---------------------------------------------------------------- 4. API / ABI / 测试零变化
report["api_abi"] = {}
for rel in chain + ["src/render/kernel/mod.rs", "src/runtime/launcher/render/args.rs",
                    "src/render/context/camera_output.rs", "src/model/render.rs"]:
    same = sha(blob(rel)) == sha(old_blob(rel))
    report["api_abi"][rel] = "unchanged" if same else "CHANGED"
    if not same:
        problems.append(f"changed: {rel}")

tests_out = subprocess.run(
    ["git", "diff", "--stat", f"{PRE}..HEAD", "--", "tests", "src/tests", "Cargo.toml", "Cargo.lock"],
    cwd=R, capture_output=True, text=True,
).stdout.strip()
report["tests_and_manifest_untouched"] = tests_out == ""
if tests_out:
    problems.append("tests/Cargo changed: " + tests_out)

# 公开 API：pub fn 签名行
def pub_fns(src):
    return sorted(re.findall(r"pub fn [a-z_]+\([^{]*?\{", src, re.S))


report["public_api"] = {
    "pub_fn_signatures_identical": sorted(
        re.sub(r"\s+", " ", x) for x in re.findall(r"    pub fn [a-z_]+\([^)]*", new_mk)
    ) == sorted(re.sub(r"\s+", " ", x) for x in re.findall(r"    pub fn [a-z_]+\([^)]*", old_mk)),
}
if not report["public_api"]["pub_fn_signatures_identical"]:
    problems.append("public fn signatures differ")

# 模块声明与登记
up_decl = re.findall(r"#\[path = \"megakernel_uploads.rs\"\]\s*\nmod megakernel_uploads;", new_mk)
mod_rs_same = sha(blob("src/render/kernel/mod.rs")) == sha(old_blob("src/render/kernel/mod.rs"))
render_md = (R / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
report["module_wiring"] = {
    "path_declaration_count": len(up_decl),
    "kernel_mod_rs_unchanged": mod_rs_same,
    "render_md_source_entries": render_md.count("[[source]]"),
    "megakernel_uploads_registered": "src/render/kernel/megakernel_uploads.rs" in render_md,
}
if len(up_decl) != 1 or not mod_rs_same or not report["module_wiring"]["megakernel_uploads_registered"]:
    problems.append("module wiring problem")

# ---------------------------------------------------------------- 5. 证据目录（实现片）
ev = R / EV
ev_files = sorted(p for p in ev.rglob("*") if p.is_file())
manifest = {}
for line in (ev / "manifest.txt").read_text(encoding="utf-8").split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h_, n_, rel = line.split(" ", 2)
        manifest[rel] = (h_, int(n_))
ev_rel = {p.relative_to(ev).as_posix() for p in ev_files if p.name != "manifest.txt"}
manifest_problems = []
if ev_rel != set(manifest):
    manifest_problems.append(f"set mismatch {ev_rel ^ set(manifest)}")
for rel in sorted(ev_rel & set(manifest)):
    b = blob(f"{EV}/{rel}")
    if (sha(b), len(b)) != manifest[rel]:
        manifest_problems.append(f"{rel}: manifest {manifest[rel]} blob {(sha(b), len(b))}")
enc = []
for p in ev_files:
    b = p.read_bytes()
    if b.startswith(b"\xef\xbb\xbf"):
        enc.append((p.name, "BOM"))
    if b"\r" in b:
        enc.append((p.relative_to(ev).as_posix(), "CR"))
BANNED = ["".join(chr(c) for c in seq) for seq in (
    (71, 49, 48, 24050, 23454, 29616),                            # banned word 1
    (23436, 25972, 29289, 29702, 36890, 36807),                   # banned word 2
    (21452, 24179, 21488, 36890, 36807),                          # banned word 3
    (19978, 28216, 23436, 20840, 31561, 20215),                   # banned word 4
    (21407, 29983, 25918, 34892),                                 # banned word 5
    (31934, 24230, 36798, 26631),                                 # banned word 6
    (23436, 25972, 28210, 26524, 36890, 36807),                   # banned word 7
    (23436, 25972, 82, 48, 49, 36890, 36807),                     # banned word 8
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),     # banned word 9
)]
word_hits = []
for p in ev_files:
    t = p.read_text(encoding="utf-8", errors="replace")
    word_hits += [(p.relative_to(ev).as_posix(), w) for w in BANNED if w in t]
readme = (ev / "README.md").read_text(encoding="utf-8")
receipt = json.loads((ev / "receipt.json").read_text(encoding="utf-8"))
report["implementation_evidence"] = {
    "files": len(ev_files),
    "manifest_entries": len(manifest),
    "manifest_self_reference": "manifest.txt" in manifest,
    "manifest_problems": manifest_problems,
    "encoding_problems": enc,
    "forbidden_word_hits": word_hits,
    "readme_lines": line_count(readme.encode()),
    "receipt_task_id": receipt.get("task_id"),
}
if manifest_problems:
    problems.append(f"manifest problems: {manifest_problems}")
if enc:
    problems.append(f"encoding problems: {enc}")
if word_hits:
    problems.append(f"forbidden words: {word_hits}")
if receipt.get("task_id") != "2026-10-28-render-bvh-r05-split-megakernel":
    problems.append("receipt task_id mismatch")

# ---------------------------------------------------------------- 6. 提交范围
commit_files = {}
for c in ["5d4d23d0", "00ea2baf"]:
    out = subprocess.run(["git", "show", "--name-only", "--pretty=format:", c], cwd=R, capture_output=True, text=True).stdout
    commit_files[c] = [x for x in out.split("\n") if x.strip()]
report["commits"] = {c: {"files": len(f), "paths": f} for c, f in commit_files.items()}
allowed_prefix = ("src/render/kernel/megakernel.rs", "src/render/kernel/megakernel_uploads.rs",
                  EV + "/", ".normify/")
outside = [p for f in commit_files.values() for p in f if not p.startswith(allowed_prefix)]
report["commits"]["outside_card_scope"] = outside
if outside:
    problems.append(f"files outside card scope: {outside}")

# ---------------------------------------------------------------- 7. 门禁日志（本复核自己跑）
log_dir = R / CHECK_EV / "logs"
gates = {}
for name in ["fmt", "clippy", "clippy-cuda", "unit", "kernel-gate"]:
    p = log_dir / f"{name}.log"
    if not p.exists():
        gates[name] = "missing"
        continue
    t = p.read_text(encoding="utf-8-sig", errors="replace")
    if name == "unit":
        res = [l for l in t.split("\n") if l.startswith("test result: ")]
        passed = sum(int(re.search(r"(\d+) passed", l).group(1)) for l in res)
        failed = sum(int(re.search(r"(\d+) failed", l).group(1)) for l in res)
        gates[name] = {"result_lines": len(res), "passed": passed, "failed": failed}
        if failed or passed != 1069:
            problems.append(f"unit gate: {passed}/{failed}")
    elif name == "kernel-gate":
        m = re.search(r"source_sha256=(\w+) ptx_sha256=(\w+) ptx_bytes=(\d+)", t)
        gates[name] = {
            "source_sha256": m.group(1) if m else None,
            "ptx_sha256": m.group(2) if m else None,
            "ptx_bytes": int(m.group(3)) if m else None,
            "matches_baseline": bool(m)
            and m.group(1) == BASELINE_SOURCE_SHA
            and m.group(2) == BASELINE_PTX_SHA
            and int(m.group(3)) == BASELINE_PTX_BYTES,
            "exit_marker": re.search(r"KERNEL_GATE_EXIT=(\d+)", t).group(1) if "KERNEL_GATE_EXIT=" in t else None,
        }
        if not gates[name]["matches_baseline"]:
            problems.append("kernel gate sha != baseline")
    else:
        em = re.search(r"^(FMT|CLIPPY|CLIPPY_CUDA)_EXIT=(\d+)", t, re.M)
        gates[name] = {"exit": em.group(2) if em else None}
        if not em or em.group(2) != "0":
            problems.append(f"gate {name} exit != 0")
report["gates"] = gates

report["problems"] = problems
out = pathlib.Path(__file__).resolve().parent / "recompute.json"
out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(report, ensure_ascii=False, indent=2))
print("RESULT problems=" + str(len(problems)))
for item in problems:
    print("PROBLEM " + str(item))
