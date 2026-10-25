"""G2 主参考：用 pinned mujoco_warp 3.15（warp 1.15.0，CPU 后端）在相同 f32 输入下重采 5 个 CONVEX case。

性质：authoring_only（不进产品路径、产品测试不跑 Python）；warp CPU 后端，不占 GPU。
输入：与交付参考相同的 MJCF 模型 + 参考 JSON 的 qpos（f32 位型原样）。
输出：`upstream-mjwarp-f32.json`（每 case 两次捕获 + 17 字段，附 provenance）。
"""

import hashlib
import json
import struct
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = ROOT / "docs/development-evidence/g10-p5-gjk-epa"
CACHE = ROOT / "target/g10-p5-g2/warp-cache-cpu"
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
CASES = [
    ("cylinder-box", "fixtures/g10-p5/models/cylinder-box.xml", "fixtures/g10-p5/references/cylinder-box.json", True),
    ("ellipsoid-box", "fixtures/g10-p5/models/ellipsoid-box.xml", "fixtures/g10-p5/references/ellipsoid-box.json", True),
    ("mesh-mesh", "fixtures/g10-p5/models/mesh-mesh.xml", "fixtures/g10-p5/references/mesh-mesh.json", True),
    ("convex-mesh", "fixtures/mjwarp-complex-physics/convex-mesh.xml", "fixtures/mjwarp-complex-physics/reference.json", False),
    ("degenerate-mesh", "fixtures/mjwarp-complex-physics/degenerate-mesh.xml", "fixtures/mjwarp-complex-physics/reference.json", False),
]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def load_qpos(reference_path, standalone, name):
    value = json.loads(Path(reference_path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(case for case in value["collision_cases"] if case["name"] == name)
    values = value["input"]["qpos"]["values"]
    return np.asarray(values, dtype=np.float32)


def capture(xml_path, qpos, nworld):
    host = mujoco.MjModel.from_xml_path(str(ROOT / xml_path))
    model = mjw.put_model(host)
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
    return {"count": count, "fields": fields, "geom_xpos": data.geom_xpos.numpy().tolist(), "geom_xmat": data.geom_xmat.numpy().reshape(data.geom_xmat.shape[0], data.geom_xmat.shape[1], 9).tolist()}


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.set_device("cpu")
    print("device:", wp.get_device())

    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-gjk-epa",
        "kind": "g2-upstream-mjwarp-f32-cpu-reference",
        "purpose": "G2 主参考：同配方上游 f32（warp CPU 后端）接触输出；ours vs 此参考为通过判据",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "provenance": {
            "versions": {"python": ".".join(map(str, __import__("sys").version_info[:3])),
                         "mujoco": mujoco.__version__, "warp": wp.__version__,
                         "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
                         "numpy": np.__version__},
            "device": str(wp.get_device()),
            "authoring_script": "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py",
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
        "cases": {},
    }
    for name, xml_path, reference_path, standalone in CASES:
        qpos_flat = load_qpos(reference_path, standalone, name)
        host_probe = mujoco.MjModel.from_xml_path(str(ROOT / xml_path))
        nworld = qpos_flat.size // host_probe.nq
        qpos = qpos_flat.reshape(nworld, host_probe.nq)
        first = capture(xml_path, qpos, nworld)
        second = capture(xml_path, qpos, nworld)
        raw_identical = first == second
        report["cases"][name] = {
            "xml": xml_path,
            "xml_sha256": sha256(ROOT / xml_path),
            "qpos": qpos_flat.tolist(),
            "qpos_bits": [bits32([value])[0] for value in qpos_flat],
            "two_captures_bitwise_identical": raw_identical,
            "output": first,
        }
        print(f"{name}: count={first['count']} two_captures={raw_identical}")

    (EVIDENCE / "upstream-mjwarp-f32.json").write_text(
        json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    print("wrote", EVIDENCE / "upstream-mjwarp-f32.json")


if __name__ == "__main__":
    main()
