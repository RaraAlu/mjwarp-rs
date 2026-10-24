//! MuJoCo 纹理转换与纹理注册表（渲染/BVH S4）。
//!
//! 语义翻译冻结上游 `render_util.create_warp_texture`（MJWarp 3.15，
//! `mujoco_warp/_src/render_util.py`）：把 `tex_data` 的 u8 纹素按
//! `(y * width + x) * nchannel` 取样，补齐为 RGBA 四通道（缺失通道 g/b=0、a=255），
//! 每通道乘 `f32(1.0 / 255.0)`。上游用 `wp.static(1.0 / 255.0)`，即 double 值 1/255
//! 先舍入成 f32 常量 `0x3B80_8081`，再与 `float(r)` 做一次 f32 乘法；本模块保持相同的
//! 常量与乘法顺序，转换结果与参考逐 texel 位等。
//!
//! 采样语义（LINEAR 过滤、CLAMP 寻址、归一化坐标、单 mip）由 `texture/cuda` 的
//! CUDA 纹理对象 + `tex.2d` 取指实现，与上游 `wp.Texture2D(..., LINEAR)` 的
//! `texture.h` 设备路径一致；本文件只做宿主转换/注册表/校验，不启动设备。
//!
//! 范围：`use_textures` 开关只影响是否实际建立纹理对象；图像级 patch、天空盒着色、
//! 3D 纹理与 mipmap 不在本片（天空盒/图像级延后 S7b/S8）。

use crate::model::render::{FieldData, RenderFields};

#[cfg(feature = "cuda-probe")]
pub mod cuda;
#[cfg(feature = "cuda-probe")]
pub use cuda::TextureObjects;

/// 上游把 u8 归一化到 [0,1] 的 f32 常量：`static_cast<float>(1.0 / 255.0)`。
pub const INV_255: f32 = f32::from_bits(0x3B80_8081);
/// MuJoCo 纹理通道数上限（`tex_nchannel`）。
pub const MAX_TEXTURE_CHANNELS: u8 = 4;

/// 纹理转换/注册表错误；全部在设备分配前给出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureError {
    /// 缺少 S1 渲染字段。
    MissingField { field: &'static str },
    /// 字段标量类型不符。
    FieldKind { field: &'static str },
    /// 同一纹理属性的数组长度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// `tex_adr` 为负。
    NegativeAddress { texture: usize },
    /// 宽或高为 0（零尺寸纹理拒绝）。
    ZeroSize { texture: usize, axis: &'static str },
    /// `tex_nchannel` 不在 1..=4。
    InvalidChannels { texture: usize, channels: usize },
    /// `tex_adr + width * height * channels` 超出 `tex_data`。
    DataRange {
        texture: usize,
        required: usize,
        available: usize,
    },
    /// 计数/长度溢出。
    Overflow { field: &'static str },
}

impl std::error::Error for TextureError {}

impl std::fmt::Display for TextureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "缺少渲染字段 {field}"),
            Self::FieldKind { field } => write!(formatter, "渲染字段 {field} 类型不符"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(formatter, "{field} 长度 {actual} != {expected}"),
            Self::NegativeAddress { texture } => {
                write!(formatter, "纹理 {texture} 的 tex_adr 为负")
            }
            Self::ZeroSize { texture, axis } => {
                write!(formatter, "纹理 {texture} 的 {axis} 为 0（零尺寸纹理）")
            }
            Self::InvalidChannels { texture, channels } => write!(
                formatter,
                "纹理 {texture} 的 tex_nchannel={channels} 不在 1..={MAX_TEXTURE_CHANNELS}"
            ),
            Self::DataRange {
                texture,
                required,
                available,
            } => write!(
                formatter,
                "纹理 {texture} 需要 {required} 字节 tex_data，只有 {available}"
            ),
            Self::Overflow { field } => write!(formatter, "{field} 计数溢出"),
        }
    }
}

/// 单个纹理的 RGBA f32 纹素（行主序、y 在外、x 在内，每纹素 4 通道）。
#[derive(Clone, Debug, PartialEq)]
pub struct TextureData {
    width: u32,
    height: u32,
    channels: u8,
    texels: Vec<f32>,
}

impl TextureData {
    /// 从 `tex_data[address..]` 的 u8 纹素转换；`channels` 为源通道数（1..=4）。
    pub fn from_raw(
        width: u32,
        height: u32,
        channels: u8,
        address: usize,
        tex_data: &[u8],
    ) -> Result<Self, TextureError> {
        let texture = 0usize;
        Self::from_raw_at(texture, width, height, channels, address, tex_data)
    }

    /// 同 [`TextureData::from_raw`]，错误里带第 `texture` 个纹理的下标。
    pub fn from_raw_at(
        texture: usize,
        width: u32,
        height: u32,
        channels: u8,
        address: usize,
        tex_data: &[u8],
    ) -> Result<Self, TextureError> {
        if width == 0 {
            return Err(TextureError::ZeroSize {
                texture,
                axis: "width",
            });
        }
        if height == 0 {
            return Err(TextureError::ZeroSize {
                texture,
                axis: "height",
            });
        }
        if !(1..=MAX_TEXTURE_CHANNELS).contains(&channels) {
            return Err(TextureError::InvalidChannels {
                texture,
                channels: usize::from(channels),
            });
        }
        let texels =
            (width as usize)
                .checked_mul(height as usize)
                .ok_or(TextureError::Overflow {
                    field: "texture_texels",
                })?;
        let required = texels
            .checked_mul(usize::from(channels))
            .ok_or(TextureError::Overflow {
                field: "texture_bytes",
            })?;
        let available = tex_data.len().saturating_sub(address);
        if tex_data.len() < address || required > available {
            return Err(TextureError::DataRange {
                texture,
                required,
                available,
            });
        }
        let mut rgba = Vec::with_capacity(texels * 4);
        for texel in 0..texels {
            let base = address + texel * usize::from(channels);
            let r = tex_data[base];
            let g = if channels > 1 { tex_data[base + 1] } else { 0 };
            let b = if channels > 2 { tex_data[base + 2] } else { 0 };
            let a = if channels > 3 {
                tex_data[base + 3]
            } else {
                255
            };
            rgba.push(f32::from(r) * INV_255);
            rgba.push(f32::from(g) * INV_255);
            rgba.push(f32::from(b) * INV_255);
            rgba.push(f32::from(a) * INV_255);
        }
        Ok(Self {
            width,
            height,
            channels,
            texels: rgba,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// 源通道数（补齐前的 `tex_nchannel`）。
    pub fn channels(&self) -> u8 {
        self.channels
    }

    /// 补齐后的 RGBA f32 纹素，长度 `width * height * 4`。
    pub fn texels(&self) -> &[f32] {
        &self.texels
    }

    /// texel 数（不含通道展开）。
    pub fn texel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

/// 纹理注册表：按模型纹理 id 顺序保存全部纹理的宿主转换结果。
///
/// 与上游 `RenderContext.textures_registry` 一致：`use_textures=true` 时逐纹理创建；
/// `use_textures=false` 时注册表为空（上游无天空盒请求时 `len == 0`）。天空盒着色
/// 不在本片，`render_skybox` 的占位纹理路径不实现（S7b）。
#[derive(Clone, Debug, Default)]
pub struct TextureRegistry {
    textures: Vec<TextureData>,
}

impl TextureRegistry {
    /// 由已转换纹理构造（空注册表合法）。
    pub fn from_textures(textures: Vec<TextureData>) -> Self {
        Self { textures }
    }

    /// 从 S1 渲染字段转换全部纹理；`use_textures=false` 时直接返回空注册表。
    pub fn from_fields(fields: &RenderFields, use_textures: bool) -> Result<Self, TextureError> {
        if !use_textures {
            return Ok(Self::default());
        }
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
        let mut textures = Vec::with_capacity(count);
        for texture in 0..count {
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
        }
        Ok(Self { textures })
    }

    /// 纹理数（等于模型 `ntex`）。
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// 按模型纹理 id 取纹理。
    pub fn texture(&self, texture: usize) -> Option<&TextureData> {
        self.textures.get(texture)
    }

    /// 逐纹理迭代（顺序等于模型纹理 id）。
    pub fn iter(&self) -> impl Iterator<Item = &TextureData> {
        self.textures.iter()
    }

    /// 全部纹理的 RGBA f32 纹素总数（4 通道展开后）。
    pub fn texels(&self) -> usize {
        self.textures
            .iter()
            .map(|texture| texture.texels().len())
            .sum()
    }
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
    use crate::model::render::{RENDER_FIELDS, RenderFieldKind};

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

    fn set(values: &mut [FieldData], name: &str, value: FieldData) {
        let index = RenderFields::index_of(name).expect("渲染字段存在");
        values[index] = value;
    }

    fn texture_fields(
        width: i32,
        height: i32,
        channels: i32,
        address: i64,
        data_len: usize,
    ) -> RenderFields {
        let mut values = empty_fields();
        set(&mut values, "tex_width", FieldData::Index(vec![width]));
        set(&mut values, "tex_height", FieldData::Index(vec![height]));
        set(
            &mut values,
            "tex_nchannel",
            FieldData::Index(vec![channels]),
        );
        set(&mut values, "tex_adr", FieldData::Size(vec![address]));
        set(
            &mut values,
            "tex_data",
            FieldData::Flag(vec![128u8; data_len]),
        );
        RenderFields::new(values)
    }

    #[test]
    fn converts_channels_and_pads_missing_components() {
        for (channels, expected) in [
            (1u8, [10.0f32 * INV_255, 0.0, 0.0, 1.0]),
            (2, [10.0 * INV_255, 20.0 * INV_255, 0.0, 1.0]),
            (3, [10.0 * INV_255, 20.0 * INV_255, 30.0 * INV_255, 1.0]),
            (
                4,
                [
                    10.0 * INV_255,
                    20.0 * INV_255,
                    30.0 * INV_255,
                    40.0 * INV_255,
                ],
            ),
        ] {
            let data = [10u8, 20, 30, 40];
            let texture = TextureData::from_raw(1, 1, channels, 0, &data).expect("转换");
            assert_eq!(texture.texel_count(), 1);
            for (channel, wanted) in expected.iter().enumerate() {
                assert_eq!(
                    texture.texels()[channel].to_bits(),
                    wanted.to_bits(),
                    "channels={channels} channel {channel}"
                );
            }
        }
        assert_eq!(INV_255, f32::from_bits(0x3B80_8081));
    }

    #[test]
    fn rejects_invalid_texture_inputs_before_allocation() {
        let registry = |fields: &RenderFields| TextureRegistry::from_fields(fields, true);
        assert_eq!(
            registry(&texture_fields(0, 8, 3, 0, 8 * 3)).unwrap_err(),
            TextureError::ZeroSize {
                texture: 0,
                axis: "width"
            }
        );
        assert_eq!(
            registry(&texture_fields(8, 0, 3, 0, 8 * 3)).unwrap_err(),
            TextureError::ZeroSize {
                texture: 0,
                axis: "height"
            }
        );
        for channels in [0, 5] {
            assert_eq!(
                registry(&texture_fields(8, 8, channels, 0, 8 * 8 * 4)).unwrap_err(),
                TextureError::InvalidChannels {
                    texture: 0,
                    channels: channels as usize
                }
            );
        }
        assert_eq!(
            registry(&texture_fields(8, 8, 3, -1, 8 * 8 * 3)).unwrap_err(),
            TextureError::NegativeAddress { texture: 0 }
        );
        assert_eq!(
            registry(&texture_fields(8, 8, 3, 0, 10)).unwrap_err(),
            TextureError::DataRange {
                texture: 0,
                required: 8 * 8 * 3,
                available: 10
            }
        );
        let mut values = empty_fields();
        set(&mut values, "tex_width", FieldData::Index(vec![8, 8]));
        set(&mut values, "tex_height", FieldData::Index(vec![8]));
        set(&mut values, "tex_nchannel", FieldData::Index(vec![3, 3]));
        set(&mut values, "tex_adr", FieldData::Size(vec![0, 192]));
        set(&mut values, "tex_data", FieldData::Flag(vec![0u8; 384]));
        assert_eq!(
            registry(&RenderFields::new(values)).unwrap_err(),
            TextureError::LengthMismatch {
                field: "tex_height",
                expected: 2,
                actual: 1
            }
        );
        let mut values = empty_fields();
        set(&mut values, "tex_width", FieldData::Float(vec![8.0]));
        set(&mut values, "tex_height", FieldData::Index(vec![8]));
        set(&mut values, "tex_nchannel", FieldData::Index(vec![3]));
        set(&mut values, "tex_adr", FieldData::Size(vec![0]));
        set(&mut values, "tex_data", FieldData::Flag(vec![0u8; 192]));
        assert_eq!(
            registry(&RenderFields::new(values)).unwrap_err(),
            TextureError::FieldKind { field: "tex_width" }
        );
    }

    #[test]
    fn use_textures_off_yields_empty_registry() {
        let fields = texture_fields(8, 8, 3, 0, 8 * 8 * 3);
        assert!(
            TextureRegistry::from_fields(&fields, false)
                .unwrap()
                .is_empty()
        );
        assert!(
            TextureRegistry::from_fields(&empty_fields_as_fields(), true)
                .unwrap()
                .is_empty()
        );
    }

    fn empty_fields_as_fields() -> RenderFields {
        RenderFields::new(empty_fields())
    }
}
