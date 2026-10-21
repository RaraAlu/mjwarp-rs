//! 设备输出分配与回读。

#[cfg(any(feature = "cuda-probe", test))]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn allocate_output(
    session: &TransferSession,
    length: usize,
) -> Result<TransferBuffer<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(length)?;
    values.fill(-131072.0);
    values[4..length - 4].fill(f32::NAN);
    session.upload(&values)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn read_flex_validity(
    buffer: &TransferBuffer<i32>,
    length: usize,
) -> Result<Vec<bool>, TransferError> {
    let values = read_output(buffer, length, "resident_flex_hessian_valid")?;
    decode_flex_validity(&values[4..length - 4])
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn decode_flex_validity(values: &[i32]) -> Result<Vec<bool>, TransferError> {
    for (index, &value) in values.iter().enumerate() {
        if value != 0 && value != 1 {
            return Err(InputError::InvalidTopology {
                field: "resident_flex_hessian_valid",
                index,
                reason: "expected 0 or 1",
            }
            .into());
        }
    }
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact(values.len())
        .map_err(|_| TransferError::HostAllocation {
            bytes: values.len(),
        })?;
    decoded.extend(values.iter().map(|&v| v == 1));
    Ok(decoded)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn allocate_integer_output(
    session: &TransferSession,
    length: usize,
) -> Result<TransferBuffer<i32>, TransferError> {
    let mut values = crate::runtime::host_staging::<i32>(length)?;
    values.fill(-131072);
    values[4..length - 4].fill(i32::MAX);
    session.upload(&values)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn read_output<T: crate::runtime::TransferElement + Into<f64>>(
    buffer: &TransferBuffer<T>,
    length: usize,
    field: &'static str,
) -> Result<Vec<T>, TransferError> {
    let mut values = crate::runtime::host_staging::<T>(length)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, field)?;
    Ok(values)
}
