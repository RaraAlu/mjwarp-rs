//! M1发射器基础：配方、签名、布局与绑定校验。
//! 全部为宿主测试，不需要GPU或NVRTC。

#[path = "launcher_abi/bounds.rs"]
mod bounds;
#[path = "launcher_abi/rebind.rs"]
mod rebind;
#[path = "launcher_abi/recipe.rs"]
mod recipe;
#[path = "launcher_abi/signature.rs"]
mod signature;
#[path = "launcher_abi/warp_array.rs"]
mod warp_array;

use mjwarp_rs::runtime::launcher::{
    Access, AliasException, BlockRule, DeviceSpan, ElemType, KernelAbi, ProofKind, SlotBinding,
    SlotDesc, SlotKind, SlotRequirement, SlotRole,
};

const META: SlotRole = SlotRole::AddressBearing {
    proof: ProofKind("mixed-solve-route"),
};

const fn ptr(index: usize, elem: ElemType, access: Access, role: SlotRole) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access,
        role,
    }
}

const fn scalar(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

/// 镜像`launch_mixed_solve`的私有ABI：
/// 元数据、右端项、因子、输出与四项u32维度。
struct MixedSolve128;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for MixedSolve128 {
    const NAME: &'static str = "mixed_solve_128";
    const BLOCK: BlockRule = BlockRule::Fixed(128);
    const SLOTS: &'static [SlotDesc] = &[
        ptr(0, ElemType::I32, Access::Read, META),
        ptr(1, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(2, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(3, ElemType::F32, Access::Write, SlotRole::Value),
        scalar(4),
        scalar(5),
        scalar(6),
        scalar(7),
    ];
}

/// 原位更新示例：输入与输出显式允许重叠。
struct InPlace;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for InPlace {
    const NAME: &'static str = "in_place";
    const BLOCK: BlockRule = BlockRule::WarpGrid;
    const SLOTS: &'static [SlotDesc] = &[
        ptr(0, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(1, ElemType::F32, Access::ReadWrite, SlotRole::Value),
        scalar(2),
    ];
    const ALIAS_EXCEPTIONS: &'static [AliasException] = &[AliasException {
        first: 0,
        second: 1,
    }];
}

/// Warp ABI示例：三维边界加三个数组。
struct WarpExample;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for WarpExample {
    const NAME: &'static str = "warp_example";
    const BLOCK: BlockRule = BlockRule::WarpGrid;
    const SLOTS: &'static [SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 3 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::I32,
                ndim: 1,
            },
            access: Access::Read,
            role: SlotRole::AddressBearing {
                proof: ProofKind("body-parent"),
            },
        },
        SlotDesc {
            index: 2,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 2,
            },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 3,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 2,
            },
            access: Access::Write,
            role: SlotRole::Value,
        },
    ];
}

fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

fn bind(slot: usize, span: DeviceSpan, elem: ElemType) -> SlotBinding {
    SlotBinding { slot, span, elem }
}

/// 四个互不重叠的合法绑定。
fn mixed_bindings() -> Vec<SlotBinding> {
    vec![
        bind(0, span(7, 0x1_0000, 256), ElemType::I32),
        bind(1, span(7, 0x2_0000, 4096), ElemType::F32),
        bind(2, span(7, 0x3_0000, 4096), ElemType::F32),
        bind(3, span(7, 0x4_0000, 4096), ElemType::F32),
    ]
}

fn requirements() -> Vec<SlotRequirement> {
    vec![
        SlotRequirement {
            slot: 0,
            min_elements: 64,
        },
        SlotRequirement {
            slot: 3,
            min_elements: 1024,
        },
    ]
}
