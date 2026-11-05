"""R06 多帧采集窗 G-2 执行/证据 wrapper（窗口内使用；只写 target/ 与证据目录）。

用法（仓库根，pinned venv python）：
  python -B target/r06-multiframe-capture-g2/run_g2.py dry-run
  python -B target/r06-multiframe-capture-g2/run_g2.py capture
  python -B target/r06-multiframe-capture-g2/run_g2.py post-gpu
  python -B target/r06-multiframe-capture-g2/run_g2.py files-sha

- dry-run：CPU-only，记录 command/exit；断言 out_dir 未被创建。
- capture：CUDA_PATH 前置；`--frames 3 --frame-dz 0.1 --repeat 2 --out-dir fixtures/render-bvh/scene-flex/multiframe`；
  记录 command/exit；exit 非 0 时**立即删除 out_dir 残留**并留痕（§12）。
- post-gpu：记录窗口后 GPU/构建进程/树/锁状态。
- files-sha：为落库产物与证据文件生成 sha256/bytes 清单（JSON），供 manifest 使用。
"""

from __future__ import annotations

import datetime
import json
import os
import pathlib
import shutil
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/r06-multiframe-capture-g2"
LOG_DIR = WORK / "logs"
EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2"
CAPTURE = REPO / "scripts/authoring/capture_render_flex.py"
CUDA_PATH = REPO / "target/toolchains/cuda-12.8.1"
OUT_DIR = REPO / "fixtures/render-bvh/scene-flex/multiframe"


def decode(data: bytes) -> str:
    for encoding in ("utf-8", "gbk"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            continue
    return data.decode("utf-8", "replace")


def write_text(path: pathlib.Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def git_status() -> str:
    return subprocess.run(["git", "status", "--porcelain"], cwd=REPO, capture_output=True, text=True).stdout.strip()


def run_mode(mode: str) -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    now = datetime.datetime.now().astimezone()
    if mode == "dry-run":
        command = [sys.executable, "-B", str(CAPTURE), "--frames", "3", "--frame-dz", "0.1",
                   "--out-dir", OUT_DIR.as_posix(), "--dry-run"]
    elif mode == "capture":
        command = [sys.executable, "-B", str(CAPTURE), "--frames", "3", "--frame-dz", "0.1",
                   "--out-dir", OUT_DIR.as_posix(), "--repeat", "2"]
    else:
        raise SystemExit(f"unknown mode {mode}")

    out_before = OUT_DIR.exists()
    env = dict(os.environ)
    env["CUDA_PATH"] = str(CUDA_PATH)
    env["MJWARP_CAPTURE_AT"] = now.isoformat(timespec="seconds")
    proc = subprocess.run(command, cwd=REPO, capture_output=True, env=env)
    stdout = decode(proc.stdout).replace("\r\n", "\n")
    stderr = decode(proc.stderr).replace("\r\n", "\n")
    out_after = OUT_DIR.exists()

    header = (
        f"# R06 G-2 {mode}\n"
        f"# command: {' '.join(command)}\n"
        f"# cwd: {REPO.as_posix()}\n"
        f"# CUDA_PATH: {CUDA_PATH.as_posix()}\n"
        f"# started: {now.isoformat(timespec='seconds')}\n"
        f"# out_dir: {OUT_DIR.as_posix()}\n"
        f"# out_dir_existed_before: {out_before}\n"
        f"# out_dir_exists_after: {out_after}\n"
        f"# exit_code: {proc.returncode}\n"
        "# ---- stdout ----\n"
    )
    write_text(LOG_DIR / f"{mode}.log", header + stdout + f"\n# ---- end (exit_code={proc.returncode}) ----\n")
    write_text(LOG_DIR / f"{mode}.stderr.log", stderr)

    if mode == "capture" and proc.returncode != 0:
        # §12：判定失败即删 out_dir 残留并留痕
        residue_before = sorted(p.as_posix() for p in OUT_DIR.rglob("*")) if OUT_DIR.exists() else []
        shutil.rmtree(OUT_DIR, ignore_errors=True)
        status = git_status()
        write_text(
            WORK / "exit3-residue-cleanup.txt",
            "# exit3 残留清理（G-2 §12）\n"
            f"# exit_code: {proc.returncode}\n"
            f"# residue_files_before_cleanup: {len(residue_before)}\n"
            + "\n".join(f"# - {p}" for p in residue_before)
            + f"\n# out_dir_after_cleanup_exists: {OUT_DIR.exists()}\n"
            f"# git_status_porcelain (fixtures 行):\n"
            + "\n".join(f"# {line}" for line in status.splitlines() if "fixtures" in line) + "\n"
            + "# git_status_porcelain (全量):\n" + "\n".join(f"# {line}" for line in status.splitlines()) + "\n",
        )
    print(f"mode={mode} exit_code={proc.returncode} out_dir_exists={OUT_DIR.exists()}")
    print(f"log={(LOG_DIR / f'{mode}.log').as_posix()}")
    return proc.returncode


def post_gpu() -> int:
    WORK.mkdir(parents=True, exist_ok=True)
    gpu = subprocess.run(["nvidia-smi", "--query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu",
                          "--format=csv,noheader"], capture_output=True, text=True).stdout.strip()
    build = subprocess.run(
        ["powershell", "-NoProfile", "-Command",
         "@(Get-Process cargo,rustc,cl,nvcc,link -ErrorAction SilentlyContinue).Count"],
        capture_output=True, text=True).stdout.strip()
    lock = subprocess.run(
        ["powershell", "-NoProfile", "-Command", "& scripts/validate/repo-write-lock.ps1 -Action status"],
        cwd=REPO, capture_output=True, text=True).stdout.strip()
    lines = [
        "# G-2 窗口收尾 GPU / 构建 / 树 / 锁状态",
        f"# ts: {datetime.datetime.now().astimezone().isoformat(timespec='seconds')}",
        f"gpu: {gpu}",
        f"build_processes: {build}",
        f"tree_dirty_lines: {len(git_status().splitlines())}",
        f"lock: {lock.splitlines()[0] if lock else '(unknown)'}",
    ]
    write_text(WORK / "post-window-gpu.txt", "\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


def files_sha() -> int:
    import hashlib

    entries = []
    if OUT_DIR.exists():
        for path in sorted(OUT_DIR.rglob("*")):
            if path.is_file():
                data = path.read_bytes()
                entries.append({"path": path.relative_to(REPO).as_posix(), "bytes": len(data),
                                "sha256": hashlib.sha256(data).hexdigest()})
    write_text(WORK / "files-sha256.json", json.dumps({"out_dir": OUT_DIR.as_posix(), "files": entries},
                                                      indent=2, ensure_ascii=False) + "\n")
    print(f"files_sha entries={len(entries)}")
    return 0


def main() -> int:
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode in ("dry-run", "capture"):
        return run_mode(mode)
    if mode == "post-gpu":
        return post_gpu()
    if mode == "files-sha":
        return files_sha()
    raise SystemExit("usage: run_g2.py dry-run|capture|post-gpu|files-sha")


if __name__ == "__main__":
    raise SystemExit(main())
