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



extern "C" __global__ void _populate_group_starts__locals__kernel_cdf6f439_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_flex_group_temp_in,
    wp::array_t<wp::int32> var_flex_group_ids_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::int32> var_filter_val_in,
    wp::array_t<wp::int32> var_cand_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::array_t<wp::int32> var_flex_group_start_indices_out,
    wp::array_t<wp::int32> var_flex_num_groups_out)
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
        const wp::int32 var_11 = 1;
        bool var_12;
        wp::int32 var_13;
        wp::int32* var_14;
        const wp::int32 var_15 = 1;
        wp::int32 var_16;
        wp::int32 var_17;
        wp::shape_t* var_18;
        const wp::int32 var_19 = 0;
        wp::int32 var_20;
        wp::shape_t var_21;
        bool var_22;
        const bool var_23 = true;
        const wp::str var_24 = "flex candidate group overflow - please increase naconmax beyond %u\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.NARROWPHASE (or = 0 for all)\n";
        wp::shape_t* var_25;
        const wp::int32 var_26 = 0;
        wp::int32 var_27;
        wp::shape_t var_28;
        wp::int32* var_29;
        wp::int32* var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        const wp::int32 var_34 = 8;
        wp::int32 var_35;
        const wp::int32 var_36 = 1;
        wp::int32 var_37;
        bool var_38;
        wp::int32* var_39;
        const wp::int32 var_40 = 0;
        wp::int32 var_41;
        //---------
        // forward
        // def kernel(                                                                            <L 2615>
        // si = wp.tid()                                                                          <L 2628>
        var_0 = builtin_tid1d();
        // ncand_limit = wp.min(ncand[0], flex_group_temp_in.shape[0])                            <L 2629>
        var_2 = wp::address(var_ncand, var_1);
        var_3 = &(var_flex_group_temp_in.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_8 = wp::load(var_2);
        var_7 = wp::min(var_8, var_5);
        // if si >= ncand_limit:                                                                  <L 2630>
        var_9 = (var_0 >= var_7);
        if (var_9) {
            // return                                                                             <L 2631>
            continue;
        }
        // if flex_group_temp_in[si] == 1:                                                        <L 2633>
        var_10 = wp::address(var_flex_group_temp_in, var_0);
        var_13 = wp::load(var_10);
        var_12 = (var_13 == var_11);
        if (var_12) {
            // g = flex_group_ids_in[si] - 1                                                      <L 2634>
            var_14 = wp::address(var_flex_group_ids_in, var_0);
            var_17 = wp::load(var_14);
            var_16 = wp::sub(var_17, var_15);
            // if g >= flex_group_start_indices_out.shape[0]:                                     <L 2635>
            var_18 = &(var_flex_group_start_indices_out.shape);
            var_21 = wp::load(var_18);
            var_20 = wp::extract(var_21, var_19);
            var_22 = (var_16 >= var_20);
            if (var_22) {
                // if wp.static(bool(warn_overflow & OverflowType.NARROWPHASE)):                  <L 2636>
                // wp.printf(                                                                     <L 2637>
                // "flex candidate group overflow - please increase naconmax beyond %u\n"         <L 2638>
                // flex_group_start_indices_out.shape[0],                                         <L 2640>
                var_25 = &(var_flex_group_start_indices_out.shape);
                var_28 = wp::load(var_25);
                var_27 = wp::extract(var_28, var_26);
                printf(var_24, var_27);
                // worldid = cand_worldid[filter_val_in[si]]                                      <L 2642>
                var_29 = wp::address(var_filter_val_in, var_0);
                var_31 = wp::load(var_29);
                var_30 = wp::address(var_cand_worldid, var_31);
                var_33 = wp::load(var_30);
                var_32 = wp::copy(var_33);
                // wp.atomic_or(overflow_out, worldid, wp.static(OverflowType.NARROWPHASE))       <L 2643>
                var_35 = wp::atomic_or(var_overflow_out, var_32, var_34);
            }
            if (!var_22) {
                // flex_group_start_indices_out[g] = si                                           <L 2645>
                wp::array_store(var_flex_group_start_indices_out, var_16, var_0);
            }
        }
        // if si == ncand_limit - 1:                                                              <L 2647>
        var_37 = wp::sub(var_7, var_36);
        var_38 = (var_0 == var_37);
        if (var_38) {
            // flex_num_groups_out[0] = flex_group_ids_in[si]                                     <L 2648>
            var_39 = wp::address(var_flex_group_ids_in, var_0);
            var_41 = wp::load(var_39);
            wp::array_store(var_flex_num_groups_out, var_40, var_41);
        }
    }
}

