"""G5 GPU 窗口 · 探针输出 vs 上游 CUDA 参考的差异量化（CPU 侧，不占 GPU）。

用途：在 Rust 验收测试之前给出逐 case / 逐字段的差异分布（位等数、非零分量最大 ULP、
参考为零分量的最大绝对残差），用于判定「≤4 ULP / 位级优先」口径与定位代码生成差异。

用法：
    python .../gpu-window/compare_probe.py
输出：gpu-window/compare-probe.json + 控制台摘要。
"""

import json
import struct
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[4]
BASE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window"
CASES = (
    "sphere-box-shallow",
    "sphere-box-deep",
    "capsule-box",
    "ellipsoid-box",
    "hfield-sphere",
    "hfield-capsule",
    "sdf-box",
)
FLOAT_FIELDS = ("dist", "pos", "frame")
FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref", "solreffriction",
          "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")


def f32(path):
    return np.fromfile(path, dtype="<f4")


def i32(path):
    return np.fromfile(path, dtype="<i4")


def read_reference(case):
    doc = json.loads((ROOT / f"fixtures/g10-p5/references/{case}.json").read_text(encoding="utf-8"))
    count = int(doc["output"]["count"])
    rows = []
    for index in range(count):
        row = {}
        for name in FIELDS:
            values = doc["output"]["fields"][name]["values"]
            width = len(values) // count if count else 0
            row[name] = np.asarray(values[index * width:(index + 1) * width], dtype=np.float32)
        for name in INT_FIELDS:
            values = doc["output"]["fields"][name]["values"]
            width = len(values) // count if count else 0
            row[name] = np.asarray(values[index * width:(index + 1) * width], dtype=np.int32)
        rows.append(row)
    return count, rows


def read_device(directory):
    count = int(i32(directory / "nacon.bin")[0])
    rows = []
    for index in range(count):
        row = {}
        for name in FIELDS:
            width = {"dist": 1, "pos": 3, "frame": 9, "friction": 5, "solref": 2,
                     "solreffriction": 2, "solimp": 5}.get(name, 1)
            row[name] = f32(directory / f"{name}.bin")[index * width:(index + 1) * width]
        for name in INT_FIELDS:
            width = {"dim": 1, "geom": 2, "worldid": 1, "type": 1, "geomcollisionid": 1}[name]
            row[name] = i32(directory / f"{name}.bin")[index * width:(index + 1) * width]
        rows.append(row)
    return count, rows


def sort_key(row):
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


def bits(array):
    return array.astype(np.float32).view(np.uint32)


def stats(dev_rows, ref_rows):
    out = {}
    for name in FLOAT_FIELDS:
        dv = np.concatenate([row[name] for row in dev_rows])
        rv = np.concatenate([row[name] for row in ref_rows])
        equal = bits(dv) == bits(rv)
        nonzero = rv != 0.0
        ulp = np.abs(dv.view(np.int32).astype(np.int64) - rv.view(np.int32).astype(np.int64))
        out[name] = {
            "cells": int(dv.size),
            "bit_equal": int(equal.sum()),
            "nonzero_cells": int(nonzero.sum()),
            "max_ulp_nonzero_ref": int(ulp[nonzero].max()) if nonzero.any() else 0,
            "zero_ref_cells": int((~nonzero).sum()),
            "zero_ref_max_abs": float(np.abs(dv[~nonzero]).max()) if (~nonzero).any() else 0.0,
            "zero_ref_max_abs_dev": float(np.abs(dv[~nonzero]).max()) if (~nonzero).any() else 0.0,
            "max_abs": float(np.abs(dv - rv).max()),
        }
    for name in FIELDS:
        if name in FLOAT_FIELDS:
            continue
        dv = np.concatenate([row[name] for row in dev_rows])
        rv = np.concatenate([row[name] for row in ref_rows])
        out[name] = {"cells": int(dv.size), "bit_equal": int((bits(dv) == bits(rv)).sum())}
    for name in INT_FIELDS:
        dv = [int(v) for row in dev_rows for v in row[name]]
        rv = [int(v) for row in ref_rows for v in row[name]]
        out[name] = {"cells": len(dv), "equal": dv == rv}
    return out


def main():
    report = {"schema": 1, "kind": "g5-probe-vs-upstream-cuda-diff", "cases": {}}
    for case in CASES:
        directory = BASE / "cases" / case / "out1"
        dev_count, dev_rows = read_device(directory)
        ref_count, ref_rows = read_reference(case)
        dev_rows.sort(key=sort_key)
        ref_rows.sort(key=sort_key)
        entry = {
            "device_count": dev_count,
            "reference_count": ref_count,
            "count_equal": dev_count == ref_count,
        }
        if dev_count == ref_count:
            entry["fields"] = stats(dev_rows, ref_rows)
        report["cases"][case] = entry
        print(case, "count", dev_count, "vs", ref_count)
        if dev_count == ref_count:
            for name in FLOAT_FIELDS:
                field = entry["fields"][name]
                print(
                    f"  {name}: bit_eq={field['bit_equal']}/{field['cells']} "
                    f"max_ulp_nonzero={field['max_ulp_nonzero_ref']} "
                    f"zero_ref_max_abs={field['zero_ref_max_abs']:.3e} max_abs={field['max_abs']:.3e}"
                )
    path = BASE / "compare-probe.json"
    path.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", path)


if __name__ == "__main__":
    main()
