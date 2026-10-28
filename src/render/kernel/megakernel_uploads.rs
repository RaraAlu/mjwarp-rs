//! S7b2/S7b3 渲染执行器的设备上传段（R05 拆分①：纯移动自 `megakernel.rs`）。
//!
//! 本文件由 `megakernel.rs` 原 L203-347「纹理数组 + 上传所有设备缓冲」段原样迁出，
//! 仅做归属调整：`self.session` → `inputs.session`、`context` → `inputs.context` 等引用改名，
//! 逻辑、顺序、类型、错误路径与逐槽占位（1 元素占位）**零变化**。
//!
//! 语义边界（与拆分前一致）：
//! - `MEGAKERNEL_SOURCE` 与 CUDA 源拼接不变（本文件不参与内核源码拼接）；
//! - `MEGAKERNEL_SLOTS` 81 槽签名不变；
//! - 未用槽仍用 1 元素占位（渲染 ABI 拒绝零长数组）；
//! - 上传顺序与错误信息保持原样（`LauncherError` 路径不变）。

use crate::model::render::RenderFields;
use crate::render::ImageOutput;
use crate::render::context::{RenderContext, ResolvedContext};
use crate::render::kernel::cast::{MeshDevice, ModelRenderArrays, SceneDevice};
use crate::runtime::launcher::LauncherError;
use crate::runtime::{TransferBuffer, TransferSession};

use crate::render::kernel::launch::Texture2dArray;
use crate::render::kernel::megakernel_arrays::{
    bool_words, flags, floats, ints, pack_blob, reals, span_addr, upload,
};

/// 上传段的外部输入（原 `megakernel.rs::render` 中段前已算好的局部量）。
pub(super) struct UploadInputs<'a> {
    pub(super) session: &'a TransferSession,
    pub(super) fields: &'a RenderFields,
    pub(super) model_arrays: &'a ModelRenderArrays,
    pub(super) resolved: &'a ResolvedContext,
    pub(super) context: &'a RenderContext,
    pub(super) scene: &'a SceneDevice,
    pub(super) mesh: &'a MeshDevice,
    /// `model_arrays.geom_type.len()`（原局部 `ngeom`）。
    pub(super) ngeom: usize,
    /// 每像素 samples（AA 开关，原局部 `nsamples`）。
    pub(super) nsamples: usize,
    pub(super) cam_projection: &'a [i32],
    pub(super) cam_fovy: &'a [f32],
    pub(super) cam_sensorsize: &'a [f32],
    pub(super) cam_intrinsic: &'a [f32],
    pub(super) light_xpos: &'a [f32],
    pub(super) light_xdir: &'a [f32],
    pub(super) geom_xpos: &'a [f32],
    pub(super) geom_xmat: &'a [f32],
    pub(super) cam_xpos: &'a [f32],
    pub(super) cam_xmat: &'a [f32],
    pub(super) cam_res_flat: &'a [i32],
    pub(super) cam_id_map: &'a [i32],
    pub(super) ray_dirs: &'a [f32],
    pub(super) ray_offsets: &'a [f32],
    pub(super) render_rgb: &'a [i32],
    pub(super) render_depth: &'a [i32],
    pub(super) render_seg: &'a [i32],
    pub(super) shading_constants: &'a [f32],
    pub(super) scale_constants: &'a [f32],
}

/// 上传完成的设备缓冲句柄与少量推导计数（原 `megakernel.rs` 局部变量集）。
pub(super) struct MegakernelUploads {
    pub(super) texture_array: Option<Texture2dArray>,
    /// BVH blob 设备缓冲：槽 56 只取 `blob_addr` 传地址，本字段仅靠结构体存活
    /// 保持设备内存到内核发射（原 `megakernel.rs` 局部变量同职责）。
    #[allow(dead_code)]
    pub(super) blob_buffer: TransferBuffer<u32>,
    pub(super) blob_addr: u64,
    pub(super) geom_type: TransferBuffer<i32>,
    pub(super) geom_dataid: TransferBuffer<i32>,
    pub(super) geom_matid: TransferBuffer<i32>,
    pub(super) geom_size: TransferBuffer<f32>,
    pub(super) geom_rgba: TransferBuffer<f32>,
    pub(super) cam_projection: TransferBuffer<i32>,
    pub(super) cam_fovy: TransferBuffer<f32>,
    pub(super) cam_sensorsize: TransferBuffer<f32>,
    pub(super) cam_intrinsic: TransferBuffer<f32>,
    pub(super) light_type: TransferBuffer<i32>,
    pub(super) nlight: usize,
    pub(super) light_castshadow: TransferBuffer<u32>,
    pub(super) light_active: TransferBuffer<u32>,
    pub(super) light_attenuation: TransferBuffer<f32>,
    pub(super) light_cutoff: TransferBuffer<f32>,
    pub(super) light_exponent: TransferBuffer<f32>,
    pub(super) light_ambient: TransferBuffer<f32>,
    pub(super) light_diffuse: TransferBuffer<f32>,
    pub(super) light_specular: TransferBuffer<f32>,
    pub(super) light_xpos: TransferBuffer<f32>,
    pub(super) light_xdir: TransferBuffer<f32>,
    pub(super) flex_vertadr: TransferBuffer<i32>,
    pub(super) flex_edge: TransferBuffer<i32>,
    pub(super) flex_radius: TransferBuffer<f32>,
    pub(super) flexvert_xpos: TransferBuffer<f32>,
    pub(super) flex_bvh_id: TransferBuffer<u32>,
    pub(super) flex_group_root: TransferBuffer<i32>,
    pub(super) flex_rgba: TransferBuffer<f32>,
    pub(super) flex_geom_flexid: TransferBuffer<i32>,
    pub(super) flex_geom_edgeid: TransferBuffer<i32>,
    pub(super) hfield_bvh_id: TransferBuffer<u32>,
    pub(super) mesh_bvh_id: TransferBuffer<u32>,
    pub(super) group_root: TransferBuffer<i32>,
    pub(super) skybox_tex_id: TransferBuffer<i32>,
    pub(super) skybox_face_width: TransferBuffer<i32>,
    pub(super) splat_rotation: TransferBuffer<f32>,
    pub(super) splat_rgba: TransferBuffer<f32>,
    pub(super) splat_group_root: TransferBuffer<i32>,
    pub(super) dummy_texture: TransferBuffer<u32>,
    pub(super) mesh_faceadr: TransferBuffer<i32>,
    pub(super) nmesh: usize,
    pub(super) mesh_normaladr: TransferBuffer<i32>,
    pub(super) mesh_normal: TransferBuffer<f32>,
    pub(super) mesh_facetexcoord: TransferBuffer<i32>,
    pub(super) mesh_faces: usize,
    pub(super) mesh_facenormal: TransferBuffer<i32>,
    pub(super) mesh_texcoord: TransferBuffer<f32>,
    pub(super) mesh_texcoord_offsets: TransferBuffer<i32>,
    pub(super) mat_texid: TransferBuffer<i32>,
    pub(super) nmat: usize,
    pub(super) mat_texuniform: TransferBuffer<u32>,
    pub(super) mat_texrepeat: TransferBuffer<f32>,
    pub(super) mat_emission: TransferBuffer<f32>,
    pub(super) mat_specular: TransferBuffer<f32>,
    pub(super) mat_shininess: TransferBuffer<f32>,
    pub(super) mat_rgba: TransferBuffer<f32>,
    pub(super) geom_xpos: TransferBuffer<f32>,
    pub(super) geom_xmat: TransferBuffer<f32>,
    pub(super) cam_xpos: TransferBuffer<f32>,
    pub(super) cam_xmat: TransferBuffer<f32>,
    pub(super) cam_res: TransferBuffer<i32>,
    pub(super) cam_id_map: TransferBuffer<i32>,
    pub(super) ray: TransferBuffer<f32>,
    pub(super) ray_offset: TransferBuffer<f32>,
    pub(super) rgb_adr: TransferBuffer<i32>,
    pub(super) depth_adr: TransferBuffer<i32>,
    pub(super) seg_adr: TransferBuffer<i32>,
    pub(super) render_rgb: TransferBuffer<u32>,
    pub(super) render_depth: TransferBuffer<u32>,
    pub(super) render_seg: TransferBuffer<u32>,
    pub(super) enabled_geom_ids: TransferBuffer<i32>,
    pub(super) shading_constants: TransferBuffer<f32>,
    pub(super) scale_constants: TransferBuffer<f32>,
    pub(super) aa_scratch: Option<TransferBuffer<f32>>,
    pub(super) rgb_scratch: Option<TransferBuffer<u32>>,
    pub(super) depth_scratch: Option<TransferBuffer<f32>>,
    pub(super) seg_scratch: Option<TransferBuffer<i32>>,
}

/// 上传纹理数组与全部设备缓冲（纯移动自 `megakernel.rs` 原 L203-347）。
pub(super) fn upload_megakernel_arrays(
    inputs: &UploadInputs<'_>,
) -> Result<MegakernelUploads, LauncherError> {
    let context = inputs.context;
    let resolved = inputs.resolved;
    let fields = inputs.fields;
    let model_arrays = inputs.model_arrays;
    let session = inputs.session;
    let scene = inputs.scene;
    let mesh = inputs.mesh;
    let nsamples = inputs.nsamples;

    // ---- 纹理数组（S4 只读句柄 → S7a `texture2d_t` 数组）----
    let texture_array = match context.textures() {
        Some(resources) if resolved.use_textures => {
            let handles = resources.handles();
            if handles.is_empty() {
                return Err(LauncherError::InvalidArray("textures"));
            }
            Some(Texture2dArray::upload(session, &handles)?)
        }
        _ => None,
    };

    // ---- 上传所有设备缓冲 ----
    let blob = pack_blob(scene, mesh);
    let blob_buffer = upload(session, &blob)?;
    let blob_addr = span_addr(&blob_buffer)?;

    let geom_type = upload(session, &model_arrays.geom_type)?;
    let geom_dataid = ints(fields, "geom_dataid")?;
    let geom_dataid = upload(session, &geom_dataid)?;
    let geom_matid = upload(session, &model_arrays.geom_matid)?;
    let geom_size = reals(fields, "geom_size")?;
    let geom_size = upload(session, &geom_size)?;
    let geom_rgba = upload(session, &model_arrays.geom_rgba)?;
    let cam_projection = upload(session, inputs.cam_projection)?;
    let cam_fovy = upload(session, inputs.cam_fovy)?;
    let cam_sensorsize = upload(session, inputs.cam_sensorsize)?;
    let cam_intrinsic = upload(session, inputs.cam_intrinsic)?;

    let light_type = ints(fields, "light_type")?;
    let nlight = light_type.len();
    let light_type = upload(session, &light_type)?;
    let light_castshadow = bool_words(&flags(fields, "light_castshadow")?);
    let light_castshadow = upload(session, &light_castshadow)?;
    let light_active = bool_words(&flags(fields, "light_active")?);
    let light_active = upload(session, &light_active)?;
    let light_attenuation = upload(session, &floats(fields, "light_attenuation")?)?;
    let light_cutoff = upload(session, &floats(fields, "light_cutoff")?)?;
    let light_exponent = upload(session, &floats(fields, "light_exponent")?)?;
    let light_ambient = upload(session, &floats(fields, "light_ambient")?)?;
    let light_diffuse = upload(session, &floats(fields, "light_diffuse")?)?;
    let light_specular = upload(session, &floats(fields, "light_specular")?)?;
    let light_xpos = upload(session, inputs.light_xpos)?;
    let light_xdir = upload(session, inputs.light_xdir)?;

    let flex_vertadr = upload(session, &[0i32])?;
    let flex_edge = upload(session, &[0i32, 0])?;
    let flex_radius = upload(session, &[0.0f32])?;
    let flexvert_xpos = upload(session, &[0.0f32; 3])?;
    let flex_bvh_id = upload(session, &[0u32, 0])?;
    let flex_group_root = upload(session, &[0i32])?;
    let flex_rgba = upload(session, &[0.0f32; 4])?;
    let flex_geom_flexid = upload(session, &[0i32])?;
    let flex_geom_edgeid = upload(session, &[0i32])?;
    let hfield_bvh_id = upload(session, &[0u32, 0])?;
    let mesh_bvh_id = upload(session, &[0u32, 0])?;
    let group_root = upload(session, &[0i32])?;
    let skybox_tex_id = upload(session, &[0i32])?;
    let skybox_face_width = upload(session, &[0i32])?;
    let splat_rotation = upload(session, &[0.0f32; 4])?;
    let splat_rgba = upload(session, &[0.0f32; 4])?;
    let splat_group_root = upload(session, &[0i32])?;
    let dummy_texture = upload(session, &[0u32; 6])?;

    let mesh_faceadr = ints(fields, "mesh_faceadr")?;
    let nmesh = mesh_faceadr.len();
    let mesh_faceadr = upload(session, &mesh_faceadr)?;
    let mesh_normaladr = ints(fields, "mesh_normaladr")?;
    let mesh_normaladr = upload(session, &mesh_normaladr)?;
    let mesh_normal = floats(fields, "mesh_normal")?;
    let mesh_normal = upload(session, &mesh_normal)?;
    let mesh_facetexcoord = ints(fields, "mesh_facetexcoord")?;
    let mesh_faces = mesh_facetexcoord.len() / 3;
    let mesh_facetexcoord = upload(session, &mesh_facetexcoord)?;
    let mesh_facenormal = ints(fields, "mesh_facenormal")?;
    let mesh_facenormal = upload(session, &mesh_facenormal)?;
    // scene-a 的网格没有 texcoord（`mesh_texcoordadr=-1`）：上传 2 个 0 占位（ABI 拒绝零长数组）。
    let mesh_texcoord = {
        let values = floats(fields, "mesh_texcoord")?;
        if values.is_empty() {
            vec![0.0f32; 2]
        } else {
            values
        }
    };
    let mesh_texcoord = upload(session, &mesh_texcoord)?;
    let mesh_texcoord_offsets = ints(fields, "mesh_texcoordadr")?;
    let mesh_texcoord_offsets = upload(session, &mesh_texcoord_offsets)?;

    let mat_texid = ints(fields, "mat_texid")?;
    let nmat = mat_texid.len() / 10;
    let mat_texid = upload(session, &mat_texid)?;
    let mat_texuniform = bool_words(&flags(fields, "mat_texuniform")?);
    let mat_texuniform = upload(session, &mat_texuniform)?;
    let mat_texrepeat = upload(session, &floats(fields, "mat_texrepeat")?)?;
    let mat_emission = upload(session, &floats(fields, "mat_emission")?)?;
    let mat_specular = upload(session, &floats(fields, "mat_specular")?)?;
    let mat_shininess = upload(session, &floats(fields, "mat_shininess")?)?;
    let mat_rgba = upload(session, &model_arrays.mat_rgba)?;

    let geom_xpos = upload(session, inputs.geom_xpos)?;
    let geom_xmat = upload(session, inputs.geom_xmat)?;
    let cam_xpos = upload(session, inputs.cam_xpos)?;
    let cam_xmat = upload(session, inputs.cam_xmat)?;

    let cam_res = upload(session, inputs.cam_res_flat)?;
    let cam_id_map = upload(session, inputs.cam_id_map)?;
    let ray = upload(session, inputs.ray_dirs)?;
    let ray_offset = upload(session, inputs.ray_offsets)?;
    let rgb_adr = upload(session, &resolved.output.rgb_adr)?;
    let depth_adr = upload(session, &resolved.output.depth_adr)?;
    let seg_adr = upload(session, &resolved.output.seg_adr)?;
    let render_rgb = upload(session, &bool_words(inputs.render_rgb))?;
    let render_depth = upload(session, &bool_words(inputs.render_depth))?;
    let render_seg = upload(session, &bool_words(inputs.render_seg))?;
    let enabled_geom_ids = match resolved.enabled_geom_ids.is_empty() {
        true => (0..inputs.ngeom as i32).collect::<Vec<i32>>(),
        false => resolved.enabled_geom_ids.clone(),
    };
    let enabled_geom_ids = upload(session, &enabled_geom_ids)?;
    let shading_constants = upload(session, inputs.shading_constants)?;
    let scale_constants = upload(session, inputs.scale_constants)?;
    // AA（nsamples>1）时 aa_accum 由 OutputResources 分配（world × rgb_pixels vec3），
    // 否则 1 元素占位（内核按 AA 标志不访问）。
    let aa_scratch = match nsamples > 1 {
        true => None,
        false => Some(upload(session, &[0.0f32; 3])?),
    };

    // 输出缓冲（S6b OutputResources；未启用时 1 元素占位、内核按标志不写）。
    let rgb_scratch = match context.outputs().is_enabled(ImageOutput::Rgb) {
        true => None,
        false => Some(upload(session, &[0u32])?),
    };
    let depth_scratch = match context.outputs().is_enabled(ImageOutput::Depth) {
        true => None,
        false => Some(upload(session, &[0.0f32])?),
    };
    let seg_scratch = match context.outputs().is_enabled(ImageOutput::Segmentation) {
        true => None,
        false => Some(upload(session, &[0i32; 2])?),
    };

    Ok(MegakernelUploads {
        texture_array,
        blob_buffer,
        blob_addr,
        geom_type,
        geom_dataid,
        geom_matid,
        geom_size,
        geom_rgba,
        cam_projection,
        cam_fovy,
        cam_sensorsize,
        cam_intrinsic,
        light_type,
        nlight,
        light_castshadow,
        light_active,
        light_attenuation,
        light_cutoff,
        light_exponent,
        light_ambient,
        light_diffuse,
        light_specular,
        light_xpos,
        light_xdir,
        flex_vertadr,
        flex_edge,
        flex_radius,
        flexvert_xpos,
        flex_bvh_id,
        flex_group_root,
        flex_rgba,
        flex_geom_flexid,
        flex_geom_edgeid,
        hfield_bvh_id,
        mesh_bvh_id,
        group_root,
        skybox_tex_id,
        skybox_face_width,
        splat_rotation,
        splat_rgba,
        splat_group_root,
        dummy_texture,
        mesh_faceadr,
        nmesh,
        mesh_normaladr,
        mesh_normal,
        mesh_facetexcoord,
        mesh_faces,
        mesh_facenormal,
        mesh_texcoord,
        mesh_texcoord_offsets,
        mat_texid,
        nmat,
        mat_texuniform,
        mat_texrepeat,
        mat_emission,
        mat_specular,
        mat_shininess,
        mat_rgba,
        geom_xpos,
        geom_xmat,
        cam_xpos,
        cam_xmat,
        cam_res,
        cam_id_map,
        ray,
        ray_offset,
        rgb_adr,
        depth_adr,
        seg_adr,
        render_rgb,
        render_depth,
        render_seg,
        enabled_geom_ids,
        shading_constants,
        scale_constants,
        aa_scratch,
        rgb_scratch,
        depth_scratch,
        seg_scratch,
    })
}
