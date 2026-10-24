//! S6a 参数矩阵/拒绝面单元测试（`cfg(test)`，只读 S1 的 3.12 模型夹具）。
//!
//! 默认调用的 28 参数逐项对照在 `tests/render_context_params.rs`（读 S0b context.json）；
//! 这里覆盖调用矩阵（选择/分辨率/输出开关）与不适合放进默认对照的拒绝面分支。

use crate::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};

use super::*;

const MODEL: &str = include_str!("../../../fixtures/render-bvh/scene-a/model-312.json");

fn fields() -> RenderFields {
    let model: serde_json::Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| {
            let values = arrays[spec.name]["values"].as_array().expect("渲染字段");
            match spec.kind {
                RenderFieldKind::Index => {
                    FieldData::Index(values.iter().map(|v| v.as_i64().unwrap() as i32).collect())
                }
                RenderFieldKind::Num => {
                    FieldData::Num(values.iter().map(|v| v.as_f64().unwrap()).collect())
                }
                RenderFieldKind::Flag => {
                    FieldData::Flag(values.iter().map(|v| v.as_u64().unwrap() as u8).collect())
                }
                RenderFieldKind::Size => {
                    FieldData::Size(values.iter().map(|v| v.as_i64().unwrap()).collect())
                }
                RenderFieldKind::Float => {
                    FieldData::Float(values.iter().map(|v| v.as_f64().unwrap() as f32).collect())
                }
            }
        })
        .collect();
    RenderFields::new(data)
}

#[test]
fn camera_and_output_matrix_resolves_like_upstream() {
    let fields = fields();
    let full = 320 * 240 + 240 * 180;
    let mask = [true, false];
    let params = ContextParams::default().with_active(CameraActive::Mask(&mask));
    let resolved = resolve_context(&fields, &params).expect("bool 掩码");
    assert_eq!(
        (resolved.cam_id_map(), resolved.cam_res()),
        (vec![0], &[[320, 240]][..])
    );

    let indices = [1, 0];
    let params = ContextParams::default().with_active(CameraActive::Indices(&indices));
    let resolved = resolve_context(&fields, &params).expect("index 列表");
    assert_eq!(resolved.cam_id_map(), vec![1, 0]);
    assert_eq!(resolved.cam_res(), &[[240, 180], [320, 240]]);
    assert_eq!(resolved.output.rgb_adr, vec![0, 43200]);
    assert_eq!(resolved.output.total_rays, full);

    let names = [String::from("cam-track")];
    let params = ContextParams::default().with_active(CameraActive::Names(&names));
    assert_eq!(
        resolve_context(&fields, &params).expect("name").cam_res(),
        &[[240, 180]]
    );

    let params = ContextParams::default().with_active(CameraActive::Empty);
    let resolved = resolve_context(&fields, &params).expect("空列表合法");
    assert_eq!(
        (
            resolved.nrender(),
            resolved.output.total_rays,
            resolved.output.ray_count
        ),
        (0, 0, 0)
    );
    assert_eq!(
        (resolved.output.seg_slots, resolved.output.aa_accum_width),
        (1, 1)
    );

    let params = ContextParams::default().with_cam_res(Some(CamRes::List(&[[100, 50], [7, 9]])));
    let resolved = resolve_context(&fields, &params).expect("异构分辨率");
    assert_eq!(
        (resolved.cam_res(), resolved.output.total_rays),
        (&[[100, 50], [7, 9]][..], 5063)
    );
    let params = ContextParams::default().with_cam_res(Some(CamRes::Tuple([100, 50])));
    assert_eq!(
        resolve_context(&fields, &params).expect("元组").cam_res(),
        &[[100, 50], [100, 50]]
    );
    let single = [[64, 32]];
    let params = ContextParams::default().with_cam_res(Some(CamRes::List(&single)));
    assert_eq!(
        resolve_context(&fields, &params).expect("单元素").cam_res(),
        &[[64, 32], [64, 32]]
    );
    let params = ContextParams::default()
        .with_active(CameraActive::Empty)
        .with_cam_res(Some(CamRes::Tuple([100, 50])));
    assert_eq!(
        resolve_context(&fields, &params)
            .expect("空 + tuple")
            .output
            .total_rays,
        0
    );

    let params = ContextParams::default().with_render_targets(
        Some(RenderFlagSpec::Bool(true)),
        Some(RenderFlagSpec::Bool(false)),
        Some(RenderFlagSpec::Bool(true)),
    );
    let resolved = resolve_context(&fields, &params).expect("关闭 depth");
    assert_eq!(resolved.output.depth_adr, vec![-1, -1]);
    assert_eq!(resolved.output.depth_buffer_bytes(1), Ok(0));
    assert_eq!(resolved.output.rgb_buffer_bytes(1), Ok(120000 * 4));
    assert_eq!(resolved.output.seg_buffer_bytes(1), Ok(120000 * 8));
    assert_eq!(
        (
            resolved.output.ray_buffer_bytes(),
            resolved.output.aa_accum_bytes(1)
        ),
        (Ok(120000 * 12), Ok(12))
    );

    let filt = [false, true];
    let rgb = [false, true];
    let params = ContextParams::default()
        .with_active(CameraActive::Mask(&filt))
        .with_render_targets(
            Some(RenderFlagSpec::List(&rgb)),
            Some(RenderFlagSpec::Bool(false)),
            None,
        );
    let resolved = resolve_context(&fields, &params).expect("按 active 重排");
    assert_eq!(
        (resolved.plan.render_rgb, resolved.plan.render_depth),
        (vec![true], vec![false])
    );
    assert_eq!(
        (resolved.output.rgb_adr, resolved.output.depth_adr),
        (vec![0], vec![-1])
    );
}

fn splat_params<'a>(
    position: &'a [[f32; 3]],
    rotation: &'a [[f32; 4]],
    scale: &'a [[f32; 3]],
    rgba: &'a [[f32; 4]],
    group_id: Option<&'a [i32]>,
) -> ContextParams<'a> {
    ContextParams {
        splat: SplatInputs {
            position: Some(position),
            rotation: Some(rotation),
            scale: Some(scale),
            rgba: Some(rgba),
            adr: None,
            group_id,
        },
        ..ContextParams::default()
    }
}

#[test]
fn rejects_invalid_splats_and_background() {
    let fields = fields();
    let position = [[0.0f32, 0.0, 0.0]];
    let rotation = [[1.0f32, 0.0, 0.0, 0.0]];
    let scale = [[0.1f32, 0.1, 0.1]];
    let rgba = [[1.0f32, 0.0, 0.0, 1.0]];
    let partial = ContextParams {
        splat: SplatInputs {
            position: Some(&position),
            ..SplatInputs::default()
        },
        ..ContextParams::default()
    };
    assert_eq!(
        resolve_context(&fields, &partial).unwrap_err(),
        ContextError::SplatMissing {
            field: "splat_rotation"
        }
    );
    let group_id: [i32; 2] = [0, 0];
    let params = splat_params(&position, &rotation, &scale, &rgba, Some(&group_id));
    assert_eq!(
        resolve_context(&fields, &params).unwrap_err(),
        ContextError::SplatGroupCount {
            expected: 1,
            actual: 2
        }
    );
    let bad_rotation = [[1.0f32, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];
    let params = splat_params(&position, &bad_rotation, &scale, &rgba, None);
    assert!(matches!(
        resolve_context(&fields, &params).unwrap_err(),
        ContextError::SplatShape { .. }
    ));
    let params = ContextParams {
        background_color: [2.0, 0.0, 0.0, 1.0],
        ..ContextParams::default()
    };
    assert_eq!(
        resolve_context(&fields, &params).unwrap_err(),
        ContextError::BackgroundColor { channel: 0 }
    );
}

fn values(fields: &RenderFields) -> Vec<FieldData> {
    (0..fields.len())
        .map(|id| fields.get(id).expect("字段下标").clone())
        .collect()
}

#[test]
fn rejects_invalid_model_field_kinds_and_lengths() {
    let fields = fields();
    let params = ContextParams::default();
    let mut data = values(&fields);
    let index = RenderFields::index_of("geom_type").unwrap();
    data[index] = FieldData::Float(vec![0.0; data[index].len()]);
    let error = resolve_context(&RenderFields::new(data), &params).unwrap_err();
    assert_eq!(error, ContextError::FieldKind { field: "geom_type" });

    let mut data = values(&fields);
    let index = RenderFields::index_of("light_attenuation").unwrap();
    if let FieldData::Float(values) = &mut data[index] {
        values.pop();
    }
    let error = resolve_context(&RenderFields::new(data), &params).unwrap_err();
    assert!(matches!(
        error,
        ContextError::LengthMismatch {
            field: "light_attenuation",
            ..
        }
    ));

    let mut data = values(&fields);
    let index = RenderFields::index_of("vis.headlight.active").unwrap();
    data[index] = FieldData::Index(vec![1, 0]);
    let error = resolve_context(&RenderFields::new(data), &params).unwrap_err();
    assert!(matches!(
        error,
        ContextError::LengthMismatch {
            field: "vis.headlight.active",
            ..
        }
    ));
}
