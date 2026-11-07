"""G-L1 capture-window discriminability check (CPU only, read-only).

Purpose (criterion spec v2 sections 5/6 + 12 G-L1): after the authoring capture,
verify that the *reference itself* is discriminating between adjacent worlds, so
the validation window cannot pass on a non-discriminating reference.

Metric (written in the spec, no ad-hoc variation):
  - depth changed pixels(world k-1 -> k) = count(bits(depth_w{k-1}_cam{c}) != bits(depth_w{k}_cam{c}))
  - seg changed pixels(world k-1 -> k)   = count(seg_pair_w{k-1}_cam{c} != seg_pair_w{k}_cam{c})
  - pairs checked: 0->1 and 1->2, for every camera present in the reference
  - thresholds: acceptance 384, collection margin 576 (depth AND seg, BOTH pairs)

Bands (exit codes match the capture-script convention):
  - < 384 on any metric/pair            -> "non-discriminating"  -> exit 3
  - >= 384 but < 576 on any metric/pair -> "insufficient-margin" -> exit 3
  - all >= 576                          -> "pass-margin"         -> exit 0

Notes:
  - `spot` has zero per-world delta (its pos/dir lie on the tri_body rotation axis);
    that is a scene-geometry fact recorded in the specs, NOT judged here. The
    pixel metrics above already reflect only the discriminable lights (sun/lamp).
  - Nothing is written; this is a read-only analysis of an existing reference dir.

Usage:
  <pinned-python> -B discriminability_check.py --reference-dir <dir> [--json-out <path>]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys

ACCEPT_MIN = 384
MARGIN_MIN = 576


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cam_ids(members: dict) -> list[int]:
    import re

    ids = set()
    for name in members:
        match = re.fullmatch(r"depth_w(\d+)_cam(\d+)(?:\.npy)?", name)
        if match:
            ids.add(int(match.group(2)))
    return sorted(ids)


def load(path: pathlib.Path) -> dict:
    import numpy as np

    if not path.is_file():
        raise SystemExit(f"discriminability: missing {path}")
    with np.load(path) as archive:
        return {name: archive[name] for name in archive.files}


def main() -> int:
    parser = argparse.ArgumentParser(description="G-L1 per-world discriminability check (CPU only)")
    parser.add_argument("--reference-dir", type=pathlib.Path, required=True)
    parser.add_argument("--json-out", type=pathlib.Path, default=None)
    args = parser.parse_args()

    import numpy as np

    root = args.reference_dir.resolve()
    if not root.is_dir():
        raise SystemExit(f"discriminability: reference dir missing: {root}")

    depth = load(root / "depth.npz")
    seg = load(root / "seg.npz")
    cameras = cam_ids(depth)
    if not cameras:
        raise SystemExit("discriminability: no depth_w{k}_cam{c}.npy members found")

    worlds = sorted({int(name.split("_")[1].lstrip("w")) for name in depth if name.startswith("depth_w")})
    if worlds != [0, 1, 2]:
        raise SystemExit(f"discriminability: expected worlds [0,1,2], got {worlds}")

    report = {
        "schema": 1,
        "kind": "perworld-discriminability-check",
        "reference_dir": str(root),
        "thresholds": {"accept_min": ACCEPT_MIN, "margin_min": MARGIN_MIN},
        "cameras": cameras,
        "worlds": worlds,
        "pairs": [],
        "inputs_sha256": {
            name: sha256_file(root / name) for name in ("rgb.npz", "depth.npz", "seg.npz", "state.npz")
        },
        "spot_note": (
            "spot light has zero per-world delta (on the tri_body rotation axis); recorded in the "
            "criterion spec as a scene-geometry fact, not judged here"
        ),
    }

    overall = "pass-margin"
    for left in (0, 1):
        right = left + 1
        for camera in cameras:
            def resolve(archive: dict, stem: str) -> str:
                for candidate in (stem, f"{stem}.npy"):
                    if candidate in archive:
                        return candidate
                raise SystemExit(f"discriminability: missing member {stem} (or {stem}.npy)")

            depth_left_key = resolve(depth, f"depth_w{left}_cam{camera}")
            depth_right_key = resolve(depth, f"depth_w{right}_cam{camera}")
            seg_left_key = resolve(seg, f"seg_w{left}_cam{camera}")
            seg_right_key = resolve(seg, f"seg_w{right}_cam{camera}")
            depth_left = depth[depth_left_key].reshape(-1).view(np.uint32)
            depth_right = depth[depth_right_key].reshape(-1).view(np.uint32)
            depth_changed = int(np.count_nonzero(depth_left != depth_right))
            seg_left = seg[seg_left_key]
            seg_right = seg[seg_right_key]
            if seg_left.shape != seg_right.shape:
                raise SystemExit(f"discriminability: seg shape mismatch {seg_left_key} vs {seg_right_key}")
            seg_changed = int(np.count_nonzero(np.any(seg_left != seg_right, axis=-1)))
            if depth_changed < ACCEPT_MIN or seg_changed < ACCEPT_MIN:
                pair_verdict = "non-discriminating"
            elif depth_changed < MARGIN_MIN or seg_changed < MARGIN_MIN:
                pair_verdict = "insufficient-margin"
            else:
                pair_verdict = "pass-margin"
            if pair_verdict == "non-discriminating":
                overall = "non-discriminating"
            elif pair_verdict == "insufficient-margin" and overall == "pass-margin":
                overall = "insufficient-margin"
            report["pairs"].append(
                {
                    "pair": f"{left:03d}->{right:03d}",
                    "camera": camera,
                    "depth_changed": depth_changed,
                    "seg_changed": seg_changed,
                    "verdict": pair_verdict,
                }
            )

    report["overall"] = overall
    text = json.dumps(report, indent=2, ensure_ascii=False)
    print(text)
    if args.json_out is not None:
        args.json_out.write_text(text + "\n", encoding="utf-8", newline="\n")
    return 0 if overall == "pass-margin" else 3


if __name__ == "__main__":
    sys.exit(main())
