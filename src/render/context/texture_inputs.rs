//! R02-S1：天空盒纹理注册表（翻译上游 `render_util.py` L514-535 的 `textures_registry`）。
//!
//! 上游只在调用方需要时才实体化 GPU 纹理：
//!
//! ```python
//! if use_textures or has_skybox:
//!     empty_tex = None if use_textures else wp.Texture2D(width=1, height=1, num_channels=4, dtype=float32)
//!     for i in range(mjm.ntex):
//!         if use_textures or i == skybox_id:
//!             textures_registry.append(create_warp_texture(mjm, i))
//!         else:
//!             textures_registry.append(empty_tex)
//! ```
//!
//! 即：`use_textures=true` → 全部实体化（现状不变）；`use_textures=false` 且有天空盒 →
//! **只有天空盒那个 id 实体化，其余 id 用 1×1 空纹理占位**；两者都不需要 → 空注册表。
//! 占位纹理保证「注册表下标 == 模型纹理 id」，设备侧 `textures.data[tex_id]` 语义不变。
//!
//! 天空盒本身是竖排 2D（MuJoCo `tex_height = 6 * tex_width`，face 顺序 +X,-X,+Y,-Y,+Z,-Z；
//! 见 `render.py` L500-502 与 `render_util.py` L514-520），因此本模块**不需要 cube 纹理
//! 路径**：实体化后的纹理由既有 `TextureObjects` 按普通 2D 建立（`texture/cuda.rs`），
//! 设备侧由 `render_shade.cu::sample_skybox` 用 `v=(face+t)/6` 取竖排。

use super::Skybox;
use crate::model::render::{FieldData, RenderFields};
use crate::render::texture::{TextureData, TextureError, TextureRegistry};

/// 1×1 占位纹理的源字节：全零，与上游 `wp.Texture2D(1,1,4,float32)` 的全零内容一致。
const DUMMY_TEXEL: [u8; 4] = [0; 4];

/// 按上游 `render_util.py` L527-535 语义构造纹理注册表。
///
/// * `use_textures=true`：全部纹理实体化（与 [`TextureRegistry::from_fields`] 相同）。
/// * `use_textures=false` 且 `skybox=Some`：仅该天空盒纹理实体化，其余 id 为 1×1 全零占位。
/// * 其余：空注册表（天空盒与纹理都不需要）。
pub fn texture_registry(
    fields: &RenderFields,
    use_textures: bool,
    skybox: Option<&Skybox>,
) -> Result<TextureRegistry, TextureError> {
    if use_textures {
        return TextureRegistry::from_fields(fields, true);
    }
    let Some(skybox) = skybox else {
        return Ok(TextureRegistry::default());
    };
    let widths = int_field(fields, "tex_width")?;
    let heights = int_field(fields, "tex_height")?;
    let channels = int_field(fields, "tex_nchannel")?;
    let addresses = size_field(fields, "tex_adr")?;
    let tex_data = byte_field(fields, "tex_data")?;
    let count = widths.len();
    for (field, actual) in [
        ("tex_height", heights.len()),
        ("tex_nchannel", channels.len()),
        ("tex_adr", addresses.len()),
    ] {
        if actual != count {
            return Err(TextureError::LengthMismatch {
                field,
                expected: count,
                actual,
            });
        }
    }
    let skybox_id =
        usize::try_from(skybox.texture_id).map_err(|_| TextureError::LengthMismatch {
            field: "skybox_texture_id",
            expected: count,
            actual: 0,
        })?;
    if skybox_id >= count {
        return Err(TextureError::LengthMismatch {
            field: "skybox_texture_id",
            expected: count,
            actual: skybox_id,
        });
    }
    // `Skybox::face_width` 必须等于该纹理的 `tex_width`（上游 face 宽度即 `mjm.tex_width`，
    // render_util.py L519）；不一致说明 host 解析与注册表字段不同源。
    let face_width = u32::try_from(widths[skybox_id])
        .map_err(|_| TextureError::Overflow { field: "tex_width" })?;
    if face_width != skybox.face_width {
        return Err(TextureError::LengthMismatch {
            field: "skybox_face_width",
            expected: skybox.face_width as usize,
            actual: face_width as usize,
        });
    }
    let mut textures = Vec::with_capacity(count);
    for texture in 0..count {
        if texture == skybox_id {
            let width = u32::try_from(widths[texture])
                .map_err(|_| TextureError::Overflow { field: "tex_width" })?;
            let height = u32::try_from(heights[texture]).map_err(|_| TextureError::Overflow {
                field: "tex_height",
            })?;
            let channel_count =
                u8::try_from(channels[texture]).map_err(|_| TextureError::InvalidChannels {
                    texture,
                    channels: usize::try_from(channels[texture]).unwrap_or(usize::MAX),
                })?;
            let address = usize::try_from(addresses[texture])
                .map_err(|_| TextureError::NegativeAddress { texture })?;
            textures.push(TextureData::from_raw_at(
                texture,
                width,
                height,
                channel_count,
                address,
                tex_data,
            )?);
        } else {
            textures.push(TextureData::from_raw_at(texture, 1, 1, 4, 0, &DUMMY_TEXEL)?);
        }
    }
    Ok(TextureRegistry::from_textures(textures))
}

fn int_field<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [i32], TextureError> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => Ok(values),
        Some(_) => Err(TextureError::FieldKind { field: name }),
        None => Err(TextureError::MissingField { field: name }),
    }
}

fn size_field<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [i64], TextureError> {
    match fields.field(name) {
        Some(FieldData::Size(values)) => Ok(values),
        Some(_) => Err(TextureError::FieldKind { field: name }),
        None => Err(TextureError::MissingField { field: name }),
    }
}

fn byte_field<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [u8], TextureError> {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => Ok(values),
        Some(_) => Err(TextureError::FieldKind { field: name }),
        None => Err(TextureError::MissingField { field: name }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neither_textures_nor_skybox_yields_empty_registry() {
        let fields = RenderFields::new(Vec::new());
        assert!(
            texture_registry(&fields, false, None)
                .expect("空注册表合法")
                .is_empty()
        );
    }
}
