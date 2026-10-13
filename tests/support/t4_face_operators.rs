// 独立因子实验；不调用满额面诊断。
use super::*;
use std::fs;
#[cfg(windows)]
use std::{
    ffi::{CString, c_char, c_void},
    path::Path,
};

use super::super::super::face_lineage as lineage;
use lineage::{NEW_N, NEW_R, OLD_M, OLD_N, OLD_R};

const NEW_M: &str = "    // Preserve Warp's left-associated quaternion product.\n    float next[4];\n    next[0]=rot[3]*q[0]+q[3]*rot[0]+rot[1]*q[2]-q[1]*rot[2];\n    next[1]=rot[3]*q[1]+q[3]*rot[1]+rot[2]*q[0]-q[2]*rot[0];\n    next[2]=rot[3]*q[2]+q[3]*rot[2]+rot[0]*q[1]-q[0]*rot[1];\n    next[3]=rot[3]*q[3]-rot[0]*q[0]-rot[1]*q[1]-rot[2]*q[2];";

fn variant(base: &str, mask: usize) -> String {
    if mask == 10 {
        return variant(base, 7)
            .replace(NEW_M, lineage::FIX_M)
            .replace("isolated_7", "isolated_10");
    }
    let mut source = base.to_owned();
    for (bit, old, new) in [(1, OLD_R, NEW_R), (2, OLD_M, NEW_M), (4, OLD_N, NEW_N)] {
        assert_eq!(source.matches(old).count(), 1);
        if mask & bit != 0 {
            source = source.replace(old, new);
        }
    }
    if mask == 8 {
        source = source.replace("face_polar(quats+4*f,F);", "wp::mat_t<3,3,float> mat(F[0][0],F[1][0],F[2][0],F[0][1],F[1][1],F[2][1],F[0][2],F[1][2],F[2][2]);\n    wp::quat_t<float> q=mat33_to_quat_polar_0(mat);\n    quats[4*f]=q.x;quats[4*f+1]=q.y;quats[4*f+2]=q.z;quats[4*f+3]=q.w;");
    }
    source.replace("void flex_faces(", &format!("void isolated_{mask}("))
}

#[cfg(windows)]
fn warp_compile(source: &str, arch: i32, native: &Path, path: &Path) -> String {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryW(path: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }
    // warp.h与context.py共同定义冻结的22参数C ABI。
    type Compile = unsafe extern "C" fn(
        *const c_char,
        *const c_char,
        i32,
        *const c_char,
        *const c_char,
        i32,
        *const *const c_char,
        bool,
        i32,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        *const c_char,
        *const c_char,
        usize,
        *mut *mut c_char,
        *mut usize,
        *mut i32,
    ) -> usize;
    let dll = native.parent().unwrap().join("bin/warp.dll");
    assert_eq!(
        hash(&fs::read(&dll).unwrap()),
        "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6"
    );
    let wide: Vec<u16> = dll.to_str().unwrap().encode_utf16().chain([0]).collect();
    let src = CString::new(source).unwrap();
    let name = CString::new("face_operators.cu").unwrap();
    let include = CString::new(native.to_str().unwrap()).unwrap();
    let output = CString::new(path.to_str().unwrap()).unwrap();
    // SAFETY: 仅加载哈希冻结的本地库。字符串存活至同步编译返回。
    // 空链接数组与零长度配对。关闭PCH，不启动Python或Warp运行时。
    unsafe {
        let module = LoadLibraryW(wide.as_ptr());
        assert!(!module.is_null());
        let pointer = GetProcAddress(module, c"wp_cuda_compile_program".as_ptr());
        assert!(!pointer.is_null());
        let compile: Compile = std::mem::transmute(pointer);
        let code = compile(
            src.as_ptr(),
            name.as_ptr(),
            arch,
            c"".as_ptr(),
            include.as_ptr(),
            0,
            std::ptr::null(),
            false,
            3,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            output.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_ne!(FreeLibrary(module), 0);
        assert_eq!(code, 0);
    }
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains("V12.9.86"));
    text.trim_end_matches('\0').to_owned()
}

#[cfg(windows)]
fn stats(actual: &[f32], expected: &[f32]) -> Value {
    assert_eq!(actual.len(), expected.len());
    let mut changed = 0usize;
    let mut max = 0.0f64;
    let mut over = 0usize;
    for (&a, &e) in actual.iter().zip(expected) {
        assert!(a.is_finite() && e.is_finite());
        changed += usize::from(a.to_bits() != e.to_bits());
        let error = (f64::from(a) - f64::from(e)).abs();
        max = max.max(error);
        over += usize::from(error > 2e-5 + 2e-5 * f64::from(e).abs());
    }
    json!({"comparisons":actual.len(),"bit_differences":changed,"max_abs_error":max,"outside_original_tolerance":over})
}

#[test]
#[ignore = "new intermediate trace; frozen Warp headers, NVIDIA and NVRTC; no Python"]
fn traces_intermediate_operators() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/t4-face-operators");
    fs::create_dir_all(&out).unwrap();
    let frozen = artifact("cu", CU_HASH).replace("\r\n", "\n");
    let current = lineage::historical(include_str!("../../src/physics/flex_face.rs")).unwrap();
    let base = super::super::super::between(&current, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
        .unwrap();
    let mut candidate=variant(base,7).replace("void isolated_7(","void trace_candidate(")
        .replace("const float F[3][3])","const float F[3][3],float* trace)")
        .replace("face_polar(quats+4*f,F);","face_polar(quats+4*f,F,result+4+worlds*31*nf+(w*nf+f)*1350);")
        .replace("    float crosses[3][3];","    for(int c=0;c<3;++c)for(int k=0;k<3;++k)trace[iter*27+3*c+k]=R[c][k];\n    float crosses[3][3];")
        .replace("    float w=sqrtf(face_dot(omega,omega));if(w<1.0e-6f) break;","    float w=sqrtf(face_dot(omega,omega));\n    for(int k=0;k<3;++k)trace[iter*27+9+k]=omega[k];trace[iter*27+12]=denom;trace[iter*27+13]=w;\n    if(w<1.0e-6f) break;")
        .replace("    // Preserve Warp's left-associated quaternion product.","    for(int k=0;k<4;++k)trace[iter*27+14+k]=rot[k];\n    // Preserve Warp's left-associated quaternion product.")
        .replace("    // Warp kEps is zero;", "    for(int k=0;k<4;++k)trace[iter*27+18+k]=next[k];trace[iter*27+22]=n;\n    // Warp kEps is zero;");
    candidate = candidate.replace(
        NEW_N,
        &format!("{NEW_N}\n    for(int k=0;k<4;++k)trace[iter*27+23+k]=q[k];"),
    );
    let reference = super::super::super::between(
        &frozen,
        "static CUDA_CALLABLE wp::quat_t<wp::float32> mat33_to_quat_polar_0(",
        "\n}\n",
    )
    .unwrap();
    let mut reference=format!("static CUDA_CALLABLE wp::quat_t<wp::float32> trace_polar({reference}\n}}\n")
        .replacen("var_F)","var_F, float* trace)",1)
        .replace("var_8 = wp::quat_to_matrix(var_4);","var_8 = wp::quat_to_matrix(var_4);for(int c=0;c<3;++c)for(int k=0;k<3;++k)trace[var_7*27+3*c+k]=var_8.data[k][c];")
        .replace("var_37 = wp::length(var_36);","var_37 = wp::length(var_36);for(int k=0;k<3;++k)trace[var_7*27+9+k]=var_36[k];trace[var_7*27+12]=var_35;trace[var_7*27+13]=var_37;")
        .replace("var_57 = wp::mul(var_56, var_4);","for(int k=0;k<4;++k)trace[var_7*27+14+k]=wp::extract(var_56,k);var_57 = wp::mul(var_56, var_4);for(int k=0;k<4;++k)trace[var_7*27+18+k]=wp::extract(var_57,k);trace[var_7*27+22]=wp::length(var_57);")
        .replace("var_58 = wp::normalize(var_57);","var_58 = wp::normalize(var_57);for(int k=0;k<4;++k)trace[var_7*27+23+k]=wp::extract(var_58,k);");
    reference += &variant(base, 8)
        .replace("void isolated_8(", "void trace_reference(")
        .replace(
            "mat33_to_quat_polar_0(mat)",
            "trace_polar(mat,result+4+worlds*31*nf+(w*nf+f)*1350)",
        );
    let source = format!(
        "{frozen}\n#undef float\n#undef int\nnamespace candidate {{\n{candidate}\n}}\nnamespace reference {{\n{reference}\n}}"
    );
    fs::write(out.join("trace.cu"), &source).unwrap();
    let ptx = compile_ptx_with_opts(
        &source,
        CompileOptions {
            fmad: Some(true),
            use_fast_math: Some(false),
            options: vec![
                "--std=c++17".into(),
                "--gpu-architecture=compute_70".into(),
                format!(
                    "--include-path={}",
                    root.join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native")
                        .display()
                ),
                "--device-as-default-execution-space".into(),
                "--define-macro=NDEBUG".into(),
                "--define-macro=WP_ENABLE_MATHDX=0".into(),
            ],
            name: Some("trace.cu".into()),
            ..Default::default()
        },
    )
    .unwrap();
    fs::write(out.join("trace.ptx"), ptx.to_src()).unwrap();
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    let module = context.load_module(ptx).unwrap();
    let face: Value = serde_json::from_str(FACE).unwrap();
    let b: Value = serde_json::from_str(BASE).unwrap();
    let cells = ints(&b["model"]["flex_cellnum"]);
    let mapping = ints(&face["model"]["flex_face_map"]);
    let faces = ints(&face["model"]["flex_face"]);
    let mut meta = vec![NF as i32, 60];
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
    let dm = stream.clone_htod(&meta).unwrap();
    let params = stream.clone_htod(&[0f32]).unwrap();
    let mut first = Vec::new();
    let mut stages = [0usize; 27];
    let mut total = 0;
    for nw in [1usize, 2, 5, 513] {
        for round in 0..4 {
            let mut nodes = vec![GUARD; 4];
            for w in 0..nw {
                nodes.extend(
                    face["cases"][(w + round * 3) % 12]["flexnode_xpos"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap() as f32),
                );
            }
            nodes.extend([GUARD; 4]);
            let dn = stream.clone_htod(&nodes).unwrap();
            let mut outputs = Vec::new();
            for entry in ["trace_candidate", "trace_reference"] {
                let mut result = stream
                    .clone_htod(&vec![GUARD; 8 + nw * (WIDTH + NF * 1350)])
                    .unwrap();
                let function = module.load_function(entry).unwrap();
                // SAFETY: 原输出后另分配每面50轮、每轮27项。两端保留哨兵。
                unsafe {
                    stream
                        .launch_builder(&function)
                        .arg(&dm)
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
                let values = stream.clone_dtoh(&result).unwrap();
                assert_eq!(&values[..4], &[GUARD; 4]);
                assert_eq!(&values[values.len() - 4..], &[GUARD; 4]);
                outputs.push(values);
            }
            for face_index in 0..nw * NF {
                total += 1;
                let start = 4 + nw * WIDTH + face_index * 1350;
                for i in 0..1350 {
                    let a = outputs[0][start + i];
                    let e = outputs[1][start + i];
                    if a.to_bits() != e.to_bits() {
                        stages[i % 27] += 1;
                        if first.len() < 16 {
                            first.push(json!({"worlds":nw,"round":round,"face":face_index,"iteration":i/27,"slot":i%27,"candidate":a,"upstream":e}));
                        }
                        break;
                    }
                }
            }
        }
    }
    fs::write(out.join("trace.json"),serde_json::to_string_pretty(&json!({"faces":total,"first_difference_slots":stages,"samples":first,"cuda_sha256":hash(source.as_bytes()),"scope":"instrumented-first-difference-not-product-acceptance"})).unwrap()+"\n").unwrap();
    println!("first difference slots: {stages:?}");
}

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
    let current = lineage::historical(include_str!("../../src/physics/flex_face.rs")).unwrap();
    let base = super::super::super::between(&current, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
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
    let product_text = include_str!("../../src/physics/flex_face.rs").replace("\r\n", "\n");
    let product_source =
        super::super::super::between(&product_text, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
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
