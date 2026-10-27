//! R09-S1 光照与阴影静态参数（纯宿主；渲染/BVH R09）。
//!
//! 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d`）：
//! - L830 `_make_compute_lighting`：per-light 光照计算入口；
//! - L850 `use_shadows` 开关、L866 `lightcastshadow`（单灯是否投影）、
//!   L880 `shadow_light_fraction`（被遮挡像素的可见度系数，签名默认 0.3）；
//! - L919-961 阴影分支：仅当 `use_shadows and lightcastshadow` 才发射阴影射线，
//!   命中时 `visible = shadow_light_fraction`。因此「`use_shadows=true` 但没有任何
//!   `castshadow` 灯」是**合法配置**（只是不产生阴影），本模块不新增该拒绝；
//! - L281 `MK_FLAG_PER_LIGHT_AMBIENT`：`enable_per_light_ambient` 的档位开关。
//!
//! 语义边界（R10-S2 结论延续）：灯位姿/静态数组按**单份**上传（内核按单 world 索引灯），
//! 本模块既不声明也不复制 `[nworld, nlight]`；逐 world 灯数据缺口见 backlog
//! `2026-10-28-render-per-world-geom-light-gap`。
//!
//! 只做校验与宿主结构，不分配设备资源、不发射内核。

use crate::model::render::{FieldData, RenderFields};

use super::super::ContextError;
use super::super::model::{floats, ints, lights};
use super::ContextParams;

/// 单灯静态参数（长度均为 `nlight` 或 `3 × nlight`，不随 `nworld` 复制）。
#[derive(Clone, Debug, PartialEq)]
pub struct LightParams {
    /// 灯类型（`mjLIGHT_SPOT=0` / `mjLIGHT_DIRECTIONAL=1` / ...）。
    pub light_type: Vec<i32>,
    /// 是否投影（`light_castshadow != 0`）。
    pub cast_shadow: Vec<bool>,
    /// 衰减系数 `(常数, 线性, 二次)`，3 f32/灯。
    pub attenuation: Vec<[f32; 3]>,
    /// 聚光截止角（上游 `d.light_cutoff` 为度，计算时转弧度）。
    pub cutoff: Vec<f32>,
    /// 衰减指数。
    pub exponent: Vec<f32>,
    /// 环境光 RGB，3 f32/灯。
    pub ambient: Vec<[f32; 3]>,
    /// 漫反射 RGB，3 f32/灯。
    pub diffuse: Vec<[f32; 3]>,
    /// 高光 RGB，3 f32/灯。
    pub specular: Vec<[f32; 3]>,
}

impl LightParams {
    /// 灯数。
    pub fn len(&self) -> usize {
        self.light_type.len()
    }

    /// 是否无灯。
    pub fn is_empty(&self) -> bool {
        self.light_type.is_empty()
    }
}

/// 校验后的光照/阴影静态参数（宿主只读视图）。
#[derive(Clone, Debug, PartialEq)]
pub struct LightingParams {
    /// 灯数（`light_type` 长度）。
    pub nlight: usize,
    /// `use_shadows`：总开关；为 true 但没有任何 castshadow 灯时仍合法。
    pub use_shadows: bool,
    /// `shadow_light_fraction`：遮挡像素可见度系数（上游默认 0.3；上游不做范围校验，
    /// 本模块不新增拒绝，原值透传）。
    pub shadow_light_fraction: f64,
    /// `enable_per_light_ambient`：按灯 ambient 档位开关（上游 `MK_FLAG_PER_LIGHT_AMBIENT`）。
    pub enable_per_light_ambient: bool,
    /// `use_ambient_lighting`：全局环境光开关。
    pub use_ambient_lighting: bool,
    /// 任一灯 `light_castshadow != 0`（决定阴影射线是否有目标灯）。
    pub any_cast_shadow: bool,
    /// `light_attenuation` 全为默认 `(1,0,0)`（上游 `np.allclose` 口径）。
    pub attenuation_is_default: bool,
    /// 模型含聚光灯（`light_type == mjLIGHT_SPOT`）。
    pub has_spot_lights: bool,
    /// 逐灯静态参数（单份，不随 world 复制）。
    pub lights: LightParams,
}

impl LightingParams {
    /// 阴影实际生效的灯数（`use_shadows` 且该灯 `cast_shadow`）。
    ///
    /// `use_shadows=false` 或没有 castshadow 灯时返回 0，均属合法配置。
    pub fn shadow_light_count(&self) -> usize {
        match self.use_shadows {
            true => self.lights.cast_shadow.iter().filter(|flag| **flag).count(),
            false => 0,
        }
    }
}

/// 校验 per-light 字段并解析光照/阴影静态参数。
///
/// 校验矩阵（字段 → 期望类型/长度；全部在读取前可诊断，不 panic）：
/// - `light_type`：`Index`，长度 `nlight`（其余字段长度基准）；
/// - `light_castshadow`：`Flag`，长度 `nlight`；
/// - `light_cutoff`、`light_exponent`：`Float`，长度 `nlight`；
/// - `light_attenuation`、`light_ambient`、`light_diffuse`、`light_specular`：
///   `Float` 或 `Index`/`Num` 之外的 `Float`，长度 `3 × nlight`。
///
/// 不新增上游未拒绝的输入：`use_shadows=true` 而无 castshadow 灯合法；
/// `shadow_light_fraction` 原值透传（上游无范围校验）。
pub fn resolve_lighting(
    fields: &RenderFields,
    params: &ContextParams,
) -> Result<LightingParams, ContextError> {
    let light_type = ints(fields, "light_type")?;
    let nlight = light_type.len();
    let cast_shadow = flags(fields, "light_castshadow")?;
    expect_len("light_castshadow", nlight, cast_shadow.len())?;
    let cutoff = floats(fields, "light_cutoff")?;
    expect_len("light_cutoff", nlight, cutoff.len())?;
    let exponent = floats(fields, "light_exponent")?;
    expect_len("light_exponent", nlight, exponent.len())?;
    let attenuation = triplets(fields, "light_attenuation", nlight)?;
    let ambient = triplets(fields, "light_ambient", nlight)?;
    let diffuse = triplets(fields, "light_diffuse", nlight)?;
    let specular = triplets(fields, "light_specular", nlight)?;
    // 上游 `light_attenuation_is_default` / `has_spot_lights`（np.allclose 口径）复用
    // S6a 已有派生，避免两套口径。
    let derived = lights(fields)?;
    let any_cast_shadow = cast_shadow.iter().any(|flag| *flag);
    Ok(LightingParams {
        nlight,
        use_shadows: params.use_shadows,
        shadow_light_fraction: params.shadow_light_fraction,
        enable_per_light_ambient: params.enable_per_light_ambient,
        use_ambient_lighting: params.use_ambient_lighting,
        any_cast_shadow,
        attenuation_is_default: derived.attenuation_is_default,
        has_spot_lights: derived.has_spot_lights,
        lights: LightParams {
            light_type: light_type.to_vec(),
            cast_shadow,
            attenuation,
            cutoff: cutoff.to_vec(),
            exponent: exponent.to_vec(),
            ambient,
            diffuse,
            specular,
        },
    })
}

/// 读取 `nlight × 3` 的浮点字段并按灯分组成 `[f32; 3]`。
fn triplets(
    fields: &RenderFields,
    name: &'static str,
    nlight: usize,
) -> Result<Vec<[f32; 3]>, ContextError> {
    let width = nlight
        .checked_mul(3)
        .ok_or(ContextError::CountOverflow { field: name })?;
    let values = floats(fields, name)?;
    expect_len(name, width, values.len())?;
    let mut rows = Vec::with_capacity(nlight);
    let (chunks, _) = values.as_chunks::<3>();
    for row in chunks {
        rows.push(*row);
    }
    Ok(rows)
}

/// 读取 `Flag`（`light_castshadow` 在 S1 字段里是 u8/uchar）并按 `!= 0` 转布尔。
fn flags(fields: &RenderFields, name: &'static str) -> Result<Vec<bool>, ContextError> {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => Ok(values.iter().map(|value| *value != 0).collect()),
        Some(_) => Err(ContextError::FieldKind { field: name }),
        None => Err(ContextError::MissingField { field: name }),
    }
}

/// 长度校验；不一致时给可诊断错误。
fn expect_len(name: &'static str, expected: usize, actual: usize) -> Result<(), ContextError> {
    if expected != actual {
        return Err(ContextError::LengthMismatch {
            field: name,
            expected,
            actual,
        });
    }
    Ok(())
}
