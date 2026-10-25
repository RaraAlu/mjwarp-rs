//! 上游 `collision_gjk` 的距离子算法：`_S1D/_S2D/_S3D`、投影与单纯形辅助。

use super::math::{cross3, dot3, length3, sub3};
use super::{FLOAT_MAX, MINVAL};

/// 上游 `_det3`：`dot(v1, cross(v2, v3))`。
#[inline]
pub(crate) fn det3(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> f32 {
    dot3(v1, cross3(v2, v3))
}

/// 上游 `_same_sign`：同正返回 1、同负返回 -1、否则 0。
#[inline]
pub(crate) fn same_sign(a: f32, b: f32) -> i32 {
    if a > 0.0 && b > 0.0 {
        return 1;
    }
    if a < 0.0 && b < 0.0 {
        return -1;
    }
    0
}

/// 上游 `_project_origin_line`。
pub(crate) fn project_origin_line(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let diff = sub3(v2, v1);
    let scl = -(dot3(v2, diff) / dot3(diff, diff));
    [
        v2[0] + scl * diff[0],
        v2[1] + scl * diff[1],
        v2[2] + scl * diff[2],
    ]
}

/// 上游 `_project_origin_plane`：返回 (投影点, 是否退化)。
pub(crate) fn project_origin_plane(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> ([f32; 3], bool) {
    let zero = [0.0_f32; 3];
    let diff21 = sub3(v2, v1);
    let diff31 = sub3(v3, v1);
    let diff32 = sub3(v3, v2);

    // n = (v1 - v2) x (v3 - v2)
    let n = cross3(diff32, diff21);
    let nv = dot3(n, v2);
    let nn = dot3(n, n);
    if nn == 0.0 {
        return (zero, true);
    }
    if nv != 0.0 && nn > MINVAL {
        return (scale_plane(n, nv, nn), false);
    }

    // n = (v2 - v1) x (v3 - v1)
    let n = cross3(diff21, diff31);
    let nv = dot3(n, v1);
    let nn = dot3(n, n);
    if nn == 0.0 {
        return (zero, true);
    }
    if nv != 0.0 && nn > MINVAL {
        return (scale_plane(n, nv, nn), false);
    }

    // n = (v1 - v3) x (v2 - v3)
    let n = cross3(diff31, diff32);
    let nv = dot3(n, v3);
    let nn = dot3(n, n);
    (scale_plane(n, nv, nn), false)
}

#[inline]
fn scale_plane(n: [f32; 3], nv: f32, nn: f32) -> [f32; 3] {
    let scl = nv / nn;
    [scl * n[0], scl * n[1], scl * n[2]]
}

/// 上游 `_S3D`：4 点单纯形的重心坐标。
pub(crate) fn s3d(s1: [f32; 3], s2: [f32; 3], s3: [f32; 3], s4: [f32; 3]) -> [f32; 4] {
    let c41 = -det3(s2, s3, s4);
    let c42 = det3(s1, s3, s4);
    let c43 = -det3(s1, s2, s4);
    let c44 = det3(s1, s2, s3);

    let m_det = c41 + c42 + c43 + c44;

    let comp1 = same_sign(m_det, c41);
    let comp2 = same_sign(m_det, c42);
    let comp3 = same_sign(m_det, c43);
    let comp4 = same_sign(m_det, c44);

    // 四个分量同号：原点在四面体内。
    if comp1 != 0 && comp2 != 0 && comp3 != 0 && comp4 != 0 {
        return [c41 / m_det, c42 / m_det, c43 / m_det, c44 / m_det];
    }

    let mut lmbda = [0.0_f32; 4];
    let mut dmin = FLOAT_MAX;

    if comp1 == 0 {
        let sub = s2d(s2, s3, s4);
        let x = [
            sub[0] * s2[0] + sub[1] * s3[0] + sub[2] * s4[0],
            sub[0] * s2[1] + sub[1] * s3[1] + sub[2] * s4[1],
            sub[0] * s2[2] + sub[1] * s3[2] + sub[2] * s4[2],
        ];
        let d = dot3(x, x);
        lmbda[0] = 0.0;
        lmbda[1] = sub[0];
        lmbda[2] = sub[1];
        lmbda[3] = sub[2];
        dmin = d;
    }

    if comp2 == 0 {
        let sub = s2d(s1, s3, s4);
        let x = [
            sub[0] * s1[0] + sub[1] * s3[0] + sub[2] * s4[0],
            sub[0] * s1[1] + sub[1] * s3[1] + sub[2] * s4[1],
            sub[0] * s1[2] + sub[1] * s3[2] + sub[2] * s4[2],
        ];
        let d = dot3(x, x);
        if d < dmin {
            lmbda[0] = sub[0];
            lmbda[1] = 0.0;
            lmbda[2] = sub[1];
            lmbda[3] = sub[2];
            dmin = d;
        }
    }

    if comp3 == 0 {
        let sub = s2d(s1, s2, s4);
        let x = [
            sub[0] * s1[0] + sub[1] * s2[0] + sub[2] * s4[0],
            sub[0] * s1[1] + sub[1] * s2[1] + sub[2] * s4[1],
            sub[0] * s1[2] + sub[1] * s2[2] + sub[2] * s4[2],
        ];
        let d = dot3(x, x);
        if d < dmin {
            lmbda[0] = sub[0];
            lmbda[1] = sub[1];
            lmbda[2] = 0.0;
            lmbda[3] = sub[2];
            dmin = d;
        }
    }

    if comp4 == 0 {
        let sub = s2d(s1, s2, s3);
        let x = [
            sub[0] * s1[0] + sub[1] * s2[0] + sub[2] * s3[0],
            sub[0] * s1[1] + sub[1] * s2[1] + sub[2] * s3[1],
            sub[0] * s1[2] + sub[1] * s2[2] + sub[2] * s3[2],
        ];
        let d = dot3(x, x);
        if d < dmin {
            lmbda[0] = sub[0];
            lmbda[1] = sub[1];
            lmbda[2] = sub[2];
            lmbda[3] = 0.0;
        }
    }
    lmbda
}

/// 上游 `_S2D`：3 点单纯形的重心坐标。
pub(crate) fn s2d(s1: [f32; 3], s2: [f32; 3], s3: [f32; 3]) -> [f32; 3] {
    let (p_o, ret) = project_origin_plane(s1, s2, s3);
    if ret {
        let value = s1d(s1, s2);
        return [value[0], value[1], 0.0];
    }

    let m_14 = s2[1] * s3[2] - s2[2] * s3[1] - s1[1] * s3[2] + s1[2] * s3[1] + s1[1] * s2[2]
        - s1[2] * s2[1];
    let m_24 = s2[0] * s3[2] - s2[2] * s3[0] - s1[0] * s3[2] + s1[2] * s3[0] + s1[0] * s2[2]
        - s1[2] * s2[0];
    let m_34 = s2[0] * s3[1] - s2[1] * s3[0] - s1[0] * s3[1] + s1[1] * s3[0] + s1[0] * s2[1]
        - s1[1] * s2[0];

    let mu1 = m_14.abs();
    let mu2 = m_24.abs();
    let mu3 = m_34.abs();

    let (m_max, x, y) = if mu1 >= mu2 && mu1 >= mu3 {
        (m_14, 1, 2)
    } else if mu2 >= mu3 {
        (m_24, 0, 2)
    } else {
        (m_34, 0, 1)
    };
    let s1_2d = [s1[x], s1[y]];
    let s2_2d = [s2[x], s2[y]];
    let s3_2d = [s3[x], s3[y]];
    let p_o_2d = [p_o[x], p_o[y]];

    let c31 = p_o_2d[0] * s2_2d[1] + p_o_2d[1] * s3_2d[0] + s2_2d[0] * s3_2d[1]
        - p_o_2d[0] * s3_2d[1]
        - p_o_2d[1] * s2_2d[0]
        - s3_2d[0] * s2_2d[1];
    let c32 = p_o_2d[0] * s3_2d[1] + p_o_2d[1] * s1_2d[0] + s3_2d[0] * s1_2d[1]
        - p_o_2d[0] * s1_2d[1]
        - p_o_2d[1] * s3_2d[0]
        - s1_2d[0] * s3_2d[1];
    let c33 = p_o_2d[0] * s1_2d[1] + p_o_2d[1] * s2_2d[0] + s1_2d[0] * s2_2d[1]
        - p_o_2d[0] * s2_2d[1]
        - p_o_2d[1] * s1_2d[0]
        - s2_2d[0] * s1_2d[1];

    let comp1 = same_sign(m_max, c31);
    let comp2 = same_sign(m_max, c32);
    let comp3 = same_sign(m_max, c33);

    // 同号：p_o 在三角形内。
    if comp1 != 0 && comp2 != 0 && comp3 != 0 {
        return [c31 / m_max, c32 / m_max, c33 / m_max];
    }

    let mut dmin = FLOAT_MAX;
    let mut lmbda = [0.0_f32; 3];

    if comp1 == 0 {
        let sub = s1d(s2, s3);
        let x = [
            sub[0] * s2[0] + sub[1] * s3[0],
            sub[0] * s2[1] + sub[1] * s3[1],
            sub[0] * s2[2] + sub[1] * s3[2],
        ];
        let d = dot3(x, x);
        lmbda[0] = 0.0;
        lmbda[1] = sub[0];
        lmbda[2] = sub[1];
        dmin = d;
    }

    if comp2 == 0 {
        let sub = s1d(s1, s3);
        let x = [
            sub[0] * s1[0] + sub[1] * s3[0],
            sub[0] * s1[1] + sub[1] * s3[1],
            sub[0] * s1[2] + sub[1] * s3[2],
        ];
        let d = dot3(x, x);
        if d < dmin {
            lmbda[0] = sub[0];
            lmbda[1] = 0.0;
            lmbda[2] = sub[1];
            dmin = d;
        }
    }

    if comp3 == 0 {
        let sub = s1d(s1, s2);
        let x = [
            sub[0] * s1[0] + sub[1] * s2[0],
            sub[0] * s1[1] + sub[1] * s2[1],
            sub[0] * s1[2] + sub[1] * s2[2],
        ];
        let d = dot3(x, x);
        if d < dmin {
            lmbda[0] = sub[0];
            lmbda[1] = sub[1];
            lmbda[2] = 0.0;
        }
    }
    let _ = dmin;
    lmbda
}

/// 上游 `_S1D`：2 点单纯形的重心坐标。
pub(crate) fn s1d(s1: [f32; 3], s2: [f32; 3]) -> [f32; 2] {
    let p_o = project_origin_line(s1, s2);

    // 找投影“影长”最大的轴。
    let mut mu_max = s1[0] - s2[0];
    let mut index = 0;
    let mu = s1[1] - s2[1];
    if mu.abs() >= mu_max.abs() {
        mu_max = mu;
        index = 1;
    }
    let mu = s1[2] - s2[2];
    if mu.abs() >= mu_max.abs() {
        mu_max = mu;
        index = 2;
    }

    let c1 = p_o[index] - s2[index];
    let c2 = s1[index] - p_o[index];

    if same_sign(mu_max, c1) != 0 && same_sign(mu_max, c2) != 0 {
        return [c1 / mu_max, c2 / mu_max];
    }
    [0.0, 1.0]
}

/// 上游 `_linear_combine`。
pub(crate) fn linear_combine(n: usize, scl: [f32; 4], matrix: &[[f32; 3]; 4]) -> [f32; 3] {
    match n {
        1 => [
            scl[0] * matrix[0][0],
            scl[0] * matrix[0][1],
            scl[0] * matrix[0][2],
        ],
        2 => [
            scl[0] * matrix[0][0] + scl[1] * matrix[1][0],
            scl[0] * matrix[0][1] + scl[1] * matrix[1][1],
            scl[0] * matrix[0][2] + scl[1] * matrix[1][2],
        ],
        3 => [
            scl[0] * matrix[0][0] + scl[1] * matrix[1][0] + scl[2] * matrix[2][0],
            scl[0] * matrix[0][1] + scl[1] * matrix[1][1] + scl[2] * matrix[2][1],
            scl[0] * matrix[0][2] + scl[1] * matrix[1][2] + scl[2] * matrix[2][2],
        ],
        _ => [
            scl[0] * matrix[0][0]
                + scl[1] * matrix[1][0]
                + scl[2] * matrix[2][0]
                + scl[3] * matrix[3][0],
            scl[0] * matrix[0][1]
                + scl[1] * matrix[1][1]
                + scl[2] * matrix[2][1]
                + scl[3] * matrix[3][1],
            scl[0] * matrix[0][2]
                + scl[1] * matrix[1][2]
                + scl[2] * matrix[2][2]
                + scl[3] * matrix[3][2],
        ],
    }
}

/// 上游 `_subdistance`。
pub(crate) fn subdistance(n: usize, simplex: &[[f32; 3]; 4]) -> [f32; 4] {
    if n == 4 {
        return s3d(simplex[0], simplex[1], simplex[2], simplex[3]);
    }
    if n == 3 {
        let lmbda3 = s2d(simplex[0], simplex[1], simplex[2]);
        return [lmbda3[0], lmbda3[1], lmbda3[2], 0.0];
    }
    if n == 2 {
        let lmbda2 = s1d(simplex[0], simplex[1]);
        return [lmbda2[0], lmbda2[1], 0.0, 0.0];
    }
    [1.0, 0.0, 0.0, 0.0]
}

/// 上游 `_tri_affine_coord`。
pub(crate) fn tri_affine_coord(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let m_14 = v2[1] * v3[2] - v2[2] * v3[1] - v1[1] * v3[2] + v1[2] * v3[1] + v1[1] * v2[2]
        - v1[2] * v2[1];
    let m_24 = v2[0] * v3[2] - v2[2] * v3[0] - v1[0] * v3[2] + v1[2] * v3[0] + v1[0] * v2[2]
        - v1[2] * v2[0];
    let m_34 = v2[0] * v3[1] - v2[1] * v3[0] - v1[0] * v3[1] + v1[1] * v3[0] + v1[0] * v2[1]
        - v1[1] * v2[0];

    let mu1 = m_14.abs();
    let mu2 = m_24.abs();
    let mu3 = m_34.abs();

    let (m_max, x, y) = if mu1 >= mu2 && mu1 >= mu3 {
        (m_14, 1, 2)
    } else if mu2 >= mu3 {
        (m_24, 0, 2)
    } else {
        (m_34, 0, 1)
    };

    let c31 =
        p[x] * v2[y] + p[y] * v3[x] + v2[x] * v3[y] - p[x] * v3[y] - p[y] * v2[x] - v3[x] * v2[y];
    let c32 =
        p[x] * v3[y] + p[y] * v1[x] + v3[x] * v1[y] - p[x] * v1[y] - p[y] * v3[x] - v1[x] * v3[y];
    let c33 =
        p[x] * v1[y] + p[y] * v2[x] + v1[x] * v2[y] - p[x] * v2[y] - p[y] * v1[x] - v2[x] * v1[y];

    [c31 / m_max, c32 / m_max, c33 / m_max]
}

/// 上游 `_tri_point_intersect`。
pub(crate) fn tri_point_intersect(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3], p: [f32; 3]) -> bool {
    let coordinates = tri_affine_coord(v1, v2, v3, p);
    let (l1, l2, l3) = (coordinates[0], coordinates[1], coordinates[2]);
    if l1 < 0.0 || l2 < 0.0 || l3 < 0.0 {
        return false;
    }
    let pr = [
        v1[0] * l1 + v2[0] * l2 + v3[0] * l3,
        v1[1] * l1 + v2[1] * l2 + v3[1] * l3,
        v1[2] * l1 + v2[2] * l2 + v3[2] * l3,
    ];
    length3(sub3(pr, p)) < MINVAL
}

/// 上游 `_same_side`。
pub(crate) fn same_side(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) -> bool {
    let n = cross3(sub3(p1, p0), sub3(p2, p0));
    let dot1 = dot3(n, sub3(p3, p0));
    let dot2 = dot3(n, [-p0[0], -p0[1], -p0[2]]);
    (dot1 > 0.0 && dot2 > 0.0) || (dot1 < 0.0 && dot2 < 0.0)
}

/// 上游 `_test_tetra`。
pub(crate) fn test_tetra(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) -> bool {
    same_side(p0, p1, p2, p3)
        && same_side(p1, p2, p3, p0)
        && same_side(p2, p3, p0, p1)
        && same_side(p3, p0, p1, p2)
}
