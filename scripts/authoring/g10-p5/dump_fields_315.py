#!/usr/bin/env python
"""G0b：用 pinned MuJoCo 3.15 生成参考侧 MJB 并 dump 碰撞相关字段（原始小端字节）。

OFFLINE ONLY：不进产品构建/测试/门禁；不初始化 warp/CUDA；不占 GPU。

产物（与 3.12 侧 `dump_collision_fields_312.cpp` 同格式，便于逐字节 parity）：
  fixtures/g10-p5/mjb-315/<model>.mjb
  fixtures/g10-p5/dump-315/<model>/<field>.bin + fields.json

字段（两层 parity）：
  ① f64 位型：oct_aabb / oct_coeff / hfield_size / geom_size / geom_margin
  ② f32 位型：hfield_data
  结构：oct_depth / oct_child / mesh_octadr / mesh_octnum / hfield_nrow / hfield_ncol /
        hfield_adr / opt_sdf / geom_type / geom_dataid
"""

import hashlib
import io
import json
import struct
import sys
from pathlib import Path

import mujoco

ROOT = Path(__file__).resolve().parents[3]
FIXTURE_DIR = ROOT / "fixtures/g10-p5"
MODELS = [
    ("hfield-box", "fixtures/mjwarp-complex-physics/hfield-box.xml"),
    ("sdf-cylinder", "fixtures/mjwarp-complex-physics/sdf-cylinder.xml"),
]

DTYPE_BYTES = {"i32": 4, "f32": 4, "f64": 8, "i64": 8}


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_field(out_dir: Path, name: str, dtype: str, values) -> dict:
    """按字段 dtype 写原始小端字节（int32/float32/float64/int64）。"""
    count = len(values)
    if dtype == "i32":
        raw = struct.pack("<%di" % count, *[int(v) for v in values])
    elif dtype == "i64":
        raw = struct.pack("<%dq" % count, *[int(v) for v in values])
    elif dtype == "f32":
        raw = struct.pack("<%df" % count, *[float(v) for v in values])
    elif dtype == "f64":
        raw = struct.pack("<%dd" % count, *[float(v) for v in values])
    else:
        raise SystemExit("unknown dtype %s" % dtype)
    assert len(raw) == count * DTYPE_BYTES[dtype]
    path = out_dir / (name + ".bin")
    path.write_bytes(raw)
    return {"name": name, "dtype": dtype, "count": count, "file": path.name}


def dump_model(name: str, xml_rel: str) -> dict:
    xml = ROOT / xml_rel
    mjb_dir = FIXTURE_DIR / "mjb-315"
    dump_dir = ROOT / "target/g10-p5/dump-315" / name
    mjb_dir.mkdir(parents=True, exist_ok=True)
    dump_dir.mkdir(parents=True, exist_ok=True)
    mjb = mjb_dir / (name + ".mjb")

    model = mujoco.MjModel.from_xml_path(str(xml))
    mujoco.mj_saveModel(model, str(mjb))
    reloaded = mujoco.MjModel.from_binary_path(str(mjb))
    assert reloaded.ngeom == model.ngeom, "同版本重载校验失败"

    noct = int(model.noct)
    ngeom = int(model.ngeom)
    nhfield = int(model.nhfield)
    nmesh = int(model.nmesh)
    fields = []
    fields.append(write_field(dump_dir, "counts", "i64",
                              [ngeom, nhfield, int(model.nhfielddata), nmesh, noct, int(model.nbody)]))
    fields.append(write_field(dump_dir, "nq_nv", "i64", [int(model.nq), int(model.nv)]))
    # hfield
    fields.append(write_field(dump_dir, "hfield_nrow", "i32", model.hfield_nrow.reshape(-1)))
    fields.append(write_field(dump_dir, "hfield_ncol", "i32", model.hfield_ncol.reshape(-1)))
    fields.append(write_field(dump_dir, "hfield_adr", "i32", model.hfield_adr.reshape(-1)))
    fields.append(write_field(dump_dir, "hfield_size", "f64", model.hfield_size.reshape(-1)))
    fields.append(write_field(dump_dir, "hfield_data", "f32", model.hfield_data.reshape(-1)))
    # octree（volume SDF）
    fields.append(write_field(dump_dir, "oct_depth", "i32", model.oct_depth.reshape(-1)))
    fields.append(write_field(dump_dir, "oct_child", "i32", model.oct_child.reshape(-1)))
    fields.append(write_field(dump_dir, "oct_aabb", "f64", model.oct_aabb.reshape(-1)))
    fields.append(write_field(dump_dir, "oct_coeff", "f64", model.oct_coeff.reshape(-1)))
    fields.append(write_field(dump_dir, "mesh_octadr", "i32", model.mesh_octadr.reshape(-1)))
    fields.append(write_field(dump_dir, "mesh_octnum", "i32", model.mesh_octnum.reshape(-1)))
    # SDF 选项
    fields.append(write_field(dump_dir, "opt_sdf", "i32",
                              [int(model.opt.sdf_iterations), int(model.opt.sdf_initpoints)]))
    # geom 基础字段
    fields.append(write_field(dump_dir, "geom_type", "i32", model.geom_type.reshape(-1)))
    fields.append(write_field(dump_dir, "geom_dataid", "i32", model.geom_dataid.reshape(-1)))
    fields.append(write_field(dump_dir, "geom_size", "f64", model.geom_size.reshape(-1)))
    fields.append(write_field(dump_dir, "geom_margin", "f64", model.geom_margin.reshape(-1)))

    document = {
        "schema": 1,
        "tool": "dump_fields_315",
        "mujoco_version": mujoco.__version__,
        "source_xml": xml_rel,
        "model_file": str(mjb.relative_to(ROOT)).replace("\\", "/"),
        "loaded_mjb": True,
        "counts": {"ngeom": ngeom, "nhfield": nhfield, "nmesh": nmesh, "noct": noct},
        "fields": fields,
    }
    (dump_dir / "fields.json").write_text(
        json.dumps(document, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    return {
        "name": name,
        "xml": xml_rel,
        "mjb": str(mjb.relative_to(ROOT)).replace("\\", "/"),
        "mjb_bytes": mjb.stat().st_size,
        "mjb_sha256": sha256_file(mjb),
        "mujoco": mujoco.__version__,
        "noct": noct,
        "dump_dir": str(dump_dir.relative_to(ROOT)).replace("\\", "/"),
    }


def main() -> int:
    print("pinned mujoco", mujoco.__version__)
    summary = [dump_model(name, rel) for name, rel in MODELS]
    out = ROOT / "target/g10-p5/dump-315" / "summary.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(summary, ensure_ascii=False, indent=1), encoding="utf-8")
    for row in summary:
        print("dump315 %s: noct=%d mjb_bytes=%d sha256=%s" % (
            row["name"], row["noct"], row["mjb_bytes"], row["mjb_sha256"]))
    print("wrote", out.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
