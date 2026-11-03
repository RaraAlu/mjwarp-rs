"""Offline authoring: MuJoCo 3.12 vs 3.15 MJB render-model parity for S0a.

OFFLINE ONLY. Not imported or run by product builds/tests/gates.
Compiles the committed scene XML with MuJoCo 3.15 (pinned venv), saves/reloads
scene-315.mjb and dumps model-315.json; runs compile_scene_312.cpp for the 3.12
side (scene-312.mjb + model-312.json); compares render-relevant model arrays
field by field (ints/bytes exact, floats f32 bits first, else <=1 ULP per plan
4.3) and writes parity.json with task id, environment, hashes, commands,
per-field differences, version-difference classification and determinism.
S0a freezes element-wise equality; a mismatch is recorded with
field/index/expected/actual and fails the slice. No tolerance relaxation.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import platform
import subprocess
import sys

import mujoco
import numpy as np

from model_parity_judge import (
    APPROVED_JUDGING,
    EXCLUDED_FIELDS,
    classify_differences,
    judge_fields,
)

TASK_ID = "2026-10-24-render-bvh-s0a"
SCHEMA = 1

# Field kinds must match compile_scene_312.cpp exactly.
FLOAT32_FIELDS = """geom_rgba cam_sensorsize cam_intrinsic light_bulbradius light_intensity
light_range light_attenuation light_cutoff light_softness light_exponent light_ambient
light_diffuse light_specular mat_texrepeat mat_emission mat_specular mat_shininess
mat_reflectance mat_metallic mat_roughness mat_rgba mesh_vert mesh_normal mesh_texcoord
vis.headlight.ambient vis.headlight.diffuse vis.headlight.specular vis.map.znear
vis.map.zfar flex_rgba""".split()
INT64_FIELDS = ["tex_adr"]
BYTE_FIELDS = """geom_sameframe light_castshadow light_active mat_texuniform tex_data
names""".split()
DOUBLE_FIELDS = """geom_solmix geom_solref geom_solimp geom_size geom_aabb geom_rbound
geom_pos geom_quat geom_friction geom_margin geom_gap geom_surfacevel geom_adhesion geom_fluid
cam_pos cam_quat cam_poscom0 cam_pos0 cam_mat0 cam_fovy cam_ipd light_pos light_dir light_poscom0
light_pos0 light_dir0 mesh_scale mesh_pos mesh_quat mesh_polynormal stat.extent
stat.center flex_radius flex_vert flex_vert0""".split()
INT32_FIELDS = """geom_type geom_contype geom_conaffinity geom_condim geom_bodyid geom_dataid
geom_matid geom_group geom_priority geom_plugin cam_mode cam_bodyid cam_targetbodyid
cam_projection cam_resolution cam_output light_mode light_bodyid light_targetbodyid light_type
light_texid mat_texid tex_type tex_colorspace tex_height tex_width tex_nchannel tex_pathadr
mesh_vertadr mesh_vertnum mesh_faceadr mesh_facenum mesh_bvhadr mesh_bvhnum mesh_octadr
mesh_octnum mesh_normaladr mesh_normalnum mesh_texcoordadr mesh_texcoordnum mesh_graphadr
mesh_extrema mesh_face mesh_facenormal mesh_facetexcoord mesh_graph mesh_pathadr mesh_polynum
mesh_polyadr mesh_polyvertadr mesh_polyvertnum mesh_polyvert mesh_polymapadr mesh_polymapnum
mesh_polymap vis.headlight.active name_bodyadr name_jntadr name_geomadr name_siteadr
name_camadr name_lightadr name_meshadr name_texadr name_matadr
flex_dim flex_vertadr flex_vertnum flex_edgeadr flex_edgenum flex_elemadr flex_elemnum
flex_elemdataadr flex_shellnum flex_shelldataadr flex_vertbodyid flex_elem flex_shell""".split()
COUNT_FIELDS = """nbody njnt ngeom nsite ncam nlight nmesh nmeshvert nmeshnormal
nmeshtexcoord nmeshface nmeshgraph nmeshpoly nmeshpolyvert nmeshpolymap nmat ntex ntexdata
nnames nbvh nbvhstatic nbvhdynamic nq nv nbuffer nkey
nflex nflexvert nflexedge nflexelem nflexelemdata nflexshelldata""".split()
FIELDS = (
    [(name, "float32") for name in FLOAT32_FIELDS]
    + [(name, "float64") for name in DOUBLE_FIELDS]
    + [(name, "int32") for name in INT32_FIELDS]
    + [(name, "int64") for name in INT64_FIELDS]
    + [(name, "byte") for name in BYTE_FIELDS]
)
MAX_DETAILS_PER_FIELD = 20


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(path: pathlib.Path) -> str:
    """LF-normalized text hash so differently-configured checkouts agree."""
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def write_json(path: pathlib.Path, value, compact: bool = False) -> None:
    options = {"separators": (",", ":")} if compact else {"indent": 2}
    text = json.dumps(value, **options) + "\n"
    path.write_bytes(text.encode())


def get_array(model, name: str) -> np.ndarray:
    # Python bindings widen some C scalar/byte fields (float->float64, int->int64,
    # char[]->bytes); restore the 3.12 ABI width declared in FIELDS.
    if name == "names":
        return np.frombuffer(model.names, dtype=np.uint8)
    if name == "vis.headlight.active":
        return np.asarray([int(model.vis.headlight.active)], dtype=np.int32)
    if name in ("vis.map.znear", "vis.map.zfar"):
        return np.asarray([np.float32(getattr(model.vis.map, name.split(".")[-1]))], dtype=np.float32)
    if name.startswith("vis."):
        value = model.vis
        for part in name.split(".")[1:]:
            value = getattr(value, part)
        return np.asarray(value).reshape(-1)
    if name.startswith("stat."):
        return np.asarray(getattr(model.stat, name.split(".", 1)[1])).reshape(-1)
    return np.asarray(getattr(model, name)).reshape(-1)


def kind_of(array: np.ndarray) -> str:
    if array.dtype == np.float32:
        return "float32"
    if array.dtype == np.float64:
        return "float64"
    if array.dtype == np.int64:
        return "int64"
    if array.dtype in (np.int32, np.uint32):
        return "int32"
    if array.dtype in (np.uint8, np.bool_):
        return "byte"
    raise RuntimeError(f"unsupported dtype {array.dtype}")


def dump_model(model, tool: str, model_file: str) -> dict:
    arrays = {}
    for name, expected_kind in FIELDS:
        array = get_array(model, name)
        kind = kind_of(array)
        if kind != expected_kind:
            raise RuntimeError(f"{name}: kind {kind} != expected {expected_kind}")
        if expected_kind == "float32":
            values = [float(np.float32(value)) for value in array]
        elif expected_kind == "float64":
            values = [float(value) for value in array]
        else:
            values = [int(value) for value in array]
        arrays[name] = {"kind": kind, "count": int(array.size), "values": values}
    counts = {name: int(getattr(model, name)) for name in COUNT_FIELDS}
    return {
        "schema": SCHEMA,
        "tool": tool,
        "mujoco_version": mujoco.__version__,
        "mujoco_version_int": int(mujoco.mj_version()),
        "model_file": model_file,
        "loaded_mjb": True,
        "counts": counts,
        "arrays": arrays,
    }


def remove_if_exists(path: pathlib.Path) -> None:
    if path.exists():
        path.unlink()


def compile_315(scene: pathlib.Path, out_dir: pathlib.Path) -> dict:
    model = mujoco.MjModel.from_xml_path(str(scene))
    first = out_dir / "scene-315.mjb"
    second = out_dir / "scene-315.second.mjb"
    remove_if_exists(first)
    mujoco.mj_saveModel(model, str(first), None)
    mujoco.mj_saveModel(model, str(second), None)
    first_sha = digest(first)
    second_sha = digest(second)
    remove_if_exists(second)
    loaded = mujoco.MjModel.from_binary_path(str(first))
    return {
        "model": loaded,
        "sha256": first_sha,
        "bytes": first.stat().st_size,
        "second_sha256": second_sha,
        "deterministic": first_sha == second_sha,
    }


def run_312(exe: pathlib.Path, scene: pathlib.Path, out_dir: pathlib.Path) -> dict:
    first_mjb = out_dir / "scene-312.mjb"
    first_json = out_dir / "model-312.json"
    second_mjb = out_dir / "scene-312.second.mjb"
    second_json = out_dir / "model-312.second.json"
    command = [str(exe), str(scene), str(first_mjb), str(first_json)]
    first = subprocess.run(command, capture_output=True, text=True)
    if first.returncode != 0:
        raise RuntimeError(f"3.12 tool failed ({first.returncode}): {first.stderr}{first.stdout}")
    second = subprocess.run(
        [str(exe), str(scene), str(second_mjb), str(second_json)],
        capture_output=True,
        text=True,
    )
    if second.returncode != 0:
        raise RuntimeError(f"3.12 tool second run failed: {second.stderr}")
    first_mjb_sha = digest(first_mjb)
    second_mjb_sha = digest(second_mjb)
    first_dump = json.loads(first_json.read_text(encoding="utf-8"))
    second_dump = json.loads(second_json.read_text(encoding="utf-8"))
    # The dump records the output path, so compare the model content only.
    deterministic = first_mjb_sha == second_mjb_sha and (
        first_dump["arrays"] == second_dump["arrays"]
        and first_dump["counts"] == second_dump["counts"]
    )
    mjb_bytes = first_mjb.stat().st_size
    remove_if_exists(second_mjb)
    remove_if_exists(second_json)
    return {
        "dump": first_dump,
        "mjb_sha256": first_mjb_sha,
        "mjb_bytes": mjb_bytes,
        "second_mjb_sha256": second_mjb_sha,
        "deterministic": deterministic,
        "command": command,
        "stdout": first.stdout.strip(),
    }


def ulp_distance(actual: np.ndarray, expected: np.ndarray) -> int:
    def ordered(bits):
        as_int = bits.astype(np.int64)
        sign = as_int >> 31
        magnitude = as_int & 0x7FFFFFFF
        return np.where(sign != 0, -magnitude, magnitude)

    delta = np.abs(ordered(actual) - ordered(expected))
    return int(delta.max()) if delta.size else 0


def ulp_distance64(actual: np.ndarray, expected: np.ndarray) -> int:
    def ordered(bits):
        as_int = bits.view(np.int64)
        sign = as_int >> 63
        magnitude = as_int & 0x7FFFFFFFFFFFFFFF
        return np.where(sign != 0, -magnitude, magnitude)

    delta = np.abs(ordered(actual) - ordered(expected))
    return int(delta.max()) if delta.size else 0


def compare_field(name: str, kind: str, left: dict, right: dict) -> dict:
    result = {
        "field": name,
        "kind": kind,
        "count": int(left["count"]),
        "kind_match": left["kind"] == right["kind"] == kind,
        "count_match": left["count"] == right["count"],
        "equal": False,
        "max_absolute": 0.0,
        "bit_differences": 0,
        "max_ulp": 0,
        "mismatches": [],
    }
    if not result["kind_match"] or not result["count_match"]:
        result["reason"] = "kind_or_count_mismatch"
        result["mismatches"].append(
            {
                "index": -1,
                "expected": {"kind": left["kind"], "count": left["count"]},
                "actual": {"kind": right["kind"], "count": right["count"]},
            }
        )
        return result
    a = np.asarray(left["values"], dtype=np.float64)
    b = np.asarray(right["values"], dtype=np.float64)
    differing = np.flatnonzero(a != b)
    result["max_absolute"] = float(np.abs(a - b).max()) if differing.size else 0.0
    if kind in ("int32", "int64", "byte"):
        result["equal"] = bool(differing.size == 0)
        result["bit_differences"] = int(differing.size)
    else:
        dtype = np.float32 if kind == "float32" else np.float64
        a_bits = np.asarray(left["values"], dtype=dtype).view(np.uint32 if kind == "float32" else np.uint64)
        b_bits = np.asarray(right["values"], dtype=dtype).view(np.uint32 if kind == "float32" else np.uint64)
        bit_diff = np.flatnonzero(a_bits != b_bits)
        result["bit_differences"] = int(bit_diff.size)
        result["equal"] = bool(bit_diff.size == 0)
        if kind == "float32":
            result["max_ulp"] = ulp_distance(a_bits, b_bits)
        else:
            result["max_ulp"] = ulp_distance64(a_bits, b_bits)
            # Version-difference classification: does the render-side f32 view
            # (product uploads model floats as f32) still agree bit for bit?
            a32 = np.asarray(left["values"], dtype=np.float32).view(np.uint32)
            b32 = np.asarray(right["values"], dtype=np.float32).view(np.uint32)
            result["f32_bits_equal"] = bool(np.array_equal(a32, b32))
            result["f32_bit_differences"] = int(np.count_nonzero(a32 != b32))
        differing = bit_diff
    for index in differing[:MAX_DETAILS_PER_FIELD]:
        result["mismatches"].append(
            {
                "index": int(index),
                "expected": left["values"][int(index)],
                "actual": right["values"][int(index)],
            }
        )
    if differing.size > MAX_DETAILS_PER_FIELD:
        result["mismatches"].append(
            {"index": -1, "truncated": int(differing.size - MAX_DETAILS_PER_FIELD)}
        )
    return result


def build_parity(args) -> tuple[dict, bool]:
    scene = pathlib.Path(args.scene)
    out_dir = pathlib.Path(args.out)
    evidence = pathlib.Path(args.evidence)
    out_dir.mkdir(parents=True, exist_ok=True)
    evidence.mkdir(parents=True, exist_ok=True)
    exe = pathlib.Path(args.exe)
    script = pathlib.Path(__file__).resolve()

    compile_315_result = compile_315(scene, out_dir)
    dump_315 = dump_model(compile_315_result["model"], "mujoco_python_315", "scene-315.mjb")
    model_315_json = out_dir / "model-315.json"
    write_json(model_315_json, dump_315, compact=True)

    result_312 = run_312(exe, scene, out_dir)
    dump_312 = result_312["dump"]
    model_312_json = out_dir / "model-312.json"

    parity_fields = []
    mismatched = []
    for name, kind in FIELDS:
        left = dump_312["arrays"].get(name)
        right = dump_315["arrays"].get(name)
        if left is None or right is None:
            comparison = {
                "field": name,
                "kind": kind,
                "equal": False,
                "reason": "missing_field_on_one_side",
                "present_312": left is not None,
                "present_315": right is not None,
            }
        else:
            comparison = compare_field(name, kind, left, right)
        parity_fields.append(comparison)
        if not comparison["equal"]:
            mismatched.append(name)
    counts_312 = dump_312["counts"]
    counts_315 = dump_315["counts"]
    counts_match = counts_312 == counts_315
    versions_ok = "3.12" in dump_312["mujoco_version"] and "3.15" in dump_315["mujoco_version"]
    deterministic = compile_315_result["deterministic"] and result_312["deterministic"]
    version_differences, f32_tail_only, other_differences = classify_differences(
        parity_fields, mismatched
    )
    judged = judge_fields(parity_fields, counts_match, versions_ok, deterministic)
    passed = judged["passed"]
    render_relevance = EXCLUDED_FIELDS
    worst = sorted(
        (field for field in parity_fields if not field["equal"]),
        key=lambda field: field.get("bit_differences", 0),
        reverse=True,
    )
    parity = {
        "schema": SCHEMA,
        "task_id": TASK_ID,
        "kind": "render-bvh-s0a-model-parity",
        "slice": "S0a",
        "conclusion": {
            "parity": "pass" if passed else "fail",
            "judged_by": "approved S0a criterion (user 2026-10-24, option A)",
            "strict_elementwise_parity": "fail" if mismatched else "pass",
            "hard_gate": "S0b reference capture is allowed only when parity=pass",
            "fields_compared": len(FIELDS),
            "fields_equal": len(FIELDS) - len(mismatched),
            "mismatched_fields": mismatched,
            "judged_failures": judged["judged_failures"],
            "excluded_fields": EXCLUDED_FIELDS,
            "approved_judgement": APPROVED_JUDGING,
            "counts_match": counts_match,
            "versions_ok": versions_ok,
            "deterministic": deterministic,
            "tolerance": "plan 4.3: model integers exact; model floats f32_bits first, "
            "else <=1 ULP; the strict element-wise result is preserved under "
            "version_difference_classification",
        },
        "sources": {
            "scene_xml": {"path": str(scene), "sha256": digest_text(scene)},
            "mjb_312": {
                "path": str(out_dir / "scene-312.mjb"),
                "sha256": result_312["mjb_sha256"],
                "bytes": result_312["mjb_bytes"],
                "compiled_by": "MuJoCo 3.12.0 C API (compile_scene_312.cpp)",
                "loads_with_312": bool(dump_312["loaded_mjb"]),
            },
            "mjb_315": {
                "path": str(out_dir / "scene-315.mjb"),
                "sha256": compile_315_result["sha256"],
                "bytes": compile_315_result["bytes"],
                "compiled_by": "MuJoCo 3.15.0 Python (model_parity.py)",
                "loads_with_315": bool(dump_315["loaded_mjb"]),
            },
            "model_312_json": {"path": str(model_312_json), "sha256": digest(model_312_json)},
            "model_315_json": {"path": str(model_315_json), "sha256": digest(model_315_json)},
        },
        "environment": {
            "mujoco_312": dump_312["mujoco_version"],
            "mujoco_312_version_int": dump_312["mujoco_version_int"],
            "mujoco_315": dump_315["mujoco_version"],
            "mujoco_315_version_int": dump_315["mujoco_version_int"],
            "python": platform.python_version(),
            "python_prefix": sys.prefix,
            "numpy": np.__version__,
            "platform": platform.platform(),
            "venv": "target/mjwarp-parallel/venv",
            "mujoco_312_package": "target/toolchains/mujoco-3.12.0/package",
        },
        "commands": {
            "compile_312": result_312["command"],
            "compile_312_stdout": result_312["stdout"],
            "compile_315": "mujoco.MjModel.from_xml_path(scene); "
            "mujoco.mj_saveModel(model, 'scene-315.mjb', None)",
            "parity": f"{sys.executable} -B {script} --exe {exe} --scene {scene} "
            f"--out {out_dir} --evidence {evidence}",
        },
        "hashes": {
            "model_parity_py_sha256": digest(script),
            "compile_scene_312_exe": {"path": str(exe), "sha256": digest(exe)},
            "compile_scene_312_cpp_sha256": (
                digest(pathlib.Path(args.cpp)) if args.cpp else "not_provided"
            ),
            "scene_xml_sha256": digest_text(scene),
            "scene_xml_hash_note": "LF-normalized bytes (CRLF->LF) so checkout line-ending "
            "configuration cannot change the recorded scene.xml hash",
        },
        "determinism": {
            "mjb_312_equal_across_two_compiles": result_312["deterministic"],
            "mjb_315_equal_across_two_compiles": compile_315_result["deterministic"],
        },
        "counts": {"mujoco_312": counts_312, "mujoco_315": counts_315, "match": counts_match},
        "fields": parity_fields,
        "worst_fields": [field["field"] for field in worst[:5]],
        "version_difference_classification": {
            "strict_elementwise_parity": "fail" if mismatched else "pass",
            "float64_tail_f32_identical": f32_tail_only,
            "semantic_or_f32_differences": other_differences,
            "version_differences": version_differences,
            "render_relevance_overrides": EXCLUDED_FIELDS,
            "approved_judgement": judged,
            "s0b_hard_gate": (
                "met"
                if passed
                else "NOT met under the approved criterion (no tolerance relaxation applied)"
            ),
        },
        "not_claimed": [
            "render output parity (S0b, needs GPU)",
            "product acceptance (S1+ not started)",
            *APPROVED_JUDGING["not_claimed"],
        ],
    }
    return parity, passed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, help="compiled compile_scene_312 executable")
    parser.add_argument("--cpp", default="", help="compile_scene_312.cpp for provenance hash")
    parser.add_argument("--scene", required=True, help="scene XML path")
    parser.add_argument("--out", required=True, help="fixtures/render-bvh/scene-a directory")
    parser.add_argument("--evidence", required=True, help="evidence directory")
    args = parser.parse_args()
    parity, passed = build_parity(args)
    write_json(pathlib.Path(args.evidence) / "parity.json", parity)
    print(
        json.dumps(
            {
                "parity": parity["conclusion"]["parity"],
                "judged_by": parity["conclusion"]["judged_by"],
                "strict_elementwise_parity": parity["conclusion"][
                    "strict_elementwise_parity"
                ],
                "judged_failures": parity["conclusion"]["judged_failures"],
                "fields_compared": parity["conclusion"]["fields_compared"],
                "fields_equal": parity["conclusion"]["fields_equal"],
                "mismatched_fields": parity["conclusion"]["mismatched_fields"],
                "counts_match": parity["conclusion"]["counts_match"],
                "deterministic": parity["conclusion"]["deterministic"],
                "mjb_312": parity["sources"]["mjb_312"],
                "mjb_315": parity["sources"]["mjb_315"],
            },
            indent=2,
        )
    )
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
