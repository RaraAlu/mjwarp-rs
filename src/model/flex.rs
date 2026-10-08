//! 检查柔体位置与可选边面字段。
//! 共享拓扑与局部坐标。

use super::{InertialModelInput, TendonModelInput};
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
impl FlexPositionModelInput {
    pub fn new(tendons: TendonModelInput, fields: FlexPositionFields) -> Result<Self, InputError> {
        let nb = tendons
            .spatial()
            .fixed()
            .camlight()
            .mocap()
            .attached()
            .rigid()
            .kinematics()
            .nbody();
        fields.validate(nb)?;
        Ok(Self {
            tendons,
            fields,
            edges: None,
            faces: None,
            hessian: None,
        })
    }
    /// 检查后启用边计算。
    /// 空行保留冻结上游语义。
    pub fn with_edges(mut self, fields: FlexEdgeFields) -> Result<Self, InputError> {
        let rigid = self
            .tendons
            .spatial()
            .fixed()
            .camlight()
            .mocap()
            .attached()
            .rigid();
        fields.validate(&self.fields, rigid)?;
        if let Some(hessian) = &self.hessian {
            hessian.validate(&self.fields, &fields)?;
        }
        self.edges = Some(fields);
        Ok(self)
    }
    pub fn edges(&self) -> Option<&FlexEdgeFields> {
        self.edges.as_ref()
    }
    /// 检查完整线性壳体面映射。
    pub fn with_faces(mut self, fields: FlexFaceFields) -> Result<Self, InputError> {
        fields.validate(&self.fields)?;
        self.faces = Some(fields);
        Ok(self)
    }
    pub fn faces(&self) -> Option<&FlexFaceFields> {
        self.faces.as_ref()
    }
    /// 启用21系数拉伸矩阵。
    /// 调用方先启用边计算。
    pub fn with_hessian(mut self, fields: FlexHessianFields) -> Result<Self, InputError> {
        let edges = self.edges.as_ref().ok_or(InputError::InvalidDimension {
            field: "flex_hessian_requires_edges",
        })?;
        fields.validate(&self.fields, edges)?;
        self.hessian = Some(fields);
        Ok(self)
    }
    pub fn hessian(&self) -> Option<&FlexHessianFields> {
        self.hessian.as_ref()
    }
    pub fn tendons(&self) -> &TendonModelInput {
        &self.tendons
    }
    pub fn fields(&self) -> &FlexPositionFields {
        &self.fields
    }
    pub fn nflex(&self) -> usize {
        self.fields.flex_interp.len()
    }
    pub fn nflexnode(&self) -> usize {
        self.fields.flex_nodebodyid.len()
    }
    pub fn nflexvert(&self) -> usize {
        self.fields.flex_vertbodyid.len()
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        TendonModelInput,
        FlexPositionFields,
        Option<FlexEdgeFields>,
        Option<FlexFaceFields>,
        Option<FlexHessianFields>,
    ) {
        (
            self.tendons,
            self.fields,
            self.edges,
            self.faces,
            self.hessian,
        )
    }
}

/// 冻结上游的完整面映射。
/// 每面保留九项全局节点号。
/// 线性面末五项使用负一。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlexFaceFields {
    pub flex_face_map: Vec<i32>,
    pub flex_face: Vec<i32>,
}
impl FlexFaceFields {
    pub fn nflexface(&self) -> usize {
        self.flex_face_map.len() / 2
    }
    fn validate(&self, positions: &FlexPositionFields) -> Result<(), InputError> {
        let overflow = || InputError::Overflow {
            field: "flex_face_fields",
        };
        let mut expected = 0usize;
        for f in 0..positions.flex_interp.len() {
            if positions.flex_interp[f] == -1 {
                expected = expected
                    .checked_add(face_count(&positions.flex_cellnum[3 * f..3 * f + 3])?)
                    .ok_or_else(overflow)?;
            }
        }
        expected
            .checked_mul(10)
            .and_then(|n| n.checked_add(2))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        expected
            .checked_mul(31)
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        for (field, actual, required) in [
            ("flex_face_map", self.flex_face_map.len(), 2 * expected),
            ("flex_face", self.flex_face.len(), 9 * expected),
        ] {
            if actual != required {
                return Err(InputError::LengthMismatch {
                    field,
                    expected: required,
                    actual,
                });
            }
        }
        let mut face = 0;
        for f in 0..positions.flex_interp.len() {
            if positions.flex_interp[f] != -1 {
                continue;
            }
            let cells = &positions.flex_cellnum[3 * f..3 * f + 3];
            for local in 0..face_count(cells)? {
                if self.flex_face_map[2 * face..2 * face + 2] != [f as i32, local as i32] {
                    return Err(InputError::InvalidTopology {
                        field: "flex_face_map",
                        index: face,
                        reason: "noncanonical_face_mapping",
                    });
                }
                let (_, nodes) = face_nodes(cells, local);
                for (slot, expected) in nodes
                    .into_iter()
                    .map(|n| positions.flex_nodeadr[f] + n)
                    .chain([-1; 5])
                    .enumerate()
                {
                    if self.flex_face[9 * face + slot] != expected {
                        return Err(InputError::InvalidTopology {
                            field: "flex_face",
                            index: 9 * face + slot,
                            reason: "noncanonical_face_nodes",
                        });
                    }
                }
                face += 1;
            }
        }
        Ok(())
    }
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
impl FlexEdgeFields {
    pub fn nflexedge(&self) -> usize {
        self.flexedge_j_rownnz.len()
    }
    pub fn nnz(&self) -> usize {
        self.flexedge_j_colind.len()
    }
    fn validate(
        &self,
        f: &FlexPositionFields,
        rigid: &InertialModelInput,
    ) -> Result<(), InputError> {
        let nf = f.flex_interp.len();
        let ne = self.nflexedge();
        let k = rigid.kinematics();
        let overflow = || InputError::Overflow {
            field: "flex_edge_fields",
        };
        // Bounds also cover the GPU descriptor and output strides.
        ne.checked_mul(4)
            .and_then(|n| n.checked_add(self.nnz()))
            .and_then(|n| n.checked_add(f.flex_vertbodyid.len()))
            .and_then(|n| k.nbody().checked_mul(2).and_then(|b| n.checked_add(b)))
            .and_then(|n| n.checked_add(k.nv()))
            .and_then(|n| n.checked_add(7))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        ne.checked_mul(2)
            .and_then(|n| n.checked_add(self.nnz()))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        for (field, actual, expected) in [
            ("flex_edgeadr", self.flex_edgeadr.len(), nf),
            ("flex_edgenum", self.flex_edgenum.len(), nf),
            ("flex_edge", self.flex_edge.len(), 2 * ne),
            ("flexedge_J_rowadr", self.flexedge_j_rowadr.len(), ne),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        let invalid = |field, index, reason| InputError::InvalidTopology {
            field,
            index,
            reason,
        };
        if let Some(index) = self
            .flexedge_j_colind
            .iter()
            .position(|&d| d < 0 || d as usize >= k.nv())
        {
            return Err(invalid(
                "flexedge_J_colind",
                index,
                "dof_reference_out_of_range",
            ));
        }
        let (mut edge, mut slot) = (0usize, 0usize);
        let parents = &k.fields().body_parentid;
        let dofbody = &rigid.fields().dof_bodyid;
        for flex in 0..nf {
            let count = self.flex_edgenum[flex];
            if count < 0
                || count as usize > ne - edge
                || self.flex_edgeadr[flex] < 0
                || self.flex_edgeadr[flex] as usize != edge
            {
                return Err(invalid(
                    "flex_edgeadr",
                    flex,
                    "invalid_contiguous_edge_range",
                ));
            }
            for e in edge..edge + count as usize {
                let mut body = [0; 2];
                for (end, b) in body.iter_mut().enumerate() {
                    let v = self.flex_edge[2 * e + end];
                    if v < 0 || v >= f.flex_vertnum[flex] {
                        return Err(invalid(
                            "flex_edge",
                            2 * e + end,
                            "vertex_reference_out_of_range",
                        ));
                    }
                    *b = f.flex_vertbodyid[f.flex_vertadr[flex] as usize + v as usize];
                }
                let adr = self.flexedge_j_rowadr[e];
                let nnz = self.flexedge_j_rownnz[e];
                if nnz < 0
                    || adr < 0
                    || adr as usize > self.nnz()
                    || nnz as usize > self.nnz() - adr as usize
                    || (nnz > 0 && adr as usize != slot)
                {
                    return Err(invalid("flexedge_J_rowadr", e, "invalid_sparse_range"));
                }
                if nnz == 0 {
                    continue;
                }
                if body.iter().any(|&b| b < 0) {
                    return Err(invalid(
                        "flexedge_J_rownnz",
                        e,
                        "interpolated_vertex_has_sparse_row",
                    ));
                }
                let cols = &self.flexedge_j_colind[slot..slot + nnz as usize];
                if cols.iter().any(|&c| c < 0 || c as usize >= k.nv())
                    || cols.windows(2).any(|p| p[0] >= p[1])
                {
                    return Err(invalid(
                        "flexedge_J_colind",
                        slot,
                        "invalid_sorted_dof_columns",
                    ));
                }
                let affects = |mut b: i32, d: i32| {
                    while b > 0 {
                        if b == d {
                            return true;
                        }
                        b = parents[b as usize];
                    }
                    false
                };
                let mut i = 0;
                for (d, &owner) in dofbody.iter().enumerate() {
                    if body.iter().any(|&b| affects(b, owner)) {
                        if cols.get(i) != Some(&(d as i32)) {
                            return Err(invalid(
                                "flexedge_J_colind",
                                slot,
                                "endpoint_dof_union_mismatch",
                            ));
                        }
                        i += 1;
                    }
                }
                if i != cols.len() {
                    return Err(invalid(
                        "flexedge_J_colind",
                        slot,
                        "endpoint_dof_union_mismatch",
                    ));
                }
                slot += nnz as usize;
            }
            edge += count as usize;
        }
        // Native nJfe can reserve unused tail slots when passive forces are off.
        // The GPU clears the whole pool before writing the checked active rows.
        if edge != ne {
            return Err(invalid("flex_edge_ranges", nf, "unclaimed_edges"));
        }
        Ok(())
    }
}

impl FlexPositionFields {
    pub(crate) fn validate(&self, nb: usize) -> Result<(), InputError> {
        let nf = self.flex_interp.len();
        let nn = self.flex_nodebodyid.len();
        let nv = self.flex_vertbodyid.len();
        let overflow = || InputError::Overflow {
            field: "flex_position_fields",
        };
        nf.checked_mul(9)
            .and_then(|n| n.checked_add(5))
            .and_then(|n| n.checked_add(nn))
            .and_then(|n| n.checked_add(nv))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        nv.checked_mul(6)
            .and_then(|n| nn.checked_mul(3).and_then(|m| n.checked_add(m)))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        for (field, actual, expected) in [
            ("flex_cellnum", self.flex_cellnum.len(), 3 * nf),
            ("flex_nodeadr", self.flex_nodeadr.len(), nf),
            ("flex_nodenum", self.flex_nodenum.len(), nf),
            ("flex_vertadr", self.flex_vertadr.len(), nf),
            ("flex_vertnum", self.flex_vertnum.len(), nf),
            ("flex_centered", self.flex_centered.len(), nf),
            ("flex_node", self.flex_node.len(), 3 * nn),
            ("flex_vert", self.flex_vert.len(), 3 * nv),
            ("flex_vert0", self.flex_vert0.len(), 3 * nv),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        for (field, values) in [
            ("flex_node", &self.flex_node),
            ("flex_vert", &self.flex_vert),
            ("flex_vert0", &self.flex_vert0),
        ] {
            if let Some(index) = values.iter().position(|x| !x.is_finite()) {
                return Err(InputError::NonFinite { field, index });
            }
        }
        let invalid = |field, index, reason| InputError::InvalidTopology {
            field,
            index,
            reason,
        };
        for (index, &id) in self.flex_nodebodyid.iter().enumerate() {
            if id < 0 || id as usize >= nb {
                return Err(invalid(
                    "flex_nodebodyid",
                    index,
                    "body_reference_out_of_range",
                ));
            }
        }
        let (mut node, mut vert) = (0usize, 0usize);
        for f in 0..nf {
            let mode = self.flex_interp[f];
            if !matches!(mode, -1..=1) {
                return Err(invalid("flex_interp", f, "unsupported_flex_interpolation"));
            }
            let vn = self.flex_vertnum[f];
            if vn <= 0
                || self.flex_vertadr[f] < 0
                || self.flex_vertadr[f] as usize != vert
                || vn as usize > nv - vert
            {
                return Err(invalid(
                    "flex_vertadr",
                    f,
                    "invalid_contiguous_vertex_range",
                ));
            }
            let count = self.flex_nodenum[f];
            let adr = self.flex_nodeadr[f];
            if count < 0
                || count as usize > nn - node
                || (count > 0 && (adr < 0 || adr as usize != node))
                || (count == 0 && adr != -1 && (adr < 0 || adr as usize != node))
            {
                return Err(invalid("flex_nodeadr", f, "invalid_contiguous_node_range"));
            }
            if mode == 0 && count != 0 {
                return Err(invalid("flex_nodenum", f, "direct_flex_has_nodes"));
            }
            if mode != 0 {
                let mut grid = 1usize;
                for &c in &self.flex_cellnum[3 * f..3 * f + 3] {
                    if c <= 0 {
                        return Err(invalid("flex_cellnum", f, "nonpositive_cell_count"));
                    }
                    grid = grid
                        .checked_mul(c as usize + 1)
                        .filter(|&n| n <= i32::MAX as usize)
                        .ok_or_else(overflow)?;
                }
                if grid != count as usize {
                    return Err(invalid("flex_nodenum", f, "linear_node_grid_mismatch"));
                }
            }
            for v in vert..vert + vn as usize {
                let body = self.flex_vertbodyid[v];
                if body < (if mode == 0 { 0 } else { -1 }) || (body >= 0 && body as usize >= nb) {
                    return Err(invalid("flex_vertbodyid", v, "body_reference_out_of_range"));
                }
                if mode != 0 {
                    // Native compilation can leave tiny out-of-cube values.
                    // Preserve upstream clamping, but forbid undefined int casts.
                    for k in 0..3 {
                        let scaled =
                            self.flex_vert0[3 * v + k] * self.flex_cellnum[3 * f + k] as f32;
                        if !scaled.is_finite() || scaled < i32::MIN as f32 || scaled >= 2147483648.0
                        {
                            return Err(invalid(
                                "flex_vert0",
                                v,
                                "cell_lookup_conversion_out_of_range",
                            ));
                        }
                    }
                }
                if mode == 0
                    && self.flex_centered[f]
                    && self.flex_vert[3 * v..3 * v + 3].iter().any(|&x| x != 0.0)
                {
                    return Err(invalid(
                        "flex_centered",
                        f,
                        "centered_vertex_has_local_offset",
                    ));
                }
            }
            if self.flex_centered[f]
                && self.flex_node[3 * node..3 * (node + count as usize)]
                    .iter()
                    .any(|&x| x != 0.0)
            {
                return Err(invalid(
                    "flex_centered",
                    f,
                    "centered_node_has_local_offset",
                ));
            }
            vert += vn as usize;
            node += count as usize;
        }
        if vert != nv || node != nn {
            return Err(invalid(
                "flex_position_ranges",
                nf,
                "unclaimed_flex_positions",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields() -> FlexPositionFields {
        FlexPositionFields {
            flex_interp: vec![0, 1],
            flex_cellnum: vec![1; 6],
            flex_nodeadr: vec![-1, 0],
            flex_nodenum: vec![0, 8],
            flex_vertadr: vec![0, 1],
            flex_vertnum: vec![1, 1],
            flex_centered: vec![true, true],
            flex_nodebodyid: vec![0; 8],
            flex_vertbodyid: vec![0, -1],
            flex_node: vec![0.0; 24],
            flex_vert: vec![0.0; 6],
            flex_vert0: vec![0.5; 6],
        }
    }
    #[test]
    fn accepts_direct_linear_centered_and_empty_fields() {
        let mut f = fields();
        f.validate(1).unwrap();
        f.flex_centered.fill(false);
        f.flex_node[0] = -0.0;
        f.flex_vert[0] = 0.2;
        f.validate(1).unwrap();
        assert_eq!(f.flex_node[0].to_bits(), (-0.0f32).to_bits());
        FlexPositionFields::default().validate(1).unwrap();
    }
    #[test]
    fn rejects_every_position_field_length_and_nonfinite_value() {
        for i in 0..9 {
            let mut f = fields();
            match i {
                0 => {
                    f.flex_cellnum.pop();
                }
                1 => {
                    f.flex_nodeadr.pop();
                }
                2 => {
                    f.flex_nodenum.pop();
                }
                3 => {
                    f.flex_vertadr.pop();
                }
                4 => {
                    f.flex_vertnum.pop();
                }
                5 => {
                    f.flex_centered.pop();
                }
                6 => {
                    f.flex_node.pop();
                }
                7 => {
                    f.flex_vert.pop();
                }
                _ => {
                    f.flex_vert0.pop();
                }
            }
            assert!(
                matches!(f.validate(1), Err(InputError::LengthMismatch { .. })),
                "field {i}"
            );
        }
        for i in 0..3 {
            for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut f = fields();
                let v = match i {
                    0 => &mut f.flex_node,
                    1 => &mut f.flex_vert,
                    _ => &mut f.flex_vert0,
                };
                let last = v.len() - 1;
                v[last] = bad;
                assert!(
                    matches!(f.validate(1),Err(InputError::NonFinite{index,..}) if index==last)
                );
            }
        }
    }
    #[test]
    fn rejects_unsafe_ranges_bodies_and_centered_offsets() {
        for i in 0..13 {
            let mut f = fields();
            match i {
                0 => f.flex_vertadr[1] = 0,
                1 => f.flex_vertadr[1] = 2,
                2 => f.flex_vertnum[0] = -1,
                3 => f.flex_vertnum[1] = i32::MAX,
                4 => f.flex_nodeadr[1] = -1,
                5 => f.flex_nodenum[1] = -1,
                6 => f.flex_nodenum[1] = 9,
                7 => f.flex_nodebodyid[7] = 1,
                8 => f.flex_nodebodyid[0] = -1,
                9 => f.flex_vertbodyid[0] = -1,
                10 => f.flex_vertbodyid[1] = -2,
                11 => f.flex_vert[0] = 1.0,
                _ => f.flex_node[23] = 1.0,
            }
            assert!(
                matches!(f.validate(1), Err(InputError::InvalidTopology { .. })),
                "case {i}"
            );
        }
        let mut f = fields();
        f.flex_interp[1] = 0;
        assert!(matches!(
            f.validate(1),
            Err(InputError::InvalidTopology {
                field: "flex_nodenum",
                ..
            })
        ));
        let mut f = fields();
        f.flex_vertnum[1] = 0;
        assert!(f.validate(1).is_err());
        let mut f = fields();
        f.flex_interp.clear();
        assert!(f.validate(1).is_err());
        let mut f = fields();
        f.flex_nodebodyid.push(0);
        f.flex_node.extend([0.0; 3]);
        assert!(f.validate(1).is_err());
        let mut f = fields();
        f.flex_vertbodyid.push(0);
        f.flex_vert.extend([0.0; 3]);
        f.flex_vert0.extend([0.0; 3]);
        assert!(f.validate(1).is_err());
    }
    #[test]
    fn rejects_high_order_and_unsafe_linear_cells() {
        for mode in [-2, 2, i32::MAX] {
            let mut f = fields();
            f.flex_interp[1] = mode;
            assert!(f.validate(1).is_err());
        }
        for cell in [-1, 0, 2, i32::MAX] {
            let mut f = fields();
            f.flex_cellnum[3] = cell;
            assert!(f.validate(1).is_err());
        }
        for coord in [f32::MIN, f32::MAX, 2147483648.0] {
            let mut f = fields();
            f.flex_vert0[5] = coord;
            assert!(f.validate(1).is_err());
        }
        for coord in [-0.1, 0.0, 1.0, 1.1] {
            let mut f = fields();
            f.flex_vert0[5] = coord;
            f.validate(1).unwrap();
        }
    }
    #[test]
    fn accepts_linear_shell_and_checks_canonical_faces() {
        let mut p = fields();
        p.flex_interp[1] = -1;
        p.validate(1).unwrap();
        let mut f = FlexFaceFields::default();
        for local in 0..6 {
            f.flex_face_map.extend([1, local]);
            f.flex_face.extend(face_nodes(&[1, 1, 1], local as usize).1);
            f.flex_face.extend([-1; 5]);
        }
        f.validate(&p).unwrap();
        for i in 0..f.flex_face.len() {
            let mut bad = f.clone();
            bad.flex_face[i] += 1;
            assert!(bad.validate(&p).is_err());
        }
        for i in 0..f.flex_face_map.len() {
            let mut bad = f.clone();
            bad.flex_face_map[i] += 1;
            assert!(bad.validate(&p).is_err());
        }
        FlexFaceFields::default()
            .validate(&FlexPositionFields::default())
            .unwrap();
        assert!(FlexFaceFields::default().validate(&p).is_err());
    }
}
