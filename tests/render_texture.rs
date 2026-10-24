//! 渲染/BVH S4 验收：纹理 u8→f32/255 转换、纹理注册表与硬件 LINEAR 采样。
//!
//! 参考：`fixtures/render-bvh/scene-a/s4/textures.json`，由离线脚本
//! `scripts/authoring/render-bvh-s4/capture_s4_textures.py` 在 GPU 上用冻结上游
//! MJWarp 3.15（`render_util._convert_texture_data` / `create_warp_texture` /
//! `wp.texture_sample`）捕获。产品测试只读静态夹具，不启动 Python。
//!
//! 主机测试默认执行；GPU 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_texture -- --ignored --test-threads=1

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::texture::{TextureData, TextureError, TextureRegistry};
use serde::Deserialize;
use sha2::{Digest, Sha256};

const TEXTURES: &str = include_str!("../fixtures/render-bvh/scene-a/s4/textures.json");
const MODEL_312: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");

#[derive(Deserialize)]
struct Fixture {
    task_id: String,
    textures: Vec<TextureMeta>,
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    samples: Samples,
    synthetic: Vec<Synthetic>,
    registry: Registry,
    rejections: Rejections,
    hardware_vs_software: HardwareSoftware,
    cross_check_312: CrossCheck,
    determinism: Determinism,
}

#[derive(Deserialize)]
struct TextureMeta {
    tex_id: i32,
    width: u32,
    height: u32,
    nchannel: u8,
    adr: u64,
    data_sha256: String,
    converted_sha256: String,
    converted_texels: usize,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
#[derive(Deserialize)]
struct Samples {
    uv_bits: Vec<u32>,
    gpu: Vec<GpuCase>,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
#[derive(Deserialize)]
struct GpuCase {
    rgba_bits: Vec<u32>,
}

#[derive(Deserialize)]
struct Synthetic {
    name: String,
    width: u32,
    height: u32,
    nchannel: u8,
    data_bits: Vec<u32>,
    converted_bits: Vec<u32>,
}

#[derive(Deserialize)]
struct Registry {
    ntex: usize,
    registry_off: usize,
    registry_on: usize,
    use_textures_off: bool,
    use_textures_on: bool,
}

#[derive(Deserialize)]
struct Rejections {
    zero_size: ZeroSize,
}

#[derive(Deserialize)]
struct ZeroSize {
    constructible: bool,
    exception: String,
    message: String,
}

#[derive(Deserialize)]
struct HardwareSoftware {
    max_abs_diff: f64,
}

#[derive(Deserialize)]
struct CrossCheck {
    tex_metadata_equal: bool,
    tex_data_equal: bool,
    tex_data_bytes: usize,
}

#[derive(Deserialize)]
struct Determinism {
    bitwise_identical: bool,
}

fn load() -> Fixture {
    serde_json::from_str(TEXTURES).expect("S4 夹具 JSON")
}

fn model_312_tex_data() -> Vec<u8> {
    let document: serde_json::Value = serde_json::from_str(MODEL_312).expect("model-312 JSON");
    document["arrays"]["tex_data"]["values"]
        .as_array()
        .expect("tex_data values")
        .iter()
        .map(|value| value.as_u64().expect("u8") as u8)
        .collect()
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn points2(bits: &[u32]) -> Vec<[f32; 2]> {
    bits.chunks(2)
        .map(|c| [f32::from_bits(c[0]), f32::from_bits(c[1])])
        .collect()
}

fn points4(bits: &[u32]) -> Vec<[f32; 4]> {
    bits.chunks(4)
        .map(|c| {
            [
                f32::from_bits(c[0]),
                f32::from_bits(c[1]),
                f32::from_bits(c[2]),
                f32::from_bits(c[3]),
            ]
        })
        .collect()
}

fn digest_f32(values: &[f32]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.to_bits().to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn empty_fields() -> Vec<FieldData> {
    RENDER_FIELDS
        .iter()
        .map(|spec| match spec.kind {
            RenderFieldKind::Index => FieldData::Index(Vec::new()),
            RenderFieldKind::Num => FieldData::Num(Vec::new()),
            RenderFieldKind::Flag => FieldData::Flag(Vec::new()),
            RenderFieldKind::Size => FieldData::Size(Vec::new()),
            RenderFieldKind::Float => FieldData::Float(Vec::new()),
        })
        .collect()
}

fn set_field(values: &mut [FieldData], name: &str, value: FieldData) {
    let index = RenderFields::index_of(name).unwrap_or_else(|| panic!("未知渲染字段 {name}"));
    values[index] = value;
}

/// 把夹具里的 3 个纹理写进渲染字段（注册表/采样输入）。
fn scene_fields(fixture: &Fixture) -> RenderFields {
    let data = model_312_tex_data();
    let mut values = empty_fields();
    set_field(
        &mut values,
        "tex_width",
        FieldData::Index(fixture.textures.iter().map(|t| t.width as i32).collect()),
    );
    set_field(
        &mut values,
        "tex_height",
        FieldData::Index(fixture.textures.iter().map(|t| t.height as i32).collect()),
    );
    set_field(
        &mut values,
        "tex_nchannel",
        FieldData::Index(
            fixture
                .textures
                .iter()
                .map(|t| i32::from(t.nchannel))
                .collect(),
        ),
    );
    set_field(
        &mut values,
        "tex_adr",
        FieldData::Size(fixture.textures.iter().map(|t| t.adr as i64).collect()),
    );
    set_field(&mut values, "tex_data", FieldData::Flag(data));
    RenderFields::new(values)
}

#[test]
fn fixture_metadata_and_cross_checks_are_clean() {
    let fixture = load();
    assert_eq!(fixture.task_id, "2026-10-24-render-bvh-s4");
    assert_eq!(fixture.textures.len(), fixture.registry.ntex);
    assert_eq!(fixture.registry.registry_off, 0, "关闭纹理时注册表为空");
    assert_eq!(fixture.registry.registry_on, fixture.registry.ntex);
    assert!(!fixture.registry.use_textures_off);
    assert!(fixture.registry.use_textures_on);
    assert!(!fixture.rejections.zero_size.constructible);
    assert_eq!(fixture.rejections.zero_size.exception, "ValueError");
    assert!(fixture.rejections.zero_size.message.contains("positive"));
    assert!(fixture.cross_check_312.tex_metadata_equal);
    assert!(fixture.cross_check_312.tex_data_equal);
    assert_eq!(fixture.cross_check_312.tex_data_bytes, 89088);
    assert!(fixture.determinism.bitwise_identical);
    assert!(fixture.hardware_vs_software.max_abs_diff <= 1.0e-6);
    assert_eq!(model_312_tex_data().len(), 89088);
}

#[test]
fn conversion_matches_frozen_reference_for_every_texture() {
    let fixture = load();
    let data = model_312_tex_data();
    for meta in &fixture.textures {
        let byte_count = meta.width as usize * meta.height as usize * usize::from(meta.nchannel);
        let slice = &data[meta.adr as usize..meta.adr as usize + byte_count];
        assert_eq!(
            digest_bytes(slice),
            meta.data_sha256,
            "纹理 {} 输入",
            meta.tex_id
        );
        let texture = TextureData::from_raw(
            meta.width,
            meta.height,
            meta.nchannel,
            meta.adr as usize,
            &data,
        )
        .unwrap_or_else(|error| panic!("纹理 {} 转换失败: {error}", meta.tex_id));
        assert_eq!(
            (texture.width(), texture.height(), texture.channels()),
            (meta.width, meta.height, meta.nchannel)
        );
        assert_eq!(texture.texel_count(), meta.converted_texels);
        assert_eq!(
            digest_f32(texture.texels()),
            meta.converted_sha256,
            "纹理 {} 转换位型",
            meta.tex_id
        );
    }
}

#[test]
fn conversion_pads_missing_channels_like_reference() {
    let fixture = load();
    assert_eq!(fixture.synthetic.len(), 3);
    for case in &fixture.synthetic {
        let data: Vec<u8> = case.data_bits.iter().map(|value| *value as u8).collect();
        let expected = points4(&case.converted_bits);
        let texture = TextureData::from_raw(case.width, case.height, case.nchannel, 0, &data)
            .unwrap_or_else(|error| panic!("{} 转换失败: {error}", case.name));
        assert_eq!(texture.texels().len(), expected.len() * 4);
        for (index, texel) in expected.iter().enumerate() {
            let actual = &texture.texels()[4 * index..4 * index + 4];
            for channel in 0..4 {
                assert_eq!(
                    actual[channel].to_bits(),
                    texel[channel].to_bits(),
                    "{} texel {index} 通道 {channel}",
                    case.name
                );
            }
            if case.nchannel < 2 {
                assert_eq!(actual[1], 0.0, "{} 补 g=0", case.name);
            }
            if case.nchannel < 3 {
                assert_eq!(actual[2], 0.0, "{} 补 b=0", case.name);
            }
            if case.nchannel < 4 {
                assert_eq!(actual[3], 1.0, "{} 补 a=1", case.name);
            }
        }
    }
}

#[test]
fn registry_from_fields_respects_use_textures_and_dimensions() {
    let fixture = load();
    let fields = scene_fields(&fixture);
    let off = TextureRegistry::from_fields(&fields, false).expect("关闭纹理合法");
    assert!(off.is_empty());
    let on = TextureRegistry::from_fields(&fields, true).expect("开启纹理合法");
    assert_eq!(on.len(), fixture.registry.ntex);
    let mut texels = 0usize;
    for (id, meta) in fixture.textures.iter().enumerate() {
        let texture = on.texture(id).expect("逐纹理注册顺序");
        assert_eq!(
            (texture.width(), texture.height(), texture.channels()),
            (meta.width, meta.height, meta.nchannel)
        );
        texels += texture.texels().len();
    }
    assert_eq!(on.texels(), texels);
    assert_eq!(on.iter().count(), fixture.registry.ntex);
}

#[test]
fn registry_rejects_invalid_textures_from_public_fields() {
    let mut values = empty_fields();
    set_field(&mut values, "tex_width", FieldData::Index(vec![0]));
    set_field(&mut values, "tex_height", FieldData::Index(vec![8]));
    set_field(&mut values, "tex_nchannel", FieldData::Index(vec![3]));
    set_field(&mut values, "tex_adr", FieldData::Size(vec![0]));
    set_field(&mut values, "tex_data", FieldData::Flag(vec![0u8; 8 * 3]));
    assert_eq!(
        TextureRegistry::from_fields(&RenderFields::new(values), true).unwrap_err(),
        TextureError::ZeroSize {
            texture: 0,
            axis: "width"
        }
    );
    let empty = TextureRegistry::from_fields(&RenderFields::new(empty_fields()), true)
        .expect("ntex=0 合法");
    assert!(empty.is_empty());
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::render::TextureObjects;
    use mjwarp_rs::runtime::TransferSession;

    fn session() -> TransferSession {
        TransferSession::new(0).expect("S4 GPU 验收需要 NVIDIA 驱动与 cuda-probe")
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn texture_objects_match_frozen_linear_samples() {
        let fixture = load();
        let fields = scene_fields(&fixture);
        let registry = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
        let session = session();
        let objects = TextureObjects::new(&session, &registry).expect("纹理对象与采样内核");
        assert_eq!(objects.len(), fixture.registry.ntex);
        let uvs = points2(&fixture.samples.uv_bits);
        assert_eq!(uvs.len(), 1040);
        for (index, meta) in fixture.textures.iter().enumerate() {
            let reference = points4(&fixture.samples.gpu[index].rgba_bits);
            assert_eq!(reference.len(), uvs.len());
            let samples = objects
                .sample(meta.tex_id as usize, &uvs)
                .expect("纹理采样");
            assert_eq!(samples.len(), reference.len());
            let mut max_diff = 0.0f64;
            let mut unequal = 0usize;
            for (position, (actual, expected)) in samples.iter().zip(reference.iter()).enumerate() {
                for channel in 0..4 {
                    if actual[channel].to_bits() != expected[channel].to_bits() {
                        unequal += 1;
                    }
                    let diff = (f64::from(actual[channel]) - f64::from(expected[channel])).abs();
                    max_diff = max_diff.max(diff);
                    assert!(
                        diff <= 1.0e-6,
                        "纹理 {} 查询 {position} 通道 {channel}: {actual:?} vs {expected:?}（差 {diff:.3e}）",
                        meta.tex_id
                    );
                }
            }
            println!(
                "texture {}: queries={} max_abs_diff={max_diff:.3e} unequal_channels={unequal}",
                meta.tex_id,
                reference.len()
            );
        }
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn texture_objects_follow_use_textures_toggle_and_lifecycle() {
        let fixture = load();
        let fields = scene_fields(&fixture);
        let session = session();
        let off = TextureRegistry::from_fields(&fields, false).expect("关闭纹理合法");
        assert!(off.is_empty());
        let objects = TextureObjects::new(&session, &off).expect("空注册表也应有采样器");
        assert_eq!(objects.len(), 0);
        assert!(objects.sample(0, &[[0.0f32, 0.0f32]]).is_err());
        for round in 0..2 {
            let registry = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
            let objects = TextureObjects::new(&session, &registry).expect("创建纹理对象");
            assert_eq!(objects.len(), fixture.registry.ntex);
            let samples = objects
                .sample(1, &[[0.25f32, 0.75f32], [0.5f32, 0.5f32]])
                .expect("采样");
            assert_eq!(samples.len(), 2);
            drop(objects);
            println!("texture lifecycle round {round}: create/sample/drop ok");
        }
    }
}
