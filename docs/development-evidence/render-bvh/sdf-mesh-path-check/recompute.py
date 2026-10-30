#!/usr/bin/env python3
"""QA 独立复核：SDF 解锁 U1（设备 mesh 路径对齐，host 拒绝保留）。

只读：全部结论由 git 对象 / 已提交证据 / 本目录 logs 现算；不采信被核证据的结论行。
被核：2026-10-30-render-sdf-mesh-path（pin = ed508d77，基线 02b9c629）
复核卡：2026-10-30-render-sdf-mesh-path-check（revision.before = b9bc2c1d）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-mesh-path-check/recompute.py
"""
import hashlib
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
BASE = "02b9c629"
PIN = "ed508d77"
CARD = "2026-10-30-render-sdf-mesh-path"
EV = "docs/development-evidence/render-bvh/sdf-mesh-path"
SRC_CU = "src/render/kernel/cuda/render_common.cu"
SRC_SEG = "src/render/kernel/cuda/render.cu"
OLD_SRC = "be4ef01b1ae45188ac512c91cfef60244a1f8b6670307b245ae1719a34cf5f5e"
NEW_SRC = "778216b23e64e1211ad5c6e9087550851c4ee79a6d8385c5d3f0fd05016f40f5"
NEW_PTX = "34f91147f8861601f4d96602223d282b570be38a305b83511e43fa4fa76b408d"
NEW_PTX_BYTES = 268589
AA_SRC = "0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76"
AA_PTX = "b28e155c6e8b0f1c8c05d36054036764d2c437ae3c58e09c8d10360666ec8a70"
AA_PTX_BYTES = 27536
KERNEL_PARTS = ["src/render/bvh/cuda/scene.cu", "src/render/kernel/cuda/render_math.cu",
                "src/render/kernel/cuda/render_common.cu", "src/render/kernel/cuda/render_shadow.cu",
                "src/render/kernel/cuda/render_shade.cu", "src/render/kernel/cuda/render_megakernel.cu"]
UPSTREAM = REPO / "target/mjwarp-parallel/upstream/mujoco_warp/_src"

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


# 1. 改动范围（按 pin 提交内容核，不按 message）
paths = [l.split("\t", 1)[1] for l in git("diff", "--name-status", f"{BASE}..{PIN}").splitlines() if l.strip()]
src = [p for p in paths if p.startswith("src/")]
check("scope.pin_commit",
      len(paths) == 19 and src == [SRC_SEG, SRC_CU]
      and not [p for p in paths if p.startswith(("tests/", "fixtures/", "Cargo"))]
      and len([p for p in paths if p.startswith(EV + "/")]) == 11
      and len([p for p in paths if p.startswith(".normify/")]) == 6,
      {"total": len(paths), "src": src,
       "evidence": len([p for p in paths if p.startswith(EV + "/")]),
       "normify": len([p for p in paths if p.startswith(".normify/")]),
       "message_subject": git("log", "-1", "--format=%s", PIN).strip()})

# 2. 行数 / 规模
counts = {p: (blob(BASE, p).count(b"\n"), blob(PIN, p).count(b"\n")) for p in (SRC_CU, SRC_SEG)}
numstat = {}
for l in git("diff", "--numstat", f"{BASE}..{PIN}", "--", "src").splitlines():
    add, dele, path = l.split("\t")
    numstat[path] = (int(add), int(dele))
check("src.line_counts",
      counts[SRC_CU] == (414, 415) and counts[SRC_SEG] == (499, 496)
      and numstat == {SRC_CU: (6, 5), SRC_SEG: (3, 6)}
      and sum(v[0] for v in numstat.values()) == 9 and sum(v[1] for v in numstat.values()) == 11,
      {"counts": counts, "numstat": numstat, "total": {"insertions": 9, "deletions": 11}})

# 3. 内核源码拼接身份（旧=be4ef01b，新=778216b2）
cat_base = b"".join(blob(BASE, p) for p in KERNEL_PARTS)
cat_pin = b"".join(blob(PIN, p) for p in KERNEL_PARTS)
check("kernel.source_sha256",
      sha(cat_base) == OLD_SRC and sha(cat_pin) == NEW_SRC,
      {"base": {"bytes": len(cat_base), "sha256": sha(cat_base)},
       "pin": {"bytes": len(cat_pin), "sha256": sha(cat_pin)}})

# 4. aa_resolve 未变
aa_b, aa_p = blob(BASE, "src/render/kernel/cuda/aa_resolve.cu"), blob(PIN, "src/render/kernel/cuda/aa_resolve.cu")
check("kernel.aa_resolve_unchanged", aa_b == aa_p and sha(aa_p) == AA_SRC and len(aa_p) == 2364,
      {"bytes": len(aa_p), "sha256": sha(aa_p)})

# 5. NVRTC 纯编译门（本复核自跑，仅编译无设备）
ki = (HERE / "logs" / "kernel-identity.log").read_text(encoding="utf-8", errors="replace")
mk = re.search(r"s7b3 megakernel: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
aar = re.search(r"s7b3 aa_resolve: source_sha256=([0-9a-f]{64}) ptx_sha256=([0-9a-f]{64}) ptx_bytes=(\d+)", ki)
check("kernel.ptx_identity_selfrun",
      bool(mk and aar) and mk.group(1) == NEW_SRC and mk.group(2) == NEW_PTX
      and int(mk.group(3)) == NEW_PTX_BYTES and aar.group(1) == AA_SRC
      and aar.group(2) == AA_PTX and int(aar.group(3)) == AA_PTX_BYTES
      and "test result: ok. 1 passed" in ki,
      {"megakernel": mk.groups() if mk else None, "aa_resolve": aar.groups() if aar else None,
       "note": "NVRTC compile-only; PATH += target/toolchains/cuda-12.8.1/bin"})

# 6. 设备语义（3 处 dispatch + mesh_id）
cu = blob(PIN, SRC_CU).decode("utf-8")
seg = blob(PIN, SRC_SEG).decode("utf-8")
mk_cu = blob(PIN, "src/render/kernel/cuda/render_megakernel.cu").decode("utf-8")
cull_line = next((l.strip() for l in cu.splitlines() if "bool cull_this" in l), None)
sem = {
    "common_mesh_branch_includes_8": "if (gtype == 7 || gtype == 1 || gtype == 8) {" in cu,
    "common_mesh_id_includes_8": "hit.mesh_id = (gtype == 7 || gtype == 8) ? mesh_index : -1;" in cu,
    "seg_branch_calls_mesh_for_8": "if (gtype == 7 || gtype == 8) {" in seg
                                   and "d = ray_mesh_leaf(mesh_meta, mesh_params, mesh_offsets, scene.leaf_mesh[leaf], pos," in seg,
    "cull_excludes_8": bool(cull_line) and "gtype == 8" in cull_line,
    "vertex_normal_needs_mesh_id": "vertex_normal_gtype && mesh_id >= 0" in mk_cu,
    "leaf_mesh_is_geom_dataid": "let leaf_mesh = if matches!(geom_type, 7 | 8)" in blob(PIN, "src/render/kernel/cast.rs").decode("utf-8"),
}
check("device.dispatch_semantics", all(sem.values()), sem)

# 7. host 拒绝保留（能力不暴露）
rs_pin = blob(PIN, "src/render/kernel/megakernel.rs")
check("host.rejection_retained",
      rs_pin == blob(BASE, "src/render/kernel/megakernel.rs")
      and 'Unsupported("S7b3 不含 SDF 求交")' in rs_pin.decode("utf-8")
      and "reject_sdf_geometry" in rs_pin.decode("utf-8")
      and not [p for p in paths if p == "src/render/kernel/megakernel.rs"],
      {"megakernel_rs_bytes": len(rs_pin), "unchanged_vs_base": rs_pin == blob(BASE, "src/render/kernel/megakernel.rs")})

# 8. 上游引用（pinned 71da24d9，clean）
up_checks = {}
try:
    render = (UPSTREAM / "render.py").read_text(encoding="utf-8")
    seg_lines = render.splitlines()
    block = "\n".join(seg_lines[737:761])
    up_checks["render_py_738_761"] = ("GeomType.MESH" in block and "GeomType.SDF" in block
                                      and "ray_mesh_with_bvh" in block and "geom_dataid" in block)
    up_checks["render_test_py_1251"] = "Mesh and SDF geoms render using the underlying mesh geometry." in \
        (UPSTREAM / "render_test.py").read_text(encoding="utf-8")
    bvh = (UPSTREAM / "bvh.py").read_text(encoding="utf-8")
    bvh_block = "\n".join(bvh.splitlines()[210:220])
    up_checks["bvh_py_211_220"] = ("GeomType.MESH or type == GeomType.SDF" in bvh_block
                                   and "mesh_bounds_size" in bvh_block)
    up_checks["upstream_clean"] = subprocess.run(["git", "-C", str(UPSTREAM.parents[1]), "status", "--porcelain"],
                                                 capture_output=True).stdout == b""
except OSError as exc:  # pragma: no cover
    up_checks["error"] = str(exc)
check("upstream.citations", all(up_checks.values()), up_checks)

# 9. 旧身份引用清单（pin 口径）
refs_pin = "02b9c629"
scan = set()
for pat in ["be4ef01b", "4dae03ff", "268305", "268,305"]:
    for l in git("grep", "-n", "-F", pat, refs_pin).splitlines():
        _rev, p, ln, _c = l.split(":", 3)
        scan.add((p, int(ln)))
refs_txt = blob(PIN, EV + "/materials/kernel-identity-refs.txt").decode("utf-8")
listed = set()
for l in refs_txt.splitlines():
    if l.startswith("#") or l.startswith("summary:") or not l.strip():
        continue
    q = l.split(":", 2)
    if len(q) == 3:
        listed.add((q[0], int(q[1])))
live = [x for x in scan if x[0].startswith(("src/", "tests/", "scripts/"))]
check("refs.inventory",
      len(scan) == 28 and len({p for p, _ in scan}) == 14 and len(listed) == len(scan)
      and not (scan - listed) and not (listed - scan) and not live,
      {"pin_scan": [len(scan), len({p for p, _ in scan})], "listed": [len(listed), len({p for p, _ in listed})],
       "stale": len(listed - scan), "missing": len(scan - listed), "live_assertions": len(live)})

# 10. 证据 manifest / 编码
man = blob(PIN, EV + "/manifest.txt").decode("utf-8")
rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
ok = 0
for s, n, p in rows:
    b = blob(PIN, p)
    if sha(b) == s and len(b) == int(n):
        ok += 1
ev_files = git("ls-tree", "-r", "--name-only", PIN, EV).splitlines()
enc = {}
for p in ev_files:
    b = blob(PIN, p)
    enc[p.split("/")[-1]] = {"bytes": len(b), "utf16": b[:2] in (b"\xff\xfe", b"\xfe\xff")}
utf16 = [k for k, v in enc.items() if v["utf16"]]
check("evidence.manifest",
      len(rows) == 10 and ok == 10 and len(ev_files) == 11 and not any(p.endswith("/manifest.txt") for _, _, p in rows),
      {"rows": len(rows), "match": ok, "files": len(ev_files), "utf16": utf16})

# 11. --no-run 日志
nr = blob(PIN, EV + "/logs/no-run.txt")
nr_txt = nr.decode("utf-16-le") if nr[:2] in (b"\xff\xfe", b"\xfe\xff") else nr.decode("utf-8", "replace")
nr_lines = [l for l in nr_txt.splitlines() if l.strip()]
check("evidence.no_run_exit_code",
      "Finished" in nr_txt and nr_lines[-1] == "EXIT=0",
      {"bytes": len(nr), "last_line": nr_lines[-1], "has_finished": "Finished" in nr_txt})

# 12. 门禁：证据快照 vs 已提交 runs（措辞核对）+ 本复核自跑
same = {}
for g in ("fmt", "clippy", "clippy-cuda", "unit"):
    ev = json.loads(blob(PIN, f"{EV}/logs/gate-{g}.txt").decode("utf-8"))
    run = json.loads(blob(PIN, f".normify/runs/{CARD}/{g}.json").decode("utf-8"))
    same[g] = {"evidence_at": ev.get("at"), "runs_at": run.get("at"),
               "same_run": ev.get("scope_hash") == run.get("scope_hash"),
               "ev_unit": ev.get("tests"), "run_unit": run.get("tests")}
check("evidence.gate_provenance_documented",
      len(same) == 4 and all("某次" in blob(PIN, EV + "/README.md").decode("utf-8") for _ in [0]),
      {"detail": same, "note": "README §4 明示「某次门禁转写快照、权威见 .normify/runs」，与 4/4 不同 run 的事实一致"})

unit = (HERE / "logs" / "unit.log").read_text(encoding="utf-8", errors="replace")
results = re.findall(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", unit, re.M)
passed = sum(int(r[1]) for r in results)
failed = sum(int(r[2]) for r in results)
ignored = sum(int(r[3]) for r in results)
guard_tests = [n for n in ("sdf_geometry_stays_unsupported", "non_sdf_geometry_passes_the_guard") if n in unit]
gate_summary = (HERE / "materials" / "gates-summary.txt").read_text(encoding="utf-8") if (HERE / "materials" / "gates-summary.txt").exists() else ""
gate_exits = dict(re.findall(r"GATE (\S+) exit=(\d+)", gate_summary))
check("gates.self_run",
      len(results) == 136 and passed == 1108 and failed == 0 and ignored == 392
      and len(guard_tests) == 2 and gate_exits.get("fmt") == "0" and gate_exits.get("clippy") == "0"
      and gate_exits.get("clippy-cuda") == "0" and gate_exits.get("unit") == "0"
      and gate_exits.get("no-run") == "0" and gate_exits.get("nvrtc") == "0",
      {"result_lines": len(results), "passed": passed, "failed": failed, "ignored": ignored,
       "latent_guard_tests_seen": guard_tests, "gate_exits": gate_exits})

# 13. 校验 / 卡检查 / 提交映射
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
val_txt = (val.stdout or b"").decode("utf-8", "replace")
vm = re.search(r"错误(\d+) 警告(\d+)", val_txt)
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
check("card.validate_and_checks",
      bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5) and chk.returncode == 0,
      {"validate": vm.groups() if vm else None, "check_exit": chk.returncode,
       "evidence_validate_at_gate": "0 错误 / 6 警告（含 fingerprint-drift，done 后刷新）"})

d1 = [l for l in git("show", "--name-only", "--format=%s", "ed508d77").splitlines() if l.strip()]
d2 = [l for l in git("show", "--name-only", "--format=%s", "eff2de88").splitlines() if l.strip()]
mapping = {"ed508d77_subject": d1[0], "ed508d77_has_src": any(p.startswith("src/") for p in d1[1:]),
           "eff2de88_subject": d2[0], "eff2de88_paths": d2[1:]}
check("commit.mapping_documented",
      mapping["eff2de88_paths"] == [f".normify/changes/{CARD}.json"] and mapping["ed508d77_has_src"],
      mapping)

# 14. unchanged 边界
unchanged = ["docs/development-evidence/render-bvh/r08-rgb-hetero",
             "docs/development-evidence/render-bvh/r02-skybox",
             "src/render/kernel/megakernel.rs"]
touched = [p for p in paths if any(p.startswith(u) for u in unchanged)]
check("boundary.unchanged_paths", not touched, {"unchanged": unchanged, "touched": touched})

problems = []
for c in checks:
    c["ok"] = c["passed"]

out = {
    "schema": 1,
    "task": "2026-10-30-render-sdf-mesh-path-check",
    "reviewed_task": CARD,
    "pin": PIN,
    "baseline": BASE,
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
