//! primitive geom 的 leaf AABB，逐 f32 位型复刻冻结 MJWarp 3.15 参考。
//!
//! 参考 kernel：`mujoco_warp/_src/bvh.py::_compute_bvh_bounds`（提交
//! `71da24d956378a87a703b6e1442b13aec0c4ac29`）。该 kernel 由 NVRTC 编译，
//! warp 的 `wp::mul(mat, vec)` / `wp::dot` 被收缩成固定的乘加顺序：
//!
//! - 三项和 `((c0*b0) + (c1*b1)) + (c2*b2)` 编译为：
//!   `t = fma(c0, b0, round(c1*b1)); t = fma(c2, b2, t)`
//!   （首项与第二项先融合，末项再融合；中间项单独圆整）。
//! - 两项和 `c0*b0 + c1*b1` 编译为 `fma(c0, b0, round(c1*b1))`。
//! - `pos ± z*half` 编译为 `fma(±z, half, pos)`。
//!
//! 该顺序由 8 个位姿（初始/移动 base body/6 个校准位姿）× 7 类 geom 共 56 个
//! leaf AABB 逐位对照钉死，见 S2a 证据的 `aabb-parity.json`；不要改成等价但
//! 圆整顺序不同的写法（如先乘再加）。

use super::{Aabb, BvhError};

/// 与 MuJoCo `geom_type` 对应的 primitive 类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeomKind {
    Plane,
    Sphere,
    Capsule,
    Ellipsoid,
    Cylinder,
    Box,
    /// mesh/SDF 的 leaf 包围盒：用 mesh bounds（半尺寸）走 box 公式。
    /// 网格三角形 BVH 属 S3，本切片不建。
    Mesh,
}

/// 单个 leaf 的几何输入。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafGeometry {
    pub kind: GeomKind,
    /// `geom_size` 的 f32 值；`Mesh` 时是 mesh bounds 的半尺寸。
    pub size: [f32; 3],
}

impl LeafGeometry {
    /// 由 MuJoCo `geom_type` 与 f32 尺寸构造。
    pub fn from_geom_type(geom_type: i32, size: [f32; 3]) -> Result<Self, BvhError> {
        let kind = match geom_type {
            0 => GeomKind::Plane,
            2 => GeomKind::Sphere,
            3 => GeomKind::Capsule,
            4 => GeomKind::Ellipsoid,
            5 => GeomKind::Cylinder,
            6 => GeomKind::Box,
            7 | 8 => GeomKind::Mesh,
            other => return Err(BvhError::UnsupportedGeomType { geom_type: other }),
        };
        Ok(Self { kind, size })
    }

    /// mesh/SDF 的 mesh bounds 半尺寸按上游 `build_mesh_bvh` 语义：
    /// `max(|min|, |max|)` 逐轴（顶点已是 f32 时与上游 f64→f32 结果位型一致）。
    pub fn mesh(mesh_half_extents: [f32; 3]) -> Self {
        Self {
            kind: GeomKind::Mesh,
            size: mesh_half_extents,
        }
    }
}

/// `build_mesh_bvh` 的 `half` 计算：逐轴 `max(|min|, |max|)`。
///
/// 上游在 f64 顶点上取 min/max 再转 f32；产品侧顶点是 f32 位型，f32 转换与
/// min/max 都保持顺序，因此结果位型一致（S2a 证据中有对照）。
pub fn mesh_half_extents(vertices: &[[f32; 3]]) -> Result<[f32; 3], BvhError> {
    if vertices.is_empty() {
        return Err(BvhError::MissingMeshBounds);
    }
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            validate_finite("mesh_vert", axis, vertex[axis])?;
            minimum[axis] = minimum[axis].min(vertex[axis]);
            maximum[axis] = maximum[axis].max(vertex[axis]);
        }
    }
    let mut half = [0.0f32; 3];
    for axis in 0..3 {
        half[axis] = minimum[axis].abs().max(maximum[axis].abs());
    }
    Ok(half)
}

/// 计算单个 leaf 的 AABB。
///
/// `rotation` 是 geom 世界旋转矩阵，按行优先，列是 geom 局部轴。
pub fn compute_leaf_aabb(
    geometry: &LeafGeometry,
    position: [f32; 3],
    rotation: [[f32; 3]; 3],
) -> Result<Aabb, BvhError> {
    for axis in 0..3 {
        validate_finite("geom_xpos", axis, position[axis])?;
        validate_finite("geom_size", axis, geometry.size[axis])?;
        for (column, value) in rotation[axis].iter().enumerate() {
            validate_finite("geom_xmat", axis * 3 + column, *value)?;
        }
    }
    Ok(match geometry.kind {
        GeomKind::Plane => plane_bounds(position, rotation, geometry.size),
        GeomKind::Sphere => sphere_bounds(position, geometry.size),
        GeomKind::Capsule => capsule_bounds(position, rotation, geometry.size),
        GeomKind::Ellipsoid => ellipsoid_bounds(position, rotation, geometry.size),
        GeomKind::Cylinder => cylinder_bounds(position, rotation, geometry.size),
        GeomKind::Box | GeomKind::Mesh => box_bounds(position, rotation, geometry.size),
    })
}

fn validate_finite(field: &'static str, index: usize, value: f32) -> Result<(), BvhError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(BvhError::NonFinite { field, index })
    }
}

/// 复刻冻结参考的 `fma.rn.f32`。
///
/// **边界（本片不改实现）**：该 f64 helper 属 render 加速结构（AABB），不进入产品物理数值链；
/// 物理数值链（`physics`/`runtime` 的 rel_mat 等）的 f64 私有工作区已由
/// `2026-10-25-exact-f32-fma-relmat` 换为 `src/math/fma.rs` 的全 f32 正确舍入 helper。
/// render 侧是否也改用共享 helper 归 render 口径另行决定（契约 L341「原有辅助也不能继续豁免」
/// 的 NP01 表述留给 render 侧评估），本片仅注明边界、不改变行为。
///
/// 宿主侧不直接用 `f32::mul_add`：本目标 debug 构建会落到不保证单次舍入的回退
/// 实现，与 release 位型可能差 1 ULP（会破坏位等验收与调试/发布一致性）。
/// 这里用 f64 计算（f32*f32 在 f64 中精确，再做一次 f64 加法与一次 f32 舍入），
/// 在 8 个位姿 × 7 类 leaf 的 56 个体上与冻结 MJWarp 参考逐位一致；
/// AABB 只作为加速结构，不进入产品物理数值链。
#[inline]
fn fused_mul_add(a: f32, b: f32, c: f32) -> f32 {
    (a as f64 * b as f64 + c as f64) as f32
}

/// 上游 `_compute_box_bounds`：8 个角点 `pos + rot @ (±size)` 后取 min/max。
fn box_bounds(position: [f32; 3], rotation: [[f32; 3]; 3], size: [f32; 3]) -> Aabb {
    let mut minimum = [1.0e10f32; 3];
    let mut maximum = [-1.0e10f32; 3];
    for index in 0..8 {
        let corner = [
            signed(size[0], index & 1),
            signed(size[1], (index >> 1) & 1),
            signed(size[2], (index >> 2) & 1),
        ];
        for axis in 0..3 {
            let world = position[axis] + matvec_component(rotation[axis], corner);
            minimum[axis] = minimum[axis].min(world);
            maximum[axis] = maximum[axis].max(world);
        }
    }
    Aabb {
        lower: minimum,
        upper: maximum,
    }
}

/// 上游 `_compute_sphere_bounds`。
fn sphere_bounds(position: [f32; 3], size: [f32; 3]) -> Aabb {
    let radius = size[0];
    let mut lower = [0.0f32; 3];
    let mut upper = [0.0f32; 3];
    for axis in 0..3 {
        lower[axis] = position[axis] - radius;
        upper[axis] = position[axis] + radius;
    }
    Aabb { lower, upper }
}

/// 上游 `_compute_capsule_bounds`：`pos ± rot[:,2]*half_length` 再按半径膨胀。
fn capsule_bounds(position: [f32; 3], rotation: [[f32; 3]; 3], size: [f32; 3]) -> Aabb {
    let radius = size[0];
    let half_length = size[1];
    let mut lower = [0.0f32; 3];
    let mut upper = [0.0f32; 3];
    for axis in 0..3 {
        let axis_z = rotation[axis][2];
        let end1 = fused_mul_add(-axis_z, half_length, position[axis]);
        let end2 = fused_mul_add(axis_z, half_length, position[axis]);
        lower[axis] = end1.min(end2) - radius;
        upper[axis] = end1.max(end2) + radius;
    }
    Aabb { lower, upper }
}

/// 上游 `_compute_ellipsoid_bounds`：逐轴 `pos ± |rot*diag(size)|` 的行范数。
fn ellipsoid_bounds(position: [f32; 3], rotation: [[f32; 3]; 3], size: [f32; 3]) -> Aabb {
    let mut extent = [0.0f32; 3];
    for axis in 0..3 {
        let row = [
            rotation[axis][0] * size[0],
            rotation[axis][1] * size[1],
            rotation[axis][2] * size[2],
        ];
        extent[axis] = dot3(row, row).sqrt();
    }
    let mut lower = [0.0f32; 3];
    let mut upper = [0.0f32; 3];
    for axis in 0..3 {
        lower[axis] = position[axis] - extent[axis];
        upper[axis] = position[axis] + extent[axis];
    }
    Aabb { lower, upper }
}

/// 上游 `_compute_cylinder_bounds`：半径方向投影 + 半高沿轴投影。
fn cylinder_bounds(position: [f32; 3], rotation: [[f32; 3]; 3], size: [f32; 3]) -> Aabb {
    let radius = size[0];
    let half_height = size[1];
    let mut lower = [0.0f32; 3];
    let mut upper = [0.0f32; 3];
    for axis in 0..3 {
        let axis_z = rotation[axis][2];
        let basis_x = rotation[axis][0];
        let basis_y = rotation[axis][1];
        let squared = fused_mul_add(basis_x, basis_x, basis_y * basis_y);
        let radial = radius * squared.sqrt();
        let extent = fused_mul_add(half_height, axis_z.abs(), radial);
        lower[axis] = position[axis] - extent;
        upper[axis] = position[axis] + extent;
    }
    Aabb { lower, upper }
}

/// 上游 `_compute_plane_bounds`：正方形范围 `2*max(size0,size1)`（非正尺寸时 1000），
/// 角点做 box 公式后各轴外扩 0.01。
fn plane_bounds(position: [f32; 3], rotation: [[f32; 3]; 3], size: [f32; 3]) -> Aabb {
    let mut size_scale = size[0].max(size[1]) * 2.0;
    if size[0] <= 0.0 || size[1] <= 0.0 {
        size_scale = 1000.0;
    }
    let mut minimum = [1.0e10f32; 3];
    let mut maximum = [-1.0e10f32; 3];
    for index in 0..4 {
        let corner = [
            signed(size_scale, index & 1),
            signed(size_scale, (index >> 1) & 1),
            0.0,
        ];
        for axis in 0..3 {
            let world = position[axis] + matvec_component(rotation[axis], corner);
            minimum[axis] = minimum[axis].min(world);
            maximum[axis] = maximum[axis].max(world);
        }
    }
    for axis in 0..3 {
        minimum[axis] -= 0.01;
        maximum[axis] += 0.01;
    }
    Aabb {
        lower: minimum,
        upper: maximum,
    }
}

/// `±size`：乘以 ±1 是精确的，保持上游角点写法。
fn signed(size: f32, positive: usize) -> f32 {
    if positive == 0 { -size } else { size }
}

/// 冻结参考的 `wp::mul(mat33, vec3)` 单个分量：`fma(c0,b0, round(c1*b1))` 再 `fma(c2,b2,·)`。
fn matvec_component(row: [f32; 3], vector: [f32; 3]) -> f32 {
    let intermediate = row[1] * vector[1];
    let intermediate = fused_mul_add(row[0], vector[0], intermediate);
    fused_mul_add(row[2], vector[2], intermediate)
}

/// 冻结参考的 `wp::dot(vec3, vec3)`：与 `wp::mul` 同一收缩规则。
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let intermediate = a[1] * b[1];
    let intermediate = fused_mul_add(a[0], b[0], intermediate);
    fused_mul_add(a[2], b[2], intermediate)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    #[test]
    fn axis_aligned_boxes_match_hand_arithmetic() {
        let geometry = LeafGeometry::from_geom_type(6, [0.5, 0.25, 0.125]).unwrap();
        let bounds = compute_leaf_aabb(&geometry, [1.0, -2.0, 3.0], IDENTITY).unwrap();
        assert_eq!(bounds.lower, [0.5, -2.25, 2.875]);
        assert_eq!(bounds.upper, [1.5, -1.75, 3.125]);
    }

    #[test]
    fn sphere_and_capsule_follow_upstream_bounds() {
        let sphere = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(2, [0.25, 0.0, 0.0]).unwrap(),
            [0.5, 0.5, 0.5],
            IDENTITY,
        )
        .unwrap();
        assert_eq!(sphere.lower, [0.25, 0.25, 0.25]);
        assert_eq!(sphere.upper, [0.75, 0.75, 0.75]);

        let capsule = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(3, [0.1, 0.5, 0.0]).unwrap(),
            [0.0, 0.0, 0.0],
            IDENTITY,
        )
        .unwrap();
        assert!((capsule.lower[2] + 0.6).abs() < 1.0e-6);
        assert!((capsule.upper[2] - 0.6).abs() < 1.0e-6);
    }

    #[test]
    fn ellipsoid_and_cylinder_extents_use_rotation_columns() {
        let ellipsoid = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(4, [0.1, 0.2, 0.3]).unwrap(),
            [0.0, 0.0, 0.0],
            IDENTITY,
        )
        .unwrap();
        assert_eq!(ellipsoid.lower, [-0.1, -0.2, -0.3]);
        assert_eq!(ellipsoid.upper, [0.1, 0.2, 0.3]);

        let cylinder = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(5, [0.2, 0.5, 0.0]).unwrap(),
            [0.0, 0.0, 0.0],
            IDENTITY,
        )
        .unwrap();
        assert_eq!(cylinder.lower, [-0.2, -0.2, -0.5]);
        assert_eq!(cylinder.upper, [0.2, 0.2, 0.5]);
    }

    #[test]
    fn plane_pads_by_upstream_amount_and_handles_nonpositive_size() {
        let plane = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(0, [2.0, 2.0, 0.1]).unwrap(),
            [0.0, 0.0, 0.0],
            IDENTITY,
        )
        .unwrap();
        assert_eq!(plane.lower, [-4.01, -4.01, -0.01]);
        assert_eq!(plane.upper, [4.01, 4.01, 0.01]);

        let infinite = compute_leaf_aabb(
            &LeafGeometry::from_geom_type(0, [0.0, 0.0, 0.0]).unwrap(),
            [0.0, 0.0, 0.0],
            IDENTITY,
        )
        .unwrap();
        assert_eq!(infinite.lower, [-1000.01, -1000.01, -0.01]);
        assert_eq!(infinite.upper, [1000.01, 1000.01, 0.01]);
    }

    #[test]
    fn mesh_half_extents_match_upstream_formula() {
        let vertices = [
            [0.0, 0.0, 0.0],
            [0.22, 0.0, 0.0],
            [0.0, 0.18, 0.05],
            [0.0, 0.0, 0.2],
        ];
        let half = mesh_half_extents(&vertices).unwrap();
        assert_eq!(half, [0.22, 0.18, 0.2]);

        let negative = [[-0.3, -0.1, 0.2], [0.1, 0.4, -0.5]];
        let half = mesh_half_extents(&negative).unwrap();
        assert_eq!(half, [0.3, 0.4, 0.5]);
    }

    #[test]
    fn fused_mul_add_matches_frozen_reference_rounding() {
        // 来自 initial 位姿 geom 1 的 z 分量：期望位型是冻结 MJWarp 参考的
        // fma.rn.f32 结果（宿主 f64 路径必须逐位一致，不受 debug/release 影响）。
        let first = fused_mul_add(
            f32::from_bits(0x3c0d133e),
            f32::from_bits(0xbe3851ec),
            f32::from_bits(0xbcff5382),
        );
        assert_eq!(first.to_bits(), 0xbd06_02f1);
        let second = fused_mul_add(
            f32::from_bits(0x3f7990cd),
            f32::from_bits(0xbdcccccd),
            first,
        );
        assert_eq!(second.to_bits(), 0xbe05_5442);
    }

    #[test]
    fn rejects_unsupported_types_and_non_finite_inputs() {
        assert_eq!(
            LeafGeometry::from_geom_type(1, [0.0; 3]),
            Err(BvhError::UnsupportedGeomType { geom_type: 1 })
        );
        let geometry = LeafGeometry::from_geom_type(6, [1.0, 1.0, 1.0]).unwrap();
        for position in [[f32::NAN, 0.0, 0.0], [0.0, f32::INFINITY, 0.0]] {
            assert!(matches!(
                compute_leaf_aabb(&geometry, position, IDENTITY),
                Err(BvhError::NonFinite { .. })
            ));
        }
    }
}
