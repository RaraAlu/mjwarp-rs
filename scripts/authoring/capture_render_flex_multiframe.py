"""R06 多帧采集脚本扩展：`capture_render_flex.py` 的帧循环模式与变化像素自检（authoring only）。

由 `scripts/authoring/capture_render_flex.py` 的 `dispatch()` 调用（同一 CLI 入口）：
- 帧循环模式 `--frames 3 --frame-dz 0.1`（spec §2 冻值）：第 k 帧把 `flex-anchor0/1/2` 的
  `body_pos.z` 加 `k * 0.10` 后 `put_model` → forward → 渲染，逐帧落盘
  `frame-{000,001,002}/{rgb,depth,seg,state}.npz`，顶层写 `capture.json` / `cam-poses.json` /
  `README.md`；输出根目录必须显式 `--out-dir`（不在本脚本内写任何 fixtures 路径）。
- `--check-deltas`：读取采集结果，计算 cam2 相邻帧 depth 位级 / seg 对变化像素数，按 spec §5
  硬编码常量（384 / 576）打印判定；不提供改阈值/改帧数/改 Δz 的参数。
- 采集路径末尾强制跑同一自检（`_delta_report`），verdict 写入 `capture.json.delta_selfcheck`；
  非 `pass-margin` 时打印「不得落库」警告并以 **exit 3** 退出（与 `--check-deltas` 的 0/3 同义），
  避免把停手交给人工记忆（QA O3）。
- `--frames` 只接受 1（原单帧路径）或 3（spec §2 冻值）：`--frames 0`/负数显式 exit 1（QA O2）。
- 不带上述参数时 `dispatch()` 返回 None，`capture_render_flex.py` 原有单帧路径逐字节不变。

边界：本模块属于离线 authoring 工具，不被产品构建/测试/cargo 门禁引用；不执行采集、不占用 GPU
窗口；不改判据/阈值/帧数/Δz/相机；不写任何验收结论。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import sys
import time

# ---- 冻结常量（spec `2026-11-05-render-r06-flex-multiframe-criterion-spec`；不得改） ----
SPEC_TASK = "2026-11-05-render-r06-flex-multiframe-criterion-spec"
SPEC_PATH = "docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md"
SPEC_SHA256 = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6"
SPEC_BYTES = 10506
FRAMES = 3  # spec §2：frame-000/001/002
FRAME_DZ = 0.10  # spec §2：每步 +0.100 m
ANCHOR_BODIES = ("flex-anchor0", "flex-anchor1", "flex-anchor2")  # spec §2
DELTA_ACCEPT_MIN = 384  # spec §5：0.5% × 76,800，硬编码
DELTA_MARGIN_MIN = 576  # spec §5：1.5 × 384，硬编码
CAM2_INDEX = 2  # spec §3：判据相机 = cam2（cam-flex）
CAM2_WIDTH, CAM2_HEIGHT = 320, 240
CAM2_PIXELS = CAM2_WIDTH * CAM2_HEIGHT
MULTIFRAME_KIND = "render-flex-multiframe-capture"
SCRIPT_TASK_ID = "2026-11-05-render-r06-flex-multiframe-capture-script"


def add_frame_args(parser: argparse.ArgumentParser) -> None:
    """把帧循环/自检参数挂到现有 capture CLI（默认值不改变单帧路径）。"""
    parser.add_argument(
        "--frames",
        type=int,
        default=1,
        help=(
            f"帧数：只接受 1（原单帧路径）或 {FRAMES}（spec §2 冻值；帧循环）；"
            "其它值（含 0/负数）显式拒绝"
        ),
    )
    parser.add_argument(
        "--frame-dz",
        type=float,
        default=FRAME_DZ,
        help=f"每帧锚点 body_pos.z 步进（spec §2 冻结为 {FRAME_DZ:.2f} m）",
    )
    parser.add_argument(
        "--check-deltas",
        action="store_true",
        help="读取 --out-dir 的采集结果，打印 cam2 相邻帧 depth/seg 变化像素自检（常量 384/576）",
    )
    parser.add_argument(
        "--refit-bvh",
        action="store_true",
        help=(
            "【仅诊断用，默认关】帧循环每帧 mjw.forward 之后、渲染之前调用 mjw.refit_bvh(m, d, rc)；"
            "用于验证「场景 BVH 停在 base → 渲染位移被剔除」假设，不改变生产默认行为"
        ),
    )


def dispatch(args: argparse.Namespace, out_dir: pathlib.Path, api) -> int | None:
    """CLI 分派：返回 None 表示走原单帧路径（逐字节不变）；否则返回退出码。"""
    if args.frames not in (1, FRAMES):
        api.fail(
            f"spec §2 冻结：--frames 只接受 1（单帧原路径）或 {FRAMES}（帧循环），"
            f"收到 {args.frames}；其它值（含 0/负数）一律拒绝"
        )
    frame_mode = args.frames > 1 or args.frame_dz != FRAME_DZ
    if args.check_deltas:
        if frame_mode:
            api.fail("--check-deltas 不能与 --frames/--frame-dz 同用（spec §5 冻结口径）")
        return check_deltas(args, out_dir, api)
    if frame_mode:
        if args.dry_run:
            return dry_run_plan(args, api)
        return capture_multiframe(args, out_dir, api)
    return None


# ----------------------------------------------------------------------------------
# 冻结前提校验 & 产物读写辅助
# ----------------------------------------------------------------------------------


def verify_frozen_spec(api) -> dict:
    """运行时校验 spec 文件与冻结 sha256/bytes 一致；不一致直接拒绝运行。"""
    path = pathlib.Path(api.ROOT) / SPEC_PATH
    if not path.is_file():
        api.fail(f"冻结 spec 不存在：{SPEC_PATH}")
    data = path.read_bytes()
    if len(data) != SPEC_BYTES or hashlib.sha256(data).hexdigest() != SPEC_SHA256:
        api.fail(f"冻结 spec 与常量不一致（sha256/bytes）：{SPEC_PATH}")
    return {"task_id": SPEC_TASK, "path": SPEC_PATH, "sha256": SPEC_SHA256, "bytes": SPEC_BYTES}


def _sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_json(path: pathlib.Path, payload: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    return _sha256(path)


def _frame_bodies(api, mjm) -> list[int]:
    import mujoco

    ids = []
    for name in ANCHOR_BODIES:
        body = mujoco.mj_name2id(mjm, mujoco.mjtObj.mjOBJ_BODY, name)
        if body < 0:
            api.fail(f"场景缺少锚点 body：{name}")
        ids.append(int(body))
    return ids


def _base_anchor_z(api, mjm, anchors: list[int]) -> list[float]:
    import numpy as np

    pos = np.asarray(mjm.body_pos)
    return [float(pos[body, 2]) for body in anchors]


def _render_frame(api, scene: pathlib.Path, frame: int, anchor_ids: list[int], rc, refit: bool = False):
    """按 spec §2 注入偏移后执行 put_model → forward → 渲染，返回 capture_once 结果。"""
    import mujoco
    import mujoco_warp as mjw
    import warp as wp

    mjm = mujoco.MjModel.from_xml_path(str(scene))
    if frame > 0:
        offset = frame * FRAME_DZ
        for body in anchor_ids:
            mjm.body_pos[body, 2] = mjm.body_pos[body, 2] + offset
    m = mjw.put_model(mjm)
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    d = mjw.put_data(mjm, mjd, nworld=1)
    mjw.forward(m, d)
    wp.synchronize()
    if refit:  # 诊断开关（--refit-bvh）：上游 render 不重建 BVH，几何变动后需显式 refit（record.py:110 同款）
        mjw.refit_bvh(m, d, rc)
        wp.synchronize()
    capture = api.capture_once(mjm, m, d, rc)
    return mjm, d, capture


def _frame_file_groups(capture: dict) -> dict[str, dict]:
    groups: dict[str, dict] = {
        kind: {name: value for name, value in capture["arrays"].items() if name.startswith(f"{kind}_")}
        for kind in ("rgb", "depth", "seg")
    }
    groups["state"] = capture["state"]
    return groups


def _capture_pass(api, root: pathlib.Path, scene: pathlib.Path, anchor_ids: list[int], rc, refit: bool = False) -> dict:
    """一次完整帧循环采集，返回 files_sha256 / frame-000 的 state 与相机位姿。"""
    hashes: dict[str, str] = {}
    first_state: dict | None = None
    first_poses: dict | None = None
    first_mjm = None
    render_seconds = []
    for frame in range(FRAMES):
        mjm, d, capture = _render_frame(api, scene, frame, anchor_ids, rc, refit)
        frame_dir = root / f"frame-{frame:03d}"
        for name, arrays in _frame_file_groups(capture).items():
            hashes[f"frame-{frame:03d}/{name}.npz"] = api.save_npz_deterministic(frame_dir / f"{name}.npz", arrays)
        render_seconds.append(capture["render_seconds"])
        if frame == 0:
            first_state = capture["state"]
            first_poses = api.flex_cam_pose_record(d)
            first_mjm = mjm
    return {
        "files_sha256": hashes,
        "state": first_state,
        "poses": first_poses,
        "mjm": first_mjm,
        "render_seconds": render_seconds,
    }


# ----------------------------------------------------------------------------------
# 帧循环采集
# ----------------------------------------------------------------------------------


def capture_multiframe(args: argparse.Namespace, out_dir: pathlib.Path, api) -> int:
    frame_spec = verify_frozen_spec(api)
    if args.frames != FRAMES:
        api.fail(f"spec §2 冻结：帧数必须为 {FRAMES}（收到 {args.frames}）；改帧数需新 spec + 用户重签")
    if args.frame_dz != FRAME_DZ:
        api.fail(f"spec §2 冻结：Δz 必须为 {FRAME_DZ:.2f} m（收到 {args.frame_dz}）；改 Δz 需新 spec + 用户重签")
    if args.out_dir is None:
        api.fail("帧循环模式必须显式 --out-dir（输出根目录参数化；本脚本不写默认 fixtures 路径）")
    if args.repeat != 2:
        api.fail("spec §8.4 冻结：两次采集逐位一致校验要求 --repeat 2（不得跳过校验）")
    if args.cam_poses_only:
        api.fail("--cam-poses-only 不能与帧循环模式同用")
    if api.PARAMS["use_fast_math"]:
        api.fail("spec §8.1 冻结：recipe 要求 use_fast_math=false")

    import mujoco
    import mujoco_warp as mjw
    import warp as wp

    wp.set_device("cuda:0")
    environment = api.check_environment(want_cuda_device=True)
    scene = pathlib.Path(args.scene).resolve()
    legacy_before = api.legacy_manifest()
    mjm = mujoco.MjModel.from_xml_path(str(scene))
    anchor_ids = _frame_bodies(api, mjm)
    anchor_z = _base_anchor_z(api, mjm, anchor_ids)
    rc = mjw.create_render_context(mjm, **api.PARAMS)

    started = time.perf_counter()
    refit = bool(getattr(args, "refit_bvh", False))
    first = _capture_pass(api, out_dir, scene, anchor_ids, rc, refit)
    repeat_dir = out_dir / ".repeat2"
    second = _capture_pass(api, repeat_dir, scene, anchor_ids, rc, refit)
    import shutil

    if first["files_sha256"] != second["files_sha256"]:
        api.fail(
            "spec §8.4：两次采集非逐位一致（逐帧逐文件 sha256 不同）\n"
            + json.dumps({"first": first["files_sha256"], "second": second["files_sha256"]}, indent=2, ensure_ascii=False)
        )
    shutil.rmtree(repeat_dir, ignore_errors=True)
    if api.legacy_manifest() != legacy_before:
        api.fail("采集期间既有 render-bvh 参考被改动（legacy_reference_untouched=false）")

    state_equal = api.state_bitwise_equal(first["state"], out_dir / "frame-000" / "state.npz")
    if not all(state_equal.values()):
        api.fail(f"frame-000 state 与落盘 state.npz 非逐位一致：{state_equal}")
    cam_poses_path = api.write_cam_poses(
        out_dir, first["mjm"], first["poses"], environment, state_equal
    )

    per_frame: dict[str, dict[str, str]] = {f"frame-{k:03d}": {} for k in range(FRAMES)}
    for name, sha in sorted(first["files_sha256"].items()):
        frame, filename = name.split("/", 1)
        per_frame[frame][filename] = sha
    # O3：采集完成后强制跑同一自检；verdict 写入 capture.json，并以自检状态决定返回码。
    delta_lines = _delta_report(out_dir, api, stream=False)
    capture = {
        "schema": api.SCHEMA,
        "task_id": api.TASK_ID,
        "script_task_id": SCRIPT_TASK_ID,
        "kind": MULTIFRAME_KIND,
        "spec": frame_spec,
        "frames": FRAMES,
        "frame_dz": FRAME_DZ,
        "frame_indices": list(range(FRAMES)),
        "anchor_bodies": list(ANCHOR_BODIES),
        "anchor_body_ids": anchor_ids,
        "anchor_body_base_z": anchor_z,
        "parameters": dict(api.PARAMS),
        "refit_bvh": refit,
        "seed": args.seed,
        "repeat": args.repeat,
        "two_captures_bitwise_identical": True,
        "files_sha256": first["files_sha256"],
        "per_frame_files_sha256": per_frame,
        "cam_poses": {"file": cam_poses_path.name, "sha256": _sha256(cam_poses_path)},
        "render_seconds": first["render_seconds"],
        "capture_seconds": round(time.perf_counter() - started, 6),
        "captured_at": os.environ.get("MJWARP_CAPTURE_AT", ""),
        "command": [sys.executable, "-B"] + sys.argv,
        "environment": environment,
        "delta_constants": {
            "accept_min": DELTA_ACCEPT_MIN,
            "margin_min": DELTA_MARGIN_MIN,
            "cam2_index": CAM2_INDEX,
            "cam2_pixels": CAM2_PIXELS,
        },
        "delta_selfcheck": {
            "thresholds": delta_lines["thresholds"],
            "cam2_pixels": delta_lines["cam2_pixels"],
            "report": delta_lines["report"],
            "overall": delta_lines["overall"],
        },
        "legacy_reference_untouched": True,
        "legacy_reference_files": len(legacy_before),
        "reads_product_tree": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    capture_sha = _write_json(out_dir / "capture.json", capture)
    readme = _multiframe_readme(frame_spec, anchor_ids, anchor_z)
    (out_dir / "README.md").write_text(readme, encoding="utf-8", newline="\n")
    _print_delta_report(delta_lines)
    summary = {
        "script_task_id": SCRIPT_TASK_ID,
        "kind": MULTIFRAME_KIND,
        "out_dir": out_dir.as_posix(),
        "frames": FRAMES,
        "frame_dz": FRAME_DZ,
        "files_sha256": first["files_sha256"],
        "capture_json_sha256": capture_sha,
        "cam_poses_sha256": _sha256(cam_poses_path),
        "two_captures_bitwise_identical": True,
        "legacy_reference_untouched": True,
        "environment": {"warp": environment["versions"]["warp"], "mujoco": environment["versions"]["mujoco"],
                        "mujoco_warp": environment["versions"]["mujoco_warp"], "python": environment["versions"]["python"]},
        "delta_report": delta_lines["report"],
        "delta_verdict": delta_lines["overall"],
    }
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    # O3：非 pass-margin 时以非零码收尾（3 = 与 --check-deltas 同义），避免把停手交给人工记忆。
    return delta_exit_code(delta_lines)


def _multiframe_readme(frame_spec: dict, anchor_ids: list[int], anchor_z: list[float]) -> str:
    return f"""\
# R06 多帧渲染参考（脚本帧循环产物）

- 生成命令：`{sys.executable} -B {' '.join(sys.argv)}`
- 冻结 spec：`{frame_spec['task_id']}`（sha256 `{frame_spec['sha256']}` / {frame_spec['bytes']} B）
- 帧集（spec §2 冻值）：frame-000 / frame-001 / frame-002；锚点 `body_pos.z` 每帧 `+{FRAME_DZ:.2f} m`
  （锚点 body：{', '.join(f'{name}=id{body}' for name, body in zip(ANCHOR_BODIES, anchor_ids))}，
  base z = {', '.join(f'{z:.6f}' for z in anchor_z)}）
- 每帧产物：`frame-{{000,001,002}}/{{rgb,depth,seg,state}}.npz`；顶层 `capture.json` / `cam-poses.json` /
  本 README。`state.npz` 字段同 base：qpos / geom_xpos / geom_xmat / light_xpos / light_xdir / flexvert_xpos。
- 两次采集：`--repeat 2`，逐帧逐文件 sha256 一致（`capture.json.two_captures_bitwise_identical = true`）。
- 变化像素自检（spec §5）：结论见 `capture.json.delta_selfcheck.overall`；非 `pass-margin` 时脚本
  **exit 3** 并打印「不得落库」警告（`--check-deltas` 独立模式同语义）。
- 本目录只是离线 authoring 产物记录；帧内/跨帧判定口径见冻结 spec，本 README 不作结论。
"""


# ----------------------------------------------------------------------------------
# cam2 相邻帧变化像素自检（spec §5；阈值 384/576 硬编码，不提供改值参数）
# ----------------------------------------------------------------------------------


def _load_cam2(root: pathlib.Path, frame: int, filename: str):
    import numpy as np

    path = root / f"frame-{frame:03d}" / filename
    if not path.is_file():
        raise SystemExit(f"R06 multiframe delta self-check: missing {path}")
    with np.load(path) as archive:
        key = f"{filename.split('.')[0]}_cam{CAM2_INDEX}"
        if key not in archive.files:
            raise SystemExit(f"R06 multiframe delta self-check: {path} missing {key}")
        return np.asarray(archive[key])


def _verdict(depth_changed: int, seg_changed: int) -> str:
    if depth_changed < DELTA_ACCEPT_MIN or seg_changed < DELTA_ACCEPT_MIN:
        return "non-discriminating"
    if depth_changed >= DELTA_MARGIN_MIN and seg_changed >= DELTA_MARGIN_MIN:
        return "pass-margin"
    return "insufficient-margin"


def _delta_report(root: pathlib.Path, api, *, stream: bool = True) -> dict:
    import numpy as np

    frames = list(range(FRAMES))
    depth = [_load_cam2(root, frame, "depth.npz") for frame in frames]
    seg = [_load_cam2(root, frame, "seg.npz") for frame in frames]
    for frame in frames:
        if depth[frame].shape != (CAM2_HEIGHT, CAM2_WIDTH):
            api.fail(f"cam2 depth 形状 {depth[frame].shape} != ({CAM2_HEIGHT}, {CAM2_WIDTH})（spec §3 冻结）")
        if seg[frame].shape != (CAM2_HEIGHT, CAM2_WIDTH, 2):
            api.fail(f"cam2 seg 形状 {seg[frame].shape} != ({CAM2_HEIGHT}, {CAM2_WIDTH}, 2)")
        if int(np.prod(depth[frame].shape)) != CAM2_PIXELS:
            api.fail("cam2 像素数不是 76,800（spec §3 冻结）")
    report = []
    overall = "pass-margin"
    for left in frames[:-1]:
        right = left + 1
        depth_changed = int(np.count_nonzero(depth[left].view(np.uint32) != depth[right].view(np.uint32)))
        seg_changed = int(np.count_nonzero(np.any(seg[left] != seg[right], axis=2)))
        verdict = _verdict(depth_changed, seg_changed)
        if verdict != "pass-margin":
            overall = "insufficient-margin" if overall == "pass-margin" else overall
        if verdict == "non-discriminating":
            overall = "non-discriminating"
        report.append({
            "pair": f"{left:03d}->{right:03d}",
            "depth_changed": depth_changed,
            "seg_changed": seg_changed,
            "verdict": verdict,
        })
    result = {"thresholds": {"accept_min": DELTA_ACCEPT_MIN, "margin_min": DELTA_MARGIN_MIN},
              "cam2_pixels": CAM2_PIXELS, "report": report, "overall": overall}
    if stream:
        _print_delta_report(result)
    return result


def _print_delta_report(result: dict) -> None:
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


def delta_exit_code(result: dict) -> int:
    """自检 verdict → 退出码：pass-margin = 0，其余 = 3（采集路径与 --check-deltas 共用）。"""
    return 0 if result["overall"] == "pass-margin" else 3


def check_deltas(args: argparse.Namespace, out_dir: pathlib.Path, api) -> int:
    if args.out_dir is None:
        api.fail("--check-deltas 必须显式 --out-dir（读取采集结果的根目录）")
    root = pathlib.Path(out_dir)
    if not root.is_dir():
        api.fail(f"--check-deltas 目录不存在：{root.as_posix()}")
    result = _delta_report(root, api, stream=True)
    return delta_exit_code(result)


# ----------------------------------------------------------------------------------
# 干跑（CPU only，不占 GPU、不渲染、不写采集产物）
# ----------------------------------------------------------------------------------


def dry_run_plan(args: argparse.Namespace, api) -> int:
    import mujoco

    if args.frames != FRAMES:
        api.fail(f"spec §2 冻结：帧数必须为 {FRAMES}（收到 {args.frames}）")
    if args.frame_dz != FRAME_DZ:
        api.fail(f"spec §2 冻结：Δz 必须为 {FRAME_DZ:.2f} m（收到 {args.frame_dz}）")
    frame_spec = verify_frozen_spec(api)
    if args.out_dir is None:
        api.fail("帧循环模式必须显式 --out-dir（输出根目录参数化）")
    scene = pathlib.Path(args.scene).resolve()
    mjm = mujoco.MjModel.from_xml_path(str(scene))
    anchor_ids = _frame_bodies(api, mjm)
    anchor_z = _base_anchor_z(api, mjm, anchor_ids)
    plan = {
        "script_task_id": SCRIPT_TASK_ID,
        "mode": "multiframe-dry-run",
        "dry_run": True,
        "spec": frame_spec,
        "frames": FRAMES,
        "frame_dz": FRAME_DZ,
        "anchor_bodies": list(ANCHOR_BODIES),
        "anchor_body_ids": anchor_ids,
        "anchor_body_base_z": anchor_z,
        "scene": scene.as_posix(),
        "scene_sha256": api.digest(scene),
        "out_dir": pathlib.Path(args.out_dir).resolve().as_posix(),
        "repeat": args.repeat,
        "parameters": dict(api.PARAMS),
        "refit_bvh": bool(getattr(args, "refit_bvh", False)),
        "expected_layout": [
            "frame-000/{rgb,depth,seg,state}.npz",
            "frame-001/{rgb,depth,seg,state}.npz",
            "frame-002/{rgb,depth,seg,state}.npz",
            "capture.json",
            "cam-poses.json",
            "README.md",
        ],
        "delta_constants": {"accept_min": DELTA_ACCEPT_MIN, "margin_min": DELTA_MARGIN_MIN},
        "gpu_used": False,
        "env": {"python": platform.python_version(), "mujoco": mujoco.__version__},
        "note": "CPU-only 计划检查；不渲染、不写采集产物、不开窗口。",
    }
    print(json.dumps(plan, indent=2, ensure_ascii=False))
    return 0
