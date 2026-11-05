#![cfg(feature = "cuda-probe")]

#[path = "support/render_bvh/allowlist.rs"]
mod allowlist;
#[path = "support/render_bvh/multiframe.rs"]
mod multiframe;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::{
    CameraModel, RayBuilder, RayMode, SceneState, TextureRegistry, resolve_context,
};
use multiframe::{
    CHANGE_PIXELS_MIN, FRAME_DZ, FRAMES, FlexHostAssembly, FlexHostData, MegakernelRenderer,
    RenderContext, assert_depth, assert_rgb, assert_seg, frames,
};
use serde_json::Value;

// ----------------------------------------------------------------------------------
#[rustfmt::skip]
fn field_values(kind: RenderFieldKind, value: &Value) -> FieldData {
    let v = value.as_array().expect("数组");
    match kind {
        RenderFieldKind::Index => FieldData::Index(v.iter().map(|x| x.as_i64().expect("i32") as i32).collect()),
        RenderFieldKind::Num => FieldData::Num(v.iter().map(|x| x.as_f64().expect("f64")).collect()),
        RenderFieldKind::Flag => FieldData::Flag(v.iter().map(|x| x.as_u64().expect("u8") as u8).collect()),
        RenderFieldKind::Size => FieldData::Size(v.iter().map(|x| x.as_i64().expect("i64")).collect()),
        RenderFieldKind::Float => FieldData::Float(v.iter().map(|x| x.as_f64().expect("f32") as f32).collect()),
    }
}

#[rustfmt::skip]
fn render_fields() -> RenderFields {
    let model = multiframe::model();
    let arrays = &model["arrays"];
    RenderFields::new(RENDER_FIELDS.iter().map(|spec| field_values(spec.kind, &arrays[spec.name]["values"])).collect())
}

// CPU 自证（普通 cargo test；不碰 CUDA）
// ----------------------------------------------------------------------------------

#[test]
fn reference_integrity_15_files() {
    assert_eq!(multiframe::verify_reference_integrity(), 15);
}

#[test]
fn rigid_arrays_bit_equal_across_frames() {
    assert_eq!(multiframe::verify_rigid_arrays_bit_equal(), FRAMES - 1);
}

#[test]
fn displacement_matches_frame_dz() {
    let means = multiframe::verify_displacement();
    assert_eq!(means.len(), FRAMES);
    assert_eq!(means[0], 0.0, "frame-000 基准位移为 0");
    assert!((means[1] - FRAME_DZ).abs() <= 1e-6);
    assert!((means[2] - 2.0 * FRAME_DZ).abs() <= 1e-6);
}

#[test]
fn world_vertex_stays_fixed() {
    let frames = frames();
    let attached = multiframe::attached_vertices();
    let world: Vec<usize> = (0..6).filter(|i| !attached.contains(i)).collect();
    assert_eq!(world.len(), 1, "world 附着顶点 1 个");
    for frame in &frames[1..] {
        for vertex in &world {
            let base = 3 * vertex;
            for slot in 0..3 {
                assert_eq!(
                    frame.flexvert_xpos[base + slot].to_bits(),
                    frames[0].flexvert_xpos[base + slot].to_bits(),
                    "world 顶点不应移动"
                );
            }
        }
    }
}

/// ⑤ 跨帧可见性（cam2-only，spec §5）：000→001 变化像素各 ≥384（实体自算）。
#[test]
fn change_pixels_000_001_meet_threshold() {
    let (depth, seg) = multiframe::change_pixels_cam2(0);
    assert!(depth >= CHANGE_PIXELS_MIN, "depth={depth}");
    assert!(seg >= CHANGE_PIXELS_MIN, "seg={seg}");
}

/// ⑥ 跨帧可见性（cam2-only，spec §5）：001→002 变化像素各 ≥384。
#[test]
fn change_pixels_001_002_meet_threshold() {
    let (depth, seg) = multiframe::change_pixels_cam2(1);
    assert!(depth >= CHANGE_PIXELS_MIN, "depth={depth}");
    assert!(seg >= CHANGE_PIXELS_MIN, "seg={seg}");
}

#[test]
fn change_pixel_threshold_is_acceptance_384() {
    assert_eq!(CHANGE_PIXELS_MIN, 384, "验收判据 384");
    let counts = multiframe::verify_change_pixels();
    assert_eq!(counts.len(), 2);
    for (depth, seg) in counts {
        assert!(depth >= 384 && seg >= 384);
    }
}

#[test]
fn change_pixels_are_recomputed_deterministically() {
    for pair in 0..FRAMES - 1 {
        let cam2 = multiframe::change_pixels_cam2(pair);
        let all = multiframe::change_pixels_all_cameras(pair);
        println!("pair {pair}: cam2={cam2:?} all={all:?}");
        assert_eq!(cam2, multiframe::change_pixels_cam2(pair));
        assert_eq!(all, multiframe::change_pixels_all_cameras(pair));
    }
}

#[test]
fn provenance_spec_sha_and_bytes() {
    let capture = multiframe::verify_provenance();
    assert_eq!(
        capture["provenance"]["spec_sha256"].as_str(),
        Some(multiframe::SPEC_SHA)
    );
    assert_eq!(
        capture["provenance"]["spec_bytes"].as_u64(),
        Some(multiframe::SPEC_BYTES)
    );
}

#[test]
fn provenance_three_script_shas() {
    let capture = multiframe::verify_provenance();
    let scripts = capture["provenance"]["authoring_script_sha256"]
        .as_object()
        .expect("三脚本");
    for (name, sha) in multiframe::SCRIPT_SHA {
        assert_eq!(
            scripts.get(name).and_then(|v| v.as_str()),
            Some(sha),
            "{name}"
        );
    }
}

/// ⑪ provenance：refit_calls=6。
#[test]
fn provenance_refit_calls_is_6() {
    let capture = multiframe::verify_provenance();
    assert_eq!(capture["refit_calls"].as_u64(), Some(6));
    assert_eq!(
        capture["capture_checks"]["refit"]["actual_calls"].as_u64(),
        Some(6)
    );
}

/// ⑫ provenance：actual_sequence == [0,1,2,0,1,2]。
#[test]
fn provenance_refit_sequence() {
    let capture = multiframe::verify_provenance();
    let sequence: Vec<i64> = capture["capture_checks"]["refit"]["actual_sequence"]
        .as_array()
        .expect("seq")
        .iter()
        .map(|v| v.as_i64().expect("i64"))
        .collect();
    assert_eq!(sequence, [0, 1, 2, 0, 1, 2]);
}

#[test]
fn provenance_refit_policy() {
    let capture = multiframe::verify_provenance();
    assert_eq!(
        capture["provenance"]["refit_policy"].as_str(),
        Some("every-frame")
    );
    assert_eq!(capture["provenance"]["refit_bvh"].as_bool(), Some(true));
}

#[test]
fn allowlist_reference_seg_is_flexid0_seg9() {
    assert_eq!(
        multiframe::verify_allowlist_reference_seg(),
        [allowlist::ALLOWLIST_FLEXID, allowlist::ALLOWLIST_SEG_TYPE]
    );
}

#[test]
fn frame_shapes_match_capture() {
    let shapes = multiframe::verify_frame_shapes();
    assert_eq!(shapes[2], (320, 240));
}

#[test]
fn capture_selfcheck_flags() {
    let capture = multiframe::capture();
    assert_eq!(
        capture["two_captures_bitwise_identical"].as_bool(),
        Some(true)
    );
    assert_eq!(capture["legacy_reference_untouched"].as_bool(), Some(true));
    assert_eq!(
        capture["delta_selfcheck"]["overall"].as_str(),
        Some("pass-margin")
    );
    assert_eq!(capture["frames"].as_u64(), Some(3));
    assert_eq!(capture["frame_dz"].as_f64(), Some(0.10));
}

#[test]
fn output_sha_static_determinism() {
    let frames = frames();
    let frame = &frames[0];
    let seg: Vec<[i32; 2]> = frame.seg[2].clone();
    let first = multiframe::output_sha(
        &frame.rgb[2].packed,
        &frame.depth_bits[2]
            .iter()
            .map(|b| f32::from_bits(*b))
            .collect::<Vec<_>>(),
        &seg,
    );
    let second = multiframe::output_sha(
        &frame.rgb[2].packed,
        &frame.depth_bits[2]
            .iter()
            .map(|b| f32::from_bits(*b))
            .collect::<Vec<_>>(),
        &seg,
    );
    assert_eq!(first, second, "同输入 sha 必须一致");
    let mut other = seg.clone();
    other[0] = [7, 7];
    let changed = multiframe::output_sha(
        &frame.rgb[2].packed,
        &frame.depth_bits[2]
            .iter()
            .map(|b| f32::from_bits(*b))
            .collect::<Vec<_>>(),
        &other,
    );
    assert_ne!(first.2, changed.2, "不同 seg 的 sha 必须不同");
}

#[test]
fn context_params_match_capture_recipe() {
    let params = multiframe::context_params();
    multiframe::assert_params_match_capture(&params, &multiframe::capture()["parameters"]);
    let resolved = resolve_context(&render_fields(), &params).expect("context 解析");
    assert_eq!(resolved.nworld, 1);
    assert_eq!(resolved.enabled_geom_ids.len(), 4, "4 个 geom 全开");
}

#[test]
fn flex_inputs_shapes() {
    let frame = &frames()[0];
    let inputs = multiframe::flex_inputs(frame);
    assert_eq!(inputs.flexvert_xpos.len(), 6 * 3, "6 顶点");
    assert_eq!(
        inputs.edges.flex_edge,
        [0, 1, 1, 2, 2, 3, 0, 1],
        "1D 边 == flex_elem"
    );
    assert_eq!(inputs.fields.flex_dim, [1, 1], "两条 1D flex");
    assert_eq!(inputs.fields.flex_radius, [0.02f32, 0.04f32]);
    assert_eq!(inputs.positions.flex_vertnum.iter().sum::<i32>(), 6);
}

#[test]
fn scene_fixture_rigid_identical_across_frames() {
    let model = multiframe::model();
    let frames = frames();
    let signatures: Vec<Vec<u32>> = frames
        .iter()
        .map(|frame| {
            let (leaves, poses, _meshes) = multiframe::scene_fixture(frame, &model);
            let mut bits = Vec::new();
            for leaf in &leaves {
                bits.push(leaf.geom_type as u32);
                bits.push(leaf.geom_id as u32);
                bits.extend(leaf.size.iter().map(|v| v.to_bits()));
            }
            for pose in &poses {
                bits.extend(pose.xpos.iter().map(|v| v.to_bits()));
                bits.extend(pose.xmat.iter().map(|v| v.to_bits()));
            }
            bits
        })
        .collect();
    assert_eq!(
        signatures[0], signatures[1],
        "frame-001 场景与 frame-000 一致"
    );
    assert_eq!(
        signatures[0], signatures[2],
        "frame-002 场景与 frame-000 一致"
    );
}

#[test]
fn camera_poses_d5_bit_provenance() {
    let poses = multiframe::camera_poses();
    assert_eq!(poses.len(), 3);
    for pose in &poses {
        assert!(pose.xpos.iter().all(|v| v.is_finite()));
        assert!(pose.xmat.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn flex_inputs_are_frame_specific() {
    let frames = frames();
    let first = multiframe::flex_inputs(&frames[0]).flexvert_xpos;
    let second = multiframe::flex_inputs(&frames[1]).flexvert_xpos;
    assert_eq!(first, frames[0].flexvert_xpos);
    assert_ne!(first, second, "帧间 flexvert 必须不同");
}

// ----------------------------------------------------------------------------------
// GPU 用例（验证窗执行；普通 cargo test 忽略）
// ----------------------------------------------------------------------------------

/// GPU 逐帧 + 跨帧 + 选项 A：3 帧参考逐相机判据、逐帧两跑 sha、三 context 刚性一致 + `refits()==0`。
#[test]
#[ignore = "需要 GPU + NVRTC（验证窗执行）"]
fn multiframe_renders_match_captured_reference() {
    let reference = frames();
    let model = multiframe::model();
    let fields = render_fields();
    let session = s7b1::session();
    let params = multiframe::context_params();
    multiframe::assert_params_match_capture(&params, &multiframe::capture()["parameters"]);
    let camera_model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = multiframe::camera_poses();
    let rays = RayBuilder::new(&session)
        .expect("build_rays 编译")
        .build(
            &camera_model,
            &resolve_context(&fields, &params).expect("解析").plan,
            RayMode::Precomputed,
        )
        .expect("S5 预计算射线");
    let renderer = MegakernelRenderer::new(&session).expect("megakernel 编译");
    // 选项 A（口径按 Chief 2026-11-05 裁决）：逐帧 state 刚性位姿建 3 个 context + 断言三者刚性签名一致 +
    // 同一 context 渲染对照 + `refits()==0`；不补 body_pos 偏移步骤（产品路径不消费 body_pos）。
    // spec §2 的实现映射：模型 body_pos 偏移对 flex 渲染惰性（产品每次渲染用通道 positions 重建合并 BVH）。
    let mut contexts = Vec::new();
    let mut signatures = Vec::new();
    for frame in &reference {
        let (leaves, leaf_poses, meshes) = multiframe::scene_fixture(frame, &model);
        let mut bits = Vec::new();
        for leaf in &leaves {
            bits.push(leaf.geom_type as u32);
            bits.push(leaf.geom_id as u32);
            bits.extend(leaf.size.iter().map(|v| v.to_bits()));
        }
        for pose in &leaf_poses {
            bits.extend(pose.xpos.iter().map(|v| v.to_bits()));
            bits.extend(pose.xmat.iter().map(|v| v.to_bits()));
        }
        signatures.push(bits);
        let resolved = resolve_context(&fields, &params).expect("解析");
        let textures = TextureRegistry::from_fields(&fields, params.use_textures).expect("纹理");
        contexts.push(
            RenderContext::create(
                &session,
                resolved,
                SceneState::new(leaves, leaf_poses, 1).expect("场景"),
                meshes,
                &textures,
            )
            .expect("context 创建"),
        );
    }
    assert_eq!(
        signatures[0], signatures[1],
        "选项 A：frame-001 刚性场景与 frame-000 完全一致"
    );
    assert_eq!(
        signatures[0], signatures[2],
        "选项 A：frame-002 刚性场景与 frame-000 完全一致"
    );
    // 逐帧：一次 assembly + render_with（各自 context），整块多相机缓冲 sha 相同；frame-000 allowlist、001/002 严格。
    let mut shas = Vec::new();
    for (index, frame) in reference.iter().enumerate() {
        let inputs = multiframe::flex_inputs(frame);
        let assembly = FlexHostAssembly::build(
            1,
            &inputs.positions,
            &inputs.edges,
            &inputs.fields,
            FlexHostData {
                flexvert_xpos: &inputs.flexvert_xpos,
                flex_bvh_ids: &[u64::MAX; 2],
                flex_group_root: &[-1i32; 2],
            },
        )
        .expect("组装")
        .expect("有 flex");
        let context = &contexts[index];
        assert_eq!(
            context.refits(),
            0,
            "产品渲染上下文不应 refit（每渲染重建合并 BVH 的架构事实）"
        );
        let mut first: Option<(String, String, String)> = None;
        for run in 0..2 {
            assembly
                .render_with(
                    &renderer,
                    context,
                    &fields,
                    &camera_model,
                    &poses,
                    &rays,
                    None,
                )
                .expect("render_with");
            let out = (
                s7b2::read_rgb(context),
                s7b1::read_depth(context),
                s7b1::read_seg(context),
            );
            let sha = multiframe::output_sha(&out.0, &out.1, &out.2);
            if run == 0 {
                assert_rgb(frame, &out.0, context.resolved());
                assert_depth(frame, &out.1, context.resolved());
                assert_seg(frame, &out.2, context.resolved());
                first = Some(sha.clone());
            } else {
                assert_eq!(
                    first.as_ref(),
                    Some(&sha),
                    "frame-{index:03} 两跑 sha 必须相同"
                );
            }
        }
        shas.push(first.expect("首跑 sha"));
        println!(
            "r06-multiframe frame-{index:03}: refits={} two-run sha equal",
            context.refits()
        );
    }
    // 跨帧可见性：实体自算（不读 delta_selfcheck）；验收口径仅 cam2，全相机合计只诊断打印。
    let pair_0 = multiframe::change_pixels_cam2(0);
    let pair_1 = multiframe::change_pixels_cam2(1);
    for (pair, (depth, seg)) in [pair_0, pair_1].iter().enumerate() {
        let (all_d, all_s) = multiframe::change_pixels_all_cameras(pair);
        println!("pair {pair} cam2={depth}/{seg}; all_cameras={all_d}/{all_s}");
        assert!(
            depth >= &CHANGE_PIXELS_MIN && seg >= &CHANGE_PIXELS_MIN,
            "pair {pair} cam2 变化像素 depth={depth} seg={seg} 低于 384"
        );
    }
    // 「同一 context」对照：frame-001/002 用 contexts[0] 渲染，结果应与其自身 context 逐位相同。
    for index in 1..FRAMES {
        let inputs = multiframe::flex_inputs(&reference[index]);
        let assembly = FlexHostAssembly::build(
            1,
            &inputs.positions,
            &inputs.edges,
            &inputs.fields,
            FlexHostData {
                flexvert_xpos: &inputs.flexvert_xpos,
                flex_bvh_ids: &[u64::MAX; 2],
                flex_group_root: &[-1i32; 2],
            },
        )
        .expect("组装")
        .expect("有 flex");
        assembly
            .render_with(
                &renderer,
                &contexts[0],
                &fields,
                &camera_model,
                &poses,
                &rays,
                None,
            )
            .expect("render_with");
        let out = (
            s7b2::read_rgb(&contexts[0]),
            s7b1::read_depth(&contexts[0]),
            s7b1::read_seg(&contexts[0]),
        );
        assert_eq!(
            multiframe::output_sha(&out.0, &out.1, &out.2),
            shas[index],
            "frame-{index:03} 同 context 渲染与自 context 逐位相同"
        );
    }
    println!("r06-multiframe PASS: 3 frames, per-frame + cross-frame + option-A 3-context audit");
}
