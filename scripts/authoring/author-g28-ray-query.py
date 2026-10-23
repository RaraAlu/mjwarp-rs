"""G28@P7 离线 authoring：原语射线查询（单体/批量/无命中）的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g28-ray-query.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/ray-query/ray-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g28-p7/authoring.json
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
FIXTURE_DIR = ROOT / "fixtures" / "ray-query"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "ray-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g28-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
RAYS = [
    ([0.0, 0.0, 0.5], [1.0, 0.0, 0.0]),
    ([0.0, 0.1, 0.5], [-1.0, 0.0, 0.0]),
    ([0.1, 0.0, 0.5], [0.0, 1.0, 0.0]),
    ([0.1, 0.0, 0.5], [0.0, -1.0, 0.0]),
    ([0.05, 0.1, 0.5], [0.0, 0.0, 1.0]),
    ([0.3, 0.2, 0.5], [0.0, 0.0, -1.0]),
    ([5.0, 5.0, 5.0], [0.0, 1.0, 0.0]),
    ([3.0, 3.0, 0.5], [1.0, 0.0, 0.0]),
    ([-2.0, 0.0, 0.5], [1.0, 0.0, 0.0]),
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in values]


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    ngeom = int(mjm.ngeom)
    if ngeom != 6:
        raise SystemExit(f"unexpected ngeom {ngeom}")
    model = mujoco_warp.put_model(mjm)
    data = mujoco_warp.make_data(mjm, nworld=1)
    mujoco_warp.kinematics(model, data)
    geom_xpos = np.asarray(data.geom_xpos.numpy())[0]
    geom_xmat = np.asarray(data.geom_xmat.numpy())[0]

    mjd = mujoco.MjData(mjm)
    mujoco.mj_kinematics(mjm, mjd)
    geomgroup = np.full(6, -1, dtype=np.int8)

    results = []
    native_max_dist = 0.0
    native_max_normal = 0.0
    native_geomid_mismatch = 0
    for index, (pnt, vec) in enumerate(RAYS):
        pnt64 = np.asarray(pnt, dtype=np.float64)
        vec64 = np.asarray(vec, dtype=np.float64)
        pnt_wp = warp.array(np.asarray(pnt, dtype=np.float32).reshape(1, 1, 3), dtype=warp.vec3)
        vec_wp = warp.array(np.asarray(vec, dtype=np.float32).reshape(1, 1, 3), dtype=warp.vec3)
        dist, geomid, normal = mujoco_warp.ray(model, data, pnt_wp, vec_wp)
        dist = float(np.asarray(dist.numpy()).reshape(-1)[0])
        geomid = int(np.asarray(geomid.numpy()).reshape(-1)[0])
        normal = [float(value) for value in np.asarray(normal.numpy()).reshape(-1)]

        native_geomid = np.zeros(1, dtype=np.int32)
        native_normal = np.zeros(3, dtype=np.float64)
        native_dist = float(
            mujoco.mj_ray(mjm, mjd, pnt64, vec64, geomgroup, 1, -1, native_geomid, native_normal)
        )
        if native_dist < 0:
            native_dist = -1.0
            native_geomid[0] = -1
            native_normal[:] = 0.0
        if geomid != int(native_geomid[0]):
            native_geomid_mismatch += 1
        native_max_dist = max(native_max_dist, abs(dist - native_dist))
        for left, right in zip(normal, native_normal):
            native_max_normal = max(native_max_normal, abs(left - right))
        results.append(
            {
                "index": index,
                "pnt": f32_list(pnt),
                "vec": f32_list(vec),
                "dist": dist,
                "geom_id": geomid,
                "normal": normal,
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
        "task_id": "2026-10-11-roadmap-g28-p7",
        "matrix": "G28",
        "phase": "P7",
        "upstream_functions": ["mujoco_warp.ray"],
        "model": {
            "path": "fixtures/ray-query/model.xml",
            "sha256": sha256(MODEL_XML),
            "ngeom": ngeom,
        },
        "model_arrays": {
            "geom_type": [int(value) for value in mjm.geom_type],
            "geom_size": [[float(value) for value in row] for row in mjm.geom_size],
        },
        "nworld": 1,
        "frozen_data": {
            "geom_xpos": [[float(value) for value in row] for row in geom_xpos],
            "geom_xmat": [
                [float(value) for value in np.asarray(row).reshape(-1)] for row in geom_xmat
            ],
        },
        "nray": len(RAYS),
        "rays": results,
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
        },
        "native_cross_check": {
            "max_abs_dist": native_max_dist,
            "max_abs_normal": native_max_normal,
            "geomid_mismatch": native_geomid_mismatch,
            "note": "原生 mj_ray（f64）先做一级判定；上游 MJWarp f32 单射线输出为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "dist_abs": 1e-06,
            "normal_abs": 1e-06,
            "native_dist_abs": 1e-05,
            "native_normal_abs": 1e-05,
            "note": "上游 f32 与产品 f32 的差异容许 1e-6（FMA 收缩顺序）；原生 f64 一级判定容许 1e-5。",
        },
    }

    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g28-ray-query.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nray": len(RAYS),
        "geom_types": fixture["model_arrays"]["geom_type"],
        "hits": [item["index"] for item in results if item["geom_id"] >= 0],
        "misses": [item["index"] for item in results if item["geom_id"] < 0],
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/ray-query/ray-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(
        "g28 ray authoring OK:"
        f" ngeom={ngeom} nray={len(RAYS)}"
        f" native_max_dist={native_max_dist:.3e}"
        f" native_max_normal={native_max_normal:.3e}"
        f" geomid_mismatch={native_geomid_mismatch}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
