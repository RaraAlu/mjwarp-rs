"""UNC-3 S3：README/receipt 逐条数字抽查（只读证据 + fixture）。

对 README §3（precheck 摘要）、§5.1（逐 case 关键数值）、§6（采样）、§7（偏差）里
可机器校验的数字逐条从证据 JSON / fixture 重新取值比对。
输出：target/qa-unc3-s3/spotcheck.json
"""
import json
import re
import sys
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EVID = REPO / "docs/development-evidence/g10-p5-branch-closure-capture"
OUT = Path(__file__).with_name("spotcheck.json")


def main():
    branch = json.loads((EVID / "upstream-branch-closure-gpu.json").read_text(encoding="utf-8"))
    limits = json.loads((EVID / "upstream-mesh-limits-gpu.json").read_text(encoding="utf-8"))
    precheck = json.loads((EVID / "precheck-cpu.json").read_text(encoding="utf-8"))
    cases = {**branch["cases"], **limits["cases"]}
    rows = []

    def add(name, claim=None, actual=None, ok=None):
        # 允许两种调用：add(name, expected, actual, ok) 或 add(name, actual, ok)
        if ok is None:
            actual, ok = claim, actual
            claim = actual
        rows.append({"claim": name, "expected": claim, "actual": actual, "match": bool(ok), "note": ""})

    # §3 precheck 摘要
    add("precheck.result", "PASS", precheck["result"], precheck["result"] == "PASS")
    add("precheck.failures", "[]", precheck["failures"], precheck["failures"] == [])
    add("precheck.u3_covers_8", "true", precheck["u3_covers_8"], precheck["u3_covers_8"] is True)
    union = precheck["u3_corner_union"]
    add("precheck.u3_corner_union", "[0..7]", union, union == list(range(8)))
    corner_sets = {
        name: precheck["cases"][name].get("box_corner_indices")
        for name in ("plane-box-corner-face-down", "plane-box-corner-face-up",
                     "plane-box-corner-tilt-x", "plane-box-corner-tilt-y", "plane-box-corner-single")
    }
    add("precheck.per_pose_corners", "{face-down:[0,1,2,3], face-up:[4,5,6,7], tilt-x:[0,1], tilt-y:[1,3], single:[7]}", corner_sets,
        corner_sets["plane-box-corner-face-down"] == [0, 1, 2, 3]
        and corner_sets["plane-box-corner-face-up"] == [4, 5, 6, 7]
        and corner_sets["plane-box-corner-tilt-x"] == [0, 1]
        and corner_sets["plane-box-corner-tilt-y"] == [1, 3]
        and corner_sets["plane-box-corner-single"] == [7])

    # §3 逐 case count
    expected_counts = {
        "plane-cylinder-degenerate": 3, "plane-box-corner-face-down": 4, "plane-box-corner-face-up": 4,
        "plane-box-corner-tilt-x": 2, "plane-box-corner-tilt-y": 2, "plane-box-corner-single": 1,
        "plane-capsule-lying": 2, "plane-cylinder-deep-tilt": 4, "mesh-mesh-two-assets": 3,
        "mesh-limits-cone-box": 1, "mesh-limits-prism-box": 4,
    }
    for name, count in expected_counts.items():
        add(f"count.{name}", count, cases[name]["output"]["count"], cases[name]["output"]["count"] == count)

    # §3 U6 角/限制统计
    u6 = precheck["cases"]["mesh-mesh-two-assets"]["mesh"]
    add("U6.nmesh", 2, u6["nmesh"], u6["nmesh"] == 2)
    add("U6.nmeshvert", 12, u6["nmeshvert"], u6["nmeshvert"] == 12)
    add("U6.nmeshpoly", 10, u6["nmeshpoly"], u6["nmeshpoly"] == 10)
    u7 = precheck["cases"]["mesh-limits-cone-box"]["mesh"]
    add("U7.max_degree", 24, u7["max_degree"], u7["max_degree"] == 24)
    u8 = precheck["cases"]["mesh-limits-prism-box"]["mesh"]
    add("U8.max_polygon_vertices", 20, u8["max_polygon_vertices"], u8["max_polygon_vertices"] == 20)
    add("U8.max_degree", 3, u8["max_degree"], u8["max_degree"] == 3)

    # §5.1 关键数值
    deg = cases["plane-cylinder-degenerate"]["output"]["fields"]
    add("U1.dist_all", "-0.009999997913837433",
        sorted({row[0] for row in deg["dist"]}),
        all(abs(row[0] - (-0.009999997913837433)) < 1e-18 for row in deg["dist"]))
    add("U1.pos_z_all", "-0.004999998956918716",
        sorted({row[2] for row in deg["pos"]}),
        all(abs(row[2] - (-0.004999998956918716)) < 1e-18 for row in deg["pos"]))
    add("U1.pos_x=[0.06,-0.03,-0.03]", [0.06, -0.03, -0.03],
        [round(row[0], 9) for row in deg["pos"]],
        sorted(round(row[0], 6) for row in deg["pos"]) == sorted([0.06, -0.03, -0.03]))

    fd = cases["plane-box-corner-face-down"]["output"]["fields"]
    add("U3.face_down.dist", -0.010000001639127731,
        sorted({row[0] for row in fd["dist"]}), all(abs(row[0] + 0.010000001639127731) < 1e-18 for row in fd["dist"]))
    add("U3.face_down.pos_z", -0.005000000819563866,
        all(abs(row[2] + 0.005000000819563866) < 1e-18 for row in fd["pos"]))

    tx = cases["plane-box-corner-tilt-x"]["output"]["fields"]
    add("U3.tilt_x.dist", -0.010000042617321014,
        all(abs(row[0] + 0.010000042617321014) < 1e-18 for row in tx["dist"]))
    add("U3.tilt_x.pos_y", -0.00707106152549386,
        all(abs(row[1] + 0.00707106152549386) < 1e-18 for row in tx["pos"]))

    cs = cases["plane-box-corner-single"]["output"]["fields"]
    add("U3.single.pos_x", 1.1678096711875696e-08, cs["pos"][0][0],
        abs(cs["pos"][0][0] - 1.1678096711875696e-08) < 1e-20)

    cap = cases["plane-capsule-lying"]["output"]["fields"]
    add("U4.endpoint_x", 0.12000001221895218, sorted(row[0] for row in cap["pos"]),
        sorted(round(row[0], 12) for row in cap["pos"]) == sorted([round(0.12000001221895218, 12), round(-0.12000001221895218, 12)]))

    deep = cases["plane-cylinder-deep-tilt"]["output"]["fields"]
    expected_deep = [-0.21660254895687103, -0.043397463858127594, -0.1716025471687317, -0.1716025471687317]
    add("U5.deep_tilt.dist", expected_deep, [row[0] for row in deep["dist"]],
        all(abs(a - b) < 1e-18 for a, b in zip([row[0] for row in deep["dist"]], expected_deep)))

    mm = cases["mesh-mesh-two-assets"]["output"]["fields"]
    add("U6.mesh_two_assets.dist", -0.009999997913837433,
        all(abs(row[0] + 0.009999997913837433) < 1e-18 for row in mm["dist"]))

    prism = cases["mesh-limits-prism-box"]["output"]["fields"]
    prism_pos = sorted([round(row[0], 6), round(row[1], 6), round(row[2], 6)] for row in prism["pos"])
    expected_prism = sorted([[-0.05, -0.05, 0.045], [-0.05, 0.05, 0.045], [0.05, -0.05, 0.045], [0.05, 0.05, 0.045]])
    add("U8.prism.pos", "4 接触 (±0.05, ±0.05, 0.045)（f32 量化）",
        prism_pos,
        all(abs(a - b) < 1e-6 for row_a, row_b in zip(prism_pos, expected_prism) for a, b in zip(row_a, row_b)))

    # §6 采样粗核（与 guard.log 对照）
    guard = (EVID / "logs/capture-attempt1-guard.log").read_text(encoding="utf-8")
    agg = dict(re.findall(r"(\w+)=([\d.\-]+)", guard.split("guard_aggregates")[-1]))
    add("samples.rows", 134, int(agg["samples"]), int(agg["samples"]) == 134)
    add("samples.baseline_used", 2849, int(agg["baseline_used"]), int(agg["baseline_used"]) == 2849)
    add("samples.used_peak", 3132, int(agg["used_peak"]), int(agg["used_peak"]) == 3132)
    add("samples.delta_peak", 283, int(agg["delta_peak"]), int(agg["delta_peak"]) == 283)
    add("samples.free_min", 12930, int(agg["free_min"]), int(agg["free_min"]) == 12930)
    add("samples.util_max", 94, int(agg["util_max"]), int(agg["util_max"]) == 94)
    add("samples.util_streak", 2, int(agg["util_streak"]), int(agg["util_streak"]) == 2)

    result = {"rows": rows, "all_match": all(row["match"] for row in rows), "mismatches": [row for row in rows if not row["match"]]}
    OUT.write_text(json.dumps(result, ensure_ascii=False, indent=1), encoding="utf-8")
    print(json.dumps({"claims": len(rows), "all_match": result["all_match"], "mismatches": result["mismatches"]}, ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
