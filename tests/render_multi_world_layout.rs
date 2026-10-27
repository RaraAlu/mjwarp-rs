//! 渲染/BVH R10-S1 验收：多 world 状态与布局（宿主侧，无 GPU、无窗口）。
//!
//! 验收点：每 world 输出区段互不重叠、不越界读邻 world；`nworld = 1` 与既有单 world
//! `OutputLayout`/`CameraOutputViews` 行为位级一致；world 越界/未激活/状态缺失/相机数或
//! 分辨率与 `nworld` 不匹配/`nworld = 0` 在读取前可诊断拒绝；`nworld × ncam` 相机位姿与
//! per-world 状态容器为「同模型多 state」语义。

use mjwarp_rs::render::camera::CameraPlan;
use mjwarp_rs::render::context::world::{
    CameraPose, WorldCameraPoses, WorldError, WorldLayout, WorldResolutions, WorldStates,
};
use mjwarp_rs::render::{CameraOutputViews, ImageOutput, WholeOutputData};

fn pixels(resolution: [u32; 2]) -> usize {
    resolution[0] as usize * resolution[1] as usize
}

fn plan(resolutions: &[[u32; 2]]) -> CameraPlan {
    let total = resolutions.iter().copied().map(pixels).sum();
    CameraPlan {
        cameras: (0..resolutions.len()).collect(),
        resolutions: resolutions.to_vec(),
        render_rgb: vec![true; resolutions.len()],
        render_depth: vec![true; resolutions.len()],
        render_seg: vec![true; resolutions.len()],
        total_rays: total,
        rgb_pixels: total,
        samples_per_pixel: 1,
        samples: 1,
        znear: 0.05,
        zfar: 100.0,
    }
}

fn pose(seed: f32) -> CameraPose {
    CameraPose {
        xpos: [seed, seed + 1.0, seed + 2.0],
        xmat: [seed, 0.0, 0.0, 0.0, seed + 4.0, 0.0, 0.0, 0.0, seed + 8.0],
    }
}

#[test]
fn world_views_are_disjoint_and_do_not_read_neighbours() {
    // 2 个激活相机：4×3（12 像素）+ 2×2（4 像素）= 16 像素/世界；3 个 world。
    let plan = plan(&[[4, 3], [2, 2]]);
    let worlds = 3;
    let layout = WorldLayout::resolve(&plan, worlds).expect("多 world 布局");
    assert_eq!((layout.worlds(), layout.cameras()), (worlds, 2));
    assert_eq!(layout.channel_pixels(ImageOutput::Rgb), 16);
    assert_eq!(
        layout.buffer_elements(ImageOutput::Rgb).unwrap(),
        worlds * 16
    );
    let rgb: Vec<u32> = (0..48u32).map(|index| 0xA000_0000 | index).collect();
    let depth: Vec<f32> = (0..48u32).map(|index| index as f32 + 0.5).collect();
    let seg: Vec<[i32; 2]> = (0..48i32).map(|index| [index, 7]).collect();
    let mut seen: Vec<std::ops::Range<usize>> = Vec::new();
    for world in 0..worlds {
        let start = world * 16;
        assert_eq!(layout.world_offset(ImageOutput::Rgb, world).unwrap(), start);
        assert_eq!(
            layout.world_range(ImageOutput::Rgb, world).unwrap(),
            start..start + 16
        );
        // 每个 world 只看到自己的 16 个元素（哨兵值域不重叠）。
        assert_eq!(
            layout.world_view(ImageOutput::Rgb, world, &rgb).unwrap(),
            &rgb[start..start + 16]
        );
        assert_eq!(
            layout
                .world_view(ImageOutput::Depth, world, &depth)
                .unwrap(),
            &depth[start..start + 16]
        );
        assert_eq!(
            layout
                .world_view(ImageOutput::Segmentation, world, &seg)
                .unwrap(),
            &seg[start..start + 16]
        );
        let range = layout.world_range(ImageOutput::Rgb, world).unwrap();
        assert!(
            seen.iter()
                .all(|previous| previous.end <= range.start || range.end <= previous.start),
            "world {world} 区段 {range:?} 与之前的 {seen:?} 重叠"
        );
        seen.push(range);
    }
    // 相机区段在世界偏移之后，且 world 间不重叠。
    assert_eq!(layout.camera_range(ImageOutput::Rgb, 0, 0).unwrap(), 0..12);
    assert_eq!(layout.camera_range(ImageOutput::Rgb, 0, 1).unwrap(), 12..16);
    assert_eq!(layout.camera_range(ImageOutput::Rgb, 2, 0).unwrap(), 32..44);
    assert_eq!(layout.camera_range(ImageOutput::Rgb, 2, 1).unwrap(), 44..48);
    // 短缓冲：区段末端越界 → 可诊断错误（不 panic）。
    assert_eq!(
        layout
            .world_view(ImageOutput::Rgb, 2, &rgb[..47])
            .unwrap_err(),
        WorldError::BufferRange {
            world: 2,
            end: 48,
            actual: 47
        }
    );
    assert_eq!(
        layout.world_view(ImageOutput::Rgb, 3, &rgb).unwrap_err(),
        WorldError::WorldOutOfRange {
            world: 3,
            worlds: 3
        }
    );
}

#[test]
fn single_world_path_matches_existing_layout_and_camera_views() {
    let plan = plan(&[[8, 6], [4, 3]]);
    let single = mjwarp_rs::render::context::OutputLayout::resolve(&plan).expect("单 world 布局");
    let world_layout = WorldLayout::resolve(&plan, 1).expect("nworld = 1 布局");
    // nworld = 1：world 布局与既有 OutputLayout 完全一致，区段就是整缓冲。
    assert_eq!(world_layout.layout(), &single);
    assert_eq!(
        world_layout.world_range(ImageOutput::Rgb, 0).unwrap(),
        0..single.rgb_pixels
    );
    assert_eq!(
        world_layout.buffer_elements(ImageOutput::Rgb).unwrap(),
        single.rgb_pixels
    );
    assert_eq!(
        world_layout
            .buffer_elements(ImageOutput::Segmentation)
            .unwrap(),
        single.seg_pixels
    );
    assert_eq!(world_layout.channel_pixels(ImageOutput::Rgb), 60);
    let rgb: Vec<u32> = (0..single.rgb_pixels as u32)
        .map(|index| index.wrapping_mul(2_654_435_761))
        .collect();
    assert_eq!(
        world_layout.world_view(ImageOutput::Rgb, 0, &rgb).unwrap(),
        rgb.as_slice()
    );
    // 与既有 S1 逐相机视图逐位一致（同 plan/同布局/同缓冲）。
    let views = CameraOutputViews::new(
        &plan,
        &single,
        1,
        WholeOutputData {
            rgb: Some(&rgb),
            ..WholeOutputData::default()
        },
    )
    .expect("逐相机视图");
    for camera in 0..plan.resolutions.len() {
        let expected = views.rgb_packed(camera).expect("packed 区段");
        let range = world_layout
            .camera_range(ImageOutput::Rgb, 0, camera)
            .expect("相机区段");
        assert_eq!(
            &rgb[range], expected,
            "world 0 相机 {camera} 与既有单 world 视图逐位一致"
        );
    }
}

#[test]
fn error_surface_is_diagnostic_without_panicking() {
    let plan = plan(&[[4, 3], [2, 2]]);
    // nworld = 0
    assert_eq!(
        WorldLayout::resolve(&plan, 0).unwrap_err(),
        WorldError::WorldCountZero
    );
    // 激活掩码长度不符 / 未激活 world
    assert_eq!(
        WorldLayout::resolve(&plan, 2)
            .unwrap()
            .with_active(vec![true])
            .unwrap_err(),
        WorldError::ActiveMaskMismatch {
            expected: 2,
            actual: 1
        }
    );
    let masked = WorldLayout::resolve(&plan, 2)
        .unwrap()
        .with_active(vec![true, false])
        .expect("掩码");
    assert!(masked.is_active(0).unwrap());
    assert!(!masked.is_active(1).unwrap());
    assert_eq!(
        masked.world_range(ImageOutput::Rgb, 1).unwrap_err(),
        WorldError::WorldInactive { world: 1 }
    );
    assert_eq!(
        masked
            .world_view(ImageOutput::Rgb, 1, &[0u32; 32])
            .unwrap_err(),
        WorldError::WorldInactive { world: 1 }
    );
    assert_eq!(
        masked.world_range(ImageOutput::Rgb, 2).unwrap_err(),
        WorldError::WorldOutOfRange {
            world: 2,
            worlds: 2
        }
    );
    // 通道未启用（depth 在相机 1 关闭）→ 相机区段可诊断拒绝。
    let mut depth_off = plan.clone();
    depth_off.render_depth = vec![true, false];
    let depth_layout = WorldLayout::resolve(&depth_off, 2).expect("depth 关闭布局");
    assert_eq!(
        depth_layout
            .camera_range(ImageOutput::Depth, 0, 1)
            .unwrap_err(),
        WorldError::ChannelDisabled {
            channel: ImageOutput::Depth,
            camera_index: 1
        }
    );
    // 相机位姿条目数不等于 nworld×ncam。
    assert_eq!(
        WorldCameraPoses::new(2, 2, vec![pose(1.0); 3]).unwrap_err(),
        WorldError::CameraCountMismatch {
            expected: 4,
            actual: 3
        }
    );
    assert_eq!(
        WorldCameraPoses::new(0, 2, Vec::new()).unwrap_err(),
        WorldError::WorldCountZero
    );
    let poses = WorldCameraPoses::new(2, 2, vec![pose(1.0); 4]).expect("位姿");
    assert_eq!(
        poses.get(2, 0).unwrap_err(),
        WorldError::WorldOutOfRange {
            world: 2,
            worlds: 2
        }
    );
    assert_eq!(
        poses.get(0, 2).unwrap_err(),
        WorldError::CameraOutOfRange {
            camera: 2,
            cameras: 2
        }
    );
    // 分辨率表与 plan 不一致。
    assert_eq!(
        WorldResolutions::new(&plan, 2, vec![[4, 3], [2, 2], [4, 3], [3, 3]]).unwrap_err(),
        WorldError::ResolutionMismatch {
            world: 1,
            camera: 1,
            expected: [2, 2],
            actual: [3, 3]
        }
    );
    assert_eq!(
        WorldResolutions::new(&plan, 2, vec![[4, 3]; 3]).unwrap_err(),
        WorldError::CameraCountMismatch {
            expected: 4,
            actual: 3
        }
    );
    // 状态容器：nworld=0 / 掩码长度 / 缺失 / 未激活 / 越界。
    assert_eq!(
        WorldStates::<u32>::new(0).unwrap_err(),
        WorldError::WorldCountZero
    );
    assert_eq!(
        WorldStates::<u32>::with_active(2, vec![true]).unwrap_err(),
        WorldError::ActiveMaskMismatch {
            expected: 2,
            actual: 1
        }
    );
    let mut states = WorldStates::new(2).expect("状态容器");
    states.insert(0, 7u32).expect("写入 world 0");
    assert_eq!(*states.state(0).unwrap(), 7);
    assert_eq!(
        states.state(1).unwrap_err(),
        WorldError::StateMissing { world: 1 }
    );
    states.set_active(1, false).expect("停用 world 1");
    assert_eq!(
        states.state(1).unwrap_err(),
        WorldError::WorldInactive { world: 1 }
    );
    assert_eq!(
        states.insert(1, 9u32).unwrap_err(),
        WorldError::WorldInactive { world: 1 }
    );
    assert_eq!(
        states.state(2).unwrap_err(),
        WorldError::WorldOutOfRange {
            world: 2,
            worlds: 2
        }
    );
    // 错误文本可诊断且非空。
    for error in [
        WorldError::WorldCountZero,
        WorldError::WorldOutOfRange {
            world: 2,
            worlds: 2,
        },
        WorldError::WorldInactive { world: 1 },
        WorldError::StateMissing { world: 1 },
        WorldError::ActiveMaskMismatch {
            expected: 2,
            actual: 1,
        },
        WorldError::CameraCountMismatch {
            expected: 4,
            actual: 3,
        },
        WorldError::CameraOutOfRange {
            camera: 2,
            cameras: 2,
        },
        WorldError::ResolutionMismatch {
            world: 1,
            camera: 1,
            expected: [2, 2],
            actual: [3, 3],
        },
        WorldError::ChannelDisabled {
            channel: ImageOutput::Depth,
            camera_index: 1,
        },
        WorldError::BufferRange {
            world: 2,
            end: 48,
            actual: 47,
        },
        WorldError::CountOverflow {
            field: "world_offset",
        },
    ] {
        assert!(!error.to_string().is_empty(), "{error:?} 文本为空");
    }
}

#[test]
fn camera_poses_are_nworld_by_ncam_same_model_multi_state() {
    let plan = plan(&[[4, 3], [2, 2]]);
    // 双 world = 同模型多 state：同相机下标、不同位姿状态。
    let poses = WorldCameraPoses::new(2, 2, vec![pose(1.0), pose(2.0), pose(11.0), pose(12.0)])
        .expect("nworld × ncam 位姿");
    assert_eq!(poses.world_slice(1).expect("world 1").len(), 2);
    let world0 = poses.world_slice(0).expect("world 0").to_vec();
    let world1 = poses.world_slice(1).expect("world 1").to_vec();
    assert_eq!(world0, vec![pose(1.0), pose(2.0)]);
    assert_eq!(world1, vec![pose(11.0), pose(12.0)]);
    // 同模型：world 间的相机数/标识一致，仅位姿 state 不同。
    assert_eq!(world0.len(), world1.len());
    assert_eq!(plan.cameras.len(), 2);
    for world in 0..2 {
        for camera in 0..2 {
            let slice = poses.world_slice(world).expect("world");
            assert_eq!(poses.get(world, camera).expect("位姿"), slice[camera]);
        }
    }
    // 单 world 的映射就是 plan 的相机顺序。
    assert_eq!(poses.world_slice(0).unwrap()[0], pose(1.0));
    assert_eq!(poses.get(1, 0).unwrap(), pose(11.0));
    assert_eq!(
        WorldCameraPoses::new(0, 2, Vec::new()).unwrap_err(),
        WorldError::WorldCountZero
    );
}

#[test]
fn resolutions_replicate_plan_and_reject_mismatch() {
    let plan = plan(&[[4, 3], [2, 2]]);
    // 同模型多 state：分辨率表逐 world 复制，且必须与 plan 一致。
    let replicated = WorldResolutions::new(
        &plan,
        3,
        vec![[4, 3], [2, 2], [4, 3], [2, 2], [4, 3], [2, 2]],
    )
    .expect("复制分辨率");
    for world in 0..3 {
        for camera in 0..2 {
            assert_eq!(
                replicated.get(world, camera).unwrap(),
                plan.resolutions[camera]
            );
        }
    }
    assert_eq!(replicated.get(2, 1).unwrap(), [2, 2]);
    assert_eq!(
        WorldResolutions::new(&plan, 0, Vec::new()).unwrap_err(),
        WorldError::WorldCountZero
    );
}
