"""Offline authoring: frozen upstream MJWarp energy_vel reference for G26.

OFFLINE ONLY. Never imported, built or executed by the Rust product, its tests,
gates or scripts. Output goes to docs/development-evidence/g26-complete/ and to
fixtures/kinetic-energy/ (a static product fixture); the product never starts
Python. Product/test runs still use cargo only.

Scenarios:
  * fixtures/mass-matrix models (mixed-joints, inertial-tree, rotated-tree,
    zero-dof, massless-tree, armature-chain) with their 8 frozen qpos cases.
  * synthetic serial-slide chains nv in {3,33,65} mirroring the existing
    device-energy unit-test scenario (non-default lane/block counts).
Two deterministic f32 qvel vectors per state; upstream energy_vel = 0.5 q^T M q.

Natural validation: the same host model evaluated by native MuJoCo 3.15 f64
(mj_forward + mj_mulM) under ABS=REL=2e-5; real-model records additionally
cross-check the frozen native fixture matrix.
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
import mujoco
import mujoco_warp as mjw
import warp as wp

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
VECTORS = 2
STATES = 8
MODELS = (
    ("mixed-joints", "fixtures/native-probe/mixed-joints.xml"),
    ("inertial-tree", "fixtures/native-probe/inertial-tree.xml"),
    ("rotated-tree", "fixtures/kinematics/rotated-tree.xml"),
    ("zero-dof", "fixtures/native-probe/zero-dof.xml"),
    ("massless-tree", "fixtures/com-position/massless-tree.xml"),
    ("armature-chain", "fixtures/mass-matrix/armature-chain.xml"),
)
SYNTHETIC_NV = (3, 33, 65)
SYNTHETIC_MODEL_KEYS = (
    "body_parentid",
    "body_jntadr",
    "body_jntnum",
    "body_pos",
    "body_quat",
    "jnt_type",
    "jnt_bodyid",
    "jnt_qposadr",
    "jnt_dofadr",
    "jnt_pos",
    "jnt_axis",
    "body_ipos",
    "body_iquat",
    "body_mass",
    "body_inertia",
    "dof_bodyid",
    "dof_jntid",
    "dof_parentid",
    "dof_armature",
    "dof_damping",
)
FIXTURE = ROOT / "fixtures" / "kinetic-energy" / "upstream-energy-vel.json"
FIXTURE_MANIFEST = ROOT / "fixtures" / "kinetic-energy" / "manifest.json"


def write_lf(path: Path, text: str) -> None:
    """Write UTF-8 with LF newlines so hashes are checkout-independent."""
    path.write_bytes(text.encode("utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def hex_bits(values) -> list[str]:
    array = np.ascontiguousarray(np.asarray(values, dtype=np.float32)).reshape(-1)
    return [format(int(v), "08x") for v in array.view(np.uint32)]


def qvel_formula(nv: int, state: int, vector: int) -> np.ndarray:
    index = np.arange(nv, dtype=np.int64)
    raw = ((index * 3 + state * 5 + vector * 7) % 21).astype(np.float32)
    return (raw * np.float32(0.125) - np.float32(1.25)).astype(np.float32)


def qpos_synthetic(nv: int, state: int) -> np.ndarray:
    values = np.array([((j + state) % 9) for j in range(nv)], dtype=np.float32)
    return (values * np.float32(0.07) - np.float32(0.28)).astype(np.float32)


def stamp(host, arrays: dict) -> None:
    for key, values in arrays.items():
        if not hasattr(host, key):
            raise RuntimeError(f"host model missing field {key}")
        target = getattr(host, key)
        array = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        if target.size != array.size:
            raise RuntimeError(f"model field size mismatch {key}: host={target.size} json={array.size}")
        if target.dtype.kind == "f":
            target[:] = array.reshape(target.shape)
        else:
            np.testing.assert_array_equal(target.reshape(-1), array, err_msg=key)


def build_fixture_host(name: str, xml_rel: str):
    fixture = json.loads((ROOT / "fixtures" / "mass-matrix" / f"{name}.json").read_text())
    host = mujoco.MjModel.from_xml_path(str(ROOT / xml_rel))
    stamp(host, fixture["model"])
    host.opt.disableflags |= int(mujoco.mjtDisableBit.mjDSBL_CONTACT)
    if int(host.nv) != int(fixture["nv"]) or int(host.nbody) != int(fixture["nbody"]):
        raise RuntimeError(f"fixture/model dimension mismatch for {name}")
    return host, fixture


def synthetic_model_arrays(n: int) -> dict:
    nb = n + 1
    parents = [i - 1 if i > 0 else -1 for i in range(n)]
    arrays = {
        "body_parentid": [0] + [i + 1 for i in parents],
        "body_jntadr": [-1] + list(range(n)),
        "body_jntnum": [0] + [1] * n,
        "body_pos": [0.0, 0.0, 0.0] + [0.08, 0.01, 0.02] * n,
        "body_quat": [1.0, 0.0, 0.0, 0.0] * nb,
        "jnt_type": [3] * n,
        "jnt_bodyid": list(range(1, nb)),
        "jnt_qposadr": list(range(n)),
        "jnt_dofadr": list(range(n)),
        "jnt_pos": [0.0] * (3 * n),
        "jnt_axis": [0.0, 0.0, 1.0] * n,
        "body_ipos": [0.0, 0.0, 0.0] + [0.03, 0.01, 0.0] * n,
        "body_iquat": [1.0, 0.0, 0.0, 0.0] * nb,
        "body_mass": [0.0] + [0.01] * n,
        "body_inertia": [0.0, 0.0, 0.0] + [0.001] * (3 * n),
        "dof_bodyid": list(range(1, nb)),
        "dof_jntid": list(range(n)),
        "dof_parentid": parents,
        "dof_armature": [2.0] * n,
        "dof_damping": [0.0] * n,
    }
    return arrays


def build_synthetic_host(n: int):
    body = "".join(
        f'<body name="b{i}" pos="0.08 0.01 0.02"><joint type="hinge" axis="0 0 1"/>'
        f'<inertial pos="0.03 0.01 0" mass="0.01" diaginertia="0.001 0.001 0.001"/>'
        for i in range(1, n + 1)
    ) + "</body>" * n
    xml = (
        '<mujoco model="g26-synthetic-chain"><compiler angle="radian" fusestatic="false"/>'
        f"<worldbody>{body}</worldbody></mujoco>"
    )
    host = mujoco.MjModel.from_xml_string(xml)
    stamp(host, synthetic_model_arrays(n))
    host.opt.disableflags |= int(mujoco.mjtDisableBit.mjDSBL_CONTACT)
    if int(host.nv) != n or int(host.nbody) != n + 1:
        raise RuntimeError(f"synthetic model dimension mismatch n={n}: nv={host.nv} nbody={host.nbody}")
    return host


def sparse_fields(host) -> dict:
    return {
        "dof_parentid": [int(v) for v in host.dof_parentid],
        "m_rownnz": [int(v) for v in host.M_rownnz],
        "m_rowadr": [int(v) for v in host.M_rowadr],
        "m_colind": [int(v) for v in host.M_colind],
        "tree_dofadr": [int(v) for v in host.tree_dofadr],
        "tree_dofnum": [int(v) for v in host.tree_dofnum],
    }


def native_energy(host, qpos: np.ndarray, qvel: np.ndarray) -> float:
    nv = int(host.nv)
    if nv == 0:
        return 0.0
    data = mujoco.MjData(host)
    data.qpos[:] = qpos.astype(np.float64)
    data.qvel[:] = qvel.astype(np.float64)
    mujoco.mj_forward(host, data)
    res = np.zeros(nv, dtype=np.float64)
    mujoco.mj_mulM(host, data, res, data.qvel)
    return 0.5 * float(np.dot(data.qvel, res))


def upstream_energy(model, data, qpos: np.ndarray, qvel: np.ndarray) -> np.ndarray:
    nv = int(model.nv)
    if nv == 0:
        # 上游动能算子按 nv 静态展开；零自由度场景直接记录零向量（无产品物理量）。
        return np.array([0.0, 0.0], dtype=np.float32)
    data.qpos.assign(qpos.reshape(1, -1))
    data.qvel.assign(qvel.reshape(1, -1))
    wp.synchronize_device()
    mjw.fwd_position(model, data, factorize=False)
    mv = wp.zeros((1, nv), dtype=wp.float32)
    mjw.mul_m(model, data, mv, data.qvel)
    mjw.energy_vel(model, data)
    wp.synchronize_device()
    energy = np.asarray(data.energy.numpy())[0].astype(np.float32)
    return energy


def main() -> int:
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    work = (ROOT / "target" / "g26-authoring").resolve()
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

    scenarios = []
    validation = []
    records_total = 0
    for kind, name, xml_rel, n in [("fixture", m, x, 0) for m, x in MODELS] + [
        ("synthetic-chain", f"synthetic-chain-{n}", "", n) for n in SYNTHETIC_NV
    ]:
        if kind == "fixture":
            host, fixture = build_fixture_host(name, xml_rel)
        else:
            host = build_synthetic_host(n)
            fixture = None
        nv = int(host.nv)
        model = mjw.put_model(host)
        data = mjw.make_data(host, nworld=1, nconmax=0, njmax=0)
        cases = []
        for state in range(STATES):
            if kind == "fixture":
                qpos = np.asarray(fixture["cases"][state]["qpos"], dtype=np.float32)
                if nv == 0 and qpos.size != int(host.nq):
                    raise RuntimeError(f"{name} qpos size mismatch")
            else:
                qpos = qpos_synthetic(nv, state)
            for vector in range(VECTORS):
                qvel = qvel_formula(nv, state, vector)
                first = upstream_energy(model, data, qpos, qvel)
                second = upstream_energy(model, data, qpos, qvel)
                if not np.array_equal(first, second):
                    raise RuntimeError(f"non-deterministic energy for {name} state={state} vector={vector}")
                native = native_energy(host, qpos, qvel)
                kinetic = float(first[1]) if nv else 0.0
                normalized = abs(kinetic - native) / (ABS + REL * abs(native)) if nv else 0.0
                frozen = None
                if kind == "fixture" and nv:
                    native_matrix = np.asarray(fixture["cases"][state]["matrix"], dtype=np.float64).reshape(nv, nv)
                    frozen = 0.5 * float(qvel.astype(np.float64) @ native_matrix @ qvel.astype(np.float64))
                record = {
                    "state": state,
                    "vector": vector,
                    "qpos_hex": hex_bits(qpos),
                    "qvel_hex": hex_bits(qvel),
                    "energy_vel_hex": hex_bits(first),
                    "energy_vel": kinetic,
                    "native_f64_energy": native,
                    "normalized_error_vs_native": normalized,
                }
                if frozen is not None:
                    record["frozen_native_matrix_f64_energy"] = frozen
                    record["normalized_error_vs_frozen_matrix"] = abs(kinetic - frozen) / (ABS + REL * abs(frozen))
                cases.append(record)
                validation.append({
                    "scenario": name,
                    "state": state,
                    "vector": vector,
                    "nv": nv,
                    "normalized_error_vs_native": normalized,
                    "within_tolerance": normalized <= 1.0,
                    "normalized_error_vs_frozen_matrix": record.get("normalized_error_vs_frozen_matrix"),
                    "frozen_matrix_within_tolerance": (record.get("normalized_error_vs_frozen_matrix", 0.0) <= 1.0),
                })
                records_total += 1
        scenarios.append({
            "kind": kind,
            "name": name,
            "nv": nv,
            "nbody": int(host.nbody),
            "nq": int(host.nq),
            "sparse": sparse_fields(host),
            "cases": cases,
        })
        worst = max(entry["normalized_error_vs_native"] for entry in validation[-len(cases):])
        if any(not entry["within_tolerance"] for entry in validation[-len(cases):]):
            bad = max(validation[-len(cases):], key=lambda entry: entry["normalized_error_vs_native"])
            raise RuntimeError(
                f"natural validation failed for {name}: worst normalized {worst:.3e} at state={bad['state']} vector={bad['vector']}"
            )
        frozen_worst = max(
            (entry["normalized_error_vs_frozen_matrix"] or 0.0) for entry in validation[-len(cases):]
        )
        print(f"natural-validation {name}: nv={nv} records={len(cases)} worst_native={worst:.3e} worst_frozen_matrix={frozen_worst:.3e}")

    inputs = sorted(
        {
            *(f"fixtures/mass-matrix/{name}.json" for name, _ in MODELS),
            *(xml_rel for _, xml_rel in MODELS if xml_rel),
        }
    )
    provenance = {
        "schema": 1,
        "kind": "upstream-energy-vel-authoring",
        "offline_only": True,
        "product_chain_reads": False,
        "fixture_is_static_product_data": True,
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
        "inputs": {rel: sha256_file(ROOT / rel) for rel in inputs if (ROOT / rel).is_file()},
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "deterministic_runs": 2,
        "records": records_total,
    }
    upstream_doc = {
        "schema": 1,
        "scope": "upstream MJWarp energy_vel (kinetic) same-condition reference; offline authoring only",
        "upstream_revision": REVISION,
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "vectors": VECTORS,
        "states": STATES,
        "scenarios": scenarios,
    }
    FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    write_lf(FIXTURE, json.dumps(upstream_doc, indent=2) + "\n")
    write_lf(output / "upstream-energy-vel.json", json.dumps(upstream_doc, indent=2) + "\n")
    write_lf(output / "natural-validation.json", json.dumps({
        "schema": 1,
        "scope": "upstream energy_vel vs native MuJoCo 3.15 f64 under ABS=REL=2e-5",
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "records": len(validation),
        "within_tolerance": all(entry["within_tolerance"] and entry["frozen_matrix_within_tolerance"] for entry in validation),
        "validation": validation,
    }, indent=2) + "\n")
    write_lf(output / "provenance.json", json.dumps(provenance, indent=2) + "\n")
    manifest = {
        "schema": 1,
        "scope": "static G26 upstream energy_vel reference fixture; product tests read it, never regenerate it",
        "upstream_revision": REVISION,
        "generator_script": "scripts/authoring/author-g26-energy-vel.py",
        "generator_script_sha256": sha256_file(Path(__file__).resolve()),
        "generator_wrapper": "scripts/authoring/author-g26-energy-vel.ps1",
        "absolute_tolerance": ABS,
        "relative_tolerance": REL,
        "files": [
            {"path": "fixtures/kinetic-energy/upstream-energy-vel.json", "sha256": sha256_file(FIXTURE)},
            {"path": "fixtures/kinetic-energy/reference.json", "sha256": sha256_file(ROOT / "fixtures/kinetic-energy/reference.json")},
        ],
    }
    write_lf(FIXTURE_MANIFEST, json.dumps(manifest, indent=2) + "\n")
    print(f"authored records={records_total} scenarios={len(scenarios)} fixture={FIXTURE.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
