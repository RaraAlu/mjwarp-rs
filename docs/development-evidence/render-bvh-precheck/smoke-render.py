"""Precheck-only smoke: produce a frozen-style render reference.

Not part of the product build or test chain. Run with the pinned authoring venv:
  target/mjwarp-parallel/venv/Scripts/python.exe \
    docs/development-evidence/render-bvh-precheck/smoke-render.py \
    docs/development-evidence/render-bvh-precheck/smoke-render.json
"""

import hashlib
import json
import os
import sys
import tempfile

import numpy as np
import warp as wp

wp.config.kernel_cache_dir = os.path.join(tempfile.gettempdir(), "mjwarp-render-precheck-cache")

import mujoco  # noqa: E402
import mujoco_warp as mjw  # noqa: E402
from mujoco_warp import test_data  # noqa: E402

XML = """
<mujoco>
  <asset>
    <texture name="grid" type="2d" builtin="checker" width="32" height="32"
             rgb1="0.8 0.1 0.1" rgb2="0.1 0.1 0.8"/>
    <material name="grid" texture="grid" texrepeat="1 1"/>
  </asset>
  <worldbody>
    <light pos="0 0 3" dir="0 0 -1" diffuse="1 1 1"/>
    <camera name="cam" pos="0 -3 1.2" xyaxes="1 0 0 0 0.4 0.9" resolution="32 32" output="rgb depth"/>
    <geom type="plane" size="3 3 0.1" material="grid"/>
    <geom type="box" pos="0 0 0.7" size="0.5 0.3 0.4" rgba="0.2 0.9 0.2 1"/>
    <geom type="sphere" pos="0.9 0 0.5" size="0.3" rgba="0.9 0.9 0.2 1"/>
  </worldbody>
</mujoco>
"""


def sha256(array: np.ndarray) -> str:
    return hashlib.sha256(np.ascontiguousarray(array).tobytes()).hexdigest()


mjm, _, m, d = test_data.fixture(xml=XML)
rc = mjw.create_render_context(
    mjm, cam_res=(32, 32), render_rgb=True, render_depth=True, render_seg=True
)
mjw.render(m, d, rc)

rgb = rc.rgb_data.numpy()
depth = rc.depth_data.numpy()
seg = rc.seg_data.numpy()
device = wp.get_device()

summary = {
    "schema": 1,
    "kind": "render-bvh-precheck-smoke",
    "environment": {
        "python": sys.version.split()[0],
        "warp": wp.__version__,
        "mujoco": mujoco.__version__,
        "mujoco_warp": mjw.__version__,
        "gpu": device.name,
        "arch": device.arch,
    },
    "fixture": {
        "xml": XML.strip(),
        "nmesh": int(mjm.nmesh),
        "ntex": int(mjm.ntex),
        "ncam": int(mjm.ncam),
        "nlight": int(mjm.nlight),
        "ngeom": int(mjm.ngeom),
    },
    "context": {
        "bvh_ngeom": int(rc.bvh_ngeom),
        "nrender": int(rc.nrender),
        "samples_per_pixel": int(rc.samples_per_pixel),
        "use_textures": bool(rc.use_textures),
        "use_fast_math": bool(rc.use_fast_math),
    },
    "outputs": {
        "rgb_shape": list(rgb.shape),
        "rgb_dtype": str(rgb.dtype),
        "rgb_min": int(rgb.min()),
        "rgb_max": int(rgb.max()),
        "rgb_sha256": sha256(rgb),
        "depth_shape": list(depth.shape),
        "depth_min": float(depth.min()),
        "depth_max": float(depth.max()),
        "depth_sha256": sha256(depth),
        "seg_shape": list(seg.shape),
        "seg_min": int(seg.min()),
        "seg_max": int(seg.max()),
        "seg_sha256": sha256(seg),
    },
}

out = sys.argv[1] if len(sys.argv) > 1 else "smoke-render.json"
with open(out, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2, sort_keys=True)
    handle.write("\n")
print(json.dumps(summary["outputs"], indent=2, sort_keys=True))
