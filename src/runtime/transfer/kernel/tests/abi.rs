//! 同步内核ABI与会话边界测试。

use super::*;

#[test]
#[ignore = "new mixed128 prefix-stride ABI; no old ABI or physics replay"]
fn mixed_solve_128_abi_checks_prefix_stride_sessions_and_grouped_capacity() {
    let session = TransferSession::new(0).unwrap();
    let other = TransferSession::new(0).unwrap();
    let source = r#"extern "C" __global__ void mixed_solve_abi(const int* meta,const float* rhs,
 const float* factor,float* output,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 __shared__ float slots[128];unsigned tid=threadIdx.x;slots[tid]=float(tid)+0.25f;__syncthreads();
 for(unsigned i=tid;i<n;i+=128)output[4+blockIdx.x*n+i]=float(blockIdx.x)*1000.0f+float(i)+slots[(tid+1)%128];
}"#;
    let kernel = SynchronousKernel::compile(&session, source, "mixed_solve_abi").unwrap();
    let metadata = session.upload(&vec![0i32; 263]).unwrap();
    let rhs = session.upload(&vec![0.0f32; 782]).unwrap();
    let factor = session.upload(&vec![0.0f32; 908]).unwrap();
    let mut output = session.upload(&vec![-131072.0f32; 782]).unwrap();
    let foreign_meta = other.upload(&vec![0i32; 263]).unwrap();
    let foreign_rhs = other.upload(&vec![0.0f32; 782]).unwrap();
    let foreign_factor = other.upload(&vec![0.0f32; 908]).unwrap();
    let mut foreign_output = other.upload(&vec![-131072.0f32; 782]).unwrap();
    let dims = [129, 300, 2, 3];
    // SAFETY: Rejection paths never launch. The probe ignores metadata and
    // inputs; its checked groups and n bound every disjoint output write.
    unsafe {
        assert!(matches!(
            kernel.launch_mixed_solve(&foreign_meta, &rhs, &factor, &mut output, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            kernel.launch_mixed_solve(&metadata, &foreign_rhs, &factor, &mut output, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            kernel.launch_mixed_solve(&metadata, &rhs, &foreign_factor, &mut output, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            kernel.launch_mixed_solve(&metadata, &rhs, &factor, &mut foreign_output, dims),
            Err(TransferError::SessionMismatch)
        ));
        for bad in [
            [0, 300, 2, 3],
            [129, 257, 2, 3],
            [129, 300, 0, 3],
            [129, 300, 2, 0],
            [u32::MAX, 300, 2, 3],
            [1, 2, 65536, 65536],
        ] {
            assert!(
                kernel
                    .launch_mixed_solve(&metadata, &rhs, &factor, &mut output, bad)
                    .is_err()
            );
        }
        let short_meta = session.upload(&vec![0i32; 262]).unwrap();
        let short_rhs = session.upload(&vec![0.0f32; 781]).unwrap();
        let short_factor = session.upload(&vec![0.0f32; 907]).unwrap();
        let mut short_output = session.upload(&vec![0.0f32; 781]).unwrap();
        assert!(
            kernel
                .launch_mixed_solve(&short_meta, &rhs, &factor, &mut output, dims)
                .is_err()
        );
        assert!(
            kernel
                .launch_mixed_solve(&metadata, &short_rhs, &factor, &mut output, dims)
                .is_err()
        );
        assert!(
            kernel
                .launch_mixed_solve(&metadata, &rhs, &short_factor, &mut output, dims)
                .is_err()
        );
        assert!(
            kernel
                .launch_mixed_solve(&metadata, &rhs, &factor, &mut short_output, dims)
                .is_err()
        );
        kernel
            .launch_mixed_solve(&metadata, &rhs, &factor, &mut output, dims)
            .unwrap();
    }
    let mut actual = vec![0.0; 782];
    output.read_range_into(0, &mut actual).unwrap();
    assert!(
        actual[..4]
            .iter()
            .chain(&actual[778..])
            .all(|&v| v == -131072.0)
    );
    for group in 0..6 {
        for i in 0..129 {
            assert_eq!(
                actual[4 + group * 129 + i],
                (group * 1000 + i) as f32 + ((i % 128 + 1) % 128) as f32 + 0.25
            );
        }
    }
}

#[test]
#[ignore = "requires NVIDIA/NVRTC; new Small64 ABI purpose"]
fn small_factor_64_abi_checks_sessions_capacities_and_work_partition() {
    let session = TransferSession::new(0).unwrap();
    let other = TransferSession::new(0).unwrap();
    let source = r#"
extern "C" __global__ void small_abi_probe(const int* m,const float* a,const float* b,float* o,
 unsigned prefix,unsigned first,unsigned count,unsigned work) {
 __shared__ unsigned slots[64];slots[threadIdx.x]=threadIdx.x;__syncthreads();
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;
 if(tid<work)o[4+tid]=blockDim.x*1000+blockIdx.x*64+slots[63-threadIdx.x];
}
"#;
    let kernel = SynchronousKernel::compile(&session, source, "small_abi_probe").unwrap();
    let m = session.upload(&vec![0i32; 192]).unwrap();
    let a = session.upload(&vec![0f32; 136]).unwrap();
    let mut out = session.upload(&vec![-131072f32; 136]).unwrap();
    let fm = other.upload(&vec![0i32; 192]).unwrap();
    let fa = other.upload(&vec![0f32; 136]).unwrap();
    let mut fo = other.upload(&vec![0f32; 136]).unwrap();
    let tiny_m = session.upload(&[0i32]).unwrap();
    let tiny = session.upload(&[0f32]).unwrap();
    let mut tiny_out = session.upload(&[0f32]).unwrap();
    // SAFETY: Rejection cases return before launch. The accepted probe has
    // two full64 blocks, 128 output slots and exactly64 shared slots.
    unsafe {
        for k in 0..4 {
            assert_eq!(
                kernel.launch_small_factor(
                    if k == 0 { &fm } else { &m },
                    if k == 1 { &fa } else { &a },
                    if k == 2 { &fa } else { &a },
                    if k == 3 { &mut fo } else { &mut out },
                    [64, 0, 64, 128]
                ),
                Err(TransferError::SessionMismatch)
            );
        }
        for dims in [
            [64, 0, 0, 128],
            [64, 0, 64, 0],
            [64, 0, 64, 129],
            [u32::MAX, 0, 64, 128],
            [64, u32::MAX, 64, 128],
            [64, 0, 64, u32::MAX],
        ] {
            assert!(matches!(
                kernel.launch_small_factor(&m, &a, &a, &mut out, dims),
                Err(TransferError::Input(_))
            ));
        }
        for k in 0..4 {
            assert!(matches!(
                kernel.launch_small_factor(
                    if k == 0 { &tiny_m } else { &m },
                    if k == 1 { &tiny } else { &a },
                    if k == 2 { &tiny } else { &a },
                    if k == 3 { &mut tiny_out } else { &mut out },
                    [64, 0, 64, 128]
                ),
                Err(TransferError::Input(_))
            ));
        }
        kernel
            .launch_small_factor(&m, &a, &a, &mut out, [64, 0, 64, 128])
            .unwrap();
    }
    let mut values = vec![0f32; 136];
    out.read_range_into(0, &mut values).unwrap();
    assert!(
        values[..4]
            .iter()
            .chain(&values[132..])
            .all(|&v| v == -131072.0)
    );
    for group in 0..2 {
        for tid in 0..64 {
            assert_eq!(
                values[4 + group * 64 + tid],
                (64000 + group * 64 + 63 - tid) as f32
            );
        }
    }
}

#[test]
#[ignore = "requires NVIDIA driver/NVRTC; new 128-thread sparse solve ABI purpose"]
fn sparse_solve_128_abi_checks_sessions_capacities_and_block_barriers() {
    let session = TransferSession::new(0).unwrap();
    let other = TransferSession::new(0).unwrap();
    let source = r#"
extern "C" __global__ void sparse_abi_probe(const int* m,const float* r,const float* f,
 float* o,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 __shared__ unsigned count[128];
 count[threadIdx.x]=threadIdx.x;
 __syncthreads();
 o[4+(unsigned long long)blockIdx.x*n+threadIdx.x]=blockDim.x*1000+blockIdx.x*n+count[127-threadIdx.x];
}
"#;
    let kernel = SynchronousKernel::compile(&session, source, "sparse_abi_probe").unwrap();
    let m = session.upload(&vec![0i32; 258]).unwrap();
    let r = session.upload(&vec![0f32; 776]).unwrap();
    let f = session.upload(&vec![0f32; 392]).unwrap();
    let mut out = session.upload(&vec![-131072f32; 776]).unwrap();
    let fm = other.upload(&vec![0i32; 258]).unwrap();
    let fr = other.upload(&vec![0f32; 776]).unwrap();
    let ff = other.upload(&vec![0f32; 392]).unwrap();
    let mut fo = other.upload(&vec![0f32; 776]).unwrap();
    let small_m = session.upload(&[0i32]).unwrap();
    let small = session.upload(&[0f32]).unwrap();
    let mut small_out = session.upload(&[0f32]).unwrap();
    // SAFETY: Rejection cases return before launch. The accepted probe has
    // n=128, six disjoint guarded slices, and exactly 128 shared slots.
    unsafe {
        for k in 0..4 {
            assert_eq!(
                kernel.launch_sparse_solve(
                    if k == 0 { &fm } else { &m },
                    if k == 1 { &fr } else { &r },
                    if k == 2 { &ff } else { &f },
                    if k == 3 { &mut fo } else { &mut out },
                    [128, 0, 2, 3]
                ),
                Err(TransferError::SessionMismatch)
            );
        }
        for dim in [
            [128, 0, 0, 3],
            [128, 0, 2, 0],
            [128, 0, u32::MAX, 3],
            [u32::MAX, 0, 2, 3],
            [128, u32::MAX, 2, 3],
            [128, 0, 2, u32::MAX],
        ] {
            assert!(matches!(
                kernel.launch_sparse_solve(&m, &r, &f, &mut out, dim),
                Err(TransferError::Input(_))
            ));
        }
        for k in 0..4 {
            assert!(matches!(
                kernel.launch_sparse_solve(
                    if k == 0 { &small_m } else { &m },
                    if k == 1 { &small } else { &r },
                    if k == 2 { &small } else { &f },
                    if k == 3 { &mut small_out } else { &mut out },
                    [128, 0, 2, 3]
                ),
                Err(TransferError::Input(_))
            ));
        }
        kernel
            .launch_sparse_solve(&m, &r, &f, &mut out, [128, 0, 2, 3])
            .unwrap();
    }
    let mut values = vec![0f32; 776];
    out.read_range_into(0, &mut values).unwrap();
    assert!(
        values[..4]
            .iter()
            .chain(&values[772..])
            .all(|&v| v == -131072.0)
    );
    for group in 0..6 {
        for tid in 0..128 {
            assert_eq!(
                values[4 + group * 128 + tid],
                (128000 + group * 128 + 127 - tid) as f32
            );
        }
    }
}
