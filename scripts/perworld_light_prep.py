"""GPU-free prep helpers for per-world L v2 light authoring.

OFFLINE ONLY (authoring): imported by `scripts/capture_render_perworld.py` and
not by product builds/tests/gates. The dry-run builder assembles the plan JSON;
the light-pose precheck builds a CPU `mujoco.MjData`, runs `mj_forward` per
world and reads `light_xpos`/`light_xdir`. It creates **no** CUDA device,
renders nothing and writes no reference fixture.

Boundary: this module only prepares inputs for a possible per-world L v2 light
capture; it produces no reference and makes no rendering-discrimination claim.
"""

from __future__ import annotations

import pathlib

DEFAULT_SCENE = "fixtures/render-bvh/scene-a/scene.xml"
DEFAULT_OUT_ROOT = "fixtures/render-bvh/scene-a/perworld"
LIGHT_NAMES = ("sun", "lamp", "spot")
LIGHT_MIN_ANGLE_DEG = 1e-6
LIGHT_MIN_MOVE_M = 1e-6


def _rel(root: pathlib.Path, path: pathlib.Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def _angle_deg(a, b) -> float:
    import math

    dot = sum(x * y for x, y in zip(a, b))
    norm = math.sqrt(sum(x * x for x in a)) * math.sqrt(sum(y * y for y in b))
    if norm == 0.0:
        return 0.0
    return math.degrees(math.acos(max(-1.0, min(1.0, dot / norm))))


def light_pose_precheck(mjm, world_qpos) -> dict:
    """Per-world light poses from CPU mj_forward; difference keys + a plain verdict.

    No rendering, no CUDA device, no reference output. `world_qpos` is the
    per-world qpos matrix the capture would upload (host side only).
    """
    import mujoco

    nlight = int(mjm.nlight)
    names = [mjm.light(i).name or f"light{i}" for i in range(nlight)]
    kinds = [int(mjm.light_type[i]) for i in range(nlight)]
    bodies = [mjm.body(int(mjm.light_bodyid[i])).name or "world" for i in range(nlight)]
    data = mujoco.MjData(mjm)
    per_world = []
    for world, qpos in enumerate(world_qpos):
        data.qpos[:] = qpos
        mujoco.mj_forward(mjm, data)
        per_world.append({
            "world": world,
            "light_xpos": [[float(v) for v in data.light_xpos[i]] for i in range(nlight)],
            "light_xdir": [[float(v) for v in data.light_xdir[i]] for i in range(nlight)],
        })
    base = per_world[0]
    diffs = []
    for record in per_world[1:]:
        keys = []
        for idx, name in enumerate(names):
            if kinds[idx] == int(mujoco.mjtLightType.mjLIGHT_DIRECTIONAL):
                value = _angle_deg(base["light_xdir"][idx], record["light_xdir"][idx])
                keys.append({"light": name, "key": "xdir_angle_deg", "value": round(value, 9)})
            else:
                moved = [
                    record["light_xpos"][idx][axis] - base["light_xpos"][idx][axis]
                    for axis in range(3)
                ]
                value = sum(delta * delta for delta in moved) ** 0.5
                keys.append({"light": name, "key": "xpos_move_m", "value": round(value, 9)})
        diffs.append({"world": record["world"], "keys": keys})
    # Verdict (plain, no rendering-discrimination claim): at least one difference
    # key per further world is non-zero; zero keys are recorded with their reason.
    zero_notes = []
    for idx, name in enumerate(names):
        values = [
            next(key["value"] for key in diff["keys"] if key["light"] == name)
            for diff in diffs
        ]
        if all(value <= LIGHT_MIN_MOVE_M for value in values):
            zero_notes.append(
                f"{name} has zero xpos-move/xdir-angle under the hinge rotation "
                f"(light position/direction lies on the tri_body z/rotation axis; "
                f"scene-geometry fact, recorded only - no discrimination claim)"
            )
    worlds_nonzero = all(
        any(key["value"] > LIGHT_MIN_MOVE_M for key in diff["keys"]) for diff in diffs
    )
    return {
        "source": "CPU mujoco.MjData + mj_forward per world (no CUDA device, no render)",
        "lights": [
            {"name": names[i], "type": kinds[i], "attached_body": bodies[i]} for i in range(nlight)
        ],
        "world_hinge_deg": [float(v) for v in []],
        "per_world": per_world,
        "diff_keys": diffs,
        "verdict": {
            "nonzero": worlds_nonzero,
            "eps": {"angle_deg": LIGHT_MIN_ANGLE_DEG, "move_m": LIGHT_MIN_MOVE_M},
            "zero_notes": zero_notes,
            "expected_magnitude": "non-zero for off-axis lights (sun xdir angle > 1e-6 deg, "
                                  "lamp/spot pos move > 1e-6 m); on-axis lights are expected 0",
        },
    }


def build_dry_run_plan(*, root: pathlib.Path, task_id: str, recipe: dict, environment: dict,
                       mjm, world_qpos, world_deg, out_dir: pathlib.Path,
                       scene_path: pathlib.Path, out_root: pathlib.Path, repeat: int,
                       seed: int, warp_cache: str, legacy_manifest_entries: int) -> dict:
    """Assemble the dry-run plan; default scene/out-root keep the pre-change plan shape."""
    import mujoco

    default_scene = (root / DEFAULT_SCENE).resolve()
    default_out_root = (root / DEFAULT_OUT_ROOT).resolve()
    scene_explicit = scene_path.resolve() != default_scene
    out_root_explicit = out_root.resolve() != default_out_root
    plan = {
        "task_id": task_id, "dry_run": True,
        "recipe": {key: value for key, value in recipe.items() if key != "capture_at_env"},
        "paths": {
            "default_scene": DEFAULT_SCENE,
            "effective_scene": _rel(root, scene_path),
            "scene_explicit": scene_explicit,
            "default_out_root": DEFAULT_OUT_ROOT,
            "effective_out_root": _rel(root, out_root),
            "out_root_explicit": out_root_explicit,
            "out_dir": _rel(root, out_dir),
        },
        "out_dir": out_dir.resolve().as_posix(), "repeat": repeat, "seed": seed,
        "warp_cache": warp_cache,
        "model_static": {
            "njnt": int(mjm.njnt), "nq": int(mjm.nq),
            "jnt_type": [int(v) for v in mjm.jnt_type],
            "jnt_qposadr": [int(v) for v in mjm.jnt_qposadr],
            "qpos0": [float(v) for v in mjm.qpos0],
            "cameras": [[i, int(mjm.cam_mode[i]), mjm.camera(i).name or f"cam{i}"]
                        for i in range(int(mjm.ncam))],
        },
        "per_world_plan": {
            "nworld": len(world_qpos), "world_hinge_deg": world_deg,
            "world_qpos": [[float(v) for v in row] for row in world_qpos],
            "state_arrays": ["qpos", "geom_xpos", "geom_xmat", "light_xpos", "light_xdir"],
            "note": "lights are captured for completeness only; the device path still indexes "
                    "lights single-world, so per-world light poses are not claimed",
        },
        "cam_active_fixed_only": [int(mode) == 0 for mode in environment["camera_modes"]],
        "legacy_manifest_entries": legacy_manifest_entries,
        "environment": environment,
    }
    if scene_explicit:
        precheck = light_pose_precheck(mjm, world_qpos)
        precheck["world_hinge_deg"] = world_deg
        precheck["scene_xml"] = _rel(root, scene_path)
        plan["light_pose_precheck"] = precheck
    return plan
