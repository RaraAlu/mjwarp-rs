//! R02-S1 天空盒参考（scene-a，`render_skybox=true`，stage A 采集）与产品路径位级比较。
//!
//! 参考由 `scripts/authoring/render-bvh-r02/capture_skybox.py` 在 pinned mujoco_warp 3.15
//! 上采集：`fixtures/render-bvh/scene-a/reference-skybox/{rgb,depth,seg}.npz + capture.json`
//! （天空盒是竖排 2D：`tex_width[2]=64`、`tex_height[2]=384 = 6*64`、`tex_type[2]=2`）。
//!
//! 本模块**只读**参考：不采集、不改夹具；npz 解析与 sha 复用 `shadow.rs` 的最小解析器，
//! RGB 量化/差异复用 `s7b2::diff_rgb`，depth/seg 复用 `kernel::output::{diff_depth, diff_seg}`。
//! 判据与 R05 完全一致（不新增/不放宽）：RGB 每通道 ≤1 LSB、depth 位级优先（`mismatched==0`
//! 且 `max_ulp<=4` 且 `beyond_ulp==0`）、seg 逐像素全等、两次运行 sha256 相同。

use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{DepthDiff, MAX_DEPTH_ULP, RefCamera, SegDiff, diff_seg};
use mjwarp_rs::render::{ContextParams, RayBuffer, RenderContext, SceneState, resolve_context};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

use super::s7b2::{RgbCamera, RgbStats, diff_rgb};
use super::shadow::{
    f32_member, f32_member_rgb, i32_member, quantize_channel, read_npz, sha256_hex,
};

/// stage A 采集目录（repo 相对路径）。
pub const CAPTURE_DIR: &str = "fixtures/render-bvh/scene-a/reference-skybox";
/// 两相机总像素（320x240 + 240x180）。
pub const EXPECTED_PIXELS: usize = 120_000;
/// 天空盒纹理 id 与面宽（scene-a：第 3 张纹理是 skybox，64x384 竖排）。
pub const SKYBOX_TEX_ID: i32 = 2;
pub const SKYBOX_FACE_WIDTH: u32 = 64;
pub const SKYBOX_TEX_HEIGHT: i32 = 384;
/// 两次运行必须位级相同的取样数（相机数）。
pub const CAMERAS: usize = 2;

/// 一组天空盒参考：两相机 RGB packed + depth/seg。
#[derive(Clone, Debug)]
pub struct SkyboxReference {
    pub rgb: Vec<RgbCamera>,
    pub depth_seg: Vec<RefCamera>,
    pub total_pixels: usize,
    pub captured_at: String,
    pub face_width: u32,
    pub tex_id: i32,
}

impl SkyboxReference {
    pub fn cameras(&self) -> usize {
        self.rgb.len()
    }
}

/// 参考目录是否已由 stage A 产出（host 自检在缺失时跳过，不 panic）。
pub fn reference_exists() -> bool {
    std::path::Path::new(CAPTURE_DIR)
        .join("capture.json")
        .exists()
}

/// 读取并校验参考；**只在 stage A 产物存在后调用**（GPU 实体与 host 解析用例）。
pub fn load() -> SkyboxReference {
    let capture_path = format!("{CAPTURE_DIR}/capture.json");
    let capture_text = std::fs::read_to_string(&capture_path).unwrap_or_else(|error| {
        panic!("缺少 R02 stage A 天空盒参考 {capture_path}（{error}）；先跑 capture_skybox.py")
    });
    let capture: Value = serde_json::from_str(&capture_text)
        .unwrap_or_else(|error| panic!("{capture_path} 不是合法 JSON：{error}"));
    verify_capture(&capture);

    let mut parts: Vec<Vec<u8>> = Vec::with_capacity(3);
    for name in ["rgb.npz", "depth.npz", "seg.npz"] {
        let path = format!("{CAPTURE_DIR}/{name}");
        let bytes =
            std::fs::read(&path).unwrap_or_else(|error| panic!("缺少参考 {path}（{error}）"));
        let expected = file_sha(&capture, name).to_string();
        assert_eq!(
            sha256_hex(&bytes),
            expected,
            "{path} sha256 与 capture.json.files 不符（参考被动过或夹具替换）"
        );
        parts.push(bytes);
    }

    let rgb_members = read_npz(&parts[0]);
    let depth_members = read_npz(&parts[1]);
    let seg_members = read_npz(&parts[2]);
    let cameras = rgb_members.len();
    assert_eq!(cameras, CAMERAS, "参考相机数");
    assert_eq!(depth_members.len(), cameras, "depth 相机数");
    assert_eq!(seg_members.len(), cameras, "seg 相机数");

    let mut rgb = Vec::with_capacity(cameras);
    let mut depth_seg = Vec::with_capacity(cameras);
    let mut total_pixels = 0usize;
    let (mut depth_adr, mut seg_adr) = (0i32, 0i32);
    for camera in 0..cameras {
        let ((height, width), values) =
            f32_member_rgb(&rgb_members, &format!("rgb_cam{camera}.npy"));
        let pixels = width * height;
        let mut packed = Vec::with_capacity(pixels);
        for index in 0..pixels {
            let base = 3 * index;
            let r = quantize_channel(values[base], "rgb.r", "r02-s1", camera);
            let g = quantize_channel(values[base + 1], "rgb.g", "r02-s1", camera);
            let b = quantize_channel(values[base + 2], "rgb.b", "r02-s1", camera);
            packed
                .push((0xFFu32 << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
        }
        rgb.push(RgbCamera {
            index: camera,
            width,
            height,
            pixels,
            packed,
        });

        let ((depth_height, depth_width), depth_values) =
            f32_member(&depth_members, &format!("depth_cam{camera}.npy"));
        assert_eq!((depth_height, depth_width), (height, width), "depth 形状");
        let depth_bits: Vec<u32> = depth_values.iter().map(|value| value.to_bits()).collect();
        let depth_zero_pixels = depth_bits.iter().filter(|bits| **bits == 0).count();
        let depth_max_hex = depth_values
            .iter()
            .max_by(|left, right| left.partial_cmp(right).expect("depth 参考不得含 NaN"))
            .map(|value| format!("0x{:08x}", value.to_bits()))
            .unwrap_or_else(|| "0x00000000".to_string());

        let ((seg_height, seg_width, seg_components), seg_values) =
            i32_member(&seg_members, &format!("seg_cam{camera}.npy"));
        assert_eq!(
            (seg_height, seg_width, seg_components),
            (height, width, 2),
            "seg 形状"
        );
        let (seg_chunks, seg_remainder) = seg_values.as_chunks::<2>();
        assert!(seg_remainder.is_empty(), "seg 数据长度");
        let seg_pairs: Vec<[i32; 2]> = seg_chunks.to_vec();
        assert_eq!(seg_pairs.len(), pixels, "seg 像素数");
        let mut seg_unique: Vec<[i32; 2]> = seg_pairs.clone();
        seg_unique.sort_unstable();
        seg_unique.dedup();

        depth_seg.push(RefCamera {
            index: camera,
            width,
            height,
            pixels,
            depth_adr,
            seg_adr,
            depth_zero_pixels,
            depth_max_hex,
            depth_bits,
            seg_pairs,
            seg_unique,
        });
        depth_adr += pixels as i32;
        seg_adr += pixels as i32;
        total_pixels += pixels;
    }
    let face_width = capture
        .pointer("/skybox/skybox_face_width")
        .and_then(Value::as_u64)
        .expect("capture.json 缺 skybox.skybox_face_width") as u32;
    let tex_id = capture
        .pointer("/skybox/skybox_tex_id")
        .and_then(Value::as_i64)
        .expect("capture.json 缺 skybox.skybox_tex_id") as i32;
    SkyboxReference {
        rgb,
        depth_seg,
        total_pixels,
        captured_at: text_at(&capture, "captured_at").to_string(),
        face_width,
        tex_id,
    }
}

fn verify_capture(capture: &Value) {
    assert_eq!(
        capture.pointer("/schema").and_then(Value::as_u64),
        Some(1),
        "capture.json schema"
    );
    assert_eq!(
        text_at(capture, "kind"),
        "r02-s1-skybox-reference",
        "capture.json kind"
    );
    assert_eq!(
        capture
            .pointer("/params28/render_skybox")
            .and_then(Value::as_bool),
        Some(true),
        "捕获必须是 render_skybox=true"
    );
    assert_eq!(
        capture.pointer("/expected_pixels").and_then(Value::as_u64),
        Some(EXPECTED_PIXELS as u64),
        "参考总像素"
    );
    assert_eq!(
        capture
            .pointer("/skybox/skybox_tex_id")
            .and_then(Value::as_i64),
        Some(SKYBOX_TEX_ID as i64),
        "天空盒纹理 id"
    );
    assert_eq!(
        capture
            .pointer("/skybox/skybox_face_width")
            .and_then(Value::as_u64),
        Some(u64::from(SKYBOX_FACE_WIDTH)),
        "天空盒 face_width 必须等于 tex_width"
    );
    assert_eq!(
        capture
            .pointer("/skybox/skybox_tex_height")
            .and_then(Value::as_i64),
        Some(SKYBOX_TEX_HEIGHT as i64),
        "天空盒必须是竖排 2D（tex_height = 6*tex_width）"
    );
    assert_eq!(
        capture
            .pointer("/determinism/sha_equal")
            .and_then(Value::as_bool),
        Some(true),
        "两次运行 sha 必须相等"
    );
    let cam_res = capture
        .pointer("/cameras/cam_res")
        .and_then(Value::as_array)
        .expect("capture.json 缺 cameras.cam_res");
    assert_eq!(cam_res.len(), CAMERAS, "参考相机数");
    assert_eq!(
        cam_res[0].as_array().map(|values| values.len()),
        Some(2),
        "cam0 分辨率"
    );
}

fn file_sha<'a>(capture: &'a Value, name: &str) -> &'a str {
    capture
        .pointer("/files")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("capture.json 缺 files"))
        .iter()
        .find(|entry| {
            entry
                .get("path")
                .and_then(Value::as_str)
                .is_some_and(|path| path.ends_with(name))
        })
        .and_then(|entry| entry.get("sha256"))
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("capture.json files 缺 {name}"))
}

fn text_at<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(&format!("/{path}"))
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("capture.json 缺 {path}"))
}

/// 产品路径场景：scene-a + `render_skybox=true`，纹理注册表按「use_textures || skybox」构造。
pub struct SkyboxScene {
    pub context: RenderContext,
    pub fields: mjwarp_rs::model::render::RenderFields,
    pub model: CameraModel,
    pub poses: Vec<CameraPose>,
    pub rays: RayBuffer,
}

fn skybox_params<'a>() -> ContextParams<'a> {
    const GROUPS: [i32; 4] = [0, 1, 2, 3];
    ContextParams {
        nworld: 1,
        enabled_geom_groups: &GROUPS,
        render_skybox: true,
        ..Default::default()
    }
    .with_render_targets(
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(true)),
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(true)),
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(true)),
    )
}

pub fn scene(
    session: &TransferSession,
    reference: &mjwarp_rs::render::kernel::output::DepthSegReference,
) -> SkyboxScene {
    let fields = super::s7b2::render_fields();
    let params = skybox_params();
    let resolved = resolve_context(&fields, &params).expect("context 解析");
    assert!(
        resolved.render_skybox && resolved.skybox.is_some(),
        "scene-a 必须有天空盒纹理且 render_skybox=true"
    );
    let textures = mjwarp_rs::render::context::texture_registry(
        &fields,
        params.use_textures,
        resolved.skybox.as_ref(),
    )
    .expect("天空盒纹理注册表");
    let fixture = super::s7b1::scene_fixture();
    let meshes = fixture.meshes.clone();
    let state = SceneState::new(fixture.leaves, fixture.poses, 1).expect("场景状态");
    let context =
        RenderContext::create(session, resolved, state, meshes, &textures).expect("context 创建");
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = super::s7b2::camera_poses(reference, context.resolved());
    let rays = super::s7b2::precomputed_rays(session, &fields);
    SkyboxScene {
        context,
        fields,
        model,
        poses,
        rays,
    }
}

/// 渲染一帧并读回 rgb/depth/seg。
#[derive(Clone, Debug)]
pub struct Rendered {
    pub rgb: Vec<u32>,
    pub depth: Vec<f32>,
    pub seg: Vec<[i32; 2]>,
}

pub fn fire(
    session: &TransferSession,
    renderer: &MegakernelRenderer,
    scene: &SkyboxScene,
) -> Rendered {
    let _ = session;
    renderer
        .render(
            &scene.context,
            &scene.fields,
            &scene.model,
            &scene.poses,
            &scene.rays,
        )
        .expect("天空盒渲染发射");
    Rendered {
        rgb: super::s7b1::rgb_of(&scene.context),
        depth: super::s7b1::read_depth(&scene.context),
        seg: super::s7b1::read_seg(&scene.context),
    }
}

/// 逐相机位级比较（复用 S7b1/S7b2 diff 口径）。
pub fn compare(
    scene: &SkyboxScene,
    reference: &SkyboxReference,
    rendered: &Rendered,
) -> (Vec<RgbStats>, Vec<DepthDiff>, Vec<SegDiff>) {
    let layout = &scene.context.resolved().output;
    let mut rgb_stats = Vec::with_capacity(reference.rgb.len());
    for camera in &reference.rgb {
        let start = layout.rgb_adr[camera.index] as usize;
        rgb_stats.push(diff_rgb(
            &rendered.rgb[start..start + camera.pixels],
            camera,
        ));
    }
    let mut depth_stats = Vec::with_capacity(reference.depth_seg.len());
    let mut seg_stats = Vec::with_capacity(reference.depth_seg.len());
    for camera in &reference.depth_seg {
        let depth_start = layout.depth_adr[camera.index] as usize;
        depth_stats.push(super::s7b1::outline::diff_depth_with_outline(
            &rendered.depth[depth_start..depth_start + camera.pixels],
            camera,
            super::s7b1::outline::Arm::Recipe,
        ));
        let seg_start = layout.seg_adr[camera.index] as usize;
        seg_stats.push(
            diff_seg(&rendered.seg[seg_start..seg_start + camera.pixels], camera)
                .expect("seg diff"),
        );
    }
    (rgb_stats, depth_stats, seg_stats)
}

/// 判据断言（与 R05 完全一致，不新增/不放宽）。
pub fn assert_criteria(
    label: &str,
    reference: &SkyboxReference,
    rgb_stats: &[RgbStats],
    depth_stats: &[DepthDiff],
    seg_stats: &[SegDiff],
) {
    for (camera, stats) in reference.rgb.iter().zip(rgb_stats) {
        assert_eq!(
            stats.beyond_one_lsb_pixels,
            0,
            "{label} cam{} RGB 超 1 LSB：{} / {} 像素，max_byte_diff={} first={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.compared,
            stats.max_byte_diff,
            stats.first
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(depth_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} depth 必须与参考位级相同（first={:?}）",
            camera.index, diff.first
        );
        assert!(
            diff.max_ulp <= MAX_DEPTH_ULP,
            "{label} cam{} depth max_ulp={} 超 {}",
            camera.index,
            diff.max_ulp,
            MAX_DEPTH_ULP
        );
        assert_eq!(
            diff.beyond_ulp, 0,
            "{label} cam{} depth 超 4 ULP 像素 {}",
            camera.index, diff.beyond_ulp
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(seg_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} seg 必须逐像素全等（first={:?}）",
            camera.index, diff.first
        );
    }
}

/// 相机摘要（打印比较规模/超判据计数）。
pub fn camera_summary(camera: &RefCamera, depth: &DepthDiff, seg: &SegDiff) -> String {
    format!(
        "cam{} pixels={} depth_mismatch={} max_ulp={} beyond_ulp={} seg_mismatch={} depth_zero_pixels={}",
        camera.index,
        camera.pixels,
        depth.mismatched,
        depth.max_ulp,
        depth.beyond_ulp,
        seg.mismatched,
        camera.depth_zero_pixels
    )
}

/// 渲染输出 sha256（两次运行相同判据）。
pub fn hash_rgb(values: &[u32]) -> String {
    sha256_hex(
        &values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<u8>>(),
    )
}

pub fn hash_depth(values: &[f32]) -> String {
    sha256_hex(
        &values
            .iter()
            .flat_map(|value| value.to_bits().to_le_bytes())
            .collect::<Vec<u8>>(),
    )
}

pub fn hash_seg(values: &[[i32; 2]]) -> String {
    sha256_hex(
        &values
            .iter()
            .flat_map(|pair| [pair[0].to_le_bytes(), pair[1].to_le_bytes()])
            .flatten()
            .collect::<Vec<u8>>(),
    )
}
