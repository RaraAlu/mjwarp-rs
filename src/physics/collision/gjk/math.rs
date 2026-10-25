//! 上游 `warp/native/vec.h`、`mat.h` 的 f32 向量/矩阵原语（运算顺序逐句对齐）。
//!
//! 说明：上游 `wp.dot(vec3)` 展开为 `a0*b0 + a1*b1 + a2*b2`（左结合），
//! `wp.length` 为 `sqrt(a0*a0 + a1*a1 + a2*a2)`，`wp.normalize` 在范数 ≤ 0 时返回零向量
//! （`kEps == 0.0f`），`wp.sign` 对 0 取 `+1`。本模块按源码表达式顺序计算（每步独立舍入）。
//!
//! 位级说明：pinned 3.15 的 ccd kernel PTX 记录 `fma.rn.f32` 1480 处、
//! `mul.rn.f32`/`add.rn.f32` 各 0 处（NVVM 默认做 FMA 收缩），因此 GPU 侧每个
//! 多点累加只舍入部分中间积。本片尝试过「dot/length/matvec 全 fused」与
//! 「仅 cross 不 fused」两种近似，实测各 case 互有胜负（见证据 ULP 表），
//! 说明逐表达式收缩顺序未完全对齐；最终保留源码顺序实现，位级定序留后续片。

/// 上游 `wp.dot(vec3)`。
#[inline]
pub(crate) fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// 上游 `wp.cross(vec3, vec3)`。
#[inline]
pub(crate) fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// 上游 `wp.length(vec3)`。
#[inline]
pub(crate) fn length3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// 上游 `wp.normalize(vec3)`：范数不大于 0 时返回零向量。
#[inline]
pub(crate) fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let norm = length3(a);
    if norm > 0.0 {
        [a[0] / norm, a[1] / norm, a[2] / norm]
    } else {
        [0.0; 3]
    }
}

/// 上游 `wp.sign(vec3)`：`v < 0 ? -1 : 1`。
#[inline]
pub(crate) fn sign3(a: [f32; 3]) -> [f32; 3] {
    [
        if a[0] < 0.0 { -1.0 } else { 1.0 },
        if a[1] < 0.0 { -1.0 } else { 1.0 },
        if a[2] < 0.0 { -1.0 } else { 1.0 },
    ]
}

#[inline]
pub(crate) fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
pub(crate) fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// 上游 `wp.cw_mul(vec3, vec3)`。
#[inline]
pub(crate) fn cw_mul3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// 上游 `scalar * vec3`（分量乘）。
#[inline]
pub(crate) fn scale3(a: [f32; 3], factor: f32) -> [f32; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

#[inline]
pub(crate) fn neg3(a: [f32; 3]) -> [f32; 3] {
    [-a[0], -a[1], -a[2]]
}

/// 行内三点乘累加（源码顺序：两次独立乘 + 两次加）。
#[inline]
fn fused3(a0: f32, a1: f32, a2: f32, vector: [f32; 3]) -> f32 {
    a0 * vector[0] + a1 * vector[1] + a2 * vector[2]
}

/// 上游 `mat33 @ vec3`（行优先矩阵，逐行 `m[i][0]*v0 + m[i][1]*v1 + m[i][2]*v2`）。
#[inline]
pub(crate) fn mul_mat3_vec(matrix: &[f32; 9], vector: [f32; 3]) -> [f32; 3] {
    [
        fused3(matrix[0], matrix[1], matrix[2], vector),
        fused3(matrix[3], matrix[4], matrix[5], vector),
        fused3(matrix[6], matrix[7], matrix[8], vector),
    ]
}

/// 上游 `transpose(mat33) @ vec3`（列点积）。
#[inline]
pub(crate) fn transpose_mul_mat3_vec(matrix: &[f32; 9], vector: [f32; 3]) -> [f32; 3] {
    [
        fused3(matrix[0], matrix[3], matrix[6], vector),
        fused3(matrix[1], matrix[4], matrix[7], vector),
        fused3(matrix[2], matrix[5], matrix[8], vector),
    ]
}

/// 上游 `mat33 @ mat33`。
#[inline]
pub(crate) fn mul_mat3_mat3(left: &[f32; 9], right: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            out[row * 3 + column] = left[row * 3] * right[column]
                + left[row * 3 + 1] * right[3 + column]
                + left[row * 3 + 2] * right[6 + column];
        }
    }
    out
}

/// 3×3 转置。
#[inline]
pub(crate) fn transpose3(matrix: &[f32; 9]) -> [f32; 9] {
    [
        matrix[0], matrix[3], matrix[6], matrix[1], matrix[4], matrix[7], matrix[2], matrix[5],
        matrix[8],
    ]
}

/// 上游 `math.make_frame`：以法线为第一行的接触坐标系（行优先）。
pub(crate) fn make_frame(normal: [f32; 3]) -> [f32; 9] {
    let a = normalize3(normal);
    let selector = if -0.5 < a[1] && a[1] < 0.5 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [0.0_f32, 0.0, 1.0]
    };
    let mut b = sub3(selector, scale3(a, dot3(a, selector)));
    b = normalize3(b);
    if length3(a) == 0.0 {
        b = [0.0; 3];
    }
    let c = cross3(a, b);
    [a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]
}
