"""G5 · ellipsoid-box 三方对照：上游 f32 CPU fixture 采集（authoring-only，不占 GPU）。

同一 qpos/qvel（取 G0b CUDA 参考的 f32 位型）在 warp **CPU** 后端重跑，输出接触字段，
并与 G0b CUDA fixture 做 worldid 分组 + 稳定排序后的逐字段位级/ULP 比较。

用法（仓库根目录）：
    target/mjwarp-parallel/venv/Scripts/python.exe -B \
        docs/development-evidence/g10-p5-gpu-kernels/gpu-window/capture_ellipsoid_cpu.py
输出：gpu-window/ellipsoid-box-cpu-capture.json
"""

import json
import struct
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[4]
BASE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window"
CASE = "ellipsoid-box"
REF = ROOT / f"fixtures/g10-p5/references/{CASE}.json"

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")


def bits(values):
    return [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in np.ravel(values)]


def rows(count, fields):
    out = []
    for index in range(count):
        row = {}
        for name in FLOAT_FIELDS:
            values = fields[name]["values"]
            width = len(values) // count if count else 0
            row[name] = np.asarray(values[index * width:(index + 1) * width], dtype=np.float32)
        for name in INT_FIELDS:
            values = fields[name]["values"]
            width = len(values) // count if count else 0
            row[name] = np.asarray(values[index * width:(index + 1) * width], dtype=np.int32)
        out.append(row)
    return out


def key(row):
    return (
        int(row["worldid"][0]),
        row["dist"].tobytes(),
        row["pos"].tobytes(),
        row["frame"].tobytes(),
        tuple(int(v) for v in row["geom"]),
        int(row["type"][0]),
        int(row["dim"][0]),
        int(row["geomcollisionid"][0]),
    )


def main():
    wp.config.kernel_cache_dir = str(ROOT / "target/g10-p5-g5/warp-cache-cpu")
    wp.set_device("cpu")
    doc = json.loads(REF.read_text(encoding="utf-8"))
    worlds = int(doc["input"]["worlds"])
    qpos = np.asarray(doc["input"]["qpos"]["values"], dtype=np.float32).reshape(worlds, -1)
    qvel = np.asarray(doc["input"]["qvel"]["values"], dtype=np.float32).reshape(
        worlds, doc["input"]["qvel"]["shape"][1]
    )
    host = mujoco.MjModel.from_xml_path(str(ROOT / doc["input"]["path"]))
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    device_fields = {}
    for name in FLOAT_FIELDS:
        values = getattr(data.contact, name).numpy()[:count]
        device_fields[name] = {"values": values.reshape(-1).tolist()}
    for name in INT_FIELDS:
        values = getattr(data.contact, name).numpy()[:count]
        device_fields[name] = {"values": values.reshape(-1).tolist()}
    cpu_rows = rows(count, device_fields)
    ref_count = int(doc["output"]["count"])
    ref_rows = rows(ref_count, doc["output"]["fields"])
    cpu_rows.sort(key=key)
    ref_rows.sort(key=key)
    stats = {}
    if count == ref_count:
        for name in FLOAT_FIELDS:
            dv = np.concatenate([row[name] for row in cpu_rows])
            rv = np.concatenate([row[name] for row in ref_rows])
            equal = (dv.view(np.uint32) == rv.view(np.uint32))
            nonzero = rv != 0.0
            ulp = np.abs(dv.view(np.int32).astype(np.int64)
                         - rv.view(np.int32).astype(np.int64))
            stats[name] = {
                "cells": int(dv.size),
                "bit_equal": int(equal.sum()),
                "nonzero_cells": int(nonzero.sum()),
                "max_ulp_nonzero_ref": int(ulp[nonzero].max()) if nonzero.any() else 0,
                "zero_ref_max_abs": float(np.abs(dv[~nonzero]).max()) if (~nonzero).any() else 0.0,
                "max_abs": float(np.abs(dv - rv).max()),
            }
        for name in INT_FIELDS:
            dv = [int(v) for row in cpu_rows for v in row[name]]
            rv = [int(v) for row in ref_rows for v in row[name]]
            stats[name] = {"cells": len(dv), "equal": dv == rv}
    report = {
        "schema": 1,
        "kind": "g5-ellipsoid-box-upstream-cpu-vs-cuda",
        "task_id": "2026-10-25-g10-p5-gpu-kernels",
        "case": CASE,
        "same_input": "qpos/qvel 取自 G0b CUDA 参考的 f32 位型",
        "cpu_count": count,
        "cuda_count": ref_count,
        "cpu_contact_fields": {
            name: device_fields[name]["values"] for name in FLOAT_FIELDS + INT_FIELDS
        },
        "sorted_compare_cpu_vs_cuda": stats,
        "provenance": {
            "device": "cpu",
            "warp": wp.__version__,
            "mujoco": mujoco.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
            "numpy": np.__version__,
            "authoring_only": True,
            "reads_product_tree": False,
        },
    }
    path = BASE / "ellipsoid-box-cpu-capture.json"
    path.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("cpu count", count, "cuda count", ref_count)
    if stats:
        for name in FLOAT_FIELDS:
            s = stats[name]
            print(f"{name}: bit_eq={s['bit_equal']}/{s['cells']} "
                  f"max_ulp_nonzero={s['max_ulp_nonzero_ref']} max_abs={s['max_abs']:.3e}")
    print("wrote", path)


if __name__ == "__main__":
    main()
