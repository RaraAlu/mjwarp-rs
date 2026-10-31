"""Regenerate the v2.1 proposal prototype exact-difference list files.

Read-only with respect to the repository: it reads the pinned fixtures, imports the committed
offline authoring replay module (scripts/authoring/scene_a_outline_replay.py) and writes only
the two list files into --out (default: the directory holding this script).

Run (offline, zero GPU, pinned interpreter):
  target/mjwarp-parallel/venv/Scripts/python.exe -B \
    docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists/generate-lists.py \
    --repo . --out docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists

The lists are the exact difference sets of the two `a = dot(vec,vec)` association forms
(form A = (0,1,2) reference form, form B = (1,0,2) product form) over the camera-1 sphere
pixels of the frozen references; see v2.1-proposal.md section A.
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

LIST_HEADER = [
    "# scene-a sphere-outline exact-diff list (criterion v2.1 proposal format)",
    "# columns: pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab",
]

ARMS = (
    ("recipe", "RECIPE_REFERENCE", 1),
    ("aa2", "AA2_REFERENCE", 2),
)


def build_list(root, reference_rel, spp):
    import scene_a_outline_replay as rp

    scene = rp.Scene(root)
    ref = rp.Reference(root, reference_rel, 1)
    sphere = rp.sphere_pixels(scene, ref)
    data_a = rp.replay(scene, ref, 1, spp, "A", sphere)
    data_b = rp.replay(scene, ref, 1, spp, "B", sphere)
    lines = [
        LIST_HEADER[0],
        f"# reference: {reference_rel}",
        f"# reference sha256: {ref.sha}",
        f"# camera: 1 raster: {ref.width}x{ref.height} spp: {spp}",
        LIST_HEADER[1],
    ]
    for pixel in sphere:
        bits_a = rp.bits(data_a[pixel][0])
        bits_b = rp.bits(data_b[pixel][0])
        if bits_a == bits_b:
            continue
        lines.append(
            f"{pixel} {pixel % ref.width} {pixel // ref.width} "
            f"{ref.seg[2 * pixel]} {ref.seg[2 * pixel + 1]} {ref.depth[pixel]} "
            f"{bits_a} {bits_b} {abs(bits_a - bits_b)}"
        )
    return "\n".join(lines) + "\n"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args(argv)
    root = args.repo.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    sys.path.insert(0, str(root / "scripts" / "authoring"))
    import scene_a_outline_replay as rp  # noqa: F401  (used by build_list)

    manifest_lines = []
    for name, attr, spp in ARMS:
        reference_rel = getattr(rp, attr)
        text = build_list(root, reference_rel, spp)
        path = out / f"scene-a-sphere-outline-{name}.txt"
        path.write_bytes(text.encode("ascii"))
        payload = path.read_bytes()
        manifest_lines.append(f"{hashlib.sha256(payload).hexdigest()} {len(payload)} {path.name}")
        print(f"{path.name}: {len(payload)} bytes sha256={hashlib.sha256(payload).hexdigest()}")
    (out / "lists-sha256.txt").write_bytes(("\n".join(manifest_lines) + "\n").encode("ascii"))


if __name__ == "__main__":
    main()
