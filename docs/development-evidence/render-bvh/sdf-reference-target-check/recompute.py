#!/usr/bin/env python3
"""QA 独立复核：U4 SDF 参考验证目标实现（render_sdf_reference + 支撑模块 + 冻结器）。

只读：结论由 committed blob / 本卡自跑门禁日志现算；不采信证据结论行。
被核 pin：4e761a7b；复核卡：2026-10-31-render-sdf-reference-target-check（revision.before = e83ed63b）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-reference-target-check/recompute.py
"""
import hashlib
import json
import os
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "4e761a7b"
CARD = "2026-10-31-render-sdf-reference-target"
EV = "docs/development-evidence/render-bvh/sdf-reference-target"

checks = []


def check(cid, ok, detail):
    checks.append({"id": cid, "passed": bool(ok), "detail": detail})


def git(*args):
    out = subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True)
    if out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {out.stderr.decode('utf-8', 'replace')[:300]}")
    return out.stdout


def blob(rev, p):
    return git("cat-file", "blob", f"{rev}:{p}")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def txt(rev, p):
    b = blob(rev, p)
    return (b.decode("utf-16-le") if b[:2] in (b"\xff\xfe",) else b.decode("utf-8", "replace")).lstrip("\ufeff")


# 1. 范围：只新增 tests/scripts/证据/卡；无产品 src/.cu/既有 tests 改动；guards 文件未生成
paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{PIN}^..{PIN}").decode().splitlines() if l.strip()]
new_tests = ["tests/render_sdf_reference.rs", "tests/support/render_bvh/sdf.rs",
             "tests/support/render_bvh/sdf_scene.rs", "tests/support/render_bvh/sdf_compare.rs"]
status = git("show", "--name-status", "--format=", PIN).decode()
check("scope.pin_commit",
      len(paths) == 23
      and all(f"A\t{p}" in status for p in new_tests)
      and "scripts/validate/render-bvh/freeze-guards-sdf.ps1" in paths
      and not [p for p in paths if p.startswith(("src/", "fixtures/", "Cargo"))
               or p.startswith("tests/") and p not in new_tests]
      and not (REPO / "scripts/validate/render-bvh/guards-sdf-validate.json").exists(),
      {"paths": len(paths), "new_tests": new_tests, "guards_file_created": False})

# 2. 行数 / 500 行棘轮 / 材料预算
counts = {p: blob(PIN, p).count(b"\n") for p in
          new_tests + ["scripts/validate/render-bvh/freeze-guards-sdf.ps1"]}
over500 = [p for p, n in counts.items() if n > 500]
budget = counts["tests/render_sdf_reference.rs"] > 360
check("files.line_counts",
      counts["tests/render_sdf_reference.rs"] == 361 and counts["tests/support/render_bvh/sdf.rs"] == 372
      and counts["tests/support/render_bvh/sdf_scene.rs"] == 323
      and counts["tests/support/render_bvh/sdf_compare.rs"] == 260
      and counts["scripts/validate/render-bvh/freeze-guards-sdf.ps1"] == 167
      and not over500,
      {"lines": counts, "over_500": over500,
       "material_budget_260_360_exceeded_by": 1 if budget else 0,
       "note": "材料行预算 260-360，实际 361（1 行）；500 行棘轮仍满足"})

# 3. 用例：8 个（host 3 不 ignore、GPU 5 ignore），名字与 receipt/test-list 一致
src = txt(PIN, "tests/render_sdf_reference.rs")
tests = re.findall(r"#\[test\]\s*\n\s*(?:#\[ignore[^\n]*\]\s*\n\s*)?fn ([a-z0-9_]+)\(", src)
gpu = re.findall(r"#\[test\]\s*\n\s*#\[ignore[^\n]*\]\s*\n\s*fn ([a-z0-9_]+)\(", src)
host = [t for t in tests if t not in gpu]
expected_host = ["model_dumps_match_signed_scene_semantics", "capture_references_parse_and_cross_checks_hold",
                 "sdf_leaf_geometry_uses_mesh_bounds"]
expected_gpu = ["sdf_mesh_matches_captured_reference", "sdf_occluder_matches_captured_reference",
                "shadow_toggle_consistency", "depth_seg_cross_path_consistency", "two_run_determinism"]
host_log = (HERE / "logs" / "host-test.log").read_text(encoding="utf-8", errors="replace")
host_ok = "test result: ok. 3 passed; 0 failed; 5 ignored" in host_log
check("cases.eight_with_three_host",
      sorted(host) == sorted(expected_host) and sorted(gpu) == sorted(expected_gpu) and host_ok,
      {"host": host, "gpu_ignored": gpu, "host_run": "3 passed / 5 ignored" if host_ok else host_log[-300:]})

# 4. 判据零放宽（代码级）
cmp_src = txt(PIN, "tests/support/render_bvh/sdf_compare.rs")
out_rs = git("cat-file", "blob", f"{PIN}:src/render/kernel/output.rs").decode()
max_ulp = int(re.search(r"pub const MAX_DEPTH_ULP: i64 = (\d+);", out_rs).group(1))
crit = {
    "max_depth_ulp_constant": max_ulp,
    "rgb_beyond_1_lsb_assert": "stats.beyond_one_lsb_pixels," in cmp_src and "0," in cmp_src,
    "depth_mismatched_assert": "diff.mismatched, 0," in cmp_src,
    "depth_max_ulp_assert": "diff.max_ulp <= MAX_DEPTH_ULP" in cmp_src,
    "depth_beyond_ulp_assert": "diff.beyond_ulp, 0," in cmp_src,
    "seg_mismatched_assert": cmp_src.count("diff.mismatched, 0,") >= 2,
    "shadow_depth_seg_bitwise": "shadow 开/关 depth 必须位级相等" in cmp_src and "shadow 开/关 seg 必须逐像素相等" in cmp_src,
    "shadow_reversed_zero": "reversed, 0," in cmp_src,
    "shadow_changed_positive": "changed > 0" in cmp_src,
    "depthseg_all_cameras": "for camera in 0..scene.model.ncam()" in cmp_src,
}
check("criteria.zero_relaxation", max_ulp == 4 and all(v for k, v in crit.items() if k != "max_depth_ulp_constant"),
      crit)

# 5. shadow 臂「两臂两跑 sha 相等」覆盖：on 臂 rgb/depth/seg 有；off 臂缺 seg
on_block = src[src.find("fn shadow_toggle_consistency"):src.find("fn depth_seg_cross_path_consistency")]
off_after = on_block[on_block.find("let again_off"):]
off_checks = re.findall(r'"shadow-off 两跑 (\w+) sha"', off_after)
check("shadow.both_arms_two_run_sha",
      sorted(re.findall(r'"shadow-on 两跑 (\w+) sha"', on_block)) == ["RGB", "depth", "seg"]
      and sorted(off_checks) == ["RGB", "depth", "seg"],
      {"shadow_on_checks": re.findall(r'"shadow-on 两跑 (\w+) sha"', on_block),
       "shadow_off_checks": off_checks,
       "note": "off 臂只断言 RGB/depth 两跑 sha，缺 seg（判据写 both arms two-run sha equal）"})

# 6. 不引用 U1b 过时 .cu 注释
files = new_tests + ["scripts/validate/render-bvh/freeze-guards-sdf.ps1"]
hits = {}
for p in files:
    t = txt(PIN, p)
    hits[p] = [kw for kw in ("host 仍拒绝", "今天不可达", "reject_sdf_geometry", "S7b3 不含 SDF 求交") if kw in t]
check("comments.no_stale_cu_reference", not any(hits.values()), hits)

# 7. 冻结器：行数/ASCII/PSParser/硬断言/实体顺序/未执行/输入计数
freeze = txt(PIN, "scripts/validate/render-bvh/freeze-guards-sdf.ps1")
psp = (HERE / "materials" / "freezer-check.txt").read_text(encoding="utf-8", errors="replace")
order = ["render_sdf_reference", "render_e2e", "render_kernel_rgb", "render_kernel_depth_seg",
         "render_kernel_flags", "render_r10_multi_world", "render_skybox_reference",
         "render_context_lifecycle", "render_mesh_bvh"]
order_ok = all(f"'{e}'" in freeze for e in order) and freeze.count("entity = '") == 9
asserts = ("expected 9 frozen exes" in freeze and "expected 83 read-only inputs" in freeze
           and "expected 92 frozen entries" in freeze)
# 现盘按冻结器规则重数 inputs
root = REPO / "fixtures/render-bvh"


def walk(sub):
    return [p for p in (root / sub).rglob("*") if p.is_file()]


sdf_n = len(walk("scene-sdf"))
hf_n = len(walk("scene-hfield"))
legacy_n = len([p for p in root.rglob("*") if p.is_file()
                and not any(str(p).replace("\\", "/").find(f"fixtures/render-bvh/{d}/") >= 0
                            for d in ("scene-shadow", "scene-hfield", "scene-sdf"))])
shadow_n = len([p for p in walk("scene-shadow")
                if re.match(r".*/scene-shadow/capture(-exact)?/", p.as_posix())])
check("freezer.static_and_counts",
      freeze.count("\n") == 167 and not [b for b in blob(PIN, "scripts/validate/render-bvh/freeze-guards-sdf.ps1") if b > 127]
      and "psparser_errors=0" in psp and asserts and order_ok
      and (sdf_n, hf_n, legacy_n, shadow_n) == (10, 10, 47, 16)
      and not (REPO / "scripts/validate/render-bvh/guards-sdf-validate.json").exists(),
      {"lines": freeze.count("\n"), "psparser": psp.strip(), "hard_asserts_present": asserts,
       "entity_order_matches_material": order_ok,
       "inputs_recount": {"scene_sdf": sdf_n, "scene_hfield": hf_n, "legacy": legacy_n,
                           "scene_shadow": shadow_n, "total": sdf_n + hf_n + legacy_n + shadow_n},
       "freeze_not_executed": not (REPO / "scripts/validate/render-bvh/guards-sdf-validate.json").exists()})

# 8. 门禁自跑：unit 137 result 行 1109/0/397；receipt 写 392 ignored → 数字错误
unit = (HERE / "logs" / "unit.log").read_text(encoding="utf-8", errors="replace")
results = re.findall(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", unit, re.M)
passed = sum(int(r[0]) for r in results)
failed = sum(int(r[1]) for r in results)
ignored = sum(int(r[2]) for r in results)
gs = (HERE / "materials" / "gates-summary.txt").read_text(encoding="utf-8")
exits = dict(re.findall(r"GATE (\S+) exit=(\d+)", gs))
nr = (HERE / "logs" / "no-run.log").read_text(encoding="utf-8", errors="replace")
receipt = json.loads(txt(PIN, EV + "/receipt.json"))
receipt_unit_text = receipt["gates"]["unit"]
check("gates.self_run_and_receipt_number",
      len(results) == 137 and (passed, failed, ignored) == (1109, 0, 397)
      and all(v == "0" for v in exits.values()) and len(exits) == 6
      and nr.splitlines()[-1] == "EXIT=0" and "Finished" in nr
      and f"{ignored} ignored" in receipt_unit_text,
      {"result_lines": len(results), "passed": passed, "failed": failed, "ignored": ignored,
       "gate_exits": exits, "no_run_last": nr.splitlines()[-1],
       "receipt_unit_text": receipt_unit_text,
       "note": "证据 gate-unit.json 也是 397 ignored；仅 receipt.gates.unit 文案写 392（U1b 旧值）"})

# 9. 证据 manifest / 编码
man = txt(PIN, EV + "/manifest.txt")
rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
ok = sum(1 for s, n, p in rows if sha(blob(PIN, p)) == s and len(blob(PIN, p)) == int(n))
ev_files = git("ls-tree", "-r", "--name-only", PIN, EV).decode().splitlines()
utf16 = [p for p in ev_files if blob(PIN, p)[:2] in (b"\xff\xfe",) or blob(PIN, p).startswith(b"\xef\xbb\xbf")]
check("evidence.manifest",
      len(rows) == 12 and ok == 12 and len(ev_files) == 13 and not utf16,
      {"rows": len(rows), "match": ok, "files": len(ev_files), "utf16_or_bom": utf16})

# 10. 卡面检查 / validate
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
check("card.checks_and_validate",
      chk.returncode == 0 and bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5),
      {"check_exit": chk.returncode, "validate": vm.groups() if vm else None})

problems = [
    {"id": "P1", "severity": "需修(轻微, 证据数字)", "check": "gates.self_run_and_receipt_number",
     "finding": "receipt.gates.unit 写 '1109 passed / 0 failed / 392 ignored'，实际（我自跑与证据 gate-unit.json 一致）是 "
                "**1109 passed / 0 failed / 397 ignored**（U1b 后 392 + 本片 5 条新 ignored GPU 用例）。",
     "fix": "receipt.gates.unit 改为 397 ignored（或写 tests_passed/ignored 两字段）；不重跑。"},
    {"id": "P2", "severity": "需修(轻微, 判据覆盖)", "check": "shadow.both_arms_two_run_sha",
     "finding": "shadow 臂判据写「两臂 two-run sha 相等」，代码只对 on 臂断言 rgb/depth/seg 两跑 sha；"
                "off 臂只断言 rgb/depth，**缺 seg 两跑 sha**（render_sdf_reference.rs 的 shadow_toggle_consistency）。",
     "fix": "在 off 臂补一条 seg 两跑 sha 断言（1 行）；或把判据措辞收窄为「shadow 两臂 rgb/depth 两跑 sha 相等」。"
            "改动后需重跑 host/unit 门禁（unit 仍 1109/0/397，行数 362）。"},
]
for c in checks:
    c["ok"] = c["passed"]

out = {"schema": 1, "task": CARD + "-check", "reviewed_task": CARD, "pin": PIN,
       "card_head": "e83ed63b", "checks": checks, "problems": problems,
       "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                   "problems": len(problems), "blockers": 0}}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
