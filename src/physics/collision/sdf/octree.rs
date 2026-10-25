//! 上游 `collision_sdf.py` 的 octree 体积采样：`box_project`、`find_oct`、
//! `sample_volume_sdf`/`sample_volume_grad` 与 Halton 序列。
//! 全部 f32；不分配设备缓冲。

/// 体积 SDF 数据（G1 字段经唯一一次 f64→f32 转换后的上传口径）。
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeData {
    /// 根节点 AABB 中心。
    pub center: [f32; 3],
    /// 根节点 AABB 半尺寸。
    pub half_size: [f32; 3],
    /// 每个节点的 (center, half_size)；`oct_aabb[node,0]`/`[node,1]`。
    pub oct_aabb: Vec<([f32; 3], [f32; 3])>,
    /// 每个节点的 8 个子节点下标（相对根；-1 表示叶）。
    pub oct_child: Vec<[i32; 8]>,
    /// 每个节点的 8 个插值系数。
    pub oct_coeff: Vec<[f32; 8]>,
    /// 根节点下标（= mesh_octadr）。
    pub root: usize,
}

impl VolumeData {
    /// 构造并检查三个数组长度一致、root 在范围内。
    pub fn new(
        center: [f32; 3],
        half_size: [f32; 3],
        oct_aabb: Vec<([f32; 3], [f32; 3])>,
        oct_child: Vec<[i32; 8]>,
        oct_coeff: Vec<[f32; 8]>,
        root: usize,
    ) -> Option<Self> {
        (oct_aabb.len() == oct_child.len()
            && oct_aabb.len() == oct_coeff.len()
            && root < oct_aabb.len())
        .then_some(Self {
            center,
            half_size,
            oct_aabb,
            oct_child,
            oct_coeff,
            root,
        })
    }

    /// 节点数。
    pub fn len(&self) -> usize {
        self.oct_aabb.len()
    }

    /// 是否没有节点。
    pub fn is_empty(&self) -> bool {
        self.oct_aabb.is_empty()
    }
}

/// 上游 `box_project`：把点投影到 AABB，返回盒距与投影点（含 `1e-4` 外扩 eps）。
pub fn box_project(center: [f32; 3], half_size: [f32; 3], xyz: [f32; 3]) -> (f32, [f32; 3]) {
    let r = [xyz[0] - center[0], xyz[1] - center[1], xyz[2] - center[2]];
    let q = [
        r[0].abs() - half_size[0],
        r[1].abs() - half_size[1],
        r[2].abs() - half_size[2],
    ];
    if q[0] <= 0.0 && q[1] <= 0.0 && q[2] <= 0.0 {
        return (0.0, xyz);
    }
    let mut dist_sqr = 0.0_f32;
    let eps = 1.0e-4_f32;
    let mut point = xyz;
    if q[0] >= 0.0 {
        dist_sqr += q[0] * q[0];
        point[0] = if r[0] > 0.0 {
            point[0] - (q[0] + eps)
        } else {
            point[0] + (q[0] + eps)
        };
    }
    if q[1] >= 0.0 {
        dist_sqr += q[1] * q[1];
        point[1] = if r[1] > 0.0 {
            point[1] - (q[1] + eps)
        } else {
            point[1] + (q[1] + eps)
        };
    }
    if q[2] >= 0.0 {
        dist_sqr += q[2] * q[2];
        point[2] = if r[2] > 0.0 {
            point[2] - (q[2] + eps)
        } else {
            point[2] + (q[2] + eps)
        };
    }
    (dist_sqr.sqrt(), point)
}

fn dot8(a: [f32; 8], b: [f32; 8]) -> f32 {
    let mut sum = 0.0_f32;
    for index in 0..8 {
        sum += a[index] * b[index];
    }
    sum
}

/// 上游 `find_oct`：从根按坐标子索引下降到叶，返回叶节点与三轴权重。
/// 返回 `None` 表示超过 100 次迭代仍未命中（上游打印错误并返回 -1）。
#[allow(clippy::needless_range_loop)]
pub fn find_oct(
    volume: &VolumeData,
    p: [f32; 3],
    grad: bool,
    root: usize,
) -> Option<(usize, [[f32; 8]; 3])> {
    let mut stack = root as i64;
    let mut niter = 100;
    let mut rx = [0.0_f32; 8];
    let mut ry = [0.0_f32; 8];
    let mut rz = [0.0_f32; 8];
    let eps = 1.0e-6_f32;
    while niter > 0 {
        niter -= 1;
        let node = stack;
        if node < 0 {
            return None;
        }
        let (center, half) = volume.oct_aabb[node as usize];
        let vmin = [
            center[0] - half[0],
            center[1] - half[1],
            center[2] - half[2],
        ];
        let vmax = [
            center[0] + half[0],
            center[1] + half[1],
            center[2] + half[2],
        ];
        if p[0] + eps < vmin[0]
            || p[0] - eps > vmax[0]
            || p[1] + eps < vmin[1]
            || p[1] - eps > vmax[1]
            || p[2] + eps < vmin[2]
            || p[2] - eps > vmax[2]
        {
            continue;
        }
        let coord = [
            (p[0] - vmin[0]) / (vmax[0] - vmin[0]),
            (p[1] - vmin[1]) / (vmax[1] - vmin[1]),
            (p[2] - vmin[2]) / (vmax[2] - vmin[2]),
        ];
        let child = &volume.oct_child[node as usize];
        if child.iter().all(|&index| index == -1) {
            if !grad {
                for j in 0..8 {
                    let x = if j & 1 != 0 { coord[0] } else { 1.0 - coord[0] };
                    let y = if j & 2 != 0 { coord[1] } else { 1.0 - coord[1] };
                    let z = if j & 4 != 0 { coord[2] } else { 1.0 - coord[2] };
                    rx[j] = x * y * z;
                }
            } else {
                let inv_cell = [
                    1.0 / (vmax[0] - vmin[0]),
                    1.0 / (vmax[1] - vmin[1]),
                    1.0 / (vmax[2] - vmin[2]),
                ];
                for j in 0..8 {
                    let x = if j & 1 != 0 { coord[0] } else { 1.0 - coord[0] };
                    let y = if j & 2 != 0 { coord[1] } else { 1.0 - coord[1] };
                    let z = if j & 4 != 0 { coord[2] } else { 1.0 - coord[2] };
                    let sx = if j & 1 != 0 { 1.0 } else { -1.0 };
                    let sy = if j & 2 != 0 { 1.0 } else { -1.0 };
                    let sz = if j & 4 != 0 { 1.0 } else { -1.0 };
                    rx[j] = sx
                        * (if j & 2 != 0 { coord[1] } else { 1.0 - coord[1] })
                        * (if j & 4 != 0 { coord[2] } else { 1.0 - coord[2] })
                        * inv_cell[0];
                    ry[j] = (if j & 1 != 0 { coord[0] } else { 1.0 - coord[0] })
                        * sy
                        * (if j & 4 != 0 { coord[2] } else { 1.0 - coord[2] })
                        * inv_cell[1];
                    rz[j] = (if j & 1 != 0 { coord[0] } else { 1.0 - coord[0] })
                        * (if j & 2 != 0 { coord[1] } else { 1.0 - coord[1] })
                        * sz
                        * inv_cell[2];
                    let _ = (x, y, z);
                }
            }
            return Some((node as usize, [rx, ry, rz]));
        }
        let x = if coord[0] < 0.5 { 0 } else { 1 };
        let y = if coord[1] < 0.5 { 0 } else { 1 };
        let z = if coord[2] < 0.5 { 0 } else { 1 };
        let next = child[4 * z + 2 * y + x];
        stack = if next != -1 {
            i64::from(next) + root as i64
        } else {
            -1
        };
    }
    None
}

/// 上游 `sample_volume_sdf`：盒距 + 叶节点三线性插值系数点积。
pub fn sample_volume_sdf(volume: &VolumeData, xyz: [f32; 3]) -> Option<f32> {
    let (dist0, point) = box_project(volume.center, volume.half_size, xyz);
    let (node, weights) = find_oct(volume, point, false, volume.root)?;
    Some(dist0 + dot8(weights[0], volume.oct_coeff[node]))
}

/// 上游 `sample_volume_grad`：盒外有限差分（h=1e-4），盒内解析权重梯度。
pub fn sample_volume_grad(volume: &VolumeData, xyz: [f32; 3]) -> Option<[f32; 3]> {
    let (dist0, point) = box_project(volume.center, volume.half_size, xyz);
    if dist0 > 0.0 {
        let h = 1.0e-4_f32;
        let f = sample_volume_sdf(volume, xyz)?;
        let grad_x = (sample_volume_sdf(volume, [xyz[0] + h, xyz[1], xyz[2]])? - f) / h;
        let grad_y = (sample_volume_sdf(volume, [xyz[0], xyz[1] + h, xyz[2]])? - f) / h;
        let grad_z = (sample_volume_sdf(volume, [xyz[0], xyz[1], xyz[2] + h])? - f) / h;
        return Some([grad_x, grad_y, grad_z]);
    }
    let (node, weights) = find_oct(volume, point, true, volume.root)?;
    Some([
        dot8(weights[0], volume.oct_coeff[node]),
        dot8(weights[1], volume.oct_coeff[node]),
        dot8(weights[2], volume.oct_coeff[node]),
    ])
}

/// 上游 `halton`：van der Corput 基数序列（f32 累加顺序与上游一致）。
pub fn halton(index: u32, base: u32) -> f32 {
    let mut n0 = index;
    let b = base as f32;
    let mut f = 1.0_f32 / b;
    let mut hn = 0.0_f32;
    while n0 > 0 {
        let n1 = n0 / base;
        let r = n0 - n1 * base;
        hn += f * (r as f32);
        f /= b;
        n0 = n1;
    }
    hn
}
