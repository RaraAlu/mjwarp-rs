"""G23@P7 离线 authoring：sensor_pos/sensor_vel 的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g23-sensor.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/sensor/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g23-p7/authoring.json
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
from mujoco_warp._src import sensor

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "sensor"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch2"
    / "2026-10-11-roadmap-g23-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g23-p7"
CASES = [
    ("default", [0.4, 0.9238795, 0.3826834, 0.0, 0.0, 0.15], [0.5, 0.3, -0.2, 0.1, -0.4]),
    ("rotated", [-0.9, 0.70710678, 0.0, 0.70710678, 0.0, -0.7], [-1.2, -0.5, 0.7, -0.3, 0.9]),
    ("cutoff", [1.2, 1.0, 0.0, 0.0, 0.0, 0.8], [2.0, -0.9, 0.4, 0.6, -1.1]),
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def i32_list(values) -> list[int]:
    return [int(value) for value in np.asarray(values).reshape(-1)]


def main() -> int:
    import mujoco_warp as mjw

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    model = mjw.put_model(mjm)
    layout = {
        "nq": int(mjm.nq),
        "nv": int(mjm.nv),
        "nsensor": int(mjm.nsensor),
        "nsensordata": int(mjm.nsensordata),
        "sensor_type": i32_list(mjm.sensor_type),
        "sensor_datatype": i32_list(mjm.sensor_datatype),
        "sensor_objtype": i32_list(mjm.sensor_objtype),
        "sensor_objid": i32_list(mjm.sensor_objid),
        "sensor_adr": i32_list(mjm.sensor_adr),
        "sensor_cutoff": f32_list(mjm.sensor_cutoff),
        "sensor_dim": i32_list(mjm.sensor_dim),
        "jnt_qposadr": i32_list(mjm.jnt_qposadr),
        "jnt_dofadr": i32_list(mjm.jnt_dofadr),
        "jnt_type": i32_list(mjm.jnt_type),
        "sensor_types": {
            "JOINTPOS": int(mujoco.mjtSensor.mjSENS_JOINTPOS),
            "JOINTVEL": int(mujoco.mjtSensor.mjSENS_JOINTVEL),
            "BALLQUAT": int(mujoco.mjtSensor.mjSENS_BALLQUAT),
            "CLOCK": int(mujoco.mjtSensor.mjSENS_CLOCK),
        },
        "data_types": {
            "REAL": int(mujoco.mjtDataType.mjDATATYPE_REAL),
            "POSITIVE": int(mujoco.mjtDataType.mjDATATYPE_POSITIVE),
        },
    }

    cases = []
    native_max = 0.0
    for name, qpos, qvel in CASES:
        data = mjw.make_data(mjm, nworld=1)
        data.qpos.assign(np.asarray([qpos], dtype=np.float32))
        data.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.kinematics(model, data)
        sensor.sensor_pos(model, data)
        sensor.sensor_vel(model, data)
        wp.synchronize_device()
        pos = np.asarray(data.sensordata.numpy())[0].copy()
        # 位置阶段先写、速度阶段再写；这里分开记录两阶段快照。
        data2 = mjw.make_data(mjm, nworld=1)
        data2.qpos.assign(np.asarray([qpos], dtype=np.float32))
        data2.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.kinematics(model, data2)
        sensor.sensor_pos(model, data2)
        wp.synchronize_device()
        pos_only = np.asarray(data2.sensordata.numpy())[0].copy()
        data3 = mjw.make_data(mjm, nworld=1)
        data3.qpos.assign(np.asarray([qpos], dtype=np.float32))
        data3.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.kinematics(model, data3)
        sensor.sensor_vel(model, data3)
        wp.synchronize_device()
        vel_only = np.asarray(data3.sensordata.numpy())[0].copy()

        mjd = mujoco.MjData(mjm)
        mjd.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mjd.qvel[:] = np.asarray(qvel, dtype=np.float64)
        mujoco.mj_kinematics(mjm, mjd)
        mujoco.mj_sensorPos(mjm, mjd)
        native_pos = np.asarray(mjd.sensordata).copy()
        mjd2 = mujoco.MjData(mjm)
        mjd2.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mjd2.qvel[:] = np.asarray(qvel, dtype=np.float64)
        mujoco.mj_kinematics(mjm, mjd2)
        mujoco.mj_sensorVel(mjm, mjd2)
        native_vel = np.asarray(mjd2.sensordata).copy()
        native_max = max(
            native_max,
            float(np.max(np.abs(pos_only.astype(np.float64) - native_pos))),
            float(np.max(np.abs(vel_only.astype(np.float64) - native_vel))),
        )
        cases.append(
            {
                "name": name,
                "qpos": f32_list(qpos),
                "qvel": f32_list(qvel),
                "time": 0.0,
                "sensordata_pos": f32_list(pos_only),
                "sensordata_vel": f32_list(vel_only),
                "combine_check": f32_list(pos),
                "native_sensordata_pos": f32_list(native_pos),
                "native_sensordata_vel": f32_list(native_vel),
            }
        )
        print(f"case {name}: pos={pos_only} vel={vel_only}")

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G23",
        "phase": "P7",
        "upstream_functions": ["sensor_pos", "sensor_vel"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "冻结上游 sensor_pos/sensor_vel 的 sensordata；本片只覆盖 jointpos/jointvel/ballquat/clock 四类。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "layout": layout,
        "cases": cases,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 1e-06,
            "native_abs": 1e-05,
            "note": "产品与上游 sensordata 容差 1e-6；原生 f64 mj_sensorPos/mj_sensorVel 一级判定容差 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g23-sensor.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "cases": len(cases),
        "layout": {
            "nsensor": layout["nsensor"],
            "nsensordata": layout["nsensordata"],
            "nq": layout["nq"],
            "nv": layout["nv"],
        },
        "native_cross_check": {
            "max_abs": native_max,
            "note": "原生 MuJoCo 3.15 f64 mj_sensorPos/mj_sensorVel 与上游 f32 sensordata 的 max abs。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/sensor/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"g23-p7 sensor authoring OK: cases={len(cases)} native_max={native_max:.3e}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
