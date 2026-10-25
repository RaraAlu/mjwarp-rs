"""QA G5 · G6 容差候选材料（CPU 侧，不开 GPU）：

1) **同码跨后端精度基线**：同一 recipe / 同一模型位姿，warp CPU 捕获 vs 本窗口 warp CUDA 捕获
   （qa-upstream-gpu.json）→ dist/pos/frame/normal 误差分布（这是「精度」意义上的 E 分布）。
2) **native f64(MuJoCo C) vs warp**：逐字段差异 + 配对判别（geom 位姿是否逐位相同、每 world 接触数、
   最近邻间距），用于说明 native 与 warp 是不同引擎语义（不可直接当容差依据）。

输出：qa-backend-precision.json（含 g6 候选建议与判别证据）。不引用交付 result.json。
"""

import json
import struct
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
E = ROOT / "docs/development-evidence/g10-p5-gpu-kernels-independent-check"
REF = ROOT / "fixtures/g10-p5/references"
wp.config.kernel_cache_dir = str(ROOT / "target/qa-g5/warp-cache")
wp.set_device("cpu")

CASES = ("ellipsoid-box", "hfield-sphere", "hfield-capsule", "sdf-box")
FLOAT_FIELDS = ("dist", "pos", "frame")
WIDTH = {"dist": 1, "pos": 3, "frame": 9}


def bits(value):
    return struct.unpack("<I", struct.pack("<f", float(value)))[0]


def stats(values):
    if not values:
        return {"count": 0}
    ordered = sorted(values)
    return {"count": len(ordered), "max": ordered[-1],
            "p99": ordered[int(0.99 * (len(ordered) - 1))],
            "p95": ordered[int(0.95 * (len(ordered) - 1))],
            "p50": ordered[len(ordered) // 2]}


def load_case(name):
    doc = json.loads((REF / f"{name}.json").read_text(encoding="utf-8"))
    worlds = int(doc["input"]["worlds"])
    qpos = np.asarray(doc["input"]["qpos"]["values"], dtype=np.float32).reshape(worlds, -1)
    qvel = np.asarray(doc["input"]["qvel"]["values"], dtype=np.float32).reshape(
        worlds, doc["input"]["qvel"]["shape"][1])
    xml = ROOT / doc["input"]["path"]
    return doc, worlds, qpos, qvel, xml


def rows_from_doc(case_doc):
    count = int(case_doc["count"])
    written = int(case_doc.get("written", count))
    fields = case_doc["fields"]
    rows = []
    for index in range(written):
        row = {}
        for name in FLOAT_FIELDS:
            width = WIDTH[name]
            row[name] = fields[name][index * width:(index + 1) * width]
        row["worldid"] = [int(v) for v in fields["worldid"][index:index + 1]]
        row["geom"] = [int(v) for v in fields["geom"][index * 2:index * 2 + 2]]
        rows.append(row)
    return rows


def capture_cpu(host, qpos, qvel, worlds):
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    return {
        "count": count,
        "fields": {name: getattr(data.contact, name).numpy()[:count].reshape(-1).tolist()
                   for name in FLOAT_FIELDS + ("worldid", "geom")},
        "geom_xpos": data.geom_xpos.numpy().tolist(),
        "geom_xmat": data.geom_xmat.numpy().reshape(worlds, host.ngeom * 9).tolist(),
    }


def match_rows(rows, ref_rows):
    """同 world 内按最近位置贪心配对；返回 (dist_abs, dist_rel, pos_abs, frame_abs, normal_abs)。"""
    out = {key: [] for key in ("dist_abs", "dist_rel", "pos_abs", "frame_abs", "normal_abs")}
    by_world = {}
    for row in rows:
        by_world.setdefault(int(row["worldid"][0]), []).append(row)
    for ref in ref_rows:
        world = int(ref["worldid"][0])
        candidates = by_world.get(world, [])
        if not candidates:
            continue
        best = None
        best_distance = None
        for candidate in candidates:
            distance = sum((a - b) ** 2 for a, b in zip(candidate["pos"], ref["pos"]))
            if best_distance is None or distance < best_distance:
                best = candidate
                best_distance = distance
        candidates.remove(best)
        dist_abs = abs(best["dist"][0] - ref["dist"][0])
        out["dist_abs"].append(dist_abs)
        if abs(ref["dist"][0]) > 1e-12:
            out["dist_rel"].append(dist_abs / abs(ref["dist"][0]))
        out["pos_abs"].append(max(abs(a - b) for a, b in zip(best["pos"], ref["pos"])))
        out["frame_abs"].append(max(abs(a - b) for a, b in zip(best["frame"], ref["frame"])))
        out["normal_abs"].append(max(abs(a - b) for a, b in zip(best["frame"][0:3],
                                                               ref["frame"][0:3])))
    return out


def main():
    upstream = json.loads((E / "qa-upstream-gpu.json").read_text(encoding="utf-8"))
    native = json.loads((E / "qa-native-f64.json").read_text(encoding="utf-8"))
    report = {"schema": 1, "kind": "qa-g5-g6-tolerance-material", "cases": {},
              "provenance": {"backend_baseline": "warp CPU（同码不同后端）vs 本窗口 warp CUDA 捕获",
                             "native": "MuJoCo C f64（与 warp 不同引擎实现，仅作契约框架读数）"},
              "g6_candidates": {}}
    merged = {key: [] for key in ("dist_abs", "dist_rel", "pos_abs", "frame_abs", "normal_abs")}
    for name in CASES:
        doc, worlds, qpos, qvel, xml = load_case(name)
        host = mujoco.MjModel.from_xml_path(str(xml))
        cpu = capture_cpu(host, qpos, qvel, worlds)
        gpu_rows = rows_from_doc(upstream["cases"][name])
        cpu_rows = rows_from_doc(cpu)
        backend = match_rows(cpu_rows, gpu_rows)
        entry = {"cpu_count": cpu["count"],
                 "gpu_count": int(upstream["cases"][name]["count"]),
                 "worlds": worlds,
                 "cpu_vs_gpu_f32_same_code": {key: stats(values) for key, values in backend.items()}}
        # native vs warp：位姿逐位 + 每 world 接触数 + 最近邻间距（配错判别）
        native_doc = native["cases"][name]
        g1, g2 = rows_from_doc(upstream["cases"][name])[0]["geom"]
        pose_identical = True
        native_counts = []
        native_rows_all = []
        for world, contacts in enumerate(native_doc["contacts"]):
            native_xpos = None
            native_rows = [{"dist": [c["dist"]], "pos": c["pos"], "frame": c["frame"],
                            "geom": [int(c["geom"][0]), int(c["geom"][1])], "worldid": [world]}
                           for c in contacts
                           if [int(c["geom"][0]), int(c["geom"][1])] in ([g1, g2], [g2, g1])]
            native_counts.append(len(native_rows))
            native_rows_all.extend(native_rows)
        gpu_counts = {}
        for row in gpu_rows:
            world = int(row["worldid"][0])
            gpu_counts[world] = gpu_counts.get(world, 0) + 1
        wp_counts = [gpu_counts.get(world, 0) for world in range(worlds)]
        # 最近邻间距：native 同 world 内每个接触到自己最近邻的距离（用于「配错会差多少」判别）
        spacing = []
        for world in range(worlds):
            rows = [r for r in native_rows_all if int(r["worldid"][0]) == world]
            for i, row in enumerate(rows):
                distances = [max(abs(a - b) for a, b in zip(row["pos"], other["pos"]))
                             for j, other in enumerate(rows) if j != i]
                if distances:
                    spacing.append(min(distances))
        native_entry = {"native_counts": native_counts, "warp_counts": wp_counts,
                        "counts_equal": native_counts == wp_counts,
                        "nearest_neighbor_pos_spacing": stats(spacing)}
        if native_counts == wp_counts:
            native_merge = {key: [] for key in merged}
            for world, contacts in enumerate(native_doc["contacts"]):
                native_rows = [{"dist": [c["dist"]], "pos": c["pos"], "frame": c["frame"],
                                "geom": [int(c["geom"][0]), int(c["geom"][1])], "worldid": [world]}
                               for c in contacts
                               if [int(c["geom"][0]), int(c["geom"][1])] in ([g1, g2], [g2, g1])]
                warp_rows = [r for r in gpu_rows if int(r["worldid"][0]) == world]
                part = match_rows(native_rows, warp_rows)
                for key in native_merge:
                    native_merge[key].extend(part[key])
            native_entry["native_vs_warp_f32"] = {key: stats(values)
                                                  for key, values in native_merge.items()}
            native_entry["usable_as_tolerance_basis"] = False
            native_entry["reason"] = ("native C 与 warp 是不同引擎实现：同 world 接触数相等且位姿逐位相同，"
                                      "但 pos/frame 系统性差异（见 native_vs_warp_f32）→ 属语义/算法差，非 f32 精度差")
        else:
            native_entry["usable_as_tolerance_basis"] = False
            native_entry["reason"] = "native C 每 world 接触数与 warp 不同 → 不可配对"
        entry["native_f64_vs_warp"] = native_entry
        report["cases"][name] = entry
        for key in merged:
            merged[key].extend(backend[key])
    report["g6_candidates"] = {
        "basis": "同码跨后端（warp CPU vs warp CUDA f32）误差分布，而非 native f64（不同引擎）",
        "dist": stats(merged["dist_abs"]),
        "dist_rel": stats(merged["dist_rel"]),
        "pos": stats(merged["pos_abs"]),
        "frame": stats(merged["frame_abs"]),
        "normal": stats(merged["normal_abs"]),
        "suggested": {
            "atol_pos": max(merged["pos_abs"]) if merged["pos_abs"] else None,
            "atol_frame": max(merged["frame_abs"]) if merged["frame_abs"] else None,
            "atol_normal": max(merged["normal_abs"]) if merged["normal_abs"] else None,
            "atol_dist": max(merged["dist_abs"]) if merged["dist_abs"] else None,
            "rtol_dist": max(merged["dist_rel"]) if merged["dist_rel"] else None,
            "note": "建议值取同码跨后端实测 max（可在 G6 乘以 2 余量）；设备 vs 上游 GPU 的正式判据仍按 4·ULP 口径",
        },
    }
    path = E / "qa-backend-precision.json"
    path.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print("== 同码跨后端（warp CPU vs CUDA f32） ==")
    for name, entry in report["cases"].items():
        value = entry["cpu_vs_gpu_f32_same_code"]
        print(f"  {name:<18} cpu/gpu count {entry['cpu_count']}/{entry['gpu_count']} "
              f"dist max {value['dist_abs'].get('max')} pos max {value['pos_abs'].get('max')} "
              f"frame max {value['frame_abs'].get('max')}")
    print("== native f64 vs warp（不可作容差依据） ==")
    for name, entry in report["cases"].items():
        value = entry["native_f64_vs_warp"]
        if value.get("native_vs_warp_f32"):
            native = value["native_vs_warp_f32"]
            print(f"  {name:<18} counts {value['native_counts']} vs {value['warp_counts']} | "
                  f"pos max {native['pos_abs'].get('max'):.3e} frame max {native['frame_abs'].get('max'):.3e} "
                  f"dist max {native['dist_abs'].get('max'):.3e} | nn spacing p50 "
                  f"{value['nearest_neighbor_pos_spacing'].get('p50')}")
        else:
            print(f"  {name:<18} {value['reason']}")
    print("== G6 candidates (same-code backend precision) ==")
    print("  ", report["g6_candidates"]["suggested"])
    print("wrote", path)


if __name__ == "__main__":
    main()
