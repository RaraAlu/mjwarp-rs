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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:133
static CUDA_CALLABLE wp::vec_t<6, wp::float32> motion_cross_0(
    wp::vec_t<6, wp::float32> var_u,
    wp::vec_t<6, wp::float32> var_v)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 2;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32> var_6;
    const wp::int32 var_7 = 3;
    wp::float32 var_8;
    const wp::int32 var_9 = 4;
    wp::float32 var_10;
    const wp::int32 var_11 = 5;
    wp::float32 var_12;
    wp::vec_t<3, wp::float32> var_13;
    const wp::int32 var_14 = 0;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::float32 var_17;
    const wp::int32 var_18 = 2;
    wp::float32 var_19;
    wp::vec_t<3, wp::float32> var_20;
    const wp::int32 var_21 = 3;
    wp::float32 var_22;
    const wp::int32 var_23 = 4;
    wp::float32 var_24;
    const wp::int32 var_25 = 5;
    wp::float32 var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<6, wp::float32> var_32;
    //---------
    // forward
    // def motion_cross(u: wp.spatial_vector, v: wp.spatial_vector) -> wp.spatial_vector:       <L 134>
    // u0 = wp.vec3(u[0], u[1], u[2])                                                         <L 136>
    var_1 = wp::extract(var_u, var_0);
    var_3 = wp::extract(var_u, var_2);
    var_5 = wp::extract(var_u, var_4);
    var_6 = wp::vec_t<3, wp::float32>(var_1, var_3, var_5);
    // u1 = wp.vec3(u[3], u[4], u[5])                                                         <L 137>
    var_8 = wp::extract(var_u, var_7);
    var_10 = wp::extract(var_u, var_9);
    var_12 = wp::extract(var_u, var_11);
    var_13 = wp::vec_t<3, wp::float32>(var_8, var_10, var_12);
    // v0 = wp.vec3(v[0], v[1], v[2])                                                         <L 138>
    var_15 = wp::extract(var_v, var_14);
    var_17 = wp::extract(var_v, var_16);
    var_19 = wp::extract(var_v, var_18);
    var_20 = wp::vec_t<3, wp::float32>(var_15, var_17, var_19);
    // v1 = wp.vec3(v[3], v[4], v[5])                                                         <L 139>
    var_22 = wp::extract(var_v, var_21);
    var_24 = wp::extract(var_v, var_23);
    var_26 = wp::extract(var_v, var_25);
    var_27 = wp::vec_t<3, wp::float32>(var_22, var_24, var_26);
    // ang = wp.cross(u0, v0)                                                                 <L 141>
    var_28 = wp::cross(var_6, var_20);
    // vel = wp.cross(u1, v0) + wp.cross(u0, v1)                                              <L 142>
    var_29 = wp::cross(var_13, var_20);
    var_30 = wp::cross(var_6, var_27);
    var_31 = wp::add(var_29, var_30);
    // return wp.spatial_vector(ang, vel)                                                     <L 144>
    var_32 = wp::vec_t<6, wp::float32>(var_28, var_31);
    return var_32;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:504
static CUDA_CALLABLE void jac_dof_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::vec_t<3, wp::float32> var_point,
    wp::int32 var_bodyid,
    wp::int32 var_dofid,
    wp::int32 var_worldid,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::int32* var_0;
    const wp::int32 var_1 = 0;
    bool var_2;
    wp::int32 var_3;
    const wp::float32 var_4 = 0.0;
    wp::vec_t<3, wp::float32> var_5;
    const wp::float32 var_6 = 0.0;
    wp::vec_t<3, wp::float32> var_7;
    wp::int32* var_8;
    wp::vec_t<3, wp::float32>* var_9;
    wp::int32 var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<6, wp::float32>* var_14;
    wp::vec_t<6, wp::float32> var_15;
    wp::vec_t<6, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    //---------
    // forward
    // def jac_dof(                                                                           <L 505>
    // if body_isdofancestor[bodyid, dofid] == 0:                                             <L 520>
    var_0 = wp::address(var_body_isdofancestor, var_bodyid, var_dofid);
    var_3 = wp::load(var_0);
    var_2 = (var_3 == var_1);
    if (var_2) {
        // return wp.vec3(0.0), wp.vec3(0.0)                                                  <L 521>
        var_5 = wp::vec_t<3, wp::float32>(var_4);
        var_7 = wp::vec_t<3, wp::float32>(var_6);
        ret_0 = var_5;
        ret_1 = var_7;
        return;
    }
    // offset = point - wp.vec3(subtree_com_in[worldid, body_rootid[bodyid]])                 <L 523>
    var_8 = wp::address(var_body_rootid, var_bodyid);
    var_10 = wp::load(var_8);
    var_9 = wp::address(var_subtree_com_in, var_worldid, var_10);
    var_12 = wp::load(var_9);
    var_11 = wp::vec_t<3, wp::float32>(var_12);
    var_13 = wp::sub(var_point, var_11);
    // cdof = cdof_in[worldid, dofid]                                                         <L 525>
    var_14 = wp::address(var_cdof_in, var_worldid, var_dofid);
    var_16 = wp::load(var_14);
    var_15 = wp::copy(var_16);
    // cdof_ang = wp.spatial_top(cdof)                                                        <L 526>
    var_17 = wp::spatial_top(var_15);
    // cdof_lin = wp.spatial_bottom(cdof)                                                     <L 527>
    var_18 = wp::spatial_bottom(var_15);
    // jacp = cdof_lin + wp.cross(cdof_ang, offset)                                           <L 529>
    var_19 = wp::cross(var_17, var_13);
    var_20 = wp::add(var_18, var_19);
    // jacr = cdof_ang                                                                        <L 530>
    var_21 = wp::copy(var_17);
    // return jacp, jacr                                                                      <L 532>
    ret_0 = var_20;
    ret_1 = var_21;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:115
static CUDA_CALLABLE wp::quat_t<wp::float32> quat_inv_0(
    wp::quat_t<wp::float32> var_quat)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 2;
    wp::float32 var_6;
    wp::float32 var_7;
    const wp::int32 var_8 = 3;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::quat_t<wp::float32> var_11;
    //---------
    // forward
    // def quat_inv(quat: wp.quat) -> wp.quat:                                                <L 116>
    // return wp.quat(quat[0], -quat[1], -quat[2], -quat[3])                                  <L 117>
    var_1 = wp::extract(var_quat, var_0);
    var_3 = wp::extract(var_quat, var_2);
    var_4 = wp::neg(var_3);
    var_6 = wp::extract(var_quat, var_5);
    var_7 = wp::neg(var_6);
    var_9 = wp::extract(var_quat, var_8);
    var_10 = wp::neg(var_9);
    var_11 = wp::quat_t<wp::float32>(var_1, var_4, var_7, var_10);
    return var_11;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:44
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\warp\_src\math.py:0
static CUDA_CALLABLE wp::float32 norm_l2_0(
    wp::vec_t<3, wp::float32> var_v)
{
    //---------
    // primal vars
    wp::float32 var_0;
    //---------
    // forward
    // def norm_l2(v: Any) -> float:                                                          <L 1>
    // return wp.length(v)                                                                    <L 12>
    var_0 = wp::length(var_v);
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:161
static CUDA_CALLABLE wp::vec_t<3, wp::float32> quat_to_vel_0(
    wp::quat_t<wp::float32> var_quat)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::float32 var_1;
    const wp::int32 var_2 = 2;
    wp::float32 var_3;
    const wp::int32 var_4 = 3;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32> var_6;
    wp::float32 var_7;
    const wp::float32 var_8 = 0.0;
    bool var_9;
    const wp::float32 var_10 = 0.0;
    wp::vec_t<3, wp::float32> var_11;
    const wp::float32 var_12 = 2.0;
    const wp::int32 var_13 = 0;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::float32 var_17 = 3.141592653589793;
    bool var_18;
    const wp::float32 var_19 = 2.0;
    const wp::float32 var_20 = 3.141592653589793;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    //---------
    // forward
    // def quat_to_vel(quat: wp.quat) -> wp.vec3:                                             <L 162>
    // axis = wp.vec3(quat[1], quat[2], quat[3])                                              <L 163>
    var_1 = wp::extract(var_quat, var_0);
    var_3 = wp::extract(var_quat, var_2);
    var_5 = wp::extract(var_quat, var_4);
    var_6 = wp::vec_t<3, wp::float32>(var_1, var_3, var_5);
    // sin_a_2 = wp.norm_l2(axis)                                                             <L 164>
    var_7 = norm_l2_0(var_6);
    // if sin_a_2 == 0.0:                                                                     <L 166>
    var_9 = (var_7 == var_8);
    if (var_9) {
        // return wp.vec3(0.0)                                                                <L 167>
        var_11 = wp::vec_t<3, wp::float32>(var_10);
        return var_11;
    }
    // speed = 2.0 * wp.atan2(sin_a_2, quat[0])                                               <L 169>
    var_14 = wp::extract(var_quat, var_13);
    var_15 = wp::atan2(var_7, var_14);
    var_16 = wp::mul(var_12, var_15);
    // if speed > wp.pi:                                                                      <L 171>
    var_18 = (var_16 > var_17);
    if (var_18) {
        // speed -= 2.0 * wp.pi                                                               <L 172>
        var_21 = wp::mul(var_19, var_20);
        var_22 = wp::sub(var_16, var_21);
    }
    var_23 = wp::where(var_18, var_22, var_16);
    // return axis * speed / sin_a_2                                                          <L 174>
    var_24 = wp::mul(var_6, var_23);
    var_25 = wp::div(var_24, var_7);
    return var_25;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE wp::float32 safe_div_0(
    wp::float32 var_x,
    wp::float32 var_y)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    bool var_1;
    const wp::float32 var_2 = 1e-15;
    const wp::float32 var_3 = 1e-15;
    wp::float32 var_4;
    wp::float32 var_5;
    //---------
    // forward
    // def safe_div(x: Any, y: Any) -> Any:                                                   <L 1>
    // return x / wp.where(y != 0.0, y, types.MJ_MINVAL)                                      <L 2>
    var_1 = (var_y != var_0);
    var_4 = wp::where(var_1, var_y, var_3);
    var_5 = wp::div(var_x, var_4);
    return var_5;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE wp::vec_t<3, wp::float32> safe_div_0(
    wp::vec_t<3, wp::float32> var_x,
    wp::float32 var_y)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    bool var_1;
    const wp::float32 var_2 = 1e-15;
    const wp::float32 var_3 = 1e-15;
    wp::float32 var_4;
    wp::vec_t<3, wp::float32> var_5;
    //---------
    // forward
    // def safe_div(x: Any, y: Any) -> Any:                                                   <L 1>
    // return x / wp.where(y != 0.0, y, types.MJ_MINVAL)                                      <L 2>
    var_1 = (var_y != var_0);
    var_4 = wp::where(var_1, var_y, var_3);
    var_5 = wp::div(var_x, var_4);
    return var_5;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:23
static CUDA_CALLABLE wp::quat_t<wp::float32> mul_quat_0(
    wp::quat_t<wp::float32> var_u,
    wp::quat_t<wp::float32> var_v)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 1;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 3;
    wp::float32 var_18;
    const wp::int32 var_19 = 3;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 0;
    wp::float32 var_24;
    const wp::int32 var_25 = 1;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 1;
    wp::float32 var_29;
    const wp::int32 var_30 = 0;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::int32 var_34 = 2;
    wp::float32 var_35;
    const wp::int32 var_36 = 3;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 3;
    wp::float32 var_41;
    const wp::int32 var_42 = 2;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::int32 var_46 = 0;
    wp::float32 var_47;
    const wp::int32 var_48 = 2;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::int32 var_51 = 1;
    wp::float32 var_52;
    const wp::int32 var_53 = 3;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    const wp::int32 var_57 = 2;
    wp::float32 var_58;
    const wp::int32 var_59 = 0;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 3;
    wp::float32 var_64;
    const wp::int32 var_65 = 1;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    const wp::int32 var_71 = 3;
    wp::float32 var_72;
    wp::float32 var_73;
    const wp::int32 var_74 = 1;
    wp::float32 var_75;
    const wp::int32 var_76 = 2;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    const wp::int32 var_80 = 2;
    wp::float32 var_81;
    const wp::int32 var_82 = 1;
    wp::float32 var_83;
    wp::float32 var_84;
    wp::float32 var_85;
    const wp::int32 var_86 = 3;
    wp::float32 var_87;
    const wp::int32 var_88 = 0;
    wp::float32 var_89;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::quat_t<wp::float32> var_92;
    //---------
    // forward
    // def mul_quat(u: wp.quat, v: wp.quat) -> wp.quat:                                       <L 24>
    // return wp.quat(                                                                        <L 25>
    // u[0] * v[0] - u[1] * v[1] - u[2] * v[2] - u[3] * v[3],                                 <L 26>
    var_1 = wp::extract(var_u, var_0);
    var_3 = wp::extract(var_v, var_2);
    var_4 = wp::mul(var_1, var_3);
    var_6 = wp::extract(var_u, var_5);
    var_8 = wp::extract(var_v, var_7);
    var_9 = wp::mul(var_6, var_8);
    var_10 = wp::sub(var_4, var_9);
    var_12 = wp::extract(var_u, var_11);
    var_14 = wp::extract(var_v, var_13);
    var_15 = wp::mul(var_12, var_14);
    var_16 = wp::sub(var_10, var_15);
    var_18 = wp::extract(var_u, var_17);
    var_20 = wp::extract(var_v, var_19);
    var_21 = wp::mul(var_18, var_20);
    var_22 = wp::sub(var_16, var_21);
    // u[0] * v[1] + u[1] * v[0] + u[2] * v[3] - u[3] * v[2],                                 <L 27>
    var_24 = wp::extract(var_u, var_23);
    var_26 = wp::extract(var_v, var_25);
    var_27 = wp::mul(var_24, var_26);
    var_29 = wp::extract(var_u, var_28);
    var_31 = wp::extract(var_v, var_30);
    var_32 = wp::mul(var_29, var_31);
    var_33 = wp::add(var_27, var_32);
    var_35 = wp::extract(var_u, var_34);
    var_37 = wp::extract(var_v, var_36);
    var_38 = wp::mul(var_35, var_37);
    var_39 = wp::add(var_33, var_38);
    var_41 = wp::extract(var_u, var_40);
    var_43 = wp::extract(var_v, var_42);
    var_44 = wp::mul(var_41, var_43);
    var_45 = wp::sub(var_39, var_44);
    // u[0] * v[2] - u[1] * v[3] + u[2] * v[0] + u[3] * v[1],                                 <L 28>
    var_47 = wp::extract(var_u, var_46);
    var_49 = wp::extract(var_v, var_48);
    var_50 = wp::mul(var_47, var_49);
    var_52 = wp::extract(var_u, var_51);
    var_54 = wp::extract(var_v, var_53);
    var_55 = wp::mul(var_52, var_54);
    var_56 = wp::sub(var_50, var_55);
    var_58 = wp::extract(var_u, var_57);
    var_60 = wp::extract(var_v, var_59);
    var_61 = wp::mul(var_58, var_60);
    var_62 = wp::add(var_56, var_61);
    var_64 = wp::extract(var_u, var_63);
    var_66 = wp::extract(var_v, var_65);
    var_67 = wp::mul(var_64, var_66);
    var_68 = wp::add(var_62, var_67);
    // u[0] * v[3] + u[1] * v[2] - u[2] * v[1] + u[3] * v[0],                                 <L 29>
    var_70 = wp::extract(var_u, var_69);
    var_72 = wp::extract(var_v, var_71);
    var_73 = wp::mul(var_70, var_72);
    var_75 = wp::extract(var_u, var_74);
    var_77 = wp::extract(var_v, var_76);
    var_78 = wp::mul(var_75, var_77);
    var_79 = wp::add(var_73, var_78);
    var_81 = wp::extract(var_u, var_80);
    var_83 = wp::extract(var_v, var_82);
    var_84 = wp::mul(var_81, var_83);
    var_85 = wp::sub(var_79, var_84);
    var_87 = wp::extract(var_u, var_86);
    var_89 = wp::extract(var_v, var_88);
    var_90 = wp::mul(var_87, var_89);
    var_91 = wp::add(var_85, var_90);
    var_92 = wp::quat_t<wp::float32>(var_22, var_45, var_68, var_91);
    return var_92;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:177
static CUDA_CALLABLE wp::vec_t<3, wp::float32> quat_sub_0(
    wp::quat_t<wp::float32> var_qa,
    wp::quat_t<wp::float32> var_qb)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 2;
    wp::float32 var_6;
    wp::float32 var_7;
    const wp::int32 var_8 = 3;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::quat_t<wp::float32> var_11;
    wp::quat_t<wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    //---------
    // forward
    // def quat_sub(qa: wp.quat, qb: wp.quat) -> wp.vec3:                                     <L 178>
    // qneg = wp.quat(qb[0], -qb[1], -qb[2], -qb[3])                                          <L 181>
    var_1 = wp::extract(var_qb, var_0);
    var_3 = wp::extract(var_qb, var_2);
    var_4 = wp::neg(var_3);
    var_6 = wp::extract(var_qb, var_5);
    var_7 = wp::neg(var_6);
    var_9 = wp::extract(var_qb, var_8);
    var_10 = wp::neg(var_9);
    var_11 = wp::quat_t<wp::float32>(var_1, var_4, var_7, var_10);
    // qdif = mul_quat(qneg, qa)                                                              <L 182>
    var_12 = mul_quat_0(var_11, var_qa);
    // return quat_to_vel(qdif)                                                               <L 185>
    var_13 = quat_to_vel_0(var_12);
    return var_13;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1072
static CUDA_CALLABLE void get_face_metadata_0(
    wp::int32 var_cellnum_x,
    wp::int32 var_cellnum_y,
    wp::int32 var_cellnum_z,
    wp::int32 var_face_elem_idx,
    wp::int32 var_order_abs,
    wp::int32 & ret_0,
    wp::int32 & ret_1,
    wp::int32 & ret_2,
    wp::int32 & ret_3,
    wp::int32 & ret_4,
    wp::int32 & ret_5)
{
    //---------
    // primal vars
    wp::int32 var_0;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 0;
    const wp::int32 var_4 = 0;
    bool var_5;
    const wp::int32 var_6 = 0;
    wp::int32 var_7;
    wp::int32 var_8;
    wp::int32 var_9;
    const wp::int32 var_10 = 2;
    wp::int32 var_11;
    bool var_12;
    const wp::int32 var_13 = 1;
    wp::int32 var_14;
    wp::int32 var_15;
    wp::int32 var_16;
    const wp::int32 var_17 = 2;
    wp::int32 var_18;
    wp::int32 var_19;
    bool var_20;
    const wp::int32 var_21 = 2;
    const wp::int32 var_22 = 2;
    wp::int32 var_23;
    wp::int32 var_24;
    wp::int32 var_25;
    wp::int32 var_26;
    const wp::int32 var_27 = 2;
    wp::int32 var_28;
    const wp::int32 var_29 = 2;
    wp::int32 var_30;
    wp::int32 var_31;
    bool var_32;
    const wp::int32 var_33 = 3;
    const wp::int32 var_34 = 2;
    wp::int32 var_35;
    wp::int32 var_36;
    wp::int32 var_37;
    wp::int32 var_38;
    wp::int32 var_39;
    const wp::int32 var_40 = 2;
    wp::int32 var_41;
    const wp::int32 var_42 = 2;
    wp::int32 var_43;
    wp::int32 var_44;
    wp::int32 var_45;
    bool var_46;
    const wp::int32 var_47 = 4;
    const wp::int32 var_48 = 2;
    wp::int32 var_49;
    wp::int32 var_50;
    const wp::int32 var_51 = 2;
    wp::int32 var_52;
    wp::int32 var_53;
    wp::int32 var_54;
    wp::int32 var_55;
    const wp::int32 var_56 = 5;
    const wp::int32 var_57 = 2;
    wp::int32 var_58;
    wp::int32 var_59;
    const wp::int32 var_60 = 2;
    wp::int32 var_61;
    wp::int32 var_62;
    wp::int32 var_63;
    wp::int32 var_64;
    wp::int32 var_65;
    wp::int32 var_66;
    wp::int32 var_67;
    wp::int32 var_68;
    wp::int32 var_69;
    wp::int32 var_70;
    wp::int32 var_71;
    wp::int32 var_72;
    wp::int32 var_73;
    const wp::int32 var_74 = 2;
    wp::int32 var_75;
    const wp::int32 var_76 = 0;
    bool var_77;
    const wp::int32 var_78 = 0;
    bool var_79;
    const wp::int32 var_80 = 1;
    bool var_81;
    wp::int32 var_82;
    wp::int32 var_83;
    bool var_84;
    const wp::int32 var_85 = 2;
    bool var_86;
    const wp::int32 var_87 = 3;
    bool var_88;
    wp::int32 var_89;
    wp::int32 var_90;
    wp::int32 var_91;
    wp::int32 var_92;
    wp::int32 var_93;
    const wp::int32 var_94 = 0;
    const wp::int32 var_95 = 0;
    bool var_96;
    wp::int32 var_97;
    wp::int32 var_98;
    const wp::int32 var_99 = 1;
    bool var_100;
    wp::int32 var_101;
    wp::int32 var_102;
    wp::int32 var_103;
    wp::int32 var_104;
    wp::int32 var_105;
    const wp::int32 var_106 = 2;
    wp::int32 var_107;
    wp::int32 var_108;
    wp::int32 var_109;
    wp::int32 var_110;
    wp::int32 var_111;
    wp::int32 var_112;
    const wp::int32 var_113 = 1;
    wp::int32 var_114;
    wp::int32 var_115;
    const wp::int32 var_116 = 1;
    wp::int32 var_117;
    //---------
    // forward
    // def get_face_metadata(                                                                 <L 1073>
    // size01 = cellnum_y * cellnum_z                                                         <L 1081>
    var_0 = wp::mul(var_cellnum_y, var_cellnum_z);
    // size23 = cellnum_x * cellnum_z                                                         <L 1082>
    var_1 = wp::mul(var_cellnum_x, var_cellnum_z);
    // size45 = cellnum_x * cellnum_y                                                         <L 1083>
    var_2 = wp::mul(var_cellnum_x, var_cellnum_y);
    // face_id = 0                                                                            <L 1085>
    // within_face = 0                                                                        <L 1086>
    // if face_elem_idx < size01:                                                             <L 1088>
    var_5 = (var_face_elem_idx < var_0);
    if (var_5) {
        // face_id = 0                                                                        <L 1089>
        // within_face = face_elem_idx                                                        <L 1090>
        var_7 = wp::copy(var_face_elem_idx);
    }
    var_8 = wp::where(var_5, var_6, var_3);
    var_9 = wp::where(var_5, var_7, var_4);
    if (!var_5) {
        // elif face_elem_idx < 2 * size01:                                                   <L 1091>
        var_11 = wp::mul(var_10, var_0);
        var_12 = (var_face_elem_idx < var_11);
        if (var_12) {
            // face_id = 1                                                                    <L 1092>
            // within_face = face_elem_idx - size01                                           <L 1093>
            var_14 = wp::sub(var_face_elem_idx, var_0);
        }
        var_15 = wp::where(var_12, var_13, var_8);
        var_16 = wp::where(var_12, var_14, var_9);
        if (!var_12) {
            // elif face_elem_idx < 2 * size01 + size23:                                      <L 1094>
            var_18 = wp::mul(var_17, var_0);
            var_19 = wp::add(var_18, var_1);
            var_20 = (var_face_elem_idx < var_19);
            if (var_20) {
                // face_id = 2                                                                <L 1095>
                // within_face = face_elem_idx - 2 * size01                                   <L 1096>
                var_23 = wp::mul(var_22, var_0);
                var_24 = wp::sub(var_face_elem_idx, var_23);
            }
            var_25 = wp::where(var_20, var_21, var_15);
            var_26 = wp::where(var_20, var_24, var_16);
            if (!var_20) {
                // elif face_elem_idx < 2 * size01 + 2 * size23:                              <L 1097>
                var_28 = wp::mul(var_27, var_0);
                var_30 = wp::mul(var_29, var_1);
                var_31 = wp::add(var_28, var_30);
                var_32 = (var_face_elem_idx < var_31);
                if (var_32) {
                    // face_id = 3                                                            <L 1098>
                    // within_face = face_elem_idx - 2 * size01 - size23                      <L 1099>
                    var_35 = wp::mul(var_34, var_0);
                    var_36 = wp::sub(var_face_elem_idx, var_35);
                    var_37 = wp::sub(var_36, var_1);
                }
                var_38 = wp::where(var_32, var_33, var_25);
                var_39 = wp::where(var_32, var_37, var_26);
                if (!var_32) {
                    // elif face_elem_idx < 2 * size01 + 2 * size23 + size45:                 <L 1100>
                    var_41 = wp::mul(var_40, var_0);
                    var_43 = wp::mul(var_42, var_1);
                    var_44 = wp::add(var_41, var_43);
                    var_45 = wp::add(var_44, var_2);
                    var_46 = (var_face_elem_idx < var_45);
                    if (var_46) {
                        // face_id = 4                                                        <L 1101>
                        // within_face = face_elem_idx - 2 * size01 - 2 * size23              <L 1102>
                        var_49 = wp::mul(var_48, var_0);
                        var_50 = wp::sub(var_face_elem_idx, var_49);
                        var_52 = wp::mul(var_51, var_1);
                        var_53 = wp::sub(var_50, var_52);
                    }
                    var_54 = wp::where(var_46, var_47, var_38);
                    var_55 = wp::where(var_46, var_53, var_39);
                    if (!var_46) {
                        // face_id = 5                                                        <L 1104>
                        // within_face = face_elem_idx - 2 * size01 - 2 * size23 - size45       <L 1105>
                        var_58 = wp::mul(var_57, var_0);
                        var_59 = wp::sub(var_face_elem_idx, var_58);
                        var_61 = wp::mul(var_60, var_1);
                        var_62 = wp::sub(var_59, var_61);
                        var_63 = wp::sub(var_62, var_2);
                    }
                    var_64 = wp::where(var_46, var_54, var_56);
                    var_65 = wp::where(var_46, var_55, var_63);
                }
                var_66 = wp::where(var_32, var_38, var_64);
                var_67 = wp::where(var_32, var_39, var_65);
            }
            var_68 = wp::where(var_20, var_25, var_66);
            var_69 = wp::where(var_20, var_26, var_67);
        }
        var_70 = wp::where(var_12, var_15, var_68);
        var_71 = wp::where(var_12, var_16, var_69);
    }
    var_72 = wp::where(var_5, var_8, var_70);
    var_73 = wp::where(var_5, var_9, var_71);
    // normal_axis = face_id // 2                                                             <L 1107>
    var_75 = wp::floordiv(var_72, var_74);
    // c1 = 0                                                                                 <L 1109>
    // if face_id == 0 or face_id == 1:                                                       <L 1110>
    var_79 = (var_72 == var_78);
    var_77 = var_79;
    if (!var_77) {
        var_81 = (var_72 == var_80);
        var_77 = var_77 || var_81;
    }
    if (var_77) {
        // c1 = cellnum_z                                                                     <L 1111>
        var_82 = wp::copy(var_cellnum_z);
    }
    var_83 = wp::where(var_77, var_82, var_76);
    if (!var_77) {
        // elif face_id == 2 or face_id == 3:                                                 <L 1112>
        var_86 = (var_72 == var_85);
        var_84 = var_86;
        if (!var_84) {
            var_88 = (var_72 == var_87);
            var_84 = var_84 || var_88;
        }
        if (var_84) {
            // c1 = cellnum_x                                                                 <L 1113>
            var_89 = wp::copy(var_cellnum_x);
        }
        var_90 = wp::where(var_84, var_89, var_83);
        if (!var_84) {
            // c1 = cellnum_y                                                                 <L 1115>
            var_91 = wp::copy(var_cellnum_y);
        }
        var_92 = wp::where(var_84, var_90, var_91);
    }
    var_93 = wp::where(var_77, var_83, var_92);
    // fixed_dim = 0                                                                          <L 1117>
    // if normal_axis == 0:                                                                   <L 1118>
    var_96 = (var_75 == var_95);
    if (var_96) {
        // fixed_dim = cellnum_x                                                              <L 1119>
        var_97 = wp::copy(var_cellnum_x);
    }
    var_98 = wp::where(var_96, var_97, var_94);
    if (!var_96) {
        // elif normal_axis == 1:                                                             <L 1120>
        var_100 = (var_75 == var_99);
        if (var_100) {
            // fixed_dim = cellnum_y                                                          <L 1121>
            var_101 = wp::copy(var_cellnum_y);
        }
        var_102 = wp::where(var_100, var_101, var_98);
        if (!var_100) {
            // fixed_dim = cellnum_z                                                          <L 1123>
            var_103 = wp::copy(var_cellnum_z);
        }
        var_104 = wp::where(var_100, var_102, var_103);
    }
    var_105 = wp::where(var_96, var_98, var_104);
    // g_fixed = (face_id % 2) * fixed_dim * order_abs                                        <L 1124>
    var_107 = wp::mod(var_72, var_106);
    var_108 = wp::mul(var_107, var_105);
    var_109 = wp::mul(var_108, var_order_abs);
    // q0 = within_face // c1                                                                 <L 1126>
    var_110 = wp::floordiv(var_73, var_93);
    // q1 = within_face % c1                                                                  <L 1127>
    var_111 = wp::mod(var_73, var_93);
    // ny_g = cellnum_y * order_abs + 1                                                       <L 1129>
    var_112 = wp::mul(var_cellnum_y, var_order_abs);
    var_114 = wp::add(var_112, var_113);
    // nz_g = cellnum_z * order_abs + 1                                                       <L 1130>
    var_115 = wp::mul(var_cellnum_z, var_order_abs);
    var_117 = wp::add(var_115, var_116);
    // return normal_axis, g_fixed, q0, q1, ny_g, nz_g                                        <L 1132>
    ret_0 = var_75;
    ret_1 = var_109;
    ret_2 = var_110;
    ret_3 = var_111;
    ret_4 = var_114;
    ret_5 = var_117;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:67
static CUDA_CALLABLE wp::quat_t<wp::float32> mat33_to_quat_polar_0(
    wp::mat_t<3, 3, wp::float32> var_F)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    const wp::float32 var_1 = 0.0;
    const wp::float32 var_2 = 0.0;
    const wp::float32 var_3 = 1.0;
    wp::quat_t<wp::float32> var_4;
    const wp::int32 var_5 = 50;
    wp::range_t var_6;
    wp::int32 var_7;
    wp::mat_t<3, 3, wp::float32> var_8;
    wp::mat_t<3, 3, wp::float32> var_9;
    const wp::int32 var_10 = 0;
    wp::vec_t<3, wp::float32> var_11;
    const wp::int32 var_12 = 1;
    wp::vec_t<3, wp::float32> var_13;
    const wp::int32 var_14 = 2;
    wp::vec_t<3, wp::float32> var_15;
    wp::mat_t<3, 3, wp::float32> var_16;
    const wp::int32 var_17 = 0;
    wp::vec_t<3, wp::float32> var_18;
    const wp::int32 var_19 = 1;
    wp::vec_t<3, wp::float32> var_20;
    const wp::int32 var_21 = 2;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::float32 var_34 = 1e-10;
    wp::float32 var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::float32 var_37;
    const wp::float32 var_38 = 1e-06;
    bool var_39;
    wp::vec_t<3, wp::float32> var_40;
    const wp::float32 var_41 = 0.5;
    wp::float32 var_42;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    wp::float32 var_45;
    wp::float32 var_46;
    const wp::int32 var_47 = 1;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::int32 var_51 = 2;
    wp::float32 var_52;
    wp::float32 var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::quat_t<wp::float32> var_56;
    wp::quat_t<wp::float32> var_57;
    wp::quat_t<wp::float32> var_58;
    //---------
    // forward
    // def mat33_to_quat_polar(F: wp.mat33) -> wp.quat:                                       <L 68>
    // cell_quat = wp.quat(0.0, 0.0, 0.0, 1.0)                                                <L 69>
    var_4 = wp::quat_t<wp::float32>(var_0, var_1, var_2, var_3);
    // for _iter in range(50):                                                                <L 70>
    var_6 = wp::range(var_5);
    start_for_0:;
        if (iter_cmp(var_6) == 0) goto end_for_0;
        var_7 = wp::iter_next(var_6);
        // rot = wp.quat_to_matrix(cell_quat)                                                 <L 71>
        var_8 = wp::quat_to_matrix(var_4);
        // rot_t = wp.transpose(rot)                                                          <L 72>
        var_9 = wp::transpose(var_8);
        // col1_rot = rot_t[0]                                                                <L 73>
        var_11 = wp::extract(var_9, var_10);
        // col2_rot = rot_t[1]                                                                <L 74>
        var_13 = wp::extract(var_9, var_12);
        // col3_rot = rot_t[2]                                                                <L 75>
        var_15 = wp::extract(var_9, var_14);
        // F_t = wp.transpose(F)                                                              <L 76>
        var_16 = wp::transpose(var_F);
        // col1_mat = F_t[0]                                                                  <L 77>
        var_18 = wp::extract(var_16, var_17);
        // col2_mat = F_t[1]                                                                  <L 78>
        var_20 = wp::extract(var_16, var_19);
        // col3_mat = F_t[2]                                                                  <L 79>
        var_22 = wp::extract(var_16, var_21);
        // omega = wp.cross(col1_rot, col1_mat) + wp.cross(col2_rot, col2_mat) + wp.cross(col3_rot, col3_mat)       <L 81>
        var_23 = wp::cross(var_11, var_18);
        var_24 = wp::cross(var_13, var_20);
        var_25 = wp::add(var_23, var_24);
        var_26 = wp::cross(var_15, var_22);
        var_27 = wp::add(var_25, var_26);
        // denom = wp.abs(wp.dot(col1_rot, col1_mat) + wp.dot(col2_rot, col2_mat) + wp.dot(col3_rot, col3_mat)) + 1.0e-10       <L 82>
        var_28 = wp::dot(var_11, var_18);
        var_29 = wp::dot(var_13, var_20);
        var_30 = wp::add(var_28, var_29);
        var_31 = wp::dot(var_15, var_22);
        var_32 = wp::add(var_30, var_31);
        var_33 = wp::abs(var_32);
        var_35 = wp::add(var_33, var_34);
        // omega = omega / denom                                                              <L 83>
        var_36 = wp::div(var_27, var_35);
        // w = wp.length(omega)                                                               <L 85>
        var_37 = wp::length(var_36);
        // if w < 1.0e-6:                                                                     <L 86>
        var_39 = (var_37 < var_38);
        if (var_39) {
            // break                                                                          <L 87>
            goto end_for_0;
        }
        // axis = omega / w                                                                   <L 89>
        var_40 = wp::div(var_36, var_37);
        // half_w = 0.5 * w                                                                   <L 90>
        var_42 = wp::mul(var_41, var_37);
        // qrot = wp.quat(                                                                    <L 91>
        // axis[0] * wp.sin(half_w),                                                          <L 92>
        var_44 = wp::extract(var_40, var_43);
        var_45 = wp::sin(var_42);
        var_46 = wp::mul(var_44, var_45);
        // axis[1] * wp.sin(half_w),                                                          <L 93>
        var_48 = wp::extract(var_40, var_47);
        var_49 = wp::sin(var_42);
        var_50 = wp::mul(var_48, var_49);
        // axis[2] * wp.sin(half_w),                                                          <L 94>
        var_52 = wp::extract(var_40, var_51);
        var_53 = wp::sin(var_42);
        var_54 = wp::mul(var_52, var_53);
        // wp.cos(half_w),                                                                    <L 95>
        var_55 = wp::cos(var_42);
        var_56 = wp::quat_t<wp::float32>(var_46, var_50, var_54, var_55);
        // cell_quat = wp.normalize(qrot * cell_quat)                                         <L 97>
        var_57 = wp::mul(var_56, var_4);
        var_58 = wp::normalize(var_57);
        wp::assign(var_4, var_58);
        goto start_for_0;
    end_for_0:;
    // return cell_quat                                                                       <L 98>
    return var_4;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:864
static CUDA_CALLABLE wp::float32 _poly_force_0(
    wp::float32 var_linear,
    wp::vec_t<2, wp::float32> var_poly,
    wp::float32 var_x,
    wp::int32 var_flg_odd)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    bool var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    //---------
    // forward
    // def _poly_force(linear: float, poly: wp.vec2, x: float, flg_odd: int) -> float:        <L 865>
    // x_val = wp.where(flg_odd == 1, wp.abs(x), x)                                           <L 866>
    var_1 = (var_flg_odd == var_0);
    var_2 = wp::abs(var_x);
    var_3 = wp::where(var_1, var_2, var_x);
    // res = linear                                                                           <L 867>
    var_4 = wp::copy(var_linear);
    // res += poly[0] * x_val                                                                 <L 868>
    var_6 = wp::extract(var_poly, var_5);
    var_7 = wp::mul(var_6, var_3);
    var_8 = wp::add(var_4, var_7);
    // res += poly[1] * x_val * x_val                                                         <L 869>
    var_10 = wp::extract(var_poly, var_9);
    var_11 = wp::mul(var_10, var_3);
    var_12 = wp::mul(var_11, var_3);
    var_13 = wp::add(var_8, var_12);
    // return res                                                                             <L 870>
    return var_13;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void normalize_with_norm_0(
    wp::vec_t<3, wp::float32> var_x,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::float32 & ret_1)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::float32 var_1 = 0.0;
    bool var_2;
    const wp::float32 var_3 = 0.0;
    wp::vec_t<3, wp::float32> var_4;
    //---------
    // forward
    // def normalize_with_norm(x: Any):                                                       <L 1>
    // norm = wp.length(x)                                                                    <L 2>
    var_0 = wp::length(var_x);
    // if norm == 0.0:                                                                        <L 3>
    var_2 = (var_0 == var_1);
    if (var_2) {
        // return x, 0.0                                                                      <L 4>
        ret_0 = var_x;
        ret_1 = var_3;
        return;
    }
    // return x / norm, norm                                                                  <L 5>
    var_4 = wp::div(var_x, var_0);
    ret_0 = var_4;
    ret_1 = var_0;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\smooth.py:2157
static CUDA_CALLABLE void _accumulate_jac_dot_chain_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_dof_jntid,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_dot_in,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<3, wp::float32> var_pvel_lin,
    wp::vec_t<3, wp::float32> var_dpnt,
    wp::vec_t<3, wp::float32> var_dvel,
    wp::int32 var_bodyid,
    wp::int32 var_rowadr,
    wp::int32 var_rownnz,
    wp::float32 var_scale,
    wp::int32 var_worldid,
    wp::array_t<wp::float32> var_ten_Jdot_out)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::int32* var_8;
    wp::int32 var_9;
    wp::int32 var_10;
    wp::range_t var_11;
    wp::int32 var_12;
    wp::int32 var_13;
    const wp::int32 var_14 = 1;
    wp::int32 var_15;
    wp::int32 var_16;
    const wp::int32 var_17 = 0;
    bool var_18;
    wp::int32 var_19;
    wp::int32* var_20;
    bool var_21;
    wp::int32 var_22;
    const wp::int32 var_23 = 1;
    wp::int32 var_24;
    bool var_25;
    const wp::int32 var_26 = 0;
    bool var_27;
    wp::int32* var_28;
    bool var_29;
    wp::int32 var_30;
    wp::vec_t<6, wp::float32>* var_31;
    wp::vec_t<6, wp::float32> var_32;
    wp::vec_t<6, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<6, wp::float32>* var_36;
    wp::vec_t<6, wp::float32> var_37;
    wp::vec_t<6, wp::float32> var_38;
    wp::int32* var_39;
    wp::int32 var_40;
    wp::int32 var_41;
    wp::int32* var_42;
    wp::int32 var_43;
    wp::int32 var_44;
    wp::int32* var_45;
    wp::int32 var_46;
    wp::int32 var_47;
    bool var_48;
    const wp::int32 var_49 = 1;
    bool var_50;
    bool var_51;
    const wp::int32 var_52 = 0;
    bool var_53;
    const wp::int32 var_54 = 3;
    wp::int32 var_55;
    bool var_56;
    wp::vec_t<6, wp::float32>* var_57;
    wp::vec_t<6, wp::float32> var_58;
    wp::vec_t<6, wp::float32> var_59;
    wp::vec_t<6, wp::float32> var_60;
    wp::vec_t<3, wp::float32> var_61;
    wp::vec_t<3, wp::float32> var_62;
    wp::vec_t<3, wp::float32> var_63;
    wp::vec_t<3, wp::float32> var_64;
    wp::vec_t<3, wp::float32> var_65;
    wp::vec_t<3, wp::float32> var_66;
    wp::vec_t<3, wp::float32> var_67;
    wp::vec_t<3, wp::float32> var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::float32 var_73 = 0.0;
    bool var_74;
    wp::slice_t var_75;
    const wp::int32 var_76 = 0;
    wp::array_t<wp::float32> var_77;
    wp::float32 var_78;
    wp::int32* var_79;
    wp::int32 var_80;
    wp::int32 var_81;
    //---------
    // forward
    // def _accumulate_jac_dot_chain(                                                         <L 2158>
    // ptr = rownnz - 1                                                                       <L 2185>
    var_1 = wp::sub(var_rownnz, var_0);
    // bid = bodyid                                                                           <L 2186>
    var_2 = wp::copy(var_bodyid);
    // while bid > 0:                                                                         <L 2187>
    start_while_0:;
    var_4 = (var_2 > var_3);
    if ((var_4) == false) goto end_while_0;
        // bdofadr = body_dofadr[bid]                                                         <L 2188>
        var_5 = wp::address(var_body_dofadr, var_2);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // bdofnum = body_dofnum[bid]                                                         <L 2189>
        var_8 = wp::address(var_body_dofnum, var_2);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // for k_rev in range(bdofnum):                                                       <L 2191>
        var_11 = wp::range(var_9);
        start_for_2:;
            if (iter_cmp(var_11) == 0) goto end_for_2;
            var_12 = wp::iter_next(var_11);
            // dof = bdofadr + bdofnum - 1 - k_rev                                            <L 2192>
            var_13 = wp::add(var_6, var_9);
            var_15 = wp::sub(var_13, var_14);
            var_16 = wp::sub(var_15, var_12);
            // while ptr >= 0:                                                                <L 2194>
    start_while_4:;
            var_18 = (var_1 >= var_17);
    if ((var_18) == false) goto end_while_4;
                // sparseid = rowadr + ptr                                                    <L 2195>
                var_19 = wp::add(var_rowadr, var_1);
                // if ten_J_colind[sparseid] <= dof:                                          <L 2196>
                var_20 = wp::address(var_ten_J_colind, var_19);
                var_22 = wp::load(var_20);
                var_21 = (var_22 <= var_16);
                if (var_21) {
                    // break                                                                  <L 2197>
                    goto end_while_4;
                }
                // ptr -= 1                                                                   <L 2198>
                var_24 = wp::sub(var_1, var_23);
                wp::assign(var_1, var_24);
    goto start_while_4;
    end_while_4:;
            // if ptr >= 0 and ten_J_colind[sparseid] == dof:                                 <L 2199>
            var_27 = (var_1 >= var_26);
            var_25 = var_27;
            if (var_25) {
                var_28 = wp::address(var_ten_J_colind, var_19);
                var_30 = wp::load(var_28);
                var_29 = (var_30 == var_16);
                var_25 = var_25 && var_29;
            }
            if (var_25) {
                // cdof = cdof_in[worldid, dof]                                               <L 2200>
                var_31 = wp::address(var_cdof_in, var_worldid, var_16);
                var_33 = wp::load(var_31);
                var_32 = wp::copy(var_33);
                // cdof_ang = wp.spatial_top(cdof)                                            <L 2201>
                var_34 = wp::spatial_top(var_32);
                // cdof_lin = wp.spatial_bottom(cdof)                                         <L 2202>
                var_35 = wp::spatial_bottom(var_32);
                // cdof_dot = cdof_dot_in[worldid, dof]                                       <L 2203>
                var_36 = wp::address(var_cdof_dot_in, var_worldid, var_16);
                var_38 = wp::load(var_36);
                var_37 = wp::copy(var_38);
                // dofjntid = dof_jntid[dof]                                                  <L 2206>
                var_39 = wp::address(var_dof_jntid, var_16);
                var_41 = wp::load(var_39);
                var_40 = wp::copy(var_41);
                // jnttype = jnt_type[dofjntid]                                               <L 2207>
                var_42 = wp::address(var_jnt_type, var_40);
                var_44 = wp::load(var_42);
                var_43 = wp::copy(var_44);
                // jntdofadr = jnt_dofadr[dofjntid]                                           <L 2208>
                var_45 = wp::address(var_jnt_dofadr, var_40);
                var_47 = wp::load(var_45);
                var_46 = wp::copy(var_47);
                // if (jnttype == JointType.BALL) or ((jnttype == JointType.FREE) and dof >= jntdofadr + 3):       <L 2209>
                var_50 = (var_43 == var_49);
                var_48 = var_50;
                if (!var_48) {
                    var_53 = (var_43 == var_52);
                    var_51 = var_53;
                    if (var_51) {
                        var_55 = wp::add(var_46, var_54);
                        var_56 = (var_16 >= var_55);
                        var_51 = var_51 && var_56;
                    }
                    var_48 = var_48 || var_51;
                }
                if (var_48) {
                    // cdof_dot = math.motion_cross(cvel_in[worldid, bid], cdof)              <L 2210>
                    var_57 = wp::address(var_cvel_in, var_worldid, var_2);
                    var_59 = wp::load(var_57);
                    var_58 = motion_cross_0(var_59, var_32);
                }
                var_60 = wp::where(var_48, var_58, var_37);
                // cdof_dot_ang = wp.spatial_top(cdof_dot)                                    <L 2212>
                var_61 = wp::spatial_top(var_60);
                // cdof_dot_lin = wp.spatial_bottom(cdof_dot)                                 <L 2213>
                var_62 = wp::spatial_bottom(var_60);
                // jacp_dot = cdof_dot_lin + wp.cross(cdof_dot_ang, offset) + wp.cross(cdof_ang, pvel_lin)       <L 2216>
                var_63 = wp::cross(var_61, var_offset);
                var_64 = wp::add(var_62, var_63);
                var_65 = wp::cross(var_34, var_pvel_lin);
                var_66 = wp::add(var_64, var_65);
                // jacp = cdof_lin + wp.cross(cdof_ang, offset)                               <L 2219>
                var_67 = wp::cross(var_34, var_offset);
                var_68 = wp::add(var_35, var_67);
                // Jdot = (wp.dot(jacp_dot, dpnt) + wp.dot(jacp, dvel)) * scale               <L 2222>
                var_69 = wp::dot(var_66, var_dpnt);
                var_70 = wp::dot(var_68, var_dvel);
                var_71 = wp::add(var_69, var_70);
                var_72 = wp::mul(var_71, var_scale);
                // if Jdot != 0.0:                                                            <L 2223>
                var_74 = (var_72 != var_73);
                if (var_74) {
                    // wp.atomic_add(ten_Jdot_out[worldid], sparseid, Jdot)                   <L 2224>
                    var_75 = wp::slice_t(var_worldid, var_worldid, var_76);
                    var_77 = wp::view(var_ten_Jdot_out, var_75);
                    var_78 = wp::atomic_add(var_77, var_19, var_72);
                }
            }
            goto start_for_2;
        end_for_2:;
        // bid = body_parentid[bid]                                                           <L 2225>
        var_79 = wp::address(var_body_parentid, var_2);
        var_81 = wp::load(var_79);
        var_80 = wp::copy(var_81);
        wp::assign(var_2, var_80);
    goto start_while_0;
    end_while_0:;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:476
static CUDA_CALLABLE wp::vec_t<6, wp::float32> transform_force_0(
    wp::vec_t<3, wp::float32> var_force,
    wp::vec_t<3, wp::float32> var_torque,
    wp::vec_t<3, wp::float32> var_offset)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<6, wp::float32> var_2;
    //---------
    // forward
    // def transform_force(force: wp.vec3, torque: wp.vec3, offset: wp.vec3) -> wp.spatial_vector:       <L 477>
    // return wp.spatial_vector(torque - wp.cross(offset, force), force)                      <L 478>
    var_0 = wp::cross(var_offset, var_force);
    var_1 = wp::sub(var_torque, var_0);
    var_2 = wp::vec_t<6, wp::float32>(var_1, var_force);
    return var_2;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:481
static CUDA_CALLABLE wp::vec_t<6, wp::float32> transform_force_1(
    wp::vec_t<6, wp::float32> var_frc,
    wp::vec_t<3, wp::float32> var_offset)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<6, wp::float32> var_2;
    //---------
    // forward
    // def transform_force(frc: wp.spatial_vector, offset: wp.vec3) -> wp.spatial_vector:       <L 482>
    // force = wp.spatial_top(frc)                                                            <L 483>
    var_0 = wp::spatial_top(var_frc);
    // torque = wp.spatial_bottom(frc)                                                        <L 484>
    var_1 = wp::spatial_bottom(var_frc);
    // return transform_force(force, torque, offset)                                          <L 485>
    var_2 = transform_force_0(var_0, var_1, var_offset);
    return var_2;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:33
static CUDA_CALLABLE wp::quat_t<wp::float32> quat_mul_axis_0(
    wp::quat_t<wp::float32> var_q,
    wp::vec_t<3, wp::float32> var_axis)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 2;
    wp::float32 var_7;
    const wp::int32 var_8 = 1;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 3;
    wp::float32 var_13;
    const wp::int32 var_14 = 2;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::int32 var_18 = 0;
    wp::float32 var_19;
    const wp::int32 var_20 = 0;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 2;
    wp::float32 var_24;
    const wp::int32 var_25 = 2;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    const wp::int32 var_29 = 3;
    wp::float32 var_30;
    const wp::int32 var_31 = 1;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    const wp::int32 var_37 = 1;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 3;
    wp::float32 var_41;
    const wp::int32 var_42 = 0;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::int32 var_46 = 1;
    wp::float32 var_47;
    const wp::int32 var_48 = 2;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::int32 var_52 = 0;
    wp::float32 var_53;
    const wp::int32 var_54 = 2;
    wp::float32 var_55;
    wp::float32 var_56;
    const wp::int32 var_57 = 1;
    wp::float32 var_58;
    const wp::int32 var_59 = 1;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 2;
    wp::float32 var_64;
    const wp::int32 var_65 = 0;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::quat_t<wp::float32> var_69;
    //---------
    // forward
    // def quat_mul_axis(q: wp.quat, axis: wp.vec3f) -> wp.quat:                              <L 34>
    // return wp.quat(                                                                        <L 36>
    // -q[1] * axis[0] - q[2] * axis[1] - q[3] * axis[2],                                     <L 37>
    var_1 = wp::extract(var_q, var_0);
    var_2 = wp::neg(var_1);
    var_4 = wp::extract(var_axis, var_3);
    var_5 = wp::mul(var_2, var_4);
    var_7 = wp::extract(var_q, var_6);
    var_9 = wp::extract(var_axis, var_8);
    var_10 = wp::mul(var_7, var_9);
    var_11 = wp::sub(var_5, var_10);
    var_13 = wp::extract(var_q, var_12);
    var_15 = wp::extract(var_axis, var_14);
    var_16 = wp::mul(var_13, var_15);
    var_17 = wp::sub(var_11, var_16);
    // q[0] * axis[0] + q[2] * axis[2] - q[3] * axis[1],                                      <L 38>
    var_19 = wp::extract(var_q, var_18);
    var_21 = wp::extract(var_axis, var_20);
    var_22 = wp::mul(var_19, var_21);
    var_24 = wp::extract(var_q, var_23);
    var_26 = wp::extract(var_axis, var_25);
    var_27 = wp::mul(var_24, var_26);
    var_28 = wp::add(var_22, var_27);
    var_30 = wp::extract(var_q, var_29);
    var_32 = wp::extract(var_axis, var_31);
    var_33 = wp::mul(var_30, var_32);
    var_34 = wp::sub(var_28, var_33);
    // q[0] * axis[1] + q[3] * axis[0] - q[1] * axis[2],                                      <L 39>
    var_36 = wp::extract(var_q, var_35);
    var_38 = wp::extract(var_axis, var_37);
    var_39 = wp::mul(var_36, var_38);
    var_41 = wp::extract(var_q, var_40);
    var_43 = wp::extract(var_axis, var_42);
    var_44 = wp::mul(var_41, var_43);
    var_45 = wp::add(var_39, var_44);
    var_47 = wp::extract(var_q, var_46);
    var_49 = wp::extract(var_axis, var_48);
    var_50 = wp::mul(var_47, var_49);
    var_51 = wp::sub(var_45, var_50);
    // q[0] * axis[2] + q[1] * axis[1] - q[2] * axis[0],                                      <L 40>
    var_53 = wp::extract(var_q, var_52);
    var_55 = wp::extract(var_axis, var_54);
    var_56 = wp::mul(var_53, var_55);
    var_58 = wp::extract(var_q, var_57);
    var_60 = wp::extract(var_axis, var_59);
    var_61 = wp::mul(var_58, var_60);
    var_62 = wp::add(var_56, var_61);
    var_64 = wp::extract(var_q, var_63);
    var_66 = wp::extract(var_axis, var_65);
    var_67 = wp::mul(var_64, var_66);
    var_68 = wp::sub(var_62, var_67);
    var_69 = wp::quat_t<wp::float32>(var_17, var_34, var_51, var_68);
    return var_69;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:52
static CUDA_CALLABLE wp::quat_t<wp::float32> axis_angle_to_quat_0(
    wp::vec_t<3, wp::float32> var_axis,
    wp::float32 var_angle)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.5;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::float32 var_3 = 0.5;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32> var_6;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::quat_t<wp::float32> var_13;
    //---------
    // forward
    // def axis_angle_to_quat(axis: wp.vec3, angle: float) -> wp.quat:                        <L 53>
    // s, c = wp.sin(angle * 0.5), wp.cos(angle * 0.5)                                        <L 54>
    var_1 = wp::mul(var_angle, var_0);
    var_2 = wp::sin(var_1);
    var_4 = wp::mul(var_angle, var_3);
    var_5 = wp::cos(var_4);
    // axis = axis * s                                                                        <L 55>
    var_6 = wp::mul(var_axis, var_2);
    // return wp.quat(c, axis[0], axis[1], axis[2])                                           <L 56>
    var_8 = wp::extract(var_6, var_7);
    var_10 = wp::extract(var_6, var_9);
    var_12 = wp::extract(var_6, var_11);
    var_13 = wp::quat_t<wp::float32>(var_5, var_8, var_10, var_12);
    return var_13;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:59
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> quat_to_mat_0(
    wp::quat_t<wp::float32> var_quat)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    const wp::int32 var_10 = 0;
    wp::float32 var_11;
    const wp::int32 var_12 = 2;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    const wp::int32 var_17 = 3;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    const wp::int32 var_22 = 1;
    wp::float32 var_23;
    wp::float32 var_24;
    const wp::int32 var_25 = 1;
    wp::float32 var_26;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    wp::float32 var_29;
    const wp::int32 var_30 = 1;
    wp::float32 var_31;
    const wp::int32 var_32 = 3;
    wp::float32 var_33;
    wp::float32 var_34;
    const wp::int32 var_35 = 2;
    wp::float32 var_36;
    const wp::int32 var_37 = 2;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    const wp::int32 var_42 = 3;
    wp::float32 var_43;
    wp::float32 var_44;
    const wp::int32 var_45 = 3;
    wp::float32 var_46;
    const wp::int32 var_47 = 3;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::float32 var_53 = 2.0;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::float32 var_56 = 2.0;
    wp::float32 var_57;
    wp::float32 var_58;
    const wp::float32 var_59 = 2.0;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    wp::float32 var_63;
    wp::float32 var_64;
    const wp::float32 var_65 = 2.0;
    wp::float32 var_66;
    wp::float32 var_67;
    const wp::float32 var_68 = 2.0;
    wp::float32 var_69;
    wp::float32 var_70;
    const wp::float32 var_71 = 2.0;
    wp::float32 var_72;
    wp::float32 var_73;
    wp::float32 var_74;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::mat_t<3, 3, wp::float32> var_77;
    //---------
    // forward
    // def quat_to_mat(quat: wp.quat) -> wp.mat33:                                            <L 60>
    // q00 = quat[0] * quat[0]                                                                <L 62>
    var_1 = wp::extract(var_quat, var_0);
    var_3 = wp::extract(var_quat, var_2);
    var_4 = wp::mul(var_1, var_3);
    // q01 = quat[0] * quat[1]                                                                <L 63>
    var_6 = wp::extract(var_quat, var_5);
    var_8 = wp::extract(var_quat, var_7);
    var_9 = wp::mul(var_6, var_8);
    // q02 = quat[0] * quat[2]                                                                <L 64>
    var_11 = wp::extract(var_quat, var_10);
    var_13 = wp::extract(var_quat, var_12);
    var_14 = wp::mul(var_11, var_13);
    // q03 = quat[0] * quat[3]                                                                <L 65>
    var_16 = wp::extract(var_quat, var_15);
    var_18 = wp::extract(var_quat, var_17);
    var_19 = wp::mul(var_16, var_18);
    // q11 = quat[1] * quat[1]                                                                <L 66>
    var_21 = wp::extract(var_quat, var_20);
    var_23 = wp::extract(var_quat, var_22);
    var_24 = wp::mul(var_21, var_23);
    // q12 = quat[1] * quat[2]                                                                <L 67>
    var_26 = wp::extract(var_quat, var_25);
    var_28 = wp::extract(var_quat, var_27);
    var_29 = wp::mul(var_26, var_28);
    // q13 = quat[1] * quat[3]                                                                <L 68>
    var_31 = wp::extract(var_quat, var_30);
    var_33 = wp::extract(var_quat, var_32);
    var_34 = wp::mul(var_31, var_33);
    // q22 = quat[2] * quat[2]                                                                <L 69>
    var_36 = wp::extract(var_quat, var_35);
    var_38 = wp::extract(var_quat, var_37);
    var_39 = wp::mul(var_36, var_38);
    // q23 = quat[2] * quat[3]                                                                <L 70>
    var_41 = wp::extract(var_quat, var_40);
    var_43 = wp::extract(var_quat, var_42);
    var_44 = wp::mul(var_41, var_43);
    // q33 = quat[3] * quat[3]                                                                <L 71>
    var_46 = wp::extract(var_quat, var_45);
    var_48 = wp::extract(var_quat, var_47);
    var_49 = wp::mul(var_46, var_48);
    // return wp.mat33(                                                                       <L 73>
    // q00 + q11 - q22 - q33,                                                                 <L 74>
    var_50 = wp::add(var_4, var_24);
    var_51 = wp::sub(var_50, var_39);
    var_52 = wp::sub(var_51, var_49);
    // 2.0 * (q12 - q03),                                                                     <L 75>
    var_54 = wp::sub(var_29, var_19);
    var_55 = wp::mul(var_53, var_54);
    // 2.0 * (q13 + q02),                                                                     <L 76>
    var_57 = wp::add(var_34, var_14);
    var_58 = wp::mul(var_56, var_57);
    // 2.0 * (q12 + q03),                                                                     <L 77>
    var_60 = wp::add(var_29, var_19);
    var_61 = wp::mul(var_59, var_60);
    // q00 - q11 + q22 - q33,                                                                 <L 78>
    var_62 = wp::sub(var_4, var_24);
    var_63 = wp::add(var_62, var_39);
    var_64 = wp::sub(var_63, var_49);
    // 2.0 * (q23 - q01),                                                                     <L 79>
    var_66 = wp::sub(var_44, var_9);
    var_67 = wp::mul(var_65, var_66);
    // 2.0 * (q13 - q02),                                                                     <L 80>
    var_69 = wp::sub(var_34, var_14);
    var_70 = wp::mul(var_68, var_69);
    // 2.0 * (q23 + q01),                                                                     <L 81>
    var_72 = wp::add(var_44, var_9);
    var_73 = wp::mul(var_71, var_72);
    // q00 - q11 - q22 + q33,                                                                 <L 82>
    var_74 = wp::sub(var_4, var_24);
    var_75 = wp::sub(var_74, var_39);
    var_76 = wp::add(var_75, var_49);
    var_77 = wp::mat_t<3, 3, wp::float32>(var_52, var_55, var_58, var_61, var_64, var_67, var_70, var_73, var_76);
    return var_77;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:991
static CUDA_CALLABLE wp::float32 _phi_0(
    wp::float32 var_s,
    wp::int32 var_i)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    const wp::float32 var_2 = 1.0;
    wp::float32 var_3;
    //---------
    // forward
    // def _phi(s: float, i: int) -> float:                                                   <L 992>
    // if i == 0:                                                                             <L 998>
    var_1 = (var_i == var_0);
    if (var_1) {
        // return 1.0 - s                                                                     <L 999>
        var_3 = wp::sub(var_2, var_s);
        return var_3;
    }
    // return s                                                                               <L 1000>
    return var_s;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1003
static CUDA_CALLABLE wp::float32 eval_basis_trilinear_0(
    wp::vec_t<3, wp::float32> var_local,
    wp::int32 var_node_idx)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::int32 var_1;
    const wp::int32 var_2 = 1;
    wp::int32 var_3;
    const wp::int32 var_4 = 1;
    wp::int32 var_5;
    const wp::int32 var_6 = 2;
    wp::int32 var_7;
    const wp::int32 var_8 = 1;
    wp::int32 var_9;
    const wp::int32 var_10 = 0;
    wp::float32 var_11;
    wp::float32 var_12;
    const wp::int32 var_13 = 1;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 2;
    wp::float32 var_18;
    wp::float32 var_19;
    wp::float32 var_20;
    //---------
    // forward
    // def eval_basis_trilinear(local: wp.vec3, node_idx: int) -> float:                      <L 1004>
    // k = node_idx & 1                                                                       <L 1010>
    var_1 = wp::bit_and(var_node_idx, var_0);
    // j = (node_idx >> 1) & 1                                                                <L 1011>
    var_3 = wp::rshift(var_node_idx, var_2);
    var_5 = wp::bit_and(var_3, var_4);
    // i = (node_idx >> 2) & 1                                                                <L 1012>
    var_7 = wp::rshift(var_node_idx, var_6);
    var_9 = wp::bit_and(var_7, var_8);
    // return _phi(local[0], i) * _phi(local[1], j) * _phi(local[2], k)                       <L 1013>
    var_11 = wp::extract(var_local, var_10);
    var_12 = _phi_0(var_11, var_9);
    var_14 = wp::extract(var_local, var_13);
    var_15 = _phi_0(var_14, var_5);
    var_16 = wp::mul(var_12, var_15);
    var_18 = wp::extract(var_local, var_17);
    var_19 = _phi_0(var_18, var_1);
    var_20 = wp::mul(var_16, var_19);
    return var_20;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:120
static CUDA_CALLABLE wp::vec_t<6, wp::float32> inert_vec_0(
    wp::vec_t<10, wp::float32> var_i,
    wp::vec_t<6, wp::float32> var_v)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 3;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 4;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 8;
    wp::float32 var_18;
    const wp::int32 var_19 = 4;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 7;
    wp::float32 var_24;
    const wp::int32 var_25 = 5;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    const wp::int32 var_29 = 3;
    wp::float32 var_30;
    const wp::int32 var_31 = 0;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::int32 var_34 = 1;
    wp::float32 var_35;
    const wp::int32 var_36 = 1;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 5;
    wp::float32 var_41;
    const wp::int32 var_42 = 2;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::int32 var_46 = 8;
    wp::float32 var_47;
    const wp::int32 var_48 = 3;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::int32 var_52 = 6;
    wp::float32 var_53;
    const wp::int32 var_54 = 5;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::float32 var_57;
    const wp::int32 var_58 = 4;
    wp::float32 var_59;
    const wp::int32 var_60 = 0;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 5;
    wp::float32 var_64;
    const wp::int32 var_65 = 1;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    const wp::int32 var_69 = 2;
    wp::float32 var_70;
    const wp::int32 var_71 = 2;
    wp::float32 var_72;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::int32 var_75 = 7;
    wp::float32 var_76;
    const wp::int32 var_77 = 3;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::float32 var_80;
    const wp::int32 var_81 = 6;
    wp::float32 var_82;
    const wp::int32 var_83 = 4;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::float32 var_86;
    const wp::int32 var_87 = 8;
    wp::float32 var_88;
    const wp::int32 var_89 = 1;
    wp::float32 var_90;
    wp::float32 var_91;
    const wp::int32 var_92 = 7;
    wp::float32 var_93;
    const wp::int32 var_94 = 2;
    wp::float32 var_95;
    wp::float32 var_96;
    wp::float32 var_97;
    const wp::int32 var_98 = 9;
    wp::float32 var_99;
    const wp::int32 var_100 = 3;
    wp::float32 var_101;
    wp::float32 var_102;
    wp::float32 var_103;
    const wp::int32 var_104 = 6;
    wp::float32 var_105;
    const wp::int32 var_106 = 2;
    wp::float32 var_107;
    wp::float32 var_108;
    const wp::int32 var_109 = 8;
    wp::float32 var_110;
    const wp::int32 var_111 = 0;
    wp::float32 var_112;
    wp::float32 var_113;
    wp::float32 var_114;
    const wp::int32 var_115 = 9;
    wp::float32 var_116;
    const wp::int32 var_117 = 4;
    wp::float32 var_118;
    wp::float32 var_119;
    wp::float32 var_120;
    const wp::int32 var_121 = 7;
    wp::float32 var_122;
    const wp::int32 var_123 = 0;
    wp::float32 var_124;
    wp::float32 var_125;
    const wp::int32 var_126 = 6;
    wp::float32 var_127;
    const wp::int32 var_128 = 1;
    wp::float32 var_129;
    wp::float32 var_130;
    wp::float32 var_131;
    const wp::int32 var_132 = 9;
    wp::float32 var_133;
    const wp::int32 var_134 = 5;
    wp::float32 var_135;
    wp::float32 var_136;
    wp::float32 var_137;
    wp::vec_t<6, wp::float32> var_138;
    //---------
    // forward
    // def inert_vec(i: types.vec10, v: wp.spatial_vector) -> wp.spatial_vector:              <L 121>
    // return wp.spatial_vector(                                                              <L 123>
    // i[0] * v[0] + i[3] * v[1] + i[4] * v[2] - i[8] * v[4] + i[7] * v[5],                   <L 124>
    var_1 = wp::extract(var_i, var_0);
    var_3 = wp::extract(var_v, var_2);
    var_4 = wp::mul(var_1, var_3);
    var_6 = wp::extract(var_i, var_5);
    var_8 = wp::extract(var_v, var_7);
    var_9 = wp::mul(var_6, var_8);
    var_10 = wp::add(var_4, var_9);
    var_12 = wp::extract(var_i, var_11);
    var_14 = wp::extract(var_v, var_13);
    var_15 = wp::mul(var_12, var_14);
    var_16 = wp::add(var_10, var_15);
    var_18 = wp::extract(var_i, var_17);
    var_20 = wp::extract(var_v, var_19);
    var_21 = wp::mul(var_18, var_20);
    var_22 = wp::sub(var_16, var_21);
    var_24 = wp::extract(var_i, var_23);
    var_26 = wp::extract(var_v, var_25);
    var_27 = wp::mul(var_24, var_26);
    var_28 = wp::add(var_22, var_27);
    // i[3] * v[0] + i[1] * v[1] + i[5] * v[2] + i[8] * v[3] - i[6] * v[5],                   <L 125>
    var_30 = wp::extract(var_i, var_29);
    var_32 = wp::extract(var_v, var_31);
    var_33 = wp::mul(var_30, var_32);
    var_35 = wp::extract(var_i, var_34);
    var_37 = wp::extract(var_v, var_36);
    var_38 = wp::mul(var_35, var_37);
    var_39 = wp::add(var_33, var_38);
    var_41 = wp::extract(var_i, var_40);
    var_43 = wp::extract(var_v, var_42);
    var_44 = wp::mul(var_41, var_43);
    var_45 = wp::add(var_39, var_44);
    var_47 = wp::extract(var_i, var_46);
    var_49 = wp::extract(var_v, var_48);
    var_50 = wp::mul(var_47, var_49);
    var_51 = wp::add(var_45, var_50);
    var_53 = wp::extract(var_i, var_52);
    var_55 = wp::extract(var_v, var_54);
    var_56 = wp::mul(var_53, var_55);
    var_57 = wp::sub(var_51, var_56);
    // i[4] * v[0] + i[5] * v[1] + i[2] * v[2] - i[7] * v[3] + i[6] * v[4],                   <L 126>
    var_59 = wp::extract(var_i, var_58);
    var_61 = wp::extract(var_v, var_60);
    var_62 = wp::mul(var_59, var_61);
    var_64 = wp::extract(var_i, var_63);
    var_66 = wp::extract(var_v, var_65);
    var_67 = wp::mul(var_64, var_66);
    var_68 = wp::add(var_62, var_67);
    var_70 = wp::extract(var_i, var_69);
    var_72 = wp::extract(var_v, var_71);
    var_73 = wp::mul(var_70, var_72);
    var_74 = wp::add(var_68, var_73);
    var_76 = wp::extract(var_i, var_75);
    var_78 = wp::extract(var_v, var_77);
    var_79 = wp::mul(var_76, var_78);
    var_80 = wp::sub(var_74, var_79);
    var_82 = wp::extract(var_i, var_81);
    var_84 = wp::extract(var_v, var_83);
    var_85 = wp::mul(var_82, var_84);
    var_86 = wp::add(var_80, var_85);
    // i[8] * v[1] - i[7] * v[2] + i[9] * v[3],                                               <L 127>
    var_88 = wp::extract(var_i, var_87);
    var_90 = wp::extract(var_v, var_89);
    var_91 = wp::mul(var_88, var_90);
    var_93 = wp::extract(var_i, var_92);
    var_95 = wp::extract(var_v, var_94);
    var_96 = wp::mul(var_93, var_95);
    var_97 = wp::sub(var_91, var_96);
    var_99 = wp::extract(var_i, var_98);
    var_101 = wp::extract(var_v, var_100);
    var_102 = wp::mul(var_99, var_101);
    var_103 = wp::add(var_97, var_102);
    // i[6] * v[2] - i[8] * v[0] + i[9] * v[4],                                               <L 128>
    var_105 = wp::extract(var_i, var_104);
    var_107 = wp::extract(var_v, var_106);
    var_108 = wp::mul(var_105, var_107);
    var_110 = wp::extract(var_i, var_109);
    var_112 = wp::extract(var_v, var_111);
    var_113 = wp::mul(var_110, var_112);
    var_114 = wp::sub(var_108, var_113);
    var_116 = wp::extract(var_i, var_115);
    var_118 = wp::extract(var_v, var_117);
    var_119 = wp::mul(var_116, var_118);
    var_120 = wp::add(var_114, var_119);
    // i[7] * v[0] - i[6] * v[1] + i[9] * v[5],                                               <L 129>
    var_122 = wp::extract(var_i, var_121);
    var_124 = wp::extract(var_v, var_123);
    var_125 = wp::mul(var_122, var_124);
    var_127 = wp::extract(var_i, var_126);
    var_129 = wp::extract(var_v, var_128);
    var_130 = wp::mul(var_127, var_129);
    var_131 = wp::sub(var_125, var_130);
    var_133 = wp::extract(var_i, var_132);
    var_135 = wp::extract(var_v, var_134);
    var_136 = wp::mul(var_133, var_135);
    var_137 = wp::add(var_131, var_136);
    var_138 = wp::vec_t<6, wp::float32>({var_28, var_57, var_86, var_103, var_120, var_137});
    return var_138;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:147
static CUDA_CALLABLE wp::vec_t<6, wp::float32> motion_cross_force_0(
    wp::vec_t<6, wp::float32> var_v,
    wp::vec_t<6, wp::float32> var_f)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 2;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32> var_6;
    const wp::int32 var_7 = 3;
    wp::float32 var_8;
    const wp::int32 var_9 = 4;
    wp::float32 var_10;
    const wp::int32 var_11 = 5;
    wp::float32 var_12;
    wp::vec_t<3, wp::float32> var_13;
    const wp::int32 var_14 = 0;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::float32 var_17;
    const wp::int32 var_18 = 2;
    wp::float32 var_19;
    wp::vec_t<3, wp::float32> var_20;
    const wp::int32 var_21 = 3;
    wp::float32 var_22;
    const wp::int32 var_23 = 4;
    wp::float32 var_24;
    const wp::int32 var_25 = 5;
    wp::float32 var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<6, wp::float32> var_32;
    //---------
    // forward
    // def motion_cross_force(v: wp.spatial_vector, f: wp.spatial_vector) -> wp.spatial_vector:       <L 148>
    // v0 = wp.vec3(v[0], v[1], v[2])                                                         <L 150>
    var_1 = wp::extract(var_v, var_0);
    var_3 = wp::extract(var_v, var_2);
    var_5 = wp::extract(var_v, var_4);
    var_6 = wp::vec_t<3, wp::float32>(var_1, var_3, var_5);
    // v1 = wp.vec3(v[3], v[4], v[5])                                                         <L 151>
    var_8 = wp::extract(var_v, var_7);
    var_10 = wp::extract(var_v, var_9);
    var_12 = wp::extract(var_v, var_11);
    var_13 = wp::vec_t<3, wp::float32>(var_8, var_10, var_12);
    // f0 = wp.vec3(f[0], f[1], f[2])                                                         <L 152>
    var_15 = wp::extract(var_f, var_14);
    var_17 = wp::extract(var_f, var_16);
    var_19 = wp::extract(var_f, var_18);
    var_20 = wp::vec_t<3, wp::float32>(var_15, var_17, var_19);
    // f1 = wp.vec3(f[3], f[4], f[5])                                                         <L 153>
    var_22 = wp::extract(var_f, var_21);
    var_24 = wp::extract(var_f, var_23);
    var_26 = wp::extract(var_f, var_25);
    var_27 = wp::vec_t<3, wp::float32>(var_22, var_24, var_26);
    // ang = wp.cross(v0, f0) + wp.cross(v1, f1)                                              <L 155>
    var_28 = wp::cross(var_6, var_20);
    var_29 = wp::cross(var_13, var_27);
    var_30 = wp::add(var_28, var_29);
    // vel = wp.cross(v0, f1)                                                                 <L 156>
    var_31 = wp::cross(var_6, var_27);
    // return wp.spatial_vector(ang, vel)                                                     <L 158>
    var_32 = wp::vec_t<6, wp::float32>(var_30, var_31);
    return var_32;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:325
static CUDA_CALLABLE wp::vec_t<6, wp::float32> _decode_pyramid_0(
    wp::int32 var_njmax_in,
    wp::array_t<wp::float32> var_pyramid,
    wp::int32 var_efc_address,
    wp::vec_t<5, wp::float32> var_mu,
    wp::int32 var_condim)
{
    //---------
    // primal vars
    wp::vec_t<6, wp::float32> var_0;
    const wp::int32 var_1 = 1;
    bool var_2;
    wp::float32* var_3;
    const wp::int32 var_4 = 0;
    wp::float32 var_5;
    const wp::float32 var_6 = 0.0;
    wp::float32 var_7;
    const wp::int32 var_8 = 0;
    const wp::int32 var_9 = 1;
    wp::int32 var_10;
    wp::range_t var_11;
    wp::int32 var_12;
    const wp::int32 var_13 = 2;
    wp::int32 var_14;
    wp::int32 var_15;
    bool var_16;
    wp::float32* var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::float32 var_20 = 0.0;
    wp::float32 var_21;
    const wp::int32 var_22 = 1;
    wp::int32 var_23;
    bool var_24;
    const wp::int32 var_25 = 1;
    wp::int32 var_26;
    wp::float32* var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    const wp::float32 var_30 = 0.0;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::int32 var_33 = 0;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    const wp::int32 var_37 = 1;
    wp::int32 var_38;
    //---------
    // forward
    // def _decode_pyramid(njmax_in: int, pyramid: wp.array[float], efc_address: int, mu: vec5, condim: int) -> wp.spatial_vector:       <L 326>
    // force = wp.spatial_vector()                                                            <L 328>
    var_0 = wp::vec_t<6, wp::float32>();
    // if condim == 1:                                                                        <L 330>
    var_2 = (var_condim == var_1);
    if (var_2) {
        // force[0] = pyramid[efc_address]                                                    <L 331>
        var_3 = wp::address(var_pyramid, var_efc_address);
        var_5 = wp::load(var_3);
        wp::assign_inplace(var_0, var_4, var_5);
        // return force                                                                       <L 332>
        return var_0;
    }
    // force[0] = float(0.0)                                                                  <L 334>
    var_7 = wp::float(var_6);
    wp::assign_inplace(var_0, var_8, var_7);
    // for i in range(condim - 1):                                                            <L 335>
    var_10 = wp::sub(var_condim, var_9);
    var_11 = wp::range(var_10);
    start_for_1:;
        if (iter_cmp(var_11) == 0) goto end_for_1;
        var_12 = wp::iter_next(var_11);
        // adr = 2 * i + efc_address                                                          <L 336>
        var_14 = wp::mul(var_13, var_12);
        var_15 = wp::add(var_14, var_efc_address);
        // if adr < njmax_in:                                                                 <L 337>
        var_16 = (var_15 < var_njmax_in);
        if (var_16) {
            // dir1 = pyramid[adr]                                                            <L 338>
            var_17 = wp::address(var_pyramid, var_15);
            var_19 = wp::load(var_17);
            var_18 = wp::copy(var_19);
        }
        if (!var_16) {
            // dir1 = 0.0                                                                     <L 340>
        }
        var_21 = wp::where(var_16, var_18, var_20);
        // if adr + 1 < njmax_in:                                                             <L 341>
        var_23 = wp::add(var_15, var_22);
        var_24 = (var_23 < var_njmax_in);
        if (var_24) {
            // dir2 = pyramid[adr + 1]                                                        <L 342>
            var_26 = wp::add(var_15, var_25);
            var_27 = wp::address(var_pyramid, var_26);
            var_29 = wp::load(var_27);
            var_28 = wp::copy(var_29);
        }
        if (!var_24) {
            // dir2 = 0.0                                                                     <L 344>
        }
        var_31 = wp::where(var_24, var_28, var_30);
        // force[0] += dir1 + dir2                                                            <L 345>
        var_32 = wp::add(var_21, var_31);
        wp::add_inplace(var_0, var_33, var_32);
        // force[i + 1] = (dir1 - dir2) * mu[i]                                               <L 346>
        var_34 = wp::sub(var_21, var_31);
        var_35 = wp::extract(var_mu, var_12);
        var_36 = wp::mul(var_34, var_35);
        var_38 = wp::add(var_12, var_37);
        wp::assign_inplace(var_0, var_38, var_36);
        goto start_for_1;
    end_for_1:;
    // return force                                                                           <L 348>
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:351
static CUDA_CALLABLE wp::vec_t<6, wp::float32> contact_force_fn_0(
    wp::int32 var_opt_cone,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::vec_t<5, wp::float32>> var_contact_friction_in,
    wp::array_t<wp::int32> var_contact_dim_in,
    wp::array_t<wp::int32> var_contact_efc_address_in,
    wp::array_t<wp::float32> var_contact_adhesion_in,
    wp::array_t<wp::float32> var_efc_force_in,
    wp::int32 var_njmax_in,
    wp::array_t<wp::int32> var_nacon_in,
    wp::int32 var_worldid,
    wp::int32 var_contact_id,
    bool var_to_world_frame)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    const wp::float32 var_1 = 0.0;
    const wp::float32 var_2 = 0.0;
    const wp::float32 var_3 = 0.0;
    const wp::float32 var_4 = 0.0;
    const wp::float32 var_5 = 0.0;
    wp::vec_t<6, wp::float32> var_6;
    wp::int32* var_7;
    wp::int32 var_8;
    wp::int32 var_9;
    const wp::int32 var_10 = 0;
    wp::int32* var_11;
    wp::int32 var_12;
    wp::int32 var_13;
    bool var_14;
    const wp::int32 var_15 = 0;
    bool var_16;
    const wp::int32 var_17 = 0;
    wp::int32* var_18;
    bool var_19;
    wp::int32 var_20;
    const wp::int32 var_21 = 0;
    bool var_22;
    const wp::int32 var_23 = 0;
    bool var_24;
    wp::slice_t var_25;
    const wp::int32 var_26 = 0;
    wp::array_t<wp::float32> var_27;
    wp::vec_t<5, wp::float32>* var_28;
    wp::vec_t<6, wp::float32> var_29;
    wp::vec_t<5, wp::float32> var_30;
    wp::vec_t<6, wp::float32> var_31;
    wp::range_t var_32;
    wp::int32 var_33;
    wp::int32* var_34;
    bool var_35;
    wp::int32 var_36;
    wp::int32* var_37;
    wp::float32* var_38;
    wp::int32 var_39;
    wp::float32 var_40;
    wp::float32* var_41;
    const wp::int32 var_42 = 0;
    wp::float32 var_43;
    wp::vec_t<6, wp::float32> var_44;
    wp::vec_t<3, wp::float32> var_45;
    wp::mat_t<3, 3, wp::float32>* var_46;
    wp::vec_t<3, wp::float32> var_47;
    wp::mat_t<3, 3, wp::float32> var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::mat_t<3, 3, wp::float32>* var_50;
    wp::vec_t<3, wp::float32> var_51;
    wp::mat_t<3, 3, wp::float32> var_52;
    wp::vec_t<6, wp::float32> var_53;
    wp::vec_t<6, wp::float32> var_54;
    //---------
    // forward
    // def contact_force_fn(                                                                  <L 352>
    // force = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                <L 370>
    var_6 = wp::vec_t<6, wp::float32>({var_0, var_1, var_2, var_3, var_4, var_5});
    // condim = contact_dim_in[contact_id]                                                    <L 371>
    var_7 = wp::address(var_contact_dim_in, var_contact_id);
    var_9 = wp::load(var_7);
    var_8 = wp::copy(var_9);
    // efc_address = contact_efc_address_in[contact_id, 0]                                    <L 372>
    var_11 = wp::address(var_contact_efc_address_in, var_contact_id, var_10);
    var_13 = wp::load(var_11);
    var_12 = wp::copy(var_13);
    // if contact_id >= 0 and contact_id <= nacon_in[0] and efc_address >= 0:                 <L 374>
    var_16 = (var_contact_id >= var_15);
    var_14 = var_16;
    if (var_14) {
        var_18 = wp::address(var_nacon_in, var_17);
        var_20 = wp::load(var_18);
        var_19 = (var_contact_id <= var_20);
        var_14 = var_14 && var_19;
    }
    if (var_14) {
        var_22 = (var_12 >= var_21);
        var_14 = var_14 && var_22;
    }
    if (var_14) {
        // if opt_cone == ConeType.PYRAMIDAL:                                                 <L 375>
        var_24 = (var_opt_cone == var_23);
        if (var_24) {
            // force = _decode_pyramid(                                                       <L 376>
            // njmax_in,                                                                      <L 377>
            // efc_force_in[worldid],                                                         <L 378>
            var_25 = wp::slice_t(var_worldid, var_worldid, var_26);
            var_27 = wp::view(var_efc_force_in, var_25);
            // efc_address,                                                                   <L 379>
            // contact_friction_in[contact_id],                                               <L 380>
            var_28 = wp::address(var_contact_friction_in, var_contact_id);
            // condim,                                                                        <L 381>
            var_30 = wp::load(var_28);
            var_29 = _decode_pyramid_0(var_njmax_in, var_27, var_12, var_30, var_8);
        }
        var_31 = wp::where(var_24, var_29, var_6);
        if (!var_24) {
            // for i in range(condim):                                                        <L 384>
            var_32 = wp::range(var_8);
            start_for_0:;
                if (iter_cmp(var_32) == 0) goto end_for_0;
                var_33 = wp::iter_next(var_32);
                // if contact_efc_address_in[contact_id, i] < njmax_in:                       <L 385>
                var_34 = wp::address(var_contact_efc_address_in, var_contact_id, var_33);
                var_36 = wp::load(var_34);
                var_35 = (var_36 < var_njmax_in);
                if (var_35) {
                    // force[i] = efc_force_in[worldid, contact_efc_address_in[contact_id, i]]       <L 386>
                    var_37 = wp::address(var_contact_efc_address_in, var_contact_id, var_33);
                    var_39 = wp::load(var_37);
                    var_38 = wp::address(var_efc_force_in, var_worldid, var_39);
                    var_40 = wp::load(var_38);
                    wp::assign_inplace(var_31, var_33, var_40);
                }
                goto start_for_0;
            end_for_0:;
        }
        // force[0] -= contact_adhesion_in[contact_id]                                        <L 389>
        var_41 = wp::address(var_contact_adhesion_in, var_contact_id);
        var_43 = wp::load(var_41);
        wp::sub_inplace(var_31, var_42, var_43);
    }
    var_44 = wp::where(var_14, var_31, var_6);
    // if to_world_frame:                                                                     <L 391>
    if (var_to_world_frame) {
        // t = wp.spatial_top(force) @ contact_frame_in[contact_id]                           <L 393>
        var_45 = wp::spatial_top(var_44);
        var_46 = wp::address(var_contact_frame_in, var_contact_id);
        var_48 = wp::load(var_46);
        var_47 = wp::mul(var_45, var_48);
        // b = wp.spatial_bottom(force) @ contact_frame_in[contact_id]                        <L 394>
        var_49 = wp::spatial_bottom(var_44);
        var_50 = wp::address(var_contact_frame_in, var_contact_id);
        var_52 = wp::load(var_50);
        var_51 = wp::mul(var_49, var_52);
        // force = wp.spatial_vector(t, b)                                                    <L 395>
        var_53 = wp::vec_t<6, wp::float32>(var_47, var_51);
    }
    var_54 = wp::where(var_to_world_frame, var_53, var_44);
    // return force                                                                           <L 397>
    return var_54;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\smooth.py:4025
static CUDA_CALLABLE void _accumulate_jac_chain_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<3, wp::float32> var_vec,
    wp::int32 var_bodyid,
    wp::int32 var_rowadr,
    wp::int32 var_rownnz,
    wp::float32 var_scale,
    wp::int32 var_worldid,
    wp::array_t<wp::float32> var_ten_J_out)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::int32* var_8;
    wp::int32 var_9;
    wp::int32 var_10;
    wp::range_t var_11;
    wp::int32 var_12;
    wp::int32 var_13;
    const wp::int32 var_14 = 1;
    wp::int32 var_15;
    wp::int32 var_16;
    const wp::int32 var_17 = 0;
    bool var_18;
    wp::int32 var_19;
    wp::int32* var_20;
    bool var_21;
    wp::int32 var_22;
    const wp::int32 var_23 = 1;
    wp::int32 var_24;
    bool var_25;
    const wp::int32 var_26 = 0;
    bool var_27;
    wp::int32* var_28;
    bool var_29;
    wp::int32 var_30;
    wp::vec_t<6, wp::float32>* var_31;
    wp::vec_t<6, wp::float32> var_32;
    wp::vec_t<6, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::float32 var_40 = 0.0;
    bool var_41;
    wp::slice_t var_42;
    const wp::int32 var_43 = 0;
    wp::array_t<wp::float32> var_44;
    wp::float32 var_45;
    wp::int32* var_46;
    wp::int32 var_47;
    wp::int32 var_48;
    //---------
    // forward
    // def _accumulate_jac_chain(                                                             <L 4026>
    // ptr = rownnz - 1                                                                       <L 4046>
    var_1 = wp::sub(var_rownnz, var_0);
    // bid = bodyid                                                                           <L 4047>
    var_2 = wp::copy(var_bodyid);
    // while bid > 0:                                                                         <L 4048>
    start_while_0:;
    var_4 = (var_2 > var_3);
    if ((var_4) == false) goto end_while_0;
        // bdofadr = body_dofadr[bid]                                                         <L 4049>
        var_5 = wp::address(var_body_dofadr, var_2);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // bdofnum = body_dofnum[bid]                                                         <L 4050>
        var_8 = wp::address(var_body_dofnum, var_2);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // for k_rev in range(bdofnum):                                                       <L 4052>
        var_11 = wp::range(var_9);
        start_for_2:;
            if (iter_cmp(var_11) == 0) goto end_for_2;
            var_12 = wp::iter_next(var_11);
            // dof = bdofadr + bdofnum - 1 - k_rev                                            <L 4053>
            var_13 = wp::add(var_6, var_9);
            var_15 = wp::sub(var_13, var_14);
            var_16 = wp::sub(var_15, var_12);
            // while ptr >= 0:                                                                <L 4055>
    start_while_4:;
            var_18 = (var_1 >= var_17);
    if ((var_18) == false) goto end_while_4;
                // sparseid = rowadr + ptr                                                    <L 4056>
                var_19 = wp::add(var_rowadr, var_1);
                // if ten_J_colind[sparseid] <= dof:                                          <L 4057>
                var_20 = wp::address(var_ten_J_colind, var_19);
                var_22 = wp::load(var_20);
                var_21 = (var_22 <= var_16);
                if (var_21) {
                    // break                                                                  <L 4058>
                    goto end_while_4;
                }
                // ptr -= 1                                                                   <L 4059>
                var_24 = wp::sub(var_1, var_23);
                wp::assign(var_1, var_24);
    goto start_while_4;
    end_while_4:;
            // if ptr >= 0 and ten_J_colind[sparseid] == dof:                                 <L 4060>
            var_27 = (var_1 >= var_26);
            var_25 = var_27;
            if (var_25) {
                var_28 = wp::address(var_ten_J_colind, var_19);
                var_30 = wp::load(var_28);
                var_29 = (var_30 == var_16);
                var_25 = var_25 && var_29;
            }
            if (var_25) {
                // cdof = cdof_in[worldid, dof]                                               <L 4061>
                var_31 = wp::address(var_cdof_in, var_worldid, var_16);
                var_33 = wp::load(var_31);
                var_32 = wp::copy(var_33);
                // cdof_ang = wp.spatial_top(cdof)                                            <L 4062>
                var_34 = wp::spatial_top(var_32);
                // cdof_lin = wp.spatial_bottom(cdof)                                         <L 4063>
                var_35 = wp::spatial_bottom(var_32);
                // jacp = cdof_lin + wp.cross(cdof_ang, offset)                               <L 4064>
                var_36 = wp::cross(var_34, var_offset);
                var_37 = wp::add(var_35, var_36);
                // J = wp.dot(jacp, vec) * scale                                              <L 4065>
                var_38 = wp::dot(var_37, var_vec);
                var_39 = wp::mul(var_38, var_scale);
                // if J != 0.0:                                                               <L 4066>
                var_41 = (var_39 != var_40);
                if (var_41) {
                    // wp.atomic_add(ten_J_out[worldid], sparseid, J)                         <L 4067>
                    var_42 = wp::slice_t(var_worldid, var_worldid, var_43);
                    var_44 = wp::view(var_ten_J_out, var_42);
                    var_45 = wp::atomic_add(var_44, var_19, var_39);
                }
            }
            goto start_for_2;
        end_for_2:;
        // bid = body_parentid[bid]                                                           <L 4068>
        var_46 = wp::address(var_body_parentid, var_2);
        var_48 = wp::load(var_46);
        var_47 = wp::copy(var_48);
        wp::assign(var_2, var_47);
    goto start_while_0;
    end_while_0:;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void normalize_with_norm_0(
    wp::vec_t<2, wp::float32> var_x,
    wp::vec_t<2, wp::float32> & ret_0,
    wp::float32 & ret_1)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::float32 var_1 = 0.0;
    bool var_2;
    const wp::float32 var_3 = 0.0;
    wp::vec_t<2, wp::float32> var_4;
    //---------
    // forward
    // def normalize_with_norm(x: Any):                                                       <L 1>
    // norm = wp.length(x)                                                                    <L 2>
    var_0 = wp::length(var_x);
    // if norm == 0.0:                                                                        <L 3>
    var_2 = (var_0 == var_1);
    if (var_2) {
        // return x, 0.0                                                                      <L 4>
        ret_0 = var_x;
        ret_1 = var_3;
        return;
    }
    // return x / norm, norm                                                                  <L 5>
    var_4 = wp::div(var_x, var_0);
    ret_0 = var_4;
    ret_1 = var_0;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\warp\_src\math.py:0
static CUDA_CALLABLE wp::float32 norm_l2_0(
    wp::vec_t<2, wp::float32> var_v)
{
    //---------
    // primal vars
    wp::float32 var_0;
    //---------
    // forward
    // def norm_l2(v: Any) -> float:                                                          <L 1>
    // return wp.length(v)                                                                    <L 12>
    var_0 = wp::length(var_v);
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:202
static CUDA_CALLABLE void wrap_inside_0(
    wp::vec_t<4, wp::float32> var_end,
    wp::float32 var_radius,
    wp::int32 var_maxiter,
    wp::float32 var_zinit,
    wp::float32 var_tolerance,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    wp::vec_t<2, wp::float32> var_4;
    const wp::int32 var_5 = 2;
    wp::float32 var_6;
    const wp::int32 var_7 = 3;
    wp::float32 var_8;
    wp::vec_t<2, wp::float32> var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::vec_t<2, wp::float32> var_12;
    wp::float32 var_13;
    bool var_14;
    bool var_15;
    bool var_16;
    const wp::float32 var_17 = 1e-15;
    bool var_18;
    bool var_19;
    bool var_20;
    const wp::float32 var_21 = -1.0;
    const wp::float32 var_22 = 10000000000.0;
    wp::vec_t<2, wp::float32> var_23;
    wp::vec_t<2, wp::float32> var_24;
    bool var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    bool var_29;
    const wp::float32 var_30 = 0.0;
    bool var_31;
    const wp::float32 var_32 = 1.0;
    bool var_33;
    wp::vec_t<2, wp::float32> var_34;
    wp::vec_t<2, wp::float32> var_35;
    wp::float32 var_36;
    bool var_37;
    const wp::float32 var_38 = -1.0;
    wp::vec_t<2, wp::float32> var_39;
    wp::vec_t<2, wp::float32> var_40;
    const wp::float32 var_41 = 0.5;
    wp::vec_t<2, wp::float32> var_42;
    wp::vec_t<2, wp::float32> var_43;
    wp::vec_t<2, wp::float32> var_44;
    wp::float32 var_45;
    wp::vec_t<2, wp::float32> var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    wp::float32 var_53;
    wp::float32 var_54;
    const wp::float32 var_55 = 2.0;
    wp::float32 var_56;
    wp::float32 var_57;
    wp::float32 var_58;
    const wp::float32 var_59 = -1.0;
    wp::float32 var_60;
    bool var_61;
    const wp::float32 var_62 = -1.0;
    const wp::float32 var_63 = 1.0;
    wp::float32 var_64;
    bool var_65;
    const wp::float32 var_66 = 0.0;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    wp::float32 var_73;
    const wp::float32 var_74 = 2.0;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::float32 var_79 = 0.0;
    bool var_80;
    const wp::float32 var_81 = 0.0;
    const wp::int32 var_82 = 0;
    wp::int32 var_83;
    bool var_84;
    bool var_85;
    wp::float32 var_86;
    bool var_87;
    wp::float32 var_88;
    const wp::float32 var_89 = 1.0;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    const wp::float32 var_95 = 1.0;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    wp::float32 var_99;
    wp::float32 var_100;
    wp::float32 var_101;
    const wp::float32 var_102 = 2.0;
    const wp::float32 var_103 = 1.0;
    wp::float32 var_104;
    wp::float32 var_105;
    wp::float32 var_106;
    wp::float32 var_107;
    wp::float32 var_108;
    const wp::float32 var_109 = -1e-15;
    bool var_110;
    const wp::float32 var_111 = 0.0;
    wp::float32 var_112;
    wp::float32 var_113;
    bool var_114;
    const wp::float32 var_115 = 0.0;
    wp::float32 var_116;
    wp::float32 var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    wp::float32 var_120;
    wp::float32 var_121;
    const wp::float32 var_122 = 2.0;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    wp::float32 var_126;
    bool var_127;
    const wp::float32 var_128 = 0.0;
    const wp::int32 var_129 = 1;
    wp::int32 var_130;
    bool var_131;
    const wp::float32 var_132 = 0.0;
    const wp::int32 var_133 = 0;
    wp::float32 var_134;
    const wp::int32 var_135 = 3;
    wp::float32 var_136;
    wp::float32 var_137;
    const wp::int32 var_138 = 1;
    wp::float32 var_139;
    const wp::int32 var_140 = 2;
    wp::float32 var_141;
    wp::float32 var_142;
    wp::float32 var_143;
    const wp::float32 var_144 = 0.0;
    bool var_145;
    wp::vec_t<2, wp::float32> var_146;
    wp::float32 var_147;
    wp::float32 var_148;
    wp::float32 var_149;
    wp::float32 var_150;
    wp::vec_t<2, wp::float32> var_151;
    wp::float32 var_152;
    wp::float32 var_153;
    wp::float32 var_154;
    wp::float32 var_155;
    wp::vec_t<2, wp::float32> var_156;
    wp::float32 var_157;
    wp::vec_t<2, wp::float32> var_158;
    wp::float32 var_159;
    wp::float32 var_160;
    const wp::int32 var_161 = 0;
    wp::float32 var_162;
    wp::float32 var_163;
    wp::float32 var_164;
    const wp::int32 var_165 = 1;
    wp::float32 var_166;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    wp::float32 var_170;
    const wp::int32 var_171 = 0;
    wp::float32 var_172;
    wp::float32 var_173;
    wp::float32 var_174;
    const wp::int32 var_175 = 1;
    wp::float32 var_176;
    wp::float32 var_177;
    wp::float32 var_178;
    wp::float32 var_179;
    wp::vec_t<2, wp::float32> var_180;
    const wp::float32 var_181 = 0.0;
    //---------
    // forward
    // def wrap_inside(                                                                       <L 203>
    // end0 = wp.vec2(end[0], end[1])                                                         <L 224>
    var_1 = wp::extract(var_end, var_0);
    var_3 = wp::extract(var_end, var_2);
    var_4 = wp::vec_t<2, wp::float32>(var_1, var_3);
    // end1 = wp.vec2(end[2], end[3])                                                         <L 225>
    var_6 = wp::extract(var_end, var_5);
    var_8 = wp::extract(var_end, var_7);
    var_9 = wp::vec_t<2, wp::float32>(var_6, var_8);
    // len0 = wp.norm_l2(end0)                                                                <L 228>
    var_10 = norm_l2_0(var_4);
    // len1 = wp.norm_l2(end1)                                                                <L 229>
    var_11 = norm_l2_0(var_9);
    // dif = end1 - end0                                                                      <L 230>
    var_12 = wp::sub(var_9, var_4);
    // dd = wp.dot(dif, dif)                                                                  <L 231>
    var_13 = wp::dot(var_12, var_12);
    // if (len0 <= radius) or (len1 <= radius) or (radius < MJ_MINVAL) or (len0 < MJ_MINVAL) or (len1 < MJ_MINVAL):       <L 234>
    var_15 = (var_10 <= var_radius);
    var_14 = var_15;
    if (!var_14) {
        var_16 = (var_11 <= var_radius);
        var_14 = var_14 || var_16;
    }
    if (!var_14) {
        var_18 = (var_radius < var_17);
        var_14 = var_14 || var_18;
    }
    if (!var_14) {
        var_19 = (var_10 < var_17);
        var_14 = var_14 || var_19;
    }
    if (!var_14) {
        var_20 = (var_11 < var_17);
        var_14 = var_14 || var_20;
    }
    if (var_14) {
        // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                                <L 235>
        var_23 = wp::vec_t<2, wp::float32>(var_22);
        var_24 = wp::vec_t<2, wp::float32>(var_22);
        ret_0 = var_21;
        ret_1 = var_23;
        ret_2 = var_24;
        return;
    }
    // if dd > MJ_MINVAL:                                                                     <L 238>
    var_25 = (var_13 > var_17);
    if (var_25) {
        // a = -wp.dot(dif, end0) / dd                                                        <L 240>
        var_26 = wp::dot(var_12, var_4);
        var_27 = wp::neg(var_26);
        var_28 = wp::div(var_27, var_13);
        // if (a > 0.0) and (a < 1.0):                                                        <L 243>
        var_31 = (var_28 > var_30);
        var_29 = var_31;
        if (var_29) {
            var_33 = (var_28 < var_32);
            var_29 = var_29 && var_33;
        }
        if (var_29) {
            // tmp = end0 + a * dif                                                           <L 244>
            var_34 = wp::mul(var_28, var_12);
            var_35 = wp::add(var_4, var_34);
            // if wp.norm_l2(tmp) <= radius:                                                  <L 245>
            var_36 = norm_l2_0(var_35);
            var_37 = (var_36 <= var_radius);
            if (var_37) {
                // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                        <L 246>
                var_39 = wp::vec_t<2, wp::float32>(var_22);
                var_40 = wp::vec_t<2, wp::float32>(var_22);
                ret_0 = var_38;
                ret_1 = var_39;
                ret_2 = var_40;
                return;
            }
        }
    }
    // pnt = 0.5 * (end0 + end1)                                                              <L 249>
    var_42 = wp::add(var_4, var_9);
    var_43 = wp::mul(var_41, var_42);
    // pnt, _ = math.normalize_with_norm(pnt)                                                 <L 250>
    normalize_with_norm_0(var_43, var_44, var_45);
    // pnt *= radius                                                                          <L 251>
    var_46 = wp::mul(var_44, var_radius);
    // A = math.safe_div(radius, len0)                                                        <L 254>
    var_47 = safe_div_0(var_radius, var_10);
    // B = math.safe_div(radius, len1)                                                        <L 255>
    var_48 = safe_div_0(var_radius, var_11);
    // sq_A = A * A                                                                           <L 256>
    var_49 = wp::mul(var_47, var_47);
    // sq_B = B * B                                                                           <L 257>
    var_50 = wp::mul(var_48, var_48);
    // cosG = math.safe_div(len0 * len0 + len1 * len1 - dd, 2.0 * len0 * len1)                <L 258>
    var_51 = wp::mul(var_10, var_10);
    var_52 = wp::mul(var_11, var_11);
    var_53 = wp::add(var_51, var_52);
    var_54 = wp::sub(var_53, var_13);
    var_56 = wp::mul(var_55, var_10);
    var_57 = wp::mul(var_56, var_11);
    var_58 = safe_div_0(var_54, var_57);
    // if cosG < -1.0 + MJ_MINVAL:                                                            <L 259>
    var_60 = wp::add(var_59, var_17);
    var_61 = (var_58 < var_60);
    if (var_61) {
        // return -1.0, pnt, pnt                                                              <L 260>
        ret_0 = var_62;
        ret_1 = var_46;
        ret_2 = var_46;
        return;
    }
    if (!var_61) {
        // elif cosG > 1.0 - MJ_MINVAL:                                                       <L 261>
        var_64 = wp::sub(var_63, var_17);
        var_65 = (var_58 > var_64);
        if (var_65) {
            // return 0.0, pnt, pnt                                                           <L 262>
            ret_0 = var_66;
            ret_1 = var_46;
            ret_2 = var_46;
            return;
        }
    }
    // G = wp.acos(cosG)                                                                      <L 263>
    var_67 = wp::acos(var_58);
    // z = zinit                                                                              <L 266>
    var_68 = wp::copy(var_zinit);
    // f = wp.asin(A * z) + wp.asin(B * z) - 2.0 * wp.asin(z) + G                             <L 267>
    var_69 = wp::mul(var_47, var_68);
    var_70 = wp::asin(var_69);
    var_71 = wp::mul(var_48, var_68);
    var_72 = wp::asin(var_71);
    var_73 = wp::add(var_70, var_72);
    var_75 = wp::asin(var_68);
    var_76 = wp::mul(var_74, var_75);
    var_77 = wp::sub(var_73, var_76);
    var_78 = wp::add(var_77, var_67);
    // if f > 0.0:                                                                            <L 270>
    var_80 = (var_78 > var_79);
    if (var_80) {
        // return 0.0, pnt, pnt                                                               <L 271>
        ret_0 = var_81;
        ret_1 = var_46;
        ret_2 = var_46;
        return;
    }
    // iter = int(0)                                                                          <L 274>
    var_83 = wp::int(var_82);
    // while (iter < maxiter) and (wp.abs(f) > tolerance):                                    <L 276>
    start_while_5:;
    var_85 = (var_83 < var_maxiter);
    var_84 = var_85;
    if (var_84) {
        var_86 = wp::abs(var_78);
        var_87 = (var_86 > var_tolerance);
        var_84 = var_84 && var_87;
    }
    if ((var_84) == false) goto end_while_5;
        // sq_z = z * z                                                                       <L 278>
        var_88 = wp::mul(var_68, var_68);
        // df = (                                                                             <L 279>
        // A / wp.max(MJ_MINVAL, wp.sqrt(1.0 - sq_z * sq_A))                                  <L 280>
        var_90 = wp::mul(var_88, var_49);
        var_91 = wp::sub(var_89, var_90);
        var_92 = wp::sqrt(var_91);
        var_93 = wp::max(var_17, var_92);
        var_94 = wp::div(var_47, var_93);
        // + B / wp.max(MJ_MINVAL, wp.sqrt(1.0 - sq_z * sq_B))                                <L 281>
        var_96 = wp::mul(var_88, var_50);
        var_97 = wp::sub(var_95, var_96);
        var_98 = wp::sqrt(var_97);
        var_99 = wp::max(var_17, var_98);
        var_100 = wp::div(var_48, var_99);
        var_101 = wp::add(var_94, var_100);
        // - 2.0 / wp.max(MJ_MINVAL, wp.sqrt(1.0 - sq_z))                                     <L 282>
        var_104 = wp::sub(var_103, var_88);
        var_105 = wp::sqrt(var_104);
        var_106 = wp::max(var_17, var_105);
        var_107 = wp::div(var_102, var_106);
        var_108 = wp::sub(var_101, var_107);
        // if df > -MJ_MINVAL:                                                                <L 286>
        var_110 = (var_108 > var_109);
        if (var_110) {
            // return 0.0, pnt, pnt                                                           <L 287>
            ret_0 = var_111;
            ret_1 = var_46;
            ret_2 = var_46;
            return;
        }
        // z1 = z - math.safe_div(f, df)                                                      <L 290>
        var_112 = safe_div_0(var_78, var_108);
        var_113 = wp::sub(var_68, var_112);
        // if z1 > z:                                                                         <L 293>
        var_114 = (var_113 > var_68);
        if (var_114) {
            // return 0.0, pnt, pnt                                                           <L 294>
            ret_0 = var_115;
            ret_1 = var_46;
            ret_2 = var_46;
            return;
        }
        // z = z1                                                                             <L 297>
        var_116 = wp::copy(var_113);
        // f = wp.asin(A * z) + wp.asin(B * z) - 2.0 * wp.asin(z) + G                         <L 298>
        var_117 = wp::mul(var_47, var_116);
        var_118 = wp::asin(var_117);
        var_119 = wp::mul(var_48, var_116);
        var_120 = wp::asin(var_119);
        var_121 = wp::add(var_118, var_120);
        var_123 = wp::asin(var_116);
        var_124 = wp::mul(var_122, var_123);
        var_125 = wp::sub(var_121, var_124);
        var_126 = wp::add(var_125, var_67);
        // if f > tolerance:                                                                  <L 301>
        var_127 = (var_126 > var_tolerance);
        if (var_127) {
            // return 0.0, pnt, pnt                                                           <L 302>
            ret_0 = var_128;
            ret_1 = var_46;
            ret_2 = var_46;
            return;
        }
        // iter += 1                                                                          <L 304>
        var_130 = wp::add(var_83, var_129);
        wp::assign(var_68, var_116);
        wp::assign(var_78, var_126);
        wp::assign(var_83, var_130);
    goto start_while_5;
    end_while_5:;
    // if iter >= maxiter:                                                                    <L 307>
    var_131 = (var_83 >= var_maxiter);
    if (var_131) {
        // return 0.0, pnt, pnt                                                               <L 308>
        ret_0 = var_132;
        ret_1 = var_46;
        ret_2 = var_46;
        return;
    }
    // if end[0] * end[3] - end[1] * end[2] > 0.0:                                            <L 311>
    var_134 = wp::extract(var_end, var_133);
    var_136 = wp::extract(var_end, var_135);
    var_137 = wp::mul(var_134, var_136);
    var_139 = wp::extract(var_end, var_138);
    var_141 = wp::extract(var_end, var_140);
    var_142 = wp::mul(var_139, var_141);
    var_143 = wp::sub(var_137, var_142);
    var_145 = (var_143 > var_144);
    if (var_145) {
        // vec = end0                                                                         <L 312>
        var_146 = wp::copy(var_4);
        // ang = wp.asin(z) - wp.asin(A * z)                                                  <L 313>
        var_147 = wp::asin(var_68);
        var_148 = wp::mul(var_47, var_68);
        var_149 = wp::asin(var_148);
        var_150 = wp::sub(var_147, var_149);
    }
    if (!var_145) {
        // vec = end1                                                                         <L 315>
        var_151 = wp::copy(var_9);
        // ang = wp.asin(z) - wp.asin(B * z)                                                  <L 316>
        var_152 = wp::asin(var_68);
        var_153 = wp::mul(var_48, var_68);
        var_154 = wp::asin(var_153);
        var_155 = wp::sub(var_152, var_154);
    }
    var_156 = wp::where(var_145, var_146, var_151);
    var_157 = wp::where(var_145, var_150, var_155);
    // vec, _ = math.normalize_with_norm(vec)                                                 <L 318>
    normalize_with_norm_0(var_156, var_158, var_159);
    // pnt = wp.vec2(                                                                         <L 319>
    // radius * (wp.cos(ang) * vec[0] - wp.sin(ang) * vec[1]),                                <L 320>
    var_160 = wp::cos(var_157);
    var_162 = wp::extract(var_158, var_161);
    var_163 = wp::mul(var_160, var_162);
    var_164 = wp::sin(var_157);
    var_166 = wp::extract(var_158, var_165);
    var_167 = wp::mul(var_164, var_166);
    var_168 = wp::sub(var_163, var_167);
    var_169 = wp::mul(var_radius, var_168);
    // radius * (wp.sin(ang) * vec[0] + wp.cos(ang) * vec[1]),                                <L 321>
    var_170 = wp::sin(var_157);
    var_172 = wp::extract(var_158, var_171);
    var_173 = wp::mul(var_170, var_172);
    var_174 = wp::cos(var_157);
    var_176 = wp::extract(var_158, var_175);
    var_177 = wp::mul(var_174, var_176);
    var_178 = wp::add(var_173, var_177);
    var_179 = wp::mul(var_radius, var_178);
    var_180 = wp::vec_t<2, wp::float32>(var_169, var_179);
    // return 0.0, pnt, pnt                                                                   <L 324>
    ret_0 = var_181;
    ret_1 = var_180;
    ret_2 = var_180;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:31
static CUDA_CALLABLE bool is_intersect_0(
    wp::vec_t<2, wp::float32> var_p1,
    wp::vec_t<2, wp::float32> var_p2,
    wp::vec_t<2, wp::float32> var_p3,
    wp::vec_t<2, wp::float32> var_p4)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    wp::float32 var_6;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 0;
    wp::float32 var_12;
    const wp::int32 var_13 = 0;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::float32 var_17;
    const wp::int32 var_18 = 1;
    wp::float32 var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    const wp::float32 var_24 = 1e-15;
    bool var_25;
    const bool var_26 = false;
    const wp::int32 var_27 = 0;
    wp::float32 var_28;
    const wp::int32 var_29 = 0;
    wp::float32 var_30;
    wp::float32 var_31;
    const wp::int32 var_32 = 1;
    wp::float32 var_33;
    const wp::int32 var_34 = 1;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::float32 var_37;
    const wp::int32 var_38 = 1;
    wp::float32 var_39;
    const wp::int32 var_40 = 1;
    wp::float32 var_41;
    wp::float32 var_42;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    const wp::int32 var_45 = 0;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::int32 var_51 = 0;
    wp::float32 var_52;
    const wp::int32 var_53 = 0;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::int32 var_56 = 1;
    wp::float32 var_57;
    const wp::int32 var_58 = 1;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::int32 var_62 = 1;
    wp::float32 var_63;
    const wp::int32 var_64 = 1;
    wp::float32 var_65;
    wp::float32 var_66;
    const wp::int32 var_67 = 0;
    wp::float32 var_68;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    wp::float32 var_73;
    wp::float32 var_74;
    bool var_75;
    const wp::int32 var_76 = 0;
    bool var_77;
    const wp::float32 var_78 = 1.0;
    bool var_79;
    const wp::float32 var_80 = 0.0;
    bool var_81;
    const wp::float32 var_82 = 1.0;
    bool var_83;
    const bool var_84 = true;
    const bool var_85 = false;
    //---------
    // forward
    // def is_intersect(p1: wp.vec2, p2: wp.vec2, p3: wp.vec2, p4: wp.vec2) -> bool:          <L 32>
    // det = (p4[1] - p3[1]) * (p2[0] - p1[0]) - (p4[0] - p3[0]) * (p2[1] - p1[1])            <L 45>
    var_1 = wp::extract(var_p4, var_0);
    var_3 = wp::extract(var_p3, var_2);
    var_4 = wp::sub(var_1, var_3);
    var_6 = wp::extract(var_p2, var_5);
    var_8 = wp::extract(var_p1, var_7);
    var_9 = wp::sub(var_6, var_8);
    var_10 = wp::mul(var_4, var_9);
    var_12 = wp::extract(var_p4, var_11);
    var_14 = wp::extract(var_p3, var_13);
    var_15 = wp::sub(var_12, var_14);
    var_17 = wp::extract(var_p2, var_16);
    var_19 = wp::extract(var_p1, var_18);
    var_20 = wp::sub(var_17, var_19);
    var_21 = wp::mul(var_15, var_20);
    var_22 = wp::sub(var_10, var_21);
    // if wp.abs(det) < MJ_MINVAL:                                                            <L 47>
    var_23 = wp::abs(var_22);
    var_25 = (var_23 < var_24);
    if (var_25) {
        // return False                                                                       <L 48>
        return var_26;
    }
    // a = ((p4[0] - p3[0]) * (p1[1] - p3[1]) - (p4[1] - p3[1]) * (p1[0] - p3[0])) / det       <L 51>
    var_28 = wp::extract(var_p4, var_27);
    var_30 = wp::extract(var_p3, var_29);
    var_31 = wp::sub(var_28, var_30);
    var_33 = wp::extract(var_p1, var_32);
    var_35 = wp::extract(var_p3, var_34);
    var_36 = wp::sub(var_33, var_35);
    var_37 = wp::mul(var_31, var_36);
    var_39 = wp::extract(var_p4, var_38);
    var_41 = wp::extract(var_p3, var_40);
    var_42 = wp::sub(var_39, var_41);
    var_44 = wp::extract(var_p1, var_43);
    var_46 = wp::extract(var_p3, var_45);
    var_47 = wp::sub(var_44, var_46);
    var_48 = wp::mul(var_42, var_47);
    var_49 = wp::sub(var_37, var_48);
    var_50 = wp::div(var_49, var_22);
    // b = ((p2[0] - p1[0]) * (p1[1] - p3[1]) - (p2[1] - p1[1]) * (p1[0] - p3[0])) / det       <L 52>
    var_52 = wp::extract(var_p2, var_51);
    var_54 = wp::extract(var_p1, var_53);
    var_55 = wp::sub(var_52, var_54);
    var_57 = wp::extract(var_p1, var_56);
    var_59 = wp::extract(var_p3, var_58);
    var_60 = wp::sub(var_57, var_59);
    var_61 = wp::mul(var_55, var_60);
    var_63 = wp::extract(var_p2, var_62);
    var_65 = wp::extract(var_p1, var_64);
    var_66 = wp::sub(var_63, var_65);
    var_68 = wp::extract(var_p1, var_67);
    var_70 = wp::extract(var_p3, var_69);
    var_71 = wp::sub(var_68, var_70);
    var_72 = wp::mul(var_66, var_71);
    var_73 = wp::sub(var_61, var_72);
    var_74 = wp::div(var_73, var_22);
    // if a >= 0 and a <= 1.0 and b >= 0.0 and b <= 1.0:                                      <L 54>
    var_77 = (var_50 >= var_76);
    var_75 = var_77;
    if (var_75) {
        var_79 = (var_50 <= var_78);
        var_75 = var_75 && var_79;
    }
    if (var_75) {
        var_81 = (var_74 >= var_80);
        var_75 = var_75 && var_81;
    }
    if (var_75) {
        var_83 = (var_74 <= var_82);
        var_75 = var_75 && var_83;
    }
    if (var_75) {
        // return True                                                                        <L 55>
        return var_84;
    }
    if (!var_75) {
        // return False                                                                       <L 57>
        return var_85;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:77
static CUDA_CALLABLE wp::float32 length_circle_0(
    wp::vec_t<2, wp::float32> var_p0,
    wp::vec_t<2, wp::float32> var_p1,
    wp::int32 var_ind,
    wp::float32 var_radius)
{
    //---------
    // primal vars
    wp::vec_t<2, wp::float32> var_0;
    wp::float32 var_1;
    wp::vec_t<2, wp::float32> var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 1;
    wp::float32 var_7;
    const wp::int32 var_8 = 0;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 0;
    wp::float32 var_12;
    const wp::int32 var_13 = 1;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    bool var_17;
    bool var_18;
    const wp::float32 var_19 = 0.0;
    bool var_20;
    const wp::int32 var_21 = 0;
    bool var_22;
    bool var_23;
    const wp::float32 var_24 = 0.0;
    bool var_25;
    const wp::int32 var_26 = 0;
    bool var_27;
    const wp::float32 var_28 = 2.0;
    const wp::float32 var_29 = 3.141592653589793;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    //---------
    // forward
    // def length_circle(p0: wp.vec2, p1: wp.vec2, ind: int, radius: float) -> float:         <L 78>
    // p0n, _ = math.normalize_with_norm(p0)                                                  <L 91>
    normalize_with_norm_0(var_p0, var_0, var_1);
    // p1n, _ = math.normalize_with_norm(p1)                                                  <L 92>
    normalize_with_norm_0(var_p1, var_2, var_3);
    // angle = wp.acos(wp.dot(p0n, p1n))                                                      <L 94>
    var_4 = wp::dot(var_0, var_2);
    var_5 = wp::acos(var_4);
    // cross = p0[1] * p1[0] - p0[0] * p1[1]                                                  <L 97>
    var_7 = wp::extract(var_p0, var_6);
    var_9 = wp::extract(var_p1, var_8);
    var_10 = wp::mul(var_7, var_9);
    var_12 = wp::extract(var_p0, var_11);
    var_14 = wp::extract(var_p1, var_13);
    var_15 = wp::mul(var_12, var_14);
    var_16 = wp::sub(var_10, var_15);
    // if (cross > 0.0 and ind != 0) or (cross < 0.0 and ind == 0):                           <L 98>
    var_20 = (var_16 > var_19);
    var_18 = var_20;
    if (var_18) {
        var_22 = (var_ind != var_21);
        var_18 = var_18 && var_22;
    }
    var_17 = var_18;
    if (!var_17) {
        var_25 = (var_16 < var_24);
        var_23 = var_25;
        if (var_23) {
            var_27 = (var_ind == var_26);
            var_23 = var_23 && var_27;
        }
        var_17 = var_17 || var_23;
    }
    if (var_17) {
        // angle = 2.0 * wp.pi - angle                                                        <L 99>
        var_30 = wp::mul(var_28, var_29);
        var_31 = wp::sub(var_30, var_5);
    }
    var_32 = wp::where(var_17, var_31, var_5);
    // return radius * angle                                                                  <L 101>
    var_33 = wp::mul(var_radius, var_32);
    return var_33;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:104
static CUDA_CALLABLE void wrap_circle_0(
    wp::vec_t<4, wp::float32> var_end,
    wp::vec_t<2, wp::float32> var_side,
    wp::float32 var_radius,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::float32 var_1 = 10000000000.0;
    bool var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 1;
    wp::float32 var_6;
    wp::vec_t<2, wp::float32> var_7;
    const wp::int32 var_8 = 2;
    wp::float32 var_9;
    const wp::int32 var_10 = 3;
    wp::float32 var_11;
    wp::vec_t<2, wp::float32> var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    bool var_16;
    bool var_17;
    bool var_18;
    const wp::float32 var_19 = 1e-15;
    bool var_20;
    const wp::float32 var_21 = -1.0;
    wp::vec_t<2, wp::float32> var_22;
    wp::vec_t<2, wp::float32> var_23;
    wp::vec_t<2, wp::float32> var_24;
    wp::float32 var_25;
    bool var_26;
    const wp::float32 var_27 = -1.0;
    wp::vec_t<2, wp::float32> var_28;
    wp::vec_t<2, wp::float32> var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::float32 var_33 = 0.0;
    const wp::float32 var_34 = 1.0;
    wp::float32 var_35;
    wp::vec_t<2, wp::float32> var_36;
    wp::vec_t<2, wp::float32> var_37;
    bool var_38;
    wp::float32 var_39;
    bool var_40;
    bool var_41;
    bool var_42;
    wp::float32 var_43;
    const wp::float32 var_44 = 0.0;
    bool var_45;
    const wp::float32 var_46 = -1.0;
    wp::vec_t<2, wp::float32> var_47;
    wp::vec_t<2, wp::float32> var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::int32 var_53 = 0;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::int32 var_56 = 1;
    wp::float32 var_57;
    wp::float32 var_58;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::int32 var_62 = 1;
    wp::float32 var_63;
    wp::float32 var_64;
    const wp::int32 var_65 = 0;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::vec_t<2, wp::float32> var_71;
    const wp::int32 var_72 = 2;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::int32 var_75 = 3;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::float32 var_80;
    const wp::int32 var_81 = 3;
    wp::float32 var_82;
    wp::float32 var_83;
    const wp::int32 var_84 = 2;
    wp::float32 var_85;
    wp::float32 var_86;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    wp::vec_t<2, wp::float32> var_90;
    const wp::int32 var_91 = 0;
    wp::float32 var_92;
    wp::float32 var_93;
    const wp::int32 var_94 = 1;
    wp::float32 var_95;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    wp::float32 var_99;
    const wp::int32 var_100 = 1;
    wp::float32 var_101;
    wp::float32 var_102;
    const wp::int32 var_103 = 0;
    wp::float32 var_104;
    wp::float32 var_105;
    wp::float32 var_106;
    wp::float32 var_107;
    wp::float32 var_108;
    wp::vec_t<2, wp::float32> var_109;
    const wp::int32 var_110 = 2;
    wp::float32 var_111;
    wp::float32 var_112;
    const wp::int32 var_113 = 3;
    wp::float32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    wp::float32 var_118;
    const wp::int32 var_119 = 3;
    wp::float32 var_120;
    wp::float32 var_121;
    const wp::int32 var_122 = 2;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    wp::float32 var_126;
    wp::float32 var_127;
    wp::vec_t<2, wp::float32> var_128;
    wp::vec_t<2, wp::float32> var_129;
    wp::vec_t<2, wp::float32> var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    wp::vec_t<2, wp::float32> var_133;
    wp::vec_t<2, wp::float32> var_134;
    wp::float32 var_135;
    wp::float32 var_136;
    wp::vec_t<2, wp::float32> var_137;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::vec_t<2, wp::float32> var_140;
    wp::float32 var_141;
    wp::float32 var_142;
    wp::vec_t<2, wp::float32> var_143;
    wp::float32 var_144;
    wp::vec_t<2, wp::float32> var_145;
    wp::float32 var_146;
    bool var_147;
    const wp::float32 var_148 = -10000.0;
    wp::float32 var_149;
    bool var_150;
    const wp::float32 var_151 = -10000.0;
    wp::float32 var_152;
    bool var_153;
    wp::vec_t<2, wp::float32> var_154;
    wp::vec_t<2, wp::float32> var_155;
    const wp::int32 var_156 = 0;
    wp::vec_t<2, wp::float32> var_157;
    wp::vec_t<2, wp::float32> var_158;
    const wp::int32 var_159 = 1;
    wp::vec_t<2, wp::float32> var_160;
    wp::vec_t<2, wp::float32> var_161;
    wp::int32 var_162;
    bool var_163;
    const wp::float32 var_164 = -1.0;
    wp::vec_t<2, wp::float32> var_165;
    wp::vec_t<2, wp::float32> var_166;
    wp::float32 var_167;
    //---------
    // forward
    // def wrap_circle(end: wp.vec4, side: wp.vec2, radius: float) -> Tuple[float, wp.vec2, wp.vec2]:       <L 105>
    // valid_side = wp.norm_l2(side) < MJ_MAXVAL                                              <L 116>
    var_0 = norm_l2_0(var_side);
    var_2 = (var_0 < var_1);
    // end0 = wp.vec2(end[0], end[1])                                                         <L 118>
    var_4 = wp::extract(var_end, var_3);
    var_6 = wp::extract(var_end, var_5);
    var_7 = wp::vec_t<2, wp::float32>(var_4, var_6);
    // end1 = wp.vec2(end[2], end[3])                                                         <L 119>
    var_9 = wp::extract(var_end, var_8);
    var_11 = wp::extract(var_end, var_10);
    var_12 = wp::vec_t<2, wp::float32>(var_9, var_11);
    // sqlen0 = wp.dot(end0, end0)                                                            <L 121>
    var_13 = wp::dot(var_7, var_7);
    // sqlen1 = wp.dot(end1, end1)                                                            <L 122>
    var_14 = wp::dot(var_12, var_12);
    // sqrad = radius * radius                                                                <L 123>
    var_15 = wp::mul(var_radius, var_radius);
    // if (sqlen0 < sqrad) or (sqlen1 < sqrad) or (radius < MJ_MINVAL):                       <L 126>
    var_17 = (var_13 < var_15);
    var_16 = var_17;
    if (!var_16) {
        var_18 = (var_14 < var_15);
        var_16 = var_16 || var_18;
    }
    if (!var_16) {
        var_20 = (var_radius < var_19);
        var_16 = var_16 || var_20;
    }
    if (var_16) {
        // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                                <L 127>
        var_22 = wp::vec_t<2, wp::float32>(var_1);
        var_23 = wp::vec_t<2, wp::float32>(var_1);
        ret_0 = var_21;
        ret_1 = var_22;
        ret_2 = var_23;
        return;
    }
    // dif = end1 - end0                                                                      <L 130>
    var_24 = wp::sub(var_12, var_7);
    // dd = wp.dot(dif, dif)                                                                  <L 131>
    var_25 = wp::dot(var_24, var_24);
    // if dd < MJ_MINVAL:                                                                     <L 132>
    var_26 = (var_25 < var_19);
    if (var_26) {
        // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                                <L 133>
        var_28 = wp::vec_t<2, wp::float32>(var_1);
        var_29 = wp::vec_t<2, wp::float32>(var_1);
        ret_0 = var_27;
        ret_1 = var_28;
        ret_2 = var_29;
        return;
    }
    // a = -wp.dot(dif, end0) / dd                                                            <L 136>
    var_30 = wp::dot(var_24, var_7);
    var_31 = wp::neg(var_30);
    var_32 = wp::div(var_31, var_25);
    // a = wp.clamp(a, 0.0, 1.0)                                                              <L 137>
    var_35 = wp::clamp(var_32, var_33, var_34);
    // tmp = a * dif + end0                                                                   <L 140>
    var_36 = wp::mul(var_35, var_24);
    var_37 = wp::add(var_36, var_7);
    // if (wp.dot(tmp, tmp) > sqrad) and (not valid_side or wp.dot(side, tmp) >= 0.0):        <L 141>
    var_39 = wp::dot(var_37, var_37);
    var_40 = (var_39 > var_15);
    var_38 = var_40;
    if (var_38) {
        var_42 = wp::unot(var_2);
        var_41 = var_42;
        if (!var_41) {
            var_43 = wp::dot(var_side, var_37);
            var_45 = (var_43 >= var_44);
            var_41 = var_41 || var_45;
        }
        var_38 = var_38 && var_41;
    }
    if (var_38) {
        // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                                <L 142>
        var_47 = wp::vec_t<2, wp::float32>(var_1);
        var_48 = wp::vec_t<2, wp::float32>(var_1);
        ret_0 = var_46;
        ret_1 = var_47;
        ret_2 = var_48;
        return;
    }
    // sqrt0 = wp.sqrt(sqlen0 - sqrad)                                                        <L 144>
    var_49 = wp::sub(var_13, var_15);
    var_50 = wp::sqrt(var_49);
    // sqrt1 = wp.sqrt(sqlen1 - sqrad)                                                        <L 145>
    var_51 = wp::sub(var_14, var_15);
    var_52 = wp::sqrt(var_51);
    // sol00 = wp.vec2(                                                                       <L 148>
    // math.safe_div(end[0] * sqrad + radius * end[1] * sqrt0, sqlen0),                       <L 149>
    var_54 = wp::extract(var_end, var_53);
    var_55 = wp::mul(var_54, var_15);
    var_57 = wp::extract(var_end, var_56);
    var_58 = wp::mul(var_radius, var_57);
    var_59 = wp::mul(var_58, var_50);
    var_60 = wp::add(var_55, var_59);
    var_61 = safe_div_0(var_60, var_13);
    // math.safe_div(end[1] * sqrad - radius * end[0] * sqrt0, sqlen0),                       <L 150>
    var_63 = wp::extract(var_end, var_62);
    var_64 = wp::mul(var_63, var_15);
    var_66 = wp::extract(var_end, var_65);
    var_67 = wp::mul(var_radius, var_66);
    var_68 = wp::mul(var_67, var_50);
    var_69 = wp::sub(var_64, var_68);
    var_70 = safe_div_0(var_69, var_13);
    var_71 = wp::vec_t<2, wp::float32>(var_61, var_70);
    // sol01 = wp.vec2(                                                                       <L 152>
    // math.safe_div(end[2] * sqrad - radius * end[3] * sqrt1, sqlen1),                       <L 153>
    var_73 = wp::extract(var_end, var_72);
    var_74 = wp::mul(var_73, var_15);
    var_76 = wp::extract(var_end, var_75);
    var_77 = wp::mul(var_radius, var_76);
    var_78 = wp::mul(var_77, var_52);
    var_79 = wp::sub(var_74, var_78);
    var_80 = safe_div_0(var_79, var_14);
    // math.safe_div(end[3] * sqrad + radius * end[2] * sqrt1, sqlen1),                       <L 154>
    var_82 = wp::extract(var_end, var_81);
    var_83 = wp::mul(var_82, var_15);
    var_85 = wp::extract(var_end, var_84);
    var_86 = wp::mul(var_radius, var_85);
    var_87 = wp::mul(var_86, var_52);
    var_88 = wp::add(var_83, var_87);
    var_89 = safe_div_0(var_88, var_14);
    var_90 = wp::vec_t<2, wp::float32>(var_80, var_89);
    // sol10 = wp.vec2(                                                                       <L 157>
    // math.safe_div(end[0] * sqrad - radius * end[1] * sqrt0, sqlen0),                       <L 158>
    var_92 = wp::extract(var_end, var_91);
    var_93 = wp::mul(var_92, var_15);
    var_95 = wp::extract(var_end, var_94);
    var_96 = wp::mul(var_radius, var_95);
    var_97 = wp::mul(var_96, var_50);
    var_98 = wp::sub(var_93, var_97);
    var_99 = safe_div_0(var_98, var_13);
    // math.safe_div(end[1] * sqrad + radius * end[0] * sqrt0, sqlen0),                       <L 159>
    var_101 = wp::extract(var_end, var_100);
    var_102 = wp::mul(var_101, var_15);
    var_104 = wp::extract(var_end, var_103);
    var_105 = wp::mul(var_radius, var_104);
    var_106 = wp::mul(var_105, var_50);
    var_107 = wp::add(var_102, var_106);
    var_108 = safe_div_0(var_107, var_13);
    var_109 = wp::vec_t<2, wp::float32>(var_99, var_108);
    // sol11 = wp.vec2(                                                                       <L 161>
    // math.safe_div(end[2] * sqrad + radius * end[3] * sqrt1, sqlen1),                       <L 162>
    var_111 = wp::extract(var_end, var_110);
    var_112 = wp::mul(var_111, var_15);
    var_114 = wp::extract(var_end, var_113);
    var_115 = wp::mul(var_radius, var_114);
    var_116 = wp::mul(var_115, var_52);
    var_117 = wp::add(var_112, var_116);
    var_118 = safe_div_0(var_117, var_14);
    // math.safe_div(end[3] * sqrad - radius * end[2] * sqrt1, sqlen1),                       <L 163>
    var_120 = wp::extract(var_end, var_119);
    var_121 = wp::mul(var_120, var_15);
    var_123 = wp::extract(var_end, var_122);
    var_124 = wp::mul(var_radius, var_123);
    var_125 = wp::mul(var_124, var_52);
    var_126 = wp::sub(var_121, var_125);
    var_127 = safe_div_0(var_126, var_14);
    var_128 = wp::vec_t<2, wp::float32>(var_118, var_127);
    // if valid_side:                                                                         <L 167>
    if (var_2) {
        // tmp0, _ = math.normalize_with_norm(sol00 + sol01)                                  <L 168>
        var_129 = wp::add(var_71, var_90);
        normalize_with_norm_0(var_129, var_130, var_131);
        // good0 = wp.dot(tmp0, side)                                                         <L 169>
        var_132 = wp::dot(var_130, var_side);
        // tmp1, _ = math.normalize_with_norm(sol10 + sol11)                                  <L 170>
        var_133 = wp::add(var_109, var_128);
        normalize_with_norm_0(var_133, var_134, var_135);
        // good1 = wp.dot(tmp1, side)                                                         <L 171>
        var_136 = wp::dot(var_134, var_side);
    }
    if (!var_2) {
        // tmp0 = sol00 - sol01                                                               <L 173>
        var_137 = wp::sub(var_71, var_90);
        // good0 = -wp.dot(tmp0, tmp0)                                                        <L 174>
        var_138 = wp::dot(var_137, var_137);
        var_139 = wp::neg(var_138);
        // tmp1 = sol10 - sol11                                                               <L 175>
        var_140 = wp::sub(var_109, var_128);
        // good1 = -wp.dot(tmp1, tmp1)                                                        <L 176>
        var_141 = wp::dot(var_140, var_140);
        var_142 = wp::neg(var_141);
    }
    var_143 = wp::where(var_2, var_130, var_137);
    var_144 = wp::where(var_2, var_132, var_139);
    var_145 = wp::where(var_2, var_134, var_140);
    var_146 = wp::where(var_2, var_136, var_142);
    // if is_intersect(end0, sol00, end1, sol01):                                             <L 179>
    var_147 = is_intersect_0(var_7, var_71, var_12, var_90);
    if (var_147) {
        // good0 = -10000.0                                                                   <L 180>
    }
    var_149 = wp::where(var_147, var_148, var_144);
    // if is_intersect(end0, sol10, end1, sol11):                                             <L 181>
    var_150 = is_intersect_0(var_7, var_109, var_12, var_128);
    if (var_150) {
        // good1 = -10000.0                                                                   <L 182>
    }
    var_152 = wp::where(var_150, var_151, var_146);
    // if good0 > good1:                                                                      <L 185>
    var_153 = (var_149 > var_152);
    if (var_153) {
        // pnt0 = sol00                                                                       <L 186>
        var_154 = wp::copy(var_71);
        // pnt1 = sol01                                                                       <L 187>
        var_155 = wp::copy(var_90);
        // ind = 0                                                                            <L 188>
    }
    if (!var_153) {
        // pnt0 = sol10                                                                       <L 190>
        var_157 = wp::copy(var_109);
        // pnt1 = sol11                                                                       <L 191>
        var_158 = wp::copy(var_128);
        // ind = 1                                                                            <L 192>
    }
    var_160 = wp::where(var_153, var_154, var_157);
    var_161 = wp::where(var_153, var_155, var_158);
    var_162 = wp::where(var_153, var_156, var_159);
    // if is_intersect(end0, pnt0, end1, pnt1):                                               <L 195>
    var_163 = is_intersect_0(var_7, var_160, var_12, var_161);
    if (var_163) {
        // return -1.0, wp.vec2(MJ_MAXVAL), wp.vec2(MJ_MAXVAL)                                <L 196>
        var_165 = wp::vec_t<2, wp::float32>(var_1);
        var_166 = wp::vec_t<2, wp::float32>(var_1);
        ret_0 = var_164;
        ret_1 = var_165;
        ret_2 = var_166;
        return;
    }
    // return length_circle(pnt0, pnt1, ind, radius), pnt0, pnt1                              <L 199>
    var_167 = length_circle_0(var_160, var_161, var_162, var_radius);
    ret_0 = var_167;
    ret_1 = var_160;
    ret_2 = var_161;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:327
static CUDA_CALLABLE void wrap_0(
    wp::vec_t<3, wp::float32> var_x0,
    wp::vec_t<3, wp::float32> var_x1,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::float32 var_radius,
    wp::int32 var_geomtype,
    wp::vec_t<3, wp::float32> var_side,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    bool var_0;
    const wp::int32 var_1 = 4;
    bool var_2;
    const wp::int32 var_3 = 5;
    bool var_4;
    const wp::float32 var_5 = 10000000000.0;
    wp::vec_t<3, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::mat_t<3, 3, wp::float32> var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    bool var_13;
    wp::float32 var_14;
    const wp::float32 var_15 = 1e-15;
    bool var_16;
    wp::float32 var_17;
    bool var_18;
    const wp::float32 var_19 = -1.0;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    const wp::int32 var_22 = 4;
    bool var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::float32 var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::float32 var_28;
    bool var_29;
    wp::vec_t<3, wp::float32> var_30;
    const wp::int32 var_31 = 0;
    wp::int32 var_32;
    bool var_33;
    const wp::int32 var_34 = 1;
    wp::float32 var_35;
    const wp::int32 var_36 = 0;
    wp::float32 var_37;
    bool var_38;
    const wp::int32 var_39 = 1;
    wp::float32 var_40;
    const wp::int32 var_41 = 2;
    wp::float32 var_42;
    bool var_43;
    const wp::int32 var_44 = 1;
    wp::int32 var_45;
    bool var_46;
    const wp::int32 var_47 = 2;
    wp::float32 var_48;
    const wp::int32 var_49 = 0;
    wp::float32 var_50;
    bool var_51;
    const wp::int32 var_52 = 2;
    wp::float32 var_53;
    const wp::int32 var_54 = 1;
    wp::float32 var_55;
    bool var_56;
    const wp::int32 var_57 = 2;
    wp::int32 var_58;
    const wp::float32 var_59 = 1.0;
    wp::vec_t<3, wp::float32> var_60;
    const wp::float32 var_61 = 0.0;
    wp::vec_t<3, wp::float32> var_62;
    wp::vec_t<3, wp::float32> var_63;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::vec_t<3, wp::float32> var_66;
    wp::vec_t<3, wp::float32> var_67;
    wp::vec_t<3, wp::float32> var_68;
    wp::float32 var_69;
    const wp::float32 var_70 = 1.0;
    const wp::float32 var_71 = 0.0;
    const wp::float32 var_72 = 0.0;
    wp::vec_t<3, wp::float32> var_73;
    const wp::float32 var_74 = 0.0;
    const wp::float32 var_75 = 1.0;
    const wp::float32 var_76 = 0.0;
    wp::vec_t<3, wp::float32> var_77;
    wp::vec_t<3, wp::float32> var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::float32 var_80;
    wp::float32 var_81;
    wp::float32 var_82;
    wp::float32 var_83;
    wp::vec_t<4, wp::float32> var_84;
    wp::float32 var_85;
    bool var_86;
    wp::vec_t<3, wp::float32> var_87;
    wp::vec_t<3, wp::float32> var_88;
    wp::float32 var_89;
    wp::float32 var_90;
    wp::vec_t<2, wp::float32> var_91;
    wp::vec_t<2, wp::float32> var_92;
    wp::float32 var_93;
    wp::vec_t<2, wp::float32> var_94;
    wp::float32 var_95;
    wp::vec_t<2, wp::float32> var_96;
    wp::vec_t<2, wp::float32> var_97;
    bool var_98;
    wp::float32 var_99;
    bool var_100;
    wp::float32 var_101;
    wp::vec_t<2, wp::float32> var_102;
    wp::vec_t<2, wp::float32> var_103;
    const wp::int32 var_104 = 20;
    const wp::float32 var_105 = 0.9999999;
    const wp::float32 var_106 = 1e-06;
    wp::float32 var_107;
    wp::vec_t<2, wp::float32> var_108;
    wp::vec_t<2, wp::float32> var_109;
    wp::float32 var_110;
    wp::vec_t<2, wp::float32> var_111;
    wp::vec_t<2, wp::float32> var_112;
    const wp::float32 var_113 = 0.0;
    bool var_114;
    const wp::float32 var_115 = -1.0;
    wp::vec_t<3, wp::float32> var_116;
    wp::vec_t<3, wp::float32> var_117;
    const wp::int32 var_118 = 0;
    wp::float32 var_119;
    wp::vec_t<3, wp::float32> var_120;
    const wp::int32 var_121 = 1;
    wp::float32 var_122;
    wp::vec_t<3, wp::float32> var_123;
    wp::vec_t<3, wp::float32> var_124;
    const wp::int32 var_125 = 0;
    wp::float32 var_126;
    wp::vec_t<3, wp::float32> var_127;
    const wp::int32 var_128 = 1;
    wp::float32 var_129;
    wp::vec_t<3, wp::float32> var_130;
    wp::vec_t<3, wp::float32> var_131;
    const wp::int32 var_132 = 5;
    bool var_133;
    const wp::int32 var_134 = 0;
    wp::float32 var_135;
    const wp::int32 var_136 = 0;
    wp::float32 var_137;
    wp::float32 var_138;
    const wp::int32 var_139 = 0;
    wp::float32 var_140;
    const wp::int32 var_141 = 0;
    wp::float32 var_142;
    wp::float32 var_143;
    wp::float32 var_144;
    const wp::int32 var_145 = 1;
    wp::float32 var_146;
    const wp::int32 var_147 = 1;
    wp::float32 var_148;
    wp::float32 var_149;
    const wp::int32 var_150 = 1;
    wp::float32 var_151;
    const wp::int32 var_152 = 1;
    wp::float32 var_153;
    wp::float32 var_154;
    wp::float32 var_155;
    wp::float32 var_156;
    wp::float32 var_157;
    const wp::int32 var_158 = 0;
    wp::float32 var_159;
    const wp::int32 var_160 = 0;
    wp::float32 var_161;
    wp::float32 var_162;
    const wp::int32 var_163 = 0;
    wp::float32 var_164;
    const wp::int32 var_165 = 0;
    wp::float32 var_166;
    wp::float32 var_167;
    wp::float32 var_168;
    const wp::int32 var_169 = 1;
    wp::float32 var_170;
    const wp::int32 var_171 = 1;
    wp::float32 var_172;
    wp::float32 var_173;
    const wp::int32 var_174 = 1;
    wp::float32 var_175;
    const wp::int32 var_176 = 1;
    wp::float32 var_177;
    wp::float32 var_178;
    wp::float32 var_179;
    wp::float32 var_180;
    wp::float32 var_181;
    const wp::int32 var_182 = 2;
    wp::float32 var_183;
    const wp::int32 var_184 = 2;
    wp::float32 var_185;
    const wp::int32 var_186 = 2;
    wp::float32 var_187;
    wp::float32 var_188;
    wp::float32 var_189;
    wp::float32 var_190;
    wp::float32 var_191;
    wp::float32 var_192;
    wp::float32 var_193;
    const wp::int32 var_194 = 2;
    const wp::int32 var_195 = 2;
    wp::float32 var_196;
    const wp::int32 var_197 = 2;
    wp::float32 var_198;
    const wp::int32 var_199 = 2;
    wp::float32 var_200;
    wp::float32 var_201;
    wp::float32 var_202;
    wp::float32 var_203;
    wp::float32 var_204;
    wp::float32 var_205;
    wp::float32 var_206;
    wp::float32 var_207;
    const wp::int32 var_208 = 2;
    const wp::int32 var_209 = 2;
    wp::float32 var_210;
    const wp::int32 var_211 = 2;
    wp::float32 var_212;
    wp::float32 var_213;
    wp::float32 var_214;
    wp::float32 var_215;
    wp::float32 var_216;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    wp::vec_t<3, wp::float32> var_220;
    wp::vec_t<3, wp::float32> var_221;
    wp::vec_t<3, wp::float32> var_222;
    wp::vec_t<3, wp::float32> var_223;
    //---------
    // forward
    // def wrap(                                                                              <L 328>
    // if geomtype != WrapType.SPHERE and geomtype != WrapType.CYLINDER:                      <L 346>
    var_2 = (var_geomtype != var_1);
    var_0 = var_2;
    if (var_0) {
        var_4 = (var_geomtype != var_3);
        var_0 = var_0 && var_4;
    }
    if (var_0) {
        // return MJ_MAXVAL, wp.vec3(MJ_MAXVAL), wp.vec3(MJ_MAXVAL)                           <L 347>
        var_6 = wp::vec_t<3, wp::float32>(var_5);
        var_7 = wp::vec_t<3, wp::float32>(var_5);
        ret_0 = var_5;
        ret_1 = var_6;
        ret_2 = var_7;
        return;
    }
    // matT = wp.transpose(mat)                                                               <L 350>
    var_8 = wp::transpose(var_mat);
    // p0 = matT @ (x0 - pos)                                                                 <L 351>
    var_9 = wp::sub(var_x0, var_pos);
    var_10 = wp::mul(var_8, var_9);
    // p1 = matT @ (x1 - pos)                                                                 <L 352>
    var_11 = wp::sub(var_x1, var_pos);
    var_12 = wp::mul(var_8, var_11);
    // if (wp.norm_l2(p0) < MJ_MINVAL) or (wp.norm_l2(p1) < MJ_MINVAL):                       <L 355>
    var_14 = norm_l2_0(var_10);
    var_16 = (var_14 < var_15);
    var_13 = var_16;
    if (!var_13) {
        var_17 = norm_l2_0(var_12);
        var_18 = (var_17 < var_15);
        var_13 = var_13 || var_18;
    }
    if (var_13) {
        // return -1.0, wp.vec3(MJ_MAXVAL), wp.vec3(MJ_MAXVAL)                                <L 356>
        var_20 = wp::vec_t<3, wp::float32>(var_5);
        var_21 = wp::vec_t<3, wp::float32>(var_5);
        ret_0 = var_19;
        ret_1 = var_20;
        ret_2 = var_21;
        return;
    }
    // if geomtype == WrapType.SPHERE:                                                        <L 359>
    var_23 = (var_geomtype == var_22);
    if (var_23) {
        // axis0, _ = math.normalize_with_norm(p0)                                            <L 361>
        normalize_with_norm_0(var_10, var_24, var_25);
        // normal = wp.cross(p0, p1)                                                          <L 364>
        var_26 = wp::cross(var_10, var_12);
        // normal, nrm = math.normalize_with_norm(normal)                                     <L 365>
        normalize_with_norm_0(var_26, var_27, var_28);
        // if nrm < MJ_MINVAL:                                                                <L 368>
        var_29 = (var_28 < var_15);
        if (var_29) {
            // axis0_abs = wp.abs(axis0)                                                      <L 370>
            var_30 = wp::abs(var_24);
            // i = int(0)                                                                     <L 371>
            var_32 = wp::int(var_31);
            // if (axis0_abs[1] > axis0_abs[0]) and (axis0_abs[1] > axis0_abs[2]):            <L 372>
            var_35 = wp::extract(var_30, var_34);
            var_37 = wp::extract(var_30, var_36);
            var_38 = (var_35 > var_37);
            var_33 = var_38;
            if (var_33) {
                var_40 = wp::extract(var_30, var_39);
                var_42 = wp::extract(var_30, var_41);
                var_43 = (var_40 > var_42);
                var_33 = var_33 && var_43;
            }
            if (var_33) {
                // i = 1                                                                      <L 373>
            }
            var_45 = wp::where(var_33, var_44, var_32);
            // if (axis0_abs[2] > axis0_abs[0]) and (axis0_abs[2] > axis0_abs[1]):            <L 374>
            var_48 = wp::extract(var_30, var_47);
            var_50 = wp::extract(var_30, var_49);
            var_51 = (var_48 > var_50);
            var_46 = var_51;
            if (var_46) {
                var_53 = wp::extract(var_30, var_52);
                var_55 = wp::extract(var_30, var_54);
                var_56 = (var_53 > var_55);
                var_46 = var_46 && var_56;
            }
            if (var_46) {
                // i = 2                                                                      <L 375>
            }
            var_58 = wp::where(var_46, var_57, var_45);
            // axis1 = wp.vec3(1.0)                                                           <L 378>
            var_60 = wp::vec_t<3, wp::float32>(var_59);
            // axis1[i] = 0.0                                                                 <L 379>
            wp::assign_inplace(var_60, var_58, var_61);
            // normal = wp.cross(axis0, axis1)                                                <L 382>
            var_62 = wp::cross(var_24, var_60);
            // normal, _ = math.normalize_with_norm(normal)                                   <L 383>
            normalize_with_norm_0(var_62, var_63, var_64);
        }
        var_65 = wp::where(var_29, var_64, var_25);
        var_66 = wp::where(var_29, var_63, var_27);
        // axis1 = wp.cross(normal, axis0)                                                    <L 386>
        var_67 = wp::cross(var_66, var_24);
        // axis1, _ = math.normalize_with_norm(axis1)                                         <L 387>
        normalize_with_norm_0(var_67, var_68, var_69);
    }
    if (!var_23) {
        // axis0 = wp.vec3(1.0, 0.0, 0.0)                                                     <L 390>
        var_73 = wp::vec_t<3, wp::float32>(var_70, var_71, var_72);
        // axis1 = wp.vec3(0.0, 1.0, 0.0)                                                     <L 393>
        var_77 = wp::vec_t<3, wp::float32>(var_74, var_75, var_76);
    }
    var_78 = wp::where(var_23, var_24, var_73);
    var_79 = wp::where(var_23, var_68, var_77);
    // end = wp.vec4(                                                                         <L 396>
    // wp.dot(p0, axis0),                                                                     <L 397>
    var_80 = wp::dot(var_10, var_78);
    // wp.dot(p0, axis1),                                                                     <L 398>
    var_81 = wp::dot(var_10, var_79);
    // wp.dot(p1, axis0),                                                                     <L 399>
    var_82 = wp::dot(var_12, var_78);
    // wp.dot(p1, axis1),                                                                     <L 400>
    var_83 = wp::dot(var_12, var_79);
    var_84 = wp::vec_t<4, wp::float32>(var_80, var_81, var_82, var_83);
    // valid_side = wp.norm_l2(side) < MJ_MAXVAL                                              <L 404>
    var_85 = norm_l2_0(var_side);
    var_86 = (var_85 < var_5);
    // if valid_side:                                                                         <L 406>
    if (var_86) {
        // sidepnt = matT @ (side - pos)                                                      <L 408>
        var_87 = wp::sub(var_side, var_pos);
        var_88 = wp::mul(var_8, var_87);
        // sidepnt_proj = wp.vec2(                                                            <L 411>
        // wp.dot(sidepnt, axis0),                                                            <L 412>
        var_89 = wp::dot(var_88, var_78);
        // wp.dot(sidepnt, axis1),                                                            <L 413>
        var_90 = wp::dot(var_88, var_79);
        var_91 = wp::vec_t<2, wp::float32>(var_89, var_90);
        // sidepnt_proj, _ = math.normalize_with_norm(sidepnt_proj)                           <L 416>
        normalize_with_norm_0(var_91, var_92, var_93);
        // sidepnt_proj *= radius                                                             <L 417>
        var_94 = wp::mul(var_92, var_radius);
    }
    var_95 = wp::where(var_86, var_93, var_69);
    if (!var_86) {
        // sidepnt_proj = wp.vec2(MJ_MAXVAL)                                                  <L 419>
        var_96 = wp::vec_t<2, wp::float32>(var_5);
    }
    var_97 = wp::where(var_86, var_94, var_96);
    // if valid_side and wp.norm_l2(sidepnt) < radius:                                        <L 422>
    var_98 = var_86;
    if (var_98) {
        var_99 = norm_l2_0(var_88);
        var_100 = (var_99 < var_radius);
        var_98 = var_98 && var_100;
    }
    if (var_98) {
        // wlen, pnt0, pnt1 = wrap_inside(end, radius)                                        <L 423>
        wrap_inside_0(var_84, var_radius, var_104, var_105, var_106, var_101, var_102, var_103);
    }
    if (!var_98) {
        // wlen, pnt0, pnt1 = wrap_circle(end, sidepnt_proj, radius)                          <L 425>
        wrap_circle_0(var_84, var_97, var_radius, var_107, var_108, var_109);
    }
    var_110 = wp::where(var_98, var_101, var_107);
    var_111 = wp::where(var_98, var_102, var_108);
    var_112 = wp::where(var_98, var_103, var_109);
    // if wlen < 0.0:                                                                         <L 428>
    var_114 = (var_110 < var_113);
    if (var_114) {
        // return -1.0, wp.vec3(MJ_MAXVAL), wp.vec3(MJ_MAXVAL)                                <L 429>
        var_116 = wp::vec_t<3, wp::float32>(var_5);
        var_117 = wp::vec_t<3, wp::float32>(var_5);
        ret_0 = var_115;
        ret_1 = var_116;
        ret_2 = var_117;
        return;
    }
    // res0 = axis0 * pnt0[0] + axis1 * pnt0[1]                                               <L 432>
    var_119 = wp::extract(var_111, var_118);
    var_120 = wp::mul(var_78, var_119);
    var_122 = wp::extract(var_111, var_121);
    var_123 = wp::mul(var_79, var_122);
    var_124 = wp::add(var_120, var_123);
    // res1 = axis0 * pnt1[0] + axis1 * pnt1[1]                                               <L 433>
    var_126 = wp::extract(var_112, var_125);
    var_127 = wp::mul(var_78, var_126);
    var_129 = wp::extract(var_112, var_128);
    var_130 = wp::mul(var_79, var_129);
    var_131 = wp::add(var_127, var_130);
    // if geomtype == WrapType.CYLINDER:                                                      <L 436>
    var_133 = (var_geomtype == var_132);
    if (var_133) {
        // L0 = wp.sqrt((p0[0] - res0[0]) * (p0[0] - res0[0]) + (p0[1] - res0[1]) * (p0[1] - res0[1]))       <L 438>
        var_135 = wp::extract(var_10, var_134);
        var_137 = wp::extract(var_124, var_136);
        var_138 = wp::sub(var_135, var_137);
        var_140 = wp::extract(var_10, var_139);
        var_142 = wp::extract(var_124, var_141);
        var_143 = wp::sub(var_140, var_142);
        var_144 = wp::mul(var_138, var_143);
        var_146 = wp::extract(var_10, var_145);
        var_148 = wp::extract(var_124, var_147);
        var_149 = wp::sub(var_146, var_148);
        var_151 = wp::extract(var_10, var_150);
        var_153 = wp::extract(var_124, var_152);
        var_154 = wp::sub(var_151, var_153);
        var_155 = wp::mul(var_149, var_154);
        var_156 = wp::add(var_144, var_155);
        var_157 = wp::sqrt(var_156);
        // L1 = wp.sqrt((p1[0] - res1[0]) * (p1[0] - res1[0]) + (p1[1] - res1[1]) * (p1[1] - res1[1]))       <L 439>
        var_159 = wp::extract(var_12, var_158);
        var_161 = wp::extract(var_131, var_160);
        var_162 = wp::sub(var_159, var_161);
        var_164 = wp::extract(var_12, var_163);
        var_166 = wp::extract(var_131, var_165);
        var_167 = wp::sub(var_164, var_166);
        var_168 = wp::mul(var_162, var_167);
        var_170 = wp::extract(var_12, var_169);
        var_172 = wp::extract(var_131, var_171);
        var_173 = wp::sub(var_170, var_172);
        var_175 = wp::extract(var_12, var_174);
        var_177 = wp::extract(var_131, var_176);
        var_178 = wp::sub(var_175, var_177);
        var_179 = wp::mul(var_173, var_178);
        var_180 = wp::add(var_168, var_179);
        var_181 = wp::sqrt(var_180);
        // res0[2] = p0[2] + (p1[2] - p0[2]) * math.safe_div(L0, L0 + wlen + L1)              <L 440>
        var_183 = wp::extract(var_10, var_182);
        var_185 = wp::extract(var_12, var_184);
        var_187 = wp::extract(var_10, var_186);
        var_188 = wp::sub(var_185, var_187);
        var_189 = wp::add(var_157, var_110);
        var_190 = wp::add(var_189, var_181);
        var_191 = safe_div_0(var_157, var_190);
        var_192 = wp::mul(var_188, var_191);
        var_193 = wp::add(var_183, var_192);
        wp::assign_inplace(var_124, var_194, var_193);
        // res1[2] = p0[2] + (p1[2] - p0[2]) * math.safe_div(L0 + wlen, L0 + wlen + L1)       <L 441>
        var_196 = wp::extract(var_10, var_195);
        var_198 = wp::extract(var_12, var_197);
        var_200 = wp::extract(var_10, var_199);
        var_201 = wp::sub(var_198, var_200);
        var_202 = wp::add(var_157, var_110);
        var_203 = wp::add(var_157, var_110);
        var_204 = wp::add(var_203, var_181);
        var_205 = safe_div_0(var_202, var_204);
        var_206 = wp::mul(var_201, var_205);
        var_207 = wp::add(var_196, var_206);
        wp::assign_inplace(var_131, var_208, var_207);
        // height = wp.abs(res1[2] - res0[2])                                                 <L 444>
        var_210 = wp::extract(var_131, var_209);
        var_212 = wp::extract(var_124, var_211);
        var_213 = wp::sub(var_210, var_212);
        var_214 = wp::abs(var_213);
        // wlen = wp.sqrt(wlen * wlen + height * height)                                      <L 445>
        var_215 = wp::mul(var_110, var_110);
        var_216 = wp::mul(var_214, var_214);
        var_217 = wp::add(var_215, var_216);
        var_218 = wp::sqrt(var_217);
    }
    var_219 = wp::where(var_133, var_218, var_110);
    // wpnt0 = mat @ res0 + pos                                                               <L 448>
    var_220 = wp::mul(var_mat, var_124);
    var_221 = wp::add(var_220, var_pos);
    // wpnt1 = mat @ res1 + pos                                                               <L 449>
    var_222 = wp::mul(var_mat, var_131);
    var_223 = wp::add(var_222, var_pos);
    // return wlen, wpnt0, wpnt1                                                              <L 451>
    ret_0 = var_219;
    ret_1 = var_221;
    ret_2 = var_223;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:133
static CUDA_CALLABLE void adj_motion_cross_0(
    wp::vec_t<6, wp::float32> var_u,
    wp::vec_t<6, wp::float32> var_v,
    wp::vec_t<6, wp::float32> & adj_u,
    wp::vec_t<6, wp::float32> & adj_v,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:504
static CUDA_CALLABLE void adj_jac_dof_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::vec_t<3, wp::float32> var_point,
    wp::int32 var_bodyid,
    wp::int32 var_dofid,
    wp::int32 var_worldid,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::array_t<wp::int32> & adj_body_parentid,
    wp::array_t<wp::int32> & adj_body_rootid,
    wp::array_t<wp::int32> & adj_dof_bodyid,
    wp::array_t<wp::int32> & adj_body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_cdof_in,
    wp::vec_t<3, wp::float32> & adj_point,
    wp::int32 & adj_bodyid,
    wp::int32 & adj_dofid,
    wp::int32 & adj_worldid,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:115
static CUDA_CALLABLE void adj_quat_inv_0(
    wp::quat_t<wp::float32> var_quat,
    wp::quat_t<wp::float32> & adj_quat,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:44
static CUDA_CALLABLE void adj_rot_vec_quat_0(
    wp::vec_t<3, wp::float32> var_vec,
    wp::quat_t<wp::float32> var_quat,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::quat_t<wp::float32> & adj_quat,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\warp\_src\math.py:0
static CUDA_CALLABLE void adj_norm_l2_0(
    wp::vec_t<3, wp::float32> var_v,
    wp::vec_t<3, wp::float32> & adj_v,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:161
static CUDA_CALLABLE void adj_quat_to_vel_0(
    wp::quat_t<wp::float32> var_quat,
    wp::quat_t<wp::float32> & adj_quat,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void adj_safe_div_0(
    wp::float32 var_x,
    wp::float32 var_y,
    wp::float32 & adj_x,
    wp::float32 & adj_y,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void adj_safe_div_0(
    wp::vec_t<3, wp::float32> var_x,
    wp::float32 var_y,
    wp::vec_t<3, wp::float32> & adj_x,
    wp::float32 & adj_y,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:23
static CUDA_CALLABLE void adj_mul_quat_0(
    wp::quat_t<wp::float32> var_u,
    wp::quat_t<wp::float32> var_v,
    wp::quat_t<wp::float32> & adj_u,
    wp::quat_t<wp::float32> & adj_v,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:177
static CUDA_CALLABLE void adj_quat_sub_0(
    wp::quat_t<wp::float32> var_qa,
    wp::quat_t<wp::float32> var_qb,
    wp::quat_t<wp::float32> & adj_qa,
    wp::quat_t<wp::float32> & adj_qb,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1072
static CUDA_CALLABLE void adj_get_face_metadata_0(
    wp::int32 var_cellnum_x,
    wp::int32 var_cellnum_y,
    wp::int32 var_cellnum_z,
    wp::int32 var_face_elem_idx,
    wp::int32 var_order_abs,
    wp::int32 & ret_0,
    wp::int32 & ret_1,
    wp::int32 & ret_2,
    wp::int32 & ret_3,
    wp::int32 & ret_4,
    wp::int32 & ret_5,
    wp::int32 & adj_cellnum_x,
    wp::int32 & adj_cellnum_y,
    wp::int32 & adj_cellnum_z,
    wp::int32 & adj_face_elem_idx,
    wp::int32 & adj_order_abs,
    wp::int32 & adj_ret_0,
    wp::int32 & adj_ret_1,
    wp::int32 & adj_ret_2,
    wp::int32 & adj_ret_3,
    wp::int32 & adj_ret_4,
    wp::int32 & adj_ret_5)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:67
static CUDA_CALLABLE void adj_mat33_to_quat_polar_0(
    wp::mat_t<3, 3, wp::float32> var_F,
    wp::mat_t<3, 3, wp::float32> & adj_F,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:864
static CUDA_CALLABLE void adj__poly_force_0(
    wp::float32 var_linear,
    wp::vec_t<2, wp::float32> var_poly,
    wp::float32 var_x,
    wp::int32 var_flg_odd,
    wp::float32 & adj_linear,
    wp::vec_t<2, wp::float32> & adj_poly,
    wp::float32 & adj_x,
    wp::int32 & adj_flg_odd,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void adj_normalize_with_norm_0(
    wp::vec_t<3, wp::float32> var_x,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::float32 & ret_1,
    wp::vec_t<3, wp::float32> & adj_x,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::float32 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\smooth.py:2157
static CUDA_CALLABLE void adj__accumulate_jac_dot_chain_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_dof_jntid,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_dot_in,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<3, wp::float32> var_pvel_lin,
    wp::vec_t<3, wp::float32> var_dpnt,
    wp::vec_t<3, wp::float32> var_dvel,
    wp::int32 var_bodyid,
    wp::int32 var_rowadr,
    wp::int32 var_rownnz,
    wp::float32 var_scale,
    wp::int32 var_worldid,
    wp::array_t<wp::float32> var_ten_Jdot_out,
    wp::array_t<wp::int32> & adj_body_parentid,
    wp::array_t<wp::int32> & adj_body_dofnum,
    wp::array_t<wp::int32> & adj_body_dofadr,
    wp::array_t<wp::int32> & adj_jnt_type,
    wp::array_t<wp::int32> & adj_jnt_dofadr,
    wp::array_t<wp::int32> & adj_dof_jntid,
    wp::array_t<wp::int32> & adj_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_cdof_dot_in,
    wp::vec_t<3, wp::float32> & adj_offset,
    wp::vec_t<3, wp::float32> & adj_pvel_lin,
    wp::vec_t<3, wp::float32> & adj_dpnt,
    wp::vec_t<3, wp::float32> & adj_dvel,
    wp::int32 & adj_bodyid,
    wp::int32 & adj_rowadr,
    wp::int32 & adj_rownnz,
    wp::float32 & adj_scale,
    wp::int32 & adj_worldid,
    wp::array_t<wp::float32> & adj_ten_Jdot_out)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:476
static CUDA_CALLABLE void adj_transform_force_0(
    wp::vec_t<3, wp::float32> var_force,
    wp::vec_t<3, wp::float32> var_torque,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<3, wp::float32> & adj_force,
    wp::vec_t<3, wp::float32> & adj_torque,
    wp::vec_t<3, wp::float32> & adj_offset,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:481
static CUDA_CALLABLE void adj_transform_force_1(
    wp::vec_t<6, wp::float32> var_frc,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<6, wp::float32> & adj_frc,
    wp::vec_t<3, wp::float32> & adj_offset,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:33
static CUDA_CALLABLE void adj_quat_mul_axis_0(
    wp::quat_t<wp::float32> var_q,
    wp::vec_t<3, wp::float32> var_axis,
    wp::quat_t<wp::float32> & adj_q,
    wp::vec_t<3, wp::float32> & adj_axis,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:52
static CUDA_CALLABLE void adj_axis_angle_to_quat_0(
    wp::vec_t<3, wp::float32> var_axis,
    wp::float32 var_angle,
    wp::vec_t<3, wp::float32> & adj_axis,
    wp::float32 & adj_angle,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:59
static CUDA_CALLABLE void adj_quat_to_mat_0(
    wp::quat_t<wp::float32> var_quat,
    wp::quat_t<wp::float32> & adj_quat,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:991
static CUDA_CALLABLE void adj__phi_0(
    wp::float32 var_s,
    wp::int32 var_i,
    wp::float32 & adj_s,
    wp::int32 & adj_i,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1003
static CUDA_CALLABLE void adj_eval_basis_trilinear_0(
    wp::vec_t<3, wp::float32> var_local,
    wp::int32 var_node_idx,
    wp::vec_t<3, wp::float32> & adj_local,
    wp::int32 & adj_node_idx,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:120
static CUDA_CALLABLE void adj_inert_vec_0(
    wp::vec_t<10, wp::float32> var_i,
    wp::vec_t<6, wp::float32> var_v,
    wp::vec_t<10, wp::float32> & adj_i,
    wp::vec_t<6, wp::float32> & adj_v,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:147
static CUDA_CALLABLE void adj_motion_cross_force_0(
    wp::vec_t<6, wp::float32> var_v,
    wp::vec_t<6, wp::float32> var_f,
    wp::vec_t<6, wp::float32> & adj_v,
    wp::vec_t<6, wp::float32> & adj_f,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:325
static CUDA_CALLABLE void adj__decode_pyramid_0(
    wp::int32 var_njmax_in,
    wp::array_t<wp::float32> var_pyramid,
    wp::int32 var_efc_address,
    wp::vec_t<5, wp::float32> var_mu,
    wp::int32 var_condim,
    wp::int32 & adj_njmax_in,
    wp::array_t<wp::float32> & adj_pyramid,
    wp::int32 & adj_efc_address,
    wp::vec_t<5, wp::float32> & adj_mu,
    wp::int32 & adj_condim,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:351
static CUDA_CALLABLE void adj_contact_force_fn_0(
    wp::int32 var_opt_cone,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::vec_t<5, wp::float32>> var_contact_friction_in,
    wp::array_t<wp::int32> var_contact_dim_in,
    wp::array_t<wp::int32> var_contact_efc_address_in,
    wp::array_t<wp::float32> var_contact_adhesion_in,
    wp::array_t<wp::float32> var_efc_force_in,
    wp::int32 var_njmax_in,
    wp::array_t<wp::int32> var_nacon_in,
    wp::int32 var_worldid,
    wp::int32 var_contact_id,
    bool var_to_world_frame,
    wp::int32 & adj_opt_cone,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_contact_frame_in,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_contact_friction_in,
    wp::array_t<wp::int32> & adj_contact_dim_in,
    wp::array_t<wp::int32> & adj_contact_efc_address_in,
    wp::array_t<wp::float32> & adj_contact_adhesion_in,
    wp::array_t<wp::float32> & adj_efc_force_in,
    wp::int32 & adj_njmax_in,
    wp::array_t<wp::int32> & adj_nacon_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_contact_id,
    bool & adj_to_world_frame,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\smooth.py:4025
static CUDA_CALLABLE void adj__accumulate_jac_chain_0(
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::vec_t<3, wp::float32> var_offset,
    wp::vec_t<3, wp::float32> var_vec,
    wp::int32 var_bodyid,
    wp::int32 var_rowadr,
    wp::int32 var_rownnz,
    wp::float32 var_scale,
    wp::int32 var_worldid,
    wp::array_t<wp::float32> var_ten_J_out,
    wp::array_t<wp::int32> & adj_body_parentid,
    wp::array_t<wp::int32> & adj_body_dofnum,
    wp::array_t<wp::int32> & adj_body_dofadr,
    wp::array_t<wp::int32> & adj_ten_J_colind,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_cdof_in,
    wp::vec_t<3, wp::float32> & adj_offset,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::int32 & adj_bodyid,
    wp::int32 & adj_rowadr,
    wp::int32 & adj_rownnz,
    wp::float32 & adj_scale,
    wp::int32 & adj_worldid,
    wp::array_t<wp::float32> & adj_ten_J_out)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:0
static CUDA_CALLABLE void adj_normalize_with_norm_0(
    wp::vec_t<2, wp::float32> var_x,
    wp::vec_t<2, wp::float32> & ret_0,
    wp::float32 & ret_1,
    wp::vec_t<2, wp::float32> & adj_x,
    wp::vec_t<2, wp::float32> & adj_ret_0,
    wp::float32 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\warp\_src\math.py:0
static CUDA_CALLABLE void adj_norm_l2_0(
    wp::vec_t<2, wp::float32> var_v,
    wp::vec_t<2, wp::float32> & adj_v,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:202
static CUDA_CALLABLE void adj_wrap_inside_0(
    wp::vec_t<4, wp::float32> var_end,
    wp::float32 var_radius,
    wp::int32 var_maxiter,
    wp::float32 var_zinit,
    wp::float32 var_tolerance,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<4, wp::float32> & adj_end,
    wp::float32 & adj_radius,
    wp::int32 & adj_maxiter,
    wp::float32 & adj_zinit,
    wp::float32 & adj_tolerance,
    wp::float32 & adj_ret_0,
    wp::vec_t<2, wp::float32> & adj_ret_1,
    wp::vec_t<2, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:31
static CUDA_CALLABLE void adj_is_intersect_0(
    wp::vec_t<2, wp::float32> var_p1,
    wp::vec_t<2, wp::float32> var_p2,
    wp::vec_t<2, wp::float32> var_p3,
    wp::vec_t<2, wp::float32> var_p4,
    wp::vec_t<2, wp::float32> & adj_p1,
    wp::vec_t<2, wp::float32> & adj_p2,
    wp::vec_t<2, wp::float32> & adj_p3,
    wp::vec_t<2, wp::float32> & adj_p4,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:77
static CUDA_CALLABLE void adj_length_circle_0(
    wp::vec_t<2, wp::float32> var_p0,
    wp::vec_t<2, wp::float32> var_p1,
    wp::int32 var_ind,
    wp::float32 var_radius,
    wp::vec_t<2, wp::float32> & adj_p0,
    wp::vec_t<2, wp::float32> & adj_p1,
    wp::int32 & adj_ind,
    wp::float32 & adj_radius,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:104
static CUDA_CALLABLE void adj_wrap_circle_0(
    wp::vec_t<4, wp::float32> var_end,
    wp::vec_t<2, wp::float32> var_side,
    wp::float32 var_radius,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<4, wp::float32> & adj_end,
    wp::vec_t<2, wp::float32> & adj_side,
    wp::float32 & adj_radius,
    wp::float32 & adj_ret_0,
    wp::vec_t<2, wp::float32> & adj_ret_1,
    wp::vec_t<2, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:327
static CUDA_CALLABLE void adj_wrap_0(
    wp::vec_t<3, wp::float32> var_x0,
    wp::vec_t<3, wp::float32> var_x1,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::float32 var_radius,
    wp::int32 var_geomtype,
    wp::vec_t<3, wp::float32> var_side,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & adj_x0,
    wp::vec_t<3, wp::float32> & adj_x1,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_mat,
    wp::float32 & adj_radius,
    wp::int32 & adj_geomtype,
    wp::vec_t<3, wp::float32> & adj_side,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _subtree_div_0ef57c26_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_body_subtreemass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_out)
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
        wp::int32 var_1;
        wp::vec_t<3, wp::float32>* var_2;
        wp::vec_t<3, wp::float32> var_3;
        wp::vec_t<3, wp::float32> var_4;
        wp::shape_t* var_5;
        const wp::int32 var_6 = 0;
        wp::int32 var_7;
        wp::shape_t var_8;
        wp::int32 var_9;
        wp::float32* var_10;
        wp::float32 var_11;
        wp::float32 var_12;
        const wp::float32 var_13 = 0.0;
        bool var_14;
        wp::vec_t<3, wp::float32> var_15;
        //---------
        // forward
        // def _subtree_div(                                                                      <L 688>
        // worldid, bodyid = wp.tid()                                                             <L 696>
        builtin_tid2d(var_0, var_1);
        // com = subtree_com_in[worldid, bodyid]                                                  <L 697>
        var_2 = wp::address(var_subtree_com_in, var_0, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // mass = body_subtreemass[worldid % body_subtreemass.shape[0], bodyid]                   <L 698>
        var_5 = &(var_body_subtreemass.shape);
        var_8 = wp::load(var_5);
        var_7 = wp::extract(var_8, var_6);
        var_9 = wp::mod(var_0, var_7);
        var_10 = wp::address(var_body_subtreemass, var_9, var_1);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // if mass != 0.0:                                                                        <L 699>
        var_14 = (var_11 != var_13);
        if (var_14) {
            // subtree_com_out[worldid, bodyid] = com / mass                                      <L 700>
            var_15 = wp::div(var_3, var_11);
            wp::array_store(var_subtree_com_out, var_0, var_1, var_15);
        }
    }
}



extern "C" __global__ void _comvel_root_15820f35_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_out)
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
        wp::int32 var_1;
        const wp::float32 var_2 = 0.0;
        const wp::int32 var_3 = 0;
        //---------
        // forward
        // def _comvel_root(cvel_out: wp.array2d[wp.spatial_vector]):                             <L 2509>
        // worldid, elementid = wp.tid()                                                          <L 2510>
        builtin_tid2d(var_0, var_1);
        // cvel_out[worldid, 0][elementid] = 0.0                                                  <L 2511>
        wp::index(var_cvel_out, var_0, var_3)[var_1] = var_2;
    }
}



extern "C" __global__ void _joint_tendon_35c34b14_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_jnt_qposadr,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::float32> var_wrap_prm,
    wp::array_t<wp::int32> var_tendon_jnt_adr,
    wp::array_t<wp::int32> var_wrap_jnt_adr,
    wp::array_t<wp::float32> var_qpos_in,
    wp::array_t<wp::float32> var_ten_J_out,
    wp::array_t<wp::float32> var_ten_length_out)
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
        wp::float32* var_11;
        wp::float32 var_12;
        wp::float32 var_13;
        wp::int32* var_14;
        wp::float32* var_15;
        wp::int32 var_16;
        wp::float32 var_17;
        wp::float32 var_18;
        wp::slice_t var_19;
        const wp::int32 var_20 = 0;
        wp::array_t<wp::float32> var_21;
        wp::float32 var_22;
        wp::int32* var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::int32* var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        wp::range_t var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        wp::int32* var_35;
        bool var_36;
        wp::int32 var_37;
        wp::int32 var_38;
        //---------
        // forward
        // def _joint_tendon(                                                                     <L 3987>
        // worldid, wrapid = wp.tid()                                                             <L 4004>
        builtin_tid2d(var_0, var_1);
        // tenid = tendon_jnt_adr[wrapid]                                                         <L 4006>
        var_2 = wp::address(var_tendon_jnt_adr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // wrapjntid = wrap_jnt_adr[wrapid]                                                       <L 4007>
        var_5 = wp::address(var_wrap_jnt_adr, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // wrapobjid = wrap_objid[wrapjntid]                                                      <L 4008>
        var_8 = wp::address(var_wrap_objid, var_6);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // prm = wrap_prm[wrapjntid]                                                              <L 4009>
        var_11 = wp::address(var_wrap_prm, var_6);
        var_13 = wp::load(var_11);
        var_12 = wp::copy(var_13);
        // L = prm * qpos_in[worldid, jnt_qposadr[wrapobjid]]                                     <L 4012>
        var_14 = wp::address(var_jnt_qposadr, var_9);
        var_16 = wp::load(var_14);
        var_15 = wp::address(var_qpos_in, var_0, var_16);
        var_18 = wp::load(var_15);
        var_17 = wp::mul(var_12, var_18);
        // wp.atomic_add(ten_length_out[worldid], tenid, L)                                       <L 4013>
        var_19 = wp::slice_t(var_0, var_0, var_20);
        var_21 = wp::view(var_ten_length_out, var_19);
        var_22 = wp::atomic_add(var_21, var_3, var_17);
        // dofadr = jnt_dofadr[wrapobjid]                                                         <L 4016>
        var_23 = wp::address(var_jnt_dofadr, var_9);
        var_25 = wp::load(var_23);
        var_24 = wp::copy(var_25);
        // rowadr = ten_J_rowadr[tenid]                                                           <L 4017>
        var_26 = wp::address(var_ten_J_rowadr, var_3);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // rownnz = ten_J_rownnz[tenid]                                                           <L 4018>
        var_29 = wp::address(var_ten_J_rownnz, var_3);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
        // for k in range(rownnz):                                                                <L 4019>
        var_32 = wp::range(var_30);
        start_for_0:;
            if (iter_cmp(var_32) == 0) goto end_for_0;
            var_33 = wp::iter_next(var_32);
            // if ten_J_colind[rowadr + k] == dofadr:                                             <L 4020>
            var_34 = wp::add(var_27, var_33);
            var_35 = wp::address(var_ten_J_colind, var_34);
            var_37 = wp::load(var_35);
            var_36 = (var_37 == var_24);
            if (var_36) {
                // ten_J_out[worldid, rowadr + k] = prm                                           <L 4021>
                var_38 = wp::add(var_27, var_33);
                wp::array_store(var_ten_J_out, var_0, var_38, var_12);
                // break                                                                          <L 4022>
                goto end_for_0;
            }
            goto start_for_0;
        end_for_0:;
    }
}



extern "C" __global__ void _comvel_branch_0f933c58_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_jntnum,
    wp::array_t<wp::int32> var_body_jntadr,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_body_branches,
    wp::array_t<wp::int32> var_body_branch_start,
    wp::array_t<wp::float32> var_qvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_dot_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        const wp::int32 var_5 = 1;
        wp::int32 var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        wp::slice_t var_10;
        const wp::int32 var_11 = 0;
        wp::array_t<wp::float32> var_12;
        wp::slice_t var_13;
        const wp::int32 var_14 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_15;
        wp::range_t var_16;
        wp::int32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::int32* var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        wp::vec_t<6, wp::float32>* var_24;
        wp::vec_t<6, wp::float32> var_25;
        wp::vec_t<6, wp::float32> var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::int32* var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::int32* var_33;
        wp::int32 var_34;
        wp::int32 var_35;
        const wp::int32 var_36 = 0;
        bool var_37;
        wp::int32 var_38;
        wp::range_t var_39;
        wp::int32 var_40;
        wp::int32* var_41;
        wp::int32 var_42;
        wp::int32 var_43;
        const wp::int32 var_44 = 0;
        bool var_45;
        const wp::int32 var_46 = 0;
        wp::int32 var_47;
        wp::vec_t<6, wp::float32>* var_48;
        const wp::int32 var_49 = 0;
        wp::int32 var_50;
        wp::float32* var_51;
        wp::vec_t<6, wp::float32> var_52;
        wp::vec_t<6, wp::float32> var_53;
        wp::float32 var_54;
        wp::vec_t<6, wp::float32> var_55;
        const wp::int32 var_56 = 1;
        wp::int32 var_57;
        wp::vec_t<6, wp::float32>* var_58;
        const wp::int32 var_59 = 1;
        wp::int32 var_60;
        wp::float32* var_61;
        wp::vec_t<6, wp::float32> var_62;
        wp::vec_t<6, wp::float32> var_63;
        wp::float32 var_64;
        wp::vec_t<6, wp::float32> var_65;
        const wp::int32 var_66 = 2;
        wp::int32 var_67;
        wp::vec_t<6, wp::float32>* var_68;
        const wp::int32 var_69 = 2;
        wp::int32 var_70;
        wp::float32* var_71;
        wp::vec_t<6, wp::float32> var_72;
        wp::vec_t<6, wp::float32> var_73;
        wp::float32 var_74;
        wp::vec_t<6, wp::float32> var_75;
        const wp::float32 var_76 = 0.0;
        const wp::float32 var_77 = 0.0;
        const wp::float32 var_78 = 0.0;
        const wp::float32 var_79 = 0.0;
        const wp::float32 var_80 = 0.0;
        const wp::float32 var_81 = 0.0;
        wp::vec_t<6, wp::float32> var_82;
        const wp::int32 var_83 = 0;
        wp::int32 var_84;
        const wp::float32 var_85 = 0.0;
        const wp::float32 var_86 = 0.0;
        const wp::float32 var_87 = 0.0;
        const wp::float32 var_88 = 0.0;
        const wp::float32 var_89 = 0.0;
        const wp::float32 var_90 = 0.0;
        wp::vec_t<6, wp::float32> var_91;
        const wp::int32 var_92 = 1;
        wp::int32 var_93;
        const wp::float32 var_94 = 0.0;
        const wp::float32 var_95 = 0.0;
        const wp::float32 var_96 = 0.0;
        const wp::float32 var_97 = 0.0;
        const wp::float32 var_98 = 0.0;
        const wp::float32 var_99 = 0.0;
        wp::vec_t<6, wp::float32> var_100;
        const wp::int32 var_101 = 2;
        wp::int32 var_102;
        const wp::int32 var_103 = 3;
        wp::int32 var_104;
        wp::vec_t<6, wp::float32>* var_105;
        wp::vec_t<6, wp::float32> var_106;
        wp::vec_t<6, wp::float32> var_107;
        const wp::int32 var_108 = 3;
        wp::int32 var_109;
        const wp::int32 var_110 = 4;
        wp::int32 var_111;
        wp::vec_t<6, wp::float32>* var_112;
        wp::vec_t<6, wp::float32> var_113;
        wp::vec_t<6, wp::float32> var_114;
        const wp::int32 var_115 = 4;
        wp::int32 var_116;
        const wp::int32 var_117 = 5;
        wp::int32 var_118;
        wp::vec_t<6, wp::float32>* var_119;
        wp::vec_t<6, wp::float32> var_120;
        wp::vec_t<6, wp::float32> var_121;
        const wp::int32 var_122 = 5;
        wp::int32 var_123;
        const wp::int32 var_124 = 3;
        wp::int32 var_125;
        wp::vec_t<6, wp::float32>* var_126;
        const wp::int32 var_127 = 3;
        wp::int32 var_128;
        wp::float32* var_129;
        wp::vec_t<6, wp::float32> var_130;
        wp::vec_t<6, wp::float32> var_131;
        wp::float32 var_132;
        wp::vec_t<6, wp::float32> var_133;
        const wp::int32 var_134 = 4;
        wp::int32 var_135;
        wp::vec_t<6, wp::float32>* var_136;
        const wp::int32 var_137 = 4;
        wp::int32 var_138;
        wp::float32* var_139;
        wp::vec_t<6, wp::float32> var_140;
        wp::vec_t<6, wp::float32> var_141;
        wp::float32 var_142;
        wp::vec_t<6, wp::float32> var_143;
        const wp::int32 var_144 = 5;
        wp::int32 var_145;
        wp::vec_t<6, wp::float32>* var_146;
        const wp::int32 var_147 = 5;
        wp::int32 var_148;
        wp::float32* var_149;
        wp::vec_t<6, wp::float32> var_150;
        wp::vec_t<6, wp::float32> var_151;
        wp::float32 var_152;
        wp::vec_t<6, wp::float32> var_153;
        const wp::int32 var_154 = 6;
        wp::int32 var_155;
        wp::vec_t<6, wp::float32> var_156;
        wp::int32 var_157;
        const wp::int32 var_158 = 1;
        bool var_159;
        const wp::int32 var_160 = 0;
        wp::int32 var_161;
        wp::vec_t<6, wp::float32>* var_162;
        wp::vec_t<6, wp::float32> var_163;
        wp::vec_t<6, wp::float32> var_164;
        const wp::int32 var_165 = 0;
        wp::int32 var_166;
        const wp::int32 var_167 = 1;
        wp::int32 var_168;
        wp::vec_t<6, wp::float32>* var_169;
        wp::vec_t<6, wp::float32> var_170;
        wp::vec_t<6, wp::float32> var_171;
        const wp::int32 var_172 = 1;
        wp::int32 var_173;
        const wp::int32 var_174 = 2;
        wp::int32 var_175;
        wp::vec_t<6, wp::float32>* var_176;
        wp::vec_t<6, wp::float32> var_177;
        wp::vec_t<6, wp::float32> var_178;
        const wp::int32 var_179 = 2;
        wp::int32 var_180;
        const wp::int32 var_181 = 0;
        wp::int32 var_182;
        wp::vec_t<6, wp::float32>* var_183;
        const wp::int32 var_184 = 0;
        wp::int32 var_185;
        wp::float32* var_186;
        wp::vec_t<6, wp::float32> var_187;
        wp::vec_t<6, wp::float32> var_188;
        wp::float32 var_189;
        wp::vec_t<6, wp::float32> var_190;
        const wp::int32 var_191 = 1;
        wp::int32 var_192;
        wp::vec_t<6, wp::float32>* var_193;
        const wp::int32 var_194 = 1;
        wp::int32 var_195;
        wp::float32* var_196;
        wp::vec_t<6, wp::float32> var_197;
        wp::vec_t<6, wp::float32> var_198;
        wp::float32 var_199;
        wp::vec_t<6, wp::float32> var_200;
        const wp::int32 var_201 = 2;
        wp::int32 var_202;
        wp::vec_t<6, wp::float32>* var_203;
        const wp::int32 var_204 = 2;
        wp::int32 var_205;
        wp::float32* var_206;
        wp::vec_t<6, wp::float32> var_207;
        wp::vec_t<6, wp::float32> var_208;
        wp::float32 var_209;
        wp::vec_t<6, wp::float32> var_210;
        const wp::int32 var_211 = 3;
        wp::int32 var_212;
        wp::vec_t<6, wp::float32> var_213;
        wp::int32 var_214;
        wp::vec_t<6, wp::float32>* var_215;
        wp::vec_t<6, wp::float32> var_216;
        wp::vec_t<6, wp::float32> var_217;
        wp::vec_t<6, wp::float32>* var_218;
        wp::float32* var_219;
        wp::vec_t<6, wp::float32> var_220;
        wp::vec_t<6, wp::float32> var_221;
        wp::float32 var_222;
        wp::vec_t<6, wp::float32> var_223;
        const wp::int32 var_224 = 1;
        wp::int32 var_225;
        wp::vec_t<6, wp::float32> var_226;
        wp::int32 var_227;
        wp::vec_t<6, wp::float32> var_228;
        wp::int32 var_229;
        //---------
        // forward
        // def _comvel_branch(                                                                    <L 2515>
        // worldid, branchid = wp.tid()                                                           <L 2531>
        builtin_tid2d(var_0, var_1);
        // start = body_branch_start[branchid]                                                    <L 2533>
        var_2 = wp::address(var_body_branch_start, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // end = body_branch_start[branchid + 1]                                                  <L 2534>
        var_6 = wp::add(var_1, var_5);
        var_7 = wp::address(var_body_branch_start, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // qvel = qvel_in[worldid]                                                                <L 2536>
        var_10 = wp::slice_t(var_0, var_0, var_11);
        var_12 = wp::view(var_qvel_in, var_10);
        // cdof = cdof_in[worldid]                                                                <L 2537>
        var_13 = wp::slice_t(var_0, var_0, var_14);
        var_15 = wp::view(var_cdof_in, var_13);
        // for i in range(start, end):                                                            <L 2539>
        var_16 = wp::range(var_3, var_8);
        start_for_0:;
            if (iter_cmp(var_16) == 0) goto end_for_0;
            var_17 = wp::iter_next(var_16);
            // bodyid = body_branches[i]                                                          <L 2540>
            var_18 = wp::address(var_body_branches, var_17);
            var_20 = wp::load(var_18);
            var_19 = wp::copy(var_20);
            // pid = body_parentid[bodyid]                                                        <L 2541>
            var_21 = wp::address(var_body_parentid, var_19);
            var_23 = wp::load(var_21);
            var_22 = wp::copy(var_23);
            // cvel = cvel_out[worldid, pid]                                                      <L 2542>
            var_24 = wp::address(var_cvel_out, var_0, var_22);
            var_26 = wp::load(var_24);
            var_25 = wp::copy(var_26);
            // dofid = body_dofadr[bodyid]                                                        <L 2543>
            var_27 = wp::address(var_body_dofadr, var_19);
            var_29 = wp::load(var_27);
            var_28 = wp::copy(var_29);
            // jntid = body_jntadr[bodyid]                                                        <L 2544>
            var_30 = wp::address(var_body_jntadr, var_19);
            var_32 = wp::load(var_30);
            var_31 = wp::copy(var_32);
            // jntnum = body_jntnum[bodyid]                                                       <L 2545>
            var_33 = wp::address(var_body_jntnum, var_19);
            var_35 = wp::load(var_33);
            var_34 = wp::copy(var_35);
            // if jntnum == 0:                                                                    <L 2547>
            var_37 = (var_34 == var_36);
            if (var_37) {
                // cvel_out[worldid, bodyid] = cvel                                               <L 2548>
                wp::array_store(var_cvel_out, var_0, var_19, var_25);
                // continue                                                                       <L 2549>
                goto start_for_0;
            }
            // for j in range(jntid, jntid + jntnum):                                             <L 2551>
            var_38 = wp::add(var_31, var_34);
            var_39 = wp::range(var_31, var_38);
            start_for_2:;
                if (iter_cmp(var_39) == 0) goto end_for_2;
                var_40 = wp::iter_next(var_39);
                // jnttype = jnt_type[j]                                                          <L 2552>
                var_41 = wp::address(var_jnt_type, var_40);
                var_43 = wp::load(var_41);
                var_42 = wp::copy(var_43);
                // if jnttype == JointType.FREE:                                                  <L 2554>
                var_45 = (var_42 == var_44);
                if (var_45) {
                    // cvel += cdof[dofid + 0] * qvel[dofid + 0]                                  <L 2555>
                    var_47 = wp::add(var_28, var_46);
                    var_48 = wp::address(var_15, var_47);
                    var_50 = wp::add(var_28, var_49);
                    var_51 = wp::address(var_12, var_50);
                    var_53 = wp::load(var_48);
                    var_54 = wp::load(var_51);
                    var_52 = wp::mul(var_53, var_54);
                    var_55 = wp::add(var_25, var_52);
                    // cvel += cdof[dofid + 1] * qvel[dofid + 1]                                  <L 2556>
                    var_57 = wp::add(var_28, var_56);
                    var_58 = wp::address(var_15, var_57);
                    var_60 = wp::add(var_28, var_59);
                    var_61 = wp::address(var_12, var_60);
                    var_63 = wp::load(var_58);
                    var_64 = wp::load(var_61);
                    var_62 = wp::mul(var_63, var_64);
                    var_65 = wp::add(var_55, var_62);
                    // cvel += cdof[dofid + 2] * qvel[dofid + 2]                                  <L 2557>
                    var_67 = wp::add(var_28, var_66);
                    var_68 = wp::address(var_15, var_67);
                    var_70 = wp::add(var_28, var_69);
                    var_71 = wp::address(var_12, var_70);
                    var_73 = wp::load(var_68);
                    var_74 = wp::load(var_71);
                    var_72 = wp::mul(var_73, var_74);
                    var_75 = wp::add(var_65, var_72);
                    // cdof_dot_out[worldid, dofid + 0] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)       <L 2559>
                    var_82 = wp::vec_t<6, wp::float32>({var_76, var_77, var_78, var_79, var_80, var_81});
                    var_84 = wp::add(var_28, var_83);
                    wp::array_store(var_cdof_dot_out, var_0, var_84, var_82);
                    // cdof_dot_out[worldid, dofid + 1] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)       <L 2560>
                    var_91 = wp::vec_t<6, wp::float32>({var_85, var_86, var_87, var_88, var_89, var_90});
                    var_93 = wp::add(var_28, var_92);
                    wp::array_store(var_cdof_dot_out, var_0, var_93, var_91);
                    // cdof_dot_out[worldid, dofid + 2] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)       <L 2561>
                    var_100 = wp::vec_t<6, wp::float32>({var_94, var_95, var_96, var_97, var_98, var_99});
                    var_102 = wp::add(var_28, var_101);
                    wp::array_store(var_cdof_dot_out, var_0, var_102, var_100);
                    // cdof_dot_out[worldid, dofid + 3] = math.motion_cross(cvel, cdof[dofid + 3])       <L 2562>
                    var_104 = wp::add(var_28, var_103);
                    var_105 = wp::address(var_15, var_104);
                    var_107 = wp::load(var_105);
                    var_106 = motion_cross_0(var_75, var_107);
                    var_109 = wp::add(var_28, var_108);
                    wp::array_store(var_cdof_dot_out, var_0, var_109, var_106);
                    // cdof_dot_out[worldid, dofid + 4] = math.motion_cross(cvel, cdof[dofid + 4])       <L 2563>
                    var_111 = wp::add(var_28, var_110);
                    var_112 = wp::address(var_15, var_111);
                    var_114 = wp::load(var_112);
                    var_113 = motion_cross_0(var_75, var_114);
                    var_116 = wp::add(var_28, var_115);
                    wp::array_store(var_cdof_dot_out, var_0, var_116, var_113);
                    // cdof_dot_out[worldid, dofid + 5] = math.motion_cross(cvel, cdof[dofid + 5])       <L 2564>
                    var_118 = wp::add(var_28, var_117);
                    var_119 = wp::address(var_15, var_118);
                    var_121 = wp::load(var_119);
                    var_120 = motion_cross_0(var_75, var_121);
                    var_123 = wp::add(var_28, var_122);
                    wp::array_store(var_cdof_dot_out, var_0, var_123, var_120);
                    // cvel += cdof[dofid + 3] * qvel[dofid + 3]                                  <L 2566>
                    var_125 = wp::add(var_28, var_124);
                    var_126 = wp::address(var_15, var_125);
                    var_128 = wp::add(var_28, var_127);
                    var_129 = wp::address(var_12, var_128);
                    var_131 = wp::load(var_126);
                    var_132 = wp::load(var_129);
                    var_130 = wp::mul(var_131, var_132);
                    var_133 = wp::add(var_75, var_130);
                    // cvel += cdof[dofid + 4] * qvel[dofid + 4]                                  <L 2567>
                    var_135 = wp::add(var_28, var_134);
                    var_136 = wp::address(var_15, var_135);
                    var_138 = wp::add(var_28, var_137);
                    var_139 = wp::address(var_12, var_138);
                    var_141 = wp::load(var_136);
                    var_142 = wp::load(var_139);
                    var_140 = wp::mul(var_141, var_142);
                    var_143 = wp::add(var_133, var_140);
                    // cvel += cdof[dofid + 5] * qvel[dofid + 5]                                  <L 2568>
                    var_145 = wp::add(var_28, var_144);
                    var_146 = wp::address(var_15, var_145);
                    var_148 = wp::add(var_28, var_147);
                    var_149 = wp::address(var_12, var_148);
                    var_151 = wp::load(var_146);
                    var_152 = wp::load(var_149);
                    var_150 = wp::mul(var_151, var_152);
                    var_153 = wp::add(var_143, var_150);
                    // dofid += 6                                                                 <L 2570>
                    var_155 = wp::add(var_28, var_154);
                }
                var_156 = wp::where(var_45, var_153, var_25);
                var_157 = wp::where(var_45, var_155, var_28);
                if (!var_45) {
                    // elif jnttype == JointType.BALL:                                            <L 2571>
                    var_159 = (var_42 == var_158);
                    if (var_159) {
                        // cdof_dot_out[worldid, dofid + 0] = math.motion_cross(cvel, cdof[dofid + 0])       <L 2572>
                        var_161 = wp::add(var_157, var_160);
                        var_162 = wp::address(var_15, var_161);
                        var_164 = wp::load(var_162);
                        var_163 = motion_cross_0(var_156, var_164);
                        var_166 = wp::add(var_157, var_165);
                        wp::array_store(var_cdof_dot_out, var_0, var_166, var_163);
                        // cdof_dot_out[worldid, dofid + 1] = math.motion_cross(cvel, cdof[dofid + 1])       <L 2573>
                        var_168 = wp::add(var_157, var_167);
                        var_169 = wp::address(var_15, var_168);
                        var_171 = wp::load(var_169);
                        var_170 = motion_cross_0(var_156, var_171);
                        var_173 = wp::add(var_157, var_172);
                        wp::array_store(var_cdof_dot_out, var_0, var_173, var_170);
                        // cdof_dot_out[worldid, dofid + 2] = math.motion_cross(cvel, cdof[dofid + 2])       <L 2574>
                        var_175 = wp::add(var_157, var_174);
                        var_176 = wp::address(var_15, var_175);
                        var_178 = wp::load(var_176);
                        var_177 = motion_cross_0(var_156, var_178);
                        var_180 = wp::add(var_157, var_179);
                        wp::array_store(var_cdof_dot_out, var_0, var_180, var_177);
                        // cvel += cdof[dofid + 0] * qvel[dofid + 0]                              <L 2576>
                        var_182 = wp::add(var_157, var_181);
                        var_183 = wp::address(var_15, var_182);
                        var_185 = wp::add(var_157, var_184);
                        var_186 = wp::address(var_12, var_185);
                        var_188 = wp::load(var_183);
                        var_189 = wp::load(var_186);
                        var_187 = wp::mul(var_188, var_189);
                        var_190 = wp::add(var_156, var_187);
                        // cvel += cdof[dofid + 1] * qvel[dofid + 1]                              <L 2577>
                        var_192 = wp::add(var_157, var_191);
                        var_193 = wp::address(var_15, var_192);
                        var_195 = wp::add(var_157, var_194);
                        var_196 = wp::address(var_12, var_195);
                        var_198 = wp::load(var_193);
                        var_199 = wp::load(var_196);
                        var_197 = wp::mul(var_198, var_199);
                        var_200 = wp::add(var_190, var_197);
                        // cvel += cdof[dofid + 2] * qvel[dofid + 2]                              <L 2578>
                        var_202 = wp::add(var_157, var_201);
                        var_203 = wp::address(var_15, var_202);
                        var_205 = wp::add(var_157, var_204);
                        var_206 = wp::address(var_12, var_205);
                        var_208 = wp::load(var_203);
                        var_209 = wp::load(var_206);
                        var_207 = wp::mul(var_208, var_209);
                        var_210 = wp::add(var_200, var_207);
                        // dofid += 3                                                             <L 2580>
                        var_212 = wp::add(var_157, var_211);
                    }
                    var_213 = wp::where(var_159, var_210, var_156);
                    var_214 = wp::where(var_159, var_212, var_157);
                    if (!var_159) {
                        // cdof_dot_out[worldid, dofid] = math.motion_cross(cvel, cdof[dofid])       <L 2582>
                        var_215 = wp::address(var_15, var_214);
                        var_217 = wp::load(var_215);
                        var_216 = motion_cross_0(var_213, var_217);
                        wp::array_store(var_cdof_dot_out, var_0, var_214, var_216);
                        // cvel += cdof[dofid] * qvel[dofid]                                      <L 2583>
                        var_218 = wp::address(var_15, var_214);
                        var_219 = wp::address(var_12, var_214);
                        var_221 = wp::load(var_218);
                        var_222 = wp::load(var_219);
                        var_220 = wp::mul(var_221, var_222);
                        var_223 = wp::add(var_213, var_220);
                        // dofid += 1                                                             <L 2585>
                        var_225 = wp::add(var_214, var_224);
                    }
                    var_226 = wp::where(var_159, var_213, var_223);
                    var_227 = wp::where(var_159, var_214, var_225);
                }
                var_228 = wp::where(var_45, var_156, var_226);
                var_229 = wp::where(var_45, var_157, var_227);
                wp::assign(var_25, var_228);
                wp::assign(var_28, var_229);
                goto start_for_2;
            end_for_2:;
            // cvel_out[worldid, bodyid] = cvel                                                   <L 2587>
            wp::array_store(var_cvel_out, var_0, var_19, var_25);
            goto start_for_0;
        end_for_0:;
    }
}



extern "C" __global__ void _subtree_vel_forward_1d7ed456_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_body_inertia,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_ximat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_linvel_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_angmom_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_subtree_bodyvel_out)
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
        wp::int32 var_1;
        wp::shape_t* var_2;
        const wp::int32 var_3 = 0;
        wp::int32 var_4;
        wp::shape_t var_5;
        wp::int32 var_6;
        wp::shape_t* var_7;
        const wp::int32 var_8 = 0;
        wp::int32 var_9;
        wp::shape_t var_10;
        wp::int32 var_11;
        wp::vec_t<6, wp::float32>* var_12;
        wp::vec_t<6, wp::float32> var_13;
        wp::vec_t<6, wp::float32> var_14;
        wp::vec_t<3, wp::float32> var_15;
        wp::vec_t<3, wp::float32> var_16;
        wp::vec_t<3, wp::float32>* var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::mat_t<3, 3, wp::float32>* var_20;
        wp::mat_t<3, 3, wp::float32> var_21;
        wp::mat_t<3, 3, wp::float32> var_22;
        wp::int32* var_23;
        wp::vec_t<3, wp::float32>* var_24;
        wp::int32 var_25;
        wp::vec_t<3, wp::float32> var_26;
        wp::vec_t<3, wp::float32> var_27;
        wp::vec_t<3, wp::float32> var_28;
        wp::vec_t<3, wp::float32> var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::float32* var_31;
        wp::vec_t<3, wp::float32> var_32;
        wp::float32 var_33;
        wp::mat_t<3, 3, wp::float32> var_34;
        wp::vec_t<3, wp::float32> var_35;
        wp::vec_t<3, wp::float32>* var_36;
        const wp::int32 var_37 = 0;
        wp::float32 var_38;
        wp::vec_t<3, wp::float32> var_39;
        const wp::int32 var_40 = 0;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::vec_t<3, wp::float32>* var_43;
        const wp::int32 var_44 = 1;
        wp::float32 var_45;
        wp::vec_t<3, wp::float32> var_46;
        const wp::int32 var_47 = 1;
        wp::float32 var_48;
        wp::float32 var_49;
        wp::vec_t<3, wp::float32>* var_50;
        const wp::int32 var_51 = 2;
        wp::float32 var_52;
        wp::vec_t<3, wp::float32> var_53;
        const wp::int32 var_54 = 2;
        wp::float32 var_55;
        wp::float32 var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::vec_t<6, wp::float32> var_58;
        //---------
        // forward
        // def _subtree_vel_forward(                                                              <L 3831>
        // worldid, bodyid = wp.tid()                                                             <L 3847>
        builtin_tid2d(var_0, var_1);
        // body_mass_id = worldid % body_mass.shape[0]                                            <L 3848>
        var_2 = &(var_body_mass.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_0, var_4);
        // body_inertia_id = worldid % body_inertia.shape[0]                                      <L 3849>
        var_7 = &(var_body_inertia.shape);
        var_10 = wp::load(var_7);
        var_9 = wp::extract(var_10, var_8);
        var_11 = wp::mod(var_0, var_9);
        // cvel = cvel_in[worldid, bodyid]                                                        <L 3851>
        var_12 = wp::address(var_cvel_in, var_0, var_1);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // ang = wp.spatial_top(cvel)                                                             <L 3852>
        var_15 = wp::spatial_top(var_13);
        // lin = wp.spatial_bottom(cvel)                                                          <L 3853>
        var_16 = wp::spatial_bottom(var_13);
        // xipos = xipos_in[worldid, bodyid]                                                      <L 3854>
        var_17 = wp::address(var_xipos_in, var_0, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // ximat = ximat_in[worldid, bodyid]                                                      <L 3855>
        var_20 = wp::address(var_ximat_in, var_0, var_1);
        var_22 = wp::load(var_20);
        var_21 = wp::copy(var_22);
        // subtree_com_root = subtree_com_in[worldid, body_rootid[bodyid]]                        <L 3856>
        var_23 = wp::address(var_body_rootid, var_1);
        var_25 = wp::load(var_23);
        var_24 = wp::address(var_subtree_com_in, var_0, var_25);
        var_27 = wp::load(var_24);
        var_26 = wp::copy(var_27);
        // lin -= wp.cross(xipos - subtree_com_root, ang)                                         <L 3859>
        var_28 = wp::sub(var_18, var_26);
        var_29 = wp::cross(var_28, var_15);
        var_30 = wp::sub(var_16, var_29);
        // subtree_linvel_out[worldid, bodyid] = body_mass[body_mass_id, bodyid] * lin            <L 3861>
        var_31 = wp::address(var_body_mass, var_6, var_1);
        var_33 = wp::load(var_31);
        var_32 = wp::mul(var_33, var_30);
        wp::array_store(var_subtree_linvel_out, var_0, var_1, var_32);
        // dv = wp.transpose(ximat) @ ang                                                         <L 3862>
        var_34 = wp::transpose(var_21);
        var_35 = wp::mul(var_34, var_15);
        // dv[0] *= body_inertia[body_inertia_id, bodyid][0]                                      <L 3863>
        var_36 = wp::address(var_body_inertia, var_11, var_1);
        var_39 = wp::load(var_36);
        var_38 = wp::extract(var_39, var_37);
        var_41 = wp::extract(var_35, var_40);
        var_42 = wp::mul(var_41, var_38);
        wp::assign_inplace(var_35, var_40, var_42);
        // dv[1] *= body_inertia[body_inertia_id, bodyid][1]                                      <L 3864>
        var_43 = wp::address(var_body_inertia, var_11, var_1);
        var_46 = wp::load(var_43);
        var_45 = wp::extract(var_46, var_44);
        var_48 = wp::extract(var_35, var_47);
        var_49 = wp::mul(var_48, var_45);
        wp::assign_inplace(var_35, var_47, var_49);
        // dv[2] *= body_inertia[body_inertia_id, bodyid][2]                                      <L 3865>
        var_50 = wp::address(var_body_inertia, var_11, var_1);
        var_53 = wp::load(var_50);
        var_52 = wp::extract(var_53, var_51);
        var_55 = wp::extract(var_35, var_54);
        var_56 = wp::mul(var_55, var_52);
        wp::assign_inplace(var_35, var_54, var_56);
        // subtree_angmom_out[worldid, bodyid] = ximat @ dv                                       <L 3866>
        var_57 = wp::mul(var_21, var_35);
        wp::array_store(var_subtree_angmom_out, var_0, var_1, var_57);
        // subtree_bodyvel_out[worldid, bodyid] = wp.spatial_vector(ang, lin)                     <L 3867>
        var_58 = wp::vec_t<6, wp::float32>(var_15, var_30);
        wp::array_store(var_subtree_bodyvel_out, var_0, var_1, var_58);
    }
}



extern "C" __global__ void _transmission_body_moment_af66f321_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::int32 var_opt_cone,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::vec_t<2, wp::int32>> var_actuator_trnid,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::int32> var_actuator_trntype_body_adr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::int32> var_moment_rowadr_in,
    wp::array_t<wp::float32> var_contact_dist_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_contact_pos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::float32> var_contact_includemargin_in,
    wp::array_t<wp::int32> var_contact_dim_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_geom_in,
    wp::array_t<wp::int32> var_contact_efc_address_in,
    wp::array_t<wp::int32> var_contact_worldid_in,
    wp::array_t<wp::int32> var_efc_J_rownnz_in,
    wp::array_t<wp::int32> var_efc_J_rowadr_in,
    wp::array_t<wp::int32> var_efc_J_colind_in,
    wp::array_t<wp::float32> var_efc_J_in,
    wp::array_t<wp::int32> var_nacon_in,
    bool var_efc_is_sparse,
    wp::array_t<wp::float32> var_actuator_moment_out,
    wp::array_t<wp::int32> var_actuator_trntype_body_ncon_out)
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
        wp::int32 var_1;
        wp::int32 var_2;
        wp::int32* var_3;
        wp::int32 var_4;
        wp::int32 var_5;
        wp::vec_t<2, wp::int32>* var_6;
        const wp::int32 var_7 = 0;
        wp::int32 var_8;
        wp::vec_t<2, wp::int32> var_9;
        const wp::int32 var_10 = 0;
        wp::int32* var_11;
        bool var_12;
        wp::int32 var_13;
        wp::int32* var_14;
        wp::int32 var_15;
        wp::int32 var_16;
        wp::vec_t<2, wp::int32>* var_17;
        wp::vec_t<2, wp::int32> var_18;
        wp::vec_t<2, wp::int32> var_19;
        const wp::int32 var_20 = 0;
        wp::int32 var_21;
        const wp::int32 var_22 = 1;
        wp::int32 var_23;
        bool var_24;
        const wp::int32 var_25 = 0;
        bool var_26;
        const wp::int32 var_27 = 0;
        bool var_28;
        wp::int32* var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        bool var_35;
        bool var_36;
        bool var_37;
        wp::float32* var_38;
        wp::float32* var_39;
        bool var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::int32 var_43;
        const wp::int32 var_44 = 0;
        bool var_45;
        wp::slice_t var_46;
        const wp::int32 var_47 = 0;
        wp::array_t<wp::int32> var_48;
        const wp::int32 var_49 = 1;
        wp::int32 var_50;
        wp::int32* var_51;
        wp::int32 var_52;
        wp::int32 var_53;
        const wp::int32 var_54 = 0;
        bool var_55;
        wp::int32* var_56;
        wp::int32 var_57;
        wp::int32 var_58;
        wp::slice_t var_59;
        const wp::int32 var_60 = 0;
        wp::array_t<wp::int32> var_61;
        bool var_62;
        const wp::int32 var_63 = 1;
        bool var_64;
        const wp::int32 var_65 = 1;
        bool var_66;
        const wp::int32 var_67 = 0;
        wp::int32* var_68;
        wp::int32 var_69;
        wp::int32 var_70;
        wp::int32* var_71;
        wp::int32 var_72;
        wp::int32 var_73;
        bool var_74;
        wp::int32* var_75;
        wp::int32 var_76;
        wp::int32 var_77;
        wp::int32 var_78;
        const wp::int32 var_79 = 0;
        wp::int32* var_80;
        wp::int32 var_81;
        wp::int32 var_82;
        wp::slice_t var_83;
        const wp::int32 var_84 = 0;
        wp::array_t<wp::float32> var_85;
        wp::int32 var_86;
        const wp::int32 var_87 = 0;
        wp::float32* var_88;
        wp::float32 var_89;
        wp::float32 var_90;
        wp::int32 var_91;
        wp::slice_t var_92;
        const wp::int32 var_93 = 0;
        wp::array_t<wp::float32> var_94;
        wp::int32 var_95;
        wp::float32* var_96;
        wp::float32 var_97;
        wp::float32 var_98;
        wp::int32 var_99;
        const wp::int32 var_100 = 1;
        wp::int32 var_101;
        const wp::float32 var_102 = 0.5;
        wp::float32 var_103;
        wp::float32 var_104;
        const wp::int32 var_105 = 2;
        wp::int32 var_106;
        wp::range_t var_107;
        wp::int32 var_108;
        wp::int32* var_109;
        wp::int32 var_110;
        wp::int32 var_111;
        wp::int32* var_112;
        wp::int32 var_113;
        wp::int32 var_114;
        bool var_115;
        wp::int32* var_116;
        wp::int32 var_117;
        wp::int32 var_118;
        wp::int32 var_119;
        const wp::int32 var_120 = 0;
        wp::int32* var_121;
        wp::int32 var_122;
        wp::int32 var_123;
        wp::slice_t var_124;
        const wp::int32 var_125 = 0;
        wp::array_t<wp::float32> var_126;
        wp::int32 var_127;
        const wp::int32 var_128 = 0;
        wp::float32* var_129;
        wp::float32 var_130;
        wp::float32 var_131;
        wp::float32 var_132;
        wp::int32 var_133;
        wp::int32 var_134;
        wp::int32 var_135;
        wp::int32 var_136;
        wp::int32 var_137;
        wp::int32 var_138;
        wp::int32 var_139;
        wp::int32 var_140;
        wp::slice_t var_141;
        const wp::int32 var_142 = 0;
        wp::array_t<wp::float32> var_143;
        wp::int32 var_144;
        wp::float32* var_145;
        wp::float32 var_146;
        wp::float32 var_147;
        wp::float32 var_148;
        wp::int32 var_149;
        const wp::int32 var_150 = 1;
        bool var_151;
        wp::vec_t<3, wp::float32>* var_152;
        wp::vec_t<3, wp::float32> var_153;
        wp::vec_t<3, wp::float32> var_154;
        wp::mat_t<3, 3, wp::float32>* var_155;
        wp::mat_t<3, 3, wp::float32> var_156;
        wp::mat_t<3, 3, wp::float32> var_157;
        const wp::int32 var_158 = 0;
        const wp::int32 var_159 = 0;
        wp::float32 var_160;
        const wp::int32 var_161 = 0;
        const wp::int32 var_162 = 1;
        wp::float32 var_163;
        const wp::int32 var_164 = 0;
        const wp::int32 var_165 = 2;
        wp::float32 var_166;
        wp::vec_t<3, wp::float32> var_167;
        const wp::int32 var_168 = 0;
        wp::int32* var_169;
        wp::int32 var_170;
        wp::int32 var_171;
        bool var_172;
        const wp::int32 var_173 = 0;
        bool var_174;
        wp::int32* var_175;
        bool var_176;
        wp::int32 var_177;
        wp::int32* var_178;
        wp::int32 var_179;
        wp::int32 var_180;
        const wp::int32 var_181 = 0;
        wp::int32* var_182;
        wp::int32 var_183;
        wp::int32 var_184;
        wp::int32 var_185;
        wp::int32 var_186;
        wp::int32 var_187;
        wp::vec_t<3, wp::float32> var_188;
        wp::vec_t<3, wp::float32> var_189;
        wp::vec_t<3, wp::float32> var_190;
        wp::vec_t<3, wp::float32> var_191;
        wp::vec_t<3, wp::float32> var_192;
        wp::slice_t var_193;
        const wp::int32 var_194 = 0;
        wp::array_t<wp::float32> var_195;
        wp::int32 var_196;
        wp::float32 var_197;
        wp::float32 var_198;
        wp::int32 var_199;
        wp::int32 var_200;
        wp::int32 var_201;
        wp::int32 var_202;
        //---------
        // forward
        // def _transmission_body_moment(                                                         <L 3062>
        // trnbodyid, conid, dofid = wp.tid()                                                     <L 3096>
        builtin_tid3d(var_0, var_1, var_2);
        // actid = actuator_trntype_body_adr[trnbodyid]                                           <L 3097>
        var_3 = wp::address(var_actuator_trntype_body_adr, var_0);
        var_5 = wp::load(var_3);
        var_4 = wp::copy(var_5);
        // bodyid = actuator_trnid[actid][0]                                                      <L 3098>
        var_6 = wp::address(var_actuator_trnid, var_4);
        var_9 = wp::load(var_6);
        var_8 = wp::extract(var_9, var_7);
        // if conid >= nacon_in[0]:                                                               <L 3100>
        var_11 = wp::address(var_nacon_in, var_10);
        var_13 = wp::load(var_11);
        var_12 = (var_1 >= var_13);
        if (var_12) {
            // return                                                                             <L 3101>
            continue;
        }
        // worldid = contact_worldid_in[conid]                                                    <L 3103>
        var_14 = wp::address(var_contact_worldid_in, var_1);
        var_16 = wp::load(var_14);
        var_15 = wp::copy(var_16);
        // geom = contact_geom_in[conid]                                                          <L 3106>
        var_17 = wp::address(var_contact_geom_in, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // g1 = geom[0]                                                                           <L 3107>
        var_21 = wp::extract(var_18, var_20);
        // g2 = geom[1]                                                                           <L 3108>
        var_23 = wp::extract(var_18, var_22);
        // if g1 < 0 or g2 < 0:                                                                   <L 3111>
        var_26 = (var_21 < var_25);
        var_24 = var_26;
        if (!var_24) {
            var_28 = (var_23 < var_27);
            var_24 = var_24 || var_28;
        }
        if (var_24) {
            // return                                                                             <L 3112>
            continue;
        }
        // b1 = geom_bodyid[g1]                                                                   <L 3115>
        var_29 = wp::address(var_geom_bodyid, var_21);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
        // b2 = geom_bodyid[g2]                                                                   <L 3116>
        var_32 = wp::address(var_geom_bodyid, var_23);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // if b1 != bodyid and b2 != bodyid:                                                      <L 3119>
        var_36 = (var_30 != var_8);
        var_35 = var_36;
        if (var_35) {
            var_37 = (var_33 != var_8);
            var_35 = var_35 && var_37;
        }
        if (var_35) {
            // return                                                                             <L 3120>
            continue;
        }
        // contact_exclude = int(contact_dist_in[conid] >= contact_includemargin_in[conid])       <L 3122>
        var_38 = wp::address(var_contact_dist_in, var_1);
        var_39 = wp::address(var_contact_includemargin_in, var_1);
        var_41 = wp::load(var_38);
        var_42 = wp::load(var_39);
        var_40 = (var_41 >= var_42);
        var_43 = wp::int(var_40);
        // if dofid == 0:                                                                         <L 3124>
        var_45 = (var_2 == var_44);
        if (var_45) {
            // wp.atomic_add(actuator_trntype_body_ncon_out[worldid], trnbodyid, 1)               <L 3125>
            var_46 = wp::slice_t(var_15, var_15, var_47);
            var_48 = wp::view(var_actuator_trntype_body_ncon_out, var_46);
            var_50 = wp::atomic_add(var_48, var_0, var_49);
        }
        // rowadr = moment_rowadr_in[worldid, actid]                                              <L 3127>
        var_51 = wp::address(var_moment_rowadr_in, var_15, var_4);
        var_53 = wp::load(var_51);
        var_52 = wp::copy(var_53);
        // if contact_exclude == 0:                                                               <L 3130>
        var_55 = (var_43 == var_54);
        if (var_55) {
            // contact_dim = contact_dim_in[conid]                                                <L 3131>
            var_56 = wp::address(var_contact_dim_in, var_1);
            var_58 = wp::load(var_56);
            var_57 = wp::copy(var_58);
            // contact_efc_address = contact_efc_address_in[conid]                                <L 3132>
            var_59 = wp::slice_t(var_1, var_1, var_60);
            var_61 = wp::view(var_contact_efc_address_in, var_59);
            // if contact_dim == 1 or opt_cone == ConeType.ELLIPTIC:                              <L 3134>
            var_64 = (var_57 == var_63);
            var_62 = var_64;
            if (!var_62) {
                var_66 = (var_opt_cone == var_65);
                var_62 = var_62 || var_66;
            }
            if (var_62) {
                // efcid0 = contact_efc_address[0]                                                <L 3135>
                var_68 = wp::address(var_61, var_67);
                var_70 = wp::load(var_68);
                var_69 = wp::copy(var_70);
                // if efc_is_sparse:                                                              <L 3136>
                if (var_efc_is_sparse) {
                    // rownnz = efc_J_rownnz_in[worldid, efcid0]                                  <L 3137>
                    var_71 = wp::address(var_efc_J_rownnz_in, var_15, var_69);
                    var_73 = wp::load(var_71);
                    var_72 = wp::copy(var_73);
                    // if dofid < rownnz:                                                         <L 3138>
                    var_74 = (var_2 < var_72);
                    if (var_74) {
                        // efc_rowadr = efc_J_rowadr_in[worldid, efcid0]                          <L 3139>
                        var_75 = wp::address(var_efc_J_rowadr_in, var_15, var_69);
                        var_77 = wp::load(var_75);
                        var_76 = wp::copy(var_77);
                        // efc_sparseid = efc_rowadr + dofid                                      <L 3140>
                        var_78 = wp::add(var_76, var_2);
                        // colind = efc_J_colind_in[worldid, 0, efc_sparseid]                     <L 3141>
                        var_80 = wp::address(var_efc_J_colind_in, var_15, var_79, var_78);
                        var_82 = wp::load(var_80);
                        var_81 = wp::copy(var_82);
                        // wp.atomic_add(actuator_moment_out[worldid], rowadr + colind, efc_J_in[worldid, 0, efc_sparseid])       <L 3142>
                        var_83 = wp::slice_t(var_15, var_15, var_84);
                        var_85 = wp::view(var_actuator_moment_out, var_83);
                        var_86 = wp::add(var_52, var_81);
                        var_88 = wp::address(var_efc_J_in, var_15, var_87, var_78);
                        var_90 = wp::load(var_88);
                        var_89 = wp::atomic_add(var_85, var_86, var_90);
                    }
                    if (!var_74) {
                        // return                                                                 <L 3144>
                        continue;
                    }
                }
                if (!var_efc_is_sparse) {
                    // colind = dofid                                                             <L 3146>
                    var_91 = wp::copy(var_2);
                    // wp.atomic_add(actuator_moment_out[worldid], rowadr + colind, efc_J_in[worldid, efcid0, dofid])       <L 3147>
                    var_92 = wp::slice_t(var_15, var_15, var_93);
                    var_94 = wp::view(var_actuator_moment_out, var_92);
                    var_95 = wp::add(var_52, var_91);
                    var_96 = wp::address(var_efc_J_in, var_15, var_69, var_2);
                    var_98 = wp::load(var_96);
                    var_97 = wp::atomic_add(var_94, var_95, var_98);
                }
                var_99 = wp::where(var_efc_is_sparse, var_81, var_91);
            }
            if (!var_62) {
                // npyramid = contact_dim - 1  # number of frictional directions                  <L 3149>
                var_101 = wp::sub(var_57, var_100);
                // efc_force = 0.5 / float(npyramid)                                              <L 3150>
                var_103 = wp::float(var_101);
                var_104 = wp::div(var_102, var_103);
                // for j in range(2 * npyramid):                                                  <L 3152>
                var_106 = wp::mul(var_105, var_101);
                var_107 = wp::range(var_106);
                start_for_4:;
                    if (iter_cmp(var_107) == 0) goto end_for_4;
                    var_108 = wp::iter_next(var_107);
                    // efcid = contact_efc_address[j]                                             <L 3153>
                    var_109 = wp::address(var_61, var_108);
                    var_111 = wp::load(var_109);
                    var_110 = wp::copy(var_111);
                    // if efc_is_sparse:                                                          <L 3154>
                    if (var_efc_is_sparse) {
                        // rownnz = efc_J_rownnz_in[worldid, efcid]                               <L 3155>
                        var_112 = wp::address(var_efc_J_rownnz_in, var_15, var_110);
                        var_114 = wp::load(var_112);
                        var_113 = wp::copy(var_114);
                        // if dofid < rownnz:                                                     <L 3156>
                        var_115 = (var_2 < var_113);
                        if (var_115) {
                            // efc_rowadr = efc_J_rowadr_in[worldid, efcid]                       <L 3157>
                            var_116 = wp::address(var_efc_J_rowadr_in, var_15, var_110);
                            var_118 = wp::load(var_116);
                            var_117 = wp::copy(var_118);
                            // efc_sparseid = efc_rowadr + dofid                                  <L 3158>
                            var_119 = wp::add(var_117, var_2);
                            // colind = efc_J_colind_in[worldid, 0, efc_sparseid]                 <L 3159>
                            var_121 = wp::address(var_efc_J_colind_in, var_15, var_120, var_119);
                            var_123 = wp::load(var_121);
                            var_122 = wp::copy(var_123);
                            // wp.atomic_add(actuator_moment_out[worldid], rowadr + colind, efc_J_in[worldid, 0, efc_sparseid] * efc_force)       <L 3160>
                            var_124 = wp::slice_t(var_15, var_15, var_125);
                            var_126 = wp::view(var_actuator_moment_out, var_124);
                            var_127 = wp::add(var_52, var_122);
                            var_129 = wp::address(var_efc_J_in, var_15, var_128, var_119);
                            var_131 = wp::load(var_129);
                            var_130 = wp::mul(var_131, var_104);
                            var_132 = wp::atomic_add(var_126, var_127, var_130);
                        }
                        var_133 = wp::where(var_115, var_117, var_76);
                        var_134 = wp::where(var_115, var_119, var_78);
                        var_135 = wp::where(var_115, var_122, var_99);
                        if (!var_115) {
                            // return                                                             <L 3162>
                            continue;
                        }
                    }
                    var_136 = wp::where(var_efc_is_sparse, var_113, var_72);
                    var_137 = wp::where(var_efc_is_sparse, var_133, var_76);
                    var_138 = wp::where(var_efc_is_sparse, var_134, var_78);
                    var_139 = wp::where(var_efc_is_sparse, var_135, var_99);
                    if (!var_efc_is_sparse) {
                        // colind = dofid                                                         <L 3164>
                        var_140 = wp::copy(var_2);
                        // wp.atomic_add(actuator_moment_out[worldid], rowadr + colind, efc_J_in[worldid, efcid, dofid] * efc_force)       <L 3165>
                        var_141 = wp::slice_t(var_15, var_15, var_142);
                        var_143 = wp::view(var_actuator_moment_out, var_141);
                        var_144 = wp::add(var_52, var_140);
                        var_145 = wp::address(var_efc_J_in, var_15, var_110, var_2);
                        var_147 = wp::load(var_145);
                        var_146 = wp::mul(var_147, var_104);
                        var_148 = wp::atomic_add(var_143, var_144, var_146);
                    }
                    var_149 = wp::where(var_efc_is_sparse, var_139, var_140);
                    wp::assign(var_72, var_136);
                    wp::assign(var_76, var_137);
                    wp::assign(var_78, var_138);
                    wp::assign(var_99, var_149);
                    goto start_for_4;
                end_for_4:;
            }
        }
        if (!var_55) {
            // elif contact_exclude == 1:                                                         <L 3168>
            var_151 = (var_43 == var_150);
            if (var_151) {
                // contact_pos = contact_pos_in[conid]                                            <L 3169>
                var_152 = wp::address(var_contact_pos_in, var_1);
                var_154 = wp::load(var_152);
                var_153 = wp::copy(var_154);
                // contact_frame = contact_frame_in[conid]                                        <L 3170>
                var_155 = wp::address(var_contact_frame_in, var_1);
                var_157 = wp::load(var_155);
                var_156 = wp::copy(var_157);
                // normal = wp.vec3(contact_frame[0, 0], contact_frame[0, 1], contact_frame[0, 2])       <L 3171>
                var_160 = wp::extract(var_156, var_158, var_159);
                var_163 = wp::extract(var_156, var_161, var_162);
                var_166 = wp::extract(var_156, var_164, var_165);
                var_167 = wp::vec_t<3, wp::float32>(var_160, var_163, var_166);
                // efcid0 = contact_efc_address_in[conid][0]                                      <L 3174>
                var_169 = wp::address(var_contact_efc_address_in, var_1, var_168);
                var_171 = wp::load(var_169);
                var_170 = wp::copy(var_171);
                // if efc_is_sparse and efcid0 >= 0:                                              <L 3175>
                var_172 = var_efc_is_sparse;
                if (var_172) {
                    var_174 = (var_170 >= var_173);
                    var_172 = var_172 && var_174;
                }
                if (var_172) {
                    // if dofid >= efc_J_rownnz_in[worldid, efcid0]:                              <L 3177>
                    var_175 = wp::address(var_efc_J_rownnz_in, var_15, var_170);
                    var_177 = wp::load(var_175);
                    var_176 = (var_2 >= var_177);
                    if (var_176) {
                        // return                                                                 <L 3178>
                        continue;
                    }
                    // sparseid = efc_J_rowadr_in[worldid, efcid0] + dofid                        <L 3179>
                    var_178 = wp::address(var_efc_J_rowadr_in, var_15, var_170);
                    var_180 = wp::load(var_178);
                    var_179 = wp::add(var_180, var_2);
                    // colind = efc_J_colind_in[worldid, 0, sparseid]                             <L 3180>
                    var_182 = wp::address(var_efc_J_colind_in, var_15, var_181, var_179);
                    var_184 = wp::load(var_182);
                    var_183 = wp::copy(var_184);
                }
                var_185 = wp::where(var_172, var_183, var_99);
                if (!var_172) {
                    // colind = dofid                                                             <L 3183>
                    var_186 = wp::copy(var_2);
                }
                var_187 = wp::where(var_172, var_185, var_186);
                // jacp1, _ = support.jac_dof(                                                    <L 3185>
                // body_parentid, body_rootid, dof_bodyid, body_isdofancestor, subtree_com_in, cdof_in, contact_pos, b1, colind, worldid       <L 3186>
                jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_153, var_30, var_187, var_15, var_188, var_189);
                // jacp2, _ = support.jac_dof(                                                    <L 3188>
                // body_parentid, body_rootid, dof_bodyid, body_isdofancestor, subtree_com_in, cdof_in, contact_pos, b2, colind, worldid       <L 3189>
                jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_153, var_33, var_187, var_15, var_190, var_191);
                // jacdif = jacp2 - jacp1                                                         <L 3192>
                var_192 = wp::sub(var_190, var_188);
                // wp.atomic_add(actuator_moment_out[worldid], rowadr + colind, wp.dot(normal, jacdif))       <L 3195>
                var_193 = wp::slice_t(var_15, var_15, var_194);
                var_195 = wp::view(var_actuator_moment_out, var_193);
                var_196 = wp::add(var_52, var_187);
                var_197 = wp::dot(var_167, var_192);
                var_198 = wp::atomic_add(var_195, var_196, var_197);
            }
            var_199 = wp::where(var_151, var_170, var_69);
            var_200 = wp::where(var_151, var_187, var_99);
        }
        var_201 = wp::where(var_55, var_69, var_199);
        var_202 = wp::where(var_55, var_99, var_200);
    }
}



extern "C" __global__ void _linear_momentum_813ce6af_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::float32> var_body_subtreemass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_linvel_in,
    wp::array_t<wp::int32> var_body_tree_,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_linvel_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::slice_t var_8;
        const wp::int32 var_9 = 0;
        wp::array_t<wp::vec_t<3, wp::float32>> var_10;
        wp::vec_t<3, wp::float32>* var_11;
        wp::vec_t<3, wp::float32> var_12;
        wp::vec_t<3, wp::float32> var_13;
        const wp::float32 var_14 = 1e-15;
        wp::shape_t* var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        wp::shape_t var_18;
        wp::int32 var_19;
        wp::float32* var_20;
        wp::float32 var_21;
        wp::float32 var_22;
        wp::vec_t<3, wp::float32>* var_23;
        wp::vec_t<3, wp::float32> var_24;
        wp::vec_t<3, wp::float32> var_25;
        //---------
        // forward
        // def _linear_momentum(                                                                  <L 3871>
        // worldid, nodeid = wp.tid()                                                             <L 3882>
        builtin_tid2d(var_0, var_1);
        // bodyid = body_tree_[nodeid]                                                            <L 3883>
        var_2 = wp::address(var_body_tree_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if bodyid:                                                                             <L 3884>
        if (var_3) {
            // pid = body_parentid[bodyid]                                                        <L 3885>
            var_5 = wp::address(var_body_parentid, var_3);
            var_7 = wp::load(var_5);
            var_6 = wp::copy(var_7);
            // wp.atomic_add(subtree_linvel_out[worldid], pid, subtree_linvel_in[worldid, bodyid])       <L 3886>
            var_8 = wp::slice_t(var_0, var_0, var_9);
            var_10 = wp::view(var_subtree_linvel_out, var_8);
            var_11 = wp::address(var_subtree_linvel_in, var_0, var_3);
            var_13 = wp::load(var_11);
            var_12 = wp::atomic_add(var_10, var_6, var_13);
        }
        // subtree_linvel_out[worldid, bodyid] /= wp.max(MJ_MINVAL, body_subtreemass[worldid % body_subtreemass.shape[0], bodyid])       <L 3887>
        var_15 = &(var_body_subtreemass.shape);
        var_18 = wp::load(var_15);
        var_17 = wp::extract(var_18, var_16);
        var_19 = wp::mod(var_0, var_17);
        var_20 = wp::address(var_body_subtreemass, var_19, var_3);
        var_22 = wp::load(var_20);
        var_21 = wp::max(var_14, var_22);
        var_23 = wp::address(var_subtree_linvel_out, var_0, var_3);
        var_25 = wp::load(var_23);
        var_24 = wp::div(var_25, var_21);
        wp::array_store(var_subtree_linvel_out, var_0, var_3, var_24);
    }
}



extern "C" __global__ void _transmission_4db1a205_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nv,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_qposadr,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_dof_parentid,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::quat_t<wp::float32>> var_site_quat,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::int32> var_actuator_trntype,
    wp::array_t<wp::vec_t<2, wp::int32>> var_actuator_trnid,
    wp::array_t<wp::float32> var_actuator_cranklength,
    wp::array_t<wp::vec_t<6, wp::float32>> var_actuator_gear,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::float32> var_qpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_site_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::float32> var_ten_J_in,
    wp::array_t<wp::float32> var_ten_length_in,
    wp::array_t<wp::int32> var_moment_nnz,
    wp::array_t<wp::float32> var_actuator_length_out,
    wp::array_t<wp::int32> var_moment_rownnz_out,
    wp::array_t<wp::int32> var_moment_rowadr_out,
    wp::array_t<wp::int32> var_moment_colind_out,
    wp::array_t<wp::float32> var_actuator_moment_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::shape_t* var_5;
        const wp::int32 var_6 = 0;
        wp::int32 var_7;
        wp::shape_t var_8;
        wp::int32 var_9;
        wp::vec_t<6, wp::float32>* var_10;
        wp::vec_t<6, wp::float32> var_11;
        wp::vec_t<6, wp::float32> var_12;
        bool var_13;
        const wp::int32 var_14 = 0;
        bool var_15;
        const wp::int32 var_16 = 1;
        bool var_17;
        wp::slice_t var_18;
        const wp::int32 var_19 = 0;
        wp::array_t<wp::float32> var_20;
        wp::vec_t<2, wp::int32>* var_21;
        const wp::int32 var_22 = 0;
        wp::int32 var_23;
        wp::vec_t<2, wp::int32> var_24;
        wp::int32* var_25;
        wp::int32 var_26;
        wp::int32 var_27;
        wp::int32* var_28;
        wp::int32 var_29;
        wp::int32 var_30;
        wp::int32* var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        const wp::int32 var_34 = 0;
        bool var_35;
        const wp::int32 var_36 = 6;
        const wp::int32 var_37 = 6;
        wp::int32 var_38;
        const wp::int32 var_39 = 0;
        wp::int32 var_40;
        const wp::int32 var_41 = 0;
        wp::int32 var_42;
        const wp::int32 var_43 = 1;
        wp::int32 var_44;
        const wp::int32 var_45 = 1;
        wp::int32 var_46;
        const wp::int32 var_47 = 2;
        wp::int32 var_48;
        const wp::int32 var_49 = 2;
        wp::int32 var_50;
        const wp::int32 var_51 = 3;
        wp::int32 var_52;
        const wp::int32 var_53 = 3;
        wp::int32 var_54;
        const wp::int32 var_55 = 4;
        wp::int32 var_56;
        const wp::int32 var_57 = 4;
        wp::int32 var_58;
        const wp::int32 var_59 = 5;
        wp::int32 var_60;
        const wp::int32 var_61 = 5;
        wp::int32 var_62;
        const wp::float32 var_63 = 0.0;
        const wp::int32 var_64 = 1;
        bool var_65;
        const wp::int32 var_66 = 3;
        wp::int32 var_67;
        wp::float32* var_68;
        const wp::int32 var_69 = 4;
        wp::int32 var_70;
        wp::float32* var_71;
        const wp::int32 var_72 = 5;
        wp::int32 var_73;
        wp::float32* var_74;
        const wp::int32 var_75 = 6;
        wp::int32 var_76;
        wp::float32* var_77;
        wp::quat_t<wp::float32> var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::float32 var_81;
        wp::float32 var_82;
        wp::quat_t<wp::float32> var_83;
        wp::quat_t<wp::float32> var_84;
        wp::vec_t<3, wp::float32> var_85;
        wp::vec_t<3, wp::float32> var_86;
        const wp::int32 var_87 = 0;
        wp::float32 var_88;
        const wp::int32 var_89 = 0;
        wp::int32 var_90;
        const wp::int32 var_91 = 1;
        wp::float32 var_92;
        const wp::int32 var_93 = 1;
        wp::int32 var_94;
        const wp::int32 var_95 = 2;
        wp::float32 var_96;
        const wp::int32 var_97 = 2;
        wp::int32 var_98;
        const wp::int32 var_99 = 0;
        wp::float32 var_100;
        const wp::int32 var_101 = 3;
        wp::int32 var_102;
        const wp::int32 var_103 = 1;
        wp::float32 var_104;
        const wp::int32 var_105 = 4;
        wp::int32 var_106;
        const wp::int32 var_107 = 2;
        wp::float32 var_108;
        const wp::int32 var_109 = 5;
        wp::int32 var_110;
        const wp::int32 var_111 = 0;
        wp::float32 var_112;
        const wp::int32 var_113 = 0;
        wp::int32 var_114;
        const wp::int32 var_115 = 1;
        wp::float32 var_116;
        const wp::int32 var_117 = 1;
        wp::int32 var_118;
        const wp::int32 var_119 = 2;
        wp::float32 var_120;
        const wp::int32 var_121 = 2;
        wp::int32 var_122;
        const wp::int32 var_123 = 3;
        wp::float32 var_124;
        const wp::int32 var_125 = 3;
        wp::int32 var_126;
        const wp::int32 var_127 = 4;
        wp::float32 var_128;
        const wp::int32 var_129 = 4;
        wp::int32 var_130;
        const wp::int32 var_131 = 5;
        wp::float32 var_132;
        const wp::int32 var_133 = 5;
        wp::int32 var_134;
        const wp::int32 var_135 = 1;
        bool var_136;
        const wp::int32 var_137 = 0;
        wp::int32 var_138;
        wp::float32* var_139;
        const wp::int32 var_140 = 1;
        wp::int32 var_141;
        wp::float32* var_142;
        const wp::int32 var_143 = 2;
        wp::int32 var_144;
        wp::float32* var_145;
        const wp::int32 var_146 = 3;
        wp::int32 var_147;
        wp::float32* var_148;
        wp::quat_t<wp::float32> var_149;
        wp::float32 var_150;
        wp::float32 var_151;
        wp::float32 var_152;
        wp::float32 var_153;
        wp::quat_t<wp::float32> var_154;
        wp::vec_t<3, wp::float32> var_155;
        wp::vec_t<3, wp::float32> var_156;
        const wp::int32 var_157 = 1;
        bool var_158;
        wp::quat_t<wp::float32> var_159;
        wp::vec_t<3, wp::float32> var_160;
        wp::quat_t<wp::float32> var_161;
        wp::vec_t<3, wp::float32> var_162;
        wp::float32 var_163;
        const wp::int32 var_164 = 3;
        wp::int32 var_165;
        const wp::int32 var_166 = 0;
        wp::int32 var_167;
        wp::int32 var_168;
        wp::float32 var_169;
        const wp::int32 var_170 = 1;
        wp::int32 var_171;
        wp::int32 var_172;
        wp::float32 var_173;
        const wp::int32 var_174 = 2;
        wp::int32 var_175;
        wp::int32 var_176;
        wp::float32 var_177;
        wp::int32 var_178;
        wp::quat_t<wp::float32> var_179;
        wp::vec_t<3, wp::float32> var_180;
        bool var_181;
        const wp::int32 var_182 = 2;
        bool var_183;
        const wp::int32 var_184 = 3;
        bool var_185;
        wp::float32* var_186;
        const wp::int32 var_187 = 0;
        wp::float32 var_188;
        wp::float32 var_189;
        wp::float32 var_190;
        const wp::int32 var_191 = 1;
        wp::int32 var_192;
        const wp::int32 var_193 = 0;
        wp::float32 var_194;
        wp::int32 var_195;
        wp::int32 var_196;
        const wp::str var_197 = "unrecognized joint type";
        wp::int32 var_198;
        wp::int32 var_199;
        wp::int32 var_200;
        wp::quat_t<wp::float32> var_201;
        wp::vec_t<3, wp::float32> var_202;
        const wp::int32 var_203 = 2;
        bool var_204;
        wp::vec_t<2, wp::int32>* var_205;
        wp::vec_t<2, wp::int32> var_206;
        wp::vec_t<2, wp::int32> var_207;
        const wp::int32 var_208 = 0;
        wp::int32 var_209;
        const wp::int32 var_210 = 1;
        wp::int32 var_211;
        const wp::int32 var_212 = 0;
        wp::float32 var_213;
        wp::shape_t* var_214;
        const wp::int32 var_215 = 0;
        wp::int32 var_216;
        wp::shape_t var_217;
        wp::int32 var_218;
        wp::float32* var_219;
        wp::float32 var_220;
        wp::float32 var_221;
        wp::mat_t<3, 3, wp::float32>* var_222;
        wp::mat_t<3, 3, wp::float32> var_223;
        wp::mat_t<3, 3, wp::float32> var_224;
        const wp::int32 var_225 = 0;
        const wp::int32 var_226 = 2;
        wp::float32 var_227;
        const wp::int32 var_228 = 1;
        const wp::int32 var_229 = 2;
        wp::float32 var_230;
        const wp::int32 var_231 = 2;
        const wp::int32 var_232 = 2;
        wp::float32 var_233;
        wp::vec_t<3, wp::float32> var_234;
        wp::vec_t<3, wp::float32>* var_235;
        wp::vec_t<3, wp::float32> var_236;
        wp::vec_t<3, wp::float32> var_237;
        wp::vec_t<3, wp::float32>* var_238;
        wp::vec_t<3, wp::float32> var_239;
        wp::vec_t<3, wp::float32> var_240;
        wp::vec_t<3, wp::float32> var_241;
        wp::float32 var_242;
        wp::float32 var_243;
        wp::float32 var_244;
        wp::float32 var_245;
        wp::float32 var_246;
        wp::float32 var_247;
        const wp::int32 var_248 = 1;
        const wp::float32 var_249 = 0.0;
        bool var_250;
        const wp::int32 var_251 = 0;
        const wp::float32 var_252 = 0.0;
        wp::float32 var_253;
        wp::int32 var_254;
        wp::float32 var_255;
        wp::float32 var_256;
        wp::float32 var_257;
        wp::float32 var_258;
        wp::float32 var_259;
        const wp::int32 var_260 = 1;
        bool var_261;
        const wp::float32 var_262 = 1.0;
        wp::float32 var_263;
        wp::float32 var_264;
        wp::vec_t<3, wp::float32> var_265;
        wp::vec_t<3, wp::float32> var_266;
        wp::vec_t<3, wp::float32> var_267;
        wp::vec_t<3, wp::float32> var_268;
        wp::vec_t<3, wp::float32> var_269;
        wp::vec_t<3, wp::float32> var_270;
        wp::vec_t<3, wp::float32> var_271;
        wp::vec_t<3, wp::float32> var_272;
        wp::int32* var_273;
        wp::int32* var_274;
        wp::int32 var_275;
        wp::int32 var_276;
        wp::int32 var_277;
        wp::int32* var_278;
        wp::int32* var_279;
        wp::int32 var_280;
        wp::int32 var_281;
        wp::int32 var_282;
        const wp::int32 var_283 = -1;
        wp::int32 var_284;
        const wp::int32 var_285 = -1;
        wp::int32 var_286;
        const wp::int32 var_287 = 0;
        bool var_288;
        wp::int32* var_289;
        wp::int32* var_290;
        wp::int32 var_291;
        wp::int32 var_292;
        wp::int32 var_293;
        const wp::int32 var_294 = 1;
        wp::int32 var_295;
        wp::int32 var_296;
        const wp::int32 var_297 = 0;
        bool var_298;
        wp::int32* var_299;
        wp::int32* var_300;
        wp::int32 var_301;
        wp::int32 var_302;
        wp::int32 var_303;
        const wp::int32 var_304 = 1;
        wp::int32 var_305;
        wp::int32 var_306;
        wp::int32 var_307;
        wp::int32 var_308;
        const wp::int32 var_309 = 0;
        wp::int32 var_310;
        bool var_311;
        const wp::int32 var_312 = 0;
        bool var_313;
        const wp::int32 var_314 = 0;
        bool var_315;
        wp::int32 var_316;
        const wp::int32 var_317 = 1;
        wp::int32 var_318;
        bool var_319;
        wp::int32* var_320;
        wp::int32 var_321;
        wp::int32 var_322;
        wp::int32 var_323;
        bool var_324;
        wp::int32* var_325;
        wp::int32 var_326;
        wp::int32 var_327;
        wp::int32 var_328;
        wp::int32 var_329;
        wp::int32 var_330;
        wp::int32 var_331;
        const wp::int32 var_332 = 1;
        wp::int32 var_333;
        bool var_334;
        const wp::int32 var_335 = 0;
        bool var_336;
        const wp::int32 var_337 = 0;
        bool var_338;
        wp::int32 var_339;
        wp::int32* var_340;
        wp::vec_t<3, wp::float32> var_341;
        wp::vec_t<3, wp::float32> var_342;
        wp::int32 var_343;
        wp::vec_t<3, wp::float32> var_344;
        wp::vec_t<3, wp::float32> var_345;
        wp::int32* var_346;
        wp::vec_t<3, wp::float32> var_347;
        wp::vec_t<3, wp::float32> var_348;
        wp::int32 var_349;
        wp::vec_t<3, wp::float32> var_350;
        wp::float32 var_351;
        wp::float32 var_352;
        wp::float32 var_353;
        wp::int32 var_354;
        wp::float32 var_355;
        const wp::int32 var_356 = 1;
        wp::int32 var_357;
        bool var_358;
        wp::int32* var_359;
        wp::int32 var_360;
        wp::int32 var_361;
        wp::int32 var_362;
        bool var_363;
        wp::int32* var_364;
        wp::int32 var_365;
        wp::int32 var_366;
        wp::int32 var_367;
        wp::int32 var_368;
        const wp::int32 var_369 = 3;
        bool var_370;
        wp::vec_t<2, wp::int32>* var_371;
        const wp::int32 var_372 = 0;
        wp::int32 var_373;
        wp::vec_t<2, wp::int32> var_374;
        const wp::int32 var_375 = 0;
        wp::float32 var_376;
        wp::float32* var_377;
        wp::float32 var_378;
        wp::float32 var_379;
        wp::int32* var_380;
        wp::int32 var_381;
        wp::int32 var_382;
        wp::int32* var_383;
        wp::int32 var_384;
        wp::int32 var_385;
        wp::int32 var_386;
        wp::range_t var_387;
        wp::int32 var_388;
        wp::int32 var_389;
        wp::int32 var_390;
        wp::int32* var_391;
        wp::int32 var_392;
        wp::float32* var_393;
        wp::float32 var_394;
        wp::float32 var_395;
        wp::float32 var_396;
        const wp::int32 var_397 = 5;
        bool var_398;
        const wp::float32 var_399 = 0.0;
        wp::int32 var_400;
        wp::range_t var_401;
        wp::int32 var_402;
        wp::int32 var_403;
        const wp::float32 var_404 = 0.0;
        wp::int32 var_405;
        wp::int32 var_406;
        const wp::int32 var_407 = 4;
        bool var_408;
        wp::vec_t<2, wp::int32>* var_409;
        wp::vec_t<2, wp::int32> var_410;
        wp::vec_t<2, wp::int32> var_411;
        const wp::int32 var_412 = 0;
        wp::int32 var_413;
        const wp::int32 var_414 = 1;
        wp::int32 var_415;
        wp::vec_t<6, wp::float32>* var_416;
        wp::vec_t<6, wp::float32> var_417;
        wp::vec_t<6, wp::float32> var_418;
        wp::shape_t* var_419;
        const wp::int32 var_420 = 0;
        wp::int32 var_421;
        wp::shape_t var_422;
        wp::int32 var_423;
        wp::vec_t<3, wp::float32> var_424;
        wp::vec_t<3, wp::float32> var_425;
        const wp::int32 var_426 = -1;
        bool var_427;
        wp::mat_t<3, 3, wp::float32>* var_428;
        wp::mat_t<3, 3, wp::float32> var_429;
        wp::mat_t<3, 3, wp::float32> var_430;
        wp::vec_t<3, wp::float32> var_431;
        wp::vec_t<3, wp::float32> var_432;
        wp::int32* var_433;
        wp::int32* var_434;
        wp::int32 var_435;
        wp::int32 var_436;
        wp::int32 var_437;
        const wp::int32 var_438 = -1;
        wp::int32 var_439;
        const wp::int32 var_440 = 0;
        bool var_441;
        wp::int32* var_442;
        wp::int32* var_443;
        wp::int32 var_444;
        wp::int32 var_445;
        wp::int32 var_446;
        const wp::int32 var_447 = 1;
        wp::int32 var_448;
        wp::int32 var_449;
        wp::int32 var_450;
        const wp::int32 var_451 = 0;
        wp::int32 var_452;
        const wp::int32 var_453 = 0;
        bool var_454;
        const wp::int32 var_455 = 1;
        wp::int32 var_456;
        wp::int32* var_457;
        wp::int32 var_458;
        wp::int32 var_459;
        wp::int32 var_460;
        const wp::float32 var_461 = 0.0;
        wp::int32 var_462;
        const wp::int32 var_463 = 1;
        wp::int32 var_464;
        const wp::int32 var_465 = 0;
        bool var_466;
        wp::vec_t<3, wp::float32>* var_467;
        wp::int32* var_468;
        wp::vec_t<3, wp::float32> var_469;
        wp::vec_t<3, wp::float32> var_470;
        wp::vec_t<3, wp::float32> var_471;
        wp::int32 var_472;
        wp::float32 var_473;
        wp::float32 var_474;
        wp::float32 var_475;
        wp::int32 var_476;
        const wp::int32 var_477 = 1;
        wp::int32 var_478;
        wp::int32* var_479;
        wp::int32 var_480;
        wp::int32 var_481;
        wp::int32 var_482;
        wp::mat_t<3, 3, wp::float32> var_483;
        wp::int32 var_484;
        wp::int32 var_485;
        wp::int32 var_486;
        wp::int32 var_487;
        wp::int32* var_488;
        wp::int32 var_489;
        wp::int32 var_490;
        wp::int32* var_491;
        wp::int32 var_492;
        wp::int32 var_493;
        wp::int32* var_494;
        wp::int32 var_495;
        wp::int32 var_496;
        wp::int32* var_497;
        wp::int32 var_498;
        wp::int32 var_499;
        wp::int32* var_500;
        wp::int32* var_501;
        wp::int32 var_502;
        wp::int32 var_503;
        wp::int32 var_504;
        const wp::int32 var_505 = 1;
        wp::int32 var_506;
        wp::int32* var_507;
        wp::int32* var_508;
        wp::int32 var_509;
        wp::int32 var_510;
        wp::int32 var_511;
        const wp::int32 var_512 = 1;
        wp::int32 var_513;
        const wp::int32 var_514 = -1;
        bool var_515;
        const wp::int32 var_516 = 0;
        bool var_517;
        const wp::int32 var_518 = 0;
        bool var_519;
        bool var_520;
        bool var_521;
        wp::int32* var_522;
        wp::int32 var_523;
        wp::int32 var_524;
        wp::int32 var_525;
        wp::int32* var_526;
        wp::int32 var_527;
        wp::int32 var_528;
        wp::int32 var_529;
        bool var_530;
        const wp::int32 var_531 = -1;
        bool var_532;
        const wp::int32 var_533 = -1;
        bool var_534;
        wp::int32 var_535;
        wp::int32 var_536;
        bool var_537;
        wp::int32 var_538;
        wp::int32 var_539;
        wp::int32 var_540;
        bool var_541;
        const wp::int32 var_542 = 0;
        wp::float32 var_543;
        const wp::float32 var_544 = 0.0;
        bool var_545;
        const wp::int32 var_546 = 1;
        wp::float32 var_547;
        const wp::float32 var_548 = 0.0;
        bool var_549;
        const wp::int32 var_550 = 2;
        wp::float32 var_551;
        const wp::float32 var_552 = 0.0;
        bool var_553;
        bool var_554;
        bool var_555;
        const wp::int32 var_556 = 3;
        wp::float32 var_557;
        const wp::float32 var_558 = 0.0;
        bool var_559;
        const wp::int32 var_560 = 4;
        wp::float32 var_561;
        const wp::float32 var_562 = 0.0;
        bool var_563;
        const wp::int32 var_564 = 5;
        wp::float32 var_565;
        const wp::float32 var_566 = 0.0;
        bool var_567;
        bool var_568;
        wp::vec_t<3, wp::float32>* var_569;
        wp::vec_t<3, wp::float32> var_570;
        wp::vec_t<3, wp::float32> var_571;
        wp::vec_t<3, wp::float32>* var_572;
        wp::vec_t<3, wp::float32> var_573;
        wp::vec_t<3, wp::float32> var_574;
        wp::mat_t<3, 3, wp::float32>* var_575;
        wp::mat_t<3, 3, wp::float32> var_576;
        wp::mat_t<3, 3, wp::float32> var_577;
        const wp::float32 var_578 = 0.0;
        wp::float32 var_579;
        wp::mat_t<3, 3, wp::float32> var_580;
        wp::vec_t<3, wp::float32> var_581;
        wp::vec_t<3, wp::float32> var_582;
        wp::float32 var_583;
        wp::float32 var_584;
        wp::vec_t<3, wp::float32> var_585;
        wp::vec_t<3, wp::float32> var_586;
        wp::float32 var_587;
        wp::vec_t<3, wp::float32> var_588;
        wp::quat_t<wp::float32>* var_589;
        wp::quat_t<wp::float32>* var_590;
        wp::quat_t<wp::float32> var_591;
        wp::quat_t<wp::float32> var_592;
        wp::quat_t<wp::float32> var_593;
        wp::quat_t<wp::float32>* var_594;
        wp::quat_t<wp::float32>* var_595;
        wp::quat_t<wp::float32> var_596;
        wp::quat_t<wp::float32> var_597;
        wp::quat_t<wp::float32> var_598;
        wp::vec_t<3, wp::float32> var_599;
        wp::float32 var_600;
        wp::float32 var_601;
        wp::vec_t<3, wp::float32> var_602;
        wp::quat_t<wp::float32> var_603;
        wp::vec_t<3, wp::float32> var_604;
        wp::float32 var_605;
        wp::vec_t<3, wp::float32> var_606;
        const wp::int32 var_607 = -1;
        wp::int32 var_608;
        const wp::int32 var_609 = -1;
        wp::int32 var_610;
        const wp::int32 var_611 = 0;
        bool var_612;
        wp::int32* var_613;
        wp::int32* var_614;
        wp::int32 var_615;
        wp::int32 var_616;
        wp::int32 var_617;
        const wp::int32 var_618 = 1;
        wp::int32 var_619;
        wp::int32 var_620;
        const wp::int32 var_621 = 0;
        bool var_622;
        wp::int32* var_623;
        wp::int32* var_624;
        wp::int32 var_625;
        wp::int32 var_626;
        wp::int32 var_627;
        const wp::int32 var_628 = 1;
        wp::int32 var_629;
        wp::int32 var_630;
        wp::int32 var_631;
        wp::int32 var_632;
        const wp::int32 var_633 = 0;
        wp::int32 var_634;
        bool var_635;
        const wp::int32 var_636 = 0;
        bool var_637;
        const wp::int32 var_638 = 0;
        bool var_639;
        wp::int32 var_640;
        bool var_641;
        bool var_642;
        bool var_643;
        wp::int32 var_644;
        const wp::int32 var_645 = 1;
        wp::int32 var_646;
        bool var_647;
        wp::int32* var_648;
        wp::int32 var_649;
        wp::int32 var_650;
        wp::int32 var_651;
        bool var_652;
        wp::int32* var_653;
        wp::int32 var_654;
        wp::int32 var_655;
        wp::int32 var_656;
        wp::int32 var_657;
        wp::int32 var_658;
        wp::int32 var_659;
        const wp::int32 var_660 = 1;
        wp::int32 var_661;
        bool var_662;
        const wp::int32 var_663 = 0;
        bool var_664;
        const wp::int32 var_665 = 0;
        bool var_666;
        wp::int32 var_667;
        bool var_668;
        bool var_669;
        bool var_670;
        wp::int32 var_671;
        wp::int32* var_672;
        wp::vec_t<3, wp::float32> var_673;
        wp::vec_t<3, wp::float32> var_674;
        wp::int32 var_675;
        wp::int32* var_676;
        wp::vec_t<3, wp::float32> var_677;
        wp::vec_t<3, wp::float32> var_678;
        wp::int32 var_679;
        const wp::float32 var_680 = 0.0;
        wp::float32 var_681;
        wp::vec_t<3, wp::float32> var_682;
        wp::float32 var_683;
        wp::float32 var_684;
        wp::float32 var_685;
        wp::vec_t<3, wp::float32> var_686;
        wp::float32 var_687;
        wp::float32 var_688;
        wp::float32 var_689;
        wp::int32 var_690;
        const wp::int32 var_691 = 1;
        wp::int32 var_692;
        bool var_693;
        wp::int32* var_694;
        wp::int32 var_695;
        wp::int32 var_696;
        wp::int32 var_697;
        bool var_698;
        wp::int32* var_699;
        wp::int32 var_700;
        wp::int32 var_701;
        wp::int32 var_702;
        wp::int32 var_703;
        wp::quat_t<wp::float32> var_704;
        wp::vec_t<3, wp::float32> var_705;
        wp::float32 var_706;
        wp::int32 var_707;
        wp::int32 var_708;
        wp::int32 var_709;
        wp::int32 var_710;
        wp::int32 var_711;
        wp::int32 var_712;
        wp::int32 var_713;
        wp::vec_t<3, wp::float32> var_714;
        wp::vec_t<3, wp::float32> var_715;
        wp::vec_t<6, wp::float32> var_716;
        wp::int32 var_717;
        wp::quat_t<wp::float32> var_718;
        wp::vec_t<2, wp::int32> var_719;
        wp::mat_t<3, 3, wp::float32> var_720;
        wp::vec_t<3, wp::float32> var_721;
        wp::float32 var_722;
        wp::int32 var_723;
        wp::int32 var_724;
        wp::int32 var_725;
        wp::int32 var_726;
        wp::int32 var_727;
        wp::int32 var_728;
        wp::int32 var_729;
        wp::int32 var_730;
        const wp::str var_731 = "unhandled transmission type %d\n";
        wp::vec_t<6, wp::float32> var_732;
        wp::int32 var_733;
        wp::quat_t<wp::float32> var_734;
        wp::vec_t<2, wp::int32> var_735;
        wp::mat_t<3, 3, wp::float32> var_736;
        wp::vec_t<3, wp::float32> var_737;
        wp::float32 var_738;
        wp::int32 var_739;
        wp::int32 var_740;
        wp::int32 var_741;
        wp::int32 var_742;
        wp::int32 var_743;
        wp::int32 var_744;
        wp::int32 var_745;
        wp::int32 var_746;
        wp::vec_t<6, wp::float32> var_747;
        wp::int32 var_748;
        wp::quat_t<wp::float32> var_749;
        wp::int32 var_750;
        wp::vec_t<2, wp::int32> var_751;
        wp::mat_t<3, 3, wp::float32> var_752;
        wp::vec_t<3, wp::float32> var_753;
        wp::float32 var_754;
        wp::int32 var_755;
        wp::int32 var_756;
        wp::int32 var_757;
        wp::int32 var_758;
        wp::int32 var_759;
        wp::int32 var_760;
        wp::int32 var_761;
        wp::int32 var_762;
        wp::vec_t<6, wp::float32> var_763;
        wp::int32 var_764;
        wp::quat_t<wp::float32> var_765;
        wp::int32 var_766;
        wp::vec_t<2, wp::int32> var_767;
        wp::float32 var_768;
        wp::mat_t<3, 3, wp::float32> var_769;
        wp::vec_t<3, wp::float32> var_770;
        wp::float32 var_771;
        wp::int32 var_772;
        wp::int32 var_773;
        wp::int32 var_774;
        wp::int32 var_775;
        wp::int32 var_776;
        wp::int32 var_777;
        wp::int32 var_778;
        wp::int32 var_779;
        wp::vec_t<6, wp::float32> var_780;
        wp::int32 var_781;
        wp::quat_t<wp::float32> var_782;
        wp::int32 var_783;
        //---------
        // forward
        // def _transmission(                                                                     <L 2618>
        // worldid, actid = wp.tid()                                                              <L 2659>
        builtin_tid2d(var_0, var_1);
        // trntype = actuator_trntype[actid]                                                      <L 2660>
        var_2 = wp::address(var_actuator_trntype, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // actuator_gear_id = worldid % actuator_gear.shape[0]                                    <L 2661>
        var_5 = &(var_actuator_gear.shape);
        var_8 = wp::load(var_5);
        var_7 = wp::extract(var_8, var_6);
        var_9 = wp::mod(var_0, var_7);
        // gear = actuator_gear[actuator_gear_id, actid]                                          <L 2662>
        var_10 = wp::address(var_actuator_gear, var_9, var_1);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // if trntype == TrnType.JOINT or trntype == TrnType.JOINTINPARENT:                       <L 2663>
        var_15 = (var_3 == var_14);
        var_13 = var_15;
        if (!var_13) {
            var_17 = (var_3 == var_16);
            var_13 = var_13 || var_17;
        }
        if (var_13) {
            // qpos = qpos_in[worldid]                                                            <L 2664>
            var_18 = wp::slice_t(var_0, var_0, var_19);
            var_20 = wp::view(var_qpos_in, var_18);
            // jntid = actuator_trnid[actid][0]                                                   <L 2665>
            var_21 = wp::address(var_actuator_trnid, var_1);
            var_24 = wp::load(var_21);
            var_23 = wp::extract(var_24, var_22);
            // jnt_typ = jnt_type[jntid]                                                          <L 2666>
            var_25 = wp::address(var_jnt_type, var_23);
            var_27 = wp::load(var_25);
            var_26 = wp::copy(var_27);
            // qadr = jnt_qposadr[jntid]                                                          <L 2667>
            var_28 = wp::address(var_jnt_qposadr, var_23);
            var_30 = wp::load(var_28);
            var_29 = wp::copy(var_30);
            // vadr = jnt_dofadr[jntid]                                                           <L 2668>
            var_31 = wp::address(var_jnt_dofadr, var_23);
            var_33 = wp::load(var_31);
            var_32 = wp::copy(var_33);
            // if jnt_typ == JointType.FREE:                                                      <L 2669>
            var_35 = (var_26 == var_34);
            if (var_35) {
                // moment_rownnz_out[worldid, actid] = 6                                          <L 2670>
                wp::array_store(var_moment_rownnz_out, var_0, var_1, var_36);
                // rowadr = wp.atomic_add(moment_nnz, worldid, 6)                                 <L 2671>
                var_38 = wp::atomic_add(var_moment_nnz, var_0, var_37);
                // moment_rowadr_out[worldid, actid] = rowadr                                     <L 2672>
                wp::array_store(var_moment_rowadr_out, var_0, var_1, var_38);
                // moment_colind_out[worldid, rowadr + 0] = vadr + 0                              <L 2673>
                var_40 = wp::add(var_32, var_39);
                var_42 = wp::add(var_38, var_41);
                wp::array_store(var_moment_colind_out, var_0, var_42, var_40);
                // moment_colind_out[worldid, rowadr + 1] = vadr + 1                              <L 2674>
                var_44 = wp::add(var_32, var_43);
                var_46 = wp::add(var_38, var_45);
                wp::array_store(var_moment_colind_out, var_0, var_46, var_44);
                // moment_colind_out[worldid, rowadr + 2] = vadr + 2                              <L 2675>
                var_48 = wp::add(var_32, var_47);
                var_50 = wp::add(var_38, var_49);
                wp::array_store(var_moment_colind_out, var_0, var_50, var_48);
                // moment_colind_out[worldid, rowadr + 3] = vadr + 3                              <L 2676>
                var_52 = wp::add(var_32, var_51);
                var_54 = wp::add(var_38, var_53);
                wp::array_store(var_moment_colind_out, var_0, var_54, var_52);
                // moment_colind_out[worldid, rowadr + 4] = vadr + 4                              <L 2677>
                var_56 = wp::add(var_32, var_55);
                var_58 = wp::add(var_38, var_57);
                wp::array_store(var_moment_colind_out, var_0, var_58, var_56);
                // moment_colind_out[worldid, rowadr + 5] = vadr + 5                              <L 2678>
                var_60 = wp::add(var_32, var_59);
                var_62 = wp::add(var_38, var_61);
                wp::array_store(var_moment_colind_out, var_0, var_62, var_60);
                // actuator_length_out[worldid, actid] = 0.0                                      <L 2679>
                wp::array_store(var_actuator_length_out, var_0, var_1, var_63);
                // if trntype == TrnType.JOINTINPARENT:                                           <L 2680>
                var_65 = (var_3 == var_64);
                if (var_65) {
                    // quat = wp.normalize(wp.quat(qpos[qadr + 3], qpos[qadr + 4], qpos[qadr + 5], qpos[qadr + 6]))       <L 2681>
                    var_67 = wp::add(var_29, var_66);
                    var_68 = wp::address(var_20, var_67);
                    var_70 = wp::add(var_29, var_69);
                    var_71 = wp::address(var_20, var_70);
                    var_73 = wp::add(var_29, var_72);
                    var_74 = wp::address(var_20, var_73);
                    var_76 = wp::add(var_29, var_75);
                    var_77 = wp::address(var_20, var_76);
                    var_79 = wp::load(var_68);
                    var_80 = wp::load(var_71);
                    var_81 = wp::load(var_74);
                    var_82 = wp::load(var_77);
                    var_78 = wp::quat_t<wp::float32>(var_79, var_80, var_81, var_82);
                    var_83 = wp::normalize(var_78);
                    // quat_neg = math.quat_inv(quat)                                             <L 2682>
                    var_84 = quat_inv_0(var_83);
                    // gearaxis = math.rot_vec_quat(wp.spatial_bottom(gear), quat_neg)            <L 2683>
                    var_85 = wp::spatial_bottom(var_11);
                    var_86 = rot_vec_quat_0(var_85, var_84);
                    // actuator_moment_out[worldid, rowadr + 0] = gear[0]                         <L 2684>
                    var_88 = wp::extract(var_11, var_87);
                    var_90 = wp::add(var_38, var_89);
                    wp::array_store(var_actuator_moment_out, var_0, var_90, var_88);
                    // actuator_moment_out[worldid, rowadr + 1] = gear[1]                         <L 2685>
                    var_92 = wp::extract(var_11, var_91);
                    var_94 = wp::add(var_38, var_93);
                    wp::array_store(var_actuator_moment_out, var_0, var_94, var_92);
                    // actuator_moment_out[worldid, rowadr + 2] = gear[2]                         <L 2686>
                    var_96 = wp::extract(var_11, var_95);
                    var_98 = wp::add(var_38, var_97);
                    wp::array_store(var_actuator_moment_out, var_0, var_98, var_96);
                    // actuator_moment_out[worldid, rowadr + 3] = gearaxis[0]                     <L 2687>
                    var_100 = wp::extract(var_86, var_99);
                    var_102 = wp::add(var_38, var_101);
                    wp::array_store(var_actuator_moment_out, var_0, var_102, var_100);
                    // actuator_moment_out[worldid, rowadr + 4] = gearaxis[1]                     <L 2688>
                    var_104 = wp::extract(var_86, var_103);
                    var_106 = wp::add(var_38, var_105);
                    wp::array_store(var_actuator_moment_out, var_0, var_106, var_104);
                    // actuator_moment_out[worldid, rowadr + 5] = gearaxis[2]                     <L 2689>
                    var_108 = wp::extract(var_86, var_107);
                    var_110 = wp::add(var_38, var_109);
                    wp::array_store(var_actuator_moment_out, var_0, var_110, var_108);
                }
                if (!var_65) {
                    // actuator_moment_out[worldid, rowadr + 0] = gear[0]                         <L 2691>
                    var_112 = wp::extract(var_11, var_111);
                    var_114 = wp::add(var_38, var_113);
                    wp::array_store(var_actuator_moment_out, var_0, var_114, var_112);
                    // actuator_moment_out[worldid, rowadr + 1] = gear[1]                         <L 2692>
                    var_116 = wp::extract(var_11, var_115);
                    var_118 = wp::add(var_38, var_117);
                    wp::array_store(var_actuator_moment_out, var_0, var_118, var_116);
                    // actuator_moment_out[worldid, rowadr + 2] = gear[2]                         <L 2693>
                    var_120 = wp::extract(var_11, var_119);
                    var_122 = wp::add(var_38, var_121);
                    wp::array_store(var_actuator_moment_out, var_0, var_122, var_120);
                    // actuator_moment_out[worldid, rowadr + 3] = gear[3]                         <L 2694>
                    var_124 = wp::extract(var_11, var_123);
                    var_126 = wp::add(var_38, var_125);
                    wp::array_store(var_actuator_moment_out, var_0, var_126, var_124);
                    // actuator_moment_out[worldid, rowadr + 4] = gear[4]                         <L 2695>
                    var_128 = wp::extract(var_11, var_127);
                    var_130 = wp::add(var_38, var_129);
                    wp::array_store(var_actuator_moment_out, var_0, var_130, var_128);
                    // actuator_moment_out[worldid, rowadr + 5] = gear[5]                         <L 2696>
                    var_132 = wp::extract(var_11, var_131);
                    var_134 = wp::add(var_38, var_133);
                    wp::array_store(var_actuator_moment_out, var_0, var_134, var_132);
                }
            }
            if (!var_35) {
                // elif jnt_typ == JointType.BALL:                                                <L 2697>
                var_136 = (var_26 == var_135);
                if (var_136) {
                    // q = wp.quat(qpos[qadr + 0], qpos[qadr + 1], qpos[qadr + 2], qpos[qadr + 3])       <L 2698>
                    var_138 = wp::add(var_29, var_137);
                    var_139 = wp::address(var_20, var_138);
                    var_141 = wp::add(var_29, var_140);
                    var_142 = wp::address(var_20, var_141);
                    var_144 = wp::add(var_29, var_143);
                    var_145 = wp::address(var_20, var_144);
                    var_147 = wp::add(var_29, var_146);
                    var_148 = wp::address(var_20, var_147);
                    var_150 = wp::load(var_139);
                    var_151 = wp::load(var_142);
                    var_152 = wp::load(var_145);
                    var_153 = wp::load(var_148);
                    var_149 = wp::quat_t<wp::float32>(var_150, var_151, var_152, var_153);
                    // q = wp.normalize(q)                                                        <L 2699>
                    var_154 = wp::normalize(var_149);
                    // axis_angle = math.quat_to_vel(q)                                           <L 2700>
                    var_155 = quat_to_vel_0(var_154);
                    // gearaxis = wp.spatial_top(gear)  # [:3]                                    <L 2701>
                    var_156 = wp::spatial_top(var_11);
                    // if trntype == TrnType.JOINTINPARENT:                                       <L 2702>
                    var_158 = (var_3 == var_157);
                    if (var_158) {
                        // quat_neg = math.quat_inv(q)                                            <L 2703>
                        var_159 = quat_inv_0(var_154);
                        // gearaxis = math.rot_vec_quat(gearaxis, quat_neg)                       <L 2704>
                        var_160 = rot_vec_quat_0(var_156, var_159);
                    }
                    var_161 = wp::where(var_158, var_159, var_84);
                    var_162 = wp::where(var_158, var_160, var_156);
                    // actuator_length_out[worldid, actid] = wp.dot(axis_angle, gearaxis)         <L 2705>
                    var_163 = wp::dot(var_155, var_162);
                    wp::array_store(var_actuator_length_out, var_0, var_1, var_163);
                    // nnz = 3                                                                    <L 2707>
                    // moment_rownnz_out[worldid, actid] = nnz                                    <L 2708>
                    wp::array_store(var_moment_rownnz_out, var_0, var_1, var_164);
                    // rowadr = wp.atomic_add(moment_nnz, worldid, nnz)                           <L 2709>
                    var_165 = wp::atomic_add(var_moment_nnz, var_0, var_164);
                    // moment_rowadr_out[worldid, actid] = rowadr                                 <L 2710>
                    wp::array_store(var_moment_rowadr_out, var_0, var_1, var_165);
                    // for i in range(3):                                                         <L 2712>
                    // sparseid = rowadr + i                                                      <L 2713>
                    var_167 = wp::add(var_165, var_166);
                    // moment_colind_out[worldid, sparseid] = vadr + i                            <L 2714>
                    var_168 = wp::add(var_32, var_166);
                    wp::array_store(var_moment_colind_out, var_0, var_167, var_168);
                    // actuator_moment_out[worldid, sparseid] = gearaxis[i]                       <L 2715>
                    var_169 = wp::extract(var_162, var_166);
                    wp::array_store(var_actuator_moment_out, var_0, var_167, var_169);
                    // sparseid = rowadr + i                                                      <L 2713>
                    var_171 = wp::add(var_165, var_170);
                    // moment_colind_out[worldid, sparseid] = vadr + i                            <L 2714>
                    var_172 = wp::add(var_32, var_170);
                    wp::array_store(var_moment_colind_out, var_0, var_171, var_172);
                    // actuator_moment_out[worldid, sparseid] = gearaxis[i]                       <L 2715>
                    var_173 = wp::extract(var_162, var_170);
                    wp::array_store(var_actuator_moment_out, var_0, var_171, var_173);
                    // sparseid = rowadr + i                                                      <L 2713>
                    var_175 = wp::add(var_165, var_174);
                    // moment_colind_out[worldid, sparseid] = vadr + i                            <L 2714>
                    var_176 = wp::add(var_32, var_174);
                    wp::array_store(var_moment_colind_out, var_0, var_175, var_176);
                    // actuator_moment_out[worldid, sparseid] = gearaxis[i]                       <L 2715>
                    var_177 = wp::extract(var_162, var_174);
                    wp::array_store(var_actuator_moment_out, var_0, var_175, var_177);
                }
                var_178 = wp::where(var_136, var_165, var_38);
                var_179 = wp::where(var_136, var_161, var_84);
                var_180 = wp::where(var_136, var_162, var_86);
                if (!var_136) {
                    // elif jnt_typ == JointType.SLIDE or jnt_typ == JointType.HINGE:             <L 2716>
                    var_183 = (var_26 == var_182);
                    var_181 = var_183;
                    if (!var_181) {
                        var_185 = (var_26 == var_184);
                        var_181 = var_181 || var_185;
                    }
                    if (var_181) {
                        // actuator_length_out[worldid, actid] = qpos[qadr] * gear[0]             <L 2717>
                        var_186 = wp::address(var_20, var_29);
                        var_188 = wp::extract(var_11, var_187);
                        var_190 = wp::load(var_186);
                        var_189 = wp::mul(var_190, var_188);
                        wp::array_store(var_actuator_length_out, var_0, var_1, var_189);
                        // nnz = 1                                                                <L 2719>
                        // moment_rownnz_out[worldid, actid] = nnz                                <L 2720>
                        wp::array_store(var_moment_rownnz_out, var_0, var_1, var_191);
                        // rowadr = wp.atomic_add(moment_nnz, worldid, nnz)                       <L 2721>
                        var_192 = wp::atomic_add(var_moment_nnz, var_0, var_191);
                        // moment_rowadr_out[worldid, actid] = rowadr                             <L 2722>
                        wp::array_store(var_moment_rowadr_out, var_0, var_1, var_192);
                        // moment_colind_out[worldid, rowadr] = vadr                              <L 2723>
                        wp::array_store(var_moment_colind_out, var_0, var_192, var_32);
                        // actuator_moment_out[worldid, rowadr] = gear[0]                         <L 2724>
                        var_194 = wp::extract(var_11, var_193);
                        wp::array_store(var_actuator_moment_out, var_0, var_192, var_194);
                    }
                    var_195 = wp::where(var_181, var_192, var_178);
                    var_196 = wp::where(var_181, var_191, var_164);
                    if (!var_181) {
                        // wp.printf("unrecognized joint type")                                   <L 2726>
                        printf(var_197);
                    }
                }
                var_198 = wp::where(var_136, var_178, var_195);
                var_199 = wp::where(var_136, var_164, var_196);
            }
            var_200 = wp::where(var_35, var_38, var_198);
            var_201 = wp::where(var_35, var_84, var_179);
            var_202 = wp::where(var_35, var_86, var_180);
        }
        if (!var_13) {
            // elif trntype == TrnType.SLIDERCRANK:                                               <L 2727>
            var_204 = (var_3 == var_203);
            if (var_204) {
                // trnid = actuator_trnid[actid]                                                  <L 2729>
                var_205 = wp::address(var_actuator_trnid, var_1);
                var_207 = wp::load(var_205);
                var_206 = wp::copy(var_207);
                // id = trnid[0]                                                                  <L 2730>
                var_209 = wp::extract(var_206, var_208);
                // idslider = trnid[1]                                                            <L 2731>
                var_211 = wp::extract(var_206, var_210);
                // gear0 = gear[0]                                                                <L 2732>
                var_213 = wp::extract(var_11, var_212);
                // rod = actuator_cranklength[worldid % actuator_cranklength.shape[0], actid]       <L 2733>
                var_214 = &(var_actuator_cranklength.shape);
                var_217 = wp::load(var_214);
                var_216 = wp::extract(var_217, var_215);
                var_218 = wp::mod(var_0, var_216);
                var_219 = wp::address(var_actuator_cranklength, var_218, var_1);
                var_221 = wp::load(var_219);
                var_220 = wp::copy(var_221);
                // site_xmat = site_xmat_in[worldid, idslider]                                    <L 2734>
                var_222 = wp::address(var_site_xmat_in, var_0, var_211);
                var_224 = wp::load(var_222);
                var_223 = wp::copy(var_224);
                // axis = wp.vec3(site_xmat[0, 2], site_xmat[1, 2], site_xmat[2, 2])              <L 2735>
                var_227 = wp::extract(var_223, var_225, var_226);
                var_230 = wp::extract(var_223, var_228, var_229);
                var_233 = wp::extract(var_223, var_231, var_232);
                var_234 = wp::vec_t<3, wp::float32>(var_227, var_230, var_233);
                // site_xpos_id = site_xpos_in[worldid, id]                                       <L 2736>
                var_235 = wp::address(var_site_xpos_in, var_0, var_209);
                var_237 = wp::load(var_235);
                var_236 = wp::copy(var_237);
                // site_xpos_idslider = site_xpos_in[worldid, idslider]                           <L 2737>
                var_238 = wp::address(var_site_xpos_in, var_0, var_211);
                var_240 = wp::load(var_238);
                var_239 = wp::copy(var_240);
                // vec = site_xpos_id - site_xpos_idslider                                        <L 2738>
                var_241 = wp::sub(var_236, var_239);
                // av = wp.dot(vec, axis)                                                         <L 2742>
                var_242 = wp::dot(var_241, var_234);
                // det = av * av + rod * rod - wp.dot(vec, vec)                                   <L 2743>
                var_243 = wp::mul(var_242, var_242);
                var_244 = wp::mul(var_220, var_220);
                var_245 = wp::add(var_243, var_244);
                var_246 = wp::dot(var_241, var_241);
                var_247 = wp::sub(var_245, var_246);
                // ok = 1                                                                         <L 2744>
                // if det <= 0.0:                                                                 <L 2745>
                var_250 = (var_247 <= var_249);
                if (var_250) {
                    // ok = 0                                                                     <L 2746>
                    // sdet = 0.0                                                                 <L 2747>
                    // length = av                                                                <L 2748>
                    var_253 = wp::copy(var_242);
                }
                var_254 = wp::where(var_250, var_251, var_248);
                if (!var_250) {
                    // sdet = wp.sqrt(det)                                                        <L 2750>
                    var_255 = wp::sqrt(var_247);
                    // length = av - sdet                                                         <L 2751>
                    var_256 = wp::sub(var_242, var_255);
                }
                var_257 = wp::where(var_250, var_252, var_255);
                var_258 = wp::where(var_250, var_253, var_256);
                // actuator_length_out[worldid, actid] = length * gear0                           <L 2753>
                var_259 = wp::mul(var_258, var_213);
                wp::array_store(var_actuator_length_out, var_0, var_1, var_259);
                // if ok == 1:                                                                    <L 2756>
                var_261 = (var_254 == var_260);
                if (var_261) {
                    // scale = 1.0 - math.safe_div(av, sdet)                                      <L 2757>
                    var_263 = safe_div_0(var_242, var_257);
                    var_264 = wp::sub(var_262, var_263);
                    // dldv = axis * scale + math.safe_div(vec, sdet)                             <L 2758>
                    var_265 = wp::mul(var_234, var_264);
                    var_266 = safe_div_0(var_241, var_257);
                    var_267 = wp::add(var_265, var_266);
                    // dlda = vec * scale                                                         <L 2759>
                    var_268 = wp::mul(var_241, var_264);
                }
                if (!var_261) {
                    // dldv = axis                                                                <L 2761>
                    var_269 = wp::copy(var_234);
                    // dlda = vec                                                                 <L 2762>
                    var_270 = wp::copy(var_241);
                }
                var_271 = wp::where(var_261, var_267, var_269);
                var_272 = wp::where(var_261, var_268, var_270);
                // b1 = body_weldid[site_bodyid[id]]                                              <L 2765>
                var_273 = wp::address(var_site_bodyid, var_209);
                var_275 = wp::load(var_273);
                var_274 = wp::address(var_body_weldid, var_275);
                var_277 = wp::load(var_274);
                var_276 = wp::copy(var_277);
                // b2 = body_weldid[site_bodyid[idslider]]                                        <L 2766>
                var_278 = wp::address(var_site_bodyid, var_211);
                var_280 = wp::load(var_278);
                var_279 = wp::address(var_body_weldid, var_280);
                var_282 = wp::load(var_279);
                var_281 = wp::copy(var_282);
                // da1_init = int(-1)                                                             <L 2767>
                var_284 = wp::int(var_283);
                // da2_init = int(-1)                                                             <L 2768>
                var_286 = wp::int(var_285);
                // if b1 > 0:                                                                     <L 2769>
                var_288 = (var_276 > var_287);
                if (var_288) {
                    // da1_init = body_dofadr[b1] + body_dofnum[b1] - 1                           <L 2770>
                    var_289 = wp::address(var_body_dofadr, var_276);
                    var_290 = wp::address(var_body_dofnum, var_276);
                    var_292 = wp::load(var_289);
                    var_293 = wp::load(var_290);
                    var_291 = wp::add(var_292, var_293);
                    var_295 = wp::sub(var_291, var_294);
                }
                var_296 = wp::where(var_288, var_295, var_284);
                // if b2 > 0:                                                                     <L 2771>
                var_298 = (var_281 > var_297);
                if (var_298) {
                    // da2_init = body_dofadr[b2] + body_dofnum[b2] - 1                           <L 2772>
                    var_299 = wp::address(var_body_dofadr, var_281);
                    var_300 = wp::address(var_body_dofnum, var_281);
                    var_302 = wp::load(var_299);
                    var_303 = wp::load(var_300);
                    var_301 = wp::add(var_302, var_303);
                    var_305 = wp::sub(var_301, var_304);
                }
                var_306 = wp::where(var_298, var_305, var_286);
                // da1 = da1_init                                                                 <L 2774>
                var_307 = wp::copy(var_296);
                // da2 = da2_init                                                                 <L 2775>
                var_308 = wp::copy(var_306);
                // ndof = int(0)                                                                  <L 2776>
                var_310 = wp::int(var_309);
                // while da1 >= 0 or da2 >= 0:                                                    <L 2777>
        start_while_0:;
                var_313 = (var_307 >= var_312);
                var_311 = var_313;
                if (!var_311) {
                    var_315 = (var_308 >= var_314);
                    var_311 = var_311 || var_315;
                }
        if ((var_311) == false) goto end_while_0;
                    // da = wp.max(da1, da2)                                                      <L 2778>
                    var_316 = wp::max(var_307, var_308);
                    // ndof += 1                                                                  <L 2779>
                    var_318 = wp::add(var_310, var_317);
                    // if da1 == da:                                                              <L 2780>
                    var_319 = (var_307 == var_316);
                    if (var_319) {
                        // da1 = dof_parentid[da1]                                                <L 2781>
                        var_320 = wp::address(var_dof_parentid, var_307);
                        var_322 = wp::load(var_320);
                        var_321 = wp::copy(var_322);
                    }
                    var_323 = wp::where(var_319, var_321, var_307);
                    // if da2 == da:                                                              <L 2782>
                    var_324 = (var_308 == var_316);
                    if (var_324) {
                        // da2 = dof_parentid[da2]                                                <L 2783>
                        var_325 = wp::address(var_dof_parentid, var_308);
                        var_327 = wp::load(var_325);
                        var_326 = wp::copy(var_327);
                    }
                    var_328 = wp::where(var_324, var_326, var_308);
                    wp::assign(var_307, var_323);
                    wp::assign(var_308, var_328);
                    wp::assign(var_310, var_318);
        goto start_while_0;
        end_while_0:;
                // moment_rownnz_out[worldid, actid] = ndof                                       <L 2785>
                wp::array_store(var_moment_rownnz_out, var_0, var_1, var_310);
                // rowadr = wp.atomic_add(moment_nnz, worldid, ndof)                              <L 2786>
                var_329 = wp::atomic_add(var_moment_nnz, var_0, var_310);
                // moment_rowadr_out[worldid, actid] = rowadr                                     <L 2787>
                wp::array_store(var_moment_rowadr_out, var_0, var_1, var_329);
                // da1 = da1_init                                                                 <L 2790>
                var_330 = wp::copy(var_296);
                // da2 = da2_init                                                                 <L 2791>
                var_331 = wp::copy(var_306);
                // ptr = ndof - 1                                                                 <L 2793>
                var_333 = wp::sub(var_310, var_332);
                // while da1 >= 0 or da2 >= 0:                                                    <L 2794>
        start_while_2:;
                var_336 = (var_330 >= var_335);
                var_334 = var_336;
                if (!var_334) {
                    var_338 = (var_331 >= var_337);
                    var_334 = var_334 || var_338;
                }
        if ((var_334) == false) goto end_while_2;
                    // da = wp.max(da1, da2)                                                      <L 2795>
                    var_339 = wp::max(var_330, var_331);
                    // jacp, jacr = support.jac_dof(                                              <L 2798>
                    // body_parentid,                                                             <L 2799>
                    // body_rootid,                                                               <L 2800>
                    // dof_bodyid,                                                                <L 2801>
                    // body_isdofancestor,                                                        <L 2802>
                    // subtree_com_in,                                                            <L 2803>
                    // cdof_in,                                                                   <L 2804>
                    // site_xpos_idslider,                                                        <L 2805>
                    // site_bodyid[idslider],                                                     <L 2806>
                    var_340 = wp::address(var_site_bodyid, var_211);
                    // da,                                                                        <L 2807>
                    // worldid,                                                                   <L 2808>
                    var_343 = wp::load(var_340);
                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_239, var_343, var_339, var_0, var_341, var_342);
                    // jacS = jacp                                                                <L 2810>
                    var_344 = wp::copy(var_341);
                    // jacA = wp.cross(jacr, axis)                                                <L 2811>
                    var_345 = wp::cross(var_342, var_234);
                    // jac, _ = support.jac_dof(                                                  <L 2812>
                    // body_parentid,                                                             <L 2813>
                    // body_rootid,                                                               <L 2814>
                    // dof_bodyid,                                                                <L 2815>
                    // body_isdofancestor,                                                        <L 2816>
                    // subtree_com_in,                                                            <L 2817>
                    // cdof_in,                                                                   <L 2818>
                    // site_xpos_id,                                                              <L 2819>
                    // site_bodyid[id],                                                           <L 2820>
                    var_346 = wp::address(var_site_bodyid, var_209);
                    // da,                                                                        <L 2821>
                    // worldid,                                                                   <L 2822>
                    var_349 = wp::load(var_346);
                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_236, var_349, var_339, var_0, var_347, var_348);
                    // jac -= jacS                                                                <L 2824>
                    var_350 = wp::sub(var_347, var_344);
                    // moment = wp.dot(dlda, jacA) + wp.dot(dldv, jac)                            <L 2827>
                    var_351 = wp::dot(var_272, var_345);
                    var_352 = wp::dot(var_271, var_350);
                    var_353 = wp::add(var_351, var_352);
                    // sparseid = rowadr + ptr                                                    <L 2828>
                    var_354 = wp::add(var_329, var_333);
                    // moment_colind_out[worldid, sparseid] = da                                  <L 2829>
                    wp::array_store(var_moment_colind_out, var_0, var_354, var_339);
                    // actuator_moment_out[worldid, sparseid] = moment * gear0                    <L 2830>
                    var_355 = wp::mul(var_353, var_213);
                    wp::array_store(var_actuator_moment_out, var_0, var_354, var_355);
                    // ptr -= 1                                                                   <L 2831>
                    var_357 = wp::sub(var_333, var_356);
                    // if da1 == da:                                                              <L 2833>
                    var_358 = (var_330 == var_339);
                    if (var_358) {
                        // da1 = dof_parentid[da1]                                                <L 2834>
                        var_359 = wp::address(var_dof_parentid, var_330);
                        var_361 = wp::load(var_359);
                        var_360 = wp::copy(var_361);
                    }
                    var_362 = wp::where(var_358, var_360, var_330);
                    // if da2 == da:                                                              <L 2835>
                    var_363 = (var_331 == var_339);
                    if (var_363) {
                        // da2 = dof_parentid[da2]                                                <L 2836>
                        var_364 = wp::address(var_dof_parentid, var_331);
                        var_366 = wp::load(var_364);
                        var_365 = wp::copy(var_366);
                    }
                    var_367 = wp::where(var_363, var_365, var_331);
                    wp::assign(var_175, var_354);
                    wp::assign(var_330, var_362);
                    wp::assign(var_331, var_367);
                    wp::assign(var_316, var_339);
                    wp::assign(var_333, var_357);
        goto start_while_2;
        end_while_2:;
            }
            var_368 = wp::where(var_204, var_329, var_200);
            if (!var_204) {
                // elif trntype == TrnType.TENDON:                                                <L 2837>
                var_370 = (var_3 == var_369);
                if (var_370) {
                    // tenid = actuator_trnid[actid][0]                                           <L 2838>
                    var_371 = wp::address(var_actuator_trnid, var_1);
                    var_374 = wp::load(var_371);
                    var_373 = wp::extract(var_374, var_372);
                    // gear0 = gear[0]                                                            <L 2840>
                    var_376 = wp::extract(var_11, var_375);
                    // actuator_length_out[worldid, actid] = ten_length_in[worldid, tenid] * gear0       <L 2841>
                    var_377 = wp::address(var_ten_length_in, var_0, var_373);
                    var_379 = wp::load(var_377);
                    var_378 = wp::mul(var_379, var_376);
                    wp::array_store(var_actuator_length_out, var_0, var_1, var_378);
                    // rownnz_ten = ten_J_rownnz[tenid]                                           <L 2843>
                    var_380 = wp::address(var_ten_J_rownnz, var_373);
                    var_382 = wp::load(var_380);
                    var_381 = wp::copy(var_382);
                    // rowadr_ten = ten_J_rowadr[tenid]                                           <L 2844>
                    var_383 = wp::address(var_ten_J_rowadr, var_373);
                    var_385 = wp::load(var_383);
                    var_384 = wp::copy(var_385);
                    // rowadr_mom = wp.atomic_add(moment_nnz, worldid, rownnz_ten)                <L 2846>
                    var_386 = wp::atomic_add(var_moment_nnz, var_0, var_381);
                    // moment_rownnz_out[worldid, actid] = rownnz_ten                             <L 2847>
                    wp::array_store(var_moment_rownnz_out, var_0, var_1, var_381);
                    // moment_rowadr_out[worldid, actid] = rowadr_mom                             <L 2848>
                    wp::array_store(var_moment_rowadr_out, var_0, var_1, var_386);
                    // for k in range(rownnz_ten):                                                <L 2850>
                    var_387 = wp::range(var_381);
                    start_for_4:;
                        if (iter_cmp(var_387) == 0) goto end_for_4;
                        var_388 = wp::iter_next(var_387);
                        // sparseid_ten = rowadr_ten + k                                          <L 2851>
                        var_389 = wp::add(var_384, var_388);
                        // sparseid_mom = rowadr_mom + k                                          <L 2852>
                        var_390 = wp::add(var_386, var_388);
                        // moment_colind_out[worldid, sparseid_mom] = ten_J_colind[sparseid_ten]       <L 2853>
                        var_391 = wp::address(var_ten_J_colind, var_389);
                        var_392 = wp::load(var_391);
                        wp::array_store(var_moment_colind_out, var_0, var_390, var_392);
                        // actuator_moment_out[worldid, sparseid_mom] = ten_J_in[worldid, sparseid_ten] * gear0       <L 2854>
                        var_393 = wp::address(var_ten_J_in, var_0, var_389);
                        var_395 = wp::load(var_393);
                        var_394 = wp::mul(var_395, var_376);
                        wp::array_store(var_actuator_moment_out, var_0, var_390, var_394);
                        goto start_for_4;
                    end_for_4:;
                }
                var_396 = wp::where(var_370, var_376, var_213);
                if (!var_370) {
                    // elif trntype == TrnType.BODY:                                              <L 2855>
                    var_398 = (var_3 == var_397);
                    if (var_398) {
                        // actuator_length_out[worldid, actid] = 0.0                              <L 2857>
                        wp::array_store(var_actuator_length_out, var_0, var_1, var_399);
                        // rowadr = wp.atomic_add(moment_nnz, worldid, nv)                        <L 2860>
                        var_400 = wp::atomic_add(var_moment_nnz, var_0, var_nv);
                        // moment_rownnz_out[worldid, actid] = nv                                 <L 2861>
                        wp::array_store(var_moment_rownnz_out, var_0, var_1, var_nv);
                        // moment_rowadr_out[worldid, actid] = rowadr                             <L 2862>
                        wp::array_store(var_moment_rowadr_out, var_0, var_1, var_400);
                        // for i in range(nv):                                                    <L 2863>
                        var_401 = wp::range(var_nv);
                        start_for_6:;
                            if (iter_cmp(var_401) == 0) goto end_for_6;
                            var_402 = wp::iter_next(var_401);
                            // sparseid = rowadr + i                                              <L 2864>
                            var_403 = wp::add(var_400, var_402);
                            // moment_colind_out[worldid, sparseid] = i                           <L 2865>
                            wp::array_store(var_moment_colind_out, var_0, var_403, var_402);
                            // actuator_moment_out[worldid, sparseid] = 0.0                       <L 2866>
                            wp::array_store(var_actuator_moment_out, var_0, var_403, var_404);
                            wp::assign(var_175, var_403);
                            goto start_for_6;
                        end_for_6:;
                    }
                    var_405 = wp::where(var_398, var_400, var_368);
                    var_406 = wp::where(var_398, var_402, var_174);
                    if (!var_398) {
                        // elif trntype == TrnType.SITE:                                          <L 2869>
                        var_408 = (var_3 == var_407);
                        if (var_408) {
                            // trnid = actuator_trnid[actid]                                      <L 2870>
                            var_409 = wp::address(var_actuator_trnid, var_1);
                            var_411 = wp::load(var_409);
                            var_410 = wp::copy(var_411);
                            // siteid = trnid[0]                                                  <L 2871>
                            var_413 = wp::extract(var_410, var_412);
                            // refid = trnid[1]                                                   <L 2872>
                            var_415 = wp::extract(var_410, var_414);
                            // gear = actuator_gear[actuator_gear_id, actid]                      <L 2874>
                            var_416 = wp::address(var_actuator_gear, var_9, var_1);
                            var_418 = wp::load(var_416);
                            var_417 = wp::copy(var_418);
                            // site_quat_id = worldid % site_quat.shape[0]                        <L 2875>
                            var_419 = &(var_site_quat.shape);
                            var_422 = wp::load(var_419);
                            var_421 = wp::extract(var_422, var_420);
                            var_423 = wp::mod(var_0, var_421);
                            // gear_translation = wp.spatial_top(gear)                            <L 2876>
                            var_424 = wp::spatial_top(var_417);
                            // gear_rotational = wp.spatial_bottom(gear)                          <L 2877>
                            var_425 = wp::spatial_bottom(var_417);
                            // if refid == -1:                                                    <L 2880>
                            var_427 = (var_415 == var_426);
                            if (var_427) {
                                // site_xmat = site_xmat_in[worldid, siteid]                      <L 2882>
                                var_428 = wp::address(var_site_xmat_in, var_0, var_413);
                                var_430 = wp::load(var_428);
                                var_429 = wp::copy(var_430);
                                // wrench_translation = site_xmat @ gear_translation              <L 2883>
                                var_431 = wp::mul(var_429, var_424);
                                // wrench_rotation = site_xmat @ gear_rotational                  <L 2884>
                                var_432 = wp::mul(var_429, var_425);
                                // b1 = body_weldid[site_bodyid[siteid]]                          <L 2887>
                                var_433 = wp::address(var_site_bodyid, var_413);
                                var_435 = wp::load(var_433);
                                var_434 = wp::address(var_body_weldid, var_435);
                                var_437 = wp::load(var_434);
                                var_436 = wp::copy(var_437);
                                // da_init = int(-1)                                              <L 2888>
                                var_439 = wp::int(var_438);
                                // if b1 > 0:                                                     <L 2889>
                                var_441 = (var_436 > var_440);
                                if (var_441) {
                                    // da_init = body_dofadr[b1] + body_dofnum[b1] - 1            <L 2890>
                                    var_442 = wp::address(var_body_dofadr, var_436);
                                    var_443 = wp::address(var_body_dofnum, var_436);
                                    var_445 = wp::load(var_442);
                                    var_446 = wp::load(var_443);
                                    var_444 = wp::add(var_445, var_446);
                                    var_448 = wp::sub(var_444, var_447);
                                }
                                var_449 = wp::where(var_441, var_448, var_439);
                                // da = da_init                                                   <L 2892>
                                var_450 = wp::copy(var_449);
                                // ndof = int(0)                                                  <L 2893>
                                var_452 = wp::int(var_451);
                                // while da >= 0:                                                 <L 2894>
        start_while_8:;
                                var_454 = (var_450 >= var_453);
        if ((var_454) == false) goto end_while_8;
                                    // ndof += 1                                                  <L 2895>
                                    var_456 = wp::add(var_452, var_455);
                                    // da = dof_parentid[da]                                      <L 2896>
                                    var_457 = wp::address(var_dof_parentid, var_450);
                                    var_459 = wp::load(var_457);
                                    var_458 = wp::copy(var_459);
                                    wp::assign(var_452, var_456);
                                    wp::assign(var_450, var_458);
        goto start_while_8;
        end_while_8:;
                                // moment_rownnz_out[worldid, actid] = ndof                       <L 2898>
                                wp::array_store(var_moment_rownnz_out, var_0, var_1, var_452);
                                // rowadr = wp.atomic_add(moment_nnz, worldid, ndof)              <L 2899>
                                var_460 = wp::atomic_add(var_moment_nnz, var_0, var_452);
                                // moment_rowadr_out[worldid, actid] = rowadr                     <L 2900>
                                wp::array_store(var_moment_rowadr_out, var_0, var_1, var_460);
                                // actuator_length_out[worldid, actid] = 0.0                      <L 2901>
                                wp::array_store(var_actuator_length_out, var_0, var_1, var_461);
                                // da = da_init                                                   <L 2904>
                                var_462 = wp::copy(var_449);
                                // ptr = ndof - 1                                                 <L 2905>
                                var_464 = wp::sub(var_452, var_463);
                                // while da >= 0:                                                 <L 2906>
        start_while_10:;
                                var_466 = (var_462 >= var_465);
        if ((var_466) == false) goto end_while_10;
                                    // jacp, jacr = support.jac_dof(                              <L 2907>
                                    // body_parentid,                                             <L 2908>
                                    // body_rootid,                                               <L 2909>
                                    // dof_bodyid,                                                <L 2910>
                                    // body_isdofancestor,                                        <L 2911>
                                    // subtree_com_in,                                            <L 2912>
                                    // cdof_in,                                                   <L 2913>
                                    // site_xpos_in[worldid, siteid],                             <L 2914>
                                    var_467 = wp::address(var_site_xpos_in, var_0, var_413);
                                    // site_bodyid[siteid],                                       <L 2915>
                                    var_468 = wp::address(var_site_bodyid, var_413);
                                    // da,                                                        <L 2916>
                                    // worldid,                                                   <L 2917>
                                    var_471 = wp::load(var_467);
                                    var_472 = wp::load(var_468);
                                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_471, var_472, var_462, var_0, var_469, var_470);
                                    // moment = wp.dot(jacp, wrench_translation) + wp.dot(jacr, wrench_rotation)       <L 2919>
                                    var_473 = wp::dot(var_469, var_431);
                                    var_474 = wp::dot(var_470, var_432);
                                    var_475 = wp::add(var_473, var_474);
                                    // sparseid = rowadr + ptr                                    <L 2920>
                                    var_476 = wp::add(var_460, var_464);
                                    // moment_colind_out[worldid, sparseid] = da                  <L 2921>
                                    wp::array_store(var_moment_colind_out, var_0, var_476, var_462);
                                    // actuator_moment_out[worldid, sparseid] = moment            <L 2922>
                                    wp::array_store(var_actuator_moment_out, var_0, var_476, var_475);
                                    // ptr -= 1                                                   <L 2923>
                                    var_478 = wp::sub(var_464, var_477);
                                    // da = dof_parentid[da]                                      <L 2924>
                                    var_479 = wp::address(var_dof_parentid, var_462);
                                    var_481 = wp::load(var_479);
                                    var_480 = wp::copy(var_481);
                                    wp::assign(var_175, var_476);
                                    wp::assign(var_462, var_480);
                                    wp::assign(var_464, var_478);
                                    wp::assign(var_341, var_469);
                                    wp::assign(var_342, var_470);
                                    wp::assign(var_353, var_475);
        goto start_while_10;
        end_while_10:;
                            }
                            var_482 = wp::where(var_427, var_460, var_405);
                            var_483 = wp::where(var_427, var_429, var_223);
                            var_484 = wp::where(var_427, var_436, var_276);
                            var_485 = wp::where(var_427, var_452, var_310);
                            var_486 = wp::where(var_427, var_462, var_316);
                            var_487 = wp::where(var_427, var_464, var_333);
                            if (!var_427) {
                                // bodyid = site_bodyid[siteid]                                   <L 2928>
                                var_488 = wp::address(var_site_bodyid, var_413);
                                var_490 = wp::load(var_488);
                                var_489 = wp::copy(var_490);
                                // bodyrefid = site_bodyid[refid]                                 <L 2929>
                                var_491 = wp::address(var_site_bodyid, var_415);
                                var_493 = wp::load(var_491);
                                var_492 = wp::copy(var_493);
                                // b0 = body_weldid[bodyid]                                       <L 2930>
                                var_494 = wp::address(var_body_weldid, var_489);
                                var_496 = wp::load(var_494);
                                var_495 = wp::copy(var_496);
                                // b1 = body_weldid[bodyrefid]                                    <L 2931>
                                var_497 = wp::address(var_body_weldid, var_492);
                                var_499 = wp::load(var_497);
                                var_498 = wp::copy(var_499);
                                // dofadr0 = body_dofadr[b0] + body_dofnum[b0] - 1                <L 2932>
                                var_500 = wp::address(var_body_dofadr, var_495);
                                var_501 = wp::address(var_body_dofnum, var_495);
                                var_503 = wp::load(var_500);
                                var_504 = wp::load(var_501);
                                var_502 = wp::add(var_503, var_504);
                                var_506 = wp::sub(var_502, var_505);
                                // dofadr1 = body_dofadr[b1] + body_dofnum[b1] - 1                <L 2933>
                                var_507 = wp::address(var_body_dofadr, var_498);
                                var_508 = wp::address(var_body_dofnum, var_498);
                                var_510 = wp::load(var_507);
                                var_511 = wp::load(var_508);
                                var_509 = wp::add(var_510, var_511);
                                var_513 = wp::sub(var_509, var_512);
                                // dofadr_common = -1                                             <L 2936>
                                // if dofadr0 >= 0 and dofadr1 >= 0:                              <L 2937>
                                var_517 = (var_506 >= var_516);
                                var_515 = var_517;
                                if (var_515) {
                                    var_519 = (var_513 >= var_518);
                                    var_515 = var_515 && var_519;
                                }
                                if (var_515) {
                                    // while dofadr0 != dofadr1:                                  <L 2939>
        start_while_12:;
                                    var_520 = (var_506 != var_513);
        if ((var_520) == false) goto end_while_12;
                                        // if dofadr0 < dofadr1:                                  <L 2940>
                                        var_521 = (var_506 < var_513);
                                        if (var_521) {
                                            // dofadr1 = dof_parentid[dofadr1]                    <L 2941>
                                            var_522 = wp::address(var_dof_parentid, var_513);
                                            var_524 = wp::load(var_522);
                                            var_523 = wp::copy(var_524);
                                        }
                                        var_525 = wp::where(var_521, var_523, var_513);
                                        if (!var_521) {
                                            // dofadr0 = dof_parentid[dofadr0]                    <L 2943>
                                            var_526 = wp::address(var_dof_parentid, var_506);
                                            var_528 = wp::load(var_526);
                                            var_527 = wp::copy(var_528);
                                        }
                                        var_529 = wp::where(var_521, var_506, var_527);
                                        // if dofadr0 == -1 or dofadr1 == -1:                     <L 2945>
                                        var_532 = (var_529 == var_531);
                                        var_530 = var_532;
                                        if (!var_530) {
                                            var_534 = (var_525 == var_533);
                                            var_530 = var_530 || var_534;
                                        }
                                        if (var_530) {
                                            // break                                              <L 2947>
                                            wp::assign(var_506, var_529);
                                            wp::assign(var_513, var_525);
                                            goto end_while_12;
                                        }
                                        var_535 = wp::where(var_530, var_506, var_529);
                                        var_536 = wp::where(var_530, var_513, var_525);
                                        wp::assign(var_506, var_535);
                                        wp::assign(var_513, var_536);
        goto start_while_12;
        end_while_12:;
                                    // if dofadr0 == dofadr1:                                     <L 2950>
                                    var_537 = (var_506 == var_513);
                                    if (var_537) {
                                        // dofadr_common = dofadr0                                <L 2951>
                                        var_538 = wp::copy(var_506);
                                    }
                                    var_539 = wp::where(var_537, var_538, var_514);
                                }
                                var_540 = wp::where(var_515, var_539, var_514);
                                // translational_transmission = not (gear[0] == 0.0 and gear[1] == 0.0 and gear[2] == 0.0)       <L 2953>
                                var_543 = wp::extract(var_417, var_542);
                                var_545 = (var_543 == var_544);
                                var_541 = var_545;
                                if (var_541) {
                                    var_547 = wp::extract(var_417, var_546);
                                    var_549 = (var_547 == var_548);
                                    var_541 = var_541 && var_549;
                                }
                                if (var_541) {
                                    var_551 = wp::extract(var_417, var_550);
                                    var_553 = (var_551 == var_552);
                                    var_541 = var_541 && var_553;
                                }
                                var_554 = wp::unot(var_541);
                                // rotational_transmission = not (gear[3] == 0.0 and gear[4] == 0.0 and gear[5] == 0.0)       <L 2954>
                                var_557 = wp::extract(var_417, var_556);
                                var_559 = (var_557 == var_558);
                                var_555 = var_559;
                                if (var_555) {
                                    var_561 = wp::extract(var_417, var_560);
                                    var_563 = (var_561 == var_562);
                                    var_555 = var_555 && var_563;
                                }
                                if (var_555) {
                                    var_565 = wp::extract(var_417, var_564);
                                    var_567 = (var_565 == var_566);
                                    var_555 = var_555 && var_567;
                                }
                                var_568 = wp::unot(var_555);
                                // site_xpos = site_xpos_in[worldid, siteid]                      <L 2956>
                                var_569 = wp::address(var_site_xpos_in, var_0, var_413);
                                var_571 = wp::load(var_569);
                                var_570 = wp::copy(var_571);
                                // ref_xpos = site_xpos_in[worldid, refid]                        <L 2957>
                                var_572 = wp::address(var_site_xpos_in, var_0, var_415);
                                var_574 = wp::load(var_572);
                                var_573 = wp::copy(var_574);
                                // ref_xmat = site_xmat_in[worldid, refid]                        <L 2958>
                                var_575 = wp::address(var_site_xmat_in, var_0, var_415);
                                var_577 = wp::load(var_575);
                                var_576 = wp::copy(var_577);
                                // length = float(0.0)                                            <L 2960>
                                var_579 = wp::float(var_578);
                                // if translational_transmission:                                 <L 2962>
                                if (var_554) {
                                    // vec = wp.transpose(ref_xmat) @ (site_xpos - ref_xpos)       <L 2964>
                                    var_580 = wp::transpose(var_576);
                                    var_581 = wp::sub(var_570, var_573);
                                    var_582 = wp::mul(var_580, var_581);
                                    // length += wp.dot(vec, gear_translation)                    <L 2965>
                                    var_583 = wp::dot(var_582, var_424);
                                    var_584 = wp::add(var_579, var_583);
                                    // wrench_translation = ref_xmat @ gear_translation           <L 2967>
                                    var_585 = wp::mul(var_576, var_424);
                                }
                                var_586 = wp::where(var_554, var_582, var_241);
                                var_587 = wp::where(var_554, var_584, var_579);
                                var_588 = wp::where(var_554, var_585, var_431);
                                // if rotational_transmission:                                    <L 2969>
                                if (var_568) {
                                    // quat = math.mul_quat(site_quat[site_quat_id, siteid], xquat_in[worldid, bodyid])       <L 2971>
                                    var_589 = wp::address(var_site_quat, var_423, var_413);
                                    var_590 = wp::address(var_xquat_in, var_0, var_489);
                                    var_592 = wp::load(var_589);
                                    var_593 = wp::load(var_590);
                                    var_591 = mul_quat_0(var_592, var_593);
                                    // refquat = math.mul_quat(site_quat[site_quat_id, refid], xquat_in[worldid, bodyrefid])       <L 2972>
                                    var_594 = wp::address(var_site_quat, var_423, var_415);
                                    var_595 = wp::address(var_xquat_in, var_0, var_492);
                                    var_597 = wp::load(var_594);
                                    var_598 = wp::load(var_595);
                                    var_596 = mul_quat_0(var_597, var_598);
                                    // vec = math.quat_sub(quat, refquat)                         <L 2975>
                                    var_599 = quat_sub_0(var_591, var_596);
                                    // length += wp.dot(vec, gear_rotational)                     <L 2976>
                                    var_600 = wp::dot(var_599, var_425);
                                    var_601 = wp::add(var_587, var_600);
                                    // wrench_rotation = ref_xmat @ gear_rotational               <L 2978>
                                    var_602 = wp::mul(var_576, var_425);
                                }
                                var_603 = wp::where(var_568, var_591, var_83);
                                var_604 = wp::where(var_568, var_599, var_586);
                                var_605 = wp::where(var_568, var_601, var_587);
                                var_606 = wp::where(var_568, var_602, var_432);
                                // actuator_length_out[worldid, actid] = length                   <L 2980>
                                wp::array_store(var_actuator_length_out, var_0, var_1, var_605);
                                // da1_init = int(-1)                                             <L 2983>
                                var_608 = wp::int(var_607);
                                // da2_init = int(-1)                                             <L 2984>
                                var_610 = wp::int(var_609);
                                // if b0 > 0:                                                     <L 2985>
                                var_612 = (var_495 > var_611);
                                if (var_612) {
                                    // da1_init = body_dofadr[b0] + body_dofnum[b0] - 1           <L 2986>
                                    var_613 = wp::address(var_body_dofadr, var_495);
                                    var_614 = wp::address(var_body_dofnum, var_495);
                                    var_616 = wp::load(var_613);
                                    var_617 = wp::load(var_614);
                                    var_615 = wp::add(var_616, var_617);
                                    var_619 = wp::sub(var_615, var_618);
                                }
                                var_620 = wp::where(var_612, var_619, var_608);
                                // if b1 > 0:                                                     <L 2987>
                                var_622 = (var_498 > var_621);
                                if (var_622) {
                                    // da2_init = body_dofadr[b1] + body_dofnum[b1] - 1           <L 2988>
                                    var_623 = wp::address(var_body_dofadr, var_498);
                                    var_624 = wp::address(var_body_dofnum, var_498);
                                    var_626 = wp::load(var_623);
                                    var_627 = wp::load(var_624);
                                    var_625 = wp::add(var_626, var_627);
                                    var_629 = wp::sub(var_625, var_628);
                                }
                                var_630 = wp::where(var_622, var_629, var_610);
                                // da1 = da1_init                                                 <L 2990>
                                var_631 = wp::copy(var_620);
                                // da2 = da2_init                                                 <L 2991>
                                var_632 = wp::copy(var_630);
                                // ndof = int(0)                                                  <L 2992>
                                var_634 = wp::int(var_633);
                                // while da1 >= 0 or da2 >= 0:                                    <L 2993>
        start_while_14:;
                                var_637 = (var_631 >= var_636);
                                var_635 = var_637;
                                if (!var_635) {
                                    var_639 = (var_632 >= var_638);
                                    var_635 = var_635 || var_639;
                                }
        if ((var_635) == false) goto end_while_14;
                                    // da = wp.max(da1, da2)                                      <L 2994>
                                    var_640 = wp::max(var_631, var_632);
                                    // if da1 == da and da2 == da:                                <L 2995>
                                    var_642 = (var_631 == var_640);
                                    var_641 = var_642;
                                    if (var_641) {
                                        var_643 = (var_632 == var_640);
                                        var_641 = var_641 && var_643;
                                    }
                                    if (var_641) {
                                        // break                                                  <L 2996>
                                        wp::assign(var_486, var_640);
                                        goto end_while_14;
                                    }
                                    var_644 = wp::where(var_641, var_486, var_640);
                                    // ndof += 1                                                  <L 2997>
                                    var_646 = wp::add(var_634, var_645);
                                    // if da1 == da:                                              <L 2998>
                                    var_647 = (var_631 == var_644);
                                    if (var_647) {
                                        // da1 = dof_parentid[da1]                                <L 2999>
                                        var_648 = wp::address(var_dof_parentid, var_631);
                                        var_650 = wp::load(var_648);
                                        var_649 = wp::copy(var_650);
                                    }
                                    var_651 = wp::where(var_647, var_649, var_631);
                                    // if da2 == da:                                              <L 3000>
                                    var_652 = (var_632 == var_644);
                                    if (var_652) {
                                        // da2 = dof_parentid[da2]                                <L 3001>
                                        var_653 = wp::address(var_dof_parentid, var_632);
                                        var_655 = wp::load(var_653);
                                        var_654 = wp::copy(var_655);
                                    }
                                    var_656 = wp::where(var_652, var_654, var_632);
                                    wp::assign(var_631, var_651);
                                    wp::assign(var_632, var_656);
                                    wp::assign(var_634, var_646);
                                    wp::assign(var_486, var_644);
        goto start_while_14;
        end_while_14:;
                                // moment_rownnz_out[worldid, actid] = ndof                       <L 3003>
                                wp::array_store(var_moment_rownnz_out, var_0, var_1, var_634);
                                // rowadr = wp.atomic_add(moment_nnz, worldid, ndof)              <L 3004>
                                var_657 = wp::atomic_add(var_moment_nnz, var_0, var_634);
                                // moment_rowadr_out[worldid, actid] = rowadr                     <L 3005>
                                wp::array_store(var_moment_rowadr_out, var_0, var_1, var_657);
                                // da1 = da1_init                                                 <L 3008>
                                var_658 = wp::copy(var_620);
                                // da2 = da2_init                                                 <L 3009>
                                var_659 = wp::copy(var_630);
                                // ptr = ndof - 1                                                 <L 3011>
                                var_661 = wp::sub(var_634, var_660);
                                // while da1 >= 0 or da2 >= 0:                                    <L 3012>
        start_while_16:;
                                var_664 = (var_658 >= var_663);
                                var_662 = var_664;
                                if (!var_662) {
                                    var_666 = (var_659 >= var_665);
                                    var_662 = var_662 || var_666;
                                }
        if ((var_662) == false) goto end_while_16;
                                    // da = wp.max(da1, da2)                                      <L 3013>
                                    var_667 = wp::max(var_658, var_659);
                                    // if da1 == da and da2 == da:                                <L 3014>
                                    var_669 = (var_658 == var_667);
                                    var_668 = var_669;
                                    if (var_668) {
                                        var_670 = (var_659 == var_667);
                                        var_668 = var_668 && var_670;
                                    }
                                    if (var_668) {
                                        // break                                                  <L 3015>
                                        wp::assign(var_486, var_667);
                                        goto end_while_16;
                                    }
                                    var_671 = wp::where(var_668, var_486, var_667);
                                    // jacp, jacr = support.jac_dof(                              <L 3017>
                                    // body_parentid,                                             <L 3018>
                                    // body_rootid,                                               <L 3019>
                                    // dof_bodyid,                                                <L 3020>
                                    // body_isdofancestor,                                        <L 3021>
                                    // subtree_com_in,                                            <L 3022>
                                    // cdof_in,                                                   <L 3023>
                                    // site_xpos,                                                 <L 3024>
                                    // site_bodyid[siteid],                                       <L 3025>
                                    var_672 = wp::address(var_site_bodyid, var_413);
                                    // da,                                                        <L 3026>
                                    // worldid,                                                   <L 3027>
                                    var_675 = wp::load(var_672);
                                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_570, var_675, var_671, var_0, var_673, var_674);
                                    // jacpref, jacrref = support.jac_dof(                        <L 3029>
                                    // body_parentid,                                             <L 3030>
                                    // body_rootid,                                               <L 3031>
                                    // dof_bodyid,                                                <L 3032>
                                    // body_isdofancestor,                                        <L 3033>
                                    // subtree_com_in,                                            <L 3034>
                                    // cdof_in,                                                   <L 3035>
                                    // ref_xpos,                                                  <L 3036>
                                    // site_bodyid[refid],                                        <L 3037>
                                    var_676 = wp::address(var_site_bodyid, var_415);
                                    // da,                                                        <L 3038>
                                    // worldid,                                                   <L 3039>
                                    var_679 = wp::load(var_676);
                                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_573, var_679, var_671, var_0, var_677, var_678);
                                    // moment = float(0.0)                                        <L 3042>
                                    var_681 = wp::float(var_680);
                                    // if translational_transmission:                             <L 3043>
                                    if (var_554) {
                                        // moment += wp.dot(jacp - jacpref, wrench_translation)       <L 3044>
                                        var_682 = wp::sub(var_673, var_677);
                                        var_683 = wp::dot(var_682, var_588);
                                        var_684 = wp::add(var_681, var_683);
                                    }
                                    var_685 = wp::where(var_554, var_684, var_681);
                                    // if rotational_transmission:                                <L 3045>
                                    if (var_568) {
                                        // moment += wp.dot(jacr - jacrref, wrench_rotation)       <L 3046>
                                        var_686 = wp::sub(var_674, var_678);
                                        var_687 = wp::dot(var_686, var_606);
                                        var_688 = wp::add(var_685, var_687);
                                    }
                                    var_689 = wp::where(var_568, var_688, var_685);
                                    // sparseid = rowadr + ptr                                    <L 3048>
                                    var_690 = wp::add(var_657, var_661);
                                    // moment_colind_out[worldid, sparseid] = da                  <L 3049>
                                    wp::array_store(var_moment_colind_out, var_0, var_690, var_671);
                                    // actuator_moment_out[worldid, sparseid] = moment            <L 3050>
                                    wp::array_store(var_actuator_moment_out, var_0, var_690, var_689);
                                    // ptr -= 1                                                   <L 3051>
                                    var_692 = wp::sub(var_661, var_691);
                                    // if da1 == da:                                              <L 3053>
                                    var_693 = (var_658 == var_671);
                                    if (var_693) {
                                        // da1 = dof_parentid[da1]                                <L 3054>
                                        var_694 = wp::address(var_dof_parentid, var_658);
                                        var_696 = wp::load(var_694);
                                        var_695 = wp::copy(var_696);
                                    }
                                    var_697 = wp::where(var_693, var_695, var_658);
                                    // if da2 == da:                                              <L 3055>
                                    var_698 = (var_659 == var_671);
                                    if (var_698) {
                                        // da2 = dof_parentid[da2]                                <L 3056>
                                        var_699 = wp::address(var_dof_parentid, var_659);
                                        var_701 = wp::load(var_699);
                                        var_700 = wp::copy(var_701);
                                    }
                                    var_702 = wp::where(var_698, var_700, var_659);
                                    wp::assign(var_175, var_690);
                                    wp::assign(var_658, var_697);
                                    wp::assign(var_659, var_702);
                                    wp::assign(var_486, var_671);
                                    wp::assign(var_661, var_692);
                                    wp::assign(var_341, var_673);
                                    wp::assign(var_342, var_674);
                                    wp::assign(var_353, var_689);
        goto start_while_16;
        end_while_16:;
                            }
                            var_703 = wp::where(var_427, var_482, var_657);
                            var_704 = wp::where(var_427, var_83, var_603);
                            var_705 = wp::where(var_427, var_241, var_604);
                            var_706 = wp::where(var_427, var_258, var_605);
                            var_707 = wp::where(var_427, var_484, var_498);
                            var_708 = wp::where(var_427, var_296, var_620);
                            var_709 = wp::where(var_427, var_306, var_630);
                            var_710 = wp::where(var_427, var_330, var_658);
                            var_711 = wp::where(var_427, var_331, var_659);
                            var_712 = wp::where(var_427, var_485, var_634);
                            var_713 = wp::where(var_427, var_487, var_661);
                            var_714 = wp::where(var_427, var_431, var_588);
                            var_715 = wp::where(var_427, var_432, var_606);
                        }
                        var_716 = wp::where(var_408, var_417, var_11);
                        var_717 = wp::where(var_408, var_703, var_405);
                        var_718 = wp::where(var_408, var_704, var_83);
                        var_719 = wp::where(var_408, var_410, var_206);
                        var_720 = wp::where(var_408, var_483, var_223);
                        var_721 = wp::where(var_408, var_705, var_241);
                        var_722 = wp::where(var_408, var_706, var_258);
                        var_723 = wp::where(var_408, var_707, var_276);
                        var_724 = wp::where(var_408, var_708, var_296);
                        var_725 = wp::where(var_408, var_709, var_306);
                        var_726 = wp::where(var_408, var_710, var_330);
                        var_727 = wp::where(var_408, var_711, var_331);
                        var_728 = wp::where(var_408, var_712, var_310);
                        var_729 = wp::where(var_408, var_486, var_316);
                        var_730 = wp::where(var_408, var_713, var_333);
                        if (!var_408) {
                            // wp.printf("unhandled transmission type %d\n", trntype)             <L 3058>
                            printf(var_731, var_3);
                        }
                    }
                    var_732 = wp::where(var_398, var_11, var_716);
                    var_733 = wp::where(var_398, var_405, var_717);
                    var_734 = wp::where(var_398, var_83, var_718);
                    var_735 = wp::where(var_398, var_206, var_719);
                    var_736 = wp::where(var_398, var_223, var_720);
                    var_737 = wp::where(var_398, var_241, var_721);
                    var_738 = wp::where(var_398, var_258, var_722);
                    var_739 = wp::where(var_398, var_276, var_723);
                    var_740 = wp::where(var_398, var_296, var_724);
                    var_741 = wp::where(var_398, var_306, var_725);
                    var_742 = wp::where(var_398, var_330, var_726);
                    var_743 = wp::where(var_398, var_331, var_727);
                    var_744 = wp::where(var_398, var_310, var_728);
                    var_745 = wp::where(var_398, var_316, var_729);
                    var_746 = wp::where(var_398, var_333, var_730);
                }
                var_747 = wp::where(var_370, var_11, var_732);
                var_748 = wp::where(var_370, var_368, var_733);
                var_749 = wp::where(var_370, var_83, var_734);
                var_750 = wp::where(var_370, var_174, var_406);
                var_751 = wp::where(var_370, var_206, var_735);
                var_752 = wp::where(var_370, var_223, var_736);
                var_753 = wp::where(var_370, var_241, var_737);
                var_754 = wp::where(var_370, var_258, var_738);
                var_755 = wp::where(var_370, var_276, var_739);
                var_756 = wp::where(var_370, var_296, var_740);
                var_757 = wp::where(var_370, var_306, var_741);
                var_758 = wp::where(var_370, var_330, var_742);
                var_759 = wp::where(var_370, var_331, var_743);
                var_760 = wp::where(var_370, var_310, var_744);
                var_761 = wp::where(var_370, var_316, var_745);
                var_762 = wp::where(var_370, var_333, var_746);
            }
            var_763 = wp::where(var_204, var_11, var_747);
            var_764 = wp::where(var_204, var_368, var_748);
            var_765 = wp::where(var_204, var_83, var_749);
            var_766 = wp::where(var_204, var_174, var_750);
            var_767 = wp::where(var_204, var_206, var_751);
            var_768 = wp::where(var_204, var_213, var_396);
            var_769 = wp::where(var_204, var_223, var_752);
            var_770 = wp::where(var_204, var_241, var_753);
            var_771 = wp::where(var_204, var_258, var_754);
            var_772 = wp::where(var_204, var_276, var_755);
            var_773 = wp::where(var_204, var_296, var_756);
            var_774 = wp::where(var_204, var_306, var_757);
            var_775 = wp::where(var_204, var_330, var_758);
            var_776 = wp::where(var_204, var_331, var_759);
            var_777 = wp::where(var_204, var_310, var_760);
            var_778 = wp::where(var_204, var_316, var_761);
            var_779 = wp::where(var_204, var_333, var_762);
        }
        var_780 = wp::where(var_13, var_11, var_763);
        var_781 = wp::where(var_13, var_200, var_764);
        var_782 = wp::where(var_13, var_83, var_765);
        var_783 = wp::where(var_13, var_174, var_766);
    }
}



extern "C" __global__ void _flex_face_kinematics_9a1620ae_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::vec_t<3, wp::int32>> var_flex_cellnum,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_face_map,
    wp::array_t<wp::int32> var_flex_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_xpos_out,
    wp::array_t<wp::quat_t<wp::float32>> var_face_quat_out)
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
        wp::int32 var_12;
        wp::vec_t<3, wp::int32>* var_13;
        wp::vec_t<3, wp::int32> var_14;
        wp::vec_t<3, wp::int32> var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        const wp::int32 var_18 = 1;
        wp::int32 var_19;
        const wp::int32 var_20 = 2;
        wp::int32 var_21;
        const wp::float32 var_22 = 0.0;
        wp::vec_t<3, wp::float32> var_23;
        const wp::float32 var_24 = 0.0;
        wp::vec_t<3, wp::float32> var_25;
        const wp::int32 var_26 = 0;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        const wp::int32 var_30 = -1;
        bool var_31;
        wp::vec_t<3, wp::float32>* var_32;
        wp::vec_t<3, wp::float32> var_33;
        wp::vec_t<3, wp::float32> var_34;
        const wp::int32 var_35 = 1;
        wp::int32 var_36;
        wp::int32 var_37;
        const wp::int32 var_38 = 1;
        wp::int32 var_39;
        wp::int32 var_40;
        const wp::int32 var_41 = 2;
        bool var_42;
        const wp::int32 var_43 = 1;
        wp::int32 var_44;
        wp::float32 var_45;
        const wp::float32 var_46 = -1.0;
        const wp::float32 var_47 = 2.0;
        wp::float32 var_48;
        wp::float32 var_49;
        wp::float32 var_50;
        wp::float32 var_51;
        const wp::int32 var_52 = 2;
        bool var_53;
        const wp::int32 var_54 = 1;
        wp::int32 var_55;
        wp::float32 var_56;
        const wp::float32 var_57 = -1.0;
        const wp::float32 var_58 = 2.0;
        wp::float32 var_59;
        wp::float32 var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        const wp::int32 var_63 = 2;
        bool var_64;
        const wp::int32 var_65 = 1;
        bool var_66;
        const wp::float32 var_67 = 1.0;
        const wp::float32 var_68 = 0.0;
        wp::float32 var_69;
        const wp::float32 var_70 = 0.5;
        wp::float32 var_71;
        const wp::int32 var_72 = 2;
        bool var_73;
        const wp::int32 var_74 = 1;
        bool var_75;
        const wp::float32 var_76 = 1.0;
        const wp::float32 var_77 = 0.0;
        wp::float32 var_78;
        const wp::float32 var_79 = 0.5;
        wp::float32 var_80;
        wp::float32 var_81;
        wp::float32 var_82;
        wp::vec_t<3, wp::float32> var_83;
        wp::vec_t<3, wp::float32> var_84;
        wp::vec_t<3, wp::float32> var_85;
        wp::vec_t<3, wp::float32> var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::vec_t<3, wp::float32> var_88;
        const wp::float32 var_89 = 0.0;
        wp::vec_t<3, wp::float32> var_90;
        const wp::int32 var_91 = 1;
        wp::int32* var_92;
        wp::int32 var_93;
        wp::int32 var_94;
        const wp::int32 var_95 = -1;
        bool var_96;
        wp::vec_t<3, wp::float32>* var_97;
        wp::vec_t<3, wp::float32> var_98;
        wp::vec_t<3, wp::float32> var_99;
        const wp::int32 var_100 = 1;
        wp::int32 var_101;
        wp::int32 var_102;
        const wp::int32 var_103 = 1;
        wp::int32 var_104;
        wp::int32 var_105;
        const wp::int32 var_106 = 2;
        bool var_107;
        const wp::int32 var_108 = 1;
        wp::int32 var_109;
        wp::float32 var_110;
        const wp::float32 var_111 = -1.0;
        const wp::float32 var_112 = 2.0;
        wp::float32 var_113;
        wp::float32 var_114;
        wp::float32 var_115;
        wp::float32 var_116;
        const wp::int32 var_117 = 2;
        bool var_118;
        const wp::int32 var_119 = 1;
        wp::int32 var_120;
        wp::float32 var_121;
        const wp::float32 var_122 = -1.0;
        const wp::float32 var_123 = 2.0;
        wp::float32 var_124;
        wp::float32 var_125;
        wp::float32 var_126;
        wp::float32 var_127;
        const wp::int32 var_128 = 2;
        bool var_129;
        const wp::int32 var_130 = 1;
        bool var_131;
        const wp::float32 var_132 = 1.0;
        const wp::float32 var_133 = 0.0;
        wp::float32 var_134;
        const wp::float32 var_135 = 0.5;
        wp::float32 var_136;
        const wp::int32 var_137 = 2;
        bool var_138;
        const wp::int32 var_139 = 1;
        bool var_140;
        const wp::float32 var_141 = 1.0;
        const wp::float32 var_142 = 0.0;
        wp::float32 var_143;
        const wp::float32 var_144 = 0.5;
        wp::float32 var_145;
        wp::float32 var_146;
        wp::float32 var_147;
        wp::vec_t<3, wp::float32> var_148;
        wp::vec_t<3, wp::float32> var_149;
        wp::vec_t<3, wp::float32> var_150;
        wp::vec_t<3, wp::float32> var_151;
        wp::vec_t<3, wp::float32> var_152;
        wp::vec_t<3, wp::float32> var_153;
        wp::vec_t<3, wp::float32> var_154;
        wp::int32 var_155;
        wp::int32 var_156;
        wp::float32 var_157;
        wp::float32 var_158;
        wp::float32 var_159;
        wp::float32 var_160;
        wp::float32 var_161;
        wp::float32 var_162;
        const wp::float32 var_163 = 0.0;
        wp::vec_t<3, wp::float32> var_164;
        const wp::int32 var_165 = 2;
        wp::int32* var_166;
        wp::int32 var_167;
        wp::int32 var_168;
        const wp::int32 var_169 = -1;
        bool var_170;
        wp::vec_t<3, wp::float32>* var_171;
        wp::vec_t<3, wp::float32> var_172;
        wp::vec_t<3, wp::float32> var_173;
        const wp::int32 var_174 = 1;
        wp::int32 var_175;
        wp::int32 var_176;
        const wp::int32 var_177 = 1;
        wp::int32 var_178;
        wp::int32 var_179;
        const wp::int32 var_180 = 2;
        bool var_181;
        const wp::int32 var_182 = 1;
        wp::int32 var_183;
        wp::float32 var_184;
        const wp::float32 var_185 = -1.0;
        const wp::float32 var_186 = 2.0;
        wp::float32 var_187;
        wp::float32 var_188;
        wp::float32 var_189;
        wp::float32 var_190;
        const wp::int32 var_191 = 2;
        bool var_192;
        const wp::int32 var_193 = 1;
        wp::int32 var_194;
        wp::float32 var_195;
        const wp::float32 var_196 = -1.0;
        const wp::float32 var_197 = 2.0;
        wp::float32 var_198;
        wp::float32 var_199;
        wp::float32 var_200;
        wp::float32 var_201;
        const wp::int32 var_202 = 2;
        bool var_203;
        const wp::int32 var_204 = 1;
        bool var_205;
        const wp::float32 var_206 = 1.0;
        const wp::float32 var_207 = 0.0;
        wp::float32 var_208;
        const wp::float32 var_209 = 0.5;
        wp::float32 var_210;
        const wp::int32 var_211 = 2;
        bool var_212;
        const wp::int32 var_213 = 1;
        bool var_214;
        const wp::float32 var_215 = 1.0;
        const wp::float32 var_216 = 0.0;
        wp::float32 var_217;
        const wp::float32 var_218 = 0.5;
        wp::float32 var_219;
        wp::float32 var_220;
        wp::float32 var_221;
        wp::vec_t<3, wp::float32> var_222;
        wp::vec_t<3, wp::float32> var_223;
        wp::vec_t<3, wp::float32> var_224;
        wp::vec_t<3, wp::float32> var_225;
        wp::vec_t<3, wp::float32> var_226;
        wp::vec_t<3, wp::float32> var_227;
        wp::vec_t<3, wp::float32> var_228;
        wp::int32 var_229;
        wp::int32 var_230;
        wp::float32 var_231;
        wp::float32 var_232;
        wp::float32 var_233;
        wp::float32 var_234;
        wp::float32 var_235;
        wp::float32 var_236;
        const wp::float32 var_237 = 0.0;
        wp::vec_t<3, wp::float32> var_238;
        const wp::int32 var_239 = 3;
        wp::int32* var_240;
        wp::int32 var_241;
        wp::int32 var_242;
        const wp::int32 var_243 = -1;
        bool var_244;
        wp::vec_t<3, wp::float32>* var_245;
        wp::vec_t<3, wp::float32> var_246;
        wp::vec_t<3, wp::float32> var_247;
        const wp::int32 var_248 = 1;
        wp::int32 var_249;
        wp::int32 var_250;
        const wp::int32 var_251 = 1;
        wp::int32 var_252;
        wp::int32 var_253;
        const wp::int32 var_254 = 2;
        bool var_255;
        const wp::int32 var_256 = 1;
        wp::int32 var_257;
        wp::float32 var_258;
        const wp::float32 var_259 = -1.0;
        const wp::float32 var_260 = 2.0;
        wp::float32 var_261;
        wp::float32 var_262;
        wp::float32 var_263;
        wp::float32 var_264;
        const wp::int32 var_265 = 2;
        bool var_266;
        const wp::int32 var_267 = 1;
        wp::int32 var_268;
        wp::float32 var_269;
        const wp::float32 var_270 = -1.0;
        const wp::float32 var_271 = 2.0;
        wp::float32 var_272;
        wp::float32 var_273;
        wp::float32 var_274;
        wp::float32 var_275;
        const wp::int32 var_276 = 2;
        bool var_277;
        const wp::int32 var_278 = 1;
        bool var_279;
        const wp::float32 var_280 = 1.0;
        const wp::float32 var_281 = 0.0;
        wp::float32 var_282;
        const wp::float32 var_283 = 0.5;
        wp::float32 var_284;
        const wp::int32 var_285 = 2;
        bool var_286;
        const wp::int32 var_287 = 1;
        bool var_288;
        const wp::float32 var_289 = 1.0;
        const wp::float32 var_290 = 0.0;
        wp::float32 var_291;
        const wp::float32 var_292 = 0.5;
        wp::float32 var_293;
        wp::float32 var_294;
        wp::float32 var_295;
        wp::vec_t<3, wp::float32> var_296;
        wp::vec_t<3, wp::float32> var_297;
        wp::vec_t<3, wp::float32> var_298;
        wp::vec_t<3, wp::float32> var_299;
        wp::vec_t<3, wp::float32> var_300;
        wp::vec_t<3, wp::float32> var_301;
        wp::vec_t<3, wp::float32> var_302;
        wp::int32 var_303;
        wp::int32 var_304;
        wp::float32 var_305;
        wp::float32 var_306;
        wp::float32 var_307;
        wp::float32 var_308;
        wp::float32 var_309;
        wp::float32 var_310;
        const wp::float32 var_311 = 0.0;
        wp::vec_t<3, wp::float32> var_312;
        const wp::int32 var_313 = 4;
        wp::int32* var_314;
        wp::int32 var_315;
        wp::int32 var_316;
        const wp::int32 var_317 = -1;
        bool var_318;
        wp::vec_t<3, wp::float32>* var_319;
        wp::vec_t<3, wp::float32> var_320;
        wp::vec_t<3, wp::float32> var_321;
        const wp::int32 var_322 = 1;
        wp::int32 var_323;
        wp::int32 var_324;
        const wp::int32 var_325 = 1;
        wp::int32 var_326;
        wp::int32 var_327;
        const wp::int32 var_328 = 2;
        bool var_329;
        const wp::int32 var_330 = 1;
        wp::int32 var_331;
        wp::float32 var_332;
        const wp::float32 var_333 = -1.0;
        const wp::float32 var_334 = 2.0;
        wp::float32 var_335;
        wp::float32 var_336;
        wp::float32 var_337;
        wp::float32 var_338;
        const wp::int32 var_339 = 2;
        bool var_340;
        const wp::int32 var_341 = 1;
        wp::int32 var_342;
        wp::float32 var_343;
        const wp::float32 var_344 = -1.0;
        const wp::float32 var_345 = 2.0;
        wp::float32 var_346;
        wp::float32 var_347;
        wp::float32 var_348;
        wp::float32 var_349;
        const wp::int32 var_350 = 2;
        bool var_351;
        const wp::int32 var_352 = 1;
        bool var_353;
        const wp::float32 var_354 = 1.0;
        const wp::float32 var_355 = 0.0;
        wp::float32 var_356;
        const wp::float32 var_357 = 0.5;
        wp::float32 var_358;
        const wp::int32 var_359 = 2;
        bool var_360;
        const wp::int32 var_361 = 1;
        bool var_362;
        const wp::float32 var_363 = 1.0;
        const wp::float32 var_364 = 0.0;
        wp::float32 var_365;
        const wp::float32 var_366 = 0.5;
        wp::float32 var_367;
        wp::float32 var_368;
        wp::float32 var_369;
        wp::vec_t<3, wp::float32> var_370;
        wp::vec_t<3, wp::float32> var_371;
        wp::vec_t<3, wp::float32> var_372;
        wp::vec_t<3, wp::float32> var_373;
        wp::vec_t<3, wp::float32> var_374;
        wp::vec_t<3, wp::float32> var_375;
        wp::vec_t<3, wp::float32> var_376;
        wp::int32 var_377;
        wp::int32 var_378;
        wp::float32 var_379;
        wp::float32 var_380;
        wp::float32 var_381;
        wp::float32 var_382;
        wp::float32 var_383;
        wp::float32 var_384;
        const wp::float32 var_385 = 0.0;
        wp::vec_t<3, wp::float32> var_386;
        const wp::int32 var_387 = 5;
        wp::int32* var_388;
        wp::int32 var_389;
        wp::int32 var_390;
        const wp::int32 var_391 = -1;
        bool var_392;
        wp::vec_t<3, wp::float32>* var_393;
        wp::vec_t<3, wp::float32> var_394;
        wp::vec_t<3, wp::float32> var_395;
        const wp::int32 var_396 = 1;
        wp::int32 var_397;
        wp::int32 var_398;
        const wp::int32 var_399 = 1;
        wp::int32 var_400;
        wp::int32 var_401;
        const wp::int32 var_402 = 2;
        bool var_403;
        const wp::int32 var_404 = 1;
        wp::int32 var_405;
        wp::float32 var_406;
        const wp::float32 var_407 = -1.0;
        const wp::float32 var_408 = 2.0;
        wp::float32 var_409;
        wp::float32 var_410;
        wp::float32 var_411;
        wp::float32 var_412;
        const wp::int32 var_413 = 2;
        bool var_414;
        const wp::int32 var_415 = 1;
        wp::int32 var_416;
        wp::float32 var_417;
        const wp::float32 var_418 = -1.0;
        const wp::float32 var_419 = 2.0;
        wp::float32 var_420;
        wp::float32 var_421;
        wp::float32 var_422;
        wp::float32 var_423;
        const wp::int32 var_424 = 2;
        bool var_425;
        const wp::int32 var_426 = 1;
        bool var_427;
        const wp::float32 var_428 = 1.0;
        const wp::float32 var_429 = 0.0;
        wp::float32 var_430;
        const wp::float32 var_431 = 0.5;
        wp::float32 var_432;
        const wp::int32 var_433 = 2;
        bool var_434;
        const wp::int32 var_435 = 1;
        bool var_436;
        const wp::float32 var_437 = 1.0;
        const wp::float32 var_438 = 0.0;
        wp::float32 var_439;
        const wp::float32 var_440 = 0.5;
        wp::float32 var_441;
        wp::float32 var_442;
        wp::float32 var_443;
        wp::vec_t<3, wp::float32> var_444;
        wp::vec_t<3, wp::float32> var_445;
        wp::vec_t<3, wp::float32> var_446;
        wp::vec_t<3, wp::float32> var_447;
        wp::vec_t<3, wp::float32> var_448;
        wp::vec_t<3, wp::float32> var_449;
        wp::vec_t<3, wp::float32> var_450;
        wp::int32 var_451;
        wp::int32 var_452;
        wp::float32 var_453;
        wp::float32 var_454;
        wp::float32 var_455;
        wp::float32 var_456;
        wp::float32 var_457;
        wp::float32 var_458;
        const wp::float32 var_459 = 0.0;
        wp::vec_t<3, wp::float32> var_460;
        const wp::int32 var_461 = 6;
        wp::int32* var_462;
        wp::int32 var_463;
        wp::int32 var_464;
        const wp::int32 var_465 = -1;
        bool var_466;
        wp::vec_t<3, wp::float32>* var_467;
        wp::vec_t<3, wp::float32> var_468;
        wp::vec_t<3, wp::float32> var_469;
        const wp::int32 var_470 = 1;
        wp::int32 var_471;
        wp::int32 var_472;
        const wp::int32 var_473 = 1;
        wp::int32 var_474;
        wp::int32 var_475;
        const wp::int32 var_476 = 2;
        bool var_477;
        const wp::int32 var_478 = 1;
        wp::int32 var_479;
        wp::float32 var_480;
        const wp::float32 var_481 = -1.0;
        const wp::float32 var_482 = 2.0;
        wp::float32 var_483;
        wp::float32 var_484;
        wp::float32 var_485;
        wp::float32 var_486;
        const wp::int32 var_487 = 2;
        bool var_488;
        const wp::int32 var_489 = 1;
        wp::int32 var_490;
        wp::float32 var_491;
        const wp::float32 var_492 = -1.0;
        const wp::float32 var_493 = 2.0;
        wp::float32 var_494;
        wp::float32 var_495;
        wp::float32 var_496;
        wp::float32 var_497;
        const wp::int32 var_498 = 2;
        bool var_499;
        const wp::int32 var_500 = 1;
        bool var_501;
        const wp::float32 var_502 = 1.0;
        const wp::float32 var_503 = 0.0;
        wp::float32 var_504;
        const wp::float32 var_505 = 0.5;
        wp::float32 var_506;
        const wp::int32 var_507 = 2;
        bool var_508;
        const wp::int32 var_509 = 1;
        bool var_510;
        const wp::float32 var_511 = 1.0;
        const wp::float32 var_512 = 0.0;
        wp::float32 var_513;
        const wp::float32 var_514 = 0.5;
        wp::float32 var_515;
        wp::float32 var_516;
        wp::float32 var_517;
        wp::vec_t<3, wp::float32> var_518;
        wp::vec_t<3, wp::float32> var_519;
        wp::vec_t<3, wp::float32> var_520;
        wp::vec_t<3, wp::float32> var_521;
        wp::vec_t<3, wp::float32> var_522;
        wp::vec_t<3, wp::float32> var_523;
        wp::vec_t<3, wp::float32> var_524;
        wp::int32 var_525;
        wp::int32 var_526;
        wp::float32 var_527;
        wp::float32 var_528;
        wp::float32 var_529;
        wp::float32 var_530;
        wp::float32 var_531;
        wp::float32 var_532;
        const wp::float32 var_533 = 0.0;
        wp::vec_t<3, wp::float32> var_534;
        const wp::int32 var_535 = 7;
        wp::int32* var_536;
        wp::int32 var_537;
        wp::int32 var_538;
        const wp::int32 var_539 = -1;
        bool var_540;
        wp::vec_t<3, wp::float32>* var_541;
        wp::vec_t<3, wp::float32> var_542;
        wp::vec_t<3, wp::float32> var_543;
        const wp::int32 var_544 = 1;
        wp::int32 var_545;
        wp::int32 var_546;
        const wp::int32 var_547 = 1;
        wp::int32 var_548;
        wp::int32 var_549;
        const wp::int32 var_550 = 2;
        bool var_551;
        const wp::int32 var_552 = 1;
        wp::int32 var_553;
        wp::float32 var_554;
        const wp::float32 var_555 = -1.0;
        const wp::float32 var_556 = 2.0;
        wp::float32 var_557;
        wp::float32 var_558;
        wp::float32 var_559;
        wp::float32 var_560;
        const wp::int32 var_561 = 2;
        bool var_562;
        const wp::int32 var_563 = 1;
        wp::int32 var_564;
        wp::float32 var_565;
        const wp::float32 var_566 = -1.0;
        const wp::float32 var_567 = 2.0;
        wp::float32 var_568;
        wp::float32 var_569;
        wp::float32 var_570;
        wp::float32 var_571;
        const wp::int32 var_572 = 2;
        bool var_573;
        const wp::int32 var_574 = 1;
        bool var_575;
        const wp::float32 var_576 = 1.0;
        const wp::float32 var_577 = 0.0;
        wp::float32 var_578;
        const wp::float32 var_579 = 0.5;
        wp::float32 var_580;
        const wp::int32 var_581 = 2;
        bool var_582;
        const wp::int32 var_583 = 1;
        bool var_584;
        const wp::float32 var_585 = 1.0;
        const wp::float32 var_586 = 0.0;
        wp::float32 var_587;
        const wp::float32 var_588 = 0.5;
        wp::float32 var_589;
        wp::float32 var_590;
        wp::float32 var_591;
        wp::vec_t<3, wp::float32> var_592;
        wp::vec_t<3, wp::float32> var_593;
        wp::vec_t<3, wp::float32> var_594;
        wp::vec_t<3, wp::float32> var_595;
        wp::vec_t<3, wp::float32> var_596;
        wp::vec_t<3, wp::float32> var_597;
        wp::vec_t<3, wp::float32> var_598;
        wp::int32 var_599;
        wp::int32 var_600;
        wp::float32 var_601;
        wp::float32 var_602;
        wp::float32 var_603;
        wp::float32 var_604;
        wp::float32 var_605;
        wp::float32 var_606;
        const wp::float32 var_607 = 0.0;
        wp::vec_t<3, wp::float32> var_608;
        const wp::int32 var_609 = 8;
        wp::int32* var_610;
        wp::int32 var_611;
        wp::int32 var_612;
        const wp::int32 var_613 = -1;
        bool var_614;
        wp::vec_t<3, wp::float32>* var_615;
        wp::vec_t<3, wp::float32> var_616;
        wp::vec_t<3, wp::float32> var_617;
        const wp::int32 var_618 = 1;
        wp::int32 var_619;
        wp::int32 var_620;
        const wp::int32 var_621 = 1;
        wp::int32 var_622;
        wp::int32 var_623;
        const wp::int32 var_624 = 2;
        bool var_625;
        const wp::int32 var_626 = 1;
        wp::int32 var_627;
        wp::float32 var_628;
        const wp::float32 var_629 = -1.0;
        const wp::float32 var_630 = 2.0;
        wp::float32 var_631;
        wp::float32 var_632;
        wp::float32 var_633;
        wp::float32 var_634;
        const wp::int32 var_635 = 2;
        bool var_636;
        const wp::int32 var_637 = 1;
        wp::int32 var_638;
        wp::float32 var_639;
        const wp::float32 var_640 = -1.0;
        const wp::float32 var_641 = 2.0;
        wp::float32 var_642;
        wp::float32 var_643;
        wp::float32 var_644;
        wp::float32 var_645;
        const wp::int32 var_646 = 2;
        bool var_647;
        const wp::int32 var_648 = 1;
        bool var_649;
        const wp::float32 var_650 = 1.0;
        const wp::float32 var_651 = 0.0;
        wp::float32 var_652;
        const wp::float32 var_653 = 0.5;
        wp::float32 var_654;
        const wp::int32 var_655 = 2;
        bool var_656;
        const wp::int32 var_657 = 1;
        bool var_658;
        const wp::float32 var_659 = 1.0;
        const wp::float32 var_660 = 0.0;
        wp::float32 var_661;
        const wp::float32 var_662 = 0.5;
        wp::float32 var_663;
        wp::float32 var_664;
        wp::float32 var_665;
        wp::vec_t<3, wp::float32> var_666;
        wp::vec_t<3, wp::float32> var_667;
        wp::vec_t<3, wp::float32> var_668;
        wp::vec_t<3, wp::float32> var_669;
        wp::vec_t<3, wp::float32> var_670;
        wp::vec_t<3, wp::float32> var_671;
        wp::vec_t<3, wp::float32> var_672;
        wp::int32 var_673;
        wp::int32 var_674;
        wp::float32 var_675;
        wp::float32 var_676;
        wp::float32 var_677;
        wp::float32 var_678;
        wp::float32 var_679;
        wp::float32 var_680;
        const wp::float32 var_681 = 0.0;
        wp::vec_t<3, wp::float32> var_682;
        wp::vec_t<3, wp::float32> var_683;
        wp::int32 var_684;
        wp::int32 var_685;
        wp::int32 var_686;
        wp::int32 var_687;
        wp::int32 var_688;
        wp::int32 var_689;
        const wp::float32 var_690 = 0.0;
        wp::mat_t<3, 3, wp::float32> var_691;
        const wp::int32 var_692 = 0;
        bool var_693;
        const wp::int32 var_694 = 0;
        wp::float32 var_695;
        const wp::int32 var_696 = 0;
        wp::float32 var_697;
        const wp::int32 var_698 = 0;
        wp::float32 var_699;
        const wp::int32 var_700 = 1;
        wp::float32 var_701;
        const wp::int32 var_702 = 1;
        wp::float32 var_703;
        const wp::int32 var_704 = 1;
        wp::float32 var_705;
        const wp::int32 var_706 = 2;
        wp::float32 var_707;
        const wp::int32 var_708 = 2;
        wp::float32 var_709;
        const wp::int32 var_710 = 2;
        wp::float32 var_711;
        wp::mat_t<3, 3, wp::float32> var_712;
        wp::mat_t<3, 3, wp::float32> var_713;
        const wp::int32 var_714 = 1;
        bool var_715;
        const wp::int32 var_716 = 0;
        wp::float32 var_717;
        const wp::int32 var_718 = 0;
        wp::float32 var_719;
        const wp::int32 var_720 = 0;
        wp::float32 var_721;
        const wp::int32 var_722 = 1;
        wp::float32 var_723;
        const wp::int32 var_724 = 1;
        wp::float32 var_725;
        const wp::int32 var_726 = 1;
        wp::float32 var_727;
        const wp::int32 var_728 = 2;
        wp::float32 var_729;
        const wp::int32 var_730 = 2;
        wp::float32 var_731;
        const wp::int32 var_732 = 2;
        wp::float32 var_733;
        wp::mat_t<3, 3, wp::float32> var_734;
        wp::mat_t<3, 3, wp::float32> var_735;
        const wp::int32 var_736 = 0;
        wp::float32 var_737;
        const wp::int32 var_738 = 0;
        wp::float32 var_739;
        const wp::int32 var_740 = 0;
        wp::float32 var_741;
        const wp::int32 var_742 = 1;
        wp::float32 var_743;
        const wp::int32 var_744 = 1;
        wp::float32 var_745;
        const wp::int32 var_746 = 1;
        wp::float32 var_747;
        const wp::int32 var_748 = 2;
        wp::float32 var_749;
        const wp::int32 var_750 = 2;
        wp::float32 var_751;
        const wp::int32 var_752 = 2;
        wp::float32 var_753;
        wp::mat_t<3, 3, wp::float32> var_754;
        wp::mat_t<3, 3, wp::float32> var_755;
        wp::mat_t<3, 3, wp::float32> var_756;
        wp::quat_t<wp::float32> var_757;
        //---------
        // forward
        // def _flex_face_kinematics(                                                             <L 479>
        // worldid, face_id = wp.tid()                                                            <L 491>
        builtin_tid2d(var_0, var_1);
        // mapping = flex_face_map[face_id]                                                       <L 493>
        var_2 = wp::address(var_flex_face_map, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // f = mapping[0]                                                                         <L 494>
        var_6 = wp::extract(var_3, var_5);
        // face_elem_idx = mapping[1]                                                             <L 495>
        var_8 = wp::extract(var_3, var_7);
        // order = flex_interp[f]                                                                 <L 497>
        var_9 = wp::address(var_flex_interp, var_6);
        var_11 = wp::load(var_9);
        var_10 = wp::copy(var_11);
        // order_abs = -order                                                                     <L 498>
        var_12 = wp::neg(var_10);
        // cellnum = flex_cellnum[f]                                                              <L 499>
        var_13 = wp::address(var_flex_cellnum, var_6);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // cx = cellnum[0]                                                                        <L 500>
        var_17 = wp::extract(var_14, var_16);
        // cy = cellnum[1]                                                                        <L 501>
        var_19 = wp::extract(var_14, var_18);
        // cz = cellnum[2]                                                                        <L 502>
        var_21 = wp::extract(var_14, var_20);
        // t1 = wp.vec3(0.0)                                                                      <L 503>
        var_23 = wp::vec_t<3, wp::float32>(var_22);
        // t2 = wp.vec3(0.0)                                                                      <L 504>
        var_25 = wp::vec_t<3, wp::float32>(var_24);
        // for local_idx in range(9):                                                             <L 505>
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_27 = wp::address(var_flex_face, var_1, var_26);
        var_29 = wp::load(var_27);
        var_28 = wp::copy(var_29);
        // if gidx != -1:                                                                         <L 507>
        var_31 = (var_28 != var_30);
        if (var_31) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_32 = wp::address(var_flexnode_xpos_in, var_0, var_28);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_26, var_33);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_36 = wp::add(var_12, var_35);
            var_37 = wp::floordiv(var_26, var_36);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_39 = wp::add(var_12, var_38);
            var_40 = wp::mod(var_26, var_39);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_42 = (var_12 == var_41);
            if (var_42) {
                var_44 = wp::sub(var_37, var_43);
                var_45 = wp::float(var_44);
            }
            if (!var_42) {
                var_48 = wp::float(var_37);
                var_49 = wp::mul(var_47, var_48);
                var_50 = wp::add(var_46, var_49);
            }
            var_51 = wp::where(var_42, var_45, var_50);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_53 = (var_12 == var_52);
            if (var_53) {
                var_55 = wp::sub(var_40, var_54);
                var_56 = wp::float(var_55);
            }
            if (!var_53) {
                var_59 = wp::float(var_40);
                var_60 = wp::mul(var_58, var_59);
                var_61 = wp::add(var_57, var_60);
            }
            var_62 = wp::where(var_53, var_56, var_61);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_64 = (var_12 == var_63);
            if (var_64) {
                var_66 = (var_37 == var_65);
                var_69 = wp::where(var_66, var_67, var_68);
            }
            if (!var_64) {
            }
            var_71 = wp::where(var_64, var_69, var_70);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_73 = (var_12 == var_72);
            if (var_73) {
                var_75 = (var_40 == var_74);
                var_78 = wp::where(var_75, var_76, var_77);
            }
            if (!var_73) {
            }
            var_80 = wp::where(var_73, var_78, var_79);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_81 = wp::mul(var_51, var_80);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_82 = wp::mul(var_71, var_62);
            // t1 += node_pos * grad0                                                             <L 523>
            var_83 = wp::mul(var_33, var_81);
            var_84 = wp::add(var_23, var_83);
            // t2 += node_pos * grad1                                                             <L 524>
            var_85 = wp::mul(var_33, var_82);
            var_86 = wp::add(var_25, var_85);
        }
        var_87 = wp::where(var_31, var_84, var_23);
        var_88 = wp::where(var_31, var_86, var_25);
        if (!var_31) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_90 = wp::vec_t<3, wp::float32>(var_89);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_26, var_90);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_92 = wp::address(var_flex_face, var_1, var_91);
        var_94 = wp::load(var_92);
        var_93 = wp::copy(var_94);
        // if gidx != -1:                                                                         <L 507>
        var_96 = (var_93 != var_95);
        if (var_96) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_97 = wp::address(var_flexnode_xpos_in, var_0, var_93);
            var_99 = wp::load(var_97);
            var_98 = wp::copy(var_99);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_91, var_98);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_101 = wp::add(var_12, var_100);
            var_102 = wp::floordiv(var_91, var_101);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_104 = wp::add(var_12, var_103);
            var_105 = wp::mod(var_91, var_104);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_107 = (var_12 == var_106);
            if (var_107) {
                var_109 = wp::sub(var_102, var_108);
                var_110 = wp::float(var_109);
            }
            if (!var_107) {
                var_113 = wp::float(var_102);
                var_114 = wp::mul(var_112, var_113);
                var_115 = wp::add(var_111, var_114);
            }
            var_116 = wp::where(var_107, var_110, var_115);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_118 = (var_12 == var_117);
            if (var_118) {
                var_120 = wp::sub(var_105, var_119);
                var_121 = wp::float(var_120);
            }
            if (!var_118) {
                var_124 = wp::float(var_105);
                var_125 = wp::mul(var_123, var_124);
                var_126 = wp::add(var_122, var_125);
            }
            var_127 = wp::where(var_118, var_121, var_126);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_129 = (var_12 == var_128);
            if (var_129) {
                var_131 = (var_102 == var_130);
                var_134 = wp::where(var_131, var_132, var_133);
            }
            if (!var_129) {
            }
            var_136 = wp::where(var_129, var_134, var_135);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_138 = (var_12 == var_137);
            if (var_138) {
                var_140 = (var_105 == var_139);
                var_143 = wp::where(var_140, var_141, var_142);
            }
            if (!var_138) {
            }
            var_145 = wp::where(var_138, var_143, var_144);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_146 = wp::mul(var_116, var_145);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_147 = wp::mul(var_136, var_127);
            // t1 += node_pos * grad0                                                             <L 523>
            var_148 = wp::mul(var_98, var_146);
            var_149 = wp::add(var_87, var_148);
            // t2 += node_pos * grad1                                                             <L 524>
            var_150 = wp::mul(var_98, var_147);
            var_151 = wp::add(var_88, var_150);
        }
        var_152 = wp::where(var_96, var_149, var_87);
        var_153 = wp::where(var_96, var_151, var_88);
        var_154 = wp::where(var_96, var_98, var_33);
        var_155 = wp::where(var_96, var_102, var_37);
        var_156 = wp::where(var_96, var_105, var_40);
        var_157 = wp::where(var_96, var_116, var_51);
        var_158 = wp::where(var_96, var_127, var_62);
        var_159 = wp::where(var_96, var_136, var_71);
        var_160 = wp::where(var_96, var_145, var_80);
        var_161 = wp::where(var_96, var_146, var_81);
        var_162 = wp::where(var_96, var_147, var_82);
        if (!var_96) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_164 = wp::vec_t<3, wp::float32>(var_163);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_91, var_164);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_166 = wp::address(var_flex_face, var_1, var_165);
        var_168 = wp::load(var_166);
        var_167 = wp::copy(var_168);
        // if gidx != -1:                                                                         <L 507>
        var_170 = (var_167 != var_169);
        if (var_170) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_171 = wp::address(var_flexnode_xpos_in, var_0, var_167);
            var_173 = wp::load(var_171);
            var_172 = wp::copy(var_173);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_165, var_172);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_175 = wp::add(var_12, var_174);
            var_176 = wp::floordiv(var_165, var_175);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_178 = wp::add(var_12, var_177);
            var_179 = wp::mod(var_165, var_178);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_181 = (var_12 == var_180);
            if (var_181) {
                var_183 = wp::sub(var_176, var_182);
                var_184 = wp::float(var_183);
            }
            if (!var_181) {
                var_187 = wp::float(var_176);
                var_188 = wp::mul(var_186, var_187);
                var_189 = wp::add(var_185, var_188);
            }
            var_190 = wp::where(var_181, var_184, var_189);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_192 = (var_12 == var_191);
            if (var_192) {
                var_194 = wp::sub(var_179, var_193);
                var_195 = wp::float(var_194);
            }
            if (!var_192) {
                var_198 = wp::float(var_179);
                var_199 = wp::mul(var_197, var_198);
                var_200 = wp::add(var_196, var_199);
            }
            var_201 = wp::where(var_192, var_195, var_200);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_203 = (var_12 == var_202);
            if (var_203) {
                var_205 = (var_176 == var_204);
                var_208 = wp::where(var_205, var_206, var_207);
            }
            if (!var_203) {
            }
            var_210 = wp::where(var_203, var_208, var_209);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_212 = (var_12 == var_211);
            if (var_212) {
                var_214 = (var_179 == var_213);
                var_217 = wp::where(var_214, var_215, var_216);
            }
            if (!var_212) {
            }
            var_219 = wp::where(var_212, var_217, var_218);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_220 = wp::mul(var_190, var_219);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_221 = wp::mul(var_210, var_201);
            // t1 += node_pos * grad0                                                             <L 523>
            var_222 = wp::mul(var_172, var_220);
            var_223 = wp::add(var_152, var_222);
            // t2 += node_pos * grad1                                                             <L 524>
            var_224 = wp::mul(var_172, var_221);
            var_225 = wp::add(var_153, var_224);
        }
        var_226 = wp::where(var_170, var_223, var_152);
        var_227 = wp::where(var_170, var_225, var_153);
        var_228 = wp::where(var_170, var_172, var_154);
        var_229 = wp::where(var_170, var_176, var_155);
        var_230 = wp::where(var_170, var_179, var_156);
        var_231 = wp::where(var_170, var_190, var_157);
        var_232 = wp::where(var_170, var_201, var_158);
        var_233 = wp::where(var_170, var_210, var_159);
        var_234 = wp::where(var_170, var_219, var_160);
        var_235 = wp::where(var_170, var_220, var_161);
        var_236 = wp::where(var_170, var_221, var_162);
        if (!var_170) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_238 = wp::vec_t<3, wp::float32>(var_237);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_165, var_238);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_240 = wp::address(var_flex_face, var_1, var_239);
        var_242 = wp::load(var_240);
        var_241 = wp::copy(var_242);
        // if gidx != -1:                                                                         <L 507>
        var_244 = (var_241 != var_243);
        if (var_244) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_245 = wp::address(var_flexnode_xpos_in, var_0, var_241);
            var_247 = wp::load(var_245);
            var_246 = wp::copy(var_247);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_239, var_246);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_249 = wp::add(var_12, var_248);
            var_250 = wp::floordiv(var_239, var_249);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_252 = wp::add(var_12, var_251);
            var_253 = wp::mod(var_239, var_252);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_255 = (var_12 == var_254);
            if (var_255) {
                var_257 = wp::sub(var_250, var_256);
                var_258 = wp::float(var_257);
            }
            if (!var_255) {
                var_261 = wp::float(var_250);
                var_262 = wp::mul(var_260, var_261);
                var_263 = wp::add(var_259, var_262);
            }
            var_264 = wp::where(var_255, var_258, var_263);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_266 = (var_12 == var_265);
            if (var_266) {
                var_268 = wp::sub(var_253, var_267);
                var_269 = wp::float(var_268);
            }
            if (!var_266) {
                var_272 = wp::float(var_253);
                var_273 = wp::mul(var_271, var_272);
                var_274 = wp::add(var_270, var_273);
            }
            var_275 = wp::where(var_266, var_269, var_274);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_277 = (var_12 == var_276);
            if (var_277) {
                var_279 = (var_250 == var_278);
                var_282 = wp::where(var_279, var_280, var_281);
            }
            if (!var_277) {
            }
            var_284 = wp::where(var_277, var_282, var_283);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_286 = (var_12 == var_285);
            if (var_286) {
                var_288 = (var_253 == var_287);
                var_291 = wp::where(var_288, var_289, var_290);
            }
            if (!var_286) {
            }
            var_293 = wp::where(var_286, var_291, var_292);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_294 = wp::mul(var_264, var_293);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_295 = wp::mul(var_284, var_275);
            // t1 += node_pos * grad0                                                             <L 523>
            var_296 = wp::mul(var_246, var_294);
            var_297 = wp::add(var_226, var_296);
            // t2 += node_pos * grad1                                                             <L 524>
            var_298 = wp::mul(var_246, var_295);
            var_299 = wp::add(var_227, var_298);
        }
        var_300 = wp::where(var_244, var_297, var_226);
        var_301 = wp::where(var_244, var_299, var_227);
        var_302 = wp::where(var_244, var_246, var_228);
        var_303 = wp::where(var_244, var_250, var_229);
        var_304 = wp::where(var_244, var_253, var_230);
        var_305 = wp::where(var_244, var_264, var_231);
        var_306 = wp::where(var_244, var_275, var_232);
        var_307 = wp::where(var_244, var_284, var_233);
        var_308 = wp::where(var_244, var_293, var_234);
        var_309 = wp::where(var_244, var_294, var_235);
        var_310 = wp::where(var_244, var_295, var_236);
        if (!var_244) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_312 = wp::vec_t<3, wp::float32>(var_311);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_239, var_312);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_314 = wp::address(var_flex_face, var_1, var_313);
        var_316 = wp::load(var_314);
        var_315 = wp::copy(var_316);
        // if gidx != -1:                                                                         <L 507>
        var_318 = (var_315 != var_317);
        if (var_318) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_319 = wp::address(var_flexnode_xpos_in, var_0, var_315);
            var_321 = wp::load(var_319);
            var_320 = wp::copy(var_321);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_313, var_320);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_323 = wp::add(var_12, var_322);
            var_324 = wp::floordiv(var_313, var_323);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_326 = wp::add(var_12, var_325);
            var_327 = wp::mod(var_313, var_326);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_329 = (var_12 == var_328);
            if (var_329) {
                var_331 = wp::sub(var_324, var_330);
                var_332 = wp::float(var_331);
            }
            if (!var_329) {
                var_335 = wp::float(var_324);
                var_336 = wp::mul(var_334, var_335);
                var_337 = wp::add(var_333, var_336);
            }
            var_338 = wp::where(var_329, var_332, var_337);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_340 = (var_12 == var_339);
            if (var_340) {
                var_342 = wp::sub(var_327, var_341);
                var_343 = wp::float(var_342);
            }
            if (!var_340) {
                var_346 = wp::float(var_327);
                var_347 = wp::mul(var_345, var_346);
                var_348 = wp::add(var_344, var_347);
            }
            var_349 = wp::where(var_340, var_343, var_348);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_351 = (var_12 == var_350);
            if (var_351) {
                var_353 = (var_324 == var_352);
                var_356 = wp::where(var_353, var_354, var_355);
            }
            if (!var_351) {
            }
            var_358 = wp::where(var_351, var_356, var_357);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_360 = (var_12 == var_359);
            if (var_360) {
                var_362 = (var_327 == var_361);
                var_365 = wp::where(var_362, var_363, var_364);
            }
            if (!var_360) {
            }
            var_367 = wp::where(var_360, var_365, var_366);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_368 = wp::mul(var_338, var_367);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_369 = wp::mul(var_358, var_349);
            // t1 += node_pos * grad0                                                             <L 523>
            var_370 = wp::mul(var_320, var_368);
            var_371 = wp::add(var_300, var_370);
            // t2 += node_pos * grad1                                                             <L 524>
            var_372 = wp::mul(var_320, var_369);
            var_373 = wp::add(var_301, var_372);
        }
        var_374 = wp::where(var_318, var_371, var_300);
        var_375 = wp::where(var_318, var_373, var_301);
        var_376 = wp::where(var_318, var_320, var_302);
        var_377 = wp::where(var_318, var_324, var_303);
        var_378 = wp::where(var_318, var_327, var_304);
        var_379 = wp::where(var_318, var_338, var_305);
        var_380 = wp::where(var_318, var_349, var_306);
        var_381 = wp::where(var_318, var_358, var_307);
        var_382 = wp::where(var_318, var_367, var_308);
        var_383 = wp::where(var_318, var_368, var_309);
        var_384 = wp::where(var_318, var_369, var_310);
        if (!var_318) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_386 = wp::vec_t<3, wp::float32>(var_385);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_313, var_386);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_388 = wp::address(var_flex_face, var_1, var_387);
        var_390 = wp::load(var_388);
        var_389 = wp::copy(var_390);
        // if gidx != -1:                                                                         <L 507>
        var_392 = (var_389 != var_391);
        if (var_392) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_393 = wp::address(var_flexnode_xpos_in, var_0, var_389);
            var_395 = wp::load(var_393);
            var_394 = wp::copy(var_395);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_387, var_394);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_397 = wp::add(var_12, var_396);
            var_398 = wp::floordiv(var_387, var_397);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_400 = wp::add(var_12, var_399);
            var_401 = wp::mod(var_387, var_400);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_403 = (var_12 == var_402);
            if (var_403) {
                var_405 = wp::sub(var_398, var_404);
                var_406 = wp::float(var_405);
            }
            if (!var_403) {
                var_409 = wp::float(var_398);
                var_410 = wp::mul(var_408, var_409);
                var_411 = wp::add(var_407, var_410);
            }
            var_412 = wp::where(var_403, var_406, var_411);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_414 = (var_12 == var_413);
            if (var_414) {
                var_416 = wp::sub(var_401, var_415);
                var_417 = wp::float(var_416);
            }
            if (!var_414) {
                var_420 = wp::float(var_401);
                var_421 = wp::mul(var_419, var_420);
                var_422 = wp::add(var_418, var_421);
            }
            var_423 = wp::where(var_414, var_417, var_422);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_425 = (var_12 == var_424);
            if (var_425) {
                var_427 = (var_398 == var_426);
                var_430 = wp::where(var_427, var_428, var_429);
            }
            if (!var_425) {
            }
            var_432 = wp::where(var_425, var_430, var_431);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_434 = (var_12 == var_433);
            if (var_434) {
                var_436 = (var_401 == var_435);
                var_439 = wp::where(var_436, var_437, var_438);
            }
            if (!var_434) {
            }
            var_441 = wp::where(var_434, var_439, var_440);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_442 = wp::mul(var_412, var_441);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_443 = wp::mul(var_432, var_423);
            // t1 += node_pos * grad0                                                             <L 523>
            var_444 = wp::mul(var_394, var_442);
            var_445 = wp::add(var_374, var_444);
            // t2 += node_pos * grad1                                                             <L 524>
            var_446 = wp::mul(var_394, var_443);
            var_447 = wp::add(var_375, var_446);
        }
        var_448 = wp::where(var_392, var_445, var_374);
        var_449 = wp::where(var_392, var_447, var_375);
        var_450 = wp::where(var_392, var_394, var_376);
        var_451 = wp::where(var_392, var_398, var_377);
        var_452 = wp::where(var_392, var_401, var_378);
        var_453 = wp::where(var_392, var_412, var_379);
        var_454 = wp::where(var_392, var_423, var_380);
        var_455 = wp::where(var_392, var_432, var_381);
        var_456 = wp::where(var_392, var_441, var_382);
        var_457 = wp::where(var_392, var_442, var_383);
        var_458 = wp::where(var_392, var_443, var_384);
        if (!var_392) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_460 = wp::vec_t<3, wp::float32>(var_459);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_387, var_460);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_462 = wp::address(var_flex_face, var_1, var_461);
        var_464 = wp::load(var_462);
        var_463 = wp::copy(var_464);
        // if gidx != -1:                                                                         <L 507>
        var_466 = (var_463 != var_465);
        if (var_466) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_467 = wp::address(var_flexnode_xpos_in, var_0, var_463);
            var_469 = wp::load(var_467);
            var_468 = wp::copy(var_469);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_461, var_468);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_471 = wp::add(var_12, var_470);
            var_472 = wp::floordiv(var_461, var_471);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_474 = wp::add(var_12, var_473);
            var_475 = wp::mod(var_461, var_474);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_477 = (var_12 == var_476);
            if (var_477) {
                var_479 = wp::sub(var_472, var_478);
                var_480 = wp::float(var_479);
            }
            if (!var_477) {
                var_483 = wp::float(var_472);
                var_484 = wp::mul(var_482, var_483);
                var_485 = wp::add(var_481, var_484);
            }
            var_486 = wp::where(var_477, var_480, var_485);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_488 = (var_12 == var_487);
            if (var_488) {
                var_490 = wp::sub(var_475, var_489);
                var_491 = wp::float(var_490);
            }
            if (!var_488) {
                var_494 = wp::float(var_475);
                var_495 = wp::mul(var_493, var_494);
                var_496 = wp::add(var_492, var_495);
            }
            var_497 = wp::where(var_488, var_491, var_496);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_499 = (var_12 == var_498);
            if (var_499) {
                var_501 = (var_472 == var_500);
                var_504 = wp::where(var_501, var_502, var_503);
            }
            if (!var_499) {
            }
            var_506 = wp::where(var_499, var_504, var_505);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_508 = (var_12 == var_507);
            if (var_508) {
                var_510 = (var_475 == var_509);
                var_513 = wp::where(var_510, var_511, var_512);
            }
            if (!var_508) {
            }
            var_515 = wp::where(var_508, var_513, var_514);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_516 = wp::mul(var_486, var_515);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_517 = wp::mul(var_506, var_497);
            // t1 += node_pos * grad0                                                             <L 523>
            var_518 = wp::mul(var_468, var_516);
            var_519 = wp::add(var_448, var_518);
            // t2 += node_pos * grad1                                                             <L 524>
            var_520 = wp::mul(var_468, var_517);
            var_521 = wp::add(var_449, var_520);
        }
        var_522 = wp::where(var_466, var_519, var_448);
        var_523 = wp::where(var_466, var_521, var_449);
        var_524 = wp::where(var_466, var_468, var_450);
        var_525 = wp::where(var_466, var_472, var_451);
        var_526 = wp::where(var_466, var_475, var_452);
        var_527 = wp::where(var_466, var_486, var_453);
        var_528 = wp::where(var_466, var_497, var_454);
        var_529 = wp::where(var_466, var_506, var_455);
        var_530 = wp::where(var_466, var_515, var_456);
        var_531 = wp::where(var_466, var_516, var_457);
        var_532 = wp::where(var_466, var_517, var_458);
        if (!var_466) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_534 = wp::vec_t<3, wp::float32>(var_533);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_461, var_534);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_536 = wp::address(var_flex_face, var_1, var_535);
        var_538 = wp::load(var_536);
        var_537 = wp::copy(var_538);
        // if gidx != -1:                                                                         <L 507>
        var_540 = (var_537 != var_539);
        if (var_540) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_541 = wp::address(var_flexnode_xpos_in, var_0, var_537);
            var_543 = wp::load(var_541);
            var_542 = wp::copy(var_543);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_535, var_542);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_545 = wp::add(var_12, var_544);
            var_546 = wp::floordiv(var_535, var_545);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_548 = wp::add(var_12, var_547);
            var_549 = wp::mod(var_535, var_548);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_551 = (var_12 == var_550);
            if (var_551) {
                var_553 = wp::sub(var_546, var_552);
                var_554 = wp::float(var_553);
            }
            if (!var_551) {
                var_557 = wp::float(var_546);
                var_558 = wp::mul(var_556, var_557);
                var_559 = wp::add(var_555, var_558);
            }
            var_560 = wp::where(var_551, var_554, var_559);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_562 = (var_12 == var_561);
            if (var_562) {
                var_564 = wp::sub(var_549, var_563);
                var_565 = wp::float(var_564);
            }
            if (!var_562) {
                var_568 = wp::float(var_549);
                var_569 = wp::mul(var_567, var_568);
                var_570 = wp::add(var_566, var_569);
            }
            var_571 = wp::where(var_562, var_565, var_570);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_573 = (var_12 == var_572);
            if (var_573) {
                var_575 = (var_546 == var_574);
                var_578 = wp::where(var_575, var_576, var_577);
            }
            if (!var_573) {
            }
            var_580 = wp::where(var_573, var_578, var_579);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_582 = (var_12 == var_581);
            if (var_582) {
                var_584 = (var_549 == var_583);
                var_587 = wp::where(var_584, var_585, var_586);
            }
            if (!var_582) {
            }
            var_589 = wp::where(var_582, var_587, var_588);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_590 = wp::mul(var_560, var_589);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_591 = wp::mul(var_580, var_571);
            // t1 += node_pos * grad0                                                             <L 523>
            var_592 = wp::mul(var_542, var_590);
            var_593 = wp::add(var_522, var_592);
            // t2 += node_pos * grad1                                                             <L 524>
            var_594 = wp::mul(var_542, var_591);
            var_595 = wp::add(var_523, var_594);
        }
        var_596 = wp::where(var_540, var_593, var_522);
        var_597 = wp::where(var_540, var_595, var_523);
        var_598 = wp::where(var_540, var_542, var_524);
        var_599 = wp::where(var_540, var_546, var_525);
        var_600 = wp::where(var_540, var_549, var_526);
        var_601 = wp::where(var_540, var_560, var_527);
        var_602 = wp::where(var_540, var_571, var_528);
        var_603 = wp::where(var_540, var_580, var_529);
        var_604 = wp::where(var_540, var_589, var_530);
        var_605 = wp::where(var_540, var_590, var_531);
        var_606 = wp::where(var_540, var_591, var_532);
        if (!var_540) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_608 = wp::vec_t<3, wp::float32>(var_607);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_535, var_608);
        }
        // gidx = flex_face[face_id, local_idx]                                                   <L 506>
        var_610 = wp::address(var_flex_face, var_1, var_609);
        var_612 = wp::load(var_610);
        var_611 = wp::copy(var_612);
        // if gidx != -1:                                                                         <L 507>
        var_614 = (var_611 != var_613);
        if (var_614) {
            // node_pos = flexnode_xpos_in[worldid, gidx]                                         <L 508>
            var_615 = wp::address(var_flexnode_xpos_in, var_0, var_611);
            var_617 = wp::load(var_615);
            var_616 = wp::copy(var_617);
            // face_xpos_out[worldid, face_id, local_idx] = node_pos                              <L 510>
            wp::array_store(var_face_xpos_out, var_0, var_1, var_609, var_616);
            // l0 = local_idx // (order_abs + 1)                                                  <L 512>
            var_619 = wp::add(var_12, var_618);
            var_620 = wp::floordiv(var_609, var_619);
            // l1 = local_idx % (order_abs + 1)                                                   <L 513>
            var_622 = wp::add(var_12, var_621);
            var_623 = wp::mod(var_609, var_622);
            // dphi0 = float(l0 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l0))              <L 515>
            var_625 = (var_12 == var_624);
            if (var_625) {
                var_627 = wp::sub(var_620, var_626);
                var_628 = wp::float(var_627);
            }
            if (!var_625) {
                var_631 = wp::float(var_620);
                var_632 = wp::mul(var_630, var_631);
                var_633 = wp::add(var_629, var_632);
            }
            var_634 = wp::where(var_625, var_628, var_633);
            // dphi1 = float(l1 - 1) if order_abs == 2 else (-1.0 + 2.0 * float(l1))              <L 516>
            var_636 = (var_12 == var_635);
            if (var_636) {
                var_638 = wp::sub(var_623, var_637);
                var_639 = wp::float(var_638);
            }
            if (!var_636) {
                var_642 = wp::float(var_623);
                var_643 = wp::mul(var_641, var_642);
                var_644 = wp::add(var_640, var_643);
            }
            var_645 = wp::where(var_636, var_639, var_644);
            // phi0 = wp.where(l0 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 517>
            var_647 = (var_12 == var_646);
            if (var_647) {
                var_649 = (var_620 == var_648);
                var_652 = wp::where(var_649, var_650, var_651);
            }
            if (!var_647) {
            }
            var_654 = wp::where(var_647, var_652, var_653);
            // phi1 = wp.where(l1 == 1, 1.0, 0.0) if order_abs == 2 else 0.5                      <L 518>
            var_656 = (var_12 == var_655);
            if (var_656) {
                var_658 = (var_623 == var_657);
                var_661 = wp::where(var_658, var_659, var_660);
            }
            if (!var_656) {
            }
            var_663 = wp::where(var_656, var_661, var_662);
            // grad0 = dphi0 * phi1                                                               <L 520>
            var_664 = wp::mul(var_634, var_663);
            // grad1 = phi0 * dphi1                                                               <L 521>
            var_665 = wp::mul(var_654, var_645);
            // t1 += node_pos * grad0                                                             <L 523>
            var_666 = wp::mul(var_616, var_664);
            var_667 = wp::add(var_596, var_666);
            // t2 += node_pos * grad1                                                             <L 524>
            var_668 = wp::mul(var_616, var_665);
            var_669 = wp::add(var_597, var_668);
        }
        var_670 = wp::where(var_614, var_667, var_596);
        var_671 = wp::where(var_614, var_669, var_597);
        var_672 = wp::where(var_614, var_616, var_598);
        var_673 = wp::where(var_614, var_620, var_599);
        var_674 = wp::where(var_614, var_623, var_600);
        var_675 = wp::where(var_614, var_634, var_601);
        var_676 = wp::where(var_614, var_645, var_602);
        var_677 = wp::where(var_614, var_654, var_603);
        var_678 = wp::where(var_614, var_663, var_604);
        var_679 = wp::where(var_614, var_664, var_605);
        var_680 = wp::where(var_614, var_665, var_606);
        if (!var_614) {
            // face_xpos_out[worldid, face_id, local_idx] = wp.vec3(0.0)                          <L 526>
            var_682 = wp::vec_t<3, wp::float32>(var_681);
            wp::array_store(var_face_xpos_out, var_0, var_1, var_609, var_682);
        }
        // normal = wp.cross(t1, t2)                                                              <L 528>
        var_683 = wp::cross(var_670, var_671);
        // normal_axis, _, _, _, _, _ = support.get_face_metadata(cx, cy, cz, face_elem_idx, order_abs)       <L 530>
        get_face_metadata_0(var_17, var_19, var_21, var_8, var_12, var_684, var_685, var_686, var_687, var_688, var_689);
        // F = wp.mat33(0.0)                                                                      <L 532>
        var_691 = wp::mat_t<3, 3, wp::float32>(var_690);
        // if normal_axis == 0:                                                                   <L 533>
        var_693 = (var_684 == var_692);
        if (var_693) {
            // F = wp.mat33(                                                                      <L 534>
            // normal[0],                                                                         <L 535>
            var_695 = wp::extract(var_683, var_694);
            // t1[0],                                                                             <L 536>
            var_697 = wp::extract(var_670, var_696);
            // t2[0],                                                                             <L 537>
            var_699 = wp::extract(var_671, var_698);
            // normal[1],                                                                         <L 538>
            var_701 = wp::extract(var_683, var_700);
            // t1[1],                                                                             <L 539>
            var_703 = wp::extract(var_670, var_702);
            // t2[1],                                                                             <L 540>
            var_705 = wp::extract(var_671, var_704);
            // normal[2],                                                                         <L 541>
            var_707 = wp::extract(var_683, var_706);
            // t1[2],                                                                             <L 542>
            var_709 = wp::extract(var_670, var_708);
            // t2[2],                                                                             <L 543>
            var_711 = wp::extract(var_671, var_710);
            var_712 = wp::mat_t<3, 3, wp::float32>(var_695, var_697, var_699, var_701, var_703, var_705, var_707, var_709, var_711);
        }
        var_713 = wp::where(var_693, var_712, var_691);
        if (!var_693) {
            // elif normal_axis == 1:                                                             <L 545>
            var_715 = (var_684 == var_714);
            if (var_715) {
                // F = wp.mat33(                                                                  <L 546>
                // t2[0],                                                                         <L 547>
                var_717 = wp::extract(var_671, var_716);
                // normal[0],                                                                     <L 548>
                var_719 = wp::extract(var_683, var_718);
                // t1[0],                                                                         <L 549>
                var_721 = wp::extract(var_670, var_720);
                // t2[1],                                                                         <L 550>
                var_723 = wp::extract(var_671, var_722);
                // normal[1],                                                                     <L 551>
                var_725 = wp::extract(var_683, var_724);
                // t1[1],                                                                         <L 552>
                var_727 = wp::extract(var_670, var_726);
                // t2[2],                                                                         <L 553>
                var_729 = wp::extract(var_671, var_728);
                // normal[2],                                                                     <L 554>
                var_731 = wp::extract(var_683, var_730);
                // t1[2],                                                                         <L 555>
                var_733 = wp::extract(var_670, var_732);
                var_734 = wp::mat_t<3, 3, wp::float32>(var_717, var_719, var_721, var_723, var_725, var_727, var_729, var_731, var_733);
            }
            var_735 = wp::where(var_715, var_734, var_713);
            if (!var_715) {
                // F = wp.mat33(                                                                  <L 558>
                // t1[0],                                                                         <L 559>
                var_737 = wp::extract(var_670, var_736);
                // t2[0],                                                                         <L 560>
                var_739 = wp::extract(var_671, var_738);
                // normal[0],                                                                     <L 561>
                var_741 = wp::extract(var_683, var_740);
                // t1[1],                                                                         <L 562>
                var_743 = wp::extract(var_670, var_742);
                // t2[1],                                                                         <L 563>
                var_745 = wp::extract(var_671, var_744);
                // normal[1],                                                                     <L 564>
                var_747 = wp::extract(var_683, var_746);
                // t1[2],                                                                         <L 565>
                var_749 = wp::extract(var_670, var_748);
                // t2[2],                                                                         <L 566>
                var_751 = wp::extract(var_671, var_750);
                // normal[2],                                                                     <L 567>
                var_753 = wp::extract(var_683, var_752);
                var_754 = wp::mat_t<3, 3, wp::float32>(var_737, var_739, var_741, var_743, var_745, var_747, var_749, var_751, var_753);
            }
            var_755 = wp::where(var_715, var_735, var_754);
        }
        var_756 = wp::where(var_693, var_713, var_755);
        // face_quat_out[worldid, face_id] = support.mat33_to_quat_polar(F)                       <L 570>
        var_757 = mat33_to_quat_polar_0(var_756);
        wp::array_store(var_face_quat_out, var_0, var_1, var_757);
    }
}



extern "C" __global__ void _angular_momentum_35b41762_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::float32> var_body_subtreemass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_linvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_subtree_bodyvel_in,
    wp::array_t<wp::int32> var_body_tree_,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_angmom_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        const wp::int32 var_5 = 0;
        bool var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        wp::vec_t<3, wp::float32>* var_10;
        wp::vec_t<3, wp::float32> var_11;
        wp::vec_t<3, wp::float32> var_12;
        wp::vec_t<3, wp::float32>* var_13;
        wp::vec_t<3, wp::float32> var_14;
        wp::vec_t<3, wp::float32> var_15;
        wp::vec_t<3, wp::float32>* var_16;
        wp::vec_t<3, wp::float32> var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<6, wp::float32>* var_19;
        wp::vec_t<6, wp::float32> var_20;
        wp::vec_t<6, wp::float32> var_21;
        wp::vec_t<3, wp::float32>* var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::vec_t<3, wp::float32> var_24;
        wp::vec_t<3, wp::float32>* var_25;
        wp::vec_t<3, wp::float32> var_26;
        wp::vec_t<3, wp::float32> var_27;
        wp::shape_t* var_28;
        const wp::int32 var_29 = 0;
        wp::int32 var_30;
        wp::shape_t var_31;
        wp::int32 var_32;
        wp::float32* var_33;
        wp::float32 var_34;
        wp::float32 var_35;
        wp::shape_t* var_36;
        const wp::int32 var_37 = 0;
        wp::int32 var_38;
        wp::shape_t var_39;
        wp::int32 var_40;
        wp::float32* var_41;
        wp::float32 var_42;
        wp::float32 var_43;
        wp::vec_t<3, wp::float32> var_44;
        wp::vec_t<3, wp::float32> var_45;
        wp::vec_t<3, wp::float32> var_46;
        wp::vec_t<3, wp::float32> var_47;
        wp::vec_t<3, wp::float32> var_48;
        wp::vec_t<3, wp::float32> var_49;
        wp::slice_t var_50;
        const wp::int32 var_51 = 0;
        wp::array_t<wp::vec_t<3, wp::float32>> var_52;
        wp::vec_t<3, wp::float32>* var_53;
        wp::vec_t<3, wp::float32> var_54;
        wp::vec_t<3, wp::float32> var_55;
        wp::vec_t<3, wp::float32> var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::slice_t var_60;
        const wp::int32 var_61 = 0;
        wp::array_t<wp::vec_t<3, wp::float32>> var_62;
        wp::vec_t<3, wp::float32> var_63;
        //---------
        // forward
        // def _angular_momentum(                                                                 <L 3891>
        // worldid, nodeid = wp.tid()                                                             <L 3906>
        builtin_tid2d(var_0, var_1);
        // bodyid = body_tree_[nodeid]                                                            <L 3907>
        var_2 = wp::address(var_body_tree_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if bodyid == 0:                                                                        <L 3909>
        var_6 = (var_3 == var_5);
        if (var_6) {
            // return                                                                             <L 3910>
            continue;
        }
        // pid = body_parentid[bodyid]                                                            <L 3912>
        var_7 = wp::address(var_body_parentid, var_3);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // xipos = xipos_in[worldid, bodyid]                                                      <L 3914>
        var_10 = wp::address(var_xipos_in, var_0, var_3);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // com = subtree_com_in[worldid, bodyid]                                                  <L 3915>
        var_13 = wp::address(var_subtree_com_in, var_0, var_3);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // com_parent = subtree_com_in[worldid, pid]                                              <L 3916>
        var_16 = wp::address(var_subtree_com_in, var_0, var_8);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // vel = subtree_bodyvel_in[worldid, bodyid]                                              <L 3917>
        var_19 = wp::address(var_subtree_bodyvel_in, var_0, var_3);
        var_21 = wp::load(var_19);
        var_20 = wp::copy(var_21);
        // linvel = subtree_linvel_in[worldid, bodyid]                                            <L 3918>
        var_22 = wp::address(var_subtree_linvel_in, var_0, var_3);
        var_24 = wp::load(var_22);
        var_23 = wp::copy(var_24);
        // linvel_parent = subtree_linvel_in[worldid, pid]  # Data field                          <L 3919>
        var_25 = wp::address(var_subtree_linvel_in, var_0, var_8);
        var_27 = wp::load(var_25);
        var_26 = wp::copy(var_27);
        // mass = body_mass[worldid % body_mass.shape[0], bodyid]                                 <L 3920>
        var_28 = &(var_body_mass.shape);
        var_31 = wp::load(var_28);
        var_30 = wp::extract(var_31, var_29);
        var_32 = wp::mod(var_0, var_30);
        var_33 = wp::address(var_body_mass, var_32, var_3);
        var_35 = wp::load(var_33);
        var_34 = wp::copy(var_35);
        // subtreemass = body_subtreemass[worldid % body_subtreemass.shape[0], bodyid]            <L 3921>
        var_36 = &(var_body_subtreemass.shape);
        var_39 = wp::load(var_36);
        var_38 = wp::extract(var_39, var_37);
        var_40 = wp::mod(var_0, var_38);
        var_41 = wp::address(var_body_subtreemass, var_40, var_3);
        var_43 = wp::load(var_41);
        var_42 = wp::copy(var_43);
        // dx = xipos - com                                                                       <L 3924>
        var_44 = wp::sub(var_11, var_14);
        // dv = wp.spatial_bottom(vel) - linvel                                                   <L 3925>
        var_45 = wp::spatial_bottom(var_20);
        var_46 = wp::sub(var_45, var_23);
        // dp = dv * mass                                                                         <L 3926>
        var_47 = wp::mul(var_46, var_34);
        // dL = wp.cross(dx, dp)                                                                  <L 3927>
        var_48 = wp::cross(var_44, var_47);
        // subtree_angmom_out[worldid, bodyid] += dL                                              <L 3930>
        var_49 = wp::atomic_add(var_subtree_angmom_out, var_0, var_3, var_48);
        // wp.atomic_add(subtree_angmom_out[worldid], pid, subtree_angmom_out[worldid, bodyid])       <L 3933>
        var_50 = wp::slice_t(var_0, var_0, var_51);
        var_52 = wp::view(var_subtree_angmom_out, var_50);
        var_53 = wp::address(var_subtree_angmom_out, var_0, var_3);
        var_55 = wp::load(var_53);
        var_54 = wp::atomic_add(var_52, var_8, var_55);
        // dx = com - com_parent                                                                  <L 3936>
        var_56 = wp::sub(var_14, var_17);
        // dv = linvel - linvel_parent                                                            <L 3937>
        var_57 = wp::sub(var_23, var_26);
        // dv *= subtreemass                                                                      <L 3938>
        var_58 = wp::mul(var_57, var_42);
        // dL = wp.cross(dx, dv)                                                                  <L 3939>
        var_59 = wp::cross(var_56, var_58);
        // wp.atomic_add(subtree_angmom_out[worldid], pid, dL)                                    <L 3940>
        var_60 = wp::slice_t(var_0, var_0, var_61);
        var_62 = wp::view(var_subtree_angmom_out, var_60);
        var_63 = wp::atomic_add(var_62, var_8, var_59);
    }
}



extern "C" __global__ void _cfrc_ext_spatial_tendon_294ed7a8_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::int32> var_tendon_adr,
    wp::array_t<wp::int32> var_tendon_num,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_stiffness,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_stiffnesspoly,
    wp::array_t<wp::float32> var_tendon_damping,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_dampingpoly,
    wp::array_t<wp::float32> var_tendon_armature,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_lengthspring,
    wp::array_t<wp::int32> var_wrap_type,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::float32> var_wrap_prm,
    wp::array_t<wp::float32> var_qacc_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::int32> var_ten_wrapadr_in,
    wp::array_t<wp::int32> var_ten_wrapnum_in,
    wp::array_t<wp::float32> var_ten_J_in,
    wp::array_t<wp::float32> var_ten_length_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_wrap_obj_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_wrap_xpos_in,
    wp::array_t<wp::float32> var_ten_velocity_in,
    wp::array_t<wp::float32> var_ten_bias_coef_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::float32> var_ten_frc_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_ext_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        const wp::int32 var_6 = 1;
        bool var_7;
        wp::int32 var_8;
        wp::float32* var_9;
        wp::float32 var_10;
        wp::float32 var_11;
        bool var_12;
        wp::shape_t* var_13;
        const wp::int32 var_14 = 0;
        wp::int32 var_15;
        wp::shape_t var_16;
        wp::int32 var_17;
        wp::float32* var_18;
        wp::float32 var_19;
        wp::float32 var_20;
        wp::shape_t* var_21;
        const wp::int32 var_22 = 0;
        wp::int32 var_23;
        wp::shape_t var_24;
        wp::int32 var_25;
        wp::vec_t<2, wp::float32>* var_26;
        wp::vec_t<2, wp::float32> var_27;
        wp::vec_t<2, wp::float32> var_28;
        bool var_29;
        const wp::float32 var_30 = 0.0;
        bool var_31;
        const wp::int32 var_32 = 0;
        wp::float32 var_33;
        const wp::float32 var_34 = 0.0;
        bool var_35;
        const wp::int32 var_36 = 1;
        wp::float32 var_37;
        const wp::float32 var_38 = 0.0;
        bool var_39;
        wp::float32* var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::shape_t* var_43;
        const wp::int32 var_44 = 0;
        wp::int32 var_45;
        wp::shape_t var_46;
        wp::int32 var_47;
        wp::vec_t<2, wp::float32>* var_48;
        wp::vec_t<2, wp::float32> var_49;
        wp::vec_t<2, wp::float32> var_50;
        const wp::int32 var_51 = 0;
        wp::float32 var_52;
        const wp::int32 var_53 = 1;
        wp::float32 var_54;
        bool var_55;
        wp::float32 var_56;
        bool var_57;
        wp::float32 var_58;
        const wp::float32 var_59 = 0.0;
        wp::float32 var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        const wp::int32 var_63 = 0;
        wp::float32 var_64;
        wp::float32 var_65;
        wp::float32 var_66;
        wp::float32 var_67;
        wp::float32 var_68;
        bool var_69;
        wp::shape_t* var_70;
        const wp::int32 var_71 = 0;
        wp::int32 var_72;
        wp::shape_t var_73;
        wp::int32 var_74;
        wp::float32* var_75;
        wp::float32 var_76;
        wp::float32 var_77;
        wp::shape_t* var_78;
        const wp::int32 var_79 = 0;
        wp::int32 var_80;
        wp::shape_t var_81;
        wp::int32 var_82;
        wp::vec_t<2, wp::float32>* var_83;
        wp::vec_t<2, wp::float32> var_84;
        wp::vec_t<2, wp::float32> var_85;
        bool var_86;
        const wp::float32 var_87 = 0.0;
        bool var_88;
        const wp::int32 var_89 = 0;
        wp::float32 var_90;
        const wp::float32 var_91 = 0.0;
        bool var_92;
        const wp::int32 var_93 = 1;
        wp::float32 var_94;
        const wp::float32 var_95 = 0.0;
        bool var_96;
        wp::float32* var_97;
        wp::float32 var_98;
        wp::float32 var_99;
        wp::float32 var_100;
        const wp::int32 var_101 = 1;
        wp::float32 var_102;
        wp::float32 var_103;
        wp::float32 var_104;
        wp::float32 var_105;
        wp::float32 var_106;
        wp::shape_t* var_107;
        const wp::int32 var_108 = 0;
        wp::int32 var_109;
        wp::shape_t var_110;
        wp::int32 var_111;
        wp::float32* var_112;
        wp::float32 var_113;
        wp::float32 var_114;
        const wp::float32 var_115 = 0.0;
        bool var_116;
        wp::float32* var_117;
        wp::float32 var_118;
        wp::float32 var_119;
        wp::int32* var_120;
        wp::int32 var_121;
        wp::int32 var_122;
        wp::int32* var_123;
        wp::int32 var_124;
        wp::int32 var_125;
        wp::range_t var_126;
        wp::int32 var_127;
        wp::int32 var_128;
        wp::float32* var_129;
        wp::int32* var_130;
        wp::float32* var_131;
        wp::int32 var_132;
        wp::float32 var_133;
        wp::float32 var_134;
        wp::float32 var_135;
        wp::float32 var_136;
        wp::float32 var_137;
        wp::float32 var_138;
        wp::float32 var_139;
        const wp::float32 var_140 = 0.0;
        bool var_141;
        wp::int32* var_142;
        wp::int32 var_143;
        wp::int32 var_144;
        wp::int32* var_145;
        wp::int32 var_146;
        wp::int32 var_147;
        wp::int32* var_148;
        wp::int32 var_149;
        wp::int32 var_150;
        wp::float32 var_151;
        const wp::int32 var_152 = -1;
        wp::int32 var_153;
        const wp::float32 var_154 = 0.0;
        wp::vec_t<3, wp::float32> var_155;
        const wp::float32 var_156 = 0.0;
        wp::vec_t<3, wp::float32> var_157;
        wp::range_t var_158;
        wp::int32 var_159;
        wp::int32 var_160;
        wp::int32* var_161;
        wp::int32 var_162;
        wp::int32 var_163;
        wp::int32* var_164;
        wp::int32 var_165;
        wp::int32 var_166;
        const wp::int32 var_167 = 2;
        bool var_168;
        wp::float32* var_169;
        wp::float32 var_170;
        wp::float32 var_171;
        const wp::int32 var_172 = -1;
        const wp::int32 var_173 = 1;
        wp::int32 var_174;
        const wp::int32 var_175 = 0;
        wp::int32 var_176;
        const wp::int32 var_177 = 0;
        wp::int32 var_178;
        const wp::int32 var_179 = 3;
        bool var_180;
        const wp::int32 var_181 = 1;
        wp::int32* var_182;
        wp::int32 var_183;
        wp::int32 var_184;
        wp::int32 var_185;
        wp::int32 var_186;
        wp::int32* var_187;
        wp::int32 var_188;
        wp::int32 var_189;
        bool var_190;
        bool var_191;
        const wp::int32 var_192 = 2;
        wp::int32 var_193;
        wp::vec_t<2, wp::int32>* var_194;
        const wp::int32 var_195 = 2;
        wp::int32 var_196;
        wp::int32 var_197;
        wp::vec_t<2, wp::int32> var_198;
        bool var_199;
        const wp::int32 var_200 = 2;
        wp::int32 var_201;
        wp::int32 var_202;
        wp::int32 var_203;
        const wp::float32 var_204 = 0.0;
        wp::vec_t<3, wp::float32> var_205;
        bool var_206;
        const wp::int32 var_207 = 0;
        bool var_208;
        const wp::int32 var_209 = 0;
        bool var_210;
        wp::int32* var_211;
        wp::vec_t<3, wp::float32>* var_212;
        wp::int32 var_213;
        wp::vec_t<3, wp::float32> var_214;
        wp::vec_t<3, wp::float32> var_215;
        wp::vec_t<3, wp::float32> var_216;
        const wp::int32 var_217 = 2;
        wp::range_t var_218;
        wp::int32 var_219;
        bool var_220;
        const wp::int32 var_221 = 2;
        wp::int32 var_222;
        wp::vec_t<6, wp::float32>* var_223;
        wp::vec_t<6, wp::float32> var_224;
        wp::vec_t<6, wp::float32> var_225;
        const wp::int32 var_226 = 2;
        wp::int32 var_227;
        const wp::int32 var_228 = 0;
        bool var_229;
        wp::vec_t<3, wp::float32> var_230;
        wp::vec_t<3, wp::float32> var_231;
        wp::vec_t<3, wp::float32> var_232;
        bool var_233;
        const wp::int32 var_234 = 0;
        bool var_235;
        bool var_236;
        wp::vec_t<3, wp::float32> var_237;
        wp::vec_t<3, wp::float32> var_238;
        wp::float32 var_239;
        const wp::float32 var_240 = 0.0;
        bool var_241;
        wp::vec_t<3, wp::float32> var_242;
        const wp::int32 var_243 = 0;
        bool var_244;
        wp::slice_t var_245;
        const wp::int32 var_246 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_247;
        wp::vec_t<3, wp::float32> var_248;
        wp::vec_t<3, wp::float32> var_249;
        wp::vec_t<3, wp::float32> var_250;
        wp::vec_t<6, wp::float32> var_251;
        wp::vec_t<6, wp::float32> var_252;
        const wp::int32 var_253 = 0;
        bool var_254;
        wp::slice_t var_255;
        const wp::int32 var_256 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_257;
        wp::vec_t<3, wp::float32> var_258;
        wp::vec_t<3, wp::float32> var_259;
        wp::vec_t<3, wp::float32> var_260;
        wp::vec_t<6, wp::float32> var_261;
        wp::vec_t<6, wp::float32> var_262;
        wp::int32 var_263;
        wp::vec_t<3, wp::float32> var_264;
        wp::vec_t<3, wp::float32> var_265;
        const wp::int32 var_266 = 1;
        wp::int32 var_267;
        //---------
        // forward
        // def _cfrc_ext_spatial_tendon(                                                          <L 1815>
        // worldid, tenid = wp.tid()                                                              <L 1852>
        builtin_tid2d(var_0, var_1);
        // adr = tendon_adr[tenid]                                                                <L 1855>
        var_2 = wp::address(var_tendon_adr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if wrap_type[adr] == WrapType.JOINT:                                                   <L 1856>
        var_5 = wp::address(var_wrap_type, var_3);
        var_8 = wp::load(var_5);
        var_7 = (var_8 == var_6);
        if (var_7) {
            // return                                                                             <L 1857>
            continue;
        }
        // frc = ten_frc_in[worldid, tenid]                                                       <L 1859>
        var_9 = wp::address(var_ten_frc_in, var_0, var_1);
        var_11 = wp::load(var_9);
        var_10 = wp::copy(var_11);
        // if not dsbl_spring:                                                                    <L 1862>
        var_12 = wp::unot(var_dsbl_spring);
        if (var_12) {
            // stiffness = tendon_stiffness[worldid % tendon_stiffness.shape[0], tenid]           <L 1863>
            var_13 = &(var_tendon_stiffness.shape);
            var_16 = wp::load(var_13);
            var_15 = wp::extract(var_16, var_14);
            var_17 = wp::mod(var_0, var_15);
            var_18 = wp::address(var_tendon_stiffness, var_17, var_1);
            var_20 = wp::load(var_18);
            var_19 = wp::copy(var_20);
            // spoly = tendon_stiffnesspoly[worldid % tendon_stiffnesspoly.shape[0], tenid]       <L 1864>
            var_21 = &(var_tendon_stiffnesspoly.shape);
            var_24 = wp::load(var_21);
            var_23 = wp::extract(var_24, var_22);
            var_25 = wp::mod(var_0, var_23);
            var_26 = wp::address(var_tendon_stiffnesspoly, var_25, var_1);
            var_28 = wp::load(var_26);
            var_27 = wp::copy(var_28);
            // if stiffness != 0.0 or spoly[0] != 0.0 or spoly[1] != 0.0:                         <L 1865>
            var_31 = (var_19 != var_30);
            var_29 = var_31;
            if (!var_29) {
                var_33 = wp::extract(var_27, var_32);
                var_35 = (var_33 != var_34);
                var_29 = var_29 || var_35;
            }
            if (!var_29) {
                var_37 = wp::extract(var_27, var_36);
                var_39 = (var_37 != var_38);
                var_29 = var_29 || var_39;
            }
            if (var_29) {
                // length = ten_length_in[worldid, tenid]                                         <L 1866>
                var_40 = wp::address(var_ten_length_in, var_0, var_1);
                var_42 = wp::load(var_40);
                var_41 = wp::copy(var_42);
                // lengthspring = tendon_lengthspring[worldid % tendon_lengthspring.shape[0], tenid]       <L 1867>
                var_43 = &(var_tendon_lengthspring.shape);
                var_46 = wp::load(var_43);
                var_45 = wp::extract(var_46, var_44);
                var_47 = wp::mod(var_0, var_45);
                var_48 = wp::address(var_tendon_lengthspring, var_47, var_1);
                var_50 = wp::load(var_48);
                var_49 = wp::copy(var_50);
                // lower = lengthspring[0]                                                        <L 1868>
                var_52 = wp::extract(var_49, var_51);
                // upper = lengthspring[1]                                                        <L 1869>
                var_54 = wp::extract(var_49, var_53);
                // x = wp.where(length > upper, length - upper, wp.where(length < lower, length - lower, 0.0))       <L 1870>
                var_55 = (var_41 > var_54);
                var_56 = wp::sub(var_41, var_54);
                var_57 = (var_41 < var_52);
                var_58 = wp::sub(var_41, var_52);
                var_60 = wp::where(var_57, var_58, var_59);
                var_61 = wp::where(var_55, var_56, var_60);
                // frc += -x * util_misc._poly_force(stiffness, spoly, x, 0)                      <L 1871>
                var_62 = wp::neg(var_61);
                var_64 = _poly_force_0(var_19, var_27, var_61, var_63);
                var_65 = wp::mul(var_62, var_64);
                var_66 = wp::add(var_10, var_65);
            }
            var_67 = wp::where(var_29, var_66, var_10);
        }
        var_68 = wp::where(var_12, var_67, var_10);
        // if not dsbl_damper:                                                                    <L 1873>
        var_69 = wp::unot(var_dsbl_damper);
        if (var_69) {
            // damping = tendon_damping[worldid % tendon_damping.shape[0], tenid]                 <L 1874>
            var_70 = &(var_tendon_damping.shape);
            var_73 = wp::load(var_70);
            var_72 = wp::extract(var_73, var_71);
            var_74 = wp::mod(var_0, var_72);
            var_75 = wp::address(var_tendon_damping, var_74, var_1);
            var_77 = wp::load(var_75);
            var_76 = wp::copy(var_77);
            // dpoly = tendon_dampingpoly[worldid % tendon_dampingpoly.shape[0], tenid]           <L 1875>
            var_78 = &(var_tendon_dampingpoly.shape);
            var_81 = wp::load(var_78);
            var_80 = wp::extract(var_81, var_79);
            var_82 = wp::mod(var_0, var_80);
            var_83 = wp::address(var_tendon_dampingpoly, var_82, var_1);
            var_85 = wp::load(var_83);
            var_84 = wp::copy(var_85);
            // if damping != 0.0 or dpoly[0] != 0.0 or dpoly[1] != 0.0:                           <L 1876>
            var_88 = (var_76 != var_87);
            var_86 = var_88;
            if (!var_86) {
                var_90 = wp::extract(var_84, var_89);
                var_92 = (var_90 != var_91);
                var_86 = var_86 || var_92;
            }
            if (!var_86) {
                var_94 = wp::extract(var_84, var_93);
                var_96 = (var_94 != var_95);
                var_86 = var_86 || var_96;
            }
            if (var_86) {
                // v = ten_velocity_in[worldid, tenid]                                            <L 1877>
                var_97 = wp::address(var_ten_velocity_in, var_0, var_1);
                var_99 = wp::load(var_97);
                var_98 = wp::copy(var_99);
                // frc += -v * util_misc._poly_force(damping, dpoly, v, 1)                        <L 1878>
                var_100 = wp::neg(var_98);
                var_102 = _poly_force_0(var_76, var_84, var_98, var_101);
                var_103 = wp::mul(var_100, var_102);
                var_104 = wp::add(var_68, var_103);
            }
            var_105 = wp::where(var_86, var_104, var_68);
        }
        var_106 = wp::where(var_69, var_105, var_68);
        // armature = tendon_armature[worldid % tendon_armature.shape[0], tenid]                  <L 1881>
        var_107 = &(var_tendon_armature.shape);
        var_110 = wp::load(var_107);
        var_109 = wp::extract(var_110, var_108);
        var_111 = wp::mod(var_0, var_109);
        var_112 = wp::address(var_tendon_armature, var_111, var_1);
        var_114 = wp::load(var_112);
        var_113 = wp::copy(var_114);
        // if armature != 0.0:                                                                    <L 1882>
        var_116 = (var_113 != var_115);
        if (var_116) {
            // acc = ten_bias_coef_in[worldid, tenid]                                             <L 1883>
            var_117 = wp::address(var_ten_bias_coef_in, var_0, var_1);
            var_119 = wp::load(var_117);
            var_118 = wp::copy(var_119);
            // rownnz = ten_J_rownnz[tenid]                                                       <L 1884>
            var_120 = wp::address(var_ten_J_rownnz, var_1);
            var_122 = wp::load(var_120);
            var_121 = wp::copy(var_122);
            // rowadr = ten_J_rowadr[tenid]                                                       <L 1885>
            var_123 = wp::address(var_ten_J_rowadr, var_1);
            var_125 = wp::load(var_123);
            var_124 = wp::copy(var_125);
            // for i in range(rownnz):                                                            <L 1886>
            var_126 = wp::range(var_121);
            start_for_1:;
                if (iter_cmp(var_126) == 0) goto end_for_1;
                var_127 = wp::iter_next(var_126);
                // sparseid = rowadr + i                                                          <L 1887>
                var_128 = wp::add(var_124, var_127);
                // acc += ten_J_in[worldid, sparseid] * qacc_in[worldid, ten_J_colind[sparseid]]       <L 1888>
                var_129 = wp::address(var_ten_J_in, var_0, var_128);
                var_130 = wp::address(var_ten_J_colind, var_128);
                var_132 = wp::load(var_130);
                var_131 = wp::address(var_qacc_in, var_0, var_132);
                var_134 = wp::load(var_129);
                var_135 = wp::load(var_131);
                var_133 = wp::mul(var_134, var_135);
                var_136 = wp::add(var_118, var_133);
                wp::assign(var_118, var_136);
                goto start_for_1;
            end_for_1:;
            // frc -= armature * acc                                                              <L 1889>
            var_137 = wp::mul(var_113, var_118);
            var_138 = wp::sub(var_106, var_137);
        }
        var_139 = wp::where(var_116, var_138, var_106);
        // if frc == 0.0:                                                                         <L 1891>
        var_141 = (var_139 == var_140);
        if (var_141) {
            // return                                                                             <L 1892>
            continue;
        }
        // num = tendon_num[tenid]                                                                <L 1894>
        var_142 = wp::address(var_tendon_num, var_1);
        var_144 = wp::load(var_142);
        var_143 = wp::copy(var_144);
        // p = ten_wrapadr_in[worldid, tenid]                                                     <L 1895>
        var_145 = wp::address(var_ten_wrapadr_in, var_0, var_1);
        var_147 = wp::load(var_145);
        var_146 = wp::copy(var_147);
        // pend = p + ten_wrapnum_in[worldid, tenid]                                              <L 1896>
        var_148 = wp::address(var_ten_wrapnum_in, var_0, var_1);
        var_150 = wp::load(var_148);
        var_149 = wp::add(var_146, var_150);
        // scaled_frc = frc                                                                       <L 1897>
        var_151 = wp::copy(var_139);
        // prevbody = int(-1)                                                                     <L 1898>
        var_153 = wp::int(var_152);
        // prevpnt = wp.vec3(0.0)                                                                 <L 1899>
        var_155 = wp::vec_t<3, wp::float32>(var_154);
        // prevcom = wp.vec3(0.0)                                                                 <L 1900>
        var_157 = wp::vec_t<3, wp::float32>(var_156);
        // for j_offset in range(num):                                                            <L 1902>
        var_158 = wp::range(var_143);
        start_for_4:;
            if (iter_cmp(var_158) == 0) goto end_for_4;
            var_159 = wp::iter_next(var_158);
            // j = adr + j_offset                                                                 <L 1903>
            var_160 = wp::add(var_3, var_159);
            // wtype = wrap_type[j]                                                               <L 1904>
            var_161 = wp::address(var_wrap_type, var_160);
            var_163 = wp::load(var_161);
            var_162 = wp::copy(var_163);
            // objid = wrap_objid[j]                                                              <L 1905>
            var_164 = wp::address(var_wrap_objid, var_160);
            var_166 = wp::load(var_164);
            var_165 = wp::copy(var_166);
            // if wtype == WrapType.PULLEY:                                                       <L 1908>
            var_168 = (var_162 == var_167);
            if (var_168) {
                // scaled_frc = frc / wrap_prm[j]                                                 <L 1909>
                var_169 = wp::address(var_wrap_prm, var_160);
                var_171 = wp::load(var_169);
                var_170 = wp::div(var_139, var_171);
                // prevbody = -1                                                                  <L 1910>
                // p += 1                                                                         <L 1911>
                var_174 = wp::add(var_146, var_173);
                // continue                                                                       <L 1912>
                wp::assign(var_146, var_174);
                wp::assign(var_151, var_170);
                wp::assign(var_153, var_172);
                goto start_for_4;
            }
            // npnt = int(0)                                                                      <L 1915>
            var_176 = wp::int(var_175);
            // body = int(0)                                                                      <L 1916>
            var_178 = wp::int(var_177);
            // if wtype == WrapType.SITE:                                                         <L 1917>
            var_180 = (var_162 == var_179);
            if (var_180) {
                // npnt = 1                                                                       <L 1918>
                // body = site_bodyid[objid]                                                      <L 1919>
                var_182 = wp::address(var_site_bodyid, var_165);
                var_184 = wp::load(var_182);
                var_183 = wp::copy(var_184);
            }
            var_185 = wp::where(var_180, var_181, var_176);
            var_186 = wp::where(var_180, var_183, var_178);
            if (!var_180) {
                // body = geom_bodyid[objid]                                                      <L 1921>
                var_187 = wp::address(var_geom_bodyid, var_165);
                var_189 = wp::load(var_187);
                var_188 = wp::copy(var_189);
                // if p < pend and wrap_obj_in[worldid, p // 2][p % 2] == objid:                  <L 1922>
                var_191 = (var_146 < var_149);
                var_190 = var_191;
                if (var_190) {
                    var_193 = wp::floordiv(var_146, var_192);
                    var_194 = wp::address(var_wrap_obj_in, var_0, var_193);
                    var_196 = wp::mod(var_146, var_195);
                    var_198 = wp::load(var_194);
                    var_197 = wp::extract(var_198, var_196);
                    var_199 = (var_197 == var_165);
                    var_190 = var_190 && var_199;
                }
                if (var_190) {
                    // npnt = 2                                                                   <L 1923>
                }
                var_201 = wp::where(var_190, var_200, var_185);
            }
            var_202 = wp::where(var_180, var_185, var_201);
            var_203 = wp::where(var_180, var_186, var_188);
            // com = wp.vec3(0.0)                                                                 <L 1925>
            var_205 = wp::vec_t<3, wp::float32>(var_204);
            // if npnt > 0 and body != 0:                                                         <L 1926>
            var_208 = (var_202 > var_207);
            var_206 = var_208;
            if (var_206) {
                var_210 = (var_203 != var_209);
                var_206 = var_206 && var_210;
            }
            if (var_206) {
                // com = subtree_com_in[worldid, body_rootid[body]]                               <L 1927>
                var_211 = wp::address(var_body_rootid, var_203);
                var_213 = wp::load(var_211);
                var_212 = wp::address(var_subtree_com_in, var_0, var_213);
                var_215 = wp::load(var_212);
                var_214 = wp::copy(var_215);
            }
            var_216 = wp::where(var_206, var_214, var_205);
            // for k in range(2):                                                                 <L 1929>
            var_218 = wp::range(var_217);
            start_for_6:;
                if (iter_cmp(var_218) == 0) goto end_for_6;
                var_219 = wp::iter_next(var_218);
                // if k >= npnt:                                                                  <L 1930>
                var_220 = (var_219 >= var_202);
                if (var_220) {
                    // continue                                                                   <L 1931>
                    goto start_for_6;
                }
                // wrap_pos = wrap_xpos_in[worldid, p // 2]                                       <L 1933>
                var_222 = wp::floordiv(var_146, var_221);
                var_223 = wp::address(var_wrap_xpos_in, var_0, var_222);
                var_225 = wp::load(var_223);
                var_224 = wp::copy(var_225);
                // pnt = wp.where(p % 2 == 0, wp.spatial_top(wrap_pos), wp.spatial_bottom(wrap_pos))       <L 1934>
                var_227 = wp::mod(var_146, var_226);
                var_229 = (var_227 == var_228);
                var_230 = wp::spatial_top(var_224);
                var_231 = wp::spatial_bottom(var_224);
                var_232 = wp::where(var_229, var_230, var_231);
                // if prevbody >= 0 and body != prevbody:                                         <L 1935>
                var_235 = (var_153 >= var_234);
                var_233 = var_235;
                if (var_233) {
                    var_236 = (var_203 != var_153);
                    var_233 = var_233 && var_236;
                }
                if (var_233) {
                    // diff = pnt - prevpnt                                                       <L 1936>
                    var_237 = wp::sub(var_232, var_155);
                    // vec, norm = math.normalize_with_norm(diff)                                 <L 1937>
                    normalize_with_norm_0(var_237, var_238, var_239);
                    // if norm > 0.0:                                                             <L 1938>
                    var_241 = (var_239 > var_240);
                    if (var_241) {
                        // force = vec * scaled_frc                                               <L 1939>
                        var_242 = wp::mul(var_238, var_151);
                        // if body != 0:                                                          <L 1940>
                        var_244 = (var_203 != var_243);
                        if (var_244) {
                            // wp.atomic_add(cfrc_ext_out[worldid], body, wp.spatial_vector(-wp.cross(com - pnt, force), force))       <L 1941>
                            var_245 = wp::slice_t(var_0, var_0, var_246);
                            var_247 = wp::view(var_cfrc_ext_out, var_245);
                            var_248 = wp::sub(var_216, var_232);
                            var_249 = wp::cross(var_248, var_242);
                            var_250 = wp::neg(var_249);
                            var_251 = wp::vec_t<6, wp::float32>(var_250, var_242);
                            var_252 = wp::atomic_add(var_247, var_203, var_251);
                        }
                        // if prevbody != 0:                                                      <L 1942>
                        var_254 = (var_153 != var_253);
                        if (var_254) {
                            // wp.atomic_sub(                                                     <L 1943>
                            // cfrc_ext_out[worldid],                                             <L 1944>
                            var_255 = wp::slice_t(var_0, var_0, var_256);
                            var_257 = wp::view(var_cfrc_ext_out, var_255);
                            // prevbody,                                                          <L 1945>
                            // wp.spatial_vector(-wp.cross(prevcom - prevpnt, force), force),       <L 1946>
                            var_258 = wp::sub(var_157, var_155);
                            var_259 = wp::cross(var_258, var_242);
                            var_260 = wp::neg(var_259);
                            var_261 = wp::vec_t<6, wp::float32>(var_260, var_242);
                            var_262 = wp::atomic_sub(var_257, var_153, var_261);
                        }
                    }
                }
                // prevbody = body                                                                <L 1948>
                var_263 = wp::copy(var_203);
                // prevpnt = pnt                                                                  <L 1949>
                var_264 = wp::copy(var_232);
                // prevcom = com                                                                  <L 1950>
                var_265 = wp::copy(var_216);
                // p += 1                                                                         <L 1951>
                var_267 = wp::add(var_146, var_266);
                wp::assign(var_146, var_267);
                wp::assign(var_153, var_263);
                wp::assign(var_155, var_264);
                wp::assign(var_157, var_265);
                goto start_for_6;
            end_for_6:;
            goto start_for_4;
        end_for_4:;
    }
}



extern "C" __global__ void _tendon_dot_62db264d_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_dof_jntid,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::int32> var_tendon_adr,
    wp::array_t<wp::int32> var_tendon_num,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_armature,
    wp::array_t<wp::int32> var_wrap_type,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::float32> var_wrap_prm,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_dot_in,
    wp::array_t<wp::float32> var_ten_Jdot_out)
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
        wp::int32 var_1;
        wp::shape_t* var_2;
        const wp::int32 var_3 = 0;
        wp::int32 var_4;
        wp::shape_t var_5;
        wp::int32 var_6;
        wp::float32* var_7;
        wp::float32 var_8;
        wp::float32 var_9;
        const wp::float32 var_10 = 0.0;
        bool var_11;
        wp::int32* var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        wp::int32* var_15;
        const wp::int32 var_16 = 1;
        bool var_17;
        wp::int32 var_18;
        const wp::float32 var_19 = 1.0;
        wp::float32 var_20;
        wp::int32* var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        const wp::int32 var_24 = 0;
        wp::int32 var_25;
        const wp::int32 var_26 = 1;
        wp::int32 var_27;
        bool var_28;
        wp::int32 var_29;
        const wp::int32 var_30 = 0;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        wp::int32 var_35;
        const wp::int32 var_36 = 1;
        wp::int32 var_37;
        wp::int32* var_38;
        wp::int32 var_39;
        wp::int32 var_40;
        wp::int32 var_41;
        const wp::int32 var_42 = 0;
        wp::int32 var_43;
        wp::int32* var_44;
        wp::int32 var_45;
        wp::int32 var_46;
        wp::int32 var_47;
        const wp::int32 var_48 = 1;
        wp::int32 var_49;
        wp::int32* var_50;
        wp::int32 var_51;
        wp::int32 var_52;
        const wp::int32 var_53 = 2;
        bool var_54;
        bool var_55;
        bool var_56;
        bool var_57;
        wp::int32 var_58;
        wp::float32* var_59;
        wp::float32 var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        const wp::int32 var_63 = 1;
        wp::int32 var_64;
        wp::vec_t<3, wp::float32>* var_65;
        wp::vec_t<3, wp::float32> var_66;
        wp::vec_t<3, wp::float32> var_67;
        wp::int32* var_68;
        wp::int32 var_69;
        wp::int32 var_70;
        wp::vec_t<6, wp::float32>* var_71;
        wp::vec_t<6, wp::float32> var_72;
        wp::vec_t<6, wp::float32> var_73;
        wp::int32* var_74;
        wp::vec_t<3, wp::float32>* var_75;
        wp::int32 var_76;
        wp::vec_t<3, wp::float32> var_77;
        wp::vec_t<3, wp::float32> var_78;
        wp::vec_t<3, wp::float32> var_79;
        wp::vec_t<3, wp::float32> var_80;
        wp::vec_t<3, wp::float32> var_81;
        wp::vec_t<3, wp::float32> var_82;
        wp::vec_t<3, wp::float32> var_83;
        bool var_84;
        const wp::int32 var_85 = 4;
        bool var_86;
        const wp::int32 var_87 = 5;
        bool var_88;
        wp::int32* var_89;
        wp::int32 var_90;
        wp::int32 var_91;
        wp::vec_t<3, wp::float32>* var_92;
        wp::vec_t<3, wp::float32> var_93;
        wp::vec_t<3, wp::float32> var_94;
        wp::vec_t<6, wp::float32>* var_95;
        wp::vec_t<6, wp::float32> var_96;
        wp::vec_t<6, wp::float32> var_97;
        wp::int32* var_98;
        wp::vec_t<3, wp::float32>* var_99;
        wp::int32 var_100;
        wp::vec_t<3, wp::float32> var_101;
        wp::vec_t<3, wp::float32> var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::vec_t<3, wp::float32> var_104;
        wp::vec_t<3, wp::float32> var_105;
        wp::vec_t<3, wp::float32> var_106;
        wp::vec_t<3, wp::float32> var_107;
        bool var_108;
        wp::vec_t<3, wp::float32> var_109;
        wp::vec_t<3, wp::float32> var_110;
        wp::float32 var_111;
        wp::vec_t<3, wp::float32> var_112;
        wp::vec_t<3, wp::float32> var_113;
        wp::vec_t<3, wp::float32> var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::vec_t<3, wp::float32> var_116;
        wp::vec_t<3, wp::float32> var_117;
        wp::vec_t<3, wp::float32> var_118;
        wp::vec_t<3, wp::float32> var_119;
        wp::vec_t<3, wp::float32> var_120;
        wp::vec_t<3, wp::float32> var_121;
        wp::vec_t<3, wp::float32> var_122;
        wp::float32 var_123;
        wp::float32 var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32> var_126;
        const wp::float32 var_127 = 1e-15;
        bool var_128;
        wp::vec_t<3, wp::float32> var_129;
        wp::vec_t<3, wp::float32> var_130;
        const wp::float32 var_131 = 0.0;
        wp::vec_t<3, wp::float32> var_132;
        wp::vec_t<3, wp::float32> var_133;
        wp::int32* var_134;
        wp::int32 var_135;
        wp::int32 var_136;
        wp::int32* var_137;
        wp::int32 var_138;
        wp::int32 var_139;
        const wp::float32 var_140 = 1.0;
        wp::float32 var_141;
        wp::float32 var_142;
        wp::float32 var_143;
        const wp::int32 var_144 = 1;
        wp::int32 var_145;
        //---------
        // forward
        // def _tendon_dot(                                                                       <L 2229>
        // worldid, tenid = wp.tid()                                                              <L 2257>
        builtin_tid2d(var_0, var_1);
        // armature = tendon_armature[worldid % tendon_armature.shape[0], tenid]                  <L 2259>
        var_2 = &(var_tendon_armature.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_0, var_4);
        var_7 = wp::address(var_tendon_armature, var_6, var_1);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // if armature == 0.0:                                                                    <L 2260>
        var_11 = (var_8 == var_10);
        if (var_11) {
            // return                                                                             <L 2261>
            continue;
        }
        // adr = tendon_adr[tenid]                                                                <L 2264>
        var_12 = wp::address(var_tendon_adr, var_1);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // if wrap_type[adr] == WrapType.JOINT:                                                   <L 2265>
        var_15 = wp::address(var_wrap_type, var_13);
        var_18 = wp::load(var_15);
        var_17 = (var_18 == var_16);
        if (var_17) {
            // return                                                                             <L 2266>
            continue;
        }
        // divisor = float(1.0)                                                                   <L 2269>
        var_20 = wp::float(var_19);
        // num = tendon_num[tenid]                                                                <L 2270>
        var_21 = wp::address(var_tendon_num, var_1);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // j = int(0)                                                                             <L 2271>
        var_25 = wp::int(var_24);
        // while j < num - 1:                                                                     <L 2272>
        start_while_2:;
        var_27 = wp::sub(var_22, var_26);
        var_28 = (var_25 < var_27);
        if ((var_28) == false) goto end_while_2;
            // type0 = wrap_type[adr + j + 0]                                                     <L 2274>
            var_29 = wp::add(var_13, var_25);
            var_31 = wp::add(var_29, var_30);
            var_32 = wp::address(var_wrap_type, var_31);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // type1 = wrap_type[adr + j + 1]                                                     <L 2275>
            var_35 = wp::add(var_13, var_25);
            var_37 = wp::add(var_35, var_36);
            var_38 = wp::address(var_wrap_type, var_37);
            var_40 = wp::load(var_38);
            var_39 = wp::copy(var_40);
            // id0 = wrap_objid[adr + j + 0]                                                      <L 2276>
            var_41 = wp::add(var_13, var_25);
            var_43 = wp::add(var_41, var_42);
            var_44 = wp::address(var_wrap_objid, var_43);
            var_46 = wp::load(var_44);
            var_45 = wp::copy(var_46);
            // id1 = wrap_objid[adr + j + 1]                                                      <L 2277>
            var_47 = wp::add(var_13, var_25);
            var_49 = wp::add(var_47, var_48);
            var_50 = wp::address(var_wrap_objid, var_49);
            var_52 = wp::load(var_50);
            var_51 = wp::copy(var_52);
            // pulley = WrapType.PULLEY                                                           <L 2280>
            // if (type0 == pulley) or (type1 == pulley):                                         <L 2281>
            var_55 = (var_33 == var_53);
            var_54 = var_55;
            if (!var_54) {
                var_56 = (var_39 == var_53);
                var_54 = var_54 || var_56;
            }
            if (var_54) {
                // if type0 == pulley:                                                            <L 2283>
                var_57 = (var_33 == var_53);
                if (var_57) {
                    // divisor = wrap_prm[adr + j]                                                <L 2284>
                    var_58 = wp::add(var_13, var_25);
                    var_59 = wp::address(var_wrap_prm, var_58);
                    var_61 = wp::load(var_59);
                    var_60 = wp::copy(var_61);
                }
                var_62 = wp::where(var_57, var_60, var_20);
                // j += 1                                                                         <L 2286>
                var_64 = wp::add(var_25, var_63);
                // continue                                                                       <L 2287>
                wp::assign(var_20, var_62);
                wp::assign(var_25, var_64);
                goto start_while_2;
            }
            // wpnt0 = site_xpos_in[worldid, id0]                                                 <L 2290>
            var_65 = wp::address(var_site_xpos_in, var_0, var_45);
            var_67 = wp::load(var_65);
            var_66 = wp::copy(var_67);
            // wbody0 = site_bodyid[id0]                                                          <L 2292>
            var_68 = wp::address(var_site_bodyid, var_45);
            var_70 = wp::load(var_68);
            var_69 = wp::copy(var_70);
            // cvel0 = cvel_in[worldid, wbody0]                                                   <L 2293>
            var_71 = wp::address(var_cvel_in, var_0, var_69);
            var_73 = wp::load(var_71);
            var_72 = wp::copy(var_73);
            // subtree_com0 = subtree_com_in[worldid, body_rootid[wbody0]]                        <L 2294>
            var_74 = wp::address(var_body_rootid, var_69);
            var_76 = wp::load(var_74);
            var_75 = wp::address(var_subtree_com_in, var_0, var_76);
            var_78 = wp::load(var_75);
            var_77 = wp::copy(var_78);
            // offset0 = wpnt0 - subtree_com0                                                     <L 2295>
            var_79 = wp::sub(var_66, var_77);
            // pvel_lin0 = wp.spatial_bottom(cvel0) - wp.cross(offset0, wp.spatial_top(cvel0))       <L 2296>
            var_80 = wp::spatial_bottom(var_72);
            var_81 = wp::spatial_top(var_72);
            var_82 = wp::cross(var_79, var_81);
            var_83 = wp::sub(var_80, var_82);
            // if (type1 == WrapType.SPHERE) or (type1 == WrapType.CYLINDER):                     <L 2299>
            var_86 = (var_39 == var_85);
            var_84 = var_86;
            if (!var_84) {
                var_88 = (var_39 == var_87);
                var_84 = var_84 || var_88;
            }
            if (var_84) {
                // return                                                                         <L 2301>
                continue;
            }
            // wbody1 = site_bodyid[id1]                                                          <L 2304>
            var_89 = wp::address(var_site_bodyid, var_51);
            var_91 = wp::load(var_89);
            var_90 = wp::copy(var_91);
            // wpnt1 = site_xpos_in[worldid, id1]                                                 <L 2305>
            var_92 = wp::address(var_site_xpos_in, var_0, var_51);
            var_94 = wp::load(var_92);
            var_93 = wp::copy(var_94);
            // cvel1 = cvel_in[worldid, wbody1]                                                   <L 2307>
            var_95 = wp::address(var_cvel_in, var_0, var_90);
            var_97 = wp::load(var_95);
            var_96 = wp::copy(var_97);
            // subtree_com1 = subtree_com_in[worldid, body_rootid[wbody1]]                        <L 2308>
            var_98 = wp::address(var_body_rootid, var_90);
            var_100 = wp::load(var_98);
            var_99 = wp::address(var_subtree_com_in, var_0, var_100);
            var_102 = wp::load(var_99);
            var_101 = wp::copy(var_102);
            // offset1 = wpnt1 - subtree_com1                                                     <L 2309>
            var_103 = wp::sub(var_93, var_101);
            // pvel_lin1 = wp.spatial_bottom(cvel1) - wp.cross(offset1, wp.spatial_top(cvel1))       <L 2310>
            var_104 = wp::spatial_bottom(var_96);
            var_105 = wp::spatial_top(var_96);
            var_106 = wp::cross(var_103, var_105);
            var_107 = wp::sub(var_104, var_106);
            // if wbody0 != wbody1:                                                               <L 2313>
            var_108 = (var_69 != var_90);
            if (var_108) {
                // dpnt, norm = math.normalize_with_norm(wpnt1 - wpnt0)                           <L 2315>
                var_109 = wp::sub(var_93, var_66);
                normalize_with_norm_0(var_109, var_110, var_111);
                // wvel0 = wp.spatial_bottom(cvel0) - wp.cross(wpnt0 - subtree_com0, wp.spatial_top(cvel0))       <L 2318>
                var_112 = wp::spatial_bottom(var_72);
                var_113 = wp::sub(var_66, var_77);
                var_114 = wp::spatial_top(var_72);
                var_115 = wp::cross(var_113, var_114);
                var_116 = wp::sub(var_112, var_115);
                // wvel1 = wp.spatial_bottom(cvel1) - wp.cross(wpnt1 - subtree_com1, wp.spatial_top(cvel1))       <L 2319>
                var_117 = wp::spatial_bottom(var_96);
                var_118 = wp::sub(var_93, var_101);
                var_119 = wp::spatial_top(var_96);
                var_120 = wp::cross(var_118, var_119);
                var_121 = wp::sub(var_117, var_120);
                // dvel = wvel1 - wvel0                                                           <L 2320>
                var_122 = wp::sub(var_121, var_116);
                // dot = wp.dot(dpnt, dvel)                                                       <L 2321>
                var_123 = wp::dot(var_110, var_122);
                // dvel += dpnt * (-dot)                                                          <L 2322>
                var_124 = wp::neg(var_123);
                var_125 = wp::mul(var_110, var_124);
                var_126 = wp::add(var_122, var_125);
                // if norm > MJ_MINVAL:                                                           <L 2323>
                var_128 = (var_111 > var_127);
                if (var_128) {
                    // dvel /= norm                                                               <L 2324>
                    var_129 = wp::div(var_126, var_111);
                }
                var_130 = wp::where(var_128, var_129, var_126);
                if (!var_128) {
                    // dvel = wp.vec3(0.0)                                                        <L 2326>
                    var_132 = wp::vec_t<3, wp::float32>(var_131);
                }
                var_133 = wp::where(var_128, var_130, var_132);
                // rownnz = ten_J_rownnz[tenid]                                                   <L 2328>
                var_134 = wp::address(var_ten_J_rownnz, var_1);
                var_136 = wp::load(var_134);
                var_135 = wp::copy(var_136);
                // rowadr = ten_J_rowadr[tenid]                                                   <L 2329>
                var_137 = wp::address(var_ten_J_rowadr, var_1);
                var_139 = wp::load(var_137);
                var_138 = wp::copy(var_139);
                // inv_divisor = math.safe_div(float(1.0), divisor)                               <L 2330>
                var_141 = wp::float(var_140);
                var_142 = safe_div_0(var_141, var_20);
                // _accumulate_jac_dot_chain(                                                     <L 2333>
                // body_parentid,                                                                 <L 2334>
                // body_dofnum,                                                                   <L 2335>
                // body_dofadr,                                                                   <L 2336>
                // jnt_type,                                                                      <L 2337>
                // jnt_dofadr,                                                                    <L 2338>
                // dof_jntid,                                                                     <L 2339>
                // ten_J_colind,                                                                  <L 2340>
                // cdof_in,                                                                       <L 2341>
                // cvel_in,                                                                       <L 2342>
                // cdof_dot_in,                                                                   <L 2343>
                // offset0,                                                                       <L 2344>
                // pvel_lin0,                                                                     <L 2345>
                // dpnt,                                                                          <L 2346>
                // dvel,                                                                          <L 2347>
                // wbody0,                                                                        <L 2348>
                // rowadr,                                                                        <L 2349>
                // rownnz,                                                                        <L 2350>
                // -inv_divisor,                                                                  <L 2351>
                var_143 = wp::neg(var_142);
                // worldid,                                                                       <L 2352>
                // ten_Jdot_out,                                                                  <L 2353>
                _accumulate_jac_dot_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_jnt_type, var_jnt_dofadr, var_dof_jntid, var_ten_J_colind, var_cdof_in, var_cvel_in, var_cdof_dot_in, var_79, var_83, var_110, var_133, var_69, var_138, var_135, var_143, var_0, var_ten_Jdot_out);
                // _accumulate_jac_dot_chain(                                                     <L 2355>
                // body_parentid,                                                                 <L 2356>
                // body_dofnum,                                                                   <L 2357>
                // body_dofadr,                                                                   <L 2358>
                // jnt_type,                                                                      <L 2359>
                // jnt_dofadr,                                                                    <L 2360>
                // dof_jntid,                                                                     <L 2361>
                // ten_J_colind,                                                                  <L 2362>
                // cdof_in,                                                                       <L 2363>
                // cvel_in,                                                                       <L 2364>
                // cdof_dot_in,                                                                   <L 2365>
                // offset1,                                                                       <L 2366>
                // pvel_lin1,                                                                     <L 2367>
                // dpnt,                                                                          <L 2368>
                // dvel,                                                                          <L 2369>
                // wbody1,                                                                        <L 2370>
                // rowadr,                                                                        <L 2371>
                // rownnz,                                                                        <L 2372>
                // inv_divisor,                                                                   <L 2373>
                // worldid,                                                                       <L 2374>
                // ten_Jdot_out,                                                                  <L 2375>
                _accumulate_jac_dot_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_jnt_type, var_jnt_dofadr, var_dof_jntid, var_ten_J_colind, var_cdof_in, var_cvel_in, var_cdof_dot_in, var_103, var_107, var_110, var_133, var_90, var_138, var_135, var_142, var_0, var_ten_Jdot_out);
            }
            // j += 1                                                                             <L 2379>
            var_145 = wp::add(var_25, var_144);
            wp::assign(var_25, var_145);
        goto start_while_2;
        end_while_2:;
    }
}



extern "C" __global__ void _tendon_bias_coef_2556dfce_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_armature,
    wp::array_t<wp::float32> var_qvel_in,
    wp::array_t<wp::float32> var_ten_Jdot_in,
    wp::array_t<wp::float32> var_ten_bias_coef_out)
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
        wp::int32 var_1;
        wp::int32 var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::float32 var_9;
        wp::float32 var_10;
        const wp::float32 var_11 = 0.0;
        bool var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        bool var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::float32* var_21;
        wp::float32 var_22;
        wp::float32 var_23;
        const wp::float32 var_24 = 0.0;
        bool var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::slice_t var_29;
        const wp::int32 var_30 = 0;
        wp::array_t<wp::float32> var_31;
        wp::float32* var_32;
        wp::float32 var_33;
        wp::float32 var_34;
        wp::float32 var_35;
        //---------
        // forward
        // def _tendon_bias_coef(                                                                 <L 2383>
        // worldid, tenid, dofid_sparse = wp.tid()                                                <L 2396>
        builtin_tid3d(var_0, var_1, var_2);
        // armature = tendon_armature[worldid % tendon_armature.shape[0], tenid]                  <L 2398>
        var_3 = &(var_tendon_armature.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_tendon_armature, var_7, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // if armature == 0.0:                                                                    <L 2399>
        var_12 = (var_9 == var_11);
        if (var_12) {
            // return                                                                             <L 2400>
            continue;
        }
        // rownnz = ten_J_rownnz[tenid]                                                           <L 2402>
        var_13 = wp::address(var_ten_J_rownnz, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // if dofid_sparse >= rownnz:                                                             <L 2403>
        var_16 = (var_2 >= var_14);
        if (var_16) {
            // return                                                                             <L 2404>
            continue;
        }
        // rowadr = ten_J_rowadr[tenid]                                                           <L 2405>
        var_17 = wp::address(var_ten_J_rowadr, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // sparseid = rowadr + dofid_sparse                                                       <L 2406>
        var_20 = wp::add(var_18, var_2);
        // ten_Jdot = ten_Jdot_in[worldid, sparseid]                                              <L 2407>
        var_21 = wp::address(var_ten_Jdot_in, var_0, var_20);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // if ten_Jdot == 0.0:                                                                    <L 2408>
        var_25 = (var_22 == var_24);
        if (var_25) {
            // return                                                                             <L 2409>
            continue;
        }
        // dofid = ten_J_colind[sparseid]                                                         <L 2411>
        var_26 = wp::address(var_ten_J_colind, var_20);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // wp.atomic_add(ten_bias_coef_out[worldid], tenid, ten_Jdot * qvel_in[worldid, dofid])       <L 2412>
        var_29 = wp::slice_t(var_0, var_0, var_30);
        var_31 = wp::view(var_ten_bias_coef_out, var_29);
        var_32 = wp::address(var_qvel_in, var_0, var_27);
        var_34 = wp::load(var_32);
        var_33 = wp::mul(var_22, var_34);
        var_35 = wp::atomic_add(var_31, var_1, var_33);
    }
}



extern "C" __global__ void _tendon_armature_0d777ffa_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::int32> var_dof_parentid,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_armature,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_ten_J_in,
    wp::array_t<wp::float32> var_M_out)
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
        wp::int32 var_1;
        wp::int32 var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::float32 var_9;
        wp::float32 var_10;
        const wp::float32 var_11 = 0.0;
        bool var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        bool var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::int32 var_21;
        wp::int32* var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::float32* var_25;
        wp::float32 var_26;
        wp::float32 var_27;
        const wp::float32 var_28 = 0.0;
        bool var_29;
        wp::int32* var_30;
        wp::int32* var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        const wp::int32 var_35 = 1;
        wp::int32 var_36;
        wp::int32 var_37;
        wp::int32 var_38;
        const wp::int32 var_39 = 0;
        bool var_40;
        bool var_41;
        wp::float32 var_42;
        const wp::int32 var_43 = 0;
        bool var_44;
        wp::int32 var_45;
        wp::int32* var_46;
        bool var_47;
        wp::int32 var_48;
        wp::int32 var_49;
        const wp::int32 var_50 = 1;
        wp::int32 var_51;
        bool var_52;
        const wp::int32 var_53 = 0;
        bool var_54;
        wp::int32* var_55;
        bool var_56;
        wp::int32 var_57;
        wp::float32* var_58;
        wp::float32 var_59;
        wp::float32 var_60;
        wp::float32 var_61;
        const wp::float32 var_62 = 0.0;
        wp::float32 var_63;
        wp::float32 var_64;
        wp::float32 var_65;
        wp::float32 var_66;
        wp::float32 var_67;
        wp::slice_t var_68;
        const wp::int32 var_69 = 0;
        wp::array_t<wp::float32> var_70;
        wp::float32 var_71;
        const wp::int32 var_72 = 1;
        wp::int32 var_73;
        wp::int32* var_74;
        wp::int32 var_75;
        wp::int32 var_76;
        //---------
        // forward
        // def _tendon_armature(                                                                  <L 1072>
        // worldid, tenid, dofid = wp.tid()                                                       <L 1086>
        builtin_tid3d(var_0, var_1, var_2);
        // armature = tendon_armature[worldid % tendon_armature.shape[0], tenid]                  <L 1088>
        var_3 = &(var_tendon_armature.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_tendon_armature, var_7, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // if armature == 0.0:                                                                    <L 1090>
        var_12 = (var_9 == var_11);
        if (var_12) {
            // return                                                                             <L 1091>
            continue;
        }
        // rownnz = ten_J_rownnz[tenid]                                                           <L 1093>
        var_13 = wp::address(var_ten_J_rownnz, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // if dofid >= rownnz:                                                                    <L 1094>
        var_16 = (var_2 >= var_14);
        if (var_16) {
            // return                                                                             <L 1095>
            continue;
        }
        // rowadr = ten_J_rowadr[tenid]                                                           <L 1096>
        var_17 = wp::address(var_ten_J_rowadr, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // dofid_sparse = dofid                                                                   <L 1097>
        var_20 = wp::copy(var_2);
        // sparseid = rowadr + dofid_sparse                                                       <L 1098>
        var_21 = wp::add(var_18, var_20);
        // dofid = ten_J_colind[sparseid]                                                         <L 1099>
        var_22 = wp::address(var_ten_J_colind, var_21);
        var_24 = wp::load(var_22);
        var_23 = wp::copy(var_24);
        // ten_Ji = ten_J_in[worldid, sparseid]                                                   <L 1100>
        var_25 = wp::address(var_ten_J_in, var_0, var_21);
        var_27 = wp::load(var_25);
        var_26 = wp::copy(var_27);
        // if ten_Ji == 0.0:                                                                      <L 1102>
        var_29 = (var_26 == var_28);
        if (var_29) {
            // return                                                                             <L 1103>
            continue;
        }
        // madr_ij = M_rowadr[dofid] + M_rownnz[dofid] - 1                                        <L 1106>
        var_30 = wp::address(var_M_rowadr, var_23);
        var_31 = wp::address(var_M_rownnz, var_23);
        var_33 = wp::load(var_30);
        var_34 = wp::load(var_31);
        var_32 = wp::add(var_33, var_34);
        var_36 = wp::sub(var_32, var_35);
        // dofidi = dofid                                                                         <L 1109>
        var_37 = wp::copy(var_23);
        // ptr = dofid_sparse                                                                     <L 1110>
        var_38 = wp::copy(var_20);
        // while dofid >= 0:                                                                      <L 1111>
        start_while_3:;
        var_40 = (var_23 >= var_39);
        if ((var_40) == false) goto end_while_3;
            // if dofid == dofidi:                                                                <L 1112>
            var_41 = (var_23 == var_37);
            if (var_41) {
                // ten_Jj = ten_Ji                                                                <L 1113>
                var_42 = wp::copy(var_26);
            }
            if (!var_41) {
                // while ptr >= 0:                                                                <L 1116>
        start_while_5:;
                var_44 = (var_38 >= var_43);
        if ((var_44) == false) goto end_while_5;
                    // sparseid = rowadr + ptr                                                    <L 1117>
                    var_45 = wp::add(var_18, var_38);
                    // if ten_J_colind[sparseid] <= dofid:                                        <L 1118>
                    var_46 = wp::address(var_ten_J_colind, var_45);
                    var_48 = wp::load(var_46);
                    var_47 = (var_48 <= var_23);
                    if (var_47) {
                        // break                                                                  <L 1119>
                        wp::assign(var_21, var_45);
                        goto end_while_5;
                    }
                    var_49 = wp::where(var_47, var_21, var_45);
                    // ptr -= 1                                                                   <L 1120>
                    var_51 = wp::sub(var_38, var_50);
                    wp::assign(var_21, var_49);
                    wp::assign(var_38, var_51);
        goto start_while_5;
        end_while_5:;
                // if ptr >= 0 and ten_J_colind[sparseid] == dofid:                               <L 1121>
                var_54 = (var_38 >= var_53);
                var_52 = var_54;
                if (var_52) {
                    var_55 = wp::address(var_ten_J_colind, var_21);
                    var_57 = wp::load(var_55);
                    var_56 = (var_57 == var_23);
                    var_52 = var_52 && var_56;
                }
                if (var_52) {
                    // ten_Jj = ten_J_in[worldid, sparseid]                                       <L 1122>
                    var_58 = wp::address(var_ten_J_in, var_0, var_21);
                    var_60 = wp::load(var_58);
                    var_59 = wp::copy(var_60);
                }
                var_61 = wp::where(var_52, var_59, var_42);
                if (!var_52) {
                    // ten_Jj = float(0.0)                                                        <L 1124>
                    var_63 = wp::float(var_62);
                }
                var_64 = wp::where(var_52, var_61, var_63);
            }
            var_65 = wp::where(var_41, var_42, var_64);
            // Mij = armature * ten_Jj * ten_Ji                                                   <L 1126>
            var_66 = wp::mul(var_9, var_65);
            var_67 = wp::mul(var_66, var_26);
            // wp.atomic_add(M_out[worldid], madr_ij, Mij)                                        <L 1128>
            var_68 = wp::slice_t(var_0, var_0, var_69);
            var_70 = wp::view(var_M_out, var_68);
            var_71 = wp::atomic_add(var_70, var_36, var_67);
            // madr_ij -= 1                                                                       <L 1129>
            var_73 = wp::sub(var_36, var_72);
            // dofid = dof_parentid[dofid]                                                        <L 1131>
            var_74 = wp::address(var_dof_parentid, var_23);
            var_76 = wp::load(var_74);
            var_75 = wp::copy(var_76);
            wp::assign(var_23, var_75);
            wp::assign(var_36, var_73);
        goto start_while_3;
        end_while_3:;
    }
}



extern "C" __global__ void _cfrc_ext_f67cbe30_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_xfrc_applied_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_ext_out)
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
        wp::int32 var_1;
        const wp::int32 var_2 = 0;
        bool var_3;
        const wp::float32 var_4 = 0.0;
        const wp::float32 var_5 = 0.0;
        const wp::float32 var_6 = 0.0;
        const wp::float32 var_7 = 0.0;
        const wp::float32 var_8 = 0.0;
        const wp::float32 var_9 = 0.0;
        wp::vec_t<6, wp::float32> var_10;
        const wp::int32 var_11 = 0;
        wp::vec_t<6, wp::float32>* var_12;
        wp::vec_t<6, wp::float32> var_13;
        wp::vec_t<6, wp::float32> var_14;
        wp::int32* var_15;
        wp::vec_t<3, wp::float32>* var_16;
        wp::int32 var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::vec_t<3, wp::float32>* var_20;
        wp::vec_t<3, wp::float32> var_21;
        wp::vec_t<3, wp::float32> var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::vec_t<6, wp::float32> var_24;
        //---------
        // forward
        // def _cfrc_ext(                                                                         <L 1489>
        // worldid, bodyid = wp.tid()                                                             <L 1499>
        builtin_tid2d(var_0, var_1);
        // if bodyid == 0:                                                                        <L 1500>
        var_3 = (var_1 == var_2);
        if (var_3) {
            // cfrc_ext_out[worldid, 0] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)         <L 1501>
            var_10 = wp::vec_t<6, wp::float32>({var_4, var_5, var_6, var_7, var_8, var_9});
            wp::array_store(var_cfrc_ext_out, var_0, var_11, var_10);
        }
        if (!var_3) {
            // xfrc_applied = xfrc_applied_in[worldid, bodyid]                                    <L 1503>
            var_12 = wp::address(var_xfrc_applied_in, var_0, var_1);
            var_14 = wp::load(var_12);
            var_13 = wp::copy(var_14);
            // subtree_com = subtree_com_in[worldid, body_rootid[bodyid]]                         <L 1504>
            var_15 = wp::address(var_body_rootid, var_1);
            var_17 = wp::load(var_15);
            var_16 = wp::address(var_subtree_com_in, var_0, var_17);
            var_19 = wp::load(var_16);
            var_18 = wp::copy(var_19);
            // xipos = xipos_in[worldid, bodyid]                                                  <L 1505>
            var_20 = wp::address(var_xipos_in, var_0, var_1);
            var_22 = wp::load(var_20);
            var_21 = wp::copy(var_22);
            // cfrc_ext_out[worldid, bodyid] = support.transform_force(xfrc_applied, subtree_com - xipos)       <L 1506>
            var_23 = wp::sub(var_18, var_21);
            var_24 = transform_force_1(var_13, var_23);
            wp::array_store(var_cfrc_ext_out, var_0, var_1, var_24);
        }
    }
}



extern "C" __global__ void _cfrc_ext_equality_460e8932_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_pos,
    wp::array_t<wp::quat_t<wp::float32>> var_site_quat,
    wp::array_t<wp::int32> var_eq_obj1id,
    wp::array_t<wp::int32> var_eq_obj2id,
    wp::array_t<wp::int32> var_eq_objtype,
    wp::array_t<wp::vec_t<11, wp::float32>> var_eq_data,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::int32> var_efc_id_in,
    wp::array_t<wp::float32> var_efc_force_in,
    wp::array_t<wp::int32> var_ne_connect_in,
    wp::array_t<wp::int32> var_ne_weld_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_ext_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::int32 var_8 = 3;
        wp::int32 var_9;
        const wp::int32 var_10 = 6;
        wp::int32 var_11;
        wp::int32 var_12;
        bool var_13;
        bool var_14;
        const wp::int32 var_15 = 3;
        wp::int32 var_16;
        const wp::int32 var_17 = 6;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        const wp::int32 var_21 = 0;
        wp::int32 var_22;
        wp::float32* var_23;
        const wp::int32 var_24 = 1;
        wp::int32 var_25;
        wp::float32* var_26;
        const wp::int32 var_27 = 2;
        wp::int32 var_28;
        wp::float32* var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::float32 var_31;
        wp::float32 var_32;
        wp::float32 var_33;
        wp::int32* var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        wp::shape_t* var_37;
        const wp::int32 var_38 = 0;
        wp::int32 var_39;
        wp::shape_t var_40;
        wp::int32 var_41;
        wp::vec_t<11, wp::float32>* var_42;
        wp::vec_t<11, wp::float32> var_43;
        wp::vec_t<11, wp::float32> var_44;
        wp::int32* var_45;
        const wp::int32 var_46 = 1;
        bool var_47;
        wp::int32 var_48;
        wp::int32* var_49;
        wp::int32 var_50;
        wp::int32 var_51;
        wp::int32* var_52;
        wp::int32 var_53;
        wp::int32 var_54;
        wp::int32 var_55;
        wp::int32 var_56;
        wp::int32* var_57;
        wp::int32 var_58;
        wp::int32 var_59;
        wp::int32* var_60;
        wp::int32 var_61;
        wp::int32 var_62;
        wp::int32 var_63;
        wp::int32 var_64;
        const wp::float32 var_65 = 0.0;
        const wp::float32 var_66 = 0.0;
        const wp::float32 var_67 = 0.0;
        wp::vec_t<3, wp::float32> var_68;
        const wp::int32 var_69 = 3;
        wp::int32 var_70;
        wp::float32* var_71;
        const wp::int32 var_72 = 4;
        wp::int32 var_73;
        wp::float32* var_74;
        const wp::int32 var_75 = 5;
        wp::int32 var_76;
        wp::float32* var_77;
        wp::vec_t<3, wp::float32> var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::float32 var_81;
        const wp::int32 var_82 = 6;
        wp::float32 var_83;
        const wp::int32 var_84 = 7;
        wp::float32 var_85;
        const wp::int32 var_86 = 8;
        wp::float32 var_87;
        const wp::int32 var_88 = 9;
        wp::float32 var_89;
        wp::quat_t<wp::float32> var_90;
        wp::quat_t<wp::float32>* var_91;
        wp::quat_t<wp::float32> var_92;
        wp::quat_t<wp::float32> var_93;
        wp::quat_t<wp::float32>* var_94;
        wp::quat_t<wp::float32> var_95;
        wp::quat_t<wp::float32> var_96;
        wp::shape_t* var_97;
        const wp::int32 var_98 = 0;
        wp::int32 var_99;
        wp::shape_t var_100;
        wp::int32 var_101;
        wp::quat_t<wp::float32>* var_102;
        wp::quat_t<wp::float32>* var_103;
        wp::quat_t<wp::float32> var_104;
        wp::quat_t<wp::float32> var_105;
        wp::quat_t<wp::float32> var_106;
        wp::quat_t<wp::float32>* var_107;
        wp::quat_t<wp::float32>* var_108;
        wp::quat_t<wp::float32> var_109;
        wp::quat_t<wp::float32> var_110;
        wp::quat_t<wp::float32> var_111;
        wp::quat_t<wp::float32> var_112;
        wp::quat_t<wp::float32> var_113;
        wp::quat_t<wp::float32> var_114;
        wp::quat_t<wp::float32> var_115;
        wp::quat_t<wp::float32> var_116;
        const wp::float32 var_117 = 0.5;
        const wp::int32 var_118 = 10;
        wp::float32 var_119;
        wp::float32 var_120;
        const wp::int32 var_121 = 1;
        wp::float32 var_122;
        const wp::int32 var_123 = 2;
        wp::float32 var_124;
        const wp::int32 var_125 = 3;
        wp::float32 var_126;
        wp::vec_t<3, wp::float32> var_127;
        wp::vec_t<3, wp::float32> var_128;
        wp::vec_t<3, wp::float32> var_129;
        const wp::int32 var_130 = 0;
        wp::float32 var_131;
        const wp::int32 var_132 = 1;
        wp::float32 var_133;
        const wp::int32 var_134 = 2;
        wp::float32 var_135;
        wp::vec_t<3, wp::float32> var_136;
        const wp::int32 var_137 = 3;
        wp::float32 var_138;
        const wp::int32 var_139 = 4;
        wp::float32 var_140;
        const wp::int32 var_141 = 5;
        wp::float32 var_142;
        wp::vec_t<3, wp::float32> var_143;
        wp::vec_t<3, wp::float32> var_144;
        wp::shape_t* var_145;
        const wp::int32 var_146 = 0;
        wp::int32 var_147;
        wp::shape_t var_148;
        wp::int32 var_149;
        wp::vec_t<3, wp::float32>* var_150;
        wp::vec_t<3, wp::float32> var_151;
        wp::vec_t<3, wp::float32> var_152;
        wp::vec_t<3, wp::float32> var_153;
        wp::mat_t<3, 3, wp::float32>* var_154;
        wp::vec_t<3, wp::float32> var_155;
        wp::mat_t<3, 3, wp::float32> var_156;
        wp::vec_t<3, wp::float32>* var_157;
        wp::vec_t<3, wp::float32> var_158;
        wp::vec_t<3, wp::float32> var_159;
        wp::int32* var_160;
        wp::vec_t<3, wp::float32>* var_161;
        wp::int32 var_162;
        wp::vec_t<3, wp::float32> var_163;
        wp::vec_t<3, wp::float32> var_164;
        wp::vec_t<3, wp::float32> var_165;
        wp::vec_t<3, wp::float32> var_166;
        wp::vec_t<3, wp::float32> var_167;
        wp::vec_t<6, wp::float32> var_168;
        wp::slice_t var_169;
        const wp::int32 var_170 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_171;
        wp::vec_t<6, wp::float32> var_172;
        const wp::int32 var_173 = 3;
        wp::float32 var_174;
        const wp::int32 var_175 = 4;
        wp::float32 var_176;
        const wp::int32 var_177 = 5;
        wp::float32 var_178;
        wp::vec_t<3, wp::float32> var_179;
        wp::vec_t<3, wp::float32> var_180;
        const wp::int32 var_181 = 0;
        wp::float32 var_182;
        const wp::int32 var_183 = 1;
        wp::float32 var_184;
        const wp::int32 var_185 = 2;
        wp::float32 var_186;
        wp::vec_t<3, wp::float32> var_187;
        wp::vec_t<3, wp::float32> var_188;
        wp::vec_t<3, wp::float32> var_189;
        wp::shape_t* var_190;
        const wp::int32 var_191 = 0;
        wp::int32 var_192;
        wp::shape_t var_193;
        wp::int32 var_194;
        wp::vec_t<3, wp::float32>* var_195;
        wp::vec_t<3, wp::float32> var_196;
        wp::vec_t<3, wp::float32> var_197;
        wp::vec_t<3, wp::float32> var_198;
        wp::mat_t<3, 3, wp::float32>* var_199;
        wp::vec_t<3, wp::float32> var_200;
        wp::mat_t<3, 3, wp::float32> var_201;
        wp::vec_t<3, wp::float32>* var_202;
        wp::vec_t<3, wp::float32> var_203;
        wp::vec_t<3, wp::float32> var_204;
        wp::int32* var_205;
        wp::vec_t<3, wp::float32>* var_206;
        wp::int32 var_207;
        wp::vec_t<3, wp::float32> var_208;
        wp::vec_t<3, wp::float32> var_209;
        wp::vec_t<3, wp::float32> var_210;
        wp::vec_t<3, wp::float32> var_211;
        wp::vec_t<3, wp::float32> var_212;
        wp::vec_t<6, wp::float32> var_213;
        wp::slice_t var_214;
        const wp::int32 var_215 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_216;
        wp::vec_t<6, wp::float32> var_217;
        wp::vec_t<3, wp::float32> var_218;
        wp::vec_t<3, wp::float32> var_219;
        wp::vec_t<3, wp::float32> var_220;
        wp::vec_t<3, wp::float32> var_221;
        wp::vec_t<6, wp::float32> var_222;
        //---------
        // forward
        // def _cfrc_ext_equality(                                                                <L 1540>
        // worldid, eqid = wp.tid()                                                               <L 1563>
        builtin_tid2d(var_0, var_1);
        // ne_connect = ne_connect_in[worldid]                                                    <L 1565>
        var_2 = wp::address(var_ne_connect_in, var_0);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // ne_weld = ne_weld_in[worldid]                                                          <L 1566>
        var_5 = wp::address(var_ne_weld_in, var_0);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // num_connect = ne_connect // 3                                                          <L 1567>
        var_9 = wp::floordiv(var_3, var_8);
        // if eqid >= num_connect + ne_weld // 6:                                                 <L 1569>
        var_11 = wp::floordiv(var_6, var_10);
        var_12 = wp::add(var_9, var_11);
        var_13 = (var_1 >= var_12);
        if (var_13) {
            // return                                                                             <L 1570>
            continue;
        }
        // is_connect = eqid < num_connect                                                        <L 1572>
        var_14 = (var_1 < var_9);
        // if is_connect:                                                                         <L 1573>
        if (var_14) {
            // efcid = 3 * eqid                                                                   <L 1574>
            var_16 = wp::mul(var_15, var_1);
        }
        if (!var_14) {
            // efcid = 6 * eqid - ne_connect                                                      <L 1576>
            var_18 = wp::mul(var_17, var_1);
            var_19 = wp::sub(var_18, var_3);
        }
        var_20 = wp::where(var_14, var_16, var_19);
        // cfrc_force = wp.vec3(                                                                  <L 1578>
        // efc_force_in[worldid, efcid + 0],                                                      <L 1579>
        var_22 = wp::add(var_20, var_21);
        var_23 = wp::address(var_efc_force_in, var_0, var_22);
        // efc_force_in[worldid, efcid + 1],                                                      <L 1580>
        var_25 = wp::add(var_20, var_24);
        var_26 = wp::address(var_efc_force_in, var_0, var_25);
        // efc_force_in[worldid, efcid + 2],                                                      <L 1581>
        var_28 = wp::add(var_20, var_27);
        var_29 = wp::address(var_efc_force_in, var_0, var_28);
        var_31 = wp::load(var_23);
        var_32 = wp::load(var_26);
        var_33 = wp::load(var_29);
        var_30 = wp::vec_t<3, wp::float32>(var_31, var_32, var_33);
        // id = efc_id_in[worldid, efcid]                                                         <L 1584>
        var_34 = wp::address(var_efc_id_in, var_0, var_20);
        var_36 = wp::load(var_34);
        var_35 = wp::copy(var_36);
        // eq_data_ = eq_data[worldid % eq_data.shape[0], id]                                     <L 1585>
        var_37 = &(var_eq_data.shape);
        var_40 = wp::load(var_37);
        var_39 = wp::extract(var_40, var_38);
        var_41 = wp::mod(var_0, var_39);
        var_42 = wp::address(var_eq_data, var_41, var_35);
        var_44 = wp::load(var_42);
        var_43 = wp::copy(var_44);
        // body_semantic = eq_objtype[id] == ObjType.BODY                                         <L 1586>
        var_45 = wp::address(var_eq_objtype, var_35);
        var_48 = wp::load(var_45);
        var_47 = (var_48 == var_46);
        // obj1 = eq_obj1id[id]                                                                   <L 1588>
        var_49 = wp::address(var_eq_obj1id, var_35);
        var_51 = wp::load(var_49);
        var_50 = wp::copy(var_51);
        // obj2 = eq_obj2id[id]                                                                   <L 1589>
        var_52 = wp::address(var_eq_obj2id, var_35);
        var_54 = wp::load(var_52);
        var_53 = wp::copy(var_54);
        // if body_semantic:                                                                      <L 1591>
        if (var_47) {
            // bodyid1 = obj1                                                                     <L 1592>
            var_55 = wp::copy(var_50);
            // bodyid2 = obj2                                                                     <L 1593>
            var_56 = wp::copy(var_53);
        }
        if (!var_47) {
            // bodyid1 = site_bodyid[obj1]                                                        <L 1595>
            var_57 = wp::address(var_site_bodyid, var_50);
            var_59 = wp::load(var_57);
            var_58 = wp::copy(var_59);
            // bodyid2 = site_bodyid[obj2]                                                        <L 1596>
            var_60 = wp::address(var_site_bodyid, var_53);
            var_62 = wp::load(var_60);
            var_61 = wp::copy(var_62);
        }
        var_63 = wp::where(var_47, var_55, var_58);
        var_64 = wp::where(var_47, var_56, var_61);
        // if is_connect:                                                                         <L 1598>
        if (var_14) {
            // cfrc_torque = wp.vec3(0.0, 0.0, 0.0)  # no torque from connect                     <L 1599>
            var_68 = wp::vec_t<3, wp::float32>(var_65, var_66, var_67);
        }
        if (!var_14) {
            // force_torque = wp.vec3(                                                            <L 1601>
            // efc_force_in[worldid, efcid + 3],                                                  <L 1602>
            var_70 = wp::add(var_20, var_69);
            var_71 = wp::address(var_efc_force_in, var_0, var_70);
            // efc_force_in[worldid, efcid + 4],                                                  <L 1603>
            var_73 = wp::add(var_20, var_72);
            var_74 = wp::address(var_efc_force_in, var_0, var_73);
            // efc_force_in[worldid, efcid + 5],                                                  <L 1604>
            var_76 = wp::add(var_20, var_75);
            var_77 = wp::address(var_efc_force_in, var_0, var_76);
            var_79 = wp::load(var_71);
            var_80 = wp::load(var_74);
            var_81 = wp::load(var_77);
            var_78 = wp::vec_t<3, wp::float32>(var_79, var_80, var_81);
            // if body_semantic:                                                                  <L 1606>
            if (var_47) {
                // relpose = wp.quat(eq_data_[6], eq_data_[7], eq_data_[8], eq_data_[9])          <L 1607>
                var_83 = wp::extract(var_43, var_82);
                var_85 = wp::extract(var_43, var_84);
                var_87 = wp::extract(var_43, var_86);
                var_89 = wp::extract(var_43, var_88);
                var_90 = wp::quat_t<wp::float32>(var_83, var_85, var_87, var_89);
                // q0 = math.mul_quat(xquat_in[worldid, bodyid1], relpose)                        <L 1608>
                var_91 = wp::address(var_xquat_in, var_0, var_63);
                var_93 = wp::load(var_91);
                var_92 = mul_quat_0(var_93, var_90);
                // q1 = xquat_in[worldid, bodyid2]                                                <L 1609>
                var_94 = wp::address(var_xquat_in, var_0, var_64);
                var_96 = wp::load(var_94);
                var_95 = wp::copy(var_96);
            }
            if (!var_47) {
                // site_quat_id = worldid % site_quat.shape[0]                                    <L 1611>
                var_97 = &(var_site_quat.shape);
                var_100 = wp::load(var_97);
                var_99 = wp::extract(var_100, var_98);
                var_101 = wp::mod(var_0, var_99);
                // q0 = math.mul_quat(xquat_in[worldid, bodyid1], site_quat[site_quat_id, obj1])       <L 1612>
                var_102 = wp::address(var_xquat_in, var_0, var_63);
                var_103 = wp::address(var_site_quat, var_101, var_50);
                var_105 = wp::load(var_102);
                var_106 = wp::load(var_103);
                var_104 = mul_quat_0(var_105, var_106);
                // q1 = math.mul_quat(xquat_in[worldid, bodyid2], site_quat[site_quat_id, obj2])       <L 1613>
                var_107 = wp::address(var_xquat_in, var_0, var_64);
                var_108 = wp::address(var_site_quat, var_101, var_53);
                var_110 = wp::load(var_107);
                var_111 = wp::load(var_108);
                var_109 = mul_quat_0(var_110, var_111);
            }
            var_112 = wp::where(var_47, var_92, var_104);
            var_113 = wp::where(var_47, var_95, var_109);
            // quat = math.mul_quat(math.quat_mul_axis(q1, force_torque), math.quat_inv(q0))       <L 1615>
            var_114 = quat_mul_axis_0(var_113, var_78);
            var_115 = quat_inv_0(var_112);
            var_116 = mul_quat_0(var_114, var_115);
            // cfrc_torque = (0.5 * eq_data_[10]) * wp.vec3(quat[1], quat[2], quat[3])            <L 1616>
            var_119 = wp::extract(var_43, var_118);
            var_120 = wp::mul(var_117, var_119);
            var_122 = wp::extract(var_116, var_121);
            var_124 = wp::extract(var_116, var_123);
            var_126 = wp::extract(var_116, var_125);
            var_127 = wp::vec_t<3, wp::float32>(var_122, var_124, var_126);
            var_128 = wp::mul(var_120, var_127);
        }
        var_129 = wp::where(var_14, var_68, var_128);
        // if bodyid1:                                                                            <L 1619>
        if (var_63) {
            // if body_semantic:                                                                  <L 1620>
            if (var_47) {
                // if is_connect:                                                                 <L 1621>
                if (var_14) {
                    // offset = wp.vec3(eq_data_[0], eq_data_[1], eq_data_[2])                    <L 1622>
                    var_131 = wp::extract(var_43, var_130);
                    var_133 = wp::extract(var_43, var_132);
                    var_135 = wp::extract(var_43, var_134);
                    var_136 = wp::vec_t<3, wp::float32>(var_131, var_133, var_135);
                }
                if (!var_14) {
                    // offset = wp.vec3(eq_data_[3], eq_data_[4], eq_data_[5])                    <L 1624>
                    var_138 = wp::extract(var_43, var_137);
                    var_140 = wp::extract(var_43, var_139);
                    var_142 = wp::extract(var_43, var_141);
                    var_143 = wp::vec_t<3, wp::float32>(var_138, var_140, var_142);
                }
                var_144 = wp::where(var_14, var_136, var_143);
            }
            if (!var_47) {
                // offset = site_pos[worldid % site_pos.shape[0], obj1]                           <L 1626>
                var_145 = &(var_site_pos.shape);
                var_148 = wp::load(var_145);
                var_147 = wp::extract(var_148, var_146);
                var_149 = wp::mod(var_0, var_147);
                var_150 = wp::address(var_site_pos, var_149, var_50);
                var_152 = wp::load(var_150);
                var_151 = wp::copy(var_152);
            }
            var_153 = wp::where(var_47, var_144, var_151);
            // pos = xmat_in[worldid, bodyid1] @ offset + xpos_in[worldid, bodyid1]               <L 1629>
            var_154 = wp::address(var_xmat_in, var_0, var_63);
            var_156 = wp::load(var_154);
            var_155 = wp::mul(var_156, var_153);
            var_157 = wp::address(var_xpos_in, var_0, var_63);
            var_159 = wp::load(var_157);
            var_158 = wp::add(var_155, var_159);
            // newpos = subtree_com_in[worldid, body_rootid[bodyid1]]                             <L 1632>
            var_160 = wp::address(var_body_rootid, var_63);
            var_162 = wp::load(var_160);
            var_161 = wp::address(var_subtree_com_in, var_0, var_162);
            var_164 = wp::load(var_161);
            var_163 = wp::copy(var_164);
            // dif = newpos - pos                                                                 <L 1634>
            var_165 = wp::sub(var_163, var_158);
            // cfrc_com = wp.spatial_vector(cfrc_torque - wp.cross(dif, cfrc_force), cfrc_force)       <L 1635>
            var_166 = wp::cross(var_165, var_30);
            var_167 = wp::sub(var_129, var_166);
            var_168 = wp::vec_t<6, wp::float32>(var_167, var_30);
            // wp.atomic_add(cfrc_ext_out[worldid], bodyid1, cfrc_com)                            <L 1638>
            var_169 = wp::slice_t(var_0, var_0, var_170);
            var_171 = wp::view(var_cfrc_ext_out, var_169);
            var_172 = wp::atomic_add(var_171, var_63, var_168);
        }
        // if bodyid2:                                                                            <L 1641>
        if (var_64) {
            // if body_semantic:                                                                  <L 1642>
            if (var_47) {
                // if is_connect:                                                                 <L 1643>
                if (var_14) {
                    // offset = wp.vec3(eq_data_[3], eq_data_[4], eq_data_[5])                    <L 1644>
                    var_174 = wp::extract(var_43, var_173);
                    var_176 = wp::extract(var_43, var_175);
                    var_178 = wp::extract(var_43, var_177);
                    var_179 = wp::vec_t<3, wp::float32>(var_174, var_176, var_178);
                }
                var_180 = wp::where(var_14, var_179, var_153);
                if (!var_14) {
                    // offset = wp.vec3(eq_data_[0], eq_data_[1], eq_data_[2])                    <L 1646>
                    var_182 = wp::extract(var_43, var_181);
                    var_184 = wp::extract(var_43, var_183);
                    var_186 = wp::extract(var_43, var_185);
                    var_187 = wp::vec_t<3, wp::float32>(var_182, var_184, var_186);
                }
                var_188 = wp::where(var_14, var_180, var_187);
            }
            var_189 = wp::where(var_47, var_188, var_153);
            if (!var_47) {
                // offset = site_pos[worldid % site_pos.shape[0], obj2]                           <L 1648>
                var_190 = &(var_site_pos.shape);
                var_193 = wp::load(var_190);
                var_192 = wp::extract(var_193, var_191);
                var_194 = wp::mod(var_0, var_192);
                var_195 = wp::address(var_site_pos, var_194, var_53);
                var_197 = wp::load(var_195);
                var_196 = wp::copy(var_197);
            }
            var_198 = wp::where(var_47, var_189, var_196);
            // pos = xmat_in[worldid, bodyid2] @ offset + xpos_in[worldid, bodyid2]               <L 1651>
            var_199 = wp::address(var_xmat_in, var_0, var_64);
            var_201 = wp::load(var_199);
            var_200 = wp::mul(var_201, var_198);
            var_202 = wp::address(var_xpos_in, var_0, var_64);
            var_204 = wp::load(var_202);
            var_203 = wp::add(var_200, var_204);
            // newpos = subtree_com_in[worldid, body_rootid[bodyid2]]                             <L 1654>
            var_205 = wp::address(var_body_rootid, var_64);
            var_207 = wp::load(var_205);
            var_206 = wp::address(var_subtree_com_in, var_0, var_207);
            var_209 = wp::load(var_206);
            var_208 = wp::copy(var_209);
            // dif = newpos - pos                                                                 <L 1656>
            var_210 = wp::sub(var_208, var_203);
            // cfrc_com = wp.spatial_vector(cfrc_torque - wp.cross(dif, cfrc_force), cfrc_force)       <L 1657>
            var_211 = wp::cross(var_210, var_30);
            var_212 = wp::sub(var_129, var_211);
            var_213 = wp::vec_t<6, wp::float32>(var_212, var_30);
            // wp.atomic_sub(cfrc_ext_out[worldid], bodyid2, cfrc_com)                            <L 1660>
            var_214 = wp::slice_t(var_0, var_0, var_215);
            var_216 = wp::view(var_cfrc_ext_out, var_214);
            var_217 = wp::atomic_sub(var_216, var_64, var_213);
        }
        var_218 = wp::where(var_64, var_198, var_153);
        var_219 = wp::where(var_64, var_203, var_158);
        var_220 = wp::where(var_64, var_208, var_163);
        var_221 = wp::where(var_64, var_210, var_165);
        var_222 = wp::where(var_64, var_213, var_168);
    }
}



extern "C" __global__ void _count_equality_constraints_54fc2a85_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_eq_type,
    wp::array_t<wp::int32> var_ne_in,
    wp::array_t<wp::int32> var_efc_type_in,
    wp::array_t<wp::int32> var_efc_id_in,
    wp::array_t<wp::int32> var_ne_connect_out,
    wp::array_t<wp::int32> var_ne_weld_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        bool var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::int32* var_8;
        wp::int32 var_9;
        wp::int32 var_10;
        const wp::int32 var_11 = 0;
        bool var_12;
        const wp::int32 var_13 = 1;
        wp::int32 var_14;
        const wp::int32 var_15 = 1;
        bool var_16;
        const wp::int32 var_17 = 1;
        wp::int32 var_18;
        //---------
        // forward
        // def _count_equality_constraints(                                                       <L 1510>
        // worldid, efcid = wp.tid()                                                              <L 1522>
        builtin_tid2d(var_0, var_1);
        // if efcid >= ne_in[worldid]:                                                            <L 1525>
        var_2 = wp::address(var_ne_in, var_0);
        var_4 = wp::load(var_2);
        var_3 = (var_1 >= var_4);
        if (var_3) {
            // return                                                                             <L 1526>
            continue;
        }
        // eq_id = efc_id_in[worldid, efcid]                                                      <L 1529>
        var_5 = wp::address(var_efc_id_in, var_0, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // eq_constraint_type = eq_type[eq_id]                                                    <L 1530>
        var_8 = wp::address(var_eq_type, var_6);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // if eq_constraint_type == EqType.CONNECT:                                               <L 1533>
        var_12 = (var_9 == var_11);
        if (var_12) {
            // wp.atomic_add(ne_connect_out, worldid, 1)                                          <L 1534>
            var_14 = wp::atomic_add(var_ne_connect_out, var_0, var_13);
        }
        if (!var_12) {
            // elif eq_constraint_type == EqType.WELD:                                            <L 1535>
            var_16 = (var_9 == var_15);
            if (var_16) {
                // wp.atomic_add(ne_weld_out, worldid, 1)                                         <L 1536>
                var_18 = wp::atomic_add(var_ne_weld_out, var_0, var_17);
            }
        }
    }
}



extern "C" __global__ void _cinert_1261260c_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_body_inertia,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_ximat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<10, wp::float32>> var_cinert_out)
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
        wp::int32 var_1;
        wp::mat_t<3, 3, wp::float32>* var_2;
        wp::mat_t<3, 3, wp::float32> var_3;
        wp::mat_t<3, 3, wp::float32> var_4;
        wp::shape_t* var_5;
        const wp::int32 var_6 = 0;
        wp::int32 var_7;
        wp::shape_t var_8;
        wp::int32 var_9;
        wp::vec_t<3, wp::float32>* var_10;
        wp::vec_t<3, wp::float32> var_11;
        wp::vec_t<3, wp::float32> var_12;
        wp::shape_t* var_13;
        const wp::int32 var_14 = 0;
        wp::int32 var_15;
        wp::shape_t var_16;
        wp::int32 var_17;
        wp::float32* var_18;
        wp::float32 var_19;
        wp::float32 var_20;
        wp::vec_t<3, wp::float32>* var_21;
        wp::int32* var_22;
        wp::vec_t<3, wp::float32>* var_23;
        wp::int32 var_24;
        wp::vec_t<3, wp::float32> var_25;
        wp::vec_t<3, wp::float32> var_26;
        wp::vec_t<3, wp::float32> var_27;
        wp::vec_t<10, wp::float32> var_28;
        wp::mat_t<3, 3, wp::float32> var_29;
        wp::mat_t<3, 3, wp::float32> var_30;
        wp::mat_t<3, 3, wp::float32> var_31;
        wp::mat_t<3, 3, wp::float32> var_32;
        const wp::int32 var_33 = 0;
        const wp::int32 var_34 = 0;
        wp::float32 var_35;
        const wp::int32 var_36 = 0;
        const wp::int32 var_37 = 1;
        const wp::int32 var_38 = 1;
        wp::float32 var_39;
        const wp::int32 var_40 = 1;
        const wp::int32 var_41 = 2;
        const wp::int32 var_42 = 2;
        wp::float32 var_43;
        const wp::int32 var_44 = 2;
        const wp::int32 var_45 = 0;
        const wp::int32 var_46 = 1;
        wp::float32 var_47;
        const wp::int32 var_48 = 3;
        const wp::int32 var_49 = 0;
        const wp::int32 var_50 = 2;
        wp::float32 var_51;
        const wp::int32 var_52 = 4;
        const wp::int32 var_53 = 1;
        const wp::int32 var_54 = 2;
        wp::float32 var_55;
        const wp::int32 var_56 = 5;
        const wp::int32 var_57 = 1;
        wp::float32 var_58;
        const wp::int32 var_59 = 1;
        wp::float32 var_60;
        wp::float32 var_61;
        const wp::int32 var_62 = 2;
        wp::float32 var_63;
        const wp::int32 var_64 = 2;
        wp::float32 var_65;
        wp::float32 var_66;
        wp::float32 var_67;
        wp::float32 var_68;
        const wp::int32 var_69 = 0;
        const wp::int32 var_70 = 0;
        wp::float32 var_71;
        const wp::int32 var_72 = 0;
        wp::float32 var_73;
        wp::float32 var_74;
        const wp::int32 var_75 = 2;
        wp::float32 var_76;
        const wp::int32 var_77 = 2;
        wp::float32 var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::float32 var_81;
        const wp::int32 var_82 = 1;
        const wp::int32 var_83 = 0;
        wp::float32 var_84;
        const wp::int32 var_85 = 0;
        wp::float32 var_86;
        wp::float32 var_87;
        const wp::int32 var_88 = 1;
        wp::float32 var_89;
        const wp::int32 var_90 = 1;
        wp::float32 var_91;
        wp::float32 var_92;
        wp::float32 var_93;
        wp::float32 var_94;
        const wp::int32 var_95 = 2;
        const wp::int32 var_96 = 0;
        wp::float32 var_97;
        wp::float32 var_98;
        const wp::int32 var_99 = 1;
        wp::float32 var_100;
        wp::float32 var_101;
        const wp::int32 var_102 = 3;
        const wp::int32 var_103 = 0;
        wp::float32 var_104;
        wp::float32 var_105;
        const wp::int32 var_106 = 2;
        wp::float32 var_107;
        wp::float32 var_108;
        const wp::int32 var_109 = 4;
        const wp::int32 var_110 = 1;
        wp::float32 var_111;
        wp::float32 var_112;
        const wp::int32 var_113 = 2;
        wp::float32 var_114;
        wp::float32 var_115;
        const wp::int32 var_116 = 5;
        const wp::int32 var_117 = 0;
        wp::float32 var_118;
        wp::float32 var_119;
        const wp::int32 var_120 = 6;
        const wp::int32 var_121 = 1;
        wp::float32 var_122;
        wp::float32 var_123;
        const wp::int32 var_124 = 7;
        const wp::int32 var_125 = 2;
        wp::float32 var_126;
        wp::float32 var_127;
        const wp::int32 var_128 = 8;
        const wp::int32 var_129 = 9;
        //---------
        // forward
        // def _cinert(                                                                           <L 704>
        // worldid, bodyid = wp.tid()                                                             <L 716>
        builtin_tid2d(var_0, var_1);
        // mat = ximat_in[worldid, bodyid]                                                        <L 717>
        var_2 = wp::address(var_ximat_in, var_0, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // inert = body_inertia[worldid % body_inertia.shape[0], bodyid]                          <L 718>
        var_5 = &(var_body_inertia.shape);
        var_8 = wp::load(var_5);
        var_7 = wp::extract(var_8, var_6);
        var_9 = wp::mod(var_0, var_7);
        var_10 = wp::address(var_body_inertia, var_9, var_1);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // mass = body_mass[worldid % body_mass.shape[0], bodyid]                                 <L 719>
        var_13 = &(var_body_mass.shape);
        var_16 = wp::load(var_13);
        var_15 = wp::extract(var_16, var_14);
        var_17 = wp::mod(var_0, var_15);
        var_18 = wp::address(var_body_mass, var_17, var_1);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // dif = xipos_in[worldid, bodyid] - subtree_com_in[worldid, body_rootid[bodyid]]         <L 720>
        var_21 = wp::address(var_xipos_in, var_0, var_1);
        var_22 = wp::address(var_body_rootid, var_1);
        var_24 = wp::load(var_22);
        var_23 = wp::address(var_subtree_com_in, var_0, var_24);
        var_26 = wp::load(var_21);
        var_27 = wp::load(var_23);
        var_25 = wp::sub(var_26, var_27);
        // res = vec10()                                                                          <L 723>
        var_28 = wp::vec_t<10, wp::float32>();
        // tmp = mat @ wp.diag(inert) @ wp.transpose(mat)                                         <L 725>
        var_29 = wp::diag(var_11);
        var_30 = wp::mul(var_3, var_29);
        var_31 = wp::transpose(var_3);
        var_32 = wp::mul(var_30, var_31);
        // res[0] = tmp[0, 0]                                                                     <L 726>
        var_35 = wp::extract(var_32, var_33, var_34);
        wp::assign_inplace(var_28, var_36, var_35);
        // res[1] = tmp[1, 1]                                                                     <L 727>
        var_39 = wp::extract(var_32, var_37, var_38);
        wp::assign_inplace(var_28, var_40, var_39);
        // res[2] = tmp[2, 2]                                                                     <L 728>
        var_43 = wp::extract(var_32, var_41, var_42);
        wp::assign_inplace(var_28, var_44, var_43);
        // res[3] = tmp[0, 1]                                                                     <L 729>
        var_47 = wp::extract(var_32, var_45, var_46);
        wp::assign_inplace(var_28, var_48, var_47);
        // res[4] = tmp[0, 2]                                                                     <L 730>
        var_51 = wp::extract(var_32, var_49, var_50);
        wp::assign_inplace(var_28, var_52, var_51);
        // res[5] = tmp[1, 2]                                                                     <L 731>
        var_55 = wp::extract(var_32, var_53, var_54);
        wp::assign_inplace(var_28, var_56, var_55);
        // res[0] += mass * (dif[1] * dif[1] + dif[2] * dif[2])                                   <L 733>
        var_58 = wp::extract(var_25, var_57);
        var_60 = wp::extract(var_25, var_59);
        var_61 = wp::mul(var_58, var_60);
        var_63 = wp::extract(var_25, var_62);
        var_65 = wp::extract(var_25, var_64);
        var_66 = wp::mul(var_63, var_65);
        var_67 = wp::add(var_61, var_66);
        var_68 = wp::mul(var_19, var_67);
        wp::add_inplace(var_28, var_69, var_68);
        // res[1] += mass * (dif[0] * dif[0] + dif[2] * dif[2])                                   <L 734>
        var_71 = wp::extract(var_25, var_70);
        var_73 = wp::extract(var_25, var_72);
        var_74 = wp::mul(var_71, var_73);
        var_76 = wp::extract(var_25, var_75);
        var_78 = wp::extract(var_25, var_77);
        var_79 = wp::mul(var_76, var_78);
        var_80 = wp::add(var_74, var_79);
        var_81 = wp::mul(var_19, var_80);
        wp::add_inplace(var_28, var_82, var_81);
        // res[2] += mass * (dif[0] * dif[0] + dif[1] * dif[1])                                   <L 735>
        var_84 = wp::extract(var_25, var_83);
        var_86 = wp::extract(var_25, var_85);
        var_87 = wp::mul(var_84, var_86);
        var_89 = wp::extract(var_25, var_88);
        var_91 = wp::extract(var_25, var_90);
        var_92 = wp::mul(var_89, var_91);
        var_93 = wp::add(var_87, var_92);
        var_94 = wp::mul(var_19, var_93);
        wp::add_inplace(var_28, var_95, var_94);
        // res[3] -= mass * dif[0] * dif[1]                                                       <L 736>
        var_97 = wp::extract(var_25, var_96);
        var_98 = wp::mul(var_19, var_97);
        var_100 = wp::extract(var_25, var_99);
        var_101 = wp::mul(var_98, var_100);
        wp::sub_inplace(var_28, var_102, var_101);
        // res[4] -= mass * dif[0] * dif[2]                                                       <L 737>
        var_104 = wp::extract(var_25, var_103);
        var_105 = wp::mul(var_19, var_104);
        var_107 = wp::extract(var_25, var_106);
        var_108 = wp::mul(var_105, var_107);
        wp::sub_inplace(var_28, var_109, var_108);
        // res[5] -= mass * dif[1] * dif[2]                                                       <L 738>
        var_111 = wp::extract(var_25, var_110);
        var_112 = wp::mul(var_19, var_111);
        var_114 = wp::extract(var_25, var_113);
        var_115 = wp::mul(var_112, var_114);
        wp::sub_inplace(var_28, var_116, var_115);
        // res[6] = mass * dif[0]                                                                 <L 740>
        var_118 = wp::extract(var_25, var_117);
        var_119 = wp::mul(var_19, var_118);
        wp::assign_inplace(var_28, var_120, var_119);
        // res[7] = mass * dif[1]                                                                 <L 741>
        var_122 = wp::extract(var_25, var_121);
        var_123 = wp::mul(var_19, var_122);
        wp::assign_inplace(var_28, var_124, var_123);
        // res[8] = mass * dif[2]                                                                 <L 742>
        var_126 = wp::extract(var_25, var_125);
        var_127 = wp::mul(var_19, var_126);
        wp::assign_inplace(var_28, var_128, var_127);
        // res[9] = mass                                                                          <L 744>
        wp::assign_inplace(var_28, var_129, var_19);
        // cinert_out[worldid, bodyid] = res                                                      <L 746>
        wp::array_store(var_cinert_out, var_0, var_1, var_28);
    }
}



extern "C" __global__ void _kinematics_branch_95f8028e_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_qpos0,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_mocapid,
    wp::array_t<wp::int32> var_body_jntnum,
    wp::array_t<wp::int32> var_body_jntadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_body_pos,
    wp::array_t<wp::quat_t<wp::float32>> var_body_quat,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_qposadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_jnt_pos,
    wp::array_t<wp::vec_t<3, wp::float32>> var_jnt_axis,
    wp::array_t<wp::int32> var_body_branches,
    wp::array_t<wp::int32> var_body_branch_start,
    wp::array_t<wp::float32> var_qpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mocap_pos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_mocap_quat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_out,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xanchor_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xaxis_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        const wp::int32 var_5 = 1;
        wp::int32 var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        wp::slice_t var_10;
        const wp::int32 var_11 = 0;
        wp::array_t<wp::float32> var_12;
        wp::range_t var_13;
        wp::int32 var_14;
        wp::int32* var_15;
        wp::int32 var_16;
        wp::int32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::int32* var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        wp::int32* var_24;
        wp::int32 var_25;
        wp::int32 var_26;
        const wp::int32 var_27 = 1;
        bool var_28;
        wp::int32* var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        const wp::int32 var_32 = 0;
        bool var_33;
        wp::int32* var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        wp::float32* var_37;
        const wp::int32 var_38 = 1;
        wp::int32 var_39;
        wp::float32* var_40;
        const wp::int32 var_41 = 2;
        wp::int32 var_42;
        wp::float32* var_43;
        wp::vec_t<3, wp::float32> var_44;
        wp::float32 var_45;
        wp::float32 var_46;
        wp::float32 var_47;
        const wp::int32 var_48 = 3;
        wp::int32 var_49;
        wp::float32* var_50;
        const wp::int32 var_51 = 4;
        wp::int32 var_52;
        wp::float32* var_53;
        const wp::int32 var_54 = 5;
        wp::int32 var_55;
        wp::float32* var_56;
        const wp::int32 var_57 = 6;
        wp::int32 var_58;
        wp::float32* var_59;
        wp::quat_t<wp::float32> var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        wp::float32 var_63;
        wp::float32 var_64;
        wp::quat_t<wp::float32> var_65;
        wp::shape_t* var_66;
        const wp::int32 var_67 = 0;
        wp::int32 var_68;
        wp::shape_t var_69;
        wp::int32 var_70;
        wp::vec_t<3, wp::float32>* var_71;
        wp::vec_t<3, wp::float32> var_72;
        wp::shape_t* var_73;
        const wp::int32 var_74 = 0;
        wp::int32 var_75;
        wp::shape_t var_76;
        wp::int32 var_77;
        wp::int32* var_78;
        wp::int32 var_79;
        wp::int32 var_80;
        wp::int32* var_81;
        wp::int32 var_82;
        wp::int32 var_83;
        const wp::int32 var_84 = 0;
        bool var_85;
        wp::vec_t<3, wp::float32>* var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::vec_t<3, wp::float32> var_88;
        wp::quat_t<wp::float32>* var_89;
        wp::quat_t<wp::float32> var_90;
        wp::quat_t<wp::float32> var_91;
        wp::vec_t<3, wp::float32> var_92;
        wp::quat_t<wp::float32> var_93;
        wp::shape_t* var_94;
        const wp::int32 var_95 = 0;
        wp::int32 var_96;
        wp::shape_t var_97;
        wp::int32 var_98;
        wp::vec_t<3, wp::float32>* var_99;
        wp::vec_t<3, wp::float32> var_100;
        wp::vec_t<3, wp::float32> var_101;
        wp::shape_t* var_102;
        const wp::int32 var_103 = 0;
        wp::int32 var_104;
        wp::shape_t var_105;
        wp::int32 var_106;
        wp::quat_t<wp::float32>* var_107;
        wp::quat_t<wp::float32> var_108;
        wp::quat_t<wp::float32> var_109;
        wp::vec_t<3, wp::float32> var_110;
        wp::quat_t<wp::float32> var_111;
        const wp::int32 var_112 = 0;
        bool var_113;
        wp::quat_t<wp::float32>* var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::quat_t<wp::float32> var_116;
        wp::vec_t<3, wp::float32>* var_117;
        wp::vec_t<3, wp::float32> var_118;
        wp::vec_t<3, wp::float32> var_119;
        wp::quat_t<wp::float32>* var_120;
        wp::quat_t<wp::float32> var_121;
        wp::quat_t<wp::float32> var_122;
        wp::vec_t<3, wp::float32> var_123;
        wp::quat_t<wp::float32> var_124;
        wp::range_t var_125;
        wp::int32 var_126;
        wp::int32* var_127;
        wp::int32 var_128;
        wp::int32 var_129;
        wp::int32* var_130;
        wp::int32 var_131;
        wp::int32 var_132;
        wp::shape_t* var_133;
        const wp::int32 var_134 = 0;
        wp::int32 var_135;
        wp::shape_t var_136;
        wp::int32 var_137;
        wp::vec_t<3, wp::float32>* var_138;
        wp::vec_t<3, wp::float32> var_139;
        wp::vec_t<3, wp::float32> var_140;
        wp::vec_t<3, wp::float32>* var_141;
        wp::vec_t<3, wp::float32> var_142;
        wp::vec_t<3, wp::float32> var_143;
        wp::vec_t<3, wp::float32> var_144;
        wp::vec_t<3, wp::float32> var_145;
        const wp::int32 var_146 = 1;
        bool var_147;
        const wp::int32 var_148 = 0;
        wp::int32 var_149;
        wp::float32* var_150;
        const wp::int32 var_151 = 1;
        wp::int32 var_152;
        wp::float32* var_153;
        const wp::int32 var_154 = 2;
        wp::int32 var_155;
        wp::float32* var_156;
        const wp::int32 var_157 = 3;
        wp::int32 var_158;
        wp::float32* var_159;
        wp::quat_t<wp::float32> var_160;
        wp::float32 var_161;
        wp::float32 var_162;
        wp::float32 var_163;
        wp::float32 var_164;
        wp::quat_t<wp::float32> var_165;
        wp::quat_t<wp::float32> var_166;
        wp::vec_t<3, wp::float32>* var_167;
        wp::vec_t<3, wp::float32> var_168;
        wp::vec_t<3, wp::float32> var_169;
        wp::vec_t<3, wp::float32> var_170;
        wp::vec_t<3, wp::float32> var_171;
        wp::quat_t<wp::float32> var_172;
        const wp::int32 var_173 = 2;
        bool var_174;
        wp::float32* var_175;
        wp::shape_t* var_176;
        const wp::int32 var_177 = 0;
        wp::int32 var_178;
        wp::shape_t var_179;
        wp::int32 var_180;
        wp::float32* var_181;
        wp::float32 var_182;
        wp::float32 var_183;
        wp::float32 var_184;
        wp::vec_t<3, wp::float32> var_185;
        wp::vec_t<3, wp::float32> var_186;
        wp::vec_t<3, wp::float32> var_187;
        const wp::int32 var_188 = 3;
        bool var_189;
        wp::shape_t* var_190;
        const wp::int32 var_191 = 0;
        wp::int32 var_192;
        wp::shape_t var_193;
        wp::int32 var_194;
        wp::float32* var_195;
        wp::float32 var_196;
        wp::float32 var_197;
        wp::float32* var_198;
        wp::float32 var_199;
        wp::float32 var_200;
        wp::quat_t<wp::float32> var_201;
        wp::quat_t<wp::float32> var_202;
        wp::vec_t<3, wp::float32>* var_203;
        wp::vec_t<3, wp::float32> var_204;
        wp::vec_t<3, wp::float32> var_205;
        wp::vec_t<3, wp::float32> var_206;
        wp::vec_t<3, wp::float32> var_207;
        wp::quat_t<wp::float32> var_208;
        wp::vec_t<3, wp::float32> var_209;
        wp::quat_t<wp::float32> var_210;
        wp::vec_t<3, wp::float32> var_211;
        wp::quat_t<wp::float32> var_212;
        const wp::int32 var_213 = 1;
        wp::int32 var_214;
        wp::quat_t<wp::float32> var_215;
        //---------
        // forward
        // def _kinematics_branch(                                                                <L 48>
        // worldid, branchid = wp.tid()                                                           <L 73>
        builtin_tid2d(var_0, var_1);
        // start = body_branch_start[branchid]                                                    <L 75>
        var_2 = wp::address(var_body_branch_start, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // end = body_branch_start[branchid + 1]                                                  <L 76>
        var_6 = wp::add(var_1, var_5);
        var_7 = wp::address(var_body_branch_start, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // qpos = qpos_in[worldid]                                                                <L 78>
        var_10 = wp::slice_t(var_0, var_0, var_11);
        var_12 = wp::view(var_qpos_in, var_10);
        // for i in range(start, end):                                                            <L 80>
        var_13 = wp::range(var_3, var_8);
        start_for_0:;
            if (iter_cmp(var_13) == 0) goto end_for_0;
            var_14 = wp::iter_next(var_13);
            // bodyid = body_branches[i]                                                          <L 81>
            var_15 = wp::address(var_body_branches, var_14);
            var_17 = wp::load(var_15);
            var_16 = wp::copy(var_17);
            // pid = body_parentid[bodyid]                                                        <L 82>
            var_18 = wp::address(var_body_parentid, var_16);
            var_20 = wp::load(var_18);
            var_19 = wp::copy(var_20);
            // jntadr = body_jntadr[bodyid]                                                       <L 83>
            var_21 = wp::address(var_body_jntadr, var_16);
            var_23 = wp::load(var_21);
            var_22 = wp::copy(var_23);
            // jntnum = body_jntnum[bodyid]                                                       <L 84>
            var_24 = wp::address(var_body_jntnum, var_16);
            var_26 = wp::load(var_24);
            var_25 = wp::copy(var_26);
            // if jntnum == 1:                                                                    <L 86>
            var_28 = (var_25 == var_27);
            if (var_28) {
                // jnt_type_ = jnt_type[jntadr]                                                   <L 87>
                var_29 = wp::address(var_jnt_type, var_22);
                var_31 = wp::load(var_29);
                var_30 = wp::copy(var_31);
                // if jnt_type_ == JointType.FREE:                                                <L 88>
                var_33 = (var_30 == var_32);
                if (var_33) {
                    // qadr = jnt_qposadr[jntadr]                                                 <L 89>
                    var_34 = wp::address(var_jnt_qposadr, var_22);
                    var_36 = wp::load(var_34);
                    var_35 = wp::copy(var_36);
                    // xpos = wp.vec3(qpos[qadr], qpos[qadr + 1], qpos[qadr + 2])                 <L 90>
                    var_37 = wp::address(var_12, var_35);
                    var_39 = wp::add(var_35, var_38);
                    var_40 = wp::address(var_12, var_39);
                    var_42 = wp::add(var_35, var_41);
                    var_43 = wp::address(var_12, var_42);
                    var_45 = wp::load(var_37);
                    var_46 = wp::load(var_40);
                    var_47 = wp::load(var_43);
                    var_44 = wp::vec_t<3, wp::float32>(var_45, var_46, var_47);
                    // xquat = wp.quat(qpos[qadr + 3], qpos[qadr + 4], qpos[qadr + 5], qpos[qadr + 6])       <L 91>
                    var_49 = wp::add(var_35, var_48);
                    var_50 = wp::address(var_12, var_49);
                    var_52 = wp::add(var_35, var_51);
                    var_53 = wp::address(var_12, var_52);
                    var_55 = wp::add(var_35, var_54);
                    var_56 = wp::address(var_12, var_55);
                    var_58 = wp::add(var_35, var_57);
                    var_59 = wp::address(var_12, var_58);
                    var_61 = wp::load(var_50);
                    var_62 = wp::load(var_53);
                    var_63 = wp::load(var_56);
                    var_64 = wp::load(var_59);
                    var_60 = wp::quat_t<wp::float32>(var_61, var_62, var_63, var_64);
                    // xquat = wp.normalize(xquat)                                                <L 92>
                    var_65 = wp::normalize(var_60);
                    // xpos_out[worldid, bodyid] = xpos                                           <L 94>
                    wp::array_store(var_xpos_out, var_0, var_16, var_44);
                    // xquat_out[worldid, bodyid] = xquat                                         <L 95>
                    wp::array_store(var_xquat_out, var_0, var_16, var_65);
                    // xanchor_out[worldid, jntadr] = xpos                                        <L 96>
                    wp::array_store(var_xanchor_out, var_0, var_22, var_44);
                    // xaxis_out[worldid, jntadr] = jnt_axis[worldid % jnt_axis.shape[0], jntadr]       <L 97>
                    var_66 = &(var_jnt_axis.shape);
                    var_69 = wp::load(var_66);
                    var_68 = wp::extract(var_69, var_67);
                    var_70 = wp::mod(var_0, var_68);
                    var_71 = wp::address(var_jnt_axis, var_70, var_22);
                    var_72 = wp::load(var_71);
                    wp::array_store(var_xaxis_out, var_0, var_22, var_72);
                    // continue                                                                   <L 98>
                    goto start_for_0;
                }
            }
            // jnt_pos_id = worldid % jnt_pos.shape[0]                                            <L 102>
            var_73 = &(var_jnt_pos.shape);
            var_76 = wp::load(var_73);
            var_75 = wp::extract(var_76, var_74);
            var_77 = wp::mod(var_0, var_75);
            // pid = body_parentid[bodyid]                                                        <L 103>
            var_78 = wp::address(var_body_parentid, var_16);
            var_80 = wp::load(var_78);
            var_79 = wp::copy(var_80);
            // mocapid = body_mocapid[bodyid]                                                     <L 106>
            var_81 = wp::address(var_body_mocapid, var_16);
            var_83 = wp::load(var_81);
            var_82 = wp::copy(var_83);
            // if mocapid >= 0:                                                                   <L 107>
            var_85 = (var_82 >= var_84);
            if (var_85) {
                // xpos = mocap_pos_in[worldid, mocapid]                                          <L 108>
                var_86 = wp::address(var_mocap_pos_in, var_0, var_82);
                var_88 = wp::load(var_86);
                var_87 = wp::copy(var_88);
                // xquat = mocap_quat_in[worldid, mocapid]                                        <L 109>
                var_89 = wp::address(var_mocap_quat_in, var_0, var_82);
                var_91 = wp::load(var_89);
                var_90 = wp::copy(var_91);
            }
            var_92 = wp::where(var_85, var_87, var_44);
            var_93 = wp::where(var_85, var_90, var_65);
            if (!var_85) {
                // xpos = body_pos[worldid % body_pos.shape[0], bodyid]                           <L 111>
                var_94 = &(var_body_pos.shape);
                var_97 = wp::load(var_94);
                var_96 = wp::extract(var_97, var_95);
                var_98 = wp::mod(var_0, var_96);
                var_99 = wp::address(var_body_pos, var_98, var_16);
                var_101 = wp::load(var_99);
                var_100 = wp::copy(var_101);
                // xquat = body_quat[worldid % body_quat.shape[0], bodyid]                        <L 112>
                var_102 = &(var_body_quat.shape);
                var_105 = wp::load(var_102);
                var_104 = wp::extract(var_105, var_103);
                var_106 = wp::mod(var_0, var_104);
                var_107 = wp::address(var_body_quat, var_106, var_16);
                var_109 = wp::load(var_107);
                var_108 = wp::copy(var_109);
            }
            var_110 = wp::where(var_85, var_92, var_100);
            var_111 = wp::where(var_85, var_93, var_108);
            // if pid >= 0:                                                                       <L 114>
            var_113 = (var_79 >= var_112);
            if (var_113) {
                // xpos = math.rot_vec_quat(xpos, xquat_out[worldid, pid]) + xpos_out[worldid, pid]       <L 115>
                var_114 = wp::address(var_xquat_out, var_0, var_79);
                var_116 = wp::load(var_114);
                var_115 = rot_vec_quat_0(var_110, var_116);
                var_117 = wp::address(var_xpos_out, var_0, var_79);
                var_119 = wp::load(var_117);
                var_118 = wp::add(var_115, var_119);
                // xquat = math.mul_quat(xquat_out[worldid, pid], xquat)                          <L 116>
                var_120 = wp::address(var_xquat_out, var_0, var_79);
                var_122 = wp::load(var_120);
                var_121 = mul_quat_0(var_122, var_111);
            }
            var_123 = wp::where(var_113, var_118, var_110);
            var_124 = wp::where(var_113, var_121, var_111);
            // for _ in range(jntnum):                                                            <L 118>
            var_125 = wp::range(var_25);
            start_for_2:;
                if (iter_cmp(var_125) == 0) goto end_for_2;
                var_126 = wp::iter_next(var_125);
                // qadr = jnt_qposadr[jntadr]                                                     <L 119>
                var_127 = wp::address(var_jnt_qposadr, var_22);
                var_129 = wp::load(var_127);
                var_128 = wp::copy(var_129);
                // jnt_type_ = jnt_type[jntadr]                                                   <L 120>
                var_130 = wp::address(var_jnt_type, var_22);
                var_132 = wp::load(var_130);
                var_131 = wp::copy(var_132);
                // jnt_axis_ = jnt_axis[worldid % jnt_axis.shape[0], jntadr]                      <L 121>
                var_133 = &(var_jnt_axis.shape);
                var_136 = wp::load(var_133);
                var_135 = wp::extract(var_136, var_134);
                var_137 = wp::mod(var_0, var_135);
                var_138 = wp::address(var_jnt_axis, var_137, var_22);
                var_140 = wp::load(var_138);
                var_139 = wp::copy(var_140);
                // xanchor = math.rot_vec_quat(jnt_pos[jnt_pos_id, jntadr], xquat) + xpos         <L 122>
                var_141 = wp::address(var_jnt_pos, var_77, var_22);
                var_143 = wp::load(var_141);
                var_142 = rot_vec_quat_0(var_143, var_124);
                var_144 = wp::add(var_142, var_123);
                // xaxis = math.rot_vec_quat(jnt_axis_, xquat)                                    <L 123>
                var_145 = rot_vec_quat_0(var_139, var_124);
                // if jnt_type_ == JointType.BALL:                                                <L 125>
                var_147 = (var_131 == var_146);
                if (var_147) {
                    // qloc = wp.quat(qpos[qadr + 0], qpos[qadr + 1], qpos[qadr + 2], qpos[qadr + 3])       <L 126>
                    var_149 = wp::add(var_128, var_148);
                    var_150 = wp::address(var_12, var_149);
                    var_152 = wp::add(var_128, var_151);
                    var_153 = wp::address(var_12, var_152);
                    var_155 = wp::add(var_128, var_154);
                    var_156 = wp::address(var_12, var_155);
                    var_158 = wp::add(var_128, var_157);
                    var_159 = wp::address(var_12, var_158);
                    var_161 = wp::load(var_150);
                    var_162 = wp::load(var_153);
                    var_163 = wp::load(var_156);
                    var_164 = wp::load(var_159);
                    var_160 = wp::quat_t<wp::float32>(var_161, var_162, var_163, var_164);
                    // qloc = wp.normalize(qloc)                                                  <L 127>
                    var_165 = wp::normalize(var_160);
                    // xquat = math.mul_quat(xquat, qloc)                                         <L 128>
                    var_166 = mul_quat_0(var_124, var_165);
                    // xpos = xanchor - math.rot_vec_quat(jnt_pos[jnt_pos_id, jntadr], xquat)       <L 130>
                    var_167 = wp::address(var_jnt_pos, var_77, var_22);
                    var_169 = wp::load(var_167);
                    var_168 = rot_vec_quat_0(var_169, var_166);
                    var_170 = wp::sub(var_144, var_168);
                }
                var_171 = wp::where(var_147, var_170, var_123);
                var_172 = wp::where(var_147, var_166, var_124);
                if (!var_147) {
                    // elif jnt_type_ == JointType.SLIDE:                                         <L 131>
                    var_174 = (var_131 == var_173);
                    if (var_174) {
                        // xpos += xaxis * (qpos[qadr] - qpos0[worldid % qpos0.shape[0], qadr])       <L 132>
                        var_175 = wp::address(var_12, var_128);
                        var_176 = &(var_qpos0.shape);
                        var_179 = wp::load(var_176);
                        var_178 = wp::extract(var_179, var_177);
                        var_180 = wp::mod(var_0, var_178);
                        var_181 = wp::address(var_qpos0, var_180, var_128);
                        var_183 = wp::load(var_175);
                        var_184 = wp::load(var_181);
                        var_182 = wp::sub(var_183, var_184);
                        var_185 = wp::mul(var_145, var_182);
                        var_186 = wp::add(var_171, var_185);
                    }
                    var_187 = wp::where(var_174, var_186, var_171);
                    if (!var_174) {
                        // elif jnt_type_ == JointType.HINGE:                                     <L 133>
                        var_189 = (var_131 == var_188);
                        if (var_189) {
                            // qpos0_ = qpos0[worldid % qpos0.shape[0], qadr]                     <L 134>
                            var_190 = &(var_qpos0.shape);
                            var_193 = wp::load(var_190);
                            var_192 = wp::extract(var_193, var_191);
                            var_194 = wp::mod(var_0, var_192);
                            var_195 = wp::address(var_qpos0, var_194, var_128);
                            var_197 = wp::load(var_195);
                            var_196 = wp::copy(var_197);
                            // qloc_ = math.axis_angle_to_quat(jnt_axis_, qpos[qadr] - qpos0_)       <L 135>
                            var_198 = wp::address(var_12, var_128);
                            var_200 = wp::load(var_198);
                            var_199 = wp::sub(var_200, var_196);
                            var_201 = axis_angle_to_quat_0(var_139, var_199);
                            // xquat = math.mul_quat(xquat, qloc_)                                <L 136>
                            var_202 = mul_quat_0(var_172, var_201);
                            // xpos = xanchor - math.rot_vec_quat(jnt_pos[jnt_pos_id, jntadr], xquat)       <L 138>
                            var_203 = wp::address(var_jnt_pos, var_77, var_22);
                            var_205 = wp::load(var_203);
                            var_204 = rot_vec_quat_0(var_205, var_202);
                            var_206 = wp::sub(var_144, var_204);
                        }
                        var_207 = wp::where(var_189, var_206, var_187);
                        var_208 = wp::where(var_189, var_202, var_172);
                    }
                    var_209 = wp::where(var_174, var_187, var_207);
                    var_210 = wp::where(var_174, var_172, var_208);
                }
                var_211 = wp::where(var_147, var_171, var_209);
                var_212 = wp::where(var_147, var_172, var_210);
                // xanchor_out[worldid, jntadr] = xanchor                                         <L 140>
                wp::array_store(var_xanchor_out, var_0, var_22, var_144);
                // xaxis_out[worldid, jntadr] = xaxis                                             <L 141>
                wp::array_store(var_xaxis_out, var_0, var_22, var_145);
                // jntadr += 1                                                                    <L 142>
                var_214 = wp::add(var_22, var_213);
                wp::assign(var_22, var_214);
                wp::assign(var_30, var_131);
                wp::assign(var_35, var_128);
                wp::assign(var_123, var_211);
                wp::assign(var_124, var_212);
                goto start_for_2;
            end_for_2:;
            // xquat = wp.normalize(xquat)                                                        <L 144>
            var_215 = wp::normalize(var_124);
            // xpos_out[worldid, bodyid] = xpos                                                   <L 145>
            wp::array_store(var_xpos_out, var_0, var_16, var_123);
            // xquat_out[worldid, bodyid] = xquat                                                 <L 146>
            wp::array_store(var_xquat_out, var_0, var_16, var_215);
            goto start_for_0;
        end_for_0:;
    }
}



extern "C" __global__ void _cam_local_to_global_ed67ebfa_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_cam_mode,
    wp::array_t<wp::int32> var_cam_bodyid,
    wp::array_t<wp::int32> var_cam_targetbodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cam_pos,
    wp::array_t<wp::quat_t<wp::float32>> var_cam_quat,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cam_poscom0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cam_pos0,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_cam_mat0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cam_xpos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_cam_xmat_out)
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
        wp::int32 var_1;
        wp::shape_t* var_2;
        const wp::int32 var_3 = 0;
        wp::int32 var_4;
        wp::shape_t var_5;
        wp::int32 var_6;
        wp::shape_t* var_7;
        const wp::int32 var_8 = 0;
        wp::int32 var_9;
        wp::shape_t var_10;
        wp::int32 var_11;
        bool var_12;
        wp::int32* var_13;
        const wp::int32 var_14 = 3;
        bool var_15;
        wp::int32 var_16;
        wp::int32* var_17;
        const wp::int32 var_18 = 4;
        bool var_19;
        wp::int32 var_20;
        bool var_21;
        wp::int32* var_22;
        const wp::int32 var_23 = 0;
        bool var_24;
        wp::int32 var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::vec_t<3, wp::float32>* var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::vec_t<3, wp::float32> var_31;
        wp::quat_t<wp::float32>* var_32;
        wp::quat_t<wp::float32> var_33;
        wp::quat_t<wp::float32> var_34;
        wp::vec_t<3, wp::float32>* var_35;
        wp::vec_t<3, wp::float32> var_36;
        wp::vec_t<3, wp::float32> var_37;
        wp::vec_t<3, wp::float32> var_38;
        wp::quat_t<wp::float32>* var_39;
        wp::quat_t<wp::float32> var_40;
        wp::quat_t<wp::float32> var_41;
        wp::mat_t<3, 3, wp::float32> var_42;
        wp::int32* var_43;
        const wp::int32 var_44 = 1;
        bool var_45;
        wp::int32 var_46;
        wp::shape_t* var_47;
        const wp::int32 var_48 = 0;
        wp::int32 var_49;
        wp::shape_t var_50;
        wp::int32 var_51;
        wp::mat_t<3, 3, wp::float32>* var_52;
        wp::mat_t<3, 3, wp::float32> var_53;
        wp::int32* var_54;
        wp::vec_t<3, wp::float32>* var_55;
        wp::int32 var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::shape_t* var_59;
        const wp::int32 var_60 = 0;
        wp::int32 var_61;
        wp::shape_t var_62;
        wp::int32 var_63;
        wp::vec_t<3, wp::float32>* var_64;
        wp::vec_t<3, wp::float32> var_65;
        wp::vec_t<3, wp::float32> var_66;
        wp::int32* var_67;
        const wp::int32 var_68 = 2;
        bool var_69;
        wp::int32 var_70;
        wp::shape_t* var_71;
        const wp::int32 var_72 = 0;
        wp::int32 var_73;
        wp::shape_t var_74;
        wp::int32 var_75;
        wp::mat_t<3, 3, wp::float32>* var_76;
        wp::mat_t<3, 3, wp::float32> var_77;
        wp::int32* var_78;
        wp::vec_t<3, wp::float32>* var_79;
        wp::int32 var_80;
        wp::shape_t* var_81;
        const wp::int32 var_82 = 0;
        wp::int32 var_83;
        wp::shape_t var_84;
        wp::int32 var_85;
        wp::vec_t<3, wp::float32>* var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::vec_t<3, wp::float32> var_88;
        wp::vec_t<3, wp::float32> var_89;
        bool var_90;
        wp::int32* var_91;
        const wp::int32 var_92 = 3;
        bool var_93;
        wp::int32 var_94;
        wp::int32* var_95;
        const wp::int32 var_96 = 4;
        bool var_97;
        wp::int32 var_98;
        wp::int32* var_99;
        wp::int32 var_100;
        wp::int32 var_101;
        wp::vec_t<3, wp::float32>* var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::vec_t<3, wp::float32> var_104;
        wp::quat_t<wp::float32>* var_105;
        wp::quat_t<wp::float32> var_106;
        wp::quat_t<wp::float32> var_107;
        wp::vec_t<3, wp::float32>* var_108;
        wp::vec_t<3, wp::float32> var_109;
        wp::vec_t<3, wp::float32> var_110;
        wp::vec_t<3, wp::float32> var_111;
        wp::int32* var_112;
        wp::vec_t<3, wp::float32>* var_113;
        wp::int32 var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::vec_t<3, wp::float32> var_116;
        wp::int32* var_117;
        const wp::int32 var_118 = 4;
        bool var_119;
        wp::int32 var_120;
        wp::int32* var_121;
        wp::vec_t<3, wp::float32>* var_122;
        wp::int32 var_123;
        wp::vec_t<3, wp::float32> var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32> var_126;
        wp::vec_t<3, wp::float32>* var_127;
        wp::vec_t<3, wp::float32> var_128;
        wp::vec_t<3, wp::float32> var_129;
        wp::vec_t<3, wp::float32> var_130;
        const wp::float32 var_131 = 0.0;
        const wp::float32 var_132 = 0.0;
        const wp::float32 var_133 = 1.0;
        wp::vec_t<3, wp::float32> var_134;
        wp::vec_t<3, wp::float32> var_135;
        wp::vec_t<3, wp::float32> var_136;
        wp::vec_t<3, wp::float32> var_137;
        wp::vec_t<3, wp::float32> var_138;
        const wp::int32 var_139 = 0;
        wp::float32 var_140;
        const wp::int32 var_141 = 0;
        wp::float32 var_142;
        const wp::int32 var_143 = 0;
        wp::float32 var_144;
        const wp::int32 var_145 = 1;
        wp::float32 var_146;
        const wp::int32 var_147 = 1;
        wp::float32 var_148;
        const wp::int32 var_149 = 1;
        wp::float32 var_150;
        const wp::int32 var_151 = 2;
        wp::float32 var_152;
        const wp::int32 var_153 = 2;
        wp::float32 var_154;
        const wp::int32 var_155 = 2;
        wp::float32 var_156;
        wp::mat_t<3, 3, wp::float32> var_157;
        wp::int32 var_158;
        wp::vec_t<3, wp::float32> var_159;
        wp::quat_t<wp::float32> var_160;
        wp::int32* var_161;
        wp::int32 var_162;
        wp::int32 var_163;
        wp::vec_t<3, wp::float32>* var_164;
        wp::vec_t<3, wp::float32> var_165;
        wp::vec_t<3, wp::float32> var_166;
        wp::quat_t<wp::float32>* var_167;
        wp::quat_t<wp::float32> var_168;
        wp::quat_t<wp::float32> var_169;
        wp::vec_t<3, wp::float32>* var_170;
        wp::vec_t<3, wp::float32> var_171;
        wp::vec_t<3, wp::float32> var_172;
        wp::vec_t<3, wp::float32> var_173;
        wp::quat_t<wp::float32>* var_174;
        wp::quat_t<wp::float32> var_175;
        wp::quat_t<wp::float32> var_176;
        wp::mat_t<3, 3, wp::float32> var_177;
        wp::int32 var_178;
        wp::vec_t<3, wp::float32> var_179;
        wp::quat_t<wp::float32> var_180;
        wp::int32 var_181;
        wp::vec_t<3, wp::float32> var_182;
        wp::quat_t<wp::float32> var_183;
        wp::int32 var_184;
        wp::vec_t<3, wp::float32> var_185;
        wp::quat_t<wp::float32> var_186;
        wp::int32 var_187;
        wp::vec_t<3, wp::float32> var_188;
        wp::quat_t<wp::float32> var_189;
        //---------
        // forward
        // def _cam_local_to_global(                                                              <L 829>
        // worldid, camid = wp.tid()                                                              <L 847>
        builtin_tid2d(var_0, var_1);
        // cam_pos_id = worldid % cam_pos.shape[0]                                                <L 848>
        var_2 = &(var_cam_pos.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_0, var_4);
        // cam_quat_id = worldid % cam_quat.shape[0]                                              <L 849>
        var_7 = &(var_cam_quat.shape);
        var_10 = wp::load(var_7);
        var_9 = wp::extract(var_10, var_8);
        var_11 = wp::mod(var_0, var_9);
        // is_target_cam = (cam_mode[camid] == CamLightType.TARGETBODY) or (cam_mode[camid] == CamLightType.TARGETBODYCOM)       <L 850>
        var_13 = wp::address(var_cam_mode, var_1);
        var_16 = wp::load(var_13);
        var_15 = (var_16 == var_14);
        var_12 = var_15;
        if (!var_12) {
            var_17 = wp::address(var_cam_mode, var_1);
            var_20 = wp::load(var_17);
            var_19 = (var_20 == var_18);
            var_12 = var_12 || var_19;
        }
        // invalid_target = is_target_cam and (cam_targetbodyid[camid] < 0)                       <L 851>
        var_21 = var_12;
        if (var_21) {
            var_22 = wp::address(var_cam_targetbodyid, var_1);
            var_25 = wp::load(var_22);
            var_24 = (var_25 < var_23);
            var_21 = var_21 && var_24;
        }
        // if invalid_target:                                                                     <L 852>
        if (var_21) {
            // bodyid = cam_bodyid[camid]                                                         <L 853>
            var_26 = wp::address(var_cam_bodyid, var_1);
            var_28 = wp::load(var_26);
            var_27 = wp::copy(var_28);
            // xpos = xpos_in[worldid, bodyid]                                                    <L 854>
            var_29 = wp::address(var_xpos_in, var_0, var_27);
            var_31 = wp::load(var_29);
            var_30 = wp::copy(var_31);
            // xquat = xquat_in[worldid, bodyid]                                                  <L 855>
            var_32 = wp::address(var_xquat_in, var_0, var_27);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // cam_xpos_out[worldid, camid] = xpos + math.rot_vec_quat(cam_pos[cam_pos_id, camid], xquat)       <L 856>
            var_35 = wp::address(var_cam_pos, var_6, var_1);
            var_37 = wp::load(var_35);
            var_36 = rot_vec_quat_0(var_37, var_33);
            var_38 = wp::add(var_30, var_36);
            wp::array_store(var_cam_xpos_out, var_0, var_1, var_38);
            // cam_xmat_out[worldid, camid] = math.quat_to_mat(math.mul_quat(xquat, cam_quat[cam_quat_id, camid]))       <L 857>
            var_39 = wp::address(var_cam_quat, var_11, var_1);
            var_41 = wp::load(var_39);
            var_40 = mul_quat_0(var_33, var_41);
            var_42 = quat_to_mat_0(var_40);
            wp::array_store(var_cam_xmat_out, var_0, var_1, var_42);
        }
        if (!var_21) {
            // elif cam_mode[camid] == CamLightType.TRACK:                                        <L 858>
            var_43 = wp::address(var_cam_mode, var_1);
            var_46 = wp::load(var_43);
            var_45 = (var_46 == var_44);
            if (var_45) {
                // cam_xmat_out[worldid, camid] = cam_mat0[worldid % cam_mat0.shape[0], camid]       <L 859>
                var_47 = &(var_cam_mat0.shape);
                var_50 = wp::load(var_47);
                var_49 = wp::extract(var_50, var_48);
                var_51 = wp::mod(var_0, var_49);
                var_52 = wp::address(var_cam_mat0, var_51, var_1);
                var_53 = wp::load(var_52);
                wp::array_store(var_cam_xmat_out, var_0, var_1, var_53);
                // body_xpos = xpos_in[worldid, cam_bodyid[camid]]                                <L 860>
                var_54 = wp::address(var_cam_bodyid, var_1);
                var_56 = wp::load(var_54);
                var_55 = wp::address(var_xpos_in, var_0, var_56);
                var_58 = wp::load(var_55);
                var_57 = wp::copy(var_58);
                // cam_xpos_out[worldid, camid] = body_xpos + cam_pos0[worldid % cam_pos0.shape[0], camid]       <L 861>
                var_59 = &(var_cam_pos0.shape);
                var_62 = wp::load(var_59);
                var_61 = wp::extract(var_62, var_60);
                var_63 = wp::mod(var_0, var_61);
                var_64 = wp::address(var_cam_pos0, var_63, var_1);
                var_66 = wp::load(var_64);
                var_65 = wp::add(var_57, var_66);
                wp::array_store(var_cam_xpos_out, var_0, var_1, var_65);
            }
            if (!var_45) {
                // elif cam_mode[camid] == CamLightType.TRACKCOM:                                 <L 862>
                var_67 = wp::address(var_cam_mode, var_1);
                var_70 = wp::load(var_67);
                var_69 = (var_70 == var_68);
                if (var_69) {
                    // cam_xmat_out[worldid, camid] = cam_mat0[worldid % cam_mat0.shape[0], camid]       <L 863>
                    var_71 = &(var_cam_mat0.shape);
                    var_74 = wp::load(var_71);
                    var_73 = wp::extract(var_74, var_72);
                    var_75 = wp::mod(var_0, var_73);
                    var_76 = wp::address(var_cam_mat0, var_75, var_1);
                    var_77 = wp::load(var_76);
                    wp::array_store(var_cam_xmat_out, var_0, var_1, var_77);
                    // cam_xpos_out[worldid, camid] = (                                           <L 864>
                    // subtree_com_in[worldid, cam_bodyid[camid]] + cam_poscom0[worldid % cam_poscom0.shape[0], camid]       <L 865>
                    var_78 = wp::address(var_cam_bodyid, var_1);
                    var_80 = wp::load(var_78);
                    var_79 = wp::address(var_subtree_com_in, var_0, var_80);
                    var_81 = &(var_cam_poscom0.shape);
                    var_84 = wp::load(var_81);
                    var_83 = wp::extract(var_84, var_82);
                    var_85 = wp::mod(var_0, var_83);
                    var_86 = wp::address(var_cam_poscom0, var_85, var_1);
                    var_88 = wp::load(var_79);
                    var_89 = wp::load(var_86);
                    var_87 = wp::add(var_88, var_89);
                    // cam_xpos_out[worldid, camid] = (                                           <L 864>
                    wp::array_store(var_cam_xpos_out, var_0, var_1, var_87);
                }
                if (!var_69) {
                    // elif cam_mode[camid] == CamLightType.TARGETBODY or cam_mode[camid] == CamLightType.TARGETBODYCOM:       <L 867>
                    var_91 = wp::address(var_cam_mode, var_1);
                    var_94 = wp::load(var_91);
                    var_93 = (var_94 == var_92);
                    var_90 = var_93;
                    if (!var_90) {
                        var_95 = wp::address(var_cam_mode, var_1);
                        var_98 = wp::load(var_95);
                        var_97 = (var_98 == var_96);
                        var_90 = var_90 || var_97;
                    }
                    if (var_90) {
                        // bodyid = cam_bodyid[camid]                                             <L 868>
                        var_99 = wp::address(var_cam_bodyid, var_1);
                        var_101 = wp::load(var_99);
                        var_100 = wp::copy(var_101);
                        // xpos = xpos_in[worldid, bodyid]                                        <L 869>
                        var_102 = wp::address(var_xpos_in, var_0, var_100);
                        var_104 = wp::load(var_102);
                        var_103 = wp::copy(var_104);
                        // xquat = xquat_in[worldid, bodyid]                                      <L 870>
                        var_105 = wp::address(var_xquat_in, var_0, var_100);
                        var_107 = wp::load(var_105);
                        var_106 = wp::copy(var_107);
                        // cam_xpos_out[worldid, camid] = xpos + math.rot_vec_quat(cam_pos[cam_pos_id, camid], xquat)       <L 871>
                        var_108 = wp::address(var_cam_pos, var_6, var_1);
                        var_110 = wp::load(var_108);
                        var_109 = rot_vec_quat_0(var_110, var_106);
                        var_111 = wp::add(var_103, var_109);
                        wp::array_store(var_cam_xpos_out, var_0, var_1, var_111);
                        // pos = xpos_in[worldid, cam_targetbodyid[camid]]                        <L 872>
                        var_112 = wp::address(var_cam_targetbodyid, var_1);
                        var_114 = wp::load(var_112);
                        var_113 = wp::address(var_xpos_in, var_0, var_114);
                        var_116 = wp::load(var_113);
                        var_115 = wp::copy(var_116);
                        // if cam_mode[camid] == CamLightType.TARGETBODYCOM:                      <L 873>
                        var_117 = wp::address(var_cam_mode, var_1);
                        var_120 = wp::load(var_117);
                        var_119 = (var_120 == var_118);
                        if (var_119) {
                            // pos = subtree_com_in[worldid, cam_targetbodyid[camid]]             <L 874>
                            var_121 = wp::address(var_cam_targetbodyid, var_1);
                            var_123 = wp::load(var_121);
                            var_122 = wp::address(var_subtree_com_in, var_0, var_123);
                            var_125 = wp::load(var_122);
                            var_124 = wp::copy(var_125);
                        }
                        var_126 = wp::where(var_119, var_124, var_115);
                        // mat_3 = wp.normalize(cam_xpos_out[worldid, camid] - pos)               <L 876>
                        var_127 = wp::address(var_cam_xpos_out, var_0, var_1);
                        var_129 = wp::load(var_127);
                        var_128 = wp::sub(var_129, var_126);
                        var_130 = wp::normalize(var_128);
                        // mat_1 = wp.normalize(wp.cross(wp.vec3(0.0, 0.0, 1.0), mat_3))          <L 878>
                        var_134 = wp::vec_t<3, wp::float32>(var_131, var_132, var_133);
                        var_135 = wp::cross(var_134, var_130);
                        var_136 = wp::normalize(var_135);
                        // mat_2 = wp.normalize(wp.cross(mat_3, mat_1))                           <L 879>
                        var_137 = wp::cross(var_130, var_136);
                        var_138 = wp::normalize(var_137);
                        // cam_xmat_out[worldid, camid] = wp.mat33(                               <L 881>
                        // mat_1[0], mat_2[0], mat_3[0],                                          <L 882>
                        var_140 = wp::extract(var_136, var_139);
                        var_142 = wp::extract(var_138, var_141);
                        var_144 = wp::extract(var_130, var_143);
                        // mat_1[1], mat_2[1], mat_3[1],                                          <L 883>
                        var_146 = wp::extract(var_136, var_145);
                        var_148 = wp::extract(var_138, var_147);
                        var_150 = wp::extract(var_130, var_149);
                        // mat_1[2], mat_2[2], mat_3[2]                                           <L 884>
                        var_152 = wp::extract(var_136, var_151);
                        var_154 = wp::extract(var_138, var_153);
                        var_156 = wp::extract(var_130, var_155);
                        var_157 = wp::mat_t<3, 3, wp::float32>(var_140, var_142, var_144, var_146, var_148, var_150, var_152, var_154, var_156);
                        // cam_xmat_out[worldid, camid] = wp.mat33(                               <L 881>
                        wp::array_store(var_cam_xmat_out, var_0, var_1, var_157);
                    }
                    var_158 = wp::where(var_90, var_100, var_27);
                    var_159 = wp::where(var_90, var_103, var_30);
                    var_160 = wp::where(var_90, var_106, var_33);
                    if (!var_90) {
                        // bodyid = cam_bodyid[camid]                                             <L 888>
                        var_161 = wp::address(var_cam_bodyid, var_1);
                        var_163 = wp::load(var_161);
                        var_162 = wp::copy(var_163);
                        // xpos = xpos_in[worldid, bodyid]                                        <L 889>
                        var_164 = wp::address(var_xpos_in, var_0, var_162);
                        var_166 = wp::load(var_164);
                        var_165 = wp::copy(var_166);
                        // xquat = xquat_in[worldid, bodyid]                                      <L 890>
                        var_167 = wp::address(var_xquat_in, var_0, var_162);
                        var_169 = wp::load(var_167);
                        var_168 = wp::copy(var_169);
                        // cam_xpos_out[worldid, camid] = xpos + math.rot_vec_quat(cam_pos[cam_pos_id, camid], xquat)       <L 891>
                        var_170 = wp::address(var_cam_pos, var_6, var_1);
                        var_172 = wp::load(var_170);
                        var_171 = rot_vec_quat_0(var_172, var_168);
                        var_173 = wp::add(var_165, var_171);
                        wp::array_store(var_cam_xpos_out, var_0, var_1, var_173);
                        // cam_xmat_out[worldid, camid] = math.quat_to_mat(math.mul_quat(xquat, cam_quat[cam_quat_id, camid]))       <L 892>
                        var_174 = wp::address(var_cam_quat, var_11, var_1);
                        var_176 = wp::load(var_174);
                        var_175 = mul_quat_0(var_168, var_176);
                        var_177 = quat_to_mat_0(var_175);
                        wp::array_store(var_cam_xmat_out, var_0, var_1, var_177);
                    }
                    var_178 = wp::where(var_90, var_158, var_162);
                    var_179 = wp::where(var_90, var_159, var_165);
                    var_180 = wp::where(var_90, var_160, var_168);
                }
                var_181 = wp::where(var_69, var_27, var_178);
                var_182 = wp::where(var_69, var_30, var_179);
                var_183 = wp::where(var_69, var_33, var_180);
            }
            var_184 = wp::where(var_45, var_27, var_181);
            var_185 = wp::where(var_45, var_30, var_182);
            var_186 = wp::where(var_45, var_33, var_183);
        }
        var_187 = wp::where(var_21, var_27, var_184);
        var_188 = wp::where(var_21, var_30, var_185);
        var_189 = wp::where(var_21, var_33, var_186);
    }
}



extern "C" __global__ void _compute_body_inertial_frames_d3bdd81a_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::vec_t<3, wp::float32>> var_body_ipos,
    wp::array_t<wp::quat_t<wp::float32>> var_body_iquat,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_ximat_out)
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
        wp::int32 var_1;
        wp::vec_t<3, wp::float32>* var_2;
        wp::vec_t<3, wp::float32> var_3;
        wp::vec_t<3, wp::float32> var_4;
        wp::quat_t<wp::float32>* var_5;
        wp::quat_t<wp::float32> var_6;
        wp::quat_t<wp::float32> var_7;
        wp::shape_t* var_8;
        const wp::int32 var_9 = 0;
        wp::int32 var_10;
        wp::shape_t var_11;
        wp::int32 var_12;
        wp::vec_t<3, wp::float32>* var_13;
        wp::vec_t<3, wp::float32> var_14;
        wp::vec_t<3, wp::float32> var_15;
        wp::vec_t<3, wp::float32> var_16;
        wp::shape_t* var_17;
        const wp::int32 var_18 = 0;
        wp::int32 var_19;
        wp::shape_t var_20;
        wp::int32 var_21;
        wp::quat_t<wp::float32>* var_22;
        wp::quat_t<wp::float32> var_23;
        wp::quat_t<wp::float32> var_24;
        wp::mat_t<3, 3, wp::float32> var_25;
        //---------
        // forward
        // def _compute_body_inertial_frames(                                                     <L 150>
        // worldid, bodyid = wp.tid()                                                             <L 161>
        builtin_tid2d(var_0, var_1);
        // xpos = xpos_in[worldid, bodyid]                                                        <L 162>
        var_2 = wp::address(var_xpos_in, var_0, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // xquat = xquat_in[worldid, bodyid]                                                      <L 163>
        var_5 = wp::address(var_xquat_in, var_0, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // xipos_out[worldid, bodyid] = xpos + math.rot_vec_quat(body_ipos[worldid % body_ipos.shape[0], bodyid], xquat)       <L 164>
        var_8 = &(var_body_ipos.shape);
        var_11 = wp::load(var_8);
        var_10 = wp::extract(var_11, var_9);
        var_12 = wp::mod(var_0, var_10);
        var_13 = wp::address(var_body_ipos, var_12, var_1);
        var_15 = wp::load(var_13);
        var_14 = rot_vec_quat_0(var_15, var_6);
        var_16 = wp::add(var_3, var_14);
        wp::array_store(var_xipos_out, var_0, var_1, var_16);
        // ximat_out[worldid, bodyid] = math.quat_to_mat(math.mul_quat(xquat, body_iquat[worldid % body_iquat.shape[0], bodyid]))       <L 165>
        var_17 = &(var_body_iquat.shape);
        var_20 = wp::load(var_17);
        var_19 = wp::extract(var_20, var_18);
        var_21 = wp::mod(var_0, var_19);
        var_22 = wp::address(var_body_iquat, var_21, var_1);
        var_24 = wp::load(var_22);
        var_23 = mul_quat_0(var_6, var_24);
        var_25 = quat_to_mat_0(var_23);
        wp::array_store(var_ximat_out, var_0, var_1, var_25);
    }
}



extern "C" __global__ void _light_local_to_global_1966b84b_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_light_mode,
    wp::array_t<wp::int32> var_light_bodyid,
    wp::array_t<wp::int32> var_light_targetbodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_pos,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_dir,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_poscom0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_pos0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_dir0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_xpos_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_light_xdir_out)
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
        wp::int32 var_1;
        wp::shape_t* var_2;
        const wp::int32 var_3 = 0;
        wp::int32 var_4;
        wp::shape_t var_5;
        wp::int32 var_6;
        wp::shape_t* var_7;
        const wp::int32 var_8 = 0;
        wp::int32 var_9;
        wp::shape_t var_10;
        wp::int32 var_11;
        bool var_12;
        wp::int32* var_13;
        const wp::int32 var_14 = 3;
        bool var_15;
        wp::int32 var_16;
        wp::int32* var_17;
        const wp::int32 var_18 = 4;
        bool var_19;
        wp::int32 var_20;
        bool var_21;
        wp::int32* var_22;
        const wp::int32 var_23 = 0;
        bool var_24;
        wp::int32 var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::vec_t<3, wp::float32>* var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::vec_t<3, wp::float32> var_31;
        wp::quat_t<wp::float32>* var_32;
        wp::quat_t<wp::float32> var_33;
        wp::quat_t<wp::float32> var_34;
        wp::vec_t<3, wp::float32>* var_35;
        wp::vec_t<3, wp::float32> var_36;
        wp::vec_t<3, wp::float32> var_37;
        wp::vec_t<3, wp::float32> var_38;
        wp::vec_t<3, wp::float32>* var_39;
        wp::vec_t<3, wp::float32> var_40;
        wp::vec_t<3, wp::float32> var_41;
        wp::int32* var_42;
        const wp::int32 var_43 = 1;
        bool var_44;
        wp::int32 var_45;
        wp::shape_t* var_46;
        const wp::int32 var_47 = 0;
        wp::int32 var_48;
        wp::shape_t var_49;
        wp::int32 var_50;
        wp::vec_t<3, wp::float32>* var_51;
        wp::vec_t<3, wp::float32> var_52;
        wp::int32* var_53;
        wp::vec_t<3, wp::float32>* var_54;
        wp::int32 var_55;
        wp::vec_t<3, wp::float32> var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::shape_t* var_58;
        const wp::int32 var_59 = 0;
        wp::int32 var_60;
        wp::shape_t var_61;
        wp::int32 var_62;
        wp::vec_t<3, wp::float32>* var_63;
        wp::vec_t<3, wp::float32> var_64;
        wp::vec_t<3, wp::float32> var_65;
        wp::int32* var_66;
        const wp::int32 var_67 = 2;
        bool var_68;
        wp::int32 var_69;
        wp::shape_t* var_70;
        const wp::int32 var_71 = 0;
        wp::int32 var_72;
        wp::shape_t var_73;
        wp::int32 var_74;
        wp::vec_t<3, wp::float32>* var_75;
        wp::vec_t<3, wp::float32> var_76;
        wp::int32* var_77;
        wp::vec_t<3, wp::float32>* var_78;
        wp::int32 var_79;
        wp::shape_t* var_80;
        const wp::int32 var_81 = 0;
        wp::int32 var_82;
        wp::shape_t var_83;
        wp::int32 var_84;
        wp::vec_t<3, wp::float32>* var_85;
        wp::vec_t<3, wp::float32> var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::vec_t<3, wp::float32> var_88;
        bool var_89;
        wp::int32* var_90;
        const wp::int32 var_91 = 3;
        bool var_92;
        wp::int32 var_93;
        wp::int32* var_94;
        const wp::int32 var_95 = 4;
        bool var_96;
        wp::int32 var_97;
        wp::int32* var_98;
        wp::int32 var_99;
        wp::int32 var_100;
        wp::vec_t<3, wp::float32>* var_101;
        wp::vec_t<3, wp::float32> var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::quat_t<wp::float32>* var_104;
        wp::quat_t<wp::float32> var_105;
        wp::quat_t<wp::float32> var_106;
        wp::vec_t<3, wp::float32>* var_107;
        wp::vec_t<3, wp::float32> var_108;
        wp::vec_t<3, wp::float32> var_109;
        wp::vec_t<3, wp::float32> var_110;
        wp::int32* var_111;
        wp::vec_t<3, wp::float32>* var_112;
        wp::int32 var_113;
        wp::vec_t<3, wp::float32> var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::int32* var_116;
        const wp::int32 var_117 = 4;
        bool var_118;
        wp::int32 var_119;
        wp::int32* var_120;
        wp::vec_t<3, wp::float32>* var_121;
        wp::int32 var_122;
        wp::vec_t<3, wp::float32> var_123;
        wp::vec_t<3, wp::float32> var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32>* var_126;
        wp::vec_t<3, wp::float32> var_127;
        wp::vec_t<3, wp::float32> var_128;
        wp::int32 var_129;
        wp::vec_t<3, wp::float32> var_130;
        wp::quat_t<wp::float32> var_131;
        wp::int32* var_132;
        wp::int32 var_133;
        wp::int32 var_134;
        wp::vec_t<3, wp::float32>* var_135;
        wp::vec_t<3, wp::float32> var_136;
        wp::vec_t<3, wp::float32> var_137;
        wp::quat_t<wp::float32>* var_138;
        wp::quat_t<wp::float32> var_139;
        wp::quat_t<wp::float32> var_140;
        wp::vec_t<3, wp::float32>* var_141;
        wp::vec_t<3, wp::float32> var_142;
        wp::vec_t<3, wp::float32> var_143;
        wp::vec_t<3, wp::float32> var_144;
        wp::vec_t<3, wp::float32>* var_145;
        wp::vec_t<3, wp::float32> var_146;
        wp::vec_t<3, wp::float32> var_147;
        wp::int32 var_148;
        wp::vec_t<3, wp::float32> var_149;
        wp::quat_t<wp::float32> var_150;
        wp::int32 var_151;
        wp::vec_t<3, wp::float32> var_152;
        wp::quat_t<wp::float32> var_153;
        wp::int32 var_154;
        wp::vec_t<3, wp::float32> var_155;
        wp::quat_t<wp::float32> var_156;
        wp::int32 var_157;
        wp::vec_t<3, wp::float32> var_158;
        wp::quat_t<wp::float32> var_159;
        wp::vec_t<3, wp::float32>* var_160;
        wp::vec_t<3, wp::float32> var_161;
        wp::vec_t<3, wp::float32> var_162;
        //---------
        // forward
        // def _light_local_to_global(                                                            <L 896>
        // worldid, lightid = wp.tid()                                                            <L 914>
        builtin_tid2d(var_0, var_1);
        // light_pos_id = worldid % light_pos.shape[0]                                            <L 915>
        var_2 = &(var_light_pos.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_0, var_4);
        // light_dir_id = worldid % light_dir.shape[0]                                            <L 916>
        var_7 = &(var_light_dir.shape);
        var_10 = wp::load(var_7);
        var_9 = wp::extract(var_10, var_8);
        var_11 = wp::mod(var_0, var_9);
        // is_target_light = (light_mode[lightid] == CamLightType.TARGETBODY) or (light_mode[lightid] == CamLightType.TARGETBODYCOM)       <L 917>
        var_13 = wp::address(var_light_mode, var_1);
        var_16 = wp::load(var_13);
        var_15 = (var_16 == var_14);
        var_12 = var_15;
        if (!var_12) {
            var_17 = wp::address(var_light_mode, var_1);
            var_20 = wp::load(var_17);
            var_19 = (var_20 == var_18);
            var_12 = var_12 || var_19;
        }
        // invalid_target = is_target_light and (light_targetbodyid[lightid] < 0)                 <L 918>
        var_21 = var_12;
        if (var_21) {
            var_22 = wp::address(var_light_targetbodyid, var_1);
            var_25 = wp::load(var_22);
            var_24 = (var_25 < var_23);
            var_21 = var_21 && var_24;
        }
        // if invalid_target:                                                                     <L 919>
        if (var_21) {
            // bodyid = light_bodyid[lightid]                                                     <L 920>
            var_26 = wp::address(var_light_bodyid, var_1);
            var_28 = wp::load(var_26);
            var_27 = wp::copy(var_28);
            // xpos = xpos_in[worldid, bodyid]                                                    <L 921>
            var_29 = wp::address(var_xpos_in, var_0, var_27);
            var_31 = wp::load(var_29);
            var_30 = wp::copy(var_31);
            // xquat = xquat_in[worldid, bodyid]                                                  <L 922>
            var_32 = wp::address(var_xquat_in, var_0, var_27);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // light_xpos_out[worldid, lightid] = xpos + math.rot_vec_quat(light_pos[light_pos_id, lightid], xquat)       <L 923>
            var_35 = wp::address(var_light_pos, var_6, var_1);
            var_37 = wp::load(var_35);
            var_36 = rot_vec_quat_0(var_37, var_33);
            var_38 = wp::add(var_30, var_36);
            wp::array_store(var_light_xpos_out, var_0, var_1, var_38);
            // light_xdir_out[worldid, lightid] = math.rot_vec_quat(light_dir[light_dir_id, lightid], xquat)       <L 924>
            var_39 = wp::address(var_light_dir, var_11, var_1);
            var_41 = wp::load(var_39);
            var_40 = rot_vec_quat_0(var_41, var_33);
            wp::array_store(var_light_xdir_out, var_0, var_1, var_40);
            // return                                                                             <L 925>
            continue;
        }
        if (!var_21) {
            // elif light_mode[lightid] == CamLightType.TRACK:                                    <L 926>
            var_42 = wp::address(var_light_mode, var_1);
            var_45 = wp::load(var_42);
            var_44 = (var_45 == var_43);
            if (var_44) {
                // light_xdir_out[worldid, lightid] = light_dir0[worldid % light_dir0.shape[0], lightid]       <L 927>
                var_46 = &(var_light_dir0.shape);
                var_49 = wp::load(var_46);
                var_48 = wp::extract(var_49, var_47);
                var_50 = wp::mod(var_0, var_48);
                var_51 = wp::address(var_light_dir0, var_50, var_1);
                var_52 = wp::load(var_51);
                wp::array_store(var_light_xdir_out, var_0, var_1, var_52);
                // body_xpos = xpos_in[worldid, light_bodyid[lightid]]                            <L 928>
                var_53 = wp::address(var_light_bodyid, var_1);
                var_55 = wp::load(var_53);
                var_54 = wp::address(var_xpos_in, var_0, var_55);
                var_57 = wp::load(var_54);
                var_56 = wp::copy(var_57);
                // light_xpos_out[worldid, lightid] = body_xpos + light_pos0[worldid % light_pos0.shape[0], lightid]       <L 929>
                var_58 = &(var_light_pos0.shape);
                var_61 = wp::load(var_58);
                var_60 = wp::extract(var_61, var_59);
                var_62 = wp::mod(var_0, var_60);
                var_63 = wp::address(var_light_pos0, var_62, var_1);
                var_65 = wp::load(var_63);
                var_64 = wp::add(var_56, var_65);
                wp::array_store(var_light_xpos_out, var_0, var_1, var_64);
            }
            if (!var_44) {
                // elif light_mode[lightid] == CamLightType.TRACKCOM:                             <L 930>
                var_66 = wp::address(var_light_mode, var_1);
                var_69 = wp::load(var_66);
                var_68 = (var_69 == var_67);
                if (var_68) {
                    // light_xdir_out[worldid, lightid] = light_dir0[worldid % light_dir0.shape[0], lightid]       <L 931>
                    var_70 = &(var_light_dir0.shape);
                    var_73 = wp::load(var_70);
                    var_72 = wp::extract(var_73, var_71);
                    var_74 = wp::mod(var_0, var_72);
                    var_75 = wp::address(var_light_dir0, var_74, var_1);
                    var_76 = wp::load(var_75);
                    wp::array_store(var_light_xdir_out, var_0, var_1, var_76);
                    // light_xpos_out[worldid, lightid] = (                                       <L 932>
                    // subtree_com_in[worldid, light_bodyid[lightid]] + light_poscom0[worldid % light_poscom0.shape[0], lightid]       <L 933>
                    var_77 = wp::address(var_light_bodyid, var_1);
                    var_79 = wp::load(var_77);
                    var_78 = wp::address(var_subtree_com_in, var_0, var_79);
                    var_80 = &(var_light_poscom0.shape);
                    var_83 = wp::load(var_80);
                    var_82 = wp::extract(var_83, var_81);
                    var_84 = wp::mod(var_0, var_82);
                    var_85 = wp::address(var_light_poscom0, var_84, var_1);
                    var_87 = wp::load(var_78);
                    var_88 = wp::load(var_85);
                    var_86 = wp::add(var_87, var_88);
                    // light_xpos_out[worldid, lightid] = (                                       <L 932>
                    wp::array_store(var_light_xpos_out, var_0, var_1, var_86);
                }
                if (!var_68) {
                    // elif light_mode[lightid] == CamLightType.TARGETBODY or light_mode[lightid] == CamLightType.TARGETBODYCOM:       <L 935>
                    var_90 = wp::address(var_light_mode, var_1);
                    var_93 = wp::load(var_90);
                    var_92 = (var_93 == var_91);
                    var_89 = var_92;
                    if (!var_89) {
                        var_94 = wp::address(var_light_mode, var_1);
                        var_97 = wp::load(var_94);
                        var_96 = (var_97 == var_95);
                        var_89 = var_89 || var_96;
                    }
                    if (var_89) {
                        // bodyid = light_bodyid[lightid]                                         <L 936>
                        var_98 = wp::address(var_light_bodyid, var_1);
                        var_100 = wp::load(var_98);
                        var_99 = wp::copy(var_100);
                        // xpos = xpos_in[worldid, bodyid]                                        <L 937>
                        var_101 = wp::address(var_xpos_in, var_0, var_99);
                        var_103 = wp::load(var_101);
                        var_102 = wp::copy(var_103);
                        // xquat = xquat_in[worldid, bodyid]                                      <L 938>
                        var_104 = wp::address(var_xquat_in, var_0, var_99);
                        var_106 = wp::load(var_104);
                        var_105 = wp::copy(var_106);
                        // light_xpos_out[worldid, lightid] = xpos + math.rot_vec_quat(light_pos[light_pos_id, lightid], xquat)       <L 939>
                        var_107 = wp::address(var_light_pos, var_6, var_1);
                        var_109 = wp::load(var_107);
                        var_108 = rot_vec_quat_0(var_109, var_105);
                        var_110 = wp::add(var_102, var_108);
                        wp::array_store(var_light_xpos_out, var_0, var_1, var_110);
                        // pos = xpos_in[worldid, light_targetbodyid[lightid]]                    <L 940>
                        var_111 = wp::address(var_light_targetbodyid, var_1);
                        var_113 = wp::load(var_111);
                        var_112 = wp::address(var_xpos_in, var_0, var_113);
                        var_115 = wp::load(var_112);
                        var_114 = wp::copy(var_115);
                        // if light_mode[lightid] == CamLightType.TARGETBODYCOM:                  <L 941>
                        var_116 = wp::address(var_light_mode, var_1);
                        var_119 = wp::load(var_116);
                        var_118 = (var_119 == var_117);
                        if (var_118) {
                            // pos = subtree_com_in[worldid, light_targetbodyid[lightid]]         <L 942>
                            var_120 = wp::address(var_light_targetbodyid, var_1);
                            var_122 = wp::load(var_120);
                            var_121 = wp::address(var_subtree_com_in, var_0, var_122);
                            var_124 = wp::load(var_121);
                            var_123 = wp::copy(var_124);
                        }
                        var_125 = wp::where(var_118, var_123, var_114);
                        // light_xdir_out[worldid, lightid] = pos - light_xpos_out[worldid, lightid]       <L 943>
                        var_126 = wp::address(var_light_xpos_out, var_0, var_1);
                        var_128 = wp::load(var_126);
                        var_127 = wp::sub(var_125, var_128);
                        wp::array_store(var_light_xdir_out, var_0, var_1, var_127);
                    }
                    var_129 = wp::where(var_89, var_99, var_27);
                    var_130 = wp::where(var_89, var_102, var_30);
                    var_131 = wp::where(var_89, var_105, var_33);
                    if (!var_89) {
                        // bodyid = light_bodyid[lightid]                                         <L 945>
                        var_132 = wp::address(var_light_bodyid, var_1);
                        var_134 = wp::load(var_132);
                        var_133 = wp::copy(var_134);
                        // xpos = xpos_in[worldid, bodyid]                                        <L 946>
                        var_135 = wp::address(var_xpos_in, var_0, var_133);
                        var_137 = wp::load(var_135);
                        var_136 = wp::copy(var_137);
                        // xquat = xquat_in[worldid, bodyid]                                      <L 947>
                        var_138 = wp::address(var_xquat_in, var_0, var_133);
                        var_140 = wp::load(var_138);
                        var_139 = wp::copy(var_140);
                        // light_xpos_out[worldid, lightid] = xpos + math.rot_vec_quat(light_pos[light_pos_id, lightid], xquat)       <L 948>
                        var_141 = wp::address(var_light_pos, var_6, var_1);
                        var_143 = wp::load(var_141);
                        var_142 = rot_vec_quat_0(var_143, var_139);
                        var_144 = wp::add(var_136, var_142);
                        wp::array_store(var_light_xpos_out, var_0, var_1, var_144);
                        // light_xdir_out[worldid, lightid] = math.rot_vec_quat(light_dir[light_dir_id, lightid], xquat)       <L 949>
                        var_145 = wp::address(var_light_dir, var_11, var_1);
                        var_147 = wp::load(var_145);
                        var_146 = rot_vec_quat_0(var_147, var_139);
                        wp::array_store(var_light_xdir_out, var_0, var_1, var_146);
                    }
                    var_148 = wp::where(var_89, var_129, var_133);
                    var_149 = wp::where(var_89, var_130, var_136);
                    var_150 = wp::where(var_89, var_131, var_139);
                }
                var_151 = wp::where(var_68, var_27, var_148);
                var_152 = wp::where(var_68, var_30, var_149);
                var_153 = wp::where(var_68, var_33, var_150);
            }
            var_154 = wp::where(var_44, var_27, var_151);
            var_155 = wp::where(var_44, var_30, var_152);
            var_156 = wp::where(var_44, var_33, var_153);
        }
        var_157 = wp::where(var_21, var_27, var_154);
        var_158 = wp::where(var_21, var_30, var_155);
        var_159 = wp::where(var_21, var_33, var_156);
        // light_xdir_out[worldid, lightid] = wp.normalize(light_xdir_out[worldid, lightid])       <L 951>
        var_160 = wp::address(var_light_xdir_out, var_0, var_1);
        var_162 = wp::load(var_160);
        var_161 = wp::normalize(var_162);
        wp::array_store(var_light_xdir_out, var_0, var_1, var_161);
    }
}



extern "C" __global__ void _cfrc_backward_72237d38_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_int_in,
    wp::array_t<wp::int32> var_body_tree_,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_int_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::slice_t var_10;
        const wp::int32 var_11 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_12;
        wp::vec_t<6, wp::float32>* var_13;
        wp::vec_t<6, wp::float32> var_14;
        wp::vec_t<6, wp::float32> var_15;
        //---------
        // forward
        // def _cfrc_backward(                                                                    <L 1430>
        // worldid, nodeid = wp.tid()                                                             <L 1440>
        builtin_tid2d(var_0, var_1);
        // bodyid = body_tree_[nodeid]                                                            <L 1441>
        var_2 = wp::address(var_body_tree_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // pid = body_parentid[bodyid]                                                            <L 1442>
        var_5 = wp::address(var_body_parentid, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if bodyid != 0:                                                                        <L 1443>
        var_9 = (var_3 != var_8);
        if (var_9) {
            // wp.atomic_add(cfrc_int_out[worldid], pid, cfrc_int_in[worldid, bodyid])            <L 1444>
            var_10 = wp::slice_t(var_0, var_0, var_11);
            var_12 = wp::view(var_cfrc_int_out, var_10);
            var_13 = wp::address(var_cfrc_int_in, var_0, var_3);
            var_15 = wp::load(var_13);
            var_14 = wp::atomic_add(var_12, var_6, var_15);
        }
    }
}



extern "C" __global__ void _compute_body_matrices_27760750_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_out)
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
        wp::int32 var_1;
        wp::quat_t<wp::float32>* var_2;
        wp::mat_t<3, 3, wp::float32> var_3;
        wp::quat_t<wp::float32> var_4;
        //---------
        // forward
        // def _compute_body_matrices(                                                            <L 169>
        // worldid, bodyid = wp.tid()                                                             <L 175>
        builtin_tid2d(var_0, var_1);
        // xmat_out[worldid, bodyid] = math.quat_to_mat(xquat_in[worldid, bodyid])                <L 176>
        var_2 = wp::address(var_xquat_in, var_0, var_1);
        var_4 = wp::load(var_2);
        var_3 = quat_to_mat_0(var_4);
        wp::array_store(var_xmat_out, var_0, var_1, var_3);
    }
}



extern "C" __global__ void _flex_vertices_68b379d4_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nflex,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::vec_t<3, wp::int32>> var_flex_cellnum,
    wp::array_t<wp::int32> var_flex_nodeadr,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_vertnum,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_vert,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_vert0,
    wp::array_t<bool> var_flex_centered,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_out)
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
        wp::int32 var_1;
        wp::range_t var_2;
        wp::int32 var_3;
        wp::int32* var_4;
        wp::int32 var_5;
        wp::int32 var_6;
        bool var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::int32* var_10;
        bool var_11;
        wp::int32 var_12;
        wp::int32* var_13;
        const wp::int32 var_14 = 0;
        bool var_15;
        wp::int32 var_16;
        wp::vec_t<3, wp::float32>* var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::vec_t<3, wp::int32>* var_20;
        wp::vec_t<3, wp::int32> var_21;
        wp::vec_t<3, wp::int32> var_22;
        const wp::int32 var_23 = 0;
        wp::int32 var_24;
        const wp::int32 var_25 = 1;
        wp::int32 var_26;
        const wp::int32 var_27 = 2;
        wp::int32 var_28;
        const wp::int32 var_29 = 0;
        wp::float32 var_30;
        wp::float32 var_31;
        wp::float32 var_32;
        wp::int32 var_33;
        const wp::int32 var_34 = 1;
        wp::int32 var_35;
        wp::int32 var_36;
        const wp::int32 var_37 = 0;
        wp::int32 var_38;
        const wp::int32 var_39 = 1;
        wp::float32 var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::int32 var_43;
        const wp::int32 var_44 = 1;
        wp::int32 var_45;
        wp::int32 var_46;
        const wp::int32 var_47 = 0;
        wp::int32 var_48;
        const wp::int32 var_49 = 2;
        wp::float32 var_50;
        wp::float32 var_51;
        wp::float32 var_52;
        wp::int32 var_53;
        const wp::int32 var_54 = 1;
        wp::int32 var_55;
        wp::int32 var_56;
        const wp::int32 var_57 = 0;
        wp::int32 var_58;
        const wp::int32 var_59 = 0;
        wp::float32 var_60;
        wp::float32 var_61;
        wp::float32 var_62;
        wp::float32 var_63;
        wp::float32 var_64;
        const wp::float32 var_65 = 0.0;
        const wp::float32 var_66 = 1.0;
        wp::float32 var_67;
        const wp::int32 var_68 = 1;
        wp::float32 var_69;
        wp::float32 var_70;
        wp::float32 var_71;
        wp::float32 var_72;
        wp::float32 var_73;
        const wp::float32 var_74 = 0.0;
        const wp::float32 var_75 = 1.0;
        wp::float32 var_76;
        const wp::int32 var_77 = 2;
        wp::float32 var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::float32 var_81;
        wp::float32 var_82;
        const wp::float32 var_83 = 0.0;
        const wp::float32 var_84 = 1.0;
        wp::float32 var_85;
        wp::vec_t<3, wp::float32> var_86;
        const wp::int32 var_87 = 1;
        wp::int32 var_88;
        const wp::int32 var_89 = 1;
        wp::int32 var_90;
        wp::int32* var_91;
        wp::int32 var_92;
        wp::int32 var_93;
        const wp::float32 var_94 = 0.0;
        const wp::float32 var_95 = 0.0;
        const wp::float32 var_96 = 0.0;
        wp::vec_t<3, wp::float32> var_97;
        const wp::int32 var_98 = 0;
        const wp::int32 var_99 = 0;
        const wp::int32 var_100 = 0;
        const wp::int32 var_101 = 4;
        wp::int32 var_102;
        const wp::int32 var_103 = 2;
        wp::int32 var_104;
        wp::int32 var_105;
        wp::int32 var_106;
        wp::float32 var_107;
        wp::int32 var_108;
        wp::int32 var_109;
        wp::int32 var_110;
        wp::int32 var_111;
        wp::int32 var_112;
        wp::int32 var_113;
        wp::int32 var_114;
        wp::int32 var_115;
        wp::int32 var_116;
        wp::vec_t<3, wp::float32>* var_117;
        wp::vec_t<3, wp::float32> var_118;
        wp::vec_t<3, wp::float32> var_119;
        wp::vec_t<3, wp::float32> var_120;
        const wp::int32 var_121 = 1;
        const wp::int32 var_122 = 4;
        wp::int32 var_123;
        const wp::int32 var_124 = 2;
        wp::int32 var_125;
        wp::int32 var_126;
        wp::int32 var_127;
        wp::float32 var_128;
        wp::int32 var_129;
        wp::int32 var_130;
        wp::int32 var_131;
        wp::int32 var_132;
        wp::int32 var_133;
        wp::int32 var_134;
        wp::int32 var_135;
        wp::int32 var_136;
        wp::int32 var_137;
        wp::vec_t<3, wp::float32>* var_138;
        wp::vec_t<3, wp::float32> var_139;
        wp::vec_t<3, wp::float32> var_140;
        wp::vec_t<3, wp::float32> var_141;
        const wp::int32 var_142 = 1;
        const wp::int32 var_143 = 0;
        const wp::int32 var_144 = 4;
        wp::int32 var_145;
        const wp::int32 var_146 = 2;
        wp::int32 var_147;
        wp::int32 var_148;
        wp::int32 var_149;
        wp::float32 var_150;
        wp::int32 var_151;
        wp::int32 var_152;
        wp::int32 var_153;
        wp::int32 var_154;
        wp::int32 var_155;
        wp::int32 var_156;
        wp::int32 var_157;
        wp::int32 var_158;
        wp::int32 var_159;
        wp::vec_t<3, wp::float32>* var_160;
        wp::vec_t<3, wp::float32> var_161;
        wp::vec_t<3, wp::float32> var_162;
        wp::vec_t<3, wp::float32> var_163;
        const wp::int32 var_164 = 1;
        const wp::int32 var_165 = 4;
        wp::int32 var_166;
        const wp::int32 var_167 = 2;
        wp::int32 var_168;
        wp::int32 var_169;
        wp::int32 var_170;
        wp::float32 var_171;
        wp::int32 var_172;
        wp::int32 var_173;
        wp::int32 var_174;
        wp::int32 var_175;
        wp::int32 var_176;
        wp::int32 var_177;
        wp::int32 var_178;
        wp::int32 var_179;
        wp::int32 var_180;
        wp::vec_t<3, wp::float32>* var_181;
        wp::vec_t<3, wp::float32> var_182;
        wp::vec_t<3, wp::float32> var_183;
        wp::vec_t<3, wp::float32> var_184;
        const wp::int32 var_185 = 1;
        const wp::int32 var_186 = 0;
        const wp::int32 var_187 = 0;
        const wp::int32 var_188 = 4;
        wp::int32 var_189;
        const wp::int32 var_190 = 2;
        wp::int32 var_191;
        wp::int32 var_192;
        wp::int32 var_193;
        wp::float32 var_194;
        wp::int32 var_195;
        wp::int32 var_196;
        wp::int32 var_197;
        wp::int32 var_198;
        wp::int32 var_199;
        wp::int32 var_200;
        wp::int32 var_201;
        wp::int32 var_202;
        wp::int32 var_203;
        wp::vec_t<3, wp::float32>* var_204;
        wp::vec_t<3, wp::float32> var_205;
        wp::vec_t<3, wp::float32> var_206;
        wp::vec_t<3, wp::float32> var_207;
        const wp::int32 var_208 = 1;
        const wp::int32 var_209 = 4;
        wp::int32 var_210;
        const wp::int32 var_211 = 2;
        wp::int32 var_212;
        wp::int32 var_213;
        wp::int32 var_214;
        wp::float32 var_215;
        wp::int32 var_216;
        wp::int32 var_217;
        wp::int32 var_218;
        wp::int32 var_219;
        wp::int32 var_220;
        wp::int32 var_221;
        wp::int32 var_222;
        wp::int32 var_223;
        wp::int32 var_224;
        wp::vec_t<3, wp::float32>* var_225;
        wp::vec_t<3, wp::float32> var_226;
        wp::vec_t<3, wp::float32> var_227;
        wp::vec_t<3, wp::float32> var_228;
        const wp::int32 var_229 = 1;
        const wp::int32 var_230 = 0;
        const wp::int32 var_231 = 4;
        wp::int32 var_232;
        const wp::int32 var_233 = 2;
        wp::int32 var_234;
        wp::int32 var_235;
        wp::int32 var_236;
        wp::float32 var_237;
        wp::int32 var_238;
        wp::int32 var_239;
        wp::int32 var_240;
        wp::int32 var_241;
        wp::int32 var_242;
        wp::int32 var_243;
        wp::int32 var_244;
        wp::int32 var_245;
        wp::int32 var_246;
        wp::vec_t<3, wp::float32>* var_247;
        wp::vec_t<3, wp::float32> var_248;
        wp::vec_t<3, wp::float32> var_249;
        wp::vec_t<3, wp::float32> var_250;
        const wp::int32 var_251 = 1;
        const wp::int32 var_252 = 4;
        wp::int32 var_253;
        const wp::int32 var_254 = 2;
        wp::int32 var_255;
        wp::int32 var_256;
        wp::int32 var_257;
        wp::float32 var_258;
        wp::int32 var_259;
        wp::int32 var_260;
        wp::int32 var_261;
        wp::int32 var_262;
        wp::int32 var_263;
        wp::int32 var_264;
        wp::int32 var_265;
        wp::int32 var_266;
        wp::int32 var_267;
        wp::vec_t<3, wp::float32>* var_268;
        wp::vec_t<3, wp::float32> var_269;
        wp::vec_t<3, wp::float32> var_270;
        wp::vec_t<3, wp::float32> var_271;
        wp::int32* var_272;
        wp::int32 var_273;
        wp::int32 var_274;
        wp::vec_t<3, wp::float32>* var_275;
        wp::vec_t<3, wp::float32> var_276;
        wp::vec_t<3, wp::float32> var_277;
        bool* var_278;
        bool var_279;
        bool var_280;
        bool var_281;
        wp::mat_t<3, 3, wp::float32>* var_282;
        wp::mat_t<3, 3, wp::float32> var_283;
        wp::mat_t<3, 3, wp::float32> var_284;
        wp::vec_t<3, wp::float32>* var_285;
        wp::vec_t<3, wp::float32> var_286;
        wp::vec_t<3, wp::float32> var_287;
        wp::vec_t<3, wp::float32> var_288;
        wp::vec_t<3, wp::float32> var_289;
        bool var_290;
        //---------
        // forward
        // def _flex_vertices(                                                                    <L 231>
        // worldid, vertid = wp.tid()                                                             <L 250>
        builtin_tid2d(var_0, var_1);
        // for f in range(nflex):                                                                 <L 252>
        var_2 = wp::range(var_nflex);
        start_for_0:;
            if (iter_cmp(var_2) == 0) goto end_for_0;
            var_3 = wp::iter_next(var_2);
            // locid = vertid - flex_vertadr[f]                                                   <L 253>
            var_4 = wp::address(var_flex_vertadr, var_3);
            var_6 = wp::load(var_4);
            var_5 = wp::sub(var_1, var_6);
            // if locid >= 0 and locid < flex_vertnum[f]:                                         <L 254>
            var_9 = (var_5 >= var_8);
            var_7 = var_9;
            if (var_7) {
                var_10 = wp::address(var_flex_vertnum, var_3);
                var_12 = wp::load(var_10);
                var_11 = (var_5 < var_12);
                var_7 = var_7 && var_11;
            }
            if (var_7) {
                // break                                                                          <L 255>
                goto end_for_0;
            }
            goto start_for_0;
        end_for_0:;
        // if flex_interp[f] != 0:                                                                <L 257>
        var_13 = wp::address(var_flex_interp, var_3);
        var_16 = wp::load(var_13);
        var_15 = (var_16 != var_14);
        if (var_15) {
            // coord = flex_vert0[vertid]                                                         <L 259>
            var_17 = wp::address(var_flex_vert0, var_1);
            var_19 = wp::load(var_17);
            var_18 = wp::copy(var_19);
            // cn = flex_cellnum[f]                                                               <L 260>
            var_20 = wp::address(var_flex_cellnum, var_3);
            var_22 = wp::load(var_20);
            var_21 = wp::copy(var_22);
            // cx = cn[0]                                                                         <L 261>
            var_24 = wp::extract(var_21, var_23);
            // cy = cn[1]                                                                         <L 262>
            var_26 = wp::extract(var_21, var_25);
            // cz = cn[2]                                                                         <L 263>
            var_28 = wp::extract(var_21, var_27);
            // ci = wp.min(int(coord[0] * float(cx)), cx - 1)                                     <L 266>
            var_30 = wp::extract(var_18, var_29);
            var_31 = wp::float(var_24);
            var_32 = wp::mul(var_30, var_31);
            var_33 = wp::int(var_32);
            var_35 = wp::sub(var_24, var_34);
            var_36 = wp::min(var_33, var_35);
            // ci = wp.max(ci, 0)                                                                 <L 267>
            var_38 = wp::max(var_36, var_37);
            // cj = wp.min(int(coord[1] * float(cy)), cy - 1)                                     <L 268>
            var_40 = wp::extract(var_18, var_39);
            var_41 = wp::float(var_26);
            var_42 = wp::mul(var_40, var_41);
            var_43 = wp::int(var_42);
            var_45 = wp::sub(var_26, var_44);
            var_46 = wp::min(var_43, var_45);
            // cj = wp.max(cj, 0)                                                                 <L 269>
            var_48 = wp::max(var_46, var_47);
            // ck = wp.min(int(coord[2] * float(cz)), cz - 1)                                     <L 270>
            var_50 = wp::extract(var_18, var_49);
            var_51 = wp::float(var_28);
            var_52 = wp::mul(var_50, var_51);
            var_53 = wp::int(var_52);
            var_55 = wp::sub(var_28, var_54);
            var_56 = wp::min(var_53, var_55);
            // ck = wp.max(ck, 0)                                                                 <L 271>
            var_58 = wp::max(var_56, var_57);
            // local_x = wp.clamp(coord[0] * float(cx) - float(ci), 0.0, 1.0)                     <L 274>
            var_60 = wp::extract(var_18, var_59);
            var_61 = wp::float(var_24);
            var_62 = wp::mul(var_60, var_61);
            var_63 = wp::float(var_38);
            var_64 = wp::sub(var_62, var_63);
            var_67 = wp::clamp(var_64, var_65, var_66);
            // local_y = wp.clamp(coord[1] * float(cy) - float(cj), 0.0, 1.0)                     <L 275>
            var_69 = wp::extract(var_18, var_68);
            var_70 = wp::float(var_26);
            var_71 = wp::mul(var_69, var_70);
            var_72 = wp::float(var_48);
            var_73 = wp::sub(var_71, var_72);
            var_76 = wp::clamp(var_73, var_74, var_75);
            // local_z = wp.clamp(coord[2] * float(cz) - float(ck), 0.0, 1.0)                     <L 276>
            var_78 = wp::extract(var_18, var_77);
            var_79 = wp::float(var_28);
            var_80 = wp::mul(var_78, var_79);
            var_81 = wp::float(var_58);
            var_82 = wp::sub(var_80, var_81);
            var_85 = wp::clamp(var_82, var_83, var_84);
            // local = wp.vec3(local_x, local_y, local_z)                                         <L 277>
            var_86 = wp::vec_t<3, wp::float32>(var_67, var_76, var_85);
            // ny_g = cy + 1                                                                      <L 280>
            var_88 = wp::add(var_26, var_87);
            // nz_g = cz + 1                                                                      <L 281>
            var_90 = wp::add(var_28, var_89);
            // nstart = flex_nodeadr[f]                                                           <L 282>
            var_91 = wp::address(var_flex_nodeadr, var_3);
            var_93 = wp::load(var_91);
            var_92 = wp::copy(var_93);
            // result = wp.vec3(0.0, 0.0, 0.0)                                                    <L 285>
            var_97 = wp::vec_t<3, wp::float32>(var_94, var_95, var_96);
            // for li in range(2):                                                                <L 286>
            // for lj in range(2):                                                                <L 287>
            // for lk in range(2):                                                                <L 288>
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_102 = wp::mul(var_98, var_101);
            var_104 = wp::mul(var_99, var_103);
            var_105 = wp::add(var_102, var_104);
            var_106 = wp::add(var_105, var_100);
            var_107 = eval_basis_trilinear_0(var_86, var_106);
            // gi = ci + li                                                                       <L 290>
            var_108 = wp::add(var_38, var_98);
            // gj = cj + lj                                                                       <L 291>
            var_109 = wp::add(var_48, var_99);
            // gk = ck + lk                                                                       <L 292>
            var_110 = wp::add(var_58, var_100);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_111 = wp::mul(var_108, var_88);
            var_112 = wp::mul(var_111, var_90);
            var_113 = wp::mul(var_109, var_90);
            var_114 = wp::add(var_112, var_113);
            var_115 = wp::add(var_114, var_110);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_116 = wp::add(var_92, var_115);
            var_117 = wp::address(var_flexnode_xpos_in, var_0, var_116);
            var_119 = wp::load(var_117);
            var_118 = wp::mul(var_107, var_119);
            var_120 = wp::add(var_97, var_118);
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_123 = wp::mul(var_98, var_122);
            var_125 = wp::mul(var_99, var_124);
            var_126 = wp::add(var_123, var_125);
            var_127 = wp::add(var_126, var_121);
            var_128 = eval_basis_trilinear_0(var_86, var_127);
            // gi = ci + li                                                                       <L 290>
            var_129 = wp::add(var_38, var_98);
            // gj = cj + lj                                                                       <L 291>
            var_130 = wp::add(var_48, var_99);
            // gk = ck + lk                                                                       <L 292>
            var_131 = wp::add(var_58, var_121);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_132 = wp::mul(var_129, var_88);
            var_133 = wp::mul(var_132, var_90);
            var_134 = wp::mul(var_130, var_90);
            var_135 = wp::add(var_133, var_134);
            var_136 = wp::add(var_135, var_131);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_137 = wp::add(var_92, var_136);
            var_138 = wp::address(var_flexnode_xpos_in, var_0, var_137);
            var_140 = wp::load(var_138);
            var_139 = wp::mul(var_128, var_140);
            var_141 = wp::add(var_120, var_139);
            // for lk in range(2):                                                                <L 288>
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_145 = wp::mul(var_98, var_144);
            var_147 = wp::mul(var_142, var_146);
            var_148 = wp::add(var_145, var_147);
            var_149 = wp::add(var_148, var_143);
            var_150 = eval_basis_trilinear_0(var_86, var_149);
            // gi = ci + li                                                                       <L 290>
            var_151 = wp::add(var_38, var_98);
            // gj = cj + lj                                                                       <L 291>
            var_152 = wp::add(var_48, var_142);
            // gk = ck + lk                                                                       <L 292>
            var_153 = wp::add(var_58, var_143);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_154 = wp::mul(var_151, var_88);
            var_155 = wp::mul(var_154, var_90);
            var_156 = wp::mul(var_152, var_90);
            var_157 = wp::add(var_155, var_156);
            var_158 = wp::add(var_157, var_153);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_159 = wp::add(var_92, var_158);
            var_160 = wp::address(var_flexnode_xpos_in, var_0, var_159);
            var_162 = wp::load(var_160);
            var_161 = wp::mul(var_150, var_162);
            var_163 = wp::add(var_141, var_161);
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_166 = wp::mul(var_98, var_165);
            var_168 = wp::mul(var_142, var_167);
            var_169 = wp::add(var_166, var_168);
            var_170 = wp::add(var_169, var_164);
            var_171 = eval_basis_trilinear_0(var_86, var_170);
            // gi = ci + li                                                                       <L 290>
            var_172 = wp::add(var_38, var_98);
            // gj = cj + lj                                                                       <L 291>
            var_173 = wp::add(var_48, var_142);
            // gk = ck + lk                                                                       <L 292>
            var_174 = wp::add(var_58, var_164);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_175 = wp::mul(var_172, var_88);
            var_176 = wp::mul(var_175, var_90);
            var_177 = wp::mul(var_173, var_90);
            var_178 = wp::add(var_176, var_177);
            var_179 = wp::add(var_178, var_174);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_180 = wp::add(var_92, var_179);
            var_181 = wp::address(var_flexnode_xpos_in, var_0, var_180);
            var_183 = wp::load(var_181);
            var_182 = wp::mul(var_171, var_183);
            var_184 = wp::add(var_163, var_182);
            // for lj in range(2):                                                                <L 287>
            // for lk in range(2):                                                                <L 288>
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_189 = wp::mul(var_185, var_188);
            var_191 = wp::mul(var_186, var_190);
            var_192 = wp::add(var_189, var_191);
            var_193 = wp::add(var_192, var_187);
            var_194 = eval_basis_trilinear_0(var_86, var_193);
            // gi = ci + li                                                                       <L 290>
            var_195 = wp::add(var_38, var_185);
            // gj = cj + lj                                                                       <L 291>
            var_196 = wp::add(var_48, var_186);
            // gk = ck + lk                                                                       <L 292>
            var_197 = wp::add(var_58, var_187);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_198 = wp::mul(var_195, var_88);
            var_199 = wp::mul(var_198, var_90);
            var_200 = wp::mul(var_196, var_90);
            var_201 = wp::add(var_199, var_200);
            var_202 = wp::add(var_201, var_197);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_203 = wp::add(var_92, var_202);
            var_204 = wp::address(var_flexnode_xpos_in, var_0, var_203);
            var_206 = wp::load(var_204);
            var_205 = wp::mul(var_194, var_206);
            var_207 = wp::add(var_184, var_205);
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_210 = wp::mul(var_185, var_209);
            var_212 = wp::mul(var_186, var_211);
            var_213 = wp::add(var_210, var_212);
            var_214 = wp::add(var_213, var_208);
            var_215 = eval_basis_trilinear_0(var_86, var_214);
            // gi = ci + li                                                                       <L 290>
            var_216 = wp::add(var_38, var_185);
            // gj = cj + lj                                                                       <L 291>
            var_217 = wp::add(var_48, var_186);
            // gk = ck + lk                                                                       <L 292>
            var_218 = wp::add(var_58, var_208);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_219 = wp::mul(var_216, var_88);
            var_220 = wp::mul(var_219, var_90);
            var_221 = wp::mul(var_217, var_90);
            var_222 = wp::add(var_220, var_221);
            var_223 = wp::add(var_222, var_218);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_224 = wp::add(var_92, var_223);
            var_225 = wp::address(var_flexnode_xpos_in, var_0, var_224);
            var_227 = wp::load(var_225);
            var_226 = wp::mul(var_215, var_227);
            var_228 = wp::add(var_207, var_226);
            // for lk in range(2):                                                                <L 288>
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_232 = wp::mul(var_185, var_231);
            var_234 = wp::mul(var_229, var_233);
            var_235 = wp::add(var_232, var_234);
            var_236 = wp::add(var_235, var_230);
            var_237 = eval_basis_trilinear_0(var_86, var_236);
            // gi = ci + li                                                                       <L 290>
            var_238 = wp::add(var_38, var_185);
            // gj = cj + lj                                                                       <L 291>
            var_239 = wp::add(var_48, var_229);
            // gk = ck + lk                                                                       <L 292>
            var_240 = wp::add(var_58, var_230);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_241 = wp::mul(var_238, var_88);
            var_242 = wp::mul(var_241, var_90);
            var_243 = wp::mul(var_239, var_90);
            var_244 = wp::add(var_242, var_243);
            var_245 = wp::add(var_244, var_240);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_246 = wp::add(var_92, var_245);
            var_247 = wp::address(var_flexnode_xpos_in, var_0, var_246);
            var_249 = wp::load(var_247);
            var_248 = wp::mul(var_237, var_249);
            var_250 = wp::add(var_228, var_248);
            // w = support.eval_basis_trilinear(local, li * 4 + lj * 2 + lk)                      <L 289>
            var_253 = wp::mul(var_185, var_252);
            var_255 = wp::mul(var_229, var_254);
            var_256 = wp::add(var_253, var_255);
            var_257 = wp::add(var_256, var_251);
            var_258 = eval_basis_trilinear_0(var_86, var_257);
            // gi = ci + li                                                                       <L 290>
            var_259 = wp::add(var_38, var_185);
            // gj = cj + lj                                                                       <L 291>
            var_260 = wp::add(var_48, var_229);
            // gk = ck + lk                                                                       <L 292>
            var_261 = wp::add(var_58, var_251);
            // node_idx = gi * ny_g * nz_g + gj * nz_g + gk                                       <L 293>
            var_262 = wp::mul(var_259, var_88);
            var_263 = wp::mul(var_262, var_90);
            var_264 = wp::mul(var_260, var_90);
            var_265 = wp::add(var_263, var_264);
            var_266 = wp::add(var_265, var_261);
            // result += w * flexnode_xpos_in[worldid, nstart + node_idx]                         <L 294>
            var_267 = wp::add(var_92, var_266);
            var_268 = wp::address(var_flexnode_xpos_in, var_0, var_267);
            var_270 = wp::load(var_268);
            var_269 = wp::mul(var_258, var_270);
            var_271 = wp::add(var_250, var_269);
            // flexvert_xpos_out[worldid, vertid] = result                                        <L 296>
            wp::array_store(var_flexvert_xpos_out, var_0, var_1, var_271);
        }
        if (!var_15) {
            // bodyid = flex_vertbodyid[vertid]                                                   <L 299>
            var_272 = wp::address(var_flex_vertbodyid, var_1);
            var_274 = wp::load(var_272);
            var_273 = wp::copy(var_274);
            // xpos = xpos_in[worldid, bodyid]                                                    <L 300>
            var_275 = wp::address(var_xpos_in, var_0, var_273);
            var_277 = wp::load(var_275);
            var_276 = wp::copy(var_277);
            // if flex_centered[f]:                                                               <L 302>
            var_278 = wp::address(var_flex_centered, var_3);
            var_279 = wp::load(var_278);
            if (var_279) {
                // flexvert_xpos_out[worldid, vertid] = xpos                                      <L 303>
                wp::array_store(var_flexvert_xpos_out, var_0, var_1, var_276);
            }
            var_280 = wp::load(var_278);
            var_281 = wp::load(var_278);
            if (!var_281) {
                // xmat = xmat_in[worldid, bodyid]                                                <L 305>
                var_282 = wp::address(var_xmat_in, var_0, var_273);
                var_284 = wp::load(var_282);
                var_283 = wp::copy(var_284);
                // local_pos = flex_vert[vertid]                                                  <L 306>
                var_285 = wp::address(var_flex_vert, var_1);
                var_287 = wp::load(var_285);
                var_286 = wp::copy(var_287);
                // flexvert_xpos_out[worldid, vertid] = xmat @ local_pos + xpos                   <L 307>
                var_288 = wp::mul(var_283, var_286);
                var_289 = wp::add(var_288, var_276);
                wp::array_store(var_flexvert_xpos_out, var_0, var_1, var_289);
            }
            var_290 = wp::load(var_278);
        }
    }
}



extern "C" __global__ void _crb_accumulate_073a11b8_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::vec_t<10, wp::float32>> var_crb_in,
    wp::array_t<wp::int32> var_body_tree_,
    wp::array_t<wp::vec_t<10, wp::float32>> var_crb_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::vec_t<10, wp::float32>* var_10;
        wp::vec_t<10, wp::float32> var_11;
        wp::vec_t<10, wp::float32> var_12;
        //---------
        // forward
        // def _crb_accumulate(                                                                   <L 1000>
        // worldid, nodeid = wp.tid()                                                             <L 1010>
        builtin_tid2d(var_0, var_1);
        // bodyid = body_tree_[nodeid]                                                            <L 1011>
        var_2 = wp::address(var_body_tree_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // pid = body_parentid[bodyid]                                                            <L 1012>
        var_5 = wp::address(var_body_parentid, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if pid == 0:                                                                           <L 1013>
        var_9 = (var_6 == var_8);
        if (var_9) {
            // return                                                                             <L 1014>
            continue;
        }
        // wp.atomic_add(crb_out, worldid, pid, crb_in[worldid, bodyid])                          <L 1015>
        var_10 = wp::address(var_crb_in, var_0, var_3);
        var_12 = wp::load(var_10);
        var_11 = wp::atomic_add(var_crb_out, var_0, var_6, var_12);
    }
}



extern "C" __global__ void _cacc_world_6ec26fdb_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::vec_t<3, wp::float32>> var_gravity,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cacc_out)
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
        const wp::float32 var_1 = 0.0;
        wp::vec_t<3, wp::float32> var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::vec_t<3, wp::float32>* var_8;
        wp::vec_t<3, wp::float32> var_9;
        wp::vec_t<3, wp::float32> var_10;
        wp::vec_t<6, wp::float32> var_11;
        const wp::int32 var_12 = 0;
        //---------
        // forward
        // def _cacc_world(                                                                       <L 1324>
        // worldid = wp.tid()                                                                     <L 1330>
        var_0 = builtin_tid1d();
        // cacc_out[worldid, 0] = wp.spatial_vector(wp.vec3(0.0), -gravity[worldid % gravity.shape[0]])       <L 1331>
        var_2 = wp::vec_t<3, wp::float32>(var_1);
        var_3 = &(var_gravity.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_gravity, var_7);
        var_10 = wp::load(var_8);
        var_9 = wp::neg(var_10);
        var_11 = wp::vec_t<6, wp::float32>(var_2, var_9);
        wp::array_store(var_cacc_out, var_0, var_12, var_11);
    }
}



extern "C" __global__ void _M_13671fca_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_dof_parentid,
    wp::array_t<wp::float32> var_dof_armature,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<10, wp::float32>> var_crb_in,
    wp::array_t<wp::float32> var_M_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32* var_6;
        wp::int32 var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        const wp::int32 var_10 = 1;
        wp::int32 var_11;
        wp::shape_t* var_12;
        const wp::int32 var_13 = 0;
        wp::int32 var_14;
        wp::shape_t var_15;
        wp::int32 var_16;
        wp::float32* var_17;
        wp::float32 var_18;
        wp::vec_t<10, wp::float32>* var_19;
        wp::vec_t<6, wp::float32>* var_20;
        wp::vec_t<6, wp::float32> var_21;
        wp::vec_t<10, wp::float32> var_22;
        wp::vec_t<6, wp::float32> var_23;
        const wp::int32 var_24 = 0;
        bool var_25;
        wp::vec_t<6, wp::float32>* var_26;
        wp::float32 var_27;
        wp::vec_t<6, wp::float32> var_28;
        wp::float32 var_29;
        const wp::int32 var_30 = 1;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        //---------
        // forward
        // def _M(                                                                                <L 1019>
        // worldid, dofid = wp.tid()                                                              <L 1032>
        builtin_tid2d(var_0, var_1);
        // bodyid = dof_bodyid[dofid]                                                             <L 1033>
        var_2 = wp::address(var_dof_bodyid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // madr_ij = M_rowadr[dofid] + M_rownnz[dofid] - 1                                        <L 1034>
        var_5 = wp::address(var_M_rowadr, var_1);
        var_6 = wp::address(var_M_rownnz, var_1);
        var_8 = wp::load(var_5);
        var_9 = wp::load(var_6);
        var_7 = wp::add(var_8, var_9);
        var_11 = wp::sub(var_7, var_10);
        // M_out[worldid, madr_ij] = dof_armature[worldid % dof_armature.shape[0], dofid]         <L 1037>
        var_12 = &(var_dof_armature.shape);
        var_15 = wp::load(var_12);
        var_14 = wp::extract(var_15, var_13);
        var_16 = wp::mod(var_0, var_14);
        var_17 = wp::address(var_dof_armature, var_16, var_1);
        var_18 = wp::load(var_17);
        wp::array_store(var_M_out, var_0, var_11, var_18);
        // buf = math.inert_vec(crb_in[worldid, bodyid], cdof_in[worldid, dofid])                 <L 1040>
        var_19 = wp::address(var_crb_in, var_0, var_3);
        var_20 = wp::address(var_cdof_in, var_0, var_1);
        var_22 = wp::load(var_19);
        var_23 = wp::load(var_20);
        var_21 = inert_vec_0(var_22, var_23);
        // while dofid >= 0:                                                                      <L 1043>
        start_while_0:;
        var_25 = (var_1 >= var_24);
        if ((var_25) == false) goto end_while_0;
            // M_out[worldid, madr_ij] += wp.dot(cdof_in[worldid, dofid], buf)                    <L 1044>
            var_26 = wp::address(var_cdof_in, var_0, var_1);
            var_28 = wp::load(var_26);
            var_27 = wp::dot(var_28, var_21);
            var_29 = wp::atomic_add(var_M_out, var_0, var_11, var_27);
            // madr_ij -= 1                                                                       <L 1045>
            var_31 = wp::sub(var_11, var_30);
            // dofid = dof_parentid[dofid]                                                        <L 1046>
            var_32 = wp::address(var_dof_parentid, var_1);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            wp::assign(var_1, var_33);
            wp::assign(var_11, var_31);
        goto start_while_0;
        end_while_0:;
    }
}



extern "C" __global__ void _flex_edges_4aa58739_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_edge,
    wp::array_t<wp::int32> var_flexedge_J_rownnz,
    wp::array_t<wp::int32> var_flexedge_J_rowadr,
    wp::array_t<wp::int32> var_flexedge_J_colind,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::int32> var_flex_edgeflexid,
    wp::array_t<wp::float32> var_qvel_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::float32> var_flexedge_J_out,
    wp::array_t<wp::float32> var_flexedge_length_out,
    wp::array_t<wp::float32> var_flexedge_velocity_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::vec_t<2, wp::int32>* var_8;
        wp::vec_t<2, wp::int32> var_9;
        wp::vec_t<2, wp::int32> var_10;
        const wp::int32 var_11 = 0;
        wp::int32 var_12;
        wp::int32 var_13;
        const wp::int32 var_14 = 1;
        wp::int32 var_15;
        wp::int32 var_16;
        wp::vec_t<3, wp::float32>* var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::vec_t<3, wp::float32>* var_20;
        wp::vec_t<3, wp::float32> var_21;
        wp::vec_t<3, wp::float32> var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::vec_t<3, wp::float32> var_24;
        wp::float32 var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::int32* var_29;
        wp::int32 var_30;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        bool var_35;
        const wp::int32 var_36 = 0;
        bool var_37;
        const wp::int32 var_38 = 0;
        bool var_39;
        const wp::int32 var_40 = 0;
        bool var_41;
        const wp::float32 var_42 = 0.0;
        wp::int32* var_43;
        wp::int32 var_44;
        wp::int32 var_45;
        wp::int32* var_46;
        wp::int32 var_47;
        wp::int32 var_48;
        wp::int32* var_49;
        wp::int32 var_50;
        wp::int32 var_51;
        wp::vec_t<3, wp::float32>* var_52;
        wp::vec_t<3, wp::float32> var_53;
        wp::vec_t<3, wp::float32> var_54;
        bool var_55;
        wp::vec_t<3, wp::float32>* var_56;
        wp::vec_t<3, wp::float32> var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::vec_t<3, wp::float32> var_60;
        const wp::float32 var_61 = 0.0;
        wp::float32 var_62;
        wp::range_t var_63;
        wp::int32 var_64;
        wp::int32 var_65;
        wp::int32* var_66;
        wp::int32 var_67;
        wp::int32 var_68;
        wp::vec_t<6, wp::float32>* var_69;
        wp::vec_t<6, wp::float32> var_70;
        wp::vec_t<6, wp::float32> var_71;
        wp::vec_t<3, wp::float32> var_72;
        wp::vec_t<3, wp::float32> var_73;
        const wp::float32 var_74 = 0.0;
        wp::float32 var_75;
        wp::int32* var_76;
        const wp::int32 var_77 = 0;
        bool var_78;
        wp::int32 var_79;
        wp::vec_t<3, wp::float32> var_80;
        wp::vec_t<3, wp::float32> var_81;
        wp::float32 var_82;
        wp::float32 var_83;
        wp::float32 var_84;
        wp::int32* var_85;
        const wp::int32 var_86 = 0;
        bool var_87;
        wp::int32 var_88;
        wp::vec_t<3, wp::float32> var_89;
        wp::vec_t<3, wp::float32> var_90;
        wp::float32 var_91;
        wp::float32 var_92;
        wp::float32 var_93;
        wp::int32 var_94;
        wp::float32* var_95;
        wp::float32 var_96;
        wp::float32 var_97;
        wp::float32 var_98;
        //---------
        // forward
        // def _flex_edges(                                                                       <L 347>
        // worldid, edgeid = wp.tid()                                                             <L 368>
        builtin_tid2d(var_0, var_1);
        // f = flex_edgeflexid[edgeid]                                                            <L 369>
        var_2 = wp::address(var_flex_edgeflexid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // vbase = flex_vertadr[f]                                                                <L 371>
        var_5 = wp::address(var_flex_vertadr, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // v = flex_edge[edgeid]                                                                  <L 372>
        var_8 = wp::address(var_flex_edge, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // vbase0 = vbase + v[0]                                                                  <L 373>
        var_12 = wp::extract(var_9, var_11);
        var_13 = wp::add(var_6, var_12);
        // vbase1 = vbase + v[1]                                                                  <L 374>
        var_15 = wp::extract(var_9, var_14);
        var_16 = wp::add(var_6, var_15);
        // pos1 = flexvert_xpos_in[worldid, vbase0]                                               <L 376>
        var_17 = wp::address(var_flexvert_xpos_in, var_0, var_13);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // pos2 = flexvert_xpos_in[worldid, vbase1]                                               <L 377>
        var_20 = wp::address(var_flexvert_xpos_in, var_0, var_16);
        var_22 = wp::load(var_20);
        var_21 = wp::copy(var_22);
        // vec = pos2 - pos1                                                                      <L 378>
        var_23 = wp::sub(var_21, var_18);
        // edge, edge_length = math.normalize_with_norm(vec)                                      <L 379>
        normalize_with_norm_0(var_23, var_24, var_25);
        // flexedge_length_out[worldid, edgeid] = edge_length                                     <L 380>
        wp::array_store(var_flexedge_length_out, var_0, var_1, var_25);
        // b1 = flex_vertbodyid[vbase0]                                                           <L 382>
        var_26 = wp::address(var_flex_vertbodyid, var_13);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // b2 = flex_vertbodyid[vbase1]                                                           <L 383>
        var_29 = wp::address(var_flex_vertbodyid, var_16);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
        // rownnz = flexedge_J_rownnz[edgeid]                                                     <L 385>
        var_32 = wp::address(var_flexedge_J_rownnz, var_1);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // if b1 < 0 or b2 < 0 or rownnz == 0:                                                    <L 386>
        var_37 = (var_27 < var_36);
        var_35 = var_37;
        if (!var_35) {
            var_39 = (var_30 < var_38);
            var_35 = var_35 || var_39;
        }
        if (!var_35) {
            var_41 = (var_33 == var_40);
            var_35 = var_35 || var_41;
        }
        if (var_35) {
            // flexedge_velocity_out[worldid, edgeid] = 0.0                                       <L 387>
            wp::array_store(var_flexedge_velocity_out, var_0, var_1, var_42);
            // return                                                                             <L 388>
            continue;
        }
        // rowadr = flexedge_J_rowadr[edgeid]                                                     <L 390>
        var_43 = wp::address(var_flexedge_J_rowadr, var_1);
        var_45 = wp::load(var_43);
        var_44 = wp::copy(var_45);
        // root1 = body_rootid[b1]                                                                <L 391>
        var_46 = wp::address(var_body_rootid, var_27);
        var_48 = wp::load(var_46);
        var_47 = wp::copy(var_48);
        // root2 = body_rootid[b2]                                                                <L 392>
        var_49 = wp::address(var_body_rootid, var_30);
        var_51 = wp::load(var_49);
        var_50 = wp::copy(var_51);
        // com1 = subtree_com_in[worldid, root1]                                                  <L 393>
        var_52 = wp::address(var_subtree_com_in, var_0, var_47);
        var_54 = wp::load(var_52);
        var_53 = wp::copy(var_54);
        // com2 = com1 if root2 == root1 else subtree_com_in[worldid, root2]                      <L 394>
        var_55 = (var_50 == var_47);
        if (var_55) {
        }
        if (!var_55) {
            var_56 = wp::address(var_subtree_com_in, var_0, var_50);
            var_57 = wp::load(var_56);
        }
        var_58 = wp::where(var_55, var_53, var_57);
        // offset1 = pos1 - com1                                                                  <L 395>
        var_59 = wp::sub(var_18, var_53);
        // offset2 = pos2 - com2                                                                  <L 396>
        var_60 = wp::sub(var_21, var_58);
        // vel = float(0.0)                                                                       <L 397>
        var_62 = wp::float(var_61);
        // for i in range(rownnz):                                                                <L 399>
        var_63 = wp::range(var_33);
        start_for_1:;
            if (iter_cmp(var_63) == 0) goto end_for_1;
            var_64 = wp::iter_next(var_63);
            // dofid = flexedge_J_colind[rowadr + i]                                              <L 400>
            var_65 = wp::add(var_44, var_64);
            var_66 = wp::address(var_flexedge_J_colind, var_65);
            var_68 = wp::load(var_66);
            var_67 = wp::copy(var_68);
            // cdof = cdof_in[worldid, dofid]                                                     <L 401>
            var_69 = wp::address(var_cdof_in, var_0, var_67);
            var_71 = wp::load(var_69);
            var_70 = wp::copy(var_71);
            // cdof_ang = wp.spatial_top(cdof)                                                    <L 402>
            var_72 = wp::spatial_top(var_70);
            // cdof_lin = wp.spatial_bottom(cdof)                                                 <L 403>
            var_73 = wp::spatial_bottom(var_70);
            // J_val = float(0.0)                                                                 <L 404>
            var_75 = wp::float(var_74);
            // if body_isdofancestor[b1, dofid] != 0:                                             <L 405>
            var_76 = wp::address(var_body_isdofancestor, var_27, var_67);
            var_79 = wp::load(var_76);
            var_78 = (var_79 != var_77);
            if (var_78) {
                // jacp1 = cdof_lin + wp.cross(cdof_ang, offset1)                                 <L 406>
                var_80 = wp::cross(var_72, var_59);
                var_81 = wp::add(var_73, var_80);
                // J_val -= wp.dot(jacp1, edge)                                                   <L 407>
                var_82 = wp::dot(var_81, var_24);
                var_83 = wp::sub(var_75, var_82);
            }
            var_84 = wp::where(var_78, var_83, var_75);
            // if body_isdofancestor[b2, dofid] != 0:                                             <L 408>
            var_85 = wp::address(var_body_isdofancestor, var_30, var_67);
            var_88 = wp::load(var_85);
            var_87 = (var_88 != var_86);
            if (var_87) {
                // jacp2 = cdof_lin + wp.cross(cdof_ang, offset2)                                 <L 409>
                var_89 = wp::cross(var_72, var_60);
                var_90 = wp::add(var_73, var_89);
                // J_val += wp.dot(jacp2, edge)                                                   <L 410>
                var_91 = wp::dot(var_90, var_24);
                var_92 = wp::add(var_84, var_91);
            }
            var_93 = wp::where(var_87, var_92, var_84);
            // flexedge_J_out[worldid, rowadr + i] = J_val                                        <L 411>
            var_94 = wp::add(var_44, var_64);
            wp::array_store(var_flexedge_J_out, var_0, var_94, var_93);
            // vel += J_val * qvel_in[worldid, dofid]                                             <L 412>
            var_95 = wp::address(var_qvel_in, var_0, var_67);
            var_97 = wp::load(var_95);
            var_96 = wp::mul(var_93, var_97);
            var_98 = wp::add(var_62, var_96);
            wp::assign(var_62, var_98);
            goto start_for_1;
        end_for_1:;
        // flexedge_velocity_out[worldid, edgeid] = vel                                           <L 414>
        wp::array_store(var_flexedge_velocity_out, var_0, var_1, var_62);
    }
}



extern "C" __global__ void _cacc_branch_ffe1d6c4_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_body_branches,
    wp::array_t<wp::int32> var_body_branch_start,
    wp::array_t<wp::float32> var_qvel_in,
    wp::array_t<wp::float32> var_qacc_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_dot_in,
    bool var_flg_acc,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cacc_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        const wp::int32 var_5 = 1;
        wp::int32 var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        wp::vec_t<6, wp::float32>* var_16;
        wp::vec_t<6, wp::float32> var_17;
        wp::vec_t<6, wp::float32> var_18;
        wp::range_t var_19;
        wp::int32 var_20;
        wp::int32* var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        wp::int32* var_24;
        wp::int32 var_25;
        wp::int32 var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::range_t var_30;
        wp::int32 var_31;
        wp::int32 var_32;
        wp::vec_t<6, wp::float32>* var_33;
        wp::int32 var_34;
        wp::float32* var_35;
        wp::vec_t<6, wp::float32> var_36;
        wp::vec_t<6, wp::float32> var_37;
        wp::float32 var_38;
        wp::vec_t<6, wp::float32> var_39;
        wp::int32 var_40;
        wp::vec_t<6, wp::float32>* var_41;
        wp::int32 var_42;
        wp::float32* var_43;
        wp::vec_t<6, wp::float32> var_44;
        wp::vec_t<6, wp::float32> var_45;
        wp::float32 var_46;
        wp::vec_t<6, wp::float32> var_47;
        wp::vec_t<6, wp::float32> var_48;
        //---------
        // forward
        // def _cacc_branch(                                                                      <L 1342>
        // worldid, branchid = wp.tid()                                                           <L 1359>
        builtin_tid2d(var_0, var_1);
        // start = body_branch_start[branchid]                                                    <L 1361>
        var_2 = wp::address(var_body_branch_start, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // end = body_branch_start[branchid + 1]                                                  <L 1362>
        var_6 = wp::add(var_1, var_5);
        var_7 = wp::address(var_body_branch_start, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // bodyid = body_branches[start]                                                          <L 1364>
        var_10 = wp::address(var_body_branches, var_3);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // pid = body_parentid[bodyid]                                                            <L 1365>
        var_13 = wp::address(var_body_parentid, var_11);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // local_cacc = cacc_out[worldid, pid]                                                    <L 1366>
        var_16 = wp::address(var_cacc_out, var_0, var_14);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // for i in range(start, end):                                                            <L 1367>
        var_19 = wp::range(var_3, var_8);
        start_for_0:;
            if (iter_cmp(var_19) == 0) goto end_for_0;
            var_20 = wp::iter_next(var_19);
            // bodyid = body_branches[i]                                                          <L 1368>
            var_21 = wp::address(var_body_branches, var_20);
            var_23 = wp::load(var_21);
            var_22 = wp::copy(var_23);
            // dofnum = body_dofnum[bodyid]                                                       <L 1369>
            var_24 = wp::address(var_body_dofnum, var_22);
            var_26 = wp::load(var_24);
            var_25 = wp::copy(var_26);
            // dofadr = body_dofadr[bodyid]                                                       <L 1370>
            var_27 = wp::address(var_body_dofadr, var_22);
            var_29 = wp::load(var_27);
            var_28 = wp::copy(var_29);
            // for j in range(dofnum):                                                            <L 1371>
            var_30 = wp::range(var_25);
            start_for_2:;
                if (iter_cmp(var_30) == 0) goto end_for_2;
                var_31 = wp::iter_next(var_30);
                // local_cacc += cdof_dot_in[worldid, dofadr + j] * qvel_in[worldid, dofadr + j]       <L 1372>
                var_32 = wp::add(var_28, var_31);
                var_33 = wp::address(var_cdof_dot_in, var_0, var_32);
                var_34 = wp::add(var_28, var_31);
                var_35 = wp::address(var_qvel_in, var_0, var_34);
                var_37 = wp::load(var_33);
                var_38 = wp::load(var_35);
                var_36 = wp::mul(var_37, var_38);
                var_39 = wp::add(var_17, var_36);
                // if flg_acc:                                                                    <L 1373>
                if (var_flg_acc) {
                    // local_cacc += cdof_in[worldid, dofadr + j] * qacc_in[worldid, dofadr + j]       <L 1374>
                    var_40 = wp::add(var_28, var_31);
                    var_41 = wp::address(var_cdof_in, var_0, var_40);
                    var_42 = wp::add(var_28, var_31);
                    var_43 = wp::address(var_qacc_in, var_0, var_42);
                    var_45 = wp::load(var_41);
                    var_46 = wp::load(var_43);
                    var_44 = wp::mul(var_45, var_46);
                    var_47 = wp::add(var_39, var_44);
                }
                var_48 = wp::where(var_flg_acc, var_47, var_39);
                wp::assign(var_17, var_48);
                goto start_for_2;
            end_for_2:;
            // cacc_out[worldid, bodyid] = local_cacc                                             <L 1375>
            wp::array_store(var_cacc_out, var_0, var_22, var_17);
            wp::assign(var_11, var_22);
            goto start_for_0;
        end_for_0:;
    }
}



extern "C" __global__ void _cfrc_3922b605_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::vec_t<10, wp::float32>> var_cinert_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cacc_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_ext_in,
    bool var_flg_cfrc_ext,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_int_out)
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
        wp::int32 var_1;
        const wp::int32 var_2 = 0;
        bool var_3;
        const wp::float32 var_4 = 0.0;
        const wp::float32 var_5 = 0.0;
        const wp::float32 var_6 = 0.0;
        const wp::float32 var_7 = 0.0;
        const wp::float32 var_8 = 0.0;
        const wp::float32 var_9 = 0.0;
        wp::vec_t<6, wp::float32> var_10;
        const wp::int32 var_11 = 0;
        wp::vec_t<6, wp::float32>* var_12;
        wp::vec_t<6, wp::float32> var_13;
        wp::vec_t<6, wp::float32> var_14;
        wp::vec_t<10, wp::float32>* var_15;
        wp::vec_t<10, wp::float32> var_16;
        wp::vec_t<10, wp::float32> var_17;
        wp::vec_t<6, wp::float32>* var_18;
        wp::vec_t<6, wp::float32> var_19;
        wp::vec_t<6, wp::float32> var_20;
        wp::vec_t<6, wp::float32> var_21;
        wp::vec_t<6, wp::float32> var_22;
        wp::vec_t<6, wp::float32> var_23;
        wp::vec_t<6, wp::float32> var_24;
        wp::vec_t<6, wp::float32>* var_25;
        wp::vec_t<6, wp::float32> var_26;
        wp::vec_t<6, wp::float32> var_27;
        wp::vec_t<6, wp::float32> var_28;
        //---------
        // forward
        // def _cfrc(                                                                             <L 1399>
        // worldid, bodyid = wp.tid()                                                             <L 1410>
        builtin_tid2d(var_0, var_1);
        // if bodyid == 0:                                                                        <L 1411>
        var_3 = (var_1 == var_2);
        if (var_3) {
            // cfrc_int_out[worldid, 0] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)         <L 1412>
            var_10 = wp::vec_t<6, wp::float32>({var_4, var_5, var_6, var_7, var_8, var_9});
            wp::array_store(var_cfrc_int_out, var_0, var_11, var_10);
            // return                                                                             <L 1413>
            continue;
        }
        // cacc = cacc_in[worldid, bodyid]                                                        <L 1414>
        var_12 = wp::address(var_cacc_in, var_0, var_1);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // cinert = cinert_in[worldid, bodyid]                                                    <L 1415>
        var_15 = wp::address(var_cinert_in, var_0, var_1);
        var_17 = wp::load(var_15);
        var_16 = wp::copy(var_17);
        // cvel = cvel_in[worldid, bodyid]                                                        <L 1416>
        var_18 = wp::address(var_cvel_in, var_0, var_1);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // frc = math.inert_vec(cinert, cacc)                                                     <L 1417>
        var_21 = inert_vec_0(var_16, var_13);
        // frc += math.motion_cross_force(cvel, math.inert_vec(cinert, cvel))                     <L 1418>
        var_22 = inert_vec_0(var_16, var_19);
        var_23 = motion_cross_force_0(var_19, var_22);
        var_24 = wp::add(var_21, var_23);
        // if flg_cfrc_ext:                                                                       <L 1419>
        if (var_flg_cfrc_ext) {
            // frc -= cfrc_ext_in[worldid, bodyid]                                                <L 1420>
            var_25 = wp::address(var_cfrc_ext_in, var_0, var_1);
            var_27 = wp::load(var_25);
            var_26 = wp::sub(var_24, var_27);
        }
        var_28 = wp::where(var_flg_cfrc_ext, var_26, var_24);
        // cfrc_int_out[worldid, bodyid] = frc                                                    <L 1422>
        wp::array_store(var_cfrc_int_out, var_0, var_1, var_28);
    }
}



extern "C" __global__ void _cdof_3e73eb78_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::int32> var_jnt_bodyid,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xanchor_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xaxis_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_out)
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
        wp::vec_t<3, wp::float32>* var_11;
        wp::vec_t<3, wp::float32> var_12;
        wp::vec_t<3, wp::float32> var_13;
        wp::mat_t<3, 3, wp::float32>* var_14;
        wp::mat_t<3, 3, wp::float32> var_15;
        wp::mat_t<3, 3, wp::float32> var_16;
        wp::int32* var_17;
        wp::vec_t<3, wp::float32>* var_18;
        wp::int32 var_19;
        wp::vec_t<3, wp::float32>* var_20;
        wp::vec_t<3, wp::float32> var_21;
        wp::vec_t<3, wp::float32> var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::slice_t var_24;
        const wp::int32 var_25 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_26;
        const wp::int32 var_27 = 0;
        bool var_28;
        const wp::float32 var_29 = 0.0;
        const wp::float32 var_30 = 0.0;
        const wp::float32 var_31 = 0.0;
        const wp::float32 var_32 = 1.0;
        const wp::float32 var_33 = 0.0;
        const wp::float32 var_34 = 0.0;
        wp::vec_t<6, wp::float32> var_35;
        const wp::int32 var_36 = 0;
        wp::int32 var_37;
        const wp::float32 var_38 = 0.0;
        const wp::float32 var_39 = 0.0;
        const wp::float32 var_40 = 0.0;
        const wp::float32 var_41 = 0.0;
        const wp::float32 var_42 = 1.0;
        const wp::float32 var_43 = 0.0;
        wp::vec_t<6, wp::float32> var_44;
        const wp::int32 var_45 = 1;
        wp::int32 var_46;
        const wp::float32 var_47 = 0.0;
        const wp::float32 var_48 = 0.0;
        const wp::float32 var_49 = 0.0;
        const wp::float32 var_50 = 0.0;
        const wp::float32 var_51 = 0.0;
        const wp::float32 var_52 = 1.0;
        wp::vec_t<6, wp::float32> var_53;
        const wp::int32 var_54 = 2;
        wp::int32 var_55;
        const wp::int32 var_56 = 0;
        wp::vec_t<3, wp::float32> var_57;
        const wp::int32 var_58 = 0;
        wp::vec_t<3, wp::float32> var_59;
        wp::vec_t<3, wp::float32> var_60;
        wp::vec_t<6, wp::float32> var_61;
        const wp::int32 var_62 = 3;
        wp::int32 var_63;
        const wp::int32 var_64 = 1;
        wp::vec_t<3, wp::float32> var_65;
        const wp::int32 var_66 = 1;
        wp::vec_t<3, wp::float32> var_67;
        wp::vec_t<3, wp::float32> var_68;
        wp::vec_t<6, wp::float32> var_69;
        const wp::int32 var_70 = 4;
        wp::int32 var_71;
        const wp::int32 var_72 = 2;
        wp::vec_t<3, wp::float32> var_73;
        const wp::int32 var_74 = 2;
        wp::vec_t<3, wp::float32> var_75;
        wp::vec_t<3, wp::float32> var_76;
        wp::vec_t<6, wp::float32> var_77;
        const wp::int32 var_78 = 5;
        wp::int32 var_79;
        const wp::int32 var_80 = 1;
        bool var_81;
        const wp::int32 var_82 = 0;
        wp::vec_t<3, wp::float32> var_83;
        const wp::int32 var_84 = 0;
        wp::vec_t<3, wp::float32> var_85;
        wp::vec_t<3, wp::float32> var_86;
        wp::vec_t<6, wp::float32> var_87;
        const wp::int32 var_88 = 0;
        wp::int32 var_89;
        const wp::int32 var_90 = 1;
        wp::vec_t<3, wp::float32> var_91;
        const wp::int32 var_92 = 1;
        wp::vec_t<3, wp::float32> var_93;
        wp::vec_t<3, wp::float32> var_94;
        wp::vec_t<6, wp::float32> var_95;
        const wp::int32 var_96 = 1;
        wp::int32 var_97;
        const wp::int32 var_98 = 2;
        wp::vec_t<3, wp::float32> var_99;
        const wp::int32 var_100 = 2;
        wp::vec_t<3, wp::float32> var_101;
        wp::vec_t<3, wp::float32> var_102;
        wp::vec_t<6, wp::float32> var_103;
        const wp::int32 var_104 = 2;
        wp::int32 var_105;
        const wp::int32 var_106 = 2;
        bool var_107;
        const wp::float32 var_108 = 0.0;
        wp::vec_t<3, wp::float32> var_109;
        wp::vec_t<6, wp::float32> var_110;
        const wp::int32 var_111 = 3;
        bool var_112;
        wp::vec_t<3, wp::float32> var_113;
        wp::vec_t<6, wp::float32> var_114;
        //---------
        // forward
        // def _cdof(                                                                             <L 750>
        // worldid, jntid = wp.tid()                                                              <L 764>
        builtin_tid2d(var_0, var_1);
        // bodyid = jnt_bodyid[jntid]                                                             <L 765>
        var_2 = wp::address(var_jnt_bodyid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // dofid = jnt_dofadr[jntid]                                                              <L 766>
        var_5 = wp::address(var_jnt_dofadr, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // jnt_type_ = jnt_type[jntid]                                                            <L 767>
        var_8 = wp::address(var_jnt_type, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // xaxis = xaxis_in[worldid, jntid]                                                       <L 768>
        var_11 = wp::address(var_xaxis_in, var_0, var_1);
        var_13 = wp::load(var_11);
        var_12 = wp::copy(var_13);
        // xmat = wp.transpose(xmat_in[worldid, bodyid])                                          <L 769>
        var_14 = wp::address(var_xmat_in, var_0, var_3);
        var_16 = wp::load(var_14);
        var_15 = wp::transpose(var_16);
        // offset = subtree_com_in[worldid, body_rootid[bodyid]] - xanchor_in[worldid, jntid]       <L 772>
        var_17 = wp::address(var_body_rootid, var_3);
        var_19 = wp::load(var_17);
        var_18 = wp::address(var_subtree_com_in, var_0, var_19);
        var_20 = wp::address(var_xanchor_in, var_0, var_1);
        var_22 = wp::load(var_18);
        var_23 = wp::load(var_20);
        var_21 = wp::sub(var_22, var_23);
        // res = cdof_out[worldid]                                                                <L 774>
        var_24 = wp::slice_t(var_0, var_0, var_25);
        var_26 = wp::view(var_cdof_out, var_24);
        // if jnt_type_ == JointType.FREE:                                                        <L 775>
        var_28 = (var_9 == var_27);
        if (var_28) {
            // res[dofid + 0] = wp.spatial_vector(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)                   <L 776>
            var_35 = wp::vec_t<6, wp::float32>({var_29, var_30, var_31, var_32, var_33, var_34});
            var_37 = wp::add(var_6, var_36);
            wp::array_store(var_26, var_37, var_35);
            // res[dofid + 1] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 1.0, 0.0)                   <L 777>
            var_44 = wp::vec_t<6, wp::float32>({var_38, var_39, var_40, var_41, var_42, var_43});
            var_46 = wp::add(var_6, var_45);
            wp::array_store(var_26, var_46, var_44);
            // res[dofid + 2] = wp.spatial_vector(0.0, 0.0, 0.0, 0.0, 0.0, 1.0)                   <L 778>
            var_53 = wp::vec_t<6, wp::float32>({var_47, var_48, var_49, var_50, var_51, var_52});
            var_55 = wp::add(var_6, var_54);
            wp::array_store(var_26, var_55, var_53);
            // res[dofid + 3] = wp.spatial_vector(xmat[0], wp.cross(xmat[0], offset))             <L 780>
            var_57 = wp::extract(var_15, var_56);
            var_59 = wp::extract(var_15, var_58);
            var_60 = wp::cross(var_59, var_21);
            var_61 = wp::vec_t<6, wp::float32>(var_57, var_60);
            var_63 = wp::add(var_6, var_62);
            wp::array_store(var_26, var_63, var_61);
            // res[dofid + 4] = wp.spatial_vector(xmat[1], wp.cross(xmat[1], offset))             <L 781>
            var_65 = wp::extract(var_15, var_64);
            var_67 = wp::extract(var_15, var_66);
            var_68 = wp::cross(var_67, var_21);
            var_69 = wp::vec_t<6, wp::float32>(var_65, var_68);
            var_71 = wp::add(var_6, var_70);
            wp::array_store(var_26, var_71, var_69);
            // res[dofid + 5] = wp.spatial_vector(xmat[2], wp.cross(xmat[2], offset))             <L 782>
            var_73 = wp::extract(var_15, var_72);
            var_75 = wp::extract(var_15, var_74);
            var_76 = wp::cross(var_75, var_21);
            var_77 = wp::vec_t<6, wp::float32>(var_73, var_76);
            var_79 = wp::add(var_6, var_78);
            wp::array_store(var_26, var_79, var_77);
        }
        if (!var_28) {
            // elif jnt_type_ == JointType.BALL:  # ball                                          <L 783>
            var_81 = (var_9 == var_80);
            if (var_81) {
                // res[dofid + 0] = wp.spatial_vector(xmat[0], wp.cross(xmat[0], offset))         <L 785>
                var_83 = wp::extract(var_15, var_82);
                var_85 = wp::extract(var_15, var_84);
                var_86 = wp::cross(var_85, var_21);
                var_87 = wp::vec_t<6, wp::float32>(var_83, var_86);
                var_89 = wp::add(var_6, var_88);
                wp::array_store(var_26, var_89, var_87);
                // res[dofid + 1] = wp.spatial_vector(xmat[1], wp.cross(xmat[1], offset))         <L 786>
                var_91 = wp::extract(var_15, var_90);
                var_93 = wp::extract(var_15, var_92);
                var_94 = wp::cross(var_93, var_21);
                var_95 = wp::vec_t<6, wp::float32>(var_91, var_94);
                var_97 = wp::add(var_6, var_96);
                wp::array_store(var_26, var_97, var_95);
                // res[dofid + 2] = wp.spatial_vector(xmat[2], wp.cross(xmat[2], offset))         <L 787>
                var_99 = wp::extract(var_15, var_98);
                var_101 = wp::extract(var_15, var_100);
                var_102 = wp::cross(var_101, var_21);
                var_103 = wp::vec_t<6, wp::float32>(var_99, var_102);
                var_105 = wp::add(var_6, var_104);
                wp::array_store(var_26, var_105, var_103);
            }
            if (!var_81) {
                // elif jnt_type_ == JointType.SLIDE:                                             <L 788>
                var_107 = (var_9 == var_106);
                if (var_107) {
                    // res[dofid] = wp.spatial_vector(wp.vec3(0.0), xaxis)                        <L 789>
                    var_109 = wp::vec_t<3, wp::float32>(var_108);
                    var_110 = wp::vec_t<6, wp::float32>(var_109, var_12);
                    wp::array_store(var_26, var_6, var_110);
                }
                if (!var_107) {
                    // elif jnt_type_ == JointType.HINGE:  # hinge                                <L 790>
                    var_112 = (var_9 == var_111);
                    if (var_112) {
                        // res[dofid] = wp.spatial_vector(xaxis, wp.cross(xaxis, offset))         <L 791>
                        var_113 = wp::cross(var_12, var_21);
                        var_114 = wp::vec_t<6, wp::float32>(var_12, var_113);
                        wp::array_store(var_26, var_6, var_114);
                    }
                }
            }
        }
    }
}



extern "C" __global__ void _cfrc_ext_contact_348e18d8_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::int32 var_opt_cone,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_contact_pos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::vec_t<5, wp::float32>> var_contact_friction_in,
    wp::array_t<wp::int32> var_contact_dim_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_geom_in,
    wp::array_t<wp::int32> var_contact_efc_address_in,
    wp::array_t<wp::int32> var_contact_worldid_in,
    wp::array_t<wp::float32> var_contact_adhesion_in,
    wp::array_t<wp::float32> var_efc_force_in,
    wp::int32 var_njmax_in,
    wp::array_t<wp::int32> var_nacon_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_ext_out)
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
        bool var_3;
        wp::int32 var_4;
        wp::vec_t<2, wp::int32>* var_5;
        wp::vec_t<2, wp::int32> var_6;
        wp::vec_t<2, wp::int32> var_7;
        const wp::int32 var_8 = 0;
        wp::int32 var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        const wp::int32 var_13 = 1;
        wp::int32 var_14;
        wp::int32* var_15;
        wp::int32 var_16;
        wp::int32 var_17;
        bool var_18;
        const wp::int32 var_19 = 0;
        bool var_20;
        const wp::int32 var_21 = 0;
        bool var_22;
        wp::int32* var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        const bool var_26 = true;
        wp::vec_t<6, wp::float32> var_27;
        wp::vec_t<3, wp::float32>* var_28;
        wp::vec_t<3, wp::float32> var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::int32* var_31;
        wp::vec_t<3, wp::float32>* var_32;
        wp::int32 var_33;
        wp::vec_t<3, wp::float32> var_34;
        wp::vec_t<3, wp::float32> var_35;
        wp::slice_t var_36;
        const wp::int32 var_37 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_38;
        wp::vec_t<3, wp::float32> var_39;
        wp::vec_t<6, wp::float32> var_40;
        wp::vec_t<6, wp::float32> var_41;
        wp::int32* var_42;
        wp::vec_t<3, wp::float32>* var_43;
        wp::int32 var_44;
        wp::vec_t<3, wp::float32> var_45;
        wp::vec_t<3, wp::float32> var_46;
        wp::slice_t var_47;
        const wp::int32 var_48 = 0;
        wp::array_t<wp::vec_t<6, wp::float32>> var_49;
        wp::vec_t<3, wp::float32> var_50;
        wp::vec_t<6, wp::float32> var_51;
        wp::vec_t<6, wp::float32> var_52;
        //---------
        // forward
        // def _cfrc_ext_contact(                                                                 <L 1670>
        // contactid = wp.tid()                                                                   <L 1691>
        var_0 = builtin_tid1d();
        // if contactid >= nacon_in[0]:                                                           <L 1693>
        var_2 = wp::address(var_nacon_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 1694>
            continue;
        }
        // geom = contact_geom_in[contactid]                                                      <L 1696>
        var_5 = wp::address(var_contact_geom_in, var_0);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // id1 = geom_bodyid[geom[0]]                                                             <L 1697>
        var_9 = wp::extract(var_6, var_8);
        var_10 = wp::address(var_geom_bodyid, var_9);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // id2 = geom_bodyid[geom[1]]                                                             <L 1698>
        var_14 = wp::extract(var_6, var_13);
        var_15 = wp::address(var_geom_bodyid, var_14);
        var_17 = wp::load(var_15);
        var_16 = wp::copy(var_17);
        // if id1 == 0 and id2 == 0:                                                              <L 1700>
        var_20 = (var_11 == var_19);
        var_18 = var_20;
        if (var_18) {
            var_22 = (var_16 == var_21);
            var_18 = var_18 && var_22;
        }
        if (var_18) {
            // return                                                                             <L 1701>
            continue;
        }
        // worldid = contact_worldid_in[contactid]                                                <L 1703>
        var_23 = wp::address(var_contact_worldid_in, var_0);
        var_25 = wp::load(var_23);
        var_24 = wp::copy(var_25);
        // force = support.contact_force_fn(                                                      <L 1706>
        // opt_cone,                                                                              <L 1707>
        // contact_frame_in,                                                                      <L 1708>
        // contact_friction_in,                                                                   <L 1709>
        // contact_dim_in,                                                                        <L 1710>
        // contact_efc_address_in,                                                                <L 1711>
        // contact_adhesion_in,                                                                   <L 1712>
        // efc_force_in,                                                                          <L 1713>
        // njmax_in,                                                                              <L 1714>
        // nacon_in,                                                                              <L 1715>
        // worldid,                                                                               <L 1716>
        // contactid,                                                                             <L 1717>
        // to_world_frame=True,                                                                   <L 1718>
        var_27 = contact_force_fn_0(var_opt_cone, var_contact_frame_in, var_contact_friction_in, var_contact_dim_in, var_contact_efc_address_in, var_contact_adhesion_in, var_efc_force_in, var_njmax_in, var_nacon_in, var_24, var_0, var_26);
        // pos = contact_pos_in[contactid]                                                        <L 1721>
        var_28 = wp::address(var_contact_pos_in, var_0);
        var_30 = wp::load(var_28);
        var_29 = wp::copy(var_30);
        // if id1:                                                                                <L 1724>
        if (var_11) {
            // com1 = subtree_com_in[worldid, body_rootid[id1]]                                   <L 1725>
            var_31 = wp::address(var_body_rootid, var_11);
            var_33 = wp::load(var_31);
            var_32 = wp::address(var_subtree_com_in, var_24, var_33);
            var_35 = wp::load(var_32);
            var_34 = wp::copy(var_35);
            // wp.atomic_sub(cfrc_ext_out[worldid], id1, support.transform_force(force, com1 - pos))       <L 1726>
            var_36 = wp::slice_t(var_24, var_24, var_37);
            var_38 = wp::view(var_cfrc_ext_out, var_36);
            var_39 = wp::sub(var_34, var_29);
            var_40 = transform_force_1(var_27, var_39);
            var_41 = wp::atomic_sub(var_38, var_11, var_40);
        }
        // if id2:                                                                                <L 1728>
        if (var_16) {
            // com2 = subtree_com_in[worldid, body_rootid[id2]]                                   <L 1729>
            var_42 = wp::address(var_body_rootid, var_16);
            var_44 = wp::load(var_42);
            var_43 = wp::address(var_subtree_com_in, var_24, var_44);
            var_46 = wp::load(var_43);
            var_45 = wp::copy(var_46);
            // wp.atomic_add(cfrc_ext_out[worldid], id2, support.transform_force(force, com2 - pos))       <L 1730>
            var_47 = wp::slice_t(var_24, var_24, var_48);
            var_49 = wp::view(var_cfrc_ext_out, var_47);
            var_50 = wp::sub(var_45, var_29);
            var_51 = transform_force_1(var_27, var_50);
            var_52 = wp::atomic_add(var_49, var_16, var_51);
        }
    }
}



extern "C" __global__ void _cfrc_ext_tendon_constraint_06fedf68_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_eq_type,
    wp::array_t<wp::int32> var_eq_obj1id,
    wp::array_t<wp::int32> var_eq_obj2id,
    wp::array_t<wp::vec_t<11, wp::float32>> var_eq_data,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_range,
    wp::array_t<wp::float32> var_tendon_length0,
    wp::array_t<wp::int32> var_nefc_in,
    wp::array_t<wp::float32> var_ten_length_in,
    wp::array_t<wp::int32> var_efc_type_in,
    wp::array_t<wp::int32> var_efc_id_in,
    wp::array_t<wp::float32> var_efc_force_in,
    wp::array_t<wp::float32> var_ten_frc_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        bool var_3;
        wp::int32 var_4;
        wp::float32* var_5;
        wp::float32 var_6;
        wp::float32 var_7;
        const wp::float32 var_8 = 0.0;
        bool var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        const wp::int32 var_13 = 0;
        bool var_14;
        wp::int32* var_15;
        wp::int32 var_16;
        wp::int32 var_17;
        wp::int32* var_18;
        const wp::int32 var_19 = 3;
        bool var_20;
        wp::int32 var_21;
        wp::int32* var_22;
        wp::int32 var_23;
        wp::int32 var_24;
        wp::int32* var_25;
        wp::int32 var_26;
        wp::int32 var_27;
        wp::slice_t var_28;
        const wp::int32 var_29 = 0;
        wp::array_t<wp::float32> var_30;
        wp::float32 var_31;
        const wp::int32 var_32 = 0;
        bool var_33;
        wp::float32* var_34;
        wp::shape_t* var_35;
        const wp::int32 var_36 = 0;
        wp::int32 var_37;
        wp::shape_t var_38;
        wp::int32 var_39;
        wp::float32* var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::float32 var_43;
        wp::shape_t* var_44;
        const wp::int32 var_45 = 0;
        wp::int32 var_46;
        wp::shape_t var_47;
        wp::int32 var_48;
        wp::vec_t<11, wp::float32>* var_49;
        wp::vec_t<11, wp::float32> var_50;
        wp::vec_t<11, wp::float32> var_51;
        const wp::int32 var_52 = 1;
        wp::float32 var_53;
        const wp::float32 var_54 = 2.0;
        const wp::int32 var_55 = 2;
        wp::float32 var_56;
        wp::float32 var_57;
        wp::float32 var_58;
        wp::float32 var_59;
        const wp::float32 var_60 = 3.0;
        const wp::int32 var_61 = 3;
        wp::float32 var_62;
        wp::float32 var_63;
        wp::float32 var_64;
        wp::float32 var_65;
        wp::float32 var_66;
        const wp::float32 var_67 = 4.0;
        const wp::int32 var_68 = 4;
        wp::float32 var_69;
        wp::float32 var_70;
        wp::float32 var_71;
        wp::float32 var_72;
        wp::float32 var_73;
        wp::float32 var_74;
        wp::slice_t var_75;
        const wp::int32 var_76 = 0;
        wp::array_t<wp::float32> var_77;
        wp::float32 var_78;
        wp::float32 var_79;
        const wp::int32 var_80 = 2;
        bool var_81;
        wp::int32* var_82;
        wp::int32 var_83;
        wp::int32 var_84;
        wp::slice_t var_85;
        const wp::int32 var_86 = 0;
        wp::array_t<wp::float32> var_87;
        wp::float32 var_88;
        const wp::int32 var_89 = 4;
        bool var_90;
        wp::int32* var_91;
        wp::int32 var_92;
        wp::int32 var_93;
        wp::shape_t* var_94;
        const wp::int32 var_95 = 0;
        wp::int32 var_96;
        wp::shape_t var_97;
        wp::int32 var_98;
        wp::vec_t<2, wp::float32>* var_99;
        wp::vec_t<2, wp::float32> var_100;
        wp::vec_t<2, wp::float32> var_101;
        wp::float32* var_102;
        wp::float32 var_103;
        wp::float32 var_104;
        const wp::int32 var_105 = 0;
        wp::float32 var_106;
        wp::float32 var_107;
        const wp::int32 var_108 = 1;
        wp::float32 var_109;
        wp::float32 var_110;
        bool var_111;
        const wp::float32 var_112 = 1.0;
        const wp::float32 var_113 = -1.0;
        wp::float32 var_114;
        wp::slice_t var_115;
        const wp::int32 var_116 = 0;
        wp::array_t<wp::float32> var_117;
        wp::float32 var_118;
        wp::float32 var_119;
        wp::int32 var_120;
        //---------
        // forward
        // def _cfrc_ext_tendon_constraint(                                                       <L 1734>
        // worldid, efcid = wp.tid()                                                              <L 1751>
        builtin_tid2d(var_0, var_1);
        // if efcid >= nefc_in[worldid]:                                                          <L 1752>
        var_2 = wp::address(var_nefc_in, var_0);
        var_4 = wp::load(var_2);
        var_3 = (var_1 >= var_4);
        if (var_3) {
            // return                                                                             <L 1753>
            continue;
        }
        // efc_force = efc_force_in[worldid, efcid]                                               <L 1755>
        var_5 = wp::address(var_efc_force_in, var_0, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if efc_force == 0.0:                                                                   <L 1756>
        var_9 = (var_6 == var_8);
        if (var_9) {
            // return                                                                             <L 1757>
            continue;
        }
        // efc_type = efc_type_in[worldid, efcid]                                                 <L 1759>
        var_10 = wp::address(var_efc_type_in, var_0, var_1);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // if efc_type == ConstraintType.EQUALITY:                                                <L 1761>
        var_14 = (var_11 == var_13);
        if (var_14) {
            // eqid = efc_id_in[worldid, efcid]                                                   <L 1762>
            var_15 = wp::address(var_efc_id_in, var_0, var_1);
            var_17 = wp::load(var_15);
            var_16 = wp::copy(var_17);
            // if eq_type[eqid] != EqType.TENDON:                                                 <L 1763>
            var_18 = wp::address(var_eq_type, var_16);
            var_21 = wp::load(var_18);
            var_20 = (var_21 != var_19);
            if (var_20) {
                // return                                                                         <L 1764>
                continue;
            }
            // obj1 = eq_obj1id[eqid]                                                             <L 1765>
            var_22 = wp::address(var_eq_obj1id, var_16);
            var_24 = wp::load(var_22);
            var_23 = wp::copy(var_24);
            // obj2 = eq_obj2id[eqid]                                                             <L 1766>
            var_25 = wp::address(var_eq_obj2id, var_16);
            var_27 = wp::load(var_25);
            var_26 = wp::copy(var_27);
            // wp.atomic_add(ten_frc_out[worldid], obj1, efc_force)                               <L 1767>
            var_28 = wp::slice_t(var_0, var_0, var_29);
            var_30 = wp::view(var_ten_frc_out, var_28);
            var_31 = wp::atomic_add(var_30, var_23, var_6);
            // if obj2 >= 0:                                                                      <L 1768>
            var_33 = (var_26 >= var_32);
            if (var_33) {
                // dif = ten_length_in[worldid, obj2] - tendon_length0[worldid % tendon_length0.shape[0], obj2]       <L 1769>
                var_34 = wp::address(var_ten_length_in, var_0, var_26);
                var_35 = &(var_tendon_length0.shape);
                var_38 = wp::load(var_35);
                var_37 = wp::extract(var_38, var_36);
                var_39 = wp::mod(var_0, var_37);
                var_40 = wp::address(var_tendon_length0, var_39, var_26);
                var_42 = wp::load(var_34);
                var_43 = wp::load(var_40);
                var_41 = wp::sub(var_42, var_43);
                // eq_data_ = eq_data[worldid % eq_data.shape[0], eqid]                           <L 1770>
                var_44 = &(var_eq_data.shape);
                var_47 = wp::load(var_44);
                var_46 = wp::extract(var_47, var_45);
                var_48 = wp::mod(var_0, var_46);
                var_49 = wp::address(var_eq_data, var_48, var_16);
                var_51 = wp::load(var_49);
                var_50 = wp::copy(var_51);
                // deriv = eq_data_[1] + 2.0 * eq_data_[2] * dif + 3.0 * eq_data_[3] * dif * dif + 4.0 * eq_data_[4] * dif * dif * dif       <L 1771>
                var_53 = wp::extract(var_50, var_52);
                var_56 = wp::extract(var_50, var_55);
                var_57 = wp::mul(var_54, var_56);
                var_58 = wp::mul(var_57, var_41);
                var_59 = wp::add(var_53, var_58);
                var_62 = wp::extract(var_50, var_61);
                var_63 = wp::mul(var_60, var_62);
                var_64 = wp::mul(var_63, var_41);
                var_65 = wp::mul(var_64, var_41);
                var_66 = wp::add(var_59, var_65);
                var_69 = wp::extract(var_50, var_68);
                var_70 = wp::mul(var_67, var_69);
                var_71 = wp::mul(var_70, var_41);
                var_72 = wp::mul(var_71, var_41);
                var_73 = wp::mul(var_72, var_41);
                var_74 = wp::add(var_66, var_73);
                // wp.atomic_sub(ten_frc_out[worldid], obj2, deriv * efc_force)                   <L 1772>
                var_75 = wp::slice_t(var_0, var_0, var_76);
                var_77 = wp::view(var_ten_frc_out, var_75);
                var_78 = wp::mul(var_74, var_6);
                var_79 = wp::atomic_sub(var_77, var_26, var_78);
            }
            // return                                                                             <L 1773>
            continue;
        }
        // if efc_type == ConstraintType.FRICTION_TENDON:                                         <L 1775>
        var_81 = (var_11 == var_80);
        if (var_81) {
            // tenid = efc_id_in[worldid, efcid]                                                  <L 1776>
            var_82 = wp::address(var_efc_id_in, var_0, var_1);
            var_84 = wp::load(var_82);
            var_83 = wp::copy(var_84);
            // wp.atomic_add(ten_frc_out[worldid], tenid, efc_force)                              <L 1777>
            var_85 = wp::slice_t(var_0, var_0, var_86);
            var_87 = wp::view(var_ten_frc_out, var_85);
            var_88 = wp::atomic_add(var_87, var_83, var_6);
            // return                                                                             <L 1778>
            continue;
        }
        // if efc_type == ConstraintType.LIMIT_TENDON:                                            <L 1780>
        var_90 = (var_11 == var_89);
        if (var_90) {
            // tenid = efc_id_in[worldid, efcid]                                                  <L 1781>
            var_91 = wp::address(var_efc_id_in, var_0, var_1);
            var_93 = wp::load(var_91);
            var_92 = wp::copy(var_93);
            // tenrange = tendon_range[worldid % tendon_range.shape[0], tenid]                    <L 1782>
            var_94 = &(var_tendon_range.shape);
            var_97 = wp::load(var_94);
            var_96 = wp::extract(var_97, var_95);
            var_98 = wp::mod(var_0, var_96);
            var_99 = wp::address(var_tendon_range, var_98, var_92);
            var_101 = wp::load(var_99);
            var_100 = wp::copy(var_101);
            // length = ten_length_in[worldid, tenid]                                             <L 1783>
            var_102 = wp::address(var_ten_length_in, var_0, var_92);
            var_104 = wp::load(var_102);
            var_103 = wp::copy(var_104);
            // dist_min = length - tenrange[0]                                                    <L 1784>
            var_106 = wp::extract(var_100, var_105);
            var_107 = wp::sub(var_103, var_106);
            // dist_max = tenrange[1] - length                                                    <L 1785>
            var_109 = wp::extract(var_100, var_108);
            var_110 = wp::sub(var_109, var_103);
            // scl = wp.where(dist_min < dist_max, 1.0, -1.0)                                     <L 1786>
            var_111 = (var_107 < var_110);
            var_114 = wp::where(var_111, var_112, var_113);
            // wp.atomic_add(ten_frc_out[worldid], tenid, scl * efc_force)                        <L 1787>
            var_115 = wp::slice_t(var_0, var_0, var_116);
            var_117 = wp::view(var_ten_frc_out, var_115);
            var_118 = wp::mul(var_114, var_6);
            var_119 = wp::atomic_add(var_117, var_92, var_118);
        }
        var_120 = wp::where(var_90, var_92, var_83);
    }
}



extern "C" __global__ void _spatial_site_tendon_d90d9688_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::int32> var_tendon_site_pair_adr,
    wp::array_t<wp::int32> var_wrap_site_pair_adr,
    wp::array_t<wp::float32> var_wrap_pulley_scale,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::float32> var_ten_J_out,
    wp::array_t<wp::float32> var_ten_length_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::float32 var_9;
        wp::float32 var_10;
        const wp::int32 var_11 = 0;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        const wp::int32 var_16 = 1;
        wp::int32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::vec_t<3, wp::float32>* var_21;
        wp::vec_t<3, wp::float32> var_22;
        wp::vec_t<3, wp::float32> var_23;
        wp::vec_t<3, wp::float32>* var_24;
        wp::vec_t<3, wp::float32> var_25;
        wp::vec_t<3, wp::float32> var_26;
        wp::vec_t<3, wp::float32> var_27;
        wp::vec_t<3, wp::float32> var_28;
        wp::float32 var_29;
        wp::slice_t var_30;
        const wp::int32 var_31 = 0;
        wp::array_t<wp::float32> var_32;
        wp::float32 var_33;
        wp::float32 var_34;
        const wp::float32 var_35 = 1e-15;
        bool var_36;
        const wp::float32 var_37 = 1.0;
        const wp::float32 var_38 = 0.0;
        const wp::float32 var_39 = 0.0;
        wp::vec_t<3, wp::float32> var_40;
        wp::vec_t<3, wp::float32> var_41;
        wp::int32* var_42;
        wp::int32 var_43;
        wp::int32 var_44;
        wp::int32* var_45;
        wp::int32 var_46;
        wp::int32 var_47;
        bool var_48;
        wp::int32* var_49;
        wp::int32 var_50;
        wp::int32 var_51;
        wp::int32* var_52;
        wp::int32 var_53;
        wp::int32 var_54;
        wp::int32* var_55;
        wp::vec_t<3, wp::float32>* var_56;
        wp::int32 var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::int32* var_60;
        wp::vec_t<3, wp::float32>* var_61;
        wp::int32 var_62;
        wp::vec_t<3, wp::float32> var_63;
        wp::vec_t<3, wp::float32> var_64;
        wp::float32 var_65;
        //---------
        // forward
        // def _spatial_site_tendon(                                                              <L 4072>
        // worldid, elementid = wp.tid()                                                          <L 4094>
        builtin_tid2d(var_0, var_1);
        // site_pair_adr = wrap_site_pair_adr[elementid]                                          <L 4097>
        var_2 = wp::address(var_wrap_site_pair_adr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // tenid = tendon_site_pair_adr[elementid]                                                <L 4098>
        var_5 = wp::address(var_tendon_site_pair_adr, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // pulley_scale = wrap_pulley_scale[site_pair_adr]                                        <L 4101>
        var_8 = wp::address(var_wrap_pulley_scale, var_3);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // id0 = wrap_objid[site_pair_adr + 0]                                                    <L 4103>
        var_12 = wp::add(var_3, var_11);
        var_13 = wp::address(var_wrap_objid, var_12);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // id1 = wrap_objid[site_pair_adr + 1]                                                    <L 4104>
        var_17 = wp::add(var_3, var_16);
        var_18 = wp::address(var_wrap_objid, var_17);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // pnt0 = site_xpos_in[worldid, id0]                                                      <L 4106>
        var_21 = wp::address(var_site_xpos_in, var_0, var_14);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // pnt1 = site_xpos_in[worldid, id1]                                                      <L 4107>
        var_24 = wp::address(var_site_xpos_in, var_0, var_19);
        var_26 = wp::load(var_24);
        var_25 = wp::copy(var_26);
        // dif = pnt1 - pnt0                                                                      <L 4108>
        var_27 = wp::sub(var_25, var_22);
        // vec, length = math.normalize_with_norm(dif)                                            <L 4109>
        normalize_with_norm_0(var_27, var_28, var_29);
        // wp.atomic_add(ten_length_out[worldid], tenid, length * pulley_scale)                   <L 4110>
        var_30 = wp::slice_t(var_0, var_0, var_31);
        var_32 = wp::view(var_ten_length_out, var_30);
        var_33 = wp::mul(var_29, var_9);
        var_34 = wp::atomic_add(var_32, var_6, var_33);
        // if length < MJ_MINVAL:                                                                 <L 4112>
        var_36 = (var_29 < var_35);
        if (var_36) {
            // vec = wp.vec3(1.0, 0.0, 0.0)                                                       <L 4113>
            var_40 = wp::vec_t<3, wp::float32>(var_37, var_38, var_39);
        }
        var_41 = wp::where(var_36, var_40, var_28);
        // body0 = site_bodyid[id0]                                                               <L 4115>
        var_42 = wp::address(var_site_bodyid, var_14);
        var_44 = wp::load(var_42);
        var_43 = wp::copy(var_44);
        // body1 = site_bodyid[id1]                                                               <L 4116>
        var_45 = wp::address(var_site_bodyid, var_19);
        var_47 = wp::load(var_45);
        var_46 = wp::copy(var_47);
        // if body0 != body1:                                                                     <L 4117>
        var_48 = (var_43 != var_46);
        if (var_48) {
            // rownnz = ten_J_rownnz[tenid]                                                       <L 4118>
            var_49 = wp::address(var_ten_J_rownnz, var_6);
            var_51 = wp::load(var_49);
            var_50 = wp::copy(var_51);
            // rowadr = ten_J_rowadr[tenid]                                                       <L 4119>
            var_52 = wp::address(var_ten_J_rowadr, var_6);
            var_54 = wp::load(var_52);
            var_53 = wp::copy(var_54);
            // offset0 = pnt0 - subtree_com_in[worldid, body_rootid[body0]]                       <L 4120>
            var_55 = wp::address(var_body_rootid, var_43);
            var_57 = wp::load(var_55);
            var_56 = wp::address(var_subtree_com_in, var_0, var_57);
            var_59 = wp::load(var_56);
            var_58 = wp::sub(var_22, var_59);
            // offset1 = pnt1 - subtree_com_in[worldid, body_rootid[body1]]                       <L 4121>
            var_60 = wp::address(var_body_rootid, var_46);
            var_62 = wp::load(var_60);
            var_61 = wp::address(var_subtree_com_in, var_0, var_62);
            var_64 = wp::load(var_61);
            var_63 = wp::sub(var_25, var_64);
            // _accumulate_jac_chain(                                                             <L 4122>
            // body_parentid,                                                                     <L 4123>
            // body_dofnum,                                                                       <L 4124>
            // body_dofadr,                                                                       <L 4125>
            // ten_J_colind,                                                                      <L 4126>
            // cdof_in,                                                                           <L 4127>
            // offset0,                                                                           <L 4128>
            // vec,                                                                               <L 4129>
            // body0,                                                                             <L 4130>
            // rowadr,                                                                            <L 4131>
            // rownnz,                                                                            <L 4132>
            // -pulley_scale,                                                                     <L 4133>
            var_65 = wp::neg(var_9);
            // worldid,                                                                           <L 4134>
            // ten_J_out,                                                                         <L 4135>
            _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_58, var_41, var_43, var_53, var_50, var_65, var_0, var_ten_J_out);
            // _accumulate_jac_chain(                                                             <L 4137>
            // body_parentid,                                                                     <L 4138>
            // body_dofnum,                                                                       <L 4139>
            // body_dofadr,                                                                       <L 4140>
            // ten_J_colind,                                                                      <L 4141>
            // cdof_in,                                                                           <L 4142>
            // offset1,                                                                           <L 4143>
            // vec,                                                                               <L 4144>
            // body1,                                                                             <L 4145>
            // rowadr,                                                                            <L 4146>
            // rownnz,                                                                            <L 4147>
            // pulley_scale,                                                                      <L 4148>
            // worldid,                                                                           <L 4149>
            // ten_J_out,                                                                         <L 4150>
            _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_63, var_41, var_46, var_53, var_50, var_9, var_0, var_ten_J_out);
        }
    }
}



extern "C" __global__ void _qLDiag_div_3444613a_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_L_in,
    wp::array_t<wp::float32> var_D_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32* var_3;
        wp::int32 var_4;
        wp::int32 var_5;
        wp::int32 var_6;
        const wp::int32 var_7 = 1;
        wp::int32 var_8;
        const wp::float32 var_9 = 1.0;
        wp::float32* var_10;
        wp::float32 var_11;
        wp::float32 var_12;
        //---------
        // forward
        // def _qLDiag_div(                                                                       <L 1180>
        // worldid, dofid = wp.tid()                                                              <L 1189>
        builtin_tid2d(var_0, var_1);
        // diag_i = M_rowadr[dofid] + M_rownnz[dofid] - 1  # Address of diagonal element of i       <L 1190>
        var_2 = wp::address(var_M_rowadr, var_1);
        var_3 = wp::address(var_M_rownnz, var_1);
        var_5 = wp::load(var_2);
        var_6 = wp::load(var_3);
        var_4 = wp::add(var_5, var_6);
        var_8 = wp::sub(var_4, var_7);
        // D_out[worldid, dofid] = 1.0 / L_in[worldid, diag_i]                                    <L 1191>
        var_10 = wp::address(var_L_in, var_0, var_8);
        var_12 = wp::load(var_10);
        var_11 = wp::div(var_9, var_12);
        wp::array_store(var_D_out, var_0, var_1, var_11);
    }
}



extern "C" __global__ void _qLD_acc_120d68f2_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::vec_t<3, wp::int32>> var_qLD_updates_,
    wp::array_t<wp::float32> var_L_in,
    wp::array_t<wp::float32> var_L_out)
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
        wp::int32 var_1;
        wp::vec_t<3, wp::int32>* var_2;
        wp::vec_t<3, wp::int32> var_3;
        wp::vec_t<3, wp::int32> var_4;
        const wp::int32 var_5 = 0;
        wp::int32 var_6;
        const wp::int32 var_7 = 1;
        wp::int32 var_8;
        const wp::int32 var_9 = 2;
        wp::int32 var_10;
        wp::int32* var_11;
        wp::int32 var_12;
        wp::int32 var_13;
        wp::int32* var_14;
        wp::int32* var_15;
        wp::int32 var_16;
        wp::int32 var_17;
        wp::int32 var_18;
        const wp::int32 var_19 = 1;
        wp::int32 var_20;
        wp::float32* var_21;
        wp::float32* var_22;
        wp::float32 var_23;
        wp::float32 var_24;
        wp::float32 var_25;
        wp::int32* var_26;
        wp::range_t var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::slice_t var_30;
        const wp::int32 var_31 = 0;
        wp::array_t<wp::float32> var_32;
        wp::int32 var_33;
        wp::int32* var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        wp::float32* var_37;
        wp::float32 var_38;
        wp::float32 var_39;
        wp::float32 var_40;
        //---------
        // forward
        // def _qLD_acc(                                                                          <L 1155>
        // worldid, nodeid = wp.tid()                                                             <L 1165>
        builtin_tid2d(var_0, var_1);
        // update = qLD_updates_[nodeid]                                                          <L 1166>
        var_2 = wp::address(var_qLD_updates_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // i, k, Madr_ki = update[0], update[1], update[2]                                        <L 1167>
        var_6 = wp::extract(var_3, var_5);
        var_8 = wp::extract(var_3, var_7);
        var_10 = wp::extract(var_3, var_9);
        // Madr_i = M_rowadr[i]  # Address of row being updated                                   <L 1168>
        var_11 = wp::address(var_M_rowadr, var_6);
        var_13 = wp::load(var_11);
        var_12 = wp::copy(var_13);
        // diag_k = M_rowadr[k] + M_rownnz[k] - 1  # Address of diagonal element of k             <L 1169>
        var_14 = wp::address(var_M_rowadr, var_8);
        var_15 = wp::address(var_M_rownnz, var_8);
        var_17 = wp::load(var_14);
        var_18 = wp::load(var_15);
        var_16 = wp::add(var_17, var_18);
        var_20 = wp::sub(var_16, var_19);
        // tmp = L_out[worldid, Madr_ki] / L_out[worldid, diag_k]                                 <L 1171>
        var_21 = wp::address(var_L_out, var_0, var_10);
        var_22 = wp::address(var_L_out, var_0, var_20);
        var_24 = wp::load(var_21);
        var_25 = wp::load(var_22);
        var_23 = wp::div(var_24, var_25);
        // for j in range(M_rownnz[i]):                                                           <L 1172>
        var_26 = wp::address(var_M_rownnz, var_6);
        var_28 = wp::load(var_26);
        var_27 = wp::range(var_28);
        start_for_0:;
            if (iter_cmp(var_27) == 0) goto end_for_0;
            var_29 = wp::iter_next(var_27);
            // wp.atomic_sub(L_out[worldid], Madr_i + j, L_in[worldid, M_rowadr[k] + j] * tmp)       <L 1174>
            var_30 = wp::slice_t(var_0, var_0, var_31);
            var_32 = wp::view(var_L_out, var_30);
            var_33 = wp::add(var_12, var_29);
            var_34 = wp::address(var_M_rowadr, var_8);
            var_36 = wp::load(var_34);
            var_35 = wp::add(var_36, var_29);
            var_37 = wp::address(var_L_in, var_0, var_35);
            var_39 = wp::load(var_37);
            var_38 = wp::mul(var_39, var_23);
            var_40 = wp::atomic_sub(var_32, var_33, var_38);
            goto start_for_0;
        end_for_0:;
        // L_out[worldid, Madr_ki] = tmp                                                          <L 1176>
        wp::array_store(var_L_out, var_0, var_10, var_23);
    }
}



extern "C" __global__ void _cfrc_ext_tendon_actuator_f699e449_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_actuator_trntype,
    wp::array_t<wp::vec_t<2, wp::int32>> var_actuator_trnid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_actuator_gear,
    wp::array_t<wp::float32> var_actuator_force_in,
    wp::array_t<wp::float32> var_ten_frc_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        const wp::int32 var_3 = 3;
        bool var_4;
        wp::int32 var_5;
        wp::float32* var_6;
        wp::float32 var_7;
        wp::float32 var_8;
        const wp::float32 var_9 = 0.0;
        bool var_10;
        wp::vec_t<2, wp::int32>* var_11;
        const wp::int32 var_12 = 0;
        wp::int32 var_13;
        wp::vec_t<2, wp::int32> var_14;
        wp::shape_t* var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        wp::shape_t var_18;
        wp::int32 var_19;
        wp::vec_t<6, wp::float32>* var_20;
        const wp::int32 var_21 = 0;
        wp::float32 var_22;
        wp::vec_t<6, wp::float32> var_23;
        wp::slice_t var_24;
        const wp::int32 var_25 = 0;
        wp::array_t<wp::float32> var_26;
        wp::float32 var_27;
        wp::float32 var_28;
        //---------
        // forward
        // def _cfrc_ext_tendon_actuator(                                                         <L 1791>
        // worldid, actid = wp.tid()                                                              <L 1801>
        builtin_tid2d(var_0, var_1);
        // if actuator_trntype[actid] != TrnType.TENDON:                                          <L 1802>
        var_2 = wp::address(var_actuator_trntype, var_1);
        var_5 = wp::load(var_2);
        var_4 = (var_5 != var_3);
        if (var_4) {
            // return                                                                             <L 1803>
            continue;
        }
        // force = actuator_force_in[worldid, actid]                                              <L 1805>
        var_6 = wp::address(var_actuator_force_in, var_0, var_1);
        var_8 = wp::load(var_6);
        var_7 = wp::copy(var_8);
        // if force == 0.0:                                                                       <L 1806>
        var_10 = (var_7 == var_9);
        if (var_10) {
            // return                                                                             <L 1807>
            continue;
        }
        // tenid = actuator_trnid[actid][0]                                                       <L 1809>
        var_11 = wp::address(var_actuator_trnid, var_1);
        var_14 = wp::load(var_11);
        var_13 = wp::extract(var_14, var_12);
        // gear = actuator_gear[worldid % actuator_gear.shape[0], actid][0]                       <L 1810>
        var_15 = &(var_actuator_gear.shape);
        var_18 = wp::load(var_15);
        var_17 = wp::extract(var_18, var_16);
        var_19 = wp::mod(var_0, var_17);
        var_20 = wp::address(var_actuator_gear, var_19, var_1);
        var_23 = wp::load(var_20);
        var_22 = wp::extract(var_23, var_21);
        // wp.atomic_add(ten_frc_out[worldid], tenid, force * gear)                               <L 1811>
        var_24 = wp::slice_t(var_0, var_0, var_25);
        var_26 = wp::view(var_ten_frc_out, var_24);
        var_27 = wp::mul(var_7, var_22);
        var_28 = wp::atomic_add(var_26, var_13, var_27);
    }
}



extern "C" __global__ void _spatial_geom_tendon_1ca37386_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::int32> var_wrap_type,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::float32> var_wrap_prm,
    wp::array_t<wp::int32> var_tendon_geom_adr,
    wp::array_t<wp::int32> var_wrap_geom_adr,
    wp::array_t<wp::float32> var_wrap_pulley_scale,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::float32> var_ten_J_out,
    wp::array_t<wp::float32> var_ten_length_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_wrap_geom_xpos_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::float32 var_9;
        wp::float32 var_10;
        const wp::int32 var_11 = 1;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        const wp::int32 var_21 = 1;
        wp::int32 var_22;
        wp::int32* var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        wp::vec_t<3, wp::float32>* var_26;
        wp::vec_t<3, wp::float32> var_27;
        wp::vec_t<3, wp::float32> var_28;
        wp::vec_t<3, wp::float32>* var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::vec_t<3, wp::float32> var_31;
        wp::vec_t<3, wp::float32>* var_32;
        wp::vec_t<3, wp::float32> var_33;
        wp::vec_t<3, wp::float32> var_34;
        wp::mat_t<3, 3, wp::float32>* var_35;
        wp::mat_t<3, 3, wp::float32> var_36;
        wp::mat_t<3, 3, wp::float32> var_37;
        wp::shape_t* var_38;
        const wp::int32 var_39 = 0;
        wp::int32 var_40;
        wp::shape_t var_41;
        wp::int32 var_42;
        wp::vec_t<3, wp::float32>* var_43;
        const wp::int32 var_44 = 0;
        wp::float32 var_45;
        wp::vec_t<3, wp::float32> var_46;
        wp::int32* var_47;
        wp::int32 var_48;
        wp::int32 var_49;
        wp::int32* var_50;
        wp::int32 var_51;
        wp::int32 var_52;
        wp::int32* var_53;
        wp::int32 var_54;
        wp::int32 var_55;
        wp::int32* var_56;
        wp::int32 var_57;
        wp::int32 var_58;
        wp::float32* var_59;
        wp::float32 var_60;
        wp::float32 var_61;
        wp::int32 var_62;
        const wp::int32 var_63 = 0;
        bool var_64;
        wp::vec_t<3, wp::float32>* var_65;
        wp::vec_t<3, wp::float32> var_66;
        wp::vec_t<3, wp::float32> var_67;
        const wp::float32 var_68 = 10000000000.0;
        wp::vec_t<3, wp::float32> var_69;
        wp::vec_t<3, wp::float32> var_70;
        wp::float32 var_71;
        wp::vec_t<3, wp::float32> var_72;
        wp::vec_t<3, wp::float32> var_73;
        wp::vec_t<6, wp::float32> var_74;
        wp::int32* var_75;
        wp::int32 var_76;
        wp::int32 var_77;
        wp::int32* var_78;
        wp::int32 var_79;
        wp::int32 var_80;
        const wp::float32 var_81 = 0.0;
        bool var_82;
        wp::vec_t<3, wp::float32> var_83;
        wp::vec_t<3, wp::float32> var_84;
        wp::vec_t<3, wp::float32> var_85;
        wp::float32 var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::float32 var_88;
        wp::float32 var_89;
        wp::float32 var_90;
        wp::slice_t var_91;
        const wp::int32 var_92 = 0;
        wp::array_t<wp::float32> var_93;
        wp::float32 var_94;
        wp::float32 var_95;
        const wp::float32 var_96 = 1e-15;
        bool var_97;
        const wp::float32 var_98 = 1.0;
        const wp::float32 var_99 = 0.0;
        const wp::float32 var_100 = 0.0;
        wp::vec_t<3, wp::float32> var_101;
        wp::vec_t<3, wp::float32> var_102;
        bool var_103;
        const wp::float32 var_104 = 1.0;
        const wp::float32 var_105 = 0.0;
        const wp::float32 var_106 = 0.0;
        wp::vec_t<3, wp::float32> var_107;
        wp::vec_t<3, wp::float32> var_108;
        bool var_109;
        bool var_110;
        wp::int32* var_111;
        wp::vec_t<3, wp::float32>* var_112;
        wp::int32 var_113;
        wp::vec_t<3, wp::float32> var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::int32* var_116;
        wp::vec_t<3, wp::float32>* var_117;
        wp::int32 var_118;
        wp::vec_t<3, wp::float32> var_119;
        wp::vec_t<3, wp::float32> var_120;
        wp::float32 var_121;
        wp::int32* var_122;
        wp::vec_t<3, wp::float32>* var_123;
        wp::int32 var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32> var_126;
        wp::int32* var_127;
        wp::vec_t<3, wp::float32>* var_128;
        wp::int32 var_129;
        wp::vec_t<3, wp::float32> var_130;
        wp::vec_t<3, wp::float32> var_131;
        wp::float32 var_132;
        wp::vec_t<3, wp::float32> var_133;
        wp::vec_t<3, wp::float32> var_134;
        wp::float32 var_135;
        wp::slice_t var_136;
        const wp::int32 var_137 = 0;
        wp::array_t<wp::float32> var_138;
        wp::float32 var_139;
        wp::float32 var_140;
        bool var_141;
        const wp::float32 var_142 = 1.0;
        const wp::float32 var_143 = 0.0;
        const wp::float32 var_144 = 0.0;
        wp::vec_t<3, wp::float32> var_145;
        wp::vec_t<3, wp::float32> var_146;
        bool var_147;
        wp::int32* var_148;
        wp::vec_t<3, wp::float32>* var_149;
        wp::int32 var_150;
        wp::vec_t<3, wp::float32> var_151;
        wp::vec_t<3, wp::float32> var_152;
        wp::int32* var_153;
        wp::vec_t<3, wp::float32>* var_154;
        wp::int32 var_155;
        wp::vec_t<3, wp::float32> var_156;
        wp::vec_t<3, wp::float32> var_157;
        wp::float32 var_158;
        wp::vec_t<3, wp::float32> var_159;
        wp::vec_t<3, wp::float32> var_160;
        wp::vec_t<3, wp::float32> var_161;
        wp::vec_t<3, wp::float32> var_162;
        //---------
        // forward
        // def _spatial_geom_tendon(                                                              <L 4155>
        // worldid, elementid = wp.tid()                                                          <L 4185>
        builtin_tid2d(var_0, var_1);
        // wrap_adr = wrap_geom_adr[elementid]                                                    <L 4186>
        var_2 = wp::address(var_wrap_geom_adr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // tenid = tendon_geom_adr[elementid]                                                     <L 4187>
        var_5 = wp::address(var_tendon_geom_adr, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // pulley_scale = wrap_pulley_scale[wrap_adr]                                             <L 4190>
        var_8 = wp::address(var_wrap_pulley_scale, var_3);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // wrap_objid_site0 = wrap_objid[wrap_adr - 1]                                            <L 4193>
        var_12 = wp::sub(var_3, var_11);
        var_13 = wp::address(var_wrap_objid, var_12);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // wrap_objid_geom = wrap_objid[wrap_adr + 0]                                             <L 4194>
        var_17 = wp::add(var_3, var_16);
        var_18 = wp::address(var_wrap_objid, var_17);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // wrap_objid_site1 = wrap_objid[wrap_adr + 1]                                            <L 4195>
        var_22 = wp::add(var_3, var_21);
        var_23 = wp::address(var_wrap_objid, var_22);
        var_25 = wp::load(var_23);
        var_24 = wp::copy(var_25);
        // site_pnt0 = site_xpos_in[worldid, wrap_objid_site0]                                    <L 4198>
        var_26 = wp::address(var_site_xpos_in, var_0, var_14);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // site_pnt1 = site_xpos_in[worldid, wrap_objid_site1]                                    <L 4199>
        var_29 = wp::address(var_site_xpos_in, var_0, var_24);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
        // geom_xpos = geom_xpos_in[worldid, wrap_objid_geom]                                     <L 4202>
        var_32 = wp::address(var_geom_xpos_in, var_0, var_19);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // geom_xmat = geom_xmat_in[worldid, wrap_objid_geom]                                     <L 4203>
        var_35 = wp::address(var_geom_xmat_in, var_0, var_19);
        var_37 = wp::load(var_35);
        var_36 = wp::copy(var_37);
        // geomsize = geom_size[worldid % geom_size.shape[0], wrap_objid_geom][0]                 <L 4204>
        var_38 = &(var_geom_size.shape);
        var_41 = wp::load(var_38);
        var_40 = wp::extract(var_41, var_39);
        var_42 = wp::mod(var_0, var_40);
        var_43 = wp::address(var_geom_size, var_42, var_19);
        var_46 = wp::load(var_43);
        var_45 = wp::extract(var_46, var_44);
        // geom_type = wrap_type[wrap_adr]                                                        <L 4205>
        var_47 = wp::address(var_wrap_type, var_3);
        var_49 = wp::load(var_47);
        var_48 = wp::copy(var_49);
        // bodyid_site0 = site_bodyid[wrap_objid_site0]                                           <L 4208>
        var_50 = wp::address(var_site_bodyid, var_14);
        var_52 = wp::load(var_50);
        var_51 = wp::copy(var_52);
        // bodyid_geom = geom_bodyid[wrap_objid_geom]                                             <L 4209>
        var_53 = wp::address(var_geom_bodyid, var_19);
        var_55 = wp::load(var_53);
        var_54 = wp::copy(var_55);
        // bodyid_site1 = site_bodyid[wrap_objid_site1]                                           <L 4210>
        var_56 = wp::address(var_site_bodyid, var_24);
        var_58 = wp::load(var_56);
        var_57 = wp::copy(var_58);
        // sideid = int(wp.round(wrap_prm[wrap_adr]))                                             <L 4213>
        var_59 = wp::address(var_wrap_prm, var_3);
        var_61 = wp::load(var_59);
        var_60 = wp::round(var_61);
        var_62 = wp::int(var_60);
        // if sideid >= 0:                                                                        <L 4214>
        var_64 = (var_62 >= var_63);
        if (var_64) {
            // side = site_xpos_in[worldid, sideid]                                               <L 4215>
            var_65 = wp::address(var_site_xpos_in, var_0, var_62);
            var_67 = wp::load(var_65);
            var_66 = wp::copy(var_67);
        }
        if (!var_64) {
            // side = wp.vec3(MJ_MAXVAL)                                                          <L 4217>
            var_69 = wp::vec_t<3, wp::float32>(var_68);
        }
        var_70 = wp::where(var_64, var_66, var_69);
        // length_geomgeom, geom_pnt0, geom_pnt1 = util_misc.wrap(site_pnt0, site_pnt1, geom_xpos, geom_xmat, geomsize, geom_type, side)       <L 4220>
        wrap_0(var_27, var_30, var_33, var_36, var_45, var_48, var_70, var_71, var_72, var_73);
        // wrap_geom_xpos_out[worldid, elementid] = wp.spatial_vector(geom_pnt0, geom_pnt1)       <L 4223>
        var_74 = wp::vec_t<6, wp::float32>(var_72, var_73);
        wp::array_store(var_wrap_geom_xpos_out, var_0, var_1, var_74);
        // rownnz = ten_J_rownnz[tenid]                                                           <L 4225>
        var_75 = wp::address(var_ten_J_rownnz, var_6);
        var_77 = wp::load(var_75);
        var_76 = wp::copy(var_77);
        // rowadr = ten_J_rowadr[tenid]                                                           <L 4226>
        var_78 = wp::address(var_ten_J_rowadr, var_6);
        var_80 = wp::load(var_78);
        var_79 = wp::copy(var_80);
        // if length_geomgeom >= 0.0:                                                             <L 4228>
        var_82 = (var_71 >= var_81);
        if (var_82) {
            // dif_sitegeom = geom_pnt0 - site_pnt0                                               <L 4229>
            var_83 = wp::sub(var_72, var_27);
            // dif_geomsite = site_pnt1 - geom_pnt1                                               <L 4230>
            var_84 = wp::sub(var_30, var_73);
            // vec_sitegeom, length_sitegeom = math.normalize_with_norm(dif_sitegeom)             <L 4231>
            normalize_with_norm_0(var_83, var_85, var_86);
            // vec_geomsite, length_geomsite = math.normalize_with_norm(dif_geomsite)             <L 4232>
            normalize_with_norm_0(var_84, var_87, var_88);
            // length_sitegeomsite = length_sitegeom + length_geomgeom + length_geomsite          <L 4235>
            var_89 = wp::add(var_86, var_71);
            var_90 = wp::add(var_89, var_88);
            // if length_sitegeomsite:                                                            <L 4237>
            if (var_90) {
                // wp.atomic_add(ten_length_out[worldid], tenid, length_sitegeomsite * pulley_scale)       <L 4238>
                var_91 = wp::slice_t(var_0, var_0, var_92);
                var_93 = wp::view(var_ten_length_out, var_91);
                var_94 = wp::mul(var_90, var_9);
                var_95 = wp::atomic_add(var_93, var_6, var_94);
            }
            // if length_sitegeom < MJ_MINVAL:                                                    <L 4241>
            var_97 = (var_86 < var_96);
            if (var_97) {
                // vec_sitegeom = wp.vec3(1.0, 0.0, 0.0)                                          <L 4242>
                var_101 = wp::vec_t<3, wp::float32>(var_98, var_99, var_100);
            }
            var_102 = wp::where(var_97, var_101, var_85);
            // if length_geomsite < MJ_MINVAL:                                                    <L 4244>
            var_103 = (var_88 < var_96);
            if (var_103) {
                // vec_geomsite = wp.vec3(1.0, 0.0, 0.0)                                          <L 4245>
                var_107 = wp::vec_t<3, wp::float32>(var_104, var_105, var_106);
            }
            var_108 = wp::where(var_103, var_107, var_87);
            // dif_body_sitegeom = bodyid_site0 != bodyid_geom                                    <L 4247>
            var_109 = (var_51 != var_54);
            // dif_body_geomsite = bodyid_geom != bodyid_site1                                    <L 4248>
            var_110 = (var_54 != var_57);
            // if dif_body_sitegeom:                                                              <L 4251>
            if (var_109) {
                // offset_site0 = site_pnt0 - subtree_com_in[worldid, body_rootid[bodyid_site0]]       <L 4252>
                var_111 = wp::address(var_body_rootid, var_51);
                var_113 = wp::load(var_111);
                var_112 = wp::address(var_subtree_com_in, var_0, var_113);
                var_115 = wp::load(var_112);
                var_114 = wp::sub(var_27, var_115);
                // offset_geom0 = geom_pnt0 - subtree_com_in[worldid, body_rootid[bodyid_geom]]       <L 4253>
                var_116 = wp::address(var_body_rootid, var_54);
                var_118 = wp::load(var_116);
                var_117 = wp::address(var_subtree_com_in, var_0, var_118);
                var_120 = wp::load(var_117);
                var_119 = wp::sub(var_72, var_120);
                // _accumulate_jac_chain(                                                         <L 4254>
                // body_parentid,                                                                 <L 4255>
                // body_dofnum,                                                                   <L 4256>
                // body_dofadr,                                                                   <L 4257>
                // ten_J_colind,                                                                  <L 4258>
                // cdof_in,                                                                       <L 4259>
                // offset_site0,                                                                  <L 4260>
                // vec_sitegeom,                                                                  <L 4261>
                // bodyid_site0,                                                                  <L 4262>
                // rowadr,                                                                        <L 4263>
                // rownnz,                                                                        <L 4264>
                // -pulley_scale,                                                                 <L 4265>
                var_121 = wp::neg(var_9);
                // worldid,                                                                       <L 4266>
                // ten_J_out,                                                                     <L 4267>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_114, var_102, var_51, var_79, var_76, var_121, var_0, var_ten_J_out);
                // _accumulate_jac_chain(                                                         <L 4269>
                // body_parentid,                                                                 <L 4270>
                // body_dofnum,                                                                   <L 4271>
                // body_dofadr,                                                                   <L 4272>
                // ten_J_colind,                                                                  <L 4273>
                // cdof_in,                                                                       <L 4274>
                // offset_geom0,                                                                  <L 4275>
                // vec_sitegeom,                                                                  <L 4276>
                // bodyid_geom,                                                                   <L 4277>
                // rowadr,                                                                        <L 4278>
                // rownnz,                                                                        <L 4279>
                // pulley_scale,                                                                  <L 4280>
                // worldid,                                                                       <L 4281>
                // ten_J_out,                                                                     <L 4282>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_119, var_102, var_54, var_79, var_76, var_9, var_0, var_ten_J_out);
            }
            // if dif_body_geomsite:                                                              <L 4286>
            if (var_110) {
                // offset_geom1 = geom_pnt1 - subtree_com_in[worldid, body_rootid[bodyid_geom]]       <L 4287>
                var_122 = wp::address(var_body_rootid, var_54);
                var_124 = wp::load(var_122);
                var_123 = wp::address(var_subtree_com_in, var_0, var_124);
                var_126 = wp::load(var_123);
                var_125 = wp::sub(var_73, var_126);
                // offset_site1 = site_pnt1 - subtree_com_in[worldid, body_rootid[bodyid_site1]]       <L 4288>
                var_127 = wp::address(var_body_rootid, var_57);
                var_129 = wp::load(var_127);
                var_128 = wp::address(var_subtree_com_in, var_0, var_129);
                var_131 = wp::load(var_128);
                var_130 = wp::sub(var_30, var_131);
                // _accumulate_jac_chain(                                                         <L 4289>
                // body_parentid,                                                                 <L 4290>
                // body_dofnum,                                                                   <L 4291>
                // body_dofadr,                                                                   <L 4292>
                // ten_J_colind,                                                                  <L 4293>
                // cdof_in,                                                                       <L 4294>
                // offset_geom1,                                                                  <L 4295>
                // vec_geomsite,                                                                  <L 4296>
                // bodyid_geom,                                                                   <L 4297>
                // rowadr,                                                                        <L 4298>
                // rownnz,                                                                        <L 4299>
                // -pulley_scale,                                                                 <L 4300>
                var_132 = wp::neg(var_9);
                // worldid,                                                                       <L 4301>
                // ten_J_out,                                                                     <L 4302>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_125, var_108, var_54, var_79, var_76, var_132, var_0, var_ten_J_out);
                // _accumulate_jac_chain(                                                         <L 4304>
                // body_parentid,                                                                 <L 4305>
                // body_dofnum,                                                                   <L 4306>
                // body_dofadr,                                                                   <L 4307>
                // ten_J_colind,                                                                  <L 4308>
                // cdof_in,                                                                       <L 4309>
                // offset_site1,                                                                  <L 4310>
                // vec_geomsite,                                                                  <L 4311>
                // bodyid_site1,                                                                  <L 4312>
                // rowadr,                                                                        <L 4313>
                // rownnz,                                                                        <L 4314>
                // pulley_scale,                                                                  <L 4315>
                // worldid,                                                                       <L 4316>
                // ten_J_out,                                                                     <L 4317>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_130, var_108, var_57, var_79, var_76, var_9, var_0, var_ten_J_out);
            }
        }
        if (!var_82) {
            // dif_sitesite = site_pnt1 - site_pnt0                                               <L 4320>
            var_133 = wp::sub(var_30, var_27);
            // vec_sitesite, length_sitesite = math.normalize_with_norm(dif_sitesite)             <L 4321>
            normalize_with_norm_0(var_133, var_134, var_135);
            // if length_sitesite:                                                                <L 4324>
            if (var_135) {
                // wp.atomic_add(ten_length_out[worldid], tenid, length_sitesite * pulley_scale)       <L 4325>
                var_136 = wp::slice_t(var_0, var_0, var_137);
                var_138 = wp::view(var_ten_length_out, var_136);
                var_139 = wp::mul(var_135, var_9);
                var_140 = wp::atomic_add(var_138, var_6, var_139);
            }
            // if length_sitesite < MJ_MINVAL:                                                    <L 4328>
            var_141 = (var_135 < var_96);
            if (var_141) {
                // vec_sitesite = wp.vec3(1.0, 0.0, 0.0)                                          <L 4329>
                var_145 = wp::vec_t<3, wp::float32>(var_142, var_143, var_144);
            }
            var_146 = wp::where(var_141, var_145, var_134);
            // if bodyid_site0 != bodyid_site1:                                                   <L 4331>
            var_147 = (var_51 != var_57);
            if (var_147) {
                // offset_site0 = site_pnt0 - subtree_com_in[worldid, body_rootid[bodyid_site0]]       <L 4332>
                var_148 = wp::address(var_body_rootid, var_51);
                var_150 = wp::load(var_148);
                var_149 = wp::address(var_subtree_com_in, var_0, var_150);
                var_152 = wp::load(var_149);
                var_151 = wp::sub(var_27, var_152);
                // offset_site1 = site_pnt1 - subtree_com_in[worldid, body_rootid[bodyid_site1]]       <L 4333>
                var_153 = wp::address(var_body_rootid, var_57);
                var_155 = wp::load(var_153);
                var_154 = wp::address(var_subtree_com_in, var_0, var_155);
                var_157 = wp::load(var_154);
                var_156 = wp::sub(var_30, var_157);
                // _accumulate_jac_chain(                                                         <L 4334>
                // body_parentid,                                                                 <L 4335>
                // body_dofnum,                                                                   <L 4336>
                // body_dofadr,                                                                   <L 4337>
                // ten_J_colind,                                                                  <L 4338>
                // cdof_in,                                                                       <L 4339>
                // offset_site0,                                                                  <L 4340>
                // vec_sitesite,                                                                  <L 4341>
                // bodyid_site0,                                                                  <L 4342>
                // rowadr,                                                                        <L 4343>
                // rownnz,                                                                        <L 4344>
                // -pulley_scale,                                                                 <L 4345>
                var_158 = wp::neg(var_9);
                // worldid,                                                                       <L 4346>
                // ten_J_out,                                                                     <L 4347>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_151, var_146, var_51, var_79, var_76, var_158, var_0, var_ten_J_out);
                // _accumulate_jac_chain(                                                         <L 4349>
                // body_parentid,                                                                 <L 4350>
                // body_dofnum,                                                                   <L 4351>
                // body_dofadr,                                                                   <L 4352>
                // ten_J_colind,                                                                  <L 4353>
                // cdof_in,                                                                       <L 4354>
                // offset_site1,                                                                  <L 4355>
                // vec_sitesite,                                                                  <L 4356>
                // bodyid_site1,                                                                  <L 4357>
                // rowadr,                                                                        <L 4358>
                // rownnz,                                                                        <L 4359>
                // pulley_scale,                                                                  <L 4360>
                // worldid,                                                                       <L 4361>
                // ten_J_out,                                                                     <L 4362>
                _accumulate_jac_chain_0(var_body_parentid, var_body_dofnum, var_body_dofadr, var_ten_J_colind, var_cdof_in, var_156, var_146, var_57, var_79, var_76, var_9, var_0, var_ten_J_out);
            }
            var_159 = wp::where(var_147, var_151, var_114);
            var_160 = wp::where(var_147, var_156, var_130);
        }
        var_161 = wp::where(var_82, var_114, var_159);
        var_162 = wp::where(var_82, var_130, var_160);
    }
}



extern "C" __global__ void _geom_local_to_global_e28b714c_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_mocapid,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_pos,
    wp::array_t<wp::quat_t<wp::float32>> var_geom_quat,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        bool var_5;
        wp::int32* var_6;
        const wp::int32 var_7 = 0;
        bool var_8;
        wp::int32 var_9;
        wp::int32* var_10;
        wp::int32* var_11;
        wp::int32 var_12;
        const wp::int32 var_13 = -1;
        bool var_14;
        wp::int32 var_15;
        wp::vec_t<3, wp::float32>* var_16;
        wp::vec_t<3, wp::float32> var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::quat_t<wp::float32>* var_19;
        wp::quat_t<wp::float32> var_20;
        wp::quat_t<wp::float32> var_21;
        wp::shape_t* var_22;
        const wp::int32 var_23 = 0;
        wp::int32 var_24;
        wp::shape_t var_25;
        wp::int32 var_26;
        wp::vec_t<3, wp::float32>* var_27;
        wp::vec_t<3, wp::float32> var_28;
        wp::vec_t<3, wp::float32> var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::shape_t* var_31;
        const wp::int32 var_32 = 0;
        wp::int32 var_33;
        wp::shape_t var_34;
        wp::int32 var_35;
        wp::quat_t<wp::float32>* var_36;
        wp::quat_t<wp::float32> var_37;
        wp::quat_t<wp::float32> var_38;
        wp::mat_t<3, 3, wp::float32> var_39;
        //---------
        // forward
        // def _geom_local_to_global(                                                             <L 180>
        // worldid, geomid = wp.tid()                                                             <L 195>
        builtin_tid2d(var_0, var_1);
        // bodyid = geom_bodyid[geomid]                                                           <L 196>
        var_2 = wp::address(var_geom_bodyid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if body_weldid[bodyid] == 0 and body_mocapid[body_rootid[bodyid]] == -1:               <L 198>
        var_6 = wp::address(var_body_weldid, var_3);
        var_9 = wp::load(var_6);
        var_8 = (var_9 == var_7);
        var_5 = var_8;
        if (var_5) {
            var_10 = wp::address(var_body_rootid, var_3);
            var_12 = wp::load(var_10);
            var_11 = wp::address(var_body_mocapid, var_12);
            var_15 = wp::load(var_11);
            var_14 = (var_15 == var_13);
            var_5 = var_5 && var_14;
        }
        if (var_5) {
            // return                                                                             <L 201>
            continue;
        }
        // xpos = xpos_in[worldid, bodyid]                                                        <L 203>
        var_16 = wp::address(var_xpos_in, var_0, var_3);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // xquat = xquat_in[worldid, bodyid]                                                      <L 204>
        var_19 = wp::address(var_xquat_in, var_0, var_3);
        var_21 = wp::load(var_19);
        var_20 = wp::copy(var_21);
        // geom_xpos_out[worldid, geomid] = xpos + math.rot_vec_quat(geom_pos[worldid % geom_pos.shape[0], geomid], xquat)       <L 205>
        var_22 = &(var_geom_pos.shape);
        var_25 = wp::load(var_22);
        var_24 = wp::extract(var_25, var_23);
        var_26 = wp::mod(var_0, var_24);
        var_27 = wp::address(var_geom_pos, var_26, var_1);
        var_29 = wp::load(var_27);
        var_28 = rot_vec_quat_0(var_29, var_20);
        var_30 = wp::add(var_17, var_28);
        wp::array_store(var_geom_xpos_out, var_0, var_1, var_30);
        // geom_xmat_out[worldid, geomid] = math.quat_to_mat(math.mul_quat(xquat, geom_quat[worldid % geom_quat.shape[0], geomid]))       <L 206>
        var_31 = &(var_geom_quat.shape);
        var_34 = wp::load(var_31);
        var_33 = wp::extract(var_34, var_32);
        var_35 = wp::mod(var_0, var_33);
        var_36 = wp::address(var_geom_quat, var_35, var_1);
        var_38 = wp::load(var_36);
        var_37 = mul_quat_0(var_20, var_38);
        var_39 = quat_to_mat_0(var_37);
        wp::array_store(var_geom_xmat_out, var_0, var_1, var_39);
    }
}



extern "C" __global__ void _transmission_body_moment_scale_e0bec4b0_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::int32> var_actuator_trntype_body_adr,
    wp::array_t<wp::int32> var_moment_rowadr_in,
    wp::array_t<wp::int32> var_actuator_trntype_body_ncon_in,
    wp::array_t<wp::float32> var_actuator_moment_out)
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
        wp::int32 var_1;
        wp::int32 var_2;
        wp::int32* var_3;
        wp::int32 var_4;
        wp::int32 var_5;
        const wp::int32 var_6 = 0;
        bool var_7;
        wp::int32* var_8;
        wp::int32 var_9;
        wp::int32 var_10;
        wp::int32* var_11;
        wp::int32 var_12;
        wp::int32 var_13;
        wp::float32 var_14;
        wp::float32 var_15;
        wp::int32 var_16;
        wp::float32* var_17;
        wp::float32 var_18;
        wp::float32 var_19;
        //---------
        // forward
        // def _transmission_body_moment_scale(                                                   <L 3199>
        // worldid, trnbodyid, dofid = wp.tid()                                                   <L 3209>
        builtin_tid3d(var_0, var_1, var_2);
        // ncon = actuator_trntype_body_ncon_in[worldid, trnbodyid]                               <L 3211>
        var_3 = wp::address(var_actuator_trntype_body_ncon_in, var_0, var_1);
        var_5 = wp::load(var_3);
        var_4 = wp::copy(var_5);
        // if ncon > 0:                                                                           <L 3213>
        var_7 = (var_4 > var_6);
        if (var_7) {
            // actid = actuator_trntype_body_adr[trnbodyid]                                       <L 3214>
            var_8 = wp::address(var_actuator_trntype_body_adr, var_1);
            var_10 = wp::load(var_8);
            var_9 = wp::copy(var_10);
            // rowadr = moment_rowadr_in[worldid, actid]                                          <L 3215>
            var_11 = wp::address(var_moment_rowadr_in, var_0, var_9);
            var_13 = wp::load(var_11);
            var_12 = wp::copy(var_13);
            // actuator_moment_out[worldid, rowadr + dofid] /= -float(ncon)                       <L 3216>
            var_14 = wp::float(var_4);
            var_15 = wp::neg(var_14);
            var_16 = wp::add(var_12, var_2);
            var_17 = wp::address(var_actuator_moment_out, var_0, var_16);
            var_19 = wp::load(var_17);
            var_18 = wp::div(var_19, var_15);
            wp::array_store(var_actuator_moment_out, var_0, var_16, var_18);
        }
    }
}



extern "C" __global__ void _qfrc_bias_4085b6de_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cfrc_int_in,
    wp::array_t<wp::float32> var_qfrc_bias_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::vec_t<6, wp::float32>* var_5;
        wp::vec_t<6, wp::float32>* var_6;
        wp::float32 var_7;
        wp::vec_t<6, wp::float32> var_8;
        wp::vec_t<6, wp::float32> var_9;
        //---------
        // forward
        // def _qfrc_bias(                                                                        <L 1455>
        // worldid, dofid = wp.tid()                                                              <L 1464>
        builtin_tid2d(var_0, var_1);
        // bodyid = dof_bodyid[dofid]                                                             <L 1465>
        var_2 = wp::address(var_dof_bodyid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // qfrc_bias_out[worldid, dofid] = wp.dot(cdof_in[worldid, dofid], cfrc_int_in[worldid, bodyid])       <L 1466>
        var_5 = wp::address(var_cdof_in, var_0, var_1);
        var_6 = wp::address(var_cfrc_int_in, var_0, var_3);
        var_8 = wp::load(var_5);
        var_9 = wp::load(var_6);
        var_7 = wp::dot(var_8, var_9);
        wp::array_store(var_qfrc_bias_out, var_0, var_1, var_7);
    }
}



extern "C" __global__ void _spatial_tendon_wrap_2bb6c667_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::int32 var_ntendon,
    wp::array_t<wp::int32> var_tendon_adr,
    wp::array_t<wp::int32> var_tendon_num,
    wp::array_t<wp::int32> var_wrap_type,
    wp::array_t<wp::int32> var_wrap_objid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_wrap_geom_xpos_in,
    wp::array_t<wp::int32> var_ten_wrapadr_out,
    wp::array_t<wp::int32> var_ten_wrapnum_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_wrap_obj_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_wrap_xpos_out)
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
        wp::int32 var_2;
        const wp::int32 var_3 = 0;
        wp::int32 var_4;
        wp::range_t var_5;
        wp::int32 var_6;
        wp::int32* var_7;
        wp::int32 var_8;
        wp::int32 var_9;
        const wp::int32 var_10 = 0;
        wp::int32 var_11;
        wp::int32* var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        wp::int32* var_15;
        const wp::int32 var_16 = 1;
        bool var_17;
        wp::int32 var_18;
        const wp::int32 var_19 = 0;
        wp::int32 var_20;
        const wp::int32 var_21 = 1;
        wp::int32 var_22;
        bool var_23;
        wp::int32 var_24;
        const wp::int32 var_25 = 0;
        wp::int32 var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        wp::int32 var_30;
        const wp::int32 var_31 = 1;
        wp::int32 var_32;
        wp::int32* var_33;
        wp::int32 var_34;
        wp::int32 var_35;
        wp::int32 var_36;
        const wp::int32 var_37 = 0;
        wp::int32 var_38;
        wp::int32* var_39;
        wp::int32 var_40;
        wp::int32 var_41;
        wp::int32 var_42;
        const wp::int32 var_43 = 1;
        wp::int32 var_44;
        wp::int32* var_45;
        wp::int32 var_46;
        wp::int32 var_47;
        const wp::int32 var_48 = 2;
        bool var_49;
        bool var_50;
        const wp::int32 var_51 = 2;
        bool var_52;
        const wp::int32 var_53 = 2;
        wp::int32 var_54;
        const wp::int32 var_55 = 2;
        wp::int32 var_56;
        const wp::float32 var_57 = 0.0;
        const wp::int32 var_58 = 3;
        wp::int32 var_59;
        const wp::int32 var_60 = 0;
        wp::int32 var_61;
        const wp::float32 var_62 = 0.0;
        const wp::int32 var_63 = 3;
        wp::int32 var_64;
        const wp::int32 var_65 = 1;
        wp::int32 var_66;
        const wp::float32 var_67 = 0.0;
        const wp::int32 var_68 = 3;
        wp::int32 var_69;
        const wp::int32 var_70 = 2;
        wp::int32 var_71;
        const wp::int32 var_72 = -2;
        const wp::int32 var_73 = 1;
        wp::int32 var_74;
        const wp::int32 var_75 = 1;
        wp::int32 var_76;
        wp::int32 var_77;
        wp::int32 var_78;
        const wp::int32 var_79 = 1;
        wp::int32 var_80;
        wp::vec_t<3, wp::float32>* var_81;
        wp::vec_t<3, wp::float32> var_82;
        wp::vec_t<3, wp::float32> var_83;
        bool var_84;
        const wp::int32 var_85 = 4;
        bool var_86;
        const wp::int32 var_87 = 5;
        bool var_88;
        wp::vec_t<6, wp::float32>* var_89;
        wp::vec_t<6, wp::float32> var_90;
        wp::vec_t<6, wp::float32> var_91;
        wp::vec_t<3, wp::float32> var_92;
        const wp::int32 var_93 = 1;
        wp::int32 var_94;
        wp::int32 var_95;
        wp::int32 var_96;
        const wp::int32 var_97 = 2;
        wp::int32 var_98;
        wp::int32* var_99;
        wp::int32 var_100;
        wp::int32 var_101;
        wp::float32 var_102;
        const wp::float32 var_103 = 10000000000.0;
        bool var_104;
        wp::vec_t<3, wp::float32> var_105;
        wp::vec_t<3, wp::float32>* var_106;
        wp::vec_t<3, wp::float32> var_107;
        wp::vec_t<3, wp::float32> var_108;
        const wp::int32 var_109 = 0;
        wp::int32 var_110;
        const wp::int32 var_111 = 2;
        wp::int32 var_112;
        const wp::int32 var_113 = 0;
        wp::int32 var_114;
        const wp::int32 var_115 = 2;
        wp::int32 var_116;
        const wp::int32 var_117 = 1;
        wp::int32 var_118;
        const wp::int32 var_119 = 2;
        wp::int32 var_120;
        const wp::int32 var_121 = 1;
        wp::int32 var_122;
        const wp::int32 var_123 = 2;
        wp::int32 var_124;
        const wp::int32 var_125 = 2;
        wp::int32 var_126;
        const wp::int32 var_127 = 2;
        wp::int32 var_128;
        const wp::int32 var_129 = 2;
        wp::int32 var_130;
        const wp::int32 var_131 = 2;
        wp::int32 var_132;
        const wp::int32 var_133 = 3;
        wp::int32 var_134;
        const wp::int32 var_135 = 2;
        wp::int32 var_136;
        const wp::int32 var_137 = 3;
        wp::int32 var_138;
        const wp::int32 var_139 = 2;
        wp::int32 var_140;
        const wp::int32 var_141 = 0;
        wp::float32 var_142;
        const wp::int32 var_143 = 3;
        wp::int32 var_144;
        const wp::int32 var_145 = 0;
        wp::int32 var_146;
        const wp::int32 var_147 = 1;
        wp::float32 var_148;
        const wp::int32 var_149 = 3;
        wp::int32 var_150;
        const wp::int32 var_151 = 1;
        wp::int32 var_152;
        const wp::int32 var_153 = 2;
        wp::float32 var_154;
        const wp::int32 var_155 = 3;
        wp::int32 var_156;
        const wp::int32 var_157 = 2;
        wp::int32 var_158;
        const wp::int32 var_159 = 0;
        wp::float32 var_160;
        const wp::int32 var_161 = 3;
        wp::int32 var_162;
        const wp::int32 var_163 = 0;
        wp::int32 var_164;
        const wp::int32 var_165 = 1;
        wp::float32 var_166;
        const wp::int32 var_167 = 3;
        wp::int32 var_168;
        const wp::int32 var_169 = 1;
        wp::int32 var_170;
        const wp::int32 var_171 = 2;
        wp::float32 var_172;
        const wp::int32 var_173 = 3;
        wp::int32 var_174;
        const wp::int32 var_175 = 2;
        wp::int32 var_176;
        const wp::int32 var_177 = 0;
        wp::float32 var_178;
        const wp::int32 var_179 = 3;
        wp::int32 var_180;
        const wp::int32 var_181 = 0;
        wp::int32 var_182;
        const wp::int32 var_183 = 1;
        wp::float32 var_184;
        const wp::int32 var_185 = 3;
        wp::int32 var_186;
        const wp::int32 var_187 = 1;
        wp::int32 var_188;
        const wp::int32 var_189 = 2;
        wp::float32 var_190;
        const wp::int32 var_191 = 3;
        wp::int32 var_192;
        const wp::int32 var_193 = 2;
        wp::int32 var_194;
        const wp::int32 var_195 = 0;
        wp::float32 var_196;
        const wp::int32 var_197 = 3;
        wp::int32 var_198;
        const wp::int32 var_199 = 0;
        wp::int32 var_200;
        const wp::int32 var_201 = 1;
        wp::float32 var_202;
        const wp::int32 var_203 = 3;
        wp::int32 var_204;
        const wp::int32 var_205 = 1;
        wp::int32 var_206;
        const wp::int32 var_207 = 2;
        wp::float32 var_208;
        const wp::int32 var_209 = 3;
        wp::int32 var_210;
        const wp::int32 var_211 = 2;
        wp::int32 var_212;
        const wp::int32 var_213 = -1;
        const wp::int32 var_214 = 3;
        wp::int32 var_215;
        const wp::int32 var_216 = 3;
        wp::int32 var_217;
        const wp::int32 var_218 = 2;
        wp::int32 var_219;
        wp::int32 var_220;
        wp::int32 var_221;
        wp::int32 var_222;
        const wp::int32 var_223 = 0;
        wp::int32 var_224;
        const wp::int32 var_225 = 2;
        wp::int32 var_226;
        const wp::int32 var_227 = 0;
        wp::int32 var_228;
        const wp::int32 var_229 = 2;
        wp::int32 var_230;
        const wp::int32 var_231 = 0;
        wp::float32 var_232;
        const wp::int32 var_233 = 3;
        wp::int32 var_234;
        const wp::int32 var_235 = 0;
        wp::int32 var_236;
        const wp::int32 var_237 = 1;
        wp::float32 var_238;
        const wp::int32 var_239 = 3;
        wp::int32 var_240;
        const wp::int32 var_241 = 1;
        wp::int32 var_242;
        const wp::int32 var_243 = 2;
        wp::float32 var_244;
        const wp::int32 var_245 = 3;
        wp::int32 var_246;
        const wp::int32 var_247 = 2;
        wp::int32 var_248;
        const wp::int32 var_249 = -1;
        const wp::int32 var_250 = 1;
        wp::int32 var_251;
        const wp::int32 var_252 = 1;
        wp::int32 var_253;
        const wp::int32 var_254 = 2;
        wp::int32 var_255;
        wp::int32 var_256;
        wp::int32 var_257;
        wp::int32 var_258;
        wp::int32 var_259;
        wp::int32 var_260;
        wp::int32 var_261;
        wp::int32 var_262;
        wp::int32 var_263;
        wp::int32 var_264;
        wp::int32 var_265;
        const wp::int32 var_266 = 0;
        wp::int32 var_267;
        const wp::int32 var_268 = 2;
        wp::int32 var_269;
        const wp::int32 var_270 = 0;
        wp::int32 var_271;
        const wp::int32 var_272 = 2;
        wp::int32 var_273;
        const wp::int32 var_274 = 0;
        wp::float32 var_275;
        const wp::int32 var_276 = 3;
        wp::int32 var_277;
        const wp::int32 var_278 = 0;
        wp::int32 var_279;
        const wp::int32 var_280 = 1;
        wp::float32 var_281;
        const wp::int32 var_282 = 3;
        wp::int32 var_283;
        const wp::int32 var_284 = 1;
        wp::int32 var_285;
        const wp::int32 var_286 = 2;
        wp::float32 var_287;
        const wp::int32 var_288 = 3;
        wp::int32 var_289;
        const wp::int32 var_290 = 2;
        wp::int32 var_291;
        const wp::int32 var_292 = -1;
        const wp::int32 var_293 = 1;
        wp::int32 var_294;
        const wp::int32 var_295 = 1;
        wp::int32 var_296;
        const wp::int32 var_297 = 1;
        wp::int32 var_298;
        wp::int32 var_299;
        wp::int32 var_300;
        wp::int32 var_301;
        wp::int32 var_302;
        wp::int32 var_303;
        wp::int32 var_304;
        const wp::int32 var_305 = 1;
        wp::int32 var_306;
        wp::shape_t* var_307;
        const wp::int32 var_308 = 0;
        wp::int32 var_309;
        wp::shape_t var_310;
        bool var_311;
        wp::int32 var_312;
        const wp::int32 var_313 = 1;
        wp::int32 var_314;
        wp::int32* var_315;
        const wp::int32 var_316 = 2;
        bool var_317;
        wp::int32 var_318;
        const bool var_319 = false;
        bool var_320;
        bool var_321;
        const wp::int32 var_322 = 1;
        wp::int32 var_323;
        bool var_324;
        const wp::int32 var_325 = 0;
        wp::int32 var_326;
        const wp::int32 var_327 = 2;
        wp::int32 var_328;
        const wp::int32 var_329 = 0;
        wp::int32 var_330;
        const wp::int32 var_331 = 2;
        wp::int32 var_332;
        wp::vec_t<3, wp::float32>* var_333;
        wp::vec_t<3, wp::float32> var_334;
        wp::vec_t<3, wp::float32> var_335;
        const wp::int32 var_336 = 0;
        wp::float32 var_337;
        const wp::int32 var_338 = 3;
        wp::int32 var_339;
        const wp::int32 var_340 = 0;
        wp::int32 var_341;
        const wp::int32 var_342 = 1;
        wp::float32 var_343;
        const wp::int32 var_344 = 3;
        wp::int32 var_345;
        const wp::int32 var_346 = 1;
        wp::int32 var_347;
        const wp::int32 var_348 = 2;
        wp::float32 var_349;
        const wp::int32 var_350 = 3;
        wp::int32 var_351;
        const wp::int32 var_352 = 2;
        wp::int32 var_353;
        const wp::int32 var_354 = -1;
        const wp::int32 var_355 = 1;
        wp::int32 var_356;
        const wp::int32 var_357 = 1;
        wp::int32 var_358;
        wp::int32 var_359;
        wp::int32 var_360;
        wp::vec_t<3, wp::float32> var_361;
        wp::int32 var_362;
        wp::int32 var_363;
        //---------
        // forward
        // def _spatial_tendon_wrap(                                                              <L 4367>
        // worldid = wp.tid()                                                                     <L 4384>
        var_0 = builtin_tid1d();
        // wrapcount = int(0)                                                                     <L 4386>
        var_2 = wp::int(var_1);
        // wrapgeomid = int(0)                                                                    <L 4387>
        var_4 = wp::int(var_3);
        // for i in range(ntendon):                                                               <L 4388>
        var_5 = wp::range(var_ntendon);
        start_for_0:;
            if (iter_cmp(var_5) == 0) goto end_for_0;
            var_6 = wp::iter_next(var_5);
            // adr = tendon_adr[i]                                                                <L 4389>
            var_7 = wp::address(var_tendon_adr, var_6);
            var_9 = wp::load(var_7);
            var_8 = wp::copy(var_9);
            // ten_wrapadr_out[worldid, i] = wrapcount                                            <L 4390>
            wp::array_store(var_ten_wrapadr_out, var_0, var_6, var_2);
            // wrapnum = int(0)                                                                   <L 4391>
            var_11 = wp::int(var_10);
            // tendonnum = tendon_num[i]                                                          <L 4392>
            var_12 = wp::address(var_tendon_num, var_6);
            var_14 = wp::load(var_12);
            var_13 = wp::copy(var_14);
            // if wrap_type[adr] == WrapType.JOINT:                                               <L 4395>
            var_15 = wp::address(var_wrap_type, var_8);
            var_18 = wp::load(var_15);
            var_17 = (var_18 == var_16);
            if (var_17) {
                // continue                                                                       <L 4396>
                goto start_for_0;
            }
            // j = int(0)                                                                         <L 4399>
            var_20 = wp::int(var_19);
            // while j < tendonnum - 1:                                                           <L 4400>
        start_while_2:;
            var_22 = wp::sub(var_13, var_21);
            var_23 = (var_20 < var_22);
        if ((var_23) == false) goto end_while_2;
                // type0 = wrap_type[adr + j + 0]                                                 <L 4402>
                var_24 = wp::add(var_8, var_20);
                var_26 = wp::add(var_24, var_25);
                var_27 = wp::address(var_wrap_type, var_26);
                var_29 = wp::load(var_27);
                var_28 = wp::copy(var_29);
                // type1 = wrap_type[adr + j + 1]                                                 <L 4403>
                var_30 = wp::add(var_8, var_20);
                var_32 = wp::add(var_30, var_31);
                var_33 = wp::address(var_wrap_type, var_32);
                var_35 = wp::load(var_33);
                var_34 = wp::copy(var_35);
                // id0 = wrap_objid[adr + j + 0]                                                  <L 4404>
                var_36 = wp::add(var_8, var_20);
                var_38 = wp::add(var_36, var_37);
                var_39 = wp::address(var_wrap_objid, var_38);
                var_41 = wp::load(var_39);
                var_40 = wp::copy(var_41);
                // id1 = wrap_objid[adr + j + 1]                                                  <L 4405>
                var_42 = wp::add(var_8, var_20);
                var_44 = wp::add(var_42, var_43);
                var_45 = wp::address(var_wrap_objid, var_44);
                var_47 = wp::load(var_45);
                var_46 = wp::copy(var_47);
                // pulley0 = type0 == WrapType.PULLEY                                             <L 4408>
                var_49 = (var_28 == var_48);
                // if pulley0 or type1 == WrapType.PULLEY:                                        <L 4409>
                var_50 = var_49;
                if (!var_50) {
                    var_52 = (var_34 == var_51);
                    var_50 = var_50 || var_52;
                }
                if (var_50) {
                    // if pulley0:                                                                <L 4410>
                    if (var_49) {
                        // row = wrapcount // 2                                                   <L 4411>
                        var_54 = wp::floordiv(var_2, var_53);
                        // col = wrapcount % 2                                                    <L 4412>
                        var_56 = wp::mod(var_2, var_55);
                        // wrap_xpos_out[worldid, row][3 * col + 0] = 0.0                         <L 4413>
                        var_59 = wp::mul(var_58, var_56);
                        var_61 = wp::add(var_59, var_60);
                        wp::index(var_wrap_xpos_out, var_0, var_54)[var_61] = var_57;
                        // wrap_xpos_out[worldid, row][3 * col + 1] = 0.0                         <L 4414>
                        var_64 = wp::mul(var_63, var_56);
                        var_66 = wp::add(var_64, var_65);
                        wp::index(var_wrap_xpos_out, var_0, var_54)[var_66] = var_62;
                        // wrap_xpos_out[worldid, row][3 * col + 2] = 0.0                         <L 4415>
                        var_69 = wp::mul(var_68, var_56);
                        var_71 = wp::add(var_69, var_70);
                        wp::index(var_wrap_xpos_out, var_0, var_54)[var_71] = var_67;
                        // wrap_obj_out[worldid, row][col] = -2                                   <L 4417>
                        wp::index(var_wrap_obj_out, var_0, var_54)[var_56] = var_72;
                        // wrapnum += 1                                                           <L 4419>
                        var_74 = wp::add(var_11, var_73);
                        // wrapcount += 1                                                         <L 4420>
                        var_76 = wp::add(var_2, var_75);
                    }
                    var_77 = wp::where(var_49, var_76, var_2);
                    var_78 = wp::where(var_49, var_74, var_11);
                    // j += 1                                                                     <L 4423>
                    var_80 = wp::add(var_20, var_79);
                    // continue                                                                   <L 4424>
                    wp::assign(var_2, var_77);
                    wp::assign(var_11, var_78);
                    wp::assign(var_20, var_80);
                    goto start_while_2;
                }
                // wpnt_site0 = site_xpos_in[worldid, id0]                                        <L 4427>
                var_81 = wp::address(var_site_xpos_in, var_0, var_40);
                var_83 = wp::load(var_81);
                var_82 = wp::copy(var_83);
                // if type1 == WrapType.SPHERE or type1 == WrapType.CYLINDER:                     <L 4430>
                var_86 = (var_34 == var_85);
                var_84 = var_86;
                if (!var_84) {
                    var_88 = (var_34 == var_87);
                    var_84 = var_84 || var_88;
                }
                if (var_84) {
                    // wrap_geom_xpos = wrap_geom_xpos_in[worldid, wrapgeomid]                    <L 4431>
                    var_89 = wp::address(var_wrap_geom_xpos_in, var_0, var_4);
                    var_91 = wp::load(var_89);
                    var_90 = wp::copy(var_91);
                    // wpnt_geom0 = wp.spatial_top(wrap_geom_xpos)                                <L 4432>
                    var_92 = wp::spatial_top(var_90);
                    // wrapgeomid += 1                                                            <L 4433>
                    var_94 = wp::add(var_4, var_93);
                    // wrapid = id1                                                               <L 4435>
                    var_95 = wp::copy(var_46);
                    // id1 = wrap_objid[adr + j + 2]                                              <L 4436>
                    var_96 = wp::add(var_8, var_20);
                    var_98 = wp::add(var_96, var_97);
                    var_99 = wp::address(var_wrap_objid, var_98);
                    var_101 = wp::load(var_99);
                    var_100 = wp::copy(var_101);
                    // if wp.norm_l2(wpnt_geom0) < MJ_MAXVAL:                                     <L 4437>
                    var_102 = norm_l2_0(var_92);
                    var_104 = (var_102 < var_103);
                    if (var_104) {
                        // wpnt_geom1 = wp.spatial_bottom(wrap_geom_xpos)                         <L 4438>
                        var_105 = wp::spatial_bottom(var_90);
                        // wpnt_site1 = site_xpos_in[worldid, id1]                                <L 4439>
                        var_106 = wp::address(var_site_xpos_in, var_0, var_100);
                        var_108 = wp::load(var_106);
                        var_107 = wp::copy(var_108);
                        // row0 = (wrapcount + 0) // 2                                            <L 4442>
                        var_110 = wp::add(var_2, var_109);
                        var_112 = wp::floordiv(var_110, var_111);
                        // col0 = (wrapcount + 0) % 2                                             <L 4443>
                        var_114 = wp::add(var_2, var_113);
                        var_116 = wp::mod(var_114, var_115);
                        // row1 = (wrapcount + 1) // 2                                            <L 4444>
                        var_118 = wp::add(var_2, var_117);
                        var_120 = wp::floordiv(var_118, var_119);
                        // col1 = (wrapcount + 1) % 2                                             <L 4445>
                        var_122 = wp::add(var_2, var_121);
                        var_124 = wp::mod(var_122, var_123);
                        // row2 = (wrapcount + 2) // 2                                            <L 4446>
                        var_126 = wp::add(var_2, var_125);
                        var_128 = wp::floordiv(var_126, var_127);
                        // col2 = (wrapcount + 2) % 2                                             <L 4447>
                        var_130 = wp::add(var_2, var_129);
                        var_132 = wp::mod(var_130, var_131);
                        // row3 = (wrapcount + 3) // 2                                            <L 4448>
                        var_134 = wp::add(var_2, var_133);
                        var_136 = wp::floordiv(var_134, var_135);
                        // col3 = (wrapcount + 3) % 2                                             <L 4449>
                        var_138 = wp::add(var_2, var_137);
                        var_140 = wp::mod(var_138, var_139);
                        // wrap_xpos_out[worldid, row0][3 * col0 + 0] = wpnt_site0[0]             <L 4451>
                        var_142 = wp::extract(var_82, var_141);
                        var_144 = wp::mul(var_143, var_116);
                        var_146 = wp::add(var_144, var_145);
                        wp::index(var_wrap_xpos_out, var_0, var_112)[var_146] = var_142;
                        // wrap_xpos_out[worldid, row0][3 * col0 + 1] = wpnt_site0[1]             <L 4452>
                        var_148 = wp::extract(var_82, var_147);
                        var_150 = wp::mul(var_149, var_116);
                        var_152 = wp::add(var_150, var_151);
                        wp::index(var_wrap_xpos_out, var_0, var_112)[var_152] = var_148;
                        // wrap_xpos_out[worldid, row0][3 * col0 + 2] = wpnt_site0[2]             <L 4453>
                        var_154 = wp::extract(var_82, var_153);
                        var_156 = wp::mul(var_155, var_116);
                        var_158 = wp::add(var_156, var_157);
                        wp::index(var_wrap_xpos_out, var_0, var_112)[var_158] = var_154;
                        // wrap_xpos_out[worldid, row1][3 * col1 + 0] = wpnt_geom0[0]             <L 4455>
                        var_160 = wp::extract(var_92, var_159);
                        var_162 = wp::mul(var_161, var_124);
                        var_164 = wp::add(var_162, var_163);
                        wp::index(var_wrap_xpos_out, var_0, var_120)[var_164] = var_160;
                        // wrap_xpos_out[worldid, row1][3 * col1 + 1] = wpnt_geom0[1]             <L 4456>
                        var_166 = wp::extract(var_92, var_165);
                        var_168 = wp::mul(var_167, var_124);
                        var_170 = wp::add(var_168, var_169);
                        wp::index(var_wrap_xpos_out, var_0, var_120)[var_170] = var_166;
                        // wrap_xpos_out[worldid, row1][3 * col1 + 2] = wpnt_geom0[2]             <L 4457>
                        var_172 = wp::extract(var_92, var_171);
                        var_174 = wp::mul(var_173, var_124);
                        var_176 = wp::add(var_174, var_175);
                        wp::index(var_wrap_xpos_out, var_0, var_120)[var_176] = var_172;
                        // wrap_xpos_out[worldid, row2][3 * col2 + 0] = wpnt_geom1[0]             <L 4459>
                        var_178 = wp::extract(var_105, var_177);
                        var_180 = wp::mul(var_179, var_132);
                        var_182 = wp::add(var_180, var_181);
                        wp::index(var_wrap_xpos_out, var_0, var_128)[var_182] = var_178;
                        // wrap_xpos_out[worldid, row2][3 * col2 + 1] = wpnt_geom1[1]             <L 4460>
                        var_184 = wp::extract(var_105, var_183);
                        var_186 = wp::mul(var_185, var_132);
                        var_188 = wp::add(var_186, var_187);
                        wp::index(var_wrap_xpos_out, var_0, var_128)[var_188] = var_184;
                        // wrap_xpos_out[worldid, row2][3 * col2 + 2] = wpnt_geom1[2]             <L 4461>
                        var_190 = wp::extract(var_105, var_189);
                        var_192 = wp::mul(var_191, var_132);
                        var_194 = wp::add(var_192, var_193);
                        wp::index(var_wrap_xpos_out, var_0, var_128)[var_194] = var_190;
                        // wrap_xpos_out[worldid, row3][3 * col3 + 0] = wpnt_site1[0]             <L 4463>
                        var_196 = wp::extract(var_107, var_195);
                        var_198 = wp::mul(var_197, var_140);
                        var_200 = wp::add(var_198, var_199);
                        wp::index(var_wrap_xpos_out, var_0, var_136)[var_200] = var_196;
                        // wrap_xpos_out[worldid, row3][3 * col3 + 1] = wpnt_site1[1]             <L 4464>
                        var_202 = wp::extract(var_107, var_201);
                        var_204 = wp::mul(var_203, var_140);
                        var_206 = wp::add(var_204, var_205);
                        wp::index(var_wrap_xpos_out, var_0, var_136)[var_206] = var_202;
                        // wrap_xpos_out[worldid, row3][3 * col3 + 2] = wpnt_site1[2]             <L 4465>
                        var_208 = wp::extract(var_107, var_207);
                        var_210 = wp::mul(var_209, var_140);
                        var_212 = wp::add(var_210, var_211);
                        wp::index(var_wrap_xpos_out, var_0, var_136)[var_212] = var_208;
                        // wrap_obj_out[worldid, row0][col0] = -1                                 <L 4467>
                        wp::index(var_wrap_obj_out, var_0, var_112)[var_116] = var_213;
                        // wrap_obj_out[worldid, row1][col1] = wrapid                             <L 4468>
                        wp::index(var_wrap_obj_out, var_0, var_120)[var_124] = var_95;
                        // wrap_obj_out[worldid, row2][col2] = wrapid                             <L 4469>
                        wp::index(var_wrap_obj_out, var_0, var_128)[var_132] = var_95;
                        // wrapnum += 3                                                           <L 4471>
                        var_215 = wp::add(var_11, var_214);
                        // wrapcount += 3                                                         <L 4472>
                        var_217 = wp::add(var_2, var_216);
                        // j += 2                                                                 <L 4473>
                        var_219 = wp::add(var_20, var_218);
                    }
                    var_220 = wp::where(var_104, var_217, var_2);
                    var_221 = wp::where(var_104, var_215, var_11);
                    var_222 = wp::where(var_104, var_219, var_20);
                    if (!var_104) {
                        // row0 = (wrapcount + 0) // 2                                            <L 4476>
                        var_224 = wp::add(var_220, var_223);
                        var_226 = wp::floordiv(var_224, var_225);
                        // col0 = (wrapcount + 0) % 2                                             <L 4477>
                        var_228 = wp::add(var_220, var_227);
                        var_230 = wp::mod(var_228, var_229);
                        // wrap_xpos_out[worldid, row0][3 * col0 + 0] = wpnt_site0[0]             <L 4479>
                        var_232 = wp::extract(var_82, var_231);
                        var_234 = wp::mul(var_233, var_230);
                        var_236 = wp::add(var_234, var_235);
                        wp::index(var_wrap_xpos_out, var_0, var_226)[var_236] = var_232;
                        // wrap_xpos_out[worldid, row0][3 * col0 + 1] = wpnt_site0[1]             <L 4480>
                        var_238 = wp::extract(var_82, var_237);
                        var_240 = wp::mul(var_239, var_230);
                        var_242 = wp::add(var_240, var_241);
                        wp::index(var_wrap_xpos_out, var_0, var_226)[var_242] = var_238;
                        // wrap_xpos_out[worldid, row0][3 * col0 + 2] = wpnt_site0[2]             <L 4481>
                        var_244 = wp::extract(var_82, var_243);
                        var_246 = wp::mul(var_245, var_230);
                        var_248 = wp::add(var_246, var_247);
                        wp::index(var_wrap_xpos_out, var_0, var_226)[var_248] = var_244;
                        // wrap_obj_out[worldid, row0][col0] = -1                                 <L 4483>
                        wp::index(var_wrap_obj_out, var_0, var_226)[var_230] = var_249;
                        // wrapnum += 1                                                           <L 4485>
                        var_251 = wp::add(var_221, var_250);
                        // wrapcount += 1                                                         <L 4486>
                        var_253 = wp::add(var_220, var_252);
                        // j += 2                                                                 <L 4487>
                        var_255 = wp::add(var_222, var_254);
                    }
                    var_256 = wp::where(var_104, var_220, var_253);
                    var_257 = wp::where(var_104, var_221, var_251);
                    var_258 = wp::where(var_104, var_222, var_255);
                    var_259 = wp::where(var_104, var_112, var_226);
                    var_260 = wp::where(var_104, var_116, var_230);
                }
                var_261 = wp::where(var_84, var_256, var_2);
                var_262 = wp::where(var_84, var_94, var_4);
                var_263 = wp::where(var_84, var_257, var_11);
                var_264 = wp::where(var_84, var_258, var_20);
                var_265 = wp::where(var_84, var_100, var_46);
                if (!var_84) {
                    // row0 = (wrapcount + 0) // 2                                                <L 4490>
                    var_267 = wp::add(var_261, var_266);
                    var_269 = wp::floordiv(var_267, var_268);
                    // col0 = (wrapcount + 0) % 2                                                 <L 4491>
                    var_271 = wp::add(var_261, var_270);
                    var_273 = wp::mod(var_271, var_272);
                    // wrap_xpos_out[worldid, row0][3 * col0 + 0] = wpnt_site0[0]                 <L 4493>
                    var_275 = wp::extract(var_82, var_274);
                    var_277 = wp::mul(var_276, var_273);
                    var_279 = wp::add(var_277, var_278);
                    wp::index(var_wrap_xpos_out, var_0, var_269)[var_279] = var_275;
                    // wrap_xpos_out[worldid, row0][3 * col0 + 1] = wpnt_site0[1]                 <L 4494>
                    var_281 = wp::extract(var_82, var_280);
                    var_283 = wp::mul(var_282, var_273);
                    var_285 = wp::add(var_283, var_284);
                    wp::index(var_wrap_xpos_out, var_0, var_269)[var_285] = var_281;
                    // wrap_xpos_out[worldid, row0][3 * col0 + 2] = wpnt_site0[2]                 <L 4495>
                    var_287 = wp::extract(var_82, var_286);
                    var_289 = wp::mul(var_288, var_273);
                    var_291 = wp::add(var_289, var_290);
                    wp::index(var_wrap_xpos_out, var_0, var_269)[var_291] = var_287;
                    // wrap_obj_out[worldid, row0][col0] = -1                                     <L 4497>
                    wp::index(var_wrap_obj_out, var_0, var_269)[var_273] = var_292;
                    // wrapnum += 1                                                               <L 4499>
                    var_294 = wp::add(var_263, var_293);
                    // wrapcount += 1                                                             <L 4500>
                    var_296 = wp::add(var_261, var_295);
                    // j += 1                                                                     <L 4501>
                    var_298 = wp::add(var_264, var_297);
                }
                var_299 = wp::where(var_84, var_261, var_296);
                var_300 = wp::where(var_84, var_263, var_294);
                var_301 = wp::where(var_84, var_264, var_298);
                var_302 = wp::where(var_84, var_259, var_269);
                var_303 = wp::where(var_84, var_260, var_273);
                // if adr + j + 1 < wrap_type.shape[0]:                                           <L 4504>
                var_304 = wp::add(var_8, var_301);
                var_306 = wp::add(var_304, var_305);
                var_307 = &(var_wrap_type.shape);
                var_310 = wp::load(var_307);
                var_309 = wp::extract(var_310, var_308);
                var_311 = (var_306 < var_309);
                if (var_311) {
                    // last_before_pulley = wrap_type[adr + j + 1] == WrapType.PULLEY             <L 4505>
                    var_312 = wp::add(var_8, var_301);
                    var_314 = wp::add(var_312, var_313);
                    var_315 = wp::address(var_wrap_type, var_314);
                    var_318 = wp::load(var_315);
                    var_317 = (var_318 == var_316);
                }
                if (!var_311) {
                    // last_before_pulley = False                                                 <L 4507>
                }
                var_320 = wp::where(var_311, var_317, var_319);
                // if j == tendonnum - 1 or last_before_pulley:                                   <L 4509>
                var_323 = wp::sub(var_13, var_322);
                var_324 = (var_301 == var_323);
                var_321 = var_324;
                if (!var_321) {
                    var_321 = var_321 || var_320;
                }
                if (var_321) {
                    // row0 = (wrapcount + 0) // 2                                                <L 4510>
                    var_326 = wp::add(var_299, var_325);
                    var_328 = wp::floordiv(var_326, var_327);
                    // col0 = (wrapcount + 0) % 2                                                 <L 4511>
                    var_330 = wp::add(var_299, var_329);
                    var_332 = wp::mod(var_330, var_331);
                    // wpnt_site1 = site_xpos_in[worldid, id1]                                    <L 4513>
                    var_333 = wp::address(var_site_xpos_in, var_0, var_265);
                    var_335 = wp::load(var_333);
                    var_334 = wp::copy(var_335);
                    // wrap_xpos_out[worldid, row0][3 * col0 + 0] = wpnt_site1[0]                 <L 4514>
                    var_337 = wp::extract(var_334, var_336);
                    var_339 = wp::mul(var_338, var_332);
                    var_341 = wp::add(var_339, var_340);
                    wp::index(var_wrap_xpos_out, var_0, var_328)[var_341] = var_337;
                    // wrap_xpos_out[worldid, row0][3 * col0 + 1] = wpnt_site1[1]                 <L 4515>
                    var_343 = wp::extract(var_334, var_342);
                    var_345 = wp::mul(var_344, var_332);
                    var_347 = wp::add(var_345, var_346);
                    wp::index(var_wrap_xpos_out, var_0, var_328)[var_347] = var_343;
                    // wrap_xpos_out[worldid, row0][3 * col0 + 2] = wpnt_site1[2]                 <L 4516>
                    var_349 = wp::extract(var_334, var_348);
                    var_351 = wp::mul(var_350, var_332);
                    var_353 = wp::add(var_351, var_352);
                    wp::index(var_wrap_xpos_out, var_0, var_328)[var_353] = var_349;
                    // wrap_obj_out[worldid, row0][col0] = -1                                     <L 4518>
                    wp::index(var_wrap_obj_out, var_0, var_328)[var_332] = var_354;
                    // wrapnum += 1                                                               <L 4519>
                    var_356 = wp::add(var_300, var_355);
                    // wrapcount += 1                                                             <L 4520>
                    var_358 = wp::add(var_299, var_357);
                }
                var_359 = wp::where(var_321, var_358, var_299);
                var_360 = wp::where(var_321, var_356, var_300);
                var_361 = wp::where(var_321, var_334, var_107);
                var_362 = wp::where(var_321, var_328, var_302);
                var_363 = wp::where(var_321, var_332, var_303);
                wp::assign(var_2, var_359);
                wp::assign(var_4, var_262);
                wp::assign(var_11, var_360);
                wp::assign(var_20, var_301);
        goto start_while_2;
        end_while_2:;
            // ten_wrapnum_out[worldid, i] = wrapnum                                              <L 4522>
            wp::array_store(var_ten_wrapnum_out, var_0, var_6, var_11);
            goto start_for_0;
        end_for_0:;
    }
}



extern "C" __global__ void _tendon_bias_qfrc_2ae21571_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_armature,
    wp::array_t<wp::float32> var_ten_J_in,
    wp::array_t<wp::float32> var_ten_bias_coef_in,
    wp::array_t<wp::float32> var_qfrc_out)
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
        wp::int32 var_1;
        wp::int32 var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::float32 var_9;
        wp::float32 var_10;
        const wp::float32 var_11 = 0.0;
        bool var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        bool var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::float32* var_21;
        wp::float32 var_22;
        wp::float32 var_23;
        const wp::float32 var_24 = 0.0;
        bool var_25;
        wp::int32* var_26;
        wp::int32 var_27;
        wp::int32 var_28;
        wp::slice_t var_29;
        const wp::int32 var_30 = 0;
        wp::array_t<wp::float32> var_31;
        wp::float32 var_32;
        wp::float32* var_33;
        wp::float32 var_34;
        wp::float32 var_35;
        wp::float32 var_36;
        //---------
        // forward
        // def _tendon_bias_qfrc(                                                                 <L 2416>
        // worldid, tenid, dofid = wp.tid()                                                       <L 2429>
        builtin_tid3d(var_0, var_1, var_2);
        // armature = tendon_armature[worldid % tendon_armature.shape[0], tenid]                  <L 2431>
        var_3 = &(var_tendon_armature.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_tendon_armature, var_7, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // if armature == 0.0:                                                                    <L 2432>
        var_12 = (var_9 == var_11);
        if (var_12) {
            // return                                                                             <L 2433>
            continue;
        }
        // rownnz = ten_J_rownnz[tenid]                                                           <L 2435>
        var_13 = wp::address(var_ten_J_rownnz, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // if dofid >= rownnz:                                                                    <L 2436>
        var_16 = (var_2 >= var_14);
        if (var_16) {
            // return                                                                             <L 2437>
            continue;
        }
        // rowadr = ten_J_rowadr[tenid]                                                           <L 2438>
        var_17 = wp::address(var_ten_J_rowadr, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // sparseid = rowadr + dofid                                                              <L 2439>
        var_20 = wp::add(var_18, var_2);
        // ten_J = ten_J_in[worldid, sparseid]                                                    <L 2440>
        var_21 = wp::address(var_ten_J_in, var_0, var_20);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // if ten_J == 0.0:                                                                       <L 2442>
        var_25 = (var_22 == var_24);
        if (var_25) {
            // return                                                                             <L 2443>
            continue;
        }
        // dofid = ten_J_colind[sparseid]                                                         <L 2445>
        var_26 = wp::address(var_ten_J_colind, var_20);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // wp.atomic_add(qfrc_out[worldid], dofid, ten_J * armature * ten_bias_coef_in[worldid, tenid])       <L 2447>
        var_29 = wp::slice_t(var_0, var_0, var_30);
        var_31 = wp::view(var_qfrc_out, var_29);
        var_32 = wp::mul(var_22, var_9);
        var_33 = wp::address(var_ten_bias_coef_in, var_0, var_1);
        var_35 = wp::load(var_33);
        var_34 = wp::mul(var_32, var_35);
        var_36 = wp::atomic_add(var_31, var_27, var_34);
    }
}



extern "C" __global__ void _flex_nodes_7c3ae771_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nflex,
    wp::array_t<wp::int32> var_flex_nodeadr,
    wp::array_t<wp::int32> var_flex_nodenum,
    wp::array_t<wp::int32> var_flex_nodebodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_node,
    wp::array_t<bool> var_flex_centered,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_out)
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
        wp::int32 var_1;
        wp::range_t var_2;
        wp::int32 var_3;
        wp::int32* var_4;
        wp::int32 var_5;
        wp::int32 var_6;
        bool var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::int32* var_10;
        bool var_11;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        wp::vec_t<3, wp::float32>* var_16;
        wp::vec_t<3, wp::float32> var_17;
        wp::vec_t<3, wp::float32> var_18;
        bool* var_19;
        bool var_20;
        bool var_21;
        bool var_22;
        wp::vec_t<3, wp::float32>* var_23;
        wp::vec_t<3, wp::float32> var_24;
        wp::vec_t<3, wp::float32> var_25;
        bool var_26;
        const wp::int32 var_27 = 0;
        wp::float32 var_28;
        const wp::float32 var_29 = 0.0;
        bool var_30;
        const wp::int32 var_31 = 1;
        wp::float32 var_32;
        const wp::float32 var_33 = 0.0;
        bool var_34;
        const wp::int32 var_35 = 2;
        wp::float32 var_36;
        const wp::float32 var_37 = 0.0;
        bool var_38;
        wp::mat_t<3, 3, wp::float32>* var_39;
        wp::mat_t<3, 3, wp::float32> var_40;
        wp::mat_t<3, 3, wp::float32> var_41;
        wp::vec_t<3, wp::float32> var_42;
        wp::vec_t<3, wp::float32> var_43;
        bool var_44;
        //---------
        // forward
        // def _flex_nodes(                                                                       <L 311>
        // worldid, nodeid = wp.tid()                                                             <L 325>
        builtin_tid2d(var_0, var_1);
        // for f in range(nflex):                                                                 <L 327>
        var_2 = wp::range(var_nflex);
        start_for_0:;
            if (iter_cmp(var_2) == 0) goto end_for_0;
            var_3 = wp::iter_next(var_2);
            // locid = nodeid - flex_nodeadr[f]                                                   <L 328>
            var_4 = wp::address(var_flex_nodeadr, var_3);
            var_6 = wp::load(var_4);
            var_5 = wp::sub(var_1, var_6);
            // if locid >= 0 and locid < flex_nodenum[f]:                                         <L 329>
            var_9 = (var_5 >= var_8);
            var_7 = var_9;
            if (var_7) {
                var_10 = wp::address(var_flex_nodenum, var_3);
                var_12 = wp::load(var_10);
                var_11 = (var_5 < var_12);
                var_7 = var_7 && var_11;
            }
            if (var_7) {
                // break                                                                          <L 330>
                goto end_for_0;
            }
            goto start_for_0;
        end_for_0:;
        // bodyid = flex_nodebodyid[nodeid]                                                       <L 332>
        var_13 = wp::address(var_flex_nodebodyid, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // xpos = xpos_in[worldid, bodyid]                                                        <L 333>
        var_16 = wp::address(var_xpos_in, var_0, var_14);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // if flex_centered[f]:                                                                   <L 335>
        var_19 = wp::address(var_flex_centered, var_3);
        var_20 = wp::load(var_19);
        if (var_20) {
            // flexnode_xpos_out[worldid, nodeid] = xpos                                          <L 336>
            wp::array_store(var_flexnode_xpos_out, var_0, var_1, var_17);
        }
        var_21 = wp::load(var_19);
        var_22 = wp::load(var_19);
        if (!var_22) {
            // local_pos = flex_node[nodeid]                                                      <L 338>
            var_23 = wp::address(var_flex_node, var_1);
            var_25 = wp::load(var_23);
            var_24 = wp::copy(var_25);
            // if local_pos[0] == 0.0 and local_pos[1] == 0.0 and local_pos[2] == 0.0:            <L 339>
            var_28 = wp::extract(var_24, var_27);
            var_30 = (var_28 == var_29);
            var_26 = var_30;
            if (var_26) {
                var_32 = wp::extract(var_24, var_31);
                var_34 = (var_32 == var_33);
                var_26 = var_26 && var_34;
            }
            if (var_26) {
                var_36 = wp::extract(var_24, var_35);
                var_38 = (var_36 == var_37);
                var_26 = var_26 && var_38;
            }
            if (var_26) {
                // flexnode_xpos_out[worldid, nodeid] = xpos                                      <L 340>
                wp::array_store(var_flexnode_xpos_out, var_0, var_1, var_17);
            }
            if (!var_26) {
                // xmat = xmat_in[worldid, bodyid]                                                <L 342>
                var_39 = wp::address(var_xmat_in, var_0, var_14);
                var_41 = wp::load(var_39);
                var_40 = wp::copy(var_41);
                // flexnode_xpos_out[worldid, nodeid] = xmat @ local_pos + xpos                   <L 343>
                var_42 = wp::mul(var_40, var_24);
                var_43 = wp::add(var_42, var_17);
                wp::array_store(var_flexnode_xpos_out, var_0, var_1, var_43);
            }
        }
        var_44 = wp::load(var_19);
    }
}



extern "C" __global__ void _site_local_to_global_084f92a4_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_site_bodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_pos,
    wp::array_t<wp::quat_t<wp::float32>> var_site_quat,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_xquat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_site_xpos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_site_xmat_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::vec_t<3, wp::float32>* var_5;
        wp::vec_t<3, wp::float32> var_6;
        wp::vec_t<3, wp::float32> var_7;
        wp::quat_t<wp::float32>* var_8;
        wp::quat_t<wp::float32> var_9;
        wp::quat_t<wp::float32> var_10;
        wp::shape_t* var_11;
        const wp::int32 var_12 = 0;
        wp::int32 var_13;
        wp::shape_t var_14;
        wp::int32 var_15;
        wp::vec_t<3, wp::float32>* var_16;
        wp::vec_t<3, wp::float32> var_17;
        wp::vec_t<3, wp::float32> var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::shape_t* var_20;
        const wp::int32 var_21 = 0;
        wp::int32 var_22;
        wp::shape_t var_23;
        wp::int32 var_24;
        wp::quat_t<wp::float32>* var_25;
        wp::quat_t<wp::float32> var_26;
        wp::quat_t<wp::float32> var_27;
        wp::mat_t<3, 3, wp::float32> var_28;
        //---------
        // forward
        // def _site_local_to_global(                                                             <L 210>
        // worldid, siteid = wp.tid()                                                             <L 222>
        builtin_tid2d(var_0, var_1);
        // bodyid = site_bodyid[siteid]                                                           <L 223>
        var_2 = wp::address(var_site_bodyid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // xpos = xpos_in[worldid, bodyid]                                                        <L 224>
        var_5 = wp::address(var_xpos_in, var_0, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // xquat = xquat_in[worldid, bodyid]                                                      <L 225>
        var_8 = wp::address(var_xquat_in, var_0, var_3);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // site_xpos_out[worldid, siteid] = xpos + math.rot_vec_quat(site_pos[worldid % site_pos.shape[0], siteid], xquat)       <L 226>
        var_11 = &(var_site_pos.shape);
        var_14 = wp::load(var_11);
        var_13 = wp::extract(var_14, var_12);
        var_15 = wp::mod(var_0, var_13);
        var_16 = wp::address(var_site_pos, var_15, var_1);
        var_18 = wp::load(var_16);
        var_17 = rot_vec_quat_0(var_18, var_9);
        var_19 = wp::add(var_6, var_17);
        wp::array_store(var_site_xpos_out, var_0, var_1, var_19);
        // site_xmat_out[worldid, siteid] = math.quat_to_mat(math.mul_quat(xquat, site_quat[worldid % site_quat.shape[0], siteid]))       <L 227>
        var_20 = &(var_site_quat.shape);
        var_23 = wp::load(var_20);
        var_22 = wp::extract(var_23, var_21);
        var_24 = wp::mod(var_0, var_22);
        var_25 = wp::address(var_site_quat, var_24, var_1);
        var_27 = wp::load(var_25);
        var_26 = mul_quat_0(var_9, var_27);
        var_28 = quat_to_mat_0(var_26);
        wp::array_store(var_site_xmat_out, var_0, var_1, var_28);
    }
}



extern "C" __global__ void _subtree_com_init_002b02fd_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_out)
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
        wp::int32 var_1;
        wp::vec_t<3, wp::float32>* var_2;
        wp::shape_t* var_3;
        const wp::int32 var_4 = 0;
        wp::int32 var_5;
        wp::shape_t var_6;
        wp::int32 var_7;
        wp::float32* var_8;
        wp::vec_t<3, wp::float32> var_9;
        wp::vec_t<3, wp::float32> var_10;
        wp::float32 var_11;
        //---------
        // forward
        // def _subtree_com_init(                                                                 <L 657>
        // worldid, bodyid = wp.tid()                                                             <L 665>
        builtin_tid2d(var_0, var_1);
        // subtree_com_out[worldid, bodyid] = xipos_in[worldid, bodyid] * body_mass[worldid % body_mass.shape[0], bodyid]       <L 666>
        var_2 = wp::address(var_xipos_in, var_0, var_1);
        var_3 = &(var_body_mass.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_body_mass, var_7, var_1);
        var_10 = wp::load(var_2);
        var_11 = wp::load(var_8);
        var_9 = wp::mul(var_10, var_11);
        wp::array_store(var_subtree_com_out, var_0, var_1, var_9);
    }
}



extern "C" __global__ void _subtree_com_acc_350f0560_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::int32> var_body_tree_,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_out)
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
        wp::int32 var_1;
        wp::int32* var_2;
        wp::int32 var_3;
        wp::int32 var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::int32 var_8 = 0;
        bool var_9;
        wp::vec_t<3, wp::float32>* var_10;
        wp::vec_t<3, wp::float32> var_11;
        wp::vec_t<3, wp::float32> var_12;
        //---------
        // forward
        // def _subtree_com_acc(                                                                  <L 670>
        // worldid, nodeid = wp.tid()                                                             <L 680>
        builtin_tid2d(var_0, var_1);
        // bodyid = body_tree_[nodeid]                                                            <L 681>
        var_2 = wp::address(var_body_tree_, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // pid = body_parentid[bodyid]                                                            <L 682>
        var_5 = wp::address(var_body_parentid, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if bodyid != 0:                                                                        <L 683>
        var_9 = (var_3 != var_8);
        if (var_9) {
            // wp.atomic_add(subtree_com_out, worldid, pid, subtree_com_in[worldid, bodyid])       <L 684>
            var_10 = wp::address(var_subtree_com_in, var_0, var_3);
            var_12 = wp::load(var_10);
            var_11 = wp::atomic_add(var_subtree_com_out, var_0, var_6, var_12);
        }
    }
}

