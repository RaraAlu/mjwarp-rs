"""QA：从 G5 冻结 case 输入独立复算 primitive 分支（不跑 GPU）。

性质：诊断/归因用影子实现（**不参与判据**），目的是回答「设备到底走了上游哪个分支」，
而不是只跑浅穿透。用 numpy.float32 逐式转写上游 `sphere_box`/`capsule_box`（表达式顺序
按上游），从 QA 自己的冻结输入 bin 复算：

1) `sphere-box-shallow` / `sphere-box-deep` 走 `sphere_box` 的「盒外」还是「盒内」分支；
2) `capsule-box` 的 `cltype/clface/secondpos` 走哪个分支（是否触发第二接触）；
3) 与上游参考 `fields.dist/pos/frame` 做交叉核对（本脚本只打印，判据由
   `qa_primitive_check.py` 对**设备真实 dump** 承担）。

输出：`target/qa-primitive/case-branch-analysis.json`（诊断中间物，不入库）。
"""
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
CASES = ROOT / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/cases"
REF = ROOT / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json"
MJ_MINVAL = 1e-15
MJ_MAXVAL = 1e10


def r(d, name, dt):
    return np.fromfile(d / f"{name}.bin", dtype=dt)


def f32(x):
    return np.float32(x)


def normalize_with_norm(x):
    n = f32(np.sqrt(f32(np.dot(x, x))))
    if n == f32(0.0):
        return x, f32(0.0)
    return f32(x / n), n


def safe_div(a, b):
    return f32(a / (b if b != 0 else f32(MJ_MINVAL)))


def matmul(m, v):
    """m: 9 floats row-major (MuJoCo xmat), v: vec3."""
    return np.array(
        [
            f32(m[0]) * v[0] + f32(m[1]) * v[1] + f32(m[2]) * v[2],
            f32(m[3]) * v[0] + f32(m[4]) * v[1] + f32(m[5]) * v[2],
            f32(m[6]) * v[0] + f32(m[7]) * v[1] + f32(m[8]) * v[2],
        ],
        dtype=np.float32,
    )


def transpose_mul(m, v):
    """m 行优先；返回 m^T @ v。"""
    return np.array(
        [
            f32(m[0]) * v[0] + f32(m[3]) * v[1] + f32(m[6]) * v[2],
            f32(m[1]) * v[0] + f32(m[4]) * v[1] + f32(m[7]) * v[2],
            f32(m[2]) * v[0] + f32(m[5]) * v[1] + f32(m[8]) * v[2],
        ],
        dtype=np.float32,
    )


def sphere_box(sphere_pos, sphere_radius, box_pos, box_rot, box_size):
    center = transpose_mul(box_rot, f32(np.subtract(sphere_pos, box_pos)))
    clamped = np.maximum(f32(-np.asarray(box_size)), np.minimum(np.asarray(box_size), center))
    clamped_dir, dist = normalize_with_norm(f32(np.subtract(clamped, center)))
    if dist <= f32(MJ_MINVAL):
        branch = "inside"
        closest = f32(2.0) * f32(box_size[0] + box_size[1] + box_size[2])
        k = 0
        for i in range(6):
            sgn = f32(1.0) if i % 2 else f32(-1.0)
            face_dist = f32(abs(f32(sgn * box_size[i // 2]) - center[i // 2]))
            if closest > face_dist:
                closest = face_dist
                k = i
        nearest = np.zeros(3, dtype=np.float32)
        nearest[k // 2] = f32(-1.0) if k % 2 else f32(1.0)
        scaled = f32(nearest * f32(sphere_radius - closest))
        pos = f32(center + scaled / f32(2.0))
        normal = matmul(box_rot, nearest)
        d = f32(-closest - sphere_radius)
    else:
        branch = "outside"
        deepest = f32(center + f32(clamped_dir * sphere_radius))
        pos = f32(f32(0.5) * f32(clamped + deepest))
        normal = matmul(box_rot, clamped_dir)
        d = f32(dist - sphere_radius)
    contact_position = f32(np.add(box_pos, matmul(box_rot, pos)))
    return d, contact_position, normal, branch


def capsule_box(capsule_pos, capsule_axis, capsule_radius, capsule_half_length,
                box_pos, box_rot, box_size):
    pos = transpose_mul(box_rot, f32(np.subtract(capsule_pos, box_pos)))
    axis = transpose_mul(box_rot, capsule_axis)
    halfaxis = f32(axis * capsule_half_length)
    axisdir = (1 if halfaxis[0] > 0 else 0) + 2 * (1 if halfaxis[1] > 0 else 0) + 4 * (
        1 if halfaxis[2] > 0 else 0)
    bestdist = f32(1.0e32)
    bestsegmentpos = f32(-12.0)
    cltype = -4
    clface = -12
    for i in (-1, 1):
        axis_tip = f32(pos + f32(i) * halfaxis)
        box_point = np.array(axis_tip, dtype=np.float32)
        n_out = 0
        ax_out = -1
        for j in range(3):
            if box_point[j] < -box_size[j]:
                n_out += 1
                ax_out = j
                box_point[j] = -box_size[j]
            elif box_point[j] > box_size[j]:
                n_out += 1
                ax_out = j
                box_point[j] = box_size[j]
        if n_out > 1:
            continue
        delta = f32(np.subtract(box_point, axis_tip))
        dist = f32(np.dot(delta, delta))
        if dist < bestdist:
            bestdist = dist
            bestsegmentpos = f32(i)
            cltype = -2 + i
            clface = ax_out
    clcorner = -123
    cledge = -123
    bestboxpos = f32(0.0)
    for i in range(8):
        for j in range(3):
            if i & (1 << j):
                continue
            box_pt = np.array(
                [
                    (box_size[0] if i & 1 else -box_size[0]),
                    (box_size[1] if i & 2 else -box_size[1]),
                    (box_size[2] if i & 4 else -box_size[2]),
                ],
                dtype=np.float32,
            )
            box_pt[j] = f32(0.0)
            dif = f32(np.subtract(box_pt, pos))
            u = f32(-f32(box_size[j]) * dif[j])
            v = f32(np.dot(halfaxis, dif))
            ma = f32(f32(box_size[j]) * f32(box_size[j]))
            mb = f32(-f32(box_size[j]) * halfaxis[j])
            mc = f32(capsule_half_length * capsule_half_length)
            det = f32(f32(ma * mc) - f32(mb * mb))
            if abs(det) < f32(MJ_MINVAL):
                continue
            idet = f32(f32(1.0) / det)
            x1 = f32(f32(f32(mc * u) - f32(mb * v)) * idet)
            x2 = f32(f32(f32(ma * v) - f32(mb * u)) * idet)
            s1 = 1
            s2 = 1
            if x1 > f32(1.0):
                x1 = f32(1.0)
                s1 = 2
                x2 = safe_div(f32(v - mb), mc)
            elif x1 < f32(-1.0):
                x1 = f32(-1.0)
                s1 = 0
                x2 = safe_div(f32(v + mb), mc)
            x2_over = x2 > f32(1.0)
            if x2_over or x2 < f32(-1.0):
                if x2_over:
                    x2 = f32(1.0)
                    s2 = 2
                    x1 = safe_div(f32(u - mb), ma)
                else:
                    x2 = f32(-1.0)
                    s2 = 0
                    x1 = safe_div(f32(u + mb), ma)
                if x1 > f32(1.0):
                    x1 = f32(1.0)
                    s1 = 2
                elif x1 < f32(-1.0):
                    x1 = f32(-1.0)
                    s1 = 0
            dif = f32(np.subtract(dif, f32(halfaxis * x2)))
            dif[j] = f32(dif[j] + f32(box_size[j] * x1))
            ct = s1 * 3 + s2
            dif_sq = f32(np.dot(dif, dif))
            if dif_sq < f32(bestdist - f32(MJ_MINVAL)):
                bestdist = dif_sq
                bestsegmentpos = x2
                bestboxpos = x1
                c2 = ct // 6
                clcorner = i + (1 << j) * c2
                cledge = j
                cltype = ct
    p = np.array([pos[0], pos[1]], dtype=np.float32)
    dd = np.array([halfaxis[0], halfaxis[1]], dtype=np.float32)
    s = np.array([box_size[0], box_size[1]], dtype=np.float32)
    secondpos = f32(-4.0)
    uu = f32(dd[0] * s[1])
    vv = f32(dd[1] * s[0])
    w_neg = f32(f32(dd[0] * p[1]) - f32(dd[1] * p[0])) < f32(0.0)
    best = f32(-1.0)
    ee1 = f32(uu - vv)
    ee2 = f32(uu + vv)
    c1 = None
    if abs(ee1) > best:
        best = f32(abs(ee1))
        c1 = 0 if ((ee1 < 0) == w_neg) else 3
    if abs(ee2) > best:
        best = f32(abs(ee2))
        c1 = 1 if ((ee2 > 0) == w_neg) else 2
    branch = None
    if cltype == -4:
        branch = "invalid"
        return [f32(MJ_MAXVAL), f32(MJ_MAXVAL)], None, None, {
            "cltype": cltype, "secondpos": float(secondpos), "branch": branch,
            "bestsegmentpos": float(bestsegmentpos)}
    if cltype >= 0 and cltype // 3 != 1:
        branch = "corner"
        c1 = axisdir ^ clcorner
        if c1 != 0 and c1 != 7:
            if c1 in (1, 2, 4):
                mul = 1
            else:
                mul = -1
                c1 = 7 - c1
            if c1 == 1:
                ax, ax1, ax2 = 0, 1, 2
            elif c1 == 2:
                ax, ax1, ax2 = 1, 2, 0
            elif c1 == 4:
                ax, ax1, ax2 = 2, 0, 1
            else:
                ax = ax1 = ax2 = 0
            if f32(axis[ax] * axis[ax]) > f32(0.5):
                m = f32(2.0) * safe_div(f32(box_size[ax]), f32(abs(halfaxis[ax])))
                secondpos = f32(min(f32(1.0) - f32(f32(mul) * bestsegmentpos), m))
            else:
                m = f32(2.0) * min(safe_div(f32(box_size[ax1]), f32(abs(halfaxis[ax1]))),
                                   safe_div(f32(box_size[ax2]), f32(abs(halfaxis[ax2]))))
                secondpos = f32(-min(f32(1.0) + f32(f32(mul) * bestsegmentpos), m))
            secondpos = f32(secondpos * f32(mul))
    elif cltype >= 0 and cltype // 3 == 1:
        branch = "edge"
        c1 = axisdir ^ clcorner
        c1 &= 7 - (1 << cledge)
        if c1 in (1, 2, 4):
            if cledge == 0:
                ax1, ax2 = 1, 2
            if cledge == 1:
                ax1, ax2 = 2, 0
            if cledge == 2:
                ax1, ax2 = 0, 1
            ax = cledge
            if abs(f32(axis[ax1])) > abs(f32(axis[ax2])):
                ax1 = ax2
            ax2 = 3 - ax - ax1
            if c1 & (1 << ax2):
                mul = 1
                secondpos = f32(f32(1.0) - bestsegmentpos)
            else:
                mul = -1
                secondpos = f32(f32(1.0) + bestsegmentpos)
            e1 = f32(2.0) * safe_div(f32(box_size[ax2]), f32(abs(halfaxis[ax2])))
            secondpos = f32(min(e1, secondpos))
            if ((axisdir & (1 << ax)) != 0) == ((c1 & (1 << ax2)) != 0):
                e2 = f32(f32(1.0) - bestboxpos)
            else:
                e2 = f32(f32(1.0) + bestboxpos)
            e1 = f32(f32(box_size[ax]) * safe_div(e2, f32(abs(halfaxis[ax]))))
            secondpos = f32(min(e1, secondpos))
            secondpos = f32(secondpos * f32(mul))
    elif cltype < 0:
        branch = "face"
        if clface != -1:
            mul = 1 if cltype == -3 else -1
            secondpos = f32(2.0)
            tmp1 = f32(np.subtract(pos, f32(halfaxis * f32(mul))))
            for i in range(3):
                if i != clface:
                    ha_r = safe_div(f32(mul), halfaxis[i])
                    e1 = f32(f32(box_size[i] - tmp1[i]) * ha_r)
                    if f32(0.0) < e1 < secondpos:
                        secondpos = e1
                    e1 = f32(f32(-f32(box_size[i]) - tmp1[i]) * ha_r)
                    if f32(0.0) < e1 < secondpos:
                        secondpos = e1
            secondpos = f32(secondpos * f32(mul))
    s1_pos_l = f32(pos + f32(halfaxis * bestsegmentpos))
    s1_pos_g = f32(np.add(matmul(box_rot, s1_pos_l), box_pos))
    d1, p1, n1, sb1 = sphere_box(s1_pos_g, capsule_radius, box_pos, box_rot, box_size)
    if secondpos > f32(-3.0):
        s2_pos_l = f32(pos + f32(halfaxis * f32(secondpos + bestsegmentpos)))
        s2_pos_g = f32(np.add(matmul(box_rot, s2_pos_l), box_pos))
        d2, p2, n2, sb2 = sphere_box(s2_pos_g, capsule_radius, box_pos, box_rot, box_size)
        has_second = True
    else:
        d2, p2, n2, sb2 = f32(MJ_MAXVAL), None, None, None
        has_second = False
    info = {
        "branch": branch, "cltype": cltype, "clface": clface, "clcorner": clcorner,
        "cledge": cledge, "bestsegmentpos": float(bestsegmentpos),
        "secondpos": float(secondpos), "has_second": has_second,
        "space_branch": sb1,
    }
    return [d1, d2], (p1, p2), (n1, n2), info


def main():
    ref = json.loads(REF.read_text(encoding="utf-8"))
    out = {}
    for case in ["sphere-box-shallow", "sphere-box-deep", "capsule-box"]:
        d = CASES / case
        typ = r(d, "geom_type", "<i4")
        size = r(d, "geom_size", "<f4")
        xpos = r(d, "geom_xpos", "<f4")
        xmat = r(d, "geom_xmat", "<f4")
        meta = dict(line.split(" ", 1) for line in
                    (d / "meta.txt").read_text(encoding="utf-8").splitlines() if " " in line)
        nworld = int(meta["nworld"])
        g1, g2 = int(meta["g1"]), int(meta["g2"])
        type1, type2 = int(meta["type1"]), int(meta["type2"])
        rows = []
        for w in range(nworld):
            def gp(g):
                return xpos[(w * len(typ) + g) * 3:(w * len(typ) + g) * 3 + 3]

            def gm(g):
                return xmat[(w * len(typ) + g) * 9:(w * len(typ) + g) * 9 + 9]

            def gs(g):
                return size[g * 3:g * 3 + 3]
            rec = {"world": w}
            if type1 == 2 and type2 == 6:  # SPHERE, BOX
                dd, pp, nn, br = sphere_box(gp(g1), gs(g1)[0], gp(g2), gm(g2), gs(g2))
                rec.update({"sphere_box_branch": br, "dist": float(dd),
                            "pos": pp.tolist(), "normal": nn.tolist()})
                # 是否盒内：|center| <= size 分量式
                center = transpose_mul(gm(g2), f32(np.subtract(gp(g1), gp(g2))))
                inside = bool(np.all(np.abs(center) <= np.asarray(gs(g2))))
                rec["center_local"] = center.tolist()
                rec["center_inside_box"] = inside
            elif type1 == 3 and type2 == 6:  # CAPSULE, BOX
                cap_rot = gm(g1)
                axis = np.array([cap_rot[2], cap_rot[5], cap_rot[8]], dtype=np.float32)
                dists, poss, norms, info = capsule_box(
                    gp(g1), axis, gs(g1)[0], gs(g1)[1], gp(g2), gm(g2), gs(g2))
                rec.update(info)
                rec.update({"axis": axis.tolist(), "dists": [float(x) for x in dists]})
            rows.append(rec)
        # 参考值（上游 f32 GPU）
        c = ref["cases"][case]
        out[case] = {
            "worlds": rows,
            "reference": {
                "count": c["count"],
                "dist": c["fields"]["dist"],
                "pos": c["fields"]["pos"],
                "frame": c["fields"]["frame"],
            },
        }
    dest = ROOT / "target/qa-primitive/case-branch-analysis.json"
    dest.write_text(json.dumps(out, indent=2, ensure_ascii=False), encoding="utf-8")
    print(dest)
    print(json.dumps(out, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
