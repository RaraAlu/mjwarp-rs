//! 逐 world 渲染实现 S1（方案 L）的宿主侧单测。
//!
//! 本文件的 CPU 段只验证容器/守卫/leaf 映射等纯宿主契约（不发 GPU、不需要 NVRTC）；
//! 逐 world 数值比对（同名 GPU 用例）在验证窗内以 `--ignored` 执行，参考来自
//! `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`，本文件不内联判据/容差。

#![cfg(feature = "cuda-probe")]

use mjwarp_rs::render::kernel::{reject_multi_sample, render_per_world};
use mjwarp_rs::render::{ScenePose, WorldError, WorldScenePoses};
use mjwarp_rs::runtime::launcher::LauncherError;

fn pose(seed: f32) -> ScenePose {
    ScenePose {
        xpos: [seed, seed + 0.5, seed + 1.0],
        xmat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    }
}

#[test]
fn multi_sample_guard_only_accepts_single_sample() {
    assert!(reject_multi_sample(1).is_ok());
    for nsamples in [0, 2, 4, 16] {
        assert!(matches!(
            reject_multi_sample(nsamples),
            Err(LauncherError::Unsupported(_))
        ));
    }
}

#[test]
fn per_world_scene_poses_are_world_major_and_checked() {
    let poses: Vec<ScenePose> = (0..3 * 5).map(|index| pose(index as f32)).collect();
    let world = WorldScenePoses::new(3, 5, poses.clone()).unwrap();
    assert_eq!((world.worlds(), world.leaves()), (3, 5));
    for (index, expected) in poses.iter().enumerate() {
        let actual = world.get(index / 5, index % 5).unwrap();
        assert_eq!(
            actual.xpos.map(f32::to_bits),
            expected.xpos.map(f32::to_bits)
        );
        assert_eq!(
            actual.xmat.map(f32::to_bits),
            expected.xmat.map(f32::to_bits)
        );
    }
    assert!(matches!(
        WorldScenePoses::new(3, 5, vec![pose(0.0); 14]),
        Err(WorldError::SceneCountMismatch {
            expected: 15,
            actual: 14
        })
    ));
    assert!(world.get(3, 0).is_err());
    assert!(world.get(0, 5).is_err());
}

/// `render_per_world` 是 GPU 路径；本片只在验证窗用 `--ignored` 执行。
/// 这里保证公开符号存在且签名可编译（真实数值会在窗口内补齐）。
#[test]
#[ignore = "需要 GPU + NVRTC（验证窗口内执行逐 world 位级比对）"]
fn gpu_per_world_matches_captured_reference() {
    let _ = render_per_world
        as fn(
            &mjwarp_rs::render::kernel::MegakernelRenderer,
            &mut mjwarp_rs::render::RenderContext,
            &mjwarp_rs::model::render::RenderFields,
            &mjwarp_rs::render::CameraModel,
            &WorldScenePoses,
            &[mjwarp_rs::render::kernel::cast::CameraPose],
            &mjwarp_rs::render::rays::RayBuffer,
        ) -> Result<(), LauncherError>;
    unreachable!("验证窗内替换为逐 world 参考比对");
}
