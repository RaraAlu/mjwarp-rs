//! 渲染/BVH S6a 验收：render context 28 参数解析、输出偏移/大小与拒绝面（纯宿主）。
//!
//! 参考：`fixtures/render-bvh/scene-a/reference/context.json`（S0b 冻结 MJWarp 3.15 的
//! `create_render_context` 参数与解析结果）与 S1 的 `model-312.json` 渲染字段。
//! 只读静态夹具，不启动 Python、不占 GPU。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::CameraError;
use mjwarp_rs::render::camera::CameraActive;
use mjwarp_rs::render::context::{
    CamRes, CameraBranch, ContextError, ContextParams, RenderFlagSpec, SplatInputs, resolve_context,
};
use serde_json::Value;

const CONTEXT: &str = include_str!("../fixtures/render-bvh/scene-a/reference/context.json");
const MODEL: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");

fn reference() -> Value {
    serde_json::from_str(CONTEXT).expect("S0b context.json")
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON 数组")
}

fn int_list(value: &Value) -> Vec<i32> {
    array(value)
        .iter()
        .map(|v| v.as_i64().expect("int") as i32)
        .collect()
}

fn f64_list(value: &Value) -> Vec<f64> {
    array(value)
        .iter()
        .map(|v| v.as_f64().expect("float"))
        .collect()
}

fn bool_list(value: &Value) -> Vec<bool> {
    array(value)
        .iter()
        .map(|v| v.as_bool().expect("bool"))
        .collect()
}

fn flag_spec(value: &Value) -> Option<RenderFlagSpec<'_>> {
    if value.is_null() {
        None
    } else {
        Some(RenderFlagSpec::Bool(value.as_bool().expect("render flag")))
    }
}

fn model_ints<'a>(fields: &'a RenderFields, name: &'static str) -> &'a [i32] {
    match fields.field(name) {
        Some(FieldData::Index(values)) => values,
        other => panic!("{name} 不是 Index: {other:?}"),
    }
}

/// 把 S1 渲染字段夹具还原成 116 字段快照。
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

#[test]
fn default_call_resolves_like_context_json() {
    let reference = reference();
    let parameters = &reference["create_render_context_parameters"];
    assert_eq!(parameters.as_object().expect("params").len(), 28);
    let groups = int_list(&parameters["enabled_geom_groups"]);
    let params = ContextParams {
        render_rgb: flag_spec(&parameters["render_rgb"]),
        render_depth: flag_spec(&parameters["render_depth"]),
        render_seg: flag_spec(&parameters["render_seg"]),
        enabled_geom_groups: &groups,
        ..ContextParams::default()
    };
    let resolved = resolve_context(&render_fields(), &params).expect("默认调用解析");
    let expected = &reference["resolved"];

    assert_eq!(
        resolved.nrender(),
        expected["nrender"].as_u64().unwrap() as usize
    );
    assert_eq!(resolved.cam_id_map(), int_list(&expected["cam_id_map"]));
    let cam_res: Vec<Vec<i32>> = resolved
        .cam_res()
        .iter()
        .map(|r| vec![r[0] as i32, r[1] as i32])
        .collect();
    let expected_res: Vec<Vec<i32>> = array(&expected["cam_res"]).iter().map(int_list).collect();
    assert_eq!(cam_res, expected_res);
    assert_eq!(resolved.plan.render_rgb, bool_list(&expected["render_rgb"]));
    assert_eq!(
        resolved.plan.render_depth,
        bool_list(&expected["render_depth"])
    );
    assert_eq!(resolved.plan.render_seg, bool_list(&expected["render_seg"]));
    assert_eq!(
        resolved.output.total_rays,
        expected["total_rays"].as_u64().unwrap() as usize
    );
    assert_eq!(
        (
            resolved.output.rgb_adr.clone(),
            resolved.output.depth_adr.clone(),
            resolved.output.seg_adr.clone()
        ),
        (
            int_list(&expected["rgb_adr"]),
            int_list(&expected["depth_adr"]),
            int_list(&expected["seg_adr"])
        )
    );
    for (actual, key) in [
        (resolved.use_textures, "use_textures"),
        (resolved.use_fast_math, "use_fast_math"),
        (resolved.use_shadows, "use_shadows"),
        (resolved.use_ambient_lighting, "use_ambient_lighting"),
        (resolved.use_precomputed_rays, "use_precomputed_rays"),
        (resolved.render_skybox, "render_skybox"),
        (resolved.enable_backface_culling, "enable_backface_culling"),
        (resolved.enable_vertex_normals, "enable_vertex_normals"),
        (resolved.enable_specular, "enable_specular"),
        (resolved.enable_emission, "enable_emission"),
        (
            resolved.enable_per_light_ambient,
            "enable_per_light_ambient",
        ),
    ] {
        assert_eq!(actual, expected[key].as_bool().unwrap(), "{key}");
    }
    assert_eq!(
        resolved.shadow_light_fraction,
        expected["shadow_light_fraction"].as_f64().unwrap()
    );
    assert_eq!(
        resolved.samples_per_pixel,
        expected["samples_per_pixel"].as_u64().unwrap() as u32
    );
    assert_eq!(
        resolved.background_color_unpacked.to_vec(),
        f64_list(&expected["background_color_unpacked"])
    );
    assert_eq!(
        resolved.bvh_ngeom,
        expected["bvh_ngeom"].as_u64().unwrap() as usize
    );
    assert_eq!(
        resolved.enabled_geom_ids,
        int_list(&expected["enabled_geom_ids"])
    );
    assert_eq!(
        resolved.geom_ray_types,
        int_list(&expected["geom_ray_types"])
    );
    assert_eq!(
        (resolved.znear, resolved.zfar),
        (
            expected["znear"].as_f64().unwrap(),
            expected["zfar"].as_f64().unwrap()
        )
    );
    assert_eq!(
        resolved.headlight.active,
        expected["headlight_active"].as_bool().unwrap()
    );
    let mut headlight = resolved.headlight.ambient.map(f64::from).to_vec();
    headlight.extend(resolved.headlight.diffuse.map(f64::from));
    headlight.extend(resolved.headlight.specular.map(f64::from));
    let mut expected_headlight = f64_list(&expected["headlight_ambient"]);
    expected_headlight.extend(f64_list(&expected["headlight_diffuse"]));
    expected_headlight.extend(f64_list(&expected["headlight_specular"]));
    assert_eq!(headlight, expected_headlight);
    assert_eq!(
        resolved.splat_count,
        expected["splat_count"].as_u64().unwrap() as usize
    );

    let paths = array(&reference["camera_paths"]);
    assert_eq!(resolved.camera_paths.len(), paths.len());
    for (actual, path) in resolved.camera_paths.iter().zip(paths) {
        assert_eq!(
            (actual.camera_id, actual.projection),
            (
                path["cam_id"].as_i64().unwrap() as i32,
                path["projection"].as_i64().unwrap() as i32
            )
        );
        assert_eq!(
            vec![actual.resolution[0] as i32, actual.resolution[1] as i32],
            int_list(&path["resolution"])
        );
        assert_eq!(
            actual.sensorsize.map(f64::from).to_vec(),
            f64_list(&path["sensorsize"])
        );
        assert_eq!(
            actual.intrinsic.map(f64::from).to_vec(),
            f64_list(&path["intrinsic"])
        );
        assert_eq!(actual.fovy_degrees, path["fovy_deg"].as_f64().unwrap());
        let branch = match actual.branch {
            CameraBranch::Intrinsic => "intrinsic",
            CameraBranch::Fovy => "fovy",
        };
        assert_eq!(branch, path["branch"].as_str().unwrap());
    }
    for (key, channels) in [("rgb_shape", 3usize), ("depth_shape", 1), ("seg_shape", 2)] {
        for (position, shape) in array(&reference["outputs"][key]).iter().enumerate() {
            let r = resolved.plan.resolutions[position];
            let mut expected_shape = vec![r[1] as i32, r[0] as i32];
            if channels > 1 {
                expected_shape.push(channels as i32);
            }
            assert_eq!(int_list(shape), expected_shape, "{key}[{position}]");
        }
    }
}

#[test]
fn geom_groups_spp_splats_skybox_and_background_resolve() {
    let fields = render_fields();
    let groups = [3];
    let params = ContextParams {
        enabled_geom_groups: &groups,
        ..ContextParams::default()
    };
    let resolved = resolve_context(&fields, &params).expect("group 3");
    let expected: Vec<i32> = model_ints(&fields, "geom_group")
        .iter()
        .enumerate()
        .filter(|(_, group)| **group == 3)
        .map(|(id, _)| id as i32)
        .collect();
    assert_eq!(
        (resolved.enabled_geom_ids.clone(), resolved.bvh_ngeom),
        (expected.clone(), expected.len())
    );
    assert_eq!(resolved.geom_ray_types, vec![7]);

    let empty: [i32; 0] = [];
    let params = ContextParams {
        enabled_geom_groups: &empty,
        ..ContextParams::default()
    };
    let resolved = resolve_context(&fields, &params).expect("空 group");
    assert_eq!((resolved.bvh_ngeom, resolved.geom_ray_types.len()), (0, 0));

    let params = ContextParams {
        samples_per_pixel: 2,
        ..ContextParams::default()
    };
    let resolved = resolve_context(&fields, &params).expect("spp=2");
    assert_eq!(resolved.output.nsamples, 4);
    assert_eq!(resolved.output.ray_count, resolved.output.total_rays * 4);
    assert_eq!(resolved.output.aa_accum_width, resolved.output.rgb_pixels);

    let position = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let rotation = [[1.0f32, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
    let scale = [[0.1f32, 0.1, 0.1], [0.2, 0.2, 0.2]];
    let rgba = [[1.0f32, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
    let group_id = [0];
    let splat = SplatInputs {
        position: Some(&position),
        rotation: Some(&rotation),
        scale: Some(&scale),
        rgba: Some(&rgba),
        adr: None,
        group_id: Some(&group_id),
    };
    let params = ContextParams {
        splat,
        ..ContextParams::default()
    };
    assert_eq!(
        resolve_context(&fields, &params)
            .expect("splat")
            .splat_count,
        2
    );

    let params = ContextParams {
        render_skybox: true,
        ..ContextParams::default()
    };
    let resolved = resolve_context(&fields, &params).expect("skybox");
    let skybox_id = model_ints(&fields, "tex_type")
        .iter()
        .position(|kind| *kind == 2)
        .unwrap();
    let skybox = resolved.skybox.expect("天空盒引用");
    assert_eq!(
        (skybox.texture_id, skybox.face_width),
        (skybox_id as i32, 64)
    );

    let params = ContextParams {
        background_color: [1.0, 0.5, 0.25, 0.0],
        ..ContextParams::default()
    };
    let resolved = resolve_context(&fields, &params).expect("背景色");
    assert_eq!(
        resolved.background_color_unpacked,
        [255.0, 127.0, 63.0, 0.0]
    );
    assert_eq!(resolved.background_color_packed, 0x00FF7F3F);
}

#[test]
fn invalid_camera_and_output_combinations_are_rejected() {
    let fields = render_fields();
    let reject = |params: &ContextParams<'_>| resolve_context(&fields, params).unwrap_err();

    let mask = [true];
    let params = ContextParams::default().with_active(CameraActive::Mask(&mask));
    assert!(matches!(reject(&params), ContextError::Camera(_)));
    let names = [String::from("missing-camera")];
    let params = ContextParams::default().with_active(CameraActive::Names(&names));
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::UnknownCameraName { .. })
    ));
    let indices = [2];
    let params = ContextParams::default().with_active(CameraActive::Indices(&indices));
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::CameraIndexOutOfRange { .. })
    ));
    let res = [[10, 10], [20, 20], [30, 30]];
    let params = ContextParams::default().with_cam_res(Some(CamRes::List(&res)));
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::ResolutionCount { .. })
    ));
    let flags = [true, false, true];
    let params = ContextParams::default().with_render_targets(
        Some(RenderFlagSpec::List(&flags)),
        None,
        None,
    );
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::RenderFlagLength { .. })
    ));

    let params = ContextParams {
        samples_per_pixel: 0,
        ..ContextParams::default()
    };
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::SamplesPerPixelZero)
    ));
    let params = ContextParams {
        samples_per_pixel: 2,
        use_precomputed_rays: false,
        ..ContextParams::default()
    };
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::SamplesRequirePrecomputedRays { .. })
    ));
    let params = ContextParams {
        samples_per_pixel: 2,
        render_rgb: Some(RenderFlagSpec::Bool(false)),
        ..ContextParams::default()
    };
    assert!(matches!(
        reject(&params),
        ContextError::Camera(CameraError::SamplesRequireRgb)
    ));
}
