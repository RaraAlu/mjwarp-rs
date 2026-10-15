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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:2286
static CUDA_CALLABLE bool _compare_candidate_0(
    wp::float32 var_dist_i,
    wp::float32 var_dist_j,
    wp::int32 var_cand_idx_i,
    wp::int32 var_cand_idx_j)
{
    //---------
    // primal vars
    bool var_0;
    const bool var_1 = true;
    bool var_2;
    bool var_3;
    bool var_4;
    const bool var_5 = true;
    const bool var_6 = false;
    //---------
    // forward
    // def _compare_candidate(                                                                <L 2287>
    // if dist_j < dist_i:                                                                    <L 2295>
    var_0 = (var_dist_j < var_dist_i);
    if (var_0) {
        // return True                                                                        <L 2296>
        return var_1;
    }
    // if dist_j == dist_i and cand_idx_j < cand_idx_i:                                       <L 2297>
    var_3 = (var_dist_j == var_dist_i);
    var_2 = var_3;
    if (var_2) {
        var_4 = (var_cand_idx_j < var_cand_idx_i);
        var_2 = var_2 && var_4;
    }
    if (var_2) {
        // return True                                                                        <L 2298>
        return var_5;
    }
    // return False                                                                           <L 2299>
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:2653
static CUDA_CALLABLE bool _tie_break_fps_0(
    wp::int32 var_curr_idx,
    wp::int32 var_sel_idx,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    const bool var_2 = true;
    wp::vec_t<2, wp::int32>* var_3;
    const wp::int32 var_4 = 0;
    wp::int32 var_5;
    wp::vec_t<2, wp::int32> var_6;
    wp::vec_t<2, wp::int32>* var_7;
    const wp::int32 var_8 = 1;
    wp::int32 var_9;
    wp::vec_t<2, wp::int32> var_10;
    wp::vec_t<2, wp::int32>* var_11;
    const wp::int32 var_12 = 0;
    wp::int32 var_13;
    wp::vec_t<2, wp::int32> var_14;
    wp::vec_t<2, wp::int32>* var_15;
    const wp::int32 var_16 = 1;
    wp::int32 var_17;
    wp::vec_t<2, wp::int32> var_18;
    bool var_19;
    bool var_20;
    bool var_21;
    bool var_22;
    bool var_23;
    //---------
    // forward
    // def _tie_break_fps(                                                                    <L 2654>
    // if sel_idx < 0:                                                                        <L 2660>
    var_1 = (var_sel_idx < var_0);
    if (var_1) {
        // return True                                                                        <L 2661>
        return var_2;
    }
    // elem1_curr = cand_elem[curr_idx][0]                                                    <L 2663>
    var_3 = wp::address(var_cand_elem, var_curr_idx);
    var_6 = wp::load(var_3);
    var_5 = wp::extract(var_6, var_4);
    // elem2_curr = cand_elem[curr_idx][1]                                                    <L 2664>
    var_7 = wp::address(var_cand_elem, var_curr_idx);
    var_10 = wp::load(var_7);
    var_9 = wp::extract(var_10, var_8);
    // elem1_sel = cand_elem[sel_idx][0]                                                      <L 2666>
    var_11 = wp::address(var_cand_elem, var_sel_idx);
    var_14 = wp::load(var_11);
    var_13 = wp::extract(var_14, var_12);
    // elem2_sel = cand_elem[sel_idx][1]                                                      <L 2667>
    var_15 = wp::address(var_cand_elem, var_sel_idx);
    var_18 = wp::load(var_15);
    var_17 = wp::extract(var_18, var_16);
    // if elem1_curr != elem1_sel:                                                            <L 2669>
    var_19 = (var_5 != var_13);
    if (var_19) {
        // return elem1_curr < elem1_sel                                                      <L 2670>
        var_20 = (var_5 < var_13);
        return var_20;
    }
    // if elem2_curr != elem2_sel:                                                            <L 2671>
    var_21 = (var_9 != var_17);
    if (var_21) {
        // return elem2_curr < elem2_sel                                                      <L 2672>
        var_22 = (var_9 < var_17);
        return var_22;
    }
    // return curr_idx < sel_idx                                                              <L 2673>
    var_23 = (var_curr_idx < var_sel_idx);
    return var_23;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1285
static CUDA_CALLABLE bool _elem_active_0(
    wp::array_t<wp::int32> var_flex_activelayers,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemlayer,
    wp::int32 var_flexid,
    wp::int32 var_elemid)
{
    //---------
    // primal vars
    wp::int32* var_0;
    const wp::int32 var_1 = 3;
    bool var_2;
    wp::int32 var_3;
    const bool var_4 = true;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::int32* var_8;
    wp::int32* var_9;
    bool var_10;
    wp::int32 var_11;
    wp::int32 var_12;
    //---------
    // forward
    // def _elem_active(                                                                      <L 1286>
    // if flex_dim[flexid] < 3:                                                               <L 1296>
    var_0 = wp::address(var_flex_dim, var_flexid);
    var_3 = wp::load(var_0);
    var_2 = (var_3 < var_1);
    if (var_2) {
        // return True                                                                        <L 1297>
        return var_4;
    }
    // return flex_elemlayer[flex_elemadr[flexid] + elemid] < flex_activelayers[flexid]       <L 1298>
    var_5 = wp::address(var_flex_elemadr, var_flexid);
    var_7 = wp::load(var_5);
    var_6 = wp::add(var_7, var_elemid);
    var_8 = wp::address(var_flex_elemlayer, var_6);
    var_9 = wp::address(var_flex_activelayers, var_flexid);
    var_11 = wp::load(var_8);
    var_12 = wp::load(var_9);
    var_10 = (var_11 < var_12);
    return var_10;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:2286
static CUDA_CALLABLE void adj__compare_candidate_0(
    wp::float32 var_dist_i,
    wp::float32 var_dist_j,
    wp::int32 var_cand_idx_i,
    wp::int32 var_cand_idx_j,
    wp::float32 & adj_dist_i,
    wp::float32 & adj_dist_j,
    wp::int32 & adj_cand_idx_i,
    wp::int32 & adj_cand_idx_j,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:2653
static CUDA_CALLABLE void adj__tie_break_fps_0(
    wp::int32 var_curr_idx,
    wp::int32 var_sel_idx,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::int32 & adj_curr_idx,
    wp::int32 & adj_sel_idx,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_cand_elem,
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
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:1285
static CUDA_CALLABLE void adj__elem_active_0(
    wp::array_t<wp::int32> var_flex_activelayers,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemlayer,
    wp::int32 var_flexid,
    wp::int32 var_elemid,
    wp::array_t<wp::int32> & adj_flex_activelayers,
    wp::array_t<wp::int32> & adj_flex_dim,
    wp::array_t<wp::int32> & adj_flex_elemadr,
    wp::array_t<wp::int32> & adj_flex_elemlayer,
    wp::int32 & adj_flexid,
    wp::int32 & adj_elemid,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _filter_flex_candidates_sorted_01c65b1e_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_ncand,
    wp::float32 var_epsilon,
    wp::array_t<wp::int64> var_sort_key,
    wp::array_t<wp::int32> var_sort_val,
    wp::array_t<wp::float32> var_cand_dist,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::int32> var_cand_active_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        bool var_9;
        wp::shape_t* var_10;
        const wp::int32 var_11 = 0;
        wp::int32 var_12;
        wp::shape_t var_13;
        bool var_14;
        const wp::int32 var_15 = 0;
        wp::int32* var_16;
        wp::int32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::int64* var_21;
        wp::int64 var_22;
        wp::int64 var_23;
        wp::int64 var_24;
        wp::int64 var_25;
        wp::int64 var_26;
        wp::int64 var_27;
        const wp::float32 var_28 = 1000000.0;
        wp::float32 var_29;
        wp::float32 var_30;
        wp::int64 var_31;
        wp::int64 var_32;
        wp::int64 var_33;
        wp::vec_t<3, wp::float32>* var_34;
        wp::vec_t<3, wp::float32> var_35;
        wp::vec_t<3, wp::float32> var_36;
        wp::float32* var_37;
        wp::float32 var_38;
        wp::float32 var_39;
        wp::float32 var_40;
        const wp::int32 var_41 = 1;
        wp::int32 var_42;
        const wp::int32 var_43 = 1;
        wp::int32 var_44;
        const wp::int32 var_45 = 0;
        bool var_46;
        wp::int64* var_47;
        wp::int64 var_48;
        wp::int64 var_49;
        wp::int64 var_50;
        wp::int64 var_51;
        bool var_52;
        wp::int64 var_53;
        wp::int64 var_54;
        wp::int64 var_55;
        bool var_56;
        wp::int32* var_57;
        wp::int32 var_58;
        wp::int32 var_59;
        wp::vec_t<3, wp::float32>* var_60;
        wp::vec_t<3, wp::float32> var_61;
        wp::vec_t<3, wp::float32> var_62;
        wp::vec_t<3, wp::float32> var_63;
        wp::float32 var_64;
        bool var_65;
        wp::float32* var_66;
        bool var_67;
        wp::float32 var_68;
        const wp::int32 var_69 = 0;
        const wp::int32 var_70 = 1;
        wp::int32 var_71;
        const wp::int32 var_72 = 1;
        bool var_73;
        const wp::int32 var_74 = 1;
        wp::int32 var_75;
        bool var_76;
        wp::int64* var_77;
        wp::int64 var_78;
        wp::int64 var_79;
        wp::int64 var_80;
        wp::int64 var_81;
        bool var_82;
        wp::int64 var_83;
        wp::int64 var_84;
        wp::int64 var_85;
        wp::int64 var_86;
        bool var_87;
        wp::int64 var_88;
        wp::int64 var_89;
        wp::int32* var_90;
        wp::int32 var_91;
        wp::int32 var_92;
        wp::vec_t<3, wp::float32>* var_93;
        wp::vec_t<3, wp::float32> var_94;
        wp::vec_t<3, wp::float32> var_95;
        wp::vec_t<3, wp::float32> var_96;
        wp::float32 var_97;
        bool var_98;
        wp::float32* var_99;
        bool var_100;
        wp::float32 var_101;
        const wp::int32 var_102 = 0;
        wp::int64 var_103;
        wp::int64 var_104;
        wp::int32 var_105;
        wp::vec_t<3, wp::float32> var_106;
        wp::vec_t<3, wp::float32> var_107;
        wp::int64 var_108;
        wp::int64 var_109;
        wp::int32 var_110;
        wp::vec_t<3, wp::float32> var_111;
        wp::vec_t<3, wp::float32> var_112;
        const wp::int32 var_113 = 1;
        wp::int32 var_114;
        wp::int32 var_115;
        //---------
        // forward
        // def _filter_flex_candidates_sorted(                                                    <L 2307>
        // si = wp.tid()                                                                          <L 2324>
        var_0 = builtin_tid1d();
        // ncand_limit = wp.min(ncand[0], cand_active_out.shape[0])                               <L 2325>
        var_2 = wp::address(var_ncand, var_1);
        var_3 = &(var_cand_active_out.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_8 = wp::load(var_2);
        var_7 = wp::min(var_8, var_5);
        // if si >= ncand_limit:                                                                  <L 2326>
        var_9 = (var_0 >= var_7);
        if (var_9) {
            // if si < cand_active_out.shape[0]:                                                  <L 2327>
            var_10 = &(var_cand_active_out.shape);
            var_13 = wp::load(var_10);
            var_12 = wp::extract(var_13, var_11);
            var_14 = (var_0 < var_12);
            if (var_14) {
                // cand_active_out[sort_val[si]] = 0                                              <L 2328>
                var_16 = wp::address(var_sort_val, var_0);
                var_17 = wp::load(var_16);
                wp::array_store(var_cand_active_out, var_17, var_15);
            }
            // return                                                                             <L 2329>
            return;
        }
        // i = sort_val[si]                                                                       <L 2331>
        var_18 = wp::address(var_sort_val, var_0);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // my_key = sort_key[si]                                                                  <L 2332>
        var_21 = wp::address(var_sort_key, var_0);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // my_group = my_key >> wp.int64(32)                                                      <L 2333>
        var_24 = 32ll;
        var_25 = wp::rshift(var_22, var_24);
        // my_spatial = my_key & wp.int64(0x7FFFFFFF)                                             <L 2334>
        var_26 = 2147483647ll;
        var_27 = wp::bit_and(var_22, var_26);
        // eps_key = wp.int64(wp.ceil(epsilon * 1000000.0)) + wp.int64(1)                         <L 2335>
        var_29 = wp::mul(var_epsilon, var_28);
        var_30 = wp::ceil(var_29);
        var_31 = wp::int64(var_30);
        var_32 = 1ll;
        var_33 = wp::add(var_31, var_32);
        // pos_i = cand_pos[i]                                                                    <L 2336>
        var_34 = wp::address(var_cand_pos, var_19);
        var_36 = wp::load(var_34);
        var_35 = wp::copy(var_36);
        // dist_i = cand_dist[i]                                                                  <L 2337>
        var_37 = wp::address(var_cand_dist, var_19);
        var_39 = wp::load(var_37);
        var_38 = wp::copy(var_39);
        // eps2 = epsilon * epsilon                                                               <L 2338>
        var_40 = wp::mul(var_epsilon, var_epsilon);
        // keep = int(1)                                                                          <L 2340>
        var_42 = wp::int(var_41);
        // j = si - 1                                                                             <L 2343>
        var_44 = wp::sub(var_0, var_43);
        // while j >= 0:                                                                          <L 2344>
        start_while_1:;
        var_46 = (var_44 >= var_45);
        if ((var_46) == false) goto end_while_1;
            // key_j = sort_key[j]                                                                <L 2345>
            var_47 = wp::address(var_sort_key, var_44);
            var_49 = wp::load(var_47);
            var_48 = wp::copy(var_49);
            // if (key_j >> wp.int64(32)) != my_group:                                            <L 2346>
            var_50 = 32ll;
            var_51 = wp::rshift(var_48, var_50);
            var_52 = (var_51 != var_25);
            if (var_52) {
                // break                                                                          <L 2347>
                goto end_while_1;
            }
            // spatial_j = key_j & wp.int64(0x7FFFFFFF)                                           <L 2348>
            var_53 = 2147483647ll;
            var_54 = wp::bit_and(var_48, var_53);
            // if my_spatial - spatial_j >= eps_key:                                              <L 2349>
            var_55 = wp::sub(var_27, var_54);
            var_56 = (var_55 >= var_33);
            if (var_56) {
                // break                                                                          <L 2350>
                goto end_while_1;
            }
            // oj = sort_val[j]                                                                   <L 2352>
            var_57 = wp::address(var_sort_val, var_44);
            var_59 = wp::load(var_57);
            var_58 = wp::copy(var_59);
            // pos_j = cand_pos[oj]                                                               <L 2353>
            var_60 = wp::address(var_cand_pos, var_58);
            var_62 = wp::load(var_60);
            var_61 = wp::copy(var_62);
            // diff = pos_i - pos_j                                                               <L 2354>
            var_63 = wp::sub(var_35, var_61);
            // if wp.dot(diff, diff) < eps2:                                                      <L 2355>
            var_64 = wp::dot(var_63, var_63);
            var_65 = (var_64 < var_40);
            if (var_65) {
                // if _compare_candidate(dist_i, cand_dist[oj], i, oj):                           <L 2356>
                var_66 = wp::address(var_cand_dist, var_58);
                var_68 = wp::load(var_66);
                var_67 = _compare_candidate_0(var_38, var_68, var_19, var_58);
                if (var_67) {
                    // keep = 0                                                                   <L 2357>
                    // break                                                                      <L 2358>
                    wp::assign(var_42, var_69);
                    goto end_while_1;
                }
            }
            // j -= 1                                                                             <L 2359>
            var_71 = wp::sub(var_44, var_70);
            wp::assign(var_44, var_71);
        goto start_while_1;
        end_while_1:;
        // if keep == 1:                                                                          <L 2362>
        var_73 = (var_42 == var_72);
        if (var_73) {
            // j = si + 1                                                                         <L 2363>
            var_75 = wp::add(var_0, var_74);
            // while j < ncand_limit:                                                             <L 2364>
        start_while_3:;
            var_76 = (var_75 < var_7);
        if ((var_76) == false) goto end_while_3;
                // key_j = sort_key[j]                                                            <L 2365>
                var_77 = wp::address(var_sort_key, var_75);
                var_79 = wp::load(var_77);
                var_78 = wp::copy(var_79);
                // if (key_j >> wp.int64(32)) != my_group:                                        <L 2366>
                var_80 = 32ll;
                var_81 = wp::rshift(var_78, var_80);
                var_82 = (var_81 != var_25);
                if (var_82) {
                    // break                                                                      <L 2367>
                    wp::assign(var_48, var_78);
                    goto end_while_3;
                }
                var_83 = wp::where(var_82, var_48, var_78);
                // spatial_j = key_j & wp.int64(0x7FFFFFFF)                                       <L 2368>
                var_84 = 2147483647ll;
                var_85 = wp::bit_and(var_83, var_84);
                // if spatial_j - my_spatial >= eps_key:                                          <L 2369>
                var_86 = wp::sub(var_85, var_27);
                var_87 = (var_86 >= var_33);
                if (var_87) {
                    // break                                                                      <L 2370>
                    wp::assign(var_48, var_83);
                    wp::assign(var_54, var_85);
                    goto end_while_3;
                }
                var_88 = wp::where(var_87, var_48, var_83);
                var_89 = wp::where(var_87, var_54, var_85);
                // oj = sort_val[j]                                                               <L 2372>
                var_90 = wp::address(var_sort_val, var_75);
                var_92 = wp::load(var_90);
                var_91 = wp::copy(var_92);
                // pos_j = cand_pos[oj]                                                           <L 2373>
                var_93 = wp::address(var_cand_pos, var_91);
                var_95 = wp::load(var_93);
                var_94 = wp::copy(var_95);
                // diff = pos_i - pos_j                                                           <L 2374>
                var_96 = wp::sub(var_35, var_94);
                // if wp.dot(diff, diff) < eps2:                                                  <L 2375>
                var_97 = wp::dot(var_96, var_96);
                var_98 = (var_97 < var_40);
                if (var_98) {
                    // if _compare_candidate(dist_i, cand_dist[oj], i, oj):                       <L 2376>
                    var_99 = wp::address(var_cand_dist, var_91);
                    var_101 = wp::load(var_99);
                    var_100 = _compare_candidate_0(var_38, var_101, var_19, var_91);
                    if (var_100) {
                        // keep = 0                                                               <L 2377>
                        // break                                                                  <L 2378>
                        wp::assign(var_42, var_102);
                        wp::assign(var_48, var_88);
                        wp::assign(var_54, var_89);
                        wp::assign(var_58, var_91);
                        wp::assign(var_61, var_94);
                        wp::assign(var_63, var_96);
                        goto end_while_3;
                    }
                    var_103 = wp::where(var_100, var_48, var_88);
                    var_104 = wp::where(var_100, var_54, var_89);
                    var_105 = wp::where(var_100, var_58, var_91);
                    var_106 = wp::where(var_100, var_61, var_94);
                    var_107 = wp::where(var_100, var_63, var_96);
                }
                var_108 = wp::where(var_98, var_103, var_88);
                var_109 = wp::where(var_98, var_104, var_89);
                var_110 = wp::where(var_98, var_105, var_91);
                var_111 = wp::where(var_98, var_106, var_94);
                var_112 = wp::where(var_98, var_107, var_96);
                // j += 1                                                                         <L 2379>
                var_114 = wp::add(var_75, var_113);
                wp::assign(var_75, var_114);
                wp::assign(var_48, var_108);
                wp::assign(var_54, var_109);
                wp::assign(var_58, var_110);
                wp::assign(var_61, var_111);
                wp::assign(var_63, var_112);
        goto start_while_3;
        end_while_3:;
        }
        var_115 = wp::where(var_73, var_75, var_44);
        // cand_active_out[i] = keep                                                              <L 2381>
        wp::array_store(var_cand_active_out, var_19, var_42);
}



extern "C" __global__ void _parallel_fps_update_and_find_max_33cb7821_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_group_start_indices_in,
    wp::array_t<wp::int32> var_flex_num_groups_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_cand_active_sorted,
    wp::array_t<wp::int32> var_sort_val,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::int32> var_selected_cidx,
    wp::array_t<wp::vec_t<3, wp::float32>> var_selected_pos,
    wp::array_t<wp::float32> var_fps_min_dist_out,
    wp::array_t<wp::float32> var_scratch_dist_out,
    wp::array_t<wp::int32> var_scratch_cidx_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        const wp::int32 var_2 = 0;
        wp::int32* var_3;
        bool var_4;
        wp::int32 var_5;
        wp::int32* var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        const wp::int32 var_9 = 0;
        bool var_10;
        const wp::float32 var_11 = -10000000000.0;
        wp::float32 var_12;
        const wp::int32 var_13 = -1;
        wp::int32* var_14;
        wp::int32 var_15;
        wp::int32 var_16;
        const wp::int32 var_17 = 0;
        wp::int32* var_18;
        wp::shape_t* var_19;
        const wp::int32 var_20 = 0;
        wp::int32 var_21;
        wp::shape_t var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        const wp::int32 var_26 = 0;
        wp::int32* var_27;
        const wp::int32 var_28 = 1;
        wp::int32 var_29;
        wp::int32 var_30;
        bool var_31;
        const wp::int32 var_32 = 1;
        wp::int32 var_33;
        wp::int32* var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        wp::int32 var_37;
        wp::vec_t<3, wp::float32>* var_38;
        wp::vec_t<3, wp::float32> var_39;
        wp::vec_t<3, wp::float32> var_40;
        const wp::float32 var_41 = -10000000000.0;
        wp::float32 var_42;
        const wp::int32 var_43 = -1;
        wp::int32 var_44;
        wp::int32 var_45;
        const wp::int32 var_46 = 64;
        wp::range_t var_47;
        wp::int32 var_48;
        wp::int32* var_49;
        const wp::int32 var_50 = 1;
        bool var_51;
        wp::int32 var_52;
        wp::int32* var_53;
        wp::int32 var_54;
        wp::int32 var_55;
        wp::float32* var_56;
        wp::float32 var_57;
        wp::float32 var_58;
        const wp::float32 var_59 = 0.0;
        bool var_60;
        wp::vec_t<3, wp::float32>* var_61;
        wp::vec_t<3, wp::float32> var_62;
        wp::vec_t<3, wp::float32> var_63;
        wp::float32 var_64;
        wp::float32 var_65;
        bool var_66;
        wp::float32 var_67;
        wp::int32 var_68;
        wp::float32 var_69;
        wp::int32 var_70;
        bool var_71;
        bool var_72;
        wp::int32 var_73;
        wp::int32 var_74;
        wp::int32 var_75;
        wp::int32 var_76;
        wp::float32 var_77;
        wp::int32 var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::int32 var_81;
        //---------
        // forward
        // def _parallel_fps_update_and_find_max(                                                 <L 2929>
        // g, tid = wp.tid()                                                                      <L 2945>
        builtin_tid2d(var_0, var_1);
        // if g >= flex_num_groups_in[0]:                                                         <L 2946>
        var_3 = wp::address(var_flex_num_groups_in, var_2);
        var_5 = wp::load(var_3);
        var_4 = (var_0 >= var_5);
        if (var_4) {
            // return                                                                             <L 2947>
            return;
        }
        // new_cidx = selected_cidx[g]                                                            <L 2949>
        var_6 = wp::address(var_selected_cidx, var_0);
        var_8 = wp::load(var_6);
        var_7 = wp::copy(var_8);
        // if new_cidx < 0:                                                                       <L 2950>
        var_10 = (var_7 < var_9);
        if (var_10) {
            // scratch_dist_out[g, tid] = float(-1e10)                                            <L 2951>
            var_12 = wp::float(var_11);
            wp::array_store(var_scratch_dist_out, var_0, var_1, var_12);
            // scratch_cidx_out[g, tid] = -1                                                      <L 2952>
            wp::array_store(var_scratch_cidx_out, var_0, var_1, var_13);
            // return                                                                             <L 2953>
            return;
        }
        // g_start = flex_group_start_indices_in[g]                                               <L 2955>
        var_14 = wp::address(var_flex_group_start_indices_in, var_0);
        var_16 = wp::load(var_14);
        var_15 = wp::copy(var_16);
        // ncand_limit = wp.min(ncand[0], cand_active_sorted.shape[0])                            <L 2956>
        var_18 = wp::address(var_ncand, var_17);
        var_19 = &(var_cand_active_sorted.shape);
        var_22 = wp::load(var_19);
        var_21 = wp::extract(var_22, var_20);
        var_24 = wp::load(var_18);
        var_23 = wp::min(var_24, var_21);
        // g_end = ncand_limit                                                                    <L 2957>
        var_25 = wp::copy(var_23);
        // if g < flex_num_groups_in[0] - 1:                                                      <L 2958>
        var_27 = wp::address(var_flex_num_groups_in, var_26);
        var_30 = wp::load(var_27);
        var_29 = wp::sub(var_30, var_28);
        var_31 = (var_0 < var_29);
        if (var_31) {
            // g_end = wp.min(ncand_limit, flex_group_start_indices_in[g + 1])                    <L 2959>
            var_33 = wp::add(var_0, var_32);
            var_34 = wp::address(var_flex_group_start_indices_in, var_33);
            var_36 = wp::load(var_34);
            var_35 = wp::min(var_23, var_36);
        }
        var_37 = wp::where(var_31, var_35, var_25);
        // new_p = selected_pos[g]                                                                <L 2961>
        var_38 = wp::address(var_selected_pos, var_0);
        var_40 = wp::load(var_38);
        var_39 = wp::copy(var_40);
        // max_d = float(-1e10)                                                                   <L 2963>
        var_42 = wp::float(var_41);
        // sel_cidx = int(-1)                                                                     <L 2964>
        var_44 = wp::int(var_43);
        // for si in range(g_start + tid, g_end, wp.static(_FPS_BLOCK_SIZE)):                     <L 2966>
        var_45 = wp::add(var_15, var_1);
        var_47 = wp::range(var_45, var_37, var_46);
        start_for_2:;
            if (iter_cmp(var_47) == 0) goto end_for_2;
            var_48 = wp::iter_next(var_47);
            // if cand_active_sorted[si] == 1:                                                    <L 2967>
            var_49 = wp::address(var_cand_active_sorted, var_48);
            var_52 = wp::load(var_49);
            var_51 = (var_52 == var_50);
            if (var_51) {
                // c_idx = sort_val[si]                                                           <L 2968>
                var_53 = wp::address(var_sort_val, var_48);
                var_55 = wp::load(var_53);
                var_54 = wp::copy(var_55);
                // md = fps_min_dist_out[c_idx]                                                   <L 2969>
                var_56 = wp::address(var_fps_min_dist_out, var_54);
                var_58 = wp::load(var_56);
                var_57 = wp::copy(var_58);
                // if md > 0.0:                                                                   <L 2970>
                var_60 = (var_57 > var_59);
                if (var_60) {
                    // d_new = wp.length(cand_pos[c_idx] - new_p)                                 <L 2971>
                    var_61 = wp::address(var_cand_pos, var_54);
                    var_63 = wp::load(var_61);
                    var_62 = wp::sub(var_63, var_39);
                    var_64 = wp::length(var_62);
                    // md = wp.min(md, d_new)                                                     <L 2972>
                    var_65 = wp::min(var_57, var_64);
                    // fps_min_dist_out[c_idx] = md                                               <L 2973>
                    wp::array_store(var_fps_min_dist_out, var_54, var_65);
                    // if md > max_d:                                                             <L 2975>
                    var_66 = (var_65 > var_42);
                    if (var_66) {
                        // max_d = md                                                             <L 2976>
                        var_67 = wp::copy(var_65);
                        // sel_cidx = c_idx                                                       <L 2977>
                        var_68 = wp::copy(var_54);
                    }
                    var_69 = wp::where(var_66, var_67, var_42);
                    var_70 = wp::where(var_66, var_68, var_44);
                    if (!var_66) {
                        // elif md == max_d:                                                      <L 2978>
                        var_71 = (var_65 == var_69);
                        if (var_71) {
                            // if _tie_break_fps(c_idx, sel_cidx, cand_elem):                     <L 2979>
                            var_72 = _tie_break_fps_0(var_54, var_70, var_cand_elem);
                            if (var_72) {
                                // sel_cidx = c_idx                                               <L 2980>
                                var_73 = wp::copy(var_54);
                            }
                            var_74 = wp::where(var_72, var_73, var_70);
                        }
                        var_75 = wp::where(var_71, var_74, var_70);
                    }
                    var_76 = wp::where(var_66, var_70, var_75);
                }
                var_77 = wp::where(var_60, var_69, var_42);
                var_78 = wp::where(var_60, var_76, var_44);
                var_79 = wp::where(var_60, var_65, var_57);
            }
            var_80 = wp::where(var_51, var_77, var_42);
            var_81 = wp::where(var_51, var_78, var_44);
            wp::assign(var_42, var_80);
            wp::assign(var_44, var_81);
            goto start_for_2;
        end_for_2:;
        // scratch_dist_out[g, tid] = max_d                                                       <L 2982>
        wp::array_store(var_scratch_dist_out, var_0, var_1, var_42);
        // scratch_cidx_out[g, tid] = sel_cidx                                                    <L 2983>
        wp::array_store(var_scratch_cidx_out, var_0, var_1, var_44);
}



extern "C" __global__ void _parallel_fps_step_condition_3f71e1ad_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_fps_groups_active_in,
    wp::array_t<wp::int32> var_fps_iter_out,
    wp::array_t<wp::int32> var_fps_condition_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        const wp::int32 var_0 = 0;
        wp::int32* var_1;
        const wp::int32 var_2 = 1;
        wp::int32 var_3;
        wp::int32 var_4;
        const wp::int32 var_5 = 0;
        bool var_6;
        const wp::int32 var_7 = 49;
        bool var_8;
        const wp::int32 var_9 = 0;
        wp::int32* var_10;
        const wp::int32 var_11 = 0;
        bool var_12;
        wp::int32 var_13;
        const wp::int32 var_14 = 0;
        const wp::int32 var_15 = 0;
        const wp::int32 var_16 = 1;
        const wp::int32 var_17 = 0;
        //---------
        // forward
        // def _parallel_fps_step_condition(                                                      <L 2761>
        // k = fps_iter_out[0] + 1                                                                <L 2768>
        var_1 = wp::address(var_fps_iter_out, var_0);
        var_4 = wp::load(var_1);
        var_3 = wp::add(var_4, var_2);
        // fps_iter_out[0] = k                                                                    <L 2769>
        wp::array_store(var_fps_iter_out, var_5, var_3);
        // if k >= wp.static(MJ_MAXCONPAIR - 1) or fps_groups_active_in[0] <= 0:                  <L 2770>
        var_8 = (var_3 >= var_7);
        var_6 = var_8;
        if (!var_6) {
            var_10 = wp::address(var_fps_groups_active_in, var_9);
            var_13 = wp::load(var_10);
            var_12 = (var_13 <= var_11);
            var_6 = var_6 || var_12;
        }
        if (var_6) {
            // fps_condition_out[0] = 0                                                           <L 2771>
            wp::array_store(var_fps_condition_out, var_15, var_14);
        }
        if (!var_6) {
            // fps_condition_out[0] = 1                                                           <L 2773>
            wp::array_store(var_fps_condition_out, var_17, var_16);
        }
}



extern "C" __global__ void _compute_filter_key_45254f93_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::int32 var_ngeom,
    wp::int32 var_nflex,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_geom,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_flex,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::int32> var_cand_worldid,
    wp::array_t<wp::int64> var_key_out,
    wp::array_t<wp::int32> var_val_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        bool var_3;
        wp::int32 var_4;
        wp::int64 var_5;
        wp::int32* var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        wp::vec_t<2, wp::int32>* var_9;
        const wp::int32 var_10 = 1;
        wp::int32 var_11;
        wp::vec_t<2, wp::int32> var_12;
        wp::vec_t<2, wp::int32>* var_13;
        const wp::int32 var_14 = 0;
        wp::int32 var_15;
        wp::vec_t<2, wp::int32> var_16;
        wp::vec_t<2, wp::int32>* var_17;
        const wp::int32 var_18 = 0;
        wp::int32 var_19;
        wp::vec_t<2, wp::int32> var_20;
        const wp::int32 var_21 = 0;
        bool var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        wp::int32 var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        const wp::int32 var_37 = 0;
        bool var_38;
        bool var_39;
        wp::int32 var_40;
        const wp::int32 var_41 = 0;
        wp::int32 var_42;
        wp::int64 var_43;
        wp::int64 var_44;
        wp::int64 var_45;
        wp::int64 var_46;
        wp::int64 var_47;
        wp::vec_t<3, wp::float32>* var_48;
        wp::vec_t<3, wp::float32> var_49;
        wp::vec_t<3, wp::float32> var_50;
        const wp::int32 var_51 = 0;
        wp::float32 var_52;
        const wp::int32 var_53 = 1;
        wp::float32 var_54;
        wp::float32 var_55;
        const wp::int32 var_56 = 2;
        wp::float32 var_57;
        wp::float32 var_58;
        const wp::float32 var_59 = 0.5773502691896258;
        wp::float32 var_60;
        const wp::float32 var_61 = 1000000.0;
        wp::float32 var_62;
        wp::int64 var_63;
        wp::int64 var_64;
        wp::int64 var_65;
        wp::int64 var_66;
        wp::int64 var_67;
        wp::int64 var_68;
        wp::int64 var_69;
        wp::int64 var_70;
        wp::int64 var_71;
        //---------
        // forward
        // def _compute_filter_key(                                                               <L 2235>
        // i = wp.tid()                                                                           <L 2253>
        var_0 = builtin_tid1d();
        // if i >= ncand[0]:                                                                      <L 2254>
        var_2 = wp::address(var_ncand, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // key_out[i] = wp.int64(9223372036854775807)  # (largest int64 value): sort unused entries to end       <L 2255>
            var_5 = 9223372036854775807ll;
            wp::array_store(var_key_out, var_0, var_5);
            // val_out[i] = i                                                                     <L 2256>
            wp::array_store(var_val_out, var_0, var_0);
            // return                                                                             <L 2257>
            return;
        }
        // worldid = cand_worldid[i]                                                              <L 2259>
        var_6 = wp::address(var_cand_worldid, var_0);
        var_8 = wp::load(var_6);
        var_7 = wp::copy(var_8);
        // flex_id = cand_flex[i][1]                                                              <L 2260>
        var_9 = wp::address(var_cand_flex, var_0);
        var_12 = wp::load(var_9);
        var_11 = wp::extract(var_12, var_10);
        // geom_id = cand_geom[i][0]                                                              <L 2261>
        var_13 = wp::address(var_cand_geom, var_0);
        var_16 = wp::load(var_13);
        var_15 = wp::extract(var_16, var_14);
        // flex1 = cand_flex[i][0]                                                                <L 2262>
        var_17 = wp::address(var_cand_flex, var_0);
        var_20 = wp::load(var_17);
        var_19 = wp::extract(var_20, var_18);
        // if geom_id >= 0:                                                                       <L 2264>
        var_22 = (var_15 >= var_21);
        if (var_22) {
            // group = geom_id * nflex + flex_id                                                  <L 2265>
            var_23 = wp::mul(var_15, var_nflex);
            var_24 = wp::add(var_23, var_11);
        }
        if (!var_22) {
            // f1 = wp.min(flex1, flex_id)                                                        <L 2267>
            var_25 = wp::min(var_19, var_11);
            // f2 = wp.max(flex1, flex_id)                                                        <L 2268>
            var_26 = wp::max(var_19, var_11);
            // group = ngeom * nflex + f1 * nflex + f2                                            <L 2269>
            var_27 = wp::mul(var_ngeom, var_nflex);
            var_28 = wp::mul(var_25, var_nflex);
            var_29 = wp::add(var_27, var_28);
            var_30 = wp::add(var_29, var_26);
        }
        var_31 = wp::where(var_22, var_24, var_30);
        // group_id = worldid * (ngeom * nflex + nflex * nflex) + group                           <L 2271>
        var_32 = wp::mul(var_ngeom, var_nflex);
        var_33 = wp::mul(var_nflex, var_nflex);
        var_34 = wp::add(var_32, var_33);
        var_35 = wp::mul(var_7, var_34);
        var_36 = wp::add(var_35, var_31);
        // is_self = int(flex1 == flex_id) if geom_id < 0 else 0                                  <L 2272>
        var_38 = (var_15 < var_37);
        if (var_38) {
            var_39 = (var_19 == var_11);
            var_40 = wp::int(var_39);
        }
        if (!var_38) {
        }
        var_42 = wp::where(var_38, var_40, var_41);
        // group_key = (wp.int64(group_id) << wp.int64(1)) | wp.int64(is_self)                    <L 2274>
        var_43 = wp::int64(var_36);
        var_44 = 1ll;
        var_45 = wp::lshift(var_43, var_44);
        var_46 = wp::int64(var_42);
        var_47 = wp::bit_or(var_45, var_46);
        // p = cand_pos[i]                                                                        <L 2278>
        var_48 = wp::address(var_cand_pos, var_0);
        var_50 = wp::load(var_48);
        var_49 = wp::copy(var_50);
        // s = (p[0] + p[1] + p[2]) * wp.static(1.0 / math.sqrt(3.0))                             <L 2279>
        var_52 = wp::extract(var_49, var_51);
        var_54 = wp::extract(var_49, var_53);
        var_55 = wp::add(var_52, var_54);
        var_57 = wp::extract(var_49, var_56);
        var_58 = wp::add(var_55, var_57);
        var_60 = wp::mul(var_58, var_59);
        // spatial_key = wp.clamp(wp.int64(s * 1000000.0) + wp.int64(100000000), wp.int64(0), wp.int64(2147483647))       <L 2280>
        var_62 = wp::mul(var_60, var_61);
        var_63 = wp::int64(var_62);
        var_64 = 100000000ll;
        var_65 = wp::add(var_63, var_64);
        var_66 = 0ll;
        var_67 = 2147483647ll;
        var_68 = wp::clamp(var_65, var_66, var_67);
        // key_out[i] = (group_key << wp.int64(32)) | spatial_key                                 <L 2282>
        var_69 = 32ll;
        var_70 = wp::lshift(var_47, var_69);
        var_71 = wp::bit_or(var_70, var_68);
        wp::array_store(var_key_out, var_0, var_71);
        // val_out[i] = i                                                                         <L 2283>
        wp::array_store(var_val_out, var_0, var_0);
}



extern "C" __global__ void _parallel_fps_check_condition_3226cc51_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_fps_groups_active_in,
    wp::array_t<wp::int32> var_fps_condition_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        bool var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        const wp::int32 var_3 = 50;
        bool var_4;
        wp::int32 var_5;
        const wp::int32 var_6 = 0;
        wp::int32* var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::int32 var_10;
        const wp::int32 var_11 = 1;
        const wp::int32 var_12 = 0;
        const wp::int32 var_13 = 0;
        const wp::int32 var_14 = 0;
        //---------
        // forward
        // def _parallel_fps_check_condition(                                                     <L 2747>
        // if ncand[0] > wp.static(MJ_MAXCONPAIR) and fps_groups_active_in[0] > 0:                <L 2754>
        var_2 = wp::address(var_ncand, var_1);
        var_5 = wp::load(var_2);
        var_4 = (var_5 > var_3);
        var_0 = var_4;
        if (var_0) {
            var_7 = wp::address(var_fps_groups_active_in, var_6);
            var_10 = wp::load(var_7);
            var_9 = (var_10 > var_8);
            var_0 = var_0 && var_9;
        }
        if (var_0) {
            // fps_condition_out[0] = 1                                                           <L 2755>
            wp::array_store(var_fps_condition_out, var_12, var_11);
        }
        if (!var_0) {
            // fps_condition_out[0] = 0                                                           <L 2757>
            wp::array_store(var_fps_condition_out, var_14, var_13);
        }
}



extern "C" __global__ void _populate_active_sorted_fe3eb253_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_sort_val,
    wp::array_t<wp::int32> var_cand_active,
    wp::array_t<wp::int32> var_cand_active_sorted_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        bool var_9;
        wp::int32* var_10;
        wp::int32* var_11;
        wp::int32 var_12;
        wp::int32 var_13;
        wp::shape_t* var_14;
        const wp::int32 var_15 = 0;
        wp::int32 var_16;
        wp::shape_t var_17;
        bool var_18;
        const wp::int32 var_19 = 0;
        //---------
        // forward
        // def _populate_active_sorted(                                                           <L 2569>
        // si = wp.tid()                                                                          <L 2577>
        var_0 = builtin_tid1d();
        // ncand_limit = wp.min(ncand[0], cand_active_sorted_out.shape[0])                        <L 2578>
        var_2 = wp::address(var_ncand, var_1);
        var_3 = &(var_cand_active_sorted_out.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_8 = wp::load(var_2);
        var_7 = wp::min(var_8, var_5);
        // if si < ncand_limit:                                                                   <L 2579>
        var_9 = (var_0 < var_7);
        if (var_9) {
            // cand_active_sorted_out[si] = cand_active[sort_val[si]]                             <L 2580>
            var_10 = wp::address(var_sort_val, var_0);
            var_12 = wp::load(var_10);
            var_11 = wp::address(var_cand_active, var_12);
            var_13 = wp::load(var_11);
            wp::array_store(var_cand_active_sorted_out, var_0, var_13);
        }
        if (!var_9) {
            // if si < cand_active_sorted_out.shape[0]:                                           <L 2582>
            var_14 = &(var_cand_active_sorted_out.shape);
            var_17 = wp::load(var_14);
            var_16 = wp::extract(var_17, var_15);
            var_18 = (var_0 < var_16);
            if (var_18) {
                // cand_active_sorted_out[si] = 0                                                 <L 2583>
                wp::array_store(var_cand_active_sorted_out, var_0, var_19);
            }
        }
}



extern "C" __global__ void _flex_sap_project_09a10101_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_flex_margin,
    wp::array_t<wp::float32> var_flex_gap,
    wp::array_t<wp::int32> var_flex_activelayers,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_flex_elemlayer,
    wp::array_t<wp::float32> var_flex_radius,
    wp::array_t<wp::int32> var_flex_elemflexid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::int32 var_nworld_in,
    wp::int32 var_nelem,
    wp::vec_t<3, wp::float32> var_direction,
    wp::array_t<wp::float32> var_projection_lower_out,
    wp::array_t<wp::float32> var_projection_upper_out,
    wp::array_t<wp::int32> var_sort_index_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_elem_aabb_lower_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_elem_aabb_upper_out,
    wp::array_t<wp::int32> var_segmented_index_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::int32* var_8;
        wp::int32 var_9;
        wp::int32 var_10;
        wp::int32* var_11;
        wp::int32 var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        wp::int32* var_15;
        const wp::int32 var_16 = 1;
        wp::int32 var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::vec_t<4, wp::int32> var_21;
        const wp::int32 var_22 = 0;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::vec_t<3, wp::float32>* var_25;
        wp::vec_t<3, wp::float32> var_26;
        wp::vec_t<3, wp::float32> var_27;
        const wp::int32 var_28 = 1;
        wp::int32 var_29;
        wp::int32 var_30;
        wp::vec_t<3, wp::float32>* var_31;
        wp::vec_t<3, wp::float32> var_32;
        wp::vec_t<3, wp::float32> var_33;
        wp::vec_t<3, wp::float32> var_34;
        wp::vec_t<3, wp::float32> var_35;
        const wp::int32 var_36 = 2;
        bool var_37;
        const wp::int32 var_38 = 2;
        wp::int32 var_39;
        wp::int32 var_40;
        wp::vec_t<3, wp::float32>* var_41;
        wp::vec_t<3, wp::float32> var_42;
        wp::vec_t<3, wp::float32> var_43;
        wp::vec_t<3, wp::float32> var_44;
        wp::vec_t<3, wp::float32> var_45;
        wp::vec_t<3, wp::float32> var_46;
        wp::vec_t<3, wp::float32> var_47;
        const wp::int32 var_48 = 3;
        bool var_49;
        const wp::int32 var_50 = 3;
        wp::int32 var_51;
        wp::int32 var_52;
        wp::vec_t<3, wp::float32>* var_53;
        wp::vec_t<3, wp::float32> var_54;
        wp::vec_t<3, wp::float32> var_55;
        wp::vec_t<3, wp::float32> var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::float32* var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        wp::float32* var_63;
        wp::float32 var_64;
        wp::float32 var_65;
        wp::float32* var_66;
        wp::float32 var_67;
        wp::float32 var_68;
        wp::float32 var_69;
        wp::float32 var_70;
        wp::vec_t<3, wp::float32> var_71;
        wp::vec_t<3, wp::float32> var_72;
        wp::vec_t<3, wp::float32> var_73;
        const wp::float32 var_74 = 0.5;
        wp::vec_t<3, wp::float32> var_75;
        wp::vec_t<3, wp::float32> var_76;
        const wp::float32 var_77 = 0.5;
        wp::vec_t<3, wp::float32> var_78;
        wp::vec_t<3, wp::float32> var_79;
        wp::float32 var_80;
        const wp::int32 var_81 = 0;
        wp::float32 var_82;
        wp::float32 var_83;
        const wp::int32 var_84 = 0;
        wp::float32 var_85;
        wp::float32 var_86;
        const wp::int32 var_87 = 1;
        wp::float32 var_88;
        wp::float32 var_89;
        const wp::int32 var_90 = 1;
        wp::float32 var_91;
        wp::float32 var_92;
        wp::float32 var_93;
        const wp::int32 var_94 = 2;
        wp::float32 var_95;
        wp::float32 var_96;
        const wp::int32 var_97 = 2;
        wp::float32 var_98;
        wp::float32 var_99;
        wp::float32 var_100;
        bool var_101;
        bool var_102;
        const wp::float32 var_103 = 10000000000.0;
        wp::float32 var_104;
        wp::float32 var_105;
        const wp::int32 var_106 = 0;
        bool var_107;
        wp::int32 var_108;
        const wp::int32 var_109 = 1;
        wp::int32 var_110;
        bool var_111;
        wp::int32 var_112;
        //---------
        // forward
        // def _flex_sap_project(                                                                 <L 1302>
        // worldid, elemid = wp.tid()                                                             <L 1329>
        builtin_tid2d(var_0, var_1);
        // flexid = flex_elemflexid[elemid]                                                       <L 1331>
        var_2 = wp::address(var_flex_elemflexid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // sort_index_out[worldid, elemid] = elemid                                               <L 1334>
        wp::array_store(var_sort_index_out, var_0, var_1, var_1);
        // dim = flex_dim[flexid]                                                                 <L 1337>
        var_5 = wp::address(var_flex_dim, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // vert_adr = flex_vertadr[flexid]                                                        <L 1338>
        var_8 = wp::address(var_flex_vertadr, var_3);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // elem_adr = flex_elemadr[flexid]                                                        <L 1339>
        var_11 = wp::address(var_flex_elemadr, var_3);
        var_13 = wp::load(var_11);
        var_12 = wp::copy(var_13);
        // e = elemid - elem_adr                                                                  <L 1340>
        var_14 = wp::sub(var_1, var_12);
        // elem_data_idx = flex_elemdataadr[flexid] + e * (dim + 1)                               <L 1341>
        var_15 = wp::address(var_flex_elemdataadr, var_3);
        var_17 = wp::add(var_6, var_16);
        var_18 = wp::mul(var_14, var_17);
        var_20 = wp::load(var_15);
        var_19 = wp::add(var_20, var_18);
        // v = _get_element_vertices(flex_elem, dim, elem_data_idx)                               <L 1343>
        var_21 = _get_element_vertices_0(var_flex_elem, var_6, var_19);
        // p0 = flexvert_xpos_in[worldid, vert_adr + v[0]]                                        <L 1345>
        var_23 = wp::extract(var_21, var_22);
        var_24 = wp::add(var_9, var_23);
        var_25 = wp::address(var_flexvert_xpos_in, var_0, var_24);
        var_27 = wp::load(var_25);
        var_26 = wp::copy(var_27);
        // p1 = flexvert_xpos_in[worldid, vert_adr + v[1]]                                        <L 1346>
        var_29 = wp::extract(var_21, var_28);
        var_30 = wp::add(var_9, var_29);
        var_31 = wp::address(var_flexvert_xpos_in, var_0, var_30);
        var_33 = wp::load(var_31);
        var_32 = wp::copy(var_33);
        // aabb_min = wp.min(p0, p1)                                                              <L 1348>
        var_34 = wp::min(var_26, var_32);
        // aabb_max = wp.max(p0, p1)                                                              <L 1349>
        var_35 = wp::max(var_26, var_32);
        // if dim >= 2:                                                                           <L 1351>
        var_37 = (var_6 >= var_36);
        if (var_37) {
            // p2 = flexvert_xpos_in[worldid, vert_adr + v[2]]                                    <L 1352>
            var_39 = wp::extract(var_21, var_38);
            var_40 = wp::add(var_9, var_39);
            var_41 = wp::address(var_flexvert_xpos_in, var_0, var_40);
            var_43 = wp::load(var_41);
            var_42 = wp::copy(var_43);
            // aabb_min = wp.min(aabb_min, p2)                                                    <L 1353>
            var_44 = wp::min(var_34, var_42);
            // aabb_max = wp.max(aabb_max, p2)                                                    <L 1354>
            var_45 = wp::max(var_35, var_42);
        }
        var_46 = wp::where(var_37, var_44, var_34);
        var_47 = wp::where(var_37, var_45, var_35);
        // if dim >= 3:                                                                           <L 1355>
        var_49 = (var_6 >= var_48);
        if (var_49) {
            // p3 = flexvert_xpos_in[worldid, vert_adr + v[3]]                                    <L 1356>
            var_51 = wp::extract(var_21, var_50);
            var_52 = wp::add(var_9, var_51);
            var_53 = wp::address(var_flexvert_xpos_in, var_0, var_52);
            var_55 = wp::load(var_53);
            var_54 = wp::copy(var_55);
            // aabb_min = wp.min(aabb_min, p3)                                                    <L 1357>
            var_56 = wp::min(var_46, var_54);
            // aabb_max = wp.max(aabb_max, p3)                                                    <L 1358>
            var_57 = wp::max(var_47, var_54);
        }
        var_58 = wp::where(var_49, var_56, var_46);
        var_59 = wp::where(var_49, var_57, var_47);
        // radius = flex_radius[flexid]                                                           <L 1360>
        var_60 = wp::address(var_flex_radius, var_3);
        var_62 = wp::load(var_60);
        var_61 = wp::copy(var_62);
        // margin = flex_margin[flexid]                                                           <L 1361>
        var_63 = wp::address(var_flex_margin, var_3);
        var_65 = wp::load(var_63);
        var_64 = wp::copy(var_65);
        // gap = flex_gap[flexid]                                                                 <L 1362>
        var_66 = wp::address(var_flex_gap, var_3);
        var_68 = wp::load(var_66);
        var_67 = wp::copy(var_68);
        // rbound = radius + margin + gap                                                         <L 1363>
        var_69 = wp::add(var_61, var_64);
        var_70 = wp::add(var_69, var_67);
        // inflate = wp.vec3(rbound, rbound, rbound)                                              <L 1364>
        var_71 = wp::vec_t<3, wp::float32>(var_70, var_70, var_70);
        // aabb_min = aabb_min - inflate                                                          <L 1365>
        var_72 = wp::sub(var_58, var_71);
        // aabb_max = aabb_max + inflate                                                          <L 1366>
        var_73 = wp::add(var_59, var_71);
        // elem_aabb_lower_out[worldid, elemid] = aabb_min                                        <L 1368>
        wp::array_store(var_elem_aabb_lower_out, var_0, var_1, var_72);
        // elem_aabb_upper_out[worldid, elemid] = aabb_max                                        <L 1369>
        wp::array_store(var_elem_aabb_upper_out, var_0, var_1, var_73);
        // center = 0.5 * (aabb_min + aabb_max)                                                   <L 1372>
        var_75 = wp::add(var_72, var_73);
        var_76 = wp::mul(var_74, var_75);
        // halfsize = 0.5 * (aabb_max - aabb_min)                                                 <L 1373>
        var_78 = wp::sub(var_73, var_72);
        var_79 = wp::mul(var_77, var_78);
        // proj_center = wp.dot(direction, center)                                                <L 1374>
        var_80 = wp::dot(var_direction, var_76);
        // proj_radius = wp.abs(direction[0]) * halfsize[0] + wp.abs(direction[1]) * halfsize[1] + wp.abs(direction[2]) * halfsize[2]       <L 1375>
        var_82 = wp::extract(var_direction, var_81);
        var_83 = wp::abs(var_82);
        var_85 = wp::extract(var_79, var_84);
        var_86 = wp::mul(var_83, var_85);
        var_88 = wp::extract(var_direction, var_87);
        var_89 = wp::abs(var_88);
        var_91 = wp::extract(var_79, var_90);
        var_92 = wp::mul(var_89, var_91);
        var_93 = wp::add(var_86, var_92);
        var_95 = wp::extract(var_direction, var_94);
        var_96 = wp::abs(var_95);
        var_98 = wp::extract(var_79, var_97);
        var_99 = wp::mul(var_96, var_98);
        var_100 = wp::add(var_93, var_99);
        // if not _elem_active(flex_activelayers, flex_dim, flex_elemadr, flex_elemlayer, flexid, e):       <L 1377>
        var_101 = _elem_active_0(var_flex_activelayers, var_flex_dim, var_flex_elemadr, var_flex_elemlayer, var_3, var_14);
        var_102 = wp::unot(var_101);
        if (var_102) {
            // projection_lower_out[worldid, elemid] = MJ_MAXVAL                                  <L 1378>
            wp::array_store(var_projection_lower_out, var_0, var_1, var_103);
            // projection_upper_out[worldid, elemid] = MJ_MAXVAL                                  <L 1379>
            wp::array_store(var_projection_upper_out, var_0, var_1, var_103);
        }
        if (!var_102) {
            // projection_lower_out[worldid, elemid] = proj_center - proj_radius                  <L 1381>
            var_104 = wp::sub(var_80, var_100);
            wp::array_store(var_projection_lower_out, var_0, var_1, var_104);
            // projection_upper_out[worldid, elemid] = proj_center + proj_radius                  <L 1382>
            var_105 = wp::add(var_80, var_100);
            wp::array_store(var_projection_upper_out, var_0, var_1, var_105);
        }
        // if elemid == 0:                                                                        <L 1385>
        var_107 = (var_1 == var_106);
        if (var_107) {
            // segmented_index_out[worldid] = worldid * nelem                                     <L 1386>
            var_108 = wp::mul(var_0, var_nelem);
            wp::array_store(var_segmented_index_out, var_0, var_108);
            // if worldid == nworld_in - 1:                                                       <L 1387>
            var_110 = wp::sub(var_nworld_in, var_109);
            var_111 = (var_0 == var_110);
            if (var_111) {
                // segmented_index_out[nworld_in] = nworld_in * nelem                             <L 1388>
                var_112 = wp::mul(var_nworld_in, var_nelem);
                wp::array_store(var_segmented_index_out, var_nworld_in, var_112);
            }
        }
}



extern "C" __global__ void _find_group_starts_d42e5537_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int64> var_sort_key,
    wp::array_t<wp::int32> var_flex_group_temp_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        bool var_9;
        const wp::int32 var_10 = 0;
        bool var_11;
        const wp::int32 var_12 = 1;
        wp::int64* var_13;
        wp::int64 var_14;
        wp::int64 var_15;
        wp::int64 var_16;
        wp::int64 var_17;
        const wp::int32 var_18 = 1;
        wp::int32 var_19;
        wp::int64* var_20;
        wp::int64 var_21;
        wp::int64 var_22;
        wp::int64 var_23;
        wp::int64 var_24;
        bool var_25;
        const wp::int32 var_26 = 1;
        const wp::int32 var_27 = 0;
        //---------
        // forward
        // def _find_group_starts(                                                                <L 2587>
        // si = wp.tid()                                                                          <L 2594>
        var_0 = builtin_tid1d();
        // ncand_limit = wp.min(ncand[0], flex_group_temp_out.shape[0])                           <L 2595>
        var_2 = wp::address(var_ncand, var_1);
        var_3 = &(var_flex_group_temp_out.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_8 = wp::load(var_2);
        var_7 = wp::min(var_8, var_5);
        // if si >= ncand_limit:                                                                  <L 2596>
        var_9 = (var_0 >= var_7);
        if (var_9) {
            // return                                                                             <L 2597>
            return;
        }
        // if si == 0:                                                                            <L 2599>
        var_11 = (var_0 == var_10);
        if (var_11) {
            // flex_group_temp_out[si] = 1                                                        <L 2600>
            wp::array_store(var_flex_group_temp_out, var_0, var_12);
        }
        if (!var_11) {
            // my_key = sort_key[si]                                                              <L 2602>
            var_13 = wp::address(var_sort_key, var_0);
            var_15 = wp::load(var_13);
            var_14 = wp::copy(var_15);
            // my_group = my_key >> wp.int64(32)                                                  <L 2603>
            var_16 = 32ll;
            var_17 = wp::rshift(var_14, var_16);
            // prev_key = sort_key[si - 1]                                                        <L 2604>
            var_19 = wp::sub(var_0, var_18);
            var_20 = wp::address(var_sort_key, var_19);
            var_22 = wp::load(var_20);
            var_21 = wp::copy(var_22);
            // prev_group = prev_key >> wp.int64(32)                                              <L 2605>
            var_23 = 32ll;
            var_24 = wp::rshift(var_21, var_23);
            // if my_group != prev_group:                                                         <L 2606>
            var_25 = (var_17 != var_24);
            if (var_25) {
                // flex_group_temp_out[si] = 1                                                    <L 2607>
                wp::array_store(var_flex_group_temp_out, var_0, var_26);
            }
            if (!var_25) {
                // flex_group_temp_out[si] = 0                                                    <L 2609>
                wp::array_store(var_flex_group_temp_out, var_0, var_27);
            }
        }
}



extern "C" __global__ void _parallel_fps_init_condition_535d7884_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_fps_groups_active_out,
    wp::array_t<wp::int32> var_fps_iter_out,
    wp::array_t<wp::int32> var_fps_condition_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        const wp::int32 var_0 = 0;
        const wp::int32 var_1 = 0;
        const wp::int32 var_2 = 0;
        const wp::int32 var_3 = 0;
        const wp::int32 var_4 = 0;
        const wp::int32 var_5 = 0;
        //---------
        // forward
        // def _parallel_fps_init_condition(                                                      <L 2735>
        // fps_groups_active_out[0] = 0                                                           <L 2741>
        wp::array_store(var_fps_groups_active_out, var_1, var_0);
        // fps_iter_out[0] = 0                                                                    <L 2742>
        wp::array_store(var_fps_iter_out, var_3, var_2);
        // fps_condition_out[0] = 0                                                               <L 2743>
        wp::array_store(var_fps_condition_out, var_5, var_4);
}



extern "C" __global__ void _parallel_fps_resolve_seed_28a97e89_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_flex_num_groups_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::float32> var_scratch_dist_in,
    wp::array_t<wp::int32> var_scratch_cidx_in,
    wp::array_t<wp::int32> var_scratch_count_in,
    wp::array_t<wp::int32> var_selected_cidx_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_selected_pos_out,
    wp::array_t<wp::int32> var_fps_groups_active_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = -1;
        bool var_2;
        const wp::int32 var_3 = 0;
        wp::int32* var_4;
        bool var_5;
        wp::int32 var_6;
        const wp::int32 var_7 = 0;
        wp::int32* var_8;
        const wp::int32 var_9 = 50;
        bool var_10;
        wp::int32 var_11;
        const wp::int32 var_12 = 0;
        wp::int32 var_13;
        const wp::int32 var_14 = 64;
        wp::range_t var_15;
        wp::int32 var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        bool var_20;
        const wp::int32 var_21 = -1;
        const wp::int32 var_22 = 0;
        const wp::int32 var_23 = 1;
        wp::int32 var_24;
        const wp::float32 var_25 = 10000000000.0;
        wp::float32 var_26;
        const wp::int32 var_27 = -1;
        wp::int32 var_28;
        const wp::int32 var_29 = 64;
        wp::range_t var_30;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        const wp::int32 var_35 = 0;
        bool var_36;
        wp::float32* var_37;
        wp::float32 var_38;
        wp::float32 var_39;
        bool var_40;
        wp::float32 var_41;
        wp::int32 var_42;
        wp::float32 var_43;
        wp::int32 var_44;
        bool var_45;
        bool var_46;
        wp::int32 var_47;
        wp::int32 var_48;
        wp::int32 var_49;
        wp::int32 var_50;
        wp::float32 var_51;
        wp::int32 var_52;
        const wp::int32 var_53 = 0;
        bool var_54;
        wp::vec_t<3, wp::float32>* var_55;
        wp::vec_t<3, wp::float32> var_56;
        //---------
        // forward
        // def _parallel_fps_resolve_seed(                                                        <L 2777>
        // g = wp.tid()                                                                           <L 2791>
        var_0 = builtin_tid1d();
        // selected_cidx_out[g] = -1                                                              <L 2792>
        wp::array_store(var_selected_cidx_out, var_0, var_1);
        // if g >= flex_num_groups_in[0] or ncand[0] <= MJ_MAXCONPAIR:                            <L 2793>
        var_4 = wp::address(var_flex_num_groups_in, var_3);
        var_6 = wp::load(var_4);
        var_5 = (var_0 >= var_6);
        var_2 = var_5;
        if (!var_2) {
            var_8 = wp::address(var_ncand, var_7);
            var_11 = wp::load(var_8);
            var_10 = (var_11 <= var_9);
            var_2 = var_2 || var_10;
        }
        if (var_2) {
            // return                                                                             <L 2794>
            return;
        }
        // total_active = int(0)                                                                  <L 2796>
        var_13 = wp::int(var_12);
        // for t in range(wp.static(_FPS_BLOCK_SIZE)):                                            <L 2797>
        var_15 = wp::range(var_14);
        start_for_1:;
            if (iter_cmp(var_15) == 0) goto end_for_1;
            var_16 = wp::iter_next(var_15);
            // total_active += scratch_count_in[g, t]                                             <L 2798>
            var_17 = wp::address(var_scratch_count_in, var_0, var_16);
            var_19 = wp::load(var_17);
            var_18 = wp::add(var_13, var_19);
            wp::assign(var_13, var_18);
            goto start_for_1;
        end_for_1:;
        // if total_active <= MJ_MAXCONPAIR:                                                      <L 2800>
        var_20 = (var_13 <= var_9);
        if (var_20) {
            // selected_cidx_out[g] = -1                                                          <L 2801>
            wp::array_store(var_selected_cidx_out, var_0, var_21);
            // return                                                                             <L 2802>
            return;
        }
        // wp.atomic_add(fps_groups_active_out, 0, 1)                                             <L 2804>
        var_24 = wp::atomic_add(var_fps_groups_active_out, var_22, var_23);
        // min_d = float(1e10)                                                                    <L 2806>
        var_26 = wp::float(var_25);
        // sel_cidx = int(-1)                                                                     <L 2807>
        var_28 = wp::int(var_27);
        // for t in range(wp.static(_FPS_BLOCK_SIZE)):                                            <L 2808>
        var_30 = wp::range(var_29);
        start_for_4:;
            if (iter_cmp(var_30) == 0) goto end_for_4;
            var_31 = wp::iter_next(var_30);
            // c_idx = scratch_cidx_in[g, t]                                                      <L 2809>
            var_32 = wp::address(var_scratch_cidx_in, var_0, var_31);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // if c_idx >= 0:                                                                     <L 2810>
            var_36 = (var_33 >= var_35);
            if (var_36) {
                // d_val = scratch_dist_in[g, t]                                                  <L 2811>
                var_37 = wp::address(var_scratch_dist_in, var_0, var_31);
                var_39 = wp::load(var_37);
                var_38 = wp::copy(var_39);
                // if d_val < min_d:                                                              <L 2812>
                var_40 = (var_38 < var_26);
                if (var_40) {
                    // min_d = d_val                                                              <L 2813>
                    var_41 = wp::copy(var_38);
                    // sel_cidx = c_idx                                                           <L 2814>
                    var_42 = wp::copy(var_33);
                }
                var_43 = wp::where(var_40, var_41, var_26);
                var_44 = wp::where(var_40, var_42, var_28);
                if (!var_40) {
                    // elif d_val == min_d:                                                       <L 2815>
                    var_45 = (var_38 == var_43);
                    if (var_45) {
                        // if _tie_break_fps(c_idx, sel_cidx, cand_elem):                         <L 2816>
                        var_46 = _tie_break_fps_0(var_33, var_44, var_cand_elem);
                        if (var_46) {
                            // sel_cidx = c_idx                                                   <L 2817>
                            var_47 = wp::copy(var_33);
                        }
                        var_48 = wp::where(var_46, var_47, var_44);
                    }
                    var_49 = wp::where(var_45, var_48, var_44);
                }
                var_50 = wp::where(var_40, var_44, var_49);
            }
            var_51 = wp::where(var_36, var_43, var_26);
            var_52 = wp::where(var_36, var_50, var_28);
            wp::assign(var_26, var_51);
            wp::assign(var_28, var_52);
            goto start_for_4;
        end_for_4:;
        // selected_cidx_out[g] = sel_cidx                                                        <L 2819>
        wp::array_store(var_selected_cidx_out, var_0, var_28);
        // if sel_cidx >= 0:                                                                      <L 2820>
        var_54 = (var_28 >= var_53);
        if (var_54) {
            // selected_pos_out[g] = cand_pos[sel_cidx]                                           <L 2821>
            var_55 = wp::address(var_cand_pos, var_28);
            var_56 = wp::load(var_55);
            wp::array_store(var_selected_pos_out, var_0, var_56);
        }
}



extern "C" __global__ void _parallel_fps_init_dist_and_find_max_49a8bf1c_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_group_start_indices_in,
    wp::array_t<wp::int32> var_flex_num_groups_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_cand_active_sorted,
    wp::array_t<wp::int32> var_sort_val,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::int32> var_selected_cidx,
    wp::array_t<wp::vec_t<3, wp::float32>> var_selected_pos,
    wp::array_t<wp::float32> var_fps_min_dist_out,
    wp::array_t<wp::int32> var_cand_active_out,
    wp::array_t<wp::float32> var_scratch_dist_out,
    wp::array_t<wp::int32> var_scratch_cidx_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        const wp::int32 var_2 = 0;
        wp::int32* var_3;
        bool var_4;
        wp::int32 var_5;
        wp::int32* var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        const wp::int32 var_9 = 0;
        bool var_10;
        const wp::float32 var_11 = -10000000000.0;
        wp::float32 var_12;
        const wp::int32 var_13 = -1;
        wp::int32* var_14;
        wp::int32 var_15;
        wp::int32 var_16;
        const wp::int32 var_17 = 0;
        wp::int32* var_18;
        wp::shape_t* var_19;
        const wp::int32 var_20 = 0;
        wp::int32 var_21;
        wp::shape_t var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        const wp::int32 var_26 = 0;
        wp::int32* var_27;
        const wp::int32 var_28 = 1;
        wp::int32 var_29;
        wp::int32 var_30;
        bool var_31;
        const wp::int32 var_32 = 1;
        wp::int32 var_33;
        wp::int32* var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        wp::int32 var_37;
        wp::vec_t<3, wp::float32>* var_38;
        wp::vec_t<3, wp::float32> var_39;
        wp::vec_t<3, wp::float32> var_40;
        const wp::float32 var_41 = -10000000000.0;
        wp::float32 var_42;
        const wp::int32 var_43 = -1;
        wp::int32 var_44;
        wp::int32 var_45;
        const wp::int32 var_46 = 64;
        wp::range_t var_47;
        wp::int32 var_48;
        wp::int32* var_49;
        const wp::int32 var_50 = 1;
        bool var_51;
        wp::int32 var_52;
        wp::int32* var_53;
        wp::int32 var_54;
        wp::int32 var_55;
        bool var_56;
        const wp::int32 var_57 = 1;
        const wp::float32 var_58 = -10000000000.0;
        const wp::int32 var_59 = 0;
        wp::vec_t<3, wp::float32>* var_60;
        wp::vec_t<3, wp::float32> var_61;
        wp::vec_t<3, wp::float32> var_62;
        wp::float32 var_63;
        bool var_64;
        wp::float32 var_65;
        wp::int32 var_66;
        wp::float32 var_67;
        wp::int32 var_68;
        bool var_69;
        bool var_70;
        wp::int32 var_71;
        wp::int32 var_72;
        wp::int32 var_73;
        wp::int32 var_74;
        wp::float32 var_75;
        wp::int32 var_76;
        wp::float32 var_77;
        wp::int32 var_78;
        //---------
        // forward
        // def _parallel_fps_init_dist_and_find_max(                                              <L 2825>
        // g, tid = wp.tid()                                                                      <L 2842>
        builtin_tid2d(var_0, var_1);
        // if g >= flex_num_groups_in[0]:                                                         <L 2843>
        var_3 = wp::address(var_flex_num_groups_in, var_2);
        var_5 = wp::load(var_3);
        var_4 = (var_0 >= var_5);
        if (var_4) {
            // return                                                                             <L 2844>
            return;
        }
        // seed_idx = selected_cidx[g]                                                            <L 2846>
        var_6 = wp::address(var_selected_cidx, var_0);
        var_8 = wp::load(var_6);
        var_7 = wp::copy(var_8);
        // if seed_idx < 0:                                                                       <L 2847>
        var_10 = (var_7 < var_9);
        if (var_10) {
            // scratch_dist_out[g, tid] = float(-1e10)                                            <L 2848>
            var_12 = wp::float(var_11);
            wp::array_store(var_scratch_dist_out, var_0, var_1, var_12);
            // scratch_cidx_out[g, tid] = -1                                                      <L 2849>
            wp::array_store(var_scratch_cidx_out, var_0, var_1, var_13);
            // return                                                                             <L 2850>
            return;
        }
        // g_start = flex_group_start_indices_in[g]                                               <L 2852>
        var_14 = wp::address(var_flex_group_start_indices_in, var_0);
        var_16 = wp::load(var_14);
        var_15 = wp::copy(var_16);
        // ncand_limit = wp.min(ncand[0], cand_active_sorted.shape[0])                            <L 2853>
        var_18 = wp::address(var_ncand, var_17);
        var_19 = &(var_cand_active_sorted.shape);
        var_22 = wp::load(var_19);
        var_21 = wp::extract(var_22, var_20);
        var_24 = wp::load(var_18);
        var_23 = wp::min(var_24, var_21);
        // g_end = ncand_limit                                                                    <L 2854>
        var_25 = wp::copy(var_23);
        // if g < flex_num_groups_in[0] - 1:                                                      <L 2855>
        var_27 = wp::address(var_flex_num_groups_in, var_26);
        var_30 = wp::load(var_27);
        var_29 = wp::sub(var_30, var_28);
        var_31 = (var_0 < var_29);
        if (var_31) {
            // g_end = wp.min(ncand_limit, flex_group_start_indices_in[g + 1])                    <L 2856>
            var_33 = wp::add(var_0, var_32);
            var_34 = wp::address(var_flex_group_start_indices_in, var_33);
            var_36 = wp::load(var_34);
            var_35 = wp::min(var_23, var_36);
        }
        var_37 = wp::where(var_31, var_35, var_25);
        // seed_p = selected_pos[g]                                                               <L 2858>
        var_38 = wp::address(var_selected_pos, var_0);
        var_40 = wp::load(var_38);
        var_39 = wp::copy(var_40);
        // max_d = float(-1e10)                                                                   <L 2859>
        var_42 = wp::float(var_41);
        // sel_cidx = int(-1)                                                                     <L 2860>
        var_44 = wp::int(var_43);
        // for si in range(g_start + tid, g_end, wp.static(_FPS_BLOCK_SIZE)):                     <L 2862>
        var_45 = wp::add(var_15, var_1);
        var_47 = wp::range(var_45, var_37, var_46);
        start_for_2:;
            if (iter_cmp(var_47) == 0) goto end_for_2;
            var_48 = wp::iter_next(var_47);
            // if cand_active_sorted[si] == 1:                                                    <L 2863>
            var_49 = wp::address(var_cand_active_sorted, var_48);
            var_52 = wp::load(var_49);
            var_51 = (var_52 == var_50);
            if (var_51) {
                // c_idx = sort_val[si]                                                           <L 2864>
                var_53 = wp::address(var_sort_val, var_48);
                var_55 = wp::load(var_53);
                var_54 = wp::copy(var_55);
                // if c_idx == seed_idx:                                                          <L 2865>
                var_56 = (var_54 == var_7);
                if (var_56) {
                    // cand_active_out[c_idx] = 1                                                 <L 2866>
                    wp::array_store(var_cand_active_out, var_54, var_57);
                    // fps_min_dist_out[c_idx] = -1e10                                            <L 2867>
                    wp::array_store(var_fps_min_dist_out, var_54, var_58);
                }
                if (!var_56) {
                    // cand_active_out[c_idx] = 0                                                 <L 2869>
                    wp::array_store(var_cand_active_out, var_54, var_59);
                    // d = wp.length(cand_pos[c_idx] - seed_p)                                    <L 2870>
                    var_60 = wp::address(var_cand_pos, var_54);
                    var_62 = wp::load(var_60);
                    var_61 = wp::sub(var_62, var_39);
                    var_63 = wp::length(var_61);
                    // fps_min_dist_out[c_idx] = d                                                <L 2871>
                    wp::array_store(var_fps_min_dist_out, var_54, var_63);
                    // if d > max_d:                                                              <L 2872>
                    var_64 = (var_63 > var_42);
                    if (var_64) {
                        // max_d = d                                                              <L 2873>
                        var_65 = wp::copy(var_63);
                        // sel_cidx = c_idx                                                       <L 2874>
                        var_66 = wp::copy(var_54);
                    }
                    var_67 = wp::where(var_64, var_65, var_42);
                    var_68 = wp::where(var_64, var_66, var_44);
                    if (!var_64) {
                        // elif d == max_d:                                                       <L 2875>
                        var_69 = (var_63 == var_67);
                        if (var_69) {
                            // if _tie_break_fps(c_idx, sel_cidx, cand_elem):                     <L 2876>
                            var_70 = _tie_break_fps_0(var_54, var_68, var_cand_elem);
                            if (var_70) {
                                // sel_cidx = c_idx                                               <L 2877>
                                var_71 = wp::copy(var_54);
                            }
                            var_72 = wp::where(var_70, var_71, var_68);
                        }
                        var_73 = wp::where(var_69, var_72, var_68);
                    }
                    var_74 = wp::where(var_64, var_68, var_73);
                }
                var_75 = wp::where(var_56, var_42, var_67);
                var_76 = wp::where(var_56, var_44, var_74);
            }
            var_77 = wp::where(var_51, var_75, var_42);
            var_78 = wp::where(var_51, var_76, var_44);
            wp::assign(var_42, var_77);
            wp::assign(var_44, var_78);
            goto start_for_2;
        end_for_2:;
        // scratch_dist_out[g, tid] = max_d                                                       <L 2879>
        wp::array_store(var_scratch_dist_out, var_0, var_1, var_42);
        // scratch_cidx_out[g, tid] = sel_cidx                                                    <L 2880>
        wp::array_store(var_scratch_cidx_out, var_0, var_1, var_44);
}



extern "C" __global__ void _parallel_fps_resolve_max_b732d115_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_flex_num_groups_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::float32> var_scratch_dist_in,
    wp::array_t<wp::int32> var_scratch_cidx_in,
    wp::array_t<wp::int32> var_selected_cidx_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_selected_pos_out,
    wp::array_t<wp::int32> var_cand_active_out,
    wp::array_t<wp::float32> var_fps_min_dist_out,
    wp::array_t<wp::int32> var_fps_groups_active_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        const wp::int32 var_1 = 0;
        wp::int32* var_2;
        bool var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        const wp::int32 var_6 = 0;
        bool var_7;
        wp::int32 var_8;
        const wp::float32 var_9 = -10000000000.0;
        wp::float32 var_10;
        const wp::int32 var_11 = -1;
        wp::int32 var_12;
        const wp::int32 var_13 = 64;
        wp::range_t var_14;
        wp::int32 var_15;
        wp::int32* var_16;
        wp::int32 var_17;
        wp::int32 var_18;
        const wp::int32 var_19 = 0;
        bool var_20;
        wp::float32* var_21;
        wp::float32 var_22;
        wp::float32 var_23;
        bool var_24;
        wp::float32 var_25;
        wp::int32 var_26;
        wp::float32 var_27;
        wp::int32 var_28;
        bool var_29;
        bool var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        wp::float32 var_35;
        wp::int32 var_36;
        bool var_37;
        const wp::int32 var_38 = 0;
        bool var_39;
        const wp::float32 var_40 = 0.0;
        bool var_41;
        wp::vec_t<3, wp::float32>* var_42;
        wp::vec_t<3, wp::float32> var_43;
        const wp::int32 var_44 = 1;
        const wp::float32 var_45 = -10000000000.0;
        const wp::int32 var_46 = -1;
        const wp::int32 var_47 = 0;
        const wp::int32 var_48 = 1;
        wp::int32 var_49;
        //---------
        // forward
        // def _parallel_fps_resolve_max(                                                         <L 2884>
        // g = wp.tid()                                                                           <L 2898>
        var_0 = builtin_tid1d();
        // if g >= flex_num_groups_in[0]:                                                         <L 2899>
        var_2 = wp::address(var_flex_num_groups_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 2900>
            return;
        }
        // if selected_cidx_out[g] < 0:                                                           <L 2902>
        var_5 = wp::address(var_selected_cidx_out, var_0);
        var_8 = wp::load(var_5);
        var_7 = (var_8 < var_6);
        if (var_7) {
            // return                                                                             <L 2903>
            return;
        }
        // max_d = float(-1e10)                                                                   <L 2905>
        var_10 = wp::float(var_9);
        // sel_cidx = int(-1)                                                                     <L 2906>
        var_12 = wp::int(var_11);
        // for t in range(wp.static(_FPS_BLOCK_SIZE)):                                            <L 2907>
        var_14 = wp::range(var_13);
        start_for_2:;
            if (iter_cmp(var_14) == 0) goto end_for_2;
            var_15 = wp::iter_next(var_14);
            // c_idx = scratch_cidx_in[g, t]                                                      <L 2908>
            var_16 = wp::address(var_scratch_cidx_in, var_0, var_15);
            var_18 = wp::load(var_16);
            var_17 = wp::copy(var_18);
            // if c_idx >= 0:                                                                     <L 2909>
            var_20 = (var_17 >= var_19);
            if (var_20) {
                // md = scratch_dist_in[g, t]                                                     <L 2910>
                var_21 = wp::address(var_scratch_dist_in, var_0, var_15);
                var_23 = wp::load(var_21);
                var_22 = wp::copy(var_23);
                // if md > max_d:                                                                 <L 2911>
                var_24 = (var_22 > var_10);
                if (var_24) {
                    // max_d = md                                                                 <L 2912>
                    var_25 = wp::copy(var_22);
                    // sel_cidx = c_idx                                                           <L 2913>
                    var_26 = wp::copy(var_17);
                }
                var_27 = wp::where(var_24, var_25, var_10);
                var_28 = wp::where(var_24, var_26, var_12);
                if (!var_24) {
                    // elif md == max_d:                                                          <L 2914>
                    var_29 = (var_22 == var_27);
                    if (var_29) {
                        // if _tie_break_fps(c_idx, sel_cidx, cand_elem):                         <L 2915>
                        var_30 = _tie_break_fps_0(var_17, var_28, var_cand_elem);
                        if (var_30) {
                            // sel_cidx = c_idx                                                   <L 2916>
                            var_31 = wp::copy(var_17);
                        }
                        var_32 = wp::where(var_30, var_31, var_28);
                    }
                    var_33 = wp::where(var_29, var_32, var_28);
                }
                var_34 = wp::where(var_24, var_28, var_33);
            }
            var_35 = wp::where(var_20, var_27, var_10);
            var_36 = wp::where(var_20, var_34, var_12);
            wp::assign(var_10, var_35);
            wp::assign(var_12, var_36);
            goto start_for_2;
        end_for_2:;
        // if sel_cidx >= 0 and max_d > 0.0:                                                      <L 2918>
        var_39 = (var_12 >= var_38);
        var_37 = var_39;
        if (var_37) {
            var_41 = (var_10 > var_40);
            var_37 = var_37 && var_41;
        }
        if (var_37) {
            // selected_cidx_out[g] = sel_cidx                                                    <L 2919>
            wp::array_store(var_selected_cidx_out, var_0, var_12);
            // selected_pos_out[g] = cand_pos[sel_cidx]                                           <L 2920>
            var_42 = wp::address(var_cand_pos, var_12);
            var_43 = wp::load(var_42);
            wp::array_store(var_selected_pos_out, var_0, var_43);
            // cand_active_out[sel_cidx] = 1                                                      <L 2921>
            wp::array_store(var_cand_active_out, var_12, var_44);
            // fps_min_dist_out[sel_cidx] = -1e10                                                 <L 2922>
            wp::array_store(var_fps_min_dist_out, var_12, var_45);
        }
        if (!var_37) {
            // selected_cidx_out[g] = -1                                                          <L 2924>
            wp::array_store(var_selected_cidx_out, var_0, var_46);
            // wp.atomic_sub(fps_groups_active_out, 0, 1)                                         <L 2925>
            var_49 = wp::atomic_sub(var_fps_groups_active_out, var_47, var_48);
        }
}



extern "C" __global__ void _flex_broadphase_bounds_18aa0451_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_flex_margin,
    wp::array_t<wp::float32> var_flex_gap,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_vertnum,
    wp::array_t<wp::float32> var_flex_radius,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_aabb_min_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_aabb_max_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        const wp::float32 var_10 = 10000000000.0;
        wp::vec_t<3, wp::float32> var_11;
        const wp::float32 var_12 = -10000000000.0;
        const wp::float32 var_13 = -10000000000.0;
        const wp::float32 var_14 = -10000000000.0;
        wp::vec_t<3, wp::float32> var_15;
        wp::range_t var_16;
        wp::int32 var_17;
        wp::int32 var_18;
        wp::vec_t<3, wp::float32>* var_19;
        wp::vec_t<3, wp::float32> var_20;
        wp::vec_t<3, wp::float32> var_21;
        wp::vec_t<3, wp::float32> var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::float32* var_24;
        wp::float32* var_25;
        wp::float32 var_26;
        wp::float32 var_27;
        wp::float32 var_28;
        wp::float32* var_29;
        wp::float32 var_30;
        wp::float32 var_31;
        wp::vec_t<3, wp::float32> var_32;
        wp::vec_t<3, wp::float32> var_33;
        wp::vec_t<3, wp::float32> var_34;
        //---------
        // forward
        // def _flex_broadphase_bounds(                                                           <L 241>
        // worldid, flexid = wp.tid()                                                             <L 254>
        builtin_tid2d(var_0, var_1);
        // start = flex_vertadr[flexid]                                                           <L 256>
        var_2 = wp::address(var_flex_vertadr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // num = flex_vertnum[flexid]                                                             <L 257>
        var_5 = wp::address(var_flex_vertnum, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if num == 0:                                                                           <L 258>
        var_9 = (var_6 == var_8);
        if (var_9) {
            // return                                                                             <L 259>
            return;
        }
        // min_bound = wp.vec3(MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL)                                   <L 261>
        var_11 = wp::vec_t<3, wp::float32>(var_10, var_10, var_10);
        // max_bound = wp.vec3(-MJ_MAXVAL, -MJ_MAXVAL, -MJ_MAXVAL)                                <L 262>
        var_15 = wp::vec_t<3, wp::float32>(var_12, var_13, var_14);
        // for i in range(num):                                                                   <L 264>
        var_16 = wp::range(var_6);
        start_for_1:;
            if (iter_cmp(var_16) == 0) goto end_for_1;
            var_17 = wp::iter_next(var_16);
            // pos = flexvert_xpos_in[worldid, start + i]                                         <L 265>
            var_18 = wp::add(var_3, var_17);
            var_19 = wp::address(var_flexvert_xpos_in, var_0, var_18);
            var_21 = wp::load(var_19);
            var_20 = wp::copy(var_21);
            // min_bound = wp.min(min_bound, pos)                                                 <L 266>
            var_22 = wp::min(var_11, var_20);
            // max_bound = wp.max(max_bound, pos)                                                 <L 267>
            var_23 = wp::max(var_15, var_20);
            wp::assign(var_11, var_22);
            wp::assign(var_15, var_23);
            goto start_for_1;
        end_for_1:;
        // margin = flex_margin[flexid] + flex_gap[flexid]                                        <L 269>
        var_24 = wp::address(var_flex_margin, var_1);
        var_25 = wp::address(var_flex_gap, var_1);
        var_27 = wp::load(var_24);
        var_28 = wp::load(var_25);
        var_26 = wp::add(var_27, var_28);
        // bound = flex_radius[flexid] + margin                                                   <L 270>
        var_29 = wp::address(var_flex_radius, var_1);
        var_31 = wp::load(var_29);
        var_30 = wp::add(var_31, var_26);
        // inflate = wp.vec3(bound, bound, bound)                                                 <L 271>
        var_32 = wp::vec_t<3, wp::float32>(var_30, var_30, var_30);
        // flex_aabb_min_out[worldid, flexid] = min_bound - inflate                               <L 273>
        var_33 = wp::sub(var_11, var_32);
        wp::array_store(var_flex_aabb_min_out, var_0, var_1, var_33);
        // flex_aabb_max_out[worldid, flexid] = max_bound + inflate                               <L 274>
        var_34 = wp::add(var_15, var_32);
        wp::array_store(var_flex_aabb_max_out, var_0, var_1, var_34);
}



extern "C" __global__ void _parallel_fps_find_seed_dc19c645_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_group_start_indices_in,
    wp::array_t<wp::int32> var_flex_num_groups_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_cand_active_sorted,
    wp::array_t<wp::int32> var_sort_val,
    wp::array_t<wp::float32> var_cand_dist,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_geom,
    wp::array_t<wp::float32> var_scratch_dist_out,
    wp::array_t<wp::int32> var_scratch_cidx_out,
    wp::array_t<wp::int32> var_scratch_count_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        bool var_2;
        const wp::int32 var_3 = 0;
        wp::int32* var_4;
        bool var_5;
        wp::int32 var_6;
        const wp::int32 var_7 = 0;
        wp::int32* var_8;
        const wp::int32 var_9 = 50;
        bool var_10;
        wp::int32 var_11;
        const wp::int32 var_12 = 0;
        wp::int32* var_13;
        wp::shape_t* var_14;
        const wp::int32 var_15 = 0;
        wp::int32 var_16;
        wp::shape_t var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32* var_20;
        wp::int32 var_21;
        wp::int32 var_22;
        bool var_23;
        const wp::int32 var_24 = 0;
        bool var_25;
        bool var_26;
        const wp::int32 var_27 = 0;
        const wp::int32 var_28 = -1;
        wp::int32* var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        wp::vec_t<2, wp::int32>* var_32;
        const wp::int32 var_33 = 0;
        wp::int32 var_34;
        wp::vec_t<2, wp::int32> var_35;
        const wp::int32 var_36 = 0;
        bool var_37;
        const wp::int32 var_38 = 0;
        const wp::int32 var_39 = -1;
        wp::int32 var_40;
        const wp::int32 var_41 = 0;
        wp::int32* var_42;
        const wp::int32 var_43 = 1;
        wp::int32 var_44;
        wp::int32 var_45;
        bool var_46;
        const wp::int32 var_47 = 1;
        wp::int32 var_48;
        wp::int32* var_49;
        wp::int32 var_50;
        wp::int32 var_51;
        wp::int32 var_52;
        const wp::int32 var_53 = 0;
        wp::int32 var_54;
        const wp::float32 var_55 = 10000000000.0;
        wp::float32 var_56;
        const wp::int32 var_57 = -1;
        wp::int32 var_58;
        wp::int32 var_59;
        const wp::int32 var_60 = 64;
        wp::range_t var_61;
        wp::int32 var_62;
        wp::int32* var_63;
        const wp::int32 var_64 = 1;
        bool var_65;
        wp::int32 var_66;
        const wp::int32 var_67 = 1;
        wp::int32 var_68;
        wp::int32* var_69;
        wp::int32 var_70;
        wp::int32 var_71;
        wp::float32* var_72;
        wp::float32 var_73;
        wp::float32 var_74;
        bool var_75;
        wp::float32 var_76;
        wp::int32 var_77;
        wp::float32 var_78;
        wp::int32 var_79;
        bool var_80;
        bool var_81;
        wp::int32 var_82;
        wp::int32 var_83;
        wp::int32 var_84;
        wp::int32 var_85;
        wp::int32 var_86;
        wp::float32 var_87;
        wp::int32 var_88;
        //---------
        // forward
        // def _parallel_fps_find_seed(                                                           <L 2677>
        // g, tid = wp.tid()                                                                      <L 2692>
        builtin_tid2d(var_0, var_1);
        // if g >= flex_num_groups_in[0] or ncand[0] <= MJ_MAXCONPAIR:                            <L 2693>
        var_4 = wp::address(var_flex_num_groups_in, var_3);
        var_6 = wp::load(var_4);
        var_5 = (var_0 >= var_6);
        var_2 = var_5;
        if (!var_2) {
            var_8 = wp::address(var_ncand, var_7);
            var_11 = wp::load(var_8);
            var_10 = (var_11 <= var_9);
            var_2 = var_2 || var_10;
        }
        if (var_2) {
            // return                                                                             <L 2694>
            return;
        }
        // ncand_limit = wp.min(ncand[0], cand_active_sorted.shape[0])                            <L 2696>
        var_13 = wp::address(var_ncand, var_12);
        var_14 = &(var_cand_active_sorted.shape);
        var_17 = wp::load(var_14);
        var_16 = wp::extract(var_17, var_15);
        var_19 = wp::load(var_13);
        var_18 = wp::min(var_19, var_16);
        // g_start = flex_group_start_indices_in[g]                                               <L 2697>
        var_20 = wp::address(var_flex_group_start_indices_in, var_0);
        var_22 = wp::load(var_20);
        var_21 = wp::copy(var_22);
        // if g_start < 0 or g_start >= ncand_limit:                                              <L 2698>
        var_25 = (var_21 < var_24);
        var_23 = var_25;
        if (!var_23) {
            var_26 = (var_21 >= var_18);
            var_23 = var_23 || var_26;
        }
        if (var_23) {
            // scratch_count_out[g, tid] = 0                                                      <L 2699>
            wp::array_store(var_scratch_count_out, var_0, var_1, var_27);
            // scratch_cidx_out[g, tid] = -1                                                      <L 2700>
            wp::array_store(var_scratch_cidx_out, var_0, var_1, var_28);
            // return                                                                             <L 2701>
            return;
        }
        // first_cand_idx = sort_val[g_start]                                                     <L 2703>
        var_29 = wp::address(var_sort_val, var_21);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
        // if cand_geom[first_cand_idx][0] >= 0:                                                  <L 2704>
        var_32 = wp::address(var_cand_geom, var_30);
        var_35 = wp::load(var_32);
        var_34 = wp::extract(var_35, var_33);
        var_37 = (var_34 >= var_36);
        if (var_37) {
            // scratch_count_out[g, tid] = 0                                                      <L 2705>
            wp::array_store(var_scratch_count_out, var_0, var_1, var_38);
            // scratch_cidx_out[g, tid] = -1                                                      <L 2706>
            wp::array_store(var_scratch_cidx_out, var_0, var_1, var_39);
            // return                                                                             <L 2707>
            return;
        }
        // g_end = ncand_limit                                                                    <L 2709>
        var_40 = wp::copy(var_18);
        // if g < flex_num_groups_in[0] - 1:                                                      <L 2710>
        var_42 = wp::address(var_flex_num_groups_in, var_41);
        var_45 = wp::load(var_42);
        var_44 = wp::sub(var_45, var_43);
        var_46 = (var_0 < var_44);
        if (var_46) {
            // g_end = wp.min(ncand_limit, flex_group_start_indices_in[g + 1])                    <L 2711>
            var_48 = wp::add(var_0, var_47);
            var_49 = wp::address(var_flex_group_start_indices_in, var_48);
            var_51 = wp::load(var_49);
            var_50 = wp::min(var_18, var_51);
        }
        var_52 = wp::where(var_46, var_50, var_40);
        // local_active = int(0)                                                                  <L 2713>
        var_54 = wp::int(var_53);
        // min_d = float(1e10)                                                                    <L 2714>
        var_56 = wp::float(var_55);
        // sel_cidx = int(-1)                                                                     <L 2715>
        var_58 = wp::int(var_57);
        // for si in range(g_start + tid, g_end, wp.static(_FPS_BLOCK_SIZE)):                     <L 2717>
        var_59 = wp::add(var_21, var_1);
        var_61 = wp::range(var_59, var_52, var_60);
        start_for_3:;
            if (iter_cmp(var_61) == 0) goto end_for_3;
            var_62 = wp::iter_next(var_61);
            // if cand_active_sorted[si] == 1:                                                    <L 2718>
            var_63 = wp::address(var_cand_active_sorted, var_62);
            var_66 = wp::load(var_63);
            var_65 = (var_66 == var_64);
            if (var_65) {
                // local_active += 1                                                              <L 2719>
                var_68 = wp::add(var_54, var_67);
                // c_idx = sort_val[si]                                                           <L 2720>
                var_69 = wp::address(var_sort_val, var_62);
                var_71 = wp::load(var_69);
                var_70 = wp::copy(var_71);
                // d_val = cand_dist[c_idx]                                                       <L 2721>
                var_72 = wp::address(var_cand_dist, var_70);
                var_74 = wp::load(var_72);
                var_73 = wp::copy(var_74);
                // if d_val < min_d:                                                              <L 2722>
                var_75 = (var_73 < var_56);
                if (var_75) {
                    // min_d = d_val                                                              <L 2723>
                    var_76 = wp::copy(var_73);
                    // sel_cidx = c_idx                                                           <L 2724>
                    var_77 = wp::copy(var_70);
                }
                var_78 = wp::where(var_75, var_76, var_56);
                var_79 = wp::where(var_75, var_77, var_58);
                if (!var_75) {
                    // elif d_val == min_d:                                                       <L 2725>
                    var_80 = (var_73 == var_78);
                    if (var_80) {
                        // if _tie_break_fps(c_idx, sel_cidx, cand_elem):                         <L 2726>
                        var_81 = _tie_break_fps_0(var_70, var_79, var_cand_elem);
                        if (var_81) {
                            // sel_cidx = c_idx                                                   <L 2727>
                            var_82 = wp::copy(var_70);
                        }
                        var_83 = wp::where(var_81, var_82, var_79);
                    }
                    var_84 = wp::where(var_80, var_83, var_79);
                }
                var_85 = wp::where(var_75, var_79, var_84);
            }
            var_86 = wp::where(var_65, var_68, var_54);
            var_87 = wp::where(var_65, var_78, var_56);
            var_88 = wp::where(var_65, var_85, var_58);
            wp::assign(var_54, var_86);
            wp::assign(var_56, var_87);
            wp::assign(var_58, var_88);
            goto start_for_3;
        end_for_3:;
        // scratch_dist_out[g, tid] = min_d                                                       <L 2729>
        wp::array_store(var_scratch_dist_out, var_0, var_1, var_56);
        // scratch_cidx_out[g, tid] = sel_cidx                                                    <L 2730>
        wp::array_store(var_scratch_cidx_out, var_0, var_1, var_58);
        // scratch_count_out[g, tid] = local_active                                               <L 2731>
        wp::array_store(var_scratch_count_out, var_0, var_1, var_54);
}

