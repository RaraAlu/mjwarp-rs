//! 仅刷新编译产物；不建立GPU会话，不执行物理内核。
use super::{PHYSICS_FMAD, check_physics_ptx, record_physics_ptx};
use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};
use std::{fs, path::PathBuf, time::Instant};

#[path = "t4_face_lineage.rs"]
mod face_lineage;
const FACE: &str = include_str!("../../src/physics/flex_face.rs");

// 复用完整证据校验器；本切片不调用其性能接口。
#[allow(dead_code)]
#[path = "t4_evidence.rs"]
mod evidence;
use evidence::{Inventory, digest, validate_inventory, verify_capture, verify_sources};

const BASE: &str = include_str!("../../fixtures/precision/t4-kernels.json");
const PRE_ROTATE: &str =
    include_str!("../../docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json");
const SOURCE_IDENTITY: &str =
    include_str!("../../docs/evidence/t4-g03-retest/rigid-rotate-recapture/source-identity.json");
const REVISION: &str = "8a5bac00fdfa9759f752993b2866098c6ff20819";
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const COM_VELOCITY: &str = include_str!("../../src/physics/com_velocity.rs");
const FIXED_TENDON: &str = include_str!("../../src/physics/fixed_tendon.rs");
const FLEX: &str = include_str!("../../src/physics/flex.rs");
const FLEX_EDGE: &str = include_str!("../../src/physics/flex_edge.rs");
const FLEX_HESSIAN: &str = include_str!("../../src/physics/flex_hessian.rs");
const MASS_MATRIX: &str = include_str!("../../src/physics/mass_matrix.rs");
const MASS_SOLVE: &str = include_str!("../../src/physics/mass_solve.rs");
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
    source
        .split_once(&format!("const {name}: &str = r#\""))
        .unwrap()
        .1
        .split_once("\"#;")
        .unwrap()
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

// 所有嵌入刚体模板的入口都要重编译，不能只看入口名称。
fn recipe(old: &evidence::Artifact) -> Option<(&'static str, String)> {
    if old.entry == "flex_faces" {
        let source = normalized(template(FACE, "FLEX_FACE_CUDA"));
        assert_eq!(
            digest(face_lineage::historical(&source).unwrap().as_bytes()),
            old.source_sha256
        );
        return Some(("flex_faces", source));
    }
    let rigid = sources();
    let (entry, candidates): (_, Vec<String>) = match old.entry.as_str() {
        "rigid_kinematics" => (ENTRY, rigid.into()),
        "attached_kinematics" | "initialize_attached" => {
            let attached = normalized(template(
                include_str!("../../src/physics/attached.rs"),
                "ATTACHED_CUDA",
            ));
            (
                if old.entry == "attached_kinematics" {
                    "attached_kinematics"
                } else {
                    "initialize_attached"
                },
                rigid.iter().map(|r| format!("{r}\n{attached}")).collect(),
            )
        }
        "camlight_com" | "camlight_rigid" => {
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            let camera = normalized(template(
                include_str!("../../src/physics/camlight.rs"),
                "CAMLIGHT_CUDA",
            ));
            (
                if old.entry == "camlight_com" {
                    "camlight_com"
                } else {
                    "camlight_rigid"
                },
                vec![format!("{fields}\n{}\n{camera}", rigid[0])],
            )
        }
        _ => return None,
    };
    let mut matches = candidates.into_iter().filter(|s| {
        digest(
            before_rotate(&s.replace(
                "if(n==0.0f) return {0,0,0,1};",
                "if(n==0.0f) return {0,0,0,0};",
            ))
            .as_bytes(),
        ) == old.source_sha256
    });
    let source = matches.next().expect("unmapped embedded rigid template");
    assert!(matches.next().is_none());
    Some((entry, source))
}

fn rotate_recipe(old: &evidence::Artifact) -> Option<(&'static str, String)> {
    if old.entry == "flex_faces" {
        return None;
    }
    let rigid = sources();
    let (entry, candidates): (_, Vec<String>) = match old.entry.as_str() {
        "rigid_kinematics" => (ENTRY, rigid.into()),
        "attached_kinematics" | "initialize_attached" => {
            let attached = normalized(template(
                include_str!("../../src/physics/attached.rs"),
                "ATTACHED_CUDA",
            ));
            (
                if old.entry == "attached_kinematics" {
                    "attached_kinematics"
                } else {
                    "initialize_attached"
                },
                rigid.iter().map(|r| format!("{r}\n{attached}")).collect(),
            )
        }
        "camlight_com" | "camlight_rigid" => {
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            let camera = normalized(template(
                include_str!("../../src/physics/camlight.rs"),
                "CAMLIGHT_CUDA",
            ));
            (
                if old.entry == "camlight_com" {
                    "camlight_com"
                } else {
                    "camlight_rigid"
                },
                vec![format!("{fields}\n{}\n{camera}", rigid[0])],
            )
        }
        _ => return None,
    };
    let mut matches = candidates
        .into_iter()
        .filter(|source| digest(before_rotate(source).as_bytes()) == old.source_sha256);
    let source = matches.next().expect("unmapped pre-rotation variant");
    assert!(matches.next().is_none());
    Some((entry, source))
}

/// 逐项重建二十份沿用模板。
/// 它验证当前调用模板没有漂移。
fn retained_recipe(old: &evidence::Artifact) -> Option<String> {
    let direct = match old.entry.as_str() {
        "com_velocity" => Some(template(COM_VELOCITY, "COM_VELOCITY_CUDA")),
        "fixed_tendon" => Some(template(FIXED_TENDON, "FIXED_TENDON_CUDA")),
        "flex_edges" => Some(template(FLEX_EDGE, "FLEX_EDGE_CUDA")),
        "flex_faces" => Some(template(FACE, "FLEX_FACE_CUDA")),
        "flex_hessian" | "flex_hessian_validate" => {
            Some(template(FLEX_HESSIAN, "FLEX_HESSIAN_CUDA"))
        }
        "flex_invalidate" | "flex_positions" => Some(template(FLEX, "FLEX_POSITION_CUDA")),
        "rigid_mass_matrix" => Some(template(MASS_MATRIX, "MASS_MATRIX_CUDA")),
        "rigid_mass_solve" => Some(template(MASS_SOLVE, "MASS_SOLVE_CUDA")),
        "rne_bias" => Some(template(RNE_BIAS, "RNE_BIAS_CUDA")),
        "sleep_copy" | "tendon_wake" => Some(template(SLEEP, "TENDON_WAKE_CUDA")),
        "tendon_indices" | "tendon_values" => Some(template(TENDON, "TENDON_CUDA")),
        _ => None,
    };
    if let Some(source) = direct {
        return Some(normalized(source));
    }
    let candidates = match old.entry.as_str() {
        "rigid_com_position" => {
            let com = normalized(template(PHYSICS, "COM_POSITION_CUDA"));
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            vec![com.clone(), format!("{fields}\n{com}")]
        }
        "spatial_moment" | "spatial_site" | "spatial_wrap" => {
            let wrap = normalized(template(TENDON_WRAP, "TENDON_WRAP_CUDA"));
            let spatial = normalized(template(SPATIAL_TENDON, "SPATIAL_TENDON_CUDA"));
            vec![format!("{wrap}\n{spatial}")]
        }
        _ => return None,
    };
    let mut matches = candidates
        .into_iter()
        .filter(|source| digest(source.as_bytes()) == old.source_sha256);
    let source = matches.next().expect("unmapped retained template");
    assert!(matches.next().is_none());
    Some(source)
}

fn copy_artifact(from: &std::path::Path, to: &std::path::Path, artifact: &evidence::Artifact) {
    for ext in ["cu", "ptx", "json"] {
        let name = format!("{}-{}.{}", artifact.entry, artifact.ptx_sha256, ext);
        fs::copy(from.join(&name), to.join(name)).unwrap();
    }
}

/// 保留十份刚体更新；另允许一份面算子修复。
/// 其他字段、参考、变体、排除项和未验收边界全部继承。
fn lineage(value: &Inventory) -> Result<(), String> {
    validate_inventory(value)?;
    let mut expected: Inventory = serde_json::from_str(PRE_ROTATE).unwrap();
    expected.source_revision = REVISION.into();
    expected
        .product_sources
        .iter_mut()
        .find(|stamp| stamp.path == "src/physics/mod.rs")
        .unwrap()
        .sha256 = digest(normalized(PHYSICS).as_bytes());
    let mut replacements = 0;
    let mut retained = 0;
    for (old, new) in expected.artifacts.iter_mut().zip(&value.artifacts) {
        if let Some((entry, source)) = rotate_recipe(old) {
            if new.entry != entry || digest(source.as_bytes()) != new.source_sha256 {
                return Err("rigid template coverage drift".into());
            }
            *old = new.clone();
            replacements += 1;
        } else {
            let source = retained_recipe(old).ok_or("unmapped retained variant")?;
            if digest(source.as_bytes()) != old.source_sha256
                || old.entry != new.entry
                || old.source_sha256 != new.source_sha256
                || old.ptx_sha256 != new.ptx_sha256
                || old.receipt_sha256 != new.receipt_sha256
            {
                return Err("retained template coverage drift".into());
            }
            retained += 1;
        }
    }
    if replacements != 10
        || retained != 20
        || serde_json::to_value(expected).unwrap() != serde_json::to_value(value).unwrap()
    {
        return Err("unexpected lineage change".into());
    }
    Ok(())
}

#[test]
fn preserves_current_sources_and_all_thirty_variants() {
    let value = current();
    lineage(&value).unwrap();
    let physics = value
        .product_sources
        .iter()
        .find(|stamp| stamp.path == "src/physics/mod.rs")
        .unwrap();
    assert_eq!(physics.sha256, digest(normalized(PHYSICS).as_bytes()));
    let rigid_hashes = sources().map(|source| digest(source.as_bytes()));
    let recorded: Vec<_> = value
        .artifacts
        .iter()
        .filter(|artifact| artifact.entry == ENTRY)
        .map(|artifact| artifact.source_sha256.as_str())
        .collect();
    assert_eq!(recorded.len(), 3);
    for hash in &rigid_hashes {
        assert!(recorded.contains(&hash.as_str()));
    }
    assert_eq!(normalized(RUNTIME).matches(HOOK).count(), 1);
    // 编译选项必须同时匹配实际产品调用，禁止诊断侧另选精度。
    for anchor in [
        "fmad: Some(PHYSICS_FMAD)",
        "use_fast_math: Some(false)",
        "options: vec![\"--std=c++17\".into(), \"--gpu-architecture=compute_70\".into()]",
    ] {
        assert!(RUNTIME.contains(anchor));
    }
    // 同时绑定模板组合与实际编译调用点。
    for anchor in [
        "let source = format!(\"{FIELD_BATCH_CUDA}\\n{}\", super::COM_POSITION_CUDA);",
        "super::fixed_tendon::FIXED_TENDON_CUDA,",
        "super::tendon_wrap::TENDON_WRAP_CUDA,",
        "super::spatial_tendon::SPATIAL_TENDON_CUDA",
        "super::tendon::TENDON_CUDA,",
        "super::flex::FLEX_POSITION_CUDA,",
        "super::sleep::TENDON_WAKE_CUDA,",
        "super::flex_edge::FLEX_EDGE_CUDA,",
        "super::flex_face::FLEX_FACE_CUDA,",
        "super::flex_hessian::FLEX_HESSIAN_CUDA,",
    ] {
        assert!(
            RESIDENT.contains(anchor),
            "missing resident route: {anchor}"
        );
    }
    for (source, anchor) in [
        (COM_VELOCITY, "COM_VELOCITY_CUDA, \"com_velocity\""),
        (MASS_MATRIX, "MASS_MATRIX_CUDA, \"rigid_mass_matrix\""),
        (MASS_SOLVE, "MASS_SOLVE_CUDA, \"rigid_mass_solve\""),
        (RNE_BIAS, "RNE_BIAS_CUDA, \"rne_bias\""),
    ] {
        assert!(source.contains(anchor), "missing direct route: {anchor}");
    }
    let identity: serde_json::Value = serde_json::from_str(SOURCE_IDENTITY).unwrap();
    assert_eq!(identity["variants"], 30);
    assert_eq!(identity["affected_variants"], 10);
    assert_eq!(identity["retained_variants"], 20);
    assert_eq!(identity["retained_template_matches"], 20);
    let mut expected: Vec<_> = value
        .artifacts
        .iter()
        .filter(|artifact| retained_recipe(artifact).is_some())
        .map(|artifact| {
            (
                artifact.entry.clone(),
                artifact.source_sha256.clone(),
                artifact.ptx_sha256.clone(),
            )
        })
        .collect();
    let mut observed: Vec<_> = identity["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["entry"].as_str().unwrap().to_owned(),
                row["source_sha256"].as_str().unwrap().to_owned(),
                row["ptx_sha256"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    expected.sort();
    observed.sort();
    assert_eq!(observed, expected);
}

#[test]
fn rejects_unrelated_drift_and_missing_rigid_variants() {
    let value = current();
    for n in 0..7 {
        let mut changed = value.clone();
        match n {
            0 => {
                changed.artifacts.pop();
            }
            1 => changed.artifacts[0].source_sha256 = "0".repeat(64),
            2 => {
                changed
                    .artifacts
                    .iter_mut()
                    .find(|a| a.entry == ENTRY)
                    .unwrap()
                    .source_sha256 = "0".repeat(64)
            }
            3 => changed.remaining.clear(),
            4 => changed.mappings[0].fields.pop().map(|_| ()).unwrap(),
            5 => changed.source_revision = "0".repeat(40),
            _ => changed.artifacts[0] = base().artifacts[0].clone(),
        }
        assert!(lineage(&changed).is_err(), "mutation {n}");
    }
}

#[test]
#[ignore = "NVRTC compile-only recapture; requires historical captures and frozen upstream sources"]
fn recompile_rigid_variants_without_launching_physics() {
    let started = Instant::now();
    let out =
        PathBuf::from(std::env::var_os("MJWARP_PHYSICS_PTX_DIR").expect("new capture directory"));
    // 新目录必须位于target，且必须不存在。绝不覆盖历史产物。
    assert!(out.is_absolute());
    let target = root().join("target").canonicalize().unwrap();
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&target)
    );
    assert!(!out.exists(), "refuse to overwrite captures");
    let history = root().join("target/t4-g03-retest");
    let upstream = root().join("target/mjwarp-parallel/upstream/mujoco_warp");
    let old = base();
    for mode in ["debug-ptx", "release-ptx"] {
        verify_capture(&old, &history.join(mode)).unwrap();
    }
    fs::create_dir(&out).unwrap();
    let mut value = old.clone();
    value.source_revision = REVISION.into();
    for stamp in &mut value.product_sources {
        if [
            "src/physics/mod.rs",
            "src/runtime/transfer/kernel.rs",
            "src/physics/flex_face.rs",
        ]
        .contains(&stamp.path.as_str())
        {
            stamp.sha256 = digest(
                normalized(&fs::read_to_string(root().join(&stamp.path)).unwrap()).as_bytes(),
            );
        }
    }
    verify_sources(&value, &root(), &upstream).unwrap();
    let mut compiled = Vec::new();
    for row in &mut value.artifacts {
        let file = format!("{}-{}.cu", row.entry, row.ptx_sha256);
        let old_cu = fs::read_to_string(history.join("debug-ptx").join(file)).unwrap();
        let plan = recipe(row);
        assert_eq!(
            plan.is_some(),
            old_cu.contains("if(n==0.0f) return {0,0,0,0};") || row.entry == "flex_faces",
            "unmapped shared template"
        );
        let Some((entry, source)) = plan else {
            for ext in ["cu", "ptx", "json"] {
                let file = format!("{}-{}.{ext}", row.entry, row.ptx_sha256);
                fs::copy(history.join("debug-ptx").join(&file), out.join(file)).unwrap();
            }
            continue;
        };
        let historical = if entry == "flex_faces" {
            face_lineage::historical(&source).unwrap()
        } else {
            source.replace(
                "if(n==0.0f) return {0,0,0,1};",
                "if(n==0.0f) return {0,0,0,0};",
            )
        };
        assert_eq!(normalized(&old_cu), historical);
        if entry != "flex_faces" {
            assert_eq!(
                normalized(&old_cu).replace(
                    "if(n==0.0f) return {0,0,0,0};",
                    "if(n==0.0f) return {0,0,0,1};"
                ),
                source
            );
        }
        let ptx = compile_ptx_with_opts(
            &source,
            CompileOptions {
                fmad: Some(PHYSICS_FMAD),
                use_fast_math: Some(false),
                options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
                name: Some(format!("{entry}.cu")),
                ..Default::default()
            },
        )
        .unwrap()
        .to_src();
        check_physics_ptx(&ptx, entry).unwrap();
        record_physics_ptx(&source, &ptx, entry, true).unwrap();
        let ptx_hash = digest(ptx.as_bytes());
        let receipt = fs::read(out.join(format!("{entry}-{ptx_hash}.json"))).unwrap();
        *row = evidence::Artifact {
            entry: entry.into(),
            source_sha256: digest(source.as_bytes()),
            ptx_sha256: ptx_hash,
            receipt_sha256: digest(&receipt),
        };
        compiled.push(row.clone());
    }
    assert_eq!(compiled.len(), 11);
    lineage(&value).unwrap();
    verify_capture(&value, &out).unwrap();
    // 所有入口重新执行原精度门禁；不只信任历史passed字段。
    for row in &value.artifacts {
        let ptx =
            fs::read_to_string(out.join(format!("{}-{}.ptx", row.entry, row.ptx_sha256))).unwrap();
        check_physics_ptx(&ptx, "current-capture").unwrap();
    }
    // 在独立新目录做文件反例；保留原始捕获。
    let first = compiled.iter().find(|a| a.entry == ENTRY).unwrap();
    for ext in ["cu", "ptx", "json"] {
        let path = out.join(format!("{ENTRY}-{}.{ext}", first.ptx_sha256));
        let saved = fs::read(&path).unwrap();
        fs::write(&path, b"corrupt").unwrap();
        assert!(verify_capture(&value, &out).is_err());
        fs::write(&path, saved).unwrap();
    }
    verify_capture(&value, &out).unwrap();
    let inventory = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(
        out.parent().unwrap().join("t4-current-kernels.json"),
        &inventory,
    )
    .unwrap();
    let report = serde_json::json!({"schema":1,"scope":"current-product-compile-only-lineage","source_revision":REVISION,"historical_inventory_sha256":digest(normalized(BASE).as_bytes()),"current_inventory_sha256":digest(&inventory),"kernel_entries":24,"variants":30,"recompiled_rigid_variants":3,"recompiled_dependent_variants":7,"recompiled_face_variants":1,"retained_verified_variants":19,"historical_modes_verified":["debug","release"],"new_capture_mode":"compile-only; not debug/release physical runs","ptx_precision_gate_passes":30,"corruption_rejections":3,"product_gpu_executions":0,"python_executions":0,"elapsed_ms":started.elapsed().as_millis(),"np01_complete":false,"np02_complete":false,"np03":"historical-not-rerun","np04_measured":false,"full_t4_admitted":false});
    fs::write(
        out.parent().unwrap().join("t4-rigid-recapture-audit.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}

#[test]
#[ignore = "NVRTC selective recapture after rigid rotation grouping change"]
fn recaptures_only_missing_rotation_variants() {
    let started = Instant::now();
    let out =
        PathBuf::from(std::env::var_os("MJWARP_PHYSICS_PTX_DIR").expect("new capture directory"));
    assert!(out.is_absolute() && !out.exists());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(root().join("target").canonicalize().unwrap())
    );
    let previous_dir = root().join("target/t4-final-face-compile/current");
    let reusable_dir = root().join("docs/evidence/gravity-nonzero-product-chain/gpu-2/ptx");
    let previous = current();
    verify_capture(&previous, &previous_dir).unwrap();
    fs::create_dir(&out).unwrap();
    let mut value = previous.clone();
    value.source_revision = REVISION.into();
    value
        .product_sources
        .iter_mut()
        .find(|stamp| stamp.path == "src/physics/mod.rs")
        .unwrap()
        .sha256 = digest(normalized(PHYSICS).as_bytes());
    let mut affected = 0;
    let mut reused = 0;
    let mut compiled = 0;
    let mut retained = 0;
    for row in &mut value.artifacts {
        let old = row.clone();
        let Some((entry, source)) = rotate_recipe(&old) else {
            copy_artifact(&previous_dir, &out, &old);
            retained += 1;
            continue;
        };
        affected += 1;
        let source_hash = digest(source.as_bytes());
        let reusable = fs::read_dir(&reusable_dir).unwrap().find_map(|item| {
            let path = item.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                return None;
            }
            let bytes = fs::read(&path).unwrap();
            let meta: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            if meta["entry"].as_str() != Some(entry)
                || meta["sourceSha256"].as_str() != Some(&source_hash)
            {
                return None;
            }
            let ptx_hash = meta["ptxSha256"].as_str().unwrap().to_owned();
            Some(evidence::Artifact {
                entry: entry.to_owned(),
                source_sha256: source_hash.clone(),
                ptx_sha256: ptx_hash,
                receipt_sha256: digest(&bytes),
            })
        });
        if let Some(artifact) = reusable {
            copy_artifact(&reusable_dir, &out, &artifact);
            *row = artifact;
            reused += 1;
            continue;
        }
        let ptx = compile_ptx_with_opts(
            &source,
            CompileOptions {
                fmad: Some(PHYSICS_FMAD),
                use_fast_math: Some(false),
                options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
                name: Some(format!("{entry}.cu")),
                ..Default::default()
            },
        )
        .unwrap()
        .to_src();
        check_physics_ptx(&ptx, entry).unwrap();
        record_physics_ptx(&source, &ptx, entry, true).unwrap();
        let ptx_hash = digest(ptx.as_bytes());
        let receipt = fs::read(out.join(format!("{entry}-{ptx_hash}.json"))).unwrap();
        *row = evidence::Artifact {
            entry: entry.to_owned(),
            source_sha256: source_hash,
            ptx_sha256: ptx_hash,
            receipt_sha256: digest(&receipt),
        };
        compiled += 1;
    }
    assert_eq!((affected, reused, compiled, retained), (10, 3, 7, 20));
    lineage(&value).unwrap();
    verify_capture(&value, &out).unwrap();
    for artifact in &value.artifacts {
        let ptx =
            fs::read_to_string(out.join(format!("{}-{}.ptx", artifact.entry, artifact.ptx_sha256)))
                .unwrap();
        check_physics_ptx(&ptx, "current-rotation-capture").unwrap();
    }
    let first = value.artifacts.first().unwrap();
    for ext in ["cu", "ptx", "json"] {
        let negative = out.parent().unwrap().join(format!("negative-copy-{ext}"));
        assert!(!negative.exists());
        fs::create_dir(&negative).unwrap();
        for item in fs::read_dir(&out).unwrap() {
            let path = item.unwrap().path();
            fs::copy(&path, negative.join(path.file_name().unwrap())).unwrap();
        }
        fs::write(
            negative.join(format!("{}-{}.{}", first.entry, first.ptx_sha256, ext)),
            b"corrupt",
        )
        .unwrap();
        assert!(verify_capture(&value, &negative).is_err());
        fs::remove_dir_all(&negative).unwrap();
    }
    verify_capture(&value, &out).unwrap();
    let inventory = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(
        out.parent().unwrap().join("t4-current-kernels.json"),
        &inventory,
    )
    .unwrap();
    let report = serde_json::json!({
        "schema": 1,
        "scope": "rigid-rotation-grouping-selective-recapture",
        "source_revision": REVISION,
        "current_inventory_sha256": digest(&inventory),
        "variants": 30,
        "affected_variants": affected,
        "reused_current_captures": reused,
        "new_compiler_calls": compiled,
        "retained_verified_variants": retained,
        "precision_gate_passes": 30,
        "negative_copy_rejections": 3,
        "original_capture_mutations": 0,
        "physical_kernel_launches": 0,
        "nonzero_numeric_selector_reruns": 0,
        "elapsed_ms": started.elapsed().as_millis(),
        "full_np_accepted": false
    });
    fs::write(
        out.parent().unwrap().join("audit.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!("{report}");
}

#[test]
#[ignore = "archive already measured face artifact; no compiler or GPU replay"]
fn archives_face_operator_repair_with_failed_numeric_gate() {
    let out = PathBuf::from(std::env::var_os("MJWARP_PHYSICS_PTX_DIR").unwrap());
    assert!(out.is_absolute() && !out.exists());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(root().join("target").canonicalize().unwrap())
    );
    let previous = root().join("target/t4-rigid-recapture/with-dependents/current");
    let mut value: Inventory = serde_json::from_str(include_str!(
        "../../docs/evidence/t4-g03-retest/face-operators/previous-kernels.json"
    ))
    .unwrap();
    verify_capture(&value, &previous).unwrap();
    let source = fs::read_to_string(root().join("target/t4-face-operators/product.cu")).unwrap();
    let ptx = fs::read_to_string(root().join("target/t4-face-operators/product.ptx")).unwrap();
    assert_eq!(source, template(&normalized(FACE), "FLEX_FACE_CUDA"));
    let result: serde_json::Value = serde_json::from_slice(
        &fs::read(root().join("target/t4-face-operators/factorial.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["product_cuda_sha256"], digest(source.as_bytes()));
    assert_eq!(result["product_ptx_sha256"], digest(ptx.as_bytes()));
    // 保留失败，不把编译证据转成数值准入。
    assert_eq!(result["repaired_product"]["bit_differences"], 2078);
    assert_eq!(result["full_acceptance"], false);
    let historic = face_lineage::historical(FACE).unwrap();
    let old = base();
    let old_stamp = old
        .product_sources
        .iter()
        .find(|s| s.path == "src/physics/flex_face.rs")
        .unwrap();
    assert_eq!(digest(historic.as_bytes()), old_stamp.sha256);
    for block in [
        face_lineage::NEW_R,
        face_lineage::FIX_M,
        face_lineage::NEW_N,
    ] {
        assert!(face_lineage::historical(&normalized(FACE).replace(block, "changed")).is_err());
    }
    fs::create_dir(&out).unwrap();
    let mut retained = 0;
    for row in &value.artifacts {
        if row.entry != "flex_faces" {
            for ext in ["cu", "ptx", "json"] {
                let name = format!("{}-{}.{ext}", row.entry, row.ptx_sha256);
                fs::copy(previous.join(&name), out.join(name)).unwrap();
            }
            retained += 1;
        }
    }
    assert_eq!(retained, 29);
    check_physics_ptx(&ptx, "flex_faces").unwrap();
    record_physics_ptx(&source, &ptx, "flex_faces", true).unwrap();
    let row = value
        .artifacts
        .iter_mut()
        .find(|a| a.entry == "flex_faces")
        .unwrap();
    row.source_sha256 = digest(source.as_bytes());
    row.ptx_sha256 = digest(ptx.as_bytes());
    row.receipt_sha256 =
        digest(&fs::read(out.join(format!("flex_faces-{}.json", row.ptx_sha256))).unwrap());
    value
        .product_sources
        .iter_mut()
        .find(|s| s.path == "src/physics/flex_face.rs")
        .unwrap()
        .sha256 = digest(normalized(FACE).as_bytes());
    value.source_revision = REVISION.into();
    lineage(&value).unwrap();
    verify_sources(
        &value,
        &root(),
        &root().join("target/mjwarp-parallel/upstream/mujoco_warp"),
    )
    .unwrap();
    verify_capture(&value, &out).unwrap();
    for artifact in &value.artifacts {
        let ptx =
            fs::read_to_string(out.join(format!("{}-{}.ptx", artifact.entry, artifact.ptx_sha256)))
                .unwrap();
        check_physics_ptx(&ptx, "face-current-capture").unwrap();
    }
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(
        out.parent().unwrap().join("t4-current-kernels.json"),
        &bytes,
    )
    .unwrap();
    let audit = serde_json::json!({"scope":"face-repair-artifact-lineage-only","source_revision":REVISION,"current_inventory_sha256":digest(&bytes),"entries":24,"variants":30,"retained_variants":29,"updated_face_variants":1,"ptx_precision_gate_passes":30,"lineage_mutation_rejections":3,"new_compiler_calls":0,"gpu_executions":0,"numerical_gate":"failed-2078-bit-differences","full_acceptance":false});
    fs::write(
        out.parent().unwrap().join("capture.json"),
        serde_json::to_string_pretty(&audit).unwrap() + "\n",
    )
    .unwrap();
}
