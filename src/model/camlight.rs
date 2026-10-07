//! 相机与光源的严格字段子集。
//! 跟踪常量来自已编译模型。

use super::{MocapModelInput, ParameterBatch};
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
#[derive(Clone, Debug, PartialEq)]
pub struct CamLightModelInput {
    mocap: MocapModelInput,
    fields: CamLightFields,
}

impl CamLightModelInput {
    pub fn new(mocap: MocapModelInput, fields: CamLightFields) -> Result<Self, InputError> {
        let nb = mocap.attached().rigid().kinematics().nbody();
        for (mode, body, target, names) in [
            (
                &fields.cam_mode,
                &fields.cam_bodyid,
                &fields.cam_targetbodyid,
                ["cam_mode", "cam_bodyid", "cam_targetbodyid"],
            ),
            (
                &fields.light_mode,
                &fields.light_bodyid,
                &fields.light_targetbodyid,
                ["light_mode", "light_bodyid", "light_targetbodyid"],
            ),
        ] {
            if body.len() > i32::MAX as usize / 12 {
                return Err(InputError::Overflow {
                    field: "camlight_fields",
                });
            }
            for (ids, name) in [(mode, names[0]), (target, names[2])] {
                check_length(ids.len(), body.len(), name)?;
            }
            for index in 0..body.len() {
                for (invalid, field, reason) in [
                    (
                        !(0..=4).contains(&mode[index]),
                        names[0],
                        "unsupported_camlight_mode",
                    ),
                    (
                        body[index] < 0 || body[index] as usize >= nb,
                        names[1],
                        "body_reference_out_of_range",
                    ),
                    (
                        target[index] >= 0 && target[index] as usize >= nb,
                        names[2],
                        "target_reference_out_of_range",
                    ),
                ] {
                    if invalid {
                        return Err(InputError::InvalidTopology {
                            field,
                            index,
                            reason,
                        });
                    }
                }
            }
        }
        let model = Self { mocap, fields };
        for field in CamLightParameter::ALL {
            let values = field.shared(model.fields());
            check_length(
                values.len(),
                field.width(model.ncam(), model.nlight()),
                field.name(),
            )?;
            check_values(field, values)?;
        }
        Ok(model)
    }

    pub fn mocap(&self) -> &MocapModelInput {
        &self.mocap
    }
    pub fn fields(&self) -> &CamLightFields {
        &self.fields
    }
    pub fn ncam(&self) -> usize {
        self.fields.cam_bodyid.len()
    }
    pub fn nlight(&self) -> usize {
        self.fields.light_bodyid.len()
    }
    pub(crate) fn into_parts(self) -> (MocapModelInput, CamLightFields) {
        (self.mocap, self.fields)
    }
}

/// 十项独立周期的可选覆盖。
/// 空字段仍要求正周期。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CamLightParameters {
    fields: [Option<ParameterBatch>; 10],
}

impl CamLightParameters {
    pub fn set(&mut self, field: CamLightParameter, batch: ParameterBatch) {
        self.fields[field as usize] = Some(batch);
    }
    pub fn get(&self, field: CamLightParameter) -> Option<&ParameterBatch> {
        self.fields[field as usize].as_ref()
    }
    pub(crate) fn validate(&self, model: &CamLightModelInput) -> Result<(), InputError> {
        for field in CamLightParameter::ALL {
            if let Some(batch) = self.get(field) {
                check_length(
                    batch.row_elements(),
                    field.shared(model.fields()).len(),
                    field.name(),
                )?;
                check_values(field, batch.values())?;
            }
        }
        Ok(())
    }
    #[cfg(feature = "cuda-probe")]
    pub(crate) fn values<'a>(
        &'a self,
        field: CamLightParameter,
        fields: &'a CamLightFields,
    ) -> &'a [f32] {
        self.get(field)
            .map_or_else(|| field.shared(fields), ParameterBatch::values)
    }
    #[cfg(feature = "cuda-probe")]
    pub(crate) fn batches(&self, field: CamLightParameter) -> usize {
        self.get(field).map_or(1, ParameterBatch::batches)
    }
}

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
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: field.name(),
                index,
            });
        }
    }
    if field == CamLightParameter::CamQuat {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
        KinematicModelInput,
    };

    fn base() -> MocapModelInput {
        let k = KinematicModelInput::new(
            0,
            KinematicFields {
                body_parentid: vec![0],
                body_jntadr: vec![-1],
                body_jntnum: vec![0],
                body_pos: vec![0.0; 3],
                body_quat: vec![1.0, 0.0, 0.0, 0.0],
                ..Default::default()
            },
        )
        .unwrap();
        let i = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 3],
                body_iquat: vec![1.0, 0.0, 0.0, 0.0],
                body_mass: vec![0.0],
                body_inertia: vec![0.0; 3],
                ..Default::default()
            },
        )
        .unwrap();
        MocapModelInput::new(
            AttachedModelInput::new(i, AttachedFields::default()).unwrap(),
            vec![-1],
        )
        .unwrap()
    }
    fn fields() -> CamLightFields {
        CamLightFields {
            cam_mode: vec![0, 1, 2, 3, 4],
            cam_bodyid: vec![0; 5],
            cam_targetbodyid: vec![-2; 5],
            cam_pos: vec![-0.0; 15],
            cam_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
            cam_pos0: vec![0.0; 15],
            cam_poscom0: vec![0.0; 15],
            cam_mat0: vec![0.0; 45],
            light_mode: vec![0, 1, 2, 3, 4],
            light_bodyid: vec![0; 5],
            light_targetbodyid: vec![-2; 5],
            light_pos: vec![0.0; 15],
            light_dir: [0.0, 0.0, 2.0].repeat(5),
            light_pos0: vec![0.0; 15],
            light_poscom0: vec![0.0; 15],
            light_dir0: vec![0.0; 15],
        }
    }
    fn field_mut(fields: &mut CamLightFields, f: CamLightParameter) -> &mut Vec<f32> {
        match f {
            CamLightParameter::CamPos => &mut fields.cam_pos,
            CamLightParameter::CamQuat => &mut fields.cam_quat,
            CamLightParameter::CamPos0 => &mut fields.cam_pos0,
            CamLightParameter::CamPoscom0 => &mut fields.cam_poscom0,
            CamLightParameter::CamMat0 => &mut fields.cam_mat0,
            CamLightParameter::LightPos => &mut fields.light_pos,
            CamLightParameter::LightDir => &mut fields.light_dir,
            CamLightParameter::LightPos0 => &mut fields.light_pos0,
            CamLightParameter::LightPoscom0 => &mut fields.light_poscom0,
            CamLightParameter::LightDir0 => &mut fields.light_dir0,
        }
    }
    #[test]
    fn preserves_all_modes_negative_targets_and_independent_periods() {
        let fields = fields();
        let model = CamLightModelInput::new(base(), fields.clone()).unwrap();
        assert_eq!((model.ncam(), model.nlight()), (5, 5));
        assert_eq!(model.fields(), &fields);
        assert_eq!(model.fields().cam_pos[0].to_bits(), (-0.0f32).to_bits());
        assert_eq!(model.clone(), model);
        let mut p = CamLightParameters::default();
        for (i, f) in CamLightParameter::ALL.into_iter().enumerate() {
            let shared = f.shared(&fields);
            p.set(
                f,
                ParameterBatch::new(i + 2, shared.len(), shared.repeat(i + 2)).unwrap(),
            );
            assert_eq!(p.get(f).unwrap().world(513), shared);
        }
        p.validate(&model).unwrap();
        let empty = CamLightModelInput::new(base(), CamLightFields::default()).unwrap();
        for f in CamLightParameter::ALL {
            p.set(
                f,
                ParameterBatch::new(i32::MAX as usize, 0, vec![]).unwrap(),
            );
        }
        p.validate(&empty).unwrap();
    }
    #[test]
    fn rejects_every_float_length_nonfinite_and_nonunit_rotation() {
        for f in CamLightParameter::ALL {
            let mut short = fields();
            field_mut(&mut short, f).pop();
            assert!(
                matches!(CamLightModelInput::new(base(),short),Err(InputError::LengthMismatch{field,..}) if field==f.name())
            );
            for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut fields = fields();
                *field_mut(&mut fields, f).last_mut().unwrap() = bad;
                assert!(
                    matches!(CamLightModelInput::new(base(),fields),Err(InputError::NonFinite{field,..}) if field==f.name())
                );
            }
        }
        for bad in [0.0, 2.0] {
            let mut fields = fields();
            fields.cam_quat[0] = bad;
            assert!(matches!(
                CamLightModelInput::new(base(), fields),
                Err(InputError::InvalidTopology {
                    field: "cam_quat",
                    ..
                })
            ));
        }
        let model = CamLightModelInput::new(base(), fields()).unwrap();
        let mut p = CamLightParameters::default();
        for f in CamLightParameter::ALL {
            p.set(f, ParameterBatch::new(1, 0, vec![]).unwrap());
            assert!(p.validate(&model).is_err());
            p = CamLightParameters::default();
        }
        let mut q = model.fields().cam_quat.repeat(7);
        *q.last_mut().unwrap() = 2.0;
        p.set(
            CamLightParameter::CamQuat,
            ParameterBatch::new(7, 20, q).unwrap(),
        );
        assert!(p.validate(&model).is_err());
    }
    #[test]
    fn rejects_topology_lengths_modes_and_unsafe_references() {
        for column in 0..6 {
            let mut f = fields();
            let v = match column {
                0 => &mut f.cam_mode,
                1 => &mut f.cam_targetbodyid,
                2 => &mut f.light_mode,
                3 => &mut f.light_targetbodyid,
                4 => &mut f.cam_bodyid,
                _ => &mut f.light_bodyid,
            };
            v.pop();
            assert!(CamLightModelInput::new(base(), f).is_err());
        }
        for column in 0..6 {
            for bad in [-1, 5, i32::MAX] {
                let mut f = fields();
                let v = match column {
                    0 => &mut f.cam_mode,
                    1 => &mut f.light_mode,
                    2 => &mut f.cam_bodyid,
                    3 => &mut f.light_bodyid,
                    4 => &mut f.cam_targetbodyid,
                    _ => &mut f.light_targetbodyid,
                };
                v[0] = bad;
                assert_eq!(
                    CamLightModelInput::new(base(), f).is_err(),
                    column < 4 || bad >= 0
                );
            }
        }
    }
}
