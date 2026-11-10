//! S7b2 渲染执行器的数据打包辅助（渲染/BVH S7b2）：字段读取、灯位姿推导、geam 位姿反解、
//! BVH blob 打包与设备上传。供 `megakernel.rs` 使用；行数口径按新文件拆分（各 ≤500 行）。

use crate::model::render::{FieldData, RenderFields};
use crate::render::bvh::{FlexSceneLeaves, SceneBvhInput, build_scene_bvh};
use crate::render::context::RenderContext;
use crate::render::context::ResolvedContext;
use crate::render::context::owner::{LeafPose, SceneLeaf, SceneState};
use crate::render::kernel::cast::{ModelRenderArrays, SceneDevice};
use crate::runtime::launcher::{
    LauncherError, RenderArg, RenderArrayValue, RenderElem, RenderScalar,
};
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

use crate::render::kernel::cast::{MeshDevice, PoseError};
pub(crate) fn to_i32(value: usize) -> Result<i32, LauncherError> {
    i32::try_from(value).map_err(|_| LauncherError::Overflow("render_count"))
}

/// 从打包后的 scene 布局反解每个 geom 的 data 位姿（leaf→geom）。
pub(crate) fn geom_data_poses(
    scene: &SceneDevice,
    ngeom: usize,
) -> Result<(Vec<f32>, Vec<f32>), LauncherError> {
    let n_nodes = scene.meta.first().copied().unwrap_or(0) as usize;
    let n_leaf = scene.n_leaf;
    let n_world = scene.meta.get(2).copied().unwrap_or(1) as usize;
    let leaf_geom = scene
        .meta
        .get(4 + n_world + 2 * n_nodes..)
        .ok_or(LauncherError::InvalidArray("scene_leaf_geom"))?;
    let leaf_size_base = 6 * n_nodes;
    let leaf_xpos_base = leaf_size_base + 3 * n_leaf;
    let leaf_xmat_base = leaf_xpos_base + 3 * n_leaf;
    let mut xpos = vec![0.0f32; 3 * ngeom];
    let mut xmat = vec![0.0f32; 9 * ngeom];
    for leaf in 0..n_leaf {
        let encoded = *leaf_geom
            .get(leaf)
            .ok_or(LauncherError::InvalidArray("scene_leaf_geom"))?;
        // R06-S3a：flex 叶编码 `-1 - flex_local` 没有 geom 位姿（设备只读 flex 通道），跳过。
        let Ok(geom) = usize::try_from(encoded) else {
            continue;
        };
        if geom >= ngeom {
            return Err(LauncherError::InvalidArray("scene_leaf_geom"));
        }
        let pos = leaf_xpos_base + 3 * leaf;
        let mat = leaf_xmat_base + 9 * leaf;
        xpos[3 * geom..3 * geom + 3].copy_from_slice(&scene.params[pos..pos + 3]);
        xmat[9 * geom..9 * geom + 9].copy_from_slice(&scene.params[mat..mat + 9]);
    }
    Ok((xpos, xmat))
}

/// R06-S3a/S3c：把 flex 叶并入本帧场景 BVH 并打包为 `SceneDevice`。
///
/// 上游每帧 `refit` 语义的等价实现（本实现逐帧重建，节点布局只服务自有打包）；
/// flex 叶编码 `geom_id = -1 - flex_local`，在 `SceneState` 里只承载布局（size/pose 零占位，
/// 几何出自 flex 通道）。
pub(crate) fn pack_flex_scene(
    context: &RenderContext,
    model_arrays: &ModelRenderArrays,
    node: &FlexSceneLeaves,
) -> Result<SceneDevice, LauncherError> {
    let state = context.scene();
    let mut bounds = state
        .bounds()
        .map_err(|error| LauncherError::Driver(format!("flex-scene-bounds: {error}")))?;
    let mut groups: Vec<u32> = state.leaves().iter().map(|leaf| leaf.group).collect();
    let mut geom_ids: Vec<i32> = state.leaves().iter().map(|leaf| leaf.geom_id).collect();
    let mut leaves = state.leaves().to_vec();
    let mut poses = state.poses().to_vec();
    for (local, leaf_bounds) in node.bounds.iter().enumerate() {
        let flex_local = i32::try_from(local).map_err(|_| LauncherError::Overflow("flex_leaf"))?;
        let encoded = -flex_local - 1;
        bounds.push(*leaf_bounds);
        groups.push(node.groups[local]);
        geom_ids.push(encoded);
        leaves.push(SceneLeaf {
            geom_type: 9,
            size: [0.0; 3],
            group: node.groups[local],
            geom_id: encoded,
        });
        poses.push(LeafPose {
            xpos: [0.0; 3],
            xmat: [0.0; 9],
        });
    }
    let combined = SceneState::new(leaves, poses, state.worlds())
        .map_err(|error| LauncherError::Driver(format!("flex-scene-state: {error}")))?;
    let bvh = build_scene_bvh(SceneBvhInput {
        bounds: &bounds,
        groups: &groups,
        geom_ids: &geom_ids,
        worlds: state.worlds(),
    })
    .map_err(|error| LauncherError::Driver(format!("flex-scene-bvh: {error}")))?;
    SceneDevice::pack(&bvh, &combined, model_arrays, context.mesh_count())
}

/// BVH blob：header(10 × u32 字节偏移/长度) + 五段数据（16 字节对齐）。
pub(crate) fn pack_blob(scene: &SceneDevice, mesh: &MeshDevice) -> Vec<u32> {
    let sections: Vec<Vec<u32>> = vec![
        scene.meta.iter().map(|value| *value as u32).collect(),
        scene.params.iter().map(|value| value.to_bits()).collect(),
        mesh.meta.iter().map(|value| *value as u32).collect(),
        mesh.params.iter().map(|value| value.to_bits()).collect(),
        mesh.flat_offsets()
            .iter()
            .map(|value| *value as u32)
            .collect(),
    ];
    let mut header = vec![0u32; 10];
    let mut body: Vec<u32> = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let offset = 10 * 4 + body.len() * 4;
        header[2 * index] = offset as u32;
        header[2 * index + 1] = (section.len() * 4) as u32;
        body.extend_from_slice(section);
        while !body.len().is_multiple_of(4) {
            body.push(0);
        }
    }
    let mut blob = header;
    blob.extend_from_slice(&body);
    blob
}

pub(crate) fn span_addr<T: TransferElement>(
    buffer: &TransferBuffer<T>,
) -> Result<u64, LauncherError> {
    Ok(buffer
        .launcher_span()
        .map_err(|error| LauncherError::Driver(format!("s7b2-span: {error}")))?
        .addr())
}

pub(crate) fn upload<T: TransferElement>(
    session: &TransferSession,
    values: &[T],
) -> Result<TransferBuffer<T>, LauncherError> {
    session
        .upload(values)
        .map_err(|error| LauncherError::Driver(format!("s7b2-upload: {error}")))
}

pub(crate) fn array<T: TransferElement>(
    buffer: &TransferBuffer<T>,
    elem: RenderElem,
    shape: &[usize],
) -> Result<RenderArg, LauncherError> {
    let span = buffer
        .launcher_span()
        .map_err(|error| LauncherError::Driver(format!("s7b2-span: {error}")))?;
    Ok(RenderArg::Array(RenderArrayValue::contiguous(
        span, elem, shape,
    )?))
}

pub(crate) fn bool_words(flags: &[i32]) -> Vec<u32> {
    let mut words = vec![0u32; flags.len().div_ceil(4)];
    for (index, flag) in flags.iter().enumerate() {
        if *flag != 0 {
            words[index / 4] |= 1u32 << (8 * (index % 4));
        }
    }
    words
}

pub(crate) fn ints(fields: &RenderFields, name: &'static str) -> Result<Vec<i32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => Ok(values.to_vec()),
        Some(_) | None => Err(LauncherError::InvalidArray(name)),
    }
}

pub(crate) fn floats(fields: &RenderFields, name: &'static str) -> Result<Vec<f32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Float(values)) => Ok(values.to_vec()),
        Some(_) | None => Err(LauncherError::InvalidArray(name)),
    }
}

/// f64 模型字段（`D`）→ f32（上游 `Model.light_pos` 是 f32）。
pub(crate) fn reals(fields: &RenderFields, name: &'static str) -> Result<Vec<f32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Num(values)) => Ok(values.iter().map(|value| *value as f32).collect()),
        Some(FieldData::Float(values)) => Ok(values.to_vec()),
        Some(_) | None => Err(LauncherError::InvalidArray(name)),
    }
}

/// `u8` 标志字段 → i32（`bool_words` 只关心非零）。
pub(crate) fn flags(fields: &RenderFields, name: &'static str) -> Result<Vec<i32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => Ok(values.iter().map(|value| i32::from(*value)).collect()),
        Some(FieldData::Index(values)) => Ok(values.to_vec()),
        Some(_) | None => Err(LauncherError::InvalidArray(name)),
    }
}

/// 固定灯（`light_mode=0`、body 为 world）的 data 位姿（S7b2）。
///
/// 上游 `smooth.py::_light_local_to_global` 的 else 分支：`light_xpos = body_xpos +
/// rot_vec_quat(pos, xquat)`、`light_xdir = normalize(rot_vec_quat(dir, xquat))`。scene-a 三盏灯
/// 都是 `light_mode=0` 且挂在 world body（`light_bodyid=0`、恒等旋转），因此
/// `rot_vec_quat(v, identity)` 在 f32 下逐位等于 `v`；推导结果 = `f32(model light_pos)` 与
/// `normalize(f32(model light_dir))`（warp normalize 的 `fma(z,z,fma(y,y,x*x))` 序列）。
///
/// 渲染器与夹具 `light_data_check` 共用本函数，确保「推导 = 上游 forward 位型」可判定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightDataPose {
    /// `d.light_xpos`（f32 位型）。
    pub xpos: [f32; 3],
    /// `d.light_xdir`（f32 位型，已归一化）。
    pub xdir: [f32; 3],
}

pub fn light_data_poses(fields: &RenderFields) -> Result<Vec<LightDataPose>, PoseError> {
    let modes = ints(fields, "light_mode").map_err(|_| PoseError::Missing("light_mode"))?;
    let bodyids = ints(fields, "light_bodyid").map_err(|_| PoseError::Missing("light_bodyid"))?;
    if modes.iter().any(|mode| *mode != 0) {
        return Err(PoseError::Kind("light_mode"));
    }
    if bodyids.iter().any(|body| *body != 0) {
        return Err(PoseError::Kind("light_bodyid"));
    }
    let pos = reals(fields, "light_pos").map_err(|_| PoseError::Missing("light_pos"))?;
    let dir = reals(fields, "light_dir").map_err(|_| PoseError::Missing("light_dir"))?;
    let count = modes.len();
    if pos.len() != 3 * count || dir.len() != 3 * count {
        return Err(PoseError::Length {
            field: "light_pos",
            expected: 3 * count,
            actual: pos.len(),
        });
    }
    let mut poses = Vec::with_capacity(count);
    for light in 0..count {
        let x = dir[3 * light];
        let y = dir[3 * light + 1];
        let z = dir[3 * light + 2];
        // 上游 warp `normalize` 的 `fma(z,z,fma(y,y,x*x))` 序列；用目标无关的精确 f32 FMA
        // （`crate::math::fma::fma_f32`）避免宿主 libm 回退在不同工具链下舍入不一致。
        let length = crate::math::fma::fma_f32(z, z, crate::math::fma::fma_f32(y, y, x * x)).sqrt();
        let xdir = if length > 0.0 {
            [x / length, y / length, z / length]
        } else {
            [0.0, 0.0, 0.0]
        };
        poses.push(LightDataPose {
            xpos: [pos[3 * light], pos[3 * light + 1], pos[3 * light + 2]],
            xdir,
        });
    }
    Ok(poses)
}

/// R09-S2b：`splat_scale` 三常量槽 = `[znear, zfar, shadow_light_fraction]`。
///
/// 第三分量原为占位 `0.0`（S7b2 unused），R09-S2 起改作 `shadow_light_fraction`
/// （上游 `render.py` L880；内核读 `splat_scale.data[0][2]`）。抽成纯函数以便 in-crate
/// 单测直接锁「shadow_light_fraction → index 2」的映射：本模块不构造完整
/// `ResolvedContext`（30+ 字段且无 `Default`，构造属集成测试面）。
pub(crate) fn scale_constants(znear: f64, zfar: f64, shadow_light_fraction: f64) -> Vec<f32> {
    vec![znear as f32, zfar as f32, shadow_light_fraction as f32]
}

/// R09-S2b：第 40 槽（0-based `MEGAKERNEL_SLOTS[40]`，1-based 第 41 项）的阴影总开关实参。
///
/// 该槽在 S7a 冻结签名里是 `Scalar(Bool)`；host 自 S7b3 起传 `resolved.use_shadows`，
/// R09-S2b 移除 host 前置拒绝后产品路径可达（行为验证在 R09-S3b 窗，本片不作数值结论）。
pub(crate) fn use_shadows_arg(use_shadows: bool) -> RenderArg {
    RenderArg::Scalar(RenderScalar::Bool(use_shadows))
}

/// R02-S1：天空盒位（bit 9；位表见证据 README，内核 `MK_FLAG_SKYBOX` 0x200u）。
pub(crate) const FLAG_SKYBOX_BIT: u32 = 1 << 9;

/// S7b2/S7b3 自有参数槽：着色常量（4×vec3）、缩放常量（vec3）与 flags word。
/// 返回 `(shading_constants, scale_constants, flags_word)`；抽到本模块以保持
/// `megakernel.rs` 的 500 行棘轮。
pub(crate) fn shading_words(
    resolved: &ResolvedContext,
    nsamples: usize,
) -> (Vec<f32>, Vec<f32>, u64) {
    let headlight = &resolved.headlight;
    let mut shading_constants = Vec::with_capacity(12);
    shading_constants.extend_from_slice(&headlight.ambient);
    shading_constants.extend_from_slice(&headlight.diffuse);
    shading_constants.extend_from_slice(&headlight.specular);
    shading_constants.extend_from_slice(&[
        resolved.background_color[0] as f32,
        resolved.background_color[1] as f32,
        resolved.background_color[2] as f32,
    ]);
    // R09-S2：第三分量原为占位 `0.0`（unused），改作 `shadow_light_fraction`
    // （上游 L880；内核 `splat_scale.data[0][2]` 读取）；R09-S2b 抽纯函数并加接线单测。
    let scale_words = scale_constants(
        resolved.znear,
        resolved.zfar,
        resolved.shadow_light_fraction,
    );
    let mut flag_bits = 0u32;
    flag_bits |= u32::from(resolved.use_textures);
    flag_bits |= u32::from(resolved.enable_vertex_normals) << 1;
    flag_bits |= u32::from(resolved.enable_specular) << 2;
    flag_bits |= u32::from(resolved.enable_emission) << 3;
    flag_bits |= u32::from(resolved.enable_per_light_ambient) << 4;
    flag_bits |= u32::from(resolved.use_ambient_lighting) << 5;
    flag_bits |= u32::from(headlight.active) << 6;
    flag_bits |= u32::from(resolved.enable_backface_culling) << 7;
    flag_bits |= u32::from(nsamples > 1) << 8;
    if resolved.render_skybox {
        flag_bits |= FLAG_SKYBOX_BIT;
    }
    let flags_word = (u64::from(resolved.background_color_packed) << 32) | u64::from(flag_bits);
    (shading_constants, scale_words, flags_word)
}

/// R08-RGB：逐相机通道掩码 → 发射实参（1/0）。
///
/// 81 槽签名的 `render_rgb`/`render_depth`/`render_seg`（0-based 51/52/53）本就是
/// 长度 `nrender` 的 `Array(Bool)`；这里把 `CameraPlan` 的逐激活相机布尔按原序转成
/// `i32`（`bool_words` 再按 4 bool/u32 打包）。**禁止用全局布尔填充**：关闭通道的
/// 相机 `*_adr = -1`，内核按 `do_rgb/do_depth/do_seg` 跳过写入。
pub(crate) fn channel_flags(flags: &[bool]) -> Vec<i32> {
    flags.iter().map(|enabled| i32::from(*enabled)).collect()
}

#[cfg(test)]
mod tests {
    use super::{channel_flags, scale_constants, use_shadows_arg};
    use crate::runtime::launcher::{
        MEGAKERNEL_SLOTS, RenderArg, RenderArgKind, RenderScalar, RenderScalarKind,
    };

    /// R08-RGB：逐相机掩码逐项映射（禁止全局布尔折叠）。
    #[test]
    fn channel_flags_maps_each_camera_independent() {
        assert_eq!(channel_flags(&[true, false, true]), vec![1, 0, 1]);
        assert_eq!(channel_flags(&[false, true]), vec![0, 1]);
        assert_eq!(channel_flags(&[true, true]), vec![1, 1]);
    }

    /// R08-RGB：全 false 与空输入按原长度返回（长度由 `CameraPlan` 保证 = nrender）。
    #[test]
    fn channel_flags_handles_all_false_and_empty() {
        assert_eq!(channel_flags(&[false, false]), vec![0, 0]);
        assert!(channel_flags(&[]).is_empty());
    }

    /// R09-S2b：`shadow_light_fraction` 必须落在 `splat_scale` 第三分量（index 2）。
    #[test]
    fn scale_constants_maps_shadow_fraction_to_third_component() {
        let words = scale_constants(0.1, 100.0, 0.3);
        assert_eq!(words.len(), 3);
        // 用 bit 比较：避免浮点相等断言被 clippy::float_cmp 拦；映射必须逐位精确。
        assert_eq!(words[0].to_bits(), 0.1f32.to_bits(), "index 0 必须是 znear");
        assert_eq!(
            words[1].to_bits(),
            100.0f32.to_bits(),
            "index 1 必须是 zfar"
        );
        assert_eq!(
            words[2].to_bits(),
            0.3f32.to_bits(),
            "index 2 必须是 shadow_light_fraction"
        );
    }

    /// 原占位 `0.0` 的语义已改为「`shadow_light_fraction = 0`」，不再被当作 unused 丢弃。
    #[test]
    fn scale_constants_placeholder_now_is_shadow_fraction() {
        let words = scale_constants(0.5, 50.0, 0.0);
        assert_eq!(words.len(), 3);
        assert_eq!(words[0].to_bits(), 0.5f32.to_bits());
        assert_eq!(words[1].to_bits(), 50.0f32.to_bits());
        assert_eq!(words[2].to_bits(), 0.0f32.to_bits());
        // 默认 `shadow_light_fraction = 0.3`（`params.rs` 既有权重）原样落到第三分量。
        assert_eq!(
            scale_constants(0.5, 50.0, 0.3)[2].to_bits(),
            0.3f32.to_bits()
        );
    }

    /// R09-S2b：布尔总开关经纯函数写实参，不依赖字符串/结构断言。
    #[test]
    fn use_shadows_arg_maps_boolean_to_slot_40_value() {
        assert!(matches!(
            use_shadows_arg(true),
            RenderArg::Scalar(RenderScalar::Bool(true))
        ));
        assert!(matches!(
            use_shadows_arg(false),
            RenderArg::Scalar(RenderScalar::Bool(false))
        ));
    }

    /// 冻结签名的第 40 槽（0-based）= 1-based 第 41 项 = `Scalar(Bool)`；81 槽不变。
    #[test]
    fn slot_40_is_bool_scalar_in_frozen_signature() {
        // P10：上游 81 槽 + 1 个尾随诊断槽 = 82。
        assert_eq!(MEGAKERNEL_SLOTS.len(), 82);
        assert_eq!(
            MEGAKERNEL_SLOTS[40],
            RenderArgKind::Scalar(RenderScalarKind::Bool)
        );
    }
}
