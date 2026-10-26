"""G10@P5 M3: capture upstream pinned-mjwarp GPU f32 reference for the mesh-mesh fixture.

Same recipe as the M1 capture script (`mesh-multicontact-m1/scripts/capture_mesh_multicontact_gpu.py`,
kept byte-unchanged): authoring only (product paths and product tests never run Python), fixture
under `fixtures/g10-p5-mesh-multicontact/`, deterministic `<key name="capture">` qpos, qpos f32 bit
patterns recorded, each case captured twice in-process. Runner calls this twice in two independent
processes with two independent warp kernel caches; the two raw JSON files must be byte-identical.
`captured_at` must be exported by the runner from the real window start (shared by both runs so the
JSON stays byte-identical).

`(MESH, MESH)` is CONVEX per `collision_driver.py` MJ_COLLISION_TABLE, so the pinned GPU path is
`convex_narrowphase` -> `ccd_kernel_builder` MULTICCD multicontact branch, exactly like the M1
mesh-box / mesh-cylinder captures. The fixture keeps `nativeccd="disable"` (target flag) and does
NOT disable MULTICCD; the CPU precheck (`precheck_mesh_mesh_cpu.py`) proved the fixture yields
nacon=4 with MULTICCD enabled vs 1 with it disabled.

Usage: python -B capture_mesh_multicontact_mesh_mesh_gpu.py <kernel-cache-dir> <output-json>
Env: MJWARP_CAPTURE_AT (ISO8601; real window start, shared across both runs)
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
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
CACHE_ROOT = ROOT / "target/m3-mesh-mesh/warp-cache"
CACHE = Path(sys.argv[1]) if len(sys.argv) > 1 else CACHE_ROOT
OUT = (
    Path(sys.argv[2])
    if len(sys.argv) > 2
    else ROOT / "docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json"
)
CAPTURED_AT = os.environ.get("MJWARP_CAPTURE_AT", "")
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
CASES = ["mesh-mesh"]


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
        "task_id": "2026-10-26-mesh-multicontact-m3-mesh-mesh",
        "kind": "m3-mesh-multicontact-upstream-mjwarp-f32-gpu-reference",
        "purpose": "M3 authoring: upstream pinned mujoco_warp GPU (fused) convex CCD multicontact fields for the mesh-mesh (MESH, MESH) fixture; not a product-precision verdict",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "dispatch": "NATIVECCD disabled; MULTICCD enabled (default). (MESH,MESH) is CONVEX per collision_driver.py MJ_COLLISION_TABLE -> convex_narrowphase -> ccd_kernel_builder multicontact branch (collision_convex.py L906-944, mesh needs mesh_polyadr>=0, L911-919)",
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
            "interpreter": {"path": sys.executable, "version": ".".join(map(str, sys.version_info[:3]))},
            "authoring_script": "docs/development-evidence/mesh-multicontact-m3/scripts/capture_mesh_multicontact_mesh_mesh_gpu.py",
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
            "captured_at": CAPTURED_AT,
            "kernel_cache_parent": CACHE.parent.relative_to(ROOT).as_posix(),
            "kernel_cache_note": "per-run cache dir is recorded in the run stdout log; both runs use the same parent and independent child caches",
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
            "ngeom": int(host_probe.ngeom),
            "geom_dataid": [int(value) for value in host_probe.geom_dataid],
            "two_captures_bitwise_identical": first == second,
            "flags": {
                "nativeccd_disabled": bool(
                    host_probe.opt.disableflags
                    & int(mujoco.mjtDisableBit.mjDSBL_NATIVECCD)
                ),
                "multiccd_enabled": not bool(
                    host_probe.opt.disableflags
                    & int(mujoco.mjtDisableBit.mjDSBL_MULTICCD)
                ),
            },
            "output": first,
        }
        print(
            f"{name}: count={first['count']} two_captures={first == second} "
            f"multiccd={report['cases'][name]['flags']['multiccd_enabled']} "
            f"geom_dataid={report['cases'][name]['geom_dataid']}"
        )
    OUT.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", OUT)


if __name__ == "__main__":
    main()
