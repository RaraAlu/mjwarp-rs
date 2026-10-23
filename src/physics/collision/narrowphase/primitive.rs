//! 基础几何原语：向量辅助、`make_frame`、plane-sphere/plane-box/sphere-sphere 核心。

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn length3(vector: [f32; 3]) -> f32 {
    dot3(vector, vector).sqrt()
}

fn normalize3(vector: [f32; 3]) -> [f32; 3] {
    let norm = length3(vector);
    [vector[0] / norm, vector[1] / norm, vector[2] / norm]
}

fn sub3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale3(vector: [f32; 3], factor: f32) -> [f32; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

/// 列向量：上游 `wp.vec3(rot[0, k], rot[1, k], rot[2, k])`。
pub(super) fn column(matrix: &[f32; 9], column: usize) -> [f32; 3] {
    [matrix[column], matrix[3 + column], matrix[6 + column]]
}

/// 上游 `math.make_frame`：以法线为第一行的接触坐标系（行优先）。
pub fn make_frame(normal: [f32; 3]) -> [f32; 9] {
    let a = normalize3(normal);
    let y = [0.0_f32, 1.0, 0.0];
    let z = [0.0_f32, 0.0, 1.0];
    let selector = if -0.5 < a[1] && a[1] < 0.5 { y } else { z };
    let mut b = sub3(selector, scale3(a, dot3(a, selector)));
    b = normalize3(b);
    if length3(a) == 0.0 {
        b = [0.0; 3];
    }
    let c = cross3(a, b);
    [a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]
}

/// 上游 `collision_primitive_core.plane_sphere`。
pub fn plane_sphere(
    plane_normal: [f32; 3],
    plane_pos: [f32; 3],
    sphere_pos: [f32; 3],
    sphere_radius: f32,
) -> (f32, [f32; 3]) {
    let dist = dot3(sub3(sphere_pos, plane_pos), plane_normal) - sphere_radius;
    let pos = sub3(sphere_pos, scale3(plane_normal, sphere_radius + 0.5 * dist));
    (dist, pos)
}

/// 上游 `collision_primitive_core.sphere_sphere`。
pub fn sphere_sphere(
    pos1: [f32; 3],
    radius1: f32,
    pos2: [f32; 3],
    radius2: f32,
) -> (f32, [f32; 3], [f32; 3]) {
    let mut dir = sub3(pos2, pos1);
    let mut dist = length3(dir);
    if dist == 0.0 {
        dir = [1.0, 0.0, 0.0];
    } else {
        dir = scale3(dir, 1.0 / dist);
    }
    let normal = dir;
    dist -= radius1 + radius2;
    let pos = [
        pos1[0] + normal[0] * (radius1 + 0.5 * dist),
        pos1[1] + normal[1] * (radius1 + 0.5 * dist),
        pos1[2] + normal[2] * (radius1 + 0.5 * dist),
    ];
    (dist, pos, normal)
}

/// 行优先 3x3 矩阵乘向量（上游 `wp.mat33 * wp.vec3`）。
fn mul_mat3_vec(matrix: &[f32; 9], vector: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * vector[0] + matrix[1] * vector[1] + matrix[2] * vector[2],
        matrix[3] * vector[0] + matrix[4] * vector[1] + matrix[5] * vector[2],
        matrix[6] * vector[0] + matrix[7] * vector[1] + matrix[8] * vector[2],
    ]
}

/// 上游 `collision_primitive_core.plane_box`：8 个角点到平面的距离与接触位置。
///
/// 角点编号按 `i & 1/2/4` 取 `+/-` 半尺寸（与上游 `wp.where` 位序一致）；
/// wrapper 对 8 个候选依次调用 `write_contact` 完成 margin/gap 过滤。
pub fn plane_box(
    plane_normal: [f32; 3],
    plane_pos: [f32; 3],
    box_pos: [f32; 3],
    box_rot: [f32; 9],
    box_size: [f32; 3],
) -> ([f32; 8], [[f32; 3]; 8]) {
    let center_dist = dot3(sub3(box_pos, plane_pos), plane_normal);
    let mut dist = [0.0_f32; 8];
    let mut pos = [[0.0_f32; 3]; 8];
    for index in 0..8 {
        let corner = [
            if index & 1 != 0 {
                box_size[0]
            } else {
                -box_size[0]
            },
            if index & 2 != 0 {
                box_size[1]
            } else {
                -box_size[1]
            },
            if index & 4 != 0 {
                box_size[2]
            } else {
                -box_size[2]
            },
        ];
        let offset = mul_mat3_vec(&box_rot, corner);
        let cdist = center_dist + dot3(plane_normal, offset);
        dist[index] = cdist;
        let mut point = [
            offset[0] + box_pos[0],
            offset[1] + box_pos[1],
            offset[2] + box_pos[2],
        ];
        for axis in 0..3 {
            point[axis] -= 0.5 * plane_normal[axis] * cdist;
        }
        pos[index] = point;
    }
    (dist, pos)
}
