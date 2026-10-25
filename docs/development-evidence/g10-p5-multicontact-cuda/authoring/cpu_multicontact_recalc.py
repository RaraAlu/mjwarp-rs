"""G10@P5 multicontact 片：CPU 复算 multicontact（authoring/diagnostic，非产品测试）。

目的（Arch 2026-10-25 建议）：在 warp CPU 上复算 cylinder-box case 的 multicontact 分支
（上游 eval_ccd_write_contact 的 gjk -> epa -> multicontact -> 逐接触写入），验证：
1. multiccd_idx 取 EPA 最终穿透面；
2. multicontact() 返回 4 个接触、id_=0..3；
3. dists/witness/frame/pos 赋值顺序与 QA 参考 8 条（4/world）一致。

输入用 G5 冻结 case bin（与 QA 参考同源）；不创建 CUDA 上下文（只 warp CPU）。
运行：target/mjwarp-parallel/venv/Scripts/python.exe <本文件>
输出：同目录 cpu_multicontact_recalc.json
"""

import json
from pathlib import Path

import numpy as np
import warp as wp

wp.init()
from mujoco_warp._src.collision_core import geom_collision_pair
from mujoco_warp._src.collision_gjk import epa_phase, gjk_phase, multicontact
from mujoco_warp._src.math import make_frame
from mujoco_warp._src.types import mat43

ROOT = Path(__file__).resolve().parents[4]
CASES = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/cylinder-box"
REFERENCE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json"
CASE = "cylinder-box"

# 上游 convex_narrowphase 的 scratch 尺寸：npolygonmax=16（含 cylinder）；nmeshdegmax>=3（取 10 上限）。
EPA_VERT = 10 + 2 * 16
EPA_FACE = 6 + 5 * 16
EPA_HORIZON = 24
NPOLYGONMAX = 16
NMESHDEGMAX = 10


@wp.kernel
def recalc_kernel(
    # model/data
    geom_type: wp.array(dtype=int),
    geom_dataid: wp.array2d(dtype=int),
    geom_size: wp.array2d(dtype=wp.vec3),
    geom_xpos: wp.array2d(dtype=wp.vec3),
    geom_xmat: wp.array2d(dtype=wp.mat33),
    mesh_empty_i: wp.array(dtype=int),
    mesh_empty_v: wp.array(dtype=wp.vec3),
    # 参数
    tolerance: float,
    gap: float,
    margin: float,
    gjk_iterations: int,
    epa_iterations: int,
    # scratch
    epa_vert: wp.array2d(dtype=wp.vec3),
    epa_vert_index: wp.array2d(dtype=int),
    epa_face: wp.array2d(dtype=int),
    epa_pr: wp.array2d(dtype=wp.vec3),
    epa_norm2: wp.array2d(dtype=float),
    epa_horizon: wp.array2d(dtype=int),
    mc_polygon: wp.array2d(dtype=wp.vec3),
    mc_clipped: wp.array2d(dtype=wp.vec3),
    mc_pnormal: wp.array2d(dtype=wp.vec3),
    mc_pdist: wp.array2d(dtype=float),
    mc_idx1: wp.array2d(dtype=int),
    mc_idx2: wp.array2d(dtype=int),
    mc_n1: wp.array2d(dtype=wp.vec3),
    mc_n2: wp.array2d(dtype=wp.vec3),
    mc_endvert: wp.array2d(dtype=wp.vec3),
    mc_face1: wp.array2d(dtype=wp.vec3),
    mc_face2: wp.array2d(dtype=wp.vec3),
    # 输出
    out_n: wp.array(dtype=int),
    out_idx: wp.array(dtype=int),
    out_dist: wp.array2d(dtype=float),
    out_pos: wp.array2d(dtype=wp.vec3),
    out_frame: wp.array2d(dtype=wp.mat33),
    overflow: wp.array(dtype=int),
):
    world = wp.tid()
    geoms = wp.vec2i(1, 0)  # 与 meta 一致：g1=1(cylinder), g2=0(box)
    geom1, geom2 = geom_collision_pair(
        geom_type, geom_dataid, geom_size,
        mesh_empty_i, mesh_empty_i, mesh_empty_i, mesh_empty_v, mesh_empty_i,
        mesh_empty_i, mesh_empty_i, mesh_empty_v, mesh_empty_i, mesh_empty_i,
        mesh_empty_i, mesh_empty_i, mesh_empty_i, mesh_empty_i,
        geom_xpos, geom_xmat, geoms, world,
    )
    geomtype1 = 5  # CYLINDER
    geomtype2 = 6  # BOX
    geom1.margin = margin
    geom2.margin = margin
    origin = geom1.pos
    geom1.pos = wp.vec3(0.0, 0.0, 0.0)
    geom2.pos = geom2.pos - origin

    needs_epa, dist, ncontact, w1, w2, result, geom1b, geom2b = gjk_phase(
        tolerance, gap, gjk_iterations, geom1, geom2, geomtype1, geomtype2,
        geom1.pos, geom2.pos,
    )
    multiccd_idx = -1
    if needs_epa:
        dist, ncontact, w1, w2, multiccd_idx = epa_phase(
            tolerance, epa_iterations, result, geom1b, geom2b, geomtype1, geomtype2,
            epa_vert[world], epa_vert_index[world], epa_face[world], epa_pr[world],
            epa_norm2[world], epa_horizon[world], False, world, overflow,
        )
    else:
        geom1b = geom1
        geom2b = geom2

    out_n[world] = 0
    out_idx[world] = multiccd_idx
    for i in range(4):
        out_dist[world, i] = 0.0
        out_pos[world, i] = wp.vec3(0.0, 0.0, 0.0)
        out_frame[world, i] = wp.mat33(0.0)

    if dist >= gap:
        return
    dist = dist + margin

    ncollision = 1
    witness1 = mat43()
    witness2 = mat43()
    dists = wp.vec4()
    witness1[0] = w1
    witness2[0] = w2
    if multiccd_idx > -1:
        ncollision, witness1, witness2, dists = multicontact(
            mc_polygon[world], mc_clipped[world], mc_pnormal[world], mc_pdist[world],
            mc_idx1[world], mc_idx2[world], mc_n1[world], mc_n2[world],
            mc_endvert[world], mc_face1[world], mc_face2[world],
            epa_vert[world], epa_vert_index[world], epa_face[world, multiccd_idx],
            w1, w2, geom1b, geom2b, geomtype1, geomtype2,
        )
        if ncollision < 1:
            ncollision = 1
            witness1[0] = w1
            witness2[0] = w2

    if dist <= margin:
        frame = make_frame(witness1[0] - witness2[0])
    else:
        frame = make_frame(witness2[0] - witness1[0])
    out_n[world] = ncollision
    for i in range(ncollision):
        out_dist[world, i] = dists[i] if ncollision > 1 else dist
        out_pos[world, i] = 0.5 * (witness1[i] + witness2[i]) + origin
        out_frame[world, i] = frame


def read_meta(directory: Path) -> dict:
    meta = {}
    for line in (directory / "meta.txt").read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if len(parts) == 2:
            meta[parts[0]] = parts[1]
    return meta


def read_bin(directory: Path, name: str, dtype) -> np.ndarray:
    return np.fromfile(directory / f"{name}.bin", dtype=dtype)


def ulp_distance(a: np.float32, b: np.float32) -> int:
    def ordered(v) -> int:
        bits = int(np.asarray(v, dtype=np.float32).view(np.int32).item())
        return -(bits & 0x7FFFFFFF) if bits < 0 else bits

    return abs(ordered(a) - ordered(b))


def main() -> int:
    meta = read_meta(CASES)
    worlds = int(meta["nworld"])
    ngeom = int(meta["ngeom"])
    tolerance = float(meta["tolerance"])
    gap = float(meta["gap"])
    margin = float(meta["margin"])
    gjk_iterations = int(meta["gjk_iterations"])
    epa_iterations = int(meta["epa_iterations"])

    xpos = read_bin(CASES, "geom_xpos", np.float32).reshape(worlds, ngeom, 3)
    xmat = read_bin(CASES, "geom_xmat", np.float32).reshape(worlds, ngeom, 9).reshape(worlds, ngeom, 3, 3)
    size = read_bin(CASES, "geom_size", np.float32).reshape(1, ngeom, 3)
    gtype = read_bin(CASES, "geom_type", np.int32)
    gdataid = read_bin(CASES, "geom_dataid", np.int32).reshape(1, ngeom)

    device = "cpu"
    empty_i = wp.array(np.zeros(1, dtype=np.int32), dtype=int, device=device)
    empty_v = wp.array(np.zeros((1, 3), dtype=np.float32), dtype=wp.vec3, device=device)

    dist_out = wp.array(np.zeros((worlds, 4), dtype=np.float32), dtype=float, device=device)
    pos_out = wp.array(np.zeros((worlds, 4, 3), dtype=np.float32), dtype=wp.vec3, device=device)
    frame_out = wp.array(np.zeros((worlds, 4, 3, 3), dtype=np.float32), dtype=wp.mat33, device=device)
    n_out = wp.array(np.zeros(worlds, dtype=np.int32), dtype=int, device=device)
    idx_out = wp.array(np.zeros(worlds, dtype=np.int32), dtype=int, device=device)
    overflow = wp.array(np.zeros(worlds, dtype=np.int32), dtype=int, device=device)

    wp.launch(
        recalc_kernel,
        dim=worlds,
        inputs=[
            wp.array(gtype, dtype=int, device=device),
            wp.array(gdataid, dtype=int, device=device),
            wp.array(size, dtype=wp.vec3, device=device),
            wp.array(xpos, dtype=wp.vec3, device=device),
            wp.array(xmat, dtype=wp.mat33, device=device),
            empty_i,
            empty_v,
            tolerance,
            gap,
            margin,
            gjk_iterations,
            epa_iterations,
            wp.array(np.zeros((worlds, EPA_VERT, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, EPA_VERT), dtype=np.int32), dtype=int, device=device),
            wp.array(np.zeros((worlds, EPA_FACE), dtype=np.int32), dtype=int, device=device),
            wp.array(np.zeros((worlds, EPA_FACE, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, EPA_FACE), dtype=np.float32), dtype=float, device=device),
            wp.array(np.zeros((worlds, EPA_HORIZON), dtype=np.int32), dtype=int, device=device),
            wp.array(np.zeros((worlds, 2 * NPOLYGONMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, 2 * NPOLYGONMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NPOLYGONMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NPOLYGONMAX), dtype=np.float32), dtype=float, device=device),
            wp.array(np.zeros((worlds, NMESHDEGMAX), dtype=np.int32), dtype=int, device=device),
            wp.array(np.zeros((worlds, NMESHDEGMAX), dtype=np.int32), dtype=int, device=device),
            wp.array(np.zeros((worlds, NMESHDEGMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NMESHDEGMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NMESHDEGMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NPOLYGONMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
            wp.array(np.zeros((worlds, NPOLYGONMAX, 3), dtype=np.float32), dtype=wp.vec3, device=device),
        ],
        outputs=[n_out, idx_out, dist_out, pos_out, frame_out, overflow],
        device=device,
    )

    n = n_out.numpy()
    idx = idx_out.numpy()
    dist = dist_out.numpy()
    pos = pos_out.numpy()
    frame = frame_out.numpy().reshape(worlds, 4, 9)

    reference = json.loads(REFERENCE.read_text(encoding="utf-8"))["cases"][CASE]
    fields = reference["fields"]
    ref_world = np.asarray(fields["worldid"], dtype=np.int32)
    ref_id = np.asarray(fields["geomcollisionid"], dtype=np.int32)
    ref_dist = np.asarray(fields["dist"], dtype=np.float32)
    ref_pos = np.asarray(fields["pos"], dtype=np.float32).reshape(-1, 3)
    ref_frame = np.asarray(fields["frame"], dtype=np.float32).reshape(-1, 9)

    rows = []
    for world in range(worlds):
        for i in range(int(n[world])):
            rows.append({
                "worldid": world,
                "geomcollisionid": i,
                "dist": float(dist[world, i]),
                "pos": pos[world, i].tolist(),
                "frame": frame[world, i].tolist(),
            })
    matched, used = [], set()
    for row in rows:
        for index in range(len(ref_dist)):
            if index in used or int(ref_world[index]) != row["worldid"] or int(ref_id[index]) != row["geomcollisionid"]:
                continue
            matched.append({
                "dist_ulp": ulp_distance(np.float32(row["dist"]), ref_dist[index]),
                "pos_abs": float(np.abs(np.asarray(row["pos"], dtype=np.float32) - ref_pos[index]).max()),
                "frame_abs": float(np.abs(np.asarray(row["frame"], dtype=np.float32) - ref_frame[index]).max()),
            })
            used.add(index)
            break
    result = {
        "case": CASE,
        "device": "warp CPU (authoring recalc)",
        "multiccd_idx": idx.tolist(),
        "recalc_n": n.tolist(),
        "reference_per_world": [int((ref_world == w).sum()) for w in range(worlds)],
        "recalc_geomcollisionids": [r["geomcollisionid"] for r in rows],
        "reference_geomcollisionids": ref_id.tolist(),
        "max_dist_ulp": max((m["dist_ulp"] for m in matched), default=None),
        "max_pos_abs": max((m["pos_abs"] for m in matched), default=None),
        "max_frame_abs": max((m["frame_abs"] for m in matched), default=None),
        "matched_rows": len(matched),
        "reference_rows": int(len(ref_dist)),
        "matched_detail": matched,
    }
    out = Path(__file__).with_name("cpu_multicontact_recalc.json")
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: result[k] for k in [
        "case", "multiccd_idx", "recalc_n", "reference_per_world",
        "recalc_geomcollisionids", "reference_geomcollisionids",
        "max_dist_ulp", "max_pos_abs", "max_frame_abs", "matched_rows", "reference_rows",
    ]}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
