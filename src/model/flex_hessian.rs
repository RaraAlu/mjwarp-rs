//! 检查21系数拉伸矩阵输入。
use super::{FlexEdgeFields, FlexPositionFields};
use crate::diagnostics::InputError;

/// 单元与系数采用原生地址。
/// 三角形也保留21系数跨度。
/// 本辅助不接受24系数布局。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlexHessianFields {
    pub flex_dim: Vec<i32>,
    pub flex_rigid: Vec<bool>,
    pub flex_elemadr: Vec<i32>,
    pub flex_elemnum: Vec<i32>,
    pub flex_elemdataadr: Vec<i32>,
    pub flex_elemedgeadr: Vec<i32>,
    pub flex_stiffnessadr: Vec<i32>,
    pub flex_elem: Vec<i32>,
    pub flex_elemedge: Vec<i32>,
    pub flexedge_length0: Vec<f32>,
    pub flex_stiffness: Vec<f32>,
}

pub(crate) fn element_edges(dim: i32) -> &'static [[usize; 2]] {
    match dim {
        1 => &[[0, 1]],
        2 => &[[1, 2], [2, 0], [0, 1]],
        _ => &[[0, 1], [1, 2], [2, 0], [2, 3], [0, 3], [1, 3]],
    }
}

impl FlexHessianFields {
    #[cfg(feature = "cuda-probe")]
    pub(crate) fn active(&self, positions: &FlexPositionFields, f: usize) -> bool {
        positions.flex_interp[f] == 0
            && !self.flex_rigid[f]
            && self.flex_dim[f] >= 2
            && self.flex_stiffnessadr[f] >= 0
            && self.flex_stiffness[self.flex_stiffnessadr[f] as usize] != 0.0
    }
    pub(super) fn validate(
        &self,
        p: &FlexPositionFields,
        e: &FlexEdgeFields,
    ) -> Result<(), InputError> {
        let nf = p.flex_interp.len();
        let overflow = || InputError::Overflow {
            field: "flex_hessian_fields",
        };
        let invalid = |field, index, reason| InputError::InvalidTopology {
            field,
            index,
            reason,
        };
        nf.checked_mul(11)
            .and_then(|n| n.checked_add(9))
            .and_then(|n| n.checked_add(self.flex_elem.len()))
            .and_then(|n| n.checked_add(self.flex_elemedge.len()))
            .and_then(|n| n.checked_add(e.flex_edge.len()))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        self.flexedge_length0
            .len()
            .checked_add(self.flex_stiffness.len())
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        for (field, actual, expected) in [
            ("flex_dim", self.flex_dim.len(), nf),
            ("flex_rigid", self.flex_rigid.len(), nf),
            ("flex_elemadr", self.flex_elemadr.len(), nf),
            ("flex_elemnum", self.flex_elemnum.len(), nf),
            ("flex_elemdataadr", self.flex_elemdataadr.len(), nf),
            ("flex_elemedgeadr", self.flex_elemedgeadr.len(), nf),
            ("flex_stiffnessadr", self.flex_stiffnessadr.len(), nf),
            (
                "flexedge_length0",
                self.flexedge_length0.len(),
                e.nflexedge(),
            ),
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
            ("flexedge_length0", &self.flexedge_length0),
            ("flex_stiffness", &self.flex_stiffness),
        ] {
            if let Some(index) = values.iter().position(|v| !v.is_finite()) {
                return Err(InputError::NonFinite { field, index });
            }
        }
        if let Some(index) = self.flexedge_length0.iter().position(|&v| v < 0.0) {
            return Err(InputError::NegativeValue {
                field: "flexedge_length0",
                index,
            });
        }
        let (mut elem, mut data, mut ee, mut stiffness) = (0usize, 0usize, 0usize, 0usize);
        for f in 0..nf {
            let dim = self.flex_dim[f];
            if !(1..=3).contains(&dim) {
                return Err(invalid("flex_dim", f, "unsupported_dimension"));
            }
            let count = self.flex_elemnum[f];
            if count < 0
                || self.flex_elemadr[f] < 0
                || self.flex_elemadr[f] as usize != elem
                || self.flex_elemdataadr[f] < 0
                || self.flex_elemdataadr[f] as usize != data
                || self.flex_elemedgeadr[f] < 0
                || self.flex_elemedgeadr[f] as usize != ee
            {
                return Err(invalid("flex_elem_ranges", f, "noncanonical_element_range"));
            }
            let count = count as usize;
            let vwidth = dim as usize + 1;
            let pattern = element_edges(dim);
            let ndata = count.checked_mul(vwidth).ok_or_else(overflow)?;
            let nee = count.checked_mul(pattern.len()).ok_or_else(overflow)?;
            let end_data = data.checked_add(ndata).ok_or_else(overflow)?;
            let end_ee = ee.checked_add(nee).ok_or_else(overflow)?;
            if end_data > self.flex_elem.len() || end_ee > self.flex_elemedge.len() {
                return Err(invalid(
                    "flex_elem_ranges",
                    f,
                    "element_range_out_of_bounds",
                ));
            }
            let sad = self.flex_stiffnessadr[f];
            if sad < -1 {
                return Err(invalid("flex_stiffnessadr", f, "invalid_stiffness_address"));
            }
            if sad >= 0 {
                let width = count.checked_mul(21).ok_or_else(overflow)?;
                if sad as usize != stiffness {
                    return Err(invalid(
                        "flex_stiffnessadr",
                        f,
                        "noncanonical_21_coefficient_range",
                    ));
                }
                stiffness = stiffness.checked_add(width).ok_or_else(overflow)?;
                if stiffness > self.flex_stiffness.len()
                    || (count == 0 && sad as usize == self.flex_stiffness.len())
                {
                    return Err(invalid(
                        "flex_stiffnessadr",
                        f,
                        "stiffness_range_out_of_bounds",
                    ));
                }
            }
            let start = p.flex_vertadr[f] as usize;
            let verts = p.flex_vertnum[f] as usize;
            if p.flex_interp[f] == 0 {
                let bodies = &p.flex_vertbodyid[start..start + verts];
                let rigid = bodies.iter().all(|&b| b == bodies[0]);
                if self.flex_rigid[f] != rigid {
                    return Err(invalid("flex_rigid", f, "noncanonical_rigid_flag"));
                }
            }
            let ea = e.flex_edgeadr[f] as usize;
            let ne = e.flex_edgenum[f] as usize;
            let mut pairs = std::collections::BTreeSet::new();
            for edge in 0..ne {
                let pair = &e.flex_edge[2 * (ea + edge)..2 * (ea + edge) + 2];
                if !pairs.insert((pair[0].min(pair[1]), pair[0].max(pair[1]))) {
                    return Err(invalid("flex_edge", ea + edge, "duplicate_hessian_edge"));
                }
            }
            for el in 0..count {
                let v = &self.flex_elem[data + el * vwidth..data + (el + 1) * vwidth];
                if v.iter().any(|&id| id < 0 || id as usize >= verts)
                    || (0..v.len()).any(|i| v[i + 1..].contains(&v[i]))
                {
                    return Err(invalid(
                        "flex_elem",
                        data + el * vwidth,
                        "invalid_distinct_local_vertices",
                    ));
                }
                for (k, &[a, b]) in pattern.iter().enumerate() {
                    let slot = ee + el * pattern.len() + k;
                    let id = self.flex_elemedge[slot];
                    if id < 0 || id as usize >= ne {
                        return Err(invalid("flex_elemedge", slot, "edge_out_of_range"));
                    }
                    let pair = &e.flex_edge[2 * (ea + id as usize)..2 * (ea + id as usize) + 2];
                    if !((pair[0] == v[a] && pair[1] == v[b])
                        || (pair[1] == v[a] && pair[0] == v[b]))
                    {
                        return Err(invalid("flex_elemedge", slot, "edge_vertex_mismatch"));
                    }
                }
            }
            elem = elem
                .checked_add(count)
                .filter(|&n| n <= i32::MAX as usize)
                .ok_or_else(overflow)?;
            data = end_data;
            ee = end_ee;
        }
        for (field, actual, expected) in [
            ("flex_elem", self.flex_elem.len(), data),
            ("flex_elemedge", self.flex_elemedge.len(), ee),
            ("flex_stiffness", self.flex_stiffness.len(), stiffness),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields() -> (FlexPositionFields, FlexEdgeFields, FlexHessianFields) {
        (
            FlexPositionFields {
                flex_interp: vec![0],
                flex_vertadr: vec![0],
                flex_vertnum: vec![3],
                flex_vertbodyid: vec![0, 1, 2],
                ..FlexPositionFields::default()
            },
            FlexEdgeFields {
                flex_edgeadr: vec![0],
                flex_edgenum: vec![3],
                flex_edge: vec![1, 2, 2, 0, 0, 1],
                flexedge_j_rowadr: vec![0; 3],
                flexedge_j_rownnz: vec![0; 3],
                ..FlexEdgeFields::default()
            },
            FlexHessianFields {
                flex_dim: vec![2],
                flex_rigid: vec![false],
                flex_elemadr: vec![0],
                flex_elemnum: vec![1],
                flex_elemdataadr: vec![0],
                flex_elemedgeadr: vec![0],
                flex_stiffnessadr: vec![0],
                flex_elem: vec![0, 1, 2],
                flex_elemedge: vec![0, 1, 2],
                flexedge_length0: vec![1.0; 3],
                flex_stiffness: vec![1.0; 21],
            },
        )
    }
    #[test]
    fn accepts_21_coefficient_triangles_and_empty_hessian_fields() {
        let (p, e, h) = fields();
        h.validate(&p, &e).unwrap();
        FlexHessianFields::default()
            .validate(&FlexPositionFields::default(), &FlexEdgeFields::default())
            .unwrap();
        let mut h = h;
        h.flex_stiffnessadr[0] = -1;
        h.flex_stiffness.clear();
        h.validate(&p, &e).unwrap();
    }
    #[test]
    fn rejects_hessian_addresses_duplicate_edges_and_24_coefficient_tails() {
        let (p, e, h) = fields();
        for k in 0..9 {
            let mut b = h.clone();
            match k {
                0 => b.flex_dim[0] = 0,
                1 => b.flex_elemnum[0] = -1,
                2 => b.flex_elemnum[0] = i32::MAX,
                3 => b.flex_stiffnessadr[0] = i32::MAX,
                4 => b.flex_rigid[0] = true,
                5 => b.flex_elem[1] = 0,
                6 => b.flex_elemedge[1] = 0,
                7 => b.flex_stiffness.extend([0.0; 3]),
                _ => b.flex_elemdataadr[0] = i32::MAX,
            }
            assert!(b.validate(&p, &e).is_err(), "case {k}");
        }
        let mut e = e;
        e.flex_edge[0] = 2;
        e.flex_edge[1] = 0;
        assert!(h.validate(&p, &e).is_err());
    }
    #[test]
    fn rejects_hessian_nonfinite_and_negative_reference_lengths() {
        let (p, e, h) = fields();
        for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut b = h.clone();
            b.flex_stiffness[20] = x;
            assert!(b.validate(&p, &e).is_err());
            let mut b = h.clone();
            b.flexedge_length0[2] = x;
            assert!(b.validate(&p, &e).is_err());
        }
        let mut h = h;
        h.flexedge_length0[0] = -1.0;
        assert!(h.validate(&p, &e).is_err());
    }
}
