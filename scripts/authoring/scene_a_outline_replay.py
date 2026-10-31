"""Replay helpers for the scene-a sphere-outline authoring tool (offline, zero GPU).

This module is part of the offline authoring exception (user-approved 2026-10-22); it is
never referenced by the product build, product tests, cargo gates or product scripts.

It replays the product f32 sphere-intersection math (`render_math.cu`
`render_sphere_hit`) with exact single-rounding arithmetic in the two `a = dot(vec,vec)`
association forms:

  * form A ("ch0"): a = fma(z,z, fma(y,y, mul(x,x)))  -- the scene-a reference form
  * form B ("ba") : a = fma(z,z, fma(x,x, mul(y,y))) -- the product form after the refix

Camera-space rays are rebuilt from the frozen camera model (`compute_ray`, ported from
`render/camera/cuda/build_rays.cu`). World rays use the frozen `cam_xmat`/`cam_xpos` bits
from each reference's `camera_data_check`. The aa2 arm (samples_per_pixel=2) replays
sub-sample 0 only, matching the product rule that depth/seg are written on the first pass
only (`render_megakernel.cu`: "Depth and seg are single-sample outputs: only the first
pass writes them"); sub-sample 0 of pixel (x, y) uses ray (2x, 2y) on the doubled grid.
"""

from __future__ import annotations

import hashlib
import json
import math
import zipfile
from fractions import Fraction
from pathlib import Path

import numpy as np

F32 = np.float32
U = 2.0 ** -24

# Frozen input pins (sha256): freeze-time drift must fail loudly instead of silently
# changing the exemption lists.
PINS = {
    "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json":
        "d6ab6dde58034df5c1bc69ab397026a8f7c3aaf58d29d25edb032d3818000dc7",
    "fixtures/render-bvh/scene-a/s7b/flags/aa2_depth_seg.json":
        "79a275eff5d45f6bef764957e5f4987d0e6abfbb9b7a2cb578c8e1f717d4b7e1",
    "fixtures/render-bvh/scene-a/reference-skybox/depth.npz":
        "28c1ea4bdd4533a3d4811f390051a9e0dfa75ccab2549d7b32b8b426ccc9ba14",
    "fixtures/render-bvh/scene-a/reference-skybox/seg.npz":
        "2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97",
    "fixtures/render-bvh/scene-a/model-312.json":
        "cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00",
    "fixtures/render-bvh/scene-a/s2/reference_hits.json":
        "609ffc527e8216654cb7b6576f998659026d0e5324429c31c7a67f058c225f84",
    "fixtures/render-bvh/scene-a/s5/camera_rays.json":
        "545a1442fc5ac67d8de9abd6d7f7cf0a24e5a86baa3b473f660d8aae83842022",
}

RECIPE_REFERENCE = "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json"
AA2_REFERENCE = "fixtures/render-bvh/scene-a/s7b/flags/aa2_depth_seg.json"
SKYBOX_DEPTH = "fixtures/render-bvh/scene-a/reference-skybox/depth.npz"
SKYBOX_SEG = "fixtures/render-bvh/scene-a/reference-skybox/seg.npz"
MODEL = "fixtures/render-bvh/scene-a/model-312.json"
HITS = "fixtures/render-bvh/scene-a/s2/reference_hits.json"
S5 = "fixtures/render-bvh/scene-a/s5/camera_rays.json"


# --- exact f32 helpers -------------------------------------------------------
def _round_f32(exact: Fraction):
    """Round an exact Fraction to nearest f32 (ties to even)."""
    if exact == 0:
        return F32(0.0)
    sign = -1 if exact < 0 else 1
    value = abs(exact)
    e = 0
    while value >= Fraction(2) ** (e + 1):
        e += 1
    while value < Fraction(2) ** e:
        e -= 1
    if e >= -126:
        q = value / Fraction(2) ** (e - 23)
        n = int(q)
        rem = q - n
        if rem > Fraction(1, 2) or (rem == Fraction(1, 2) and n % 2 == 1):
            n += 1
        return F32(sign * float(Fraction(n) * Fraction(2) ** (e - 23)))
    return F32(sign * float(value))


def fma32(a, b, c):
    return _round_f32(Fraction(float(F32(a))) * Fraction(float(F32(b))) + Fraction(float(F32(c))))


def mul32(a, b):
    return F32(F32(a) * F32(b))


def div32(a, b):
    return F32(np.float64(F32(a)) / np.float64(F32(b)))


def sub32(a, b):
    return F32(F32(a) - F32(b))


def add32(a, b):
    return F32(F32(a) + F32(b))


def sqrt32(x):
    return F32(np.sqrt(np.float64(F32(x))))


def tanf(x):
    return F32(math.tan(np.float64(F32(x))))


def bits(value):
    return int(np.asarray(F32(value)).view(np.uint32))


def frombits(value):
    return F32(np.asarray(np.uint32(int(value))).view(np.float32).item())


DEG_TO_RAD = F32(0.017453292519943295)


# --- camera-space ray port (render/camera/cuda/build_rays.cu) ----------------
def normalize3(a):
    length = sqrt32(fma32(a[2], a[2], fma32(a[1], a[1], mul32(a[0], a[0]))))
    if length > F32(0.0):
        return [div32(a[0], length), div32(a[1], length), div32(a[2], length)]
    return [F32(0.0), F32(0.0), F32(0.0)]


def compute_ray(proj, fovy, ssx, ssy, intr, w, h, px, py, znear):
    """Camera-space direction for one pixel; perspective intrinsic/fovy branches."""
    inv_img_h = div32(F32(1.0), F32(h))
    if proj == 1:
        raise SystemExit("orthographic camera is not part of the scene-a fixture")
    aspect = mul32(F32(w), inv_img_h)
    sensor_h = F32(ssy)
    if sensor_h != F32(0.0):
        fx, fy, cx, cy = (F32(intr[i]) for i in range(4))
        sensor_w = F32(ssx)
        sensor_aspect = div32(sensor_w, sensor_h)
        if aspect > sensor_aspect:
            sensor_h = div32(sensor_w, aspect)
        elif aspect < sensor_aspect:
            sensor_w = mul32(sensor_h, aspect)
        inv_fx_znear = div32(F32(znear), fx)
        inv_fy_znear = div32(F32(znear), fy)
        half_sensor_w = mul32(F32(0.5), sensor_w)
        half_sensor_h = mul32(F32(0.5), sensor_h)
        left = mul32(sub32(F32(0.0), inv_fx_znear), sub32(half_sensor_w, cx))
        right = mul32(inv_fx_znear, add32(half_sensor_w, cx))
        top = mul32(inv_fy_znear, sub32(half_sensor_h, cy))
        bottom = mul32(sub32(F32(0.0), inv_fy_znear), add32(half_sensor_h, cy))
    else:
        fovy_rad = mul32(F32(fovy), DEG_TO_RAD)
        half_height = mul32(F32(znear), tanf(mul32(F32(0.5), fovy_rad)))
        half_width = mul32(half_height, aspect)
        left = sub32(F32(0.0), half_width)
        right = half_width
        top = half_height
        bottom = sub32(F32(0.0), half_height)
    u = div32(add32(F32(px), F32(0.5)), F32(w))
    v = mul32(add32(F32(py), F32(0.5)), inv_img_h)
    x = fma32(sub32(right, left), u, left)
    y = fma32(sub32(bottom, top), v, top)
    return normalize3([x, y, sub32(F32(0.0), F32(znear))])


# --- sphere intersection (render_math.cu::render_sphere_hit) -----------------
def dot_ch0(a, b):
    return fma32(a[2], b[2], fma32(a[1], b[1], mul32(a[0], b[0])))


def dot_ba(a, b):
    return fma32(a[2], b[2], fma32(a[0], b[0], mul32(a[1], b[1])))


def mat_vec(mat, v):
    """Row-major mat * vec as contracted by the frozen kernel: fma(2, fma(0, mul(1)))."""
    return [fma32(mat[3 * r + 2], v[2], fma32(mat[3 * r], v[0], mul32(mat[3 * r + 1], v[1])))
            for r in range(3)]


def sphere_hit(local_dir, xmat, cam_pos, sphere_pos, r2, form):
    world_dir = mat_vec(xmat, local_dir)
    d = [sub32(cam_pos[i], sphere_pos[i]) for i in range(3)]
    dot_a = dot_ch0 if form == "A" else dot_ba
    a = dot_a(world_dir, world_dir)
    b = dot_ba(world_dir, d)
    dd = dot_ba(d, d)
    c = sub32(dd, r2)
    det = fma32(b, b, -mul32(a, c))
    if det < F32(1.0e-15):
        return F32(-1.0), (det, a, b, c)
    sq = sqrt32(det)
    den = F32(1.0 / np.float64(a))
    nb = sub32(F32(0.0), b)
    x0 = mul32(den, sub32(nb, sq))
    x1 = mul32(den, sub32(sq, b))
    sol = x0 if x0 >= 0 else (x1 if x1 >= 0 else F32(-1.0))
    if sol < 0:
        return F32(-1.0), (det, a, b, c)
    return mul32(sol, F32(-local_dir[2])), (det, a, b, c)


# --- frozen fixtures --------------------------------------------------------
def load_json(root, rel):
    return json.loads((root / rel).read_text(encoding="utf-8"))


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def check_pins(root):
    bad = []
    for rel, want in PINS.items():
        got = sha256_file(root / rel)
        if got != want:
            bad.append((rel, want, got))
    return bad


def read_npz(root, path):
    """Read arrays of a fixture npz without importing the reference package."""
    out = {}
    with zipfile.ZipFile(root / path) as archive:
        for name in archive.namelist():
            if not name.endswith(".npy"):
                continue
            with archive.open(name) as handle:
                magic = np.lib.format.read_magic(handle)
                if magic == (1, 0):
                    shape, fortran, dtype = np.lib.format.read_array_header_1_0(handle)
                else:
                    shape, fortran, dtype = np.lib.format.read_array_header_2_0(handle)
                data = np.frombuffer(handle.read(), dtype=dtype).reshape(
                    shape, order="F" if fortran else "C")
                out[name.split(".")[0]] = data
    return out


class Scene:
    """Frozen model geometry/camera data needed by the replay."""

    def __init__(self, root: Path):
        model = load_json(root, MODEL)
        arrays = model["arrays"]
        self.fovy = [float(v) for v in arrays["cam_fovy"]["values"]]
        self.sensorsize = [float(v) for v in arrays["cam_sensorsize"]["values"]]
        self.intrinsic = [float(v) for v in arrays["cam_intrinsic"]["values"]]
        self.projection = [int(v) for v in arrays["cam_projection"]["values"]]
        # Product semantics: znear = f64(f32 vis.map.znear) * f64(f32 stat.extent) -> f32
        # (src/render/context/model.rs). Round the f32 operands first: the exact product of
        # 0.00999999978f * 2.5f is a f32 tie and rounds to even (= 0.0249999985f).
        znear_raw = F32(arrays["vis.map.znear"]["values"][0])
        extent_raw = F32(arrays["stat.extent"]["values"][0])
        self.znear = F32(float(znear_raw) * float(extent_raw))
        hits = load_json(root, HITS)
        self.enabled = [int(v) for v in hits["enabled_geom_ids"]]
        geom_type = [int(v) for v in arrays["geom_type"]["values"]]
        geom_size = [float(v) for v in arrays["geom_size"]["values"]]
        self.sphere_geom = [g for g in self.enabled if geom_type[g] == 2][0]
        slot = self.enabled.index(self.sphere_geom)
        xpos = hits["cases"]["initial"]["geom_xpos_bits"]
        self.sphere_pos = [frombits(int(xpos[3 * slot + i])) for i in range(3)]
        radius = geom_size[3 * self.sphere_geom]
        self.r2 = mul32(radius, radius)
        self._dirs = {}

    def local_dir(self, cam, w, h, px, py):
        key = (cam, w, h, px, py)
        if key not in self._dirs:
            self._dirs[key] = compute_ray(
                self.projection[cam], self.fovy[cam], self.sensorsize[2 * cam],
                self.sensorsize[2 * cam + 1], self.intrinsic[4 * cam:4 * cam + 4],
                w, h, px, py, self.znear)
        return self._dirs[key]


class Reference:
    """One camera of one frozen depth/seg reference plus its frozen camera pose."""

    def __init__(self, root: Path, rel: str, cam_index: int):
        doc = load_json(root, rel)
        cam = doc["cameras"][cam_index]
        self.width = int(cam["width"])
        self.height = int(cam["height"])
        self.pixels = self.width * self.height
        self.index = cam_index
        self.depth = [int(v) for v in cam["depth_bits"]]
        self.seg = [int(v) for v in cam["seg_pairs"]]
        pose = doc["camera_data_check"][cam_index]
        self.cam_pos = [frombits(int(v)) for v in pose["origin_bits"]]
        self.xmat = [frombits(int(v)) for v in pose["xmat_bits"]]
        self.sha = sha256_file(root / rel)


def sphere_pixels(scene: Scene, ref: Reference):
    return [p for p in range(ref.pixels) if ref.seg[2 * p] == scene.sphere_geom]


def replay(scene, ref, camera, spp, form, pixels):
    """Depth value + hit terms for the given pixels (sub-sample 0 when spp > 1)."""
    out = {}
    for pixel in pixels:
        x, y = pixel % ref.width, pixel // ref.width
        local = scene.local_dir(camera, ref.width * spp, ref.height * spp, x * spp, y * spp)
        value, terms = sphere_hit(local, ref.xmat, ref.cam_pos, scene.sphere_pos, scene.r2, form)
        out[pixel] = (value, terms, local)
    return out


def band4_pixels(scene, ref):
    """Criterion v2 section 2 condition 2 as written: 4-neighbour non-sphere hit."""
    out = []
    for pixel in sphere_pixels(scene, ref):
        x, y = pixel % ref.width, pixel // ref.width
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < ref.width and 0 <= ny < ref.height and \
                    ref.seg[2 * (ny * ref.width + nx)] != scene.sphere_geom:
                out.append(pixel)
                break
    return out


def next_pow2(value):
    return 2 ** math.ceil(math.log2(value))


def bound_v2(pixels, depths_a, terms_a, terms_b, dirs):
    """Criterion v2 section-3 bound: global extremes + per-pixel maximum."""
    rows = []
    per_pixel = []
    for pixel in pixels:
        det_a = terms_a[pixel][0]
        det_b = terms_b[pixel][0]
        if det_a <= 0 or det_b <= 0 or depths_a[pixel] <= 0:
            continue
        sqrt_det = math.sqrt(min(float(det_a), float(det_b)))
        c_abs = abs(float(terms_a[pixel][3]))
        vz = abs(float(dirs[pixel][2]))
        depth = float(depths_a[pixel])
        rows.append((sqrt_det, c_abs, vz, depth))
        ulp = 2.0 ** (math.floor(math.log2(depth)) - 23)
        per_pixel.append(5.0 * U * c_abs * vz / (2.0 * (1.0 - 4.0 * U) * sqrt_det) / ulp)
    sqrt_det_min = min(r[0] for r in rows)
    c_max = max(r[1] for r in rows)
    vz_max = max(r[2] for r in rows)
    depth_min = min(r[3] for r in rows)
    ulp_depth = 2.0 ** (math.floor(math.log2(depth_min)) - 23)
    raw = 5.0 * U * c_max * vz_max / (2.0 * (1.0 - 4.0 * U) * sqrt_det_min) / ulp_depth
    return {
        "sqrt_det_min": sqrt_det_min, "c_max": c_max, "vz_max": vz_max,
        "depth_min": depth_min, "ulp_depth": ulp_depth, "raw": raw,
        "with_margin": raw * 1.15, "pow2": next_pow2(raw * 1.15),
        "per_pixel_max": max(per_pixel) if per_pixel else 0.0,
    }
