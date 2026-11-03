//! R06-S3a：2D/3D flex mesh 面生成与三角 BVH 构建/重拟合（渲染/BVH R06）。
//!
//! 逐式翻译冻结上游 `71da24d` 的 `bvh.py`：`build_flex_bvh`（L994-1109，面数/面序）、
//! `_build_flex_2d_elements`（L682-753）、`_build_flex_2d_sides`（L756-814）、
//! `_build_flex_3d_shells`（L817-860）、`accumulate_flex_vertex_normals`（L626-668）与
//! `normalize_vertex_normals`（L671-678）。
//!
//! 面点缓冲按有效三角绕序直接展开（上游 `face_index[i] = base+i` 为恒等映射，绕序编码在
//! 点排列里；这里等价地按索引三元组落点、索引取连续 `0..3n`）。每个 `(flex, world)` 单独
//! 建树，`group_root` = 该树根（[`MeshBvh`] 最后一个节点）——与上游 `flex_group_root`「该
//! world 分组子树根」语义等价；设备命中复用既有 `MeshBvhQuery`（identity 位姿、
//! `cull_backfaces = false`），不需要新设备内核。
//!
//! 已知差异（2D/3D 未接产品数据；S2c/S3c 接线时在窗口内验证）：上游 2D 顶点法线在并行
//! kernel 里 `+=` 累加（元素间顺序不定），本实现按下标升序确定性累加；三角 BVH 节点布局是
//! 我们的分箱 SAH（叶容量 8），上游是 Warp `sah`/`leaf_size = 2`，验收只看语义命中。

use super::{MeshBvh, MeshBvhError, MeshBvhInput};

/// 2D/3D flex mesh 构建错误；全部在分配前拒绝。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlexMeshError {
    /// 只有 2D/3D flex 有 mesh 面；1D 走胶囊图元。
    UnsupportedDimension { dim: i32 },
    /// 计数为负或面数/点数溢出。
    Count { field: &'static str },
    /// 重拟合时面点数与构建时不一致（拓扑变更不支持）。
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 顶点/拓扑索引越界。
    IndexOutOfRange { field: &'static str, index: usize },
    /// 半径或顶点位置含非有限分量。
    NonFinite { field: &'static str, index: usize },
    /// 三角 BVH 构建失败。
    MeshBvh(MeshBvhError),
}

impl std::fmt::Display for FlexMeshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDimension { dim } => {
                write!(formatter, "flex mesh 只支持 2/3 维（实际 {dim}）")
            }
            Self::Count { field } => write!(formatter, "{field} 计数非法或溢出"),
            Self::Length {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "{field} 长度不匹配：期望 {expected}，实际 {actual}"
            ),
            Self::IndexOutOfRange { field, index } => {
                write!(formatter, "{field}[{index}] 超出可用范围")
            }
            Self::NonFinite { field, index } => write!(formatter, "{field}[{index}] 不是有限数"),
            Self::MeshBvh(error) => write!(formatter, "flex mesh 三角 BVH 失败：{error}"),
        }
    }
}

impl std::error::Error for FlexMeshError {}

/// 2D/3D flex 的显式拓扑输入（S2c 数据层目标形状；本片不接产品数据）。
///
/// 索引语义与上游 `mjm` 一致：`elem`/`shell` 是扁平数组，`*_data_adr` 是该 flex 的起点；
/// 顶点用 `vert_adr + 局部号` 取（`vertices` 传当前 world 的 flex 顶点表）。
pub struct FlexMeshInput<'a> {
    /// 1/2/3；只有 2/3 有 mesh 面。
    pub dim: i32,
    /// 图元半径（2D 顶/底与侧边偏移用）。
    pub radius: f32,
    /// 顶点起始下标。
    pub vert_adr: i32,
    /// 顶点数（局部号上界）。
    pub vert_num: i32,
    /// 2D 元素数据起始下标。
    pub elem_data_adr: i32,
    /// 2D 元素数。
    pub elem_num: i32,
    /// 扁平元素顶点索引（2D：每元素 3 个）。
    pub elem: &'a [i32],
    /// shell 数据起始下标。
    pub shell_data_adr: i32,
    /// shell 片段数。
    pub shell_num: i32,
    /// 扁平 shell 顶点索引（3D：每片段 3 个；2D：每片段 2 个）。
    pub shell: &'a [i32],
}

/// 单个 `(flex, world)` 的 flex mesh：面点/索引与可重拟合的三角 BVH。
pub struct FlexMeshBvh {
    points: Vec<[f32; 3]>,
    indices: Vec<u32>,
    bvh: MeshBvh,
}

impl FlexMeshBvh {
    /// 按上游面序生成三角面并构建 BVH。
    pub fn build(input: &FlexMeshInput<'_>, vertices: &[[f32; 3]]) -> Result<Self, FlexMeshError> {
        let (points, indices) = build_faces(input, vertices)?;
        let bvh = MeshBvh::build(MeshBvhInput {
            vertices: &points,
            indices: &indices,
        })
        .map_err(FlexMeshError::MeshBvh)?;
        Ok(Self {
            points,
            indices,
            bvh,
        })
    }

    /// 用新顶点重算面点并重拟合 BVH（上游 `mesh.points = ...; mesh.refit()`）；拓扑不变。
    pub fn refit(
        &mut self,
        input: &FlexMeshInput<'_>,
        vertices: &[[f32; 3]],
    ) -> Result<(), FlexMeshError> {
        let (points, _) = build_faces(input, vertices)?;
        if points.len() != self.points.len() {
            return Err(FlexMeshError::Length {
                field: "flex_mesh_points",
                expected: self.points.len(),
                actual: points.len(),
            });
        }
        self.points = points;
        self.bvh
            .set_points(&self.points)
            .map_err(FlexMeshError::MeshBvh)?;
        self.bvh.refit();
        Ok(())
    }

    /// 面点（每 3 个点一个三角，顺序即有效绕序）。
    pub fn points(&self) -> &[[f32; 3]] {
        &self.points
    }

    /// 三角数（= 上游 `nface`）。
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// 该 `(flex, world)` 分组的子树根（设备 `group_root` 语义）。
    pub fn group_root(&self) -> u32 {
        (self.bvh.nodes().len() - 1) as u32
    }

    /// 底层三角 BVH（复用既有语义查询/重拟合）。
    pub fn bvh(&self) -> &MeshBvh {
        &self.bvh
    }
}

/// 面数（上游 `build_flex_bvh` L1017-1020）：2D = `2*nelem + 2*nshell`，3D = `nshell`。
fn face_count(dim: i32, elem_num: usize, shell_num: usize) -> Result<usize, FlexMeshError> {
    match dim {
        2 => elem_num
            .checked_mul(2)
            .and_then(|value| {
                shell_num
                    .checked_mul(2)
                    .and_then(|sides| value.checked_add(sides))
            })
            .ok_or(FlexMeshError::Count { field: "nface" }),
        3 => Ok(shell_num),
        _ => Err(FlexMeshError::UnsupportedDimension { dim }),
    }
}

/// 生成面点与连续索引（有效绕序）。
fn build_faces(
    input: &FlexMeshInput<'_>,
    vertices: &[[f32; 3]],
) -> Result<(Vec<[f32; 3]>, Vec<u32>), FlexMeshError> {
    let elem_num = count("nflexelem", input.elem_num)?;
    let shell_num = count("nflexshell", input.shell_num)?;
    let vert_num = count("nflexvert", input.vert_num)?;
    if !input.radius.is_finite() {
        return Err(FlexMeshError::NonFinite {
            field: "radius",
            index: 0,
        });
    }
    let nface = face_count(input.dim, elem_num, shell_num)?;
    let point_count = nface
        .checked_mul(3)
        .ok_or(FlexMeshError::Count { field: "points" })?;
    let mut points = Vec::with_capacity(point_count);
    if input.dim == 2 {
        let normals = vertex_normals_2d(input, vertices, elem_num, vert_num)?;
        emit_2d_elements(input, vertices, &normals, elem_num, vert_num, &mut points)?;
        emit_2d_sides(input, vertices, &normals, shell_num, vert_num, &mut points)?;
    } else {
        emit_3d_shells(input, vertices, shell_num, vert_num, &mut points)?;
    }
    let index_count = u32::try_from(points.len()).map_err(|_| FlexMeshError::Count {
        field: "flex_mesh_indices",
    })?;
    Ok((points, (0..index_count).collect()))
}

/// 2D 元素的两个面（上游 `_build_flex_2d_elements`）。
fn emit_2d_elements(
    input: &FlexMeshInput<'_>,
    vertices: &[[f32; 3]],
    normals: &[[f32; 3]],
    elem_num: usize,
    vert_num: usize,
    points: &mut Vec<[f32; 3]>,
) -> Result<(), FlexMeshError> {
    let elem_adr = address("flex_elemdataadr", input.elem_data_adr)?;
    for element in 0..elem_num {
        let base = elem_adr
            .checked_add(element * 3)
            .ok_or(FlexMeshError::Count { field: "flex_elem" })?;
        let v0 = vertex(input.elem, base, vertices, input.vert_adr, vert_num)?;
        let v1 = vertex(input.elem, base + 1, vertices, input.vert_adr, vert_num)?;
        let v2 = vertex(input.elem, base + 2, vertices, input.vert_adr, vert_num)?;
        let n0 = normals[local(input.elem, base, vert_num)?];
        let n1 = normals[local(input.elem, base + 1, vert_num)?];
        let n2 = normals[local(input.elem, base + 2, vert_num)?];
        let radius = input.radius;
        // top：p_pos（+r*n），绕序 (0,1,2)；bottom：p_neg（-r*n），绕序 (0,2,1)。
        points.push(offset(v0, n0, radius));
        points.push(offset(v1, n1, radius));
        points.push(offset(v2, n2, radius));
        points.push(offset(v0, n0, -radius));
        points.push(offset(v2, n2, -radius));
        points.push(offset(v1, n1, -radius));
    }
    Ok(())
}

/// 2D shell 侧面的两个三角（上游 `_build_flex_2d_sides`）。
fn emit_2d_sides(
    input: &FlexMeshInput<'_>,
    vertices: &[[f32; 3]],
    normals: &[[f32; 3]],
    shell_num: usize,
    vert_num: usize,
    points: &mut Vec<[f32; 3]>,
) -> Result<(), FlexMeshError> {
    let shell_adr = address("flex_shelldataadr", input.shell_data_adr)?;
    for fragment in 0..shell_num {
        let base = shell_adr
            .checked_add(fragment * 2)
            .ok_or(FlexMeshError::Count {
                field: "flex_shell",
            })?;
        let v0 = vertex(input.shell, base, vertices, input.vert_adr, vert_num)?;
        let v1 = vertex(input.shell, base + 1, vertices, input.vert_adr, vert_num)?;
        let n0 = normals[local(input.shell, base, vert_num)?];
        let n1 = normals[local(input.shell, base + 1, vert_num)?];
        let radius = input.radius;
        // face 0：(v0+r*n0, v1-r*n1, v1+r*n1)；face 1：(v1-r*n1, v0-r*n0, v0+r*n0)。
        points.push(offset(v0, n0, radius));
        points.push(offset(v1, n1, -radius));
        points.push(offset(v1, n1, radius));
        points.push(offset(v1, n1, -radius));
        points.push(offset(v0, n0, -radius));
        points.push(offset(v0, n0, radius));
    }
    Ok(())
}

/// 3D shell 三角（上游 `_build_flex_3d_shells`，顶点直接取世界位置）。
fn emit_3d_shells(
    input: &FlexMeshInput<'_>,
    vertices: &[[f32; 3]],
    shell_num: usize,
    vert_num: usize,
    points: &mut Vec<[f32; 3]>,
) -> Result<(), FlexMeshError> {
    let shell_adr = address("flex_shelldataadr", input.shell_data_adr)?;
    for fragment in 0..shell_num {
        let base = shell_adr
            .checked_add(fragment * 3)
            .ok_or(FlexMeshError::Count {
                field: "flex_shell",
            })?;
        for corner in 0..3 {
            points.push(vertex(
                input.shell,
                base + corner,
                vertices,
                input.vert_adr,
                vert_num,
            )?);
        }
    }
    Ok(())
}

/// 2D 顶点法线：元素面法线按下标升序累加后逐顶点归一化（上游 L626-678 的确定性版本）。
fn vertex_normals_2d(
    input: &FlexMeshInput<'_>,
    vertices: &[[f32; 3]],
    elem_num: usize,
    vert_num: usize,
) -> Result<Vec<[f32; 3]>, FlexMeshError> {
    let mut normals = vec![[0.0f32; 3]; vert_num];
    let elem_adr = address("flex_elemdataadr", input.elem_data_adr)?;
    for element in 0..elem_num {
        let base = elem_adr
            .checked_add(element * 3)
            .ok_or(FlexMeshError::Count { field: "flex_elem" })?;
        let v0 = vertex(input.elem, base, vertices, input.vert_adr, vert_num)?;
        let v1 = vertex(input.elem, base + 1, vertices, input.vert_adr, vert_num)?;
        let v2 = vertex(input.elem, base + 2, vertices, input.vert_adr, vert_num)?;
        let face = normalize(cross(sub(v1, v0), sub(v2, v0)));
        for corner in 0..3 {
            let index = local(input.elem, base + corner, vert_num)?;
            for axis in 0..3 {
                normals[index][axis] += face[axis];
            }
        }
    }
    for normal in &mut normals {
        *normal = normalize(*normal);
    }
    Ok(normals)
}

/// 负下标检查（拓扑/计数都按 `i32` 输入）。
fn count(field: &'static str, value: i32) -> Result<usize, FlexMeshError> {
    usize::try_from(value).map_err(|_| FlexMeshError::Count { field })
}

/// 数据起始下标检查。
fn address(field: &'static str, value: i32) -> Result<usize, FlexMeshError> {
    usize::try_from(value).map_err(|_| FlexMeshError::Count { field })
}

/// 取某个拓扑项的局部顶点号并检查上界。
fn local(indices: &[i32], slot: usize, vert_num: usize) -> Result<usize, FlexMeshError> {
    let index = *indices.get(slot).ok_or(FlexMeshError::IndexOutOfRange {
        field: "flex_topology",
        index: slot,
    })?;
    let index = usize::try_from(index).map_err(|_| FlexMeshError::IndexOutOfRange {
        field: "flex_topology",
        index: slot,
    })?;
    if index >= vert_num {
        return Err(FlexMeshError::IndexOutOfRange {
            field: "flex_topology",
            index: slot,
        });
    }
    Ok(index)
}

/// 局部顶点号 → 世界顶点（`vert_adr + 局部号`）。
fn vertex(
    indices: &[i32],
    slot: usize,
    vertices: &[[f32; 3]],
    vert_adr: i32,
    vert_num: usize,
) -> Result<[f32; 3], FlexMeshError> {
    let index = local(indices, slot, vert_num)?;
    let address = address("flex_vertadr", vert_adr)?
        .checked_add(index)
        .ok_or(FlexMeshError::Count {
            field: "flex_vertadr",
        })?;
    let value = *vertices
        .get(address)
        .ok_or(FlexMeshError::IndexOutOfRange {
            field: "flex_vertices",
            index: address,
        })?;
    if !value.iter().all(|component| component.is_finite()) {
        return Err(FlexMeshError::NonFinite {
            field: "flex_vertices",
            index: address,
        });
    }
    Ok(value)
}

/// 顶点沿法线偏移 `radius`（上游 `v + radius * n` 的逐分量写法）。
fn offset(vertex: [f32; 3], normal: [f32; 3], radius: f32) -> [f32; 3] {
    [0, 1, 2].map(|axis| vertex[axis] + radius * normal[axis])
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Warp `wp.normalize`：`sqrt(x*x + y*y + z*z)`，长度非正时给零向量。
fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if length > 0.0 {
        [value[0] / length, value[1] / length, value[2] / length]
    } else {
        [0.0; 3]
    }
}

#[cfg(test)]
#[path = "flex_mesh/tests.rs"]
mod tests;
