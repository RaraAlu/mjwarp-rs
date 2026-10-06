//! 宿主转换与连续GPU字段交换。
//! 不提供模型上传入口。

use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

/// 连续批量字段的GPU传输辅助。
/// 不解释物理字段或模型身份。
/// 不替代Model、Data或put_data。
///
/// ```no_run
/// use mjwarp_rs::{diagnostics::TransferError, io::DeviceBatch,
///     model::BatchLayout, runtime::TransferSession};
/// let session = TransferSession::new(0)?;
/// let layout = BatchLayout::new(2, 3, 4)?;
/// let mut field = DeviceBatch::upload(&session, layout, &[0.0_f32; 6])?;
/// field.write_world(1, &[1.0, 2.0, 3.0])?;
/// let mut world = [0.0; 3];
/// field.read_world_into(1, &mut world)?;
/// # Ok::<(), TransferError>(())
/// ```
pub struct DeviceBatch<T: TransferElement> {
    layout: BatchLayout,
    buffer: TransferBuffer<T>,
}

impl<T: TransferElement> DeviceBatch<T> {
    pub fn upload(
        session: &TransferSession,
        layout: BatchLayout,
        source: &[T],
    ) -> Result<Self, TransferError> {
        check_field_layout::<T>(layout, source.len())?;
        let buffer = session.upload(source)?;
        Ok(Self { layout, buffer })
    }

    pub fn layout(&self) -> BatchLayout {
        self.layout
    }

    pub fn read_into(&self, target: &mut [T]) -> Result<(), TransferError> {
        check_length("target", self.layout.total_elements(), target.len())?;
        self.buffer.read_range_into(0, target)
    }

    pub fn read_world_into(&self, world: usize, target: &mut [T]) -> Result<(), TransferError> {
        let range = self.layout.world_elements(world)?;
        check_length("target", range.len(), target.len())?;
        self.buffer.read_range_into(range.start, target)
    }

    pub fn write_world(&mut self, world: usize, source: &[T]) -> Result<(), TransferError> {
        let range = self.layout.world_elements(world)?;
        check_length("source", range.len(), source.len())?;
        self.buffer.write_range(range.start, source)
    }

    pub fn copy_world_from(
        &mut self,
        world: usize,
        source: &Self,
        source_world: usize,
    ) -> Result<(), TransferError> {
        let target = self.layout.world_elements(world)?;
        let input = source.layout.world_elements(source_world)?;
        check_length("source_world", target.len(), input.len())?;
        self.buffer
            .copy_range_from(target.start, &source.buffer, input.start, target.len())
    }

    pub fn copy_world_within(
        &mut self,
        world: usize,
        source_world: usize,
    ) -> Result<(), TransferError> {
        let target = self.layout.world_elements(world)?;
        let source = self.layout.world_elements(source_world)?;
        self.buffer
            .copy_range_within(target.start, source.start, target.len())
    }
}

/// 先执行严格宿主转换再上传。
/// 该入口不替代等价状态导入。
pub fn upload_f64_batch(
    session: &TransferSession,
    layout: BatchLayout,
    source: &[f64],
) -> Result<DeviceBatch<f32>, TransferError> {
    check_field_layout::<f32>(layout, source.len())?;
    let mut values = crate::runtime::host_staging::<f32>(source.len())?;
    convert_f64_to_f32_into(source, &mut values)?;
    DeviceBatch::upload(session, layout, &values)
}

fn check_field_layout<T: TransferElement>(
    layout: BatchLayout,
    elements: usize,
) -> Result<(), InputError> {
    if layout.element_bytes() != size_of::<T>() {
        return Err(InputError::InvalidDimension {
            field: "element_bytes",
        });
    }
    check_length("source", layout.total_elements(), elements)
}

/// 先检查全量输入，再转换。
/// 拒绝非有限值与f32溢出。
/// 允许舍入、下溢和负零。
/// 此严格辅助接口不替代put_data。
pub fn convert_f64_to_f32_into(source: &[f64], target: &mut [f32]) -> Result<(), InputError> {
    check_length("target", source.len(), target.len())?;
    for (index, &value) in source.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "source",
                index,
            });
        }
        if !(value as f32).is_finite() {
            return Err(InputError::ScalarOverflow {
                field: "source",
                index,
            });
        }
    }
    for (&value, output) in source.iter().zip(target) {
        *output = value as f32;
    }
    Ok(())
}

/// 复制一个世界的连续f32字段。
/// 保留全部位模式。
/// 任意输入错误都不改写目标。
pub fn copy_world_f32(
    layout: BatchLayout,
    world: usize,
    source: &[f32],
    target: &mut [f32],
) -> Result<(), InputError> {
    if layout.element_bytes() != size_of::<f32>() {
        return Err(InputError::InvalidDimension {
            field: "element_bytes",
        });
    }
    let range = layout.world_elements(world)?;
    check_length("source", layout.elements_per_world(), source.len())?;
    check_length("target", layout.total_elements(), target.len())?;
    target[range].copy_from_slice(source);
    Ok(())
}

fn check_length(field: &'static str, expected: usize, actual: usize) -> Result<(), InputError> {
    if expected != actual {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_three_device_field_types_and_empty_layouts() {
        let layout = BatchLayout::new(3, 5, 4).unwrap();
        check_field_layout::<f32>(layout, 15).unwrap();
        check_field_layout::<i32>(layout, 15).unwrap();
        check_field_layout::<u32>(layout, 15).unwrap();
        check_field_layout::<f32>(BatchLayout::new(2, 0, 4).unwrap(), 0).unwrap();
    }

    #[test]
    fn rejects_device_field_size_and_length() {
        assert_eq!(
            check_field_layout::<f32>(BatchLayout::new(2, 3, 8).unwrap(), 6),
            Err(InputError::InvalidDimension {
                field: "element_bytes"
            })
        );
        assert_eq!(
            check_field_layout::<u32>(BatchLayout::new(2, 3, 4).unwrap(), 5),
            Err(InputError::LengthMismatch {
                field: "source",
                expected: 6,
                actual: 5
            })
        );
    }

    #[test]
    fn converts_with_rounding_underflow_and_negative_zero() {
        let source = [
            1.5,
            -0.0,
            f64::MIN_POSITIVE,
            f64::from(f32::MAX),
            16_777_217.0,
        ];
        let mut target = [9.0; 5];
        convert_f64_to_f32_into(&source, &mut target).unwrap();
        assert_eq!(target, [1.5, -0.0, 0.0, f32::MAX, 16_777_216.0]);
        assert_eq!(target[1].to_bits(), (-0.0_f32).to_bits());
        convert_f64_to_f32_into(&[], &mut []).unwrap();
    }

    #[test]
    fn rejects_non_finite_inputs_without_partial_writes() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut target = [7.0; 3];
            assert_eq!(
                convert_f64_to_f32_into(&[1.0, 2.0, invalid], &mut target),
                Err(InputError::NonFinite {
                    field: "source",
                    index: 2
                })
            );
            assert_eq!(target, [7.0; 3]);
        }
    }

    #[test]
    fn rejects_conversion_overflow_without_partial_writes() {
        for invalid in [f64::MAX, -f64::MAX] {
            let mut target = [7.0; 2];
            assert_eq!(
                convert_f64_to_f32_into(&[1.0, invalid], &mut target),
                Err(InputError::ScalarOverflow {
                    field: "source",
                    index: 1
                })
            );
            assert_eq!(target, [7.0; 2]);
        }
    }

    #[test]
    fn rejects_conversion_length_without_writes() {
        let mut target = [7.0];
        assert_eq!(
            convert_f64_to_f32_into(&[1.0, 2.0], &mut target),
            Err(InputError::LengthMismatch {
                field: "target",
                expected: 2,
                actual: 1
            })
        );
        assert_eq!(target, [7.0]);
    }

    #[test]
    fn copies_one_world_and_preserves_bits() {
        let layout = BatchLayout::new(3, 2, 4).unwrap();
        let source = [f32::from_bits(0x7fc0_0017), -0.0];
        let mut target = [7.0; 6];
        copy_world_f32(layout, 1, &source, &mut target).unwrap();
        assert_eq!(&target[..2], &[7.0; 2]);
        assert_eq!(&target[4..], &[7.0; 2]);
        assert_eq!(target[2].to_bits(), source[0].to_bits());
        assert_eq!(target[3].to_bits(), source[1].to_bits());
        copy_world_f32(BatchLayout::new(2, 0, 4).unwrap(), 1, &[], &mut []).unwrap();
    }

    #[test]
    fn rejects_copy_inputs_without_writes() {
        let mut target = [7.0; 4];
        for (layout, world, source) in [
            (BatchLayout::new(2, 2, 8).unwrap(), 0, &[1.0, 2.0][..]),
            (BatchLayout::new(2, 2, 4).unwrap(), 2, &[1.0, 2.0][..]),
            (BatchLayout::new(2, 2, 4).unwrap(), 1, &[1.0][..]),
            (BatchLayout::new(2, 3, 4).unwrap(), 1, &[1.0, 2.0, 3.0][..]),
        ] {
            assert!(copy_world_f32(layout, world, source, &mut target).is_err());
            assert_eq!(target, [7.0; 4]);
        }
    }
}
