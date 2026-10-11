"""Offline diagnostic authoring; product tests never execute this script."""
import hashlib
import importlib.util
import inspect
import json
import platform
from pathlib import Path

import numpy as np
import mujoco
import mujoco_warp as mjw
import warp as wp
from mujoco_warp._src import smooth, util_misc

ROOT = Path(__file__).resolve().parents[2]
WORK = ROOT / "target/frame393-trace"


def replace_once(source, old, new):
    assert source.count(old) == 1, old
    return source.replace(old, new)


def instrument():
    # Clone the frozen functions, not the installed package. Keep all arithmetic.
    source = inspect.getsource(util_misc.wrap_inside.func)
    source = replace_once(source, "def wrap_inside(", "def traced_inside(")
    source = replace_once(source, "  end: wp.vec4,", "  end: wp.vec4,\n  trace: wp.array(dtype=float),")
    additions = [
        ("  dd = wp.dot(dif, dif)", "\n  trace[10] = len0\n  trace[11] = len1\n  trace[12] = dif[0]\n  trace[13] = dif[1]\n  trace[14] = dd"),
        ("  sq_B = B * B", "\n  trace[15] = A\n  trace[16] = B\n  trace[17] = sq_A\n  trace[18] = sq_B"),
        ("  cosG = math.safe_div(len0 * len0 + len1 * len1 - dd, 2.0 * len0 * len1)", "\n  trace[19] = cosG"),
        ("  G = wp.acos(cosG)", "\n  trace[20] = G"),
        ("  # make sure init is not on the other side", "\n  trace[21] = z\n  trace[22] = f"),
        ("    # check sign; SHOULD NOT OCCUR", "\n    trace[23 + iter * 6] = z\n    trace[24 + iter * 6] = f\n    trace[25 + iter * 6] = sq_z\n    trace[26 + iter * 6] = df"),
        ("    z1 = z - math.safe_div(f, df)", "\n    trace[27 + iter * 6] = z1"),
        ("    # exit if positive: SHOULD NOT OCCUR", "\n    trace[28 + iter * 6] = f"),
        ("  # check convergence", "\n  trace[143] = float(iter)\n  trace[144] = z\n  trace[145] = f"),
        ("  vec, _ = math.normalize_with_norm(vec)", "\n  trace[146] = ang\n  trace[147] = vec[0]\n  trace[148] = vec[1]"),
    ]
    for anchor, extra in additions:
        source = replace_once(source, anchor, anchor + extra)
    # Only the final return follows this exact blank-line delimiter.
    source = replace_once(source, "\n  return 0.0, pnt, pnt\n", "\n  trace[149] = pnt[0]\n  trace[150] = pnt[1]\n  return 0.0, pnt, pnt\n")
    wrap = inspect.getsource(util_misc.wrap.func)
    wrap = replace_once(wrap, "def wrap(", "def traced_wrap(")
    wrap = replace_once(wrap, "  x0: wp.vec3,", "  trace: wp.array(dtype=float), x0: wp.vec3,")
    wrap = replace_once(wrap, "  p1 = matT @ (x1 - pos)", "  p1 = matT @ (x1 - pos)\n  for i in range(3):\n    trace[i] = p0[i]\n    trace[3 + i] = p1[i]")
    wrap = replace_once(wrap, "  # handle sidesite", "  for i in range(4):\n    trace[6 + i] = end[i]\n\n  # handle sidesite")
    wrap = replace_once(wrap, "wrap_inside(end, radius)", "traced_inside(end, trace, radius)")
    wrap = replace_once(wrap, "    # correct wlen for height", "    trace[151] = L0\n    trace[152] = L1\n    trace[153] = res0[2]\n    trace[154] = res1[2]\n\n    # correct wlen for height")
    wrap = replace_once(wrap, "  return wlen, wpnt0, wpnt1", "  for i in range(3):\n    trace[155 + i] = wpnt0[i]\n    trace[158 + i] = wpnt1[i]\n  return wlen, wpnt0, wpnt1")
    prefix = "from typing import Tuple\nimport warp as wp\nfrom mujoco_warp._src import math\nfrom mujoco_warp._src.util_misc import MJ_MINVAL, MJ_MAXVAL, WrapType, wrap_circle, wrap\n"
    kernel = '''
@wp.kernel
def probe(inputs: wp.array(dtype=float), trace: wp.array(dtype=float), output: wp.array(dtype=float)):
    x0 = wp.vec3(inputs[0], inputs[1], inputs[2])
    x1 = wp.vec3(inputs[3], inputs[4], inputs[5])
    pos = wp.vec3(inputs[6], inputs[7], inputs[8])
    mat = wp.mat33(inputs[9], inputs[10], inputs[11], inputs[12], inputs[13], inputs[14], inputs[15], inputs[16], inputs[17])
    side = wp.vec3(inputs[18], inputs[19], inputs[20])
    length, a, b = wrap(x0, x1, pos, mat, inputs[21], 5, side)
    tlength, ta, tb = traced_wrap(trace, x0, x1, pos, mat, inputs[21], 5, side)
    output[0] = length
    output[7] = tlength
    for i in range(3):
        output[1 + i] = a[i]
        output[4 + i] = b[i]
        output[8 + i] = ta[i]
        output[11 + i] = tb[i]
'''
    full = inspect.getsource(smooth._spatial_geom_tendon.func)
    full = replace_once(full, "def _spatial_geom_tendon(", "def traced_geom(\n  trace: wp.array2d[float],")
    full = replace_once(full, "util_misc.wrap(site_pnt0,", "traced_wrap(trace[worldid * wrap_geom_adr.shape[0] + elementid], site_pnt0,")
    prefix += "from mujoco_warp._src.smooth import _accumulate_jac_chain\n"
    return prefix + source + "\n" + wrap + kernel + "\n" + full


def full_pipeline(module, base, row):
    host = mujoco.MjModel.from_xml_path(str(ROOT / "fixtures/geom-tendon/wrap-tree.xml"))
    for key, values in base["model"].items():
        target = getattr(host, key)
        incoming = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        if target.dtype.kind == "f":
            target[:] = incoming.reshape(target.shape)
        else:
            np.testing.assert_array_equal(target.reshape(-1), incoming)
    model = mjw.put_model(host, batch_sizes={"geom_size": 3})
    model.geom_size.assign(np.asarray([c["geom_size"] for c in base["cases"][:3]], dtype=np.float32).reshape(3, host.ngeom, 3))
    data = mjw.make_data(host, nworld=3, nconmax=0, njmax=0)
    motion = json.loads((ROOT / "fixtures/geom-tendon/inside-cylinder-motion.json").read_text())
    case = next(c for c in motion["cases"] if c["source_frame"] == 393)
    for key in ("qpos", "mocap_pos", "mocap_quat"):
        array = getattr(data, key)
        values = np.asarray([base["cases"][0][key], base["cases"][1][key], case[key]], dtype=np.float32)
        assert hashlib.sha256(values.tobytes()).hexdigest() == row["input_hashes"][key]
        array.assign(values.reshape(array.numpy().shape))
    mjw.kinematics(model, data)
    mjw.com_pos(model, data)
    mjw.tendon(model, data)
    original = {k: getattr(data, k).numpy().copy() for k in ("wrap_xpos", "ten_J", "ten_length")}
    for key, values in original.items():
        np.testing.assert_array_equal(values[2].reshape(-1).view(np.uint32), np.asarray(row[key], dtype=np.float32).view(np.uint32))
    count = model.wrap_geom_adr.size
    trace = wp.zeros((3 * count, 161), dtype=wp.float32)
    launch = wp.launch

    def instrumented_launch(kernel, *args, **kwargs):
        if kernel is smooth._spatial_geom_tendon:
            kernel = module.traced_geom
            kwargs["inputs"] = [trace] + kwargs["inputs"]
        return launch(kernel, *args, **kwargs)

    wp.launch = instrumented_launch
    try:
        mjw.tendon(model, data)
    finally:
        wp.launch = launch
    differences = {}
    for key, expected in original.items():
        actual = getattr(data, key).numpy()
        differences[key] = int(np.count_nonzero(actual.view(np.uint32) != expected.view(np.uint32)))
    element = int(np.flatnonzero(model.wrap_geom_adr.numpy() == 13)[0])
    return trace.numpy()[2 * count + element].tolist(), differences


def main():
    assert wp.__version__ == "1.15.0"
    assert mujoco.__version__ == "3.15.0"
    assert np.__version__ == "2.5.4"
    provenance = json.loads((Path(__file__).parent / "provenance.json").read_text())
    for relative, digest in provenance["source_hashes"].items():
        assert hashlib.sha256((Path(mjw.__file__).parent / relative).read_bytes()).hexdigest() == digest, relative
    source_path = Path(inspect.getfile(util_misc))
    assert hashlib.sha256(source_path.read_bytes()).hexdigest() == provenance["source_hashes"]["_src/util_misc.py"]
    reference = json.loads((Path(__file__).parent / "reference.json").read_text())
    row = next(r for r in reference["rows"] if (r["worlds"], r["world"], r["kind"], r["case"]) == (3, 2, "motion", 393))
    base = json.loads((ROOT / "fixtures/geom-tendon/wrap-tree.json").read_text())
    # Tendon 4: site 13 -> cylinder 3 (side 18) -> site 14.
    inputs = np.array(row["site_xpos"][39:42] + row["site_xpos"][42:45] + row["geom_xpos"][9:12]
                      + row["geom_xmat"][27:36] + row["site_xpos"][54:57] + [base["cases"][2]["geom_size"][9]], dtype=np.float32)
    WORK.mkdir(parents=True, exist_ok=True)
    module_path = WORK / "upstream_trace.py"
    module_path.write_text(instrument(), encoding="utf-8", newline="\n")
    spec = importlib.util.spec_from_file_location("frame393_upstream_trace", module_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")
    wp.set_module_options({"enable_backward": False, "fuse_fp": True, "fast_math": False}, module)
    trace = wp.zeros(161, dtype=wp.float32)
    output = wp.zeros(14, dtype=wp.float32)
    wp.launch(module.probe, dim=1, inputs=[wp.array(inputs), trace, output])
    values = output.numpy()
    # A standalone call may specialize differently. Retain that diagnostic fact.
    np.testing.assert_array_equal(values[:7].view(np.uint32), values[7:].view(np.uint32))
    full_trace, differences = full_pipeline(module, base, row)
    assert np.isfinite(full_trace).all()
    result = {"frame": 393, "worlds": 3, "world": 2, "tendon": 4, "inputs": inputs.tolist(),
              "isolated_output": values[:7].tolist(), "isolated_trace": trace.numpy().tolist(),
              "output": [0.0] + row["wrap_xpos"][45:51], "trace": full_trace,
              "module_options": wp.get_module_options(module), "gpu": wp.get_device().name,
              "schema": 1, "scope": "frame393 diagnostic; no tolerance acceptance",
              "platform": platform.platform(), "upstream_revision": provenance["upstream_revision"],
              "inputs_sha256": hashlib.sha256(inputs.tobytes()).hexdigest(),
              "author_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "reference_sha256": hashlib.sha256((Path(__file__).parent / "reference.json").read_bytes()).hexdigest(),
              "cache_hashes": {str(p.relative_to(WORK)).replace("\\", "/"): hashlib.sha256(p.read_bytes()).hexdigest()
                               for p in sorted((WORK / "warp-cache").rglob("*")) if p.is_file()},
              "upstream_source_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
              "instrumented_source_sha256": hashlib.sha256(module_path.read_bytes()).hexdigest(),
              "isolated_instrumentation_preserves_output_bits": True,
              "full_instrumentation_bit_mismatches": differences}
    (WORK / "upstream.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    assert not any(differences.values()), differences


if __name__ == "__main__":
    main()
