#!/usr/bin/env python3
"""QA 独立重算：render split 步 1 复核（只读）。

被核主卡：`2026-10-30-render-module-split`（步 1，Arch，verified）
范围：基线 `9e1567a2`（主卡 revision.before）→ 落地 `7de14d30`；当前 HEAD `4cdeffec`（仅 backlog 卡）
只读边界：不改 `.normify`/证据、不跑 GPU；允许门禁构建。

输出 `recompute.json` + stdout（logs/recompute.log）。
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

PRE = "9e1567a2"
POST = "7de14d30"
HEAD = "4cdeffec"
EVID = "docs/development-evidence/render-bvh/render-module-split"
CHECK_EVID = "docs/development-evidence/render-bvh/render-module-split-check"
MAIN_TASK = "2026-10-30-render-module-split"
CHECK_TASK = "2026-10-30-render-module-split-check"

EXPECTED_ASSIGNMENT = {
    "mjwarp.render.kernel": ["mjwarp_rs::render::render"],
    "mjwarp.render.bvh": ["mjwarp_rs::render::bvh::refit_bvh", "mjwarp_rs::render::bvh::refit_splat_bvh"],
    "mjwarp.render.context": [
        "mjwarp_rs::render::create_render_context",
        "mjwarp_rs::render::get_depth",
        "mjwarp_rs::render::get_rgb",
        "mjwarp_rs::render::get_segmentation",
        "mjwarp_rs::render::RenderContext",
        "mjwarp_rs::render::OutputChannels",
        "mjwarp_rs::render::ImageOutput",
        "mjwarp_rs::render::ImageBatchLayout",
    ],
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


def block_values(chunk, key):
    m = re.search(rf'^{key}\s*=\s*"([^"]*)"', chunk, re.M)
    return m.group(1) if m else None


def block_text(chunk, key):
    m = re.search(rf'{key}\s*=\s*"""(.*?)"""', chunk, re.S)
    if m:
        return m.group(1).strip()
    m = re.search(rf'^{key}\s*=\s*"([^"]*)"', chunk, re.M)
    return m.group(1) if m else None


def parse_module(text):
    mid = re.search(r'^id\s*=\s*"([^"]+)"', text, re.M)
    sources = re.findall(r'\[\[source\]\]\s*\n\s*path\s*=\s*"([^"]+)"', text)
    apis = []
    for chunk in text.split("[[apis]]")[1:]:
        path = re.search(r'^path\s*=\s*"([^"]+)"', chunk, re.M)
        if not path:
            continue
        apis.append({
            "protocol": block_values(chunk, "protocol"),
            "path": path.group(1),
            "zh": block_text(chunk, "zh"),
            "en": block_text(chunk, "en"),
        })
    deps = []
    for chunk in text.split("[[deps]]")[1:]:
        kind = block_values(chunk, "kind")
        to = block_values(chunk, "to")
        from_api = block_values(chunk, "from_api")
        to_api = block_values(chunk, "to_api")
        if not to:
            continue
        label_zh = block_text(chunk, "zh")
        label_en = block_text(chunk, "en")
        deps.append({"kind": kind, "to": to, "from_api": from_api, "to_api": to_api,
                     "zh": label_zh, "en": label_en})
    return (mid.group(1) if mid else None), {"sources": sources, "apis": apis, "deps": deps, "text": text}


def load_tree(rev):
    code, out, _ = git("ls-tree", "-r", "--name-only", rev, "--", ".normify/modules")
    files = [line for line in out.splitlines() if line.endswith(".md")]
    blobs = batch_blobs(rev, files)
    mods = {}
    for path, data in blobs.items():
        if data is None:
            continue
        mid, parsed = parse_module(data.decode("utf-8"))
        parsed["file"] = path
        mods[mid] = parsed
    return mods


def leaves_of(mods):
    ids = set(mods)
    return sorted(mid for mid in ids if not any(other.startswith(mid + ".") for other in ids))


def api_keys(mod):
    return {a["path"] for a in mod["apis"]}


# ---------------- 1. 结构计数 ----------------
pre, post = load_tree(PRE), load_tree(POST)
pre_leaves, post_leaves = leaves_of(pre), leaves_of(post)
pre_apis = [(a["protocol"], a["path"]) for m in pre.values() for a in m["apis"]]
post_apis = [(a["protocol"], a["path"]) for m in post.values() for a in m["apis"]]
add(
    "structure_counts",
    len(pre) == 68 and len(post) == 71 and len(pre_leaves) == 61 and len(post_leaves) == 63
    and len(pre_apis) == 398 and len(post_apis) == 398
    and len(set(pre_apis)) == 398 and len(set(post_apis)) == 398,
    {
        "modules": [len(pre), len(post)],
        "leaves": [len(pre_leaves), len(post_leaves)],
        "api_keys": [len(pre_apis), len(post_apis)],
        "api_duplicates": [len(pre_apis) - len(set(pre_apis)), len(post_apis) - len(set(post_apis))],
        "new_modules": sorted(set(post) - set(pre)),
        "removed_modules": sorted(set(pre) - set(post)),
    },
)

render_pre, render_post = pre["mjwarp.render"], post["mjwarp.render"]
children = {mid for mid in post if mid.startswith("mjwarp.render.")}
add(
    "render_container_conversion",
    set(render_pre["sources"]) == set(render_post["sources"]) and len(render_post["sources"]) == 53
    and len(render_pre["apis"]) == 11 and len(render_post["apis"]) == 0
    and len(render_pre["deps"]) == 6 and all(d["from_api"] for d in render_pre["deps"])
    and len(render_post["deps"]) == 0
    and "mjwarp.render" not in post_leaves and "mjwarp.render" in pre_leaves
    and children == {"mjwarp.render.kernel", "mjwarp.render.bvh", "mjwarp.render.context"},
    {
        "sources": [len(render_pre["sources"]), len(render_post["sources"])],
        "source_path_set_identical": set(render_pre["sources"]) == set(render_post["sources"]),
        "apis": [len(render_pre["apis"]), len(render_post["apis"])],
        "deps": [len(render_pre["deps"]), len(render_post["deps"])],
        "pre_deps_all_from_api": all(d["from_api"] for d in render_pre["deps"]),
        "post_is_container": "mjwarp.render" not in post_leaves,
        "children": sorted(children),
    },
)

# ---------------- 2. 三叶 API/依赖归属 ----------------
leaf_rows, assign_ok, own_ok = {}, True, True
for mid, expected in EXPECTED_ASSIGNMENT.items():
    got = sorted(a["path"] for a in post[mid]["apis"])
    if got != sorted(expected):
        assign_ok = False
    own = {f"{a['protocol']}:{a['path']}" for a in post[mid]["apis"]}
    rows = []
    for dep in post[mid]["deps"]:
        rows.append({"to": dep["to"], "kind": dep["kind"], "from_api": dep["from_api"]})
        if dep["from_api"] not in own:
            own_ok = False
    leaf_rows[mid] = {"apis": got, "own_api_keys_with_protocol": sorted(own),
                      "sources": len(post[mid]["sources"]), "deps": rows}
add(
    "three_leaves_assignment",
    assign_ok and own_ok
    and len(post["mjwarp.render.kernel"]["deps"]) == 4
    and len(post["mjwarp.render.context"]["deps"]) == 2
    and len(post["mjwarp.render.bvh"]["deps"]) == 0
    and all(len(post[mid]["sources"]) == 0 for mid in EXPECTED_ASSIGNMENT)
    and all(d["from_api"] for mid in EXPECTED_ASSIGNMENT for d in post[mid]["deps"]),
    {
        "leaves": leaf_rows,
        "api_assignment_matches": assign_ok,
        "from_api_is_own_leaf_api": own_ok,
        "step1_leaf_sources_zero": all(len(post[mid]["sources"]) == 0 for mid in EXPECTED_ASSIGNMENT),
    },
)

# ---------------- 3. 11 API / 6 依赖逐字 ----------------
pre_render_apis = {a["path"]: a for a in render_pre["apis"]}
post_leaf_apis = {a["path"]: a for mid in EXPECTED_ASSIGNMENT for a in post[mid]["apis"]}
api_detail = {}
for key, old in pre_render_apis.items():
    new = post_leaf_apis.get(key)
    same = new is not None and old["protocol"] == new["protocol"] and old["zh"] == new["zh"] and old["en"] == new["en"]
    api_detail[key] = {"present": new is not None, "protocol_zh_en_identical": same}
add(
    "apis_verbatim_11",
    len(pre_render_apis) == 11 and set(pre_render_apis) == set(post_leaf_apis) and all(
        row["protocol_zh_en_identical"] for row in api_detail.values()),
    {"moved": len(api_detail), "identical": sum(1 for r in api_detail.values() if r["protocol_zh_en_identical"]),
     "per_api": api_detail},
)

pre_deps = [dict(d) for d in render_pre["deps"]]
post_deps = [(mid, d) for mid in EXPECTED_ASSIGNMENT for d in post[mid]["deps"]]
pre_keys = sorted(json.dumps(d, sort_keys=True, ensure_ascii=False) for d in pre_deps)
post_keys = sorted(json.dumps(d, sort_keys=True, ensure_ascii=False) for _, d in post_deps)
dep_dist = {}
for mid, dep in post_deps:
    dep_dist.setdefault(mid, []).append(dep["from_api"])
all_other_deps_pre = {mid: sorted(json.dumps(d, sort_keys=True, ensure_ascii=False) for d in m["deps"])
                      for mid, m in pre.items() if mid != "mjwarp.render"}
all_other_deps_post = {mid: sorted(json.dumps(d, sort_keys=True, ensure_ascii=False) for d in m["deps"])
                       for mid, m in post.items()
                       if mid not in EXPECTED_ASSIGNMENT and mid != "mjwarp.render"}
add(
    "deps_verbatim_6",
    pre_keys == post_keys and len(pre_keys) == 6
    and len(dep_dist.get("mjwarp.render.kernel", [])) == 4
    and len(dep_dist.get("mjwarp.render.context", [])) == 2
    and all_other_deps_pre == all_other_deps_post
    and all(d["zh"] and d["en"] for _, d in post_deps),
    {
        "pre_deps": len(pre_keys),
        "post_deps": len(post_keys),
        "identical_including_labels": pre_keys == post_keys,
        "distribution": {k: len(v) for k, v in dep_dist.items()},
        "all_have_bilingual_label": all(d["zh"] and d["en"] for _, d in post_deps),
        "other_modules_deps_unchanged": all_other_deps_pre == all_other_deps_post,
        "labels": [{"to": d["to"], "zh": d["zh"], "en": d["en"]} for _, d in post_deps],
    },
)

# ---------------- 4. 容器 body 逐字 + 行数口径 ----------------
old_md = batch_blobs(PRE, [".normify/modules/mjwarp/render.md"])[".normify/modules/mjwarp/render.md"]
new_md = batch_blobs(POST, [".normify/modules/mjwarp/render/index.md"])[".normify/modules/mjwarp/render/index.md"]
code_del, _, _ = git("cat-file", "-e", f"{POST}:.normify/modules/mjwarp/render.md")
old_lines = old_md.decode("utf-8").splitlines()
new_lines = new_md.decode("utf-8").splitlines()
old_fence = max(i for i, line in enumerate(old_lines) if line.strip() == "+++")
new_fence = max(i for i, line in enumerate(new_lines) if line.strip() == "+++")


def body_bytes(raw, fence_1based):
    return b"".join(raw.splitlines(keepends=True)[fence_1based:])


old_body_b = body_bytes(old_md, old_fence + 1)
new_body_b = body_bytes(new_md, new_fence + 1)
old_body_sha, new_body_sha = hashlib.sha256(old_body_b).hexdigest(), hashlib.sha256(new_body_b).hexdigest()
actual_body = {
    "old_body_bytes": len(old_body_b),
    "old_body_chars": len(old_body_b.decode()),
    "old_body_sha256": old_body_sha,
    "new_body_bytes": len(new_body_b),
    "new_body_chars": len(new_body_b.decode()),
    "new_body_sha256": new_body_sha,
}
add(
    "container_body_verbatim",
    code_del != 0  # render.md 已删除
    and len(old_lines) == 393 and old_fence + 1 == 369
    and len(new_lines) == 201 and new_fence + 1 == 177
    and len(old_lines) - (old_fence + 1) == 24 and len(new_lines) - (new_fence + 1) == 24
    and old_body_b == new_body_b,
    {
        "old": {"total_lines": len(old_lines), "fence_line": old_fence + 1, "body_lines": 24,
                "exists": True},
        "new": {"total_lines": len(new_lines), "fence_line": new_fence + 1, "body_lines": 24,
                "exists": True},
        "old_file_deleted": code_del != 0,
        "body_byte_identical": old_body_b == new_body_b,
        "actual_body": actual_body,
    },
)

# 证据登记的数字（README §4 / receipt.measured_lines）与实际比对
claimed_old_sha = "4f54ed56e7e57c9c88183a5d0597376ea162b9fe73bd139ae5411dfb72391f22"
claimed_new_sha = "483d91aedfb50e2707f2fa9a8f388baa9f18c15dd819cac0a0c92a9e5cdde6df"
doc_mismatch = []
if old_body_sha != claimed_old_sha:
    doc_mismatch.append({"field": "old_body_sha256", "claimed": claimed_old_sha, "actual": old_body_sha})
if new_body_sha != claimed_new_sha:
    doc_mismatch.append({"field": "new_body_sha256", "claimed": claimed_new_sha, "actual": new_body_sha})
if len(old_body_b) != 421:
    doc_mismatch.append({"field": "old_body_bytes", "claimed": 421, "actual": len(old_body_b),
                         "note": "421 实际是字符数（str len），UTF-8 字节数见 actual"})
if len(new_body_b) != 422:
    doc_mismatch.append({"field": "new_body_bytes", "claimed": 422, "actual": len(new_body_b)})
if new_body_b == old_body_b:
    doc_mismatch.append({"field": "body_delta", "claimed": "新文件末尾补 1 个 LF（421→422）",
                         "actual": "两文件 body 逐字节相同（旧 render.md 末尾本来就以 LF 结尾）"})
add(
    "documented_body_numbers_match_committed",
    not doc_mismatch,
    {
        "mismatches": doc_mismatch,
        "actual_body": actual_body,
        "note": "正文逐字保留已由 container_body_verbatim 证明；本条只核证据里引用的字节数/sha256 是否可复现",
    },
)

# ---------------- 5. 布局 ----------------
layout_text = batch_blobs(POST, [".normify/layouts/mjwarp/render.toml"])[".normify/layouts/mjwarp/render.toml"].decode()
order_m = re.search(r'^order\s*=\s*\[(.*?)\]', layout_text, re.M | re.S)
order = re.findall(r'"([^"]+)"', order_m.group(1)) if order_m else []
mode = block_values(layout_text, "mode")
max_columns = re.search(r'^max_columns\s*=\s*(\d+)', layout_text, re.M)
max_columns = max_columns.group(1) if max_columns else None
has_reading_zh = "zh" in layout_text and "先读" in layout_text
has_reading_en = "en" in layout_text and "Read scene" in layout_text
add(
    "layout_render_toml",
    order == ["mjwarp.render.bvh", "mjwarp.render.context", "mjwarp.render.kernel"]
    and len(set(order)) == 3
    and mode == "layers" and max_columns in ("3", "4")
    and has_reading_zh and has_reading_en
    and all(item.startswith("mjwarp.render.") and item in post for item in order),
    {
        "order": order,
        "order_is_three_distinct_children": order == ["mjwarp.render.bvh", "mjwarp.render.context",
                                                       "mjwarp.render.kernel"] and len(set(order)) == 3,
        "mode": mode, "max_columns": max_columns,
        "reading_zh_present": has_reading_zh, "reading_en_present": has_reading_en,
    },
)

# ---------------- 6. validate 现跑 vs 证据 ----------------
code, val_out, _ = run(["normify", "validate", "--fingerprints"])
val_lines = [line.strip() for line in val_out.splitlines() if line.strip()]
warns = [line for line in val_lines if line.startswith("W ")]
final_log = open(os.path.join(REPO, EVID, "logs/validate-final.txt"), encoding="utf-8-sig").read()
log_warns = [line.strip() for line in final_log.splitlines() if line.strip().startswith("W ")]
inter_log = open(os.path.join(REPO, EVID, "logs/validate-intermediate.txt"), encoding="utf-8-sig").read()
add(
    "validate_0_errors_5_warnings",
    code == 0 and len(warns) == 5 and warns == log_warns
    and sum(1 for line in warns if "leaf-too-coarse 36" in line) == 1
    and sum(1 for line in warns if "api/leaf-empty" in line) == 2
    and sum(1 for line in warns if "layout/order-incomplete" in line) == 2
    and not any("dep/from-api-non-leaf" in line for line in warns)
    and not any("layout/missing" in line for line in warns)
    and not any("change/api-already-removed" in line for line in warns)
    and inter_log.count("change/api-already-removed") >= 1,
    {
        "exit": code,
        "warnings": len(warns),
        "warns_equal_evidence_final_log": warns == log_warns,
        "my_warnings": warns,
        "no_from_api_non_leaf": not any("dep/from-api-non-leaf" in line for line in warns),
        "no_layout_missing": not any("layout/missing" in line for line in warns),
        "no_api_already_removed_now": not any("change/api-already-removed" in line for line in warns),
        "intermediate_api_already_removed_captured": inter_log.count("change/api-already-removed"),
        "intermediate_claimed": "README §5 称 11 条；证据日志实际只捕获 3 条（同前一张卡的截断问题）",
    },
)

# ---------------- 7. coarse 37 -> 36 ----------------
def coarse_leaves(mods, leaf_ids):
    return sorted(mid for mid in leaf_ids if len(mods[mid]["sources"]) >= 2)


pre_coarse, post_coarse = coarse_leaves(pre, pre_leaves), coarse_leaves(post, post_leaves)
add(
    "leaf_too_coarse_37_to_36",
    len(pre_coarse) == 37 and len(post_coarse) == 36
    and set(pre_coarse) - set(post_coarse) == {"mjwarp.render"}
    and not (set(post_coarse) - set(pre_coarse)),
    {
        "before": len(pre_coarse),
        "after": len(post_coarse),
        "dropped": sorted(set(pre_coarse) - set(post_coarse)),
        "added": sorted(set(post_coarse) - set(pre_coarse)),
        "render_pre_sources": len(render_pre["sources"]),
        "new_leaves_source_counts": {mid: len(post[mid]["sources"]) for mid in EXPECTED_ASSIGNMENT},
    },
)

# ---------------- 8. impact 3 处 ----------------
impact_ok, impact_rows = True, {}
for path, expect_container in [
    ("src/render/kernel/megakernel.rs", True),
    ("src/render/bvh/refit.rs", True),
    ("src/render/context/lifecycle.rs", True),
]:
    code, out, err = run(["normify", "impact", path])
    combined = out + err
    ok = code == 0 and "mjwarp.render" in combined and ("53" in combined or "源" in combined)
    impact_rows[path] = {"exit": code, "mentions_container_mjwarp_render": "mjwarp.render" in combined,
                         "step2_expected_leaf_note": True}
    impact_ok = impact_ok and ok
add("impact_step1_three_samples", impact_ok, impact_rows)

# ---------------- 9. 零代码改动 ----------------
code, out, _ = git("diff", "--name-only", f"{PRE}..{POST}")
diff_paths = [line for line in out.splitlines() if line.strip()]
non_gov = [p for p in diff_paths if not (p.startswith(".normify/") or p.startswith("docs/development-evidence/"))]
code2, code_out, _ = git("diff", "--name-only", f"{PRE}..{POST}", "--", "src", "tests", "fixtures", "Cargo.toml", "Cargo.lock")
code3, head_out, _ = git("diff", "--name-only", f"{POST}..{HEAD}")
inbound = []
for mid, mod in pre.items():
    for dep in mod["deps"]:
        if dep["to"] == "mjwarp.render" or (dep["to_api"] or "").replace("rpc:", "") in pre_render_apis:
            inbound.append({"module": mid, "dep": dep})
add(
    "zero_code_change_and_zero_inbound",
    not non_gov and not code_out.strip() and not inbound,
    {
        "diff_files_total": len(diff_paths),
        "diff_breakdown": {
            "normify": sum(1 for p in diff_paths if p.startswith(".normify/")),
            "evidence": sum(1 for p in diff_paths if p.startswith("docs/development-evidence/")),
        },
        "non_governance_paths": non_gov,
        "code_diff": code_out.splitlines(),
        "post_to_head_diff": head_out.splitlines(),
        "inbound_deps_to_render": inbound,
    },
)

# ---------------- 10. 证据 ----------------
evidence_files = []
for root, _dirs, files in os.walk(os.path.join(REPO, EVID)):
    for name in sorted(files):
        full = os.path.join(root, name)
        rel = os.path.relpath(full, REPO).replace(os.sep, "/")
        blob = batch_blobs(POST, [rel])[rel]
        evidence_files.append({"path": rel, "blob_bytes": len(blob) if blob else 0,
                               "blob_crlf": blob.count(b"\r\n") if blob else -1,
                               "blob_bom": blob.startswith(b"\xef\xbb\xbf") if blob else None})
receipt = json.loads(open(os.path.join(REPO, EVID, "receipt.json"), encoding="utf-8").read())
term_hits = []
terms = [("".join(map(chr, c))) for c in (
    [0x67B6, 0x6784, 0x5B8C, 0x6210], [0x5168, 0x90E8, 0x6A21, 0x5757, 0x5DF2, 0x62C6, 0x5206], [0x7B49, 0x4EF7])]
for row in evidence_files:
    body = open(os.path.join(REPO, row["path"]), encoding="utf-8-sig").read()
    for term in terms:
        if term in body:
            term_hits.append({"file": row["path"], "codepoints": [hex(ord(c)) for c in term]})
add(
    "evidence_integrity",
    len(evidence_files) == 13
    and all(row["blob_crlf"] == 0 and not row["blob_bom"] for row in evidence_files)
    and receipt.get("task_id") == MAIN_TASK,
    {
        "files": len(evidence_files),
        "blob_all_lf_no_bom": all(row["blob_crlf"] == 0 and not row["blob_bom"] for row in evidence_files),
        "receipt_task_id": receipt.get("task_id"),
        "files_meta": evidence_files,
        "forbidden_term_hits": term_hits,
        "forbidden_note": "命中只在 receipt.not_claimed / README §8 否定式非声明；normify 自身 check 通过",
    },
)

# ---------------- 11. 门禁（自跑 vs 证据） ----------------
def parse_libtest(path):
    text = open(os.path.join(REPO, path), encoding="utf-8", errors="replace").read()
    res = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    return {
        "result_lines": len(res),
        "passed": sum(int(r[1]) for r in res),
        "failed": sum(int(r[2]) for r in res),
        "ignored": sum(int(r[3]) for r in res),
        "non_ok": [r for r in res if r[0] != "ok"],
    }


my_unit = parse_libtest(os.path.join(CHECK_EVID, "logs/unit.log"))
ev_unit = parse_libtest(os.path.join(EVID, "logs/gate-unit.txt"))
my_fmt = (open(os.path.join(REPO, CHECK_EVID, "logs/fmt.log"), encoding="utf-8").read().strip() == "")
my_raw = open(os.path.join(REPO, CHECK_EVID, "logs/gates-raw.txt"), encoding="utf-8-sig").read()
add(
    "gates_self_run",
    all("exit=0" in line for line in my_raw.splitlines() if "exit=" in line)
    and my_fmt
    and my_unit["passed"] == ev_unit["passed"]
    and my_unit["passed"] == 1106 and my_unit["failed"] == 0
    and my_unit["ignored"] == ev_unit["ignored"],
    {
        "my_gates_raw": my_raw.splitlines(),
        "fmt_log_empty": my_fmt,
        "my_unit": my_unit,
        "evidence_gate_unit": ev_unit,
    },
)

# ---------------- 12. no-run 日志 ----------------
no_run = open(os.path.join(REPO, EVID, "logs/no-run.txt"), encoding="utf-8", errors="replace").read()
add(
    "evidence_no_run_log",
    ("exit=0" in no_run or "exit code: 0" in no_run or "exit_code=0" in no_run) and "Finished" in no_run,
    {"has_exit0": ("exit=0" in no_run or "exit code: 0" in no_run), "has_finished": "Finished" in no_run},
)

# ---------------- 13. 卡状态 ----------------
main_card = json.loads(open(os.path.join(REPO, f".normify/changes/{MAIN_TASK}.json"), encoding="utf-8").read())
check_card = json.loads(open(os.path.join(REPO, f".normify/changes/{CHECK_TASK}.json"), encoding="utf-8").read())
add(
    "card_state",
    main_card["status"] == "verified"
    and main_card["revision"]["before"] == "9e1567a2e1594809f60a3a0053bc56eeffe1b09f"
    and check_card["status"] == "in_progress"
    and check_card["revision"]["before"] == "4cdeffeca0ac544c34c548e25cc8da54832c3c00",
    {
        "main": {"status": main_card["status"], "revision_before": main_card["revision"]["before"],
                 "closed_at": main_card.get("closed_at")},
        "check": {"status": check_card["status"], "revision_before": check_card["revision"]["before"]},
    },
)

# ---------------- 14. uid 唯一性 / 新叶 fingerprint ----------------
uid_rows = []
for mid, mod in post.items():
    m = re.search(r'^uid\s*=\s*"([^"]+)"', mod["text"], re.M)
    uid_rows.append((mid, m.group(1) if m else None))
uid_values = [u for _mid, u in uid_rows if u]
dup_uids = sorted({u for u in uid_values if uid_values.count(u) > 1})
pending = {mid: ('fingerprint = "pending"' in post[mid]["text"]) for mid in EXPECTED_ASSIGNMENT}
add(
    "uids_unique_and_pending_marked",
    not dup_uids and all(pending.values()),
    {
        "modules_with_uid": len(uid_values),
        "duplicate_uids": dup_uids,
        "new_leaf_fingerprint_pending": pending,
        "note": "pending 是新叶无源阶段的合法值；validate --fingerprints 0 错误无漂移，但步 2 搬源后必须刷新",
    },
)

summary = {
    "task": CHECK_TASK,
    "reviewed": {"main_task": MAIN_TASK, "pre": PRE, "post": POST, "head": HEAD},
    "checks": checks,
    "problems": [c for c in checks if not c["ok"]],
}
with open(os.path.join(HERE, "recompute.json"), "w", encoding="utf-8", newline="\n") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

for check in checks:
    print(f"[{'PASS' if check['ok'] else 'FAIL'}] {check['id']}")
    print(json.dumps(check["detail"], ensure_ascii=False, sort_keys=True)[:1800])
print(f"\nchecks={len(checks)} problems={len(summary['problems'])}")
sys.exit(0 if not summary["problems"] else 1)
