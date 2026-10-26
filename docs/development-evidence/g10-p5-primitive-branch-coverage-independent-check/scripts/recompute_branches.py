"""Independent CPU recomputation of B3 branch predicates using pinned formulas.

Does NOT reuse the delivery precheck script. Loads each fixture with MuJoCo CPU
(pinned venv 3.12.13 / mujoco 3.15.0) to get geom types/sizes/mesh graph data,
takes geom_xpos/geom_xmat from the captured GPU reference (same device inputs),
and evaluates the branch predicates in float32 exactly as
frozen-source/_src/collision_primitive*.py does.
"""
import json
import struct
import sys
from pathlib import Path

import mujoco
import numpy as np

ROOT = Path(r"D:\Rust\mjwarp-rs")
REF = ROOT / "docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json"
FIX = ROOT / "fixtures/g10-p5-primitive-branches"
F = np.float32


def f32(x):
    return F(x)


def bits(x):
    return struct.unpack("<I", struct.pack("<f", float(x)))[0]


def vec(s):
    if isinstance(s, str):
        return np.array([F(v) for v in s.split()], dtype=np.float32)
    return np.array([F(v) for v in s], dtype=np.float32)


def dot(a, b):
    return F(F(a[0] * b[0]) + F(F(a[1] * b[1]) + F(a[2] * b[2])))


def sub(a, b):
    return np.array([F(a[0] - b[0]), F(a[1] - b[1]), F(a[2] - b[2])], dtype=np.float32)


def mul(a, s):
    s = F(s)
    return np.array([F(a[0] * s), F(a[1] * s), F(a[2] * s)], dtype=np.float32)


def fma(a, b, c):
    return F(float(a) * float(b) + float(c))


def main():
    ref = json.loads(REF.read_text(encoding="utf-8"))
    out = {"cases": [], "failures": []}
    for name, case in ref["cases"].items():
        xml = ROOT / case["xml"]
        model = mujoco.MjModel.from_xml_path(str(xml))
        data = mujoco.MjData(model)
        data.qpos[:] = np.array(case["qpos"], dtype=np.float64)
        mujoco.mj_forward(model, data)
        out_o = case["output"]
        gx0, gx1 = out_o["geom_xpos"][0]
        xm0, xm1 = out_o["geom_xmat"][0]
        p0 = vec(gx0)
        p1 = vec(gx1)
        m0 = vec(xm0).reshape(3, 3)
        m1 = vec(xm1).reshape(3, 3)
        t0, t1 = int(model.geom_type[0]), int(model.geom_type[1])
        s0 = [F(v) for v in model.geom_size[0]]
        s1 = [F(v) for v in model.geom_size[1]]
        row = {
            "case": name,
            "geom_type": [t0, t1],
            "geom_size": [[float(v) for v in s0], [float(v) for v in s1]],
            "xml_sha_matches_inputs": None,
            "hit": False,
            "branch": None,
            "predicates": {},
        }
        if name == "plane-mesh-graph":
            dataid = int(model.geom_dataid[1])
            vertnum = int(model.mesh_vertnum[dataid])
            graphadr = int(model.mesh_graphadr[dataid])
            graph_numvert = int(model.mesh_graph[graphadr]) if graphadr >= 0 else -1
            row["predicates"] = {
                "mesh_vertnum": vertnum,
                "mesh_graphadr": graphadr,
                "graph_numvert": graph_numvert,
                "graph_branch_condition": "graphadr != -1 and vertnum >= 10",
                "graph_branch": bool(graphadr != -1 and vertnum >= 10),
            }
            row["hit"] = row["predicates"]["graph_branch"]
            row["branch"] = "plane_convex graph hillclimb" if row["hit"] else "plane_convex exhaustive"
        elif name == "sphere-sphere-concentric":
            d = sub(p1, p0)
            length_sq = dot(d, d)
            dist = F(np.sqrt(length_sq, dtype=np.float32))
            row["predicates"] = {
                "center_distance_f32": float(dist),
                "distance_bits": bits(dist),
                "is_zero": bool(dist == 0.0),
                "fallback_n": [1.0, 0.0, 0.0] if dist == 0.0 else None,
            }
            row["hit"] = bool(dist == 0.0)
            row["branch"] = "sphere_sphere dist==0 fallback" if row["hit"] else "generic"
        elif name == "sphere-capsule-clamp":
            axis = np.array([m1[0][2], m1[1][2], m1[2][2]], dtype=np.float32)
            offset = sub(p0, p1)
            projection = dot(axis, offset)
            half = s1[1]
            clamped = min(max(projection, -half), half)
            row["predicates"] = {
                "axis": [float(v) for v in axis],
                "offset": [float(v) for v in offset],
                "projection_f32": float(projection),
                "half_length": float(half),
                "projection_gt_half": bool(projection > half),
                "clamped_value": float(clamped),
            }
            row["hit"] = bool(projection > half or projection < -half)
            row["branch"] = "sphere_capsule projection clamp" if row["hit"] else "projection interior"
        elif name in ("sphere-cylinder-cap", "sphere-cylinder-corner"):
            axis = np.array([m1[0][2], m1[1][2], m1[2][2]], dtype=np.float32)
            vecp = sub(p0, p1)
            x = dot(vecp, axis)
            a_proj = mul(axis, x)
            p_proj = sub(vecp, a_proj)
            p2 = dot(p_proj, p_proj)
            radius = s1[0]
            half = s1[1]
            collide_side = abs(x) < half
            collide_cap = p2 < F(radius * radius)
            row["predicates"] = {
                "x": float(x),
                "collide_side": bool(collide_side),
                "collide_cap": bool(collide_cap),
                "radius": float(radius),
                "half_height": float(half),
            }
            if name == "sphere-cylinder-cap":
                row["hit"] = bool((not collide_side) and collide_cap and x > 0)
                row["branch"] = "sphere_cylinder cap" if row["hit"] else "other"
            else:
                row["hit"] = bool((not collide_side) and (not collide_cap))
                row["branch"] = "sphere_cylinder corner" if row["hit"] else "other"
        elif name == "capsule-capsule-nonparallel":
            axis1 = np.array([m0[0][2], m0[1][2], m0[2][2]], dtype=np.float32)
            axis2 = np.array([m1[0][2], m1[1][2], m1[2][2]], dtype=np.float32)
            a1 = mul(axis1, s0[1])
            a2 = mul(axis2, s1[1])
            ma = dot(a1, a1)
            mb = -dot(a1, a2)
            mc = dot(a2, a2)
            det_unfused = F(F(ma * mc) - F(mb * mb))
            det_fused = fma(ma, mc, -F(mb * mb))
            mj_minval = F(1e-15)
            row["predicates"] = {
                "axis_dot": float(dot(axis1, axis2)),
                "ma": float(ma),
                "mb": float(mb),
                "mc": float(mc),
                "det_unfused": float(det_unfused),
                "det_fused": float(det_fused),
                "mj_minval": float(mj_minval),
                "nonparallel_unfused": bool(abs(det_unfused) >= mj_minval),
                "nonparallel_fused": bool(abs(det_fused) >= mj_minval),
            }
            row["hit"] = bool(abs(det_unfused) >= mj_minval and abs(det_fused) >= mj_minval)
            row["branch"] = "capsule_capsule non-parallel det" if row["hit"] else "parallel branch"
        # fixture sha vs inputs manifest
        manifest = json.loads((ROOT / "docs/development-evidence/g10-p5-primitive-branch-coverage/inputs-sha256.json").read_text(encoding="utf-8"))
        import hashlib

        actual = hashlib.sha256(xml.read_bytes()).hexdigest()
        manifest_sha = manifest["files"][xml.name]["sha256"]
        ref_sha = case["xml_sha256"]
        row["xml_sha_matches_inputs"] = bool(actual == manifest_sha == ref_sha)
        row["reference_count"] = int(out_o["count"])
        if not row["hit"]:
            out["failures"].append(f"{name}: branch predicate not hit")
        if not row["xml_sha_matches_inputs"]:
            out["failures"].append(f"{name}: xml sha mismatch")
        out["cases"].append(row)
        print(name, "branch=", row["branch"], "hit=", row["hit"], "ref_count=", row["reference_count"])
        for k, v in row["predicates"].items():
            print("   ", k, "=", v)
    (ROOT / "target/qa-b3").mkdir(parents=True, exist_ok=True)
    (ROOT / "target/qa-b3/branch-predicates.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
    print("FAILURES", out["failures"])


if __name__ == "__main__":
    main()
