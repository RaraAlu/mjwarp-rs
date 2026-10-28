"""R05-S3a reverse calculation: does this fixture hit the f32(1e-5) threshold?

CPU only (mujoco model load + numpy float32 arithmetic; no GPU, no warp).
Reproduces evidence README section 12: per-cell planarity diff in f32 on the
actual MuJoCo-normalized hfield data, boundary distance to f32(1e-5) =
0x3727C5AC, fits_plane residual check and the upstream greedy triangulation
shape (vertices/indices/triangles).

Usage:
  target\\mjwarp-parallel\\venv\\Scripts\\python.exe -B \
      docs/development-evidence/render-bvh/r05-s3a-hfield-capture/materials/reverse-calc.py
"""

import pathlib

import mujoco
import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[5]
SCENE = ROOT / "fixtures/render-bvh/scene-hfield/scene.xml"
THRESHOLD = np.float32(1e-5)


def main() -> int:
    model = mujoco.MjModel.from_xml_path(str(SCENE))
    nr = int(model.hfield_nrow[0])
    nc = int(model.hfield_ncol[0])
    adr = int(model.hfield_adr[0])
    flat = np.asarray(model.hfield_data[adr : adr + nr * nc], dtype=np.float32)
    # Upstream bvh.py reshapes the flat store as (nr, nc) row-major.
    data = flat.reshape((nr, nc))
    print("scene=", SCENE.relative_to(ROOT).as_posix())
    print("nrow=%d ncol=%d size=%s" % (nr, nc, list(map(float, model.hfield_size[0]))))
    print("threshold bits=0x%08X value=%.9g" % (THRESHOLD.view(np.uint32), float(THRESHOLD)))
    print("stored_flat_hex=", [hex(int(v)) for v in flat.view(np.uint32)])

    print("\n[cell diffs] (f32, upstream formula)")
    diffs = []
    for r in range(nr - 1):
        for c in range(nc - 1):
            d = np.float32(
                np.float32(data[r, c] + data[r + 1, c + 1])
                - np.float32(data[r, c + 1] + data[r + 1, c])
            )
            diffs.append(d)
            dist = abs(float(abs(d)) - float(THRESHOLD))
            print(
                "cell(%d,%d) diff=%+.9g bits=0x%08X planar=%s |abs-thr|=%.3g"
                % (r, c, float(d), int(abs(d).view(np.uint32)), bool(abs(d) < THRESHOLD), dist)
            )
    print("any_exact_threshold=", any(abs(float(abs(d))) == float(THRESHOLD) for d in diffs))
    print("min_|abs-thr|=", min(abs(float(abs(d)) - float(THRESHOLD)) for d in diffs))

    visited = np.zeros((nr - 1, nc - 1), dtype=bool)
    points = []
    pmap = {}
    indices = []
    residuals = []

    def point_index(r, c):
        if (r, c) in pmap:
            return pmap[(r, c)]
        index = len(points)
        pmap[(r, c)] = index
        points.append((r, c))
        return index

    def planar(a, b, c, d):
        diff = np.float32(np.float32(a + d) - np.float32(b + c))
        return bool(abs(diff) < THRESHOLD)

    for r in range(nr - 1):
        for c in range(nc - 1):
            if visited[r, c]:
                continue
            z00, z01, z10, z11 = data[r, c], data[r, c + 1], data[r + 1, c], data[r + 1, c + 1]
            if not planar(z00, z01, z10, z11):
                i00 = point_index(r, c)
                i01 = point_index(r, c + 1)
                i10 = point_index(r + 1, c)
                i11 = point_index(r + 1, c + 1)
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
            itl = point_index(r, c)
            itr = point_index(r, c + width)
            ibl = point_index(r + height, c)
            ibr = point_index(r + height, c + width)
            indices += [itl, itr, ibr, itl, ibr, ibl]

    print(
        "\n[triangulation] vertices=%d indices=%d tris=%d" % (len(points), len(indices), len(indices) // 3)
    )
    if residuals:
        print(
            "[fits residuals] count=%d max=%.9g min_nonzero=%.9g boundary_hits=%d"
            % (
                len(residuals),
                max(residuals),
                min([x for x in residuals if x > 0], default=0.0),
                sum(1 for x in residuals if x == float(THRESHOLD)),
            )
        )
    else:
        print("[fits residuals] none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
