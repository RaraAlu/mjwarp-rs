"""Offline reference authoring only. Product builds/tests never run Python."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import subprocess
import sys
import time

import mujoco
import numpy as np
import warp as wp
import mujoco_warp as mjw
from mujoco_warp._src import io, smooth

ROOT = Path(__file__).resolve().parents[2]
DEST = Path(__file__).resolve().parent
WORK = ROOT / "target/g03-block-reference"
UPSTREAM = ROOT / "target/mjwarp-parallel/upstream"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
MODELS = {
    "mixed-joints": "native-probe/mixed-joints.xml",
    "inertial-tree": "native-probe/inertial-tree.xml",
    "rotated-tree": "kinematics/rotated-tree.xml",
    "zero-dof": "native-probe/zero-dof.xml",
    "massless-tree": "com-position/massless-tree.xml",
    "armature-chain": "mass-matrix/armature-chain.xml",
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(array):
    return hashlib.sha256(np.asarray(array).tobytes()).hexdigest()


def dump(path, value):
    indent = None if path.name == "reference.json" else 2
    path.write_text(json.dumps(value, indent=indent, allow_nan=False) + "\n", encoding="utf-8", newline="\n")


def main():
    assert (mujoco.__version__, wp.__version__, np.__version__) == ("3.15.0", "1.15.0", "2.5.4")
    assert subprocess.check_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True).strip() == REVISION
    assert not subprocess.check_output(["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True).strip()
    installed = Path(mjw.__file__).parent
    sources = {}
    for path in sorted((UPSTREAM / "mujoco_warp").rglob("*.py")):
        rel = path.relative_to(UPSTREAM / "mujoco_warp")
        assert sha(path) == sha(installed / rel), str(rel)
        sources[rel.as_posix()] = sha(path)
    WORK.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(WORK / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")
    models = []
    inputs = {}
    for name, xml in MODELS.items():
        mp = ROOT / f"fixtures/mass-matrix/{name}.json"
        sp = ROOT / f"fixtures/mass-solve/{name}.json"
        xp = ROOT / "fixtures" / xml
        base, native = json.loads(mp.read_text()), json.loads(sp.read_text())
        for p in (mp, sp, xp):
            inputs[p.relative_to(ROOT).as_posix()] = sha(p)
        host = mujoco.MjModel.from_xml_path(str(xp))
        for key, values in base["model"].items():
            target = getattr(host, key)
            values = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
            assert target.size == values.size, key
            if target.dtype.kind == "f":
                target[:] = values.reshape(target.shape)
            else:
                np.testing.assert_array_equal(target.reshape(-1), values, err_msg=key)
        model = mjw.put_model(host)
        mappings = {}
        for key, values in base["model"].items():
            array = getattr(model, key, None)
            if isinstance(array, wp.array):
                actual = array.numpy().reshape(-1)
                np.testing.assert_array_equal(actual, np.asarray(values, dtype=actual.dtype), err_msg=key)
                mappings[key] = {"dtype": str(actual.dtype), "sha256": digest(actual)}
        layout = io.m_block_layout(host)
        blocks = []
        for start, size in io._m_blocks(host):
            adr = int(layout["dof_adr"][start])
            route = "compact" if adr == -2 else "sparse_ldl" if adr == -1 else "small_cholesky" if size in layout["scalar_tiles"] and start in layout["scalar_tiles"][size] else "tile_cholesky"
            blocks.append({"start": start, "size": size, "route": route, "upstream_address": adr})
        print(name, blocks, flush=True)
        configurations = {(8, 3)}
        if name == "rotated-tree":
            configurations.update((n, 3) for n in (1, 255, 256, 257, 513))
        if name == "mixed-joints":
            configurations.update((8, n) for n in (1, 2, 5))
        if name == "armature-chain":
            configurations.add((1, 3))
        rows = []
        for worlds, nrhs in sorted(configurations):
            data = mjw.make_data(host, nworld=worlds, nconmax=0, njmax=0)
            q = np.asarray([native["cases"][w % 8]["qpos"] for w in range(worlds)], dtype=np.float32).reshape(worlds, host.nq)
            data.qpos.assign(q)
            wp.synchronize_device()
            start_time = time.perf_counter()
            mjw.kinematics(model, data)
            mjw.com_pos(model, data)
            mjw.crb(model, data)
            mjw.factor_m(model, data)
            solutions = []
            rhs_hashes = []
            for r in range(nrhs):
                rhs = np.asarray([native["cases"][w % 8]["rhs"] for w in range(worlds)], dtype=np.float32).reshape(worlds, 3, host.nv)[:, r % 3, :].copy()
                y = wp.array(rhs, dtype=float)
                x = wp.zeros((worlds, host.nv), dtype=float)
                mjw.solve_m(model, data, x, y)
                solutions.append(x.numpy())
                rhs_hashes.append(digest(rhs))
            wp.synchronize_device()
            elapsed = time.perf_counter() - start_time
            ld, inv = data.qLD.numpy(), data.qLDiagInv.numpy()
            assert ld.dtype == inv.dtype == np.float32
            for w in range(worlds):
                c = native["cases"][w % 8]
                factor = np.zeros((host.nv, host.nv), dtype=np.float32)
                diagonal = np.zeros(host.nv, dtype=np.float32)
                native_factor = np.zeros((host.nv, host.nv), dtype=np.float64)
                native_inv = np.zeros(host.nv, dtype=np.float64)
                matrix = np.asarray(c["matrix"], dtype=np.float64).reshape(host.nv, host.nv)
                for block in blocks:
                    b, n, off = block["start"], block["size"], block["upstream_address"]
                    sl = slice(b, b + n)
                    if block["route"] == "compact":
                        diagonal[sl] = inv[w, sl]
                        native_inv[sl] = 1.0 / np.diag(matrix)[sl]
                    elif block["route"] == "sparse_ldl":
                        raise RuntimeError("Frozen fixtures unexpectedly need sparse factors")
                    else:
                        factor[sl, sl] = np.triu(ld[w, off:off + n*n].reshape(n, n))
                        # Independent f64 factor of the ORIGINAL frozen native matrix.
                        # This does not use candidate outputs or replace native solutions.
                        native_factor[sl, sl] = np.linalg.cholesky(matrix[sl, sl]).T
                solution = np.stack([s[w] for s in solutions]).reshape(-1)
                assert np.isfinite(factor).all() and np.isfinite(diagonal).all() and np.isfinite(solution).all()
                rows.append({"worlds": worlds, "rhs_count": nrhs, "world": w, "case": w % 8,
                             "qpos": q[w].tolist(), "rhs": [float(v) for r in range(nrhs) for v in np.asarray(c["rhs"], dtype=np.float32).reshape(3, host.nv)[r % 3]],
                             "qpos_batch_sha256": digest(q), "rhs_batch_sha256": rhs_hashes,
                             "factor": factor.reshape(-1).tolist(), "diagonal_inverse": diagonal.tolist(),
                             "native_factor": native_factor.reshape(-1).tolist(), "native_diagonal_inverse": native_inv.tolist(),
                             "solution": solution.tolist(), "authoring_cold_seconds": elapsed})
        models.append({"name": name, "model_sha256": sha(mp), "native_sha256": sha(sp),
                       "blocks": blocks, "device_fields": mappings, "rows": rows})
    options = wp.get_module_options(smooth)
    assert options["fuse_fp"] and not options["fast_math"]
    dump(DEST / "reference.json", {"schema": 1, "revision": REVISION, "scalar": "f32",
                                  "absolute_tolerance": 2e-4, "relative_tolerance": 2e-4,
                                  "models": models})
    dump(DEST / "manifest.json", {"schema": 1, "revision": REVISION,
        "packages": {p: importlib.metadata.version(p) for p in ("mujoco-warp", "mujoco", "warp-lang", "numpy")},
        "python": sys.version, "module_options": options, "source_hashes": sources,
        "gpu": subprocess.check_output(["nvidia-smi", "--query-gpu=name,uuid,driver_version,memory.total,memory.used", "--format=csv,noheader"], text=True).strip(),
        "inputs": inputs, "reference_sha256": sha(DEST / "reference.json"),
        "authoring_sha256": sha(Path(__file__)),
        "cache_hashes": {p.relative_to(WORK).as_posix(): sha(p) for p in sorted((WORK / "warp-cache").rglob("*")) if p.is_file()},
        "stages": ["kinematics", "com_pos", "crb", "factor_m", "solve_m"],
        "np04": "Not accepted: cold authoring timings include compilation and copies, not steady-state comparison."})


if __name__ == "__main__":
    main()
