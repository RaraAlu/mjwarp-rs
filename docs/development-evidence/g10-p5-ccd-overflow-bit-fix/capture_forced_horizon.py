"""G10@P5 修正片 authoring：强制 horizon 容量下直调 pinned 上游 `collision_gjk.ccd`。

目的：本片修正设备「EPA 返回 FLOAT_MAX 时多置 OverflowType.CCD(1<<4)」的偏差。
按 QA 独立复核方法（`2026-10-25-g10-p5-gpu-kernels-independent-check`）构造
**强制 horizon 容量 = 2** 的 3 个 witness case（`cylinder-box-h2` / `box-box-margin-h2` /
`mesh-mesh-h2`）：上游 `ccd()` 的 `horizon` 形参是一个 `wp.array[int]`，其长度即
`Polytope.max_horizon`（`_add_edge` 在 `n == horizon.shape[0]` 时返回 -1 → `EPA_HORIZON`），
把该数组长度设为 2 即可在**不改上游源码**的前提下确定性触发 `_epa` 的 horizon 溢出分支，
`ccd()` 返回 `dist = FLOAT_MAX`、只置 `OverflowType.EPA_HORIZON(1<<8)`。

输入复用 G5 冻结窗口的 case bins（`docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/`，
只读；由 `prepare_cases.py --device cuda` 生成，与设备探针同一份），不复制、不改写。

调用口径（与 `eval_ccd_write_contact` 的 ccd 段逐项对齐，且与设备 `overflow_probe` 同参数）：
1. Geom 由 case bins 直接构造（geom_xpos/xmat/size/type/dataid/margin + mesh_vert/graph）；
2. `geom1.margin = geom2.margin = margin`；
3. 原点平移到 geom1：`geom1.pos = 0`、`geom2.pos -= origin`；
4. `cutoff = gap`（pairid=(-1,-1)，非 collision sensor）；
5. `ccd(tolerance, cutoff, gjk_iterations, epa_iterations, geom1, geom2, type1, type2,
   geom1.pos, geom2.pos, vert, vert_index, face, face_pr, face_norm2, horizon,
   warn_overflow=True, worldid, overflow_out)`；
6. 每个 world 单独 launch（dim=1，`worldid` 显式传入），EPA 缓冲逐 pair 独占
   （G5 首跑教训：同一 warp 内核多线程共享 EPA 缓冲 + `array2d` 世界维读取不可靠）。

用法（仓库根目录；GPU 窗口内运行）：
    target/mjwarp-parallel/venv/Scripts/python.exe -B \
        docs/development-evidence/g10-p5-ccd-overflow-bit-fix/capture_forced_horizon.py \
        --device cuda --out docs/development-evidence/g10-p5-ccd-overflow-bit-fix/upstream-forced-horizon-run1.json

输出 JSON 只含位型（dist/x1/x2 同时给 f32 值与 bits）、overflow 整数、ncontact、
multiccd_idx、每例参数与输入 bin sha256、pinned 上游 revision，供 CPU 侧
`tests/g10_p5_ccd_overflow.rs` 与设备探针逐位对照。
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

from mujoco_warp._src.collision_core import Geom
from mujoco_warp._src.collision_gjk import ccd
from mujoco_warp._src.types import MJ_MAX_EPAFACES

ROOT = Path(__file__).resolve().parents[3]
CASES_ROOT = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases"
CASES = ("cylinder-box", "box-box-margin", "mesh-mesh")
HORIZON_CAPACITY = 2
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_meta(directory):
    meta = {}
    for line in (directory / "meta.txt").read_text(encoding="utf-8").splitlines():
        if line:
            key, _, value = line.partition(" ")
            meta[key] = value
    return meta


def load_bin(directory, name, dtype):
    path = directory / f"{name}.bin"
    if not path.exists():
        raise SystemExit(f"missing {path}")
    return np.fromfile(path, dtype=dtype)


def build_kernel():
    @wp.kernel(module="unique", enable_backward=False)
    def _forced_horizon_kernel(
        # Model（case bins 直接给出，与设备探针同源）：
        geom_type: wp.array[int],
        geom_dataid: wp.array[int],
        geom_size: wp.array[wp.vec3],
        geom_margin: wp.array[float],
        mesh_vertadr: wp.array[int],
        mesh_vertnum: wp.array[int],
        mesh_graphadr: wp.array[int],
        mesh_vert: wp.array[wp.vec3],
        mesh_graph: wp.array[int],
        # 按 world 抽出的 1D pose（不用 array2d 世界维读取）：
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
        # EPA 缓冲（上游产品口径；horizon 长度 = 容量 2）：
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
    ):
        geom1 = Geom()
        geom1.index = -1
        geom1.pos = pos1_in[worldid]
        geom1.rot = rot1_in[worldid]
        geom1.normal = wp.vec3(geom1.rot[0, 2], geom1.rot[1, 2], geom1.rot[2, 2])
        geom1.size = geom_size[gid1]
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
        geom2.size = geom_size[gid2]
        geom2.margin = margin
        geom2.vertadr = -1
        geom2.vertnum = -1
        geom2.graphadr = -1
        geom2.mesh_polynum = -1
        geom2.mesh_polyadr = -1

        # 仅 ccd() witness 路径需要的 mesh 拓扑（vert/graph）；多接触 poly 数组不参与。
        dataid1 = geom_dataid[gid1]
        if dataid1 >= 0 and geom_type[gid1] == 7:  # GeomType.MESH
            geom1.vertadr = mesh_vertadr[dataid1]
            geom1.vertnum = mesh_vertnum[dataid1]
            geom1.graphadr = mesh_graphadr[dataid1]
            geom1.vert = mesh_vert
            geom1.graph = mesh_graph
        dataid2 = geom_dataid[gid2]
        if dataid2 >= 0 and geom_type[gid2] == 7:  # GeomType.MESH
            geom2.vertadr = mesh_vertadr[dataid2]
            geom2.vertnum = mesh_vertnum[dataid2]
            geom2.graphadr = mesh_graphadr[dataid2]
            geom2.vert = mesh_vert
            geom2.graph = mesh_graph

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

    return _forced_horizon_kernel


def case_arrays(directory):
    """把 case bins 转成上游 `ccd()` 需要的模型数组（f32/int32，小端）。"""
    meta = read_meta(directory)
    nworld = int(meta["nworld"])
    ngeom = int(meta["ngeom"])
    # 模型数组按 ngeom 维：geom_* 是 [ngeom] / [ngeom][3] / [ngeom][3][3]。
    geom_type = load_bin(directory, "geom_type", "<i4")
    geom_dataid = load_bin(directory, "geom_dataid", "<i4")
    geom_size = load_bin(directory, "geom_size", "<f4").reshape(ngeom, 3)
    geom_margin = load_bin(directory, "geom_margin", "<f4")
    geom_xpos = load_bin(directory, "geom_xpos", "<f4").reshape(nworld, ngeom, 3)
    geom_xmat = load_bin(directory, "geom_xmat", "<f4").reshape(nworld, ngeom, 3, 3)
    mesh_vertadr = load_bin(directory, "mesh_vertadr", "<i4")
    mesh_vertnum = load_bin(directory, "mesh_vertnum", "<i4")
    mesh_graphadr = load_bin(directory, "mesh_graphadr", "<i4")
    mesh_vert = load_bin(directory, "mesh_vert", "<f4").reshape(-1, 3)
    mesh_graph = load_bin(directory, "mesh_graph", "<i4")
    return {
        "meta": meta,
        "nworld": nworld,
        "ngeom": ngeom,
        "geom_type": geom_type,
        "geom_dataid": geom_dataid,
        "geom_size": geom_size,
        "geom_margin": geom_margin,
        "geom_xpos": geom_xpos,
        "geom_xmat": geom_xmat,
        "mesh_vertadr": mesh_vertadr,
        "mesh_vertnum": mesh_vertnum,
        "mesh_graphadr": mesh_graphadr,
        "mesh_vert": mesh_vert,
        "mesh_graph": mesh_graph,
    }


def capture_case(case, kernel):
    directory = CASES_ROOT / case
    data = case_arrays(directory)
    meta = data["meta"]
    worlds = data["nworld"]
    g1 = int(meta["g1"])
    g2 = int(meta["g2"])
    margin = float(meta["margin"])
    cutoff = float(meta["gap"])
    gjk_iterations = int(meta["gjk_iterations"])
    epa_iterations = int(meta["epa_iterations"])
    tolerance = float(meta["tolerance"])

    ngeom = data["ngeom"]
    xpos = np.ascontiguousarray(data["geom_xpos"], dtype=np.float32)
    xmat = np.ascontiguousarray(data["geom_xmat"], dtype=np.float32)
    pos1 = wp.array(np.ascontiguousarray(xpos[:, g1, :]), dtype=wp.vec3)
    pos2 = wp.array(np.ascontiguousarray(xpos[:, g2, :]), dtype=wp.vec3)
    rot1 = wp.array(np.ascontiguousarray(xmat[:, g1, :, :]), dtype=wp.mat33)
    rot2 = wp.array(np.ascontiguousarray(xmat[:, g2, :, :]), dtype=wp.mat33)

    model_arrays = [
        wp.array(np.ascontiguousarray(data["geom_type"], dtype=np.int32), dtype=int),
        wp.array(np.ascontiguousarray(data["geom_dataid"], dtype=np.int32), dtype=int),
        wp.array(np.ascontiguousarray(data["geom_size"], dtype=np.float32), dtype=wp.vec3),
        wp.array(np.ascontiguousarray(data["geom_margin"], dtype=np.float32), dtype=float),
        wp.array(np.ascontiguousarray(data["mesh_vertadr"], dtype=np.int32), dtype=int),
        wp.array(np.ascontiguousarray(data["mesh_vertnum"], dtype=np.int32), dtype=int),
        wp.array(np.ascontiguousarray(data["mesh_graphadr"], dtype=np.int32), dtype=int),
        wp.array(np.ascontiguousarray(data["mesh_vert"], dtype=np.float32), dtype=wp.vec3),
        wp.array(np.ascontiguousarray(data["mesh_graph"], dtype=np.int32), dtype=int),
    ]

    # 上游 `collision_convex.py` 的 EPA 缓冲分配口径；horizon 长度 = 容量（这里 2）。
    vert = wp.empty(10 + 2 * epa_iterations, dtype=wp.vec3)
    vert_index = wp.empty(10 + 2 * epa_iterations, dtype=int)
    face = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=int)
    face_pr = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=wp.vec3)
    face_norm2 = wp.empty(6 + MJ_MAX_EPAFACES * epa_iterations, dtype=float)
    horizon = wp.empty(HORIZON_CAPACITY, dtype=int)

    dist_out = wp.empty(worlds, dtype=float)
    ncon_out = wp.empty(worlds, dtype=int)
    x1_out = wp.empty(worlds, dtype=wp.vec3)
    x2_out = wp.empty(worlds, dtype=wp.vec3)
    idx_out = wp.empty(worlds, dtype=int)
    overflow_out = wp.zeros(worlds, dtype=int)

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
            ],
        )
    wp.synchronize_device()

    dist = dist_out.numpy()
    ncon = ncon_out.numpy()
    x1 = x1_out.numpy()
    x2 = x2_out.numpy()
    idx = idx_out.numpy()
    overflow = overflow_out.numpy()

    input_bins = [
        "geom_type",
        "geom_dataid",
        "geom_size",
        "geom_margin",
        "geom_xpos",
        "geom_xmat",
        "mesh_vertadr",
        "mesh_vertnum",
        "mesh_graphadr",
        "mesh_vert",
        "mesh_graph",
    ]
    return {
        "case": case,
        "nworld": worlds,
        "horizon_capacity": HORIZON_CAPACITY,
        "pair": [g1, g2],
        "params": {
            "margin": margin,
            "cutoff": cutoff,
            "tolerance": tolerance,
            "gjk_iterations": gjk_iterations,
            "epa_iterations": epa_iterations,
        },
        "dist_bits": [int(v) for v in dist.view(np.uint32)],
        "dist": [float(v) for v in dist],
        "ncontact": [int(v) for v in ncon],
        "x1_bits": [int(v) for row in x1.view(np.uint32) for v in row],
        "x2_bits": [int(v) for row in x2.view(np.uint32) for v in row],
        "multiccd_idx": [int(v) for v in idx],
        "overflow": [int(v) for v in overflow],
        "input_bins_sha256": {
            f"{name}.bin": sha256(directory / f"{name}.bin") for name in input_bins
        },
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--device", default="cuda", choices=("cpu", "cuda"))
    parser.add_argument("--out", required=True)
    arguments = parser.parse_args()

    wp.config.kernel_cache_dir = str(ROOT / f"target/g10-p5-p58/warp-cache-{arguments.device}")
    wp.set_device(arguments.device)
    kernel = build_kernel()

    report = {
        "schema": 1,
        "kind": "g10-p5-ccd-overflow-forced-horizon-upstream",
        "task_id": "2026-10-25-g10-p5-ccd-overflow-bit-fix",
        "purpose": (
            "强制 horizon 容量 = 2 时上游 ccd() 的 overflow 位（期望 EPA_HORIZON=256，不含 CCD=16）"
        ),
        "device": arguments.device,
        "horizon_capacity": HORIZON_CAPACITY,
        "call_convention": [
            "Geom 由 G5 窗口 case bins 直接构造（geom_xpos/xmat/size/type/dataid/margin + mesh_vert/graph）",
            "geom1.margin = geom2.margin = margin；原点平移到 geom1；cutoff = gap",
            "horizon 数组长度 = 2（上游 Polytope.max_horizon = horizon.shape[0]）→ 确定性触发 _add_edge 的 -1 分支",
            "每 world 单独 launch（dim=1，worldid 显式传入），EPA 缓冲逐 pair 独占",
        ],
        "cases": {},
        "provenance": {
            "warp": wp.__version__,
            "warp_device": str(wp.get_device()),
            "mujoco": mujoco.__version__,
            "mujoco_warp": getattr(mjw, "__version__", "3.15.0"),
            "numpy": np.__version__,
            "python": ".".join(map(str, sys.version_info[:3])),
            "pinned_upstream_revision": PINNED_REVISION,
            "authoring_script": str(Path(__file__).resolve().relative_to(ROOT)).replace("\\", "/"),
            "authoring_script_sha256": sha256(__file__),
            "authoring_only": True,
            "reads_product_tree": False,
            "product_python_dependency": False,
        },
    }
    for case in CASES:
        report["cases"][case] = capture_case(case, kernel)
        entry = report["cases"][case]
        print(f"{case}: overflow={entry['overflow']} ncontact={entry['ncontact']} "
              f"dist_bits={entry['dist_bits']}")

    Path(arguments.out).write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"wrote {arguments.out} (device={arguments.device})")


if __name__ == "__main__":
    main()
