//! 连续批量字段的宿主布局。
//! 不分配设备缓冲。

use std::ops::Range;

use crate::diagnostics::{InputError, NativeProbeError};

/// Windows原生探针的小型DTO。
/// 计数采用原生有符号64位。
/// 本类型不表示完整模型布局。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeModelInfo {
    pub schema: u32,
    pub native_version: i32,
    pub pointer_bytes: u32,
    pub num_bytes: u32,
    pub index_bytes: u32,
    pub size_bytes: u32,
    pub native_model_bytes: u64,
    pub nq: i64,
    pub nv: i64,
    pub nu: i64,
    pub na: i64,
    pub nbody: i64,
    pub njnt: i64,
    pub ngeom: i64,
    pub nsensordata: i64,
}

const _: () = {
    assert!(size_of::<NativeModelInfo>() == 96);
    assert!(align_of::<NativeModelInfo>() == 8);
    assert!(std::mem::offset_of!(NativeModelInfo, native_model_bytes) == 24);
    assert!(std::mem::offset_of!(NativeModelInfo, nq) == 32);
    assert!(std::mem::offset_of!(NativeModelInfo, nbody) == 64);
};

impl NativeModelInfo {
    pub const VERSION: i32 = 3_012_000;

    /// 仅验证候选ABI与计数容量。
    /// 不验证完整字段与引用。
    pub fn validate(self) -> Result<(), NativeProbeError> {
        if self.schema != 1
            || self.native_version != Self::VERSION
            || (
                self.pointer_bytes,
                self.num_bytes,
                self.index_bytes,
                self.size_bytes,
            ) != (8, 8, 4, 8)
            || self.native_model_bytes == 0
        {
            return Err(NativeProbeError::AbiMismatch);
        }
        for (field, count) in [
            ("nq", self.nq),
            ("nv", self.nv),
            ("nu", self.nu),
            ("na", self.na),
            ("nbody", self.nbody),
            ("njnt", self.njnt),
            ("ngeom", self.ngeom),
            ("nsensordata", self.nsensordata),
        ] {
            if count < 0 || (field == "nbody" && count == 0) {
                return Err(InputError::InvalidDimension { field }.into());
            }
            let count = usize::try_from(count).map_err(|_| InputError::Overflow { field })?;
            BatchLayout::new(1, count, 8)?;
        }
        Ok(())
    }
}

/// 连续布局 `[world, element]`。
/// 不描述共享字段或原生ABI。
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

#[cfg(test)]
mod tests {
    use super::*;

    fn native_info() -> NativeModelInfo {
        NativeModelInfo {
            schema: 1,
            native_version: NativeModelInfo::VERSION,
            pointer_bytes: 8,
            num_bytes: 8,
            index_bytes: 4,
            size_bytes: 8,
            native_model_bytes: 1,
            nbody: 1,
            ..Default::default()
        }
    }

    #[test]
    fn checks_native_dto_layout_and_zero_dof() {
        assert_eq!(size_of::<NativeModelInfo>(), 96);
        assert_eq!(std::mem::offset_of!(NativeModelInfo, nsensordata), 88);
        native_info().validate().unwrap();
        let mut info = native_info();
        info.nu = 2; // nu counts control inputs, not actuators.
        info.validate().unwrap();
    }

    #[test]
    fn rejects_native_schema_version_and_scalar_widths() {
        let original = native_info();
        for info in [
            NativeModelInfo {
                schema: 2,
                ..original
            },
            NativeModelInfo {
                native_version: 123,
                ..original
            },
            NativeModelInfo {
                num_bytes: 4,
                ..original
            },
            NativeModelInfo {
                index_bytes: 8,
                ..original
            },
            NativeModelInfo {
                size_bytes: 4,
                ..original
            },
            NativeModelInfo {
                pointer_bytes: 4,
                ..original
            },
            NativeModelInfo {
                native_model_bytes: 0,
                ..original
            },
        ] {
            assert_eq!(info.validate(), Err(NativeProbeError::AbiMismatch));
        }
    }

    #[test]
    fn rejects_native_negative_and_overflowing_counts() {
        let original = native_info();
        for info in [
            NativeModelInfo { nq: -1, ..original },
            NativeModelInfo { nv: -1, ..original },
            NativeModelInfo { nu: -1, ..original },
            NativeModelInfo { na: -1, ..original },
            NativeModelInfo {
                nbody: 0,
                ..original
            },
            NativeModelInfo {
                njnt: -1,
                ..original
            },
            NativeModelInfo {
                ngeom: -1,
                ..original
            },
            NativeModelInfo {
                nsensordata: -1,
                ..original
            },
            NativeModelInfo {
                nq: i64::MAX,
                ..original
            },
        ] {
            assert!(matches!(info.validate(), Err(NativeProbeError::Input(_))));
        }
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
