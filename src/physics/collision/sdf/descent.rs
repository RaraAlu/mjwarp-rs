//! 上游 `collision_sdf.py` 的优化层：`sdf()`/`sdf_grad()` 分发、`clearance`、
//! `compute_grad`、Armijo `gradient_step` 与 `gradient_descent`。
//! 全部 f32；`sdf_iterations` 与 `sdf_initpoints` 语义由调用方从 `m.opt` 提供。

use super::super::gjk::math::{
    add3, dot3, mul_mat3_vec, normalize3, scale3, sub3, transpose_mul_mat3_vec,
};
use super::super::relmat;
use super::{
    SdfShape, SdfType,
    octree::{sample_volume_grad, sample_volume_sdf},
    primitives,
};

/// 上游 `OptimizationParams`。
#[derive(Clone, Copy, Debug)]
pub struct OptimizationParams {
    /// `transpose(rot1) * rot2`（g1 局部帧 → g2 局部帧）。
    pub rel_mat: [f32; 9],
    /// `transpose(rot1) * (pos2 - pos1)`。
    pub rel_pos: [f32; 3],
    pub attr1: [f32; 3],
    pub attr2: [f32; 3],
}

/// 未实现的插件/ray-mesh 分支返回的有限大值（上游在同类错误路径打印后返回 0）。
const UNIMPLEMENTED: f32 = f32::MAX;

/// 上游 `sdf(type, p, ...)`：几何体自身的 SDF。
pub fn sdf(shape: &SdfShape, p: [f32; 3]) -> f32 {
    match shape.geom {
        SdfType::Plane => p[2],
        SdfType::Sphere => primitives::sphere(p, shape.size),
        SdfType::Capsule => primitives::capsule(p, shape.size),
        SdfType::Cylinder => primitives::cylinder(p, shape.size),
        SdfType::Box => primitives::box_sdf(p, shape.size),
        SdfType::Ellipsoid => primitives::ellipsoid(p, shape.size),
        SdfType::Mesh | SdfType::Sdf => volume_sdf(shape, p),
    }
}

/// 上游对 geom2 强制 `GeomType.SDF` 的 `sdf()` 调用。
fn forced_sdf(shape: &SdfShape, p: [f32; 3]) -> f32 {
    volume_sdf(shape, p)
}

fn volume_sdf(shape: &SdfShape, p: [f32; 3]) -> f32 {
    match shape.volume {
        Some(volume) => sample_volume_sdf(volume, p).unwrap_or(UNIMPLEMENTED),
        None => UNIMPLEMENTED,
    }
}

/// 上游 `sdf_grad(type, p, ...)`：几何体自身的 SDF 梯度。
pub fn sdf_grad(shape: &SdfShape, p: [f32; 3]) -> [f32; 3] {
    match shape.geom {
        SdfType::Plane => [0.0, 0.0, 1.0],
        SdfType::Sphere => primitives::grad_sphere(p),
        SdfType::Capsule => primitives::grad_capsule(p, shape.size),
        SdfType::Cylinder => primitives::grad_cylinder(p, shape.size),
        SdfType::Box => primitives::grad_box(p, shape.size),
        SdfType::Ellipsoid => primitives::grad_ellipsoid(p, shape.size),
        SdfType::Mesh | SdfType::Sdf => volume_grad(shape, p),
    }
}

/// 上游对 geom2 强制 `GeomType.SDF` 的 `sdf_grad()` 调用。
fn forced_sdf_grad(shape: &SdfShape, p: [f32; 3]) -> [f32; 3] {
    volume_grad(shape, p)
}

fn volume_grad(shape: &SdfShape, p: [f32; 3]) -> [f32; 3] {
    match shape.volume {
        Some(volume) => sample_volume_grad(volume, p).unwrap_or([0.0; 3]),
        None => [0.0; 3],
    }
}

/// 上游 `clearance`：`sfd_intersection` 时取 `max`，否则取 `a + b + |max|`。
pub fn clearance(
    shape1: &SdfShape,
    shape2: &SdfShape,
    p1: [f32; 3],
    p2: [f32; 3],
    sfd_intersection: bool,
) -> f32 {
    let sdf1 = sdf(shape1, p1);
    let sdf2 = forced_sdf(shape2, p2);
    let max_val = sdf1.max(sdf2);
    if sfd_intersection {
        max_val
    } else {
        sdf1 + sdf2 + max_val.abs()
    }
}

/// 上游 `compute_grad`：按 `sfd_intersection` 选梯度组合。
pub fn compute_grad(
    shape1: &SdfShape,
    shape2: &SdfShape,
    p1: [f32; 3],
    p2: [f32; 3],
    params: &OptimizationParams,
    sfd_intersection: bool,
) -> [f32; 3] {
    let a_ = sdf(shape1, p1);
    let b_ = forced_sdf(shape2, p2);
    let grad1 = sdf_grad(shape1, p1);
    let grad2 = forced_sdf_grad(shape2, p2);
    let grad1_transformed = transpose_mul_mat3_vec(&params.rel_mat, grad1);
    if sfd_intersection {
        return if a_ > b_ { grad1_transformed } else { grad2 };
    }
    let mut gradient = add3(grad2, grad1_transformed);
    let max_val = a_.max(b_);
    let max_grad = if a_ > b_ { grad1_transformed } else { grad2 };
    // 上游 `wp.sign(max_val)`（pinned warp `include/warp-headers/builtin.h:765`：
    // `x < 0 ? -1 : 1`）：`-0.0`/`0.0`/`NaN` → `+1`，负数 → `-1`。
    // 旧实现手写 `>0→1, <0→-1, else→0` 在 max_val==0/NaN 时与上游不一致。
    let sign = primitives::upstream_sign(max_val);
    gradient = add3(gradient, scale3(max_grad, sign));
    gradient
}

/// 上游 `gradient_step`：Armijo 回溯（`amin=1e-4`、`rho=0.5`、`c=0.1`）。
pub fn gradient_step(
    shape1: &SdfShape,
    shape2: &SdfShape,
    x0: [f32; 3],
    params: &OptimizationParams,
    niter: usize,
    sfd_intersection: bool,
) -> (f32, [f32; 3]) {
    let amin = 1.0e-4_f32;
    let rho = 0.5_f32;
    let c = 0.1_f32;
    let mut dist = 1.0e10_f32;
    let mut x = x0;
    for _ in 0..niter {
        let mut alpha = 2.0_f32;
        let x2 = x;
        let mut x1 = add3(mul_mat3_vec(&params.rel_mat, x2), params.rel_pos);
        let grad = compute_grad(shape1, shape2, x1, x2, params, sfd_intersection);
        let dist0 = clearance(shape1, shape2, x1, x, sfd_intersection);
        let grad_dot = dot3(grad, grad);
        if grad_dot < 1.0e-12 {
            return (dist0, x);
        }
        let mut wolfe = -c * alpha * grad_dot;
        loop {
            alpha *= rho;
            wolfe *= rho;
            x = sub3(x2, scale3(grad, alpha));
            x1 = add3(mul_mat3_vec(&params.rel_mat, x), params.rel_pos);
            dist = clearance(shape1, shape2, x1, x, sfd_intersection);
            if alpha <= amin || (dist - dist0) <= wolfe {
                break;
            }
        }
        if dist > dist0 {
            return (dist0, x2);
        }
    }
    (dist, x)
}

/// 相对旋转 `transpose(rot1) * rot2`：与上游 `collision_sdf.py:724`（`params.rel_mat =
/// wp.transpose(rot1) * rot2`）同语义，行优先 `rel[row][col] = Σ_k rot1[k][row] * rot2[k][col]`。
/// 复用 `relmat::upstream_relative_matrix`（fma 链累加，对应上游 `warp/native/mat.h`
/// `mul(mat,mat)` 的 `muladd`），不保留第二份同名实现。
pub(crate) fn relative_matrix(rot1: &[f32; 9], rot2: &[f32; 9]) -> [f32; 9] {
    relmat::upstream_relative_matrix(rot1, rot2)
}

/// 上游 `gradient_descent`：`sdf_iterations` 次普通步 + 1 次 intersection 步，
/// 再归一化两条梯度求接触法线、位置（`pos = rot2*x + pos2`、`pos3 = pos - n*dist/2`）。
#[allow(clippy::too_many_arguments)]
pub fn gradient_descent(
    shape1: &SdfShape,
    shape2: &SdfShape,
    x0_initial: [f32; 3],
    pos1: [f32; 3],
    rot1: [f32; 9],
    pos2: [f32; 3],
    rot2: [f32; 9],
    sdf_iterations: usize,
) -> (f32, [f32; 3], [f32; 3]) {
    let params = OptimizationParams {
        rel_mat: relative_matrix(&rot1, &rot2),
        rel_pos: transpose_mul_mat3_vec(&rot1, sub3(pos2, pos1)),
        attr1: shape1.size,
        attr2: shape2.size,
    };
    let (_, x) = gradient_step(shape1, shape2, x0_initial, &params, sdf_iterations, false);
    let (dist, x) = gradient_step(shape1, shape2, x, &params, 1, true);
    let x1 = add3(mul_mat3_vec(&params.rel_mat, x), params.rel_pos);
    let grad1 = normalize3(transpose_mul_mat3_vec(
        &params.rel_mat,
        sdf_grad(shape1, x1),
    ));
    let grad2 = normalize3(forced_sdf_grad(shape2, x));
    let n = normalize3(sub3(grad1, grad2));
    let pos = add3(mul_mat3_vec(&rot2, x), pos2);
    let n = mul_mat3_vec(&rot2, n);
    let pos3 = sub3(pos, scale3(n, dist / 2.0));
    (dist, pos3, n)
}
