//! 渲染/BVH R10-S2 验收：nworld > 1 端到端与 world 隔离。
//!
//! 当前覆盖**宿主侧 per-world 相机位姿打包**（CPU，无 GPU）：广播（`ncam` 位姿 → 全部 world）
//! 与 per-world（`nworld × ncam`，world-major）两种输入、非法长度与超大计数的可诊断错误。
//! GPU 端到端（nworld=2 渲染、world 隔离、两次 SHA）在验证窗内执行，用例与窗口材料见
//! `docs/development-evidence/render-bvh/r10-s2-multi-world-e2e/`（窗口材料先报 Arch → Chief 签）。

#![cfg(feature = "cuda-probe")]

#[path = "support/render_bvh/r10.rs"]
mod r10;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;

use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, diff_seg};
use mjwarp_rs::render::kernel::world_camera::world_camera_pose_arrays;
use mjwarp_rs::render::kernel::{DepthSegRenderer, MegakernelRenderer};
use mjwarp_rs::render::resolve_context;
use mjwarp_rs::runtime::launcher::LauncherError;
use s7b1::{
    camera_data_poses, precomputed_rays, render_and_read, render_fields, scene_fixture, session,
};
use s7b2::{parse_rgb_fast_math_false_reference, read_rgb, recipe_depth_seg, reference_depth_seg};

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

/// GPU 用例（验证窗口内执行）：nworld=2 depth/seg 的 per-world 相机 + world 隔离。
///
/// 口径（方案 A）：world0 对**同配方**（`use_fast_math=False` 采集）的 nworld=1 冻结参考做位级
/// 对照（depth `mismatched == 0` / ≤4 ULP 兜底、seg 全等；fast-math `REFERENCE` 与 depth/seg 内核
/// 存在 2/4 ULP 已知跨配方偏差，S7b1 只作跨配方记录，故不作位移级基准）；world1 用确定性第二
/// state（相机绕 z 旋转 90°），要求 `nworld=2 的 world1 区段 == nworld=1 单独渲染同 state 逐位`；
/// 另用「world1 几何整体 +1000」验证 world0 不受 world1 场景影响、world1 全 miss/背景。
#[test]
#[ignore = "需要 GPU + NVRTC（验证窗口内执行）"]
fn gpu_nworld_two_depth_seg_per_world_cameras() {
    let session = session();
    let fields = render_fields();
    let fixture = scene_fixture();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let reference = recipe_depth_seg();
    let params1 = r10::depth_seg_params(1);
    let resolved1 = resolve_context(&fields, &params1).expect("nworld=1 解析");
    let base = camera_data_poses(&reference, &model, &resolved1);
    let world1: Vec<CameraPose> = base.iter().map(|pose| r10::rotated(pose, 1)).collect();
    let meshes = fixture.meshes.clone();
    let ctx1 = r10::context_with_state(
        &session,
        &fields,
        &params1,
        r10::state_worlds(&fixture, 1, 0.0),
        meshes.clone(),
        false,
    );
    let ctx2 = r10::context_with_state(
        &session,
        &fields,
        &r10::depth_seg_params(2),
        r10::state_worlds(&fixture, 2, 0.0),
        meshes.clone(),
        false,
    );
    let rays = precomputed_rays(&session, &fields);
    let renderer = DepthSegRenderer::new(&session).expect("depth/seg 内核");
    let layout = ctx2.resolved().output.clone();
    let per = r10::per_world_poses(&base, &world1);
    let (depth_a, seg_a) = render_and_read(&renderer, &fields, &model, &ctx2, &per, Some(&rays));
    let (depth_b, seg_b) = render_and_read(&renderer, &fields, &model, &ctx2, &per, Some(&rays));
    assert_eq!(depth_a.len(), 2 * layout.depth_pixels);
    assert_eq!(seg_a.len(), 2 * layout.seg_slots);
    assert_eq!(
        r10::sha256_depth(&depth_a),
        r10::sha256_depth(&depth_b),
        "两次运行 depth SHA 相同"
    );
    assert_eq!(
        r10::sha256_seg(&seg_a),
        r10::sha256_seg(&seg_b),
        "两次运行 seg SHA 相同"
    );
    let depth_stride = layout.depth_pixels;
    let seg_stride = layout.seg_slots;
    // world0：对既有 nworld=1 冻结参考（逐相机 ≤4 ULP / seg 全等）。
    for (index, camera) in reference.cameras.iter().enumerate() {
        let depth_adr = reference.depth_adr[index] as usize;
        let seg_adr = reference.seg_adr[index] as usize;
        let pixels = camera.pixels;
        let depth = &depth_a[depth_adr..depth_adr + pixels];
        let diff = s7b1::outline::recipe_depth_diff(depth, camera);
        assert_eq!(diff.mismatched, 0, "world0 相机 {index} depth 不匹配");
        assert!(
            diff.max_ulp <= MAX_DEPTH_ULP,
            "world0 相机 {index} depth ULP"
        );
        let seg = &seg_a[seg_adr..seg_adr + pixels];
        assert_eq!(diff_seg(seg, camera).expect("seg diff").mismatched, 0);
    }
    // world1：与 nworld=1 单独渲染同 state 逐位相同（同模型多 state 等价）。
    let (depth1, seg1) = render_and_read(&renderer, &fields, &model, &ctx1, &world1, Some(&rays));
    assert_eq!(
        &depth_a[depth_stride..],
        &depth1[..],
        "world1 depth 必须与单独渲染同 state 逐位相同"
    );
    assert_eq!(
        &seg_a[seg_stride..],
        &seg1[..],
        "world1 seg 必须与单独渲染同 state 全等"
    );
    assert!(
        depth_a[..depth_stride]
            .iter()
            .zip(&depth_a[depth_stride..])
            .any(|(left, right)| left.to_bits() != right.to_bits()),
        "per-world 相机必须生效（两 world 图像不同）"
    );
    // 隔离：world1 几何整体 +1000 后 world0 逐位不变、world1 全 miss/背景。
    let ctx_iso = r10::context_with_state(
        &session,
        &fields,
        &r10::depth_seg_params(2),
        r10::state_worlds(&fixture, 2, 1000.0),
        meshes,
        false,
    );
    let (depth_iso, seg_iso) =
        render_and_read(&renderer, &fields, &model, &ctx_iso, &per, Some(&rays));
    assert_eq!(
        &depth_iso[..depth_stride],
        &depth_a[..depth_stride],
        "world1 场景位移不得改变 world0"
    );
    assert!(
        depth_iso[depth_stride..].iter().all(|value| *value == 0.0),
        "world1 场景位移后必须全 miss"
    );
    assert!(
        seg_iso[seg_stride..].iter().all(|pair| *pair == [-1, -1]),
        "world1 场景位移后分割必须全背景"
    );
}

/// GPU 用例（验证窗口内执行）：nworld=2 RGB 的 per-world 相机 + world1 单 world 等价。
#[test]
#[ignore = "需要 GPU + NVRTC（验证窗口内执行）"]
fn gpu_nworld_two_rgb_per_world_cameras() {
    let session = s7b2::session();
    let fields = s7b2::render_fields();
    let fixture = scene_fixture();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let reference = parse_rgb_fast_math_false_reference();
    let depth_reference = reference_depth_seg();
    let params1 = s7b2::default_params();
    let resolved1 = resolve_context(&fields, &params1).expect("nworld=1 解析");
    let base = camera_data_poses(&depth_reference, &model, &resolved1);
    let world1: Vec<CameraPose> = base.iter().map(|pose| r10::rotated(pose, 1)).collect();
    let meshes = fixture.meshes.clone();
    let ctx1 = r10::context_with_state(
        &session,
        &fields,
        &params1,
        r10::state_worlds(&fixture, 1, 0.0),
        meshes.clone(),
        true,
    );
    let ctx2 = r10::context_with_state(
        &session,
        &fields,
        &r10::rgb_params(2),
        r10::state_worlds(&fixture, 2, 0.0),
        meshes,
        true,
    );
    let rays = s7b2::precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("megakernel");
    let layout = ctx2.resolved().output.clone();
    let per = r10::per_world_poses(&base, &world1);
    s7b2::render(&session, &renderer, &ctx2, &fields, &model, &per, &rays);
    let rgb_a = read_rgb(&ctx2);
    s7b2::render(&session, &renderer, &ctx2, &fields, &model, &per, &rays);
    let rgb_b = read_rgb(&ctx2);
    assert_eq!(rgb_a.len(), 2 * layout.rgb_pixels);
    assert_eq!(
        r10::sha256_rgb(&rgb_a),
        r10::sha256_rgb(&rgb_b),
        "两次运行 RGB SHA 相同"
    );
    // world0：对既有 nworld=1 冻结 RGB 参考（每相机 >1 LSB 像素 = 0）。
    let mut start = 0usize;
    for camera in &reference.cameras {
        let pixels = camera.packed.len();
        let stats = s7b2::diff_rgb(&rgb_a[start..start + pixels], camera);
        assert_eq!(stats.beyond_one_lsb_pixels, 0, "world0 RGB >1LSB 像素");
        start += pixels;
    }
    // world1：与 nworld=1 单独渲染同 state 逐位相同。
    s7b2::render(&session, &renderer, &ctx1, &fields, &model, &world1, &rays);
    let rgb1 = read_rgb(&ctx1);
    let stride = layout.rgb_pixels;
    assert_eq!(&rgb_a[stride..], &rgb1[..], "world1 RGB 必须逐位相同");
    assert!(
        rgb_a[..stride]
            .iter()
            .zip(&rgb_a[stride..])
            .any(|(left, right)| left != right),
        "per-world 相机必须生效（两 world 图像不同）"
    );
}
