"""Offline authoring: U038 independent fluid / multi-body atomic reference.

OFFLINE ONLY. Not imported/built/run by product tests. Product tests read the
static fixtures; they never start Python.
"""

from __future__ import annotations

import json
import pathlib
import sys

import numpy as np

import u038_reference_common as common
import u038_reference_chain as chain
import u038_reference_scenarios as scenarios
import u038_reference_validate as validate


def main() -> int:
    env = common.check_pinned_env()
    wp, smooth, passive = common.bootstrap_warp()
    common.OUT_ROOT.mkdir(parents=True, exist_ok=True)
    print(json.dumps({"pinned_env": env}, indent=2))

    # ---------- fluid ----------
    config = scenarios.fluid_config()
    native = common.build_native_model(config)
    topology = scenarios.validate_topology(config, native)
    config = scenarios.finalize_config(config, native)
    qpos = np.asarray(config["qpos"], dtype=np.float32)
    qvel = np.asarray(config["qvel"], dtype=np.float32)
    first = chain.run_kinematics_chain(wp, smooth, config, native, qpos, qvel)
    second = chain.run_kinematics_chain(wp, smooth, config, native, qpos, qvel)
    for name in first:
        if not np.array_equal(first[name], second[name], equal_nan=True):
            raise RuntimeError(f"non-deterministic upstream chain output: {name}")
    fluid_runs = [chain.launch_fluid(wp, passive, config, first) for _ in range(2)]
    fluid = fluid_runs[0]
    if not np.array_equal(fluid_runs[0], fluid_runs[1]):
        raise RuntimeError("non-deterministic frozen fluid device output")
    expected = validate.fluid_force_f64(config, first)
    delta = np.abs(fluid.astype(np.float64) - expected)
    zero_bodies = [
        body
        for body in range(config["nbody"])
        if body == 0 or config["body_mass"][0][body] < common.MJ_MINVAL
    ]
    for body in zero_bodies:
        if np.any(fluid[:, body, :] != 0.0):
            raise RuntimeError(f"expected exact zero fluid force for body {body}")
    if float(delta.max()) > 1.0e-3:
        raise RuntimeError(f"fluid device output deviates from independent f64 recompute: {delta.max()}")
    # zero fluid parameters must produce exact zeros
    zero_config = dict(config)
    zero_config["wind"] = [[0.0, 0.0, 0.0]]
    zero_config["density"] = [0.0]
    zero_config["viscosity"] = [0.0]
    zero_output = chain.launch_fluid(wp, passive, zero_config, first)
    if np.any(zero_output != 0.0):
        raise RuntimeError("zero fluid parameters did not produce exact zero output")
    print(json.dumps({"fluid": {"worlds": config["worlds"], "nbody": config["nbody"],
                                "max_abs_vs_f64": float(delta.max()),
                                "compared_slots": int(fluid.size)}}, indent=2))

    # ---------- atomic ----------
    atomic = scenarios.atomic_config()
    atomic_native = common.build_native_model(atomic)
    atomic_topology = scenarios.validate_topology(atomic, atomic_native)
    atomic = scenarios.finalize_config(atomic, atomic_native)
    atomic_qpos = np.asarray(atomic["qpos"], dtype=np.float32)
    atomic_qvel = np.asarray(atomic["qvel"], dtype=np.float32)
    atomic_chain = chain.run_kinematics_chain(wp, smooth, atomic, atomic_native, atomic_qpos, atomic_qvel)
    atomic_chain_again = chain.run_kinematics_chain(
        wp, smooth, atomic, atomic_native, atomic_qpos, atomic_qvel
    )
    for name in atomic_chain:
        if not np.array_equal(atomic_chain[name], atomic_chain_again[name], equal_nan=True):
            raise RuntimeError(f"non-deterministic upstream chain output: {name}")
    gravity_runs = [
        chain.launch_gravity(wp, passive, atomic, atomic_chain) for _ in range(8)
    ]
    gravity_expected = validate.gravity_force_f64(atomic, atomic_chain)
    gravity_delta = np.abs(gravity_runs[0].astype(np.float64) - gravity_expected)
    if float(gravity_delta.max()) > 1.0e-3:
        raise RuntimeError(
            f"gravity device output deviates from independent f64 recompute: {gravity_delta.max()}"
        )
    print(json.dumps({"atomic": {"worlds": atomic["worlds"], "nv": atomic["nv"],
                                 "max_abs_vs_f64": float(gravity_delta.max()),
                                 "variants": len(validate.report_variant_runs(gravity_runs))}}, indent=2))

    # ---------- fixtures ----------
    for name, scenario, native_topology, intermediates in (
        ("fluid", config, topology, first),
        ("atomic", atomic, atomic_topology, atomic_chain),
    ):
        directory = common.OUT_ROOT / name
        directory.mkdir(parents=True, exist_ok=True)
        outputs = fluid_runs if name == "fluid" else gravity_runs
        field = "fluid_applied" if name == "fluid" else "qfrc_gravcomp"
        scenario_payload = {key: value for key, value in scenario.items() if key not in (
            "body_rootid", "body_geomnum", "body_geomadr", "dof_bodyid", "body_isdofancestor",
            "body_branches", "body_branch_start", "body_tree_levels",
        )}
        common.write_json(directory / "scenario.json", scenario_payload)
        intermediate = {
            "schema": 1,
            "provenance": "frozen_upstream_smooth_chain_device_readback",
            "fields": [common.float_payload(key, value) for key, value in intermediates.items()],
        }
        common.write_json(directory / "intermediate.json", intermediate)
        output = {
            "schema": 1,
            "evidence_kind": "frozen-upstream-device-output",
            "upstream_revision": common.REVISION,
            "warp_version": common.WARP_VERSION,
            "entry": common.FLUID_ENTRY if name == "fluid" else common.GRAVITY_ENTRY,
            "field": field,
            "worlds": scenario["worlds"],
            "count": int(outputs[0].size),
            "values": [float(v) for v in outputs[0].reshape(-1)],
            "bits": common.bits_of(outputs[0]),
            "run_variants": [
                {"runs": variant["runs"], "bits": variant["bits"]}
                for variant in validate.report_variant_runs(outputs)
            ],
            "deterministic_runs": len(validate.report_variant_runs(outputs)) == 1,
            "authoring_only": True,
            "product_python_dependency": False,
        }
        common.write_json(directory / "output.json", output)
        natural = {
            "schema": 1,
            "evidence_kind": "u038-reference-natural-validation",
            "upstream_revision": common.REVISION,
            "warp_version": common.WARP_VERSION,
            "topology": native_topology,
            "independent_f64_recompute_max_abs": float(
                delta.max() if name == "fluid" else gravity_delta.max()
            ),
            "frozen_artifact_hashes": {
                "smooth_cu_sha256": common.SMOOTH_CU_SHA256,
                "smooth_meta_sha256": common.SMOOTH_META_SHA256,
                "smooth_ptx_sha256": common.SMOOTH_PTX_SHA256,
                "passive_cu_sha256": common.PASSIVE_CU_SHA256,
                "passive_meta_sha256": common.PASSIVE_META_SHA256,
                "passive_ptx_sha256": common.PASSIVE_PTX_SHA256,
            },
            "cache_mode": "frozen_artifacts_seeded_then_loaded_cached",
            "deterministic_chain_runs": 2,
            "deterministic_output_runs": len(validate.report_variant_runs(outputs)) == 1,
            "zero_parameter_output_exact_zero": name == "fluid",
            "authoring_only": True,
            "product_python_dependency": False,
        }
        common.write_json(directory / "natural-validation.json", natural)
        modules = [
            pathlib.Path(__file__),
            pathlib.Path(common.__file__),
            pathlib.Path(chain.__file__),
            pathlib.Path(scenarios.__file__),
            pathlib.Path(validate.__file__),
        ]
        for source in modules:
            (directory / source.name).write_bytes(source.read_bytes())
        authoring = {
            "schema": 1,
            "upstream_revision": common.REVISION,
            "warp_version": common.WARP_VERSION,
            "python_version": sys.version,
            "python_executable_sha256": common.digest(pathlib.Path(sys.executable)),
            "device": str(wp.get_device("cuda:0")),
            "scenario": name,
            "entry": common.FLUID_ENTRY if name == "fluid" else common.GRAVITY_ENTRY,
            "generator_sha256": common.digest(pathlib.Path(__file__)),
            "common_sha256": common.digest(pathlib.Path(common.__file__)),
            "chain_sha256": common.digest(pathlib.Path(chain.__file__)),
            "scenarios_sha256": common.digest(pathlib.Path(scenarios.__file__)),
            "validate_sha256": common.digest(pathlib.Path(validate.__file__)),
            "scenario_sha256": common.digest(directory / "scenario.json"),
            "intermediate_sha256": common.digest(directory / "intermediate.json"),
            "output_sha256": common.digest(directory / "output.json"),
            "natural_validation_sha256": common.digest(directory / "natural-validation.json"),
            "authoring_only": True,
            "product_python_dependency": False,
        }
        common.write_json(directory / "authoring.json", authoring)
        receipt = {
            "schema": 1,
            "evidence_kind": "u038-reference-frozen-upstream-device",
            "upstream_revision": common.REVISION,
            "warp_version": common.WARP_VERSION,
            "module": "mujoco_warp._src.smooth + mujoco_warp._src.passive",
            "entry": authoring["entry"],
            "files": {
                name: common.digest(directory / name)
                for name in (
                    "scenario.json",
                    "intermediate.json",
                    "output.json",
                    "natural-validation.json",
                    "authoring.json",
                )
            },
            "authoring_only": True,
            "product_python_dependency": False,
        }
        common.write_json(directory / "receipt.json", receipt)
        print(json.dumps({"scenario": name, "directory": str(directory)}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
