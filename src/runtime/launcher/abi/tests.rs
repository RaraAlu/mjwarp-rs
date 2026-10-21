use super::*;
use std::mem::{align_of, offset_of, size_of};

const L2_KINEMATICS: &str = include_str!("../../warp_kinematics.rs");
const L2_GRAVITY: &str = include_str!("../../warp_gravity.rs");

#[test]
fn warp_array_matches_frozen_l2_layout() {
    assert_eq!(size_of::<WarpArray<f32>>(), 56);
    assert_eq!(align_of::<WarpArray<f32>>(), 8);
    assert_eq!(offset_of!(WarpArray<f32>, data), 0);
    assert_eq!(offset_of!(WarpArray<f32>, grad), 8);
    assert_eq!(offset_of!(WarpArray<f32>, shape), 16);
    assert_eq!(offset_of!(WarpArray<f32>, strides), 32);
    assert_eq!(offset_of!(WarpArray<f32>, ndim), 48);
    assert_eq!(offset_of!(WarpArray<f32>, flags), 50);
    assert_eq!(offset_of!(WarpArray<f32>, padding), 52);
    assert_eq!(size_of::<WarpArray<f64>>(), 56);
    let fields = "    data: u64,\n    grad: u64,\n    shape: [i32; 4],\n    strides: [i32; 4],\n    ndim: u16,\n    flags: u16,\n    padding: u32,\n}";
    // `warp_com.rs`不定义数组结构；重复定义只有两份。
    for source in [L2_KINEMATICS, L2_GRAVITY] {
        assert!(source.contains(&format!("struct WarpArray {{\n{fields}")));
    }
}

#[test]
fn launch_bounds_match_frozen_l2_layouts() {
    assert_eq!(size_of::<LaunchBounds1>(), 24);
    assert_eq!(size_of::<LaunchBounds2>(), 24);
    assert_eq!(size_of::<LaunchBounds3>(), 32);
    assert_eq!(align_of::<LaunchBounds1>(), 8);
    assert_eq!(offset_of!(LaunchBounds1, padding), 4);
    assert_eq!(offset_of!(LaunchBounds1, size), 8);
    assert_eq!(offset_of!(LaunchBounds1, coord_mult), 16);
    assert_eq!(offset_of!(LaunchBounds2, size), 8);
    assert_eq!(offset_of!(LaunchBounds2, coord_mult), 16);
    assert_eq!(offset_of!(LaunchBounds3, padding), 12);
    assert_eq!(offset_of!(LaunchBounds3, size), 16);
    assert_eq!(offset_of!(LaunchBounds3, coord_mult), 24);
    assert!(L2_KINEMATICS.contains(
        "struct WarpBounds2 {\n    shape: [i32; 2],\n    size: u64,\n    coord_mult: u64,\n}"
    ));
    assert!(L2_GRAVITY.contains(
            "struct WarpBounds3 {\n    shape: [i32; 3],\n    padding: u32,\n    size: u64,\n    coord_mult: u64,\n}"
        ));
    assert!(L2_KINEMATICS.contains("coord_mult: 1,"));
}

#[test]
fn contiguous_array_uses_row_major_byte_strides() {
    let array = WarpArray::<f32>::contiguous(0x1000, &[3, 5, 7]).unwrap();
    assert_eq!(array.ndim(), 3);
    assert_eq!(array.shape(), [3, 5, 7]);
    assert_eq!(array.strides(), [140, 28, 4]);
    assert_eq!(array.byte_len(), 420);
    let doubles = WarpArray::<f64>::contiguous(0x1000, &[2, 6]).unwrap();
    assert_eq!(doubles.strides(), [48, 8]);
}
