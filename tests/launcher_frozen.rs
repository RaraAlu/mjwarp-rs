//! L2 冻结产物统一发射面的宿主检查。
//! 这些用例不发射内核，只核对准入、边界与数组描述。
#![cfg(feature = "cuda-probe")]

use mjwarp_rs::runtime::launcher::{FrozenPtx, LaunchBounds2, LaunchBounds3, WarpArray};
use sha2::{Digest, Sha256};

const PTX: &str = ".version 7.0\n.target sm_70\n.address_size 64\n";

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn frozen_ptx_admits_only_matching_digest() {
    let expected = digest(PTX.as_bytes());
    let frozen = FrozenPtx::from_ptx("frozen_probe", PTX, &expected).unwrap();
    assert_eq!(frozen.name(), "frozen_probe");
    assert_eq!(frozen.ptx(), PTX);
    assert_eq!(frozen.sha256(), expected);
    assert!(FrozenPtx::from_ptx("frozen_probe", PTX, "00").is_err());
    assert!(FrozenPtx::from_ptx("bad/name", PTX, &expected).is_err());
    assert!(FrozenPtx::from_ptx("frozen_probe", "", &digest(b"")).is_err());
}

#[test]
fn frozen_ptx_file_bytes_strip_one_trailing_nul() {
    let mut bytes = PTX.as_bytes().to_vec();
    bytes.push(0);
    let raw = digest(&bytes);
    let frozen = FrozenPtx::from_file_bytes("frozen_file", &bytes, &raw).unwrap();
    assert_eq!(frozen.ptx(), PTX);
    assert_eq!(frozen.sha256(), raw);
    // 摘要按原始字节核验，剥离后的文本摘要不同。
    assert!(FrozenPtx::from_file_bytes("frozen_file", &bytes, &digest(PTX.as_bytes())).is_err());
    let mut inner = PTX.as_bytes().to_vec();
    inner.insert(5, 0);
    let inner_digest = digest(&inner);
    assert!(FrozenPtx::from_file_bytes("frozen_file", &inner, &inner_digest).is_err());
}

#[test]
fn warp_array_descriptor_matches_row_major_layout() {
    let array = WarpArray::<f32>::with_element_bytes(0x1000, &[2, 3], 12).unwrap();
    assert_eq!(array.shape(), [2, 3]);
    assert_eq!(array.strides(), [36, 12]);
    // byte_len按元素类型计数；向量宽度由步长表达。
    assert_eq!(array.byte_len(), 24);
    assert_eq!(array.data(), 0x1000);
    let flat = WarpArray::<i32>::with_element_bytes(0x2000, &[7], 4).unwrap();
    assert_eq!(flat.shape(), [7]);
    assert_eq!(flat.byte_len(), 28);
    assert!(WarpArray::<f32>::with_element_bytes(0, &[1], 4).is_err());
    assert!(WarpArray::<f32>::with_element_bytes(0x1000, &[2, 0], 4).is_err());
    assert!(WarpArray::<f32>::with_element_bytes(0x1000, &[1, 2, 3, 4, 5], 4).is_err());
    assert!(WarpArray::<f32>::with_element_bytes(0x1000, &[1], 0).is_err());
}

#[test]
fn launch_bounds_reject_zero_and_overflow() {
    let two = LaunchBounds2::new([4, 8]).unwrap();
    assert_eq!(two.shape(), [4, 8]);
    assert_eq!(two.size(), 32);
    let three = LaunchBounds3::new([2, 3, 5]).unwrap();
    assert_eq!(three.size(), 30);
    assert!(LaunchBounds2::new([0, 8]).is_err());
    assert!(LaunchBounds3::new([1, 0, 5]).is_err());
    assert!(LaunchBounds2::new([u32::MAX as usize, 2]).is_err());
}
