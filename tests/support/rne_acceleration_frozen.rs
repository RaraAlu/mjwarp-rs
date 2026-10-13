//! 新qacc阶段首差诊断。
//! 它不提供原生超差放行。
use cudarc::{
    driver::{
        CudaContext, CudaSlice, CudaStream, DevicePtr, DeviceRepr, LaunchConfig, PushKernelArg,
    },
    nvrtc::Ptx,
};
use sha2::{Digest, Sha256};

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
// SAFETY: Fixed-width repr(C), no references or uninitialized padding;
// sizes match the pinned Warp 1.15 PTX 56-byte and 24-byte value parameters.
unsafe impl DeviceRepr for Array {}
// SAFETY: One-dimensional bounds use shape[1] as explicit zero padding.
// Two-dimensional bounds use both entries; both frozen layouts are 24 bytes.
unsafe impl DeviceRepr for Bounds {}

fn array<T>(
    buffer: &CudaSlice<T>,
    stream: &CudaStream,
    shape: &[usize],
    width: usize,
    guarded: bool,
) -> Array {
    let (ptr, _record) = buffer.device_ptr(stream);
    let bytes = width * 4;
    Array {
        data: ptr + if guarded { 16 } else { 0 },
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
fn bounds(x: usize, y: Option<usize>) -> Bounds {
    Bounds {
        shape: [x as i32, y.unwrap_or(0) as i32],
        size: (x * y.unwrap_or(1)) as u64,
        coord_mult: 1,
    }
}

pub(super) fn run(
    metadata: &[i32],
    qvel: &[f32],
    qacc: &[f32],
    com: &[f32],
    velocity: &[f32],
    flag: bool,
) -> Vec<f32> {
    assert_eq!(std::mem::size_of::<Array>(), 56);
    assert_eq!(std::mem::size_of::<Bounds>(), 24);
    let nb = metadata[0] as usize;
    let nv = metadata[1] as usize;
    assert!(nv > 0);
    assert_eq!(qvel.len(), nv);
    assert_eq!(qacc.len(), nv);
    assert_eq!(com.len(), 14 * nb + 6 * nv);
    assert_eq!(velocity.len(), 6 * nb + 6 * nv);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let prefix = root.join("target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6");
    let raw = std::fs::read(format!("{}.sm89.ptx", prefix.display())).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&raw)),
        "1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd"
    );
    let raw_cu = std::fs::read(format!("{}.cu", prefix.display())).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(raw_cu)),
        "3a6b3856727f63797a28646a5864aa02cd76a20ed3658199c69b3ad75c802bfa"
    );
    let text = std::str::from_utf8(&raw)
        .unwrap()
        .strip_suffix('\0')
        .unwrap();
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    let module = context.load_module(Ptx::from_src(text)).unwrap();
    let parent = &metadata[5..5 + nb];
    let num = &metadata[5 + nb..5 + 2 * nb];
    let adr = &metadata[5 + 2 * nb..5 + 3 * nb];
    let body = &metadata[5 + 3 * nb..5 + 3 * nb + nv];
    let depth = &metadata[5 + 3 * nb + nv..];
    // 冻结io.py逐叶构造完整祖先链。
    let mut branches = Vec::new();
    let mut starts = Vec::new();
    for leaf in 1..nb {
        if parent[1..].contains(&(leaf as i32)) {
            continue;
        }
        starts.push(branches.len() as i32);
        let mut chain = Vec::new();
        let mut b = leaf;
        while b != 0 {
            chain.push(b as i32);
            b = parent[b] as usize;
        }
        branches.extend(chain.into_iter().rev());
    }
    starts.push(branches.len() as i32);
    let nbranch = starts.len() - 1;
    assert!(nbranch > 0);
    let dp = stream.clone_htod(parent).unwrap();
    let dn = stream.clone_htod(num).unwrap();
    let da = stream.clone_htod(adr).unwrap();
    let db = stream.clone_htod(&branches).unwrap();
    let ds = stream.clone_htod(&starts).unwrap();
    let di = stream.clone_htod(body).unwrap();
    let dqv = stream.clone_htod(qvel).unwrap();
    let dqa = stream.clone_htod(qacc).unwrap();
    let dci = stream.clone_htod(&com[4 * nb..14 * nb]).unwrap();
    let dcd = stream.clone_htod(&com[14 * nb..]).unwrap();
    let dcv = stream.clone_htod(&velocity[..6 * nb]).unwrap();
    let ddot = stream.clone_htod(&velocity[6 * nb..]).unwrap();
    let gravity: Vec<_> = metadata[2..5]
        .iter()
        .map(|&v| f32::from_bits(v as u32))
        .collect();
    let dg = stream.clone_htod(&gravity).unwrap();
    let output = |length| {
        let mut v = vec![0.0f32; length + 8];
        v[..4].fill(-131072.0);
        v[length + 4..].fill(-131072.0);
        stream.clone_htod(&v).unwrap()
    };
    let acc = output(6 * nb);
    let frc = output(6 * nb);
    let bias = output(nv);
    let ext = output(6 * nb);
    let a = |buffer: &CudaSlice<f32>, n, width, guarded| {
        array(buffer, &stream, &[1, n], width, guarded)
    };
    let i = |buffer: &CudaSlice<i32>, n| array(buffer, &stream, &[n], 1, false);
    let config = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (256, 1, 1),
        shared_mem_bytes: 0,
    };
    // SAFETY: All five entry signatures match the frozen CU and PTX value ABI.
    // Contiguous array descriptors have checked sizes, zero grad and live buffers.
    // Input and output buffers do not alias, except intentional backward in-place
    // accumulation. Level barriers prevent parent reads racing descendant writes.
    // Each launch synchronizes before any descriptor or buffer goes out of scope.
    unsafe {
        let function = module
            .load_function("_cacc_world_6ec26fdb_cuda_kernel_forward")
            .unwrap();
        stream
            .launch_builder(&function)
            .arg(&bounds(1, None))
            .arg(&array(&dg, &stream, &[1], 3, false))
            .arg(&a(&acc, nb, 6, true))
            .launch(config)
            .unwrap();
        stream.synchronize().unwrap();
        let function = module
            .load_function("_cacc_branch_ffe1d6c4_cuda_kernel_forward")
            .unwrap();
        let mut launch = stream.launch_builder(&function);
        let dim = bounds(1, Some(nbranch));
        launch.arg(&dim);
        let args = [
            i(&dp, nb),
            i(&dn, nb),
            i(&da, nb),
            i(&db, branches.len()),
            i(&ds, starts.len()),
            a(&dqv, nv, 1, false),
            a(&dqa, nv, 1, false),
            a(&dcd, nv, 6, false),
            a(&ddot, nv, 6, false),
        ];
        for argument in &args {
            launch.arg(argument);
        }
        let flag = u8::from(flag);
        let acc_arg = a(&acc, nb, 6, true);
        launch.arg(&flag).arg(&acc_arg).launch(config).unwrap();
        stream.synchronize().unwrap();
        let function = module
            .load_function("_cfrc_3922b605_cuda_kernel_forward")
            .unwrap();
        stream
            .launch_builder(&function)
            .arg(&bounds(1, Some(nb)))
            .arg(&a(&dci, nb, 10, false))
            .arg(&a(&dcv, nb, 6, false))
            .arg(&a(&acc, nb, 6, true))
            .arg(&a(&ext, nb, 6, true))
            .arg(&0u8)
            .arg(&a(&frc, nb, 6, true))
            .launch(config)
            .unwrap();
        stream.synchronize().unwrap();
        let function = module
            .load_function("_cfrc_backward_72237d38_cuda_kernel_forward")
            .unwrap();
        for level in (0..=*depth.iter().max().unwrap()).rev() {
            let nodes: Vec<_> = (0..nb)
                .filter(|&b| depth[b] == level)
                .map(|b| b as i32)
                .collect();
            let tree = stream.clone_htod(&nodes).unwrap();
            stream
                .launch_builder(&function)
                .arg(&bounds(1, Some(nodes.len())))
                .arg(&i(&dp, nb))
                .arg(&a(&frc, nb, 6, true))
                .arg(&i(&tree, nodes.len()))
                .arg(&a(&frc, nb, 6, true))
                .launch(config)
                .unwrap();
            stream.synchronize().unwrap();
        }
        let function = module
            .load_function("_qfrc_bias_4085b6de_cuda_kernel_forward")
            .unwrap();
        stream
            .launch_builder(&function)
            .arg(&bounds(1, Some(nv)))
            .arg(&i(&di, nv))
            .arg(&a(&dcd, nv, 6, false))
            .arg(&a(&frc, nb, 6, true))
            .arg(&a(&bias, nv, 1, true))
            .launch(config)
            .unwrap();
        stream.synchronize().unwrap();
    }
    let mut result = Vec::new();
    for buffer in [&acc, &frc, &bias] {
        let values = stream.clone_dtoh(buffer).unwrap();
        assert_eq!(&values[..4], &[-131072.0; 4]);
        assert_eq!(&values[values.len() - 4..], &[-131072.0; 4]);
        result.extend_from_slice(&values[4..values.len() - 4]);
    }
    result
}
