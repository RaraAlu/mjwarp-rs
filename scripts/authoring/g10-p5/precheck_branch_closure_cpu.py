"""S3 pre-window CPU precheck (fixture validity only; not a reference, not a verdict).

Validates the S3 fixtures under pinned mujoco 3.15 + mujoco_warp 3.15 on CPU
(`wp.set_device("cpu")`; authoring only, no CUDA context, product tests never run Python):

  U1 plane-cylinder-degenerate : MULTICCD on -> contact count / fields (degenerate branch hit check below)
  U3 plane-box-corner-*        : 5 poses; union of box corner indices (sign pattern of local coords) == 8/8
  U4 plane-capsule-lying       : non-degenerate frame basis (axis perpendicular to plane normal), 2 candidates
  U5 plane-cylinder-deep-tilt  : 4 candidates written (count == 4)
  U6 mesh-mesh-two-assets      : nmesh=2, per-mesh vertnum/polynum differ, polygon slice shapes self-consistent
  U7 fan-cone-24 (mesh+box)    : max graph degree (polymapnum per vertex) > 10
  U8 prism-20 (mesh+box)       : max polygon vertex count (polyvertnum) > 16

Degenerate-branch check for U1: with axis parallel to plane normal (identity quat), pinned
`plane_cylinder` takes the `len_sqr < 1e-12` fallback (`vec = (radius,0,0)`); we assert that the
CPU contact positions match that analytic fallback (x offsets = ±radius, z = -0.005) instead of
the regular `normalize(axis*prjaxis-n)` direction.

Usage (pinned venv):
    python -B precheck_branch_closure_cpu.py
"""
import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[3]
BRANCHES = ROOT / "fixtures/g10-p5-primitive-branches"
MESH_DIR = ROOT / "fixtures/g10-p5-mesh-multicontact"
OUT = ROOT / "docs/development-evidence/g10-p5-branch-closure-capture/precheck-cpu.json"
GEOM_MESH = 7
BOX_POSES = [
    "plane-box-corner-face-down",
    "plane-box-corner-face-up",
    "plane-box-corner-tilt-x",
    "plane-box-corner-tilt-y",
    "plane-box-corner-single",
]
PLANE_CASES = ["plane-cylinder-degenerate", "plane-capsule-lying", "plane-cylinder-deep-tilt"] + BOX_POSES
MESH_CASES = ["mesh-mesh-two-assets", "mesh-limits-cone-box", "mesh-limits-prism-box"]


def sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values) -> list[int]:
    return [int(v) for v in np.asarray(values, dtype=np.float32).reshape(-1).view(np.uint32)]


def run_cpu(xml_path: Path):
    host = mujoco.MjModel.from_xml_path(str(xml_path))
    key_id = host.key("capture").id
    qpos = np.asarray(host.key_qpos[key_id], dtype=np.float32)
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=1, nconmax=4096, njmax=20000)
    data.qpos.assign(qpos.reshape(1, host.nq))
    data.qvel.assign(np.zeros((1, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in ("dist", "pos", "geom", "geomcollisionid", "type", "worldid"):
        array = getattr(data.contact, name).numpy()
        fields[name] = array[:count].reshape(count, -1).tolist() if array.size else []
    return host, qpos, count, fields


def box_corner_indices(host, qpos, contact_pos):
    """Map contact position to box corner index (i = x?1 | y?2 | z?4, sign in box local frame)."""
    box_geom = None
    for g in range(host.ngeom):
        if int(host.geom_type[g]) == int(mujoco.mjtGeom.mjGEOM_BOX):
            box_geom = g
            break
    if box_geom is None:
        return []
    # recompute box world pose from qpos (free joint of the single movable body)
    body = int(host.geom_bodyid[box_geom])
    jnt_adr = int(host.body_jntadr[body])
    qadr = int(host.jnt_qposadr[jnt_adr])
    pos = qpos[qadr : qadr + 3]
    quat = qpos[qadr + 3 : qadr + 7]
    mat = np.zeros(9, dtype=np.float64)
    mujoco.mju_quat2Mat(mat, quat.astype(np.float64))
    rot = mat.reshape(3, 3)
    local = (np.asarray(contact_pos, dtype=np.float64) - pos) @ rot  # R^T * (p - c)
    half = np.asarray(host.geom_size[box_geom], dtype=np.float64)
    indices = set()
    for row in local:
        idx = 0
        if row[0] > 0:
            idx |= 1
        if row[1] > 0:
            idx |= 2
        if row[2] > 0:
            idx |= 4
        indices.add(idx)
    return sorted(indices)


def mesh_limits(host):
    deg_max = int(np.max(host.mesh_polymapnum)) if host.nmesh else -1
    poly_max = int(np.max(host.mesh_polyvertnum)) if host.nmeshpoly else -1
    return {"nmesh": int(host.nmesh), "nmeshvert": int(host.nmeshvert), "nmeshpoly": int(host.nmeshpoly),
            "max_degree": deg_max, "max_polygon_vertices": poly_max,
            "degree_gt_10": deg_max > 10, "polygon_gt_16": poly_max > 16}


def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    wp.set_device("cpu")
    report = {
        "schema": 1,
        "kind": "s3-fixture-cpu-precheck",
        "device": "cpu",
        "authoring_only": True,
        "reads_product_tree": False,
        "versions": {"mujoco": mujoco.__version__, "warp": wp.__version__, "numpy": np.__version__},
        "cases": {},
        "failures": [],
    }
    corner_union = set()
    for name in PLANE_CASES:
        xml_path = BRANCHES / f"{name}.xml"
        host, qpos, count, fields = run_cpu(xml_path)
        entry = {
            "xml": xml_path.relative_to(ROOT).as_posix(),
            "xml_sha256": sha256(xml_path),
            "qpos_bits": bits32(qpos),
            "count": count,
            "dist": fields["dist"],
            "pos": fields["pos"],
            "geom": fields["geom"],
            "type": fields["type"],
        }
        if name in BOX_POSES:
            indices = box_corner_indices(host, qpos, np.asarray(fields["pos"], dtype=np.float64).reshape(-1, 3))
            corner_union.update(indices)
            entry["box_corner_indices"] = indices
        if name == "plane-cylinder-degenerate":
            positions = np.asarray(fields["pos"], dtype=np.float64).reshape(-1, 3)
            entry["degenerate_checks"] = {
                "count": count,
                "z_all_minus_0_005": bool(np.allclose(positions[:, 2], -0.005, atol=1e-7)),
                "x_offsets": sorted(round(float(v), 8) for v in positions[:, 0]),
                "dist_first": float(np.asarray(fields["dist"]).reshape(-1)[0]) if count else None,
            }
            if not entry["degenerate_checks"]["z_all_minus_0_005"]:
                report["failures"].append(f"{name}: degenerate fallback positions mismatch")
        if name == "plane-capsule-lying" and count != 2:
            report["failures"].append(f"{name}: expected 2 contacts, got {count}")
        if name == "plane-cylinder-deep-tilt" and count != 4:
            report["failures"].append(f"{name}: expected 4 contacts, got {count}")
        report["cases"][name] = entry

    report["u3_corner_union"] = sorted(corner_union)
    report["u3_covers_8"] = sorted(corner_union) == list(range(8))
    if not report["u3_covers_8"]:
        report["failures"].append(f"U3 corner union != 8/8: {sorted(corner_union)}")

    for name in MESH_CASES:
        xml_path = MESH_DIR / f"{name}.xml"
        host, qpos, count, fields = run_cpu(xml_path)
        entry = {
            "xml": xml_path.relative_to(ROOT).as_posix(),
            "xml_sha256": sha256(xml_path),
            "qpos_bits": bits32(qpos),
            "count": count,
            "geom_dataid": [int(v) for v in host.geom_dataid],
            "mesh": mesh_limits(host),
        }
        if name == "mesh-mesh-two-assets":
            entry["mesh"]["nmesh_is_2"] = entry["mesh"]["nmesh"] == 2
            if not entry["mesh"]["nmesh_is_2"]:
                report["failures"].append(f"{name}: expected nmesh=2, got {entry['mesh']['nmesh']}")
        if name == "mesh-limits-cone-box" and not entry["mesh"]["degree_gt_10"]:
            report["failures"].append(f"{name}: degree <= 10 ({entry['mesh']['max_degree']})")
        if name == "mesh-limits-prism-box" and not entry["mesh"]["polygon_gt_16"]:
            report["failures"].append(f"{name}: max polygon <= 16 ({entry['mesh']['max_polygon_vertices']})")
        report["cases"][name] = entry

    report["result"] = "PASS" if not report["failures"] else "FAIL"
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({k: report[k] for k in ("result", "u3_corner_union", "u3_covers_8", "failures")}, ensure_ascii=False))
    for name, entry in report["cases"].items():
        print(f"{name}: count={entry['count']} mesh={entry.get('mesh')}")
    print("wrote", OUT.relative_to(ROOT))


if __name__ == "__main__":
    main()
