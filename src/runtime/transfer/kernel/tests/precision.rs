//! 同步内核精度提升与降级拒绝测试。

use super::tests_support::*;
use super::*;

#[test]
fn rejects_generated_physics_precision_promotions_and_demotions() {
    for code in [
        "add.rn.f64 %fd1, %fd2, %fd3;",
        "cvt.f32.f64 %f1, %fd2;",
        ".reg .f64 %fd<2>;",
        "mul.rn.f16 %h1, %h2, %h3;",
        "add.f16x2 %r1, %r2, %r3;",
        "cvt.rn.bf16.f32 %h1, %f1;",
        "add.bf16x2 %r1, %r2, %r3;",
        "cvt.rn.tf32.f32 %r1, %f1;",
    ] {
        assert!(matches!(
            check_physics_ptx(code, "probe"),
            Err(ProbeError::Compilation {
                stage: "physics-precision-ptx",
                ..
            })
        ));
    }
    check_physics_ptx(
        "// f64 is forbidden\n.reg .b64 %rd<2>;\nadd.rn.f32 %f16, %f64, %f3;",
        "probe",
    )
    .unwrap();
    let reduction = ".reg .f64 %fd<3>;\ncvt.rn.f64.s64 %fd1, %rd1;\nmul.rn.f64 %fd2, %fd1, 0d3BF921FB54442D19;\ncvt.rn.f32.f64 %f1, %fd2;";
    check_physics_ptx(reduction, "probe").unwrap();
    check_physics_ptx(&reduction.replace("mul.rn.f64", "mul.f64"), "probe").unwrap();
    for changed in [
        reduction.replace("f64.s64", "f64.f32"),
        reduction.replace("0d3BF921FB54442D19", "0d3FF0000000000000"),
        reduction.replace("%fd1, 0d", "%fd3, 0d"),
        reduction.replace("mul.rn.f64", "mul.rz.f64"),
        format!("{reduction}\nadd.rn.f64 %fd1, %fd1, %fd2;"),
        format!("{reduction}\nmul.f64 %fd1, %fd1, %fd2;"),
    ] {
        assert!(check_physics_ptx(&changed, "probe").is_err());
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn physical_kernel_fuses_f32_multiply_add_like_warp() {
    let session = TransferSession::new(0).unwrap();
    let source = r#"
extern "C" __global__ void fma_probe(const int* m,const float* p,const float* s,
    float* o,unsigned a,unsigned b,unsigned c,unsigned n) {
  if(threadIdx.x==0 && blockIdx.x==0) o[0]=p[0]*s[0]+p[1];
}
"#;
    let kernel = SynchronousKernel::compile(&session, source, "fma_probe").unwrap();
    let a = 1.0 + f32::EPSILON;
    let b = 1.0 - f32::EPSILON;
    let meta = session.upload(&[0_i32]).unwrap();
    let parameters = session.upload(&[a, -1.0]).unwrap();
    let state = session.upload(&[b]).unwrap();
    let mut output = session.upload(&[0.0_f32]).unwrap();
    // SAFETY: 固定内核只读两参数和一状态。
    // 同会话输出独占一个f32元素。
    unsafe {
        kernel
            .launch(&meta, &parameters, &state, &mut output, [0, 0, 0, 1])
            .unwrap();
    }
    let mut values = [0.0_f32];
    output.read_range_into(0, &mut values).unwrap();
    assert_eq!(values[0], a.mul_add(b, -1.0));
    assert_ne!(values[0], a * b - 1.0);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn preserves_f64_buffer_width_in_fixed_kernel_abi() {
    let session = TransferSession::new(0).unwrap();
    let source = SOURCE.replace("float", "double");
    assert!(matches!(
        SynchronousKernel::compile(&session, &source, "adapter_probe"),
        Err(TransferError::Backend(ProbeError::Compilation {
            stage: "physics-precision-ptx",
            ..
        }))
    ));
    // Only this ABI test opts out; production compile always checks PTX.
    let kernel =
        SynchronousKernel::compile_checked(&session, &source, "adapter_probe", false).unwrap();
    let meta = session.upload(&[0i32]).unwrap();
    let parameters = session.upload(&[2.0f64.powi(-40)]).unwrap();
    let state = session.upload(&[1.0f64, -1.0]).unwrap();
    let mut output = session.upload(&[0.0f64; 2]).unwrap();
    // SAFETY: This trusted shader uses double pointers and two elements in
    // same-session f64 state/output buffers. Metadata is one i32 element.
    unsafe {
        kernel
            .launch(&meta, &parameters, &state, &mut output, [0, 0, 0, 2])
            .unwrap();
    }
    let mut values = [0.0f64; 2];
    output.read_range_into(0, &mut values).unwrap();
    assert_eq!(values, [1.0 + 2.0f64.powi(-40), -1.0 + 2.0f64.powi(-40)]);
    assert_ne!(values[0], 1.0);
}
