"""G27@P7 离线 authoring：qpos0 依赖常量（set_const_0 子集）的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g27-qpos-constants.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/set-const/qpos-dependent.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g27-p7/authoring.json
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
MODEL_XML = ROOT / "fixtures" / "set-const" / "qpos-model.xml"
FIXTURE_JSON = ROOT / "fixtures" / "set-const" / "qpos-dependent.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g27-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
CASES = [
    ("default", None),
    ("modified", [0.9659258, 0.0, 0.0, 0.258819, 0.1]),
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in values]


def flat(array) -> list[float]:
    return f32_list(np.asarray(array).reshape(-1))


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    dims = [int(mjm.nbody), int(mjm.nq), int(mjm.nv), int(mjm.ncam), int(mjm.nlight), int(mjm.ntendon)]
    if dims != [3, 5, 4, 2, 2, 1]:
        raise SystemExit(f"unexpected model dims {dims}")
    nbody, nq, nv, ncam, nlight, ntendon = dims

    def upstream_case(qpos0):
        native_model = mujoco.MjModel.from_xml_path(str(MODEL_XML))
        if qpos0 is not None:
            native_model.qpos0[:] = np.asarray(qpos0, dtype=np.float64)
            native_data = mujoco.MjData(native_model)
            mujoco.mj_setConst(native_model, native_data)
        m = mujoco_warp.put_model(native_model)
        d = mujoco_warp.make_data(native_model, nworld=1)
        mujoco_warp.set_const_0(m, d, restore=False)
        inputs = {
            "ten_length": flat(d.ten_length.numpy()),
            "cam_xpos": flat(d.cam_xpos.numpy()),
            "cam_xmat": flat(d.cam_xmat.numpy()),
            "light_xpos": flat(d.light_xpos.numpy()),
            "light_xdir": flat(d.light_xdir.numpy()),
            "xpos": flat(d.xpos.numpy()),
            "subtree_com": flat(d.subtree_com.numpy()),
            "m_rownnz": [int(value) for value in np.asarray(m.M_rownnz.numpy()).reshape(-1)],
            "m_rowadr": [int(value) for value in np.asarray(m.M_rowadr.numpy()).reshape(-1)],
            "m_values": flat(d.M.numpy()),
        }
        expected = {
            "tendon_length0": flat(m.tendon_length0.numpy()),
            "cam_pos0": flat(m.cam_pos0.numpy()),
            "cam_poscom0": flat(m.cam_poscom0.numpy()),
            "cam_mat0": flat(m.cam_mat0.numpy()),
            "light_pos0": flat(m.light_pos0.numpy()),
            "light_poscom0": flat(m.light_poscom0.numpy()),
            "light_dir0": flat(m.light_dir0.numpy()),
            "meaninertia": [float(np.asarray(m.stat.meaninertia.numpy()).reshape(-1)[0])],
        }
        native = {
            "tendon_length0": f32_list(native_model.tendon_length0),
            "cam_pos0": f32_list(np.asarray(native_model.cam_pos0).reshape(-1)),
            "cam_poscom0": f32_list(np.asarray(native_model.cam_poscom0).reshape(-1)),
            "cam_mat0": f32_list(np.asarray(native_model.cam_mat0).reshape(-1)),
            "light_pos0": f32_list(np.asarray(native_model.light_pos0).reshape(-1)),
            "light_poscom0": f32_list(np.asarray(native_model.light_poscom0).reshape(-1)),
            "light_dir0": f32_list(np.asarray(native_model.light_dir0).reshape(-1)),
            "meaninertia": [float(np.float32(native_model.stat.meaninertia))],
        }
        return inputs, expected, native

    cases = []
    native_max = 0.0
    for case_id, qpos0 in CASES:
        inputs, expected, native = upstream_case(qpos0)
        model_arrays = {
            "cam_bodyid": [int(value) for value in mjm.cam_bodyid],
            "cam_targetbodyid": [int(value) for value in mjm.cam_targetbodyid],
            "light_bodyid": [int(value) for value in mjm.light_bodyid],
            "light_targetbodyid": [int(value) for value in mjm.light_targetbodyid],
        }
        for field, values in expected.items():
            target = native[field]
            if len(values) != len(target):
                raise SystemExit(f"{case_id} {field} length mismatch")
            for left, right in zip(values, target):
                native_max = max(native_max, abs(left - right))
        cases.append(
            {
                "id": case_id,
                "qpos0": None if qpos0 is None else f32_list(qpos0),
                "model_arrays": model_arrays,
                "inputs": inputs,
                "expected": expected,
            }
        )

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
        "task_id": "2026-10-11-roadmap-g27-p7",
        "matrix": "G27",
        "phase": "P7",
        "upstream_function": "set_const_0",
        "model": {
            "path": "fixtures/set-const/qpos-model.xml",
            "sha256": sha256(MODEL_XML),
            "nbody": nbody,
            "nq": nq,
            "nv": nv,
            "ncam": ncam,
            "nlight": nlight,
            "ntendon": ntendon,
        },
        "nworld": 1,
        "cases": cases,
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "function": "mujoco_warp.set_const_0",
        },
        "native_cross_check": {
            "max_abs_vs_native_mj_setConst": native_max,
            "note": "原生 mj_setConst（f64）先做一级判定；上游 MJWarp f32 常量为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 0.0,
            "native_abs": 1e-05,
            "note": "常量只做拷贝/减法/对角线均值，产品与上游逐位一致（容差 0）；原生 f64 一级判定容许 1e-5。",
        },
    }

    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g27-qpos-constants.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "cases": [case["id"] for case in cases],
        "case_qpos0": {case["id"]: case["qpos0"] for case in cases},
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/set-const/qpos-dependent.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(
        "g27-p7 qpos-constants authoring OK:"
        f" cases={len(cases)}"
        f" native_max_abs={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
