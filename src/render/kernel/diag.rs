//! P10：`MJWARP_DIAG_PIXEL_DUMP` 的可选产品侧中间量采集（纯诊断，零行为）。
//!
//! 门控：env `MJWARP_DIAG_PIXEL_DUMP` 非空时启用；未设/空时**零分配、零回读、零输出**，
//! 默认渲染路径传 `diag_addr = 0`，内核所有诊断 store 被首行 `nullptr` 判断跳过。
//!
//! 启用时每次 `render_impl` 分配一块设备缓冲区（`launch_worlds * total_rays * STRIDE` 个 f32），
//! 内核按 `(worldid * total_rays + rayid)` 写固定 stride 记录；launch 后回读并按 world 保存，
//! 测试侧 dump 通过 [`MegakernelRenderer::diag_record`] 取单条记录并解码。
//!
//! 布局与 `src/render/kernel/cuda/render_diag.cu` 的 `DIAG_*` 常量逐项对应（schema 见证据 README）。

use std::sync::Mutex;

use crate::runtime::launcher::LauncherError;
use crate::runtime::{TransferBuffer, TransferSession};

/// dump 门控环境变量（与测试侧 `tests/support/render_bvh/light/diag.rs` 同源）。
pub const ENV: &str = "MJWARP_DIAG_PIXEL_DUMP";
/// 每个 ray 的诊断记录长度（f32 槽位数）。
pub const STRIDE: usize = 64;
/// 每灯记录长度/槽位数。
pub const LIGHT_STRIDE: usize = 8;
pub const LIGHT_SLOTS: usize = 4;

const COLOR: usize = 0;
const BASE_COLOR: usize = 3;
const TEX_UV: usize = 6;
const TEX_COLOR: usize = 8;
const BARY_U: usize = 11;
const BARY_V: usize = 12;
const T: usize = 13;
const FACE: usize = 14;
const MESH_ID: usize = 15;
const HIT_POINT: usize = 16;
const NORMAL: usize = 19;
const RAY_DIR: usize = 22;
const LIGHT_BASE: usize = 25;

/// 单灯诊断量（f32 原值；`visible` 为 0/1 或阴影 fraction）。
#[derive(Clone, Copy, Debug, Default)]
pub struct DiagLight {
    pub weight: f32,
    pub visible: f32,
    pub ndotl: f32,
    pub ndoth: f32,
    pub half: [f32; 3],
    pub spec_pow: f32,
}

/// 单 ray 诊断记录（f32 未量化值；`face`/`mesh_id` 为 i32 位型解码）。
#[derive(Clone, Copy, Debug, Default)]
pub struct DiagRecord {
    pub color: [f32; 3],
    pub base_color: [f32; 3],
    pub tex_uv: [f32; 2],
    pub tex_color: [f32; 3],
    pub bary_u: f32,
    pub bary_v: f32,
    pub t: f32,
    pub face: i32,
    pub mesh_id: i32,
    pub hit_point: [f32; 3],
    pub normal: [f32; 3],
    /// 内核落盘的是 `ray_dir_world`；`view_dir = -ray_dir_world`（精确负号位翻转）在解码时派生。
    pub ray_dir: [f32; 3],
    pub view_dir: [f32; 3],
    pub lights: [DiagLight; LIGHT_SLOTS],
}

impl DiagRecord {
    /// 解码固定 stride 记录；长度不符返回 `None`（诊断失败不 panic、不影响默认路径）。
    pub fn from_slice(record: &[f32]) -> Option<Self> {
        if record.len() != STRIDE {
            return None;
        }
        let mut lights = [DiagLight::default(); LIGHT_SLOTS];
        for (slot, light) in lights.iter_mut().enumerate() {
            let base = LIGHT_BASE + slot * LIGHT_STRIDE;
            *light = DiagLight {
                weight: record[base],
                visible: record[base + 1],
                ndotl: record[base + 2],
                ndoth: record[base + 3],
                half: [record[base + 4], record[base + 5], record[base + 6]],
                spec_pow: record[base + 7],
            };
        }
        Some(Self {
            color: [record[COLOR], record[COLOR + 1], record[COLOR + 2]],
            base_color: [
                record[BASE_COLOR],
                record[BASE_COLOR + 1],
                record[BASE_COLOR + 2],
            ],
            tex_uv: [record[TEX_UV], record[TEX_UV + 1]],
            tex_color: [
                record[TEX_COLOR],
                record[TEX_COLOR + 1],
                record[TEX_COLOR + 2],
            ],
            bary_u: record[BARY_U],
            bary_v: record[BARY_V],
            t: record[T],
            face: record[FACE].to_bits() as i32,
            mesh_id: record[MESH_ID].to_bits() as i32,
            hit_point: [
                record[HIT_POINT],
                record[HIT_POINT + 1],
                record[HIT_POINT + 2],
            ],
            normal: [record[NORMAL], record[NORMAL + 1], record[NORMAL + 2]],
            ray_dir: [record[RAY_DIR], record[RAY_DIR + 1], record[RAY_DIR + 2]],
            view_dir: [-record[RAY_DIR], -record[RAY_DIR + 1], -record[RAY_DIR + 2]],
            lights,
        })
    }
}

/// env 门控：未设/空一律 false（默认路径零副作用）。
pub fn enabled() -> bool {
    match std::env::var(ENV) {
        Ok(value) => !value.trim().is_empty(),
        Err(_) => false,
    }
}

pub(super) struct DiagCapture {
    world: Option<usize>,
    rays: usize,
    values: Vec<f32>,
}

/// `MegakernelRenderer` 持有的已回读诊断记录（按 world 覆盖，仅 env 开启时非空）。
pub(super) type DiagState = Mutex<Vec<DiagCapture>>;

pub(super) struct DiagBuffer {
    buffer: TransferBuffer<f32>,
    rays: usize,
}

impl DiagBuffer {
    /// env 关闭时返回 `None`（零分配）；开启时分配并清零。
    pub(super) fn new(
        session: &TransferSession,
        rays: usize,
    ) -> Result<Option<Self>, LauncherError> {
        if !enabled() {
            return Ok(None);
        }
        let len = rays
            .checked_mul(STRIDE)
            .ok_or(LauncherError::Overflow("diag_len"))?;
        let buffer = session
            .upload(&vec![0.0f32; len])
            .map_err(|error| LauncherError::Driver(format!("diag-upload: {error}")))?;
        Ok(Some(Self { buffer, rays }))
    }

    /// 设备地址（内核尾随槽 `diag_ptr`）；0 由调用方在 `None` 时直接给出。
    pub(super) fn addr(&self) -> Result<u64, LauncherError> {
        self.buffer
            .launcher_span()
            .map(|span| span.addr())
            .map_err(|error| LauncherError::Driver(format!("diag-span: {error}")))
    }

    /// launch 后回读整块并按 world 覆盖保存（诊断失败只打印、不影响渲染结果）。
    pub(super) fn finish(
        self,
        world: Option<usize>,
        state: &DiagState,
    ) -> Result<(), LauncherError> {
        let mut values = vec![0.0f32; self.buffer.len()];
        self.buffer
            .read_range_into(0, &mut values)
            .map_err(|error| LauncherError::Driver(format!("diag-read: {error}")))?;
        let mut captures = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        captures.retain(|capture| capture.world != world);
        captures.push(DiagCapture {
            world,
            rays: self.rays,
            values,
        });
        Ok(())
    }
}

/// 尾随槽地址：`None`（env 关闭）时为 0，内核据此跳过全部诊断 store。
pub(super) fn arg_addr(buffer: &Option<DiagBuffer>) -> Result<u64, LauncherError> {
    match buffer {
        Some(buffer) => buffer.addr(),
        None => Ok(0),
    }
}

impl super::MegakernelRenderer {
    /// launch 后回读（无诊断缓冲时为空操作）。
    pub(super) fn finish_diag(&self, buffer: Option<DiagBuffer>, world: Option<usize>) {
        if let Some(buffer) = buffer
            && let Err(error) = buffer.finish(world, &self.diag)
        {
            eprintln!("diag capture failed: {error}");
        }
    }

    /// 取某 world 某 ray 的诊断记录（仅诊断轮；未开启/越界/未命中返回 `None`）。
    pub fn diag_record(&self, world: usize, ray: usize) -> Option<DiagRecord> {
        let captures = self
            .diag
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let capture = captures
            .iter()
            .rev()
            .find(|capture| capture.world == Some(world))?;
        if ray >= capture.rays {
            return None;
        }
        let start = ray.checked_mul(STRIDE)?;
        DiagRecord::from_slice(capture.values.get(start..start + STRIDE)?)
    }
}
