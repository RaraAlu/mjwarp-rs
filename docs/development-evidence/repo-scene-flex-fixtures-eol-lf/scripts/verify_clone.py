"""R06 scene-flex 文本夹具 eol=lf —— fresh clone（autocrlf=true）对照验证。

- before：clone 于「无规则」提交（当前 HEAD 父提交），3 个文本夹具应 CRLF → sha/bytes 与 G-2 files-sha256.json 不一致；
- after：把工作树 `.gitattributes`（含新规则，逐字节同最终提交）复制进 clone 并 `git restore --source=HEAD :/`
  重新物化 → 3 个文本夹具应回到 LF、sha/bytes 与 G-2 记录一致，12 个 npz 始终逐字节一致；
- 输出：target/repo-scene-flex-fixtures-eol-lf/clone-compare.json（机器可读）+ 控制台摘要。
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import shutil
import subprocess

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
CLONE = pathlib.Path(r"D:\eol-clone")
OUT = REPO / "target/repo-scene-flex-fixtures-eol-lf"
EXPECTED = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2/artifacts/files-sha256.json"
FIXTURE_REL = "fixtures/render-bvh/scene-flex/multiframe"


def git(clone: bool, *argv: str) -> subprocess.CompletedProcess:
    cwd = CLONE if clone else REPO
    return subprocess.run(["git", *argv], cwd=cwd, capture_output=True, text=True)


def file_entry(path: pathlib.Path) -> dict:
    data = path.read_bytes()
    return {"path": path.as_posix(), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
            "crlf_count": data.count(b"\r\n")}


def _force_rmtree(path: pathlib.Path) -> None:
    """Windows 上 git pack 文件常带只读位，清掉只读再删。"""
    import os
    import stat

    def onerror(func, target, _exc):
        try:
            os.chmod(target, stat.S_IWRITE)
            func(target)
        except Exception:
            pass

    shutil.rmtree(path, onerror=onerror)


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    # 每次从干净 clone 开始（自包含；autocrlf=true 复现 QA O1 场景）
    if CLONE.exists():
        _force_rmtree(CLONE)
    clone_proc = subprocess.run(
        ["git", "-c", "core.longpaths=true", "-c", "core.autocrlf=true", "clone", "--no-local", str(REPO), str(CLONE)],
        capture_output=True, text=True,
    )
    assert clone_proc.returncode == 0, clone_proc.stderr[-500:]
    print("clone HEAD", git(True, "rev-parse", "HEAD").stdout.strip()[:12],
          "| status_lines", len(git(True, "status", "--porcelain").stdout.splitlines()))
    base_head = git(True, "rev-parse", "HEAD").stdout.strip()
    expected = json.loads(EXPECTED.read_text(encoding="utf-8"))
    expected_by_rel = {entry["path"]: entry for entry in expected["files"]}  # 已是仓相对路径
    text_rel = [f"{FIXTURE_REL}/capture.json", f"{FIXTURE_REL}/cam-poses.json", f"{FIXTURE_REL}/README.md"]
    npz_rel = [f"{FIXTURE_REL}/" + entry for entry in
               ("frame-000/depth.npz", "frame-000/rgb.npz", "frame-000/seg.npz", "frame-000/state.npz",
                "frame-001/depth.npz", "frame-001/rgb.npz", "frame-001/seg.npz", "frame-001/state.npz",
                "frame-002/depth.npz", "frame-002/rgb.npz", "frame-002/seg.npz", "frame-002/state.npz")]
    assert len(npz_rel) == 12, len(npz_rel)

    def snapshot(label: str) -> dict:
        files = {rel: file_entry(CLONE / rel) for rel in text_rel + npz_rel}
        matches = {rel: (files[rel]["bytes"] == expected_by_rel[rel]["bytes"]
                         and files[rel]["sha256"] == expected_by_rel[rel]["sha256"]) for rel in files}
        return {"label": label, "files": files, "match_expected": matches,
                "text_match": all(matches[rel] for rel in text_rel),
                "npz_match": all(matches[rel] for rel in npz_rel),
                "clone_git_status": git(True, "status", "--porcelain").stdout.splitlines()}

    before = snapshot("clone-before-rule")
    print(f"before: HEAD={git(True, 'rev-parse', 'HEAD').stdout.strip()[:12]} text_match={before['text_match']} "
          f"npz_match={before['npz_match']} status_lines={len(before['clone_git_status'])}")
    for rel in text_rel:
        print(f"  before {rel}: bytes={before['files'][rel]['bytes']} crlf={before['files'][rel]['crlf_count']} "
              f"match={before['match_expected'][rel]}")

    # 把工作树 .gitattributes（含新规则）复制进 clone，做 clone-local 临时提交（clone 无本仓 hooks，
    # 仅用于模拟「fresh clone 于含规则提交」），再 reset --hard 强制按新属性重新物化。
    parent_attr = REPO / ".gitattributes"
    shutil.copyfile(parent_attr, CLONE / ".gitattributes")
    git(True, "add", ".gitattributes")
    commit = git(True, "-c", "user.name=eol-test", "-c", "user.email=eol-test@local",
                 "commit", "-m", "temp: eol rule (clone-local, throwaway)")
    print("clone-local commit exit", commit.returncode)
    # reset --hard 不会重写「index 与 HEAD 相同」的文件；删除后再 checkout 强制按已提交规则重新物化
    for rel in text_rel:
        (CLONE / rel).unlink()
    checkout = git(True, "checkout", "--", *text_rel)
    print("checkout exit", checkout.returncode, checkout.stderr.strip()[:120])
    clone_local_head = git(True, "rev-parse", "HEAD").stdout.strip()
    after = snapshot("clone-with-rule")
    after = snapshot("clone-with-rule")
    print(f"after: text_match={after['text_match']} npz_match={after['npz_match']} "
          f"status_lines={len(after['clone_git_status'])}")
    for rel in text_rel:
        print(f"  after {rel}: bytes={after['files'][rel]['bytes']} crlf={after['files'][rel]['crlf_count']} "
              f"match={after['match_expected'][rel]}")

    attr_after = {
        "text": git(True, "check-attr", "text", "--", *text_rel).stdout.splitlines(),
        "eol": git(True, "check-attr", "eol", "--", *text_rel).stdout.splitlines(),
        "npz_text": git(True, "check-attr", "text", "--", *npz_rel[:2]).stdout.splitlines(),
        "npz_diff": git(True, "check-attr", "diff", "--", *npz_rel[:2]).stdout.splitlines(),
    }
    after_dirty = after["clone_git_status"]
    after_dirty_paths = [line[3:].strip() for line in after_dirty]
    report = {
        "clone": {"path": CLONE.as_posix(),
                  "base_head": base_head,
                  "clone_local_rule_commit": clone_local_head,
                  "core_autocrlf": git(True, "config", "core.autocrlf").stdout.strip(),
                  "rule_file_sha256": hashlib.sha256(parent_attr.read_bytes()).hexdigest()},
        "expected_source": EXPECTED.relative_to(REPO).as_posix(),
        "before_rule": before,
        "after_rule": after,
        "after_dirty_paths": after_dirty_paths,
        "attributes_after": attr_after,
        "verdict": {
            "before_text_drift_reproduced": before["text_match"] is False,
            "after_text_match": after["text_match"],
            "after_npz_match": after["npz_match"],
            "after_clone_clean": after_dirty_paths == [],
            "before_npz_match": before["npz_match"],
        },
    }
    (OUT / "clone-compare.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n",
                                            encoding="utf-8", newline="\n")
    print("verdict:", json.dumps(report["verdict"], ensure_ascii=False))
    return 0 if all(report["verdict"].values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
