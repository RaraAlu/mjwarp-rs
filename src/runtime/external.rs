//! P1连续f32描述符。
//! 本格式不冻结生产张量ABI。

use crate::diagnostics::{ProbeError, ResourceError};
#[cfg(any(feature = "cuda-probe", test))]
use std::{collections::HashMap, sync::Weak};

pub const EXTERNAL_PROBE_ABI_VERSION: u32 = 1;
pub const EXTERNAL_PROBE_F32: u32 = 1;
pub const EXTERNAL_PROBE_WRITABLE: u32 = 1;

/// 固定宽度的P1数据声明。
/// 整数字段不构造枚举判别值。
/// 此声明不包含所有者与事件。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExternalBufferDescriptor {
    pub abi_version: u32,
    pub struct_size: u32,
    pub device_ordinal: u32,
    pub flags: u32,
    pub context: u64,
    pub allocation: u64,
    pub allocation_bytes: u64,
    pub offset_bytes: u64,
    pub elements: u64,
    pub stride_bytes: u64,
    pub layout_version: u64,
    pub scalar_type: u32,
    pub rank: u32,
    pub reserved: [u64; 2],
}

impl ExternalBufferDescriptor {
    /// 只验证声明，不查询或解引用地址。
    pub fn validate(&self) -> Result<(), ProbeError> {
        if self.abi_version != EXTERNAL_PROBE_ABI_VERSION
            || self.struct_size as usize != size_of::<Self>()
            || self.reserved != [0; 2]
            || self.flags & !EXTERNAL_PROBE_WRITABLE != 0
        {
            return Err(ResourceError::InvalidExternalDescriptor.into());
        }
        if self.rank != 1 || self.scalar_type != EXTERNAL_PROBE_F32 || self.stride_bytes != 4 {
            return Err(ResourceError::UnsupportedExternalLayout.into());
        }
        if self.context == 0 || self.allocation == 0 || self.layout_version == 0 {
            return Err(ResourceError::InvalidExternalDescriptor.into());
        }
        if self.elements == 0 || self.elements > u32::MAX as u64 {
            return Err(ResourceError::InvalidExternalDescriptor.into());
        }
        if !self.allocation.is_multiple_of(4)
            || !self.allocation_bytes.is_multiple_of(4)
            || !self.offset_bytes.is_multiple_of(4)
        {
            return Err(ResourceError::Misaligned.into());
        }
        let end = self
            .offset_bytes
            .checked_add(self.elements * 4)
            .filter(|&end| end <= self.allocation_bytes)
            .ok_or(ResourceError::InvalidExternalDescriptor)?;
        self.allocation
            .checked_add(end)
            .ok_or(ResourceError::InvalidExternalDescriptor)?;
        self.allocation
            .checked_add(self.allocation_bytes)
            .ok_or(ResourceError::InvalidExternalDescriptor)?;
        usize::try_from(self.allocation_bytes)
            .map_err(|_| ResourceError::InvalidExternalDescriptor)?;
        Ok(())
    }
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) struct ImportRegistry<T> {
    owners: HashMap<(u64, u64), Weak<T>>,
}

#[cfg(any(feature = "cuda-probe", test))]
impl<T> ImportRegistry<T> {
    pub fn new() -> Self {
        Self {
            owners: HashMap::new(),
        }
    }

    pub fn register(&mut self, identity: (u64, u64), owner: Weak<T>) -> Result<(), ProbeError> {
        self.owners.retain(|_, owner| owner.strong_count() != 0);
        if self.owners.contains_key(&identity) {
            return Err(ResourceError::DuplicateExternalAllocation.into());
        }
        self.owners.insert(identity, owner);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::lease::{Access, DeviceIdentity, LeasedBuffer, ViewRequest};
    use std::mem::{align_of, offset_of};

    fn descriptor() -> ExternalBufferDescriptor {
        ExternalBufferDescriptor {
            abi_version: 1,
            struct_size: 96,
            device_ordinal: 0,
            flags: 1,
            context: 7,
            allocation: 0x1000,
            allocation_bytes: 128,
            offset_bytes: 64,
            elements: 16,
            stride_bytes: 4,
            layout_version: 1,
            scalar_type: 1,
            rank: 1,
            reserved: [0; 2],
        }
    }

    #[test]
    fn matches_fixed_width_c_layout() {
        assert_eq!(size_of::<ExternalBufferDescriptor>(), 96);
        assert_eq!(align_of::<ExternalBufferDescriptor>(), 8);
        assert_eq!(
            [
                offset_of!(ExternalBufferDescriptor, abi_version),
                offset_of!(ExternalBufferDescriptor, struct_size),
                offset_of!(ExternalBufferDescriptor, device_ordinal),
                offset_of!(ExternalBufferDescriptor, flags),
                offset_of!(ExternalBufferDescriptor, context),
                offset_of!(ExternalBufferDescriptor, allocation),
                offset_of!(ExternalBufferDescriptor, allocation_bytes),
                offset_of!(ExternalBufferDescriptor, offset_bytes),
                offset_of!(ExternalBufferDescriptor, elements),
                offset_of!(ExternalBufferDescriptor, stride_bytes),
                offset_of!(ExternalBufferDescriptor, layout_version),
                offset_of!(ExternalBufferDescriptor, scalar_type),
                offset_of!(ExternalBufferDescriptor, rank),
                offset_of!(ExternalBufferDescriptor, reserved),
            ],
            [0, 4, 8, 12, 16, 24, 32, 40, 48, 56, 64, 72, 76, 80]
        );
    }

    #[test]
    fn accepts_read_only_and_writable_contiguous_ranges() {
        descriptor().validate().unwrap();
        ExternalBufferDescriptor {
            flags: 0,
            ..descriptor()
        }
        .validate()
        .unwrap();
        ExternalBufferDescriptor {
            offset_bytes: 0,
            elements: 32,
            ..descriptor()
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn rejects_versions_sizes_flags_and_reserved_data() {
        for changed in [
            ExternalBufferDescriptor {
                abi_version: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                abi_version: 2,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                struct_size: 88,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                flags: 2,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                reserved: [1, 0],
                ..descriptor()
            },
            ExternalBufferDescriptor {
                context: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                allocation: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                layout_version: 0,
                ..descriptor()
            },
        ] {
            assert!(changed.validate().is_err(), "{changed:?}");
        }
    }

    #[test]
    fn rejects_dtype_rank_and_noncontiguous_layouts() {
        for changed in [
            ExternalBufferDescriptor {
                scalar_type: 2,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                rank: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                rank: 2,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                stride_bytes: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                stride_bytes: 8,
                ..descriptor()
            },
        ] {
            assert_eq!(
                changed.validate(),
                Err(ResourceError::UnsupportedExternalLayout.into())
            );
        }
    }

    #[test]
    fn rejects_empty_overflowing_and_misaligned_ranges() {
        for changed in [
            ExternalBufferDescriptor {
                elements: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                elements: u64::MAX,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                elements: 17,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                offset_bytes: u64::MAX - 3,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                allocation: u64::MAX - 3,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                allocation_bytes: 0,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                allocation_bytes: 127,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                offset_bytes: 1,
                ..descriptor()
            },
            ExternalBufferDescriptor {
                allocation: 0x1001,
                ..descriptor()
            },
        ] {
            assert!(changed.validate().is_err(), "{changed:?}");
        }
    }

    #[test]
    fn rejects_duplicate_import_while_only_lease_keeps_owner() {
        let identity = DeviceIdentity {
            device: 0,
            context: 7,
        };
        let buffer = LeasedBuffer::new(vec![0u32; 32], identity, 128, 4, true).unwrap();
        let owner = buffer.owner_weak();
        let mut registry = ImportRegistry::new();
        registry.register((7, 42), owner.clone()).unwrap();
        let lease = buffer
            .lease(ViewRequest {
                target: identity,
                offset: 0,
                bytes: 64,
                layout: 1,
                access: Access::Write,
            })
            .unwrap();
        drop(buffer);
        assert_eq!(
            registry.register((7, 42), owner.clone()),
            Err(ResourceError::DuplicateExternalAllocation.into())
        );
        drop(lease);
        registry.register((7, 42), Weak::new()).unwrap();
        assert!(owner.upgrade().is_none());
    }

    #[test]
    fn registry_distinguishes_contexts_and_allocation_generations() {
        let first = std::sync::Arc::new(1u32);
        let mut registry = ImportRegistry::new();
        for identity in [(7, 1), (8, 1), (7, 2)] {
            registry
                .register(identity, std::sync::Arc::downgrade(&first))
                .unwrap();
        }
        assert_eq!(registry.owners.len(), 3);
    }
}
