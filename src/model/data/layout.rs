//! A1-1/A2：设备数据布局契约。
//! 批量布局按世界连续展开。
//! 受保护缓冲首尾各留四个哨兵槽。
//! 精确跨度等于守卫槽加世界批量长度。
//! A2起批量布局归属本模块，`model::BatchLayout`保留为兼容导出。

use crate::diagnostics::InputError;
use std::ops::Range;

/// 每端哨兵槽数。
pub const GUARD_SLOTS_PER_END: usize = 4;

/// 首尾哨兵槽总数。
pub const GUARD_SLOTS: usize = GUARD_SLOTS_PER_END * 2;

/// 连续布局 `[world, element]`。
/// 不描述共享字段或原生ABI。
/// A2起归属设备数据定义；`model`根保留兼容导出。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatchLayout {
    worlds: usize,
    elements_per_world: usize,
    element_bytes: usize,
    total_elements: usize,
    stride_bytes: usize,
    total_bytes: usize,
}

impl BatchLayout {
    /// 允许每世界零个元素。
    /// 拒绝零世界和零元素字节。
    pub fn new(
        worlds: usize,
        elements_per_world: usize,
        element_bytes: usize,
    ) -> Result<Self, InputError> {
        if worlds == 0 {
            return Err(InputError::InvalidDimension { field: "worlds" });
        }
        if element_bytes == 0 {
            return Err(InputError::InvalidDimension {
                field: "element_bytes",
            });
        }
        let overflow = || InputError::Overflow {
            field: "batch_layout",
        };
        let total_elements = worlds
            .checked_mul(elements_per_world)
            .ok_or_else(overflow)?;
        let stride_bytes = elements_per_world
            .checked_mul(element_bytes)
            .ok_or_else(overflow)?;
        let total_bytes = total_elements
            .checked_mul(element_bytes)
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(overflow)?;
        Ok(Self {
            worlds,
            elements_per_world,
            element_bytes,
            total_elements,
            stride_bytes,
            total_bytes,
        })
    }

    pub fn worlds(self) -> usize {
        self.worlds
    }

    pub fn elements_per_world(self) -> usize {
        self.elements_per_world
    }

    pub fn element_bytes(self) -> usize {
        self.element_bytes
    }

    pub fn total_elements(self) -> usize {
        self.total_elements
    }

    pub fn stride_bytes(self) -> usize {
        self.stride_bytes
    }

    pub fn total_bytes(self) -> usize {
        self.total_bytes
    }

    pub fn world_elements(self, world: usize) -> Result<Range<usize>, InputError> {
        self.check_world(world)?;
        let start = world * self.elements_per_world;
        Ok(start..start + self.elements_per_world)
    }

    pub fn world_bytes(self, world: usize) -> Result<Range<usize>, InputError> {
        self.check_world(world)?;
        let start = world * self.stride_bytes;
        Ok(start..start + self.stride_bytes)
    }

    fn check_world(self, world: usize) -> Result<(), InputError> {
        if world >= self.worlds {
            return Err(InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: self.worlds,
            });
        }
        Ok(())
    }
}

/// 设备批量字段的共享布局描述。
///
/// 只记录世界数、每世界元素数与元素字节。
/// 不持有设备资源，也不代表设备缓冲已分配。
/// 精确跨度只对受保护缓冲生效；
/// 宽输出缓冲按最小长度校对。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceDataLayout {
    batch: BatchLayout,
}

impl DeviceDataLayout {
    /// 允许每世界零个元素。
    /// 拒绝零世界与零元素字节。
    pub fn new(
        worlds: usize,
        elements_per_world: usize,
        element_bytes: usize,
    ) -> Result<Self, InputError> {
        Ok(Self {
            batch: BatchLayout::new(worlds, elements_per_world, element_bytes)?,
        })
    }

    /// 复用已检查的宿主批量布局。
    pub fn from_batch(batch: BatchLayout) -> Self {
        Self { batch }
    }

    pub fn batch(&self) -> BatchLayout {
        self.batch
    }

    pub fn worlds(&self) -> usize {
        self.batch.worlds()
    }

    pub fn elements_per_world(&self) -> usize {
        self.batch.elements_per_world()
    }

    pub fn element_bytes(&self) -> usize {
        self.batch.element_bytes()
    }

    /// 不含守卫槽的总元素数。
    pub fn total_elements(&self) -> usize {
        self.batch.total_elements()
    }

    pub fn stride_bytes(&self) -> usize {
        self.batch.stride_bytes()
    }

    pub fn total_bytes(&self) -> usize {
        self.batch.total_bytes()
    }

    /// 不含守卫槽的单世界元素区间。
    pub fn world_elements(&self, world: usize) -> Result<Range<usize>, InputError> {
        self.batch.world_elements(world)
    }

    /// 受保护缓冲区首尾各四槽，单世界区间加头部守卫偏移。
    pub fn guarded_world_elements(&self, world: usize) -> Result<Range<usize>, InputError> {
        let range = self.batch.world_elements(world)?;
        OffsetSpan::new(GUARD_SLOTS_PER_END)
            .shift(range)
            .ok_or(InputError::Overflow {
                field: "guarded_world_elements",
            })
    }

    /// 受保护缓冲区的精确元素跨度。
    pub fn guarded_span_elements(&self) -> Result<usize, InputError> {
        GUARD_SLOTS
            .checked_add(self.batch.total_elements())
            .ok_or(InputError::Overflow {
                field: "guarded_span_elements",
            })
    }

    /// 受保护缓冲区的精确字节跨度。
    pub fn guarded_span_bytes(&self) -> Result<usize, InputError> {
        let elements = self.guarded_span_elements()?;
        elements
            .checked_mul(self.batch.element_bytes())
            .ok_or(InputError::Overflow {
                field: "guarded_span_bytes",
            })
    }

    /// 受保护缓冲要求精确跨度；不足与超出都拒绝。
    pub fn check_guarded_elements(&self, available: usize) -> Result<(), InputError> {
        let expected = self.guarded_span_elements()?;
        if available != expected {
            return Err(InputError::LengthMismatch {
                field: "guarded_span",
                expected,
                actual: available,
            });
        }
        Ok(())
    }

    /// 宽输出缓冲只要求覆盖最小写入长度。
    pub fn check_minimum_elements(&self, available: usize) -> Result<(), InputError> {
        let expected = self.batch.total_elements();
        if available < expected {
            return Err(InputError::LengthMismatch {
                field: "minimum_span",
                expected,
                actual: available,
            });
        }
        Ok(())
    }
}

/// 守卫槽偏移的受检换算。
struct OffsetSpan {
    offset: usize,
}

impl OffsetSpan {
    fn new(offset: usize) -> Self {
        Self { offset }
    }

    fn shift(&self, range: Range<usize>) -> Option<Range<usize>> {
        let start = range.start.checked_add(self.offset)?;
        let end = range.end.checked_add(self.offset)?;
        Some(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_guard_span_from_world_batch() {
        let layout = DeviceDataLayout::new(3, 7, 4).unwrap();
        assert_eq!(layout.worlds(), 3);
        assert_eq!(layout.elements_per_world(), 7);
        assert_eq!(layout.total_elements(), 21);
        assert_eq!(layout.stride_bytes(), 28);
        assert_eq!(layout.total_bytes(), 84);
        assert_eq!(layout.guarded_span_elements().unwrap(), 29);
        assert_eq!(layout.guarded_span_bytes().unwrap(), 116);
        assert_eq!(layout.world_elements(2).unwrap(), 14..21);
        assert_eq!(layout.guarded_world_elements(2).unwrap(), 18..25);
    }

    #[test]
    fn allows_zero_elements_and_rejects_bad_dimensions() {
        let empty = DeviceDataLayout::new(2, 0, 4).unwrap();
        assert_eq!(empty.total_elements(), 0);
        assert_eq!(empty.guarded_span_elements().unwrap(), GUARD_SLOTS);
        assert_eq!(empty.world_elements(1).unwrap(), 0..0);
        assert_eq!(empty.guarded_world_elements(1).unwrap(), 4..4);
        assert!(DeviceDataLayout::new(0, 2, 4).is_err());
        assert!(DeviceDataLayout::new(2, 2, 0).is_err());
        assert!(DeviceDataLayout::new(usize::MAX, 2, 4).is_err());
        assert!(DeviceDataLayout::new(1, 2, usize::MAX).is_err());
    }

    #[test]
    fn guarded_span_is_exact_and_wide_outputs_use_minimum() {
        let layout = DeviceDataLayout::new(2, 5, 4).unwrap();
        let span = layout.guarded_span_elements().unwrap();
        layout.check_guarded_elements(span).unwrap();
        for bad in [span - 1, span + 1, 0] {
            assert!(layout.check_guarded_elements(bad).is_err());
        }
        layout.check_minimum_elements(10).unwrap();
        layout.check_minimum_elements(11).unwrap();
        assert!(layout.check_minimum_elements(9).is_err());
    }

    #[test]
    fn reports_world_index_and_offset_overflow() {
        let layout = DeviceDataLayout::new(2, 3, 4).unwrap();
        assert!(matches!(
            layout.world_elements(2),
            Err(InputError::InvalidIndex { field: "world", .. })
        ));
        assert!(matches!(
            layout.guarded_world_elements(2),
            Err(InputError::InvalidIndex { field: "world", .. })
        ));
    }

    #[test]
    fn converts_checked_host_batch_layout() {
        let batch = BatchLayout::new(4, 6, 8).unwrap();
        let layout = DeviceDataLayout::from_batch(batch);
        assert_eq!(layout.batch(), batch);
        assert_eq!(layout.total_bytes(), batch.total_bytes());
        assert_eq!(layout.guarded_span_bytes().unwrap(), 4 * 6 * 8 + 8 * 8);
    }

    #[test]
    fn locates_dense_world_ranges() {
        let layout = BatchLayout::new(3, 5, 4).unwrap();
        assert_eq!(layout.worlds(), 3);
        assert_eq!(layout.elements_per_world(), 5);
        assert_eq!(layout.element_bytes(), 4);
        assert_eq!(layout.total_elements(), 15);
        assert_eq!(layout.stride_bytes(), 20);
        assert_eq!(layout.total_bytes(), 60);
        assert_eq!(layout.world_elements(2).unwrap(), 10..15);
        assert_eq!(layout.world_bytes(2).unwrap(), 40..60);
    }

    #[test]
    fn permits_empty_fields() {
        let layout = BatchLayout::new(2, 0, 4).unwrap();
        assert_eq!(layout.total_bytes(), 0);
        assert_eq!(layout.world_elements(1).unwrap(), 0..0);
        assert_eq!(layout.world_bytes(1).unwrap(), 0..0);
    }

    #[test]
    fn rejects_invalid_dimensions_and_worlds() {
        assert_eq!(
            BatchLayout::new(0, 1, 4),
            Err(InputError::InvalidDimension { field: "worlds" })
        );
        assert_eq!(
            BatchLayout::new(1, 1, 0),
            Err(InputError::InvalidDimension {
                field: "element_bytes"
            })
        );
        let layout = BatchLayout::new(2, 0, 4).unwrap();
        let expected = Err(InputError::InvalidIndex {
            field: "world",
            index: 2,
            limit: 2,
        });
        assert_eq!(layout.world_elements(2), expected);
        assert_eq!(layout.world_bytes(2), expected);
    }

    #[test]
    fn rejects_element_stride_and_slice_capacity_overflow() {
        for (worlds, elements, bytes) in [
            (2, usize::MAX, 1),
            (1, usize::MAX, 2),
            (1, isize::MAX as usize + 1, 1),
            (2, isize::MAX as usize / 4, 4),
        ] {
            assert_eq!(
                BatchLayout::new(worlds, elements, bytes),
                Err(InputError::Overflow {
                    field: "batch_layout"
                })
            );
        }
    }
}
