"""R05 hfield-channel 独立复核（QA；只读 + 静态复算）。

复核对象：`2026-10-28-render-bvh-r05-hfield-channel`（实现 `9a6fd66e` / 收口 `c8ccf66c`，判据树 HEAD `9a32bf87`）。
本脚本独立重算，不引用 Backend/Arch 数字：

  1. 实现证据完整性：7 文件 / manifest 6 条对 committed blob、不自引用、LF 无 BOM、禁词 0；
  2. 行数与红线（hfield_inputs 70、tests 218、params 197、hfield 372、mod 384、红灯文件 499/500/494/500/499/490、io/render 453）；
  3. 零 native / 零字段表 / 零 ABI：提交只增 4 源码路径 + 治理/证据；`RENDER_FIELDS` 仍 116；
     `model/render.rs`/`io/render.rs`/native/`render.cu` 等零改动；81 槽、DEVICE_SOURCES 28、CollisionArgs 504 B；
  4. 适配器语义静态核对：`nhfield==0 → Ok(None)` 提前返回、`size` 仅经 `hfield_size_f32()`（唯一转换点）、
     `data` 借用不复制、无 `nworld`、错误类型直传；
  5. 两层语义：体积层 `check_ranges` 只拒负数（`>= 0`）、`num_to_f32` 即 `as f32`、S1 层对 `<=0` 拒绝；
  6. 模块源 36 且 hfield_inputs/hfield 均登记；
  7. 门禁：四门禁 exit 0、unit 1069 过 / 0 败 / 387 ignored、实现片 7 条单测在 test 日志通过；
  8. 我的独立对抗性探针日志（临时测试，运行后删除）：4 条用例全过。
"""
import hashlib
import json
import pathlib
import re
import subprocess

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/render-bvh/r05-hfield-channel-check"
IMPL = REPO / "docs/development-evidence/render-bvh/r05-hfield-channel"
JUDGED = "9a32bf87"

result = {"task_id": "2026-10-28-render-bvh-r05-hfield-channel-check", "judged_tree": JUDGED, "checks": {}, "problems": []}


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
    b = blob("docs/development-evidence/render-bvh/r05-hfield-channel/" + rel)
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
    "src/render/context/hfield_inputs.rs": 70,
    "tests/render_hfield_inputs.rs": 218,
    "src/render/context/params.rs": 197,
    "src/render/context/hfield.rs": 372,
    "src/render/mod.rs": 384,
    "src/model/render.rs": 500,
    "src/io/render.rs": 453,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/render/context/camera_output.rs": 499,
    "src/render/kernel/megakernel.rs": 490,
}.items():
    n = (REPO / rel).read_bytes().count(b"\n")
    lines[rel] = {"lf": n, "claimed": claim, "match": n == claim, "le_500": n <= 500}
    if n != claim:
        fail(f"行数不符：{rel} 实测 {n} 声称 {claim}")
note("line_counts", lines)

# 3. 零 native / 零字段表 / 零 ABI
commit_files = [l.split("\t")[2] for l in git("show", "--numstat", "--format=", "9a6fd66e").splitlines() if l.strip()]
note("impl_commit_files", commit_files)
allowed = {
    "src/render/context/hfield_inputs.rs",
    "src/render/context/params.rs",
    "tests/render_hfield_inputs.rs",
    ".normify/modules/mjwarp/render.md",
    ".normify/changes/2026-10-28-render-bvh-r05-hfield-channel.json",
}
out_of_scope = [
    f for f in commit_files
    if f not in allowed and not f.startswith("docs/development-evidence/render-bvh/r05-hfield-channel/")
]
note("impl_commit_out_of_scope", out_of_scope)
if out_of_scope:
    fail(f"实现提交含范围外路径：{out_of_scope}")
redline_paths = [
    "src/model/render.rs", "src/io/render.rs", "native", "include", "src/render/kernel",
    "fixtures", "src/runtime", "Cargo.toml", "Cargo.lock",
]
redline = git("diff", "--numstat", "9a6fd66e^..HEAD", "--", *redline_paths).strip()
note("redline_native_diff_since_impl_parent", redline)
if redline:
    fail(f"红线/native/夹具路径有改动：{redline}")
model = (REPO / "src/model/render.rs").read_text(encoding="utf-8")
spec = re.search(r"pub const RENDER_FIELDS: \[FieldSpec; (\d+)\]", model)
note("render_fields_len", int(spec.group(1)) if spec else None)
if not spec or spec.group(1) != "116":
    fail("RENDER_FIELDS 不再是 116 项")
abi = {
    "megakernel_slots": re.search(r"pub static MEGAKERNEL_SLOTS: \[RenderArgKind; (\d+)\]",
                                  (REPO / "src/runtime/launcher/render/args.rs").read_text(encoding="utf-8")).group(1),
    "device_sources": re.search(r"DEVICE_SOURCES: \[\(&str, &str\); (\d+)\]",
                                (REPO / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8")).group(1),
    "collision_args_504": "size_of::<CollisionArgs>(), 504" in
    (REPO / "src/runtime/collision_gpu/abi_tests.rs").read_text(encoding="utf-8"),
    "hfield_bvh_id_placeholder": "upload(&self.session, &[0u32, 0])" in
    (REPO / "src/render/kernel/megakernel.rs").read_text(encoding="utf-8"),
}
note("abi", abi)
if abi["megakernel_slots"] != "81" or abi["device_sources"] != "28" or not abi["collision_args_504"] or not abi["hfield_bvh_id_placeholder"]:
    fail(f"ABI 核对失败：{abi}")

# 4. 适配器语义静态核对
adapter = (REPO / "src/render/context/hfield_inputs.rs").read_text(encoding="utf-8")
code = "\n".join(l for l in adapter.splitlines() if not l.strip().startswith("//"))
sem = {
    "has_from_volume": "pub fn from_volume" in code,
    "has_as_inputs": "pub fn as_inputs" in code,
    "has_resolve": "pub fn resolve" in code,
    "nhfield_zero_early_return": "if model.counts().nhfield == 0" in code and "return Ok(None)" in code,
    "size_via_single_conversion_fn": "model.hfield_size_f32()?" in code,
    "no_local_f64_to_f32_conversion": "as f32" not in code and "num_to_f32" not in code,
    "data_borrowed": "model.hfield_data()?" in code and ".to_vec()" not in code and "clone()" not in code,
    "no_nworld_in_code": "nworld" not in code,
    "error_passthrough_result": "Result<Option<Self>, NativeProbeError>" in adapter,
}
note("adapter_static", sem)
for k, v in sem.items():
    if not v:
        fail(f"适配器静态核对失败：{k}")

# 5. 两层语义
volume = (REPO / "src/model/volume_input.rs").read_text(encoding="utf-8")
vol = (REPO / "src/model/volume.rs").read_text(encoding="utf-8")
two_layer = {
    "volume_accepts_nonnegative": "nrow[index] >= 0" in volume and "ncol[index] >= 0" in volume and "adr[index] >= 0" in volume,
    "volume_rejects_negative": ">= 0" in volume,
    "num_to_f32_is_as_f32": "values.iter().map(|&value| value as f32).collect()" in vol,
    "s1_rejects_nonpositive": "rows <= 0 || cols <= 0" in (REPO / "src/render/context/hfield.rs").read_text(encoding="utf-8"),
}
note("two_layer_semantics", two_layer)
if not all(two_layer.values()):
    fail(f"两层语义静态核对失败：{two_layer}")

# 6. 模块源
render_md = (REPO / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
srcs = re.findall(r'\[\[source\]\]\npath = "([^"]+)"', render_md)
note("module_sources", {"count": len(srcs),
                        "hfield_inputs": "src/render/context/hfield_inputs.rs" in srcs,
                        "hfield": "src/render/context/hfield.rs" in srcs})
if len(srcs) != 36 or "src/render/context/hfield_inputs.rs" not in srcs:
    fail(f"模块源登记不符：count={len(srcs)}")

# 7. 门禁与实现单测
runs = REPO / ".normify/runs/2026-10-28-render-bvh-r05-hfield-channel"
gates = {}
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    p = runs / f"{g}.json"
    if p.exists():
        d = json.loads(p.read_text(encoding="utf-8"))
        gates[g] = {"exit": d.get("exit"), "at": d.get("at"), "passed": d.get("tests", {}).get("passed"),
                    "failed": d.get("tests", {}).get("failed"), "ignored": d.get("tests", {}).get("ignored")}
note("impl_gate_records", gates)
if len(gates) != 4 or any(v["exit"] != 0 for v in gates.values()):
    fail(f"实现片门禁记录不全或非 0：{gates}")
if gates.get("unit", {}).get("passed") != 1069 or gates.get("unit", {}).get("failed") != 0:
    fail(f"实现片 unit 计数不符：{gates.get('unit')}")

test_log = (IMPL / "logs/test.log").read_text(encoding="utf-8", errors="replace")
names = ["adapts_volume_hfield_into_inputs", "resolve_full_chain_matches_s1_views", "no_hfield_returns_none",
         "g10_fixture_arrays_match_volume_adapter", "size_conversion_is_single_f64_to_f32",
         "boundary_errors_are_rejected_by_volume_construction", "adapter_is_stable_and_copy_free"]
impl_tests = {"test_log_7_passed": "test result: ok. 7 passed; 0 failed" in test_log,
              "all_names_in_log": all(n in test_log for n in names)}
note("impl_tests", impl_tests)
if not all(impl_tests.values()):
    fail(f"实现单测日志不符：{impl_tests}")

# 8. 我的独立对抗性探针
probe_path = EVID / "logs/probe-adapter.log"
probe = probe_path.read_text(encoding="utf-8", errors="replace") if probe_path.exists() else ""
probes = {
    "passed": "test result: ok. 4 passed; 0 failed" in probe,
    "cases": all(n in probe for n in ("zero_nrow_and_zero_ncol_two_layer", "negative_and_overflow_rejected_at_volume_layer",
                                      "multi_hfield_and_single_conversion", "no_hfield_returns_none_without_field_reads")),
}
note("my_probes", probes)
if not all(probes.values()):
    fail(f"我的适配器探针未全过：{probes}")

# 9. 我的四门禁日志
own = {}
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    p = EVID / "logs" / f"{g}.log"
    if p.exists():
        own[g] = {"bytes": p.stat().st_size}
note("my_gate_logs", own)
unit_own = (EVID / "logs/unit.log").read_text(encoding="utf-8", errors="replace") if (EVID / "logs/unit.log").exists() else ""
passed = sum(int(m.group(1)) for m in re.finditer(r"test result: ok\. (\d+) passed", unit_own))
failed = sum(int(m.group(1)) for m in re.finditer(r"test result: ok\. \d+ passed; (\d+) failed", unit_own))
note("my_unit_aggregate", {"passed": passed, "failed": failed,
                           "result_lines": len(re.findall(r"test result:", unit_own))})
if passed != 1069 or failed != 0:
    fail(f"我的 unit 聚合不符：passed={passed} failed={failed}")

# 10. 本复核证据 manifest 自校
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
            b = p.read_bytes()
            src = "worktree"
        else:
            b = blob(rel)
            src = "git_blob"
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
