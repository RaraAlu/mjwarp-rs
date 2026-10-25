"""QA 独立复核一键复跑：上游重采 → 独立 oracle → 产品 probe（临时 worktree）→ 三方比对。

所有日志以 UTF-8 写入本证据目录；JSON 产物由各步生成。
用法：python qa_run_all.py
"""

import subprocess
import sys
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[2]
PYTHON = ROOT / "target" / "mjwarp-parallel" / "venv" / "Scripts" / "python.exe"
ORACLE = ROOT / "target" / "qa-g4" / "oracle-target" / "debug" / "qa-g4-oracle.exe"


def run(command, log_name, cwd=ROOT):
    log_path = EVIDENCE / log_name
    print(f"== {log_name}: {' '.join(str(part) for part in command)}")
    result = subprocess.run([str(part) for part in command], cwd=str(cwd), text=True,
                            capture_output=True, check=False)
    body = []
    body.append(f"$ {' '.join(str(part) for part in command)}")
    body.append(f"# exit={result.returncode} cwd={cwd}")
    body.append("--- stdout ---")
    body.append(result.stdout)
    body.append("--- stderr ---")
    body.append(result.stderr)
    log_path.write_text("\n".join(body), encoding="utf-8")
    print(f"   -> {log_name} exit={result.returncode} "
          f"stdout_lines={len(result.stdout.splitlines())}")
    return result


def main():
    failures = []

    result = run([PYTHON, EVIDENCE / "qa_capture_upstream.py"], "qa-capture.log")
    if result.returncode != 0:
        failures.append("capture")

    result = run([ORACLE, EVIDENCE / "qa-upstream-cases.json",
                  EVIDENCE / "qa-oracle-output.json"], "qa-oracle.log")
    if result.returncode != 0:
        failures.append("oracle")

    result = run([PYTHON, EVIDENCE / "qa_probe_run.py"], "qa-probe.log")
    if result.returncode != 0:
        failures.append("probe")

    result = run([PYTHON, EVIDENCE / "qa_compare.py"], "qa-compare.log")
    if result.returncode != 0:
        failures.append("compare")

    # 追加节：SDF 相对旋转约定判定（非单位旋转对）
    result = run([PYTHON, EVIDENCE / "qa_capture_sdf_rotation.py"], "qa-sdf-capture.log")
    if result.returncode != 0:
        failures.append("sdf-capture")

    result = run([PYTHON, EVIDENCE / "qa_probe_sdf_run.py"], "qa-sdf-probe.log")
    if result.returncode != 0:
        failures.append("sdf-probe")

    result = run([PYTHON, EVIDENCE / "qa_compare_sdf_rotation.py"], "qa-sdf-compare.log")
    if result.returncode != 0:
        failures.append("sdf-compare")

    print("failures:", failures if failures else "none")
    if failures:
        sys.exit(1)


if __name__ == "__main__":
    main()
