//! 动能向量形状与标签检查。

use super::*;

pub(super) fn check(v: &[f32], expected: usize, field: &'static str) -> Result<(), InputError> {
    if v.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: v.len(),
        });
    }
    if let Some(index) = v.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}
pub(super) fn check_pair(
    s: &Spec,
    count: usize,
    q: KineticEnergyVector<'_>,
    m: KineticEnergyVector<'_>,
) -> Result<(), TransferError> {
    if q.model_id != s.model_id || m.model_id != s.model_id {
        return Err(TransferError::ModelMismatch);
    }
    if q.state_id != m.state_id {
        return Err(InputError::InvalidTopology {
            field: "kinetic_energy_pair",
            index: 0,
            reason: "state_mismatch",
        }
        .into());
    }
    check(q.values, count, "qvel")?;
    check(m.values, count, "Mqvel")?;
    Ok(())
}
