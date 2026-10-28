"""R05-S1 hfield 静态参数独立复核（QA；只读 + 静态复算）。

复核对象：`2026-10-28-render-bvh-r05-s1-hfield-params`（实现 `c2a098aa` / 收口 `50f088c4`，判据树 HEAD `4723fd44`）。
本脚本独立重算，不引用 Backend/Arch 数字：

  1. 实现证据完整性：7 文件 / manifest 6 条对 committed blob、不自引用、LF 无 BOM、禁词 0；
  2. 行数与红线（hfield.rs 332、tests 284、params 193、mod 384、红线 499/500/494/500/499、megakernel 490）；
  3. 裁决 A 证据：`RENDER_FIELDS` 116 项无 hfield、`model/render.rs` 500 行、`io/render.rs` 无 hfield、
     `RenderFields::new` 仅包装 Vec、`field()` 只按 `RENDER_FIELDS` 查名；
  4. 语义面静态核对：`HfieldError` 变体/Display、无 panic 风险点（用户数据无 unwrap、长度检查先于索引）、
     `resolve_hfield` 无 `nworld` 参数、无 per-world 复制循环；
  5. ABI/红线：81 槽、`hfield_bvh_id` 占位 `[0u32, 0]`、红线文件自判据树零改动；
  6. 门禁：四门禁记录 exit 0、unit 1060 过 / 0 败 / 387 ignored、10 条实现单测在 test 日志中通过；
  7. 我的独立对抗性探针日志（临时测试，运行后删除）：7 条攻击用例全过 + 外部构造非法 HfieldParams 的 panic 观察。
"""
import hashlib
import json
import pathlib
import re
import subprocess

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/render-bvh/r05-s1-hfield-params-check"
IMPL = REPO / "docs/development-evidence/render-bvh/r05-s1-hfield-params"
JUDGED = "4723fd4402572d14f03f77bd64ac6331a0867447"

result = {"task_id": "2026-10-28-render-bvh-r05-s1-hfield-params-check", "judged_head": JUDGED, "checks": {}, "problems": []}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def note(k, v):
    result["checks"][k] = v
    print(k, "=", json.dumps(v, ensure_ascii=False) if not isinstance(v, str) else v)


def fail(m):
    result["problems"].append(m)
    print("[FAIL]", m)


def blob(rel):
    return subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=REPO, capture_output=True).stdout


def git(*args):
    return subprocess.run(["git", *args], cwd=REPO, capture_output=True, text=True, encoding="utf-8").stdout


# 1. 实现证据完整性
files = sorted(p for p in IMPL.rglob("*") if p.is_file())
note("impl_evidence_files", len(files))
if len(files) != 7:
    fail(f"实现证据文件数应为 7，实测 {len(files)}")
man = []
for line in (IMPL / "manifest.txt").read_text(encoding="utf-8").splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    h, size, rel = line.split()
    b = blob("docs/development-evidence/render-bvh/r05-s1-hfield-params/" + rel)
    ok = sha(b) == h and len(b) == int(size)
    man.append({"path": rel, "bytes": len(b), "claimed": int(size), "ok": ok})
    if not ok:
        fail(f"实现证据 manifest 不符：{rel}")
note("impl_manifest", {"entries": len(man), "all_ok": all(m["ok"] for m in man)})
if len(man) != 6:
    fail(f"实现证据 manifest 条目数应为 6，实测 {len(man)}")
if any(m["path"] == "manifest.txt" for m in man):
    fail("实现证据 manifest 自引用")
eol_bad = []
for p in files:
    b = p.read_bytes()
    if b.count(b"\r") or b.startswith(b"\xef\xbb\xbf"):
        eol_bad.append(p.relative_to(IMPL).as_posix())
note("impl_evidence_eol_bad", eol_bad)
if eol_bad:
    fail(f"实现证据 EOL/BOM 异常：{eol_bad}")

banned = ["".join(chr(c) for c in s) for s in (
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (31934, 24230, 36798, 26631), (21407, 29983, 25918, 34892), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807),
)]
hits = []
for p in files:
    t = p.read_text(encoding="utf-8", errors="replace")
    for w in banned:
        if w in t:
            hits.append({"path": p.relative_to(IMPL).as_posix(), "term": w})
note("impl_banned_hits", hits)
for h in hits:
    fail(f"实现证据禁词命中 {h['term']} @ {h['path']}")

# 2. 行数与红线
lines = {}
for rel, claim in {
    "src/render/context/hfield.rs": 332,
    "tests/render_hfield_params.rs": 284,
    "src/render/context/params.rs": 193,
    "src/render/mod.rs": 384,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "src/render/kernel/megakernel.rs": 490,
}.items():
    n = (REPO / rel).read_bytes().count(b"\n")
    lines[rel] = {"lf": n, "claimed": claim, "match": n == claim, "le_500": n <= 500}
    if n != claim:
        fail(f"行数不符：{rel} 实测 {n} 声称 {claim}")
note("line_counts", lines)

# 3. 裁决 A 证据
model = (REPO / "src/model/render.rs").read_text(encoding="utf-8")
spec = re.search(r"pub const RENDER_FIELDS: \[FieldSpec; (\d+)\]", model)
i = model.index("RENDER_FIELDS: [FieldSpec;")
block = model[i:]
entries = re.search(r"RENDER_FIELDS: \[FieldSpec; \d+\] = \[(.*?)\n\];", block, re.S)
ruling_a = {
    "render_fields_len": int(spec.group(1)) if spec else None,
    "render_fields_entries": len(re.findall(r"\bf\(", entries.group(1))) if entries else None,
    "model_render_has_hfield": "hfield" in model,
    "io_render_has_hfield": "hfield" in (REPO / "src/io/render.rs").read_text(encoding="utf-8"),
    "new_wraps_vec": "pub fn new(data: Vec<FieldData>) -> Self {\n        Self { data }\n    }" in model,
    "field_looks_up_render_fields": "Self::index_of(name).and_then(|id| self.data.get(id))" in model,
    "index_of_uses_render_fields": "RENDER_FIELDS.iter().position(|spec| spec.name == name)" in model,
}
note("ruling_a_evidence", ruling_a)
if ruling_a["render_fields_len"] != 116 or ruling_a["render_fields_entries"] != 116:
    fail(f"RENDER_FIELDS 不是 116 项：{ruling_a}")
if ruling_a["model_render_has_hfield"] or ruling_a["io_render_has_hfield"]:
    fail("model/render.rs 或 io/render.rs 出现 hfield（与裁决 A 前提不符）")
if not (ruling_a["new_wraps_vec"] and ruling_a["field_looks_up_render_fields"] and ruling_a["index_of_uses_render_fields"]):
    fail(f"RenderFields 注入/查名事实与证据不符：{ruling_a}")

# 4. 语义面静态核对
hfield = (REPO / "src/render/context/hfield.rs").read_text(encoding="utf-8")
code_lines = [l for l in hfield.splitlines() if not l.strip().startswith("//")]
code = "\n".join(code_lines)
variants = re.findall(r"^\s{4}([A-Z][A-Za-z]+)\s*\{", hfield, re.M)
semantic = {
    "error_variants": variants,
    "display_arms": len(re.findall(r"Self::[A-Z][A-Za-z]+ \{", hfield)),
    "no_nworld_param": "nworld" not in re.search(r"pub fn resolve_hfield[^{]*", hfield).group(0),
    "resolve_signature": re.search(r"pub fn resolve_hfield<'a>\(\s*inputs: &HfieldInputs<'a>,?\s*\)", hfield) is not None,
    "no_unwrap_on_user_data": "unwrap()" not in code and "expect(" not in code,
    "checked_mul": "checked_mul" in code,
    "checked_add": "checked_add" in code,
    "try_reserve_exact": "try_reserve_exact" in code,
    "length_checks_before_index": code.index("ncol.len() != nhfield") < code.index("hfields.push("),
    "no_world_loop": "for world" not in code and "nworld" not in code,
}
fieldkind_constructed = len(re.findall(r"HfieldError::FieldKind\s*\{", hfield))
semantic["fieldkind_constructions"] = fieldkind_constructed
note("semantics_static", semantic)
for key in ("resolve_signature", "no_nworld_param", "no_unwrap_on_user_data", "checked_mul", "checked_add",
            "try_reserve_exact", "length_checks_before_index", "no_world_loop"):
    if not semantic[key]:
        fail(f"语义面静态核对失败：{key}")
if semantic["display_arms"] < len(variants):
    fail(f"Display 臂数少于变体数：{semantic['display_arms']} < {len(variants)}")

# 5. ABI / 红线
args = (REPO / "src/runtime/launcher/render/args.rs").read_text(encoding="utf-8")
m = re.search(r"pub static MEGAKERNEL_SLOTS: \[RenderArgKind; (\d+)\]", args)
mega = (REPO / "src/render/kernel/megakernel.rs").read_text(encoding="utf-8")
abi = {
    "megakernel_slots": int(m.group(1)) if m else None,
    "hfield_bvh_id_placeholder": "upload(&self.session, &[0u32, 0])" in mega,
    "hfield_bvh_id_slot_push": "args.push(array(&hfield_bvh_id, RenderElem::U64, &[1])" in mega,
    "device_sources": re.search(r"DEVICE_SOURCES: \[\(&str, &str\); (\d+)\]", (REPO / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8")).group(1),
    "collision_args_504": "size_of::<CollisionArgs>(), 504" in (REPO / "src/runtime/collision_gpu/abi_tests.rs").read_text(encoding="utf-8"),
}
note("abi", abi)
if abi["megakernel_slots"] != 81 or not abi["hfield_bvh_id_placeholder"] or abi["device_sources"] != "28" or not abi["collision_args_504"]:
    fail(f"ABI 核对失败：{abi}")

redline = git(
    "diff", "--numstat", f"{JUDGED}..HEAD", "--", "src/render/kernel/cuda/render.cu", "src/render/kernel/launch.rs",
    "src/render/kernel/cast.rs", "src/model/render.rs", "src/render/context/camera_output.rs", "fixtures/precision",
    "Cargo.toml", "Cargo.lock", "src/runtime", "src/render/kernel/*.cu",
).strip()
note("redline_diff_since_judged_head", redline)
if redline:
    fail(f"红线/夹具路径有改动：{redline}")
impl_scope = git("diff", "--numstat", "50f088c4..HEAD", "--", "src/render/context/hfield.rs",
                 "src/render/context/params.rs", "src/render/mod.rs", "tests/render_hfield_params.rs").strip()
uncommitted = git("status", "--porcelain", "--", "src/render/context/hfield.rs", "src/render/context/params.rs", "src/render/mod.rs", "tests/render_hfield_params.rs").strip()
note("impl_files_diff_since_closure", {"diff": impl_scope, "uncommitted": uncommitted})
if impl_scope or uncommitted:
    fail(f"实现文件在收口后被改动：{impl_scope!r} uncommitted={uncommitted!r}")

# 6. 门禁与单测记录
runs = REPO / ".normify/runs/2026-10-28-render-bvh-r05-s1-hfield-params"
gates = {}
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    p = runs / f"{g}.json"
    if p.exists():
        d = json.loads(p.read_text(encoding="utf-8"))
        gates[g] = {"exit": d.get("exit"), "at": d.get("at"), "passed": d.get("tests", {}).get("passed"),
                    "failed": d.get("tests", {}).get("failed"), "ignored": d.get("tests", {}).get("ignored")}
note("gate_records", gates)
if len(gates) != 4 or any(v["exit"] != 0 for v in gates.values()):
    fail(f"门禁记录不全或非 0 退出：{gates}")
if gates.get("unit", {}).get("passed") != 1060 or gates.get("unit", {}).get("failed") != 0:
    fail(f"实现片 unit 计数不符：{gates.get('unit')}")

test_log = (IMPL / "logs/test.log").read_text(encoding="utf-8", errors="replace")
names = ["accepts_two_valid_hfields", "no_hfield_is_legal_and_returns_none",
         "partial_fields_report_first_missing_field", "size_shape_mismatch_is_rejected",
         "row_col_adr_lengths_must_match", "adr_out_of_range_is_reported_with_index",
         "non_positive_rows_or_cols_are_rejected", "extreme_span_does_not_panic_and_reports_range",
         "empty_field_set_is_legal_and_returns_none", "resolution_is_single_copy_and_stable"]
tests = {"test_log_10_passed": "test result: ok. 10 passed; 0 failed" in test_log,
         "all_names_in_log": all(n in test_log for n in names)}
note("impl_tests", tests)
if not all(tests.values()):
    fail(f"实现单测日志不符：{tests}")

# 7. 我的独立对抗性探针日志
probe1 = (EVID / "logs/probe-attack.log").read_text(encoding="utf-8", errors="replace") if (EVID / "logs/probe-attack.log").exists() else ""
probe2 = (EVID / "logs/probe-construction.log").read_text(encoding="utf-8", errors="replace") if (EVID / "logs/probe-construction.log").exists() else ""
probes = {
    "attack_tests_passed": "test result: ok. 7 passed; 0 failed" in probe1,
    "attack_cases_in_log": all(n in probe1 for n in ("overlap_adr_sum_exceeds_data_len_is_length_mismatch",
                                                     "empty_with_inconsistent_lengths_reports_which_field",
                                                     "missing_field_priority_matrix",
                                                     "extra_data_tail_is_allowed_and_slices_are_bounded",
                                                     "exact_boundary_adr_plus_span_equals_len_is_ok",
                                                     "adversarial_inputs_never_panic",
                                                     "success_view_is_borrowed_single_copy")),
    "construction_probe_passed": "test result: ok. 2 passed; 0 failed" in probe2,
    "constructed_invalid_params_panics_in_debug": "panicked at src\\render\\context\\hfield.rs:119" in probe2 or "panicked at src/render/context/hfield.rs:119" in probe2,
    "constructed_empty_params_is_empty_true": "externally constructed empty params.is_empty() = true" in probe2,
}
note("my_probes", probes)
if not probes["attack_tests_passed"] or not probes["attack_cases_in_log"]:
    fail(f"我的攻击探针未全过：{probes}")
if not probes["construction_probe_passed"]:
    fail(f"构造探针未通过：{probes}")

# 8. 本复核证据自身
self_files = sorted(p for p in EVID.rglob("*") if p.is_file() and p.name != "recompute.json")
note("check_evidence_files", len(self_files))


# 9. 本复核证据 manifest（证据目录内按工作树；仓库内被复核文件按 committed blob）
man_path = EVID / "manifest.txt"
if man_path.exists():
    self_man = []
    for line in man_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        h, size, rel = line.split()
        p = EVID / rel
        if p.exists():
            b = p.read_bytes(); src = "worktree"
        else:
            b = blob(rel); src = "git_blob"
        ok = sha(b) == h and len(b) == int(size)
        self_man.append({"path": rel, "source": src, "bytes": len(b), "claimed": int(size), "ok": ok})
        if not ok:
            fail(f"本复核 manifest 不符：{rel}")
    note("check_manifest", {"entries": len(self_man), "all_ok": all(m["ok"] for m in self_man),
                              "self_reference": any(m["path"] == "manifest.txt" for m in self_man)})
    if any(m["path"] == "manifest.txt" for m in self_man):
        fail("本复核 manifest 自引用")
else:
    note("check_manifest", "absent (first run)")

note("problems", result["problems"])
print("problems =", len(result["problems"]))
EVID.mkdir(parents=True, exist_ok=True)
(EVID / "recompute.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
