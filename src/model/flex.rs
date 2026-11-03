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

/// R06-S2b：flex 渲染数据源字段（原生 additive `flex_render` 桶）。
/// 只读：不进入物理/设备路径；`flex_edge` 由 G01 字段集提供（不重复携带）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlexRenderFields {
    /// 柔体数（上游 `mjm.nflex`）。
    pub nflex: usize,
    /// 全部柔体顶点数（上游 `mjm.nflexvert`）。
    pub nflexvert: usize,
    /// 全部柔体边数（上游 `mjm.nflexedge`）。
    pub nflexedge: usize,
    /// `[nflex]`：1 = lines、2 = triangles、3 = tetrahedra。
    pub flex_dim: Vec<i32>,
    /// `[nflex]`：图元半径（原生 mjtNum 已在 io 侧统一转 f32）。
    pub flex_radius: Vec<f32>,
    /// 展平 `[nflex, 4]`：rgba（原生 float）。
    pub flex_rgba: Vec<f32>,
    /// `[nflex]`：首边地址。
    pub flex_edgeadr: Vec<i32>,
    /// `[nflex]`：边数。
    pub flex_edgenum: Vec<i32>,
}

/// R06-S2c：flex shell/elem 拓扑只读字段（原生 additive `mjwarp_flex_topology_*` 桶）。
/// 只读：不进物理/设备路径；`flex_vertadr`/`vertnum` 仍在 `FlexPositionFields`。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlexTopologyFields {
    pub nflex: usize,
    pub nflexelem: usize,
    pub nflexelemdata: usize,
    pub nflexshelldata: usize,
    pub flex_elemadr: Vec<i32>,
    pub flex_elemnum: Vec<i32>,
    pub flex_elemdataadr: Vec<i32>,
    pub flex_elem: Vec<i32>,
    pub flex_shellnum: Vec<i32>,
    pub flex_shelldataadr: Vec<i32>,
    pub flex_shell: Vec<i32>,
}

impl FlexTopologyFields {
    /// 校验计数、长度与非负（地址链/逐维宽度由 `io::flex::topology` 边界联合校验）。
    pub fn validate(&self) -> Result<(), InputError> {
        let overflow = || InputError::Overflow {
            field: "flex_topology_fields",
        };
        let invalid = |field, index, reason| InputError::InvalidTopology {
            field,
            index,
            reason,
        };
        if self.nflex > i32::MAX as usize
            || self.nflexelem > i32::MAX as usize
            || self.nflexelemdata > i32::MAX as usize
            || self.nflexshelldata > i32::MAX as usize
        {
            return Err(overflow());
        }
        for (field, len, expected) in [
            ("flex_elemadr", self.flex_elemadr.len(), self.nflex),
            ("flex_elemnum", self.flex_elemnum.len(), self.nflex),
            ("flex_elemdataadr", self.flex_elemdataadr.len(), self.nflex),
            ("flex_shellnum", self.flex_shellnum.len(), self.nflex),
            (
                "flex_shelldataadr",
                self.flex_shelldataadr.len(),
                self.nflex,
            ),
            ("flex_elem", self.flex_elem.len(), self.nflexelemdata),
            ("flex_shell", self.flex_shell.len(), self.nflexshelldata),
        ] {
            if len != expected {
                return Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual: len,
                });
            }
        }
        let negative = self
            .flex_elemadr
            .iter()
            .chain(&self.flex_elemnum)
            .chain(&self.flex_elemdataadr)
            .chain(&self.flex_shellnum)
            .chain(&self.flex_shelldataadr)
            .any(|&value| value < 0);
        if negative {
            return Err(invalid("flex_topology_fields", 0, "negative_value"));
        }
        Ok(())
    }
}

/// flex 图元映射（上游 `render_util.py` L495-512）：每项 = (flex id, edge id 或 -1)。
///
/// 1D flex 每条边一个胶囊图元（`edgeid = flex_edgeadr[f] + e`）；2D/3D flex 每个一个盒图元
/// （`edgeid = -1`）。长度即 `bvh_nflexgeom`。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlexGeomMapping {
    pub flexid: Vec<i32>,
    pub edgeid: Vec<i32>,
}

impl FlexGeomMapping {
    pub fn len(&self) -> usize {
        self.flexid.len()
    }
    pub fn is_empty(&self) -> bool {
        self.flexid.is_empty()
    }
}

impl FlexRenderFields {
    /// 校验计数、长度与边地址拓扑（不 panic、不静默回退）。
    pub fn validate(&self) -> Result<(), InputError> {
        let overflow = || InputError::Overflow {
            field: "flex_render_fields",
        };
        let invalid = |field, index, reason| InputError::InvalidTopology {
            field,
            index,
            reason,
        };
        if self.nflex > i32::MAX as usize || self.nflexvert > i32::MAX as usize {
            return Err(overflow());
        }
        for (field, len) in [
            ("flex_dim", self.flex_dim.len()),
            ("flex_radius", self.flex_radius.len()),
            ("flex_edgeadr", self.flex_edgeadr.len()),
            ("flex_edgenum", self.flex_edgenum.len()),
        ] {
            if len != self.nflex {
                return Err(InputError::LengthMismatch {
                    field,
                    expected: self.nflex,
                    actual: len,
                });
            }
        }
        let rgba = self.nflex.checked_mul(4).ok_or_else(overflow)?;
        if self.flex_rgba.len() != rgba {
            return Err(InputError::LengthMismatch {
                field: "flex_rgba",
                expected: rgba,
                actual: self.flex_rgba.len(),
            });
        }
        if self.nflex != 0 && self.nflexvert == 0 {
            return Err(invalid("flex_vertnum", 0, "empty_flex_vertices"));
        }
        for f in 0..self.nflex {
            if !(1..=3).contains(&self.flex_dim[f]) {
                return Err(invalid("flex_dim", f, "unsupported_dimension"));
            }
            let radius = self.flex_radius[f];
            if !radius.is_finite() || radius < 0.0 {
                return Err(invalid("flex_radius", f, "invalid_radius"));
            }
            for channel in 0..4 {
                if !self.flex_rgba[4 * f + channel].is_finite() {
                    return Err(invalid("flex_rgba", 4 * f + channel, "non_finite"));
                }
            }
            let start = usize::try_from(self.flex_edgeadr[f])
                .map_err(|_| invalid("flex_edgeadr", f, "negative_address"))?;
            let count = usize::try_from(self.flex_edgenum[f])
                .map_err(|_| invalid("flex_edgenum", f, "negative_count"))?;
            let next = self
                .flex_edgeadr
                .get(f + 1)
                .map(|value| usize::try_from(*value))
                .transpose()
                .map_err(|_| invalid("flex_edgeadr", f + 1, "negative_address"))?
                .unwrap_or(self.nflexedge);
            if f == 0 && start != 0 {
                return Err(invalid("flex_edgeadr", 0, "first_edge_not_zero"));
            }
            if start.checked_add(count) != Some(next) {
                return Err(invalid("flex_edgeadr", f, "edge_range_gap_or_overlap"));
            }
        }
        let last = self
            .flex_edgeadr
            .last()
            .map(|value| usize::try_from(*value))
            .transpose()
            .map_err(|_| invalid("flex_edgeadr", 0, "negative_address"))?
            .unwrap_or(0);
        let last_count = self
            .flex_edgenum
            .last()
            .map(|value| usize::try_from(*value))
            .transpose()
            .map_err(|_| invalid("flex_edgenum", 0, "negative_count"))?
            .unwrap_or(0);
        if last.checked_add(last_count) != Some(self.nflexedge) {
            return Err(invalid("flex_edgeadr", 0, "edge_total_mismatch"));
        }
        Ok(())
    }

    /// 派生 flex 图元映射（上游 `render_util.py` L495-512）；先做完整校验。
    pub fn geom_mapping(&self) -> Result<FlexGeomMapping, InputError> {
        self.validate()?;
        let mut mapping = FlexGeomMapping::default();
        for f in 0..self.nflex {
            let one_dimensional = self.flex_dim[f] == 1;
            if !one_dimensional {
                mapping.flexid.push(f as i32);
                mapping.edgeid.push(-1);
                continue;
            }
            let count =
                usize::try_from(self.flex_edgenum[f]).map_err(|_| InputError::Overflow {
                    field: "flex_geom_mapping",
                })?;
            let start =
                usize::try_from(self.flex_edgeadr[f]).map_err(|_| InputError::Overflow {
                    field: "flex_geom_mapping",
                })?;
            for edge in start..start + count {
                let edge = i32::try_from(edge).map_err(|_| InputError::Overflow {
                    field: "flex_geom_mapping",
                })?;
                mapping.flexid.push(f as i32);
                mapping.edgeid.push(edge);
            }
        }
        Ok(mapping)
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
#[cfg(test)]
mod render_tests {
    use super::{FlexGeomMapping, FlexRenderFields};

    fn single_1d() -> FlexRenderFields {
        FlexRenderFields {
            nflex: 1,
            nflexvert: 2,
            nflexedge: 2,
            flex_dim: vec![1],
            flex_radius: vec![0.02],
            flex_rgba: vec![1.0, 0.0, 0.0, 1.0],
            flex_edgeadr: vec![0],
            flex_edgenum: vec![2],
        }
    }

    /// 空模型：0 计数、空映射（与 S2 空通道等价占位口径一致）。
    #[test]
    fn empty_flex_render_fields_validate() {
        let empty = FlexRenderFields::default();
        empty.validate().unwrap();
        assert_eq!(empty.geom_mapping().unwrap(), FlexGeomMapping::default());
    }

    /// 单 1D flex：每条边一个胶囊图元，edgeid = 边号。
    #[test]
    fn single_1d_flex_maps_one_capsule_per_edge() {
        let mapping = single_1d().geom_mapping().unwrap();
        assert_eq!(mapping.len(), 2);
        assert_eq!(mapping.flexid, [0, 0]);
        assert_eq!(mapping.edgeid, [0, 1]);
    }

    /// 多 flex 混合维度：1D 按边、2D 按 flex（edgeid = -1）。
    #[test]
    fn mixed_dimensions_map_boxes_and_capsules() {
        let fields = FlexRenderFields {
            nflex: 2,
            nflexvert: 3,
            nflexedge: 2,
            flex_dim: vec![1, 2],
            flex_radius: vec![0.02, 0.03],
            flex_rgba: vec![0.0; 8],
            flex_edgeadr: vec![0, 1],
            flex_edgenum: vec![1, 1],
        };
        let mapping = fields.geom_mapping().unwrap();
        assert_eq!(mapping.flexid, [0, 1]);
        assert_eq!(mapping.edgeid, [0, -1]);
    }

    /// 长度 / 计数 / 拓扑不一致：显式拒绝（不 panic、不静默回退）。
    #[test]
    fn rejects_length_count_and_topology_violations() {
        let cases: Vec<(&str, FlexRenderFields)> = vec![
            (
                "rgba_len",
                FlexRenderFields {
                    flex_rgba: vec![0.0; 3],
                    ..single_1d()
                },
            ),
            (
                "dim_len",
                FlexRenderFields {
                    nflex: 2,
                    ..single_1d()
                },
            ),
            (
                "empty_vertices",
                FlexRenderFields {
                    nflexvert: 0,
                    ..single_1d()
                },
            ),
            (
                "dim_range",
                FlexRenderFields {
                    flex_dim: vec![4],
                    ..single_1d()
                },
            ),
            (
                "radius_nan",
                FlexRenderFields {
                    flex_radius: vec![f32::NAN],
                    ..single_1d()
                },
            ),
            (
                "radius_negative",
                FlexRenderFields {
                    flex_radius: vec![-1.0],
                    ..single_1d()
                },
            ),
            (
                "rgba_nan",
                FlexRenderFields {
                    flex_rgba: vec![f32::NAN; 4],
                    ..single_1d()
                },
            ),
            (
                "first_edge_not_zero",
                FlexRenderFields {
                    flex_edgeadr: vec![1],
                    ..single_1d()
                },
            ),
            (
                "edge_total_mismatch",
                FlexRenderFields {
                    nflexedge: 3,
                    ..single_1d()
                },
            ),
            (
                "nflex_over_i32",
                FlexRenderFields {
                    nflex: i32::MAX as usize + 1,
                    ..single_1d()
                },
            ),
        ];
        for (name, fields) in cases {
            assert!(fields.validate().is_err(), "case {name} 必须显式拒绝");
        }
    }
}
