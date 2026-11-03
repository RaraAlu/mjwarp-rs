#![cfg(feature = "cuda-probe")]
//! R06-S3b③ flex 设备侧 GPU 验证（窗口内执行；本文件只做作者 + 编译验证，不开窗）。
//! 参考 `scene-flex/reference/base/`（attempt E）+ `state.npz`；判据零放宽：rgb beyond_1_lsb=0、
//! depth max_ulp<=4 且 beyond=0、seg 全等（cam-flex 654 个 `(flexid,9)`）、两跑 sha256 相同。
#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;

use mjwarp_rs::model::flex::{FlexEdgeFields, FlexPositionFields, FlexRenderFields};
use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{mesh_half_extents, vertices_from_f64};
use mjwarp_rs::render::context::flex_host::{FlexHostAssembly, FlexHostData};
use mjwarp_rs::render::context::owner::MeshState;
use mjwarp_rs::render::context::{RenderFlagSpec, ResolvedContext};
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, RefCamera, ulp_distance};
use mjwarp_rs::render::{
    CameraModel, ContextParams, LeafPose, RayBuffer, RenderContext, SceneLeaf, SceneState,
    TextureRegistry, resolve_context,
};
use serde_json::Value;
const MODEL: &str = include_str!("../fixtures/render-bvh/scene-flex/model-312.json");
const RGB_NPZ: &[u8] = include_bytes!("../fixtures/render-bvh/scene-flex/reference/base/rgb.npz");
const DEPTH_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-flex/reference/base/depth.npz");
const SEG_NPZ: &[u8] = include_bytes!("../fixtures/render-bvh/scene-flex/reference/base/seg.npz");
const STATE_NPZ: &[u8] =
    include_bytes!("../fixtures/render-bvh/scene-flex/reference/base/state.npz");
/// 参考 `capture.json`（provenance 自证：两跑一致 / legacy 未动 / flex 覆盖）。
const CAPTURE_JSON: &str =
    include_str!("../fixtures/render-bvh/scene-flex/reference/base/capture.json");

fn model() -> Value {
    serde_json::from_str(MODEL).expect("scene-flex/model-312.json")
}

fn ints(model: &Value, name: &str) -> Vec<i32> {
    values(model, name)
        .iter()
        .map(|v| v.as_i64().expect("i32") as i32)
        .collect()
}

fn numbers(model: &Value, name: &str) -> Vec<f64> {
    values(model, name)
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect()
}

fn values<'a>(model: &'a Value, name: &str) -> &'a Vec<Value> {
    model["arrays"][name]["values"].as_array().expect("数组")
}

fn field_values(kind: RenderFieldKind, value: &Value) -> FieldData {
    let values = value.as_array().expect("数组");
    match kind {
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
}

fn render_fields() -> RenderFields {
    let model = model();
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| field_values(spec.kind, &arrays[spec.name]["values"]))
        .collect();
    RenderFields::new(data)
}

/// 参考侧结构（复用 shadow 支持模块的 npz/npy 解析与量化）。
struct Reference {
    rgb: Vec<s7b2::RgbCamera>,
    depth_seg: Vec<RefCamera>,
    flex_pixels: Vec<usize>,
}

fn f32_values(members: &[(String, Vec<u8>)], name: &str) -> (Vec<usize>, Vec<f32>) {
    let (descr, shape, data) = shadow::member(members, name);
    assert_eq!(descr, "<f4", "{name} descr");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    let values: Vec<f32> = chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect();
    (shape, values)
}

fn build_reference() -> Reference {
    let capture: Value = serde_json::from_str(CAPTURE_JSON).expect("capture.json");
    assert_eq!(capture["two_captures_bitwise_identical"], true);
    assert_eq!(capture["legacy_reference_untouched"], true);
    let rgb_members = shadow::read_npz(RGB_NPZ);
    let depth_members = shadow::read_npz(DEPTH_NPZ);
    let seg_members = shadow::read_npz(SEG_NPZ);
    let cameras = rgb_members.len();
    assert_eq!(depth_members.len(), cameras);
    assert_eq!(seg_members.len(), cameras);
    let mut rgb = Vec::with_capacity(cameras);
    let mut depth_seg = Vec::with_capacity(cameras);
    let mut flex_pixels = Vec::with_capacity(cameras);
    let mut depth_adr = 0i32;
    let mut seg_adr = 0i32;
    for camera in 0..cameras {
        let ((height, width), values) =
            shadow::f32_member_rgb(&rgb_members, &format!("rgb_cam{camera}.npy"));
        let pixels = width * height;
        let mut packed = Vec::with_capacity(pixels);
        for index in 0..pixels {
            let base = 3 * index;
            let r = shadow::quantize_channel(values[base], "rgb.r", "scene-flex", camera);
            let g = shadow::quantize_channel(values[base + 1], "rgb.g", "scene-flex", camera);
            let b = shadow::quantize_channel(values[base + 2], "rgb.b", "scene-flex", camera);
            packed
                .push((0xFFu32 << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
        }
        rgb.push(s7b2::RgbCamera {
            index: camera,
            width,
            height,
            pixels,
            packed,
        });
        let ((depth_height, depth_width), depth_values) =
            shadow::f32_member(&depth_members, &format!("depth_cam{camera}.npy"));
        assert_eq!((depth_height, depth_width), (height, width), "depth 形状");
        let depth_bits: Vec<u32> = depth_values.iter().map(|value| value.to_bits()).collect();
        let ((seg_height, seg_width, components), seg_values) =
            shadow::i32_member(&seg_members, &format!("seg_cam{camera}.npy"));
        assert_eq!(
            (seg_height, seg_width, components),
            (height, width, 2),
            "seg 形状"
        );
        let (pairs, remainder) = seg_values.as_chunks::<2>();
        assert!(remainder.is_empty());
        let seg_pairs: Vec<[i32; 2]> = pairs.to_vec();
        flex_pixels.push(seg_pairs.iter().filter(|pair| pair[1] == 9).count());
        let mut seg_unique = seg_pairs.clone();
        seg_unique.sort_unstable();
        seg_unique.dedup();
        depth_seg.push(RefCamera {
            index: camera,
            width,
            height,
            pixels,
            depth_adr,
            seg_adr,
            depth_zero_pixels: depth_bits.iter().filter(|bits| **bits == 0).count(),
            depth_max_hex: "0x00000000".to_string(),
            depth_bits,
            seg_pairs,
            seg_unique,
        });
        depth_adr += pixels as i32;
        seg_adr += pixels as i32;
    }
    Reference {
        rgb,
        depth_seg,
        flex_pixels,
    }
}

/// 场景（刚性几何叶）+ 网格资源；世界位姿取参考 `state.npz` 的 `geom_xpos`/`geom_xmat`。
fn scene_meshes(model: &Value) -> Vec<MeshState> {
    let mesh_vert = numbers(model, "mesh_vert");
    let mesh_face = ints(model, "mesh_face");
    let vertadr = ints(model, "mesh_vertadr");
    let vertnum = ints(model, "mesh_vertnum");
    let faceadr = ints(model, "mesh_faceadr");
    let facenum = ints(model, "mesh_facenum");
    let mut meshes = Vec::new();
    for mesh in 0..vertadr.len() {
        let start = vertadr[mesh] as usize * 3;
        let vertices = vertices_from_f64(&mesh_vert[start..start + vertnum[mesh] as usize * 3])
            .expect("网格顶点");
        let face_start = faceadr[mesh] as usize * 3;
        let indices = mesh_face[face_start..face_start + facenum[mesh] as usize * 3]
            .iter()
            .map(|value| *value as u32)
            .collect();
        meshes.push(MeshState { vertices, indices });
    }
    meshes
}

fn scene_fixture(
    model: &Value,
    state_members: &[(String, Vec<u8>)],
) -> (Vec<SceneLeaf>, Vec<LeafPose>, Vec<MeshState>) {
    let meshes = scene_meshes(model);
    let mesh_half: Vec<[f32; 3]> = meshes
        .iter()
        .map(|mesh| mesh_half_extents(&mesh.vertices).expect("网格 bounds"))
        .collect();
    let geom_type = ints(model, "geom_type");
    let geom_size = numbers(model, "geom_size");
    let geom_dataid = ints(model, "geom_dataid");
    let (shape, xpos) = f32_values(state_members, "geom_xpos.npy");
    let (mat_shape, xmat) = f32_values(state_members, "geom_xmat.npy");
    let ngeom = geom_type.len();
    assert_eq!(shape, vec![1, ngeom, 3]);
    assert_eq!(mat_shape, vec![1, ngeom, 3, 3]);
    let mut leaves = Vec::with_capacity(ngeom);
    let mut poses = Vec::with_capacity(ngeom);
    for geom in 0..ngeom {
        let mut size = [
            geom_size[3 * geom] as f32,
            geom_size[3 * geom + 1] as f32,
            geom_size[3 * geom + 2] as f32,
        ];
        if matches!(geom_type[geom], 7 | 8) {
            size = mesh_half[geom_dataid[geom] as usize];
        }
        leaves.push(SceneLeaf {
            geom_type: geom_type[geom],
            size,
            group: 0,
            geom_id: geom as i32,
        });
        let mut matrix = [0.0f32; 9];
        matrix.copy_from_slice(&xmat[9 * geom..9 * geom + 9]);
        poses.push(LeafPose {
            xpos: [xpos[3 * geom], xpos[3 * geom + 1], xpos[3 * geom + 2]],
            xmat: matrix,
        });
    }
    (leaves, poses, meshes)
}

fn flex_model(model: &Value) -> FlexRenderFields {
    let counts = &model["counts"];
    FlexRenderFields {
        nflex: counts["nflex"].as_u64().expect("nflex") as usize,
        nflexvert: counts["nflexvert"].as_u64().expect("nflexvert") as usize,
        nflexedge: counts["nflexedge"].as_u64().expect("nflexedge") as usize,
        flex_dim: ints(model, "flex_dim"),
        flex_radius: numbers(model, "flex_radius")
            .iter()
            .map(|v| *v as f32)
            .collect(),
        flex_rgba: numbers(model, "flex_rgba")
            .iter()
            .map(|v| *v as f32)
            .collect(),
        flex_edgeadr: ints(model, "flex_edgeadr"),
        flex_edgenum: ints(model, "flex_edgenum"),
    }
}

fn flex_positions(model: &Value) -> FlexPositionFields {
    FlexPositionFields {
        flex_vertadr: ints(model, "flex_vertadr"),
        flex_vertnum: ints(model, "flex_vertnum"),
        flex_vertbodyid: ints(model, "flex_vertbodyid"),
        ..Default::default()
    }
}

fn flex_edges(model: &Value) -> FlexEdgeFields {
    FlexEdgeFields {
        flex_edgeadr: ints(model, "flex_edgeadr"),
        flex_edgenum: ints(model, "flex_edgenum"),
        flex_edge: ints(model, "flex_edge"),
        flexedge_j_rownnz: vec![1; ints(model, "flex_edge").len() / 2],
        ..Default::default()
    }
}

/// 相机世界位姿：`cam_pos0`/`cam_mat0`（MuJoCo 3.12 编译期初始世界位姿，qpos0 即本状态）。
fn camera_poses(model: &Value) -> Vec<CameraPose> {
    let pos = numbers(model, "cam_pos0");
    let mat = numbers(model, "cam_mat0");
    (0..pos.len() / 3)
        .map(|camera| {
            let mut xmat = [0.0f32; 9];
            for (index, slot) in xmat.iter_mut().enumerate() {
                *slot = mat[9 * camera + index] as f32;
            }
            CameraPose {
                xpos: [
                    pos[3 * camera] as f32,
                    pos[3 * camera + 1] as f32,
                    pos[3 * camera + 2] as f32,
                ],
                xmat,
            }
        })
        .collect()
}

type Output = (Vec<u32>, Vec<f32>, Vec<[i32; 2]>);

fn hash_rendered(rendered: &Output) -> (String, String, String) {
    let (rgb_u32, depth_f32, seg_pairs) = rendered;
    let join = |words: Vec<[u8; 4]>| words.concat();
    let rgb = join(rgb_u32.iter().map(|value| value.to_le_bytes()).collect());
    let depth = join(
        depth_f32
            .iter()
            .map(|value| value.to_bits().to_le_bytes())
            .collect(),
    );
    let seg = join(
        seg_pairs
            .iter()
            .flat_map(|pair| [pair[0].to_le_bytes(), pair[1].to_le_bytes()])
            .collect(),
    );
    (
        shadow::sha256_hex(&rgb),
        shadow::sha256_hex(&depth),
        shadow::sha256_hex(&seg),
    )
}

#[allow(clippy::too_many_arguments)]
fn render_once(
    renderer: &MegakernelRenderer,
    context: &RenderContext,
    fields: &RenderFields,
    model: &CameraModel,
    poses: &[CameraPose],
    rays: &RayBuffer,
    assembly: &FlexHostAssembly<'_>,
) -> Output {
    assembly
        .render_with(renderer, context, fields, model, poses, rays, None)
        .expect("flex 渲染发射");
    (
        s7b2::read_rgb(context),
        s7b1::read_depth(context),
        s7b1::read_seg(context),
    )
}

fn compare(reference: &Reference, layout: &ResolvedContext, rendered: &Output) {
    let (rgb_out, depth_out, seg_out) = rendered;
    for camera in &reference.rgb {
        let start = layout.output.rgb_adr[camera.index] as usize;
        let stats = s7b2::diff_rgb(&rgb_out[start..start + camera.pixels], camera);
        assert!(
            stats.max_byte_diff <= 1,
            "cam{} rgb max_byte_diff={}",
            camera.index,
            stats.max_byte_diff
        );
        assert_eq!(
            stats.beyond_one_lsb_pixels, 0,
            "cam{} rgb 超 1 LSB",
            camera.index
        );
    }
    for camera in &reference.depth_seg {
        let start = layout.output.depth_adr[camera.index] as usize;
        let (mut mismatched, mut max_ulp, mut beyond) = (0usize, 0i64, 0usize);
        for (value, expected) in depth_out[start..start + camera.pixels]
            .iter()
            .zip(&camera.depth_bits)
        {
            let bits = value.to_bits();
            if bits != *expected {
                mismatched += 1;
            }
            let ulp = ulp_distance(bits, *expected);
            max_ulp = max_ulp.max(ulp);
            if ulp > MAX_DEPTH_ULP {
                beyond += 1;
            }
        }
        assert_eq!(mismatched, 0, "cam{} depth 位级必须相同", camera.index);
        assert!(
            max_ulp <= MAX_DEPTH_ULP,
            "cam{} depth max_ulp={}",
            camera.index,
            max_ulp
        );
        assert_eq!(beyond, 0, "cam{} depth 超 4 ULP 像素", camera.index);
        let seg_start = layout.output.seg_adr[camera.index] as usize;
        let seg_mismatched = seg_out[seg_start..seg_start + camera.pixels]
            .iter()
            .zip(&camera.seg_pairs)
            .filter(|(a, e)| a != e)
            .count();
        assert_eq!(seg_mismatched, 0, "cam{} seg 逐像素全等", camera.index);
        println!(
            "r06-s3b③[flex] cam{} depth max_ulp={max_ulp} seg_mismatched=0",
            camera.index
        );
    }
}

#[test]
#[ignore = "需要 GPU + NVRTC（开窗卡执行）"]
fn flex_renders_match_captured_reference() {
    let reference = build_reference();
    assert_eq!(
        reference.flex_pixels,
        vec![0, 0, 654],
        "flex 像素只在 cam-flex"
    );
    let model = model();
    let fields = render_fields();
    let session = s7b1::session();
    let state_members = shadow::read_npz(STATE_NPZ);
    let (leaves, poses_leaf, meshes) = scene_fixture(&model, &state_members);
    let params = ContextParams {
        nworld: 1,
        ..Default::default()
    }
    .with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(true)),
    );
    let resolved = resolve_context(&fields, &params).expect("context 解析");
    let state = SceneState::new(leaves, poses_leaf, 1).expect("场景状态");
    let textures = TextureRegistry::from_fields(&fields, true).expect("纹理注册表");
    let context =
        RenderContext::create(&session, resolved, state, meshes, &textures).expect("context 创建");
    let camera_model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_poses(&model);
    let rays = s7b1::precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("megakernel 编译");
    let (_, flexvert) = f32_values(&state_members, "flexvert_xpos.npy");
    let flex_model = flex_model(&model);
    let flex_positions = flex_positions(&model);
    let flex_edges = flex_edges(&model);
    let bvh_ids = [u64::MAX; 2];
    let assembly = FlexHostAssembly::build(
        1,
        &flex_positions,
        &flex_edges,
        &flex_model,
        FlexHostData {
            flexvert_xpos: &flexvert,
            flex_bvh_ids: &bvh_ids,
            flex_group_root: &[-1i32; 2],
        },
    )
    .expect("flex 宿主组装")
    .expect("有 flex 必须 Some");
    let first = render_once(
        &renderer,
        &context,
        &fields,
        &camera_model,
        &poses,
        &rays,
        &assembly,
    );
    let second = render_once(
        &renderer,
        &context,
        &fields,
        &camera_model,
        &poses,
        &rays,
        &assembly,
    );
    let first_hash = hash_rendered(&first);
    let second_hash = hash_rendered(&second);
    assert_eq!(first_hash, second_hash, "两次运行 sha256 必须相同");
    let layout = context.resolved();
    compare(&reference, layout, &first);
    println!(
        "r06-s3b③[flex] two-run sha256 identical rgb={} depth={} seg={}",
        first_hash.0, first_hash.1, first_hash.2
    );
}
