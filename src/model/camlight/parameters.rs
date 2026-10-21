//! 相机与光源独立参数批。

use super::*;
use crate::diagnostics::InputError;
use crate::model::ParameterBatch;

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
        self.validate_mode(model, true)
    }
    pub(crate) fn validate_mode(
        &self,
        model: &CamLightModelInput,
        strict: bool,
    ) -> Result<(), InputError> {
        for field in CamLightParameter::ALL {
            if let Some(batch) = self.get(field) {
                check_length(
                    batch.row_elements(),
                    field.shared(model.fields()).len(),
                    field.name(),
                )?;
                check_values_mode(field, batch.values(), strict)?;
            } else {
                check_values_mode(field, field.shared(model.fields()), strict)?;
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
