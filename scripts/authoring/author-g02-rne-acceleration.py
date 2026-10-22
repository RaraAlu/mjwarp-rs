"""Offline authoring: frozen upstream MJWarp RNE acceleration reference.

OFFLINE ONLY. Never imported, built or executed by the Rust product, its tests,
gates or scripts. Output goes under docs/development-evidence/ and is not read
by the product. Product/test runs still use cargo only and never start Python.

Scenarios mirror tests/resident_rne_acceleration.rs:
  5 models x 3 gravities x 8 states x flags{False,True}, worlds=1 per record.
Reference is generated with the frozen upstream checkout 71da24d... using the
same .mjb models and the fixture qpos/qvel/qacc/gravity inputs. The existing
frozen native fixture (fixtures/rne-acceleration) is used as the natural
validation target (MuJoCo 3.12 f64, ABS=REL=2e-5).
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import importlib.metadata as metadata

import numpy as np
import warp as wp
import mujoco
import mujoco_warp as mjw

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
EXPECTED = {
    "python": "3.12.13",
    "mujoco": "3.15.0",
    "warp": "1.15.0",
    "numpy": "2.5.4",
    "mujoco-warp": "3.15.0",
}
ABS = 2e-5
REL = 2e-5
MODELS = (
    ("mixed-joints", "fixtures/native-probe/mixed-joints.xml"),
    ("inertial-tree", "fixtures/native-probe/inertial-tree.xml"),
    ("rotated-tree", "fixtures/kinematics/rotated-tree.xml"),
    ("zero-dof", "fixtures/native-probe/zero-dof.xml"),
    ("massless-tree", "fixtures/com-position/massless-tree.xml"),
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_host(name: str, xml_rel: str) -> "mujoco.MjModel":
    """Load a 3.15-loadable XML host, then stamp the product input model arrays.

    The frozen .mjb fixtures were authored by MuJoCo 3.12 and cannot be loaded by
    the pinned 3.15 authoring interpreter; the product input model for this
    scenario is fixtures/rne-bias/<name>.json, so the upstream model is rebuilt
    from the XML host with those exact arrays (same approach as the G03 authoring).
    """
    base = json.loads((ROOT / "fixtures" / "rne-bias" / f"{name}.json").read_text())
    host = mujoco.MjModel.from_xml_path(str(ROOT / xml_rel))
    for key, values in base["model"].items():
        if not hasattr(host, key):
            continue
        target = getattr(host, key)
        array = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        if target.size != array.size:
            raise RuntimeError(f"model field size mismatch for {name}: {key}")
        if target.dtype.kind == "f":
            target[:] = array.reshape(target.shape)
        else:
            np.testing.assert_array_equal(target.reshape(-1), array, err_msg=key)
    return host

def hex_bits(values) -> list[str]:
    array = np.ascontiguousarray(np.asarray(values, dtype=np.float32)).reshape(-1)
    return [format(int(v), "08x") for v in array.view(np.uint32)]


def rne_record(model, data, mjm, case, gravity) -> dict:
    """Run the upstream rne chain for one state; return both flags."""
    nv = int(mjm.nv)
    qpos = np.asarray(case["qpos"], dtype=np.float32)
    qvel = np.asarray(case["qvel"], dtype=np.float32)
    qacc = np.asarray(case["qacc"], dtype=np.float32)
    records = {}
    for flag in (False, True):
        data.qpos.assign(qpos.reshape(1, -1))
        data.qvel.assign(qvel.reshape(1, -1))
        data.qacc.assign(qacc.reshape(1, -1))
        wp.synchronize_device()
        mjw.kinematics(model, data)
        mjw.com_pos(model, data)
        mjw.crb(model, data)
        mjw.com_vel(model, data)
        mjw.rne(model, data, flg_acc=flag)
        wp.synchronize_device()
        qfrc = np.asarray(data.qfrc_bias.numpy())[0].astype(np.float32)
        cacc = np.asarray(data.cacc.numpy())[0].astype(np.float32)
        cfrc = np.asarray(data.cfrc_int.numpy())[0].astype(np.float32)
        if qfrc.size != nv or cacc.size != 6 * int(mjm.nbody) or cfrc.size != 6 * int(mjm.nbody):
            raise RuntimeError(f"unexpected field shapes nv={nv} nbody={mjm.nbody}")
        records[flag] = {
            "qfrc_bias_hex": hex_bits(qfrc),
            "cacc_hex": hex_bits(cacc),
            "cfrc_int_hex": hex_bits(cfrc),
            "qfrc_bias": [float(v) for v in qfrc],
        }
    return records


def main() -> int:
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    work = (ROOT / "target" / "g02-rne-authoring").resolve()
    work.mkdir(parents=True, exist_ok=True)

    versions = {
        "python": platform.python_version(),
        "mujoco": mujoco.__version__,
        "warp": wp.__version__,
        "numpy": np.__version__,
        "mujoco-warp": metadata.version("mujoco-warp"),
    }
    for key, expected in EXPECTED.items():
        if versions[key] != expected:
            raise RuntimeError(f"pinned version mismatch: {key}={versions[key]} expected {expected}")
    head = subprocess.check_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True).strip()
    if head != REVISION:
        raise RuntimeError(f"upstream HEAD mismatch: {head}")
    if subprocess.check_output(["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True).strip():
        raise RuntimeError("upstream checkout is not clean")

    wp.config.kernel_cache_dir = str(work / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")

    records = {}
    validation = {}
    for name, xml_rel in MODELS:
        fixture = json.loads((ROOT / "fixtures" / "rne-acceleration" / f"{name}.json").read_text())
        cases = fixture["cases"]
        gravities = fixture["gravities"]
        if len(cases) != 8 or len(gravities) != 3:
            raise RuntimeError(f"unexpected fixture shape for {name}")
        mjm = build_host(name, xml_rel)
        if int(mjm.nv) != int(fixture["nv"]) or int(mjm.nbody) != int(fixture["nbody"]):
            raise RuntimeError(f"fixture/model dimension mismatch for {name}")
        name_records = []
        name_validation = []
        for g in range(3):
            mjm.opt.gravity[:] = np.asarray(gravities[g], dtype=np.float64)
            model = mjw.put_model(mjm)
            data = mjw.make_data(mjm, nworld=1, nconmax=0, njmax=0)
            for state, case in enumerate(cases):
                first = rne_record(model, data, mjm, case, gravities[g])
                second = rne_record(model, data, mjm, case, gravities[g])
                for flag in (False, True):
                    if first[flag] != second[flag]:
                        raise RuntimeError(f"non-deterministic rne for {name} g={g} state={state} flag={flag}")
                    native = np.asarray(case["qfrc"][int(flag)][g], dtype=np.float64)
                    observed = np.asarray(first[flag]["qfrc_bias"], dtype=np.float64)
                    if observed.size != native.size:
                        raise RuntimeError("native qfrc size mismatch")
                    normalized = np.abs(observed - native) / (ABS + REL * np.abs(native))
                    max_error = float(np.max(np.abs(observed - native))) if native.size else 0.0
                    max_norm = float(np.max(normalized)) if native.size else 0.0
                    record = {
                        "model": name,
                        "gravity": g,
                        "gravity_values": [float(v) for v in gravities[g]],
                        "state": state,
                        "flag": flag,
                        "nv": int(mjm.nv),
                        "nbody": int(mjm.nbody),
                        "qfrc_bias_hex": first[flag]["qfrc_bias_hex"],
                        "cacc_hex": first[flag]["cacc_hex"],
                        "cfrc_int_hex": first[flag]["cfrc_int_hex"],
                    }
                    name_records.append(record)
                    name_validation.append({
                        "model": name,
                        "gravity": g,
                        "state": state,
                        "flag": flag,
                        "max_abs_error_vs_native": max_error,
                        "max_normalized_error_vs_native": max_norm,
                        "within_tolerance": max_norm <= 1.0,
                    })
        records[name] = name_records
        validation[name] = name_validation
        worst = max(entry["max_normalized_error_vs_native"] for entry in name_validation)
        if any(not entry["within_tolerance"] for entry in name_validation):
            worst_entry = max(name_validation, key=lambda e: e["max_normalized_error_vs_native"])
            print(f"natural-validation WARNING {name}: worst normalized {worst:.3e} state={worst_entry['state']} gravity={worst_entry['gravity']} flag={worst_entry['flag']}")
        print(f"natural-validation {name}: records={len(name_records)} worst_normalized={worst:.3e}")

    inputs = ["fixtures/rne-acceleration/manifest.json"]
    for name, xml_rel in MODELS:
        inputs.append(f"fixtures/rne-acceleration/{name}.json")
        inputs.append(xml_rel)
    provenance = {
        "schema": 1,
        "kind": "upstream-rne-acceleration-authoring",
        "offline_only": True,
        "product_chain_reads": False,
        "upstream_revision": REVISION,
        "upstream_head": head,
        "upstream_clean": True,
        "versions": versions,
        "gpu": {
            "name": str(wp.get_device()),
            "driver": subprocess.check_output(
                ["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"], text=True
            ).strip(),
        },
        "command": " ".join(sys.argv),
        "cwd": str(ROOT),
        "script_sha256": sha256_file(Path(__file__).resolve()),
        "inputs": {rel: sha256_file(ROOT / rel) for rel in inputs},
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "deterministic_runs": 2,
        "records": sum(len(v) for v in records.values()),
    }
    (output / "upstream-rne.json").write_text(json.dumps({
        "schema": 1,
        "scope": "upstream RNE acceleration; offline authoring only; not product fixtures",
        "upstream_revision": REVISION,
        "records": records,
    }, indent=2) + "\n")
    (output / "natural-validation.json").write_text(json.dumps({
        "schema": 1,
        "scope": "upstream RNE acceleration vs frozen native MuJoCo fixture",
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "validation": validation,
    }, indent=2) + "\n")
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(f"authored records={provenance['records']} output={output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
