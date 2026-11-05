"""G-2 落库夹具的 fresh clone 字节安全复核（core.autocrlf=true；只读仓库，clone 落 target/）。

做法：`git clone --no-checkout` → `core.longpaths=true` + `core.autocrlf=true` checkout HEAD，
把 clone 工作树与本机证据 `artifacts/files-sha256.json`（15 个落库文件）逐文件比对 sha256/bytes，
并抽核证据目录 manifest（`-text` 规则）在 clone 中是否同样自洽。

判定口径：
  * `*.npz`（12 个参考数据文件）必须逐字节一致——`.gitattributes` 的 `*.npz -text -diff` 应保证；
  * 3 个文本夹件（capture.json / cam-poses.json / README.md）若缺 eol 固定规则，autocrlf=true
    的 checkout 会 LF→CRLF，导致 sha/bytes 与证据记录不一致 → 记为观察（不影响 npz 参考数据）。

用法（仓库根）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qag2-verify/verify_fresh_clone.py
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding="utf-8", errors="replace", newline="\n")
    except Exception:  # pragma: no cover
        pass

REPO = next(p for p in pathlib.Path(__file__).resolve().parents
            if (p / "Cargo.toml").is_file() and (p / "scripts/authoring").is_dir())
CLONE = REPO / "target/qag2-verify/clone"
EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2"
FILES_SHA = json.loads((EVID / "artifacts/files-sha256.json").read_text(encoding="utf-8"))
TARGET_REV = "38a7767c"  # 复核目标提交（G-2 证据补充收口；fixtures 自 5f2bd0bb 起）、可用 QA_TARGET_REV 覆盖


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(args, capture_output=True)


def main() -> int:
    target = (REPO / ".git")  # 仅用于类型占位，避免误删
    CLONE.parent.mkdir(parents=True, exist_ok=True)
    if not CLONE.is_dir():
        cp = run(["git", "clone", "--no-checkout", "--quiet", str(REPO), str(CLONE)])
        print(f"clone: exit={cp.returncode}")
    else:
        run(["git", "-C", str(CLONE), "fetch", "--quiet", "origin"])
        print("clone: reused existing target/qag2-verify/clone (fetched origin)")
    run(["git", "-C", str(CLONE), "config", "core.longpaths", "true"])
    run(["git", "-C", str(CLONE), "config", "core.autocrlf", "true"])
    ck = run(["git", "-C", str(CLONE), "-c", "core.autocrlf=true", "checkout", "-f", TARGET_REV])
    print(f"checkout (core.autocrlf=true) target={TARGET_REV}: exit={ck.returncode}")
    head = run(["git", "-C", str(CLONE), "rev-parse", "--short", "HEAD"]).stdout.decode().strip()
    print(f"clone HEAD={head}")
    print(f"clone core.autocrlf={run(['git', '-C', str(CLONE), 'config', 'core.autocrlf']).stdout.decode().strip()}")

    print("\n== 落库 15 文件：clone 工作树 vs 证据 files-sha256.json ==")
    print(f"{'file':40s} {'recorded bytes':>14s} {'clone bytes':>11s} {'sha match':>9s} {'CRLF in clone':>13s}")
    mismatch = []
    for item in FILES_SHA["files"]:
        rel = item["path"]
        raw = (CLONE / rel).read_bytes()
        match = sha(raw) == item["sha256"] and len(raw) == item["bytes"]
        if not match:
            mismatch.append(rel)
        print(f"{rel.split('multiframe/')[1]:40s} {item['bytes']:14d} {len(raw):11d} {str(match):>9s} "
              f"{raw.count(chr(13).encode() + chr(10).encode()):13d}")
    npz_bad = [r for r in mismatch if r.endswith(".npz")]
    text_bad = [r for r in mismatch if not r.endswith(".npz")]
    print(f"\nnpz byte-safe: {12 - len(npz_bad)}/12", f"| mismatched: {npz_bad}")
    print(f"text files byte-stable: {3 - len(text_bad)}/3", f"| mismatched: {text_bad}")

    print("\n== 证据目录 manifest（-text 规则）在 clone 中自核 ==")
    manifest_path = EVID / "manifest.txt"
    entries = [ln for ln in manifest_path.read_text(encoding="utf-8").splitlines()
               if ln and not ln.startswith("#")]
    bad = []
    for line in entries:
        h, n, rel = line.split()
        raw = (CLONE / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2" / rel).read_bytes()
        if sha(raw) != h or len(raw) != int(n):
            bad.append(rel)
    print(f"evidence manifest entries={len(entries)} mismatched={len(bad)} {bad[:5]}")

    expected_npz_ok = not npz_bad
    print("\nRESULT: " + ("PASS" if expected_npz_ok else "FAIL")
          + f" (npz 12/12 byte-safe={expected_npz_ok}; text mismatches={len(text_bad)}; evidence manifest bad={len(bad)})")
    return 0 if expected_npz_ok and len(bad) == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
