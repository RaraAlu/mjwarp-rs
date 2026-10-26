"""B3 pre-window CPU precheck (not a reference, not a verdict): prove each new fixture
actually lands in its target branch by recomputing the pinned branch predicates in f32
from the fixture qpos, and record CPU-backend contact counts for sanity only.

The device-side comparison reference is still the GPU capture (separate file).
"""
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIXTURES = ROOT / "fixtures/g10-p5-primitive-branches"
CASES = [
    "plane-mesh-graph",
    "sphere-sphere-concentric",
    "sphere-capsule-clamp",
    "sphere-cylinder-cap",
    "sphere-cylinder-corner",
    "capsule-capsule-nonparallel",
]
MJ_MINVAL = 1.0e-15
MJ_MAXVAL = 1.0e10


def f32(values):
    return np.asarray(values, dtype=np.float32)


def check_plane_mesh_graph(host, geom_pos, geom_mat):
    vertadr = int(host.mesh_vertadr[0])
    vertnum = int(host.mesh_vertnum[0])
    graphadr = int(host.mesh_graphadr[0])
    verts = f32(host.mesh_vert[vertadr : vertadr + vertnum])
    convex_pos = f32(geom_pos[1])
    rot = f32(geom_mat[1]).reshape(3, 3)
    plane_pos = f32(geom_pos[0])
    plane_rot = f32(geom_mat[0]).reshape(3, 3)
    plane_normal = plane_rot @ f32([0.0, 0.0, 1.0])
    plane_pos_local = rot.T @ (plane_pos - convex_pos)
    n = rot.T @ plane_normal
    supports = [float(np.dot(plane_pos_local - v, n)) for v in verts]
    max_support = max(supports)
    graph_branch = graphadr != -1 and vertnum >= 10
    return {
        "branch": "plane_convex graph hillclimb (pinned L79/L140: graphadr != -1 and vertnum >= 10)",
        "graphadr": graphadr,
        "vertnum": vertnum,
        "graph_numvert": int(host.mesh_graph[graphadr]) if graphadr != -1 else None,
        "max_support": max_support,
        "graph_branch_taken": bool(graph_branch),
        "exhaustive_branch_taken": bool(not graph_branch),
        "support_positive": bool(max_support >= 0.0),
        "hit": bool(graph_branch and max_support >= 0.0),
    }


def check_sphere_sphere_concentric(geom_pos):
    pos1 = f32(geom_pos[0])
    pos2 = f32(geom_pos[1])
    delta = pos2 - pos1
    dist = float(np.sqrt(np.dot(delta, delta)))
    return {
        "branch": "sphere_sphere dist == 0 fallback (pinned L81-84: n = (1,0,0))",
        "center_dist": dist,
        "expected_normal": [1.0, 0.0, 0.0],
        "hit": bool(dist == 0.0),
    }


def check_sphere_capsule_clamp(geom_pos, geom_mat):
    sphere_pos = f32(geom_pos[0])
    cap_pos = f32(geom_pos[1])
    cap_mat = f32(geom_mat[1]).reshape(3, 3)
    axis = f32([cap_mat[0, 2], cap_mat[1, 2], cap_mat[2, 2]])
    half_length = 0.12
    offset = sphere_pos - cap_pos
    projection = float(np.dot(axis, offset))
    return {
        "branch": "sphere_capsule projection clamp (pinned L117-121: clamp(dot(axis,offset), -half, half))",
        "projection": projection,
        "half_length": half_length,
        "clamped": bool(abs(projection) > half_length),
        "hit": bool(abs(projection) > half_length),
    }


def check_sphere_cylinder(geom_pos, geom_mat, wanted):
    sphere_pos = f32(geom_pos[0])
    cyl_pos = f32(geom_pos[1])
    cyl_mat = f32(geom_mat[1]).reshape(3, 3)
    axis = f32([cyl_mat[0, 2], cyl_mat[1, 2], cyl_mat[2, 2]])
    radius = 0.06
    half_height = 0.1
    vec = sphere_pos - cyl_pos
    x = float(np.dot(vec, axis))
    a_proj = axis * x
    p_proj = vec - a_proj
    p_proj_sqr = float(np.dot(p_proj, p_proj))
    collide_side = abs(x) < half_height
    collide_cap = p_proj_sqr < radius * radius
    if collide_side and collide_cap:
        dist_cap = half_height - abs(x)
        dist_radius = radius - float(np.sqrt(p_proj_sqr))
        if dist_cap < dist_radius:
            collide_side = False
        else:
            collide_cap = False
    taken = "side" if collide_side else ("cap" if collide_cap else "corner")
    return {
        "branch": f"sphere_cylinder {wanted} branch (pinned L437-462)",
        "x": x,
        "p_proj_sqr": p_proj_sqr,
        "collide_side": bool(collide_side),
        "collide_cap": bool(collide_cap),
        "branch_taken": taken,
        "hit": bool(taken == wanted),
    }


def check_capsule_capsule_nonparallel(geom_pos, geom_mat):
    pos1 = f32(geom_pos[0])
    pos2 = f32(geom_pos[1])
    rot1 = f32(geom_mat[0]).reshape(3, 3)
    rot2 = f32(geom_mat[1]).reshape(3, 3)
    axis1 = f32([rot1[0, 2], rot1[1, 2], rot1[2, 2]])
    axis2 = f32([rot2[0, 2], rot2[1, 2], rot2[2, 2]])
    half1 = 0.12
    half2 = 0.12
    a1 = axis1 * half1
    a2 = axis2 * half2
    dif = pos1 - pos2
    ma = float(np.dot(a1, a1))
    mb = -float(np.dot(a1, a2))
    mc = float(np.dot(a2, a2))
    u = -float(np.dot(a1, dif))
    v = float(np.dot(a2, dif))
    det = ma * mc - mb * mb
    return {
        "branch": "capsule_capsule non-parallel det branch (pinned L177-205)",
        "det": det,
        "det_over_minval": bool(abs(det) >= MJ_MINVAL),
        "axis_dot": float(np.dot(axis1, axis2)),
        "hit": bool(abs(det) >= MJ_MINVAL),
    }


def run_cpu_collision(host, qpos):
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=1, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos.reshape(1, host.nq))
    data.qvel.assign(np.zeros((1, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    geom_pos = data.geom_xpos.numpy()[0]
    geom_mat = data.geom_xmat.numpy()[0].reshape(-1, 9)
    dists = data.contact.dist.numpy()[:count].reshape(-1).tolist() if count else []
    ids = data.contact.geomcollisionid.numpy()[:count].reshape(-1).tolist() if count else []
    return count, [float(value) for value in dists], [int(value) for value in ids], geom_pos, geom_mat


def main():
    wp.set_device("cpu")
    print("device:", wp.get_device())
    results = {}
    failures = []
    for name in CASES:
        xml = FIXTURES / f"{name}.xml"
        row = {"xml": xml.relative_to(ROOT).as_posix()}
        try:
            host = mujoco.MjModel.from_xml_path(str(xml))
        except Exception as error:  # noqa: BLE001
            row["parse"] = f"FAIL {error}"
            failures.append(f"{name}: parse failed: {error}")
            results[name] = row
            continue
        row["parse"] = "ok"
        row["ngeom"] = int(host.ngeom)
        row["nativeccd_disabled"] = bool(
            host.opt.disableflags & mujoco.mjtDisableBit.mjDSBL_NATIVECCD
        )
        key_id = host.key("capture").id
        qpos = f32(host.key_qpos[key_id])
        count, dists, ids, geom_pos, geom_mat = run_cpu_collision(host, qpos)
        row["cpu_backend_nacon"] = count
        row["cpu_backend_dists"] = dists
        row["cpu_backend_geomcollisionid"] = ids
        if name == "plane-mesh-graph":
            row.update(check_plane_mesh_graph(host, geom_pos, geom_mat))
        elif name == "sphere-sphere-concentric":
            row.update(check_sphere_sphere_concentric(geom_pos))
        elif name == "sphere-capsule-clamp":
            row.update(check_sphere_capsule_clamp(geom_pos, geom_mat))
        elif name == "sphere-cylinder-cap":
            row.update(check_sphere_cylinder(geom_pos, geom_mat, "cap"))
        elif name == "sphere-cylinder-corner":
            row.update(check_sphere_cylinder(geom_pos, geom_mat, "corner"))
        elif name == "capsule-capsule-nonparallel":
            row.update(check_capsule_capsule_nonparallel(geom_pos, geom_mat))
        row["ok"] = bool(
            row.get("ngeom") == 2
            and row.get("nativeccd_disabled") is True
            and row.get("hit") is True
            and count > 0
        )
        if not row["ok"]:
            failures.append(f"{name}: {row}")
        results[name] = row
        print(f"{name}: nacon={count} hit={row.get('hit')} branch={row.get('branch')}")
        for key in ("branch_taken", "det", "projection", "max_support", "graphadr", "vertnum",
                    "center_dist"):
            if key in row:
                print(f"  {key}={row[key]}")
    report = {
        "kind": "b3-fixture-cpu-precheck",
        "note": "precheck only: proves fixtures parse and hit the intended branch predicate in f32; "
                "the comparison reference is still the GPU capture",
        "interpreter": {"path": sys.executable, "version": ".".join(map(str, sys.version_info[:3]))},
        "versions": {
            "mujoco": mujoco.__version__,
            "warp": wp.__version__,
            "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
            "numpy": np.__version__,
        },
        "device": str(wp.get_device()),
        "cases": results,
        "failures": failures,
        "result": "PASS" if not failures else "FAIL",
    }
    out = ROOT / "target/g10-p5-primitive-branches-precheck/precheck-branches-cpu.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print("RESULT:", report["result"], "failures:", failures)
    print("wrote", out)
    sys.exit(0 if not failures else 1)


if __name__ == "__main__":
    main()
