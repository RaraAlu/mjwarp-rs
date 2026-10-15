#define WP_NO_BFLOAT16

#define WP_TILE_BLOCK_DIM 256
#define WP_NO_CRT
#include "builtin.h"
#include "deterministic.h"

// Map wp.breakpoint() to a device brkpt at the call site so cuda-gdb attributes the stop to the generated .cu line
#if defined(__CUDACC__) && !defined(_MSC_VER)
#define __debugbreak() __brkpt()
#endif

// avoid namespacing of float type for casting to float type, this is to avoid wp::float(x), which is not valid in C++
#define float(x) cast_float(x)
#define adj_float(x, adj_x, adj_ret) adj_cast_float(x, adj_x, adj_ret)

#define int(x) cast_int(x)
#define adj_int(x, adj_x, adj_ret) adj_cast_int(x, adj_x, adj_ret)

#define builtin_tid1d() wp::tid(_idx, dim)
#define builtin_tid2d(x, y) wp::tid(x, y, _idx, dim)
#define builtin_tid3d(x, y, z) wp::tid(x, y, z, _idx, dim)
#define builtin_tid4d(x, y, z, w) wp::tid(x, y, z, w, _idx, dim)

#define builtin_block_dim() wp::block_dim()

// CUDA Thread Block Cluster shape declaration. Expands to __cluster_dims__
// only on devices that support clusters (compute capability 9.0+); otherwise
// expands to nothing so the same source compiles cleanly for any target arch.
#if defined(__CUDA_ARCH__) && (__CUDA_ARCH__ >= 900)
#define WP_CLUSTER_DIMS(x, y, z) __cluster_dims__(x, y, z)
#else
#define WP_CLUSTER_DIMS(x, y, z)
#endif


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:0
static CUDA_CALLABLE wp::int32 sap_binary_search_0(
    wp::array_t<wp::int32> var_values,
    wp::int32 var_value,
    wp::int32 var_lower,
    wp::int32 var_upper)
{
    //---------
    // primal vars
    bool var_0;
    wp::int32 var_1;
    const wp::int32 var_2 = 1;
    wp::int32 var_3;
    wp::int32* var_4;
    bool var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::int32 var_8;
    const wp::int32 var_9 = 1;
    wp::int32 var_10;
    wp::int32 var_11;
    //---------
    // forward
    // def sap_binary_search(values: wp.array[Any], value: Any, lower: int, upper: int) -> int:       <L 1>
    // while lower < upper:                                                                   <L 3>
    start_while_0:;
    var_0 = (var_lower < var_upper);
    if ((var_0) == false) goto end_while_0;
        // mid = (lower + upper) >> 1                                                         <L 4>
        var_1 = wp::add(var_lower, var_upper);
        var_3 = wp::rshift(var_1, var_2);
        // if values[mid] > value:                                                            <L 5>
        var_4 = wp::address(var_values, var_3);
        var_6 = wp::load(var_4);
        var_5 = (var_6 > var_value);
        if (var_5) {
            // upper = mid                                                                    <L 6>
            var_7 = wp::copy(var_3);
        }
        var_8 = wp::where(var_5, var_7, var_upper);
        if (!var_5) {
            // lower = mid + 1                                                                <L 8>
            var_10 = wp::add(var_3, var_9);
        }
        var_11 = wp::where(var_5, var_lower, var_10);
        wp::assign(var_lower, var_11);
        wp::assign(var_upper, var_8);
    goto start_while_0;
    end_while_0:;
    // return upper                                                                           <L 9>
    return var_upper;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:51
static CUDA_CALLABLE bool _flex_element_aabb_filter_0(
    wp::vec_t<3, wp::float32> var_box1_min,
    wp::vec_t<3, wp::float32> var_box1_max,
    wp::vec_t<3, wp::float32> var_box2_min,
    wp::vec_t<3, wp::float32> var_box2_max)
{
    //---------
    // primal vars
    bool var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    bool var_5;
    const wp::int32 var_6 = 0;
    wp::float32 var_7;
    const wp::int32 var_8 = 0;
    wp::float32 var_9;
    bool var_10;
    const bool var_11 = true;
    bool var_12;
    const wp::int32 var_13 = 1;
    wp::float32 var_14;
    const wp::int32 var_15 = 1;
    wp::float32 var_16;
    bool var_17;
    const wp::int32 var_18 = 1;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    bool var_22;
    const bool var_23 = true;
    bool var_24;
    const wp::int32 var_25 = 2;
    wp::float32 var_26;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    bool var_29;
    const wp::int32 var_30 = 2;
    wp::float32 var_31;
    const wp::int32 var_32 = 2;
    wp::float32 var_33;
    bool var_34;
    const bool var_35 = true;
    const bool var_36 = false;
    //---------
    // forward
    // def _flex_element_aabb_filter(                                                         <L 52>
    // if box1_max[0] < box2_min[0] or box1_min[0] > box2_max[0]:                             <L 60>
    var_2 = wp::extract(var_box1_max, var_1);
    var_4 = wp::extract(var_box2_min, var_3);
    var_5 = (var_2 < var_4);
    var_0 = var_5;
    if (!var_0) {
        var_7 = wp::extract(var_box1_min, var_6);
        var_9 = wp::extract(var_box2_max, var_8);
        var_10 = (var_7 > var_9);
        var_0 = var_0 || var_10;
    }
    if (var_0) {
        // return True                                                                        <L 61>
        return var_11;
    }
    // if box1_max[1] < box2_min[1] or box1_min[1] > box2_max[1]:                             <L 62>
    var_14 = wp::extract(var_box1_max, var_13);
    var_16 = wp::extract(var_box2_min, var_15);
    var_17 = (var_14 < var_16);
    var_12 = var_17;
    if (!var_12) {
        var_19 = wp::extract(var_box1_min, var_18);
        var_21 = wp::extract(var_box2_max, var_20);
        var_22 = (var_19 > var_21);
        var_12 = var_12 || var_22;
    }
    if (var_12) {
        // return True                                                                        <L 63>
        return var_23;
    }
    // if box1_max[2] < box2_min[2] or box1_min[2] > box2_max[2]:                             <L 64>
    var_26 = wp::extract(var_box1_max, var_25);
    var_28 = wp::extract(var_box2_min, var_27);
    var_29 = (var_26 < var_28);
    var_24 = var_29;
    if (!var_24) {
        var_31 = wp::extract(var_box1_min, var_30);
        var_33 = wp::extract(var_box2_max, var_32);
        var_34 = (var_31 > var_33);
        var_24 = var_24 || var_34;
    }
    if (var_24) {
        // return True                                                                        <L 65>
        return var_35;
    }
    // return False                                                                           <L 66>
    return var_36;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1266
static CUDA_CALLABLE wp::vec_t<4, wp::int32> _get_element_vertices_0(
    wp::array_t<wp::int32> var_flex_elem,
    wp::int32 var_dim,
    wp::int32 var_elem_data_idx)
{
    //---------
    // primal vars
    wp::int32* var_0;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 1;
    wp::int32 var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    const wp::int32 var_8 = -1;
    wp::int32 var_9;
    const wp::int32 var_10 = -1;
    wp::int32 var_11;
    const wp::int32 var_12 = 2;
    bool var_13;
    const wp::int32 var_14 = 2;
    wp::int32 var_15;
    wp::int32* var_16;
    wp::int32 var_17;
    wp::int32 var_18;
    wp::int32 var_19;
    const wp::int32 var_20 = 3;
    bool var_21;
    const wp::int32 var_22 = 3;
    wp::int32 var_23;
    wp::int32* var_24;
    wp::int32 var_25;
    wp::int32 var_26;
    wp::int32 var_27;
    wp::vec_t<4, wp::int32> var_28;
    //---------
    // forward
    // def _get_element_vertices(                                                             <L 1267>
    // v0 = flex_elem[elem_data_idx]                                                          <L 1274>
    var_0 = wp::address(var_flex_elem, var_elem_data_idx);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // v1 = flex_elem[elem_data_idx + 1]                                                      <L 1275>
    var_4 = wp::add(var_elem_data_idx, var_3);
    var_5 = wp::address(var_flex_elem, var_4);
    var_7 = wp::load(var_5);
    var_6 = wp::copy(var_7);
    // v2 = int(-1)                                                                           <L 1276>
    var_9 = wp::int(var_8);
    // v3 = int(-1)                                                                           <L 1277>
    var_11 = wp::int(var_10);
    // if dim >= 2:                                                                           <L 1278>
    var_13 = (var_dim >= var_12);
    if (var_13) {
        // v2 = flex_elem[elem_data_idx + 2]                                                  <L 1279>
        var_15 = wp::add(var_elem_data_idx, var_14);
        var_16 = wp::address(var_flex_elem, var_15);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
    }
    var_19 = wp::where(var_13, var_17, var_9);
    // if dim >= 3:                                                                           <L 1280>
    var_21 = (var_dim >= var_20);
    if (var_21) {
        // v3 = flex_elem[elem_data_idx + 3]                                                  <L 1281>
        var_23 = wp::add(var_elem_data_idx, var_22);
        var_24 = wp::address(var_flex_elem, var_23);
        var_26 = wp::load(var_24);
        var_25 = wp::copy(var_26);
    }
    var_27 = wp::where(var_21, var_25, var_11);
    // return wp.vec4i(v0, v1, v2, v3)                                                        <L 1282>
    var_28 = wp::vec_t<4, wp::int32>(var_1, var_6, var_19, var_27);
    return var_28;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1240
static CUDA_CALLABLE bool _exclude_self_collision_0(
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::vec_t<4, wp::int32> var_v1,
    wp::int32 var_n1,
    wp::vec_t<4, wp::int32> var_v2,
    wp::int32 var_n2,
    wp::int32 var_vert_adr)
{
    //---------
    // primal vars
    wp::range_t var_0;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    wp::int32 var_5;
    wp::int32* var_6;
    wp::int32 var_7;
    wp::int32 var_8;
    wp::range_t var_9;
    wp::int32 var_10;
    wp::int32 var_11;
    bool var_12;
    const bool var_13 = true;
    bool var_14;
    const wp::int32 var_15 = 0;
    bool var_16;
    const wp::int32 var_17 = 0;
    bool var_18;
    wp::int32 var_19;
    wp::int32* var_20;
    wp::int32 var_21;
    wp::int32 var_22;
    bool var_23;
    const bool var_24 = true;
    const bool var_25 = false;
    //---------
    // forward
    // def _exclude_self_collision(                                                           <L 1241>
    // for i in range(n1):                                                                    <L 1251>
    var_0 = wp::range(var_n1);
    start_for_0:;
        if (iter_cmp(var_0) == 0) goto end_for_0;
        var_1 = wp::iter_next(var_0);
        // idx1 = v1[i]                                                                       <L 1252>
        var_2 = wp::extract(var_v1, var_1);
        // if idx1 >= 0:                                                                      <L 1253>
        var_4 = (var_2 >= var_3);
        if (var_4) {
            // b1 = flex_vertbodyid[vert_adr + idx1]                                          <L 1254>
            var_5 = wp::add(var_vert_adr, var_2);
            var_6 = wp::address(var_flex_vertbodyid, var_5);
            var_8 = wp::load(var_6);
            var_7 = wp::copy(var_8);
            // for j in range(n2):                                                            <L 1255>
            var_9 = wp::range(var_n2);
            start_for_2:;
                if (iter_cmp(var_9) == 0) goto end_for_2;
                var_10 = wp::iter_next(var_9);
                // idx2 = v2[j]                                                               <L 1256>
                var_11 = wp::extract(var_v2, var_10);
                // if idx1 == idx2:                                                           <L 1257>
                var_12 = (var_2 == var_11);
                if (var_12) {
                    // return True                                                            <L 1258>
                    return var_13;
                }
                // if idx2 >= 0 and b1 >= 0:                                                  <L 1259>
                var_16 = (var_11 >= var_15);
                var_14 = var_16;
                if (var_14) {
                    var_18 = (var_7 >= var_17);
                    var_14 = var_14 && var_18;
                }
                if (var_14) {
                    // b2 = flex_vertbodyid[vert_adr + idx2]                                  <L 1260>
                    var_19 = wp::add(var_vert_adr, var_11);
                    var_20 = wp::address(var_flex_vertbodyid, var_19);
                    var_22 = wp::load(var_20);
                    var_21 = wp::copy(var_22);
                    // if b1 == b2:                                                           <L 1261>
                    var_23 = (var_7 == var_21);
                    if (var_23) {
                        // return True                                                        <L 1262>
                        return var_24;
                    }
                }
                goto start_for_2;
            end_for_2:;
        }
        goto start_for_0;
    end_for_0:;
    // return False                                                                           <L 1263>
    return var_25;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:69
static CUDA_CALLABLE bool _axis_separated_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_q0,
    wp::vec_t<3, wp::float32> var_q1,
    wp::vec_t<3, wp::float32> var_q2,
    wp::vec_t<3, wp::float32> var_ax,
    wp::float32 var_cutoff_sq)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    wp::float32 var_25;
    bool var_26;
    const wp::float32 var_27 = 1e-12;
    bool var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    bool var_31;
    const bool var_32 = true;
    const bool var_33 = false;
    //---------
    // forward
    // def _axis_separated(                                                                   <L 70>
    // min1 = wp.min(wp.dot(p0, ax), wp.min(wp.dot(p1, ax), wp.dot(p2, ax)))                  <L 81>
    var_0 = wp::dot(var_p0, var_ax);
    var_1 = wp::dot(var_p1, var_ax);
    var_2 = wp::dot(var_p2, var_ax);
    var_3 = wp::min(var_1, var_2);
    var_4 = wp::min(var_0, var_3);
    // max1 = wp.max(wp.dot(p0, ax), wp.max(wp.dot(p1, ax), wp.dot(p2, ax)))                  <L 82>
    var_5 = wp::dot(var_p0, var_ax);
    var_6 = wp::dot(var_p1, var_ax);
    var_7 = wp::dot(var_p2, var_ax);
    var_8 = wp::max(var_6, var_7);
    var_9 = wp::max(var_5, var_8);
    // min2 = wp.min(wp.dot(q0, ax), wp.min(wp.dot(q1, ax), wp.dot(q2, ax)))                  <L 83>
    var_10 = wp::dot(var_q0, var_ax);
    var_11 = wp::dot(var_q1, var_ax);
    var_12 = wp::dot(var_q2, var_ax);
    var_13 = wp::min(var_11, var_12);
    var_14 = wp::min(var_10, var_13);
    // max2 = wp.max(wp.dot(q0, ax), wp.max(wp.dot(q1, ax), wp.dot(q2, ax)))                  <L 84>
    var_15 = wp::dot(var_q0, var_ax);
    var_16 = wp::dot(var_q1, var_ax);
    var_17 = wp::dot(var_q2, var_ax);
    var_18 = wp::max(var_16, var_17);
    var_19 = wp::max(var_15, var_18);
    // diff = wp.max(min1 - max2, min2 - max1)                                                <L 85>
    var_20 = wp::sub(var_4, var_19);
    var_21 = wp::sub(var_14, var_9);
    var_22 = wp::max(var_20, var_21);
    // if diff > 0.0:                                                                         <L 86>
    var_24 = (var_22 > var_23);
    if (var_24) {
        // lax_sq = wp.length_sq(ax)                                                          <L 87>
        var_25 = wp::length_sq(var_ax);
        // if lax_sq > 1e-12 and diff * diff > cutoff_sq * lax_sq:                            <L 88>
        var_28 = (var_25 > var_27);
        var_26 = var_28;
        if (var_26) {
            var_29 = wp::mul(var_22, var_22);
            var_30 = wp::mul(var_cutoff_sq, var_25);
            var_31 = (var_29 > var_30);
            var_26 = var_26 && var_31;
        }
        if (var_26) {
            // return True                                                                    <L 89>
            return var_32;
        }
    }
    // return False                                                                           <L 90>
    return var_33;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:93
static CUDA_CALLABLE wp::vec_t<3, wp::float32> _point_segment_axis_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> var_b)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::float32 var_1;
    const wp::float32 var_2 = 1e-12;
    bool var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::float32 var_7 = 0.0;
    const wp::float32 var_8 = 1.0;
    wp::float32 var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    //---------
    // forward
    // def _point_segment_axis(p: wp.vec3, a: wp.vec3, b: wp.vec3) -> wp.vec3:                <L 94>
    // ab = b - a                                                                             <L 95>
    var_0 = wp::sub(var_b, var_a);
    // ab_len_sq = wp.length_sq(ab)                                                           <L 96>
    var_1 = wp::length_sq(var_0);
    // if ab_len_sq > 1e-12:                                                                  <L 97>
    var_3 = (var_1 > var_2);
    if (var_3) {
        // ap = p - a                                                                         <L 98>
        var_4 = wp::sub(var_p, var_a);
        // t = wp.clamp(wp.dot(ap, ab) / ab_len_sq, 0.0, 1.0)                                 <L 99>
        var_5 = wp::dot(var_4, var_0);
        var_6 = wp::div(var_5, var_1);
        var_9 = wp::clamp(var_6, var_7, var_8);
        // return ap - t * ab                                                                 <L 100>
        var_10 = wp::mul(var_9, var_0);
        var_11 = wp::sub(var_4, var_10);
        return var_11;
    }
    // return p - a                                                                           <L 101>
    var_12 = wp::sub(var_p, var_a);
    return var_12;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:104
static CUDA_CALLABLE bool _triangle_sat_separated_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_q0,
    wp::vec_t<3, wp::float32> var_q1,
    wp::vec_t<3, wp::float32> var_q2,
    wp::float32 var_cutoff_sq)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::float32 var_15 = 0.0;
    bool var_16;
    wp::float32 var_17;
    bool var_18;
    const wp::float32 var_19 = 1e-12;
    bool var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    bool var_23;
    const bool var_24 = true;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::float32 var_40 = 0.0;
    bool var_41;
    wp::float32 var_42;
    bool var_43;
    const wp::float32 var_44 = 1e-12;
    bool var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    bool var_48;
    const bool var_49 = true;
    const wp::int32 var_50 = 0;
    wp::vec_t<3, wp::float32> var_51;
    const wp::int32 var_52 = 1;
    bool var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::vec_t<3, wp::float32> var_55;
    const wp::int32 var_56 = 2;
    bool var_57;
    wp::vec_t<3, wp::float32> var_58;
    wp::vec_t<3, wp::float32> var_59;
    wp::vec_t<3, wp::float32> var_60;
    const wp::int32 var_61 = 0;
    wp::vec_t<3, wp::float32> var_62;
    const wp::int32 var_63 = 1;
    bool var_64;
    wp::vec_t<3, wp::float32> var_65;
    wp::vec_t<3, wp::float32> var_66;
    const wp::int32 var_67 = 2;
    bool var_68;
    wp::vec_t<3, wp::float32> var_69;
    wp::vec_t<3, wp::float32> var_70;
    wp::vec_t<3, wp::float32> var_71;
    wp::vec_t<3, wp::float32> var_72;
    bool var_73;
    const bool var_74 = true;
    const wp::int32 var_75 = 1;
    wp::vec_t<3, wp::float32> var_76;
    const wp::int32 var_77 = 1;
    bool var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::vec_t<3, wp::float32> var_80;
    const wp::int32 var_81 = 2;
    bool var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::vec_t<3, wp::float32> var_84;
    wp::vec_t<3, wp::float32> var_85;
    wp::vec_t<3, wp::float32> var_86;
    bool var_87;
    const bool var_88 = true;
    const wp::int32 var_89 = 2;
    wp::vec_t<3, wp::float32> var_90;
    const wp::int32 var_91 = 1;
    bool var_92;
    wp::vec_t<3, wp::float32> var_93;
    wp::vec_t<3, wp::float32> var_94;
    const wp::int32 var_95 = 2;
    bool var_96;
    wp::vec_t<3, wp::float32> var_97;
    wp::vec_t<3, wp::float32> var_98;
    wp::vec_t<3, wp::float32> var_99;
    wp::vec_t<3, wp::float32> var_100;
    bool var_101;
    const bool var_102 = true;
    const wp::int32 var_103 = 1;
    wp::vec_t<3, wp::float32> var_104;
    const wp::int32 var_105 = 1;
    bool var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    const wp::int32 var_109 = 2;
    bool var_110;
    wp::vec_t<3, wp::float32> var_111;
    wp::vec_t<3, wp::float32> var_112;
    wp::vec_t<3, wp::float32> var_113;
    const wp::int32 var_114 = 0;
    wp::vec_t<3, wp::float32> var_115;
    const wp::int32 var_116 = 1;
    bool var_117;
    wp::vec_t<3, wp::float32> var_118;
    wp::vec_t<3, wp::float32> var_119;
    const wp::int32 var_120 = 2;
    bool var_121;
    wp::vec_t<3, wp::float32> var_122;
    wp::vec_t<3, wp::float32> var_123;
    wp::vec_t<3, wp::float32> var_124;
    wp::vec_t<3, wp::float32> var_125;
    bool var_126;
    const bool var_127 = true;
    const wp::int32 var_128 = 1;
    wp::vec_t<3, wp::float32> var_129;
    const wp::int32 var_130 = 1;
    bool var_131;
    wp::vec_t<3, wp::float32> var_132;
    wp::vec_t<3, wp::float32> var_133;
    const wp::int32 var_134 = 2;
    bool var_135;
    wp::vec_t<3, wp::float32> var_136;
    wp::vec_t<3, wp::float32> var_137;
    wp::vec_t<3, wp::float32> var_138;
    wp::vec_t<3, wp::float32> var_139;
    bool var_140;
    const bool var_141 = true;
    const wp::int32 var_142 = 2;
    wp::vec_t<3, wp::float32> var_143;
    const wp::int32 var_144 = 1;
    bool var_145;
    wp::vec_t<3, wp::float32> var_146;
    wp::vec_t<3, wp::float32> var_147;
    const wp::int32 var_148 = 2;
    bool var_149;
    wp::vec_t<3, wp::float32> var_150;
    wp::vec_t<3, wp::float32> var_151;
    wp::vec_t<3, wp::float32> var_152;
    wp::vec_t<3, wp::float32> var_153;
    bool var_154;
    const bool var_155 = true;
    const wp::int32 var_156 = 2;
    wp::vec_t<3, wp::float32> var_157;
    const wp::int32 var_158 = 1;
    bool var_159;
    wp::vec_t<3, wp::float32> var_160;
    wp::vec_t<3, wp::float32> var_161;
    const wp::int32 var_162 = 2;
    bool var_163;
    wp::vec_t<3, wp::float32> var_164;
    wp::vec_t<3, wp::float32> var_165;
    wp::vec_t<3, wp::float32> var_166;
    const wp::int32 var_167 = 0;
    wp::vec_t<3, wp::float32> var_168;
    const wp::int32 var_169 = 1;
    bool var_170;
    wp::vec_t<3, wp::float32> var_171;
    wp::vec_t<3, wp::float32> var_172;
    const wp::int32 var_173 = 2;
    bool var_174;
    wp::vec_t<3, wp::float32> var_175;
    wp::vec_t<3, wp::float32> var_176;
    wp::vec_t<3, wp::float32> var_177;
    wp::vec_t<3, wp::float32> var_178;
    bool var_179;
    const bool var_180 = true;
    const wp::int32 var_181 = 1;
    wp::vec_t<3, wp::float32> var_182;
    const wp::int32 var_183 = 1;
    bool var_184;
    wp::vec_t<3, wp::float32> var_185;
    wp::vec_t<3, wp::float32> var_186;
    const wp::int32 var_187 = 2;
    bool var_188;
    wp::vec_t<3, wp::float32> var_189;
    wp::vec_t<3, wp::float32> var_190;
    wp::vec_t<3, wp::float32> var_191;
    wp::vec_t<3, wp::float32> var_192;
    bool var_193;
    const bool var_194 = true;
    const wp::int32 var_195 = 2;
    wp::vec_t<3, wp::float32> var_196;
    const wp::int32 var_197 = 1;
    bool var_198;
    wp::vec_t<3, wp::float32> var_199;
    wp::vec_t<3, wp::float32> var_200;
    const wp::int32 var_201 = 2;
    bool var_202;
    wp::vec_t<3, wp::float32> var_203;
    wp::vec_t<3, wp::float32> var_204;
    wp::vec_t<3, wp::float32> var_205;
    wp::vec_t<3, wp::float32> var_206;
    bool var_207;
    const bool var_208 = true;
    const wp::int32 var_209 = 0;
    wp::vec_t<3, wp::float32> var_210;
    const wp::int32 var_211 = 1;
    bool var_212;
    wp::vec_t<3, wp::float32> var_213;
    wp::vec_t<3, wp::float32> var_214;
    const wp::int32 var_215 = 2;
    bool var_216;
    wp::vec_t<3, wp::float32> var_217;
    wp::vec_t<3, wp::float32> var_218;
    wp::vec_t<3, wp::float32> var_219;
    wp::vec_t<3, wp::float32> var_220;
    bool var_221;
    const bool var_222 = true;
    const wp::int32 var_223 = 1;
    wp::vec_t<3, wp::float32> var_224;
    const wp::int32 var_225 = 1;
    bool var_226;
    wp::vec_t<3, wp::float32> var_227;
    wp::vec_t<3, wp::float32> var_228;
    const wp::int32 var_229 = 2;
    bool var_230;
    wp::vec_t<3, wp::float32> var_231;
    wp::vec_t<3, wp::float32> var_232;
    wp::vec_t<3, wp::float32> var_233;
    wp::vec_t<3, wp::float32> var_234;
    bool var_235;
    const bool var_236 = true;
    const wp::int32 var_237 = 2;
    wp::vec_t<3, wp::float32> var_238;
    const wp::int32 var_239 = 1;
    bool var_240;
    wp::vec_t<3, wp::float32> var_241;
    wp::vec_t<3, wp::float32> var_242;
    const wp::int32 var_243 = 2;
    bool var_244;
    wp::vec_t<3, wp::float32> var_245;
    wp::vec_t<3, wp::float32> var_246;
    wp::vec_t<3, wp::float32> var_247;
    wp::vec_t<3, wp::float32> var_248;
    bool var_249;
    const bool var_250 = true;
    const wp::int32 var_251 = 0;
    wp::vec_t<3, wp::float32> var_252;
    const wp::int32 var_253 = 1;
    bool var_254;
    wp::vec_t<3, wp::float32> var_255;
    wp::vec_t<3, wp::float32> var_256;
    const wp::int32 var_257 = 2;
    bool var_258;
    wp::vec_t<3, wp::float32> var_259;
    wp::vec_t<3, wp::float32> var_260;
    wp::vec_t<3, wp::float32> var_261;
    wp::vec_t<3, wp::float32> var_262;
    bool var_263;
    const bool var_264 = true;
    const wp::int32 var_265 = 1;
    wp::vec_t<3, wp::float32> var_266;
    const wp::int32 var_267 = 1;
    bool var_268;
    wp::vec_t<3, wp::float32> var_269;
    wp::vec_t<3, wp::float32> var_270;
    const wp::int32 var_271 = 2;
    bool var_272;
    wp::vec_t<3, wp::float32> var_273;
    wp::vec_t<3, wp::float32> var_274;
    wp::vec_t<3, wp::float32> var_275;
    wp::vec_t<3, wp::float32> var_276;
    bool var_277;
    const bool var_278 = true;
    const wp::int32 var_279 = 2;
    wp::vec_t<3, wp::float32> var_280;
    const wp::int32 var_281 = 1;
    bool var_282;
    wp::vec_t<3, wp::float32> var_283;
    wp::vec_t<3, wp::float32> var_284;
    const wp::int32 var_285 = 2;
    bool var_286;
    wp::vec_t<3, wp::float32> var_287;
    wp::vec_t<3, wp::float32> var_288;
    wp::vec_t<3, wp::float32> var_289;
    wp::vec_t<3, wp::float32> var_290;
    bool var_291;
    const bool var_292 = true;
    const wp::int32 var_293 = 0;
    wp::vec_t<3, wp::float32> var_294;
    const wp::int32 var_295 = 1;
    bool var_296;
    wp::vec_t<3, wp::float32> var_297;
    wp::vec_t<3, wp::float32> var_298;
    const wp::int32 var_299 = 2;
    bool var_300;
    wp::vec_t<3, wp::float32> var_301;
    wp::vec_t<3, wp::float32> var_302;
    wp::vec_t<3, wp::float32> var_303;
    const wp::int32 var_304 = 0;
    wp::vec_t<3, wp::float32> var_305;
    const wp::int32 var_306 = 1;
    bool var_307;
    wp::vec_t<3, wp::float32> var_308;
    wp::vec_t<3, wp::float32> var_309;
    const wp::int32 var_310 = 2;
    bool var_311;
    wp::vec_t<3, wp::float32> var_312;
    wp::vec_t<3, wp::float32> var_313;
    wp::vec_t<3, wp::float32> var_314;
    wp::vec_t<3, wp::float32> var_315;
    bool var_316;
    const bool var_317 = true;
    const wp::int32 var_318 = 1;
    wp::vec_t<3, wp::float32> var_319;
    const wp::int32 var_320 = 1;
    bool var_321;
    wp::vec_t<3, wp::float32> var_322;
    wp::vec_t<3, wp::float32> var_323;
    const wp::int32 var_324 = 2;
    bool var_325;
    wp::vec_t<3, wp::float32> var_326;
    wp::vec_t<3, wp::float32> var_327;
    wp::vec_t<3, wp::float32> var_328;
    wp::vec_t<3, wp::float32> var_329;
    bool var_330;
    const bool var_331 = true;
    const wp::int32 var_332 = 2;
    wp::vec_t<3, wp::float32> var_333;
    const wp::int32 var_334 = 1;
    bool var_335;
    wp::vec_t<3, wp::float32> var_336;
    wp::vec_t<3, wp::float32> var_337;
    const wp::int32 var_338 = 2;
    bool var_339;
    wp::vec_t<3, wp::float32> var_340;
    wp::vec_t<3, wp::float32> var_341;
    wp::vec_t<3, wp::float32> var_342;
    wp::vec_t<3, wp::float32> var_343;
    bool var_344;
    const bool var_345 = true;
    const wp::int32 var_346 = 1;
    wp::vec_t<3, wp::float32> var_347;
    const wp::int32 var_348 = 1;
    bool var_349;
    wp::vec_t<3, wp::float32> var_350;
    wp::vec_t<3, wp::float32> var_351;
    const wp::int32 var_352 = 2;
    bool var_353;
    wp::vec_t<3, wp::float32> var_354;
    wp::vec_t<3, wp::float32> var_355;
    wp::vec_t<3, wp::float32> var_356;
    const wp::int32 var_357 = 0;
    wp::vec_t<3, wp::float32> var_358;
    const wp::int32 var_359 = 1;
    bool var_360;
    wp::vec_t<3, wp::float32> var_361;
    wp::vec_t<3, wp::float32> var_362;
    const wp::int32 var_363 = 2;
    bool var_364;
    wp::vec_t<3, wp::float32> var_365;
    wp::vec_t<3, wp::float32> var_366;
    wp::vec_t<3, wp::float32> var_367;
    wp::vec_t<3, wp::float32> var_368;
    bool var_369;
    const bool var_370 = true;
    const wp::int32 var_371 = 1;
    wp::vec_t<3, wp::float32> var_372;
    const wp::int32 var_373 = 1;
    bool var_374;
    wp::vec_t<3, wp::float32> var_375;
    wp::vec_t<3, wp::float32> var_376;
    const wp::int32 var_377 = 2;
    bool var_378;
    wp::vec_t<3, wp::float32> var_379;
    wp::vec_t<3, wp::float32> var_380;
    wp::vec_t<3, wp::float32> var_381;
    wp::vec_t<3, wp::float32> var_382;
    bool var_383;
    const bool var_384 = true;
    const wp::int32 var_385 = 2;
    wp::vec_t<3, wp::float32> var_386;
    const wp::int32 var_387 = 1;
    bool var_388;
    wp::vec_t<3, wp::float32> var_389;
    wp::vec_t<3, wp::float32> var_390;
    const wp::int32 var_391 = 2;
    bool var_392;
    wp::vec_t<3, wp::float32> var_393;
    wp::vec_t<3, wp::float32> var_394;
    wp::vec_t<3, wp::float32> var_395;
    wp::vec_t<3, wp::float32> var_396;
    bool var_397;
    const bool var_398 = true;
    const wp::int32 var_399 = 2;
    wp::vec_t<3, wp::float32> var_400;
    const wp::int32 var_401 = 1;
    bool var_402;
    wp::vec_t<3, wp::float32> var_403;
    wp::vec_t<3, wp::float32> var_404;
    const wp::int32 var_405 = 2;
    bool var_406;
    wp::vec_t<3, wp::float32> var_407;
    wp::vec_t<3, wp::float32> var_408;
    wp::vec_t<3, wp::float32> var_409;
    const wp::int32 var_410 = 0;
    wp::vec_t<3, wp::float32> var_411;
    const wp::int32 var_412 = 1;
    bool var_413;
    wp::vec_t<3, wp::float32> var_414;
    wp::vec_t<3, wp::float32> var_415;
    const wp::int32 var_416 = 2;
    bool var_417;
    wp::vec_t<3, wp::float32> var_418;
    wp::vec_t<3, wp::float32> var_419;
    wp::vec_t<3, wp::float32> var_420;
    wp::vec_t<3, wp::float32> var_421;
    bool var_422;
    const bool var_423 = true;
    const wp::int32 var_424 = 1;
    wp::vec_t<3, wp::float32> var_425;
    const wp::int32 var_426 = 1;
    bool var_427;
    wp::vec_t<3, wp::float32> var_428;
    wp::vec_t<3, wp::float32> var_429;
    const wp::int32 var_430 = 2;
    bool var_431;
    wp::vec_t<3, wp::float32> var_432;
    wp::vec_t<3, wp::float32> var_433;
    wp::vec_t<3, wp::float32> var_434;
    wp::vec_t<3, wp::float32> var_435;
    bool var_436;
    const bool var_437 = true;
    const wp::int32 var_438 = 2;
    wp::vec_t<3, wp::float32> var_439;
    const wp::int32 var_440 = 1;
    bool var_441;
    wp::vec_t<3, wp::float32> var_442;
    wp::vec_t<3, wp::float32> var_443;
    const wp::int32 var_444 = 2;
    bool var_445;
    wp::vec_t<3, wp::float32> var_446;
    wp::vec_t<3, wp::float32> var_447;
    wp::vec_t<3, wp::float32> var_448;
    wp::vec_t<3, wp::float32> var_449;
    bool var_450;
    const bool var_451 = true;
    const wp::int32 var_452 = 0;
    wp::vec_t<3, wp::float32> var_453;
    const wp::int32 var_454 = 1;
    bool var_455;
    wp::vec_t<3, wp::float32> var_456;
    wp::vec_t<3, wp::float32> var_457;
    const wp::int32 var_458 = 2;
    bool var_459;
    wp::vec_t<3, wp::float32> var_460;
    wp::vec_t<3, wp::float32> var_461;
    wp::vec_t<3, wp::float32> var_462;
    const wp::int32 var_463 = 0;
    wp::vec_t<3, wp::float32> var_464;
    wp::vec_t<3, wp::float32> var_465;
    const wp::int32 var_466 = 1;
    bool var_467;
    wp::vec_t<3, wp::float32> var_468;
    wp::vec_t<3, wp::float32> var_469;
    wp::vec_t<3, wp::float32> var_470;
    wp::vec_t<3, wp::float32> var_471;
    const wp::int32 var_472 = 2;
    bool var_473;
    wp::vec_t<3, wp::float32> var_474;
    wp::vec_t<3, wp::float32> var_475;
    wp::vec_t<3, wp::float32> var_476;
    wp::vec_t<3, wp::float32> var_477;
    wp::vec_t<3, wp::float32> var_478;
    wp::vec_t<3, wp::float32> var_479;
    wp::vec_t<3, wp::float32> var_480;
    bool var_481;
    const bool var_482 = true;
    const wp::int32 var_483 = 1;
    wp::vec_t<3, wp::float32> var_484;
    wp::vec_t<3, wp::float32> var_485;
    const wp::int32 var_486 = 1;
    bool var_487;
    wp::vec_t<3, wp::float32> var_488;
    wp::vec_t<3, wp::float32> var_489;
    wp::vec_t<3, wp::float32> var_490;
    wp::vec_t<3, wp::float32> var_491;
    const wp::int32 var_492 = 2;
    bool var_493;
    wp::vec_t<3, wp::float32> var_494;
    wp::vec_t<3, wp::float32> var_495;
    wp::vec_t<3, wp::float32> var_496;
    wp::vec_t<3, wp::float32> var_497;
    wp::vec_t<3, wp::float32> var_498;
    wp::vec_t<3, wp::float32> var_499;
    wp::vec_t<3, wp::float32> var_500;
    bool var_501;
    const bool var_502 = true;
    const wp::int32 var_503 = 2;
    wp::vec_t<3, wp::float32> var_504;
    wp::vec_t<3, wp::float32> var_505;
    const wp::int32 var_506 = 1;
    bool var_507;
    wp::vec_t<3, wp::float32> var_508;
    wp::vec_t<3, wp::float32> var_509;
    wp::vec_t<3, wp::float32> var_510;
    wp::vec_t<3, wp::float32> var_511;
    const wp::int32 var_512 = 2;
    bool var_513;
    wp::vec_t<3, wp::float32> var_514;
    wp::vec_t<3, wp::float32> var_515;
    wp::vec_t<3, wp::float32> var_516;
    wp::vec_t<3, wp::float32> var_517;
    wp::vec_t<3, wp::float32> var_518;
    wp::vec_t<3, wp::float32> var_519;
    wp::vec_t<3, wp::float32> var_520;
    bool var_521;
    const bool var_522 = true;
    const wp::int32 var_523 = 1;
    wp::vec_t<3, wp::float32> var_524;
    const wp::int32 var_525 = 1;
    bool var_526;
    wp::vec_t<3, wp::float32> var_527;
    wp::vec_t<3, wp::float32> var_528;
    const wp::int32 var_529 = 2;
    bool var_530;
    wp::vec_t<3, wp::float32> var_531;
    wp::vec_t<3, wp::float32> var_532;
    wp::vec_t<3, wp::float32> var_533;
    const wp::int32 var_534 = 0;
    wp::vec_t<3, wp::float32> var_535;
    wp::vec_t<3, wp::float32> var_536;
    const wp::int32 var_537 = 1;
    bool var_538;
    wp::vec_t<3, wp::float32> var_539;
    wp::vec_t<3, wp::float32> var_540;
    wp::vec_t<3, wp::float32> var_541;
    wp::vec_t<3, wp::float32> var_542;
    const wp::int32 var_543 = 2;
    bool var_544;
    wp::vec_t<3, wp::float32> var_545;
    wp::vec_t<3, wp::float32> var_546;
    wp::vec_t<3, wp::float32> var_547;
    wp::vec_t<3, wp::float32> var_548;
    wp::vec_t<3, wp::float32> var_549;
    wp::vec_t<3, wp::float32> var_550;
    wp::vec_t<3, wp::float32> var_551;
    bool var_552;
    const bool var_553 = true;
    const wp::int32 var_554 = 1;
    wp::vec_t<3, wp::float32> var_555;
    wp::vec_t<3, wp::float32> var_556;
    const wp::int32 var_557 = 1;
    bool var_558;
    wp::vec_t<3, wp::float32> var_559;
    wp::vec_t<3, wp::float32> var_560;
    wp::vec_t<3, wp::float32> var_561;
    wp::vec_t<3, wp::float32> var_562;
    const wp::int32 var_563 = 2;
    bool var_564;
    wp::vec_t<3, wp::float32> var_565;
    wp::vec_t<3, wp::float32> var_566;
    wp::vec_t<3, wp::float32> var_567;
    wp::vec_t<3, wp::float32> var_568;
    wp::vec_t<3, wp::float32> var_569;
    wp::vec_t<3, wp::float32> var_570;
    wp::vec_t<3, wp::float32> var_571;
    bool var_572;
    const bool var_573 = true;
    const wp::int32 var_574 = 2;
    wp::vec_t<3, wp::float32> var_575;
    wp::vec_t<3, wp::float32> var_576;
    const wp::int32 var_577 = 1;
    bool var_578;
    wp::vec_t<3, wp::float32> var_579;
    wp::vec_t<3, wp::float32> var_580;
    wp::vec_t<3, wp::float32> var_581;
    wp::vec_t<3, wp::float32> var_582;
    const wp::int32 var_583 = 2;
    bool var_584;
    wp::vec_t<3, wp::float32> var_585;
    wp::vec_t<3, wp::float32> var_586;
    wp::vec_t<3, wp::float32> var_587;
    wp::vec_t<3, wp::float32> var_588;
    wp::vec_t<3, wp::float32> var_589;
    wp::vec_t<3, wp::float32> var_590;
    wp::vec_t<3, wp::float32> var_591;
    bool var_592;
    const bool var_593 = true;
    const wp::int32 var_594 = 2;
    wp::vec_t<3, wp::float32> var_595;
    const wp::int32 var_596 = 1;
    bool var_597;
    wp::vec_t<3, wp::float32> var_598;
    wp::vec_t<3, wp::float32> var_599;
    const wp::int32 var_600 = 2;
    bool var_601;
    wp::vec_t<3, wp::float32> var_602;
    wp::vec_t<3, wp::float32> var_603;
    wp::vec_t<3, wp::float32> var_604;
    const wp::int32 var_605 = 0;
    wp::vec_t<3, wp::float32> var_606;
    wp::vec_t<3, wp::float32> var_607;
    const wp::int32 var_608 = 1;
    bool var_609;
    wp::vec_t<3, wp::float32> var_610;
    wp::vec_t<3, wp::float32> var_611;
    wp::vec_t<3, wp::float32> var_612;
    wp::vec_t<3, wp::float32> var_613;
    const wp::int32 var_614 = 2;
    bool var_615;
    wp::vec_t<3, wp::float32> var_616;
    wp::vec_t<3, wp::float32> var_617;
    wp::vec_t<3, wp::float32> var_618;
    wp::vec_t<3, wp::float32> var_619;
    wp::vec_t<3, wp::float32> var_620;
    wp::vec_t<3, wp::float32> var_621;
    wp::vec_t<3, wp::float32> var_622;
    bool var_623;
    const bool var_624 = true;
    const wp::int32 var_625 = 1;
    wp::vec_t<3, wp::float32> var_626;
    wp::vec_t<3, wp::float32> var_627;
    const wp::int32 var_628 = 1;
    bool var_629;
    wp::vec_t<3, wp::float32> var_630;
    wp::vec_t<3, wp::float32> var_631;
    wp::vec_t<3, wp::float32> var_632;
    wp::vec_t<3, wp::float32> var_633;
    const wp::int32 var_634 = 2;
    bool var_635;
    wp::vec_t<3, wp::float32> var_636;
    wp::vec_t<3, wp::float32> var_637;
    wp::vec_t<3, wp::float32> var_638;
    wp::vec_t<3, wp::float32> var_639;
    wp::vec_t<3, wp::float32> var_640;
    wp::vec_t<3, wp::float32> var_641;
    wp::vec_t<3, wp::float32> var_642;
    bool var_643;
    const bool var_644 = true;
    const wp::int32 var_645 = 2;
    wp::vec_t<3, wp::float32> var_646;
    wp::vec_t<3, wp::float32> var_647;
    const wp::int32 var_648 = 1;
    bool var_649;
    wp::vec_t<3, wp::float32> var_650;
    wp::vec_t<3, wp::float32> var_651;
    wp::vec_t<3, wp::float32> var_652;
    wp::vec_t<3, wp::float32> var_653;
    const wp::int32 var_654 = 2;
    bool var_655;
    wp::vec_t<3, wp::float32> var_656;
    wp::vec_t<3, wp::float32> var_657;
    wp::vec_t<3, wp::float32> var_658;
    wp::vec_t<3, wp::float32> var_659;
    wp::vec_t<3, wp::float32> var_660;
    wp::vec_t<3, wp::float32> var_661;
    wp::vec_t<3, wp::float32> var_662;
    bool var_663;
    const bool var_664 = true;
    const wp::int32 var_665 = 0;
    wp::vec_t<3, wp::float32> var_666;
    const wp::int32 var_667 = 1;
    bool var_668;
    wp::vec_t<3, wp::float32> var_669;
    wp::vec_t<3, wp::float32> var_670;
    const wp::int32 var_671 = 2;
    bool var_672;
    wp::vec_t<3, wp::float32> var_673;
    wp::vec_t<3, wp::float32> var_674;
    wp::vec_t<3, wp::float32> var_675;
    const wp::int32 var_676 = 0;
    wp::vec_t<3, wp::float32> var_677;
    wp::vec_t<3, wp::float32> var_678;
    const wp::int32 var_679 = 1;
    bool var_680;
    wp::vec_t<3, wp::float32> var_681;
    wp::vec_t<3, wp::float32> var_682;
    wp::vec_t<3, wp::float32> var_683;
    wp::vec_t<3, wp::float32> var_684;
    const wp::int32 var_685 = 2;
    bool var_686;
    wp::vec_t<3, wp::float32> var_687;
    wp::vec_t<3, wp::float32> var_688;
    wp::vec_t<3, wp::float32> var_689;
    wp::vec_t<3, wp::float32> var_690;
    wp::vec_t<3, wp::float32> var_691;
    wp::vec_t<3, wp::float32> var_692;
    wp::vec_t<3, wp::float32> var_693;
    bool var_694;
    const bool var_695 = true;
    const wp::int32 var_696 = 1;
    wp::vec_t<3, wp::float32> var_697;
    wp::vec_t<3, wp::float32> var_698;
    const wp::int32 var_699 = 1;
    bool var_700;
    wp::vec_t<3, wp::float32> var_701;
    wp::vec_t<3, wp::float32> var_702;
    wp::vec_t<3, wp::float32> var_703;
    wp::vec_t<3, wp::float32> var_704;
    const wp::int32 var_705 = 2;
    bool var_706;
    wp::vec_t<3, wp::float32> var_707;
    wp::vec_t<3, wp::float32> var_708;
    wp::vec_t<3, wp::float32> var_709;
    wp::vec_t<3, wp::float32> var_710;
    wp::vec_t<3, wp::float32> var_711;
    wp::vec_t<3, wp::float32> var_712;
    wp::vec_t<3, wp::float32> var_713;
    bool var_714;
    const bool var_715 = true;
    const wp::int32 var_716 = 2;
    wp::vec_t<3, wp::float32> var_717;
    wp::vec_t<3, wp::float32> var_718;
    const wp::int32 var_719 = 1;
    bool var_720;
    wp::vec_t<3, wp::float32> var_721;
    wp::vec_t<3, wp::float32> var_722;
    wp::vec_t<3, wp::float32> var_723;
    wp::vec_t<3, wp::float32> var_724;
    const wp::int32 var_725 = 2;
    bool var_726;
    wp::vec_t<3, wp::float32> var_727;
    wp::vec_t<3, wp::float32> var_728;
    wp::vec_t<3, wp::float32> var_729;
    wp::vec_t<3, wp::float32> var_730;
    wp::vec_t<3, wp::float32> var_731;
    wp::vec_t<3, wp::float32> var_732;
    wp::vec_t<3, wp::float32> var_733;
    bool var_734;
    const bool var_735 = true;
    const wp::int32 var_736 = 1;
    wp::vec_t<3, wp::float32> var_737;
    const wp::int32 var_738 = 1;
    bool var_739;
    wp::vec_t<3, wp::float32> var_740;
    wp::vec_t<3, wp::float32> var_741;
    const wp::int32 var_742 = 2;
    bool var_743;
    wp::vec_t<3, wp::float32> var_744;
    wp::vec_t<3, wp::float32> var_745;
    wp::vec_t<3, wp::float32> var_746;
    const wp::int32 var_747 = 0;
    wp::vec_t<3, wp::float32> var_748;
    wp::vec_t<3, wp::float32> var_749;
    const wp::int32 var_750 = 1;
    bool var_751;
    wp::vec_t<3, wp::float32> var_752;
    wp::vec_t<3, wp::float32> var_753;
    wp::vec_t<3, wp::float32> var_754;
    wp::vec_t<3, wp::float32> var_755;
    const wp::int32 var_756 = 2;
    bool var_757;
    wp::vec_t<3, wp::float32> var_758;
    wp::vec_t<3, wp::float32> var_759;
    wp::vec_t<3, wp::float32> var_760;
    wp::vec_t<3, wp::float32> var_761;
    wp::vec_t<3, wp::float32> var_762;
    wp::vec_t<3, wp::float32> var_763;
    wp::vec_t<3, wp::float32> var_764;
    bool var_765;
    const bool var_766 = true;
    const wp::int32 var_767 = 1;
    wp::vec_t<3, wp::float32> var_768;
    wp::vec_t<3, wp::float32> var_769;
    const wp::int32 var_770 = 1;
    bool var_771;
    wp::vec_t<3, wp::float32> var_772;
    wp::vec_t<3, wp::float32> var_773;
    wp::vec_t<3, wp::float32> var_774;
    wp::vec_t<3, wp::float32> var_775;
    const wp::int32 var_776 = 2;
    bool var_777;
    wp::vec_t<3, wp::float32> var_778;
    wp::vec_t<3, wp::float32> var_779;
    wp::vec_t<3, wp::float32> var_780;
    wp::vec_t<3, wp::float32> var_781;
    wp::vec_t<3, wp::float32> var_782;
    wp::vec_t<3, wp::float32> var_783;
    wp::vec_t<3, wp::float32> var_784;
    bool var_785;
    const bool var_786 = true;
    const wp::int32 var_787 = 2;
    wp::vec_t<3, wp::float32> var_788;
    wp::vec_t<3, wp::float32> var_789;
    const wp::int32 var_790 = 1;
    bool var_791;
    wp::vec_t<3, wp::float32> var_792;
    wp::vec_t<3, wp::float32> var_793;
    wp::vec_t<3, wp::float32> var_794;
    wp::vec_t<3, wp::float32> var_795;
    const wp::int32 var_796 = 2;
    bool var_797;
    wp::vec_t<3, wp::float32> var_798;
    wp::vec_t<3, wp::float32> var_799;
    wp::vec_t<3, wp::float32> var_800;
    wp::vec_t<3, wp::float32> var_801;
    wp::vec_t<3, wp::float32> var_802;
    wp::vec_t<3, wp::float32> var_803;
    wp::vec_t<3, wp::float32> var_804;
    bool var_805;
    const bool var_806 = true;
    const wp::int32 var_807 = 2;
    wp::vec_t<3, wp::float32> var_808;
    const wp::int32 var_809 = 1;
    bool var_810;
    wp::vec_t<3, wp::float32> var_811;
    wp::vec_t<3, wp::float32> var_812;
    const wp::int32 var_813 = 2;
    bool var_814;
    wp::vec_t<3, wp::float32> var_815;
    wp::vec_t<3, wp::float32> var_816;
    wp::vec_t<3, wp::float32> var_817;
    const wp::int32 var_818 = 0;
    wp::vec_t<3, wp::float32> var_819;
    wp::vec_t<3, wp::float32> var_820;
    const wp::int32 var_821 = 1;
    bool var_822;
    wp::vec_t<3, wp::float32> var_823;
    wp::vec_t<3, wp::float32> var_824;
    wp::vec_t<3, wp::float32> var_825;
    wp::vec_t<3, wp::float32> var_826;
    const wp::int32 var_827 = 2;
    bool var_828;
    wp::vec_t<3, wp::float32> var_829;
    wp::vec_t<3, wp::float32> var_830;
    wp::vec_t<3, wp::float32> var_831;
    wp::vec_t<3, wp::float32> var_832;
    wp::vec_t<3, wp::float32> var_833;
    wp::vec_t<3, wp::float32> var_834;
    wp::vec_t<3, wp::float32> var_835;
    bool var_836;
    const bool var_837 = true;
    const wp::int32 var_838 = 1;
    wp::vec_t<3, wp::float32> var_839;
    wp::vec_t<3, wp::float32> var_840;
    const wp::int32 var_841 = 1;
    bool var_842;
    wp::vec_t<3, wp::float32> var_843;
    wp::vec_t<3, wp::float32> var_844;
    wp::vec_t<3, wp::float32> var_845;
    wp::vec_t<3, wp::float32> var_846;
    const wp::int32 var_847 = 2;
    bool var_848;
    wp::vec_t<3, wp::float32> var_849;
    wp::vec_t<3, wp::float32> var_850;
    wp::vec_t<3, wp::float32> var_851;
    wp::vec_t<3, wp::float32> var_852;
    wp::vec_t<3, wp::float32> var_853;
    wp::vec_t<3, wp::float32> var_854;
    wp::vec_t<3, wp::float32> var_855;
    bool var_856;
    const bool var_857 = true;
    const wp::int32 var_858 = 2;
    wp::vec_t<3, wp::float32> var_859;
    wp::vec_t<3, wp::float32> var_860;
    const wp::int32 var_861 = 1;
    bool var_862;
    wp::vec_t<3, wp::float32> var_863;
    wp::vec_t<3, wp::float32> var_864;
    wp::vec_t<3, wp::float32> var_865;
    wp::vec_t<3, wp::float32> var_866;
    const wp::int32 var_867 = 2;
    bool var_868;
    wp::vec_t<3, wp::float32> var_869;
    wp::vec_t<3, wp::float32> var_870;
    wp::vec_t<3, wp::float32> var_871;
    wp::vec_t<3, wp::float32> var_872;
    wp::vec_t<3, wp::float32> var_873;
    wp::vec_t<3, wp::float32> var_874;
    wp::vec_t<3, wp::float32> var_875;
    bool var_876;
    const bool var_877 = true;
    const bool var_878 = false;
    //---------
    // forward
    // def _triangle_sat_separated(                                                           <L 105>
    // e1_0 = p1 - p0                                                                         <L 116>
    var_0 = wp::sub(var_p1, var_p0);
    // e1_1 = p2 - p1                                                                         <L 117>
    var_1 = wp::sub(var_p2, var_p1);
    // e1_2 = p0 - p2                                                                         <L 118>
    var_2 = wp::sub(var_p0, var_p2);
    // n1 = wp.cross(e1_0, e1_1)                                                              <L 121>
    var_3 = wp::cross(var_0, var_1);
    // p_proj = wp.dot(p0, n1)                                                                <L 122>
    var_4 = wp::dot(var_p0, var_3);
    // q0_d = wp.dot(q0, n1)                                                                  <L 123>
    var_5 = wp::dot(var_q0, var_3);
    // q1_d = wp.dot(q1, n1)                                                                  <L 124>
    var_6 = wp::dot(var_q1, var_3);
    // q2_d = wp.dot(q2, n1)                                                                  <L 125>
    var_7 = wp::dot(var_q2, var_3);
    // diff1 = wp.max(p_proj - wp.max(q0_d, wp.max(q1_d, q2_d)), wp.min(q0_d, wp.min(q1_d, q2_d)) - p_proj)       <L 126>
    var_8 = wp::max(var_6, var_7);
    var_9 = wp::max(var_5, var_8);
    var_10 = wp::sub(var_4, var_9);
    var_11 = wp::min(var_6, var_7);
    var_12 = wp::min(var_5, var_11);
    var_13 = wp::sub(var_12, var_4);
    var_14 = wp::max(var_10, var_13);
    // if diff1 > 0.0:                                                                        <L 127>
    var_16 = (var_14 > var_15);
    if (var_16) {
        // ln1_sq = wp.length_sq(n1)                                                          <L 128>
        var_17 = wp::length_sq(var_3);
        // if ln1_sq > 1e-12 and diff1 * diff1 > cutoff_sq * ln1_sq:                          <L 129>
        var_20 = (var_17 > var_19);
        var_18 = var_20;
        if (var_18) {
            var_21 = wp::mul(var_14, var_14);
            var_22 = wp::mul(var_cutoff_sq, var_17);
            var_23 = (var_21 > var_22);
            var_18 = var_18 && var_23;
        }
        if (var_18) {
            // return True                                                                    <L 130>
            return var_24;
        }
    }
    // e2_0 = q1 - q0                                                                         <L 133>
    var_25 = wp::sub(var_q1, var_q0);
    // e2_1 = q2 - q1                                                                         <L 134>
    var_26 = wp::sub(var_q2, var_q1);
    // e2_2 = q0 - q2                                                                         <L 135>
    var_27 = wp::sub(var_q0, var_q2);
    // n2 = wp.cross(e2_0, e2_1)                                                              <L 136>
    var_28 = wp::cross(var_25, var_26);
    // q_proj = wp.dot(q0, n2)                                                                <L 137>
    var_29 = wp::dot(var_q0, var_28);
    // p0_d = wp.dot(p0, n2)                                                                  <L 138>
    var_30 = wp::dot(var_p0, var_28);
    // p1_d = wp.dot(p1, n2)                                                                  <L 139>
    var_31 = wp::dot(var_p1, var_28);
    // p2_d = wp.dot(p2, n2)                                                                  <L 140>
    var_32 = wp::dot(var_p2, var_28);
    // diff2 = wp.max(wp.min(p0_d, wp.min(p1_d, p2_d)) - q_proj, q_proj - wp.max(p0_d, wp.max(p1_d, p2_d)))       <L 141>
    var_33 = wp::min(var_31, var_32);
    var_34 = wp::min(var_30, var_33);
    var_35 = wp::sub(var_34, var_29);
    var_36 = wp::max(var_31, var_32);
    var_37 = wp::max(var_30, var_36);
    var_38 = wp::sub(var_29, var_37);
    var_39 = wp::max(var_35, var_38);
    // if diff2 > 0.0:                                                                        <L 142>
    var_41 = (var_39 > var_40);
    if (var_41) {
        // ln2_sq = wp.length_sq(n2)                                                          <L 143>
        var_42 = wp::length_sq(var_28);
        // if ln2_sq > 1e-12 and diff2 * diff2 > cutoff_sq * ln2_sq:                          <L 144>
        var_45 = (var_42 > var_44);
        var_43 = var_45;
        if (var_43) {
            var_46 = wp::mul(var_39, var_39);
            var_47 = wp::mul(var_cutoff_sq, var_42);
            var_48 = (var_46 > var_47);
            var_43 = var_43 && var_48;
        }
        if (var_43) {
            // return True                                                                    <L 145>
            return var_49;
        }
    }
    // for i in range(3):                                                                     <L 148>
    // e1 = e1_0                                                                              <L 149>
    var_51 = wp::copy(var_0);
    // if i == 1:                                                                             <L 150>
    var_53 = (var_50 == var_52);
    if (var_53) {
        // e1 = e1_1                                                                          <L 151>
        var_54 = wp::copy(var_1);
    }
    var_55 = wp::where(var_53, var_54, var_51);
    if (!var_53) {
        // elif i == 2:                                                                       <L 152>
        var_57 = (var_50 == var_56);
        if (var_57) {
            // e1 = e1_2                                                                      <L 153>
            var_58 = wp::copy(var_2);
        }
        var_59 = wp::where(var_57, var_58, var_55);
    }
    var_60 = wp::where(var_53, var_55, var_59);
    // for j in range(3):                                                                     <L 154>
    // e2 = e2_0                                                                              <L 155>
    var_62 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_64 = (var_61 == var_63);
    if (var_64) {
        // e2 = e2_1                                                                          <L 157>
        var_65 = wp::copy(var_26);
    }
    var_66 = wp::where(var_64, var_65, var_62);
    if (!var_64) {
        // elif j == 2:                                                                       <L 158>
        var_68 = (var_61 == var_67);
        if (var_68) {
            // e2 = e2_2                                                                      <L 159>
            var_69 = wp::copy(var_27);
        }
        var_70 = wp::where(var_68, var_69, var_66);
    }
    var_71 = wp::where(var_64, var_66, var_70);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_72 = wp::cross(var_60, var_71);
    var_73 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_72, var_cutoff_sq);
    if (var_73) {
        // return True                                                                        <L 161>
        return var_74;
    }
    // e2 = e2_0                                                                              <L 155>
    var_76 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_78 = (var_75 == var_77);
    if (var_78) {
        // e2 = e2_1                                                                          <L 157>
        var_79 = wp::copy(var_26);
    }
    var_80 = wp::where(var_78, var_79, var_76);
    if (!var_78) {
        // elif j == 2:                                                                       <L 158>
        var_82 = (var_75 == var_81);
        if (var_82) {
            // e2 = e2_2                                                                      <L 159>
            var_83 = wp::copy(var_27);
        }
        var_84 = wp::where(var_82, var_83, var_80);
    }
    var_85 = wp::where(var_78, var_80, var_84);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_86 = wp::cross(var_60, var_85);
    var_87 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_86, var_cutoff_sq);
    if (var_87) {
        // return True                                                                        <L 161>
        return var_88;
    }
    // e2 = e2_0                                                                              <L 155>
    var_90 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_92 = (var_89 == var_91);
    if (var_92) {
        // e2 = e2_1                                                                          <L 157>
        var_93 = wp::copy(var_26);
    }
    var_94 = wp::where(var_92, var_93, var_90);
    if (!var_92) {
        // elif j == 2:                                                                       <L 158>
        var_96 = (var_89 == var_95);
        if (var_96) {
            // e2 = e2_2                                                                      <L 159>
            var_97 = wp::copy(var_27);
        }
        var_98 = wp::where(var_96, var_97, var_94);
    }
    var_99 = wp::where(var_92, var_94, var_98);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_100 = wp::cross(var_60, var_99);
    var_101 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_100, var_cutoff_sq);
    if (var_101) {
        // return True                                                                        <L 161>
        return var_102;
    }
    // e1 = e1_0                                                                              <L 149>
    var_104 = wp::copy(var_0);
    // if i == 1:                                                                             <L 150>
    var_106 = (var_103 == var_105);
    if (var_106) {
        // e1 = e1_1                                                                          <L 151>
        var_107 = wp::copy(var_1);
    }
    var_108 = wp::where(var_106, var_107, var_104);
    if (!var_106) {
        // elif i == 2:                                                                       <L 152>
        var_110 = (var_103 == var_109);
        if (var_110) {
            // e1 = e1_2                                                                      <L 153>
            var_111 = wp::copy(var_2);
        }
        var_112 = wp::where(var_110, var_111, var_108);
    }
    var_113 = wp::where(var_106, var_108, var_112);
    // for j in range(3):                                                                     <L 154>
    // e2 = e2_0                                                                              <L 155>
    var_115 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_117 = (var_114 == var_116);
    if (var_117) {
        // e2 = e2_1                                                                          <L 157>
        var_118 = wp::copy(var_26);
    }
    var_119 = wp::where(var_117, var_118, var_115);
    if (!var_117) {
        // elif j == 2:                                                                       <L 158>
        var_121 = (var_114 == var_120);
        if (var_121) {
            // e2 = e2_2                                                                      <L 159>
            var_122 = wp::copy(var_27);
        }
        var_123 = wp::where(var_121, var_122, var_119);
    }
    var_124 = wp::where(var_117, var_119, var_123);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_125 = wp::cross(var_113, var_124);
    var_126 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_125, var_cutoff_sq);
    if (var_126) {
        // return True                                                                        <L 161>
        return var_127;
    }
    // e2 = e2_0                                                                              <L 155>
    var_129 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_131 = (var_128 == var_130);
    if (var_131) {
        // e2 = e2_1                                                                          <L 157>
        var_132 = wp::copy(var_26);
    }
    var_133 = wp::where(var_131, var_132, var_129);
    if (!var_131) {
        // elif j == 2:                                                                       <L 158>
        var_135 = (var_128 == var_134);
        if (var_135) {
            // e2 = e2_2                                                                      <L 159>
            var_136 = wp::copy(var_27);
        }
        var_137 = wp::where(var_135, var_136, var_133);
    }
    var_138 = wp::where(var_131, var_133, var_137);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_139 = wp::cross(var_113, var_138);
    var_140 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_139, var_cutoff_sq);
    if (var_140) {
        // return True                                                                        <L 161>
        return var_141;
    }
    // e2 = e2_0                                                                              <L 155>
    var_143 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_145 = (var_142 == var_144);
    if (var_145) {
        // e2 = e2_1                                                                          <L 157>
        var_146 = wp::copy(var_26);
    }
    var_147 = wp::where(var_145, var_146, var_143);
    if (!var_145) {
        // elif j == 2:                                                                       <L 158>
        var_149 = (var_142 == var_148);
        if (var_149) {
            // e2 = e2_2                                                                      <L 159>
            var_150 = wp::copy(var_27);
        }
        var_151 = wp::where(var_149, var_150, var_147);
    }
    var_152 = wp::where(var_145, var_147, var_151);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_153 = wp::cross(var_113, var_152);
    var_154 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_153, var_cutoff_sq);
    if (var_154) {
        // return True                                                                        <L 161>
        return var_155;
    }
    // e1 = e1_0                                                                              <L 149>
    var_157 = wp::copy(var_0);
    // if i == 1:                                                                             <L 150>
    var_159 = (var_156 == var_158);
    if (var_159) {
        // e1 = e1_1                                                                          <L 151>
        var_160 = wp::copy(var_1);
    }
    var_161 = wp::where(var_159, var_160, var_157);
    if (!var_159) {
        // elif i == 2:                                                                       <L 152>
        var_163 = (var_156 == var_162);
        if (var_163) {
            // e1 = e1_2                                                                      <L 153>
            var_164 = wp::copy(var_2);
        }
        var_165 = wp::where(var_163, var_164, var_161);
    }
    var_166 = wp::where(var_159, var_161, var_165);
    // for j in range(3):                                                                     <L 154>
    // e2 = e2_0                                                                              <L 155>
    var_168 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_170 = (var_167 == var_169);
    if (var_170) {
        // e2 = e2_1                                                                          <L 157>
        var_171 = wp::copy(var_26);
    }
    var_172 = wp::where(var_170, var_171, var_168);
    if (!var_170) {
        // elif j == 2:                                                                       <L 158>
        var_174 = (var_167 == var_173);
        if (var_174) {
            // e2 = e2_2                                                                      <L 159>
            var_175 = wp::copy(var_27);
        }
        var_176 = wp::where(var_174, var_175, var_172);
    }
    var_177 = wp::where(var_170, var_172, var_176);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_178 = wp::cross(var_166, var_177);
    var_179 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_178, var_cutoff_sq);
    if (var_179) {
        // return True                                                                        <L 161>
        return var_180;
    }
    // e2 = e2_0                                                                              <L 155>
    var_182 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_184 = (var_181 == var_183);
    if (var_184) {
        // e2 = e2_1                                                                          <L 157>
        var_185 = wp::copy(var_26);
    }
    var_186 = wp::where(var_184, var_185, var_182);
    if (!var_184) {
        // elif j == 2:                                                                       <L 158>
        var_188 = (var_181 == var_187);
        if (var_188) {
            // e2 = e2_2                                                                      <L 159>
            var_189 = wp::copy(var_27);
        }
        var_190 = wp::where(var_188, var_189, var_186);
    }
    var_191 = wp::where(var_184, var_186, var_190);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_192 = wp::cross(var_166, var_191);
    var_193 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_192, var_cutoff_sq);
    if (var_193) {
        // return True                                                                        <L 161>
        return var_194;
    }
    // e2 = e2_0                                                                              <L 155>
    var_196 = wp::copy(var_25);
    // if j == 1:                                                                             <L 156>
    var_198 = (var_195 == var_197);
    if (var_198) {
        // e2 = e2_1                                                                          <L 157>
        var_199 = wp::copy(var_26);
    }
    var_200 = wp::where(var_198, var_199, var_196);
    if (!var_198) {
        // elif j == 2:                                                                       <L 158>
        var_202 = (var_195 == var_201);
        if (var_202) {
            // e2 = e2_2                                                                      <L 159>
            var_203 = wp::copy(var_27);
        }
        var_204 = wp::where(var_202, var_203, var_200);
    }
    var_205 = wp::where(var_198, var_200, var_204);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e1, e2), cutoff_sq):               <L 160>
    var_206 = wp::cross(var_166, var_205);
    var_207 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_206, var_cutoff_sq);
    if (var_207) {
        // return True                                                                        <L 161>
        return var_208;
    }
    // for i in range(3):                                                                     <L 164>
    // e = e1_0                                                                               <L 165>
    var_210 = wp::copy(var_0);
    // if i == 1:                                                                             <L 166>
    var_212 = (var_209 == var_211);
    if (var_212) {
        // e = e1_1                                                                           <L 167>
        var_213 = wp::copy(var_1);
    }
    var_214 = wp::where(var_212, var_213, var_210);
    if (!var_212) {
        // elif i == 2:                                                                       <L 168>
        var_216 = (var_209 == var_215);
        if (var_216) {
            // e = e1_2                                                                       <L 169>
            var_217 = wp::copy(var_2);
        }
        var_218 = wp::where(var_216, var_217, var_214);
    }
    var_219 = wp::where(var_212, var_214, var_218);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n1), cutoff_sq):                <L 170>
    var_220 = wp::cross(var_219, var_3);
    var_221 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_220, var_cutoff_sq);
    if (var_221) {
        // return True                                                                        <L 171>
        return var_222;
    }
    // e = e1_0                                                                               <L 165>
    var_224 = wp::copy(var_0);
    // if i == 1:                                                                             <L 166>
    var_226 = (var_223 == var_225);
    if (var_226) {
        // e = e1_1                                                                           <L 167>
        var_227 = wp::copy(var_1);
    }
    var_228 = wp::where(var_226, var_227, var_224);
    if (!var_226) {
        // elif i == 2:                                                                       <L 168>
        var_230 = (var_223 == var_229);
        if (var_230) {
            // e = e1_2                                                                       <L 169>
            var_231 = wp::copy(var_2);
        }
        var_232 = wp::where(var_230, var_231, var_228);
    }
    var_233 = wp::where(var_226, var_228, var_232);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n1), cutoff_sq):                <L 170>
    var_234 = wp::cross(var_233, var_3);
    var_235 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_234, var_cutoff_sq);
    if (var_235) {
        // return True                                                                        <L 171>
        return var_236;
    }
    // e = e1_0                                                                               <L 165>
    var_238 = wp::copy(var_0);
    // if i == 1:                                                                             <L 166>
    var_240 = (var_237 == var_239);
    if (var_240) {
        // e = e1_1                                                                           <L 167>
        var_241 = wp::copy(var_1);
    }
    var_242 = wp::where(var_240, var_241, var_238);
    if (!var_240) {
        // elif i == 2:                                                                       <L 168>
        var_244 = (var_237 == var_243);
        if (var_244) {
            // e = e1_2                                                                       <L 169>
            var_245 = wp::copy(var_2);
        }
        var_246 = wp::where(var_244, var_245, var_242);
    }
    var_247 = wp::where(var_240, var_242, var_246);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n1), cutoff_sq):                <L 170>
    var_248 = wp::cross(var_247, var_3);
    var_249 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_248, var_cutoff_sq);
    if (var_249) {
        // return True                                                                        <L 171>
        return var_250;
    }
    // for i in range(3):                                                                     <L 174>
    // e = e2_0                                                                               <L 175>
    var_252 = wp::copy(var_25);
    // if i == 1:                                                                             <L 176>
    var_254 = (var_251 == var_253);
    if (var_254) {
        // e = e2_1                                                                           <L 177>
        var_255 = wp::copy(var_26);
    }
    var_256 = wp::where(var_254, var_255, var_252);
    if (!var_254) {
        // elif i == 2:                                                                       <L 178>
        var_258 = (var_251 == var_257);
        if (var_258) {
            // e = e2_2                                                                       <L 179>
            var_259 = wp::copy(var_27);
        }
        var_260 = wp::where(var_258, var_259, var_256);
    }
    var_261 = wp::where(var_254, var_256, var_260);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n2), cutoff_sq):                <L 180>
    var_262 = wp::cross(var_261, var_28);
    var_263 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_262, var_cutoff_sq);
    if (var_263) {
        // return True                                                                        <L 181>
        return var_264;
    }
    // e = e2_0                                                                               <L 175>
    var_266 = wp::copy(var_25);
    // if i == 1:                                                                             <L 176>
    var_268 = (var_265 == var_267);
    if (var_268) {
        // e = e2_1                                                                           <L 177>
        var_269 = wp::copy(var_26);
    }
    var_270 = wp::where(var_268, var_269, var_266);
    if (!var_268) {
        // elif i == 2:                                                                       <L 178>
        var_272 = (var_265 == var_271);
        if (var_272) {
            // e = e2_2                                                                       <L 179>
            var_273 = wp::copy(var_27);
        }
        var_274 = wp::where(var_272, var_273, var_270);
    }
    var_275 = wp::where(var_268, var_270, var_274);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n2), cutoff_sq):                <L 180>
    var_276 = wp::cross(var_275, var_28);
    var_277 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_276, var_cutoff_sq);
    if (var_277) {
        // return True                                                                        <L 181>
        return var_278;
    }
    // e = e2_0                                                                               <L 175>
    var_280 = wp::copy(var_25);
    // if i == 1:                                                                             <L 176>
    var_282 = (var_279 == var_281);
    if (var_282) {
        // e = e2_1                                                                           <L 177>
        var_283 = wp::copy(var_26);
    }
    var_284 = wp::where(var_282, var_283, var_280);
    if (!var_282) {
        // elif i == 2:                                                                       <L 178>
        var_286 = (var_279 == var_285);
        if (var_286) {
            // e = e2_2                                                                       <L 179>
            var_287 = wp::copy(var_27);
        }
        var_288 = wp::where(var_286, var_287, var_284);
    }
    var_289 = wp::where(var_282, var_284, var_288);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, wp.cross(e, n2), cutoff_sq):                <L 180>
    var_290 = wp::cross(var_289, var_28);
    var_291 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_290, var_cutoff_sq);
    if (var_291) {
        // return True                                                                        <L 181>
        return var_292;
    }
    // for i in range(3):                                                                     <L 184>
    // p = p0                                                                                 <L 185>
    var_294 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 186>
    var_296 = (var_293 == var_295);
    if (var_296) {
        // p = p1                                                                             <L 187>
        var_297 = wp::copy(var_p1);
    }
    var_298 = wp::where(var_296, var_297, var_294);
    if (!var_296) {
        // elif i == 2:                                                                       <L 188>
        var_300 = (var_293 == var_299);
        if (var_300) {
            // p = p2                                                                         <L 189>
            var_301 = wp::copy(var_p2);
        }
        var_302 = wp::where(var_300, var_301, var_298);
    }
    var_303 = wp::where(var_296, var_298, var_302);
    // for j in range(3):                                                                     <L 190>
    // q = q0                                                                                 <L 191>
    var_305 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_307 = (var_304 == var_306);
    if (var_307) {
        // q = q1                                                                             <L 193>
        var_308 = wp::copy(var_q1);
    }
    var_309 = wp::where(var_307, var_308, var_305);
    if (!var_307) {
        // elif j == 2:                                                                       <L 194>
        var_311 = (var_304 == var_310);
        if (var_311) {
            // q = q2                                                                         <L 195>
            var_312 = wp::copy(var_q2);
        }
        var_313 = wp::where(var_311, var_312, var_309);
    }
    var_314 = wp::where(var_307, var_309, var_313);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_315 = wp::sub(var_303, var_314);
    var_316 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_315, var_cutoff_sq);
    if (var_316) {
        // return True                                                                        <L 197>
        return var_317;
    }
    // q = q0                                                                                 <L 191>
    var_319 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_321 = (var_318 == var_320);
    if (var_321) {
        // q = q1                                                                             <L 193>
        var_322 = wp::copy(var_q1);
    }
    var_323 = wp::where(var_321, var_322, var_319);
    if (!var_321) {
        // elif j == 2:                                                                       <L 194>
        var_325 = (var_318 == var_324);
        if (var_325) {
            // q = q2                                                                         <L 195>
            var_326 = wp::copy(var_q2);
        }
        var_327 = wp::where(var_325, var_326, var_323);
    }
    var_328 = wp::where(var_321, var_323, var_327);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_329 = wp::sub(var_303, var_328);
    var_330 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_329, var_cutoff_sq);
    if (var_330) {
        // return True                                                                        <L 197>
        return var_331;
    }
    // q = q0                                                                                 <L 191>
    var_333 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_335 = (var_332 == var_334);
    if (var_335) {
        // q = q1                                                                             <L 193>
        var_336 = wp::copy(var_q1);
    }
    var_337 = wp::where(var_335, var_336, var_333);
    if (!var_335) {
        // elif j == 2:                                                                       <L 194>
        var_339 = (var_332 == var_338);
        if (var_339) {
            // q = q2                                                                         <L 195>
            var_340 = wp::copy(var_q2);
        }
        var_341 = wp::where(var_339, var_340, var_337);
    }
    var_342 = wp::where(var_335, var_337, var_341);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_343 = wp::sub(var_303, var_342);
    var_344 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_343, var_cutoff_sq);
    if (var_344) {
        // return True                                                                        <L 197>
        return var_345;
    }
    // p = p0                                                                                 <L 185>
    var_347 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 186>
    var_349 = (var_346 == var_348);
    if (var_349) {
        // p = p1                                                                             <L 187>
        var_350 = wp::copy(var_p1);
    }
    var_351 = wp::where(var_349, var_350, var_347);
    if (!var_349) {
        // elif i == 2:                                                                       <L 188>
        var_353 = (var_346 == var_352);
        if (var_353) {
            // p = p2                                                                         <L 189>
            var_354 = wp::copy(var_p2);
        }
        var_355 = wp::where(var_353, var_354, var_351);
    }
    var_356 = wp::where(var_349, var_351, var_355);
    // for j in range(3):                                                                     <L 190>
    // q = q0                                                                                 <L 191>
    var_358 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_360 = (var_357 == var_359);
    if (var_360) {
        // q = q1                                                                             <L 193>
        var_361 = wp::copy(var_q1);
    }
    var_362 = wp::where(var_360, var_361, var_358);
    if (!var_360) {
        // elif j == 2:                                                                       <L 194>
        var_364 = (var_357 == var_363);
        if (var_364) {
            // q = q2                                                                         <L 195>
            var_365 = wp::copy(var_q2);
        }
        var_366 = wp::where(var_364, var_365, var_362);
    }
    var_367 = wp::where(var_360, var_362, var_366);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_368 = wp::sub(var_356, var_367);
    var_369 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_368, var_cutoff_sq);
    if (var_369) {
        // return True                                                                        <L 197>
        return var_370;
    }
    // q = q0                                                                                 <L 191>
    var_372 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_374 = (var_371 == var_373);
    if (var_374) {
        // q = q1                                                                             <L 193>
        var_375 = wp::copy(var_q1);
    }
    var_376 = wp::where(var_374, var_375, var_372);
    if (!var_374) {
        // elif j == 2:                                                                       <L 194>
        var_378 = (var_371 == var_377);
        if (var_378) {
            // q = q2                                                                         <L 195>
            var_379 = wp::copy(var_q2);
        }
        var_380 = wp::where(var_378, var_379, var_376);
    }
    var_381 = wp::where(var_374, var_376, var_380);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_382 = wp::sub(var_356, var_381);
    var_383 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_382, var_cutoff_sq);
    if (var_383) {
        // return True                                                                        <L 197>
        return var_384;
    }
    // q = q0                                                                                 <L 191>
    var_386 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_388 = (var_385 == var_387);
    if (var_388) {
        // q = q1                                                                             <L 193>
        var_389 = wp::copy(var_q1);
    }
    var_390 = wp::where(var_388, var_389, var_386);
    if (!var_388) {
        // elif j == 2:                                                                       <L 194>
        var_392 = (var_385 == var_391);
        if (var_392) {
            // q = q2                                                                         <L 195>
            var_393 = wp::copy(var_q2);
        }
        var_394 = wp::where(var_392, var_393, var_390);
    }
    var_395 = wp::where(var_388, var_390, var_394);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_396 = wp::sub(var_356, var_395);
    var_397 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_396, var_cutoff_sq);
    if (var_397) {
        // return True                                                                        <L 197>
        return var_398;
    }
    // p = p0                                                                                 <L 185>
    var_400 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 186>
    var_402 = (var_399 == var_401);
    if (var_402) {
        // p = p1                                                                             <L 187>
        var_403 = wp::copy(var_p1);
    }
    var_404 = wp::where(var_402, var_403, var_400);
    if (!var_402) {
        // elif i == 2:                                                                       <L 188>
        var_406 = (var_399 == var_405);
        if (var_406) {
            // p = p2                                                                         <L 189>
            var_407 = wp::copy(var_p2);
        }
        var_408 = wp::where(var_406, var_407, var_404);
    }
    var_409 = wp::where(var_402, var_404, var_408);
    // for j in range(3):                                                                     <L 190>
    // q = q0                                                                                 <L 191>
    var_411 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_413 = (var_410 == var_412);
    if (var_413) {
        // q = q1                                                                             <L 193>
        var_414 = wp::copy(var_q1);
    }
    var_415 = wp::where(var_413, var_414, var_411);
    if (!var_413) {
        // elif j == 2:                                                                       <L 194>
        var_417 = (var_410 == var_416);
        if (var_417) {
            // q = q2                                                                         <L 195>
            var_418 = wp::copy(var_q2);
        }
        var_419 = wp::where(var_417, var_418, var_415);
    }
    var_420 = wp::where(var_413, var_415, var_419);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_421 = wp::sub(var_409, var_420);
    var_422 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_421, var_cutoff_sq);
    if (var_422) {
        // return True                                                                        <L 197>
        return var_423;
    }
    // q = q0                                                                                 <L 191>
    var_425 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_427 = (var_424 == var_426);
    if (var_427) {
        // q = q1                                                                             <L 193>
        var_428 = wp::copy(var_q1);
    }
    var_429 = wp::where(var_427, var_428, var_425);
    if (!var_427) {
        // elif j == 2:                                                                       <L 194>
        var_431 = (var_424 == var_430);
        if (var_431) {
            // q = q2                                                                         <L 195>
            var_432 = wp::copy(var_q2);
        }
        var_433 = wp::where(var_431, var_432, var_429);
    }
    var_434 = wp::where(var_427, var_429, var_433);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_435 = wp::sub(var_409, var_434);
    var_436 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_435, var_cutoff_sq);
    if (var_436) {
        // return True                                                                        <L 197>
        return var_437;
    }
    // q = q0                                                                                 <L 191>
    var_439 = wp::copy(var_q0);
    // if j == 1:                                                                             <L 192>
    var_441 = (var_438 == var_440);
    if (var_441) {
        // q = q1                                                                             <L 193>
        var_442 = wp::copy(var_q1);
    }
    var_443 = wp::where(var_441, var_442, var_439);
    if (!var_441) {
        // elif j == 2:                                                                       <L 194>
        var_445 = (var_438 == var_444);
        if (var_445) {
            // q = q2                                                                         <L 195>
            var_446 = wp::copy(var_q2);
        }
        var_447 = wp::where(var_445, var_446, var_443);
    }
    var_448 = wp::where(var_441, var_443, var_447);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, p - q, cutoff_sq):                          <L 196>
    var_449 = wp::sub(var_409, var_448);
    var_450 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_449, var_cutoff_sq);
    if (var_450) {
        // return True                                                                        <L 197>
        return var_451;
    }
    // for i in range(3):                                                                     <L 200>
    // p = p0                                                                                 <L 201>
    var_453 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 202>
    var_455 = (var_452 == var_454);
    if (var_455) {
        // p = p1                                                                             <L 203>
        var_456 = wp::copy(var_p1);
    }
    var_457 = wp::where(var_455, var_456, var_453);
    if (!var_455) {
        // elif i == 2:                                                                       <L 204>
        var_459 = (var_452 == var_458);
        if (var_459) {
            // p = p2                                                                         <L 205>
            var_460 = wp::copy(var_p2);
        }
        var_461 = wp::where(var_459, var_460, var_457);
    }
    var_462 = wp::where(var_455, var_457, var_461);
    // for j in range(3):                                                                     <L 206>
    // qa = q0                                                                                <L 207>
    var_464 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_465 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_467 = (var_463 == var_466);
    if (var_467) {
        // qa = q1                                                                            <L 210>
        var_468 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_469 = wp::copy(var_q2);
    }
    var_470 = wp::where(var_467, var_468, var_464);
    var_471 = wp::where(var_467, var_469, var_465);
    if (!var_467) {
        // elif j == 2:                                                                       <L 212>
        var_473 = (var_463 == var_472);
        if (var_473) {
            // qa = q2                                                                        <L 213>
            var_474 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_475 = wp::copy(var_q0);
        }
        var_476 = wp::where(var_473, var_474, var_470);
        var_477 = wp::where(var_473, var_475, var_471);
    }
    var_478 = wp::where(var_467, var_470, var_476);
    var_479 = wp::where(var_467, var_471, var_477);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_480 = _point_segment_axis_0(var_462, var_478, var_479);
    var_481 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_480, var_cutoff_sq);
    if (var_481) {
        // return True                                                                        <L 216>
        return var_482;
    }
    // qa = q0                                                                                <L 207>
    var_484 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_485 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_487 = (var_483 == var_486);
    if (var_487) {
        // qa = q1                                                                            <L 210>
        var_488 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_489 = wp::copy(var_q2);
    }
    var_490 = wp::where(var_487, var_488, var_484);
    var_491 = wp::where(var_487, var_489, var_485);
    if (!var_487) {
        // elif j == 2:                                                                       <L 212>
        var_493 = (var_483 == var_492);
        if (var_493) {
            // qa = q2                                                                        <L 213>
            var_494 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_495 = wp::copy(var_q0);
        }
        var_496 = wp::where(var_493, var_494, var_490);
        var_497 = wp::where(var_493, var_495, var_491);
    }
    var_498 = wp::where(var_487, var_490, var_496);
    var_499 = wp::where(var_487, var_491, var_497);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_500 = _point_segment_axis_0(var_462, var_498, var_499);
    var_501 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_500, var_cutoff_sq);
    if (var_501) {
        // return True                                                                        <L 216>
        return var_502;
    }
    // qa = q0                                                                                <L 207>
    var_504 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_505 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_507 = (var_503 == var_506);
    if (var_507) {
        // qa = q1                                                                            <L 210>
        var_508 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_509 = wp::copy(var_q2);
    }
    var_510 = wp::where(var_507, var_508, var_504);
    var_511 = wp::where(var_507, var_509, var_505);
    if (!var_507) {
        // elif j == 2:                                                                       <L 212>
        var_513 = (var_503 == var_512);
        if (var_513) {
            // qa = q2                                                                        <L 213>
            var_514 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_515 = wp::copy(var_q0);
        }
        var_516 = wp::where(var_513, var_514, var_510);
        var_517 = wp::where(var_513, var_515, var_511);
    }
    var_518 = wp::where(var_507, var_510, var_516);
    var_519 = wp::where(var_507, var_511, var_517);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_520 = _point_segment_axis_0(var_462, var_518, var_519);
    var_521 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_520, var_cutoff_sq);
    if (var_521) {
        // return True                                                                        <L 216>
        return var_522;
    }
    // p = p0                                                                                 <L 201>
    var_524 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 202>
    var_526 = (var_523 == var_525);
    if (var_526) {
        // p = p1                                                                             <L 203>
        var_527 = wp::copy(var_p1);
    }
    var_528 = wp::where(var_526, var_527, var_524);
    if (!var_526) {
        // elif i == 2:                                                                       <L 204>
        var_530 = (var_523 == var_529);
        if (var_530) {
            // p = p2                                                                         <L 205>
            var_531 = wp::copy(var_p2);
        }
        var_532 = wp::where(var_530, var_531, var_528);
    }
    var_533 = wp::where(var_526, var_528, var_532);
    // for j in range(3):                                                                     <L 206>
    // qa = q0                                                                                <L 207>
    var_535 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_536 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_538 = (var_534 == var_537);
    if (var_538) {
        // qa = q1                                                                            <L 210>
        var_539 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_540 = wp::copy(var_q2);
    }
    var_541 = wp::where(var_538, var_539, var_535);
    var_542 = wp::where(var_538, var_540, var_536);
    if (!var_538) {
        // elif j == 2:                                                                       <L 212>
        var_544 = (var_534 == var_543);
        if (var_544) {
            // qa = q2                                                                        <L 213>
            var_545 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_546 = wp::copy(var_q0);
        }
        var_547 = wp::where(var_544, var_545, var_541);
        var_548 = wp::where(var_544, var_546, var_542);
    }
    var_549 = wp::where(var_538, var_541, var_547);
    var_550 = wp::where(var_538, var_542, var_548);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_551 = _point_segment_axis_0(var_533, var_549, var_550);
    var_552 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_551, var_cutoff_sq);
    if (var_552) {
        // return True                                                                        <L 216>
        return var_553;
    }
    // qa = q0                                                                                <L 207>
    var_555 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_556 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_558 = (var_554 == var_557);
    if (var_558) {
        // qa = q1                                                                            <L 210>
        var_559 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_560 = wp::copy(var_q2);
    }
    var_561 = wp::where(var_558, var_559, var_555);
    var_562 = wp::where(var_558, var_560, var_556);
    if (!var_558) {
        // elif j == 2:                                                                       <L 212>
        var_564 = (var_554 == var_563);
        if (var_564) {
            // qa = q2                                                                        <L 213>
            var_565 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_566 = wp::copy(var_q0);
        }
        var_567 = wp::where(var_564, var_565, var_561);
        var_568 = wp::where(var_564, var_566, var_562);
    }
    var_569 = wp::where(var_558, var_561, var_567);
    var_570 = wp::where(var_558, var_562, var_568);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_571 = _point_segment_axis_0(var_533, var_569, var_570);
    var_572 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_571, var_cutoff_sq);
    if (var_572) {
        // return True                                                                        <L 216>
        return var_573;
    }
    // qa = q0                                                                                <L 207>
    var_575 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_576 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_578 = (var_574 == var_577);
    if (var_578) {
        // qa = q1                                                                            <L 210>
        var_579 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_580 = wp::copy(var_q2);
    }
    var_581 = wp::where(var_578, var_579, var_575);
    var_582 = wp::where(var_578, var_580, var_576);
    if (!var_578) {
        // elif j == 2:                                                                       <L 212>
        var_584 = (var_574 == var_583);
        if (var_584) {
            // qa = q2                                                                        <L 213>
            var_585 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_586 = wp::copy(var_q0);
        }
        var_587 = wp::where(var_584, var_585, var_581);
        var_588 = wp::where(var_584, var_586, var_582);
    }
    var_589 = wp::where(var_578, var_581, var_587);
    var_590 = wp::where(var_578, var_582, var_588);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_591 = _point_segment_axis_0(var_533, var_589, var_590);
    var_592 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_591, var_cutoff_sq);
    if (var_592) {
        // return True                                                                        <L 216>
        return var_593;
    }
    // p = p0                                                                                 <L 201>
    var_595 = wp::copy(var_p0);
    // if i == 1:                                                                             <L 202>
    var_597 = (var_594 == var_596);
    if (var_597) {
        // p = p1                                                                             <L 203>
        var_598 = wp::copy(var_p1);
    }
    var_599 = wp::where(var_597, var_598, var_595);
    if (!var_597) {
        // elif i == 2:                                                                       <L 204>
        var_601 = (var_594 == var_600);
        if (var_601) {
            // p = p2                                                                         <L 205>
            var_602 = wp::copy(var_p2);
        }
        var_603 = wp::where(var_601, var_602, var_599);
    }
    var_604 = wp::where(var_597, var_599, var_603);
    // for j in range(3):                                                                     <L 206>
    // qa = q0                                                                                <L 207>
    var_606 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_607 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_609 = (var_605 == var_608);
    if (var_609) {
        // qa = q1                                                                            <L 210>
        var_610 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_611 = wp::copy(var_q2);
    }
    var_612 = wp::where(var_609, var_610, var_606);
    var_613 = wp::where(var_609, var_611, var_607);
    if (!var_609) {
        // elif j == 2:                                                                       <L 212>
        var_615 = (var_605 == var_614);
        if (var_615) {
            // qa = q2                                                                        <L 213>
            var_616 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_617 = wp::copy(var_q0);
        }
        var_618 = wp::where(var_615, var_616, var_612);
        var_619 = wp::where(var_615, var_617, var_613);
    }
    var_620 = wp::where(var_609, var_612, var_618);
    var_621 = wp::where(var_609, var_613, var_619);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_622 = _point_segment_axis_0(var_604, var_620, var_621);
    var_623 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_622, var_cutoff_sq);
    if (var_623) {
        // return True                                                                        <L 216>
        return var_624;
    }
    // qa = q0                                                                                <L 207>
    var_626 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_627 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_629 = (var_625 == var_628);
    if (var_629) {
        // qa = q1                                                                            <L 210>
        var_630 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_631 = wp::copy(var_q2);
    }
    var_632 = wp::where(var_629, var_630, var_626);
    var_633 = wp::where(var_629, var_631, var_627);
    if (!var_629) {
        // elif j == 2:                                                                       <L 212>
        var_635 = (var_625 == var_634);
        if (var_635) {
            // qa = q2                                                                        <L 213>
            var_636 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_637 = wp::copy(var_q0);
        }
        var_638 = wp::where(var_635, var_636, var_632);
        var_639 = wp::where(var_635, var_637, var_633);
    }
    var_640 = wp::where(var_629, var_632, var_638);
    var_641 = wp::where(var_629, var_633, var_639);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_642 = _point_segment_axis_0(var_604, var_640, var_641);
    var_643 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_642, var_cutoff_sq);
    if (var_643) {
        // return True                                                                        <L 216>
        return var_644;
    }
    // qa = q0                                                                                <L 207>
    var_646 = wp::copy(var_q0);
    // qb = q1                                                                                <L 208>
    var_647 = wp::copy(var_q1);
    // if j == 1:                                                                             <L 209>
    var_649 = (var_645 == var_648);
    if (var_649) {
        // qa = q1                                                                            <L 210>
        var_650 = wp::copy(var_q1);
        // qb = q2                                                                            <L 211>
        var_651 = wp::copy(var_q2);
    }
    var_652 = wp::where(var_649, var_650, var_646);
    var_653 = wp::where(var_649, var_651, var_647);
    if (!var_649) {
        // elif j == 2:                                                                       <L 212>
        var_655 = (var_645 == var_654);
        if (var_655) {
            // qa = q2                                                                        <L 213>
            var_656 = wp::copy(var_q2);
            // qb = q0                                                                        <L 214>
            var_657 = wp::copy(var_q0);
        }
        var_658 = wp::where(var_655, var_656, var_652);
        var_659 = wp::where(var_655, var_657, var_653);
    }
    var_660 = wp::where(var_649, var_652, var_658);
    var_661 = wp::where(var_649, var_653, var_659);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(p, qa, qb), cutoff_sq):       <L 215>
    var_662 = _point_segment_axis_0(var_604, var_660, var_661);
    var_663 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_662, var_cutoff_sq);
    if (var_663) {
        // return True                                                                        <L 216>
        return var_664;
    }
    // for i in range(3):                                                                     <L 219>
    // q = q0                                                                                 <L 220>
    var_666 = wp::copy(var_q0);
    // if i == 1:                                                                             <L 221>
    var_668 = (var_665 == var_667);
    if (var_668) {
        // q = q1                                                                             <L 222>
        var_669 = wp::copy(var_q1);
    }
    var_670 = wp::where(var_668, var_669, var_666);
    if (!var_668) {
        // elif i == 2:                                                                       <L 223>
        var_672 = (var_665 == var_671);
        if (var_672) {
            // q = q2                                                                         <L 224>
            var_673 = wp::copy(var_q2);
        }
        var_674 = wp::where(var_672, var_673, var_670);
    }
    var_675 = wp::where(var_668, var_670, var_674);
    // for j in range(3):                                                                     <L 225>
    // pa = p0                                                                                <L 226>
    var_677 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_678 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_680 = (var_676 == var_679);
    if (var_680) {
        // pa = p1                                                                            <L 229>
        var_681 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_682 = wp::copy(var_p2);
    }
    var_683 = wp::where(var_680, var_681, var_677);
    var_684 = wp::where(var_680, var_682, var_678);
    if (!var_680) {
        // elif j == 2:                                                                       <L 231>
        var_686 = (var_676 == var_685);
        if (var_686) {
            // pa = p2                                                                        <L 232>
            var_687 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_688 = wp::copy(var_p0);
        }
        var_689 = wp::where(var_686, var_687, var_683);
        var_690 = wp::where(var_686, var_688, var_684);
    }
    var_691 = wp::where(var_680, var_683, var_689);
    var_692 = wp::where(var_680, var_684, var_690);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_693 = _point_segment_axis_0(var_675, var_691, var_692);
    var_694 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_693, var_cutoff_sq);
    if (var_694) {
        // return True                                                                        <L 235>
        return var_695;
    }
    // pa = p0                                                                                <L 226>
    var_697 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_698 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_700 = (var_696 == var_699);
    if (var_700) {
        // pa = p1                                                                            <L 229>
        var_701 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_702 = wp::copy(var_p2);
    }
    var_703 = wp::where(var_700, var_701, var_697);
    var_704 = wp::where(var_700, var_702, var_698);
    if (!var_700) {
        // elif j == 2:                                                                       <L 231>
        var_706 = (var_696 == var_705);
        if (var_706) {
            // pa = p2                                                                        <L 232>
            var_707 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_708 = wp::copy(var_p0);
        }
        var_709 = wp::where(var_706, var_707, var_703);
        var_710 = wp::where(var_706, var_708, var_704);
    }
    var_711 = wp::where(var_700, var_703, var_709);
    var_712 = wp::where(var_700, var_704, var_710);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_713 = _point_segment_axis_0(var_675, var_711, var_712);
    var_714 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_713, var_cutoff_sq);
    if (var_714) {
        // return True                                                                        <L 235>
        return var_715;
    }
    // pa = p0                                                                                <L 226>
    var_717 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_718 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_720 = (var_716 == var_719);
    if (var_720) {
        // pa = p1                                                                            <L 229>
        var_721 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_722 = wp::copy(var_p2);
    }
    var_723 = wp::where(var_720, var_721, var_717);
    var_724 = wp::where(var_720, var_722, var_718);
    if (!var_720) {
        // elif j == 2:                                                                       <L 231>
        var_726 = (var_716 == var_725);
        if (var_726) {
            // pa = p2                                                                        <L 232>
            var_727 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_728 = wp::copy(var_p0);
        }
        var_729 = wp::where(var_726, var_727, var_723);
        var_730 = wp::where(var_726, var_728, var_724);
    }
    var_731 = wp::where(var_720, var_723, var_729);
    var_732 = wp::where(var_720, var_724, var_730);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_733 = _point_segment_axis_0(var_675, var_731, var_732);
    var_734 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_733, var_cutoff_sq);
    if (var_734) {
        // return True                                                                        <L 235>
        return var_735;
    }
    // q = q0                                                                                 <L 220>
    var_737 = wp::copy(var_q0);
    // if i == 1:                                                                             <L 221>
    var_739 = (var_736 == var_738);
    if (var_739) {
        // q = q1                                                                             <L 222>
        var_740 = wp::copy(var_q1);
    }
    var_741 = wp::where(var_739, var_740, var_737);
    if (!var_739) {
        // elif i == 2:                                                                       <L 223>
        var_743 = (var_736 == var_742);
        if (var_743) {
            // q = q2                                                                         <L 224>
            var_744 = wp::copy(var_q2);
        }
        var_745 = wp::where(var_743, var_744, var_741);
    }
    var_746 = wp::where(var_739, var_741, var_745);
    // for j in range(3):                                                                     <L 225>
    // pa = p0                                                                                <L 226>
    var_748 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_749 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_751 = (var_747 == var_750);
    if (var_751) {
        // pa = p1                                                                            <L 229>
        var_752 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_753 = wp::copy(var_p2);
    }
    var_754 = wp::where(var_751, var_752, var_748);
    var_755 = wp::where(var_751, var_753, var_749);
    if (!var_751) {
        // elif j == 2:                                                                       <L 231>
        var_757 = (var_747 == var_756);
        if (var_757) {
            // pa = p2                                                                        <L 232>
            var_758 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_759 = wp::copy(var_p0);
        }
        var_760 = wp::where(var_757, var_758, var_754);
        var_761 = wp::where(var_757, var_759, var_755);
    }
    var_762 = wp::where(var_751, var_754, var_760);
    var_763 = wp::where(var_751, var_755, var_761);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_764 = _point_segment_axis_0(var_746, var_762, var_763);
    var_765 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_764, var_cutoff_sq);
    if (var_765) {
        // return True                                                                        <L 235>
        return var_766;
    }
    // pa = p0                                                                                <L 226>
    var_768 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_769 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_771 = (var_767 == var_770);
    if (var_771) {
        // pa = p1                                                                            <L 229>
        var_772 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_773 = wp::copy(var_p2);
    }
    var_774 = wp::where(var_771, var_772, var_768);
    var_775 = wp::where(var_771, var_773, var_769);
    if (!var_771) {
        // elif j == 2:                                                                       <L 231>
        var_777 = (var_767 == var_776);
        if (var_777) {
            // pa = p2                                                                        <L 232>
            var_778 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_779 = wp::copy(var_p0);
        }
        var_780 = wp::where(var_777, var_778, var_774);
        var_781 = wp::where(var_777, var_779, var_775);
    }
    var_782 = wp::where(var_771, var_774, var_780);
    var_783 = wp::where(var_771, var_775, var_781);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_784 = _point_segment_axis_0(var_746, var_782, var_783);
    var_785 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_784, var_cutoff_sq);
    if (var_785) {
        // return True                                                                        <L 235>
        return var_786;
    }
    // pa = p0                                                                                <L 226>
    var_788 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_789 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_791 = (var_787 == var_790);
    if (var_791) {
        // pa = p1                                                                            <L 229>
        var_792 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_793 = wp::copy(var_p2);
    }
    var_794 = wp::where(var_791, var_792, var_788);
    var_795 = wp::where(var_791, var_793, var_789);
    if (!var_791) {
        // elif j == 2:                                                                       <L 231>
        var_797 = (var_787 == var_796);
        if (var_797) {
            // pa = p2                                                                        <L 232>
            var_798 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_799 = wp::copy(var_p0);
        }
        var_800 = wp::where(var_797, var_798, var_794);
        var_801 = wp::where(var_797, var_799, var_795);
    }
    var_802 = wp::where(var_791, var_794, var_800);
    var_803 = wp::where(var_791, var_795, var_801);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_804 = _point_segment_axis_0(var_746, var_802, var_803);
    var_805 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_804, var_cutoff_sq);
    if (var_805) {
        // return True                                                                        <L 235>
        return var_806;
    }
    // q = q0                                                                                 <L 220>
    var_808 = wp::copy(var_q0);
    // if i == 1:                                                                             <L 221>
    var_810 = (var_807 == var_809);
    if (var_810) {
        // q = q1                                                                             <L 222>
        var_811 = wp::copy(var_q1);
    }
    var_812 = wp::where(var_810, var_811, var_808);
    if (!var_810) {
        // elif i == 2:                                                                       <L 223>
        var_814 = (var_807 == var_813);
        if (var_814) {
            // q = q2                                                                         <L 224>
            var_815 = wp::copy(var_q2);
        }
        var_816 = wp::where(var_814, var_815, var_812);
    }
    var_817 = wp::where(var_810, var_812, var_816);
    // for j in range(3):                                                                     <L 225>
    // pa = p0                                                                                <L 226>
    var_819 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_820 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_822 = (var_818 == var_821);
    if (var_822) {
        // pa = p1                                                                            <L 229>
        var_823 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_824 = wp::copy(var_p2);
    }
    var_825 = wp::where(var_822, var_823, var_819);
    var_826 = wp::where(var_822, var_824, var_820);
    if (!var_822) {
        // elif j == 2:                                                                       <L 231>
        var_828 = (var_818 == var_827);
        if (var_828) {
            // pa = p2                                                                        <L 232>
            var_829 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_830 = wp::copy(var_p0);
        }
        var_831 = wp::where(var_828, var_829, var_825);
        var_832 = wp::where(var_828, var_830, var_826);
    }
    var_833 = wp::where(var_822, var_825, var_831);
    var_834 = wp::where(var_822, var_826, var_832);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_835 = _point_segment_axis_0(var_817, var_833, var_834);
    var_836 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_835, var_cutoff_sq);
    if (var_836) {
        // return True                                                                        <L 235>
        return var_837;
    }
    // pa = p0                                                                                <L 226>
    var_839 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_840 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_842 = (var_838 == var_841);
    if (var_842) {
        // pa = p1                                                                            <L 229>
        var_843 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_844 = wp::copy(var_p2);
    }
    var_845 = wp::where(var_842, var_843, var_839);
    var_846 = wp::where(var_842, var_844, var_840);
    if (!var_842) {
        // elif j == 2:                                                                       <L 231>
        var_848 = (var_838 == var_847);
        if (var_848) {
            // pa = p2                                                                        <L 232>
            var_849 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_850 = wp::copy(var_p0);
        }
        var_851 = wp::where(var_848, var_849, var_845);
        var_852 = wp::where(var_848, var_850, var_846);
    }
    var_853 = wp::where(var_842, var_845, var_851);
    var_854 = wp::where(var_842, var_846, var_852);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_855 = _point_segment_axis_0(var_817, var_853, var_854);
    var_856 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_855, var_cutoff_sq);
    if (var_856) {
        // return True                                                                        <L 235>
        return var_857;
    }
    // pa = p0                                                                                <L 226>
    var_859 = wp::copy(var_p0);
    // pb = p1                                                                                <L 227>
    var_860 = wp::copy(var_p1);
    // if j == 1:                                                                             <L 228>
    var_862 = (var_858 == var_861);
    if (var_862) {
        // pa = p1                                                                            <L 229>
        var_863 = wp::copy(var_p1);
        // pb = p2                                                                            <L 230>
        var_864 = wp::copy(var_p2);
    }
    var_865 = wp::where(var_862, var_863, var_859);
    var_866 = wp::where(var_862, var_864, var_860);
    if (!var_862) {
        // elif j == 2:                                                                       <L 231>
        var_868 = (var_858 == var_867);
        if (var_868) {
            // pa = p2                                                                        <L 232>
            var_869 = wp::copy(var_p2);
            // pb = p0                                                                        <L 233>
            var_870 = wp::copy(var_p0);
        }
        var_871 = wp::where(var_868, var_869, var_865);
        var_872 = wp::where(var_868, var_870, var_866);
    }
    var_873 = wp::where(var_862, var_865, var_871);
    var_874 = wp::where(var_862, var_866, var_872);
    // if _axis_separated(p0, p1, p2, q0, q1, q2, _point_segment_axis(q, pa, pb), cutoff_sq):       <L 234>
    var_875 = _point_segment_axis_0(var_817, var_873, var_874);
    var_876 = _axis_separated_0(var_p0, var_p1, var_p2, var_q0, var_q1, var_q2, var_875, var_cutoff_sq);
    if (var_876) {
        // return True                                                                        <L 235>
        return var_877;
    }
    // return False                                                                           <L 237>
    return var_878;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:0
static CUDA_CALLABLE void adj_sap_binary_search_0(
    wp::array_t<wp::int32> var_values,
    wp::int32 var_value,
    wp::int32 var_lower,
    wp::int32 var_upper,
    wp::array_t<wp::int32> & adj_values,
    wp::int32 & adj_value,
    wp::int32 & adj_lower,
    wp::int32 & adj_upper,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:51
static CUDA_CALLABLE void adj__flex_element_aabb_filter_0(
    wp::vec_t<3, wp::float32> var_box1_min,
    wp::vec_t<3, wp::float32> var_box1_max,
    wp::vec_t<3, wp::float32> var_box2_min,
    wp::vec_t<3, wp::float32> var_box2_max,
    wp::vec_t<3, wp::float32> & adj_box1_min,
    wp::vec_t<3, wp::float32> & adj_box1_max,
    wp::vec_t<3, wp::float32> & adj_box2_min,
    wp::vec_t<3, wp::float32> & adj_box2_max,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1266
static CUDA_CALLABLE void adj__get_element_vertices_0(
    wp::array_t<wp::int32> var_flex_elem,
    wp::int32 var_dim,
    wp::int32 var_elem_data_idx,
    wp::array_t<wp::int32> & adj_flex_elem,
    wp::int32 & adj_dim,
    wp::int32 & adj_elem_data_idx,
    wp::vec_t<4, wp::int32> & adj_ret)
{
    //---------
    // primal vars
    wp::int32* var_0;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 1;
    wp::int32 var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    const wp::int32 var_8 = -1;
    wp::int32 var_9;
    const wp::int32 var_10 = -1;
    wp::int32 var_11;
    const wp::int32 var_12 = 2;
    bool var_13;
    const wp::int32 var_14 = 2;
    wp::int32 var_15;
    wp::int32* var_16;
    wp::int32 var_17;
    wp::int32 var_18;
    wp::int32 var_19;
    const wp::int32 var_20 = 3;
    bool var_21;
    const wp::int32 var_22 = 3;
    wp::int32 var_23;
    wp::int32* var_24;
    wp::int32 var_25;
    wp::int32 var_26;
    wp::int32 var_27;
    wp::vec_t<4, wp::int32> var_28;
    //---------
    // dual vars
    wp::int32 adj_0 = {};
    wp::int32 adj_1 = {};
    wp::int32 adj_2 = {};
    wp::int32 adj_3 = {};
    wp::int32 adj_4 = {};
    wp::int32 adj_5 = {};
    wp::int32 adj_6 = {};
    wp::int32 adj_7 = {};
    wp::int32 adj_8 = {};
    wp::int32 adj_9 = {};
    wp::int32 adj_10 = {};
    wp::int32 adj_11 = {};
    wp::int32 adj_12 = {};
    bool adj_13 = {};
    wp::int32 adj_14 = {};
    wp::int32 adj_15 = {};
    wp::int32 adj_16 = {};
    wp::int32 adj_17 = {};
    wp::int32 adj_18 = {};
    wp::int32 adj_19 = {};
    wp::int32 adj_20 = {};
    bool adj_21 = {};
    wp::int32 adj_22 = {};
    wp::int32 adj_23 = {};
    wp::int32 adj_24 = {};
    wp::int32 adj_25 = {};
    wp::int32 adj_26 = {};
    wp::int32 adj_27 = {};
    wp::vec_t<4, wp::int32> adj_28 = {};
    //---------
    // forward
    // def _get_element_vertices(                                                             <L 1267>
    // v0 = flex_elem[elem_data_idx]                                                          <L 1274>
    var_0 = wp::address(var_flex_elem, var_elem_data_idx);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // v1 = flex_elem[elem_data_idx + 1]                                                      <L 1275>
    var_4 = wp::add(var_elem_data_idx, var_3);
    var_5 = wp::address(var_flex_elem, var_4);
    var_7 = wp::load(var_5);
    var_6 = wp::copy(var_7);
    // v2 = int(-1)                                                                           <L 1276>
    var_9 = wp::int(var_8);
    // v3 = int(-1)                                                                           <L 1277>
    var_11 = wp::int(var_10);
    // if dim >= 2:                                                                           <L 1278>
    var_13 = (var_dim >= var_12);
    if (var_13) {
        // v2 = flex_elem[elem_data_idx + 2]                                                  <L 1279>
        var_15 = wp::add(var_elem_data_idx, var_14);
        var_16 = wp::address(var_flex_elem, var_15);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
    }
    var_19 = wp::where(var_13, var_17, var_9);
    // if dim >= 3:                                                                           <L 1280>
    var_21 = (var_dim >= var_20);
    if (var_21) {
        // v3 = flex_elem[elem_data_idx + 3]                                                  <L 1281>
        var_23 = wp::add(var_elem_data_idx, var_22);
        var_24 = wp::address(var_flex_elem, var_23);
        var_26 = wp::load(var_24);
        var_25 = wp::copy(var_26);
    }
    var_27 = wp::where(var_21, var_25, var_11);
    // return wp.vec4i(v0, v1, v2, v3)                                                        <L 1282>
    var_28 = wp::vec_t<4, wp::int32>(var_1, var_6, var_19, var_27);
    goto label0;
    //---------
    // reverse
    label0:;
    adj_28 += adj_ret;
    wp::adj_vec_t(var_1, var_6, var_19, var_27, adj_1, adj_6, adj_19, adj_27, adj_28);
    // adj: return wp.vec4i(v0, v1, v2, v3)                                                   <L 1282>
    wp::adj_where(var_21, var_25, var_11, adj_21, adj_25, adj_11, adj_27);
    if (var_21) {
        wp::adj_copy(var_26, adj_24, adj_25);
        wp::adj_address(var_flex_elem, var_23, adj_flex_elem, adj_23, adj_24);
        wp::adj_add(var_elem_data_idx, var_22, adj_elem_data_idx, adj_22, adj_23);
        // adj: v3 = flex_elem[elem_data_idx + 3]                                             <L 1281>
    }
    // adj: if dim >= 3:                                                                      <L 1280>
    wp::adj_where(var_13, var_17, var_9, adj_13, adj_17, adj_9, adj_19);
    if (var_13) {
        wp::adj_copy(var_18, adj_16, adj_17);
        wp::adj_address(var_flex_elem, var_15, adj_flex_elem, adj_15, adj_16);
        wp::adj_add(var_elem_data_idx, var_14, adj_elem_data_idx, adj_14, adj_15);
        // adj: v2 = flex_elem[elem_data_idx + 2]                                             <L 1279>
    }
    // adj: if dim >= 2:                                                                      <L 1278>
    // adj: v3 = int(-1)                                                                      <L 1277>
    // adj: v2 = int(-1)                                                                      <L 1276>
    wp::adj_copy(var_7, adj_5, adj_6);
    wp::adj_address(var_flex_elem, var_4, adj_flex_elem, adj_4, adj_5);
    wp::adj_add(var_elem_data_idx, var_3, adj_elem_data_idx, adj_3, adj_4);
    // adj: v1 = flex_elem[elem_data_idx + 1]                                                 <L 1275>
    wp::adj_copy(var_2, adj_0, adj_1);
    wp::adj_address(var_flex_elem, var_elem_data_idx, adj_flex_elem, adj_elem_data_idx, adj_0);
    // adj: v0 = flex_elem[elem_data_idx]                                                     <L 1274>
    // adj: def _get_element_vertices(                                                        <L 1267>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1240
static CUDA_CALLABLE void adj__exclude_self_collision_0(
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::vec_t<4, wp::int32> var_v1,
    wp::int32 var_n1,
    wp::vec_t<4, wp::int32> var_v2,
    wp::int32 var_n2,
    wp::int32 var_vert_adr,
    wp::array_t<wp::int32> & adj_flex_vertbodyid,
    wp::vec_t<4, wp::int32> & adj_v1,
    wp::int32 & adj_n1,
    wp::vec_t<4, wp::int32> & adj_v2,
    wp::int32 & adj_n2,
    wp::int32 & adj_vert_adr,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:69
static CUDA_CALLABLE void adj__axis_separated_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_q0,
    wp::vec_t<3, wp::float32> var_q1,
    wp::vec_t<3, wp::float32> var_q2,
    wp::vec_t<3, wp::float32> var_ax,
    wp::float32 var_cutoff_sq,
    wp::vec_t<3, wp::float32> & adj_p0,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    wp::vec_t<3, wp::float32> & adj_q0,
    wp::vec_t<3, wp::float32> & adj_q1,
    wp::vec_t<3, wp::float32> & adj_q2,
    wp::vec_t<3, wp::float32> & adj_ax,
    wp::float32 & adj_cutoff_sq,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:93
static CUDA_CALLABLE void adj__point_segment_axis_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> var_b,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_a,
    wp::vec_t<3, wp::float32> & adj_b,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:104
static CUDA_CALLABLE void adj__triangle_sat_separated_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_q0,
    wp::vec_t<3, wp::float32> var_q1,
    wp::vec_t<3, wp::float32> var_q2,
    wp::float32 var_cutoff_sq,
    wp::vec_t<3, wp::float32> & adj_p0,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    wp::vec_t<3, wp::float32> & adj_q0,
    wp::vec_t<3, wp::float32> & adj_q1,
    wp::vec_t<3, wp::float32> & adj_q2,
    wp::float32 & adj_cutoff_sq,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _flex_sap_sweep__locals__kernel_ad54bdd4_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_flex_contype,
    wp::array_t<wp::int32> var_flex_conaffinity,
    wp::array_t<wp::float32> var_flex_margin,
    wp::array_t<wp::float32> var_flex_gap,
    wp::array_t<wp::int32> var_flex_selfcollide,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::float32> var_flex_radius,
    wp::array_t<wp::int32> var_flex_elemflexid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_aabb_min_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_aabb_max_in,
    wp::int32 var_nelem,
    wp::array_t<wp::int32> var_sort_index_in,
    wp::array_t<wp::int32> var_cumulative_sum_in,
    wp::int32 var_nsweep_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_aabb_lower_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_aabb_upper_in,
    wp::int32 var_max_pairs,
    wp::array_t<wp::int32> var_ncollision_out,
    wp::array_t<wp::int32> var_overflow_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_out,
    wp::array_t<wp::int32> var_collision_worldid_out)
{
    wp::tile_shared_storage_t tile_mem;

    for (size_t _idx = static_cast<size_t>(blockDim.x) * static_cast<size_t>(blockIdx.x) + static_cast<size_t>(threadIdx.x);
         _idx < dim.size;
         _idx += static_cast<size_t>(blockDim.x) * static_cast<size_t>(gridDim.x))
    {
            // reset shared memory allocator
        wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::shape_t* var_1;
        const wp::int32 var_2 = 0;
        wp::int32 var_3;
        wp::shape_t var_4;
        const wp::int32 var_5 = 1;
        wp::int32 var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        bool var_10;
        const wp::int32 var_11 = 0;
        wp::int32 var_12;
        wp::int32 var_13;
        const wp::int32 var_14 = 1;
        wp::int32 var_15;
        const wp::int32 var_16 = 0;
        bool var_17;
        const wp::int32 var_18 = 1;
        wp::int32 var_19;
        wp::int32* var_20;
        wp::int32 var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        wp::int32 var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::int32* var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::int32* var_33;
        wp::int32 var_34;
        wp::int32 var_35;
        wp::int32* var_36;
        wp::int32 var_37;
        wp::int32 var_38;
        const bool var_39 = true;
        bool var_40;
        bool var_41;
        wp::int32* var_42;
        const wp::int32 var_43 = 0;
        bool var_44;
        wp::int32 var_45;
        wp::int32* var_46;
        wp::int32* var_47;
        wp::int32 var_48;
        wp::int32 var_49;
        wp::int32 var_50;
        const wp::int32 var_51 = 0;
        bool var_52;
        wp::int32 var_53;
        bool var_54;
        wp::int32 var_55;
        wp::int32 var_56;
        wp::int32 var_57;
        wp::int32 var_58;
        wp::int32 var_59;
        wp::int32 var_60;
        wp::int32 var_61;
        wp::int32 var_62;
        wp::int32 var_63;
        wp::int32 var_64;
        wp::int32 var_65;
        const bool var_66 = true;
        bool var_67;
        wp::vec_t<3, wp::float32>* var_68;
        wp::vec_t<3, wp::float32>* var_69;
        wp::vec_t<3, wp::float32>* var_70;
        wp::vec_t<3, wp::float32>* var_71;
        bool var_72;
        wp::vec_t<3, wp::float32> var_73;
        wp::vec_t<3, wp::float32> var_74;
        wp::vec_t<3, wp::float32> var_75;
        wp::vec_t<3, wp::float32> var_76;
        wp::int32 var_77;
        wp::int32* var_78;
        wp::int32 var_79;
        wp::int32 var_80;
        wp::int32* var_81;
        wp::int32 var_82;
        wp::int32 var_83;
        wp::int32* var_84;
        wp::int32 var_85;
        wp::int32 var_86;
        wp::int32* var_87;
        wp::int32 var_88;
        wp::int32 var_89;
        bool var_90;
        wp::int32 var_91;
        wp::int32 var_92;
        bool var_93;
        wp::int32 var_94;
        wp::int32 var_95;
        wp::vec_t<3, wp::float32>* var_96;
        wp::vec_t<3, wp::float32> var_97;
        wp::vec_t<3, wp::float32> var_98;
        wp::vec_t<3, wp::float32>* var_99;
        wp::vec_t<3, wp::float32> var_100;
        wp::vec_t<3, wp::float32> var_101;
        wp::vec_t<3, wp::float32>* var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::vec_t<3, wp::float32> var_104;
        wp::vec_t<3, wp::float32>* var_105;
        wp::vec_t<3, wp::float32> var_106;
        wp::vec_t<3, wp::float32> var_107;
        bool var_108;
        wp::int32 var_109;
        wp::int32* var_110;
        wp::int32 var_111;
        wp::int32 var_112;
        wp::int32* var_113;
        wp::int32 var_114;
        wp::int32 var_115;
        wp::int32* var_116;
        wp::int32 var_117;
        wp::int32 var_118;
        wp::int32 var_119;
        wp::int32* var_120;
        const wp::int32 var_121 = 1;
        wp::int32 var_122;
        wp::int32 var_123;
        wp::int32 var_124;
        wp::int32 var_125;
        wp::vec_t<4, wp::int32> var_126;
        const bool var_127 = true;
        wp::int32 var_128;
        wp::int32* var_129;
        const wp::int32 var_130 = 1;
        wp::int32 var_131;
        wp::int32 var_132;
        wp::int32 var_133;
        wp::int32 var_134;
        wp::vec_t<4, wp::int32> var_135;
        const wp::int32 var_136 = 1;
        wp::int32 var_137;
        const wp::int32 var_138 = 1;
        wp::int32 var_139;
        bool var_140;
        wp::int32 var_141;
        const wp::int32 var_142 = 2;
        bool var_143;
        const bool var_144 = true;
        bool var_145;
        const wp::int32 var_146 = 2;
        bool var_147;
        const bool var_148 = true;
        wp::float32* var_149;
        wp::float32 var_150;
        wp::float32 var_151;
        const wp::float32 var_152 = 2.0;
        wp::float32 var_153;
        wp::float32 var_154;
        const bool var_155 = true;
        bool var_156;
        wp::float32* var_157;
        wp::float32 var_158;
        wp::float32 var_159;
        wp::float32* var_160;
        wp::float32 var_161;
        wp::float32 var_162;
        wp::float32* var_163;
        wp::float32 var_164;
        wp::float32 var_165;
        wp::float32* var_166;
        wp::float32 var_167;
        wp::float32 var_168;
        wp::float32* var_169;
        wp::float32 var_170;
        wp::float32 var_171;
        wp::float32 var_172;
        wp::float32 var_173;
        const wp::int32 var_174 = 0;
        wp::int32 var_175;
        wp::int32 var_176;
        wp::vec_t<3, wp::float32>* var_177;
        wp::vec_t<3, wp::float32> var_178;
        wp::vec_t<3, wp::float32> var_179;
        const wp::int32 var_180 = 1;
        wp::int32 var_181;
        wp::int32 var_182;
        wp::vec_t<3, wp::float32>* var_183;
        wp::vec_t<3, wp::float32> var_184;
        wp::vec_t<3, wp::float32> var_185;
        const wp::int32 var_186 = 2;
        wp::int32 var_187;
        wp::int32 var_188;
        wp::vec_t<3, wp::float32>* var_189;
        wp::vec_t<3, wp::float32> var_190;
        wp::vec_t<3, wp::float32> var_191;
        const bool var_192 = true;
        const wp::int32 var_193 = 0;
        wp::int32 var_194;
        wp::int32 var_195;
        wp::vec_t<3, wp::float32>* var_196;
        wp::vec_t<3, wp::float32> var_197;
        wp::vec_t<3, wp::float32> var_198;
        const wp::int32 var_199 = 1;
        wp::int32 var_200;
        wp::int32 var_201;
        wp::vec_t<3, wp::float32>* var_202;
        wp::vec_t<3, wp::float32> var_203;
        wp::vec_t<3, wp::float32> var_204;
        const wp::int32 var_205 = 2;
        wp::int32 var_206;
        wp::int32 var_207;
        wp::vec_t<3, wp::float32>* var_208;
        wp::vec_t<3, wp::float32> var_209;
        wp::vec_t<3, wp::float32> var_210;
        bool var_211;
        wp::int32 var_212;
        wp::int32 var_213;
        wp::int32 var_214;
        const wp::int32 var_215 = 0;
        const wp::int32 var_216 = 1;
        wp::int32 var_217;
        bool var_218;
        const bool var_219 = true;
        const wp::str var_220 = "Flex SAP buffer overflow - please increase naconmax beyond %u\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.BROADPHASE (or = 0 for all)\n";
        const wp::int32 var_221 = 4;
        wp::int32 var_222;
        wp::vec_t<2, wp::int32> var_223;
        //---------
        // forward
        // def kernel(                                                                            <L 1394>
        // worldelemid = wp.tid()                                                                 <L 1428>
        var_0 = builtin_tid1d();
        // nworldelem = cumulative_sum_in.shape[0]                                                <L 1430>
        var_1 = &(var_cumulative_sum_in.shape);
        var_4 = wp::load(var_1);
        var_3 = wp::extract(var_4, var_2);
        // nworkpackages = cumulative_sum_in[nworldelem - 1]                                      <L 1431>
        var_6 = wp::sub(var_3, var_5);
        var_7 = wp::address(var_cumulative_sum_in, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // while worldelemid < nworkpackages:                                                     <L 1433>
        start_while_0:;
        var_10 = (var_0 < var_8);
        if ((var_10) == false) goto end_while_0;
            // i = sap_binary_search(cumulative_sum_in, worldelemid, 0, nworldelem)               <L 1434>
            var_12 = sap_binary_search_0(var_cumulative_sum_in, var_0, var_11, var_3);
            // j = i + worldelemid + 1                                                            <L 1435>
            var_13 = wp::add(var_12, var_0);
            var_15 = wp::add(var_13, var_14);
            // if i > 0:                                                                          <L 1436>
            var_17 = (var_12 > var_16);
            if (var_17) {
                // j -= cumulative_sum_in[i - 1]                                                  <L 1437>
                var_19 = wp::sub(var_12, var_18);
                var_20 = wp::address(var_cumulative_sum_in, var_19);
                var_22 = wp::load(var_20);
                var_21 = wp::sub(var_15, var_22);
            }
            var_23 = wp::where(var_17, var_21, var_15);
            // worldid = i // nelem                                                               <L 1439>
            var_24 = wp::floordiv(var_12, var_nelem);
            // i = i % nelem                                                                      <L 1440>
            var_25 = wp::mod(var_12, var_nelem);
            // j = j % nelem                                                                      <L 1441>
            var_26 = wp::mod(var_23, var_nelem);
            // elem1 = sort_index_in[worldid, i]                                                  <L 1443>
            var_27 = wp::address(var_sort_index_in, var_24, var_25);
            var_29 = wp::load(var_27);
            var_28 = wp::copy(var_29);
            // elem2 = sort_index_in[worldid, j]                                                  <L 1444>
            var_30 = wp::address(var_sort_index_in, var_24, var_26);
            var_32 = wp::load(var_30);
            var_31 = wp::copy(var_32);
            // flexid1 = flex_elemflexid[elem1]                                                   <L 1446>
            var_33 = wp::address(var_flex_elemflexid, var_28);
            var_35 = wp::load(var_33);
            var_34 = wp::copy(var_35);
            // flexid2 = flex_elemflexid[elem2]                                                   <L 1447>
            var_36 = wp::address(var_flex_elemflexid, var_31);
            var_38 = wp::load(var_36);
            var_37 = wp::copy(var_38);
            // if wp.static(is_self):                                                             <L 1449>
            // if flexid1 != flexid2 or flex_selfcollide[flexid1] == 0 or (flex_contype[flexid1] & flex_conaffinity[flexid1]) == 0:       <L 1450>
            var_41 = (var_34 != var_37);
            var_40 = var_41;
            if (!var_40) {
                var_42 = wp::address(var_flex_selfcollide, var_34);
                var_45 = wp::load(var_42);
                var_44 = (var_45 == var_43);
                var_40 = var_40 || var_44;
            }
            if (!var_40) {
                var_46 = wp::address(var_flex_contype, var_34);
                var_47 = wp::address(var_flex_conaffinity, var_34);
                var_49 = wp::load(var_46);
                var_50 = wp::load(var_47);
                var_48 = wp::bit_and(var_49, var_50);
                var_52 = (var_48 == var_51);
                var_40 = var_40 || var_52;
            }
            if (var_40) {
                // worldelemid += nsweep_in                                                       <L 1451>
                var_53 = wp::add(var_0, var_nsweep_in);
                // continue                                                                       <L 1452>
                wp::assign(var_0, var_53);
                goto start_while_0;
            }
            // if elem1 > elem2:                                                                  <L 1458>
            var_54 = (var_28 > var_31);
            if (var_54) {
                // tmpelem = elem1                                                                <L 1459>
                var_55 = wp::copy(var_28);
                // elem1 = elem2                                                                  <L 1460>
                var_56 = wp::copy(var_31);
                // elem2 = tmpelem                                                                <L 1461>
                var_57 = wp::copy(var_55);
                // tmpid = flexid1                                                                <L 1462>
                var_58 = wp::copy(var_34);
                // flexid1 = flexid2                                                              <L 1463>
                var_59 = wp::copy(var_37);
                // flexid2 = tmpid                                                                <L 1464>
                var_60 = wp::copy(var_58);
            }
            var_61 = wp::where(var_54, var_56, var_28);
            var_62 = wp::where(var_54, var_57, var_31);
            var_63 = wp::where(var_54, var_59, var_34);
            var_64 = wp::where(var_54, var_60, var_37);
            // worldelemid += nsweep_in                                                           <L 1466>
            var_65 = wp::add(var_0, var_nsweep_in);
            // if not wp.static(is_self):                                                         <L 1468>
            var_67 = wp::unot(var_66);
            if (var_67) {
                // if _flex_element_aabb_filter(                                                  <L 1469>
                // flex_aabb_min_in[worldid, flexid1],                                            <L 1470>
                var_68 = wp::address(var_flex_aabb_min_in, var_24, var_63);
                // flex_aabb_max_in[worldid, flexid1],                                            <L 1471>
                var_69 = wp::address(var_flex_aabb_max_in, var_24, var_63);
                // flex_aabb_min_in[worldid, flexid2],                                            <L 1472>
                var_70 = wp::address(var_flex_aabb_min_in, var_24, var_64);
                // flex_aabb_max_in[worldid, flexid2],                                            <L 1473>
                var_71 = wp::address(var_flex_aabb_max_in, var_24, var_64);
                var_73 = wp::load(var_68);
                var_74 = wp::load(var_69);
                var_75 = wp::load(var_70);
                var_76 = wp::load(var_71);
                var_72 = _flex_element_aabb_filter_0(var_73, var_74, var_75, var_76);
                if (var_72) {
                    // continue                                                                   <L 1475>
                    wp::assign(var_0, var_65);
                    goto start_while_0;
                }
                var_77 = wp::where(var_72, var_0, var_65);
                // contype1 = flex_contype[flexid1]                                               <L 1477>
                var_78 = wp::address(var_flex_contype, var_63);
                var_80 = wp::load(var_78);
                var_79 = wp::copy(var_80);
                // conaffinity1 = flex_conaffinity[flexid1]                                       <L 1478>
                var_81 = wp::address(var_flex_conaffinity, var_63);
                var_83 = wp::load(var_81);
                var_82 = wp::copy(var_83);
                // contype2 = flex_contype[flexid2]                                               <L 1479>
                var_84 = wp::address(var_flex_contype, var_64);
                var_86 = wp::load(var_84);
                var_85 = wp::copy(var_86);
                // conaffinity2 = flex_conaffinity[flexid2]                                       <L 1480>
                var_87 = wp::address(var_flex_conaffinity, var_64);
                var_89 = wp::load(var_87);
                var_88 = wp::copy(var_89);
                // if not ((contype1 & conaffinity2) or (contype2 & conaffinity1)):               <L 1481>
                var_91 = wp::bit_and(var_79, var_88);
                var_90 = var_91;
                if (!var_90) {
                    var_92 = wp::bit_and(var_85, var_82);
                    var_90 = var_90 || var_92;
                }
                var_93 = wp::unot(var_90);
                if (var_93) {
                    // continue                                                                   <L 1482>
                    wp::assign(var_0, var_77);
                    goto start_while_0;
                }
                var_94 = wp::where(var_93, var_0, var_77);
            }
            var_95 = wp::where(var_67, var_94, var_65);
            // lower1 = aabb_lower_in[worldid, elem1]                                             <L 1484>
            var_96 = wp::address(var_aabb_lower_in, var_24, var_61);
            var_98 = wp::load(var_96);
            var_97 = wp::copy(var_98);
            // upper1 = aabb_upper_in[worldid, elem1]                                             <L 1485>
            var_99 = wp::address(var_aabb_upper_in, var_24, var_61);
            var_101 = wp::load(var_99);
            var_100 = wp::copy(var_101);
            // lower2 = aabb_lower_in[worldid, elem2]                                             <L 1486>
            var_102 = wp::address(var_aabb_lower_in, var_24, var_62);
            var_104 = wp::load(var_102);
            var_103 = wp::copy(var_104);
            // upper2 = aabb_upper_in[worldid, elem2]                                             <L 1487>
            var_105 = wp::address(var_aabb_upper_in, var_24, var_62);
            var_107 = wp::load(var_105);
            var_106 = wp::copy(var_107);
            // if _flex_element_aabb_filter(lower1, upper1, lower2, upper2):                      <L 1489>
            var_108 = _flex_element_aabb_filter_0(var_97, var_100, var_103, var_106);
            if (var_108) {
                // continue                                                                       <L 1490>
                wp::assign(var_0, var_95);
                goto start_while_0;
            }
            var_109 = wp::where(var_108, var_0, var_95);
            // dim1 = flex_dim[flexid1]                                                           <L 1492>
            var_110 = wp::address(var_flex_dim, var_63);
            var_112 = wp::load(var_110);
            var_111 = wp::copy(var_112);
            // vert_adr1 = flex_vertadr[flexid1]                                                  <L 1493>
            var_113 = wp::address(var_flex_vertadr, var_63);
            var_115 = wp::load(var_113);
            var_114 = wp::copy(var_115);
            // elem_adr1 = flex_elemadr[flexid1]                                                  <L 1494>
            var_116 = wp::address(var_flex_elemadr, var_63);
            var_118 = wp::load(var_116);
            var_117 = wp::copy(var_118);
            // e1 = elem1 - elem_adr1                                                             <L 1495>
            var_119 = wp::sub(var_61, var_117);
            // elem_data_idx1 = flex_elemdataadr[flexid1] + e1 * (dim1 + 1)                       <L 1496>
            var_120 = wp::address(var_flex_elemdataadr, var_63);
            var_122 = wp::add(var_111, var_121);
            var_123 = wp::mul(var_119, var_122);
            var_125 = wp::load(var_120);
            var_124 = wp::add(var_125, var_123);
            // v1_indices = _get_element_vertices(flex_elem, dim1, elem_data_idx1)                <L 1497>
            var_126 = _get_element_vertices_0(var_flex_elem, var_111, var_124);
            // if wp.static(is_self):                                                             <L 1499>
            // e2 = elem2 - elem_adr1                                                             <L 1500>
            var_128 = wp::sub(var_62, var_117);
            // elem_data_idx2 = flex_elemdataadr[flexid1] + e2 * (dim1 + 1)                       <L 1501>
            var_129 = wp::address(var_flex_elemdataadr, var_63);
            var_131 = wp::add(var_111, var_130);
            var_132 = wp::mul(var_128, var_131);
            var_134 = wp::load(var_129);
            var_133 = wp::add(var_134, var_132);
            // v2_indices = _get_element_vertices(flex_elem, dim1, elem_data_idx2)                <L 1502>
            var_135 = _get_element_vertices_0(var_flex_elem, var_111, var_133);
            // if _exclude_self_collision(                                                        <L 1503>
            // flex_vertbodyid,                                                                   <L 1504>
            // v1_indices,                                                                        <L 1505>
            // dim1 + 1,                                                                          <L 1506>
            var_137 = wp::add(var_111, var_136);
            // v2_indices,                                                                        <L 1507>
            // dim1 + 1,                                                                          <L 1508>
            var_139 = wp::add(var_111, var_138);
            // vert_adr1,                                                                         <L 1509>
            var_140 = _exclude_self_collision_0(var_flex_vertbodyid, var_126, var_137, var_135, var_139, var_114);
            if (var_140) {
                // continue                                                                       <L 1511>
                wp::assign(var_0, var_109);
                goto start_while_0;
            }
            var_141 = wp::where(var_140, var_0, var_109);
            // if dim1 == 2:                                                                      <L 1538>
            var_143 = (var_111 == var_142);
            if (var_143) {
                // dim2 = dim1 if wp.static(is_self) else flex_dim[flexid2]                       <L 1539>
                // if dim2 == 2 and wp.static(enable_sat):                                        <L 1540>
                var_147 = (var_111 == var_146);
                var_145 = var_147;
                if (var_145) {
                    var_145 = var_145 && var_148;
                }
                if (var_145) {
                    // r1 = flex_radius[flexid1]                                                  <L 1541>
                    var_149 = wp::address(var_flex_radius, var_63);
                    var_151 = wp::load(var_149);
                    var_150 = wp::copy(var_151);
                    // cutoff = float(2.0) * r1                                                   <L 1542>
                    var_153 = wp::float(var_152);
                    var_154 = wp::mul(var_153, var_150);
                    // if not wp.static(is_self):                                                 <L 1543>
                    var_156 = wp::unot(var_155);
                    if (var_156) {
                        // cutoff = (                                                             <L 1544>
                        // r1 + flex_radius[flexid2] + flex_margin[flexid1] + flex_margin[flexid2] + flex_gap[flexid1] + flex_gap[flexid2]       <L 1545>
                        var_157 = wp::address(var_flex_radius, var_64);
                        var_159 = wp::load(var_157);
                        var_158 = wp::add(var_150, var_159);
                        var_160 = wp::address(var_flex_margin, var_63);
                        var_162 = wp::load(var_160);
                        var_161 = wp::add(var_158, var_162);
                        var_163 = wp::address(var_flex_margin, var_64);
                        var_165 = wp::load(var_163);
                        var_164 = wp::add(var_161, var_165);
                        var_166 = wp::address(var_flex_gap, var_63);
                        var_168 = wp::load(var_166);
                        var_167 = wp::add(var_164, var_168);
                        var_169 = wp::address(var_flex_gap, var_64);
                        var_171 = wp::load(var_169);
                        var_170 = wp::add(var_167, var_171);
                    }
                    var_172 = wp::where(var_156, var_170, var_154);
                    // cutoff_sq = cutoff * cutoff                                                <L 1547>
                    var_173 = wp::mul(var_172, var_172);
                    // p0 = flexvert_xpos_in[worldid, vert_adr1 + v1_indices[0]]                  <L 1548>
                    var_175 = wp::extract(var_126, var_174);
                    var_176 = wp::add(var_114, var_175);
                    var_177 = wp::address(var_flexvert_xpos_in, var_24, var_176);
                    var_179 = wp::load(var_177);
                    var_178 = wp::copy(var_179);
                    // p1 = flexvert_xpos_in[worldid, vert_adr1 + v1_indices[1]]                  <L 1549>
                    var_181 = wp::extract(var_126, var_180);
                    var_182 = wp::add(var_114, var_181);
                    var_183 = wp::address(var_flexvert_xpos_in, var_24, var_182);
                    var_185 = wp::load(var_183);
                    var_184 = wp::copy(var_185);
                    // p2 = flexvert_xpos_in[worldid, vert_adr1 + v1_indices[2]]                  <L 1550>
                    var_187 = wp::extract(var_126, var_186);
                    var_188 = wp::add(var_114, var_187);
                    var_189 = wp::address(var_flexvert_xpos_in, var_24, var_188);
                    var_191 = wp::load(var_189);
                    var_190 = wp::copy(var_191);
                    // vadr2 = vert_adr1 if wp.static(is_self) else flex_vertadr[flexid2]         <L 1551>
                    // q0 = flexvert_xpos_in[worldid, vadr2 + v2_indices[0]]                      <L 1552>
                    var_194 = wp::extract(var_135, var_193);
                    var_195 = wp::add(var_114, var_194);
                    var_196 = wp::address(var_flexvert_xpos_in, var_24, var_195);
                    var_198 = wp::load(var_196);
                    var_197 = wp::copy(var_198);
                    // q1 = flexvert_xpos_in[worldid, vadr2 + v2_indices[1]]                      <L 1553>
                    var_200 = wp::extract(var_135, var_199);
                    var_201 = wp::add(var_114, var_200);
                    var_202 = wp::address(var_flexvert_xpos_in, var_24, var_201);
                    var_204 = wp::load(var_202);
                    var_203 = wp::copy(var_204);
                    // q2 = flexvert_xpos_in[worldid, vadr2 + v2_indices[2]]                      <L 1554>
                    var_206 = wp::extract(var_135, var_205);
                    var_207 = wp::add(var_114, var_206);
                    var_208 = wp::address(var_flexvert_xpos_in, var_24, var_207);
                    var_210 = wp::load(var_208);
                    var_209 = wp::copy(var_210);
                    // if _triangle_sat_separated(p0, p1, p2, q0, q1, q2, cutoff_sq):             <L 1555>
                    var_211 = _triangle_sat_separated_0(var_178, var_184, var_190, var_197, var_203, var_209, var_173);
                    if (var_211) {
                        // continue                                                               <L 1556>
                        wp::assign(var_0, var_141);
                        goto start_while_0;
                    }
                    var_212 = wp::where(var_211, var_0, var_141);
                }
                var_213 = wp::where(var_145, var_212, var_141);
            }
            var_214 = wp::where(var_143, var_213, var_141);
            // idx = wp.atomic_add(ncollision_out, 0, 1)                                          <L 1558>
            var_217 = wp::atomic_add(var_ncollision_out, var_215, var_216);
            // if idx >= max_pairs:                                                               <L 1559>
            var_218 = (var_217 >= var_max_pairs);
            if (var_218) {
                // if wp.static(bool(warn_overflow & OverflowType.BROADPHASE)):                   <L 1560>
                // wp.printf(                                                                     <L 1561>
                // "Flex SAP buffer overflow - please increase naconmax beyond %u\n"              <L 1562>
                // max_pairs,                                                                     <L 1564>
                printf(var_220, var_max_pairs);
                // wp.atomic_or(overflow_out, worldid, wp.static(OverflowType.BROADPHASE))        <L 1566>
                var_222 = wp::atomic_or(var_overflow_out, var_24, var_221);
                // return                                                                         <L 1567>
                continue;
            }
            // collision_pair_out[idx] = wp.vec2i(elem1, elem2)                                   <L 1569>
            var_223 = wp::vec_t<2, wp::int32>(var_61, var_62);
            wp::array_store(var_collision_pair_out, var_217, var_223);
            // collision_worldid_out[idx] = worldid                                               <L 1570>
            wp::array_store(var_collision_worldid_out, var_217, var_24);
            wp::assign(var_0, var_214);
        goto start_while_0;
        end_while_0:;
    }
}

