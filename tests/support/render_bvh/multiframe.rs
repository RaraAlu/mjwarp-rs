//! R06 多帧实体 A 共享夹具与 CPU 自检（GPU 用例见 `tests/render_flex_multiframe.rs`）。
//!
//! 参考：G-2 落库的 3 帧参考 `fixtures/render-bvh/scene-flex/multiframe/`（verdict pass-margin）。
//! 普通 `cargo test` 即可验证：参考完整性（12 npz sha 对齐 capture、3 文本 sha 锚定）、刚性数组逐帧位等、
//! flex 位移 0/0.1/0.2 m、跨帧变化像素（实体自算 ≥384，不读 `delta_selfcheck`）、provenance（spec/三脚本/
//! refit_calls=6/actual_sequence）、allowlist 像素参考 seg；并向 GPU 用例提供 scene/flex/context 构造与逐帧判据。
//!
//! 边界：不改现有 `render_flex_validate.rs`；不改 G-2 夹具/spec/判据。

use serde_json::Value;

pub const FRAMES: usize = 3;
pub const FRAME_DZ: f64 = 0.10;
pub const CHANGE_PIXELS_MIN: usize = 384;
pub const SPEC_SHA: &str = "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6";
pub const SPEC_BYTES: u64 = 10_506;
#[rustfmt::skip]
pub const SCRIPT_SHA: [(&str, &str); 3] = [("capture_render_flex.py", "f50e150e4dc29522ce5f49b64bbb50b01bfc5177cee82a3f3c704c9e5c0a16f9"), ("capture_render_flex_multiframe.py", "46d7500227738e8a3fab8f79106bb02599b1b92b29ca4bda6baf807b989e7bd1"), ("capture_render_flex_multiframe_checks.py", "ef77f38b6159a49971a85303961647b0aa88b596fce7dc924998b1e21f8201a9")];
#[rustfmt::skip]
pub const TEXT_FILES: [(&str, &str); 3] = [("multiframe/capture.json", "767f8c1b0ea63880b6bbdec0f30452fd24b6166ccef8aceea43acf29417a7252"), ("multiframe/cam-poses.json", "97d6a50b3fe9ac4bbfddb699af5fa5b7e3d435c013c212b1e3702387fa459afb"), ("multiframe/README.md", "fa51f4b325973236aa78b4b54eb1a8bd0d407aa72a30cab4f9310be6187f34e2")];

pub const MODEL: &str = include_str!("../../../fixtures/render-bvh/scene-flex/model-312.json");
pub const CAPTURE: &str =
    include_str!("../../../fixtures/render-bvh/scene-flex/multiframe/capture.json");
pub const CAM_POSES: &str =
    include_str!("../../../fixtures/render-bvh/scene-flex/multiframe/cam-poses.json");
pub const MULTIFRAME_README: &str =
    include_str!("../../../fixtures/render-bvh/scene-flex/multiframe/README.md");
pub const FRAME_NPZ: [[&[u8]; 4]; FRAMES] = [
    [
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-000/rgb.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-000/depth.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-000/seg.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-000/state.npz"),
    ],
    [
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-001/rgb.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-001/depth.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-001/seg.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-001/state.npz"),
    ],
    [
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-002/rgb.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-002/depth.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-002/seg.npz"),
        include_bytes!("../../../fixtures/render-bvh/scene-flex/multiframe/frame-002/state.npz"),
    ],
];

pub fn capture() -> Value {
    serde_json::from_str(CAPTURE).expect("multiframe/capture.json")
}
pub fn model() -> Value {
    serde_json::from_str(MODEL).expect("scene-flex/model-312.json")
}
#[rustfmt::skip]
fn model_ints(model: &Value, name: &str) -> Vec<i32> { model["arrays"][name]["values"].as_array().expect(name).iter().map(|v| v.as_i64().expect("i32") as i32).collect() }
#[rustfmt::skip]
fn model_numbers(model: &Value, name: &str) -> Vec<f64> { model["arrays"][name]["values"].as_array().expect(name).iter().map(|v| v.as_f64().expect("f64")).collect() }
#[rustfmt::skip]
fn model_count(model: &Value, name: &str) -> usize { model["counts"][name].as_u64().expect(name) as usize }

#[rustfmt::skip]
fn f32_any(members: &[(String, Vec<u8>)], name: &str) -> (Vec<usize>, Vec<f32>) {
    let (descr, shape, data) = crate::shadow::member(members, name);
    assert_eq!(descr, "<f4", "{name} descr");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    (shape, chunks.iter().map(|c| f32::from_le_bytes(*c)).collect())
}
#[rustfmt::skip]
fn camera_slots(members: &[(String, Vec<u8>)]) -> Vec<usize> {
    let mut slots: Vec<usize> = members.iter().filter_map(|(name, _)| name.strip_prefix("rgb_cam").and_then(|rest| rest.strip_suffix(".npy")).and_then(|n| n.parse().ok())).collect();
    slots.sort_unstable();
    slots
}

pub struct Frame {
    pub index: usize,
    pub rgb: Vec<crate::s7b2::RgbCamera>,
    pub depth_bits: Vec<Vec<u32>>,
    pub seg: Vec<Vec<[i32; 2]>>,
    pub flexvert_xpos: Vec<f32>,
    pub rigid: RigidState,
}
#[derive(Debug, PartialEq, Eq)]
pub struct RigidState {
    pub geom_xpos: Vec<u32>,
    pub geom_xmat: Vec<u32>,
    pub light_xpos: Vec<u32>,
    pub light_xdir: Vec<u32>,
    pub qpos: Vec<u32>,
}

#[rustfmt::skip]
fn parse_frame(index: usize) -> Frame {
    let (rgb_bytes, depth_bytes, seg_bytes, state_bytes) = (FRAME_NPZ[index][0], FRAME_NPZ[index][1], FRAME_NPZ[index][2], FRAME_NPZ[index][3]);
    let (rgb_m, depth_m, seg_m) = (crate::shadow::read_npz(rgb_bytes), crate::shadow::read_npz(depth_bytes), crate::shadow::read_npz(seg_bytes));
    let state_m = crate::shadow::read_npz(state_bytes);
    let slots = camera_slots(&rgb_m);
    assert_eq!(slots, vec![0, 1, 2], "frame-{index:03} 相机槽位");
    let (mut rgb, mut depth_bits, mut seg) = (Vec::new(), Vec::new(), Vec::new());
    for camera in &slots {
        let ((h, w), rgb_f) = crate::shadow::f32_member_rgb(&rgb_m, &format!("rgb_cam{camera}.npy"));
        let pixels = w * h;
        let mut packed = Vec::with_capacity(pixels);
        for i in 0..pixels {
            let q = |c: f32| crate::shadow::quantize_channel(c, "rgb", "scene-flex-multiframe", *camera);
            let (r, g, b) = (q(rgb_f[3 * i]), q(rgb_f[3 * i + 1]), q(rgb_f[3 * i + 2]));
            packed.push((0xFFu32 << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
        }
        rgb.push(crate::s7b2::RgbCamera { index: *camera, width: w, height: h, pixels, packed });
        let ((dh, dw), depth_f) = crate::shadow::f32_member(&depth_m, &format!("depth_cam{camera}.npy"));
        assert_eq!((dh, dw), (h, w), "frame-{index:03} cam{camera} depth 形状");
        depth_bits.push(depth_f.iter().map(|v| v.to_bits()).collect());
        let ((sh, sw, comp), seg_v) = crate::shadow::i32_member(&seg_m, &format!("seg_cam{camera}.npy"));
        assert_eq!((sh, sw, comp), (h, w, 2), "frame-{index:03} cam{camera} seg 形状");
        let (pairs, remainder) = seg_v.as_chunks::<2>();
        assert!(remainder.is_empty());
        seg.push(pairs.to_vec());
    }
    let (flex_shape, flexvert_xpos) = f32_any(&state_m, "flexvert_xpos.npy");
    assert_eq!(flex_shape, vec![1, 6, 3], "frame-{index:03} flexvert_xpos 形状");
    let bits_of = |name: &str| -> Vec<u32> { let (_, values) = f32_any(&state_m, name); values.iter().map(|v| v.to_bits()).collect() };
    Frame { index, rgb, depth_bits, seg, flexvert_xpos, rigid: RigidState {
        geom_xpos: bits_of("geom_xpos.npy"), geom_xmat: bits_of("geom_xmat.npy"),
        light_xpos: bits_of("light_xpos.npy"), light_xdir: bits_of("light_xdir.npy"), qpos: bits_of("qpos.npy"),
    } }
}

pub fn frames() -> Vec<Frame> {
    (0..FRAMES).map(parse_frame).collect()
}

#[rustfmt::skip]
pub fn attached_vertices() -> Vec<usize> {
    model_ints(&model(), "flex_vertbodyid").iter().enumerate().filter_map(|(i, body)| (*body != 0).then_some(i)).collect()
}

/// CPU 自检 A：参考完整性——12 npz sha 对齐 `capture.files_sha256` + 3 文本 sha 对齐记录常量（15 文件）。
#[rustfmt::skip]
pub fn verify_reference_integrity() -> usize {
    let capture = capture();
    let recorded = capture["files_sha256"].as_object().expect("files_sha256");
    assert_eq!(recorded.len(), 12, "capture.files_sha256 条目数");
    let mut checked = 0usize;
    for (frame, npz) in FRAME_NPZ.iter().enumerate() {
        for (slot, file) in ["rgb", "depth", "seg", "state"].iter().enumerate() {
            let key = format!("frame-{frame:03}/{file}.npz");
            let expected = recorded.get(&key).and_then(|v| v.as_str()).unwrap_or_else(|| panic!("capture.files_sha256 缺 {key}"));
            assert_eq!(crate::shadow::sha256_hex(npz[slot]), expected, "{key} sha256");
            checked += 1;
        }
    }
    for ((name, expected), bytes) in TEXT_FILES.iter().zip([CAPTURE.as_bytes(), CAM_POSES.as_bytes(), MULTIFRAME_README.as_bytes()]) {
        assert_eq!(crate::shadow::sha256_hex(bytes), *expected, "{name} sha256（文本夹具记录值）");
        checked += 1;
    }
    assert_eq!(checked, 15, "参考完整性 15 文件（12 npz + 3 文本）");
    checked
}

/// CPU 自检 B：帧集刚性数组逐帧与 frame-000 位级相等（证「只有 flex 动」）。
#[rustfmt::skip]
pub fn verify_rigid_arrays_bit_equal() -> usize {
    let frames = frames();
    let mut checked = 0usize;
    for frame in &frames[1..] {
        assert_eq!(frame.rigid, frames[0].rigid, "frame-{:03} 刚性数组与 frame-000 位级相等", frame.index);
        checked += 1;
    }
    checked
}

/// CPU 自检 C：附着顶点 `flexvert_xpos` 相对 frame-000 平均位移 = k×0.100 m（±1e-6）。
pub fn verify_displacement() -> Vec<f64> {
    let frames = frames();
    let attached = attached_vertices();
    assert_eq!(attached.len(), 5, "附着顶点数");
    let mut means = Vec::new();
    for (k, frame) in frames.iter().enumerate() {
        let mut sum = 0.0f64;
        for vertex in &attached {
            let base = 3 * vertex;
            let d = |slot: usize| {
                f64::from(frame.flexvert_xpos[base + slot])
                    - f64::from(frames[0].flexvert_xpos[base + slot])
            };
            sum += (d(0).powi(2) + d(1).powi(2) + d(2).powi(2)).sqrt();
        }
        let mean = sum / attached.len() as f64;
        let expected = k as f64 * FRAME_DZ;
        assert!(
            (mean - expected).abs() <= 1e-6,
            "frame-{k:03} 位移均值 {mean} != {expected}"
        );
        means.push(mean);
    }
    means
}

#[rustfmt::skip]
pub fn change_pixels(pair: usize) -> (usize, usize) {
    let frames = frames();
    let (left, right) = (&frames[pair], &frames[pair + 1]);
    let (mut depth_changed, mut seg_changed) = (0usize, 0usize);
    for camera in 0..left.rgb.len() {
        assert_eq!(left.rgb[camera].pixels, left.depth_bits[camera].len(), "cam{camera} depth 长度");
        depth_changed += left.depth_bits[camera].iter().zip(&right.depth_bits[camera]).filter(|(a, b)| a != b).count();
        seg_changed += left.seg[camera].iter().zip(&right.seg[camera]).filter(|(a, b)| a != b).count();
    }
    (depth_changed, seg_changed)
}

/// CPU 自检 D：跨帧可见性——两段变化像素各 ≥384（验收判据，非采集余量 576）。
pub fn verify_change_pixels() -> Vec<(usize, usize)> {
    let mut counts = Vec::new();
    for pair in 0..FRAMES - 1 {
        let (depth, seg) = change_pixels(pair);
        assert!(
            depth >= CHANGE_PIXELS_MIN,
            "{pair:03}->{} depth 变化 {depth} < {CHANGE_PIXELS_MIN}",
            pair + 1
        );
        assert!(
            seg >= CHANGE_PIXELS_MIN,
            "{pair:03}->{} seg 变化 {seg} < {CHANGE_PIXELS_MIN}",
            pair + 1
        );
        counts.push((depth, seg));
    }
    counts
}

/// CPU 自检 E：provenance——spec sha/bytes、三脚本 sha、refit 计次/序列/policy/flag。
#[rustfmt::skip]
pub fn verify_provenance() -> Value {
    let capture = capture();
    let provenance = capture["provenance"].clone();
    assert_eq!(provenance["spec_sha256"].as_str(), Some(SPEC_SHA), "spec sha256");
    assert_eq!(provenance["spec_bytes"].as_u64(), Some(SPEC_BYTES), "spec bytes");
    let scripts = provenance["authoring_script_sha256"].as_object().expect("三脚本 sha");
    assert_eq!(scripts.len(), SCRIPT_SHA.len(), "三脚本数量");
    for (name, sha) in SCRIPT_SHA {
        assert_eq!(scripts.get(name).and_then(|v| v.as_str()), Some(sha), "{name} sha256");
    }
    assert_eq!(provenance["refit_policy"].as_str(), Some("every-frame"), "refit policy");
    assert_eq!(provenance["refit_bvh"].as_bool(), Some(true), "refit_bvh");
    assert_eq!(capture["refit_calls"].as_u64(), Some(6), "refit_calls");
    let refit = &capture["capture_checks"]["refit"];
    assert_eq!(refit["actual_calls"].as_u64(), Some(6), "actual_calls");
    let sequence: Vec<i64> = refit["actual_sequence"].as_array().expect("actual_sequence").iter().map(|v| v.as_i64().expect("i64")).collect();
    assert_eq!(sequence, [0, 1, 2, 0, 1, 2], "actual_sequence");
    assert_eq!(refit["ok"].as_bool(), Some(true), "refit 自查 ok");
    capture
}

/// CPU 自检 F：allowlist 像素（cam2 x=134 y=128）参考 seg 必须为 `(flexid0, 9)`。
#[rustfmt::skip]
pub fn verify_allowlist_reference_seg() -> [i32; 2] {
    let frames = frames();
    let camera = crate::allowlist::ALLOWLIST_CAMERA;
    let width = frames[0].rgb[camera].width;
    let flat = crate::allowlist::ALLOWLIST_PIXEL_Y * width + crate::allowlist::ALLOWLIST_PIXEL_X;
    let pair = frames[0].seg[camera][flat];
    assert_eq!(pair, [crate::allowlist::ALLOWLIST_FLEXID, crate::allowlist::ALLOWLIST_SEG_TYPE], "allowlist 参考 seg");
    pair
}

/// CPU 自检 G：帧参考形状（cam0 320×240 / cam1 240×180 / cam2 320×240；cam2 = 76,800 px）。
#[rustfmt::skip]
pub fn verify_frame_shapes() -> Vec<(usize, usize)> {
    let frames = frames();
    let shapes: Vec<(usize, usize)> = frames[0].rgb.iter().map(|c| (c.width, c.height)).collect();
    assert_eq!(shapes, vec![(320, 240), (240, 180), (320, 240)], "相机分辨率");
    assert_eq!(shapes[2].0 * shapes[2].1, 76_800, "cam2 像素数");
    shapes
}

/// CPU 自检 H：两跑 sha 静态实现（GPU 用例复用同一哈希）。
#[rustfmt::skip]
pub fn output_sha(rgb: &[u32], depth: &[f32], seg: &[[i32; 2]]) -> (String, String, String) {
    let join = |w: Vec<[u8; 4]>| w.concat();
    let rgb_bytes = join(rgb.iter().map(|v| v.to_le_bytes()).collect());
    let depth_bytes = join(depth.iter().map(|v| v.to_bits().to_le_bytes()).collect());
    let seg_bytes = join(seg.iter().flat_map(|p| [p[0].to_le_bytes(), p[1].to_le_bytes()]).collect());
    (crate::shadow::sha256_hex(&rgb_bytes), crate::shadow::sha256_hex(&depth_bytes), crate::shadow::sha256_hex(&seg_bytes))
}

// ----------------------------------------------------------------------------------
// GPU 用例共用构造（本模块只在 `tests/render_flex_multiframe.rs` 里被 include，编译期无 GPU 依赖）。
// ----------------------------------------------------------------------------------

pub use mjwarp_rs::model::flex::{FlexEdgeFields, FlexPositionFields, FlexRenderFields};
pub use mjwarp_rs::render::LeafPose;
pub use mjwarp_rs::render::RenderContext;
pub use mjwarp_rs::render::SceneLeaf;
pub use mjwarp_rs::render::bvh::{mesh_half_extents, vertices_from_f64};
pub use mjwarp_rs::render::context::RenderFlagSpec;
pub use mjwarp_rs::render::context::ResolvedContext;
pub use mjwarp_rs::render::context::flex_host::{FlexHostAssembly, FlexHostData};
pub use mjwarp_rs::render::context::owner::MeshState;
pub use mjwarp_rs::render::kernel::MegakernelRenderer;
pub use mjwarp_rs::render::kernel::cast::CameraPose;
pub use mjwarp_rs::render::{CameraActive, ContextParams};

#[rustfmt::skip]
pub fn scene_meshes(model: &Value) -> Vec<MeshState> {
    let (vert, face) = (model_numbers(model, "mesh_vert"), model_ints(model, "mesh_face"));
    let (vadr, vnum) = (model_ints(model, "mesh_vertadr"), model_ints(model, "mesh_vertnum"));
    let (fadr, fnum) = (model_ints(model, "mesh_faceadr"), model_ints(model, "mesh_facenum"));
    (0..vadr.len()).map(|mesh| {
        let start = vadr[mesh] as usize * 3;
        let vertices = vertices_from_f64(&vert[start..start + vnum[mesh] as usize * 3]).expect("顶点");
        let face_start = fadr[mesh] as usize * 3;
        let indices = face[face_start..face_start + fnum[mesh] as usize * 3].iter().map(|v| *v as u32).collect();
        MeshState { vertices, indices }
    }).collect()
}

/// 场景叶 + 位姿：几何类型/尺寸取模型，位姿取 frame-k 的 `geom_xpos/geom_xmat`（f32 位型直读）。
#[rustfmt::skip]
pub fn scene_fixture(frame: &Frame, model: &Value) -> (Vec<SceneLeaf>, Vec<LeafPose>, Vec<MeshState>) {
    let meshes = scene_meshes(model);
    let mesh_half: Vec<[f32; 3]> = meshes.iter().map(|m| mesh_half_extents(&m.vertices).expect("bounds")).collect();
    let (geom_type, geom_size, geom_dataid) = (model_ints(model, "geom_type"), model_numbers(model, "geom_size"), model_ints(model, "geom_dataid"));
    let ngeom = geom_type.len();
    let bits = &frame.rigid.geom_xpos;
    let mat_bits = &frame.rigid.geom_xmat;
    let (mut leaves, mut poses) = (Vec::with_capacity(ngeom), Vec::with_capacity(ngeom));
    for geom in 0..ngeom {
        let mut size = [geom_size[3 * geom] as f32, geom_size[3 * geom + 1] as f32, geom_size[3 * geom + 2] as f32];
        if matches!(geom_type[geom], 7 | 8) { size = mesh_half[geom_dataid[geom] as usize]; }
        leaves.push(SceneLeaf { geom_type: geom_type[geom], size, group: 0, geom_id: geom as i32 });
        let mut xmat = [0.0f32; 9];
        xmat.copy_from_slice(&mat_bits[9 * geom..9 * geom + 9].iter().map(|b| f32::from_bits(*b)).collect::<Vec<_>>());
        poses.push(LeafPose { xpos: [f32::from_bits(bits[3 * geom]), f32::from_bits(bits[3 * geom + 1]), f32::from_bits(bits[3 * geom + 2])], xmat });
    }
    (leaves, poses, meshes)
}

#[rustfmt::skip]
pub fn flex_model(model: &Value) -> FlexRenderFields {
    FlexRenderFields {
        nflex: model_count(model, "nflex"), nflexvert: model_count(model, "nflexvert"), nflexedge: model_count(model, "nflexedge"),
        flex_dim: model_ints(model, "flex_dim"),
        flex_radius: model_numbers(model, "flex_radius").iter().map(|v| *v as f32).collect(),
        flex_rgba: model_numbers(model, "flex_rgba").iter().map(|v| *v as f32).collect(),
        flex_edgeadr: model_ints(model, "flex_edgeadr"), flex_edgenum: model_ints(model, "flex_edgenum"),
    }
}
#[rustfmt::skip]
pub fn flex_positions(model: &Value) -> FlexPositionFields {
    FlexPositionFields { flex_vertadr: model_ints(model, "flex_vertadr"), flex_vertnum: model_ints(model, "flex_vertnum"), flex_vertbodyid: model_ints(model, "flex_vertbodyid"), ..Default::default() }
}
/// dim==1 时元素就是边（model-312 无 flex_edge 数组；交叉校验派生前提）。
#[rustfmt::skip]
pub fn flex_edges(model: &Value) -> FlexEdgeFields {
    let (nflexvert, nflexelemdata, nflexedge) = (model_count(model, "nflexvert"), model_count(model, "nflexelemdata"), model_count(model, "nflexedge"));
    let (flex_edgeadr, flex_edgenum) = (model_ints(model, "flex_edgeadr"), model_ints(model, "flex_edgenum"));
    let (flex_vertadr, flex_vertnum) = (model_ints(model, "flex_vertadr"), model_ints(model, "flex_vertnum"));
    let flex_edge = model_ints(model, "flex_elem");
    assert_eq!(nflexelemdata, 2 * nflexedge, "dim==1 元素==边");
    assert_eq!(flex_edgeadr.first(), Some(&0));
    for flex in 0..flex_edgeadr.len() {
        let (start, verts) = (flex_edgeadr[flex] as usize, flex_vertnum[flex] as usize);
        assert!((flex_vertadr[flex] as usize) < nflexvert);
        for edge in start..start + flex_edgenum[flex] as usize {
            for slot in 0..2 { assert!((0..verts as i32).contains(&flex_edge[edge * 2 + slot]), "顶点号越界"); }
        }
    }
    let mut rowadr = Vec::with_capacity(nflexedge);
    let mut acc = 0i32;
    for _ in 0..nflexedge { rowadr.push(acc); acc += 2; }
    FlexEdgeFields { flex_edgeadr, flex_edgenum, flex_edge: flex_edge.clone(), flexedge_j_rowadr: rowadr, flexedge_j_rownnz: vec![2; nflexedge], flexedge_j_colind: flex_edge }
}

/// GPU 用例的 flex 输入（每帧一份；flexvert_xpos 取该帧参考 state）。
pub struct FlexInputs {
    pub positions: FlexPositionFields,
    pub edges: FlexEdgeFields,
    pub fields: FlexRenderFields,
    pub flexvert_xpos: Vec<f32>,
}
#[rustfmt::skip]
pub fn flex_inputs(frame: &Frame) -> FlexInputs {
    let model = model();
    FlexInputs { positions: flex_positions(&model), edges: flex_edges(&model), fields: flex_model(&model), flexvert_xpos: frame.flexvert_xpos.clone() }
}

/// 采集 recipe 的 context 参数（与 capture.json.parameters 一致；`enabled_geom_groups` 必须 4/4）。
#[rustfmt::skip]
pub fn context_params() -> ContextParams<'static> {
    ContextParams { nworld: 1, render_rgb: Some(RenderFlagSpec::Bool(true)), render_depth: Some(RenderFlagSpec::Bool(true)), render_seg: Some(RenderFlagSpec::Bool(true)), use_fast_math: false, use_shadows: true, enabled_geom_groups: &[0, 1, 2, 3], ..Default::default() }
}
#[rustfmt::skip]
pub fn assert_params_match_capture(params: &ContextParams<'_>, recipe: &Value) {
    assert_eq!(params.use_shadows, recipe["use_shadows"].as_bool().expect("bool"), "use_shadows");
    assert_eq!(params.use_fast_math, recipe["use_fast_math"].as_bool().expect("bool"), "use_fast_math");
    assert_eq!(params.samples_per_pixel as u64, recipe["samples_per_pixel"].as_u64().expect("u32"), "samples_per_pixel");
    assert_eq!(params.enabled_geom_groups, recipe["enabled_geom_groups"].as_array().expect("groups").iter().map(|v| v.as_i64().expect("i32") as i32).collect::<Vec<_>>().as_slice(), "enabled_geom_groups");
    assert!(matches!(params.cam_active, CameraActive::All), "cam_active");
    assert_eq!(params.background_color.to_vec(), recipe["background_color"].as_array().expect("bg").iter().map(|v| v.as_f64().expect("f64")).collect::<Vec<_>>(), "background_color");
}

#[rustfmt::skip]
pub fn camera_poses() -> Vec<CameraPose> {
    let poses = serde_json::from_str::<Value>(CAM_POSES).expect("multiframe/cam-poses.json")["poses"].clone();
    let bits = |name: &str| -> Vec<u32> { poses[name].as_array().expect(name).iter().map(|v| v.as_u64().expect("u32") as u32).collect() };
    let dec = |name: &str| -> Vec<f32> { poses[name].as_array().expect(name).iter().map(|v| v.as_f64().expect("f64") as f32).collect() };
    let (pos, mat, dpos, dmat) = (bits("cam_xpos_bits"), bits("cam_xmat_bits"), dec("cam_xpos"), dec("cam_xmat"));
    let ncam = pos.len() / 3;
    assert!(ncam > 0 && pos.len() % 3 == 0 && mat.len() == 9 * ncam, "D5 位姿 provenance 形状");
    (0..ncam).map(|c| {
        let xpos = [f32::from_bits(pos[3 * c]), f32::from_bits(pos[3 * c + 1]), f32::from_bits(pos[3 * c + 2])];
        let xmat: [f32; 9] = std::array::from_fn(|i| f32::from_bits(mat[9 * c + i]));
        assert_eq!(xpos, [dpos[3 * c], dpos[3 * c + 1], dpos[3 * c + 2]], "cam{c} xpos 位型/十进制");
        assert_eq!(xmat, std::array::from_fn(|i| dmat[9 * c + i]), "cam{c} xmat 位型/十进制");
        CameraPose { xpos, xmat }
    }).collect()
}

// ----------------------------------------------------------------------------------
// 逐帧判据（GPU 用例调用；CPU 侧只验证其数据前提）
// ----------------------------------------------------------------------------------

/// rgb：逐相机 `max_byte_diff <= 1` 且 `beyond_1_lsb == 0`。
#[rustfmt::skip]
pub fn assert_rgb(frame: &Frame, rendered: &[u32], layout: &ResolvedContext) {
    for camera in &frame.rgb {
        let start = layout.output.rgb_adr[camera.index] as usize;
        let stats = crate::s7b2::diff_rgb(&rendered[start..start + camera.pixels], camera);
        assert!(stats.max_byte_diff <= 1, "frame-{:03} cam{} rgb max_byte_diff={}", frame.index, camera.index, stats.max_byte_diff);
        assert_eq!(stats.beyond_one_lsb_pixels, 0, "frame-{:03} cam{} rgb 超 1 LSB", frame.index, camera.index);
    }
}

/// depth：frame-000 仅 cam2/(134,128)/flexid0 走 allowlist；frame-001/002 严格 bit-first（allowlist 漂移 → 大声失败）。
#[rustfmt::skip]
pub fn assert_depth(frame: &Frame, rendered: &[f32], layout: &ResolvedContext) {
    for camera in &frame.rgb {
        let start = layout.output.depth_adr[camera.index] as usize;
        let actual = &rendered[start..start + camera.pixels];
        let expected = &frame.depth_bits[camera.index];
        if frame.index == crate::multiframe::allowlist_frame() {
            crate::allowlist::print_depth_stats(camera.index, camera.width, actual, expected);
            crate::allowlist::assert_depth_allowlisted(camera.index, camera.width, actual, expected, &frame.seg[camera.index]);
        } else {
            // 001/002 若 allowlist 像素出现同像素同类型差 → 大声失败并停手（禁止自动扩 allowlist）。
            if camera.index == crate::allowlist::ALLOWLIST_CAMERA {
                let flat = crate::allowlist::ALLOWLIST_PIXEL_Y * camera.width + crate::allowlist::ALLOWLIST_PIXEL_X;
                let (bits, want) = (actual[flat].to_bits(), expected[flat]);
                if bits != want {
                    let seg = frame.seg[camera.index][flat];
                    let ulp = mjwarp_rs::render::kernel::output::ulp_distance(bits, want);
                    panic!("frame-{:03} allowlist 漂移：cam{} 像素 ({},{}) 出现同像素同类型差 —— bits={bits:#010x} expected={want:#010x} ulp={ulp} seg={seg:?}（停手报用户，禁止自动扩 allowlist）", frame.index, camera.index, crate::allowlist::ALLOWLIST_PIXEL_X, crate::allowlist::ALLOWLIST_PIXEL_Y);
                }
            }
            crate::allowlist::assert_depth_strict(camera.index, camera.width, actual, expected);
        }
    }
}

#[rustfmt::skip]
pub fn assert_seg(frame: &Frame, rendered: &[[i32; 2]], layout: &ResolvedContext) {
    for camera in &frame.rgb {
        let start = layout.output.seg_adr[camera.index] as usize;
        let mismatched = rendered[start..start + camera.pixels].iter().zip(&frame.seg[camera.index]).filter(|(a, e)| a != e).count();
        assert_eq!(mismatched, 0, "frame-{:03} cam{} seg 逐像素全等", frame.index, camera.index);
    }
}

pub fn allowlist_frame() -> usize {
    0
}
