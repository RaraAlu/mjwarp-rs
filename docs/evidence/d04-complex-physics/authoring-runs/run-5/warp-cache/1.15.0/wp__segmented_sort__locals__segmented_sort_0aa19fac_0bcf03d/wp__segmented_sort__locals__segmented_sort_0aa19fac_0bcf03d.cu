#define WP_NO_BFLOAT16

#define WP_TILE_BLOCK_DIM 128
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



extern "C" __global__ void _segmented_sort__locals__segmented_sort_57bc410b_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::float32> var_projection_lower_in,
    wp::array_t<wp::int32> var_sort_index_in,
    wp::array_t<wp::float32> var_projection_lower_out,
    wp::array_t<wp::int32> var_sort_index_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::slice_t var_1;
        const wp::int32 var_2 = 0;
        wp::array_t<wp::float32> var_3;
        const wp::int32 var_4 = 2;
        const wp::str var_5 = "shared";
        wp::tile_shared_t<wp::float32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true> var_6 = wp::tile_alloc_empty<wp::float32,wp::tile_shape_t<2>,wp::tile_stride_t<1>,false>();
        const wp::int32 var_7 = 0;
        wp::slice_t var_8;
        const wp::int32 var_9 = 0;
        wp::array_t<wp::int32> var_10;
        const wp::str var_11 = "shared";
        wp::tile_shared_t<wp::int32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true> var_12 = wp::tile_alloc_empty<wp::int32,wp::tile_shape_t<2>,wp::tile_stride_t<1>,false>();
        const wp::int32 var_13 = 0;
        wp::slice_t var_14;
        const wp::int32 var_15 = 0;
        wp::array_t<wp::float32> var_16;
        const wp::int32 var_17 = 0;
        wp::slice_t var_18;
        const wp::int32 var_19 = 0;
        wp::array_t<wp::int32> var_20;
        const wp::int32 var_21 = 0;
        //---------
        // forward
        // def segmented_sort(                                                                    <L 570>
        // worldid = wp.tid()                                                                     <L 578>
        var_0 = builtin_tid1d();
        // keys = wp.tile_load(projection_lower_in[worldid], shape=tile_size, storage="shared")       <L 581>
        var_1 = wp::slice_t(var_0, var_0, var_2);
        var_3 = wp::view(var_projection_lower_in, var_1);
        var_6 = wp::tile_load<wp::float32, true, false, 2>(var_3, var_7);
        // values = wp.tile_load(sort_index_in[worldid], shape=tile_size, storage="shared")       <L 582>
        var_8 = wp::slice_t(var_0, var_0, var_9);
        var_10 = wp::view(var_sort_index_in, var_8);
        var_12 = wp::tile_load<wp::int32, true, false, 2>(var_10, var_13);
        // wp.tile_sort(keys, values)                                                             <L 585>
        wp::tile_sort(var_6, var_12);
        // wp.tile_store(projection_lower_out[worldid], keys)                                     <L 588>
        var_14 = wp::slice_t(var_0, var_0, var_15);
        var_16 = wp::view(var_projection_lower_out, var_14);
        wp::tile_store<wp::float32, true, false>(var_16, var_17, var_6);
        // wp.tile_store(sort_index_out[worldid], values)                                         <L 589>
        var_18 = wp::slice_t(var_0, var_0, var_19);
        var_20 = wp::view(var_sort_index_out, var_18);
        wp::tile_store<wp::int32, true, false>(var_20, var_21, var_12);
}



extern "C" __global__ void _segmented_sort__locals__segmented_sort_57bc410b_cuda_kernel_backward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::float32> var_projection_lower_in,
    wp::array_t<wp::int32> var_sort_index_in,
    wp::array_t<wp::float32> var_projection_lower_out,
    wp::array_t<wp::int32> var_sort_index_out,
    wp::array_t<wp::float32> adj_projection_lower_in,
    wp::array_t<wp::int32> adj_sort_index_in,
    wp::array_t<wp::float32> adj_projection_lower_out,
    wp::array_t<wp::int32> adj_sort_index_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::slice_t var_1;
        const wp::int32 var_2 = 0;
        wp::array_t<wp::float32> var_3;
        const wp::int32 var_4 = 2;
        const wp::str var_5 = "shared";
        wp::tile_shared_t<wp::float32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true> var_6 = wp::tile_alloc_empty<wp::float32,wp::tile_shape_t<2>,wp::tile_stride_t<1>,true>();
        const wp::int32 var_7 = 0;
        wp::slice_t var_8;
        const wp::int32 var_9 = 0;
        wp::array_t<wp::int32> var_10;
        const wp::str var_11 = "shared";
        wp::tile_shared_t<wp::int32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true> var_12 = wp::tile_alloc_empty<wp::int32,wp::tile_shape_t<2>,wp::tile_stride_t<1>,true>();
        const wp::int32 var_13 = 0;
        wp::slice_t var_14;
        const wp::int32 var_15 = 0;
        wp::array_t<wp::float32> var_16;
        const wp::int32 var_17 = 0;
        wp::slice_t var_18;
        const wp::int32 var_19 = 0;
        wp::array_t<wp::int32> var_20;
        const wp::int32 var_21 = 0;
        //---------
        // dual vars
        wp::int32 adj_0 = {};
        wp::slice_t adj_1 = {};
        wp::int32 adj_2 = {};
        wp::array_t<wp::float32> adj_3 = {};
        wp::int32 adj_4 = {};
        wp::str adj_5 = {};
        wp::tile_shared_t<wp::float32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true>& adj_6 = var_6;
        wp::int32 adj_7 = {};
        wp::slice_t adj_8 = {};
        wp::int32 adj_9 = {};
        wp::array_t<wp::int32> adj_10 = {};
        wp::str adj_11 = {};
        wp::tile_shared_t<wp::int32,wp::tile_layout_strided_t<wp::tile_shape_t<2>, wp::tile_stride_t<1>>, true>& adj_12 = var_12;
        wp::int32 adj_13 = {};
        wp::slice_t adj_14 = {};
        wp::int32 adj_15 = {};
        wp::array_t<wp::float32> adj_16 = {};
        wp::int32 adj_17 = {};
        wp::slice_t adj_18 = {};
        wp::int32 adj_19 = {};
        wp::array_t<wp::int32> adj_20 = {};
        wp::int32 adj_21 = {};
        //---------
        // forward
        // def segmented_sort(                                                                    <L 570>
        // worldid = wp.tid()                                                                     <L 578>
        var_0 = builtin_tid1d();
        // keys = wp.tile_load(projection_lower_in[worldid], shape=tile_size, storage="shared")       <L 581>
        var_1 = wp::slice_t(var_0, var_0, var_2);
        var_3 = wp::view(var_projection_lower_in, var_1);
        var_6 = wp::tile_load<wp::float32, true, false, 2>(var_3, var_7);
        // values = wp.tile_load(sort_index_in[worldid], shape=tile_size, storage="shared")       <L 582>
        var_8 = wp::slice_t(var_0, var_0, var_9);
        var_10 = wp::view(var_sort_index_in, var_8);
        var_12 = wp::tile_load<wp::int32, true, false, 2>(var_10, var_13);
        // wp.tile_sort(keys, values)                                                             <L 585>
        wp::tile_sort(var_6, var_12);
        // wp.tile_store(projection_lower_out[worldid], keys)                                     <L 588>
        var_14 = wp::slice_t(var_0, var_0, var_15);
        var_16 = wp::view(var_projection_lower_out, var_14);
        // wp::tile_store<wp::float32, true, false>(var_16, var_17, var_6);
        // wp.tile_store(sort_index_out[worldid], values)                                         <L 589>
        var_18 = wp::slice_t(var_0, var_0, var_19);
        var_20 = wp::view(var_sort_index_out, var_18);
        // wp::tile_store<wp::int32, true, false>(var_20, var_21, var_12);
        //---------
        // reverse
        wp::adj_tile_store(var_20, var_21, var_12, adj_20, adj_21, adj_12);
        wp::adj_view(var_sort_index_out, var_18, adj_sort_index_out, adj_18, adj_20);
        // adj: wp.tile_store(sort_index_out[worldid], values)                                    <L 589>
        wp::adj_tile_store(var_16, var_17, var_6, adj_16, adj_17, adj_6);
        wp::adj_view(var_projection_lower_out, var_14, adj_projection_lower_out, adj_14, adj_16);
        // adj: wp.tile_store(projection_lower_out[worldid], keys)                                <L 588>
        // adj: wp.tile_sort(keys, values)                                                        <L 585>
        wp::adj_tile_load(var_10, var_13, adj_10, adj_13, adj_12);
        wp::adj_view(var_sort_index_in, var_8, adj_sort_index_in, adj_8, adj_10);
        // adj: values = wp.tile_load(sort_index_in[worldid], shape=tile_size, storage="shared")  <L 582>
        wp::adj_tile_load(var_3, var_7, adj_3, adj_7, adj_6);
        wp::adj_view(var_projection_lower_in, var_1, adj_projection_lower_in, adj_1, adj_3);
        // adj: keys = wp.tile_load(projection_lower_in[worldid], shape=tile_size, storage="shared")  <L 581>
        // adj: worldid = wp.tid()                                                                <L 578>
        // adj: def segmented_sort(                                                               <L 570>
        return;
}

