"""Offline GPU reference authoring only. Product builds and tests never run Python."""

import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys

import mujoco
import mujoco_warp as mjw
from mujoco_warp._src import collision_driver, collision_flex, collision_gjk
from mujoco_warp._src import collision_primitive, collision_sdf, passive, smooth
from mujoco_warp._src.types import vec8, vec8i
import numpy as np
import warp as wp


ROOT = Path(__file__).resolve().parents[2]
DEST = Path(__file__).resolve().parent
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
RUN_ID = os.environ.get("MJWARP_D04_RUN_ID", "")
if not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,31}", RUN_ID):
    raise RuntimeError("MJWARP_D04_RUN_ID must be a short lowercase run identity")
WORK = ROOT / "target/d04-complex-physics" / RUN_ID
GENERATED = WORK / "generated"

CONTACT_FLOAT_FIELDS = (
    "dist", "pos", "frame", "includemargin", "friction", "solref",
    "solreffriction", "solimp", "adhesion",
)
CONTACT_INT_FIELDS = (
    "dim", "geom", "flex", "elem", "vert", "worldid", "type",
    "geomcollisionid",
)
FLEX_FLOAT_FIELDS = (
    "flexnode_xpos", "flexvert_xpos", "flexedge_length",
    "flexedge_velocity", "flexedge_J", "face_xpos", "face_quat",
    "flexvert_hessian", "flexedge_hessian", "qfrc_spring",
    "qfrc_damper", "qfrc_passive",
)
FLEX_BOOL_FIELDS = ("flex_hessian_valid",)
COVERAGE_CONTRACT = {
    "G07": {
        "broadphase_modes": ["NXN", "SAP_TILE", "SAP_SEGMENTED"],
        "filter_boundaries": ["plane-excluded"],
    },
    "G08": {"primitive_pairs": ["plane-sphere", "sphere-sphere"]},
    "G09": {
        "convex_boundaries": ["convex-mesh", "box-box-multiccd", "degenerate-mesh"]
    },
    "G10": {
        "terrain_sdf_boundaries": ["hfield-box", "sdf-cylinder", "volume-gradient-outside"]
    },
    "G11": {
        "flex_contact_boundaries": ["flex-geom", "flex-self", "flex-coupled"]
    },
    "G22": {
        "flex_dynamics_boundaries": ["rope-1d", "cloth-2d", "softbody-3d", "contact-coupled"]
    },
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(array):
    return hashlib.sha256(np.asarray(array).tobytes()).hexdigest()


def dump(path, value):
    path.write_text(
        json.dumps(value, indent=2, allow_nan=False) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def input_array_record(array):
    value = np.asarray(array, dtype=np.float32)
    return {
        "dtype": "float32",
        "shape": list(value.shape),
        "sha256": digest(value),
        "values": value.reshape(-1).tolist(),
        "bits": value.view(np.uint32).reshape(-1).astype(np.uint64).tolist(),
    }


def device_array_record(array):
    value = array.numpy()
    assert value.dtype in (np.dtype("float32"), np.dtype("int32"), np.dtype("bool"))
    if value.dtype == np.dtype("float32"):
        assert np.isfinite(value).all()
    return {
        "dtype": str(value.dtype),
        "shape": list(value.shape),
        "sha256": digest(value),
        "values": value.reshape(-1).tolist(),
    }


def contact_record(data, minimum=1):
    count = int(data.nacon.numpy()[0])
    assert count >= minimum
    fields = {}
    for name in CONTACT_FLOAT_FIELDS + CONTACT_INT_FIELDS:
        value = getattr(data.contact, name).numpy()[:count]
        expected = np.float32 if name in CONTACT_FLOAT_FIELDS else np.int32
        assert value.dtype == expected, name
        if expected == np.float32:
            assert np.isfinite(value).all(), name
        fields[name] = {
            "dtype": str(value.dtype),
            "shape": list(value.shape),
            "sha256": digest(value),
            "values": value.reshape(-1).tolist(),
        }
    return {"count": count, "fields": fields}


def default_state(host, worlds, offset):
    qpos = np.tile(host.qpos0.astype(np.float32), (worlds, 1))
    qvel = np.zeros((worlds, host.nv), dtype=np.float32)
    for world in range(worlds):
        if host.nq >= 7:
            qpos[world, 0:3] += np.asarray(offset, dtype=np.float32) * world
        if host.nv:
            qvel[world] = np.linspace(
                -0.02 * (world + 1), 0.03 * (world + 1), host.nv,
                dtype=np.float32,
            )
    return qpos, qvel


def collision_case(
    name, families, source, worlds=2, offset=(0.0, 0.0, 0.0), flex=False,
    minimum_contacts=1,
):
    host = mujoco.MjModel.from_xml_path(str(source))
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    qpos, qvel = default_state(host, worlds, offset)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    stages = ["kinematics"]
    if flex:
        mjw.flex(model, data)
        stages.append("flex")
    mjw.collision(model, data)
    stages.append("collision")
    wp.synchronize_device()
    return {
        "name": name,
        "families": families,
        "input": {
            "path": source.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "worlds": worlds,
            "qpos": input_array_record(qpos),
            "qvel": input_array_record(qvel),
        },
        "execution": {
            "stages": stages,
            "broadphase": int(model.opt.broadphase),
            "broadphase_filter": int(model.opt.broadphase_filter),
        },
        "output": contact_record(data, minimum_contacts),
    }


def primitive_cases():
    source = ROOT / "fixtures/collision-reference-input/plane-sphere.xml"
    states_path = ROOT / "fixtures/collision-reference-input/states-f32.json"
    states = json.loads(states_path.read_text(encoding="utf-8"))
    host = mujoco.MjModel.from_xml_path(str(source))
    worlds = len(states)
    qpos = np.asarray([row["qpos"] for row in states], dtype=np.float32)
    qvel = np.asarray([row["qvel"] for row in states], dtype=np.float32)
    all_filter = int(
        mjw.BroadphaseFilter.PLANE | mjw.BroadphaseFilter.SPHERE
        | mjw.BroadphaseFilter.AABB | mjw.BroadphaseFilter.OBB
    )
    modes = (
        ("nxn", int(mjw.BroadphaseType.NXN), all_filter, 1),
        ("sap-tile", int(mjw.BroadphaseType.SAP_TILE), all_filter, 1),
        ("sap-segmented", int(mjw.BroadphaseType.SAP_SEGMENTED), all_filter, 1),
        ("plane-filtered", int(mjw.BroadphaseType.NXN), int(mjw.BroadphaseFilter.SPHERE), 0),
    )
    rows = []
    for label, broadphase, broadphase_filter, minimum in modes:
        model = mjw.put_model(host)
        model.opt.broadphase = broadphase
        model.opt.broadphase_filter = broadphase_filter
        data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
        data.qpos.assign(qpos)
        data.qvel.assign(qvel)
        mjw.kinematics(model, data)
        mjw.collision(model, data)
        wp.synchronize_device()
        output = contact_record(data, minimum)
        if label == "plane-filtered":
            assert output["count"] == 0
        rows.append({
            "name": f"plane-sphere-{label}",
            "families": ["G07", "G08"],
            "input": {
                "path": source.relative_to(ROOT).as_posix(),
                "sha256": sha(source),
                "states_path": states_path.relative_to(ROOT).as_posix(),
                "states_sha256": sha(states_path),
                "worlds": worlds,
                "qpos": input_array_record(qpos),
                "qvel": input_array_record(qvel),
            },
            "execution": {
                "stages": ["kinematics", "collision"],
                "broadphase": broadphase,
                "broadphase_filter": broadphase_filter,
            },
            "output": output,
        })
    return rows


def flex_self_case():
    source = DEST / "flex-self.xml"
    host = mujoco.MjModel.from_xml_path(str(source))
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=2, nconmax=4096, njmax=20000)
    mjw.kinematics(model, data)
    mjw.flex(model, data)
    positions = data.flexvert_xpos.numpy()
    for world in range(2):
        positions[world, 2] = positions[world, 6] + np.asarray(
            [0.0, 0.0, 0.005], dtype=np.float32
        )
    data.flexvert_xpos.assign(positions)
    mjw.collision(model, data)
    wp.synchronize_device()
    return {
        "name": "flex-self",
        "families": ["G07", "G11"],
        "input": {
            "path": source.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "worlds": 2,
            "flexvert_xpos_override": input_array_record(positions),
        },
        "execution": {"stages": ["kinematics", "flex", "collision"]},
        "output": contact_record(data),
    }


def flex_case(name, source):
    host = mujoco.MjModel.from_xml_path(str(source))
    model_overrides = {}
    if name == "rope-edge":
        host.flex_edgestiffness[:] = 100.0
        model_overrides["flex_edgestiffness"] = {
            "native_dtype": str(host.flex_edgestiffness.dtype),
            "values": host.flex_edgestiffness.tolist(),
        }
    model = mjw.put_model(host)
    worlds = 2
    data = mjw.make_data(host, nworld=worlds, nconmax=4096, njmax=20000)
    qpos = np.tile(host.qpos0.astype(np.float32), (worlds, 1))
    qvel = np.zeros((worlds, host.nv), dtype=np.float32)
    if host.nq:
        qpos[0] += np.linspace(-0.012, 0.018, host.nq, dtype=np.float32)
        qpos[1] += np.linspace(0.015, -0.009, host.nq, dtype=np.float32)
    if host.nv:
        qvel[0] = np.linspace(-0.3, 0.2, host.nv, dtype=np.float32)
        qvel[1] = np.linspace(0.25, -0.35, host.nv, dtype=np.float32)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.com_pos(model, data)
    mjw.flex(model, data)
    mjw.com_vel(model, data)
    mjw.passive(model, data)
    wp.synchronize_device()
    fields = {
        field: device_array_record(getattr(data, field))
        for field in FLEX_FLOAT_FIELDS + FLEX_BOOL_FIELDS
    }
    assert np.linalg.norm(data.qfrc_spring.numpy()) > 0.0
    return {
        "name": name,
        "families": ["G22"],
        "input": {
            "path": source.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "worlds": worlds,
            "qpos": input_array_record(qpos),
            "qvel": input_array_record(qvel),
            "model_overrides": model_overrides,
        },
        "execution": {
            "stages": ["kinematics", "com_pos", "flex", "com_vel", "passive"]
        },
        "output": {"fields": fields},
    }


def flex_coupled_case():
    source = DEST / "flex-coupled.xml"
    host = mujoco.MjModel.from_xml_path(str(source))
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=2, nconmax=4096, njmax=20000)
    qpos, qvel = default_state(host, 2, (0.0, 0.0, 0.002))
    if host.nq:
        qpos += np.linspace(-0.004, 0.006, host.nq, dtype=np.float32)
    data.qpos.assign(qpos)
    data.qvel.assign(qvel)
    mjw.kinematics(model, data)
    mjw.com_pos(model, data)
    mjw.flex(model, data)
    mjw.com_vel(model, data)
    mjw.passive(model, data)
    mjw.collision(model, data)
    wp.synchronize_device()
    fields = {
        field: device_array_record(getattr(data, field))
        for field in FLEX_FLOAT_FIELDS + FLEX_BOOL_FIELDS
    }
    return {
        "name": "contact-coupled",
        "families": ["G11", "G22"],
        "input": {
            "path": source.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "worlds": 2,
            "qpos": input_array_record(qpos),
            "qvel": input_array_record(qvel),
        },
        "execution": {
            "stages": ["kinematics", "com_pos", "flex", "com_vel", "passive", "collision"]
        },
        "output": {"contacts": contact_record(data), "fields": fields},
    }


@wp.kernel
def volume_probe_kernel(
    oct_child: wp.array(dtype=vec8i),
    oct_aabb: wp.array2d(dtype=wp.vec3),
    oct_coeff: wp.array(dtype=vec8),
    root: int,
    points: wp.array(dtype=wp.vec3),
    distances: wp.array(dtype=float),
    gradients: wp.array(dtype=wp.vec3),
):
    tid = wp.tid()
    volume = collision_sdf.VolumeData()
    volume.center = oct_aabb[root, 0]
    volume.half_size = oct_aabb[root, 1]
    volume.root = root
    volume.oct_aabb = oct_aabb
    volume.oct_child = oct_child
    volume.oct_coeff = oct_coeff
    volume.valid = True
    distances[tid] = collision_sdf.sample_volume_sdf(points[tid], volume)
    gradients[tid] = collision_sdf.sample_volume_grad(points[tid], volume)


def volume_probe_case():
    source = DEST / "sdf-cylinder.xml"
    host = mujoco.MjModel.from_xml_path(str(source))
    model = mjw.put_model(host)
    root = int(model.mesh_octadr.numpy()[0])
    points_np = np.asarray(
        [[0.0, 0.0, 0.0], [0.25, -0.1, 0.15], [2.0, 0.0, 0.0]],
        dtype=np.float32,
    )
    points = wp.array(points_np, dtype=wp.vec3)
    distances = wp.empty(len(points_np), dtype=float)
    gradients = wp.empty(len(points_np), dtype=wp.vec3)
    wp.launch(
        volume_probe_kernel,
        dim=len(points_np),
        inputs=[model.oct_child, model.oct_aabb, model.oct_coeff, root, points],
        outputs=[distances, gradients],
    )
    wp.synchronize_device()
    return {
        "name": "volume-gradient-outside",
        "families": ["G10"],
        "input": {
            "path": source.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "points": input_array_record(points_np),
        },
        "execution": {"stages": ["sample_volume_sdf", "sample_volume_grad"]},
        "output": {
            "distance": device_array_record(distances),
            "gradient": device_array_record(gradients),
        },
    }


def collect_inputs(cases):
    inputs = {}
    for case in cases:
        data = case["input"]
        inputs[data["path"]] = data["sha256"]
        if "states_path" in data:
            inputs[data["states_path"]] = data["states_sha256"]
    return dict(sorted(inputs.items()))


def snapshot_intake():
    if WORK.exists():
        raise RuntimeError(f"run directory already exists: {WORK}")
    intake = WORK / "intake"
    intake.mkdir(parents=True)
    paths = [Path(__file__)] + sorted(DEST.glob("*.xml")) + [
        ROOT / "fixtures/collision-reference-input/plane-sphere.xml",
        ROOT / "fixtures/collision-reference-input/states-f32.json",
    ]
    inventory = {}
    for path in paths:
        rel = path.relative_to(ROOT).as_posix()
        target = intake / rel.replace("/", "__")
        shutil.copyfile(path, target)
        inventory[rel] = sha(path)
    dump(intake / "inventory.json", inventory)
    GENERATED.mkdir()


def main():
    assert (mujoco.__version__, wp.__version__, np.__version__) == (
        "3.15.0", "1.15.0", "2.5.4"
    )
    head = subprocess.check_output(
        ["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True
    ).strip()
    assert head == REVISION
    assert not subprocess.check_output(
        ["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True
    ).strip()
    installed = Path(mjw.__file__).parent
    sources = {}
    installed_sources = {}
    for path in sorted((UPSTREAM / "mujoco_warp").rglob("*.py")):
        rel = path.relative_to(UPSTREAM / "mujoco_warp")
        upstream_hash = sha(path)
        installed_hash = sha(installed / rel)
        assert upstream_hash == installed_hash, str(rel)
        sources[rel.as_posix()] = upstream_hash
        installed_sources[rel.as_posix()] = installed_hash

    snapshot_intake()
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")

    collision_cases = primitive_cases() + [
        collision_case(
            "sphere-sphere", ["G07", "G08"], DEST / "sphere-sphere.xml",
            offset=(0.005, 0.0, 0.0),
        ),
        collision_case(
            "convex-mesh", ["G07", "G09"], DEST / "convex-mesh.xml",
            offset=(0.01, -0.01, 0.0),
        ),
        collision_case(
            "box-box-multiccd", ["G07", "G09"], DEST / "box-box.xml",
            offset=(0.005, -0.005, 0.0), minimum_contacts=2,
        ),
        collision_case(
            "degenerate-mesh", ["G07", "G09"], DEST / "degenerate-mesh.xml",
        ),
        collision_case(
            "hfield-box", ["G07", "G10"], DEST / "hfield-box.xml",
            offset=(0.02, 0.0, -0.01),
        ),
        collision_case(
            "sdf-cylinder", ["G07", "G10"], DEST / "sdf-cylinder.xml",
            offset=(0.0, 0.0, -0.04),
        ),
        collision_case(
            "flex-geom", ["G07", "G09", "G11"], DEST / "flex-contact.xml",
            flex=True,
        ),
        flex_self_case(),
    ]
    flex_cases = [
        flex_case("rope-edge", DEST / "flex-rope.xml"),
        flex_case("cloth-stretch-bend", DEST / "flex-cloth.xml"),
        flex_case("softbody-3d", DEST / "flex-softbody.xml"),
        flex_coupled_case(),
    ]
    probes = [volume_probe_case()]
    all_cases = collision_cases + flex_cases + probes
    reference = {
        "schema": 1,
        "upstream_revision": REVISION,
        "scalar_policy": {
            "inputs": "f32",
            "floating_outputs": "f32",
            "integer_outputs": "i32",
            "boolean_outputs": "bool",
        },
        "coverage_contract": COVERAGE_CONTRACT,
        "collision_cases": collision_cases,
        "flex_cases": flex_cases,
        "probes": probes,
    }
    reference_path = GENERATED / "reference.json"
    dump(reference_path, reference)

    modules = {
        "collision_driver": collision_driver,
        "collision_flex": collision_flex,
        "collision_gjk": collision_gjk,
        "collision_primitive": collision_primitive,
        "collision_sdf": collision_sdf,
        "passive": passive,
        "smooth": smooth,
    }
    options = {name: wp.get_module_options(module) for name, module in modules.items()}
    for value in options.values():
        assert value["fuse_fp"] and not value["fast_math"]
    cache = {
        path.relative_to(WORK).as_posix(): sha(path)
        for path in sorted((WORK / "warp-cache").rglob("*"))
        if path.is_file()
    }
    assert cache
    device = wp.get_device()
    manifest = {
        "schema": 1,
        "scope": "D04 upstream GPU reference samples; not product implementation",
        "run_id": RUN_ID,
        "upstream_revision": REVISION,
        "packages": {
            package: importlib.metadata.version(package)
            for package in ("mujoco-warp", "mujoco", "warp-lang", "numpy")
        },
        "python": sys.version,
        "platform": platform.platform(),
        "gpu": subprocess.check_output(
            [
                "nvidia-smi", "--query-gpu=name,uuid,driver_version,memory.total",
                "--format=csv,noheader",
            ], text=True,
        ).strip(),
        "cuda_arch": device.arch,
        "module_options": options,
        "inputs": collect_inputs(all_cases),
        "source_hashes": sources,
        "installed_source_root": str(installed),
        "installed_source_hashes": installed_sources,
        "cache_hashes": cache,
        "reference_sha256": sha(reference_path),
        "authoring_script_sha256": sha(Path(__file__)),
        "coverage_contract": COVERAGE_CONTRACT,
        "coverage_counts": {
            "collision_cases": len(collision_cases),
            "flex_cases": len(flex_cases),
            "probe_cases": len(probes),
            "contact_float_fields": len(CONTACT_FLOAT_FIELDS),
            "contact_int_fields": len(CONTACT_INT_FIELDS),
            "flex_float_fields": len(FLEX_FLOAT_FIELDS),
            "flex_bool_fields": len(FLEX_BOOL_FIELDS),
        },
        "np01": "Captured inputs and floating outputs are f32; identifiers are i32; validity flags are bool.",
        "np02": "Frozen and imported source hashes match; generated CUDA/PTX hashes and module options are recorded.",
        "np03": "Reference creation only; Rust product equivalence remains unclaimed until matching stages exist.",
        "np04": "Not applicable to reference admission; authoring time is not a fair performance measurement.",
    }
    manifest_path = GENERATED / "manifest.json"
    dump(manifest_path, manifest)
    shutil.copyfile(reference_path, DEST / "reference.json")
    shutil.copyfile(manifest_path, DEST / "manifest.json")
    print("D04_RUN", RUN_ID)
    print("D04_COLLISION_CASES", len(collision_cases))
    print("D04_FLEX_CASES", len(flex_cases))
    print("D04_PROBES", len(probes))
    print("D04_CACHE_FILES", len(cache))


if __name__ == "__main__":
    main()
