"""G20@P7 离线 authoring：逆向动力学 qfrc_inverse 的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g20-inverse.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/inverse/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g20-p7/authoring.json
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
FIXTURE_DIR = ROOT / "fixtures" / "inverse"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT / "docs" / "development-evidence" / "backlog-batch2"
    / "2026-10-11-roadmap-g20-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g20-p7"

QPOS_CASES = [
    ("default", [0.4, -0.6, 0.1], [0.5, -0.3, 0.2]),
    ("opposite", [-0.9, 0.7, -0.25], [-1.1, 0.8, -0.4]),
    ("large", [1.2, -1.4, 0.35], [2.0, -1.5, 0.9]),
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
    row_nnz = i32_list(model.M_rownnz.numpy())
    row_adr = i32_list(model.M_rowadr.numpy())
    col_ind = i32_list(model.M_colind.numpy())
    mulm_rowadr = i32_list(model.M_mulm_rowadr.numpy())
    mulm_col = i32_list(model.M_mulm_col.numpy())
    mulm_madr = i32_list(model.M_mulm_madr.numpy())

    cases = []
    native_max = 0.0
    forward_balance = 0.0
    for name, qpos, qvel in QPOS_CASES:
        data = mjw.make_data(mjm, nworld=1)
        data.qpos.assign(np.asarray([qpos], dtype=np.float32))
        data.qvel.assign(np.asarray([qvel], dtype=np.float32))
        mjw.forward(model, data)
        wp.synchronize_device()
        qacc = np.asarray(data.qacc.numpy())[0]
        mjw.inverse(model, data)
        wp.synchronize_device()
        inverse = np.asarray(data.qfrc_inverse.numpy())[0]
        bias = np.asarray(data.qfrc_bias.numpy())[0]
        passive = np.asarray(data.qfrc_passive.numpy())[0]
        constraint = np.asarray(data.qfrc_constraint.numpy())[0]
        mass = np.asarray(data.M.numpy())[0]

        mjd = mujoco.MjData(mjm)
        mjd.qpos[:] = np.asarray(qpos, dtype=np.float64)
        mjd.qvel[:] = np.asarray(qvel, dtype=np.float64)
        mujoco.mj_forward(mjm, mjd)
        mujoco.mj_inverse(mjm, mjd)
        native_inverse = np.asarray(mjd.qfrc_inverse).copy()
        native_max = max(native_max, float(np.max(np.abs(np.asarray(inverse, dtype=np.float64) - native_inverse))))
        forward_balance = max(forward_balance, float(np.max(np.abs(native_inverse))))

        cases.append(
            {
                "name": name,
                "qpos": f32_list(qpos),
                "qvel": f32_list(qvel),
                "qacc": f32_list(qacc),
                "qfrc_bias": f32_list(bias),
                "qfrc_passive": f32_list(passive),
                "qfrc_constraint": f32_list(constraint),
                "mass": f32_list(mass),
                "qfrc_inverse": f32_list(inverse),
                "native_qfrc_inverse": f32_list(native_inverse),
                "native_forward_balance": float(np.max(np.abs(native_inverse))),
            }
        )
        print(
            f"case {name}: inverse_max={float(np.max(np.abs(inverse))):.3e}"
            f" native_max={float(np.max(np.abs(native_inverse))):.3e}"
        )

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G20",
        "phase": "P7",
        "upstream_functions": ["forward", "inverse"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "上游 forward 产生 qacc（正向动力学），随后 inverse 计算 qfrc_inverse = qfrc_bias + M@qacc - qfrc_passive - qfrc_constraint；M 为 CSR。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "mass_layout": {
            "nq": int(mjm.nq),
            "nv": int(mjm.nv),
            "nnz": len(col_ind),
            "row_nnz": row_nnz,
            "row_adr": row_adr,
            "col_ind": col_ind,
            "nnz_mulm": len(mulm_col),
            "mulm_rowadr": mulm_rowadr,
            "mulm_col": mulm_col,
            "mulm_madr": mulm_madr,
            "note": "M 值按 d.M（CSR 行）冻结；乘向量使用上游 M_mulm_rowadr/M_mulm_col/M_mulm_madr 收集布局。",
        },
        "cases": cases,
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 1e-05,
            "native_abs": 1e-05,
            "force_balance_abs": 1e-05,
            "note": "产品与上游 f32 容差 1e-5（M@qacc 的 FMA 收缩/求和顺序末位差）；原生 f64 一级判定与正逆向力平衡容差 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g20-inverse.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "cases": len(cases),
        "mass_layout": {
            "nv": fixture["mass_layout"]["nv"],
            "nnz": fixture["mass_layout"]["nnz"],
        },
        "native_cross_check": {
            "upstream_vs_native_max_abs": native_max,
            "native_forward_balance_max_abs": forward_balance,
            "note": "原生 MuJoCo 3.15 f64 mj_forward + mj_inverse：上游 f32 qfrc_inverse 与原生 f64 的 max abs；逆动力学回代力平衡 = 原生 qfrc_inverse 最大值。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/inverse/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g20-p7 inverse authoring OK:"
        f" cases={len(cases)} upstream_vs_native={native_max:.3e}"
        f" native_balance={forward_balance:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
