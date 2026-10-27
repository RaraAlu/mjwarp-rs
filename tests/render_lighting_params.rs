//! 渲染/BVH R09-S1 验收：光照与阴影静态参数校验（纯宿主）。
//!
//! 参考：`fixtures/render-bvh/scene-a/model-312.json`（S1 渲染字段）与冻结上游
//! `render.py`（`71da24d`）L830/L850/L866/L880/L919-961 的光照/阴影语义。
//! 只读静态夹具，不启动 Python、不占 GPU。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
use mjwarp_rs::render::ContextParams;
use mjwarp_rs::render::context::ContextError;
use mjwarp_rs::render::resolve_lighting;
use serde_json::Value;

const MODEL: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("JSON 数组")
}

/// 按 `RenderFieldKind` 把夹具数组还原成字段数据。
fn build_field(kind: RenderFieldKind, values: &[Value]) -> FieldData {
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

/// S1 渲染字段快照；对 `target` 字段应用 `mutate`（用于构造错误输入）。
fn render_fields_with(target: &str, mutate: impl Fn(&mut FieldData)) -> RenderFields {
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| {
            let mut field = build_field(spec.kind, array(&arrays[spec.name]["values"]));
            if spec.name == target {
                mutate(&mut field);
            }
            field
        })
        .collect();
    RenderFields::new(data)
}

fn render_fields() -> RenderFields {
    render_fields_with("", |_| {})
}

fn int_field<'a>(fields: &'a RenderFields, name: &str) -> &'a [i32] {
    match fields.field(name) {
        Some(FieldData::Index(values)) => values,
        other => panic!("{name} 应为 Index：{other:?}"),
    }
}

fn float_field<'a>(fields: &'a RenderFields, name: &str) -> &'a [f32] {
    match fields.field(name) {
        Some(FieldData::Float(values)) => values,
        other => panic!("{name} 应为 Float：{other:?}"),
    }
}

fn flag_field<'a>(fields: &'a RenderFields, name: &str) -> &'a [u8] {
    match fields.field(name) {
        Some(FieldData::Flag(values)) => values,
        other => panic!("{name} 应为 Flag：{other:?}"),
    }
}

fn params_with(use_shadows: bool, shadow_light_fraction: f64) -> ContextParams<'static> {
    ContextParams {
        use_shadows,
        shadow_light_fraction,
        ..Default::default()
    }
}

#[test]
fn resolves_per_light_fields_and_derived_flags() {
    let fields = render_fields();
    let nlight = int_field(&fields, "light_type").len();
    assert!(nlight > 0, "夹具应含灯");
    let lighting = resolve_lighting(&fields, &ContextParams::default()).expect("光照参数解析");
    assert_eq!(lighting.nlight, nlight);
    assert_eq!(lighting.lights.len(), nlight);
    assert_eq!(lighting.lights.cutoff.len(), nlight);
    assert_eq!(lighting.lights.exponent.len(), nlight);
    assert_eq!(lighting.lights.attenuation.len(), nlight);
    assert_eq!(lighting.lights.ambient.len(), nlight);
    assert_eq!(lighting.lights.diffuse.len(), nlight);
    assert_eq!(lighting.lights.specular.len(), nlight);
    let cast = flag_field(&fields, "light_castshadow");
    let expected_any = cast.iter().any(|flag| *flag != 0);
    assert_eq!(lighting.any_cast_shadow, expected_any);
    assert!(!lighting.use_shadows);
    assert_eq!(lighting.shadow_light_fraction, 0.3);
    assert!(lighting.enable_per_light_ambient);
    assert!(lighting.use_ambient_lighting);
    // `use_shadows=false` 时阴影灯数恒为 0（与上游 L919 分支一致）。
    assert_eq!(lighting.shadow_light_count(), 0);
    // 位姿/静态数组保持单份：逐灯数组长度与 nworld 无关。
    assert_eq!(lighting.lights.light_type, int_field(&fields, "light_type"));
    assert_eq!(lighting.lights.cast_shadow.len(), nlight);
    let expected_cutoff = float_field(&fields, "light_cutoff");
    assert_eq!(lighting.lights.cutoff, expected_cutoff);
}

#[test]
fn per_light_arrays_stay_single_world_regardless_of_nworld() {
    let fields = render_fields();
    let one = resolve_lighting(&fields, &ContextParams::default()).expect("nworld=1");
    let three = ContextParams {
        nworld: 3,
        ..Default::default()
    };
    let three = resolve_lighting(&fields, &three).expect("nworld=3");
    // 灯数组不随 world 复制：R10 结论延续，per-world 灯数据缺口见 backlog。
    assert_eq!(one.lights, three.lights);
    assert_eq!(one.nlight, three.nlight);
}

#[test]
fn shadow_without_castshadow_lights_is_legal() {
    let fields = render_fields_with("light_castshadow", |field| {
        if let FieldData::Flag(values) = field {
            for value in values.iter_mut() {
                *value = 0;
            }
        }
    });
    let lighting = resolve_lighting(&fields, &params_with(true, 0.3)).expect("无投影灯仍合法");
    assert!(!lighting.any_cast_shadow);
    assert_eq!(lighting.shadow_light_count(), 0);
    assert!(lighting.use_shadows);
}

#[test]
fn shadow_light_fraction_passes_through_without_new_rejection() {
    for fraction in [0.0f64, 1.0, 0.3, 2.5, -1.0] {
        let lighting = resolve_lighting(&render_fields(), &params_with(true, fraction))
            .expect("上游无范围校验，本片不新增拒绝");
        assert_eq!(lighting.shadow_light_fraction, fraction);
    }
}

#[test]
fn rejects_wrong_field_kinds() {
    let cases = [
        ("light_type", FieldData::Float(vec![0.0])),
        ("light_castshadow", FieldData::Float(vec![0.0])),
        ("light_cutoff", FieldData::Index(vec![0])),
        ("light_exponent", FieldData::Index(vec![0])),
        ("light_attenuation", FieldData::Index(vec![0])),
        ("light_ambient", FieldData::Index(vec![0])),
        ("light_diffuse", FieldData::Index(vec![0])),
        ("light_specular", FieldData::Index(vec![0])),
    ];
    for (name, replacement) in cases {
        let fields = render_fields_with(name, |field| *field = replacement.clone());
        match resolve_lighting(&fields, &ContextParams::default()) {
            Err(ContextError::FieldKind { field }) => assert_eq!(field, name),
            other => panic!("{name} 类型错误应报 FieldKind，实际 {other:?}"),
        }
    }
}

#[test]
fn rejects_length_mismatch_with_diagnostics() {
    let fields = render_fields();
    let nlight = int_field(&fields, "light_type").len();
    // `light_castshadow` 少一个。
    let short = render_fields_with("light_castshadow", |field| {
        if let FieldData::Flag(values) = field {
            values.pop();
        }
    });
    assert_eq!(
        resolve_lighting(&short, &ContextParams::default()).unwrap_err(),
        ContextError::LengthMismatch {
            field: "light_castshadow",
            expected: nlight,
            actual: nlight - 1,
        }
    );
    // `light_attenuation` 少一个分量（3*nlight）。
    let short = render_fields_with("light_attenuation", |field| {
        if let FieldData::Float(values) = field {
            values.pop();
        }
    });
    assert_eq!(
        resolve_lighting(&short, &ContextParams::default()).unwrap_err(),
        ContextError::LengthMismatch {
            field: "light_attenuation",
            expected: 3 * nlight,
            actual: 3 * nlight - 1,
        }
    );
    // `light_ambient` 多一个分量。
    let long = render_fields_with("light_ambient", |field| {
        if let FieldData::Float(values) = field {
            values.push(0.0);
        }
    });
    assert_eq!(
        resolve_lighting(&long, &ContextParams::default()).unwrap_err(),
        ContextError::LengthMismatch {
            field: "light_ambient",
            expected: 3 * nlight,
            actual: 3 * nlight + 1,
        }
    );
    // 逐灯标量字段（cutoff/exponent）。
    for name in ["light_cutoff", "light_exponent"] {
        let fields = render_fields_with(name, |field| {
            if let FieldData::Float(values) = field {
                values.pop();
            }
        });
        match resolve_lighting(&fields, &ContextParams::default()) {
            Err(ContextError::LengthMismatch { field, .. }) => assert_eq!(field, name),
            other => panic!("{name} 长度错误应报 LengthMismatch，实际 {other:?}"),
        }
    }
}

#[test]
fn zero_lights_is_legal() {
    // 其余灯字段需同步清空才满足长度矩阵。
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| {
            let values = array(&arrays[spec.name]["values"]);
            match spec.name {
                "light_type" | "light_castshadow" | "light_cutoff" | "light_exponent"
                | "light_attenuation" | "light_ambient" | "light_diffuse" | "light_specular" => {
                    match spec.kind {
                        RenderFieldKind::Index => FieldData::Index(Vec::new()),
                        RenderFieldKind::Flag => FieldData::Flag(Vec::new()),
                        RenderFieldKind::Float => FieldData::Float(Vec::new()),
                        other => panic!("灯字段类型意外：{other:?}"),
                    }
                }
                _ => build_field(spec.kind, values),
            }
        })
        .collect();
    let fields = RenderFields::new(data);
    let lighting = resolve_lighting(&fields, &ContextParams::default()).expect("无灯合法");
    assert_eq!(lighting.nlight, 0);
    assert_eq!(lighting.lights.len(), 0);
    assert!(!lighting.any_cast_shadow);
    assert_eq!(lighting.shadow_light_count(), 0);
    let _ = fields;
}

#[test]
fn skybox_flag_does_not_change_lighting_validation() {
    // skybox 与光照解耦：`render_skybox` 只影响 skybox 字段，不改变光照校验结果。
    let fields = render_fields();
    let off = ContextParams {
        render_skybox: false,
        ..Default::default()
    };
    let on = ContextParams {
        render_skybox: true,
        ..Default::default()
    };
    assert_eq!(
        resolve_lighting(&fields, &off).expect("skybox off"),
        resolve_lighting(&fields, &on).expect("skybox on")
    );
}

#[test]
fn rejects_missing_light_field() {
    // 截断到 `light_type` 之前：该字段及之后均视为缺失，`light_type` 必定 MissingField。
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let cutoff = RENDER_FIELDS
        .iter()
        .position(|spec| spec.name == "light_type")
        .expect("light_type 字段");
    let data = RENDER_FIELDS
        .iter()
        .take(cutoff)
        .map(|spec| build_field(spec.kind, array(&arrays[spec.name]["values"])))
        .collect();
    let fields = RenderFields::new(data);
    assert_eq!(
        resolve_lighting(&fields, &ContextParams::default()).unwrap_err(),
        ContextError::MissingField {
            field: "light_type"
        }
    );
}
