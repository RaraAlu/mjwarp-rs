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
const REVISION: &str = "934667a8e958155d4fe7e033e148978d498778e4";
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const RUNTIME: &str = include_str!("../../src/runtime/transfer/kernel.rs");
const HOOK: &str =
    "\n#[cfg(test)]\n#[path = \"../../../tests/support/t4_rigid_capture.rs\"]\nmod recapture;\n";
const ENTRY: &str = "rigid_kinematics";

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
            s.replace(
                "if(n==0.0f) return {0,0,0,1};",
                "if(n==0.0f) return {0,0,0,0};",
            )
            .as_bytes(),
        ) == old.source_sha256
    });
    let source = matches.next().expect("unmapped embedded rigid template");
    assert!(matches.next().is_none());
    Some((entry, source))
}

/// 保留十份刚体更新；另允许一份面算子修复。
/// 其他字段、参考、变体、排除项和未验收边界全部继承。
fn lineage(value: &Inventory) -> Result<(), String> {
    validate_inventory(value)?;
    let mut expected = base();
    expected.source_revision = REVISION.into();
    for (path, text) in [
        ("src/physics/mod.rs", PHYSICS),
        ("src/runtime/transfer/kernel.rs", RUNTIME),
        ("src/physics/flex_face.rs", FACE),
    ] {
        expected
            .product_sources
            .iter_mut()
            .find(|s| s.path == path)
            .unwrap()
            .sha256 = digest(normalized(text).as_bytes());
    }
    let mut replacements = 0;
    for (old, new) in expected.artifacts.iter_mut().zip(&value.artifacts) {
        if let Some((entry, source)) = recipe(old) {
            if new.entry != entry || digest(source.as_bytes()) != new.source_sha256 {
                return Err("rigid template coverage drift".into());
            }
            *old = new.clone();
            replacements += 1;
        }
    }
    if replacements != 11
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
    for stamp in &value.product_sources {
        let text = fs::read_to_string(root().join(&stamp.path)).unwrap();
        assert_eq!(
            digest(normalized(&text).as_bytes()),
            stamp.sha256,
            "{}",
            stamp.path
        );
    }
    let b = base();
    let zero: serde_json::Value = serde_json::from_str(include_str!(
        "../../fixtures/precision/g01-zero-quaternion-result.json"
    ))
    .unwrap();
    let complete_hash = digest(sources()[2].as_bytes());
    for (old, new) in b
        .artifacts
        .iter()
        .zip(&value.artifacts)
        .filter(|(_, a)| a.entry == ENTRY)
    {
        if new.source_sha256 == complete_hash {
            // 将本轮完整入口产物绑定到前轮608项实测产物。
            assert_eq!(new.ptx_sha256, zero["product_ptx_sha256"].as_str().unwrap());
        } else {
            // 严格入口排除零分支，实际PTX必须保持原字节。
            assert_eq!(new.ptx_sha256, old.ptx_sha256);
        }
    }
    let old_hash = |path| {
        &b.product_sources
            .iter()
            .find(|s| s.path == path)
            .unwrap()
            .sha256
    };
    let runtime = normalized(RUNTIME);
    assert_eq!(runtime.matches(HOOK).count(), 1);
    assert_eq!(
        &digest(runtime.replace(HOOK, "").as_bytes()),
        old_hash("src/runtime/transfer/kernel.rs")
    );
    assert_eq!(
        &digest(
            normalized(PHYSICS)
                .replace(
                    "if(n==0.0f) return {0,0,0,1};",
                    "if(n==0.0f) return {0,0,0,0};"
                )
                .as_bytes()
        ),
        old_hash("src/physics/mod.rs")
    );
    // 编译选项必须同时匹配实际产品调用，禁止诊断侧另选精度。
    for anchor in [
        "fmad: Some(PHYSICS_FMAD)",
        "use_fast_math: Some(false)",
        "options: vec![\"--std=c++17\".into(), \"--gpu-architecture=compute_70\".into()]",
    ] {
        assert!(RUNTIME.contains(anchor));
    }
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
