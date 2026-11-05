"""R06 多帧采集自检与 provenance（拆自 `capture_render_flex_multiframe.py`，守 500 行棘轮）。

职责（采集路径内建，失败一律 exit 3 停手）：
- **refit 计次**：`frames × passes`（3 × 2 = 6）次 render 前恰好 6 次 refit；顺序必须是
  每遍 `frame0,frame1,frame2`，无分支可跳过（G-1 负结果的镜像：`--repeat 2` 第二遍漏 refit）。
- **① base 幂等**：`frame-000` 的 rgb/depth/seg/state 与 `reference/base/` **数组级位级全等**。
- **② 位移断言**：读**采集侧 state**（设备回读落盘的 `flexvert_xpos`）——附着到非 world body 的顶点
  相对 base 帧平均位移 = `k × 0.100 m`（±1e-6）；不用 CPU `mjd.flexvert_xpos`。
- **③ 变化像素余量**：`000→001` 与 `001→002` 的 depth 与 seg 变化像素均 ≥ 576（pass-margin 口径）。
- **delta 自检**（`--check-deltas` 与采集末尾共用）：cam2 相邻帧 depth 位级 / seg 对变化像素 + 三态判定。
- **provenance**：`spec_sha256`（`1d9fa5ce…`/10,506 B）+ `authoring_script_sha256`（修复后三脚本）
  + refit 标记，供 QA 区分采集轮次、证明同一签名对象。

上游依据：`mujoco_warp/record.py:110-111`（callback 内 refit→render）与
`upstream/contrib/render.py:452-453`（每帧 `mjw.refit_bvh`→`mjw.render`，`contrib/` 在仓库根、不在包内）；
`render()` 自身不 refit（G-1 诊断卡 A/B 定论）。
"""

from __future__ import annotations

import hashlib
import pathlib

REFIT_POLICY = "every-frame"  # 每帧 forward 之后、render 之前无条件 refit
DISPLACEMENT_TOL = 1e-6  # 附着顶点平均位移与 k×FRAME_DZ 的允许偏差（m）
BASE_REFERENCE_DIR = "fixtures/render-bvh/scene-flex/reference/base"
SPEC_PATH = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
CHECK_SCRIPT_NAMES = (
    "capture_render_flex.py",
    "capture_render_flex_multiframe.py",
    "capture_render_flex_multiframe_checks.py",
)

# ---- spec 冻值（采集/自检共用；改这些需新 spec + 用户重签） ----
FRAMES = 3  # spec §2：frame-000/001/002
DELTA_ACCEPT_MIN = 384  # spec §5：0.5% × 76,800，硬编码
DELTA_MARGIN_MIN = 576  # spec §5：1.5 × 384，硬编码
CAM2_INDEX = 2  # spec §3：判据相机 = cam2（cam-flex）
CAM2_WIDTH, CAM2_HEIGHT = 320, 240
CAM2_PIXELS = CAM2_WIDTH * CAM2_HEIGHT

# 每帧 refit 的调用记录（顺序即调用序；CPU 自检据此断言 frames × passes）。
REFIT_LOG: list[dict] = []


def reset_refit_log() -> None:
    REFIT_LOG.clear()


def record_refit(frame: int) -> None:
    """记录一次「render 前 refit」；每帧必须恰好调用一次。"""
    REFIT_LOG.append({"frame": int(frame)})


def refit_summary(frames: int, passes: int) -> dict:
    """断言 refit 次数与顺序：frames × passes，且每遍按帧序 0..frames-1。"""
    expected_sequence = list(range(frames)) * passes
    actual_sequence = [entry["frame"] for entry in REFIT_LOG]
    expected_calls = frames * passes
    return {
        "policy": REFIT_POLICY,
        "ok": actual_sequence == expected_sequence and len(actual_sequence) == expected_calls,
        "expected_calls": expected_calls,
        "actual_calls": len(actual_sequence),
        "expected_sequence": expected_sequence,
        "actual_sequence": actual_sequence,
        "frames": frames,
        "passes": passes,
    }


# ----------------------------------------------------------------------------------
# delta 自检（spec §5；阈值 384/576 硬编码，不提供改值参数）
# ----------------------------------------------------------------------------------


def load_cam2(root: pathlib.Path, frame: int, filename: str):
    import numpy as np

    path = root / f"frame-{frame:03d}" / filename
    if not path.is_file():
        raise SystemExit(f"R06 multiframe delta self-check: missing {path}")
    with np.load(path) as archive:
        key = f"{filename.split('.')[0]}_cam{CAM2_INDEX}"
        if key not in archive.files:
            raise SystemExit(f"R06 multiframe delta self-check: {path} missing {key}")
        return np.asarray(archive[key])


def verdict(depth_changed: int, seg_changed: int) -> str:
    if depth_changed < DELTA_ACCEPT_MIN or seg_changed < DELTA_ACCEPT_MIN:
        return "non-discriminating"
    if depth_changed >= DELTA_MARGIN_MIN and seg_changed >= DELTA_MARGIN_MIN:
        return "pass-margin"
    return "insufficient-margin"


def print_delta_report(result: dict) -> None:
    """打印自检明细；非 pass-margin 时补「不得落库」警告（采集路径与独立自检共用）。"""
    print(
        "R06 multiframe delta self-check (spec §5; thresholds hardcoded 384/576, cam2 pixels 76,800)"
    )
    for item in result["report"]:
        print(
            f"  cam2 {item['pair']}: depth_changed={item['depth_changed']} "
            f"seg_changed={item['seg_changed']} -> {item['verdict']}"
        )
    print(f"  overall: {result['overall']}")
    if result["overall"] != "pass-margin":
        print("  spec §5：未达采集前置余量（>=576），不得落库为验收材料；停手报用户，不得改阈值。")


def delta_report(root: pathlib.Path, api, frames: int = FRAMES, *, stream: bool = True) -> dict:
    import numpy as np

    frame_indices = list(range(frames))
    depth = [load_cam2(root, frame, "depth.npz") for frame in frame_indices]
    seg = [load_cam2(root, frame, "seg.npz") for frame in frame_indices]
    for frame in frame_indices:
        if depth[frame].shape != (CAM2_HEIGHT, CAM2_WIDTH):
            api.fail(f"cam2 depth 形状 {depth[frame].shape} != ({CAM2_HEIGHT}, {CAM2_WIDTH})（spec §3 冻结）")
        if seg[frame].shape != (CAM2_HEIGHT, CAM2_WIDTH, 2):
            api.fail(f"cam2 seg 形状 {seg[frame].shape} != ({CAM2_HEIGHT}, {CAM2_WIDTH}, 2)")
        if int(np.prod(depth[frame].shape)) != CAM2_PIXELS:
            api.fail("cam2 像素数不是 76,800（spec §3 冻结）")
    report = []
    overall = "pass-margin"
    for left in frame_indices[:-1]:
        right = left + 1
        depth_changed = int(np.count_nonzero(depth[left].view(np.uint32) != depth[right].view(np.uint32)))
        seg_changed = int(np.count_nonzero(np.any(seg[left] != seg[right], axis=2)))
        pair_verdict = verdict(depth_changed, seg_changed)
        if pair_verdict != "pass-margin":
            overall = "insufficient-margin" if overall == "pass-margin" else overall
        if pair_verdict == "non-discriminating":
            overall = "non-discriminating"
        report.append({
            "pair": f"{left:03d}->{right:03d}",
            "depth_changed": depth_changed,
            "seg_changed": seg_changed,
            "verdict": pair_verdict,
        })
    result = {"thresholds": {"accept_min": DELTA_ACCEPT_MIN, "margin_min": DELTA_MARGIN_MIN},
              "cam2_pixels": CAM2_PIXELS, "report": report, "overall": overall}
    if stream:
        print_delta_report(result)
    return result


def delta_exit_code(result: dict) -> int:
    """自检 verdict → 退出码：pass-margin = 0，其余 = 3（采集路径与 --check-deltas 共用）。"""
    return 0 if result["overall"] == "pass-margin" else 3


# ----------------------------------------------------------------------------------
# provenance
# ----------------------------------------------------------------------------------


def _sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def provenance_block(api) -> dict:
    """采集 provenance：spec sha + 修复后 authoring 脚本 sha + refit 标记。"""
    root = pathlib.Path(api.ROOT)
    scripts_dir = pathlib.Path(__file__).resolve().parent
    spec = root / SPEC_PATH
    return {
        "spec_sha256": _sha256(spec),
        "spec_bytes": len(spec.read_bytes()),
        "spec_path": SPEC_PATH,
        "authoring_script_sha256": {
            name: _sha256(scripts_dir / name) for name in CHECK_SCRIPT_NAMES
        },
        "refit_policy": REFIT_POLICY,
        "refit_bvh": True,
    }


# ----------------------------------------------------------------------------------
# 三条硬自查
# ----------------------------------------------------------------------------------


def _bit_equal(actual, expected) -> bool:
    """数组级位级比较：浮点按 uint32 位型、整型按值；形状必须一致。"""
    import numpy as np

    if np.asarray(actual).shape != np.asarray(expected).shape:
        return False
    a = np.ascontiguousarray(actual)
    b = np.ascontiguousarray(expected)
    if a.dtype.kind == "f" or b.dtype.kind == "f":
        return np.array_equal(a.view(np.uint32), b.astype(a.dtype).view(np.uint32))
    return np.array_equal(a, b)


def check_frame000_vs_base(root, first: dict) -> dict:
    """① frame-000 与 reference/base 的四类产物数组级位级全等。"""
    import numpy as np

    base_dir = pathlib.Path(root) / BASE_REFERENCE_DIR
    mismatches: list[dict] = []
    if not base_dir.is_dir():
        return {"ok": False, "mismatches": [{"reason": f"base 目录不存在：{base_dir.as_posix()}"}]}

    with np.load(base_dir / "state.npz") as archive:
        base_state = {name: archive[name] for name in archive.files}
    arrays_checked = 0
    for name in sorted(base_state):
        arrays_checked += 1
        actual = first["state"].get(name)
        if actual is None or not _bit_equal(actual, base_state[name]):
            mismatches.append({"array": f"state.{name}", "bit_equal": False})
    for filename in ("rgb.npz", "depth.npz", "seg.npz"):
        with np.load(base_dir / filename) as archive:
            for name in sorted(archive.files):
                arrays_checked += 1
                actual = first["arrays"].get(name)
                if actual is None or not _bit_equal(actual, archive[name]):
                    mismatches.append({"array": f"{filename}:{name}", "bit_equal": False})
    return {
        "ok": not mismatches,
        "base_dir": base_dir.as_posix(),
        "arrays_checked": arrays_checked,
        "mismatches": mismatches,
    }


def check_displacement(states: list[dict], mjm, frame_dz: float) -> dict:
    """② 采集侧 state 的 flexvert_xpos 位移断言：附着顶点均值 = k × frame_dz（±tol）。"""
    import numpy as np

    positions = [np.asarray(state["flexvert_xpos"], dtype=np.float64).reshape(-1, 3) for state in states]
    vertbodyid = [int(v) for v in np.asarray(mjm.flex_vertbodyid).reshape(-1)]
    attached = [index for index, body in enumerate(vertbodyid) if body != 0]
    frames: dict[str, dict] = {}
    ok = True
    for frame, position in enumerate(positions):
        magnitudes = np.linalg.norm(position - positions[0], axis=1)
        mean_m = float(magnitudes[attached].mean()) if attached else 0.0
        expected_m = frame * frame_dz
        frame_ok = abs(mean_m - expected_m) <= DISPLACEMENT_TOL
        ok = ok and frame_ok
        frames[f"frame-{frame:03d}"] = {
            "attached_mean_m": mean_m,
            "expected_m": expected_m,
            "delta_m": mean_m - expected_m,
            "ok": frame_ok,
        }
    return {
        "ok": ok,
        "tolerance_m": DISPLACEMENT_TOL,
        "attached_vertices": len(attached),
        "world_vertices": len(vertbodyid) - len(attached),
        "source": "采集侧 state.npz 的 flexvert_xpos（设备回读），非 CPU mjd",
        "frames": frames,
    }


def check_delta_margins(delta_result: dict, margin_min: int) -> dict:
    """③ 两段相邻帧变化像素均 ≥ margin_min（pass-margin 口径）。"""
    pairs: dict[str, dict] = {}
    for item in delta_result["report"]:
        depth_ok = int(item["depth_changed"]) >= margin_min
        seg_ok = int(item["seg_changed"]) >= margin_min
        pairs[item["pair"]] = {
            "depth_changed": int(item["depth_changed"]),
            "seg_changed": int(item["seg_changed"]),
            "margin_min": margin_min,
            "ok": depth_ok and seg_ok,
        }
    return {
        "ok": bool(pairs) and all(entry["ok"] for entry in pairs.values()),
        "margin_min": margin_min,
        "overall": delta_result["overall"],
        "pairs": pairs,
    }


def run_capture_checks(api, first: dict, delta_result: dict, frames: int, passes: int, frame_dz: float) -> dict:
    """采集路径内建自检汇总：任一不通过 → ok=False（调用方 exit 3）。"""
    checks = {
        "refit": refit_summary(frames, passes),
        "frame000_vs_base": check_frame000_vs_base(api.ROOT, first),
        "displacement": check_displacement(first["states"], first["mjm"], frame_dz),
        "delta_margins": check_delta_margins(delta_result, DELTA_MARGIN_MIN),
    }
    failures = [name for name, result in checks.items() if not result["ok"]]
    return {
        "ok": not failures,
        "failures": failures,
        "checks": checks,
        "provenance": provenance_block(api),
    }
