use super::*;

#[test]
#[ignore = "new intermediate trace; frozen Warp headers, NVIDIA and NVRTC; no Python"]
fn traces_intermediate_operators() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("target/t4-face-operators");
    fs::create_dir_all(&out).unwrap();
    let frozen = artifact("cu", CU_HASH).replace("\r\n", "\n");
    let current = lineage::historical(include_str!("../../../src/physics/flex_face.rs")).unwrap();
    let base = super::super::super::super::between(&current, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;")
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
    let reference = super::super::super::super::between(
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
