"""M08@P7 离线 authoring：局部重置 P7 字段扩展的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-m08-extended-reset.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/local-reset/extended-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-m08-p7/authoring.json
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
MODEL_XML = ROOT / "fixtures" / "local-reset" / "extended-model.xml"
FIXTURE_JSON = ROOT / "fixtures" / "local-reset" / "extended-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-m08-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
NWORLD = 2
FIELDS = [
    "time",
    "qpos",
    "qvel",
    "ctrl",
    "act",
    "qfrc_applied",
    "xfrc_applied",
    "mocap_pos",
    "mocap_quat",
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    flat = np.asarray(values, dtype=np.float64).reshape(-1)
    return [float(np.float32(value)) for value in flat]


def flat(array) -> list[float]:
    return f32_list(np.asarray(array).reshape(-1))


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    dims = {
        "nbody": int(mjm.nbody),
        "nq": int(mjm.nq),
        "nv": int(mjm.nv),
        "nu": int(mjm.nu),
        "na": int(mjm.na),
        "nmocap": int(mjm.nmocap),
        "nkey": int(mjm.nkey),
    }
    if dims != {"nbody": 4, "nq": 2, "nv": 2, "nu": 1, "na": 1, "nmocap": 1, "nkey": 2}:
        raise SystemExit(f"unexpected model dims {dims}")
    nworld = NWORLD
    model = mujoco_warp.put_model(mjm)
    initial = {
        "time": f32_list([0.5, 0.9]),
        "qpos": f32_list([[1.0, 2.0], [3.0, 4.0]]),
        "qvel": f32_list([[0.1, 0.2], [0.3, 0.4]]),
        "ctrl": f32_list([[0.6], [-0.9]]),
        "act": f32_list([[0.4], [-0.2]]),
        "qfrc_applied": f32_list([[0.5, -0.3], [0.7, 0.1]]),
        "xfrc_applied": f32_list(np.arange(float(nworld * dims["nbody"] * 6)).reshape(-1) * 0.01),
        "mocap_pos": f32_list([[0.2, 0.1, -0.3], [-0.4, 0.5, 0.6]]),
        "mocap_quat": f32_list([[0.9, 0.1, 0.2, 0.3], [0.5, -0.5, 0.5, 0.5]]),
    }

    def fresh_data():
        data = mujoco_warp.make_data(mjm, nworld=nworld)

        def copy_into(field, values, shape):
            array = warp.array(
                np.asarray(values, dtype=np.float32).reshape(shape),
                dtype=getattr(data, field).dtype,
            )
            warp.copy(getattr(data, field), array)

        copy_into("time", initial["time"], (nworld,))
        copy_into("qpos", initial["qpos"], (nworld, dims["nq"]))
        copy_into("qvel", initial["qvel"], (nworld, dims["nv"]))
        copy_into("ctrl", initial["ctrl"], (nworld, dims["nu"]))
        copy_into("act", initial["act"], (nworld, dims["na"]))
        copy_into("qfrc_applied", initial["qfrc_applied"], (nworld, dims["nv"]))
        copy_into("xfrc_applied", initial["xfrc_applied"], (nworld, dims["nbody"], 6))
        copy_into("mocap_pos", initial["mocap_pos"], (nworld, -1, 3))
        copy_into("mocap_quat", initial["mocap_quat"], (nworld, -1, 4))
        return data

    def snapshot(data):
        return {
            "time": flat(data.time.numpy()),
            "qpos": flat(data.qpos.numpy()),
            "qvel": flat(data.qvel.numpy()),
            "ctrl": flat(data.ctrl.numpy()),
            "act": flat(data.act.numpy()),
            "qfrc_applied": flat(data.qfrc_applied.numpy()),
            "xfrc_applied": flat(data.xfrc_applied.numpy()),
            "mocap_pos": flat(data.mocap_pos.numpy()),
            "mocap_quat": flat(data.mocap_quat.numpy()),
        }

    cases = []

    def run_mask(case_id: str, mask) -> None:
        data = fresh_data()
        if mask is None:
            mujoco_warp.reset_data(model, data)
        elif all(isinstance(item, bool) for item in mask):
            mujoco_warp.reset_data(model, data, reset=warp.array(np.asarray(mask, dtype=bool)))
        else:
            mujoco_warp.reset_data(model, data, reset=warp.array(np.asarray(mask, dtype=np.int32)))
        cases.append(
            {
                "id": case_id,
                "operation": "reset_data",
                "mask": mask,
                "expected": snapshot(data),
            }
        )

    run_mask("mask_none", None)
    run_mask("mask_bool", [True, False])
    run_mask("mask_flags", [0, 1])

    def run_keyframe(case_id: str, key) -> None:
        data = fresh_data()
        if isinstance(key, list):
            mujoco_warp.reset_data_keyframe(model, data, warp.array(np.asarray(key, dtype=np.int32)))
        else:
            mujoco_warp.reset_data_keyframe(model, data, key)
        cases.append(
            {
                "id": case_id,
                "operation": "reset_data_keyframe",
                "key": key,
                "expected": snapshot(data),
            }
        )

    run_keyframe("keyframe_all_1", 1)
    run_keyframe("keyframe_worlds_mixed", [0, -1])
    run_keyframe("keyframe_worlds_last", [1, 1])

    mjd = mujoco.MjData(mjm)

    def native_snapshot():
        return {
            "time": f32_list(np.repeat(mjd.time, nworld)),
            "qpos": f32_list(np.tile(np.asarray(mjd.qpos), nworld)),
            "qvel": f32_list(np.tile(np.asarray(mjd.qvel), nworld)),
            "ctrl": f32_list(np.tile(np.asarray(mjd.ctrl), nworld)),
            "act": f32_list(np.tile(np.asarray(mjd.act), nworld)),
            "qfrc_applied": f32_list(np.tile(np.asarray(mjd.qfrc_applied), nworld)),
            "xfrc_applied": f32_list(np.tile(np.asarray(mjd.xfrc_applied).reshape(-1), nworld)),
            "mocap_pos": f32_list(np.tile(np.asarray(mjd.mocap_pos).reshape(-1), nworld)),
            "mocap_quat": f32_list(np.tile(np.asarray(mjd.mocap_quat).reshape(-1), nworld)),
        }

    native_max = 0.0
    for case in cases:
        mask = case.get("mask")
        if case["operation"] == "reset_data":
            mujoco.mj_resetData(mjm, mjd)
            native = native_snapshot()
            worlds = [world for world in range(nworld) if mask is None or bool(mask[world])]
        else:
            key = case["key"]
            if isinstance(key, list):
                if key[0] < 0:
                    continue
                mujoco.mj_resetDataKeyframe(mjm, mjd, key[0])
                native = native_snapshot()
                worlds = [world for world in range(nworld) if key[world] >= 0]
            else:
                mujoco.mj_resetDataKeyframe(mjm, mjd, key)
                native = native_snapshot()
                worlds = list(range(nworld))
        expected = case["expected"]
        for world in worlds:
            for field in FIELDS:
                actual_all = expected[field]
                native_all = native[field]
                expected_world = actual_all[
                    world * (len(actual_all) // nworld) : (world + 1) * (len(actual_all) // nworld)
                ]
                native_world = native_all[
                    world * (len(native_all) // nworld) : (world + 1) * (len(native_all) // nworld)
                ]
                for left, right in zip(expected_world, native_world):
                    native_max = max(native_max, abs(left - right))

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
        "task_id": "2026-10-11-roadmap-m08-p7",
        "matrix": "M08",
        "phase": "P7",
        "upstream_functions": ["reset_data", "reset_data_keyframe"],
        "model": {
            "path": "fixtures/local-reset/extended-model.xml",
            "sha256": sha256(MODEL_XML),
            **dims,
        },
        "model_arrays": {
            "body_mocapid": [int(value) for value in mjm.body_mocapid],
            "body_pos": f32_list(np.asarray(mjm.body_pos).reshape(-1)),
            "body_quat": f32_list(np.asarray(mjm.body_quat).reshape(-1)),
            "qpos0": f32_list(mjm.qpos0),
            "key_time": f32_list(mjm.key_time),
            "key_qpos": f32_list(np.asarray(mjm.key_qpos).reshape(-1)),
            "key_qvel": f32_list(np.asarray(mjm.key_qvel).reshape(-1)),
            "key_act": f32_list(np.asarray(mjm.key_act).reshape(-1)),
            "key_ctrl": f32_list(np.asarray(mjm.key_ctrl).reshape(-1)),
            "key_mpos": f32_list(np.asarray(mjm.key_mpos).reshape(-1)),
            "key_mquat": f32_list(np.asarray(mjm.key_mquat).reshape(-1)),
        },
        "nworld": nworld,
        "initial": initial,
        "cases": cases,
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "functions": ["mujoco_warp.reset_data", "mujoco_warp.reset_data_keyframe"],
        },
        "native_cross_check": {
            "max_abs_vs_native_mj_resetData": native_max,
            "note": "原生 mj_resetData/mj_resetDataKeyframe（f64）先做一级判定；上游 MJWarp f32 输出为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 0.0,
            "native_abs": 1e-05,
            "note": "重置只做拷贝/置零/mocap 复位，产品与上游逐位一致（容差 0）；原生 f64 一级判定容许 1e-5。",
        },
    }
    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-m08-extended-reset.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "dims": dims,
        "nworld": nworld,
        "cases": [case["id"] for case in cases],
        "fields": FIELDS,
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/local-reset/extended-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "m08-p7 extended reset authoring OK:"
        f" cases={len(cases)} fields={len(FIELDS)}"
        f" native_max_abs={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
