//! 检查柔体位置与可选边面字段。
//! 共享拓扑与局部坐标。
mod edge;
mod face;
mod input;
mod position;
#[cfg(test)]
mod tests;

use super::TendonModelInput;
use crate::diagnostics::InputError;
#[path = "flex_hessian.rs"]
mod hessian;
pub use hessian::FlexHessianFields;

/// 原生柔体位置字段。
/// 仅支持直接与线性插值。
/// 线性壳体沿用冻结节点语义。
/// 本辅助不重建内部节点。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlexPositionFields {
    pub flex_interp: Vec<i32>,
    pub flex_cellnum: Vec<i32>,
    pub flex_nodeadr: Vec<i32>,
    pub flex_nodenum: Vec<i32>,
    pub flex_vertadr: Vec<i32>,
    pub flex_vertnum: Vec<i32>,
    pub flex_centered: Vec<bool>,
    pub flex_nodebodyid: Vec<i32>,
    pub flex_vertbodyid: Vec<i32>,
    pub flex_node: Vec<f32>,
    pub flex_vert: Vec<f32>,
    pub flex_vert0: Vec<f32>,
}

/// 接收已编译模型的位置子集。
/// 可选边面字段不提供弹性。
/// 休眠需要独立可选包装。
///
/// ```compile_fail
/// fn mutate(m: &mut mjwarp_rs::model::FlexPositionModelInput) {
///     m.fields().flex_interp[0] = 2;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FlexPositionModelInput {
    tendons: TendonModelInput,
    fields: FlexPositionFields,
    edges: Option<FlexEdgeFields>,
    faces: Option<FlexFaceFields>,
    hessian: Option<FlexHessianFields>,
}

/// 冻结上游的完整面映射。
/// 每面保留九项全局节点号。
/// 线性面末五项使用负一。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlexFaceFields {
    pub flex_face_map: Vec<i32>,
    pub flex_face: Vec<i32>,
}

/// 共享边拓扑与原生稀疏行。
/// 边端点采用柔体局部编号。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlexEdgeFields {
    pub flex_edgeadr: Vec<i32>,
    pub flex_edgenum: Vec<i32>,
    pub flex_edge: Vec<i32>,
    pub flexedge_j_rowadr: Vec<i32>,
    pub flexedge_j_rownnz: Vec<i32>,
    pub flexedge_j_colind: Vec<i32>,
}

fn face_count(cells: &[i32]) -> Result<usize, InputError> {
    let [x, y, z] = [cells[0] as usize, cells[1] as usize, cells[2] as usize];
    x.checked_mul(y)
        .and_then(|n| x.checked_mul(z).and_then(|a| n.checked_add(a)))
        .and_then(|n| y.checked_mul(z).and_then(|a| n.checked_add(a)))
        .and_then(|n| n.checked_mul(2))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "flex_face_fields",
        })
}
// Frozen support.py: get_face_metadata and gather_face_node_index, order one.
// Checked positive cells and node-grid bounds protect all integer arithmetic.
pub(crate) fn face_nodes(cells: &[i32], local: usize) -> (i32, [i32; 4]) {
    let [x, y, z] = [cells[0], cells[1], cells[2]];
    let mut within = local as i32;
    let mut side = 0;
    for size in [y * z, y * z, x * z, x * z, x * y, x * y] {
        if within < size {
            break;
        }
        within -= size;
        side += 1;
    }
    let axis = side / 2;
    let c1 = [z, x, y][axis as usize];
    let fixed = (side % 2) * [x, y, z][axis as usize];
    let (q0, q1) = (within / c1, within % c1);
    let nodes = std::array::from_fn(|slot| {
        let (a, b) = (q0 + slot as i32 / 2, q1 + slot as i32 % 2);
        let [i, j, k] = match axis {
            0 => [fixed, a, b],
            1 => [b, fixed, a],
            _ => [a, b, fixed],
        };
        (i * (y + 1) + j) * (z + 1) + k
    });
    (axis, nodes)
}
