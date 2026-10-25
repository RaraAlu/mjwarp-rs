"""G4 主参考：用 pinned mujoco_warp 3.15（warp 1.15.0，CPU 后端）重采 hfield 宿主三个 case。

性质：authoring_only（不进产品路径、产品测试不跑 Python）；warp CPU 后端，不占 GPU。
输入：G0b/d04 参考的同一 MJCF 模型 + 参考 JSON 的 qpos（f32 位型原样）。
输出：`fixtures/g10-p5/hfield-f32/{hfield-box,hfield-sphere,hfield-capsule}-cpu.json`
（两次捕获、hfield/几何模型静态数组、f32 运动学与接触字段，附 provenance）。
"""

import hashlib
import json
import struct
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "fixtures/g10-p5/hfield-f32"
CACHE = ROOT / "target/g10-p5-g4/warp-cache-cpu"

# (name, xml, reference, standalone)
CASES = (
    ("hfield-box", "fixtures/mjwarp-complex-physics/hfield-box.xml",
     "fixtures/mjwarp-complex-physics/reference.json", False),
    ("hfield-sphere", "fixtures/g10-p5/models/hfield-sphere.xml",
     "fixtures/g10-p5/references/hfield-sphere.json", True),
    ("hfield-capsule", "fixtures/g10-p5/models/hfield-capsule.xml",
     "fixtures/g10-p5/references/hfield-capsule.json", True),
)

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
SMALL_ARRAYS = ("geom_type", "geom_size", "geom_dataid", "geom_margin", "geom_gap",
                "geom_rbound", "hfield_size", "hfield_nrow", "hfield_ncol", "hfield_adr",
                "hfield_data")


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def squeeze(array):
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def load_qpos(reference_path, standalone, name):
    value = json.loads(Path(reference_path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(case for case in value["collision_cases"] if case["name"] == name)
    return np.asarray(value["input"]["qpos"]["values"], dtype=np.float32)


def opt_value(model, name, default):
    value = getattr(model, "opt", None)
    if value is None:
        return default
    for key in (name,):
        if hasattr(value, key):
            item = getattr(value, key)
            try:
                array = item.numpy() if hasattr(item, "numpy") else item
                return int(np.asarray(array).reshape(-1)[0])
            except Exception:  # noqa: BLE001 - authoring 脚本容忍不同 opt 表达
                continue
    return default


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
        fields[name] = array[:count].reshape(count, -1).tolist() if array.size else []
    return {
        "count": count,
        "fields": fields,
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(nworld, host.ngeom * 9).tolist(),
    }


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.set_device("cpu")
    print("device:", wp.get_device())

    for name, xml_path, reference_path, standalone in CASES:
        qpos_flat = load_qpos(ROOT / reference_path, standalone, name)
        host = mujoco.MjModel.from_xml_path(str(ROOT / xml_path))
        nworld = qpos_flat.size // host.nq
        qpos = qpos_flat.reshape(nworld, host.nq)
        model = mjw.put_model(host)
        first = capture(host, qpos, nworld, model)
        second = capture(host, qpos, nworld, model)
        raw_identical = first == second
        arrays = {}
        opt = {}
        for opt_name in ("ccd_tolerance", "ccd_iterations"):
            try:
                value = getattr(model.opt, opt_name)
                raw_opt = value.numpy() if hasattr(value, "numpy") else value
                opt[opt_name] = float(np.asarray(raw_opt).reshape(-1)[0])
            except Exception:  # noqa: BLE001 - authoring 脚本容忍不同 opt 表达
                opt[opt_name] = None
        for array_name in SMALL_ARRAYS:
            array = squeeze(getattr(model, array_name).numpy())
            arrays[array_name] = array.tolist()
        report = {
            "schema": 1,
            "task_id": "2026-10-25-g10-p5-hfield-host",
            "kind": "g4-upstream-mjwarp-f32-cpu-reference",
            "purpose": "G4 主参考：同配方上游 f32（warp CPU 后端）hfield CONVEX 接触输出；ours vs 此参考为通过判据",
            "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
            "case": {
                "name": name,
                "xml": xml_path,
                "xml_sha256": sha256(ROOT / xml_path),
                "reference": reference_path,
                "qpos": qpos_flat.tolist(),
                "qpos_bits": [bits32([value])[0] for value in qpos_flat],
                "worlds": nworld,
                "two_captures_bitwise_identical": raw_identical,
                "output": first,
            },
            "model": {"arrays": arrays, "opt": opt},
            "provenance": {
                "versions": {
                    "python": ".".join(map(str, __import__("sys").version_info[:3])),
                    "mujoco": mujoco.__version__,
                    "warp": wp.__version__,
                    "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
                    "numpy": np.__version__,
                },
                "device": str(wp.get_device()),
                "authoring_script": "scripts/authoring/g10-p5/capture_hfield_f32.py",
                "authoring_script_sha256": sha256(__file__),
                "authoring_only": True,
                "reads_product_tree": False,
                "product_python_dependency": False,
            },
        }
        path = OUT / f"{name}-cpu.json"
        path.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        print(f"{name}: count={first['count']} two_captures={raw_identical} ngeom={host.ngeom} "
              f"nrow={arrays['hfield_nrow']} ncol={arrays['hfield_ncol']} size={arrays['hfield_size']}")


if __name__ == "__main__":
    main()
