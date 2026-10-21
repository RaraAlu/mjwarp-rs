//! 相机与光源的已检查字段。
//! 跟踪常量来自已编译模型。

use crate::diagnostics::InputError;

/// 共享拓扑与十项浮点字段。
/// 模式依次采用原生值0至4。
/// 负目标引用采用固定回退。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CamLightFields {
    pub cam_mode: Vec<i32>,
    pub cam_bodyid: Vec<i32>,
    pub cam_targetbodyid: Vec<i32>,
    pub cam_pos: Vec<f32>,
    /// 局部姿态，顺序wxyz。
    pub cam_quat: Vec<f32>,
    /// 初始位置减子树质心。
    pub cam_poscom0: Vec<f32>,
    /// 初始位置减体世界位置。
    pub cam_pos0: Vec<f32>,
    /// 初始世界姿态，按行展开。
    pub cam_mat0: Vec<f32>,
    pub light_mode: Vec<i32>,
    pub light_bodyid: Vec<i32>,
    pub light_targetbodyid: Vec<i32>,
    pub light_pos: Vec<f32>,
    /// 局部方向，允许零与非单位值。
    pub light_dir: Vec<f32>,
    pub light_poscom0: Vec<f32>,
    pub light_pos0: Vec<f32>,
    /// 初始世界方向。
    pub light_dir0: Vec<f32>,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CamLightParameter {
    CamPos,
    CamQuat,
    CamPoscom0,
    CamPos0,
    CamMat0,
    LightPos,
    LightDir,
    LightPoscom0,
    LightPos0,
    LightDir0,
}

impl CamLightParameter {
    pub const ALL: [Self; 10] = [
        Self::CamPos,
        Self::CamQuat,
        Self::CamPoscom0,
        Self::CamPos0,
        Self::CamMat0,
        Self::LightPos,
        Self::LightDir,
        Self::LightPoscom0,
        Self::LightPos0,
        Self::LightDir0,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CamPos => "cam_pos",
            Self::CamQuat => "cam_quat",
            Self::CamPoscom0 => "cam_poscom0",
            Self::CamPos0 => "cam_pos0",
            Self::CamMat0 => "cam_mat0",
            Self::LightPos => "light_pos",
            Self::LightDir => "light_dir",
            Self::LightPoscom0 => "light_poscom0",
            Self::LightPos0 => "light_pos0",
            Self::LightDir0 => "light_dir0",
        }
    }

    pub(crate) fn shared(self, fields: &CamLightFields) -> &[f32] {
        match self {
            Self::CamPos => &fields.cam_pos,
            Self::CamQuat => &fields.cam_quat,
            Self::CamPoscom0 => &fields.cam_poscom0,
            Self::CamPos0 => &fields.cam_pos0,
            Self::CamMat0 => &fields.cam_mat0,
            Self::LightPos => &fields.light_pos,
            Self::LightDir => &fields.light_dir,
            Self::LightPoscom0 => &fields.light_poscom0,
            Self::LightPos0 => &fields.light_pos0,
            Self::LightDir0 => &fields.light_dir0,
        }
    }

    fn width(self, nc: usize, nl: usize) -> usize {
        match self {
            Self::CamQuat => 4 * nc,
            Self::CamMat0 => 9 * nc,
            Self::CamPos | Self::CamPoscom0 | Self::CamPos0 => 3 * nc,
            _ => 3 * nl,
        }
    }
}

/// 检查共享拓扑与全部字段。
/// 不重算模型跟踪常量。
///
/// ```compile_fail
/// use mjwarp_rs::model::CamLightModelInput;
/// fn mutate(model: &mut CamLightModelInput) {
///     model.fields().cam_mode[0] = 1;
/// }
/// ```
fn check_length(actual: usize, expected: usize, field: &'static str) -> Result<(), InputError> {
    if actual != expected {
        return Err(InputError::LengthMismatch {
            field,
            actual,
            expected,
        });
    }
    Ok(())
}

fn check_values(field: CamLightParameter, values: &[f32]) -> Result<(), InputError> {
    check_values_mode(field, values, true)
}
fn check_values_mode(
    field: CamLightParameter,
    values: &[f32],
    strict: bool,
) -> Result<(), InputError> {
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: field.name(),
                index,
            });
        }
    }
    if strict && field == CamLightParameter::CamQuat {
        for (index, q) in values.as_chunks::<4>().0.iter().enumerate() {
            let squared: f64 = q.iter().map(|&v| f64::from(v).powi(2)).sum();
            if (squared - 1.0).abs() > 2e-6 {
                return Err(InputError::InvalidTopology {
                    field: field.name(),
                    index,
                    reason: "nonunit_model_rotation",
                });
            }
        }
    }
    Ok(())
}

#[path = "camlight/model.rs"]
mod model;
#[path = "camlight/parameters.rs"]
mod parameters;
#[cfg(test)]
#[path = "camlight/tests.rs"]
mod tests;

pub use model::CamLightModelInput;
pub use parameters::CamLightParameters;
