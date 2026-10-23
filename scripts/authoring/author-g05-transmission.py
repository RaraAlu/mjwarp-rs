"""G05@P7 离线 authoring：关节传动（transmission）端到端冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g05-transmission.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/actuation/transmission-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g05-p7/authoring.json
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
MODEL_XML = ROOT / "fixtures" / "actuation" / "model.xml"
BASE_FIXTURE = ROOT / "fixtures" / "actuation" / "actuation-cases.json"
FIXTURE_JSON = ROOT / "fixtures" / "actuation" / "transmission-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g05-p7"
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
    flat = np.asarray(values, dtype=np.float64).reshape(-1)
    return [float(np.float32(value)) for value in flat]


def flat(array) -> list[float]:
    return f32_list(np.asarray(array))


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    base = json.loads(BASE_FIXTURE.read_text(encoding="utf-8"))
    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    nq, nv, nu, na, nact = int(mjm.nq), int(mjm.nv), int(mjm.nu), int(mjm.na), int(mjm.nactuator)
    if (nq, nv, nu, na, nact) != (2, 2, 3, 1, 3):
        raise SystemExit(f"unexpected model dims {nq} {nv} {nu} {na} {nact}")
    model = mujoco_warp.put_model(mjm)
    data = mujoco_warp.make_data(mjm, nworld=NWORLD)
    warp.copy(data.qpos, warp.array(np.asarray(QPOS, dtype=np.float32).reshape(NWORLD, nq)))
    warp.copy(data.qvel, warp.array(np.asarray(QVEL, dtype=np.float32).reshape(NWORLD, nv)))
    warp.copy(data.ctrl, warp.array(np.asarray(CTRL, dtype=np.float32).reshape(NWORLD, nu)))
    warp.copy(data.act, warp.array(np.asarray(ACT, dtype=np.float32).reshape(NWORLD, na)))
    mujoco_warp.transmission(model, data)
    mujoco_warp.fwd_velocity(model, data)
    mujoco_warp.fwd_actuation(model, data)

    moment_width = int(np.asarray(data.moment_colind.numpy()).shape[1])
    fixture = {
        "task_id": "2026-10-11-roadmap-g05-p7",
        "matrix": "G05",
        "phase": "P7",
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
            **base["model_arrays"],
            "jnt_type": [int(value) for value in mjm.jnt_type],
            "jnt_qposadr": [int(value) for value in mjm.jnt_qposadr],
            "jnt_dofadr": [int(value) for value in mjm.jnt_dofadr],
            "actuator_trntype": [int(value) for value in mjm.actuator_trntype],
            "actuator_trnid": [int(value) for value in np.asarray(mjm.actuator_trnid).reshape(-1)],
            "actuator_gear": f32_list(mjm.actuator_gear),
        },
        "nworld": NWORLD,
        "initial": {
            "qpos": f32_list(QPOS),
            "qvel": f32_list(QVEL),
            "ctrl": f32_list(CTRL),
            "act": f32_list(ACT),
        },
        "transmission_reference": {
            "actuator_length": flat(data.actuator_length.numpy()),
            "actuator_velocity": flat(data.actuator_velocity.numpy()),
            "moment_width": moment_width,
            "moment_rownnz": [int(value) for value in np.asarray(data.moment_rownnz.numpy()).reshape(-1)],
            "moment_rowadr": [int(value) for value in np.asarray(data.moment_rowadr.numpy()).reshape(-1)],
            "moment_colind": [int(value) for value in np.asarray(data.moment_colind.numpy()).reshape(-1)],
            "actuator_moment": flat(data.actuator_moment.numpy()),
        },
        "expected_actuation": {
            "act_dot": flat(data.act_dot.numpy()),
            "actuator_force": flat(data.actuator_force.numpy()),
            "qfrc_actuator": flat(data.qfrc_actuator.numpy()),
        },
    }

    # 原生一级判定：逐世界 f64 阶段链，与上游 f32 结果比较。
    native_max = 0.0
    for world in range(NWORLD):
        mjd = mujoco.MjData(mjm)
        mjd.qpos[:] = np.asarray(QPOS[world], dtype=np.float64)
        mjd.qvel[:] = np.asarray(QVEL[world], dtype=np.float64)
        mjd.ctrl[:] = np.asarray(CTRL[world], dtype=np.float64)
        mjd.act[:] = np.asarray(ACT[world], dtype=np.float64)
        mujoco.mj_fwdPosition(mjm, mjd)
        mujoco.mj_fwdVelocity(mjm, mjd)
        mujoco.mj_fwdActuation(mjm, mjd)
        for field, native, width in [
            ("actuator_length", mjd.actuator_length, nact),
            ("actuator_velocity", mjd.actuator_velocity, nact),
        ]:
            product = np.asarray(
                fixture["transmission_reference"][field][world * width : (world + 1) * width],
                dtype=np.float64,
            )
            native = np.asarray(native, dtype=np.float64)
            native_max = max(native_max, float(np.max(np.abs(product - native))))
        for field, native, width in [
            ("act_dot", mjd.act_dot, na),
            ("actuator_force", mjd.actuator_force, nact),
            ("qfrc_actuator", mjd.qfrc_actuator, nv),
        ]:
            product = np.asarray(
                fixture["expected_actuation"][field][world * width : (world + 1) * width],
                dtype=np.float64,
            )
            native = np.asarray(native, dtype=np.float64)
            native_max = max(native_max, float(np.max(np.abs(product - native))))

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

    fixture["upstream_reference"] = {
        "revision": PINNED_REVISION,
        "warp": warp.__version__,
        "mujoco_warp": "3.15.0",
        "mujoco": mujoco.__version__,
        "functions": ["mujoco_warp.transmission", "mujoco_warp.fwd_velocity", "mujoco_warp.fwd_actuation"],
    }
    fixture["native_cross_check"] = {
        "max_abs_vs_native": native_max,
        "note": "原生 mj_fwdPosition/mj_fwdVelocity/mj_fwdActuation（f64）先做一级判定；上游 MJWarp f32 为二级参考。",
    }
    fixture["environment"] = {"python": platform.python_version(), "gpu": gpu}
    fixture["tolerance"] = {
        "upstream_abs": 0.0,
        "native_abs": 1e-05,
        "note": "transmission 只做乘/拷贝，产品与上游逐位一致（容差 0）；原生 f64 一级判定容许 1e-5。",
    }

    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g05-transmission.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nworld": NWORLD,
        "nactuator": nact,
        "moment_width": moment_width,
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/actuation/transmission-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g05-p7 transmission authoring OK:"
        f" nact={nact} nworld={NWORLD} moment_width={moment_width}"
        f" native_max_abs={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
