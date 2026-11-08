#!/usr/bin/env python3
"""G-L0 pilot readout for per-world L v2 light (signed spec v3 section 6): zero GPU, authoring only.

Computes, on the reference side (the pilot capture output):
  C1 (light-driven, cam-persp): adjacent-world pixels with max_channel |delta rgb| >= 1/255
  C2 (geometry-driven, cam-hinge): adjacent-world depth bit-changed pixels and seg pair-changed pixels
Verdict: both sets and both pairs >= 576 => pass-margin; any in [384,576) => insufficient;
any < 384 => non-discriminating.

Also performs the post-capture metadata self-check required by spec v3 section 9.3 item 4
(command / captured_at / reference_tag / per_world_scope / cam_active / context.resolved /
recipe.cameras 10-field two-camera snapshot), prints per-world summaries and per-world visible
seg (id, type) sets, and verifies the two-capture bitwise flag.

This script is authoring-only and must never be imported by product code.

Usage:
  python -B readout_c1_c2.py --pilot-dir <dir> [--json-out <path>]
  python -B readout_c1_c2.py --selftest

Exit codes: 0 pass-margin; 3 insufficient or non-discriminating; 2 input/metadata error.
"""

import argparse
import json
import pathlib
import sys

import numpy as np

THRESH_RGB = 1.0 / 255.0
ACCEPT = 384
MARGIN = 576
WORLDS = (0, 1, 2)
PAIRS = ((0, 1), (1, 2))
CAM0 = 0  # cam-persp (lighting view)
CAM1 = 1  # cam-hinge (geometry view)
CAMERA_RECIPE_KEYS = ("slot", "mujoco_cam_id", "name", "mode", "xpos", "xquat",
                      "fovy", "resolution", "sensorsize", "intrinsic")


def load_arrays(pilot_dir: pathlib.Path) -> dict:
    arrays = {}
    for kind in ("rgb", "depth", "seg"):
        path = pilot_dir / f"{kind}.npz"
        with np.load(path) as npz:
            for name in npz.files:
                arrays[name] = npz[name]
    return arrays


def c1_counts(arrays: dict) -> dict:
    out = {}
    for a, b in PAIRS:
        x = arrays[f"rgb_w{a}_cam{CAM0}"]
        y = arrays[f"rgb_w{b}_cam{CAM0}"]
        delta = np.abs(x.astype(np.float64) - y.astype(np.float64)).max(axis=-1)
        bitwise = np.any(x.view(np.int32) != y.view(np.int32), axis=-1)
        out[f"{a}->{b}"] = {
            "rgb_changed_px": int(np.count_nonzero(delta >= THRESH_RGB)),
            "rgb_bitwise_changed_px": int(np.count_nonzero(bitwise)),
        }
    return out


def c2_counts(arrays: dict) -> dict:
    out = {}
    for a, b in PAIRS:
        d0, d1 = arrays[f"depth_w{a}_cam{CAM1}"], arrays[f"depth_w{b}_cam{CAM1}"]
        s0, s1 = arrays[f"seg_w{a}_cam{CAM1}"], arrays[f"seg_w{b}_cam{CAM1}"]
        out[f"{a}->{b}"] = {
            "depth_changed_px": int(np.count_nonzero(d0.view(np.int32) != d1.view(np.int32))),
            "seg_changed_px": int(np.count_nonzero(np.any(s0 != s1, axis=-1))),
        }
    return out


def diagnostics(arrays: dict) -> dict:
    out = {"cam_persp_depth_seg": {}, "cam_hinge_rgb": {}}
    for a, b in PAIRS:
        d0, d1 = arrays[f"depth_w{a}_cam{CAM0}"], arrays[f"depth_w{b}_cam{CAM0}"]
        s0, s1 = arrays[f"seg_w{a}_cam{CAM0}"], arrays[f"seg_w{b}_cam{CAM0}"]
        out["cam_persp_depth_seg"][f"{a}->{b}"] = {
            "depth_changed_px": int(np.count_nonzero(d0.view(np.int32) != d1.view(np.int32))),
            "seg_changed_px": int(np.count_nonzero(np.any(s0 != s1, axis=-1))),
        }
        x = arrays[f"rgb_w{a}_cam{CAM1}"]
        y = arrays[f"rgb_w{b}_cam{CAM1}"]
        delta = np.abs(x.astype(np.float64) - y.astype(np.float64)).max(axis=-1)
        out["cam_hinge_rgb"][f"{a}->{b}"] = {
            "rgb_changed_px": int(np.count_nonzero(delta >= THRESH_RGB)),
            "rgb_bitwise_changed_px": int(np.count_nonzero(np.any(x.view(np.int32) != y.view(np.int32), axis=-1))),
        }
    return out


def per_world_summary(arrays: dict) -> dict:
    out = {}
    for cam in (CAM0, CAM1):
        for w in WORLDS:
            rgb = arrays[f"rgb_w{w}_cam{cam}"]
            depth = arrays[f"depth_w{w}_cam{cam}"]
            seg = arrays[f"seg_w{w}_cam{cam}"]
            hit = depth > 0
            pairs = np.unique(seg.reshape(-1, 2), axis=0)
            out[f"world{w}_cam{cam}"] = {
                "rgb_mean": round(float(rgb.mean()), 6),
                "rgb_min": round(float(rgb.min()), 6),
                "rgb_max": round(float(rgb.max()), 6),
                "depth_hit_px": int(np.count_nonzero(hit)),
                "depth_min_hit": round(float(depth[hit].min()), 6) if hit.any() else None,
                "depth_max_hit": round(float(depth[hit].max()), 6) if hit.any() else None,
                "seg_pairs": [[int(i), int(t)] for i, t in pairs],
            }
    return out


def verdict(c1: dict, c2: dict) -> dict:
    c1_vals = [v["rgb_changed_px"] for v in c1.values()]
    depth_vals = [v["depth_changed_px"] for v in c2.values()]
    seg_vals = [v["seg_changed_px"] for v in c2.values()]
    all_pass_margin = all(v >= MARGIN for v in c1_vals + depth_vals + seg_vals)
    any_below_accept = any(v < ACCEPT for v in c1_vals + depth_vals + seg_vals)
    if all_pass_margin:
        name = "pass-margin"
    elif any_below_accept:
        name = "non-discriminating"
    else:
        name = "insufficient"
    return {
        "verdict": name,
        "thresholds": {"accept": ACCEPT, "collection_margin": MARGIN, "rgb_magnitude": THRESH_RGB},
        "all_sets_pass_margin": all_pass_margin,
        "any_below_accept": any_below_accept,
        "c1_values": c1_vals,
        "c2_depth_values": depth_vals,
        "c2_seg_values": seg_vals,
        "exit_code": 0 if name == "pass-margin" else 3,
    }


def is_iso8601(value: str) -> bool:
    import datetime
    try:
        datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))
        return True
    except ValueError:
        return False


def metadata_selfcheck(pilot_dir: pathlib.Path) -> dict:
    capture = json.loads((pilot_dir / "capture.json").read_text(encoding="utf-8"))
    checks = {}
    cmd = capture.get("command") or {}
    checks["command_non_empty"] = bool(cmd.get("argv"))
    at = capture.get("captured_at") or ""
    checks["captured_at_non_empty_iso8601"] = bool(at) and is_iso8601(at)
    checks["reference_tag"] = capture.get("reference_tag")
    checks["reference_tag_is_tag"] = capture.get("reference_tag") == "nw3-light-hinge02040"
    scope = capture.get("per_world_scope") or {}
    covers = " ".join(scope.get("covers", "")) if isinstance(scope.get("covers"), str) else " ".join(scope.get("covers", []))
    does_not = json.dumps(scope.get("does_not_cover", []), ensure_ascii=False)
    checks["scope_covers_light_and_geom"] = ("per-world light poses" in covers) and ("per-world geom poses" in covers)
    checks["scope_no_old_wording"] = "device path indexes lights single-world" not in does_not
    checks["cam_active"] = (capture.get("parameters") or {}).get("cam_active")
    checks["cam_active_expected"] = checks["cam_active"] == [True, True, False]
    resolved = (capture.get("context") or {}).get("resolved") or {}
    checks["nrender"] = resolved.get("nrender")
    checks["nrender_expected"] = resolved.get("nrender") == 2
    checks["cam_id_map"] = resolved.get("cam_id_map")
    checks["cam_id_map_expected"] = resolved.get("cam_id_map") == [0, 1]
    checks["cam_res"] = resolved.get("cam_res")
    checks["cam_res_expected"] = resolved.get("cam_res") == [[320, 240], [320, 240]]
    cameras = (capture.get("recipe") or {}).get("cameras") or []
    checks["recipe_cameras_count"] = len(cameras)
    checks["recipe_cameras_expected"] = len(cameras) == 2
    checks["recipe_cameras_10_fields"] = all(tuple(sorted(c.keys())) == tuple(sorted(CAMERA_RECIPE_KEYS)) for c in cameras)
    checks["camera_names"] = [c.get("name") for c in cameras]
    checks["camera_names_expected"] = checks["camera_names"] == ["cam-persp", "cam-hinge"]
    checks["cam_hinge_fovy_path"] = bool(cameras) and cameras[-1].get("sensorsize") == [0.0, 0.0]
    checks["two_captures_bitwise_identical"] = capture.get("two_captures_bitwise_identical")
    checks["files_sha256_set"] = sorted((capture.get("files_sha256") or {}).keys())
    checks["files_sha256_expected"] = checks["files_sha256_set"] == ["depth.npz", "rgb.npz", "seg.npz", "state.npz"]
    failed = [k for k, v in checks.items() if k.endswith("_expected") and v is not True]
    failed += [k for k in ("command_non_empty", "captured_at_non_empty_iso8601", "scope_covers_light_and_geom",
                           "scope_no_old_wording", "recipe_cameras_10_fields", "cam_hinge_fovy_path",
                           "two_captures_bitwise_identical") if checks.get(k) is not True]
    return {"checks": checks, "failed": failed, "ok": not failed}


def run_readout(pilot_dir: pathlib.Path) -> dict:
    arrays = load_arrays(pilot_dir)
    needed = [f"{kind}_w{w}_cam{c}" for kind in ("rgb", "depth", "seg") for w in WORLDS for c in (CAM0, CAM1)]
    missing = [n for n in needed if n not in arrays]
    if missing:
        return {"error": f"missing pilot arrays: {missing}"}
    c1 = c1_counts(arrays)
    c2 = c2_counts(arrays)
    meta = metadata_selfcheck(pilot_dir)
    return {
        "schema": 1,
        "pilot_dir": pilot_dir.as_posix(),
        "c1_cam_persp": c1,
        "c2_cam_hinge": c2,
        "diagnostics_no_threshold": diagnostics(arrays),
        "per_world_summary": per_world_summary(arrays),
        "metadata_selfcheck": meta,
        "verdict": verdict(c1, c2),
    }


def _synth(c1_px=0, c2_px=0, ulp_only_px=0, c1_only=False):
    arrays = {}
    base_rgb = np.full((240, 320, 3), 0.5, dtype=np.float32)
    base_depth = np.ones((240, 320), dtype=np.float32)
    base_seg = np.zeros((240, 320, 2), dtype=np.int32)
    for w in WORLDS:
        for c in (CAM0, CAM1):
            arrays[f"rgb_w{w}_cam{c}"] = base_rgb.copy()
            arrays[f"depth_w{w}_cam{c}"] = base_depth.copy()
            arrays[f"seg_w{w}_cam{c}"] = base_seg.copy()
    flat = 240 * 320

    def mark(arr_name, value, count, channel=None):
        arr = arrays[arr_name]
        if arr.ndim == 3 and arr.shape[-1] == 3:
            cols = np.arange(count)
            if channel is None:
                arr.reshape(-1, 3)[:count] = value
            else:
                arr.reshape(-1, 3)[:count, channel] = value
        elif arr.ndim == 3:
            arr.reshape(-1, 2)[:count] = value
        else:
            arr.reshape(-1)[:count] = value
        return arr

    if c1_px:
        mark(f"rgb_w1_cam{CAM0}", np.float32(0.5 + 0.01), c1_px)
        mark(f"rgb_w2_cam{CAM0}", np.float32(0.5 + 0.02), c1_px)
    if ulp_only_px:
        arr = arrays[f"rgb_w1_cam{CAM0}"]
        view = arr.reshape(-1)
        view[:ulp_only_px] = np.nextafter(view[:ulp_only_px], np.float32(1.0))
    if c2_px:
        mark(f"depth_w1_cam{CAM1}", np.float32(2.0), c2_px)
        mark(f"depth_w2_cam{CAM1}", np.float32(3.0), c2_px)
        mark(f"seg_w1_cam{CAM1}", [1, 5], c2_px)
        mark(f"seg_w2_cam{CAM1}", [2, 5], c2_px)
    if c1_only:
        mark(f"rgb_w1_cam{CAM0}", np.float32(0.5 + 0.01), 2000)
        mark(f"rgb_w2_cam{CAM0}", np.float32(0.5 + 0.02), 2000)
    assert sorted(arrays) == sorted([f"{k}_w{w}_cam{c}" for k in ("rgb", "depth", "seg")
                                     for w in WORLDS for c in (CAM0, CAM1)])
    return arrays


def selftest() -> int:
    cases = [
        ("pass-both-sets", dict(c1_px=700, c2_px=700), "pass-margin", 0),
        ("insufficient-margin", dict(c1_px=500, c2_px=700), "insufficient", 3),
        ("non-discriminating", dict(c1_px=100, c2_px=700), "non-discriminating", 3),
        ("ulp-only-rejected", dict(ulp_only_px=4000, c2_px=700), "non-discriminating", 3),
        ("identical-zero", dict(), "non-discriminating", 3),
        ("c1-only-rejected", dict(c1_only=True), "non-discriminating", 3),
    ]
    failures = 0
    for name, kwargs, expect_verdict, expect_exit in cases:
        arrays = _synth(**kwargs)
        c1 = c1_counts(arrays)
        c2 = c2_counts(arrays)
        v = verdict(c1, c2)
        ok = v["verdict"] == expect_verdict and v["exit_code"] == expect_exit
        failures += 0 if ok else 1
        print(f"{'PASS' if ok else 'FAIL'} {name}: verdict={v['verdict']} exit={v['exit_code']} "
              f"c1={v['c1_values']} depth={v['c2_depth_values']} seg={v['c2_seg_values']}")
    print(f"SUMMARY {'PASS' if failures == 0 else 'FAIL'} {len(cases) - failures}/{len(cases)}")
    return 0 if failures == 0 else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--pilot-dir", type=pathlib.Path)
    mode.add_argument("--selftest", action="store_true")
    parser.add_argument("--json-out", type=pathlib.Path, default=None)
    args = parser.parse_args()
    if args.selftest:
        return selftest()
    result = run_readout(args.pilot_dir)
    if "error" in result:
        print(json.dumps(result, ensure_ascii=False, indent=1))
        return 2
    text = json.dumps(result, ensure_ascii=False, indent=1)
    print(text)
    if args.json_out:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(text + "\n", encoding="utf-8", newline="\n")
    if not result["metadata_selfcheck"]["ok"]:
        return 2
    return result["verdict"]["exit_code"]


if __name__ == "__main__":
    raise SystemExit(main())
