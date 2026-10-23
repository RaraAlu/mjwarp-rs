"""G28@P8 离线 authoring：多世界批量射线查询的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g28-batch-query.ps1 调用，
产品构建/测试/门禁/验收脚本均不启动 Python。

产出：
  fixtures/ray-query/batch-cases.json
  docs/development-evidence/backlog-batch1/2026-10-11-roadmap-g28-p8/authoring.json
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
MODEL_XML = ROOT / "fixtures" / "ray-query" / "batch-model.xml"
FIXTURE_JSON = ROOT / "fixtures" / "ray-query" / "batch-cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch1"
    / "2026-10-11-roadmap-g28-p8"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
QPOS = [0.0, 0.6]
RAYS = [
    ([0.0, 0.0, 0.5], [1.0, 0.0, 0.0]),
    ([0.0, 0.0, 0.5], [-1.0, 0.0, 0.0]),
    ([0.25, 0.0, 1.0], [0.0, 0.0, -1.0]),
    ([3.0, 3.0, 0.5], [1.0, 0.0, 0.0]),
    ([0.0, 0.0, 0.5], [0.0, 1.0, 0.0]),
]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def f32_list(values) -> list[float]:
    flat = np.asarray(values, dtype=np.float64).reshape(-1)
    return [float(np.float32(value)) for value in flat]


def main() -> int:
    import mujoco
    import mujoco_warp
    import warp

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    ngeom = int(mjm.ngeom)
    nworld = len(QPOS)
    if ngeom != 3:
        raise SystemExit(f"unexpected ngeom {ngeom}")
    model = mujoco_warp.put_model(mjm)

    per_world = []
    native_max = 0.0
    for world, qpos in enumerate(QPOS):
        data = mujoco_warp.make_data(mjm, nworld=1)
        warp.copy(data.qpos, warp.array(np.asarray([[qpos]], dtype=np.float32)))
        mujoco_warp.kinematics(model, data)
        geom_xpos = f32_list(np.asarray(data.geom_xpos.numpy())[0])
        geom_xmat = f32_list(np.asarray(data.geom_xmat.numpy())[0])
        hits = []
        for pnt, vec in RAYS:
            pnt_wp = warp.array(np.asarray(pnt, dtype=np.float32).reshape(1, 1, 3), dtype=warp.vec3)
            vec_wp = warp.array(np.asarray(vec, dtype=np.float32).reshape(1, 1, 3), dtype=warp.vec3)
            dist, geomid, normal = mujoco_warp.ray(model, data, pnt_wp, vec_wp)
            dist = float(np.asarray(dist.numpy()).reshape(-1)[0])
            geomid = int(np.asarray(geomid.numpy()).reshape(-1)[0])
            normal = f32_list(np.asarray(normal.numpy()).reshape(-1))
            hits.append({"dist": dist, "geom_id": geomid, "normal": normal})
            mjd = mujoco.MjData(mjm)
            mjd.qpos[:] = np.asarray([qpos], dtype=np.float64)
            mujoco.mj_kinematics(mjm, mjd)
            native_geomid = np.zeros(1, dtype=np.int32)
            native_normal = np.zeros(3, dtype=np.float64)
            native_dist = float(
                mujoco.mj_ray(
                    mjm,
                    mjd,
                    np.asarray(pnt, dtype=np.float64),
                    np.asarray(vec, dtype=np.float64),
                    np.full(6, -1, dtype=np.int8),
                    1,
                    -1,
                    native_geomid,
                    native_normal,
                )
            )
            if native_dist < 0:
                native_dist = -1.0
                native_geomid[0] = -1
                native_normal[:] = 0.0
            if geomid != int(native_geomid[0]):
                raise SystemExit(f"geomid mismatch world={world}")
            native_max = max(native_max, abs(dist - native_dist))
            for left, right in zip(normal, native_normal):
                native_max = max(native_max, abs(left - right))
        per_world.append(
            {
                "qpos": f32_list([qpos]),
                "geom_xpos": geom_xpos,
                "geom_xmat": geom_xmat,
                "hits": hits,
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
        "task_id": "2026-10-11-roadmap-g28-p8",
        "matrix": "G28",
        "phase": "P8",
        "upstream_functions": ["kinematics", "ray"],
        "model": {
            "path": "fixtures/ray-query/batch-model.xml",
            "sha256": sha256(MODEL_XML),
            "ngeom": ngeom,
        },
        "model_arrays": {
            "geom_type": [int(value) for value in mjm.geom_type],
            "geom_size": [f32_list(row) for row in mjm.geom_size],
        },
        "nworld": nworld,
        "nray": len(RAYS),
        "rays": [
            {"pnt": f32_list(pnt), "vec": f32_list(vec)} for pnt, vec in RAYS
        ],
        "worlds": per_world,
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": warp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "逐世界 kinematics + ray()；与 rays() 共用 _ray 内核，批量结果按世界展开。",
        },
        "native_cross_check": {
            "max_abs_dist": native_max,
            "geomid_mismatch": 0,
            "note": "原生 mj_kinematics + mj_ray（f64）先做一级判定；上游 MJWarp f32 为二级参考。",
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "dist_abs": 1e-06,
            "normal_abs": 1e-06,
            "native_dist_abs": 1e-05,
            "note": "产品与上游 FMA 收缩差异容许 1e-6；原生 f64 一级判定容许 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": fixture["task_id"],
        "generated_by": "scripts/authoring/author-g28-batch-query.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "nworld": nworld,
        "nray": len(RAYS),
        "ngeom": ngeom,
        "native_cross_check": fixture["native_cross_check"],
        "environment": fixture["environment"],
        "outputs": ["fixtures/ray-query/batch-cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g28-p8 batch query authoring OK:"
        f" nworld={nworld} nray={len(RAYS)} ngeom={ngeom}"
        f" native_max={native_max:.3e}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
