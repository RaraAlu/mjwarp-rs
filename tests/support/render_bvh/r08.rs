//! R08-S2 共享辅助：选择矩阵 context 参数、`OutputValues` → 整缓冲 → 逐相机视图、
//! S7b 冻结参考对账、射线/位姿、参考对账计数与证据哈希。
//! 全部路径不重渲染参考：参考取 S7b 同配方冻结夹具。
#![allow(dead_code)]

use mjwarp_rs::model::render::{RenderFields, RenderObject};
use mjwarp_rs::render::camera::CameraActive;
use mjwarp_rs::render::camera_output::{CameraOutputViews, WholeOutputData};
use mjwarp_rs::render::context::OutputLayout;
use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{DepthSegReference, MAX_DEPTH_ULP, diff_depth, diff_seg};
use mjwarp_rs::render::{
    CameraModel, ContextParams, ImageOutput, OutputValues, RayBuffer, RayBuilder, RayMode,
    RenderContext, ResolvedContext, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use sha2::{Digest, Sha256};

use super::s7b2::{self, RgbReference, diff_rgb};

/// 三通道整缓冲（`OutputResources::read` 的 `OutputValues` 解成切片前的持有者）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutputBuffers {
    pub rgb: Option<Vec<u32>>,
    pub depth: Option<Vec<f32>>,
    pub seg: Option<Vec<[i32; 2]>>,
}

impl OutputBuffers {
    /// 逐通道 `OutputValues` → 整缓冲（同通道后到覆盖先到，用于构造/对账）。
    pub fn from_values(values: &[OutputValues]) -> Self {
        let mut buffers = Self::default();
        for value in values {
            match value {
                OutputValues::Rgb(channel) => buffers.rgb = Some(channel.clone()),
                OutputValues::Depth(channel) => buffers.depth = Some(channel.clone()),
                OutputValues::Seg(channel) => buffers.seg = Some(channel.clone()),
            }
        }
        buffers
    }

    /// 读回 context 中**已启用**的通道（禁用通道保持 `None`，对应上游未分配）。
    pub fn read_enabled(context: &RenderContext) -> Self {
        let mut buffers = Self::default();
        for output in [
            ImageOutput::Rgb,
            ImageOutput::Depth,
            ImageOutput::Segmentation,
        ] {
            if !context.outputs().is_enabled(output) {
                continue;
            }
            match context.outputs().read(output).expect("输出读回") {
                OutputValues::Rgb(channel) => buffers.rgb = Some(channel),
                OutputValues::Depth(channel) => buffers.depth = Some(channel),
                OutputValues::Seg(channel) => buffers.seg = Some(channel),
            }
        }
        buffers
    }

    pub fn whole(&self) -> WholeOutputData<'_> {
        WholeOutputData {
            rgb: self.rgb.as_deref(),
            depth: self.depth.as_deref(),
            segmentation: self.seg.as_deref(),
        }
    }

    /// 在该 context 解析结果上建立逐相机视图（单 world）；失败即测试失败。
    pub fn views<'a>(&'a self, resolved: &'a ResolvedContext) -> CameraOutputViews<'a> {
        CameraOutputViews::new(
            &resolved.plan,
            &resolved.output,
            resolved.nworld,
            self.whole(),
        )
        .expect("逐相机视图")
    }
}

/// S8/S7b 默认参数（`enabled_geom_groups=[0,1,2,3]`，三通道全开）+ 指定 `cam_active`。
pub fn render_params<'a>(active: CameraActive<'a>) -> ContextParams<'a> {
    s7b2::default_params().with_active(active)
}

/// 默认参数 + `cam_active` + 逐相机输出开关（`RenderFlagSpec::List`）。
pub fn targets_params<'a>(
    active: CameraActive<'a>,
    rgb: Option<RenderFlagSpec<'a>>,
    depth: Option<RenderFlagSpec<'a>>,
    seg: Option<RenderFlagSpec<'a>>,
) -> ContextParams<'a> {
    s7b2::default_params()
        .with_active(active)
        .with_render_targets(rgb, depth, seg)
}

/// 宿主解析（选择矩阵/异构通道）：失败即测试失败。
pub fn resolved(fields: &RenderFields, params: &ContextParams<'_>) -> ResolvedContext {
    resolve_context(fields, params).expect("选择矩阵 context 解析")
}

/// 相机名（UTF-8；`cam_active` 的 Names 形态与 id 映射用）。
pub fn camera_names(fields: &RenderFields) -> Vec<String> {
    let model = CameraModel::from_fields(fields).expect("相机模型");
    (0..model.ncam())
        .map(|camera| {
            String::from_utf8(
                fields
                    .name_bytes(RenderObject::Camera, camera as i64)
                    .expect("相机名")
                    .to_vec(),
            )
            .expect("UTF-8 相机名")
        })
        .collect()
}

/// 全部通道开启时，按激活顺序的 rgb/depth/seg adr（三通道一致）。
pub fn expected_adr(model: &CameraModel, order: &[usize]) -> [Vec<i32>; 3] {
    let mut adr = [Vec::new(), Vec::new(), Vec::new()];
    let mut pixels = 0i32;
    for camera in order {
        let resolution = model.camera(*camera).expect("相机").resolution;
        for channel in &mut adr {
            channel.push(pixels);
        }
        pixels += (resolution[0] * resolution[1]) as i32;
    }
    adr
}

/// 按 `order`（模型相机 id 顺序）拼接参考整缓冲；长度须与布局一致（视图构造会校验）。
pub fn reference_buffers(
    order: &[usize],
    rgb_ref: &RgbReference,
    recipe: &DepthSegReference,
) -> OutputBuffers {
    let mut buffers = OutputBuffers::default();
    let (mut rgb, mut depth, mut seg) = (Vec::new(), Vec::new(), Vec::new());
    for camera in order {
        rgb.extend_from_slice(&rgb_ref.cameras[*camera].packed);
        depth.extend(
            recipe.cameras[*camera]
                .depth_bits
                .iter()
                .map(|bits| f32::from_bits(*bits)),
        );
        seg.extend_from_slice(&recipe.cameras[*camera].seg_pairs);
    }
    if !rgb.is_empty() {
        buffers.rgb = Some(rgb);
    }
    if !depth.is_empty() {
        buffers.depth = Some(depth);
    }
    if !seg.is_empty() {
        buffers.seg = Some(seg);
    }
    buffers
}

/// 按布局的**启用通道**拼接参考整缓冲（异构通道用）：只拼 adr >= 0 的相机区段。
pub fn reference_buffers_for_layout(
    order: &[usize],
    layout: &OutputLayout,
    rgb_ref: &RgbReference,
    recipe: &DepthSegReference,
) -> OutputBuffers {
    let mut buffers = OutputBuffers::default();
    let (mut rgb, mut depth, mut seg) = (Vec::new(), Vec::new(), Vec::new());
    for (position, camera) in order.iter().enumerate() {
        if layout.rgb_adr[position] >= 0 {
            rgb.extend_from_slice(&rgb_ref.cameras[*camera].packed);
        }
        if layout.depth_adr[position] >= 0 {
            depth.extend(
                recipe.cameras[*camera]
                    .depth_bits
                    .iter()
                    .map(|bits| f32::from_bits(*bits)),
            );
        }
        if layout.seg_adr[position] >= 0 {
            seg.extend_from_slice(&recipe.cameras[*camera].seg_pairs);
        }
    }
    if layout.rgb_pixels > 0 {
        buffers.rgb = Some(rgb);
    }
    if layout.depth_pixels > 0 {
        buffers.depth = Some(depth);
    }
    if layout.seg_pixels > 0 {
        buffers.seg = Some(seg);
    }
    buffers
}

/// 预计算射线（按该 context 的实际 plan 生成，支持子集/乱序选择）。
pub fn rays_for_plan(
    session: &TransferSession,
    fields: &RenderFields,
    resolved: &ResolvedContext,
) -> RayBuffer {
    let model = CameraModel::from_fields(fields).expect("相机模型");
    RayBuilder::new(session)
        .expect("build_rays 编译")
        .build(&model, &resolved.plan, RayMode::Precomputed)
        .expect("预计算射线")
}

/// 相机 data 位姿：参考 `camera_data` 按**模型相机 id** 索引（S7b1 同源，支持子集/乱序）。
pub fn poses_for_plan(
    recipe: &DepthSegReference,
    model: &CameraModel,
    resolved: &ResolvedContext,
) -> Vec<CameraPose> {
    let mut poses = vec![
        CameraPose {
            xpos: [0.0; 3],
            xmat: [0.0; 9]
        };
        model.ncam()
    ];
    for camera in &resolved.plan.cameras {
        let data = &recipe.camera_data[*camera];
        poses[*camera] = CameraPose {
            xpos: data.origin_bits.map(f32::from_bits),
            xmat: data.xmat_bits.map(f32::from_bits),
        };
    }
    poses
}

/// 深度 f32 位型（逐位比较用）。
pub fn depth_bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// 三通道缓冲 SHA-256（rgb u32 LE / depth 位型 LE / seg i32 LE），窗口两次运行对账用。
pub fn sha256_buffers(buffers: &OutputBuffers) -> String {
    let mut hasher = Sha256::new();
    if let Some(rgb) = &buffers.rgb {
        for value in rgb {
            hasher.update(value.to_le_bytes());
        }
    }
    if let Some(depth) = &buffers.depth {
        for value in depth {
            hasher.update(value.to_bits().to_le_bytes());
        }
    }
    if let Some(seg) = &buffers.seg {
        for pair in seg {
            for value in pair {
                hasher.update(value.to_le_bytes());
            }
        }
    }
    format!("{:x}", hasher.finalize())
}

/// 逐相机参考对账计数（按**激活顺序位置**取视图，按模型相机 id 取参考）。
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewStats {
    pub cameras: usize,
    pub rgb_pixels: usize,
    pub rgb_beyond_1_lsb: usize,
    pub depth_pixels: usize,
    pub depth_mismatch: usize,
    pub max_depth_ulp: i64,
    pub seg_pairs: usize,
    pub seg_mismatch: usize,
}

/// RGB 对同配方参考 ≤1 LSB、depth ≤4 ULP、seg 全等；失败处给出逐相机定位。
pub fn assert_views_match_reference(
    views: &CameraOutputViews<'_>,
    rgb_ref: &RgbReference,
    recipe: &DepthSegReference,
) -> ViewStats {
    let mut stats = ViewStats {
        cameras: views.camera_count(),
        ..ViewStats::default()
    };
    for position in 0..views.camera_count() {
        let camera = views.camera_id(position).expect("相机 id") as usize;
        let rgb = diff_rgb(
            views.rgb_packed(position).expect("rgb 区段"),
            &rgb_ref.cameras[camera],
        );
        let depth = diff_depth(
            views.depth_raw(position).expect("depth 区段"),
            &recipe.cameras[camera],
        )
        .expect("depth diff");
        let seg = diff_seg(
            views.segmentation_raw(position).expect("seg 区段"),
            &recipe.cameras[camera],
        )
        .expect("seg diff");
        println!(
            "r08[views] pos{position} cam{camera}: rgb_pixels={} beyond_1_lsb={} depth_pixels={} \
             depth_mismatch={} max_ulp={} seg_pairs={} seg_mismatch={}",
            rgb.compared,
            rgb.beyond_one_lsb_pixels,
            depth.compared,
            depth.mismatched,
            depth.max_ulp,
            seg.compared,
            seg.mismatched
        );
        assert_eq!(
            rgb.beyond_one_lsb_pixels, 0,
            "cam{camera} RGB 超 1 LSB（first={:?}）",
            rgb.first
        );
        assert_eq!(
            depth.mismatched, 0,
            "cam{camera} depth 必须与同配方参考位等（first={:?}）",
            depth.first
        );
        assert!(
            depth.max_ulp <= MAX_DEPTH_ULP,
            "cam{camera} depth ULP 超限 {}",
            depth.max_ulp
        );
        assert_eq!(
            seg.mismatched, 0,
            "cam{camera} seg 必须与参考全等（first={:?}）",
            seg.first
        );
        stats.rgb_pixels += rgb.compared;
        stats.rgb_beyond_1_lsb += rgb.beyond_one_lsb_pixels;
        stats.depth_pixels += depth.compared;
        stats.depth_mismatch += depth.mismatched;
        stats.max_depth_ulp = stats.max_depth_ulp.max(depth.max_ulp);
        stats.seg_pairs += seg.compared;
        stats.seg_mismatch += seg.mismatched;
    }
    stats
}

/// 逐相机原始三通道（视图已按激活顺序切片）：rgb packed / depth f32 / seg 对。
pub type RawCamera = (Vec<u32>, Vec<f32>, Vec<[i32; 2]>);

/// 按模型相机 id 收集视图原始值（隔离/重排结论按模型相机对齐）。
pub fn raw_by_model_camera(views: &CameraOutputViews<'_>, cameras: &[usize]) -> Vec<RawCamera> {
    let mut out: Vec<Option<RawCamera>> = vec![None; cameras.len().max(1)];
    for position in 0..views.camera_count() {
        let camera = views.camera_id(position).expect("相机 id") as usize;
        out[camera] = Some((
            views.rgb_packed(position).expect("rgb").to_vec(),
            views.depth_raw(position).expect("depth").to_vec(),
            views.segmentation_raw(position).expect("seg").to_vec(),
        ));
    }
    cameras
        .iter()
        .map(|camera| out[*camera].take().unwrap_or_default())
        .collect()
}
