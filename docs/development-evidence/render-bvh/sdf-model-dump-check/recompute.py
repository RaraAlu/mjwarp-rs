#!/usr/bin/env python3
"""QA 独立复核：SDF 场景产品侧模型 dump（U4a）。

只读：结论由 committed blob / 本卡自跑复现现算；不采信证据结论行。
被核 pin：60541da3；复核卡：2026-10-31-render-sdf-model-dump-check（revision.before = abbb19ed）

用法（仓库根目录）：python docs/development-evidence/render-bvh/sdf-model-dump-check/recompute.py
"""
import hashlib
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
PIN = "60541da3"
BASE = "4932f7f3^"
CARD = "2026-10-31-render-sdf-model-dump"
EV = "docs/development-evidence/render-bvh/sdf-model-dump"
DUMPS = {"model-312.json": "tests/support/render_bvh/sdf/model-312.json",
         "model-mixed-312.json": "tests/support/render_bvh/sdf/model-mixed-312.json"}
EXPECT = {
    "model-312.json": {"bytes": 53849, "sha256": "7de15ff458aa8f6b1c026bbed5d9b1fbaf8f5dd39b385bab0b79d5c289b40d36"},
    "model-mixed-312.json": {"bytes": 54205, "sha256": "60698f6baaeff8dcc11e725e56242bd5d6f947741cb7070b4e1f32ec13765578"},
}
TOOL_SHA = "cd91e099659ebda3f51a84d5392f7d726c0c61378bfa2487215ddd00b2de2970"
TOOL_BYTES = 161280

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


# 1. dump 身份（committed blob 现算）
dump_report = {}
for name, path in DUMPS.items():
    b = blob(PIN, path)
    d = json.loads(b)
    arr = d["arrays"]
    mv = arr["mesh_vert"]["values"]
    dump_report[name] = {
        "path": path, "bytes": len(b), "sha256": sha(b),
        "mujoco_version": d["mujoco_version"], "mujoco_version_int": d["mujoco_version_int"],
        "model_file": d["model_file"], "source_xml": d["source_xml"], "loaded_mjb": d["loaded_mjb"],
        "counts": d["counts"],
        "geom_type": arr["geom_type"]["values"], "geom_dataid": arr["geom_dataid"]["values"],
        "geom_size": arr["geom_size"]["values"],
        "mesh_vert_count": len(mv) // 3,
        "mesh_bounds": [[min(mv[0::3]), max(mv[0::3])], [min(mv[1::3]), max(mv[1::3])], [min(mv[2::3]), max(mv[2::3])]],
        "cam_resolution": arr["cam_resolution"]["values"],
        "light_castshadow": arr["light_castshadow"]["values"],
    }
check("dumps.identity",
      all(dump_report[n]["bytes"] == EXPECT[n]["bytes"] and dump_report[n]["sha256"] == EXPECT[n]["sha256"]
          for n in DUMPS),
      {n: {k: v for k, v in dump_report[n].items() if k in ("bytes", "sha256", "mujoco_version", "loaded_mjb")}
       for n in DUMPS})

# 2. 确定性（本卡用 pinned 环境重跑两次，同路径）
det_txt = (HERE / "materials" / "rerun-determinism.txt").read_text(encoding="utf-8")
det_ok = True
det_detail = {}
for name, exp in EXPECT.items():
    shas = re.findall(rf"{re.escape(name)}: run\d exit=0 sha256=([0-9a-f]{{64}}) bytes=(\d+)", det_txt)
    det_detail[name] = {"runs": [s for s, _ in shas], "byte_runs": [int(b) for _, b in shas]}
    det_ok = det_ok and len(shas) == 2 and all(s == exp["sha256"] for s, _ in shas) \
        and all(int(b) == exp["bytes"] for _, b in shas)
check("determinism.rerun",
      det_ok and TOOL_SHA in det_txt and "unchanged (byte-identical)" in det_txt,
      det_detail)

# 3. 语义：counts 与 dump-counts.json 一致；SDF 规则前提（mesh bounds vs geom_size）
counts = json.loads(blob(PIN, EV + "/materials/dump-counts.json"))
sem_ok = True
sem_detail = {}
for name in DUMPS:
    d = dump_report[name]
    ev = counts[name]
    sem_detail[name] = {
        "counts_match": d["counts"] == ev["counts"],
        "geom_type_match": d["geom_type"] == ev["geom_type"],
        "dataid_match": d["geom_dataid"] == ev["geom_dataid"],
        "nmesh_ok": d["counts"]["nmesh"] == 1 and d["counts"]["nmeshvert"] == 8 and d["counts"]["nmeshface"] == 12,
        "sdf_dataid_points_mesh0": d["geom_dataid"][1] == 0 and d["geom_type"][1] == 8,
        "mesh_bounds_pm_0_35": all(abs(abs(v) - 0.349999994) < 1e-8 for axis in d["mesh_bounds"] for v in axis),
        "sdf_geom_size_pm_0_35": all(abs(v - 0.349999994) < 1e-8 for v in d["geom_size"][3:6]),
        "ncam_nlight_nq": [d["counts"]["ncam"], d["counts"]["nlight"], d["counts"]["nq"]],
        "cam_resolution": d["cam_resolution"],
        "mujoco_version_ok": d["mujoco_version"] == "3.12.0" and d["mujoco_version_int"] == 3012000,
    }
    sem_ok = sem_ok and all(v for k, v in sem_detail[name].items()
                            if k in ("counts_match", "geom_type_match", "dataid_match", "nmesh_ok",
                                     "sdf_dataid_points_mesh0", "mesh_bounds_pm_0_35",
                                     "sdf_geom_size_pm_0_35", "mujoco_version_ok"))
    sem_ok = sem_ok and sem_detail[name]["ncam_nlight_nq"] == [2, 2, 0] \
        and sem_detail[name]["cam_resolution"] == [320, 240, 240, 180]
check("dumps.semantics", sem_ok, sem_detail)

# 4. dumper 工具身份（target 下的离线工具）
tool_path = REPO / "target/render-bvh/s0a/compile_scene_312.exe"
tool_ok = tool_path.exists() and sha(tool_path.read_bytes()) == TOOL_SHA and tool_path.stat().st_size == TOOL_BYTES
runtime = (REPO / "target/toolchains/mujoco-3.12.0/package/bin/mujoco.dll").exists()
check("tool.identity", tool_ok and runtime,
      {"tool_exists": tool_path.exists(), "tool_sha256": sha(tool_path.read_bytes()) if tool_path.exists() else None,
       "tool_bytes": tool_path.stat().st_size if tool_path.exists() else None,
       "mujoco_3_12_runtime": runtime,
       "note": "dumper 是 offline 工具（target 下），不参与产品 targets/tests/gates"})

# 5. 范围：只 tests/support JSon + 证据 + 卡；不动 src/fixtures/Cargo/scripts/*.rs
paths = [l.split("\t")[-1] for l in git("diff", "--name-status", f"{BASE}..{PIN}").decode().splitlines() if l.strip()]
bad = [p for p in paths if p.startswith(("src/", "fixtures/", "Cargo", "scripts/")) or p.endswith(".rs")]
added = [p for p in paths if p in DUMPS.values()]
check("scope.pin_commit",
      len(paths) == 10 and not bad and sorted(added) == sorted(DUMPS.values())
      and len([p for p in paths if p.startswith(EV + "/")]) == 7,
      {"paths": len(paths), "bad": bad, "dumps_added": added,
       "evidence": len([p for p in paths if p.startswith(EV + "/")])})

# 6. .mjb 只在 target（未入库）
tracked_mjb = [p for p in git("ls-files").decode().splitlines()
               if p.lower().endswith(".mjb") and ("sdf-model-dump" in p or p.startswith("tests/support"))]
target_mjb = sorted(p.name for p in (REPO / "target/render-bvh/sdf-model-dump").glob("*.mjb"))
check("intermediate.mjb_not_committed",
      not tracked_mjb and target_mjb == ["scene-sdf-mixed.mjb", "scene-sdf.mjb"],
      {"tracked_mjb": tracked_mjb, "target_mjb": target_mjb})

# 7. 证据 manifest / 编码
man = txt(PIN, EV + "/manifest.txt")
rows = [l.split(None, 2) for l in man.splitlines() if l and not l.startswith("#")]
ok = sum(1 for s, n, p in rows if sha(blob(PIN, p)) == s and len(blob(PIN, p)) == int(n))
files = git("ls-tree", "-r", "--name-only", PIN, EV).decode().splitlines()
utf16 = [p.split("/")[-1] for p in files if blob(PIN, p)[:2] in (b"\xff\xfe",)]
check("evidence.manifest",
      len(rows) == 6 and ok == 6 and len(files) == 7 and not utf16,
      {"rows": len(rows), "match": ok, "files": len(files), "utf16": utf16})

# 8. 卡面检查 / validate
chk = subprocess.run(["normify", "check", CARD], cwd=REPO, capture_output=True)
val = subprocess.run(["normify", "validate", "--fingerprints"], cwd=REPO, capture_output=True)
vm = re.search(r"错误(\d+) 警告(\d+)", (val.stdout or b"").decode("utf-8", "replace"))
check("card.checks_and_validate",
      chk.returncode == 0 and bool(vm) and (int(vm.group(1)), int(vm.group(2))) == (0, 5),
      {"check_exit": chk.returncode, "validate": vm.groups() if vm else None})

problems = []
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
