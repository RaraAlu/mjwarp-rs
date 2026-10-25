//! G10@P5：产品 GPU 入口（NVRTC 运行时编译 + 逐 world launch/回读）。
//!
//! 与 G5 独立 nvcc 探针同源：设备函数来自 `src/physics/collision/cuda/*.cu`，由
//! [`source`] 按依赖顺序拼成一个翻译单元并追加 `product.cu`，经 cudarc 的 NVRTC
//! 直接 API 编译为 PTX，加载后按 `grid=(nworld,1,1)`/`block=(1,1,1)` 发射并回读。
//!
//! 编译选项对齐探针 nvcc 配方：`fmad=true`、`use_fast_math=false`、`--std=c++17`、
//! `--gpu-architecture=sm_89`；NVRTC 默认优化级别即 `-O3`。所有失败返回
//! [`CollisionGpuError`]（编译期 panic 由 `catch_unwind` 兜底），不 panic。
//!
//! 无 GPU 单测走 seam：`compile_source_with`（注入编译闭包）与 `program_from_parts`
//! （注入构造错误），测试里不创建真实 `CudaContext`。

mod abi;
mod error;
mod source;

#[cfg(test)]
mod nvrtc_smoke;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use cudarc::driver::{
    CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, DevicePtr, DevicePtrMut,
    DeviceRepr, LaunchConfig, PushKernelArg, SyncOnDrop, ValidAsZeroBits,
};
use cudarc::nvrtc::{CompileOptions, Ptx, compile_ptx_with_opts};

use abi::{CollisionArgs, CollisionContactOut, validate_inputs};
pub use error::CollisionGpuError;

/// 产品入口符号（与 `product.cu` 的 `extern "C" __global__` 一一对应）。
const CONTACT_ENTRY: &str = "mjwarp_collision_contact";
const HFIELD_ENTRY: &str = "mjwarp_collision_hfield";
const SDF_ENTRY: &str = "mjwarp_collision_sdf";

/// 产品内核种类：决定 `run` 走哪条设备路径。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionKernel {
    /// `mjwarp_collision_contact`：`eval_ccd_write_contact` 单对路径。
    Contact,
    /// `mjwarp_collision_hfield`：hfield prism CCD 路径。
    Hfield,
    /// `mjwarp_collision_sdf`：SDF 单 pair 子集（每初始点一次下降）。
    Sdf,
}

/// 设备侧标量参数（与探针 `meta.txt` 的标量一一对应，字段顺序即 ABI 顺序）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionGpuMeta {
    pub nworld: i32,
    pub ngeom: i32,
    pub g1: i32,
    pub g2: i32,
    pub type1: i32,
    pub type2: i32,
    pub geomgeomid: i32,
    pub naccdmax: i32,
    pub hfield_id: i32,
    pub oct_root: i32,
    pub noct: i32,
    pub tolerance: f32,
    pub margin: f32,
    pub gap: f32,
    pub gjk_iterations: i32,
    pub epa_iterations: i32,
    pub condim: i32,
    pub pairid0: i32,
    pub pairid1: i32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
    pub adhesion: f32,
    pub sdf_initpoints: i32,
    pub sdf_iterations: i32,
    pub naconmax: i32,
    pub efc_width: i32,
}

/// 一次发射的 23 个宿主输入数组（与探针 `DeviceArrays` 相同的 slice 口径）。
///
/// 模型静态字段（mesh/hfield/octree）按上游口径与 world 无关：调用方给一份共享数据。
#[derive(Clone, Copy, Debug)]
pub struct CollisionGpuInputs<'a> {
    pub geom_xpos: &'a [f32],
    pub geom_xmat: &'a [f32],
    pub geom_size: &'a [f32],
    pub geom_margin: &'a [f32],
    pub geom_gap: &'a [f32],
    pub geom_rbound: &'a [f32],
    pub geom_aabb: &'a [f32],
    pub geom_type: &'a [i32],
    pub geom_dataid: &'a [i32],
    pub mesh_vertadr: &'a [i32],
    pub mesh_vertnum: &'a [i32],
    pub mesh_graphadr: &'a [i32],
    pub mesh_octadr: &'a [i32],
    pub mesh_vert: &'a [f32],
    pub mesh_graph: &'a [i32],
    pub hfield_size: &'a [f32],
    pub hfield_nrow: &'a [i32],
    pub hfield_ncol: &'a [i32],
    pub hfield_adr: &'a [i32],
    pub hfield_data: &'a [f32],
    pub oct_aabb: &'a [f32],
    pub oct_child: &'a [i32],
    pub oct_coeff: &'a [f32],
    pub meta: CollisionGpuMeta,
}

/// 回读结果：数组字段只保留前 `nacon` 行（与探针 `out1/` 落盘口径一致）。
#[derive(Clone, Debug)]
pub struct CollisionGpuOutput {
    pub nacon: i32,
    pub nccd: i32,
    /// 长度 = `nworld`（每 world 的 `OverflowType` 位汇总）。
    pub overflow: Vec<u32>,
    pub dist: Vec<f32>,
    pub pos: Vec<f32>,
    pub frame: Vec<f32>,
    pub includemargin: Vec<f32>,
    pub friction: Vec<f32>,
    pub solref: Vec<f32>,
    pub solreffriction: Vec<f32>,
    pub solimp: Vec<f32>,
    pub adhesion: Vec<f32>,
    pub dim: Vec<i32>,
    pub geom: Vec<i32>,
    pub efc_address: Vec<i32>,
    pub worldid: Vec<i32>,
    pub contact_type: Vec<i32>,
    pub geomcollisionid: Vec<i32>,
}

/// 已编译并加载的产品程序（一个上下文、一个模块、三个入口、默认流）。
pub struct CollisionGpuProgram {
    context: Arc<CudaContext>,
    /// 模块由三个 `CudaFunction` 共同持有；字段仅为显式表达生命周期。
    _module: Arc<CudaModule>,
    contact: CudaFunction,
    hfield: CudaFunction,
    sdf: CudaFunction,
    stream: Arc<CudaStream>,
}

impl CollisionGpuProgram {
    /// 编译设备源并在 `device` 上加载三个入口。
    pub fn new(device: usize) -> Result<Self, CollisionGpuError> {
        let ptx = compile_program()?;
        let context = CudaContext::new(device)
            .map_err(|error| format!("CudaContext::new({device}): {error:?}"));
        program_from_parts(context, Ok(ptx))
    }

    /// 所属上下文（调用方据此核对设备归属）。
    pub fn context(&self) -> &Arc<CudaContext> {
        &self.context
    }

    /// 发射一次内核并回读接触数组。
    ///
    /// `inputs.meta.naconmax`/`efc_width` 决定输出缓冲容量；`kind` 决定入口。
    pub fn run(
        &self,
        kind: CollisionKernel,
        inputs: &CollisionGpuInputs<'_>,
    ) -> Result<CollisionGpuOutput, CollisionGpuError> {
        validate_inputs(inputs)?;
        let stream = &self.stream;
        let meta = &inputs.meta;
        let naconmax = meta.naconmax as usize;
        let efc_width = meta.efc_width as usize;

        let geom_xpos = upload(stream, inputs.geom_xpos)?;
        let geom_xmat = upload(stream, inputs.geom_xmat)?;
        let geom_size = upload(stream, inputs.geom_size)?;
        let geom_margin = upload(stream, inputs.geom_margin)?;
        let geom_gap = upload(stream, inputs.geom_gap)?;
        let geom_rbound = upload(stream, inputs.geom_rbound)?;
        let geom_aabb = upload(stream, inputs.geom_aabb)?;
        let geom_type = upload(stream, inputs.geom_type)?;
        let geom_dataid = upload(stream, inputs.geom_dataid)?;
        let mesh_vertadr = upload(stream, inputs.mesh_vertadr)?;
        let mesh_vertnum = upload(stream, inputs.mesh_vertnum)?;
        let mesh_graphadr = upload(stream, inputs.mesh_graphadr)?;
        let mesh_octadr = upload(stream, inputs.mesh_octadr)?;
        let mesh_vert = upload(stream, inputs.mesh_vert)?;
        let mesh_graph = upload(stream, inputs.mesh_graph)?;
        let hfield_size = upload(stream, inputs.hfield_size)?;
        let hfield_nrow = upload(stream, inputs.hfield_nrow)?;
        let hfield_ncol = upload(stream, inputs.hfield_ncol)?;
        let hfield_adr = upload(stream, inputs.hfield_adr)?;
        let hfield_data = upload(stream, inputs.hfield_data)?;
        let oct_aabb = upload(stream, inputs.oct_aabb)?;
        let oct_child = upload(stream, inputs.oct_child)?;
        let oct_coeff = upload(stream, inputs.oct_coeff)?;

        let mut dist = zeroed::<f32>(stream, naconmax)?;
        let mut pos = zeroed::<f32>(stream, naconmax * 3)?;
        let mut frame = zeroed::<f32>(stream, naconmax * 9)?;
        let mut includemargin = zeroed::<f32>(stream, naconmax)?;
        let mut friction = zeroed::<f32>(stream, naconmax * 5)?;
        let mut solref = zeroed::<f32>(stream, naconmax * 2)?;
        let mut solreffriction = zeroed::<f32>(stream, naconmax * 2)?;
        let mut solimp = zeroed::<f32>(stream, naconmax * 5)?;
        let mut adhesion = zeroed::<f32>(stream, naconmax)?;
        let mut dim = zeroed::<i32>(stream, naconmax)?;
        let mut geom = zeroed::<i32>(stream, naconmax * 2)?;
        let mut efc_address = zeroed::<i32>(stream, naconmax * efc_width)?;
        let mut worldid = zeroed::<i32>(stream, naconmax)?;
        let mut contact_type = zeroed::<i32>(stream, naconmax)?;
        let mut geomcollisionid = zeroed::<i32>(stream, naconmax)?;
        let mut nacon = zeroed::<i32>(stream, 1)?;
        let mut nccd = zeroed::<i32>(stream, 1)?;
        let mut overflow = zeroed::<u32>(stream, meta.nworld as usize)?;

        let mut guards = Vec::new();
        let nacon_pointer = output_pointer(&mut nacon, stream, &mut guards);
        let args = CollisionArgs {
            geom_xpos: input_pointer(&geom_xpos, stream, &mut guards),
            geom_xmat: input_pointer(&geom_xmat, stream, &mut guards),
            geom_size: input_pointer(&geom_size, stream, &mut guards),
            geom_margin: input_pointer(&geom_margin, stream, &mut guards),
            geom_gap: input_pointer(&geom_gap, stream, &mut guards),
            geom_rbound: input_pointer(&geom_rbound, stream, &mut guards),
            geom_aabb: input_pointer(&geom_aabb, stream, &mut guards),
            geom_type: input_pointer(&geom_type, stream, &mut guards),
            geom_dataid: input_pointer(&geom_dataid, stream, &mut guards),
            mesh_vertadr: input_pointer(&mesh_vertadr, stream, &mut guards),
            mesh_vertnum: input_pointer(&mesh_vertnum, stream, &mut guards),
            mesh_graphadr: input_pointer(&mesh_graphadr, stream, &mut guards),
            mesh_octadr: input_pointer(&mesh_octadr, stream, &mut guards),
            mesh_vert: input_pointer(&mesh_vert, stream, &mut guards),
            mesh_graph: input_pointer(&mesh_graph, stream, &mut guards),
            hfield_size: input_pointer(&hfield_size, stream, &mut guards),
            hfield_nrow: input_pointer(&hfield_nrow, stream, &mut guards),
            hfield_ncol: input_pointer(&hfield_ncol, stream, &mut guards),
            hfield_adr: input_pointer(&hfield_adr, stream, &mut guards),
            hfield_data: input_pointer(&hfield_data, stream, &mut guards),
            oct_aabb: input_pointer(&oct_aabb, stream, &mut guards),
            oct_child: input_pointer(&oct_child, stream, &mut guards),
            oct_coeff: input_pointer(&oct_coeff, stream, &mut guards),

            nworld: meta.nworld,
            ngeom: meta.ngeom,
            g1: meta.g1,
            g2: meta.g2,
            type1: meta.type1,
            type2: meta.type2,
            geomgeomid: meta.geomgeomid,
            naccdmax: meta.naccdmax,
            hfield_id: meta.hfield_id,
            oct_root: meta.oct_root,
            noct: meta.noct,
            tolerance: meta.tolerance,
            margin: meta.margin,
            gap: meta.gap,
            gjk_iterations: meta.gjk_iterations,
            epa_iterations: meta.epa_iterations,
            condim: meta.condim,
            pairid0: meta.pairid0,
            pairid1: meta.pairid1,
            friction: meta.friction,
            solref: meta.solref,
            solreffriction: meta.solreffriction,
            solimp: meta.solimp,
            adhesion: meta.adhesion,
            sdf_initpoints: meta.sdf_initpoints,
            sdf_iterations: meta.sdf_iterations,
            naconmax: meta.naconmax,
            efc_width: meta.efc_width,

            out: CollisionContactOut {
                dist: output_pointer(&mut dist, stream, &mut guards),
                pos: output_pointer(&mut pos, stream, &mut guards),
                frame: output_pointer(&mut frame, stream, &mut guards),
                includemargin: output_pointer(&mut includemargin, stream, &mut guards),
                friction: output_pointer(&mut friction, stream, &mut guards),
                solref: output_pointer(&mut solref, stream, &mut guards),
                solreffriction: output_pointer(&mut solreffriction, stream, &mut guards),
                solimp: output_pointer(&mut solimp, stream, &mut guards),
                dim: output_pointer(&mut dim, stream, &mut guards),
                geom: output_pointer(&mut geom, stream, &mut guards),
                efc_address: output_pointer(&mut efc_address, stream, &mut guards),
                worldid: output_pointer(&mut worldid, stream, &mut guards),
                contact_type: output_pointer(&mut contact_type, stream, &mut guards),
                geomcollisionid: output_pointer(&mut geomcollisionid, stream, &mut guards),
                adhesion: output_pointer(&mut adhesion, stream, &mut guards),
                nacon: nacon_pointer,
                efc_address_width: meta.efc_width,
            },
            nacon: nacon_pointer,
            nccd: output_pointer(&mut nccd, stream, &mut guards),
            overflow: output_pointer(&mut overflow, stream, &mut guards),
        };

        let configuration = LaunchConfig {
            grid_dim: (meta.nworld as u32, 1, 1),
            block_dim: (1, 1, 1),
            shared_mem_bytes: 0,
        };
        let function = match kind {
            CollisionKernel::Contact => &self.contact,
            CollisionKernel::Hfield => &self.hfield,
            CollisionKernel::Sdf => &self.sdf,
        };
        let mut builder = stream.launch_builder(function);
        builder.arg(&args);
        // SAFETY: 每个设备指针都指向按 `meta` 容量分配、且在本调用结束前一直存活
        // 的设备缓冲；`args` 的布局由 `tests.rs` 的 ABI 测试锁死（与设备结构逐字段一致）；
        // `grid=(nworld,1,1)` 与设备侧 `blockIdx.x` 取 world 的约定一致。
        unsafe { builder.launch(configuration) }
            .map_err(|error| CollisionGpuError::Launch(format!("launch {kind:?}: {error:?}")))?;
        stream
            .synchronize()
            .map_err(|error| CollisionGpuError::Launch(format!("synchronize: {error:?}")))?;
        drop(guards);

        let nacon_values = readback(stream, &nacon)?;
        let nccd_values = readback(stream, &nccd)?;
        let overflow_values = readback(stream, &overflow)?;
        let dist_values = readback(stream, &dist)?;
        let pos_values = readback(stream, &pos)?;
        let frame_values = readback(stream, &frame)?;
        let includemargin_values = readback(stream, &includemargin)?;
        let friction_values = readback(stream, &friction)?;
        let solref_values = readback(stream, &solref)?;
        let solreffriction_values = readback(stream, &solreffriction)?;
        let solimp_values = readback(stream, &solimp)?;
        let adhesion_values = readback(stream, &adhesion)?;
        let dim_values = readback(stream, &dim)?;
        let geom_values = readback(stream, &geom)?;
        let efc_address_values = readback(stream, &efc_address)?;
        let worldid_values = readback(stream, &worldid)?;
        let contact_type_values = readback(stream, &contact_type)?;
        let geomcollisionid_values = readback(stream, &geomcollisionid)?;

        let nacon = nacon_values[0];
        let rows = nacon.clamp(0, meta.naconmax) as usize;
        Ok(CollisionGpuOutput {
            nacon,
            nccd: nccd_values[0],
            overflow: overflow_values,
            dist: truncated(dist_values, rows),
            pos: truncated(pos_values, rows * 3),
            frame: truncated(frame_values, rows * 9),
            includemargin: truncated(includemargin_values, rows),
            friction: truncated(friction_values, rows * 5),
            solref: truncated(solref_values, rows * 2),
            solreffriction: truncated(solreffriction_values, rows * 2),
            solimp: truncated(solimp_values, rows * 5),
            adhesion: truncated(adhesion_values, rows),
            dim: truncated(dim_values, rows),
            geom: truncated(geom_values, rows * 2),
            efc_address: truncated(efc_address_values, rows * efc_width),
            worldid: truncated(worldid_values, rows),
            contact_type: truncated(contact_type_values, rows),
            geomcollisionid: truncated(geomcollisionid_values, rows),
        })
    }
}

/// 编译选项：与 G5 探针 nvcc `-O3 -arch=sm_89 --fmad=true -std=c++17`（无 fast-math）对齐。
pub(crate) fn compile_options() -> CompileOptions {
    CompileOptions {
        fmad: Some(true),
        use_fast_math: Some(false),
        options: vec![
            "--std=c++17".to_owned(),
            "--gpu-architecture=sm_89".to_owned(),
        ],
        include_paths: vec![],
        name: Some("mjwarp_collision_product.cu".to_owned()),
        ..Default::default()
    }
}

/// 用注入的编译器编译拼装后的设备源（单测传返回 `Err` 的闭包，不碰 NVRTC）。
pub(crate) fn compile_source_with<F>(compiler: F) -> Result<Ptx, CollisionGpuError>
where
    F: FnOnce(&str, CompileOptions) -> Result<Ptx, String>,
{
    let text = source::assemble_device_source();
    let options = compile_options();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| compiler(&text, options)))
        .map_err(|_| CollisionGpuError::Compile("NVRTC编译异常".to_owned()))?
        .map_err(CollisionGpuError::Compile)
}

/// 生产编译路径：`compile_source_with` + 真实 `compile_ptx_with_opts`。
/// 失败信息带上设备源指纹，便于定位是哪个源修订编译失败。
pub(crate) fn compile_program() -> Result<Ptx, CollisionGpuError> {
    let fingerprint = source::source_sha256();
    compile_source_with(|text, options| {
        compile_ptx_with_opts(text, options)
            .map_err(|error| format!("source_sha256={fingerprint}: {error:?}"))
    })
}

/// 加载模块与三个入口（无 GPU 时不调用）。
pub(crate) fn load_program(
    context: Arc<CudaContext>,
    ptx: Ptx,
) -> Result<CollisionGpuProgram, CollisionGpuError> {
    let module = context
        .load_module(ptx)
        .map_err(|error| CollisionGpuError::Load(format!("load_module: {error:?}")))?;
    let load = |entry: &str| {
        module
            .load_function(entry)
            .map_err(|error| CollisionGpuError::Load(format!("{entry}: {error:?}")))
    };
    let contact = load(CONTACT_ENTRY)?;
    let hfield = load(HFIELD_ENTRY)?;
    let sdf = load(SDF_ENTRY)?;
    let stream = context.default_stream();
    Ok(CollisionGpuProgram {
        context,
        _module: module,
        contact,
        hfield,
        sdf,
        stream,
    })
}

/// 构造 seam：设备结果与 PTX 结果都可注入错误（单测不创建真实 context）。
/// 先判 PTX 错误，保证编译失败时根本不碰设备。
pub(crate) fn program_from_parts(
    device_result: Result<Arc<CudaContext>, String>,
    ptx: Result<Ptx, CollisionGpuError>,
) -> Result<CollisionGpuProgram, CollisionGpuError> {
    let ptx = ptx?;
    let context = device_result.map_err(CollisionGpuError::NoDevice)?;
    load_program(context, ptx)
}

fn zeroed<T: DeviceRepr + ValidAsZeroBits>(
    stream: &Arc<CudaStream>,
    len: usize,
) -> Result<CudaSlice<T>, CollisionGpuError> {
    stream
        .alloc_zeros::<T>(len)
        .map_err(|error| CollisionGpuError::Launch(format!("alloc: {error:?}")))
}

/// 上传 23 个输入数组之一；空数组给 1 元素零缓冲（0 字节分配在 CUDA 上无意义，
/// 且对应内核在该数组上的读取次数为 0）。
fn upload<T: DeviceRepr + ValidAsZeroBits>(
    stream: &Arc<CudaStream>,
    values: &[T],
) -> Result<CudaSlice<T>, CollisionGpuError> {
    if values.is_empty() {
        return zeroed(stream, 1);
    }
    stream
        .clone_htod(values)
        .map_err(|error| CollisionGpuError::Launch(format!("upload: {error:?}")))
}

fn input_pointer<'a, T>(
    slice: &'a CudaSlice<T>,
    stream: &'a CudaStream,
    guards: &mut Vec<SyncOnDrop<'a>>,
) -> *const T {
    let (pointer, guard) = slice.device_ptr(stream);
    guards.push(guard);
    pointer as usize as *const T
}

fn output_pointer<'a, T>(
    slice: &'a mut CudaSlice<T>,
    stream: &'a CudaStream,
    guards: &mut Vec<SyncOnDrop<'a>>,
) -> *mut T {
    let (pointer, guard) = slice.device_ptr_mut(stream);
    guards.push(guard);
    pointer as usize as *mut T
}

fn readback<T: DeviceRepr, S: DevicePtr<T>>(
    stream: &Arc<CudaStream>,
    slice: &S,
) -> Result<Vec<T>, CollisionGpuError> {
    stream
        .clone_dtoh(slice)
        .map_err(|error| CollisionGpuError::Launch(format!("readback: {error:?}")))
}

fn truncated<T>(mut values: Vec<T>, count: usize) -> Vec<T> {
    if count < values.len() {
        values.truncate(count);
    }
    values
}
