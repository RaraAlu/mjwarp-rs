"""Capture-metadata prerequisites for the per-world authoring script (spec v2 §11.1).

OFFLINE ONLY: imported by `scripts/capture_render_perworld.py`; not imported by
product builds/tests/gates. This module owns the tooling/metadata surface of the
S-2 capture-window prerequisites:

1. `command`: the actual command line (`argv` verbatim + rendered + cwd);
2. `captured_at`: strict non-empty ISO8601 from `MJWARP_CAPTURE_AT` (missing or
   empty -> explicit non-zero failure; never write an empty string);
3. `per_world_scope`: new coverage wording (per-world light poses + per-world
   geom poses, with the host-only light semantics premise of spec v2 §10.3);
4. reference identity: keep the script task id as the provenance source and add
   `reference_tag` (reference directory name by convention = spec §2.2 tag),
   instead of parametrising `task_id`;
5. `camera_recipe`: both active cameras' 10-field snapshot (spec v3 §2.4: the
   existing 8 fields plus the raw `sensorsize`/`intrinsic` model arrays, no
   conversion; cam-hinge uses the fovy path and is not claimed to have intrinsics);
6. `discriminability_counts`: the reference-side discriminability counts that
   spec v3 §2.4 requires in `capture.json` (`c1_rgb_changed_px` with both the
   bitwise and the `max_channel|drgb| >= 1/255` counts, `c2_depth_changed_px`,
   `c2_seg_changed_px`, plus the two diagnostic items), bit-identical to the
   G-L0 pilot readout `readout_c1_c2.py` (same unpack, same f32->f64 diff, same
   per-pixel comparisons; no re-derived formula).

Zero GPU: pure metadata helpers; the module also provides a CPU self-check with
positive/negative cases (`python -B scripts/perworld_capture_meta.py`).

Boundary: this module produces no reference and makes no capture-validity or
numeric claim.
"""

from __future__ import annotations

import datetime
import pathlib
import subprocess
import sys

TASK_ID_SOURCE = "script-constant"
TASK_ID = "2026-10-30-render-perworld-capture-authoring"
SCOPE_COVERS = (
    "per-world light poses (light_xpos/light_xdir) AND per-world geom poses "
    "(geom_xpos/geom_xmat) with nworld > 1; device-path light semantics are "
    "host-only retransmission (kernel light arrays stay 1-D, worldid == 0 per "
    "launch, .cu unchanged) as fixed in spec v2 §10.3"
)
SCOPE_NOT_COVERS = (
    "AA x perworld (nsamples>1 still Unsupported), single-launch performance "
    "plan K, continuous animation / large world counts, and any "
    "upstream-implementation equivalence claim"
)
OLD_SCOPE_DOES_NOT_COVER = "per-world light poses (device path indexes lights single-world)"


def fail(message: str) -> None:
    raise SystemExit(f"perworld capture metadata: {message}")


def command_line(argv, cwd) -> dict:
    """Actual command line; argv is recorded verbatim (non-empty)."""
    items = [str(item) for item in argv]
    while items and not items[-1].strip():
        items.pop()
    if not items:
        fail("command argv is empty (the actual command line must be recorded)")
    rendered = subprocess.list2cmdline(items)
    if not rendered.strip():
        fail("command line rendering is empty")
    return {"argv": items, "cwd": str(cwd), "rendered": rendered}


def require_capture_at(env=None) -> str:
    """Strict non-empty ISO8601 `MJWARP_CAPTURE_AT`; explicit failure otherwise."""
    if env is None:
        env = __import__("os").environ
    raw = env.get("MJWARP_CAPTURE_AT")
    if raw is None:
        fail("MJWARP_CAPTURE_AT is missing; set an explicit non-empty ISO8601 timestamp before capture")
    if not str(raw).strip():
        fail("MJWARP_CAPTURE_AT is empty; set an explicit non-empty ISO8601 timestamp before capture")
    text = str(raw).strip()
    try:
        datetime.datetime.fromisoformat(text.replace("Z", "+00:00"))
    except ValueError:
        fail(f"MJWARP_CAPTURE_AT is not ISO8601: {text!r}")
    return text


def capture_at_env_echo(env=None) -> dict:
    """Dry-run state echo only; dry-run never consumes the env value."""
    if env is None:
        env = __import__("os").environ
    raw = env.get("MJWARP_CAPTURE_AT")
    return {"present": raw is not None, "non_empty": bool(str(raw or "").strip())}


def per_world_scope() -> dict:
    """New coverage wording; field names kept for existing consumers."""
    return {"covers": SCOPE_COVERS, "does_not_cover": SCOPE_NOT_COVERS}


def reference_tag(cli_tag, out_dir) -> str:
    """Reference identity: CLI override wins, else the reference directory name (spec §2.2)."""
    if cli_tag is not None and str(cli_tag).strip():
        return str(cli_tag).strip()
    name = pathlib.Path(out_dir).name.strip()
    if not name:
        fail(f"cannot derive reference_tag from out_dir {out_dir!r}; pass --reference-tag")
    return name


def dry_run_echo(argv, cwd, cli_tag, out_dir, env=None) -> dict:
    """Additive dry-run echo: command / reference_tag / captured_at_env (env not consumed)."""
    return {
        "command": command_line(argv, cwd),
        "reference_tag": reference_tag(cli_tag, out_dir),
        "captured_at_env": capture_at_env_echo(env),
    }


def capture_metadata(argv, cwd, cli_tag, out_dir, env=None) -> dict:
    """Concrete capture.json metadata block for the four §11.1 prerequisites."""
    return {
        "command": command_line(argv, cwd),
        "captured_at": require_capture_at(env),
        "reference_tag": reference_tag(cli_tag, out_dir),
        "per_world_scope": per_world_scope(),
        "task_id": TASK_ID,
        "task_id_source": TASK_ID_SOURCE,
    }


def camera_recipe(mjm, rc) -> list[dict]:
    """Both active cameras' snapshot: 10 fields, raw model arrays (spec v3 §2.4).

    `sensorsize`/`intrinsic` are copied from the pinned-MuJoCo model arrays as-is
    (no conversion, no f32/f64 change). `cam-hinge` keeps `sensorsize=[0,0]` and the
    default `intrinsic=[0.01,0.01,0,0]`, i.e. it uses the fovy path - do not claim
    that it has intrinsics.
    """
    cameras = []
    cam_res = rc.cam_res.numpy()
    for slot, cam_id in enumerate(int(v) for v in rc.cam_id_map.numpy()):
        name = mjm.camera(cam_id).name or f"cam{cam_id}"
        cameras.append({"slot": slot, "mujoco_cam_id": int(cam_id), "name": name,
                        "mode": int(mjm.cam_mode[cam_id]),
                        "xpos": [float(v) for v in mjm.cam_pos[cam_id]],
                        "xquat": [float(v) for v in mjm.cam_quat[cam_id]],
                        "fovy": float(mjm.cam_fovy[cam_id]),
                        "resolution": [int(cam_res[slot][0]), int(cam_res[slot][1])],
                        "sensorsize": [float(v) for v in mjm.cam_sensorsize[cam_id]],
                        "intrinsic": [float(v) for v in mjm.cam_intrinsic[cam_id]]})
    return cameras


DISCRIMINABILITY_PAIRS = ((0, 1), (1, 2))
RGB_THRESHOLD = 1.0 / 255.0


def _rgb_counts(left, right) -> dict:
    """`max_channel|drgb| >= 1/255` magnitude count + bitwise-changed count (readout's c1_counts)."""
    import numpy as np

    delta = np.abs(left.astype(np.float64) - right.astype(np.float64)).max(axis=-1)
    return {"magnitude": int(np.count_nonzero(delta >= RGB_THRESHOLD)),
            "bitwise": int(np.count_nonzero(np.any(left.view(np.int32) != right.view(np.int32), axis=-1)))}


def discriminability_counts(arrays: dict) -> dict:
    """spec v3 §2.4 reference-side discriminability counts, bit-identical to `readout_c1_c2.py`.

    `arrays` is the capture array map (`rgb_w{world}_cam{camera}` f32 (H,W,3),
    `depth_w{world}_cam{camera}` f32 (H,W), `seg_w{world}_cam{camera}` i32 (H,W,2)).
    C1 = cam0/cam-persp rgb, C2 = cam1/cam-hinge depth bits + seg pairs; the two
    diagnostic items are cam-persp depth/seg and cam-hinge rgb (ownership never swapped).
    Every field is keyed by world pair `{"0->1": ..., "1->2": ...}` (spec §2.4).
    """
    import numpy as np

    required = [f"{kind}_w{world}_cam{cam}" for kind in ("rgb", "depth", "seg")
                for pair in DISCRIMINABILITY_PAIRS for world in pair for cam in (0, 1)]
    missing = [name for name in required if name not in arrays]
    if missing:
        fail(f"discriminability counts need the two-capture arrays; missing {missing}")
    counts = {"c1_rgb_changed_px": {}, "c2_depth_changed_px": {}, "c2_seg_changed_px": {},
              "campersp_depth_seg_changed_px": {}, "camhinge_rgb_changed_px": {}}
    for left, right in DISCRIMINABILITY_PAIRS:
        key = f"{left}->{right}"
        counts["c1_rgb_changed_px"][key] = _rgb_counts(arrays[f"rgb_w{left}_cam0"], arrays[f"rgb_w{right}_cam0"])
        counts["camhinge_rgb_changed_px"][key] = _rgb_counts(arrays[f"rgb_w{left}_cam1"], arrays[f"rgb_w{right}_cam1"])
        depth_left, depth_right = arrays[f"depth_w{left}_cam1"], arrays[f"depth_w{right}_cam1"]
        seg_left, seg_right = arrays[f"seg_w{left}_cam1"], arrays[f"seg_w{right}_cam1"]
        counts["c2_depth_changed_px"][key] = int(np.count_nonzero(depth_left.view(np.int32) != depth_right.view(np.int32)))
        counts["c2_seg_changed_px"][key] = int(np.count_nonzero(np.any(seg_left != seg_right, axis=-1)))
        persp_depth_left, persp_depth_right = arrays[f"depth_w{left}_cam0"], arrays[f"depth_w{right}_cam0"]
        persp_seg_left, persp_seg_right = arrays[f"seg_w{left}_cam0"], arrays[f"seg_w{right}_cam0"]
        counts["campersp_depth_seg_changed_px"][key] = {
            "depth": int(np.count_nonzero(persp_depth_left.view(np.int32) != persp_depth_right.view(np.int32))),
            "seg": int(np.count_nonzero(np.any(persp_seg_left != persp_seg_right, axis=-1)))}
    return counts


def refit_scene_before_render(m, d, rc) -> None:
    """Refit the scene BVH from the current device data immediately before a render.

    Minimal P0-1 fix (G-L1 material §0.2, same family as R06 H1): upstream
    `create_render_context` builds the scene BVH from a CPU snapshot taken at
    construction time, so per-world device poses must be refit into the BVH
    before rendering. `refit_bvh` launches bounds over `(d.nworld, bvh_ngeom)`
    and reads `d.geom_xpos`/`d.geom_xmat`, i.e. one call covers all worlds.
    """
    import mujoco_warp as mjw

    mjw.refit_bvh(m, d, rc)


def verify_capture_metadata(capture: dict) -> None:
    """Post-write self-check: non-empty command/captured_at/reference_tag and new scope wording."""
    command = capture.get("command") or {}
    if not command.get("argv") or not str(command.get("rendered") or "").strip():
        fail("capture.json command is empty (spec v2 §11.1 item 1)")
    captured_at = str(capture.get("captured_at") or "").strip()
    if not captured_at:
        fail("capture.json captured_at is empty (spec v2 §11.1 item 2)")
    try:
        datetime.datetime.fromisoformat(captured_at.replace("Z", "+00:00"))
    except ValueError:
        fail(f"capture.json captured_at is not ISO8601: {captured_at!r}")
    if not str(capture.get("reference_tag") or "").strip():
        fail("capture.json reference_tag is empty (spec v2 §11.1 item 4)")
    scope = capture.get("per_world_scope") or {}
    covers = str(scope.get("covers") or "")
    does_not_cover = str(scope.get("does_not_cover") or "")
    if "per-world light poses" not in covers or "geom poses" not in covers:
        fail("capture.json per_world_scope.covers does not state per-world light + geom coverage")
    if OLD_SCOPE_DOES_NOT_COVER in does_not_cover:
        fail("capture.json per_world_scope.does_not_cover still carries the old light-poses wording")
    if not str(capture.get("recipe", {}).get("task_id") or "").strip():
        fail("capture.json recipe.task_id is missing (task-id provenance source)")
    for key in ("c1_rgb_changed_px", "c2_depth_changed_px", "c2_seg_changed_px",
                "campersp_depth_seg_changed_px", "camhinge_rgb_changed_px"):
        value = capture.get(key)
        if not isinstance(value, dict) or sorted(value) != ["0->1", "1->2"]:
            fail(f"capture.json {key} is missing or does not carry the 0->1 / 1->2 world pairs (spec v3 §2.4)")
    for key in ("c1_rgb_changed_px", "camhinge_rgb_changed_px"):
        for pair, value in capture[key].items():
            if not isinstance(value, dict) or sorted(value) != ["bitwise", "magnitude"]:
                fail(f"capture.json {key}[{pair}] must carry bitwise + magnitude counts (spec v3 §2.4)")
    for pair, value in capture["campersp_depth_seg_changed_px"].items():
        if not isinstance(value, dict) or sorted(value) != ["depth", "seg"]:
            fail(f"capture.json campersp_depth_seg_changed_px[{pair}] must carry depth + seg counts")


def _case(name: str, body) -> tuple[str, bool]:
    try:
        body()
        return name, True
    except SystemExit as error:
        print(f"FAIL {name}: unexpected exit {error}")
        return name, False


def _case_expect_exit(name: str, body, needle: str) -> tuple[str, bool]:
    try:
        body()
    except SystemExit as error:
        text = str(error)
        if needle in text:
            return name, True
        print(f"FAIL {name}: exit message missing {needle!r}: {text}")
        return name, False
    print(f"FAIL {name}: expected a non-zero exit")
    return name, False


def selfcheck() -> int:
    """CPU positive/negative cases for the four prerequisites (no GPU, no capture)."""
    iso = "2026-11-07T13:00:00+08:00"
    results = [
        _case("command-positive", lambda: command_line([sys.executable, "scripts/capture_render_perworld.py", "--dry-run"], pathlib.Path.cwd())),
        _case_expect_exit("command-negative-empty-argv", lambda: command_line([], pathlib.Path.cwd()), "argv is empty"),
        _case("captured-at-positive", lambda: require_capture_at({"MJWARP_CAPTURE_AT": iso})),
        _case_expect_exit("captured-at-negative-missing", lambda: require_capture_at({}), "is missing"),
        _case_expect_exit("captured-at-negative-empty", lambda: require_capture_at({"MJWARP_CAPTURE_AT": "   "}), "is empty"),
        _case_expect_exit("captured-at-negative-not-iso", lambda: require_capture_at({"MJWARP_CAPTURE_AT": "not-a-time"}), "not ISO8601"),
        _case("scope-positive-new-wording", lambda: _scope_asserts()),
        _case_expect_exit("scope-negative-old-wording", lambda: _scope_old_negative(), "old light-poses wording"),
        _case("reference-tag-positive-from-dir", lambda: _tag_from_dir()),
        _case("reference-tag-positive-cli-override", lambda: _tag_override()),
        _case_expect_exit("reference-tag-negative-empty-dir", lambda: reference_tag(None, ""), "cannot derive reference_tag"),
        _case("capture-metadata-positive", lambda: verify_capture_metadata(_synthetic_capture(iso))),
        _case_expect_exit("capture-metadata-negative-empty-command", lambda: verify_capture_metadata(_synthetic_capture(iso, command={"argv": [], "rendered": ""})), "command is empty"),
        _case_expect_exit("capture-metadata-negative-old-scope", lambda: verify_capture_metadata(_synthetic_capture(iso, scope={"covers": SCOPE_COVERS, "does_not_cover": OLD_SCOPE_DOES_NOT_COVER})), "old light-poses wording"),
        _case("discriminability-positive", lambda: _discriminability_case()),
        _case_expect_exit("discriminability-negative-missing-array", lambda: _discriminability_negative(), "missing"),
    ]
    passed = sum(1 for _, ok in results if ok)
    print(f"SELFCHECK {'PASS' if passed == len(results) else 'FAIL'} {passed}/{len(results)}")
    return 0 if passed == len(results) else 1


def _scope_asserts() -> None:
    scope = per_world_scope()
    if "per-world light poses" not in scope["covers"] or "geom poses" not in scope["covers"]:
        fail("new scope wording misses light/geom coverage")
    if OLD_SCOPE_DOES_NOT_COVER in scope["does_not_cover"]:
        fail("new scope still carries the old light-poses wording")


def _scope_old_negative() -> None:
    legacy = {"covers": "per-world geom poses", "does_not_cover": OLD_SCOPE_DOES_NOT_COVER}
    if OLD_SCOPE_DOES_NOT_COVER in legacy["does_not_cover"]:
        fail("old light-poses wording is still present in the legacy shape")


def _tag_from_dir() -> None:
    if reference_tag(None, "fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040") != "nw3-light-hinge02040":
        fail("reference_tag derivation from directory name failed")


def _tag_override() -> None:
    if reference_tag("tag-x", "fixtures/whatever") != "tag-x":
        fail("reference_tag CLI override failed")


def _synthetic_counts() -> dict:
    pair = {"0->1": {"magnitude": 1, "bitwise": 2}, "1->2": {"magnitude": 0, "bitwise": 0}}
    return {"c1_rgb_changed_px": pair, "camhinge_rgb_changed_px": pair,
            "c2_depth_changed_px": {"0->1": 1, "1->2": 0},
            "c2_seg_changed_px": {"0->1": 1, "1->2": 0},
            "campersp_depth_seg_changed_px": {"0->1": {"depth": 0, "seg": 0}, "1->2": {"depth": 0, "seg": 0}}}


def _discriminability_case() -> None:
    import numpy as np

    arrays = {}
    for world in range(3):
        for cam in (0, 1):
            arrays[f"rgb_w{world}_cam{cam}"] = np.zeros((2, 2, 3), dtype=np.float32)
            arrays[f"depth_w{world}_cam{cam}"] = np.zeros((2, 2), dtype=np.float32)
            arrays[f"seg_w{world}_cam{cam}"] = np.full((2, 2, 2), -1, dtype=np.int32)
    # cam-persp：像素(0,0) 蓝 +1/255（magnitude+bitwise 都计）；像素(1,0) 254/255 -> 1.0 饱和步进（只 bitwise）。
    arrays["rgb_w0_cam0"][1, 0] = [1.0, 0.0, 0.0]
    arrays["rgb_w1_cam0"][0, 0] = [0.0, 0.0, 1.0 / 255.0]
    arrays["rgb_w1_cam0"][1, 0] = [254.0 / 255.0, 0.0, 0.0]
    # cam-hinge：depth 位型 + seg pair 变化（诊断项 cam-persp depth/seg 保持 0）。
    arrays["depth_w1_cam1"][1, 1] = np.float32(1.0)
    arrays["seg_w1_cam1"][0, 1] = np.array([3, 5], dtype=np.int32)
    for cam in (0, 1):
        for kind in ("rgb", "depth", "seg"):
            arrays[f"{kind}_w2_cam{cam}"] = arrays[f"{kind}_w1_cam{cam}"].copy()
    expected = {"c1_rgb_changed_px": {"0->1": {"magnitude": 1, "bitwise": 2}, "1->2": {"magnitude": 0, "bitwise": 0}},
                "camhinge_rgb_changed_px": {"0->1": {"magnitude": 0, "bitwise": 0}, "1->2": {"magnitude": 0, "bitwise": 0}},
                "c2_depth_changed_px": {"0->1": 1, "1->2": 0},
                "c2_seg_changed_px": {"0->1": 1, "1->2": 0},
                "campersp_depth_seg_changed_px": {"0->1": {"depth": 0, "seg": 0}, "1->2": {"depth": 0, "seg": 0}}}
    counts = discriminability_counts(arrays)
    if counts != expected:
        fail(f"discriminability counts mismatch: {counts} != {expected}")


def _discriminability_negative() -> None:
    discriminability_counts({"rgb_w0_cam0": None})


def _synthetic_capture(captured_at: str, command=None, scope=None) -> dict:
    return {
        "command": command if command is not None else command_line([sys.executable, "script.py"], pathlib.Path.cwd()),
        "captured_at": captured_at,
        "reference_tag": "nw3-light-hinge02040",
        "per_world_scope": scope if scope is not None else per_world_scope(),
        "recipe": {"task_id": TASK_ID},
        **_synthetic_counts(),
    }


if __name__ == "__main__":
    raise SystemExit(selfcheck())
