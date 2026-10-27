//! 渲染/BVH R10-S2 验收：nworld > 1 端到端与 world 隔离。
//!
//! 当前覆盖**宿主侧 per-world 相机位姿打包**（CPU，无 GPU）：广播（`ncam` 位姿 → 全部 world）
//! 与 per-world（`nworld × ncam`，world-major）两种输入、非法长度与超大计数的可诊断错误。
//! GPU 端到端（nworld=2 渲染、world 隔离、两次 SHA）在验证窗内执行，用例与窗口材料见
//! `docs/development-evidence/render-bvh/r10-s2-multi-world-e2e/`（窗口材料先报 Arch → Chief 签）。

#![cfg(feature = "cuda-probe")]

use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::world_camera::world_camera_pose_arrays;
use mjwarp_rs::runtime::launcher::LauncherError;

fn pose(seed: f32) -> CameraPose {
    CameraPose {
        xpos: [seed, seed + 1.0, seed + 2.0],
        xmat: [seed, 0.0, 0.0, 0.0, seed + 4.0, 0.0, 0.0, 0.0, seed + 8.0],
    }
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

#[test]
fn single_world_poses_broadcast_to_all_worlds() {
    // 既有 S8 路径：ncam 位姿 + nworld>1 → 每个 world 一份相同位姿（world-major）。
    let poses = [pose(1.0), pose(2.0)];
    let (xpos, xmat) = world_camera_pose_arrays(&poses, 3, 2).expect("广播 per-world 位姿");
    assert_eq!((xpos.len(), xmat.len()), (3 * 3 * 2, 9 * 3 * 2));
    let expected_xpos: Vec<f32> = (0..3)
        .flat_map(|_| [1.0, 2.0, 3.0, 2.0, 3.0, 4.0])
        .collect();
    let expected_xmat: Vec<f32> = (0..3)
        .flat_map(|_| {
            let mut matrix = Vec::new();
            for seed in [1.0f32, 2.0] {
                matrix.extend_from_slice(&[
                    seed,
                    0.0,
                    0.0,
                    0.0,
                    seed + 4.0,
                    0.0,
                    0.0,
                    0.0,
                    seed + 8.0,
                ]);
            }
            matrix
        })
        .collect();
    assert_eq!(bits(&xpos), bits(&expected_xpos));
    assert_eq!(bits(&xmat), bits(&expected_xmat));
}

#[test]
fn per_world_poses_are_world_major_and_keep_bits() {
    // 同模型多 state：nworld × ncam 位姿按 world-major 展开，位型逐位保持。
    let poses = [pose(1.0), pose(2.0), pose(11.0), pose(12.0)];
    let (xpos, xmat) = world_camera_pose_arrays(&poses, 2, 2).expect("per-world 位姿");
    assert_eq!((xpos.len(), xmat.len()), (2 * 3 * 2, 2 * 9 * 2));
    // world0 = poses[0..2]，world1 = poses[2..4]。
    let mut expected_xpos = Vec::new();
    let mut expected_xmat = Vec::new();
    for pose in &poses {
        expected_xpos.extend_from_slice(&pose.xpos);
        expected_xmat.extend_from_slice(&pose.xmat);
    }
    assert_eq!(bits(&xpos), bits(&expected_xpos));
    assert_eq!(bits(&xmat), bits(&expected_xmat));
    // world 维独立：world1 的相机 0 位姿来自 poses[2]，不是 poses[0]。
    assert_eq!(&xpos[6..9], &[11.0, 12.0, 13.0]);
}

#[test]
fn invalid_pose_counts_and_huge_worlds_are_rejected_without_panic() {
    let poses = [pose(1.0), pose(2.0)];
    // 长度既不是 ncam 也不是 nworld×ncam。
    assert!(matches!(
        world_camera_pose_arrays(&poses, 2, 3).unwrap_err(),
        LauncherError::InvalidArray("camera_pose_count")
    ));
    // nworld / ncam 为 0。
    assert!(matches!(
        world_camera_pose_arrays(&poses, 0, 2).unwrap_err(),
        LauncherError::InvalidArray("camera_pose_count")
    ));
    assert!(matches!(
        world_camera_pose_arrays(&poses, 1, 0).unwrap_err(),
        LauncherError::InvalidArray("camera_pose_count")
    ));
    // 超大 nworld × ncam 走 checked_mul → Overflow，不 panic。
    let huge = std::panic::catch_unwind(|| world_camera_pose_arrays(&poses, usize::MAX, 2));
    let huge = huge.expect("world_camera_pose_arrays 不应 panic");
    assert!(matches!(
        huge,
        Err(LauncherError::Overflow("camera_pose_count"))
    ));
}
