"""QA independent recompute of fixture qpos f32 bits via pinned mujoco (CPU only; no warp/CUDA import)."""
import json
import struct
import sys
from pathlib import Path

import mujoco

ROOT = Path(r"D:\Rust\mjwarp-rs")
CASES = ["plane-sphere", "plane-capsule", "plane-ellipsoid", "plane-cylinder", "plane-box",
         "plane-mesh", "sphere-sphere", "sphere-capsule", "sphere-cylinder", "capsule-capsule"]
out = {"mujoco_version": mujoco.__version__, "interpreter": sys.executable, "python": ".".join(map(str, sys.version_info[:3])), "cases": {}}
for name in CASES:
    path = ROOT / "fixtures/g10-p5-primitive" / (name + ".xml")
    model = mujoco.MjModel.from_xml_path(str(path))
    key = model.key("capture")
    qpos = model.key_qpos[key.id].astype("float32")
    bits = [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in qpos]
    xmldoc = Path(path).read_bytes()
    import hashlib
    out["cases"][name] = {
        "qpos": [float(v) for v in qpos],
        "qpos_bits": bits,
        "xml_sha256": hashlib.sha256(xmldoc).hexdigest(),
        "nativeccd_disabled": bool(model.opt.disableflags & mujoco.mjtDisableBit.mjDSBL_NATIVECCD),
        "nq": int(model.nq),
        "ngeom": int(model.ngeom),
    }
target = ROOT / "target/qa-primitive/mujoco-qpos.json"
target.write_text(json.dumps(out, indent=1), encoding="utf-8")
print("cases", len(out["cases"]))
print("mujoco", out["mujoco_version"])
