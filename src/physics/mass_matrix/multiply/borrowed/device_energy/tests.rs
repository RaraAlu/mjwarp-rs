use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn raw(buffer: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; buffer.len()];
    buffer.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}

mod core;
#[cfg(feature = "cuda-probe")]
mod gpu;
