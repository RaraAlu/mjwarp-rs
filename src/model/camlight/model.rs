//! 相机与光源模型输入。

use super::*;
use crate::diagnostics::InputError;
use crate::model::MocapModelInput;

#[derive(Clone, Debug, PartialEq)]
pub struct CamLightModelInput {
    mocap: MocapModelInput,
    fields: CamLightFields,
}

impl CamLightModelInput {
    pub fn new(mocap: MocapModelInput, fields: CamLightFields) -> Result<Self, InputError> {
        Self::checked(mocap, fields, true)
    }

    /// 检查布局而不要求单位姿态。
    pub fn new_equivalent(
        mocap: MocapModelInput,
        fields: CamLightFields,
    ) -> Result<Self, InputError> {
        Self::checked(mocap, fields, false)
    }

    fn checked(
        mocap: MocapModelInput,
        fields: CamLightFields,
        strict: bool,
    ) -> Result<Self, InputError> {
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
            if strict {
                check_values(field, values)?;
            } else {
                check_values_mode(field, values, false)?;
            }
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
