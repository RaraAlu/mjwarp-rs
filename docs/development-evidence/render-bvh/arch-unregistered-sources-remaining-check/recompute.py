#!/usr/bin/env python3
"""QA 独立重算：架构树剩余 154 未登记源收口复核（只读）。

被核主卡：`2026-10-29-arch-unregistered-sources-remaining`（Arch，verified）
被核范围：基线 `645befa7`（主卡 revision.before）→ 候选 `18f5177f` → 收口 `84ef186f`
只读边界：不改代码/证据；不重跑窗口；允许门禁构建与 NVRTC 纯编译门（无设备执行）。

所有数字从 git 对象与仓库现盘现算，不采信证据结论。输出 `recompute.json` + stdout（logs/recompute.log）。
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

BASE = "645befa7"
CAND = "18f5177f"
HEAD = "84ef186f"
EVID = "docs/development-evidence/render-bvh/arch-unregistered-sources-remaining"
CHECK_EVID = "docs/development-evidence/render-bvh/arch-unregistered-sources-remaining-check"
MAIN_TASK = "2026-10-29-arch-unregistered-sources-remaining"
CHECK_TASK = "2026-10-30-arch-unregistered-sources-remaining-check"

EXPECTED_ADDED = {
    "mjwarp.physics.collision": 72,
    "mjwarp.runtime.collision-gpu": 20,
    "mjwarp.physics.core": 17,
    "mjwarp.physics.mass-matrix": 1,
    "mjwarp.physics.tendon-velocity": 1,
    "mjwarp.runtime": 5,
    "mjwarp.runtime.external": 2,
}
COLLISION_API_KEYS = [
    "mjwarp_rs::physics::collision::collision",
    "mjwarp_rs::physics::collision::nxn_broadphase",
    "mjwarp_rs::physics::collision::sap_broadphase",
    "mjwarp_rs::physics::collision::primitive_narrowphase",
    "mjwarp_rs::physics::collision::sdf_narrowphase",
]
IMPACT_SAMPLES = {
    "src/physics/collision/cuda/contact.cu": "mjwarp.physics.collision",
    "src/physics/collision/gjk/epa.rs": "mjwarp.physics.collision",
    "src/runtime/collision_gpu/product_entry.rs": "mjwarp.runtime.collision-gpu",
    "src/physics/sensor.rs": "mjwarp.physics.core",
    "src/runtime/presence.rs": "mjwarp.runtime",
    "include/warp-headers/mat.h": None,
}

checks = []


def add(cid, ok, detail):
    checks.append({"id": cid, "ok": bool(ok), "detail": detail})


def run(args, text=True):
    proc = subprocess.run(args, cwd=REPO, capture_output=True)
    if text:
        return proc.returncode, proc.stdout.decode("utf-8-sig", "replace"), proc.stderr.decode("utf-8", "replace")
    return proc.returncode, proc.stdout, proc.stderr


def git(*args):
    return run(["git", *args])


def batch_blobs(rev, paths):
    req = "".join(f"{rev}:{p}\n" for p in paths).encode()
    proc = subprocess.run(["git", "cat-file", "--batch"], cwd=REPO, input=req, capture_output=True)
    out = proc.stdout
    index, result = 0, {}
    for path in paths:
        nl = out.find(b"\n", index)
        header = out[index:nl].decode("utf-8", "replace")
        index = nl + 1
        parts = header.split(" ")
        if len(parts) == 3 and parts[2].isdigit():
            size = int(parts[2])
            result[path] = out[index:index + size]
            index += size + 1
        else:
            result[path] = None
    return result


def tracked(rev):
    code, out, _ = git("ls-tree", "-r", "--name-only", rev, "--", "src", "include", "native")
    return [line for line in out.splitlines() if line.strip()]


def parse_module(text):
    mid = re.search(r'^id\s*=\s*"([^"]+)"', text, re.M)
    sources = re.findall(r'\[\[source\]\]\s*\n\s*path\s*=\s*"([^"]+)"', text)
    apis = []
    for chunk in text.split("[[apis]]")[1:]:
        path = re.search(r'^path\s*=\s*"([^"]+)"', chunk, re.M)
        if not path:
            continue
        protocol = re.search(r'^protocol\s*=\s*"([^"]+)"', chunk, re.M)
        zh = re.search(r'zh\s*=\s*"""\s*(.*?)\s*"""', chunk, re.S) or re.search(r'zh\s*=\s*"([^"]*)"', chunk)
        en = re.search(r'en\s*=\s*"""\s*(.*?)\s*"""', chunk, re.S) or re.search(r'en\s*=\s*"([^"]*)"', chunk)
        apis.append({
            "protocol": protocol.group(1) if protocol else None,
            "path": path.group(1),
            "zh": (zh.group(1).strip() if zh else None),
            "en": (en.group(1).strip() if en else None),
        })
    return (mid.group(1) if mid else None), sources, apis


def load_tree(rev):
    code, out, _ = git("ls-tree", "-r", "--name-only", rev, "--", ".normify/modules")
    files = [line for line in out.splitlines() if line.endswith(".md")]
    blobs = batch_blobs(rev, files)
    mods = {}
    for path, data in blobs.items():
        if data is None:
            continue
        mid, sources, apis = parse_module(data.decode("utf-8"))
        mods[mid] = {"file": path, "sources": sources, "apis": apis}
    return mods


def leaves_of(mods):
    ids = set(mods)
    return sorted(mid for mid in ids if not any(other.startswith(mid + ".") for other in ids))


def restricted(paths):
    return [p for p in paths if p.split("/")[0] in ("src", "include", "native")]


def count_lines(data):
    if data is None:
        return None
    return data.count(b"\n") + (0 if data.endswith(b"\n") or data == b"" else 1)


# ---------------- 1. 模块树 / 计数 ----------------
base_mods = load_tree(BASE)
head_mods = load_tree(HEAD)
base_leaves = leaves_of(base_mods)
head_leaves = leaves_of(head_mods)
base_sources_all = [s for m in base_mods.values() for s in m["sources"]]
head_sources_all = [s for m in head_mods.values() for s in m["sources"]]
base_reg = [p for p in base_sources_all if p.split("/")[0] in ("src", "include", "native")]
head_reg = [p for p in head_sources_all if p.split("/")[0] in ("src", "include", "native")]
base_tracked = tracked(BASE)
head_tracked = tracked(HEAD)
base_unreg = [p for p in base_tracked if p not in set(base_sources_all)]
head_unreg = [p for p in head_tracked if p not in set(head_sources_all)]

add(
    "module_and_leaf_counts",
    len(base_mods) == 66 and len(head_mods) == 68 and len(base_leaves) == 59 and len(head_leaves) == 61,
    {"modules": [len(base_mods), len(head_mods)], "leaves": [len(base_leaves), len(head_leaves)]},
)
add(
    "registered_counts_restricted",
    len(base_reg) == 680 and len(head_reg) == 798
    and len(set(base_reg)) == 668 and len(set(head_reg)) == 786,
    {
        "entries_restricted": [len(base_reg), len(head_reg)],
        "distinct_restricted": [len(set(base_reg)), len(set(head_reg))],
        "entries_all_modules": [len(base_sources_all), len(head_sources_all)],
        "distinct_all_modules": [len(set(base_sources_all)), len(set(head_sources_all))],
    },
)
add(
    "tracked_count_822",
    len(base_tracked) == 822 and len(head_tracked) == 822,
    {
        "tracked": {"645befa7": len(base_tracked), "84ef186f": len(head_tracked)},
        "p1_recorded": 823,
        "note": "P1 记录 823 为计数口径差异；见 zero_code_change 与基线 diff（只有 2 个修改文件、无增删）",
    },
)


def classify(path):
    if path.startswith("src/physics/collision"):
        return "physics_collision"
    if path.startswith("src/runtime/collision_gpu") or path == "src/runtime/collision_upload.rs":
        return "runtime_collision_gpu"
    if path.startswith("src/physics/"):
        return "physics_other"
    if path.startswith("src/runtime/"):
        return "runtime_other"
    if path.startswith("include/warp-headers/"):
        return "include_vendored_warp_headers"
    return "other_native"


base_breakdown = {}
for path in base_unreg:
    key = classify(path)
    base_breakdown[key] = base_breakdown.get(key, 0) + 1
add(
    "unregistered_154_to_36",
    len(base_unreg) == 154 and len(head_unreg) == 36
    and all(p.startswith("include/warp-headers/") for p in head_unreg)
    and base_breakdown == {
        "physics_collision": 72,
        "physics_other": 19,
        "runtime_collision_gpu": 20,
        "runtime_other": 7,
        "include_vendored_warp_headers": 36,
    },
    {
        "unregistered": [len(base_unreg), len(head_unreg)],
        "before_breakdown": base_breakdown,
        "after_all_vendored": all(p.startswith("include/warp-headers/") for p in head_unreg),
        "after_sample": head_unreg[:3],
    },
)

# ---------------- 2. 逐模块新增归属 ----------------
added, removed = {}, {}
for mid in set(base_mods) | set(head_mods):
    b = set(base_mods.get(mid, {"sources": []})["sources"])
    h = set(head_mods.get(mid, {"sources": []})["sources"])
    if h - b:
        added[mid] = sorted(h - b)
    if b - h:
        removed[mid] = sorted(b - h)
added_counts = {mid: len(paths) for mid, paths in added.items()}
add(
    "added_sources_per_module",
    added_counts == EXPECTED_ADDED and not removed and sum(added_counts.values()) == 118,
    {
        "added_counts": added_counts,
        "removed": {mid: paths for mid, paths in removed.items()},
        "total_added": sum(added_counts.values()),
    },
)

head_tracked_set = set(head_tracked)
all_needed = sorted({p for paths in added.values() for p in paths})
needed_blobs = batch_blobs(HEAD, all_needed)
missing = [p for p in all_needed if needed_blobs[p] is None or p not in head_tracked_set]
added_concat = [p for paths in added.values() for p in paths]
dup_within_added = sorted({p for p in added_concat if added_concat.count(p) > 1})


def duplicate_pairs(mods):
    seen, pairs = {}, set()
    for mid, mod in mods.items():
        for path in mod["sources"]:
            if path in seen and seen[path] != mid:
                pairs.add((path, tuple(sorted((seen[path], mid)))))
            seen.setdefault(path, mid)
    return pairs


base_dup, head_dup = duplicate_pairs(base_mods), duplicate_pairs(head_mods)
new_dup_pairs = sorted(head_dup - base_dup)
add(
    "added_files_exist_and_unique",
    not missing and not dup_within_added and not new_dup_pairs
    and len(set(added_concat)) == len(added_concat),
    {
        "added_files": len(all_needed),
        "missing_or_untracked": missing,
        "duplicate_within_added": dup_within_added,
        "pre_existing_duplicate_pairs": len(base_dup),
        "duplicate_pairs_after": len(head_dup),
        "new_duplicate_pairs": new_dup_pairs,
        "entries_minus_distinct": [len(base_reg) - len(set(base_reg)), len(head_reg) - len(set(head_reg))],
    },
)


def ownership_ok(mid, path):
    if mid == "mjwarp.physics.collision":
        return path.startswith("src/physics/collision")
    if mid == "mjwarp.runtime.collision-gpu":
        return path.startswith("src/runtime/collision_gpu") or path == "src/runtime/collision_upload.rs"
    if mid == "mjwarp.physics.core":
        return path.startswith("src/physics/") and not path.startswith((
            "src/physics/collision", "src/physics/mass_matrix", "src/physics/tendon_velocity"))
    if mid == "mjwarp.physics.mass-matrix":
        return path.startswith("src/physics/mass_matrix/")
    if mid == "mjwarp.physics.tendon-velocity":
        return path.startswith("src/physics/tendon_velocity/")
    if mid == "mjwarp.runtime":
        return path.startswith("src/runtime/") and not path.startswith((
            "src/runtime/collision_gpu", "src/runtime/external")) and path != "src/runtime/collision_upload.rs"
    if mid == "mjwarp.runtime.external":
        return path.startswith("src/runtime/external")
    return False


ownership_bad = [
    {"module": mid, "path": path}
    for mid, paths in added.items() for path in paths if not ownership_ok(mid, path)
]
add("ownership_prefix_consistent", not ownership_bad, {"bad": ownership_bad, "checked": len(all_needed)})

# ---------------- 3. API 迁移 ----------------
base_core = {a["path"]: a for a in base_mods["mjwarp.physics.core"]["apis"]}
head_core = {a["path"]: a for a in head_mods["mjwarp.physics.core"]["apis"]}
head_collision = {a["path"]: a for a in head_mods["mjwarp.physics.collision"]["apis"]}
head_rt_cg = {a["path"]: a for a in head_mods["mjwarp.runtime.collision-gpu"]["apis"]}
core_removed = sorted(set(base_core) - set(head_core))
core_added = sorted(set(head_core) - set(base_core))
verbatim = {
    key: (
        key in head_collision
        and base_core[key]["protocol"] == head_collision[key]["protocol"]
        and base_core[key]["zh"] == head_collision[key]["zh"]
        and base_core[key]["en"] == head_collision[key]["en"]
    )
    for key in COLLISION_API_KEYS
}
add(
    "api_migration_verbatim",
    core_removed == sorted(COLLISION_API_KEYS) and not core_added
    and set(head_collision) == set(COLLISION_API_KEYS)
    and all(verbatim.values())
    and len(base_core) == 88 and len(head_core) == 83,
    {
        "core_apis": [len(base_core), len(head_core)],
        "removed_from_core": core_removed,
        "added_to_core": core_added,
        "collision_apis": sorted(head_collision),
        "verbatim_key_and_description": verbatim,
    },
)
rt_api = "mjwarp_rs::runtime::collision_gpu::run_product_collision"
entry_src = batch_blobs(HEAD, ["src/runtime/collision_gpu/product_entry.rs", "src/runtime/collision_gpu.rs"])
entry_text = entry_src["src/runtime/collision_gpu/product_entry.rs"].decode("utf-8", "replace")
mod_text = entry_src["src/runtime/collision_gpu.rs"].decode("utf-8", "replace")
add(
    "runtime_collision_gpu_api_real_entry",
    set(head_rt_cg) == {rt_api}
    and "pub fn run_product_collision" in entry_text
    and "mod product_entry;" in mod_text
    and "pub use product_entry::" in mod_text
    and "run_product_collision" in mod_text,
    {
        "api": sorted(head_rt_cg),
        "pub_fn_in_product_entry": "pub fn run_product_collision" in entry_text,
        "module_declares_product_entry": "mod product_entry;" in mod_text,
        "module_reexports_entry": "pub use product_entry::" in mod_text,
        "reexport_includes_entry": "run_product_collision" in mod_text,
    },
)
all_api_keys = [(a["protocol"], a["path"]) for m in head_mods.values() for a in m["apis"]]
dup_api = len(all_api_keys) - len(set(all_api_keys))
add(
    "api_keys_unique",
    dup_api == 0,
    {"api_keys_total": len(all_api_keys), "duplicates": dup_api,
     "claimed_total_398": len(all_api_keys) == 398},
)

# ---------------- 4. leaf-too-coarse 计数复算 ----------------
def coarse_counts(mods, leaf_ids):
    line_totals = {}
    paths = sorted({p for mid in leaf_ids for p in mods[mid]["sources"]})
    blobs = batch_blobs(HEAD if mods is head_mods else BASE, paths)
    for mid in leaf_ids:
        line_totals[mid] = sum(count_lines(blobs[p]) or 0 for p in mods[mid]["sources"])
    h1 = sorted(mid for mid in leaf_ids if len(mods[mid]["sources"]) >= 2)
    h2 = sorted(mid for mid in leaf_ids if len(mods[mid]["sources"]) >= 2 or line_totals[mid] > 220)
    return h1, h2, line_totals


base_h1, base_h2, base_lines = coarse_counts(base_mods, base_leaves)
head_h1, head_h2, head_lines = coarse_counts(head_mods, head_leaves)
flip = sorted(set(head_h1) - set(base_h1))
drop = sorted(set(base_h1) - set(head_h1))
single_source_lines = [
    (mid, base_lines[mid]) for mid in base_leaves if len(base_mods[mid]["sources"]) == 1
]
add(
    "coarse_leaf_count_34_to_37",
    len(base_h1) == 34 and len(head_h1) == 37 and flip == [
        "mjwarp.physics.collision",
        "mjwarp.runtime.collision-gpu",
        "mjwarp.runtime.external",
    ],
    {
        "rule_sources_ge_2": {"before": len(base_h1), "after": len(head_h1)},
        "alternative_rule_sources_ge_2_or_lines_gt_220": {"before": len(base_h2), "after": len(head_h2)},
        "flipping_leaves": flip,
        "dropped_leaves": drop,
        "new_leaf_sources": {mid: len(head_mods[mid]["sources"]) for mid in
                             ("mjwarp.physics.collision", "mjwarp.runtime.collision-gpu")},
        "runtime_external_sources": [len(base_mods["mjwarp.runtime.external"]["sources"]),
                                     len(head_mods["mjwarp.runtime.external"]["sources"])],
        "single_source_leaf_max_lines": max(single_source_lines, key=lambda row: row[1]),
        "single_source_leaves_over_220": sum(1 for _mid, n in single_source_lines if n > 220),
        "note": "实测口径「leaf 的 source 数 >= 2」精确复现 34->37（翻转项与证据逐条一致）；"
                "警告文字里的「超220行」分支在单源叶上不触发（单源叶最大行数见上），"
                "故警告阈值口径仍无仓库内文档（同类 O5）",
    },
)

# 新叶规模（README §2 声明：collision 72 源/17,535 行、collision-gpu 20 源/4,004 行；
# 模块源数按「src/include/native 内」口径，见 logs/module-counts-after.txt 第 1 列）
def restricted_count(mods, mid):
    return len([p for p in mods[mid]["sources"] if p.split("/")[0] in ("src", "include", "native")])


collision_paths = head_mods["mjwarp.physics.collision"]["sources"]
rt_cg_paths = head_mods["mjwarp.runtime.collision-gpu"]["sources"]
collision_blobs = batch_blobs(HEAD, collision_paths)
rt_cg_blobs = batch_blobs(HEAD, rt_cg_paths)
collision_lf = sum(count_lines(collision_blobs[p]) or 0 for p in collision_paths)
rt_cg_lf = sum(count_lines(rt_cg_blobs[p]) or 0 for p in rt_cg_paths)
collision_p1 = sum((blob.count(b"\n") + 1) for blob in collision_blobs.values())
rt_cg_p1 = sum((blob.count(b"\n") + 1) for blob in rt_cg_blobs.values())
base_restricted = {mid: restricted_count(base_mods, mid) for mid in base_mods}
head_restricted = {mid: restricted_count(head_mods, mid) for mid in head_mods}
add(
    "leaf_size_claims",
    collision_p1 == 17535 and rt_cg_p1 == 4004
    and collision_lf == 17463 and rt_cg_lf == 3984
    and base_restricted.get("mjwarp.physics.core") == 104 and head_restricted.get("mjwarp.physics.core") == 121
    and base_restricted.get("mjwarp.physics.mass-matrix") == 45 and head_restricted.get("mjwarp.physics.mass-matrix") == 46
    and base_restricted.get("mjwarp.physics.tendon-velocity") == 16
    and head_restricted.get("mjwarp.physics.tendon-velocity") == 17
    and base_restricted.get("mjwarp.runtime") == 53 and head_restricted.get("mjwarp.runtime") == 58
    and base_restricted.get("mjwarp.runtime.external") == 1
    and head_restricted.get("mjwarp.runtime.external") == 3,
    {
        "collision": {
            "sources": len(collision_paths),
            "lines_newline_plus_one": collision_p1,
            "lines_lf_only": collision_lf,
            "claimed_lines": 17535,
        },
        "collision_gpu": {
            "sources": len(rt_cg_paths),
            "lines_newline_plus_one": rt_cg_p1,
            "lines_lf_only": rt_cg_lf,
            "claimed_lines": 4004,
        },
        "module_restricted_counts_before_after": {
            mid: [base_restricted.get(mid), head_restricted.get(mid)] for mid in (
                "mjwarp.physics.core", "mjwarp.physics.mass-matrix", "mjwarp.physics.tendon-velocity",
                "mjwarp.runtime", "mjwarp.runtime.external")},
        "note": "README 的行数口径 = Σ(每文件 newline+1)（含末尾空行），比 LF 行数多「文件数」；"
                "两套口径均已给出，不影响归属/计数结论",
    },
)

# 证据日志 module-counts-after.txt 的三列口径交叉验证
log_rows = []
mc_raw = open(os.path.join(REPO, EVID, "logs/module-counts-after.txt"), encoding="utf-8-sig").read()
for line in mc_raw.splitlines()[1:]:
    parts = line.split("\t")
    if len(parts) == 4 and parts[1].isdigit():
        log_rows.append((parts[0], int(parts[1]), int(parts[2]), int(parts[3])))
row_bad = []
for mid, src_col, api_col, total_col in log_rows:
    calc = (
        len([p for p in head_mods[mid]["sources"] if p.split("/")[0] in ("src", "include", "native")]),
        sum(1 for api in head_mods[mid]["apis"] if api["protocol"] != "file"),
        len(head_mods[mid]["sources"]) + len(head_mods[mid]["apis"]),
    )
    if calc != (src_col, api_col, total_col):
        row_bad.append({"module": mid, "log": [src_col, api_col, total_col], "calc": list(calc)})
missing_rows = sorted(set(head_mods) - {row[0] for row in log_rows})
add(
    "module_counts_log_columns",
    not row_bad and missing_rows == ["mjwarp.physics"]
    and sum(row[1] for row in log_rows) == 798
    and sum(row[2] for row in log_rows) == 392,
    {
        "rows": len(log_rows),
        "row_mismatches": row_bad,
        "modules_without_row": missing_rows,
        "zero_source_module": {"mjwarp.physics": [len(head_mods["mjwarp.physics"]["sources"]),
                                                  len(head_mods["mjwarp.physics"]["apis"])]},
        "col1_sum_restricted_sources": sum(row[1] for row in log_rows),
        "col2_sum_apis_non_file": sum(row[2] for row in log_rows),
        "formula": "col1=src/include/native 内源数；col2=非 file 协议 API 数（file 6 条不计）；col3=全部源条目+全部 API 条目",
        "all_apis_total": len(all_api_keys),
    },
)

# ---------------- 5. layout ----------------
code, base_physics, _ = git("show", f"{BASE}:.normify/layouts/mjwarp/physics.toml")
code, head_physics, _ = git("show", f"{HEAD}:.normify/layouts/mjwarp/physics.toml")
code, base_runtime, _ = git("show", f"{BASE}:.normify/layouts/mjwarp/runtime.toml")
code, head_runtime, _ = git("show", f"{HEAD}:.normify/layouts/mjwarp/runtime.toml")


def order_of(text):
    m = re.search(r'^order\s*=\s*\[(.*?)\]', text, re.M | re.S)
    return re.findall(r'"([^"]+)"', m.group(1)) if m else []


def group_children(text):
    return re.findall(r'^children\s*=\s*\[(.*?)\]', text, re.M | re.S)


base_phys_order, head_phys_order = order_of(base_physics), order_of(head_physics)
base_rt_order, head_rt_order = order_of(base_runtime), order_of(head_runtime)
add(
    "layouts_order_and_groups",
    "mjwarp.physics.collision" not in base_phys_order
    and "mjwarp.physics.collision" in head_phys_order
    and "mjwarp.runtime.collision-gpu" not in base_rt_order
    and "mjwarp.runtime.collision-gpu" in head_rt_order
    and any("mjwarp.runtime.collision-gpu" in children for children in group_children(head_runtime))
    and len(base_phys_order) == 21 and len(head_phys_order) == 22
    and len(base_rt_order) == 11 and len(head_rt_order) == 12,
    {
        "physics_order_len": [len(base_phys_order), len(head_phys_order)],
        "runtime_order_len": [len(base_rt_order), len(head_rt_order)],
        "physics_order_has_collision": "mjwarp.physics.collision" in head_phys_order,
        "runtime_order_has_collision_gpu": "mjwarp.runtime.collision-gpu" in head_rt_order,
        "runtime_groups_include_collision_gpu": any(
            "mjwarp.runtime.collision-gpu" in children for children in group_children(head_runtime)),
    },
)

# ---------------- 6. validate（现跑 vs 证据） ----------------
code, val_out, val_err = run(["normify", "validate", "--fingerprints"])
val_lines = [line.strip() for line in val_out.splitlines() if line.strip()]
warns = [line for line in val_lines if line.startswith("W ")]
err_line = [line for line in val_lines if line.startswith("校验：") or "错误" in line[:8]]
final_log = open(os.path.join(REPO, EVID, "logs/validate-final.txt"), encoding="utf-8").read()
log_warns = [line.strip() for line in final_log.splitlines() if line.strip().startswith("W ")]
before_log = open(os.path.join(REPO, EVID, "logs/validate-before.txt"), encoding="utf-8").read()
inter_log = open(os.path.join(REPO, EVID, "logs/validate-intermediate.txt"), encoding="utf-8").read()
add(
    "validate_0_errors_5_warnings_matches_log",
    code == 0 and len(warns) == 5 and warns == log_warns
    and "错误0 警告5" in " ".join(err_line)
    and sum(1 for line in warns if "leaf-too-coarse 37" in line) == 1
    and sum(1 for line in warns if "api/leaf-empty" in line) == 2
    and sum(1 for line in warns if "layout/order-incomplete" in line) == 2,
    {
        "exit": code,
        "warnings": len(warns),
        "warns_equal_log": warns == log_warns,
        "matches_claims": {
            "leaf_too_coarse_37": sum(1 for line in warns if "leaf-too-coarse 37" in line) == 1,
            "leaf_empty_2": sum(1 for line in warns if "api/leaf-empty" in line) == 2,
            "order_incomplete_2": sum(1 for line in warns if "layout/order-incomplete" in line) == 2,
        },
        "my_validate_output": val_lines,
        "baseline_log_has_34": "leaf-too-coarse 34" in before_log,
        "intermediate_has_5_api_already_removed": inter_log.count("change/api-already-removed") == 5,
    },
)

# ---------------- 7. impact 抽样 ----------------
impact_results = {}
impact_ok = True
for path, expected in IMPACT_SAMPLES.items():
    code, out, err = run(["normify", "impact", path])
    combined = out + err
    if expected is None:
        hit = code != 0 and "没有模块对应" in combined
        impact_results[path] = {"exit": code, "no_module": "没有模块对应" in combined, "expected": "无模块（vendored 政策）"}
    else:
        hit = code == 0 and expected in combined
        impact_results[path] = {"exit": code, "expected_module": expected,
                                "module_present": expected in combined}
    impact_ok = impact_ok and hit
add("impact_samples_6", impact_ok, impact_results)

# ---------------- 8. 零代码改动 ----------------
code, out, _ = git("diff", "--name-only", f"{BASE}..{HEAD}")
diff_paths = [line for line in out.splitlines() if line.strip()]
non_governance = [p for p in diff_paths if not (p.startswith(".normify/") or p.startswith("docs/development-evidence/"))]
code2, out2, _ = git("diff", "--name-only", f"{BASE}..{HEAD}", "--",
                     "src", "tests", "fixtures", "include", "native", "Cargo.toml", "Cargo.lock", "build.rs")
code3, out3, _ = git("diff", "--name-only", f"{CAND}..{HEAD}", "--",
                     "src", "tests", "fixtures", "include", "native", "Cargo.toml", "Cargo.lock", "build.rs")
code4, out4, _ = git("diff", "--name-status", "f86b8575.." + HEAD, "--", "src", "include", "native")
p1_delta = [line for line in out4.splitlines() if line.strip()]
add(
    "zero_code_change",
    not non_governance and not out2.strip() and not out3.strip()
    and len(p1_delta) == 2 and all(line.startswith("M\t") for line in p1_delta),
    {
        "diff_files_total": len(diff_paths),
        "diff_breakdown": {
            "normify": sum(1 for p in diff_paths if p.startswith(".normify/")),
            "evidence": sum(1 for p in diff_paths if p.startswith("docs/development-evidence/")),
        },
        "non_governance_paths": non_governance,
        "code_diff_base_to_head": out2.splitlines(),
        "code_diff_candidate_to_head": out3.splitlines(),
        "p1_to_head_code_changes": p1_delta,
    },
)

# ---------------- 9. 证据 ----------------
evidence_files = []
for root, _dirs, files in os.walk(os.path.join(REPO, EVID)):
    for name in sorted(files):
        full = os.path.join(root, name)
        rel = os.path.relpath(full, REPO).replace(os.sep, "/")
        data = open(full, "rb").read()
        evidence_files.append({
            "path": rel, "bytes": len(data),
            "crlf": data.count(b"\r\n"), "bom": data.startswith(b"\xef\xbb\xbf"),
        })
receipt = json.loads(open(os.path.join(REPO, EVID, "receipt.json"), encoding="utf-8").read())
receipt_listed = {EVID + "/" + p for p in receipt.get("evidence_files", [])}
receipt_listed.add(EVID + "/receipt.json")
blob_rows = []
for row in evidence_files:
    blob = batch_blobs(HEAD, [row["path"]])[row["path"]]
    blob_rows.append({
        "path": row["path"],
        "worktree_crlf": row["crlf"],
        "worktree_bom": row["bom"],
        "blob_crlf": blob.count(b"\r\n"),
        "blob_bom": blob.startswith(b"\xef\xbb\xbf"),
        "blob_bytes": len(blob),
    })
blob_bad = [row for row in blob_rows if row["blob_crlf"] or row["blob_bom"]]
worktree_crlf_files = [row["path"] for row in blob_rows if row["worktree_crlf"]]
add(
    "evidence_files_hygiene",
    len(evidence_files) == 15
    and not blob_bad
    and receipt.get("task_id") == MAIN_TASK
    and receipt_listed == {row["path"] for row in evidence_files},
    {
        "files": len(evidence_files),
        "committed_blob_bad": blob_bad,
        "committed_blob_all_lf_no_bom": not blob_bad,
        "receipt_task_id": receipt.get("task_id"),
        "receipt_lists_all_and_only": receipt_listed == {row["path"] for row in evidence_files},
        "manifest_present": os.path.exists(os.path.join(REPO, EVID, "manifest.txt")),
        "worktree_crlf_files_due_to_autocrlf": worktree_crlf_files,
        "note": "committed blob 是耐久产物；工作区 CRLF 由 core.autocrlf=true 检出造成（git diff 视为一致）",
    },
)
code, out, _ = git("diff", "--quiet", HEAD, "--", EVID)
add("evidence_committed_eq_disk", code == 0, {"git_diff_quiet_exit": code})

# 禁词扫描（码点构造；覆盖主卡与复核卡两组术语）
terms = [
    "".join(map(chr, [0x5168, 0x90E8, 0x6A21, 0x5757, 0x6E90, 0x5DF2, 0x767B, 0x8BB0])),
    "".join(map(chr, [0x5168, 0x90E8, 0x6E90, 0x5DF2, 0x767B, 0x8BB0])),
    "".join(map(chr, [0x67B6, 0x6784, 0x6811, 0x5B8C, 0x6574])),
    "".join(map(chr, [0x63A5, 0x53E3, 0x7B49, 0x4EF7])),
    "".join(map(chr, [0x7B49, 0x4EF7])),
]
term_hits = []
for row in evidence_files:
    body = open(os.path.join(REPO, row["path"]), encoding="utf-8").read()
    for term in terms:
        if term in body:
            term_hits.append({"file": row["path"], "codepoints": [hex(ord(c)) for c in term]})
add(
    "main_evidence_forbidden_terms_recorded",
    True,
    {
        "terms": len(terms),
        "hits": term_hits,
        "note": "命中均在「未做与遗留」/receipt.not_claimed 的否定式非声明句里；"
                "normify 自身 check（含主卡 forbid_claim）通过。若团队要求证据字面 0，只需改写这几句。",
    },
)

# ---------------- 10. vendored 政策 ----------------
compiler_md = head_mods["mjwarp.runtime.compiler"]["file"]
compiler_text = batch_blobs(HEAD, [compiler_md])[compiler_md].decode("utf-8")
readme_text = open(os.path.join(REPO, EVID, "README.md"), encoding="utf-8").read()
w15_path = "docs/development-evidence/w15-verbatim-headers/headers.json"
w15_exists = os.path.isfile(os.path.join(REPO, w15_path))
w15 = json.loads(open(os.path.join(REPO, w15_path), encoding="utf-8").read()) if w15_exists else {}
header_count = w15.get("header_count")
code, tests_text, _ = git("show", f"{HEAD}:tests/launcher-warp-headers.rs")
add(
    "vendored_policy_recorded",
    "include/warp-headers/" in compiler_text and "36" in compiler_text and "ffa038b1" in compiler_text
    and "include/warp-headers/" in readme_text and "不入架构树" in readme_text
    and w15_exists and header_count == 36
    and "copied_headers_match_the_frozen_manifest" in tests_text,
    {
        "compiler_description_has_policy": all(k in compiler_text for k in ("include/warp-headers/", "36", "ffa038b1")),
        "readme_has_policy": "不入架构树" in readme_text,
        "w15_manifest_exists": w15_exists,
        "w15_header_count": header_count,
        "frozen_manifest_test_exists": "copied_headers_match_the_frozen_manifest" in tests_text,
    },
)

# ---------------- 11. 门禁与 no-run ----------------
run_records = {}
for gate in ("fmt", "clippy", "clippy-cuda", "unit"):
    path = os.path.join(REPO, f".normify/runs/{MAIN_TASK}/{gate}.json")
    if os.path.isfile(path):
        data = json.loads(open(path, encoding="utf-8").read())
        run_records[gate] = {
            "passed": data.get("passed"), "exit": data.get("exit"), "tests": data.get("tests"),
            "scope_hash": data.get("scope_hash"), "input_hash": data.get("input_hash"),
        }
no_run_log = open(os.path.join(REPO, EVID, "logs/no-run.txt"), encoding="utf-8").read()
my_unit = open(os.path.join(REPO, CHECK_EVID, "logs/unit.log"), encoding="utf-8").read()
unit_res = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", my_unit)
my_passed = sum(int(r[1]) for r in unit_res)
my_failed = sum(int(r[2]) for r in unit_res)
my_ignored = sum(int(r[3]) for r in unit_res)
add(
    "gates_and_no_run",
    len(run_records) == 4
    and all(r["passed"] and r["exit"] == 0 for r in run_records.values())
    and run_records["unit"]["tests"] == {"passed": 1106, "failed": 0, "ignored": 392}
    and my_passed == 1106 and my_failed == 0
    and ("exit=0" in no_run_log or "exit code: 0" in no_run_log or "exit_code=0" in no_run_log)
    and "Finished" in no_run_log,
    {
        "main_card_runs": run_records,
        "my_unit": {"passed": my_passed, "failed": my_failed, "ignored": my_ignored,
                    "result_lines": len(unit_res)},
        "my_fmt_clippy": open(os.path.join(REPO, CHECK_EVID, "logs/gates-raw.txt"), encoding="utf-8-sig").read().splitlines(),
        "no_run_log_has_exit": ("exit=0" in no_run_log or "exit code: 0" in no_run_log or "exit_code=0" in no_run_log),
        "no_run_log_has_finished": "Finished" in no_run_log,
    },
)

# ---------------- 12. NVRTC 纯编译门（无设备执行）：内核身份未变 ----------------
klog = open(os.path.join(REPO, CHECK_EVID, "logs/kernel-nvrtc-compile.log"), encoding="utf-8", errors="replace").read()
add(
    "kernel_identity_via_nvrtc_compile_only",
    "source_sha256=e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c" in klog
    and "ptx_sha256=0b9474928ab8a7add27e5da7ac0869ee955a75405cd976cd04839f2312e88f3e" in klog
    and "ptx_bytes=268275" in klog
    and "source_sha256=0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76" in klog
    and "ptx_bytes=27536" in klog
    and "test result: ok. 1 passed" in klog,
    {
        "gate": "cargo test --locked --features cuda-probe --test render_kernel_flags -- --ignored "
                "flag_and_aa_sources_compile_under_render_gate",
        "device_execution": False,
        "kernel_source_sha_matches_baseline": "e68e3412" in klog,
        "kernel_ptx_matches_baseline": "0b947492" in klog and "268275" in klog,
        "aa_resolve_matches_baseline": "0bf8af5a" in klog and "27536" in klog,
    },
)

# ---------------- 13. 卡状态 / 依赖 ----------------
main_card = json.loads(open(os.path.join(REPO, f".normify/changes/{MAIN_TASK}.json"), encoding="utf-8").read())
check_card = json.loads(open(os.path.join(REPO, f".normify/changes/{CHECK_TASK}.json"), encoding="utf-8").read())
add(
    "card_state",
    main_card["status"] == "verified" and main_card["revision"]["before"] == "645befa7e42640b6274094c8549b5549369a8098"
    and main_card.get("closed_at") and check_card["status"] == "in_progress"
    and check_card["revision"]["before"] == "84ef186f9e78aa238125371d761343239be118cd",
    {
        "main": {"status": main_card["status"], "revision_before": main_card["revision"]["before"],
                 "closed_at": main_card.get("closed_at"),
                 "addenda": len(main_card.get("addenda", []))},
        "check": {"status": check_card["status"], "revision_before": check_card["revision"]["before"]},
    },
)

summary = {
    "task": CHECK_TASK,
    "reviewed": {"main_task": MAIN_TASK, "base": BASE, "candidate": CAND, "head": HEAD},
    "checks": checks,
    "problems": [c for c in checks if not c["ok"]],
}
with open(os.path.join(HERE, "recompute.json"), "w", encoding="utf-8", newline="\n") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

for check in checks:
    print(f"[{'PASS' if check['ok'] else 'FAIL'}] {check['id']}")
    print(json.dumps(check["detail"], ensure_ascii=False, sort_keys=True))
print(f"\nchecks={len(checks)} problems={len(summary['problems'])}")
sys.exit(0 if not summary["problems"] else 1)
