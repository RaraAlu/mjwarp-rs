#![cfg(feature = "cuda-probe")]
//! W14：tile 共享内存声明、发射约束与静态 `__shared__` 编译门禁。
//! 产品测试只读静态冻结夹具，不启动上游作者流程。

use mjwarp_rs::runtime::launcher::{
    AbiSignature, Access, ArtifactCache, BlockRule, CompileRecipe, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, NvrtcSource, RecipePurpose, SharedMemoryLimits, SlotDesc, SlotKind,
    SlotRequirement, SlotRole,
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

#[cfg(windows)]
#[test]
#[ignore = "requires NVRTC and the CUDA SDK include directory (MJWARP_CUDA_INCLUDE)"]
fn nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers() {
    let cuda_include = std::env::var("MJWARP_CUDA_INCLUDE")
        .expect("set MJWARP_CUDA_INCLUDE to the CUDA 12.8 include directory");
    let source = NvrtcSource::new("w14_tile_probe", TILE_SOURCE)
        .unwrap()
        .with_headers_from_dir(&root().join("include/warp-headers"))
        .unwrap()
        .with_include_path(cuda_include)
        .unwrap();
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
