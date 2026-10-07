//! 检查柔体节点与顶点位置。
//! 共享拓扑与局部坐标。

use super::TendonModelInput;
use crate::diagnostics::InputError;

/// 原生柔体位置字段。
/// 仅支持直接与线性插值。
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
/// 不提供边、弹性与休眠。
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
        Ok(Self { tendons, fields })
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
    pub(crate) fn into_parts(self) -> (TendonModelInput, FlexPositionFields) {
        (self.tendons, self.fields)
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
            if mode != 0 && mode != 1 {
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
            if mode == 1 {
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
                if mode == 1 {
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
    fn rejects_high_order_shell_and_unsafe_linear_cells() {
        for mode in [-2, -1, 2, i32::MAX] {
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
}
