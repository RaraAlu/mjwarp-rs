//! S2：把 32 项宿主快照（模型静态字段 + mjData geom 帧）组装成 [`CollisionGpuInputs`] 视图。
//!
//! 本模块只做 CPU 组装与既有 `validate_collision_inputs` 校验：不分配设备缓冲、不使用 GPU、
//! 不做产品入口接线（S3 才接）、不改设备源/ABI/判据。空/部分空语义沿用 M0a：
//! 9 项 mesh_poly 全空 → sidecar 不构造（`mesh_poly_args = null`）；部分为空 → `InvalidArgument`。
//!
//! 字段顺序与 [`CollisionGpuInputs`] 一致；`meta` 由调用方给出（32 项 slice 视图不含 meta）。

use super::{CollisionGpuError, CollisionGpuInputs, CollisionGpuMeta, abi::validate_inputs};

#[cfg(feature = "native-model-probe")]
use crate::{
    io::{CollisionInputsFieldValue, CollisionInputsFrame, CollisionInputsModelInput},
    model::collision_inputs::{COLLISION_INPUTS_FIELDS, CollisionInputsFieldSource},
};

/// 32 项宿主快照的借用视图；字段顺序/宽度与 [`CollisionGpuInputs`] 1:1。
///
/// 模型静态字段（mesh/hfield/octree）与 world 无关：调用方给一份共享数据。
/// `geom_xpos`/`geom_xmat` 是**单个 world** 的 mjData 帧快照（ngeom×3 / ngeom×9）；
/// `nworld > 1` 的重复由更上层负责（本片不做产品接线）。
#[derive(Clone, Copy, Debug)]
pub struct CollisionInputsSnapshot<'a> {
    pub geom_xpos: &'a [f32],
    pub geom_xmat: &'a [f32],
    pub geom_size: &'a [f32],
    pub geom_margin: &'a [f32],
    pub geom_gap: &'a [f32],
    pub geom_rbound: &'a [f32],
    pub geom_aabb: &'a [f32],
    pub geom_type: &'a [i32],
    pub geom_dataid: &'a [i32],
    pub mesh_vertadr: &'a [i32],
    pub mesh_vertnum: &'a [i32],
    pub mesh_graphadr: &'a [i32],
    pub mesh_octadr: &'a [i32],
    pub mesh_vert: &'a [f32],
    pub mesh_graph: &'a [i32],
    pub mesh_polynum: &'a [i32],
    pub mesh_polyadr: &'a [i32],
    pub mesh_polynormal: &'a [f32],
    pub mesh_polyvertadr: &'a [i32],
    pub mesh_polyvertnum: &'a [i32],
    pub mesh_polyvert: &'a [i32],
    pub mesh_polymapadr: &'a [i32],
    pub mesh_polymapnum: &'a [i32],
    pub mesh_polymap: &'a [i32],
    pub hfield_size: &'a [f32],
    pub hfield_nrow: &'a [i32],
    pub hfield_ncol: &'a [i32],
    pub hfield_adr: &'a [i32],
    pub hfield_data: &'a [f32],
    pub oct_aabb: &'a [f32],
    pub oct_child: &'a [i32],
    pub oct_coeff: &'a [f32],
}

impl<'a> CollisionInputsSnapshot<'a> {
    /// 由 S1 的模型快照与 mjData 帧快照组装视图。
    ///
    /// 先按 [`COLLISION_INPUTS_FIELDS`] 校验 30 项模型字段的类型/长度（与
    /// `CollisionInputsCounts` 口径一致），再借用它们的拥有式 `Vec`；帧快照长度按
    /// 单 world 口径校验（`ngeom×3` / `ngeom×9`）。失败返回 [`CollisionGpuError::InvalidArgument`]。
    #[cfg(feature = "native-model-probe")]
    pub fn from_model_input(
        model: &'a CollisionInputsModelInput,
        frame: &'a CollisionInputsFrame,
    ) -> Result<Self, CollisionGpuError> {
        let counts = model.counts();
        counts
            .validate()
            .map_err(|_| CollisionGpuError::InvalidArgument("快照模型计数无效"))?;
        let ngeom = usize::try_from(counts.ngeom)
            .map_err(|_| CollisionGpuError::InvalidArgument("快照 ngeom 溢出"))?;
        let expected_xpos = ngeom
            .checked_mul(3)
            .ok_or(CollisionGpuError::InvalidArgument("快照 ngeom 溢出"))?;
        let expected_xmat = ngeom
            .checked_mul(9)
            .ok_or(CollisionGpuError::InvalidArgument("快照 ngeom 溢出"))?;
        if frame.geom_xpos.len() != expected_xpos || frame.geom_xmat.len() != expected_xmat {
            return Err(CollisionGpuError::InvalidArgument(
                "快照 geom 帧长度须为单 world 的 ngeom×3 / ngeom×9",
            ));
        }
        for id in 2..u32::try_from(COLLISION_INPUTS_FIELDS.len()).unwrap_or(u32::MAX) {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            if spec.source != CollisionInputsFieldSource::Model {
                return Err(CollisionGpuError::InvalidArgument(
                    "快照字段来源应为 mjModel 静态字段",
                ));
            }
            let expected = counts
                .expected(spec)
                .map_err(|_| CollisionGpuError::InvalidArgument("快照字段计数溢出"))?;
            let actual = match model.field(id) {
                Some(CollisionInputsFieldValue::Float(values)) => values.len(),
                Some(CollisionInputsFieldValue::Index(values)) => values.len(),
                None => return Err(CollisionGpuError::InvalidArgument("快照缺少模型字段")),
            };
            if actual != expected {
                return Err(CollisionGpuError::InvalidArgument(
                    "快照字段长度与计数不一致",
                ));
            }
        }
        Ok(Self {
            geom_xpos: &frame.geom_xpos,
            geom_xmat: &frame.geom_xmat,
            geom_size: float_field(model, 2)?,
            geom_margin: float_field(model, 3)?,
            geom_gap: float_field(model, 4)?,
            geom_rbound: float_field(model, 5)?,
            geom_aabb: float_field(model, 6)?,
            geom_type: index_field(model, 7)?,
            geom_dataid: index_field(model, 8)?,
            mesh_vertadr: index_field(model, 9)?,
            mesh_vertnum: index_field(model, 10)?,
            mesh_graphadr: index_field(model, 11)?,
            mesh_octadr: index_field(model, 12)?,
            mesh_vert: float_field(model, 13)?,
            mesh_graph: index_field(model, 14)?,
            mesh_polynum: index_field(model, 15)?,
            mesh_polyadr: index_field(model, 16)?,
            mesh_polynormal: float_field(model, 17)?,
            mesh_polyvertadr: index_field(model, 18)?,
            mesh_polyvertnum: index_field(model, 19)?,
            mesh_polyvert: index_field(model, 20)?,
            mesh_polymapadr: index_field(model, 21)?,
            mesh_polymapnum: index_field(model, 22)?,
            mesh_polymap: index_field(model, 23)?,
            hfield_size: float_field(model, 24)?,
            hfield_nrow: index_field(model, 25)?,
            hfield_ncol: index_field(model, 26)?,
            hfield_adr: index_field(model, 27)?,
            hfield_data: float_field(model, 28)?,
            oct_aabb: float_field(model, 29)?,
            oct_child: index_field(model, 30)?,
            oct_coeff: float_field(model, 31)?,
        })
    }
}

/// 用调用方给定的 [`CollisionGpuMeta`] 组装 [`CollisionGpuInputs`] 并调用既有
/// `validate_collision_inputs`（语义与 [`super::CollisionGpuProgram::run`] 首步校验完全一致），
/// 随后执行**方案 C 的产品路径容量门**：mesh 多接触设备固定容量（face≤20 / degree≤10）超界时
/// 返回 [`CollisionGpuError::MeshMulticontactCapacity`]，不再让产品静默回退单接触。
///
/// 9 项 mesh_poly 全空 → 校验通过且容量门通过，`run` 侧保持 `mesh_poly_args = null`；
/// 部分为空或长度不自洽 → [`CollisionGpuError::InvalidArgument`]。直连
/// [`super::CollisionGpuProgram::run`] 的探针路径不变（仍按设备真实行为 record）。本函数不 panic。
pub fn inputs_from_snapshot<'a>(
    snapshot: &CollisionInputsSnapshot<'a>,
    meta: CollisionGpuMeta,
) -> Result<CollisionGpuInputs<'a>, CollisionGpuError> {
    let inputs = CollisionGpuInputs {
        geom_xpos: snapshot.geom_xpos,
        geom_xmat: snapshot.geom_xmat,
        geom_size: snapshot.geom_size,
        geom_margin: snapshot.geom_margin,
        geom_gap: snapshot.geom_gap,
        geom_rbound: snapshot.geom_rbound,
        geom_aabb: snapshot.geom_aabb,
        geom_type: snapshot.geom_type,
        geom_dataid: snapshot.geom_dataid,
        mesh_vertadr: snapshot.mesh_vertadr,
        mesh_vertnum: snapshot.mesh_vertnum,
        mesh_graphadr: snapshot.mesh_graphadr,
        mesh_octadr: snapshot.mesh_octadr,
        mesh_vert: snapshot.mesh_vert,
        mesh_graph: snapshot.mesh_graph,
        mesh_polynum: snapshot.mesh_polynum,
        mesh_polyadr: snapshot.mesh_polyadr,
        mesh_polynormal: snapshot.mesh_polynormal,
        mesh_polyvertadr: snapshot.mesh_polyvertadr,
        mesh_polyvertnum: snapshot.mesh_polyvertnum,
        mesh_polyvert: snapshot.mesh_polyvert,
        mesh_polymapadr: snapshot.mesh_polymapadr,
        mesh_polymapnum: snapshot.mesh_polymapnum,
        mesh_polymap: snapshot.mesh_polymap,
        hfield_size: snapshot.hfield_size,
        hfield_nrow: snapshot.hfield_nrow,
        hfield_ncol: snapshot.hfield_ncol,
        hfield_adr: snapshot.hfield_adr,
        hfield_data: snapshot.hfield_data,
        oct_aabb: snapshot.oct_aabb,
        oct_child: snapshot.oct_child,
        oct_coeff: snapshot.oct_coeff,
        meta,
    };
    validate_inputs(&inputs)?;
    // 方案 C（产品路径容量门）：设备 mesh 多接触是固定容量（face≤20 / degree≤10），超界时
    // 设备会逐 feature 回退单接触；产品路径在这里显式拒绝，不把静默近似交给下游。
    let violations = super::types::mesh_multicontact_capacity_violations(
        inputs.mesh_polyvertnum,
        inputs.mesh_polymapnum,
    );
    if !violations.is_empty() {
        return Err(CollisionGpuError::MeshMulticontactCapacity {
            violations,
            reject_path: super::types::MESH_MULTICONTACT_REJECT_PATH,
        });
    }
    Ok(inputs)
}

/// `run` 内 `mesh_poly_args` 是否非空的同一判据（`!mesh_polynum.is_empty()`）。
///
/// 探针/产品语义分离：本片不改 `run`，该谓词供 host 测试断言 sidecar 是否存在。
#[doc(hidden)]
pub fn mesh_poly_sidecar_present(inputs: &CollisionGpuInputs<'_>) -> bool {
    !inputs.mesh_polynum.is_empty()
}

#[cfg(feature = "native-model-probe")]
fn float_field(model: &CollisionInputsModelInput, id: u32) -> Result<&[f32], CollisionGpuError> {
    match model.field(id) {
        Some(CollisionInputsFieldValue::Float(values)) => Ok(values),
        _ => Err(CollisionGpuError::InvalidArgument("快照字段类型应为 f32")),
    }
}

#[cfg(feature = "native-model-probe")]
fn index_field(model: &CollisionInputsModelInput, id: u32) -> Result<&[i32], CollisionGpuError> {
    match model.field(id) {
        Some(CollisionInputsFieldValue::Index(values)) => Ok(values),
        _ => Err(CollisionGpuError::InvalidArgument("快照字段类型应为 i32")),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CollisionGpuError, CollisionGpuMeta, CollisionInputsSnapshot, inputs_from_snapshot,
    };
    use crate::runtime::collision_gpu::types::{
        MESH_MULTICONTACT_DEGREE_MAX, MESH_MULTICONTACT_FACE_MAX, MESH_MULTICONTACT_REJECT_PATH,
        MeshCapacityDimension,
    };
    const FAN_CONE_24_OBJ: &str =
        include_str!("../../../fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj");
    const MULTICONTACT_GEOM_CU: &str =
        include_str!("../../physics/collision/cuda/multicontact_geom.cu");
    /// 解析 OBJ 的 `v`/`f`（只关心顶点数与逐面顶点号；索引转 0 基）。
    fn parse_obj(text: &str) -> (usize, Vec<Vec<usize>>) {
        let mut vertices = 0;
        let mut polygons = Vec::new();
        for line in text.lines().map(str::trim) {
            if line.starts_with("v ") {
                vertices += 1;
            } else if let Some(rest) = line.strip_prefix("f ") {
                polygons.push(rest.split_whitespace().map(face_index).collect());
            }
        }
        (vertices, polygons)
    }
    fn face_index(token: &str) -> usize {
        token
            .split('/')
            .next()
            .unwrap_or(token)
            .parse::<usize>()
            .expect("OBJ face index")
            - 1
    }
    /// 单 mesh 的最小合法 sidecar + geom 数组（够走通 `validate_inputs` 的形状/值域校验）。
    struct Fixture {
        geom_xpos: Vec<f32>,
        geom_xmat: Vec<f32>,
        geom_size: Vec<f32>,
        geom_margin: Vec<f32>,
        geom_gap: Vec<f32>,
        geom_rbound: Vec<f32>,
        geom_aabb: Vec<f32>,
        geom_type: Vec<i32>,
        geom_dataid: Vec<i32>,
        mesh_vertadr: Vec<i32>,
        mesh_vertnum: Vec<i32>,
        mesh_vert: Vec<f32>,
        mesh_polynum: Vec<i32>,
        mesh_polyadr: Vec<i32>,
        mesh_polynormal: Vec<f32>,
        mesh_polyvertadr: Vec<i32>,
        mesh_polyvertnum: Vec<i32>,
        mesh_polyvert: Vec<i32>,
        mesh_polymapadr: Vec<i32>,
        mesh_polymapnum: Vec<i32>,
        mesh_polymap: Vec<i32>,
    }
    impl Fixture {
        fn new(vertices: usize, polygons: &[Vec<usize>]) -> Self {
            let mut polyvertadr = Vec::new();
            let mut polyvertnum = Vec::new();
            let mut polyvert = Vec::new();
            for face in polygons {
                polyvertadr.push(polyvert.len() as i32);
                polyvertnum.push(face.len() as i32);
                polyvert.extend(face.iter().map(|vertex| *vertex as i32));
            }
            let mut polymap_counts = vec![0usize; vertices];
            for vertex in polygons.iter().flatten() {
                polymap_counts[*vertex] += 1;
            }
            let mut polymapadr = Vec::new();
            let mut polymapnum = Vec::new();
            let mut polymap = Vec::new();
            for (vertex, count) in polymap_counts.iter().enumerate() {
                polymapadr.push(polymap.len() as i32);
                polymapnum.push(*count as i32);
                for (polygon, face) in polygons.iter().enumerate() {
                    if face.contains(&vertex) {
                        polymap.push(polygon as i32);
                    }
                }
            }
            Self {
                geom_xpos: vec![0.0; 3],
                geom_xmat: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                geom_size: vec![0.1; 3],
                geom_margin: vec![0.0],
                geom_gap: vec![0.0],
                geom_rbound: vec![0.1],
                geom_aabb: vec![0.0; 6],
                geom_type: vec![7],
                geom_dataid: vec![0],
                mesh_vertadr: vec![0],
                mesh_vertnum: vec![vertices as i32],
                mesh_vert: vec![0.0; vertices * 3],
                mesh_polynum: vec![polygons.len() as i32],
                mesh_polyadr: vec![0],
                mesh_polynormal: vec![0.0; polygons.len() * 3],
                mesh_polyvertadr: polyvertadr,
                mesh_polyvertnum: polyvertnum,
                mesh_polyvert: polyvert,
                mesh_polymapadr: polymapadr,
                mesh_polymapnum: polymapnum,
                mesh_polymap: polymap,
            }
        }
        /// 9 项 mesh_poly 全空：无 sidecar，产品行为与改动前一致。
        fn without_sidecar() -> Self {
            let mut fixture = Self::new(0, &[]);
            fixture.mesh_polynum.clear();
            fixture.mesh_polyadr.clear();
            fixture
        }
        fn snapshot(&self) -> CollisionInputsSnapshot<'_> {
            CollisionInputsSnapshot {
                geom_xpos: &self.geom_xpos,
                geom_xmat: &self.geom_xmat,
                geom_size: &self.geom_size,
                geom_margin: &self.geom_margin,
                geom_gap: &self.geom_gap,
                geom_rbound: &self.geom_rbound,
                geom_aabb: &self.geom_aabb,
                geom_type: &self.geom_type,
                geom_dataid: &self.geom_dataid,
                mesh_vertadr: &self.mesh_vertadr,
                mesh_vertnum: &self.mesh_vertnum,
                mesh_graphadr: &[],
                mesh_octadr: &[],
                mesh_vert: &self.mesh_vert,
                mesh_graph: &[],
                mesh_polynum: &self.mesh_polynum,
                mesh_polyadr: &self.mesh_polyadr,
                mesh_polynormal: &self.mesh_polynormal,
                mesh_polyvertadr: &self.mesh_polyvertadr,
                mesh_polyvertnum: &self.mesh_polyvertnum,
                mesh_polyvert: &self.mesh_polyvert,
                mesh_polymapadr: &self.mesh_polymapadr,
                mesh_polymapnum: &self.mesh_polymapnum,
                mesh_polymap: &self.mesh_polymap,
                hfield_size: &[],
                hfield_nrow: &[],
                hfield_ncol: &[],
                hfield_adr: &[],
                hfield_data: &[],
                oct_aabb: &[],
                oct_child: &[],
                oct_coeff: &[],
            }
        }
    }
    fn meta() -> CollisionGpuMeta {
        CollisionGpuMeta {
            nworld: 1,
            ngeom: 1,
            naconmax: 1,
            efc_width: 1,
            ..CollisionGpuMeta::default()
        }
    }
    /// 断言产品入口显式拒绝；返回（维度, actual, limit）列表与 Display 文本。
    fn reject(fixture: &Fixture) -> (Vec<(MeshCapacityDimension, i32, i32)>, String) {
        let error = inputs_from_snapshot(&fixture.snapshot(), meta()).expect_err("产品入口应拒绝");
        let text = error.to_string();
        let CollisionGpuError::MeshMulticontactCapacity {
            violations,
            reject_path,
        } = error
        else {
            panic!("期望容量拒绝");
        };
        assert_eq!(reject_path, MESH_MULTICONTACT_REJECT_PATH);
        let list = violations
            .iter()
            .map(|v| (v.dimension, v.actual, v.limit))
            .collect();
        (list, text)
    }
    #[test]
    fn product_entry_rejects_fan_cone_24_on_both_dimensions() {
        let (vertices, polygons) = parse_obj(FAN_CONE_24_OBJ);
        assert_eq!((vertices, polygons.len()), (25, 25), "fan-cone-24 几何");
        let (violations, text) = reject(&Fixture::new(vertices, &polygons));
        assert_eq!(
            violations,
            vec![
                (MeshCapacityDimension::Face, 24, MESH_MULTICONTACT_FACE_MAX),
                (
                    MeshCapacityDimension::Degree,
                    24,
                    MESH_MULTICONTACT_DEGREE_MAX
                ),
            ],
            "fan-cone-24 两维同时报告"
        );
        for needle in [
            "face",
            "actual=24",
            "limit=20",
            "degree",
            "limit=10",
            "拒绝路径",
        ] {
            assert!(text.contains(needle), "Display 缺 {needle}：{text}");
        }
    }
    #[test]
    fn product_entry_rejects_face_only() {
        let mut polygons: Vec<Vec<usize>> = vec![(0..21).collect()];
        polygons.push(vec![0, 1, 2]);
        let (violations, _) = reject(&Fixture::new(21, &polygons));
        assert_eq!(
            violations,
            vec![(MeshCapacityDimension::Face, 21, MESH_MULTICONTACT_FACE_MAX)]
        );
    }
    #[test]
    fn product_entry_rejects_degree_only() {
        let polygons: Vec<Vec<usize>> = (0..24)
            .map(|index| vec![1 + index, 1 + (index + 1) % 24, 0])
            .collect();
        let (violations, _) = reject(&Fixture::new(25, &polygons));
        assert_eq!(
            violations,
            vec![(
                MeshCapacityDimension::Degree,
                24,
                MESH_MULTICONTACT_DEGREE_MAX
            )]
        );
    }
    #[test]
    fn product_entry_accepts_face_20_degree_10_boundary() {
        let mut polygons: Vec<Vec<usize>> = vec![(0..20).collect()];
        for index in 0..10 {
            polygons.push(vec![index, (index + 1) % 20, 20]);
        }
        assert!(inputs_from_snapshot(&Fixture::new(21, &polygons).snapshot(), meta()).is_ok());
    }
    #[test]
    fn product_entry_accepts_without_mesh_sidecar() {
        assert!(inputs_from_snapshot(&Fixture::without_sidecar().snapshot(), meta()).is_ok());
    }
    #[test]
    fn host_limits_match_device_macros() {
        fn base_macro(source: &str, name: &str) -> i32 {
            source
                .lines()
                .find_map(|line| {
                    let mut parts = line.split_whitespace();
                    (parts.next() == Some("#define") && parts.next() == Some(name))
                        .then(|| parts.next()?.parse::<i32>().ok())
                        .flatten()
                })
                .unwrap_or_else(|| panic!("multicontact_geom.cu 缺少 {name}"))
        }
        assert_eq!(
            base_macro(MULTICONTACT_GEOM_CU, "G5_MC_MESH_FACE_MAX"),
            MESH_MULTICONTACT_FACE_MAX
        );
        assert_eq!(
            base_macro(MULTICONTACT_GEOM_CU, "G5_MC_MESH_DEG_MAX"),
            MESH_MULTICONTACT_DEGREE_MAX
        );
    }
}
