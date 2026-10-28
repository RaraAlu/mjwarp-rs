//! R02-S1 验收（离线，无 GPU）：天空盒竖排 2D 纹理注册表、方向→face/sc,tc 映射与 flag 位。
//!
//! 参考口径（pinned `mujoco_warp 3.15.0`）：`render_util.py` L514-535、`render.py`
//! L494-556 / L1259-1268 / L1017-1031。`skybox_uv` 是**测试参考实现**（逐字对照上游公式），
//! 只钉 face 号/符号/clamp 语义，**不是产品路径**；设备实现见
//! `render_shade.cu::sample_skybox`，其数值对齐由后续窗口位级比对兜底（本片无 GPU 执行）。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::context::{Skybox, texture_registry};
use mjwarp_rs::render::texture::TextureError;
use mjwarp_rs::runtime::launcher::{MEGAKERNEL_SLOTS, RenderArgKind};

const MEGAKERNEL_CU: &str = include_str!("../src/render/kernel/cuda/render_megakernel.cu");

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

/// scene-a 风格三纹理：0/1 普通 2D，2 为天空盒竖排 64×384。
fn scene_fields() -> RenderFields {
    let widths = [64i32, 32, 64];
    let heights = [64i32, 32, 384];
    let channels = [3i32, 3, 3];
    let mut adr = Vec::new();
    let mut total = 0usize;
    for (width, height) in widths.iter().zip(heights.iter()) {
        adr.push(total as i64);
        total += (width * height * 3) as usize;
    }
    let mut values = empty_fields();
    set_field(&mut values, "tex_width", FieldData::Index(widths.to_vec()));
    set_field(
        &mut values,
        "tex_height",
        FieldData::Index(heights.to_vec()),
    );
    set_field(
        &mut values,
        "tex_nchannel",
        FieldData::Index(channels.to_vec()),
    );
    set_field(&mut values, "tex_adr", FieldData::Size(adr));
    set_field(&mut values, "tex_data", FieldData::Flag(vec![9u8; total]));
    RenderFields::new(values)
}

fn skybox_of(texture_id: i32, face_width: u32) -> Skybox {
    Skybox {
        texture_id,
        face_width,
    }
}

#[test]
fn skybox_only_registry_materializes_skybox_and_dummy_textures() {
    let fields = scene_fields();
    let registry =
        texture_registry(&fields, false, Some(&skybox_of(2, 64))).expect("天空盒子集注册表合法");
    assert_eq!(
        registry.len(),
        3,
        "注册表长度必须等于 ntex（下标 == 纹理 id）"
    );
    let skybox = registry.texture(2).expect("天空盒纹理");
    assert_eq!(
        (skybox.width(), skybox.height(), skybox.channels()),
        (64, 384, 3),
        "天空盒必须是竖排 2D（tex_height = 6*tex_width）"
    );
    // 非天空盒 id 为 1×1 全零占位（上游 empty_tex：`Texture2D(1, 1, 4, float32)`）。
    for id in [0usize, 1] {
        let dummy = registry.texture(id).expect("占位纹理");
        assert_eq!((dummy.width(), dummy.height()), (1, 1));
        assert_eq!(dummy.texels(), &[0.0f32; 4], "id={id} 占位纹理必须全零");
    }
    assert_eq!(registry.texels(), 64 * 384 * 4 + 2 * 4);
}

#[test]
fn use_textures_materializes_all_ids_unchanged() {
    let fields = scene_fields();
    let on = texture_registry(&fields, true, None).expect("use_textures 全量注册表");
    assert_eq!(on.len(), 3);
    assert_eq!(on.texture(2).expect("id2").height(), 384);
    // `use_textures=true` 时是否请求天空盒不改变注册表（上游 L527 两条件都实体化全部）。
    let with_skybox =
        texture_registry(&fields, true, Some(&skybox_of(2, 64))).expect("带天空盒全量注册表");
    assert_eq!(with_skybox.iter().count(), 3);
    for id in 0..3 {
        assert_eq!(
            on.texture(id).unwrap().texels(),
            with_skybox.texture(id).unwrap().texels(),
            "id={id} 全量实体化结果必须与是否请求天空盒无关"
        );
    }
}

#[test]
fn no_skybox_and_no_textures_yields_empty_registry() {
    assert!(
        texture_registry(&scene_fields(), false, None)
            .expect("空注册表合法")
            .is_empty()
    );
}

#[test]
fn skybox_face_width_must_match_tex_width() {
    // face_width 应等于 `tex_width[skybox_id]`（上游 render_util.py L519）。
    assert_eq!(
        texture_registry(&scene_fields(), false, Some(&skybox_of(2, 32))).unwrap_err(),
        TextureError::LengthMismatch {
            field: "skybox_face_width",
            expected: 32,
            actual: 64,
        }
    );
}

#[test]
fn skybox_texture_id_must_be_in_range() {
    assert_eq!(
        texture_registry(&scene_fields(), false, Some(&skybox_of(5, 64))).unwrap_err(),
        TextureError::LengthMismatch {
            field: "skybox_texture_id",
            expected: 3,
            actual: 5,
        }
    );
}

/// 内核 `MK_FLAG_*` 位表：既有 bit0-8 定义不得被本片改动，bit9 追加且不与既有位冲突；
/// `MEGAKERNEL_SLOTS` 81 槽种类与顺序零改动（1 bounds + 73 数组 + 7 标量）。
#[test]
fn kernel_flag_bits_and_slot_signature_unchanged() {
    for define in [
        "#define MK_FLAG_TEXTURES 0x1u",
        "#define MK_FLAG_VERTEX_NORMALS 0x2u",
        "#define MK_FLAG_SPECULAR 0x4u",
        "#define MK_FLAG_EMISSION 0x8u",
        "#define MK_FLAG_PER_LIGHT_AMBIENT 0x10u",
        "#define MK_FLAG_AMBIENT_LIGHTING 0x20u",
        "#define MK_FLAG_HEADLIGHT 0x40u",
        "#define MK_FLAG_CULL_BACKFACES 0x80u",
        "#define MK_FLAG_AA 0x100u",
        "#define MK_FLAG_SKYBOX 0x200u",
    ] {
        assert!(MEGAKERNEL_CU.contains(define), "缺位定义: {define}");
    }
    assert_eq!(MEGAKERNEL_SLOTS.len(), 81);
    let scalars = MEGAKERNEL_SLOTS
        .iter()
        .filter(|kind| matches!(kind, RenderArgKind::Scalar(_)))
        .count();
    let bounds = MEGAKERNEL_SLOTS
        .iter()
        .filter(|kind| matches!(kind, RenderArgKind::Bounds2D))
        .count();
    assert_eq!((bounds, scalars), (1, 7));
}

/// 测试参考：上游 `sample_skybox`（render.py L503-554）的方向→(face, s, v)。
fn skybox_uv(ray_dir_world: [f32; 3], face_width: u32) -> (u8, f32, f32) {
    let rx = ray_dir_world[0];
    let ry = ray_dir_world[2];
    let rz = -ray_dir_world[1];
    let (arx, ary, arz) = (rx.abs(), ry.abs(), rz.abs());
    let (face, sc, tc, ma) = if arx >= ary && arx >= arz {
        if rx > 0.0 {
            (0u8, -rz, -ry, arx)
        } else {
            (1u8, rz, -ry, arx)
        }
    } else if ary >= arz {
        if ry > 0.0 {
            (2u8, rx, rz, ary)
        } else {
            (3u8, rx, -rz, ary)
        }
    } else if rz > 0.0 {
        (4u8, rx, -ry, arz)
    } else {
        (5u8, -rx, -ry, arz)
    };
    // 上游 `math.safe_div(x, y) = x / (y != 0 ? y : MJ_MINVAL)`（math.py L318-319）。
    let safe_div = |x: f32, y: f32| x / if y != 0.0 { y } else { 1.0e-15 };
    let s = (safe_div(sc, ma) + 1.0) * 0.5;
    let t_min = 0.5 * (1.0f32 / face_width as f32);
    let t = ((safe_div(tc, ma) + 1.0) * 0.5).clamp(t_min, 1.0 - t_min);
    (face, s, (f32::from(face) + t) * (1.0f32 / 6.0f32))
}

/// 六轴 → 六面。世界方向先按 GL3 约定旋转（`rx=x, ry=z, rz=-y`），故世界 +Z→face2、
/// -Z→face3、-Y→face4、+Y→face5；中心 uv 均为 0.5。
#[test]
fn axis_directions_map_to_the_six_faces() {
    let expected = [
        ([1.0f32, 0.0, 0.0], 0u8),
        ([-1.0, 0.0, 0.0], 1),
        ([0.0, 0.0, 1.0], 2),
        ([0.0, 0.0, -1.0], 3),
        ([0.0, -1.0, 0.0], 4),
        ([0.0, 1.0, 0.0], 5),
    ];
    for (dir, face) in expected {
        let (actual_face, s, v) = skybox_uv(dir, 64);
        assert_eq!(actual_face, face, "方向 {dir:?}");
        assert_eq!(s.to_bits(), 0.5f32.to_bits(), "face 中心 s");
        let wanted_v = (f32::from(face) + 0.5) * (1.0f32 / 6.0f32);
        assert_eq!(v.to_bits(), wanted_v.to_bits(), "face 中心 v");
    }
}

/// 1/6 常量必须与 warp 编译期常量逐位一致（`0x3E2AAAAB`）。
#[test]
fn sixth_constant_matches_warp_static_value() {
    assert_eq!((1.0f32 / 6.0f32).to_bits(), 0x3E2A_AAAB);
}

/// face 边界：`arx == ary` 时取 +X 侧（上游 `>=` 判定）；t 在两端被 clamp 到半 texel。
#[test]
fn face_boundary_and_t_clamp() {
    // (1,0,1)：rx=1、ry=1 → arx==ary 取 +X；sc=-rz=0、tc=-ry=-1 → t=0 → clamp t_min。
    let (face, s, v) = skybox_uv([1.0, 0.0, 1.0], 64);
    assert_eq!((face, s.to_bits()), (0, 0.5f32.to_bits()));
    let t_min = 0.5f32 / 64.0f32;
    let wanted_v = t_min * (1.0f32 / 6.0f32);
    assert_eq!(v.to_bits(), wanted_v.to_bits(), "t 必须夹到 t_min");
    // (1,0,-1)：tc=-ry=1 → t=1 → clamp 1-t_min（边界像素不得跨面串色）。
    let (face, _s, v) = skybox_uv([1.0, 0.0, -1.0], 64);
    assert_eq!(face, 0);
    let wanted_v = (1.0f32 - t_min) * (1.0f32 / 6.0f32);
    assert_eq!(v.to_bits(), wanted_v.to_bits(), "t 必须夹到 1-t_min");
    // face_width=1 时 clamp 区间退化为 0.5；face_width=1024 时夹到 1-1/2048。
    assert!((skybox_uv([1.0, 0.0, -1.0], 1).2 * 6.0 - 0.5).abs() < 1.0e-6);
    let wide = skybox_uv([1.0, 0.0, -1.0], 1024).2 * 6.0;
    assert!((wide - (1.0 - 0.5 / 1024.0)).abs() < 1.0e-6);
}

/// 零方向向量：`ma == 0` 时 `safe_div` 退化为 `x/1e-15`，不得产生 NaN。
#[test]
fn zero_direction_is_not_nan() {
    let (_face, s, v) = skybox_uv([0.0, 0.0, 0.0], 64);
    assert!(!s.is_nan() && !v.is_nan(), "零方向不得产生 NaN");
    assert_eq!(s.to_bits(), 0.5f32.to_bits());
}
