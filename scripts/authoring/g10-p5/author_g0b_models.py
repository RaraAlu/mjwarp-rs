#!/usr/bin/env python
"""G0b：生成 GJK/EPA 直达参考补采所需的模型 XML（纯 CPU、可复现）。

设计约束（Arch 2026-10-25）：
  - `default_state` 的 offset 只作用于**第一个 freejoint 前 3 维**，所以「要被移动的 body」
    必须是 XML 中第一个 freejoint body；
  - 每 case worlds=2：world0=qpos0、world1=qpos0+offset；接触/深穿透/分离用独立 case 表达；
  - 模型全部离线 authoring（不进产品构建/测试/门禁）。

输出：fixtures/g10-p5/models/<name>.xml
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT_DIR = ROOT / "fixtures/g10-p5/models"

COMMON_OPTION = '<option gravity="0 0 0"><flag nativeccd="disable" multiccd="enable"/></option>'
SDF_OPTION = '<option gravity="0 0 0" sdf_iterations="10" sdf_initpoints="40"/>'
CUBE_MESH = (
    '<asset><mesh name="cube" vertex="1 1 1  1 1 -1  1 -1 1  1 -1 -1  '
    '-1 1 1  -1 1 -1  -1 -1 1  -1 -1 -1"/></asset>'
)
TETRA_MESH = '<asset><mesh name="tetra" vertex="0 0 0  0.2 0 0  0 0.2 0  0 0 0.2"/></asset>'
HFIELD_ASSET = (
    '<asset><hfield name="terrain" nrow="3" ncol="3" size="1 1 0.1 0.1" '
    'elevation="0 0.25 0  0.25 0.5 0.25  0 0.25 0"/></asset>'
)

# (name, offset, xml) —— offset 写进清单供捕获脚本使用
MODELS = [
    (
        "sphere-box-shallow",
        (0.005, 0.0, 0.0),
        f"""<mujoco model="g0b_sphere_box_shallow">
  {COMMON_OPTION}
  <worldbody>
    <body name="sphere_body" pos="0.05 0 0.1">
      <freejoint/>
      <geom name="sphere" type="sphere" size="0.1" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "sphere-box-deep",
        (0.0, 0.0, -0.06),
        f"""<mujoco model="g0b_sphere_box_deep">
  {COMMON_OPTION}
  <worldbody>
    <body name="sphere_body" pos="0.02 0 0.02">
      <freejoint/>
      <geom name="sphere" type="sphere" size="0.1" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "capsule-box",
        (0.01, 0.0, -0.01),
        f"""<mujoco model="g0b_capsule_box">
  {COMMON_OPTION}
  <worldbody>
    <body name="capsule_body" pos="0.04 0 0.14">
      <freejoint/>
      <geom name="capsule" type="capsule" size="0.05 0.12" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "cylinder-box",
        (0.01, 0.01, 0.0),
        f"""<mujoco model="g0b_cylinder_box">
  {COMMON_OPTION}
  <worldbody>
    <body name="cylinder_body" pos="0.04 0.04 0.1">
      <freejoint/>
      <geom name="cylinder" type="cylinder" size="0.08 0.1" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "ellipsoid-box",
        (0.01, 0.0, 0.0),
        f"""<mujoco model="g0b_ellipsoid_box">
  {COMMON_OPTION}
  <worldbody>
    <body name="ellipsoid_body" pos="0.04 0 0.1">
      <freejoint/>
      <geom name="ellipsoid" type="ellipsoid" size="0.1 0.06 0.05" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "hfield-sphere",
        (0.02, 0.0, -0.01),
        f"""<mujoco model="g0b_hfield_sphere">
  <option gravity="0 0 0"/>
  {HFIELD_ASSET}
  <worldbody>
    <body name="sphere_body" pos="0.05 0 0.15">
      <freejoint/>
      <geom name="sphere" type="sphere" size="0.1" mass="1"/>
    </body>
    <geom name="terrain" type="hfield" hfield="terrain"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "hfield-capsule",
        (0.02, 0.0, 0.0),
        f"""<mujoco model="g0b_hfield_capsule">
  <option gravity="0 0 0"/>
  {HFIELD_ASSET}
  <worldbody>
    <body name="capsule_body" pos="0.04 0 0.2">
      <freejoint/>
      <geom name="capsule" type="capsule" size="0.05 0.12" mass="1"/>
    </body>
    <geom name="terrain" type="hfield" hfield="terrain"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "sdf-box",
        (0.0, 0.0, -0.04),
        f"""<mujoco model="g0b_sdf_box">
  {SDF_OPTION}
  {CUBE_MESH}
  <worldbody>
    <body name="sdf_body" pos="0 0 0.3">
      <freejoint/>
      <geom name="volume" type="sdf" mesh="cube" mass="1"/>
    </body>
    <geom name="box" type="box" size="0.1 0.08 0.06"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "mesh-mesh",
        (0.01, 0.01, 0.0),
        f"""<mujoco model="g0b_mesh_mesh">
  <option gravity="0 0 0"><flag nativeccd="disable"/></option>
  {TETRA_MESH}
  <worldbody>
    <body name="right_body" pos="0.05 0.05 0.05">
      <freejoint/>
      <geom name="right" type="mesh" mesh="tetra" mass="1"/>
    </body>
    <geom name="left" type="mesh" mesh="tetra"/>
  </worldbody>
</mujoco>
""",
    ),
    (
        "box-box-margin",
        (0.005, -0.005, 0.0),
        f"""<mujoco model="g0b_box_box_margin">
  <option gravity="0 0 0"><flag nativeccd="disable" multiccd="disable"/></option>
  <worldbody>
    <body name="lower" pos="0 0 0.08">
      <freejoint/>
      <geom name="lower_box" type="box" size="0.1 0.08 0.07" mass="1" margin="0.01"/>
    </body>
    <body name="upper" pos="0.04 0.02 0.17" euler="7 11 13">
      <geom name="upper_box" type="box" size="0.09 0.07 0.08" margin="0.01"/>
    </body>
  </worldbody>
</mujoco>
""",
    ),
]


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    index = {"schema": 1, "kind": "g0b-model-index", "models": []}
    for name, offset, xml in MODELS:
        path = OUT_DIR / (name + ".xml")
        path.write_text(xml, encoding="utf-8", newline="\n")
        index["models"].append({
            "name": name,
            "xml": str(path.relative_to(ROOT)).replace("\\", "/"),
            "offset": list(offset),
        })
        print("wrote", path.relative_to(ROOT))
    index_path = OUT_DIR / "index.json"
    index_path.write_text(json.dumps(index, ensure_ascii=False, indent=1), encoding="utf-8")
    print("wrote", index_path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
