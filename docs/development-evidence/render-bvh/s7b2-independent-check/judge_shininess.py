"""QA: material-exponent sensitivity (reference side) + product-shininess cross-check.

Shows the changed material value actually matters to the reference and that the product follows
it exactly at two distinct values (6.4 and 38.4 vs the baseline 12.8), i.e. the headlight
specular exponent is read from `mat_shininess` (material semantics) rather than a fixed 64.
"""

from __future__ import annotations

import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
REF = ROOT / "target/qa-s7b2/ref"
OUT = ROOT / "target/qa-s7b2/out"


def load_ref(tag: str) -> dict:
    return json.loads((REF / f"ref_{tag}.json").read_text(encoding="utf-8"))


def channels(packed: np.ndarray) -> np.ndarray:
    values = np.ascontiguousarray(packed, dtype=np.uint32)
    return np.stack(
        [
            (values >> np.uint32(16) & np.uint32(0xFF)),
            (values >> np.uint32(8) & np.uint32(0xFF)),
            (values & np.uint32(0xFF)),
        ],
        axis=-1,
    ).astype(np.int16)


def byte_stats(a: np.ndarray, b: np.ndarray) -> dict:
    diff = np.abs(channels(a) - channels(b)).max(axis=1)
    return {
        "pixels": int(diff.size),
        "changed_pixels": int(np.count_nonzero(diff > 0)),
        "beyond_1_lsb": int(np.count_nonzero(diff > 1)),
        "max_byte_diff": int(diff.max()),
        "histogram_0_1_2_3_gt3": [int(np.count_nonzero(diff == v)) for v in (0, 1, 2, 3)]
        + [int(np.count_nonzero(diff > 3))],
    }


def product(tag: str) -> np.ndarray:
    return np.frombuffer((OUT / f"rgb_{tag}.bin").read_bytes(), dtype=np.uint32)


def main() -> int:
    base = load_ref("base")
    report = {"reference_sensitivity": {}, "product_equality": {}}
    for tag in ("shin", "shin2"):
        modified = load_ref(tag)
        per_camera = []
        for index in (0, 1):
            base_cam = base["cameras"][index]
            mod_cam = modified["cameras"][index]
            depth_equal = base_cam["depth_bits"] == mod_cam["depth_bits"]
            seg_equal = base_cam["seg_pairs"] == mod_cam["seg_pairs"]
            per_camera.append(
                {
                    "index": index,
                    "shininess": modified["shininess"],
                    "rgb_byte_diff_vs_base_reference": byte_stats(
                        np.asarray(mod_cam["rgb_bits"], dtype=np.uint32),
                        np.asarray(base_cam["rgb_bits"], dtype=np.uint32),
                    ),
                    "depth_bits_unchanged": depth_equal,
                    "seg_unchanged": seg_equal,
                }
            )
        report["reference_sensitivity"][tag] = per_camera
        products = []
        for index in (0, 1):
            start = 0 if index == 0 else 76800
            pixels = 76800 if index == 0 else 43200
            modified_product = product(tag)[start : start + pixels]
            modified_reference = np.asarray(
                modified["cameras"][index]["rgb_bits"], dtype=np.uint32
            )
            base_product = product("base")[start : start + pixels]
            products.append(
                {
                    "index": index,
                    "product_equals_modified_reference": bool(
                        np.array_equal(modified_product, modified_reference)
                    ),
                    "product_byte_stats_vs_base_product": byte_stats(
                        modified_product, base_product
                    ),
                }
            )
        report["product_equality"][tag] = products
    out = OUT / "judge_shininess.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
