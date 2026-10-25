#!/usr/bin/env python
"""G0b：3.12 vs 3.15 双版本字段 parity（纯 CPU，不初始化 warp/CUDA）。

输入：
  fixtures/g10-p5/dump-312/<model>/   （MuJoCo 3.12 C++ dump）
  fixtures/g10-p5/dump-315/<model>/   （pinned MuJoCo 3.15 python dump）
输出：
  fixtures/g10-p5/octree-parity.json

判据：
  ① oct_aabb / oct_coeff 等 **f64 位型**逐字节全等（octree 原始数据）；
  ② hfield_data **f32 位型**逐字节全等、hfield_size f64 全等；
  ③ 结构字段（oct_depth/oct_child/mesh_octadr/octnum/hfield_nrow/ncol/adr/opt_sdf/
     geom_type/geom_dataid/counts/nq_nv）逐字节全等。
先测后定：
  - 全等 → SDF/hfield 数值验收走位级；
  - 不全等 → G3/G4/G5 统一用 3.15 导出的 oct_* 夹具作两端输入，G6 unmet 写明。
"""

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIR = ROOT / "fixtures/g10-p5"
MODELS = ["hfield-box", "sdf-cylinder"]

F64_FIELDS = ["oct_aabb", "oct_coeff", "hfield_size", "geom_size", "geom_margin"]
F32_FIELDS = ["hfield_data"]
STRUCT_FIELDS = [
    "counts",
    "nq_nv",
    "oct_depth",
    "oct_child",
    "mesh_octadr",
    "mesh_octnum",
    "hfield_nrow",
    "hfield_ncol",
    "hfield_adr",
    "opt_sdf",
    "geom_type",
    "geom_dataid",
]


def load_index(side: str, model: str) -> dict:
    path = ROOT / "target/g10-p5" / ("dump-%s" % side) / model / "fields.json"
    if not path.is_file():
        raise SystemExit("缺少 %s（先跑 3.12/3.15 dump）" % path)
    document = json.loads(path.read_text(encoding="utf-8"))
    base = path.parent
    return {
        "document": document,
        "base": base,
        "entries": {entry["name"]: entry for entry in document["fields"]},
    }


def compare_field(left: dict, right: dict, name: str) -> dict:
    if name not in left["entries"] or name not in right["entries"]:
        return {"name": name, "status": "missing", "equal": False}
    left_entry = left["entries"][name]
    right_entry = right["entries"][name]
    left_bytes = (left["base"] / left_entry["file"]).read_bytes()
    right_bytes = (right["base"] / right_entry["file"]).read_bytes()
    equal = left_bytes == right_bytes
    result = {
        "status": "ok",
        "name": name,
        "dtype": left_entry["dtype"],
        "count": left_entry["count"],
        "bytes": len(left_bytes),
        "equal": equal,
        "sha256_312": hashlib.sha256(left_bytes).hexdigest(),
        "sha256_315": hashlib.sha256(right_bytes).hexdigest(),
    }
    if not equal:
        result["dtype_mismatch"] = left_entry["dtype"] != right_entry["dtype"]
        result["count_mismatch"] = left_entry["count"] != right_entry["count"]
        result["first_diff_byte"] = next(
            (index for index, (a, b) in enumerate(zip(left_bytes, right_bytes)) if a != b),
            min(len(left_bytes), len(right_bytes)),
        )
    return result


def main() -> int:
    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-fixtures",
        "kind": "g0b-octree-parity",
        "note": "3.12 C++ dump vs pinned 3.15 python dump；逐字段原始字节比较（f64/f32/i32/i64 小端）。"
                "3.15 无法加载 3.12 MJB（实测 different number of sizes），故两侧各自生成、同版本重载校验。",
        "models": {},
        "overall": {},
    }
    for model in MODELS:
        left = load_index("312", model)
        right = load_index("315", model)
        model_report = {
            "source_xml": left["document"]["source_xml"],
            "mujoco_312": left["document"]["mujoco_version"],
            "mujoco_315": right["document"]["mujoco_version"],
            "counts_312": left["document"]["counts"],
            "counts_315": right["document"]["counts"],
            "f64_fields": [compare_field(left, right, name) for name in F64_FIELDS],
            "f32_fields": [compare_field(left, right, name) for name in F32_FIELDS],
            "struct_fields": [compare_field(left, right, name) for name in STRUCT_FIELDS],
        }
        for group in ["f64_fields", "f32_fields", "struct_fields"]:
            model_report[group] = [row for row in model_report[group] if row["status"] != "missing"]

        octree_fields = [row for row in model_report["f64_fields"] if row["name"].startswith("oct_")]
        model_report["octree_f64_equal"] = all(row["equal"] for row in octree_fields) and bool(octree_fields)
        hfield_fields = [
            row for row in model_report["f64_fields"] + model_report["f32_fields"]
            if row["name"].startswith("hfield_")
        ]
        model_report["hfield_equal"] = all(row["equal"] for row in hfield_fields) and bool(hfield_fields)
        model_report["struct_equal"] = all(row["equal"] for row in model_report["struct_fields"])
        report["models"][model] = model_report
        print(
            "parity %s: octree_f64_equal=%s hfield_equal=%s struct_equal=%s noct_312=%s noct_315=%s"
            % (model, model_report["octree_f64_equal"], model_report["hfield_equal"],
               model_report["struct_equal"],
               model_report["counts_312"].get("noct"), model_report["counts_315"].get("noct"))
        )

    octree_equal = all(row["octree_f64_equal"] for row in report["models"].values())
    hfield_equal = all(row["hfield_equal"] for row in report["models"].values())
    structure_equal = all(row["struct_equal"] for row in report["models"].values())
    overall_equal = octree_equal and hfield_equal and structure_equal
    report["overall"] = {
        "octree_f64_equal": octree_equal,
        "hfield_equal": hfield_equal,
        "struct_equal": structure_equal,
        "all_equal": overall_equal,
        "decision": (
            "SDF/hfield 数值验收走位级（两版本字段全等）"
            if overall_equal
            else "G3/G4/G5 统一用 3.15 导出的 oct_* 夹具作两端输入；G6 unmet 写明产品 3.12 编译 octree 与参考 3.15 不同"
        ),
    }
    out = FIXTURE_DIR / "octree-parity.json"
    out.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("overall:", json.dumps(report["overall"], ensure_ascii=False))
    print("wrote", out.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
