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



extern "C" __global__ void _sap_project__locals__sap_project_4dbbd22c_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_ngeom,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::int32 var_nworld_in,
    wp::vec_t<3, wp::float32> var_direction_in,
    wp::array_t<wp::float32> var_projection_lower_out,
    wp::array_t<wp::float32> var_projection_upper_out,
    wp::array_t<wp::int32> var_sort_index_out,
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
        const bool var_2 = false;
        const bool var_3 = false;
        wp::vec_t<3, wp::float32>* var_4;
        wp::vec_t<3, wp::float32> var_5;
        wp::vec_t<3, wp::float32> var_6;
        wp::shape_t* var_7;
        const wp::int32 var_8 = 0;
        wp::int32 var_9;
        wp::shape_t var_10;
        wp::int32 var_11;
        wp::float32* var_12;
        wp::float32 var_13;
        wp::float32 var_14;
        const wp::float32 var_15 = 0.0;
        bool var_16;
        const wp::float32 var_17 = 10000000000.0;
        wp::float32 var_18;
        wp::float32 var_19;
        wp::shape_t* var_20;
        const wp::int32 var_21 = 0;
        wp::int32 var_22;
        wp::shape_t var_23;
        wp::int32 var_24;
        wp::float32* var_25;
        wp::float32 var_26;
        wp::float32 var_27;
        wp::shape_t* var_28;
        const wp::int32 var_29 = 0;
        wp::int32 var_30;
        wp::shape_t var_31;
        wp::int32 var_32;
        wp::float32* var_33;
        wp::float32 var_34;
        wp::float32 var_35;
        wp::float32 var_36;
        bool var_37;
        bool var_38;
        wp::float32 var_39;
        wp::float32 var_40;
        //---------
        // forward
        // def sap_project(                                                                       <L 386>
        // worldid, geomid = wp.tid()                                                             <L 405>
        builtin_tid2d(var_0, var_1);
        // sort_index_out[worldid, geomid] = geomid                                               <L 407>
        wp::array_store(var_sort_index_out, var_0, var_1, var_1);
        // if wp.static(opt_broadphase == BroadphaseType.SAP_SEGMENTED):                          <L 408>
        // if wp.static(ngeom_dataid > 1):                                                        <L 414>
        // xpos = geom_xpos_in[worldid, geomid]                                                   <L 421>
        var_4 = wp::address(var_geom_xpos_in, var_0, var_1);
        var_6 = wp::load(var_4);
        var_5 = wp::copy(var_6);
        // rbound = geom_rbound[worldid % geom_rbound.shape[0], geomid]                           <L 422>
        var_7 = &(var_geom_rbound.shape);
        var_10 = wp::load(var_7);
        var_9 = wp::extract(var_10, var_8);
        var_11 = wp::mod(var_0, var_9);
        var_12 = wp::address(var_geom_rbound, var_11, var_1);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // if rbound == 0.0:                                                                      <L 424>
        var_16 = (var_13 == var_15);
        if (var_16) {
            // rbound = MJ_MAXVAL                                                                 <L 426>
            var_18 = wp::copy(var_17);
        }
        var_19 = wp::where(var_16, var_18, var_13);
        // radius = rbound + geom_margin[worldid % geom_margin.shape[0], geomid] + geom_gap[worldid % geom_gap.shape[0], geomid]       <L 428>
        var_20 = &(var_geom_margin.shape);
        var_23 = wp::load(var_20);
        var_22 = wp::extract(var_23, var_21);
        var_24 = wp::mod(var_0, var_22);
        var_25 = wp::address(var_geom_margin, var_24, var_1);
        var_27 = wp::load(var_25);
        var_26 = wp::add(var_19, var_27);
        var_28 = &(var_geom_gap.shape);
        var_31 = wp::load(var_28);
        var_30 = wp::extract(var_31, var_29);
        var_32 = wp::mod(var_0, var_30);
        var_33 = wp::address(var_geom_gap, var_32, var_1);
        var_35 = wp::load(var_33);
        var_34 = wp::add(var_26, var_35);
        // center = wp.dot(direction_in, xpos)                                                    <L 429>
        var_36 = wp::dot(var_direction_in, var_5);
        // if not wp.isnan(center):                                                               <L 431>
        var_37 = wp::isnan(var_36);
        var_38 = wp::unot(var_37);
        if (var_38) {
            // projection_lower_out[worldid, geomid] = center - radius                            <L 432>
            var_39 = wp::sub(var_36, var_34);
            wp::array_store(var_projection_lower_out, var_0, var_1, var_39);
            // projection_upper_out[worldid, geomid] = center + radius                            <L 433>
            var_40 = wp::add(var_36, var_34);
            wp::array_store(var_projection_upper_out, var_0, var_1, var_40);
        }
        if (!var_38) {
            // projection_lower_out[worldid, geomid] = MJ_MAXVAL                                  <L 435>
            wp::array_store(var_projection_lower_out, var_0, var_1, var_17);
            // projection_upper_out[worldid, geomid] = MJ_MAXVAL                                  <L 436>
            wp::array_store(var_projection_upper_out, var_0, var_1, var_17);
        }
}

