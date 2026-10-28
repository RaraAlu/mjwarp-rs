"""R05-S3a reverse calculation on the EFFECTIVE upstream grid (CPU only).

Determines the MuJoCo/upstream data convention empirically, then runs the
upstream-aligned triangulation (f32 thresholds, greedy coplanar merge) on the
effective grid, checks the f32(1e-5)=0x3727C5AC boundary, and proves the fixture
is transpose-sensitive (transposing the effective grid changes the result).

No GPU, no warp: only mujoco model load + numpy float32 arithmetic.

Usage:
  target\\mjwarp-parallel\\venv\\Scripts\\python.exe -B \
      docs/development-evidence/render-bvh/r05-s3a-hfield-capture/materials/reverse-calc.py
"""

import hashlib
import pathlib
import xml.etree.ElementTree as ET

import mujoco
import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[5]
SCENE = ROOT / "fixtures/render-bvh/scene-hfield/scene.xml"
THRESHOLD = np.float32(1e-5)


def xml_grid(scene):
    root = ET.parse(scene).getroot()
    hfield = root.find(".//hfield")
    nrow = int(hfield.get("nrow"))
    ncol = int(hfield.get("ncol"))
    values = [float(v) for v in hfield.get("elevation").split()]
    assert len(values) == nrow * ncol, "elevation count"
    return nrow, ncol, np.asarray(values, dtype=np.float64).reshape((nrow, ncol))


def candidates(nrow, ncol):
    """Candidate stored layouts: normalization rounding x flatten order x flip transform.

    MuJoCo normalizes the XML elevation to [0,1] and has an image-origin convention;
    the script determines empirically which (normalization, layout) reproduces the
    stored f32 bits exactly, instead of assuming one.
    """
    grid = xml_grid(SCENE)[2]
    lo, hi = grid.min(), grid.max()
    norm64 = (grid - lo) / (hi - lo)
    g32 = grid.astype(np.float32)
    norm32 = (g32 - np.float32(lo)) / (np.float32(hi) - np.float32(lo))
    normals = {"norm_f64_then_f32": norm64.astype(np.float32), "norm_f32_math": norm32}
    flips = {
        "identity": lambda x: x,
        "flipud": np.flipud,
        "fliplr": np.fliplr,
        "rot180": lambda x: np.flipud(np.fliplr(x)),
        "transpose": lambda x: x.T,
        "transpose_flipud": lambda x: np.flipud(x.T),
        "transpose_fliplr": lambda x: np.fliplr(x.T),
    }
    variants = {}
    for normal_name, normal in normals.items():
        for flip_name, flip in flips.items():
            variants[f"{normal_name}+{flip_name}"] = np.ascontiguousarray(
                flip(normal)).flatten(order="C")
    return variants


def triangulate(data):
    """Upstream _optimize_hfield_mesh shape (points/indices/residuals), f32 thresholds."""
    nr, nc = data.shape
    visited = np.zeros((nr - 1, nc - 1), dtype=bool)
    points, pmap, indices, residuals = [], {}, [], []

    def point_index(r, c):
        if (r, c) in pmap:
            return pmap[(r, c)]
        pmap[(r, c)] = len(points)
        points.append((r, c))
        return pmap[(r, c)]

    def planar(a, b, c, d):
        diff = np.float32(np.float32(a + d) - np.float32(b + c))
        return bool(abs(diff) < THRESHOLD)

    for r in range(nr - 1):
        for c in range(nc - 1):
            if visited[r, c]:
                continue
            z00, z01, z10, z11 = data[r, c], data[r, c + 1], data[r + 1, c], data[r + 1, c + 1]
            if not planar(z00, z01, z10, z11):
                i00, i01 = point_index(r, c), point_index(r, c + 1)
                i10, i11 = point_index(r + 1, c), point_index(r + 1, c + 1)
                indices += [i00, i01, i11, i00, i11, i10]
                visited[r, c] = True
                continue
            slope_x = np.float32(z01 - z00)
            slope_y = np.float32(z10 - z00)

            def fits(rr, cc):
                if rr >= nr - 1 or cc >= nc - 1:
                    return False
                a, b, c0, d0 = data[rr, cc], data[rr, cc + 1], data[rr + 1, cc], data[rr + 1, cc + 1]
                if not planar(a, b, c0, d0):
                    return False
                predicted = np.float32(
                    np.float32(z00 + np.float32(np.float32(rr - r) * slope_y))
                    + np.float32(np.float32(cc - c) * slope_x)
                )
                d1 = np.float32(a - predicted)
                d2 = np.float32(np.float32(b - a) - slope_x)
                d3 = np.float32(np.float32(c0 - a) - slope_y)
                residuals.extend([abs(float(d1)), abs(float(d2)), abs(float(d3))])
                return bool(abs(d1) < THRESHOLD and abs(d2) < THRESHOLD and abs(d3) < THRESHOLD)

            width = 1
            while c + width < nc - 1 and not visited[r, c + width] and fits(r, c + width):
                width += 1
            height = 1
            while r + height < nr - 1:
                ok = True
                for k in range(width):
                    if visited[r + height, c + k] or not fits(r + height, c + k):
                        ok = False
                        break
                if ok:
                    height += 1
                else:
                    break
            for rr in range(r, r + height):
                for cc in range(c, c + width):
                    visited[rr, cc] = True
            itl, itr = point_index(r, c), point_index(r, c + width)
            ibl, ibr = point_index(r + height, c), point_index(r + height, c + width)
            indices += [itl, itr, ibr, itl, ibr, ibl]
    return points, indices, residuals


def coordinates_checksum(data, points):
    """Rows/cols -> local x/y/z (sx,sy,sz2) -> f32 bytes -> sha256 (order sensitive)."""
    sx, sy, sz = np.float32(1.5), np.float32(1.0), np.float32(0.25)
    width, height = np.float32(0.5 * 3), np.float32(0.5 * 3)
    payload = bytearray()
    for (r, c) in points:
        x = np.float32(sx * np.float32(np.float32(c) / width - np.float32(1.0)))
        y = np.float32(sy * np.float32(np.float32(r) / height - np.float32(1.0)))
        z = np.float32(data[r, c] * sz)
        payload += np.asarray([x, y, z], dtype=np.float32).tobytes()
    return hashlib.sha256(bytes(payload)).hexdigest()


def main() -> int:
    model = mujoco.MjModel.from_xml_path(str(SCENE))
    nr, nc = int(model.hfield_nrow[0]), int(model.hfield_ncol[0])
    adr = int(model.hfield_adr[0])
    flat = np.asarray(model.hfield_data[adr : adr + nr * nc], dtype=np.float32)
    print("scene=", SCENE.relative_to(ROOT).as_posix())
    print("nrow=%d ncol=%d size=%s" % (nr, nc, list(map(float, model.hfield_size[0]))))
    print("threshold bits=0x%08X value=%.9g" % (THRESHOLD.view(np.uint32), float(THRESHOLD)))
    print("\n[1] mjm.hfield_data raw flat order (f32 bits, memory order)")
    print([hex(int(v)) for v in flat.view(np.uint32)])

    print("\n[2] which layout matches the stored flat array? (bitwise)")
    matched = []
    for name, values in candidates(nr, nc).items():
        equal = bool(np.array_equal(values.view(np.uint32), flat.view(np.uint32)))
        mismatch = int((values.view(np.uint32) != flat.view(np.uint32)).sum())
        print("  %-26s bitwise_equal=%s mismatches=%d" % (name, equal, mismatch))
        if equal:
            matched.append(name)
    print("  matched_layouts=", matched if matched else "NONE")

    effective = flat.reshape((nr, nc))  # upstream bvh.py reshape((nr, nc)) row-major
    print("\n[3] effective grid (upstream reshape row-major over stored flat)")
    print(effective)

    print("\n[4] cell planarity on effective grid (upstream formula, f32)")
    diffs = []
    for r in range(nr - 1):
        for c in range(nc - 1):
            diff = np.float32(
                np.float32(effective[r, c] + effective[r + 1, c + 1])
                - np.float32(effective[r, c + 1] + effective[r + 1, c])
            )
            diffs.append(diff)
            print("  cell(%d,%d) diff=%+.9g bits=0x%08X planar=%s"
                  % (r, c, float(diff), int(abs(diff).view(np.uint32)), bool(abs(diff) < THRESHOLD)))
    print("  any_exact_threshold=", any(abs(float(abs(d))) == float(THRESHOLD) for d in diffs))
    print("  min_|abs-thr|=", min(abs(float(abs(d)) - float(THRESHOLD)) for d in diffs))

    points, indices, residuals = triangulate(effective)
    print("\n[5] triangulation on effective grid")
    print("  vertices=%d indices=%d tris=%d" % (len(points), len(indices), len(indices) // 3))
    print("  coords_sha256=", coordinates_checksum(effective, points))
    if residuals:
        print("  fits_residuals: count=%d max=%.9g min_nonzero=%.9g boundary_hits=%d"
              % (len(residuals), max(residuals), min([x for x in residuals if x > 0], default=0.0),
                 sum(1 for x in residuals if x == float(THRESHOLD))))

    tpoints, tindices, tresiduals = triangulate(np.ascontiguousarray(effective.T))
    tchecksum = coordinates_checksum(np.ascontiguousarray(effective.T), tpoints)
    print("\n[6] transpose sensitivity (same triangulation on effective grid^T)")
    print("  transposed: vertices=%d indices=%d tris=%d coords_sha256=%s"
          % (len(tpoints), len(tindices), len(tindices) // 3, tchecksum))
    distinguishable = (len(tpoints), len(tindices), tchecksum) != (len(points), len(indices),
                                                                  coordinates_checksum(effective, points))
    print("  transpose_distinguishable=", distinguishable)
    if tresiduals:
        print("  transposed fits max=%.9g" % max(tresiduals))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
