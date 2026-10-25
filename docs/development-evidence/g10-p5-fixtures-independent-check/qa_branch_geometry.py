"""QA G0b review: check the sphere-box shallow/deep branch distinction from geometry.

mjwarp collision_gjk.py:2528-2569 shrinks a sphere to a point and, when the shrunk
GJK distance > tolerance (sphere centre outside the box), inflates to a shallow contact;
otherwise it reruns GJK+EPA (deep penetration). This script only checks the geometric
premise of those two cases, not the product code path.
"""

import json
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIX = ROOT / "fixtures/g10-p5"


def point_box_distance(point, half):
    q = np.maximum(np.abs(point) - half, 0.0)
    return float(np.linalg.norm(q))


def main():
    index = {row["name"]: row for row in json.loads((FIX / "models/index.json").read_text())["models"]}
    out = {}
    for name in ("sphere-box-shallow", "sphere-box-deep"):
        row = index[name]
        model = mujoco.MjModel.from_xml_path(str(ROOT / row["xml"]))
        data = mujoco.MjData(model)
        world = {}
        for w in (0, 1):
            qpos = model.qpos0.astype(np.float32).copy()
            qpos[0:3] += np.asarray(row["offset"], dtype=np.float32) * w
            data.qpos[:] = qpos
            mujoco.mj_kinematics(model, data)
            center = data.xpos[1].copy()
            half = model.geom_size[0]  # static box is geom 0
            d = point_box_distance(center, half)
            world[str(w)] = {
                "sphere_center": [float(v) for v in center],
                "box_half": [float(v) for v in half],
                "point_box_distance": d,
                "center_inside_box": bool(np.all(np.abs(center) < half)),
            }
        out[name] = world
        print(name, json.dumps(world))
    (ROOT / "target/qa-g0b/branch_geometry_report.json").write_text(
        json.dumps(out, indent=1), encoding="utf-8")


if __name__ == "__main__":
    main()
