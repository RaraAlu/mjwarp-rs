#![cfg(feature = "cuda-probe")]
//! R06-S3b③/⑥ flex 设备侧 GPU 验证：`#[ignore]` GPU 用例 + 普通 `cargo test` 可跑的 CPU 自证。
//! 参考 `scene-flex/reference/base/`（attempt E）+ `state.npz`；模型输入取 `scene-flex/model-312.json`。
//! GPU 判据零放宽：rgb beyond_1_lsb=0、depth max_ulp<=4 且 beyond=0、seg 全等（cam-flex 654 个
//! `(flexid,9)`）、两跑 sha256 相同。CPU 自证：model dump → flex fields → host 通道/BVH 输入逐数组断言。
//! GPU: cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1

#[path = "support/render_bvh/s7b1.rs"]
mod s7b1;
#[path = "support/render_bvh/s7b2.rs"]
mod s7b2;
#[path = "support/render_bvh/shadow.rs"]
mod shadow;

use mjwarp_rs::model::flex::{
    FlexEdgeFields, FlexPositionFields, FlexRenderFields, FlexTopologyFields,
};
use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::bvh::{
    FlexMeshAssembly, FlexMeshChannel, mesh_half_extents, vertices_from_f64,
};
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
const CAPTURE_JSON: &str =
    include_str!("../fixtures/render-bvh/scene-flex/reference/base/capture.json");

#[rustfmt::skip]
fn model() -> Value { serde_json::from_str(MODEL).expect("scene-flex/model-312.json") }
fn values<'a>(model: &'a Value, name: &str) -> &'a Vec<Value> {
    model["arrays"][name]["values"].as_array().expect("数组")
}
#[rustfmt::skip]
fn ints(model: &Value, name: &str) -> Vec<i32> { values(model, name).iter().map(|v| v.as_i64().expect("i32") as i32).collect() }
fn numbers(model: &Value, name: &str) -> Vec<f64> {
    values(model, name)
        .iter()
        .map(|v| v.as_f64().expect("f64"))
        .collect()
}
#[rustfmt::skip]
fn count(model: &Value, name: &str) -> usize { model["counts"][name].as_u64().expect("count") as usize }

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
    let model = model();
    let arrays = &model["arrays"];
    RenderFields::new(RENDER_FIELDS.iter().map(|spec| field_values(spec.kind, &arrays[spec.name]["values"])).collect())
}

#[rustfmt::skip]
fn f32_values(members: &[(String, Vec<u8>)], name: &str) -> (Vec<usize>, Vec<f32>) {
    let (descr, shape, data) = shadow::member(members, name);
    assert_eq!(descr, "<f4", "{name} descr");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    (shape, chunks.iter().map(|c| f32::from_le_bytes(*c)).collect())
}

#[rustfmt::skip]
type RefData = (Vec<s7b2::RgbCamera>, Vec<RefCamera>, Vec<usize>);

#[rustfmt::skip]
fn build_reference() -> RefData {
    let capture: Value = serde_json::from_str(CAPTURE_JSON).expect("capture.json");
    assert_eq!(capture["two_captures_bitwise_identical"], true);
    assert_eq!(capture["legacy_reference_untouched"], true);
    let (rgb_m, depth_m, seg_m) = (shadow::read_npz(RGB_NPZ), shadow::read_npz(DEPTH_NPZ), shadow::read_npz(SEG_NPZ));
    let cameras = rgb_m.len();
    assert_eq!((depth_m.len(), seg_m.len()), (cameras, cameras));
    let (mut rgb, mut depth_seg, mut flex_pixels) = (Vec::new(), Vec::new(), Vec::new());
    let (mut depth_adr, mut seg_adr) = (0i32, 0i32);
    for camera in 0..cameras {
        let ((h, w), rgb_f) = shadow::f32_member_rgb(&rgb_m, &format!("rgb_cam{camera}.npy"));
        let pixels = w * h;
        let mut packed = Vec::with_capacity(pixels);
        for i in 0..pixels {
            let q = |c: f32| shadow::quantize_channel(c, "rgb", "scene-flex", camera);
            let (r, g, b) = (q(rgb_f[3 * i]), q(rgb_f[3 * i + 1]), q(rgb_f[3 * i + 2]));
            packed.push((0xFFu32 << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
        }
        rgb.push(s7b2::RgbCamera { index: camera, width: w, height: h, pixels, packed });
        let ((dh, dw), depth_f) = shadow::f32_member(&depth_m, &format!("depth_cam{camera}.npy"));
        assert_eq!((dh, dw), (h, w), "depth 形状");
        let ((sh, sw, comp), seg_v) = shadow::i32_member(&seg_m, &format!("seg_cam{camera}.npy"));
        assert_eq!((sh, sw, comp), (h, w, 2), "seg 形状");
        let (pairs, rem) = seg_v.as_chunks::<2>();
        assert!(rem.is_empty());
        let seg_pairs: Vec<[i32; 2]> = pairs.to_vec();
        flex_pixels.push(seg_pairs.iter().filter(|p| p[1] == 9).count());
        let depth_bits: Vec<u32> = depth_f.iter().map(|v| v.to_bits()).collect();
        depth_seg.push(RefCamera {
            index: camera, width: w, height: h, pixels, depth_adr, seg_adr,
            depth_zero_pixels: depth_bits.iter().filter(|b| **b == 0).count(),
            depth_max_hex: "recorded".to_string(), depth_bits, seg_pairs, seg_unique: Vec::new(),
        });
        depth_adr += pixels as i32;
        seg_adr += pixels as i32;
    }
    (rgb, depth_seg, flex_pixels)
}

#[rustfmt::skip]
fn scene_meshes(model: &Value) -> Vec<MeshState> {
    let (vert, face) = (numbers(model, "mesh_vert"), ints(model, "mesh_face"));
    let (vadr, vnum) = (ints(model, "mesh_vertadr"), ints(model, "mesh_vertnum"));
    let (fadr, fnum) = (ints(model, "mesh_faceadr"), ints(model, "mesh_facenum"));
    (0..vadr.len()).map(|mesh| {
        let start = vadr[mesh] as usize * 3;
        let vertices = vertices_from_f64(&vert[start..start + vnum[mesh] as usize * 3]).expect("顶点");
        let face_start = fadr[mesh] as usize * 3;
        let indices = face[face_start..face_start + fnum[mesh] as usize * 3].iter().map(|v| *v as u32).collect();
        MeshState { vertices, indices }
    }).collect()
}

#[rustfmt::skip]
fn scene_fixture(model: &Value, state: &[(String, Vec<u8>)]) -> (Vec<SceneLeaf>, Vec<LeafPose>, Vec<MeshState>) {
    let meshes = scene_meshes(model);
    let mesh_half: Vec<[f32; 3]> = meshes.iter().map(|m| mesh_half_extents(&m.vertices).expect("bounds")).collect();
    let (geom_type, geom_size, geom_dataid) = (ints(model, "geom_type"), numbers(model, "geom_size"), ints(model, "geom_dataid"));
    let (shape, xpos) = f32_values(state, "geom_xpos.npy");
    let (mat_shape, xmat) = f32_values(state, "geom_xmat.npy");
    let ngeom = geom_type.len();
    assert_eq!((shape, mat_shape), (vec![1, ngeom, 3], vec![1, ngeom, 3, 3]));
    let mut leaves = Vec::with_capacity(ngeom);
    let mut poses = Vec::with_capacity(ngeom);
    for geom in 0..ngeom {
        let mut size = [geom_size[3 * geom] as f32, geom_size[3 * geom + 1] as f32, geom_size[3 * geom + 2] as f32];
        if matches!(geom_type[geom], 7 | 8) { size = mesh_half[geom_dataid[geom] as usize]; }
        leaves.push(SceneLeaf { geom_type: geom_type[geom], size, group: 0, geom_id: geom as i32 });
        let mut matrix = [0.0f32; 9];
        matrix.copy_from_slice(&xmat[9 * geom..9 * geom + 9]);
        poses.push(LeafPose { xpos: [xpos[3 * geom], xpos[3 * geom + 1], xpos[3 * geom + 2]], xmat: matrix });
    }
    (leaves, poses, meshes)
}

#[rustfmt::skip]
fn flex_model(model: &Value) -> FlexRenderFields {
    FlexRenderFields {
        nflex: count(model, "nflex"), nflexvert: count(model, "nflexvert"), nflexedge: count(model, "nflexedge"),
        flex_dim: ints(model, "flex_dim"),
        flex_radius: numbers(model, "flex_radius").iter().map(|v| *v as f32).collect(),
        flex_rgba: numbers(model, "flex_rgba").iter().map(|v| *v as f32).collect(),
        flex_edgeadr: ints(model, "flex_edgeadr"), flex_edgenum: ints(model, "flex_edgenum"),
    }
}

#[rustfmt::skip]
fn flex_positions(model: &Value) -> FlexPositionFields {
    FlexPositionFields { flex_vertadr: ints(model, "flex_vertadr"), flex_vertnum: ints(model, "flex_vertnum"), flex_vertbodyid: ints(model, "flex_vertbodyid"), ..Default::default() }
}

/// `model-312.json` 不含原生 `flex_edge` 数组（dump 字段清单未列它）：dim==1 时元素就是边
/// （上游 `mjmodel.h`：`flex_elem` 存 `dim+1 = 2` 个顶点 id/元素，元素数 == 边数），故按
/// `flex_elem` 派生；四条交叉校验确认派生前提：`nflexelemdata == 2*nflexedge`、
/// `sum(edgenum) == nflexedge`、`edgeadr[0] == 0`、端点 < 各自 `flex_vertnum`。
#[rustfmt::skip]
fn flex_edges(model: &Value) -> FlexEdgeFields {
    let (nflexvert, nflexelemdata, nflexedge) = (count(model, "nflexvert"), count(model, "nflexelemdata"), count(model, "nflexedge"));
    let (flex_edgeadr, flex_edgenum) = (ints(model, "flex_edgeadr"), ints(model, "flex_edgenum"));
    let (flex_vertadr, flex_vertnum) = (ints(model, "flex_vertadr"), ints(model, "flex_vertnum"));
    let flex_edge = ints(model, "flex_elem");
    assert_eq!(nflexelemdata, 2 * nflexedge, "dim==1 元素==边");
    assert_eq!(flex_edge.len(), nflexedge * 2);
    assert_eq!(flex_edgeadr.first(), Some(&0));
    assert_eq!(flex_edgenum.iter().sum::<i32>() as usize, nflexedge);
    for flex in 0..flex_edgeadr.len() {
        let (start, verts) = (flex_edgeadr[flex] as usize, flex_vertnum[flex] as usize);
        assert!((flex_vertadr[flex] as usize) < nflexvert);
        for edge in start..start + flex_edgenum[flex] as usize {
            for slot in 0..2 {
                let local = *flex_edge.get(edge * 2 + slot).expect("edge");
                assert!((0..verts as i32).contains(&local), "顶点号越界");
            }
        }
    }
    // 稀疏行同源推导：每条边一行、行内 2 个非零（两个端点），`flexedge_j_colind` 即端点顶点号；
    // `FlexEdgeFields::nflexedge()` = `rownnz.len()`，故宿主校验据此与 model `nflexedge` 对齐。
    let mut rowadr = Vec::with_capacity(nflexedge);
    let mut acc = 0i32;
    for _ in 0..nflexedge { rowadr.push(acc); acc += 2; }
    FlexEdgeFields {
        flex_edgeadr, flex_edgenum, flex_edge: flex_edge.clone(), flexedge_j_rowadr: rowadr,
        flexedge_j_rownnz: vec![2; nflexedge], flexedge_j_colind: flex_edge
    }
}

#[rustfmt::skip]
fn flex_topology(model: &Value) -> FlexTopologyFields {
    FlexTopologyFields {
        nflex: count(model, "nflex"),
        nflexelem: count(model, "nflexelem"), nflexelemdata: count(model, "nflexelemdata"),
        nflexshelldata: count(model, "nflexshelldata"),
        flex_elemadr: ints(model, "flex_elemadr"), flex_elemnum: ints(model, "flex_elemnum"),
        flex_elemdataadr: ints(model, "flex_elemdataadr"), flex_elem: ints(model, "flex_elem"),
        flex_shellnum: ints(model, "flex_shellnum"), flex_shelldataadr: ints(model, "flex_shelldataadr"),
        flex_shell: ints(model, "flex_shell")
    }
}

#[rustfmt::skip]
fn camera_poses(model: &Value) -> Vec<CameraPose> {
    let (pos, mat) = (numbers(model, "cam_pos0"), numbers(model, "cam_mat0"));
    (0..pos.len() / 3).map(|c| {
        let mut xmat = [0.0f32; 9];
        for (i, slot) in xmat.iter_mut().enumerate() { *slot = mat[9 * c + i] as f32; }
        CameraPose { xpos: [pos[3 * c] as f32, pos[3 * c + 1] as f32, pos[3 * c + 2] as f32], xmat }
    }).collect()
}

#[rustfmt::skip]
type Output = (Vec<u32>, Vec<f32>, Vec<[i32; 2]>);

#[rustfmt::skip]
fn hash_rendered(r: &Output) -> (String, String, String) {
    let join = |w: Vec<[u8; 4]>| w.concat();
    let rgb = join(r.0.iter().map(|v| v.to_le_bytes()).collect());
    let depth = join(r.1.iter().map(|v| v.to_bits().to_le_bytes()).collect());
    let seg = join(r.2.iter().flat_map(|p| [p[0].to_le_bytes(), p[1].to_le_bytes()]).collect());
    (shadow::sha256_hex(&rgb), shadow::sha256_hex(&depth), shadow::sha256_hex(&seg))
}

#[allow(clippy::too_many_arguments)]
#[rustfmt::skip]
fn render_once(renderer: &MegakernelRenderer, context: &RenderContext, fields: &RenderFields, model: &CameraModel, poses: &[CameraPose], rays: &RayBuffer, assembly: &FlexHostAssembly<'_>) -> Output {
    assembly.render_with(renderer, context, fields, model, poses, rays, None).expect("flex 渲染发射");
    (s7b2::read_rgb(context), s7b1::read_depth(context), s7b1::read_seg(context))
}

#[rustfmt::skip]
fn compare(reference: &RefData, layout: &ResolvedContext, rendered: &Output) {
    for camera in &reference.0 {
        let start = layout.output.rgb_adr[camera.index] as usize;
        let stats = s7b2::diff_rgb(&rendered.0[start..start + camera.pixels], camera);
        assert!(stats.max_byte_diff <= 1, "cam{} rgb max_byte_diff={}", camera.index, stats.max_byte_diff);
        assert_eq!(stats.beyond_one_lsb_pixels, 0, "cam{} rgb 超 1 LSB", camera.index);
    }
    for camera in &reference.1 {
        let start = layout.output.depth_adr[camera.index] as usize;
        let (mut mismatched, mut max_ulp, mut beyond) = (0usize, 0i64, 0usize);
        for (value, expected) in rendered.1[start..start + camera.pixels].iter().zip(&camera.depth_bits) {
            let bits = value.to_bits();
            mismatched += usize::from(bits != *expected);
            let ulp = ulp_distance(bits, *expected);
            max_ulp = max_ulp.max(ulp);
            beyond += usize::from(ulp > MAX_DEPTH_ULP);
        }
        assert_eq!((mismatched, beyond), (0, 0), "cam{} depth 位级/超 4 ULP", camera.index);
        assert!(max_ulp <= MAX_DEPTH_ULP, "cam{} depth max_ulp={}", camera.index, max_ulp);
        let seg_start = layout.output.seg_adr[camera.index] as usize;
        let seg_mismatched = rendered.2[seg_start..seg_start + camera.pixels].iter().zip(&camera.seg_pairs).filter(|(a, e)| a != e).count();
        assert_eq!(seg_mismatched, 0, "cam{} seg 逐像素全等", camera.index);
    }
}

/// CPU 自证（普通 `cargo test`，不碰 CUDA）：model dump → flex 三字段结构 → host 通道/BVH 输入。
#[test]
#[rustfmt::skip]
fn flex_model_dump_schema_self_check() {
    let model = model();
    let (nflex, nflexvert, nflexedge) = (count(&model, "nflex"), count(&model, "nflexvert"), count(&model, "nflexedge"));
    let expectations: [(&str, usize, &str); 17] = [
        ("flex_dim", nflex, "int32"), ("flex_radius", nflex, "float64"), ("flex_rgba", 4 * nflex, "float32"),
        ("flex_vertadr", nflex, "int32"), ("flex_vertnum", nflex, "int32"), ("flex_vertbodyid", nflexvert, "int32"),
        ("flex_edgeadr", nflex, "int32"), ("flex_edgenum", nflex, "int32"),
        ("flex_elem", count(&model, "nflexelemdata"), "int32"),
        ("flex_elemadr", nflex, "int32"), ("flex_elemnum", nflex, "int32"), ("flex_elemdataadr", nflex, "int32"),
        ("flex_shellnum", nflex, "int32"), ("flex_shelldataadr", nflex, "int32"),
        ("flex_shell", count(&model, "nflexshelldata"), "int32"),
        ("flex_vert", 3 * nflexvert, "float64"), ("flex_vert0", 3 * nflexvert, "float64"),
    ];
    for (name, expected, kind) in expectations {
        let array = &model["arrays"][name];
        assert_eq!(array["kind"], kind, "{name} kind");
        assert_eq!(array["count"].as_u64().expect("count") as usize, expected, "{name} count");
        assert_eq!(values(&model, name).len(), expected, "{name} values");
    }
    let (fields, positions, edges) = (flex_model(&model), flex_positions(&model), flex_edges(&model));
    assert_eq!((fields.nflex, fields.nflexvert, fields.nflexedge), (2, 6, nflexedge));
    assert_eq!(edges.flex_edge, [0, 1, 1, 2, 2, 3, 0, 1], "1D 边 == flex_elem");
    assert_eq!(positions.flex_vertnum.iter().sum::<i32>(), 6);
    let state = shadow::read_npz(STATE_NPZ);
    let (shape, flexvert) = f32_values(&state, "flexvert_xpos.npy");
    assert_eq!(shape, vec![1, 6, 3]);
    let assembly = FlexHostAssembly::build(1, &positions, &edges, &fields, FlexHostData { flexvert_xpos: &flexvert, flex_bvh_ids: &[u64::MAX; 2], flex_group_root: &[-1i32; 2] })
        .expect("CPU 组装").expect("有 flex");
    let inputs = assembly.inputs();
    assert_eq!(inputs.flex_geom_edgeid, [0, 1, 2, 3]);
    let topology = flex_topology(&model);
    let channel = FlexMeshChannel {
        topology: &topology, flex_dim: &fields.flex_dim, flex_vertadr: &positions.flex_vertadr,
        flex_radius: &fields.flex_radius, flexvert_xpos: &flexvert, flex_edge: &edges.flex_edge,
        geom_flexid: inputs.flex_geom_flexid, geom_edgeid: inputs.flex_geom_edgeid, worlds: 1, nflexvert,
    };
    let mesh = FlexMeshAssembly::build(&channel).expect("BVH 输入组装");
    assert_eq!(mesh.mesh_count, 0, "1D 场景无 2D/3D mesh 段");
    assert_eq!(mesh.leaves.bounds.len(), 4, "nflexgeom 个胶囊叶");
    assert_eq!(mesh.bvh_ids, [u64::MAX; 2]);
}

#[test]
#[ignore = "需要 GPU + NVRTC（开窗卡执行）"]
#[rustfmt::skip]
fn flex_renders_match_captured_reference() {
    let reference = build_reference();
    assert_eq!(reference.2, vec![0, 0, 654], "flex 像素只在 cam-flex");
    let model = model();
    let fields = render_fields();
    let session = s7b1::session();
    let state = shadow::read_npz(STATE_NPZ);
    let (leaves, leaf_poses, meshes) = scene_fixture(&model, &state);
    let params = ContextParams { nworld: 1, ..Default::default() }.with_render_targets(Some(RenderFlagSpec::Bool(true)), Some(RenderFlagSpec::Bool(true)), Some(RenderFlagSpec::Bool(true)));
    let resolved = resolve_context(&fields, &params).expect("context 解析");
    let context = RenderContext::create(&session, resolved, SceneState::new(leaves, leaf_poses, 1).expect("场景状态"), meshes, &TextureRegistry::from_fields(&fields, true).expect("纹理")).expect("context 创建");
    let camera_model = CameraModel::from_fields(&fields).expect("相机模型");
    let poses = camera_poses(&model);
    let rays = s7b1::precomputed_rays(&session, &fields);
    let renderer = MegakernelRenderer::new(&session).expect("megakernel 编译");
    let (_, flexvert) = f32_values(&state, "flexvert_xpos.npy");
    let (positions, edges, flex_fields) = (flex_positions(&model), flex_edges(&model), flex_model(&model));
    let assembly = FlexHostAssembly::build(1, &positions, &edges, &flex_fields, FlexHostData { flexvert_xpos: &flexvert, flex_bvh_ids: &[u64::MAX; 2], flex_group_root: &[-1i32; 2] }).expect("组装").expect("有 flex");
    let first = render_once(&renderer, &context, &fields, &camera_model, &poses, &rays, &assembly);
    let second = render_once(&renderer, &context, &fields, &camera_model, &poses, &rays, &assembly);
    let (h1, h2) = (hash_rendered(&first), hash_rendered(&second));
    assert_eq!(h1, h2, "两次运行 sha256 必须相同");
    compare(&reference, context.resolved(), &first);
    println!("r06-s3b③[flex] two-run sha256 identical rgb={} depth={} seg={}", h1.0, h1.1, h1.2);
}
