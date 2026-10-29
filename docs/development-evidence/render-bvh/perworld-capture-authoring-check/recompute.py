#!/usr/bin/env python3
"""QA 独立重算：逐 world 渲染采集 authoring 复核（只读）。

被核主卡：`2026-10-30-render-perworld-capture-authoring`（Arch，verified）
实现提交：`28822740`（脚本 + 证据 + 卡）→ 收口 `a4d25ead`；主卡 revision.before = `84ef186f`
只读边界：不跑 GPU 采集、不写 fixtures 参考、不改产品代码；允许 pinned python 离线 `--dry-run`。

输出 `recompute.json` + stdout（logs/recompute.log）。
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

BASE = "84ef186f"
IMPL = "28822740"
CLOSE = "a4d25ead"
HEAD = "b4e050d1"
EVID = "docs/development-evidence/render-bvh/perworld-capture-authoring"
CHECK_EVID = "docs/development-evidence/render-bvh/perworld-capture-authoring-check"
MAIN_TASK = "2026-10-30-render-perworld-capture-authoring"
CHECK_TASK = "2026-10-30-render-perworld-capture-authoring-check"
SCRIPT = "scripts/capture_render_perworld.py"
VENV_PY = "target/mjwarp-parallel/venv/Scripts/python.exe"
UPSTREAM = "target/mjwarp-parallel/upstream"
UPSTREAM_REV = "71da24d956378a87a703b6e1442b13aec0c4ac29"
OUT_DIR = "fixtures/render-bvh/scene-a/perworld/nw3-hinge02040"
SCENE = "fixtures/render-bvh/scene-a/scene.xml"
EXPECTED_SCRIPT_SHA = "602e36daab79fd4dc269d9f935adc916ec8057b742e023d270541404d477fb36"

checks = []


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})


def run(args, env=None, text=True):
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    proc = subprocess.run(args, cwd=REPO, capture_output=True, env=full_env)
    if text:
        return (proc.returncode, proc.stdout.decode("utf-8-sig", "replace"),
                proc.stderr.decode("utf-8", "replace"))
    return proc.returncode, proc.stdout, proc.stderr


def git(*args):
    return run(["git", *args])


def blob(rev, path):
    code, data, _ = run(["git", "cat-file", "-p", f"{rev}:{path}"], text=False)
    return data if code == 0 else None


def disk(path):
    full = os.path.join(REPO, path)
    if not os.path.isfile(full):
        return None
    return open(full, "rb").read()


def text_of(path, rev=None):
    data = blob(rev, path) if rev else disk(path)
    return None if data is None else data.decode("utf-8-sig")


def lf_lines(data):
    return data.count(b"\n")


PY = os.path.join(REPO, VENV_PY)

# ---------------- 1. 脚本事实 ----------------
script_raw = disk(SCRIPT)
script_text = script_raw.decode("ascii", "replace")
code, out, err = run([PY, "-c", "import ast,sys;ast.parse(open(sys.argv[1],'rb').read());print('PY_SYNTAX_OK')", SCRIPT])
script = {
    "path": SCRIPT,
    "bytes": len(script_raw),
    "sha256": hashlib.sha256(script_raw).hexdigest(),
    "lf_lines": lf_lines(script_raw),
    "non_ascii_bytes": sum(1 for b in script_raw if b > 0x7F),
    "ast_parse": code == 0 and "PY_SYNTAX_OK" in out,
}
receipt = json.loads(text_of(f"{EVID}/receipt.json"))
add(
    "script_facts_match_receipt",
    script["sha256"] == EXPECTED_SCRIPT_SHA
    and script["sha256"] == receipt["script"]["sha256"]
    and script["bytes"] == receipt["script"]["bytes"] == 23521
    and script["lf_lines"] == receipt["script"]["lines_lf"] == 497
    and script["lf_lines"] <= 500
    and script["non_ascii_bytes"] == 0
    and script["ast_parse"],
    script,
)

# ---------------- 2. 脚本静态范围/防护 ----------------
guard = {
    "out_dir_inside_perworld_guard": "is_relative_to(OUT_ROOT" in script_text,
    "append_only_guard": "already exists and is not empty" in script_text,
    "legacy_manifest_before_after": "legacy_manifest() == legacy_before" in script_text,
    "dry_run_no_device": "check_environment(want_cuda_device=False" in script_text
                         and "if want_cuda_device:" in script_text
                         and script_text.count("wp.get_device(") == 1,
    "writes_only_under_out_dir": all(
        token in script_text for token in ("out_dir / name", "tmp_second", "(out_dir / \"capture.json\")")),
    "no_product_tree_read": ("product_python_dependency\": False" in script_text
                              and "reads_product_tree\": False" in script_text),
    "per_world_scope_in_capture": ("per_world_scope" in script_text
                                   and "does_not_cover" in script_text
                                   and "per-world light poses" in script_text),
    "defaults": all(token in script_text for token in (
        "DEFAULT_WORLDS = 3", "DEFAULT_HINGE_DEG = \"0,20,40\"",
        "default=\"false\"", "default=\"true\"", "default=20261030", "choices=(1, 2)")),
    "revision_pinned": UPSTREAM_REV in script_text,
    "derived_from_hfield": text_of("scripts/capture_render_hfield.py") is not None,
}
add("script_static_guards", all(guard.values()), guard)

# ---------------- 3. pinned 环境 / 上游 / runner 事实 ----------------
env_code = (
    "import hashlib,json,platform,sys,inspect\n"
    "import mujoco, mujoco_warp as mjw, numpy as np, warp as wp\n"
    "out={'python':platform.python_version(),'prefix':sys.prefix,'executable':sys.executable,\n"
    " 'exe_sha256':hashlib.sha256(open(sys.executable,'rb').read()).hexdigest(),\n"
    " 'versions':{'python':platform.python_version(),'warp':wp.__version__,'mujoco':mujoco.__version__,\n"
    "  'mujoco_warp':mjw.__version__,'numpy':np.__version__},\n"
    " 'put_data_params':list(inspect.signature(mjw.put_data).parameters),\n"
    " 'ctx_params':list(inspect.signature(mjw.create_render_context).parameters),\n"
    " 'has_get':all(hasattr(mjw,n) for n in ('get_rgb','get_depth','get_segmentation'))}\n"
    "print(json.dumps(out))\n"
)
code, out, err = run([PY, "-c", env_code])
env_info = json.loads(out) if code == 0 else {}
up_head_code, up_head, _ = run(["git", "-C", os.path.join(REPO, UPSTREAM), "rev-parse", "HEAD"])
up_status_code, up_status, _ = run(["git", "-C", os.path.join(REPO, UPSTREAM), "status", "--porcelain"])
scene_raw = disk(SCENE)
receipt_env = receipt["environment"]
env_ok = (
    code == 0
    and env_info.get("versions") == receipt_env["versions"]
    and env_info.get("exe_sha256") == receipt_env["python_executable_sha256"]
    and os.path.normcase(env_info.get("executable", "")) == os.path.normcase(PY)
    and up_head.strip() == UPSTREAM_REV
    and up_head.strip() == receipt_env["upstream_revision"]
    and up_status.strip() == ""
    and hashlib.sha256(scene_raw).hexdigest() == receipt_env["scene_xml"]["sha256"]
    and len(scene_raw) == receipt_env["scene_xml"]["bytes"]
)
add(
    "pinned_environment_and_scene",
    env_ok,
    {
        "pinned_python_exists": os.path.isfile(PY),
        "versions": env_info.get("versions"),
        "exe_sha256": env_info.get("exe_sha256"),
        "upstream_head": up_head.strip(),
        "upstream_clean": up_status.strip() == "",
        "scene_bytes": len(scene_raw),
        "scene_sha256": hashlib.sha256(scene_raw).hexdigest(),
        "matches_receipt": env_ok,
        "import_stderr_head": err.strip().splitlines()[:2],
    },
)

# ---------------- 4. 能力事实（pinned 上游源码） ----------------
src = {name: text_of(f"{UPSTREAM}/mujoco_warp/_src/{name}") or "" for name in
       ("io.py", "types.py", "render_util.py", "render.py")}
put_sig = re.search(r"def put_data\((.*?)\)\s*->", src["io.py"], re.S)
put_sig_text = put_sig.group(1) if put_sig else ""
qpos_decl = re.search(r'qpos\s*:\s*array\(\s*"nworld"\s*,\s*"nq"\s*,\s*float\s*\)', src["types.py"])
geom_xpos_decl = re.search(r'geom_xpos\s*:\s*array\(\s*"nworld"\s*,\s*"ngeom"\s*,\s*wp\.vec3\s*\)', src["types.py"])
geom_xmat_decl = re.search(r'geom_xmat\s*:\s*array\(\s*"nworld"\s*,\s*"ngeom"\s*,\s*wp\.mat33\s*\)', src["types.py"])
ctx_sig = re.search(r"def create_render_context\([\s\S]{0,400}?nworld", src["render_util.py"])
get_rgb_sig = re.search(r"def get_rgb\(", src["render_util.py"])
get_rgb_doc_shape = re.search(r"get_rgb[\s\S]{0,600}?shape \(nworld, height, width\)", src["render_util.py"])
worldid_index = re.findall(r"geom_xpos_in\[worldid\s*,\s*\w+\]", src["render.py"])
geometries_doc = re.search(r"\(nworld, ngeom, 3\)", src["types.py"]) and re.search(r"\(nworld, ngeom, 3, 3\)", src["types.py"])
qpos_doc = re.search(r"\(nworld, nq\)", src["types.py"])
capability = {
    "put_data_nworld_param": "nworld" in put_sig_text or "nworld" in src["io.py"],
    "data_qpos_nworld_nq": bool(qpos_decl),
    "data_geom_xpos_nworld_ngeom_3": bool(geom_xpos_decl),
    "data_geom_xmat_nworld_ngeom_3_3": bool(geom_xmat_decl),
    "docstring_shapes": bool(geometries_doc and qpos_doc),
    "create_render_context_nworld": bool(ctx_sig),
    "get_rgb_defined": bool(get_rgb_sig),
    "get_rgb_doc_nworld_hw": bool(get_rgb_doc_shape),
    "render_samples_geom_xpos_in_worldid": len(worldid_index) > 0,
    "runtime_put_data_params": env_info.get("put_data_params"),
    "runtime_ctx_params_head": (env_info.get("ctx_params") or [])[:3],
    "runtime_has_get_rgb_depth_seg": env_info.get("has_get"),
    "worldid_index_samples": worldid_index[:3],
}
add(
    "perworld_capability_from_pinned_upstream",
    capability["put_data_nworld_param"] and capability["data_qpos_nworld_nq"]
    and capability["data_geom_xpos_nworld_ngeom_3"] and capability["data_geom_xmat_nworld_ngeom_3_3"]
    and capability["docstring_shapes"] and capability["create_render_context_nworld"]
    and capability["get_rgb_defined"] and capability["get_rgb_doc_nworld_hw"]
    and capability["render_samples_geom_xpos_in_worldid"]
    and "nworld" in (env_info.get("put_data_params") or [])
    and "nworld" in (env_info.get("ctx_params") or [])
    and env_info.get("has_get") is True,
    capability,
)

# ---------------- 5. legacy manifest 独立计数 ----------------
def legacy_manifest():
    root = os.path.join(REPO, "fixtures/render-bvh")
    exclude = os.path.join(REPO, "fixtures/render-bvh/scene-a/perworld")
    count = 0
    for base, _dirs, files in os.walk(root):
        if os.path.normcase(base).startswith(os.path.normcase(exclude)):
            continue
        for name in files:
            if os.path.normcase(os.path.join(base, name)).startswith(os.path.normcase(exclude)):
                continue
            count += 1
    return count


legacy_count = legacy_manifest()
add(
    "legacy_manifest_count",
    legacy_count == receipt["dry_run"]["legacy_manifest_entries"] == 69,
    {"my_count": legacy_count, "receipt": receipt["dry_run"]["legacy_manifest_entries"],
     "dryrun_plan_entries": json.loads(text_of(f"{EVID}/materials/dryrun-plan.json"))["legacy_manifest_entries"]},
)

# ---------------- 6. dry-run 离线复跑（不写输出、不建 device） ----------------
out_abs = os.path.join(REPO, OUT_DIR)
out_exists_before = os.path.exists(out_abs)
manifest_before = sorted(
    (os.path.relpath(os.path.join(b, f), REPO), hashlib.sha256(open(os.path.join(b, f), "rb").read()).hexdigest())
    for b, _d, fs in os.walk(os.path.join(REPO, "fixtures/render-bvh")) for f in fs
)
cmd = [PY, "-B", SCRIPT, "--dry-run", "--out-dir", OUT_DIR]
code, out, err = run(cmd)
out_exists_after = os.path.exists(out_abs)
manifest_after = sorted(
    (os.path.relpath(os.path.join(b, f), REPO), hashlib.sha256(open(os.path.join(b, f), "rb").read()).hexdigest())
    for b, _d, fs in os.walk(os.path.join(REPO, "fixtures/render-bvh")) for f in fs
)
material_raw = text_of(f"{EVID}/materials/dryrun-plan.json")
material = json.loads(material_raw)
plan = json.loads(out) if code == 0 and out.strip() else {}
dry_ok = (
    code == 0 and out_exists_before is False and out_exists_after is False
    and manifest_before == manifest_after
    and plan.get("dry_run") is True
    and plan == material
)
add(
    "dry_run_reproduced_equals_material",
    dry_ok,
    {
        "command": " ".join(cmd),
        "exit": code,
        "out_dir_existed_before": out_exists_before,
        "out_dir_exists_after": out_exists_after,
        "fixtures_manifest_unchanged": manifest_before == manifest_after,
        "stdout_equals_materials_dryrun_plan_json": plan == material,
        "plan_keys": sorted(plan.keys()),
        "cam_active_fixed_only": plan.get("cam_active_fixed_only"),
        "camera_modes": (plan.get("environment") or {}).get("camera_modes"),
        "stderr_head": err.strip().splitlines()[:3],
    },
)

# 无 device 复跑：把 CUDA 可见设备置为不存在的索引，仍应 exit 0 且输出一致
code2, out2, err2 = run(cmd, env={"CUDA_VISIBLE_DEVICES": "99"})
plan2 = json.loads(out2) if code2 == 0 and out2.strip() else {}
add(
    "dry_run_needs_no_cuda_device",
    code2 == 0 and plan2 == material,
    {
        "env": "CUDA_VISIBLE_DEVICES=99",
        "exit": code2,
        "stdout_equals_material": plan2 == material,
        "stderr_head": err2.strip().splitlines()[:3],
        "static_proof": "dry 分支传 want_cuda_device=False，wp.get_device 只在 if want_cuda_device: 内",
    },
)

# ---------------- 7. 证据（5 文件）committed blob LF/无 BOM、/task_id、禁词 ----------------
evidence_paths = [SCRIPT, f"{EVID}/README.md", f"{EVID}/receipt.json",
                  f"{EVID}/materials/dryrun-plan.json",
                  f".normify/changes/{MAIN_TASK}.json"]
evidence_rows = []
for path in evidence_paths:
    committed = blob(HEAD, path)
    work = disk(path)
    evidence_rows.append({
        "path": path,
        "blob_bytes": len(committed),
        "blob_lf": lf_lines(committed),
        "blob_crlf": committed.count(b"\r\n"),
        "blob_bom": committed.startswith(b"\xef\xbb\xbf"),
        "worktree_crlf": work.count(b"\r\n"),
    })
terms = ["".join(map(chr, c)) for c in (
    [0x9010, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x20, 0x706F, 0x4F4D, 0x59FF, 0x5DF2, 0x8986, 0x76D6],
    [0x6E32, 0x67D3, 0x6570, 0x503C, 0x9A8C, 0x6536, 0x901A, 0x8FC7],
    [0x7B49, 0x4EF7],
)]
scanned = [p for p in evidence_paths if not p.startswith(".normify/")]
term_hits = []
for path in scanned:
    body = blob(HEAD, path).decode("utf-8-sig")
    for term in terms:
        if term in body:
            term_hits.append({"file": path, "codepoints": [hex(ord(c)) for c in term]})
add(
    "evidence_integrity_5_files",
    len(evidence_rows) == 5
    and all(row["blob_crlf"] == 0 and not row["blob_bom"] for row in evidence_rows)
    and receipt.get("task_id") == MAIN_TASK
    and not term_hits,
    {
        "files": evidence_rows,
        "committed_blob_all_lf_no_bom": all(row["blob_crlf"] == 0 and not row["blob_bom"]
                                            for row in evidence_rows),
        "worktree_crlf_note": "materials/dryrun-plan.json 工作区 CRLF 是作者重定向输出产物；committed blob 为 LF",
        "receipt_task_id": receipt.get("task_id"),
        "forbidden_scan_scope": "只扫 4 个证据文件（不含卡面；卡面自身含 forbid_claim 术语定义）",
        "forbidden_term_hits": term_hits,
        "scope_wording": {
            "readme_section_4": "不宣称 per-world 灯位姿" in (text_of(f"{EVID}/README.md") or ""),
            "receipt_not_claimed_english": any("light" in item for item in receipt.get("not_claimed", [])),
            "dryrun_plan_note": "not claimed" in json.dumps(material, ensure_ascii=False),
            "script_capture_scope": "does_not_cover" in script_text,
        },
    },
)

# ---------------- 8. 零越界 ----------------
code, out, _ = git("diff", "--name-only", f"{BASE}..{CLOSE}",
                   "--", "fixtures", "src", "tests", "Cargo.toml", "Cargo.lock", "build.rs")
zero = [line for line in out.splitlines() if line.strip()]
code, scripts_out, _ = git("diff", "--name-status", f"{BASE}..{CLOSE}", "--", "scripts")
script_changes = [line for line in scripts_out.splitlines() if line.strip()]
code, stat_out, _ = git("diff", "--stat", f"{IMPL}^..{CLOSE}")
add(
    "zero_out_of_scope_changes",
    not zero and script_changes == [f"A\t{SCRIPT}"] and not os.path.exists(out_abs),
    {
        "fixtures_src_tests_cargo_diff": zero,
        "scripts_changes": script_changes,
        "impl_and_close_stat": stat_out.splitlines()[-1] if stat_out else "",
        "perworld_out_dir_absent_in_repo": not os.path.exists(out_abs),
    },
)

# ---------------- 9. 卡状态 ----------------
main_card = json.loads(text_of(f".normify/changes/{MAIN_TASK}.json"))
check_card = json.loads(text_of(f".normify/changes/{CHECK_TASK}.json"))
add(
    "card_state",
    main_card["status"] == "verified"
    and main_card["revision"]["before"] == "84ef186f9e78aa238125371d761343239be118cd"
    and bool(main_card.get("closed_at"))
    and check_card["status"] == "in_progress"
    and check_card["revision"]["before"] == "b4e050d1114c95516874ee07fb02518552b4a5bd",
    {
        "main": {"status": main_card["status"], "revision_before": main_card["revision"]["before"],
                 "closed_at": main_card.get("closed_at")},
        "check": {"status": check_card["status"], "revision_before": check_card["revision"]["before"]},
    },
)

summary = {
    "task": CHECK_TASK,
    "reviewed": {"main_task": MAIN_TASK, "base": BASE, "impl": IMPL, "close": CLOSE, "head": HEAD},
    "checks": checks,
    "problems": [c for c in checks if not c["ok"]],
}
with open(os.path.join(HERE, "recompute.json"), "w", encoding="utf-8", newline="\n") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

for check in checks:
    print(f"[{'PASS' if check['ok'] else 'FAIL'}] {check['id']}")
    print(json.dumps(check["detail"], ensure_ascii=False, sort_keys=True)[:1600])
print(f"\nchecks={len(checks)} problems={len(summary['problems'])}")
sys.exit(0 if not summary["problems"] else 1)
