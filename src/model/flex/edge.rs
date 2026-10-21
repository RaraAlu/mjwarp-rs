//! 边拓扑与原生稀疏行校验。

use crate::diagnostics::InputError;
use crate::model::InertialModelInput;

use super::*;

impl FlexEdgeFields {
    pub fn nflexedge(&self) -> usize {
        self.flexedge_j_rownnz.len()
    }
    pub fn nnz(&self) -> usize {
        self.flexedge_j_colind.len()
    }
    pub(super) fn validate(
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
