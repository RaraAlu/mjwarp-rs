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



extern "C" __global__ void _sap_project__locals__sap_project_632c2c31_cuda_kernel_forward(
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
        const bool var_2 = true;
        const wp::int32 var_3 = 0;
        bool var_4;
        wp::int32 var_5;
        const wp::int32 var_6 = 1;
        wp::int32 var_7;
        bool var_8;
        wp::int32 var_9;
        const bool var_10 = false;
        wp::vec_t<3, wp::float32>* var_11;
        wp::vec_t<3, wp::float32> var_12;
        wp::vec_t<3, wp::float32> var_13;
        wp::shape_t* var_14;
        const wp::int32 var_15 = 0;
        wp::int32 var_16;
        wp::shape_t var_17;
        wp::int32 var_18;
        wp::float32* var_19;
        wp::float32 var_20;
        wp::float32 var_21;
        const wp::float32 var_22 = 0.0;
        bool var_23;
        const wp::float32 var_24 = 10000000000.0;
        wp::float32 var_25;
        wp::float32 var_26;
        wp::shape_t* var_27;
        const wp::int32 var_28 = 0;
        wp::int32 var_29;
        wp::shape_t var_30;
        wp::int32 var_31;
        wp::float32* var_32;
        wp::float32 var_33;
        wp::float32 var_34;
        wp::shape_t* var_35;
        const wp::int32 var_36 = 0;
        wp::int32 var_37;
        wp::shape_t var_38;
        wp::int32 var_39;
        wp::float32* var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::float32 var_43;
        bool var_44;
        bool var_45;
        wp::float32 var_46;
        wp::float32 var_47;
        //---------
        // forward
        // def sap_project(                                                                       <L 386>
        // worldid, geomid = wp.tid()                                                             <L 405>
        builtin_tid2d(var_0, var_1);
        // sort_index_out[worldid, geomid] = geomid                                               <L 407>
        wp::array_store(var_sort_index_out, var_0, var_1, var_1);
        // if wp.static(opt_broadphase == BroadphaseType.SAP_SEGMENTED):                          <L 408>
        // if geomid == 0:                                                                        <L 409>
        var_4 = (var_1 == var_3);
        if (var_4) {
            // segmented_index_out[worldid] = worldid * ngeom                                     <L 410>
            var_5 = wp::mul(var_0, var_ngeom);
            wp::array_store(var_segmented_index_out, var_0, var_5);
            // if worldid == nworld_in - 1:                                                       <L 411>
            var_7 = wp::sub(var_nworld_in, var_6);
            var_8 = (var_0 == var_7);
            if (var_8) {
                // segmented_index_out[nworld_in] = nworld_in * ngeom                             <L 412>
                var_9 = wp::mul(var_nworld_in, var_ngeom);
                wp::array_store(var_segmented_index_out, var_nworld_in, var_9);
            }
        }
        // if wp.static(ngeom_dataid > 1):                                                        <L 414>
        // xpos = geom_xpos_in[worldid, geomid]                                                   <L 421>
        var_11 = wp::address(var_geom_xpos_in, var_0, var_1);
        var_13 = wp::load(var_11);
        var_12 = wp::copy(var_13);
        // rbound = geom_rbound[worldid % geom_rbound.shape[0], geomid]                           <L 422>
        var_14 = &(var_geom_rbound.shape);
        var_17 = wp::load(var_14);
        var_16 = wp::extract(var_17, var_15);
        var_18 = wp::mod(var_0, var_16);
        var_19 = wp::address(var_geom_rbound, var_18, var_1);
        var_21 = wp::load(var_19);
        var_20 = wp::copy(var_21);
        // if rbound == 0.0:                                                                      <L 424>
        var_23 = (var_20 == var_22);
        if (var_23) {
            // rbound = MJ_MAXVAL                                                                 <L 426>
            var_25 = wp::copy(var_24);
        }
        var_26 = wp::where(var_23, var_25, var_20);
        // radius = rbound + geom_margin[worldid % geom_margin.shape[0], geomid] + geom_gap[worldid % geom_gap.shape[0], geomid]       <L 428>
        var_27 = &(var_geom_margin.shape);
        var_30 = wp::load(var_27);
        var_29 = wp::extract(var_30, var_28);
        var_31 = wp::mod(var_0, var_29);
        var_32 = wp::address(var_geom_margin, var_31, var_1);
        var_34 = wp::load(var_32);
        var_33 = wp::add(var_26, var_34);
        var_35 = &(var_geom_gap.shape);
        var_38 = wp::load(var_35);
        var_37 = wp::extract(var_38, var_36);
        var_39 = wp::mod(var_0, var_37);
        var_40 = wp::address(var_geom_gap, var_39, var_1);
        var_42 = wp::load(var_40);
        var_41 = wp::add(var_33, var_42);
        // center = wp.dot(direction_in, xpos)                                                    <L 429>
        var_43 = wp::dot(var_direction_in, var_12);
        // if not wp.isnan(center):                                                               <L 431>
        var_44 = wp::isnan(var_43);
        var_45 = wp::unot(var_44);
        if (var_45) {
            // projection_lower_out[worldid, geomid] = center - radius                            <L 432>
            var_46 = wp::sub(var_43, var_41);
            wp::array_store(var_projection_lower_out, var_0, var_1, var_46);
            // projection_upper_out[worldid, geomid] = center + radius                            <L 433>
            var_47 = wp::add(var_43, var_41);
            wp::array_store(var_projection_upper_out, var_0, var_1, var_47);
        }
        if (!var_45) {
            // projection_lower_out[worldid, geomid] = MJ_MAXVAL                                  <L 435>
            wp::array_store(var_projection_lower_out, var_0, var_1, var_24);
            // projection_upper_out[worldid, geomid] = MJ_MAXVAL                                  <L 436>
            wp::array_store(var_projection_upper_out, var_0, var_1, var_24);
        }
}

