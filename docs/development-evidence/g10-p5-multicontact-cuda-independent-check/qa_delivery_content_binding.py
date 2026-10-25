"""QA 内容级绑定核：交付 `artifacts-sha256.txt` 的每项 vs `git show <product-head>:<path>` 的 blob 内容。

背景（Arch 2026-10-25 窗口条件 2）：交付窗口 HEAD 绑的是产品提交的父提交 `9d337bfe`，产品代码
当时在工作树未提交。本脚本按产品提交 `9a282074` 直接取 git blob 内容重算 sha256，与交付清单逐项
比对：**21/21 全同 → 内容级绑定成立**（窗口跑的是最终提交内容）；任一不同 → 立即报阻断、不重跑。

用法：python qa_delivery_content_binding.py --rev 9a282074 --json ... --log ...
"""

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
DELIVERY = REPO / "docs/development-evidence/g10-p5-multicontact-cuda"


def read_manifest():
    entries = []
    for line in (DELIVERY / "artifacts-sha256.txt").read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 2:
            raise SystemExit(f"无法解析 artifacts 行：{line!r}")
        entries.append((parts[0].strip(), parts[1].strip().lower()))
    return entries


def blob_sha256(rev, relpath):
    proc = subprocess.run(["git", "-C", str(REPO), "show", f"{rev}:{relpath}"],
                          capture_output=True)
    if proc.returncode != 0:
        return None, proc.stderr.decode("utf-8", "replace").strip()
    return hashlib.sha256(proc.stdout).hexdigest(), None


def run_git(*args):
    proc = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True)
    return (proc.returncode,
            proc.stdout.decode("utf-8", "replace"),
            proc.stderr.decode("utf-8", "replace"))


def eol_evidence(relpath, rev):
    """EOL 归一化证据：check-attr / core.autocrlf / hash-object 与提交 blob 对照。"""
    worktree = REPO / relpath
    raw = worktree.read_bytes()
    normalized = raw.replace(b"\r\n", b"\n")
    _, attr_out, _ = run_git("check-attr", "-a", "--", relpath)
    _, autocrlf, _ = run_git("config", "--get", "core.autocrlf")
    _, raw_hash, _ = run_git("hash-object", "--no-filters", relpath)
    _, clean_hash, _ = run_git("hash-object", relpath)
    _, blob_hash, _ = run_git("rev-parse", f"{rev}:{relpath}")
    proc = subprocess.run(["git", "-C", str(REPO), "show", f"{rev}:{relpath}"],
                          capture_output=True)
    blob = proc.stdout
    return {
        "path": relpath,
        "git_check_attr_a": attr_out.strip().splitlines(),
        "git_check_attr_note": ("空 = 该路径无 .gitattributes 规则，文本判定回落到 core.autocrlf"
                                if not attr_out.strip() else "见列表"),
        "core_autocrlf": autocrlf.strip(),
        "worktree_bytes": len(raw),
        "worktree_crlf_count": raw.count(b"\r\n"),
        "blob_bytes": len(blob),
        "blob_crlf_count": blob.count(b"\r\n"),
        "worktree_normalized_equals_blob": normalized == blob,
        "git_hash_object_no_filters": raw_hash.strip(),
        "git_hash_object_clean": clean_hash.strip(),
        "git_rev_parse_blob": blob_hash.strip(),
        "clean_filter_equals_committed_blob": clean_hash.strip() == blob_hash.strip(),
        "meaning": ("工作树 CRLF 经 git clean（autocrlf）归一化为 LF 后与提交 blob 完全相同 → "
                    "提交该文件不改变内容，仅是仓库 EOL 归一化行为"),
    }


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
    parser = argparse.ArgumentParser()
    parser.add_argument("--rev", default="9a282074")
    parser.add_argument("--json", default=None)
    parser.add_argument("--log", default=None)
    args = parser.parse_args()

    entries = read_manifest()
    results = []
    mismatches = []
    missing = []
    for relpath, listed in entries:
        blob, error = blob_sha256(args.rev, relpath)
        worktree = REPO / relpath
        worktree_hash = None
        if worktree.is_file():
            worktree_hash = hashlib.sha256(worktree.read_bytes()).hexdigest()
        item = {
            "path": relpath,
            "listed_sha256": listed,
            "blob_sha256": blob,
            "worktree_sha256": worktree_hash,
            "blob_matches_listed": blob == listed,
            "worktree_matches_listed": worktree_hash == listed,
        }
        if blob is None:
            item["git_error"] = error
            missing.append(item)
        elif blob != listed:
            mismatches.append(item)
        results.append(item)

    eol = []
    for item in mismatches:
        if item.get("worktree_matches_listed"):
            eol.append(eol_evidence(item["path"], args.rev))
    data = {
        "delivery_manifest": str((DELIVERY / "artifacts-sha256.txt").relative_to(REPO)),
        "product_rev": args.rev,
        "entries": len(entries),
        "blob_matches_listed": sum(1 for item in results if item["blob_matches_listed"]),
        "worktree_matches_listed": sum(1 for item in results if item["worktree_matches_listed"]),
        "mismatches": mismatches,
        "missing": missing,
        "eol_analysis": eol,
        "arch_ruling": {
            "decision": "accept-1",
            "date": "2026-10-25",
            "text": "EOL-only（CRLF<->LF 44 行尾）→ 归一化后相同 → 内容级绑定成立、不重跑；"
                    "源码/测试/清单/任务记录 20 项严格 blob 相同，artifacts 21/21 与工作树一致",
            "action": "无需重跑窗口；不改交付文件；证据中记录 check-attr/core.autocrlf/hash-object 归属",
        },
        "content_binding_ok": len(entries) > 0 and not mismatches and not missing,
        "content_binding_ok_with_eol_ruling": (
            len(entries) > 0 and not missing
            and all(item["worktree_matches_listed"] for item in mismatches)
            and all(e["worktree_normalized_equals_blob"] and e["clean_filter_equals_committed_blob"]
                    for e in eol)
            and len(mismatches) == len(eol)),
    }
    data["conclusion"] = (
        "内容级绑定成立（含 Arch 2026-10-25 裁定 accept-1）：20/21 与产品提交 blob 严格相同；"
        "唯一差异 window-output.txt 为 EOL-only（CRLF 44 处，clean/autocrlf 归一化后与提交 blob "
        "逐字节相同，hash-object 与 rev-parse 均为 %s）→ 接受、不重跑"
        % (eol[0]["git_hash_object_clean"] if eol else "n/a")
        if data["content_binding_ok_with_eol_ruling"]
        else "存在非 EOL 的内容级差异或缺失：不得放行，立即报 Arch")
    text = json.dumps(data, indent=2, ensure_ascii=False)
    print(text)
    if args.json:
        pathlib.Path(args.json).write_text(text + "\n", encoding="utf-8")
    if args.log:
        lines = [f"rev={args.rev} entries={len(entries)} "
                 f"blob_matches={data['blob_matches_listed']} "
                 f"worktree_matches={data['worktree_matches_listed']} "
                 f"content_binding_ok={data['content_binding_ok']}"]
        for item in results:
            lines.append(f"{item['path']}: blob_ok={item['blob_matches_listed']} "
                         f"worktree_ok={item['worktree_matches_listed']}")
        for e in eol:
            lines.append(f"eol_analysis[{e['path']}]: core.autocrlf={e['core_autocrlf']} "
                         f"check_attr={e['git_check_attr_a']} "
                         f"crlf worktree={e['worktree_crlf_count']} blob={e['blob_crlf_count']} "
                         f"normalized_equals_blob={e['worktree_normalized_equals_blob']} "
                         f"clean_filter_equals_blob={e['clean_filter_equals_committed_blob']} "
                         f"hash_object_clean={e['git_hash_object_clean']} "
                         f"blob={e['git_rev_parse_blob']}")
        pathlib.Path(args.log).write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0 if data["content_binding_ok_with_eol_ruling"] else 1


if __name__ == "__main__":
    sys.exit(main())
