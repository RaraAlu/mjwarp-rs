#!/usr/bin/env python
"""G0b：GPU 短窗口捕获 GJK/EPA 直达参考（pinned 3.15 + MuJoCo Warp 3.15）。

范围（Arch 2026-10-25 捕获硬口径）：
  - 每 case worlds=2：world0=qpos0、world1=qpos0+offset（offset 只作用第一个 freejoint 前 3 维）；
  - 每次捕获两次位等；输出 contact 17 字段；provenance 完整；
  - 原始接触顺序记录为观察项（不得静默忽略）。

输入：fixtures/g10-p5/models/index.json（author_g0b_models.py 生成）
输出：fixtures/g10-p5/references/<name>.json + index.json

OFFLINE ONLY：不进产品构建/测试/门禁；不读产品代码。
"""

import hashlib
import importlib.metadata
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np
import warp as wp
import mujoco_warp as mjw

ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIR = ROOT / "fixtures/g10-p5"
MODEL_INDEX = FIXTURE_DIR / "models/index.json"
OUT_DIR = FIXTURE_DIR / "references"
CACHE_DIR = ROOT / "target/g10-p5/warp-cache"
UPSTREAM_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-25-g10-p5-fixtures"

CONTACT_FLOAT_FIELDS = (
    "dist", "pos", "frame", "includemargin", "friction", "solref",
    "solreffriction", "solimp", "adhesion",
)
CONTACT_INT_FIELDS = (
    "dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid",
)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def digest(array: np.ndarray) -> str:
    return sha256_bytes(np.ascontiguousarray(array).tobytes())


def input_record(array: np.ndarray) -> dict:
    return {
        "dtype": str(array.dtype),
        "shape": list(array.shape),
        "sha256": digest(array),
        "values": array.reshape(-1).tolist(),
    }


def device_array_record(array: np.ndarray) -> dict:
    return input_record(array)


def contact_record(data) -> dict:
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in CONTACT_FLOAT_FIELDS + CONTACT_INT_FIELDS:
        value = getattr(data.contact, name).numpy()[:count]
        expected = np.float32 if name in CONTACT_FLOAT_FIELDS else np.int32
        assert value.dtype == expected, name
        if expected == np.float32:
            assert np.isfinite(value).all(), name
        fields[name] = device_array_record(value)
    return {"count": count, "fields": fields}


def default_state(host, worlds: int, offset) -> tuple:
    qpos = np.tile(host.qpos0.astype(np.float32), (worlds, 1))
    qvel = np.zeros((worlds, host.nv), dtype=np.float32)
    for world in range(worlds):
        if host.nq >= 7:
            qpos[world, 0:3] += np.asarray(offset, dtype=np.float32) * world
        if host.nv:
            qvel[world] = np.linspace(
                -0.02 * (world + 1), 0.03 * (world + 1), host.nv, dtype=np.float32
            )
    return qpos, qvel


def capture_once(host, offset) -> dict:
    """一次完整捕获：put_model → make_data(worlds=2) → qpos/qvel → kinematics → collision。"""
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=2, nconmax=4096, njmax=20000)
    qpos, qvel = default_state(host, 2, offset)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    return {
        "qpos": input_record(qpos),
        "qvel": input_record(qvel),
        "contact": contact_record(data),
    }


def f32_bits(value) -> int:
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def contact_rows(contact: dict) -> list:
    """把 contact 拆成逐接触 dict（字段宽度 = len(values)//count）。"""
    count = contact["count"]
    fields = contact["fields"]
    if count == 0:
        return []
    rows = []
    for index in range(count):
        row = {}
        for name, record in fields.items():
            values = record["values"]
            width = len(values) // count
            row[name] = tuple(values[index * width:(index + 1) * width])
        rows.append(row)
    return rows


def row_key(row: dict) -> tuple:
    """稳定排序键：worldid + dist bits + pos bits + frame bits + geom/type/dim。"""
    return (
        int(row["worldid"][0]),
        f32_bits(row["dist"][0]),
        tuple(f32_bits(v) for v in row["pos"]),
        tuple(f32_bits(v) for v in row["frame"]),
        tuple(int(v) for v in row["geom"]),
        int(row["type"][0]),
        int(row["dim"][0]),
    )


def order_observation(world_ids: list) -> dict:
    """接触顺序观察项：是否按 world 分组、原始 worldid 序列。"""
    grouped = all(
        world_ids[i] <= world_ids[i + 1] for i in range(len(world_ids) - 1)
    )
    return {
        "worldid_sequence": world_ids,
        "grouped_by_world_in_raw_order": grouped,
        "note": "通过判据用 worldid 分组 + 每 world 内 (dist,pos) 稳定排序后位级比较；"
                "原始顺序是否相等作观察项记录。",
    }


def main() -> int:
    if not MODEL_INDEX.is_file():
        raise SystemExit("缺少 %s（先跑 author_g0b_models.py）" % MODEL_INDEX)
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE_DIR)
    wp.init()
    wp.set_device("cuda:0")
    device = wp.get_device()
    gpu_name = device.name
    print("device:", gpu_name)

    index = json.loads(MODEL_INDEX.read_text(encoding="utf-8"))
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    summary = []
    for row in index["models"]:
        name = row["name"]
        xml = ROOT / row["xml"]
        offset = tuple(row["offset"])
        host = mujoco.MjModel.from_xml_path(str(xml))
        first = capture_once(host, offset)
        second = capture_once(host, offset)
        raw_identical = first["contact"] == second["contact"] and first["qpos"] == second["qpos"]
        rows_first = contact_rows(first["contact"])
        rows_second = contact_rows(second["contact"])
        sorted_identical = sorted(row_key(row) for row in rows_first) == sorted(
            row_key(row) for row in rows_second
        )
        observation = order_observation(first["contact"]["fields"]["worldid"]["values"])
        document = {
            "schema": 1,
            "task_id": TASK_ID,
            "kind": "g0b-gjk-epa-direct-reference",
            "name": name,
            "families": ["G09", "G10"],
            "input": {
                "path": row["xml"],
                "sha256": sha256_file(xml),
                "worlds": 2,
                "offset": list(offset),
                "qpos": first["qpos"],
                "qvel": first["qvel"],
            },
            "execution": {
                "stages": ["kinematics", "collision"],
                "nworld": 2,
                "nconmax": 4096,
                "njmax": 20000,
            },
            "output": first["contact"],
            "determinism": {
                "two_captures_raw_bitwise_identical": bool(raw_identical),
                "two_captures_sorted_bitwise_identical": bool(sorted_identical),
                "note": "通过判据 = worldid 分组 + 每 world 内 (dist,pos) 稳定排序后位级比较；"
                        "原始顺序是否相等为观察项（SDF initpoint/broadphase/multiccd 写入顺序）。",
            },
            "order_observation": observation,
            "provenance": {
                "upstream_revision": UPSTREAM_REVISION,
                "authoring_script": "scripts/authoring/g10-p5/capture_g0b_references.py",
                "authoring_script_sha256": sha256_file(Path(__file__)),
                "model_index_sha256": sha256_file(MODEL_INDEX),
                "versions": {
                    "python": sys.version.split()[0],
                    "mujoco": mujoco.__version__,
                    "warp": wp.__version__,
                    "mujoco_warp": importlib.metadata.version("mujoco-warp"),
                    "numpy": np.__version__,
                },
                "gpu": gpu_name,
                "authoring_only": True,
                "reads_product_tree": False,
                "product_python_dependency": False,
            },
        }
        out = OUT_DIR / (name + ".json")
        out.write_text(json.dumps(document, ensure_ascii=False, indent=1), encoding="utf-8")
        per_world = {}
        for world in sorted(set(observation["worldid_sequence"])):
            per_world[str(world)] = observation["worldid_sequence"].count(world)
        summary.append({
            "name": name,
            "file": str(out.relative_to(ROOT)).replace("\\", "/"),
            "sha256": sha256_file(out),
            "count": first["contact"]["count"],
            "per_world": per_world,
            "two_captures_raw_bitwise_identical": bool(raw_identical),
            "two_captures_sorted_bitwise_identical": bool(sorted_identical),
            "grouped_by_world_in_raw_order": observation["grouped_by_world_in_raw_order"],
        })
        print("%-20s count=%-3d per_world=%s sorted_identical=%s grouped=%s"
              % (name, first["contact"]["count"], per_world, sorted_identical,
                 observation["grouped_by_world_in_raw_order"]))

    index_doc = {
        "schema": 1,
        "task_id": TASK_ID,
        "kind": "g0b-reference-index",
        "purpose": "GJK/EPA 直达参考补采（真实缺口 8-9 项；既有 d04 参考另见 audit）",
        "upstream_revision": UPSTREAM_REVISION,
        "cases": summary,
        "provenance": {
            "authoring_script": "scripts/authoring/g10-p5/capture_g0b_references.py",
            "authoring_script_sha256": sha256_file(Path(__file__)),
            "versions": {
                "python": sys.version.split()[0],
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": importlib.metadata.version("mujoco-warp"),
                "numpy": np.__version__,
            },
            "gpu": gpu_name,
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
    }
    index_path = OUT_DIR / "index.json"
    index_path.write_text(json.dumps(index_doc, ensure_ascii=False, indent=1), encoding="utf-8")
    print("wrote", index_path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
