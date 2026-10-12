"""Offline stage trace; never invoked by product builds or tests."""
import json
import subprocess
from pathlib import Path

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp
from mujoco_warp._src import smooth

from reference import ROOT, UPSTREAM, REVISION, sha, dump


def main():
    assert (mujoco.__version__, wp.__version__, np.__version__) == ("3.15.0", "1.15.0", "2.5.4")
    assert subprocess.check_output(["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True).strip() == REVISION
    assert not subprocess.check_output(["git", "-C", str(UPSTREAM), "status", "--porcelain"], text=True).strip()
    for source in (UPSTREAM / "mujoco_warp").rglob("*.py"):
        assert sha(source) == sha(Path(mjw.__file__).parent / source.relative_to(UPSTREAM / "mujoco_warp"))
    work = ROOT / "target/g03-first-difference"
    work.mkdir(parents=True, exist_ok=True)
    wp.config.kernel_cache_dir = str(ROOT / "target/g03-block-reference/warp-cache")
    wp.init()
    wp.set_device("cuda:0")
    base_path = ROOT / "fixtures/mass-matrix/mixed-joints.json"
    native_path = ROOT / "fixtures/mass-solve/mixed-joints.json"
    base = json.loads(base_path.read_text())
    native = json.loads(native_path.read_text())
    host = mujoco.MjModel.from_xml_path(str(ROOT / "fixtures/native-probe/mixed-joints.xml"))
    for key, values in base["model"].items():
        target = getattr(host, key)
        values = np.asarray(values, dtype=np.float32 if target.dtype.kind == "f" else np.int32)
        if target.dtype.kind == "f":
            target[:] = values.reshape(target.shape)
        else:
            np.testing.assert_array_equal(target.reshape(-1), values)
    model = mjw.put_model(host)
    data = mjw.make_data(host, nworld=8, nconmax=0, njmax=0)
    qpos = np.asarray([c["qpos"] for c in native["cases"]], dtype=np.float32)
    data.qpos.assign(qpos)
    mjw.kinematics(model, data)
    mjw.com_pos(model, data)
    mjw.crb(model, data)
    mjw.factor_m(model, data)
    solutions = []
    for r in range(3):
        rhs = np.asarray([c["rhs"] for c in native["cases"]], dtype=np.float32).reshape(8, 3, host.nv)[:, r, :].copy()
        x = wp.zeros((8, host.nv), dtype=float)
        mjw.solve_m(model, data, x, wp.array(rhs, dtype=float))
        solutions.append(x.numpy())
    fields = {k: getattr(data, k).numpy() for k in ("xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis", "subtree_com", "cinert", "cdof", "crb")}
    matrix = np.zeros((8, host.nv, host.nv), dtype=np.float32)
    packed = data.M.numpy()
    adr, nnz = model.M_rowadr.numpy(), model.M_rownnz.numpy()
    for i in range(host.nv):
        j, off = i, int(adr[i] + nnz[i] - 1)
        while j >= 0:
            matrix[:, i, j] = matrix[:, j, i] = packed[:, off]
            j, off = host.dof_parentid[j], off - 1
    fields["matrix"] = matrix
    fields["factor"] = np.triu(data.qLD.numpy()[:, :host.nv**2].reshape(8, host.nv, host.nv))
    fields["solution"] = np.stack(solutions, axis=1)
    frozen = json.loads((Path(__file__).parent / "reference.json").read_text())
    rows = frozen["models"][0]["rows"]
    for w in range(8):
        row = next(r for r in rows if (r["worlds"], r["rhs_count"], r["world"]) == (8, 3, w))
        for key in ("factor", "solution"):
            np.testing.assert_array_equal(fields[key][w].reshape(-1), np.asarray(row[key], dtype=np.float32))
    result = {"schema": 1, "revision": REVISION, "worlds": 8, "rhs_count": 3,
              "case": 1, "scalar": "f32", "module_options": wp.get_module_options(smooth),
              "inputs": {str(p.relative_to(ROOT)): sha(p) for p in (base_path, native_path)},
              "authoring_sha256": sha(Path(__file__)),
              "rows": [{k: a[w].reshape(-1).tolist() for k, a in fields.items()} for w in range(8)]}
    dump(work / "upstream.json", result)
    print("TRACE upstream fields", list(fields), flush=True)


if __name__ == "__main__":
    main()
