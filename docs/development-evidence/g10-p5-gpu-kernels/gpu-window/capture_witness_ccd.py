"""G5 · witness 对照 authoring：直调 pinned 上游 `collision_gjk.ccd`（authoring-only）。

背景（Arch 2026-10-25 裁决 1）：`cylinder-box`/`box-box-margin`/`mesh-mesh` 三例上游走
multicontact（设备版未实装），接触数组不做全等；但它们仍可做 **`ccd()` witness 对照**：
用同一探针输入（`gpu-window/cases/<case>/`，由 `prepare_cases.py --device cuda` 生成），
按上游产品路径 `collision_convex.eval_ccd_write_contact` 的调用方式直调上游
`collision_gjk.ccd`，取 `dist/ncontact/x1/x2/multiccd_idx/overflow`，供设备探针
（`witness_probe.exe`，同一批 device 源）对照（≤4 ULP、位级优先）。

调用口径与上游产品路径逐项对齐（不是从我们 host 代码反推）：
1. `Geom` 按上游 `collision_core.geom_collision_pair_from_types` 的字段口径手工构造
   （pos/rot/size/normal/margin/vertadr/vertnum/graphadr/mesh 数组/index=-1）；
2. `geom1.margin = geom2.margin = margin`（`includemargin`，上游 `eval_ccd_write_contact`）；
3. 原点平移到 `geom1.pos`：`geom1.pos = 0`、`geom2.pos -= origin`（平移不变性）；
4. `cutoff = gap`（非 collision sensor；本批模型 `pairid=(-1,-1)`，`gap = geom_gap1+geom_gap2`）；
5. `ccd(tolerance, cutoff, gjk_iterations, epa_iterations, geom1, geom2, type1, type2,
   geom1.pos, geom2.pos, epa_buffers..., warn_overflow=True, worldid, overflow)`；
6. `tolerance`/`gjk_iterations`/`epa_iterations` 取 `cases/<case>/meta.txt`（与设备探针同值；
   `box-box-margin` 的 16 是上游 `collision_convex.py:1272` 的 `nboxbox == ncollision → 16`）。

2026-10-25 首跑踩到三个 authoring 缺陷（已在代码里修掉，留档避免复发）：
- **warp CUDA 的 launch 线程数可以多于 `dim`**：内核必须显式守卫 `worldid >= worlds`；
- **`wp.array2d` 世界维在内核里读取不可靠**（首跑 world1 得到 world0 结果）→ 改为在 Python
  侧按 world 抽出 `pos1/pos2/rot1/rot2` **一维数组**传入；
- **同名 EPA 缓冲被多线程共享**（上游产品是每 pair 独立 `epa_vert[ccdid]`）→ 改为
  **每 world 单独 launch（dim=1，`worldid` 显式传入）**，顺序执行、缓冲复用安全。
  同时输出 `debug_kernel_inputs` 供证据核对（每 world 输入必须不同）。

输入互证（同一脚本内自查）：
- 本脚本按参考 JSON 重建模型/数据后跑 `kinematics`，`data.geom_xpos/xmat` 与
  `cases/<case>/geom_xpos.bin|xmat.bin` 位级比较（case bins 由 `prepare_cases.py --device cuda`
  生成）：`--device cuda` 应位级相等；`--device cpu` 只记录（上游自身 CPU/CUDA 后端差）；
- `ccd dist + margin` 与交付参考 `fixtures/g10-p5/references/<case>.json` 的接触 `dist` 做口径互证：
  单接触 case 全部接触行 dist 相同（应位级命中）；`box-box-margin` 多接触行走 `multicontact`
  逐点距离，只有与 EPA witness 对应的行等于 `ccd dist + margin`，故按 world 记最小 ULP。

用法（仓库根目录；cpu 不占 GPU，cuda 在 GPU 窗口内运行）：
    target/mjwarp-parallel/venv/Scripts/python.exe -B \
        docs/development-evidence/g10-p5-gpu-kernels/gpu-window/capture_witness_ccd.py --device cpu
    ... --device cuda      # 窗口内：绑定上游 f32 GPU witness（正式判据参考）

输出：`gpu-window/witness-<device>.json`。
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

from mujoco_warp._src.collision_core import Geom
from mujoco_warp._src.collision_gjk import ccd
from mujoco_warp._src.types import MJ_MAX_EPAFACES, MJ_MAX_EPAHORIZON

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window"
CASE_ROOT = EVIDENCE / "cases"
REF = ROOT / "fixtures/g10-p5/references"
CASES = ("cylinder-box", "box-box-margin", "mesh-mesh")
DEVICE = "cpu"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def meta_map(directory):
    out = {}
    for line in (directory / "meta.txt").read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        key, _, value = line.partition(" ")
        out[key] = value
    return out


def build_kernel():
    @wp.kernel(module="unique", enable_backward=False)
    def _witness_kernel(
        # Model（与 world 维无关）：
        geom_type: wp.array[int],
        geom_dataid: wp.array2d[int],
        geom_size: wp.array2d[wp.vec3],
        mesh_vertadr: wp.array[int],
        mesh_vertnum: wp.array[int],
        mesh_graphadr: wp.array[int],
        mesh_vert: wp.array[wp.vec3],
        mesh_graph: wp.array[int],
        mesh_polynum: wp.array[int],
        mesh_polyadr: wp.array[int],
        mesh_polynormal: wp.array[wp.vec3],
        mesh_polyvertadr: wp.array[int],
        mesh_polyvertnum: wp.array[int],
        mesh_polyvert: wp.array[int],
        mesh_polymapadr: wp.array[int],
        mesh_polymapnum: wp.array[int],
        mesh_polymap: wp.array[int],
        # Data in（按 world 抽出的一维数组；不依赖 array2d 世界维读取）：
        pos1_in: wp.array[wp.vec3],
        pos2_in: wp.array[wp.vec3],
        rot1_in: wp.array[wp.mat33],
        rot2_in: wp.array[wp.mat33],
        # In：
        gid1: int,
        gid2: int,
        worldid: int,
        margin: float,
        cutoff: float,
        gjk_iterations: int,
        epa_iterations: int,
        tolerance: float,
        # EPA 缓冲（上游 `collision_convex.py` 分配口径）
        vert: wp.array[wp.vec3],
        vert_index: wp.array[int],
        face: wp.array[int],
        face_pr: wp.array[wp.vec3],
        face_norm2: wp.array[float],
        horizon: wp.array[int],
        # Data out：
        dist_out: wp.array[float],
        ncon_out: wp.array[int],
        x1_out: wp.array[wp.vec3],
        x2_out: wp.array[wp.vec3],
        idx_out: wp.array[int],
        overflow_out: wp.array[int],
        # Debug out（复核每 world 输入确实不同）
        debug_pos1_out: wp.array[wp.vec3],
        debug_pos2_out: wp.array[wp.vec3],
    ):
        # 每次 launch 只跑 1 个 world（dim=1，worldid 显式传入）：
        # warp CUDA 的多线程共享 EPA 缓冲/struct 语义在 authoring 内核里不可靠，
        # 逐 world 独立 launch 可保证与产品路径（每 pair 独立 ccdid 缓冲）同口径。
        geom1 = Geom()
        geom1.index = -1
        geom1.pos = pos1_in[worldid]
        geom1.rot = rot1_in[worldid]
        geom1.normal = wp.vec3(geom1.rot[0, 2], geom1.rot[1, 2], geom1.rot[2, 2])
        geom1.size = geom_size[0, gid1]
        geom1.margin = margin
        geom1.vertadr = -1
        geom1.vertnum = -1
        geom1.graphadr = -1
        geom1.mesh_polynum = -1
        geom1.mesh_polyadr = -1

        geom2 = Geom()
        geom2.index = -1
        geom2.pos = pos2_in[worldid]
        geom2.rot = rot2_in[worldid]
        geom2.normal = wp.vec3(geom2.rot[0, 2], geom2.rot[1, 2], geom2.rot[2, 2])
        geom2.size = geom_size[0, gid2]
        geom2.margin = margin
        geom2.vertadr = -1
        geom2.vertnum = -1
        geom2.graphadr = -1
        geom2.mesh_polynum = -1
        geom2.mesh_polyadr = -1

        dataid1 = geom_dataid[0, gid1]
        if dataid1 >= 0 and geom_type[gid1] == 7:  # GeomType.MESH
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

        dataid2 = geom_dataid[0, gid2]
        if dataid2 >= 0 and geom_type[gid2] == 7:  # GeomType.MESH
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

        debug_pos1_out[worldid] = geom1.pos
        debug_pos2_out[worldid] = geom2.pos

        # 上游 `eval_ccd_write_contact`：原点平移到 geom1。
        origin = geom1.pos
        geom1.pos = wp.vec3(0.0)
        geom2.pos = geom2.pos - origin

        dist, ncontact, x1, x2, idx = ccd(
            tolerance,
            cutoff,
            gjk_iterations,
            epa_iterations,
            geom1,
            geom2,
            geom_type[gid1],
            geom_type[gid2],
            geom1.pos,
            geom2.pos,
            vert,
            vert_index,
            face,
            face_pr,
            face_norm2,
            horizon,
            True,
            worldid,
            overflow_out,
        )
        dist_out[worldid] = dist
        ncon_out[worldid] = ncontact
        x1_out[worldid] = x1
        x2_out[worldid] = x2
        idx_out[worldid] = idx

    return _witness_kernel


def capture_case(case, kernel):
    directory = CASE_ROOT / case
    meta = meta_map(directory)
    reference = json.loads((REF / f"{case}.json").read_text(encoding="utf-8"))
    worlds = int(meta["nworld"])
    ngeom = int(meta["ngeom"])
    qpos = np.asarray(reference["input"]["qpos"]["values"], dtype=np.float32).reshape(worlds, -1)
    qvel = np.asarray(reference["input"]["qvel"]["values"], dtype=np.float32).reshape(
        worlds, reference["input"]["qvel"]["shape"][1]
    )
    host = mujoco.MjModel.from_xml_path(str(ROOT / reference["input"]["path"]))
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    wp.synchronize_device()

    # 输入互证 ①：kinematics 后的 geom_xpos/xmat 与设备探针输入（CUDA 捕获的 case bins）比较。
    # 同后端（--device cuda）应位级相等；--device cpu 与 CUDA bins 存在上游自身后端差
    # （mesh-mesh 的 xmat 有 2 个分量 -2.65e-9 ↔ 0.0），故 CPU 只记录不判失败。
    run_xpos = np.ascontiguousarray(data.geom_xpos.numpy(), dtype=np.float32).reshape(-1)
    run_xmat = np.ascontiguousarray(data.geom_xmat.numpy(), dtype=np.float32).reshape(-1)
    case_xpos = np.fromfile(directory / "geom_xpos.bin", dtype="<f4")
    case_xmat = np.fromfile(directory / "geom_xmat.bin", dtype="<f4")

    def diff_report(expected, actual):
        expected_bits = expected.view(np.uint32)
        actual_bits = actual.view(np.uint32)
        mismatch = expected_bits != actual_bits
        return {
            "components": int(expected_bits.size),
            "bit_equal_components": int((~mismatch).sum()),
            "bit_equal_all": bool((~mismatch).all()),
            "max_abs": float(np.abs(expected - actual).max()) if expected_bits.size else 0.0,
        }

    xpos_check = diff_report(case_xpos, run_xpos)
    xmat_check = diff_report(case_xmat, run_xmat)

    g1 = int(meta["g1"])
    g2 = int(meta["g2"])
    margin = float(meta["margin"])
    cutoff = float(meta["gap"])
    gjk_iterations = int(meta["gjk_iterations"])
    epa_iterations = int(meta["epa_iterations"])
    tolerance = float(meta["tolerance"])

    # 按 world 抽出 1D pose 数组（绕开 array2d 世界维读取；见文件头“authoring 缺陷”）。
    xpos = np.ascontiguousarray(data.geom_xpos.numpy().reshape(worlds, ngeom, 3), dtype=np.float32)
    xmat = np.ascontiguousarray(
        data.geom_xmat.numpy().reshape(worlds, ngeom, 3, 3), dtype=np.float32
    )
    pos1 = wp.array(np.ascontiguousarray(xpos[:, g1, :]), dtype=wp.vec3)
    pos2 = wp.array(np.ascontiguousarray(xpos[:, g2, :]), dtype=wp.vec3)
    rot1 = wp.array(np.ascontiguousarray(xmat[:, g1, :, :]), dtype=wp.mat33)
    rot2 = wp.array(np.ascontiguousarray(xmat[:, g2, :, :]), dtype=wp.mat33)

    vert = wp.empty(10 + 2 * epa_iterations, dtype=wp.vec3)
    vert_index = wp.empty(10 + 2 * epa_iterations, dtype=int)
    face = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=int)
    face_pr = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=wp.vec3)
    face_norm2 = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=float)
    horizon = wp.empty(MJ_MAX_EPAHORIZON, dtype=int)

    dist_out = wp.empty(worlds, dtype=float)
    ncon_out = wp.empty(worlds, dtype=int)
    x1_out = wp.empty(worlds, dtype=wp.vec3)
    x2_out = wp.empty(worlds, dtype=wp.vec3)
    idx_out = wp.empty(worlds, dtype=int)
    overflow_out = wp.zeros(worlds, dtype=int)
    debug_pos1_out = wp.empty(worlds, dtype=wp.vec3)
    debug_pos2_out = wp.empty(worlds, dtype=wp.vec3)

    model_arrays = [
        model.geom_type,
        model.geom_dataid,
        model.geom_size,
        model.mesh_vertadr,
        model.mesh_vertnum,
        model.mesh_graphadr,
        model.mesh_vert,
        model.mesh_graph,
        model.mesh_polynum,
        model.mesh_polyadr,
        model.mesh_polynormal,
        model.mesh_polyvertadr,
        model.mesh_polyvertnum,
        model.mesh_polyvert,
        model.mesh_polymapadr,
        model.mesh_polymapnum,
        model.mesh_polymap,
    ]
    for world in range(worlds):
        wp.launch(
            kernel,
            dim=1,
            inputs=model_arrays
            + [
                pos1,
                pos2,
                rot1,
                rot2,
                g1,
                g2,
                world,
                margin,
                cutoff,
                gjk_iterations,
                epa_iterations,
                tolerance,
                vert,
                vert_index,
                face,
                face_pr,
                face_norm2,
                horizon,
                dist_out,
                ncon_out,
                x1_out,
                x2_out,
                idx_out,
                overflow_out,
                debug_pos1_out,
                debug_pos2_out,
            ],
        )
    wp.synchronize_device()

    dist = dist_out.numpy()
    ncon = ncon_out.numpy()
    x1 = x1_out.numpy()
    x2 = x2_out.numpy()
    idx = idx_out.numpy()
    overflow = overflow_out.numpy()
    debug_pos1 = debug_pos1_out.numpy()
    debug_pos2 = debug_pos2_out.numpy()

    # 输入互证 ②：`ccd dist + margin` 与交付参考（同后端 CUDA 捕获）接触 dist 的口径互证。
    # 单接触 case 全部接触行 dist 相同 → 应位级命中；box-box-margin 的多接触行走
    # `multicontact` 逐点距离，只有与 EPA witness 对应的行等于 `ccd dist + margin`，
    # 故按 world 记录「是否命中参考 dist 集合」+ 最小 ULP。
    ref_count = int(reference["output"]["count"])
    ref_dist = np.asarray(reference["output"]["fields"]["dist"]["values"], dtype=np.float32)
    ref_worldid = np.asarray(reference["output"]["fields"]["worldid"]["values"], dtype=np.int32)
    corrected = np.asarray(dist + np.float32(margin), dtype=np.float32)

    def ulp(left, right):
        left_key = int(np.float32(left).view(np.int32)) if left >= 0 else (
            0x8000_0000 - int(np.float32(left).view(np.int32)))
        right_key = int(np.float32(right).view(np.int32)) if right >= 0 else (
            0x8000_0000 - int(np.float32(right).view(np.int32)))
        return abs(left_key - right_key)

    per_world = []
    for world in range(worlds):
        candidates = ref_dist[ref_worldid == world]
        if candidates.size == 0:
            per_world.append({"worldid": world, "reference_rows": 0})
            continue
        best_ulp = min(ulp(corrected[world], value) for value in candidates)
        per_world.append({
            "worldid": world,
            "reference_rows": int(candidates.size),
            "witness_dist_plus_margin": float(corrected[world]),
            "witness_dist_plus_margin_bits": int(corrected.view(np.uint32)[world]),
            "min_ulp_vs_reference_contact_dist": int(best_ulp),
            "bit_equal_to_a_reference_row": bool(best_ulp == 0),
        })
    sanity = {
        "reference_contact_count": ref_count,
        "reference_dist_unique_bits": sorted({int(v) for v in ref_dist.view(np.uint32)}),
        "note": (
            "单接触 case 的参考接触 dist == ccd dist + margin；box-box-margin 多接触行 "
            "取 multicontact 逐点距离，只有 EPA witness 对应行命中，故按 world 记最小 ULP"
        ),
        "per_world": per_world,
    }

    debug_pos1_bits = [[int(v) for v in row] for row in debug_pos1.view(np.uint32)]
    debug_pos2_bits = [[int(v) for v in row] for row in debug_pos2.view(np.uint32)]
    return {
        "case": case,
        "worlds": worlds,
        "pair": [g1, g2],
        "params": {
            "margin": margin,
            "cutoff": cutoff,
            "tolerance": tolerance,
            "gjk_iterations": gjk_iterations,
            "epa_iterations": epa_iterations,
        },
        "witness": {
            "dist": [float(v) for v in dist],
            "dist_bits": [int(v) for v in dist.view(np.uint32)],
            "ncontact": [int(v) for v in ncon],
            "x1": [[float(v) for v in row] for row in x1],
            "x1_bits": [[int(v) for v in row] for row in x1.view(np.uint32)],
            "x2": [[float(v) for v in row] for row in x2],
            "x2_bits": [[int(v) for v in row] for row in x2.view(np.uint32)],
            "multiccd_idx": [int(v) for v in idx],
            "overflow": [int(v) for v in overflow],
        },
        "debug_kernel_inputs": {
            "note": "内核实际读到的 geom1.pos/geom2.pos（未平移）；每 world 必须不同",
            "geom1_pos": [[float(v) for v in row] for row in debug_pos1],
            "geom1_pos_bits": debug_pos1_bits,
            "geom2_pos": [[float(v) for v in row] for row in debug_pos2],
            "geom2_pos_bits": debug_pos2_bits,
        },
        "input_self_check": {
            "geom_xpos": xpos_check,
            "geom_xmat": xmat_check,
            "same_backend_required": True,
            "note": (
                "case bins 是上游 CUDA 捕获；--device cuda 应位级相等；--device cpu 只记录"
                "（mesh-mesh 的 xmat 有 2 个分量 -2.65e-9 ↔ 0.0，上游自身后端差）"
            ),
        },
        "sanity_vs_contact_reference": sanity,
        "reference_path": f"fixtures/g10-p5/references/{case}.json",
    }


def main():
    global DEVICE
    arguments = sys.argv[1:]
    if "--device" in arguments:
        DEVICE = arguments[arguments.index("--device") + 1]
    if DEVICE not in ("cpu", "cuda"):
        raise SystemExit("--device 只接受 cpu|cuda")
    wp.config.kernel_cache_dir = str(ROOT / f"target/g10-p5-g5/warp-cache-{DEVICE}")
    wp.set_device(DEVICE)
    kernel = build_kernel()

    report = {
        "schema": 1,
        "kind": "g5-witness-upstream-ccd",
        "task_id": "2026-10-25-g10-p5-gpu-kernels",
        "purpose": (
            "设备探针 witness 对照的上游参考：按上游产品路径口径直调 pinned "
            "`collision_gjk.ccd`（multicontact 三例）"
        ),
        "device": DEVICE,
        "call_convention": [
            "Geom 按 geom_collision_pair_from_types 的字段口径手工构造（mesh 字段齐全）",
            "geom1.margin = geom2.margin = margin(includemargin)",
            "原点平移到 geom1：geom1.pos=0, geom2.pos-=origin",
            "cutoff = gap（pairid=(-1,-1)，非 collision sensor）",
            "ccd(tolerance, cutoff, gjk_iterations, epa_iterations, geom1, geom2, type1, type2, geom1.pos, geom2.pos, ...)",
            "tolerance/gjk_iterations/epa_iterations 取 cases/<case>/meta.txt（与设备探针同值）",
            "pos/rot 按 world 抽成一维数组传入；每 world 单独 launch（dim=1），EPA 缓冲逐 pair 独占",
        ],
        "cases": {},
        "provenance": {
            "device": DEVICE,
            "warp": wp.__version__,
            "warp_device": str(wp.get_device()),
            "mujoco": mujoco.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
            "numpy": np.__version__,
            "python": ".".join(map(str, sys.version_info[:3])),
            "authoring_script": (
                "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/capture_witness_ccd.py"
            ),
            "authoring_script_sha256": sha256(__file__),
            "pinned_upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
    }
    failures = []
    for case in CASES:
        entry = capture_case(case, kernel)
        report["cases"][case] = entry
        check = entry["input_self_check"]
        same_backend = (
            check["geom_xpos"]["bit_equal_all"] and check["geom_xmat"]["bit_equal_all"]
        )
        if DEVICE == "cuda" and not same_backend:
            failures.append(f"{case}: kinematics 输入与设备 case bins 位型不等（同后端应为 0）")
        dists = entry["witness"]["dist"]
        debug_pos = [
            (tuple(one), tuple(two))
            for one, two in zip(
                entry["debug_kernel_inputs"]["geom1_pos_bits"],
                entry["debug_kernel_inputs"]["geom2_pos_bits"],
            )
        ]
        witness_x1 = entry["witness"]["x1_bits"]
        if len(set(debug_pos)) != len(debug_pos):
            failures.append(f"{case}: 内核 geom1.pos 每 world 相同，world 维读取错误")
        if len(set(tuple(row) for row in witness_x1)) != len(witness_x1):
            failures.append(f"{case}: 上游 witness x1 每 world 相同，world 维读取错误")
        best = min(row.get("min_ulp_vs_reference_contact_dist", 0) for row in
                   entry["sanity_vs_contact_reference"]["per_world"])
        print(
            f"{case}[{DEVICE}]: witness_dist={dists} "
            f"x1_world0={entry['witness']['x1'][0]} "
            f"x1_world1={entry['witness']['x1'][1]} "
            f"ncontact={entry['witness']['ncontact']} idx={entry['witness']['multiccd_idx']} "
            f"input_bit_eq={same_backend} "
            f"(xpos {check['geom_xpos']['bit_equal_components']}/{check['geom_xpos']['components']}, "
            f"xmat {check['geom_xmat']['bit_equal_components']}/{check['geom_xmat']['components']}) "
            f"dist+margin_min_ulp_vs_reference={best}"
        )
    report["failures"] = failures
    path = EVIDENCE / f"witness-{DEVICE}.json"
    path.write_text(json.dumps(report, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print("failures:", failures)
    print("wrote", path)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
