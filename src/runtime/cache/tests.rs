use super::*;
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mjwarp-cache-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture(config: ProbeConfig) -> Payload {
    Payload {
        key: Key::new(config, (8, 9), 13020),
        stages: entrypoints(config)
            .into_iter()
            .map(|name| Stage {
                entrypoint: name.into(),
                abi: abi(config.backend).into(),
                block_threads: BLOCK_THREADS as u32,
                shared_memory: 0,
                ptx: ".version 6.0\n".into(),
            })
            .collect(),
    }
}

#[test]
fn fingerprints_recipe_source_dependencies_and_device() {
    let config = ProbeConfig::default();
    let key = fixture(config).key;
    let mut variants = vec![
        Key::new(config, (9, 0), 13020),
        Key::new(config, (8, 9), 13010),
        Key::new(
            ProbeConfig {
                backend: ProbeBackend::CubeClCpp,
                ..config
            },
            (8, 9),
            13020,
        ),
        Key::new(
            ProbeConfig {
                kernel: ProbeKernel::BlockScan,
                ..config
            },
            (8, 9),
            13020,
        ),
    ];
    for field in 0..5 {
        let mut changed = key.clone();
        match field {
            0 => changed.schema += 1,
            1 => changed.recipe.push('x'),
            2 => changed.source_dependencies_sha256.push('x'),
            3 => changed.os.push('x'),
            _ => changed.arch.push('x'),
        }
        variants.push(changed);
    }
    for changed in variants {
        assert_ne!(key.hash().unwrap(), changed.hash().unwrap());
    }
    assert_eq!(
        key,
        Key::new(
            ProbeConfig {
                elements: 1,
                replays: 1000,
                ..config
            },
            (8, 9),
            13020
        )
    );
}

#[test]
fn roundtrips_all_routes_and_scan_stages() {
    for backend in [
        ProbeBackend::NativePtx,
        ProbeBackend::CubeClCpp,
        ProbeBackend::CubeClLlvm,
    ] {
        for kernel in ProbeKernel::ALL {
            if backend == ProbeBackend::NativePtx && kernel != ProbeKernel::Affine {
                continue;
            }
            let config = ProbeConfig {
                backend,
                kernel,
                ..Default::default()
            };
            let payload = fixture(config);
            let read = decode(&payload.encode().unwrap(), &payload.key, config).unwrap();
            assert_eq!(
                read.stages.len(),
                if kernel == ProbeKernel::GlobalScan {
                    3
                } else {
                    1
                }
            );
        }
    }
}

#[test]
fn rejects_unknown_fields_duplicate_fields_and_bad_json() {
    let config = ProbeConfig::default();
    let payload = fixture(config);
    let bytes = payload.encode().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["extra"] = true.into();
    assert!(decode(&serde_json::to_vec(&value).unwrap(), &payload.key, config).is_err());
    let text = String::from_utf8(bytes).unwrap();
    let duplicate = text.replace("\"schema\":1", "\"schema\":1,\"schema\":1");
    assert!(decode(duplicate.as_bytes(), &payload.key, config).is_err());
    for bytes in [b"".as_slice(), b"{", b"null"] {
        assert!(decode(bytes, &payload.key, config).is_err());
    }
}

#[test]
fn rejects_checksum_and_stale_keys() {
    let config = ProbeConfig::default();
    let payload = fixture(config);
    let mut value: serde_json::Value = serde_json::from_slice(&payload.encode().unwrap()).unwrap();
    value["payload"]["stages"][0]["ptx"] = "tampered".into();
    assert!(matches!(
        decode(&serde_json::to_vec(&value).unwrap(), &payload.key, config),
        Err(ProbeError::Artifact {
            stage: "checksum",
            ..
        })
    ));
    let changed = Key::new(config, (8, 9), 13010);
    assert!(matches!(
        decode(&payload.encode().unwrap(), &changed, config),
        Err(ProbeError::Artifact {
            stage: "key-mismatch",
            ..
        })
    ));
}

#[test]
fn rejects_missing_duplicate_unknown_and_reordered_stages() {
    let config = ProbeConfig {
        backend: ProbeBackend::CubeClLlvm,
        kernel: ProbeKernel::GlobalScan,
        ..Default::default()
    };
    let payload = fixture(config);
    let mut variants = Vec::new();
    let mut missing = payload.clone();
    missing.stages.pop();
    variants.push(missing);
    let mut extra = payload.clone();
    extra.stages.push(extra.stages[0].clone());
    variants.push(extra);
    let mut duplicate = payload.clone();
    duplicate.stages[1] = duplicate.stages[0].clone();
    variants.push(duplicate);
    let mut unknown = payload.clone();
    unknown.stages[0].entrypoint = "unknown".into();
    variants.push(unknown);
    let mut reordered = payload.clone();
    reordered.stages.swap(0, 1);
    variants.push(reordered);
    for variant in variants {
        assert!(decode(&variant.encode().unwrap(), &payload.key, config).is_err());
    }
}

#[test]
fn rejects_empty_nul_large_ptx_and_invalid_shared_memory() {
    let config = ProbeConfig::default();
    let payload = fixture(config);
    for ptx in [String::new(), "\0".into(), "x".repeat(MAX_PTX_BYTES + 1)] {
        let mut changed = payload.clone();
        changed.stages[0].ptx = ptx;
        assert!(decode(&changed.encode().unwrap(), &payload.key, config).is_err());
    }
    for bytes in [1, 128 * 1024 + 1] {
        let mut changed = payload.clone();
        changed.stages[0].shared_memory = bytes;
        assert!(decode(&changed.encode().unwrap(), &payload.key, config).is_err());
    }
    assert!(decode(&vec![b' '; MAX_ARCHIVE_BYTES + 1], &payload.key, config).is_err());
}

#[test]
fn rejects_wrong_declared_abi_and_launch_dimensions() {
    let config = ProbeConfig::default();
    let payload = fixture(config);
    let mut wrong_abi = payload.clone();
    wrong_abi.stages[0].abi = abi(ProbeBackend::CubeClCpp).into();
    assert!(decode(&wrong_abi.encode().unwrap(), &payload.key, config).is_err());
    let mut wrong_block = payload.clone();
    wrong_block.stages[0].block_threads = 256;
    assert!(decode(&wrong_block.encode().unwrap(), &payload.key, config).is_err());
}

#[test]
fn bounded_reader_rejects_oversized_files() {
    let directory = Temp::new();
    let config = ProbeConfig::default();
    let payload = fixture(config);
    let file = File::create(payload.key.path(&directory.0).unwrap()).unwrap();
    file.set_len(MAX_ARCHIVE_BYTES as u64 + 1).unwrap();
    assert!(matches!(
        read(&directory.0, &payload.key, config),
        Err(ProbeError::Artifact {
            stage: "archive-size",
            ..
        })
    ));
}

#[test]
fn cache_hit_never_calls_compiler_and_refresh_replaces() {
    let directory = Temp::new();
    let config = ProbeConfig::default();
    let payload = fixture(config);
    assert!(read(&directory.0, &payload.key, config).unwrap().is_none());
    let first = build(&directory.0, payload.key.clone(), config, false, || {
        Ok(payload.stages.clone())
    })
    .unwrap();
    assert_eq!(first.status, ArtifactStatus::Compiled);
    let hit = build(&directory.0, payload.key.clone(), config, false, || {
        panic!("hit compiled")
    })
    .unwrap();
    assert_eq!(hit.status, ArtifactStatus::Hit);
    assert_eq!(first.path, hit.path);
    let mut stages = payload.stages.clone();
    stages[0].ptx.push_str("// refresh");
    let refreshed = build(&directory.0, payload.key.clone(), config, true, || {
        Ok(stages.clone())
    })
    .unwrap();
    assert_eq!(refreshed.status, ArtifactStatus::Compiled);
    assert_eq!(
        read(&directory.0, &payload.key, config)
            .unwrap()
            .unwrap()
            .stages[0]
            .ptx,
        stages[0].ptx
    );
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
fn corruption_never_calls_compiler_without_explicit_refresh() {
    let directory = Temp::new();
    let config = ProbeConfig::default();
    let payload = fixture(config);
    fs::write(payload.key.path(&directory.0).unwrap(), b"corrupted").unwrap();
    assert!(
        build(&directory.0, payload.key.clone(), config, false, || panic!(
            "corruption compiled"
        ))
        .is_err()
    );
    build(&directory.0, payload.key.clone(), config, true, || {
        Ok(payload.stages.clone())
    })
    .unwrap();
    assert!(read(&directory.0, &payload.key, config).unwrap().is_some());
}

#[test]
fn failed_compile_or_validation_preserves_last_good_artifact() {
    let directory = Temp::new();
    let config = ProbeConfig::default();
    let payload = fixture(config);
    write(&directory.0, &payload).unwrap();
    let path = payload.key.path(&directory.0).unwrap();
    let old = fs::read(&path).unwrap();
    assert!(
        build(&directory.0, payload.key.clone(), config, true, || Err(
            error("compile", "test")
        ))
        .is_err()
    );
    assert!(
        build(&directory.0, payload.key.clone(), config, true, || Ok(
            vec![]
        ))
        .is_err()
    );
    assert_eq!(fs::read(path).unwrap(), old);
}

#[test]
fn concurrent_writers_leave_one_complete_archive() {
    let directory = Temp::new();
    let config = ProbeConfig::default();
    let payload = fixture(config);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| write(&directory.0, &payload).unwrap());
        }
    });
    assert!(read(&directory.0, &payload.key, config).unwrap().is_some());
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}

#[test]
fn failed_rename_keeps_existing_target_and_cleans_temp() {
    let directory = Temp::new();
    let payload = fixture(ProbeConfig::default());
    let path = payload.key.path(&directory.0).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(write(&directory.0, &payload).is_err());
    assert!(path.is_dir());
    assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
}
