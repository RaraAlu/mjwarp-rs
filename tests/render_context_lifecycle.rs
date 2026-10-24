#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S6b 验收：render context 生命周期、资源所有权与显式 refit。
//!
//! 覆盖（plan S6b 五条验收）：
//! - 100 次 create/destroy 后显存回到阈值内，所有权类型不允许释放后使用；
//! - 移动位姿后 render 仍用旧 BVH，`refit_bvh()` 后命中新位置（上游语义）；
//! - render 不触发物理、不隐式 refit（重拟合计数与当前位姿都不变）；
//! - 禁用输出不可读（`InputError::DisabledOutput`）；
//! - context drop 释放 BVH/mesh/texture/output 全部资源。
//!
//! 参考：S0a/S1 的 `model-312.json`（渲染字段）、S2a/S2b（场景 BVH）、S3（网格 BVH）、
//! S4（纹理）、S6a（context 参数与输出布局）。渲染像素内核属 S7b，本片不实现。
//!
//! 主机用例默认执行；GPU 用例默认 ignore，显式执行：
//! cargo test --locked --features cuda-probe --test render_context_lifecycle -- --ignored --test-threads=1

use mjwarp_rs::diagnostics::{InputError, TransferError};
use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{LeafGeometry, Ray, compute_leaf_aabb};
use mjwarp_rs::render::context::owner::OwnerError;
use mjwarp_rs::render::context::{CamRes, MeshState, RenderFlagSpec};
use mjwarp_rs::render::texture::TextureRegistry;
use mjwarp_rs::render::{
    ContextParams, ImageOutput, LeafPose, OutputValues, RenderContext, ResolvedContext, SceneLeaf,
    SceneState, device_memory, release_stats, resolve_context,
};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

const MODEL: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");
/// 100 次 create/destroy 的显存增长阈值；实测值写进证据（见 S6b README）。
const MAX_CYCLE_GROWTH_BYTES: usize = 16 << 20;
const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("数组")
}

/// 把 S1 渲染字段夹具还原成 116 字段快照（同 `render_context_params` 用例）。
fn render_fields() -> RenderFields {
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| {
            let values = array(&arrays[spec.name]["values"]);
            match spec.kind {
                RenderFieldKind::Index => FieldData::Index(
                    values
                        .iter()
                        .map(|v| v.as_i64().expect("i32") as i32)
                        .collect(),
                ),
                RenderFieldKind::Num => {
                    FieldData::Num(values.iter().map(|v| v.as_f64().expect("f64")).collect())
                }
                RenderFieldKind::Flag => FieldData::Flag(
                    values
                        .iter()
                        .map(|v| v.as_u64().expect("u8") as u8)
                        .collect(),
                ),
                RenderFieldKind::Size => {
                    FieldData::Size(values.iter().map(|v| v.as_i64().expect("i64")).collect())
                }
                RenderFieldKind::Float => FieldData::Float(
                    values
                        .iter()
                        .map(|v| v.as_f64().expect("f32") as f32)
                        .collect(),
                ),
            }
        })
        .collect();
    RenderFields::new(data)
}

fn default_resolved(params: &ContextParams<'_>) -> ResolvedContext {
    resolve_context(&render_fields(), params).expect("context 解析")
}

fn session() -> TransferSession {
    TransferSession::new(0).expect("S6b GPU 验收需要 NVIDIA 驱动与 cuda-probe")
}

fn box_leaf() -> SceneLeaf {
    SceneLeaf {
        geom_type: 6,
        size: [1.0, 1.0, 1.0],
        group: 0,
        geom_id: 0,
    }
}

fn pose(x: f32) -> LeafPose {
    LeafPose {
        xpos: [x, 0.0, 0.0],
        xmat: IDENTITY,
    }
}

fn world_ray(x: f32, z: f32) -> Ray {
    Ray {
        origin: [x, 0.0, z],
        direction: [0.0, 0.0, -1.0],
    }
}

fn single_box_state() -> SceneState {
    SceneState::new(vec![box_leaf()], vec![pose(0.0)], 1).expect("单 box 场景")
}

fn identity_rotation() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

#[test]
fn scene_state_bounds_match_s2a_leaf_aabb() {
    let leaves = vec![
        box_leaf(),
        SceneLeaf {
            geom_type: 2,
            size: [0.5, 0.0, 0.0],
            group: 0,
            geom_id: 1,
        },
    ];
    let poses = vec![
        pose(0.0),
        LeafPose {
            xpos: [3.0, 1.0, 0.0],
            xmat: IDENTITY,
        },
    ];
    let state = SceneState::new(leaves.clone(), poses.clone(), 1).unwrap();
    let bounds = state.bounds().unwrap();
    assert_eq!(bounds.len(), 2);
    let box_geometry = LeafGeometry::from_geom_type(6, [1.0, 1.0, 1.0]).unwrap();
    assert_eq!(
        bounds[0],
        compute_leaf_aabb(&box_geometry, poses[0].xpos, identity_rotation()).unwrap()
    );
    let sphere_geometry = LeafGeometry::from_geom_type(2, [0.5, 0.0, 0.0]).unwrap();
    assert_eq!(
        bounds[1],
        compute_leaf_aabb(&sphere_geometry, poses[1].xpos, identity_rotation()).unwrap()
    );
    assert_eq!(state.ray_geometry()[0].xpos, poses[0].xpos);
}

#[test]
fn scene_state_rejects_invalid_inputs_before_allocation() {
    assert_eq!(
        SceneState::new(Vec::new(), Vec::new(), 1),
        Err(OwnerError::Bvh(
            mjwarp_rs::render::bvh::BvhError::EmptyScene
        ))
    );
    assert_eq!(
        SceneState::new(vec![box_leaf()], vec![pose(0.0)], 0),
        Err(OwnerError::EmptyWorlds)
    );
    assert_eq!(
        SceneState::new(vec![box_leaf()], Vec::new(), 1),
        Err(OwnerError::LengthMismatch {
            field: "scene_pose",
            expected: 1,
            actual: 0,
        })
    );
    let mut state = single_box_state();
    assert_eq!(
        state.set_poses(&[pose(0.0), pose(0.0)]),
        Err(OwnerError::LengthMismatch {
            field: "scene_pose",
            expected: 1,
            actual: 2,
        })
    );
    // hfield（geom_type=1）不在 S2a 支持面；在分配前拒绝。
    let unsupported = SceneState::new(
        vec![SceneLeaf {
            geom_type: 1,
            size: [1.0, 1.0, 1.0],
            group: 0,
            geom_id: 0,
        }],
        vec![pose(0.0)],
        1,
    )
    .unwrap();
    assert!(matches!(
        unsupported.bounds(),
        Err(OwnerError::LeafGeometry { leaf: 0, .. })
    ));
    // 非有限位姿在构建前拒绝。
    let mut nan_state = single_box_state();
    nan_state
        .set_poses(&[LeafPose {
            xpos: [f32::NAN, 0.0, 0.0],
            xmat: IDENTITY,
        }])
        .unwrap();
    assert!(matches!(
        nan_state.bounds(),
        Err(OwnerError::LeafBounds { leaf: 0, .. })
    ));
}

#[test]
#[ignore = "需要 GPU 驱动（创建传输会话）"]
fn world_count_mismatch_is_rejected_before_buffers() {
    let resolved = default_resolved(&ContextParams::default());
    let state = SceneState::new(vec![box_leaf()], vec![pose(0.0)], resolved.nworld + 1).unwrap();
    let matches = matches!(
        RenderContext::create(
            &session(),
            resolved,
            state,
            Vec::new(),
            &TextureRegistry::default()
        ),
        Err(mjwarp_rs::render::LifecycleError::Owner(
            OwnerError::WorldsMismatch { .. }
        ))
    );
    assert!(matches);
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn stale_bvh_until_explicit_refit_and_render_never_refits() {
    let session = session();
    let resolved = default_resolved(&ContextParams::default());
    let state = single_box_state();
    let mut context = RenderContext::create(
        &session,
        resolved,
        state,
        Vec::new(),
        &TextureRegistry::default(),
    )
    .expect("context 创建");

    // 位姿 A：BVH 与当前几何一致，命中。
    let poses_before = context.scene().poses().to_vec();
    let hits = context
        .render_rays(0, &[world_ray(0.0, 5.0)])
        .expect("render 查询");
    assert_eq!(hits[0].geom_id, 0);
    assert!((hits[0].distance - 4.0).abs() < 1.0e-5);

    // 移动位姿但不 refit：BVH 仍是旧边界（上游允许陈旧 BVH 渲染）。
    context.update_poses(&[pose(10.0)]).expect("更新位姿");
    assert_eq!(context.refits(), 0);
    let stale_b = context.render_rays(0, &[world_ray(10.0, 5.0)]).unwrap();
    assert_eq!(stale_b[0].geom_id, -1, "陈旧 BVH 不覆盖新位置");
    let stale_a = context.render_rays(0, &[world_ray(0.0, 5.0)]).unwrap();
    assert_eq!(stale_a[0].geom_id, -1, "几何已移动，A 不再命中");
    // render 不隐式 refit、不改当前位姿。
    assert_eq!(context.refits(), 0);
    assert_eq!(context.scene().poses(), &[pose(10.0)][..]);
    assert_ne!(context.scene().poses(), &poses_before[..]);

    // 显式 refit 后命中新位置。
    context.refit_bvh().expect("显式 refit");
    assert_eq!(context.refits(), 1);
    let fresh_b = context.render_rays(0, &[world_ray(10.0, 5.0)]).unwrap();
    assert_eq!(fresh_b[0].geom_id, 0);
    assert!((fresh_b[0].distance - 4.0).abs() < 1.0e-5);
    let fresh_a = context.render_rays(0, &[world_ray(0.0, 5.0)]).unwrap();
    assert_eq!(fresh_a[0].geom_id, -1);
    assert_eq!(context.refits(), 1, "render 不得隐式重拟合");
    println!(
        "stale_bvh: initial_distance={:.9} stale_b_geom={} stale_a_geom={} fresh_b_distance={:.9} \
         fresh_a_geom={} refits={}",
        hits[0].distance,
        stale_b[0].geom_id,
        stale_a[0].geom_id,
        fresh_b[0].distance,
        fresh_a[0].geom_id,
        context.refits()
    );
    context.destroy();
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn disabled_outputs_cannot_be_read_and_sizes_follow_layout() {
    let session = session();
    let params = ContextParams::default().with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(false)),
        Some(RenderFlagSpec::Bool(false)),
    );
    let resolved = default_resolved(&params);
    let state = single_box_state();
    let context = RenderContext::create(
        &session,
        resolved.clone(),
        state,
        Vec::new(),
        &TextureRegistry::default(),
    )
    .expect("context 创建");
    let outputs = context.outputs();
    assert!(outputs.is_enabled(ImageOutput::Rgb));
    assert!(!outputs.is_enabled(ImageOutput::Depth));
    assert!(!outputs.is_enabled(ImageOutput::Segmentation));

    let rgb_bytes = resolved.output.rgb_buffer_bytes(resolved.nworld).unwrap();
    assert_eq!(outputs.byte_len(ImageOutput::Rgb).unwrap(), rgb_bytes);
    for output in [ImageOutput::Depth, ImageOutput::Segmentation] {
        let disabled = matches!(
            outputs.byte_len(output),
            Err(OwnerError::Transfer(TransferError::Input(
                InputError::DisabledOutput
            )))
        );
        assert!(disabled, "{output:?} 字节数必须拒绝");
        let read_rejected = matches!(
            outputs.read(output),
            Err(OwnerError::Transfer(TransferError::Input(
                InputError::DisabledOutput
            )))
        );
        assert!(read_rejected, "{output:?} 读取必须拒绝");
    }
    let OutputValues::Rgb(values) = outputs.read(ImageOutput::Rgb).unwrap() else {
        panic!("rgb 读取类型");
    };
    assert_eq!(values.len(), resolved.output.rgb_pixels * resolved.nworld);
    assert_eq!(values.len() * 4, rgb_bytes);
    assert!(values.iter().all(|packed| *packed == 0), "新分配输出为零");
    println!(
        "disabled_outputs: rgb_pixels={} nworld={} rgb_bytes={rgb_bytes} depth_enabled={} seg_enabled={}",
        resolved.output.rgb_pixels,
        resolved.nworld,
        outputs.is_enabled(ImageOutput::Depth),
        outputs.is_enabled(ImageOutput::Segmentation)
    );
    context.destroy();
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn hundred_create_destroy_cycles_keep_memory_stable() {
    let session = session();
    let params = ContextParams::default().with_cam_res(Some(CamRes::Tuple([640, 480])));
    let resolved = default_resolved(&params);
    let textures = TextureRegistry::default();

    // 预热一次：排除首次 module 加载与驱动池首次扩张，再取基线。
    let warm = RenderContext::create(
        &session,
        resolved.clone(),
        single_box_state(),
        Vec::new(),
        &textures,
    )
    .expect("预热 context");
    warm.destroy();
    let baseline = device_memory().expect("设备显存查询").0;
    let (created_before, destroyed_before) = release_stats();

    let mut samples = Vec::with_capacity(10);
    let started = std::time::Instant::now();
    for cycle in 0..100 {
        let context = RenderContext::create(
            &session,
            resolved.clone(),
            single_box_state(),
            Vec::new(),
            &textures,
        )
        .unwrap_or_else(|error| panic!("第 {cycle} 次创建失败：{error}"));
        let hits = context
            .render_rays(0, &[world_ray(0.0, 5.0)])
            .expect("render");
        assert_eq!(hits[0].geom_id, 0);
        context.destroy();
        if (cycle + 1) % 10 == 0 {
            samples.push(device_memory().expect("设备显存查询").0);
        }
    }

    let (created_after, destroyed_after) = release_stats();
    assert_eq!(created_after - created_before, 100, "创建计数");
    assert_eq!(destroyed_after - destroyed_before, 100, "销毁计数");
    let free = device_memory().expect("设备显存查询").0;
    let growth = baseline.saturating_sub(free);
    println!(
        "cycle100: baseline={baseline} samples={samples:?} free={free} growth={growth} \
         threshold={MAX_CYCLE_GROWTH_BYTES} elapsed_ms={}",
        started.elapsed().as_millis()
    );
    assert!(
        growth <= MAX_CYCLE_GROWTH_BYTES,
        "显存增长 {growth} B 超过阈值 {MAX_CYCLE_GROWTH_BYTES} B；基线 {baseline} B、当前 {free} B、\
         每 10 次采样 {samples:?}"
    );
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn drop_releases_mesh_texture_and_output_resources() {
    let session = session();
    let fields = render_fields();
    let resolved = resolve_context(&fields, &ContextParams::default()).expect("context 解析");
    let textures = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
    assert!(!textures.is_empty(), "scene-a 模型含纹理");

    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mesh = MeshState {
        vertices: vertices.clone(),
        indices: vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3],
    };
    let build = |session: &TransferSession| {
        RenderContext::create(
            session,
            resolved.clone(),
            single_box_state(),
            vec![mesh.clone()],
            &textures,
        )
    };
    // 预热一次：首次加载 scene/mesh/sample 三个内核模块会让驱动保留一次性代码缓存
    // （实测约 32 MiB，记录在 S6b 证据里），第二次 create/destroy 才用于判断资源是否真的释放。
    let before_warmup = device_memory().expect("设备显存查询").0;
    build(&session).expect("预热 context").destroy();
    let baseline = device_memory().expect("设备显存查询").0;
    let mut context = build(&session).expect("context 创建");
    assert_eq!(context.mesh_count(), 1);
    let owned = context.textures().expect("use_textures=true 且注册表非空");
    assert_eq!(owned.len(), textures.len());
    let samples = owned.sample(0, &[[0.5, 0.5]]).expect("纹理采样");
    assert_eq!(samples.len(), 1);
    assert!(samples[0].iter().all(|value| value.is_finite()));

    // 网格查询在局部系；set_points 后必须显式 refit 才更新边界（S3 语义）。
    let hit = context
        .mesh_rays(0, &[world_ray(0.1, 5.0)], true)
        .expect("网格查询");
    assert_eq!(hit[0].geom_id, 0);
    let mut moved = vertices.clone();
    for vertex in &mut moved {
        vertex[2] += 10.0;
    }
    context.set_mesh_points(0, &moved).expect("更新顶点");
    assert_eq!(context.mesh_refits(0), Some(0));
    let stale = context.mesh_rays(0, &[world_ray(0.1, 5.0)], true).unwrap();
    assert_eq!(stale[0].geom_id, -1, "未 refit 的网格 BVH 边界陈旧");
    context.refit_mesh(0).expect("网格 refit");
    assert_eq!(context.mesh_refits(0), Some(1));
    let fresh = context.mesh_rays(0, &[world_ray(0.1, 15.0)], true).unwrap();
    assert_eq!(fresh[0].geom_id, 0);

    // 显式 destroy：输出缓冲、纹理对象、网格/场景资源全部释放。
    let mesh_refits = context.mesh_refits(0);
    let in_use = device_memory().expect("设备显存查询").0;
    assert!(
        baseline.saturating_sub(in_use) > 0,
        "context 应占用非零设备显存（基线 {baseline} B、占用时 {in_use} B）"
    );
    context.destroy();
    let after = device_memory().expect("设备显存查询").0;
    let retained = baseline.saturating_sub(after);
    assert!(
        retained <= MAX_CYCLE_GROWTH_BYTES,
        "destroy 后仍占用 {retained} B（基线 {baseline} B、占用时 {in_use} B、当前 {after} B）"
    );
    println!(
        "drop: warmup_retention={} baseline={baseline} in_use={in_use} after={after} retained={retained} \
         textures={} mesh_stale_geom={} mesh_fresh_geom={} mesh_refits={:?} tex_sample0={:?}",
        before_warmup.saturating_sub(baseline),
        textures.len(),
        stale[0].geom_id,
        fresh[0].geom_id,
        mesh_refits,
        samples[0]
    );
    // 释放后可再次创建（无 use-after-free：所有访问都经 owner 借用）。
    let again = RenderContext::create(
        &session,
        default_resolved(&ContextParams::default()),
        single_box_state(),
        Vec::new(),
        &TextureRegistry::default(),
    )
    .expect("销毁后重建");
    again.destroy();
}
