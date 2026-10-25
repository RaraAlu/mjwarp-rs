//! S7b2 渲染执行器的数据打包辅助（渲染/BVH S7b2）：字段读取、灯位姿推导、geam 位姿反解、
//! BVH blob 打包与设备上传。供 `megakernel.rs` 使用；行数口径按新文件拆分（各 ≤500 行）。

use crate::model::render::{FieldData, RenderFields};
use crate::render::kernel::cast::SceneDevice;
use crate::runtime::launcher::{LauncherError, RenderArg, RenderArrayValue, RenderElem};
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
        let geom = *leaf_geom
            .get(leaf)
            .ok_or(LauncherError::InvalidArray("scene_leaf_geom"))? as usize;
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
        let length = z.mul_add(z, y.mul_add(y, x * x)).sqrt();
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
