"""QA 附加实验：体积 SDF 相对旋转约定判定（非单位旋转对）。

目的：验证上游 `collision_sdf.py` 的 `params.rel_mat = wp.transpose(rot1) * rot2`
（行优先 `aᵀb`）与产品 `sdf/descent.rs::relative_matrix`（实测为 `bᵀa`）在
**非单位相对旋转**下是否导致接触不一致。

做法：用 `fixtures/g10-p5/models/sdf-box.xml` 的模型与 octree 体积数据，只改 geom 位姿/旋转
（SDF 几何是局部体积，旋转不改 octree 数组；本脚本会重采 octree 并核对与 fixture 二进制逐字节一致）：
- `sdf-rot-ident`：两侧单位旋转（对照，应与 G3 一致）；
- `sdf-rot-sdf`：SDF 侧旋转、box 单位；
- `sdf-rot-box`：box 侧旋转、SDF 单位；
- `sdf-rot-both`：两侧都旋转。
每场景 2 world、`sdf_initpoints=40`；上游 CPU 后端两次独立捕获。

不改产品代码、不读产品源码；输出 `qa-sdf-rotation-cases.json`。
"""

import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
WORK = ROOT / "target" / "qa-g4"
SDF_DIR = ROOT / "fixtures/g10-p5/sdf-f32"
BASE_XML = (ROOT / "fixtures/g10-p5/models/sdf-box.xml").read_text(encoding="utf-8")

FLOAT_FIELDS = ("dist", "pos", "frame")
INT_FIELDS = ("geom", "worldid")
SMALL_ARRAYS = ("geom_type", "geom_size", "geom_dataid", "geom_aabb", "mesh_octadr")
BIN_ARRAYS = ("oct_aabb", "oct_child", "oct_coeff")


def sha256_path(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def squeeze(array):
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def capture(host, qpos, nworld, model):
    data = mjw.make_data(host, nworld=nworld, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((nworld, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = getattr(data.contact, name).numpy()
        fields[name] = array[:count].reshape(count, -1).tolist() if count else []
    return {
        "count": count,
        "fields": fields,
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(nworld, host.ngeom * 9).tolist(),
    }


@wp.kernel
def _upstream_rel_mat(xmat: wp.array2d(dtype=wp.mat33), out: wp.array(dtype=wp.mat33)):
    """upstream `collision_sdf.py:724` 的原式：params.rel_mat = transpose(rot1) * rot2。"""
    i = wp.tid()
    out[i] = wp.transpose(xmat[i, 0]) * xmat[i, 1]


def warp_relative_matrices(host, qpos, nworld, model):
    """直接在 warp 设备数组上算 rel_mat，得到上游实际使用的位型（供产品对照）。"""
    data = mjw.make_data(host, nworld=nworld, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((nworld, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    out = wp.empty(nworld, dtype=wp.mat33)
    wp.launch(_upstream_rel_mat, dim=nworld, inputs=[data.geom_xmat, out])
    wp.synchronize_device()
    return out.numpy().reshape(nworld, 9).tolist()


def rot_z(angle):
    (s, c) = (np.sin(angle), np.cos(angle))
    return [float(c), float(-s), 0.0, float(s), float(c), 0.0, 0.0, 0.0, 1.0]


def quat_z(angle):
    return [float(np.cos(angle / 2.0)), 0.0, 0.0, float(np.sin(angle / 2.0))]


def quat_axis(axis, angle):
    axis = np.asarray(axis, dtype=np.float64)
    axis = axis / np.linalg.norm(axis)
    half = angle / 2.0
    return [float(np.cos(half))] + [float(component * np.sin(half)) for component in axis]


# (name, note, box 属性串, 每 world 的 (pos, quat))
SCENES = (
    ("sdf-rot-ident", "两侧单位旋转（对照）", "",
     ([0.0, 0.0, 0.30], [1.0, 0.0, 0.0, 0.0]), ([0.0, 0.0, 0.26], [1.0, 0.0, 0.0, 0.0])),
    ("sdf-rot-sdf", "SDF 侧旋转、box 单位", "",
     ([0.0, 0.0, 0.30], quat_z(0.7)), ([0.0, 0.0, 0.26], quat_axis([0.3, -0.5, 0.8], 0.9))),
    ("sdf-rot-box", "box 侧旋转、SDF 单位", ' euler="25 10 30"',
     ([0.0, 0.0, 0.30], [1.0, 0.0, 0.0, 0.0]), ([0.0, 0.0, 0.26], [1.0, 0.0, 0.0, 0.0])),
    ("sdf-rot-both", "两侧都旋转", ' euler="25 10 30"',
     ([0.0, 0.0, 0.30], quat_z(0.7)), ([0.0, 0.0, 0.26], quat_axis([0.3, -0.5, 0.8], 0.9))),
    # 与 QA helper 核对同一旋转对：SDF 侧 Rz(0.7)、box 侧 Rx(0.4)（euler 22.9183 度 = 0.4 rad）
    ("sdf-rot-rz-rx", "SDF 侧 Rz(0.7) + box 侧 Rx(0.4)，即 helper 核对的旋转对", ' euler="22.918312 0 0"',
     ([0.0, 0.0, 0.30], quat_z(0.7)), ([0.0, 0.0, 0.30], quat_z(0.7))),
)


def build_xml(box_attrs):
    xml = BASE_XML
    assert '<geom name="box" type="box" size="0.1 0.08 0.06"/>' in xml
    xml = xml.replace('<geom name="box" type="box" size="0.1 0.08 0.06"/>',
                      f'<geom name="box" type="box" size="0.1 0.08 0.06"{box_attrs}/>')
    return xml


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    scenes_dir = WORK / "sdf-rotation-scenes"
    scenes_dir.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.set_device("cpu")
    print("device:", wp.get_device())

    out = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-sdf-relative-rotation-capture",
        "purpose": "判定非单位相对旋转下上游 aᵀb 与产品 bᵀa 是否导致 SDF 接触不一致",
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
                "numpy": np.__version__,
            },
            "device": str(wp.get_device()),
            "script_sha256": sha256_path(Path(__file__)),
            "reads_product_tree": False,
        },
        "oct_files": {},
        "scenes": [],
    }

    # octree 数组一次采集（所有场景同一体积），与 fixture 二进制逐字节比对。
    base_xml_path = scenes_dir / "base.xml"
    base_xml_path.write_text(BASE_XML, encoding="utf-8")
    base_host = mujoco.MjModel.from_xml_path(str(base_xml_path))
    base_model = mjw.put_model(base_host)
    for name in BIN_ARRAYS:
        array = getattr(base_model, name).numpy()
        dtype = "i32" if array.dtype == np.int32 else "f32"
        fresh_path = WORK / f"fresh-{name}.{dtype}"
        fresh_path.write_bytes(array.astype("<i4" if dtype == "i32" else "<f4").tobytes())
        fixture_path = SDF_DIR / f"{name}.{dtype}"
        fresh_sha = sha256_path(fresh_path)
        fixture_sha = sha256_path(fixture_path)
        out["oct_files"][name] = {
            "fixture_path": f"fixtures/g10-p5/sdf-f32/{fixture_path.name}",
            "fresh_count": int(array.size),
            "fresh_sha256": fresh_sha,
            "fixture_sha256": fixture_sha,
            "bitwise_equal_to_fixture": fresh_sha == fixture_sha,
        }
        print(f"oct {name}: fresh==fixture {fresh_sha == fixture_sha}")

    for name, note, box_attrs, world0, world1 in SCENES:
        xml = build_xml(box_attrs)
        path = scenes_dir / f"{name}.xml"
        path.write_text(xml, encoding="utf-8")
        host = mujoco.MjModel.from_xml_path(str(path))
        qpos_flat = np.asarray([*world0[0], *world0[1], *world1[0], *world1[1]], dtype=np.float32)
        assert qpos_flat.size == 2 * host.nq, (name, qpos_flat.size, host.nq)
        nworld = 2
        qpos = qpos_flat.reshape(nworld, host.nq)
        model = mjw.put_model(host)
        first = capture(host, qpos, nworld, model)
        second = capture(host, qpos, nworld, model)
        arrays = {key: squeeze(getattr(model, key).numpy()).tolist() for key in SMALL_ARRAYS}
        record = {
            "name": name,
            "note": note,
            "xml": xml,
            "xml_sha256": sha256_path(path),
            "qpos": qpos_flat.tolist(),
            "worlds": nworld,
            "sdf_iterations": int(host.opt.sdf_iterations),
            "sdf_initpoints": int(host.opt.sdf_initpoints),
            "two_captures_bitwise_identical": first == second,
            "rel_mat_warp": warp_relative_matrices(host, qpos, nworld, model),
            "output": first,
            "model": {"arrays": arrays},
        }
        out["scenes"].append(record)
        print(f"{name:<16} count={first['count']} two_captures="
              f"{record['two_captures_bitwise_identical']} initpoints={host.opt.sdf_initpoints}")

    path = EVIDENCE / "qa-sdf-rotation-cases.json"
    path.write_text(json.dumps(out, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", path, path.stat().st_size)


if __name__ == "__main__":
    main()
