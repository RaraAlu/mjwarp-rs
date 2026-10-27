//! QA 独立复核（R10-S1）：多 world 状态与布局（宿主侧）。
//!
//! 临时复核 harness：复核时复制到 tests/ 执行，跑完删除；不进入提交。

use mjwarp_rs::render::camera::CameraPlan;
use mjwarp_rs::render::context::world::{
    CameraPose, WorldCameraPoses, WorldError, WorldLayout, WorldResolutions, WorldStates,
};
use mjwarp_rs::render::context::OutputLayout;
use mjwarp_rs::render::{CameraOutputViews, ImageOutput, WholeOutputData};

fn pixels(resolution: [u32; 2]) -> usize {
    resolution[0] as usize * resolution[1] as usize
}

fn plan(resolutions: &[[u32; 2]]) -> CameraPlan {
    let total: usize = resolutions.iter().copied().map(pixels).sum();
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

/// QA 自己按 plan 重算的单 world 通道布局（与 OutputLayout 独立实现）。
fn qa_layout(plan: &CameraPlan) -> ([usize; 3], Vec<Vec<i32>>) {
    let mut totals = [0usize; 3];
    let mut adrs = vec![Vec::new(); 3];
    for camera in 0..plan.resolutions.len() {
        for (channel, enabled) in [
            (0usize, plan.render_rgb[camera]),
            (1usize, plan.render_depth[camera]),
            (2usize, plan.render_seg[camera]),
        ] {
            if enabled {
                adrs[channel].push(totals[channel] as i32);
                totals[channel] += pixels(plan.resolutions[camera]);
            } else {
                adrs[channel].push(-1);
            }
        }
    }
    (totals, adrs)
}

#[test]
fn qa_layout_arithmetic_and_single_world_bitequal() {
    let mut p = plan(&[[5, 4], [3, 2], [1, 7]]);
    p.render_depth = vec![true, false, true];
    p.render_seg = vec![false, true, true];
    let single = OutputLayout::resolve(&p).expect("单 world 布局");
    let (qa_totals, qa_adr) = qa_layout(&p);
    assert_eq!(
        (single.rgb_pixels, single.depth_pixels, single.seg_pixels),
        (qa_totals[0], qa_totals[1], qa_totals[2])
    );
    assert_eq!(single.rgb_adr, qa_adr[0]);
    assert_eq!(single.depth_adr, qa_adr[1]);
    assert_eq!(single.seg_adr, qa_adr[2]);
    assert_eq!((qa_totals[0], qa_totals[1], qa_totals[2]), (33, 27, 13));

    let one = WorldLayout::resolve(&p, 1).expect("nworld=1");
    assert_eq!(one.layout(), &single);
    assert_eq!((one.worlds(), one.cameras()), (1, 3));
    for (channel, total) in [
        (ImageOutput::Rgb, qa_totals[0]),
        (ImageOutput::Depth, qa_totals[1]),
        (ImageOutput::Segmentation, qa_totals[2]),
    ] {
        assert_eq!(one.channel_pixels(channel), total);
        assert_eq!(one.buffer_elements(channel).unwrap(), total);
        assert_eq!(one.world_range(channel, 0).unwrap(), 0..total);
    }

    let rgb: Vec<u32> = (0..qa_totals[0] as u32)
        .map(|index| index.wrapping_mul(2_654_435_761))
        .collect();
    let views = CameraOutputViews::new(
        &p,
        &single,
        1,
        WholeOutputData {
            rgb: Some(&rgb),
            ..WholeOutputData::default()
        },
    )
    .expect("逐相机视图");
    for camera in 0..p.resolutions.len() {
        let start = single.rgb_adr[camera] as usize;
        let end = start + pixels(p.resolutions[camera]);
        assert_eq!(one.camera_range(ImageOutput::Rgb, 0, camera).unwrap(), start..end);
        assert_eq!(&rgb[start..end], views.rgb_packed(camera).expect("packed"));
    }

    let three = WorldLayout::resolve(&p, 3).expect("nworld=3");
    for (channel, total) in [
        (ImageOutput::Rgb, qa_totals[0]),
        (ImageOutput::Depth, qa_totals[1]),
        (ImageOutput::Segmentation, qa_totals[2]),
    ] {
        assert_eq!(three.buffer_elements(channel).unwrap(), 3 * total);
        for world in 0..3usize {
            assert_eq!(three.world_offset(channel, world).unwrap(), world * total);
            assert_eq!(
                three.world_range(channel, world).unwrap(),
                world * total..(world + 1) * total
            );
        }
    }
    // world 内逐相机区段 = world 偏移 + QA 重算 adr
    for world in 0..3usize {
        for camera in 0..2usize {
            let start = world * qa_totals[0] + qa_adr[0][camera] as usize;
            let end = start + pixels(p.resolutions[camera]);
            assert_eq!(three.camera_range(ImageOutput::Rgb, world, camera).unwrap(), start..end);
        }
    }
}

#[test]
fn qa_three_world_sentinel_isolation() {
    let p = plan(&[[4, 3], [2, 2]]);
    let three = WorldLayout::resolve(&p, 3).expect("3 world");
    let rgb: Vec<u32> = (0..48u32).map(|index| 0xA000_0000 | index).collect();
    let depth: Vec<f32> = (0..48u32).map(|index| index as f32 + 0.5).collect();
    let seg: Vec<[i32; 2]> = (0..48i32).map(|index| [index, 7]).collect();
    let mut seen: Vec<std::ops::Range<usize>> = Vec::new();
    for world in 0..3usize {
        let start = world * 16;
        let range = three.world_range(ImageOutput::Rgb, world).unwrap();
        assert_eq!(range, start..start + 16);
        assert_eq!(three.world_view(ImageOutput::Rgb, world, &rgb).unwrap(), &rgb[start..start + 16]);
        assert_eq!(three.world_view(ImageOutput::Depth, world, &depth).unwrap(), &depth[start..start + 16]);
        assert_eq!(three.world_view(ImageOutput::Segmentation, world, &seg).unwrap(), &seg[start..start + 16]);
        assert!(seen.iter().all(|prev| prev.end <= range.start || range.end <= prev.start));
        seen.push(range);
    }
    assert_eq!(three.camera_range(ImageOutput::Rgb, 0, 0).unwrap(), 0..12);
    assert_eq!(three.camera_range(ImageOutput::Rgb, 0, 1).unwrap(), 12..16);
    assert_eq!(three.camera_range(ImageOutput::Rgb, 2, 0).unwrap(), 32..44);
    assert_eq!(three.camera_range(ImageOutput::Rgb, 2, 1).unwrap(), 44..48);
    assert_eq!(
        three.world_view(ImageOutput::Rgb, 2, &rgb[..47]).unwrap_err(),
        WorldError::BufferRange { world: 2, end: 48, actual: 47 }
    );
    assert_eq!(
        three.world_view(ImageOutput::Rgb, 3, &rgb).unwrap_err(),
        WorldError::WorldOutOfRange { world: 3, worlds: 3 }
    );
}

#[test]
fn qa_error_surface_diagnostic_and_panic_free() {
    let p = plan(&[[4, 3], [2, 2]]);
    assert_eq!(WorldLayout::resolve(&p, 0).unwrap_err(), WorldError::WorldCountZero);
    assert_eq!(WorldStates::<u32>::new(0).unwrap_err(), WorldError::WorldCountZero);
    assert_eq!(WorldCameraPoses::new(0, 2, Vec::new()).unwrap_err(), WorldError::WorldCountZero);
    assert_eq!(WorldResolutions::new(&p, 0, Vec::new()).unwrap_err(), WorldError::WorldCountZero);
    assert_eq!(
        WorldLayout::resolve(&p, 2).unwrap().with_active(vec![true]).unwrap_err(),
        WorldError::ActiveMaskMismatch { expected: 2, actual: 1 }
    );
    assert_eq!(
        WorldStates::<u32>::with_active(2, vec![true]).unwrap_err(),
        WorldError::ActiveMaskMismatch { expected: 2, actual: 1 }
    );
    let masked = WorldLayout::resolve(&p, 2).unwrap().with_active(vec![true, false]).unwrap();
    assert!(!masked.is_active(1).unwrap());
    assert_eq!(masked.world_range(ImageOutput::Rgb, 1).unwrap_err(), WorldError::WorldInactive { world: 1 });
    assert_eq!(
        masked.world_view(ImageOutput::Rgb, 1, &[0u32; 32]).unwrap_err(),
        WorldError::WorldInactive { world: 1 }
    );
    assert_eq!(
        masked.world_range(ImageOutput::Rgb, 2).unwrap_err(),
        WorldError::WorldOutOfRange { world: 2, worlds: 2 }
    );
    let mut depth_off = p.clone();
    depth_off.render_depth = vec![true, false];
    let depth_layout = WorldLayout::resolve(&depth_off, 2).unwrap();
    assert_eq!(
        depth_layout.camera_range(ImageOutput::Depth, 0, 1).unwrap_err(),
        WorldError::ChannelDisabled { channel: ImageOutput::Depth, camera_index: 1 }
    );
    assert_eq!(
        depth_layout.camera_range(ImageOutput::Rgb, 0, 2).unwrap_err(),
        WorldError::CameraOutOfRange { camera: 2, cameras: 2 }
    );
    assert_eq!(
        WorldCameraPoses::new(2, 2, vec![pose(1.0); 3]).unwrap_err(),
        WorldError::CameraCountMismatch { expected: 4, actual: 3 }
    );
    let poses = WorldCameraPoses::new(2, 2, vec![pose(1.0), pose(2.0), pose(11.0), pose(12.0)]).unwrap();
    assert_eq!(poses.get(2, 0).unwrap_err(), WorldError::WorldOutOfRange { world: 2, worlds: 2 });
    assert_eq!(poses.get(0, 2).unwrap_err(), WorldError::CameraOutOfRange { camera: 2, cameras: 2 });
    assert_eq!(poses.world_slice(2).unwrap_err(), WorldError::WorldOutOfRange { world: 2, worlds: 2 });
    assert_eq!(
        WorldResolutions::new(&p, 2, vec![[4, 3]; 3]).unwrap_err(),
        WorldError::CameraCountMismatch { expected: 4, actual: 3 }
    );
    assert_eq!(
        WorldResolutions::new(&p, 2, vec![[4, 3], [2, 2], [4, 3], [3, 3]]).unwrap_err(),
        WorldError::ResolutionMismatch {
            world: 1,
            camera: 1,
            expected: [2, 2],
            actual: [3, 3]
        }
    );
    let mut states = WorldStates::new(2).unwrap();
    states.insert(0, 7u32).unwrap();
    assert_eq!(*states.state(0).unwrap(), 7);
    assert_eq!(states.state(1).unwrap_err(), WorldError::StateMissing { world: 1 });
    states.set_active(1, false).unwrap();
    assert_eq!(states.state(1).unwrap_err(), WorldError::WorldInactive { world: 1 });
    assert_eq!(states.insert(1, 9u32).unwrap_err(), WorldError::WorldInactive { world: 1 });
    assert_eq!(states.state(2).unwrap_err(), WorldError::WorldOutOfRange { world: 2, worlds: 2 });
    // 极大 nworld：resolve 先分配 active 掩码 -> 容量溢出 panic（QA 观察 F1，非本片阻断项）
    let tiny = plan(&[[1, 2]]);
    let caught = std::panic::catch_unwind(|| WorldLayout::resolve(&tiny, usize::MAX));
    assert!(caught.is_err(), "极大 nworld 未按预期 panic（行为已变化，需复审）");
    // 同一入口在可分配范围内的越界仍是可诊断错误
    let normal = WorldLayout::resolve(&tiny, 4).unwrap();
    assert_eq!(
        normal.world_range(ImageOutput::Rgb, 4).unwrap_err(),
        WorldError::WorldOutOfRange { world: 4, worlds: 4 }
    );
    // Layout(ContextError)：单 world 布局本身溢出
    // 手工构造（避免 QA helper 的 sum 自身溢出），让 OutputLayout::resolve 的 checked_add 溢出
    let massive = CameraPlan {
        cameras: vec![0, 1],
        resolutions: vec![[u32::MAX, u32::MAX], [u32::MAX, u32::MAX]],
        render_rgb: vec![true, true],
        render_depth: vec![true, true],
        render_seg: vec![true, true],
        total_rays: 0,
        rgb_pixels: 0,
        samples_per_pixel: 1,
        samples: 1,
        znear: 0.05,
        zfar: 100.0,
    };
    assert!(matches!(WorldLayout::resolve(&massive, 2).unwrap_err(), WorldError::Layout(_)));
    // 空相机表：ncam = 0 行为明确
    let empty_plan = plan(&[]);
    let empty = WorldLayout::resolve(&empty_plan, 3).unwrap();
    assert_eq!(empty.cameras(), 0);
    assert_eq!(empty.world_range(ImageOutput::Rgb, 2).unwrap(), 0..0);
    assert!(empty.world_view(ImageOutput::Rgb, 2, &[] as &[u32]).unwrap().is_empty());
    assert_eq!(
        empty.camera_range(ImageOutput::Rgb, 0, 0).unwrap_err(),
        WorldError::CameraOutOfRange { camera: 0, cameras: 0 }
    );
    // Display 非空且关键字段可诊断
    let errors = [
        WorldError::WorldCountZero,
        WorldError::WorldOutOfRange { world: 2, worlds: 2 },
        WorldError::WorldInactive { world: 1 },
        WorldError::StateMissing { world: 1 },
        WorldError::ActiveMaskMismatch { expected: 2, actual: 1 },
        WorldError::CameraCountMismatch { expected: 4, actual: 3 },
        WorldError::CameraOutOfRange { camera: 2, cameras: 2 },
        WorldError::ResolutionMismatch { world: 1, camera: 1, expected: [2, 2], actual: [3, 3] },
        WorldError::ChannelDisabled { channel: ImageOutput::Depth, camera_index: 1 },
        WorldError::BufferRange { world: 2, end: 48, actual: 47 },
        WorldError::CountOverflow { field: "world_offset" },
    ];
    for error in errors {
        assert!(!error.to_string().is_empty(), "{error:?} 文本为空");
    }
    assert!(WorldError::WorldOutOfRange { world: 2, worlds: 2 }.to_string().contains("world 2"));
    assert!(WorldError::CameraOutOfRange { camera: 2, cameras: 2 }.to_string().contains("相机下标 2"));
    assert!(WorldError::BufferRange { world: 2, end: 48, actual: 47 }.to_string().contains("48"));
}

#[test]
fn qa_poses_states_same_model_multi_world() {
    let poses = WorldCameraPoses::new(2, 2, vec![pose(1.0), pose(2.0), pose(11.0), pose(12.0)]).unwrap();
    let world0 = poses.world_slice(0).unwrap().to_vec();
    let world1 = poses.world_slice(1).unwrap().to_vec();
    assert_eq!(world0, vec![pose(1.0), pose(2.0)]);
    assert_eq!(world1, vec![pose(11.0), pose(12.0)]);
    assert_eq!(world0.len(), world1.len());
    for world in 0..2 {
        for camera in 0..2 {
            assert_eq!(poses.get(world, camera).unwrap(), poses.world_slice(world).unwrap()[camera]);
        }
    }
    let mut states = WorldStates::new(3).unwrap();
    states.insert(0, "w0").unwrap();
    states.insert(2, "w2").unwrap();
    assert_eq!(*states.state(2).unwrap(), "w2");
    assert_eq!(states.state(1).unwrap_err(), WorldError::StateMissing { world: 1 });
    states.set_active(1, false).unwrap();
    assert_eq!(states.insert(1, "w1").unwrap_err(), WorldError::WorldInactive { world: 1 });
    states.set_active(1, true).unwrap();
    states.insert(1, "w1").unwrap();
    assert_eq!(*states.state(1).unwrap(), "w1");
    assert_eq!(states.worlds(), 3);
    let p = plan(&[[4, 3], [2, 2]]);
    let resolutions =
        WorldResolutions::new(&p, 3, vec![[4, 3], [2, 2], [4, 3], [2, 2], [4, 3], [2, 2]]).unwrap();
    for world in 0..3 {
        for camera in 0..2 {
            assert_eq!(resolutions.get(world, camera).unwrap(), p.resolutions[camera]);
        }
    }
}

#[test]
fn qa_probe_empty_camera_plan_and_huge_worlds() {
    let empty = plan(&[]);
    // 空相机表 + nworld>1：WorldResolutions::new 内部 chunks(ncam=0) 的行为
    let caught = std::panic::catch_unwind(|| WorldResolutions::new(&empty, 3, Vec::new()).is_ok());
    println!("WorldResolutions(empty plan, worlds=3, []) panic caught: {}", caught.is_err());
    // 空相机表的布局本身行为明确
    let layout = WorldLayout::resolve(&empty, 3).expect("empty plan layout");
    assert_eq!(layout.cameras(), 0);
    assert_eq!(layout.world_range(ImageOutput::Rgb, 2).unwrap(), 0..0);
    // 极大 worlds 的分配型 panic（WorldStates / WorldLayout）
    let huge_states = std::panic::catch_unwind(|| WorldStates::<u32>::new(usize::MAX).is_ok());
    println!("WorldStates::new(usize::MAX) panic caught: {}", huge_states.is_err());
    let huge_layout = std::panic::catch_unwind(|| WorldLayout::resolve(&plan(&[[1, 1]]), usize::MAX).is_ok());
    println!("WorldLayout::resolve(usize::MAX) panic caught: {}", huge_layout.is_err());
}

