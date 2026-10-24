//! 渲染内核的编译、加载与发射（渲染/BVH S7a）。
//!
//! 编译路径唯一：`NvrtcSource`（W15 头树）→ `WARP_1_15_RENDER` → `check_render_ptx`
//! → 模块缓存 → 发射。没有绕过渲染门禁的公开编译入口；像素内核属 S7b。
//!
//! 纹理：`Texture2dArray` 把 S4 的只读 `CUtexObject` 句柄编码成 `wp.texture2d_t`
//! 设备数组（24 字节元素、align 8），Drop 时释放设备数组；句柄所有权仍在 S4 的
//! `TextureObjects`，本模块不销毁纹理对象。

use std::path::PathBuf;
use std::sync::Arc;

use crate::render::texture::cuda::TextureHandle2d;
use crate::runtime::launcher::{
    Artifact, ArtifactCache, CompileRecipe, KernelHandle, LaunchDims, LauncherError, ModuleCache,
    NvrtcSource, RecipePurpose, RenderArg, RenderArrayValue, RenderElem, RenderPlan,
    RenderSignature, Texture2d,
};
use crate::runtime::{TransferBuffer, TransferSession};

use super::check_render_ptx;

/// W15 头树的仓库相对路径（原样复制，内容进入产物键）。
pub const W15_HEADERS_DIR: &str = "include/warp-headers";

fn headers_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(W15_HEADERS_DIR)
}

/// 只编译并通过渲染精度门禁（不加载模块、不需要 GPU）。
///
/// 这是渲染内核唯一的编译入口；[`RenderKernel::compile`] 也经由它。
pub fn compile_render_artifact(
    name: &'static str,
    source: &'static str,
    entry: &'static str,
) -> Result<Arc<Artifact>, LauncherError> {
    let source = NvrtcSource::new(name, source)?.with_headers_from_dir(&headers_root())?;
    let cache = ArtifactCache::in_memory();
    let artifact = cache.compile(
        &source,
        &CompileRecipe::WARP_1_15_RENDER,
        RecipePurpose::Render,
    )?;
    check_render_ptx(artifact.ptx(), entry)
        .map_err(|error| LauncherError::Precision(format!("render-ptx: {error}")))?;
    Ok(artifact)
}

/// 已编译并加载入口的渲染内核。
pub struct RenderKernel {
    cache: ModuleCache,
    handle: KernelHandle,
    artifact: Arc<Artifact>,
    signature: RenderSignature,
    entry: &'static str,
}

impl RenderKernel {
    /// 编译 + 渲染门禁 + 加载入口（需要 NVIDIA 驱动与 NVRTC）。
    pub fn compile(
        session: &TransferSession,
        name: &'static str,
        source: &'static str,
        signature: RenderSignature,
        entry: &'static str,
    ) -> Result<Self, LauncherError> {
        let artifact = compile_render_artifact(name, source, entry)?;
        let context = Arc::clone(session.stream_ref().context());
        let cache = ModuleCache::new(context);
        let handle = cache.function(cache.context(), &artifact, entry)?;
        Ok(Self {
            cache,
            handle,
            artifact,
            signature,
            entry,
        })
    }

    pub fn artifact(&self) -> &Arc<Artifact> {
        &self.artifact
    }

    pub fn signature(&self) -> RenderSignature {
        self.signature
    }

    pub fn entry(&self) -> &'static str {
        self.entry
    }

    /// 模块缓存统计（模块/入口加载次数与命中）。
    pub fn cache(&self) -> &ModuleCache {
        &self.cache
    }

    /// 按签名校验后同步发射；返回前等待完成并检查驱动错误。
    pub fn launch(
        &self,
        session: &TransferSession,
        args: &[RenderArg],
        dims: LaunchDims,
    ) -> Result<(), LauncherError> {
        let plan = RenderPlan::plan(self.signature, args, dims)?;
        plan.launch(&self.handle, session, args)
    }
}

/// `wp.texture2d_t` 的设备数组：24 字节元素、align 8。
///
/// 会话缓冲只支持 f32/f64/i32/u32，因此按 6 个 u32 打包每个 24 字节元素
/// （本机序与 `texture2d_t` 的 C++ 布局逐字节一致）。句柄所有权仍属 S4
/// `TextureObjects`；本类型只持有设备数组并在 Drop 时释放。
pub struct Texture2dArray {
    buffer: TransferBuffer<u32>,
    len: usize,
}

/// 每个 `texture2d_t` 的 u32 字数（24 B / 4）。
const WORDS_PER_TEXTURE: usize = Texture2d::BYTES / 4;

fn encode_texture(texture: Texture2d) -> [u32; WORDS_PER_TEXTURE] {
    [
        texture.tex as u32,
        (texture.tex >> 32) as u32,
        texture.width as u32,
        texture.height as u32,
        texture.num_channels as u32,
        0,
    ]
}

fn decode_texture(words: &[u32; WORDS_PER_TEXTURE]) -> Texture2d {
    Texture2d {
        tex: u64::from(words[0]) | (u64::from(words[1]) << 32),
        width: words[2] as i32,
        height: words[3] as i32,
        num_channels: words[4] as i32,
    }
}

impl Texture2dArray {
    /// 把只读句柄编码成 `texture2d_t` 数组；空列表现阶段拒绝（上游纹理数组至少 1 项）。
    pub fn upload(
        session: &TransferSession,
        handles: &[TextureHandle2d],
    ) -> Result<Self, LauncherError> {
        if handles.is_empty() {
            return Err(LauncherError::InvalidArray("纹理句柄数组不能为空"));
        }
        let words = handles
            .len()
            .checked_mul(WORDS_PER_TEXTURE)
            .ok_or(LauncherError::Overflow("texture2d_array_words"))?;
        let mut encoded = Vec::with_capacity(words);
        for handle in handles {
            encoded.extend_from_slice(&encode_texture(Texture2d {
                tex: handle.tex,
                width: i32::try_from(handle.width)
                    .map_err(|_| LauncherError::Overflow("texture_width"))?,
                height: i32::try_from(handle.height)
                    .map_err(|_| LauncherError::Overflow("texture_height"))?,
                num_channels: i32::try_from(handle.channels)
                    .map_err(|_| LauncherError::Overflow("texture_channels"))?,
            }));
        }
        let buffer = session
            .upload(&encoded)
            .map_err(|error| LauncherError::Driver(format!("texture2d-array: {error}")))?;
        Ok(Self {
            buffer,
            len: handles.len(),
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 设备数组实参（元素 `texture2d_t`、形状 `[len]`）。
    pub fn render_arg(&self) -> Result<RenderArrayValue, LauncherError> {
        let span = self
            .buffer
            .launcher_span()
            .map_err(|error| LauncherError::Driver(format!("texture2d-span: {error}")))?;
        RenderArrayValue::contiguous(span, RenderElem::Texture2d, &[self.len])
    }

    /// 回读第 `index` 个元素（用于 round-trip 校验）。
    pub fn read(&self, index: usize) -> Result<Texture2d, LauncherError> {
        if index >= self.len {
            return Err(LauncherError::InvalidSpan("纹理数组下标越界"));
        }
        let mut words = [0u32; WORDS_PER_TEXTURE];
        self.buffer
            .read_range_into(index * WORDS_PER_TEXTURE, &mut words)
            .map_err(|error| LauncherError::Driver(format!("texture2d-read: {error}")))?;
        Ok(decode_texture(&words))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::device_memory;
    use crate::render::texture::cuda::TextureObjects;
    use crate::render::texture::{TextureData, TextureRegistry};
    use crate::runtime::TransferElement;
    use crate::runtime::launcher::{
        BlockRule, LaunchBounds2, RenderArgKind, RenderScalar, RenderScalarKind, RenderSignature,
    };
    /// ABI round-trip 探针：逐类读实参并把位型写到输出（只有单线程写入）。
    const PROBE_SOURCE: &str = r#"
#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"
#include "bvh.h"
#include "texture.h"
extern "C" __global__ void s7a_abi_probe(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> a_i32, wp::array_t<wp::uint32> a_u32,
    wp::array_t<wp::uint64> a_u64, wp::array_t<bool> a_bool,
    wp::array_t<float> a_f32, wp::array_t<wp::vec2> a_vec2,
    wp::array_t<wp::vec3> a_vec3, wp::array_t<wp::vec4> a_vec4,    wp::array_t<wp::vec2i> a_vec2i, wp::array_t<wp::vec3i> a_vec3i,
    wp::array_t<wp::mat33> a_mat33, wp::array_t<wp::quatf> a_quat,
    wp::array_t<wp::texture2d_t> a_tex,
    wp::int32 s_i32, wp::uint64 s_u64, bool s_bool,
    wp::array_t<wp::uint32> out_u32, wp::array_t<wp::uint64> out_u64,
    wp::array_t<wp::vec2i> out_vec2i, wp::array_t<float> out_f32)
{
    if (dim.size == 0 || blockIdx.x != 0 || threadIdx.x != 0) { return; }
    out_u32[0] = (wp::uint32)a_i32[0]; out_u32[1] = (wp::uint32)a_i32[3];
    out_u32[2] = a_u32[0];
    out_u32[3] = (wp::uint32)(a_u64[0] & 0xffffffffull);
    out_u32[4] = (wp::uint32)(a_u64[0] >> 32);
    out_u32[5] = (wp::uint32)a_bool[0] | ((wp::uint32)a_bool[1] << 1)
        | ((wp::uint32)a_bool[2] << 2) | ((wp::uint32)a_bool[3] << 3);
    out_u32[6] = __float_as_uint(a_f32[0]); out_u32[7] = __float_as_uint(a_f32[2]);
    out_u32[8] = __float_as_uint(a_vec2[1][0]); out_u32[9] = __float_as_uint(a_vec2[1][1]);
    out_u32[10] = __float_as_uint(a_vec3[0][0]); out_u32[11] = __float_as_uint(a_vec3[1][2]);
    out_u32[12] = __float_as_uint(a_vec4[0][0]); out_u32[13] = __float_as_uint(a_vec4[0][3]);
    out_u32[14] = (wp::uint32)a_vec2i[1][0]; out_u32[15] = (wp::uint32)a_vec2i[1][1];
    out_u32[16] = (wp::uint32)a_vec3i[0][2];
    out_u32[17] = __float_as_uint(a_mat33[0].data[0][0]);
    out_u32[18] = __float_as_uint(a_mat33[0].data[1][2]);
    out_u32[19] = __float_as_uint(a_quat[0][2]);
    wp::texture2d_t tex = a_tex[0];
    out_u32[20] = (wp::uint32)(tex.tex & 0xffffffffull);
    out_u32[21] = (wp::uint32)(tex.tex >> 32);
    out_u32[22] = (wp::uint32)tex.width; out_u32[23] = (wp::uint32)tex.height;
    out_u32[24] = (wp::uint32)tex.num_channels;
    out_u32[25] = (wp::uint32)s_i32; out_u32[26] = (wp::uint32)s_bool;
    out_u64[0] = s_u64; out_u64[1] = a_u64[0];
    out_vec2i[0] = a_vec2i[0];
    out_f32[0] = a_f32[1]; out_f32[1] = a_vec3[0][1];
}
"#;

    const PROBE_ENTRY: &str = "s7a_abi_probe";

    static PROBE_SLOTS: [RenderArgKind; 21] = [
        RenderArgKind::Bounds2D,
        RenderArgKind::Array(RenderElem::I32),
        RenderArgKind::Array(RenderElem::U32),
        RenderArgKind::Array(RenderElem::U64),
        RenderArgKind::Array(RenderElem::Bool),
        RenderArgKind::Array(RenderElem::F32),
        RenderArgKind::Array(RenderElem::Vec2),
        RenderArgKind::Array(RenderElem::Vec3),
        RenderArgKind::Array(RenderElem::Vec4),
        RenderArgKind::Array(RenderElem::Vec2I),
        RenderArgKind::Array(RenderElem::Vec3I),
        RenderArgKind::Array(RenderElem::Mat33),
        RenderArgKind::Array(RenderElem::Quat),
        RenderArgKind::Array(RenderElem::Texture2d),
        RenderArgKind::Scalar(RenderScalarKind::I32),
        RenderArgKind::Scalar(RenderScalarKind::U64),
        RenderArgKind::Scalar(RenderScalarKind::Bool),
        RenderArgKind::Array(RenderElem::U32),
        RenderArgKind::Array(RenderElem::U64),
        RenderArgKind::Array(RenderElem::Vec2I),
        RenderArgKind::Array(RenderElem::F32),
    ];

    fn session() -> TransferSession {
        TransferSession::new(0).expect("S7a GPU 验收需要 NVIDIA 驱动")
    }

    fn array_arg<T: TransferElement>(
        buffer: &TransferBuffer<T>,
        elem: RenderElem,
        shape: &[usize],
    ) -> RenderArg {
        let span = buffer.launcher_span().expect("缓冲设备区间");
        RenderArg::Array(RenderArrayValue::contiguous(span, elem, shape).expect("数组实参"))
    }

    fn warp_grid() -> LaunchDims {
        LaunchDims::new((1, 1, 1), (256, 1, 1), 0).expect("发射维度")
    }
    fn identity_texture_objects(session: &TransferSession) -> TextureObjects {
        let registry = TextureRegistry::from_textures(vec![
            TextureData::from_raw(
                2,
                2,
                4,
                0,
                &[
                    0, 64, 128, 255, 255, 128, 64, 0, 16, 32, 48, 64, 80, 96, 112, 128,
                ],
            )
            .expect("2x2 纹理"),
        ]);
        TextureObjects::new(session, &registry).expect("纹理对象")
    }

    /// 全参数类 round-trip：发射后逐字段比对（GPU）。
    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn round_trips_every_render_argument_class() {
        let session = session();
        let i32_values = [11i32, -12, 13, -14];
        let u32_values = [0x1122_3344u32];
        let u64_values = [0x0102_0304_0506_0708u64];
        let bool_bytes = [1u8, 0, 1, 0];
        let bool_word = i32::from_ne_bytes(bool_bytes);
        let f32_values = [1.5f32, -2.25, 0.0, 3.75];
        let vec2_values = [0.5f32, 1.5, 2.5, 3.5];
        let vec3_values = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let vec4_values = [0.25f32, 0.5, 0.75, 1.0];
        let vec2i_values = [1i32, 2, 3, 4];
        let vec3i_values = [5i32, 6, 7];
        let mat33_values = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let quat_values = [0.1f32, 0.2, 0.3, 0.4];

        let i32_buffer = session.upload(&i32_values).unwrap();
        let u32_buffer = session.upload(&u32_values).unwrap();
        // 会话缓冲只支持 f32/f64/i32/u32：u64 按两个 u32 字上传（本机序即 u64 位型）。
        let u64_words = [u64_values[0] as u32, (u64_values[0] >> 32) as u32];
        let u64_buffer = session.upload(&u64_words).unwrap();
        let bool_buffer = session.upload(&[bool_word]).unwrap();
        let f32_buffer = session.upload(&f32_values).unwrap();
        let vec2_buffer = session.upload(&vec2_values).unwrap();
        let vec3_buffer = session.upload(&vec3_values).unwrap();
        let vec4_buffer = session.upload(&vec4_values).unwrap();
        let vec2i_buffer = session.upload(&vec2i_values).unwrap();
        let vec3i_buffer = session.upload(&vec3i_values).unwrap();
        let mat33_buffer = session.upload(&mat33_values).unwrap();
        let quat_buffer = session.upload(&quat_values).unwrap();
        let textures = identity_texture_objects(&session);
        let handle = textures.handle(0).expect("纹理句柄");
        let texture_array = Texture2dArray::upload(&session, &[handle]).expect("纹理数组");

        let out_u32 = session.upload(&[0u32; 32]).unwrap();
        let out_u64 = session.upload(&[0u32; 4]).unwrap();
        let out_vec2i = session.upload(&[0i32; 2]).unwrap();
        let out_f32 = session.upload(&[0f32; 2]).unwrap();

        let args = vec![
            RenderArg::Bounds2D(LaunchBounds2::new([1, 1]).expect("边界")),
            array_arg(&i32_buffer, RenderElem::I32, &[4]),
            array_arg(&u32_buffer, RenderElem::U32, &[1]),
            array_arg(&u64_buffer, RenderElem::U64, &[1]),
            array_arg(&bool_buffer, RenderElem::Bool, &[4]),
            array_arg(&f32_buffer, RenderElem::F32, &[4]),
            array_arg(&vec2_buffer, RenderElem::Vec2, &[2]),
            array_arg(&vec3_buffer, RenderElem::Vec3, &[2]),
            array_arg(&vec4_buffer, RenderElem::Vec4, &[1]),
            array_arg(&vec2i_buffer, RenderElem::Vec2I, &[2]),
            array_arg(&vec3i_buffer, RenderElem::Vec3I, &[1]),
            array_arg(&mat33_buffer, RenderElem::Mat33, &[1]),
            array_arg(&quat_buffer, RenderElem::Quat, &[1]),
            RenderArg::Array(texture_array.render_arg().expect("纹理实参")),
            RenderArg::Scalar(RenderScalar::I32(7)),
            RenderArg::Scalar(RenderScalar::U64(0x1122_3344_5566_7788)),
            RenderArg::Scalar(RenderScalar::Bool(true)),
            array_arg(&out_u32, RenderElem::U32, &[32]),
            array_arg(&out_u64, RenderElem::U64, &[2]),
            array_arg(&out_vec2i, RenderElem::Vec2I, &[1]),
            array_arg(&out_f32, RenderElem::F32, &[2]),
        ];

        let signature =
            RenderSignature::new(PROBE_ENTRY, BlockRule::WarpGrid, &PROBE_SLOTS).expect("签名");
        let kernel =
            RenderKernel::compile(&session, PROBE_ENTRY, PROBE_SOURCE, signature, PROBE_ENTRY)
                .expect("渲染内核编译");
        let ptx = kernel.artifact().ptx();
        assert_eq!(check_render_ptx(ptx, PROBE_ENTRY), Ok(()));
        assert_eq!(
            ptx.matches(".global .align 8 .f64").count(),
            2,
            "只允许 W15 两条常量"
        );
        kernel.launch(&session, &args, warp_grid()).expect("发射");
        // 第二次发射复用同一模块（模块缓存命中）。
        kernel
            .launch(&session, &args, warp_grid())
            .expect("重复发射");
        assert_eq!(kernel.cache().stats().module_loads, 1);

        let mut got_u32 = vec![0u32; 32];
        out_u32.read_range_into(0, &mut got_u32).unwrap();
        let mut got_u64_words = [0u32; 4];
        out_u64.read_range_into(0, &mut got_u64_words).unwrap();
        let got_u64 = [
            u64::from(got_u64_words[0]) | (u64::from(got_u64_words[1]) << 32),
            u64::from(got_u64_words[2]) | (u64::from(got_u64_words[3]) << 32),
        ];
        let mut got_vec2i = vec![0i32; 2];
        out_vec2i.read_range_into(0, &mut got_vec2i).unwrap();
        let mut got_f32 = [0f32; 2];
        out_f32.read_range_into(0, &mut got_f32).unwrap();

        let expected_u32 = [
            i32_values[0] as u32,
            i32_values[3] as u32,
            u32_values[0],
            u64_values[0] as u32,
            (u64_values[0] >> 32) as u32,
            0b0101,
            f32_values[0].to_bits(),
            f32_values[2].to_bits(),
            vec2_values[2].to_bits(),
            vec2_values[3].to_bits(),
            vec3_values[0].to_bits(),
            vec3_values[5].to_bits(),
            vec4_values[0].to_bits(),
            vec4_values[3].to_bits(),
            vec2i_values[2] as u32,
            vec2i_values[3] as u32,
            vec3i_values[2] as u32,
            mat33_values[0].to_bits(),
            mat33_values[5].to_bits(),
            quat_values[2].to_bits(),
            (handle.tex & 0xFFFF_FFFF) as u32,
            (handle.tex >> 32) as u32,
            handle.width,
            handle.height,
            handle.channels,
            7,
            1,
        ];
        assert_eq!(&got_u32[..expected_u32.len()], &expected_u32);
        assert!(
            got_u32[expected_u32.len()..]
                .iter()
                .all(|value| *value == 0)
        );
        assert_eq!(got_u64, [0x1122_3344_5566_7788, u64_values[0]]);
        assert_eq!(got_vec2i, [vec2i_values[0], vec2i_values[1]]);
        assert_eq!(got_f32, [f32_values[1], vec3_values[1]]);
        assert_eq!(texture_array.read(0).unwrap().tex, handle.tex);
        assert_eq!(texture_array.read(0).unwrap().width, handle.width as i32);
    }

    /// 纹理句柄存活 + drop 释放设备数组（24 B 级分配落在驱动池内，用重复 create/drop 查泄漏）。
    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn texture_handle_survives_launch_and_drop_releases_device_array() {
        let session = session();
        let textures = identity_texture_objects(&session);
        let handle = textures.handle(0).expect("纹理句柄");
        assert!(textures.handle(textures.len()).is_none());
        let array = Texture2dArray::upload(&session, &[handle]).expect("纹理数组");
        let arg = array.render_arg().expect("纹理实参");
        assert_eq!(arg.elem(), RenderElem::Texture2d);
        assert_eq!(arg.span().bytes(), Texture2d::BYTES as u64);
        let read = array.read(0).expect("回读");
        assert_eq!(
            (read.tex, read.width, read.height, read.num_channels),
            (handle.tex, 2, 2, 4)
        );
        drop(array);
        // 5 轮 create/drop：若每次 drop 未释放，累计泄漏 >8 MiB 会被下界断言抓到；
        // 驱动池复用导致的单次 free 计数不动不计入泄漏。
        let big: Vec<_> = std::iter::repeat_n(handle, 50_000).collect();
        let baseline = device_memory().expect("设备显存").0;
        for _ in 0..5 {
            let buffer = Texture2dArray::upload(&session, &big).expect("大纹理数组");
            assert_eq!(buffer.len(), big.len());
            drop(buffer);
        }
        let after = device_memory().expect("设备显存").0;
        assert!(
            baseline.saturating_sub(after) <= (8 << 20),
            "5 轮 create/drop 不得泄漏（基线 {baseline}、当前 {after}）"
        );
        println!(
            "s7a texture: baseline={baseline} after_cycles={after} bytes_per_cycle={} tex={:#x}",
            big.len() * Texture2d::BYTES,
            handle.tex
        );
    }
}
