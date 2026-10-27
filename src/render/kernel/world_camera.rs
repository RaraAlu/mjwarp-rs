//! R10-S2：nworld > 1 的相机位姿打包（宿主侧，无 GPU 运行）。
//!
//! 上游相机输入是 `cam_xpos_in[worldid, mujoco_cam_id]` / `cam_xmat_in[worldid, mujoco_cam_id]`；
//! 内核按 `worldid * (数组第二维) + 相机槽` 读取。本模块把调用方位姿归一成 **world-major 展平**
//! 数组（每 world 一份 ncam 相机位姿），供 megakernel 与 depth/seg 内核以 `[nworld, ncam]`
//! 形状上传。允许两种输入：
//!
//! - `poses.len() == ncam`：单 world 位姿，广播到全部 world（既有 S8 路径，nworld=1 逐位不变）；
//! - `poses.len() == nworld × ncam`：per-world 位姿（同模型多 state，world-major）。
//!
//! 其余长度返回 `InvalidArray("camera_pose_count")`，不 panic。

use crate::render::camera::CameraModel;
use crate::render::context::ResolvedContext;
use crate::runtime::launcher::LauncherError;

use super::cast::CameraPose;

/// 把调用方位姿归一成 `(xpos: 3*nworld*ncam, xmat: 9*nworld*ncam)` 的 world-major 展平数组。
pub fn world_camera_pose_arrays(
    poses: &[CameraPose],
    nworld: usize,
    ncam: usize,
) -> Result<(Vec<f32>, Vec<f32>), LauncherError> {
    if nworld == 0 || ncam == 0 {
        return Err(LauncherError::InvalidArray("camera_pose_count"));
    }
    let expected = nworld
        .checked_mul(ncam)
        .ok_or(LauncherError::Overflow("camera_pose_count"))?;
    let per_world = poses.len() == expected;
    if !per_world && poses.len() != ncam {
        return Err(LauncherError::InvalidArray("camera_pose_count"));
    }
    let mut xpos = Vec::new();
    xpos.try_reserve_exact(3 * expected)
        .map_err(|_| LauncherError::Overflow("camera_xpos"))?;
    let mut xmat = Vec::new();
    xmat.try_reserve_exact(9 * expected)
        .map_err(|_| LauncherError::Overflow("camera_xmat"))?;
    for world in 0..nworld {
        for camera in 0..ncam {
            let index = if per_world {
                world * ncam + camera
            } else {
                camera
            };
            let pose = poses[index];
            xpos.extend_from_slice(&pose.xpos);
            xmat.extend_from_slice(&pose.xmat);
        }
    }
    Ok((xpos, xmat))
}

/// 按**激活相机顺序**展开 per-world 位姿（供 depth/seg 内核以 `[nworld, nrender]` 上传）。
///
/// 输入同 [`world_camera_pose_arrays`]：`ncam`（广播）或 `nworld × ncam`（world-major）。
pub fn world_active_camera_arrays(
    model: &CameraModel,
    resolved: &ResolvedContext,
    poses: &[CameraPose],
) -> Result<(Vec<f32>, Vec<f32>), LauncherError> {
    let nworld = resolved.nworld;
    let ncam = model.ncam();
    let (xpos, xmat) = world_camera_pose_arrays(poses, nworld, ncam)?;
    let active = &resolved.plan.cameras;
    let expected = nworld
        .checked_mul(active.len())
        .ok_or(LauncherError::Overflow("camera_pose_count"))?;
    let mut active_xpos = Vec::new();
    active_xpos
        .try_reserve_exact(3 * expected)
        .map_err(|_| LauncherError::Overflow("camera_xpos"))?;
    let mut active_xmat = Vec::new();
    active_xmat
        .try_reserve_exact(9 * expected)
        .map_err(|_| LauncherError::Overflow("camera_xmat"))?;
    for world in 0..nworld {
        for camera_id in active {
            let index = world * ncam + camera_id;
            let start = 3 * index;
            active_xpos.extend_from_slice(&xpos[start..start + 3]);
            let start = 9 * index;
            active_xmat.extend_from_slice(&xmat[start..start + 9]);
        }
    }
    Ok((active_xpos, active_xmat))
}
