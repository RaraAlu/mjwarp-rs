"""Offline authoring only; never imported or executed by product tests."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import platform
import subprocess
import sys

import mujoco
import numpy as np
import warp as wp
import mujoco_warp as mjw
from mujoco_warp._src import smooth

ROOT = Path(__file__).resolve().parents[2]
WORK = ROOT / "target/mjwarp-parallel"
UPSTREAM = WORK / "upstream"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
FLOAT_FIELDS = ["xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
                "subtree_com", "cdof", "geom_xpos", "geom_xmat", "site_xpos", "site_xmat",
                "ten_length", "ten_J", "wrap_xpos"]
INT_FIELDS = ["ten_wrapadr", "ten_wrapnum", "wrap_obj"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8", newline="\n")


def main():
    assert mujoco.__version__ == "3.15.0"
    assert wp.__version__ == "1.15.0"
    assert np.__version__ == "2.5.4"
    assert subprocess.check_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True).strip() == REVISION
    assert not subprocess.check_output(["git", "-C", str(UPSTREAM), "diff", "--name-only"], text=True).strip()
    installed = Path(mjw.__file__).parent
    sources = {}
    for path in sorted((UPSTREAM / "mujoco_warp").rglob("*.py")):
        rel = path.relative_to(UPSTREAM / "mujoco_warp")
        assert sha(path) == sha(installed / rel), str(rel)
        sources[str(rel).replace("\\", "/")] = sha(path)
    base_path = ROOT / "fixtures/geom-tendon/wrap-tree.json"
    motion_path = ROOT / "fixtures/geom-tendon/inside-cylinder-motion.json"
    xml = ROOT / "fixtures/geom-tendon/wrap-tree.xml"
    base, motion = json.loads(base_path.read_text()), json.loads(motion_path.read_text())
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")
    host = mujoco.MjModel.from_xml_path(str(xml))
    # Compile topology offline, then explicitly quantize every consumed fixture field.
    for key, values in base["model"].items():
        target = getattr(host, key)
        incoming = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        assert target.size == incoming.size, key
        if target.dtype.kind != "f":
            np.testing.assert_array_equal(target.reshape(-1), incoming, err_msg=key)
        else:
            target[:] = incoming.reshape(target.shape)
    model = mjw.put_model(host, batch_sizes={"geom_size": 3})
    model.geom_size.assign(np.asarray([c["geom_size"] for c in base["cases"][:3]], dtype=np.float32).reshape(3, host.ngeom, 3))
    mappings = {}
    for key, values in base["model"].items():
        device_array = getattr(model, key, None)
        if not isinstance(device_array, wp.array):
            continue
        actual = device_array.numpy().reshape(-1)
        if key == "geom_size":
            values = [c["geom_size"] for c in base["cases"][:3]]
        expected = np.asarray(values, dtype=actual.dtype).reshape(-1)
        np.testing.assert_array_equal(actual, expected, err_msg=key)
        mappings[key] = {"dtype": str(actual.dtype), "sha256": hashlib.sha256(actual.tobytes()).hexdigest()}
    mappings["geom_size"] = {"dtype": "float32", "sha256": hashlib.sha256(model.geom_size.numpy().tobytes()).hexdigest()}
    rows = []
    for worlds in (3, 513):
        data = mjw.make_data(host, nworld=worlds, nconmax=0, njmax=0)
        for kind, cases in (("base", list(range(4))), ("motion", motion["cases"])):
            for case in cases:
                if kind == "base":
                    inputs = [base["cases"][case * 3 + w % 3] for w in range(worlds)]
                    selected = [w for w in (0, 1, 2, 512) if w < worlds]
                    case_id = case
                else:
                    inputs = [base["cases"][w % 3] for w in range(worlds)]
                    selected = [w for w in (2, 512) if w < worlds]
                    for w in selected:
                        inputs[w] = case
                    case_id = case["source_frame"]
                input_hashes = {}
                for key in ("qpos", "mocap_pos", "mocap_quat"):
                    arr = getattr(data, key)
                    values = np.asarray([c[key] for c in inputs], dtype=np.float32).reshape(arr.numpy().shape)
                    arr.assign(values)
                    input_hashes[key] = hashlib.sha256(values.tobytes()).hexdigest()
                # Only these unmodified upstream stages produce the reference results.
                mjw.kinematics(model, data)
                mjw.com_pos(model, data)
                mjw.tendon(model, data)
                wp.synchronize_device()
                arrays = {key: getattr(data, key).numpy() for key in FLOAT_FIELDS + INT_FIELDS}
                for key in FLOAT_FIELDS:
                    assert arrays[key].dtype == np.float32, key
                    assert np.isfinite(arrays[key]).all(), key
                for world in selected:
                    rows.append({"worlds": worlds, "world": world, "kind": kind, "case": case_id,
                                 "input_hashes": input_hashes,
                                 **{key: value[world].reshape(-1).tolist() for key, value in arrays.items()}})
    options = wp.get_module_options(smooth)
    assert options["fuse_fp"] and not options["fast_math"]
    device = wp.get_device()
    dump(WORK / "reference.json", {"schema": 1, "upstream_revision": REVISION,
                                   "float_fields": FLOAT_FIELDS, "int_fields": INT_FIELDS, "rows": rows})
    cache = {str(p.relative_to(WORK)).replace("\\", "/"): sha(p)
             for p in sorted((WORK / "warp-cache").rglob("*")) if p.is_file()}
    dump(WORK / "provenance.json", {
        "scope": "G01 geom tendon diagnostic subset; not complete T4 acceptance",
        "upstream_revision": REVISION,
        "packages": {p: importlib.metadata.version(p) for p in ("mujoco-warp", "mujoco", "warp-lang", "numpy")},
        "python": sys.version, "platform": platform.platform(),
        "gpu": subprocess.check_output(["nvidia-smi", "--query-gpu=name,uuid,driver_version", "--format=csv,noheader"], text=True).strip(),
        "cuda_arch": device.arch, "module_options": options,
        "cuda_toolkit": "12.9.86 (verified generated PTX header)",
        "licenses": {p: importlib.metadata.metadata(p).get("License-Expression") or importlib.metadata.metadata(p).get("License")
                     for p in ("mujoco-warp", "mujoco", "warp-lang", "numpy")},
        "inputs": {str(p.relative_to(ROOT)).replace("\\", "/"): sha(p) for p in (base_path, motion_path, xml)},
        "device_model_fields": mappings, "source_hashes": sources, "cache_hashes": cache,
        "reference_sha256": sha(WORK / "reference.json"),
        "authoring_script_sha256": sha(Path(__file__)),
        "policy": {"float": "Report exact bit mismatches, absolute, symmetric relative and ULP errors. No tolerance acceptance claim.",
                   "integer": "Exact equality", "performance": "Not measured"}})
    print("REFERENCE_ROWS", len(rows))
    print("MODULE_OPTIONS", options)


if __name__ == "__main__":
    main()
