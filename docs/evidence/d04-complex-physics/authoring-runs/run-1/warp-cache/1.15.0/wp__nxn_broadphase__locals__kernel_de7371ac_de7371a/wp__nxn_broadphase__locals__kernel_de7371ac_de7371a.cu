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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:114
static CUDA_CALLABLE bool _sphere_filter_0(
    wp::float32 var_size1,
    wp::float32 var_size2,
    wp::float32 var_margin1,
    wp::float32 var_margin2,
    wp::vec_t<3, wp::float32> var_xpos1,
    wp::vec_t<3, wp::float32> var_xpos2)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    bool var_6;
    //---------
    // forward
    // def _sphere_filter(size1: float, size2: float, margin1: float, margin2: float, xpos1: wp.vec3, xpos2: wp.vec3) -> bool:       <L 115>
    // bound = size1 + size2 + margin1 + margin2                                              <L 116>
    var_0 = wp::add(var_size1, var_size2);
    var_1 = wp::add(var_0, var_margin1);
    var_2 = wp::add(var_1, var_margin2);
    // dif = xpos2 - xpos1                                                                    <L 117>
    var_3 = wp::sub(var_xpos2, var_xpos1);
    // dist_sq = wp.dot(dif, dif)                                                             <L 118>
    var_4 = wp::dot(var_3, var_3);
    // return dist_sq <= bound * bound                                                        <L 119>
    var_5 = wp::mul(var_2, var_2);
    var_6 = (var_4 <= var_5);
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:283
static CUDA_CALLABLE bool _broadphase_filter__locals__func_2(
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_aabb,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_geom1,
    wp::int32 var_geom2,
    wp::int32 var_worldid)
{
    //---------
    // primal vars
    const bool var_0 = false;
    const wp::int32 var_1 = 0;
    const wp::int32 var_2 = 0;
    wp::vec_t<3, wp::float32>* var_3;
    const wp::int32 var_4 = 0;
    wp::vec_t<3, wp::float32>* var_5;
    const wp::int32 var_6 = 1;
    wp::vec_t<3, wp::float32>* var_7;
    const wp::int32 var_8 = 1;
    wp::vec_t<3, wp::float32>* var_9;
    const bool var_10 = false;
    const wp::int32 var_11 = 0;
    wp::float32* var_12;
    wp::float32* var_13;
    const bool var_14 = false;
    const wp::int32 var_15 = 0;
    wp::float32* var_16;
    wp::float32* var_17;
    const bool var_18 = false;
    const wp::int32 var_19 = 0;
    wp::float32* var_20;
    wp::float32* var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::vec_t<3, wp::float32>* var_28;
    wp::vec_t<3, wp::float32>* var_29;
    wp::mat_t<3, 3, wp::float32>* var_30;
    wp::mat_t<3, 3, wp::float32>* var_31;
    bool var_32;
    const wp::float32 var_33 = 0.0;
    bool var_34;
    wp::float32 var_35;
    const wp::float32 var_36 = 0.0;
    bool var_37;
    wp::float32 var_38;
    const wp::int32 var_39 = 0;
    const wp::int32 var_40 = 2;
    bool var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    wp::vec_t<3, wp::float32> var_44;
    wp::vec_t<3, wp::float32> var_45;
    bool var_46;
    const bool var_47 = false;
    const wp::int32 var_48 = 0;
    const wp::int32 var_49 = 0;
    const bool var_50 = true;
    //---------
    // forward
    // def func(                                                                              <L 284>
    // aabb_id = worldid % ngeom_aabb if wp.static(ngeom_aabb > 1) else 0                     <L 303>
    // center1, center2 = geom_aabb[aabb_id, geom1, 0], geom_aabb[aabb_id, geom2, 0]  # kernel_analyzer: ignore       <L 304>
    var_3 = wp::address(var_geom_aabb, var_1, var_geom1, var_2);
    var_5 = wp::address(var_geom_aabb, var_1, var_geom2, var_4);
    // size1, size2 = geom_aabb[aabb_id, geom1, 1], geom_aabb[aabb_id, geom2, 1]  # kernel_analyzer: ignore       <L 305>
    var_7 = wp::address(var_geom_aabb, var_1, var_geom1, var_6);
    var_9 = wp::address(var_geom_aabb, var_1, var_geom2, var_8);
    // rbound_id = worldid % ngeom_rbound if wp.static(ngeom_rbound > 1) else 0               <L 307>
    // rbound1, rbound2 = geom_rbound[rbound_id, geom1], geom_rbound[rbound_id, geom2]  # kernel_analyzer: ignore       <L 308>
    var_12 = wp::address(var_geom_rbound, var_11, var_geom1);
    var_13 = wp::address(var_geom_rbound, var_11, var_geom2);
    // margin_id = worldid % ngeom_margin if wp.static(ngeom_margin > 1) else 0               <L 309>
    // margin1, margin2 = geom_margin[margin_id, geom1], geom_margin[margin_id, geom2]  # kernel_analyzer: ignore       <L 310>
    var_16 = wp::address(var_geom_margin, var_15, var_geom1);
    var_17 = wp::address(var_geom_margin, var_15, var_geom2);
    // gap_id = worldid % ngeom_gap if wp.static(ngeom_gap > 1) else 0                        <L 311>
    // gap1, gap2 = geom_gap[gap_id, geom1], geom_gap[gap_id, geom2]  # kernel_analyzer: ignore       <L 312>
    var_20 = wp::address(var_geom_gap, var_19, var_geom1);
    var_21 = wp::address(var_geom_gap, var_19, var_geom2);
    // effective_margin1 = margin1 + gap1                                                     <L 313>
    var_23 = wp::load(var_16);
    var_24 = wp::load(var_20);
    var_22 = wp::add(var_23, var_24);
    // effective_margin2 = margin2 + gap2                                                     <L 314>
    var_26 = wp::load(var_17);
    var_27 = wp::load(var_21);
    var_25 = wp::add(var_26, var_27);
    // xpos1, xpos2 = geom_xpos_in[worldid, geom1], geom_xpos_in[worldid, geom2]              <L 315>
    var_28 = wp::address(var_geom_xpos_in, var_worldid, var_geom1);
    var_29 = wp::address(var_geom_xpos_in, var_worldid, var_geom2);
    // xmat1, xmat2 = geom_xmat_in[worldid, geom1], geom_xmat_in[worldid, geom2]              <L 316>
    var_30 = wp::address(var_geom_xmat_in, var_worldid, var_geom1);
    var_31 = wp::address(var_geom_xmat_in, var_worldid, var_geom2);
    // if rbound1 == 0.0 or rbound2 == 0.0:                                                   <L 318>
    var_35 = wp::load(var_12);
    var_34 = (var_35 == var_33);
    var_32 = var_34;
    if (!var_32) {
        var_38 = wp::load(var_13);
        var_37 = (var_38 == var_36);
        var_32 = var_32 || var_37;
    }
    if (var_32) {
        // if wp.static(opt_broadphase_filter & BroadphaseFilter.PLANE):                      <L 319>
    }
    if (!var_32) {
        // if wp.static(opt_broadphase_filter & BroadphaseFilter.SPHERE):                     <L 322>
        // if not _sphere_filter(rbound1, rbound2, effective_margin1, effective_margin2, xpos1, xpos2):       <L 323>
        var_42 = wp::load(var_12);
        var_43 = wp::load(var_13);
        var_44 = wp::load(var_28);
        var_45 = wp::load(var_29);
        var_41 = _sphere_filter_0(var_42, var_43, var_22, var_25, var_44, var_45);
        var_46 = wp::unot(var_41);
        if (var_46) {
            // return False                                                                   <L 324>
            return var_47;
        }
        // if wp.static(opt_broadphase_filter & BroadphaseFilter.AABB):                       <L 325>
        // if wp.static(opt_broadphase_filter & BroadphaseFilter.OBB):                        <L 328>
    }
    // return True                                                                            <L 332>
    return var_50;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:350
static CUDA_CALLABLE void _add_geom_pair_0(
    wp::int32 var_naconmax_in,
    wp::int32 var_geom1,
    wp::int32 var_geom2,
    wp::int32 var_type1,
    wp::int32 var_type2,
    wp::int32 var_worldid,
    wp::vec_t<2, wp::int32> var_pairid,
    wp::array_t<wp::int32> var_ncollision_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_out,
    wp::array_t<wp::int32> var_collision_worldid_out)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    const wp::int32 var_1 = 1;
    wp::int32 var_2;
    bool var_3;
    bool var_4;
    wp::vec_t<2, wp::int32> var_5;
    wp::vec_t<2, wp::int32> var_6;
    wp::vec_t<2, wp::int32> var_7;
    //---------
    // forward
    // def _add_geom_pair(                                                                    <L 351>
    // cid = wp.atomic_add(ncollision_out, 0, 1)                                              <L 368>
    var_2 = wp::atomic_add(var_ncollision_out, var_0, var_1);
    // if cid >= naconmax_in:                                                                 <L 370>
    var_3 = (var_2 >= var_naconmax_in);
    if (var_3) {
        // return                                                                             <L 371>
        return;
    }
    // if type1 > type2:                                                                      <L 373>
    var_4 = (var_type1 > var_type2);
    if (var_4) {
        // pair = wp.vec2i(geom2, geom1)                                                      <L 374>
        var_5 = wp::vec_t<2, wp::int32>(var_geom2, var_geom1);
    }
    if (!var_4) {
        // pair = wp.vec2i(geom1, geom2)                                                      <L 376>
        var_6 = wp::vec_t<2, wp::int32>(var_geom1, var_geom2);
    }
    var_7 = wp::where(var_4, var_5, var_6);
    // collision_pair_out[cid] = pair                                                         <L 378>
    wp::array_store(var_collision_pair_out, var_2, var_7);
    // collision_pairid_out[cid] = pairid                                                     <L 379>
    wp::array_store(var_collision_pairid_out, var_2, var_pairid);
    // collision_worldid_out[cid] = worldid                                                   <L 380>
    wp::array_store(var_collision_worldid_out, var_2, var_worldid);
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:114
static CUDA_CALLABLE void adj__sphere_filter_0(
    wp::float32 var_size1,
    wp::float32 var_size2,
    wp::float32 var_margin1,
    wp::float32 var_margin2,
    wp::vec_t<3, wp::float32> var_xpos1,
    wp::vec_t<3, wp::float32> var_xpos2,
    wp::float32 & adj_size1,
    wp::float32 & adj_size2,
    wp::float32 & adj_margin1,
    wp::float32 & adj_margin2,
    wp::vec_t<3, wp::float32> & adj_xpos1,
    wp::vec_t<3, wp::float32> & adj_xpos2,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:283
static CUDA_CALLABLE void adj__broadphase_filter__locals__func_2(
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_aabb,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_geom1,
    wp::int32 var_geom2,
    wp::int32 var_worldid,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_aabb,
    wp::array_t<wp::float32> & adj_geom_rbound,
    wp::array_t<wp::float32> & adj_geom_margin,
    wp::array_t<wp::float32> & adj_geom_gap,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_geom_xmat_in,
    wp::int32 & adj_geom1,
    wp::int32 & adj_geom2,
    wp::int32 & adj_worldid,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_driver.py:350
static CUDA_CALLABLE void adj__add_geom_pair_0(
    wp::int32 var_naconmax_in,
    wp::int32 var_geom1,
    wp::int32 var_geom2,
    wp::int32 var_type1,
    wp::int32 var_type2,
    wp::int32 var_worldid,
    wp::vec_t<2, wp::int32> var_pairid,
    wp::array_t<wp::int32> var_ncollision_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_out,
    wp::array_t<wp::int32> var_collision_worldid_out,
    wp::int32 & adj_naconmax_in,
    wp::int32 & adj_geom1,
    wp::int32 & adj_geom2,
    wp::int32 & adj_type1,
    wp::int32 & adj_type2,
    wp::int32 & adj_worldid,
    wp::vec_t<2, wp::int32> & adj_pairid,
    wp::array_t<wp::int32> & adj_ncollision_out,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_collision_pair_out,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_collision_pairid_out,
    wp::array_t<wp::int32> & adj_collision_worldid_out)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _nxn_broadphase__locals__kernel_40aca73b_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_aabb,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::vec_t<2, wp::int32>> var_nxn_geom_pair,
    wp::array_t<wp::vec_t<2, wp::int32>> var_nxn_pairid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::array_t<wp::int32> var_body_awake_in,
    wp::int32 var_naconmax_in,
    wp::array_t<wp::int32> var_body_awake_prev_in,
    wp::array_t<wp::int32> var_ncollision_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_out,
    wp::array_t<wp::int32> var_collision_worldid_out)
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
        wp::vec_t<2, wp::int32>* var_2;
        wp::vec_t<2, wp::int32> var_3;
        wp::vec_t<2, wp::int32> var_4;
        const wp::int32 var_5 = 0;
        wp::int32 var_6;
        const wp::int32 var_7 = 1;
        wp::int32 var_8;
        wp::int32* var_9;
        wp::int32 var_10;
        wp::int32 var_11;
        wp::int32* var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        const bool var_15 = false;
        const bool var_16 = false;
        wp::vec_t<2, wp::int32>* var_17;
        wp::vec_t<2, wp::int32> var_18;
        wp::vec_t<2, wp::int32> var_19;
        bool var_20;
        bool var_21;
        const wp::int32 var_22 = 1;
        wp::int32 var_23;
        const wp::int32 var_24 = 0;
        bool var_25;
        //---------
        // forward
        // def kernel(                                                                            <L 726>
        // worldid, elementid = wp.tid()                                                          <L 751>
        builtin_tid2d(var_0, var_1);
        // geom = nxn_geom_pair[elementid]                                                        <L 753>
        var_2 = wp::address(var_nxn_geom_pair, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // geom1 = geom[0]                                                                        <L 754>
        var_6 = wp::extract(var_3, var_5);
        // geom2 = geom[1]                                                                        <L 755>
        var_8 = wp::extract(var_3, var_7);
        // type1 = geom_type[geom1]                                                               <L 756>
        var_9 = wp::address(var_geom_type, var_6);
        var_11 = wp::load(var_9);
        var_10 = wp::copy(var_11);
        // type2 = geom_type[geom2]                                                               <L 757>
        var_12 = wp::address(var_geom_type, var_8);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // if wp.static(ngeom_dataid > 1):                                                        <L 759>
        // if wp.static(enable_sleep):                                                            <L 764>
        // pairid = nxn_pairid[elementid]                                                         <L 789>
        var_17 = wp::address(var_nxn_pairid, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // if (                                                                                   <L 790>
        // wp.static(_broadphase_filter(opt_broadphase_filter, ngeom_aabb, ngeom_rbound, ngeom_margin, ngeom_gap))(       <L 791>
        // geom_aabb, geom_rbound, geom_margin, geom_gap, geom_xpos_in, geom_xmat_in, geom1, geom2, worldid       <L 792>
        var_21 = _broadphase_filter__locals__func_2(var_geom_aabb, var_geom_rbound, var_geom_margin, var_geom_gap, var_geom_xpos_in, var_geom_xmat_in, var_6, var_8, var_0);
        var_20 = var_21;
        if (!var_20) {
            // or pairid[1] >= 0                                                                  <L 794>
            var_23 = wp::extract(var_18, var_22);
            var_25 = (var_23 >= var_24);
            var_20 = var_20 || var_25;
        }
        if (var_20) {
            // _add_geom_pair(                                                                    <L 796>
            // naconmax_in,                                                                       <L 797>
            // geom1,                                                                             <L 798>
            // geom2,                                                                             <L 799>
            // type1,                                                                             <L 800>
            // type2,                                                                             <L 801>
            // worldid,                                                                           <L 802>
            // pairid,                                                                            <L 803>
            // ncollision_out,                                                                    <L 804>
            // collision_pair_out,                                                                <L 805>
            // collision_pairid_out,                                                              <L 806>
            // collision_worldid_out,                                                             <L 807>
            _add_geom_pair_0(var_naconmax_in, var_6, var_8, var_10, var_13, var_0, var_18, var_ncollision_out, var_collision_pair_out, var_collision_pairid_out, var_collision_worldid_out);
        }
}

