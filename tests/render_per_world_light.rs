//! per-world L v2 判据实体 `render_per_world_light`（spec v3 §10.1；GPU 用例 `#[ignore]`，G-L2 才真跑）。
//!
//! 读 §2 参考（rgb/depth/seg 各 6 个 world×相机键 + state）→ §4 判据 A 两台相机各自逐 world 严格位级 →
//! §5 判据 B 灯字段数据面位型 → §6 判据 C 参考侧自算 C1/C2（384 验收/576 采集前置、两对 world、两套不互换）
//! → 两跑 sha；另加 world 隔离性断言（两台相机，不替代 A/B/C）。
//! 产品段 = Card H `render_per_world_with_lights` + `WorldLightPoses`（逐 world `light_xpos/light_xdir`
//! 从 `state.npz` 逐位透传）。测试期模型输入 = `tests/support/render_bvh/light/model-315.json`（3.15 dump）。
//!
//! 运行门控：参考目录尚不存在（G-L1 采集后才有）→ GPU 用例 `#[ignore]`；若被显式执行且参考缺失，
//! 显式报「not executed（reference missing）」而不是静默 skip。默认门禁（无 `cuda-probe`）为空文件。
//!
//! GPU 真跑（G-L2 冻结 exe）：
//!   cargo test --locked --features cuda-probe --test render_per_world_light -- --ignored --test-threads=1
#![cfg(feature = "cuda-probe")]

#[path = "support/render_bvh/light.rs"]
mod light;

use mjwarp_rs::render::kernel::{MegakernelRenderer, render_per_world_with_lights};
use mjwarp_rs::render::{WorldScenePoses, resolve_context};
use mjwarp_rs::runtime::TransferSession;

use light::{ACTIVE_CAMERA_COUNT, PIXELS, Verdict, WORLDS};

// ---------------------------------------------------------------------------
// 参考 npz 最小解析（ZIP_STORED + npy v1/v2/v3 头；不引第三方依赖、不新增夹具）
// ---------------------------------------------------------------------------

mod npz {
    pub fn members(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut members = Vec::new();
        let mut offset = 0usize;
        while offset + 4 <= bytes.len() && &bytes[offset..offset + 4] == b"PK\x03\x04" {
            let compressed =
                u32::from_le_bytes(bytes[offset + 18..offset + 22].try_into().unwrap());
            let name_len = u16::from_le_bytes([bytes[offset + 26], bytes[offset + 27]]) as usize;
            let extra_len = u16::from_le_bytes([bytes[offset + 28], bytes[offset + 29]]) as usize;
            let method = u16::from_le_bytes([bytes[offset + 8], bytes[offset + 9]]);
            assert_eq!(method, 0, "npz 仅支持 ZIP_STORED");
            let name_start = offset + 30;
            let name =
                String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).to_string();
            let data_start = name_start + name_len + extra_len;
            let data_end = data_start + compressed as usize;
            assert!(data_end <= bytes.len(), "npz 成员 {name} 越界");
            members.push((name, bytes[data_start..data_end].to_vec()));
            offset = data_end;
        }
        assert!(!members.is_empty(), "npz 无成员");
        members
    }

    /// 解析 npy 头（只支持 C 顺序小端）；返回 `(descr, shape, raw data)`。
    pub fn member<'a>(
        members: &'a [(String, Vec<u8>)],
        name: &str,
    ) -> (String, Vec<usize>, &'a [u8]) {
        let (_, bytes) = members
            .iter()
            .find(|(member_name, _)| member_name == name)
            .unwrap_or_else(|| panic!("npz 缺少成员 {name}"));
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
        (descr, shape, &bytes[header_end..])
    }
}

// ---------------------------------------------------------------------------
// 参考数据（runtime fs；G-L1 后存在）
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub rgb: Vec<u32>,
    pub depth: Vec<f32>,
    pub seg: Vec<[i32; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceState {
    pub geom_xpos: Vec<f32>,
    pub geom_xmat: Vec<f32>,
    pub light_xpos: Vec<f32>,
    pub light_xdir: Vec<f32>,
}

pub struct Reference {
    /// `[world][cam]`，cam 顺序 = 启用相机顺序（cam-persp、cam-hinge）。
    pub frames: Vec<Vec<Frame>>,
    /// 原始 f32 rgb（`[world][cam][pixel] = [r,g,b]`，未被量化成 8-bit）：判据 C 的 C1 口径必须用它，
    /// 因为 spec v3 §6 比较的是 f32 `max_channel|Δrgb| ≥ 1/255`，量化回字节会引入 254↔255 饱和步进误差。
    pub rgb_f32: Vec<Vec<Vec<[f32; 3]>>>,
    pub state: ReferenceState,
    pub capture: serde_json::Value,
}

fn f32_member(members: &[(String, Vec<u8>)], name: &str) -> Vec<f32> {
    let (descr, _, data) = npz::member(members, name);
    assert_eq!(descr, "<f4", "{name} dtype");
    data.as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

fn i32_member(members: &[(String, Vec<u8>)], name: &str) -> Vec<i32> {
    let (descr, _, data) = npz::member(members, name);
    assert_eq!(descr, "<i4", "{name} dtype");
    data.as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| i32::from_le_bytes(*chunk))
        .collect()
}

/// 参考 f32 rgb（[0,1] 且落在 byte/255 网格）-> 产品 packed u32（0xFF<<24 | r<<16 | g<<8 | b）。
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

fn read_file(dir: &std::path::Path, name: &str) -> Result<Vec<u8>, String> {
    std::fs::read(dir.join(name)).map_err(|error| format!("{}{name}: {error}", dir.display()))
}

/// 读参考目录；目录/文件缺失 -> `Err`（调用方必须显式报「未执行」）。
#[rustfmt::skip]
pub fn load_reference(dir: &std::path::Path) -> Result<Reference, String> {
    if !dir.is_dir() {
        return Err(format!("reference missing (G-L1 capture pending): {} (required rgb/depth/seg/state npz + capture.json)", dir.display()));
    }
    let rgb = npz::members(&read_file(dir, "rgb.npz")?);
    let depth = npz::members(&read_file(dir, "depth.npz")?);
    let seg = npz::members(&read_file(dir, "seg.npz")?);
    let state = npz::members(&read_file(dir, "state.npz")?);
    let capture: serde_json::Value = serde_json::from_slice(&read_file(dir, "capture.json")?).map_err(|error| format!("capture.json: {error}"))?;
    let mut frames = Vec::with_capacity(WORLDS);
    let mut rgb_f32 = Vec::with_capacity(WORLDS);
    for world in 0..WORLDS {
        let mut row = Vec::with_capacity(ACTIVE_CAMERA_COUNT);
        let mut rgb_row = Vec::with_capacity(ACTIVE_CAMERA_COUNT);
        for cam in 0..ACTIVE_CAMERA_COUNT {
            let rgb_values = f32_member(&rgb, &format!("rgb_w{world}_cam{cam}.npy"));
            let depth_values = f32_member(&depth, &format!("depth_w{world}_cam{cam}.npy"));
            let seg_values = i32_member(&seg, &format!("seg_w{world}_cam{cam}.npy"));
            assert_eq!(rgb_values.len(), 3 * PIXELS);
            assert_eq!(depth_values.len(), PIXELS);
            assert_eq!(seg_values.len(), 2 * PIXELS);
            rgb_row.push(rgb_values.chunks(3).map(|pixel| [pixel[0], pixel[1], pixel[2]]).collect());
            row.push(Frame { rgb: packed_rgb(&rgb_values), depth: depth_values,
                             seg: seg_values.chunks(2).map(|pair| [pair[0], pair[1]]).collect() });
        }
        frames.push(row);
        rgb_f32.push(rgb_row);
    }
    Ok(Reference {
        frames, rgb_f32,
        state: ReferenceState { geom_xpos: f32_member(&state, "geom_xpos.npy"), geom_xmat: f32_member(&state, "geom_xmat.npy"),
                                light_xpos: f32_member(&state, "light_xpos.npy"), light_xdir: f32_member(&state, "light_xdir.npy") },
        capture,
    })
}

/// 世界隔离用：深拷贝 state 并仅改 world1 的几何位姿 + 灯位姿。
fn moved_world1(state: &ReferenceState) -> ReferenceState {
    let mut moved = state.clone();
    for leaf in 0..light::LEAVES {
        let base = (light::LEAVES + leaf) * 3;
        moved.geom_xpos[base] += 0.5;
    }
    for light in 0..light::CAMERAS {
        let base = (3 + light) * 3;
        moved.light_xpos[base] += 0.5;
    }
    moved
}

#[test]
#[rustfmt::skip]
fn missing_reference_reports_explicit_not_executed_error() {
    let dir = light::reference_dir().parent().expect("参考根").join("definitely-missing");
    let error = match load_reference(&dir) { Ok(_) => panic!("缺失参考必须返回 Err"), Err(error) => error };
    assert!(error.contains("reference missing"), "错误必须显式说明参考缺失：{error}");
    assert!(!error.contains("panic"), "不得用 panic 代替显式错误");
}

/// spec v3 §2.1/§2.2 写死内容的完整 capture.json 样本（值取自 pinned 3.15 的 pilot attempt 3）。
fn good_capture() -> serde_json::Value {
    serde_json::json!({
        "two_captures_bitwise_identical": true,
        "captured_at": "2026-11-08T19:00:00+08:00",
        "reference_tag": "nw3-light-hinge02040",
        "legacy_reference_untouched": true,
        "command": {"argv": ["python.exe", "capture_render_perworld.py", "--scene",
            "fixtures/render-bvh/scene-a-light/scene.xml", "--out-root", "out", "--out-dir", "out/tag",
            "--reference-tag", "nw3-light-hinge02040", "--worlds", "3", "--hinge-deg", "0,20,40",
            "--use-fast-math", "false", "--use-shadows", "true", "--seed", "20261030", "--repeat", "2"]},
        "per_world_scope": {"covers": "per-world light poses (light_xpos/light_xdir) AND per-world geom \
            poses (geom_xpos/geom_xmat) with nworld > 1", "does_not_cover": "AA x perworld"},
        "parameters": {"cam_active": [true, true, false]},
        "recipe": {
            "scene_xml": {"path": "fixtures/render-bvh/scene-a-light/scene.xml",
                "sha256": "4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716", "bytes": 5134},
            "cameras": [
                {"slot": 0, "mujoco_cam_id": 0, "name": "cam-persp", "mode": 0,
                 "xpos": [1.2, -1.1, 0.8],
                 "xquat": [0.9233805168766387, 0.10259783520851541, -0.20519567041703082, 0.3077935056255462],
                 "fovy": 73.73979461915735, "resolution": [320, 240],
                 "sensorsize": [0.035999998450279236, 0.024000000208616257],
                 "intrinsic": [0.017999999225139618, 0.01600000075995922, 0.0, 0.0]},
                {"slot": 1, "mujoco_cam_id": 1, "name": "cam-hinge", "mode": 0,
                 "xpos": [-0.0925, 0.4981, 0.8296],
                 "xquat": [-0.2110880661810797, -0.14976304695424203, 0.5589241752358911, 0.7877922469914213],
                 "fovy": 30.0, "resolution": [320, 240], "sensorsize": [0.0, 0.0],
                 "intrinsic": [0.009999999776482582, 0.009999999776482582, 0.0, 0.0]}
            ]
        },
        "context": {"resolved": {"nworld": 3, "nrender": 2, "cam_id_map": [0, 1],
            "cam_res": [[320, 240], [320, 240]], "legacy_manifest_entries": 112}}
    })
}

#[test]
#[rustfmt::skip]
fn capture_provenance_self_check_rejects_bad_or_missing_fields() {
    light::check_capture_provenance(&good_capture());
    type Mutation = Box<dyn Fn(&mut serde_json::Value)>;
    let mutations: Vec<(&str, Mutation)> = vec![
        ("two-run flag", Box::new(|v: &mut serde_json::Value| v["two_captures_bitwise_identical"] = serde_json::json!(false))),
        ("captured_at", Box::new(|v: &mut serde_json::Value| v["captured_at"] = serde_json::json!(""))),
        ("tag", Box::new(|v: &mut serde_json::Value| v["reference_tag"] = serde_json::json!("wrong"))),
        ("scene sha", Box::new(|v: &mut serde_json::Value| v["recipe"]["scene_xml"]["sha256"] = serde_json::json!("00"))),
        ("scene bytes", Box::new(|v: &mut serde_json::Value| v["recipe"]["scene_xml"]["bytes"] = serde_json::json!(5135))),
        ("nrender", Box::new(|v: &mut serde_json::Value| v["context"]["resolved"]["nrender"] = serde_json::json!(1))),
        ("nworld", Box::new(|v: &mut serde_json::Value| v["context"]["resolved"]["nworld"] = serde_json::json!(2))),
        ("cam_id_map", Box::new(|v: &mut serde_json::Value| v["context"]["resolved"]["cam_id_map"] = serde_json::json!([0, 2]))),
        ("cam_res", Box::new(|v: &mut serde_json::Value| v["context"]["resolved"]["cam_res"] = serde_json::json!([[320, 240], [240, 180]]))),
        ("cam_active", Box::new(|v: &mut serde_json::Value| v["parameters"]["cam_active"] = serde_json::json!([true, false, false]))),
        ("legacy entries", Box::new(|v: &mut serde_json::Value| v["context"]["resolved"]["legacy_manifest_entries"] = serde_json::json!(107))),
        ("camera field missing", Box::new(|v: &mut serde_json::Value| { v["recipe"]["cameras"][0].as_object_mut().unwrap().remove("intrinsic"); })),
        ("camera fovy", Box::new(|v: &mut serde_json::Value| v["recipe"]["cameras"][0]["fovy"] = serde_json::json!(60.0))),
        ("camera sensorsize", Box::new(|v: &mut serde_json::Value| v["recipe"]["cameras"][0]["sensorsize"] = serde_json::json!([0.0, 0.0]))),
        ("camera intrinsic", Box::new(|v: &mut serde_json::Value| v["recipe"]["cameras"][1]["intrinsic"] = serde_json::json!([0.02, 0.02, 0.0, 0.0]))),
        ("camera quat", Box::new(|v: &mut serde_json::Value| v["recipe"]["cameras"][1]["xquat"] = serde_json::json!([0.0, 0.0, 0.0, 1.0]))),
        ("camera order", Box::new(|v: &mut serde_json::Value| v["recipe"]["cameras"].as_array_mut().unwrap().swap(0, 1))),
        ("command argv", Box::new(|v: &mut serde_json::Value| v["command"]["argv"] = serde_json::json!(["python.exe"]))),
        ("scope covers", Box::new(|v: &mut serde_json::Value| v["per_world_scope"]["covers"] = serde_json::json!("light poses only"))),
        ("legacy untouched", Box::new(|v: &mut serde_json::Value| v["legacy_reference_untouched"] = serde_json::json!(false))),
    ];
    for (label, mutate) in mutations.iter() {
        let mut bad = good_capture();
        mutate(&mut bad);
        assert!(std::panic::catch_unwind(|| light::check_capture_provenance(&bad)).is_err(), "provenance 自核必须拒绝 {label}");
    }
}

#[test]
#[ignore = "G-L2 验证窗冻结 exe：需要 G-L1 采集参考（目录不存在时显式报未执行，不静默 skip）"]
fn gpu_per_world_light_matches_reference() {
    let reference = load_reference(&light::reference_dir())
        .unwrap_or_else(|error| panic!("not executed: {error}"));
    light::check_capture_provenance(&reference.capture);

    let session = TransferSession::new(0).expect("本用例需要 NVIDIA 驱动与 cuda-probe");
    let fields = light::render_fields();
    let model = light::camera_model(&fields);
    let resolved = resolve_context(&fields, &light::context_params()).expect("context 解析");
    assert_eq!(
        resolved.cam_id_map(),
        vec![0, 1],
        "启用相机顺序必须是 cam-persp/cam-hinge"
    );
    assert_eq!(resolved.nrender(), ACTIVE_CAMERA_COUNT);
    let rays = light::precomputed_rays(&session, &fields, &resolved);
    let geometry = light::scene_geometry();
    let camera_poses = light::camera_poses(&fields);
    let renderer = MegakernelRenderer::new(&session).expect("megakernel 编译");
    let mut context = light::create_context(&session, &fields, resolved.clone(), &geometry);

    // ---- 判据 C（参考侧自算，必须在任何产品数据前）----
    let counts = light::criterion_c(&reference);
    let verdict = light::verdict(&counts);
    println!(
        "c1_rgb_changed={:?} c2_depth_changed={:?} c2_seg_changed={:?} verdict={verdict:?}",
        counts.c1_rgb_changed, counts.c2_depth_changed, counts.c2_seg_changed
    );
    println!(
        "diag campersp depth={:?} seg={:?}; camhinge rgb={:?}",
        counts.diag_campersp_depth, counts.diag_campersp_seg, counts.diag_camhinge_rgb
    );
    assert_eq!(
        verdict,
        Verdict::PassMargin,
        "参考侧判别力必须 pass-margin（两套两对 ≥576），实际 {verdict:?}"
    );
    // 参考可用时：实体 C1（magnitude/bitwise）必须等于 capture.json 记录的 §2.4 计数；缺字段/旧结构显式打印原因。
    println!(
        "capture.json C1 交叉核对 = {:?}",
        light::check_capture_c1(&reference.capture, &counts)
    );

    // ---- 产品渲染（Card H：per-world 灯位姿逐位透传）----
    let scene_poses = light::scene_poses(&reference.state);
    let lights = light::world_light_poses(&reference.state);
    render_per_world_with_lights(
        &renderer,
        &mut context,
        &fields,
        &model,
        &scene_poses,
        &lights,
        &camera_poses,
        &rays,
    )
    .expect("per-world 灯发射");
    let product = light::rendered_frames(&context, &resolved);
    let first_sha = light::sha256_frames(&product);

    // ---- 判据 A：两台相机各自逐 world 严格位级 ----
    for (world, row) in light::criterion_a(&reference, &product).iter().enumerate() {
        for (cam, stats) in row.iter().enumerate() {
            println!("A world{world} cam{cam}: {stats:?}");
            assert_eq!(
                stats.rgb_mismatched_pixels, 0,
                "world{world} cam{cam} rgb 不匹配像素"
            );
            assert_eq!(
                stats.rgb_max_byte_diff, 0,
                "world{world} cam{cam} rgb 最大字节差"
            );
            assert_eq!(
                stats.depth_mismatched_pixels, 0,
                "world{world} cam{cam} depth 位级不匹配"
            );
            assert_eq!(
                stats.depth_beyond_ulp, 0,
                "world{world} cam{cam} depth 超 0 ULP"
            );
            assert_eq!(
                stats.seg_mismatched_pixels, 0,
                "world{world} cam{cam} seg 不匹配像素"
            );
        }
    }

    // ---- 判据 B：灯字段数据面位型 ----
    let bstats = light::criterion_b(&reference.state, &lights);
    println!("B: {bstats:?}");
    assert_eq!(
        (bstats.xpos_mismatched, bstats.xdir_mismatched),
        (0, 0),
        "灯字段位型必须逐位相等（0 例外）"
    );

    // ---- 两跑 sha（确定性）----
    render_per_world_with_lights(
        &renderer,
        &mut context,
        &fields,
        &model,
        &scene_poses,
        &lights,
        &camera_poses,
        &rays,
    )
    .expect("per-world 灯发射（第二跑）");
    let second_sha = light::sha256_frames(&light::rendered_frames(&context, &resolved));
    assert_eq!(
        first_sha, second_sha,
        "两跑 rgb/depth/seg 整块 sha 必须相等"
    );

    // ---- world 隔离（两台相机；不替代 A/B/C）----
    let moved_state = moved_world1(&reference.state);
    let moved_scene: WorldScenePoses = light::scene_poses(&moved_state);
    let moved_lights = light::world_light_poses(&moved_state);
    render_per_world_with_lights(
        &renderer,
        &mut context,
        &fields,
        &model,
        &moved_scene,
        &moved_lights,
        &camera_poses,
        &rays,
    )
    .expect("隔离用例发射");
    let moved = light::rendered_frames(&context, &resolved);
    let changed = light::isolation_changed(&product, &moved);
    println!("isolation changed={changed:?}");
    for world in [0usize, 2] {
        assert_eq!(
            changed[world],
            vec![false; ACTIVE_CAMERA_COUNT],
            "world{world} 两台相机输出必须逐位不变"
        );
    }
    assert!(
        changed[1].iter().any(|value| *value),
        "world1 变化必须影响其输出"
    );
}

/// 本机人工交叉核对（Chief 2026-11-08 F1 要求）：用 G-L0 pilot attempt 3 的参考 npz 校验 C1 口径
/// 与 `readout_c1_c2.py` 逐值一致。**pilot 产物只在 `target/`、不入库**，故 `#[ignore]`，手动执行：
///   cargo test --locked --features cuda-probe --test render_per_world_light -- --ignored pilot_c1
#[test]
#[ignore = "pilot 参考在 target/（不入库）：本机人工交叉核对 C1=38,285/40,644"]
#[rustfmt::skip]
fn pilot_c1_crosscheck_matches_readout() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/render-bvh/perworld-l-v2-gl0");
    let reference = load_reference(&root.join("pilot-out/nw3-light-hinge02040")).unwrap_or_else(|error| panic!("not executed: {error}"));
    light::check_capture_provenance(&reference.capture);
    let counts = light::criterion_c(&reference);
    let verdict = light::verdict(&counts);
    println!("pilot C1={:?} C2 depth={:?} seg={:?} verdict={verdict:?}",
             counts.c1_rgb_changed, counts.c2_depth_changed, counts.c2_seg_changed);
    assert_eq!(counts.c1_rgb_changed, [38_285, 40_644], "C1 必须与 readout_c1_c2.py 的 f32 1/255 口径逐值一致（254↔255 饱和步进不计）");
    assert_eq!(counts.c1_rgb_bitwise_changed, [43_604, 41_815], "C1 bitwise 与 pilot readout 逐值一致");
    // 旧 pilot capture.json 无 §2.4 字段 -> 显式打印跳过原因（不静默）；用真实 counts 构造 §2.4 字典 -> 必须比对通过。
    let pilot_capture_check = light::check_capture_c1(&reference.capture, &counts);
    println!("旧 pilot capture.json（无 §2.4 字段）C1 交叉核对 = {pilot_capture_check:?}");
    assert!(matches!(pilot_capture_check, light::CaptureC1Check::Skipped(_)), "旧参考缺 §2.4 字段必须显式跳过");
    let spec_c1 = serde_json::json!({"c1_rgb_changed_px": {
        "0->1": {"magnitude": counts.c1_rgb_changed[0], "bitwise": counts.c1_rgb_bitwise_changed[0]},
        "1->2": {"magnitude": counts.c1_rgb_changed[1], "bitwise": counts.c1_rgb_bitwise_changed[1]}}});
    assert_eq!(light::check_capture_c1(&spec_c1, &counts), light::CaptureC1Check::Compared, "§2.4 字典结构与实体 counts 必须比对通过");
    assert_eq!(counts.c2_depth_changed, [35_828, 33_883], "C2 depth 与 pilot readout 一致");
    assert_eq!(counts.c2_seg_changed, [16_041, 16_754], "C2 seg 与 pilot readout 一致");
    assert_eq!(verdict, Verdict::PassMargin);
    // 修复后 readout 产物在场时再交叉一次（不入库文件，存在才比）。
    if let Ok(bytes) = std::fs::read(root.join("window/readout.json")) {
        let readout: serde_json::Value = serde_json::from_slice(&bytes).expect("readout.json");
        let recorded: Vec<u64> = ["0->1", "1->2"].iter()
            .map(|pair| readout["c1_cam_persp"][pair]["rgb_changed_px"].as_u64().expect("readout C1")).collect();
        assert_eq!(recorded, vec![38_285, 40_644], "readout.json C1 交叉核对");
    }
}
