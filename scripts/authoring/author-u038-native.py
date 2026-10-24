"""Offline authoring: native MuJoCo f64 CPU reference for the U038 3D scenario.

OFFLINE ONLY. Not imported or run by product builds/tests/scripts; it never
starts Python from the product side and does not read any GPU/product output.

Scenario identity: the frozen U038 3D fixture package
(`fixtures/gravity-upstream-abi/three-d-*`, upstream 71da24d, Warp 1.15.0,
7 worlds, free+ball+hinge+slide, nq=13, nv=11). This script rebuilds the same
model/state as a native MJCF, runs native MuJoCo f64 CPU (`mj_forward`) and
writes the first-reference values (`xpos/xquat/xmat/xipos/xanchor/xaxis/
subtree_com/cdof/qfrc_gravcomp`).

Quantization rule (contract: inputs take the same quantization): every model
parameter and qpos value used by the GPU references was already quantized to
f32; this script quantizes the same way (numpy.float32) and then widens to f64,
so both sides see the same real inputs. Outputs are kept as f64 raw values plus
an f32 round-to-nearest comparison value.

Native meaning of qfrc_gravcomp: `mjData.qfrc_gravcomp` is MuJoCo's generalized
antigravity force, sum_b J_b^T (-body_gravcomp_b * mass_b * gravity). The GPU
candidate/secondary values use the same definition. `qfrc_bias` at qvel=0 is
the gravity generalized force without the `body_gravcomp` scaling, so
`qfrc_gravcomp != -qfrc_bias` unless every body has body_gravcomp=1; the bias
values are archived only to document the mapping.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import sys

import numpy as np
import mujoco

ROOT = pathlib.Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "fixtures" / "gravity-upstream-abi"
KIN = FIXTURES / "three-d-kinematics"
MODEL_PACKAGE = FIXTURES / "three-d-model-package.json"
CHAIN_PACKAGE = FIXTURES / "three-d-package.json"
OUTPUT = ROOT / "docs" / "development-evidence" / "u038-3d" / "native-two-reference"
TASK_ID = "2026-10-24-u038-native-two-reference"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
WARP_VERSION = "1.15.0"
EXPECTED_VERSIONS = {"python": "3.12.13", "mujoco": "3.15.0", "numpy": "2.5.4"}
FIELD_ORDER = [
    "xpos",
    "xquat_wxyz",
    "xipos",
    "xmat",
    "xanchor",
    "xaxis",
    "subtree_com",
    "cdof",
]
# MuJoCo jnt_type values: 0 free / 1 ball / 2 slide / 3 hinge.
JOINT_NAMES = {0: "free", 1: "ball", 2: "slide", 3: "hinge"}
# xaxis only has a single Cartesian axis for hinge and slide joints; MuJoCo
# ignores jnt_axis for ball joints and free joints have no single axis, so the
# frozen Warp-derived xaxis entries for those joints are not the same field.
XAXIS_NATIVE_JOINT_TYPES = (2, 3)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: pathlib.Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: pathlib.Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2) + "\n").encode())


def f32(value) -> float:
    return float(np.float32(value))


def f32_values(values) -> list[float]:
    return [f32(value) for value in values]


def bits_hex(values: np.ndarray) -> list[str]:
    flat = np.asarray(values, dtype=np.float32).reshape(-1)
    return [f"{int(bits):08x}" for bits in flat.view(np.uint32)]


def from_hex_bits(hex_bits) -> np.ndarray:
    return np.array(
        [
            np.frombuffer(int(bits, 16).to_bytes(4, "little"), dtype=np.float32)[0]
            for bits in hex_bits
        ],
        dtype=np.float32,
    )


def fmt(values) -> str:
    """Serialize f32-quantized values into MJCF with round-trip precision."""
    return " ".join(repr(f32(value)) for value in values)


def build_xml(kin: dict, mass: list[float], gravcomp: list[float]) -> str:
    nbody = kin["nbody"]
    parent = kin["body_parentid"]

    def block(body: int) -> str:
        children = [b for b in range(1, nbody) if parent[b] == body]
        inner = "".join(block(child) for child in children)
        joint = ""
        for index, joint_body in enumerate(kin["jnt_bodyid"]):
            if joint_body != body:
                continue
            kind = kin["jnt_type"][index]
            if kind == 0:
                joint = "<freejoint/>"
            else:
                name = JOINT_NAMES[kind]
                joint = (
                    f'<joint type="{name}" pos="{fmt(kin["jnt_pos"][index])}"'
                    f' axis="{fmt(kin["jnt_axis"][index])}"/>'
                )
        inertial = (
            f'<inertial pos="{fmt(kin["body_ipos"][body])}" quat="1 0 0 0"'
            f' mass="{repr(mass[body])}" diaginertia="1 1 1"/>'
        )
        return (
            f'<body name="b{body}" pos="{fmt(kin["body_pos"][body])}"'
            f' quat="{fmt(kin["body_quat_wxyz"][body])}"'
            f' gravcomp="{repr(gravcomp[body])}">{joint}{inertial}{inner}</body>'
        )

    roots = [b for b in range(1, nbody) if parent[b] == 0]
    bodies = "".join(block(body) for body in roots)
    return f"<mujoco><worldbody>{bodies}</worldbody></mujoco>"


def validate_frozen_inputs(kin: dict, package: dict, model_package: dict) -> dict:
    chain = load(CHAIN_PACKAGE)
    if package["upstream_revision"] != REVISION or chain["upstream_revision"] != REVISION:
        raise RuntimeError("frozen upstream revision mismatch")
    if package["warp_version"] != WARP_VERSION:
        raise RuntimeError("frozen warp version mismatch")
    checks = {
        "kinematics/input.json": digest(KIN / "input.json"),
        "kinematics/output.json": digest(KIN / "output.json"),
        "kinematics/raw-bits.json": digest(KIN / "raw-bits.json"),
        "three-d-model-package.json": digest(MODEL_PACKAGE),
        "gravity/receipt.json": digest(FIXTURES / "generated" / "receipt.json"),
    }
    expected = {
        "kinematics/input.json": chain["kinematics"]["input_sha256"],
        "kinematics/output.json": chain["kinematics"]["output_sha256"],
        "kinematics/raw-bits.json": chain["kinematics"]["raw_bits_sha256"],
        "three-d-model-package.json": chain["model_package"]["sha256"],
        "gravity/receipt.json": chain["gravity"]["receipt_sha256"],
    }
    for name, expected_hash in expected.items():
        if checks[name] != expected_hash:
            raise RuntimeError(f"frozen hash mismatch: {name}")
    # Parameter periods must be identical rows, otherwise one native model would
    # not represent every world's parameter period.
    for name, periods in (
        ("gravity", chain["parameter_periods"]["gravity"]),
        ("body_mass", chain["parameter_periods"]["body_mass"]),
        ("body_gravcomp", chain["parameter_periods"]["body_gravcomp"]),
    ):
        rows = np.asarray(model_package["model"][f"{'opt_' if name == 'gravity' else ''}{name}"], dtype=np.float64)
        width = 3 if name == "gravity" else kin["nbody"]
        rows = rows.reshape(-1, width)
        if rows.shape[0] != periods:
            raise RuntimeError(f"{name} period count mismatch")
        if not np.array_equal(rows, np.repeat(rows[:1], rows.shape[0], axis=0)):
            raise RuntimeError(f"{name} periods differ; a single native model is invalid")
    if int(kin["worlds"]) != 7 or int(kin["nbranch"]) != 2:
        raise RuntimeError("unexpected scenario dimensions")
    if int(kin["nq"]) != 13 or int(kin["nv"]) != 11 or int(kin["nbody"]) != 5:
        raise RuntimeError("unexpected scenario dimensions")
    return {"files": checks, "periods": {"gravity": 2, "body_mass": 3, "body_gravcomp": 5}}


def dof_parentid_layout(kin: dict) -> list[int]:
    per_joint = {0: 6, 1: 3, 2: 1, 3: 1}
    body_last = {}
    parent = []
    for index, (body, kind) in enumerate(zip(kin["jnt_bodyid"], kin["jnt_type"])):
        parent_body = kin["body_parentid"][body]
        previous = -1
        while parent_body > 0:
            if parent_body in body_last:
                previous = body_last[parent_body]
                break
            parent_body = kin["body_parentid"][parent_body]
        for _ in range(per_joint[kind]):
            parent.append(previous)
            previous = len(parent) - 1
        body_last[body] = len(parent) - 1
    return parent


def native_fields(kin: dict, model, data) -> dict:
    worlds = int(kin["worlds"])
    native = {name: [] for name in FIELD_ORDER}
    gravity = []
    bias = []
    for world in range(worlds):
        qpos = np.asarray(kin["qpos"][world], dtype=np.float32).astype(np.float64)
        data.qpos[:] = qpos
        data.qvel[:] = 0.0
        mujoco.mj_forward(model, data)
        native["xpos"].extend(np.asarray(data.xpos).reshape(-1).tolist())
        native["xquat_wxyz"].extend(np.asarray(data.xquat).reshape(-1).tolist())
        native["xipos"].extend(np.asarray(data.xipos).reshape(-1).tolist())
        native["xmat"].extend(np.asarray(data.xmat).reshape(-1).tolist())
        native["xanchor"].extend(np.asarray(data.xanchor).reshape(-1).tolist())
        native["xaxis"].extend(np.asarray(data.xaxis).reshape(-1).tolist())
        native["subtree_com"].extend(np.asarray(data.subtree_com).reshape(-1).tolist())
        native["cdof"].extend(np.asarray(data.cdof).reshape(-1).tolist())
        gravity.extend(np.asarray(data.qfrc_gravcomp).reshape(-1).tolist())
        bias.extend(np.asarray(data.qfrc_bias).reshape(-1).tolist())
    return native, gravity, bias


def compare_against_frozen(native: dict, gravity: list[float], kin: dict) -> dict:
    """Sanity check: the native model must represent the same frozen scenario.

    This is a diagnostic only; native values are generated before reading these
    frozen GPU bits and are never adjusted to them.
    """
    raw = load(KIN / "raw-bits.json")
    frozen = {field["field"]: from_hex_bits(field["bits"]) for field in raw["fields"]}
    gravity_raw = load(
        ROOT / "docs/development-evidence/u038-3d/debt-2/upstream-gravity-raw.json"
    )
    gravity_frozen = from_hex_bits(gravity_raw["field_bits"][0])
    report = {"fields": {}, "gravity": {}}
    excluded = xaxis_excluded_indices(kin)
    for name in FIELD_ORDER:
        values = np.asarray(native[name], dtype=np.float64)
        expected = frozen[name].astype(np.float64)
        if values.size != expected.size:
            raise RuntimeError(f"frozen {name} length mismatch")
        mask = np.ones(values.size, dtype=bool)
        if name == "xaxis":
            mask[excluded] = False
        diff = np.abs(values[mask] - expected[mask])
        report["fields"][name] = {
            "compared": int(mask.sum()),
            "excluded": int((~mask).sum()),
            "max_abs": float(diff.max()),
            "frozen_f32_bits_differing": int(
                np.count_nonzero(values[mask].astype(np.float32) != frozen[name][mask])
            ),
        }
        if name == "xaxis":
            ball_free = ~mask
            report["fields"][name]["excluded_max_abs"] = float(
                np.abs(values[ball_free] - expected[ball_free]).max()
            )
    gravity_values = np.asarray(gravity, dtype=np.float64)
    if gravity_values.size != gravity_frozen.size:
        raise RuntimeError("frozen qfrc_gravcomp length mismatch")
    diff = np.abs(gravity_values - gravity_frozen.astype(np.float64))
    report["gravity"] = {
        "compared": int(gravity_values.size),
        "max_abs": float(diff.max()),
        "frozen_f32_bits_differing": int(
            np.count_nonzero(gravity_values.astype(np.float32) != gravity_frozen)
        ),
    }
    return report


def xaxis_excluded_indices(kin: dict) -> list[int]:
    """Flat indices of free/ball xaxis entries (no native single-axis field)."""
    joints = list(kin["jnt_type"])
    worlds = int(kin["worlds"])
    excluded = []
    for world in range(worlds):
        for joint, kind in enumerate(joints):
            if kind in XAXIS_NATIVE_JOINT_TYPES:
                continue
            for component in range(3):
                excluded.append(world * len(joints) * 3 + joint * 3 + component)
    return excluded


def assert_equivalence(report: dict) -> None:
    limits = {
        "xpos": 1e-5,
        "xquat_wxyz": 1e-5,
        "xipos": 1e-5,
        "xmat": 1e-5,
        "xanchor": 1e-5,
        "xaxis": 1e-5,
        "subtree_com": 1e-5,
        "cdof": 1e-5,
    }
    for name, limit in limits.items():
        if report["fields"][name]["max_abs"] > limit:
            raise RuntimeError(
                f"native reference does not match frozen scenario: {name} "
                f"max_abs={report['fields'][name]['max_abs']:.3e} > {limit:.0e}"
            )
    if report["gravity"]["max_abs"] > 1e-3:
        raise RuntimeError(
            "native qfrc_gravcomp does not match frozen scenario: "
            f"max_abs={report['gravity']['max_abs']:.3e}"
        )


def main() -> int:
    if sys.prefix != str(ROOT / "target" / "mjwarp-parallel" / "venv"):
        raise RuntimeError(f"not the pinned authoring venv: {sys.prefix}")
    import platform

    versions = {
        "python": platform.python_version(),
        "mujoco": mujoco.__version__,
        "numpy": np.__version__,
    }
    for name, expected in EXPECTED_VERSIONS.items():
        if versions[name] != expected:
            raise RuntimeError(f"version mismatch {name}={versions[name]}")
    kin = load(KIN / "input.json")
    package = load(CHAIN_PACKAGE)
    model_package = load(MODEL_PACKAGE)
    identity = validate_frozen_inputs(kin, package, model_package)

    nbody = int(kin["nbody"])
    mass = f32_values(model_package["model"]["body_mass"][:nbody])
    gravcomp = f32_values(model_package["model"]["body_gravcomp"][:nbody])
    gravity = f32_values(model_package["model"]["opt_gravity"][:3])
    model = mujoco.MjModel.from_xml_string(build_xml(kin, mass, gravcomp))
    model.opt.gravity[:] = np.asarray(gravity, dtype=np.float64)
    if [int(v) for v in model.dof_parentid] != dof_parentid_layout(kin):
        raise RuntimeError("native dof_parentid does not match the frozen layout")
    if int(model.nq) != 13 or int(model.nv) != 11 or int(model.nbody) != 5:
        raise RuntimeError("native model dimensions mismatch")

    data = mujoco.MjData(model)
    native, gravity_values, bias_values = native_fields(kin, model, data)
    second, second_gravity, _ = native_fields(kin, model, data)
    for name in FIELD_ORDER:
        if not np.array_equal(np.asarray(native[name]), np.asarray(second[name])):
            raise RuntimeError(f"non-deterministic native output: {name}")
    if not np.array_equal(np.asarray(gravity_values), np.asarray(second_gravity)):
        raise RuntimeError("non-deterministic native qfrc_gravcomp")
    for name in FIELD_ORDER:
        values = np.asarray(native[name], dtype=np.float64)
        if not np.isfinite(values).all():
            raise RuntimeError(f"non-finite native output: {name}")
    if not np.isfinite(np.asarray(gravity_values)).all():
        raise RuntimeError("non-finite native qfrc_gravcomp")

    equivalence = compare_against_frozen(native, gravity_values, kin)
    assert_equivalence(equivalence)
    excluded = xaxis_excluded_indices(kin)

    fields = {}
    for name in FIELD_ORDER:
        values = np.asarray(native[name], dtype=np.float64)
        quantized = values.astype(np.float32)
        entry = {
            "count": int(values.size),
            "f64": values.tolist(),
            "f32": quantized.astype(np.float64).tolist(),
            "f32_bits": bits_hex(quantized),
        }
        if name == "xaxis":
            entry["native_equivalent"] = {
                "joint_types": ["slide", "hinge"],
                "included_flat_indices": [
                    index for index in range(values.size) if index not in set(excluded)
                ],
                "excluded_flat_indices": excluded,
                "excluded_count": len(excluded),
                "note": "free/ball joints have no single native Cartesian axis; "
                "MuJoCo ignores jnt_axis for ball, so those entries are not the same field",
            }
        fields[name] = entry
    gravity_values = np.asarray(gravity_values, dtype=np.float64)
    bias_values = np.asarray(bias_values, dtype=np.float64)

    reference = {
        "schema": 1,
        "task_id": TASK_ID,
        "kind": "native-mujoco-f64-cpu-reference",
        "scenario": "u038-3d-free-ball-hinge-slide-7worlds",
        "provenance": {
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "frozen_files": identity["files"],
            "parameter_periods": identity["periods"],
            "model_source": "frozen three-d-kinematics/input.json + three-d-model-package.json",
            "generator": "scripts/authoring/author-u038-native.py",
        },
        "environment": {
            "python": versions["python"],
            "python_executable": sys.executable,
            "numpy": versions["numpy"],
            "mujoco": versions["mujoco"],
            "native_version": int(mujoco.mj_version()),
            "platform": platform.platform(),
        },
        "quantization": {
            "inputs": "f32 (numpy.float32) then widened to f64; all model parameters and qpos",
            "outputs": "f64 raw (primary, matches tests/support/two_reference.rs Reference.values) "
            "plus f32 round-to-nearest comparison values (diagnostic)",
            "gravity_switch": "model.opt.gravity = frozen f32 opt_gravity; qvel=0; mj_forward",
        },
        "qfrc_gravcomp_mapping": {
            "native_field": "mjData.qfrc_gravcomp",
            "definition": "generalized antigravity force sum_b J_b^T (-body_gravcomp_b * mass_b * gravity)",
            "qfrc_bias_note": "mjData.qfrc_bias at qvel=0 is the gravity generalized force without "
            "body_gravcomp scaling; qfrc_gravcomp != -qfrc_bias unless every body_gravcomp=1",
            "auxiliary_qfrc_bias_f64": bias_values.tolist(),
        },
        "fields": fields,
        "gravity": {
            "name": "qfrc_gravcomp",
            "count": int(gravity_values.size),
            "f64": gravity_values.tolist(),
            "f32": gravity_values.astype(np.float32).astype(np.float64).tolist(),
            "f32_bits": bits_hex(gravity_values.astype(np.float32)),
        },
        "deterministic_runs": 2,
        "native_equivalent_scenario_check": equivalence,
    }
    write_json(OUTPUT / "native-reference.json", reference)
    authoring = {
        "schema": 1,
        "task_id": TASK_ID,
        "kind": "native-reference-authoring",
        "versions": versions,
        "native_version": int(mujoco.mj_version()),
        "python_prefix": sys.prefix,
        "python_executable_sha256": digest(pathlib.Path(sys.executable)),
        "generator_sha256": digest(pathlib.Path(__file__).resolve()),
        "frozen_files": identity["files"],
        "parameter_periods": identity["periods"],
        "deterministic_runs": 2,
        "finite_outputs": True,
        "native_equivalent_scenario_check": equivalence,
        "xaxis_excluded_flat_indices": excluded,
        "emit": "native-reference.json",
        "authoring_only": True,
        "product_python_dependency": False,
        "judged_inputs": {
            "native": "native-reference.json (generated before reading candidate/secondary)",
            "candidate": "docs/development-evidence/u038-3d/debt-2/product-fields-raw.json + product-gravity-raw.json",
            "secondary": "fixtures/gravity-upstream-abi/three-d-kinematics/raw-bits.json + "
            "docs/development-evidence/u038-3d/debt-2/upstream-gravity-raw.json",
        },
    }
    write_json(OUTPUT / "authoring.json", authoring)
    summary = {
        "native_reference": "ok",
        "output": (OUTPUT / "native-reference.json").as_posix(),
        "equivalence_max_abs": {
            name: equivalence["fields"][name]["max_abs"] for name in FIELD_ORDER
        },
        "gravity_equivalence_max_abs": equivalence["gravity"]["max_abs"],
    }
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
