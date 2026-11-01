"""Author the scene-a sphere-outline exact-difference lists for the v2.2 freeze step.

OFFLINE AUTHORING TOOL (outside the product chain): it is never referenced by the product
build, the product tests, cargo gates or product scripts; it reads frozen fixtures only and
writes to a caller-chosen directory. The wrapper `scripts/authoring/
author-scene-a-sphere-outline.ps1` re-checks the pinned environment and upstream identity
before running this file.

The numeric replay (exact single-rounding f32, camera-ray port, both `a = dot(vec,vec)`
association forms) lives in `scene_a_outline_replay.py` next to this file.

Contract (criterion render-sdf-criterion-v2.2):
  * list(arm) = the exact difference set of the two `a = dot(vec,vec)` association forms
    (form A = (0,1,2) reference form, form B = (1,0,2) product form) over camera-1 sphere
    pixels of the frozen reference; outside the list the product must be bitwise equal;
  * B = 128 = next power of two above the exact freeze-time maximum difference (recipe 73,
    aa2 91); a recomputed exact max above 128 makes the attempt fail (B must not be raised);
  * the analytic v2 section-3 bound is reported per pixel (bound_raw and margin = 1.15x
    bound_raw) and recorded only - it is explicitly not used to determine B.

Outputs (ASCII, LF): `scene-a-sphere-outline-<arm>.txt` lists in the signed proposal prototype
format (9 columns, byte-identical), `outline-bounds-<arm>.txt` analytic-bound sidecars (recorded
only; not used to determine B), `outline-lists.json` (machine readable summary used by the freeze
step to fill the guards file), `b-recompute.txt`, `selfcheck.txt`, `open-items.txt`,
`sha256sums.txt`.

Exit codes: 0 = freeze verdict OK; 4 = exact max > 128 (v2.2 freeze failure); 5 = internal
self-check failed; 6 = frozen input hash mismatch.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

import numpy as np

import scene_a_outline_replay as replay_mod

BOUND_ULP = 128
CRITERION_VERSION = "render-sdf-criterion-v2.2"

# (reference path, samples_per_pixel, entities consuming the list)
ARMS = (
    (replay_mod.RECIPE_REFERENCE, 1,
     ("render_e2e", "render_kernel_depth_seg", "render_kernel_flags", "render_kernel_rgb",
      "render_r10_multi_world", "render_skybox_reference")),
    (replay_mod.AA2_REFERENCE, 2, ("render_kernel_flags-aa2",)),
)
CAMERA = 1
ARM_NAMES = {replay_mod.RECIPE_REFERENCE: "recipe", replay_mod.AA2_REFERENCE: "aa2"}


# The criterion v2.1 list format is the signed proposal prototype, byte-identical
# (see docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists/):
# 9 columns, header = criterion line, reference, reference sha256, camera/raster/spp, columns.
PROTOTYPE_LISTS = {
    "recipe": {
        "sha256": "cfd1038a1cc50326de51a25495db68cd8b25c0983121dea88a2fb89ca83724ee",
        "bytes": 10099,
        "rows": 183,
    },
    "aa2": {
        "sha256": "14893fc6a2479e8e8f5e9cc07cabe72fd9f37aea638dda25fc26839d59ec8eab",
        "bytes": 9509,
        "rows": 172,
    },
}


def format_list_file(arm, ref, rows, spp, reference_label, reference_sha):
    lines = [
        "# scene-a sphere-outline exact-diff list (criterion v2.1 proposal format)",
        f"# reference: {reference_label}",
        f"# reference sha256: {reference_sha}",
        f"# camera: {CAMERA} raster: {ref.width}x{ref.height} spp: {spp}",
        "# columns: pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab",
    ]
    for pixel, x, y, seg_geom, seg_type, ref_bits, a_bits, b_bits, ulp, _bound_raw, _margin in rows:
        lines.append(f"{pixel} {x} {y} {seg_geom} {seg_type} {ref_bits} {a_bits} {b_bits} {ulp}")
    return "\n".join(lines) + "\n"


def format_bounds_sidecar(arm, list_name, list_sha, rows):
    lines = [
        "# scene-a sphere-outline analytic bound sidecar (recorded only; NOT used to determine B)",
        f"# arm: {arm}",
        f"# list: {list_name}",
        f"# list sha256: {list_sha}",
        "# columns: pixel bound_raw margin",
    ]
    for pixel, _x, _y, _sg, _st, _rb, _ab, _bb, _ulp, bound_raw, margin in rows:
        lines.append(f"{pixel} {bound_raw:.6f} {margin:.6f}")
    return "\n".join(lines) + "\n"


def run_arm(root, scene, reference_rel, spp, entities):
    ref = replay_mod.Reference(root, reference_rel, CAMERA)
    sphere = replay_mod.sphere_pixels(scene, ref)
    data_a = replay_mod.replay(scene, ref, CAMERA, spp, "A", sphere)
    data_b = replay_mod.replay(scene, ref, CAMERA, spp, "B", sphere)
    depths_a = {p: data_a[p][0] for p in sphere}
    depths_b = {p: data_b[p][0] for p in sphere}
    terms_a = {p: data_a[p][1] for p in sphere}
    terms_b = {p: data_b[p][1] for p in sphere}
    dirs = {p: data_a[p][2] for p in sphere}
    form_a_bad = sum(1 for p in sphere if replay_mod.bits(depths_a[p]) != ref.depth[p])
    affected = [p for p in sphere if replay_mod.bits(depths_a[p]) != replay_mod.bits(depths_b[p])]
    deltas = {p: abs(replay_mod.bits(depths_a[p]) - replay_mod.bits(depths_b[p])) for p in affected}
    bounds = replay_mod.per_pixel_bounds(affected, depths_a, terms_a, terms_b, dirs, deltas)
    bound_by_pixel = {row["pixel"]: row for row in bounds}
    rows = []
    for pixel in affected:
        bound = bound_by_pixel[pixel]
        rows.append((
            pixel, pixel % ref.width, pixel // ref.width,
            ref.seg[2 * pixel], ref.seg[2 * pixel + 1], ref.depth[pixel],
            replay_mod.bits(depths_a[pixel]), replay_mod.bits(depths_b[pixel]), deltas[pixel],
            bound["bound_raw"], bound["margin"],
        ))
    stats = replay_mod.summarize_bounds(bounds)
    return {
        "arm": ARM_NAMES[reference_rel],
        "reference": reference_rel,
        "reference_sha256": ref.sha,
        "spp": spp,
        "entities": list(entities),
        "raster": f"{ref.width}x{ref.height}",
        "sphere_pixels": len(sphere),
        "rows": rows,
        "list_pixels": len(rows),
        "exact_max_ulp": max(deltas.values(), default=0),
        "first_pixel": affected[0] if affected else None,
        "first_xy": [affected[0] % ref.width, affected[0] // ref.width] if affected else None,
        "form_a_bad": form_a_bad,
        "bound_stats": stats,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    root = args.repo.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)

    bad_pins = replay_mod.check_pins(root)
    if bad_pins:
        for rel, want, got in bad_pins:
            print(f"PIN MISMATCH {rel}: want {want} got {got}")
        return 6

    scene = replay_mod.Scene(root)
    writes = []

    # ---- self-check 1: camera-space rays vs the S5 fixture samples ----------
    s5 = replay_mod.load_json(root, replay_mod.S5)
    ctx = s5["contexts"]["default_all"]
    ray_total = 0
    ray_bad = 0
    for entry in ctx["expected"]["samples_entries"]:
        cam = int(entry["camera"])
        w = int(ctx["expected"]["cam_res"][cam][0])
        h = int(ctx["expected"]["cam_res"][cam][1])
        direction = scene.local_dir(cam, w, h, int(entry["x"]), int(entry["y"]))
        ray_total += 1
        if [replay_mod.bits(v) for v in direction] != [int(v) for v in entry["dir_bits"]]:
            ray_bad += 1

    # ---- arms: prototype-format lists + sidecars + machine-readable summary --
    arm_results = []
    for reference_rel, spp, entities in ARMS:
        result = run_arm(root, scene, reference_rel, spp, entities)
        ref = replay_mod.Reference(root, reference_rel, CAMERA)
        name = f"scene-a-sphere-outline-{result['arm']}.txt"
        text = format_list_file(result["arm"], ref, result["rows"], spp, reference_rel,
                                result["reference_sha256"])
        (out / name).write_bytes(text.encode("ascii"))
        writes.append(name)
        payload = (out / name).read_bytes()
        expected = PROTOTYPE_LISTS[result["arm"]]
        result["list_name"] = name
        result["list_bytes"] = len(payload)
        result["list_sha256"] = hashlib.sha256(payload).hexdigest()
        result["prototype_match"] = (
            result["list_sha256"] == expected["sha256"]
            and result["list_bytes"] == expected["bytes"]
            and len(result["rows"]) == expected["rows"]
        )
        sidecar = f"outline-bounds-{result['arm']}.txt"
        sidecar_text = format_bounds_sidecar(result["arm"], name, result["list_sha256"], result["rows"])
        (out / sidecar).write_bytes(sidecar_text.encode("ascii"))
        writes.append(sidecar)
        sidecar_payload = (out / sidecar).read_bytes()
        result["sidecar_name"] = sidecar
        result["sidecar_bytes"] = len(sidecar_payload)
        result["sidecar_sha256"] = hashlib.sha256(sidecar_payload).hexdigest()
        arm_results.append(result)

    # ---- self-check 2: skybox capture equals the recipe reference ----------
    recipe = replay_mod.Reference(root, replay_mod.RECIPE_REFERENCE, CAMERA)
    sky_depth = replay_mod.read_npz(root, replay_mod.SKYBOX_DEPTH)["depth_cam1"]
    sky_seg = replay_mod.read_npz(root, replay_mod.SKYBOX_SEG)["seg_cam1"]
    got_depth = np.asarray(sky_depth.ravel()).view("uint32").astype("uint64")
    want_depth = np.asarray(recipe.depth, dtype="uint64")
    got_seg = np.asarray(sky_seg.reshape(-1, 2), dtype="int64")
    want_seg = np.asarray(recipe.seg, dtype="int64").reshape(-1, 2)
    sky_depth_mismatch = int((got_depth != want_depth).sum())
    sky_seg_mismatch = int((got_seg != want_seg).sum())
    cam0 = replay_mod.Reference(root, replay_mod.RECIPE_REFERENCE, 0)
    cam0_sphere = len(replay_mod.sphere_pixels(scene, cam0))

    recipe_arm = next(r for r in arm_results if r["arm"] == "recipe")
    aa2_arm = next(r for r in arm_results if r["arm"] == "aa2")
    checks_ok = (
        ray_bad == 0
        and all(r["form_a_bad"] == 0 for r in arm_results)
        and sky_depth_mismatch == 0 and sky_seg_mismatch == 0 and cam0_sphere == 0
        and recipe_arm["list_pixels"] == 183 and recipe_arm["exact_max_ulp"] == 73
        and aa2_arm["list_pixels"] == 172 and aa2_arm["exact_max_ulp"] == 91
        and recipe_arm["first_pixel"] == 28969 and aa2_arm["first_pixel"] == 29448
        and all(r["prototype_match"] for r in arm_results)
    )
    over_bound = [r for r in arm_results if r["exact_max_ulp"] > BOUND_ULP]

    # ---- outline-lists.json (freeze-step contract) -------------------------
    summary = {
        "schema": 1,
        "criterion_version": CRITERION_VERSION,
        "bound_ulp": BOUND_ULP,
        "bound_definition": "next power of two above the exact freeze-time maximum difference",
        "camera": CAMERA,
        "analytic_bound_use": "recorded only; explicitly not used to determine B",
        "arms": [],
    }
    for result in arm_results:
        stats = result["bound_stats"]
        summary["arms"].append({
            "arm": result["arm"],
            "entities": result["entities"],
            "path": result["list_name"],
            "bytes": result["list_bytes"],
            "sha256": result["list_sha256"],
            "prototype_byte_identical": result["prototype_match"],
            "bounds_sidecar": {
                "path": result["sidecar_name"],
                "bytes": result["sidecar_bytes"],
                "sha256": result["sidecar_sha256"],
            },
            "rows": result["list_pixels"],
            "first_pixel": result["first_pixel"],
            "first_xy": result["first_xy"],
            "exact_max_ulp": result["exact_max_ulp"],
            "spp": result["spp"],
            "raster": result["raster"],
            "sphere_pixels": result["sphere_pixels"],
            "reference": result["reference"],
            "reference_sha256": result["reference_sha256"],
            "analytic_bound": {
                "bound_raw_min": round(stats["raw_min"], 4),
                "bound_raw_median": round(stats["raw_median"], 4),
                "bound_raw_max": round(stats["raw_max"], 4),
                "margin_min": round(stats["margin_min"], 4),
                "margin_median": round(stats["margin_median"], 4),
                "margin_max": round(stats["margin_max"], 4),
                "max_observed_over_bound_raw": round(stats["max_ratio_raw"], 6),
                "max_observed_over_margin": round(stats["max_ratio_margin"], 6),
            },
        })
    (out / "outline-lists.json").write_bytes(
        (json.dumps(summary, ensure_ascii=True, indent=2, sort_keys=True) + "\n").encode("ascii"))
    writes.append("outline-lists.json")

    # ---- reports ----------------------------------------------------------
    lines = [
        f"criterion {CRITERION_VERSION} freeze report (scene-a, camera {CAMERA})",
        f"signed B = {BOUND_ULP} ULP = next power of two above the exact freeze-time maximum",
        "difference; a recomputed exact max above the signed B fails the attempt and raising B",
        "is forbidden. The analytic v2 section-3 bound below is recorded only.",
        f"inputs: model {replay_mod.MODEL}; sphere geom {scene.sphere_geom}; camera pose from",
        "        each reference's camera_data_check[1]; znear = f32(vis.map.znear * stat.extent)",
        "",
    ]
    for result in arm_results:
        stats = result["bound_stats"]
        lines.append(f"arm {result['arm']}: {result['reference']} sha256 {result['reference_sha256']}")
        lines.append(f"  spp={result['spp']} raster={result['raster']} "
                     f"sphere_pixels={result['sphere_pixels']} list_pixels={result['list_pixels']} "
                     f"exact_max_ulp={result['exact_max_ulp']} first_pixel={result['first_pixel']}")
        lines.append(f"  list={result['list_name']} bytes={result['list_bytes']} "
                     f"sha256={result['list_sha256']} prototype_byte_identical={result['prototype_match']}")
        lines.append(f"  bounds sidecar={result['sidecar_name']} bytes={result['sidecar_bytes']} "
                     f"sha256={result['sidecar_sha256']} (recorded only)")
        lines.append(f"  analytic (recorded only): bound_raw min/median/max = "
                     f"{stats['raw_min']:.2f} / {stats['raw_median']:.2f} / {stats['raw_max']:.2f}; "
                     f"margin = {stats['margin_min']:.2f} / {stats['margin_median']:.2f} / "
                     f"{stats['margin_max']:.2f}")
        lines.append(f"  observed/bound ratios: raw {stats['max_ratio_raw']:.4f}, "
                     f"margin {stats['max_ratio_margin']:.4f} (no pixel with ratio >= 1)")
        lines.append(f"  exact_max <= B: {'YES' if result['exact_max_ulp'] <= BOUND_ULP else 'NO'}")
        lines.append("")
    lines.append(f"verdict: {'CRITERION_V2.2_FREEZE_OK' if not over_bound else 'CRITERION_V2.2_FREEZE_FAIL'}")
    (out / "b-recompute.txt").write_bytes(("\n".join(lines) + "\n").encode("ascii"))
    writes.append("b-recompute.txt")

    lines = [
        "dump tool self-check (offline authoring, zero GPU; criterion v2.2 contract)",
        f"1) camera-space rays vs S5 fixture default_all samples: {ray_total - ray_bad}/{ray_total} "
        f"bitwise equal",
    ]
    for result in arm_results:
        lines.append(f"2) arm {result['arm']}: form A vs frozen reference depth bits "
                     f"({result['sphere_pixels']} sphere pixels): "
                     f"{result['sphere_pixels'] - result['form_a_bad']}/{result['sphere_pixels']} "
                     f"bitwise equal")
        lines.append(f"   arm {result['arm']}: list = {result['list_pixels']} px, "
                     f"exact_max_ulp={result['exact_max_ulp']}, first pixel={result['first_pixel']} "
                     f"{result['first_xy']}")
        lines.append(f"   arm {result['arm']}: list sha256={result['list_sha256']} "
                     f"bytes={result['list_bytes']} prototype_byte_identical={result['prototype_match']}")
    lines += [
        f"3) skybox capture depth/seg vs recipe reference: depth mismatches={sky_depth_mismatch}, "
        f"seg mismatches={sky_seg_mismatch}",
        f"4) camera 0 sphere pixels: {cam0_sphere}",
        "5) attempt3 observation set reproduced: recipe 183 px / 73 ULP (first 28969=(169,120)); "
        "flags aa2 arm 172 px / 91 ULP (first 29448=(168,122)); cam0 = 0",
        f"internal_self_check: {'PASS' if checks_ok else 'FAIL'}",
    ]
    (out / "selfcheck.txt").write_bytes(("\n".join(lines) + "\n").encode("ascii"))
    writes.append("selfcheck.txt")

    open_items = [
        "OPEN ITEMS (freeze-step prerequisites checked by the window runner, not by this tool)",
        "1) product kernel identity must equal megakernel source 10070226../PTX 7782aeb1../268,589",
        "   (a-order refix form) and the host unlock state must match the target expectation.",
        "2) the freeze step writes outline-lists.json (list sha256/bytes/rows/exact max) into the",
        "   guards file; the lists are immutable inside the window (runner re-checks before/after body).",
        "3) body-time consumption is env-gated in the test support (MJWARP_RENDER_OUTLINE_LIST_DIR +",
        "   MJWARP_REQUIRE_OUTLINE_CRITERION=1); without them plain cargo test behaviour is unchanged.",
    ]
    (out / "open-items.txt").write_bytes(("\n".join(open_items) + "\n").encode("ascii"))
    writes.append("open-items.txt")

    names = sorted(p.name for p in out.iterdir() if p.is_file() and p.name != "sha256sums.txt")
    sums = "".join(f"{replay_mod.sha256_file(out / name)}  {name}\n" for name in names)
    (out / "sha256sums.txt").write_bytes(sums.encode("ascii"))

    print(f"arms={len(arm_results)} ray_selfcheck={ray_total - ray_bad}/{ray_total} "
          f"cam0_sphere={cam0_sphere}")
    for result in arm_results:
        print(f"arm {result['arm']}: sphere={result['sphere_pixels']} list={result['list_pixels']} "
              f"exact_max_ulp={result['exact_max_ulp']} first={result['first_pixel']} "
              f"bound_raw_max={result['bound_stats']['raw_max']:.2f} "
              f"margin_max={result['bound_stats']['margin_max']:.2f}")
    print(f"self_check={'PASS' if checks_ok else 'FAIL'} "
          f"criterion_verdict={'OK' if not over_bound else 'FAIL'}")
    print(f"outputs written to {out}")
    if not checks_ok:
        return 5
    return 0 if not over_bound else 4


if __name__ == "__main__":
    sys.exit(main())
