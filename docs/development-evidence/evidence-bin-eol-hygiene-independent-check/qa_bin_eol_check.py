"""QA 独立复核：证据 `.bin`（及同类二进制）在 `core.autocrlf=true` 下的 checkout 卫生。

做法：
1. 主仓 HEAD 上，1130 个已跟踪 `.bin` 逐文件 sha256 与 blob 对比（工作树未被破坏）；
2. 自建 clone（`target/qa-bin-eol-check`，`--shared --no-checkout`，不复用 Backend 的
   `target/bin-eol*`），配 `core.autocrlf=true` + `core.longpaths=true`；
3. 逆向判别：checkout 修复前提交（`e00de497`），逐个 `.bin` 比对 blob 字节与工作树字节，
   列出被 EOL 转换破坏的文件（size 变化 / sha256 变化 / 是否纯 `\\n`→`\\r\\n` 插入），
   并确认 `git status` 仍为干净（静默损坏）；
4. 对照：checkout 修复后提交（`526e7659`）后 1130/1130 字节一致；
5. `git check-attr` 抽样（覆盖 docs/development-evidence/**、fixtures/**、include/** 等）确认
   `.bin` 为 `text: unset` / `diff: unset`；
6. 范围核对：修复提交只动 `.gitattributes` + `.normify` 记录 + 新增证据目录，线性历史，未动
   `Cargo.*` / `fixtures/mjwarp-complex-physics/*.xml` / S2–S8 证据。

用法：python qa_bin_eol_check.py [--repo D:\\Rust\\mjwarp-rs] [--out qa-bin-eol-check.json]
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

PRE_FIX = "e00de497"
POST_FIX = "526e7659"
CLONE_NAME = "qa-bin-eol-check"
# .gitattributes 修复覆盖的 5 类二进制扩展名（见 526e7659）
BINARY_EXTS = (".bin", ".f32", ".npz", ".gz", ".mjb")


def run(cmd, cwd=None, check=True, input_bytes=None):
    p = subprocess.run(cmd, cwd=cwd, capture_output=True, input=input_bytes)
    if check and p.returncode != 0:
        raise RuntimeError(
            "command failed (%s): %s\nstdout:\n%s\nstderr:\n%s"
            % (p.returncode, cmd, p.stdout[-2000:], p.stderr[-4000:])
        )
    return p


def git(cwd, *args, check=True, input_bytes=None):
    return run(["git", "-c", "core.longpaths=true", *args], cwd=cwd, check=check,
               input_bytes=input_bytes)


def read_bytes(path):
    """读文件原始字节；Windows 长路径加 \\\\?\\ 前缀。"""
    p = os.path.abspath(str(path))
    if os.name == "nt" and len(p) >= 240 and not p.startswith("\\\\?\\"):
        p = "\\\\?\\" + p
    with open(p, "rb") as f:
        return f.read()


def rmtree(path):
    path = Path(path)
    if not path.exists():
        return
    if os.name == "nt":
        subprocess.run(["cmd", "/c", "rd", "/s", "/q", "\\\\?\\" + str(path.resolve())],
                       capture_output=True)
    if path.exists():
        shutil.rmtree(path, ignore_errors=True)
    if path.exists():
        raise RuntimeError("cannot remove previous clone: %s" % path)


def ls_binary_blobs(repo, commit):
    out = git(repo, "ls-tree", "-r", "-z", "--full-tree", commit).stdout
    entries = {}
    for rec in out.split(b"\0"):
        if not rec:
            continue
        meta, path = rec.split(b"\t", 1)
        _mode, typ, sha = meta.split(b" ")
        assert typ == b"blob", (typ, path)
        p = path.decode("utf-8")
        if p.endswith(BINARY_EXTS):
            entries[p] = sha.decode("ascii")
    return entries


def cat_blobs(repo, shas):
    inp = ("\n".join(sorted(set(shas))) + "\n").encode("ascii")
    p = git(repo, "cat-file", "--batch", input_bytes=inp)
    data = p.stdout
    out = {}
    i = 0
    while i < len(data):
        nl = data.index(b"\n", i)
        header = data[i:nl].decode("ascii")
        i = nl + 1
        sha, _typ, size = header.split()
        size = int(size)
        out[sha] = data[i:i + size]
        i += size + 1
    return out


def compare_commit(repo, commit):
    blobs = ls_binary_blobs(repo, commit)
    raw = cat_blobs(repo, blobs.values())
    identical = 0
    mismatches = []
    for path in sorted(blobs):
        blob = raw[blobs[path]]
        target = Path(repo) / path
        try:
            work = read_bytes(target)
        except FileNotFoundError:
            mismatches.append({"path": path, "error": "missing_in_worktree"})
            continue
        if work == blob:
            identical += 1
            continue
        rec = {
            "path": path,
            "blob_size": len(blob),
            "worktree_size": len(work),
            "size_delta": len(work) - len(blob),
            "sha256_blob": hashlib.sha256(blob).hexdigest(),
            "sha256_worktree": hashlib.sha256(work).hexdigest(),
            "n_lf_in_blob": blob.count(b"\n"),
            "n_crlf_in_worktree": work.count(b"\r\n"),
            "pure_crlf_insertion": blob.replace(b"\n", b"\r\n") == work,
        }
        k = 0
        while k < min(len(blob), len(work)) and blob[k] == work[k]:
            k += 1
        rec["first_diff_offset"] = k
        rec["diff_bytes"] = [
            {"offset": k, "blob": blob[k:k + 1].hex(), "worktree": work[k:k + 1].hex()}
        ]
        mismatches.append(rec)
    families = {}
    for path in blobs:
        ext = "." + path.rsplit(".", 1)[-1]
        families[ext] = families.get(ext, 0) + 1
    mismatch_families = {}
    for m in mismatches:
        ext = "." + m["path"].rsplit(".", 1)[-1]
        mismatch_families[ext] = mismatch_families.get(ext, 0) + 1
    return {
        "commit": commit,
        "extension_filter": list(BINARY_EXTS),
        "tracked_binary": len(blobs),
        "family_counts": families,
        "identical": identical,
        "mismatch_count": len(mismatches),
        "mismatch_families": mismatch_families,
        "mismatches": mismatches,
    }


def status_clean(repo):
    out = git(repo, "status", "--porcelain", "--untracked-files=no").stdout.decode("utf-8", "replace")
    return {"porcelain_untracked_ignored": out.strip(), "clean": out.strip() == ""}


def check_attr(repo, paths):
    if not paths:
        return []
    grouped = {}
    # Windows 命令行长度限制：分批查询（每批 64 个路径）
    for start in range(0, len(paths), 64):
        chunk = paths[start:start + 64]
        p = git(repo, "check-attr", "-z", "text", "diff", "eol", "--", *chunk)
        fields = p.stdout.split(b"\0")
        i = 0
        while i + 2 < len(fields):
            if fields[i] == b"":
                i += 1
                continue
            grouped.setdefault(fields[i].decode("utf-8", "replace"), {})[
                fields[i + 1].decode()] = fields[i + 2].decode()
            i += 3
    return [{"path": k, **v} for k, v in sorted(grouped.items())]


def pick_samples(repo):
    """抽样：覆盖 docs/development-evidence/**、fixtures/**、include/**、docs/evidence/**、src/**。"""
    tracked = git(repo, "ls-files", "-z").stdout.split(b"\0")
    tracked = [p.decode("utf-8") for p in tracked if p]
    samples = []

    def take(pred, n=2):
        out = []
        for p in tracked:
            if pred(p):
                out.append(p)
                if len(out) >= n:
                    break
        return out

    bins = [p for p in tracked if p.endswith(BINARY_EXTS)]
    samples += take(lambda p: p.startswith("docs/development-evidence/") and p.endswith(".bin"), 3)
    samples += take(lambda p: p.startswith("docs/development-evidence/") and p.endswith(".json"), 2)
    samples += take(lambda p: p.startswith("docs/development-evidence/") and p.endswith(".md"), 1)
    samples += take(lambda p: p.startswith("fixtures/") and (p.endswith(".xml") or p.endswith(".json")), 2)
    samples += take(lambda p: p.startswith("fixtures/") and p.endswith(".mjb"), 1)
    samples += take(lambda p: p.startswith("fixtures/mjwarp-complex-physics/") and p.endswith(".xml"), 1)
    samples += take(lambda p: p.startswith("fixtures/") and p.endswith(".json"), 1)
    samples += take(lambda p: p.startswith("include/"), 2)
    samples += take(lambda p: p.startswith("docs/evidence/") and p.endswith(".json"), 1)
    samples += take(lambda p: p.startswith("src/") and p.endswith(".rs"), 1)
    samples += take(lambda p: p == ".gitattributes", 1)
    # 全部 tracked binary 的属性单独全量核一遍（期望 text=unset, diff=unset）
    return samples, bins


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=r"D:\Rust\mjwarp-rs")
    ap.add_argument("--out", default="qa-bin-eol-check.json")
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    evidence = Path(__file__).resolve().parent
    out_path = evidence / args.out

    clone_pre = repo / "target" / CLONE_NAME
    clone_post = repo / "target" / (CLONE_NAME + "-after")
    for p in (clone_pre, clone_post):
        assert p.parent == repo / "target", p
        assert p.name in (CLONE_NAME, CLONE_NAME + "-after"), p
        assert p.name not in ("bin-eol", "bin-eol-verify"), "must not reuse Backend clone dirs"

    backend_dirs = sorted(p.name for p in (repo / "target").glob("*bin-eol*")
                          if not p.name.startswith(CLONE_NAME))
    head = git(repo, "rev-parse", "HEAD").stdout.decode().strip()
    git_version = git(repo, "version").stdout.decode().strip()

    report = {
        "schema": 1,
        "kind": "qa-bin-eol-checkout-hygiene",
        "task_id": "2026-10-25-evidence-bin-eol-hygiene-independent-check",
        "env": {
            "git": git_version,
            "python": sys.version.split()[0],
            "platform": os.name,
        },
        "repo": {
            "path": str(repo),
            "head": head,
            "pre_fix": PRE_FIX,
            "post_fix": POST_FIX,
            "post_fix_parent": git(repo, "rev-parse", POST_FIX + "^").stdout.decode().strip(),
            "pre_fix_title": git(repo, "log", "-1", "--format=%s", PRE_FIX).stdout.decode().strip(),
            "post_fix_title": git(repo, "log", "-1", "--format=%s", POST_FIX).stdout.decode().strip(),
        },
        "isolation": {
            "clone_paths": [str(clone_pre), str(clone_post)],
            "backend_paths_present": backend_dirs,
            "reused_backend_clone": False,
            "reused_backend_logs": False,
            "note": "只读主仓对象库（--shared clone 用 alternates），未读写 Backend 的 target/bin-eol* 内容",
        },
    }

    # 1) 主仓 HEAD：已跟踪 .bin 与 blob 100% 对比
    report["head_worktree"] = compare_commit(repo, head)

    # 2) 抽样 + 全量 attribute
    samples, bins = pick_samples(repo)
    report["attribute_samples"] = check_attr(repo, samples)
    family_attrs = check_attr(repo, bins)
    by_family = {}
    for rec in family_attrs:
        ext = "." + rec["path"].rsplit(".", 1)[-1]
        d = by_family.setdefault(ext, {"checked": 0, "text_unset": 0, "diff_unset": 0, "violations": 0})
        d["checked"] += 1
        if rec.get("text") == "unset":
            d["text_unset"] += 1
        if rec.get("diff") == "unset":
            d["diff_unset"] += 1
        if rec.get("text") != "unset" or rec.get("diff") != "unset":
            d["violations"] += 1
    report["binary_attribute_summary"] = {
        "checked": len(family_attrs),
        "text_unset": sum(1 for r in family_attrs if r.get("text") == "unset"),
        "diff_unset": sum(1 for r in family_attrs if r.get("diff") == "unset"),
        "by_family": by_family,
        "violations": [
            r for r in family_attrs
            if r.get("text") != "unset" or r.get("diff") != "unset"
        ][:20],
    }
    report["attribute_samples_binary_only"] = family_attrs[:5]

    # 3) 自建 clone（--shared --no-checkout），autocrlf=true。两个独立 clone：
    #    - before：checkout 修复前提交，复现 EOL 破坏（逆向判别力）
    #    - after ：全新 clone 直接 checkout 修复后提交（对照，避免「原地升级不重写未变 blob」干扰）

    def fresh_clone(dst):
        rmtree(dst)
        run(["git", "-c", "core.longpaths=true", "clone", "--shared", "--no-checkout",
             str(repo), str(dst)])
        git(dst, "config", "core.longpaths", "true")
        git(dst, "config", "core.autocrlf", "true")
        return {
            "path": str(dst),
            "config_autocrlf": git(dst, "config", "core.autocrlf").stdout.decode().strip(),
            "config_longpaths": git(dst, "config", "core.longpaths").stdout.decode().strip(),
        }

    report["clone"] = {"before": fresh_clone(clone_pre), "after": fresh_clone(clone_post)}

    # 4) 逆向判别：修复前提交（fresh clone + autocrlf=true）
    git(clone_pre, "checkout", "-q", "-f", PRE_FIX)
    report["pre_fix"] = compare_commit(clone_pre, PRE_FIX)
    report["pre_fix"]["status_after_checkout"] = status_clean(clone_pre)
    report["pre_fix"]["gitattributes_blob"] = {
        "path": ".gitattributes",
        "commit": PRE_FIX,
        "present": git(clone_pre, "cat-file", "-e", PRE_FIX + ":.gitattributes", check=False).returncode == 0,
        "has_bin_rule": "*.bin" in git(clone_pre, "cat-file", "-p", PRE_FIX + ":.gitattributes").stdout.decode("utf-8", "replace"),
    }

    # 5) 对照：修复后提交，全新 clone（fresh checkout）
    git(clone_post, "checkout", "-q", "-f", POST_FIX)
    report["post_fix"] = compare_commit(clone_post, POST_FIX)
    report["post_fix"]["status_after_checkout"] = status_clean(clone_post)
    report["post_fix"]["clone_kind"] = "fresh clone at post-fix commit"

    # 5b) 观察项：原地从修复前 checkout 到修复后（未变 blob 不会被重写）
    git(clone_pre, "checkout", "-q", "-f", POST_FIX)
    report["in_place_upgrade"] = compare_commit(clone_pre, POST_FIX)
    report["in_place_upgrade"]["status_after_checkout"] = status_clean(clone_pre)
    report["in_place_upgrade"]["note"] = (
        "原地升级（先 checkout 修复前、再 checkout 修复后）不会重写 blob 未变的文件："
        "7 个已损坏文件保持损坏，且索引里仍留着 checkout 时的 stat 信息，`git status` 依旧干净"
        "（触碰该文件、stat 变化后才显示 M）。修复保证的是「修复后全新 checkout/clone」得到正确字节；"
        "对已存在的损坏工作树需要 `git checkout -- .` 或重新 clone 才能修复。"
    )

    # 6) 范围与历史
    parent = report["repo"]["post_fix_parent"]
    status = git(repo, "show", "--name-status", "--format=", POST_FIX).stdout.decode("utf-8", "replace")
    allowed_prefixes = (".gitattributes", ".normify/",
                        "docs/development-evidence/evidence-bin-eol-hygiene/")
    entries = []
    violations = []
    for line in status.splitlines():
        if not line.strip():
            continue
        parts = line.split("\t")
        st, path = parts[0], parts[-1]
        entries.append({"status": st, "path": path})
        if st == "M" and path in (".gitattributes",):
            continue
        if not path.startswith(allowed_prefixes):
            violations.append({"status": st, "path": path})
    forbidden_hits = [
        e for e in entries
        if e["path"].startswith(("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"))
        or e["path"].startswith("fixtures/mjwarp-complex-physics/")
        or e["path"].startswith(("docs/evidence/", "docs/development-evidence/s2", "docs/development-evidence/s3"))
        or ("/s2-" in e["path"] or "/s3-" in e["path"] or "/s4-" in e["path"]
            or "/s5-" in e["path"] or "/s6-" in e["path"] or "/s7-" in e["path"] or "/s8-" in e["path"])
    ]
    merges = git(repo, "rev-list", "--merges", POST_FIX, "--not", PRE_FIX).stdout.decode().strip()
    ancestor = git(repo, "merge-base", "--is-ancestor", POST_FIX, head, check=False).returncode == 0
    parent_ok = git(repo, "merge-base", "--is-ancestor", parent, POST_FIX, check=False).returncode == 0
    report["scope"] = {
        "commit": POST_FIX,
        "parent": parent,
        "changed_entries": entries,
        "allowed_prefixes": list(allowed_prefixes),
        "violations": violations,
        "forbidden_hits": forbidden_hits,
        "merge_commits_between": merges,
        "is_ancestor_of_head": ancestor,
        "parent_is_ancestor_of_commit": parent_ok,
        "linear_history": parent_ok and merges == "",
        "cargo_or_fixture_xml_or_s2s8_changed": bool(forbidden_hits),
    }

    # 判定
    report["verdict"] = {
        "head_worktree_all_identical": report["head_worktree"]["mismatch_count"] == 0,
        "pre_fix_mismatches": report["pre_fix"]["mismatch_count"],
        "pre_fix_silent": report["pre_fix"]["status_after_checkout"]["clean"],
        "post_fix_fresh_clone_all_identical": report["post_fix"]["mismatch_count"] == 0,
        "in_place_upgrade_mismatches_observed": report["in_place_upgrade"]["mismatch_count"],
        "in_place_upgrade_status_still_clean": report["in_place_upgrade"]["status_after_checkout"]["clean"],
        "in_place_upgrade_needs_manual_repair": report["in_place_upgrade"]["mismatch_count"] > 0,
        "binary_attr_violations": len(report["binary_attribute_summary"]["violations"]),
        "scope_violations": len(violations) + len(forbidden_hits),
        "linear_history": report["scope"]["linear_history"],
        "pass": (
            report["head_worktree"]["mismatch_count"] == 0
            and report["pre_fix"]["mismatch_count"] > 0
            and report["post_fix"]["mismatch_count"] == 0
            and len(report["binary_attribute_summary"]["violations"]) == 0
            and len(violations) == 0
            and len(forbidden_hits) == 0
            and report["scope"]["linear_history"]
            and ancestor
        ),
    }

    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("== repo ==")
    print("  head            ", head)
    print("  pre_fix         ", PRE_FIX, "| post_fix", POST_FIX, "| parent", parent)
    print("  clone before   ", clone_pre)
    print("  clone after    ", clone_post)
    print("  backend dirs present:", backend_dirs)
    print("== head worktree ==")
    print("  tracked binary %d, identical %d, mismatch %d"
          % (report["head_worktree"]["tracked_binary"],
             report["head_worktree"]["identical"],
             report["head_worktree"]["mismatch_count"]))
    print("== attributes ==")
    print("  binary checked %d, text unset %d, diff unset %d, violations %d"
          % (report["binary_attribute_summary"]["checked"],
             report["binary_attribute_summary"]["text_unset"],
             report["binary_attribute_summary"]["diff_unset"],
             len(report["binary_attribute_summary"]["violations"])))
    for s in report["attribute_samples"]:
        print("   sample %-70s text=%s diff=%s eol=%s"
              % (s["path"], s.get("text"), s.get("diff"), s.get("eol")))
    print("== pre-fix checkout (autocrlf=true) ==")
    print("  tracked binary %d, corrupted %d, status clean %s"
          % (report["pre_fix"]["tracked_binary"],
             report["pre_fix"]["mismatch_count"],
             report["pre_fix"]["status_after_checkout"]["clean"]))
    for m in report["pre_fix"]["mismatches"]:
        print("   %s  %d -> %d bytes (delta %+d)  lf=%d  pure_crlf=%s"
              % (m.get("path"), m.get("blob_size", -1), m.get("worktree_size", -1),
                 m.get("size_delta", 0), m.get("n_lf_in_blob", -1),
                 m.get("pure_crlf_insertion")))
        print("      sha256 blob=%s worktree=%s first_diff=%s"
              % (m.get("sha256_blob"), m.get("sha256_worktree"), m.get("first_diff_offset")))
    print("== post-fix fresh clone (autocrlf=true) ==")
    print("  tracked binary %d, identical %d, mismatch %d, status clean %s"
          % (report["post_fix"]["tracked_binary"],
             report["post_fix"]["identical"],
             report["post_fix"]["mismatch_count"],
             report["post_fix"]["status_after_checkout"]["clean"]))
    print("== in-place upgrade (pre-fix checkout -> post-fix, observation) ==")
    print("  mismatch %d (git 不重写未变 blob), status clean %s; 修复只保证 fresh checkout，"
          "已损坏工作树需 git checkout -- . 或重新 clone"
          % (report["in_place_upgrade"]["mismatch_count"],
             report["in_place_upgrade"]["status_after_checkout"]["clean"]))
    print("== scope ==")
    print("  entries %d, violations %d, forbidden %d, merges '%s', linear %s, ancestor %s"
          % (len(entries), len(violations), len(forbidden_hits), merges,
             report["scope"]["linear_history"], ancestor))
    for e in entries:
        print("   %s %s" % (e["status"], e["path"]))
    print("== verdict ==")
    print(" ", json.dumps(report["verdict"], ensure_ascii=False))
    print("report:", out_path)
    return 0 if report["verdict"]["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
