#![allow(dead_code)]
//! R05-S3b 风格的 SDF 捕获参考读取（读 U3 `2026-10-30-render-sdf-capture` 采集窗产物，CPU only）。
//!
//! - 参考来自 `fixtures/render-bvh/scene-sdf/capture-sdf/{sdf-mesh,sdf-occluder}/`
//!   的 `rgb.npz` / `depth.npz` / `seg.npz` / `capture.json`（U3 采集窗，attempt=1，
//!   pinned MJWarp 3.15 + warp 1.15，`use_fast_math=false`、`use_shadows=true`、`spp=1`）。
//! - 本模块**只读**：不重新采集、不修改夹具；npz 解析复用 `shadow.rs` 的最小解析器
//!   （ZIP_STORED + 小写端定长类型 + C 顺序），不新增依赖。
//! - 参考完整性：npz 文件 sha256 必须与 `capture.json.files_sha256` 逐项一致；capture.json
//!   的 task_id/kind/group/recipe/cameras/fast-math/shadows/legacy 全部做硬断言。
//! - 数值口径与 R09-S3b 相同：RGB 参考是 `get_rgb` 解包的 f32（`byte/255`），按
//!   `round(v*255)` 还原 packed u32；depth 参考是 raw f32 位型；seg 参考是
//!   `(object_id, object_type)` int32 对。
//! - 本模块不产生任何渲染数值结论；只把参考转成可比较结构。

use mjwarp_rs::render::kernel::output::RefCamera;
use serde_json::Value;

use super::s7b2::RgbCamera;
use super::shadow::{
    f32_member, f32_member_rgb, i32_member, quantize_channel, read_npz, sha256_hex,
};

/// U3 采集任务 id（capture.json `task_id`）。
pub const CAPTURE_TASK_ID: &str = "2026-10-30-render-sdf-capture";
/// 签字采集脚本 sha256（U3 收口值）。
pub const SCRIPT_SHA256: &str = "8064596876aeec3ee2fe1f63e83d7de87f898a8dbe9a1ef64f64026de8a77ba9";
/// 采集时 legacy 参考文件数（SDF 采集窗自检值）。
pub const LEGACY_FILES: u64 = 69;

const SCENE_MESH_SHA256: &str = "376eb81ce200b29373af3965c7abefd90b8f00188a688db583080836dfa868e8";
const SCENE_OCCLUDER_SHA256: &str =
    "7869cfae5e16795ebf86b93c931ef36a8e94464452762e28f1357e0e68a1584e";

/// 捕获组（与 `--group` 取值一致）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SdfGroup {
    /// `scene.xml`：平面 + SDF cube（mesh 底层）+ 球 + 两相机 + 两灯。
    Mesh,
    /// `scene-mixed.xml`：在上组基础上加 1 个静态 box 遮挡体。
    Occluder,
}

impl SdfGroup {
    /// 产物子目录名。
    pub fn directory(self) -> &'static str {
        match self {
            Self::Mesh => "sdf-mesh",
            Self::Occluder => "sdf-occluder",
        }
    }

    /// 报告用标签（与目录名一致）。
    pub fn label(self) -> &'static str {
        self.directory()
    }

    /// 该组签字场景 sha256。
    pub fn scene_sha256(self) -> &'static str {
        match self {
            Self::Mesh => SCENE_MESH_SHA256,
            Self::Occluder => SCENE_OCCLUDER_SHA256,
        }
    }

    /// 该组模型的 geom 数（enabled_geom_ids 期望值）。
    pub fn expected_geoms(self) -> usize {
        match self {
            Self::Mesh => 3,
            Self::Occluder => 4,
        }
    }

    /// 捕获树中的组目录（repo 相对路径）。
    pub fn capture_dir(self) -> String {
        format!(
            "fixtures/render-bvh/scene-sdf/capture-sdf/{}",
            self.directory()
        )
    }
}

/// 一组 SDF 捕获参考：两相机 RGB packed + depth/seg。
#[derive(Clone, Debug)]
pub struct SdfReference {
    pub group: SdfGroup,
    pub rgb: Vec<RgbCamera>,
    pub depth_seg: Vec<RefCamera>,
    pub total_pixels: usize,
    pub captured_at: String,
}

impl SdfReference {
    /// 相机数（RGB）。
    pub fn cameras(&self) -> usize {
        self.rgb.len()
    }
}

/// 读取并校验一组参考；**只在真实捕获产物上调用**（GPU 实体与 host 解析用例）。
pub fn load(group: SdfGroup) -> SdfReference {
    let directory = group.capture_dir();
    let capture_path = format!("{directory}/capture.json");
    let capture_text = std::fs::read_to_string(&capture_path).unwrap_or_else(|error| {
        panic!(
            "缺少 SDF 捕获参考 {capture_path}（{error}）；该路径由 U3 采集窗产出，\
             只在真实捕获产物上启用"
        )
    });
    let capture: Value = serde_json::from_str(&capture_text)
        .unwrap_or_else(|error| panic!("{capture_path} 不是合法 JSON：{error}"));
    verify_capture(&capture, group);

    let mut parts: Vec<Vec<u8>> = Vec::with_capacity(3);
    for name in ["rgb.npz", "depth.npz", "seg.npz"] {
        let path = format!("{directory}/{name}");
        let bytes =
            std::fs::read(&path).unwrap_or_else(|error| panic!("缺少捕获参考 {path}（{error}）"));
        let expected = text_at(&capture, &format!("files_sha256/{name}")).to_string();
        assert_eq!(
            sha256_hex(&bytes),
            expected,
            "{path} sha256 与 capture.json.files_sha256 不符（参考被动过或夹具替换）"
        );
        parts.push(bytes);
    }

    let rgb_members = read_npz(&parts[0]);
    let depth_members = read_npz(&parts[1]);
    let seg_members = read_npz(&parts[2]);
    let cameras = rgb_members.len();
    assert!(cameras > 0, "{} rgb.npz 成员为空", group.label());
    assert_eq!(
        depth_members.len(),
        cameras,
        "{} depth 相机数",
        group.label()
    );
    assert_eq!(seg_members.len(), cameras, "{} seg 相机数", group.label());

    let mut rgb = Vec::with_capacity(cameras);
    let mut depth_seg = Vec::with_capacity(cameras);
    let mut total_pixels = 0usize;
    let mut depth_adr = 0i32;
    let mut seg_adr = 0i32;
    for camera in 0..cameras {
        let ((height, width), values) =
            f32_member_rgb(&rgb_members, &format!("rgb_cam{camera}.npy"));
        let pixels = width * height;
        let mut packed = Vec::with_capacity(pixels);
        for index in 0..pixels {
            let base = 3 * index;
            let r = quantize_channel(values[base], "rgb.r", group.label(), camera);
            let g = quantize_channel(values[base + 1], "rgb.g", group.label(), camera);
            let b = quantize_channel(values[base + 2], "rgb.b", group.label(), camera);
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
    SdfReference {
        group,
        rgb,
        depth_seg,
        total_pixels,
        captured_at: text_at(&capture, "captured_at").to_string(),
    }
}

fn verify_capture(capture: &Value, group: SdfGroup) {
    let label = group.label();
    assert_eq!(
        text_at(capture, "task_id"),
        CAPTURE_TASK_ID,
        "{label} capture.json task_id"
    );
    assert_eq!(
        text_at(capture, "kind"),
        "render-sdf-capture",
        "{label} capture.json kind"
    );
    assert_eq!(
        text_at(capture, "group"),
        label,
        "{label} capture.json group"
    );
    assert!(
        bool_at(capture, "two_captures_bitwise_identical"),
        "{label} A/B 两次采集必须逐字节一致"
    );
    assert!(
        bool_at(capture, "legacy_reference_untouched"),
        "{label} 旧参考未动标志"
    );
    assert_eq!(
        number_at(capture, "legacy_reference_files"),
        LEGACY_FILES,
        "{label} 旧参考文件数"
    );
    assert!(
        !bool_at(capture, "use_fast_math"),
        "{label} 根 use_fast_math"
    );
    assert!(bool_at(capture, "use_shadows"), "{label} 根 use_shadows");
    assert_eq!(
        number_at(capture, "repeat"),
        2,
        "{label} repeat 必须为 2（A/B）"
    );
    // recipe：场景/脚本 sha、组名、fast-math/shadows、npz 清单。
    assert_eq!(
        text_at(capture, "recipe/group"),
        label,
        "{label} recipe.group"
    );
    assert_eq!(
        text_at(capture, "recipe/scene_xml/sha256"),
        group.scene_sha256(),
        "{label} recipe.scene_xml.sha256 必须等于签字场景 sha"
    );
    assert_eq!(
        text_at(capture, "recipe/script/sha256"),
        SCRIPT_SHA256,
        "{label} recipe.script.sha256 必须等于签字脚本 sha"
    );
    assert!(
        !bool_at(capture, "recipe/use_fast_math"),
        "{label} recipe.use_fast_math"
    );
    assert!(
        bool_at(capture, "recipe/use_shadows"),
        "{label} recipe.use_shadows"
    );
    let npz_files = capture
        .pointer("/recipe/npz_files")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{label} 缺少 recipe.npz_files"));
    let names: Vec<&str> = npz_files.iter().filter_map(Value::as_str).collect();
    assert_eq!(
        names,
        ["rgb.npz", "depth.npz", "seg.npz"],
        "{label} npz 清单"
    );
    // 同配方口径：use_fast_math 四处一致 false；resolved use_shadows=true、spp=1。
    for path in [
        "parameters/use_fast_math",
        "context/parameters/use_fast_math",
        "context/resolved/use_fast_math",
    ] {
        assert!(!bool_at(capture, path), "{label} {path}");
    }
    assert!(
        bool_at(capture, "context/resolved/use_shadows"),
        "{label} context.resolved.use_shadows"
    );
    assert_eq!(
        number_at(capture, "context/resolved/samples_per_pixel"),
        1,
        "{label} samples_per_pixel"
    );
    assert_eq!(
        number_at(capture, "context/resolved/nrender"),
        2,
        "{label} nrender"
    );
    assert_eq!(
        number_at(capture, "context/resolved/total_rays"),
        120_000,
        "{label} total_rays"
    );
    // 相机 recipe：两相机、分辨率与 npz 形状一致。
    let cameras = capture
        .pointer("/recipe/cameras")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{label} 缺少 recipe.cameras"));
    assert_eq!(cameras.len(), 2, "{label} 相机数");
    let names: Vec<&str> = cameras
        .iter()
        .map(|camera| camera["name"].as_str().expect("camera name"))
        .collect();
    assert_eq!(names, ["cam-front", "cam-top"], "{label} 相机名");
    let resolutions: Vec<(u64, u64)> = cameras
        .iter()
        .map(|camera| {
            let resolution = camera["resolution"]
                .as_array()
                .unwrap_or_else(|| panic!("{label} 相机 resolution"));
            (
                resolution[0].as_u64().expect("resolution w"),
                resolution[1].as_u64().expect("resolution h"),
            )
        })
        .collect();
    assert_eq!(resolutions, [(320, 240), (240, 180)], "{label} 相机分辨率");
}

impl SdfReference {
    /// 参考场景 sha（签字值；供 host 用例断言）。
    pub fn scene_sha256(&self) -> &'static str {
        self.group.scene_sha256()
    }
}

fn text_at<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(&format!("/{path}"))
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("capture.json 缺少字符串字段 {path}"))
}

fn bool_at(value: &Value, path: &str) -> bool {
    value
        .pointer(&format!("/{path}"))
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("capture.json 缺少布尔字段 {path}"))
}

fn number_at(value: &Value, path: &str) -> u64 {
    value
        .pointer(&format!("/{path}"))
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("capture.json 缺少数值字段 {path}"))
}
