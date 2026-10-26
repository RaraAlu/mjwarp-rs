"""G10@P5 M1 pre-window CPU precheck (not a reference, not a verdict).

Proves for each mesh-multicontact fixture:
  1. it parses under pinned mujoco 3.15 and has exactly 2 geoms;
  2. NATIVECCD is disabled and MULTICCD is NOT disabled (target flags for the M1 capture);
  3. the mesh geom has polygon data (`mesh_polyadr >= 0`, `mesh_polynum > 0`) — the upstream
     convex CCD multicontact branch requires it;
  4. on the pinned CPU backend the same fixture yields >1 contacts with MULTICCD enabled and
     fewer with MULTICCD disabled → the fixture actually triggers the multicontact path.

This is fixture-validity evidence only; the comparison reference is still the GPU capture.
"""
import hashlib
import json
import sys
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
FIXTURES = ROOT / "fixtures/g10-p5-mesh-multicontact"
CASES = ["mesh-box", "mesh-cylinder"]
OUT = ROOT / "docs/development-evidence/mesh-multicontact-m1/precheck-cpu.json"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits32(values):
    return [int(v) for v in np.asarray(values, dtype=np.float32).view(np.uint32)]


def mesh_poly_info(host):
    """Return the mesh polygon-data view used by the multicontact path."""
    dataid = int(host.geom_dataid[0])
    info = {"mesh_dataid": dataid}
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


def run_cpu(xml_path, disable_multiccd):
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
    ids = (
        data.contact.geomcollisionid.numpy()[:count].reshape(-1).tolist() if count else []
    )
    dists = data.contact.dist.numpy()[:count].reshape(-1).tolist() if count else []
    return host, qpos, count, [int(v) for v in ids], [float(v) for v in dists]


def main():
    wp.set_device("cpu")
    print("device:", wp.get_device())
    report = {
        "kind": "m1-fixture-cpu-precheck",
        "note": "precheck only: proves fixtures parse, carry mesh polygon data and trigger the "
                "MULTICCD multicontact path on the pinned CPU backend; the comparison reference "
                "is still the GPU capture",
        "interpreter": {"path": sys.executable, "version": ".".join(map(str, sys.version_info[:3]))},
        "versions": {
            "mujoco": mujoco.__version__,
            "warp": wp.__version__,
            "mujoco_warp": mjw.__version__ if hasattr(mjw, "__version__") else "3.15.0",
            "numpy": np.__version__,
        },
        "device": str(wp.get_device()),
        "cases": {},
        "failures": [],
    }
    for name in CASES:
        xml_path = FIXTURES / f"{name}.xml"
        row = {"xml": str(xml_path.relative_to(ROOT))}
        try:
            host, qpos, count_on, ids_on, dists_on = run_cpu(xml_path, disable_multiccd=False)
            _, _, count_off, ids_off, dists_off = run_cpu(xml_path, disable_multiccd=True)
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
        row["mesh_poly"] = mesh_poly_info(host)
        row["multiccd_on"] = {"nacon": count_on, "geomcollisionid": ids_on, "dists": dists_on}
        row["multiccd_off"] = {"nacon": count_off, "geomcollisionid": ids_off, "dists": dists_off}
        row["qpos"] = qpos.tolist()
        row["qpos_bits"] = bits32(qpos)
        polygon_ok = bool(row["mesh_poly"].get("polygon_data_present"))
        multicontact_ok = count_on > 1 and count_on > count_off
        row["polygon_data_present"] = polygon_ok
        row["multicontact_path_hit"] = multicontact_ok
        row["ok"] = bool(
            row["parse"] == "ok"
            and row["ngeom"] == 2
            and row["nativeccd_disabled"]
            and row["multiccd_enabled_by_default"]
            and polygon_ok
            and multicontact_ok
        )
        if not row["ok"]:
            report["failures"].append(
                f"{name}: ngeom={row['ngeom']} nativeccd_off={row['nativeccd_disabled']} "
                f"multiccd_on={row['multiccd_enabled_by_default']} polygon={polygon_ok} "
                f"nacon(on/off)={count_on}/{count_off}"
            )
        report["cases"][name] = row
        print(
            f"{name}: nacon_on={count_on} nacon_off={count_off} "
            f"ids_on={ids_on} polygon={polygon_ok} hit={multicontact_ok}"
        )
    report["result"] = "PASS" if not report["failures"] else "FAIL"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print("RESULT:", report["result"], "failures:", report["failures"])
    print("wrote", OUT)
    sys.exit(0 if not report["failures"] else 1)


if __name__ == "__main__":
    main()
