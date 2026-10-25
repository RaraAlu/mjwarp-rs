"""G3 主参考：用 pinned mujoco_warp 3.15（warp 1.15.0，CPU 后端）重采 sdf-box 的体积 SDF 接触。

性质：authoring_only（不进产品路径、产品测试不跑 Python）；warp CPU 后端，不占 GPU。
输入：与 G0b 参考相同的 MJCF 模型 + `fixtures/g10-p5/references/sdf-box.json` 的 qpos（f32 位型原样）。
输出：`fixtures/g10-p5/sdf-f32/sdf-box-cpu.json` + 三个 octree 二进制数组
（`oct_aabb.f32`、`oct_child.i32`、`oct_coeff.f32`，小端；JSON 记 dtype/count/sha256）。
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
OUT = ROOT / "fixtures/g10-p5/sdf-f32"
CACHE = ROOT / "target/g10-p5-g3/warp-cache-cpu"
REFERENCE = ROOT / "fixtures/g10-p5/references/sdf-box.json"
XML = ROOT / "fixtures/g10-p5/models/sdf-box.xml"

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
SMALL_ARRAYS = ("geom_type", "geom_size", "geom_dataid", "geom_aabb", "mesh_octadr")
BIN_ARRAYS = ("oct_aabb", "oct_child", "oct_coeff")


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


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

    reference = json.loads(REFERENCE.read_text(encoding="utf-8"))
    qpos_flat = np.asarray(reference["input"]["qpos"]["values"], dtype=np.float32)
    host = mujoco.MjModel.from_xml_path(str(XML))
    nworld = qpos_flat.size // host.nq
    qpos = qpos_flat.reshape(nworld, host.nq)
    model = mjw.put_model(host)

    first = capture(host, qpos, nworld, model)
    second = capture(host, qpos, nworld, model)
    raw_identical = first == second
    print("count:", first["count"], "two_captures:", raw_identical)

    model_arrays = {}
    for name in SMALL_ARRAYS:
        array = squeeze(getattr(model, name).numpy())
        model_arrays[name] = array.tolist()
        print(name, array.shape)
    model_files = {}
    for name in BIN_ARRAYS:
        array = getattr(model, name).numpy()
        dtype = "i32" if array.dtype == np.int32 else "f32"
        path = OUT / f"{name}.{dtype}"
        path.write_bytes(array.astype("<i4" if dtype == "i32" else "<f4").tobytes())
        model_files[name] = {
            "path": f"fixtures/g10-p5/sdf-f32/{path.name}",
            "dtype": dtype,
            "count": int(array.size),
            "shape": list(array.shape),
            "sha256": sha256(path),
        }
        print(name, array.shape, dtype)

    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-sdf-host",
        "kind": "g3-upstream-mjwarp-f32-cpu-reference",
        "purpose": "G3 主参考：同配方上游 f32（warp CPU 后端）体积 SDF 接触输出；ours vs 此参考为通过判据",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "case": {
            "name": "sdf-box",
            "xml": "fixtures/g10-p5/models/sdf-box.xml",
            "xml_sha256": sha256(XML),
            "reference": "fixtures/g10-p5/references/sdf-box.json",
            "qpos": qpos_flat.tolist(),
            "qpos_bits": [bits32([value])[0] for value in qpos_flat],
            "worlds": nworld,
            "sdf_iterations": int(host.opt.sdf_iterations),
            "sdf_initpoints": int(host.opt.sdf_initpoints),
            "two_captures_bitwise_identical": raw_identical,
            "output": first,
        },
        "model": {
            "arrays": model_arrays,
            "files": model_files,
        },
        "provenance": {
            "versions": {
                "python": ".".join(map(str, __import__("sys").version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
                "numpy": np.__version__,
            },
            "device": str(wp.get_device()),
            "authoring_script": "scripts/authoring/g10-p5/capture_sdf_f32.py",
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
    }
    path = OUT / "sdf-box-cpu.json"
    path.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", path, path.stat().st_size)


if __name__ == "__main__":
    main()
