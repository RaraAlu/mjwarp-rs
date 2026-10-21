//! 仅刷新编译产物；不建立GPU会话，不执行物理内核。
use super::{PHYSICS_FMAD, check_physics_ptx, record_physics_ptx};
use std::{fs, path::PathBuf};

#[path = "t4_face_lineage.rs"]
mod face_lineage;
const FACE: &str = include_str!("../../src/physics/flex_face.rs");

// 复用完整证据校验器；本切片不调用其性能接口。
#[allow(dead_code)]
#[path = "t4_evidence.rs"]
mod evidence;
use evidence::{Inventory, digest, validate_inventory, verify_capture, verify_sources};

#[path = "t4_rigid_capture/lineage.rs"]
mod lineage;
#[path = "t4_rigid_capture/recipe.rs"]
mod recipe;
#[path = "t4_rigid_capture/tests_archive.rs"]
mod tests_archive;
#[path = "t4_rigid_capture/tests_host.rs"]
mod tests_host;
#[path = "t4_rigid_capture/tests_recapture.rs"]
mod tests_recapture;

const BASE: &str = include_str!("../../fixtures/precision/t4-kernels.json");
const PRE_ROTATE: &str =
    include_str!("../../docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json");
const SOURCE_IDENTITY: &str =
    include_str!("../../docs/evidence/t4-g03-retest/rigid-rotate-recapture/source-identity.json");
const REVISION: &str = "8a5bac00fdfa9759f752993b2866098c6ff20819";
const PHYSICS: &str = include!("../../docs/development-evidence/t4-resident-anchors/physics.rs");
const RESIDENT: &str = include!("../../docs/development-evidence/t4-resident-anchors/resident.rs");
const COM_VELOCITY: &str = include_str!("../../src/physics/com_velocity.rs");
const FIXED_TENDON: &str = include_str!("../../src/physics/fixed_tendon.rs");
const FLEX: &str = include_str!("../../src/physics/flex.rs");
const FLEX_EDGE: &str = include_str!("../../src/physics/flex_edge.rs");
const FLEX_HESSIAN: &str = include_str!("../../src/physics/flex_hessian.rs");
const MASS_MATRIX: &str = include_str!("../../src/physics/mass_matrix.rs");
const MASS_SOLVE: &str =
    include!("../../docs/development-evidence/t4-resident-anchors/mass-solve.rs");
const RNE_BIAS: &str = include_str!("../../src/physics/rne_bias.rs");
const SLEEP: &str = include_str!("../../src/physics/sleep.rs");
const SPATIAL_TENDON: &str = include_str!("../../src/physics/spatial_tendon.rs");
const TENDON: &str = include_str!("../../src/physics/tendon.rs");
const TENDON_WRAP: &str = include_str!("../../src/physics/tendon_wrap.rs");
const RUNTIME: &str = include_str!("../../src/runtime/transfer/kernel.rs");
const HOOK: &str =
    "\n#[cfg(test)]\n#[path = \"../../../tests/support/t4_rigid_capture.rs\"]\nmod recapture;\n";
const ENTRY: &str = "rigid_kinematics";
const ROTATE_BEFORE: &str = r#"__device__ V rotate(Q q,V v) {
  float d=q.x*v.x+q.y*v.y+q.z*v.z;
  float s=q.w*q.w-q.x*q.x-q.y*q.y-q.z*q.z;
  return {s*v.x+2*d*q.x+2*q.w*(q.y*v.z-q.z*v.y),
          s*v.y+2*d*q.y+2*q.w*(q.z*v.x-q.x*v.z),
          s*v.z+2*d*q.z+2*q.w*(q.x*v.y-q.y*v.x)};
}"#;
const ROTATE_AFTER: &str = r#"__device__ V rotate(Q q,V v) {
  V u={q.x,q.y,q.z};
  V lhs=scale(scale(u,dot_v(u,v)),2.0f);
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,scale(v,factor));
  return add(r,scale(cross_v(u,v),2.0f*q.w));
}"#;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn normalized(s: &str) -> String {
    s.replace("\r\n", "\n")
}
fn base() -> Inventory {
    serde_json::from_str(BASE).unwrap()
}
fn current() -> Inventory {
    serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/precision/t4-current-kernels.json")).unwrap(),
    )
    .unwrap()
}
fn template<'a>(source: &'a str, name: &str) -> &'a str {
    let hint = format!("缺源码锚点 {name}，见 t4-resident-anchors/README.md");
    let head = format!("const {name}: &str = r#\"");
    let (_, tail) = source.split_once(&head).unwrap_or_else(|| panic!("{hint}"));
    tail.split_once("\"#;")
        .unwrap_or_else(|| panic!("{hint}：模板未闭合"))
        .0
}
fn sources() -> [String; 3] {
    let kernel = normalized(template(PHYSICS, "KINEMATICS_CUDA"));
    let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
    [
        kernel.clone(),
        format!("{fields}\n#define MJWARP_MOCAP\n{kernel}"),
        format!("{fields}\n#define MJWARP_MOCAP\n#define MJWARP_G01_EQUIVALENT\n{kernel}"),
    ]
}

fn before_rotate(source: &str) -> String {
    source
        .replace(
            "__device__ float dot_v(V a,V b) { return a.x*b.x+a.y*b.y+a.z*b.z; }\n",
            "",
        )
        .replace(
            "__device__ V cross_v(V a,V b) { return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }\n",
            "",
        )
        .replace(ROTATE_AFTER, ROTATE_BEFORE)
}
