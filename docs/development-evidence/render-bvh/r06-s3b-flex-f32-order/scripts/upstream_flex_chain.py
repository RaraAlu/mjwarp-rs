"""Upstream-side instrumented replay of the frozen 1D flex hit chain (diagnostic only).

Replicates mujoco_warp 3.15 / upstream 71da24d `render.py` flex branch + `ray.py`
ray_capsule using warp builtins, compiled by the pinned upstream warp (CUDA 12.9, sm_89),
and dumps per-pixel intermediates for the cam2 flex-1 capsule pixels.

No reference files are written; outputs go to
target/render-bvh/r06-s3b-flex-window/f32-order/.
"""
import json
import pathlib

import numpy as np
import warp as wp

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target" / "render-bvh" / "r06-s3b-flex-window" / "f32-order"
OUT.mkdir(parents=True, exist_ok=True)

MJ_MINVAL = 1.0e-15
SLOTS = 24

wp.init()
print("warp", wp.config.version, "device", wp.get_device())


@wp.func
def q_z2vec(vec: wp.vec3):
    q = wp.vec4(0.0, 0.0, 0.0, 1.0)
    norm = wp.length(vec)
    if norm < MJ_MINVAL:
        return q
    v = vec / norm
    axis = wp.vec3(-v[1], v[0], 0.0)
    a = wp.length(axis)
    if a < MJ_MINVAL:
        if v[2] < 0.0:
            q = wp.vec4(1.0, 0.0, 0.0, 0.0)
        return q
    axis = axis / a
    angle = wp.atan2(a, v[2])
    s = wp.sin(angle * 0.5)
    c = wp.cos(angle * 0.5)
    return wp.vec4(c, axis[0] * s, axis[1] * s, axis[2] * s)


@wp.func
def q_to_mat(q: wp.vec4) -> wp.mat33:
    q00 = q[0] * q[0]
    q01 = q[0] * q[1]
    q02 = q[0] * q[2]
    q03 = q[0] * q[3]
    q11 = q[1] * q[1]
    q12 = q[1] * q[2]
    q13 = q[1] * q[3]
    q22 = q[2] * q[2]
    q23 = q[2] * q[3]
    q33 = q[3] * q[3]
    return wp.mat33(
        q00 + q11 - q22 - q33,
        2.0 * (q12 - q03),
        2.0 * (q13 + q02),
        2.0 * (q12 + q03),
        q00 - q11 + q22 - q33,
        2.0 * (q23 - q01),
        2.0 * (q13 - q02),
        2.0 * (q23 + q01),
        q00 - q11 - q22 + q33,
    )


@wp.func
def ray_quad(a: float, b: float, c: float):
    det = b * b - a * c
    if det < MJ_MINVAL:
        return -1.0, -1.0, -1.0
    det = wp.sqrt(det)
    den = 1.0 / a if a != 0.0 else 1.0 / MJ_MINVAL
    x0 = (-b - det) * den
    x1 = (-b + det) * den
    sol = x0 if x0 >= 0.0 else (x1 if x1 >= 0.0 else -1.0)
    return sol, x0, x1


@wp.func
def ray_capsule(pos: wp.vec3, mat: wp.mat33, size: wp.vec3, pnt: wp.vec3, vec: wp.vec3) -> float:
    ssz = size[0] + size[1]
    dist_sqr = ssz * ssz
    dif = pnt - pos
    a_sp = wp.dot(vec, vec)
    b_sp = wp.dot(vec, dif)
    c_sp = wp.dot(dif, dif) - dist_sqr
    sol, x0, x1 = ray_quad(a_sp, b_sp, c_sp)
    if sol < 0.0:
        return -1.0
    matT = wp.transpose(mat)
    lpnt = wp.mul(matT, pnt - pos)
    lvec = wp.mul(matT, vec)
    x = -1.0
    sq_size0 = size[0] * size[0]
    a = lvec[0] * lvec[0] + lvec[1] * lvec[1]
    b = lvec[0] * lpnt[0] + lvec[1] * lpnt[1]
    c = lpnt[0] * lpnt[0] + lpnt[1] * lpnt[1] - sq_size0
    sol, x0, x1 = ray_quad(a, b, c)
    part = 0
    if sol >= 0.0 and wp.abs(lpnt[2] + sol * lvec[2]) <= size[1]:
        if x < 0.0 or sol < x:
            x = sol
    ldif = wp.vec3(lpnt[0], lpnt[1], lpnt[2] - size[1])
    a = a + lvec[2] * lvec[2]
    b = wp.dot(lvec, ldif)
    c = wp.dot(ldif, ldif) - sq_size0
    sol, x0, x1 = ray_quad(a, b, c)
    if x0 >= 0.0 and lpnt[2] + x0 * lvec[2] >= size[1] and (x < 0.0 or x0 < x):
        x = x0
        part = 1
    if x1 >= 0.0 and lpnt[2] + x1 * lvec[2] >= size[1] and (x < 0.0 or x1 < x):
        x = x1
        part = 1
    ldif = wp.vec3(ldif[0], ldif[1], lpnt[2] + size[1])
    b = wp.dot(lvec, ldif)
    c = wp.dot(ldif, ldif) - sq_size0
    sol, x0, x1 = ray_quad(a, b, c)
    if x0 >= 0.0 and lpnt[2] + x0 * lvec[2] <= -size[1] and (x < 0.0 or x0 < x):
        x = x0
        part = -1
    if x1 >= 0.0 and lpnt[2] + x1 * lvec[2] <= -size[1] and (x < 0.0 or x1 < x):
        x = x1
        part = -1
    return x


@wp.kernel
def flex_chain(
    rays: wp.array(dtype=wp.vec3),
    cam_mat: wp.array(dtype=wp.mat33),
    cam_pos: wp.array(dtype=wp.vec3),
    verts: wp.array(dtype=wp.vec3),
    radius: wp.array(dtype=float),
    v0: int,
    v1: int,
    flexid: int,
    out: wp.array2d(dtype=float),
    outd: wp.array(dtype=float),
):
    i = wp.tid()
    origin = cam_pos[0]
    dirw = wp.mul(cam_mat[0], rays[i])
    a0 = verts[v0]
    a1 = verts[v1]
    pos = 0.5 * (a0 + a1)
    vec = a1 - a0
    length = wp.length(vec)
    quat = q_z2vec(vec)
    mat = q_to_mat(quat)
    size = wp.vec3(radius[flexid], 0.5 * length, 0.0)
    d = ray_capsule(pos, mat, size, origin, dirw)
    matT = wp.transpose(mat)
    lpnt = wp.mul(matT, origin - pos)
    lvec = wp.mul(matT, dirw)
    outd[i] = d
    row = 0
    for k in range(3):
        out[i, row] = pos[k]
        row += 1
    for k in range(3):
        out[i, row] = vec[k]
        row += 1
    out[i, row] = length
    row += 1
    for k in range(4):
        out[i, row] = quat[k]
        row += 1
    for k in range(3):
        out[i, row] = size[k]
        row += 1
    for k in range(3):
        out[i, row] = dirw[k]
        row += 1
    for k in range(3):
        out[i, row] = lpnt[k]
        row += 1
    for k in range(3):
        out[i, row] = lvec[k]
        row += 1


def main():
    rays_raw = np.frombuffer((OUT / "cam2-ray.f32le").read_bytes(), dtype="<f4").reshape(-1, 6)
    dirs = rays_raw[:, :3].astype(np.float32)
    seg = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/seg.npz")["seg_cam2"].reshape(-1, 2)
    flexmask = seg[:, 1] == 9
    flex1 = flexmask & (seg[:, 0] == 1)
    print("flex1 px", int(flex1.sum()))

    state = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/state.npz")
    flexvert = state["flexvert_xpos"][0].astype(np.float32)
    poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
    f32of = lambda b: np.uint32(b).view(np.float32)
    cam_xpos = np.array([f32of(b) for b in poses["poses"]["cam_xpos_bits"][6:9]], dtype=np.float32)
    cam_xmat = np.array([f32of(b) for b in poses["poses"]["cam_xmat_bits"][18:27]], dtype=np.float32).reshape(3, 3)
    radius = np.array([0.02, 0.04], dtype=np.float32)

    wp_rays = wp.array(dirs, dtype=wp.vec3, device="cuda")
    wp_mat = wp.array(np.stack([cam_xmat]), dtype=wp.mat33, device="cuda")
    wp_pos = wp.array(np.stack([cam_xpos]), dtype=wp.vec3, device="cuda")
    wp_verts = wp.array(flexvert.astype(np.float32), dtype=wp.vec3, device="cuda")
    wp_radius = wp.array(radius, dtype=float, device="cuda")
    out = wp.zeros((len(dirs), SLOTS), dtype=float, device="cuda")
    outd = wp.zeros(len(dirs), dtype=float, device="cuda")
    wp.launch(
        flex_chain,
        dim=len(dirs),
        inputs=[wp_rays, wp_mat, wp_pos, wp_verts, wp_radius, 4, 5, 1, out, outd],
        device="cuda",
    )
    wp.synchronize()

    vals = out.numpy().astype(np.float32)
    d = outd.numpy().astype(np.float32)
    dirz = dirs[:, 2]
    depth_pred = d * (-dirz)
    ref_depth = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].reshape(-1).astype(np.float32)
    prod_depth = np.frombuffer(
        (ROOT / "target/render-bvh/r06-s3b-flex-window/d8/depth_all.f32le").read_bytes(), dtype="<f4"
    )[120000:196800].astype(np.float32)

    m = flex1
    bp = depth_pred[m].view(np.uint32)
    br = ref_depth[m].view(np.uint32)
    bq = prod_depth[m].view(np.uint32)
    print("upstream-replay vs REFERENCE depth mismatched:", int((bp != br).sum()), "/", int(m.sum()))
    print("upstream-replay vs PRODUCT   depth mismatched:", int((bp != bq).sum()), "/", int(m.sum()))
    print("PRODUCT vs REFERENCE         depth mismatched:", int((bq != br).sum()), "/", int(m.sum()))
    up = np.abs(bp.astype(np.int64) - br.astype(np.int64))
    print("replay-vs-ref ULP median", float(np.median(up)), "max", int(up.max()))
    up2 = np.abs(bp.astype(np.int64) - bq.astype(np.int64))
    print("replay-vs-prod ULP median", float(np.median(up2)), "max", int(up2.max()))

    np.savez(
        OUT / "upstream-flex1-chain.npz",
        dirs=dirs[m],
        values=vals[m],
        d=d[m],
        depth_pred=depth_pred[m],
        ref_depth=ref_depth[m],
        prod_depth=prod_depth[m],
        cam_xpos=cam_xpos,
        cam_xmat=cam_xmat,
        flexvert=flexvert,
    )
    print("saved", OUT / "upstream-flex1-chain.npz")


if __name__ == "__main__":
    main()
