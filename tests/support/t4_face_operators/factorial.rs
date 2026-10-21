use super::*;
use super::compile::warp_compile;
use super::stats::stats;

#[test]
#[cfg(windows)]
#[ignore = "new compiler/operator factorial experiment; frozen native Warp, NVIDIA and NVRTC; no Python"]
fn isolates_operators_targets_and_compilers() {
    let started = Instant::now();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/t4-face-operators");
    fs::create_dir_all(&out).unwrap();
    let native = root.join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native");
    let frozen = artifact("cu", CU_HASH);
    let frozen_ptx = artifact("sm89.ptx", PTX_HASH);
    let current = lineage::historical(include_str!("../../../src/physics/flex_face.rs")).unwrap();
    let base = super::super::super::super::between(&current, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
        .unwrap();
    assert_eq!(
        hash(base.as_bytes()),
        "890180caec7c0cd05560e370de89d6a5e93db24ac6c63249ce5332450330dcb3"
    );
    let mut source = format!("{frozen}\n#undef float\n#undef int\n");
    for mask in (0..9).chain([10]) {
        source += &format!("\nnamespace v{mask} {{\n{}\n}}\n", variant(base, mask));
    }
    fs::write(out.join("factorial.cu"), &source).unwrap();
    // 同一源码跨两个编译器和两个目标；另独立控制Warp选项束。
    let mut modules = Vec::new();
    let mut captures = Vec::new();
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    for (compiler, arch, warp_options) in [
        (128, 70, false),
        (128, 70, true),
        (128, 89, true),
        (129, 70, true),
        (129, 89, true),
    ] {
        let id = format!(
            "{compiler}-{arch}-{}",
            if warp_options { "warp" } else { "product" }
        );
        eprintln!("compile {id}");
        let path = out.join(format!("{id}.ptx"));
        let text = if compiler == 129 {
            warp_compile(&source, arch, &native, &path)
        } else {
            let mut options = vec![
                "--std=c++17".into(),
                format!("--gpu-architecture=compute_{arch}"),
                format!("--include-path={}", native.display()),
                "--define-macro=NDEBUG".into(),
                "--undefine-macro=WP_VERIFY_FP".into(),
                "--define-macro=WP_ENABLE_MATHDX=0".into(),
                "--device-as-default-execution-space".into(),
            ];
            if warp_options {
                options.extend(
                    [
                        "--extra-device-vectorization",
                        "--restrict",
                        "--diag-suppress=177,550",
                    ]
                    .map(str::to_owned),
                );
            }
            let ptx = compile_ptx_with_opts(
                &source,
                CompileOptions {
                    fmad: Some(true),
                    use_fast_math: Some(false),
                    options,
                    name: Some("face_operators.cu".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            let text = ptx.to_src();
            assert!(text.contains("V12.8.93"));
            fs::write(&path, &text).unwrap();
            text
        };
        captures.push(json!({"id":id,"ptx_sha256":hash(text.as_bytes()),"compiler":compiler,"arch":arch,"warp_options":warp_options}));
        modules.push(context.load_module(Ptx::from_src(text)).unwrap());
    }
    let cached = context
        .load_module(Ptx::from_src(frozen_ptx.strip_suffix('\0').unwrap()))
        .unwrap();
    let product_text = include_str!("../../../src/physics/flex_face.rs").replace("\r\n", "\n");
    let product_source =
        super::super::super::super::between(&product_text, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
            .unwrap();
    assert_eq!(
        variant(base, 10).replace("isolated_10", "flex_faces"),
        product_source
    );
    let product_ptx = compile_ptx_with_opts(
        product_source,
        CompileOptions {
            fmad: Some(true),
            use_fast_math: Some(false),
            options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
            name: Some("flex_faces.cu".into()),
            ..Default::default()
        },
    )
    .unwrap();
    fs::write(out.join("product.cu"), product_source).unwrap();
    fs::write(out.join("product.ptx"), product_ptx.to_src()).unwrap();
    let product_ptx_hash = hash(product_ptx.to_src().as_bytes());
    let product = context.load_module(product_ptx).unwrap();
    let face: Value = serde_json::from_str(FACE).unwrap();
    let base: Value = serde_json::from_str(BASE).unwrap();
    let interp = ints(&face["model"]["flex_interp"]);
    let cells = ints(&base["model"]["flex_cellnum"]);
    let mapping = ints(&face["model"]["flex_face_map"]);
    let faces = ints(&face["model"]["flex_face"]);
    assert_eq!(
        (face["cases"].as_array().unwrap().len(), faces.len()),
        (12, NF * 9)
    );
    let mut meta = vec![NF as i32, (NN * 3) as i32];
    for f in 0..NF {
        let id = mapping[2 * f] as usize;
        let [x, y, z] = cells[3 * id..3 * id + 3].try_into().unwrap();
        let index = mapping[2 * f + 1];
        meta.push(if index < 2 * y * z {
            0
        } else if index < 2 * (y * z + x * z) {
            1
        } else {
            2
        });
        meta.extend_from_slice(&faces[9 * f..9 * f + 9]);
    }
    let di = stream.clone_htod(&interp).unwrap();
    let dc = stream.clone_htod(&cells).unwrap();
    let dm = stream.clone_htod(&mapping).unwrap();
    let df = stream.clone_htod(&faces).unwrap();
    let metadata = stream.clone_htod(&meta).unwrap();
    let params = stream.clone_htod(&[0f32]).unwrap();
    let mut outputs: Vec<Vec<f32>> = vec![Vec::new(); 57];
    let mut batches = Vec::new();
    for nw in [1usize, 2, 5, 513] {
        for round in 0..4 {
            let mut nodes = vec![GUARD; 4];
            for w in 0..nw {
                let a = face["cases"][(w + round * 3) % 12]["flexnode_xpos"]
                    .as_array()
                    .unwrap();
                assert_eq!(a.len(), 3 * NN);
                nodes.extend(a.iter().map(|v| v.as_f64().unwrap() as f32));
            }
            nodes.extend([GUARD; 4]);
            let dn = stream.clone_htod(&nodes).unwrap();
            let initial = vec![GUARD; 8 + nw * WIDTH];
            let run = Instant::now();
            for slot in 0..57 {
                let mut result = stream.clone_htod(&initial).unwrap();
                let (module, mask) = if slot == 55 {
                    (&cached, 9)
                } else if slot == 56 {
                    (&product, 11)
                } else {
                    (&modules[slot / 11], slot % 11)
                };
                if mask == 9 {
                    let function = module.load_function(ENTRY).unwrap();
                    let args = [
                        array(&di, &stream, 0, &[4], 1),
                        array(&dc, &stream, 0, &[4], 3),
                        array(&dm, &stream, 0, &[NF], 2),
                        array(&df, &stream, 0, &[NF, 9], 1),
                        array(&dn, &stream, 4, &[nw, NN], 3),
                        output_array(&result, &stream, nw, false),
                        output_array(&result, &stream, nw, true),
                    ];
                    let dim = bounds(nw, NF);
                    stream.synchronize().unwrap();
                    let mut launch = stream.launch_builder(&function);
                    launch.arg(&dim);
                    for arg in &args {
                        launch.arg(arg);
                    }
                    // SAFETY: 冻结数组ABI。共享槽限定256线程；字段步长互不重叠。
                    unsafe {
                        launch.launch(config(nw * NF)).unwrap();
                    }
                } else {
                    let function = module
                        .load_function(&if mask == 11 {
                            "flex_faces".into()
                        } else {
                            format!("isolated_{mask}")
                        })
                        .unwrap();
                    // SAFETY: 全部变体保留产品ABI、元数据与独占世界输出。
                    unsafe {
                        stream
                            .launch_builder(&function)
                            .arg(&metadata)
                            .arg(&params)
                            .arg(&dn)
                            .arg(&mut result)
                            .arg(&0u32)
                            .arg(&0u32)
                            .arg(&0u32)
                            .arg(&(nw as u32))
                            .launch(config(nw))
                            .unwrap();
                    }
                }
                stream.synchronize().unwrap();
                let values = stream.clone_dtoh(&result).unwrap();
                assert_eq!(&values[..4], &[GUARD; 4]);
                assert_eq!(&values[4 + nw * WIDTH..], &[GUARD; 4]);
                assert!(
                    values[4..4 + nw * WIDTH]
                        .iter()
                        .all(|v| v.is_finite() && *v != GUARD)
                );
                outputs[slot].extend_from_slice(&values[4..4 + nw * WIDTH]);
            }
            assert_eq!(stream.clone_dtoh(&dn).unwrap(), nodes);
            batches.push(json!({"worlds":nw,"round":round,"wall_us":run.elapsed().as_micros(),"owned_device_bytes":4*(interp.len()+cells.len()+mapping.len()+faces.len()+meta.len()+1+nodes.len()+initial.len())}));
        }
    }
    assert_eq!(outputs[55].len(), 2084 * WIDTH);
    let mut comparisons = Vec::new();
    for group in 0..5 {
        for mask in 0..11 {
            let values = &outputs[11 * group + mask];
            comparisons.push(json!({"configuration":captures[group]["id"],"mask":mask,"against_same_compiler_upstream":stats(values,&outputs[11*group+9]),"against_cached_upstream":stats(values,&outputs[55]),"against_128_sm70":stats(values,&outputs[11+mask])}));
        }
    }
    let report = json!({"schema":1,"scope":"face-operators-factorial","cuda_sha256":hash(source.as_bytes()),"captures":captures,"batches":batches,"comparisons":comparisons,"source_revision":"d2d7b467f0e96c5ea0795849ce71d0078b4be917","upstream_cuda_sha256":CU_HASH,"upstream_ptx_sha256":PTX_HASH,"world_counts":[1,2,5,513],"rounds":4,"cases":12,"faces":NF,"values_per_configuration":2084*WIDTH,"elapsed_ms":started.elapsed().as_millis(),"python_executions":0,"full_acceptance":false,"product_cuda_sha256":hash(product_source.as_bytes()),"product_ptx_sha256":product_ptx_hash,"repaired_product":stats(&outputs[56],&outputs[55])});
    fs::write(
        out.join("factorial.json"),
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .unwrap();
    for c in report["comparisons"].as_array().unwrap() {
        assert_eq!(
            c["against_cached_upstream"]["outside_original_tolerance"],
            0
        );
    }
    assert_eq!(
        report["repaired_product"]["bit_differences"], 0,
        "repaired product must match frozen artifact"
    );
    println!("factorial report: {}", out.join("factorial.json").display());
}
