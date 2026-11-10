"""Check product render_flex.cu flex_quat_z2vec/flex_quat_to_mat against upstream CPU probe results.

Product formulas transcribed from src/render/kernel/cuda/render_flex.cu. Compare the resulting
rotation matrices with the values the pinned upstream produced on CUDA-independent CPU warp
(see probe_quat.log).
"""
import math

import numpy as np

UPSTREAM_MATS = {
    (0.0, 0.0, -1.0): [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    (0.0, 0.0, 1.0): [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    (0.0, 0.5, 0.0): [[1.0, -0.0, 0.0], [0.0, 0.0, 1.0], [-0.0, -1.0, 0.0]],
    (0.5, 0.0, 0.0): [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
    (0.0, 0.0, 0.0): [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
    (0.6000000238418579, 0.800000011920929, 0.0): [[0.6399999856948853, -0.47999998927116394, 0.6000000238418579],
                                                    [-0.47999998927116394, 0.36000001430511475, 0.800000011920929],
                                                    [-0.6000000238418579, -0.800000011920929, -0.0]],
    (-0.6000000238418579, -0.800000011920929, 0.0): [[0.6399999856948853, -0.47999998927116394, -0.6000000238418579],
                                                     [-0.47999998927116394, 0.36000001430511475, -0.800000011920929],
                                                     [0.6000000238418579, 0.800000011920929, -0.0]],
}

MINVAL = 1.0e-15


def length3(v):
    x, y, z = np.float32(v[0]), np.float32(v[1]), np.float32(v[2])
    # product: sqrtf(fmaf(z,z,fmaf(y,y,x*x))); emulate with exact f64 then f32 round (fma-equivalent here)
    s = np.float32(np.float64(z) * np.float64(z) + (np.float64(y) * np.float64(y) + np.float64(x) * np.float64(x)))
    return np.float32(math.sqrt(np.float64(s)))


def quat_z2vec(vec):
    quat = np.zeros(4, dtype=np.float32)  # product layout: [w, x, y, z] positional
    quat[3] = 1.0
    norm = length3(vec)
    if norm < MINVAL:
        return quat
    unit = vec / norm
    axis = np.array([-unit[1], unit[0], 0.0], dtype=np.float32)
    a = length3(axis)
    if a < MINVAL:
        if unit[2] < 0.0:
            quat[0] = 1.0
        return quat
    axis = axis / a
    angle = np.arctan2(np.float64(a), np.float64(unit[2]))
    half = np.float32(np.float64(angle) * 0.5)
    s = np.float32(math.sin(np.float64(half)))
    c = np.float32(math.cos(np.float64(half)))
    quat[0] = c
    quat[1] = axis[0] * s
    quat[2] = axis[1] * s
    quat[3] = axis[2] * s
    return quat


def quat_to_mat(quat):
    w, x, y, z = quat  # positional [w,x,y,z]
    q00, q01, q02, q03 = w * w, w * x, w * y, w * z
    q11, q12, q13 = x * x, x * y, x * z
    q22, q23, q33 = y * y, y * z, z * z
    return np.array([
        [q00 + q11 - q22 - q33, 2.0 * (q12 - q03), 2.0 * (q13 + q02)],
        [2.0 * (q12 + q03), q00 - q11 + q22 - q33, 2.0 * (q23 - q01)],
        [2.0 * (q13 - q02), 2.0 * (q23 + q01), q00 - q11 - q22 + q33],
    ], dtype=np.float32)


print("vec -> product quat -> product mat vs upstream mat")
for vec, up_mat in UPSTREAM_MATS.items():
    v = np.array(vec, dtype=np.float32)
    q = quat_z2vec(v)
    m = quat_to_mat(q)
    up = np.array(up_mat, dtype=np.float32)
    d = float(np.max(np.abs(m - up)))
    print(f"vec={np.round(v, 6).tolist()} quat={np.round(q, 6).tolist()} max|prod-up|={d:.2e} match={d < 1e-6}")
