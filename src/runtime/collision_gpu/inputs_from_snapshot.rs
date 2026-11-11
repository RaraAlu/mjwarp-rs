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
/// 随后执行**方案 C 的产品路径容量门**：mesh 多接触设备固定容量（A2 起 face≤32 / degree≤32）超界时
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
    // 方案 C（产品路径容量门）：设备 mesh 多接触是固定容量（A2 起 face≤32 / degree≤32），超界时
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
#[path = "inputs_from_snapshot_tests.rs"]
mod tests;
