"""Author the scene-a sphere-outline exemption lists for the U4 attempt4 freeze step.

OFFLINE AUTHORING TOOL (outside the product chain): it is never referenced by the product
build, the product tests, cargo gates or product scripts; it reads frozen fixtures only and
writes to a caller-chosen directory. The wrapper `scripts/authoring/
author-scene-a-sphere-outline.ps1` re-checks the pinned environment and upstream identity
before running this file.

The numeric replay (exact single-rounding f32, camera-ray port, both `a = dot(vec,vec)`
association forms) lives in `scene_a_outline_replay.py` next to this file. See that module's
docstring for the semantics and the aa2 sub-sample rule.

Outputs (ASCII, LF): per-entity lists `scene-a-sphere-outline-<entity>.txt`, the
criterion-v2 section-3 recomputation `b-recompute.txt`, `selfcheck.txt`, `open-items.txt`
and `sha256sums.txt`.

Exit codes: 0 = lists written and the criterion-v2 freeze verdict OK; 4 = lists written but
the criterion-v2 freeze verdict FAILS; 5 = internal self-check failed; 6 = frozen input
hash mismatch.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

import numpy as np

import scene_a_outline_replay as replay_mod

F32 = replay_mod.F32
U = replay_mod.U


# (output suffix, reference path, samples_per_pixel, camera index, entities sharing it).
# Camera 0 has no sphere hit pixels at all, so only camera 1 needs a list.
ARMS = (
    (replay_mod.RECIPE_REFERENCE,
     ("render_e2e", "render_kernel_depth_seg", "render_kernel_flags", "render_kernel_rgb",
      "render_r10_multi_world", "render_skybox_reference")),
    (replay_mod.AA2_REFERENCE, ("render_kernel_flags-aa2",)),
)
SPP = {replay_mod.RECIPE_REFERENCE: 1, replay_mod.AA2_REFERENCE: 2}
CAMERA = 1


def format_list_file(entity, ref, rows, reference_label, reference_sha):
    lines = [
        "# scene-a sphere-outline exemption list (criterion render-sdf-criterion-v2)",
        f"# entity: {entity}",
        f"# camera: {ref.index}  resolution: {ref.width}x{ref.height}  pixels: {ref.pixels}",
        f"# reference: {reference_label}",
        f"# reference sha256: {reference_sha}",
        "# replay: exact f32 product math (offline authoring tool, zero GPU)",
        "# form A = a-order (0,1,2) [reference form]; form B = a-order (1,0,2) [product form]",
        "# columns: pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_depth_bits",
        "#          formB_depth_bits ulp_ab outline_band4 derived_affected",
        "# outline_band4 = criterion v2 condition 2 (4-neighbour non-sphere hit);",
        "# derived_affected = form A and form B depth bits differ.",
        f"# rows: {len(rows)}",
    ]
    lines.extend(" ".join(str(v) for v in row) for row in rows)
    return "\n".join(lines) + "\n"


def run_arm(root, scene, reference_rel):
    ref = replay_mod.Reference(root, reference_rel, CAMERA)
    spp = SPP[reference_rel]
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
    affected_ulp = [abs(replay_mod.bits(depths_a[p]) - replay_mod.bits(depths_b[p]))
                    for p in affected]
    band = replay_mod.band4_pixels(scene, ref)
    band_set = set(band)
    affected_set = set(affected)
    covered = sum(1 for p in affected if p in band_set)
    rows = [(p, p % ref.width, p // ref.width, ref.seg[2 * p], ref.seg[2 * p + 1],
             ref.depth[p], replay_mod.bits(depths_a[p]), replay_mod.bits(depths_b[p]),
             abs(replay_mod.bits(depths_a[p]) - replay_mod.bits(depths_b[p])),
             p in band_set, p in affected_set)
            for p in sphere]
    return {
        "ref": ref, "reference_rel": reference_rel, "spp": spp,
        "rows": rows, "sphere": len(sphere), "form_a_bad": form_a_bad,
        "affected_count": len(affected), "max_ulp": max(affected_ulp, default=0),
        "band_count": len(band), "covered": covered,
        "band_bound": replay_mod.bound_v2(band, depths_a, terms_a, terms_b, dirs) if band else None,
        "affected_bound": replay_mod.bound_v2(affected, depths_a, terms_a, terms_b, dirs)
        if affected else None,
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

    # ---- per arm: replay, lists, self-checks --------------------------------
    arms = {}
    for reference_rel, entities in ARMS:
        result = run_arm(root, scene, reference_rel)
        arms[reference_rel] = result
        for entity in entities:
            name = f"scene-a-sphere-outline-{entity}.txt"
            (out / name).write_text(
                format_list_file(entity, result["ref"], result["rows"], reference_rel,
                                 result["ref"].sha),
                encoding="ascii", newline="\n")
            writes.append(name)

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

    checks_ok = (
        ray_bad == 0
        and all(r["form_a_bad"] == 0 for r in arms.values())
        and sky_depth_mismatch == 0 and sky_seg_mismatch == 0 and cam0_sphere == 0
    )
    recipe_result = arms[replay_mod.RECIPE_REFERENCE]
    aa2_result = arms[replay_mod.AA2_REFERENCE]
    if recipe_result["affected_count"] != 183 or recipe_result["max_ulp"] != 73:
        checks_ok = False
    if aa2_result["affected_count"] != 172 or aa2_result["max_ulp"] != 91:
        checks_ok = False

    # ---- criterion-v2 freeze verdict --------------------------------------
    verdict_ok = True
    for result in arms.values():
        if result["covered"] != result["affected_count"]:
            verdict_ok = False
        for bound in (result["band_bound"], result["affected_bound"]):
            if bound is not None and bound["pow2"] > 128:
                verdict_ok = False

    # ---- reports ----------------------------------------------------------
    lines = [
        "criterion render-sdf-criterion-v2 freeze-time B recomputation (scene-a, camera 1)",
        "signed B = 128 ULP; per v2 section 3 a recomputed B > 128 fails the attempt and",
        "raising B is forbidden (a criterion revision needs a new user signature).",
        f"inputs: model {replay_mod.MODEL}; sphere geom {scene.sphere_geom}; camera pose from",
        "        each reference's camera_data_check[1]; znear = f32(vis.map.znear * stat.extent)",
        "",
    ]
    for reference_rel, result in arms.items():
        lines.append(f"arm {reference_rel}: sha256 {result['ref'].sha}")
        lines.append(f"  spp={result['spp']} raster={result['ref'].width}x{result['ref'].height} "
                     f"sphere_pixels={result['sphere']} affected={result['affected_count']} "
                     f"max_observed_ulp={result['max_ulp']}")
        for name, bound in (("band4", result["band_bound"]),
                            ("derived_affected", result["affected_bound"])):
            if bound is None:
                lines.append(f"  set {name}: empty")
                continue
            lines.append(
                f"  set {name}: sqrt(det)_min={bound['sqrt_det_min']:.6e} "
                f"|c|max={bound['c_max']:.6f} |vz|max={bound['vz_max']:.6f} "
                f"depth_min={bound['depth_min']:.6f} ULP(depth)={bound['ulp_depth']:.6e}")
            lines.append(
                f"    raw_bound={bound['raw']:.2f} ULP x1.15={bound['with_margin']:.2f} "
                f"pow2={bound['pow2']} per_pixel_max={bound['per_pixel_max']:.2f}")
        lines.append(f"  band4 covers {result['covered']}/{result['affected_count']} affected pixels.")
        lines.append("")
    lines.append(f"verdict: {'CRITERION_V2_FREEZE_OK' if verdict_ok else 'CRITERION_V2_FREEZE_FAIL'}")
    (out / "b-recompute.txt").write_text("\n".join(lines) + "\n", encoding="ascii", newline="\n")
    writes.append("b-recompute.txt")

    lines = [
        "dump tool self-check (offline authoring, zero GPU)",
        f"1) camera-space rays vs S5 fixture default_all samples: {ray_total - ray_bad}/{ray_total} "
        f"bitwise equal",
    ]
    for reference_rel, result in arms.items():
        lines.append(
            f"2) arm {reference_rel}: form A vs frozen reference depth bits (sphere pixels): "
            f"{result['sphere'] - result['form_a_bad']}/{result['sphere']} bitwise equal")
        lines.append(
            f"   arm {reference_rel}: form A vs form B (derived affected set): "
            f"{result['affected_count']} pixels, max_ulp={result['max_ulp']}")
        lines.append(
            f"   arm {reference_rel}: band4 pixels={result['band_count']}, covers "
            f"{result['covered']}/{result['affected_count']} of the derived affected set")
    lines += [
        f"3) skybox capture depth/seg vs recipe reference: depth mismatches={sky_depth_mismatch}, "
        f"seg mismatches={sky_seg_mismatch}",
        f"4) camera 0 sphere pixels: {cam0_sphere} (camera 0 looks away from the sphere; no list needed)",
        "5) attempt3 observation set reproduced: cam1 183 px / max 73 (recipe arm), "
        "render_kernel_flags aa2 arm 172 px / max 91, cam0 0",
        f"internal_self_check: {'PASS' if checks_ok else 'FAIL'}",
    ]
    (out / "selfcheck.txt").write_text("\n".join(lines) + "\n", encoding="ascii", newline="\n")
    writes.append("selfcheck.txt")

    open_items = [
        "OPEN ITEMS (not silently dropped; decisions belong to Chief/user)",
        "1) The criterion-v2 section-2 predicate (band4) does not cover the derived affected set,",
        "   and the section-3 recomputation exceeds the signed B=128 ULP on both sets (see",
        "   b-recompute.txt). Under the signed v2 this attempt therefore FAILS at the freeze step;",
        "   any revision (list predicate / bound / B value) is a criterion change that needs the",
        "   user's signature.",
        "2) The aa2 arm keys off the product rule that depth/seg are written on the first pass only",
        "   (render_megakernel.cu); this tool replays sub-sample 0 on the doubled image grid and",
        "   reproduces the attempt3 172 px / max 91 observation bit-for-bit in form A.",
    ]
    (out / "open-items.txt").write_text("\n".join(open_items) + "\n", encoding="ascii", newline="\n")
    writes.append("open-items.txt")

    names = sorted(p.name for p in out.iterdir() if p.is_file() and p.name != "sha256sums.txt")
    sums = "".join(f"{replay_mod.sha256_file(out / name)}  {name}\n" for name in names)
    (out / "sha256sums.txt").write_text(sums, encoding="ascii", newline="\n")

    digest = hashlib.sha256()
    for name in sorted(writes):
        digest.update((out / name).read_bytes())
    print(f"entities={sum(len(entities) for _, entities in ARMS)} arms={len(ARMS)} "
          f"ray_selfcheck={ray_total - ray_bad}/{ray_total}")
    for reference_rel, result in arms.items():
        label = "recipe" if reference_rel == replay_mod.RECIPE_REFERENCE else "aa2"
        bound = result["affected_bound"]
        print(f"arm {label}: sphere={result['sphere']} affected={result['affected_count']} "
              f"max_ulp={result['max_ulp']} band4={result['band_count']} covered={result['covered']} "
              f"B_recompute={bound['pow2'] if bound else 'n/a'}")
    print(f"cam0_sphere={cam0_sphere} skybox_depth_mismatch={sky_depth_mismatch} "
          f"self_check={'PASS' if checks_ok else 'FAIL'} criterion_verdict="
          f"{'OK' if verdict_ok else 'FAIL'}")
    print(f"content_digest={digest.hexdigest()}")
    print(f"outputs written to {out}")
    if not checks_ok:
        return 5
    return 0 if verdict_ok else 4


if __name__ == "__main__":
    sys.exit(main())
