//! 设备数值与守卫检查。

#[cfg(any(feature = "cuda-probe", test))]
use crate::diagnostics::{InputError, ProbeError, TransferError};

#[cfg(any(feature = "cuda-probe", test))]
pub(in crate::physics) fn check_device_values<T: Copy + Into<f64>>(
    values: &[T],
    field: &'static str,
) -> Result<(), TransferError> {
    check_device_guards(values)?;
    for (index, value) in values[4..values.len() - 4].iter().enumerate() {
        if !(*value).into().is_finite() {
            return Err(InputError::NonFinite { field, index }.into());
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
pub(in crate::physics) fn check_device_guards<T: Copy + Into<f64>>(
    values: &[T],
) -> Result<(), TransferError> {
    // Private callers supply a checked payload layout and two four-value guards.
    debug_assert!(values.len() >= 8);
    for index in (0..4).chain(values.len() - 4..values.len()) {
        if values[index].into() != -131072.0 {
            return Err(ProbeError::Mismatch {
                index,
                expected: -131072.0,
                actual: values[index].into() as f32,
            }
            .into());
        }
    }
    Ok(())
}
