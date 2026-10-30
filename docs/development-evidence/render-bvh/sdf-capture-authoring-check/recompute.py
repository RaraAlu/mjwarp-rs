#!/usr/bin/env python3
"""QA 独立复核：SDF 渲染参考采集 authoring（U2；离线 dry-run，零 GPU/零参考写入）。

只读：数字由 git 对象 / pinned 环境复跑 / 本目录 materials 现算；不采信被核证据的结论行。
被核：2026-10-30-render-sdf-capture-authoring（pin = 65a16430，U2 起点 828e37cc）
复核卡：2026-10-30-render-sdf-capture-authoring-check（revision.before = b9bc2c1d）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-capture-authoring-check/recompute.py
"""
import ast
import hashlib
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "65a16430"
START = "828e37cc"
CARD = "2026-10-30-render-sdf-capture-authoring"
EV = "docs/development-evidence/render-bvh/sdf-capture-authoring"
EVW = "docs/development-evidence/render-bvh/sdf-capture-window-material"
SCRIPT = "scripts/capture_render_sdf.py"
SCENE = "fixtures/render-bvh/scene-sdf/scene.xml"
SCENE_MIXED = "fixtures/render-bvh/scene-sdf/scene-mixed.xml"
SCRIPT_SHA = "9acd2130f58cfc2cc495bc73bae4e12cb1e9fb3c68c4d245819e809e524fde67"
SCENE_SHA = "376eb81ce200b29373af3965c7abefd90b8f00188a688db583080836dfa868e8"
SCENE_MIXED_SHA = "7869cfae5e16795ebf86b93c931ef36a8e94464452762e28f1357e0e68a1584e"

checks = []


def check(cid, ok, detail):
    checks.append({"id": cid, "passed": bool(ok), "detail": detail})


def git(*args, binary=False):
    out = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    if out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {out.stderr.decode('utf-8', 'replace')[:300]}")
    return out.stdout if binary else out.stdout.decode("utf-8", "replace")


def blob(rev, path):
    return git("cat-file", "blob", f"{rev}:{path}", binary=True)


def sha(b):
    return hashlib.sha256(b).hexdigest()


# 1. 脚本身份
s = blob(PIN, SCRIPT)
code = s.decode("utf-8")
try:
    ast.parse(code)
    ast_ok = True
except SyntaxError:
    ast_ok = False
check("script.identity",
      len(s) == 20380 and sha(s) == SCRIPT_SHA and s.count(b"\n") == 427
      and sum(1 for b in s if b > 127) == 0 and s.count(b"\r") == 0 and ast_ok,
      {"bytes": len(s), "sha256": sha(s), "lines": s.count(b"\n"),
       "non_ascii": sum(1 for b in s if b > 127), "cr": s.count(b"\r"), "ast_parse": ast_ok})

# 2. 场景身份
scenes = {}
for p, exp, nm in ((SCENE, SCENE_SHA, "scene"), (SCENE_MIXED, SCENE_MIXED_SHA, "scene_mixed")):
    b = blob(PIN, p)
    scenes[nm] = {"path": p, "bytes": len(b), "lines": b.count(b"\n"), "sha256": sha(b)}
check("scene.identity",
      scenes["scene"]["bytes"] == 3388 and scenes["scene"]["lines"] == 59 and sha(blob(PIN, SCENE)) == SCENE_SHA
      and scenes["scene_mixed"]["bytes"] == 3062 and scenes["scene_mixed"]["lines"] == 55
      and sha(blob(PIN, SCENE_MIXED)) == SCENE_MIXED_SHA,
      scenes)

# 3. 静态场景校验（QA 用 pinned MuJoCo 独立重跑，CPU-only）vs 证据 scene-check.json
qa_scene = json.loads((HERE / "materials" / "scene-check-qa.json").read_text(encoding="utf-8"))
ev_scene_b = blob(PIN, EV + "/logs/scene-check.json")
ev_scene = json.loads((ev_scene_b.decode("utf-16-le") if ev_scene_b[:2] in (b"\xff\xfe",) else ev_scene_b.decode()).lstrip("\ufeff"))
scene_diffs = []
for e, (g, m) in zip(ev_scene, qa_scene.items()):
    for k in ("ngeom", "nlight", "ncam", "nq", "nmesh", "sdf_iterations", "sdf_initpoints"):
        if e[k] != m[k]:
            scene_diffs.append([g, k, e[k], m[k]])
    eg = [{k: v for k, v in x.items() if k != "group"} for x in e["geoms"]]
    mg = [{k: v for k, v in x.items() if k != "group"} for x in m["geoms"]]
    if eg != mg:
        scene_diffs.append([g, "geoms"])
    if e["meshes"] != m["meshes"]:
        scene_diffs.append([g, "meshes"])
check("scene.static_check_reproduced", not scene_diffs,
      {"diffs": scene_diffs, "evidence_bytes": len(ev_scene_b),
       "qa_facts": {g: {k: d[k] for k in ("ngeom", "nlight", "ncam", "nq", "nmesh", "sdf_iterations", "sdf_initpoints")}
                    for g, d in qa_scene.items()}})

# 4. dry-run 离线复跑（CUDA_VISIBLE_DEVICES=99，无设备创建）vs 证据日志
dry = {}
for g in ("sdf-mesh", "sdf-occluder"):
    ev_b = blob(PIN, f"{EV}/logs/dryrun-{g}.json")
    ev = json.loads((ev_b.decode("utf-16-le") if ev_b[:2] in (b"\xff\xfe",) else ev_b.decode()).lstrip("\ufeff"))
    mine = json.loads((HERE / "materials" / f"dryrun-qa-{g}.json").read_text(encoding="utf-8"))
    diffs = []

    def walk(a, b, path=""):
        if type(a) is not type(b):
            diffs.append([path, type(a).__name__, type(b).__name__])
            return
        if isinstance(a, dict):
            for k in sorted(set(a) | set(b)):
                if k not in a or k not in b:
                    diffs.append([path + "." + k, a.get(k, "<missing>"), b.get(k, "<missing>")])
                else:
                    walk(a[k], b[k], path + "." + k)
        elif isinstance(a, list):
            if len(a) != len(b):
                diffs.append([path, f"len{len(a)}", f"len{len(b)}"])
                return
            for i, (x, y) in enumerate(zip(a, b)):
                walk(x, y, f"{path}[{i}]")
        elif a != b:
            diffs.append([path, a, b])

    walk(ev, mine)
    dry[g] = {"field_diffs": len(diffs), "first_diffs": diffs[:5],
              "legacy_entries": mine.get("legacy_manifest_entries"),
              "dry_run": mine.get("dry_run"), "repeat": mine.get("repeat"),
              "exit": 0}
check("dryrun.reproduced",
      all(v["field_diffs"] == 0 and v["legacy_entries"] == 69 and v["dry_run"] is True and v["repeat"] == 2
          for v in dry.values()),
      {"detail": dry, "note": "本复核以 CUDA_VISIBLE_DEVICES=99 复跑，exit 0；--out-dir 未创建"})

# 5. legacy 清单 69（文件系统 + 已跟踪两种口径）
root = REPO / "fixtures/render-bvh"
excl = (root / "scene-sdf").resolve()
fs_files = [p for p in root.rglob("*") if p.is_file() and not p.resolve().is_relative_to(excl)]
tracked = [p for p in git("ls-files", "fixtures/render-bvh").splitlines()
           if not p.startswith("fixtures/render-bvh/scene-sdf/")]
check("legacy.manifest_69",
      len(fs_files) == 69 and len(tracked) == 69 and not (root / "scene-sdf" / "capture-sdf").exists(),
      {"filesystem": len(fs_files), "tracked": len(tracked),
       "capture_dir_exists": (root / "scene-sdf" / "capture-sdf").exists()})

# 6. U2 提交范围（起点 828e37cc 之前 .. pin）
paths = [l.split("\t", 1)[1] for l in git("diff", "--name-status", f"{START}^..{PIN}").splitlines() if l.strip()]
check("scope.u2_commits",
      len(paths) == 17 and [p for p in paths if p.startswith(("src/", "tests/"))] == []
      and [p for p in paths if p.startswith("scripts/")] == [SCRIPT]
      and sorted(p for p in paths if p.startswith("fixtures/")) == [SCENE_MIXED, SCENE]
      and len([p for p in paths if p.startswith("docs/development-evidence/")]) == 12,
      {"total": len(paths), "scripts": [p for p in paths if p.startswith("scripts/")],
       "fixtures": sorted(p for p in paths if p.startswith("fixtures/")),
       "evidence": len([p for p in paths if p.startswith("docs/development-evidence/")]),
       "normify": [p for p in paths if p.startswith(".normify/")]})

# 7. 证据 manifest / 编码
man_report = {}
for d in (EV, EVW):
    man = blob(PIN, d + "/manifest.txt").decode("utf-8")
    rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
    okn = 0
    for s2, n2, p2 in rows:
        b = blob(PIN, p2)
        if sha(b) == s2 and len(b) == int(n2):
            okn += 1
    files = git("ls-tree", "-r", "--name-only", PIN, d).splitlines()
    utf16 = [p.split("/")[-1] for p in files if blob(PIN, p)[:2] in (b"\xff\xfe", b"\xfe\xff")]
    man_report[d.split("/")[-1]] = {"files": len(files), "rows": len(rows), "match": okn, "utf16": utf16,
                                    "self_registered": any(p2.endswith("/manifest.txt") for _, _, p2 in rows)}
check("evidence.manifests",
      man_report["sdf-capture-authoring"] == {"files": 8, "rows": 7, "match": 7, "utf16": man_report["sdf-capture-authoring"]["utf16"], "self_registered": False}
      and len(man_report["sdf-capture-authoring"]["utf16"]) == 3
      and man_report["sdf-capture-window-material"] == {"files": 4, "rows": 3, "match": 3, "utf16": [], "self_registered": False},
      man_report)

# 8. 窗口材料：命令逐字 / 阈值 / 双签栏
plan = blob(PIN, EVW + "/materials/window-plan.txt").decode("utf-8")
ep = json.loads(blob(PIN, EVW + "/materials/expected-products.json").decode("utf-8"))
cmd_mesh = ("target\\mjwarp-parallel\\venv\\Scripts\\python.exe -B scripts/capture_render_sdf.py "
            "--group sdf-mesh --use-fast-math false --use-shadows true --repeat 2 "
            "--out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh")
cmd_occ = cmd_mesh.replace("sdf-mesh", "sdf-occluder")
window = {
    "commands_verbatim": cmd_mesh in plan and cmd_occ in plan,
    "regular_threshold_no_exception": ("util < 40" in plan and "NO pre-approved exception" in plan
                                       and "OpenUtilLimit" in plan),
    "double_sign_block_unfilled": "USER DOUBLE-SIGN BLOCK" in plan and "user signature / date           : ____________________" in plan,
    "ep_self_checks": ep.get("self_checks", {}),
    "ep_script_sha": ep.get("script", {}).get("sha256") == SCRIPT_SHA,
    "ep_scene_shas": [x["scene_sha256"] for x in ep.get("groups", [])] == [SCENE_SHA, SCENE_MIXED_SHA],
    "ep_no_numeric_claim": "no numeric verdict" in json.dumps(ep, ensure_ascii=False),
}
check("window.material_consistency",
      window["commands_verbatim"] and window["regular_threshold_no_exception"]
      and window["double_sign_block_unfilled"] and window["ep_script_sha"] and window["ep_scene_shas"]
      and window["ep_no_numeric_claim"],
      window)

# 9. 窗口材料声称的脚本自检 vs 脚本实现
enforced_inside = 'if not out_dir.is_relative_to(SCENE_DIR.resolve()):' in code
outdir_empty_guard = any(k in code for k in ("iterdir", "is_empty", "non-empty", "nonempty", "exists()"))
check("script.out_dir_self_checks",
      enforced_inside and outdir_empty_guard,
      {"policy_declared": ep.get("self_checks", {}).get("out_dir_must_be_empty"),
       "script_enforces_inside_subtree": enforced_inside,
       "script_enforces_non_empty_refusal": outdir_empty_guard,
       "mkdir_call": "out_dir.mkdir(parents=True, exist_ok=True)" in code,
       "note": "窗口材料/expected-products 声明「--out-dir 必须非空即拒」，脚本未见该检查"})

# 10. 产品独立性（脚本不被产品构建/测试/门禁引用）
refs = subprocess.run(["git", "grep", "-n", "-I", "capture_render_sdf", "HEAD", "--",
                       "src", "tests", "build.rs", ".normify/gates.toml"],
                      cwd=REPO, capture_output=True).stdout.decode("utf-8", "replace")
prod_refs = [l for l in refs.splitlines() if l.strip()]
check("product.independence", not prod_refs, {"product_refs": prod_refs[:5],
                                              "gates_toml_mentions": "capture_render_sdf" in
                                              blob(PIN, ".normify/gates.toml").decode("utf-8")})

# 11. 卡面校验 / validate
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
check("card.validate_and_checks",
      bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5) and chk.returncode == 0,
      {"validate": vm.groups() if vm else None, "check_exit": chk.returncode})

# 12. 收据口径
r = json.loads(blob(PIN, EV + "/receipt.json").decode("utf-8"))
check("receipt.claims_and_boundaries",
      r["task_id"] == CARD and r["claims"] == {"numeric_acceptance": False, "sdf_rendering_covered": False}
      and r["boundaries"]["gpu"] is False and r["boundaries"]["window_opened"] is False
      and r["boundaries"]["device_execution"] is False and r["boundaries"]["src_changed"] is False
      and r["boundaries"]["tests_changed"] is False
      and r["dry_run"]["exit_codes"] == {"sdf-mesh": 0, "sdf-occluder": 0}
      and r["dry_run"]["out_dir_written"] is False and r["dry_run"]["cuda_device_created"] is False,
      {"task_id": r["task_id"], "claims": r["claims"], "boundaries": r["boundaries"],
       "dry_run": {k: r["dry_run"][k] for k in ("exit_codes", "cuda_device_created", "out_dir_written",
                                                "out_dir_exists_after", "legacy_manifest_entries")}})

problems = [
    {
        "id": "P1",
        "severity": "需修(轻微)",
        "check": "script.out_dir_self_checks",
        "finding": "窗口材料与 expected-products 都声明「--out-dir 必须未存在/非空即拒（hard self-check, 违反即非零退出）」，"
                   "但 scripts/capture_render_sdf.py 只校验 out-dir 位于 scene-sdf 子树内，随后 "
                   "`out_dir.mkdir(parents=True, exist_ok=True)` 直接覆盖既有产物，没有非空拒绝检查。"
                   "U3 窗口若对同一 group 重跑，旧产物会被静默覆盖，与「失败即停手/不静默」口径不一致。",
        "fix": "在 main() 里 out_dir 解析后加：`if out_dir.exists() and any(out_dir.iterdir()): fail(\"--out-dir 非空\")`；"
               "或把窗口材料/expected-products 的措辞改成实际行为（不重跑 dry-run 亦可，需重跑 dry-run 才刷新证据 sha）。",
    }
]
for c in checks:
    c["ok"] = c["passed"]

out = {
    "schema": 1,
    "task": "2026-10-30-render-sdf-capture-authoring-check",
    "reviewed_task": CARD,
    "pin": PIN,
    "u2_start": START,
    "card_head": "b9bc2c1d",
    "checks": checks,
    "problems": problems,
    "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                "problems": len(problems), "blockers": 0},
}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
