//! A2：渲染布局归属的宿主验证。
//! 只核对布局迁移、尺寸、世界数量、空批次与溢出。
//! 本用例不宣称批量渲染、像素生成或BVH功能完成。

use mjwarp_rs::diagnostics::InputError;
use mjwarp_rs::model::BatchLayout as CompatBatchLayout;
use mjwarp_rs::model::data::{BatchLayout as DeviceBatchLayout, DeviceDataLayout};
use mjwarp_rs::render::{ImageBatchLayout, ImageOutput, OutputChannels};

const ALL: OutputChannels = OutputChannels {
    rgb: true,
    depth: true,
    segmentation: true,
};

const RGB_ONLY: OutputChannels = OutputChannels {
    rgb: true,
    depth: false,
    segmentation: false,
};

#[test]
fn batch_layout_keeps_the_compatibility_export() {
    fn name<T>() -> &'static str {
        std::any::type_name::<T>()
    }
    assert_eq!(name::<CompatBatchLayout>(), name::<DeviceBatchLayout>());
    let layout = CompatBatchLayout::new(3, 5, 4).unwrap();
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
fn batch_layout_allows_empty_batches_and_rejects_bad_dimensions() {
    let empty = CompatBatchLayout::new(2, 0, 4).unwrap();
    assert_eq!(empty.total_elements(), 0);
    assert_eq!(empty.total_bytes(), 0);
    assert_eq!(empty.world_elements(1).unwrap(), 0..0);
    assert!(matches!(
        CompatBatchLayout::new(0, 1, 4),
        Err(InputError::InvalidDimension { field: "worlds" })
    ));
    assert!(matches!(
        CompatBatchLayout::new(1, 1, 0),
        Err(InputError::InvalidDimension {
            field: "element_bytes"
        })
    ));
    assert!(matches!(
        CompatBatchLayout::new(2, 0, 4).unwrap().world_elements(2),
        Err(InputError::InvalidIndex { field: "world", .. })
    ));
}

#[test]
fn batch_layout_rejects_stride_and_capacity_overflow() {
    for (worlds, elements, bytes) in [
        (2, usize::MAX, 1),
        (1, usize::MAX, 2),
        (1, isize::MAX as usize + 1, 1),
        (2, isize::MAX as usize / 4, 4),
    ] {
        assert!(matches!(
            CompatBatchLayout::new(worlds, elements, bytes),
            Err(InputError::Overflow {
                field: "batch_layout"
            })
        ));
    }
}

#[test]
fn device_layout_derives_guarded_pixel_spans() {
    let pixels = DeviceDataLayout::new(2, 6, 4).unwrap();
    assert_eq!(pixels.worlds(), 2);
    assert_eq!(pixels.total_elements(), 12);
    assert_eq!(pixels.guarded_span_elements().unwrap(), 8 + 12);
    assert_eq!(pixels.guarded_span_bytes().unwrap(), (8 + 12) * 4);
    pixels.check_minimum_elements(12).unwrap();
    assert!(pixels.check_minimum_elements(11).is_err());
    pixels.check_guarded_elements(20).unwrap();
    assert!(pixels.check_guarded_elements(19).is_err());
    assert_eq!(pixels.guarded_world_elements(1).unwrap(), 4 + 6..4 + 12);
}

#[test]
fn image_layout_uses_device_layout_contract() {
    let layout = ImageBatchLayout::new(2, 3, 2, ALL).unwrap();
    assert_eq!(layout.worlds(), 2);
    assert_eq!(layout.width(), 3);
    assert_eq!(layout.height(), 2);
    assert_eq!(layout.channels(), ALL);
    assert_eq!(layout.pixel_index(0, 0, 0).unwrap(), 0);
    assert_eq!(layout.pixel_index(1, 1, 2).unwrap(), 11);
    assert_eq!(layout.buffer_bytes(ImageOutput::Rgb).unwrap(), 12 * 4);
    assert_eq!(layout.buffer_bytes(ImageOutput::Depth).unwrap(), 12 * 4);
    assert_eq!(
        layout.buffer_bytes(ImageOutput::Segmentation).unwrap(),
        12 * 8
    );
    assert_eq!(
        layout
            .pixel_range(ImageOutput::Segmentation, 1, 0, 1)
            .unwrap(),
        14..16
    );
    assert_eq!(layout.pixel_range(ImageOutput::Rgb, 0, 1, 1).unwrap(), 4..5);
}

#[test]
fn image_layout_rejects_empty_worlds_pixels_and_overflow() {
    assert!(matches!(
        ImageBatchLayout::new(0, 3, 2, ALL),
        Err(InputError::InvalidDimension { field: "worlds" })
    ));
    assert!(matches!(
        ImageBatchLayout::new(1, 0, 2, ALL),
        Err(InputError::InvalidDimension { field: "width" })
    ));
    assert!(matches!(
        ImageBatchLayout::new(1, 3, 0, ALL),
        Err(InputError::InvalidDimension { field: "height" })
    ));
    assert!(matches!(
        ImageBatchLayout::new(1, usize::MAX, 2, ALL),
        Err(InputError::Overflow {
            field: "image_layout"
        })
    ));
    let layout = ImageBatchLayout::new(1, 3, 2, RGB_ONLY).unwrap();
    assert!(matches!(
        layout.buffer_bytes(ImageOutput::Segmentation),
        Err(InputError::DisabledOutput)
    ));
    assert!(matches!(
        layout.pixel_index(1, 0, 0),
        Err(InputError::InvalidIndex {
            field: "world",
            index: 1,
            limit: 1
        })
    ));
    assert!(matches!(
        layout.pixel_index(0, 2, 0),
        Err(InputError::InvalidIndex { field: "row", .. })
    ));
    assert!(matches!(
        layout.pixel_index(0, 0, 3),
        Err(InputError::InvalidIndex {
            field: "column",
            ..
        })
    ));
}
