"""Upstream-side instrumented replay extended to flex-0 (three edges) for the residual pixel.

Reuses the chain helpers of upstream_flex_chain.py (pinned warp 1.15 / CUDA 12.9 / sm89).
Dumps the winning edge's intermediates for every cam2 flex-0 pixel and compares the
resulting depth against the reference and product depth bits.

No reference files are written; output goes to
target/render-bvh/r06-s3b-flex-window/f32-order/residual/.
"""
import json
import pathlib
import sys

import numpy as np
import warp as wp

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target" / "render-bvh" / "r06-s3b-flex-window" / "f32-order"
RES = OUT / "residual"
sys.path.insert(0, str(ROOT / "target" / "s3b-rca"))
import upstream_flex_chain as uf  # noqa: E402

SLOTS = 26

wp.init()


@wp.kernel
def flex0_chain(
    rays: wp.array(dtype=wp.vec3),
    cam_mat: wp.array(dtype=wp.mat33),
    cam_pos: wp.array(dtype=wp.vec3),
    verts: wp.array(dtype=wp.vec3),
    radius: wp.array(dtype=float),
    out: wp.array2d(dtype=float),
    outd: wp.array(dtype=float),
    out_edge: wp.array(dtype=int),
    out_d3: wp.array2d(dtype=float),
):
    i = wp.tid()
    origin = cam_pos[0]
    dirw = wp.mul(cam_mat[0], rays[i])
    best = -1.0
    best_edge = -1
    best_pos = wp.vec3(0.0, 0.0, 0.0)
    best_vec = wp.vec3(0.0, 0.0, 0.0)
    best_len = 0.0
    best_quat = wp.vec4(0.0, 0.0, 0.0, 1.0)
    best_size = wp.vec3(0.0, 0.0, 0.0)
    for e in range(3):
        a0 = verts[e]
        a1 = verts[e + 1]
        pos = 0.5 * (a0 + a1)
        vec = a1 - a0
        length = wp.length(vec)
        quat = uf.q_z2vec(vec)
        mat = uf.q_to_mat(quat)
        size = wp.vec3(radius[0], 0.5 * length, 0.0)
        d = uf.ray_capsule(pos, mat, size, origin, dirw)
        out_d3[i, e] = d
        if d >= 0.0 and (best < 0.0 or d < best):
            best = d
            best_edge = e
            best_pos = pos
            best_vec = vec
            best_len = length
            best_quat = quat
            best_size = size
    outd[i] = best
    out_edge[i] = best_edge
    row = 0
    out[i, row] = wp.float32(best_edge)
    row += 1
    for k in range(3):
        out[i, row] = best_pos[k]
        row += 1
    for k in range(3):
        out[i, row] = best_vec[k]
        row += 1
    out[i, row] = best_len
    row += 1
    for k in range(4):
        out[i, row] = best_quat[k]
        row += 1
    for k in range(3):
        out[i, row] = best_size[k]
        row += 1
    for k in range(3):
        out[i, row] = dirw[k]
        row += 1
    mat_w = uf.q_to_mat(best_quat)
    matT = wp.transpose(mat_w)
    lpnt = wp.mul(matT, origin - best_pos)
    lvec = wp.mul(matT, dirw)
    for k in range(3):
        out[i, row] = lpnt[k]
        row += 1
    for k in range(3):
        out[i, row] = lvec[k]
        row += 1


def main():
    rays = np.frombuffer((OUT / "cam2-ray.f32le").read_bytes(), dtype="<f4").reshape(-1, 6)
    dirs = rays[:, :3].astype(np.float32)
    state = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/state.npz")
    flexvert = state["flexvert_xpos"][0].astype(np.float32)
    poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
    f32of = lambda b: np.uint32(b).view(np.float32)
    cam_pos = np.array([f32of(b) for b in poses["poses"]["cam_xpos_bits"][6:9]], dtype=np.float32)
    cam_xmat = np.array([f32of(b) for b in poses["poses"]["cam_xmat_bits"][18:27]], dtype=np.float32).reshape(3, 3)
    radius = np.array([0.02, 0.04], dtype=np.float32)

    wp_rays = wp.array(dirs, dtype=wp.vec3, device="cuda")
    wp_mat = wp.array(np.stack([cam_xmat]), dtype=wp.mat33, device="cuda")
    wp_pos = wp.array(np.stack([cam_pos]), dtype=wp.vec3, device="cuda")
    wp_verts = wp.array(flexvert.astype(np.float32), dtype=wp.vec3, device="cuda")
    wp_radius = wp.array(radius, dtype=float, device="cuda")
    out = wp.zeros((len(dirs), SLOTS), dtype=float, device="cuda")
    outd = wp.zeros(len(dirs), dtype=float, device="cuda")
    out_edge = wp.zeros(len(dirs), dtype=int, device="cuda")
    out_d3 = wp.zeros((len(dirs), 3), dtype=float, device="cuda")
    wp.launch(
        flex0_chain,
        dim=len(dirs),
        inputs=[wp_rays, wp_mat, wp_pos, wp_verts, wp_radius, out, outd, out_edge, out_d3],
        device="cuda",
    )
    wp.synchronize()
    vals = out.numpy().astype(np.float32)
    d = outd.numpy().astype(np.float32)
    edge = out_edge.numpy().astype(np.int32)
    d3 = out_d3.numpy().astype(np.float32)
    dirz = dirs[:, 2]
    depth_pred = d * (-dirz)
    ref_depth = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].reshape(-1).astype(np.float32)
    prod_depth = np.frombuffer((RES / "cam2-depth.f32le").read_bytes(), dtype="<f4").astype(np.float32)
    seg = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/seg.npz")["seg_cam2"].reshape(-1, 2)
    flex0 = (seg[:, 1] == 9) & (seg[:, 0] == 0)
    flex1 = (seg[:, 1] == 9) & (seg[:, 0] == 1)
    m = flex0
    bp = depth_pred[m].view(np.uint32)
    br = ref_depth[m].view(np.uint32)
    bq = prod_depth[m].view(np.uint32)
    print(f"flex0 pixels={int(m.sum())}")
    print("oracle vs REFERENCE depth mismatched:", int((bp != br).sum()), "/", int(m.sum()))
    print("oracle vs PRODUCT   depth mismatched:", int((bp != bq).sum()), "/", int(m.sum()))
    ulp_ref = np.abs(bp.astype(np.int64) - br.astype(np.int64))
    ulp_prod = np.abs(bp.astype(np.int64) - bq.astype(np.int64))
    print("oracle-vs-ref ULP median", float(np.median(ulp_ref)), "max", int(ulp_ref.max()))
    print("oracle-vs-prod ULP median", float(np.median(ulp_prod)), "max", int(ulp_prod.max()))
    # residual pixel
    idx = 128 * 320 + 134
    print(f"\nresidual pixel (134,128) idx={idx}: oracle_edge={int(edge[idx])} d={d[idx]!r} "
          f"depth_pred={depth_pred[idx]!r} bits={int(depth_pred[idx].view(np.uint32)):#010x}")
    print(f"  ref_depth={ref_depth[idx]!r} bits={int(ref_depth[idx].view(np.uint32)):#010x} "
          f"prod_depth={prod_depth[idx]!r} bits={int(prod_depth[idx].view(np.uint32)):#010x}")
    for e in range(3):
        print(f"  edge{e} d={d3[idx, e]!r} depth={np.float32(d3[idx, e] * (-dirz[idx]))!r} "
              f"bits={int(np.float32(d3[idx, e] * (-dirz[idx])).view(np.uint32)):#010x}")
    print("  intermediates:", vals[idx])
    np.savez(
        RES / "upstream-flex0-chain.npz",
        dirs=dirs[m],
        edge=edge[m],
        d=d[m],
        d3=d3[m],
        values=vals[m],
        depth_pred=depth_pred[m],
        ref_depth=ref_depth[m],
        prod_depth=prod_depth[m],
        cam_xpos=cam_pos,
        cam_xmat=cam_xmat,
        flexvert=flexvert,
        residual_index=np.array([idx]),
    )
    print("saved", RES / "upstream-flex0-chain.npz")


if __name__ == "__main__":
    main()
