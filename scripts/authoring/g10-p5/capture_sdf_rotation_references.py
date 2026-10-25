"""G10@P5 G3 修正片 authoring：非单位相对旋转 SDF 场景的上游 f32 CPU 参考捕获。

背景：上游 `collision_sdf.py:724` 定义 `params.rel_mat = wp.transpose(rot1) * rot2`
（行优先 `aᵀb`，累加按 `warp/native/mat.h mul(mat,mat)` 的 fma 链）；产品旧
`sdf/descent.rs::relative_matrix` 构造的是 `bᵀa` 且用普通乘加。G3 `sdf-box` case 的
geom1 恒为单位旋转（`aᵀb = bᵀa`）未暴露该偏差；本脚本用同一模型 `fixtures/g10-p5/models/sdf-box.xml`
只改 geom 位姿/旋转，捕获同配方上游 **f32 CPU** 参考（`warp CPU` 后端，两次独立捕获位等）。

场景（2 world × 40 initpoints）：
- `sdf-rot-ident`：两侧单位旋转（对照）；
- `sdf-rot-sdf`：SDF 侧旋转、box 单位；
- `sdf-rot-box`：box 侧旋转、SDF 单位；
- `sdf-rot-both`：两侧都旋转；
- `sdf-rot-rz-rx`：SDF 侧 `Rz(0.7)` + box 侧 `Rx(0.4)`（`euler="22.918312 0 0"`）。

只读上游（pinned `mujoco_warp` 3.15 / `warp` 1.15）；不读产品源码、不跑产品测试；
产品测试不启动 Python。octree 数组复用 `fixtures/g10-p5/sdf-f32/` 并逐字节核对。

用法（仓库根目录）：
    target/mjwarp-parallel/venv/Scripts/python.exe -B \
        scripts/authoring/g10-p5/capture_sdf_rotation_references.py
输出：
    fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json（产品测试夹具）
    docs/development-evidence/g10-p5-sdf-relmat-fix/capture-report.json（证据）
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
EVIDENCE = ROOT / "docs/development-evidence/g10-p5-sdf-relmat-fix"
FIXTURE_DIR = ROOT / "fixtures/g10-p5/sdf-rotation"
SDF_DIR = ROOT / "fixtures/g10-p5/sdf-f32"
BASE_XML = (ROOT / "fixtures/g10-p5/models/sdf-box.xml").read_text(encoding="utf-8")

FLOAT_FIELDS = ("dist", "pos", "frame")
INT_FIELDS = ("geom", "worldid")
SMALL_ARRAYS = ("geom_type", "geom_size", "geom_dataid", "geom_aabb", "mesh_octadr")
BIN_ARRAYS = ("oct_aabb", "oct_child", "oct_coeff")


def sha256_path(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


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
        fields[name] = array[:count].reshape(count, -1).tolist() if count else []
    return {
        "count": count,
        "fields": fields,
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(nworld, host.ngeom * 9).tolist(),
    }


@wp.kernel
def _upstream_rel_mat(xmat: wp.array2d(dtype=wp.mat33), out: wp.array(dtype=wp.mat33)):
    """上游 `collision_sdf.py:724` 原式：params.rel_mat = transpose(rot1) * rot2。"""
    index = wp.tid()
    out[index] = wp.transpose(xmat[index, 0]) * xmat[index, 1]


def warp_relative_matrices(host, qpos, nworld, model):
    """warp 设备数组上的 `transpose(rot1)*rot2` 实际位型（供产品对照）。"""
    data = mjw.make_data(host, nworld=nworld, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((nworld, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    out = wp.empty(nworld, dtype=wp.mat33)
    wp.launch(_upstream_rel_mat, dim=nworld, inputs=[data.geom_xmat, out])
    wp.synchronize_device()
    return out.numpy().reshape(nworld, 9).tolist()


def rot_z(angle):
    (s, c) = (np.sin(angle), np.cos(angle))
    return [float(c), float(-s), 0.0, float(s), float(c), 0.0, 0.0, 0.0, 1.0]


def quat_z(angle):
    return [float(np.cos(angle / 2.0)), 0.0, 0.0, float(np.sin(angle / 2.0))]


def quat_axis(axis, angle):
    axis = np.asarray(axis, dtype=np.float64)
    axis = axis / np.linalg.norm(axis)
    half = angle / 2.0
    return [float(np.cos(half))] + [float(component * np.sin(half)) for component in axis]


# (name, note, box 属性串, world0 (pos, quat), world1 (pos, quat))
SCENES = (
    (
        "sdf-rot-ident",
        "两侧单位旋转（对照）",
        "",
        ([0.0, 0.0, 0.30], [1.0, 0.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.26], [1.0, 0.0, 0.0, 0.0]),
    ),
    (
        "sdf-rot-sdf",
        "SDF 侧旋转、box 单位",
        "",
        ([0.0, 0.0, 0.30], quat_z(0.7)),
        ([0.0, 0.0, 0.26], quat_axis([0.3, -0.5, 0.8], 0.9)),
    ),
    (
        "sdf-rot-box",
        "box 侧旋转、SDF 单位",
        ' euler="25 10 30"',
        ([0.0, 0.0, 0.30], [1.0, 0.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.26], [1.0, 0.0, 0.0, 0.0]),
    ),
    (
        "sdf-rot-both",
        "两侧都旋转",
        ' euler="25 10 30"',
        ([0.0, 0.0, 0.30], quat_z(0.7)),
        ([0.0, 0.0, 0.26], quat_axis([0.3, -0.5, 0.8], 0.9)),
    ),
    (
        "sdf-rot-rz-rx",
        "SDF 侧 Rz(0.7) + box 侧 Rx(0.4)",
        ' euler="22.918312 0 0"',
        ([0.0, 0.0, 0.30], quat_z(0.7)),
        ([0.0, 0.0, 0.30], quat_z(0.7)),
    ),
)


def build_xml(box_attrs):
    xml = BASE_XML
    box_line = '<geom name="box" type="box" size="0.1 0.08 0.06"/>'
    if box_line not in xml:
        raise SystemExit("base xml box line 不在预期位置")
    return xml.replace(
        box_line, f'<geom name="box" type="box" size="0.1 0.08 0.06"{box_attrs}/>'
    )


def main():
    work = ROOT / "target/g10-p5-sdf-relmat"
    work.mkdir(parents=True, exist_ok=True)
    scenes_dir = work / "scenes"
    scenes_dir.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(work / "warp-cache")
    wp.set_device("cpu")
    print("device:", wp.get_device())

    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-sdf-relmat-fix",
        "kind": "g10-p5-sdf-rotation-capture-report",
        "device": str(wp.get_device()),
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "oct_files": {},
        "scenes": {},
    }

    # 1) octree 数组：从同一 base XML 新鲜采集，与既有 fixture 二进制逐字节核对。
    base_xml_path = scenes_dir / "base.xml"
    base_xml_path.write_text(BASE_XML, encoding="utf-8")
    base_host = mujoco.MjModel.from_xml_path(str(base_xml_path))
    base_model = mjw.put_model(base_host)
    oct_files = {}
    for name in BIN_ARRAYS:
        array = getattr(base_model, name).numpy()
        dtype = "i32" if array.dtype == np.int32 else "f32"
        fresh_path = work / f"fresh-{name}.{dtype}"
        fresh_path.write_bytes(array.astype("<i4" if dtype == "i32" else "<f4").tobytes())
        fixture_path = SDF_DIR / f"{name}.{dtype}"
        fresh_sha = sha256_path(fresh_path)
        fixture_sha = sha256_path(fixture_path)
        oct_files[name] = {
            "path": f"fixtures/g10-p5/sdf-f32/{fixture_path.name}",
            "count": int(array.size),
            "sha256": fixture_sha,
            "fresh_sha256": fresh_sha,
            "bitwise_equal_to_fixture": fresh_sha == fixture_sha,
        }
        if fresh_sha != fixture_sha:
            raise SystemExit(f"oct {name} 新鲜采集与 fixture 不一致")
        print(f"oct {name}: fresh==fixture ({fresh_sha[:16]})")
    report["oct_files"] = oct_files

    # 2) 逐场景捕获两次（位等）并记录上游接触、位姿、warp rel_mat、模型数组。
    scenes = []
    for name, note, box_attrs, world0, world1 in SCENES:
        xml = build_xml(box_attrs)
        xml_path = scenes_dir / f"{name}.xml"
        xml_path.write_text(xml, encoding="utf-8")
        host = mujoco.MjModel.from_xml_path(str(xml_path))
        qpos_flat = np.asarray(
            [*world0[0], *world0[1], *world1[0], *world1[1]], dtype=np.float32
        )
        nworld = 2
        if qpos_flat.size != nworld * host.nq:
            raise SystemExit(f"{name}: qpos 长度 {qpos_flat.size} != 2*nq {2 * host.nq}")
        qpos = qpos_flat.reshape(nworld, host.nq)
        model = mjw.put_model(host)
        first = capture(host, qpos, nworld, model)
        second = capture(host, qpos, nworld, model)
        arrays = {key: squeeze(getattr(model, key).numpy()).tolist() for key in SMALL_ARRAYS}
        scene = {
            "name": name,
            "note": note,
            "xml": xml,
            "xml_sha256": sha256_path(xml_path),
            "qpos": qpos_flat.tolist(),
            "worlds": nworld,
            "sdf_iterations": int(host.opt.sdf_iterations),
            "sdf_initpoints": int(host.opt.sdf_initpoints),
            "two_captures_bitwise_identical": first == second,
            "rel_mat_warp": warp_relative_matrices(host, qpos, nworld, model),
            "output": first,
            "model": {"arrays": arrays},
        }
        if not scene["two_captures_bitwise_identical"]:
            raise SystemExit(f"{name}: 两次捕获不位等")
        scenes.append(scene)
        per_world = {}
        for world in range(nworld):
            per_world[f"world{world}"] = sum(
                1
                for row in first["fields"]["worldid"]
                if int(row[0]) == world
            )
        report["scenes"][name] = {
            "note": note,
            "two_captures_bitwise_identical": True,
            "count": first["count"],
            "contacts_per_world": per_world,
            "xml_sha256": scene["xml_sha256"],
        }
        print(f"{name:<16} count={first['count']} two_captures=True per_world={per_world}")

    # 3) 写产品测试夹具。
    script_path = Path(__file__)
    fixture = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-sdf-relmat-fix",
        "kind": "g10-p5-sdf-rotation-cpu-reference",
        "purpose": (
            "非单位相对旋转 SDF 场景的上游 f32 CPU 参考：验证产品相对旋转约定 = "
            "transpose(rot1)*rot2（行优先）+ fma 累加"
        ),
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "model": {
            "base_xml": "fixtures/g10-p5/models/sdf-box.xml",
            "oct_files": oct_files,
        },
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
                "numpy": np.__version__,
            },
            "device": str(wp.get_device()),
            "authoring_only": True,
            "reads_product_tree": False,
            "capture_script": "scripts/authoring/g10-p5/capture_sdf_rotation_references.py",
            "capture_script_sha256": sha256_path(script_path),
            "two_captures_bitwise_identical_all_scenes": True,
            "oct_files_bitwise_equal_to_fixture": True,
        },
        "acceptance_scenes": [
            "sdf-rot-sdf",
            "sdf-rot-box",
            "sdf-rot-both",
            "sdf-rot-rz-rx",
        ],
        "scenes": scenes,
    }
    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    fixture_path = FIXTURE_DIR / "sdf-rotation-cpu.json"
    fixture_path.write_text(
        json.dumps(fixture, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    print("wrote", fixture_path, fixture_path.stat().st_size)

    # 4) 与 QA 独立捕获交叉核对（只读另一任务证据；同配方应逐位相等）。
    qa_path = (
        ROOT
        / "docs/development-evidence/g10-p5-hfield-host-independent-check"
        / "qa-sdf-rotation-cases.json"
    )
    if qa_path.is_file():
        qa = json.loads(qa_path.read_text(encoding="utf-8"))
        qa_map = {scene["name"]: scene for scene in qa["scenes"]}
        cross = {}
        for scene in scenes:
            name = scene["name"]
            reference = qa_map.get(name)
            same = reference is not None and (
                reference["output"] == scene["output"]
                and reference["rel_mat_warp"] == scene["rel_mat_warp"]
                and reference["qpos"] == scene["qpos"]
            )
            cross[name] = same
            print(f"cross-check QA {name}: {same}")
        report["qa_reference_cross_check"] = {
            "path": "docs/development-evidence/g10-p5-hfield-host-independent-check/qa-sdf-rotation-cases.json",
            "sha256": sha256_path(qa_path),
            "all_output_relmat_qpos_bitwise_equal": all(cross.values()),
            "per_scene": cross,
        }
        if not all(cross.values()):
            raise SystemExit("本片独立捕获与 QA 参考不一致（同配方应逐位相等）")

    report["fixture"] = {
        "path": "fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json",
        "sha256": sha256_path(fixture_path),
        "bytes": fixture_path.stat().st_size,
        "acceptance_scenes": fixture["acceptance_scenes"],
    }
    report["provenance"] = fixture["provenance"]
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    report_path = EVIDENCE / "capture-report.json"
    report_path.write_text(
        json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    print("wrote", report_path)


if __name__ == "__main__":
    main()
