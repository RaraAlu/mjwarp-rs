//! S7b1 渲染输入：相机位姿、模型透明度数组与三份设备打包（渲染/BVH S7b1）。
//!
//! - [`CameraPose`]：相机 data 世界位姿（`cam_xpos`/`cam_xmat` f32 位型，上游 `d.cam_*`）；
//!   固定相机可由 `f32(cam_pos)` + `quat_to_mat(f32(cam_quat))` 派生（`cam_bodyid=[0,0]`），
//!   TARGETBODY 跟踪相机的 data `cam_xmat` 必须由调用方传入（见 `output::CameraData`）。
//! - [`ModelRenderArrays`]：`_geom_rgba`（透明 geom 跳过）需要的最小模型数组。
//! - [`SceneDevice`]：场景 BVH 的 device 布局（与 S2b `scene.cu` 同式，另加 world 根与
//!   leaf→网格绑定；不改 `src/render/bvh/**`）。
//! - [`MeshDevice`]：网格三角 BVH 的 device 布局（S3 `mesh.cu` 语义的拼接版）。
//! - [`CameraDevice`]：按**激活相机顺序**打包的相机参数/位姿/输出偏移。
//!
//! 全部布局常量钉死在对应类型的文档注释里；发射侧只读取这些字段上传，不在设备侧做
//! 布局推断。像素内核见 `kernel/cuda/render.cu`。

use crate::math::quat_to_mat;
use crate::model::render::{FieldData, RenderFields};
use crate::render::bvh::{NodeKind, SceneBvh};
use crate::render::camera::CameraModel;
use crate::render::context::ResolvedContext;
use crate::render::context::owner::SceneState;
use crate::runtime::launcher::LauncherError;

// 拆分②：网格三角 BVH 打包以 `#[path]` 在本文件内声明为子模块，父模块 `kernel/mod.rs` 零改动
// （卡片 scope 只含 `cast.rs` 与新文件；`megakernel.rs` 拆分①同款先例）。
#[path = "cast_mesh.rs"]
mod cast_mesh;

pub use cast_mesh::MeshDevice;

/// 渲染输入解析错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PoseError {
    Missing(&'static str),
    Kind(&'static str),
    Length {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl std::fmt::Display for PoseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(field) => write!(formatter, "缺少渲染字段 {field}"),
            Self::Kind(field) => write!(formatter, "渲染字段 {field} 类型不符"),
            Self::Length {
                field,
                expected,
                actual,
            } => write!(formatter, "{field} 长度 {actual} != {expected}"),
        }
    }
}

impl std::error::Error for PoseError {}

/// 单个相机的世界位姿（`cam_xpos` + 行主序 `cam_xmat`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub xpos: [f32; 3],
    pub xmat: [f32; 9],
}

impl CameraPose {
    /// 从 S1 `RenderFields` 的 `cam_pos`/`cam_quat` 派生全部模型相机世界位姿。
    pub fn from_fields(fields: &RenderFields) -> Result<Vec<Self>, PoseError> {
        let position = real_array(fields, "cam_pos")?;
        let quaternion = real_array(fields, "cam_quat")?;
        if !position.len().is_multiple_of(3) {
            return Err(PoseError::Length {
                field: "cam_pos",
                expected: position.len() / 3 * 3,
                actual: position.len(),
            });
        }
        let count = position.len() / 3;
        if quaternion.len() != count * 4 {
            return Err(PoseError::Length {
                field: "cam_quat",
                expected: count * 4,
                actual: quaternion.len(),
            });
        }
        let mut poses = Vec::with_capacity(count);
        for index in 0..count {
            let base = index * 3;
            let xpos = [
                position[base] as f32,
                position[base + 1] as f32,
                position[base + 2] as f32,
            ];
            let qbase = index * 4;
            let quat = [
                quaternion[qbase] as f32,
                quaternion[qbase + 1] as f32,
                quaternion[qbase + 2] as f32,
                quaternion[qbase + 3] as f32,
            ];
            let matrix = quat_to_mat(quat);
            poses.push(Self {
                xpos,
                xmat: [
                    matrix[0][0],
                    matrix[0][1],
                    matrix[0][2],
                    matrix[1][0],
                    matrix[1][1],
                    matrix[1][2],
                    matrix[2][0],
                    matrix[2][1],
                    matrix[2][2],
                ],
            });
        }
        Ok(poses)
    }

    /// 行主序旋转矩阵视图。
    pub fn rotation(&self) -> [[f32; 3]; 3] {
        [
            [self.xmat[0], self.xmat[1], self.xmat[2]],
            [self.xmat[3], self.xmat[4], self.xmat[5]],
            [self.xmat[6], self.xmat[7], self.xmat[8]],
        ]
    }

    /// `cam_xmat @ v`（与上游 kernel 相同的行点积语义；f32 顺序 `((m0*v0+m1*v1)+m2*v2)`）。
    pub fn rotate(&self, vector: [f32; 3]) -> [f32; 3] {
        let matrix = self.rotation();
        let mut out = [0.0f32; 3];
        for axis in 0..3 {
            out[axis] = matrix[axis][0] * vector[0]
                + matrix[axis][1] * vector[1]
                + matrix[axis][2] * vector[2];
        }
        out
    }
}

/// `_geom_rgba` 与 `geom_type` 需要的最小模型数组（冻结上游 `render.py::_geom_rgba`）。
/// 模型数组的 world 维在 S7b1 固定为 1（scene-a 单模型），内核读 `worldid % 1 == 0`。
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRenderArrays {
    pub geom_type: Vec<i32>,
    pub geom_rgba: Vec<f32>,
    pub geom_matid: Vec<i32>,
    pub mat_rgba: Vec<f32>,
    pub geom_dataid: Vec<i32>,
}

impl ModelRenderArrays {
    /// 从 S1 渲染字段取数组并校验长度（`geom_rgba`/`mat_rgba` 为 4 通道交错）。
    pub fn from_fields(fields: &RenderFields) -> Result<Self, LauncherError> {
        let geom_type = ints(fields, "geom_type")?;
        let geom_matid = ints(fields, "geom_matid")?;
        let geom_dataid = ints(fields, "geom_dataid")?;
        let geom_rgba = floats(fields, "geom_rgba")?;
        let mat_rgba = floats(fields, "mat_rgba")?;
        let ngeom = geom_type.len();
        if geom_matid.len() != ngeom || geom_dataid.len() != ngeom || geom_rgba.len() != 4 * ngeom {
            return Err(LauncherError::InvalidArray("geom_model_arrays"));
        }
        if !mat_rgba.len().is_multiple_of(4) {
            return Err(LauncherError::InvalidArray("mat_rgba"));
        }
        Ok(Self {
            geom_type,
            geom_rgba,
            geom_matid,
            mat_rgba,
            geom_dataid,
        })
    }
}

/// 场景 BVH 的 device 打包（`render.cu::scene_view` 消费）。
///
/// ```text
/// meta   = [n_nodes, n_leaf, n_world, reserved,
///           roots(n_world), links(2*n_nodes), leaf_geom(n_leaf), leaf_mesh(n_leaf)]
/// params = [node bounds 6*n_nodes, leaf size 3*n_leaf, leaf xpos 3*n_leaf,
///           leaf xmat(row-major) 9*n_leaf]
/// ```
///
/// `leaf_geom` 取 S2a 的 `SceneBvh::leaf_geom_id`；`leaf_mesh` 是 `geom_dataid`
/// （mesh 类型 7/8）或 -1（primitive），S7b1 用它选择网格三角 BVH 段。
#[derive(Clone, Debug, PartialEq)]
pub struct SceneDevice {
    pub meta: Vec<i32>,
    pub params: Vec<f32>,
    pub n_leaf: usize,
}

impl SceneDevice {
    /// 按当前 `SceneBvh` 节点/叶与 `SceneState` 的位姿打包；`mesh_count` 用于在打包期
    /// 拒绝越界的 `leaf_mesh`。
    pub fn pack(
        bvh: &SceneBvh,
        state: &SceneState,
        model: &ModelRenderArrays,
        mesh_count: usize,
    ) -> Result<Self, LauncherError> {
        let nodes = bvh.nodes();
        let n_nodes = nodes.len();
        let n_leaf = bvh.leaf_count();
        let worlds = bvh.worlds();
        if n_leaf == 0 || n_nodes == 0 || worlds == 0 {
            return Err(LauncherError::InvalidArray("scene_bvh"));
        }
        if state.leaves().len() != n_leaf || state.poses().len() != n_leaf {
            return Err(LauncherError::InvalidArray("scene_leaf_count"));
        }
        let meta_len = 4usize
            .checked_add(worlds)
            .and_then(|value| value.checked_add(2 * n_nodes))
            .and_then(|value| value.checked_add(2 * n_leaf))
            .ok_or(LauncherError::Overflow("scene_meta"))?;
        let params_len = 6usize
            .checked_mul(n_nodes)
            .and_then(|value| value.checked_add(15 * n_leaf))
            .ok_or(LauncherError::Overflow("scene_params"))?;
        let mut meta = vec![0i32; meta_len];
        meta[0] = to_i32(n_nodes, "scene_nodes")?;
        meta[1] = to_i32(n_leaf, "scene_leaf")?;
        meta[2] = to_i32(worlds, "scene_worlds")?;
        let links = 4 + worlds;
        for world in 0..worlds {
            let root = bvh
                .group_root(world)
                .map_err(|_| LauncherError::InvalidArray("scene_world_root"))?;
            meta[4 + world] = to_i32(root as usize, "scene_root")?;
        }
        for (index, node) in nodes.iter().enumerate() {
            match node.kind {
                NodeKind::Internal { left, right } => {
                    meta[links + 2 * index] = to_i32(left as usize, "scene_child")?;
                    meta[links + 2 * index + 1] = to_i32(right as usize, "scene_child")?;
                }
                NodeKind::Leaf { leaf } => {
                    meta[links + 2 * index] = -to_i32(leaf as usize, "scene_leaf")? - 1;
                    meta[links + 2 * index + 1] = 0;
                }
            }
        }
        let leaf_geom = links + 2 * n_nodes;
        let mut params = vec![0.0f32; params_len];
        let leaf_size = 6 * n_nodes;
        let leaf_xpos = leaf_size + 3 * n_leaf;
        let leaf_xmat = leaf_xpos + 3 * n_leaf;
        for (leaf, (geometry, pose)) in state.leaves().iter().zip(state.poses()).enumerate() {
            let geom = bvh
                .leaf_geom_id(leaf as u32)
                .ok_or(LauncherError::InvalidArray("scene_leaf_geom"))?;
            meta[leaf_geom + leaf] = geom;
            let geom_index =
                usize::try_from(geom).map_err(|_| LauncherError::InvalidArray("scene_geom_id"))?;
            let geom_type = *model
                .geom_type
                .get(geom_index)
                .ok_or(LauncherError::InvalidArray("scene_geom_type"))?;
            let leaf_mesh = if matches!(geom_type, 7 | 8) {
                *model
                    .geom_dataid
                    .get(geom_index)
                    .ok_or(LauncherError::InvalidArray("scene_geom_dataid"))?
            } else {
                -1
            };
            if leaf_mesh >= 0 && leaf_mesh as usize >= mesh_count {
                return Err(LauncherError::InvalidArray("scene_leaf_mesh"));
            }
            meta[leaf_geom + n_leaf + leaf] = leaf_mesh;
            params[leaf_size + 3 * leaf..leaf_size + 3 * leaf + 3].copy_from_slice(&geometry.size);
            params[leaf_xpos + 3 * leaf..leaf_xpos + 3 * leaf + 3].copy_from_slice(&pose.xpos);
            params[leaf_xmat + 9 * leaf..leaf_xmat + 9 * leaf + 9].copy_from_slice(&pose.xmat);
        }
        for (index, node) in nodes.iter().enumerate() {
            let offset = 6 * index;
            params[offset..offset + 3].copy_from_slice(&node.bounds.lower);
            params[offset + 3..offset + 6].copy_from_slice(&node.bounds.upper);
        }
        Ok(Self {
            meta,
            params,
            n_leaf,
        })
    }
}

/// 按激活相机顺序打包的相机参数/位姿/输出偏移（`render.cu` 内核按 `camid` 读取）。
#[derive(Clone, Debug, PartialEq)]
pub struct CameraDevice {
    pub projection: Vec<i32>,
    pub fovy: Vec<f32>,
    /// `(sensorsize.x, sensorsize.y)` 交错，2 f32/相机。
    pub sensorsize: Vec<f32>,
    /// `cam_intrinsic` 4 f32/相机。
    pub intrinsic: Vec<f32>,
    /// `cam_xpos` 3 f32/相机。
    pub xpos: Vec<f32>,
    /// 行主序 `cam_xmat` 9 f32/相机。
    pub xmat: Vec<f32>,
    /// `(width, height)`，`Vec2I` 元素。
    pub res: Vec<[i32; 2]>,
    pub depth_adr: Vec<i32>,
    pub seg_adr: Vec<i32>,
    pub render_depth: Vec<i32>,
    pub render_seg: Vec<i32>,
}

impl CameraDevice {
    /// 从模型相机输入、相机计划、S6a 输出布局与调用方传入的 **data 位姿** 推导。
    ///
    /// `poses` 按模型相机 id 索引（上游 `d.cam_xpos`/`d.cam_xmat` f32 位型）：固定相机可由
    /// `CameraPose::from_fields` 得到，TARGETBODY/TRACK 相机必须用 forward 的 data 位姿
    /// （`output::CameraData`），故本函数不从模型 `cam_quat` 重新推导。
    pub fn build(
        model: &CameraModel,
        resolved: &ResolvedContext,
        poses: &[CameraPose],
    ) -> Result<Self, LauncherError> {
        if poses.len() != model.ncam() {
            return Err(LauncherError::InvalidArray("camera_pose_count"));
        }
        let plan = &resolved.plan;
        let cameras = plan.cameras.len();
        if plan.resolutions.len() != cameras
            || resolved.output.depth_adr.len() != cameras
            || resolved.output.seg_adr.len() != cameras
        {
            return Err(LauncherError::InvalidArray("camera_plan"));
        }
        let mut device = Self {
            projection: Vec::with_capacity(cameras),
            fovy: Vec::with_capacity(cameras),
            sensorsize: Vec::with_capacity(2 * cameras),
            intrinsic: Vec::with_capacity(4 * cameras),
            xpos: Vec::with_capacity(3 * cameras),
            xmat: Vec::with_capacity(9 * cameras),
            res: Vec::with_capacity(cameras),
            depth_adr: resolved.output.depth_adr.clone(),
            seg_adr: resolved.output.seg_adr.clone(),
            render_depth: Vec::with_capacity(cameras),
            render_seg: Vec::with_capacity(cameras),
        };
        for (position, camera_id) in plan.cameras.iter().enumerate() {
            let camera = model
                .camera(*camera_id)
                .ok_or(LauncherError::InvalidArray("camera_index"))?;
            let pose = poses
                .get(*camera_id)
                .ok_or(LauncherError::InvalidArray("camera_pose_index"))?;
            device.projection.push(camera.projection.code());
            device.fovy.push(camera.fovy as f32);
            device.sensorsize.extend_from_slice(&camera.sensorsize);
            device.intrinsic.extend_from_slice(&camera.intrinsic);
            device.xpos.extend_from_slice(&pose.xpos);
            device.xmat.extend_from_slice(&pose.xmat);
            let resolution = plan.resolutions[position];
            device.res.push([
                to_i32(resolution[0] as usize, "camera_width")?,
                to_i32(resolution[1] as usize, "camera_height")?,
            ]);
            device
                .render_depth
                .push(i32::from(plan.render_depth[position]));
            device.render_seg.push(i32::from(plan.render_seg[position]));
        }
        Ok(device)
    }

    pub fn len(&self) -> usize {
        self.projection.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projection.is_empty()
    }
}

pub(super) fn to_i32(value: usize, field: &'static str) -> Result<i32, LauncherError> {
    i32::try_from(value).map_err(|_| LauncherError::Overflow(field))
}

fn ints(fields: &RenderFields, name: &'static str) -> Result<Vec<i32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Index(values)) => Ok(values.to_vec()),
        Some(_) => Err(LauncherError::InvalidArray(name)),
        None => Err(LauncherError::InvalidArray(name)),
    }
}

fn floats(fields: &RenderFields, name: &'static str) -> Result<Vec<f32>, LauncherError> {
    match fields.field(name) {
        Some(FieldData::Float(values)) => Ok(values.to_vec()),
        Some(_) => Err(LauncherError::InvalidArray(name)),
        None => Err(LauncherError::InvalidArray(name)),
    }
}

fn real_array(fields: &RenderFields, name: &'static str) -> Result<Vec<f64>, PoseError> {
    match fields.field(name) {
        Some(FieldData::Num(values)) => Ok(values.to_vec()),
        Some(FieldData::Float(values)) => {
            Ok(values.iter().map(|value| f64::from(*value)).collect())
        }
        Some(_) => Err(PoseError::Kind(name)),
        None => Err(PoseError::Missing(name)),
    }
}
