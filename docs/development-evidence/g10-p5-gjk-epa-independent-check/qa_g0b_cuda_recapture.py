"""QA G2 review side item: re-run G0b's 10 captures on the CUDA backend with QA's own harness,
twice each, and compare bitwise with the delivered fixtures (upgrades the G0b limitation).

QA-written; does not reuse delivery code. GPU window is short (kernel cache hit otherwise compile).
"""

import hashlib
import json
import struct
import subprocess
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(r"D:\Rust\mjwarp-rs")
QA = ROOT / "target/qa-g2"
FIX = ROOT / "fixtures/g10-p5"
FLOAT_FIELDS = ("dist", "pos", "frame", "includemargin", "friction", "solref",
                "solreffriction", "solimp", "adhesion")
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bits(values):
    return [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in values]


def capture(host, offset):
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=2, nconmax=4096, njmax=20000)
    qpos = np.tile(host.qpos0.astype(np.float32), (2, 1))
    qvel = np.zeros((2, host.nv), dtype=np.float32)
    for world in range(2):
        qpos[world, 0:3] += np.asarray(offset, dtype=np.float32) * world
        qvel[world] = np.linspace(-0.02 * (world + 1), 0.03 * (world + 1), host.nv, dtype=np.float32)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    count = int(data.nacon.numpy()[0])
    fields = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = getattr(data.contact, name).numpy()
        fields[name] = array[:count].reshape(count, -1).tolist() if array.size else []
    return {"count": count, "fields": fields}


def fixture_fields(name):
    doc = json.loads((FIX / "references" / (name + ".json")).read_text(encoding="utf-8"))
    count = doc["output"]["count"]
    out = {}
    for field, record in doc["output"]["fields"].items():
        values = record["values"]
        width = len(values) // count if count else 0
        out[field] = [values[i * width:(i + 1) * width] for i in range(count)]
    return count, out


def fields_equal(a, b):
    if len(a) != len(b):
        return False
    for name in FLOAT_FIELDS + INT_FIELDS:
        va, vb = a[name], b[name]
        if len(va) != len(vb):
            return False
        for ra, rb in zip(va, vb):
            if [int(v) for v in bits(ra)] != [int(v) for v in bits(rb)]:
                return False
    return True


def rows(a, count):
    out = []
    for index in range(count):
        row = {}
        for name in FLOAT_FIELDS + INT_FIELDS:
            width = len(a[name][index]) if len(a[name]) else 0
            row[name] = a[name][index] if width else []
        out.append(row)
    return out


def sort_key(row):
    return (int(row["worldid"][0]), bits(row["dist"]), bits(row["pos"]),
            bits(row["frame"]), tuple(int(v) for v in row["geom"]),
            int(row["type"][0]), int(row["dim"][0]), int(row["geomcollisionid"][0]))


def sorted_equal(a, b):
    if len(a) != len(b):
        return False
    sa = sorted(a, key=sort_key)
    sb = sorted(b, key=sort_key)
    for ra, rb in zip(sa, sb):
        for name in FLOAT_FIELDS:
            if bits(ra[name]) != bits(rb[name]):
                return False
        for name in INT_FIELDS:
            if [int(v) for v in ra[name]] != [int(v) for v in rb[name]]:
                return False
    return True


def smi(args):
    try:
        return subprocess.run(["nvidia-smi"] + args, capture_output=True, text=True, timeout=30).stdout.strip()
    except Exception as exc:  # noqa: BLE001
        return "ERROR: %s" % exc


def main():
    QA.mkdir(parents=True, exist_ok=True)
    cache = QA / "warp-cache-cuda"
    cache.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(cache)
    wp.init()
    wp.set_device("cuda:0")
    report = {
        "schema": 1,
        "kind": "qa-g0b-cuda-two-capture-recheck",
        "device": str(wp.get_device()),
        "versions": {"mujoco": mujoco.__version__, "warp": wp.__version__, "numpy": np.__version__},
        "gpu_before": {
            "memory": smi(["--query-gpu=memory.used,memory.total", "--format=csv,noheader"]),
            "compute_apps": smi(["--query-compute-apps=pid,process_name", "--format=csv,noheader"]),
        },
        "cases": {},
    }
    index = json.loads((FIX / "models/index.json").read_text(encoding="utf-8"))
    all_twice = True
    all_raw = True
    all_sorted = True
    for row in index["models"]:
        name = row["name"]
        host = mujoco.MjModel.from_xml_path(str(ROOT / row["xml"]))
        first = capture(host, row["offset"])
        second = capture(host, row["offset"])
        twice = fields_equal(first["fields"], second["fields"]) and first["count"] == second["count"]
        fcount, ffields = fixture_fields(name)
        raw_fixture = first["count"] == fcount and fields_equal(first["fields"], ffields)
        sorted_fixture = sorted_equal(rows(first["fields"], first["count"]), rows(ffields, fcount))
        all_twice = all_twice and twice
        all_raw = all_raw and raw_fixture
        all_sorted = all_sorted and sorted_fixture
        report["cases"][name] = {
            "count": first["count"],
            "delivered_count": fcount,
            "cuda_twice_raw_identical": twice,
            "cuda_matches_delivered_raw": raw_fixture,
            "cuda_matches_delivered_sorted": sorted_fixture,
            "worldid_sequence": [int(r[0]) for r in first["fields"]["worldid"]],
        }
        print("%-19s count=%-3d vs_delivered_count=%-3d twice=%-5s raw=%-5s sorted=%s" % (
            name, first["count"], fcount, twice, raw_fixture, sorted_fixture))
    report["overall"] = {
        "all_cuda_twice_raw_identical": all_twice,
        "all_match_delivered_raw": all_raw,
        "all_match_delivered_sorted": all_sorted,
    }
    report["gpu_after"] = {
        "memory": smi(["--query-gpu=memory.used,memory.total", "--format=csv,noheader"]),
        "compute_apps": smi(["--query-compute-apps=pid,process_name", "--format=csv,noheader"]),
        "processes": smi(["--query-compute-apps=process_name", "--format=csv,noheader"]),
    }
    (QA / "g0b-cuda-recapture.json").write_text(json.dumps(report, indent=1), encoding="utf-8")
    print("overall:", json.dumps(report["overall"]))
    print("gpu_before:", report["gpu_before"]["memory"], "| gpu_after:", report["gpu_after"]["memory"])


if __name__ == "__main__":
    main()
