"""G05@P4 离线 authoring：执行器力（fwd_actuation）的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g05-actuation.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/actuation/actuation-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g05-p4/authoring.json
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "actuation"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "actuation-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g05-p4"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
NWORLD = 2
QPOS = [[0.2, -0.4], [0.7, 0.1]]
QVEL = [[0.5, -0.2], [-0.3, 0.8]]
CTRL = [[0.4, -0.6, 0.9], [2.0, 0.3, -1.5]]
ACT = [[0.5], [-0.7]]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in values]


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    nq, nv, nu, na, nact = (
        int(mjm.nq),
        int(mjm.nv),
        int(mjm.nu),
        int(mjm.na),
        int(mjm.nactuator),
    )
    if (nq, nv, nu, na, nact) != (2, 2, 3, 1, 3):
        raise SystemExit(f"unexpected model dims nq={nq} nv={nv} nu={nu} na={na} nact={nact}")

    model = mujoco_warp.put_model(mjm)
    data = mujoco_warp.make_data(mjm, nworld=NWORLD)
    warp.copy(
        data.qpos, warp.array(np.asarray(QPOS, dtype=np.float32).reshape(NWORLD, nq))
    )
    warp.copy(
        data.qvel, warp.array(np.asarray(QVEL, dtype=np.float32).reshape(NWORLD, nv))
    )
    warp.copy(
        data.ctrl, warp.array(np.asarray(CTRL, dtype=np.float32).reshape(NWORLD, nu))
    )
    warp.copy(data.act, warp.array(np.asarray(ACT, dtype=np.float32).reshape(NWORLD, na)))
    mujoco_warp.transmission(model, data)
    mujoco_warp.fwd_velocity(model, data)
    mujoco_warp.fwd_actuation(model, data)

    def flat(array) -> list[float]:
        return f32_list(np.asarray(array.numpy()).reshape(-1))

    moment_rownnz = data.moment_rownnz.numpy()
    moment_rowadr = data.moment_rowadr.numpy()
    moment_colind = data.moment_colind.numpy().astype(np.int64)
    actuator_moment = data.actuator_moment.numpy()
    moment_width = int(moment_colind.shape[1])

    output = {
        "act_dot": flat(data.act_dot),
        "actuator_force": flat(data.actuator_force),
        "qfrc_actuator": flat(data.qfrc_actuator),
    }

    # 原生一级判定：同一初始状态按原生 f64 阶段链计算。
    mjd = mujoco.MjData(mjm)
    mjd.qpos[:] = np.asarray(QPOS[0], dtype=np.float64)
    mjd.qvel[:] = np.asarray(QVEL[0], dtype=np.float64)
    mjd.ctrl[:] = np.asarray(CTRL[0], dtype=np.float64)
    mjd.act[:] = np.asarray(ACT[0], dtype=np.float64)
    mujoco.mj_fwdPosition(mjm, mjd)
    mujoco.mj_fwdVelocity(mjm, mjd)
    mujoco.mj_fwdActuation(mjm, mjd)
    native = {
        "act_dot": [float(value) for value in np.asarray(mjd.act_dot)],
        "actuator_force": [float(value) for value in np.ravel(mjd.actuator_force)],
        "qfrc_actuator": [float(value) for value in np.asarray(mjd.qfrc_actuator)],
    }
    native_max = 0.0
    for field, size in (("act_dot", na), ("actuator_force", nact), ("qfrc_actuator", nv)):
        product = np.asarray(output[field], dtype=np.float64)[:size]
        reference = np.asarray(native[field], dtype=np.float64)
        if product.shape != reference.shape:
            raise SystemExit(f"{field} shape mismatch {product.shape} vs {reference.shape}")
        native_max = max(native_max, float(np.max(np.abs(product - reference))))

    upstream_head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if upstream_head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {upstream_head} != pinned {PINNED_REVISION}")
    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()

    fixture = {
        "task_id": "2026-10-11-roadmap-g05-p4",
        "matrix": "G05",
        "phase": "P4",
        "upstream_functions": ["transmission", "fwd_velocity", "fwd_actuation"],
        "model": {
            "path": "fixtures/actuation/model.xml",
            "sha256": sha256(MODEL_XML),
            "nq": nq,
            "nv": nv,
            "nu": nu,
            "na": na,
            "nactuator": nact,
            "timestep": float(mjm.opt.timestep),
        },
        "model_arrays": {
            "dyntype": [int(value) for value in mjm.actuator_dyntype.ravel()],
            "gaintype": [int(value) for value in mjm.actuator_gaintype.ravel()],
            "biastype": [int(value) for value in mjm.actuator_biastype.ravel()],
            "actadr": [int(value) for value in mjm.actuator_actadr.ravel()],
            "actnum": [int(value) for value in mjm.actuator_actnum.ravel()],
            "ctrladr": [int(value) for value in mjm.actuator_ctrladr.ravel()],
            "ctrlnum": [int(value) for value in mjm.actuator_ctrlnum.ravel()],
            "dynprm": f32_list(np.ravel(mjm.actuator_dynprm)),
            "gainprm": f32_list(np.ravel(mjm.actuator_gainprm)),
            "biasprm": f32_list(np.ravel(mjm.actuator_biasprm)),
            "actlimited": [bool(value) for value in mjm.actuator_actlimited.ravel()],
            "actrange": f32_list(np.ravel(mjm.actuator_actrange)),
            "actearly": [bool(value) for value in mjm.actuator_actearly.ravel()],
            "forcelimited": [bool(value) for value in mjm.actuator_forcelimited.ravel()],
            "forcerange": f32_list(np.ravel(mjm.actuator_forcerange)),
            "ctrllimited": [bool(value) for value in mjm.actuator_ctrllimited.ravel()],
            "ctrlrange": f32_list(np.ravel(mjm.actuator_ctrlrange)),
        },
        "nworld": NWORLD,
        "initial": {
            "ctrl": f32_list(np.asarray(CTRL, dtype=np.float32).reshape(-1)),
            "act": f32_list(np.asarray(ACT, dtype=np.float32).reshape(-1)),
            "qpos": f32_list(np.asarray(QPOS, dtype=np.float32).reshape(-1)),
        },
        "frozen_inputs": {
            "ctrl": f32_list(np.asarray(CTRL, dtype=np.float32).reshape(-1)),
            "act": f32_list(np.asarray(ACT, dtype=np.float32).reshape(-1)),
            "actuator_length": flat(data.actuator_length),
            "actuator_velocity": flat(data.actuator_velocity),
            "moment_width": moment_width,
            "moment_rownnz": [int(value) for value in moment_rownnz.reshape(-1)],
            "moment_rowadr": [int(value) for value in moment_rowadr.reshape(-1)],
            "moment_colind": [int(value) for value in moment_colind.reshape(-1)],
            "actuator_moment": flat(data.actuator_moment),
        },
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "functions": ["mujoco_warp.transmission", "mujoco_warp.fwd_velocity", "mujoco_warp.fwd_actuation"],
        },
        "expected": output,
        "native_cross_check": {
            "max_abs_vs_native": native_max,
            "note": "原生 mj_fwdPosition/mj_fwdVelocity/mj_fwdActuation（f64）先做一级判定；上游 MJWarp f32 输出为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 0.0,
            "native_abs": 1e-05,
            "note": "上游 f32 输出逐位比较（容差 0）；原生 f64 一级判定容许 1e-5 绝对差。",
        },
    }
    # 初始 qpos 只用于说明；产品不消费（joint 传动长度/速度已冻结）。
    fixture["initial"]["qpos"] = f32_list(np.asarray(QPOS, dtype=np.float32).reshape(-1))

    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g05-actuation.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nworld": NWORLD,
        "nactuator": nact,
        "na": na,
        "moment_width": moment_width,
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/actuation/actuation-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(
        "g05 actuation authoring OK:"
        f" nact={nact} na={na} nworld={NWORLD} moment_width={moment_width}"
        f" native_max_abs={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
