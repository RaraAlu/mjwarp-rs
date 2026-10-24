//! CUDA 纹理对象与线性采样（渲染/BVH S4，cuda-probe）。
//!
//! 上游 `render_util.create_warp_texture` 用 `wp.Texture2D(data, filter_mode=LINEAR)`：
//! 浮点 RGBA CUDA 数组 + 纹理对象（归一化坐标、CLAMP 寻址、单 mip），设备端由
//! `wp.texture_sample` 走 `tex2D<float4>` 硬件取指。本模块用驱动 API 建立同样的
//! 纹理对象（`cuArray3DCreate_v2` → `cuMemcpy2D_v2` → `cuTexObjectCreate`），采样内核
//! `sample.cu` 用 PTX `tex.2d.v4.f32` 取指，因此采样值与冻结参考逐位一致，而不是用
//! 软件双线性去近似硬件过滤。
//!
//! 上下文：cudarc 的传输会话使用设备主上下文（primary context）；本模块在创建/销毁
//! 纹理对象期间 retain + push 同一主上下文，结束后 pop + release，不改变会话状态。

use cudarc::driver::sys;
use std::ptr;

use super::{TextureData, TextureRegistry};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::runtime::{SynchronousKernel, TransferSession};

const ENTRY: &str = "sample_texture2d";
const SOURCE: &str = include_str!("sample.cu");
const TEXTURE_CHANNELS: u32 = 4;
const FLOAT_BYTES: usize = 4;

/// 驱动错误 → 传输错误；`stage` 标记失败阶段。
fn check(stage: &'static str, result: sys::CUresult) -> Result<(), TransferError> {
    if result == sys::CUresult::CUDA_SUCCESS {
        Ok(())
    } else {
        Err(ProbeError::Cuda {
            stage,
            code: result as u32,
        }
        .into())
    }
}

/// 设备主上下文的 retain/push 守卫；失败路径也保证 pop 与 release。
struct PrimaryContext {
    device: sys::CUdevice,
    pushed: bool,
}

impl Drop for PrimaryContext {
    fn drop(&mut self) {
        unsafe {
            if self.pushed {
                let mut popped: sys::CUcontext = ptr::null_mut();
                let _ = sys::cuCtxPopCurrent_v2(&mut popped);
            }
            let _ = sys::cuDevicePrimaryCtxRelease_v2(self.device);
        }
    }
}

fn with_primary_context<T>(
    device: usize,
    operation: impl FnOnce() -> Result<T, TransferError>,
) -> Result<T, TransferError> {
    let ordinal = i32::try_from(device).map_err(|_| InputError::Overflow {
        field: "texture_device",
    })?;
    unsafe {
        let mut raw_device: sys::CUdevice = 0;
        check("texture-device", sys::cuDeviceGet(&mut raw_device, ordinal))?;
        let mut context: sys::CUcontext = ptr::null_mut();
        check(
            "texture-primary-context",
            sys::cuDevicePrimaryCtxRetain(&mut context, raw_device),
        )?;
        let mut guard = PrimaryContext {
            device: raw_device,
            pushed: false,
        };
        check("texture-context-push", sys::cuCtxPushCurrent_v2(context))?;
        guard.pushed = true;
        let result = operation();
        drop(guard);
        result
    }
}

/// 单个 CUDA 纹理对象及其底层数组；drop 前必须显式销毁。
struct CudaTexture {
    texture: sys::CUtexObject,
    array: sys::CUarray,
    width: u32,
    height: u32,
}

/// 模型纹理的 CUDA 对象集合与采样器；与上游 `RenderContext.textures` 对应。
pub struct TextureObjects {
    session: TransferSession,
    kernel: SynchronousKernel,
    textures: Vec<CudaTexture>,
}

impl TextureObjects {
    /// 按注册表顺序创建全部纹理对象，并编译采样内核。
    pub fn new(
        session: &TransferSession,
        registry: &TextureRegistry,
    ) -> Result<Self, TransferError> {
        let kernel = SynchronousKernel::compile(session, SOURCE, ENTRY)?;
        let mut textures = Vec::with_capacity(registry.len());
        for texture in registry.iter() {
            textures.push(create_texture(session.device(), texture)?);
        }
        Ok(Self {
            session: session.clone(),
            kernel,
            textures,
        })
    }

    /// 纹理数（等于模型 `ntex`）。
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// 按模型纹理 id 采样固定 uv 列表；返回每查询 RGBA。
    pub fn sample(&self, texture: usize, uvs: &[[f32; 2]]) -> Result<Vec<[f32; 4]>, TransferError> {
        let entry = self.textures.get(texture).ok_or(InputError::InvalidIndex {
            field: "texture",
            index: texture,
            limit: self.textures.len(),
        })?;
        if uvs.is_empty() {
            return Ok(Vec::new());
        }
        let count = u32::try_from(uvs.len()).map_err(|_| InputError::Overflow {
            field: "texture_queries",
        })?;
        let state_len = uvs.len().checked_mul(2).ok_or(InputError::Overflow {
            field: "texture_state",
        })?;
        let out_len = uvs.len().checked_mul(4).ok_or(InputError::Overflow {
            field: "texture_output",
        })?;
        let handle = entry.texture;
        let meta = [
            entry.width as i32,
            entry.height as i32,
            count as i32,
            (handle & 0xFFFF_FFFF) as u32 as i32,
            (handle >> 32) as u32 as i32,
        ];
        let mut state = Vec::with_capacity(state_len);
        for uv in uvs {
            state.push(uv[0]);
            state.push(uv[1]);
        }
        let params = [0.0f32];
        let meta_buffer = self.session.upload(&meta)?;
        let params_buffer = self.session.upload(&params)?;
        let state_buffer = self.session.upload(&state)?;
        let mut out = vec![0.0f32; out_len];
        let mut out_buffer = self.session.upload(&out)?;
        // SAFETY: ABI 与缓冲范围在此校验；sample.cu 只读 meta/state、写 4*count 个 f32，
        // grid 恰好覆盖 count 且有 tid 守卫；缓冲与会话一致，发射同步完成。
        unsafe {
            self.kernel.launch(
                &meta_buffer,
                &params_buffer,
                &state_buffer,
                &mut out_buffer,
                [0, 0, 0, count],
            )?;
        }
        out_buffer.read_range_into(0, &mut out)?;
        let mut samples = Vec::with_capacity(uvs.len());
        for index in 0..uvs.len() {
            let base = 4 * index;
            samples.push([out[base], out[base + 1], out[base + 2], out[base + 3]]);
        }
        Ok(samples)
    }
}

impl Drop for TextureObjects {
    fn drop(&mut self) {
        if self.textures.is_empty() {
            return;
        }
        let device = self.session.device();
        // 先释放纹理对象，再释放其引用的数组；失败时保留（上下文销毁时回收）。
        let _ = with_primary_context(device, || {
            unsafe {
                for texture in &self.textures {
                    let _ = sys::cuTexObjectDestroy(texture.texture);
                    let _ = sys::cuArrayDestroy(texture.array);
                }
            }
            Ok::<(), TransferError>(())
        });
    }
}

fn create_texture(device: usize, data: &TextureData) -> Result<CudaTexture, TransferError> {
    let width = usize::try_from(data.width()).map_err(|_| InputError::Overflow {
        field: "texture_width",
    })?;
    let height = usize::try_from(data.height()).map_err(|_| InputError::Overflow {
        field: "texture_height",
    })?;
    let expected = width
        .checked_mul(height)
        .and_then(|texels| texels.checked_mul(4))
        .ok_or(InputError::Overflow {
            field: "texture_texels",
        })?;
    if data.texels().len() != expected {
        return Err(InputError::InvalidDimension {
            field: "texture_texels",
        }
        .into());
    }
    let pitch = width
        .checked_mul(TEXTURE_CHANNELS as usize)
        .and_then(|bytes| bytes.checked_mul(FLOAT_BYTES))
        .ok_or(InputError::Overflow {
            field: "texture_pitch",
        })?;
    with_primary_context(device, || unsafe {
        let descriptor = sys::CUDA_ARRAY3D_DESCRIPTOR {
            Width: width,
            Height: height,
            Depth: 0,
            Format: sys::CUarray_format::CU_AD_FORMAT_FLOAT,
            NumChannels: TEXTURE_CHANNELS,
            Flags: 0,
        };
        let mut array: sys::CUarray = ptr::null_mut();
        check(
            "texture-array-create",
            sys::cuArray3DCreate_v2(&mut array, &descriptor),
        )?;
        let copy = sys::CUDA_MEMCPY2D {
            srcXInBytes: 0,
            srcY: 0,
            srcMemoryType: sys::CUmemorytype::CU_MEMORYTYPE_HOST,
            srcHost: data.texels().as_ptr().cast(),
            srcDevice: 0,
            srcArray: ptr::null_mut(),
            srcPitch: pitch,
            dstXInBytes: 0,
            dstY: 0,
            dstMemoryType: sys::CUmemorytype::CU_MEMORYTYPE_ARRAY,
            dstHost: ptr::null_mut(),
            dstDevice: 0,
            dstArray: array,
            dstPitch: 0,
            WidthInBytes: pitch,
            Height: height,
        };
        if let Err(error) = check("texture-array-copy", sys::cuMemcpy2D_v2(&copy)) {
            let _ = sys::cuArrayDestroy(array);
            return Err(error);
        }
        let resource = sys::CUDA_RESOURCE_DESC {
            resType: sys::CUresourcetype::CU_RESOURCE_TYPE_ARRAY,
            res: sys::CUDA_RESOURCE_DESC_st__bindgen_ty_1 {
                array: sys::CUDA_RESOURCE_DESC_st__bindgen_ty_1__bindgen_ty_1 { hArray: array },
            },
            flags: 0,
        };
        let texture = sys::CUDA_TEXTURE_DESC {
            addressMode: [sys::CUaddress_mode::CU_TR_ADDRESS_MODE_CLAMP; 3],
            filterMode: sys::CUfilter_mode::CU_TR_FILTER_MODE_LINEAR,
            flags: sys::CU_TRSF_NORMALIZED_COORDINATES,
            maxAnisotropy: 0,
            mipmapFilterMode: sys::CUfilter_mode::CU_TR_FILTER_MODE_LINEAR,
            mipmapLevelBias: 0.0,
            minMipmapLevelClamp: 0.0,
            maxMipmapLevelClamp: 0.0,
            borderColor: [0.0; 4],
            reserved: [0; 12],
        };
        let mut object: sys::CUtexObject = 0;
        if let Err(error) = check(
            "texture-object-create",
            sys::cuTexObjectCreate(&mut object, &resource, &texture, ptr::null()),
        ) {
            let _ = sys::cuArrayDestroy(array);
            return Err(error);
        }
        Ok(CudaTexture {
            texture: object,
            array,
            width: data.width(),
            height: data.height(),
        })
    })
}
