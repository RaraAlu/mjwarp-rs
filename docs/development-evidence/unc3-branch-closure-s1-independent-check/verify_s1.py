"""UNC-3 S1 独立复核：pinned/设备公式复算（CPU，不建 CUDA context）。

覆盖：
1. plane_box 8 角点：B1 现有 case（reference qpos）实际进 cutoff 的角点集合；
2. G09 plane-box 三个世界（flat4/edge2/corner1）的角点集合（对照「现有 case 只出现 3 个」口径）；
3. U3 设计 4 pose 的角点覆盖（face-down / tilt-x45 / tilt-y45 / single-corner）→ 并集是否 8；
   以及 180° 翻转 pose 的最小修补；
4. U5 plane_cylinder 倾斜：4 条候选进 cutoff 所需的穿透深度；
5. U2 plane_ellipsoid 防御分支：f32 平方溢出阈值（size 多大时 length=inf → guard 触发）；
6. U1 plane_cylinder 退化：vec/len_sqr 是否精确为 0（f32 逐步）。

公式来源（pinned rev 71da24d9）：
- plane_box：collision_primitive_core.py L366-390；wrapper cutoff = collision_core.py L255-256
  (detected = dist < margin+gap；margin=gap=0 → dist<0)。
- plane_cylinder：collision_primitive_core.py L496-557。
- plane_ellipsoid：collision_primitive_core.py L312-339（无 guard）；设备 guard 在
  primitive_plane_shapes.cu L100-102。
- write_contact 详情：collision_core.write_contact L214-256。

注意：本脚本只复算数学，不声称设备/上游执行结果。
"""
import json
import math
import struct
import sys
from pathlib import Path

import numpy as np

REPO = Path(r"D:\Rust\mjwarp-rs")
OUT = Path(__file__).with_name("verify_s1.json")

F32 = np.float32
FLT_MAX = np.finfo(np.float32).max


def f32(x):
    return float(F32(x))


def quat_to_R(w, x, y, z):
    """单位 quat (w,x,y,z) → 3x3 行优先旋转矩阵（double 复算，输入按 f64 原值）。"""
    n = math.sqrt(w * w + x * x + y * y + z * z)
    w, x, y, z = w / n, x / n, y / n, z / n
    return np.array(
        [
            [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
            [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
            [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
        ],
        dtype=np.float64,
    )


def rot_x(deg):
    t = math.radians(deg)
    c, s = math.cos(t), math.sin(t)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]], dtype=np.float64)


def rot_y(deg):
    t = math.radians(deg)
    c, s = math.cos(t), math.sin(t)
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]], dtype=np.float64)


def rot_z(deg):
    t = math.radians(deg)
    c, s = math.cos(t), math.sin(t)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]], dtype=np.float64)


def corners(size):
    """pinned/设备同序：i&1→x 正，i&2→y 正，i&4→z 正。"""
    out = []
    for i in range(8):
        out.append(
            np.array(
                [
                    size[0] if (i & 1) else -size[0],
                    size[1] if (i & 2) else -size[1],
                    size[2] if (i & 4) else -size[2],
                ],
                dtype=np.float64,
            )
        )
    return out


def plane_box_contacts(R, center, size, margin=0.0, gap=0.0, normal=(0.0, 0.0, 1.0)):
    """返回 [(corner_index, dist)]；detected = dist < margin+gap（pinned write_contact L255-256）。"""
    n = np.array(normal, dtype=np.float64)
    center = np.array(center, dtype=np.float64)
    center_dist = float(np.dot(center, n))  # plane_pos = 0
    rows = []
    for i, corner in enumerate(corners(size)):
        rotated = R @ corner
        dist = center_dist + float(np.dot(n, rotated))
        rows.append((i, dist, dist < margin + gap))
    return rows


def plane_cylinder_candidates(R_axis, center_z, radius, half_height, normal=(0.0, 0.0, 1.0)):
    """pinned plane_cylinder L496-557 的 4 条候选 dist（plane_pos=0, 轴 = R@(0,0,1)）。"""
    n = np.array(normal, dtype=np.float64)
    axis = R_axis @ np.array([0.0, 0.0, 1.0])
    prjaxis = float(np.dot(n, axis))
    if prjaxis > 0:
        axis = -axis
        prjaxis = -prjaxis
    dist0 = center_z  # cylinder_center=(0,0,center_z), plane_pos=0
    vec = axis * prjaxis - n
    len_sqr = float(np.dot(vec, vec))
    if len_sqr >= 1e-12:
        vec = vec * (radius / math.sqrt(len_sqr))
    else:
        vec = np.array([radius, 0.0, 0.0])
    prjvec = float(np.dot(vec, n))
    axis = axis * half_height
    prjaxis = prjaxis * half_height
    dist1 = dist0 + prjaxis + prjvec
    dist2 = dist0 - prjaxis + prjvec
    prjvec1 = -prjvec * 0.5
    dist3 = dist0 + prjaxis + prjvec1
    return {
        "axis": [float(v) for v in axis],
        "prjaxis": prjaxis,
        "len_sqr": len_sqr,
        "vec": [float(v) for v in vec],
        "dists": [float(dist1), float(dist2), float(dist3), float(dist3)],
        "in_cutoff": [bool(d < 0.0) for d in [dist1, dist2, dist3, dist3]],
    }


def plane_ellipsoid_f32_length(local_n, size):
    """设备 primitive_plane_shapes.cu L96-99 的 f32 计算：length = sqrt(dot(scaled,scaled))。"""
    scaled = np.array([F32(local_n[i] * size[i]) for i in range(3)], dtype=F32)
    acc = F32(0.0)
    for v in scaled:
        acc = F32(acc + F32(v * v))
    if acc > FLT_MAX:  # f32 乘法溢出已在上面发生；这里显式检查 dot 是否 inf
        pass
    length = F32(np.sqrt(acc))
    return float(acc), float(length)


def plane_cylinder_degenerate_f32(radius, half_height, center_z):
    """U1 构造的 f32 逐步：axis=(0,0,1) → 翻转 → vec = axis*prjaxis - n。"""
    axis = np.array([F32(0.0), F32(0.0), F32(1.0)], dtype=F32)
    n = np.array([F32(0.0), F32(0.0), F32(1.0)], dtype=F32)
    prjaxis = F32(axis[0] * n[0] + axis[1] * n[1] + axis[2] * n[2])
    if prjaxis > 0:
        axis = np.array([F32(-axis[0]), F32(-axis[1]), F32(-axis[2])], dtype=F32)
        prjaxis = F32(-prjaxis)
    vec = np.array(
        [F32(F32(axis[i] * prjaxis) - n[i]) for i in range(3)], dtype=F32
    )
    len_sqr = F32(vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2])
    fallback = bool(len_sqr < F32(1e-12))
    vec = np.array([F32(radius), F32(0.0), F32(0.0)], dtype=F32) if fallback else vec
    prjvec = F32(vec[0] * n[0] + vec[1] * n[1] + vec[2] * n[2])
    axis_h = np.array([F32(axis[i] * F32(half_height)) for i in range(3)], dtype=F32)
    prjaxis_h = F32(prjaxis * F32(half_height))
    dist1 = F32(F32(center_z) + prjaxis_h + prjvec)
    dist2 = F32(F32(center_z) - prjaxis_h + prjvec)
    return {
        "prjaxis": float(prjaxis),
        "vec": [float(v) for v in vec],
        "len_sqr": float(len_sqr),
        "took_fallback": fallback,
        "dist1": float(dist1),
        "dist2": float(dist2),
    }


def main():
    result = {}

    # ---- 1. B1 现有 plane-box case（reference 的真实 qpos，1 world） ----
    ref = json.loads(
        (REPO / "docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json").read_text(
            encoding="utf-8"
        )
    )
    box_case = ref["cases"]["plane-box"]
    qpos = box_case["qpos"]  # x,y,z,qw,qx,qy,qz
    count_ref = box_case["output"]["count"]
    R = quat_to_R(qpos[3], qpos[4], qpos[5], qpos[6])
    rows = plane_box_contacts(R, (0.0, 0.0, qpos[2]), (0.1, 0.08, 0.07))
    contacts = [{"corner": i, "dist": d} for (i, d, hit) in rows if hit]
    result["b1_plane_box"] = {
        "qpos": qpos,
        "reference_count": count_ref,
        "corners": [{"corner": i, "dist": d, "detected": hit} for (i, d, hit) in rows],
        "contact_set": sorted(c["corner"] for c in contacts),
        "contact_count": len(contacts),
        "match_reference_count": len(contacts) == count_ref,
    }

    # ---- 2. G09 plane-box 三世界（box_a 0.5/0.4/0.3；qpos 来自 cases.json） ----
    g09 = json.loads((REPO / "fixtures/plane-box/cases.json").read_text(encoding="utf-8"))
    g09_rows = {}
    for w in g09["worlds"]:
        if w["name"] not in ("flat4", "edge2", "corner1"):
            continue
        q = w["qpos"][:7]
        Rg = quat_to_R(q[3], q[4], q[5], q[6])
        rows_g = plane_box_contacts(Rg, (q[0], q[1], q[2]), (0.5, 0.4, 0.3))
        cs = sorted(i for (i, d, hit) in rows_g if hit)
        g09_rows[w["name"]] = {
            "center": [q[0], q[1], q[2]],
            "contact_set": cs,
            "count": len(cs),
            "dists": {i: round(d, 9) for (i, d, hit) in rows_g},
        }
    union = sorted({i for v in g09_rows.values() for i in v["contact_set"]})
    result["g09_plane_box"] = {"worlds": g09_rows, "union_bottom_face": union}

    # ---- 3. U3 设计 4 pose（box 0.1/0.08/0.07，δ=0.01 穿透） ----
    size = (0.1, 0.08, 0.07)
    delta = 0.01
    poses = {}

    def pose_contacts(name, Rm):
        # 旋转后最低角点放到 -δ
        zs = [float((Rm @ c)[2]) for c in corners(size)]
        z0 = -min(zs) - delta
        rows_p = plane_box_contacts(Rm, (0.0, 0.0, z0), size)
        cs = sorted(i for (i, d, hit) in rows_p if hit)
        poses[name] = {
            "center_z": z0,
            "contact_set": cs,
            "count": len(cs),
            "dists": {i: round(d, 9) for (i, d, hit) in rows_p},
        }
        return cs

    pose_contacts("A_face_down", np.eye(3))
    pose_contacts("B_tilt_x45", rot_x(45))
    pose_contacts("C_tilt_y45", rot_y(45))
    # D：单角朝下 —— 使 (+,+,+) 角成为最低角：第三行 = -(1,1,1)/√3
    d = np.array([1.0, 1.0, 1.0]) / math.sqrt(3.0)
    e1 = np.array([1.0, -1.0, 0.0]) / math.sqrt(2.0)
    e2 = np.cross(-d, e1)
    R_single = np.vstack([e1, e2, -d])
    pose_contacts("D_single_corner", R_single)
    union_4 = sorted({i for v in poses.values() for i in v["contact_set"]})

    # 最小修补：A 的 180° 翻转（+z 面朝下）→ 覆盖另一面 4 角
    pose_contacts("A2_face_down_flipped", rot_x(180))
    union_5 = sorted({i for v in poses.values() for i in v["contact_set"]})
    result["u3_pose_design"] = {
        "box_half_extents": size,
        "penetration_delta": delta,
        "poses": poses,
        "union_four_poses": union_4,
        "covered_by_four_poses": len(union_4),
        "union_plus_flipped_pose": union_5,
        "covered_plus_flip": len(union_5),
        "all_eight_covered_after_flip": union_5 == list(range(8)),
    }

    # ---- 4. U5 plane_cylinder 倾斜：4 候选进 cutoff 的所需中心高度 ----
    radius, half_height = 0.06, 0.1
    u5 = {}
    for tilt in (10.0, 20.0, 30.0, 45.0):
        probe = plane_cylinder_candidates(rot_x(tilt), 0.0, radius, half_height)
        # 4 条候选距离随中心 z 线性平移；求全部 <0 的上界
        max_z_all4 = -max(probe["dists"]) - 1e-12
        one = plane_cylinder_candidates(rot_x(tilt), -0.01 + max(probe["dists"]), radius, half_height)
        u5[f"tilt_{int(tilt)}"] = {
            "len_sqr": probe["len_sqr"],
            "dists_at_center0": [round(v, 9) for v in probe["dists"]],
            "center_z_upper_bound_all4_in_cutoff": round(max_z_all4, 9),
            "lower_end_offset": round(max(probe["dists"]), 9),
        }
    result["u5_cylinder_tilt"] = {
        "radius": radius,
        "half_height": half_height,
        "tilt_cases": u5,
        "note": "center_z 需 < upper_bound 才能 4 条候选全部 dist<0；该 bound 为负 = 深穿透",
    }

    # ---- 5. U2 ellipsoid 防御分支：f32 平方溢出阈值 ----
    overflow_threshold = math.sqrt(FLT_MAX)
    u2 = {}
    for size_z in (1.0, 1e10, 1.8e19, 1.844e19, 1.85e19, 1.9e19, 2e19, 1e20):
        acc, length = plane_ellipsoid_f32_length((0.0, 0.0, 1.0), (0.1, 0.1, size_z))
        u2[f"{size_z:.3g}"] = {
            "dot_f32": acc,
            "length_f32": length,
            "isfinite": bool(np.isfinite(length)),
            "guard_triggers": bool(not (length > 0.0) or not np.isfinite(length)),
        }
    result["u2_ellipsoid_guard"] = {
        "flt_max": float(FLT_MAX),
        "sqrt_flt_max_overflow_threshold": overflow_threshold,
        "cases": u2,
    }

    # ---- 6. U1 plane_cylinder 退化 f32 逐步 ----
    result["u1_cylinder_degenerate"] = plane_cylinder_degenerate_f32(radius, half_height, half_height - delta)

    OUT.write_text(json.dumps(result, ensure_ascii=False, indent=1), encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
