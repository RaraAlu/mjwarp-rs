"""G21@P7 离线 authoring：deriv_smooth_vel（速度导数）的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g21-deriv.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/deriv/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g21-p7/authoring.json
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
from mujoco_warp._src import derivative, passive

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "deriv"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT / "docs" / "development-evidence" / "backlog-batch2"
    / "2026-10-11-roadmap-g21-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g21-p7"
POLY = [[0.5, 0.2], [0.0, 0.0], [0.3, -0.1]]
EPS = 1.0e-3
POLY_CASES = [
    ("poly-default", [0.3, -0.5, 0.1], [0.8, -0.6, 0.4]),
    ("poly-negative", [-0.7, 0.9, -0.2], [-1.5, 0.4, -0.9]),
    ("poly-large", [1.1, -1.3, 0.3], [3.0, -2.2, 1.6]),
]
NATIVE_CASES = [
    ("linear-default", [0.3, -0.5, 0.1], [0.8, -0.6, 0.4]),
    ("linear-large", [-0.4, 0.6, -0.1], [2.4, -1.7, 1.1]),
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def i32_list(values) -> list[int]:
    return [int(value) for value in np.asarray(values).reshape(-1)]


def put_model(imports, with_poly: bool):
    mjm = imports["mujoco"].MjModel.from_xml_path(str(MODEL_XML))
    model = imports["mjw"].put_model(mjm)
    if with_poly:
        poly = np.asarray(POLY, dtype=np.float32).reshape(1, mjm.nv, 2)
        model.dof_dampingpoly = wp.array(poly, dtype=wp.vec2)
    return mjm, model


def derive(imports, mjm, model, qpos, qvel):
    mjw = imports["mjw"]
    data = mjw.make_data(mjm, nworld=1)
    data.qpos.assign(np.asarray([qpos], dtype=np.float32))
    data.qvel.assign(np.asarray([qvel], dtype=np.float32))
    mjw.forward(model, data)
    wp.synchronize_device()
    out = wp.zeros((1, int(model.nC)), dtype=float)
    derivative.deriv_smooth_vel(model, data, out)
    wp.synchronize_device()
    return {
        "qvel": f32_list(np.asarray(data.qvel.numpy())[0]),
        "mass": f32_list(np.asarray(data.M.numpy())[0]),
        "qfrc_deriv": f32_list(np.asarray(out.numpy())[0]),
    }


def diagonal_entries(mjm, model):
    fullm_i = i32_list(model.M_fullm_i.numpy())
    fullm_j = i32_list(model.M_fullm_j.numpy())
    elemid = i32_list(model.M_elemid.numpy())
    diagonal = {}
    entries = []
    for dofi, dofj in zip(fullm_i, fullm_j):
        if dofi == dofj:
            madr = elemid[dofi * mjm.nv + dofj]
            if madr >= 0 and dofi not in diagonal:
                diagonal[dofi] = int(madr)
                entries.append([int(madr), int(dofi)])
    entries.sort()
    return entries


def upstream_fd(imports, mjm, model, qpos, qvel):
    mjw = imports["mjw"]
    mjm_nv = mjm.nv
    values = np.zeros(mjm_nv, dtype=np.float64)
    for dof in range(mjm_nv):
        probes = []
        for sign in (1.0, -1.0):
            probe = mjw.make_data(mjm, nworld=1)
            probe.qpos.assign(np.asarray([qpos], dtype=np.float32))
            qvel_probe = np.asarray(qvel, dtype=np.float32).copy()
            qvel_probe[dof] = np.float32(qvel_probe[dof] + np.float32(sign * EPS))
            probe.qvel.assign(np.asarray([qvel_probe], dtype=np.float32))
            mjw.forward(model, probe)
            passive.passive(model, probe)
            wp.synchronize_device()
            probes.append(np.asarray(probe.qfrc_passive.numpy())[0][dof])
        values[dof] = (float(probes[0]) - float(probes[1])) / (2.0 * EPS)
    return [float(value) for value in values]


def native_fd(mjm, qpos, qvel):
    values = np.zeros(mjm.nv, dtype=np.float64)
    for dof in range(mjm.nv):
        probes = []
        for sign in (1.0, -1.0):
            mjd = mujoco.MjData(mjm)
            mjd.qpos[:] = np.asarray(qpos, dtype=np.float64)
            qvel_probe = np.asarray(qvel, dtype=np.float64).copy()
            qvel_probe[dof] += sign * EPS
            mjd.qvel[:] = qvel_probe
            mujoco.mj_forward(mjm, mjd)
            probes.append(float(mjd.qfrc_passive[dof]))
        values[dof] = (probes[0] - probes[1]) / (2.0 * EPS)
    return [float(value) for value in values]


def layout_record(mjm, model) -> dict:
    damping = np.asarray(model.dof_damping.numpy())
    damping_poly = np.asarray(model.dof_dampingpoly.numpy())
    return {
        "nq": int(mjm.nq),
        "nv": int(mjm.nv),
        "nnz": int(model.nC),
        "fullm_size": int(len(model.M_fullm_i.numpy())),
        "fullm_i": i32_list(model.M_fullm_i.numpy()),
        "fullm_j": i32_list(model.M_fullm_j.numpy()),
        "elemid": i32_list(model.M_elemid.numpy()),
        "damping_batches": int(damping.shape[0]),
        "damping": f32_list(damping),
        "damping_poly_batches": int(damping_poly.shape[0]),
        "damping_poly": f32_list(damping_poly),
        "damper_enabled": True,
        "timestep": f32_list([mjm.opt.timestep])[0],
    }


def main() -> int:
    import mujoco_warp as mjw

    imports = {"mjw": mjw, "mujoco": mujoco}
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    poly_mjm, poly_model = put_model(imports, with_poly=True)
    linear_mjm, linear_model = put_model(imports, with_poly=False)
    poly_entries = diagonal_entries(poly_mjm, poly_model)
    linear_entries = diagonal_entries(linear_mjm, linear_model)
    dt = float(poly_mjm.opt.timestep)

    upstream_fd_max = 0.0
    upstream_cases = []
    for name, qpos, qvel in POLY_CASES:
        record = derive(imports, poly_mjm, poly_model, qpos, qvel)
        fd = upstream_fd(imports, poly_mjm, poly_model, qpos, qvel)
        for madr, dof in poly_entries:
            upstream_fd_max = max(
                upstream_fd_max,
                abs((record["mass"][madr] - record["qfrc_deriv"][madr]) - dt * fd[dof]),
            )
        record.update({"name": name, "qpos": f32_list(qpos), "passive_fd": fd})
        upstream_cases.append(record)
        print(
            f"case {name}: deriv_max={max(abs(value) for value in record['qfrc_deriv']):.3e}"
        )

    native_fd_max = 0.0
    native_cases = []
    for name, qpos, qvel in NATIVE_CASES:
        record = derive(imports, linear_mjm, linear_model, qpos, qvel)
        fd = native_fd(linear_mjm, qpos, qvel)
        for madr, dof in linear_entries:
            native_fd_max = max(
                native_fd_max,
                abs((record["mass"][madr] - record["qfrc_deriv"][madr]) - dt * fd[dof]),
            )
        record.update({"name": name, "qpos": f32_list(qpos), "native_passive_fd": fd})
        native_cases.append(record)
        print(f"native case {name}: native_fd_max={native_fd_max:.3e}")

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G21",
        "phase": "P7",
        "upstream_functions": ["deriv_smooth_vel"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "冻结上游 deriv_smooth_vel（M - dt*qDeriv）；无执行器/肌腱/流体，只含关节阻尼的被动速度导数项。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "poly": {
            "layout": layout_record(poly_mjm, poly_model),
            "entries": poly_entries,
            "cases": upstream_cases,
        },
        "linear": {
            "layout": layout_record(linear_mjm, linear_model),
            "entries": linear_entries,
            "cases": native_cases,
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 1e-06,
            "fd_abs": 1e-04,
            "fd_eps": EPS,
            "note": "产品与上游解析输出容差 1e-6；解析与中央差分（eps=1e-3）对照容差 1e-4。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g21-deriv.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "poly_cases": len(upstream_cases),
        "linear_cases": len(native_cases),
        "layout": {
            "nv": fixture["poly"]["layout"]["nv"],
            "nnz": fixture["poly"]["layout"]["nnz"],
            "fullm_size": fixture["poly"]["layout"]["fullm_size"],
        },
        "analytic_vs_finite_difference": {
            "upstream_fd_max_abs": upstream_fd_max,
            "native_fd_max_abs": native_fd_max,
            "fd_eps": EPS,
            "note": "解析导数（M - dt*qDeriv 对角）与上游 f32 中央差分（含合成多项式阻尼）、原生 f64 中央差分（线性阻尼）的 max abs。多项式阻尼为 MJWarp 专属字段（由执行器 dampingpoly 折叠），原生无对应字段，故原生只用线性阻尼配置对照。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/deriv/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g21-p7 deriv authoring OK:"
        f" poly_cases={len(upstream_cases)} linear_cases={len(native_cases)}"
        f" upstream_fd={upstream_fd_max:.3e} native_fd={native_fd_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
