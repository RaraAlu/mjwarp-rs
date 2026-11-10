//! P10：`MJWARP_DIAG_PIXEL_DUMP` 的可选产品侧中间量采集（纯诊断，零行为）。
//!
//! 门控：env `MJWARP_DIAG_PIXEL_DUMP` 非空时启用；未设/空时**零分配、零回读、零输出**，
//! 默认渲染路径传 `diag_addr = 0`，内核所有诊断 store 被首行 `nullptr` 判断跳过。
//!
//! 启用时每次 `render_impl` 分配一块设备缓冲区（`launch_worlds * total_rays * STRIDE` 个 f32），
//! 内核按 `rayid * STRIDE` 写固定 stride 记录——`rayid` 是**单次发射内**的射线号，不是全局下标。
//! 因此诊断采集要求 **per-world 单次发射**：此时每个 world 单独回读、按 `diag_record(world, ray)` 取值
//! 即等价；若将来改成多 world 单次发射，各 world 会把记录写到同一 `rayid` 区间并静默互相覆盖
//! （诊断数据失效，不影响渲染数值）——约束与风险见证据 README §2。
//!
//! 布局与 `src/render/kernel/cuda/render_megakernel.cu` 顶部的 `DIAG_*` 常量逐项对应（该文件是
//! `MEGAKERNEL_SOURCE` 7 文件拼接之一；**没有**单独的 `render_diag.cu`；schema 见证据 README）。

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

#[cfg(test)]
mod tests {
    use super::*;

    /// 按内核布局把一条记录编码成 64 个 f32 槽位（仅测试：模拟设备侧 store 的位型落盘）。
    fn encode(record: &DiagRecord) -> [f32; STRIDE] {
        let mut slots = [0.0f32; STRIDE];
        slots[COLOR..COLOR + 3].copy_from_slice(&record.color);
        slots[BASE_COLOR..BASE_COLOR + 3].copy_from_slice(&record.base_color);
        slots[TEX_UV..TEX_UV + 2].copy_from_slice(&record.tex_uv);
        slots[TEX_COLOR..TEX_COLOR + 3].copy_from_slice(&record.tex_color);
        slots[BARY_U] = record.bary_u;
        slots[BARY_V] = record.bary_v;
        slots[T] = record.t;
        slots[FACE] = f32::from_bits(record.face as u32);
        slots[MESH_ID] = f32::from_bits(record.mesh_id as u32);
        slots[HIT_POINT..HIT_POINT + 3].copy_from_slice(&record.hit_point);
        slots[NORMAL..NORMAL + 3].copy_from_slice(&record.normal);
        slots[RAY_DIR..RAY_DIR + 3].copy_from_slice(&record.ray_dir);
        for (slot, light) in record.lights.iter().enumerate() {
            let base = LIGHT_BASE + slot * LIGHT_STRIDE;
            slots[base] = light.weight;
            slots[base + 1] = light.visible;
            slots[base + 2] = light.ndotl;
            slots[base + 3] = light.ndoth;
            slots[base + 4..base + 7].copy_from_slice(&light.half);
            slots[base + 7] = light.spec_pow;
        }
        slots
    }

    fn bits(values: &[f32]) -> Vec<u32> {
        values.iter().map(|value| value.to_bits()).collect()
    }

    /// G4：纯 CPU 合成记录往返（64 f32 编码 → 解码 → 再编码），不依赖 GPU。
    #[test]
    fn synthetic_record_round_trips_bit_exactly() {
        let light0 = DiagLight {
            weight: 0.5,
            visible: 1.0,
            ndotl: 0.25,
            ndoth: 0.125,
            half: [0.1, 0.2, 0.3],
            spec_pow: 8.0,
        };
        let light3 = DiagLight {
            weight: 0.75,
            visible: 0.0,
            ndotl: -0.5,
            ndoth: 0.0,
            half: [-0.1, 0.0, 0.4],
            spec_pow: 32.0,
        };
        let record = DiagRecord {
            color: [0.25, -1.5, 3.5],
            base_color: [0.1, 0.2, 0.3],
            tex_uv: [0.75, 0.125],
            tex_color: [1.0, 0.5, 0.0],
            bary_u: 0.375,
            bary_v: 0.25,
            t: 12.5,
            face: -3,
            mesh_id: 42,
            hit_point: [1.0, -2.0, 3.0],
            normal: [0.0, 0.0, 1.0],
            ray_dir: [-0.0, 0.5, -1.0],
            view_dir: [0.0, 0.0, 0.0],
            lights: [light0, DiagLight::default(), DiagLight::default(), light3],
        };
        let slots = encode(&record);
        let decoded = DiagRecord::from_slice(&slots).expect("64 f32 记录应可解码");
        assert_eq!(bits(&decoded.color), bits(&record.color));
        assert_eq!(bits(&decoded.base_color), bits(&record.base_color));
        assert_eq!(bits(&decoded.tex_uv), bits(&record.tex_uv));
        assert_eq!(bits(&decoded.tex_color), bits(&record.tex_color));
        assert_eq!(
            bits(&[decoded.bary_u, decoded.bary_v]),
            bits(&[record.bary_u, record.bary_v])
        );
        assert_eq!(decoded.t.to_bits(), record.t.to_bits());
        assert_eq!(decoded.face, -3, "face 以 i32 位型解码");
        assert_eq!(decoded.mesh_id, 42, "mesh_id 以 i32 位型解码");
        assert_eq!(bits(&decoded.hit_point), bits(&record.hit_point));
        assert_eq!(bits(&decoded.normal), bits(&record.normal));
        assert_eq!(bits(&decoded.ray_dir), bits(&record.ray_dir));
        // view_dir = -ray_dir 的精确负号位翻转（含 -0.0 边界：-(-0.0) = +0.0）。
        let expected_view: Vec<u32> = record
            .ray_dir
            .iter()
            .map(|value| value.to_bits() ^ 0x8000_0000)
            .collect();
        assert_eq!(bits(&decoded.view_dir), expected_view);
        assert_eq!(decoded.view_dir[0].to_bits(), 0x0000_0000);
        // 灯槽位型往返：第 1 与第 4（headlight）slot。
        for slot in [0usize, LIGHT_SLOTS - 1] {
            let (expected, actual) = (&record.lights[slot], &decoded.lights[slot]);
            assert_eq!(actual.weight.to_bits(), expected.weight.to_bits());
            assert_eq!(actual.visible.to_bits(), expected.visible.to_bits());
            assert_eq!(actual.ndotl.to_bits(), expected.ndotl.to_bits());
            assert_eq!(actual.ndoth.to_bits(), expected.ndoth.to_bits());
            assert_eq!(bits(&actual.half), bits(&expected.half));
            assert_eq!(actual.spec_pow.to_bits(), expected.spec_pow.to_bits());
        }
        // 保留槽 57..63 恒 0；再编码回 64 f32 与原始编码逐位一致。
        assert_eq!(bits(&slots[57..]), vec![0u32; STRIDE - 57]);
        assert_eq!(bits(&encode(&decoded)), bits(&slots));
    }

    /// 长度不符必须拒绝（诊断失败不 panic、不影响默认路径）。
    #[test]
    fn decode_rejects_wrong_length() {
        assert!(DiagRecord::from_slice(&[0.0f32; STRIDE - 1]).is_none());
        assert!(DiagRecord::from_slice(&[0.0f32; STRIDE + 1]).is_none());
        assert!(DiagRecord::from_slice(&[0.0f32; STRIDE]).is_some());
    }
}
