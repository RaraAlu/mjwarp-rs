"""R06 多帧诊断：A/B（refit 开关）对照分析（CPU-only，零 GPU）。

输入（诊断采集产物，不写 fixtures）：
  target/r06-multiframe-diag/norefit/frame-{000,001,002}/{rgb,depth,seg,state}.npz + capture.json + cam-poses.json
  target/r06-multiframe-diag/refit/  （同结构）
参考：fixtures/render-bvh/scene-flex/reference/base/{rgb,depth,seg,state}.npz（只读）

输出：target/r06-multiframe-diag/analysis.json + 控制台摘要（另存 analysis-summary.txt）
分析项：
  A. 每组 frame-000 与 base：四类数组逐元素位级对照。
  B. 每组 flexvert_xpos 相对 frame-000 的位移：逐顶点、附着顶点均值/期望（k*0.100 m）、world 顶点。
  C. 每组逐相机渲染消费：相邻帧 depth 位级/seg 对变化计数（cam2 与窗口 697/616、244/183 对照）、
     cam2 flex 像素数（seg 第二分量==9，base≈654）、flex 掩码交并、|Δdepth|、flex 顶点投影 bbox 对照。
  D. A/B 交叉：逐帧 state 六项是否逐位相同；capture.json refit 标记；计数表。
"""

from __future__ import annotations

import json
import math
import pathlib

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/r06-multiframe-diag"
BASE = REPO / "fixtures/render-bvh/scene-flex/reference/base"
SCENE = REPO / "fixtures/render-bvh/scene-flex/scene.xml"
GROUPS = {"A_norefit": WORK / "norefit", "B_refit": WORK / "refit"}
FRAMES = 3
FRAME_DZ = 0.10
FLEX_SEG_TYPE = 9
CAM2 = 2
CAM2_FOVY_DEG = 45.0
CAM2_RES = (320, 240)


def load_npz(path: pathlib.Path):
    import numpy as np

    with np.load(path) as archive:
        return {name: archive[name] for name in archive.files}


def bits(array):
    import numpy as np

    return np.ascontiguousarray(np.asarray(array)).view(np.uint32)


def ulp_stats(a, b) -> dict:
    import numpy as np

    ia = bits(a).astype(np.int64)
    ib = bits(b).astype(np.int64)
    if ia.shape != ib.shape:
        return {"shape_mismatch": [list(ia.shape), list(ib.shape)]}
    diff = int(ia.size - int(np.count_nonzero(ia == ib)))
    out = {"size": int(ia.size), "bit_differ": diff}
    if diff:
        distance = np.abs(ia - ib)
        out["max_ulp"] = int(distance.max())
    else:
        out["max_ulp"] = 0
    return out


def model_map() -> dict:
    import mujoco

    mjm = mujoco.MjModel.from_xml_path(str(SCENE))
    flexes = []
    for flex in range(int(mjm.nflex)):
        adr, num = int(mjm.flex_vertadr[flex]), int(mjm.flex_vertnum[flex])
        flexes.append({
            "name": mujoco.mj_id2name(mjm, mujoco.mjtObj.mjOBJ_FLEX, flex) or f"flex{flex}",
            "vertadr": adr,
            "vertnum": num,
            "vertbodyid": [int(v) for v in mjm.flex_vertbodyid[adr : adr + num]],
        })
    return {"nflex": int(mjm.nflex), "nflexvert": int(mjm.nflexvert), "flexes": flexes}


def project_to_cam2(points_cam2: list[list[float]]) -> list[list[float]]:
    """世界点集已由调用方转到 cam2 局部坐标；按 fovy=45、320x240 投影到像素。"""
    width, height = CAM2_RES
    aspect = width / height
    tan_half = math.tan(math.radians(CAM2_FOVY_DEG) / 2.0)
    pixels = []
    for x, y, z in points_cam2:
        if z >= -1e-9:
            continue
        u_ndc = (x / -z) / (tan_half * aspect)
        v_ndc = (y / -z) / tan_half
        pixels.append([(u_ndc + 1.0) * 0.5 * (width - 1), (1.0 - v_ndc) * 0.5 * (height - 1)])
    return pixels


def world_to_cam2(points_world, cam_pos, cam_xmat) -> list[list[float]]:
    import numpy as np

    rot = np.asarray(cam_xmat, dtype=np.float64).reshape(3, 3)  # cam_xmat 行主序（世界→相机转置）
    pos = np.asarray(cam_pos, dtype=np.float64)
    return [((np.asarray(p, dtype=np.float64) - pos) @ rot).tolist() for p in points_world]


def analyze_group(name: str, root: pathlib.Path, mm: dict) -> dict:
    import numpy as np

    capture = json.loads((root / "capture.json").read_text(encoding="utf-8"))
    cam_poses = json.loads((root / "cam-poses.json").read_text(encoding="utf-8"))
    states = [load_npz(root / f"frame-{k:03d}/state.npz") for k in range(FRAMES)]
    depth = [load_npz(root / f"frame-{k:03d}/depth.npz") for k in range(FRAMES)]
    seg = [load_npz(root / f"frame-{k:03d}/seg.npz") for k in range(FRAMES)]
    rgb = [load_npz(root / f"frame-{k:03d}/rgb.npz") for k in range(FRAMES)]

    # A. frame-000 vs base
    base_state = load_npz(BASE / "state.npz")
    state_cmp = {key: ulp_stats(states[0][key], base_state[key]) for key in sorted(states[0])}
    outputs_cmp = {}
    for filename in ("rgb.npz", "depth.npz", "seg.npz"):
        base_arrays = load_npz(BASE / filename)
        outputs_cmp[filename] = {
            key: ulp_stats(load_npz(root / "frame-000" / filename)[key], base_arrays[key])
            if key in base_arrays else {"missing_in_base": True}
            for key in sorted(load_npz(root / "frame-000" / filename))
        }
    all_equal = all(v.get("bit_differ") == 0 for v in state_cmp.values()) and all(
        v.get("bit_differ") == 0 for group in outputs_cmp.values() for v in group.values()
    )

    # B. flexvert displacement vs frame-000
    positions = [np.asarray(s["flexvert_xpos"], dtype=np.float64).reshape(-1, 3) for s in states]
    base_pos = positions[0]
    bodyids: list[int] = []
    flex_of_vertex: list[str] = []
    for flex in mm["flexes"]:
        bodyids.extend(flex["vertbodyid"])
        flex_of_vertex.extend([flex["name"]] * flex["vertnum"])
    displacement = {}
    for k in range(FRAMES):
        delta = positions[k] - base_pos
        mags = np.linalg.norm(delta, axis=1)
        attached = [i for i, body in enumerate(bodyids) if body != 0]
        displacement[f"frame-{k:03d}"] = {
            "nominal_offset_m": round(k * FRAME_DZ, 6),
            "per_vertex": [
                {
                    "vertex": i,
                    "flex": flex_of_vertex[i],
                    "bodyid": bodyids[i],
                    "dz": float(delta[i, 2]),
                    "magnitude": float(mags[i]),
                    "expected_magnitude": round(k * FRAME_DZ, 6) if bodyids[i] != 0 else 0.0,
                }
                for i in range(len(bodyids))
            ],
            "attached_mean_m": float(mags[attached].mean()) if attached else 0.0,
            "attached_expected_m": round(k * FRAME_DZ, 6),
            "all_vertices_mean_m": float(mags.mean()),
            "world_attached": [
                {"vertex": i, "magnitude": float(mags[i])} for i in range(len(bodyids)) if bodyids[i] == 0
            ],
        }

    # C. render consumption
    cam2_seg = seg
    per_camera = {}
    ncam = sum(1 for key in depth[0] if key.startswith("depth_cam"))
    for cam in range(ncam):
        key_d, key_s = f"depth_cam{cam}", f"seg_cam{cam}"
        masks = [np.asarray(s[key_s])[..., 1] == FLEX_SEG_TYPE for s in seg]
        cam_entry = {
            "resolution": list(np.asarray(depth[0][key_d]).shape),
            "flex_pixels_by_frame": [int(m.sum()) for m in masks],
            "pairs": {},
        }
        for pair in range(FRAMES - 1):
            left, right = pair, pair + 1
            d_left = np.asarray(depth[left][key_d], dtype=np.float32)
            d_right = np.asarray(depth[right][key_d], dtype=np.float32)
            s_left = np.asarray(seg[left][key_s])
            s_right = np.asarray(seg[right][key_s])
            depth_changed_mask = bits(d_left) != bits(d_right)
            seg_changed_mask = np.any(s_left != s_right, axis=2)
            delta_depth = np.abs(d_right.astype(np.float64) - d_left.astype(np.float64))
            changed = delta_depth[depth_changed_mask]
            cam_entry["pairs"][f"{left:03d}->{right:03d}"] = {
                "depth_changed": int(depth_changed_mask.sum()),
                "seg_changed": int(seg_changed_mask.sum()),
                "flex_pixels_left": int(masks[left].sum()),
                "flex_pixels_right": int(masks[right].sum()),
                "flex_mask_jaccard": float(
                    (masks[left] & masks[right]).sum() / max(1, (masks[left] | masks[right]).sum())
                ),
                "mean_abs_delta_depth_on_changed_px": float(changed.mean()) if changed.size else None,
            }
        per_camera[f"cam{cam}"] = cam_entry

    # cam2 flex 顶点投影 bbox 对照（用 cam2 位姿；验证渲染覆盖是否随注入移动）
    poses = cam_poses["poses"]
    pos_bits = poses["cam_xpos_bits"]
    mat_bits = poses["cam_xmat_bits"]
    import struct

    def f32(v: int) -> float:
        return struct.unpack("<f", struct.pack("<I", v & 0xFFFFFFFF))[0]

    cam_pos = [f32(b) for b in pos_bits[CAM2 * 3 : CAM2 * 3 + 3]]
    cam_mat = [f32(b) for b in mat_bits[CAM2 * 9 : CAM2 * 9 + 9]]
    projection = {}
    for k in range(FRAMES):
        local = world_to_cam2(positions[k], cam_pos, cam_mat)
        pixels = project_to_cam2(local)
        if pixels:
            us = [p[0] for p in pixels]
            vs = [p[1] for p in pixels]
            projection[f"frame-{k:03d}"] = {
                "projected_bbox_uv": [min(us), max(us), min(vs), max(vs)],
                "projected_centroid_uv": [sum(us) / len(us), sum(vs) / len(vs)],
                "projected_pixels_count": len(pixels),
            }

    return {
        "capture_refit_bvh": capture.get("refit_bvh"),
        "capture_delta_selfcheck": capture.get("delta_selfcheck"),
        "capture_two_identical": capture.get("two_captures_bitwise_identical"),
        "A_frame000_vs_base": {"state": state_cmp, "outputs": outputs_cmp, "all_bit_equal": all_equal},
        "B_flexvert_displacement": displacement,
        "C_render_consumption": {
            "delta_thresholds": capture.get("delta_constants"),
            "per_camera": per_camera,
            "cam2_flex_vertex_projection": projection,
        },
        "misc": {
            "rgb_shapes": {key: list(np.asarray(rgb[0][key]).shape) for key in sorted(rgb[0])},
        },
    }


def cross_group(report: dict) -> None:
    import numpy as np

    a, b = report["groups"]["A_norefit"], report["groups"]["B_refit"]
    state_equal = {}
    for k in range(FRAMES):
        sa = load_npz(WORK / "norefit" / f"frame-{k:03d}/state.npz")
        sb = load_npz(WORK / "refit" / f"frame-{k:03d}/state.npz")
        state_equal[f"frame-{k:03d}"] = {
            key: ulp_stats(sa[key], sb[key]) for key in sorted(sa)
        }
    base_seg = load_npz(BASE / "seg.npz")[f"seg_cam{CAM2}"]
    base_flex_pixels = int((np.asarray(base_seg)[..., 1] == FLEX_SEG_TYPE).sum())
    table = {}
    for pair in ("000->001", "001->002"):
        table[pair] = {
            "A_norefit": a["C_render_consumption"]["per_camera"][f"cam{CAM2}"]["pairs"][pair],
            "B_refit": b["C_render_consumption"]["per_camera"][f"cam{CAM2}"]["pairs"][pair],
        }
    report["D_cross_group"] = {
        "state_bitwise_identical_between_groups": state_equal,
        "state_all_identical": all(
            v.get("bit_differ") == 0 for group in state_equal.values() for v in group.values()
        ),
        "base_cam2_flex_pixels": base_flex_pixels,
        "cam2_counts_table": table,
        "refit_flag": {"A_norefit": a["capture_refit_bvh"], "B_refit": b["capture_refit_bvh"]},
    }


def main() -> int:
    report: dict = {"task_id": "2026-11-05-render-r06-flex-multiframe-pose-diag"}
    mm = model_map()
    report["model"] = mm
    report["groups"] = {name: analyze_group(name, root, mm) for name, root in GROUPS.items()}
    cross_group(report)

    lines = ["# R06 多帧诊断 A/B（refit 开关）摘要", ""]
    lines.append("## frame-000 vs base（数组级位级）")
    for name in GROUPS:
        g = report["groups"][name]
        lines.append(f"- {name}: all_bit_equal={g['A_frame000_vs_base']['all_bit_equal']}")
    lines.append("")
    lines.append("## flexvert_xpos 相对 frame-000 位移")
    for name in GROUPS:
        for frame, entry in report["groups"][name]["B_flexvert_displacement"].items():
            lines.append(
                f"- {name} {frame}: attached_mean={entry['attached_mean_m']:.6f} m "
                f"(expected {entry['attached_expected_m']:.6f}), all_vertices_mean="
                f"{entry['all_vertices_mean_m']:.6f}, world_vertex={entry['world_attached']}"
            )
    lines.append("")
    lines.append("## cam2 flex 像素数（seg 第二分量==9；base 参考）")
    lines.append(f"- base: {report['D_cross_group']['base_cam2_flex_pixels']}")
    for name in GROUPS:
        cam2 = report["groups"][name]["C_render_consumption"]["per_camera"][f"cam{CAM2}"]
        lines.append(f"- {name}: by_frame={cam2['flex_pixels_by_frame']}")
    lines.append("")
    lines.append("## cam2 相邻帧变化计数（thresholds accept=384 / margin=576）")
    for pair, entry in report["D_cross_group"]["cam2_counts_table"].items():
        a, b = entry["A_norefit"], entry["B_refit"]
        lines.append(
            f"- {pair}: A(norefit) depth={a['depth_changed']} seg={a['seg_changed']} | "
            f"B(refit) depth={b['depth_changed']} seg={b['seg_changed']}"
        )
    lines.append("")
    lines.append("## A/B state 是否逐位相同")
    lines.append(f"- state_all_identical={report['D_cross_group']['state_all_identical']}")
    lines.append(f"- capture refit 标记: A={report['D_cross_group']['refit_flag']['A_norefit']} "
                 f"B={report['D_cross_group']['refit_flag']['B_refit']}")
    lines.append("")
    lines.append("## cam2 flex 顶点投影 bbox（uv；u=列, v=行）")
    for name in GROUPS:
        proj = report["groups"][name]["C_render_consumption"]["cam2_flex_vertex_projection"]
        for frame, entry in proj.items():
            lines.append(f"- {name} {frame}: bbox={[round(v, 1) for v in entry['projected_bbox_uv']]} "
                         f"centroid={[round(v, 1) for v in entry['projected_centroid_uv']]}")
    text = "\n".join(lines) + "\n"
    (WORK / "analysis-summary.txt").write_text(text, encoding="utf-8", newline="\n")
    (WORK / "analysis.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n",
                                        encoding="utf-8", newline="\n")
    print(text)
    print("wrote", WORK / "analysis.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
