//! 刚体内部量与七项输出的静态映射。
//! 零分支已有产物实证；完整精度仍不放行。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-rigid-internal.json");
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const TRANSFER: &str = include_str!("../../src/runtime/transfer.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/mod.rs", PHYSICS),
    ("src/physics/resident.rs", RESIDENT),
    ("src/runtime/transfer.rs", TRANSFER),
];
const EXTERNAL: &[&str] = &[
    "upstream/_src/types.py",
    "upstream/_src/smooth.py",
    "upstream/_src/math.py",
    "upstream/_src/io.py",
    "warp/_src/types.py",
    "warp/_src/builtins.py",
    "warp/native/quat.h",
    "warp/native/builtin.h",
];
// 顺序、表达式数量与边界共同构成审查分母。
const STEPS: &[&str] = &[
    "normalize-positive|source-boundary|warp/native/quat.h|normalize|wp.quat|3|5",
    "quaternion-product|same-scalar|upstream/_src/math.py|mul_quat|wp.quat|3|3",
    "rotation|same-scalar|upstream/_src/math.py|rot_vec_quat|wp.vec3|3|3",
    "matrix|same-scalar|upstream/_src/math.py|quat_to_mat|wp.mat33|3|4",
    "free-state|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|3|5",
    "mocap|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|2|5",
    "parent-compose|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3+wp.quat|3|2",
    "joint-anchor|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|1",
    "joint-axis|different-order|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|2",
    "slide|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|2",
    "ball|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.quat|2|3",
    "hinge-trigonometry|same-scalar|upstream/_src/math.py|axis_angle_to_quat|float+wp.quat|2|4",
    "rotation-accumulate|same-scalar|upstream/_src/smooth.py|_kinematics_branch|wp.quat|1|3",
    "offcenter-correction|different-expression|upstream/_src/smooth.py|_kinematics_branch|wp.vec3|1|1",
    "final-normalize|normalization-dependent|upstream/_src/smooth.py|_kinematics_branch|wp.quat|1|1",
    "inertial-position|fused-stage|upstream/_src/smooth.py|_compute_body_inertial_frames|wp.vec3|1|1",
    "inertial-matrix|fused-stage|upstream/_src/smooth.py|_compute_body_inertial_frames|wp.mat33|1|1",
];
const OUTPUTS: &[&str] = &[
    "xpos|wp.vec3|float* xp=result+4+(unsigned long long)w*(28*nb+6*nj);|store_v(xp+3*b,p);|&v[..3 * nb]|_kinematics_branch",
    "xquat|wp.quat|float* xq=xp+3*nb;|store_q(xq+4*b,r);|&v[3 * nb..7 * nb]|_kinematics_branch",
    "xmat|wp.mat33|float* xm=xq+4*nb;|matrix(xm+9*b,r);|&v[7 * nb..16 * nb]|_compute_body_matrices",
    "xipos|wp.vec3|float* xi=xm+9*nb;|store_v(xi+3*b,add(p,rotate(r,load_v(ip+3*b))));|&v[16 * nb..19 * nb]|_compute_body_inertial_frames",
    "ximat|wp.mat33|float* im=xi+3*nb;|matrix(im+9*b,multiply(r,load_q(iq+4*b)));|&v[19 * nb..28 * nb]|_compute_body_inertial_frames",
    "xanchor|wp.vec3|float* anchor=im+9*nb;|store_v(anchor+3*j,a);|&v[28 * nb..28 * nb + 3 * nj]|_kinematics_branch",
    "xaxis|wp.vec3|float* xa=anchor+3*nj;|store_v(xa+3*j,ax);|&v[28 * nb + 3 * nj..]|_kinematics_branch",
];
const BOUNDARIES: [&str; 3] = [
    "world-pose-preserved",
    "serial-versus-branch-schedule",
    "diagnostic-f64-no-feedback",
];
const REMAINING: [&str; 5] = [
    "zero-quaternion-full-pipeline-not-retested",
    "operator-order-and-generated-semantics",
    "other-kernel-internals-and-outputs",
    "upstream-generated-kernels",
    "measured-performance",
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    classification: String,
    scalar: String,
    upstream_source: String,
    symbol: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    name: String,
    scalar: String,
    upstream_type: String,
    pointer: String,
    write: String,
    view: String,
    symbol: String,
    upstream_write: String,
    upstream_declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ZeroQuaternion {
    status: String,
    product_return: [u8; 4],
    warp_header_return: [u8; 4],
    warp_doc_claim: String,
    upstream_generated_verified: bool,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    external_sources: Vec<Stamp>,
    steps: Vec<Step>,
    outputs: Vec<Output>,
    boundaries: Vec<String>,
    zero_quaternion: ZeroQuaternion,
    np01_complete: bool,
    np02_complete: bool,
    equivalence_eligible: bool,
    remaining: Vec<String>,
}
fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}
fn kernel() -> &'static str {
    between(PHYSICS, "const KINEMATICS_CUDA: &str = r#\"", "\"#;").unwrap()
}
/// 旧捕获仅证明历史源码，不充当修复后的完整捕获。
/// 只回溯已实测的一个分支；其他漂移仍触发原哈希检查。
pub(super) fn historical_rigid_kernel(text: &str) -> Result<String, String> {
    let current = "if(n==0.0f) return {0,0,0,1};";
    require(text.matches(current).count() == 1, "zero fallback drift")?;
    Ok(text.replace(current, "if(n==0.0f) return {0,0,0,0};"))
}
fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("source: {path}"))
}
fn anchor(text: &str, expected: &str) -> Check {
    require(
        !expected.trim().is_empty() && text.contains(expected),
        format!("anchor: {expected}"),
    )
}
fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
    text.split_once(&format!("def {symbol}("))
        .map(|(_, s)| s.split("\n\n\n").next().unwrap())
        .ok_or_else(|| format!("symbol: {symbol}"))
}
fn validate_steps(i: &Inventory, text: &str) -> Check {
    require(i.steps.len() == STEPS.len(), "internal coverage")?;
    for (s, expected) in i.steps.iter().zip(STEPS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                s.id,
                s.classification,
                s.upstream_source,
                s.symbol,
                s.upstream_type,
                s.product.len(),
                s.upstream.len()
            ) == *expected
                && s.scalar == "f32",
            "internal identity/type/boundary",
        )?;
        for a in &s.product {
            anchor(text, a)?;
        }
        require(
            s.upstream.iter().all(|a| !a.trim().is_empty()),
            "empty upstream anchor",
        )?;
    }
    for a in [
        "struct V { float x,y,z; };",
        "struct Q { float w,x,y,z; };",
        "__device__ V add(V a,V b) { return {a.x+b.x,a.y+b.y,a.z+b.z}; }",
        "__device__ V scale(V a,float s) { return {a.x*s,a.y*s,a.z*s}; }",
    ] {
        anchor(text, a)?;
    }
    Ok(())
}
fn validate_outputs(i: &Inventory) -> Check {
    require(i.outputs.len() == OUTPUTS.len(), "output coverage")?;
    for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
        require(
            format!(
                "{}|{}|{}|{}|{}|{}",
                o.name, o.upstream_type, o.pointer, o.write, o.view, o.symbol
            ) == *expected
                && o.scalar == "f32",
            "output identity/type/view",
        )?;
        anchor(kernel(), &o.pointer)?;
        anchor(kernel(), &o.write)?;
        anchor(PHYSICS, &format!("pub {}: &'a [f32]", o.name))?;
        anchor(PHYSICS, &format!("{}: {},", o.name, o.view))?;
        require(
            super::upstream_scalar(&o.upstream_declaration)? == "f32",
            "output upstream type",
        )?;
        require(!o.upstream_write.trim().is_empty(), "empty output writer")?;
    }
    for a in [
        "let stride = packed(&[(nbody, 28), (njnt, 6)])?;",
        "let v = &self.values[r.start + 4..r.end + 4];",
        "__device__ void store_q(float* p,Q q) { p[0]=q.w;p[1]=q.x;p[2]=q.y;p[3]=q.z; }",
    ] {
        anchor(PHYSICS, a)?;
    }
    anchor(RESIDENT, "rigid: TransferBuffer<f32>")?;
    anchor(
        RESIDENT,
        "let rigid = read_output(&self.rigid, lengths[0], \"resident_rigid_output\")?;",
    )?;
    let read = between(RESIDENT, "fn read_output<", "\n}")?;
    for a in [
        "Result<Vec<T>, TransferError>",
        "host_staging::<T>(length)",
        "buffer.read_range_into(0, &mut values)?;",
        "Ok(values)",
    ] {
        anchor(read, a)?;
    }
    anchor(TRANSFER, "target.copy_from_slice(staging.values());")?;
    // f64仅用于只读有限值诊断，不回写物理结果。
    let check = between(PHYSICS, "fn check_device_values<", "\n}")?;
    anchor(check, "values: &[T]")?;
    anchor(check, "if !(*value).into().is_finite()")?;
    Ok(())
}
fn validate_boundaries(i: &Inventory, text: &str) -> Check {
    let normalized = text.replace("\r\n", "\n");
    let text = normalized.as_str();
    require(
        i.boundaries.iter().map(String::as_str).eq(BOUNDARIES),
        "scope boundaries",
    )?;
    let z = &i.zero_quaternion;
    require(
        z.status == "matched-generated-artifact"
            && z.product_return == [0, 0, 0, 1]
            && z.warp_header_return == [0, 0, 0, 1]
            && z.warp_doc_claim == "zero quaternion"
            && z.upstream_generated_verified
            && !i.equivalence_eligible,
        "zero quaternion evidence boundary",
    )?;
    anchor(
        text,
        "#ifdef MJWARP_G01_EQUIVALENT\n  if(n==0.0f) return {0,0,0,1};\n#endif",
    )?;
    anchor(text, "if(!b) { p=load_v(xp); r=load_q(xq); }")?;
    anchor(text, "for(unsigned b=0;b<nb;++b)")?;
    anchor(RESIDENT, "\"#define MJWARP_G01_EQUIVALENT\\n\"")?;
    anchor(RESIDENT, "data.rigid.write_range(start, &[0.0; 3])?;")?;
    // 完整入口仅检查长度与有限值，不排除零四元数。
    let check = between(RESIDENT, "fn check_g01_qpos(", "\n}")?;
    anchor(
        check,
        "check_finite_field(\"kinematics_qpos\", expected, qpos)?;",
    )?;
    require(
        !check.contains("norm_squared"),
        "zero input boundary changed",
    )?;
    Ok(())
}
fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1
            && i.scope == "g01-rigid-internal-and-output"
            && i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::REVISION,
        "scope/revision",
    )?;
    require(
        !i.np01_complete
            && !i.np02_complete
            && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "unaccepted scope",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "product coverage",
    )?;
    for s in &i.product_sources {
        require(digest(source(&s.path)?) == s.sha256, "product drift")?;
    }
    require(
        i.external_sources.len() == EXTERNAL.len()
            && i.external_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == EXTERNAL.iter().copied().collect(),
        "external coverage",
    )?;
    validate_steps(i, kernel())?;
    validate_outputs(i)?;
    validate_boundaries(i, kernel())
}
type Texts = BTreeMap<String, String>;
fn ext<'a>(texts: &'a Texts, path: &str) -> Result<&'a str, String> {
    texts
        .get(path)
        .map(String::as_str)
        .ok_or_else(|| format!("external: {path}"))
}
fn verify_external(i: &Inventory, texts: &Texts) -> Check {
    for s in &i.external_sources {
        require(digest(ext(texts, &s.path)?) == s.sha256, "external drift")?;
    }
    let types = ext(texts, "upstream/_src/types.py")?;
    super::verify_external(
        &super::inventory(),
        types,
        ext(texts, "warp/_src/types.py")?,
    )?;
    for s in &i.steps {
        let text = ext(texts, &s.upstream_source)?;
        let section = if s.upstream_source.ends_with(".py") {
            body(text, &s.symbol)?
        } else {
            text
        };
        for a in &s.upstream {
            anchor(if a.starts_with("def ") { text } else { section }, a)?;
        }
    }
    let smooth = ext(texts, "upstream/_src/smooth.py")?;
    let data = types.split_once("class Data:").ok_or("Data class")?.1;
    for o in &i.outputs {
        anchor(data, &format!("  {}: {}\n", o.name, o.upstream_declaration))?;
        let section = body(smooth, &o.symbol)?;
        anchor(
            section,
            &format!("{}_out: wp.array2d[{}]", o.name, o.upstream_type),
        )?;
        anchor(section, &o.upstream_write)?;
    }
    // 原生头文件与注册文档相互矛盾，不能只取文档。
    let quat = ext(texts, "warp/native/quat.h")?;
    let normalize = between(quat, "quat_t<Type> normalize(const quat_t<Type>& q)", "\n}")?;
    anchor(normalize, "if (l > Type(kEps))")?;
    anchor(
        normalize,
        "return quat_t<Type>(Type(0), Type(0), Type(0), Type(1));",
    )?;
    anchor(
        quat,
        "quat_t(Type x = Type(0), Type y = Type(0), Type z = Type(0), Type w = Type(0))",
    )?;
    anchor(ext(texts, "warp/native/builtin.h")?, "#define kEps 0.0f")?;
    let builtin = ext(texts, "warp/_src/builtins.py")?;
    anchor(
        builtin,
        "\"normalize\",\n    input_types={\"a\": quaternion(dtype=Float)},",
    )?;
    anchor(builtin, "the zero quaternion is returned.")?;
    anchor(
        ext(texts, "upstream/_src/io.py")?,
        "ancestor_chain = lambda b: ancestor_chain(mjm.body_parentid[b]) + [b] if b else []",
    )?;
    anchor(
        body(smooth, "_kinematics_branch")?,
        "worldid, branchid = wp.tid()",
    )?;
    anchor(body(smooth, "kinematics")?, "dim=(d.nworld, m.nbranch)")?;
    Ok(())
}
fn capture_variant(text: &str) -> Result<(bool, bool), String> {
    let has = |s: &str| text.lines().any(|l| l == s);
    let batch = has("#define MJWARP_FIELD_BATCHES");
    let mocap = has("#define MJWARP_MOCAP");
    let equivalent = has("#define MJWARP_G01_EQUIVALENT");
    require(
        batch == mocap && (!equivalent || batch),
        "kernel macro combination",
    )?;
    Ok((batch, equivalent))
}
fn verify_captures(root: &Path) -> Result<usize, String> {
    let m: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let rows: Vec<_> = m["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .filter(|a| a["entry"] == "rigid_kinematics")
        .collect();
    require(rows.len() == 3, "rigid variants")?;
    let mut count = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut variants = BTreeSet::new();
        for a in &rows {
            let h = a["ptx_sha256"].as_str().ok_or("hash")?;
            require(
                h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let mut text = String::new();
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes =
                    std::fs::read(root.join(mode).join(format!("rigid_kinematics-{h}.{ext}")))
                        .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes))
                        == a[key].as_str().ok_or("artifact hash")?,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    text = String::from_utf8(bytes)
                        .map_err(|e| e.to_string())?
                        .replace("\r\n", "\n");
                } else if ext == "json" {
                    let r: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        r["entry"] == "rigid_kinematics"
                            && r["passed"] == true
                            && r["precisionGate"] == "f32-physics"
                            && r["fmad"] == true
                            && r["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            anchor(
                &text,
                &historical_rigid_kernel(kernel())?.replace("\r\n", "\n"),
            )?;
            require(
                variants.insert(capture_variant(&text)?),
                "duplicate variant",
            )?;
            count += 1;
        }
        require(
            variants
                == [(false, false), (true, false), (true, true)]
                    .into_iter()
                    .collect(),
            "active variant coverage",
        )?;
    }
    Ok(count)
}
#[test]
fn covers_seventeen_internal_groups_and_seven_outputs() {
    validate(&inventory()).unwrap();
    assert_eq!((STEPS.len(), OUTPUTS.len()), (17, 7));
}
#[test]
fn rejects_missing_retyped_or_reclassified_internal_groups() {
    for index in 0..STEPS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.steps.remove(index);
                }
                1 => i.steps[index].scalar = "f64".into(),
                2 => i.steps[index].classification = "equivalent".into(),
                3 => {
                    i.steps[index].product.pop();
                }
                _ => i.steps[index].product[0] = "wrong_product".into(),
            }
            assert!(validate_steps(&i, kernel()).is_err(), "{index}/{mutation}");
        }
    }
}
#[test]
fn rejects_missing_or_misrouted_output_fields() {
    for index in 0..OUTPUTS.len() {
        for mutation in 0..5 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.outputs.remove(index);
                }
                1 => i.outputs[index].scalar = "f64".into(),
                2 => i.outputs[index].view = "&v[..]".into(),
                3 => i.outputs[index].pointer = "wrong_pointer".into(),
                _ => i.outputs[index].write = "wrong_write".into(),
            }
            assert!(validate_outputs(&i).is_err());
        }
    }
}
#[test]
fn preserves_zero_quaternion_evidence_and_active_macros() {
    validate_boundaries(&inventory(), &kernel().replace('\n', "\r\n")).unwrap();
    for mutation in 0..5 {
        let mut i = inventory();
        match mutation {
            0 => i.zero_quaternion.status = "accepted".into(),
            1 => i.zero_quaternion.warp_header_return = [0; 4],
            2 => i.zero_quaternion.upstream_generated_verified = false,
            3 => i.equivalence_eligible = true,
            _ => i.zero_quaternion.product_return = [0; 4],
        }
        assert!(validate_boundaries(&i, kernel()).is_err());
    }
    assert!(
        validate_boundaries(
            &inventory(),
            &kernel().replace("if(n==0.0f) return {0,0,0,1};", "changed_fallback")
        )
        .is_err()
    );
    assert_eq!(
        capture_variant("// #define MJWARP_G01_EQUIVALENT\n#ifdef MJWARP_MOCAP").unwrap(),
        (false, false)
    );
    assert!(capture_variant("#define MJWARP_G01_EQUIVALENT").is_err());
    // 两段已测溯源，各自只回溯一个修复。
    // 第一段：旋转分组修复（8a5bac0）。整文件还原为c74aede版本。
    let pre_rotation = super::rotate_lineage::before(PHYSICS).unwrap();
    assert_eq!(
        digest(&pre_rotation),
        "71ee8dd5718779bc45430b95ba42c9995ed6c047e8590509ef3e68c4de8d0e6a"
    );
    // 第二段：零四元数分支。模板还原为历史实测捕获。
    // 该段之前的整文件差异只有模块导出，故比较刚体模板。
    let pre_zero = historical_rigid_kernel(
        between(&pre_rotation, "const KINEMATICS_CUDA: &str = r#\"", "\"#;").unwrap(),
    )
    .unwrap();
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json")).unwrap();
    let captured: Vec<_> = kernels["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["entry"] == "rigid_kinematics")
        .filter(|a| {
            a["ptx_sha256"] == "7d1db8ebffc8c03d39bec4b615e65847cca505e58dd8fb856ce5b624c3e03eb5"
        })
        .collect();
    assert_eq!(captured.len(), 1);
    assert_eq!(digest(&pre_zero), captured[0]["source_sha256"]);
    assert!(super::rotate_lineage::before(&PHYSICS.replace("2.0f*q.w", "2*q.w")).is_err());
    assert!(historical_rigid_kernel(&kernel().replace("{0,0,0,1}", "{1,0,0,0}")).is_err());
    let report: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/precision/g01-zero-quaternion-result.json"
    ))
    .unwrap();
    assert_eq!(report["zero_fallback"], serde_json::json!([0, 0, 0, 1]));
    assert_eq!(report["comparisons"], 608);
    assert_eq!(report["differences"], serde_json::json!([]));
    assert_eq!(report["upstream_revision"], super::REVISION);
    assert_eq!(
        report["upstream_ptx_sha256"],
        "1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd"
    );
    // 本次只刷新受影响的源码戳，不重跑旧整组配置。
    for text in [
        include_str!("../../fixtures/precision/g01-state-packing.json"),
        include_str!("../../fixtures/precision/g01-field-internal.json"),
        include_str!("../../fixtures/precision/g01-topology-frames.json"),
        FIXTURE,
    ] {
        let value: serde_json::Value = serde_json::from_str(text).unwrap();
        let stamp = value["product_sources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["path"] == "src/physics/mod.rs")
            .unwrap();
        assert_eq!(stamp["sha256"], digest(PHYSICS));
    }
}
#[test]
fn rejects_source_drift_and_premature_precision_acceptance() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    for mutation in 0..4 {
        let mut i = inventory();
        match mutation {
            0 => i.np01_complete = true,
            1 => i.np02_complete = true,
            2 => {
                i.external_sources.pop();
            }
            _ => i.remaining.clear(),
        }
        assert!(validate(&i).is_err());
    }
}
#[test]
#[ignore = "reads frozen sources and existing captures without running Python or GPU"]
fn audit_frozen_rigid_internal_and_output_sources() {
    let i = inventory();
    validate(&i).unwrap();
    let mut texts = Texts::new();
    for s in &i.external_sources {
        let (root, path) = s.path.split_once('/').unwrap();
        let env = if root == "upstream" {
            "MJWARP_FROZEN_UPSTREAM_ROOT"
        } else {
            "MJWARP_FROZEN_WARP_ROOT"
        };
        let root = std::env::var_os(env).expect(env);
        texts.insert(
            s.path.clone(),
            std::fs::read_to_string(Path::new(&root).join(path))
                .unwrap()
                .replace("\r\n", "\n"),
        );
    }
    verify_external(&i, &texts).unwrap();
    let mut changed = i.clone();
    changed.steps[0].upstream[0] = "wrong_upstream".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = i.clone();
    changed.outputs[0].upstream_declaration = "array(\"nworld\", \"nbody\", float)".into();
    assert!(verify_external(&changed, &texts).is_err());
    let mut changed = texts.clone();
    changed.get_mut("warp/native/quat.h").unwrap().push('\n');
    assert!(verify_external(&i, &changed).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("evidence root");
    let captures = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1,"scope":i.scope,"inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
        "internal_groups":i.steps.len(),"output_fields":i.outputs.len(),"historical_captured_variants":captures,
        "current_full_capture_verified":false,
        "zero_quaternion_status":i.zero_quaternion.status,"equivalence_eligible":false,
        "np01_complete":false,"np02_complete":false,"np03":"historical-unchanged","np04":"not-measured",
        "gpu_executions":0,"python_executions":0,"remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_RIGID_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
