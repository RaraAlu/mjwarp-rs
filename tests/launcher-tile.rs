#![cfg(feature = "cuda-probe")]
//! W14：tile 共享内存声明、发射约束与静态 `__shared__` 编译门禁。
//! 产品测试只读静态冻结夹具，不启动上游作者流程。

use mjwarp_rs::runtime::launcher::{
    AbiSignature, Access, ArtifactCache, ArtifactKey, BlockRule, CompileRecipe,
    DeviceSharedMemoryLimits, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    NvrtcSource, RecipePurpose, SharedMemoryLimits, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
use std::path::PathBuf;

const SCALAR_SLOT: [SlotDesc; 1] = [SlotDesc {
    index: 0,
    kind: SlotKind::Scalar { bytes: 4 },
    access: Access::Read,
    role: SlotRole::Value,
}];

const TILE_SOURCE: &str = r#"
#define WP_NO_BFLOAT16
#define WP_TILE_BLOCK_DIM 32
#define WP_NO_CRT
#include "builtin.h"
#include "deterministic.h"

extern "C" __global__ void w14_tile_probe(float* out, int n) {
    __shared__ __align__(16) float tile[49];
    __shared__ int info[1];
    int lane = threadIdx.x;
    for (int i = lane; i < 49; i += blockDim.x) {
        tile[i] = 0.0f;
    }
    __syncthreads();
    if (lane == 0) {
        info[0] = 1;
    }
    __syncthreads();
    if (lane == 0) {
        out[blockIdx.x] = tile[0] + float(info[0]);
    }
    __syncthreads();
}
"#;

struct Tile32;
unsafe impl KernelAbi for Tile32 {
    const NAME: &'static str = "w14_tile_probe";
    const BLOCK: BlockRule = BlockRule::Tile(32);
    const SHARED_BYTES: u32 = 49 * 4 + 4;
    const SLOTS: &'static [SlotDesc] = &SCALAR_SLOT;
}
impl KernelSpec for Tile32 {
    type Dims = u32;

    fn spans(_dims: &u32) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(Vec::new())
    }

    fn launch_dims(dims: &u32) -> Result<LaunchDims, LauncherError> {
        LaunchDims::new((*dims, 1, 1), (32, 1, 1), 0)
    }
}

struct Tile64;
unsafe impl KernelAbi for Tile64 {
    const NAME: &'static str = "w14_tile_probe_wrong_block";
    const BLOCK: BlockRule = BlockRule::Tile(64);
    const SHARED_BYTES: u32 = 0;
    const SLOTS: &'static [SlotDesc] = &SCALAR_SLOT;
}
impl KernelSpec for Tile64 {
    type Dims = u32;

    fn spans(_dims: &u32) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(Vec::new())
    }

    fn launch_dims(dims: &u32) -> Result<LaunchDims, LauncherError> {
        LaunchDims::new((*dims, 1, 1), (32, 1, 1), 0)
    }
}

struct BadShared;
unsafe impl KernelAbi for BadShared {
    const NAME: &'static str = "w14_bad_shared";
    const BLOCK: BlockRule = BlockRule::Tile(32);
    const SHARED_BYTES: u32 = 6;
    const SLOTS: &'static [SlotDesc] = &SCALAR_SLOT;
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn tile_block_rule_must_equal_the_compiled_thread_count() {
    assert_eq!(BlockRule::Tile(32).threads(), 32);
    let kernel = Kernel::<Tile32>::new().unwrap();
    let plan = kernel.plan(&4u32, &[], &[]).unwrap();
    assert_eq!(plan.dims().block(), (32, 1, 1));
    assert_eq!(plan.dims().shared_bytes(), 0);
    let wrong = Kernel::<Tile64>::new().unwrap();
    assert!(matches!(
        wrong.plan(&4u32, &[], &[]),
        Err(LauncherError::InvalidArray("线程块违反ABI规则"))
    ));
    for block in [
        BlockRule::Tile(0),
        BlockRule::Tile(48),
        BlockRule::Tile(2048),
    ] {
        assert!(AbiSignature::check("tile", block, &SCALAR_SLOT, &[]).is_err());
    }
    assert!(AbiSignature::check("tile", BlockRule::Tile(32), &SCALAR_SLOT, &[]).is_ok());
}

#[test]
fn launch_dims_reject_dynamic_shared_memory_without_opt_in_and_over_limits() {
    assert!(matches!(
        LaunchDims::new((1, 1, 1), (32, 1, 1), 64),
        Err(LauncherError::Unsupported("动态共享内存未opt-in"))
    ));
    assert!(matches!(
        LaunchDims::new((1, 1, 1), (32, 1, 1), 64 * 1024),
        Err(LauncherError::InvalidArray("共享内存超过设备上限"))
    ));
    let limits = SharedMemoryLimits::new(48 * 1024, 128 * 1024).unwrap();
    assert_eq!(limits.per_block(), 48 * 1024);
    assert_eq!(limits.opt_in(), 128 * 1024);
    assert!(SharedMemoryLimits::new(0, 1).is_err());
    assert!(SharedMemoryLimits::new(64 * 1024, 32 * 1024).is_err());
    assert!(matches!(
        LaunchDims::with_limits((1, 1, 1), (32, 1, 1), 64 * 1024, limits, false),
        Err(LauncherError::InvalidArray("共享内存超过设备上限"))
    ));
    let dynamic = LaunchDims::with_limits((1, 1, 1), (32, 1, 1), 64 * 1024, limits, true).unwrap();
    assert_eq!(dynamic.shared_bytes(), 64 * 1024);
    assert!(matches!(
        LaunchDims::with_limits((1, 1, 1), (32, 1, 1), 160 * 1024, limits, true),
        Err(LauncherError::InvalidArray("共享内存超过设备上限"))
    ));
}

#[test]
fn static_shared_memory_is_checked_against_device_limits() {
    let kernel = Kernel::<Tile32>::new().unwrap();
    assert_eq!(
        kernel.plan(&2u32, &[], &[]).unwrap().dims().shared_bytes(),
        0
    );
    let tight = SharedMemoryLimits::new(64, 128).unwrap();
    assert!(matches!(
        kernel.plan_with_limits(&2u32, &[], &[], tight),
        Err(LauncherError::InvalidArray("静态共享内存超过设备上限"))
    ));
    let roomy = SharedMemoryLimits::new(48 * 1024, 48 * 1024).unwrap();
    assert!(kernel.plan_with_limits(&2u32, &[], &[], roomy).is_ok());
    assert!(matches!(
        AbiSignature::of::<BadShared>(),
        Err(LauncherError::InvalidSignature { .. })
    ));
}

#[test]
fn device_limits_intersect_with_injected_limits() {
    let injected = SharedMemoryLimits::new(96 * 1024, 128 * 1024).unwrap();
    let device = DeviceSharedMemoryLimits::new(64 * 1024, 96 * 1024).unwrap();
    let merged = injected.intersect(device);
    assert_eq!(merged.per_block(), 64 * 1024);
    assert_eq!(merged.opt_in(), 96 * 1024);
    let conservative = SharedMemoryLimits::DEFAULT
        .intersect(DeviceSharedMemoryLimits::new(32 * 1024, 32 * 1024).unwrap());
    assert_eq!(conservative.per_block(), 32 * 1024);
    assert_eq!(conservative.opt_in(), 32 * 1024);
    let clamped = SharedMemoryLimits::new(96 * 1024, 128 * 1024)
        .unwrap()
        .intersect(DeviceSharedMemoryLimits::new(64 * 1024, 64 * 1024).unwrap());
    assert_eq!(clamped.per_block(), 64 * 1024);
    assert_eq!(clamped.opt_in(), 64 * 1024);
    assert!(DeviceSharedMemoryLimits::new(0, 1).is_err());
    assert!(DeviceSharedMemoryLimits::new(64 * 1024, 32 * 1024).is_err());
    // 无 cuda-probe 时查询必须安全返回 None（回退保守默认）。
    assert!(DeviceSharedMemoryLimits::query(usize::MAX).is_none());
}

#[cfg(windows)]
#[test]
#[ignore = "requires NVIDIA driver (real device attribute query)"]
fn queries_real_device_shared_memory_limits() {
    let device = DeviceSharedMemoryLimits::query(0).expect("device shared-memory attributes");
    assert!(device.per_block() > 0);
    assert!(device.opt_in() >= device.per_block());
    let limits = SharedMemoryLimits::for_device(0);
    assert!(limits.per_block() <= SharedMemoryLimits::DEFAULT.per_block());
    assert_eq!(limits.per_block(), SharedMemoryLimits::DEFAULT.per_block());
    assert_eq!(
        limits,
        SharedMemoryLimits::DEFAULT.intersect(device),
        "for_device must intersect the conservative default with device attributes"
    );
}

#[cfg(windows)]
#[test]
#[ignore = "requires NVRTC and the CUDA SDK include directory (MJWARP_CUDA_INCLUDE)"]
fn nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers() {
    let cuda_include = std::env::var("MJWARP_CUDA_INCLUDE")
        .expect("set MJWARP_CUDA_INCLUDE to the CUDA 12.8 include directory");
    let fp16 = PathBuf::from(&cuda_include).join("cuda_fp16.h");
    let without_external = NvrtcSource::new("w14_tile_probe", TILE_SOURCE)
        .unwrap()
        .with_headers_from_dir(&root().join("include/warp-headers"))
        .unwrap()
        .with_include_path(cuda_include.clone())
        .unwrap();
    let source = without_external
        .clone()
        .with_external_header(&fp16)
        .unwrap();
    let key_without = ArtifactKey::new(
        &without_external,
        &CompileRecipe::WARP_1_15_DIAGNOSTIC,
        (12, 8),
    );
    let key_with = ArtifactKey::new(&source, &CompileRecipe::WARP_1_15_DIAGNOSTIC, (12, 8));
    assert_ne!(
        key_without.sha256(),
        key_with.sha256(),
        "cuda_fp16.h digest must enter the artifact key"
    );
    let cache = ArtifactCache::in_memory();
    let artifact = cache
        .compile(
            &source,
            &CompileRecipe::WARP_1_15_DIAGNOSTIC,
            RecipePurpose::Diagnostic,
        )
        .unwrap();
    assert!(artifact.ptx().contains("w14_tile_probe"));
    assert!(
        artifact.ptx().contains(".shared"),
        "PTX must declare static shared memory"
    );
    assert_eq!(cache.stats().compiles, 1);
    let tile = Kernel::<Tile32>::new().unwrap();
    assert!(tile.plan(&7u32, &[], &[]).is_ok());
}
