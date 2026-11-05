"""R06 多帧诊断：带命令行/退出码 wrapper 的采集执行器（诊断窗用；仅写 target/）。

用法（仓库根，pinned venv python）：
  python -B target/r06-multiframe-diag/run_capture.py --label norefit --out-dir target/r06-multiframe-diag/norefit
  python -B target/r06-multiframe-diag/run_capture.py --label refit   --out-dir target/r06-multiframe-diag/refit --refit-bvh

日志（UTF-8/LF）：target/r06-multiframe-diag/logs/<label>-capture.log
  - 首行：完整命令行 + 起始时刻 + CUDA_PATH
  - 主体：脚本 stdout（含 delta 自检摘要 JSON）
  - 末行：exit_code=<n>
stderr 另存 <label>-capture.stderr.log（UTF-8/LF）。
"""

from __future__ import annotations

import argparse
import datetime
import os
import pathlib
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/r06-multiframe-diag"
LOG_DIR = WORK / "logs"
CAPTURE = REPO / "scripts/authoring/capture_render_flex.py"
CUDA_PATH = REPO / "target/toolchains/cuda-12.8.1"


def decode(data: bytes) -> str:
    for encoding in ("utf-8", "gbk"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            continue
    return data.decode("utf-8", "replace")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", required=True)
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--refit-bvh", action="store_true")
    args = parser.parse_args()

    WORK.mkdir(parents=True, exist_ok=True)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    out_dir = (REPO / args.out_dir).resolve()
    command = [
        sys.executable, "-B", str(CAPTURE),
        "--frames", "3", "--frame-dz", "0.1",
        "--out-dir", out_dir.as_posix(), "--repeat", "2",
    ]
    if args.refit_bvh:
        command.append("--refit-bvh")
    env = dict(os.environ)
    env["CUDA_PATH"] = str(CUDA_PATH)
    env["MJWARP_CAPTURE_AT"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    started = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    proc = subprocess.run(command, cwd=REPO, capture_output=True, env=env)
    stdout = decode(proc.stdout).replace("\r\n", "\n")
    stderr = decode(proc.stderr).replace("\r\n", "\n")
    header = (
        f"# R06 多帧诊断采集（label={args.label}）\n"
        f"# command: {' '.join(command)}\n"
        f"# cwd: {REPO.as_posix()}\n"
        f"# CUDA_PATH: {CUDA_PATH.as_posix()}\n"
        f"# started: {started}\n"
        f"# out_dir: {out_dir.as_posix()}\n"
        f"# exit_code: {proc.returncode}\n"
        "# ---- script stdout ----\n"
    )
    (LOG_DIR / f"{args.label}-capture.log").write_text(
        header + stdout + f"\n# ---- end (exit_code={proc.returncode}) ----\n",
        encoding="utf-8", newline="\n",
    )
    (LOG_DIR / f"{args.label}-capture.stderr.log").write_text(stderr, encoding="utf-8", newline="\n")
    print(f"label={args.label} exit_code={proc.returncode} out_dir={out_dir.as_posix()}")
    print(f"log={(LOG_DIR / f'{args.label}-capture.log').as_posix()}")
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
