//! 新耦合稀疏输入专用诊断。
//! 直接加载冻结PTX，不编译或执行Python。
use cudarc::driver::{
    CudaContext, CudaSlice, CudaStream, DevicePtr, DeviceRepr, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::Ptx;
use sha2::{Digest, Sha256};

const CACHE: &str = "target/g03-block-reference/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6";
pub const CU_HASH: &str = "dcb2ad088602f8e514ab6e71ac9e1befd49822affb1b199af7514ee4533f6c35";
pub const PTX_HASH: &str = "20e3469d3360cbe8b3cae3f5bf2a46cbeb1f33160d136804c3892cf1d2c453f9";
#[repr(C)]
#[derive(Clone, Copy)]
struct Array {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
}
#[repr(C)]
struct Bounds {
    shape: [i32; 2],
    size: u64,
    coord_mult: u64,
}
// SAFETY: repr(C), initialized padding, and frozen array_t ABI assertions below.
unsafe impl DeviceRepr for Array {}
// SAFETY: Fixed launch_bounds_t<2> layout; all fields hold initialized integers.
unsafe impl DeviceRepr for Bounds {}
fn array<T>(b: &CudaSlice<T>, s: &CudaStream, shape: &[usize], width: usize) -> Array {
    let (ptr, _record) = b.device_ptr(s);
    let bytes = 4 * width;
    Array {
        data: ptr,
        grad: 0,
        shape: [shape[0] as i32, *shape.get(1).unwrap_or(&0) as i32, 0, 0],
        strides: [
            (if shape.len() == 2 {
                shape[1] * bytes
            } else {
                bytes
            }) as i32,
            if shape.len() == 2 { bytes as i32 } else { 0 },
            0,
            0,
        ],
        ndim: shape.len() as u16,
        flags: 0,
        padding: 0,
    }
}
fn bounds(worlds: usize, items: usize) -> Bounds {
    Bounds {
        shape: [worlds as i32, items as i32],
        size: (worlds * items) as u64,
        coord_mult: 1,
    }
}

/// 层序与更新元组来自独立C++冻结转录，不取产品生成值。
pub fn factor(
    rownnz: &[i32],
    rowadr: &[i32],
    updates: &[i32],
    offsets: &[i32],
    matrix: &[f32],
    worlds: usize,
) -> (Vec<f32>, Vec<f32>) {
    assert_eq!(std::mem::size_of::<Array>(), 56);
    assert_eq!(std::mem::size_of::<Bounds>(), 24);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut ptx = String::new();
    for (ext, hash) in [("cu", CU_HASH), ("sm89.ptx", PTX_HASH)] {
        let bytes = std::fs::read(root.join(format!("{CACHE}.{ext}"))).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), hash);
        if ext == "sm89.ptx" {
            ptx = String::from_utf8(bytes).unwrap();
        }
    }
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    let module = context
        .load_module(Ptx::from_src(ptx.trim_end_matches('\0')))
        .unwrap();
    // 冻结PTX保留256项线程共享槽；默认1024线程会越界。
    assert!(ptx.contains("get_smem_baseEvE9smem_base[1024]"));
    let config = |items: usize| LaunchConfig {
        grid_dim: ((items as u32).div_ceil(256), 1, 1),
        block_dim: (256, 1, 1),
        shared_mem_bytes: 0,
    };
    let update = module
        .load_function("_qLD_acc_120d68f2_cuda_kernel_forward")
        .unwrap();
    let diag = module
        .load_function("_qLDiag_div_3444613a_cuda_kernel_forward")
        .unwrap();
    let n = rownnz.len();
    let nc = matrix.len() / worlds;
    let rn = stream.clone_htod(rownnz).unwrap();
    let ra = stream.clone_htod(rowadr).unwrap();
    let all = stream.clone_htod(updates).unwrap();
    let l = stream.clone_htod(matrix).unwrap();
    let d = stream.clone_htod(&vec![0.0f32; worlds * n]).unwrap();
    let rn_arg = array(&rn, &stream, &[n], 1);
    let ra_arg = array(&ra, &stream, &[n], 1);
    let l_arg = array(&l, &stream, &[worlds, nc], 1);
    let d_arg = array(&d, &stream, &[worlds, n], 1);
    for level in (0..offsets.len() - 1).rev() {
        let count = (offsets[level + 1] - offsets[level]) as usize;
        let mut u_arg = array(&all, &stream, &[count], 3);
        u_arg.data += (offsets[level] as u64) * 12;
        let dim = bounds(worlds, count);
        let mut launch = stream.launch_builder(&update);
        launch
            .arg(&dim)
            .arg(&rn_arg)
            .arg(&ra_arg)
            .arg(&u_arg)
            .arg(&l_arg)
            .arg(&l_arg);
        // SAFETY: Frozen 24/56-byte ABI, real native CSR ranges, independent
        // updates and exact in-place L alias. Allocations survive synchronization.
        unsafe {
            launch.launch(config(worlds * count)).unwrap();
        }
        stream.synchronize().unwrap();
    }
    let dim = bounds(worlds, n);
    let mut launch = stream.launch_builder(&diag);
    launch
        .arg(&dim)
        .arg(&rn_arg)
        .arg(&ra_arg)
        .arg(&l_arg)
        .arg(&d_arg);
    // SAFETY: Same asserted ABI; n diagonal writes per world stay inside d.
    unsafe {
        launch.launch(config(worlds * n)).unwrap();
    }
    stream.synchronize().unwrap();
    (
        stream.clone_dtoh(&l).unwrap(),
        stream.clone_dtoh(&d).unwrap(),
    )
}
