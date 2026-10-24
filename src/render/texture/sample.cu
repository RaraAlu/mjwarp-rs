// GPU 2D texture sampler (render/BVH S4).
//
// ABI (SynchronousKernel four-buffer launch):
//   extern "C" __global__ void sample_texture2d(const int* meta, const float* params,
//       const float* state, float* out, unsigned d0, unsigned d1, unsigned d2, unsigned d3);
//
// meta:   [width, height, query_count, texture_handle_lo, texture_handle_hi]
// params: [unused]
// state:  [u, v] per query, normalized texture coordinates (Warp normalized_coords=True)
// out:    [r, g, b, a] per query, hardware LINEAR fetch
//
// The texture object is created on the host (texture/cuda.rs) with the same descriptor
// the frozen upstream `wp.Texture2D(..., filter_mode=LINEAR)` uses: CU_AD_FORMAT_FLOAT,
// 4 channels, 1 mip level, normalized coordinates, CLAMP address mode, LINEAR filtering.
// The PTX tex instruction is the device path of Warp `native/texture.h`
// (`texture_sample_helper<vec4f>::sample_2d` -> `tex2D<float4>`), so samples match the
// frozen MJWarp 3.15 reference bit-for-bit instead of approximating the hardware filter
// in software. The handle is reconstructed from two int slots of the meta buffer because
// the fixed four-buffer launch ABI has no u64 parameter (render launch ABI is S6).

extern "C" __global__ void sample_texture2d(const int* meta, const float* params,
                                            const float* state, float* out, unsigned d0,
                                            unsigned d1, unsigned d2, unsigned d3) {
  unsigned tid = blockIdx.x * blockDim.x + threadIdx.x;
  if (tid >= d3) {
    return;
  }
  const unsigned long long handle =
      (unsigned long long)(unsigned)meta[3] | ((unsigned long long)(unsigned)meta[4] << 32);
  const float u = state[2 * tid];
  const float v = state[2 * tid + 1];
  float r;
  float g;
  float b;
  float a;
  asm volatile("tex.2d.v4.f32.f32 {%0, %1, %2, %3}, [%4, {%5, %6}];"
               : "=f"(r), "=f"(g), "=f"(b), "=f"(a)
               : "l"(handle), "f"(u), "f"(v));
  out[4 * tid] = r;
  out[4 * tid + 1] = g;
  out[4 * tid + 2] = b;
  out[4 * tid + 3] = a;
}
