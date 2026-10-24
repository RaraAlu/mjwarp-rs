"""Offline S0a camera-intrinsics sanity check (authoring helper).

OFFLINE ONLY: not imported or run by product builds/tests/gates; the product test
chain never starts Python. This guard exists because the first frozen scene-a
fixture set ``sensorsize`` on the perspective camera without a focal length /
principal point. MuJoCo then keeps ``cam_intrinsic`` at all zeros and leaves
``cam_fovy`` at its 180-degree sentinel, and MJWarp's ``compute_ray`` divides by
``fx = 0`` and emits zero-length rays, so that camera rendered pure background.
Model-array parity alone cannot catch this: both MuJoCo versions agree on the
broken values, so parity passed while the render path was unusable.

This check runs after ``model_parity.py`` in ``run_s0a.ps1`` and asserts, for
every camera of both compiled models: all intrinsics are finite, focal lengths
are positive, and every perspective camera has a real fovy inside (0, 180)
instead of the sentinel. It writes ``camera-sanity.json`` next to the parity
receipt and exits non-zero on any violation.

Usage:
  camera_sanity.py --fixture fixtures/render-bvh/scene-a \\
                   --evidence docs/development-evidence/render-bvh/s0a-reference-fixture
"""

from __future__ import annotations

import argparse
import json
import math
import pathlib

TASK_ID = "2026-10-24-render-bvh-s0a"
SCHEMA = 1
FOVY_SENTINEL = 180.0
PERSPECTIVE = 0
MODELS = (("312", "model-312.json"), ("315", "model-315.json"))


def load_arrays(path: pathlib.Path) -> dict[str, list[float]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    arrays = {}
    for name, entry in payload["arrays"].items():
        if name.startswith("_"):
            continue
        if not isinstance(entry, dict) or "values" not in entry:
            raise SystemExit(f"{path.name}: array {name} has an unexpected shape")
        values = entry["values"]
        if len(values) != entry["count"]:
            raise SystemExit(f"{path.name}: array {name} count/length mismatch")
        arrays[name] = values
    return arrays


def camera_records(arrays: dict[str, list[float]]) -> list[dict]:
    fovy = arrays["cam_fovy"]
    intrinsic = arrays["cam_intrinsic"]
    sensorsize = arrays["cam_sensorsize"]
    resolution = arrays["cam_resolution"]
    projection = arrays["cam_projection"]
    ncam = len(fovy)
    if len(intrinsic) != 4 * ncam or len(sensorsize) != 2 * ncam:
        raise SystemExit("camera array sizes disagree with cam_fovy count")
    records = []
    for cam in range(ncam):
        records.append(
            {
                "camera": cam,
                "projection": int(projection[cam]),
                "resolution": [int(resolution[2 * cam]), int(resolution[2 * cam + 1])],
                "sensorsize": [float(sensorsize[2 * cam]), float(sensorsize[2 * cam + 1])],
                "intrinsic": [float(v) for v in intrinsic[4 * cam : 4 * cam + 4]],
                "fovy_deg": float(fovy[cam]),
            }
        )
    return records


def check_camera(record: dict) -> list[str]:
    problems = []
    fx, fy, cx, cy = record["intrinsic"]
    label = f"camera {record['camera']}"
    for name, value in zip(("fx", "fy", "cx", "cy"), (fx, fy, cx, cy)):
        if not math.isfinite(value):
            problems.append(f"{label}: intrinsic {name} is not finite ({value!r})")
    if not (fx > 0.0 and fy > 0.0):
        problems.append(f"{label}: focal lengths must be positive (fx={fx!r}, fy={fy!r})")
    fovy = record["fovy_deg"]
    if not math.isfinite(fovy):
        problems.append(f"{label}: fovy is not finite ({fovy!r})")
    elif not (0.0 < fovy < FOVY_SENTINEL - 1e-9):
        problems.append(f"{label}: fovy {fovy!r} outside (0, {FOVY_SENTINEL}) sentinel range")
    if record["projection"] != PERSPECTIVE and not (0.0 < fovy < FOVY_SENTINEL - 1e-9):
        problems.append(f"{label}: non-perspective camera without a usable fovy ({fovy!r})")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description="S0a camera-intrinsics sanity check")
    parser.add_argument("--fixture", default="fixtures/render-bvh/scene-a")
    parser.add_argument("--evidence", default="docs/development-evidence/render-bvh/s0a-reference-fixture")
    args = parser.parse_args()
    fixture = pathlib.Path(args.fixture)
    evidence = pathlib.Path(args.evidence)
    evidence.mkdir(parents=True, exist_ok=True)
    report = {"schema": SCHEMA, "task_id": TASK_ID, "kind": "render-bvh-s0a-camera-sanity",
              "checks": ["intrinsic components finite", "fx>0 and fy>0",
                         f"0 < cam_fovy < {FOVY_SENTINEL} (180-degree sentinel rejected)"],
              "models": {}}
    problems = []
    for tag, name in MODELS:
        arrays = load_arrays(fixture / name)
        records = camera_records(arrays)
        model_problems = []
        for record in records:
            model_problems.extend(check_camera(record))
        report["models"][f"mujoco_{tag}"] = {
            "model_file": name,
            "cameras": records,
            "verdict": "pass" if not model_problems else "fail",
            "problems": model_problems,
        }
        problems.extend(f"mujoco {tag}: {problem}" for problem in model_problems)
    report["verdict"] = "pass" if not problems else "fail"
    report["problems"] = problems
    (evidence / "camera-sanity.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n"
    )
    if problems:
        for problem in problems:
            print(f"camera-sanity FAIL: {problem}")
        return 1
    print(f"camera-sanity PASS: {sum(len(v['cameras']) for v in report['models'].values())} camera records across 2 models")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
