#!/usr/bin/env python3
"""SDF any-hit cull latent fix（2026-10-30-render-sdf-anyhit-cull-latent-fix）的 QA 独立复核重算。

只读：全部结论由 git 对象 / 已提交证据 / 本目录 logs 现算，不采信被核证据的结论行。
被核 pin：f63834e0（工作+证据+卡）；基线 e6eb0c9a；复核卡 revision.before = 23277f36（本脚本自身提交之前）。

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-anyhit-cull-latent-fix-check/recompute.py
输出：同目录 recompute.json（checks + problems）与 materials/*.txt。
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
BASE = "e6eb0c9a"
HEAD = "f63834e0"
CARD = "2026-10-30-render-sdf-anyhit-cull-latent-fix"
CHECK_CARD = "2026-10-30-render-sdf-anyhit-cull-latent-fix-check"
EV = "docs/development-evidence/render-bvh/sdf-anyhit-cull-latent-fix"
EVQ = "docs/development-evidence/render-bvh/sdf-anyhit-cull-latent-fix-check"
SRC_CU = "src/render/kernel/cuda/render_common.cu"
SRC_RS = "src/render/kernel/megakernel.rs"
OLD = ["e68e3412", "0b947492", "268275", "268,275"]
NEW_SRC = "be4ef01b1ae45188ac512c91cfef60244a1f8b6670307b245ae1719a34cf5f5e"
NEW_PTX = "4dae03ffb3fe82214da930714c7fd1ccb6a2c8d5c5eb7cc8e489624b4e7dd085"
NEW_PTX_BYTES = 268305
AA_SRC = "0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76"
AA_PTX = "b28e155c6e8b0f1c8c05d36054036764d2c437ae3c58e09c8d10360666ec8a70"
AA_PTX_BYTES = 27536

checks = []


def check(cid, ok, detail):
    checks.append({"id": cid, "passed": bool(ok), "detail": detail})
    return bool(ok)


def git(*args, binary=False):
    cmd = ["git"] + list(args)
    out = subprocess.run(cmd, cwd=REPO, capture_output=True)
    if out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {out.stderr.decode('utf-8', 'replace')[:400]}")
    return out.stdout if binary else out.stdout.decode("utf-8")


def blob(rev, path):
    return git("cat-file", "blob", f"{rev}:{path}", binary=True)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lines(b):
    return b.count(b"\n")


# ---------------------------------------------------------------- 1. 范围
diff = git("diff", "--name-status", f"{BASE}..{HEAD}")
changed = [l for l in diff.splitlines() if l.strip()]
paths = [l.split("\t", 1)[1] for l in changed]
src = [p for p in paths if p.startswith("src/")]
forbidden = [p for p in paths if p.startswith(("tests/", "fixtures/")) or p in ("Cargo.toml", "Cargo.lock")]
check(
    "scope.changed_paths",
    len(paths) == 19 and src == [SRC_CU, SRC_RS] and not forbidden,
    {"total": len(paths), "src": src, "forbidden": forbidden,
     "normify": len([p for p in paths if p.startswith(".normify/")]),
     "evidence": len([p for p in paths if p.startswith(EV + "/")])},
)

# ---------------------------------------------------------------- 2. 行数 / diff 规模
lb, lh = lines(blob(BASE, SRC_CU)), lines(blob(HEAD, SRC_CU))
rb, rh = lines(blob(BASE, SRC_RS)), lines(blob(HEAD, SRC_RS))
numstat = {}
for l in git("diff", "--numstat", f"{BASE}..{HEAD}", "--", "src").splitlines():
    add, dele, path = l.split("\t")
    numstat[path] = (int(add), int(dele))
tot_add = sum(v[0] for v in numstat.values())
tot_del = sum(v[1] for v in numstat.values())
check(
    "src.line_counts",
    (lb, lh, rb, rh) == (410, 414, 409, 440) and numstat[SRC_CU] == (8, 4) and numstat[SRC_RS] == (35, 4)
    and (tot_add, tot_del) == (43, 8),
    {"render_common.cu": [lb, lh], "megakernel.rs": [rb, rh], "numstat": numstat,
     "total": {"insertions": tot_add, "deletions": tot_del}},
)

# ---------------------------------------------------------------- 3. 内核源码拼接身份
mod_rs = blob(HEAD, "src/render/kernel/mod.rs").decode("utf-8")
m = re.search(r"pub const MEGAKERNEL_SOURCE: &str = concat!\((.*?)\);", mod_rs, re.S)
parts = re.findall(r'include_str!\("([^"]+)"\)', m.group(1)) if m else []
kernel_dir = "src/render/kernel/"
# pure-posix 归一（mod.rs 里 ../bvh/cuda/scene.cu 相对 src/render/kernel/）
norm = []
for rel in parts:
    parts_out = []
    for seg in pathlib.PurePosixPath(kernel_dir, rel).parts:
        if seg == "..":
            parts_out.pop()
        else:
            parts_out.append(seg)
    norm.append("/".join(parts_out))
cat_base = b"".join(blob(BASE, p) for p in norm)
cat_head = b"".join(blob(HEAD, p) for p in norm)
check(
    "kernel.source_sha256",
    len(norm) == 6
    and sha(cat_base) == "e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c"
    and sha(cat_head) == NEW_SRC
    and len(cat_base) == 88190
    and len(cat_head) == 88785,
    {"parts": norm, "base_bytes": len(cat_base), "base_sha256": sha(cat_base),
     "head_bytes": len(cat_head), "head_sha256": sha(cat_head)},
)

# ---------------------------------------------------------------- 4. aa_resolve 不变
aa_base, aa_head = blob(BASE, "src/render/kernel/cuda/aa_resolve.cu"), blob(HEAD, "src/render/kernel/cuda/aa_resolve.cu")
check(
    "kernel.aa_resolve_unchanged",
    aa_base == aa_head and sha(aa_head) == AA_SRC and len(aa_head) == 2364,
    {"bytes": len(aa_head), "sha256": sha(aa_head), "same_as_base": aa_base == aa_head},
)

# ---------------------------------------------------------------- 5. NVRTC 纯编译门（本复核自跑）
ki_path = HERE / "logs" / "kernel-identity.log"
ki = ki_path.read_text(encoding="utf-8", errors="replace") if ki_path.exists() else ""
mk = re.search(r"s7b3 megakernel: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
aar = re.search(r"s7b3 aa_resolve: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
ok_line = "test result: ok. 1 passed; 0 failed" in ki
check(
    "kernel.ptx_identity_selfrun",
    bool(mk and aar) and mk.group(1) == NEW_SRC and mk.group(2) == NEW_PTX
    and int(mk.group(3)) == NEW_PTX_BYTES and aar.group(1) == AA_SRC
    and aar.group(2) == AA_PTX and int(aar.group(3)) == AA_PTX_BYTES and ok_line,
    {"megakernel": mk.groups() if mk else None, "aa_resolve": aar.groups() if aar else None,
     "test_ok": ok_line,
     "note": "NVRTC 仅编译、无设备执行；PATH 加 target/toolchains/cuda-12.8.1/bin 后运行"},
)

# ---------------------------------------------------------------- 6. cull 语义 / host 拒绝
cu = blob(HEAD, SRC_CU).decode("utf-8")
rs = blob(HEAD, SRC_RS).decode("utf-8")
cull_line = [l.strip() for l in cu.splitlines() if "bool cull_this" in l]
host_guard = [l.strip() for l in rs.splitlines() if "fn reject_sdf_geometry" in l or 'Unsupported("S7b3 不含 SDF 求交")' in l]
sem = {
    "cull_line": cull_line[0] if cull_line else None,
    "gtype8_not_intersected_on_device": "else if (gtype != 8)" in cu,
    "d_init_minus_one": "float d = -1.0f;" in cu,
    "host_guard": host_guard,
    "host_reject_called": "reject_sdf_geometry(&ints(fields, \"geom_type\")?)" in rs,
    "host_only_tests": "mod sdf_geometry_guard_tests" in rs and "#[cfg(test)]" in rs,
}
check(
    "cull.semantics_latent",
    bool(cull_line) and "gtype == 8" in cull_line[0] and sem["gtype8_not_intersected_on_device"]
    and sem["d_init_minus_one"] and sem["host_reject_called"] and sem["host_only_tests"]
    and "S7b3 不含 SDF 求交" in rs,
    sem,
)

# ---------------------------------------------------------------- 7. 旧身份引用清单
scan = set()
per_pattern = {}
for pat in OLD:
    out = git("grep", "-n", "-F", pat, HEAD)
    n = 0
    for l in out.splitlines():
        _rev, path, lineno, _content = l.split(":", 3)
        scan.add((path, int(lineno)))
        n += 1
    per_pattern[pat] = n
refs_path = f"{EV}/materials/kernel-identity-refs.txt"
refs = blob(HEAD, refs_path).decode("utf-8")
listed = set()
for l in refs.splitlines():
    if l.startswith("#") or l.startswith("summary:"):
        continue
    parts = l.split(":", 2)
    if len(parts) == 3:
        listed.add((parts[0], int(parts[1])))
listed_files = {p for p, _ in listed}
scan_no_self = {x for x in scan if x[0] != refs_path}
missing = sorted(scan_no_self - listed)
live = [x for x in scan if x[0].startswith(("src/", "tests/", "scripts/"))]
missing_cat = {}
for p, _ln in missing:
    key = p.split("/")[0]
    missing_cat[key] = missing_cat.get(key, 0) + 1
refs_txt = "\n".join([
    "# 旧内核身份引用清单复核（QA 独立重算，只读）",
    f"# 被核 pin: {HEAD}；基线 {BASE}",
    f"# 模式: {', '.join(OLD)}（并集按 (path,line) 去重）",
    "",
    f"清单文件自称: total_lines=59 total_files=31 docs=55/27 fixtures=4 code=0",
    f"清单实测: entries={len(listed)} files={len(listed_files)} "
    f"docs={len([p for p in listed_files if p.startswith('docs/')])} "
    f"fixtures={len([p for p in listed_files if p.startswith('fixtures/')])} "
    f"other={len([p for p in listed_files if not p.startswith(('docs/', 'fixtures/'))])}",
    f"清单过期项（列了但树上没有）: 0" if not (listed - scan_no_self) else f"清单过期项: {len(listed - scan_no_self)}",
    "",
    f"全仓实测（排除清单文件自身）: entries={len(scan_no_self)} files={len({p for p, _ in scan_no_self})}",
    f"其中未列入清单: entries={len(missing)} files={len({p for p, _ in missing})}；按顶层目录: {json.dumps(missing_cat, sort_keys=True)}",
    f"活体断言（src/tests/scripts）: {len(live)} 行",
    "",
    "未列入清单的明细（path:line）:",
] + [f"  {p}:{ln}" for p, ln in missing] + ["", "逐模式命中行数: " + json.dumps(per_pattern, sort_keys=True)])
(HERE / "materials" / "refs-inventory.txt").write_text(refs_txt + "\n", encoding="utf-8", newline="\n")
check(
    "refs.inventory",
    len(listed) == 59 and len(listed_files) == 31 and not (listed - scan_no_self) and not live
    and len(scan_no_self) == 59 and not missing,
    {"listed": [len(listed), len(listed_files)], "stale_in_list": len(listed - scan_no_self),
     "full_repo_excl_list": [len(scan_no_self), len({p for p, _ in scan_no_self})],
     "unlisted": [len(missing), len({p for p, _ in missing})], "unlisted_by_dir": missing_cat,
     "live_assertions": len(live), "per_pattern": per_pattern,
     "note": "清单自身 59/31 自洽且 0 过期；README 的「全仓检索 59/31」口径不含 .normify 卡、docs 下 3 个 recompute.py、本片 README/receipt"},
)

# ---------------------------------------------------------------- 8. 证据 manifest / 编码
man = blob(HEAD, f"{EV}/manifest.txt").decode("utf-8")
rows = []
for l in man.splitlines():
    if l and not l.startswith("#"):
        sha_s, size_s, path = l.split(None, 2)
        rows.append((sha_s, int(size_s), path))
man_ok = 0
bad = []
for sha_s, size_s, path in rows:
    b = blob(HEAD, path)
    if sha(b) == sha_s and len(b) == size_s:
        man_ok += 1
    else:
        bad.append(path)
self_registered = any(p == f"{EV}/manifest.txt" for _, _, p in rows)
ev_files = git("ls-tree", "-r", "--name-only", HEAD, EV).splitlines()
enc = {}
for p in ev_files:
    b = blob(HEAD, p)
    enc[p] = {"bytes": len(b), "cr": b.count(b"\r"), "utf16": b[:2] in (b"\xff\xfe", b"\xfe\xff"),
              "bom_utf8": b.startswith(b"\xef\xbb\xbf")}
utf16 = [p for p, v in enc.items() if v["utf16"]]
hyg = "\n".join([
    "# 证据文件编码 / manifest 复核（只读，按 committed blob）",
    "",
    f"manifest 行数: {len(rows)}；self-register: {self_registered}；manifest 自身 bytes={len(man)}",
    f"sha256+bytes 全部一致: {man_ok}/{len(rows)}" + (f"；不一致: {bad}" if bad else ""),
    "",
    "证据目录文件（共 %d，manifest 不自登记 => 记录 %d）:" % (len(ev_files), len(rows)),
] + [f"  {p.split('/')[-1]:35s} bytes={v['bytes']:9d} CR={v['cr']:3d} utf16={v['utf16']}" for p, v in sorted(enc.items())]
  + ["", f"UTF-16 blob: {len(utf16)} -> {utf16}"])
(HERE / "materials" / "evidence-hygiene.txt").write_text(hyg + "\n", encoding="utf-8", newline="\n")
check(
    "evidence.manifest",
    len(rows) == 10 and man_ok == 10 and not self_registered and len(utf16) == 2,
    {"rows": len(rows), "manifest_ok": man_ok, "self_registered": self_registered,
     "evidence_files": len(ev_files), "utf16_blobs": utf16},
)

# ---------------------------------------------------------------- 9. 门禁自跑（本复核）
unit = (HERE / "logs" / "unit.log").read_text(encoding="utf-8", errors="replace")
results = re.findall(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", unit, re.M)
passed = sum(int(r[1]) for r in results)
failed = sum(int(r[2]) for r in results)
ignored = sum(int(r[3]) for r in results)
new_tests = [n for n in ("sdf_geometry_stays_unsupported", "non_sdf_geometry_passes_the_guard") if n in unit]
gates_summary = (HERE / "materials" / "gates-summary.txt").read_text(encoding="utf-8") if (HERE / "materials" / "gates-summary.txt").exists() else ""
gate_exits = dict(re.findall(r"GATE (\S+) exit=(\d+)", gates_summary))
check(
    "gates.self_run",
    len(results) == 136 and passed == 1108 and failed == 0 and ignored == 392
    and len(new_tests) == 2 and all(v == "0" for v in gate_exits.values()) and len(gate_exits) == 4,
    {"result_lines": len(results), "passed": passed, "failed": failed, "ignored": ignored,
     "new_host_tests_seen": new_tests, "gate_exits": gate_exits},
)

# ---------------------------------------------------------------- 10. 卡面门禁 / 状态（只读命令）
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
val_txt = (val.stdout or b"").decode("utf-8", "replace")
val_err = re.search(r"错误(\d+) 警告(\d+)", val_txt)
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
chk_txt = (chk.stdout or b"").decode("utf-8", "replace")
check(
    "card.validate_and_checks",
    val.returncode == 0 and val_err and (int(val_err.group(1)), int(val_err.group(2))) == (0, 5)
    and chk.returncode == 0 and "全部检查通过" in chk_txt,
    {"validate": val_err.groups() if val_err else None, "validate_exit": val.returncode,
     "check_exit": chk.returncode, "check_says_pass": "全部检查通过" in chk_txt,
     "note": "证据 materials/validate.txt 为门禁时点 0/6（含 fingerprint-drift），done 刷指纹后现为 0/5"},
)

# ---------------------------------------------------------------- 11. 提交映射（message vs 内容）
d1 = git("show", "--stat", "--oneline", "d132336c")
d2 = git("show", "--stat", "--oneline", "f63834e0")
d1_paths = git("show", "--name-only", "--format=", "d132336c").split()
d2_paths = git("show", "--name-only", "--format=", "f63834e0").split()
mapping = {
    "d132336c_message": d1.splitlines()[0],
    "d132336c_has_src": [SRC_CU, SRC_RS] == [p for p in d1_paths if p.startswith("src/")],
    "d132336c_paths": len(d1_paths),
    "f63834e0_message": d2.splitlines()[0],
    "f63834e0_paths": d2_paths,
    "parent_ok": git("rev-parse", "d132336c^").strip().startswith(BASE),
}
check(
    "commit.mapping_documented",
    mapping["d132336c_has_src"] and mapping["f63834e0_paths"] == [f".normify/changes/{CARD}.json"]
    and mapping["parent_ok"] and len(d1_paths) == 19,
    mapping,
)

# ---------------------------------------------------------------- 12. 证据门禁来源（gate-*.txt vs .normify/runs）
prov = {}
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    ev = json.loads(blob(HEAD, f"{EV}/logs/gate-{g}.txt").decode("utf-8"))
    run = json.loads(blob(HEAD, f".normify/runs/{CARD}/{g}.json").decode("utf-8"))
    prov[g] = {
        "evidence_at": ev.get("at"), "run_at": run.get("at"),
        "evidence_scope_hash": (ev.get("scope_hash") or "")[:12],
        "run_scope_hash": (run.get("scope_hash") or "")[:12],
        "same_run": ev.get("scope_hash") == run.get("scope_hash"),
        "unit_ev": ev.get("tests"), "unit_run": run.get("tests"),
    }
same = [g for g, v in prov.items() if v["same_run"]]
prov_txt = "\n".join(["# 证据 gate-*.txt 与 .normify/runs/*.json 来源比对（只读，按 committed blob）", ""]
                     + [f"{g}: evidence_at={v['evidence_at']} scope={v['evidence_scope_hash']} | "
                        f"runs_at={v['run_at']} scope={v['run_scope_hash']} | same_run={v['same_run']} | "
                        f"unit ev={v['unit_ev']} run={v['unit_run']}" for g, v in prov.items()])
(HERE / "materials" / "gate-provenance.txt").write_text(prov_txt + "\n", encoding="utf-8", newline="\n")
check(
    "evidence.gate_provenance",
    len(same) == 4,
    {"same_run_gates": same, "detail": prov,
     "note": "README §4 称 gate-*.txt 为 .normify/runs 的转写快照；实测四次均不同（证据 = 04:25:2x 那次，runs = 04:26:2x 那次），结论同为全过、unit 1108/0/392"},
)

# ---------------------------------------------------------------- 13. no-run 日志含 exit 码
nr_path = HERE / "logs" / "no-run.log"
nr = nr_path.read_text(encoding="utf-8", errors="replace") if nr_path.exists() else ""
ev_nr = blob(HEAD, f"{EV}/logs/no-run.txt")
ev_nr_txt = ev_nr.decode("utf-16-le") if ev_nr[:2] in (b"\xff\xfe", b"\xfe\xff") else ev_nr.decode("utf-8", "replace")
nrun_txt = "\n".join([
    "# --no-run 证据日志复核",
    "",
    f"被核证据 logs/no-run.txt: bytes={len(ev_nr)} encoding=UTF-16LE CR={ev_nr.count(b'\r')} "
    f"has_Finished={'Finished' in ev_nr_txt} has_exit_marker={'exit' in ev_nr_txt.lower()}",
    f"卡面 acceptance 要求：`--no-run` 日志含 exit 码与 `Finished`；receipt.no_run.exit={json.loads(blob(HEAD, f'{EV}/receipt.json'))['no_run']['exit']}",
    "",
    f"QA 自跑 logs/no-run.log: has_Finished={'Finished' in nr} exit_line="
    + repr([l for l in nr.splitlines() if "EXIT" in l or l.startswith("exit")][:3]),
])
(HERE / "materials" / "no-run-exit.txt").write_text(nrun_txt + "\n", encoding="utf-8", newline="\n")
check(
    "evidence.no_run_exit_code",
    ("exit" in ev_nr_txt.lower()) and ("Finished" in ev_nr_txt) and "EXIT=" in nr and "Finished" in nr,
    {"evidence_no_run_has_exit_marker": "exit" in ev_nr_txt.lower(),
     "evidence_no_run_has_Finished": "Finished" in ev_nr_txt,
     "evidence_receipt_exit": json.loads(blob(HEAD, f"{EV}/receipt.json"))["no_run"]["exit"],
     "qa_selfrun_has_exit_line": "EXIT=" in nr,
     "note": "卡面 acceptance：`--no-run` 日志含 exit 码与 Finished；QA 自跑日志已按此格式（EXIT=0）"},
)

# ---------------------------------------------------------------- 14. unchanged 路径
unchanged = ["docs/development-evidence/render-bvh/r08-rgb-hetero",
             "docs/development-evidence/render-bvh/r08-rgb-hetero-window-material",
             "docs/development-evidence/render-bvh/r02-skybox"]
touched = [p for p in paths if any(p.startswith(u) for u in unchanged)]
check(
    "boundary.unchanged_paths",
    not touched,
    {"declared_unchanged": unchanged, "touched": touched,
     "note": "R08-S3 窗口证据 r08-rgb-hetero-validate 未列入卡面 unchanged，但同样 0 改动（按本次 diff 文件集合核）"},
)

# ---------------------------------------------------------------- 15. 证据目录卫生（本片）
self_files = sorted(str(p.relative_to(REPO)).replace("\\", "/") for p in (HERE).rglob("*") if p.is_file())
check(
    "qa.evidence_scope",
    all(p.startswith(EVQ) for p in self_files),
    {"files": len(self_files), "dir": EVQ},
)

problems = [
    {
        "id": "P1",
        "severity": "需修(轻微)",
        "check": "evidence.no_run_exit_code",
        "finding": f"{EV}/logs/no-run.txt 无 exit 码（卡面 acceptance 明文要求「日志含 exit 码与 Finished」）；"
                   "receipt 记 exit=0 但日志内无 exit 行，且日志为 PowerShell stderr 包装的 UTF-16LE。",
        "fix": "重定向时追加 `; \"EXIT=$LASTEXITCODE\"` 把 exit 码写进日志（或按卡面把要求改成「receipt 记 exit」）；不需重跑测试。",
    },
    {
        "id": "P2",
        "severity": "需修(轻微)",
        "check": "evidence.gate_provenance",
        "finding": "README §4 称 `logs/gate-*.txt` 是 `.normify/runs/2026-10-30-render-sdf-anyhit-cull-latent-fix/*.json` 的转写快照；"
                   "实测 4/4 都不是同一 run（证据 at=04:25:2x、runs at=04:26:2x；scope_hash/input_hash/log_sha256 全不同）。"
                   "两次都全过、unit 均 1108/0/392，结论不受影响。",
        "fix": "改措辞为「门禁第一次运行的 JSON 快照（权威记录见 .normify/runs 的第二次运行）」，或重转写第二次 run；不需重跑。",
    },
    {
        "id": "P3",
        "severity": "需修(轻微)",
        "check": "refs.inventory",
        "finding": "README §3 的「全仓检索旧三值 59 行/31 文件」不可按全仓口径复现：独立扫描（排除清单文件自身）= 79 行/41 文件；"
                   "未列入的 20 行/10 文件 = 5 个 `.normify` 卡、docs 下 3 个 recompute.py（8 行）、本片 README(4)+receipt(3)。"
                   "清单自身 59/31 自洽且 0 过期；「src/tests/scripts 活体断言 0」我独立确认成立。",
        "fix": "把「全仓」改为实际口径（docs+fixtures，且排除 docs 下 .py 复算脚本与本片自述文件），或补全清单并重算 59/31；不需重跑。",
    },
]
for c in checks:
    c["ok"] = c["passed"]

out = {
    "schema": 1,
    "task": CHECK_CARD,
    "reviewed_task": CARD,
    "pin": HEAD,
    "baseline": BASE,
    "card_head": "23277f36",
    "checks": checks,
    "problems": problems,
    "summary": {
        "checks_total": len(checks),
        "checks_passed": sum(1 for c in checks if c["passed"]),
        "problems": len(problems),
        "blockers": 0,
    },
}
(HERE / "recompute.json").write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n",
                                     encoding="utf-8", newline="\n")
print(json.dumps(out["summary"], ensure_ascii=False))
for c in checks:
    print(("PASS " if c["passed"] else "FAIL ") + c["id"])
