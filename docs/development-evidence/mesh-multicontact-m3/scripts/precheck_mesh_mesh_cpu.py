"""G10@P5 M3 pre-window CPU precheck (not a reference, not a verdict).

Proves for the new mesh-mesh fixture (`fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml`):
  1. it parses under pinned mujoco 3.15 and has exactly 2 geoms, both `MESH` (type 7);
  2. NATIVECCD is disabled and MULTICCD is NOT disabled (same target flags as the M1 fixtures);
  3. both mesh geoms carry polygon data (`mesh_polyadr >= 0`, `mesh_polynum > 0`) and their
     per-mesh polygon/vertex offsets differ (`mesh_polyadr` / `mesh_vertadr`), i.e. the case
     exercises the **global** polygon sidecar arrays with **per-geom** offsets;
  4. on the pinned CPU backend the same fixture yields >1 contacts with MULTICCD enabled and
     fewer with MULTICCD disabled -> the fixture really triggers the multicontact path for a
     `(MESH, MESH)` pair (collision_driver.py MJ_COLLISION_TABLE maps it to CONVEX).

This is fixture-validity evidence only; the comparison reference is still the GPU capture
(separate signed capture window). No CUDA context is created (`wp.set_device("cpu")`); authoring
only, product tests never run Python.

Usage (pinned venv):
    python -B precheck_mesh_mesh_cpu.py
"""
import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
CASES = ["mesh-mesh"]
OUT = ROOT / "docs/development-evidence/mesh-multicontact-m3/precheck-cpu.json"
GEOM_MESH = 7


def sha256(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values) -> list[int]:
    return [int(v) for v in np.asarray(values, dtype=np.float32).reshape(-1).view(np.uint32)]


def mesh_poly_info(host, geom_index: int) -> dict:
    """Per-geom mesh polygon-data view used by the multicontact path (global arrays + offsets)."""
    dataid = int(host.geom_dataid[geom_index])
    info = {"geom_index": geom_index, "mesh_dataid": dataid}
    if dataid < 0:
        return info
    info.update(
        {
            "vertadr": int(host.mesh_vertadr[dataid]),
            "vertnum": int(host.mesh_vertnum[dataid]),
            "graphadr": int(host.mesh_graphadr[dataid]),
            "polynum": int(host.mesh_polynum[dataid]),
            "polyadr": int(host.mesh_polyadr[dataid]),
            "polyvertnum": int(host.mesh_polyvertnum[dataid]),
            "polyvertadr": int(host.mesh_polyvertadr[dataid]),
            "polymapnum": int(host.mesh_polymapnum[dataid]),
            "polymapadr": int(host.mesh_polymapadr[dataid]),
            "polygon_data_present": bool(
                int(host.mesh_polyadr[dataid]) >= 0 and int(host.mesh_polynum[dataid]) > 0
            ),
        }
    )
    return info


def run_cpu(xml_path: Path, disable_multiccd: bool):
    host = mujoco.MjModel.from_xml_path(str(xml_path))
    if disable_multiccd:
        host.opt.disableflags |= int(mujoco.mjtDisableBit.mjDSBL_MULTICCD)
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
    ids = data.contact.geomcollisionid.numpy()[:count].reshape(-1).tolist() if count else []
    dists = data.contact.dist.numpy()[:count].reshape(-1).tolist() if count else []
    geoms = data.contact.geom.numpy()[:count].reshape(-1, 2).tolist() if count else []
    return host, qpos, count, [int(v) for v in ids], [float(v) for v in dists], geoms


def main() -> int:
    wp.set_device("cpu")
    print("device:", wp.get_device())
    report = {
        "schema": 1,
        "task_id": "2026-10-26-mesh-multicontact-m3-mesh-mesh",
        "kind": "m3-mesh-mesh-fixture-cpu-precheck",
        "purpose": "fixture validity only: mesh-mesh fixture parses under pinned mujoco 3.15, both "
                   "geoms are MESH with polygon data, and the pinned CPU backend hits the MULTICCD "
                   "multicontact path; the comparison reference is the signed GPU capture",
        "upstream_revision": "71da24d956378a87a703b6e1442b13aec0c4ac29",
        "dispatch": "collision_driver.py MJ_COLLISION_TABLE (MESH, MESH) -> CONVEX; convex_narrowphase "
                    "-> ccd_kernel_builder multicontact branch (multicontact() in collision_gjk.py)",
        "authoring_only": True,
        "reads_product_tree": False,
        "product_python_dependency": False,
        "interpreter": {
            "path": sys.executable,
            "version": ".".join(map(str, sys.version_info[:3])),
        },
        "versions": {
            "mujoco": mujoco.__version__,
            "warp": wp.__version__,
            "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
            "numpy": np.__version__,
        },
        "authoring_script": "docs/development-evidence/mesh-multicontact-m3/scripts/precheck_mesh_mesh_cpu.py",
        "authoring_script_sha256": sha256(Path(__file__)),
        "device": str(wp.get_device()),
        "cases": {},
        "failures": [],
    }
    for name in CASES:
        xml_path = FIXTURES / f"{name}.xml"
        row = {"xml": str(xml_path.relative_to(ROOT))}
        try:
            host, qpos, count_on, ids_on, dists_on, geoms_on = run_cpu(xml_path, disable_multiccd=False)
            _, _, count_off, ids_off, dists_off, geoms_off = run_cpu(xml_path, disable_multiccd=True)
        except Exception as error:  # noqa: BLE001
            row["parse"] = f"FAIL {error}"
            report["failures"].append(f"{name}: run failed: {error}")
            report["cases"][name] = row
            continue
        row["parse"] = "ok"
        row["xml_sha256"] = sha256(xml_path)
        row["ngeom"] = int(host.ngeom)
        row["nq"] = int(host.nq)
        row["nativeccd_disabled"] = bool(
            host.opt.disableflags & int(mujoco.mjtDisableBit.mjDSBL_NATIVECCD)
        )
        row["multiccd_enabled_by_default"] = not bool(
            host.opt.disableflags & int(mujoco.mjtDisableBit.mjDSBL_MULTICCD)
        )
        row["geom_types"] = [int(t) for t in host.geom_type]
        row["geom_dataid"] = [int(v) for v in host.geom_dataid]
        row["mesh_poly_by_geom"] = [mesh_poly_info(host, index) for index in range(int(host.ngeom))]
        row["mesh_global_shapes"] = {
            "nmesh": int(host.nmesh),
            "nmeshvert": int(host.nmeshvert),
            "nmeshpoly": int(host.nmeshpoly),
            "nmeshpolymap": int(host.mesh_polymap.size),
            "nmeshpolyvert": int(host.mesh_polyvert.size),
            "mesh_polynum": [int(v) for v in host.mesh_polynum],
            "mesh_polyadr": [int(v) for v in host.mesh_polyadr],
            "mesh_vertadr": [int(v) for v in host.mesh_vertadr],
            "mesh_vertnum": [int(v) for v in host.mesh_vertnum],
        }
        row["multiccd_on"] = {
            "nacon": count_on,
            "geomcollisionid": ids_on,
            "dists": dists_on,
            "geoms": geoms_on,
        }
        row["multiccd_off"] = {
            "nacon": count_off,
            "geomcollisionid": ids_off,
            "dists": dists_off,
            "geoms": geoms_off,
        }
        row["qpos"] = qpos.tolist()
        row["qpos_bits"] = bits32(qpos)
        both_mesh = row["geom_types"] == [GEOM_MESH, GEOM_MESH]
        polygon_ok = all(
            entry.get("polygon_data_present", False) for entry in row["mesh_poly_by_geom"]
        )
        offsets_distinct = (
            row["mesh_poly_by_geom"][0].get("polyadr") != row["mesh_poly_by_geom"][1].get("polyadr")
            and row["mesh_poly_by_geom"][0].get("vertadr") != row["mesh_poly_by_geom"][1].get("vertadr")
        )
        multicontact_ok = count_on > 1 and count_on > count_off
        row["both_geoms_mesh"] = both_mesh
        row["polygon_data_present"] = polygon_ok
        row["per_geom_offsets_distinct"] = offsets_distinct
        row["multicontact_path_hit"] = multicontact_ok
        row["ok"] = bool(
            row["parse"] == "ok"
            and row["ngeom"] == 2
            and both_mesh
            and row["nativeccd_disabled"]
            and row["multiccd_enabled_by_default"]
            and polygon_ok
            and offsets_distinct
            and multicontact_ok
        )
        if not row["ok"]:
            report["failures"].append(
                f"{name}: ngeom={row['ngeom']} both_mesh={both_mesh} "
                f"nativeccd_off={row['nativeccd_disabled']} "
                f"multiccd_on={row['multiccd_enabled_by_default']} polygon={polygon_ok} "
                f"offsets_distinct={offsets_distinct} nacon(on/off)={count_on}/{count_off}"
            )
        report["cases"][name] = row
        print(
            f"{name}: nacon_on={count_on} nacon_off={count_off} "
            f"geoms={geoms_on} polygon={polygon_ok} offsets_distinct={offsets_distinct} "
            f"hit={multicontact_ok}"
        )
    report["result"] = "PASS" if not report["failures"] else "FAIL"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print("RESULT:", report["result"], "failures:", report["failures"])
    print("wrote", OUT)
    return 0 if not report["failures"] else 1


if __name__ == "__main__":
    sys.exit(main())
