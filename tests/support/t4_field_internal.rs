//! 逐字段核对批量打包与质心内部量。
//! 此切片不代表完整精度验收。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-field-internal.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const PARAMETERS: &str = include_str!("../../src/model/parameters.rs");
const CAMLIGHT: &str = include_str!("../../src/model/camlight.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    ("src/physics/mod.rs", PHYSICS),
    (
        "src/physics/attached.rs",
        include_str!("../../src/physics/attached.rs"),
    ),
    (
        "src/physics/camlight.rs",
        include_str!("../../src/physics/camlight.rs"),
    ),
    ("src/model/parameters.rs", PARAMETERS),
    ("src/model/camlight.rs", CAMLIGHT),
];
const REMAINING: [&str; 6] = [
    "other-packers-per-field",
    "integer-topology-and-derived-pack-fields",
    "other-kernel-locals-and-accumulators",
    "complete-output-mapping",
    "upstream-generated-kernels",
    "measured-performance",
];
// 人工审查固定对应关系。
// 不把不同顺序冒充逐算子等价。
const STEPS: &[(&str, &str, &str, &str)] = &[
    ("mass-init", "_subtree_com_init", "derived-cache", "float"),
    (
        "weighted-position",
        "_subtree_com_init",
        "same-scalar",
        "wp.vec3",
    ),
    ("mass-accumulate", "_subtree_div", "derived-cache", "float"),
    (
        "position-accumulate",
        "_subtree_com_acc",
        "different-order",
        "wp.vec3",
    ),
    ("normalize", "_subtree_div", "derived-cache", "wp.vec3"),
    ("inertia-offset", "_cinert", "same-scalar", "wp.vec3"),
    ("inertia-rotation", "_cinert", "different-order", "wp.mat33"),
    ("inertia-diagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-offdiagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-moments", "_cinert", "same-scalar", "vec10"),
    ("motion-offset", "_cdof", "same-scalar", "wp.vec3"),
    (
        "free-translation",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
    ("motion-axis", "_cdof", "same-scalar", "wp.vec3"),
    ("slide-motion", "_cdof", "same-scalar", "wp.spatial_vector"),
    (
        "rotation-motion",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    group: String,
    variant: String,
    name: String,
    slot: usize,
    descriptor_scalar: String,
    parameter_scalar: String,
    width: String,
    reader: String,
    entry: String,
    upstream_declaration: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    symbol: String,
    classification: String,
    scalar: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    upstream_smooth: Stamp,
    fields: Vec<Field>,
    com_steps: Vec<Step>,
    np01_complete: bool,
    remaining: Vec<String>,
}

fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}

fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("unreviewed source: {path}"))
}

fn com_source() -> &'static str {
    between(PHYSICS, "const COM_POSITION_CUDA: &str = r#\"", "\"#;").unwrap()
}

fn kernel(entry: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    match entry {
        "rigid_kinematics" => Ok((PHYSICS, "model", "desc")),
        "rigid_com_position" => Ok((com_source(), "p", "desc")),
        "attached_kinematics" => Ok((source("src/physics/attached.rs")?, "model", "desc")),
        "camlight_rigid" | "camlight_com" => Ok((source("src/physics/camlight.rs")?, "model", "d")),
        _ => Err("unreviewed reader entry".into()),
    }
}

fn reader_anchor(f: &Field) -> Result<String, String> {
    let (_, parameters, base) = kernel(&f.entry)?;
    let descriptor = if f.slot == 0 {
        base.into()
    } else {
        format!("{base}+{}", 2 * f.slot)
    };
    Ok(format!(
        "const float* {}=field_parameter({parameters},{descriptor},w,{});",
        f.reader, f.width
    ))
}

fn variants(text: &str, start: &str, prefix: &str) -> Result<Vec<String>, String> {
    Ok(between(text, start, "];")?
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect())
}

fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-batched-parameters-and-com-internals",
        "scope",
    )?;
    require(
        i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::REVISION,
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining work",
    )?;
    let paths: BTreeSet<_> = i.product_sources.iter().map(|s| s.path.as_str()).collect();
    require(
        paths.len() == i.product_sources.len()
            && paths == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for stamp in &i.product_sources {
        require(
            digest(source(&stamp.path)?) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    require(
        i.upstream_smooth.path == "_src/smooth.py"
            && i.upstream_smooth.sha256
                == "6268f0e1b6cb4c9ec6bbd3aa29828dfb36224efb2a3a3e1f54e5f87b6623fdef",
        "smooth source",
    )?;
    let input = super::inventory();
    let mut expected = Vec::new();
    for (group, start) in [
        ("rigid", "const RIGID_PARAMETERS:"),
        ("com", "const COM_PARAMETERS:"),
        ("attached", "const ATTACHED_PARAMETERS:"),
    ] {
        for (slot, variant) in variants(RESIDENT, start, "KinematicsParameter::")?
            .into_iter()
            .enumerate()
        {
            expected.push((group, slot, variant));
        }
    }
    for (slot, variant) in variants(CAMLIGHT, "pub const ALL:", "Self::")?
        .into_iter()
        .enumerate()
    {
        expected.push(("camlight", slot, variant));
    }
    require(
        expected.len() == 23 && i.fields.len() == 23,
        "field coverage",
    )?;
    // 同时覆盖公开枚举，避免新增字段静默漏审。
    let public: BTreeSet<_> = mjwarp_rs::model::KinematicsParameter::ALL
        .into_iter()
        .map(|f| f.name())
        .chain(
            mjwarp_rs::model::CamLightParameter::ALL
                .into_iter()
                .map(|f| f.name()),
        )
        .collect();
    require(
        public.len() == 23
            && i.fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<BTreeSet<_>>()
                == public,
        "public field coverage",
    )?;
    for (f, (group, slot, variant)) in i.fields.iter().zip(expected) {
        require(
            f.group == group && f.slot == slot && f.variant == variant,
            "packing order",
        )?;
        require(
            f.descriptor_scalar == "i32" && f.parameter_scalar == "f32",
            "packed scalar",
        )?;
        let declaration = input
            .fields
            .iter()
            .find(|p| p.name == f.name)
            .ok_or("unknown input")?;
        require(
            f.upstream_declaration == declaration.upstream_declaration
                && super::upstream_scalar(&f.upstream_declaration)? == "f32",
            "upstream field scalar",
        )?;
        let host = if group == "camlight" {
            CAMLIGHT
        } else {
            PARAMETERS
        };
        require(
            host.contains(&format!("Self::{} => \"{}\",", f.variant, f.name)),
            "host field identity",
        )?;
        let shared = between(host, "pub(crate) fn shared(", "\n}")?;
        require(
            shared.lines().any(|l| {
                l.trim().starts_with(&format!("Self::{} => &", f.variant))
                    && l.trim().ends_with(&format!(".{},", f.name))
            }),
            "shared field source",
        )?;
        let entry = match group {
            "rigid" => "rigid_kinematics",
            "com" => "rigid_com_position",
            "attached" => "attached_kinematics",
            _ if matches!(slot, 2 | 7) => "camlight_com",
            _ => "camlight_rigid",
        };
        require(f.entry == entry && !f.reader.is_empty(), "reader routing")?;
        require(
            kernel(entry)?.0.contains(&reader_anchor(f)?),
            "descriptor reader",
        )?;
    }
    for (start, end) in [
        ("fn pack_model(", "// Descriptors hold"),
        ("fn pack_camlight(", "impl PackedModel"),
    ] {
        let pack = between(RESIDENT, start, end)?;
        for anchor in [
            "host_staging::<i32>(metadata_len)",
            "host_staging::<f32>(parameters_len.max(1))",
            "metadata[cursor] = offset as i32;",
            "metadata[cursor + 1] = parameters.batches(f) as i32;",
            "values[offset..offset + v.len()].copy_from_slice(v);",
            "offset += v.len();",
            "cursor += 2;",
        ] {
            require(pack.contains(anchor), "typed pack route")?;
        }
    }
    for anchor in [
        "parameters: session.upload(&self.parameters)?",
        "return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;",
    ] {
        require(RESIDENT.contains(anchor), "upload and independent period")?;
    }
    validate_com(i, com_source())
}

fn validate_com(i: &Inventory, text: &str) -> Check {
    require(i.com_steps.len() == STEPS.len(), "COM step coverage")?;
    // 哈希绑定整个模板，局部锚点只提供可读映射。
    require(digest(text) == digest(com_source()), "COM template drift")?;
    for (step, &(id, symbol, classification, ty)) in i.com_steps.iter().zip(STEPS) {
        require(
            (
                step.id.as_str(),
                step.symbol.as_str(),
                step.classification.as_str(),
                step.upstream_type.as_str(),
            ) == (id, symbol, classification, ty)
                && step.scalar == "f32",
            "COM type or boundary",
        )?;
        require(
            !step.product.is_empty() && !step.upstream.is_empty(),
            "COM missing anchors",
        )?;
        for anchor in &step.product {
            require(
                !anchor.trim().is_empty() && text.contains(anchor),
                "COM product anchor",
            )?;
        }
    }
    Ok(())
}

fn verify_upstream(i: &Inventory, smooth: &str, types: &str, warp: &str) -> Check {
    require(
        digest(smooth) == i.upstream_smooth.sha256,
        "upstream smooth drift",
    )?;
    super::verify_external(&super::inventory(), types, warp)?;
    for anchor in [
        "class vec10f(wp.types.vector(length=10, dtype=float)):",
        "vec10 = vec10f",
    ] {
        require(types.contains(anchor), "inertia type")?;
    }
    for anchor in [
        "class spatial_vectorf(vector(length=6, dtype=float32)):",
        "spatial_vector = spatial_vectorf",
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "mat33 = mat33f",
    ] {
        require(warp.contains(anchor), "internal scalar type")?;
    }
    for step in &i.com_steps {
        let body = smooth
            .split_once(&format!("def {}(", step.symbol))
            .ok_or("upstream symbol")?
            .1;
        let body = body.split("\n\n\n").next().ok_or("upstream body")?;
        for anchor in &step.upstream {
            require(
                !anchor.trim().is_empty() && body.contains(anchor),
                format!("upstream expression: {}", step.id),
            )?;
        }
    }
    Ok(())
}

fn verify_captures(i: &Inventory, root: &Path) -> Result<(usize, usize), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"].as_array().ok_or("artifacts")?;
    let mut total = 0;
    let mut batched = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut entries = BTreeSet::new();
        let mut fields = BTreeSet::new();
        for artifact in artifacts {
            let entry = artifact["entry"].as_str().ok_or("entry")?;
            if kernel(entry).is_err() {
                continue;
            }
            let ptx = artifact["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                ptx.len() == 64 && ptx.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let stem = format!("{entry}-{ptx}");
            let mut cuda = String::new();
            for (ext, hash_key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes = std::fs::read(root.join(mode).join(format!("{stem}.{ext}")))
                    .map_err(|e| e.to_string())?;
                let expected = artifact[hash_key].as_str().ok_or("artifact hash")?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == expected,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    cuda = std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())?
                        .to_owned();
                }
                if ext == "json" {
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        receipt["entry"] == entry
                            && receipt["passed"] == true
                            && receipt["precisionGate"] == "f32-physics"
                            && receipt["fmad"] == true
                            && receipt["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            if entry == "rigid_com_position" {
                require(
                    cuda.replace("\r\n", "\n").contains(com_source()),
                    "captured COM template",
                )?;
            }
            // 只给真正启用批量宏的产物记字段覆盖。
            if cuda
                .lines()
                .any(|l| l.trim() == "#define MJWARP_FIELD_BATCHES")
            {
                for f in i.fields.iter().filter(|f| f.entry == entry) {
                    require(cuda.contains(&reader_anchor(f)?), "captured field reader")?;
                    fields.insert(f.name.as_str());
                }
                batched += 1;
            }
            entries.insert((entry, ptx));
            total += 1;
        }
        require(
            entries.len() == 10 && fields.len() == 23,
            "capture coverage",
        )?;
    }
    Ok((total, batched))
}

#[test]
fn maps_all_batched_fields_and_com_steps() {
    validate(&inventory()).unwrap();
}

#[test]
fn rejects_each_missing_or_misrouted_field() {
    for index in 0..23 {
        for mutation in 0..7 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.fields.remove(index);
                }
                1 => i.fields[index] = i.fields[(index + 1) % 23].clone(),
                2 => i.fields[index].parameter_scalar = "f64".into(),
                3 => i.fields[index].descriptor_scalar = "f32".into(),
                4 => i.fields[index].slot += 1,
                5 => i.fields[index].width = "1".into(),
                _ => i.fields[index].reader = "wrong_reader".into(),
            }
            assert!(validate(&i).is_err(), "field {index}, mutation {mutation}");
        }
    }
}

#[test]
fn rejects_each_missing_or_mistyped_internal_step() {
    for index in 0..STEPS.len() {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.com_steps.remove(index);
                }
                1 => i.com_steps[index] = i.com_steps[(index + 1) % STEPS.len()].clone(),
                2 => i.com_steps[index].scalar = "f64".into(),
                3 => i.com_steps[index].upstream_type = "wp.float64".into(),
                4 => i.com_steps[index].classification = "equivalent".into(),
                _ => i.com_steps[index].product.clear(),
            }
            assert!(
                validate_com(&i, com_source()).is_err(),
                "step {index}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn rejects_source_drift_and_premature_completion() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let i = inventory();
    for (old, new) in [
        ("float v=0.0f;", "double v=0.0;"),
        ("sm[up]+=sm[b]", "sm[up]=sm[b]"),
        ("float t[9]", "double t[9]"),
        ("1.0f", "1.0"),
    ] {
        assert!(validate_com(&i, &com_source().replace(old, new)).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    i.np01_complete = false;
    i.remaining.clear();
    assert!(validate(&i).is_err());
}

#[test]
#[ignore = "reads frozen sources and existing captures without GPU execution"]
fn audit_field_internal_sources_and_captures() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let smooth = std::fs::read_to_string(Path::new(&root).join("_src/smooth.py"))
        .unwrap()
        .replace("\r\n", "\n");
    let types = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &smooth, &types, &warp).unwrap();
    let mut changed = i.clone();
    changed.com_steps[0].upstream[0] = "body_mass: wp.array2d[wp.float64],".into();
    assert!(verify_upstream(&changed, &smooth, &types, &warp).is_err());
    assert!(verify_upstream(&i, &(smooth.clone() + "\n"), &types, &warp).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root");
    let (captures, batched) = verify_captures(&i, Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "batched_fields":i.fields.len(), "com_step_groups":i.com_steps.len(),
        "captured_variants":captures, "batched_variants":batched,
        "gpu_executions":0, "python_executions":0, "np01_complete":false,
        "np02_complete":false, "np03":"historical-unchanged", "np04":"not-measured",
        "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_FIELD_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
