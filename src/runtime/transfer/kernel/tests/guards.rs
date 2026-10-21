//! 同步内核六缓冲与编译错误测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn checks_hessian_six_buffers_and_fixed_abi_without_quarantine() {
    let session = TransferSession::new(0).unwrap();
    let other = TransferSession::new(0).unwrap();
    let source = r#"
extern "C" __global__ void hessian_probe(const int* m,const float* p,const float* x,
    const float* e,const int* v,float* o,unsigned n) {
  unsigned i=blockIdx.x*blockDim.x+threadIdx.x;
  if(i<n) o[i]=float(m[0])+p[0]+x[i]+e[i]+float(v[i]);
}
"#;
    let kernel = SynchronousKernel::compile(&session, source, "hessian_probe").unwrap();
    let m = session.upload(&[3i32]).unwrap();
    let p = session.upload(&[2.0f32]).unwrap();
    let x = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
    let e = session.upload(&[4.0f32; 3]).unwrap();
    let v = session.upload(&[1i32; 3]).unwrap();
    let mut output = session.upload(&[0.0f32; 3]).unwrap();
    let fm = other.upload(&[0i32]).unwrap();
    let fp = other.upload(&[0.0f32]).unwrap();
    let fx = other.upload(&[0.0f32; 3]).unwrap();
    let fe = other.upload(&[0.0f32; 3]).unwrap();
    let fv = other.upload(&[0i32; 3]).unwrap();
    let mut fo = other.upload(&[0.0f32; 3]).unwrap();
    let empty = session.upload::<f32>(&[]).unwrap();
    // SAFETY: The trusted seven-argument shader reads three entries.
    // Six rejection cases return before launch; one accepted case uses
    // matching sessions and bounded, distinct input/output allocations.
    unsafe {
        for k in 0..6 {
            assert_eq!(
                kernel.launch_flex_hessian(
                    (if k == 0 { &fm } else { &m }, if k == 1 { &fp } else { &p }),
                    if k == 2 { &fx } else { &x },
                    if k == 3 { &fe } else { &e },
                    if k == 4 { &fv } else { &v },
                    if k == 5 { &mut fo } else { &mut output },
                    3
                ),
                Err(TransferError::SessionMismatch)
            );
        }
        for worlds in [0, u32::MAX] {
            assert!(matches!(
                kernel.launch_flex_hessian((&m, &p), &x, &e, &v, &mut output, worlds),
                Err(TransferError::Input(_))
            ));
        }
        assert!(matches!(
            kernel.launch_flex_hessian((&m, &empty), &x, &e, &v, &mut output, 3),
            Err(TransferError::Input(_))
        ));
        kernel
            .launch_flex_hessian((&m, &p), &x, &e, &v, &mut output, 3)
            .unwrap();
    }
    let mut values = [0.0; 3];
    output.read_range_into(0, &mut values).unwrap();
    assert_eq!(values, [11.0, 12.0, 13.0]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn compiler_errors_do_not_quarantine_transfers() {
    let session = TransferSession::new(0).unwrap();
    assert!(matches!(
        SynchronousKernel::compile(&session, "invalid cuda source", "bad"),
        Err(TransferError::Backend(ProbeError::Compilation { .. }))
    ));
    assert!(session.upload(&[1.0]).is_ok());
    let kernel = SynchronousKernel::compile(&session, SOURCE, "adapter_probe").unwrap();
    // Simulate completion uncertainty without an actual GPU failure. This
    // exercises module retention; the old transfer tests cover buffer owners.
    session
        .inner
        .status
        .uncertain
        .store(true, std::sync::atomic::Ordering::Release);
    drop(kernel);
    session
        .inner
        .status
        .uncertain
        .store(false, std::sync::atomic::Ordering::Release);
}
