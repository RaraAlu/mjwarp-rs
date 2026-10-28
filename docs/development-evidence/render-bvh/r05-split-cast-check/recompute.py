#!/usr/bin/env python3
"""R05 拆分②（cast.rs MeshDevice 段纯移动）独立复核重算脚本。

口径：
- 本复核证据目录内文件按 index blob（提交后 = committed blob，LF）；被复核仓库文件按 `git show HEAD:<path>`。
- 全部数字由本脚本现算，不采信任务卡 / 实现证据 / Arch / Backend 给出的数字（只作"待核声明"）。
- 只读：不写工作区、不跑 GPU、不重开窗口；四门禁日志由 QA 另行独立复跑。

校验面：
  1. 迁出块 L284-372（89 行）字节级一致；
  2. `cast.rs` 全量重构等价：旧文件经「import 收窄 + 插入 7 行 + 删除 89 行块 + 1 空行 + to_i32 可见性」后逐字节等于新文件；
  3. PTX / source 零变化（6 段 include_str 拼接 sha + QA 自跑内核编译门）；
  4. API / 对外路径（`pub use cast_mesh::MeshDevice;`、mod.rs 零改动、外部引用文件零改动、公开项集合不变、`to_i32` 复用）；
  5. 行数 / 红线 / 模块源登记；
  6. 证据完整性与提交范围。
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

R = pathlib.Path(
    subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=pathlib.Path(__file__).resolve().parent,
        capture_output=True, text=True, check=True,
    ).stdout.strip()
)
PRE = "4a5a8449"
IMPL_EV = "docs/development-evidence/render-bvh/r05-split-cast"
CHECK_EV = "docs/development-evidence/render-bvh/r05-split-cast-check"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()

BASELINE_SOURCE_SHA = "2d2c27df985882c2b571514579dff19f9103bfc903ec786db42a592ce1bd8c2a"
BASELINE_PTX_SHA = "e955bb4b0280228639f3e544b459873c83d1dae672662a43a96cccc8dabc4ffa"
BASELINE_PTX_BYTES = 258660
BLOCK_LINES = 89
BLOCK_SHA_CLAIM = "0b68b0c930d1a09b15a819804ad61c3ee762d028f0c67c847690d44bf643c96e"
BLOCK_BYTES_CLAIM = 3296

# 禁词（9 条 = 6 全局 + 本片 3 词表）；以码点构造，避免脚本自身命中
BANNED = ["".join(chr(c) for c in seq) for seq in (
    (71, 49, 48, 24050, 23454, 29616),
    (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807),
    (19978, 28216, 23436, 20840, 31561, 20215),
    (21407, 29983, 25918, 34892),
    (31934, 24230, 36798, 26631),
    (23436, 25972, 28210, 26524, 36890, 36807),
    (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
)]

problems = []


def blob(rev, rel):
    return subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=R, capture_output=True, check=True).stdout


def head(rel):
    return blob("HEAD", rel)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lf_count(b):
    return b.count(b"\n") + (0 if b.endswith(b"\n") else 1)


def split_lines(b):
    return b.decode("utf-8").split("\n")


def check(cond, msg):
    if not cond:
        problems.append(msg)
    return cond


rep = {"head": HEAD, "pre_split": PRE}

# ---------------------------------------------------------------- 1. 迁出块字节一致（最硬）
old_cast = blob(PRE, "src/render/kernel/cast.rs")
new_cast = head("src/render/kernel/cast.rs")
new_mesh = head("src/render/kernel/cast_mesh.rs")
ol, nl, ml = split_lines(old_cast), split_lines(new_cast), split_lines(new_mesh)

old_block = "\n".join(ol[283:372]).encode() + b"\n"   # old L284-372
new_block = "\n".join(ml[13:102]).encode() + b"\n"    # cast_mesh L14-102
rep["moved_block"] = {
    "old_range": "L284-372",
    "new_range": "cast_mesh.rs L14-102",
    "lines": BLOCK_LINES,
    "old_sha256": sha(old_block),
    "old_bytes": len(old_block),
    "new_sha256": sha(new_block),
    "new_bytes": len(new_block),
    "byte_identical": old_block == new_block,
    "sha_equals_implementation_claim": sha(old_block) == BLOCK_SHA_CLAIM,
    "bytes_equal_implementation_claim": len(old_block) == BLOCK_BYTES_CLAIM,
}
check(rep["moved_block"]["byte_identical"], "moved block not byte identical")
check(rep["moved_block"]["byte_identical"] and rep["moved_block"]["sha_equals_implementation_claim"],
      "block sha != implementation claim")
check(rep["moved_block"]["bytes_equal_implementation_claim"], "block bytes != implementation claim")
check(lf_count(old_cast) == 494, f"pre-split cast.rs lines {lf_count(old_cast)} != 494")
check(lf_count(new_cast) == 411, f"new cast.rs lines {lf_count(new_cast)} != 411")
check(lf_count(new_mesh) == 102, f"cast_mesh lines {lf_count(new_mesh)} != 102")

rep["cast_mesh_composition"] = {
    "header_lines": 13,
    "block_lines": 102 - 13,
    "line9": ml[8],
    "line10": ml[9],
    "line12": ml[11],
}
check(ml[8] == "use crate::render::bvh::{MeshBvh, MeshNodeKind};", "cast_mesh import line9 unexpected")
check(ml[9] == "use crate::runtime::launcher::LauncherError;", "cast_mesh import line10 unexpected")
check(ml[11] == "use super::to_i32;", "cast_mesh to_i32 import unexpected")

# ---------------------------------------------------------------- 2. cast.rs 全量重构等价（4 个 hunk）
diff = subprocess.run(
    ["git", "diff", "-U0", f"{PRE}..HEAD", "--", "src/render/kernel/cast.rs"],
    cwd=R, capture_output=True, check=True,
).stdout.decode()
hunks = [h for h in re.split(r"\n(?=@@ )", diff) if h.startswith("@@ ")]


def hunk_lines(h):
    removed = [l[1:] for l in h.split("\n") if l.startswith("-") and not l.startswith("---")]
    added = [l[1:] for l in h.split("\n") if l.startswith("+") and not l.startswith("+++")]
    return removed, added


rep["cast_rs_hunks"] = {
    "count": len(hunks),
    "headers": [h.split("\n")[0] for h in hunks],
    "removed_total": sum(len(hunk_lines(h)[0]) for h in hunks),
    "added_total": sum(len(hunk_lines(h)[1]) for h in hunks),
}
check(len(hunks) == 4, f"cast.rs hunks {len(hunks)} != 4")

rem0, add0 = hunk_lines(hunks[0])   # import 收窄
rem1, add1 = hunk_lines(hunks[1])   # 插入 7 行（注释 2 + #[path] + mod + 空行 + pub use + 空行）
rem2, add2 = hunk_lines(hunks[2])   # 删除 90 行 = 89 行块 + 1 空行
rem3, add3 = hunk_lines(hunks[3])   # to_i32 可见性

rep["cast_rs_changes"] = {
    "hunk0_removed": rem0,
    "hunk0_added": add0,
    "hunk1_added": add1,
    "hunk1_added_count": len(add1),
    "hunk2_removed_count": len(rem2),
    "hunk2_added_count": len(add2),
    "hunk3_removed": rem3,
    "hunk3_added": add3,
}
check(rem0 == ["use crate::render::bvh::{MeshBvh, MeshNodeKind, NodeKind, SceneBvh};"], "hunk0 removed import unexpected")
check(add0 == ["use crate::render::bvh::{NodeKind, SceneBvh};"], "hunk0 narrowed import unexpected")
check(len(add1) == 7, f"insertion lines {len(add1)} != 7")
check(len(rem2) == BLOCK_LINES + 1, f"hunk2 removed {len(rem2)} != 90")
check(len(add2) == 0, "hunk2 has additions")
check(rem3 == ["fn to_i32(value: usize, field: &'static str) -> Result<i32, LauncherError> {"], "hunk3 removed line unexpected")
check(add3 == ["pub(super) fn to_i32(value: usize, field: &'static str) -> Result<i32, LauncherError> {"], "hunk3 added line unexpected")

# 插入的 7 行结构（2 注释 + #[path] + mod + 空行 + pub use + 空行）
check(sum(1 for l in add1 if l.startswith("//")) == 2, "inserted comments != 2")
check(add1[2] == '#[path = "cast_mesh.rs"]', "inserted #[path] line unexpected")
check(add1[3] == "mod cast_mesh;", "inserted mod line unexpected")
check(add1[4] == "", "inserted line 5 not blank")
check(add1[5] == "pub use cast_mesh::MeshDevice;", "inserted pub use line unexpected")
check(add1[6] == "", "inserted line 7 not blank")

# hunk2 前 89 行 = 迁出块；第 90 行为空行
block_from_diff = ("\n".join(rem2[:BLOCK_LINES])).encode() + b"\n"
rep["cast_rs_changes"]["hunk2_block_sha256"] = sha(block_from_diff)
rep["cast_rs_changes"]["hunk2_block_matches_old_block"] = sha(block_from_diff) == sha(old_block)
rep["cast_rs_changes"]["hunk2_90th_line"] = rem2[BLOCK_LINES]
check(rep["cast_rs_changes"]["hunk2_block_matches_old_block"], "hunk2 removed block differs from pre-split block")
check(rem2[BLOCK_LINES] == "", "hunk2 90th removed line is not blank")

# 全量重构：旧文件经上述变换后逐字节等于新文件
#   ol[0:16]   = 旧 L1-L16（doc + 前 3 条 import）
#   add0       = 收窄后的 bvh import（替换旧 L17）
#   ol[17:22]  = 旧 L18-L22（其余 import + 空行）
#   add1       = 插入的 7 行（旧 L22 之后）
#   ol[22:283] = 旧 L23-L283（PoseError/CameraPose/ModelRenderArrays/SceneDevice，块前）
#   ol[373:]   = 旧 L374 起（块 + 其后 1 空行之后）
reconstructed = ol[0:16] + add0 + ol[17:22] + add1 + ol[22:283] + ol[373:]
# 再应用 hunk3（`to_i32` 可见性：`fn` → `pub(super) fn`，函数体不变）
check(rem3[0] in reconstructed, "to_i32 pre-split line not found in reconstruction")
reconstructed[reconstructed.index(rem3[0])] = add3[0]
recon_bytes = "\n".join(reconstructed).encode()
rep["reconstruction"] = {
    "method": "old[0:16] + narrowed_import + old[17:22] + inserted_7 + old[373:]",
    "byte_identical_to_new_file": recon_bytes == new_cast,
    "bytes": len(recon_bytes),
    "new_bytes": len(new_cast),
}
check(rep["reconstruction"]["byte_identical_to_new_file"], "reconstructed cast.rs != HEAD cast.rs")

# ---------------------------------------------------------------- 3. PTX / source 零变化
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
    h.update(head(rel))
    cuda_same = cuda_same and sha(head(rel)) == sha(blob(PRE, rel))
rep["kernel_source"] = {
    "concat_files": chain,
    "concat_sha256": h.hexdigest(),
    "baseline_source_sha256": BASELINE_SOURCE_SHA,
    "equals_baseline": h.hexdigest() == BASELINE_SOURCE_SHA,
    "cuda_files_byte_identical_to_pre_split": cuda_same,
}
check(rep["kernel_source"]["equals_baseline"], "concat source sha != baseline")
check(cuda_same, "cuda sources changed")

# ---------------------------------------------------------------- 4. API / 对外路径
cast_text = new_cast.decode()
mesh_text = new_mesh.decode()
rep["api"] = {
    "pub_use_meshdevice_count": cast_text.count("pub use cast_mesh::MeshDevice;"),
    "path_decl_count": cast_text.count('#[path = "cast_mesh.rs"]'),
    "mod_decl_count": cast_text.count("mod cast_mesh;"),
    "mod_rs_blob_equal": sha(head("src/render/kernel/mod.rs")) == sha(blob(PRE, "src/render/kernel/mod.rs")),
    "meshdevice_struct_in_cast_rs": "pub struct MeshDevice" in cast_text,
}
check(rep["api"]["pub_use_meshdevice_count"] == 1, "#[path]/pub use MeshDevice count != 1")
check(rep["api"]["path_decl_count"] == 1, "#[path] decl count != 1")
check(rep["api"]["mod_decl_count"] == 1, "mod cast_mesh count != 1")
check(rep["api"]["mod_rs_blob_equal"], "kernel/mod.rs changed")
check(not rep["api"]["meshdevice_struct_in_cast_rs"], "MeshDevice still defined in cast.rs")
check("cast_mesh" not in head("src/render/kernel/mod.rs").decode(), "kernel/mod.rs mentions cast_mesh")

# 外部引用 MeshDevice 的文件零改动
refs = set()
for p in (R / "src").rglob("*.rs"):
    if "MeshDevice" in p.read_text(encoding="utf-8", errors="replace"):
        refs.add(p.relative_to(R).as_posix())
refs_same = {}
for rel in sorted(refs - {"src/render/kernel/cast.rs", "src/render/kernel/cast_mesh.rs"}):
    same = sha(head(rel)) == sha(blob(PRE, rel))
    refs_same[rel] = "unchanged" if same else "CHANGED"
    check(same, f"external MeshDevice user changed: {rel}")
rep["api"]["meshdevice_ref_files"] = sorted(refs)
rep["api"]["external_ref_files"] = refs_same

# 公开项集合：旧单文件 == 新两文件并集（类型/函数名）
def pub_items(text):
    return sorted(re.findall(r"^pub (?:struct|enum|fn) ([A-Za-z_][A-Za-z0-9_]*)", text, re.M))


old_pub = pub_items("\n".join(ol))
new_pub = sorted(pub_items(cast_text) + pub_items(mesh_text))
rep["api"]["public_items_pre_split"] = old_pub
rep["api"]["public_items_head_union"] = new_pub
check(old_pub == new_pub, f"public item set changed: {sorted(set(old_pub) ^ set(new_pub))}")

# to_i32：仍在 cast.rs、pub(super)、函数体逐字节不变、调用点分布
def fn_body(text, sig):
    i = text.index(sig)
    return text[i:].split("\n}\n", 1)[0].split(sig, 1)[1]

body_old = fn_body("\n".join(ol), "fn to_i32(value: usize")
body_new = fn_body(cast_text, "fn to_i32(value: usize")
rep["to_i32"] = {
    "in_cast_rs": "fn to_i32(" in cast_text,
    "visibility_pub_super": "pub(super) fn to_i32(" in cast_text,
    "body_identical": body_old == body_new,
    "call_sites_cast_rs": cast_text.count("to_i32("),      # 含定义处 1 次
    "call_sites_cast_mesh_rs": mesh_text.count("to_i32("),  # 新文件无定义，全部为调用
    "cast_mesh_imports_super": "use super::to_i32;" in mesh_text,
}
check(rep["to_i32"]["in_cast_rs"], "to_i32 missing from cast.rs")
check(rep["to_i32"]["visibility_pub_super"], "to_i32 not pub(super)")
check(rep["to_i32"]["body_identical"], "to_i32 body changed")
check(rep["to_i32"]["cast_mesh_imports_super"], "cast_mesh not importing super::to_i32")
check(rep["to_i32"]["call_sites_cast_rs"] > 0 and rep["to_i32"]["call_sites_cast_mesh_rs"] > 0,
      "to_i32 call sites missing")

# 变更路径只应有 cast.rs + cast_mesh.rs + 证据 + .normify
touched = [t for t in subprocess.run(
    ["git", "diff", "--name-only", f"{PRE}..HEAD"], cwd=R, capture_output=True, text=True, check=True,
).stdout.split("\n") if t.strip()]
non_evidence = [t for t in touched if not t.startswith(IMPL_EV) and not t.startswith(".normify/")]
rep["changed_source_paths"] = non_evidence
rep["all_changed_paths"] = touched
check(non_evidence == ["src/render/kernel/cast.rs", "src/render/kernel/cast_mesh.rs"],
      f"unexpected changed source paths: {non_evidence}")

# ---------------------------------------------------------------- 5. 行数 / 红线 / 模块
line_claims = {
    "src/render/kernel/cast.rs": 411,
    "src/render/kernel/cast_mesh.rs": 102,
    "src/render/kernel/mod.rs": 474,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "src/runtime/launcher/render/args.rs": 494,
    "src/render/kernel/megakernel.rs": 406,
    "src/render/kernel/megakernel_uploads.rs": 384,
    "src/render/kernel/megakernel_arrays.rs": 326,
}
rep["lines"] = {}
for rel, want in line_claims.items():
    got = lf_count(head(rel))
    rep["lines"][rel] = {"lines": got, "claim": want, "match": got == want}
    check(got == want, f"line count {rel}: {got} != {want}")
rep["lines"]["cast_rs_before"] = lf_count(old_cast)
rep["lines_limits"] = {
    "cast_rs_le_430": rep["lines"]["src/render/kernel/cast.rs"]["lines"] <= 430,
    "new_file_le_500": rep["lines"]["src/render/kernel/cast_mesh.rs"]["lines"] <= 500,
    "net_delta_arithmetic": {
        "before": 494,
        "minus_block": BLOCK_LINES,
        "minus_blank_after_block": 1,
        "plus_inserted": 7,
        "expected_after": 494 - BLOCK_LINES - 1 + 7,
        "actual_after": lf_count(new_cast),
        "consistent": 494 - BLOCK_LINES - 1 + 7 == lf_count(new_cast),
    },
}
check(rep["lines_limits"]["net_delta_arithmetic"]["consistent"], "line delta arithmetic inconsistent")

unchanged_files = [
    "src/render/kernel/cuda/render.cu", "src/render/kernel/launch.rs", "src/model/render.rs",
    "src/render/context/camera_output.rs", "src/runtime/launcher/render/args.rs",
    "src/render/kernel/megakernel.rs", "src/render/kernel/megakernel_uploads.rs",
    "src/render/kernel/megakernel_arrays.rs", "src/render/kernel/mod.rs",
    "tests/render_kernel_rgb.rs", "tests/render_lighting_kernel.rs", "tests/render_launch_abi.rs",
    "Cargo.toml", "Cargo.lock",
]
rep["unchanged_files"] = {}
for rel in unchanged_files:
    same = sha(head(rel)) == sha(blob(PRE, rel))
    rep["unchanged_files"][rel] = "unchanged" if same else "CHANGED"
    check(same, f"changed: {rel}")

# 未抽 SceneDevice（未批的 L182-292 备选）：仍完整在 cast.rs（重构等价已覆盖）
rep["scene_device"] = {
    "struct_in_cast_rs": "pub struct SceneDevice" in cast_text,
    "impl_in_cast_rs": "impl SceneDevice" in cast_text,
    "hunk_count_whole_file": len(hunks),
}
check(rep["scene_device"]["struct_in_cast_rs"] and rep["scene_device"]["impl_in_cast_rs"],
      "SceneDevice not intact in cast.rs")

# 模块源登记
mod_md = (R / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
kernel_files = sorted(p.name for p in (R / "src/render/kernel").glob("*.rs"))
registered = {f: ('"src/render/kernel/' + f + '"') in mod_md for f in kernel_files}
rep["module_sources"] = {
    "total_entries": mod_md.count("[[source]]"),
    "cast_mesh_registered": '"src/render/kernel/cast_mesh.rs"' in mod_md,
    "kernel_files_registered": registered,
    "kernel_files_unregistered": sorted(f for f, ok in registered.items() if not ok),
}
check(rep["module_sources"]["cast_mesh_registered"], "cast_mesh.rs not registered")
check(rep["module_sources"]["total_entries"] == 38, f"module sources {rep['module_sources']['total_entries']} != 38")

validate = subprocess.run(["normify", "validate", "--fingerprints"], cwd=R, capture_output=True)
vtext = (validate.stdout + validate.stderr).decode("utf-8", errors="replace")
vm = re.search(r"错误(\d+)", vtext)
rep["validate"] = {"exit": validate.returncode, "errors": int(vm.group(1)) if vm else None}
check(rep["validate"]["errors"] == 0, "validate errors != 0")

# ---------------------------------------------------------------- 6. 证据完整性（实现片）
ev = R / IMPL_EV
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
    manifest_problems.append(f"set mismatch: {sorted(ev_rel ^ set(manifest))}")
for rel in sorted(ev_rel & set(manifest)):
    b = head(f"{IMPL_EV}/{rel}")
    if (sha(b), len(b)) != manifest[rel]:
        manifest_problems.append(f"{rel}: manifest {manifest[rel]} blob {(sha(b), len(b))}")
enc = []
for p in ev_files:
    b = p.read_bytes()
    if b.startswith(b"\xef\xbb\xbf"):
        enc.append((p.name, "BOM"))
    if b"\r" in b:
        enc.append((p.relative_to(ev).as_posix(), "CR"))
word_hits = []
for p in ev_files:
    t = p.read_text(encoding="utf-8", errors="replace")
    word_hits += [(p.relative_to(ev).as_posix(), w) for w in BANNED if w in t]
readme = (ev / "README.md").read_text(encoding="utf-8")
receipt = json.loads((ev / "receipt.json").read_text(encoding="utf-8"))
plan_markers = {
    "mentions_block_range_L284_372": "L284-372" in readme,
    "mentions_to_i32_kept": "to_i32" in readme,
    "mentions_411_and_102": ("411" in readme and "102" in readme),
    "mentions_unapproved_L182_292": "L182-292" in readme,
    "mentions_budget_delta": "预算" in readme,
    "mentions_insert_9_lines": "+9 行" in readme,
}
rep["implementation_evidence"] = {
    "files": len(ev_files),
    "manifest_entries": len(manifest),
    "manifest_self_reference": "manifest.txt" in manifest,
    "manifest_problems": manifest_problems,
    "encoding_problems": enc,
    "forbidden_word_hits": word_hits,
    "readme_lines": lf_count(readme.encode()),
    "receipt_task_id": receipt.get("task_id"),
    "plan_vs_actual_markers": plan_markers,
    "insert_lines_claim_vs_actual": {"claim": 9, "actual": 7},
}
check(not manifest_problems, f"manifest problems: {manifest_problems}")
check(not enc, f"encoding problems: {enc}")
check(not word_hits, f"forbidden words: {word_hits}")
check(receipt.get("task_id") == "2026-10-28-render-bvh-r05-split-cast", "receipt task_id mismatch")
check(all(v for k, v in plan_markers.items() if k != "mentions_insert_9_lines"),
      f"plan_vs_actual markers missing: {plan_markers}")

# ---------------------------------------------------------------- 7. 提交范围
commit_files = {}
for c in ["72a7de66", "3e8628b6"]:
    out = subprocess.run(["git", "show", "--name-only", "--pretty=format:", c],
                         cwd=R, capture_output=True, text=True, check=True).stdout
    commit_files[c] = [x for x in out.split("\n") if x.strip()]
allowed = ("src/render/kernel/cast.rs", "src/render/kernel/cast_mesh.rs", IMPL_EV + "/", ".normify/")
outside = [p for f in commit_files.values() for p in f if not p.startswith(allowed)]
rep["commits"] = {c: {"files": len(f), "paths": f} for c, f in commit_files.items()}
rep["commits"]["outside_card_scope"] = outside
check(not outside, f"files outside card scope: {outside}")

# ---------------------------------------------------------------- 8. 门禁日志（QA 自己跑）
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
        ignored = sum(int(re.search(r"(\d+) ignored", l).group(1)) for l in res)
        gates[name] = {"result_lines": len(res), "passed": passed, "failed": failed, "ignored": ignored}
        check(passed == 1069 and failed == 0, f"unit gate {passed}/{failed}")
    elif name == "kernel-gate":
        m2 = re.search(r"source_sha256=(\w+) ptx_sha256=(\w+) ptx_bytes=(\d+)", t)
        gates[name] = {
            "source_sha256": m2.group(1) if m2 else None,
            "ptx_sha256": m2.group(2) if m2 else None,
            "ptx_bytes": int(m2.group(3)) if m2 else None,
            "matches_baseline": bool(m2) and m2.group(1) == BASELINE_SOURCE_SHA
            and m2.group(2) == BASELINE_PTX_SHA and int(m2.group(3)) == BASELINE_PTX_BYTES,
            "exit_marker": re.search(r"KERNEL_GATE_EXIT=(\d+)", t).group(1) if "KERNEL_GATE_EXIT=" in t else None,
        }
        check(gates[name]["matches_baseline"], "kernel gate sha != baseline")
    else:
        em = re.search(r"^(FMT|CLIPPY|CLIPPY_CUDA)_EXIT=(\d+)", t, re.M)
        gates[name] = {"exit": em.group(2) if em else None}
        check(bool(em) and em.group(2) == "0", f"gate {name} exit != 0")
rep["gates"] = gates

rep["problems"] = problems
out = pathlib.Path(__file__).resolve().parent / "recompute.json"
out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(rep, ensure_ascii=False, indent=2))
print("RESULT problems=" + str(len(problems)))
for item in problems:
    print("PROBLEM " + str(item))
