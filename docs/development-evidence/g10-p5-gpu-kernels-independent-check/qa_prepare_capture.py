"""QA 独立复核 · G5：自写输入准备 + 上游 f32 GPU 参考捕获 + native f64 读数。

性质：authoring-only（不进产品路径）。GPU 窗口内以 `wp.set_device("cuda:0")` 运行：
1. 10 个 G0b case：按 G0b 参考的 qpos/qvel 重跑 kinematics+collision，**自己捕获**上游 f32 GPU
   接触字段（不读交付 fixture 的接触值），并把同一次运行得到的 `geom_xpos/geom_xmat`（GPU 位型）
   与模型数组写成我自己的设备探针输入 bins；
2. 3 个 multicontact case + 姿态扫描：直接调上游 `collision_gjk.ccd`（逐 world launch、
   每 world 独立 EPA 缓冲、显式世界守卫）捕获 `ccd()` witness；
3. 容量/溢出场景：`naconmax` 截断、`naccdmax` 截断、hfield `MJ_MAXCONPAIR` 溢出的上游行为；
4. native f64（MuJoCo C，CPU，无需窗口）：同一 qpos 的接触字段（仅作 G6 容差候选材料，非判据）。

输出：
- `qa-gpu/cases/<case>/`：设备探针输入（meta.txt + 小端 bins）
- `qa-upstream-gpu.json`：上游 f32 GPU 接触参考 + 容量/溢出场景
- `qa-witness-gpu.json`：上游 `ccd()` witness 参考
- `qa-native-f64.json`：native C f64 接触字段
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[2]
WORK = EVIDENCE / "qa-gpu"
CASES = WORK / "cases"
REF = ROOT / "fixtures/g10-p5/references"

FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "worldid", "type", "geomcollisionid")

CONTACT_CASES = ("ellipsoid-box", "hfield-sphere", "hfield-capsule", "sdf-box")
WITNESS_CASES = ("cylinder-box", "box-box-margin", "mesh-mesh")
ALL10 = ("sphere-box-shallow", "sphere-box-deep", "capsule-box", "ellipsoid-box",
         "hfield-sphere", "hfield-capsule", "sdf-box", "cylinder-box",
         "box-box-margin", "mesh-mesh")

GEOM_BOX = 6
GEOM_SDF = 8
GEOM_HFIELD = 1


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def squeeze(array):
    import numpy as np
    array = np.asarray(array)
    while array.ndim >= 2 and array.shape[0] == 1:
        array = array[0]
    return array


def write_bin(directory, name, array, dtype):
    import numpy as np
    values = np.ascontiguousarray(squeeze(np.asarray(array)), dtype=dtype)
    path = directory / f"{name}.bin"
    path.write_bytes(values.tobytes())
    return {"file": path.name, "dtype": "f32" if dtype == np.float32 else "i32",
            "count": int(values.size), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def write_meta(directory, meta):
    lines = []
    for key, value in meta:
        if isinstance(value, list):
            lines.append(f"{key} " + " ".join(repr(v) for v in value))
        else:
            lines.append(f"{key} {value}")
    (directory / "meta.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def load_reference(name):
    import numpy as np
    doc = json.loads((REF / f"{name}.json").read_text(encoding="utf-8"))
    worlds = int(doc["input"]["worlds"])
    qpos = np.asarray(doc["input"]["qpos"]["values"], dtype=np.float32).reshape(worlds, -1)
    qvel = np.asarray(doc["input"]["qvel"]["values"], dtype=np.float32).reshape(
        worlds, doc["input"]["qvel"]["shape"][1])
    xml = doc["input"]["path"]
    return doc, worlds, qpos, qvel, xml


def capture_contacts(host, qpos, qvel, worlds, naconmax=None, naccdmax=None):
    import mujoco_warp as mjw
    import numpy as np
    import warp as wp
    kwargs = dict(nworld=worlds, nconmax=4096, njmax=20000)
    if naconmax is not None:
        kwargs["naconmax"] = naconmax
    if naccdmax is not None:
        kwargs["naccdmax"] = naccdmax
    model = mjw.put_model(host)
    data = mjw.make_data(host, **kwargs)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    capacity = int(data.contact.dist.shape[0])
    written = min(count, capacity)
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = getattr(data.contact, name).numpy()[:written]
        fields[name] = array.reshape(-1).tolist() if written else []
    return {"count": count, "capacity": capacity, "written": written,
            "fields": fields,
            "geom_xpos": data.geom_xpos.numpy().tolist(),
            "geom_xmat": data.geom_xmat.numpy().reshape(worlds, host.ngeom * 9).tolist(),
            "nccd": data.nccd.numpy().tolist() if hasattr(data, "nccd") else None,
            "overflow": data.overflow.numpy().tolist()}


def write_case_inputs(dirname, param_name, host, model, data, worlds, kind="auto",
                      case_label=None):
    """把 GPU 位型的位姿 + 模型数组写成设备探针输入（QA 自己的格式）。"""
    import numpy as np
    directory = CASES / dirname
    directory.mkdir(parents=True, exist_ok=True)
    arrays = {}

    def record(key, array, dtype):
        arrays[key] = write_bin(directory, key, array, dtype)

    record("geom_xpos", data.geom_xpos.numpy(), np.float32)
    record("geom_xmat", data.geom_xmat.numpy().reshape(worlds, host.ngeom * 9), np.float32)
    record("geom_size", model.geom_size.numpy(), np.float32)
    record("geom_margin", model.geom_margin.numpy(), np.float32)
    record("geom_gap", model.geom_gap.numpy(), np.float32)
    record("geom_rbound", model.geom_rbound.numpy(), np.float32)
    record("geom_aabb", model.geom_aabb.numpy(), np.float32)
    record("geom_type", model.geom_type.numpy(), np.int32)
    record("geom_dataid", model.geom_dataid.numpy(), np.int32)
    record("mesh_vertadr", model.mesh_vertadr.numpy(), np.int32)
    record("mesh_vertnum", model.mesh_vertnum.numpy(), np.int32)
    record("mesh_graphadr", model.mesh_graphadr.numpy(), np.int32)
    record("mesh_octadr", model.mesh_octadr.numpy(), np.int32)
    record("mesh_vert", model.mesh_vert.numpy(), np.float32)
    record("mesh_graph", model.mesh_graph.numpy(), np.int32)
    record("hfield_size", model.hfield_size.numpy(), np.float32)
    record("hfield_nrow", model.hfield_nrow.numpy(), np.int32)
    record("hfield_ncol", model.hfield_ncol.numpy(), np.int32)
    record("hfield_adr", model.hfield_adr.numpy(), np.int32)
    record("hfield_data", model.hfield_data.numpy(), np.float32)
    record("oct_aabb", model.oct_aabb.numpy(), np.float32)
    record("oct_child", model.oct_child.numpy(), np.int32)
    record("oct_coeff", model.oct_coeff.numpy(), np.float32)

    geom_type = squeeze(model.geom_type.numpy()).tolist()
    dataid = squeeze(model.geom_dataid.numpy()).tolist()
    geom_margin = squeeze(model.geom_margin.numpy()).tolist()
    geom_gap = squeeze(model.geom_gap.numpy()).tolist()
    geom_condim = squeeze(model.geom_condim.numpy()).tolist()
    nmaxcondim = max(int(v) for v in geom_condim)
    efc_width = max(1, 2 * (nmaxcondim - 1))
    # 参考第一行给出 pair 方向与接触参数（同 pair 全行相同；仅作输入口径）
    ref = json.loads((REF / f"{param_name}.json").read_text(encoding="utf-8"))
    fields = ref["output"]["fields"]
    count = int(ref["output"]["count"])
    first = {key: values["values"] for key, values in fields.items()}
    width = lambda key: len(first[key]) // count if count else 0
    def row(key):
        width_ = width(key)
        return first[key][:width_]
    g1, g2 = int(row("geom")[0]), int(row("geom")[1])
    type1, type2 = int(geom_type[g1]), int(geom_type[g2])
    if kind == "auto":
        if type2 == GEOM_SDF:
            kind = "sdf"
        elif type1 == GEOM_HFIELD:
            kind = "hfield"
        else:
            kind = "contact"
    meta = [
        ("case", case_label or dirname), ("kind", kind),
        ("nworld", worlds), ("ngeom", int(host.ngeom)),
        ("naconmax", 4096 * worlds), ("naccdmax", 4096 * worlds), ("efc_width", efc_width),
        ("g1", g1), ("g2", g2), ("type1", type1), ("type2", type2),
        ("tolerance", float(host.opt.ccd_tolerance)),
        ("gjk_iterations", int(host.opt.ccd_iterations)),
        ("epa_iterations", 16 if (type1 == GEOM_BOX and type2 == GEOM_BOX)
         else int(host.opt.ccd_iterations)),
        ("margin", float(geom_margin[g1] + geom_margin[g2])),
        ("gap", float(geom_gap[g1] + geom_gap[g2])),
        ("condim", int(row("dim")[0])),
        ("friction", [float(v) for v in row("friction")]),
        ("solref", [float(v) for v in row("solref")]),
        ("solreffriction", [float(v) for v in row("solreffriction")]),
        ("solimp", [float(v) for v in row("solimp")]),
        ("adhesion", float(row("adhesion")[0])),
        ("pairid0", -1), ("pairid1", -1),
        ("sdf_initpoints", int(host.opt.sdf_initpoints)),
        ("sdf_iterations", int(host.opt.sdf_iterations)),
        ("oct_root", int(squeeze(model.mesh_octadr.numpy())[dataid[g2]]) if type2 == GEOM_SDF else -1),
        ("hfield_id", int(dataid[g1]) if kind == "hfield" else -1),
        ("hfield_data_count", arrays["hfield_data"]["count"]),
        ("noct", int(arrays["oct_coeff"]["count"]) // 8),
        ("nmeshvert", int(arrays["mesh_vert"]["count"]) // 3),
        ("nmeshgraph", int(arrays["mesh_graph"]["count"])),
    ]
    write_meta(directory, meta)
    return {"case": case_label or dirname, "kind": kind, "arrays": arrays}


def write_custom_inputs(dirname, host, model, data, worlds, g1, g2, type1, type2,
                        margin, gap, kind, max_horizon=24, case_label=None,
                        params_source=None):
    """自造场景（无 G0b 参考）的设备探针输入；接触参数取 params_source 首行或默认值。"""
    import numpy as np
    directory = CASES / dirname
    directory.mkdir(parents=True, exist_ok=True)
    arrays = {}

    def record(key, array, dtype):
        arrays[key] = write_bin(directory, key, array, dtype)

    record("geom_xpos", data.geom_xpos.numpy(), np.float32)
    record("geom_xmat", data.geom_xmat.numpy().reshape(worlds, host.ngeom * 9), np.float32)
    record("geom_size", model.geom_size.numpy(), np.float32)
    record("geom_margin", model.geom_margin.numpy(), np.float32)
    record("geom_gap", model.geom_gap.numpy(), np.float32)
    record("geom_rbound", model.geom_rbound.numpy(), np.float32)
    record("geom_aabb", model.geom_aabb.numpy(), np.float32)
    record("geom_type", model.geom_type.numpy(), np.int32)
    record("geom_dataid", model.geom_dataid.numpy(), np.int32)
    record("mesh_vertadr", model.mesh_vertadr.numpy(), np.int32)
    record("mesh_vertnum", model.mesh_vertnum.numpy(), np.int32)
    record("mesh_graphadr", model.mesh_graphadr.numpy(), np.int32)
    record("mesh_octadr", model.mesh_octadr.numpy(), np.int32)
    record("mesh_vert", model.mesh_vert.numpy(), np.float32)
    record("mesh_graph", model.mesh_graph.numpy(), np.int32)
    record("hfield_size", model.hfield_size.numpy(), np.float32)
    record("hfield_nrow", model.hfield_nrow.numpy(), np.int32)
    record("hfield_ncol", model.hfield_ncol.numpy(), np.int32)
    record("hfield_adr", model.hfield_adr.numpy(), np.int32)
    record("hfield_data", model.hfield_data.numpy(), np.float32)
    record("oct_aabb", model.oct_aabb.numpy(), np.float32)
    record("oct_child", model.oct_child.numpy(), np.int32)
    record("oct_coeff", model.oct_coeff.numpy(), np.float32)
    dataid = squeeze(model.geom_dataid.numpy()).tolist()
    if params_source is not None:
        fields = params_source["fields"]
        count = int(params_source["count"])
        width = lambda key: len(fields[key]) // count if count else 0
        def row(key):
            w = width(key)
            return fields[key][:w]
        condim = int(row("dim")[0])
        friction = [float(v) for v in row("friction")]
        solref = [float(v) for v in row("solref")]
        solreffriction = [float(v) for v in row("solreffriction")]
        solimp = [float(v) for v in row("solimp")]
        adhesion = float(row("adhesion")[0])
    else:
        condim = 3
        friction = [1.0, 1.0, 0.004999999888241291, 9.999999747378752e-05,
                    9.999999747378752e-05]
        solref = [0.019999999552965164, 1.0]
        solreffriction = [0.0, 0.0]
        solimp = [0.8999999761581421, 0.949999988079071, 0.0010000000474974513, 0.5, 2.0]
        adhesion = 0.0
    nmaxcondim = max(int(v) for v in squeeze(model.geom_condim.numpy()).tolist())
    efc_width = max(1, 2 * (nmaxcondim - 1))
    meta = [
        ("case", case_label or dirname), ("kind", kind),
        ("nworld", worlds), ("ngeom", int(host.ngeom)),
        ("naconmax", 4096 * worlds), ("naccdmax", 4096 * worlds), ("efc_width", efc_width),
        ("g1", g1), ("g2", g2), ("type1", type1), ("type2", type2),
        ("tolerance", float(host.opt.ccd_tolerance)),
        ("gjk_iterations", int(host.opt.ccd_iterations)),
        ("epa_iterations", 16 if (type1 == GEOM_BOX and type2 == GEOM_BOX)
         else int(host.opt.ccd_iterations)),
        ("margin", float(margin)), ("gap", float(gap)), ("condim", condim),
        ("friction", friction), ("solref", solref),
        ("solreffriction", solreffriction), ("solimp", solimp),
        ("adhesion", adhesion),
        ("pairid0", -1), ("pairid1", -1),
        ("sdf_initpoints", int(host.opt.sdf_initpoints)),
        ("sdf_iterations", int(host.opt.sdf_iterations)),
        ("oct_root", -1),
        ("hfield_id", int(dataid[g1]) if type1 == GEOM_HFIELD else -1),
        ("hfield_data_count", arrays["hfield_data"]["count"]),
        ("noct", int(arrays["oct_coeff"]["count"]) // 8),
        ("nmeshvert", int(arrays["mesh_vert"]["count"]) // 3),
        ("nmeshgraph", int(arrays["mesh_graph"]["count"])),
        ("max_horizon", max_horizon),
    ]
    write_meta(directory, meta)
    return {"case": case_label or dirname, "kind": kind, "arrays": arrays}


# ---------------- witness：上游直接调 ccd() ----------------

def witness_kernel():
    import mujoco_warp as mjw
    import mujoco_warp._src.collision_gjk as gjk
    import mujoco_warp._src.types as types
    import warp as wp
    from mujoco_warp._src.collision_core import Geom

    mj_max_epafaces = types.MJ_MAX_EPAFACES

    @wp.kernel(module="unique", enable_backward=False)
    def _witness(
        # 单 world 的位姿
        pos1_in: wp.array(dtype=wp.vec3),
        rot1_in: wp.array(dtype=wp.mat33),
        pos2_in: wp.array(dtype=wp.vec3),
        rot2_in: wp.array(dtype=wp.mat33),
        # 模型数组
        geom_size: wp.array2d(dtype=wp.vec3),
        geom_type: wp.array(dtype=int),
        geom_dataid: wp.array2d(dtype=int),
        mesh_vertadr: wp.array(dtype=int),
        mesh_vertnum: wp.array(dtype=int),
        mesh_graphadr: wp.array(dtype=int),
        mesh_vert: wp.array(dtype=wp.vec3),
        mesh_graph: wp.array(dtype=int),
        mesh_polynum: wp.array(dtype=int),
        mesh_polyadr: wp.array(dtype=int),
        mesh_polynormal: wp.array(dtype=wp.vec3),
        mesh_polyvertadr: wp.array(dtype=int),
        mesh_polyvertnum: wp.array(dtype=int),
        mesh_polyvert: wp.array(dtype=int),
        mesh_polymapadr: wp.array(dtype=int),
        mesh_polymapnum: wp.array(dtype=int),
        mesh_polymap: wp.array(dtype=int),
        # 参数
        g1: int,
        g2: int,
        tolerance: float,
        gjk_iterations: int,
        epa_iterations: int,
        margin: float,
        cutoff: float,
        horizon_buffer: wp.array(dtype=int),
        # 输出
        dist_out: wp.array(dtype=float),
        ncontact_out: wp.array(dtype=int),
        x1_out: wp.array(dtype=wp.vec3),
        x2_out: wp.array(dtype=wp.vec3),
        index_out: wp.array(dtype=int),
        overflow_out: wp.array(dtype=int),
    ):
        i = wp.tid()  # 恒为 0（每 world 一次 launch）
        geom1 = Geom()
        geom2 = Geom()
        geom1.pos = pos1_in[i]
        geom1.rot = rot1_in[i]
        geom1.normal = wp.vec3(geom1.rot[0, 2], geom1.rot[1, 2], geom1.rot[2, 2])
        geom2.pos = pos2_in[i]
        geom2.rot = rot2_in[i]
        geom2.normal = wp.vec3(geom2.rot[0, 2], geom2.rot[1, 2], geom2.rot[2, 2])
        dataid1 = geom_dataid[0, g1]
        dataid2 = geom_dataid[0, g2]
        geom1.size = geom_size[0, g1]
        geom2.size = geom_size[0, g2]
        geom1.margin = margin
        geom2.margin = margin
        geom1.index = -1
        geom2.index = -1
        geom1.polyvert = types.mat63()
        geom2.polyvert = types.mat63()
        geom1.vertadr = -1
        geom1.vertnum = -1
        geom1.graphadr = -1
        geom1.mesh_polynum = -1
        geom1.mesh_polyadr = -1
        geom2.vertadr = -1
        geom2.vertnum = -1
        geom2.graphadr = -1
        geom2.mesh_polynum = -1
        geom2.mesh_polyadr = -1
        if geom_type[g1] == types.GeomType.MESH:
            geom1.vertadr = mesh_vertadr[dataid1]
            geom1.vertnum = mesh_vertnum[dataid1]
            geom1.graphadr = mesh_graphadr[dataid1]
            geom1.mesh_polynum = mesh_polynum[dataid1]
            geom1.mesh_polyadr = mesh_polyadr[dataid1]
            geom1.vert = mesh_vert
            geom1.graph = mesh_graph
            geom1.mesh_polynormal = mesh_polynormal
            geom1.mesh_polyvertadr = mesh_polyvertadr
            geom1.mesh_polyvertnum = mesh_polyvertnum
            geom1.mesh_polyvert = mesh_polyvert
            geom1.mesh_polymapadr = mesh_polymapadr
            geom1.mesh_polymapnum = mesh_polymapnum
            geom1.mesh_polymap = mesh_polymap
        if geom_type[g2] == types.GeomType.MESH:
            geom2.vertadr = mesh_vertadr[dataid2]
            geom2.vertnum = mesh_vertnum[dataid2]
            geom2.graphadr = mesh_graphadr[dataid2]
            geom2.mesh_polynum = mesh_polynum[dataid2]
            geom2.mesh_polyadr = mesh_polyadr[dataid2]
            geom2.vert = mesh_vert
            geom2.graph = mesh_graph
            geom2.mesh_polynormal = mesh_polynormal
            geom2.mesh_polyvertadr = mesh_polyvertadr
            geom2.mesh_polyvertnum = mesh_polyvertnum
            geom2.mesh_polyvert = mesh_polyvert
            geom2.mesh_polymapadr = mesh_polymapadr
            geom2.mesh_polymapnum = mesh_polymapnum
            geom2.mesh_polymap = mesh_polymap
        # 上游 eval_ccd_write_contact：原点平移到 geom1
        origin = geom1.pos
        geom1.pos = wp.vec3(0.0)
        geom2.pos = geom2.pos - origin
        # 容量按 epa_iterations 上界取常量（算法只用 iterations 次迭代；缓冲更大不影响结果）
        vert = wp.zeros(138, dtype=wp.vec3)
        vert_index = wp.zeros(138, dtype=int)
        face = wp.zeros(326, dtype=int)
        face_pr = wp.zeros(326, dtype=wp.vec3)
        face_norm2 = wp.zeros(326, dtype=float)
        worldid = 0
        dist, ncontact, x1, x2, idx = gjk.ccd(
            tolerance, cutoff, gjk_iterations, epa_iterations, geom1, geom2,
            geom_type[g1], geom_type[g2], geom1.pos, geom2.pos,
            vert, vert_index, face, face_pr, face_norm2, horizon_buffer,
            True, worldid, overflow_out)
        dist_out[i] = dist
        ncontact_out[i] = ncontact
        x1_out[i] = x1
        x2_out[i] = x2
        index_out[i] = idx

    return _witness


def capture_witness(case, host, qpos, worlds, override=None):
    import mujoco_warp as mjw
    import numpy as np
    import warp as wp
    kernel = witness_kernel()
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((worlds, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    geom_type = squeeze(model.geom_type.numpy()).tolist()
    geom_margin = squeeze(model.geom_margin.numpy()).tolist()
    geom_gap = squeeze(model.geom_gap.numpy()).tolist()
    if override is not None:
        g1, g2 = int(override["g1"]), int(override["g2"])
        margin = float(override.get("margin", geom_margin[g1] + geom_margin[g2]))
        gap = float(override.get("gap", geom_gap[g1] + geom_gap[g2]))
    else:
        ref = json.loads((REF / f"{case}.json").read_text(encoding="utf-8"))
        fields = ref["output"]["fields"]
        count = int(ref["output"]["count"])
        first = {key: values["values"] for key, values in fields.items()}
        width = lambda key: len(first[key]) // count if count else 0
        geom_row = first["geom"][:width("geom")]
        g1, g2 = int(geom_row[0]), int(geom_row[1])
        margin = float(geom_margin[g1] + geom_margin[g2])
        gap = float(geom_gap[g1] + geom_gap[g2])
    type1, type2 = int(geom_type[g1]), int(geom_type[g2])
    epa_iterations = 16 if (type1 == GEOM_BOX and type2 == GEOM_BOX) else int(host.opt.ccd_iterations)
    horizon_capacity = int((override or {}).get("horizon", 24))
    xpos = data.geom_xpos.numpy()
    xmat = data.geom_xmat.numpy()
    results = {"dist": [], "ncontact": [], "x1": [], "x2": [], "index": [], "overflow": []}
    for world in range(worlds):
        pos1 = wp.array([wp.vec3(*[float(v) for v in xpos[world][g1]])], dtype=wp.vec3)
        rot1 = wp.array([wp.mat33(*[float(v) for v in xmat[world][g1].reshape(9)])], dtype=wp.mat33)
        pos2 = wp.array([wp.vec3(*[float(v) for v in xpos[world][g2]])], dtype=wp.vec3)
        rot2 = wp.array([wp.mat33(*[float(v) for v in xmat[world][g2].reshape(9)])], dtype=wp.mat33)
        outs = {
            "dist": wp.zeros(1, dtype=float), "ncontact": wp.zeros(1, dtype=int),
            "x1": wp.zeros(1, dtype=wp.vec3), "x2": wp.zeros(1, dtype=wp.vec3),
            "index": wp.zeros(1, dtype=int), "overflow": wp.zeros(1, dtype=int),
        }
        horizon_buffer = wp.zeros(horizon_capacity, dtype=int)
        wp.launch(kernel, dim=1, inputs=[
            pos1, rot1, pos2, rot2,
            model.geom_size, model.geom_type, model.geom_dataid,
            model.mesh_vertadr, model.mesh_vertnum, model.mesh_graphadr,
            model.mesh_vert, model.mesh_graph,
            model.mesh_polynum, model.mesh_polyadr, model.mesh_polynormal,
            model.mesh_polyvertadr, model.mesh_polyvertnum, model.mesh_polyvert,
            model.mesh_polymapadr, model.mesh_polymapnum, model.mesh_polymap,
            g1, g2, float(host.opt.ccd_tolerance), int(host.opt.ccd_iterations),
            epa_iterations, margin, gap,
            horizon_buffer,
            outs["dist"], outs["ncontact"], outs["x1"], outs["x2"], outs["index"],
            outs["overflow"]])
        wp.synchronize_device()
        results["dist"].append(float(outs["dist"].numpy()[0]))
        results["ncontact"].append(int(outs["ncontact"].numpy()[0]))
        results["x1"].append([float(v) for v in outs["x1"].numpy()[0]])
        results["x2"].append([float(v) for v in outs["x2"].numpy()[0]])
        results["index"].append(int(outs["index"].numpy()[0]))
        results["overflow"].append(int(outs["overflow"].numpy()[0]))
    return {"case": case, "worlds": worlds, "g1": g1, "g2": g2,
            "type1": type1, "type2": type2, "margin": margin, "gap": gap,
            "epa_iterations": epa_iterations, "horizon_capacity": horizon_capacity,
            "results": results}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device", default="cuda")
    parser.add_argument("--native-only", action="store_true")
    args = parser.parse_args()
    import mujoco
    import numpy as np
    WORK.mkdir(parents=True, exist_ok=True)
    CASES.mkdir(parents=True, exist_ok=True)

    report = {
        "schema": 1,
        "kind": "qa-g5-independent-upstream-capture",
        "device": args.device,
        "cases": {},
        "capacity_cases": {},
        "contact_cases": list(CONTACT_CASES),
        "witness_cases": list(WITNESS_CASES),
        "provenance": {"script_sha256": sha256(Path(__file__)),
                       "reads_product_tree": False},
    }

    if not args.native_only:
        import mujoco_warp as mjw
        import warp as wp
        wp.config.kernel_cache_dir = str(ROOT / "target/qa-g5/warp-cache")
        wp.set_device(args.device)
        report["provenance"]["versions"] = {
            "python": ".".join(map(str, sys.version_info[:3])),
            "mujoco": mujoco.__version__, "warp": wp.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"), "numpy": np.__version__,
            "warp_device": str(wp.get_device()),
        }

        # 1) 10 case 上游捕获 + 输入 bins（GPU 位型）
        for name in ALL10:
            doc, worlds, qpos, qvel, xml = load_reference(name)
            host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
            first = capture_contacts(host, qpos, qvel, worlds)
            second = capture_contacts(host, qpos, qvel, worlds)
            raw_same = first == second
            model = mjw.put_model(host)
            data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
            data.qpos.assign(qpos)
            data.qvel.assign(qvel)
            mjw.kinematics(model, data)
            prepare = write_case_inputs(name, name, host, model, data, worlds)
            report["cases"][name] = {
                "worlds": worlds, "xml": xml,
                "count": first["count"], "written": first["written"],
                "two_captures_raw_bitwise_identical": raw_same,
                "fields": first["fields"],
                "nccd": first["nccd"], "overflow": first["overflow"],
                "geom_xpos": first["geom_xpos"], "geom_xmat": first["geom_xmat"],
                "input_bins": prepare["arrays"],
            }
            print(f"{name}: count={first['count']} raw_same={raw_same}")

        # 2) witness（multicontact 三例，直接 ccd）+ 姿态扫描（找 is_discrete/EPA 溢出）
        witness = {"cases": {}, "sweep": []}
        for name in WITNESS_CASES:
            doc, worlds, qpos, qvel, xml = load_reference(name)
            host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
            witness["cases"][name] = capture_witness(name, host, qpos, worlds)
            model = mjw.put_model(host)
            data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
            data.qpos.assign(qpos)
            data.qvel.assign(np.zeros((worlds, host.nv), dtype=np.float32))
            mjw.kinematics(model, data)
            write_case_inputs(name, name, host, model, data, worlds, kind="witness")
            print(f"witness {name}: " + str(witness["cases"][name]["results"]["dist"]))
        # 扫描：用三个 base case 的模型，qpos 扰动生成更多 world 场景（同一配对）
        rng = np.random.default_rng(20251025)
        for name in WITNESS_CASES:
            doc, base_worlds, base_qpos, base_qvel, xml = load_reference(name)
            host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
            nq = host.nq
            # 找 free joint 的 qpos 段（位置 3 + 四元数 4）；没有则只扰动前 7 位
            free_adr = None
            for joint in range(host.njnt):
                if int(host.jnt_type[joint]) == int(mujoco.mjtJoint.mjJNT_FREE):
                    free_adr = int(host.jnt_qposadr[joint])
                    break
            scenarios = []
            for _ in range(24):
                qpos = base_qpos[0].copy()
                axis = rng.normal(size=3).astype(np.float64)
                axis /= np.linalg.norm(axis)
                angle = float(rng.uniform(-1.4, 1.4))
                quat = np.array([np.cos(angle / 2)] + list(axis * np.sin(angle / 2)),
                                dtype=np.float32)
                if free_adr is not None:
                    qpos[free_adr:free_adr + 3] = rng.uniform(
                        -0.08, 0.08, size=3).astype(np.float32)
                    qpos[free_adr + 3:free_adr + 7] = quat
                scenarios.append(qpos)
            swept = np.stack(scenarios, axis=0)
            capture = capture_witness(name, host, swept, len(scenarios))
            witness["sweep"].append(capture)
            model = mjw.put_model(host)
            data = mjw.make_data(host, nworld=len(scenarios), nconmax=4096, njmax=20000)
            data.qpos.assign(swept)
            data.qvel.assign(np.zeros((len(scenarios), host.nv), dtype=np.float32))
            mjw.kinematics(model, data)
            write_case_inputs(name + "-sweep", name, host, model, data, len(scenarios),
                              kind="witness", case_label=name + "-sweep")
            print(f"witness sweep {name}: n={len(scenarios)}")
        (EVIDENCE / "qa-witness-gpu.json").write_text(
            json.dumps(witness, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")

        # 2b) 强制小 horizon 容量（上游 ccd() 的 horizon 数组长度即容量）→ HORIZON 溢出分支
        forced = {}
        for name in WITNESS_CASES:
            doc, worlds, qpos, qvel, xml = load_reference(name)
            host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
            geom_margin = squeeze(host.geom_margin).tolist() if hasattr(host, "geom_margin") else None
            # 取 base pair 与 margin/gap（与设备输入同口径）
            ref = json.loads((REF / f"{name}.json").read_text(encoding="utf-8"))
            fields = ref["output"]["fields"]
            count = int(ref["output"]["count"])
            geom_row = fields["geom"]["values"][: count and 2 or 2]
            g1, g2 = int(geom_row[0]), int(geom_row[1])
            margin = float(squeeze(host.geom_margin)[g1] + squeeze(host.geom_margin)[g2])
            gap = float(squeeze(host.geom_gap)[g1] + squeeze(host.geom_gap)[g2])
            forced[name] = capture_witness(
                name, host, qpos, worlds,
                override={"g1": g1, "g2": g2, "margin": margin, "gap": gap, "horizon": 2})
            print(f"forced-horizon {name}: overflow=" + str(forced[name]["results"]["overflow"]))
        witness["forced_horizon"] = forced
        # 设备输入：从同名 case 目录复制 bins，meta 覆写 max_horizon / kind=witness
        import shutil
        for name in WITNESS_CASES:
            src = CASES / name
            dst = CASES / (name + "-h2")
            if dst.exists():
                shutil.rmtree(dst)
            dst.mkdir(parents=True)
            for path in src.iterdir():
                if path.name == "meta.txt":
                    continue
                shutil.copyfile(path, dst / path.name)
            meta = {}
            for line in (src / "meta.txt").read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition(" ")
                meta[key] = value
            meta["case"] = name + "-h2"
            meta["kind"] = "witness"
            meta["max_horizon"] = "2"
            write_meta(dst, list(meta.items()))
        (EVIDENCE / "qa-witness-gpu.json").write_text(
            json.dumps(witness, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")

        # 2c) 自造 is_discrete 场景：两个零 margin BOX（上游 is_discrete 分支）
        custom_dir = ROOT / "target/qa-g5/custom"
        custom_dir.mkdir(parents=True, exist_ok=True)
        box_box_xml = (
            '<mujoco model="qa_box_box">\n  <option gravity="0 0 0"/>\n'
            "  <worldbody>\n"
            '    <body name="b" pos="0.02 0.01 0.03"><freejoint/>'
            '<geom name="g0" type="box" size="0.1 0.08 0.06" mass="1"/></body>\n'
            '    <geom name="g1" type="box" size="0.12 0.1 0.09"/>\n'
            "  </worldbody>\n</mujoco>\n"
        )
        box_box_path = custom_dir / "box-box.xml"
        box_box_path.write_text(box_box_xml, encoding="utf-8")
        host = mujoco.MjModel.from_xml_path(str(box_box_path))
        rng2 = np.random.default_rng(20251026)
        free_adr = None
        for joint in range(host.njnt):
            if int(host.jnt_type[joint]) == int(mujoco.mjtJoint.mjJNT_FREE):
                free_adr = int(host.jnt_qposadr[joint])
                break
        base = np.zeros((2, host.nq), dtype=np.float32)
        base[1, free_adr:free_adr + 3] = np.array([0.04, -0.02, 0.01], dtype=np.float32)
        sweep = []
        for _ in range(24):
            q = np.zeros(host.nq, dtype=np.float32)
            axis = rng2.normal(size=3)
            axis /= np.linalg.norm(axis)
            angle = float(rng2.uniform(-2.8, 2.8))
            quat = np.array([np.cos(angle / 2)] + list(axis * np.sin(angle / 2)),
                            dtype=np.float32)
            q[free_adr:free_adr + 3] = rng2.uniform(-0.07, 0.07, size=3).astype(np.float32)
            q[free_adr + 3:free_adr + 7] = quat
            sweep.append(q)
        sweep = np.stack(sweep, axis=0)
        discret = capture_witness("qa-box-box", host, base, 2,
                                  override={"g1": 0, "g2": 1, "margin": 0.0, "gap": 0.0})
        discret_sweep = capture_witness("qa-box-box", host, sweep, len(sweep),
                                        override={"g1": 0, "g2": 1, "margin": 0.0, "gap": 0.0})
        witness["custom_is_discrete"] = {"base": discret, "sweep": discret_sweep}
        model = mjw.put_model(host)
        for label, poses, n in (("qa-box-box", base, 2), ("qa-box-box-sweep", sweep, len(sweep))):
            data = mjw.make_data(host, nworld=n, nconmax=4096, njmax=20000)
            data.qpos.assign(poses)
            data.qvel.assign(np.zeros((n, host.nv), dtype=np.float32))
            mjw.kinematics(model, data)
            write_custom_inputs(label, host, model, data, n, 0, 1,
                                int(squeeze(model.geom_type.numpy()).tolist()[0]), 6,
                                0.0, 0.0, "witness", case_label=label)
        print("custom is_discrete scene captured")

        # 2d) 自造 hfield 溢出场景（13x13 平地 + 大球，> MJ_MAXCONPAIR 个候选棱柱）
        hfield_xml = (
            '<mujoco model="qa_hfield_overflow">\n  <option gravity="0 0 0"/>\n'
            '  <asset><hfield name="terrain" nrow="13" ncol="13" size="1 1 0.1 0.1" elevation="' +
            " ".join("0" for _ in range(169)) + '"/></asset>\n'
            "  <worldbody>\n"
            '    <geom name="terrain" type="hfield" hfield="terrain"/>\n'
            '    <body name="b" pos="0 0 0.6"><freejoint/>'
            '<geom name="s" type="sphere" size="2.5" mass="1"/></body>\n'
            "  </worldbody>\n</mujoco>\n"
        )
        overflow_path = custom_dir / "hfield-overflow.xml"
        overflow_path.write_text(hfield_xml, encoding="utf-8")
        host = mujoco.MjModel.from_xml_path(str(overflow_path))
        qpos = np.asarray([[0.0, 0.0, 0.6, 1.0, 0.0, 0.0, 0.0]], dtype=np.float32)
        qvel = np.zeros((1, host.nv), dtype=np.float32)
        overflow_capture = capture_contacts(host, qpos, qvel, 1)
        model = mjw.put_model(host)
        data = mjw.make_data(host, nworld=1, nconmax=4096, njmax=20000)
        data.qpos.assign(qpos)
        data.qvel.assign(qvel)
        mjw.kinematics(model, data)
        write_custom_inputs(
            "hfield-overflow", host, model, data, 1, 0, 1, 1, 2, 0.0, 0.0, "hfield",
            params_source={"fields": overflow_capture["fields"],
                           "count": overflow_capture["count"]})
        witness["custom_hfield_overflow"] = {
            "count": overflow_capture["count"],
            "written": overflow_capture["written"],
            "fields": overflow_capture["fields"],
            "geom_xpos": overflow_capture["geom_xpos"],
            "geom_xmat": overflow_capture["geom_xmat"],
            "overflow": overflow_capture["overflow"],
            "nccd": overflow_capture["nccd"],
        }
        (EVIDENCE / "qa-witness-gpu.json").write_text(
            json.dumps(witness, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        print("custom hfield overflow captured: count=" + str(overflow_capture["count"]) +
              " overflow=" + str(overflow_capture["overflow"]))
        capacity = {}
        # (a) naconmax 截断：sdf-box 80 接触，容量压到 40
        name = "sdf-box"
        doc, worlds, qpos, qvel, xml = load_reference(name)
        host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
        cap = capture_contacts(host, qpos, qvel, worlds, naconmax=40)
        capacity["sdf-box-naconmax40"] = {
            "count": cap["count"], "written": cap["written"],
            "capacity": cap["capacity"], "fields": cap["fields"],
            "geom_xpos": cap["geom_xpos"], "geom_xmat": cap["geom_xmat"],
            "nccd": cap["nccd"], "overflow": cap["overflow"],
            "source": "sdf-box",
        }
        # (b) naccdmax 截断：ellipsoid-box 两 world 各 1 次 EPA，容量压到 1
        name = "ellipsoid-box"
        doc, worlds, qpos, qvel, xml = load_reference(name)
        host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
        cap = capture_contacts(host, qpos, qvel, worlds, naccdmax=1)
        capacity["ellipsoid-box-naccdmax1"] = {
            "count": cap["count"], "written": cap["written"],
            "capacity": cap["capacity"], "fields": cap["fields"],
            "geom_xpos": cap["geom_xpos"], "geom_xmat": cap["geom_xmat"],
            "nccd": cap["nccd"], "overflow": cap["overflow"],
            "source": "ellipsoid-box",
        }
        report["capacity_cases"] = capacity
        # 为设备探针派生容量场景输入（同一份位姿/数组，只改 meta 容量）
        def derive_capacity(dirname, source, naconmax, naccdmax, efc_width=None):
            import shutil
            src = CASES / source
            dst = CASES / dirname
            if dst.exists():
                shutil.rmtree(dst)
            dst.mkdir(parents=True)
            for path in src.iterdir():
                if path.name == "meta.txt":
                    continue
                shutil.copyfile(path, dst / path.name)
            meta = {}
            for line in (src / "meta.txt").read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition(" ")
                meta[key] = value
            meta["case"] = dirname
            meta["naconmax"] = str(naconmax)
            meta["naccdmax"] = str(naccdmax)
            if efc_width is not None:
                meta["efc_width"] = str(efc_width)
            write_meta(dst, list(meta.items()))

        derive_capacity("sdf-box-naconmax40", "sdf-box", 40, 8192)
        derive_capacity("ellipsoid-box-naccdmax1", "ellipsoid-box", 8192, 1)
        # nworld=2 上传布局：hfield 场景即 2 world（cases/hfield-sphere 已是 2 world）
        print("derived capacity case inputs")
        (EVIDENCE / "qa-upstream-gpu.json").write_text(
            json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        print("wrote qa-upstream-gpu.json / qa-witness-gpu.json")

    # 4) native f64（CPU，无需 GPU）
    native = {"schema": 1, "kind": "qa-native-f64-contact-readings", "cases": {}}
    for name in CONTACT_CASES:
        doc, worlds, qpos, qvel, xml = load_reference(name)
        host = mujoco.MjModel.from_xml_path(str(ROOT / xml))
        model = mujoco.MjModel.from_xml_path(str(ROOT / xml))
        data = mujoco.MjData(model)
        readings = []
        for world in range(worlds):
            data.qpos[:] = np.asarray(qpos[world], dtype=np.float64)
            data.qvel[:] = np.asarray(qvel[world], dtype=np.float64)
            mujoco.mj_forward(model, data)
            ncon = int(data.ncon)
            rows = []
            for index in range(ncon):
                contact = data.contact[index]
                rows.append({
                    "dist": float(contact.dist),
                    "pos": [float(v) for v in contact.pos],
                    "frame": [float(v) for v in contact.frame.reshape(9)],
                    "geom": [int(contact.geom[0]), int(contact.geom[1])],
                    "worldid": world,
                })
            readings.append(rows)
        native["cases"][name] = {"worlds": worlds, "contacts": readings}
        print(f"native {name}: " + str([len(r) for r in readings]))
    (EVIDENCE / "qa-native-f64.json").write_text(
        json.dumps(native, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("wrote qa-native-f64.json")


if __name__ == "__main__":
    main()
