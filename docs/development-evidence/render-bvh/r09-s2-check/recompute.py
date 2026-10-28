"""R09-S2 独立复算脚本（QA；只读）。

复核对象：实现提交 `6b5d6dc6`、收口 `1ad2f268`（判据树 HEAD）。
本脚本独立重算（不引用 Backend/Arch 报告值）：
  1. 证据目录每个文件 LF/CR/BOM；
  2. `manifest.txt` 每条 sha256/bytes（路径解析规则：先仓库根，再证据目录）；
  3. 关键文件行数（LF 实数）与证据声称值对照；
  4. `MEGAKERNEL_SOURCE` 六段拼接连结后的 sha256（对应内核编译门 source_sha256）；
  5. 禁词扫描（任务卡 3 词；脚本内以码点拼装，避免字面自命中）；
  6. 槽位/拼接/读取点静态断言（81 槽、`splat_scale` 读取点、host 侧阴影拒绝仍有）；
  7. 红线路径 `git diff 6b5d6dc6^..1ad2f268` 为空。

输出：recompute.json（机器可读）+ 标准输出。
"""
import hashlib
import json
import pathlib
import re
import subprocess

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/render-bvh/r09-s2-check"
IMPL_PARENT = "6b5d6dc6^"
CLOSURE = "1ad2f268"

result = {"task_id": "2026-10-28-render-bvh-r09-s2-check", "checks": {}, "problems": []}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def note(key, value):
    result["checks"][key] = value
    print(f"{key} = {value}")


def fail(msg):
    result["problems"].append(msg)
    print(f"[FAIL] {msg}")


# 1. 证据文件 EOL/BOM
eol = {}
for p in sorted(EVID.rglob("*")):
    if p.is_file() and p.name != "recompute.json":  # 本脚本产物，不入清单、不参与自校验
        b = p.read_bytes()
        eol[p.relative_to(EVID).as_posix()] = {
            "bytes": len(b),
            "lf": b.count(b"\n"),
            "cr": b.count(b"\r"),
            "bom": b.startswith(b"\xef\xbb\xbf"),
        }
note("evidence_eol_bom", eol)
for rel, v in eol.items():
    if v["cr"]:
        fail(f"{rel} 含 CR")
    if v["bom"]:
        fail(f"{rel} 含 BOM")

# 2. manifest 逐条重算
man_path = EVID / "manifest.txt"
man_lines = man_path.read_text(encoding="utf-8").splitlines() if man_path.exists() else []
if not man_path.exists():
    note("manifest_status", "absent (first run: manifest is generated after README/receipt/logs)")
man = []
for line in man_lines:
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    digest, size, rel = line.split()
    if (EVID / rel).exists():
        # 证据目录内文件：按工作树（=待提交/已提交 blob，LF）复核
        b = (EVID / rel).read_bytes()
        source = "worktree"
    else:
        # 外部被复核文件：按 committed blob（LF 归一）复核，避免工作树 CRLF（仓库既有 EOL 状态）误报
        blob = subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=REPO, capture_output=True).stdout
        if blob:
            b = blob
            source = "git_blob"
        else:
            p = REPO / rel
            if not p.exists():
                fail(f"manifest 目标不存在：{rel}")
                continue
            b = p.read_bytes()
            source = "worktree_fallback"
    ok = sha(b) == digest and len(b) == int(size)
    man.append({"path": rel, "source": source, "sha256_match": sha(b) == digest, "bytes": len(b), "claimed": int(size), "ok": ok})
    if not ok:
        fail(f"manifest 不符：{rel}")
note("manifest_entries", man)
note("manifest_count", len(man))
if any(m["path"].endswith("manifest.txt") for m in man):
    fail("manifest 自引用")

# 3. 行数
claimed_lines = {
    "src/render/kernel/cuda/render_shadow.cu": 30,
    "src/render/kernel/cuda/render_shade.cu": 359,
    "src/render/kernel/cuda/render_megakernel.cu": 340,
    "src/render/kernel/megakernel.rs": 492,
    "src/render/kernel/megakernel_arrays.rs": 241,
    "src/render/kernel/mod.rs": 474,
    "tests/render_lighting_kernel.rs": 145,
    "src/render/kernel/cuda/render.cu": 499,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/cast.rs": 494,
    "src/model/render.rs": 500,
    "src/render/context/camera_output.rs": 499,
    "docs/development-evidence/render-bvh/r09-s2-lighting-kernel/README.md": 57,
    "docs/development-evidence/render-bvh/r09-s2-lighting-kernel/receipt.json": 113,
}
lines = {}
for rel, claim in claimed_lines.items():
    b = (REPO / rel).read_bytes()
    n = b.count(b"\n")
    lines[rel] = {"lf": n, "claimed": claim, "match": n == claim}
    if n != claim:
        fail(f"行数不符：{rel} 实测 {n}，声称 {claim}")
note("line_counts", lines)

# 4. 拼接 sha256
parts = [
    "src/render/bvh/cuda/scene.cu",
    "src/render/kernel/cuda/render_math.cu",
    "src/render/kernel/cuda/render_common.cu",
    "src/render/kernel/cuda/render_shadow.cu",
    "src/render/kernel/cuda/render_shade.cu",
    "src/render/kernel/cuda/render_megakernel.cu",
]
cat = b"".join((REPO / x).read_bytes() for x in parts)
note("concat_sha256", sha(cat))
note("concat_bytes", len(cat))
impl_receipt = json.loads(
    (REPO / "docs/development-evidence/render-bvh/r09-s2-lighting-kernel/receipt.json").read_text(encoding="utf-8")
)
note("impl_receipt_source_sha256", impl_receipt["kernel_compile_gate"]["source_sha256"])
note("impl_receipt_ptx", {
    "sha256": impl_receipt["kernel_compile_gate"]["ptx_sha256"],
    "bytes": impl_receipt["kernel_compile_gate"]["ptx_bytes"],
})
if sha(cat) != impl_receipt["kernel_compile_gate"]["source_sha256"]:
    fail("拼接 sha256 与实现证据声称值不符")

# 5. 禁词（码点拼装，避免本脚本自命中）
banned = [
    "".join(chr(c) for c in s)
    for s in (
        (23436, 25972, 28210, 26524, 36890, 36807),
        (23436, 25972, 82, 48, 49, 36890, 36807),
        (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
        (71, 49, 48, 24050, 23454, 29616),
        (23436, 25972, 29289, 29702, 36890, 36807),
        (21452, 24179, 21488, 36890, 36807),
        (19978, 28216, 23436, 20840, 31561, 20215),
        (21407, 29983, 25918, 34892),
        (31934, 24230, 36798, 26631),
    )
]
hits = []
for p in sorted(EVID.rglob("*")) + [
    REPO / "tests/render_lighting_kernel.rs",
    REPO / "src/render/kernel/megakernel_arrays.rs",
    REPO / "src/render/kernel/mod.rs",
    REPO / "src/render/kernel/megakernel.rs",
    REPO / "src/render/kernel/cuda/render_shadow.cu",
    REPO / "src/render/kernel/cuda/render_shade.cu",
    REPO / "src/render/kernel/cuda/render_megakernel.cu",
    REPO / "target/r09-s2-qa/recompute.py",
]:
    if not p.is_file():
        continue
    t = p.read_text(encoding="utf-8", errors="replace")
    for w in banned:
        if w in t:
            hits.append({"path": p.relative_to(REPO).as_posix(), "term": w})
note("banned_term_hits", hits)
for h in hits:
    fail(f"禁词命中 {h['term']} @ {h['path']}")

# 6. 槽位/读取点/守卫
args = (REPO / "src/runtime/launcher/render/args.rs").read_text(encoding="utf-8")
m = re.search(r"pub static MEGAKERNEL_SLOTS: \[RenderArgKind; (\d+)\]", args)
note("megakernel_slots_len", int(m.group(1)) if m else None)
if not m or m.group(1) != "81":
    fail("MEGAKERNEL_SLOTS 不是 81 槽")
cu = (REPO / "src/render/kernel/cuda/render_megakernel.cu").read_text(encoding="utf-8")
reads = re.findall(r"splat_scale\.data\[(\d+)\]\[(\d+)\]", cu)
note("splat_scale_reads", reads)
if reads.count(("0", "2")) != 1:
    fail("splat_scale[0][2] 读取点不唯一")
shadow = (REPO / "src/render/kernel/cuda/render_shadow.cu").read_text(encoding="utf-8")
note("shadow_reuses_cast_ray", "cast_ray(" in shadow)
note("shadow_source_has_any_hit_api", "any_hit" in shadow.lower())
mk = (REPO / "src/render/kernel/megakernel.rs").read_text(encoding="utf-8")
note("host_still_rejects_use_shadows", "S7b3 不含阴影" in mk)
note("host_guard_lines", [i + 1 for i, l in enumerate(mk.splitlines()) if "S7b3 不含阴影" in l])
mod_rs = (REPO / "src/render/kernel/mod.rs").read_text(encoding="utf-8")
note("concat_order", [name for name in ("render_common.cu", "render_shadow.cu", "render_shade.cu", "render_megakernel.cu") if name in mod_rs])
note("module_source_count", len(re.findall(r'\[\[source\]\]\npath = "', (REPO / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8"))))

# 7. 红线 diff
red = subprocess.run(
    ["git", "diff", "--numstat", f"{IMPL_PARENT}..{CLOSURE}", "--",
     "src/render/kernel/cuda/render.cu", "src/render/kernel/launch.rs", "src/render/kernel/cast.rs",
     "src/model/render.rs", "src/render/context/camera_output.rs", "src/render/bvh", "fixtures",
     "Cargo.toml", "Cargo.lock", "src/runtime", "src/runtime/collision_gpu"],
    cwd=REPO, capture_output=True, text=True, encoding="utf-8",
)
note("redline_diff", red.stdout.strip())
if red.stdout.strip():
    fail("红线路径有改动")

# 8. 改动文件清单
stat = subprocess.run(
    ["git", "diff", "--numstat", f"{IMPL_PARENT}..{CLOSURE}"],
    cwd=REPO, capture_output=True, text=True, encoding="utf-8",
)
note("diff_numstat", [l for l in stat.stdout.splitlines() if l.strip()])

note("problems", result["problems"])
print("problems =", len(result["problems"]))
(EVID / "recompute.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n"
)
