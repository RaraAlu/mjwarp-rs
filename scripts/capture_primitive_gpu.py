"""B0: capture upstream pinned-mjwarp GPU f32 contact reference for the 10 primitive combos.

Authoring only (product paths and product tests never run Python). Each fixture already carries
`<flag nativeccd="disable"/>` and a deterministic `<key name="capture">` qpos; qpos f32 bit
patterns are recorded per case. Runner calls this twice in two independent processes with two
independent warp kernel caches; the two raw JSON files must be byte-identical.

Usage: python -B capture_primitive_gpu.py <kernel-cache-dir> <output-json>
Env: MJWARP_CAPTURE_AT (ISO8601; share across both runs so the JSON file is byte-identical)
"""
import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIXTURES = ROOT / "fixtures/g10-p5-primitive"
CACHE_ROOT = ROOT / "target/g10-p5-primitive-capture/warp-cache"
CACHE = Path(sys.argv[1]) if len(sys.argv) > 1 else CACHE_ROOT
OUT = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "target/g10-p5-primitive-capture/upstream-primitive-gpu.json"
CAPTURED_AT = os.environ.get("MJWARP_CAPTURE_AT", "")
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
CASES = [
    "plane-sphere", "plane-capsule", "plane-ellipsoid", "plane-cylinder", "plane-box", "plane-mesh",
    "sphere-sphere", "sphere-capsule", "sphere-cylinder", "capsule-capsule",
]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return tuple(struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values)


def gpu_info():
    raw = subprocess.check_output(
        ["nvidia-smi", "--query-gpu=name,memory.total,driver_version,compute_cap", "--format=csv,noheader"],
        text=True,
    ).strip()
    name, memory_total, driver, capability = [part.strip() for part in raw.split(",")]
    return {"name": name, "memory_total": memory_total, "driver": driver, "compute_capability": capability}


def capture(xml_path, qpos, nworld):
    host = mujoco.MjModel.from_xml_path(str(xml_path))
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
    return {
        "count": count,
        "fields": fields,
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(data.geom_xmat.shape[0], data.geom_xmat.shape[1], 9).tolist(),
    }


def main():
    CACHE.mkdir(parents=True, exist_ok=True)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.set_device("cuda")
    print("device:", wp.get_device())
    print("kernel_cache_dir:", CACHE)
    report = {
        "schema": 1,
        "task_id": "2026-10-26-g10-p5-primitive-capture",
        "kind": "g10-p5-primitive-upstream-mjwarp-f32-gpu-reference",
        "purpose": "B0 authoring: upstream pinned mujoco_warp GPU (fused) contact fields for the 10 primitive combos; not a product-precision verdict",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "dispatch": "nativeccd=disable -> (BOX,BOX) overridden to PRIMITIVE; 10 combos route PRIMITIVE per frozen collision_driver.py L47-81/L908-915 and _PRIMITIVE_COLLISIONS L1335-1349",
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
                "numpy": np.__version__,
            },
            "device": "cuda:0",
            "gpu": gpu_info(),
            "interpreter": {"path": sys.executable, "version": ".".join(map(str, sys.version_info[:3]))},            "authoring_script": "scripts/capture_primitive_gpu.py",
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
            "captured_at": CAPTURED_AT,
            "kernel_cache_root": CACHE_ROOT.relative_to(ROOT).as_posix(),
        },
        "cases": {},
    }
    for name in CASES:
        xml_path = FIXTURES / f"{name}.xml"
        host_probe = mujoco.MjModel.from_xml_path(str(xml_path))
        key_id = host_probe.key("capture").id
        qpos_flat = np.asarray(host_probe.key_qpos[key_id], dtype=np.float32)
        nworld = 1
        qpos = qpos_flat.reshape(nworld, host_probe.nq)
        first = capture(xml_path, qpos, nworld)
        second = capture(xml_path, qpos, nworld)
        report["cases"][name] = {
            "xml": xml_path.relative_to(ROOT).as_posix(),
            "xml_sha256": sha256(xml_path),
            "qpos": qpos_flat.tolist(),
            "qpos_bits": [bits32([value])[0] for value in qpos_flat],
            "nworld": nworld,
            "two_captures_bitwise_identical": first == second,
            "output": first,
        }
        print(f"{name}: count={first['count']} two_captures={first == second}")
    OUT.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", OUT)


if __name__ == "__main__":
    main()