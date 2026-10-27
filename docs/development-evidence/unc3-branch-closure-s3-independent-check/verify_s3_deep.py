"""UNC-3 S3 独立复核 · 深度复算（CPU，double 复算 + f32 输入量化）。

只读：8 个 plane fixture（XML qpos/size）、两个 obj、上游 GPU 参考、precheck-cpu.json、logs。
输出：target/qa-unc3-s3/verify_s3_deep.json

复算内容：
1. 8 个 plane case 用 pinned 公式（collision_primitive_core.py L366-390 / L283-308 /
   L495-557 / L312-339）从 fixture qpos 复算 dist/pos，与上游 GPU 参考逐值比 ULP；
2. fixture keyframe qpos 的 f32 位型与参考 `qpos_bits` 精确相等（同源证明）；
3. OBJ 拓扑（顶点数/多边形数/逐顶点多边形度/最大多边形顶点数）与 precheck mesh 统计比对；
4. compute-apps 计数/过滤、temp_max、window wall、floor、三口径 delta。
"""
import json
import math
import re
import struct
import sys
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/g10-p5-branch-closure-capture"
OUT = Path(__file__).with_name("verify_s3_deep.json")


def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


def bits(x):
    return struct.unpack("<I", struct.pack("<f", f32(x)))[0]


def ulp(a, b):
    ba, bb = bits(a), bits(b)
    if (ba >> 31) != (bb >> 31):
        return abs(ba & 0x7FFFFFFF) + abs(bb & 0x7FFFFFFF) + 1
    return abs(ba - bb)


def norm(v):
    n = math.sqrt(sum(c * c for c in v))
    return [c / n for c in v] if n else [0.0, 0.0, 0.0]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def quat_to_R(w, x, y, z):
    n = math.sqrt(w * w + x * x + y * y + z * z)
    w, x, y, z = w / n, x / n, y / n, z / n
    return [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]


def apply(R, v):
    return [sum(R[i][j] * v[j] for j in range(3)) for i in range(3)]


def plane_sphere(n, plane_pos, sphere_pos, radius):
    dist = dot([sphere_pos[i] - plane_pos[i] for i in range(3)], n) - radius
    pos = [sphere_pos[i] - n[i] * (radius + 0.5 * dist) for i in range(3)]
    return dist, pos


def plane_box(n, plane_pos, box_pos, R, size):
    center_dist = dot([box_pos[i] - plane_pos[i] for i in range(3)], n)
    rows = []
    for i in range(8):
        local = [
            size[0] if (i & 1) else -size[0],
            size[1] if (i & 2) else -size[1],
            size[2] if (i & 4) else -size[2],
        ]
        corner = apply(R, local)
        cdist = center_dist + dot(n, corner)
        pos = [corner[k] + box_pos[k] - 0.5 * n[k] * cdist for k in range(3)]
        rows.append({"corner": i, "dist": cdist, "pos": pos})
    return rows


def plane_capsule(n, plane_pos, capsule_pos, axis, radius, half_length):
    rows = []
    for sign in (1.0, -1.0):
        center = [capsule_pos[k] + sign * axis[k] * half_length for k in range(3)]
        dist, pos = plane_sphere(n, plane_pos, center, radius)
        rows.append({"dist": dist, "pos": pos})
    return rows


def plane_cylinder(n, plane_pos, center, R_axis, radius, half_height):
    axis = apply(R_axis, [0.0, 0.0, 1.0])
    prjaxis = dot(n, axis)
    if prjaxis > 0:
        axis = [-c for c in axis]
        prjaxis = -prjaxis
    dist0 = dot([center[k] - plane_pos[k] for k in range(3)], n)
    vec = [axis[k] * prjaxis - n[k] for k in range(3)]
    len_sqr = dot(vec, vec)
    if len_sqr >= 1e-12:
        vec = [c * (radius / math.sqrt(len_sqr)) for c in vec]
    else:
        vec = [radius, 0.0, 0.0]
    prjvec = dot(vec, n)
    axis_h = [c * half_height for c in axis]
    prjaxis_h = prjaxis * half_height
    dist1 = dist0 + prjaxis_h + prjvec
    pos1 = [center[k] + vec[k] + axis_h[k] - n[k] * (dist1 * 0.5) for k in range(3)]
    dist2 = dist0 - prjaxis_h + prjvec
    pos2 = [center[k] + vec[k] - axis_h[k] - n[k] * (dist2 * 0.5) for k in range(3)]
    prjvec1 = -prjvec * 0.5
    dist3 = dist0 + prjaxis_h + prjvec1
    vec1 = norm(cross(vec, axis_h))
    vec1 = [c * (radius * math.sqrt(3.0) * 0.5) for c in vec1]
    base = [center[k] + axis_h[k] - vec[k] * 0.5 for k in range(3)]
    pos3 = [base[k] + vec1[k] - n[k] * (dist3 * 0.5) for k in range(3)]
    pos4 = [base[k] - vec1[k] - n[k] * (dist3 * 0.5) for k in range(3)]
    return [
        {"dist": dist1, "pos": pos1},
        {"dist": dist2, "pos": pos2},
        {"dist": dist3, "pos": pos3},
        {"dist": dist3, "pos": pos4},
    ]


def parse_xml(path):
    text = path.read_text(encoding="utf-8")
    qpos = re.search(r'<key[^>]*qpos="([^"]+)"', text)
    size = re.search(r'<geom[^>]*type="(box|capsule|cylinder)"[^>]*size="([^"]+)"', text)
    return {
        "qpos": [float(v) for v in qpos.group(1).split()] if qpos else None,
        "geom_type": size.group(1) if size else None,
        "size": [float(v) for v in size.group(2).split()] if size else None,
    }


def obj_topology(path):
    verts = []
    faces = []
    for line in path.read_text(encoding="utf-8").splitlines():
        parts = line.split()
        if not parts:
            continue
        if parts[0] == "v":
            verts.append([float(v) for v in parts[1:4]])
        elif parts[0] == "f":
            faces.append([int(v.split("/")[0]) - 1 for v in parts[1:]])
    degree = [0] * len(verts)
    for face in faces:
        for index in face:
            degree[index] += 1
    return {
        "verts": len(verts),
        "faces": len(faces),
        "face_sizes": sorted({len(f) for f in faces}),
        "max_face_vertices": max(len(f) for f in faces),
        "degree_max": max(degree),
        "degree_hist": {str(d): degree.count(d) for d in sorted(set(degree))},
    }


def compare_rows(recomputed, reference, fields):
    out = {"rows_recomputed": len(recomputed), "rows_reference": len(reference), "matches": []}
    for index, row in enumerate(recomputed):
        if index >= len(reference):
            break
        ref = reference[index]
        entry = {"row": index}
        for field in fields:
            value = row[field] if not isinstance(row[field], list) else None
            if isinstance(row[field], list):
                ulps = [ulp(row[field][k], ref[field][k]) for k in range(3)]
                entry[f"{field}_ulp_max"] = max(ulps)
            else:
                entry[f"{field}_ulp"] = ulp(row[field], ref[field])
        out["matches"].append(entry)
    return out


def main():
    branch = json.loads((EVID / "upstream-branch-closure-gpu.json").read_text(encoding="utf-8"))
    precheck = json.loads((EVID / "precheck-cpu.json").read_text(encoding="utf-8"))
    result = {"plane_recompute": {}, "obj_topology": {}, "misc": {}}

    plane_cases = [
        "plane-cylinder-degenerate",
        "plane-capsule-lying",
        "plane-box-corner-face-down",
        "plane-box-corner-face-up",
        "plane-box-corner-tilt-x",
        "plane-box-corner-tilt-y",
        "plane-box-corner-single",
        "plane-cylinder-deep-tilt",
    ]
    for name in plane_cases:
        case = branch["cases"][name]
        xml_path = REPO / case["xml"]
        fixture = parse_xml(xml_path)
        qpos = [f32(v) for v in fixture["qpos"]]
        center = [qpos[0], qpos[1], qpos[2]]
        R = quat_to_R(qpos[3], qpos[4], qpos[5], qpos[6])
        n = [0.0, 0.0, 1.0]
        plane_pos = [0.0, 0.0, 0.0]
        if fixture["geom_type"] == "box":
            rows = plane_box(n, plane_pos, center, R, fixture["size"])
        elif fixture["geom_type"] == "capsule":
            rows = plane_capsule(n, plane_pos, center, apply(R, [0.0, 0.0, 1.0]), fixture["size"][0], fixture["size"][1])
        else:
            rows = plane_cylinder(n, plane_pos, center, R, fixture["size"][0], fixture["size"][1])
        contact = [row for row in rows if row["dist"] < 0.0]
        ref_fields = case["output"]["fields"]
        ref_rows = [
            {"dist": ref_fields["dist"][i][0], "pos": ref_fields["pos"][i]}
            for i in range(len(ref_fields["dist"]))
        ]
        result["plane_recompute"][name] = {
            "fixture_qpos_f32_bits": [bits(v) for v in qpos],
            "reference_qpos_bits": case["qpos_bits"],
            "qpos_bits_exact_match": [bits(v) for v in qpos] == case["qpos_bits"],
            "recomputed_count": len(contact),
            "reference_count": len(ref_rows),
            "compare": compare_rows(contact, ref_rows, ["dist", "pos"]),
        }

    # OBJ 拓扑 vs precheck mesh 统计
    for obj, case in [
        ("fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj", "mesh-limits-cone-box"),
        ("fixtures/g10-p5-mesh-multicontact/prism-20.obj", "mesh-limits-prism-box"),
    ]:
        topo = obj_topology(REPO / obj)
        mesh = precheck["cases"][case]["mesh"]
        result["obj_topology"][obj] = {
            "obj": topo,
            "precheck_mesh": mesh,
            "verts_match": topo["verts"] == mesh.get("nmeshvert"),
            "polygons_match": topo["faces"] == mesh.get("nmeshpoly"),
            "max_face_vertices_match": topo["max_face_vertices"] == mesh.get("max_polygon_vertices"),
            "degree_max_match": topo["degree_max"] == mesh.get("max_degree"),
        }

    # misc：窗口时长、floor、compute-apps、温度
    facts = (EVID / "logs/capture-attempt1-facts.txt").read_text(encoding="utf-8")
    baseline = (EVID / "logs/capture-attempt1-baseline.txt").read_text(encoding="utf-8")
    guard = (EVID / "logs/capture-attempt1-guard.log").read_text(encoding="utf-8")
    samples_csv = (EVID / "logs/capture-attempt1-samples.csv").read_text(encoding="utf-8").splitlines()
    rows = [line.split(",") for line in samples_csv[1:] if line.strip()]
    temps = [int(r[5]) for r in rows]
    base_points = [list(map(int, line.split(",")[:4])) for line in baseline.splitlines() if re.match(r"^\d+, \d+, \d+, \d+, [\d.]+$", line.strip())]
    start = re.search(r"window_start=(\S+)", facts).group(1)
    end = re.search(r"window_end=(\S+)", facts).group(1)

    def parse_ts(text):
        # 2026-10-27T09:37:42.194+08:00
        m = re.match(r"(\d{4}-\d{2}-\d{2})T(\d{2}):(\d{2}):(\d{2})\.(\d+)", text)
        import datetime

        base = datetime.datetime.strptime(m.group(1) + " " + m.group(2) + ":" + m.group(3) + ":" + m.group(4), "%Y-%m-%d %H:%M:%S")
        return base + datetime.timedelta(milliseconds=int(m.group(5)))

    wall = (parse_ts(end) - parse_ts(start)).total_seconds()

    post_apps_raw = re.search(r"post_compute_apps_raw=(.*)", facts).group(1)
    base_apps_raw = [line for line in baseline.splitlines() if line.startswith("2688,")]
    base_apps_raw = base_apps_raw[0] if base_apps_raw else ""
    post_entries = [e for e in post_apps_raw.split("|") if e.strip()]
    base_entries = [e for e in base_apps_raw.split("|") if e.strip()]
    task_filter = re.compile(r"python|g10_p5|cargo|mjwarp")
    result["misc"] = {
        "window_wall_s": round(wall, 3),
        "facts_window_wall_s": 41.5,
        "floor_recomputed": min(10240, base_points[0][1] - 3072),
        "facts_floor": 10085,
        "open_gate_free": 10240,
        "baseline_points": base_points,
        "temp_max": max(temps),
        "compute_apps_post_entries": len(post_entries),
        "compute_apps_post_insufficient": sum(1 for e in post_entries if "Insufficient Permissions" in e),
        "compute_apps_post_task_filtered": [e for e in post_entries if task_filter.search(e)],
        "compute_apps_baseline_entries": len(base_entries),
        "compute_apps_baseline_insufficient": sum(1 for e in base_entries if "Insufficient Permissions" in e),
        "compute_apps_baseline_task_filtered": [e for e in base_entries if task_filter.search(e)],
        "guard_preconditions_used_free_util": re.search(r"phase=preconditions n=1 used_mib=(\d+) free_mib=(\d+) util=(\d+)", guard).groups(),
        "guard_aggregates": guard.split("guard_aggregates")[-1].strip(),
    }

    OUT.write_text(json.dumps(result, ensure_ascii=False, indent=1), encoding="utf-8")
    summary = {
        "plane_cases": {
            name: {
                "recomputed_count": value["recomputed_count"],
                "reference_count": value["reference_count"],
                "count_match": value["recomputed_count"] == value["reference_count"],
                "qpos_bits_exact": value["qpos_bits_exact_match"],
                "dist_ulp_max": max(m.get("dist_ulp_max", m.get("dist_ulp", 0)) for m in value["compare"]["matches"]) if value["compare"]["matches"] else None,
                "pos_ulp_max": max(m.get("pos_ulp_max", 0) for m in value["compare"]["matches"]) if value["compare"]["matches"] else None,
            }
            for name, value in result["plane_recompute"].items()
        },
        "obj_topology": {k: {kk: vv for kk, vv in v.items() if kk.endswith("match")} for k, v in result["obj_topology"].items()},
        "misc": {k: result["misc"][k] for k in ("window_wall_s", "floor_recomputed", "temp_max", "compute_apps_post_entries", "compute_apps_post_insufficient", "compute_apps_post_task_filtered", "compute_apps_baseline_entries", "compute_apps_baseline_insufficient")},
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
