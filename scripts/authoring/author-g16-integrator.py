"""G16@P6 离线 authoring：Euler 与 RK4 积分状态推进的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g16-integrator.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/integrator/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g16-p6/authoring.json
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import mujoco
import numpy as np
import warp as wp
from mujoco_warp._src import forward as fwd

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "integrator"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT / "docs" / "development-evidence" / "backlog-batch2"
    / "2026-10-11-roadmap-g16-p6"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g16-p6"

CASES = [
    (
        "default",
        [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.5, -0.3, 0.2, 0.4, 0.1, -0.2, 0.3, -0.5, 0.2, 0.7, -0.6],
    ),
    (
        "rotated",
        [
            0.2, -0.1, 0.8, 0.9238795, 0.3826834, 0.0, 0.0,
            0.9659258, 0.0, 0.258819, 0.0, 0.7, -0.4,
        ],
        [-1.2, 0.4, 0.3, -2.0, 0.5, 1.0, -1.5, 0.8, 0.6, -2.2, 1.4],
    ),
    (
        "spinning",
        [
            -0.3, 0.2, 1.2, 1.0, 0.0, 0.0, 0.0,
            0.70710678, 0.0, 0.0, 0.70710678, -1.2, 0.9,
        ],
        [0.1, 0.2, 0.3, 3.0, -1.0, 2.0, 0.5, 2.5, -3.0, 4.0, -2.0],
    ),
]
A_TABLEAU = [0.5, 0.5, 1.0]
B_TABLEAU = [1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def perturb(model, data, scale, qpos_t0, qvel_t0):
    wp.launch(
        fwd._next_position,
        dim=(data.nworld, model.njnt),
        inputs=[
            model.opt.timestep,
            model.jnt_type,
            model.jnt_qposadr,
            model.jnt_dofadr,
            qpos_t0,
            data.qvel,
            scale,
        ],
        outputs=[data.qpos],
    )
    wp.launch(
        fwd._next_velocity,
        dim=(data.nworld, model.nv),
        inputs=[model.opt.timestep, qvel_t0, data.qacc, scale],
        outputs=[data.qvel],
    )


def upstream_stages(mjw, model, mjm, qpos, qvel):
    data = mjw.make_data(mjm, nworld=1)
    data.qpos.assign(np.asarray([qpos], dtype=np.float32))
    data.qvel.assign(np.asarray([qvel], dtype=np.float32))
    mjw.forward(model, data)
    wp.synchronize_device()
    stages = [(np.asarray(data.qvel.numpy())[0], np.asarray(data.qacc.numpy())[0])]
    qpos_t0 = wp.clone(data.qpos)
    qvel_t0 = wp.clone(data.qvel)
    for scale in A_TABLEAU:
        perturb(model, data, scale, qpos_t0, qvel_t0)
        mjw.forward(model, data)
        wp.synchronize_device()
        stages.append(
            (np.asarray(data.qvel.numpy())[0], np.asarray(data.qacc.numpy())[0])
        )
    return stages


def recompose(stages):
    qvel_rk = np.zeros_like(stages[0][0])
    qacc_rk = np.zeros_like(stages[0][1])
    for (qvel, qacc), weight in zip(stages, B_TABLEAU):
        qvel_rk = qvel_rk + np.float32(weight) * qvel
        qacc_rk = qacc_rk + np.float32(weight) * qacc
    return qvel_rk, qacc_rk


def native_euler(mjm, qpos, qvel):
    data = mujoco.MjData(mjm)
    data.qpos[:] = np.asarray(qpos, dtype=np.float64)
    data.qvel[:] = np.asarray(qvel, dtype=np.float64)
    mujoco.mj_forward(mjm, data)
    timestep = float(mjm.opt.timestep)
    # Python 绑定未导出 mj_integrateVel；f64 参考直接按上游 _next_velocity 公式推进速度。
    data.qvel[:] = data.qvel + timestep * data.qacc
    mujoco.mj_integratePos(mjm, data.qpos, data.qvel, timestep)
    return np.asarray(data.qpos).copy(), np.asarray(data.qvel).copy()


def native_rk4_compose(mjm, qpos, qvel):
    timestep = float(mjm.opt.timestep)
    data = mujoco.MjData(mjm)
    data.qpos[:] = np.asarray(qpos, dtype=np.float64)
    data.qvel[:] = np.asarray(qvel, dtype=np.float64)
    mujoco.mj_forward(mjm, data)
    stages = [(np.asarray(data.qvel).copy(), np.asarray(data.qacc).copy())]
    qpos_t0 = np.asarray(qpos, dtype=np.float64).copy()
    qvel_t0 = np.asarray(qvel, dtype=np.float64).copy()
    qvel_stage, qacc_stage = stages[0]
    for scale in A_TABLEAU:
        qpos_stage = qpos_t0.copy()
        mujoco.mj_integratePos(mjm, qpos_stage, qvel_stage, scale * timestep)
        qvel_stage = qvel_t0 + (scale * qacc_stage) * timestep
        data.qpos[:] = qpos_stage
        data.qvel[:] = qvel_stage
        mujoco.mj_forward(mjm, data)
        stages.append((np.asarray(data.qvel).copy(), np.asarray(data.qacc).copy()))
    qvel_rk = np.zeros_like(qvel_t0)
    qacc_rk = np.zeros_like(qacc_stage)
    for (stage_vel, stage_acc), weight in zip(stages, B_TABLEAU):
        qvel_rk = qvel_rk + weight * stage_vel
        qacc_rk = qacc_rk + weight * stage_acc
    qvel_final = qvel_t0 + timestep * qacc_rk
    qpos_final = qpos_t0.copy()
    mujoco.mj_integratePos(mjm, qpos_final, qvel_rk, timestep)
    return qpos_final, qvel_final


def native_step(mjm_rk4, qpos, qvel):
    data = mujoco.MjData(mjm_rk4)
    data.qpos[:] = np.asarray(qpos, dtype=np.float64)
    data.qvel[:] = np.asarray(qvel, dtype=np.float64)
    mujoco.mj_forward(mjm_rk4, data)
    mujoco.mj_step(mjm_rk4, data)
    return np.asarray(data.qpos).copy(), np.asarray(data.qvel).copy()


def max_abs(left, right) -> float:
    return float(np.max(np.abs(np.asarray(left, dtype=np.float64) - np.asarray(right, dtype=np.float64))))


def main() -> int:
    import mujoco_warp as mjw

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    timestep = float(mjm.opt.timestep)
    mjm.opt.disableflags |= int(mujoco.mjtDisableBit.mjDSBL_EULERDAMP)
    model = mjw.put_model(mjm)
    mjm_rk4 = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    mjm_rk4.opt.integrator = mujoco.mjtIntegrator.mjINT_RK4

    cases = []
    for name, qpos, qvel in CASES:
        stages = upstream_stages(mjw, model, mjm, qpos, qvel)
        qvel_rk, qacc_rk = recompose(stages)

        euler_data = mjw.make_data(mjm, nworld=1)
        euler_data.qpos.assign(np.asarray([qpos], dtype=np.float32))
        euler_data.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.forward(model, euler_data)
        mjw.euler(model, euler_data)
        wp.synchronize_device()
        euler_qpos = np.asarray(euler_data.qpos.numpy())[0]
        euler_qvel = np.asarray(euler_data.qvel.numpy())[0]
        euler_time = np.asarray(euler_data.time.numpy())[0]

        rk4_data = mjw.make_data(mjm, nworld=1)
        rk4_data.qpos.assign(np.asarray([qpos], dtype=np.float32))
        rk4_data.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.forward(model, rk4_data)
        mjw.rungekutta4(model, rk4_data)
        wp.synchronize_device()
        rk4_qpos = np.asarray(rk4_data.qpos.numpy())[0]
        rk4_qvel = np.asarray(rk4_data.qvel.numpy())[0]
        rk4_time = np.asarray(rk4_data.time.numpy())[0]

        # 上游组合自洽性：用冻结 4 级导数重放组合，应复现上游最终状态（f32 舍入内）。
        replay = mjw.make_data(mjm, nworld=1)
        replay.qpos.assign(np.asarray([qpos], dtype=np.float32))
        replay.qvel.assign(np.asarray([qvel], dtype=np.float32))
        replay.qacc.assign(np.asarray([qacc_rk], dtype=np.float32))
        wp.launch(
            fwd._next_velocity,
            dim=(1, mjm.nv),
            inputs=[model.opt.timestep, replay.qvel, replay.qacc, 1.0],
            outputs=[replay.qvel],
        )
        wp.launch(
            fwd._next_position,
            dim=(1, mjm.njnt),
            inputs=[
                model.opt.timestep,
                model.jnt_type,
                model.jnt_qposadr,
                model.jnt_dofadr,
                replay.qpos,
                wp.array(np.asarray([qvel_rk], dtype=np.float32), dtype=float),
                1.0,
            ],
            outputs=[replay.qpos],
        )
        wp.synchronize_device()
        replay_max = max_abs(
            np.concatenate([rk4_qpos, rk4_qvel]),
            np.concatenate(
                [
                    np.asarray(replay.qpos.numpy())[0],
                    np.asarray(replay.qvel.numpy())[0],
                ]
            ),
        )

        native_euler_qpos, native_euler_qvel = native_euler(mjm, qpos, qvel)
        native_qpos, native_qvel = native_rk4_compose(mjm_rk4, qpos, qvel)
        step_qpos, step_qvel = native_step(mjm_rk4, qpos, qvel)

        cases.append(
            {
                "name": name,
                "qpos": f32_list(qpos),
                "qvel": f32_list(qvel),
                "euler": {
                    "qacc": f32_list(np.asarray(euler_data.qacc.numpy())[0]),
                    "qpos": f32_list(euler_qpos),
                    "qvel": f32_list(euler_qvel),
                    "time": float(np.float32(euler_time)),
                    "native_qpos": f32_list(native_euler_qpos),
                    "native_qvel": f32_list(native_euler_qvel),
                },
                "rk4": {
                    "stages": [
                        {"qvel": f32_list(qvel_stage), "qacc": f32_list(qacc_stage)}
                        for qvel_stage, qacc_stage in stages
                    ],
                    "qpos": f32_list(rk4_qpos),
                    "qvel": f32_list(rk4_qvel),
                    "time": float(np.float32(rk4_time)),
                    "native_qpos": f32_list(native_qpos),
                    "native_qvel": f32_list(native_qvel),
                    "native_step_qpos": f32_list(step_qpos),
                    "native_step_qvel": f32_list(step_qvel),
                    "replay_max_abs": replay_max,
                },
            }
        )
        print(
            f"case {name}: euler_time={euler_time} rk4_time={rk4_time}"
            f" replay_max={replay_max:.3e}"
            f" native_rk4_vs_step={max_abs(np.concatenate([native_qpos, native_qvel]), np.concatenate([step_qpos, step_qvel])):.3e}"
        )

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G16",
        "phase": "P6",
        "upstream_functions": ["euler", "rungekutta4"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "上游 euler 在禁用 EULERDAMP 时走直接 _advance；RK4 4 级导数由上游 forward 逐级采集后冻结。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "joints": {
            "nq": int(mjm.nq),
            "nv": int(mjm.nv),
            "njnt": int(mjm.njnt),
            "jnt_type": [int(value) for value in mjm.jnt_type],
            "jnt_qposadr": [int(value) for value in mjm.jnt_qposadr],
            "jnt_dofadr": [int(value) for value in mjm.jnt_dofadr],
            "timestep": f32_list([timestep])[0],
        },
        "tableau": {
            "A": f32_list(A_TABLEAU),
            "B": f32_list(B_TABLEAU),
        },
        "cases": cases,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_mul": 1.0,
            "native_abs": 1e-05,
            "note": "产品与上游逐位比较（mul=1 表示逐位）；原生 f64 一级判定容差 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g16-integrator.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "cases": len(cases),
        "joints": {"nq": fixture["joints"]["nq"], "nv": fixture["joints"]["nv"]},
        "native_cross_check": {
            "euler_max_abs": max(
                max_abs(
                    np.concatenate([case["euler"]["qpos"], case["euler"]["qvel"]]),
                    np.concatenate(
                        [case["euler"]["native_qpos"], case["euler"]["native_qvel"]]
                    ),
                )
                for case in cases
            ),
            "rk4_max_abs": max(
                max_abs(
                    np.concatenate([case["rk4"]["qpos"], case["rk4"]["qvel"]]),
                    np.concatenate([case["rk4"]["native_qpos"], case["rk4"]["native_qvel"]]),
                )
                for case in cases
            ),
            "rk4_compose_vs_native_step": max(
                max_abs(
                    np.concatenate([case["rk4"]["native_qpos"], case["rk4"]["native_qvel"]]),
                    np.concatenate(
                        [case["rk4"]["native_step_qpos"], case["rk4"]["native_step_qvel"]]
                    ),
                )
                for case in cases
            ),
            "note": "原生 f64：euler 用 mj_forward+mj_integratePos 与 _next_velocity 公式；RK4 用原生 f64 复刻 4 级组合并对照原生 mj_step(RK4)。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/integrator/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g16-p6 integrator authoring OK:"
        f" cases={len(cases)} euler_max={evidence['native_cross_check']['euler_max_abs']:.3e}"
        f" rk4_max={evidence['native_cross_check']['rk4_max_abs']:.3e}"
        f" compose_vs_step={evidence['native_cross_check']['rk4_compose_vs_native_step']:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
