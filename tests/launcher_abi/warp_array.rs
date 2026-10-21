//! 四维Warp数组与四维边界拒绝测试。

use mjwarp_rs::runtime::launcher::{AbiSignature, LauncherError, WarpArray};

use super::*;

#[test]
fn rejects_four_dimensional_launch_bounds() {
    const FOUR: &[SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 4 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 1,
            },
            access: Access::Write,
            role: SlotRole::Value,
        },
    ];
    assert!(matches!(
        AbiSignature::check("four_dim_launch", BlockRule::WarpGrid, FOUR, &[]),
        Err(LauncherError::InvalidSignature {
            abi: "four_dim_launch",
            ..
        })
    ));
}

#[test]
fn accepts_four_dimensional_warp_arrays() {
    const ARRAY4: &[SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 1 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 4,
            },
            access: Access::Write,
            role: SlotRole::Value,
        },
    ];
    AbiSignature::check("array4", BlockRule::WarpGrid, ARRAY4, &[]).unwrap();
    let array = WarpArray::<f32>::contiguous(0x1000, &[2, 3, 4, 5]).unwrap();
    assert_eq!(array.ndim(), 4);
    assert_eq!(array.shape(), [2, 3, 4, 5]);
    assert_eq!(array.strides(), [240, 80, 20, 4]);
    assert_eq!(array.byte_len(), 480);
    assert_eq!(std::mem::size_of::<WarpArray<f32>>(), 56);
}
