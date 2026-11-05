"""落库后真实 HEAD fresh clone 复核（autocrlf=true）：规则已在最终提交中。

输出 JSON：target/repo-scene-flex-fixtures-eol-lf/clone-postcommit.json
检查：clone 干净；15 个夹具（3 文本 + 12 npz）sha256/bytes 与 G-2 files-sha256.json 逐条一致；
      3 文本夹具无 CRLF；npz 仍 text/diff unset；clone 的 .gitattributes 含规则。
"""

from __future__ import annotations

import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
CLONE = pathlib.Path(r"D:\eol-clone-postcommit")
OUT = REPO / "target/repo-scene-flex-fixtures-eol-lf"
EXPECTED = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2/artifacts/files-sha256.json"


def _force_rmtree(path: pathlib.Path) -> None:
    def onerror(func, target, _exc):
        try:
            os.chmod(target, stat.S_IWRITE)
            func(target)
        except Exception:
            pass

    shutil.rmtree(path, onerror=onerror)


def git(*argv: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(CLONE), *argv], capture_output=True, text=True)


def main() -> int:
    if CLONE.exists():
        _force_rmtree(CLONE)
    clone = subprocess.run(
        ["git", "-c", "core.longpaths=true", "-c", "core.autocrlf=true", "clone", "--no-local",
         str(REPO), str(CLONE)],
        capture_output=True, text=True,
    )
    assert clone.returncode == 0, clone.stderr[-500:]
    head = git("rev-parse", "HEAD").stdout.strip()
    status = git("status", "--porcelain").stdout.splitlines()
    autocrlf = git("config", "core.autocrlf").stdout.strip()
    expected = json.loads(EXPECTED.read_text(encoding="utf-8"))
    files = {}
    for entry in expected["files"]:
        data = (CLONE / entry["path"]).read_bytes()
        files[entry["path"]] = {
            "bytes": len(data),
            "crlf_count": data.count(b"\r\n"),
            "sha256": hashlib.sha256(data).hexdigest(),
            "match": len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"],
        }
    text_rel = [p for p in files if p.endswith((".json", ".md"))]
    npz_rel = [p for p in files if p.endswith(".npz")]
    attr_text = git("check-attr", "text", "--", *text_rel).stdout.splitlines()
    attr_eol = git("check-attr", "eol", "--", *text_rel).stdout.splitlines()
    attr_npz_text = git("check-attr", "text", "--", *npz_rel[:2]).stdout.splitlines()
    attr_npz_diff = git("check-attr", "diff", "--", *npz_rel[:2]).stdout.splitlines()
    rule_present = "fixtures/render-bvh/scene-flex/** text eol=lf" in (
        (CLONE / ".gitattributes").read_text(encoding="utf-8"))
    report = {
        "clone": {"path": CLONE.as_posix(), "head": head, "core_autocrlf": autocrlf,
                  "status_lines": status, "clean": status == []},
        "expected_source": EXPECTED.relative_to(REPO).as_posix(),
        "files": files,
        "attributes": {"text": attr_text, "eol": attr_eol, "npz_text": attr_npz_text, "npz_diff": attr_npz_diff},
        "rule_present_in_clone": rule_present,
        "verdict": {
            "clone_clean": status == [],
            "all_15_match": all(entry["match"] for entry in files.values()),
            "text_lf_no_crlf": all(files[p]["crlf_count"] == 0 for p in text_rel),
            "npz_binary_attrs": all("unset" in line for line in attr_npz_text + attr_npz_diff),
            "rule_present": rule_present,
        },
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "clone-postcommit.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n",
                                               encoding="utf-8", newline="\n")
    print(f"HEAD {head[:12]} | clean={report['verdict']['clone_clean']} "
          f"| 15_files_match={report['verdict']['all_15_match']} | text_no_crlf={report['verdict']['text_lf_no_crlf']} "
          f"| npz_binary={report['verdict']['npz_binary_attrs']} | rule_present={rule_present}")
    return 0 if all(report["verdict"].values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
