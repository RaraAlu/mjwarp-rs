//! 编译产物复算与选择性重采集测试。

use std::time::Instant;

use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts};

use super::lineage::lineage;
use super::recipe::{copy_artifact, recipe, rotate_recipe};
use super::*;

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
