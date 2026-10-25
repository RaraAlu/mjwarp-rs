// S7b3 AA 解析内核（渲染/BVH S7b3）：冻结上游 `render.py::_aa_resolve` 的逐式翻译。
//
// 上游（pinned 3.15）：
//   worldid, i = wp.tid()                    # dim = rgb_data.shape = (nworld, rgb_pixels)
//   c = accum[worldid, i] * inv_n            # inv_n = 1 / samples_per_pixel²
//   rgb_out[worldid, i] = pack_rgba_to_uint32(
//       wp.clamp(c[0], 0.0, 255.0), wp.clamp(c[1], 0.0, 255.0), wp.clamp(c[2], 0.0, 255.0), 255.0)
//
// 与 `_render_megakernel` 的 AA 路径配套：`aa` 时 `store_pixel` 把 `color * 255.0` 累加到
// `aa_accum`（float vec3），每帧按 sample 顺序发射 megakernel 后由本内核求平均并打包。
//
// 本文件被 `mod.rs::AA_RESOLVE_SOURCE` 单独拼接（不带 scene.cu / 着色链），因此自带
// `pack_rgba_to_uint32` 的等价实现（ABGR、`int()` 截断，与 `render_shade.cu` 逐式相同）。
// `inv_n` 走 1 元素 f32 数组槽：S7a 冻结的标量槽只支持 bool/i32/u64，没有 f32 标量。

#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"

// 上游 `pack_rgba_to_uint32`：`int()` 截断（不是四舍五入），通道顺序 ABGR。
__device__ __forceinline__ unsigned int aa_resolve_pack(float r, float g, float b, float a) {
    return ((unsigned int)(int)a << 24) | ((unsigned int)(int)r << 16) | ((unsigned int)(int)g << 8) |
           (unsigned int)(int)b;
}

extern "C" __global__ void aa_resolve(wp::launch_bounds_t<2> dim, wp::array_t<wp::vec3> accum,
                                      wp::array_t<wp::uint32> rgb_out,
                                      wp::array_t<wp::float32> inv_n) {
    size_t index = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= dim.size) {
        return;
    }
    int worldid = 0;
    int pixel = 0;
    wp::tid(worldid, pixel, index, dim);
    if (worldid >= dim.shape[0]) {
        return;
    }
    // accum/rgb 的 pitch 都是 rgb_pixels；AA 要求至少一个 RGB 输出（S6a 已校验）。
    int pitch = accum.shape.dims[1];
    long offset = (long)worldid * pitch + pixel;
    wp::vec3 c = accum.data[offset];
    float inv = inv_n.data[0];
    float r = fminf(fmaxf(c[0] * inv, 0.0f), 255.0f);
    float g = fminf(fmaxf(c[1] * inv, 0.0f), 255.0f);
    float b = fminf(fmaxf(c[2] * inv, 0.0f), 255.0f);
    rgb_out.data[offset] = aa_resolve_pack(r, g, b, 255.0f);
}
