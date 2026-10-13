//! 原生几何字段的拥有式宿主转换。
//! 调用方提供快照；本模块不捕获原生指针。

use super::convert_f64_to_f32_into;
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{
        NativeModelInfo, PrimitiveGeometryFields, PrimitiveGeometryInput, collision::check_lengths,
    },
};

/// 保留原生f64与i32的十三字段快照。
/// 本类型不是新增C ABI或完整模型。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePrimitiveGeometrySnapshot {
    pub info: NativeModelInfo,
    pub geom_type: Vec<i32>,
    pub geom_bodyid: Vec<i32>,
    pub geom_contype: Vec<i32>,
    pub geom_conaffinity: Vec<i32>,
    pub geom_condim: Vec<i32>,
    pub geom_priority: Vec<i32>,
    pub geom_solmix: Vec<f64>,
    pub geom_solref: Vec<f64>,
    pub geom_solimp: Vec<f64>,
    pub geom_size: Vec<f64>,
    pub geom_friction: Vec<f64>,
    pub geom_margin: Vec<f64>,
    pub geom_gap: Vec<f64>,
}

impl NativePrimitiveGeometrySnapshot {
    /// 检查ABI与全部长度后一次量化。
    /// 失败时不发布部分转换结果。
    pub fn into_input(self) -> Result<PrimitiveGeometryInput, NativeProbeError> {
        self.info.validate()?;
        let n = usize::try_from(self.info.ngeom)
            .map_err(|_| InputError::Overflow { field: "ngeom" })?;
        let nbody = usize::try_from(self.info.nbody)
            .map_err(|_| InputError::Overflow { field: "nbody" })?;
        if n > i32::MAX as usize / 5 || nbody > i32::MAX as usize {
            return Err(InputError::Overflow {
                field: "primitive_geometry",
            }
            .into());
        }
        check_lengths(
            n,
            [
                ("geom_type", self.geom_type.len(), 1),
                ("geom_bodyid", self.geom_bodyid.len(), 1),
                ("geom_contype", self.geom_contype.len(), 1),
                ("geom_conaffinity", self.geom_conaffinity.len(), 1),
                ("geom_condim", self.geom_condim.len(), 1),
                ("geom_priority", self.geom_priority.len(), 1),
                ("geom_solmix", self.geom_solmix.len(), 1),
                ("geom_solref", self.geom_solref.len(), 2),
                ("geom_solimp", self.geom_solimp.len(), 5),
                ("geom_size", self.geom_size.len(), 3),
                ("geom_friction", self.geom_friction.len(), 3),
                ("geom_margin", self.geom_margin.len(), 1),
                ("geom_gap", self.geom_gap.len(), 1),
            ],
        )?;
        Ok(PrimitiveGeometryInput::new(
            nbody,
            PrimitiveGeometryFields {
                geom_type: self.geom_type,
                geom_bodyid: self.geom_bodyid,
                geom_contype: self.geom_contype,
                geom_conaffinity: self.geom_conaffinity,
                geom_condim: self.geom_condim,
                geom_priority: self.geom_priority,
                geom_solmix: convert("geom_solmix", &self.geom_solmix)?,
                geom_solref: convert("geom_solref", &self.geom_solref)?,
                geom_solimp: convert("geom_solimp", &self.geom_solimp)?,
                geom_size: convert("geom_size", &self.geom_size)?,
                geom_friction: convert("geom_friction", &self.geom_friction)?,
                geom_margin: convert("geom_margin", &self.geom_margin)?,
                geom_gap: convert("geom_gap", &self.geom_gap)?,
            },
        )?)
    }
}

fn convert(field: &'static str, values: &[f64]) -> Result<Vec<f32>, NativeProbeError> {
    // 默认功能不引入原生构建；沿用既有可失败分配模式。
    let bytes = values
        .len()
        .checked_mul(size_of::<f32>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(InputError::Overflow { field })?;
    let mut target = Vec::new();
    target
        .try_reserve_exact(values.len())
        .map_err(|_| NativeProbeError::HostAllocation { bytes })?;
    target.resize(values.len(), 0.0_f32);
    convert_f64_to_f32_into(values, &mut target).map_err(|error| match error {
        InputError::NonFinite { index, .. } => InputError::NonFinite { field, index },
        InputError::ScalarOverflow { index, .. } => InputError::ScalarOverflow { field, index },
        other => other,
    })?;
    Ok(target)
}
