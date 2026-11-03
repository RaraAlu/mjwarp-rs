
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
