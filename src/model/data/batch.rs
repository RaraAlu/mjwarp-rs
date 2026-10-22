//! A1-1/A1-5：世界批量、共享参数周期与设备批量字段。
//! 世界批量独立推进；参数批量按周期共享。
//! 拓扑与映射只存一份，不随世界复制。

use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::model::data::layout::DeviceDataLayout;
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

/// 世界批量下标契约。
///
/// 只记录世界数并检查世界序号。
/// 不持有宿主字段或设备资源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldBatch {
    worlds: usize,
}

impl WorldBatch {
    pub fn new(worlds: usize) -> Result<Self, InputError> {
        if worlds == 0 {
            return Err(InputError::InvalidDimension { field: "worlds" });
        }
        Ok(Self { worlds })
    }

    pub fn worlds(self) -> usize {
        self.worlds
    }

    pub fn check(self, world: usize) -> Result<usize, InputError> {
        if world >= self.worlds {
            return Err(InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: self.worlds,
            });
        }
        Ok(world)
    }
}

/// 按周期共享的参数批量。
///
/// 批量下标等于世界序号对周期取模。
/// 周期为零一律拒绝。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldBatch {
    period: usize,
}

impl FieldBatch {
    pub fn new(period: usize) -> Result<Self, InputError> {
        if period == 0 {
            return Err(InputError::InvalidDimension {
                field: "parameter_period",
            });
        }
        Ok(Self { period })
    }

    pub fn period(self) -> usize {
        self.period
    }

    pub fn sample(self, world: usize) -> usize {
        world % self.period
    }

    /// 先检查世界范围，再取共享参数下标。
    pub fn sample_index(self, worlds: WorldBatch, world: usize) -> Result<usize, InputError> {
        Ok(self.sample(worlds.check(world)?))
    }
}

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

    /// 共享布局描述；守卫跨度契约见`DeviceDataLayout`。
    pub fn data_layout(&self) -> DeviceDataLayout {
        DeviceDataLayout::from_batch(self.layout)
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
    fn shares_parameter_rows_by_world_period() {
        let batch = FieldBatch::new(3).unwrap();
        assert_eq!(batch.period(), 3);
        assert_eq!(
            [0, 1, 2, 3, 4, 5, 6].map(|world| batch.sample(world)),
            [0, 1, 2, 0, 1, 2, 0]
        );
        assert!(FieldBatch::new(0).is_err());
    }

    #[test]
    fn checks_world_range_before_sampling() {
        let worlds = WorldBatch::new(4).unwrap();
        let batch = FieldBatch::new(2).unwrap();
        assert_eq!(worlds.worlds(), 4);
        assert_eq!(worlds.check(1).unwrap(), 1);
        assert!(matches!(
            worlds.check(4),
            Err(InputError::InvalidIndex {
                field: "world",
                index: 4,
                limit: 4
            })
        ));
        assert_eq!(batch.sample_index(worlds, 3).unwrap(), 1);
        assert!(batch.sample_index(worlds, 4).is_err());
        assert!(WorldBatch::new(0).is_err());
    }

    #[test]
    fn validates_field_element_width_and_length() {
        let layout = BatchLayout::new(2, 3, 4).unwrap();
        check_field_layout::<f32>(layout, 6).unwrap();
        check_field_layout::<i32>(layout, 6).unwrap();
        assert!(matches!(
            check_field_layout::<f64>(layout, 6),
            Err(InputError::InvalidDimension {
                field: "element_bytes"
            })
        ));
        assert!(matches!(
            check_field_layout::<f32>(layout, 5),
            Err(InputError::LengthMismatch {
                field: "source",
                ..
            })
        ));
    }

    #[test]
    fn exposes_shared_layout_descriptor() {
        let layout = BatchLayout::new(2, 3, 4).unwrap();
        let descriptor = DeviceDataLayout::from_batch(layout);
        assert_eq!(descriptor.worlds(), 2);
        assert_eq!(descriptor.elements_per_world(), 3);
        assert_eq!(descriptor.guarded_span_elements().unwrap(), 14);
    }
}
