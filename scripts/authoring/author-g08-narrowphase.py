"""G08@P5 离线 authoring：plane-sphere / sphere-sphere 窄相接触的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g08-narrowphase.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/narrowphase/cases.json
  docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g08-p5/authoring.json
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
from mujoco_warp._src import collision_driver

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "narrowphase"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch2"
    / "2026-10-11-roadmap-g08-p5"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g08-p5"
NXN = 0
ALL_FILTERS = 15
CAPACITY = 4096
WORLDS = [
    [
        0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0,
        0.15, 0.0, 0.1, 0.9659258, 0.258819, 0.0, 0.0,
        1.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0,
    ],
    [
        0.05, 0.02, 0.03, 1.0, 0.0, 0.0, 0.0,
        0.1, 0.05, 0.14, 0.9238795, 0.0, 0.0, 0.3826834,
        1.2, 0.0, 0.3, 1.0, 0.0, 0.0, 0.0,
    ],
]
CONTACT_FLOAT_FIELDS = (
    "dist", "pos", "frame", "includemargin", "friction", "solref",
    "solreffriction", "solimp", "adhesion",
)
CONTACT_INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32_list(values) -> list[float]:
    return [float(np.float32(value)) for value in np.asarray(values).reshape(-1)]


def i32_list(values) -> list[int]:
    return [int(value) for value in np.asarray(values).reshape(-1)]


def rows(value) -> list[list[float]]:
    array = np.asarray(value, dtype=np.float32)
    return [f32_list(row) for row in array.reshape(array.shape[0], -1)]


def contact_fields(data, count: int) -> dict:
    fields = {}
    for name in CONTACT_FLOAT_FIELDS:
        fields[name] = f32_list(getattr(data.contact, name).numpy()[:count])
    for name in CONTACT_INT_FIELDS:
        values = getattr(data.contact, name).numpy()[:count]
        if name == "geom":
            fields[name] = [i32_list(row) for row in values]
        else:
            fields[name] = i32_list(values)
    return fields


def main() -> int:
    import mujoco_warp as mjw

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    model = mjw.put_model(mjm)
    aabb = np.asarray(model.geom_aabb.numpy())
    broadphase_model = {
        "ngeom": int(mjm.ngeom),
        "geom_type": i32_list(model.geom_type.numpy()),
        "dataid_batches": int(np.asarray(model.geom_dataid.numpy()).shape[0]),
        "geom_dataid": i32_list(model.geom_dataid.numpy()),
        "aabb_batches": int(aabb.shape[0]),
        "geom_aabb_center": [
            f32_list(aabb[batch, geom, 0])
            for batch in range(aabb.shape[0])
            for geom in range(aabb.shape[1])
        ],
        "geom_aabb_size": [
            f32_list(aabb[batch, geom, 1])
            for batch in range(aabb.shape[0])
            for geom in range(aabb.shape[1])
        ],
        "rbound_batches": int(np.asarray(model.geom_rbound.numpy()).shape[0]),
        "geom_rbound": f32_list(model.geom_rbound.numpy()),
        "margin_batches": int(np.asarray(model.geom_margin.numpy()).shape[0]),
        "geom_margin": f32_list(model.geom_margin.numpy()),
        "gap_batches": int(np.asarray(model.geom_gap.numpy()).shape[0]),
        "geom_gap": f32_list(model.geom_gap.numpy()),
        "pairs": [i32_list(row) for row in np.asarray(model.nxn_geom_pair_filtered.numpy())],
        "pairids": [i32_list(row) for row in np.asarray(model.nxn_pairid_filtered.numpy())],
        "full_pairid": [i32_list(row) for row in np.asarray(model.nxn_pairid.numpy())],
    }
    narrowphase_model = {
        "ngeom": int(mjm.ngeom),
        "geom_type": i32_list(mjm.geom_type),
        "geom_condim": i32_list(mjm.geom_condim),
        "geom_priority": i32_list(mjm.geom_priority),
        "geom_solmix": f32_list(mjm.geom_solmix),
        "geom_solref": rows(mjm.geom_solref),
        "geom_solimp": rows(mjm.geom_solimp),
        "geom_friction": rows(mjm.geom_friction),
        "geom_margin": f32_list(mjm.geom_margin),
        "geom_gap": f32_list(mjm.geom_gap),
        "geom_adhesion": f32_list(mjm.geom_adhesion),
        "geom_size": rows(mjm.geom_size),
    }

    qpos = np.asarray(WORLDS, dtype=np.float32)
    data = mjw.make_data(mjm, nworld=len(WORLDS), naconmax=CAPACITY, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((len(WORLDS), mjm.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    wp.synchronize_device()
    world_records = [
        {
            "qpos": f32_list(qpos[index]),
            "geom_xpos": f32_list(np.asarray(data.geom_xpos.numpy())[index]),
            "geom_xmat": f32_list(np.asarray(data.geom_xmat.numpy())[index]),
        }
        for index in range(len(WORLDS))
    ]

    bdata = mjw.make_data(mjm, nworld=len(WORLDS), naconmax=CAPACITY, njmax=20000)
    bdata.qpos.assign(qpos)
    bdata.qvel.assign(np.zeros((len(WORLDS), mjm.nv), dtype=np.float32))
    mjw.kinematics(model, bdata)
    model.opt.broadphase = NXN
    model.opt.broadphase_filter = ALL_FILTERS
    ctx = collision_driver.create_collision_context(bdata.naconmax)
    bdata.ncollision.zero_()
    collision_driver.nxn_broadphase(model, bdata, ctx)
    wp.synchronize_device()
    ncollision = int(np.asarray(bdata.ncollision.numpy()).reshape(-1)[0])
    written = min(ncollision, int(bdata.naconmax))
    candidates = {
        "ncollision": ncollision,
        "pairs": [i32_list(row) for row in np.asarray(ctx.collision_pair.numpy())[:written]],
        "pairids": [i32_list(row) for row in np.asarray(ctx.collision_pairid.numpy())[:written]],
        "worldids": i32_list(np.asarray(ctx.collision_worldid.numpy())[:written]),
    }

    cdata = mjw.make_data(mjm, nworld=len(WORLDS), naconmax=CAPACITY, njmax=20000)
    cdata.qpos.assign(qpos)
    cdata.qvel.assign(np.zeros((len(WORLDS), mjm.nv), dtype=np.float32))
    mjw.kinematics(model, cdata)
    wp.synchronize_device()
    mjw.collision(model, cdata)
    wp.synchronize_device()
    count = int(np.asarray(cdata.nacon.numpy()).reshape(-1)[0])
    contacts = {"count": count, "fields": contact_fields(cdata, count)}
    print(f"upstream contacts: {count}, candidates: {ncollision}")

    native_worlds = []
    for index, world in enumerate(WORLDS):
        mjd = mujoco.MjData(mjm)
        mjd.qpos[:] = np.asarray(world, dtype=np.float64)
        mujoco.mj_kinematics(mjm, mjd)
        mujoco.mj_collision(mjm, mjd)
        ncon = int(mjd.ncon)
        native_contacts = {
            "dist": [float(mjd.contact[i].dist) for i in range(ncon)],
            "pos": f32_list([mjd.contact[i].pos for i in range(ncon)]),
            "frame": f32_list([mjd.contact[i].frame for i in range(ncon)]),
            "includemargin": [float(mjd.contact[i].includemargin) for i in range(ncon)],
            "dim": [int(mjd.contact[i].dim) for i in range(ncon)],
            "geom": [i32_list(mjd.contact[i].geom) for i in range(ncon)],
        }
        native_worlds.append({"ncon": ncon, "contacts": native_contacts})
        print(f"native world {index}: ncon={ncon}")

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G08",
        "phase": "P5",
        "upstream_functions": ["nxn_broadphase", "collision"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": "冻结上游 collision()（NXN 宽相 + primitive 窄相）；接触顺序由原子计数决定，比较按 (worldid, geom) 规范化。",
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "broadphase": broadphase_model,
        "narrowphase": narrowphase_model,
        "nworld": len(WORLDS),
        "worlds": world_records,
        "candidates": candidates,
        "contacts": contacts,
        "native": {"worlds": native_worlds},
        "constants": {
            "MJ_MINVAL": float(mujoco.mjMINVAL),
            "MJ_MINMU": float(mujoco.mjMINMU),
        },
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "tolerance": {
            "upstream_abs": 1e-06,
            "native_abs": 1e-05,
            "note": "接触浮点字段与上游容差 1e-6（FMA/求和顺序末位差）；原生 f64 mj_collision 容差 1e-5。",
        },
    }
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g08-narrowphase.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "ngeom": int(mjm.ngeom),
        "nworld": len(WORLDS),
        "candidates": ncollision,
        "contacts": count,
        "native_ncon": [world["ncon"] for world in native_worlds],
        "constants": fixture["constants"],
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/narrowphase/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g08-p5 narrowphase authoring OK:"
        f" ngeom={mjm.ngeom} worlds={len(WORLDS)} candidates={ncollision}"
        f" contacts={count} native={[world['ncon'] for world in native_worlds]}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
