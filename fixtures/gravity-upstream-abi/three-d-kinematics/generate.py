"""Offline authoring: frozen upstream U038 3D rotation / combined-state reference.

OFFLINE ONLY. Not imported/built/run by product tests or scripts. Product tests
read the static fixtures; they never start Python.

Scenario: 5 bodies, 2 branches, mixed joints free+ball+hinge+slide
(nq=13, nv=11), 7 worlds with nonzero quaternion rotations and 3D body rotations.
Generates upstream kinematics/com/cdof outputs with the pinned Warp cache and
writes: fixtures/gravity-upstream-abi/three-d-kinematics/*, a 3D model package and
a product-chain package. Reuses the frozen upstream gravity module artifact.
The fixture `body_isdofancestor` follows the pinned upstream `put_model` semantics
(DOFs that affect each body) and is validated bit-exactly against
`mujoco_warp.put_model` on an equivalent native model (`natural-validation.json`).
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys

import numpy as np

import u038_3d_common
import u038_3d_config
import u038_3d_reference
from u038_3d_common import *
from u038_3d_config import *
from u038_3d_reference import *

def main() -> int:
    versions = {
        "python": sys.version.split()[0],
        "warp": None,
        "mujoco": None,
        "numpy": np.__version__,
        "mujoco-warp": None,
    }
    revision = subprocess.check_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        raise RuntimeError(f"upstream revision mismatch: {revision}")
    if subprocess.check_output(["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True).strip():
        raise RuntimeError("upstream checkout is dirty")
    if digest(UPSTREAM / "mujoco_warp" / "_src" / "smooth.py") != SMOOTH_SHA256:
        raise RuntimeError("smooth.py hash mismatch")
    if digest(UPSTREAM / "mujoco_warp" / "_src" / "types.py") != TYPES_SHA256:
        raise RuntimeError("types.py hash mismatch")
    stem = "wp_mujoco_warp._src.smooth_ca013d6"
    cache_files = {
        "smooth.cu": CACHE / f"{stem}.cu",
        "smooth.meta": CACHE / f"{stem}.meta",
        "smooth.ptx": CACHE / f"{stem}.sm89.ptx",
    }
    for name, expected in [("smooth.cu", CU_SHA256), ("smooth.meta", META_SHA256), ("smooth.ptx", PTX_SHA256)]:
        if digest(cache_files[name]) != expected:
            raise RuntimeError(f"{name} hash mismatch")

    sys.path.insert(0, str(UPSTREAM))
    import warp as wp

    if wp.__version__ != EXPECTED_VERSIONS["warp"]:
        raise RuntimeError(f"unexpected Warp version: {wp.__version__}")
    versions["warp"] = wp.__version__
    wp.config.kernel_cache_dir = str(CACHE_ROOT)
    wp.init()
    import mujoco_warp._src.smooth as smooth

    device = "cuda:0"
    wp.load_module(module=wp.get_module(smooth.__name__), device=device)

    config = build_config()
    values = run_upstream(config)
    second = run_upstream(config)
    for name, field in values.items():
        if not np.array_equal(field, second[name], equal_nan=True):
            raise RuntimeError(f"non-deterministic upstream output: {name}")
    if any(not np.isfinite(field).all() for field in values.values()):
        raise RuntimeError("non-finite upstream output")

    ancestor = body_isdofancestor_rows(config)
    native_ancestor, native_provenance = native_isdofancestor(config)
    if ancestor != native_ancestor:
        raise RuntimeError("body_isdofancestor does not match upstream put_model semantics")
    matrix_sha = digest_text(json.dumps(ancestor, separators=(",", ":")))

    HERE.mkdir(parents=True, exist_ok=True)
    input_path = HERE / "input.json"
    write_json(input_path, {k: v for k, v in config.items() if k != "extra_authoring"})
    raw = {
        "schema": 1,
        "provenance": "direct_upstream_device_readback_before_finite_validation",
        "upstream_revision": revision,
        "fields": [float_payload(name, field) for name, field in values.items()],
    }
    write_json(HERE / "raw-bits.json", raw)
    write_json(HERE / "raw-readback.json", raw)
    write_json(
        HERE / "state.json",
        {
            "schema": 1,
            "input_sha256": digest(input_path),
            "initial_outputs": "nan_except_world_body_zero_and_zero_accumulators",
            "raw_readback_preceded_finite_validation": True,
            "finite_validation_performed": False,
        },
    )
    output = {
        "schema": 1,
        "evidence_kind": "frozen-upstream-generated-3d-qpos-kinematics-com-cdof",
        "upstream_revision": revision,
        "warp_version": wp.__version__,
        "input_sha256": digest(input_path),
        "worlds": WORLDS,
        "nbody": NBODY,
        "njnt": NJNT,
        "nq": NQ,
        "nv": NV,
        **{name: [float(v) for v in field.reshape(-1)] for name, field in values.items()},
        "all_qpos_nonzero": True,
        "nonzero_rotation_coverage": True,
        "three_d_rotation_coverage": True,
        "candidate_kinematics_inputs": False,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    write_json(HERE / "output.json", output)
    natural = {
        "schema": 1,
        "evidence_kind": "body_isdofancestor-vs-upstream-put-model",
        "upstream_revision": revision,
        "warp_version": wp.__version__,
        "mujoco_version": native_provenance["mujoco"],
        "mujoco_warp_version": native_provenance["mujoco_warp"],
        "converter": native_provenance["converter"],
        "upstream_io_source_sha256": native_provenance["io_source_sha256"],
        "source_anchor": "mujoco_warp/_src/io.py: precompute body_isdofancestor (dof_parentid chain)",
        "dof_parentid": native_provenance["dof_parentid"],
        "rows": config["nbody"],
        "columns": config["nv"],
        "matrix": ancestor,
        "matrix_sha256": matrix_sha,
        "exact_match": True,
        "authoring_only": True,
        "product_python_dependency": False,
    }
    write_json(HERE / "natural-validation.json", natural)
    generator = HERE / "generate.py"
    generator.write_bytes(pathlib.Path(__file__).read_bytes())
    generator_modules = {}
    for module in (u038_3d_common, u038_3d_config, u038_3d_reference):
        source = pathlib.Path(module.__file__)
        (HERE / source.name).write_bytes(source.read_bytes())
        generator_modules[source.name] = digest(source)
    authoring = {
        "schema": 1,
        "upstream_revision": revision,
        "warp_version": wp.__version__,
        "python_version": sys.version,
        "python_executable_sha256": digest(pathlib.Path(sys.executable)),
        "device": str(wp.get_device(device)),
        "smooth_source_sha256": SMOOTH_SHA256,
        "types_source_sha256": TYPES_SHA256,
        "input_sha256": digest(input_path),
        "output_sha256": digest(HERE / "output.json"),
        "raw_bits_sha256": digest(HERE / "raw-bits.json"),
        "state_sha256": digest(HERE / "state.json"),
        "kernels": KERNELS,
        "scenario": "3d-free-ball-hinge-slide",
        "worlds": WORLDS,
        "nq": NQ,
        "nv": NV,
        "all_qpos_nonzero": True,
        "nonzero_rotation_coverage": True,
        "three_d_rotation_coverage": True,
        "deterministic_runs": 2,
        "generator_modules": generator_modules,
        "natural_validation": {
            "file": "natural-validation.json",
            "sha256": digest(HERE / "natural-validation.json"),
            "body_isdofancestor_sha256": matrix_sha,
            "exact_match": True,
        },
        "authoring_only": True,
        "product_python_dependency": False,
    }
    write_json(HERE / "authoring.json", authoring)
    receipt_files = {
        name: digest(HERE / name)
        for name in (
            "input.json",
            "output.json",
            "raw-bits.json",
            "raw-readback.json",
            "state.json",
            "natural-validation.json",
            "authoring.json",
            "generate.py",
            "u038_3d_common.py",
            "u038_3d_config.py",
            "u038_3d_reference.py",
        )
    }
    write_json(
        HERE / "receipt.json",
        {
            "schema": 1,
            "evidence_kind": "frozen-upstream-generated-3d-qpos-kinematics-com-cdof",
            "upstream_revision": revision,
            "warp_version": wp.__version__,
            "module": "mujoco_warp._src.smooth",
            "smooth_source_sha256": SMOOTH_SHA256,
            "types_source_sha256": TYPES_SHA256,
            "target_arch": 89,
            "mode": "release",
            "fast_math": False,
            "fuse_fp": True,
            "compiler_options_provenance": "reused_frozen_generated_module",
            "module_artifact": {
                "directory": "../independent-kinematics",
                "cu_sha256": CU_SHA256,
                "meta_sha256": META_SHA256,
                "ptx_sha256": PTX_SHA256,
            },
            "files": receipt_files,
            "generation_attempts": 2,
            "all_qpos_nonzero": True,
            "nonzero_rotation_coverage": True,
            "three_d_rotation_coverage": True,
            "authoring_only": True,
            "product_python_dependency": False,
        },
    )

    model_package = build_model_package(config, ancestor)
    write_json(ROOT / "fixtures" / "gravity-upstream-abi" / "three-d-model-package.json", model_package)
    build_product_package(config, model_package, output)
    print(json.dumps({"authoring": "ok", "receipt": (HERE / "receipt.json").as_posix()}, indent=2))
    return 0



if __name__ == "__main__":
    raise SystemExit(main())
