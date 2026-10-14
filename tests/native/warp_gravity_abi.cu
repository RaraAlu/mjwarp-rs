#define WP_NO_BFLOAT16
#define WP_TILE_BLOCK_DIM 256
#define WP_NO_CRT
#include "builtin.h"

#define builtin_tid3d(x, y, z) wp::tid(x, y, z, _idx, dim)

#define RECORD_ARRAY(index, value)                                                      \
    out[18 + (index) * 6] = (unsigned long long)(value).data;                          \
    out[19 + (index) * 6] = (unsigned long long)(value).shape[0];                      \
    out[20 + (index) * 6] = (unsigned long long)(value).shape[1];                      \
    out[21 + (index) * 6] = (unsigned long long)(value).strides[0];                    \
    out[22 + (index) * 6] = (unsigned long long)(value).strides[1];                    \
    out[23 + (index) * 6] = (unsigned long long)(value).ndim

extern "C" __global__ void _gravity_force_abi_probe_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::vec_t<3, wp::float32>> var_opt_gravity,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::float32> var_body_gravcomp,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::float32> var_qfrc_gravcomp_out)
{
    if (blockIdx.x != 0 || threadIdx.x != 0)
        return;
    unsigned long long* out = (unsigned long long*)var_qfrc_gravcomp_out.data;
    out[0] = sizeof(wp::array_t<wp::float32>);
    out[1] = alignof(wp::array_t<wp::float32>);
    out[2] = (char*)&var_qfrc_gravcomp_out.data - (char*)&var_qfrc_gravcomp_out;
    out[3] = (char*)&var_qfrc_gravcomp_out.grad - (char*)&var_qfrc_gravcomp_out;
    out[4] = (char*)&var_qfrc_gravcomp_out.shape - (char*)&var_qfrc_gravcomp_out;
    out[5] = (char*)&var_qfrc_gravcomp_out.strides - (char*)&var_qfrc_gravcomp_out;
    out[6] = (char*)&var_qfrc_gravcomp_out.ndim - (char*)&var_qfrc_gravcomp_out;
    out[7] = (char*)&var_qfrc_gravcomp_out.flags - (char*)&var_qfrc_gravcomp_out;
    out[8] = sizeof(wp::launch_bounds_t<3>);
    out[9] = alignof(wp::launch_bounds_t<3>);
    out[10] = (char*)&dim.shape - (char*)&dim;
    out[11] = (char*)&dim.size - (char*)&dim;
    out[12] = (char*)&dim.coord_mult - (char*)&dim;
    out[13] = (unsigned long long)dim.shape[0];
    out[14] = (unsigned long long)dim.shape[1];
    out[15] = (unsigned long long)dim.shape[2];
    out[16] = (unsigned long long)dim.size;
    out[17] = (unsigned long long)dim.coord_mult;
    RECORD_ARRAY(0, var_opt_gravity);
    RECORD_ARRAY(1, var_body_parentid);
    RECORD_ARRAY(2, var_body_rootid);
    RECORD_ARRAY(3, var_body_mass);
    RECORD_ARRAY(4, var_body_gravcomp);
    RECORD_ARRAY(5, var_dof_bodyid);
    RECORD_ARRAY(6, var_body_isdofancestor);
    RECORD_ARRAY(7, var_xipos_in);
    RECORD_ARRAY(8, var_subtree_com_in);
    RECORD_ARRAY(9, var_cdof_in);
    RECORD_ARRAY(10, var_qfrc_gravcomp_out);
}

static CUDA_CALLABLE wp::vec_t<3, wp::float32> gravity_jac_dof(
    wp::array_t<wp::int32> body_rootid,
    wp::array_t<wp::int32> body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> subtree_com,
    wp::array_t<wp::vec_t<6, wp::float32>> cdof,
    wp::vec_t<3, wp::float32> point,
    wp::int32 bodyid,
    wp::int32 dofid,
    wp::int32 worldid)
{
    if (wp::load(wp::address(body_isdofancestor, bodyid, dofid)) == 0)
        return wp::vec_t<3, wp::float32>(0.0f);
    wp::int32 rootid = wp::load(wp::address(body_rootid, bodyid));
    wp::vec_t<3, wp::float32> offset =
        wp::sub(point, wp::load(wp::address(subtree_com, worldid, rootid)));
    wp::vec_t<6, wp::float32> dof = wp::load(wp::address(cdof, worldid, dofid));
    return wp::add(wp::spatial_bottom(dof), wp::cross(wp::spatial_top(dof), offset));
}

extern "C" __global__ void _gravity_force_bridge_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::vec_t<3, wp::float32>> opt_gravity,
    wp::array_t<wp::int32> body_parentid,
    wp::array_t<wp::int32> body_rootid,
    wp::array_t<wp::float32> body_mass,
    wp::array_t<wp::float32> body_gravcomp,
    wp::array_t<wp::int32> dof_bodyid,
    wp::array_t<wp::int32> body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> xipos,
    wp::array_t<wp::vec_t<3, wp::float32>> subtree_com,
    wp::array_t<wp::vec_t<6, wp::float32>> cdof,
    wp::array_t<wp::float32> qfrc_gravcomp)
{
    wp::tile_shared_storage_t tile_mem;
    for (size_t _idx = static_cast<size_t>(blockDim.x) * static_cast<size_t>(blockIdx.x) +
                       static_cast<size_t>(threadIdx.x);
         _idx < dim.size;
         _idx += static_cast<size_t>(blockDim.x) * static_cast<size_t>(gridDim.x)) {
        wp::tile_shared_storage_t::init();
        wp::int32 worldid;
        wp::int32 bodyid;
        wp::int32 dofid;
        builtin_tid3d(worldid, bodyid, dofid);
        bodyid = wp::add(bodyid, wp::int32(1));
        wp::float32 gravcomp = wp::load(
            wp::address(body_gravcomp, wp::mod(worldid, body_gravcomp.shape[0]), bodyid));
        wp::vec_t<3, wp::float32> gravity =
            wp::load(wp::address(opt_gravity, wp::mod(worldid, opt_gravity.shape[0])));
        if (gravcomp != 0.0f) {
            wp::float32 mass =
                wp::load(wp::address(body_mass, wp::mod(worldid, body_mass.shape[0]), bodyid));
            wp::vec_t<3, wp::float32> force = wp::mul(wp::mul(wp::neg(gravity), mass), gravcomp);
            wp::vec_t<3, wp::float32> point = wp::load(wp::address(xipos, worldid, bodyid));
            wp::vec_t<3, wp::float32> jac = gravity_jac_dof(
                body_rootid,
                body_isdofancestor,
                subtree_com,
                cdof,
                point,
                bodyid,
                dofid,
                worldid);
            wp::atomic_add(qfrc_gravcomp, worldid, dofid, wp::dot(jac, force));
        }
    }
}

#undef RECORD_ARRAY
#undef builtin_tid3d
