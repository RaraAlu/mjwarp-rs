"""Regenerate the pinned-GPU quat reference used by fix card 2026-10-28-render-bvh-camera-pose-warp-fma-align.

Context: the upstream product path (mujoco_warp camera kernel) computes
  cam_xmat = quat_to_mat(mul_quat(xquat_world, cam_quat))
with xquat_world == identity for fixed cameras. This script launches the same chain on the pinned GPU env
(warp 1.15.0 / CUDA Toolkit 12.9 / driver 13.2 / RTX 4070 Ti SUPER), for:
  idx 0        hfield cam0 cam_quat (tests/support/render_bvh/hfield/model-312.json)
  idx 1        hfield cam1 cam_quat
  idx 2        scene-a cam0 cam_quat (fixtures/render-bvh/scene-a/model-312.json)
  idx 3..1002  1000 standard-normal quats (numpy default_rng seed 20261029), memory order (w,x,y,z)

Output JSON fields: quat_bits[1003][4], mat_bits[1003][9], mul_quat_bits[1003][4] (u32 f32 bit patterns,
row-major mat33). Deterministic for the pinned versions.

Usage (repo root):  <pinned venv>/python -B docs/development-evidence/render-bvh/r05-s3b-camera-pose-warp-fma-align/materials/scripts/gpu_quat_reference.py --out <path>.json
"""
import argparse
import json
import pathlib

import numpy as np
import warp as wp


def find_repo_root() -> pathlib.Path:
    for parent in pathlib.Path(__file__).resolve().parents:
        if (parent / "Cargo.toml").is_file():
            return parent
    return pathlib.Path.cwd()


wp.config.kernel_cache_dir = str(
    find_repo_root() / "target/render-bvh/r05-s3b-window/ptx-probe-cache"
)
from mujoco_warp._src import math as mjmath  # noqa: E402


@wp.kernel
def chain_kernel(xq: wp.array(dtype=wp.quat), cq: wp.array(dtype=wp.quat),
                 out: wp.array(dtype=wp.mat33), outq: wp.array(dtype=wp.quat)):
    i = wp.tid()
    q = mjmath.mul_quat(xq[i], cq[i])
    outq[i] = q
    out[i] = mjmath.quat_to_mat(q)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    repo = pathlib.Path(args.repo).resolve()

    h = json.loads((repo / "tests/support/render_bvh/hfield/model-312.json").read_text(encoding="utf-8"))
    sa = json.loads((repo / "fixtures/render-bvh/scene-a/model-312.json").read_text(encoding="utf-8"))
    hq = np.asarray(h["arrays"]["cam_quat"]["values"], dtype=np.float32)
    saq = np.asarray(sa["arrays"]["cam_quat"]["values"], dtype=np.float32)
    known = np.stack([hq[0:4], hq[4:8], saq[0:4]])
    rng = np.random.default_rng(20261029)
    random_quats = rng.standard_normal((1000, 4)).astype(np.float32)
    quats_mj = np.concatenate([known, random_quats], axis=0).astype(np.float32)

    xq = np.zeros((quats_mj.shape[0], 4), dtype=np.float32)
    xq[:, 0] = 1.0
    xq_a = wp.array(xq, dtype=wp.quat)
    cq_a = wp.array(quats_mj, dtype=wp.quat)
    out = wp.zeros(quats_mj.shape[0], dtype=wp.mat33)
    outq = wp.zeros(quats_mj.shape[0], dtype=wp.quat)
    wp.launch(chain_kernel, dim=quats_mj.shape[0], inputs=[xq_a, cq_a, out, outq])
    wp.synchronize()

    payload = {
        "names": ["hfield_cam0", "hfield_cam1", "scenea_cam0"],
        "seed": 20261029,
        "warp_version": wp.__version__,
        "quat_bits": quats_mj.view(np.uint32).tolist(),
        "mat_bits": out.numpy().astype(np.float32).reshape(quats_mj.shape[0], 9).view(np.uint32).tolist(),
        "mul_quat_bits": outq.numpy().astype(np.float32).view(np.uint32).tolist(),
    }
    target = pathlib.Path(args.out)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
    print(f"wrote {target} ({quats_mj.shape[0]} quats, warp {wp.__version__})")


if __name__ == "__main__":
    main()
