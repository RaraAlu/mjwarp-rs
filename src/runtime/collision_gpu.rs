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
mod buffers;
mod context;
mod error;
mod inputs_from_snapshot;
mod product_entry;
mod source;
mod types;

#[cfg(test)]
mod abi_tests;
#[cfg(test)]
mod fallback_tests;
#[cfg(test)]
mod multicontact_tests;
#[cfg(test)]
mod nvrtc_smoke;
#[cfg(test)]
mod primitive_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod validate_tests;

use std::sync::Arc;

use cudarc::driver::{
    CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::{CompileOptions, Ptx, compile_ptx_with_opts};

use super::presence;
use abi::{CollisionArgs, CollisionContactOut, MjwarpMeshPolyArgs, validate_inputs};
use buffers::{input_pointer, output_pointer, readback, truncated, upload, zeroed};
pub use error::CollisionGpuError;
pub use inputs_from_snapshot::{
    CollisionInputsSnapshot, inputs_from_snapshot, mesh_poly_sidecar_present,
};
pub use product_entry::{
    ProductCollisionOutcome, ProductCollisionRequest, ProductHostReason, run_product_collision,
};
pub use types::{CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput};

/// 仅测试入口：对 32 项宿主输入做纯 CPU 校验（不建 CUDA context、无行为变更）。
///
/// `CollisionGpuProgram::run` 在首次分配前同样调用 `abi::validate_inputs`；本函数把同一
/// 校验以 `#[doc(hidden)]` 暴露给集成测试，供 M0b/M2 的 CPU 自检门常驻调用（替代临时探针）。
/// 语义与 `run()` 的校验完全一致，不改任何发射行为。
#[doc(hidden)]
pub fn validate_collision_inputs(inputs: &CollisionGpuInputs<'_>) -> Result<(), CollisionGpuError> {
    validate_inputs(inputs)
}

/// 产品入口符号（与 `product.cu` 的 `extern "C" __global__` 一一对应）。
const CONTACT_ENTRY: &str = "mjwarp_collision_contact";
const HFIELD_ENTRY: &str = "mjwarp_collision_hfield";
const SDF_ENTRY: &str = "mjwarp_collision_sdf";
const PRIMITIVE_ENTRY: &str = "mjwarp_collision_primitive";
const CONTACT_MULTI_ENTRY: &str = "mjwarp_collision_contact_multi";
const BOXBOX_ENTRY: &str = "mjwarp_collision_boxbox";

/// 产品内核种类：决定 `run` 走哪条设备路径。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionKernel {
    /// `mjwarp_collision_contact`：`eval_ccd_write_contact` 单对路径。
    Contact,
    /// `mjwarp_collision_hfield`：hfield prism CCD 路径。
    Hfield,
    /// `mjwarp_collision_sdf`：SDF 单 pair 子集（每初始点一次下降）。
    Sdf,
    /// `mjwarp_collision_primitive`：primitive narrowphase（SPHERE-BOX / CAPSULE-BOX，
    /// 以及 B1 plane 族 6 组合 PLANE-{SPHERE,CAPSULE,ELLIPSOID,CYLINDER,BOX,MESH}）。
    Primitive,
    /// `mjwarp_collision_contact_multi`：`eval_ccd_write_contact` 的 `use_multiccd=true`
    /// 静态特化（box/mesh/cylinder 组合 <=4 个接触；mesh 分支本片回退单接触）。
    ContactMulti,
    /// `mjwarp_collision_boxbox`：上游 `box_box_wrapper`（NATIVECCD 关闭时 `(BOX,BOX)` 的
    /// primitive dispatch；8 个候选经 `detected` 过滤后写入）。
    BoxBox,
}

mod validate;
pub(crate) use validate::{validate_boxbox_types, validate_primitive_types};

/// 已编译并加载的产品程序（一个上下文、一个模块、六个入口、默认流）。
pub struct CollisionGpuProgram {
    context: Arc<CudaContext>,
    /// 模块由四个 `CudaFunction` 共同持有；字段仅为显式表达生命周期。
    _module: Arc<CudaModule>,
    contact: CudaFunction,
    hfield: CudaFunction,
    sdf: CudaFunction,
    primitive: CudaFunction,
    contact_multi: CudaFunction,
    boxbox: CudaFunction,
    stream: Arc<CudaStream>,
}

impl CollisionGpuProgram {
    /// 编译设备源并在 `device` 上加载六个入口。
    ///
    /// 三态回退（**不 panic**）：NVRTC 缺失 → [`CollisionGpuError::Compile`]；驱动缺失 / 无设备 /
    /// `CudaContext::new` 内部 panic（cudarc `culib()`）→ [`CollisionGpuError::NoDevice`]。
    /// 编译先于设备构造，编译失败时不触碰设备。
    pub fn new(device: usize) -> Result<Self, CollisionGpuError> {
        let ptx = compile_program()?;
        let context = context::device_context(device)?;
        load_program(context, ptx)
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
        if kind == CollisionKernel::Primitive {
            validate_primitive_types(meta.type1, meta.type2)?;
        }
        if kind == CollisionKernel::BoxBox {
            validate_boxbox_types(meta.type1, meta.type2)?;
        }
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
        // M0a sidecar 的 9 路可选输入：全空时 `upload` 给 1 元素零缓冲（不启用），
        // 非空时由 `validate_inputs` 保证 9 项齐全且内部自洽。
        let mesh_poly_present = !inputs.mesh_polynum.is_empty();
        let mesh_polynum = upload(stream, inputs.mesh_polynum)?;
        let mesh_polyadr = upload(stream, inputs.mesh_polyadr)?;
        let mesh_polynormal = upload(stream, inputs.mesh_polynormal)?;
        let mesh_polyvertadr = upload(stream, inputs.mesh_polyvertadr)?;
        let mesh_polyvertnum = upload(stream, inputs.mesh_polyvertnum)?;
        let mesh_polyvert = upload(stream, inputs.mesh_polyvert)?;
        let mesh_polymapadr = upload(stream, inputs.mesh_polymapadr)?;
        let mesh_polymapnum = upload(stream, inputs.mesh_polymapnum)?;
        let mesh_polymap = upload(stream, inputs.mesh_polymap)?;
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

        // sidecar 存储先于 guards 声明：guard 借用它，drop 顺序要求 guards 先于该存储释放。
        let mut sidecar_storage: Option<CudaSlice<MjwarpMeshPolyArgs>> = None;
        let mut guards = Vec::new();
        // M0a：9 数组全空 → `mesh_poly_args = null`（不构造 sidecar，行为与改动前一致）。
        if mesh_poly_present {
            let sidecar = MjwarpMeshPolyArgs {
                mesh_polynum: input_pointer(&mesh_polynum, stream, &mut guards),
                mesh_polyadr: input_pointer(&mesh_polyadr, stream, &mut guards),
                mesh_polynormal: input_pointer(&mesh_polynormal, stream, &mut guards),
                mesh_polyvertadr: input_pointer(&mesh_polyvertadr, stream, &mut guards),
                mesh_polyvertnum: input_pointer(&mesh_polyvertnum, stream, &mut guards),
                mesh_polyvert: input_pointer(&mesh_polyvert, stream, &mut guards),
                mesh_polymapadr: input_pointer(&mesh_polymapadr, stream, &mut guards),
                mesh_polymapnum: input_pointer(&mesh_polymapnum, stream, &mut guards),
                mesh_polymap: input_pointer(&mesh_polymap, stream, &mut guards),
            };
            sidecar_storage = Some(
                stream
                    .clone_htod(&[sidecar])
                    .map_err(|error| CollisionGpuError::Launch(format!("upload: {error:?}")))?,
            );
        }
        let mesh_poly_args: *const MjwarpMeshPolyArgs = match sidecar_storage.as_ref() {
            Some(storage) => input_pointer(storage, stream, &mut guards),
            None => std::ptr::null(),
        };
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
            mesh_poly_args,

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
            CollisionKernel::Primitive => &self.primitive,
            CollisionKernel::ContactMulti => &self.contact_multi,
            CollisionKernel::BoxBox => &self.boxbox,
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
///
/// NVRTC 库存在性预检：缺库返回 [`CollisionGpuError::Compile`]（cudarc 缺库会 panic）。
pub(crate) fn compile_program() -> Result<Ptx, CollisionGpuError> {
    if !presence::nvrtc_library_present() {
        return Err(CollisionGpuError::Compile(
            "NVRTC库缺失（nvrtc 不可加载）".to_owned(),
        ));
    }
    let fingerprint = source::source_sha256();
    compile_source_with(|text, options| {
        compile_ptx_with_opts(text, options)
            .map_err(|error| format!("source_sha256={fingerprint}: {error:?}"))
    })
}

/// 加载模块与六个入口（无 GPU 时不调用）。
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
    let primitive = load(PRIMITIVE_ENTRY)?;
    let contact_multi = load(CONTACT_MULTI_ENTRY)?;
    let boxbox = load(BOXBOX_ENTRY)?;
    let stream = context.default_stream();
    Ok(CollisionGpuProgram {
        context,
        _module: module,
        contact,
        hfield,
        sdf,
        primitive,
        contact_multi,
        boxbox,
        stream,
    })
}

/// 构造 seam：设备结果与 PTX 结果都可注入错误（单测不创建真实 context）。
/// 先判 PTX 错误，保证编译失败时根本不碰设备。
#[cfg(test)]
pub(crate) fn program_from_parts(
    device_result: Result<Arc<CudaContext>, String>,
    ptx: Result<Ptx, CollisionGpuError>,
) -> Result<CollisionGpuProgram, CollisionGpuError> {
    let ptx = ptx?;
    let context = device_result.map_err(CollisionGpuError::NoDevice)?;
    load_program(context, ptx)
}
