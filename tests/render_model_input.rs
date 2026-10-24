#![cfg(feature = "native-model-probe")]
//! S1 渲染模型输入验收。
//! 纯宿主部分（字段规范、拒绝面、名称语义）在 native-model-probe feature 下运行；
//! 读取 3.12 MJB 的原生对照与 `mj_id2name` 对照默认 ignore（需要可信 DLL）。

use mjwarp_rs::{
    diagnostics::NativeProbeError,
    model::{
        FieldData, NativeModelInfo, RENDER_FIELDS, RenderFieldKind, RenderFields,
        RenderModelCounts, RenderObject,
    },
};

const REFERENCE: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");
/// 不在渲染字段表的17个字段：全部是碰撞/约束/求解专用，渲染器不读取；
/// 由既有 G01/碰撞探针路径覆盖。
const NON_RENDER_FIELDS: [&str; 17] = [
    "geom_contype",
    "geom_conaffinity",
    "geom_condim",
    "geom_priority",
    "geom_plugin",
    "geom_sameframe",
    "geom_solmix",
    "geom_solref",
    "geom_solimp",
    "geom_aabb",
    "geom_rbound",
    "geom_friction",
    "geom_margin",
    "geom_gap",
    "geom_surfacevel",
    "geom_adhesion",
    "geom_fluid",
];

fn reference() -> serde_json::Value {
    serde_json::from_str(REFERENCE).unwrap()
}

fn reference_counts() -> RenderModelCounts {
    let counts = &reference()["counts"];
    let read = |name: &str| counts[name].as_i64().unwrap();
    RenderModelCounts {
        nbody: read("nbody"),
        njnt: read("njnt"),
        ngeom: read("ngeom"),
        nsite: read("nsite"),
        ncam: read("ncam"),
        nlight: read("nlight"),
        nmat: read("nmat"),
        ntex: read("ntex"),
        ntexdata: read("ntexdata"),
        nmesh: read("nmesh"),
        nmeshvert: read("nmeshvert"),
        nmeshnormal: read("nmeshnormal"),
        nmeshtexcoord: read("nmeshtexcoord"),
        nmeshface: read("nmeshface"),
        nmeshgraph: read("nmeshgraph"),
        nmeshpoly: read("nmeshpoly"),
        nmeshpolyvert: read("nmeshpolyvert"),
        nmeshpolymap: read("nmeshpolymap"),
        nnames: read("nnames"),
    }
}

fn synthetic_info() -> NativeModelInfo {
    NativeModelInfo {
        schema: 1,
        native_version: NativeModelInfo::VERSION,
        pointer_bytes: 8,
        num_bytes: 8,
        index_bytes: 4,
        size_bytes: 8,
        native_model_bytes: 1,
        nbody: 1,
        ..Default::default()
    }
}

fn synthetic_data(counts: RenderModelCounts) -> Vec<FieldData> {
    RENDER_FIELDS
        .iter()
        .map(|spec| {
            let count = counts.expected(spec).unwrap();
            match spec.kind {
                RenderFieldKind::Index => FieldData::Index(vec![0; count]),
                RenderFieldKind::Num => FieldData::Num(vec![0.0; count]),
                RenderFieldKind::Flag => FieldData::Flag(vec![0; count]),
                RenderFieldKind::Size => FieldData::Size(vec![0; count]),
                RenderFieldKind::Float => FieldData::Float(vec![0.0; count]),
            }
        })
        .collect()
}

fn synthetic_input() -> Result<mjwarp_rs::io::RenderModelInput, NativeProbeError> {
    let counts = reference_counts();
    let data = synthetic_data(counts);
    mjwarp_rs::io::RenderModelInput::new(synthetic_info(), counts, RenderFields::new(data))
}

fn index_of(name: &str) -> usize {
    RenderFields::index_of(name).unwrap_or_else(|| panic!("missing field {name}"))
}

fn replace(data: &mut [FieldData], name: &str, value: FieldData) {
    data[index_of(name)] = value;
}

#[test]
fn field_table_covers_render_scope_and_reference_counts() {
    let reference = reference();
    let arrays = reference["arrays"].as_object().unwrap();
    assert_eq!(RENDER_FIELDS.len(), 116);
    let mut names: Vec<&str> = RENDER_FIELDS.iter().map(|spec| spec.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), RENDER_FIELDS.len(), "duplicate field names");
    let counts = reference_counts();
    for spec in RENDER_FIELDS {
        let entry = arrays
            .get(spec.name)
            .unwrap_or_else(|| panic!("reference missing {}", spec.name));
        let expected = counts.expected(&spec).unwrap();
        assert_eq!(
            entry["count"].as_i64().unwrap(),
            expected as i64,
            "count {}",
            spec.name
        );
        let kind = entry["kind"].as_str().unwrap();
        assert_eq!(kind, kind_name(spec.kind), "kind {}", spec.name);
    }
    for name in NON_RENDER_FIELDS {
        assert!(!names.contains(&name), "{name}");
        assert!(arrays.contains_key(name), "{name}");
    }
    let reference_keys = arrays.keys().filter(|key| !key.starts_with('_')).count();
    assert_eq!(names.len() + NON_RENDER_FIELDS.len(), reference_keys);
}

fn kind_name(kind: RenderFieldKind) -> &'static str {
    match kind {
        RenderFieldKind::Index => "int32",
        RenderFieldKind::Num => "float64",
        RenderFieldKind::Flag => "byte",
        RenderFieldKind::Size => "int64",
        RenderFieldKind::Float => "float32",
    }
}

#[test]
fn accepts_synthetic_reference_layout() {
    let input = synthetic_input().unwrap();
    assert_eq!(input.counts().ncam, 2);
    assert_eq!(mjwarp_rs::model::RENDER_FIELDS.len(), 116);
    assert_eq!(input.field("tex_data").unwrap().len(), 89_088);
}

type Mutation = Box<dyn Fn(&mut Vec<FieldData>, &mut RenderModelCounts)>;

#[test]
fn rejects_invalid_counts_lengths_kinds_and_ranges() {
    let counts = reference_counts();
    let info = synthetic_info();
    let cases: Vec<(&str, Mutation)> = vec![
        ("negative count", Box::new(|_, c| c.ncam = -1)),
        ("count overflow", Box::new(|_, c| c.nmesh = i64::MAX)),
        (
            "short field",
            Box::new(|data, _| replace(data, "cam_fovy", FieldData::Num(vec![]))),
        ),
        (
            "wrong kind",
            Box::new(|data, _| replace(data, "cam_fovy", FieldData::Index(vec![0; 2]))),
        ),
        (
            "negative resolution",
            Box::new(|data, _| replace(data, "cam_resolution", FieldData::Index(vec![-1; 4]))),
        ),
        (
            "texture id out of range",
            Box::new(|data, _| replace(data, "light_texid", FieldData::Index(vec![99; 3]))),
        ),
        (
            "texture address out of range",
            Box::new(|data, _| replace(data, "tex_adr", FieldData::Size(vec![99_999; 3]))),
        ),
        (
            "name address out of range",
            Box::new(|data, _| replace(data, "name_camadr", FieldData::Index(vec![9_999; 2]))),
        ),
        (
            "mesh range past capacity",
            Box::new(|data, _| replace(data, "mesh_vertnum", FieldData::Index(vec![9; 1]))),
        ),
        (
            "unterminated names",
            Box::new(|data, _| replace(data, "names", FieldData::Flag(vec![b'a'; 193]))),
        ),
        (
            "empty names",
            Box::new(|data, _| replace(data, "names", FieldData::Flag(vec![]))),
        ),
    ];
    for (name, mutate) in cases {
        let mut mutated_counts = counts;
        let mut data = synthetic_data(counts);
        mutate(&mut data, &mut mutated_counts);
        let result =
            mjwarp_rs::io::RenderModelInput::new(info, mutated_counts, RenderFields::new(data));
        assert!(result.is_err(), "case {name} was accepted");
    }
}

#[test]
fn name_lookup_matches_mj_id2name_table_semantics() {
    let counts = reference_counts();
    let mut data = synthetic_data(counts);
    // 名称缓冲：`__a\0b\0`；相机0指向"", 相机1指向"b"。
    replace(
        &mut data,
        "names",
        FieldData::Flag(vec![0, b'_', b'_', b'a', 0, b'b', 0]),
    );
    replace(&mut data, "name_camadr", FieldData::Index(vec![0, 5]));
    let mut mutated = counts;
    mutated.nnames = 7;
    let input =
        mjwarp_rs::io::RenderModelInput::new(synthetic_info(), mutated, RenderFields::new(data))
            .unwrap();
    assert_eq!(input.name_bytes(RenderObject::Camera, 0), Some(&[][..]));
    assert_eq!(input.name_bytes(RenderObject::Camera, 1), Some(&b"b"[..]));
    assert_eq!(input.name_bytes(RenderObject::Camera, -1), None);
    assert_eq!(input.name_bytes(RenderObject::Camera, 2), None);
    assert_eq!(input.name(RenderObject::Camera, 1), Some("b"));
    // 其它对象表的默认0偏移指向空名，语义仍为 Some(&[]).
    assert_eq!(input.name_bytes(RenderObject::Body, 0), Some(&[][..]));
    assert_eq!(input.name_bytes(RenderObject::Body, mutated.nbody), None);
}

fn write_evidence(value: &serde_json::Value) {
    if let Some(dir) = std::env::var_os("MJWARP_S1_EVIDENCE_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("field-diff.json"),
            serde_json::to_vec_pretty(value).unwrap(),
        )
        .unwrap();
    }
}

#[cfg(feature = "native-model-probe")]
mod native {
    use super::*;
    use mjwarp_rs::io::NativeModelProbe;

    const MJB: &[u8] = include_bytes!("../fixtures/render-bvh/scene-a/scene-312.mjb");

    fn load() -> NativeModelProbe {
        let path: std::path::PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
        // SAFETY: acceptance supplies a verified MuJoCo 3.12 DLL and the frozen MJB.
        unsafe { NativeModelProbe::load_trusted(&path, MJB) }.unwrap()
    }

    /// 精确数表：按字段名从参考 JSON 文本切出数字 token，再用 std 的
    /// 正确舍入解析器解析（serde_json 默认 f64 解析会有 1 ULP 误差）。
    fn reference_tokens(name: &str) -> Vec<&'static str> {
        let needle = format!("\"{name}\":");
        let start = REFERENCE.find(&needle).unwrap();
        let marker = "\"values\": [";
        let begin = start + REFERENCE[start..].find(marker).unwrap() + marker.len();
        let end = begin + REFERENCE[begin..].find(']').unwrap();
        REFERENCE[begin..end]
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .collect()
    }

    fn compare(spec: &mjwarp_rs::model::FieldSpec, data: &FieldData, tokens: &[&str]) -> usize {
        if data.len() != tokens.len() {
            return 1;
        }
        let mut mismatches = 0;
        macro_rules! cmp {
            ($values:expr, $parse:expr) => {
                for (index, &value) in $values.iter().enumerate() {
                    let want = $parse(tokens[index]);
                    if value != want {
                        mismatches += 1;
                        if mismatches <= 3 {
                            eprintln!("{}[{index}]: {value} != {want}", spec.name);
                        }
                    }
                }
            };
        }
        match data {
            FieldData::Index(values) => cmp!(values, |token: &str| i32::try_from(
                token.parse::<i64>().unwrap()
            )
            .unwrap()),
            FieldData::Size(values) => cmp!(values, |token: &str| token.parse::<i64>().unwrap()),
            FieldData::Flag(values) => cmp!(values, |token: &str| token.parse::<u8>().unwrap()),
            FieldData::Num(values) => cmp!(values, |token: &str| token.parse::<f64>().unwrap()),
            FieldData::Float(values) => cmp!(values, |token: &str| token.parse::<f32>().unwrap()),
        }
        mismatches
    }

    #[test]
    #[ignore = "需要可信MuJoCo DLL与冻结3.12 MJB"]
    fn matches_scene_a_reference_model_arrays() {
        let probe = load();
        let model = probe.render_snapshot().unwrap().into_model().unwrap();
        assert_eq!(model.counts().ncam, 2);
        assert_eq!(model.counts(), reference_counts());
        let reference = reference();
        let arrays = reference["arrays"].as_object().unwrap();
        let reference_keys = arrays.keys().filter(|key| !key.starts_with('_')).count();
        assert_eq!(
            RENDER_FIELDS.len() + NON_RENDER_FIELDS.len(),
            reference_keys
        );
        let mut rows = Vec::new();
        let mut mismatched = 0;
        for spec in RENDER_FIELDS {
            let tokens = reference_tokens(spec.name);
            let data = model.field(spec.name).unwrap();
            let mismatches = compare(&spec, data, &tokens);
            mismatched += usize::from(mismatches != 0);
            rows.push(serde_json::json!({
                "field": spec.name,
                "kind": kind_name(spec.kind),
                "count": tokens.len(),
                "mismatches": mismatches,
            }));
        }
        let evidence = serde_json::json!({
            "schema": 1,
            "task_id": "2026-10-24-render-bvh-s1",
            "kind": "render-model-input-field-diff",
            "reference": "fixtures/render-bvh/scene-a/model-312.json",
            "mjb": "fixtures/render-bvh/scene-a/scene-312.mjb",
            "fields_compared": RENDER_FIELDS.len(),
            "fields_mismatched": mismatched,
            "excluded_fields": NON_RENDER_FIELDS,
            "rows": rows,
        });
        write_evidence(&evidence);
        assert_eq!(mismatched, 0, "render model field mismatches");
    }

    #[test]
    #[ignore = "需要可信MuJoCo DLL与mj_id2name"]
    fn name_lookup_matches_mj_id2name() {
        let probe = load();
        let model = probe.render_snapshot().unwrap().into_model().unwrap();
        let counts = model.counts();
        let mut checked = 0;
        for object in mjwarp_rs::model::RENDER_OBJECTS {
            let count = object.count(counts);
            for id in [-2, -1, 0, 1, count - 1, count, count + 1] {
                let native = probe.render_name_oracle(object, id).unwrap();
                let rust = model.name_bytes(object, id).map(<[u8]>::to_vec);
                assert_eq!(rust, native, "{object:?}[{id}]");
                checked += 1;
            }
            for id in 0..count {
                let native = probe.render_name_oracle(object, id).unwrap();
                let rust = model.name_bytes(object, id).map(<[u8]>::to_vec);
                assert_eq!(rust, native, "{object:?}[{id}]");
                checked += 1;
            }
        }
        assert!(
            checked >= 40,
            "name comparison surface too small: {checked}"
        );
    }
}
