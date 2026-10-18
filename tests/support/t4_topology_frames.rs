//! 刚体拓扑与附着相机的静态精度证据。
//! 不把类型映射当成算子或数值等价。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-topology-frames.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const ATTACHED: &str = include_str!("../../src/physics/attached.rs");
const CAMLIGHT: &str = include_str!("../../src/physics/camlight.rs");
const MOCAP: &str = include_str!("../../src/model/mocap.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    ("src/physics/mod.rs", PHYSICS),
    ("src/physics/attached.rs", ATTACHED),
    ("src/physics/camlight.rs", CAMLIGHT),
    ("src/model/mocap.rs", MOCAP),
];
// 独立登记顺序、类型和来源集合。
// 新路线须重新审查，不能自动扩充分母。
const ROUTES: &[&str] = &[
    "rigid:body_parentid:copy:body_parentid",
    "rigid:body_jntadr:copy:body_jntadr",
    "rigid:body_jntnum:copy:body_jntnum",
    "rigid:jnt_type:copy:jnt_type",
    "rigid:jnt_qposadr:copy:jnt_qposadr",
    "rigid:nmocap:derived:nmocap+body_mocapid",
    "rigid:body_mocapid:copy:body_mocapid",
    "com:body_parentid:copy:body_parentid",
    "com:jnt_type:copy:jnt_type",
    "com:jnt_bodyid:copy:jnt_bodyid",
    "com:jnt_dofadr:copy:jnt_dofadr",
    "attached:rigid-stride:stride:",
    "attached:geom_bodyid:copy:geom_bodyid",
    "attached:site_bodyid:copy:site_bodyid",
    "attached:static-geom:derived:geom_bodyid+body_parentid+body_jntnum+body_mocapid+body_rootid+body_weldid",
    "camlight:rigid-stride:stride:",
    "camlight:com-stride:stride:",
    "camlight:cam_mode:copy:cam_mode",
    "camlight:cam_bodyid:copy:cam_bodyid",
    "camlight:cam_targetbodyid:copy:cam_targetbodyid",
    "camlight:light_mode:copy:light_mode",
    "camlight:light_bodyid:copy:light_bodyid",
    "camlight:light_targetbodyid:copy:light_targetbodyid",
];
const STEPS: &[&str] = &[
    "rotation:src/physics/mod.rs:same-scalar:_src/math.py:rot_vec_quat:wp.vec3:3:3",
    "quaternion-product:src/physics/mod.rs:same-scalar:_src/math.py:mul_quat:wp.quat:2:2",
    "quaternion-matrix:src/physics/mod.rs:same-scalar:_src/math.py:quat_to_mat:wp.mat33:2:3",
    "normalize:src/physics/camlight.rs:same-scalar:native/vec.h:normalize:float:2:4",
    "cross:src/physics/camlight.rs:same-scalar:native/vec.h:cross:wp.vec3:1:2",
    "geom-pose:src/physics/attached.rs:static-cache:_src/smooth.py:_geom_local_to_global:wp.vec3+wp.mat33:2:2",
    "site-pose:src/physics/attached.rs:same-scalar:_src/smooth.py:_site_local_to_global:wp.vec3+wp.mat33:2:2",
    "camera-fixed:src/physics/camlight.rs:same-scalar:_src/smooth.py:_cam_local_to_global:wp.vec3+wp.mat33:2:2",
    "camera-track:src/physics/camlight.rs:same-scalar:_src/smooth.py:_cam_local_to_global:wp.vec3+wp.mat33:2:2",
    "camera-com:src/physics/camlight.rs:two-launches:_src/smooth.py:_cam_local_to_global:wp.vec3:1:1",
    "camera-target:src/physics/camlight.rs:two-launches:_src/smooth.py:_cam_local_to_global:wp.vec3+wp.mat33:6:4",
    "light-fixed:src/physics/camlight.rs:same-scalar:_src/smooth.py:_light_local_to_global:wp.vec3:2:2",
    "light-track:src/physics/camlight.rs:same-scalar:_src/smooth.py:_light_local_to_global:wp.vec3:2:2",
    "light-com:src/physics/camlight.rs:two-launches:_src/smooth.py:_light_local_to_global:wp.vec3:1:1",
    "light-target:src/physics/camlight.rs:two-launches:_src/smooth.py:_light_local_to_global:wp.vec3:2:2",
    "light-invalid:src/physics/camlight.rs:skip-normalize:_src/smooth.py:_light_local_to_global:wp.vec3:2:2",
];
const OUTPUTS: &[&str] = &[
    "geom_xpos:src/physics/attached.rs:wp.vec3:&v[..3 * ng]",
    "geom_xmat:src/physics/attached.rs:wp.mat33:&v[3 * ng..12 * ng]",
    "site_xpos:src/physics/attached.rs:wp.vec3:&v[12 * ng..12 * ng + 3 * ns]",
    "site_xmat:src/physics/attached.rs:wp.mat33:&v[12 * ng + 3 * ns..]",
    "cam_xpos:src/physics/camlight.rs:wp.vec3:&v[..3 * nc]",
    "cam_xmat:src/physics/camlight.rs:wp.mat33:&v[3 * nc..12 * nc]",
    "light_xpos:src/physics/camlight.rs:wp.vec3:&v[12 * nc..12 * nc + 3 * nl]",
    "light_xdir:src/physics/camlight.rs:wp.vec3:&v[12 * nc + 3 * nl..]",
];
const REMAINING: [&str; 4] = [
    "other-kernel-internals-and-outputs",
    "operator-order-and-preprocessing-semantics",
    "upstream-generated-kernels",
    "measured-performance",
];
const EXTERNAL: &[(&str, &str)] = &[
    ("upstream", "_src/types.py"),
    ("upstream", "_src/smooth.py"),
    ("upstream", "_src/math.py"),
    ("warp", "_src/types.py"),
    ("warp", "native/vec.h"),
    ("warp", "native/builtin.h"),
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    name: String,
    declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    group: String,
    id: String,
    kind: String,
    scalar: String,
    write: String,
    read: String,
    upstream: Vec<Field>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    file: String,
    classification: String,
    scalar: String,
    upstream_file: String,
    symbol: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    name: String,
    file: String,
    upstream_type: String,
    scalar: String,
    write: String,
    view: String,
    upstream_declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct External {
    root: String,
    path: String,
    sha256: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    external_sources: Vec<External>,
    routes: Vec<Route>,
    steps: Vec<Step>,
    outputs: Vec<Output>,
    np01_complete: bool,
    np02_complete: bool,
    remaining: Vec<String>,
}
fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("source: {path}"))
}
fn template(path: &str) -> Result<&'static str, String> {
    let start = if path == "src/physics/mod.rs" {
        "const KINEMATICS_CUDA: &str = r#\""
    } else {
        ": &str = r#\""
    };
    between(source(path)?, start, "\"#;")
}
fn reader(group: &str) -> Result<&'static str, String> {
    match group {
        "rigid" => template("src/physics/mod.rs"),
        "com" => between(PHYSICS, "const COM_POSITION_CUDA: &str = r#\"", "\"#;"),
        "attached" => template("src/physics/attached.rs"),
        "camlight" => template("src/physics/camlight.rs"),
        _ => Err("reader group".into()),
    }
}
fn writer(group: &str) -> Result<&'static str, String> {
    match group {
        "rigid" => between(RESIDENT, "let rigid = pack_model(", "&RIGID_PARAMETERS,"),
        "com" => between(RESIDENT, "let com = pack_model(", "&COM_PARAMETERS,"),
        "attached" => between(
            RESIDENT,
            "let attached = pack_model(",
            "&ATTACHED_PARAMETERS,",
        ),
        "camlight" => between(RESIDENT, "fn pack_camlight(", "let metadata_len"),
        _ => Err("writer group".into()),
    }
}
fn contains(text: &str, anchor: &str) -> Check {
    require(
        !anchor.trim().is_empty() && text.contains(anchor),
        format!("anchor: {anchor}"),
    )
}
fn validate_routes(i: &Inventory) -> Check {
    require(i.routes.len() == ROUTES.len(), "route count")?;
    let mut cursors = BTreeMap::new();
    for (r, expected) in i.routes.iter().zip(ROUTES) {
        require(
            format!(
                "{}:{}:{}:{}",
                r.group,
                r.id,
                r.kind,
                r.upstream
                    .iter()
                    .map(|u| u.name.as_str())
                    .collect::<Vec<_>>()
                    .join("+")
            ) == *expected
                && r.scalar == "i32",
            "route identity/type",
        )?;
        let host = compact(writer(&r.group)?);
        let write = compact(&r.write);
        require(!write.is_empty(), "empty writer")?;
        let cursor = cursors.entry(r.group.as_str()).or_insert(0);
        let index = host[*cursor..].find(&write).ok_or("writer order")? + *cursor;
        *cursor = index + write.len();
        contains(reader(&r.group)?, &r.read)?;
        for u in &r.upstream {
            require(
                super::upstream_scalar(&u.declaration)? == "i32",
                "integer source",
            )?;
        }
    }
    // 普通字段复制与私有编码分别守卫。
    let pack = between(RESIDENT, "fn pack_model(", "// Descriptors hold")?;
    for anchor in [
        "host_staging::<i32>(metadata_len)",
        "for t in topology {",
        "metadata[cursor..cursor + t.len()].copy_from_slice(t);",
        "cursor += t.len();",
    ] {
        contains(pack, anchor)?;
    }
    contains(
        MOCAP,
        "let nmocap = body_mocapid.iter().filter(|&&id| id >= 0).count();",
    )?;
    contains(
        &compact(MOCAP),
        "static_body[body]=static_body[f.body_parentid[body]asusize]&&f.body_jntnum[body]==0&&id==-1;",
    )?;
    contains(MOCAP, ".map(|&body| static_body[body as usize])")?;
    contains(RESIDENT, "static_geom.into_iter().map(i32::from).collect()")?;
    contains(
        reader("attached")?,
        "const int* static_geom=desc; desc+=ng;",
    )?;
    contains(reader("rigid")?, "desc=mocap+nb;")?;
    Ok(())
}
fn validate_steps(i: &Inventory) -> Check {
    require(i.steps.len() == STEPS.len(), "step count")?;
    for (s, expected) in i.steps.iter().zip(STEPS) {
        require(
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                s.id,
                s.file,
                s.classification,
                s.upstream_file,
                s.symbol,
                s.upstream_type,
                s.product.len(),
                s.upstream.len()
            ) == *expected
                && s.scalar == "f32",
            "step type/boundary",
        )?;
        for a in &s.product {
            contains(template(&s.file)?, a)?;
        }
        require(
            s.upstream.iter().all(|s| !s.trim().is_empty()),
            "empty upstream step",
        )?;
    }
    contains(PHYSICS, "struct V { float x,y,z; };")?;
    contains(PHYSICS, "struct Q { float w,x,y,z; };")?;
    Ok(())
}
fn validate_outputs(i: &Inventory) -> Check {
    require(i.outputs.len() == OUTPUTS.len(), "output count")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!("{}:{}:{}:{}", o.name, o.file, o.upstream_type, o.view) == *expected
                && o.scalar == "f32",
            "output type/view",
        )?;
        contains(template(&o.file)?, &o.write)?;
        contains(source(&o.file)?, &format!("pub {}: &'a [f32]", o.name))?;
        contains(source(&o.file)?, &format!("{}: {},", o.name, o.view))?;
        require(
            super::upstream_scalar(&o.upstream_declaration)? == "f32",
            "output upstream scalar",
        )?;
    }
    Ok(())
}
fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "g01-topology-and-frame-precision"
            && i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::REVISION,
        "revision/scope",
    )?;
    require(
        !i.np01_complete
            && !i.np02_complete
            && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "unaccepted boundary",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for s in &i.product_sources {
        require(digest(source(&s.path)?) == s.sha256, "product source drift")?;
    }
    require(
        i.external_sources.len() == EXTERNAL.len()
            && i.external_sources
                .iter()
                .map(|s| (s.root.as_str(), s.path.as_str()))
                .collect::<BTreeSet<_>>()
                == EXTERNAL.iter().copied().collect(),
        "external coverage",
    )?;
    validate_routes(i)?;
    validate_steps(i)?;
    validate_outputs(i)
}
type ExternalText = BTreeMap<(String, String), String>;
fn external<'a>(texts: &'a ExternalText, root: &str, path: &str) -> Result<&'a str, String> {
    texts
        .get(&(root.into(), path.into()))
        .map(String::as_str)
        .ok_or_else(|| format!("external: {root}/{path}"))
}
fn verify_external(i: &Inventory, texts: &ExternalText) -> Check {
    for s in &i.external_sources {
        require(
            digest(external(texts, &s.root, &s.path)?) == s.sha256,
            "external drift",
        )?;
    }
    let types = external(texts, "upstream", "_src/types.py")?;
    let warp = external(texts, "warp", "_src/types.py")?;
    super::verify_external(&super::inventory(), types, warp)?;
    for anchor in [
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "mat33 = mat33f",
    ] {
        contains(warp, anchor)?;
    }
    let model = between(types, "class Model:", "class Contact:")?;
    for r in &i.routes {
        for u in &r.upstream {
            contains(model, &format!("  {}: {}\n", u.name, u.declaration))?;
        }
    }
    let data = types.split_once("class Data:").ok_or("Data class")?.1;
    for o in &i.outputs {
        contains(data, &format!("  {}: {}\n", o.name, o.upstream_declaration))?;
    }
    for s in &i.steps {
        let text = external(
            texts,
            if s.upstream_file.starts_with("native/") {
                "warp"
            } else {
                "upstream"
            },
            &s.upstream_file,
        )?;
        let body = if s.upstream_file.ends_with(".py") {
            text.split_once(&format!("def {}(", s.symbol))
                .ok_or("upstream symbol")?
                .1
                .split("\n\n\n")
                .next()
                .ok_or("upstream body")?
        } else {
            text
        };
        for a in &s.upstream {
            // 签名锚点包括def，单独在完整来源查找。
            contains(if a.starts_with("def ") { text } else { body }, a)?;
        }
    }
    contains(
        external(texts, "warp", "native/builtin.h")?,
        "#define kEps 0.0f",
    )?;
    contains(
        external(texts, "upstream", "_src/smooth.py")?,
        "if body_weldid[bodyid] == 0 and body_mocapid[body_rootid[bodyid]] == -1:",
    )?;
    Ok(())
}
fn verify_captures(root: &Path) -> Result<(usize, usize), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"].as_array().ok_or("artifacts")?;
    let selected = [
        ("rigid_kinematics", "rigid", 3),
        ("rigid_com_position", "com", 2),
        ("attached_kinematics", "attached", 3),
        ("initialize_attached", "attached", 2),
        ("camlight_rigid", "camlight", 1),
        ("camlight_com", "camlight", 1),
    ];
    let mut total = 0;
    let mut resident = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        for (entry, group, count) in selected {
            let matches: Vec<_> = artifacts.iter().filter(|a| a["entry"] == entry).collect();
            require(matches.len() == count, "capture coverage")?;
            let mut hashes = BTreeSet::new();
            let mut active = 0;
            for a in matches {
                let hash = a["ptx_sha256"].as_str().ok_or("hash")?;
                require(
                    hash.len() == 64
                        && hash.bytes().all(|b| b.is_ascii_hexdigit())
                        && hashes.insert(hash),
                    "capture path/duplicate",
                )?;
                let mut text = String::new();
                for (ext, key) in [
                    ("cu", "source_sha256"),
                    ("ptx", "ptx_sha256"),
                    ("json", "receipt_sha256"),
                ] {
                    let bytes =
                        std::fs::read(root.join(mode).join(format!("{entry}-{hash}.{ext}")))
                            .map_err(|e| e.to_string())?;
                    require(
                        format!("{:x}", Sha256::digest(&bytes))
                            == a[key].as_str().ok_or("artifact hash")?,
                        "capture drift",
                    )?;
                    if ext == "cu" {
                        text = String::from_utf8(bytes)
                            .map_err(|e| e.to_string())?
                            .replace("\r\n", "\n");
                    } else if ext == "json" {
                        let r: serde_json::Value =
                            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                        require(
                            r["entry"] == entry
                                && r["passed"] == true
                                && r["precisionGate"] == "f32-physics"
                                && r["fmad"] == true
                                && r["useFastMath"] == false,
                            "precision receipt",
                        )?;
                    }
                }
                let current = reader(group)?.replace("\r\n", "\n");
                // 旧刚体产物只核对历史分支；新产物另记定向实证。
                let captured = if group == "rigid" {
                    super::rigid_internal::historical_rigid_kernel(&current)?
                } else {
                    current
                };
                contains(&text, &captured)?;
                let uses_resident = text.contains("#define MJWARP_FIELD_BATCHES")
                    && (group == "com"
                        || group == "camlight"
                        || text.contains("#define MJWARP_MOCAP"));
                if uses_resident {
                    active += 1;
                    resident += 1;
                }
                total += 1;
            }
            require(active > 0, "missing active resident topology")?;
        }
    }
    Ok((total, resident))
}
#[test]
fn covers_topology_internal_groups_and_outputs() {
    validate(&inventory()).unwrap();
    assert_eq!((ROUTES.len(), STEPS.len(), OUTPUTS.len()), (23, 16, 8));
}
#[test]
fn rejects_every_missing_or_mistyped_topology_route() {
    for index in 0..ROUTES.len() {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.routes.remove(index);
                }
                1 => i.routes[index].scalar = "f64".into(),
                2 => i.routes[index].read = "wrong_reader".into(),
                3 => i.routes[index].write = "wrong_writer".into(),
                4 => i.routes[index].upstream.push(Field {
                    name: "bad".into(),
                    declaration: "int".into(),
                }),
                _ => i.routes.swap(index, (index + 1) % ROUTES.len()),
            }
            assert!(validate_routes(&i).is_err(), "{index}/{mutation}");
        }
    }
}
#[test]
fn rejects_internal_and_output_precision_drift() {
    for index in 0..STEPS.len() {
        for mutation in 0..4 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.steps.remove(index);
                }
                1 => i.steps[index].scalar = "f64".into(),
                2 => {
                    i.steps[index].product.pop();
                }
                _ => i.steps[index].classification = "equivalent".into(),
            }
            assert!(validate_steps(&i).is_err());
        }
    }
    for index in 0..OUTPUTS.len() {
        for mutation in 0..4 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(index);
                }
                1 => i.outputs[index].scalar = "f64".into(),
                2 => i.outputs[index].view = "&v[..]".into(),
                _ => i.outputs[index].write = "wrong_output".into(),
            }
            assert!(validate_outputs(&i).is_err());
        }
    }
}
#[test]
fn rejects_source_gaps_and_premature_acceptance() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let mut i = inventory();
    i.external_sources.pop();
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.np02_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.remaining.clear();
    assert!(validate(&i).is_err());
}
#[test]
#[ignore = "reads frozen upstream and existing captures; never executes Python or GPU"]
fn audit_frozen_topology_and_frame_sources() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = ExternalText::new();
    for s in &i.external_sources {
        let env = if s.root == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(env).expect(env);
        let text = std::fs::read_to_string(Path::new(&root).join(&s.path))
            .unwrap()
            .replace("\r\n", "\n");
        texts.insert((s.root.clone(), s.path.clone()), text);
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.routes[0].upstream[0].declaration = "array(\"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.steps[0].upstream[0] = "def bad():".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.outputs[0].upstream_declaration = "array(\"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("evidence root");
    let (captures, resident) = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
        "topology_routes":i.routes.len(),"internal_groups":i.steps.len(),"output_fields":i.outputs.len(),
        "upstream_topology_fields":i.routes.iter().flat_map(|r|r.upstream.iter().map(|u|u.name.as_str())).collect::<BTreeSet<_>>().len(),
        "captured_variants":captures,"resident_variants":resident,
        "np01_complete":false,"np02_complete":false,"np03":"historical-unchanged","np04":"not-measured",
        "gpu_executions":0,"python_executions":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_TOPOLOGY_FRAMES_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
