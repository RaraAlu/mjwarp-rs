//! 渲染/BVH R08-S1 验收：逐相机输出视图（宿主侧，无 GPU、无窗口）。
//! 语义按冻结上游 `render_util.py`（`71da24d`）的 `unpack_rgb_kernel`/`extract_depth_kernel`/
//! `_extract_seg_kernel` 与入口 L212/L228/L264；参考夹具为 authoring 期 pinned 上游 GPU 采集。

use mjwarp_rs::render::camera::CameraPlan;
use mjwarp_rs::render::context::OutputLayout;
use mjwarp_rs::render::texture::INV_255;
use mjwarp_rs::render::{
    CameraImage, CameraOutputViews, CameraViewError, ImageOutput, WholeOutputData,
};
use serde_json::Value;

const RGB_REFERENCE: &str =
    include_str!("../fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json");
const DEPTH_SEG_REFERENCE: &str =
    include_str!("../fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json");
/// 上游 `wp.static(1.0 / 255.0)` 的 f32 位型（`float32(1/255)`）。
const INV_255_BITS: u32 = 0x3B80_8081;

fn nums(value: &Value) -> &[Value] {
    value.as_array().expect("数组")
}
fn u32s(value: &Value) -> Vec<u32> {
    nums(value)
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect()
}
fn i32s(value: &Value) -> Vec<i32> {
    nums(value)
        .iter()
        .map(|v| v.as_i64().unwrap() as i32)
        .collect()
}
fn pixels(resolution: [u32; 2]) -> usize {
    resolution[0] as usize * resolution[1] as usize
}
fn offset(resolutions: &[[u32; 2]], index: usize) -> usize {
    resolutions[..index].iter().copied().map(pixels).sum()
}

/// 冻结参考的逐相机位型：RGB packed u32、depth u32 位型、seg 对。
struct References {
    resolutions: Vec<[u32; 2]>,
    rgb_packed: Vec<Vec<u32>>,
    depth_bits: Vec<Vec<u32>>,
    seg_pairs: Vec<Vec<[i32; 2]>>,
}

fn references() -> References {
    let rgb = serde_json::from_str::<Value>(RGB_REFERENCE).expect("rgb 参考解析");
    let depth_seg = serde_json::from_str::<Value>(DEPTH_SEG_REFERENCE).expect("depth/seg 参考解析");
    assert_eq!(rgb["resolved"]["cam_res"], depth_seg["resolved"]["cam_res"]);
    // 夹具声明的上游编码：解包 = float32(byte) * float32(1/255)；采集期 unpack 与 get_rgb 相等。
    assert_eq!(
        rgb["self_check"]["unpack_matches_get_rgb"],
        Value::Bool(true)
    );
    assert!(
        rgb["encoding"]["rgb"]
            .as_str()
            .unwrap()
            .contains("float32(1/255)")
    );
    assert!(
        depth_seg["encoding"]["depth"]
            .as_str()
            .unwrap()
            .contains("binary32 bits")
    );
    let mut out = References {
        resolutions: Vec::new(),
        rgb_packed: Vec::new(),
        depth_bits: Vec::new(),
        seg_pairs: Vec::new(),
    };
    for camera in nums(&rgb["cameras"]) {
        let resolution = [
            camera["width"].as_u64().unwrap() as u32,
            camera["height"].as_u64().unwrap() as u32,
        ];
        let index = out.resolutions.len();
        assert_eq!(
            pixels(resolution),
            camera["pixels"].as_u64().unwrap() as usize
        );
        assert_eq!(
            camera["rgb_adr"].as_u64().unwrap() as usize,
            offset(&out.resolutions, index)
        );
        out.resolutions.push(resolution);
        out.rgb_packed.push(u32s(&camera["rgb_bits_packed"]));
    }
    for (index, camera) in nums(&depth_seg["cameras"]).iter().enumerate() {
        let resolution = out.resolutions[index];
        let start = offset(&out.resolutions, index);
        assert_eq!(camera["depth_adr"].as_u64().unwrap() as usize, start);
        assert_eq!(camera["seg_adr"].as_u64().unwrap() as usize, start);
        let bits = u32s(&camera["depth_bits"]);
        let pairs = i32s(&camera["seg_pairs"]);
        assert_eq!(
            (bits.len(), pairs.len()),
            (pixels(resolution), pixels(resolution) * 2)
        );
        out.depth_bits.push(bits);
        out.seg_pairs.push(pairs.as_chunks::<2>().0.to_vec());
    }
    out
}

impl References {
    fn total_pixels(&self) -> usize {
        self.resolutions.iter().copied().map(pixels).sum()
    }
    fn zero_depth_pixels(&self, camera: usize) -> usize {
        self.depth_bits[camera].iter().filter(|b| **b == 0).count()
    }
}

fn plan_for(resolutions: &[[u32; 2]]) -> CameraPlan {
    let total = resolutions.iter().copied().map(pixels).sum();
    CameraPlan {
        cameras: (0..resolutions.len()).collect(),
        resolutions: resolutions.to_vec(),
        render_rgb: vec![true; resolutions.len()],
        render_depth: vec![true; resolutions.len()],
        render_seg: vec![true; resolutions.len()],
        total_rays: total,
        rgb_pixels: total,
        samples_per_pixel: 1,
        samples: 1,
        znear: 0.05,
        zfar: 100.0,
    }
}

fn flatten<T: Copy>(per_camera: &[Vec<T>]) -> Vec<T> {
    per_camera.iter().flat_map(|v| v.iter().copied()).collect()
}
fn whole_rgb(references: &References) -> Vec<u32> {
    flatten(&references.rgb_packed)
}
fn whole_seg(references: &References) -> Vec<[i32; 2]> {
    flatten(&references.seg_pairs)
}
fn whole_depth(references: &References) -> Vec<f32> {
    references
        .depth_bits
        .iter()
        .flatten()
        .map(|b| f32::from_bits(*b))
        .collect()
}

/// 测试侧独立实现的上游 `unpack_rgb_kernel`（输出 r,g,b 的位型）。
fn expected_rgb(value: u32) -> [u32; 3] {
    let inv = f32::from_bits(INV_255_BITS);
    [
        (((value >> 16) & 0xFF) as f32 * inv).to_bits(),
        (((value >> 8) & 0xFF) as f32 * inv).to_bits(),
        ((value & 0xFF) as f32 * inv).to_bits(),
    ]
}

/// 测试侧独立实现的上游 `clamp(val / depth_scale, 0.0, 1.0)`。
fn expected_depth(value: f32, depth_scale: f32) -> f32 {
    (value / depth_scale).clamp(0.0, 1.0)
}

fn views_for<'a>(
    plan: &'a CameraPlan,
    layout: &'a OutputLayout,
    data: WholeOutputData<'a>,
) -> CameraOutputViews<'a> {
    CameraOutputViews::new(plan, layout, 1, data).expect("视图")
}

#[test]
fn rgb_views_match_frozen_packed_reference_and_upstream_unpack() {
    let references = references();
    assert_eq!(references.resolutions, vec![[320, 240], [240, 180]]);
    assert_eq!(references.total_pixels(), 120_000);
    assert_eq!(
        (
            references.zero_depth_pixels(0),
            references.zero_depth_pixels(1)
        ),
        (16_995, 27_498)
    );
    // 上游 `float32(1.0 / 255.0)`：double 1/255 舍入到 f32 == 冻结位型 == 产品常量。
    assert_eq!(((1.0f64 / 255.0) as f32).to_bits(), INV_255_BITS);
    assert_eq!(INV_255.to_bits(), INV_255_BITS);
    let plan = plan_for(&references.resolutions);
    let layout = OutputLayout::resolve(&plan).expect("布局");
    let rgb = whole_rgb(&references);
    let views = views_for(
        &plan,
        &layout,
        WholeOutputData {
            rgb: Some(&rgb),
            ..WholeOutputData::default()
        },
    );
    assert_eq!(
        (views.camera_count(), layout.rgb_adr.clone()),
        (2, vec![0, 76_800])
    );
    let (mut packed_equal, mut unpacked_equal) = (0usize, 0usize);
    for (index, packed) in references.rgb_packed.iter().enumerate() {
        let resolution = references.resolutions[index];
        assert_eq!(views.camera_id(index).expect("id"), index as i32);
        assert_eq!(views.resolution(index).expect("分辨率"), resolution);
        assert_eq!(views.rgb_packed(index).expect("packed"), packed.as_slice());
        packed_equal += packed.len();
        let image: CameraImage<[f32; 3]> = views.rgb(index).expect("解包");
        assert_eq!(
            (image.width(), image.height()),
            (resolution[0], resolution[1])
        );
        let (width, height) = (resolution[0] as usize, resolution[1] as usize);
        assert_eq!(
            image.row_major_index(width as u32 - 1, height as u32 - 1),
            Some(packed.len() - 1)
        );
        assert_eq!(image.row_major_index(width as u32, 0), None);
        let actual: Vec<[u32; 3]> = image.pixels().iter().map(|p| p.map(f32::to_bits)).collect();
        let expected: Vec<[u32; 3]> = packed.iter().map(|v| expected_rgb(*v)).collect();
        assert_eq!(
            actual, expected,
            "相机 {index} 解包须与上游 unpack 公式逐位相等"
        );
        unpacked_equal += packed.len() * 3;
    }
    assert_eq!((packed_equal, unpacked_equal), (120_000, 360_000));
}

#[test]
fn depth_and_segmentation_views_match_frozen_reference() {
    let references = references();
    let plan = plan_for(&references.resolutions);
    let layout = OutputLayout::resolve(&plan).expect("布局");
    let (depth, seg) = (whole_depth(&references), whole_seg(&references));
    let views = views_for(
        &plan,
        &layout,
        WholeOutputData {
            depth: Some(&depth),
            segmentation: Some(&seg),
            ..WholeOutputData::default()
        },
    );
    let (mut raw_equal, mut seg_equal) = (0usize, 0usize);
    for (index, expected_bits) in references.depth_bits.iter().enumerate() {
        let raw = views.depth_raw(index).expect("depth 区段");
        assert_eq!(
            raw.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            *expected_bits
        );
        raw_equal += raw.len();
        // `depth_scale = 1.0`：参考深度最大约 1.68 m，既覆盖原值也覆盖 clamp 到 1.0。
        for scale in [1.0f32, 0.1, 100.0] {
            let scaled = views.depth(index, scale).expect("缩放");
            for (value, expected) in scaled.pixels().iter().zip(expected_bits) {
                let expected = expected_depth(f32::from_bits(*expected), scale);
                assert_eq!(
                    value.to_bits(),
                    expected.to_bits(),
                    "相机 {index} depth/{scale}"
                );
            }
        }
        assert!(
            expected_bits
                .iter()
                .any(|b| expected_depth(f32::from_bits(*b), 1.0) == 1.0),
            "夹具须覆盖 clamp 到 1.0 的像素"
        );
    }
    for (index, expected) in references.seg_pairs.iter().enumerate() {
        assert_eq!(
            views.segmentation_raw(index).expect("seg"),
            expected.as_slice()
        );
        let image = views.segmentation(index).expect("分割图");
        assert_eq!(image.pixels(), expected.as_slice());
        let background = image
            .pixels()
            .iter()
            .filter(|pair| **pair == [-1, -1])
            .count();
        assert_eq!(
            background,
            references.zero_depth_pixels(index),
            "背景 (-1,-1) 与 depth 零像素数一致"
        );
        seg_equal += image.pixels().len();
    }
    assert_eq!((raw_equal, seg_equal), (120_000, 120_000));
}

#[test]
fn camera_regions_are_isolated_and_use_camera_local_row_major_order() {
    // 哨兵：A 相机 4×3（12 像素）、B 相机 2×2（4 像素），相机顺序与模型 id 相反。
    let mut plan = plan_for(&[[4, 3], [2, 2]]);
    plan.cameras = vec![1, 0];
    let layout = OutputLayout::resolve(&plan).expect("布局");
    assert_eq!(
        (layout.rgb_adr.clone(), layout.depth_adr.clone()),
        (vec![0, 12], vec![0, 12])
    );
    let rgb: Vec<u32> = (0..16u32).map(|g| g | (g << 8) | (g << 16)).collect();
    let depth: Vec<f32> = (0..16u32).map(|g| g as f32 + 0.25).collect();
    let seg: Vec<[i32; 2]> = (0..16i32).map(|g| [g, g / 12]).collect();
    let data = WholeOutputData {
        rgb: Some(&rgb),
        depth: Some(&depth),
        segmentation: Some(&seg),
    };
    let views = views_for(&plan, &layout, data);
    assert_eq!(
        (
            views.camera_id(0).expect("id"),
            views.camera_id(1).expect("id")
        ),
        (1, 0)
    );
    assert_eq!(views.rgb_packed(0).expect("A"), &rgb[..12]);
    assert_eq!(views.rgb_packed(1).expect("B"), &rgb[12..]);
    let (a, b) = (views.rgb(0).expect("A 解包"), views.rgb(1).expect("B 解包"));
    assert_eq!((a.width(), a.height(), b.width(), b.height()), (4, 3, 2, 2));
    for local in 0..12u32 {
        let expected = expected_rgb(local | (local << 8) | (local << 16));
        assert_eq!(a.pixels()[local as usize].map(f32::to_bits), expected);
        assert_eq!(
            a.get(local % 4, local / 4)
                .expect("A 像素")
                .map(f32::to_bits),
            expected
        );
    }
    let last = expected_rgb(15 | (15 << 8) | (15 << 16));
    assert_eq!(
        b.pixels()[3].map(f32::to_bits),
        last,
        "B 相机只含全局 12..16"
    );
    // depth 用 16.0 缩放：值域 (0,1)，可反解全局下标，且不跨相机读邻居。
    for (index, start, count) in [(0usize, 0u32, 12u32), (1, 12, 4)] {
        let image = views.depth(index, 16.0).expect("depth");
        assert_eq!(image.pixels().len(), count as usize);
        for local in 0..count {
            let expected = ((start + local) as f32 + 0.25) / 16.0;
            assert_eq!(image.pixels()[local as usize].to_bits(), expected.to_bits());
        }
    }
    assert_eq!(views.segmentation_raw(0).expect("A seg"), &seg[..12]);
    assert_eq!(views.segmentation_raw(1).expect("B seg"), &seg[12..]);
    assert_eq!(
        views.segmentation(1).expect("B seg 图").pixels()[0],
        [12, 1]
    );
}

#[test]
fn rejects_invalid_inputs_without_panicking() {
    let references = references();
    let plan = plan_for(&references.resolutions);
    let layout = OutputLayout::resolve(&plan).expect("布局");
    let (rgb, depth) = (whole_rgb(&references), whole_depth(&references));
    let seg = whole_seg(&references);
    let data = WholeOutputData {
        rgb: Some(&rgb),
        depth: Some(&depth),
        segmentation: Some(&seg),
    };
    let views = views_for(&plan, &layout, data);
    // 越界 camera_index（激活相机顺序位置，不是模型相机 id）。
    for index in [2usize, 9, usize::MAX] {
        assert_eq!(
            views.rgb(index).unwrap_err(),
            CameraViewError::CameraIndex { index, cameras: 2 }
        );
        assert_eq!(
            views.camera_id(index).unwrap_err(),
            CameraViewError::CameraIndex { index, cameras: 2 }
        );
    }
    // nworld>1 本片拒绝；通道未提供；长度不符。
    assert_eq!(
        CameraOutputViews::new(&plan, &layout, 2, data).unwrap_err(),
        CameraViewError::NworldUnsupported { nworld: 2 }
    );
    let empty = views_for(&plan, &layout, WholeOutputData::default());
    assert_eq!(
        empty.rgb(0).unwrap_err(),
        CameraViewError::BufferMissing {
            channel: ImageOutput::Rgb
        }
    );
    let short = WholeOutputData {
        rgb: Some(&rgb[..rgb.len() - 1]),
        ..data
    };
    assert_eq!(
        CameraOutputViews::new(&plan, &layout, 1, short).unwrap_err(),
        CameraViewError::BufferLength {
            channel: ImageOutput::Rgb,
            expected: 120_000,
            actual: 119_999
        }
    );
    // 通道未启用：该相机 depth_adr = -1（只提供启用相机的 depth 段）。
    let mut depth_off = plan_for(&references.resolutions);
    depth_off.render_depth = vec![true, false];
    let off_layout = OutputLayout::resolve(&depth_off).expect("布局");
    assert_eq!(off_layout.depth_adr, vec![0, -1]);
    let depth_only = depth[..76_800].to_vec();
    let off_data = WholeOutputData {
        depth: Some(&depth_only),
        ..WholeOutputData::default()
    };
    let partial = views_for(&depth_off, &off_layout, off_data);
    assert_eq!(partial.depth_raw(0).expect("相机 0").len(), 76_800);
    assert_eq!(
        partial.depth_raw(1).unwrap_err(),
        CameraViewError::ChannelDisabled {
            channel: ImageOutput::Depth,
            camera_index: 1
        }
    );
    // depth_scale 非法：<=0 或非有限。
    for depth_scale in [0.0f32, -1.0, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            views.depth(0, depth_scale).unwrap_err(),
            CameraViewError::DepthScale { depth_scale }
        );
    }
    assert!(matches!(
        views.depth(0, f32::NAN).unwrap_err(),
        CameraViewError::DepthScale { .. }
    ));
    // plan/layout 不一致：相机数、开关表长度、通道像素总数、区段越界。
    let three_layout = OutputLayout::resolve(&plan_for(&[[2, 2], [2, 2], [2, 2]])).expect("布局");
    assert!(matches!(
        CameraOutputViews::new(&plan, &three_layout, 1, WholeOutputData::default()).unwrap_err(),
        CameraViewError::CameraCount {
            expected: 2,
            actual: 3
        }
    ));
    let mut flags = plan.clone();
    flags.render_seg = vec![true];
    assert!(matches!(
        CameraOutputViews::new(&flags, &layout, 1, WholeOutputData::default()).unwrap_err(),
        CameraViewError::PlanFlags {
            channel: ImageOutput::Segmentation,
            cameras: 2,
            actual: 1
        }
    ));
    let mut flipped = plan.clone();
    flipped.render_depth = vec![true, false];
    let rgb_only = WholeOutputData {
        rgb: Some(&vec![0u32; 120_000]),
        ..WholeOutputData::default()
    };
    assert!(matches!(
        CameraOutputViews::new(&flipped, &layout, 1, rgb_only).unwrap_err(),
        CameraViewError::LayoutPixels {
            channel: ImageOutput::Depth,
            expected: 76_800,
            actual: 120_000
        }
    ));
    let mut broken = layout.clone();
    broken.depth_adr = vec![0, 90_000];
    let broken_data = WholeOutputData {
        depth: Some(&depth),
        ..WholeOutputData::default()
    };
    let broken_views = views_for(&plan, &broken, broken_data);
    assert_eq!(
        broken_views.depth_raw(1).unwrap_err(),
        CameraViewError::BufferRange {
            channel: ImageOutput::Depth,
            camera_index: 1,
            end: 133_200,
            actual: 120_000
        }
    );
    // 错误文本可诊断且不空。
    assert!(empty.rgb(0).unwrap_err().to_string().contains("rgb"));
}
