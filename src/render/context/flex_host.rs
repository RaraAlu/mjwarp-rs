//! R06-S3pre：渲染 context 侧的 `FlexHostInputs` 组装层（宿主侧，零设备改动）。
//! 语义边界：
//! - 无 flex（`FlexRenderFields::nflex == 0`）→ [`FlexHostAssembly::build`] 返回 `Ok(None)`，调用方走
//!   既有 `MegakernelRenderer::render`；有 flex → `Ok(Some)`，调用方走
//!   `render_with_flex(..., Some(&assembly.inputs()))`。两条路径都只是**宿主侧实参装配**，
//!   `src/render/kernel/cuda/` 与全部 `.cu` 零改动 ⇒ 无 flex 路径输出与接线前位级一致。
//! - 依赖边界：policy 禁止 `mjwarp.render → mjwarp.physics`，故本组装层**不接收** physics 的
//!   `FlexRenderView` 类型；调用方把 S2b 只读视图里的 data 切片（`flexvert_xpos`）与 S3 前置的
//!   BVH/group-root 缓冲经 [`FlexHostData`] 传入。`render → model` 允许，三份 model 输入直传。
//! - 校验失败返回 [`FlexWiringError`]（不 panic、不静默回退）；组装完成后复用 S2 通道的
//!   [`FlexHostInputs::validate`] 做第二层校验。

use crate::diagnostics::InputError;
use crate::model::flex::{FlexEdgeFields, FlexGeomMapping, FlexPositionFields, FlexRenderFields};
use crate::render::kernel::megakernel::FlexHostInputs;
use crate::runtime::launcher::LauncherError;

mod render;

/// 调用方提供的 data 面切片（来自 S2b physics 只读视图 / S3 前置占位）。
#[derive(Clone, Copy, Debug)]
pub struct FlexHostData<'a> {
    /// 展平 `[nworld, nflexvert, 3]`：flex 顶点世界位姿（S2b `FlexPositionOutput::world(w).flexvert_xpos`）。
    pub flexvert_xpos: &'a [f32],
    /// `[nflex]`：flex BVH 句柄；S3 前置阶段由调用方给 0 占位（S3 本体填真值）。
    pub flex_bvh_ids: &'a [u64],
    /// 展平 `[nworld, nflex]`：flex BVH group root；S3 前置阶段由调用方给 -1 占位。
    pub flex_group_root: &'a [i32],
}

/// 组装错误（全部显式、可诊断；不 panic、不静默回退）。
#[derive(Clone, Debug, PartialEq)]
pub enum FlexWiringError {
    /// 数组长度与模型计数不一致。
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 计数溢出/非法或拓扑不一致。
    Count { field: &'static str },
    /// model 侧派生/校验失败（`FlexRenderFields::validate` 等）。
    Model(InputError),
    /// S2 宿主通道校验失败（`FlexHostInputs::validate`）。
    Channel(LauncherError),
}

impl From<InputError> for FlexWiringError {
    fn from(error: InputError) -> Self {
        Self::Model(error)
    }
}

impl From<LauncherError> for FlexWiringError {
    fn from(error: LauncherError) -> Self {
        Self::Channel(error)
    }
}

impl std::fmt::Display for FlexWiringError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Length {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "flex 组装长度不一致：{field} {actual} != {expected}"
            ),
            Self::Count { field } => write!(formatter, "flex 组装计数/拓扑非法：{field}"),
            Self::Model(error) => write!(formatter, "flex 组装模型校验失败：{error}"),
            Self::Channel(error) => write!(formatter, "flex 组装通道校验失败：{error}"),
        }
    }
}

impl std::error::Error for FlexWiringError {}

/// 组装结果：持有 geom 映射与输入借用；[`Self::inputs`] 给出可传给 `render_with_flex` 的视图。
#[derive(Clone, Debug)]
pub struct FlexHostAssembly<'a> {
    nworld: usize,
    positions: &'a FlexPositionFields,
    edges: &'a FlexEdgeFields,
    model: &'a FlexRenderFields,
    data: FlexHostData<'a>,
    mapping: FlexGeomMapping,
}

impl<'a> FlexHostAssembly<'a> {
    /// 组装宿主 flex 通道：无 flex 返回 `Ok(None)`；有 flex 返回 `Ok(Some)`（含显式校验）。
    pub fn build(
        nworld: usize,
        positions: &'a FlexPositionFields,
        edges: &'a FlexEdgeFields,
        model: &'a FlexRenderFields,
        data: FlexHostData<'a>,
    ) -> Result<Option<Self>, FlexWiringError> {
        if model.nflex == 0 {
            return Ok(None);
        }
        model.validate()?;
        check_sources(nworld, positions, edges, model, &data)?;
        let mapping = model.geom_mapping()?;
        let assembly = Self {
            nworld,
            positions,
            edges,
            model,
            data,
            mapping,
        };
        // 第二层：复用 S2 通道校验（长度/世界数/顶点范围/映射）。
        assembly.inputs().validate(nworld)?;
        Ok(Some(assembly))
    }

    /// 世界数（上游 `nworld`）。
    pub fn nworld(&self) -> usize {
        self.nworld
    }
    /// 柔体数。
    pub fn nflex(&self) -> usize {
        self.model.nflex
    }
    /// 全部顶点数。
    pub fn nflexvert(&self) -> usize {
        self.model.nflexvert
    }
    /// 全部边数。
    pub fn nflexedge(&self) -> usize {
        self.model.nflexedge
    }
    /// `bvh_nflexgeom`（`flex_geom_flexid.len()`）。
    pub fn bvh_nflexgeom(&self) -> usize {
        self.mapping.len()
    }

    /// 供 `MegakernelRenderer::render_with_flex` 使用的通道视图（借用本组装结果）。
    pub fn inputs(&self) -> FlexHostInputs<'_> {
        FlexHostInputs {
            nworld: self.nworld,
            nflex: self.model.nflex,
            nflexvert: self.model.nflexvert,
            flex_vertadr: &self.positions.flex_vertadr,
            flex_edge: &self.edges.flex_edge,
            flex_radius: &self.model.flex_radius,
            flexvert_xpos: self.data.flexvert_xpos,
            flex_rgba: &self.model.flex_rgba,
            flex_bvh_id: self.data.flex_bvh_ids,
            flex_group_root: self.data.flex_group_root,
            flex_geom_flexid: &self.mapping.flexid,
            flex_geom_edgeid: &self.mapping.edgeid,
        }
    }
}

fn expect(field: &'static str, expected: usize, actual: usize) -> Result<(), FlexWiringError> {
    if expected == actual {
        Ok(())
    } else {
        Err(FlexWiringError::Length {
            field,
            expected,
            actual,
        })
    }
}

fn checked(field: &'static str, value: Option<usize>) -> Result<usize, FlexWiringError> {
    value.ok_or(FlexWiringError::Count { field })
}

/// 同源性/拓扑校验（model-positions-edges-data 之间必须一致）。
fn check_sources(
    nworld: usize,
    positions: &FlexPositionFields,
    edges: &FlexEdgeFields,
    model: &FlexRenderFields,
    data: &FlexHostData<'_>,
) -> Result<(), FlexWiringError> {
    expect("flex_vertadr", model.nflex, positions.flex_vertadr.len())?;
    expect("flex_vertnum", model.nflex, positions.flex_vertnum.len())?;
    let vertbodyid = positions.flex_vertbodyid.len();
    expect("flex_vertbodyid", model.nflexvert, vertbodyid)?;
    let edge_entries = checked("flex_edge", model.nflexedge.checked_mul(2))?;
    expect("flex_edge", edge_entries, edges.flex_edge.len())?;
    if edges.nflexedge() != model.nflexedge {
        return Err(FlexWiringError::Count { field: "nflexedge" });
    }
    // flex_vertadr/vertnum 链：首项 0、逐 flex 累加、总和 = nflexvert。
    let mut next_vertex = 0usize;
    for flex in 0..model.nflex {
        let start =
            usize::try_from(positions.flex_vertadr[flex]).map_err(|_| FlexWiringError::Count {
                field: "flex_vertadr",
            })?;
        let count =
            usize::try_from(positions.flex_vertnum[flex]).map_err(|_| FlexWiringError::Count {
                field: "flex_vertnum",
            })?;
        if start != next_vertex {
            return Err(FlexWiringError::Count {
                field: "flex_vertadr",
            });
        }
        next_vertex = checked("flex_vertadr", start.checked_add(count))?;
        // 边端点用柔体局部顶点号，必须 < flex_vertnum[flex]。
        let edge_start =
            usize::try_from(model.flex_edgeadr[flex]).map_err(|_| FlexWiringError::Count {
                field: "flex_edgeadr",
            })?;
        let edge_count =
            usize::try_from(model.flex_edgenum[flex]).map_err(|_| FlexWiringError::Count {
                field: "flex_edgenum",
            })?;
        for edge in edge_start..edge_start + edge_count {
            for slot in 0..2 {
                let index = checked(
                    "flex_edge",
                    edge.checked_mul(2).and_then(|v| v.checked_add(slot)),
                )?;
                let local = usize::try_from(edges.flex_edge[index])
                    .map_err(|_| FlexWiringError::Count { field: "flex_edge" })?;
                if local >= count {
                    return Err(FlexWiringError::Count { field: "flex_edge" });
                }
            }
        }
    }
    if next_vertex != model.nflexvert {
        return Err(FlexWiringError::Count {
            field: "flex_vertadr",
        });
    }
    let verts_per_world = checked("flexvert_xpos", model.nflexvert.checked_mul(3))?;
    let xpos_len = checked("flexvert_xpos", nworld.checked_mul(verts_per_world))?;
    expect("flexvert_xpos", xpos_len, data.flexvert_xpos.len())?;
    expect("flex_bvh_id", model.nflex, data.flex_bvh_ids.len())?;
    let root_len = checked("flex_group_root", nworld.checked_mul(model.nflex))?;
    expect("flex_group_root", root_len, data.flex_group_root.len())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FlexHostAssembly, FlexHostData, FlexWiringError};
    use crate::model::flex::{FlexEdgeFields, FlexPositionFields, FlexRenderFields};

    fn model_single() -> FlexRenderFields {
        FlexRenderFields {
            nflex: 1,
            nflexvert: 2,
            nflexedge: 1,
            flex_dim: vec![1],
            flex_radius: vec![0.02],
            flex_rgba: vec![1.0, 0.0, 0.0, 1.0],
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
        }
    }

    fn positions_single() -> FlexPositionFields {
        FlexPositionFields {
            flex_vertadr: vec![0],
            flex_vertnum: vec![2],
            flex_vertbodyid: vec![0, 0],
            ..Default::default()
        }
    }

    fn edges_single() -> FlexEdgeFields {
        FlexEdgeFields {
            flex_edgeadr: vec![0],
            flex_edgenum: vec![1],
            flex_edge: vec![0, 1],
            flexedge_j_rownnz: vec![1],
            ..Default::default()
        }
    }

    const XPOS: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    const BVH: [u64; 1] = [0];
    const ROOT: [i32; 1] = [-1];

    fn data_single<'a>(xpos: &'a [f32], bvh: &'a [u64], root: &'a [i32]) -> FlexHostData<'a> {
        FlexHostData {
            flexvert_xpos: xpos,
            flex_bvh_ids: bvh,
            flex_group_root: root,
        }
    }

    /// 无 flex → `Ok(None)`（调用方走既有 `render`；两次调用一致）。
    #[test]
    fn no_flex_returns_none_deterministically() {
        let model = FlexRenderFields::default();
        let positions = FlexPositionFields::default();
        let edges = FlexEdgeFields::default();
        let data = FlexHostData {
            flexvert_xpos: &[],
            flex_bvh_ids: &[],
            flex_group_root: &[],
        };
        let first = FlexHostAssembly::build(1, &positions, &edges, &model, data);
        let second = FlexHostAssembly::build(1, &positions, &edges, &model, data);
        assert!(first.unwrap().is_none());
        assert!(second.unwrap().is_none());
    }

    /// 有 flex → `Ok(Some)`，通道 9 数组/计数/映射逐项落位。
    #[test]
    fn single_flex_assembles_full_channel() {
        let model = model_single();
        let positions = positions_single();
        let edges = edges_single();
        let assembly = FlexHostAssembly::build(
            1,
            &positions,
            &edges,
            &model,
            data_single(&XPOS, &BVH, &ROOT),
        )
        .unwrap()
        .expect("有 flex 必须组装 Some");
        assert_eq!(assembly.nworld(), 1);
        assert_eq!(assembly.nflex(), 1);
        assert_eq!(assembly.nflexvert(), 2);
        assert_eq!(assembly.nflexedge(), 1);
        assert_eq!(assembly.bvh_nflexgeom(), 1);
        let inputs = assembly.inputs();
        assert_eq!(inputs.nworld, 1);
        assert_eq!(inputs.nflex, 1);
        assert_eq!(inputs.nflexvert, 2);
        assert_eq!(inputs.flex_vertadr, [0]);
        assert_eq!(inputs.flex_edge, [0, 1]);
        assert_eq!(inputs.flex_radius, [0.02f32]);
        assert_eq!(inputs.flexvert_xpos, XPOS);
        assert_eq!(inputs.flex_rgba.len(), 4);
        assert_eq!(inputs.flex_bvh_id, BVH);
        assert_eq!(inputs.flex_group_root, ROOT);
        assert_eq!(inputs.flex_geom_flexid, [0]);
        assert_eq!(inputs.flex_geom_edgeid, [0]);
        assert!(inputs.validate(1).is_ok());
    }

    /// 多 flex / 多 world：计数与映射（1D 每边一图元、2D 每 flex 一图元）。
    #[test]
    fn multi_flex_multi_world_assembles_mapping() {
        let model = FlexRenderFields {
            nflex: 2,
            nflexvert: 3,
            nflexedge: 2,
            flex_dim: vec![1, 2],
            flex_radius: vec![0.02, 0.03],
            flex_rgba: vec![0.5; 8],
            flex_edgeadr: vec![0, 1],
            flex_edgenum: vec![1, 1],
        };
        let positions = FlexPositionFields {
            flex_vertadr: vec![0, 1],
            flex_vertnum: vec![1, 2],
            flex_vertbodyid: vec![0, 0, 0],
            ..Default::default()
        };
        let edges = FlexEdgeFields {
            flex_edgeadr: vec![0, 1],
            flex_edgenum: vec![1, 1],
            flex_edge: vec![0, 0, 0, 1],
            flexedge_j_rownnz: vec![1, 1],
            ..Default::default()
        };
        let xpos = [0.0f32; 3 * 2 * 3];
        let bvh = [7u64, 0];
        let root = [-1i32; 4];
        let assembly = FlexHostAssembly::build(
            2,
            &positions,
            &edges,
            &model,
            FlexHostData {
                flexvert_xpos: &xpos,
                flex_bvh_ids: &bvh,
                flex_group_root: &root,
            },
        )
        .unwrap()
        .expect("有 flex 必须组装 Some");
        assert_eq!(assembly.bvh_nflexgeom(), 2);
        let inputs = assembly.inputs();
        assert_eq!(inputs.flex_geom_flexid, [0, 1]);
        assert_eq!(inputs.flex_geom_edgeid, [0, -1]);
    }

    /// 尺寸不一致 → 显式 `Length` 错误（不 panic）。
    #[test]
    fn length_mismatches_are_rejected_explicitly() {
        let model = model_single();
        let positions = positions_single();
        let edges = edges_single();
        let cases: [(&str, FlexHostData<'_>); 3] = [
            ("flexvert_xpos", data_single(&XPOS[..3], &BVH, &ROOT)),
            ("flex_bvh_id", data_single(&XPOS, &BVH[..0], &ROOT)),
            ("flex_group_root", data_single(&XPOS, &BVH, &ROOT[..0])),
        ];
        for (field, data) in cases {
            let error = FlexHostAssembly::build(1, &positions, &edges, &model, data).unwrap_err();
            assert!(
                matches!(error, FlexWiringError::Length { field: name, .. } if name == field),
                "{field}"
            );
        }
    }

    /// 拓扑不一致（顶点链断裂 / 边端点越界）→ 显式 `Count` 错误。
    #[test]
    fn topology_mismatches_are_rejected_explicitly() {
        let model = model_single();
        let edges = edges_single();
        let mut broken_positions = positions_single();
        broken_positions.flex_vertnum = vec![1];
        let error = FlexHostAssembly::build(
            1,
            &broken_positions,
            &edges,
            &model,
            data_single(&XPOS, &BVH, &ROOT),
        )
        .unwrap_err();
        assert!(matches!(error, FlexWiringError::Count { .. }));
        let mut broken_edges = edges_single();
        broken_edges.flex_edge = vec![0, 2];
        let error = FlexHostAssembly::build(
            1,
            &positions_single(),
            &broken_edges,
            &model,
            data_single(&XPOS, &BVH, &ROOT),
        )
        .unwrap_err();
        assert!(matches!(error, FlexWiringError::Count { .. }));
    }

    /// `flex_vertnum` 短于 nflex → 显式 `Length`（原来是可直接触发的索引越界 panic 路径）。
    #[test]
    fn short_flex_vertnum_returns_length_error() {
        let mut positions = positions_single();
        positions.flex_vertnum.clear();
        let data = data_single(&XPOS, &BVH, &ROOT);
        let error = FlexHostAssembly::build(1, &positions, &edges_single(), &model_single(), data)
            .unwrap_err();
        let FlexWiringError::Length { field, .. } = error else {
            panic!("必须返回 Length 错误，实际不是");
        };
        assert_eq!(field, "flex_vertnum");
    }

    /// 有 flex 两次组装逐位一致（同输入确定性）。
    #[test]
    fn assembly_is_bit_identical_across_calls() {
        let model = model_single();
        let positions = positions_single();
        let edges = edges_single();
        let first = FlexHostAssembly::build(
            1,
            &positions,
            &edges,
            &model,
            data_single(&XPOS, &BVH, &ROOT),
        )
        .unwrap()
        .unwrap();
        let second = FlexHostAssembly::build(
            1,
            &positions,
            &edges,
            &model,
            data_single(&XPOS, &BVH, &ROOT),
        )
        .unwrap()
        .unwrap();
        assert_eq!(first.inputs().nworld, second.inputs().nworld);
        assert_eq!(first.inputs().flex_vertadr, second.inputs().flex_vertadr);
        assert_eq!(
            first
                .inputs()
                .flexvert_xpos
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            second
                .inputs()
                .flexvert_xpos
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
    }
}
