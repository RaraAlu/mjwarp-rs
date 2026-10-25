//! 上游 `collision_sdf.py` 的原语 SDF 与梯度（等价表达式，f32）。
//! 仅被动几何查询：无状态、无分配。

/// 原语尺寸向量（`geom_size` 前三项，与上游 `attr_vec3` 等价）。
pub type Attr = [f32; 3];

/// 逐分量取绝对值。
fn abs3(a: [f32; 3]) -> [f32; 3] {
    [a[0].abs(), a[1].abs(), a[2].abs()]
}

/// 逐分量除法（上游 `wp.cw_div`）。
fn cw_div3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] / b[0], a[1] / b[1], a[2] / b[2]]
}

/// 逐分量乘法（上游 `wp.cw_mul`）。
fn cw_mul3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// 逐分量最大值（上游 `wp.max(vec3)`）。
fn max3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
}

/// 点积（f32，源码顺序累加）。
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// 向量长度。
fn length3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

/// 向量归一化（零长度返回零向量，与上游 `wp.normalize` 的零向量行为一致）。
pub fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let norm = length3(a);
    if norm == 0.0 {
        [0.0; 3]
    } else {
        [a[0] / norm, a[1] / norm, a[2] / norm]
    }
}

/// 原语球 SDF。
pub fn sphere(p: [f32; 3], size: Attr) -> f32 {
    length3(p) - size[0]
}

/// 原语盒 SDF（含内部 `radial_field` 分支，与上游逐式一致）。
pub fn box_sdf(p: [f32; 3], size: Attr) -> f32 {
    let a = [
        abs3(p)[0] - size[0],
        abs3(p)[1] - size[1],
        abs3(p)[2] - size[2],
    ];
    if a[0] >= 0.0 || a[1] >= 0.0 || a[2] >= 0.0 {
        let b = max3(a, [0.0; 3]);
        return length3(b) + a[0].max(a[1]).max(a[2]).min(0.0);
    }
    let b = radial_field(a, p, size);
    let t = cw_div3([-a[0], -a[1], -a[2]], abs3(b));
    -t[0].min(t[1]).min(t[2]) * length3(b)
}

/// 原语椭球 SDF。
pub fn ellipsoid(p: [f32; 3], size: Attr) -> f32 {
    let scaled = [p[0] / size[0], p[1] / size[1], p[2] / size[2]];
    let k0 = length3(scaled);
    let k1 = length3([
        p[0] / (size[0] * size[0]),
        p[1] / (size[1] * size[1]),
        p[2] / (size[2] * size[2]),
    ]);
    let denom = if k1 != 0.0 { k1 } else { 1.0e-12 };
    k0 * (k0 - 1.0) / denom
}

/// 原语胶囊 SDF。
pub fn capsule(p: [f32; 3], size: Attr) -> f32 {
    let (r, h) = (size[0], size[1]);
    let pz = p[2].clamp(-h, h);
    length3([p[0], p[1], p[2] - pz]) - r
}

/// 原语圆柱 SDF。
pub fn cylinder(p: [f32; 3], size: Attr) -> f32 {
    let (r, h) = (size[0], size[1]);
    let dx = length3([p[0], p[1], 0.0]) - r;
    let dy = p[2].abs() - h;
    dx.max(dy).min(0.0) + length3([dx.max(0.0), dy.max(0.0), 0.0])
}

/// 上游 `wp.sign` 语义（pinned warp `include/warp-headers/builtin.h:765`：
/// `sign(x) = x < 0 ? -1 : 1`）：负数 → `-1`；`-0.0`/`0.0`/`NaN` → `+1`。
///
/// 不用 `f32::signum()`：Rust 对 `-0.0` 返回 `-1.0`、对 `NaN` 返回 `NaN`，与上游不符；
/// 宿主 `gjk/math.rs::sign3` 已是该语义。`sdf/descent.rs::compute_grad` 复用本函数，
/// 保证全链只有一份 sign 实现。
pub(crate) fn upstream_sign(x: f32) -> f32 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

/// 内部径向场：`normalize(-size/a)` 后按点符号取向外方向。
pub fn radial_field(a: [f32; 3], x: [f32; 3], size: Attr) -> [f32; 3] {
    let field = normalize3(cw_div3([-size[0], -size[1], -size[2]], a));
    [
        field[0] * upstream_sign(x[0]),
        field[1] * upstream_sign(x[1]),
        field[2] * upstream_sign(x[2]),
    ]
}

/// 原语球梯度。
pub fn grad_sphere(p: [f32; 3]) -> [f32; 3] {
    let c = length3(p);
    if c > 1.0e-9 {
        [p[0] / c, p[1] / c, p[2] / c]
    } else {
        [0.0; 3]
    }
}

/// 原语盒梯度。
pub fn grad_box(p: [f32; 3], size: Attr) -> [f32; 3] {
    let a = [
        abs3(p)[0] - size[0],
        abs3(p)[1] - size[1],
        abs3(p)[2] - size[2],
    ];
    if a[0].max(a[1]).max(a[2]) < 0.0 {
        return radial_field(a, p, size);
    }
    let b = max3(a, [0.0; 3]);
    let c = length3(b);
    let mut g = cw_mul3(cw_div3(b, [c, c, c]), cw_div3(p, abs3(p)));
    if a[0] <= 0.0 {
        g[0] = 0.0;
    }
    if a[1] <= 0.0 {
        g[1] = 0.0;
    }
    if a[2] <= 0.0 {
        g[2] = 0.0;
    }
    g
}

/// 原语椭球梯度。
pub fn grad_ellipsoid(p: [f32; 3], size: Attr) -> [f32; 3] {
    let a = [p[0] / size[0], p[1] / size[1], p[2] / size[2]];
    let b = [a[0] / size[0], a[1] / size[1], a[2] / size[2]];
    let k0 = length3(a);
    let k1 = length3(b);
    let inv_k0 = 1.0 / k0;
    let inv_k1 = 1.0 / k1;
    let gk0 = [b[0] * inv_k0, b[1] * inv_k0, b[2] * inv_k0];
    let gk1 = [
        b[0] * inv_k1 / (size[0] * size[0]),
        b[1] * inv_k1 / (size[1] * size[1]),
        b[2] * inv_k1 / (size[2] * size[2]),
    ];
    let df_dk0 = (2.0 * k0 - 1.0) * inv_k1;
    let df_dk1 = k0 * (k0 - 1.0) * inv_k1 * inv_k1;
    let raw = [
        gk0[0] * df_dk0 - gk1[0] * df_dk1,
        gk0[1] * df_dk0 - gk1[1] * df_dk1,
        gk0[2] * df_dk0 - gk1[2] * df_dk1,
    ];
    normalize3(raw)
}

/// 原语胶囊梯度。
pub fn grad_capsule(p: [f32; 3], size: Attr) -> [f32; 3] {
    let h = size[1];
    let pz = p[2].clamp(-h, h);
    let diff = [p[0], p[1], p[2] - pz];
    let c = length3(diff);
    if c > super::MINVAL {
        [diff[0] / c, diff[1] / c, diff[2] / c]
    } else {
        [0.0; 3]
    }
}

/// 原语圆柱梯度（上游 `grad_cylinder`，含内部径向/轴向分支）。
pub fn grad_cylinder(p: [f32; 3], size: Attr) -> [f32; 3] {
    let (r, h) = (size[0], size[1]);
    let radial_dist = length3([p[0], p[1], 0.0]);
    let u = if radial_dist > super::MINVAL {
        [p[0] / radial_dist, p[1] / radial_dist, 0.0]
    } else {
        [0.0; 3]
    };
    let w = [0.0, 0.0, upstream_sign(p[2])];
    let dx = radial_dist - r;
    let dy = p[2].abs() - h;
    if dx > 0.0 && dy > 0.0 {
        let len_v = length3([dx, dy, 0.0]);
        if len_v > super::MINVAL {
            [
                u[0] * (dx / len_v) + w[0] * (dy / len_v),
                u[1] * (dx / len_v) + w[1] * (dy / len_v),
                u[2] * (dx / len_v) + w[2] * (dy / len_v),
            ]
        } else {
            [0.0; 3]
        }
    } else if dx > 0.0 {
        u
    } else if dy > 0.0 {
        w
    } else if dx > dy {
        u
    } else {
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 上游 `wp.sign` 全语义：负数 -1；`-0.0`/`0.0`/`NaN`/正数 → +1。
    #[test]
    fn upstream_sign_matches_warp_sign_semantics() {
        assert_eq!(upstream_sign(-1.5), -1.0);
        assert_eq!(upstream_sign(f32::NEG_INFINITY), -1.0);
        assert_eq!(upstream_sign(0.0), 1.0);
        assert_eq!(upstream_sign(-0.0), 1.0);
        assert_eq!(upstream_sign(f32::NAN), 1.0);
        assert_eq!(upstream_sign(2.5), 1.0);
        assert_eq!(upstream_sign(f32::INFINITY), 1.0);
    }

    /// 反向判别力：修正前的 `f32::signum()` 在 `-0.0`/`NaN` 上与本片 `upstream_sign`
    /// 相反/不同；若把产品实现退回 `signum()`，上面与下面的产品路径测试都会失败。
    #[test]
    fn pre_fix_signum_behaviour_differs_from_upstream() {
        assert_eq!((-0.0_f32).signum(), -1.0); // Rust: -1；上游: +1
        assert!((f32::NAN).signum().is_nan()); // Rust: NaN；上游: +1
        assert_eq!((0.0_f32).signum(), 1.0); // 两者一致
        assert_eq!((0.0_f32).signum(), upstream_sign(0.0));
        assert_ne!((-0.0_f32).signum(), upstream_sign(-0.0));
        assert_ne!((f32::NAN).signum(), upstream_sign(f32::NAN));
    }

    /// 产品路径反向判别力：`radial_field` 对 `-0.0`/`NaN` 取上游 +1；
    /// 修正前 `signum()` 会让 `-0.0` 分量变号、`NaN` 分量变 NaN（本测试会失败）。
    #[test]
    fn radial_field_uses_upstream_sign_for_negative_zero_and_nan() {
        let a = [-0.5, -0.5, -0.5];
        let size = [1.0, 1.0, 1.0];
        let positive = radial_field(a, [1.0, 1.0, 1.0], size);
        let zero = radial_field(a, [-0.0, 0.0, -0.0], size);
        assert_eq!(zero, positive, "±0.0 必须都取 +1");
        let nan = radial_field(a, [f32::NAN, 0.0, -0.0], size);
        assert_eq!(nan, positive, "NaN 必须取 +1，而不是传播 NaN");
        let negative = radial_field(a, [-1.0, -2.0, -3.0], size);
        assert_eq!(negative, [-positive[0], -positive[1], -positive[2]]);
    }

    /// 产品路径反向判别力：`grad_cylinder` 的轴向 `w` 对 `-0.0` 取 +1；
    /// 修正前 `signum()` 返回 -1（本测试会失败）。
    #[test]
    fn grad_cylinder_uses_upstream_sign_for_negative_zero() {
        // r > h 且点位于轴心：dx=-r、dy=-h，走 `else → w` 分支。
        assert_eq!(
            grad_cylinder([0.0, 0.0, -0.0], [2.0, 1.0, 1.0]),
            [0.0, 0.0, 1.0]
        );
        assert_eq!(
            grad_cylinder([0.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
            [0.0, 0.0, 1.0]
        );
    }
}
