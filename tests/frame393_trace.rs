#![cfg(feature = "cuda-probe")]
//! G01隔离诊断，不承担容差准入。
#[path = "support/geom_tendon_input.rs"]
mod input;
#[path = "../src/physics/tendon_wrap.rs"]
mod wrap_source;

use cudarc::{
    driver::{CudaContext, LaunchConfig, PushKernelArg},
    nvrtc::{CompileOptions, compile_ptx_with_opts},
};
use input::{fixture, floats, motion_fixture, plan, write};
use mjwarp_rs::runtime::TransferSession;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn replace_once(source: &mut String, old: &str, new: &str) {
    assert_eq!(source.matches(old).count(), 1, "trace anchor: {old}");
    *source = source.replace(old, new);
}

fn instrumented_source() -> String {
    let mut source = wrap_source::TENDON_WRAP_CUDA.to_owned();
    replace_once(
        &mut source,
        "inside(V2 e0,V2 e1,float r)",
        "inside(V2 e0,V2 e1,float r,float* trace)",
    );
    replace_once(
        &mut source,
        "int type,SV side)",
        "int type,SV side,float* trace)",
    );
    replace_once(&mut source, "inside(e0,e1,r)", "inside(e0,e1,r,trace)");
    for (anchor, extra) in [
        (
            "SV p0=mt(mat,sub(x0,pos)),p1=mt(mat,sub(x1,pos));",
            "trace[0]=p0.x;trace[1]=p0.y;trace[2]=p0.z;trace[3]=p1.x;trace[4]=p1.y;trace[5]=p1.z;",
        ),
        (
            "V2 e0={dot(p0,ax0),dot(p0,ax1)},e1={dot(p1,ax0),dot(p1,ax1)};",
            "trace[6]=e0.x;trace[7]=e0.y;trace[8]=e1.x;trace[9]=e1.y;",
        ),
        (
            "dd=dx*dx+dy*dy;",
            "trace[10]=l0;trace[11]=l1;trace[12]=dx;trace[13]=dy;trace[14]=dd;",
        ),
        (
            "float A=sd(r,l0),B=sd(r,l1),aa=A*A,bb=B*B;",
            "trace[15]=A;trace[16]=B;trace[17]=aa;trace[18]=bb;",
        ),
        ("float c=sd(l0*l0+l1*l1-dd,2.0f*l0*l1);", "trace[19]=c;"),
        ("float G=ac(c),z=1.0f-1e-7f;", "trace[20]=G;trace[21]=z;"),
        ("float f=as(A*z)+as(B*z)-2.0f*as(z)+G;", "trace[22]=f;"),
        (
            "-2.0f/fmaxf(1e-15f,sqrtf(1.0f-zz));",
            "trace[23+6*iter]=z;trace[24+6*iter]=f;trace[25+6*iter]=zz;trace[26+6*iter]=df;",
        ),
        ("float z1=z-sd(f,df);", "trace[27+6*iter]=z1;"),
        (
            "z=z1;f=as(A*z)+as(B*z)-2.0f*as(z)+G;",
            "trace[28+6*iter]=f;",
        ),
        (
            "p={r*(cosf(ang)*vx-sinf(ang)*vy),r*(sinf(ang)*vx+cosf(ang)*vy)};",
            "trace[143]=float(iter);trace[144]=z;trace[145]=f;trace[146]=ang;trace[147]=vx;trace[148]=vy;trace[149]=p.x;trace[150]=p.y;",
        ),
        (
            "b.z=p0.z+(p1.z-p0.z)*sd(l0+w.len,l0+w.len+l1);",
            "trace[151]=l0;trace[152]=l1;trace[153]=a.z;trace[154]=b.z;",
        ),
    ] {
        replace_once(&mut source, anchor, &format!("{anchor}\n{extra}"));
    }
    source
}

const ENTRY: &str = r#"
extern "C" __global__ void frame393(const float* p,float* out) {
  if(blockIdx.x || threadIdx.x) return;
  raw::Wrap3 r=raw::wrap(raw::sl(p),raw::sl(p+3),raw::sl(p+6),p+9,p[21],5,raw::sl(p+18));
  out[0]=r.len;out[1]=r.a.x;out[2]=r.a.y;out[3]=r.a.z;
  out[4]=r.b.x;out[5]=r.b.y;out[6]=r.b.z;
  traced::Wrap3 t=traced::wrap(traced::sl(p),traced::sl(p+3),traced::sl(p+6),p+9,p[21],5,traced::sl(p+18),out+14);
  out[7]=t.len;out[8]=t.a.x;out[9]=t.a.y;out[10]=t.a.z;
  out[11]=t.b.x;out[12]=t.b.y;out[13]=t.b.z;
  out[169]=t.a.x;out[170]=t.a.y;out[171]=t.a.z;
  out[172]=t.b.x;out[173]=t.b.y;out[174]=t.b.z;
  // Independent f32 probes distinguish the two FMA orders from separate rounding.
  for(int i=0;i<2;++i) {
    float x=out[20+2*i],y=out[21+2*i];
    float xx=__fmul_rn(x,x),yy=__fmul_rn(y,y);
    float xy=fmaf(x,x,yy),yx=fmaf(y,y,xx),separate=__fadd_rn(xx,yy);
    int j=175+8*i;
    out[j]=xx;out[j+1]=yy;out[j+2]=xy;out[j+3]=yx;out[j+4]=separate;
    out[j+5]=sqrtf(xy);out[j+6]=sqrtf(yx);out[j+7]=sqrtf(separate);
  }
  replay::Wrap3 v=replay::wrap(replay::sl(p),replay::sl(p+3),replay::sl(p+6),p+9,p[21],5,replay::sl(p+18));
  out[191]=v.len;out[192]=v.a.x;out[193]=v.a.y;out[194]=v.a.z;
  out[195]=v.b.x;out[196]=v.b.y;out[197]=v.b.z;
}
"#;

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|v| v.to_bits()).collect()
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn trace_label(index: usize) -> String {
    match index {
        0..=5 => format!("local_endpoint[{index}]"),
        6..=9 => format!("projected_endpoint[{}]", index - 6),
        10 => "inside.len0".into(),
        11 => "inside.len1".into(),
        12..=14 => ["inside.dx", "inside.dy", "inside.dd"][index - 12].into(),
        15..=22 => [
            "inside.A",
            "inside.B",
            "inside.A_squared",
            "inside.B_squared",
            "inside.cosG",
            "inside.G",
            "inside.initial_z",
            "inside.initial_f",
        ][index - 15]
            .into(),
        23..=142 => format!(
            "newton[{}].{}",
            (index - 23) / 6,
            ["z", "f", "z_squared", "df", "z_next", "f_next"][(index - 23) % 6]
        ),
        143..=154 => [
            "iterations",
            "final_z",
            "final_f",
            "angle",
            "direction_x",
            "direction_y",
            "contact_x",
            "contact_y",
            "cylinder.L0",
            "cylinder.L1",
            "cylinder.z0",
            "cylinder.z1",
        ][index - 143]
            .into(),
        _ => format!("world_contact[{}]", index - 155),
    }
}

fn trace_reference() -> Value {
    let bytes = include_bytes!("../fixtures/mjwarp-geom-tendon/trace393.json");
    let value: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(
        sha(bytes),
        "c62c8ed0c4ba7b868c9632baca7cc828d3759b11e6d05a06ac695f60e43b29f4"
    );
    assert_eq!(
        value["author_sha256"],
        sha(include_bytes!("../fixtures/mjwarp-geom-tendon/trace393.py"))
    );
    assert_eq!(
        value["reference_sha256"],
        sha(include_bytes!(
            "../fixtures/mjwarp-geom-tendon/reference.json"
        ))
    );
    assert_eq!(
        value["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    for key in ["wrap_xpos", "ten_J", "ten_length"] {
        assert_eq!(value["full_instrumentation_bit_mismatches"][key], 0);
    }
    let inputs = floats(&value, "inputs");
    assert_eq!(
        value["inputs_sha256"],
        sha(&inputs
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>())
    );
    assert_eq!(floats(&value, "trace").len(), 161);
    value
}

#[test]
fn trace_anchors_match_the_current_cuda_source() {
    assert!(instrumented_source().contains("trace[154]"));
    trace_reference();
}

#[test]
#[ignore = "需要NVIDIA与NVRTC；仅作首差诊断"]
fn traces_frame393_before_newton_without_changing_production() {
    let upstream = trace_reference();
    let inputs = floats(&upstream, "inputs");
    assert_eq!(inputs.len(), 22);
    let directory = std::path::PathBuf::from(
        std::env::var_os("MJWARP_FRAME393_DIR").unwrap_or_else(|| "target/frame393-trace".into()),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let mut replay = wrap_source::TENDON_WRAP_CUDA.to_owned();
    // The product now pins this order. Keep the independent inline replay form.
    // Do not change the production source or any convergence threshold.
    replace_once(
        &mut replay,
        "l0=inside_norm(e0),l1=inside_norm(e1)",
        "l0=sqrtf(__fmaf_rn(e0.y,e0.y,__fmul_rn(e0.x,e0.x))),l1=sqrtf(__fmaf_rn(e1.y,e1.y,__fmul_rn(e1.x,e1.x)))",
    );
    let source = format!(
        "namespace raw {{ {} }}\nnamespace traced {{ {} }}\nnamespace replay {{ {replay} }}\n{ENTRY}",
        wrap_source::TENDON_WRAP_CUDA,
        instrumented_source()
    );
    // Match the production compiler options; keep diagnostics outside the public API.
    let ptx = compile_ptx_with_opts(
        &source,
        CompileOptions {
            fmad: Some(true),
            use_fast_math: Some(false),
            options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
            ..Default::default()
        },
    )
    .unwrap();
    std::fs::write(directory.join("local.cu"), &source).unwrap();
    std::fs::write(directory.join("local.ptx"), ptx.to_src()).unwrap();
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    let function = context
        .load_module(ptx)
        .unwrap()
        .load_function("frame393")
        .unwrap();
    let device_inputs = stream.clone_htod(&inputs).unwrap();
    let mut device_output = stream.alloc_zeros::<f32>(198).unwrap();
    // SAFETY: One thread reads 22 f32 inputs and writes at most 198 f32 outputs.
    // The separate allocations share a context and stay alive through readback.
    unsafe {
        stream
            .launch_builder(&function)
            .arg(&device_inputs)
            .arg(&mut device_output)
            .launch(LaunchConfig {
                grid_dim: (1, 1, 1),
                block_dim: (1, 1, 1),
                shared_mem_bytes: 0,
            })
            .unwrap();
    }
    let output = stream.clone_dtoh(&device_output).unwrap();
    assert!(output.iter().all(|v| v.is_finite()));
    assert_eq!(
        bits(&output[..7]),
        bits(&output[7..14]),
        "instrumentation perturbed the endpoint"
    );

    // Verify the isolated local endpoint against the actual resident pipeline.
    let fixture = fixture();
    let motion = motion_fixture();
    let case = motion["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["source_frame"] == 393)
        .unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = plan(&session, &fixture);
    let mut data = plan.create_data(3).unwrap();
    for (w, c) in [&fixture["cases"][0], &fixture["cases"][1], case]
        .into_iter()
        .enumerate()
    {
        write(&mut data, w, c);
    }
    plan.update(&mut data).unwrap();
    let snapshot = data.readback().unwrap();
    let attached = snapshot.attached().world(2).unwrap();
    let tendon = snapshot.spatial_tendon().world(2).unwrap();
    let actual_inputs: Vec<_> = [
        &attached.site_xpos[39..42],
        &attached.site_xpos[42..45],
        &attached.geom_xpos[9..12],
        &attached.geom_xmat[27..36],
        &attached.site_xpos[54..57],
        &floats(&fixture["cases"][2], "geom_size")[9..10],
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    assert_eq!(bits(&actual_inputs), bits(&inputs), "wrap input differs");
    assert_eq!(
        bits(&output[1..7]),
        bits(&tendon.wrap_xpos[45..51]),
        "isolated local output differs from full pipeline"
    );
    let expected = floats(&upstream, "trace");
    let actual = &output[14..175];
    let differences: Vec<_> = actual.iter().zip(&expected).enumerate()
        .filter(|(_, (a,b))| a.to_bits() != b.to_bits())
        .map(|(index, (a,b))| json!({"index":index,"stage":trace_label(index),"actual":a,"upstream":b,
            "actual_bits":format!("{:08x}",a.to_bits()),"upstream_bits":format!("{:08x}",b.to_bits())})).collect();
    let report = json!({"frame":393,"scope":"G01 diagnostic only", "inputs":inputs,
        "trace":actual,"upstream_trace":expected,"differences":differences,
        "first_difference":differences.first(),
        "norm_candidate_order":["xx", "yy", "fma_xx_plus_yy", "fma_yy_plus_xx", "separate_sum",
            "sqrt_fma_xx_plus_yy", "sqrt_fma_yy_plus_xx", "sqrt_separate_sum"],
        "norm_candidates":&output[175..191],"local_endpoint":&output[..7],
        "replay_endpoint":&output[191..198],
        "replay_matches_upstream_bits":bits(&output[191..198]) == bits(&floats(&upstream, "output")),
        "source_sha256":sha(source.as_bytes()),
        "upstream_endpoint":upstream["output"],"ten_J_actual":tendon.ten_jacobian,
        "instrumentation_preserves_output_bits":true,"matches_resident_endpoint_bits":true});
    std::fs::write(
        directory.join("local.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("FRAME393_TRACE {report}");
    assert_eq!(
        bits(&output[191..198]),
        bits(&floats(&upstream, "output")),
        "pinning the two initial norm orders did not explain the endpoint difference"
    );
}
