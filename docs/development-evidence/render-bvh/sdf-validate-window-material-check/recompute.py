#!/usr/bin/env python3
"""QA 独立复核：U4 SDF 验证窗材料（条件式例外 amend 版）。

只读：结论由 committed blob 现算；不采信证据结论行。
被核 pin：285ba51f（amend：用户条件式例外 + 92 分解对账 + body 零编译口径）
复核卡：2026-10-31-render-sdf-validate-window-material-check（revision.before = ca88e04f）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-validate-window-material-check/recompute.py
"""
import hashlib
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "285ba51f"
PREV = "800f4a61"
CARD = "2026-10-31-render-sdf-validate-window-material"
EV = "docs/development-evidence/render-bvh/sdf-validate-window-material"

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


# 1. 范围：amend 只动材料 + 卡；entity-table 未动
BASE = "285ba51f^"
paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{BASE}..{PIN}").decode().splitlines() if l.strip()]
check("scope.amend_only_material",
      len(paths) == 6
      and not [p for p in paths if p.startswith(("src/", "tests/", "scripts/", "fixtures/", "Cargo"))]
      and blob(PREV, EV + "/materials/entity-table.json") == blob(PIN, EV + "/materials/entity-table.json"),
      {"paths": paths, "entity_table_unchanged": blob(PREV, EV + "/materials/entity-table.json")
       == blob(PIN, EV + "/materials/entity-table.json")})

# 2. 9 实体定稿（1 new + 8 regressions；换入 mesh_bvh，移出 camera_rays / r08_cameras）
et = json.loads(txt(PIN, EV + "/materials/entity-table.json"))
ents = [e["entity"] for e in et["entities"]]
expected = ["render_sdf_reference", "render_e2e", "render_kernel_rgb", "render_kernel_depth_seg",
            "render_kernel_flags", "render_r10_multi_world", "render_skybox_reference",
            "render_context_lifecycle", "render_mesh_bvh"]
check("entities.final_selection",
      ents == expected and len(ents) == 9
      and [e["entity"] for e in et["entities"] if e["source"] == "new"] == ["render_sdf_reference"]
      and "render_camera_rays" not in ents and "render_r08_cameras" not in ents
      and "render_mesh_bvh" in ents,
      {"entities": ents, "order_ok": ents == expected,
       "new": [e["entity"] for e in et["entities"] if e["source"] == "new"],
       "dropped_absent": "render_camera_rays" not in ents and "render_r08_cameras" not in ents})

# 3. 92 分解 + 与 S3 的 68 对账（材料口径；实际文件数在窗口冻结时逐条核）
ep = json.loads(txt(PIN, EV + "/materials/expected-products.json"))
guards = et["guards_expected"]
breakdown_ok = (guards["frozen_exes"], guards["guard_inputs"], guards["total"]) == (9, 83, 92) \
    and "scene-sdf 10" in guards["input_breakdown"] and "scene-hfield 10" in guards["input_breakdown"] \
    and "legacy 47" in guards["input_breakdown"] and "scene-shadow capture 16" in guards["input_breakdown"] \
    and 10 + 10 + 47 + 16 == 83
rec = json.dumps(ep, ensure_ascii=False)
check("guards.92_and_s3_reconciliation",
      breakdown_ok and "+10 scene-sdf" in rec and "legacy 42 -> 47" in rec
      and ep["guards"]["aggregate"].startswith("generated at freeze")
      and "not preset" in ep["guards"]["aggregate"],
      {"counts": [guards["frozen_exes"], guards["guard_inputs"], guards["total"]],
       "breakdown": guards["input_breakdown"],
       "reconciliation_present": "+10 scene-sdf" in rec and "legacy 42 -> 47" in rec,
       "aggregate_not_preset": ep["guards"]["aggregate"]})

# 4. 命令：材料声明「5 条逐字」，实际 window-plan 只有 4 条完整 powershell 命令
wp = txt(PIN, EV + "/materials/window-plan.txt")
cmd_lines = [l.strip() for l in wp.splitlines() if l.strip().startswith("powershell -NoProfile")]
readme = txt(PIN, EV + "/README.md")
receipt = json.loads(txt(PIN, EV + "/receipt.json"))
claim_5 = ("五条命令" in readme) or ("5 verbatim" in json.dumps(receipt, ensure_ascii=False))
exception_cmd_verbatim = any("-MaxUtil 60" in l and "monitor-until-clean.ps1" in l for l in cmd_lines)
check("commands.five_verbatim_as_claimed",
      len(cmd_lines) == 5 and claim_5 and exception_cmd_verbatim,
      {"verbatim_powershell_commands": len(cmd_lines), "claims_five": claim_5,
       "exception_monitor_command_verbatim": exception_cmd_verbatim,
       "note": "材料声明 5 条逐字命令；实际 4 条（monitor/freeze/guards-only/body），条件式例外 monitor 命令只描述了 -MaxUtil 60 与新 CSV，未给逐字命令/CSV 名"})

# 5. 判据零放宽（与 amend 前逐字相同；且等于 R09/R05 口径）
crit = ep["criteria"]
crit_ok = (crit["rgb"] == "<= 1 LSB per channel (beyond_1_lsb=0, max_byte_diff<=1)"
           and crit["depth"].startswith("bitwise-first")
           and crit["seg"].startswith("exact")
           and "no numerical reference claim" in crit["shadow_arm"]
           and "bitwise" in crit["depth_seg_cross_path"]
           and ep["criteria"] == json.loads(txt(PREV, EV + "/materials/expected-products.json"))["criteria"])
check("criteria.zero_relaxation", crit_ok, {"criteria": crit, "unchanged_vs_prev": crit_ok})

# 6. 条件式例外协议五要素 + authoring_exception_used=false + 现场按序执行
cp = ep["conditional_protocol"]
step2 = cp["step2_exception_only_if_step1_fails"]
six = (step2["mode"] == "validation_only_exception_util_lt_60"
       and step2["approved_by"].startswith("user via Chief")
       and step2["authoring_exception_used"] is False
       and len(step2["five_elements"]) == 5 and "not_a_normal_threshold_pass=true" in step2["five_elements"]
       and "original_normal_threshold_attempts_retained = the failed regular CSV" in step2["five_elements"]
       and "failed CSV + snapshot + verdict" in step2["steps"][0]
       and "both monitors" in cp["evidence_rule"]
       and "never written as a normal pass" in cp["evidence_rule"])
check("exception.conditional_protocol", six,
      {"mode": step2["mode"], "approved_by": step2["approved_by"],
       "authoring_exception_used": step2["authoring_exception_used"],
       "five_elements": step2["five_elements"], "evidence_rule": cp["evidence_rule"]})

# 7. 签字：user only / 三件 / 已批准；时间戳占位 "00:5x" 计数
ds = ep["double_sign"]
sig_txt = json.dumps([ep["double_sign"], receipt.get("signature", {})], ensure_ascii=False)
placeholder = sum(txt(PIN, p).count("00:5x") for p in (
    EV + "/README.md", EV + "/receipt.json",
    EV + "/materials/window-plan.txt", EV + "/materials/expected-products.json"))
check("signature.user_and_artifacts",
      ds["approved_by"] == "user via Chief" and ds["decision"].startswith("approved")
      and ds["required_on"] == ["this window plan", "pre-window baseline snapshot",
                                "the 92-entry guards identity after freeze"]
      and placeholder == 0,
      {"approved_by": ds["approved_by"], "decision": ds["decision"], "required_on": ds["required_on"],
       "placeholder_timestamp_occurrences": placeholder,
       "note": "签字时间用 '2026-10-31T00:5x+08:00' 占位（10 处），不是具体时间戳"})

# 8. shell / 常规路径 / body 零编译口径
shell_ok = all("powershell" in l for l in cmd_lines) and "Windows PowerShell 5.1" in json.dumps(ep, ensure_ascii=False)
regular_ok = ("default MaxUtil=40" in wp and "do NOT pass -OpenUtilLimit or -MaxUtil 60" in wp
              and "-MaxUtil 60" not in cmd_lines[0])
zero_compile = ("body zero compile" in wp and "body starts" in wp
                and "freeze_compile_note" in ep and "expected build" in ep["freeze_compile_note"]
                and "no Rust compile after the body starts" in ep["freeze_compile_note"])
check("discipline.shell_regular_zero_compile", shell_ok and regular_ok and zero_compile,
      {"shell_ok": shell_ok, "regular_first": regular_ok, "body_zero_compile_wording": zero_compile})

# 9. 证据 manifest / ASCII / 编码；receipt 自身文件表（自引用应为旧值 → P1）
man = txt(PIN, EV + "/manifest.txt")
rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
ok = sum(1 for s, n, p in rows if sha(blob(PIN, p)) == s and len(blob(PIN, p)) == int(n))
files = git("ls-tree", "-r", "--name-only", PIN, EV).decode().splitlines()
wp_b = blob(PIN, EV + "/materials/window-plan.txt")
self_entry = receipt["files"]["receipt.json"]
receipt_b = blob(PIN, EV + "/receipt.json")
self_ok = self_entry["sha256"] == sha(receipt_b) and self_entry["bytes"] == len(receipt_b)
check("evidence.manifest_and_self_entry",
      len(rows) == 5 and ok == 5 and len(files) == 6 and wp_b.count(13) == 0
      and sum(1 for b in wp_b if b > 127) == 0
      and not [p for p in files if blob(PIN, p)[:2] in (b"\xff\xfe",) or blob(PIN, p).startswith(b"\xef\xbb\xbf")]
      and self_ok,
      {"manifest": {"rows": len(rows), "match": ok, "files": len(files)},
       "window_plan_ascii": sum(1 for b in wp_b if b > 127) == 0,
       "receipt_self_entry": self_entry,
       "receipt_actual": {"sha256": sha(receipt_b), "bytes": len(receipt_b)},
       "self_entry_matches_actual": self_ok})

# 10. README 标题与实际纪律一致性（标题仍写「无例外」）
headings = [l for l in readme.splitlines() if l.startswith("## ")]
heading_claims_no_exception = any("无例外" in h for h in headings)
check("readme.heading_consistency", not heading_claims_no_exception,
      {"headings": headings, "stale_no_exception_heading": heading_claims_no_exception})

# 11. 卡面检查 / validate
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
check("card.checks_and_validate",
      chk.returncode == 0 and bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5),
      {"check_exit": chk.returncode, "validate": vm.groups() if vm else None})

problems = [
    {"id": "P1", "severity": "需修(轻微, 证据数字)", "check": "evidence.manifest_and_self_entry",
     "finding": "receipt.files 里 receipt.json 自条目仍是 amend 前的旧值：sha 4ef24623…/2078 B/53 行；"
                "实际 receipt 为 87631dba…/3825 B。manifest.txt 不含 receipt 正确值，但 receipt 自己列了过期的自引用。",
     "fix": "receipt.files 去掉 receipt.json 自条目（manifest 本身就不登记自己），或注明自引用不可自证；不重跑。"},
    {"id": "P2", "severity": "需修(轻微, 签字卫生)", "check": "signature.user_and_artifacts",
     "finding": "签字时间在 4 个文件里共 10 处写成 '2026-10-31T00:5x+08:00'（分钟位占位 x）。签字件应是具体时间戳；"
                "另：材料 amend 发生在签字之后，双签块只写「plan version sha256 记录在 receipt」，未标明用户实际批准的是哪一版文本。",
     "fix": "用 Chief 转达的准确时间（或明确写「分钟未转达」并在窗口事实里记 Chief 时间）替换；并在双签块注明签字覆盖的是协议内容（regular + conditional），"
            "计划文本 amend 由 Arch 按转达落字、非用户逐字节 review。不重跑。"},
    {"id": "P3", "severity": "需修(轻微, 措辞)", "check": "readme.heading_consistency",
     "finding": "README §2 标题仍是「窗口纪律（常规 util<40，无例外）」，但该节内容已改成条件式例外协议，标题与正文矛盾。",
     "fix": "标题改为「窗口纪律（先常规 util<40；失败才启用已签条件式 validation-only 例外）」；不重跑。"},
    {"id": "P4", "severity": "需修(轻微, 可执行性)", "check": "commands.four_or_five",
     "finding": "README §2/receipt 声明「5 条命令逐字」，window-plan 实际只有 4 条完整 powershell 命令（monitor/freeze/guards-only/body）；"
                "条件式例外的 monitor 命令只写「-Required 60 -MaxMinutes 30 -MaxUtil 60 (powershell 5.1) with a NEW CSV」，"
                "没有完整命令行、也没规定新 CSV 文件名（窗口内现场决定时容易写错/覆盖失败 CSV）。",
     "fix": "在 window-plan 里补第 5 条逐字命令（例如 -OutCsv ...monitor-pre-window-exception.csv ... -MaxUtil 60），或把「5 条」改成「4 条 + 条件式例外命令模板」；不重跑。"},
]
for c in checks:
    c["ok"] = c["passed"]

out = {"schema": 1, "task": CARD + "-check", "reviewed_task": CARD, "pin": PIN,
       "card_head": "ca88e04f", "checks": checks, "problems": problems,
       "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                   "problems": len(problems), "blockers": 0}}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
