//! 柔体位置字段校验。

use crate::diagnostics::InputError;

use super::*;

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
