//! 刚体拓扑与附着相机的静态精度证据。
//! 不把类型映射当成算子或数值等价。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
#[path = "t4_topology_frames/captures.rs"]
mod captures;
#[path = "t4_topology_frames/external.rs"]
mod external;
#[path = "t4_topology_frames/routes.rs"]
mod routes;
#[path = "t4_topology_frames/tests.rs"]
mod tests;
#[path = "t4_topology_frames/validate.rs"]
mod validate;

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-topology-frames.json");
const RESIDENT: &str = include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/evidence/t4-g03-retest/resident-source-anchor.rs"
));
const PHYSICS: &str = include!("../../docs/development-evidence/t4-resident-anchors/physics.rs");
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
