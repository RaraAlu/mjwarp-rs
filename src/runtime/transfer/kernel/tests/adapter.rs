//! 同步内核适配器守卫测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn rejects_foreign_sessions_empty_buffers_and_invalid_work_without_quarantine() {
    let session = TransferSession::new(0).unwrap();
    let other = TransferSession::new(0).unwrap();
    let kernel = SynchronousKernel::compile(&session, SOURCE, "adapter_probe").unwrap();
    let meta = session.upload(&[3i32]).unwrap();
    let foreign = other.upload(&[3i32]).unwrap();
    let parameters = session.upload(&[2.0f32]).unwrap();
    let state = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
    let empty = session.upload::<f32>(&[]).unwrap();
    let mut output = session.upload(&[0.0f32; 3]).unwrap();
    // SAFETY: This trusted kernel uses three state/output elements; rejected
    // paths return before launch and do not dereference any buffer pointer.
    unsafe {
        assert_eq!(
            kernel.launch(&foreign, &parameters, &state, &mut output, [1, 2, 3, 3]),
            Err(TransferError::SessionMismatch)
        );
        assert!(matches!(
            kernel.launch(&meta, &parameters, &empty, &mut output, [1, 2, 3, 3]),
            Err(TransferError::Input(_))
        ));
        for count in [0, u32::MAX] {
            assert!(matches!(
                kernel.launch(&meta, &parameters, &state, &mut output, [1, 2, 3, count]),
                Err(TransferError::Input(_))
            ));
        }
        kernel
            .launch(&meta, &parameters, &state, &mut output, [1, 2, 3, 3])
            .unwrap();
    }
    drop(kernel);
    let mut values = [0.0; 3];
    output.read_range_into(0, &mut values).unwrap();
    assert_eq!(values, [12.0, 13.0, 14.0]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn preserves_three_independent_buffer_widths_in_mixed_kernel_abi() {
    let session = TransferSession::new(0).unwrap();
    let source = SOURCE
        .replace("const float* p", "const double* p")
        .replace("float* o", "int* o");
    let kernel =
        SynchronousKernel::compile_checked(&session, &source, "adapter_probe", false).unwrap();
    let meta = session.upload(&[3i32]).unwrap();
    let parameters = session.upload(&[2.0f64]).unwrap();
    let state = session.upload(&[1.0f32, 2.0, 3.0]).unwrap();
    let mut output = session.upload(&[-1i32; 3]).unwrap();
    // SAFETY: The trusted shader reads one i32, one f64 and three f32
    // elements, then writes three i32 elements in the same session.
    unsafe {
        kernel
            .launch(&meta, &parameters, &state, &mut output, [1, 2, 3, 3])
            .unwrap();
    }
    let mut values = [0i32; 3];
    output.read_range_into(0, &mut values).unwrap();
    assert_eq!(values, [12, 13, 14]);
}
