"""M08@P3 离线 authoring：局部重置（世界掩码与关键帧）的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-m08-reset.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/local-reset/reset-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-m08-p3/authoring.json
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
FIXTURE_DIR = ROOT / "fixtures" / "local-reset"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "reset-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-m08-p3"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
NWORLD = 3
INITIAL_TIME = [0.5, 1.0, 2.0]
INITIAL_QPOS = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]
INITIAL_QVEL = [[0.1, 0.2], [0.3, 0.4], [0.5, 0.6]]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in values]


def f32_matrix(rows) -> list[list[float]]:
    return [f32_list(row) for row in rows]


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    nq, nv, nkey = int(mjm.nq), int(mjm.nv), int(mjm.nkey)
    if (nq, nv, nkey) != (2, 2, 3):
        raise SystemExit(f"unexpected model dims nq={nq} nv={nv} nkey={nkey}")
    mjd = mujoco.MjData(mjm)
    model = mujoco_warp.put_model(mjm)

    def fresh_data():
        data = mujoco_warp.make_data(mjm, nworld=NWORLD)
        warp.copy(data.time, warp.array(np.asarray(INITIAL_TIME, dtype=np.float32)))
        warp.copy(
            data.qpos, warp.array(np.asarray(INITIAL_QPOS, dtype=np.float32).reshape(NWORLD, nq))
        )
        warp.copy(
            data.qvel, warp.array(np.asarray(INITIAL_QVEL, dtype=np.float32).reshape(NWORLD, nv))
        )
        return data

    def snapshot(data):
        return {
            "time": f32_list(data.time.numpy()),
            "qpos": f32_matrix(data.qpos.numpy()),
            "qvel": f32_matrix(data.qvel.numpy()),
        }

    def native_reset_data():
        mujoco.mj_resetData(mjm, mjd)
        return {
            "time": [float(mjd.time)] * NWORLD,
            "qpos": [f32_list(mjd.qpos) for _ in range(NWORLD)],
            "qvel": [f32_list(mjd.qvel) for _ in range(NWORLD)],
        }

    def native_keyframe(key: int):
        mujoco.mj_resetDataKeyframe(mjm, mjd, key)
        return {
            "time": [float(mjd.time)] * NWORLD,
            "qpos": [f32_list(mjd.qpos) for _ in range(NWORLD)],
            "qvel": [f32_list(mjd.qvel) for _ in range(NWORLD)],
        }

    cases = []

    def run_mask(case_id: str, mask) -> dict:
        data = fresh_data()
        if mask is None:
            mujoco_warp.reset_data(model, data)
        elif isinstance(mask, list) and all(isinstance(item, bool) for item in mask):
            mujoco_warp.reset_data(model, data, reset=warp.array(np.asarray(mask, dtype=bool)))
        else:
            mujoco_warp.reset_data(
                model, data, reset=warp.array(np.asarray(mask, dtype=np.int32))
            )
        return {
            "id": case_id,
            "operation": "reset_data",
            "mask": mask,
            "expected": snapshot(data),
        }

    cases.append(run_mask("mask_none", None))
    cases.append(run_mask("mask_bool", [True, False, True]))
    cases.append(run_mask("mask_bool_all_false", [False, False, False]))
    cases.append(run_mask("mask_flags", [1, 0, 1]))

    def run_keyframe(case_id: str, key) -> dict:
        data = fresh_data()
        if isinstance(key, list):
            mujoco_warp.reset_data_keyframe(
                model, data, warp.array(np.asarray(key, dtype=np.int32))
            )
        else:
            mujoco_warp.reset_data_keyframe(model, data, key)
        return {
            "id": case_id,
            "operation": "reset_data_keyframe",
            "key": key,
            "expected": snapshot(data),
        }

    cases.append(run_keyframe("keyframe_all_1", 1))
    cases.append(run_keyframe("keyframe_worlds_mixed", [0, 2, -1]))
    cases.append(run_keyframe("keyframe_worlds_none", [-1, -1, -1]))

    native_max = 0.0
    for case in cases:
        expected = case["expected"]
        if case["operation"] == "reset_data":
            reference = native_reset_data()
        else:
            keys = case["key"]
            if isinstance(keys, list):
                valid = {value for value in keys if value >= 0}
                if len(valid) != 1:
                    continue
                key = next(iter(valid))
            else:
                key = keys
            reference = native_keyframe(key)
        for world in range(NWORLD):
            if isinstance(case.get("mask"), list) and not bool(case["mask"][world]):
                continue
            if case["operation"] == "reset_data_keyframe" and isinstance(case["key"], list):
                if case["key"][world] < 0:
                    continue
            for field in ("time", "qpos", "qvel"):
                actual = expected[field][world]
                target = reference[field][world]
                if isinstance(actual, list):
                    for left, right in zip(actual, target):
                        native_max = max(native_max, abs(left - right))
                else:
                    native_max = max(native_max, abs(actual - target))

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
        "task_id": "2026-10-11-roadmap-m08-p3",
        "matrix": "M08",
        "phase": "P3",
        "upstream_functions": ["reset_data", "reset_data_keyframe"],
        "model": {
            "path": "fixtures/local-reset/model.xml",
            "sha256": sha256(MODEL_XML),
            "nbody": int(mjm.nbody),
            "nq": nq,
            "nv": nv,
            "nkey": nkey,
        },
        "model_arrays": {
            "qpos0": f32_list(mjm.qpos0),
            "key_time": f32_list(mjm.key_time),
            "key_qpos": f32_matrix(mjm.key_qpos),
            "key_qvel": f32_matrix(mjm.key_qvel),
        },
        "nworld": NWORLD,
        "initial": {
            "time": f32_list(INITIAL_TIME),
            "qpos": f32_matrix(INITIAL_QPOS),
            "qvel": f32_matrix(INITIAL_QVEL),
        },
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "functions": ["reset_data", "reset_data_keyframe"],
        },
        "cases": cases,
        "native_cross_check": {
            "max_abs_vs_native_mj_resetData": native_max,
            "note": "原生 mj_resetData/mj_resetDataKeyframe 先做一级判定；上游 MJWarp 输出为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "abs": 0.0,
            "note": "重置只做拷贝/置零；期望与上游逐位一致，容差 0。",
        },
    }

    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-m08-reset.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nworld": NWORLD,
        "cases": [case["id"] for case in cases],
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/local-reset/reset-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(
        "m08 reset authoring OK:"
        f" nq={nq} nv={nv} nkey={nkey} nworld={NWORLD}"
        f" cases={len(cases)}"
        f" native_max_abs={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
