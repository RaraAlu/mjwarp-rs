"""CPU self-check for the dual-camera `camera_recipe` (spec v3 §2.4 / §11.1 item 6-3).

Zero GPU, zero capture: AST/static assertions on the two authoring scripts plus a
pinned-venv CPU stub proof on both scene variants.

Static assertions
  1. `camera_recipe` is defined in `scripts/perworld_capture_meta.py` only (exactly
     one definition) and no longer in `scripts/capture_render_perworld.py`;
  2. the main script imports it from `perworld_capture_meta` exactly once and calls
     it exactly once;
  3. the meta module keeps no product/GPU imports at module top (offline tooling);
  4. the append dict keys are exactly the 10 fields, in the fixed order;
  5. no new file under scripts/ and only the two card scripts changed;
  6. line counts (git-blob newline) main 500 -> 487, meta 244 -> 270, both <= 500.

CPU stub proof (pinned venv MuJoCo, no CUDA)
  - parse `fixtures/render-bvh/scene-a-light/scene.xml` (3 cameras, active 2) and
    `fixtures/render-bvh/scene-a/scene.xml` (2 cameras, active 1);
  - stub rc = cam_id_map + cam_res built from the model; call camera_recipe;
  - assert every active camera has exactly the 10 keys in order and each value is
    bit-identical to the raw model array (or the stub cam_res for resolution);
  - assert cam-hinge records sensorsize [0,0] / intrinsic [0.01,0.01,0,0] and that
    it uses the fovy path (no intrinsics claim);
  - negative mutations (missing key / extra key / wrong value) must be rejected by
    the same assertion helper.

Exit code 0 only when every case passes (last line: SUMMARY PASS n/n).
"""

from __future__ import annotations

import ast
import json
import pathlib
import struct
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
SCRIPTS = ROOT / "scripts"
MAIN = SCRIPTS / "capture_render_perworld.py"
META = SCRIPTS / "perworld_capture_meta.py"
# Base revision for the before/after comparison: the card start-transition commit
# (HEAD before the script edits). Pinned so the assertions still reproduce after
# this card is committed.
BASE_REV = "19d5fb5143358191ee9af45ea3eac5144abd868d"
FIELDS = ["slot", "mujoco_cam_id", "name", "mode", "xpos", "xquat", "fovy",
          "resolution", "sensorsize", "intrinsic"]
EXPECTED = {
    "scene-a-light": {
        "path": "fixtures/render-bvh/scene-a-light/scene.xml",
        "active": [0, 1],
        "resolution": {"cam-persp": (320, 240), "cam-hinge": (320, 240)},
        "sensorsize": {"cam-persp": [0.035999998450279236, 0.024000000208616257],
                       "cam-hinge": [0.0, 0.0]},
        "intrinsic": {"cam-persp": [0.017999999225139618, 0.01600000075995922, 0.0, 0.0],
                      "cam-hinge": [0.009999999776482582, 0.009999999776482582, 0.0, 0.0]},
        "fovy": {"cam-persp": 73.73979461915735, "cam-hinge": 30.0},
    },
    "scene-a": {
        "path": "fixtures/render-bvh/scene-a/scene.xml",
        "active": [0],
        "resolution": {"cam-persp": (320, 240)},
        "sensorsize": {"cam-persp": [0.035999998450279236, 0.024000000208616257]},
        "intrinsic": {"cam-persp": [0.017999999225139618, 0.01600000075995922, 0.0, 0.0]},
        "fovy": {"cam-persp": 73.73979461915735},
    },
}
results: list[tuple[str, bool]] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    results.append((name, ok))
    print(("PASS " if ok else "FAIL ") + name + ((" :: " + detail) if detail else ""))


def head_lines(rel: str) -> int:
    blob = subprocess.run(["git", "-C", str(ROOT), "cat-file", "-p", BASE_REV + ":" + rel],
                          capture_output=True, check=True).stdout
    return blob.count(b"\n")


def worktree_lines(rel: str) -> int:
    return (ROOT / rel).read_bytes().count(b"\n")


def bit_test(value, model_value) -> bool:
    if isinstance(model_value, (int,)) and not isinstance(model_value, bool):
        return int(value) == int(model_value)
    return struct.pack("<d", float(value)) == struct.pack("<d", float(model_value))


def mutations(fn) -> None:
    """The assertion helper must reject missing/extra/wrong fields (mutation cases)."""
    base = fn("scene-a-light")
    bad_missing = [dict(cam) for cam in base]
    bad_missing[1].pop("intrinsic")
    check("negative-missing-key", raises(lambda: assert_rows(bad_missing, "scene-a-light")))
    bad_extra = [dict(cam) for cam in base]
    bad_extra[0]["extra"] = 1
    check("negative-extra-key", raises(lambda: assert_rows(bad_extra, "scene-a-light")))
    bad_value = [dict(cam) for cam in base]
    bad_value[0]["sensorsize"] = [0.0, 0.0]
    check("negative-wrong-value", raises(lambda: assert_rows(bad_value, "scene-a-light")))


def raises(fn) -> bool:
    try:
        fn()
    except AssertionError:
        return True
    return False


def assert_rows(rows, scene: str) -> None:
    """Assertion helper: 10 keys in order, values bit-identical to model/stub."""
    import mujoco
    import numpy as np

    exp = EXPECTED[scene]
    mjm = mujoco.MjModel.from_xml_path(str(ROOT / exp["path"]))
    active = exp["active"]
    assert len(rows) == len(active), f"{scene}: {len(rows)} rows != {len(active)} active cameras"
    for row, cam_id in zip(rows, active):
        assert list(row.keys()) == FIELDS, f"{scene}: key order/closure {list(row.keys())}"
        name = mjm.camera(cam_id).name
        assert row["name"] == name, f"{scene}: name"
        assert row["slot"] == active.index(cam_id), f"{scene}: slot"
        assert row["mujoco_cam_id"] == cam_id, f"{scene}: mujoco_cam_id"
        assert row["mode"] == int(mjm.cam_mode[cam_id]), f"{scene}: mode"
        for key, model in [
            ("xpos", mjm.cam_pos[cam_id]), ("xquat", mjm.cam_quat[cam_id]),
            ("sensorsize", mjm.cam_sensorsize[cam_id]), ("intrinsic", mjm.cam_intrinsic[cam_id]),
        ]:
            assert len(row[key]) == len(model), f"{scene}: {name}.{key} length"
            assert all(bit_test(row[key][i], model[i]) for i in range(len(model))), \
                f"{scene}: {name}.{key} not bit-identical: {row[key]} vs {list(model)}"
        assert bit_test(row["fovy"], mjm.cam_fovy[cam_id]), f"{scene}: {name}.fovy"
        w, h = exp["resolution"][name]
        assert row["resolution"] == [w, h], f"{scene}: {name}.resolution"
        # explicit expected values from the pinned model / spec
        assert all(bit_test(row["sensorsize"][i], exp["sensorsize"][name][i])
                   for i in range(len(exp["sensorsize"][name]))), f"{scene}: {name}.sensorsize"
        assert all(bit_test(row["intrinsic"][i], exp["intrinsic"][name][i])
                   for i in range(len(exp["intrinsic"][name]))), f"{scene}: {name}.intrinsic"
        assert bit_test(row["fovy"], exp["fovy"][name]), f"{scene}: {name}.fovy expected"


def stub_rc(scene: str):
    """Stub render context: only the two attributes camera_recipe reads."""
    import mujoco
    import numpy as np

    exp = EXPECTED[scene]
    mjm = mujoco.MjModel.from_xml_path(str(ROOT / exp["path"]))
    active = [c for c in range(int(mjm.ncam)) if int(mjm.cam_mode[c]) == 0]
    assert active == exp["active"], f"{scene}: active camera ids {active}"
    res = np.asarray([mjm.cam_resolution[c] for c in active], dtype=np.int64)

    class NpWrap:
        """Mimic a warp array's `.numpy()` accessor."""

        def __init__(self, values):
            self._values = values

        def numpy(self):
            return self._values

    class StubRc:
        cam_id_map = NpWrap(np.asarray(active, dtype=np.int64))
        cam_res = NpWrap(res)

    return mjm, StubRc(), active


def main() -> int:
    sys.path.insert(0, str(SCRIPTS))
    from perworld_capture_meta import camera_recipe  # noqa: E402

    # ---- static assertions -------------------------------------------------
    main_tree = ast.parse(MAIN.read_text(encoding="utf-8"))
    meta_tree = ast.parse(META.read_text(encoding="utf-8"))
    check("no-def-in-main", not any(isinstance(n, ast.FunctionDef) and n.name == "camera_recipe"
                                    for n in ast.walk(main_tree)))
    defs = [n for n in ast.walk(meta_tree) if isinstance(n, ast.FunctionDef) and n.name == "camera_recipe"]
    check("one-def-in-meta", len(defs) == 1)
    imports = [n for n in ast.walk(main_tree)
               if isinstance(n, ast.ImportFrom) and n.module == "perworld_capture_meta"
               and any(a.name == "camera_recipe" for a in n.names)]
    check("main-import-once", len(imports) == 1)
    calls = [n for n in ast.walk(main_tree)
             if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "camera_recipe"]
    check("main-call-once", len(calls) == 1)
    top_imports = set()
    for node in meta_tree.body:
        if isinstance(node, ast.Import):
            top_imports.update(a.name.split(".")[0] for a in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            top_imports.add(node.module.split(".")[0])
    check("meta-no-product-imports", not (top_imports & {"warp", "mujoco", "mujoco_warp", "numpy"}),
          str(sorted(top_imports)))

    # append dict key closure/order (the single append in the def body)
    fn = defs[0]
    dicts = [n for n in ast.walk(fn) if isinstance(n, ast.Dict)]
    keys = [k.value for k in dicts[0].keys]
    check("append-keys-10-in-order", keys == FIELDS, str(keys))

    changed = subprocess.run(["git", "-C", str(ROOT), "diff", "--name-only", BASE_REV, "--", "scripts/"],
                             capture_output=True, text=True, encoding="utf-8").stdout.split()
    check("only-two-scripts-changed",
          changed == ["scripts/capture_render_perworld.py", "scripts/perworld_capture_meta.py"], str(changed))
    added = subprocess.run(["git", "-C", str(ROOT), "diff", "--diff-filter=A", "--name-only", BASE_REV,
                            "--", "scripts/"],
                           capture_output=True, text=True, encoding="utf-8").stdout.split()
    check("no-new-module-file", not added, str(added))

    line_rows = {"scripts/capture_render_perworld.py": (500, 487),
                 "scripts/perworld_capture_meta.py": (244, 270)}
    for rel, (before, after) in line_rows.items():
        got_before, got_after = head_lines(rel), worktree_lines(rel)
        check(f"lines {rel}", (got_before, got_after) == (before, after), f"{got_before}->{got_after}")
        check(f"lines<=500 {rel}", got_after <= 500, str(got_after))

    # ---- CPU stub proof ----------------------------------------------------
    for scene in ("scene-a-light", "scene-a"):
        mjm, rc, active = stub_rc(scene)
        rows = camera_recipe(mjm, rc)
        print(f"  {scene}: {len(rows)} rows, keys={list(rows[0].keys())}")
        for row in rows:
            print("    " + json.dumps(row, ensure_ascii=False))
        assert_rows(rows, scene)
        check(f"stub-10-keys {scene}", all(list(r.keys()) == FIELDS for r in rows))
        check(f"stub-bitwise {scene}", True)

    mutations(lambda scene: camera_recipe(*stub_rc(scene)[:2]))

    passed = sum(1 for _, ok in results if ok)
    print(f"SUMMARY {'PASS' if passed == len(results) else 'FAIL'} {passed}/{len(results)}")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
