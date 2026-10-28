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
    subprocess.run(["git", "rev-parse", "--show-toplevel"],
                   cwd=pathlib.Path(__file__).resolve().parent,
                   capture_output=True, text=True, check=True).stdout.strip()
)
PRE = "59d1b2a4"          # S2 前树（拆分② O1 修正后）
IMPL = "docs/development-evidence/render-bvh/r05-s2-hfield-intersect"
CHECK_EV = "docs/development-evidence/render-bvh/r05-s2-hfield-intersect-check"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()

problems = []


def blob(rev, rel):
    return subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=R, capture_output=True, check=True).stdout


def head(rel):
    return blob("HEAD", rel)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lf(b):
    return b.count(b"\n") + (0 if b.endswith(b"\n") else 1)


def check(cond, msg):
    if not cond:
        problems.append(msg)


rep = {"head": HEAD, "pre_split": PRE}

# 1. 行数
claims = {
    "src/render/kernel/megakernel.rs": 404,
    "src/render/kernel/megakernel_uploads.rs": 405,
    "src/render/kernel/cast.rs": 411,
    "src/render/kernel/cast_mesh.rs": 223,
    "src/render/bvh/hfield_mesh.rs": 266,
    "src/render/bvh/aabb.rs": 457,
    "src/render/bvh/mod.rs": 278,
    "src/render/context.rs": 453,
    "src/render/context/hfield.rs": 449,
    "src/render/context/owner.rs": 415,
    "src/render/mod.rs": 385,
    "src/render/kernel/cuda/render_common.cu": 410,
    "src/render/kernel/cuda/render_shade.cu": 383,
    "src/render/kernel/cuda/render_megakernel.cu": 347,
    "src/render/kernel/cuda/render_shadow.cu": 32,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "tests/render_hfield_mesh.rs": 240,
}
rep["lines"] = {}
for rel, want in claims.items():
    got = lf(head(rel))
    rep["lines"][rel] = {"lines": got, "claim": want, "match": got == want}
    check(got == want, f"line count {rel}: {got} != {want}")

# 2. 红线未改（相对 PRE 的 blob）
unchanged = [
    "src/render/kernel/cuda/render.cu", "src/render/kernel/launch.rs",
    "src/model/render.rs", "src/render/context/camera_output.rs",
    "src/runtime/launcher/render/args.rs",  # 81 槽表
]
rep["unchanged_redlines"] = {}
for rel in unchanged:
    same = sha(head(rel)) == sha(blob(PRE, rel))
    rep["unchanged_redlines"][rel] = "unchanged" if same else "CHANGED"
    check(same, f"redline changed: {rel}")

# 3. 槽表：MEGAKERNEL_SLOTS[64] 类型 + 81 槽
args_text = head("src/runtime/launcher/render/args.rs").decode()
rep["slots"] = {
    "args_rs_unchanged": sha(head("src/runtime/launcher/render/args.rs")) == sha(blob(PRE, "src/runtime/launcher/render/args.rs")),
}
tokens = re.findall(r"RenderArgKind::(\w+)", args_text)
rep["slots"]["kinds_found"] = len(tokens)
# 直接从源码数 MEGAKERNEL_SLOTS 的 81 项：由测试断言与 args.rs 未改共同保证；此处记录 args.rs 未变
check(rep["slots"]["args_rs_unchanged"], "args.rs changed (slot table)")

# 4. 新 .cu 数量：变更集中无新增 .cu
changed = [p for p in subprocess.run(["git", "diff", "--name-only", f"{PRE}..HEAD"],
                                     cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
cu = [p for p in changed if p.endswith(".cu")]
rep["changed_cu"] = cu
check("src/render/kernel/cuda/render.cu" not in changed, "render.cu changed")
new_cu = subprocess.run(["git", "diff", "--name-status", f"{PRE}..HEAD"], cwd=R,
                        capture_output=True, text=True, check=True).stdout
rep["new_files"] = [line.split("\t")[1] for line in new_cu.split("\n") if line.startswith("A\t")]
check(not any(p.endswith(".cu") for p in rep["new_files"]), "new .cu added")

# 5. 设备分派与槽位语义（源码静态断言）
common = head("src/render/kernel/cuda/render_common.cu").decode()
mg = head("src/render/kernel/cuda/render_megakernel.cu").decode()
shade = head("src/render/kernel/cuda/render_shade.cu").decode()
shadow = head("src/render/kernel/cuda/render_shadow.cu").decode()
kernel_text = mg
rep["device"] = {
    "kernel_param_type": "wp::array_t<wp::uint64> hfield_bvh_id" in kernel_text,
    "gtype1_dispatches_mesh": "gtype == 7 || gtype == 1" in common,
    "sentinel_skip": "0xFFFFFFFFFFFFFFFFull" in common,
    "gtype8_not_intersected": "gtype != 8" in common,
    "shadow_no_backface_for_1_7": "shadow_ray && (gtype == 7 || gtype == 1)" in common,
    "vertex_normal_gtype_gate": "vertex_normal_gtype == 7 || hit_geom_type == 8" in kernel_text or
                                "(hit_geom_type == 7 || hit_geom_type == 8)" in kernel_text,
    "texture_hfield_present": "texture_hfield(" in shade and "gtype == 1" in shade,
    "shadow_passes_both_params": "geom_dataid" in shadow and "hfield_bvh_id" in shadow,
    "sample_texture_uses_geom_size": "(const float*)(geom_size.data + geom_id)" in kernel_text,
}
for key in ("kernel_param_type", "gtype1_dispatches_mesh", "sentinel_skip", "gtype8_not_intersected",
            "shadow_no_backface_for_1_7", "vertex_normal_gtype_gate", "texture_hfield_present",
            "shadow_passes_both_params", "sample_texture_uses_geom_size"):
    check(rep["device"][key], f"device check failed: {key}")

# 6. host 打包：u32 低/高字对 + 长度/sentinel
uploads = head("src/render/kernel/megakernel_uploads.rs").decode()
castm = head("src/render/kernel/cast_mesh.rs").decode()
rep["host_pack"] = {
    "low_word_first": "hfield_words.push(*id as u32);" in uploads,
    "high_word": "hfield_words.push((*id >> 32) as u32);" in uploads,
    "empty_uses_one_sentinel": "&[u64::MAX]" in uploads,
    "count_recorded": "hfield_bvh_count = hfield_ids.len()" in uploads,
    "sentinel_unused_slots": "vec![u64::MAX; nhfield]" in castm,
    "blob_index_append": "mesh_count.checked_add(order)" in castm or "mesh_count\n                .checked_add(order)" in castm,
    "used_hfield_only": "if *kind != 1" in castm,
    "requires_injection": "set_hfield" in castm,
}
for key, val in rep["host_pack"].items():
    check(val, f"host pack check failed: {key}")

# 7. 注入 None 路径与 API 零改动
ctx = head("src/render/context.rs").decode()
rep["injection"] = {
    "set_hfield_builder": "pub fn set_hfield(" in ctx,
    "hfield_field_default_none": "hfield: None," in ctx,
    "resolve_context_unchanged": "resolve_context(" in ctx,
    "hfield_input_owned": "pub struct HfieldContextInput" in head("src/render/context/hfield.rs").decode(),
}
check(rep["injection"]["set_hfield_builder"], "set_hfield missing")
check(rep["injection"]["hfield_field_default_none"], "default None missing")

# 8. 三角化阈值口径：f32(1e-5) < 1e-5（说明 f32 阈值与 f64 阈值在边界值上不同）
import struct
f32_thr = struct.unpack("<f", struct.pack("<f", 1.0e-5))[0]
rep["threshold"] = {
    "f32_of_1e-5": f32_thr,
    "f32_below_f64": f32_thr < 1.0e-5,
    "f32_bits": struct.unpack("<I", struct.pack("<f", 1.0e-5))[0],
    "implementation_uses_f64_threshold": "1.0e-5f64" in head("src/render/bvh/hfield_mesh.rs").decode(),
    "upstream_compares_f32": "numpy 2.5.4 NEP 50: np.float32 < python float → f32 比较",
}
check(rep["threshold"]["f32_below_f64"], "f32(1e-5) >= 1e-5 (unexpected)")
check(rep["threshold"]["implementation_uses_f64_threshold"], "implementation threshold not f64")

# 9. 证据完整性（实现片）
ev = R / IMPL
ev_files = sorted(p for p in ev.rglob("*") if p.is_file())
manifest = {}
for line in (ev / "manifest.txt").read_text(encoding="utf-8").split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h_, n_, rel = line.split(" ", 2)
        manifest[rel] = (h_, int(n_))
ev_rel = {p.relative_to(ev).as_posix() for p in ev_files if p.name != "manifest.txt"}
man_problems = []
if ev_rel != set(manifest):
    man_problems.append(f"set mismatch {sorted(ev_rel ^ set(manifest))}")
for rel in sorted(ev_rel & set(manifest)):
    b = head(f"{IMPL}/{rel}")
    if (sha(b), len(b)) != manifest[rel]:
        man_problems.append(f"{rel}: manifest {manifest[rel]} blob {(sha(b), len(b))}")
enc = []
for p in ev_files:
    b = p.read_bytes()
    if b.startswith(b"\xef\xbb\xbf"):
        enc.append((p.name, "BOM"))
    if b"\r" in b:
        enc.append((p.relative_to(ev).as_posix(), "CR"))
readme = (ev / "README.md").read_text(encoding="utf-8")
receipt = json.loads((ev / "receipt.json").read_text(encoding="utf-8"))
BANNED = ["".join(chr(c) for c in seq) for seq in (
    (71, 49, 48, 24050, 23454, 29616), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (21407, 29983, 25918, 34892), (31934, 24230, 36798, 26631),
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
)]
word_hits = []
for p in ev_files:
    t = p.read_text(encoding="utf-8", errors="replace")
    word_hits += [(p.relative_to(ev).as_posix(), w) for w in BANNED if w in t]
markers = {
    "section_rebase": "rebase" in readme or "拆分①实际交付" in readme or "行数口径" in readme,
    "section_align": "bvh.py" in readme and "L472" in readme,
    "section_bvh_policy": "LEAF_CAPACITY" in readme or "leaf_size" in readme,
    "section_slot_abi": "MEGAKERNEL_SLOTS" in readme and "u64::MAX" in readme,
    "section_injection": "set_hfield" in readme,
    "section_device": "render_common.cu" in readme,
    "section_old_assertions": "render_context_lifecycle" in readme and "render_scene_bvh_host" in readme,
    "zero_numeric_conclusion": "零渲染数值结论" in readme,
    "receipt_task_r2": receipt.get("task_id") == "2026-10-28-render-bvh-r05-s2-hfield-intersect-r2",
    "receipt_reregistered": "re_registered_from" in receipt,
}
rep["impl_evidence"] = {
    "files": len(ev_files),
    "manifest_entries": len(manifest),
    "manifest_problems": man_problems,
    "manifest_self_reference": "manifest.txt" in manifest,
    "encoding_problems": enc,
    "forbidden_word_hits": word_hits,
    "readme_lines": lf(readme.encode()),
    "markers": markers,
}
check(not man_problems, f"manifest problems: {man_problems}")
check(not enc, f"encoding problems: {enc}")
check(not word_hits, f"forbidden words: {word_hits}")
check(markers["receipt_task_r2"], "receipt task_id != r2")
check(markers["receipt_reregistered"], "receipt missing re_registered_from")
for k, v in markers.items():
    check(v, f"README/receipt marker missing: {k}")

# 10. 提交范围
commit_files = {}
for c in ["0656bd6e", "c84c1760"]:
    out = subprocess.run(["git", "show", "--name-only", "--pretty=format:", c],
                         cwd=R, capture_output=True, text=True, check=True).stdout
    commit_files[c] = [x for x in out.split("\n") if x.strip()]
allowed = ("src/render/", "tests/", IMPL + "/", ".normify/")
outside = [p for f in commit_files.values() for p in f if not p.startswith(allowed)]
rep["commits"] = {c: {"files": len(f), "paths": f} for c, f in commit_files.items()}
rep["commits"]["outside_card_scope"] = outside
check(not outside, f"files outside card scope: {outside}")

# 11. 模块源与指纹
mod_md = (R / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
rep["module_sources"] = {
    "entries": mod_md.count("[[source]]"),
    "hfield_mesh_registered": '"src/render/bvh/hfield_mesh.rs"' in mod_md,
}
check(rep["module_sources"]["entries"] == 39, f"module sources {rep['module_sources']['entries']} != 39")
check(rep["module_sources"]["hfield_mesh_registered"], "hfield_mesh.rs not registered")
validate = subprocess.run(["normify", "validate", "--fingerprints"], cwd=R, capture_output=True)
vtext = (validate.stdout + validate.stderr).decode("utf-8", "replace")
vm = re.search(r"错误(\d+)", vtext)
rep["validate"] = {"exit": validate.returncode, "errors": int(vm.group(1)) if vm else None}
check(rep["validate"]["errors"] == 0, "validate errors != 0")

# 12. 门禁日志（QA 自己跑）
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
        check(passed == 1082 and failed == 0, f"unit gate {passed}/{failed}")
    elif name == "kernel-gate":
        m2 = re.search(r"source_sha256=(\w+) ptx_sha256=(\w+) ptx_bytes=(\d+)", t)
        gates[name] = {
            "source_sha256": m2.group(1) if m2 else None,
            "ptx_sha256": m2.group(2) if m2 else None,
            "ptx_bytes": int(m2.group(3)) if m2 else None,
            "expected": {
                "source_sha256": "72bc364c4b732e278396d2aebd09625872f6dcecba469a98a3e40aba04851746",
                "ptx_sha256": "97467d835e27107a4a0803678e194f4868fb552707543a544e270cbde8a9bd13",
                "ptx_bytes": 264015,
            },
            "exit_marker": re.search(r"KERNEL_GATE_EXIT=(\d+)", t).group(1) if "KERNEL_GATE_EXIT=" in t else None,
        }
        exp = gates[name]["expected"]
        ok = (m2 and m2.group(1) == exp["source_sha256"] and m2.group(2) == exp["ptx_sha256"]
              and int(m2.group(3)) == exp["ptx_bytes"])
        gates[name]["matches_expected"] = bool(ok)
        check(ok, "kernel gate sha/bytes != expected")
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
