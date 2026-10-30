#!/usr/bin/env python3
"""QA 独立复核：SDF host 解锁（U1b）。

只读：结论由 committed blob / 本卡自跑日志现算；不采信证据结论行。
被核 pin：af96c048；复核卡：2026-10-31-render-sdf-host-unlock-check（revision.before = abbb19ed）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-host-unlock-check/recompute.py
"""
import hashlib
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "af96c048"
BASE = "af96c048^"
CARD = "2026-10-31-render-sdf-host-unlock"
EV = "docs/development-evidence/render-bvh/sdf-host-unlock"
KERNEL_PARTS = ["src/render/bvh/cuda/scene.cu", "src/render/kernel/cuda/render_math.cu",
                "src/render/kernel/cuda/render_common.cu", "src/render/kernel/cuda/render_shadow.cu",
                "src/render/kernel/cuda/render_shade.cu", "src/render/kernel/cuda/render_megakernel.cu"]
U1_SRC = "778216b23e64e1211ad5c6e9087550851c4ee79a6d8385c5d3f0fd05016f40f5"
U1_PTX = "34f91147f8861601f4d96602223d282b570be38a305b83511e43fa4fa76b408d"

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


# 1. 范围（只 1 个 src 文件；.cu/fixtures/tests/scripts/Cargo 零改）
paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{BASE}..{PIN}").decode().splitlines() if l.strip()]
src = [p for p in paths if p.startswith("src/")]
check("scope.pin_commit",
      len(paths) == 20 and src == ["src/render/kernel/megakernel.rs"]
      and not [p for p in paths if p.startswith(("tests/", "fixtures/", "scripts/", "Cargo"))
               or p.startswith("src/render/kernel/cuda/")],
      {"paths": len(paths), "src": src,
       "evidence": len([p for p in paths if p.startswith(EV + "/")])})

# 2. 行数 / numstat
mk_b, mk_p = blob(BASE, "src/render/kernel/megakernel.rs"), blob(PIN, "src/render/kernel/megakernel.rs")
num = [l for l in git("diff", "--numstat", f"{BASE}..{PIN}", "--", "src").decode().splitlines() if l.strip()]
check("src.line_counts",
      (mk_b.count(b"\n"), mk_p.count(b"\n")) == (440, 406) and num == ["1\t35\tsrc/render/kernel/megakernel.rs"],
      {"lines": [mk_b.count(b"\n"), mk_p.count(b"\n")], "numstat": num})

# 3. guard 移除（可执行引用 0；记录 .cu 注释残留）
src_hits = subprocess.run(["git", "grep", "-n", "-F", "reject_sdf_geometry", PIN, "--", "src"], cwd=REPO, capture_output=True).stdout.decode("utf-8", "replace")
hits = [l for l in src_hits.splitlines() if l.strip()]
rs_hits = [l for l in hits if l.split(":", 2)[1].endswith(".rs")]
cu_hits = [l for l in hits if l.split(":", 2)[1].endswith(".cu")]
err_hits = subprocess.run(["git", "grep", "-n", "-F", "S7b3 不含 SDF 求交", PIN, "--", "src"], cwd=REPO, capture_output=True).stdout.decode("utf-8", "replace").strip()
check("guard.removed",
      not rs_hits and not err_hits and len(cu_hits) == 2,
      {"rust_executable_refs": len(rs_hits), "cu_comment_refs": len(cu_hits),
       "old_error_string_refs": len([l for l in err_hits.splitlines() if l.strip()]),
       "cu_refs_detail": cu_hits,
       "note": "executable refs 0；render_common.cu 仍有 2 行注释提到已删除的 guard（语义已过时，见 problems P1）"})

# 4. 其它拒绝原样
code = mk_p.decode()
kept = {k: (k in code) for k in ("S7b3 不含 splat", "light_mode=0", "use_precomputed_rays",
                                 "enabled_geom_ids", "splat_count", "ray_count", "nsamples")}
check("host.other_rejections_kept", all(kept.values()), kept)

# 5. 内核身份不变（.cu 零改 + concat sha + aa + NVRTC 自跑）
cat = b"".join(blob(PIN, p) for p in KERNEL_PARTS)
aa = blob(PIN, "src/render/kernel/cuda/aa_resolve.cu")
ki = (HERE / "logs" / "kernel-identity.log").read_text(encoding="utf-8", errors="replace")
mk = re.search(r"s7b3 megakernel: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
aar = re.search(r"s7b3 aa_resolve: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
check("kernel.identity_unchanged",
      sha(cat) == U1_SRC and aa.count(b"\n") == blob(PIN, "src/render/kernel/cuda/aa_resolve.cu").count(b"\n")
      and git("diff", "--stat", f"{BASE}..{PIN}", "--", "src/render/kernel/cuda").strip() == b""
      and bool(mk) and mk.group(1) == U1_SRC and mk.group(2) == U1_PTX and int(mk.group(3)) == 268589
      and bool(aar) and int(aar.group(3)) == 27536,
      {"concat": {"bytes": len(cat), "sha256": sha(cat)}, "cuda_diff_empty": True,
       "nvrtc_selfrun": {"megakernel": mk.groups() if mk else None, "aa": aar.groups() if aar else None}})

# 6. 门禁自跑（本卡日志）+ guard 单测已删
unit = (HERE / "logs" / "unit.log").read_text(encoding="utf-8", errors="replace")
results = re.findall(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", unit, re.M)
passed = sum(int(r[0]) for r in results)
failed = sum(int(r[1]) for r in results)
ignored = sum(int(r[2]) for r in results)
gs = (HERE / "materials" / "gates-summary.txt").read_text(encoding="utf-8")
exits = dict(re.findall(r"GATE (\S+) exit=(\d+)", gs))
nr = (HERE / "logs" / "no-run.log").read_text(encoding="utf-8", errors="replace")
check("gates.self_run",
      len(results) == 136 and (passed, failed, ignored) == (1106, 0, 392)
      and all(v == "0" for v in exits.values()) and len(exits) == 6
      and nr.splitlines()[-1] == "EXIT=0" and "Finished" in nr
      and "sdf_geometry_guard_tests" not in unit,
      {"result_lines": len(results), "passed": passed, "failed": failed, "ignored": ignored,
       "gate_exits": exits, "no_run_last": nr.splitlines()[-1],
       "guard_tests_absent": "sdf_geometry_guard_tests" not in unit,
       "delta_vs_u1": "1108 -> 1106 (removed 2 host-only tests)"})

# 7. 证据 manifest / removed-guard.txt
man = txt(PIN, EV + "/manifest.txt")
rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
ok = sum(1 for s, n, p in rows if sha(blob(PIN, p)) == s and len(blob(PIN, p)) == int(n))
files = git("ls-tree", "-r", "--name-only", PIN, EV).decode().splitlines()
utf16 = [p.split("/")[-1] for p in files if blob(PIN, p)[:2] in (b"\xff\xfe",)]
rg = txt(PIN, EV + "/materials/removed-guard.txt")
check("evidence.manifest_and_removed_guard",
      len(rows) == 12 and ok == 12 and len(files) == 13 and not utf16
      and "HEAD:src/render/kernel/megakernel.rs:55" in rg and "working tree after this slice" in rg,
      {"manifest": {"rows": len(rows), "match": ok, "files": len(files)}, "utf16": utf16,
       "removed_guard_has_old_hits": "HEAD:src/render/kernel/megakernel.rs:55" in rg})

# 8. 卡面：回滚声明 / 检查 / validate
card = json.loads(txt(PIN, ".normify/changes/" + CARD + ".json"))
note = card.get("note") or ""
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
check("card.rollback_and_checks",
      "git revert" in note and "U4" in note and chk.returncode == 0
      and bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5),
      {"rollback_in_note": "git revert" in note, "check_exit": chk.returncode,
       "validate": vm.groups() if vm else None,
       "rollback_also_in": ["commit message", "README §4", "receipt"]})

problems = [
    {
        "id": "P1",
        "severity": "需修(轻微, 证据口径)",
        "check": "guard.removed",
        "finding": "README §2 写「残留引用：工作树 reject_sdf_geometry / S7b3 不含 SDF 求交 = 0」，但 pin 上 src 仍有 2 处 "
                   "`reject_sdf_geometry` 引用（render_common.cu:391、:397 的注释，removed-guard.txt 自己已列出）；"
                   "且这两行注释语义已被 U1b 反转（现在写「host 仍拒绝 8 / 本片改动今天不可达」）。可执行引用确实为 0、旧错误串 0 命中。",
        "fix": "把 README §2 改成「可执行引用 0（.rs 0 命中、`S7b3 不含 SDF 求交` 0 命中）；`.cu` 注释仍有 2 行引用，因本片零 .cu 改动"
               "（内核身份必须不变）保留，语义待下一次内核重基线时更新」；不改代码、不重跑。",
    }
]
for c in checks:
    c["ok"] = c["passed"]

out = {"schema": 1, "task": CARD + "-check", "reviewed_task": CARD, "pin": PIN,
       "card_head": "abbb19ed", "checks": checks, "problems": problems,
       "summary": {"checks_total": len(checks), "checks_passed": sum(1 for c in checks if c["passed"]),
                   "problems": len(problems), "blockers": 0}}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
