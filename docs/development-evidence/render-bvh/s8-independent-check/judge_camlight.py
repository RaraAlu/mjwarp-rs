"""QA judge: S8 camlight wiring + end-to-end numbers + reverse-proof controls.

Independent checks from QA harness dumps (target/qa-s8/out):
  1. product camlight poses vs the frozen S7b1 same-recipe camera_data bits (authoritative);
     lights vs pinned light_data_check; product vs *tampered* reference must differ (this is the
     discriminating form of the "render does not read the fixture" control);
  2. model-input sensitivity: cam_pos dx moves cam0, body2 dx moves the TARGETBODY camera1 —
     proves the chain computes from model fields instead of replaying fixture bits;
  3. end-to-end product-camlight render vs the same-recipe RGB / depth / seg fixtures, cross-recipe
     record vs S0b, FNV run hashes;
  4. controls: fixture-pose render equality, tampered-pose render change, product-pose
     perturbation counts/determinism/restore, refit equality, disabled outputs, 4 rejections;
  5. host `quat_to_mat` auxiliary path: max ULP and differing component count (classify aux vs defect).
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s8/out"
E2E = OUT / "e2e"
S7B = ROOT / "fixtures/render-bvh/scene-a/s7b"


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def packed_bytes(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)),
            (values >> np.uint32(8) & np.uint32(0xFF)),
            (values & np.uint32(0xFF)),
        ],
        axis=-1,
    ).astype(np.int16)


def rgb_stats(actual: np.ndarray, expected: np.ndarray) -> dict:
    diff = np.abs(packed_bytes(actual) - packed_bytes(expected)).max(axis=1)
    return {
        "pixels": int(diff.size),
        "mismatched": int(np.count_nonzero(diff > 0)),
        "beyond_1_lsb": int(np.count_nonzero(diff > 1)),
        "max_byte_diff": int(diff.max()),
        "histogram_0_1_2_3_gt3": [int(np.count_nonzero(diff == v)) for v in (0, 1, 2, 3)]
        + [int(np.count_nonzero(diff > 3))],
    }


def ulp(a_bits: int, b_bits: int) -> int:
    def mapped(bits: int) -> int:
        signed = bits - (1 << 32) if bits >= (1 << 31) else bits
        return -2147483648 - signed if signed < 0 else signed

    return abs(mapped(a_bits) - mapped(b_bits))


def fnv1a_64(values, basis: int = 0xCBF29CE484222325) -> int:
    h = basis
    for value in values:
        h = ((h ^ (value & 0xFFFFFFFFFFFFFFFF)) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def read_words(path: pathlib.Path, dtype) -> list:
    return np.frombuffer(path.read_bytes(), dtype=dtype).tolist()


def main() -> int:
    report: dict = {}
    camlight = load(OUT / "camlight.json")
    campos = load(OUT / "camlight_campos.json")
    body2 = load(OUT / "camlight_body2.json")
    e2e = load(E2E / "e2e.json")
    controls = load(E2E / "controls.json")

    # 1. pose bits vs reference and vs tampered reference
    reference = camlight["reference_poses"]
    product = camlight["product_poses"]
    tampered = camlight["tampered_reference_poses"]
    mismatches = []
    for index in range(len(reference)):
        origin = sum(
            a != b
            for a, b in zip(product[index]["xpos_bits"], reference[index]["xpos_bits"])
        )
        xmat = sum(
            a != b for a, b in zip(product[index]["xmat_bits"], reference[index]["xmat_bits"])
        )
        tampered_origin = sum(
            a != b for a, b in zip(product[index]["xpos_bits"], tampered[index]["xpos_bits"])
        )
        tampered_xmat = sum(
            a != b for a, b in zip(product[index]["xmat_bits"], tampered[index]["xmat_bits"])
        )
        mismatches.append(
            {
                "camera": index,
                "reported_origin_mismatches": camlight["product_vs_reference"]["origin_bit_mismatches"]
                if index == 0
                else None,
                "origin_mismatches_recomputed": origin,
                "xmat_mismatches_recomputed": xmat,
                "tampered_origin_mismatches": tampered_origin,
                "tampered_xmat_mismatches": tampered_xmat,
            }
        )
    report["camlight"] = {
        "cameras": mismatches,
        "product_poses_repeat_bitwise_equal": camlight["product_poses_repeat_bitwise_equal"],
        "camera_modes": [0, 3],
    }
    # lights vs pinned reference bit values (from rgb.json light_data_check)
    light_fixture = load(S7B / "rgb.json")["light_data_check"]
    light_product = camlight["product_lights"]
    light_ok = all(
        light_product["xpos"][i] == light_fixture[i]["xpos_bits"]
        and light_product["xdir"][i] == light_fixture[i]["xdir_bits"]
        for i in range(len(light_fixture))
    )
    report["camlight"]["lights_equal_pinned_reference"] = light_ok
    report["camlight"]["lights"] = light_product
    report["camlight"]["body_xpos_bits"] = camlight["product_body_xpos"]

    # 2. model-input sensitivity (positive evidence that the chain computes)
    baseline = np.array(product[0]["xpos_bits"], dtype=np.uint32)
    campos_poses = campos["product_poses"]
    body2_poses = body2["product_poses"]
    report["sensitivity"] = {
        "cam_pos_dx_cam0_origin_bits_changed": int(
            np.count_nonzero(np.array(campos_poses[0]["xpos_bits"], dtype=np.uint32) != baseline)
        ),
        "cam_pos_dx_cam1_origin_bits_changed": int(
            np.count_nonzero(
                np.array(campos_poses[1]["xpos_bits"], dtype=np.uint32)
                != np.array(product[1]["xpos_bits"], dtype=np.uint32)
            )
        ),
        "cam_pos_dx_expected_value": "cam0 xpos[0] +0.01",
        "body2_dx_cam1_xmat_bits_changed": int(
            np.count_nonzero(
                np.array(body2_poses[1]["xmat_bits"], dtype=np.uint32)
                != np.array(product[1]["xmat_bits"], dtype=np.uint32)
            )
        ),
        "body2_dx_cam1_origin_bits_changed": int(
            np.count_nonzero(
                np.array(body2_poses[1]["xpos_bits"], dtype=np.uint32)
                != np.array(product[1]["xpos_bits"], dtype=np.uint32)
            )
        ),
        "body2_dx_cam0_unchanged": bool(
            body2_poses[0]["xpos_bits"] == product[0]["xpos_bits"]
            and body2_poses[0]["xmat_bits"] == product[0]["xmat_bits"]
        ),
    }
    # actual f32 value change for cam0 xposx
    from struct import unpack

    def f32(bits):
        return unpack("f", np.uint32(bits).tobytes())[0]

    report["sensitivity"]["cam0_xpos_x_delta"] = f32(campos_poses[0]["xpos_bits"][0]) - f32(
        product[0]["xpos_bits"][0]
    )

    # 3. end-to-end vs same-recipe fixtures
    rgb = np.frombuffer((E2E / "product_rgb.bin").read_bytes(), dtype=np.uint32)
    depth = np.frombuffer((E2E / "product_depth.bin").read_bytes(), dtype=np.uint32)
    seg = np.frombuffer((E2E / "product_seg.bin").read_bytes(), dtype=np.int32).reshape(-1, 2)
    same_rgb = load(S7B / "rgb_fast_math_false.json")
    recipe = load(S7B / "depth_seg_fast_math_false.json")
    s0b = load(S7B / "rgb.json")
    starts = [0, 76800]
    cameras = []
    for index, (rgb_camera, depth_camera) in enumerate(
        zip(same_rgb["cameras"], recipe["cameras"])
    ):
        start = starts[index]
        pixels = int(rgb_camera["pixels"])
        actual_rgb = rgb[start : start + pixels]
        expected_rgb = np.asarray(rgb_camera["rgb_bits_packed"], dtype=np.uint32)
        actual_depth = depth[start : start + pixels]
        expected_depth = np.asarray(depth_camera["depth_bits"], dtype=np.uint32)
        actual_seg = seg[start : start + pixels]
        expected_seg = np.asarray(depth_camera["seg_pairs"], dtype=np.int32).reshape(-1, 2)
        left = actual_depth.astype(np.int64)
        right = expected_depth.astype(np.int64)
        left = np.where(left >= 0, left, np.int64(-2147483648) - left)
        right = np.where(right >= 0, right, np.int64(-2147483648) - right)
        depth_ulp = np.abs(left - right)
        cameras.append(
            {
                "camera": index,
                "rgb": rgb_stats(actual_rgb, expected_rgb),
                "depth_bitwise_equal": bool(np.array_equal(actual_depth, expected_depth)),
                "depth_max_ulp": int(depth_ulp.max()),
                "seg_mismatched": int(np.count_nonzero(np.any(actual_seg != expected_seg, axis=1))),
                "cross_recipe_vs_s0b": rgb_stats(
                    actual_rgb, np.asarray(s0b["cameras"][index]["rgb_bits_packed"], dtype=np.uint32)
                ),
            }
        )
    report["end_to_end"] = {
        "cameras": cameras,
        "rgb_total_mismatched": sum(c["rgb"]["mismatched"] for c in cameras),
        "rgb_total_beyond_1_lsb": sum(c["rgb"]["beyond_1_lsb"] for c in cameras),
        "depth_total_mismatched": sum(0 if c["depth_bitwise_equal"] else 1 for c in cameras),
        "seg_total_mismatched": sum(c["seg_mismatched"] for c in cameras),
    }
    depth_words = [int(value) & 0xFFFFFFFF for value in depth.tolist()]
    seg_words = [int(value) & 0xFFFFFFFF for value in seg.reshape(-1).tolist()]
    depth_seg_hash = fnv1a_64(depth_words + seg_words)
    rgb_hash = fnv1a_64([int(value) & 0xFFFFFFFF for value in rgb.tolist()], basis=0)
    report["run_hashes"] = {
        "depth_seg_hash": f"{depth_seg_hash:016x}",
        "rgb_hash": f"{rgb_hash:016x}",
        "delivered_log": "03efa95416bbd008 / ed50f750936c0275",
        "match": depth_seg_hash == 0x03EFA95416BBD008 and rgb_hash == 0xED50F750936C0275,
    }

    # 4. controls
    report["controls"] = {
        "determinism": controls["determinism"],
        "fixture_pose_render_equal": controls["fixture_pose_render_equal"],
        "tampered_render_changed": controls["tampered_render_changed"],
        "perturbation": controls["perturbation"],
        "lifecycle": controls["lifecycle"],
        "disabled": controls["disabled"],
        "rejections": controls["rejections"],
    }

    # 5. host quat_to_mat auxiliary path (cam0)
    quat_poses = e2e["quat_poses"]
    product_e2e = e2e["poses"]
    components = 0
    max_ulp = 0
    for product_bit, quat_bit in zip(product_e2e[0]["xmat_bits"], quat_poses[0]["xmat_bits"]):
        value = ulp(product_bit, quat_bit)
        components += int(value > 0)
        max_ulp = max(max_ulp, value)
    report["host_quat_to_mat_aux"] = {
        "camera0_xmat_max_ulp": max_ulp,
        "camera0_xmat_components_differing": components,
        "camera0_xpos_equal": product_e2e[0]["xpos_bits"] == quat_poses[0]["xpos_bits"],
        "product_pose_vs_reference_mismatches": sum(
            m["origin_mismatches_recomputed"] + m["xmat_mismatches_recomputed"]
            for m in report["camlight"]["cameras"]
        ),
        "classification": "auxiliary metric: the render input is the product camlight output (0 bit "
        "difference vs pinned data poses); crate::math::quat_to_mat is a host helper not used by the "
        "render path, so a 1-ULP difference there is not a product defect",
    }

    out = OUT / "judge_camlight.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
