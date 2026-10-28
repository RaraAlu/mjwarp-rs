//! R05-S2 地形 hfield 三角化（宿主；逐条对齐冻结上游 `bvh.py`）。
//!
//! 上游锚点（MJWarp 3.15，`71da24d`）：
//! - `bvh.py::_optimize_hfield_mesh` L453-578：贪心共面合并 + 顶点/索引顺序；
//! - `bvh.py::build_hfield_bvh` L581-622：`nr/nc/data/size` 读取、`width/height`、
//!   `points/indices` 构建、`half = 0.5*(pmax-pmin)`。
//!
//! ## 精度口径（pinned 上游环境 numpy 2.5.4，NEP 50 弱提升）
//!
//! 上游 `_optimize_hfield_mesh` 里 `sx`/`sz_scale` 是 `np.float32`、`width`/`height` 是
//! Python float；NEP 50 下 Python float 作为弱标量先转 `float32` 再参与 `float32` 运算：
//! - `x = sx * (float(c) / width - 1.0)`：括号内按 f64 算，再转 f32，再 f32 乘 `sx`；
//! - `y = sy * (float(r) / height - 1.0)`：同上；
//! - `z = float(data[r, c]) * sz_scale`：`data` 是 f32，`float(·)` 无损，`f32 * f32`；
//! - 共面判据 `abs((z00+z11)-(z01+z10)) < 1e-5` 与 `fits_plane` 的预测/斜率比较
//!   全在 `float32` 下算（`python float` 阈值转 f32；该阈值不落在相邻 f32 之间，
//!   故与 f64 精确比较等价）；
//! - `half = 0.5*(pmax-pmin)`：逐轴 f32 减、再乘 0.5（2 的幂，无舍入）。
//!
//! ## 语义边界
//!
//! - 网格三角形 BVH 复用 [`MeshBvh::build`]（分箱 SAH、`LEAF_CAPACITY=8`）；
//!   与上游 `wp.Mesh(bvh_constructor="sah", bvh_leaf_size=2)` 的差异见 S2 证据（策略差异，
//!   不改上游参考/容差）。
//! - 本模块只产**宿主几何**（顶点/索引/BVH/半尺寸），不产任何渲染数值结论。
//! - 网格退化（`nrow < 2 || ncol < 2` → 无三角形）按上游会先于 `wp.Mesh` 在 `np.min`
//!   处失败；这里给可读错误 `HfieldEmptyMesh`（不 panic）。

use std::collections::HashMap;

use super::{BvhError, MeshBvh, MeshBvhInput};

/// 单个 hfield 的三角化输入（高程数据已按 `data[nrow × ncol]` 行主序切好）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HfieldMeshInput<'a> {
    /// 高程行数（上游 `mjm.hfield_nrow[hfieldid]`）。
    pub nrow: i32,
    /// 高程列数（上游 `mjm.hfield_ncol[hfieldid]`）。
    pub ncol: i32,
    /// `hfield_size` 前三项 `(sx, sy, sz_scale)` 的 f32 原值。
    pub size: [f32; 3],
    /// 高程数据 `data[r, c]`（长度须 `>= nrow*ncol`；只读行主序）。
    pub data: &'a [f32],
}

/// 单个 hfield 的三角化产物：网格三角 BVH + 上游 `half` 半尺寸。
#[derive(Debug, PartialEq)]
pub struct HfieldMesh {
    bvh: MeshBvh,
    half: [f32; 3],
}

impl HfieldMesh {
    /// 网格三角 BVH（顶点/索引保持上游插入顺序）。
    pub fn bvh(&self) -> &MeshBvh {
        &self.bvh
    }

    /// 上游 `build_hfield_bvh` 返回的 `half = 0.5*(pmax-pmin)`（逐轴 f32）。
    ///
    /// 场景 BVH 的 hfield leaf 用它作为 box 半尺寸，并按 `center = pos + rot[:,2]*half[2]`
    /// 偏移（`bvh.py` L225-233）。
    pub fn half(&self) -> [f32; 3] {
        self.half
    }

    /// 顶点（f32 位型，上游 `np.array(points_list, dtype=np.float32)`）。
    pub fn vertices(&self) -> &[[f32; 3]] {
        self.bvh.vertices()
    }

    /// 三角形索引（每 3 个一组，上游 `indices_list`）。
    pub fn indices(&self) -> &[u32] {
        self.bvh.indices()
    }
}

/// 构建 hfield 网格三角 BVH：三角化（上游 `_optimize_hfield_mesh`）+ [`MeshBvh::build`]。
pub fn build_hfield_mesh(input: &HfieldMeshInput<'_>) -> Result<HfieldMesh, BvhError> {
    let (vertices, indices) = triangulate_hfield(input)?;
    let bvh = MeshBvh::build(MeshBvhInput {
        vertices: &vertices,
        indices: &indices,
    })
    .map_err(BvhError::HfieldMesh)?;
    let half = half_extents(&vertices);
    Ok(HfieldMesh { bvh, half })
}

/// 上游 `_optimize_hfield_mesh`：贪心共面合并后返回 `(points, indices)`。
///
/// 逐条对应上游（行号为 `bvh.py`）：
/// - L472-484 `get_point_index`：顶点首次出现才插入（`points_map` 去重）；
/// - L488-513 非共面单格：`TL,TR,BR` + `TL,BR,BL` 两个三角形；
/// - L516-565 共面扩展：先宽后高，`fits_plane` 逐格校验共面 + 同平面 + 斜率一致；
/// - L567-576 合并矩形：同样 `TL,TR,BR` + `TL,BR,BL` 顺序。
pub fn triangulate_hfield(
    input: &HfieldMeshInput<'_>,
) -> Result<(Vec<[f32; 3]>, Vec<u32>), BvhError> {
    let nr = input.nrow;
    let nc = input.ncol;
    if nr <= 0 || nc <= 0 {
        return Err(BvhError::HfieldDimensions { nrow: nr, ncol: nc });
    }
    let nr = nr as usize;
    let nc = nc as usize;
    let cells = nr.checked_mul(nc).ok_or(BvhError::HfieldCountOverflow {
        nrow: input.nrow,
        ncol: input.ncol,
    })?;
    if input.data.len() < cells {
        return Err(BvhError::HfieldDataLength {
            expected: cells,
            actual: input.data.len(),
        });
    }
    if nr < 2 || nc < 2 {
        return Err(BvhError::HfieldEmptyMesh {
            nrow: input.nrow,
            ncol: input.ncol,
        });
    }

    // L595-596：`width = 0.5*max(nc-1,1)`、`height = 0.5*max(nr-1,1)`（Python float）。
    let width = 0.5 * (nc - 1).max(1) as f64;
    let height = 0.5 * (nr - 1).max(1) as f64;
    let sx = input.size[0];
    let sy = input.size[1];
    let sz_scale = input.size[2];
    let data = input.data;

    let mut points: Vec<[f32; 3]> = Vec::new();
    let mut points_map: HashMap<(usize, usize), u32> = HashMap::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut visited = vec![false; (nr - 1) * (nc - 1)];

    // L472-484 `get_point_index`：坐标按 NEP 50 的 float32 口径（见模块文档）。
    let mut point_index = |r: usize, c: usize, points: &mut Vec<[f32; 3]>| -> u32 {
        if let Some(index) = points_map.get(&(r, c)) {
            return *index;
        }
        let x = sx * (((c as f64) / width - 1.0) as f32);
        let y = sy * (((r as f64) / height - 1.0) as f32);
        let z = data[r * nc + c] * sz_scale;
        let index = points.len() as u32;
        points.push([x, y, z]);
        points_map.insert((r, c), index);
        index
    };

    // L499-500：共面判据（f32 差值与 f32 阈值；与 f64 精确比较等价，见模块文档）。
    let threshold = 1.0e-5f64;
    let planar = |d00: f32, d01: f32, d10: f32, d11: f32| -> bool {
        let diff = (d00 + d11) - (d01 + d10);
        f64::from(diff.abs()) < threshold
    };

    for r in 0..(nr - 1) {
        for c in 0..(nc - 1) {
            if visited[r * (nc - 1) + c] {
                continue;
            }
            let z00 = data[r * nc + c];
            let z01 = data[r * nc + c + 1];
            let z10 = data[(r + 1) * nc + c];
            let z11 = data[(r + 1) * nc + c + 1];

            if !planar(z00, z01, z10, z11) {
                // L504-513：单格两个三角形（TL,TR,BL,BR 取点顺序）。
                let i00 = point_index(r, c, &mut points);
                let i01 = point_index(r, c + 1, &mut points);
                let i10 = point_index(r + 1, c, &mut points);
                let i11 = point_index(r + 1, c + 1, &mut points);
                indices.extend_from_slice(&[i00, i01, i11]);
                indices.extend_from_slice(&[i00, i11, i10]);
                visited[r * (nc - 1) + c] = true;
                continue;
            }

            // L517-545：以起始格斜率定义平面，`fits_plane` 逐格校验。
            let slope_x = z01 - z00;
            let slope_y = z10 - z00;
            let fits_plane = |rr: usize, cc: usize| -> bool {
                if rr >= nr - 1 || cc >= nc - 1 {
                    return false;
                }
                let cz00 = data[rr * nc + cc];
                let cz01 = data[rr * nc + cc + 1];
                let cz10 = data[(rr + 1) * nc + cc];
                let cz11 = data[(rr + 1) * nc + cc + 1];
                if !planar(cz00, cz01, cz10, cz11) {
                    return false;
                }
                // L535：`z_pred = z00 + (rr-r)*slope_y + (cc-c)*slope_x`（f32 左结合）。
                let mut z_pred = z00 + ((rr - r) as f32) * slope_y;
                z_pred += ((cc - c) as f32) * slope_x;
                if f64::from((cz00 - z_pred).abs()) >= threshold {
                    return false;
                }
                let cslope_x = cz01 - cz00;
                let cslope_y = cz10 - cz00;
                if f64::from((cslope_x - slope_x).abs()) >= threshold
                    || f64::from((cslope_y - slope_y).abs()) >= threshold
                {
                    return false;
                }
                true
            };

            // L547-563：先扩宽，再整行扩高。
            let mut w = 1usize;
            while c + w < nc - 1 && !visited[r * (nc - 1) + (c + w)] && fits_plane(r, c + w) {
                w += 1;
            }
            let mut h = 1usize;
            while r + h < nr - 1 {
                let mut row_ok = true;
                for k in 0..w {
                    if visited[(r + h) * (nc - 1) + (c + k)] || !fits_plane(r + h, c + k) {
                        row_ok = false;
                        break;
                    }
                }
                if row_ok {
                    h += 1;
                } else {
                    break;
                }
            }

            // L564-565：标记访问。
            for rr in r..(r + h) {
                for cc in c..(c + w) {
                    visited[rr * (nc - 1) + cc] = true;
                }
            }

            // L567-576：合并矩形（TL,TR,BL,BR 取点顺序；两个三角形顺序不变）。
            let idx_tl = point_index(r, c, &mut points);
            let idx_tr = point_index(r, c + w, &mut points);
            let idx_bl = point_index(r + h, c, &mut points);
            let idx_br = point_index(r + h, c + w, &mut points);
            indices.extend_from_slice(&[idx_tl, idx_tr, idx_br]);
            indices.extend_from_slice(&[idx_tl, idx_br, idx_bl]);
        }
    }

    Ok((points, indices))
}

/// 上游 L608-610：`half = 0.5*(pmax-pmin)`（逐轴 f32；2 的幂缩放无舍入）。
fn half_extents(vertices: &[[f32; 3]]) -> [f32; 3] {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(vertex[axis]);
            maximum[axis] = maximum[axis].max(vertex[axis]);
        }
    }
    let mut half = [0.0f32; 3];
    for axis in 0..3 {
        half[axis] = 0.5f32 * (maximum[axis] - minimum[axis]);
    }
    half
}
