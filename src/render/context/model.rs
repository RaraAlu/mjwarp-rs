//! 从 S1 渲染字段派生的模型侧解析值（渲染/BVH S6a，纯宿主）。
//!
//! 这些派生量与冻结上游 `render_util.create_render_context` 使用的 MjModel 字段一一对应：
//! `np.isin(geom_group, enabled_geom_groups)`、灯光 `light_type/light_attenuation`、
//! `vis.headlight`、天空盒纹理定位、`znear/zfar = vis.map.* * stat.extent`。
//! 全部字段缺失/类型/长度错误都在这里映射成 [`ContextError`]，供 `context::resolve_context`
//! 在分配前返回。

use crate::model::render::{FieldData, RenderFields};

use super::{ContextError, Headlight, Skybox};

/// 上游 `mjtTexture.mjTEXTURE_SKYBOX`。
pub(super) const TEXTURE_SKYBOX: i32 = 2;

pub(super) struct SquashedGeometry {
    pub(super) ids: Vec<i32>,
    pub(super) ray_types: Vec<i32>,
}

/// `np.isin(geom_group, enabled_geom_groups)`：启用 geom id 与出现的 raw geom 类型。
pub(super) fn squash_geometry(
    fields: &RenderFields,
    groups: &[i32],
) -> Result<SquashedGeometry, ContextError> {
    let geom_group = ints(fields, "geom_group")?;
    let geom_type = ints(fields, "geom_type")?;
    if geom_group.len() != geom_type.len() {
        return Err(ContextError::LengthMismatch {
            field: "geom_type",
            expected: geom_group.len(),
            actual: geom_type.len(),
        });
    }
    let mut ids = Vec::new();
    let mut ray_types = Vec::new();
    for (id, group) in geom_group.iter().enumerate() {
        if groups.contains(group) {
            ids.push(id as i32);
            let kind = geom_type[id];
            if !ray_types.contains(&kind) {
                ray_types.push(kind);
            }
        }
    }
    ray_types.sort_unstable();
    Ok(SquashedGeometry { ids, ray_types })
}

pub(super) struct Lights {
    pub(super) attenuation_is_default: bool,
    pub(super) has_spot_lights: bool,
}

/// `light_attenuation_is_default` / `has_spot_lights`（上游 `np.allclose` 口径）。
pub(super) fn lights(fields: &RenderFields) -> Result<Lights, ContextError> {
    let light_type = ints(fields, "light_type")?;
    let attenuation = floats(fields, "light_attenuation")?;
    let expected = light_type
        .len()
        .checked_mul(3)
        .ok_or(ContextError::CountOverflow {
            field: "light_attenuation",
        })?;
    if attenuation.len() != expected {
        return Err(ContextError::LengthMismatch {
            field: "light_attenuation",
            expected,
            actual: attenuation.len(),
        });
    }
    let mut attenuation_is_default = true;
    let (rows, _) = attenuation.as_chunks::<3>();
    for row in rows {
        for (channel, value) in row.iter().enumerate() {
            let target = if channel == 0 { 1.0f32 } else { 0.0f32 };
            if !np_allclose(*value, target) {
                attenuation_is_default = false;
            }
        }
    }
    Ok(Lights {
        attenuation_is_default,
        has_spot_lights: light_type.contains(&super::LIGHT_SPOT),
    })
}

/// `np.allclose(value, target)` 默认 `rtol=1e-5, atol=1e-8`。
fn np_allclose(value: f32, target: f32) -> bool {
    let difference = f64::from(value) - f64::from(target);
    difference.abs() <= 1e-8 + 1e-5 * f64::from(target).abs()
}

/// `vis.headlight`：active + ambient/diffuse/specular 三组 f32。
pub(super) fn headlight(fields: &RenderFields) -> Result<Headlight, ContextError> {
    let active = ints(fields, "vis.headlight.active")?;
    if active.len() != 1 {
        return Err(ContextError::LengthMismatch {
            field: "vis.headlight.active",
            expected: 1,
            actual: active.len(),
        });
    }
    Ok(Headlight {
        active: active[0] != 0,
        ambient: vector3(fields, "vis.headlight.ambient")?,
        diffuse: vector3(fields, "vis.headlight.diffuse")?,
        specular: vector3(fields, "vis.headlight.specular")?,
    })
}

/// 天空盒定位：`render_skybox=true` 且模型含 `mjTEXTURE_SKYBOX` 纹理时返回引用；
/// 上游在没有天空盒纹理时静默把 `render_skybox` 降为 false。
pub(super) fn skybox(
    fields: &RenderFields,
    requested: bool,
) -> Result<Option<Skybox>, ContextError> {
    if !requested {
        return Ok(None);
    }
    let texture_type = ints(fields, "tex_type")?;
    let width = ints(fields, "tex_width")?;
    if texture_type.len() != width.len() {
        return Err(ContextError::LengthMismatch {
            field: "tex_width",
            expected: texture_type.len(),
            actual: width.len(),
        });
    }
    let Some(texture_id) = texture_type.iter().position(|kind| *kind == TEXTURE_SKYBOX) else {
        return Ok(None);
    };
    let face_width =
        u32::try_from(width[texture_id]).map_err(|_| ContextError::LengthMismatch {
            field: "tex_width",
            expected: 1,
            actual: 0,
        })?;
    Ok(Some(Skybox {
        texture_id: texture_id as i32,
        face_width,
    }))
}

/// `float(vis.map.znear * stat.extent)`、`float(vis.map.zfar * stat.extent)`：
/// 上游 MjModel 里 `vis.map.*` 是 f32、`stat.extent` 是 f64，乘积按 f64 计算。
pub(super) fn near_far(fields: &RenderFields) -> Result<(f64, f64), ContextError> {
    let znear = single(fields, "vis.map.znear")?;
    let zfar = single(fields, "vis.map.zfar")?;
    let extent = nums(fields, "stat.extent")?;
    if extent.len() != 1 {
        return Err(ContextError::LengthMismatch {
            field: "stat.extent",
            expected: 1,
            actual: extent.len(),
        });
    }
    Ok((f64::from(znear) * extent[0], f64::from(zfar) * extent[0]))
}

fn single(fields: &RenderFields, name: &'static str) -> Result<f32, ContextError> {
    let values = floats(fields, name)?;
    if values.len() != 1 {
        return Err(ContextError::LengthMismatch {
            field: name,
            expected: 1,
            actual: values.len(),
        });
    }
    Ok(values[0])
}

pub(super) fn ints<'a>(
    fields: &'a RenderFields,
    name: &'static str,
) -> Result<&'a [i32], ContextError> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => Ok(values),
        Some(_) => Err(ContextError::FieldKind { field: name }),
        None => Err(ContextError::MissingField { field: name }),
    }
}

pub(super) fn floats<'a>(
    fields: &'a RenderFields,
    name: &'static str,
) -> Result<&'a [f32], ContextError> {
    match fields.field(name) {
        Some(FieldData::Float(values)) => Ok(values),
        Some(_) => Err(ContextError::FieldKind { field: name }),
        None => Err(ContextError::MissingField { field: name }),
    }
}

fn nums<'a>(fields: &'a RenderFields, name: &'static str) -> Result<&'a [f64], ContextError> {
    match fields.field(name) {
        Some(FieldData::Num(values)) => Ok(values),
        Some(_) => Err(ContextError::FieldKind { field: name }),
        None => Err(ContextError::MissingField { field: name }),
    }
}

fn vector3(fields: &RenderFields, name: &'static str) -> Result<[f32; 3], ContextError> {
    let values = floats(fields, name)?;
    if values.len() != 3 {
        return Err(ContextError::LengthMismatch {
            field: name,
            expected: 3,
            actual: values.len(),
        });
    }
    Ok([values[0], values[1], values[2]])
}
