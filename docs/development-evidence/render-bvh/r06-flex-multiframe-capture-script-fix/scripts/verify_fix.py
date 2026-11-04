"""R06 多帧采集脚本 O3 小修 —— CPU-only 可复跑验证（零 GPU、不采集、不写仓）。

用法（仓库根目录）：
    <pinned-python> -B docs/development-evidence/render-bvh/r06-flex-multiframe-capture-script-fix/scripts/verify_fix.py

检查项：
  A. O3 动态：`delta_exit_code()` 三态映射（pass-margin=0 / insufficient-margin=3 /
     non-discriminating=3），`_print_delta_report()` 非 pass 时输出「不得落库」警告、pass 时不输出。
  B. O3 接线（AST 静态）：`capture_multiframe()` 必须调用 `_delta_report()` 且末尾返回
     `delta_exit_code(...)`；`capture.json` 字典必须含 `delta_selfcheck` 键；不得出现无条件的
     `return 0` 收尾。
  C. O2：`--frames 0` / `--frames -1` / `--frames 2` 均 exit 1 且 stdout/stderr 含拒绝原因；
     `--frames 3 --dry-run` exit 0。
  D. 回归：`--check-deltas` 三组合成数据 0/3/3；dry-run stdout 为纯 JSON 可 `json.load`。
  E. 行数棘轮：`capture_render_flex.py` 保持 498 行（git blob 换行数，未改）+ 扩展模块 ≤500。
  F. 冻结 spec：sha256 / bytes 与脚本常量一致（未变）。
  G. 单帧路径：`capture_render_flex.py` 与 HEAD 逐字节一致（`git diff --quiet`）。
"""

from __future__ import annotations

import ast
import contextlib
import hashlib
import importlib.util
import io
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[5]
MODULE_PATH = REPO / "scripts/authoring/capture_render_flex_multiframe.py"
MAIN_PATH = REPO / "scripts/authoring/capture_render_flex.py"
SPEC_PATH = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
LOG_DIR = pathlib.Path(__file__).resolve().parents[1] / "logs"

RESULTS: list[dict] = []


def decode_output(data: bytes) -> str:
    """子进程输出解码：先按 UTF-8 严格解，失败退回本地 ANSI（cp936），最后替换。"""
    for encoding in ("utf-8", "gbk"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            continue
    return data.decode("utf-8", "replace")


def normalize_text(text: str) -> str:
    """证据目录要求 LF：子进程在 Windows 管道会把 \\n 翻成 \\r\\n，落盘前归一化。"""
    return text.replace("\r\n", "\n")


def record(name: str, ok: bool, detail) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def git(*argv: str) -> bytes:
    return subprocess.run(["git", *argv], cwd=REPO, check=True, capture_output=True).stdout


def blob_lines(rel_path: str) -> int:
    return git("show", f"HEAD:{rel_path}").count(b"\n")


def work_lines(rel_path: str) -> int:
    return (REPO / rel_path).read_bytes().count(b"\n")


def load_module():
    spec = importlib.util.spec_from_file_location("capture_mf_fix", MODULE_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def check_o3_mapping(mf) -> None:
    cases = {"pass-margin": 0, "insufficient-margin": 3, "non-discriminating": 3}
    got = {verdict: mf.delta_exit_code({"overall": verdict}) for verdict in cases}
    record("A1 delta_exit_code 三态映射", got == cases, got)

    def emitted(overall: str, depth: int, seg: int) -> str:
        buffer = io.StringIO()
        result = {
            "thresholds": {"accept_min": mf.DELTA_ACCEPT_MIN, "margin_min": mf.DELTA_MARGIN_MIN},
            "cam2_pixels": mf.CAM2_PIXELS,
            "report": [{"pair": "000->001", "depth_changed": depth, "seg_changed": seg,
                        "verdict": mf._verdict(depth, seg)}],
            "overall": overall,
        }
        with contextlib.redirect_stdout(buffer):
            mf._print_delta_report(result)
        return buffer.getvalue()

    pass_text = emitted("pass-margin", 600, 600)
    warn_text = emitted("insufficient-margin", 500, 600)
    non_text = emitted("non-discriminating", 300, 600)
    record("A2 pass-margin 不打印警告", "不得落库" not in pass_text, pass_text.strip().splitlines()[-1])
    record("A3 insufficient-margin 打印警告", "不得落库" in warn_text, warn_text.strip().splitlines()[-1])
    record("A4 non-discriminating 打印警告", "不得落库" in non_text, non_text.strip().splitlines()[-1])

    threshold_cases = [
        ((600, 650), "pass-margin"),
        ((576, 576), "pass-margin"),
        ((575, 600), "insufficient-margin"),
        ((600, 575), "insufficient-margin"),
        ((384, 600), "insufficient-margin"),
        ((383, 600), "non-discriminating"),
        ((600, 383), "non-discriminating"),
    ]
    got = {(d, s): mf._verdict(d, s) for d, s in (c for c, _ in threshold_cases)}
    expected = {c: v for c, v in threshold_cases}
    record("A5 384/576 阈值边界映射", got == expected, {f"{d},{s}": v for (d, s), v in got.items()})


def check_o3_wiring() -> None:
    source = MODULE_PATH.read_text(encoding="utf-8")
    tree = ast.parse(source, str(MODULE_PATH))
    capture_fn = next(
        node for node in ast.walk(tree) if isinstance(node, ast.FunctionDef) and node.name == "capture_multiframe"
    )
    calls = {node.func.id for node in ast.walk(capture_fn)
             if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)}
    returns = [node for node in ast.walk(capture_fn) if isinstance(node, ast.Return)]
    last_return = returns[-1] if returns else None
    return_src = ast.unparse(last_return.value) if last_return and last_return.value else "<none>"
    has_delta_selfcheck = "delta_selfcheck" in {node.value for node in ast.walk(capture_fn)
                                                if isinstance(node, ast.Constant) and isinstance(node.value, str)}
    record("B1 capture_multiframe 调用 _delta_report", "_delta_report" in calls, sorted(calls & {"_delta_report", "delta_exit_code", "_print_delta_report"}))
    record("B2 capture.json 含 delta_selfcheck", has_delta_selfcheck, has_delta_selfcheck)
    record("B3 末尾返回 delta_exit_code(...)", "delta_exit_code" in return_src, return_src)
    plain_return_0 = [node for node in returns if isinstance(node.value, ast.Constant) and node.value.value == 0]
    record("B4 无无条件 return 0 收尾", not plain_return_0, len(plain_return_0))
    record("B5 采集路径打印自检", "_print_delta_report" in calls, "_print_delta_report" in calls)

    before = git("show", "HEAD:scripts/authoring/capture_render_flex_multiframe.py").decode("utf-8")
    before_lines = [f"{i + 1:4d}: {line}" for i, line in enumerate(before.splitlines())
                    if "_delta_report(out_dir, api, stream=False)" in line or line.strip() == "return 0"]
    after_lines = [f"{i + 1:4d}: {line}" for i, line in enumerate(source.splitlines())
                   if "delta_selfcheck" in line or "delta_exit_code" in line or "_print_delta_report(delta_lines)" in line]
    (LOG_DIR / "o3-before-after.txt").write_text(
        "# O3 修复前（HEAD blob）\n" + "\n".join(before_lines)
        + "\n\n# O3 修复后（工作树）\n" + "\n".join(after_lines) + "\n",
        encoding="utf-8", newline="\n")
    record("B6 前后行为对照已落 logs/o3-before-after.txt", True, f"before {len(before_lines)} 行 / after {len(after_lines)} 行")


def check_o2_and_regression() -> None:
    py = sys.executable
    main = str(MAIN_PATH)
    base = [py, "-B", main]
    cases = {
        "o2-frames-0": ["--frames", "0", "--out-dir", "target/multiframe-work/never", "--dry-run"],
        "o2-frames-minus-1": ["--frames", "-1", "--out-dir", "target/multiframe-work/never", "--dry-run"],
        "o2-frames-2": ["--frames", "2", "--out-dir", "target/multiframe-work/never", "--dry-run"],
        "dry-run-multiframe": ["--frames", "3", "--frame-dz", "0.1",
                               "--out-dir", "target/multiframe-work/fix-dry-run", "--dry-run"],
    }
    recorded = {}
    for name, argv in cases.items():
        proc = subprocess.run(base + argv, cwd=REPO, capture_output=True)
        stdout = decode_output(proc.stdout)
        stderr = decode_output(proc.stderr)
        recorded[name] = {"argv": argv, "exit": proc.returncode,
                          "stdout_head": stdout.splitlines()[:2], "stderr_head": stderr.splitlines()[:2]}
        (LOG_DIR / f"{name}.stdout.txt").write_text(normalize_text(stdout), encoding="utf-8", newline="\n")
        (LOG_DIR / f"{name}.stderr.txt").write_text(normalize_text(stderr), encoding="utf-8", newline="\n")
        if name.startswith("o2-"):
            record(f"C {name} exit=1 且有原因", proc.returncode == 1 and "只接受 1" in (stdout + stderr),
                   {"exit": proc.returncode, "reason": (stdout + stderr).strip().splitlines()[-1][:120]})
        else:
            parsed = json.loads(stdout)
            record("D1 dry-run exit0 且 stdout 纯 JSON", proc.returncode == 0 and parsed["frames"] == 3,
                   {"exit": proc.returncode, "keys": len(parsed)})
    (LOG_DIR / "o2-dryrun-cases.json").write_text(
        json.dumps(recorded, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")

    synth = {"check-deltas-synth-pass": 0, "check-deltas-synth-insufficient": 3,
             "check-deltas-synth-non-discriminating": 3}
    got = {}
    for name, expected in synth.items():
        proc = subprocess.run(base + ["--check-deltas", "--out-dir", f"target/multiframe-work/{name.removeprefix('check-deltas-')}"],
                              cwd=REPO, capture_output=True)
        got[name] = proc.returncode
        record(f"D2 {name} exit={expected}", proc.returncode == expected, proc.returncode)
    (LOG_DIR / "delta-regression.json").write_text(
        json.dumps(got, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def check_lines_and_spec() -> None:
    main_lines = blob_lines("scripts/authoring/capture_render_flex.py")
    module_lines = work_lines("scripts/authoring/capture_render_flex_multiframe.py")
    main_diff = subprocess.run(["git", "diff", "--quiet", "HEAD", "--",
                                "scripts/authoring/capture_render_flex.py"], cwd=REPO)
    record("E1 主脚本 498 行（git blob）且未改", main_lines == 498 and main_diff.returncode == 0,
           {"lines": main_lines, "unchanged": main_diff.returncode == 0})
    record("E2 扩展模块 ≤500 行", module_lines <= 500, module_lines)
    (LOG_DIR / "line-counts.txt").write_text(
        f"# git blob 换行数口径（不是 Get-Content / Measure-Object -Line）\n"
        f"scripts/authoring/capture_render_flex.py\t{main_lines}\n"
        f"scripts/authoring/capture_render_flex_multiframe.py\t{module_lines}\n",
        encoding="utf-8", newline="\n")

    mf = load_module()
    data = SPEC_PATH.read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    record("F1 冻结 spec sha/bytes 未变",
           sha == mf.SPEC_SHA256 and len(data) == mf.SPEC_BYTES,
           {"sha256": sha, "bytes": len(data)})
    main_sha = hashlib.sha256(MAIN_PATH.read_bytes()).hexdigest()
    module_sha = hashlib.sha256(MODULE_PATH.read_bytes()).hexdigest()
    (LOG_DIR / "script-identity-after.json").write_text(
        json.dumps({
            "capture_render_flex.py": {"lines": main_lines, "sha256_before": hashlib.sha256(
                git("show", "HEAD:scripts/authoring/capture_render_flex.py")).hexdigest(),
                "sha256_after": main_sha, "changed": False},
            "capture_render_flex_multiframe.py": {"lines_before": blob_lines(
                "scripts/authoring/capture_render_flex_multiframe.py"), "lines_after": module_lines,
                "sha256_before": hashlib.sha256(git("show", "HEAD:scripts/authoring/capture_render_flex_multiframe.py")).hexdigest(),
                "sha256_after": module_sha},
        }, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    record("G1 单帧路径主脚本逐字节未变（sha 前后同）", main_sha == hashlib.sha256(
        git("show", "HEAD:scripts/authoring/capture_render_flex.py")).hexdigest(), main_sha[:16] + "…")


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="backslashreplace")
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    mf = load_module()
    check_o3_mapping(mf)
    check_o3_wiring()
    check_o2_and_regression()
    check_lines_and_spec()
    failed = [item["name"] for item in RESULTS if not item["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "fails": failed, "checks": RESULTS}
    (LOG_DIR / "verify-fix.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"RESULT: {payload['result']} ({len(RESULTS) - len(failed)}/{len(RESULTS)} pass, {len(failed)} fail)")
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
