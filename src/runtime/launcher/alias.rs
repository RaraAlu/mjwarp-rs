//! W19：缓冲绑定的容量、上下文与别名联合校验。
//! 重绑只接受数值槽，并对整批结果重新校验。
//! 本模块只比较地址区间，不访问设备。

use super::{
    LauncherError,
    abi::{AbiSignature, SlotRole},
};

/// 一段设备字节区间。
/// `context`区分上下文；不同上下文的区间不比较地址。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceSpan {
    context: u64,
    addr: u64,
    bytes: u64,
}

impl DeviceSpan {
    /// 地址非零、长度非零且末端不溢出。
    pub fn new(context: u64, addr: u64, bytes: u64) -> Result<Self, LauncherError> {
        if addr == 0 {
            return Err(LauncherError::InvalidSpan("设备地址为空"));
        }
        if bytes == 0 {
            return Err(LauncherError::InvalidSpan("区间长度为零"));
        }
        addr.checked_add(bytes)
            .ok_or(LauncherError::Overflow("device_span_end"))?;
        Ok(Self {
            context,
            addr,
            bytes,
        })
    }

    /// 同一分配的子视图；偏移与长度按字节计。
    pub fn view(&self, offset: u64, bytes: u64) -> Result<Self, LauncherError> {
        let end = offset
            .checked_add(bytes)
            .ok_or(LauncherError::Overflow("device_view_end"))?;
        if end > self.bytes {
            return Err(LauncherError::InvalidSpan("视图超出原区间"));
        }
        Self::new(self.context, self.addr + offset, bytes)
    }

    pub fn context(&self) -> u64 {
        self.context
    }

    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// 同一上下文且字节区间相交。
    pub fn overlaps(&self, other: &Self) -> bool {
        self.context == other.context
            && self.addr < other.addr + other.bytes
            && other.addr < self.addr + self.bytes
    }
}

/// 一个缓冲槽位的实际绑定。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotBinding {
    pub slot: usize,
    pub span: DeviceSpan,
    pub elem: super::ElemType,
}

/// 一个缓冲槽位的最小元素数。
/// 未列出的缓冲槽位只要求非空。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotRequirement {
    pub slot: usize,
    pub min_elements: u64,
}

/// 对整组绑定做联合校验。
///
/// 依次核对：槽位覆盖、元素类型、上下文、
/// 对齐、容量与别名。任一失败即返回错误。
pub fn validate_bindings(
    signature: &AbiSignature,
    bindings: &[SlotBinding],
    requirements: &[SlotRequirement],
) -> Result<(), LauncherError> {
    let slots = signature.slots();
    let mut bound: Vec<Option<&SlotBinding>> = vec![None; slots.len()];
    for binding in bindings {
        let Some(desc) = slots.get(binding.slot) else {
            return Err(LauncherError::UnexpectedBinding { slot: binding.slot });
        };
        let Some(elem) = desc.kind.elem() else {
            return Err(LauncherError::UnexpectedBinding { slot: binding.slot });
        };
        if bound[binding.slot].is_some() {
            return Err(LauncherError::DuplicateBinding { slot: binding.slot });
        }
        if binding.elem != elem {
            return Err(LauncherError::ElementMismatch { slot: binding.slot });
        }
        if !binding.span.addr().is_multiple_of(elem.bytes()) {
            return Err(LauncherError::Misaligned { slot: binding.slot });
        }
        bound[binding.slot] = Some(binding);
    }
    let mut context = None;
    for (index, desc) in slots.iter().enumerate() {
        if !desc.kind.is_buffer() {
            continue;
        }
        let Some(binding) = bound[index] else {
            return Err(LauncherError::MissingBinding { slot: index });
        };
        match context {
            None => context = Some(binding.span.context()),
            Some(id) if id != binding.span.context() => {
                return Err(LauncherError::ContextMismatch { slot: index });
            }
            Some(_) => {}
        }
    }
    for requirement in requirements {
        let Some(binding) = bound.get(requirement.slot).copied().flatten() else {
            return Err(LauncherError::UnexpectedBinding {
                slot: requirement.slot,
            });
        };
        let required = requirement
            .min_elements
            .checked_mul(binding.elem.bytes())
            .ok_or(LauncherError::Overflow("slot_requirement_bytes"))?;
        if binding.span.bytes() < required {
            return Err(LauncherError::Capacity {
                slot: requirement.slot,
                required,
                actual: binding.span.bytes(),
            });
        }
    }
    // 可写槽独占；只读槽之间允许重叠；例外须显式声明。
    for first in 0..slots.len() {
        let Some(a) = bound[first] else { continue };
        for second in first + 1..slots.len() {
            let Some(b) = bound[second] else { continue };
            if !a.span.overlaps(&b.span) {
                continue;
            }
            let writes = slots[first].access.writes() || slots[second].access.writes();
            if writes && !signature.allows_overlap(first, second) {
                return Err(LauncherError::Alias { first, second });
            }
        }
    }
    Ok(())
}

/// 批量重绑数值槽，返回新的完整绑定。
///
/// 规则：只接受数值槽；类型与上下文不变；
/// 容量不得变小；整批结果须通过联合校验。
/// 任一失败时不返回部分结果，原绑定保持不变。
pub fn rebind_values(
    signature: &AbiSignature,
    current: &[SlotBinding],
    updates: &[SlotBinding],
    requirements: &[SlotRequirement],
) -> Result<Vec<SlotBinding>, LauncherError> {
    let slots = signature.slots();
    let mut next = current.to_vec();
    for (i, update) in updates.iter().enumerate() {
        let Some(desc) = slots.get(update.slot) else {
            return Err(LauncherError::UnexpectedBinding { slot: update.slot });
        };
        if !desc.kind.is_buffer() {
            return Err(LauncherError::UnexpectedBinding { slot: update.slot });
        }
        if matches!(desc.role, SlotRole::AddressBearing { .. }) {
            return Err(LauncherError::AddressSlotRebind { slot: update.slot });
        }
        if updates[..i].iter().any(|other| other.slot == update.slot) {
            return Err(LauncherError::DuplicateBinding { slot: update.slot });
        }
        let Some(old) = next.iter_mut().find(|binding| binding.slot == update.slot) else {
            return Err(LauncherError::MissingBinding { slot: update.slot });
        };
        if update.elem != old.elem {
            return Err(LauncherError::ElementMismatch { slot: update.slot });
        }
        if update.span.context() != old.span.context() {
            return Err(LauncherError::ContextMismatch { slot: update.slot });
        }
        if update.span.bytes() < old.span.bytes() {
            return Err(LauncherError::RebindShrink { slot: update.slot });
        }
        *old = *update;
    }
    validate_bindings(signature, &next, requirements)?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_detect_overlap_only_within_one_context() {
        let a = DeviceSpan::new(1, 0x1000, 0x100).unwrap();
        let touching = DeviceSpan::new(1, 0x1100, 0x10).unwrap();
        let inside = DeviceSpan::new(1, 0x10f0, 0x20).unwrap();
        let other_context = DeviceSpan::new(2, 0x1000, 0x100).unwrap();
        assert!(!a.overlaps(&touching));
        assert!(a.overlaps(&inside));
        assert!(inside.overlaps(&a));
        assert!(!a.overlaps(&other_context));
    }

    #[test]
    fn rejects_empty_null_and_overflowing_spans() {
        assert!(DeviceSpan::new(1, 0, 4).is_err());
        assert!(DeviceSpan::new(1, 4, 0).is_err());
        assert_eq!(
            DeviceSpan::new(1, u64::MAX - 3, 8),
            Err(LauncherError::Overflow("device_span_end"))
        );
        let base = DeviceSpan::new(1, 0x1000, 64).unwrap();
        assert_eq!(base.view(16, 16).unwrap().addr(), 0x1010);
        assert!(base.view(60, 8).is_err());
        assert_eq!(
            base.view(u64::MAX, 2),
            Err(LauncherError::Overflow("device_view_end"))
        );
    }
}
