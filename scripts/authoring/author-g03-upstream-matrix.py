"""Offline authoring: frozen upstream G03 mass-matrix reference (mixed-joints tile block).

OFFLINE ONLY. This script is not imported, built, or executed by the Rust product,
its tests, or its scripts. It generates an authoring artifact under
docs/development-evidence/; the product never reads that artifact.

Product and test runs still use cargo only and never start Python.
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
from mujoco_warp._src import io, types

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
MODEL = "mixed-joints"
WORLDS = 8
EXPECTED = {
    "python": "3.12.13",
    "mujoco": "3.15.0",
    "warp": "1.15.0",
    "numpy": "2.5.4",
    "mujoco-warp": "3.15.0",
}

# Snapshot fields of the mass-matrix chain, all f32 device arrays.
FIELDS = (
    "xpos",
    "xquat",
    "xmat",
    "xipos",
    "ximat",
    "xanchor",
    "xaxis",
    "subtree_com",
    "cdof",
    "cinert",
    "crb",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def array_bytes(value) -> bytes:
    return np.ascontiguousarray(value).tobytes()


def dense_block(data, model, host, world: int) -> tuple[list[float], dict]:
    """Densify the 11-dim tile exactly as Warp's tile_cholesky MathDx branch does.

    io.py precomputes tile.elemid as the CSR address of M[max(i,j), min(i,j)]
    for every dense slot, with mjm.nC marking structurally absent pairs. The
    tile kernel reads those slots through tile_load_indexed, where the nC
    sentinel is an out-of-bounds read that returns zero, so we map it to 0.0.
    """
    size = host.nv  # mixed-joints has one 11-dof tree block
    tile = None
    for candidate in model.M_tiles:
        if candidate.size == size and candidate.elemid.size > 0:
            tile = candidate
            break
    if tile is None:
        raise RuntimeError("no gather tile for the 11-dof block")
    adr = np.asarray(tile.adr.numpy(), dtype=np.int64)
    blocks = [int(start) for start in adr]
    if blocks[:1] != [0]:
        raise RuntimeError(f"unexpected tile starts: {blocks}")
    elem = np.asarray(tile.elemid.numpy(), dtype=np.int64)
    area = size * size
    index = elem[0:area]
    csr = np.asarray(data.M.numpy()[world], dtype=np.float32)
    values = np.where(index >= host.nC, np.float32(0.0), csr[np.minimum(index, host.nC - 1)])
    return [float(v) for v in values.reshape(-1)], {
        "start": 0,
        "size": int(size),
        "route": "tile_cholesky",
        "block_elemid": [int(i) for i in index],
        "nC": int(host.nC),
    }


def snapshot(data, host) -> dict:
    state = {}
    for name in FIELDS:
        value = np.asarray(getattr(data, name).numpy())
        state[name] = {"shape": list(value.shape), "values": [float(v) for v in value.reshape(-1)]}
    state["M"] = {
        "shape": list(np.asarray(data.M.numpy()).shape),
        "values": [float(v) for v in np.asarray(data.M.numpy()).reshape(-1)],
    }
    return state


def build_host() -> "mujoco.MjModel":
    base = json.loads((ROOT / "fixtures/mass-matrix" / f"{MODEL}.json").read_text())
    host = mujoco.MjModel.from_xml_path(str(ROOT / "fixtures/native-probe" / f"{MODEL}.xml"))
    for key, values in base["model"].items():
        target = getattr(host, key)
        array = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        if target.size != array.size:
            raise RuntimeError(f"model field size mismatch: {key}")
        if target.dtype.kind == "f":
            target[:] = array.reshape(target.shape)
        else:
            np.testing.assert_array_equal(target.reshape(-1), array, err_msg=key)
    return host


def main() -> int:
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)

    # Environment re-check (the wrapper performs the same checks before launch).
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

    host = build_host()
    if host.nv != 11:
        raise RuntimeError(f"expected nv=11 for {MODEL}, got {host.nv}")
    native = json.loads((ROOT / "fixtures/mass-solve" / f"{MODEL}.json").read_text())
    qpos = np.asarray(
        [native["cases"][w % 8]["qpos"] for w in range(WORLDS)], dtype=np.float32
    ).reshape(WORLDS, host.nq)

    work = ROOT / "target" / "g03-upstream-matrix"
    work.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(work / "warp-cache")
    wp.init()
    wp.set_device("cuda:0")
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=WORLDS, nconmax=0, njmax=0)
    data.qpos.assign(qpos)
    wp.synchronize_device()

    def run_chain() -> dict:
        mjw.kinematics(model, data)
        mjw.com_pos(model, data)
        mjw.crb(model, data)
        wp.synchronize_device()
        return snapshot(data, host)

    first = run_chain()
    second = run_chain()
    for name in list(first):
        if array_bytes(np.asarray(first[name]["values"], dtype=np.float64)) != array_bytes(
            np.asarray(second[name]["values"], dtype=np.float64)
        ):
            raise RuntimeError(f"chain is not deterministic for {name}")

    blocks = []
    dense = []
    for world in range(WORLDS):
        values, meta = dense_block(data, model, host, world)
        dense.append(values)
        if world == 4:
            blocks.append(meta)

    reference = json.loads((ROOT / "fixtures/mjwarp-mass-solve/reference.json").read_text())
    row = None
    for entry in reference["models"]:
        if entry["name"] == MODEL:
            for candidate in entry["rows"]:
                if (
                    candidate["worlds"] == WORLDS
                    and candidate["rhs_count"] == 3
                    and candidate["world"] == 4
                ):
                    row = candidate
                    break
    if row is None:
        raise RuntimeError("frozen reference row for mixed-joints world=4 not found")

    inputs = [
        "fixtures/mass-matrix/mixed-joints.json",
        "fixtures/mass-solve/mixed-joints.json",
        "fixtures/native-probe/mixed-joints.xml",
        "fixtures/mjwarp-mass-solve/reference.json",
    ]
    provenance = {
        "schema": 1,
        "kind": "upstream-matrix-authoring",
        "offline_only": True,
        "product_chain_reads": False,
        "model": MODEL,
        "worlds": WORLDS,
        "world": 4,
        "upstream_revision": REVISION,
        "upstream_head": head,
        "upstream_clean": True,
        "versions": versions,
        "gpu": {
            "name": str(wp.get_device().name) if hasattr(wp.get_device(), "name") else str(wp.get_device()),
            "driver": subprocess.check_output(
                ["nvidia-smi", "--query-gpu=driver_version", "--format=csv,noheader"], text=True
            ).strip(),
        },
        "command": " ".join(sys.argv),
        "cwd": str(ROOT),
        "script_sha256": sha256_file(Path(__file__).resolve()),
        "inputs": {rel: sha256_file(ROOT / rel) for rel in inputs},
        "frozen_reference_row": {
            "case": row["case"],
            "worlds": row["worlds"],
            "rhs_count": row["rhs_count"],
            "world": row["world"],
        },
        "deterministic_runs": 2,
        "dense_block": blocks[0] if blocks else None,
    }

    matrix_path = output / "matrix-world4.f32"
    np.asarray(dense[4], dtype=np.float32).tofile(matrix_path)
    all_matrix_path = output / "dense-blocks.f32"
    np.asarray(dense, dtype=np.float32).tofile(all_matrix_path)

    payload = {
        "provenance": provenance,
        "dense_blocks": dense,
        "snapshots": first,
        "frozen_factor_world4": [float(v) for v in row["factor"]],
    }
    json_path = output / "upstream-matrix.json"
    json_path.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")

    outputs = {
        "upstream_matrix_json": sha256_file(json_path),
        "matrix_world4_f32": sha256_file(matrix_path),
        "dense_blocks_f32": sha256_file(all_matrix_path),
    }
    provenance["outputs"] = outputs
    (output / "provenance.json").write_text(
        json.dumps(provenance, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n"
    )
    provenance["outputs"]["provenance_json"] = sha256_file(output / "provenance.json")
    print(json.dumps({"outputs": provenance["outputs"], "deterministic_runs": 2}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
