//! R05 前置：从既有体积通道构建 hfield 只读输入（方案 A，薄适配器）。
//!
//! 背景（2026-10-28 裁决 A 延伸）：渲染侧 `RENDER_FIELDS`（116 项）没有 hfield 通道，
//! 且 `src/model/render.rs` 是 500 行红线文件；而模型侧已存在两条**已验收**通道：
//! `VolumeModelInput`（`src/model/volume_input.rs`，G1/G10 链路）与 `CollisionInputs`
//! （`src/model/collision_inputs.rs`）。本适配器复用 `VolumeModelInput`，**零 native 改动、
//! 零 `RENDER_FIELDS` 扩展、零红线改动、零 ABI 变更**（方案 B 的代价见证据 README）。
//!
//! 语义（与 [`super::hfield`] 的 `HfieldInputs` 对齐）：
//! - `nhfield == 0` → `Ok(None)`（无 hfield，合法，不新增上游未拒绝输入）；
//! - 否则 `size` 取 `hfield_size_f32()`（**唯一 f64→f32 转换点**，`model::volume::num_to_f32`，
//!   G10 先例，不另写转换）；`nrow`/`ncol`/`adr` 取 `ints("hfield_*")`；`data` 取
//!   `hfield_data()`（**不复制**）；
//! - 单份：不读 `nworld`、不按 world 复制；长度/范围校验由 `VolumeModelInput::new` 在构造时
//!   完成，本适配器错误直传 `NativeProbeError`。
//!
//! 生命周期说明：`HfieldInputs::size` 是 `&[f32]`，而体积通道的 `hfield_size` 是 f64、
//! 转换结果是 owned `Vec<f32>`；因此适配器返回持有该转换结果的 [`VolumeHfieldInputs`]，
//! 再用 [`VolumeHfieldInputs::as_inputs`] 得到可传给 `resolve_hfield` 的 `HfieldInputs`。
//! 这样不改 S1 已验收的 `HfieldInputs` API（避免为 `size` 引入 `Cow` 破坏已验收接口）。

use crate::diagnostics::NativeProbeError;
use crate::model::volume_input::VolumeModelInput;

use super::hfield::{HfieldError, HfieldInputs, HfieldParams, resolve_hfield};

/// 从 [`VolumeModelInput`] 适配出的 hfield 输入（持有 `size` 的 f64→f32 转换结果，其余数组借用）。
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeHfieldInputs<'a> {
    size: Vec<f32>,
    nrow: &'a [i32],
    ncol: &'a [i32],
    adr: &'a [i32],
    data: &'a [f32],
}

impl<'a> VolumeHfieldInputs<'a> {
    /// 从已验证的 [`VolumeModelInput`] 构建 hfield 输入。
    ///
    /// `nhfield == 0` 返回 `Ok(None)`（无 hfield 合法）；否则读取五字段并返回适配结果。
    /// 长度/范围错误由 `VolumeModelInput::new` 在构造期排除，本函数错误为字段访问错误直传。
    pub fn from_volume(model: &'a VolumeModelInput) -> Result<Option<Self>, NativeProbeError> {
        if model.counts().nhfield == 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            size: model.hfield_size_f32()?,
            nrow: model.ints("hfield_nrow")?,
            ncol: model.ints("hfield_ncol")?,
            adr: model.ints("hfield_adr")?,
            data: model.hfield_data()?,
        }))
    }

    /// 组装成 [`resolve_hfield`] 可接受的借用视图（五字段全 `Some`）。
    pub fn as_inputs(&self) -> HfieldInputs<'_> {
        HfieldInputs {
            size: Some(&self.size),
            nrow: Some(self.nrow),
            ncol: Some(self.ncol),
            adr: Some(self.adr),
            data: Some(self.data),
        }
    }

    /// 一步解析：`resolve_hfield(&self.as_inputs())`。
    pub fn resolve(&self) -> Result<Option<HfieldParams<'_>>, HfieldError> {
        resolve_hfield(&self.as_inputs())
    }
}
