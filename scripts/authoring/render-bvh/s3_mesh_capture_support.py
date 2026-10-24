"""S3 authoring support: reference-capture kernel, device readback and ray design.

OFFLINE ONLY: imported by `capture_s3_mesh.py`, never by product builds/tests/gates.

The reference kernel calls the frozen upstream `ray_mesh_with_bvh` `@wp.func`
exactly as the render path does; `cast` launches it and returns f32 bit patterns.
The ray design is deterministic and derived from device-side f32 mesh/present pose
bits so the fixture is reproducible.
"""

from __future__ import annotations

import numpy as np
import warp as wp

from mujoco_warp._src.ray import ray_mesh_with_bvh

DEGENERATE_VERTICES = np.array(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
    dtype=np.float32,
)
# [0,1,2] collinear, [1,1,1] fully collapsed, the rest valid.
DEGENERATE_INDICES = np.array([0, 1, 2, 0, 1, 3, 0, 2, 4, 3, 4, 0, 1, 1, 1], dtype=np.int32)


@wp.kernel
def _mesh_query_kernel(
    mesh_bvh_id: wp.array[wp.uint64],
    geom_xpos: wp.array2d[wp.vec3],
    geom_xmat: wp.array2d[wp.mat33],
    mesh_dataid: int,
    geomid: int,
    pnt: wp.array[wp.vec3],
    vec: wp.array[wp.vec3],
    max_t: wp.array[float],
    cull_backfaces: bool,
    dist_out: wp.array[float],
    normal_out: wp.array[wp.vec3],
    u_out: wp.array[float],
    v_out: wp.array[float],
    face_out: wp.array[int],
):
    tid = wp.tid()
    d, n, u, v, f, _mesh_id = ray_mesh_with_bvh(
        mesh_bvh_id, mesh_dataid, geom_xpos[0, geomid], geom_xmat[0, geomid],
        pnt[tid], vec[tid], max_t[tid], cull_backfaces,
    )
    dist_out[tid] = d
    normal_out[tid] = n
    u_out[tid] = u
    v_out[tid] = v
    face_out[tid] = f


def bits(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def ints(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.int32).reshape(-1).tolist()


def cast(mesh_bvh_id, xpos, xmat, mesh_dataid: int, geomid: int, pnt, vec, max_t, cull: bool) -> dict:
    nray = pnt.shape[0]
    # Pad the per-geom arrays up to `geomid` so the kernel reads [0, geomid].
    width = geomid + 1
    xpos_values = np.zeros((1, width, 3), dtype=np.float32)
    xpos_values[0, geomid] = np.asarray(xpos, dtype=np.float32)
    xmat_values = np.zeros((1, width, 3, 3), dtype=np.float32)
    xmat_values[0, geomid] = np.asarray(xmat, dtype=np.float32).reshape(3, 3)
    xpos_w = wp.array(xpos_values, dtype=wp.vec3)
    xmat_w = wp.array(xmat_values, dtype=wp.mat33)
    pnt_w = wp.array(np.asarray(pnt, dtype=np.float32), dtype=wp.vec3)
    vec_w = wp.array(np.asarray(vec, dtype=np.float32), dtype=wp.vec3)
    max_w = wp.array(np.asarray(max_t, dtype=np.float32), dtype=float)
    outs = {name: wp.empty(nray, dtype=kind) for name, kind in
            (("dist", float), ("normal", wp.vec3), ("u", float), ("v", float), ("face", int))}
    wp.launch(
        _mesh_query_kernel,
        dim=nray,
        inputs=[mesh_bvh_id, xpos_w, xmat_w, mesh_dataid, geomid, pnt_w, vec_w, max_w, cull],
        outputs=[outs["dist"], outs["normal"], outs["u"], outs["v"], outs["face"]],
    )
    wp.synchronize()
    distance = np.asarray(outs["dist"].numpy(), dtype=np.float32)
    return {
        "hit": [1 if value >= 0.0 else 0 for value in distance],
        "distance_bits": bits(distance),
        "normal_bits": bits(np.asarray(outs["normal"].numpy(), dtype=np.float32)),
        "u_bits": bits(np.asarray(outs["u"].numpy(), dtype=np.float32)),
        "v_bits": bits(np.asarray(outs["v"].numpy(), dtype=np.float32)),
        "face": ints(np.asarray(outs["face"].numpy(), dtype=np.int32)),
    }


def normalize3(vectors: np.ndarray) -> np.ndarray:
    lengths = np.sqrt(np.sum(np.asarray(vectors, dtype=np.float32) ** 2, axis=-1, keepdims=True))
    return np.asarray(np.asarray(vectors, dtype=np.float32) / lengths, dtype=np.float32)


def world_to_local(origins, directions, xpos, xmat):
    """Mirror upstream `_ray_map`: lpnt = R^T (p - pos), lvec = R^T v (f32 ops)."""
    rotation = np.asarray(xmat, dtype=np.float32).reshape(3, 3)
    difference = np.asarray(np.asarray(origins, np.float32) - np.asarray(xpos, np.float32), np.float32)
    lpnt = np.asarray((rotation.T @ difference.T).T, dtype=np.float32)
    lvec = np.asarray((rotation.T @ np.asarray(directions, np.float32).T).T, dtype=np.float32)
    return lpnt, lvec


def design_mesh_rays(vertices, indices, xpos, xmat):
    """Deterministic mesh rays: oblique front/back faces, near-edge grazing, misses, axis rays.

    Directions are deliberately generic (no ray is aligned with a face normal, a face
    plane, or a mesh edge) so the device Woop computation never lands on the exact-zero
    determinant that triggers upstream's f64 fallback branch; the capture still verifies
    this statically with `s3_mesh_replay`.
    """
    rotation = np.asarray(xmat, dtype=np.float32).reshape(3, 3)
    world = np.asarray(rotation @ np.asarray(vertices, dtype=np.float32).T, dtype=np.float32).T
    world = np.asarray(world + np.asarray(xpos, dtype=np.float32), dtype=np.float32)
    face_count = indices.shape[0] // 3
    centers = [
        np.mean(np.asarray([world[int(indices[3 * face + corner])] for corner in range(3)]), axis=0)
        for face in range(face_count)
    ]
    centroid = np.mean(np.asarray(centers, dtype=np.float32), axis=0)
    extent = np.float32(max(np.linalg.norm(world - centroid, axis=1)))
    origins, directions, tags = [], [], []
    for face in range(face_count):
        v0, v1, v2 = (world[int(indices[3 * face + corner])] for corner in range(3))
        center = np.asarray((v0 + v1 + v2) / np.float32(3.0), dtype=np.float32)
        normal = normalize3(np.cross(v1 - v0, v2 - v0))
        tangent1 = normalize3(v1 - v0)
        tangent2 = normalize3(np.cross(normal, tangent1))
        skew = np.asarray(
            tangent1 * extent * np.float32(0.35) + tangent2 * extent * np.float32(0.22), dtype=np.float32
        )
        front = np.asarray(center + normal * extent * np.float32(2.0) + skew, dtype=np.float32)
        back = np.asarray(center - normal * extent * np.float32(2.0) + skew, dtype=np.float32)
        origins += [front, back]
        directions += [normalize3(center - front), normalize3(center - back)]
        tags += ["front", "back"]
        # Near-edge rays: aim just beside the v0-v1 edge midpoint (graze, not exact hit).
        edge_mid = np.asarray((v0 + v1) * np.float32(0.5), dtype=np.float32)
        perpendicular = normalize3(np.cross(normal, v1 - v0))
        for factor in (0.004, -0.003, 0.012):
            target = np.asarray(edge_mid + perpendicular * extent * np.float32(factor), dtype=np.float32)
            origin = np.asarray(front + tangent2 * extent * np.float32(0.3), dtype=np.float32)
            origins.append(origin)
            directions.append(normalize3(target - origin))
            tags.append("graze")
        # Misses: away from the face and past its side.
        origins.append(front)
        directions.append(normalize3(
            np.asarray(front + normal * extent + tangent1 * extent * np.float32(0.7), dtype=np.float32) - front
        ))
        tags.append("miss")
        origins.append(np.asarray(
            center + tangent1 * extent * np.float32(4.0) + tangent2 * extent * np.float32(3.0), dtype=np.float32
        ))
        directions.append(normalize3(
            np.asarray(center + normal * extent * np.float32(2.0) - origins[-1], dtype=np.float32)
        ))
        tags.append("miss")
        # World-axis rays (zero direction components) through slightly off-centre points.
        for axis in range(3):
            for sign in (1.0, -1.0):
                direction = np.zeros(3, dtype=np.float32)
                direction[axis] = np.float32(sign)
                side = np.zeros(3, dtype=np.float32)
                side[(axis + 1) % 3] = extent * np.float32(0.12)
                origins.append(np.asarray(
                    center - direction * extent * np.float32(1.9) + side + skew * np.float32(0.2),
                    dtype=np.float32,
                ))
                directions.append(direction)
                tags.append("axis")
    return np.asarray(origins, dtype=np.float32), np.asarray(directions, dtype=np.float32), tags


def degenerate_rays():
    vertices = DEGENERATE_VERTICES
    origins, directions = [], []
    for tri in ((0, 1, 3), (0, 2, 4), (3, 4, 0)):
        center = np.asarray((vertices[tri[0]] + vertices[tri[1]] + vertices[tri[2]]) / np.float32(3.0))
        for axis, sign in ((0, 1.0), (0, -1.0), (1, 1.0), (2, 1.0)):
            direction = np.zeros(3, dtype=np.float32)
            direction[axis] = np.float32(sign)
            origins.append(np.asarray(center - direction * np.float32(3.0), dtype=np.float32))
            directions.append(direction)
    # Four deterministic extras (indices 12..15): axis and diagonal rays. They can cross
    # an edge/vertex exactly, but only where the competing candidate is degenerate,
    # coplanar or culled, so each ray keeps a single valid nearest candidate. The set
    # therefore avoids the *ambiguous ties that remain after culling*, not every exact
    # crossing; the evidence records the 10 exact edge/vertex crossings and that they
    # leave the decision unique.
    for origin, direction in (
        ([0.5, -2.0, 0.0], [0.0, 1.0, 0.0]), ([3.0, 0.0, 0.0], [-1.0, 0.0, 0.0]),
        ([3.0, 3.0, 5.0], [0.0, 0.0, -1.0]), ([-1.4, -1.1, -0.9], [1.0, 1.0, 1.0]),
    ):
        origins.append(np.asarray(origin, dtype=np.float32))
        directions.append(np.asarray(direction, dtype=np.float32))
    # Coverage-fix ray (index 16, task 2026-10-25-render-bvh-s3-coverage-fix): valid face
    # 1 = (0,1,3) lies in z=0 and needs a -z front hit, while the 12 centre rays only use
    # +-x, +y and +z, so face 1 used to be the only valid face never hit. Aim inside face
    # 1 at x < y: no face-2 triangle is above the point and face 3 (back-facing from
    # above) is culled first, so the nearest valid hit is face 1, unambiguously.
    origins.append(np.asarray([0.25, 0.5, 3.0], dtype=np.float32))
    directions.append(np.asarray([0.0, 0.0, -1.0], dtype=np.float32))
    return np.asarray(origins, dtype=np.float32), np.asarray(directions, dtype=np.float32)


def degenerate_coverage(degenerate: dict) -> dict:
    """Valid faces are the non-zero-area triangles; the coverage fix requires all of them.

    The 2026-10-25 fix added the -z ray that reaches valid face 1 = (0,1,3); this audit
    turns the former coverage gap into a captured, machine-checked property.
    """
    face_count = DEGENERATE_INDICES.shape[0] // 3
    valid_faces = []
    for face in range(face_count):
        v0, v1, v2 = (DEGENERATE_VERTICES[DEGENERATE_INDICES[3 * face + corner]] for corner in range(3))
        if np.linalg.norm(np.cross(v1 - v0, v2 - v0)) > 0.0:
            valid_faces.append(face)
    faces = np.asarray(degenerate["face"], dtype=np.int32)
    hit = np.asarray(degenerate["hit"], dtype=bool)
    hit_faces = sorted({int(face) for face in faces[hit]})
    return {
        "ray_count": int(hit.shape[0]),
        "hit_count": int(hit.sum()),
        "valid_faces": valid_faces,
        "hit_faces": hit_faces,
        "uncovered_valid_faces": [face for face in valid_faces if face not in hit_faces],
        "invalid_face_hits": sorted({int(face) for face in faces[hit] if face not in valid_faces}),
    }


def checked_degenerate_coverage(degenerate: dict) -> dict:
    """`degenerate_coverage` plus the coverage-fix invariant: every valid face is hit."""
    coverage = degenerate_coverage(degenerate)
    if coverage["uncovered_valid_faces"] or coverage["invalid_face_hits"]:
        raise SystemExit(f"S3 mesh reference capture mismatch: degenerate coverage {coverage}")
    return coverage
