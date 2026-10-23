"""G09@P5 离线 authoring：plane-box 多接触/退化场景的冻结参考。

离线专用：本脚本只被 scripts/authoring/author-g09-plane-box.ps1 调用。
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。

产出：
  fixtures/plane-box/cases.json
  docs/development-evidence/backlog-batch3/2026-10-11-roadmap-g09-p5/authoring.json
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
from mujoco_warp._src import collision_driver  # noqa: F401  (pinned-source audit marker)

ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "fixtures" / "plane-box"
MODEL_XML = FIXTURE_DIR / "model.xml"
FIXTURE_JSON = FIXTURE_DIR / "cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch3"
    / "2026-10-11-roadmap-g09-p5"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g09-p5"
NXN = 0
ALL_FILTERS = 15
CAPACITY = 2048
# (名称, 活动 box 索引(a=1/b=2), 四元数 wxyz, 模式, 量值米)
# pen：min 角点间隙 = -量值（穿入）；gap：min 角点间隙 = +量值（不接触）。
WORLD_SPECS = [
    ("flat4", 1, [1.0, 0.0, 0.0, 0.0], "pen", 0.01),
    ("edge2", 1, [0.96592583, 0.25881905, 0.0, 0.0], "pen", 0.003),
    ("corner1", 1, [0.93969262, 0.24184476, 0.24184476, 0.0], "pen", 0.003),
    ("gap0", 1, [1.0, 0.0, 0.0, 0.0], "gap", 1.0),
    ("yaw45_flat4", 1, [0.92387953, 0.0, 0.0, 0.38268343], "pen", 0.01),
    ("thin_flat4", 2, [1.0, 0.0, 0.0, 0.0], "pen", 0.0008),
    ("thin_gap0", 2, [1.0, 0.0, 0.0, 0.0], "gap", 0.002),
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


def native_contacts(mjm, qpos) -> dict:
    mjd = mujoco.MjData(mjm)
    mjd.qpos[:] = np.asarray(qpos, dtype=np.float64)
    mujoco.mj_kinematics(mjm, mjd)
    mujoco.mj_collision(mjm, mjd)
    count = int(mjd.ncon)
    return {
        "ncon": count,
        "dist": [float(mjd.contact[i].dist) for i in range(count)],
        "pos": f32_list([mjd.contact[i].pos for i in range(count)]),
        "frame": f32_list([mjd.contact[i].frame for i in range(count)]),
        "includemargin": f32_list(
            [mjd.contact[i].includemargin for i in range(count)]
        ),
        "dim": [int(mjd.contact[i].dim) for i in range(count)],
        "geom": [i32_list(mjd.contact[i].geom) for i in range(count)],
    }


def world_qpos(mjm, box_index: int, quat, mode: str, amount: float) -> np.ndarray:
    """按 box 角点最低间隙构造 qpos（box 中心 z = -min_offset + 间隙）。"""
    qpos = np.zeros(int(mjm.nq), dtype=np.float64)
    size = np.asarray(mjm.geom_size[box_index], dtype=np.float64)
    corners = np.array(
        [
            [sx * size[0], sy * size[1], sz * size[2]]
            for sx in (-1.0, 1.0)
            for sy in (-1.0, 1.0)
            for sz in (-1.0, 1.0)
        ]
    )
    rot = np.zeros(9, dtype=np.float64)
    mujoco.mju_quat2Mat(rot, np.asarray(quat, dtype=np.float64))
    rot = rot.reshape(3, 3)
    offsets = corners @ rot.T
    clearance = -amount if mode == "pen" else amount
    center_z = -float(np.min(offsets[:, 2])) + clearance
    pos_slot = 0 if box_index == 1 else 7
    qpos[pos_slot : pos_slot + 3] = [0.0, 0.0, center_z]
    qpos[pos_slot + 3 : pos_slot + 7] = quat
    inactive = 2 if box_index == 1 else 1
    inactive_slot = 0 if inactive == 1 else 7
    qpos[inactive_slot : inactive_slot + 3] = [0.0, 0.0, 10.0]
    qpos[inactive_slot + 3 : inactive_slot + 7] = [1.0, 0.0, 0.0, 0.0]
    return qpos


def main() -> int:
    import mujoco_warp as mjw

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=UPSTREAM, check=True, capture_output=True, text=True
    ).stdout.strip()
    if head != PINNED_REVISION:
        raise SystemExit(f"upstream HEAD {head} != pinned {PINNED_REVISION}")

    mjm = mujoco.MjModel.from_xml_path(str(MODEL_XML))
    model = mjw.put_model(mjm)
    worlds_qpos = [
        world_qpos(mjm, box_index, quat, mode, amount)
        for _, box_index, quat, mode, amount in WORLD_SPECS
    ]
    qpos = np.asarray(worlds_qpos, dtype=np.float32)

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

    data = mjw.make_data(mjm, nworld=len(WORLD_SPECS), naconmax=CAPACITY, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((len(WORLD_SPECS), mjm.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    wp.synchronize_device()
    world_records = [
        {
            "name": name,
            "qpos": f32_list(qpos[index]),
            "geom_xpos": f32_list(np.asarray(data.geom_xpos.numpy())[index]),
            "geom_xmat": f32_list(np.asarray(data.geom_xmat.numpy())[index]),
        }
        for index, name in enumerate(spec[0] for spec in WORLD_SPECS)
    ]

    cdata = mjw.make_data(mjm, nworld=len(WORLD_SPECS), naconmax=CAPACITY, njmax=20000)
    cdata.qpos.assign(qpos)
    cdata.qvel.assign(np.zeros((len(WORLD_SPECS), mjm.nv), dtype=np.float32))
    mjw.kinematics(model, cdata)
    model.opt.broadphase = NXN
    model.opt.broadphase_filter = ALL_FILTERS
    wp.synchronize_device()
    mjw.collision(model, cdata)
    wp.synchronize_device()
    count = int(np.asarray(cdata.nacon.numpy()).reshape(-1)[0])
    contacts = {"count": count, "fields": contact_fields(cdata, count)}
    print(f"upstream contacts: {count}")

    # 每个世界的手动候选：仅 plane-box（避免引入未实现的 box-box 对）。
    pairs, pairids, worldids = [], [], []
    for index in range(len(WORLD_SPECS)):
        for box in (1, 2):
            pairs.append([0, box])
            pairids.append([-1, -1])
            worldids.append(index)
    candidates = {
        "pairs": pairs,
        "pairids": pairids,
        "worldids": worldids,
    }

    native = {
        "worlds": [
            {"name": spec[0], "contacts": native_contacts(mjm, spec_qpos)}
            for spec, spec_qpos in zip(WORLD_SPECS, worlds_qpos)
        ]
    }
    for world in native["worlds"]:
        print(f"native {world['name']}: ncon={world['contacts']['ncon']}")

    gpu = subprocess.run(
        ["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    fixture = {
        "task_id": TASK_ID,
        "matrix": "G09",
        "phase": "P5",
        "upstream_functions": ["kinematics", "collision", "plane_box_wrapper"],
        "upstream_reference": {
            "revision": PINNED_REVISION,
            "warp": wp.__version__,
            "mujoco_warp": "3.15.0",
            "mujoco": mujoco.__version__,
            "note": (
                "冻结上游 collision()（NXN 宽相 + primitive 窄相 plane-box 分支，"
                "`collision_primitive.plane_box_wrapper` → `plane_box` 8 角点写接触）；"
                "每世界 2 个 plane-box 候选（含远端无接触的 box），接触按 (worldid, geom) 分组有序对照。"
            ),
        },
        "model": {"path": MODEL_XML.relative_to(ROOT).as_posix(), "sha256": sha256(MODEL_XML)},
        "narrowphase": narrowphase_model,
        "nworld": len(WORLD_SPECS),
        "worlds": world_records,
        "candidates": candidates,
        "contacts": contacts,
        "native": native,
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
    FIXTURE_JSON.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_JSON.write_text(
        json.dumps(fixture, ensure_ascii=False, indent=1) + "\n", encoding="utf-8"
    )
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    evidence = {
        "task_id": TASK_ID,
        "generated_by": "scripts/authoring/author-g09-plane-box.ps1",
        "model_sha256": fixture["model"]["sha256"],
        "ngeom": int(mjm.ngeom),
        "nworld": len(WORLD_SPECS),
        "contacts": count,
        "native_ncon": [world["contacts"]["ncon"] for world in native["worlds"]],
        "constants": fixture["constants"],
        "environment": {"python": platform.python_version(), "gpu": gpu},
        "outputs": ["fixtures/plane-box/cases.json"],
    }
    EVIDENCE_JSON.write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(
        "g09-p5 plane-box authoring OK: "
        f"ngeom={mjm.ngeom} worlds={len(WORLD_SPECS)} contacts={count} "
        f"native={[world['contacts']['ncon'] for world in native['worlds']]}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
