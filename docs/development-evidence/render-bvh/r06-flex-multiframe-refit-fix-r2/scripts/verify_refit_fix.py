"""R06 多帧 refit 修复 r2 —— CPU-only 可复跑自检（零 GPU、不采集、不写 fixtures）。

用法（仓库根，pinned venv python）：
    target/mjwarp-parallel/venv/Scripts/python.exe -B \
        docs/development-evidence/render-bvh/r06-flex-multiframe-refit-fix-r2/scripts/verify_refit_fix.py

检查：
  A. 静态/AST：主模块无 `--refit-bvh`、`_render_frame` 内 **无条件** refit（refit 行在 capture_once 之前、
     无 `if refit` 分支）；主模块与自查模块行数 ≤500；单帧主脚本 `capture_render_flex.py` 零 diff。
  B. refit 计次：3 帧 × 2 遍 = 6 次且顺序正确才 PASS；3 次（漏第二遍）/乱序 → FAIL。
  C. 三条硬自查（合成数据）：base 幂等 PASS/FAIL（改 1 bit）、位移 PASS/FAIL（±1e-6 边界）、
     变化余量 PASS/FAIL（<576）；`run_capture_checks` 汇总 ok/failures 正确。
  D. provenance：spec sha/bytes 与常量一致；三脚本 sha 与实际文件一致；refit_bvh=True。
  E. CLI：`--help` exit0 且无 `--refit-bvh`；传 `--refit-bvh` exit2；dry-run exit0 且
     refit_policy/refit_calls=6、不写产物；`--check-deltas` 合成三态 0/3/3。
"""

from __future__ import annotations

import ast
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[5]
EVID = pathlib.Path(__file__).resolve().parents[1]
LOG_DIR = EVID / "logs"
BASE = REPO / "fixtures/render-bvh/scene-flex/reference/base"
SPEC = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SCRIPTS = REPO / "scripts/authoring"
WORK = REPO / "target/r06-multiframe-refit-fix-dry"
RESULTS: list[dict] = []


def record(name: str, ok: bool, detail=None) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def load_checks_module():
    sys.path.insert(0, str(SCRIPTS))
    import capture_render_flex_multiframe_checks as checks

    return checks


def load_main_module():
    sys.path.insert(0, str(SCRIPTS))
    import capture_render_flex_multiframe as module

    return module


def make_fake_first(base_dir: pathlib.Path) -> dict:
    """用 base 参考构造 frame-000 的伪采集产物（位级全等 → 自查应 PASS）。"""
    import numpy as np

    def load(filename):
        with np.load(base_dir / filename) as archive:
            return {name: archive[name] for name in archive.files}

    state = load("state.npz")
    arrays = {}
    for kind in ("rgb", "depth", "seg"):
        arrays.update(load(f"{kind}.npz"))
    return {
        "state": state,
        "arrays": arrays,
        "states": make_fake_states(0.1),
        "mjm": FakeMjm(),
    }


def make_fake_states(frame_dz: float, noise: float = 0.0):
    """附着顶点（index 1..5）+k*0.1；world 顶点（index 0）不动。"""
    import numpy as np

    base = np.zeros((1, 6, 3), dtype=np.float32)
    base[0, 1:, 0] = [0.1, 0.2, 0.3, 0.4, 0.5]  # 任意基准位置
    states = []
    for k in range(3):
        pos = base.copy()
        pos[0, 1:, 2] += np.float32(k * frame_dz + (noise if k else 0.0))
        states.append({"flexvert_xpos": pos})
    return states


class FakeMjm:
    def __init__(self):
        import numpy as np

        self.flex_vertbodyid = np.asarray([0, 1, 2, 3, 1, 2], dtype=np.int32)


def _api_stub():
    """真实采集路径里 api 是 `capture_render_flex.py` 模块（提供 ROOT）；这里用最小 stub。"""
    import types

    return types.SimpleNamespace(ROOT=str(REPO))


def delta_result(depth_changed: int, seg_changed: int) -> dict:
    pairs = [
        {"pair": "000->001", "depth_changed": depth_changed, "seg_changed": seg_changed},
        {"pair": "001->002", "depth_changed": depth_changed, "seg_changed": seg_changed},
    ]
    overall = "pass-margin" if depth_changed >= 576 and seg_changed >= 576 else (
        "non-discriminating" if min(depth_changed, seg_changed) < 384 else "insufficient-margin")
    return {"report": pairs, "overall": overall, "thresholds": {"accept_min": 384, "margin_min": 576}}


def write_synth_delta(root: pathlib.Path, depth_changed: int, seg_changed: int) -> None:
    import numpy as np

    for frame in range(3):
        frame_dir = root / f"frame-{frame:03d}"
        frame_dir.mkdir(parents=True, exist_ok=True)
        depth = np.zeros((240, 320), dtype=np.float32)
        seg = np.full((240, 320, 2), -1, dtype=np.int32)
        if frame > 0:
            flat_depth = depth.reshape(-1)
            flat_depth[:depth_changed] = np.float32(frame)  # 前 N 像素与上一帧不同
            seg_flat = seg.reshape(-1, 2)
            seg_flat[:seg_changed] = np.asarray([frame, 9], dtype=np.int32)  # 帧间 seg 不同才算变化
        np.savez(frame_dir / "depth.npz", depth_cam2=depth)
        np.savez(frame_dir / "seg.npz", seg_cam2=seg)


def main() -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    checks = load_checks_module()
    main_module = load_main_module()
    main_path = SCRIPTS / "capture_render_flex_multiframe.py"
    checks_path = SCRIPTS / "capture_render_flex_multiframe_checks.py"
    main_source = main_path.read_text(encoding="utf-8")

    # ---------- A. 静态 / AST ----------
    tree = ast.parse(main_source, str(main_path))
    render_fn = next(node for node in ast.walk(tree)
                     if isinstance(node, ast.FunctionDef) and node.name == "_render_frame")
    render_source = ast.unparse(render_fn)
    refit_at = render_source.find("mjw.refit_bvh(m, d, rc)")
    capture_at = render_source.find("api.capture_once")
    record("A1 _render_frame 内无条件 refit 且在 capture_once 之前",
           refit_at >= 0 and 0 <= refit_at < capture_at,
           {"refit_at": refit_at, "capture_once_at": capture_at})
    record("A2 无 if refit 分支、_render_frame 签名无 refit 形参",
           "if refit" not in render_source and "refit" not in [arg.arg for arg in render_fn.args.args],
           {"if_refit": "if refit" in render_source, "args": [arg.arg for arg in render_fn.args.args]})
    record("A3 无 --refit-bvh（CLI 已删除）", "--refit-bvh" not in main_source, "--refit-bvh" in main_source)
    main_lines = main_path.read_bytes().count(b"\n")
    checks_lines = checks_path.read_bytes().count(b"\n")
    record("A4 主模块 ≤500 行", main_lines <= 500, main_lines)
    record("A5 自查模块 ≤500 行", checks_lines <= 500, checks_lines)
    single_diff = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "scripts/authoring/capture_render_flex.py"],
                                 cwd=REPO)
    single_lines = (SCRIPTS / "capture_render_flex.py").read_bytes().count(b"\n")
    record("A6 单帧主脚本零 diff", single_diff.returncode == 0, {"lines": single_lines})

    # ---------- B. refit 计次 ----------
    checks.reset_refit_log()
    for _ in range(2):
        for frame in range(3):
            checks.record_refit(frame)
    summary = checks.refit_summary(3, 2)
    record("B1 两遍×3帧=6 次 refit → PASS", summary["ok"] and summary["actual_calls"] == 6,
           {"calls": summary["actual_calls"], "sequence": summary["actual_sequence"]})
    checks.reset_refit_log()
    for frame in range(3):
        checks.record_refit(frame)
    record("B2 只记一遍（3 次）→ FAIL（漏 --repeat 2 第二遍）", not checks.refit_summary(3, 2)["ok"],
           checks.refit_summary(3, 2)["actual_calls"])
    checks.reset_refit_log()
    for frame in (0, 2, 1, 0, 1, 2):
        checks.record_refit(frame)
    record("B3 乱序 → FAIL", not checks.refit_summary(3, 2)["ok"], checks.refit_summary(3, 2)["actual_sequence"])

    # ---------- C. 三条硬自查 ----------
    fake_first = make_fake_first(BASE)
    base_ok = checks.check_frame000_vs_base(str(REPO), fake_first)
    record("C1 base 幂等 PASS（合成=base 原数组）", base_ok["ok"], {"arrays": base_ok["arrays_checked"]})
    import numpy as np

    broken = {name: value.copy() for name, value in fake_first["state"].items()}
    bits = broken["qpos"].view(np.uint32)
    bits.reshape(-1)[0] ^= 1  # 改 1 bit
    broken_first = {"state": broken, "arrays": fake_first["arrays"],
                    "states": fake_first["states"], "mjm": fake_first["mjm"]}
    base_bad = checks.check_frame000_vs_base(str(REPO), broken_first)
    record("C2 base 幂等 FAIL（改 1 bit）", not base_bad["ok"], base_bad["mismatches"][:2])

    mjm = FakeMjm()
    disp_ok = checks.check_displacement(make_fake_states(0.1), mjm, 0.1)
    record("C3 位移 PASS（精确 k×0.100 m）", disp_ok["ok"],
           {frame: entry["attached_mean_m"] for frame, entry in disp_ok["frames"].items()})
    disp_bad = checks.check_displacement(make_fake_states(0.1, noise=1e-3), mjm, 0.1)
    record("C4 位移 FAIL（+1e-3 超 ±1e-6）", not disp_bad["ok"],
           {frame: entry["delta_m"] for frame, entry in disp_bad["frames"].items()})
    disp_edge = checks.check_displacement(make_fake_states(0.1, noise=5e-7), mjm, 0.1)
    record("C5 位移边界（+5e-7 在 ±1e-6 容差内）PASS", disp_edge["ok"],
           {frame: entry["delta_m"] for frame, entry in disp_edge["frames"].items()})

    margin_ok = checks.check_delta_margins(delta_result(600, 600), 576)
    margin_low = checks.check_delta_margins(delta_result(500, 600), 576)
    margin_non = checks.check_delta_margins(delta_result(300, 600), 576)
    record("C6 变化余量 PASS（600/600）", margin_ok["ok"], margin_ok["overall"])
    record("C7 变化余量 FAIL（500 <576）", not margin_low["ok"], margin_low["pairs"]["000->001"])
    record("C8 变化余量 FAIL（300 non-discriminating）", not margin_non["ok"], margin_non["overall"])

    checks.reset_refit_log()
    for _ in range(2):
        for frame in range(3):
            checks.record_refit(frame)
    bundle_ok = checks.run_capture_checks(_api_stub(), fake_first, delta_result(600, 600), 3, 2, 0.1)
    record("C9 run_capture_checks 全 PASS", bundle_ok["ok"] and not bundle_ok["failures"],
           {"failures": bundle_ok["failures"]})
    bundle_bad = checks.run_capture_checks(_api_stub(), broken_first, delta_result(600, 600), 3, 2, 0.1)
    record("C10 run_capture_checks 检出失败项", (not bundle_bad["ok"]) and "frame000_vs_base" in bundle_bad["failures"],
           bundle_bad["failures"])

    # ---------- D. provenance ----------
    provenance = checks.provenance_block(_api_stub())
    import hashlib

    spec_bytes = SPEC.read_bytes()
    ok_spec = (provenance["spec_sha256"] == "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
               and provenance["spec_bytes"] == 10506 == len(spec_bytes)
               and hashlib.sha256(spec_bytes).hexdigest() == provenance["spec_sha256"])
    record("D1 provenance spec sha/bytes 一致", ok_spec, {"sha": provenance["spec_sha256"][:16], "bytes": provenance["spec_bytes"]})
    sha_ok = all(
        provenance["authoring_script_sha256"][name] == hashlib.sha256((SCRIPTS / name).read_bytes()).hexdigest()
        for name in provenance["authoring_script_sha256"]
    )
    record("D2 provenance 三脚本 sha 与实际一致", sha_ok and provenance["refit_bvh"] is True,
           {"scripts": list(provenance["authoring_script_sha256"]), "refit_bvh": provenance["refit_bvh"]})

    # ---------- E. CLI ----------
    py = sys.executable
    capture = str(SCRIPTS / "capture_render_flex.py")

    def run(args: list[str]) -> subprocess.CompletedProcess:
        return subprocess.run([py, "-B", capture, *args], cwd=REPO, capture_output=True)

    help_proc = run(["--help"])
    help_text = help_proc.stdout.decode("utf-8", "replace") + help_proc.stderr.decode("utf-8", "replace")
    record("E1 --help exit0 且无 --refit-bvh", help_proc.returncode == 0 and "--refit-bvh" not in help_text,
           {"exit": help_proc.returncode, "has_switch": "--refit-bvh" in help_text})

    flag_proc = run(["--frames", "3", "--frame-dz", "0.1", "--out-dir", str(WORK / "never"), "--refit-bvh", "--dry-run"])
    record("E2 传 --refit-bvh 被拒（argparse exit2）", flag_proc.returncode == 2, flag_proc.returncode)

    dry_dir = WORK / "dry-run"
    if dry_dir.exists():
        import shutil

        shutil.rmtree(dry_dir)
    dry_proc = run(["--frames", "3", "--frame-dz", "0.1", "--out-dir", str(dry_dir), "--dry-run"])
    dry_stdout = dry_proc.stdout.decode("utf-8", "replace")
    plan = json.loads(dry_stdout[dry_stdout.find("{"):]) if dry_proc.returncode == 0 else {}
    (LOG_DIR / "dry-run.log").write_text(
        f"# command: {py} -B {capture} --frames 3 --frame-dz 0.1 --out-dir {dry_dir} --dry-run\n"
        f"# exit_code: {dry_proc.returncode}\n" + dry_stdout.replace("\r\n", "\n"),
        encoding="utf-8", newline="\n")
    record("E3 dry-run exit0 且不写产物", dry_proc.returncode == 0 and not dry_dir.exists(),
           {"exit": dry_proc.returncode, "out_dir_created": dry_dir.exists()})
    record("E4 dry-run 计划 refit_policy/refit_calls=6",
           plan.get("refit_policy") == "every-frame" and plan.get("refit_calls") == 6,
           {"refit_policy": plan.get("refit_policy"), "refit_calls": plan.get("refit_calls")})

    synth = {"pass": (600, 600, 0), "insufficient": (500, 600, 3), "non-discriminating": (300, 600, 3)}
    synth_results = {}
    for name, (depth_changed, seg_changed, expected_exit) in synth.items():
        root = WORK / f"synth-{name}"
        if root.exists():
            import shutil

            shutil.rmtree(root)
        write_synth_delta(root, depth_changed, seg_changed)
        proc = run(["--check-deltas", "--out-dir", str(root)])
        synth_results[name] = {"exit": proc.returncode, "expected": expected_exit}
        (LOG_DIR / f"check-deltas-synth-{name}.log").write_text(
            f"# command: {py} -B {capture} --check-deltas --out-dir {root}\n# exit_code: {proc.returncode}\n"
            + proc.stdout.decode("utf-8", "replace").replace("\r\n", "\n"),
            encoding="utf-8", newline="\n")
        record(f"E5 --check-deltas {name} exit={expected_exit}", proc.returncode == expected_exit,
               {"exit": proc.returncode, "expected": expected_exit})

    # ---------- 汇总 ----------
    failed = [item["name"] for item in RESULTS if not item["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "fails": failed,
               "main_module_lines": main_lines, "checks_module_lines": checks_lines, "checks": RESULTS}
    (LOG_DIR / "verify-refit-fix.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    (LOG_DIR / "line-counts.txt").write_text(
        "# git blob 换行数口径\n"
        f"scripts/authoring/capture_render_flex.py\t{single_lines}\n"
        f"scripts/authoring/capture_render_flex_multiframe.py\t{main_lines}\n"
        f"scripts/authoring/capture_render_flex_multiframe_checks.py\t{checks_lines}\n",
        encoding="utf-8", newline="\n")
    print(f"RESULT: {payload['result']} ({len(RESULTS) - len(failed)}/{len(RESULTS)} pass, {len(failed)} fail)")
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
