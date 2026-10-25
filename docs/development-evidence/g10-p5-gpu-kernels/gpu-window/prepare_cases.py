"""G5 GPU 窗口 · 设备内核输入准备 + 参考口径绑定（authoring-only）。

性质：authoring-only；使用 pinned mujoco_warp 3.15 / warp 1.15；不进产品路径，
产品测试不启动 Python。`--device cpu` 不占 GPU（只做输入口径诊断）；
`--device cuda` 在 GPU 窗口内运行（输入位姿与上游 GPU 同后端，并绑定参考）。

做的事：
1. G5 的 10 个 case：从交付的 CUDA 参考 `fixtures/g10-p5/references/<case>.json`
   （`capture_g0b_references.py` 以 `wp.set_device("cuda:0")` 捕获，provenance 含 GPU 名）
   取 qpos/qvel f32 位型，在当前 warp 后端重跑 kinematics + collision；
2. 对比：与交付参考按 worldid 分组 + `(dist,pos,frame,geom,type,dim,geomcollisionid)`
   稳定排序后逐字段位级比较；`--device cuda` 时这是「本窗口绑定参考」的证据，
   `--device cpu` 时只作输入口径诊断（CPU/CUDA 后端在个别 case 上本就有 ULP 级差异）；
3. 输出每个 case 一个目录 `cases/<case>/`：`meta.txt`（探针用 key value）+ 小端二进制
   数组（f32/i32），供 nvcc 独立探针读取；同时写 `prepare-cases-report.json`。

用法：
    python .../gpu-window/prepare_cases.py --device cpu     # 窗口前预演（不占 GPU）
    python .../gpu-window/prepare_cases.py --device cuda    # 窗口内绑定
"""

import hashlib
import json
import struct
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window"
OUT = EVIDENCE / "cases"
REF = ROOT / "fixtures/g10-p5/references"

# G0b 交付参考（CUDA 捕获）覆盖的 10 个 case。
CASES = (
    "sphere-box-shallow",
    "sphere-box-deep",
    "capsule-box",
    "ellipsoid-box",
    "hfield-sphere",
    "hfield-capsule",
    "sdf-box",
    "cylinder-box",
    "box-box-margin",
    "mesh-mesh",
)
# 接触数组全等（非 multiccd 组合 + hfield + SDF）。
CONTACT_CASES = set(CASES[:7])
# 仅 ccd() witness 对照（上游会走 multicontact，设备未实装）。
WITNESS_CASES = set(CASES[7:])
# 窗口内做新鲜双捕获的 spot-check case（覆盖 atomic 顺序敏感 / hfield / multiccd）。
SPOT_CASES = ("sdf-box", "hfield-sphere", "cylinder-box")

FLOAT_FIELDS = (
    "dist",
    "pos",
    "frame",
    "includemargin",
    "friction",
    "solref",
    "solreffriction",
    "solimp",
    "adhesion",
)
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")

GEOM_BOX = 6
GEOM_SDF = 8
GEOM_HFIELD = 1

DEVICE = "cpu"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits(values):
    return [struct.unpack("<I", struct.pack("<f", float(value)))[0] for value in np.ravel(values)]


def ref_fields(doc):
    """交付参考 `output.fields[name].values` → 扁平值列表。"""
    count = int(doc["output"]["count"])
    out = {}
    for name, record in doc["output"]["fields"].items():
        out[name] = record["values"]
    return count, out


def rows(count, fields):
    """扁平字段 → 逐接触行；宽度 = len(values)//count（0 宽度得到空列表）。"""
    out = []
    for index in range(count):
        row = {}
        for name, values in fields.items():
            width = len(values) // count if count else 0
            row[name] = values[index * width:(index + 1) * width] if width else []
        out.append(row)
    return out


def row_key(row):
    return (
        int(row["worldid"][0]),
        bits(row["dist"]),
        bits(row["pos"]),
        bits(row["frame"]),
        tuple(int(v) for v in row["geom"]),
        int(row["type"][0]),
        int(row["dim"][0]),
        int(row["geomcollisionid"][0]),
    )


def sorted_bitwise_equal(count_a, fields_a, count_b, fields_b):
    if count_a != count_b:
        return False
    left = sorted(rows(count_a, fields_a), key=row_key)
    right = sorted(rows(count_b, fields_b), key=row_key)
    for row_a, row_b in zip(left, right):
        for name in FLOAT_FIELDS:
            if bits(row_a[name]) != bits(row_b[name]):
                return False
        for name in INT_FIELDS:
            if [int(v) for v in row_a[name]] != [int(v) for v in row_b[name]]:
                return False
    return True


def capture_run(host, qpos, qvel, worlds):
    """当前 warp 后端：kinematics + collision（--device cuda 时占 GPU 窗口）。"""
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = getattr(data.contact, name).numpy()[:count]
        fields[name] = array.reshape(-1).tolist() if array.size else []
    return model, data, count, fields


def squeeze(array):
    array = np.asarray(array)
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def write_bin(directory, name, array, dtype):
    values = np.ascontiguousarray(squeeze(np.asarray(array)), dtype=dtype)
    path = directory / f"{name}.bin"
    path.write_bytes(values.tobytes())
    return {
        "file": path.name,
        "dtype": "f32" if dtype == np.float32 else "i32",
        "count": int(values.size),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def as_f32(array):
    # 唯一 f64→f32 舍入点：上游模型数组上传按 f32（G1 口径）。
    return np.ascontiguousarray(np.asarray(array), dtype=np.float32)


def as_i32(array):
    return np.ascontiguousarray(np.asarray(array), dtype=np.int32)


def prepare_case(name, spot):
    reference_path = REF / f"{name}.json"
    doc = json.loads(reference_path.read_text(encoding="utf-8"))
    worlds = int(doc["input"]["worlds"])
    qpos = np.asarray(doc["input"]["qpos"]["values"], dtype=np.float32).reshape(worlds, -1)
    qvel = np.asarray(doc["input"]["qvel"]["values"], dtype=np.float32).reshape(
        worlds, doc["input"]["qvel"]["shape"][1]
    )
    xml = ROOT / doc["input"]["path"]
    host = mujoco.MjModel.from_xml_path(str(xml))

    model, data, count, fields = capture_run(host, qpos, qvel, worlds)
    ref_count, ref = ref_fields(doc)
    matches = sorted_bitwise_equal(count, fields, ref_count, ref)

    # spot-check：第二次捕获（原始位等 + 排序位等）。
    second = None
    if spot:
        _, _, count2, fields2 = capture_run(host, qpos, qvel, worlds)
        second = {
            "count": count2,
            "two_captures_raw_bitwise_identical": count == count2 and fields == fields2,
            "second_matches_reference_sorted_bitwise": sorted_bitwise_equal(
                count2, fields2, ref_count, ref
            ),
        }

    directory = OUT / name
    directory.mkdir(parents=True, exist_ok=True)
    arrays = {}

    def record(name_key, array, dtype):
        arrays[name_key] = write_bin(directory, name_key, array, dtype)

    record("geom_xpos", as_f32(data.geom_xpos.numpy()), np.float32)
    record("geom_xmat", as_f32(data.geom_xmat.numpy().reshape(worlds, host.ngeom * 9)), np.float32)
    record("geom_size", as_f32(squeeze(model.geom_size.numpy())), np.float32)
    record("geom_margin", as_f32(squeeze(model.geom_margin.numpy())), np.float32)
    record("geom_gap", as_f32(squeeze(model.geom_gap.numpy())), np.float32)
    record("geom_rbound", as_f32(squeeze(model.geom_rbound.numpy())), np.float32)
    record("geom_aabb", as_f32(squeeze(model.geom_aabb.numpy())), np.float32)
    record("geom_type", as_i32(squeeze(model.geom_type.numpy())), np.int32)
    record("geom_dataid", as_i32(squeeze(model.geom_dataid.numpy())), np.int32)
    record("mesh_vertadr", as_i32(squeeze(model.mesh_vertadr.numpy())), np.int32)
    record("mesh_vertnum", as_i32(squeeze(model.mesh_vertnum.numpy())), np.int32)
    record("mesh_graphadr", as_i32(squeeze(model.mesh_graphadr.numpy())), np.int32)
    record("mesh_octadr", as_i32(squeeze(model.mesh_octadr.numpy())), np.int32)
    record("mesh_vert", as_f32(model.mesh_vert.numpy()), np.float32)
    record("mesh_graph", as_i32(model.mesh_graph.numpy()), np.int32)
    record("hfield_size", as_f32(squeeze(model.hfield_size.numpy())), np.float32)
    record("hfield_nrow", as_i32(squeeze(model.hfield_nrow.numpy())), np.int32)
    record("hfield_ncol", as_i32(squeeze(model.hfield_ncol.numpy())), np.int32)
    record("hfield_adr", as_i32(squeeze(model.hfield_adr.numpy())), np.int32)
    record("hfield_data", as_f32(model.hfield_data.numpy()), np.float32)
    record("oct_aabb", as_f32(model.oct_aabb.numpy()), np.float32)
    record("oct_child", as_i32(model.oct_child.numpy()), np.int32)
    record("oct_coeff", as_f32(model.oct_coeff.numpy()), np.float32)

    # 解析接触参数：取交付参考第一行的等价量（同一 pair 全行相同）；
    # pairid 取默认 (-1,-1)（本批模型无 <pair>）；gap = 两侧 geom_gap 之和。
    row = rows(ref_count, ref)[0]
    g1, g2 = int(row["geom"][0]), int(row["geom"][1])
    geom_gap = as_f32(squeeze(model.geom_gap.numpy()))
    gap = float(geom_gap[g1] + geom_gap[g2])
    geom_type = as_i32(squeeze(model.geom_type.numpy()))
    type1, type2 = int(geom_type[g1]), int(geom_type[g2])
    geom_condim = as_i32(squeeze(model.geom_condim.numpy()))
    nmaxcondim = max(int(v) for v in geom_condim)
    efc_width = max(1, 2 * (nmaxcondim - 1))
    dataid = as_i32(squeeze(model.geom_dataid.numpy()))

    if type2 == GEOM_SDF:
        kind = "sdf"
    elif type1 == GEOM_HFIELD:
        kind = "hfield"
    else:
        kind = "contact"

    meta = [
        ("case", name),
        ("category", "contact" if name in CONTACT_CASES else "witness"),
        ("kind", kind),
        ("nworld", worlds),
        ("ngeom", int(host.ngeom)),
        ("naconmax", 4096 * worlds),
        ("naccdmax", 4096 * worlds),
        ("efc_width", efc_width),
        ("g1", g1),
        ("g2", g2),
        ("type1", type1),
        ("type2", type2),
        ("tolerance", float(host.opt.ccd_tolerance)),
        ("gjk_iterations", int(host.opt.ccd_iterations)),
        (
            "epa_iterations",
            16 if (type1 == GEOM_BOX and type2 == GEOM_BOX) else int(host.opt.ccd_iterations),
        ),
        ("margin", float(row["includemargin"][0])),
        ("gap", gap),
        ("condim", int(row["dim"][0])),
        ("friction", [float(v) for v in row["friction"]]),
        ("solref", [float(v) for v in row["solref"]]),
        ("solreffriction", [float(v) for v in row["solreffriction"]]),
        ("solimp", [float(v) for v in row["solimp"]]),
        ("adhesion", float(row["adhesion"][0])),
        ("pairid0", -1),
        ("pairid1", -1),
        ("sdf_initpoints", int(host.opt.sdf_initpoints)),
        ("sdf_iterations", int(host.opt.sdf_iterations)),
        (
            "oct_root",
            int(as_i32(squeeze(model.mesh_octadr.numpy()))[dataid[g2]]) if type2 == GEOM_SDF else -1,
        ),
        ("hfield_id", int(dataid[g1]) if kind == "hfield" else -1),
        ("hfield_data_count", arrays["hfield_data"]["count"]),
        ("noct", int(arrays["oct_coeff"]["count"]) // 8),
        ("nmeshvert", arrays["mesh_vert"]["count"] // 3),
        ("nmeshgraph", arrays["mesh_graph"]["count"]),
    ]
    lines = []
    for key, value in meta:
        if isinstance(value, list):
            lines.append(f"{key} " + " ".join(repr(v) for v in value))
        else:
            lines.append(f"{key} {value}")
    (directory / "meta.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")

    return {
        "kind": "g5-device-case-inputs",
        "category": "contact" if name in CONTACT_CASES else "witness",
        "backend": DEVICE,
        "arrays": arrays,
        "meta": "meta.txt",
        "contact_run": {
            "count": count,
            "backend": DEVICE,
            "matches_reference_sorted_bitwise": matches,
        },
        "spot": second,
        "reference": {
            "path": f"fixtures/g10-p5/references/{name}.json",
            "sha256": sha256(reference_path),
            "count": ref_count,
            "backend": "cuda",
        },
    }


def main():
    global DEVICE
    arguments = sys.argv[1:]
    if "--device" in arguments:
        DEVICE = arguments[arguments.index("--device") + 1]
    if DEVICE not in ("cpu", "cuda"):
        raise SystemExit("--device 只接受 cpu|cuda")
    OUT.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(ROOT / f"target/g10-p5-g5/warp-cache-{DEVICE}")
    wp.set_device(DEVICE)
    report = {
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-gpu-kernels",
        "kind": "g5-device-case-inputs-prep",
        "purpose": (
            "准备设备内核输入；在当前后端重跑并与交付 CUDA 参考排序位级对比"
            "（cuda=窗口绑定，cpu=输入口径诊断）"
        ),
        "device": DEVICE,
        "cases": {},
        "provenance": {
            "versions": {
                "python": ".".join(map(str, sys.version_info[:3])),
                "mujoco": mujoco.__version__,
                "warp": wp.__version__,
                "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
                "numpy": np.__version__,
            },
            "device": DEVICE,
            "authoring_script": (
                "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/prepare_cases.py"
            ),
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
            "performance_measured": False,
        },
    }
    all_match = True
    for name in CASES:
        record = prepare_case(name, spot=DEVICE == "cuda" and name in SPOT_CASES)
        report["cases"][name] = record
        match = record["contact_run"]["matches_reference_sorted_bitwise"]
        all_match = all_match and match
        spot = record["spot"]
        extra = ""
        if spot is not None:
            extra = (
                f" two_captures_raw={spot['two_captures_raw_bitwise_identical']}"
                f" second_sorted={spot['second_matches_reference_sorted_bitwise']}"
            )
        print(
            f"{name}[{DEVICE}]: contacts={record['contact_run']['count']} "
            f"ref={record['reference']['count']} sorted_bitwise={match}{extra}"
        )
    report["all_matches_reference_sorted_bitwise"] = all_match
    (EVIDENCE / "prepare-cases-report.json").write_text(
        json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    print(f"all_matches_reference_sorted_bitwise({DEVICE}):", all_match)


if __name__ == "__main__":
    main()
