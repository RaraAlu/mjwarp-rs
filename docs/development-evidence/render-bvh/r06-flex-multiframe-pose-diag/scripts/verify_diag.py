"""R06 多帧诊断 A/B 证据自检（CPU-only，零 GPU）。

用法（仓库根）：target/mjwarp-parallel/venv/Scripts/python.exe -B \
    docs/development-evidence/render-bvh/r06-flex-multiframe-pose-diag/scripts/verify_diag.py

检查：
  A. analysis.json 关键事实：A/B 两组 refit 标记、verdict、cam2 flex 像素数、变化计数、位移、state 组间逐位相同。
  B. 证据内 state.npz 重算位移：附着顶点均值应为 0.100 / 0.200 m；两组逐帧 state 逐位相同。
  C. frame-000 vs base：证据内 state.npz 与 fixtures 参考 base/state.npz 逐元素位级。
  D. 边界事实：fixtures/src/tests 零 diff；冻结 spec sha256/bytes 未变。
  E. 诊断开关：HEAD blob 无 refit；工作树有 `--refit-bvh`（action=store_true 默认关）且 `_render_frame` 条件调用
     `mjw.refit_bvh`；脚本行数 ≤500。
  F. 证据文本编码：UTF-8/LF/无 BOM（manifest 覆盖的文本文件）。
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parents[5]
EVID = pathlib.Path(__file__).resolve().parents[1]
WORK = REPO / "target/r06-multiframe-diag"
BASE = REPO / "fixtures/render-bvh/scene-flex/reference/base"
SPEC = REPO / "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SCRIPT = REPO / "scripts/authoring/capture_render_flex_multiframe.py"

RESULTS: list[dict] = []


def record(name: str, ok: bool, detail) -> None:
    RESULTS.append({"name": name, "ok": bool(ok), "detail": detail})
    print(f"[{'PASS' if ok else 'FAIL'}] {name}: {detail}")


def load_npz(path: pathlib.Path):
    import numpy as np

    with np.load(path) as archive:
        return {name: archive[name] for name in archive.files}


def bits(array):
    import numpy as np

    return np.ascontiguousarray(array).view(np.uint32)


def git(*argv: str) -> bytes:
    return subprocess.run(["git", *argv], cwd=REPO, check=True, capture_output=True).stdout


def main() -> int:
    analysis = json.loads((EVID / "analysis/analysis.json").read_text(encoding="utf-8"))
    groups = analysis["groups"]

    # A. analysis.json 关键事实
    record("A1 A 组 refit=False / verdict=non-discriminating",
           groups["A_norefit"]["capture_refit_bvh"] is False
           and groups["A_norefit"]["capture_delta_selfcheck"]["overall"] == "non-discriminating",
           {"refit": groups["A_norefit"]["capture_refit_bvh"],
            "overall": groups["A_norefit"]["capture_delta_selfcheck"]["overall"]})
    record("A2 B 组 refit=True / verdict=pass-margin",
           groups["B_refit"]["capture_refit_bvh"] is True
           and groups["B_refit"]["capture_delta_selfcheck"]["overall"] == "pass-margin",
           {"refit": groups["B_refit"]["capture_refit_bvh"],
            "overall": groups["B_refit"]["capture_delta_selfcheck"]["overall"]})
    a_cam2 = groups["A_norefit"]["C_render_consumption"]["per_camera"]["cam2"]
    b_cam2 = groups["B_refit"]["C_render_consumption"]["per_camera"]["cam2"]
    record("A3 cam2 flex 像素：A 崩塌 / B 保持", a_cam2["flex_pixels_by_frame"] == [654, 246, 69]
           and b_cam2["flex_pixels_by_frame"] == [654, 718, 798],
           {"A": a_cam2["flex_pixels_by_frame"], "B": b_cam2["flex_pixels_by_frame"]})
    a_pairs = {pair: (p["depth_changed"], p["seg_changed"]) for pair, p in a_cam2["pairs"].items()}
    b_pairs = {pair: (p["depth_changed"], p["seg_changed"]) for pair, p in b_cam2["pairs"].items()}
    record("A4 A 组复现窗口计数 697/616、244/183",
           a_pairs == {"000->001": (697, 616), "001->002": (244, 183)}, a_pairs)
    record("A5 B 组两段均 ≥576（pass-margin 门槛）",
           all(d >= 576 and s >= 576 for d, s in b_pairs.values()), b_pairs)
    record("A6 A/B state 组间逐位相同", analysis["D_cross_group"]["state_all_identical"] is True,
           analysis["D_cross_group"]["state_all_identical"])
    record("A7 frame-000 两组均与 base 位级相同",
           groups["A_norefit"]["A_frame000_vs_base"]["all_bit_equal"] is True
           and groups["B_refit"]["A_frame000_vs_base"]["all_bit_equal"] is True,
           {"A": groups["A_norefit"]["A_frame000_vs_base"]["all_bit_equal"],
            "B": groups["B_refit"]["A_frame000_vs_base"]["all_bit_equal"]})

    # B. 从证据内 state.npz 独立重算位移
    import numpy as np

    def displacement(group: str, frame: int) -> float:
        base_state = load_npz(EVID / f"groups/{group}/frame-000/state.npz")["flexvert_xpos"]
        state = load_npz(EVID / f"groups/{group}/frame-{frame:03d}/state.npz")["flexvert_xpos"]
        delta = np.asarray(state, dtype=np.float64).reshape(-1, 3) - np.asarray(
            base_state, dtype=np.float64).reshape(-1, 3)
        mags = np.linalg.norm(delta, axis=1)
        return float(mags[1:].mean())  # 顶点 0 附着 world；其余 5 个附着 anchor

    disp = {group: [round(displacement(group, k), 6) for k in range(3)] for group in ("norefit", "refit")}
    ok_disp = all(disp[group] == [0.0, 0.1, 0.2] for group in disp)
    record("B1 证据内 state 重算位移 = 0 / 0.100 / 0.200 m（两组）", ok_disp, disp)

    state_equal = True
    for frame in range(3):
        sa = load_npz(EVID / f"groups/norefit/frame-{frame:03d}/state.npz")
        sb = load_npz(EVID / f"groups/refit/frame-{frame:03d}/state.npz")
        for key in sa:
            if not np.array_equal(bits(sa[key]), bits(sb[key])):
                state_equal = False
    record("B2 证据内 A/B 逐帧 state 逐元素位级相同", state_equal, state_equal)

    # C. frame-000 vs base（独立复核）
    base_state = load_npz(BASE / "state.npz")
    frames_equal = {}
    for group in ("norefit", "refit"):
        frame_state = load_npz(EVID / f"groups/{group}/frame-000/state.npz")
        frames_equal[group] = all(
            np.array_equal(bits(frame_state[key]), bits(base_state[key])) for key in frame_state
        )
    record("C1 证据内 frame-000 state == base state（两组）", all(frames_equal.values()), frames_equal)

    # D. 边界事实
    diff = git("diff", "--stat", "HEAD", "--", "fixtures", "src", "tests").decode().strip()
    record("D1 fixtures/src/tests 零 diff", diff == "", diff or "(empty)")
    spec = SPEC.read_bytes()
    record("D2 冻结 spec sha/bytes 未变",
           hashlib.sha256(spec).hexdigest() == "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
           and len(spec) == 10506,
           {"sha256": hashlib.sha256(spec).hexdigest()[:16] + "…", "bytes": len(spec)})

    # E. 诊断开关
    head_blob = git("show", "HEAD:scripts/authoring/capture_render_flex_multiframe.py").decode("utf-8")
    work_text = SCRIPT.read_text(encoding="utf-8")
    record("E1 HEAD blob 无 refit（根因：采集脚本从未 refit）", "refit" not in head_blob,
           "refit" in head_blob)
    record("E2 工作树有默认关的 --refit-bvh",
           '"--refit-bvh"' in work_text and 'action="store_true"' in work_text
           and "mjw.refit_bvh(m, d, rc)" in work_text and "if refit:" in work_text,
           {"flag": '"--refit-bvh"' in work_text,
            "default_off": 'action="store_true"' in work_text,
            "guarded_call": "if refit:" in work_text})
    lines = SCRIPT.read_bytes().count(b"\n")
    record("E3 脚本行数 ≤500", lines <= 500, lines)

    # F. 编码
    text_files = sorted(EVID.rglob("*"))
    bad = []
    for path in text_files:
        if not path.is_file() or path.name.endswith(".npz"):
            continue
        data = path.read_bytes()
        if data[:3] == b"\xef\xbb\xbf" or b"\r\n" in data:
            bad.append(path.relative_to(EVID).as_posix())
    record("F1 证据文本 UTF-8/LF/无 BOM", not bad, bad or "(all ok)")

    failed = [item["name"] for item in RESULTS if not item["ok"]]
    payload = {"result": "PASS" if not failed else "FAIL", "fails": failed, "checks": RESULTS}
    (EVID / "analysis/verify-diag.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"RESULT: {payload['result']} ({len(RESULTS) - len(failed)}/{len(RESULTS)} pass, {len(failed)} fail)")
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
