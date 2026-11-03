//! 逐 world 渲染实现 S1（方案 L）的宿主与 GPU 用例。
//! CPU 段（默认执行）：容器/守卫/leaf 映射与参考解码自查，不发 GPU。GPU 段（`--ignored`，验证窗）：
//! 对 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/` 逐 world 比对 rgb/depth/seg 参考
//! （sha256 与 capture.json 绑定），并验 world 隔离与两次运行 SHA 相同；不新增/修改夹具。
#![cfg(feature = "cuda-probe")]

use mjwarp_rs::render::context::RenderFlagSpec;
use mjwarp_rs::render::kernel::output::MAX_DEPTH_ULP;
use mjwarp_rs::render::kernel::{MegakernelRenderer, reject_multi_sample, render_per_world};
use mjwarp_rs::render::{
    CameraActive, CameraModel, ContextParams, LeafPose, RayBuilder, RayMode, ScenePose, SceneState,
    WorldError, WorldScenePoses, resolve_context,
};
use mjwarp_rs::runtime::launcher::LauncherError;
use sha2::{Digest, Sha256};

#[path = "support/render_bvh/r10.rs"]
mod r10;
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;

const PERWORLD_DIR: &str = "../fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/";
const RGB_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/rgb.npz");
const DEPTH_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/depth.npz");
const SEG_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/seg.npz");
const STATE_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/state.npz");
const RGB_SHA256: &str = "305f2bfdffc4e4efe406c4b8461a67ea115df73bf16d4303496fd3aa4eee56b7";
const DEPTH_SHA256: &str = "cc065d0499f957d14d071fde295152c6e5cd0ca7989adb0cb7065226179903c5";
const SEG_SHA256: &str = "b46b637097f9d53cf7d30a97d40c7511f5d266668c6dea45952826490e15a11a";
const STATE_SHA256: &str = "36c56b4c26ac4b36602d8fec31bb3a56b33031513b1e8660ded372c777b6e426";
const WORLDS: usize = 3;
const LEAVES: usize = 7;
const HEIGHT: usize = 240;
const WIDTH: usize = 320;
const PIXELS: usize = HEIGHT * WIDTH;
const GEOM_GROUPS: [i32; 4] = [0, 1, 2, 3];
const CAM_ACTIVE: [bool; 2] = [true, false];

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

// ---------------------------------------------------------------------------
// 最小 npz/npy 解析（ZIP_STORED + npy v1/v2 头；不引第三方依赖、不新增夹具）。
// ---------------------------------------------------------------------------

fn npz_members(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut members = Vec::new();
    let mut offset = 0usize;
    while offset + 4 <= bytes.len() && &bytes[offset..offset + 4] == b"PK\x03\x04" {
        let compressed = u32::from_le_bytes(bytes[offset + 18..offset + 22].try_into().unwrap());
        let name_len = u16::from_le_bytes([bytes[offset + 26], bytes[offset + 27]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[offset + 28], bytes[offset + 29]]) as usize;
        let method = u16::from_le_bytes([bytes[offset + 8], bytes[offset + 9]]);
        assert_eq!(method, 0, "npz 仅支持 ZIP_STORED");
        let name_start = offset + 30;
        let name = String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).to_string();
        let data_start = name_start + name_len + extra_len;
        let data_end = data_start + compressed as usize;
        assert!(data_end <= bytes.len(), "npz 成员 {name} 越界");
        members.push((name, bytes[data_start..data_end].to_vec()));
        offset = data_end;
    }
    assert!(!members.is_empty(), "npz 无成员");
    members
}

/// 解析 npy 头：`(dtype, shape, raw data)`（只支持 C 顺序小端）。
fn npy(bytes: &[u8]) -> (String, Vec<usize>, Vec<u8>) {
    assert_eq!(&bytes[..6], b"\x93NUMPY", "npy magic");
    let (header_len, header_start) = match bytes[6] {
        1 => (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10usize),
        2 | 3 => (
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
            12usize,
        ),
        version => panic!("npy 版本 {version} 未支持"),
    };
    let header_end = header_start + header_len;
    let header = std::str::from_utf8(&bytes[header_start..header_end]).expect("npy header");
    assert!(header.contains("fortran_order': False"), "npy 必须 C 顺序");
    let descr_start = header.find("descr':").expect("npy descr") + "descr':".len();
    let rest = &header[descr_start..];
    let quote = rest.find('\'').expect("npy descr 开引号") + 1;
    let tail = &rest[quote..];
    let descr = tail[..tail.find('\'').expect("npy descr 闭引号")].to_string();
    let shape_start = header.find("shape':").expect("npy shape") + "shape':".len();
    let shape_rest = &header[shape_start..];
    let open = shape_rest.find('(').expect("npy shape 左括号");
    let close = shape_rest.find(')').expect("npy shape 右括号");
    let shape: Vec<usize> = shape_rest[open + 1..close]
        .split(',')
        .filter_map(|item| {
            let item = item.trim();
            (!item.is_empty()).then(|| item.parse::<usize>().expect("npy shape 维度"))
        })
        .collect();
    (descr, shape, bytes[header_end..].to_vec())
}

fn member(members: &[(String, Vec<u8>)], name: &str) -> (String, Vec<usize>, Vec<u8>) {
    let (_, bytes) = members
        .iter()
        .find(|(member_name, _)| member_name == name)
        .unwrap_or_else(|| panic!("npz 缺少成员 {name}"));
    npy(bytes)
}

macro_rules! scalar_member {
    ($name:ident, $ty:ty, $descr:literal, $width:literal) => {
        fn $name(members: &[(String, Vec<u8>)], name: &str) -> (Vec<usize>, Vec<$ty>) {
            let (descr, shape, data) = member(members, name);
            assert_eq!(descr, $descr, "{name} dtype");
            let (chunks, rest) = data.as_chunks::<$width>();
            assert!(rest.is_empty(), "{name} 数据长度");
            (
                shape,
                chunks.iter().map(|c| <$ty>::from_le_bytes(*c)).collect(),
            )
        }
    };
}
scalar_member!(f32_member, f32, "<f4", 4);
scalar_member!(i32_member, i32, "<i4", 4);

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 参考身份与解码布局自查（不启动 GPU；窗口前即可执行）。
#[test]
fn perworld_reference_decodes_with_bound_hashes() {
    for (bytes, expected, name) in [
        (RGB_NPZ, RGB_SHA256, "rgb.npz"),
        (DEPTH_NPZ, DEPTH_SHA256, "depth.npz"),
        (SEG_NPZ, SEG_SHA256, "seg.npz"),
        (STATE_NPZ, STATE_SHA256, "state.npz"),
    ] {
        assert_eq!(
            sha256_hex(bytes),
            expected,
            "{PERWORLD_DIR}{name} sha256 不符"
        );
    }
    let rgb = npz_members(RGB_NPZ);
    let depth = npz_members(DEPTH_NPZ);
    let seg = npz_members(SEG_NPZ);
    for world in 0..WORLDS {
        let (shape, values) = f32_member(&rgb, &format!("rgb_w{world}_cam0.npy"));
        assert_eq!(shape, vec![HEIGHT, WIDTH, 3], "rgb_w{world} 布局");
        assert_eq!(values.len(), PIXELS * 3);
        let (shape, values) = f32_member(&depth, &format!("depth_w{world}_cam0.npy"));
        assert_eq!(shape, vec![HEIGHT, WIDTH], "depth_w{world} 布局");
        assert_eq!(values.len(), PIXELS);
        let (shape, values) = i32_member(&seg, &format!("seg_w{world}_cam0.npy"));
        assert_eq!(shape, vec![HEIGHT, WIDTH, 2], "seg_w{world} 布局");
        assert_eq!(values.len(), PIXELS * 2);
    }
    let poses = scene_poses_from_state();
    assert_eq!(poses.len(), WORLDS * LEAVES);
    // 三个 world 的 hinge 角度不同 → 至少一个 leaf 位姿不同（采集语义自查）。
    assert!(
        (0..LEAVES).any(|slot| poses[slot].xpos.map(f32::to_bits)
            != poses[LEAVES + slot].xpos.map(f32::to_bits)),
        "world0 与 world1 至少应有一个 leaf 位姿不同"
    );
}

/// per-world 位姿：`ScenePose` 世界优先，按 scene leaf 顺序（fixture 顺序 = geom 槽位）。
fn scene_poses_from_state() -> Vec<ScenePose> {
    let members = npz_members(STATE_NPZ);
    let (xpos_shape, xpos) = f32_member(&members, "geom_xpos.npy");
    let (xmat_shape, xmat) = f32_member(&members, "geom_xmat.npy");
    assert_eq!(xpos_shape, vec![WORLDS, LEAVES, 3], "state geom_xpos 形状");
    assert_eq!(
        xmat_shape,
        vec![WORLDS, LEAVES, 3, 3],
        "state geom_xmat 形状"
    );
    let mut poses = Vec::with_capacity(WORLDS * LEAVES);
    for world in 0..WORLDS {
        for leaf in 0..LEAVES {
            let base = (world * LEAVES + leaf) * 3;
            let mbase = (world * LEAVES + leaf) * 9;
            poses.push(ScenePose {
                xpos: std::array::from_fn(|index| xpos[base + index]),
                xmat: std::array::from_fn(|index| xmat[mbase + index]),
            });
        }
    }
    poses
}

fn perworld_params(nworld: usize) -> ContextParams<'static> {
    ContextParams {
        nworld,
        render_rgb: Some(RenderFlagSpec::Bool(true)),
        render_depth: Some(RenderFlagSpec::Bool(true)),
        render_seg: Some(RenderFlagSpec::Bool(true)),
        use_fast_math: false,
        use_shadows: true,
        enabled_geom_groups: &GEOM_GROUPS,
        cam_active: CameraActive::Mask(&CAM_ACTIVE),
        shadow_light_fraction: 0.3,
        samples_per_pixel: 1,
        ..Default::default()
    }
}

fn ulp(a: f32, b: f32) -> u32 {
    let map = |value: f32| -> i64 {
        let bits = value.to_bits() as i32;
        if bits < 0 {
            0x8000_0000i64 - i64::from(bits)
        } else {
            i64::from(bits)
        }
    };
    (map(a) - map(b)).unsigned_abs() as u32
}

fn packed_rgb(values: &[f32]) -> Vec<u32> {
    values
        .chunks(3)
        .map(|pixel| {
            let bytes = [pixel[0], pixel[1], pixel[2]].map(|channel| {
                assert!((0.0..=1.0).contains(&channel), "rgb 参考越界：{channel}");
                let scaled = channel * 255.0;
                assert!(
                    (scaled - scaled.round()).abs() <= 0.01,
                    "rgb 参考不在 byte/255 网格：{channel}"
                );
                scaled.round() as u32
            });
            (0xFFu32 << 24) | (bytes[0] << 16) | (bytes[1] << 8) | bytes[2]
        })
        .collect()
}

fn scene_state(state_poses: &[ScenePose], fixture: &s7b1::SceneFixture) -> SceneState {
    let mut leaves = Vec::with_capacity(WORLDS * LEAVES);
    for world in 0..WORLDS {
        for leaf in &fixture.leaves {
            let mut leaf = *leaf;
            leaf.group = world as u32;
            leaves.push(leaf);
        }
    }
    let poses: Vec<LeafPose> = state_poses
        .iter()
        .map(|pose| LeafPose {
            xpos: pose.xpos,
            xmat: pose.xmat,
        })
        .collect();
    SceneState::new(leaves, poses, WORLDS).expect("3 world 场景状态")
}

#[test]
#[ignore = "需要 GPU + NVRTC（验证窗口内执行逐 world 位级比对）"]
fn gpu_per_world_matches_captured_reference() {
    let state_poses = scene_poses_from_state();
    let world_poses =
        WorldScenePoses::new(WORLDS, LEAVES, state_poses.clone()).expect("per-world 位姿");
    let session = s7b2::session();
    let fields = s7b2::render_fields();
    let model = CameraModel::from_fields(&fields).expect("相机模型");
    let params3 = perworld_params(WORLDS);
    let resolved3 = resolve_context(&fields, &params3).expect("nworld=3 解析");
    assert_eq!(resolved3.nrender(), 1, "capture 只激活 1 个相机");
    assert_eq!(resolved3.output.nsamples, 1, "capture spp=1");
    let base = s7b1::camera_data_poses(&s7b2::reference_depth_seg(), &model, &resolved3);
    assert_eq!(
        base[0].xpos.map(f32::to_bits),
        [1.2f32, -1.1, 0.8].map(f32::to_bits),
        "cam-persp 相机位姿必须与采集口径一致"
    );
    let mut camera_poses = Vec::with_capacity(WORLDS * base.len());
    for _ in 0..WORLDS {
        camera_poses.extend_from_slice(&base);
    }
    let fixture = s7b1::scene_fixture();
    assert_eq!(fixture.leaves.len(), LEAVES, "scene-a leaf 数");
    let mut context = r10::context_with_state(
        &session,
        &fields,
        &params3,
        scene_state(&state_poses, &fixture),
        fixture.meshes.clone(),
        true,
    );
    let renderer = MegakernelRenderer::new(&session).expect("megakernel 编译");
    // 射线必须按本实体自己的 resolved plan 构造：perworld 参数只激活 1 个相机
    // （`cam_active=[true,false]` → total_rays=76,800），不能用默认双相机 helper。
    let rays = RayBuilder::new(&session)
        .expect("build_rays 编译")
        .build(&model, &resolved3.plan, RayMode::Precomputed)
        .expect("per-world 预计算射线");
    // 内核按 `total_rays × nsamples` 校验射线数，且 total_rays = 本 plan 激活相机数 × 宽 × 高
    // （不能混入未激活相机的射线，否则触发 `InvalidArray("ray_count")`）。
    assert_eq!(
        rays.len(),
        resolved3.output.ray_count,
        "射线条数必须等于 resolved plan 的 ray_count"
    );
    assert_eq!(
        resolved3.output.total_rays,
        resolved3.nrender() * WIDTH * HEIGHT,
        "总射线数必须 = 激活相机数 × 宽 × 高"
    );
    let launch = |context: &mut mjwarp_rs::render::RenderContext, poses: &WorldScenePoses| {
        render_per_world(
            &renderer,
            context,
            &fields,
            &model,
            poses,
            &camera_poses,
            &rays,
        )
        .expect("逐 world 发射");
    };
    launch(&mut context, &world_poses);
    let rgb_a = s7b1::rgb_of(&context);
    let depth_a = s7b1::read_depth(&context);
    let seg_a = s7b1::read_seg(&context);
    assert_eq!(rgb_a.len(), WORLDS * PIXELS);
    assert_eq!(depth_a.len(), WORLDS * PIXELS);
    assert_eq!(seg_a.len(), WORLDS * PIXELS);

    let rgb = npz_members(RGB_NPZ);
    let depth = npz_members(DEPTH_NPZ);
    let seg = npz_members(SEG_NPZ);
    for world in 0..WORLDS {
        let slice = world * PIXELS..(world + 1) * PIXELS;
        let (_, rgb_values) = f32_member(&rgb, &format!("rgb_w{world}_cam0.npy"));
        let reference = s7b2::RgbCamera {
            index: 0,
            width: WIDTH,
            height: HEIGHT,
            pixels: PIXELS,
            packed: packed_rgb(&rgb_values),
        };
        let stats = s7b2::diff_rgb(&rgb_a[slice.clone()], &reference);
        assert_eq!(stats.mismatched_pixels, 0, "world {world} RGB 不匹配像素");
        assert_eq!(stats.max_byte_diff, 0, "world {world} RGB 最大字节差");
        let (_, depth_values) = f32_member(&depth, &format!("depth_w{world}_cam0.npy"));
        let mut mismatched = 0usize;
        let mut max_ulp = 0u32;
        let mut beyond = 0usize;
        for (index, expected) in depth_values.iter().enumerate() {
            let actual = depth_a[slice.start + index];
            if actual.to_bits() != expected.to_bits() {
                mismatched += 1;
                let distance = ulp(actual, *expected);
                max_ulp = max_ulp.max(distance);
                beyond += usize::from(i64::from(distance) > MAX_DEPTH_ULP);
            }
        }
        assert_eq!(
            mismatched, 0,
            "world {world} depth 逐位不匹配（max_ulp={max_ulp}）"
        );
        assert_eq!(beyond, 0, "world {world} depth 超 {MAX_DEPTH_ULP} ULP");
        let (_, seg_values) = i32_member(&seg, &format!("seg_w{world}_cam0.npy"));
        for pixel in 0..PIXELS {
            assert_eq!(
                seg_a[slice.start + pixel],
                [seg_values[2 * pixel], seg_values[2 * pixel + 1]],
                "world {world} seg 像素 {pixel}"
            );
        }
    }

    // 两次运行 SHA 相同（确定性）。
    launch(&mut context, &world_poses);
    assert_eq!(
        r10::sha256_rgb(&rgb_a),
        r10::sha256_rgb(&s7b1::rgb_of(&context))
    );
    assert_eq!(
        r10::sha256_depth(&depth_a),
        r10::sha256_depth(&s7b1::read_depth(&context))
    );
    assert_eq!(
        r10::sha256_seg(&seg_a),
        r10::sha256_seg(&s7b1::read_seg(&context))
    );

    // world 隔离：仅改 world1 位姿后，world0/world2 逐位不变、world1 必须变化。
    let mut moved = state_poses.clone();
    for slot in 0..LEAVES {
        moved[LEAVES + slot].xpos[0] += 0.5;
    }
    let moved_poses = WorldScenePoses::new(WORLDS, LEAVES, moved).expect("隔离用例位姿");
    let (rgb_before, depth_before) = (s7b1::rgb_of(&context), s7b1::read_depth(&context));
    launch(&mut context, &moved_poses);
    let (rgb_after, depth_after) = (s7b1::rgb_of(&context), s7b1::read_depth(&context));
    for world in [0usize, 2] {
        let slice = world * PIXELS..(world + 1) * PIXELS;
        assert_eq!(
            &rgb_before[slice.clone()],
            &rgb_after[slice.clone()],
            "world {world} RGB 隔离"
        );
        assert_eq!(
            depth_before[slice.clone()]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            depth_after[slice]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            "world {world} depth 隔离"
        );
    }
    assert_ne!(
        rgb_before[PIXELS..2 * PIXELS],
        rgb_after[PIXELS..2 * PIXELS],
        "world1 位姿变更必须影响其输出（隔离反例成立性）"
    );
}
