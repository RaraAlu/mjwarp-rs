"""M10@P7 离线 authoring：冻结上游 `types.Callback` 字段与 `types.Stage` 数值。

离线专用：本脚本只被 scripts/authoring/author-m10-callback.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/callback/contract.json
  docs/development-evidence/backlog-batch3/2026-10-11-roadmap-m10-p7/authoring.json
"""

from __future__ import annotations

import dataclasses
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import mujoco
import warp as wp

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_JSON = ROOT / "fixtures" / "callback" / "contract.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch3"
    / "2026-10-11-roadmap-m10-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-m10-p7"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    from mujoco_warp._src import types

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=UPSTREAM,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    fields = dataclasses.fields(types.Callback)
    callback_fields = [field.name for field in fields]
    callback_defaults = {field.name: field.default for field in fields}
    if callback_fields != [
        "passive",
        "control",
        "act_dyn",
        "act_gain",
        "act_bias",
        "sensor",
        "contactfilter",
    ]:
        raise SystemExit(f"unexpected Callback fields: {callback_fields}")
    if any(value is not None for value in callback_defaults.values()):
        raise SystemExit(f"unexpected Callback defaults: {callback_defaults}")

    sensor_stage_values = {
        "POS": int(types.Stage.POS),
        "VEL": int(types.Stage.VEL),
        "ACC": int(types.Stage.ACC),
    }
    stage_enum_values = {
        "POS": int(mujoco.mjtStage.mjSTAGE_POS),
        "VEL": int(mujoco.mjtStage.mjSTAGE_VEL),
        "ACC": int(mujoco.mjtStage.mjSTAGE_ACC),
    }
    if sensor_stage_values != stage_enum_values:
        raise SystemExit(
            f"Stage values differ from mjtStage: {sensor_stage_values} vs {stage_enum_values}"
        )

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "M10",
        "phase": "P7",
        "upstream_functions": [
            "types.Callback",
            "types.Stage",
            "forward.forward",
            "passive.passive",
            "sensor.sensor_pos",
            "sensor.sensor_vel",
            "sensor.sensor_acc",
            "collision_driver",
        ],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": (
                "冻结上游 dataclasses.fields(types.Callback) 的槽位顺序/默认值与 "
                "types.Stage POS/VEL/ACC 数值（并与 MuJoCo mjtStage 交叉核对）；"
                "调用点顺序由证据 README 的 file:line 映射人工核对，不做数值对照。"
            ),
        },
        "callback_fields": callback_fields,
        "callback_defaults": callback_defaults,
        "sensor_stage_values": sensor_stage_values,
        "stage_enum_values": stage_enum_values,
        "environment": {
            "python": platform.python_version(),
            "gpu": gpu,
        },
    }
    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-m10-callback.ps1",
        "callback_fields": callback_fields,
        "callback_defaults_none": True,
        "sensor_stage_values": sensor_stage_values,
        "stage_enum_values": stage_enum_values,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/callback/contract.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        f"m10-p7 callback authoring OK: fields={len(callback_fields)} "
        f"sensor_stages={sensor_stage_values}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
