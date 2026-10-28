"""R09-S3b secondary-ray 修复独立复核（QA；只读 + 纯 numpy 复算）。

复核对象：实现 `e17da2b7`、其后 attempt1 红项 b 的测试修正 `d501118b`、当前 HEAD `d501118b`。
本脚本独立复算（不引用 Backend/Arch 报告值）：
  1. 证据目录 EOL/BOM + manifest sha256/bytes；
  2. 关键文件行数（红线/棘轮）；
  3. capture-exact 冻结参考 sha256 与 `shadow.rs` 冻结常量对照；
  4. 上游锚点原文（pinned render.py / ray.py：any-hit 不填 normal、统一 cull 跳过零向量 normal）；
  5. 产品路径静态核对（cast_ray 的 shadow_ray 参数、两处调用、修复前 cull 行为）；
  6. attempt1 红项与像素 dump 的独立复算（1436/1451/77/4745/3537/1208；全在参考阴影集内、只偏亮、
     1195 像素完全无阴影）；
  7. 上游 cull 开关自身差异的独立复算（cull-off 单渲染 vs 冻结参考：cam1 beyond-1 LSB = 38）；
  8. dev 验收测试本体的 GPU 复跑日志（dev 诊断，非验证窗）中的 0 判据；
  9. 禁词与红线 diff。

说明：attempt1 原始窗口产物与 dev 诊断 dump 在 `target/`（未入库，属既有 provenance）；
本脚本对其只读复算并在 JSON 中标注来源，不改动、不重跑窗口。
"""
import hashlib
import json
import pathlib
import re
import subprocess

import numpy as np

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/render-bvh/r09-s3b-shadow-secondary-ray-check"
IMPL = "e17da2b7"
IMPL_PARENT = "e17da2b7^"
PRE_FIX = "cbee1127"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()

result = {"task_id": "2026-10-28-render-bvh-r09-s3b-shadow-secondary-ray-check", "head": HEAD, "checks": {}, "problems": []}


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


# 1. 证据 EOL/BOM + manifest
eol = {}
for p in sorted(EVID.rglob("*")):
    if p.is_file() and p.name != "recompute.json":
        b = p.read_bytes()
        eol[p.relative_to(EVID).as_posix()] = {"bytes": len(b), "lf": b.count(b"\n"), "cr": b.count(b"\r"), "bom": b.startswith(b"\xef\xbb\xbf")}
note("evidence_eol_bom", eol)
for rel, v in eol.items():
    if v["cr"]:
        fail(f"{rel} 含 CR")
    if v["bom"]:
        fail(f"{rel} 含 BOM")

man_path = EVID / "manifest.txt"
if man_path.exists():
    man = []
    for line in man_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        digest, size, rel = line.split()
        if (EVID / rel).exists():
            b = (EVID / rel).read_bytes()
            src = "worktree"
        else:
            b = blob(rel)
            src = "git_blob"
        ok = sha(b) == digest and len(b) == int(size)
        man.append({"path": rel, "source": src, "bytes": len(b), "claimed": int(size), "ok": ok})
        if not ok:
            fail(f"manifest 不符：{rel}")
    note("manifest_entries", man)
    note("manifest_count", len(man))
    if any(m["path"].endswith("manifest.txt") for m in man):
        fail("manifest 自引用")
else:
    note("manifest_status", "absent (first run)")

# 2. 行数
claimed = {
    "src/render/kernel/cuda/render_common.cu": 393,
    "src/render/kernel/cuda/render_megakernel.cu": 340,
    "src/render/kernel/cuda/render_shadow.cu": 30,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "tests/render_shadow_reference.rs": 307,
    "tests/support/render_bvh/shadow.rs": 416,
    "tests/support/render_bvh/shadow_scene.rs": 238,
}
lines = {}
for rel, claim in claimed.items():
    n = (REPO / rel).read_bytes().count(b"\n")
    lines[rel] = {"lf": n, "claimed": claim, "match": n == claim}
    if n != claim:
        fail(f"行数不符：{rel} 实测 {n} 声称 {claim}")
note("line_counts", lines)

# 3. 冻结参考 sha 与 shadow.rs 常量对照
cap = REPO / "fixtures/render-bvh/scene-shadow/capture-exact"
npz_sha = {}
for g in ("shadow-on", "shadow-off"):
    for n in ("rgb", "depth", "seg"):
        npz_sha[f"{g}/{n}.npz"] = sha((cap / g / f"{n}.npz").read_bytes())
note("reference_npz_sha256", npz_sha)
shadow_rs = (REPO / "tests/support/render_bvh/shadow.rs").read_text(encoding="utf-8")
frozen = re.findall(r'"([0-9a-f]{64})"', shadow_rs[:4000])
note("shadow_rs_frozen_sha256", frozen)
for g, idx in (("shadow-on", 0), ("shadow-off", 3)):
    for j, n in enumerate(("rgb", "depth", "seg")):
        exp = frozen[idx + j] if len(frozen) > idx + j else None
        actual = npz_sha[f"{g}/{n}.npz"]
        if exp != actual:
            fail(f"冻结参考 sha 不符：{g}/{n} 期望 {exp} 实测 {actual}")
note("reference_sha_match", all(frozen[i] == list(npz_sha.values())[i] for i in range(6)) if len(frozen) >= 6 else None)

# 4. 上游锚点（pinned）
PY = REPO / "target/render-bvh-precheck/venv312/Lib/site-packages/mujoco_warp/_src"
py_text = (PY / "render.py").read_text(encoding="utf-8")
ray_text = (PY / "ray.py").read_text(encoding="utf-8")
anchors = {
    "shadow_uses_anyhit_variant": "cast_ray_first_hit" in py_text and "first_hit=True" in py_text,
    "mesh_anyhit_returns_bool_no_normal": "hit = ray_mesh_with_bvh_anyhit(" in py_text and "d = 0.0 if hit else -1.0" in py_text,
    "unified_cull_has_zero_normal_note": "skips branches with a zero-vector normal (any-hit)" in py_text,
    "unified_cull_code": "if cull_backfaces and d >= 0.0 and wp.dot(ray_dir_world, n) > 0.0:" in py_text,
    "anyhit_helper_uses_mesh_query_ray_anyhit": "wp.mesh_query_ray_anyhit" in ray_text,
    "anyhit_helper_has_no_cull_param": "def ray_mesh_with_bvh_anyhit(" in ray_text and "cull_backfaces" not in ray_text.split("def ray_mesh_with_bvh_anyhit(")[1].split("def ")[0],
}
note("upstream_anchors", anchors)
for k, v in anchors.items():
    if not v:
        fail(f"上游锚点缺失：{k}")

# 5. 产品路径静态核对
common = (REPO / "src/render/kernel/cuda/render_common.cu").read_text(encoding="utf-8")
mega = (REPO / "src/render/kernel/cuda/render_megakernel.cu").read_text(encoding="utf-8")
shad = (REPO / "src/render/kernel/cuda/render_shadow.cu").read_text(encoding="utf-8")
static = {
    "cast_ray_has_shadow_ray_param": "bool cull_backfaces, bool shadow_ray)" in common,
    "mesh_inner_cull_disabled_for_shadow": "cull_backfaces && !shadow_ray" in common,
    "outer_cull_only_non_mesh_for_shadow": "cull_backfaces && !(shadow_ray && gtype == 7)" in common,
    "primary_passes_false": "cull_backfaces, false);" in mega,
    "shadow_passes_true": "cull_backfaces, true);" in shad,
    "no_other_cast_ray_call_sites": mega.count("cast_ray(") == 1 and shad.count("cast_ray(") == 1,
}
note("product_static", static)
for k, v in static.items():
    if not v:
        fail(f"产品静态核对失败：{k}")

pre = subprocess.run(["git", "show", f"{IMPL_PARENT}:src/render/kernel/cuda/render_common.cu"], cwd=REPO, capture_output=True).stdout.decode("utf-8", "replace")
pre_facts = {
    "pre_fix_mesh_cull_applied": "ray_mesh_leaf(blob.mesh_meta, blob.mesh_params, blob.mesh_offsets, mesh_index,\n                                  pos, leaf_mat, origin, direction, query_dist, cull_backfaces" in pre,
    "pre_fix_outer_cull_unconditional": "if (cull_backfaces && d >= 0.0f && dot(direction, n) > 0.0f)" in pre,
    "pre_fix_lines": pre.count("\n"),
}
note("pre_fix_code", pre_facts)
if not pre_facts["pre_fix_mesh_cull_applied"] or not pre_facts["pre_fix_outer_cull_unconditional"]:
    fail("修复前 cull 行为核对失败")

# 6. attempt1 红项 + 像素 dump 复算（target/ 原始产物，只读）
att1 = REPO / "target/s3b-validation-window/attempt1"
log = (att1 / "logs/render_shadow_reference.log").read_text(encoding="utf-8", errors="replace")
m1451 = re.search(r"cam1 rgb: pixels=(\d+) mismatched=(\d+) max_byte_diff=(\d+) beyond_1_lsb=(\d+)", log)
vis = re.search(r"product: rgb_diff=(\d+) depth_diff=(\d+) seg_diff=(\d+) \| reference: rgb_diff=(\d+) depth_diff=(\d+) seg_diff=(\d+)", log)
attempt1 = {
    "cam1_pixels": int(m1451.group(1)),
    "cam1_mismatched": int(m1451.group(2)),
    "cam1_max_byte_diff": int(m1451.group(3)),
    "cam1_beyond_1_lsb": int(m1451.group(4)),
    "product_shadow_pixels": int(vis.group(1)),
    "reference_shadow_pixels": int(vis.group(4)),
    "missing_shadow_pixels": int(vis.group(4)) - int(vis.group(1)),
    "provenance": "target/s3b-validation-window/attempt1/logs/render_shadow_reference.log (uncommitted)",
}
note("attempt1", attempt1)
if attempt1["missing_shadow_pixels"] != 1208:
    fail(f"漏判像素数不符：{attempt1['missing_shadow_pixels']}")

on_q = np.round(np.load(cap / "shadow-on/rgb.npz")["rgb_cam1"] * 255).astype(np.int32)
off_q = np.round(np.load(cap / "shadow-off/rgb.npz")["rgb_cam1"] * 255).astype(np.int32)
shadow_set = (on_q != off_q).any(axis=-1).reshape(-1)
dump_path = REPO / "target/dev-diag/shadow-cam1-mismatch.txt"
dump = dump_path.read_text(encoding="utf-8", errors="replace").strip().splitlines()
inside = brighter = eq_off = mism = beyond = 0
maxdiff = 0
for line in dump:
    f = dict(re.findall(r"(\w+)=([^ ]+)", line))
    pix = int(f["pixel"])
    exp, got = tuple(eval(f["exp"])), tuple(eval(f["got"]))
    maxdiff = max(maxdiff, int(f["maxdiff"]))
    inside += 0 if shadow_set[pix] else 1
    brighter += 0 if all(g >= e for g, e in zip(got, exp)) else 1
    eq_off += 0 if got == tuple(off_q.reshape(-1, 3)[pix]) else 1
    mism += 1 if any(g != e for g, e in zip(got, exp)) else 0
    beyond += 1 if any(abs(g - e) > 1 for g, e in zip(got, exp)) else 0
dump_facts = {
    "dump_lines": len(dump),
    "reference_shadow_pixels": int(shadow_set.sum()),
    "outside_reference_shadow_set": inside,
    "not_all_channels_got_ge_exp": brighter,
    "pixels_exactly_equal_shadow_off": len(dump) - eq_off,
    "recomputed_mismatched": mism,
    "recomputed_beyond_1_lsb": beyond,
    "max_byte_diff": maxdiff,
    "provenance": "target/dev-diag/shadow-cam1-mismatch.txt (uncommitted)",
}
note("attempt1_pixel_dump", dump_facts)
if inside or brighter or maxdiff != attempt1["cam1_max_byte_diff"]:
    fail("像素 dump 复算不符（应在参考阴影集内且只偏亮、max 与 attempt1 一致）")

# 7. 上游 cull 开关自身差异复算（dev 单渲染产物，只读）
cull_off = REPO / "target/r09-s3b-check/cull-off/rgb.npz"
if cull_off.exists():
    off = np.load(cull_off)
    on = np.load(cap / "shadow-on/rgb.npz")
    probe = {}
    for cam in ("rgb_cam0", "rgb_cam1"):
        a = np.round(on[cam] * 255).astype(np.int32)
        b = np.round(off[cam] * 255).astype(np.int32)
        d = np.abs(a - b).max(axis=-1)
        probe[cam] = {"any_diff": int((d > 0).sum()), "beyond_1_lsb": int((d > 1).sum()), "max": int(d.max())}
    note("upstream_cull_off_vs_reference", probe)
    if probe["rgb_cam1"]["beyond_1_lsb"] != 38:
        fail(f"cam1 cull 差异复算不是 38：{probe['rgb_cam1']['beyond_1_lsb']}")
else:
    note("upstream_cull_off_vs_reference", "artifact absent")

# 8. dev 验收 GPU 复跑日志（我的 dev 诊断，非验证窗）
devlog = (EVID / "logs/dev-acceptance.log").read_text(encoding="utf-8", errors="replace")
dev = {
    "shadow_on_cam0_rgb_zero": "r09-s3b[shadow-on] cam0 rgb: pixels=76800 mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in devlog,
    "shadow_on_cam1_rgb_zero": "r09-s3b[shadow-on] cam1 rgb: pixels=43200 mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in devlog,
    "shadow_on_depth_seg_zero": devlog.count("depth: compared=76800 mismatch=0 max_ulp=0 beyond_ulp=0") == 1
    and devlog.count("depth: compared=43200 mismatch=0 max_ulp=0 beyond_ulp=0") == 1
    and devlog.count("seg_mismatch=0") == 4,
    "shadow_off_pre_unlock_zero": "pre-unlock: depth_mismatch=0 max_ulp=0 seg_mismatch=0" in devlog and "pre-unlock rgb: mismatched=0 max_byte_diff=0 beyond_1_lsb=0" in devlog,
    "visibility_4745_eq_4745": "product: rgb_diff=4745 depth_diff=0 seg_diff=0 | reference: rgb_diff=4745" in devlog,
    "all_3_tests_pass": "test result: ok. 3 passed; 0 failed; 0 ignored" in devlog,
    "note": "dev diagnostic rerun of committed acceptance test; NOT the attempt2 validation window",
}
note("dev_acceptance_gpu", dev)
for k, v in dev.items():
    if k != "note" and not v:
        fail(f"dev 验收复跑核对失败：{k}")

# 9. 禁词 + 红线 diff
banned = ["".join(chr(c) for c in s) for s in (
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (31934, 24230, 36798, 26631), (21407, 29983, 25918, 34892), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807),
)]
hits = []
for f in list(EVID.rglob("*")) + [REPO / p for p in ("src/render/kernel/cuda/render_common.cu", "src/render/kernel/cuda/render_megakernel.cu", "src/render/kernel/cuda/render_shadow.cu")]:
    if not f.is_file():
        continue
    t = f.read_text(encoding="utf-8", errors="replace")
    for w in banned:
        if w in t:
            hits.append({"path": f.relative_to(REPO).as_posix(), "term": w})
note("banned_term_hits", hits)
for h in hits:
    fail(f"禁词命中 {h['term']} @ {h['path']}")

red = subprocess.run(
    ["git", "diff", "--numstat", f"{PRE_FIX}..HEAD", "--", "fixtures", "tests/support/render_bvh",
     "tests/render_shadow_reference.rs", "src/render/kernel/cuda/render.cu", "src/render/kernel/launch.rs",
     "src/render/kernel/cast.rs", "src/model/render.rs", "src/render/context/camera_output.rs",
     "src/runtime", "Cargo.toml", "Cargo.lock"],
    cwd=REPO, capture_output=True, text=True, encoding="utf-8",
).stdout
note("redline_reference_diff_since_prefix", red.strip())
if red.strip():
    fail("判据/参考/夹具/红线路径有改动")

stat = subprocess.run(["git", "diff", "--numstat", f"{IMPL_PARENT}..HEAD", "--", "src/render/kernel"], cwd=REPO, capture_output=True, text=True, encoding="utf-8").stdout
note("kernel_diff_since_impl_parent", [l for l in stat.splitlines() if l.strip()])

note("problems", result["problems"])
print("problems =", len(result["problems"]))
EVID.mkdir(parents=True, exist_ok=True)
(EVID / "recompute.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
