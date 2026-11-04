"""Calibrate the offline cam2 ray model against reference rigid-geometry depth.

If emulated rays/camera pose are right, predicted plane/box hit distances must match the
reference depth to ~1e-7 relative (f32 kernel rounding only). This separates ray-model
error from flex-chain error. Read-only; writes target/s3b-rca/.
"""
import json
import pathlib

import numpy as np

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
F32 = np.float32


def fma(a, b, c):
    return F32(np.float64(a) * np.float64(b) + np.float64(c))


def bits_arr(bits):
    return np.array([np.uint32(b).view(np.float32) for b in bits], dtype=np.float64)


def cam2_rays(w=320, h=240, fovy_deg=45.0, znear=0.024999999441206455):
    deg = F32(0.017453292519943295)
    znear = F32(znear)
    fovy = F32(fovy_deg)
    inv_img_h = F32(1.0) / F32(h)
    aspect = F32(w) * inv_img_h
    fovy_rad = F32(fovy * deg)
    half_h = F32(znear * F32(np.tan(np.float64(F32(0.5) * fovy_rad))))
    half_w = F32(half_h * aspect)
    left, right = F32(-half_w), F32(half_w)
    top, bottom = F32(half_h), F32(-half_h)
    dw = F32(right - left)
    dh = F32(bottom - top)
    dirs = np.empty((h, w, 3), dtype=np.float32)
    for py in range(h):
        v = F32(F32(py) + F32(0.5)) * inv_img_h
        y = fma(dh, v, top)
        for px in range(w):
            u = F32(F32(px) + F32(0.5)) / F32(w)
            x = fma(dw, u, left)
            z = F32(-znear)
            length = F32(np.sqrt(np.float64(fma(z, z, fma(y, y, F32(x * x))))))
            dirs[py, px] = (F32(x / length), F32(y / length), F32(z / length))
    return dirs


def main():
    poses = json.loads((ROOT / "fixtures/render-bvh/scene-flex/reference/base/cam-poses.json").read_text(encoding="utf-8"))
    state = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/state.npz")
    ref_depth = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/depth.npz")["depth_cam2"].astype(np.float64)
    seg = np.load(ROOT / "fixtures/render-bvh/scene-flex/reference/base/seg.npz")["seg_cam2"]
    model = json.loads((ROOT / "fixtures/render-bvh/scene-flex/model-312.json").read_text(encoding="utf-8"))
    arr = model["arrays"]
    geom_type = [int(v) for v in arr["geom_type"]["values"]]
    geom_size = np.array(arr["geom_size"]["values"], dtype=np.float64).reshape(-1, 3)
    geom_pos = np.array(arr["geom_pos"]["values"], dtype=np.float64).reshape(-1, 3)
    geom_xpos = state["geom_xpos"][0].astype(np.float32).astype(np.float64)
    geom_xmat = state["geom_xmat"][0].astype(np.float32).astype(np.float64).reshape(-1, 3, 3)

    dirs = cam2_rays().reshape(-1, 3).astype(np.float64)
    dirz = dirs[:, 2]
    cam_pos = bits_arr(poses["poses"]["cam_xpos_bits"][6:9])
    cam_mat = bits_arr(poses["poses"]["cam_xmat_bits"][18:27]).reshape(3, 3)
    dirw = dirs @ cam_mat.T
    dirw /= np.linalg.norm(dirw, axis=1, keepdims=True)
    orig = np.broadcast_to(cam_pos, (len(dirs), 3))

    flat_seg = seg.reshape(-1, 2)
    print("geom types", geom_type)
    for gid in range(len(geom_type)):
        m = (flat_seg[:, 0] == gid) & (flat_seg[:, 1] == 5)
        n = int(m.sum())
        if n == 0:
            continue
        t = geom_type[gid]
        o = orig[m]
        d = dirw[m]
        if t == 0:  # plane: local z=0, normal = R[:,2]
            nrm = geom_xmat[gid][:, 2]
            center = geom_xpos[gid]
            denom = d @ nrm
            num = (center - o) @ nrm
            with np.errstate(divide="ignore", invalid="ignore"):
                dist = num / denom
        elif t == 6:  # box: local space slab test
            R = geom_xmat[gid]
            size = geom_size[gid]
            # ray_box: local ray map, slab intersection, then distances
            # simple f64 approximation is enough for calibration
            lo = (o - geom_xpos[gid]) @ R
            ld = d @ R
            tmin, tmax = 0.0, np.inf
            with np.errstate(divide="ignore", invalid="ignore"):
                inv = 1.0 / ld
                t1 = (-size[0] - lo[:, 0]) * inv[:, 0]
                t2 = (size[0] - lo[:, 0]) * inv[:, 0]
                tmin = np.maximum(tmin, np.minimum(t1, t2))
                tmax = np.minimum(tmax, np.maximum(t1, t2))
                for k in (1, 2):
                    t1 = (-size[k] - lo[:, k]) * inv[:, k]
                    t2 = (size[k] - lo[:, k]) * inv[:, k]
                    tmin = np.maximum(tmin, np.minimum(t1, t2))
                    tmax = np.minimum(tmax, np.maximum(t1, t2))
            dist = np.where((tmax >= tmin) & (tmin >= 0), tmin, np.nan)
        else:
            print(f"geom {gid} type {t}: {n} px (skip)")
            continue
        pred_depth = dist * (-dirz[m])
        ref = ref_depth.reshape(-1)[m]
        ok = np.isfinite(pred_depth) & (ref > 0)
        rel = (pred_depth[ok] - ref[ok]) / ref[ok]
        print(f"geom {gid} type {t}: {n} px, finite {int(ok.sum())}, rel diff median {np.median(rel):.3e} "
              f"p90 {np.percentile(np.abs(rel), 90):.3e} max {np.max(np.abs(rel)):.3e}")


if __name__ == "__main__":
    main()
