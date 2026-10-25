"""QA judge: light data poses (3 lights, xpos/xdir f32 bits), four-way.

Sources compared bit-for-bit:
  1. QA f32 recompute from the RenderFields model snapshot (`model-312.json`), following the
     upstream `smooth.py::_light_local_to_global` else-branch formula
     `xpos = body_xpos + rot_vec_quat(pos, xquat)`, `xdir = normalize(rot_vec_quat(dir, xquat))`
     with the world body identity quaternion (verified from the .mjb via MuJoCo).
  2. pinned 3.15 forward (`target/qa-s7b2/ref/ref_base.json::lights`, captured by QA).
  3. product `light_data_poses` (harness_base.json).
  4. delivered fixture `rgb.json::light_data_check`.

The f32 operation order is not assumed: two normalize/dot orders are computed and reported so
the matching order is evidence, not belief.
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s7b2/out"
REF = ROOT / "target/qa-s7b2/ref"
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"
MJB = ROOT / "fixtures/render-bvh/scene-a/scene-315.mjb"

f32 = np.float32


def fma(a, b, c):
    """Correctly rounded f32 fma (f32*f32 products are exact in f64, sum rounds once)."""
    return f32(float(f32(a)) * float(f32(b)) + float(f32(c)))


def sqrt_f32(value):
    return f32(np.sqrt(f32(value)))


def normalize_variants(vector):
    x, y, z = (f32(value) for value in vector)
    # V1: length = sqrt(fma(z,z,fma(y,y,x*x))), then per-component division.
    length_fma = sqrt_f32(fma(z, z, fma(y, y, f32(x * x))))
    # V2: left-associative plain dot sqrt((x*x + y*y) + z*z), division.
    length_plain = sqrt_f32(f32(f32(f32(x * x) + f32(y * y)) + f32(z * z)))
    # V3: fma length but multiply by reciprocal.
    reciprocal = f32(1.0) / length_fma
    return {
        "fma_div": [f32(x / length_fma), f32(y / length_fma), f32(z / length_fma)],
        "fma_mul_recip": [f32(x * reciprocal), f32(y * reciprocal), f32(z * reciprocal)],
        "plain_div": [f32(x / length_plain), f32(y / length_plain), f32(z / length_plain)],
    }


def bits3(values):
    return [int(value) for value in np.asarray(values, dtype=np.float32).view(np.uint32)]


def main() -> int:
    model = json.loads(
        (ROOT / "fixtures/render-bvh/scene-a/model-312.json").read_text(encoding="utf-8")
    )["arrays"]
    mode = [int(v) for v in model["light_mode"]["values"]]
    bodyid = [int(v) for v in model["light_bodyid"]["values"]]
    pos = [float(v) for v in model["light_pos"]["values"]]
    direction = [float(v) for v in model["light_dir"]["values"]]
    count = len(mode)

    # world body quaternion from the actual .mjb (identity expected -> rot_vec_quat is identity)
    import mujoco

    mjm = mujoco.MjModel.from_binary_path(str(MJB))
    mjd = mujoco.MjData(mjm)
    mujoco.mj_forward(mjm, mjd)
    world_xquat = np.asarray(mjd.xquat[0], dtype=np.float64)
    light_bodyids_from_mjb = np.asarray(mjm.light_bodyid)
    light_modes_from_mjb = np.asarray(mjm.light_mode)

    product = json.loads((OUT / "harness_base.json").read_text(encoding="utf-8"))["lights"]
    pinned = json.loads((REF / "ref_base.json").read_text(encoding="utf-8"))["lights"]
    fixture = json.loads((S7B / "rgb.json").read_text(encoding="utf-8"))["light_data_check"]

    report = {
        "world_body_xquat": [float(v) for v in world_xquat],
        "world_body_xquat_is_identity": bool(np.array_equal(world_xquat, [1.0, 0.0, 0.0, 0.0])),
        "mjb_light_bodyid": [int(v) for v in light_bodyids_from_mjb],
        "mjb_light_mode": [int(v) for v in light_modes_from_mjb],
        "lights": [],
    }
    for index in range(count):
        position = [f32(pos[3 * index + axis]) for axis in range(3)]
        d = [direction[3 * index + axis] for axis in range(3)]
        variants = normalize_variants(d)
        xpos_bits = bits3(position)
        entry = {
            "index": index,
            "mode": mode[index],
            "bodyid": bodyid[index],
            "xpos_bits": xpos_bits,
            "xdir_bits_variants": {name: bits3(values) for name, values in variants.items()},
            "pinned_forward_xpos_bits": pinned[index]["xpos_bits"],
            "pinned_forward_xdir_bits": pinned[index]["xdir_bits"],
            "product_xpos_bits": product[index]["xpos_bits"],
            "product_xdir_bits": product[index]["xdir_bits"],
            "fixture_xpos_bits": fixture[index]["xpos_bits"],
            "fixture_xdir_bits": fixture[index]["xdir_bits"],
        }
        entry["xpos_matches"] = {
            "recompute_vs_pinned": xpos_bits == pinned[index]["xpos_bits"],
            "recompute_vs_product": xpos_bits == product[index]["xpos_bits"],
            "recompute_vs_fixture": xpos_bits == fixture[index]["xpos_bits"],
            "pinned_vs_product": pinned[index]["xpos_bits"] == product[index]["xpos_bits"],
            "pinned_vs_fixture": pinned[index]["xpos_bits"] == fixture[index]["xpos_bits"],
        }
        entry["xdir_variant_matches_pinned"] = {
            name: bits3(values) == pinned[index]["xdir_bits"] for name, values in variants.items()
        }
        entry["xdir_matches"] = {
            "product_vs_pinned": product[index]["xdir_bits"] == pinned[index]["xdir_bits"],
            "fixture_vs_pinned": fixture[index]["xdir_bits"] == pinned[index]["xdir_bits"],
        }
        report["lights"].append(entry)

    out = OUT / "judge_light.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(
        {
            "world_identity": report["world_body_xquat_is_identity"],
            "mjb_bodyid": report["mjb_light_bodyid"],
            "mjb_mode": report["mjb_light_mode"],
            "xpos_all": [light["xpos_matches"] for light in report["lights"]],
            "xdir_variants": [light["xdir_variant_matches_pinned"] for light in report["lights"]],
            "xdir_cross": [light["xdir_matches"] for light in report["lights"]],
        },
        indent=1,
    ))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
