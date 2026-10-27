"""S3: capture upstream pinned-mjwarp GPU f32 contact reference for the branch-closure fixtures.

Same recipe as B3 `capture_primitive_branches_gpu.py` and M3 mesh-mesh capture (authoring only;
product paths and product tests never run Python; deterministic `<key name="capture">` qpos;
qpos f32 bit patterns recorded per case; each case captured twice in-process). The runner calls
this twice in two independent processes with two independent warp kernel caches; the two raw
JSON files must be byte-identical. `captured_at` must be exported by the runner from the real
window start (shared by both runs so the JSON stays byte-identical).

Two groups are written to two separate reference files (both new; old references are never read
or written):
  * branch group  -> <out-dir>/upstream-branch-closure-gpu.json   (U1/U3/U4/U5/U6, acceptance group)
  * limits group  -> <out-dir>/upstream-mesh-limits-gpu.json      (U7/U8, record-only group)

Usage: python -B capture_branch_closure_gpu.py <kernel-cache-dir> <output-dir>
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
BRANCHES = ROOT / "fixtures/g10-p5-primitive-branches"
MESH_DIR = ROOT / "fixtures/g10-p5-mesh-multicontact"
CAPTURED_AT = os.environ.get("MJWARP_CAPTURE_AT", "")
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")

# (case name, fixture dir, group)
CASES = [
    ("plane-cylinder-degenerate", BRANCHES, "branch"),
    ("plane-box-corner-face-down", BRANCHES, "branch"),
    ("plane-box-corner-face-up", BRANCHES, "branch"),
    ("plane-box-corner-tilt-x", BRANCHES, "branch"),
    ("plane-box-corner-tilt-y", BRANCHES, "branch"),
    ("plane-box-corner-single", BRANCHES, "branch"),
    ("plane-capsule-lying", BRANCHES, "branch"),
    ("plane-cylinder-deep-tilt", BRANCHES, "branch"),
    ("mesh-mesh-two-assets", MESH_DIR, "branch"),
    ("mesh-limits-cone-box", MESH_DIR, "limits"),
    ("mesh-limits-prism-box", MESH_DIR, "limits"),
]
OUTPUTS = {
    "branch": "upstream-branch-closure-gpu.json",
    "limits": "upstream-mesh-limits-gpu.json",
}


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
    cache = Path(sys.argv[1])
    out_dir = Path(sys.argv[2])
    cache.mkdir(parents=True, exist_ok=True)
    out_dir.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(cache)
    wp.set_device("cuda")
    print("device:", wp.get_device())
    print("kernel_cache_dir:", cache)
    reports = {}
    for group in ("branch", "limits"):
        reports[group] = {
            "schema": 1,
            "task_id": "2026-10-27-unc3-branch-closure-s3-capture",
            "kind": f"g10-p5-branch-closure-{group}-upstream-mjwarp-f32-gpu-reference",
            "purpose": (
                "S3 authoring: upstream pinned mujoco_warp GPU (fused) contact fields for the branch-closure "
                "fixtures; acceptance group (U1/U3/U4/U5/U6)" if group == "branch" else
                "S3 authoring: upstream pinned mujoco_warp GPU contact fields for the mesh limit group "
                "(U7/U8); record-only, not an acceptance basis"
            ),
            "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
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
                "authoring_script": "scripts/authoring/g10-p5/capture_branch_closure_gpu.py",
                "authoring_script_sha256": sha256(__file__),
                "authoring_only": True,
                "reads_product_tree": False,
                "product_python_dependency": False,
                "captured_at": CAPTURED_AT,
                "kernel_cache_parent": str(cache.parent),
                "kernel_cache_note": "per-run cache dir is recorded in the run stdout log; both runs use the same parent and independent child caches",
            },
            "cases": {},
        }
    for name, fixture_dir, group in CASES:
        xml_path = fixture_dir / f"{name}.xml"
        host_probe = mujoco.MjModel.from_xml_path(str(xml_path))
        key_id = host_probe.key("capture").id
        qpos_flat = np.asarray(host_probe.key_qpos[key_id], dtype=np.float32)
        nworld = 1
        qpos = qpos_flat.reshape(nworld, host_probe.nq)
        first = capture(xml_path, qpos, nworld)
        second = capture(xml_path, qpos, nworld)
        reports[group]["cases"][name] = {
            "xml": xml_path.relative_to(ROOT).as_posix(),
            "xml_sha256": sha256(xml_path),
            "qpos": qpos_flat.tolist(),
            "qpos_bits": [bits32([value])[0] for value in qpos_flat],
            "nworld": nworld,
            "two_captures_bitwise_identical": first == second,
            "output": first,
        }
        print(f"{group}/{name}: count={first['count']} two_captures={first == second}")
    for group, report in reports.items():
        target = out_dir / OUTPUTS[group]
        target.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        print("wrote", target)


if __name__ == "__main__":
    main()
