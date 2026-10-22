#define WP_NO_BFLOAT16
#define WP_TILE_BLOCK_DIM 256
#define WP_NO_CRT
#include "builtin.h"
#include "deterministic.h"


static CUDA_CALLABLE wp::vec_t<3, wp::float32> load_v3(const float* p) { return wp::vec_t<3, wp::float32>(p[0], p[1], p[2]); }
static CUDA_CALLABLE wp::quat_t<wp::float32> load_q4(const float* p) { return wp::quat_t<wp::float32>(p[0], p[1], p[2], p[3]); }
static CUDA_CALLABLE void store_v3(float* p, wp::vec_t<3, wp::float32> v) {
  const wp::int32 i0 = 0; const wp::int32 i1 = 1; const wp::int32 i2 = 2;
  p[0] = wp::extract(v, i0); p[1] = wp::extract(v, i1); p[2] = wp::extract(v, i2);
}
static CUDA_CALLABLE void store_q4(float* p, wp::quat_t<wp::float32> q) {
  const wp::int32 i0 = 0; const wp::int32 i1 = 1; const wp::int32 i2 = 2; const wp::int32 i3 = 3;
  p[0] = wp::extract(q, i0); p[1] = wp::extract(q, i1); p[2] = wp::extract(q, i2); p[3] = wp::extract(q, i3);
}

extern "C" __global__ void probe_normalize(const float* in,float* out) {
  wp::quat_t<wp::float32> q=load_q4(in);
  store_q4(out,wp::normalize(q));
}
