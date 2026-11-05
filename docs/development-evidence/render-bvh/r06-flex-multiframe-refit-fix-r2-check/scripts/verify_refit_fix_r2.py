"""R06 多帧 refit 修复 r2 —— QA 独立复核脚本（只读仓库；零 GPU、不采集、不写 fixtures）。

覆盖 8 项复核口径（与执行方自检脚本相互独立实现，不复用其判定代码路径）：
  A. 调用点唯一无分支：AST 确认 `_render_frame` 内恰好一次 `mjw.refit_bvh`、位于 `mjw.forward`
     与 `api.capture_once` 之间、不在任何 If/Try/For/While 分支下；无 `if refit` / 无 refit 形参；
     无 `--refit-bvh`；`record_refit` 紧随 refit；`_capture_pass` 恰好调用 2 次（`--repeat 2`
     两遍都走同一帧循环）；`reset_refit_log` 在首遍前仅一次。
  B. 计次断言：frames×passes=6、顺序 [0,1,2,0,1,2]；反例（只记一遍 / 乱序 / 多记）必须走
     `refit_summary` 判定路径并 FAIL。
  C. 三条硬自查（真实 base 数据 + 合成位移）：base 幂等（15 数组）、位移 = k×0.100 m（真实模型
     flex_vertbodyid + 采集侧 state flexvert_xpos）、两段变化 ≥576；失败汇总进
     `run_capture_checks.failures`（→ 采集路径 exit 3）。
  D. provenance：spec 双 sha（`1d9fa5ce…`/10,506 B）+ 三脚本 sha 与实际文件一致 + refit 标记。
  E. CLI：`--refit-bvh` exit2；`--frames 0/4`、`--frame-dz 0.2`、`--repeat 1` exit1；
     dry-run exit0 且 refit_policy/refit_calls=6、不写产物；`--check-deltas` 合成三态 0/3/3。
  F. 边界/文档：冻结面（spec、帧数、Δz、阈值、语义）零改动；`77aeb6da..HEAD` 无
     src/tests/fixtures/Cargo 改动；无 multiframe fixtures / npz；行数 498/427/296；
     上游引用路径与行号内容正确（`mujoco_warp/record.py:110-111`、`contrib/render.py:452-453`）；
     证据 manifest 11/11、四门禁记录、工作树 clean、forbid_claim 0 命中。

用法（仓库根，pinned venv）：
  target/mjwarp-parallel/venv/Scripts/python.exe -B target/qa-r2-check/verify_refit_fix_r2.py
"""

from __future__ import annotations

import ast
import hashlib
import inspect
import json
import pathlib
import subprocess
import sys

REPO = next(p for p in pathlib.Path(__file__).resolve().parents
            if (p / "Cargo.toml").is_file() and (p / "scripts/authoring").is_dir())
SCRIPTS = REPO / "scripts/authoring"
EVID = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-refit-fix-r2"
BASE = REPO / "fixtures/render-bvh/scene-flex/reference/base"
SCENE = REPO / "fixtures/render-bvh/scene-flex/scene.xml"
SPEC = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
UPSTREAM = REPO / "target/mjwarp-parallel/upstream"
WORK = REPO / "target/qa-r2-check/work"
SPECDIR = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/"
RUNS = REPO / ".normify/runs/2026-11-05-render-r06-flex-multiframe-refit-fix-r2"
FIX_RANGE = "77aeb6da..HEAD"
SPEC_SHA = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
RESULTS: list[dict] = []
sys.dont_write_bytecode = True  # 不在 scripts/authoring 生成 __pycache__（保持工作树 clean）

for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding="utf-8", errors="replace", newline="\n")
    except Exception:  # pragma: no cover - 旧解释器/非 tty
        pass


def decode_child(data: bytes) -> str:
    """子进程输出按 UTF-8 解，失败回退 GBK（Windows 控制台 ANSI），不产生 U+FFFD。"""
    if not data:
        return ""
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("gbk", "replace")


def record(name: str, ok: bool, detail=None) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                          encoding="utf-8", errors="replace")


def blob(rev: str, rel: str) -> bytes:
    return subprocess.run(["git", "-C", str(REPO), "show", f"{rev}:{rel}"], capture_output=True).stdout


def call_names(node: ast.AST) -> list[tuple[ast.Call, str]]:
    out = []
    for n in ast.walk(node):
        if isinstance(n, ast.Call):
            try:
                out.append((n, ast.unparse(n.func)))
            except Exception:  # pragma: no cover - defensive
                continue
    return out


def control_ancestors(fn: ast.FunctionDef, target: ast.AST) -> list[str]:
    """返回 target 的祖先控制节点类型（If/Try/For/While/With），即 target 是否被条件/循环守卫。"""
    parents: dict[int, list[ast.AST]] = {}

    def walk(node: ast.AST, stack: list[ast.AST]) -> None:
        parents[id(node)] = stack
        for child in ast.iter_child_nodes(node):
            walk(child, stack + [node])

    walk(fn, [])
    guards = []
    for parent in parents[id(target)]:
        if isinstance(parent, (ast.If, ast.Try, ast.For, ast.While, ast.With)):
            guards.append(f"{type(parent).__name__}@{parent.lineno}")
    return guards


def main() -> int:
    sys.path.insert(0, str(SCRIPTS))
    import capture_render_flex_multiframe_checks as checks

    main_path = SCRIPTS / "capture_render_flex_multiframe.py"
    checks_path = SCRIPTS / "capture_render_flex_multiframe_checks.py"
    single_path = SCRIPTS / "capture_render_flex.py"
    main_source = main_path.read_text(encoding="utf-8")
    checks_source = checks_path.read_text(encoding="utf-8")
    single_source = single_path.read_text(encoding="utf-8")

    # ---------- A. 调用点唯一无分支 ----------
    tree = ast.parse(main_source, str(main_path))
    fn = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef) and n.name == "_render_frame")
    render_calls = [(n, name) for n, name in call_names(fn) if name == "mjw.refit_bvh"]
    capture_calls = [(n, name) for n, name in call_names(fn) if name == "api.capture_once"]
    forward_calls = [(n, name) for n, name in call_names(fn) if name in ("mjw.forward", "mj_forward")]
    record("A1 _render_frame 内恰好 1 次 mjw.refit_bvh、1 次 api.capture_once",
           len(render_calls) == 1 and len(capture_calls) == 1,
           {"refit": len(render_calls), "capture_once": len(capture_calls)})
    refit_line = render_calls[0][0].lineno if render_calls else -1
    forward_lines = [n.lineno for n, _ in forward_calls]
    capture_line = capture_calls[0][0].lineno if capture_calls else -1
    record("A2 refit 位于 forward 之后、capture_once（渲染）之前",
           bool(forward_lines) and max(forward_lines) < refit_line < capture_line,
           {"forward": forward_lines, "refit": refit_line, "capture_once": capture_line})
    guards = control_ancestors(fn, render_calls[0][0]) if render_calls else ["missing"]
    record("A3 refit 无 If/Try/For/While/With 守卫（无条件执行）", not guards, {"guards": guards})
    record("A4 无 `if refit` 分支、_render_frame 签名无 refit 形参（无开关形参）",
           "if refit" not in main_source and "refit" not in [a.arg for a in fn.args.args],
           {"args": [a.arg for a in fn.args.args]})
    record("A5 无 --refit-bvh（CLI 已删除，未知参数 argparse exit2）",
           "--refit-bvh" not in main_source and "--refit-bvh" not in single_source)
    rec_calls = [(n, name) for n, name in call_names(fn) if name == "record_refit"]
    record("A6 record_refit 恰好 1 次且紧随 refit（行序）",
           len(rec_calls) == 1 and refit_line < rec_calls[0][0].lineno,
           {"record_line": [n.lineno for n, _ in rec_calls]})
    def refit_calls_in(source: str) -> int:
        t = ast.parse(source)
        return sum(1 for n in ast.walk(t) if isinstance(n, ast.Call)
                   and ast.unparse(n.func) == "mjw.refit_bvh")

    main_refit_calls = refit_calls_in(main_source)
    checks_refit_calls = refit_calls_in(checks_source)
    record("A7 模块源码仅 1 处 refit_bvh 调用；自查模块 0 处（拆出的模块不重复 put/refit）",
           main_refit_calls == 1 and checks_refit_calls == 0,
           {"main": main_refit_calls, "checks": checks_refit_calls})
    capture_fn = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef) and n.name == "capture_multiframe")
    pass_calls = [(n, name) for n, name in call_names(capture_fn) if name == "_capture_pass"]
    reset_calls = [(n, name) for n, name in call_names(capture_fn) if name == "reset_refit_log"]
    record("A8 两遍采集都走同一帧循环（_capture_pass 调用 2 次）+ reset_refit_log 首遍前 1 次",
           len(pass_calls) == 2 and len(reset_calls) == 1
           and reset_calls[0][0].lineno < min(n.lineno for n, _ in pass_calls),
           {"capture_pass": len(pass_calls), "reset": len(reset_calls)})
    capture_body = ast.unparse(capture_fn)
    uc_calls = [(n, name) for n, name in call_names(capture_fn) if name == "run_capture_checks"]
    record("A9 采集路径调用 run_capture_checks 且以 checks[ok] 决定 exit 0/3（失败打印不得落库）",
           len(uc_calls) == 1 and ("checks['ok']" in capture_body or 'checks["ok"]' in capture_body)
           and "return 3" in capture_body and "不得落库" in capture_body,
           {"calls": len(uc_calls), "return3": "return 3" in capture_body})
    record("A10 单帧主脚本挂载点唯一（dispatch_multiframe(...) 1 次、add_frame_args(...) 1 次）",
           single_source.count("dispatch_multiframe(") == 1 and single_source.count("add_frame_args(parser)") == 1,
           {"dispatch": single_source.count("dispatch_multiframe("),
            "add_frame": single_source.count("add_frame_args(parser)")})

    # ---------- B. 计次断言（含反例） ----------
    def seq(passes: int, frames=3, order=None) -> dict:
        """记录 passes 遍 refit，但按采集路径的硬约束断言期望 = frames × 2（args.repeat==2）。"""
        checks.reset_refit_log()
        if order is None:
            for _ in range(passes):
                for f in range(frames):
                    checks.record_refit(f)
        else:
            for f in order:
                checks.record_refit(f)
        return checks.refit_summary(frames, 2)

    s1 = seq(2)
    record("B1 两遍×3帧=6 次、顺序 [0,1,2,0,1,2] → PASS",
           s1["ok"] and s1["actual_calls"] == 6 and s1["actual_sequence"] == [0, 1, 2, 0, 1, 2],
           {"actual": s1["actual_sequence"]})
    s2 = seq(1)
    record("B2 只记一遍（3 次）→ FAIL（镜像 --repeat 2 第二遍漏 refit）",
           not s2["ok"] and s2["actual_calls"] == 3, {"actual": s2["actual_sequence"], "ok": s2["ok"]})
    s3 = seq(0, order=[0, 2, 1, 0, 1, 2])
    record("B3 乱序 [0,2,1,0,1,2] → FAIL", not s3["ok"],
           {"actual": s3["actual_sequence"], "ok": s3["ok"]})
    s4 = seq(3)
    record("B4 多记（9 次）→ FAIL", not s4["ok"] and s4["actual_calls"] == 9,
           {"actual_calls": s4["actual_calls"], "ok": s4["ok"]})
    record("B5 反例所测函数即采集路径使用的判定函数（run_capture_checks 内联 refit_summary）",
           '"refit": refit_summary(' in checks_source,
           {"wired": '"refit": refit_summary(' in checks_source})

    # ---------- C. 三条硬自查（真实 base 数据 + 合成位移） ----------
    import numpy as np

    def load_npz(p: pathlib.Path) -> dict:
        with np.load(p) as a:
            return {k: a[k] for k in a.files}

    base_state = load_npz(BASE / "state.npz")
    base_arrays: dict = {}
    for kind in ("rgb", "depth", "seg"):
        base_arrays.update(load_npz(BASE / f"{kind}.npz"))

    import mujoco

    mjm = mujoco.MjModel.from_xml_path(str(SCENE))
    vertbody = np.asarray(mjm.flex_vertbodyid).reshape(-1)
    attached_idx = [i for i, b in enumerate(vertbody) if b != 0]
    base_pos = np.asarray(base_state["flexvert_xpos"], dtype=np.float64).reshape(-1, 3)

    def states_with(offset: float = 0.1, noise: float = 0.0):
        out = []
        for k in range(3):
            pos = base_pos.copy()
            for i in attached_idx:
                pos[i, 2] += k * offset + (noise if k else 0.0)
            out.append({"flexvert_xpos": pos.astype(np.float32)})
        return out

    states = states_with()
    fake_first = {"state": base_state, "arrays": base_arrays, "states": states, "mjm": mjm}
    c1 = checks.check_frame000_vs_base(str(REPO), fake_first)
    record("C1 base 幂等 PASS（真实 base 四类产物 15 数组位级全等）",
           c1["ok"] and c1["arrays_checked"] == 15, {"arrays_checked": c1["arrays_checked"],
                                                     "mismatches": c1["mismatches"][:2]})
    broken_state = {k: v.copy() for k, v in base_state.items()}
    broken_state["qpos"].view(np.uint32).reshape(-1)[0] ^= 1
    c2 = checks.check_frame000_vs_base(str(REPO), {"state": broken_state, "arrays": base_arrays,
                                                   "states": states, "mjm": mjm})
    record("C2 base 幂等 FAIL（qpos 改 1 bit）", not c2["ok"], {"mismatches": c2["mismatches"][:2]})
    d1 = checks.check_displacement(states_with(), mjm, 0.1)
    record("C3 位移 PASS（附着顶点均值 = k×0.100 m，读采集侧 state）", d1["ok"],
           {k: round(v["attached_mean_m"], 9) for k, v in d1["frames"].items()})
    d2 = checks.check_displacement(states_with(noise=2e-6), mjm, 0.1)
    record("C4 位移 FAIL（+2e-6 超 ±1e-6）", not d2["ok"],
           {k: v["delta_m"] for k, v in d2["frames"].items()})
    d3 = checks.check_displacement(states_with(noise=5e-7), mjm, 0.1)
    record("C5 位移边界（+5e-7 在 ±1e-6 内）PASS", d3["ok"],
           {k: v["delta_m"] for k, v in d3["frames"].items()})
    disp_tree = ast.parse(inspect.getsource(checks.check_displacement))
    disp_fn = next(n for n in ast.walk(disp_tree) if isinstance(n, ast.FunctionDef))
    disp_names = {n.id for n in ast.walk(disp_fn) if isinstance(n, ast.Name)}
    disp_strs = {n.value for n in ast.walk(disp_fn)
                 if isinstance(n, ast.Constant) and isinstance(n.value, str)}
    record("C6 位移数据源为 state.flexvert_xpos（设备回读），函数体不引用 CPU mjd 变量",
           "mjd" not in disp_names and "flexvert_xpos" in disp_strs,
           {"names": sorted(disp_names), "has_flexvert_key": "flexvert_xpos" in disp_strs,
            "mjd_name": "mjd" in disp_names})

    def delta_result(depth_changed: int, seg_changed: int) -> dict:
        pairs = [{"pair": "000->001", "depth_changed": depth_changed, "seg_changed": seg_changed},
                 {"pair": "001->002", "depth_changed": depth_changed, "seg_changed": seg_changed}]
        overall = "pass-margin" if depth_changed >= 576 and seg_changed >= 576 else (
            "non-discriminating" if min(depth_changed, seg_changed) < 384 else "insufficient-margin")
        return {"report": pairs, "overall": overall, "thresholds": {"accept_min": 384, "margin_min": 576}}

    m_ok = checks.check_delta_margins(delta_result(600, 600), 576)
    m_low = checks.check_delta_margins(delta_result(500, 600), 576)
    m_non = checks.check_delta_margins(delta_result(300, 600), 576)
    record("C7 变化余量 PASS（600/600 ≥576）", m_ok["ok"], m_ok["overall"])
    record("C8 变化余量 FAIL（depth 500 <576）", not m_low["ok"], m_low["overall"])
    record("C9 变化余量 FAIL（depth 300 <384，non-discriminating）", not m_non["ok"], m_non["overall"])
    record("C10 阈值仍硬编码 384/576、CLI 无改值入口",
           checks.DELTA_ACCEPT_MIN == 384 and checks.DELTA_MARGIN_MIN == 576
           and "accept-min" not in main_source and "margin-min" not in main_source,
           {"accept": checks.DELTA_ACCEPT_MIN, "margin": checks.DELTA_MARGIN_MIN})

    class Api:
        ROOT = str(REPO)

    def bundle(first, delta, recorded_passes=2, expected_passes=2) -> dict:
        checks.reset_refit_log()
        for _ in range(recorded_passes):
            for f in range(3):
                checks.record_refit(f)
        return checks.run_capture_checks(Api(), first, delta, 3, expected_passes, 0.1)

    b_ok = bundle(fake_first, delta_result(600, 600))
    record("C11 run_capture_checks 全 PASS（refit 6 / base / 位移 / 余量）",
           b_ok["ok"] and not b_ok["failures"], {"failures": b_ok["failures"]})
    b_refit = bundle(fake_first, delta_result(600, 600), recorded_passes=1, expected_passes=2)
    record("C12 run_capture_checks 命中 refit 失败（只记一遍）", "refit" in b_refit["failures"],
           {"failures": b_refit["failures"]})
    b_base = bundle({"state": broken_state, "arrays": base_arrays, "states": states, "mjm": mjm},
                    delta_result(600, 600))
    record("C13 run_capture_checks 命中 frame000_vs_base 失败（1 bit）",
           "frame000_vs_base" in b_base["failures"], {"failures": b_base["failures"]})
    b_disp = bundle({"state": base_state, "arrays": base_arrays,
                     "states": states_with(noise=2e-6), "mjm": mjm}, delta_result(600, 600))
    record("C14 run_capture_checks 命中 displacement 失败（+2e-6）",
           "displacement" in b_disp["failures"], {"failures": b_disp["failures"]})
    b_margin = bundle(fake_first, delta_result(500, 600))
    record("C15 run_capture_checks 命中 delta_margins 失败（500<576）",
           "delta_margins" in b_margin["failures"], {"failures": b_margin["failures"]})

    # ---------- D. provenance ----------
    prov = checks.provenance_block(Api())
    spec_bytes = SPEC.read_bytes()
    record("D1 provenance spec 双 sha（1d9fa5ce…/10,506 B）与实际 spec 一致",
           prov["spec_sha256"] == SPEC_SHA and prov["spec_bytes"] == 10506 == len(spec_bytes)
           and hashlib.sha256(spec_bytes).hexdigest() == prov["spec_sha256"],
           {"sha": prov["spec_sha256"][:16], "bytes": prov["spec_bytes"]})
    want_scripts = {"capture_render_flex.py", "capture_render_flex_multiframe.py",
                    "capture_render_flex_multiframe_checks.py"}
    sha_ok = all(prov["authoring_script_sha256"][n] == hashlib.sha256((SCRIPTS / n).read_bytes()).hexdigest()
                 for n in prov["authoring_script_sha256"])
    record("D2 provenance 三脚本 sha 与实际文件一致 + refit_bvh=true + policy=every-frame",
           sha_ok and set(prov["authoring_script_sha256"]) == want_scripts
           and prov["refit_bvh"] is True and prov["refit_policy"] == "every-frame",
           {"scripts": sorted(prov["authoring_script_sha256"]), "refit_bvh": prov["refit_bvh"]})
    record("D3 capture.json 落 provenance/refit_policy/refit_calls（=frames×repeat=6）",
           '"provenance": checks["provenance"]' in main_source
           and '"refit_calls": FRAMES * args.repeat' in main_source,
           {"provenance": '"provenance": checks["provenance"]' in main_source})

    # ---------- E. CLI 行为（子进程，CPU-only） ----------
    WORK.mkdir(parents=True, exist_ok=True)
    py = sys.executable
    capture_cli = str(single_path)

    def run_cli(*args: str) -> subprocess.CompletedProcess:
        return subprocess.run([py, "-B", capture_cli, *args], cwd=str(REPO), capture_output=True)

    h = run_cli("--help")
    help_text = decode_child(h.stdout + h.stderr)
    record("E1 --help exit0 且无 --refit-bvh", h.returncode == 0 and "--refit-bvh" not in help_text,
           {"exit": h.returncode})
    e2 = run_cli("--frames", "3", "--frame-dz", "0.1", "--out-dir", str(WORK / "never"),
                 "--refit-bvh", "--dry-run")
    record("E2 传 --refit-bvh 被 argparse 拒（exit2）", e2.returncode == 2, {"exit": e2.returncode})
    for args, expect, label in (
        (("--frames", "0", "--frame-dz", "0.1", "--out-dir", str(WORK / "never")), 1, "--frames 0"),
        (("--frames", "4", "--frame-dz", "0.1", "--out-dir", str(WORK / "never")), 1, "--frames 4"),
        (("--frames", "3", "--frame-dz", "0.2", "--out-dir", str(WORK / "never")), 1, "--frame-dz 0.2"),
        (("--frames", "3", "--frame-dz", "0.1", "--repeat", "1", "--out-dir", str(WORK / "never")), 1,
         "--repeat 1"),
    ):
        p = run_cli(*args)
        record(f"E3 {label} exit={expect}", p.returncode == expect,
               {"exit": p.returncode,
                "msg": decode_child((p.stderr or p.stdout)).strip()[-140:]})

    dry_dir = WORK / "dry-run"
    if dry_dir.exists():
        import shutil

        shutil.rmtree(dry_dir)
    e7 = run_cli("--frames", "3", "--frame-dz", "0.1", "--out-dir", str(dry_dir), "--dry-run")
    plan = {}
    if e7.returncode == 0:
        out = decode_child(e7.stdout)
        plan = json.loads(out[out.find("{"):])
    record("E4 dry-run exit0、计划 refit_policy=every-frame / refit_calls=6、不写产物",
           e7.returncode == 0 and plan.get("refit_policy") == "every-frame"
           and plan.get("refit_calls") == 6 and not dry_dir.exists(),
           {"exit": e7.returncode, "refit_policy": plan.get("refit_policy"),
            "refit_calls": plan.get("refit_calls"), "out_dir_created": dry_dir.exists()})

    def write_synth(root: pathlib.Path, depth_changed: int, seg_changed: int) -> None:
        for frame in range(3):
            d = root / f"frame-{frame:03d}"
            d.mkdir(parents=True, exist_ok=True)
            depth = np.zeros((240, 320), dtype=np.float32)
            seg = np.full((240, 320, 2), -1, dtype=np.int32)
            if frame > 0:
                depth.reshape(-1)[:depth_changed] = np.float32(frame)
                seg.reshape(-1, 2)[:seg_changed] = np.asarray([frame, 9], dtype=np.int32)
            np.savez(d / "depth.npz", depth_cam2=depth)
            np.savez(d / "seg.npz", seg_cam2=seg)

    for name, (dc, sc, expect) in {"pass": (600, 600, 0), "insufficient": (500, 600, 3),
                                   "non-discriminating": (300, 600, 3)}.items():
        root = WORK / f"synth-{name}"
        if root.exists():
            import shutil

            shutil.rmtree(root)
        write_synth(root, dc, sc)
        proc = run_cli("--check-deltas", "--out-dir", str(root))
        record(f"E5 --check-deltas {name} exit={expect}", proc.returncode == expect,
               {"exit": proc.returncode, "expected": expect})

    # ---------- F. 冻结面 / 边界 / 行数 / 文档引用 ----------
    r = git("diff", "--name-only", FIX_RANGE, "--", "src/", "tests/", "fixtures/", "Cargo.toml", "Cargo.lock")
    record("F1 修复范围 77aeb6da..HEAD 无 src/tests/fixtures/Cargo 改动", not r.stdout.strip(),
           {"changed": (r.stdout or "").split("\n")[:5]})
    spec_same = git("diff", "--name-only", FIX_RANGE, "--", SPECDIR).stdout.strip() == ""
    cur_sha = hashlib.sha256(spec_bytes).hexdigest()
    record("F2 冻结 spec 零改动（sha/bytes 1d9fa5ce…/10,506）",
           spec_same and cur_sha == SPEC_SHA and len(spec_bytes) == 10506,
           {"sha": cur_sha[:16], "bytes": len(spec_bytes)})
    record("F3 无 multiframe fixtures 目录、无 npz 产物",
           not (REPO / "fixtures/render-bvh/scene-flex/multiframe").exists()
           and git("ls-files", "fixtures").stdout.count("multiframe") == 0
           and git("diff", "--name-only", FIX_RANGE, "--", "*.npz").stdout.strip() == "",
           {"multiframe_dir": (REPO / "fixtures/render-bvh/scene-flex/multiframe").exists()})
    counts = {rel.split("/")[-1]: blob("HEAD", rel).count(b"\n") for rel in (
        "scripts/authoring/capture_render_flex.py",
        "scripts/authoring/capture_render_flex_multiframe.py",
        "scripts/authoring/capture_render_flex_multiframe_checks.py")}
    record("F4 行数 498 / 427 / 296（git blob 换行口径），两模块 ≤500",
           counts.get("capture_render_flex.py") == 498
           and counts.get("capture_render_flex_multiframe.py") == 427
           and counts.get("capture_render_flex_multiframe_checks.py") == 296, counts)
    record("F5 单帧主脚本逐字节不变（工作树 vs 77aeb6da）",
           single_path.read_bytes() == blob("77aeb6da", "scripts/authoring/capture_render_flex.py"),
           {"sha": hashlib.sha256(single_path.read_bytes()).hexdigest()[:16]})
    ref_ok = None
    if (UPSTREAM / "mujoco_warp/record.py").is_file() and (UPSTREAM / "contrib/render.py").is_file():
        rl = (UPSTREAM / "mujoco_warp/record.py").read_text(encoding="utf-8").split("\n")
        cl = (UPSTREAM / "contrib/render.py").read_text(encoding="utf-8").split("\n")
        ref_ok = ("mjw.refit_bvh" in rl[109] and "mjw.render" in rl[110]
                  and "mjw.refit_bvh" in cl[451] and "mjw.render" in cl[452])
    record("F6 上游行号内容核对（record.py:110-111 / contrib/render.py:452-453 = refit→render）",
           ref_ok is True, {"upstream_present": ref_ok is not None, "content_ok": ref_ok})
    docs = {"scripts/authoring/README.md": (SCRIPTS / "README.md").read_text(encoding="utf-8"),
            "capture_render_flex_multiframe_checks.py": checks_source,
            "capture_render_flex_multiframe.py": main_source}
    bad = [n for n, t in docs.items() if "_src/record" in t or "contrib/render.py:110" in t]
    full_ref = [n for n, t in docs.items() if "mujoco_warp/record.py:110-111" in t]
    contrib_ok = all("contrib/render.py:452-453" in t for t in docs.values())
    record("F7 文档引用修正：无 `_src/record`、无 `contrib/render.py:110`；正确路径齐全",
           not bad and "scripts/authoring/README.md" in full_ref
           and "capture_render_flex_multiframe_checks.py" in full_ref and contrib_ok,
           {"bad": bad, "full_path_in": full_ref, "contrib_ok": contrib_ok})
    evid_ok = False
    base_dir = "docs/development-evidence/render-bvh/r06-flex-multiframe-refit-fix-r2"
    try:
        raw = (EVID / "manifest.txt").read_bytes()
        lines = [ln for ln in raw.decode("utf-8").splitlines() if ln and not ln.startswith("#")]
        evid_ok = raw.startswith(b"#") and len(lines) == 11
        for ln in lines:
            sha, size, rel = ln.split()
            full = REPO / base_dir / rel
            if not full.is_file() or hashlib.sha256(full.read_bytes()).hexdigest() != sha \
                    or len(full.read_bytes()) != int(size):
                evid_ok = False
    except Exception as exc:  # noqa: BLE001 - 记录而不隐藏
        print("manifest check error:", exc)
    record("F8 证据 manifest 11/11 自核（evidence-dir-relative + base dir）", evid_ok)
    gates = {g: json.loads((RUNS / f"{g}.json").read_text(encoding="utf-8"))["passed"]
             for g in ("fmt", "clippy", "clippy-cuda", "unit")}
    units = json.loads((RUNS / "unit.json").read_text(encoding="utf-8"))["tests"]
    record("F9 四门禁记录齐全（unit 1321 过 / 0 败）",
           all(gates.values()) and units["passed"] == 1321 and units["failed"] == 0,
           {"gates": gates, "unit": units})
    raw_status = git("status", "--porcelain").stdout.strip()
    own = (".normify/changes/2026-11-05-render-r06-flex-multiframe-refit-fix-r2-check.json",
           "docs/development-evidence/render-bvh/r06-flex-multiframe-refit-fix-r2-check/")
    foreign = [ln for ln in raw_status.splitlines()
               if ln.strip() and not any(o in ln for o in own)]
    record("F10 工作树除本复核证据/卡外 clean（无未预期改动）", not foreign,
           {"dirty": raw_status[:300], "foreign": foreign[:5]})
    terms = ["多帧已通过", "动画一致性已覆盖", "上游等价", "对外发布依据"]
    evid_text = "\n".join(p.read_text(encoding="utf-8", errors="replace")
                          for p in EVID.rglob("*") if p.is_file() and p.suffix in (".md", ".json", ".txt"))
    hits = [t for t in terms if t in evid_text]
    record("F11 forbid_claim 四词 0 命中（证据文本）", not hits, {"hits": hits})

    # ---------- 汇总 ----------
    failed = [it["name"] for it in RESULTS if not it["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "total": len(RESULTS),
               "passed": len(RESULTS) - len(failed), "fails": failed, "checks": RESULTS,
               "head": git("rev-parse", "HEAD").stdout.strip()}
    if "--json" in sys.argv:
        out_path = pathlib.Path(sys.argv[sys.argv.index("--json") + 1])
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
                            encoding="utf-8", newline="\n")
        print("json written:", out_path.as_posix())
    print(f"\nRESULT: {payload['result']} ({payload['passed']}/{payload['total']} pass, {len(failed)} fail)")
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
