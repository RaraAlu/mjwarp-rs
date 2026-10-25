"""QA 独立捕获：G4 hfield host 复核用的上游 f32（warp CPU 后端）参考。

性质：QA 自写（不复用交付 `scripts/authoring/g10-p5/capture_hfield_f32.py` 的实现），
用于 (1) 三个交付 case 的 provenance 独立重采比对；(2) 合成场景矩阵（边界 cell、
截断、filter 拒绝、margin/gap、旋转 hfield、六类对侧 geom、MJ_MAXCONPAIR 溢出、
1e-3 去重）的上游 f32 输出。

每个场景两次独立捕获（两次都新建 Data），逐位比较后写入。所有 f32 原样落盘，
下游 Rust probe/oracle 直接读本文件。

不写产品树、不读产品源码；输出到证据目录。
"""

import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
WORK = ROOT / "target" / "qa-g4"
SCENES = WORK / "scenes"

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin")
INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")
MODEL_ARRAYS = (
    "geom_type", "geom_size", "geom_dataid", "geom_margin", "geom_gap", "geom_rbound",
    "hfield_size", "hfield_nrow", "hfield_ncol", "hfield_adr", "hfield_data",
)
MESH_ARRAYS = ("mesh_vert", "mesh_vertadr", "mesh_vertnum", "mesh_graphadr")


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def f32(value):
    return float(np.float32(value))


def squeeze(array):
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def capture(host, qpos, nworld, model):
    """一次捕获：新建 Data → kinematics → collision → 读接触/overflow。"""
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
        fields[name] = array[:count].reshape(count, -1).tolist() if count else []
    return {
        "count": count,
        "fields": fields,
        "overflow": data.overflow.numpy().tolist(),
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(nworld, host.ngeom * 9).tolist(),
    }


def load_reference_qpos(path, standalone, name):
    value = json.loads((ROOT / path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(case for case in value["collision_cases"] if case["name"] == name)
        return np.asarray(value["input"]["qpos"]["values"], dtype=np.float32)
    return np.asarray(value["input"]["qpos"]["values"], dtype=np.float32)


def elevation(values):
    return " ".join(f"{value:.9g}" for value in values)


def flat(nrow, ncol):
    return [0.0] * (nrow * ncol)


def hfield_asset(nrow, ncol, size, data):
    return (
        f'<hfield name="terrain" nrow="{nrow}" ncol="{ncol}" '
        f'size="{elevation(size)}" elevation="{elevation(data)}"/>'
    )


def world_hfield_scene(nrow, ncol, size, data, geom_xml, body_pos, quat=None, extra_asset=""):
    hf_pose = "" if quat is None else f'quat="{quat}"'
    return (
        '<mujoco model="qa_hfield">\n  <option gravity="0 0 0"/>\n'
        f"  <asset>{hfield_asset(nrow, ncol, size, data)}{extra_asset}</asset>\n"
        "  <worldbody>\n"
        f'    <geom name="terrain" type="hfield" hfield="terrain" {hf_pose}/>\n'
        f'    <body name="obj" pos="{elevation(body_pos)}"><freejoint/>{geom_xml}</body>\n'
        "  </worldbody>\n</mujoco>\n"
    )


# (name, xml, qpos, note)
def build_scenes():
    scenes = []

    # 1) 交付三个 case：同配方（仓库 XML + 参考 qpos）独立重采
    cases = (
        ("deliv-hfield-box", "fixtures/mjwarp-complex-physics/hfield-box.xml",
         "fixtures/mjwarp-complex-physics/reference.json", False),
        ("deliv-hfield-sphere", "fixtures/g10-p5/models/hfield-sphere.xml",
         "fixtures/g10-p5/references/hfield-sphere.json", True),
        ("deliv-hfield-capsule", "fixtures/g10-p5/models/hfield-capsule.xml",
         "fixtures/g10-p5/references/hfield-capsule.json", True),
    )
    for name, xml_path, ref_path, standalone in cases:
        xml = (ROOT / xml_path).read_text(encoding="utf-8")
        case_name = name.removeprefix("deliv-")
        qpos = load_reference_qpos(ref_path, standalone, case_name)
        scenes.append({
            "name": name,
            "note": "delivered case recapture (same recipe: repo XML + reference qpos)",
            "xml": xml,
            "xml_path": xml_path,
            "qpos": qpos.tolist(),
            "delivered_fixture": f"fixtures/g10-p5/hfield-f32/{case_name}-cpu.json",
        })

    flat3 = flat(3, 3)
    size3 = [1.0, 1.0, 0.1, 0.1]

    def add(name, note, xml, qpos):
        scenes.append({"name": name, "note": note, "xml": xml, "qpos": qpos})

    # 2) 中心/边界 cell 命中 + filter 拒绝
    add("syn-flat-sphere-center", "3x3 flat, sphere centre over middle cell",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.05]),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    add("syn-flat-sphere-plus-x", "boundary cell: sphere near +x edge (cmax clamp)",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.92, 0.0, 0.05]),
        [0.92, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    add("syn-flat-sphere-minus-x", "boundary cell: sphere near -x edge (cmin clamp)",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [-0.92, 0.0, 0.05]),
        [-0.92, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    add("syn-outside-x", "filter reject: sphere beyond size[0]",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [1.5, 0.0, 0.0]),
        [1.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0])
    add("syn-above-top", "filter reject: sphere above size[2] top",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.5]),
        [0.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0])
    add("syn-below-base", "filter reject: sphere below -size[3] base",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, -0.5]),
        [0.0, 0.0, -0.5, 1.0, 0.0, 0.0, 0.0])
    # 切向（侧向）接触：对侧 geom 靠近 hfield 边缘、近似 45° 法向
    add("syn-side-sphere", "sphere near +x edge, ~45deg oblique contact",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.94, 0.02, 0.02]),
        [0.94, 0.02, 0.02, 1.0, 0.0, 0.0, 0.0])
    add("syn-side-box", "rotated box near +x edge, edge/corner-ish contact",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="g" type="box" size="0.1 0.1 0.1" euler="0 25 0" mass="1"/>',
                           [0.9, -0.02, 0.03]),
        [0.9, -0.02, 0.03, 1.0, 0.0, 0.0, 0.0])

    # 3) 子网格截断：5x5 平面 + 一个 spike 落在候选子网格之外
    spike = flat(5, 5)
    spike[2 * 5 + 3] = 1.0  # row 2, col 3
    add("syn-trunc-spike", "5x5 with one spike cell: height variation / candidate cell handling",
        world_hfield_scene(5, 5, size3, spike,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.05]),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    plateau = [0.5] * 9
    plateau[8] = 0.0  # corner 0 使归一化保持 0..0.5，其余为 0.5
    add("syn-plateau-sphere", "3x3 plateau data 0.5, sphere touching above",
        world_hfield_scene(3, 3, size3, plateau,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.05]),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])

    # 4) margin / gap 语义
    add("syn-margin", "sphere with geom margin 0.01 / gap 0.02",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" margin="0.01" gap="0.02" mass="1"/>',
                           [0.0, 0.0, 0.05]),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    add("syn-gap-skip", "sphere above with positive gap: margin+gap skip branch",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" gap="0.3" mass="1"/>',
                           [0.0, 0.0, 0.25]),
        [0.0, 0.0, 0.25, 1.0, 0.0, 0.0, 0.0])

    # 5) 旋转 hfield（hf_mat 非单位）
    add("syn-rot-hfield", "hfield geom rotated about z by 40deg, sphere above",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.05],
                           quat="0.9396926 0 0 0.3420201"),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    add("syn-rot-hfield-tilt", "hfield geom tilted about y by 20deg",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.05],
                           quat="0.9848078 0 0.1736482 0"),
        [0.0, 0.0, 0.05, 1.0, 0.0, 0.0, 0.0])
    # 对侧 geom 带非单位旋转：单独验证「world→hfield 帧」旋转约定（交付测试 helper 的转置问题）
    add("syn-rot-box", "box rotated (euler 20,0,30) penetrating flat hfield",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="g" type="box" size="0.12 0.08 0.1" euler="20 0 30" mass="1"/>',
                           [0.02, 0.0, 0.06]),
        [0.02, 0.0, 0.06, 1.0, 0.0, 0.0, 0.0])
    add("syn-rot-capsule", "capsule tilted (euler 35,0,25) penetrating flat hfield",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="g" type="capsule" size="0.05 0.12" euler="35 0 25" mass="1"/>',
                           [0.03, 0.0, 0.1]),
        [0.03, 0.0, 0.1, 1.0, 0.0, 0.0, 0.0])

    # 6) 深穿透（EPA 分支）
    add("syn-deep-sphere", "sphere deeply penetrating (EPA)",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, -0.05]),
        [0.0, 0.0, -0.05, 1.0, 0.0, 0.0, 0.0])

    # 7) MJ_MAXCONPAIR 溢出（自己构造超容量场景）
    add("syn-overflow-11", "11x11 flat + sphere r=2: >50 candidate contacts",
        world_hfield_scene(11, 11, size3, flat(11, 11),
                           '<geom name="s" type="sphere" size="2" mass="1"/>', [0.0, 0.0, 0.5]),
        [0.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0])
    add("syn-overflow-13", "13x13 flat + sphere r=2.5: overflow with larger candidate set",
        world_hfield_scene(13, 13, size3, flat(13, 13),
                           '<geom name="s" type="sphere" size="2.5" mass="1"/>', [0.1, 0.0, 0.6]),
        [0.1, 0.0, 0.6, 1.0, 0.0, 0.0, 0.0])

    # 8) 1e-3 去重边界：平滑起伏 hfield + 小半径几何
    bump = [0.0, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0]
    for radius, name in ((0.02, "syn-dedup-small"), (0.05, "syn-dedup-mid"), (0.11, "syn-dedup-wide")):
        add(name, f"smooth 3x3 bump + sphere r={radius}: 1e-3 dedup boundary probe",
            world_hfield_scene(3, 3, size3, bump,
                               f'<geom name="s" type="sphere" size="{radius}" mass="1"/>',
                               [0.05, 0.0, 0.11]),
            [0.05, 0.0, 0.11, 1.0, 0.0, 0.0, 0.0])
    add("syn-dedup-capsule", "smooth bump + small capsule: clustered contacts",
        world_hfield_scene(3, 3, size3, bump,
                           '<geom name="c" type="capsule" size="0.03 0.1" mass="1"/>',
                           [0.04, 0.0, 0.18]),
        [0.04, 0.0, 0.18, 1.0, 0.0, 0.0, 0.0])

    # 9) 六类对侧 geom 配对（CONVEX 路径）
    for name, geom in (
        ("syn-class-box", '<geom name="g" type="box" size="0.1 0.1 0.1" mass="1"/>'),
        ("syn-class-sphere", '<geom name="g" type="sphere" size="0.1" mass="1"/>'),
        ("syn-class-capsule", '<geom name="g" type="capsule" size="0.05 0.12" mass="1"/>'),
        ("syn-class-ellipsoid", '<geom name="g" type="ellipsoid" size="0.12 0.08 0.05" mass="1"/>'),
        ("syn-class-cylinder", '<geom name="g" type="cylinder" size="0.1 0.08" mass="1"/>'),
    ):
        add(name, f"pairing {name.split('-')[-1]} vs hfield through CONVEX ccd",
            world_hfield_scene(3, 3, size3, flat3, geom, [0.02, 0.0, 0.04]),
            [0.02, 0.0, 0.04, 1.0, 0.0, 0.0, 0.0])
    add("syn-class-mesh", "pairing mesh (tetra, 4 verts) vs hfield through CONVEX ccd",
        world_hfield_scene(3, 3, size3, flat3,
                           '<geom name="g" type="mesh" mesh="tet" mass="1"/>',
                           [0.03, 0.03, -0.02],
                           extra_asset='<mesh name="tet" vertex="0 0 0  0.2 0 0  0 0.2 0  0 0 0.2"/>'),
        [0.03, 0.03, -0.02, 1.0, 0.0, 0.0, 0.0])

    # 10) 两 world 分组（worldid 语义）
    add("syn-two-world", "2 worlds, sphere at different x: worldid grouping",
        world_hfield_scene(3, 3, size3, bump,
                           '<geom name="s" type="sphere" size="0.1" mass="1"/>', [0.0, 0.0, 0.12]),
        [0.0, 0.0, 0.12, 1.0, 0.0, 0.0, 0.0,
         0.2, 0.0, 0.1, 1.0, 0.0, 0.0, 0.0])
    return scenes


def main():
    SCENES.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.set_device("cpu")
    print("device:", wp.get_device())

    out = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-upstream-mjwarp-f32-cpu-recapture",
        "criterion": "同一配方上游 f32（warp CPU 后端）接触输出；ours vs 此参考为判据",
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
                "numpy": np.__version__,
            },
            "device": str(wp.get_device()),
            "script_sha256": sha256_bytes(Path(__file__).read_bytes()),
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
        "scenes": [],
    }
    for scene in build_scenes():
        name = scene["name"]
        xml = scene["xml"]
        path = SCENES / f"{name}.xml"
        path.write_text(xml, encoding="utf-8")
        host = mujoco.MjModel.from_xml_path(str(path))
        qpos_flat = np.asarray(scene["qpos"], dtype=np.float32)
        assert qpos_flat.size % host.nq == 0, (name, qpos_flat.size, host.nq)
        nworld = qpos_flat.size // host.nq
        qpos = qpos_flat.reshape(nworld, host.nq)
        model = mjw.put_model(host)
        first = capture(host, qpos, nworld, model)
        second = capture(host, qpos, nworld, model)
        arrays = {key: squeeze(getattr(model, key).numpy()).tolist() for key in MODEL_ARRAYS}
        mesh_arrays = {}
        for key in MESH_ARRAYS:
            if hasattr(model, key):
                mesh_arrays[key] = squeeze(getattr(model, key).numpy()).tolist()
        opt = {}
        for opt_name in ("ccd_tolerance", "ccd_iterations"):
            value = getattr(model.opt, opt_name)
            raw = value.numpy() if hasattr(value, "numpy") else value
            opt[opt_name] = float(np.asarray(raw).reshape(-1)[0])
        record = dict(scene)
        record.update({
            "xml_sha256": sha256_bytes(xml.encode("utf-8")),
            "nworld": nworld,
            "two_captures_bitwise_identical": first == second,
            "output": first,
            "model": {"arrays": arrays, "mesh_arrays": mesh_arrays, "opt": opt},
        })
        out["scenes"].append(record)
        print(f"{name:<26} ngeom={host.ngeom} nq={host.nq} worlds={nworld} count={first['count']} "
              f"overflow={first['overflow']} two_captures={record['two_captures_bitwise_identical']}")

    path = EVIDENCE / "qa-upstream-cases.json"
    path.write_text(json.dumps(out, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote", path)


if __name__ == "__main__":
    main()
