"""G10@P5 multicontact 片：只读准备阶段的 CPU 复算脚本（authoring/diagnostic，非产品测试）。

目的（Arch 2026-10-25 建议）：
1. 用上游 `collision_primitive_core.box_box` 在 **warp CPU** 上复算 `box-box-margin` case，
   验证设备 `box_box_wrapper` 移植计划里的 8 槽候选、`detected = dist < margin + gap`
   筛选与 `id_ = i` 顺序是否能精确复现 QA 参考的 10 条接触（5 条/world）。
2. 不创建 CUDA 上下文、不写产品代码；输入用 G5 冻结 case 的 bin（与 QA 参考同源）。

运行：target/mjwarp-parallel/venv/Scripts/python.exe <本文件>
输出：同目录 cpu_boxbox_recalc.json（含逐行对照与匹配统计）
"""

import json
from pathlib import Path

import numpy as np
import warp as wp

wp.init()
# 上游函数来自离线 venv 的 mujoco_warp 3.15.0（与 QA 参考 provenance 一致）。
from mujoco_warp._src.collision_primitive_core import box_box
from mujoco_warp._src.math import make_frame

ROOT = Path(__file__).resolve().parents[4]
CASES = ROOT / "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/box-box-margin"
REFERENCE = ROOT / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json"
CASE = "box-box-margin"


@wp.kernel
def recalc_kernel(
    pos: wp.array2d(dtype=wp.vec3),
    rot: wp.array2d(dtype=wp.mat33),
    size: wp.array(dtype=wp.vec3),
    margin: float,
    out_dist: wp.array2d(dtype=float),
    out_pos: wp.array2d(dtype=wp.vec3),
    out_frame: wp.array2d(dtype=wp.mat33),
):
    world = wp.tid()
    dist, pos_out, normal = box_box(pos[world, 0], rot[world, 0], size[0], pos[world, 1], rot[world, 1], size[1], margin)
    for i in range(8):
        out_dist[world, i] = dist[i]
        out_pos[world, i] = pos_out[i]
        out_frame[world, i] = make_frame(normal[i])


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
    def ordered(v: np.float32) -> int:
        bits = np.asarray(v, dtype=np.float32).view(np.int32).item()
        if bits < 0:
            return -(bits & 0x7FFFFFFF)
        return bits

    return abs(ordered(a) - ordered(b))


def main() -> int:
    meta = read_meta(CASES)
    worlds = int(meta["nworld"])
    ngeom = int(meta["ngeom"])
    g1 = int(meta["g1"])
    g2 = int(meta["g2"])
    margin = float(meta["margin"])
    gap = float(meta["gap"])
    assert int(meta["type1"]) == 6 and int(meta["type2"]) == 6, "本复算只针对 box-box"

    xpos = read_bin(CASES, "geom_xpos", np.float32).reshape(worlds, ngeom, 3)
    xmat = read_bin(CASES, "geom_xmat", np.float32).reshape(worlds, ngeom, 9)
    size = read_bin(CASES, "geom_size", np.float32).reshape(ngeom, 3)

    # 只取 pair (g1, g2)：第二个 geom 在 row 1（wrapper 以 (box1, box2) 顺序调用）。
    pos_pair = xpos[:, [g1, g2], :]
    rot_pair = xmat[:, [g1, g2], :].reshape(worlds, 2, 3, 3)
    size_pair = size[[g1, g2], :]

    out_dist = np.zeros((worlds, 8), dtype=np.float32)
    out_pos = np.zeros((worlds, 8, 3), dtype=np.float32)
    out_frame = np.zeros((worlds, 8, 3, 3), dtype=np.float32)
    dist_wp = wp.array(out_dist, dtype=float, device="cpu")
    pos_wp = wp.array(out_pos, dtype=wp.vec3, device="cpu")
    frame_wp = wp.array(out_frame, dtype=wp.mat33, device="cpu")
    wp.launch(
        recalc_kernel,
        dim=worlds,
        inputs=[
            wp.array(pos_pair, dtype=wp.vec3, device="cpu"),
            wp.array(rot_pair, dtype=wp.mat33, device="cpu"),
            wp.array(size_pair, dtype=wp.vec3, device="cpu"),
            margin,
        ],
        outputs=[dist_wp, pos_wp, frame_wp],
        device="cpu",
    )
    out_dist = dist_wp.numpy()
    out_pos = pos_wp.numpy()
    out_frame = frame_wp.numpy()

    reference = json.loads(REFERENCE.read_text(encoding="utf-8"))["cases"][CASE]
    fields = reference["fields"]
    ref_world = np.asarray(fields["worldid"], dtype=np.int32)
    ref_geomcid = np.asarray(fields["geomcollisionid"], dtype=np.int32)
    ref_dist = np.asarray(fields["dist"], dtype=np.float32)
    ref_pos = np.asarray(fields["pos"], dtype=np.float32).reshape(-1, 3)
    ref_frame = np.asarray(fields["frame"], dtype=np.float32).reshape(-1, 9)

    # detected 语义：dist < margin + gap（wrapper 里 write_contact 的过滤）。
    rows = []
    for world in range(worlds):
        for i in range(8):
            distance = float(out_dist[world, i])
            if not (distance < margin + gap):
                continue
            rows.append(
                {
                    "worldid": world,
                    "geomcollisionid": i,
                    "dist": distance,
                    "pos": out_pos[world, i].astype(np.float32).tolist(),
                    "frame": out_frame[world, i].reshape(-1).astype(np.float32).tolist(),
                }
            )

    matched = []
    used = set()
    for row in rows:
        best = None
        for index in range(len(ref_dist)):
            if index in used:
                continue
            if int(ref_world[index]) != row["worldid"]:
                continue
            if int(ref_geomcid[index]) != row["geomcollisionid"]:
                continue
            d_ulp = ulp_distance(np.float32(row["dist"]), ref_dist[index])
            p_diff = np.abs(np.asarray(row["pos"], dtype=np.float32) - ref_pos[index]).max()
            f_diff = np.abs(np.asarray(row["frame"], dtype=np.float32) - ref_frame[index]).max()
            best = {
                "ref_index": index,
                "dist_ulp": int(d_ulp),
                "pos_abs": float(p_diff),
                "frame_abs": float(f_diff),
            }
            used.add(index)
            break
        matched.append({"recalc": row, "reference_row": best})

    unmatched_reference = [
        int(i) for i in range(len(ref_dist)) if i not in used
    ]
    result = {
        "case": CASE,
        "device": "warp CPU (authoring recalc)",
        "margin": margin,
        "gap": gap,
        "recalc_contact_count": len(rows),
        "reference_contact_count": int(len(ref_dist)),
        "recalc_per_world": [
            sum(1 for row in rows if row["worldid"] == world) for world in range(worlds)
        ],
        "reference_per_world": [
            int((ref_world == world).sum()) for world in range(worlds)
        ],
        "recalc_geomcollisionids": [row["geomcollisionid"] for row in rows],
        "reference_geomcollisionids": ref_geomcid.tolist(),
        "max_dist_ulp": max((m["reference_row"]["dist_ulp"] for m in matched if m["reference_row"]), default=None),
        "max_pos_abs": max((m["reference_row"]["pos_abs"] for m in matched if m["reference_row"]), default=None),
        "max_frame_abs": max((m["reference_row"]["frame_abs"] for m in matched if m["reference_row"]), default=None),
        "matched_rows": matched,
        "unmatched_reference_rows": unmatched_reference,
    }
    out = Path(__file__).with_name("cpu_boxbox_recalc.json")
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: result[k] for k in [
        "case", "recalc_contact_count", "reference_contact_count",
        "recalc_per_world", "reference_per_world",
        "recalc_geomcollisionids", "reference_geomcollisionids",
        "max_dist_ulp", "max_pos_abs", "max_frame_abs", "unmatched_reference_rows",
    ]}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
