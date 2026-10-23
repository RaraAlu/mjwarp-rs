"""G24@P7 离线 authoring：上游历史环形缓冲冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g24-history.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/history/cases.json
  docs/development-evidence/backlog-batch3/2026-10-11-roadmap-g24-p7/authoring.json
"""

from __future__ import annotations

import json
import platform
import subprocess
import sys

import mujoco
import numpy as np
import warp as wp

import mujoco_warp as mjw
from mujoco_warp._src.types import CtrlChart, GainType

from g24_history_authoring import (
    EVIDENCE_DIR,
    EVIDENCE_JSON,
    FIXTURE_JSON,
    MODEL_XML,
    NATIVE_TOLERANCE,
    PINNED_REVISION,
    ROOT,
    TASK_ID,
    UPSTREAM,
    build_scenarios,
    f32,
    f32_list,
    f32_matrix,
    model_buffers,
    real_reset,
    region_elements,
    sha256,
    synthetic_actuator_reset,
)

def main() -> int:
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=UPSTREAM,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    model = mjw.put_model(mjm)
    nworld = 2
    data = mjw.make_data(mjm, nworld=nworld)
    wp.synchronize_device()
    nhistory = int(model.nhistory)
    timestep = f32(np.asarray(model.opt.timestep.numpy())[0])
    initial = np.asarray(data.history.numpy()).astype(np.float32).copy()
    if initial.shape != (nworld, nhistory):
        raise SystemExit(f"unexpected history shape {initial.shape}")

    actuators, sensors = model_buffers(mjm, model)

    mjd = mujoco.MjData(mjm)
    native_words = np.asarray(mjd.history, dtype=np.float64).ravel()
    native_abs = float(np.max(np.abs(initial[0].astype(np.float64) - native_words)))
    if native_abs > NATIVE_TOLERANCE:
        raise SystemExit(f"native C history mismatch max abs {native_abs:.3e}")

    dirty = np.full((nworld, nhistory), 7.5, dtype=np.float32)
    reset_cases = []
    for name, mask, dirty_flag in (
        ("initial", None, False),
        ("all_reset", None, True),
        ("local_mask_first", [True, False], True),
        ("local_mask_second", [False, True], True),
    ):
        words = real_reset(model, nworld, dirty if dirty_flag else initial, mask)
        if name == "all_reset":
            all_abs = float(
                np.max(np.abs(words[0].astype(np.float64) - native_words))
            )
            if all_abs > NATIVE_TOLERANCE:
                raise SystemExit(f"all_reset vs native max abs {all_abs:.3e}")
        reset_cases.append(
            {
                "name": name,
                "dirty": dirty_flag,
                "mask": mask,
                "expected_words": f32_matrix(words),
            }
        )

    so3_samples, so3_channels = 3, 4
    so3_initial = np.zeros(
        (1, region_elements(so3_samples, so3_channels)), dtype=np.float32
    )
    so3_expected = synthetic_actuator_reset(
        1,
        so3_samples,
        so3_channels,
        timestep,
        gaintype=int(GainType.SO3),
        ctrlspec=int(CtrlChart.QUAT),
    )

    scenarios = build_scenarios(model, initial, actuators, sensors)

    buffers = [
        {"kind": "actuator", **record} for record in actuators
    ] + [{"kind": "sensor", **record} for record in sensors]

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G24",
        "phase": "P7",
        "upstream_functions": [
            "history._history_physical_index",
            "history._history_find_index",
            "history._history_insert_scalar",
            "history._history_insert_vector",
            "history._history_read_scalar",
            "history._history_read_vector",
            "history._reset_actuator_history_kernel",
            "history._reset_sensor_history_kernel",
        ],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": (
                "冻结上游 history 的 wp.func 自定义内核回放（插入/读取）与两个重置内核输出；"
                "模型 nworld=2，真实缓冲布局取自 MJWarp put_model；"
                "原生 C mj_resetData 的 d.history（f64）作为独立参考。"
            ),
        },
        "model": {
            "path": MODEL_XML.relative_to(ROOT).as_posix(),
            "sha256": sha256(MODEL_XML),
        },
        "layout": {
            "nhistory": nhistory,
            "nworld": nworld,
            "timestep": timestep,
            "actuators": actuators,
            "sensors": sensors,
        },
        "reset": {
            "nhistory": nhistory,
            "nworld": nworld,
            "timestep": timestep,
            "initial_words": f32_matrix(initial),
            "dirty_words": f32_list(dirty),
            "native_words": [float(value) for value in native_words],
            "native_abs": native_abs,
            "buffers": buffers,
            "cases": reset_cases,
            "synthetic_actuator": {
                "samples": so3_samples,
                "channels": so3_channels,
                "timestep": timestep,
                "so3_quat": True,
                "initial_words": f32_matrix(so3_initial),
                "expected_words": f32_matrix(so3_expected),
            },
        },
        "mechanics": {
            "scenarios": [scenario.export() for scenario in scenarios],
        },
        "environment": {
            "python": platform.python_version(),
            "gpu": gpu,
        },
        "tolerance": {
            "upstream_words": "逐位一致",
            "upstream_reads_abs": 1e-6,
            "native_abs": NATIVE_TOLERANCE,
            "note": (
                "缓冲字按 f32 逐位对照；读取值允许 FMA 末位差（1e-6 冻结容差）；"
                "原生 C f64 参考只做一级判定。"
            ),
        },
    }
    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g24-history.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nhistory": nhistory,
        "nworld": nworld,
        "reset_cases": [case["name"] for case in reset_cases],
        "synthetic_so3_quat": True,
        "scenario_count": len(scenarios),
        "read_count": sum(len(scenario.reads) for scenario in scenarios),
        "native_max_abs": native_abs,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/history/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g24-p7 history authoring OK: "
        f"nhistory={nhistory} reset_cases={len(reset_cases)} "
        f"scenarios={len(scenarios)} native_max={native_abs:.3e}"
    )
    return 0

if __name__ == "__main__":
    sys.exit(main())
