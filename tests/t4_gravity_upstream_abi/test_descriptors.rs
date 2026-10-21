//! Warp布局与重力描述符测试。

use super::abi::*;
use super::package_gate::*;
use super::support::*;
use super::*;

#[test]
pub(super) fn warp_abi_layout_and_gravity_descriptors_are_exact() {
    assert_eq!(size_of::<WarpArray>(), 56);
    assert_eq!(std::mem::align_of::<WarpArray>(), 8);
    assert_eq!(std::mem::offset_of!(WarpArray, shape), 16);
    assert_eq!(std::mem::offset_of!(WarpArray, strides), 32);
    assert_eq!(std::mem::offset_of!(WarpArray, ndim), 48);
    assert_eq!(std::mem::offset_of!(WarpArray, flags), 50);
    assert_eq!(size_of::<WarpBounds3>(), 32);
    assert_eq!(std::mem::align_of::<WarpBounds3>(), 8);
    assert_eq!(std::mem::offset_of!(WarpBounds3, size), 16);
    assert_eq!(std::mem::offset_of!(WarpBounds3, coord_mult), 24);

    let package = package().unwrap();
    validate_package(&package).unwrap();
    let bounds = WarpBounds3 {
        shape: [7, 4, 4],
        padding: 0,
        size: 112,
        coord_mult: 1,
    };
    assert_eq!(bounds.size, 7 * 4 * 4);
    let specs: [(&[usize], usize); 11] = [
        (&[2], 12),
        (&[5], 4),
        (&[5], 4),
        (&[3, 5], 4),
        (&[5, 5], 4),
        (&[4], 4),
        (&[5, 4], 4),
        (&[7, 5], 12),
        (&[7, 5], 12),
        (&[7, 4], 24),
        (&[7, 4], 4),
    ];
    let arrays = specs
        .into_iter()
        .enumerate()
        .map(|(index, (shape, bytes))| {
            contiguous(0x1000 + index as u64 * 0x1000, shape, bytes).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(arrays[0].strides, [12, 0, 0, 0]);
    assert_eq!(arrays[3].strides, [20, 4, 0, 0]);
    assert_eq!(arrays[7].strides, [60, 12, 0, 0]);
    assert_eq!(arrays[9].strides, [96, 24, 0, 0]);
    assert_eq!(arrays[10].strides, [16, 4, 0, 0]);
    assert!(contiguous(0, &[1], 4).is_err());
    assert!(contiguous(1, &[0], 4).is_err());
    assert!(contiguous(1, &[1, 1, 1, 1, 1], 4).is_err());
    assert_eq!(
        max_absolute_finite(&[f32::NAN], &[0.0]),
        Err(GateError::Mismatch("non-finite output"))
    );
    assert_eq!(
        max_absolute_finite(&[0.0], &[f32::INFINITY]),
        Err(GateError::Mismatch("non-finite output"))
    );
}
