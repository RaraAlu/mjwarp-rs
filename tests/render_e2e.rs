#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S8 端到端验收：产品 camlight data 位姿 → 渲染内核（两相机 RGB/depth/seg）。
//!
//! 硬口径（Arch 2026-10-25 S8 start）：
//! - 渲染输入的相机 data 位姿只能来自 `physics::camlight`（`src/physics/camlight.rs`），
//!   夹具 `camera_data` 只用于**比对**，绝不作为渲染输入；
//! - RGB 对同配方（`use_fast_math=False`）参考 ≤1 LSB、depth ≤4 ULP、seg 全等；
//!   S0b fast-math 参考的差异只作跨配方已知偏差记录，不作通过依据；
//! - 反证：产品 camlight 输出注入扰动后渲染必须随之变化；夹具 data 位姿被篡改
//!   不得影响以产品位姿渲染的结果（渲染不读夹具）。
//!
//! GPU/NVRTC 用例默认 ignore：
//! cargo test --locked --features cuda-probe --test render_e2e -- --ignored --test-threads=1

#[path = "support/render_bvh/e2e.rs"]
mod e2e;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;

use mjwarp_rs::diagnostics::{InputError, TransferError};
use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::context::owner::OwnerError;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::ulp_distance;
use mjwarp_rs::render::{ContextParams, ImageOutput, device_memory, release_stats};
use mjwarp_rs::runtime::launcher::LauncherError;
use s7b2::{
    create_context, default_params, parse_rgb_fast_math_false_reference, parse_rgb_reference,
    precomputed_rays, recipe_depth_seg, render_fields, session,
};

/// destroy 后允许的显存增长（与 S6b `MAX_CYCLE_GROWTH_BYTES` 同口径）。
const MAX_LIFECYCLE_GROWTH_BYTES: usize = 16 << 20;

/// 主机：产品 camlight 输入夹具覆盖冻结 scene-a 维度；参考夹具形状与 r01-p8 证据路径可见。
#[test]
fn fixtures_cover_frozen_scene_and_closure_evidence_paths() {
    let fixture = e2e::camlight_model_fixture();
    let expected = [
        ("nq", 8usize),
        ("nv", 7),
        ("nbody", 3),
        ("njnt", 2),
        ("ngeom", 7),
        ("nsite", 0),
        ("nmocap", 0),
        ("ncam", 2),
        ("nlight", 3),
    ];
    assert_eq!(fixture.counts, expected.to_vec());
    assert_eq!(fixture.qpos0.len(), 8, "qpos0 是模型默认姿态");
    let recipe = recipe_depth_seg();
    assert_eq!((recipe.total_rays, recipe.camera_data.len()), (120_000, 2));
    assert_eq!(
        (
            recipe.camera_data[0].mode,
            recipe.camera_data[1].mode,
            recipe.camera_data[1].target_bodyid
        ),
        (0, 3, 2),
        "相机 0 fixed、相机 1 TARGETBODY 跟踪 body 2"
    );
    assert_eq!(parse_rgb_fast_math_false_reference().total_pixels, 120_000);
    // r01-p8 收口映射的可追溯路径（十项证据文件必须存在；只做存在性判定）。
    let paths = [
        (
            "世界/相机/资产(S1)",
            "docs/development-evidence/render-bvh/s1-model-input/receipt.json",
        ),
        (
            "BVH(S2)",
            "docs/development-evidence/render-bvh/s2b-scene-bvh-gpu/receipt.json",
        ),
        (
            "BVH(S3)",
            "docs/development-evidence/render-bvh/s3-mesh-bvh/README.md",
        ),
        (
            "28默认参数(S6a)",
            "docs/development-evidence/render-bvh/s6a-context-params/receipt.json",
        ),
        (
            "生命周期(S6b)",
            "docs/development-evidence/render-bvh/s6b-context-lifecycle/receipt.json",
        ),
        (
            "RGB(S7b2)",
            "docs/development-evidence/render-bvh/s7b2-shading/receipt.json",
        ),
        (
            "深度(S7b1)",
            "docs/development-evidence/render-bvh/s7b1-depth-seg/receipt.json",
        ),
        (
            "分割(S7b1)",
            "docs/development-evidence/render-bvh/s7b1-depth-seg/receipt.json",
        ),
        (
            "资源检查(S6b)",
            "docs/development-evidence/render-bvh/s6b-context-lifecycle/README.md",
        ),
        (
            "拒绝面(S7b2)",
            "docs/development-evidence/render-bvh/s7b2-shading/README.md",
        ),
        (
            "flag/AA(S7b3)",
            "docs/development-evidence/render-bvh/s7b3-flags/receipt.json",
        ),
    ];
    for (item, path) in paths {
        assert!(
            std::path::Path::new(path).exists(),
            "r01-p8 收口映射「{item}」的可追溯路径缺失：{path}"
        );
    }
    println!(
        "s8[host] camlight 输入夹具覆盖 nbody={} nq={} ncam={} nlight={}；参考 120,000 像素；\
         r01-p8 十项证据路径可见",
        fixture.counts[2].1, fixture.counts[0].1, fixture.counts[7].1, fixture.counts[8].1
    );
}

/// GPU：产品 camlight data 位姿与 S7b1 同配方参考 `camera_data` 逐位一致；
/// camera0 与 quat_to_mat 辅助口径 ≤1 ULP、camera1 与参考 data xmat 位等；灯位姿一致。
#[test]
#[ignore = "需要 GPU + NVRTC（产品 camlight 内核）"]
fn product_camlight_poses_match_reference_bits() {
    let session = session();
    let fields = render_fields();
    let context = create_context(&session, &fields, &default_params());
    let recipe = recipe_depth_seg();
    let product = e2e::product_camlight(&session);
    let diff = e2e::compare_pose_bits(&product.poses, &recipe.camera_data);
    println!(
        "s8[camlight] product=origin_bits{:?} xmat_bits{:?} product_fk_body_xpos={:?}",
        product.poses[0].xpos.map(f32::to_bits),
        product.poses[0].xmat.map(f32::to_bits),
        product.body_xpos
    );
    println!("s8[camlight] bit_diff={diff:?}");
    for (index, data) in recipe.camera_data.iter().enumerate() {
        assert_eq!(data.index, index, "参考 camera_data 索引连续");
        assert_eq!(
            diff.origin_mismatches[index], 0,
            "cam{index} origin 必须与参考 data 位姿逐位一致（max_abs={}）",
            diff.max_origin_abs
        );
        assert_eq!(
            diff.xmat_mismatches[index], 0,
            "cam{index} xmat 必须与参考 data 位姿逐位一致（max_abs={}）",
            diff.max_xmat_abs
        );
    }
    // camera0（fixed）：以参考 data 位姿位等为硬口径；host `quat_to_mat` 是辅助口径——
    // warp `quat_to_mat` 走 `quat_rotate` 三列路径，与 host 代数展开可在末位差 1 ULP。
    let quat_poses = CameraPose::from_fields(&fields).expect("f32 quat 相机位姿");
    let mut max_ulp = 0i64;
    let mut components = 0usize;
    for (&actual, &expected) in product.poses[0].xmat.iter().zip(&quat_poses[0].xmat) {
        let ulp = ulp_distance(actual.to_bits(), expected.to_bits());
        components += usize::from(ulp > 0);
        max_ulp = max_ulp.max(ulp);
    }
    println!(
        "s8[camlight] camera0 vs host quat_to_mat: max_ulp={max_ulp} components_diff={components}/9\
         （辅助口径；权威为参考 data 位姿位等）"
    );
    assert!(
        max_ulp <= 1,
        "camera0 与 host quat_to_mat 派生值必须 ≤1 ULP（实际 {max_ulp}）"
    );
    assert_eq!(
        product.poses[0].xpos.map(f32::to_bits),
        quat_poses[0].xpos.map(f32::to_bits),
        "camera0 xpos 必须与 cam_pos 逐位一致"
    );
    // camera1（TARGETBODY）：data xmat 是 forward look-at 产物，必须与参考 data xmat 一致。
    assert_eq!(
        product.poses[1].xmat.map(f32::to_bits),
        recipe.camera_data[1].xmat_bits,
        "camera1 TARGETBODY xmat 必须与参考 data xmat 逐位一致"
    );
    // 灯位姿：产品 camlight 输出 == 渲染器 `light_data_poses` == pinned 3.15 参考位型。
    e2e::assert_light_poses(&product, &fields);
    // 夹具位姿只用于比对：解码后的位姿数组必须与产品输出逐位一致（不参与渲染）。
    let fixture_poses = s7b1::camera_data_poses(
        &recipe,
        &CameraModel::from_fields(&fields).expect("相机模型"),
        context.resolved(),
    );
    for (index, pose) in fixture_poses.iter().enumerate() {
        assert_eq!(
            pose.xpos.map(f32::to_bits),
            product.poses[index].xpos.map(f32::to_bits)
        );
        assert_eq!(
            pose.xmat.map(f32::to_bits),
            product.poses[index].xmat.map(f32::to_bits)
        );
    }
    println!("s8[camlight] 两相机 data 位姿 + 三灯位姿与 pinned 3.15 参考逐位一致");
}

/// GPU：端到端两相机 RGB ≤1 LSB、depth ≤4 ULP、seg 全等；两次运行哈希逐位相同；
/// 与夹具 data 位姿渲染结果逐位相同（位姿来源替换无副作用）。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn end_to_end_two_cameras_match_same_recipe_reference() {
    let session = session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let context = create_context(&session, &fields, &default_params());
    let product = e2e::product_camlight(&session);
    let recipe = recipe_depth_seg();
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S8 渲染内核编译");
    let (rgb, depth, seg) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    let layout = &context.resolved().output;
    // ① depth/seg 对同配方参考（≤4 ULP / seg 全等）与 ② RGB 同配方 ≤1 LSB。
    e2e::assert_depth_seg_same_recipe(&recipe, &depth, &seg);
    let same_recipe = parse_rgb_fast_math_false_reference();
    e2e::assert_rgb_same_recipe(&same_recipe, &recipe, &layout.rgb_adr, &rgb);
    // ③ 跨配方（S0b fast-math）偏差记录：只记录，不作通过依据。
    e2e::print_cross_recipe(&parse_rgb_reference(), &same_recipe, &layout.rgb_adr, &rgb);
    // ④ 两次运行输出哈希逐位相同（RGB + depth + seg）。
    let (rgb2, depth2, seg2) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    assert_eq!(rgb2, rgb, "两次运行 RGB 必须逐位相同");
    assert_eq!(depth2, depth, "两次运行 depth 必须逐位相同");
    assert_eq!(seg2, seg, "两次运行 seg 必须逐位相同");
    let depth_seg_hash = s7b1::output_hash(&depth, &seg);
    let rgb_hash = rgb.iter().fold(0u64, |hash, value| {
        (hash ^ u64::from(*value)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    println!(
        "s8[e2e] run: rgb_pixels={} depth_pixels={} seg_pairs={} depth_seg_hash={depth_seg_hash:016x} \
         rgb_hash={rgb_hash:016x}",
        rgb.len(),
        depth.len(),
        seg.len()
    );
    // ⑤ 位姿来源替换无副作用：夹具 data 位姿渲染结果与产品 camlight 位姿逐位相同。
    let fixture_poses = s7b1::camera_data_poses(&recipe, &model, context.resolved());
    let (rgb_fixture, depth_fixture, seg_fixture) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &fixture_poses, &rays);
    assert_eq!(rgb_fixture, rgb, "产品位姿 vs 夹具位姿 RGB 必须逐位相同");
    assert_eq!(
        depth_fixture, depth,
        "产品位姿 vs 夹具位姿 depth 必须逐位相同"
    );
    assert_eq!(seg_fixture, seg, "产品位姿 vs 夹具位姿 seg 必须逐位相同");
    println!(
        "s8[e2e] 产品 camlight 位姿与夹具 data 位姿渲染结果逐位相同（两相机 120,000/120,000）"
    );
}

/// GPU：反证——渲染输入的位姿来源是产品 camlight 输出，不是夹具回填。调用链：
/// `e2e::product_camlight` → `with_camlight`/`update_rigid`/`update_com`/`update_camlight`
/// → `readback` → `render_read_all(..., &product.poses, ...)`。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn render_input_poses_come_from_camlight_not_fixture() {
    let session = session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let context = create_context(&session, &fields, &default_params());
    let product = e2e::product_camlight(&session);
    let recipe = recipe_depth_seg();
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S8 渲染内核编译");
    let (rgb0, depth0, seg0) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);

    // ① 篡改**本地比对副本**的夹具 data 位姿（identity + 原点）：产品位姿渲染不受影响，
    //    且产品位姿仍与**原始**参考逐位一致（证明渲染不读夹具）。
    let tampered = e2e::tampered_camera_data(&recipe);
    // 判别力（QA 复核项）：篡改副本必须真与产品位姿不同，否则本反证只证明渲染确定性。
    let tamper_diff = e2e::compare_pose_bits(&product.poses, &tampered.camera_data);
    assert!(
        tamper_diff.origin_mismatches.iter().any(|n| *n > 0)
            || tamper_diff.xmat_mismatches.iter().any(|n| *n > 0),
        "篡改副本必须与产品位姿可判别（否则反证只证明确定性）"
    );
    let (rgb1, depth1, seg1) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    assert_eq!(
        (rgb1, depth1, seg1),
        (rgb0.clone(), depth0.clone(), seg0.clone()),
        "篡改夹具位姿不得影响产品位姿渲染"
    );
    let original_diff = e2e::compare_pose_bits(&product.poses, &recipe.camera_data);
    assert_eq!(
        original_diff.origin_mismatches.iter().sum::<usize>()
            + original_diff.xmat_mismatches.iter().sum::<usize>(),
        0,
        "产品位姿仍与原始参考逐位一致"
    );

    // ② 若渲染真的吃夹具：把篡改后的夹具位姿当输入，输出必须大幅变化（可检测）。
    let tampered_poses = s7b1::camera_data_poses(&tampered, &model, context.resolved());
    assert_ne!(
        tampered_poses[1].xmat.map(f32::to_bits),
        product.poses[1].xmat.map(f32::to_bits),
        "篡改夹具必须真正改变位姿"
    );
    let (rgb_tampered, _, _) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &tampered_poses, &rays);
    let changed = rgb_tampered
        .iter()
        .zip(&rgb0)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        changed > 0,
        "夹具位姿篡改必须可检测（若渲染从夹具回填，此处不会为 0）"
    );
    println!(
        "s8[reverse] 篡改夹具位姿作输入：RGB 变化 {changed}/{} 像素",
        rgb0.len()
    );

    // ③ 产品 camlight 输出注入扰动（camera1 xmat 旋转 + xpos 平移）：渲染必须随之变化。
    //    渲染内核用 cam_xmat/cam_xpos 把局部射线变换到世界系（render_megakernel.cu
    //    第 148-151 行），故扰动会改变射线世界方向，depth/分割/RGB 全部响应。
    let perturbed = e2e::perturb_camera1(&product.poses, 0.05, 0.3, 0.0);
    let (rgb_perturbed, depth_perturbed, seg_perturbed) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &perturbed, &rays);
    let rgb_changed = rgb_perturbed
        .iter()
        .zip(&rgb0)
        .filter(|(a, b)| a != b)
        .count();
    let depth_changed = depth_perturbed
        .iter()
        .zip(&depth0)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    let seg_changed = seg_perturbed
        .iter()
        .zip(&seg0)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        rgb_changed > 0 && depth_changed > 0 && seg_changed > 0,
        "产品 camlight 位姿扰动必须改变渲染输出（rgb={rgb_changed} depth={depth_changed} seg={seg_changed}）"
    );
    println!("s8[reverse] 扰动：rgb={rgb_changed} depth={depth_changed} seg={seg_changed}");
    // 同扰动重复渲染逐位相同（确定性）；不同扰动结果不同（扰动可区分）；恢复产品位姿后逐位复现。
    let (rgb_repeat, _, _) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &perturbed, &rays);
    assert_eq!(rgb_repeat, rgb_perturbed, "同扰动重复渲染必须逐位相同");
    let perturbed2 = e2e::perturb_camera1(&product.poses, 0.0, 0.0, 0.3);
    let (rgb_other, _, _) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &perturbed2, &rays);
    assert_ne!(rgb_other, rgb_perturbed, "不同扰动必须产生不同结果");
    let (rgb_restored, _, _) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    assert_eq!(rgb_restored, rgb0, "恢复产品位姿后渲染必须逐位复现");
    println!("s8[reverse] 反证成立：渲染输入位姿来源 = physics::camlight 输出（见 doc/README）");
}

/// GPU：生命周期 create→render→显式 refit→render→destroy；refit 帧结构断言；
/// 禁用输出 `DisabledOutput`；destroy 后显存回基线、无泄漏（S6b 口径）。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn lifecycle_create_render_refit_render_destroy() {
    let session = session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let product = e2e::product_camlight(&session);
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S8 渲染内核编译");
    // 预热两次完整链路（create→render→destroy）：把首次模块加载/驱动代码缓存排除在基线外
    // （S6b 记录过一次性驱动缓存 32 MiB 级；本片渲染内核首次 launch 同理）。
    for _ in 0..2 {
        let warm = create_context(&session, &fields, &default_params());
        e2e::render_read_all(&renderer, &warm, &fields, &model, &product.poses, &rays);
        warm.destroy();
    }
    let baseline = device_memory().expect("设备显存查询").0;
    let (created_before, destroyed_before) = release_stats();

    let mut context = create_context(&session, &fields, &default_params());
    let (rgb0, depth0, seg0) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    // 显式 refit：位姿未变 → 输出逐位相同（无冻结参考时只做结构断言）。
    context.refit_bvh().expect("显式 refit");
    assert_eq!(context.refits(), 1, "refit_bvh 是唯一重拟合入口");
    let (rgb1, depth1, seg1) =
        e2e::render_read_all(&renderer, &context, &fields, &model, &product.poses, &rays);
    assert_eq!(rgb1, rgb0, "位姿不变的 refit 不改变 RGB");
    assert_eq!(depth1, depth0, "位姿不变的 refit 不改变 depth");
    assert_eq!(seg1, seg0, "位姿不变的 refit 不改变 seg");
    println!("s8[lifecycle] create→render→refit(1)→render 输出逐位相同");
    context.destroy();
    let (created_after, destroyed_after) = release_stats();
    assert_eq!(created_after - created_before, 1, "创建计数");
    assert_eq!(destroyed_after - destroyed_before, 1, "销毁计数");
    let free = device_memory().expect("设备显存查询").0;
    let growth = baseline.saturating_sub(free);
    println!("s8[lifecycle] baseline={baseline} B free={free} B growth={growth} B");
    assert!(
        growth <= MAX_LIFECYCLE_GROWTH_BYTES,
        "destroy 后显存增长 {growth} B 超阈值 {MAX_LIFECYCLE_GROWTH_BYTES} B"
    );

    // 禁用输出：只启 rgb 的 context，depth/seg 读回必须 `InputError::DisabledOutput`。
    let params = ContextParams::default().with_render_targets(
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(true)),
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(false)),
        Some(mjwarp_rs::render::context::RenderFlagSpec::Bool(false)),
    );
    let disabled = create_context(&session, &fields, &params);
    assert!(disabled.outputs().is_enabled(ImageOutput::Rgb));
    for output in [ImageOutput::Depth, ImageOutput::Segmentation] {
        let rejected = matches!(
            disabled.outputs().read(output),
            Err(OwnerError::Transfer(TransferError::Input(
                InputError::DisabledOutput
            )))
        );
        assert!(rejected, "{output:?} 读取必须返回 DisabledOutput");
    }
    disabled.destroy();
    println!("s8[lifecycle] 禁用输出 DisabledOutput 判定成立");
}

/// GPU：负向控制保持——1 类写前拒绝（动态射线）+ AA 射线数；`use_shadows`/`render_skybox` 已解锁 → 独立「接受 → 发射成功」断言。
#[test]
#[ignore = "需要 GPU + NVRTC"]
fn negative_controls_hold_with_product_poses() {
    let session = session();
    let fields = render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let product = e2e::product_camlight(&session);
    let rays = precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("S8 渲染内核编译");
    let mut shadow_params = default_params();
    shadow_params.use_shadows = true;
    let shadow_context = create_context(&session, &fields, &shadow_params);
    let shadow_result = renderer.render(&shadow_context, &fields, &model, &product.poses, &rays);
    assert!(shadow_result.is_ok(), "use_shadows 接受");
    // R02-S1：`render_skybox=true` 已解锁（接受 + 发射成功 + RGB 缓冲已写）。
    let skybox_context = create_context(
        &session,
        &fields,
        &ContextParams {
            render_skybox: true,
            ..default_params()
        },
    );
    assert!(
        skybox_context.resolved().skybox.is_some(),
        "scene-a 天空盒纹理"
    );
    let skybox_result = renderer.render(&skybox_context, &fields, &model, &product.poses, &rays);
    assert!(skybox_result.is_ok(), "render_skybox 必须被接受");
    assert!(
        skybox_context.outputs().read(ImageOutput::Rgb).is_ok(),
        "skybox RGB"
    );
    // 动态射线仍写前拒绝。
    let dynamic_params = ContextParams {
        use_precomputed_rays: false,
        ..default_params()
    };
    let dynamic_context = create_context(&session, &fields, &dynamic_params);
    let dynamic_result = renderer.render(&dynamic_context, &fields, &model, &product.poses, &rays);
    assert!(
        matches!(dynamic_result, Err(LauncherError::Unsupported(_))),
        "use_precomputed_rays=false 必须在写前拒绝"
    );
    let aa_context = create_context(
        &session,
        &fields,
        &ContextParams {
            samples_per_pixel: 2,
            ..default_params()
        },
    );
    assert_eq!(aa_context.resolved().plan.samples, 4, "spp=2 → nsamples=4");
    let result = renderer.render(&aa_context, &fields, &model, &product.poses, &rays);
    assert!(
        matches!(result, Err(LauncherError::InvalidArray("ray_count"))),
        "AA 射线数不匹配必须拒绝（实际 {result:?}）"
    );
    aa_context.destroy();
    println!(
        "s8[negative] shadows+skybox accepted; dynamic-rays 写前拒绝 + AA 射线数（nsamples×total_rays）判定成立"
    );
}
