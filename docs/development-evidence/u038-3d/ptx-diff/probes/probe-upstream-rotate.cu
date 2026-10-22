#define WP_NO_BFLOAT16
#define WP_TILE_BLOCK_DIM 256
#define WP_NO_CRT
#include "builtin.h"
#include "deterministic.h"

static CUDA_CALLABLE wp::vec_t<3, wp::float32> rot_vec_quat_0(
    wp::vec_t<3, wp::float32> var_vec,
    wp::quat_t<wp::float32> var_quat)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 2;
    wp::float32 var_5;
    const wp::int32 var_6 = 3;
    wp::float32 var_7;
    wp::vec_t<3, wp::float32> var_8;
    const wp::float32 var_9 = 2.0;
    wp::float32 var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::vec_t<3, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    const wp::float32 var_18 = 2.0;
    wp::float32 var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    //---------
    // forward
    // def rot_vec_quat(vec: wp.vec3, quat: wp.quat) -> wp.vec3:                              <L 45>
    // s, u = quat[0], wp.vec3(quat[1], quat[2], quat[3])                                     <L 46>
    var_1 = wp::extract(var_quat, var_0);
    var_3 = wp::extract(var_quat, var_2);
    var_5 = wp::extract(var_quat, var_4);
    var_7 = wp::extract(var_quat, var_6);
    var_8 = wp::vec_t<3, wp::float32>(var_3, var_5, var_7);
    // r = 2.0 * (wp.dot(u, vec) * u) + (s * s - wp.dot(u, u)) * vec                          <L 47>
    var_10 = wp::dot(var_8, var_vec);
    var_11 = wp::mul(var_10, var_8);
    var_12 = wp::mul(var_9, var_11);
    var_13 = wp::mul(var_1, var_1);
    var_14 = wp::dot(var_8, var_8);
    var_15 = wp::sub(var_13, var_14);
    var_16 = wp::mul(var_15, var_vec);
    var_17 = wp::add(var_12, var_16);
    // r = r + 2.0 * s * wp.cross(u, vec)                                                     <L 48>
    var_19 = wp::mul(var_18, var_1);
    var_20 = wp::cross(var_8, var_vec);
    var_21 = wp::mul(var_19, var_20);
    var_22 = wp::add(var_17, var_21);
    // return r                                                                               <L 49>
    return var_22;
}

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

extern "C" __global__ void probe_rotate(const float* in,float* out) {
  wp::quat_t<wp::float32> q=load_q4(in); wp::vec_t<3, wp::float32> v=load_v3(in+4);
  store_v3(out,rot_vec_quat_0(v,q));
}
