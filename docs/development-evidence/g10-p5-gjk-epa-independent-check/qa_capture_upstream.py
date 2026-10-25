"""QA G2 independent review: re-capture the upstream mujoco_warp f32 reference (CPU backend) and
compare it bitwise with the delivered upstream-mjwarp-f32.json; also dump product inputs
(geom/mesh metadata, upstream f32 poses, native f64->f32 poses) for QA's own Rust harness.

QA-written; does not reuse delivery support code.
"""

import hashlib
import json
import struct
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g2"
CACHE = QA / "warp-cache-cpu"
DELIVERED = ROOT / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")
CASES = [
    ("cylinder-box", "fixtures/g10-p5/models/cylinder-box.xml",
     "fixtures/g10-p5/references/cylinder-box.json", True),
    ("ellipsoid-box", "fixtures/g10-p5/models/ellipsoid-box.xml",
     "fixtures/g10-p5/references/ellipsoid-box.json", True),
    ("mesh-mesh", "fixtures/g10-p5/models/mesh-mesh.xml",
     "fixtures/g10-p5/references/mesh-mesh.json", True),
    ("convex-mesh", "fixtures/mjwarp-complex-physics/convex-mesh.xml",
     "fixtures/mjwarp-complex-physics/reference.json", False),
    ("degenerate-mesh", "fixtures/mjwarp-complex-physics/degenerate-mesh.xml",
     "fixtures/mjwarp-complex-physics/reference.json", False),
]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def f32_bits(values):
    return [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values]


def f32(values):
    return [float(np.float32(v)) for v in np.asarray(values).reshape(-1)]


def flatten(value):
    if isinstance(value, (list, tuple)):
        for item in value:
            yield from flatten(item)
    else:
        yield value


def nested_bits(value):
    return f32_bits(list(flatten(value)))


def load_qpos(reference_path, standalone, name):
    value = json.loads((ROOT / reference_path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(c for c in value["collision_cases"] if c["name"] == name)
    return np.asarray(value["input"]["qpos"]["values"], dtype=np.float32)


def load_native_reference(reference_path, standalone, name):
    value = json.loads((ROOT / reference_path).read_text(encoding="utf-8"))
    if not standalone:
        value = next(c for c in value["collision_cases"] if c["name"] == name)
    output = value["output"]
    fields = output["fields"]
    return {
        "count": output["count"],
        "worldid": fields["worldid"]["values"],
        "geomcollisionid": fields["geomcollisionid"]["values"],
        "dist": fields["dist"]["values"],
        "pos": fields["pos"]["values"],
        "frame": fields["frame"]["values"],
    }


def capture(host, qpos):
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=qpos.shape[0], nconmax=4096, njmax=20000)
    data.qpos.assign(qpos)
    data.qvel.assign(np.zeros((qpos.shape[0], host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = getattr(data.contact, name).numpy()
        if array.size == 0:
            fields[name] = []
        else:
            fields[name] = array[:count].reshape(count, -1).tolist()
    xpos = data.geom_xpos.numpy()
    xmat = data.geom_xmat.numpy().reshape(xpos.shape[0], xpos.shape[1], 9)
    return {
        "count": count,
        "fields": fields,
        "geom_xpos": xpos.tolist(),
        "geom_xmat": xmat.tolist(),
    }


def mesh_tables(mjm):
    out = []
    for mid in range(mjm.nmesh):
        vertadr = int(mjm.mesh_vertadr[mid])
        vertnum = int(mjm.mesh_vertnum[mid])
        polyadr = int(mjm.mesh_polyadr[mid])
        polynum = int(mjm.mesh_polynum[mid])
        graphadr = int(mjm.mesh_graphadr[mid])
        polyvert_start = int(mjm.mesh_polyvertadr[polyadr])
        polyvert_count = int(np.sum(mjm.mesh_polyvertnum[polyadr:polyadr + polynum]))
        polymap_start = int(mjm.mesh_polymapadr[vertadr])
        polymap_count = int(np.sum(mjm.mesh_polymapnum[vertadr:vertadr + vertnum]))
        out.append({
            "vert": f32(mjm.mesh_vert[vertadr:vertadr + vertnum].reshape(-1)),
            "polyadr": polyadr,
            "polynormal": f32(mjm.mesh_polynormal[polyadr:polyadr + polynum].reshape(-1)),
            "polyvertadr": [int(v) for v in mjm.mesh_polyvertadr[polyadr:polyadr + polynum]],
            "polyvertnum": [int(v) for v in mjm.mesh_polyvertnum[polyadr:polyadr + polynum]],
            "polyvert": [int(v) for v in mjm.mesh_polyvert[polyvert_start:polyvert_start + polyvert_count]],
            "polymapadr": [int(v) for v in mjm.mesh_polymapadr[vertadr:vertadr + vertnum]],
            "polymapnum": [int(v) for v in mjm.mesh_polymapnum[vertadr:vertadr + vertnum]],
            "polymap": [int(v) for v in mjm.mesh_polymap[polymap_start:polymap_start + polymap_count]],
            "graph": ([int(v) for v in mjm.mesh_graph[graphadr:graphadr + 2 + 2 * vertnum
                                                        + 2 * int(mjm.mesh_graph[graphadr])]]
                      if graphadr >= 0 else []),
        })
    return out


def native_poses(mjm, qpos):
    mjd = mujoco.MjData(mjm)
    out = {"geom_xpos": [], "geom_xmat": []}
    for world in range(qpos.shape[0]):
        mjd.qpos[:] = qpos[world]
        mujoco.mj_forward(mjm, mjd)
        out["geom_xpos"].append(np.asarray(f32(mjd.geom_xpos), dtype=np.float32)
                                .reshape(mjm.ngeom, 3).tolist())
        out["geom_xmat"].append(np.asarray(f32(mjd.geom_xmat), dtype=np.float32)
                                .reshape(mjm.ngeom, 9).tolist())
    return out


def compare_capture(mine, delivered, label):
    """Bitwise compare one case capture against the delivered record."""
    problems = []
    a, b = mine, delivered
    if a["count"] != b["count"]:
        problems.append({"field": "count", "qa": a["count"], "delivered": b["count"]})
    for name in FLOAT_FIELDS + INT_FIELDS:
        if nested_bits(a["fields"][name]) != nested_bits(b["fields"][name]):
            problems.append({"field": name})
    for name in ("geom_xpos", "geom_xmat"):
        if nested_bits(a[name]) != nested_bits(b[name]):
            problems.append({"field": name})
    return {"label": label, "bitwise_identical": not problems, "problems": problems}


def main():
    QA.mkdir(parents=True, exist_ok=True)
    CACHE.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(CACHE)
    wp.init()
    wp.set_device("cpu")
    delivered = json.loads(DELIVERED.read_text(encoding="utf-8"))
    report = {
        "schema": 1,
        "kind": "qa-g2-upstream-mjwarp-f32-recapture",
        "device": str(wp.get_device()),
        "versions": {"python": ".".join(map(str, __import__("sys").version_info[:3])),
                     "mujoco": mujoco.__version__, "warp": wp.__version__,
                     "numpy": np.__version__},
        "qa_script_sha256": sha256(__file__),
        "delivered_provenance_audit": {},
        "cases": {},
        "compare": {},
    }
    prov = delivered["provenance"]
    report["delivered_provenance_audit"] = {
        "task_id": delivered.get("task_id") == "2026-10-25-g10-p5-gjk-epa",
        "upstream_revision": delivered.get("upstream_revision") == "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "script_sha_matches_current_file": prov["authoring_script_sha256"] == sha256(
            ROOT / "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py"),
        "device_cpu": prov["device"] == "cpu",
        "versions": prov["versions"],
        "flags": {"authoring_only": prov["authoring_only"],
                  "reads_product_tree": prov["reads_product_tree"],
                  "product_python_dependency": prov["product_python_dependency"]},
    }
    compare = {}
    inputs = {"mujoco_version": mujoco.__version__, "cases": []}
    for name, xml_rel, ref_rel, standalone in CASES:
        xml = ROOT / xml_rel
        host = mujoco.MjModel.from_xml_path(str(xml))
        qpos_flat = load_qpos(ref_rel, standalone, name)
        nworld = qpos_flat.size // host.nq
        qpos = qpos_flat.reshape(nworld, host.nq)
        first = capture(host, qpos)
        second = capture(host, qpos)
        raw_identical = (first["count"] == second["count"]
                         and all(nested_bits(first["fields"][k]) == nested_bits(second["fields"][k])
                                 for k in first["fields"])
                         and nested_bits(first["geom_xpos"]) == nested_bits(second["geom_xpos"])
                         and nested_bits(first["geom_xmat"]) == nested_bits(second["geom_xmat"]))
        delivered_case = delivered["cases"][name]
        cmp_case = compare_capture(first, delivered_case["output"], name)
        cmp_case["two_captures_bitwise_identical"] = raw_identical
        cmp_case["qpos_bits_match_delivered"] = f32_bits(qpos.reshape(-1)) == delivered_case["qpos_bits"]
        cmp_case["xml_sha256_match"] = sha256(xml) == delivered_case["xml_sha256"]
        cmp_case["count"] = first["count"]
        compare[name] = cmp_case
        # product inputs for the QA Rust harness
        record = {
            "name": name,
            "xml": xml_rel,
            "xml_sha256": sha256(xml),
            "nworld": nworld,
            "nq": int(host.nq),
            "qpos": f32(qpos.reshape(-1)),
            "geom_type": [int(v) for v in host.geom_type],
            "geom_size": f32(host.geom_size),
            "geom_dataid": [int(v) for v in host.geom_dataid],
            "mesh": mesh_tables(host),
            "upstream_poses": {"geom_xpos": first["geom_xpos"], "geom_xmat": first["geom_xmat"]},
            "native_poses": native_poses(host, qpos),
            "native_reference": load_native_reference(ref_rel, standalone, name),
            "pair": sorted({tuple(g) for row in first["fields"]["geom"] for g in [row]}),
            "upstream": first,
        }
        inputs["cases"].append(record)
        report["cases"][name] = {
            "count": first["count"],
            "two_captures_bitwise_identical": raw_identical,
            "qpos_bits_match_delivered": cmp_case["qpos_bits_match_delivered"],
            "xml_sha256_match": cmp_case["xml_sha256_match"],
            "bitwise_vs_delivered": cmp_case["bitwise_identical"],
            "problems": cmp_case["problems"],
        }
        print("%-16s count=%d twice=%s vs_delivered=%s qpos=%s xml=%s" % (
            name, first["count"], raw_identical, cmp_case["bitwise_identical"],
            cmp_case["qpos_bits_match_delivered"], cmp_case["xml_sha256_match"]))
    report["compare"] = compare
    report["overall"] = {
        "all_cases_bitwise_equal_to_delivered": all(c["bitwise_identical"] for c in compare.values()),
        "all_two_captures_bitwise_identical": all(c["two_captures_bitwise_identical"] for c in compare.values()),
        "counts": {name: compare[name]["count"] for name in compare},
    }
    (QA / "qa-upstream-mjwarp-f32.json").write_text(json.dumps(report, indent=1), encoding="utf-8")
    (QA / "qa-case-inputs.json").write_text(json.dumps(inputs, indent=1), encoding="utf-8")
    print("overall:", json.dumps(report["overall"]))
    print("wrote", QA / "qa-upstream-mjwarp-f32.json", QA / "qa-case-inputs.json")


if __name__ == "__main__":
    main()
