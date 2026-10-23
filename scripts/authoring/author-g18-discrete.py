"""G18@P6 离线 authoring：DISCRETE 积分器单步推进的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g18-discrete.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/discrete/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g18-p6/authoring.json
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

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "discrete"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch2"
    / "2026-10-11-roadmap-g18-p6"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g18-p6"
CASES = [
    ("default", [1.0, 0.0, 0.0, 0.0, 0.4, 0.1], [0.5, -0.4, 0.2, 0.7, -0.3]),
    ("tilted", [0.9659258, 0.0, 0.258819, 0.0, -0.9, -0.2], [-1.2, 0.6, -0.5, -0.8, 0.9]),
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def main() -> int:
    import mujoco_warp as mjw

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    model = mjw.put_model(mjm)
    cases = []
    native_max = 0.0
    for name, qpos, qvel in CASES:
        data = mjw.make_data(mjm, nworld=1)
        data.qpos.assign(np.asarray([qpos], dtype=np.float32))
        data.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.forward(model, data)
        wp.synchronize_device()
        qacc = np.asarray(data.qacc.numpy())[0].copy()
        mjw.discrete(model, data)
        wp.synchronize_device()
        expected_qpos = np.asarray(data.qpos.numpy())[0].copy()
        expected_qvel = np.asarray(data.qvel.numpy())[0].copy()
        expected_time = np.asarray(data.time.numpy())[0]

        mjd = mujoco.MjData(mjm)
        mjd.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mjd.qvel[:] = np.asarray(qvel, dtype=np.float64)
        mujoco.mj_forward(mjm, mjd)
        native_qacc = np.asarray(mjd.qacc).copy()
        mjd.qvel[:] = np.asarray(qvel, dtype=np.float64) + float(mjm.opt.timestep) * native_qacc
        mujoco.mj_integratePos(mjm, mjd.qpos, mjd.qvel, float(mjm.opt.timestep))
        native_qpos = np.asarray(mjd.qpos).copy()
        native_qvel = np.asarray(mjd.qvel).copy()
        native_max = max(
            native_max,
            float(np.max(np.abs(expected_qpos.astype(np.float64) - native_qpos))),
            float(np.max(np.abs(expected_qvel.astype(np.float64) - native_qvel))),
        )
        cases.append(
            {
                "name": name,
                "qpos": f32_list(qpos),
                "qvel": f32_list(qvel),
                "qacc": f32_list(qacc),
                "discrete_qpos": f32_list(expected_qpos),
                "discrete_qvel": f32_list(expected_qvel),
                "discrete_time": float(np.float32(expected_time)),
                "native_qpos": f32_list(native_qpos),
                "native_qvel": f32_list(native_qvel),
            }
        )
        print(f"case {name}: native_max={native_max:.3e}")

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G18",
        "phase": "P6",
        "upstream_functions": ["forward", "discrete"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "冻结上游 forward 产生的 qacc 与 discrete（_advance(m,d,qacc)）单步结果；模型 integrator=DISCRETE，无自由关节（自由体 gyro 修正不在本片）。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "joints": {
            "nq": int(mjm.nq),
            "nv": int(mjm.nv),
            "njnt": int(mjm.njnt),
            "jnt_type": [int(value) for value in mjm.jnt_type],
            "jnt_qposadr": [int(value) for value in mjm.jnt_qposadr],
            "jnt_dofadr": [int(value) for value in mjm.jnt_dofadr],
            "timestep": f32_list([mjm.opt.timestep])[0],
            "integrator": int(mjm.opt.integrator),
        },
        "cases": cases,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 1e-06,
            "native_abs": 1e-05,
            "note": "产品与上游逐位对照优先（FMA 收缩末位差按 1e-6）；原生 f64 一级判定容差 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g18-discrete.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "cases": len(cases),
        "joints": {"nq": fixture["joints"]["nq"], "nv": fixture["joints"]["nv"]},
        "native_cross_check": {"max_abs": native_max},
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/discrete/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"g18-p6 discrete authoring OK: cases={len(cases)} native_max={native_max:.3e}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
