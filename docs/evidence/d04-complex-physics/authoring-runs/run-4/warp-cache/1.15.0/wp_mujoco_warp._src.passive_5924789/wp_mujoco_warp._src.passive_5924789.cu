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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1622
static CUDA_CALLABLE bool _flex_hessian_active_0(
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<bool> var_flex_rigid,
    wp::int32 var_f)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    const bool var_2 = false;
    bool var_3;
    wp::int32* var_4;
    const wp::int32 var_5 = 0;
    bool var_6;
    wp::int32 var_7;
    bool* var_8;
    bool var_9;
    wp::int32* var_10;
    const wp::int32 var_11 = 2;
    bool var_12;
    wp::int32 var_13;
    const bool var_14 = false;
    wp::int32* var_15;
    wp::int32 var_16;
    wp::int32 var_17;
    const wp::int32 var_18 = 0;
    bool var_19;
    const bool var_20 = false;
    wp::float32* var_21;
    const wp::float32 var_22 = 0.0;
    bool var_23;
    wp::float32 var_24;
    //---------
    // forward
    // def _flex_hessian_active(                                                              <L 1623>
    // if f < 0:                                                                              <L 1633>
    var_1 = (var_f < var_0);
    if (var_1) {
        // return False                                                                       <L 1634>
        return var_2;
    }
    // if flex_interp[f] != 0 or flex_rigid[f] or flex_dim[f] < 2:                            <L 1635>
    var_4 = wp::address(var_flex_interp, var_f);
    var_7 = wp::load(var_4);
    var_6 = (var_7 != var_5);
    var_3 = var_6;
    if (!var_3) {
        var_8 = wp::address(var_flex_rigid, var_f);
        var_9 = wp::load(var_8);
        var_3 = var_3 || var_9;
    }
    if (!var_3) {
        var_10 = wp::address(var_flex_dim, var_f);
        var_13 = wp::load(var_10);
        var_12 = (var_13 < var_11);
        var_3 = var_3 || var_12;
    }
    if (var_3) {
        // return False                                                                       <L 1636>
        return var_14;
    }
    // stiffness_adr = flex_stiffnessadr[f]                                                   <L 1637>
    var_15 = wp::address(var_flex_stiffnessadr, var_f);
    var_17 = wp::load(var_15);
    var_16 = wp::copy(var_17);
    // if stiffness_adr < 0:                                                                  <L 1638>
    var_19 = (var_16 < var_18);
    if (var_19) {
        // return False                                                                       <L 1639>
        return var_20;
    }
    // return flex_stiffness[stiffness_adr] != 0.0                                            <L 1640>
    var_21 = wp::address(var_flex_stiffness, var_16);
    var_24 = wp::load(var_21);
    var_23 = (var_24 != var_22);
    return var_23;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:53
static CUDA_CALLABLE wp::vec_t<3, wp::float32> geom_semiaxes_0(
    wp::vec_t<3, wp::float32> var_size,
    wp::int32 var_geom_type)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 2;
    bool var_1;
    const wp::int32 var_2 = 0;
    wp::float32 var_3;
    wp::vec_t<3, wp::float32> var_4;
    const wp::int32 var_5 = 3;
    bool var_6;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::vec_t<3, wp::float32> var_12;
    const wp::int32 var_13 = 5;
    bool var_14;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    const wp::int32 var_17 = 1;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    //---------
    // forward
    // def geom_semiaxes(size: wp.vec3, geom_type: int) -> wp.vec3:  # kernel_analyzer: ignore       <L 54>
    // if geom_type == GeomType.SPHERE:                                                       <L 55>
    var_1 = (var_geom_type == var_0);
    if (var_1) {
        // r = size[0]                                                                        <L 56>
        var_3 = wp::extract(var_size, var_2);
        // return wp.vec3(r, r, r)                                                            <L 57>
        var_4 = wp::vec_t<3, wp::float32>(var_3, var_3, var_3);
        return var_4;
    }
    // if geom_type == GeomType.CAPSULE:                                                      <L 59>
    var_6 = (var_geom_type == var_5);
    if (var_6) {
        // radius = size[0]                                                                   <L 60>
        var_8 = wp::extract(var_size, var_7);
        // half_length = size[1]                                                              <L 61>
        var_10 = wp::extract(var_size, var_9);
        // return wp.vec3(radius, radius, half_length + radius)                               <L 62>
        var_11 = wp::add(var_10, var_8);
        var_12 = wp::vec_t<3, wp::float32>(var_8, var_8, var_11);
        return var_12;
    }
    // if geom_type == GeomType.CYLINDER:                                                     <L 64>
    var_14 = (var_geom_type == var_13);
    if (var_14) {
        // radius = size[0]                                                                   <L 65>
        var_16 = wp::extract(var_size, var_15);
        // half_length = size[1]                                                              <L 66>
        var_18 = wp::extract(var_size, var_17);
        // return wp.vec3(radius, radius, half_length)                                        <L 67>
        var_19 = wp::vec_t<3, wp::float32>(var_16, var_16, var_18);
        return var_19;
    }
    var_20 = wp::where(var_14, var_16, var_8);
    var_21 = wp::where(var_14, var_18, var_10);
    // return size                                                                            <L 70>
    return var_size;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:42
static CUDA_CALLABLE wp::float32 _pow2_0(
    wp::float32 var_val)
{
    //---------
    // primal vars
    wp::float32 var_0;
    //---------
    // forward
    // def _pow2(val: float) -> float:                                                        <L 43>
    // return val * val                                                                       <L 44>
    var_0 = wp::mul(var_val, var_val);
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:47
static CUDA_CALLABLE wp::float32 _pow4_0(
    wp::float32 var_val)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    //---------
    // forward
    // def _pow4(val: float) -> float:                                                        <L 48>
    // sq = val * val                                                                         <L 49>
    var_0 = wp::mul(var_val, var_val);
    // return sq * sq                                                                         <L 50>
    var_1 = wp::mul(var_0, var_0);
    return var_1;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:73
static CUDA_CALLABLE wp::float32 ellipsoid_max_moment_0(
    wp::vec_t<3, wp::float32> var_size,
    wp::int32 var_dir)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::int32 var_1 = 1;
    wp::int32 var_2;
    const wp::int32 var_3 = 3;
    wp::int32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 2;
    wp::int32 var_7;
    const wp::int32 var_8 = 3;
    wp::int32 var_9;
    wp::float32 var_10;
    const wp::float32 var_11 = 1.6755160819145563;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    //---------
    // forward
    // def ellipsoid_max_moment(size: wp.vec3, dir: int) -> float:                            <L 74>
    // d0 = size[dir]                                                                         <L 75>
    var_0 = wp::extract(var_size, var_dir);
    // d1 = size[(dir + 1) % 3]                                                               <L 76>
    var_2 = wp::add(var_dir, var_1);
    var_4 = wp::mod(var_2, var_3);
    var_5 = wp::extract(var_size, var_4);
    // d2 = size[(dir + 2) % 3]                                                               <L 77>
    var_7 = wp::add(var_dir, var_6);
    var_9 = wp::mod(var_7, var_8);
    var_10 = wp::extract(var_size, var_9);
    // return wp.static(8.0 / 15.0 * wp.pi) * d0 * _pow4(wp.max(d1, d2))                      <L 78>
    var_12 = wp::mul(var_11, var_0);
    var_13 = wp::max(var_5, var_10);
    var_14 = _pow4_0(var_13);
    var_15 = wp::mul(var_12, var_14);
    return var_15;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:101
static CUDA_CALLABLE wp::quat_t<wp::float32> compute_interp_cell_quat_0(
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::int32 var_order,
    wp::int32 var_ci,
    wp::int32 var_cj,
    wp::int32 var_ck,
    wp::int32 var_cy,
    wp::int32 var_cz,
    wp::int32 var_ny_g,
    wp::int32 var_nz_g,
    wp::int32 var_nstart,
    wp::int32 var_worldid)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::int32 var_1;
    const wp::int32 var_2 = 1;
    wp::int32 var_3;
    wp::int32 var_4;
    const wp::int32 var_5 = 1;
    wp::int32 var_6;
    wp::int32 var_7;
    const wp::float32 var_8 = 0.0;
    wp::mat_t<3, 3, wp::float32> var_9;
    const wp::int32 var_10 = 0;
    wp::int32 var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    wp::range_t var_14;
    wp::int32 var_15;
    const wp::int32 var_16 = 1;
    wp::int32 var_17;
    wp::range_t var_18;
    wp::int32 var_19;
    const wp::int32 var_20 = 1;
    wp::int32 var_21;
    wp::range_t var_22;
    wp::int32 var_23;
    bool var_24;
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
    wp::vec_t<3, wp::float32>* var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::vec_t<3, wp::float32> var_39;
    const wp::int32 var_40 = 0;
    bool var_41;
    const wp::int32 var_42 = -1;
    wp::float32 var_43;
    const wp::int32 var_44 = 1;
    wp::float32 var_45;
    wp::float32 var_46;
    const wp::int32 var_47 = 0;
    bool var_48;
    const wp::int32 var_49 = -1;
    wp::float32 var_50;
    const wp::int32 var_51 = 1;
    wp::float32 var_52;
    wp::float32 var_53;
    const wp::int32 var_54 = 0;
    bool var_55;
    const wp::int32 var_56 = -1;
    wp::float32 var_57;
    const wp::int32 var_58 = 1;
    wp::float32 var_59;
    wp::float32 var_60;
    const wp::float32 var_61 = 0.5;
    wp::float32 var_62;
    const wp::float32 var_63 = 0.5;
    wp::float32 var_64;
    const wp::float32 var_65 = 0.5;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::int32 var_73 = 0;
    wp::float32 var_74;
    wp::float32 var_75;
    const wp::int32 var_76 = 0;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::int32 var_79 = 1;
    wp::float32 var_80;
    wp::float32 var_81;
    const wp::int32 var_82 = 2;
    const wp::int32 var_83 = 1;
    wp::float32 var_84;
    wp::float32 var_85;
    const wp::int32 var_86 = 0;
    wp::float32 var_87;
    wp::float32 var_88;
    const wp::int32 var_89 = 1;
    wp::float32 var_90;
    wp::float32 var_91;
    const wp::int32 var_92 = 2;
    const wp::int32 var_93 = 2;
    wp::float32 var_94;
    wp::float32 var_95;
    const wp::int32 var_96 = 0;
    wp::float32 var_97;
    wp::float32 var_98;
    const wp::int32 var_99 = 1;
    wp::float32 var_100;
    wp::float32 var_101;
    const wp::int32 var_102 = 2;
    const wp::int32 var_103 = 1;
    wp::int32 var_104;
    wp::int32 var_105;
    wp::quat_t<wp::float32> var_106;
    //---------
    // forward
    // def compute_interp_cell_quat(                                                          <L 102>
    // npc = (order + 1) * (order + 1) * (order + 1)                                          <L 118>
    var_1 = wp::add(var_order, var_0);
    var_3 = wp::add(var_order, var_2);
    var_4 = wp::mul(var_1, var_3);
    var_6 = wp::add(var_order, var_5);
    var_7 = wp::mul(var_4, var_6);
    // F = wp.mat33(0.0)                                                                      <L 119>
    var_9 = wp::mat_t<3, 3, wp::float32>(var_8);
    // idx = int(0)                                                                           <L 120>
    var_11 = wp::int(var_10);
    // for li in range(order + 1):                                                            <L 121>
    var_13 = wp::add(var_order, var_12);
    var_14 = wp::range(var_13);
    start_for_0:;
        if (iter_cmp(var_14) == 0) goto end_for_0;
        var_15 = wp::iter_next(var_14);
        // for lj in range(order + 1):                                                        <L 122>
        var_17 = wp::add(var_order, var_16);
        var_18 = wp::range(var_17);
        start_for_2:;
            if (iter_cmp(var_18) == 0) goto end_for_2;
            var_19 = wp::iter_next(var_18);
            // for lk in range(order + 1):                                                    <L 123>
            var_21 = wp::add(var_order, var_20);
            var_22 = wp::range(var_21);
            start_for_4:;
                if (iter_cmp(var_22) == 0) goto end_for_4;
                var_23 = wp::iter_next(var_22);
                // if idx < npc:                                                              <L 124>
                var_24 = (var_11 < var_7);
                if (var_24) {
                    // gi = ci * order + li                                                   <L 125>
                    var_25 = wp::mul(var_ci, var_order);
                    var_26 = wp::add(var_25, var_15);
                    // gj = cj * order + lj                                                   <L 126>
                    var_27 = wp::mul(var_cj, var_order);
                    var_28 = wp::add(var_27, var_19);
                    // gk = ck * order + lk                                                   <L 127>
                    var_29 = wp::mul(var_ck, var_order);
                    var_30 = wp::add(var_29, var_23);
                    // gidx = gi * ny_g * nz_g + gj * nz_g + gk                               <L 128>
                    var_31 = wp::mul(var_26, var_ny_g);
                    var_32 = wp::mul(var_31, var_nz_g);
                    var_33 = wp::mul(var_28, var_nz_g);
                    var_34 = wp::add(var_32, var_33);
                    var_35 = wp::add(var_34, var_30);
                    // node_pos = flexnode_xpos_in[worldid, nstart + gidx]                    <L 130>
                    var_36 = wp::add(var_nstart, var_35);
                    var_37 = wp::address(var_flexnode_xpos_in, var_worldid, var_36);
                    var_39 = wp::load(var_37);
                    var_38 = wp::copy(var_39);
                    // dphi_x = float(-1) if li == 0 else float(1)                            <L 132>
                    var_41 = (var_15 == var_40);
                    if (var_41) {
                        var_43 = wp::float(var_42);
                    }
                    if (!var_41) {
                        var_45 = wp::float(var_44);
                    }
                    var_46 = wp::where(var_41, var_43, var_45);
                    // dphi_y = float(-1) if lj == 0 else float(1)                            <L 133>
                    var_48 = (var_19 == var_47);
                    if (var_48) {
                        var_50 = wp::float(var_49);
                    }
                    if (!var_48) {
                        var_52 = wp::float(var_51);
                    }
                    var_53 = wp::where(var_48, var_50, var_52);
                    // dphi_z = float(-1) if lk == 0 else float(1)                            <L 134>
                    var_55 = (var_23 == var_54);
                    if (var_55) {
                        var_57 = wp::float(var_56);
                    }
                    if (!var_55) {
                        var_59 = wp::float(var_58);
                    }
                    var_60 = wp::where(var_55, var_57, var_59);
                    // phi_x = float(0.5)                                                     <L 135>
                    var_62 = wp::float(var_61);
                    // phi_y = float(0.5)                                                     <L 136>
                    var_64 = wp::float(var_63);
                    // phi_z = float(0.5)                                                     <L 137>
                    var_66 = wp::float(var_65);
                    // grad_x = dphi_x * phi_y * phi_z                                        <L 139>
                    var_67 = wp::mul(var_46, var_64);
                    var_68 = wp::mul(var_67, var_66);
                    // grad_y = phi_x * dphi_y * phi_z                                        <L 140>
                    var_69 = wp::mul(var_62, var_53);
                    var_70 = wp::mul(var_69, var_66);
                    // grad_z = phi_x * phi_y * dphi_z                                        <L 141>
                    var_71 = wp::mul(var_62, var_64);
                    var_72 = wp::mul(var_71, var_60);
                    // for r in range(3):                                                     <L 143>
                    // F[r, 0] += node_pos[r] * grad_x                                        <L 144>
                    var_74 = wp::extract(var_38, var_73);
                    var_75 = wp::mul(var_74, var_68);
                    wp::add_inplace(var_9, var_73, var_76, var_75);
                    // F[r, 1] += node_pos[r] * grad_y                                        <L 145>
                    var_77 = wp::extract(var_38, var_73);
                    var_78 = wp::mul(var_77, var_70);
                    wp::add_inplace(var_9, var_73, var_79, var_78);
                    // F[r, 2] += node_pos[r] * grad_z                                        <L 146>
                    var_80 = wp::extract(var_38, var_73);
                    var_81 = wp::mul(var_80, var_72);
                    wp::add_inplace(var_9, var_73, var_82, var_81);
                    // F[r, 0] += node_pos[r] * grad_x                                        <L 144>
                    var_84 = wp::extract(var_38, var_83);
                    var_85 = wp::mul(var_84, var_68);
                    wp::add_inplace(var_9, var_83, var_86, var_85);
                    // F[r, 1] += node_pos[r] * grad_y                                        <L 145>
                    var_87 = wp::extract(var_38, var_83);
                    var_88 = wp::mul(var_87, var_70);
                    wp::add_inplace(var_9, var_83, var_89, var_88);
                    // F[r, 2] += node_pos[r] * grad_z                                        <L 146>
                    var_90 = wp::extract(var_38, var_83);
                    var_91 = wp::mul(var_90, var_72);
                    wp::add_inplace(var_9, var_83, var_92, var_91);
                    // F[r, 0] += node_pos[r] * grad_x                                        <L 144>
                    var_94 = wp::extract(var_38, var_93);
                    var_95 = wp::mul(var_94, var_68);
                    wp::add_inplace(var_9, var_93, var_96, var_95);
                    // F[r, 1] += node_pos[r] * grad_y                                        <L 145>
                    var_97 = wp::extract(var_38, var_93);
                    var_98 = wp::mul(var_97, var_70);
                    wp::add_inplace(var_9, var_93, var_99, var_98);
                    // F[r, 2] += node_pos[r] * grad_z                                        <L 146>
                    var_100 = wp::extract(var_38, var_93);
                    var_101 = wp::mul(var_100, var_72);
                    wp::add_inplace(var_9, var_93, var_102, var_101);
                    // idx += 1                                                               <L 148>
                    var_104 = wp::add(var_11, var_103);
                }
                var_105 = wp::where(var_24, var_104, var_11);
                wp::assign(var_11, var_105);
                goto start_for_4;
            end_for_4:;
            goto start_for_2;
        end_for_2:;
        goto start_for_0;
    end_for_0:;
    // return mat33_to_quat_polar(F)                                                          <L 150>
    var_106 = mat33_to_quat_polar_0(var_9);
    return var_106;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1056
static CUDA_CALLABLE wp::mat_t<6, 2, wp::int32> stretch_edge_endpoints_0(
    wp::int32 var_dim)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 3;
    bool var_1;
    const wp::int32 var_2 = 0;
    const wp::int32 var_3 = 1;
    const wp::int32 var_4 = 1;
    const wp::int32 var_5 = 2;
    const wp::int32 var_6 = 2;
    const wp::int32 var_7 = 0;
    const wp::int32 var_8 = 2;
    const wp::int32 var_9 = 3;
    const wp::int32 var_10 = 0;
    const wp::int32 var_11 = 3;
    const wp::int32 var_12 = 1;
    const wp::int32 var_13 = 3;
    const wp::int32 var_14 = 6;
    const wp::int32 var_15 = 2;
    wp::tuple_t<wp::int32, wp::int32> var_16;
    wp::mat_t<6, 2, wp::int32> var_17;
    const wp::int32 var_18 = 1;
    const wp::int32 var_19 = 2;
    const wp::int32 var_20 = 2;
    const wp::int32 var_21 = 0;
    const wp::int32 var_22 = 0;
    const wp::int32 var_23 = 1;
    const wp::int32 var_24 = 0;
    const wp::int32 var_25 = 0;
    const wp::int32 var_26 = 0;
    const wp::int32 var_27 = 0;
    const wp::int32 var_28 = 0;
    const wp::int32 var_29 = 0;
    const wp::int32 var_30 = 6;
    const wp::int32 var_31 = 2;
    wp::tuple_t<wp::int32, wp::int32> var_32;
    wp::mat_t<6, 2, wp::int32> var_33;
    wp::mat_t<6, 2, wp::int32> var_34;
    //---------
    // forward
    // def stretch_edge_endpoints(dim: int):                                                  <L 1057>
    // return wp.where(                                                                       <L 1058>
    // dim == 3,                                                                              <L 1059>
    var_1 = (var_dim == var_0);
    // wp.matrix(0, 1, 1, 2, 2, 0, 2, 3, 0, 3, 1, 3, shape=(6, 2), dtype=int),                <L 1060>
    var_16 = wp::tuple(var_14, var_15);
    var_17 = wp::mat_t<6, 2, wp::int32>({var_2, var_3, var_4, var_5, var_6, var_7, var_8, var_9, var_10, var_11, var_12, var_13});
    // wp.matrix(1, 2, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, shape=(6, 2), dtype=int),                <L 1061>
    var_32 = wp::tuple(var_30, var_31);
    var_33 = wp::mat_t<6, 2, wp::int32>({var_18, var_19, var_20, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_29});
    var_34 = wp::where(var_1, var_17, var_33);
    return var_34;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1065
static CUDA_CALLABLE wp::mat_t<6, 3, wp::float32> stretch_edge_vectors_0(
    wp::mat_t<4, 3, wp::float32> var_vert_xpos,
    wp::int32 var_dim)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 2;
    bool var_1;
    const wp::int32 var_2 = 3;
    const wp::int32 var_3 = 6;
    wp::int32 var_4;
    wp::mat_t<6, 2, wp::int32> var_5;
    wp::mat_t<6, 3, wp::float32> var_6;
    wp::range_t var_7;
    wp::int32 var_8;
    const wp::int32 var_9 = 0;
    wp::int32 var_10;
    wp::vec_t<3, wp::float32> var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<3, wp::float32> var_15;
    //---------
    // forward
    // def stretch_edge_vectors(vert_xpos: mat43, dim: int) -> mat63:                         <L 1066>
    // nedge = 3 if dim == 2 else 6                                                           <L 1067>
    var_1 = (var_dim == var_0);
    if (var_1) {
    }
    if (!var_1) {
    }
    var_4 = wp::where(var_1, var_2, var_3);
    // edges = stretch_edge_endpoints(dim)                                                    <L 1068>
    var_5 = stretch_edge_endpoints_0(var_dim);
    // edgevec = mat63()                                                                      <L 1069>
    var_6 = wp::mat_t<6, 3, wp::float32>();
    // for e in range(nedge):                                                                 <L 1070>
    var_7 = wp::range(var_4);
    start_for_0:;
        if (iter_cmp(var_7) == 0) goto end_for_0;
        var_8 = wp::iter_next(var_7);
        // edgevec[e] = vert_xpos[edges[e, 0]] - vert_xpos[edges[e, 1]]                       <L 1071>
        var_10 = wp::extract(var_5, var_8, var_9);
        var_11 = wp::extract(var_vert_xpos, var_10);
        var_13 = wp::extract(var_5, var_8, var_12);
        var_14 = wp::extract(var_vert_xpos, var_13);
        var_15 = wp::sub(var_11, var_14);
        wp::assign_inplace(var_6, var_8, var_15);
        goto start_for_0;
    end_for_0:;
    // return edgevec                                                                         <L 1072>
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1075
static CUDA_CALLABLE wp::vec_t<6, wp::float32> stretch_elongation_0(
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::int32 var_worldid,
    wp::int32 var_elemedge_adr,
    wp::int32 var_edge_adr,
    wp::int32 var_nedge)
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
    wp::range_t var_7;
    wp::int32 var_8;
    wp::int32 var_9;
    wp::int32* var_10;
    wp::int32 var_11;
    wp::int32 var_12;
    wp::float32* var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32* var_16;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    //---------
    // forward
    // def stretch_elongation(                                                                <L 1076>
    // elongation = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                        <L 1088>
    var_6 = wp::vec_t<6, wp::float32>({var_0, var_1, var_2, var_3, var_4, var_5});
    // for e in range(nedge):                                                                 <L 1089>
    var_7 = wp::range(var_nedge);
    start_for_0:;
        if (iter_cmp(var_7) == 0) goto end_for_0;
        var_8 = wp::iter_next(var_7);
        // e_idx = edge_adr + flex_elemedge[elemedge_adr + e]                                 <L 1090>
        var_9 = wp::add(var_elemedge_adr, var_8);
        var_10 = wp::address(var_flex_elemedge, var_9);
        var_12 = wp::load(var_10);
        var_11 = wp::add(var_edge_adr, var_12);
        // deformed = flexedge_length_in[worldid, e_idx]                                      <L 1091>
        var_13 = wp::address(var_flexedge_length_in, var_worldid, var_11);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // reference = flexedge_length0[e_idx]                                                <L 1092>
        var_16 = wp::address(var_flexedge_length0, var_11);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // elongation[e] = deformed * deformed - reference * reference                        <L 1093>
        var_19 = wp::mul(var_14, var_14);
        var_20 = wp::mul(var_17, var_17);
        var_21 = wp::sub(var_19, var_20);
        wp::assign_inplace(var_6, var_8, var_21);
        goto start_for_0;
    end_for_0:;
    // return elongation                                                                      <L 1094>
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:955
static CUDA_CALLABLE void _snh_jacobi_rot_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::mat_t<3, 3, wp::float32> var_V,
    wp::int32 var_p,
    wp::int32 var_q,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    bool & ret_2)
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
    const wp::int32 var_17 = 0;
    wp::float32 var_18;
    const wp::int32 var_19 = 0;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::int32 var_22 = 1;
    wp::float32 var_23;
    const wp::int32 var_24 = 1;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    wp::float32 var_29;
    const wp::int32 var_30 = 2;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::int32 var_34 = 0;
    wp::float32 var_35;
    const wp::int32 var_36 = 0;
    wp::float32 var_37;
    wp::float32 var_38;
    const wp::int32 var_39 = 1;
    wp::float32 var_40;
    const wp::int32 var_41 = 1;
    wp::float32 var_42;
    wp::float32 var_43;
    wp::float32 var_44;
    const wp::int32 var_45 = 2;
    wp::float32 var_46;
    const wp::int32 var_47 = 2;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::float32 var_52 = 2e-06;
    wp::float32 var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    bool var_57;
    const bool var_58 = true;
    const wp::float32 var_59 = 0.5;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::float32 var_62 = 0.0;
    bool var_63;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::float32 var_66;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::float32 var_73 = 1.0;
    const wp::float32 var_74 = 1.0;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    const wp::int32 var_80 = 0;
    wp::float32 var_81;
    wp::float32 var_82;
    wp::float32 var_83;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::float32 var_86;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    wp::float32 var_95;
    wp::float32 var_96;
    const wp::int32 var_97 = 1;
    wp::float32 var_98;
    wp::float32 var_99;
    wp::float32 var_100;
    wp::float32 var_101;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    wp::float32 var_105;
    wp::float32 var_106;
    wp::float32 var_107;
    wp::float32 var_108;
    wp::float32 var_109;
    wp::float32 var_110;
    wp::float32 var_111;
    wp::float32 var_112;
    wp::float32 var_113;
    const wp::int32 var_114 = 2;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    wp::float32 var_120;
    wp::float32 var_121;
    wp::float32 var_122;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    wp::float32 var_126;
    wp::float32 var_127;
    wp::float32 var_128;
    wp::float32 var_129;
    wp::float32 var_130;
    const bool var_131 = false;
    //---------
    // forward
    // def _snh_jacobi_rot(A: wp.mat33, V: wp.mat33, p: int, q: int) -> tuple[wp.mat33, wp.mat33, bool]:       <L 956>
    // pp = A[0, p] * A[0, p] + A[1, p] * A[1, p] + A[2, p] * A[2, p]                         <L 957>
    var_1 = wp::extract(var_A, var_0, var_p);
    var_3 = wp::extract(var_A, var_2, var_p);
    var_4 = wp::mul(var_1, var_3);
    var_6 = wp::extract(var_A, var_5, var_p);
    var_8 = wp::extract(var_A, var_7, var_p);
    var_9 = wp::mul(var_6, var_8);
    var_10 = wp::add(var_4, var_9);
    var_12 = wp::extract(var_A, var_11, var_p);
    var_14 = wp::extract(var_A, var_13, var_p);
    var_15 = wp::mul(var_12, var_14);
    var_16 = wp::add(var_10, var_15);
    // qq = A[0, q] * A[0, q] + A[1, q] * A[1, q] + A[2, q] * A[2, q]                         <L 958>
    var_18 = wp::extract(var_A, var_17, var_q);
    var_20 = wp::extract(var_A, var_19, var_q);
    var_21 = wp::mul(var_18, var_20);
    var_23 = wp::extract(var_A, var_22, var_q);
    var_25 = wp::extract(var_A, var_24, var_q);
    var_26 = wp::mul(var_23, var_25);
    var_27 = wp::add(var_21, var_26);
    var_29 = wp::extract(var_A, var_28, var_q);
    var_31 = wp::extract(var_A, var_30, var_q);
    var_32 = wp::mul(var_29, var_31);
    var_33 = wp::add(var_27, var_32);
    // pq = A[0, p] * A[0, q] + A[1, p] * A[1, q] + A[2, p] * A[2, q]                         <L 959>
    var_35 = wp::extract(var_A, var_34, var_p);
    var_37 = wp::extract(var_A, var_36, var_q);
    var_38 = wp::mul(var_35, var_37);
    var_40 = wp::extract(var_A, var_39, var_p);
    var_42 = wp::extract(var_A, var_41, var_q);
    var_43 = wp::mul(var_40, var_42);
    var_44 = wp::add(var_38, var_43);
    var_46 = wp::extract(var_A, var_45, var_p);
    var_48 = wp::extract(var_A, var_47, var_q);
    var_49 = wp::mul(var_46, var_48);
    var_50 = wp::add(var_44, var_49);
    // if wp.abs(pq) <= 2.0e-6 * wp.sqrt(pp) * wp.sqrt(qq):                                   <L 960>
    var_51 = wp::abs(var_50);
    var_53 = wp::sqrt(var_16);
    var_54 = wp::mul(var_52, var_53);
    var_55 = wp::sqrt(var_33);
    var_56 = wp::mul(var_54, var_55);
    var_57 = (var_51 <= var_56);
    if (var_57) {
        // return A, V, True                                                                  <L 961>
        ret_0 = var_A;
        ret_1 = var_V;
        ret_2 = var_58;
        return;
    }
    // delta = 0.5 * (qq - pp)                                                                <L 962>
    var_60 = wp::sub(var_33, var_16);
    var_61 = wp::mul(var_59, var_60);
    // t = (pq if delta >= 0.0 else -pq) / (wp.abs(delta) + wp.sqrt(delta * delta + pq * pq))       <L 963>
    var_63 = (var_61 >= var_62);
    if (var_63) {
    }
    if (!var_63) {
        var_64 = wp::neg(var_50);
    }
    var_65 = wp::where(var_63, var_50, var_64);
    var_66 = wp::abs(var_61);
    var_67 = wp::mul(var_61, var_61);
    var_68 = wp::mul(var_50, var_50);
    var_69 = wp::add(var_67, var_68);
    var_70 = wp::sqrt(var_69);
    var_71 = wp::add(var_66, var_70);
    var_72 = wp::div(var_65, var_71);
    // c = 1.0 / wp.sqrt(1.0 + t * t)                                                         <L 964>
    var_75 = wp::mul(var_72, var_72);
    var_76 = wp::add(var_74, var_75);
    var_77 = wp::sqrt(var_76);
    var_78 = wp::div(var_73, var_77);
    // s = t * c                                                                              <L 965>
    var_79 = wp::mul(var_72, var_78);
    // for row in range(3):                                                                   <L 966>
    // arp = A[row, p]                                                                        <L 967>
    var_81 = wp::extract(var_A, var_80, var_p);
    // arq = A[row, q]                                                                        <L 968>
    var_82 = wp::extract(var_A, var_80, var_q);
    // A[row, p] = c * arp - s * arq                                                          <L 969>
    var_83 = wp::mul(var_78, var_81);
    var_84 = wp::mul(var_79, var_82);
    var_85 = wp::sub(var_83, var_84);
    wp::assign_inplace(var_A, var_80, var_p, var_85);
    // A[row, q] = s * arp + c * arq                                                          <L 970>
    var_86 = wp::mul(var_79, var_81);
    var_87 = wp::mul(var_78, var_82);
    var_88 = wp::add(var_86, var_87);
    wp::assign_inplace(var_A, var_80, var_q, var_88);
    // vrp = V[row, p]                                                                        <L 971>
    var_89 = wp::extract(var_V, var_80, var_p);
    // vrq = V[row, q]                                                                        <L 972>
    var_90 = wp::extract(var_V, var_80, var_q);
    // V[row, p] = c * vrp - s * vrq                                                          <L 973>
    var_91 = wp::mul(var_78, var_89);
    var_92 = wp::mul(var_79, var_90);
    var_93 = wp::sub(var_91, var_92);
    wp::assign_inplace(var_V, var_80, var_p, var_93);
    // V[row, q] = s * vrp + c * vrq                                                          <L 974>
    var_94 = wp::mul(var_79, var_89);
    var_95 = wp::mul(var_78, var_90);
    var_96 = wp::add(var_94, var_95);
    wp::assign_inplace(var_V, var_80, var_q, var_96);
    // arp = A[row, p]                                                                        <L 967>
    var_98 = wp::extract(var_A, var_97, var_p);
    // arq = A[row, q]                                                                        <L 968>
    var_99 = wp::extract(var_A, var_97, var_q);
    // A[row, p] = c * arp - s * arq                                                          <L 969>
    var_100 = wp::mul(var_78, var_98);
    var_101 = wp::mul(var_79, var_99);
    var_102 = wp::sub(var_100, var_101);
    wp::assign_inplace(var_A, var_97, var_p, var_102);
    // A[row, q] = s * arp + c * arq                                                          <L 970>
    var_103 = wp::mul(var_79, var_98);
    var_104 = wp::mul(var_78, var_99);
    var_105 = wp::add(var_103, var_104);
    wp::assign_inplace(var_A, var_97, var_q, var_105);
    // vrp = V[row, p]                                                                        <L 971>
    var_106 = wp::extract(var_V, var_97, var_p);
    // vrq = V[row, q]                                                                        <L 972>
    var_107 = wp::extract(var_V, var_97, var_q);
    // V[row, p] = c * vrp - s * vrq                                                          <L 973>
    var_108 = wp::mul(var_78, var_106);
    var_109 = wp::mul(var_79, var_107);
    var_110 = wp::sub(var_108, var_109);
    wp::assign_inplace(var_V, var_97, var_p, var_110);
    // V[row, q] = s * vrp + c * vrq                                                          <L 974>
    var_111 = wp::mul(var_79, var_106);
    var_112 = wp::mul(var_78, var_107);
    var_113 = wp::add(var_111, var_112);
    wp::assign_inplace(var_V, var_97, var_q, var_113);
    // arp = A[row, p]                                                                        <L 967>
    var_115 = wp::extract(var_A, var_114, var_p);
    // arq = A[row, q]                                                                        <L 968>
    var_116 = wp::extract(var_A, var_114, var_q);
    // A[row, p] = c * arp - s * arq                                                          <L 969>
    var_117 = wp::mul(var_78, var_115);
    var_118 = wp::mul(var_79, var_116);
    var_119 = wp::sub(var_117, var_118);
    wp::assign_inplace(var_A, var_114, var_p, var_119);
    // A[row, q] = s * arp + c * arq                                                          <L 970>
    var_120 = wp::mul(var_79, var_115);
    var_121 = wp::mul(var_78, var_116);
    var_122 = wp::add(var_120, var_121);
    wp::assign_inplace(var_A, var_114, var_q, var_122);
    // vrp = V[row, p]                                                                        <L 971>
    var_123 = wp::extract(var_V, var_114, var_p);
    // vrq = V[row, q]                                                                        <L 972>
    var_124 = wp::extract(var_V, var_114, var_q);
    // V[row, p] = c * vrp - s * vrq                                                          <L 973>
    var_125 = wp::mul(var_78, var_123);
    var_126 = wp::mul(var_79, var_124);
    var_127 = wp::sub(var_125, var_126);
    wp::assign_inplace(var_V, var_114, var_p, var_127);
    // V[row, q] = s * vrp + c * vrq                                                          <L 974>
    var_128 = wp::mul(var_79, var_123);
    var_129 = wp::mul(var_78, var_124);
    var_130 = wp::add(var_128, var_129);
    wp::assign_inplace(var_V, var_114, var_q, var_130);
    // return A, V, False                                                                     <L 975>
    ret_0 = var_A;
    ret_1 = var_V;
    ret_2 = var_131;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:978
static CUDA_CALLABLE void _snh_swap_cols_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::mat_t<3, 3, wp::float32> var_V,
    wp::vec_t<3, wp::float32> var_s,
    wp::int32 var_p,
    wp::int32 var_q,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    bool var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::float32 var_9;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 2;
    wp::float32 var_16;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    //---------
    // forward
    // def _snh_swap_cols(A: wp.mat33, V: wp.mat33, s: wp.vec3, p: int, q: int) -> tuple[wp.mat33, wp.mat33, wp.vec3]:       <L 979>
    // if s[q] > s[p]:                                                                        <L 980>
    var_0 = wp::extract(var_s, var_q);
    var_1 = wp::extract(var_s, var_p);
    var_2 = (var_0 > var_1);
    if (var_2) {
        // sp = s[p]                                                                          <L 981>
        var_3 = wp::extract(var_s, var_p);
        // s[p] = s[q]                                                                        <L 982>
        var_4 = wp::extract(var_s, var_q);
        wp::assign_inplace(var_s, var_p, var_4);
        // s[q] = sp                                                                          <L 983>
        wp::assign_inplace(var_s, var_q, var_3);
        // for row in range(3):                                                               <L 984>
        // arp = A[row, p]                                                                    <L 985>
        var_6 = wp::extract(var_A, var_5, var_p);
        // A[row, p] = A[row, q]                                                              <L 986>
        var_7 = wp::extract(var_A, var_5, var_q);
        wp::assign_inplace(var_A, var_5, var_p, var_7);
        // A[row, q] = arp                                                                    <L 987>
        wp::assign_inplace(var_A, var_5, var_q, var_6);
        // vrp = V[row, p]                                                                    <L 988>
        var_8 = wp::extract(var_V, var_5, var_p);
        // V[row, p] = V[row, q]                                                              <L 989>
        var_9 = wp::extract(var_V, var_5, var_q);
        wp::assign_inplace(var_V, var_5, var_p, var_9);
        // V[row, q] = vrp                                                                    <L 990>
        wp::assign_inplace(var_V, var_5, var_q, var_8);
        // arp = A[row, p]                                                                    <L 985>
        var_11 = wp::extract(var_A, var_10, var_p);
        // A[row, p] = A[row, q]                                                              <L 986>
        var_12 = wp::extract(var_A, var_10, var_q);
        wp::assign_inplace(var_A, var_10, var_p, var_12);
        // A[row, q] = arp                                                                    <L 987>
        wp::assign_inplace(var_A, var_10, var_q, var_11);
        // vrp = V[row, p]                                                                    <L 988>
        var_13 = wp::extract(var_V, var_10, var_p);
        // V[row, p] = V[row, q]                                                              <L 989>
        var_14 = wp::extract(var_V, var_10, var_q);
        wp::assign_inplace(var_V, var_10, var_p, var_14);
        // V[row, q] = vrp                                                                    <L 990>
        wp::assign_inplace(var_V, var_10, var_q, var_13);
        // arp = A[row, p]                                                                    <L 985>
        var_16 = wp::extract(var_A, var_15, var_p);
        // A[row, p] = A[row, q]                                                              <L 986>
        var_17 = wp::extract(var_A, var_15, var_q);
        wp::assign_inplace(var_A, var_15, var_p, var_17);
        // A[row, q] = arp                                                                    <L 987>
        wp::assign_inplace(var_A, var_15, var_q, var_16);
        // vrp = V[row, p]                                                                    <L 988>
        var_18 = wp::extract(var_V, var_15, var_p);
        // V[row, p] = V[row, q]                                                              <L 989>
        var_19 = wp::extract(var_V, var_15, var_q);
        wp::assign_inplace(var_V, var_15, var_p, var_19);
        // V[row, q] = vrp                                                                    <L 990>
        wp::assign_inplace(var_V, var_15, var_q, var_18);
    }
    // return A, V, s                                                                         <L 991>
    ret_0 = var_A;
    ret_1 = var_V;
    ret_2 = var_s;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:994
static CUDA_CALLABLE void _snh_svd_0(
    wp::mat_t<3, 3, wp::float32> var_F,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::mat_t<3, 3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 1;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    const wp::int32 var_24 = 2;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    const wp::int32 var_29 = 0;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::int32 var_33 = 1;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    const wp::int32 var_37 = 2;
    wp::float32 var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    const wp::float32 var_41 = 0.0;
    bool var_42;
    const wp::float32 var_43 = 1.0;
    wp::float32 var_44;
    wp::mat_t<3, 3, wp::float32> var_45;
    const wp::float32 var_46 = 0.0;
    const wp::float32 var_47 = 0.0;
    const wp::float32 var_48 = 0.0;
    const wp::float32 var_49 = 0.0;
    const wp::float32 var_50 = 0.0;
    const wp::float32 var_51 = 0.0;
    const wp::float32 var_52 = 0.0;
    const wp::float32 var_53 = 0.0;
    const wp::float32 var_54 = 0.0;
    wp::mat_t<3, 3, wp::float32> var_55;
    wp::mat_t<3, 3, wp::float32> var_56;
    const wp::int32 var_57 = 3;
    wp::mat_t<3, 3, wp::float32> var_58;
    const wp::int32 var_59 = 24;
    wp::range_t var_60;
    wp::int32 var_61;
    const wp::int32 var_62 = 0;
    const wp::int32 var_63 = 1;
    wp::mat_t<3, 3, wp::float32> var_64;
    wp::mat_t<3, 3, wp::float32> var_65;
    bool var_66;
    const wp::int32 var_67 = 0;
    const wp::int32 var_68 = 2;
    wp::mat_t<3, 3, wp::float32> var_69;
    wp::mat_t<3, 3, wp::float32> var_70;
    bool var_71;
    const wp::int32 var_72 = 1;
    const wp::int32 var_73 = 2;
    wp::mat_t<3, 3, wp::float32> var_74;
    wp::mat_t<3, 3, wp::float32> var_75;
    bool var_76;
    bool var_77;
    wp::mat_t<3, 3, wp::float32> var_78;
    wp::mat_t<3, 3, wp::float32> var_79;
    const wp::int32 var_80 = 0;
    const wp::int32 var_81 = 0;
    wp::float32 var_82;
    const wp::int32 var_83 = 0;
    const wp::int32 var_84 = 0;
    wp::float32 var_85;
    wp::float32 var_86;
    const wp::int32 var_87 = 1;
    const wp::int32 var_88 = 0;
    wp::float32 var_89;
    const wp::int32 var_90 = 1;
    const wp::int32 var_91 = 0;
    wp::float32 var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    const wp::int32 var_95 = 2;
    const wp::int32 var_96 = 0;
    wp::float32 var_97;
    const wp::int32 var_98 = 2;
    const wp::int32 var_99 = 0;
    wp::float32 var_100;
    wp::float32 var_101;
    wp::float32 var_102;
    const wp::int32 var_103 = 0;
    const wp::int32 var_104 = 1;
    wp::float32 var_105;
    const wp::int32 var_106 = 0;
    const wp::int32 var_107 = 1;
    wp::float32 var_108;
    wp::float32 var_109;
    const wp::int32 var_110 = 1;
    const wp::int32 var_111 = 1;
    wp::float32 var_112;
    const wp::int32 var_113 = 1;
    const wp::int32 var_114 = 1;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    const wp::int32 var_118 = 2;
    const wp::int32 var_119 = 1;
    wp::float32 var_120;
    const wp::int32 var_121 = 2;
    const wp::int32 var_122 = 1;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    const wp::int32 var_126 = 0;
    const wp::int32 var_127 = 2;
    wp::float32 var_128;
    const wp::int32 var_129 = 0;
    const wp::int32 var_130 = 2;
    wp::float32 var_131;
    wp::float32 var_132;
    const wp::int32 var_133 = 1;
    const wp::int32 var_134 = 2;
    wp::float32 var_135;
    const wp::int32 var_136 = 1;
    const wp::int32 var_137 = 2;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::float32 var_140;
    const wp::int32 var_141 = 2;
    const wp::int32 var_142 = 2;
    wp::float32 var_143;
    const wp::int32 var_144 = 2;
    const wp::int32 var_145 = 2;
    wp::float32 var_146;
    wp::float32 var_147;
    wp::float32 var_148;
    wp::vec_t<3, wp::float32> var_149;
    const wp::int32 var_150 = 0;
    const wp::int32 var_151 = 1;
    wp::mat_t<3, 3, wp::float32> var_152;
    wp::mat_t<3, 3, wp::float32> var_153;
    wp::vec_t<3, wp::float32> var_154;
    const wp::int32 var_155 = 0;
    const wp::int32 var_156 = 2;
    wp::mat_t<3, 3, wp::float32> var_157;
    wp::mat_t<3, 3, wp::float32> var_158;
    wp::vec_t<3, wp::float32> var_159;
    const wp::int32 var_160 = 1;
    const wp::int32 var_161 = 2;
    wp::mat_t<3, 3, wp::float32> var_162;
    wp::mat_t<3, 3, wp::float32> var_163;
    wp::vec_t<3, wp::float32> var_164;
    wp::float32 var_165;
    const wp::float32 var_166 = 0.0;
    bool var_167;
    const wp::int32 var_168 = 0;
    const wp::int32 var_169 = 2;
    wp::float32 var_170;
    wp::float32 var_171;
    const wp::int32 var_172 = 2;
    const wp::int32 var_173 = 2;
    wp::float32 var_174;
    wp::float32 var_175;
    const wp::int32 var_176 = 2;
    const wp::int32 var_177 = 1;
    const wp::int32 var_178 = 2;
    wp::float32 var_179;
    wp::float32 var_180;
    const wp::int32 var_181 = 2;
    const wp::int32 var_182 = 2;
    wp::float32 var_183;
    wp::float32 var_184;
    const wp::int32 var_185 = 2;
    const wp::int32 var_186 = 2;
    const wp::int32 var_187 = 2;
    wp::float32 var_188;
    wp::float32 var_189;
    const wp::int32 var_190 = 2;
    const wp::int32 var_191 = 2;
    wp::float32 var_192;
    wp::float32 var_193;
    const wp::int32 var_194 = 2;
    const wp::int32 var_195 = 0;
    const wp::int32 var_196 = 0;
    wp::float32 var_197;
    const wp::int32 var_198 = 1;
    const wp::int32 var_199 = 0;
    wp::float32 var_200;
    const wp::int32 var_201 = 2;
    const wp::int32 var_202 = 0;
    wp::float32 var_203;
    wp::vec_t<3, wp::float32> var_204;
    const wp::int32 var_205 = 0;
    const wp::int32 var_206 = 1;
    wp::float32 var_207;
    const wp::int32 var_208 = 1;
    const wp::int32 var_209 = 1;
    wp::float32 var_210;
    const wp::int32 var_211 = 2;
    const wp::int32 var_212 = 1;
    wp::float32 var_213;
    wp::vec_t<3, wp::float32> var_214;
    const wp::int32 var_215 = 0;
    const wp::int32 var_216 = 2;
    wp::float32 var_217;
    const wp::int32 var_218 = 1;
    const wp::int32 var_219 = 2;
    wp::float32 var_220;
    const wp::int32 var_221 = 2;
    const wp::int32 var_222 = 2;
    wp::float32 var_223;
    wp::vec_t<3, wp::float32> var_224;
    wp::float32 var_225;
    const wp::float32 var_226 = 0.0;
    bool var_227;
    const wp::int32 var_228 = 3;
    wp::mat_t<3, 3, wp::float32> var_229;
    const wp::float32 var_230 = 0.0;
    const wp::float32 var_231 = 0.0;
    const wp::float32 var_232 = 0.0;
    wp::vec_t<3, wp::float32> var_233;
    const wp::float32 var_234 = 1.0;
    wp::float32 var_235;
    wp::vec_t<3, wp::float32> var_236;
    wp::float32 var_237;
    wp::vec_t<3, wp::float32> var_238;
    wp::vec_t<3, wp::float32> var_239;
    wp::float32 var_240;
    const wp::float32 var_241 = 1e-06;
    wp::float32 var_242;
    bool var_243;
    const wp::float32 var_244 = 1.0;
    wp::float32 var_245;
    wp::vec_t<3, wp::float32> var_246;
    wp::vec_t<3, wp::float32> var_247;
    const wp::int32 var_248 = 0;
    wp::int32 var_249;
    const wp::int32 var_250 = 1;
    wp::float32 var_251;
    wp::float32 var_252;
    wp::float32 var_253;
    wp::float32 var_254;
    bool var_255;
    const wp::int32 var_256 = 1;
    wp::int32 var_257;
    const wp::int32 var_258 = 2;
    wp::float32 var_259;
    wp::float32 var_260;
    wp::float32 var_261;
    wp::float32 var_262;
    bool var_263;
    const wp::int32 var_264 = 2;
    wp::int32 var_265;
    const wp::int32 var_266 = 0;
    bool var_267;
    const wp::float32 var_268 = 1.0;
    const wp::float32 var_269 = 0.0;
    wp::float32 var_270;
    wp::float32 var_271;
    const wp::int32 var_272 = 0;
    wp::float32 var_273;
    wp::float32 var_274;
    wp::float32 var_275;
    const wp::int32 var_276 = 1;
    bool var_277;
    const wp::float32 var_278 = 1.0;
    const wp::float32 var_279 = 0.0;
    wp::float32 var_280;
    wp::float32 var_281;
    const wp::int32 var_282 = 1;
    wp::float32 var_283;
    wp::float32 var_284;
    wp::float32 var_285;
    const wp::int32 var_286 = 2;
    bool var_287;
    const wp::float32 var_288 = 1.0;
    const wp::float32 var_289 = 0.0;
    wp::float32 var_290;
    wp::float32 var_291;
    const wp::int32 var_292 = 2;
    wp::float32 var_293;
    wp::float32 var_294;
    wp::float32 var_295;
    wp::vec_t<3, wp::float32> var_296;
    wp::vec_t<3, wp::float32> var_297;
    wp::vec_t<3, wp::float32> var_298;
    wp::vec_t<3, wp::float32> var_299;
    const wp::int32 var_300 = 0;
    const wp::int32 var_301 = 1;
    wp::float32 var_302;
    const wp::int32 var_303 = 1;
    const wp::int32 var_304 = 1;
    wp::float32 var_305;
    const wp::int32 var_306 = 2;
    const wp::int32 var_307 = 1;
    wp::float32 var_308;
    wp::vec_t<3, wp::float32> var_309;
    const wp::int32 var_310 = 0;
    const wp::int32 var_311 = 2;
    wp::float32 var_312;
    const wp::int32 var_313 = 1;
    const wp::int32 var_314 = 2;
    wp::float32 var_315;
    const wp::int32 var_316 = 2;
    const wp::int32 var_317 = 2;
    wp::float32 var_318;
    wp::vec_t<3, wp::float32> var_319;
    wp::float32 var_320;
    wp::float32 var_321;
    wp::vec_t<3, wp::float32> var_322;
    wp::vec_t<3, wp::float32> var_323;
    const wp::int32 var_324 = 0;
    wp::float32 var_325;
    const wp::int32 var_326 = 0;
    wp::float32 var_327;
    const wp::int32 var_328 = 0;
    wp::float32 var_329;
    const wp::int32 var_330 = 1;
    wp::float32 var_331;
    const wp::int32 var_332 = 1;
    wp::float32 var_333;
    const wp::int32 var_334 = 1;
    wp::float32 var_335;
    const wp::int32 var_336 = 2;
    wp::float32 var_337;
    const wp::int32 var_338 = 2;
    wp::float32 var_339;
    const wp::int32 var_340 = 2;
    wp::float32 var_341;
    wp::mat_t<3, 3, wp::float32> var_342;
    //---------
    // forward
    // def _snh_svd(F: wp.mat33) -> tuple[wp.mat33, wp.vec3, wp.mat33]:                       <L 995>
    // scale = float(0.0)                                                                     <L 996>
    var_1 = wp::float(var_0);
    // for row in range(3):                                                                   <L 997>
    // for col in range(3):                                                                   <L 998>
    // scale = wp.max(scale, wp.abs(F[row, col]))                                             <L 999>
    var_4 = wp::extract(var_F, var_2, var_3);
    var_5 = wp::abs(var_4);
    var_6 = wp::max(var_1, var_5);
    var_8 = wp::extract(var_F, var_2, var_7);
    var_9 = wp::abs(var_8);
    var_10 = wp::max(var_6, var_9);
    var_12 = wp::extract(var_F, var_2, var_11);
    var_13 = wp::abs(var_12);
    var_14 = wp::max(var_10, var_13);
    // for col in range(3):                                                                   <L 998>
    // scale = wp.max(scale, wp.abs(F[row, col]))                                             <L 999>
    var_17 = wp::extract(var_F, var_15, var_16);
    var_18 = wp::abs(var_17);
    var_19 = wp::max(var_14, var_18);
    var_21 = wp::extract(var_F, var_15, var_20);
    var_22 = wp::abs(var_21);
    var_23 = wp::max(var_19, var_22);
    var_25 = wp::extract(var_F, var_15, var_24);
    var_26 = wp::abs(var_25);
    var_27 = wp::max(var_23, var_26);
    // for col in range(3):                                                                   <L 998>
    // scale = wp.max(scale, wp.abs(F[row, col]))                                             <L 999>
    var_30 = wp::extract(var_F, var_28, var_29);
    var_31 = wp::abs(var_30);
    var_32 = wp::max(var_27, var_31);
    var_34 = wp::extract(var_F, var_28, var_33);
    var_35 = wp::abs(var_34);
    var_36 = wp::max(var_32, var_35);
    var_38 = wp::extract(var_F, var_28, var_37);
    var_39 = wp::abs(var_38);
    var_40 = wp::max(var_36, var_39);
    // A = F * (1.0 / scale) if scale > 0.0 else wp.mat33(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)       <L 1000>
    var_42 = (var_40 > var_41);
    if (var_42) {
        var_44 = wp::div(var_43, var_40);
        var_45 = wp::mul(var_F, var_44);
    }
    if (!var_42) {
        var_55 = wp::mat_t<3, 3, wp::float32>(var_46, var_47, var_48, var_49, var_50, var_51, var_52, var_53, var_54);
    }
    var_56 = wp::where(var_42, var_45, var_55);
    // V = wp.identity(3, dtype=float)                                                        <L 1001>
    var_58 = wp::identity<3, wp::float32>();
    // for _ in range(24):                                                                    <L 1002>
    var_60 = wp::range(var_59);
    start_for_0:;
        if (iter_cmp(var_60) == 0) goto end_for_0;
        var_61 = wp::iter_next(var_60);
        // A, V, d01 = _snh_jacobi_rot(A, V, 0, 1)                                            <L 1003>
        _snh_jacobi_rot_0(var_56, var_58, var_62, var_63, var_64, var_65, var_66);
        // A, V, d02 = _snh_jacobi_rot(A, V, 0, 2)                                            <L 1004>
        _snh_jacobi_rot_0(var_64, var_65, var_67, var_68, var_69, var_70, var_71);
        // A, V, d12 = _snh_jacobi_rot(A, V, 1, 2)                                            <L 1005>
        _snh_jacobi_rot_0(var_69, var_70, var_72, var_73, var_74, var_75, var_76);
        // if d01 and d02 and d12:                                                            <L 1006>
        var_77 = var_66;
        if (var_77) {
            var_77 = var_77 && var_71;
        }
        if (var_77) {
            var_77 = var_77 && var_76;
        }
        if (var_77) {
            // break                                                                          <L 1007>
            wp::assign(var_56, var_74);
            wp::assign(var_58, var_75);
            goto end_for_0;
        }
        var_78 = wp::where(var_77, var_56, var_74);
        var_79 = wp::where(var_77, var_58, var_75);
        wp::assign(var_56, var_78);
        wp::assign(var_58, var_79);
        goto start_for_0;
    end_for_0:;
    // s = wp.vec3(                                                                           <L 1009>
    // A[0, 0] * A[0, 0] + A[1, 0] * A[1, 0] + A[2, 0] * A[2, 0],                             <L 1010>
    var_82 = wp::extract(var_56, var_80, var_81);
    var_85 = wp::extract(var_56, var_83, var_84);
    var_86 = wp::mul(var_82, var_85);
    var_89 = wp::extract(var_56, var_87, var_88);
    var_92 = wp::extract(var_56, var_90, var_91);
    var_93 = wp::mul(var_89, var_92);
    var_94 = wp::add(var_86, var_93);
    var_97 = wp::extract(var_56, var_95, var_96);
    var_100 = wp::extract(var_56, var_98, var_99);
    var_101 = wp::mul(var_97, var_100);
    var_102 = wp::add(var_94, var_101);
    // A[0, 1] * A[0, 1] + A[1, 1] * A[1, 1] + A[2, 1] * A[2, 1],                             <L 1011>
    var_105 = wp::extract(var_56, var_103, var_104);
    var_108 = wp::extract(var_56, var_106, var_107);
    var_109 = wp::mul(var_105, var_108);
    var_112 = wp::extract(var_56, var_110, var_111);
    var_115 = wp::extract(var_56, var_113, var_114);
    var_116 = wp::mul(var_112, var_115);
    var_117 = wp::add(var_109, var_116);
    var_120 = wp::extract(var_56, var_118, var_119);
    var_123 = wp::extract(var_56, var_121, var_122);
    var_124 = wp::mul(var_120, var_123);
    var_125 = wp::add(var_117, var_124);
    // A[0, 2] * A[0, 2] + A[1, 2] * A[1, 2] + A[2, 2] * A[2, 2],                             <L 1012>
    var_128 = wp::extract(var_56, var_126, var_127);
    var_131 = wp::extract(var_56, var_129, var_130);
    var_132 = wp::mul(var_128, var_131);
    var_135 = wp::extract(var_56, var_133, var_134);
    var_138 = wp::extract(var_56, var_136, var_137);
    var_139 = wp::mul(var_135, var_138);
    var_140 = wp::add(var_132, var_139);
    var_143 = wp::extract(var_56, var_141, var_142);
    var_146 = wp::extract(var_56, var_144, var_145);
    var_147 = wp::mul(var_143, var_146);
    var_148 = wp::add(var_140, var_147);
    var_149 = wp::vec_t<3, wp::float32>(var_102, var_125, var_148);
    // A, V, s = _snh_swap_cols(A, V, s, 0, 1)                                                <L 1014>
    _snh_swap_cols_0(var_56, var_58, var_149, var_150, var_151, var_152, var_153, var_154);
    // A, V, s = _snh_swap_cols(A, V, s, 0, 2)                                                <L 1015>
    _snh_swap_cols_0(var_152, var_153, var_154, var_155, var_156, var_157, var_158, var_159);
    // A, V, s = _snh_swap_cols(A, V, s, 1, 2)                                                <L 1016>
    _snh_swap_cols_0(var_157, var_158, var_159, var_160, var_161, var_162, var_163, var_164);
    // if wp.determinant(V) < 0.0:                                                            <L 1018>
    var_165 = wp::determinant(var_163);
    var_167 = (var_165 < var_166);
    if (var_167) {
        // for r in range(3):                                                                 <L 1019>
        // V[r, 2] = -V[r, 2]                                                                 <L 1020>
        var_170 = wp::extract(var_163, var_168, var_169);
        var_171 = wp::neg(var_170);
        wp::assign_inplace(var_163, var_168, var_172, var_171);
        // A[r, 2] = -A[r, 2]                                                                 <L 1021>
        var_174 = wp::extract(var_162, var_168, var_173);
        var_175 = wp::neg(var_174);
        wp::assign_inplace(var_162, var_168, var_176, var_175);
        // V[r, 2] = -V[r, 2]                                                                 <L 1020>
        var_179 = wp::extract(var_163, var_177, var_178);
        var_180 = wp::neg(var_179);
        wp::assign_inplace(var_163, var_177, var_181, var_180);
        // A[r, 2] = -A[r, 2]                                                                 <L 1021>
        var_183 = wp::extract(var_162, var_177, var_182);
        var_184 = wp::neg(var_183);
        wp::assign_inplace(var_162, var_177, var_185, var_184);
        // V[r, 2] = -V[r, 2]                                                                 <L 1020>
        var_188 = wp::extract(var_163, var_186, var_187);
        var_189 = wp::neg(var_188);
        wp::assign_inplace(var_163, var_186, var_190, var_189);
        // A[r, 2] = -A[r, 2]                                                                 <L 1021>
        var_192 = wp::extract(var_162, var_186, var_191);
        var_193 = wp::neg(var_192);
        wp::assign_inplace(var_162, var_186, var_194, var_193);
    }
    // u0 = wp.vec3(A[0, 0], A[1, 0], A[2, 0])                                                <L 1023>
    var_197 = wp::extract(var_162, var_195, var_196);
    var_200 = wp::extract(var_162, var_198, var_199);
    var_203 = wp::extract(var_162, var_201, var_202);
    var_204 = wp::vec_t<3, wp::float32>(var_197, var_200, var_203);
    // u1 = wp.vec3(A[0, 1], A[1, 1], A[2, 1])                                                <L 1024>
    var_207 = wp::extract(var_162, var_205, var_206);
    var_210 = wp::extract(var_162, var_208, var_209);
    var_213 = wp::extract(var_162, var_211, var_212);
    var_214 = wp::vec_t<3, wp::float32>(var_207, var_210, var_213);
    // u2 = wp.vec3(A[0, 2], A[1, 2], A[2, 2])                                                <L 1025>
    var_217 = wp::extract(var_162, var_215, var_216);
    var_220 = wp::extract(var_162, var_218, var_219);
    var_223 = wp::extract(var_162, var_221, var_222);
    var_224 = wp::vec_t<3, wp::float32>(var_217, var_220, var_223);
    // s0 = wp.length(u0)                                                                     <L 1026>
    var_225 = wp::length(var_204);
    // if s0 == 0.0:                                                                          <L 1027>
    var_227 = (var_225 == var_226);
    if (var_227) {
        // return wp.identity(3, dtype=float), wp.vec3(0.0, 0.0, 0.0), V                      <L 1028>
        var_229 = wp::identity<3, wp::float32>();
        var_233 = wp::vec_t<3, wp::float32>(var_230, var_231, var_232);
        ret_0 = var_229;
        ret_1 = var_233;
        ret_2 = var_163;
        return;
    }
    // u0 = u0 * (1.0 / s0)                                                                   <L 1030>
    var_235 = wp::div(var_234, var_225);
    var_236 = wp::mul(var_204, var_235);
    // u1 = u1 - wp.dot(u0, u1) * u0                                                          <L 1031>
    var_237 = wp::dot(var_236, var_214);
    var_238 = wp::mul(var_237, var_236);
    var_239 = wp::sub(var_214, var_238);
    // norm = wp.length(u1)                                                                   <L 1032>
    var_240 = wp::length(var_239);
    // if norm > 1.0e-6 * s0:                                                                 <L 1033>
    var_242 = wp::mul(var_241, var_225);
    var_243 = (var_240 > var_242);
    if (var_243) {
        // u1 = u1 * (1.0 / norm)                                                             <L 1034>
        var_245 = wp::div(var_244, var_240);
        var_246 = wp::mul(var_239, var_245);
    }
    var_247 = wp::where(var_243, var_246, var_239);
    if (!var_243) {
        // axis = int(0)                                                                      <L 1036>
        var_249 = wp::int(var_248);
        // if wp.abs(u0[1]) < wp.abs(u0[axis]):                                               <L 1037>
        var_251 = wp::extract(var_236, var_250);
        var_252 = wp::abs(var_251);
        var_253 = wp::extract(var_236, var_249);
        var_254 = wp::abs(var_253);
        var_255 = (var_252 < var_254);
        if (var_255) {
            // axis = 1                                                                       <L 1038>
        }
        var_257 = wp::where(var_255, var_256, var_249);
        // if wp.abs(u0[2]) < wp.abs(u0[axis]):                                               <L 1039>
        var_259 = wp::extract(var_236, var_258);
        var_260 = wp::abs(var_259);
        var_261 = wp::extract(var_236, var_257);
        var_262 = wp::abs(var_261);
        var_263 = (var_260 < var_262);
        if (var_263) {
            // axis = 2                                                                       <L 1040>
        }
        var_265 = wp::where(var_263, var_264, var_257);
        // u1 = wp.vec3(                                                                      <L 1041>
        // (1.0 if axis == 0 else 0.0) - u0[axis] * u0[0],                                    <L 1042>
        var_267 = (var_265 == var_266);
        if (var_267) {
        }
        if (!var_267) {
        }
        var_270 = wp::where(var_267, var_268, var_269);
        var_271 = wp::extract(var_236, var_265);
        var_273 = wp::extract(var_236, var_272);
        var_274 = wp::mul(var_271, var_273);
        var_275 = wp::sub(var_270, var_274);
        // (1.0 if axis == 1 else 0.0) - u0[axis] * u0[1],                                    <L 1043>
        var_277 = (var_265 == var_276);
        if (var_277) {
        }
        if (!var_277) {
        }
        var_280 = wp::where(var_277, var_278, var_279);
        var_281 = wp::extract(var_236, var_265);
        var_283 = wp::extract(var_236, var_282);
        var_284 = wp::mul(var_281, var_283);
        var_285 = wp::sub(var_280, var_284);
        // (1.0 if axis == 2 else 0.0) - u0[axis] * u0[2],                                    <L 1044>
        var_287 = (var_265 == var_286);
        if (var_287) {
        }
        if (!var_287) {
        }
        var_290 = wp::where(var_287, var_288, var_289);
        var_291 = wp::extract(var_236, var_265);
        var_293 = wp::extract(var_236, var_292);
        var_294 = wp::mul(var_291, var_293);
        var_295 = wp::sub(var_290, var_294);
        var_296 = wp::vec_t<3, wp::float32>(var_275, var_285, var_295);
        // u1 = wp.normalize(u1)                                                              <L 1046>
        var_297 = wp::normalize(var_296);
    }
    var_298 = wp::where(var_243, var_247, var_297);
    // u2 = wp.cross(u0, u1)                                                                  <L 1048>
    var_299 = wp::cross(var_236, var_298);
    // a1 = wp.vec3(A[0, 1], A[1, 1], A[2, 1])                                                <L 1049>
    var_302 = wp::extract(var_162, var_300, var_301);
    var_305 = wp::extract(var_162, var_303, var_304);
    var_308 = wp::extract(var_162, var_306, var_307);
    var_309 = wp::vec_t<3, wp::float32>(var_302, var_305, var_308);
    // a2 = wp.vec3(A[0, 2], A[1, 2], A[2, 2])                                                <L 1050>
    var_312 = wp::extract(var_162, var_310, var_311);
    var_315 = wp::extract(var_162, var_313, var_314);
    var_318 = wp::extract(var_162, var_316, var_317);
    var_319 = wp::vec_t<3, wp::float32>(var_312, var_315, var_318);
    // sigma = wp.vec3(s0, wp.dot(u1, a1), wp.dot(u2, a2)) * scale                            <L 1051>
    var_320 = wp::dot(var_298, var_309);
    var_321 = wp::dot(var_299, var_319);
    var_322 = wp::vec_t<3, wp::float32>(var_225, var_320, var_321);
    var_323 = wp::mul(var_322, var_40);
    // U = wp.mat33(u0[0], u1[0], u2[0], u0[1], u1[1], u2[1], u0[2], u1[2], u2[2])            <L 1052>
    var_325 = wp::extract(var_236, var_324);
    var_327 = wp::extract(var_298, var_326);
    var_329 = wp::extract(var_299, var_328);
    var_331 = wp::extract(var_236, var_330);
    var_333 = wp::extract(var_298, var_332);
    var_335 = wp::extract(var_299, var_334);
    var_337 = wp::extract(var_236, var_336);
    var_339 = wp::extract(var_298, var_338);
    var_341 = wp::extract(var_299, var_340);
    var_342 = wp::mat_t<3, 3, wp::float32>(var_325, var_327, var_329, var_331, var_333, var_335, var_337, var_339, var_341);
    // return U, sigma, V                                                                     <L 1053>
    ret_0 = var_342;
    ret_1 = var_323;
    ret_2 = var_163;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1097
static CUDA_CALLABLE wp::mat_t<6, 6, wp::float32> stretch_metric_0(
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::int32 var_nedge)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::mat_t<6, 6, wp::float32> var_1;
    const wp::int32 var_2 = 0;
    wp::int32 var_3;
    wp::range_t var_4;
    wp::int32 var_5;
    wp::range_t var_6;
    wp::int32 var_7;
    wp::int32 var_8;
    wp::float32* var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    //---------
    // forward
    // def stretch_metric(                                                                    <L 1098>
    // metric = mat66(0.0)                                                                    <L 1105>
    var_1 = wp::mat_t<6, 6, wp::float32>(var_0);
    // idx = int(0)                                                                           <L 1106>
    var_3 = wp::int(var_2);
    // for e1 in range(nedge):                                                                <L 1107>
    var_4 = wp::range(var_nedge);
    start_for_0:;
        if (iter_cmp(var_4) == 0) goto end_for_0;
        var_5 = wp::iter_next(var_4);
        // for e2 in range(e1, nedge):                                                        <L 1108>
        var_6 = wp::range(var_5, var_nedge);
        start_for_2:;
            if (iter_cmp(var_6) == 0) goto end_for_2;
            var_7 = wp::iter_next(var_6);
            // val = flex_stiffness[stiffness_adr + idx]                                      <L 1109>
            var_8 = wp::add(var_stiffness_adr, var_3);
            var_9 = wp::address(var_flex_stiffness, var_8);
            var_11 = wp::load(var_9);
            var_10 = wp::copy(var_11);
            // metric[e1, e2] = val                                                           <L 1110>
            wp::assign_inplace(var_1, var_5, var_7, var_10);
            // metric[e2, e1] = val                                                           <L 1111>
            wp::assign_inplace(var_1, var_7, var_5, var_10);
            // idx += 1                                                                       <L 1112>
            var_13 = wp::add(var_3, var_12);
            wp::assign(var_3, var_13);
            goto start_for_2;
        end_for_2:;
        goto start_for_0;
    end_for_0:;
    // return metric                                                                          <L 1113>
    return var_1;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1116
static CUDA_CALLABLE wp::vec_t<6, wp::float32> stretch_tension_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_elongation,
    wp::int32 var_nedge)
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
    wp::range_t var_7;
    wp::int32 var_8;
    const wp::float32 var_9 = 0.0;
    wp::float32 var_10;
    wp::range_t var_11;
    wp::int32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    //---------
    // forward
    // def stretch_tension(metric: mat66, elongation: vec6, nedge: int) -> vec6:              <L 1117>
    // tension = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                           <L 1118>
    var_6 = wp::vec_t<6, wp::float32>({var_0, var_1, var_2, var_3, var_4, var_5});
    // for e1 in range(nedge):                                                                <L 1119>
    var_7 = wp::range(var_nedge);
    start_for_0:;
        if (iter_cmp(var_7) == 0) goto end_for_0;
        var_8 = wp::iter_next(var_7);
        // t_val = float(0.0)                                                                 <L 1120>
        var_10 = wp::float(var_9);
        // for e2 in range(nedge):                                                            <L 1121>
        var_11 = wp::range(var_nedge);
        start_for_2:;
            if (iter_cmp(var_11) == 0) goto end_for_2;
            var_12 = wp::iter_next(var_11);
            // t_val += metric[e1, e2] * elongation[e2]                                       <L 1122>
            var_13 = wp::extract(var_metric, var_8, var_12);
            var_14 = wp::extract(var_elongation, var_12);
            var_15 = wp::mul(var_13, var_14);
            var_16 = wp::add(var_10, var_15);
            wp::assign(var_10, var_16);
            goto start_for_2;
        end_for_2:;
        // tension[e1] = t_val                                                                <L 1123>
        wp::assign_inplace(var_6, var_8, var_10);
        goto start_for_0;
    end_for_0:;
    // return tension                                                                         <L 1124>
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1127
static CUDA_CALLABLE void stretch_elasticity_0(
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::vec_t<6, wp::float32> var_elongation,
    wp::int32 var_nedge,
    wp::mat_t<6, 6, wp::float32> & ret_0,
    wp::vec_t<6, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::mat_t<6, 6, wp::float32> var_0;
    wp::vec_t<6, wp::float32> var_1;
    //---------
    // forward
    // def stretch_elasticity(                                                                <L 1128>
    // metric = stretch_metric(flex_stiffness, stiffness_adr, nedge)                          <L 1136>
    var_0 = stretch_metric_0(var_flex_stiffness, var_stiffness_adr, var_nedge);
    // tension = stretch_tension(metric, elongation, nedge)                                   <L 1137>
    var_1 = stretch_tension_0(var_0, var_elongation, var_nedge);
    // return metric, tension                                                                 <L 1138>
    ret_0 = var_0;
    ret_1 = var_1;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:724
static CUDA_CALLABLE wp::vec_t<6, wp::float32> _snh_cubic_0(
    wp::vec_t<6, wp::float32> var_tension,
    wp::vec_t<6, wp::float32> var_s,
    wp::float32 var_gamma)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 2;
    wp::float32 var_3;
    const wp::int32 var_4 = 4;
    wp::float32 var_5;
    const wp::float32 var_6 = 0.5;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    const wp::int32 var_9 = 2;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::float32 var_16 = 0.5;
    const wp::int32 var_17 = 0;
    wp::float32 var_18;
    const wp::int32 var_19 = 4;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::int32 var_22 = 5;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::float32 var_26 = 0.5;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    const wp::int32 var_29 = 4;
    wp::float32 var_30;
    wp::float32 var_31;
    const wp::int32 var_32 = 3;
    wp::float32 var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    wp::float32 var_53;
    const wp::float32 var_54 = 2.0;
    wp::float32 var_55;
    const wp::int32 var_56 = 0;
    wp::float32 var_57;
    wp::float32 var_58;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::int32 var_62 = 1;
    wp::float32 var_63;
    wp::float32 var_64;
    wp::float32 var_65;
    const wp::int32 var_66 = 2;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    const wp::int32 var_72 = 3;
    wp::float32 var_73;
    wp::float32 var_74;
    wp::float32 var_75;
    const wp::int32 var_76 = 4;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::float32 var_80;
    wp::float32 var_81;
    const wp::int32 var_82 = 5;
    wp::float32 var_83;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::vec_t<6, wp::float32> var_86;
    //---------
    // forward
    // def _snh_cubic(tension: vec6, s: vec6, gamma: float) -> vec6:                          <L 725>
    // a = s[0]                                                                               <L 726>
    var_1 = wp::extract(var_s, var_0);
    // b = s[2]                                                                               <L 727>
    var_3 = wp::extract(var_s, var_2);
    // c = s[4]                                                                               <L 728>
    var_5 = wp::extract(var_s, var_4);
    // d = 0.5 * (s[0] + s[2] - s[1])                                                         <L 729>
    var_8 = wp::extract(var_s, var_7);
    var_10 = wp::extract(var_s, var_9);
    var_11 = wp::add(var_8, var_10);
    var_13 = wp::extract(var_s, var_12);
    var_14 = wp::sub(var_11, var_13);
    var_15 = wp::mul(var_6, var_14);
    // e = 0.5 * (s[0] + s[4] - s[5])                                                         <L 730>
    var_18 = wp::extract(var_s, var_17);
    var_20 = wp::extract(var_s, var_19);
    var_21 = wp::add(var_18, var_20);
    var_23 = wp::extract(var_s, var_22);
    var_24 = wp::sub(var_21, var_23);
    var_25 = wp::mul(var_16, var_24);
    // f = 0.5 * (s[2] + s[4] - s[3])                                                         <L 731>
    var_28 = wp::extract(var_s, var_27);
    var_30 = wp::extract(var_s, var_29);
    var_31 = wp::add(var_28, var_30);
    var_33 = wp::extract(var_s, var_32);
    var_34 = wp::sub(var_31, var_33);
    var_35 = wp::mul(var_26, var_34);
    // cof0 = b * c - f * f                                                                   <L 732>
    var_36 = wp::mul(var_3, var_5);
    var_37 = wp::mul(var_35, var_35);
    var_38 = wp::sub(var_36, var_37);
    // cof1 = a * c - e * e                                                                   <L 733>
    var_39 = wp::mul(var_1, var_5);
    var_40 = wp::mul(var_25, var_25);
    var_41 = wp::sub(var_39, var_40);
    // cof2 = a * b - d * d                                                                   <L 734>
    var_42 = wp::mul(var_1, var_3);
    var_43 = wp::mul(var_15, var_15);
    var_44 = wp::sub(var_42, var_43);
    // cof3 = e * f - c * d                                                                   <L 735>
    var_45 = wp::mul(var_25, var_35);
    var_46 = wp::mul(var_5, var_15);
    var_47 = wp::sub(var_45, var_46);
    // cof4 = d * f - b * e                                                                   <L 736>
    var_48 = wp::mul(var_15, var_35);
    var_49 = wp::mul(var_3, var_25);
    var_50 = wp::sub(var_48, var_49);
    // cof5 = d * e - a * f                                                                   <L 737>
    var_51 = wp::mul(var_15, var_25);
    var_52 = wp::mul(var_1, var_35);
    var_53 = wp::sub(var_51, var_52);
    // scale = 2.0 * gamma                                                                    <L 738>
    var_55 = wp::mul(var_54, var_gamma);
    // return vec6(                                                                           <L 739>
    // tension[0] + scale * (cof0 + cof3 + cof4),                                             <L 740>
    var_57 = wp::extract(var_tension, var_56);
    var_58 = wp::add(var_38, var_47);
    var_59 = wp::add(var_58, var_50);
    var_60 = wp::mul(var_55, var_59);
    var_61 = wp::add(var_57, var_60);
    // tension[1] - scale * cof3,                                                             <L 741>
    var_63 = wp::extract(var_tension, var_62);
    var_64 = wp::mul(var_55, var_47);
    var_65 = wp::sub(var_63, var_64);
    // tension[2] + scale * (cof1 + cof3 + cof5),                                             <L 742>
    var_67 = wp::extract(var_tension, var_66);
    var_68 = wp::add(var_41, var_47);
    var_69 = wp::add(var_68, var_53);
    var_70 = wp::mul(var_55, var_69);
    var_71 = wp::add(var_67, var_70);
    // tension[3] - scale * cof5,                                                             <L 743>
    var_73 = wp::extract(var_tension, var_72);
    var_74 = wp::mul(var_55, var_53);
    var_75 = wp::sub(var_73, var_74);
    // tension[4] + scale * (cof2 + cof4 + cof5),                                             <L 744>
    var_77 = wp::extract(var_tension, var_76);
    var_78 = wp::add(var_44, var_50);
    var_79 = wp::add(var_78, var_53);
    var_80 = wp::mul(var_55, var_79);
    var_81 = wp::add(var_77, var_80);
    // tension[5] - scale * cof4,                                                             <L 745>
    var_83 = wp::extract(var_tension, var_82);
    var_84 = wp::mul(var_55, var_50);
    var_85 = wp::sub(var_83, var_84);
    var_86 = wp::vec_t<6, wp::float32>({var_61, var_65, var_71, var_75, var_81, var_85});
    return var_86;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:769
static CUDA_CALLABLE wp::mat_t<6, 6, wp::float32> _snh_cubic_metric_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_s,
    wp::float32 var_gamma)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 2;
    wp::float32 var_3;
    const wp::int32 var_4 = 4;
    wp::float32 var_5;
    const wp::float32 var_6 = 0.5;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    const wp::int32 var_9 = 2;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::float32 var_16 = 0.5;
    const wp::int32 var_17 = 0;
    wp::float32 var_18;
    const wp::int32 var_19 = 4;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::int32 var_22 = 5;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::float32 var_26 = 0.5;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    const wp::int32 var_29 = 4;
    wp::float32 var_30;
    wp::float32 var_31;
    const wp::int32 var_32 = 3;
    wp::float32 var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    const wp::float32 var_36 = 1.0;
    const wp::float32 var_37 = 0.0;
    const wp::float32 var_38 = 0.0;
    const wp::float32 var_39 = 0.5;
    const wp::float32 var_40 = 0.5;
    const wp::float32 var_41 = 0.0;
    const wp::float32 var_42 = 0.0;
    const wp::float32 var_43 = 0.0;
    const wp::float32 var_44 = 0.0;
    const wp::float32 var_45 = -0.5;
    const wp::float32 var_46 = 0.0;
    const wp::float32 var_47 = 0.0;
    const wp::float32 var_48 = 0.0;
    const wp::float32 var_49 = 1.0;
    const wp::float32 var_50 = 0.0;
    const wp::float32 var_51 = 0.5;
    const wp::float32 var_52 = 0.0;
    const wp::float32 var_53 = 0.5;
    const wp::float32 var_54 = 0.0;
    const wp::float32 var_55 = 0.0;
    const wp::float32 var_56 = 0.0;
    const wp::float32 var_57 = 0.0;
    const wp::float32 var_58 = 0.0;
    const wp::float32 var_59 = -0.5;
    const wp::float32 var_60 = 0.0;
    const wp::float32 var_61 = 0.0;
    const wp::float32 var_62 = 1.0;
    const wp::float32 var_63 = 0.0;
    const wp::float32 var_64 = 0.5;
    const wp::float32 var_65 = 0.5;
    const wp::float32 var_66 = 0.0;
    const wp::float32 var_67 = 0.0;
    const wp::float32 var_68 = 0.0;
    const wp::float32 var_69 = 0.0;
    const wp::float32 var_70 = -0.5;
    const wp::float32 var_71 = 0.0;
    const wp::int32 var_72 = 6;
    const wp::int32 var_73 = 6;
    wp::tuple_t<wp::int32, wp::int32> var_74;
    wp::mat_t<6, 6, wp::float32> var_75;
    const wp::int32 var_76 = 0;
    const wp::int32 var_77 = 0;
    wp::float32 var_78;
    const wp::int32 var_79 = 1;
    wp::float32 var_80;
    const wp::int32 var_81 = 2;
    wp::float32 var_82;
    const wp::int32 var_83 = 3;
    wp::float32 var_84;
    const wp::int32 var_85 = 4;
    wp::float32 var_86;
    const wp::int32 var_87 = 5;
    wp::float32 var_88;
    wp::float32 var_89;
    wp::float32 var_90;
    wp::float32 var_91;
    const wp::float32 var_92 = 2.0;
    wp::float32 var_93;
    wp::float32 var_94;
    wp::float32 var_95;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    const wp::float32 var_99 = 2.0;
    wp::float32 var_100;
    wp::float32 var_101;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    wp::float32 var_105;
    const wp::float32 var_106 = 2.0;
    wp::float32 var_107;
    wp::float32 var_108;
    wp::float32 var_109;
    wp::float32 var_110;
    wp::float32 var_111;
    wp::float32 var_112;
    wp::float32 var_113;
    wp::float32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    wp::float32 var_120;
    wp::float32 var_121;
    wp::float32 var_122;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    wp::float32 var_126;
    wp::float32 var_127;
    wp::float32 var_128;
    wp::float32 var_129;
    wp::float32 var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    wp::float32 var_133;
    wp::float32 var_134;
    wp::float32 var_135;
    wp::float32 var_136;
    wp::float32 var_137;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::vec_t<6, wp::float32> var_140;
    const wp::int32 var_141 = 1;
    wp::int32 var_142;
    wp::range_t var_143;
    wp::int32 var_144;
    const wp::float32 var_145 = 2.0;
    wp::float32 var_146;
    wp::float32 var_147;
    wp::float32 var_148;
    bool var_149;
    const wp::int32 var_150 = 1;
    const wp::int32 var_151 = 0;
    wp::float32 var_152;
    const wp::int32 var_153 = 1;
    wp::float32 var_154;
    const wp::int32 var_155 = 2;
    wp::float32 var_156;
    const wp::int32 var_157 = 3;
    wp::float32 var_158;
    const wp::int32 var_159 = 4;
    wp::float32 var_160;
    const wp::int32 var_161 = 5;
    wp::float32 var_162;
    wp::float32 var_163;
    wp::float32 var_164;
    wp::float32 var_165;
    const wp::float32 var_166 = 2.0;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    wp::float32 var_170;
    wp::float32 var_171;
    wp::float32 var_172;
    const wp::float32 var_173 = 2.0;
    wp::float32 var_174;
    wp::float32 var_175;
    wp::float32 var_176;
    wp::float32 var_177;
    wp::float32 var_178;
    wp::float32 var_179;
    const wp::float32 var_180 = 2.0;
    wp::float32 var_181;
    wp::float32 var_182;
    wp::float32 var_183;
    wp::float32 var_184;
    wp::float32 var_185;
    wp::float32 var_186;
    wp::float32 var_187;
    wp::float32 var_188;
    wp::float32 var_189;
    wp::float32 var_190;
    wp::float32 var_191;
    wp::float32 var_192;
    wp::float32 var_193;
    wp::float32 var_194;
    wp::float32 var_195;
    wp::float32 var_196;
    wp::float32 var_197;
    wp::float32 var_198;
    wp::float32 var_199;
    wp::float32 var_200;
    wp::float32 var_201;
    wp::float32 var_202;
    wp::float32 var_203;
    wp::float32 var_204;
    wp::float32 var_205;
    wp::float32 var_206;
    wp::float32 var_207;
    wp::float32 var_208;
    wp::float32 var_209;
    wp::float32 var_210;
    wp::float32 var_211;
    wp::float32 var_212;
    wp::float32 var_213;
    wp::vec_t<6, wp::float32> var_214;
    const wp::int32 var_215 = 1;
    wp::int32 var_216;
    wp::range_t var_217;
    wp::int32 var_218;
    const wp::float32 var_219 = 2.0;
    wp::float32 var_220;
    wp::float32 var_221;
    wp::float32 var_222;
    bool var_223;
    const wp::int32 var_224 = 2;
    const wp::int32 var_225 = 0;
    wp::float32 var_226;
    const wp::int32 var_227 = 1;
    wp::float32 var_228;
    const wp::int32 var_229 = 2;
    wp::float32 var_230;
    const wp::int32 var_231 = 3;
    wp::float32 var_232;
    const wp::int32 var_233 = 4;
    wp::float32 var_234;
    const wp::int32 var_235 = 5;
    wp::float32 var_236;
    wp::float32 var_237;
    wp::float32 var_238;
    wp::float32 var_239;
    const wp::float32 var_240 = 2.0;
    wp::float32 var_241;
    wp::float32 var_242;
    wp::float32 var_243;
    wp::float32 var_244;
    wp::float32 var_245;
    wp::float32 var_246;
    const wp::float32 var_247 = 2.0;
    wp::float32 var_248;
    wp::float32 var_249;
    wp::float32 var_250;
    wp::float32 var_251;
    wp::float32 var_252;
    wp::float32 var_253;
    const wp::float32 var_254 = 2.0;
    wp::float32 var_255;
    wp::float32 var_256;
    wp::float32 var_257;
    wp::float32 var_258;
    wp::float32 var_259;
    wp::float32 var_260;
    wp::float32 var_261;
    wp::float32 var_262;
    wp::float32 var_263;
    wp::float32 var_264;
    wp::float32 var_265;
    wp::float32 var_266;
    wp::float32 var_267;
    wp::float32 var_268;
    wp::float32 var_269;
    wp::float32 var_270;
    wp::float32 var_271;
    wp::float32 var_272;
    wp::float32 var_273;
    wp::float32 var_274;
    wp::float32 var_275;
    wp::float32 var_276;
    wp::float32 var_277;
    wp::float32 var_278;
    wp::float32 var_279;
    wp::float32 var_280;
    wp::float32 var_281;
    wp::float32 var_282;
    wp::float32 var_283;
    wp::float32 var_284;
    wp::float32 var_285;
    wp::float32 var_286;
    wp::float32 var_287;
    wp::vec_t<6, wp::float32> var_288;
    const wp::int32 var_289 = 1;
    wp::int32 var_290;
    wp::range_t var_291;
    wp::int32 var_292;
    const wp::float32 var_293 = 2.0;
    wp::float32 var_294;
    wp::float32 var_295;
    wp::float32 var_296;
    bool var_297;
    const wp::int32 var_298 = 3;
    const wp::int32 var_299 = 0;
    wp::float32 var_300;
    const wp::int32 var_301 = 1;
    wp::float32 var_302;
    const wp::int32 var_303 = 2;
    wp::float32 var_304;
    const wp::int32 var_305 = 3;
    wp::float32 var_306;
    const wp::int32 var_307 = 4;
    wp::float32 var_308;
    const wp::int32 var_309 = 5;
    wp::float32 var_310;
    wp::float32 var_311;
    wp::float32 var_312;
    wp::float32 var_313;
    const wp::float32 var_314 = 2.0;
    wp::float32 var_315;
    wp::float32 var_316;
    wp::float32 var_317;
    wp::float32 var_318;
    wp::float32 var_319;
    wp::float32 var_320;
    const wp::float32 var_321 = 2.0;
    wp::float32 var_322;
    wp::float32 var_323;
    wp::float32 var_324;
    wp::float32 var_325;
    wp::float32 var_326;
    wp::float32 var_327;
    const wp::float32 var_328 = 2.0;
    wp::float32 var_329;
    wp::float32 var_330;
    wp::float32 var_331;
    wp::float32 var_332;
    wp::float32 var_333;
    wp::float32 var_334;
    wp::float32 var_335;
    wp::float32 var_336;
    wp::float32 var_337;
    wp::float32 var_338;
    wp::float32 var_339;
    wp::float32 var_340;
    wp::float32 var_341;
    wp::float32 var_342;
    wp::float32 var_343;
    wp::float32 var_344;
    wp::float32 var_345;
    wp::float32 var_346;
    wp::float32 var_347;
    wp::float32 var_348;
    wp::float32 var_349;
    wp::float32 var_350;
    wp::float32 var_351;
    wp::float32 var_352;
    wp::float32 var_353;
    wp::float32 var_354;
    wp::float32 var_355;
    wp::float32 var_356;
    wp::float32 var_357;
    wp::float32 var_358;
    wp::float32 var_359;
    wp::float32 var_360;
    wp::float32 var_361;
    wp::vec_t<6, wp::float32> var_362;
    const wp::int32 var_363 = 1;
    wp::int32 var_364;
    wp::range_t var_365;
    wp::int32 var_366;
    const wp::float32 var_367 = 2.0;
    wp::float32 var_368;
    wp::float32 var_369;
    wp::float32 var_370;
    bool var_371;
    const wp::int32 var_372 = 4;
    const wp::int32 var_373 = 0;
    wp::float32 var_374;
    const wp::int32 var_375 = 1;
    wp::float32 var_376;
    const wp::int32 var_377 = 2;
    wp::float32 var_378;
    const wp::int32 var_379 = 3;
    wp::float32 var_380;
    const wp::int32 var_381 = 4;
    wp::float32 var_382;
    const wp::int32 var_383 = 5;
    wp::float32 var_384;
    wp::float32 var_385;
    wp::float32 var_386;
    wp::float32 var_387;
    const wp::float32 var_388 = 2.0;
    wp::float32 var_389;
    wp::float32 var_390;
    wp::float32 var_391;
    wp::float32 var_392;
    wp::float32 var_393;
    wp::float32 var_394;
    const wp::float32 var_395 = 2.0;
    wp::float32 var_396;
    wp::float32 var_397;
    wp::float32 var_398;
    wp::float32 var_399;
    wp::float32 var_400;
    wp::float32 var_401;
    const wp::float32 var_402 = 2.0;
    wp::float32 var_403;
    wp::float32 var_404;
    wp::float32 var_405;
    wp::float32 var_406;
    wp::float32 var_407;
    wp::float32 var_408;
    wp::float32 var_409;
    wp::float32 var_410;
    wp::float32 var_411;
    wp::float32 var_412;
    wp::float32 var_413;
    wp::float32 var_414;
    wp::float32 var_415;
    wp::float32 var_416;
    wp::float32 var_417;
    wp::float32 var_418;
    wp::float32 var_419;
    wp::float32 var_420;
    wp::float32 var_421;
    wp::float32 var_422;
    wp::float32 var_423;
    wp::float32 var_424;
    wp::float32 var_425;
    wp::float32 var_426;
    wp::float32 var_427;
    wp::float32 var_428;
    wp::float32 var_429;
    wp::float32 var_430;
    wp::float32 var_431;
    wp::float32 var_432;
    wp::float32 var_433;
    wp::float32 var_434;
    wp::float32 var_435;
    wp::vec_t<6, wp::float32> var_436;
    const wp::int32 var_437 = 1;
    wp::int32 var_438;
    wp::range_t var_439;
    wp::int32 var_440;
    const wp::float32 var_441 = 2.0;
    wp::float32 var_442;
    wp::float32 var_443;
    wp::float32 var_444;
    bool var_445;
    const wp::int32 var_446 = 5;
    const wp::int32 var_447 = 0;
    wp::float32 var_448;
    const wp::int32 var_449 = 1;
    wp::float32 var_450;
    const wp::int32 var_451 = 2;
    wp::float32 var_452;
    const wp::int32 var_453 = 3;
    wp::float32 var_454;
    const wp::int32 var_455 = 4;
    wp::float32 var_456;
    const wp::int32 var_457 = 5;
    wp::float32 var_458;
    wp::float32 var_459;
    wp::float32 var_460;
    wp::float32 var_461;
    const wp::float32 var_462 = 2.0;
    wp::float32 var_463;
    wp::float32 var_464;
    wp::float32 var_465;
    wp::float32 var_466;
    wp::float32 var_467;
    wp::float32 var_468;
    const wp::float32 var_469 = 2.0;
    wp::float32 var_470;
    wp::float32 var_471;
    wp::float32 var_472;
    wp::float32 var_473;
    wp::float32 var_474;
    wp::float32 var_475;
    const wp::float32 var_476 = 2.0;
    wp::float32 var_477;
    wp::float32 var_478;
    wp::float32 var_479;
    wp::float32 var_480;
    wp::float32 var_481;
    wp::float32 var_482;
    wp::float32 var_483;
    wp::float32 var_484;
    wp::float32 var_485;
    wp::float32 var_486;
    wp::float32 var_487;
    wp::float32 var_488;
    wp::float32 var_489;
    wp::float32 var_490;
    wp::float32 var_491;
    wp::float32 var_492;
    wp::float32 var_493;
    wp::float32 var_494;
    wp::float32 var_495;
    wp::float32 var_496;
    wp::float32 var_497;
    wp::float32 var_498;
    wp::float32 var_499;
    wp::float32 var_500;
    wp::float32 var_501;
    wp::float32 var_502;
    wp::float32 var_503;
    wp::float32 var_504;
    wp::float32 var_505;
    wp::float32 var_506;
    wp::float32 var_507;
    wp::float32 var_508;
    wp::float32 var_509;
    wp::vec_t<6, wp::float32> var_510;
    const wp::int32 var_511 = 1;
    wp::int32 var_512;
    wp::range_t var_513;
    wp::int32 var_514;
    const wp::float32 var_515 = 2.0;
    wp::float32 var_516;
    wp::float32 var_517;
    wp::float32 var_518;
    bool var_519;
    //---------
    // forward
    // def _snh_cubic_metric(metric: mat66, s: vec6, gamma: float) -> mat66:                  <L 770>
    // a = s[0]                                                                               <L 771>
    var_1 = wp::extract(var_s, var_0);
    // b = s[2]                                                                               <L 772>
    var_3 = wp::extract(var_s, var_2);
    // c = s[4]                                                                               <L 773>
    var_5 = wp::extract(var_s, var_4);
    // d = 0.5 * (s[0] + s[2] - s[1])                                                         <L 774>
    var_8 = wp::extract(var_s, var_7);
    var_10 = wp::extract(var_s, var_9);
    var_11 = wp::add(var_8, var_10);
    var_13 = wp::extract(var_s, var_12);
    var_14 = wp::sub(var_11, var_13);
    var_15 = wp::mul(var_6, var_14);
    // e = 0.5 * (s[0] + s[4] - s[5])                                                         <L 775>
    var_18 = wp::extract(var_s, var_17);
    var_20 = wp::extract(var_s, var_19);
    var_21 = wp::add(var_18, var_20);
    var_23 = wp::extract(var_s, var_22);
    var_24 = wp::sub(var_21, var_23);
    var_25 = wp::mul(var_16, var_24);
    // f = 0.5 * (s[2] + s[4] - s[3])                                                         <L 776>
    var_28 = wp::extract(var_s, var_27);
    var_30 = wp::extract(var_s, var_29);
    var_31 = wp::add(var_28, var_30);
    var_33 = wp::extract(var_s, var_32);
    var_34 = wp::sub(var_31, var_33);
    var_35 = wp::mul(var_26, var_34);
    // basis = wp.matrix(                                                                     <L 778>
    // 1.0,                                                                                   <L 779>
    // 0.0,                                                                                   <L 780>
    // 0.0,                                                                                   <L 781>
    // 0.5,                                                                                   <L 782>
    // 0.5,                                                                                   <L 783>
    // 0.0,                                                                                   <L 784>
    // 0.0,                                                                                   <L 785>
    // 0.0,                                                                                   <L 786>
    // 0.0,                                                                                   <L 787>
    // -0.5,                                                                                  <L 788>
    // 0.0,                                                                                   <L 789>
    // 0.0,                                                                                   <L 790>
    // 0.0,                                                                                   <L 791>
    // 1.0,                                                                                   <L 792>
    // 0.0,                                                                                   <L 793>
    // 0.5,                                                                                   <L 794>
    // 0.0,                                                                                   <L 795>
    // 0.5,                                                                                   <L 796>
    // 0.0,                                                                                   <L 797>
    // 0.0,                                                                                   <L 798>
    // 0.0,                                                                                   <L 799>
    // 0.0,                                                                                   <L 800>
    // 0.0,                                                                                   <L 801>
    // -0.5,                                                                                  <L 802>
    // 0.0,                                                                                   <L 803>
    // 0.0,                                                                                   <L 804>
    // 1.0,                                                                                   <L 805>
    // 0.0,                                                                                   <L 806>
    // 0.5,                                                                                   <L 807>
    // 0.5,                                                                                   <L 808>
    // 0.0,                                                                                   <L 809>
    // 0.0,                                                                                   <L 810>
    // 0.0,                                                                                   <L 811>
    // 0.0,                                                                                   <L 812>
    // -0.5,                                                                                  <L 813>
    // 0.0,                                                                                   <L 814>
    // shape=(6, 6),                                                                          <L 815>
    var_74 = wp::tuple(var_72, var_73);
    // dtype=float,                                                                           <L 816>
    var_75 = wp::mat_t<6, 6, wp::float32>({var_36, var_37, var_38, var_39, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_57, var_58, var_59, var_60, var_61, var_62, var_63, var_64, var_65, var_66, var_67, var_68, var_69, var_70, var_71});
    // for j in range(6):                                                                     <L 818>
    // h0 = basis[j, 0]                                                                       <L 819>
    var_78 = wp::extract(var_75, var_76, var_77);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_80 = wp::extract(var_75, var_76, var_79);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_82 = wp::extract(var_75, var_76, var_81);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_84 = wp::extract(var_75, var_76, var_83);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_86 = wp::extract(var_75, var_76, var_85);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_88 = wp::extract(var_75, var_76, var_87);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_89 = wp::mul(var_80, var_5);
    var_90 = wp::mul(var_3, var_82);
    var_91 = wp::add(var_89, var_90);
    var_93 = wp::mul(var_92, var_35);
    var_94 = wp::mul(var_93, var_88);
    var_95 = wp::sub(var_91, var_94);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_96 = wp::mul(var_78, var_5);
    var_97 = wp::mul(var_1, var_82);
    var_98 = wp::add(var_96, var_97);
    var_100 = wp::mul(var_99, var_25);
    var_101 = wp::mul(var_100, var_86);
    var_102 = wp::sub(var_98, var_101);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_103 = wp::mul(var_78, var_3);
    var_104 = wp::mul(var_1, var_80);
    var_105 = wp::add(var_103, var_104);
    var_107 = wp::mul(var_106, var_15);
    var_108 = wp::mul(var_107, var_84);
    var_109 = wp::sub(var_105, var_108);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_110 = wp::mul(var_86, var_35);
    var_111 = wp::mul(var_25, var_88);
    var_112 = wp::add(var_110, var_111);
    var_113 = wp::mul(var_82, var_15);
    var_114 = wp::sub(var_112, var_113);
    var_115 = wp::mul(var_5, var_84);
    var_116 = wp::sub(var_114, var_115);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_117 = wp::mul(var_84, var_35);
    var_118 = wp::mul(var_15, var_88);
    var_119 = wp::add(var_117, var_118);
    var_120 = wp::mul(var_80, var_25);
    var_121 = wp::sub(var_119, var_120);
    var_122 = wp::mul(var_3, var_86);
    var_123 = wp::sub(var_121, var_122);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_124 = wp::mul(var_84, var_25);
    var_125 = wp::mul(var_15, var_86);
    var_126 = wp::add(var_124, var_125);
    var_127 = wp::mul(var_78, var_35);
    var_128 = wp::sub(var_126, var_127);
    var_129 = wp::mul(var_1, var_88);
    var_130 = wp::sub(var_128, var_129);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_131 = wp::add(var_95, var_116);
    var_132 = wp::add(var_131, var_123);
    var_133 = wp::neg(var_116);
    var_134 = wp::add(var_102, var_116);
    var_135 = wp::add(var_134, var_130);
    var_136 = wp::neg(var_130);
    var_137 = wp::add(var_109, var_123);
    var_138 = wp::add(var_137, var_130);
    var_139 = wp::neg(var_123);
    var_140 = wp::vec_t<6, wp::float32>({var_132, var_133, var_135, var_136, var_138, var_139});
    // for i in range(j + 1):                                                                 <L 832>
    var_142 = wp::add(var_76, var_141);
    var_143 = wp::range(var_142);
    start_for_0:;
        if (iter_cmp(var_143) == 0) goto end_for_0;
        var_144 = wp::iter_next(var_143);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_146 = wp::mul(var_145, var_gamma);
        var_147 = wp::extract(var_140, var_144);
        var_148 = wp::mul(var_146, var_147);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_144, var_76, var_148);
        // if i != j:                                                                         <L 835>
        var_149 = (var_144 != var_76);
        if (var_149) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_76, var_144, var_148);
        }
        goto start_for_0;
    end_for_0:;
    // h0 = basis[j, 0]                                                                       <L 819>
    var_152 = wp::extract(var_75, var_150, var_151);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_154 = wp::extract(var_75, var_150, var_153);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_156 = wp::extract(var_75, var_150, var_155);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_158 = wp::extract(var_75, var_150, var_157);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_160 = wp::extract(var_75, var_150, var_159);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_162 = wp::extract(var_75, var_150, var_161);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_163 = wp::mul(var_154, var_5);
    var_164 = wp::mul(var_3, var_156);
    var_165 = wp::add(var_163, var_164);
    var_167 = wp::mul(var_166, var_35);
    var_168 = wp::mul(var_167, var_162);
    var_169 = wp::sub(var_165, var_168);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_170 = wp::mul(var_152, var_5);
    var_171 = wp::mul(var_1, var_156);
    var_172 = wp::add(var_170, var_171);
    var_174 = wp::mul(var_173, var_25);
    var_175 = wp::mul(var_174, var_160);
    var_176 = wp::sub(var_172, var_175);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_177 = wp::mul(var_152, var_3);
    var_178 = wp::mul(var_1, var_154);
    var_179 = wp::add(var_177, var_178);
    var_181 = wp::mul(var_180, var_15);
    var_182 = wp::mul(var_181, var_158);
    var_183 = wp::sub(var_179, var_182);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_184 = wp::mul(var_160, var_35);
    var_185 = wp::mul(var_25, var_162);
    var_186 = wp::add(var_184, var_185);
    var_187 = wp::mul(var_156, var_15);
    var_188 = wp::sub(var_186, var_187);
    var_189 = wp::mul(var_5, var_158);
    var_190 = wp::sub(var_188, var_189);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_191 = wp::mul(var_158, var_35);
    var_192 = wp::mul(var_15, var_162);
    var_193 = wp::add(var_191, var_192);
    var_194 = wp::mul(var_154, var_25);
    var_195 = wp::sub(var_193, var_194);
    var_196 = wp::mul(var_3, var_160);
    var_197 = wp::sub(var_195, var_196);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_198 = wp::mul(var_158, var_25);
    var_199 = wp::mul(var_15, var_160);
    var_200 = wp::add(var_198, var_199);
    var_201 = wp::mul(var_152, var_35);
    var_202 = wp::sub(var_200, var_201);
    var_203 = wp::mul(var_1, var_162);
    var_204 = wp::sub(var_202, var_203);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_205 = wp::add(var_169, var_190);
    var_206 = wp::add(var_205, var_197);
    var_207 = wp::neg(var_190);
    var_208 = wp::add(var_176, var_190);
    var_209 = wp::add(var_208, var_204);
    var_210 = wp::neg(var_204);
    var_211 = wp::add(var_183, var_197);
    var_212 = wp::add(var_211, var_204);
    var_213 = wp::neg(var_197);
    var_214 = wp::vec_t<6, wp::float32>({var_206, var_207, var_209, var_210, var_212, var_213});
    // for i in range(j + 1):                                                                 <L 832>
    var_216 = wp::add(var_150, var_215);
    var_217 = wp::range(var_216);
    start_for_2:;
        if (iter_cmp(var_217) == 0) goto end_for_2;
        var_218 = wp::iter_next(var_217);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_220 = wp::mul(var_219, var_gamma);
        var_221 = wp::extract(var_214, var_218);
        var_222 = wp::mul(var_220, var_221);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_218, var_150, var_222);
        // if i != j:                                                                         <L 835>
        var_223 = (var_218 != var_150);
        if (var_223) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_150, var_218, var_222);
        }
        wp::assign(var_148, var_222);
        goto start_for_2;
    end_for_2:;
    // h0 = basis[j, 0]                                                                       <L 819>
    var_226 = wp::extract(var_75, var_224, var_225);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_228 = wp::extract(var_75, var_224, var_227);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_230 = wp::extract(var_75, var_224, var_229);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_232 = wp::extract(var_75, var_224, var_231);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_234 = wp::extract(var_75, var_224, var_233);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_236 = wp::extract(var_75, var_224, var_235);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_237 = wp::mul(var_228, var_5);
    var_238 = wp::mul(var_3, var_230);
    var_239 = wp::add(var_237, var_238);
    var_241 = wp::mul(var_240, var_35);
    var_242 = wp::mul(var_241, var_236);
    var_243 = wp::sub(var_239, var_242);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_244 = wp::mul(var_226, var_5);
    var_245 = wp::mul(var_1, var_230);
    var_246 = wp::add(var_244, var_245);
    var_248 = wp::mul(var_247, var_25);
    var_249 = wp::mul(var_248, var_234);
    var_250 = wp::sub(var_246, var_249);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_251 = wp::mul(var_226, var_3);
    var_252 = wp::mul(var_1, var_228);
    var_253 = wp::add(var_251, var_252);
    var_255 = wp::mul(var_254, var_15);
    var_256 = wp::mul(var_255, var_232);
    var_257 = wp::sub(var_253, var_256);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_258 = wp::mul(var_234, var_35);
    var_259 = wp::mul(var_25, var_236);
    var_260 = wp::add(var_258, var_259);
    var_261 = wp::mul(var_230, var_15);
    var_262 = wp::sub(var_260, var_261);
    var_263 = wp::mul(var_5, var_232);
    var_264 = wp::sub(var_262, var_263);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_265 = wp::mul(var_232, var_35);
    var_266 = wp::mul(var_15, var_236);
    var_267 = wp::add(var_265, var_266);
    var_268 = wp::mul(var_228, var_25);
    var_269 = wp::sub(var_267, var_268);
    var_270 = wp::mul(var_3, var_234);
    var_271 = wp::sub(var_269, var_270);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_272 = wp::mul(var_232, var_25);
    var_273 = wp::mul(var_15, var_234);
    var_274 = wp::add(var_272, var_273);
    var_275 = wp::mul(var_226, var_35);
    var_276 = wp::sub(var_274, var_275);
    var_277 = wp::mul(var_1, var_236);
    var_278 = wp::sub(var_276, var_277);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_279 = wp::add(var_243, var_264);
    var_280 = wp::add(var_279, var_271);
    var_281 = wp::neg(var_264);
    var_282 = wp::add(var_250, var_264);
    var_283 = wp::add(var_282, var_278);
    var_284 = wp::neg(var_278);
    var_285 = wp::add(var_257, var_271);
    var_286 = wp::add(var_285, var_278);
    var_287 = wp::neg(var_271);
    var_288 = wp::vec_t<6, wp::float32>({var_280, var_281, var_283, var_284, var_286, var_287});
    // for i in range(j + 1):                                                                 <L 832>
    var_290 = wp::add(var_224, var_289);
    var_291 = wp::range(var_290);
    start_for_4:;
        if (iter_cmp(var_291) == 0) goto end_for_4;
        var_292 = wp::iter_next(var_291);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_294 = wp::mul(var_293, var_gamma);
        var_295 = wp::extract(var_288, var_292);
        var_296 = wp::mul(var_294, var_295);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_292, var_224, var_296);
        // if i != j:                                                                         <L 835>
        var_297 = (var_292 != var_224);
        if (var_297) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_224, var_292, var_296);
        }
        wp::assign(var_148, var_296);
        goto start_for_4;
    end_for_4:;
    // h0 = basis[j, 0]                                                                       <L 819>
    var_300 = wp::extract(var_75, var_298, var_299);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_302 = wp::extract(var_75, var_298, var_301);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_304 = wp::extract(var_75, var_298, var_303);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_306 = wp::extract(var_75, var_298, var_305);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_308 = wp::extract(var_75, var_298, var_307);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_310 = wp::extract(var_75, var_298, var_309);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_311 = wp::mul(var_302, var_5);
    var_312 = wp::mul(var_3, var_304);
    var_313 = wp::add(var_311, var_312);
    var_315 = wp::mul(var_314, var_35);
    var_316 = wp::mul(var_315, var_310);
    var_317 = wp::sub(var_313, var_316);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_318 = wp::mul(var_300, var_5);
    var_319 = wp::mul(var_1, var_304);
    var_320 = wp::add(var_318, var_319);
    var_322 = wp::mul(var_321, var_25);
    var_323 = wp::mul(var_322, var_308);
    var_324 = wp::sub(var_320, var_323);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_325 = wp::mul(var_300, var_3);
    var_326 = wp::mul(var_1, var_302);
    var_327 = wp::add(var_325, var_326);
    var_329 = wp::mul(var_328, var_15);
    var_330 = wp::mul(var_329, var_306);
    var_331 = wp::sub(var_327, var_330);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_332 = wp::mul(var_308, var_35);
    var_333 = wp::mul(var_25, var_310);
    var_334 = wp::add(var_332, var_333);
    var_335 = wp::mul(var_304, var_15);
    var_336 = wp::sub(var_334, var_335);
    var_337 = wp::mul(var_5, var_306);
    var_338 = wp::sub(var_336, var_337);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_339 = wp::mul(var_306, var_35);
    var_340 = wp::mul(var_15, var_310);
    var_341 = wp::add(var_339, var_340);
    var_342 = wp::mul(var_302, var_25);
    var_343 = wp::sub(var_341, var_342);
    var_344 = wp::mul(var_3, var_308);
    var_345 = wp::sub(var_343, var_344);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_346 = wp::mul(var_306, var_25);
    var_347 = wp::mul(var_15, var_308);
    var_348 = wp::add(var_346, var_347);
    var_349 = wp::mul(var_300, var_35);
    var_350 = wp::sub(var_348, var_349);
    var_351 = wp::mul(var_1, var_310);
    var_352 = wp::sub(var_350, var_351);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_353 = wp::add(var_317, var_338);
    var_354 = wp::add(var_353, var_345);
    var_355 = wp::neg(var_338);
    var_356 = wp::add(var_324, var_338);
    var_357 = wp::add(var_356, var_352);
    var_358 = wp::neg(var_352);
    var_359 = wp::add(var_331, var_345);
    var_360 = wp::add(var_359, var_352);
    var_361 = wp::neg(var_345);
    var_362 = wp::vec_t<6, wp::float32>({var_354, var_355, var_357, var_358, var_360, var_361});
    // for i in range(j + 1):                                                                 <L 832>
    var_364 = wp::add(var_298, var_363);
    var_365 = wp::range(var_364);
    start_for_6:;
        if (iter_cmp(var_365) == 0) goto end_for_6;
        var_366 = wp::iter_next(var_365);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_368 = wp::mul(var_367, var_gamma);
        var_369 = wp::extract(var_362, var_366);
        var_370 = wp::mul(var_368, var_369);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_366, var_298, var_370);
        // if i != j:                                                                         <L 835>
        var_371 = (var_366 != var_298);
        if (var_371) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_298, var_366, var_370);
        }
        wp::assign(var_148, var_370);
        goto start_for_6;
    end_for_6:;
    // h0 = basis[j, 0]                                                                       <L 819>
    var_374 = wp::extract(var_75, var_372, var_373);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_376 = wp::extract(var_75, var_372, var_375);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_378 = wp::extract(var_75, var_372, var_377);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_380 = wp::extract(var_75, var_372, var_379);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_382 = wp::extract(var_75, var_372, var_381);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_384 = wp::extract(var_75, var_372, var_383);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_385 = wp::mul(var_376, var_5);
    var_386 = wp::mul(var_3, var_378);
    var_387 = wp::add(var_385, var_386);
    var_389 = wp::mul(var_388, var_35);
    var_390 = wp::mul(var_389, var_384);
    var_391 = wp::sub(var_387, var_390);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_392 = wp::mul(var_374, var_5);
    var_393 = wp::mul(var_1, var_378);
    var_394 = wp::add(var_392, var_393);
    var_396 = wp::mul(var_395, var_25);
    var_397 = wp::mul(var_396, var_382);
    var_398 = wp::sub(var_394, var_397);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_399 = wp::mul(var_374, var_3);
    var_400 = wp::mul(var_1, var_376);
    var_401 = wp::add(var_399, var_400);
    var_403 = wp::mul(var_402, var_15);
    var_404 = wp::mul(var_403, var_380);
    var_405 = wp::sub(var_401, var_404);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_406 = wp::mul(var_382, var_35);
    var_407 = wp::mul(var_25, var_384);
    var_408 = wp::add(var_406, var_407);
    var_409 = wp::mul(var_378, var_15);
    var_410 = wp::sub(var_408, var_409);
    var_411 = wp::mul(var_5, var_380);
    var_412 = wp::sub(var_410, var_411);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_413 = wp::mul(var_380, var_35);
    var_414 = wp::mul(var_15, var_384);
    var_415 = wp::add(var_413, var_414);
    var_416 = wp::mul(var_376, var_25);
    var_417 = wp::sub(var_415, var_416);
    var_418 = wp::mul(var_3, var_382);
    var_419 = wp::sub(var_417, var_418);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_420 = wp::mul(var_380, var_25);
    var_421 = wp::mul(var_15, var_382);
    var_422 = wp::add(var_420, var_421);
    var_423 = wp::mul(var_374, var_35);
    var_424 = wp::sub(var_422, var_423);
    var_425 = wp::mul(var_1, var_384);
    var_426 = wp::sub(var_424, var_425);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_427 = wp::add(var_391, var_412);
    var_428 = wp::add(var_427, var_419);
    var_429 = wp::neg(var_412);
    var_430 = wp::add(var_398, var_412);
    var_431 = wp::add(var_430, var_426);
    var_432 = wp::neg(var_426);
    var_433 = wp::add(var_405, var_419);
    var_434 = wp::add(var_433, var_426);
    var_435 = wp::neg(var_419);
    var_436 = wp::vec_t<6, wp::float32>({var_428, var_429, var_431, var_432, var_434, var_435});
    // for i in range(j + 1):                                                                 <L 832>
    var_438 = wp::add(var_372, var_437);
    var_439 = wp::range(var_438);
    start_for_8:;
        if (iter_cmp(var_439) == 0) goto end_for_8;
        var_440 = wp::iter_next(var_439);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_442 = wp::mul(var_441, var_gamma);
        var_443 = wp::extract(var_436, var_440);
        var_444 = wp::mul(var_442, var_443);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_440, var_372, var_444);
        // if i != j:                                                                         <L 835>
        var_445 = (var_440 != var_372);
        if (var_445) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_372, var_440, var_444);
        }
        wp::assign(var_148, var_444);
        goto start_for_8;
    end_for_8:;
    // h0 = basis[j, 0]                                                                       <L 819>
    var_448 = wp::extract(var_75, var_446, var_447);
    // h1 = basis[j, 1]                                                                       <L 820>
    var_450 = wp::extract(var_75, var_446, var_449);
    // h2 = basis[j, 2]                                                                       <L 821>
    var_452 = wp::extract(var_75, var_446, var_451);
    // h3 = basis[j, 3]                                                                       <L 822>
    var_454 = wp::extract(var_75, var_446, var_453);
    // h4 = basis[j, 4]                                                                       <L 823>
    var_456 = wp::extract(var_75, var_446, var_455);
    // h5 = basis[j, 5]                                                                       <L 824>
    var_458 = wp::extract(var_75, var_446, var_457);
    // dc0 = h1 * c + b * h2 - 2.0 * f * h5                                                   <L 825>
    var_459 = wp::mul(var_450, var_5);
    var_460 = wp::mul(var_3, var_452);
    var_461 = wp::add(var_459, var_460);
    var_463 = wp::mul(var_462, var_35);
    var_464 = wp::mul(var_463, var_458);
    var_465 = wp::sub(var_461, var_464);
    // dc1 = h0 * c + a * h2 - 2.0 * e * h4                                                   <L 826>
    var_466 = wp::mul(var_448, var_5);
    var_467 = wp::mul(var_1, var_452);
    var_468 = wp::add(var_466, var_467);
    var_470 = wp::mul(var_469, var_25);
    var_471 = wp::mul(var_470, var_456);
    var_472 = wp::sub(var_468, var_471);
    // dc2 = h0 * b + a * h1 - 2.0 * d * h3                                                   <L 827>
    var_473 = wp::mul(var_448, var_3);
    var_474 = wp::mul(var_1, var_450);
    var_475 = wp::add(var_473, var_474);
    var_477 = wp::mul(var_476, var_15);
    var_478 = wp::mul(var_477, var_454);
    var_479 = wp::sub(var_475, var_478);
    // dc3 = h4 * f + e * h5 - h2 * d - c * h3                                                <L 828>
    var_480 = wp::mul(var_456, var_35);
    var_481 = wp::mul(var_25, var_458);
    var_482 = wp::add(var_480, var_481);
    var_483 = wp::mul(var_452, var_15);
    var_484 = wp::sub(var_482, var_483);
    var_485 = wp::mul(var_5, var_454);
    var_486 = wp::sub(var_484, var_485);
    // dc4 = h3 * f + d * h5 - h1 * e - b * h4                                                <L 829>
    var_487 = wp::mul(var_454, var_35);
    var_488 = wp::mul(var_15, var_458);
    var_489 = wp::add(var_487, var_488);
    var_490 = wp::mul(var_450, var_25);
    var_491 = wp::sub(var_489, var_490);
    var_492 = wp::mul(var_3, var_456);
    var_493 = wp::sub(var_491, var_492);
    // dc5 = h3 * e + d * h4 - h0 * f - a * h5                                                <L 830>
    var_494 = wp::mul(var_454, var_25);
    var_495 = wp::mul(var_15, var_456);
    var_496 = wp::add(var_494, var_495);
    var_497 = wp::mul(var_448, var_35);
    var_498 = wp::sub(var_496, var_497);
    var_499 = wp::mul(var_1, var_458);
    var_500 = wp::sub(var_498, var_499);
    // dg = vec6(dc0 + dc3 + dc4, -dc3, dc1 + dc3 + dc5, -dc5, dc2 + dc4 + dc5, -dc4)         <L 831>
    var_501 = wp::add(var_465, var_486);
    var_502 = wp::add(var_501, var_493);
    var_503 = wp::neg(var_486);
    var_504 = wp::add(var_472, var_486);
    var_505 = wp::add(var_504, var_500);
    var_506 = wp::neg(var_500);
    var_507 = wp::add(var_479, var_493);
    var_508 = wp::add(var_507, var_500);
    var_509 = wp::neg(var_493);
    var_510 = wp::vec_t<6, wp::float32>({var_502, var_503, var_505, var_506, var_508, var_509});
    // for i in range(j + 1):                                                                 <L 832>
    var_512 = wp::add(var_446, var_511);
    var_513 = wp::range(var_512);
    start_for_10:;
        if (iter_cmp(var_513) == 0) goto end_for_10;
        var_514 = wp::iter_next(var_513);
        // value = 2.0 * gamma * dg[i]                                                        <L 833>
        var_516 = wp::mul(var_515, var_gamma);
        var_517 = wp::extract(var_510, var_514);
        var_518 = wp::mul(var_516, var_517);
        // metric[i, j] += value                                                              <L 834>
        wp::add_inplace(var_metric, var_514, var_446, var_518);
        // if i != j:                                                                         <L 835>
        var_519 = (var_514 != var_446);
        if (var_519) {
            // metric[j, i] += value                                                          <L 836>
            wp::add_inplace(var_metric, var_446, var_514, var_518);
        }
        wp::assign(var_148, var_518);
        goto start_for_10;
    end_for_10:;
    // return metric                                                                          <L 837>
    return var_metric;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:840
static CUDA_CALLABLE bool _snh_positive3_0(
    wp::mat_t<3, 3, wp::float32> var_A)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 1;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    const wp::int32 var_24 = 2;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    const wp::int32 var_29 = 0;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::int32 var_33 = 1;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    const wp::int32 var_37 = 2;
    wp::float32 var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    const wp::float32 var_41 = 1e-05;
    wp::float32 var_42;
    const wp::int32 var_43 = 0;
    const wp::int32 var_44 = 0;
    wp::float32 var_45;
    bool var_46;
    const bool var_47 = false;
    const wp::int32 var_48 = 1;
    const wp::int32 var_49 = 1;
    wp::float32 var_50;
    const wp::int32 var_51 = 0;
    const wp::int32 var_52 = 1;
    wp::float32 var_53;
    const wp::int32 var_54 = 0;
    const wp::int32 var_55 = 1;
    wp::float32 var_56;
    const wp::int32 var_57 = 0;
    const wp::int32 var_58 = 0;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    bool var_63;
    const bool var_64 = false;
    const wp::int32 var_65 = 1;
    const wp::int32 var_66 = 2;
    wp::float32 var_67;
    const wp::int32 var_68 = 0;
    const wp::int32 var_69 = 1;
    wp::float32 var_70;
    const wp::int32 var_71 = 0;
    const wp::int32 var_72 = 2;
    wp::float32 var_73;
    const wp::int32 var_74 = 0;
    const wp::int32 var_75 = 0;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    const wp::int32 var_80 = 2;
    const wp::int32 var_81 = 2;
    wp::float32 var_82;
    const wp::int32 var_83 = 0;
    const wp::int32 var_84 = 2;
    wp::float32 var_85;
    const wp::int32 var_86 = 0;
    const wp::int32 var_87 = 2;
    wp::float32 var_88;
    const wp::int32 var_89 = 0;
    const wp::int32 var_90 = 0;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    wp::float32 var_95;
    wp::float32 var_96;
    wp::float32 var_97;
    bool var_98;
    //---------
    // forward
    // def _snh_positive3(A: wp.mat33) -> bool:                                               <L 841>
    // scale = float(0.0)                                                                     <L 842>
    var_1 = wp::float(var_0);
    // for row in range(3):                                                                   <L 843>
    // for col in range(3):                                                                   <L 844>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 845>
    var_4 = wp::extract(var_A, var_2, var_3);
    var_5 = wp::abs(var_4);
    var_6 = wp::max(var_1, var_5);
    var_8 = wp::extract(var_A, var_2, var_7);
    var_9 = wp::abs(var_8);
    var_10 = wp::max(var_6, var_9);
    var_12 = wp::extract(var_A, var_2, var_11);
    var_13 = wp::abs(var_12);
    var_14 = wp::max(var_10, var_13);
    // for col in range(3):                                                                   <L 844>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 845>
    var_17 = wp::extract(var_A, var_15, var_16);
    var_18 = wp::abs(var_17);
    var_19 = wp::max(var_14, var_18);
    var_21 = wp::extract(var_A, var_15, var_20);
    var_22 = wp::abs(var_21);
    var_23 = wp::max(var_19, var_22);
    var_25 = wp::extract(var_A, var_15, var_24);
    var_26 = wp::abs(var_25);
    var_27 = wp::max(var_23, var_26);
    // for col in range(3):                                                                   <L 844>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 845>
    var_30 = wp::extract(var_A, var_28, var_29);
    var_31 = wp::abs(var_30);
    var_32 = wp::max(var_27, var_31);
    var_34 = wp::extract(var_A, var_28, var_33);
    var_35 = wp::abs(var_34);
    var_36 = wp::max(var_32, var_35);
    var_38 = wp::extract(var_A, var_28, var_37);
    var_39 = wp::abs(var_38);
    var_40 = wp::max(var_36, var_39);
    // tol = 1.0e-5 * scale                                                                   <L 846>
    var_42 = wp::mul(var_41, var_40);
    // if A[0, 0] <= tol:                                                                     <L 847>
    var_45 = wp::extract(var_A, var_43, var_44);
    var_46 = (var_45 <= var_42);
    if (var_46) {
        // return False                                                                       <L 848>
        return var_47;
    }
    // pivot = A[1, 1] - A[0, 1] * (A[0, 1] / A[0, 0])                                        <L 849>
    var_50 = wp::extract(var_A, var_48, var_49);
    var_53 = wp::extract(var_A, var_51, var_52);
    var_56 = wp::extract(var_A, var_54, var_55);
    var_59 = wp::extract(var_A, var_57, var_58);
    var_60 = wp::div(var_56, var_59);
    var_61 = wp::mul(var_53, var_60);
    var_62 = wp::sub(var_50, var_61);
    // if pivot <= tol:                                                                       <L 850>
    var_63 = (var_62 <= var_42);
    if (var_63) {
        // return False                                                                       <L 851>
        return var_64;
    }
    // cross = A[1, 2] - A[0, 1] * (A[0, 2] / A[0, 0])                                        <L 852>
    var_67 = wp::extract(var_A, var_65, var_66);
    var_70 = wp::extract(var_A, var_68, var_69);
    var_73 = wp::extract(var_A, var_71, var_72);
    var_76 = wp::extract(var_A, var_74, var_75);
    var_77 = wp::div(var_73, var_76);
    var_78 = wp::mul(var_70, var_77);
    var_79 = wp::sub(var_67, var_78);
    // return A[2, 2] - A[0, 2] * (A[0, 2] / A[0, 0]) - cross * (cross / pivot) > tol         <L 853>
    var_82 = wp::extract(var_A, var_80, var_81);
    var_85 = wp::extract(var_A, var_83, var_84);
    var_88 = wp::extract(var_A, var_86, var_87);
    var_91 = wp::extract(var_A, var_89, var_90);
    var_92 = wp::div(var_88, var_91);
    var_93 = wp::mul(var_85, var_92);
    var_94 = wp::sub(var_82, var_93);
    var_95 = wp::div(var_79, var_62);
    var_96 = wp::mul(var_79, var_95);
    var_97 = wp::sub(var_94, var_96);
    var_98 = (var_97 > var_42);
    return var_98;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:856
static CUDA_CALLABLE void _snh_eigen3_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::float32 var_1;
    const wp::int32 var_2 = 0;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 1;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::float32 var_23;
    const wp::int32 var_24 = 2;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    const wp::int32 var_29 = 0;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::int32 var_33 = 1;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    const wp::int32 var_37 = 2;
    wp::float32 var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    const wp::int32 var_41 = 3;
    wp::mat_t<3, 3, wp::float32> var_42;
    const wp::float32 var_43 = 0.0;
    bool var_44;
    const wp::float32 var_45 = 0.0;
    const wp::float32 var_46 = 0.0;
    const wp::float32 var_47 = 0.0;
    wp::vec_t<3, wp::float32> var_48;
    const wp::float32 var_49 = 1.0;
    wp::float32 var_50;
    wp::mat_t<3, 3, wp::float32> var_51;
    bool var_52;
    const wp::int32 var_53 = 0;
    const wp::int32 var_54 = 1;
    wp::float32 var_55;
    const wp::float32 var_56 = 0.0;
    bool var_57;
    const wp::int32 var_58 = 0;
    const wp::int32 var_59 = 2;
    wp::float32 var_60;
    const wp::float32 var_61 = 0.0;
    bool var_62;
    const wp::int32 var_63 = 1;
    const wp::int32 var_64 = 2;
    wp::float32 var_65;
    const wp::float32 var_66 = 0.0;
    bool var_67;
    const wp::int32 var_68 = 0;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    const wp::int32 var_71 = 1;
    const wp::int32 var_72 = 1;
    wp::float32 var_73;
    const wp::int32 var_74 = 2;
    const wp::int32 var_75 = 2;
    wp::float32 var_76;
    wp::vec_t<3, wp::float32> var_77;
    const wp::int32 var_78 = 0;
    const wp::int32 var_79 = 0;
    wp::float32 var_80;
    const wp::int32 var_81 = 1;
    const wp::int32 var_82 = 1;
    wp::float32 var_83;
    wp::float32 var_84;
    const wp::int32 var_85 = 2;
    const wp::int32 var_86 = 2;
    wp::float32 var_87;
    wp::float32 var_88;
    const wp::float32 var_89 = 1.0;
    const wp::float32 var_90 = 3.0;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::mat_t<3, 3, wp::float32> var_93;
    const wp::int32 var_94 = 0;
    const wp::int32 var_95 = 0;
    const wp::int32 var_96 = 1;
    const wp::int32 var_97 = 1;
    const wp::int32 var_98 = 2;
    const wp::int32 var_99 = 2;
    const wp::float32 var_100 = 0.0;
    wp::float32 var_101;
    const wp::int32 var_102 = 0;
    const wp::int32 var_103 = 0;
    wp::float32 var_104;
    wp::float32 var_105;
    wp::float32 var_106;
    const wp::int32 var_107 = 1;
    wp::float32 var_108;
    wp::float32 var_109;
    wp::float32 var_110;
    const wp::int32 var_111 = 2;
    wp::float32 var_112;
    wp::float32 var_113;
    wp::float32 var_114;
    const wp::int32 var_115 = 1;
    const wp::int32 var_116 = 0;
    wp::float32 var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    const wp::int32 var_120 = 1;
    wp::float32 var_121;
    wp::float32 var_122;
    wp::float32 var_123;
    const wp::int32 var_124 = 2;
    wp::float32 var_125;
    wp::float32 var_126;
    wp::float32 var_127;
    const wp::int32 var_128 = 2;
    const wp::int32 var_129 = 0;
    wp::float32 var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    const wp::int32 var_133 = 1;
    wp::float32 var_134;
    wp::float32 var_135;
    wp::float32 var_136;
    const wp::int32 var_137 = 2;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::float32 var_140;
    const wp::float32 var_141 = 1.0;
    wp::float32 var_142;
    wp::mat_t<3, 3, wp::float32> var_143;
    const wp::int32 var_144 = 0;
    const wp::int32 var_145 = 0;
    wp::float32 var_146;
    const wp::int32 var_147 = 0;
    const wp::int32 var_148 = 0;
    wp::float32 var_149;
    wp::float32 var_150;
    const wp::int32 var_151 = 1;
    const wp::int32 var_152 = 1;
    wp::float32 var_153;
    const wp::int32 var_154 = 1;
    const wp::int32 var_155 = 1;
    wp::float32 var_156;
    wp::float32 var_157;
    wp::float32 var_158;
    const wp::int32 var_159 = 2;
    const wp::int32 var_160 = 2;
    wp::float32 var_161;
    const wp::int32 var_162 = 2;
    const wp::int32 var_163 = 2;
    wp::float32 var_164;
    wp::float32 var_165;
    wp::float32 var_166;
    const wp::float32 var_167 = 2.0;
    const wp::int32 var_168 = 0;
    const wp::int32 var_169 = 1;
    wp::float32 var_170;
    const wp::int32 var_171 = 0;
    const wp::int32 var_172 = 1;
    wp::float32 var_173;
    wp::float32 var_174;
    const wp::int32 var_175 = 0;
    const wp::int32 var_176 = 2;
    wp::float32 var_177;
    const wp::int32 var_178 = 0;
    const wp::int32 var_179 = 2;
    wp::float32 var_180;
    wp::float32 var_181;
    wp::float32 var_182;
    const wp::int32 var_183 = 1;
    const wp::int32 var_184 = 2;
    wp::float32 var_185;
    const wp::int32 var_186 = 1;
    const wp::int32 var_187 = 2;
    wp::float32 var_188;
    wp::float32 var_189;
    wp::float32 var_190;
    wp::float32 var_191;
    wp::float32 var_192;
    const wp::float32 var_193 = 1.0;
    const wp::float32 var_194 = 6.0;
    wp::float32 var_195;
    wp::float32 var_196;
    wp::float32 var_197;
    const wp::float32 var_198 = 1.0;
    wp::float32 var_199;
    wp::mat_t<3, 3, wp::float32> var_200;
    const wp::int32 var_201 = 0;
    const wp::int32 var_202 = 0;
    wp::float32 var_203;
    const wp::int32 var_204 = 1;
    const wp::int32 var_205 = 1;
    wp::float32 var_206;
    const wp::int32 var_207 = 2;
    const wp::int32 var_208 = 2;
    wp::float32 var_209;
    wp::float32 var_210;
    const wp::int32 var_211 = 1;
    const wp::int32 var_212 = 2;
    wp::float32 var_213;
    const wp::int32 var_214 = 1;
    const wp::int32 var_215 = 2;
    wp::float32 var_216;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    const wp::int32 var_220 = 0;
    const wp::int32 var_221 = 1;
    wp::float32 var_222;
    const wp::int32 var_223 = 0;
    const wp::int32 var_224 = 1;
    wp::float32 var_225;
    const wp::int32 var_226 = 2;
    const wp::int32 var_227 = 2;
    wp::float32 var_228;
    wp::float32 var_229;
    const wp::int32 var_230 = 0;
    const wp::int32 var_231 = 2;
    wp::float32 var_232;
    const wp::int32 var_233 = 1;
    const wp::int32 var_234 = 2;
    wp::float32 var_235;
    wp::float32 var_236;
    wp::float32 var_237;
    wp::float32 var_238;
    wp::float32 var_239;
    const wp::int32 var_240 = 0;
    const wp::int32 var_241 = 2;
    wp::float32 var_242;
    const wp::int32 var_243 = 0;
    const wp::int32 var_244 = 1;
    wp::float32 var_245;
    const wp::int32 var_246 = 1;
    const wp::int32 var_247 = 2;
    wp::float32 var_248;
    wp::float32 var_249;
    const wp::int32 var_250 = 0;
    const wp::int32 var_251 = 2;
    wp::float32 var_252;
    const wp::int32 var_253 = 1;
    const wp::int32 var_254 = 1;
    wp::float32 var_255;
    wp::float32 var_256;
    wp::float32 var_257;
    wp::float32 var_258;
    wp::float32 var_259;
    const wp::float32 var_260 = 0.5;
    wp::float32 var_261;
    const wp::float32 var_262 = 2.0;
    const wp::float32 var_263 = 1.0;
    wp::float32 var_264;
    wp::float32 var_265;
    wp::float32 var_266;
    const wp::float32 var_267 = 1.0;
    const wp::float32 var_268 = 3.0;
    wp::float32 var_269;
    wp::float32 var_270;
    wp::float32 var_271;
    wp::float32 var_272;
    const wp::float32 var_273 = 0.0;
    bool var_274;
    wp::float32 var_275;
    wp::float32 var_276;
    const wp::int32 var_277 = 0;
    const wp::int32 var_278 = 0;
    const wp::int32 var_279 = 1;
    const wp::int32 var_280 = 1;
    const wp::int32 var_281 = 2;
    const wp::int32 var_282 = 2;
    const wp::int32 var_283 = 0;
    wp::vec_t<3, wp::float32> var_284;
    const wp::int32 var_285 = 1;
    wp::vec_t<3, wp::float32> var_286;
    const wp::int32 var_287 = 2;
    wp::vec_t<3, wp::float32> var_288;
    wp::vec_t<3, wp::float32> var_289;
    wp::vec_t<3, wp::float32> var_290;
    wp::vec_t<3, wp::float32> var_291;
    wp::float32 var_292;
    wp::float32 var_293;
    wp::float32 var_294;
    wp::vec_t<3, wp::float32> var_295;
    wp::float32 var_296;
    bool var_297;
    wp::vec_t<3, wp::float32> var_298;
    wp::float32 var_299;
    wp::vec_t<3, wp::float32> var_300;
    wp::float32 var_301;
    bool var_302;
    wp::vec_t<3, wp::float32> var_303;
    wp::float32 var_304;
    wp::vec_t<3, wp::float32> var_305;
    wp::float32 var_306;
    const wp::float32 var_307 = 1.0;
    wp::float32 var_308;
    wp::float32 var_309;
    wp::vec_t<3, wp::float32> var_310;
    const wp::int32 var_311 = 0;
    wp::float32 var_312;
    wp::float32 var_313;
    const wp::int32 var_314 = 1;
    wp::float32 var_315;
    wp::float32 var_316;
    bool var_317;
    const wp::float32 var_318 = 1.0;
    const wp::int32 var_319 = 0;
    wp::float32 var_320;
    const wp::int32 var_321 = 0;
    wp::float32 var_322;
    wp::float32 var_323;
    const wp::int32 var_324 = 2;
    wp::float32 var_325;
    const wp::int32 var_326 = 2;
    wp::float32 var_327;
    wp::float32 var_328;
    wp::float32 var_329;
    wp::float32 var_330;
    wp::float32 var_331;
    const wp::int32 var_332 = 2;
    wp::float32 var_333;
    wp::float32 var_334;
    wp::float32 var_335;
    const wp::float32 var_336 = 0.0;
    const wp::int32 var_337 = 0;
    wp::float32 var_338;
    wp::float32 var_339;
    wp::vec_t<3, wp::float32> var_340;
    const wp::float32 var_341 = 1.0;
    const wp::int32 var_342 = 1;
    wp::float32 var_343;
    const wp::int32 var_344 = 1;
    wp::float32 var_345;
    wp::float32 var_346;
    const wp::int32 var_347 = 2;
    wp::float32 var_348;
    const wp::int32 var_349 = 2;
    wp::float32 var_350;
    wp::float32 var_351;
    wp::float32 var_352;
    wp::float32 var_353;
    wp::float32 var_354;
    const wp::float32 var_355 = 0.0;
    const wp::int32 var_356 = 2;
    wp::float32 var_357;
    wp::float32 var_358;
    const wp::int32 var_359 = 1;
    wp::float32 var_360;
    wp::float32 var_361;
    wp::float32 var_362;
    wp::vec_t<3, wp::float32> var_363;
    wp::float32 var_364;
    wp::vec_t<3, wp::float32> var_365;
    wp::vec_t<3, wp::float32> var_366;
    wp::vec_t<3, wp::float32> var_367;
    wp::vec_t<3, wp::float32> var_368;
    wp::vec_t<3, wp::float32> var_369;
    wp::float32 var_370;
    wp::float32 var_371;
    wp::float32 var_372;
    const wp::float32 var_373 = 1.0;
    const wp::float32 var_374 = 0.0;
    const wp::float32 var_375 = 0.0;
    const wp::float32 var_376 = 0.0;
    bool var_377;
    const wp::float32 var_378 = 0.5;
    wp::float32 var_379;
    wp::float32 var_380;
    wp::float32 var_381;
    wp::float32 var_382;
    wp::float32 var_383;
    wp::float32 var_384;
    wp::float32 var_385;
    const wp::float32 var_386 = 0.0;
    bool var_387;
    wp::float32 var_388;
    wp::float32 var_389;
    wp::float32 var_390;
    wp::float32 var_391;
    wp::float32 var_392;
    wp::float32 var_393;
    wp::float32 var_394;
    wp::float32 var_395;
    wp::float32 var_396;
    const wp::float32 var_397 = 1.0;
    const wp::float32 var_398 = 1.0;
    wp::float32 var_399;
    wp::float32 var_400;
    wp::float32 var_401;
    wp::float32 var_402;
    wp::float32 var_403;
    wp::float32 var_404;
    wp::float32 var_405;
    wp::float32 var_406;
    wp::float32 var_407;
    wp::float32 var_408;
    wp::float32 var_409;
    wp::float32 var_410;
    wp::float32 var_411;
    wp::float32 var_412;
    wp::float32 var_413;
    wp::float32 var_414;
    wp::vec_t<3, wp::float32> var_415;
    wp::vec_t<3, wp::float32> var_416;
    wp::vec_t<3, wp::float32> var_417;
    wp::vec_t<3, wp::float32> var_418;
    wp::vec_t<3, wp::float32> var_419;
    wp::vec_t<3, wp::float32> var_420;
    wp::vec_t<3, wp::float32> var_421;
    const wp::int32 var_422 = 0;
    wp::float32 var_423;
    const wp::int32 var_424 = 0;
    wp::float32 var_425;
    const wp::int32 var_426 = 0;
    wp::float32 var_427;
    const wp::int32 var_428 = 1;
    wp::float32 var_429;
    const wp::int32 var_430 = 1;
    wp::float32 var_431;
    const wp::int32 var_432 = 1;
    wp::float32 var_433;
    const wp::int32 var_434 = 2;
    wp::float32 var_435;
    const wp::int32 var_436 = 2;
    wp::float32 var_437;
    const wp::int32 var_438 = 2;
    wp::float32 var_439;
    wp::mat_t<3, 3, wp::float32> var_440;
    //---------
    // forward
    // def _snh_eigen3(A: wp.mat33) -> tuple[wp.vec3, wp.mat33]:                              <L 857>
    // scale = float(0.0)                                                                     <L 858>
    var_1 = wp::float(var_0);
    // for row in range(3):                                                                   <L 859>
    // for col in range(3):                                                                   <L 860>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 861>
    var_4 = wp::extract(var_A, var_2, var_3);
    var_5 = wp::abs(var_4);
    var_6 = wp::max(var_1, var_5);
    var_8 = wp::extract(var_A, var_2, var_7);
    var_9 = wp::abs(var_8);
    var_10 = wp::max(var_6, var_9);
    var_12 = wp::extract(var_A, var_2, var_11);
    var_13 = wp::abs(var_12);
    var_14 = wp::max(var_10, var_13);
    // for col in range(3):                                                                   <L 860>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 861>
    var_17 = wp::extract(var_A, var_15, var_16);
    var_18 = wp::abs(var_17);
    var_19 = wp::max(var_14, var_18);
    var_21 = wp::extract(var_A, var_15, var_20);
    var_22 = wp::abs(var_21);
    var_23 = wp::max(var_19, var_22);
    var_25 = wp::extract(var_A, var_15, var_24);
    var_26 = wp::abs(var_25);
    var_27 = wp::max(var_23, var_26);
    // for col in range(3):                                                                   <L 860>
    // scale = wp.max(scale, wp.abs(A[row, col]))                                             <L 861>
    var_30 = wp::extract(var_A, var_28, var_29);
    var_31 = wp::abs(var_30);
    var_32 = wp::max(var_27, var_31);
    var_34 = wp::extract(var_A, var_28, var_33);
    var_35 = wp::abs(var_34);
    var_36 = wp::max(var_32, var_35);
    var_38 = wp::extract(var_A, var_28, var_37);
    var_39 = wp::abs(var_38);
    var_40 = wp::max(var_36, var_39);
    // Q = wp.identity(3, dtype=float)                                                        <L 862>
    var_42 = wp::identity<3, wp::float32>();
    // if scale == 0.0:                                                                       <L 863>
    var_44 = (var_40 == var_43);
    if (var_44) {
        // return wp.vec3(0.0, 0.0, 0.0), Q                                                   <L 864>
        var_48 = wp::vec_t<3, wp::float32>(var_45, var_46, var_47);
        ret_0 = var_48;
        ret_1 = var_42;
        return;
    }
    // D = A * (1.0 / scale)                                                                  <L 865>
    var_50 = wp::div(var_49, var_40);
    var_51 = wp::mul(var_A, var_50);
    // if D[0, 1] == 0.0 and D[0, 2] == 0.0 and D[1, 2] == 0.0:                               <L 866>
    var_55 = wp::extract(var_51, var_53, var_54);
    var_57 = (var_55 == var_56);
    var_52 = var_57;
    if (var_52) {
        var_60 = wp::extract(var_51, var_58, var_59);
        var_62 = (var_60 == var_61);
        var_52 = var_52 && var_62;
    }
    if (var_52) {
        var_65 = wp::extract(var_51, var_63, var_64);
        var_67 = (var_65 == var_66);
        var_52 = var_52 && var_67;
    }
    if (var_52) {
        // return wp.vec3(A[0, 0], A[1, 1], A[2, 2]), Q                                       <L 867>
        var_70 = wp::extract(var_A, var_68, var_69);
        var_73 = wp::extract(var_A, var_71, var_72);
        var_76 = wp::extract(var_A, var_74, var_75);
        var_77 = wp::vec_t<3, wp::float32>(var_70, var_73, var_76);
        ret_0 = var_77;
        ret_1 = var_42;
        return;
    }
    // mean = (D[0, 0] + D[1, 1] + D[2, 2]) * (1.0 / 3.0)                                     <L 869>
    var_80 = wp::extract(var_51, var_78, var_79);
    var_83 = wp::extract(var_51, var_81, var_82);
    var_84 = wp::add(var_80, var_83);
    var_87 = wp::extract(var_51, var_85, var_86);
    var_88 = wp::add(var_84, var_87);
    var_91 = wp::div(var_89, var_90);
    var_92 = wp::mul(var_88, var_91);
    // B = D                                                                                  <L 870>
    var_93 = wp::copy(var_51);
    // B[0, 0] -= mean                                                                        <L 871>
    wp::sub_inplace(var_93, var_94, var_95, var_92);
    // B[1, 1] -= mean                                                                        <L 872>
    wp::sub_inplace(var_93, var_96, var_97, var_92);
    // B[2, 2] -= mean                                                                        <L 873>
    wp::sub_inplace(var_93, var_98, var_99, var_92);
    // bscale = float(0.0)                                                                    <L 874>
    var_101 = wp::float(var_100);
    // for row in range(3):                                                                   <L 875>
    // for col in range(3):                                                                   <L 876>
    // bscale = wp.max(bscale, wp.abs(B[row, col]))                                           <L 877>
    var_104 = wp::extract(var_93, var_102, var_103);
    var_105 = wp::abs(var_104);
    var_106 = wp::max(var_101, var_105);
    var_108 = wp::extract(var_93, var_102, var_107);
    var_109 = wp::abs(var_108);
    var_110 = wp::max(var_106, var_109);
    var_112 = wp::extract(var_93, var_102, var_111);
    var_113 = wp::abs(var_112);
    var_114 = wp::max(var_110, var_113);
    // for col in range(3):                                                                   <L 876>
    // bscale = wp.max(bscale, wp.abs(B[row, col]))                                           <L 877>
    var_117 = wp::extract(var_93, var_115, var_116);
    var_118 = wp::abs(var_117);
    var_119 = wp::max(var_114, var_118);
    var_121 = wp::extract(var_93, var_115, var_120);
    var_122 = wp::abs(var_121);
    var_123 = wp::max(var_119, var_122);
    var_125 = wp::extract(var_93, var_115, var_124);
    var_126 = wp::abs(var_125);
    var_127 = wp::max(var_123, var_126);
    // for col in range(3):                                                                   <L 876>
    // bscale = wp.max(bscale, wp.abs(B[row, col]))                                           <L 877>
    var_130 = wp::extract(var_93, var_128, var_129);
    var_131 = wp::abs(var_130);
    var_132 = wp::max(var_127, var_131);
    var_134 = wp::extract(var_93, var_128, var_133);
    var_135 = wp::abs(var_134);
    var_136 = wp::max(var_132, var_135);
    var_138 = wp::extract(var_93, var_128, var_137);
    var_139 = wp::abs(var_138);
    var_140 = wp::max(var_136, var_139);
    // B = B * (1.0 / bscale)                                                                 <L 878>
    var_142 = wp::div(var_141, var_140);
    var_143 = wp::mul(var_93, var_142);
    // p = wp.sqrt(                                                                           <L 879>
    // (                                                                                      <L 880>
    // B[0, 0] * B[0, 0]                                                                      <L 881>
    var_146 = wp::extract(var_143, var_144, var_145);
    var_149 = wp::extract(var_143, var_147, var_148);
    var_150 = wp::mul(var_146, var_149);
    // + B[1, 1] * B[1, 1]                                                                    <L 882>
    var_153 = wp::extract(var_143, var_151, var_152);
    var_156 = wp::extract(var_143, var_154, var_155);
    var_157 = wp::mul(var_153, var_156);
    var_158 = wp::add(var_150, var_157);
    // + B[2, 2] * B[2, 2]                                                                    <L 883>
    var_161 = wp::extract(var_143, var_159, var_160);
    var_164 = wp::extract(var_143, var_162, var_163);
    var_165 = wp::mul(var_161, var_164);
    var_166 = wp::add(var_158, var_165);
    // + 2.0 * (B[0, 1] * B[0, 1] + B[0, 2] * B[0, 2] + B[1, 2] * B[1, 2])                    <L 884>
    var_170 = wp::extract(var_143, var_168, var_169);
    var_173 = wp::extract(var_143, var_171, var_172);
    var_174 = wp::mul(var_170, var_173);
    var_177 = wp::extract(var_143, var_175, var_176);
    var_180 = wp::extract(var_143, var_178, var_179);
    var_181 = wp::mul(var_177, var_180);
    var_182 = wp::add(var_174, var_181);
    var_185 = wp::extract(var_143, var_183, var_184);
    var_188 = wp::extract(var_143, var_186, var_187);
    var_189 = wp::mul(var_185, var_188);
    var_190 = wp::add(var_182, var_189);
    var_191 = wp::mul(var_167, var_190);
    var_192 = wp::add(var_166, var_191);
    // * (1.0 / 6.0)                                                                          <L 886>
    var_195 = wp::div(var_193, var_194);
    var_196 = wp::mul(var_192, var_195);
    var_197 = wp::sqrt(var_196);
    // B = B * (1.0 / p)                                                                      <L 888>
    var_199 = wp::div(var_198, var_197);
    var_200 = wp::mul(var_143, var_199);
    // determinant = (                                                                        <L 889>
    // B[0, 0] * (B[1, 1] * B[2, 2] - B[1, 2] * B[1, 2])                                      <L 890>
    var_203 = wp::extract(var_200, var_201, var_202);
    var_206 = wp::extract(var_200, var_204, var_205);
    var_209 = wp::extract(var_200, var_207, var_208);
    var_210 = wp::mul(var_206, var_209);
    var_213 = wp::extract(var_200, var_211, var_212);
    var_216 = wp::extract(var_200, var_214, var_215);
    var_217 = wp::mul(var_213, var_216);
    var_218 = wp::sub(var_210, var_217);
    var_219 = wp::mul(var_203, var_218);
    // - B[0, 1] * (B[0, 1] * B[2, 2] - B[0, 2] * B[1, 2])                                    <L 891>
    var_222 = wp::extract(var_200, var_220, var_221);
    var_225 = wp::extract(var_200, var_223, var_224);
    var_228 = wp::extract(var_200, var_226, var_227);
    var_229 = wp::mul(var_225, var_228);
    var_232 = wp::extract(var_200, var_230, var_231);
    var_235 = wp::extract(var_200, var_233, var_234);
    var_236 = wp::mul(var_232, var_235);
    var_237 = wp::sub(var_229, var_236);
    var_238 = wp::mul(var_222, var_237);
    var_239 = wp::sub(var_219, var_238);
    // + B[0, 2] * (B[0, 1] * B[1, 2] - B[0, 2] * B[1, 1])                                    <L 892>
    var_242 = wp::extract(var_200, var_240, var_241);
    var_245 = wp::extract(var_200, var_243, var_244);
    var_248 = wp::extract(var_200, var_246, var_247);
    var_249 = wp::mul(var_245, var_248);
    var_252 = wp::extract(var_200, var_250, var_251);
    var_255 = wp::extract(var_200, var_253, var_254);
    var_256 = wp::mul(var_252, var_255);
    var_257 = wp::sub(var_249, var_256);
    var_258 = wp::mul(var_242, var_257);
    var_259 = wp::add(var_239, var_258);
    // r_val = 0.5 * determinant                                                              <L 894>
    var_261 = wp::mul(var_260, var_259);
    // root = 2.0 * wp.cos(wp.acos(wp.min(1.0, wp.abs(r_val))) * (1.0 / 3.0))                 <L 896>
    var_264 = wp::abs(var_261);
    var_265 = wp::min(var_263, var_264);
    var_266 = wp::acos(var_265);
    var_269 = wp::div(var_267, var_268);
    var_270 = wp::mul(var_266, var_269);
    var_271 = wp::cos(var_270);
    var_272 = wp::mul(var_262, var_271);
    // if r_val < 0.0:                                                                        <L 897>
    var_274 = (var_261 < var_273);
    if (var_274) {
        // root = -root                                                                       <L 898>
        var_275 = wp::neg(var_272);
    }
    var_276 = wp::where(var_274, var_275, var_272);
    // B[0, 0] -= root                                                                        <L 899>
    wp::sub_inplace(var_200, var_277, var_278, var_276);
    // B[1, 1] -= root                                                                        <L 900>
    wp::sub_inplace(var_200, var_279, var_280, var_276);
    // B[2, 2] -= root                                                                        <L 901>
    wp::sub_inplace(var_200, var_281, var_282, var_276);
    // row0 = B[0]                                                                            <L 903>
    var_284 = wp::extract(var_200, var_283);
    // row1 = B[1]                                                                            <L 904>
    var_286 = wp::extract(var_200, var_285);
    // row2 = B[2]                                                                            <L 905>
    var_288 = wp::extract(var_200, var_287);
    // cr0 = wp.cross(row0, row1)                                                             <L 906>
    var_289 = wp::cross(var_284, var_286);
    // cr1 = wp.cross(row0, row2)                                                             <L 907>
    var_290 = wp::cross(var_284, var_288);
    // cr2 = wp.cross(row1, row2)                                                             <L 908>
    var_291 = wp::cross(var_286, var_288);
    // n0 = wp.dot(cr0, cr0)                                                                  <L 909>
    var_292 = wp::dot(var_289, var_289);
    // n1 = wp.dot(cr1, cr1)                                                                  <L 910>
    var_293 = wp::dot(var_290, var_290);
    // n2 = wp.dot(cr2, cr2)                                                                  <L 911>
    var_294 = wp::dot(var_291, var_291);
    // best_cr = cr0                                                                          <L 912>
    var_295 = wp::copy(var_289);
    // best_n = n0                                                                            <L 913>
    var_296 = wp::copy(var_292);
    // if n1 > best_n:                                                                        <L 914>
    var_297 = (var_293 > var_296);
    if (var_297) {
        // best_cr = cr1                                                                      <L 915>
        var_298 = wp::copy(var_290);
        // best_n = n1                                                                        <L 916>
        var_299 = wp::copy(var_293);
    }
    var_300 = wp::where(var_297, var_298, var_295);
    var_301 = wp::where(var_297, var_299, var_296);
    // if n2 > best_n:                                                                        <L 917>
    var_302 = (var_294 > var_301);
    if (var_302) {
        // best_cr = cr2                                                                      <L 918>
        var_303 = wp::copy(var_291);
        // best_n = n2                                                                        <L 919>
        var_304 = wp::copy(var_294);
    }
    var_305 = wp::where(var_302, var_303, var_300);
    var_306 = wp::where(var_302, var_304, var_301);
    // q = best_cr * (1.0 / wp.sqrt(best_n))                                                  <L 920>
    var_308 = wp::sqrt(var_306);
    var_309 = wp::div(var_307, var_308);
    var_310 = wp::mul(var_305, var_309);
    // if wp.abs(q[0]) > wp.abs(q[1]):                                                        <L 922>
    var_312 = wp::extract(var_310, var_311);
    var_313 = wp::abs(var_312);
    var_315 = wp::extract(var_310, var_314);
    var_316 = wp::abs(var_315);
    var_317 = (var_313 > var_316);
    if (var_317) {
        // inv = 1.0 / wp.sqrt(q[0] * q[0] + q[2] * q[2])                                     <L 923>
        var_320 = wp::extract(var_310, var_319);
        var_322 = wp::extract(var_310, var_321);
        var_323 = wp::mul(var_320, var_322);
        var_325 = wp::extract(var_310, var_324);
        var_327 = wp::extract(var_310, var_326);
        var_328 = wp::mul(var_325, var_327);
        var_329 = wp::add(var_323, var_328);
        var_330 = wp::sqrt(var_329);
        var_331 = wp::div(var_318, var_330);
        // u = wp.vec3(-q[2] * inv, 0.0, q[0] * inv)                                          <L 924>
        var_333 = wp::extract(var_310, var_332);
        var_334 = wp::neg(var_333);
        var_335 = wp::mul(var_334, var_331);
        var_338 = wp::extract(var_310, var_337);
        var_339 = wp::mul(var_338, var_331);
        var_340 = wp::vec_t<3, wp::float32>(var_335, var_336, var_339);
    }
    if (!var_317) {
        // inv = 1.0 / wp.sqrt(q[1] * q[1] + q[2] * q[2])                                     <L 926>
        var_343 = wp::extract(var_310, var_342);
        var_345 = wp::extract(var_310, var_344);
        var_346 = wp::mul(var_343, var_345);
        var_348 = wp::extract(var_310, var_347);
        var_350 = wp::extract(var_310, var_349);
        var_351 = wp::mul(var_348, var_350);
        var_352 = wp::add(var_346, var_351);
        var_353 = wp::sqrt(var_352);
        var_354 = wp::div(var_341, var_353);
        // u = wp.vec3(0.0, q[2] * inv, -q[1] * inv)                                          <L 927>
        var_357 = wp::extract(var_310, var_356);
        var_358 = wp::mul(var_357, var_354);
        var_360 = wp::extract(var_310, var_359);
        var_361 = wp::neg(var_360);
        var_362 = wp::mul(var_361, var_354);
        var_363 = wp::vec_t<3, wp::float32>(var_355, var_358, var_362);
    }
    var_364 = wp::where(var_317, var_331, var_354);
    var_365 = wp::where(var_317, var_340, var_363);
    // v = wp.cross(q, u)                                                                     <L 928>
    var_366 = wp::cross(var_310, var_365);
    // Du = D * u                                                                             <L 929>
    var_367 = wp::mul(var_51, var_365);
    // Dv = D * v                                                                             <L 930>
    var_368 = wp::mul(var_51, var_366);
    // Dq = D * q                                                                             <L 931>
    var_369 = wp::mul(var_51, var_310);
    // a = wp.dot(u, Du)                                                                      <L 933>
    var_370 = wp::dot(var_365, var_367);
    // b = wp.dot(u, Dv)                                                                      <L 934>
    var_371 = wp::dot(var_365, var_368);
    // c = wp.dot(v, Dv)                                                                      <L 935>
    var_372 = wp::dot(var_366, var_368);
    // cosine = 1.0                                                                           <L 936>
    // sine = 0.0                                                                             <L 937>
    // t = 0.0                                                                                <L 938>
    // if b != 0.0:                                                                           <L 939>
    var_377 = (var_371 != var_376);
    if (var_377) {
        // delta = 0.5 * (c - a)                                                              <L 940>
        var_379 = wp::sub(var_372, var_370);
        var_380 = wp::mul(var_378, var_379);
        // wscale = wp.max(wp.abs(delta), wp.abs(b))                                          <L 941>
        var_381 = wp::abs(var_380);
        var_382 = wp::abs(var_371);
        var_383 = wp::max(var_381, var_382);
        // x = delta / wscale                                                                 <L 942>
        var_384 = wp::div(var_380, var_383);
        // y = b / wscale                                                                     <L 943>
        var_385 = wp::div(var_371, var_383);
        // t = (y if x >= 0.0 else -y) / (wp.abs(x) + wp.sqrt(x * x + y * y))                 <L 944>
        var_387 = (var_384 >= var_386);
        if (var_387) {
        }
        if (!var_387) {
            var_388 = wp::neg(var_385);
        }
        var_389 = wp::where(var_387, var_385, var_388);
        var_390 = wp::abs(var_384);
        var_391 = wp::mul(var_384, var_384);
        var_392 = wp::mul(var_385, var_385);
        var_393 = wp::add(var_391, var_392);
        var_394 = wp::sqrt(var_393);
        var_395 = wp::add(var_390, var_394);
        var_396 = wp::div(var_389, var_395);
        // cosine = 1.0 / wp.sqrt(1.0 + t * t)                                                <L 945>
        var_399 = wp::mul(var_396, var_396);
        var_400 = wp::add(var_398, var_399);
        var_401 = wp::sqrt(var_400);
        var_402 = wp::div(var_397, var_401);
        // sine = t * cosine                                                                  <L 946>
        var_403 = wp::mul(var_396, var_402);
    }
    var_404 = wp::where(var_377, var_402, var_373);
    var_405 = wp::where(var_377, var_403, var_374);
    var_406 = wp::where(var_377, var_396, var_375);
    // value = wp.vec3(wp.dot(q, Dq) * scale, (a - t * b) * scale, (c + t * b) * scale)       <L 948>
    var_407 = wp::dot(var_310, var_369);
    var_408 = wp::mul(var_407, var_40);
    var_409 = wp::mul(var_406, var_371);
    var_410 = wp::sub(var_370, var_409);
    var_411 = wp::mul(var_410, var_40);
    var_412 = wp::mul(var_406, var_371);
    var_413 = wp::add(var_372, var_412);
    var_414 = wp::mul(var_413, var_40);
    var_415 = wp::vec_t<3, wp::float32>(var_408, var_411, var_414);
    // c1 = cosine * u - sine * v                                                             <L 949>
    var_416 = wp::mul(var_404, var_365);
    var_417 = wp::mul(var_405, var_366);
    var_418 = wp::sub(var_416, var_417);
    // c2 = sine * u + cosine * v                                                             <L 950>
    var_419 = wp::mul(var_405, var_365);
    var_420 = wp::mul(var_404, var_366);
    var_421 = wp::add(var_419, var_420);
    // Q = wp.mat33(q[0], c1[0], c2[0], q[1], c1[1], c2[1], q[2], c1[2], c2[2])               <L 951>
    var_423 = wp::extract(var_310, var_422);
    var_425 = wp::extract(var_418, var_424);
    var_427 = wp::extract(var_421, var_426);
    var_429 = wp::extract(var_310, var_428);
    var_431 = wp::extract(var_418, var_430);
    var_433 = wp::extract(var_421, var_432);
    var_435 = wp::extract(var_310, var_434);
    var_437 = wp::extract(var_418, var_436);
    var_439 = wp::extract(var_421, var_438);
    var_440 = wp::mat_t<3, 3, wp::float32>(var_423, var_425, var_427, var_429, var_431, var_433, var_435, var_437, var_439);
    // return value, Q                                                                        <L 952>
    ret_0 = var_415;
    ret_1 = var_440;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1202
static CUDA_CALLABLE void snh_project_0(
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_vert0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::int32 var_vbase,
    wp::vec_t<4, wp::int32> var_vert,
    wp::vec_t<3, wp::float32> var_size,
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::vec_t<6, wp::float32> var_elongation,
    bool & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    wp::mat_t<4, 3, wp::float32> & ret_2,
    wp::mat_t<3, 3, wp::float32> & ret_3,
    wp::vec_t<3, wp::float32> & ret_4,
    wp::vec_t<3, wp::float32> & ret_5)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::int32 var_1;
    wp::int32 var_2;
    wp::vec_t<3, wp::float32>* var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    const wp::int32 var_6 = 1;
    wp::int32 var_7;
    wp::int32 var_8;
    wp::vec_t<3, wp::float32>* var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    const wp::int32 var_12 = 2;
    wp::int32 var_13;
    wp::int32 var_14;
    wp::vec_t<3, wp::float32>* var_15;
    wp::vec_t<3, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    const wp::int32 var_18 = 3;
    wp::int32 var_19;
    wp::int32 var_20;
    wp::vec_t<3, wp::float32>* var_21;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<3, wp::float32> var_23;
    const wp::float32 var_24 = 2.0;
    const wp::int32 var_25 = 0;
    wp::float32 var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 0;
    wp::float32 var_29;
    wp::float32 var_30;
    const wp::float32 var_31 = 2.0;
    const wp::int32 var_32 = 1;
    wp::float32 var_33;
    wp::float32 var_34;
    const wp::int32 var_35 = 1;
    wp::float32 var_36;
    wp::float32 var_37;
    const wp::float32 var_38 = 2.0;
    const wp::int32 var_39 = 2;
    wp::float32 var_40;
    wp::float32 var_41;
    const wp::int32 var_42 = 2;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::vec_t<3, wp::float32> var_45;
    const wp::float32 var_46 = 2.0;
    const wp::int32 var_47 = 0;
    wp::float32 var_48;
    wp::float32 var_49;
    const wp::int32 var_50 = 0;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::float32 var_53 = 2.0;
    const wp::int32 var_54 = 1;
    wp::float32 var_55;
    wp::float32 var_56;
    const wp::int32 var_57 = 1;
    wp::float32 var_58;
    wp::float32 var_59;
    const wp::float32 var_60 = 2.0;
    const wp::int32 var_61 = 2;
    wp::float32 var_62;
    wp::float32 var_63;
    const wp::int32 var_64 = 2;
    wp::float32 var_65;
    wp::float32 var_66;
    wp::vec_t<3, wp::float32> var_67;
    const wp::float32 var_68 = 2.0;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    wp::float32 var_71;
    const wp::int32 var_72 = 0;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::float32 var_75 = 2.0;
    const wp::int32 var_76 = 1;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::int32 var_79 = 1;
    wp::float32 var_80;
    wp::float32 var_81;
    const wp::float32 var_82 = 2.0;
    const wp::int32 var_83 = 2;
    wp::float32 var_84;
    wp::float32 var_85;
    const wp::int32 var_86 = 2;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::vec_t<3, wp::float32> var_89;
    const wp::int32 var_90 = 21;
    wp::int32 var_91;
    wp::float32* var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    const wp::int32 var_95 = 22;
    wp::int32 var_96;
    wp::float32* var_97;
    wp::float32 var_98;
    wp::float32 var_99;
    const wp::int32 var_100 = 23;
    wp::int32 var_101;
    wp::float32* var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    wp::vec_t<3, wp::float32> var_105;
    wp::vec_t<3, wp::float32> var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::vec_t<3, wp::float32> var_110;
    const wp::int32 var_111 = 0;
    wp::vec_t<3, wp::float32> var_112;
    wp::mat_t<3, 3, wp::float32> var_113;
    wp::mat_t<3, 3, wp::float32> var_114;
    const wp::int32 var_115 = 2;
    wp::vec_t<3, wp::float32> var_116;
    wp::mat_t<3, 3, wp::float32> var_117;
    wp::mat_t<3, 3, wp::float32> var_118;
    const wp::int32 var_119 = 4;
    wp::vec_t<3, wp::float32> var_120;
    wp::mat_t<3, 3, wp::float32> var_121;
    wp::mat_t<3, 3, wp::float32> var_122;
    wp::mat_t<3, 3, wp::float32> var_123;
    wp::vec_t<3, wp::float32> var_124;
    wp::mat_t<3, 3, wp::float32> var_125;
    wp::mat_t<3, 3, wp::float32> var_126;
    wp::vec_t<3, wp::float32> var_127;
    wp::vec_t<3, wp::float32> var_128;
    wp::vec_t<3, wp::float32> var_129;
    wp::vec_t<3, wp::float32> var_130;
    wp::vec_t<3, wp::float32> var_131;
    wp::vec_t<3, wp::float32> var_132;
    wp::mat_t<4, 3, wp::float32> var_133;
    const wp::int32 var_134 = 0;
    const wp::int32 var_135 = 1;
    const wp::int32 var_136 = 2;
    const wp::int32 var_137 = 3;
    const wp::int32 var_138 = 6;
    wp::mat_t<6, 6, wp::float32> var_139;
    wp::vec_t<6, wp::float32> var_140;
    wp::vec_t<6, wp::float32> var_141;
    wp::mat_t<6, 6, wp::float32> var_142;
    wp::mat_t<4, 3, wp::float32> var_143;
    const wp::float32 var_144 = 0.0;
    const wp::float32 var_145 = 0.0;
    const wp::float32 var_146 = 0.0;
    wp::vec_t<3, wp::float32> var_147;
    const wp::int32 var_148 = 0;
    wp::vec_t<3, wp::float32> var_149;
    const wp::int32 var_150 = 1;
    wp::vec_t<3, wp::float32> var_151;
    const wp::int32 var_152 = 2;
    wp::vec_t<3, wp::float32> var_153;
    const wp::int32 var_154 = 3;
    const wp::int32 var_155 = 3;
    wp::mat_t<6, 2, wp::int32> var_156;
    wp::mat_t<6, 3, wp::float32> var_157;
    const wp::float32 var_158 = 0.0;
    const wp::float32 var_159 = 0.0;
    const wp::float32 var_160 = 0.0;
    const wp::float32 var_161 = 0.0;
    const wp::float32 var_162 = 0.0;
    const wp::float32 var_163 = 0.0;
    wp::vec_t<6, wp::float32> var_164;
    const wp::float32 var_165 = 0.0;
    const wp::float32 var_166 = 0.0;
    const wp::float32 var_167 = 0.0;
    const wp::float32 var_168 = 0.0;
    const wp::float32 var_169 = 0.0;
    const wp::float32 var_170 = 0.0;
    wp::vec_t<6, wp::float32> var_171;
    const wp::float32 var_172 = 0.0;
    const wp::float32 var_173 = 0.0;
    const wp::float32 var_174 = 0.0;
    const wp::float32 var_175 = 0.0;
    const wp::float32 var_176 = 0.0;
    const wp::float32 var_177 = 0.0;
    wp::vec_t<6, wp::float32> var_178;
    const wp::int32 var_179 = 0;
    const wp::int32 var_180 = 0;
    wp::int32 var_181;
    wp::vec_t<3, wp::float32> var_182;
    const wp::int32 var_183 = 1;
    wp::int32 var_184;
    wp::vec_t<3, wp::float32> var_185;
    wp::vec_t<3, wp::float32> var_186;
    const wp::int32 var_187 = 0;
    wp::float32 var_188;
    const wp::int32 var_189 = 0;
    wp::float32 var_190;
    wp::float32 var_191;
    const wp::int32 var_192 = 1;
    wp::float32 var_193;
    const wp::int32 var_194 = 1;
    wp::float32 var_195;
    wp::float32 var_196;
    const wp::int32 var_197 = 2;
    wp::float32 var_198;
    const wp::int32 var_199 = 2;
    wp::float32 var_200;
    wp::float32 var_201;
    const wp::int32 var_202 = 1;
    const wp::int32 var_203 = 0;
    wp::int32 var_204;
    wp::vec_t<3, wp::float32> var_205;
    const wp::int32 var_206 = 1;
    wp::int32 var_207;
    wp::vec_t<3, wp::float32> var_208;
    wp::vec_t<3, wp::float32> var_209;
    const wp::int32 var_210 = 0;
    wp::float32 var_211;
    const wp::int32 var_212 = 0;
    wp::float32 var_213;
    wp::float32 var_214;
    const wp::int32 var_215 = 1;
    wp::float32 var_216;
    const wp::int32 var_217 = 1;
    wp::float32 var_218;
    wp::float32 var_219;
    const wp::int32 var_220 = 2;
    wp::float32 var_221;
    const wp::int32 var_222 = 2;
    wp::float32 var_223;
    wp::float32 var_224;
    const wp::int32 var_225 = 2;
    const wp::int32 var_226 = 0;
    wp::int32 var_227;
    wp::vec_t<3, wp::float32> var_228;
    const wp::int32 var_229 = 1;
    wp::int32 var_230;
    wp::vec_t<3, wp::float32> var_231;
    wp::vec_t<3, wp::float32> var_232;
    const wp::int32 var_233 = 0;
    wp::float32 var_234;
    const wp::int32 var_235 = 0;
    wp::float32 var_236;
    wp::float32 var_237;
    const wp::int32 var_238 = 1;
    wp::float32 var_239;
    const wp::int32 var_240 = 1;
    wp::float32 var_241;
    wp::float32 var_242;
    const wp::int32 var_243 = 2;
    wp::float32 var_244;
    const wp::int32 var_245 = 2;
    wp::float32 var_246;
    wp::float32 var_247;
    const wp::int32 var_248 = 3;
    const wp::int32 var_249 = 0;
    wp::int32 var_250;
    wp::vec_t<3, wp::float32> var_251;
    const wp::int32 var_252 = 1;
    wp::int32 var_253;
    wp::vec_t<3, wp::float32> var_254;
    wp::vec_t<3, wp::float32> var_255;
    const wp::int32 var_256 = 0;
    wp::float32 var_257;
    const wp::int32 var_258 = 0;
    wp::float32 var_259;
    wp::float32 var_260;
    const wp::int32 var_261 = 1;
    wp::float32 var_262;
    const wp::int32 var_263 = 1;
    wp::float32 var_264;
    wp::float32 var_265;
    const wp::int32 var_266 = 2;
    wp::float32 var_267;
    const wp::int32 var_268 = 2;
    wp::float32 var_269;
    wp::float32 var_270;
    const wp::int32 var_271 = 4;
    const wp::int32 var_272 = 0;
    wp::int32 var_273;
    wp::vec_t<3, wp::float32> var_274;
    const wp::int32 var_275 = 1;
    wp::int32 var_276;
    wp::vec_t<3, wp::float32> var_277;
    wp::vec_t<3, wp::float32> var_278;
    const wp::int32 var_279 = 0;
    wp::float32 var_280;
    const wp::int32 var_281 = 0;
    wp::float32 var_282;
    wp::float32 var_283;
    const wp::int32 var_284 = 1;
    wp::float32 var_285;
    const wp::int32 var_286 = 1;
    wp::float32 var_287;
    wp::float32 var_288;
    const wp::int32 var_289 = 2;
    wp::float32 var_290;
    const wp::int32 var_291 = 2;
    wp::float32 var_292;
    wp::float32 var_293;
    const wp::int32 var_294 = 5;
    const wp::int32 var_295 = 0;
    wp::int32 var_296;
    wp::vec_t<3, wp::float32> var_297;
    const wp::int32 var_298 = 1;
    wp::int32 var_299;
    wp::vec_t<3, wp::float32> var_300;
    wp::vec_t<3, wp::float32> var_301;
    const wp::int32 var_302 = 0;
    wp::float32 var_303;
    const wp::int32 var_304 = 0;
    wp::float32 var_305;
    wp::float32 var_306;
    const wp::int32 var_307 = 1;
    wp::float32 var_308;
    const wp::int32 var_309 = 1;
    wp::float32 var_310;
    wp::float32 var_311;
    const wp::int32 var_312 = 2;
    wp::float32 var_313;
    const wp::int32 var_314 = 2;
    wp::float32 var_315;
    wp::float32 var_316;
    wp::vec_t<6, wp::float32> var_317;
    wp::vec_t<6, wp::float32> var_318;
    wp::vec_t<6, wp::float32> var_319;
    wp::float32 var_320;
    wp::float32 var_321;
    wp::float32 var_322;
    wp::vec_t<3, wp::float32> var_323;
    const wp::int32 var_324 = 0;
    wp::float32 var_325;
    const wp::int32 var_326 = 1;
    wp::float32 var_327;
    wp::float32 var_328;
    const wp::int32 var_329 = 2;
    wp::float32 var_330;
    wp::float32 var_331;
    const wp::float32 var_332 = 2.0;
    wp::float32 var_333;
    const wp::float32 var_334 = 1.0;
    wp::float32 var_335;
    wp::float32 var_336;
    const wp::int32 var_337 = 1;
    wp::float32 var_338;
    const wp::int32 var_339 = 2;
    wp::float32 var_340;
    wp::float32 var_341;
    const wp::int32 var_342 = 0;
    wp::float32 var_343;
    const wp::int32 var_344 = 2;
    wp::float32 var_345;
    wp::float32 var_346;
    const wp::int32 var_347 = 0;
    wp::float32 var_348;
    const wp::int32 var_349 = 1;
    wp::float32 var_350;
    wp::float32 var_351;
    wp::vec_t<3, wp::float32> var_352;
    const wp::float32 var_353 = 2.0;
    const wp::int32 var_354 = 0;
    wp::float32 var_355;
    wp::float32 var_356;
    const wp::int32 var_357 = 0;
    wp::float32 var_358;
    wp::float32 var_359;
    wp::float32 var_360;
    wp::float32 var_361;
    const wp::int32 var_362 = 0;
    wp::float32 var_363;
    wp::float32 var_364;
    const wp::int32 var_365 = 0;
    wp::float32 var_366;
    wp::float32 var_367;
    wp::float32 var_368;
    const wp::int32 var_369 = 0;
    wp::float32 var_370;
    wp::float32 var_371;
    const wp::float32 var_372 = 2.0;
    const wp::int32 var_373 = 1;
    wp::float32 var_374;
    wp::float32 var_375;
    const wp::int32 var_376 = 1;
    wp::float32 var_377;
    wp::float32 var_378;
    wp::float32 var_379;
    wp::float32 var_380;
    const wp::int32 var_381 = 1;
    wp::float32 var_382;
    wp::float32 var_383;
    const wp::int32 var_384 = 1;
    wp::float32 var_385;
    wp::float32 var_386;
    wp::float32 var_387;
    const wp::int32 var_388 = 1;
    wp::float32 var_389;
    wp::float32 var_390;
    const wp::float32 var_391 = 2.0;
    const wp::int32 var_392 = 2;
    wp::float32 var_393;
    wp::float32 var_394;
    const wp::int32 var_395 = 2;
    wp::float32 var_396;
    wp::float32 var_397;
    wp::float32 var_398;
    wp::float32 var_399;
    const wp::int32 var_400 = 2;
    wp::float32 var_401;
    wp::float32 var_402;
    const wp::int32 var_403 = 2;
    wp::float32 var_404;
    wp::float32 var_405;
    wp::float32 var_406;
    const wp::int32 var_407 = 2;
    wp::float32 var_408;
    wp::float32 var_409;
    const wp::float32 var_410 = 2.0;
    const wp::int32 var_411 = 0;
    wp::float32 var_412;
    wp::float32 var_413;
    const wp::int32 var_414 = 1;
    wp::float32 var_415;
    wp::float32 var_416;
    wp::float32 var_417;
    wp::float32 var_418;
    const wp::int32 var_419 = 0;
    wp::float32 var_420;
    wp::float32 var_421;
    const wp::int32 var_422 = 1;
    wp::float32 var_423;
    wp::float32 var_424;
    wp::float32 var_425;
    const wp::int32 var_426 = 2;
    wp::float32 var_427;
    wp::float32 var_428;
    wp::float32 var_429;
    const wp::float32 var_430 = 2.0;
    const wp::int32 var_431 = 0;
    wp::float32 var_432;
    wp::float32 var_433;
    const wp::int32 var_434 = 2;
    wp::float32 var_435;
    wp::float32 var_436;
    wp::float32 var_437;
    wp::float32 var_438;
    const wp::int32 var_439 = 0;
    wp::float32 var_440;
    wp::float32 var_441;
    const wp::int32 var_442 = 2;
    wp::float32 var_443;
    wp::float32 var_444;
    wp::float32 var_445;
    const wp::int32 var_446 = 1;
    wp::float32 var_447;
    wp::float32 var_448;
    wp::float32 var_449;
    const wp::float32 var_450 = 2.0;
    const wp::int32 var_451 = 1;
    wp::float32 var_452;
    wp::float32 var_453;
    const wp::int32 var_454 = 2;
    wp::float32 var_455;
    wp::float32 var_456;
    wp::float32 var_457;
    wp::float32 var_458;
    const wp::int32 var_459 = 1;
    wp::float32 var_460;
    wp::float32 var_461;
    const wp::int32 var_462 = 2;
    wp::float32 var_463;
    wp::float32 var_464;
    wp::float32 var_465;
    const wp::int32 var_466 = 0;
    wp::float32 var_467;
    wp::float32 var_468;
    wp::float32 var_469;
    wp::mat_t<3, 3, wp::float32> var_470;
    bool var_471;
    wp::mat_t<3, 3, wp::float32> var_472;
    wp::vec_t<3, wp::float32> var_473;
    wp::mat_t<3, 3, wp::float32> var_474;
    const wp::float32 var_475 = 0.0;
    const wp::int32 var_476 = 0;
    wp::float32 var_477;
    wp::float32 var_478;
    const wp::float32 var_479 = 0.0;
    const wp::int32 var_480 = 1;
    wp::float32 var_481;
    wp::float32 var_482;
    const wp::float32 var_483 = 0.0;
    const wp::int32 var_484 = 2;
    wp::float32 var_485;
    wp::float32 var_486;
    const wp::int32 var_487 = 0;
    const wp::int32 var_488 = 0;
    wp::float32 var_489;
    wp::float32 var_490;
    const wp::int32 var_491 = 0;
    const wp::int32 var_492 = 1;
    wp::float32 var_493;
    wp::float32 var_494;
    const wp::int32 var_495 = 0;
    const wp::int32 var_496 = 2;
    wp::float32 var_497;
    wp::float32 var_498;
    const wp::int32 var_499 = 1;
    const wp::int32 var_500 = 0;
    wp::float32 var_501;
    wp::float32 var_502;
    const wp::int32 var_503 = 1;
    const wp::int32 var_504 = 1;
    wp::float32 var_505;
    wp::float32 var_506;
    const wp::int32 var_507 = 1;
    const wp::int32 var_508 = 2;
    wp::float32 var_509;
    wp::float32 var_510;
    const wp::int32 var_511 = 2;
    const wp::int32 var_512 = 0;
    wp::float32 var_513;
    wp::float32 var_514;
    const wp::int32 var_515 = 2;
    const wp::int32 var_516 = 1;
    wp::float32 var_517;
    wp::float32 var_518;
    const wp::int32 var_519 = 2;
    const wp::int32 var_520 = 2;
    wp::float32 var_521;
    wp::float32 var_522;
    wp::mat_t<3, 3, wp::float32> var_523;
    wp::mat_t<3, 3, wp::float32> var_524;
    wp::mat_t<3, 3, wp::float32> var_525;
    wp::mat_t<3, 3, wp::float32> var_526;
    const wp::float32 var_527 = 0.0;
    const wp::float32 var_528 = 0.0;
    const wp::float32 var_529 = 0.0;
    wp::vec_t<3, wp::float32> var_530;
    const wp::float32 var_531 = 0.0;
    const wp::float32 var_532 = 0.0;
    const wp::float32 var_533 = 0.0;
    wp::vec_t<3, wp::float32> var_534;
    const wp::int32 var_535 = 0;
    const wp::int32 var_536 = 0;
    const wp::int32 var_537 = 1;
    wp::vec_t<3, wp::int32> var_538;
    const wp::int32 var_539 = 1;
    const wp::int32 var_540 = 2;
    const wp::int32 var_541 = 2;
    wp::vec_t<3, wp::int32> var_542;
    const wp::int32 var_543 = 0;
    wp::int32 var_544;
    wp::int32 var_545;
    const wp::float32 var_546 = 0.0;
    const wp::float32 var_547 = 0.0;
    const wp::float32 var_548 = 0.0;
    const wp::float32 var_549 = 0.0;
    const wp::float32 var_550 = 0.0;
    const wp::float32 var_551 = 0.0;
    wp::vec_t<6, wp::float32> var_552;
    const wp::int32 var_553 = 0;
    wp::float32 var_554;
    wp::float32 var_555;
    wp::float32 var_556;
    const wp::int32 var_557 = 1;
    wp::float32 var_558;
    wp::float32 var_559;
    wp::float32 var_560;
    const wp::int32 var_561 = 2;
    wp::float32 var_562;
    wp::float32 var_563;
    wp::float32 var_564;
    const wp::int32 var_565 = 3;
    wp::float32 var_566;
    wp::float32 var_567;
    wp::float32 var_568;
    const wp::int32 var_569 = 4;
    wp::float32 var_570;
    wp::float32 var_571;
    wp::float32 var_572;
    const wp::int32 var_573 = 5;
    wp::float32 var_574;
    wp::float32 var_575;
    wp::float32 var_576;
    wp::vec_t<6, wp::float32> var_577;
    wp::float32 var_578;
    const wp::float32 var_579 = 0.5;
    wp::float32 var_580;
    wp::float32 var_581;
    wp::float32 var_582;
    wp::float32 var_583;
    wp::float32 var_584;
    wp::float32 var_585;
    wp::float32 var_586;
    wp::float32 var_587;
    wp::float32 var_588;
    wp::float32 var_589;
    const wp::int32 var_590 = 3;
    wp::int32 var_591;
    wp::int32 var_592;
    wp::float32 var_593;
    wp::float32 var_594;
    const wp::float32 var_595 = 0.0;
    wp::float32 var_596;
    wp::float32 var_597;
    wp::float32 var_598;
    wp::float32 var_599;
    wp::float32 var_600;
    const wp::float32 var_601 = 0.0;
    wp::float32 var_602;
    wp::float32 var_603;
    wp::float32 var_604;
    wp::float32 var_605;
    wp::float32 var_606;
    const wp::int32 var_607 = 1;
    wp::int32 var_608;
    wp::int32 var_609;
    const wp::float32 var_610 = 0.0;
    const wp::float32 var_611 = 0.0;
    const wp::float32 var_612 = 0.0;
    const wp::float32 var_613 = 0.0;
    const wp::float32 var_614 = 0.0;
    const wp::float32 var_615 = 0.0;
    wp::vec_t<6, wp::float32> var_616;
    const wp::int32 var_617 = 0;
    wp::float32 var_618;
    wp::float32 var_619;
    wp::float32 var_620;
    const wp::int32 var_621 = 1;
    wp::float32 var_622;
    wp::float32 var_623;
    wp::float32 var_624;
    const wp::int32 var_625 = 2;
    wp::float32 var_626;
    wp::float32 var_627;
    wp::float32 var_628;
    const wp::int32 var_629 = 3;
    wp::float32 var_630;
    wp::float32 var_631;
    wp::float32 var_632;
    const wp::int32 var_633 = 4;
    wp::float32 var_634;
    wp::float32 var_635;
    wp::float32 var_636;
    const wp::int32 var_637 = 5;
    wp::float32 var_638;
    wp::float32 var_639;
    wp::float32 var_640;
    wp::vec_t<6, wp::float32> var_641;
    wp::float32 var_642;
    const wp::float32 var_643 = 0.5;
    wp::float32 var_644;
    wp::float32 var_645;
    wp::float32 var_646;
    wp::float32 var_647;
    wp::float32 var_648;
    wp::float32 var_649;
    wp::float32 var_650;
    wp::float32 var_651;
    wp::float32 var_652;
    wp::float32 var_653;
    const wp::int32 var_654 = 3;
    wp::int32 var_655;
    wp::int32 var_656;
    wp::float32 var_657;
    wp::float32 var_658;
    const wp::float32 var_659 = 0.0;
    wp::float32 var_660;
    wp::float32 var_661;
    wp::float32 var_662;
    wp::float32 var_663;
    wp::float32 var_664;
    const wp::float32 var_665 = 0.0;
    wp::float32 var_666;
    wp::float32 var_667;
    wp::float32 var_668;
    wp::float32 var_669;
    wp::float32 var_670;
    const wp::int32 var_671 = 2;
    wp::int32 var_672;
    wp::int32 var_673;
    const wp::float32 var_674 = 0.0;
    const wp::float32 var_675 = 0.0;
    const wp::float32 var_676 = 0.0;
    const wp::float32 var_677 = 0.0;
    const wp::float32 var_678 = 0.0;
    const wp::float32 var_679 = 0.0;
    wp::vec_t<6, wp::float32> var_680;
    const wp::int32 var_681 = 0;
    wp::float32 var_682;
    wp::float32 var_683;
    wp::float32 var_684;
    const wp::int32 var_685 = 1;
    wp::float32 var_686;
    wp::float32 var_687;
    wp::float32 var_688;
    const wp::int32 var_689 = 2;
    wp::float32 var_690;
    wp::float32 var_691;
    wp::float32 var_692;
    const wp::int32 var_693 = 3;
    wp::float32 var_694;
    wp::float32 var_695;
    wp::float32 var_696;
    const wp::int32 var_697 = 4;
    wp::float32 var_698;
    wp::float32 var_699;
    wp::float32 var_700;
    const wp::int32 var_701 = 5;
    wp::float32 var_702;
    wp::float32 var_703;
    wp::float32 var_704;
    wp::vec_t<6, wp::float32> var_705;
    wp::float32 var_706;
    const wp::float32 var_707 = 0.5;
    wp::float32 var_708;
    wp::float32 var_709;
    wp::float32 var_710;
    wp::float32 var_711;
    wp::float32 var_712;
    wp::float32 var_713;
    wp::float32 var_714;
    wp::float32 var_715;
    wp::float32 var_716;
    wp::float32 var_717;
    const wp::int32 var_718 = 3;
    wp::int32 var_719;
    wp::int32 var_720;
    wp::float32 var_721;
    wp::float32 var_722;
    const wp::float32 var_723 = 0.0;
    wp::float32 var_724;
    wp::float32 var_725;
    wp::float32 var_726;
    wp::float32 var_727;
    wp::float32 var_728;
    const wp::float32 var_729 = 0.0;
    wp::float32 var_730;
    wp::float32 var_731;
    wp::float32 var_732;
    wp::float32 var_733;
    wp::float32 var_734;
    bool var_735;
    const wp::int32 var_736 = 0;
    const wp::int32 var_737 = 0;
    wp::float32 var_738;
    const wp::float32 var_739 = 0.0;
    bool var_740;
    const wp::int32 var_741 = 1;
    const wp::int32 var_742 = 1;
    wp::float32 var_743;
    const wp::float32 var_744 = 0.0;
    bool var_745;
    const wp::int32 var_746 = 2;
    const wp::int32 var_747 = 2;
    wp::float32 var_748;
    const wp::float32 var_749 = 0.0;
    bool var_750;
    const wp::int32 var_751 = 0;
    wp::float32 var_752;
    const wp::float32 var_753 = 0.0;
    bool var_754;
    const wp::int32 var_755 = 1;
    wp::float32 var_756;
    const wp::float32 var_757 = 0.0;
    bool var_758;
    const wp::int32 var_759 = 2;
    wp::float32 var_760;
    const wp::float32 var_761 = 0.0;
    bool var_762;
    const wp::int32 var_763 = 0;
    wp::float32 var_764;
    const wp::float32 var_765 = 0.0;
    bool var_766;
    const wp::int32 var_767 = 1;
    wp::float32 var_768;
    const wp::float32 var_769 = 0.0;
    bool var_770;
    const wp::int32 var_771 = 2;
    wp::float32 var_772;
    const wp::float32 var_773 = 0.0;
    bool var_774;
    //---------
    // forward
    // def snh_project(                                                                       <L 1203>
    // v0 = flex_vert0[vbase + vert[0]]                                                       <L 1215>
    var_1 = wp::extract(var_vert, var_0);
    var_2 = wp::add(var_vbase, var_1);
    var_3 = wp::address(var_flex_vert0, var_2);
    var_5 = wp::load(var_3);
    var_4 = wp::copy(var_5);
    // v1 = flex_vert0[vbase + vert[1]] - v0                                                  <L 1216>
    var_7 = wp::extract(var_vert, var_6);
    var_8 = wp::add(var_vbase, var_7);
    var_9 = wp::address(var_flex_vert0, var_8);
    var_11 = wp::load(var_9);
    var_10 = wp::sub(var_11, var_4);
    // v2 = flex_vert0[vbase + vert[2]] - v0                                                  <L 1217>
    var_13 = wp::extract(var_vert, var_12);
    var_14 = wp::add(var_vbase, var_13);
    var_15 = wp::address(var_flex_vert0, var_14);
    var_17 = wp::load(var_15);
    var_16 = wp::sub(var_17, var_4);
    // v3 = flex_vert0[vbase + vert[3]] - v0                                                  <L 1218>
    var_19 = wp::extract(var_vert, var_18);
    var_20 = wp::add(var_vbase, var_19);
    var_21 = wp::address(var_flex_vert0, var_20);
    var_23 = wp::load(var_21);
    var_22 = wp::sub(var_23, var_4);
    // rest0 = wp.vec3(2.0 * size[0] * v1[0], 2.0 * size[1] * v1[1], 2.0 * size[2] * v1[2])       <L 1219>
    var_26 = wp::extract(var_size, var_25);
    var_27 = wp::mul(var_24, var_26);
    var_29 = wp::extract(var_10, var_28);
    var_30 = wp::mul(var_27, var_29);
    var_33 = wp::extract(var_size, var_32);
    var_34 = wp::mul(var_31, var_33);
    var_36 = wp::extract(var_10, var_35);
    var_37 = wp::mul(var_34, var_36);
    var_40 = wp::extract(var_size, var_39);
    var_41 = wp::mul(var_38, var_40);
    var_43 = wp::extract(var_10, var_42);
    var_44 = wp::mul(var_41, var_43);
    var_45 = wp::vec_t<3, wp::float32>(var_30, var_37, var_44);
    // rest1 = wp.vec3(2.0 * size[0] * v2[0], 2.0 * size[1] * v2[1], 2.0 * size[2] * v2[2])       <L 1220>
    var_48 = wp::extract(var_size, var_47);
    var_49 = wp::mul(var_46, var_48);
    var_51 = wp::extract(var_16, var_50);
    var_52 = wp::mul(var_49, var_51);
    var_55 = wp::extract(var_size, var_54);
    var_56 = wp::mul(var_53, var_55);
    var_58 = wp::extract(var_16, var_57);
    var_59 = wp::mul(var_56, var_58);
    var_62 = wp::extract(var_size, var_61);
    var_63 = wp::mul(var_60, var_62);
    var_65 = wp::extract(var_16, var_64);
    var_66 = wp::mul(var_63, var_65);
    var_67 = wp::vec_t<3, wp::float32>(var_52, var_59, var_66);
    // rest2 = wp.vec3(2.0 * size[0] * v3[0], 2.0 * size[1] * v3[1], 2.0 * size[2] * v3[2])       <L 1221>
    var_70 = wp::extract(var_size, var_69);
    var_71 = wp::mul(var_68, var_70);
    var_73 = wp::extract(var_22, var_72);
    var_74 = wp::mul(var_71, var_73);
    var_77 = wp::extract(var_size, var_76);
    var_78 = wp::mul(var_75, var_77);
    var_80 = wp::extract(var_22, var_79);
    var_81 = wp::mul(var_78, var_80);
    var_84 = wp::extract(var_size, var_83);
    var_85 = wp::mul(var_82, var_84);
    var_87 = wp::extract(var_22, var_86);
    var_88 = wp::mul(var_85, var_87);
    var_89 = wp::vec_t<3, wp::float32>(var_74, var_81, var_88);
    // k21 = flex_stiffness[stiffness_adr + 21]                                               <L 1223>
    var_91 = wp::add(var_stiffness_adr, var_90);
    var_92 = wp::address(var_flex_stiffness, var_91);
    var_94 = wp::load(var_92);
    var_93 = wp::copy(var_94);
    // k22 = flex_stiffness[stiffness_adr + 22]                                               <L 1224>
    var_96 = wp::add(var_stiffness_adr, var_95);
    var_97 = wp::address(var_flex_stiffness, var_96);
    var_99 = wp::load(var_97);
    var_98 = wp::copy(var_99);
    // k23 = flex_stiffness[stiffness_adr + 23]                                               <L 1225>
    var_101 = wp::add(var_stiffness_adr, var_100);
    var_102 = wp::address(var_flex_stiffness, var_101);
    var_104 = wp::load(var_102);
    var_103 = wp::copy(var_104);
    // grad0 = wp.cross(rest1, rest2) * k23                                                   <L 1227>
    var_105 = wp::cross(var_67, var_89);
    var_106 = wp::mul(var_105, var_103);
    // grad1 = wp.cross(rest2, rest0) * k23                                                   <L 1228>
    var_107 = wp::cross(var_89, var_45);
    var_108 = wp::mul(var_107, var_103);
    // grad2 = wp.cross(rest0, rest1) * k23                                                   <L 1229>
    var_109 = wp::cross(var_45, var_67);
    var_110 = wp::mul(var_109, var_103);
    // F = -wp.outer(edgevec[0], grad0) + wp.outer(edgevec[2], grad1) - wp.outer(edgevec[4], grad2)       <L 1231>
    var_112 = wp::extract(var_edgevec, var_111);
    var_113 = wp::outer(var_112, var_106);
    var_114 = wp::neg(var_113);
    var_116 = wp::extract(var_edgevec, var_115);
    var_117 = wp::outer(var_116, var_108);
    var_118 = wp::add(var_114, var_117);
    var_120 = wp::extract(var_edgevec, var_119);
    var_121 = wp::outer(var_120, var_110);
    var_122 = wp::sub(var_118, var_121);
    // rotation, sigma, V = _snh_svd(F)                                                       <L 1232>
    _snh_svd_0(var_122, var_123, var_124, var_125);
    // VT = wp.transpose(V)                                                                   <L 1233>
    var_126 = wp::transpose(var_125);
    // hg1 = VT * grad0                                                                       <L 1235>
    var_127 = wp::mul(var_126, var_106);
    // hg2 = VT * grad1                                                                       <L 1236>
    var_128 = wp::mul(var_126, var_108);
    // hg3 = VT * grad2                                                                       <L 1237>
    var_129 = wp::mul(var_126, var_110);
    // hg0 = -(hg1 + hg2 + hg3)                                                               <L 1238>
    var_130 = wp::add(var_127, var_128);
    var_131 = wp::add(var_130, var_129);
    var_132 = wp::neg(var_131);
    // gradient = mat43()                                                                     <L 1239>
    var_133 = wp::mat_t<4, 3, wp::float32>();
    // gradient[0] = hg0                                                                      <L 1240>
    wp::assign_inplace(var_133, var_134, var_132);
    // gradient[1] = hg1                                                                      <L 1241>
    wp::assign_inplace(var_133, var_135, var_127);
    // gradient[2] = hg2                                                                      <L 1242>
    wp::assign_inplace(var_133, var_136, var_128);
    // gradient[3] = hg3                                                                      <L 1243>
    wp::assign_inplace(var_133, var_137, var_129);
    // metric, tension = stretch_elasticity(flex_stiffness, stiffness_adr, elongation, 6)       <L 1245>
    stretch_elasticity_0(var_flex_stiffness, var_stiffness_adr, var_elongation, var_138, var_139, var_140);
    // tension = _snh_cubic(tension, elongation, k21)                                         <L 1246>
    var_141 = _snh_cubic_0(var_140, var_elongation, var_93);
    // metric = _snh_cubic_metric(metric, elongation, k21)                                    <L 1247>
    var_142 = _snh_cubic_metric_0(var_139, var_elongation, var_93);
    // vertex = mat43()                                                                       <L 1249>
    var_143 = wp::mat_t<4, 3, wp::float32>();
    // vertex[0] = wp.vec3(0.0, 0.0, 0.0)                                                     <L 1250>
    var_147 = wp::vec_t<3, wp::float32>(var_144, var_145, var_146);
    wp::assign_inplace(var_143, var_148, var_147);
    // vertex[1] = VT * rest0                                                                 <L 1251>
    var_149 = wp::mul(var_126, var_45);
    wp::assign_inplace(var_143, var_150, var_149);
    // vertex[2] = VT * rest1                                                                 <L 1252>
    var_151 = wp::mul(var_126, var_67);
    wp::assign_inplace(var_143, var_152, var_151);
    // vertex[3] = VT * rest2                                                                 <L 1253>
    var_153 = wp::mul(var_126, var_89);
    wp::assign_inplace(var_143, var_154, var_153);
    // edges = stretch_edge_endpoints(3)                                                      <L 1255>
    var_156 = stretch_edge_endpoints_0(var_155);
    // reference = mat63()                                                                    <L 1256>
    var_157 = wp::mat_t<6, 3, wp::float32>();
    // sq0 = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                               <L 1257>
    var_164 = wp::vec_t<6, wp::float32>({var_158, var_159, var_160, var_161, var_162, var_163});
    // sq1 = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                               <L 1258>
    var_171 = wp::vec_t<6, wp::float32>({var_165, var_166, var_167, var_168, var_169, var_170});
    // sq2 = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                               <L 1259>
    var_178 = wp::vec_t<6, wp::float32>({var_172, var_173, var_174, var_175, var_176, var_177});
    // for e in range(6):                                                                     <L 1260>
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_181 = wp::extract(var_156, var_179, var_180);
    var_182 = wp::extract(var_143, var_181);
    var_184 = wp::extract(var_156, var_179, var_183);
    var_185 = wp::extract(var_143, var_184);
    var_186 = wp::sub(var_182, var_185);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_179, var_186);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_188 = wp::extract(var_186, var_187);
    var_190 = wp::extract(var_186, var_189);
    var_191 = wp::mul(var_188, var_190);
    wp::assign_inplace(var_164, var_179, var_191);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_193 = wp::extract(var_186, var_192);
    var_195 = wp::extract(var_186, var_194);
    var_196 = wp::mul(var_193, var_195);
    wp::assign_inplace(var_171, var_179, var_196);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_198 = wp::extract(var_186, var_197);
    var_200 = wp::extract(var_186, var_199);
    var_201 = wp::mul(var_198, var_200);
    wp::assign_inplace(var_178, var_179, var_201);
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_204 = wp::extract(var_156, var_202, var_203);
    var_205 = wp::extract(var_143, var_204);
    var_207 = wp::extract(var_156, var_202, var_206);
    var_208 = wp::extract(var_143, var_207);
    var_209 = wp::sub(var_205, var_208);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_202, var_209);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_211 = wp::extract(var_209, var_210);
    var_213 = wp::extract(var_209, var_212);
    var_214 = wp::mul(var_211, var_213);
    wp::assign_inplace(var_164, var_202, var_214);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_216 = wp::extract(var_209, var_215);
    var_218 = wp::extract(var_209, var_217);
    var_219 = wp::mul(var_216, var_218);
    wp::assign_inplace(var_171, var_202, var_219);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_221 = wp::extract(var_209, var_220);
    var_223 = wp::extract(var_209, var_222);
    var_224 = wp::mul(var_221, var_223);
    wp::assign_inplace(var_178, var_202, var_224);
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_227 = wp::extract(var_156, var_225, var_226);
    var_228 = wp::extract(var_143, var_227);
    var_230 = wp::extract(var_156, var_225, var_229);
    var_231 = wp::extract(var_143, var_230);
    var_232 = wp::sub(var_228, var_231);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_225, var_232);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_234 = wp::extract(var_232, var_233);
    var_236 = wp::extract(var_232, var_235);
    var_237 = wp::mul(var_234, var_236);
    wp::assign_inplace(var_164, var_225, var_237);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_239 = wp::extract(var_232, var_238);
    var_241 = wp::extract(var_232, var_240);
    var_242 = wp::mul(var_239, var_241);
    wp::assign_inplace(var_171, var_225, var_242);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_244 = wp::extract(var_232, var_243);
    var_246 = wp::extract(var_232, var_245);
    var_247 = wp::mul(var_244, var_246);
    wp::assign_inplace(var_178, var_225, var_247);
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_250 = wp::extract(var_156, var_248, var_249);
    var_251 = wp::extract(var_143, var_250);
    var_253 = wp::extract(var_156, var_248, var_252);
    var_254 = wp::extract(var_143, var_253);
    var_255 = wp::sub(var_251, var_254);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_248, var_255);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_257 = wp::extract(var_255, var_256);
    var_259 = wp::extract(var_255, var_258);
    var_260 = wp::mul(var_257, var_259);
    wp::assign_inplace(var_164, var_248, var_260);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_262 = wp::extract(var_255, var_261);
    var_264 = wp::extract(var_255, var_263);
    var_265 = wp::mul(var_262, var_264);
    wp::assign_inplace(var_171, var_248, var_265);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_267 = wp::extract(var_255, var_266);
    var_269 = wp::extract(var_255, var_268);
    var_270 = wp::mul(var_267, var_269);
    wp::assign_inplace(var_178, var_248, var_270);
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_273 = wp::extract(var_156, var_271, var_272);
    var_274 = wp::extract(var_143, var_273);
    var_276 = wp::extract(var_156, var_271, var_275);
    var_277 = wp::extract(var_143, var_276);
    var_278 = wp::sub(var_274, var_277);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_271, var_278);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_280 = wp::extract(var_278, var_279);
    var_282 = wp::extract(var_278, var_281);
    var_283 = wp::mul(var_280, var_282);
    wp::assign_inplace(var_164, var_271, var_283);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_285 = wp::extract(var_278, var_284);
    var_287 = wp::extract(var_278, var_286);
    var_288 = wp::mul(var_285, var_287);
    wp::assign_inplace(var_171, var_271, var_288);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_290 = wp::extract(var_278, var_289);
    var_292 = wp::extract(var_278, var_291);
    var_293 = wp::mul(var_290, var_292);
    wp::assign_inplace(var_178, var_271, var_293);
    // ref_e = vertex[edges[e, 0]] - vertex[edges[e, 1]]                                      <L 1261>
    var_296 = wp::extract(var_156, var_294, var_295);
    var_297 = wp::extract(var_143, var_296);
    var_299 = wp::extract(var_156, var_294, var_298);
    var_300 = wp::extract(var_143, var_299);
    var_301 = wp::sub(var_297, var_300);
    // reference[e] = ref_e                                                                   <L 1262>
    wp::assign_inplace(var_157, var_294, var_301);
    // sq0[e] = ref_e[0] * ref_e[0]                                                           <L 1263>
    var_303 = wp::extract(var_301, var_302);
    var_305 = wp::extract(var_301, var_304);
    var_306 = wp::mul(var_303, var_305);
    wp::assign_inplace(var_164, var_294, var_306);
    // sq1[e] = ref_e[1] * ref_e[1]                                                           <L 1264>
    var_308 = wp::extract(var_301, var_307);
    var_310 = wp::extract(var_301, var_309);
    var_311 = wp::mul(var_308, var_310);
    wp::assign_inplace(var_171, var_294, var_311);
    // sq2[e] = ref_e[2] * ref_e[2]                                                           <L 1265>
    var_313 = wp::extract(var_301, var_312);
    var_315 = wp::extract(var_301, var_314);
    var_316 = wp::mul(var_313, var_315);
    wp::assign_inplace(var_178, var_294, var_316);
    // prod0 = metric * sq0                                                                   <L 1267>
    var_317 = wp::mul(var_142, var_164);
    // prod1 = metric * sq1                                                                   <L 1268>
    var_318 = wp::mul(var_142, var_171);
    // prod2 = metric * sq2                                                                   <L 1269>
    var_319 = wp::mul(var_142, var_178);
    // geo = wp.vec3(wp.dot(tension, sq0), wp.dot(tension, sq1), wp.dot(tension, sq2))        <L 1270>
    var_320 = wp::dot(var_141, var_164);
    var_321 = wp::dot(var_141, var_171);
    var_322 = wp::dot(var_141, var_178);
    var_323 = wp::vec_t<3, wp::float32>(var_320, var_321, var_322);
    // J = sigma[0] * sigma[1] * sigma[2]                                                     <L 1272>
    var_325 = wp::extract(var_124, var_324);
    var_327 = wp::extract(var_124, var_326);
    var_328 = wp::mul(var_325, var_327);
    var_330 = wp::extract(var_124, var_329);
    var_331 = wp::mul(var_328, var_330);
    // volume_stiffness = 2.0 * k22                                                           <L 1273>
    var_333 = wp::mul(var_332, var_98);
    // pressure = volume_stiffness * (J - 1.0)                                                <L 1274>
    var_335 = wp::sub(var_331, var_334);
    var_336 = wp::mul(var_333, var_335);
    // cof = wp.vec3(sigma[1] * sigma[2], sigma[0] * sigma[2], sigma[0] * sigma[1])           <L 1275>
    var_338 = wp::extract(var_124, var_337);
    var_340 = wp::extract(var_124, var_339);
    var_341 = wp::mul(var_338, var_340);
    var_343 = wp::extract(var_124, var_342);
    var_345 = wp::extract(var_124, var_344);
    var_346 = wp::mul(var_343, var_345);
    var_348 = wp::extract(var_124, var_347);
    var_350 = wp::extract(var_124, var_349);
    var_351 = wp::mul(var_348, var_350);
    var_352 = wp::vec_t<3, wp::float32>(var_341, var_346, var_351);
    // a00 = 2.0 * sigma[0] * sigma[0] * wp.dot(sq0, prod0) + volume_stiffness * cof[0] * cof[0] + geo[0]       <L 1277>
    var_355 = wp::extract(var_124, var_354);
    var_356 = wp::mul(var_353, var_355);
    var_358 = wp::extract(var_124, var_357);
    var_359 = wp::mul(var_356, var_358);
    var_360 = wp::dot(var_164, var_317);
    var_361 = wp::mul(var_359, var_360);
    var_363 = wp::extract(var_352, var_362);
    var_364 = wp::mul(var_333, var_363);
    var_366 = wp::extract(var_352, var_365);
    var_367 = wp::mul(var_364, var_366);
    var_368 = wp::add(var_361, var_367);
    var_370 = wp::extract(var_323, var_369);
    var_371 = wp::add(var_368, var_370);
    // a11 = 2.0 * sigma[1] * sigma[1] * wp.dot(sq1, prod1) + volume_stiffness * cof[1] * cof[1] + geo[1]       <L 1278>
    var_374 = wp::extract(var_124, var_373);
    var_375 = wp::mul(var_372, var_374);
    var_377 = wp::extract(var_124, var_376);
    var_378 = wp::mul(var_375, var_377);
    var_379 = wp::dot(var_171, var_318);
    var_380 = wp::mul(var_378, var_379);
    var_382 = wp::extract(var_352, var_381);
    var_383 = wp::mul(var_333, var_382);
    var_385 = wp::extract(var_352, var_384);
    var_386 = wp::mul(var_383, var_385);
    var_387 = wp::add(var_380, var_386);
    var_389 = wp::extract(var_323, var_388);
    var_390 = wp::add(var_387, var_389);
    // a22 = 2.0 * sigma[2] * sigma[2] * wp.dot(sq2, prod2) + volume_stiffness * cof[2] * cof[2] + geo[2]       <L 1279>
    var_393 = wp::extract(var_124, var_392);
    var_394 = wp::mul(var_391, var_393);
    var_396 = wp::extract(var_124, var_395);
    var_397 = wp::mul(var_394, var_396);
    var_398 = wp::dot(var_178, var_319);
    var_399 = wp::mul(var_397, var_398);
    var_401 = wp::extract(var_352, var_400);
    var_402 = wp::mul(var_333, var_401);
    var_404 = wp::extract(var_352, var_403);
    var_405 = wp::mul(var_402, var_404);
    var_406 = wp::add(var_399, var_405);
    var_408 = wp::extract(var_323, var_407);
    var_409 = wp::add(var_406, var_408);
    // a01 = 2.0 * sigma[0] * sigma[1] * wp.dot(sq0, prod1) + volume_stiffness * cof[0] * cof[1] + pressure * sigma[2]       <L 1280>
    var_412 = wp::extract(var_124, var_411);
    var_413 = wp::mul(var_410, var_412);
    var_415 = wp::extract(var_124, var_414);
    var_416 = wp::mul(var_413, var_415);
    var_417 = wp::dot(var_164, var_318);
    var_418 = wp::mul(var_416, var_417);
    var_420 = wp::extract(var_352, var_419);
    var_421 = wp::mul(var_333, var_420);
    var_423 = wp::extract(var_352, var_422);
    var_424 = wp::mul(var_421, var_423);
    var_425 = wp::add(var_418, var_424);
    var_427 = wp::extract(var_124, var_426);
    var_428 = wp::mul(var_336, var_427);
    var_429 = wp::add(var_425, var_428);
    // a02 = 2.0 * sigma[0] * sigma[2] * wp.dot(sq0, prod2) + volume_stiffness * cof[0] * cof[2] + pressure * sigma[1]       <L 1281>
    var_432 = wp::extract(var_124, var_431);
    var_433 = wp::mul(var_430, var_432);
    var_435 = wp::extract(var_124, var_434);
    var_436 = wp::mul(var_433, var_435);
    var_437 = wp::dot(var_164, var_319);
    var_438 = wp::mul(var_436, var_437);
    var_440 = wp::extract(var_352, var_439);
    var_441 = wp::mul(var_333, var_440);
    var_443 = wp::extract(var_352, var_442);
    var_444 = wp::mul(var_441, var_443);
    var_445 = wp::add(var_438, var_444);
    var_447 = wp::extract(var_124, var_446);
    var_448 = wp::mul(var_336, var_447);
    var_449 = wp::add(var_445, var_448);
    // a12 = 2.0 * sigma[1] * sigma[2] * wp.dot(sq1, prod2) + volume_stiffness * cof[1] * cof[2] + pressure * sigma[0]       <L 1282>
    var_452 = wp::extract(var_124, var_451);
    var_453 = wp::mul(var_450, var_452);
    var_455 = wp::extract(var_124, var_454);
    var_456 = wp::mul(var_453, var_455);
    var_457 = wp::dot(var_171, var_319);
    var_458 = wp::mul(var_456, var_457);
    var_460 = wp::extract(var_352, var_459);
    var_461 = wp::mul(var_333, var_460);
    var_463 = wp::extract(var_352, var_462);
    var_464 = wp::mul(var_461, var_463);
    var_465 = wp::add(var_458, var_464);
    var_467 = wp::extract(var_124, var_466);
    var_468 = wp::mul(var_336, var_467);
    var_469 = wp::add(var_465, var_468);
    // A = wp.mat33(a00, a01, a02, a01, a11, a12, a02, a12, a22)                              <L 1283>
    var_470 = wp::mat_t<3, 3, wp::float32>(var_371, var_429, var_449, var_429, var_390, var_469, var_449, var_469, var_409);
    // if _snh_positive3(A):                                                                  <L 1285>
    var_471 = _snh_positive3_0(var_470);
    if (var_471) {
        // stretch = A                                                                        <L 1286>
        var_472 = wp::copy(var_470);
    }
    if (!var_471) {
        // eig, Q = _snh_eigen3(A)                                                            <L 1288>
        _snh_eigen3_0(var_470, var_473, var_474);
        // ev0 = wp.max(0.0, eig[0])                                                          <L 1289>
        var_477 = wp::extract(var_473, var_476);
        var_478 = wp::max(var_475, var_477);
        // ev1 = wp.max(0.0, eig[1])                                                          <L 1290>
        var_481 = wp::extract(var_473, var_480);
        var_482 = wp::max(var_479, var_481);
        // ev2 = wp.max(0.0, eig[2])                                                          <L 1291>
        var_485 = wp::extract(var_473, var_484);
        var_486 = wp::max(var_483, var_485);
        // QD = wp.mat33(                                                                     <L 1292>
        // Q[0, 0] * ev0,                                                                     <L 1293>
        var_489 = wp::extract(var_474, var_487, var_488);
        var_490 = wp::mul(var_489, var_478);
        // Q[0, 1] * ev1,                                                                     <L 1294>
        var_493 = wp::extract(var_474, var_491, var_492);
        var_494 = wp::mul(var_493, var_482);
        // Q[0, 2] * ev2,                                                                     <L 1295>
        var_497 = wp::extract(var_474, var_495, var_496);
        var_498 = wp::mul(var_497, var_486);
        // Q[1, 0] * ev0,                                                                     <L 1296>
        var_501 = wp::extract(var_474, var_499, var_500);
        var_502 = wp::mul(var_501, var_478);
        // Q[1, 1] * ev1,                                                                     <L 1297>
        var_505 = wp::extract(var_474, var_503, var_504);
        var_506 = wp::mul(var_505, var_482);
        // Q[1, 2] * ev2,                                                                     <L 1298>
        var_509 = wp::extract(var_474, var_507, var_508);
        var_510 = wp::mul(var_509, var_486);
        // Q[2, 0] * ev0,                                                                     <L 1299>
        var_513 = wp::extract(var_474, var_511, var_512);
        var_514 = wp::mul(var_513, var_478);
        // Q[2, 1] * ev1,                                                                     <L 1300>
        var_517 = wp::extract(var_474, var_515, var_516);
        var_518 = wp::mul(var_517, var_482);
        // Q[2, 2] * ev2,                                                                     <L 1301>
        var_521 = wp::extract(var_474, var_519, var_520);
        var_522 = wp::mul(var_521, var_486);
        var_523 = wp::mat_t<3, 3, wp::float32>(var_490, var_494, var_498, var_502, var_506, var_510, var_514, var_518, var_522);
        // stretch = QD * wp.transpose(Q)                                                     <L 1303>
        var_524 = wp::transpose(var_474);
        var_525 = wp::mul(var_523, var_524);
    }
    var_526 = wp::where(var_471, var_472, var_525);
    // symmetric = wp.vec3(0.0, 0.0, 0.0)                                                     <L 1305>
    var_530 = wp::vec_t<3, wp::float32>(var_527, var_528, var_529);
    // skew = wp.vec3(0.0, 0.0, 0.0)                                                          <L 1306>
    var_534 = wp::vec_t<3, wp::float32>(var_531, var_532, var_533);
    // pair_i = wp.vec3i(0, 0, 1)                                                             <L 1307>
    var_538 = wp::vec_t<3, wp::int32>(var_535, var_536, var_537);
    // pair_j = wp.vec3i(1, 2, 2)                                                             <L 1308>
    var_542 = wp::vec_t<3, wp::int32>(var_539, var_540, var_541);
    // for mode in range(3):                                                                  <L 1309>
    // ii = pair_i[mode]                                                                      <L 1310>
    var_544 = wp::extract(var_538, var_543);
    // jj = pair_j[mode]                                                                      <L 1311>
    var_545 = wp::extract(var_542, var_543);
    // direction = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                         <L 1312>
    var_552 = wp::vec_t<6, wp::float32>({var_546, var_547, var_548, var_549, var_550, var_551});
    // for e in range(6):                                                                     <L 1313>
    // direction[e] = reference[e, ii] * reference[e, jj]                                     <L 1314>
    var_554 = wp::extract(var_157, var_553, var_544);
    var_555 = wp::extract(var_157, var_553, var_545);
    var_556 = wp::mul(var_554, var_555);
    wp::assign_inplace(var_552, var_553, var_556);
    var_558 = wp::extract(var_157, var_557, var_544);
    var_559 = wp::extract(var_157, var_557, var_545);
    var_560 = wp::mul(var_558, var_559);
    wp::assign_inplace(var_552, var_557, var_560);
    var_562 = wp::extract(var_157, var_561, var_544);
    var_563 = wp::extract(var_157, var_561, var_545);
    var_564 = wp::mul(var_562, var_563);
    wp::assign_inplace(var_552, var_561, var_564);
    var_566 = wp::extract(var_157, var_565, var_544);
    var_567 = wp::extract(var_157, var_565, var_545);
    var_568 = wp::mul(var_566, var_567);
    wp::assign_inplace(var_552, var_565, var_568);
    var_570 = wp::extract(var_157, var_569, var_544);
    var_571 = wp::extract(var_157, var_569, var_545);
    var_572 = wp::mul(var_570, var_571);
    wp::assign_inplace(var_552, var_569, var_572);
    var_574 = wp::extract(var_157, var_573, var_544);
    var_575 = wp::extract(var_157, var_573, var_545);
    var_576 = wp::mul(var_574, var_575);
    wp::assign_inplace(var_552, var_573, var_576);
    // response = metric * direction                                                          <L 1315>
    var_577 = wp::mul(var_142, var_552);
    // material = wp.dot(direction, response)                                                 <L 1316>
    var_578 = wp::dot(var_552, var_577);
    // geometric = 0.5 * (geo[ii] + geo[jj])                                                  <L 1317>
    var_580 = wp::extract(var_323, var_544);
    var_581 = wp::extract(var_323, var_545);
    var_582 = wp::add(var_580, var_581);
    var_583 = wp::mul(var_579, var_582);
    // sum_s = sigma[ii] + sigma[jj]                                                          <L 1318>
    var_584 = wp::extract(var_124, var_544);
    var_585 = wp::extract(var_124, var_545);
    var_586 = wp::add(var_584, var_585);
    // diff_s = sigma[ii] - sigma[jj]                                                         <L 1319>
    var_587 = wp::extract(var_124, var_544);
    var_588 = wp::extract(var_124, var_545);
    var_589 = wp::sub(var_587, var_588);
    // cross_s = pressure * sigma[3 - ii - jj]                                                <L 1320>
    var_591 = wp::sub(var_590, var_544);
    var_592 = wp::sub(var_591, var_545);
    var_593 = wp::extract(var_124, var_592);
    var_594 = wp::mul(var_336, var_593);
    // symmetric[mode] = wp.max(0.0, sum_s * sum_s * material + geometric - cross_s)          <L 1321>
    var_596 = wp::mul(var_586, var_586);
    var_597 = wp::mul(var_596, var_578);
    var_598 = wp::add(var_597, var_583);
    var_599 = wp::sub(var_598, var_594);
    var_600 = wp::max(var_595, var_599);
    wp::assign_inplace(var_530, var_543, var_600);
    // skew[mode] = wp.max(0.0, diff_s * diff_s * material + geometric + cross_s)             <L 1322>
    var_602 = wp::mul(var_589, var_589);
    var_603 = wp::mul(var_602, var_578);
    var_604 = wp::add(var_603, var_583);
    var_605 = wp::add(var_604, var_594);
    var_606 = wp::max(var_601, var_605);
    wp::assign_inplace(var_534, var_543, var_606);
    // ii = pair_i[mode]                                                                      <L 1310>
    var_608 = wp::extract(var_538, var_607);
    // jj = pair_j[mode]                                                                      <L 1311>
    var_609 = wp::extract(var_542, var_607);
    // direction = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                         <L 1312>
    var_616 = wp::vec_t<6, wp::float32>({var_610, var_611, var_612, var_613, var_614, var_615});
    // for e in range(6):                                                                     <L 1313>
    // direction[e] = reference[e, ii] * reference[e, jj]                                     <L 1314>
    var_618 = wp::extract(var_157, var_617, var_608);
    var_619 = wp::extract(var_157, var_617, var_609);
    var_620 = wp::mul(var_618, var_619);
    wp::assign_inplace(var_616, var_617, var_620);
    var_622 = wp::extract(var_157, var_621, var_608);
    var_623 = wp::extract(var_157, var_621, var_609);
    var_624 = wp::mul(var_622, var_623);
    wp::assign_inplace(var_616, var_621, var_624);
    var_626 = wp::extract(var_157, var_625, var_608);
    var_627 = wp::extract(var_157, var_625, var_609);
    var_628 = wp::mul(var_626, var_627);
    wp::assign_inplace(var_616, var_625, var_628);
    var_630 = wp::extract(var_157, var_629, var_608);
    var_631 = wp::extract(var_157, var_629, var_609);
    var_632 = wp::mul(var_630, var_631);
    wp::assign_inplace(var_616, var_629, var_632);
    var_634 = wp::extract(var_157, var_633, var_608);
    var_635 = wp::extract(var_157, var_633, var_609);
    var_636 = wp::mul(var_634, var_635);
    wp::assign_inplace(var_616, var_633, var_636);
    var_638 = wp::extract(var_157, var_637, var_608);
    var_639 = wp::extract(var_157, var_637, var_609);
    var_640 = wp::mul(var_638, var_639);
    wp::assign_inplace(var_616, var_637, var_640);
    // response = metric * direction                                                          <L 1315>
    var_641 = wp::mul(var_142, var_616);
    // material = wp.dot(direction, response)                                                 <L 1316>
    var_642 = wp::dot(var_616, var_641);
    // geometric = 0.5 * (geo[ii] + geo[jj])                                                  <L 1317>
    var_644 = wp::extract(var_323, var_608);
    var_645 = wp::extract(var_323, var_609);
    var_646 = wp::add(var_644, var_645);
    var_647 = wp::mul(var_643, var_646);
    // sum_s = sigma[ii] + sigma[jj]                                                          <L 1318>
    var_648 = wp::extract(var_124, var_608);
    var_649 = wp::extract(var_124, var_609);
    var_650 = wp::add(var_648, var_649);
    // diff_s = sigma[ii] - sigma[jj]                                                         <L 1319>
    var_651 = wp::extract(var_124, var_608);
    var_652 = wp::extract(var_124, var_609);
    var_653 = wp::sub(var_651, var_652);
    // cross_s = pressure * sigma[3 - ii - jj]                                                <L 1320>
    var_655 = wp::sub(var_654, var_608);
    var_656 = wp::sub(var_655, var_609);
    var_657 = wp::extract(var_124, var_656);
    var_658 = wp::mul(var_336, var_657);
    // symmetric[mode] = wp.max(0.0, sum_s * sum_s * material + geometric - cross_s)          <L 1321>
    var_660 = wp::mul(var_650, var_650);
    var_661 = wp::mul(var_660, var_642);
    var_662 = wp::add(var_661, var_647);
    var_663 = wp::sub(var_662, var_658);
    var_664 = wp::max(var_659, var_663);
    wp::assign_inplace(var_530, var_607, var_664);
    // skew[mode] = wp.max(0.0, diff_s * diff_s * material + geometric + cross_s)             <L 1322>
    var_666 = wp::mul(var_653, var_653);
    var_667 = wp::mul(var_666, var_642);
    var_668 = wp::add(var_667, var_647);
    var_669 = wp::add(var_668, var_658);
    var_670 = wp::max(var_665, var_669);
    wp::assign_inplace(var_534, var_607, var_670);
    // ii = pair_i[mode]                                                                      <L 1310>
    var_672 = wp::extract(var_538, var_671);
    // jj = pair_j[mode]                                                                      <L 1311>
    var_673 = wp::extract(var_542, var_671);
    // direction = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                         <L 1312>
    var_680 = wp::vec_t<6, wp::float32>({var_674, var_675, var_676, var_677, var_678, var_679});
    // for e in range(6):                                                                     <L 1313>
    // direction[e] = reference[e, ii] * reference[e, jj]                                     <L 1314>
    var_682 = wp::extract(var_157, var_681, var_672);
    var_683 = wp::extract(var_157, var_681, var_673);
    var_684 = wp::mul(var_682, var_683);
    wp::assign_inplace(var_680, var_681, var_684);
    var_686 = wp::extract(var_157, var_685, var_672);
    var_687 = wp::extract(var_157, var_685, var_673);
    var_688 = wp::mul(var_686, var_687);
    wp::assign_inplace(var_680, var_685, var_688);
    var_690 = wp::extract(var_157, var_689, var_672);
    var_691 = wp::extract(var_157, var_689, var_673);
    var_692 = wp::mul(var_690, var_691);
    wp::assign_inplace(var_680, var_689, var_692);
    var_694 = wp::extract(var_157, var_693, var_672);
    var_695 = wp::extract(var_157, var_693, var_673);
    var_696 = wp::mul(var_694, var_695);
    wp::assign_inplace(var_680, var_693, var_696);
    var_698 = wp::extract(var_157, var_697, var_672);
    var_699 = wp::extract(var_157, var_697, var_673);
    var_700 = wp::mul(var_698, var_699);
    wp::assign_inplace(var_680, var_697, var_700);
    var_702 = wp::extract(var_157, var_701, var_672);
    var_703 = wp::extract(var_157, var_701, var_673);
    var_704 = wp::mul(var_702, var_703);
    wp::assign_inplace(var_680, var_701, var_704);
    // response = metric * direction                                                          <L 1315>
    var_705 = wp::mul(var_142, var_680);
    // material = wp.dot(direction, response)                                                 <L 1316>
    var_706 = wp::dot(var_680, var_705);
    // geometric = 0.5 * (geo[ii] + geo[jj])                                                  <L 1317>
    var_708 = wp::extract(var_323, var_672);
    var_709 = wp::extract(var_323, var_673);
    var_710 = wp::add(var_708, var_709);
    var_711 = wp::mul(var_707, var_710);
    // sum_s = sigma[ii] + sigma[jj]                                                          <L 1318>
    var_712 = wp::extract(var_124, var_672);
    var_713 = wp::extract(var_124, var_673);
    var_714 = wp::add(var_712, var_713);
    // diff_s = sigma[ii] - sigma[jj]                                                         <L 1319>
    var_715 = wp::extract(var_124, var_672);
    var_716 = wp::extract(var_124, var_673);
    var_717 = wp::sub(var_715, var_716);
    // cross_s = pressure * sigma[3 - ii - jj]                                                <L 1320>
    var_719 = wp::sub(var_718, var_672);
    var_720 = wp::sub(var_719, var_673);
    var_721 = wp::extract(var_124, var_720);
    var_722 = wp::mul(var_336, var_721);
    // symmetric[mode] = wp.max(0.0, sum_s * sum_s * material + geometric - cross_s)          <L 1321>
    var_724 = wp::mul(var_714, var_714);
    var_725 = wp::mul(var_724, var_706);
    var_726 = wp::add(var_725, var_711);
    var_727 = wp::sub(var_726, var_722);
    var_728 = wp::max(var_723, var_727);
    wp::assign_inplace(var_530, var_671, var_728);
    // skew[mode] = wp.max(0.0, diff_s * diff_s * material + geometric + cross_s)             <L 1322>
    var_730 = wp::mul(var_717, var_717);
    var_731 = wp::mul(var_730, var_706);
    var_732 = wp::add(var_731, var_711);
    var_733 = wp::add(var_732, var_722);
    var_734 = wp::max(var_729, var_733);
    wp::assign_inplace(var_534, var_671, var_734);
    // active = (                                                                             <L 1324>
    // stretch[0, 0] > 0.0                                                                    <L 1325>
    var_738 = wp::extract(var_526, var_736, var_737);
    var_740 = (var_738 > var_739);
    var_735 = var_740;
    if (!var_735) {
        // or stretch[1, 1] > 0.0                                                             <L 1326>
        var_743 = wp::extract(var_526, var_741, var_742);
        var_745 = (var_743 > var_744);
        var_735 = var_735 || var_745;
    }
    if (!var_735) {
        // or stretch[2, 2] > 0.0                                                             <L 1327>
        var_748 = wp::extract(var_526, var_746, var_747);
        var_750 = (var_748 > var_749);
        var_735 = var_735 || var_750;
    }
    if (!var_735) {
        // or symmetric[0] > 0.0                                                              <L 1328>
        var_752 = wp::extract(var_530, var_751);
        var_754 = (var_752 > var_753);
        var_735 = var_735 || var_754;
    }
    if (!var_735) {
        // or symmetric[1] > 0.0                                                              <L 1329>
        var_756 = wp::extract(var_530, var_755);
        var_758 = (var_756 > var_757);
        var_735 = var_735 || var_758;
    }
    if (!var_735) {
        // or symmetric[2] > 0.0                                                              <L 1330>
        var_760 = wp::extract(var_530, var_759);
        var_762 = (var_760 > var_761);
        var_735 = var_735 || var_762;
    }
    if (!var_735) {
        // or skew[0] > 0.0                                                                   <L 1331>
        var_764 = wp::extract(var_534, var_763);
        var_766 = (var_764 > var_765);
        var_735 = var_735 || var_766;
    }
    if (!var_735) {
        // or skew[1] > 0.0                                                                   <L 1332>
        var_768 = wp::extract(var_534, var_767);
        var_770 = (var_768 > var_769);
        var_735 = var_735 || var_770;
    }
    if (!var_735) {
        // or skew[2] > 0.0                                                                   <L 1333>
        var_772 = wp::extract(var_534, var_771);
        var_774 = (var_772 > var_773);
        var_735 = var_735 || var_774;
    }
    // return active, rotation, gradient, stretch, symmetric, skew                            <L 1335>
    ret_0 = var_735;
    ret_1 = var_123;
    ret_2 = var_133;
    ret_3 = var_526;
    ret_4 = var_530;
    ret_5 = var_534;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1141
static CUDA_CALLABLE void stretch_stiffness_0(
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::int32 var_worldid,
    wp::int32 var_stiffness_adr,
    wp::int32 var_elemedge_adr,
    wp::int32 var_edge_adr,
    wp::int32 var_nedge,
    wp::mat_t<6, 6, wp::float32> & ret_0,
    wp::vec_t<6, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<6, wp::float32> var_0;
    wp::mat_t<6, 6, wp::float32> var_1;
    wp::vec_t<6, wp::float32> var_2;
    wp::range_t var_3;
    wp::int32 var_4;
    wp::float32 var_5;
    const wp::float32 var_6 = 0.0;
    wp::float32 var_7;
    //---------
    // forward
    // def stretch_stiffness(                                                                 <L 1142>
    // elongation = stretch_elongation(flex_elemedge, flexedge_length0, flexedge_length_in, worldid, elemedge_adr, edge_adr, nedge)       <L 1156>
    var_0 = stretch_elongation_0(var_flex_elemedge, var_flexedge_length0, var_flexedge_length_in, var_worldid, var_elemedge_adr, var_edge_adr, var_nedge);
    // metric, tension = stretch_elasticity(flex_stiffness, stiffness_adr, elongation, nedge)       <L 1157>
    stretch_elasticity_0(var_flex_stiffness, var_stiffness_adr, var_0, var_nedge, var_1, var_2);
    // for e in range(nedge):                                                                 <L 1158>
    var_3 = wp::range(var_nedge);
    start_for_0:;
        if (iter_cmp(var_3) == 0) goto end_for_0;
        var_4 = wp::iter_next(var_3);
        // tension[e] = wp.max(tension[e], 0.0)                                               <L 1159>
        var_5 = wp::extract(var_2, var_4);
        var_7 = wp::max(var_5, var_6);
        wp::assign_inplace(var_2, var_4, var_7);
        goto start_for_0;
    end_for_0:;
    // return metric, tension                                                                 <L 1160>
    ret_0 = var_1;
    ret_1 = var_2;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1338
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> snh_projected_block_0(
    wp::mat_t<3, 3, wp::float32> var_rotation,
    wp::mat_t<4, 3, wp::float32> var_gradient,
    wp::mat_t<3, 3, wp::float32> var_stretch,
    wp::vec_t<3, wp::float32> var_symmetric,
    wp::vec_t<3, wp::float32> var_skew,
    wp::int32 var_i,
    wp::int32 var_j)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    const wp::float32 var_2 = 0.5;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    const wp::float32 var_9 = 0.5;
    const wp::int32 var_10 = 0;
    wp::float32 var_11;
    const wp::int32 var_12 = 0;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::float32 var_16 = 0.5;
    const wp::int32 var_17 = 1;
    wp::float32 var_18;
    const wp::int32 var_19 = 1;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.5;
    const wp::int32 var_24 = 1;
    wp::float32 var_25;
    const wp::int32 var_26 = 1;
    wp::float32 var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    const wp::float32 var_30 = 0.5;
    const wp::int32 var_31 = 2;
    wp::float32 var_32;
    const wp::int32 var_33 = 2;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    const wp::float32 var_37 = 0.5;
    const wp::int32 var_38 = 2;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::int32 var_44 = 0;
    const wp::int32 var_45 = 0;
    wp::float32 var_46;
    const wp::int32 var_47 = 0;
    wp::float32 var_48;
    wp::float32 var_49;
    const wp::int32 var_50 = 0;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::int32 var_56 = 1;
    wp::float32 var_57;
    wp::float32 var_58;
    wp::float32 var_59;
    const wp::int32 var_60 = 2;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 2;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::float32 var_66;
    const wp::int32 var_67 = 0;
    const wp::int32 var_68 = 1;
    wp::float32 var_69;
    const wp::int32 var_70 = 0;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::int32 var_73 = 1;
    wp::float32 var_74;
    wp::float32 var_75;
    const wp::int32 var_76 = 1;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::int32 var_79 = 0;
    wp::float32 var_80;
    wp::float32 var_81;
    wp::float32 var_82;
    const wp::int32 var_83 = 0;
    const wp::int32 var_84 = 2;
    wp::float32 var_85;
    const wp::int32 var_86 = 0;
    wp::float32 var_87;
    wp::float32 var_88;
    const wp::int32 var_89 = 2;
    wp::float32 var_90;
    wp::float32 var_91;
    const wp::int32 var_92 = 2;
    wp::float32 var_93;
    wp::float32 var_94;
    const wp::int32 var_95 = 0;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    const wp::int32 var_99 = 1;
    const wp::int32 var_100 = 0;
    wp::float32 var_101;
    const wp::int32 var_102 = 1;
    wp::float32 var_103;
    wp::float32 var_104;
    const wp::int32 var_105 = 0;
    wp::float32 var_106;
    wp::float32 var_107;
    const wp::int32 var_108 = 0;
    wp::float32 var_109;
    wp::float32 var_110;
    const wp::int32 var_111 = 1;
    wp::float32 var_112;
    wp::float32 var_113;
    wp::float32 var_114;
    const wp::int32 var_115 = 1;
    const wp::int32 var_116 = 1;
    wp::float32 var_117;
    const wp::int32 var_118 = 1;
    wp::float32 var_119;
    wp::float32 var_120;
    const wp::int32 var_121 = 1;
    wp::float32 var_122;
    wp::float32 var_123;
    const wp::int32 var_124 = 0;
    wp::float32 var_125;
    wp::float32 var_126;
    const wp::int32 var_127 = 0;
    wp::float32 var_128;
    wp::float32 var_129;
    wp::float32 var_130;
    const wp::int32 var_131 = 2;
    wp::float32 var_132;
    wp::float32 var_133;
    const wp::int32 var_134 = 2;
    wp::float32 var_135;
    wp::float32 var_136;
    wp::float32 var_137;
    const wp::int32 var_138 = 1;
    const wp::int32 var_139 = 2;
    wp::float32 var_140;
    const wp::int32 var_141 = 1;
    wp::float32 var_142;
    wp::float32 var_143;
    const wp::int32 var_144 = 2;
    wp::float32 var_145;
    wp::float32 var_146;
    const wp::int32 var_147 = 2;
    wp::float32 var_148;
    wp::float32 var_149;
    const wp::int32 var_150 = 1;
    wp::float32 var_151;
    wp::float32 var_152;
    wp::float32 var_153;
    const wp::int32 var_154 = 2;
    const wp::int32 var_155 = 0;
    wp::float32 var_156;
    const wp::int32 var_157 = 2;
    wp::float32 var_158;
    wp::float32 var_159;
    const wp::int32 var_160 = 0;
    wp::float32 var_161;
    wp::float32 var_162;
    const wp::int32 var_163 = 0;
    wp::float32 var_164;
    wp::float32 var_165;
    const wp::int32 var_166 = 2;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    const wp::int32 var_170 = 2;
    const wp::int32 var_171 = 1;
    wp::float32 var_172;
    const wp::int32 var_173 = 2;
    wp::float32 var_174;
    wp::float32 var_175;
    const wp::int32 var_176 = 1;
    wp::float32 var_177;
    wp::float32 var_178;
    const wp::int32 var_179 = 1;
    wp::float32 var_180;
    wp::float32 var_181;
    const wp::int32 var_182 = 2;
    wp::float32 var_183;
    wp::float32 var_184;
    wp::float32 var_185;
    const wp::int32 var_186 = 2;
    const wp::int32 var_187 = 2;
    wp::float32 var_188;
    const wp::int32 var_189 = 2;
    wp::float32 var_190;
    wp::float32 var_191;
    const wp::int32 var_192 = 2;
    wp::float32 var_193;
    wp::float32 var_194;
    const wp::int32 var_195 = 0;
    wp::float32 var_196;
    wp::float32 var_197;
    const wp::int32 var_198 = 0;
    wp::float32 var_199;
    wp::float32 var_200;
    wp::float32 var_201;
    const wp::int32 var_202 = 1;
    wp::float32 var_203;
    wp::float32 var_204;
    const wp::int32 var_205 = 1;
    wp::float32 var_206;
    wp::float32 var_207;
    wp::float32 var_208;
    wp::mat_t<3, 3, wp::float32> var_209;
    wp::mat_t<3, 3, wp::float32> var_210;
    wp::mat_t<3, 3, wp::float32> var_211;
    wp::mat_t<3, 3, wp::float32> var_212;
    //---------
    // forward
    // def snh_projected_block(                                                               <L 1339>
    // a = gradient[i]                                                                        <L 1349>
    var_0 = wp::extract(var_gradient, var_i);
    // b = gradient[j]                                                                        <L 1350>
    var_1 = wp::extract(var_gradient, var_j);
    // s0 = 0.5 * (symmetric[0] + skew[0])                                                    <L 1351>
    var_4 = wp::extract(var_symmetric, var_3);
    var_6 = wp::extract(var_skew, var_5);
    var_7 = wp::add(var_4, var_6);
    var_8 = wp::mul(var_2, var_7);
    // d0 = 0.5 * (symmetric[0] - skew[0])                                                    <L 1352>
    var_11 = wp::extract(var_symmetric, var_10);
    var_13 = wp::extract(var_skew, var_12);
    var_14 = wp::sub(var_11, var_13);
    var_15 = wp::mul(var_9, var_14);
    // s1 = 0.5 * (symmetric[1] + skew[1])                                                    <L 1353>
    var_18 = wp::extract(var_symmetric, var_17);
    var_20 = wp::extract(var_skew, var_19);
    var_21 = wp::add(var_18, var_20);
    var_22 = wp::mul(var_16, var_21);
    // d1 = 0.5 * (symmetric[1] - skew[1])                                                    <L 1354>
    var_25 = wp::extract(var_symmetric, var_24);
    var_27 = wp::extract(var_skew, var_26);
    var_28 = wp::sub(var_25, var_27);
    var_29 = wp::mul(var_23, var_28);
    // s2 = 0.5 * (symmetric[2] + skew[2])                                                    <L 1355>
    var_32 = wp::extract(var_symmetric, var_31);
    var_34 = wp::extract(var_skew, var_33);
    var_35 = wp::add(var_32, var_34);
    var_36 = wp::mul(var_30, var_35);
    // d2 = 0.5 * (symmetric[2] - skew[2])                                                    <L 1356>
    var_39 = wp::extract(var_symmetric, var_38);
    var_41 = wp::extract(var_skew, var_40);
    var_42 = wp::sub(var_39, var_41);
    var_43 = wp::mul(var_37, var_42);
    // local = wp.mat33(                                                                      <L 1358>
    // stretch[0, 0] * a[0] * b[0] + s0 * a[1] * b[1] + s1 * a[2] * b[2],                     <L 1359>
    var_46 = wp::extract(var_stretch, var_44, var_45);
    var_48 = wp::extract(var_0, var_47);
    var_49 = wp::mul(var_46, var_48);
    var_51 = wp::extract(var_1, var_50);
    var_52 = wp::mul(var_49, var_51);
    var_54 = wp::extract(var_0, var_53);
    var_55 = wp::mul(var_8, var_54);
    var_57 = wp::extract(var_1, var_56);
    var_58 = wp::mul(var_55, var_57);
    var_59 = wp::add(var_52, var_58);
    var_61 = wp::extract(var_0, var_60);
    var_62 = wp::mul(var_22, var_61);
    var_64 = wp::extract(var_1, var_63);
    var_65 = wp::mul(var_62, var_64);
    var_66 = wp::add(var_59, var_65);
    // stretch[0, 1] * a[0] * b[1] + d0 * a[1] * b[0],                                        <L 1360>
    var_69 = wp::extract(var_stretch, var_67, var_68);
    var_71 = wp::extract(var_0, var_70);
    var_72 = wp::mul(var_69, var_71);
    var_74 = wp::extract(var_1, var_73);
    var_75 = wp::mul(var_72, var_74);
    var_77 = wp::extract(var_0, var_76);
    var_78 = wp::mul(var_15, var_77);
    var_80 = wp::extract(var_1, var_79);
    var_81 = wp::mul(var_78, var_80);
    var_82 = wp::add(var_75, var_81);
    // stretch[0, 2] * a[0] * b[2] + d1 * a[2] * b[0],                                        <L 1361>
    var_85 = wp::extract(var_stretch, var_83, var_84);
    var_87 = wp::extract(var_0, var_86);
    var_88 = wp::mul(var_85, var_87);
    var_90 = wp::extract(var_1, var_89);
    var_91 = wp::mul(var_88, var_90);
    var_93 = wp::extract(var_0, var_92);
    var_94 = wp::mul(var_29, var_93);
    var_96 = wp::extract(var_1, var_95);
    var_97 = wp::mul(var_94, var_96);
    var_98 = wp::add(var_91, var_97);
    // stretch[1, 0] * a[1] * b[0] + d0 * a[0] * b[1],                                        <L 1362>
    var_101 = wp::extract(var_stretch, var_99, var_100);
    var_103 = wp::extract(var_0, var_102);
    var_104 = wp::mul(var_101, var_103);
    var_106 = wp::extract(var_1, var_105);
    var_107 = wp::mul(var_104, var_106);
    var_109 = wp::extract(var_0, var_108);
    var_110 = wp::mul(var_15, var_109);
    var_112 = wp::extract(var_1, var_111);
    var_113 = wp::mul(var_110, var_112);
    var_114 = wp::add(var_107, var_113);
    // stretch[1, 1] * a[1] * b[1] + s0 * a[0] * b[0] + s2 * a[2] * b[2],                     <L 1363>
    var_117 = wp::extract(var_stretch, var_115, var_116);
    var_119 = wp::extract(var_0, var_118);
    var_120 = wp::mul(var_117, var_119);
    var_122 = wp::extract(var_1, var_121);
    var_123 = wp::mul(var_120, var_122);
    var_125 = wp::extract(var_0, var_124);
    var_126 = wp::mul(var_8, var_125);
    var_128 = wp::extract(var_1, var_127);
    var_129 = wp::mul(var_126, var_128);
    var_130 = wp::add(var_123, var_129);
    var_132 = wp::extract(var_0, var_131);
    var_133 = wp::mul(var_36, var_132);
    var_135 = wp::extract(var_1, var_134);
    var_136 = wp::mul(var_133, var_135);
    var_137 = wp::add(var_130, var_136);
    // stretch[1, 2] * a[1] * b[2] + d2 * a[2] * b[1],                                        <L 1364>
    var_140 = wp::extract(var_stretch, var_138, var_139);
    var_142 = wp::extract(var_0, var_141);
    var_143 = wp::mul(var_140, var_142);
    var_145 = wp::extract(var_1, var_144);
    var_146 = wp::mul(var_143, var_145);
    var_148 = wp::extract(var_0, var_147);
    var_149 = wp::mul(var_43, var_148);
    var_151 = wp::extract(var_1, var_150);
    var_152 = wp::mul(var_149, var_151);
    var_153 = wp::add(var_146, var_152);
    // stretch[2, 0] * a[2] * b[0] + d1 * a[0] * b[2],                                        <L 1365>
    var_156 = wp::extract(var_stretch, var_154, var_155);
    var_158 = wp::extract(var_0, var_157);
    var_159 = wp::mul(var_156, var_158);
    var_161 = wp::extract(var_1, var_160);
    var_162 = wp::mul(var_159, var_161);
    var_164 = wp::extract(var_0, var_163);
    var_165 = wp::mul(var_29, var_164);
    var_167 = wp::extract(var_1, var_166);
    var_168 = wp::mul(var_165, var_167);
    var_169 = wp::add(var_162, var_168);
    // stretch[2, 1] * a[2] * b[1] + d2 * a[1] * b[2],                                        <L 1366>
    var_172 = wp::extract(var_stretch, var_170, var_171);
    var_174 = wp::extract(var_0, var_173);
    var_175 = wp::mul(var_172, var_174);
    var_177 = wp::extract(var_1, var_176);
    var_178 = wp::mul(var_175, var_177);
    var_180 = wp::extract(var_0, var_179);
    var_181 = wp::mul(var_43, var_180);
    var_183 = wp::extract(var_1, var_182);
    var_184 = wp::mul(var_181, var_183);
    var_185 = wp::add(var_178, var_184);
    // stretch[2, 2] * a[2] * b[2] + s1 * a[0] * b[0] + s2 * a[1] * b[1],                     <L 1367>
    var_188 = wp::extract(var_stretch, var_186, var_187);
    var_190 = wp::extract(var_0, var_189);
    var_191 = wp::mul(var_188, var_190);
    var_193 = wp::extract(var_1, var_192);
    var_194 = wp::mul(var_191, var_193);
    var_196 = wp::extract(var_0, var_195);
    var_197 = wp::mul(var_22, var_196);
    var_199 = wp::extract(var_1, var_198);
    var_200 = wp::mul(var_197, var_199);
    var_201 = wp::add(var_194, var_200);
    var_203 = wp::extract(var_0, var_202);
    var_204 = wp::mul(var_36, var_203);
    var_206 = wp::extract(var_1, var_205);
    var_207 = wp::mul(var_204, var_206);
    var_208 = wp::add(var_201, var_207);
    var_209 = wp::mat_t<3, 3, wp::float32>(var_66, var_82, var_98, var_114, var_137, var_153, var_169, var_185, var_208);
    // return rotation * local * wp.transpose(rotation)                                       <L 1369>
    var_210 = wp::mul(var_rotation, var_209);
    var_211 = wp::transpose(var_rotation);
    var_212 = wp::mul(var_210, var_211);
    return var_212;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1163
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> stretch_stiffness_block_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_tension,
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::int32 var_dim,
    wp::int32 var_i,
    wp::int32 var_j,
    wp::float32 var_scale)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 2;
    bool var_1;
    const wp::int32 var_2 = 3;
    const wp::int32 var_3 = 6;
    wp::int32 var_4;
    wp::mat_t<6, 2, wp::int32> var_5;
    const wp::float32 var_6 = 0.0;
    const wp::float32 var_7 = 0.0;
    const wp::float32 var_8 = 0.0;
    const wp::float32 var_9 = 0.0;
    const wp::float32 var_10 = 0.0;
    const wp::float32 var_11 = 0.0;
    const wp::float32 var_12 = 0.0;
    const wp::float32 var_13 = 0.0;
    const wp::float32 var_14 = 0.0;
    wp::mat_t<3, 3, wp::float32> var_15;
    wp::range_t var_16;
    wp::int32 var_17;
    const wp::int32 var_18 = 0;
    wp::int32 var_19;
    bool var_20;
    const wp::float32 var_21 = 1.0;
    const wp::int32 var_22 = 1;
    wp::int32 var_23;
    bool var_24;
    const wp::float32 var_25 = -1.0;
    const wp::float32 var_26 = 0.0;
    wp::float32 var_27;
    wp::float32 var_28;
    const wp::float32 var_29 = 0.0;
    bool var_30;
    wp::range_t var_31;
    wp::int32 var_32;
    const wp::int32 var_33 = 0;
    wp::int32 var_34;
    bool var_35;
    const wp::float32 var_36 = 1.0;
    const wp::int32 var_37 = 1;
    wp::int32 var_38;
    bool var_39;
    const wp::float32 var_40 = -1.0;
    const wp::float32 var_41 = 0.0;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::float32 var_44 = 0.0;
    bool var_45;
    const wp::float32 var_46 = 2.0;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::int32 var_52 = 0;
    const wp::int32 var_53 = 0;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::float32 var_57;
    const wp::int32 var_58 = 1;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 2;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::float32 var_66;
    wp::float32 var_67;
    const wp::int32 var_68 = 1;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    wp::float32 var_73;
    const wp::int32 var_74 = 1;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::int32 var_79 = 2;
    wp::float32 var_80;
    wp::float32 var_81;
    wp::float32 var_82;
    wp::float32 var_83;
    const wp::int32 var_84 = 2;
    const wp::int32 var_85 = 0;
    wp::float32 var_86;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    const wp::int32 var_90 = 1;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    const wp::int32 var_95 = 2;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    wp::float32 var_99;
    const wp::float32 var_100 = 0.0;
    wp::float32 var_101;
    wp::range_t var_102;
    wp::int32 var_103;
    const wp::int32 var_104 = 0;
    wp::int32 var_105;
    bool var_106;
    const wp::float32 var_107 = 1.0;
    const wp::int32 var_108 = 1;
    wp::int32 var_109;
    bool var_110;
    const wp::float32 var_111 = -1.0;
    const wp::float32 var_112 = 0.0;
    wp::float32 var_113;
    wp::float32 var_114;
    const wp::int32 var_115 = 0;
    wp::int32 var_116;
    bool var_117;
    const wp::float32 var_118 = 1.0;
    const wp::int32 var_119 = 1;
    wp::int32 var_120;
    bool var_121;
    const wp::float32 var_122 = -1.0;
    const wp::float32 var_123 = 0.0;
    wp::float32 var_124;
    wp::float32 var_125;
    bool var_126;
    const wp::float32 var_127 = 0.0;
    bool var_128;
    const wp::float32 var_129 = 0.0;
    bool var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    wp::float32 var_133;
    wp::float32 var_134;
    wp::float32 var_135;
    wp::float32 var_136;
    const wp::int32 var_137 = 0;
    const wp::int32 var_138 = 0;
    const wp::int32 var_139 = 1;
    const wp::int32 var_140 = 1;
    const wp::int32 var_141 = 2;
    const wp::int32 var_142 = 2;
    //---------
    // forward
    // def stretch_stiffness_block(                                                           <L 1164>
    // nedge = 3 if dim == 2 else 6                                                           <L 1174>
    var_1 = (var_dim == var_0);
    if (var_1) {
    }
    if (!var_1) {
    }
    var_4 = wp::where(var_1, var_2, var_3);
    // edges = stretch_edge_endpoints(dim)                                                    <L 1175>
    var_5 = stretch_edge_endpoints_0(var_dim);
    // blk = wp.mat33(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                            <L 1176>
    var_15 = wp::mat_t<3, 3, wp::float32>(var_6, var_7, var_8, var_9, var_10, var_11, var_12, var_13, var_14);
    // for a in range(nedge):                                                                 <L 1177>
    var_16 = wp::range(var_4);
    start_for_0:;
        if (iter_cmp(var_16) == 0) goto end_for_0;
        var_17 = wp::iter_next(var_16);
        // sa = 1.0 if i == edges[a, 0] else (-1.0 if i == edges[a, 1] else 0.0)              <L 1178>
        var_19 = wp::extract(var_5, var_17, var_18);
        var_20 = (var_i == var_19);
        if (var_20) {
        }
        if (!var_20) {
            var_23 = wp::extract(var_5, var_17, var_22);
            var_24 = (var_i == var_23);
            if (var_24) {
            }
            if (!var_24) {
            }
            var_27 = wp::where(var_24, var_25, var_26);
        }
        var_28 = wp::where(var_20, var_21, var_27);
        // if sa == 0.0:                                                                      <L 1179>
        var_30 = (var_28 == var_29);
        if (var_30) {
            // continue                                                                       <L 1180>
            goto start_for_0;
        }
        // for b in range(nedge):                                                             <L 1181>
        var_31 = wp::range(var_4);
        start_for_2:;
            if (iter_cmp(var_31) == 0) goto end_for_2;
            var_32 = wp::iter_next(var_31);
            // sb = 1.0 if j == edges[b, 0] else (-1.0 if j == edges[b, 1] else 0.0)          <L 1182>
            var_34 = wp::extract(var_5, var_32, var_33);
            var_35 = (var_j == var_34);
            if (var_35) {
            }
            if (!var_35) {
                var_38 = wp::extract(var_5, var_32, var_37);
                var_39 = (var_j == var_38);
                if (var_39) {
                }
                if (!var_39) {
                }
                var_42 = wp::where(var_39, var_40, var_41);
            }
            var_43 = wp::where(var_35, var_36, var_42);
            // if sb == 0.0:                                                                  <L 1183>
            var_45 = (var_43 == var_44);
            if (var_45) {
                // continue                                                                   <L 1184>
                goto start_for_2;
            }
            // w = 2.0 * scale * metric[a, b] * sa * sb                                       <L 1185>
            var_47 = wp::mul(var_46, var_scale);
            var_48 = wp::extract(var_metric, var_17, var_32);
            var_49 = wp::mul(var_47, var_48);
            var_50 = wp::mul(var_49, var_28);
            var_51 = wp::mul(var_50, var_43);
            // for r in range(3):                                                             <L 1186>
            // for c in range(3):                                                             <L 1187>
            // blk[r, c] += w * edgevec[a, r] * edgevec[b, c]                                 <L 1188>
            var_54 = wp::extract(var_edgevec, var_17, var_52);
            var_55 = wp::mul(var_51, var_54);
            var_56 = wp::extract(var_edgevec, var_32, var_53);
            var_57 = wp::mul(var_55, var_56);
            wp::add_inplace(var_15, var_52, var_53, var_57);
            var_59 = wp::extract(var_edgevec, var_17, var_52);
            var_60 = wp::mul(var_51, var_59);
            var_61 = wp::extract(var_edgevec, var_32, var_58);
            var_62 = wp::mul(var_60, var_61);
            wp::add_inplace(var_15, var_52, var_58, var_62);
            var_64 = wp::extract(var_edgevec, var_17, var_52);
            var_65 = wp::mul(var_51, var_64);
            var_66 = wp::extract(var_edgevec, var_32, var_63);
            var_67 = wp::mul(var_65, var_66);
            wp::add_inplace(var_15, var_52, var_63, var_67);
            // for c in range(3):                                                             <L 1187>
            // blk[r, c] += w * edgevec[a, r] * edgevec[b, c]                                 <L 1188>
            var_70 = wp::extract(var_edgevec, var_17, var_68);
            var_71 = wp::mul(var_51, var_70);
            var_72 = wp::extract(var_edgevec, var_32, var_69);
            var_73 = wp::mul(var_71, var_72);
            wp::add_inplace(var_15, var_68, var_69, var_73);
            var_75 = wp::extract(var_edgevec, var_17, var_68);
            var_76 = wp::mul(var_51, var_75);
            var_77 = wp::extract(var_edgevec, var_32, var_74);
            var_78 = wp::mul(var_76, var_77);
            wp::add_inplace(var_15, var_68, var_74, var_78);
            var_80 = wp::extract(var_edgevec, var_17, var_68);
            var_81 = wp::mul(var_51, var_80);
            var_82 = wp::extract(var_edgevec, var_32, var_79);
            var_83 = wp::mul(var_81, var_82);
            wp::add_inplace(var_15, var_68, var_79, var_83);
            // for c in range(3):                                                             <L 1187>
            // blk[r, c] += w * edgevec[a, r] * edgevec[b, c]                                 <L 1188>
            var_86 = wp::extract(var_edgevec, var_17, var_84);
            var_87 = wp::mul(var_51, var_86);
            var_88 = wp::extract(var_edgevec, var_32, var_85);
            var_89 = wp::mul(var_87, var_88);
            wp::add_inplace(var_15, var_84, var_85, var_89);
            var_91 = wp::extract(var_edgevec, var_17, var_84);
            var_92 = wp::mul(var_51, var_91);
            var_93 = wp::extract(var_edgevec, var_32, var_90);
            var_94 = wp::mul(var_92, var_93);
            wp::add_inplace(var_15, var_84, var_90, var_94);
            var_96 = wp::extract(var_edgevec, var_17, var_84);
            var_97 = wp::mul(var_51, var_96);
            var_98 = wp::extract(var_edgevec, var_32, var_95);
            var_99 = wp::mul(var_97, var_98);
            wp::add_inplace(var_15, var_84, var_95, var_99);
            goto start_for_2;
        end_for_2:;
        goto start_for_0;
    end_for_0:;
    // geo = float(0.0)                                                                       <L 1189>
    var_101 = wp::float(var_100);
    // for a in range(nedge):                                                                 <L 1190>
    var_102 = wp::range(var_4);
    start_for_4:;
        if (iter_cmp(var_102) == 0) goto end_for_4;
        var_103 = wp::iter_next(var_102);
        // sa = 1.0 if i == edges[a, 0] else (-1.0 if i == edges[a, 1] else 0.0)              <L 1191>
        var_105 = wp::extract(var_5, var_103, var_104);
        var_106 = (var_i == var_105);
        if (var_106) {
        }
        if (!var_106) {
            var_109 = wp::extract(var_5, var_103, var_108);
            var_110 = (var_i == var_109);
            if (var_110) {
            }
            if (!var_110) {
            }
            var_113 = wp::where(var_110, var_111, var_112);
        }
        var_114 = wp::where(var_106, var_107, var_113);
        // sb = 1.0 if j == edges[a, 0] else (-1.0 if j == edges[a, 1] else 0.0)              <L 1192>
        var_116 = wp::extract(var_5, var_103, var_115);
        var_117 = (var_j == var_116);
        if (var_117) {
        }
        if (!var_117) {
            var_120 = wp::extract(var_5, var_103, var_119);
            var_121 = (var_j == var_120);
            if (var_121) {
            }
            if (!var_121) {
            }
            var_124 = wp::where(var_121, var_122, var_123);
        }
        var_125 = wp::where(var_117, var_118, var_124);
        // if sa != 0.0 and sb != 0.0:                                                        <L 1193>
        var_128 = (var_114 != var_127);
        var_126 = var_128;
        if (var_126) {
            var_130 = (var_125 != var_129);
            var_126 = var_126 && var_130;
        }
        if (var_126) {
            // geo += tension[a] * sa * sb                                                    <L 1194>
            var_131 = wp::extract(var_tension, var_103);
            var_132 = wp::mul(var_131, var_114);
            var_133 = wp::mul(var_132, var_125);
            var_134 = wp::add(var_101, var_133);
        }
        var_135 = wp::where(var_126, var_134, var_101);
        wp::assign(var_28, var_114);
        wp::assign(var_43, var_125);
        wp::assign(var_101, var_135);
        goto start_for_4;
    end_for_4:;
    // geo *= scale                                                                           <L 1195>
    var_136 = wp::mul(var_101, var_scale);
    // blk[0, 0] += geo                                                                       <L 1196>
    wp::add_inplace(var_15, var_137, var_138, var_136);
    // blk[1, 1] += geo                                                                       <L 1197>
    wp::add_inplace(var_15, var_139, var_140, var_136);
    // blk[2, 2] += geo                                                                       <L 1198>
    wp::add_inplace(var_15, var_141, var_142, var_136);
    // return blk                                                                             <L 1199>
    return var_15;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1277
static CUDA_CALLABLE wp::float32 flex_dphi_0(
    wp::float32 var_s,
    wp::int32 var_i,
    wp::int32 var_order)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    const wp::float32 var_2 = -1.0;
    const wp::float32 var_3 = 1.0;
    wp::float32 var_4;
    //---------
    // forward
    // def flex_dphi(s: float, i: int, order: int) -> float:                                  <L 1278>
    // return -1.0 if i == 0 else 1.0                                                         <L 1279>
    var_1 = (var_i == var_0);
    if (var_1) {
    }
    if (!var_1) {
    }
    var_4 = wp::where(var_1, var_2, var_3);
    return var_4;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1272
static CUDA_CALLABLE wp::float32 flex_phi_0(
    wp::float32 var_s,
    wp::int32 var_i,
    wp::int32 var_order)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    const wp::float32 var_2 = 1.0;
    wp::float32 var_3;
    wp::float32 var_4;
    //---------
    // forward
    // def flex_phi(s: float, i: int, order: int) -> float:                                   <L 1273>
    // return 1.0 - s if i == 0 else s                                                        <L 1274>
    var_1 = (var_i == var_0);
    if (var_1) {
        var_3 = wp::sub(var_2, var_s);
    }
    if (!var_1) {
    }
    var_4 = wp::where(var_1, var_3, var_s);
    return var_4;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1282
static CUDA_CALLABLE wp::float32 dphi2D_0(
    wp::float32 var_s0,
    wp::int32 var_l0,
    wp::float32 var_s1,
    wp::int32 var_l1,
    wp::int32 var_order,
    wp::int32 var_direction)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    bool var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    //---------
    // forward
    // def dphi2D(s0: float, l0: int, s1: float, l1: int, order: int, direction: int) -> float:       <L 1283>
    // if direction == 0:                                                                     <L 1284>
    var_1 = (var_direction == var_0);
    if (var_1) {
        // return flex_dphi(s0, l0, order) * flex_phi(s1, l1, order)                          <L 1285>
        var_2 = flex_dphi_0(var_s0, var_l0, var_order);
        var_3 = flex_phi_0(var_s1, var_l1, var_order);
        var_4 = wp::mul(var_2, var_3);
        return var_4;
    }
    if (!var_1) {
        // return flex_phi(s0, l0, order) * flex_dphi(s1, l1, order)                          <L 1287>
        var_5 = flex_phi_0(var_s0, var_l0, var_order);
        var_6 = flex_dphi_0(var_s1, var_l1, var_order);
        var_7 = wp::mul(var_5, var_6);
        return var_7;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2766
static CUDA_CALLABLE void _apply_face_forces_0(
    wp::array_t<wp::int32> var_flex_nodebodyid,
    wp::array_t<wp::int32> var_flex_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::int32 var_face_id,
    wp::vec_t<2, wp::float32> var_local_coords,
    wp::vec_t<3, wp::float32> var_wt1,
    wp::vec_t<3, wp::float32> var_wt2,
    wp::float32 var_stiffness_scale,
    wp::int32 var_order_abs,
    wp::int32 var_worldid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_body_force_out)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::int32 var_1;
    const wp::int32 var_2 = 3;
    wp::range_t var_3;
    wp::int32 var_4;
    bool var_5;
    const wp::int32 var_6 = 3;
    wp::range_t var_7;
    wp::int32 var_8;
    bool var_9;
    const wp::int32 var_10 = 0;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::float32 var_13;
    const wp::int32 var_14 = 0;
    wp::float32 var_15;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    const wp::int32 var_18 = 1;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    wp::int32* var_22;
    wp::int32 var_23;
    wp::int32 var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::int32* var_29;
    wp::int32 var_30;
    wp::int32 var_31;
    wp::vec_t<3, wp::float32>* var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32>* var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::vec_t<3, wp::float32> var_39;
    wp::vec_t<3, wp::float32> var_40;
    wp::vec_t<6, wp::float32> var_41;
    wp::vec_t<6, wp::float32> var_42;
    const wp::int32 var_43 = 1;
    wp::int32 var_44;
    //---------
    // forward
    // def _apply_face_forces(                                                                <L 2767>
    // idx = int(0)                                                                           <L 2785>
    var_1 = wp::int(var_0);
    // for l0 in range(3):                                                                    <L 2786>
    var_3 = wp::range(var_2);
    start_for_0:;
        if (iter_cmp(var_3) == 0) goto end_for_0;
        var_4 = wp::iter_next(var_3);
        // if l0 > order_abs:                                                                 <L 2787>
        var_5 = (var_4 > var_order_abs);
        if (var_5) {
            // continue                                                                       <L 2788>
            goto start_for_0;
        }
        // for l1 in range(3):                                                                <L 2789>
        var_7 = wp::range(var_6);
        start_for_2:;
            if (iter_cmp(var_7) == 0) goto end_for_2;
            var_8 = wp::iter_next(var_7);
            // if l1 > order_abs:                                                             <L 2790>
            var_9 = (var_8 > var_order_abs);
            if (var_9) {
                // continue                                                                   <L 2791>
                goto start_for_2;
            }
            // g0 = support.dphi2D(local_coords[0], l0, local_coords[1], l1, order_abs, 0)       <L 2792>
            var_11 = wp::extract(var_local_coords, var_10);
            var_13 = wp::extract(var_local_coords, var_12);
            var_15 = dphi2D_0(var_11, var_4, var_13, var_8, var_order_abs, var_14);
            // g1 = support.dphi2D(local_coords[0], l0, local_coords[1], l1, order_abs, 1)       <L 2793>
            var_17 = wp::extract(var_local_coords, var_16);
            var_19 = wp::extract(var_local_coords, var_18);
            var_21 = dphi2D_0(var_17, var_4, var_19, var_8, var_order_abs, var_20);
            // gidx = flex_face[face_id, idx]                                                 <L 2795>
            var_22 = wp::address(var_flex_face, var_face_id, var_1);
            var_24 = wp::load(var_22);
            var_23 = wp::copy(var_24);
            // frc = (wt2 * g0 - wt1 * g1) * stiffness_scale                                  <L 2797>
            var_25 = wp::mul(var_wt2, var_15);
            var_26 = wp::mul(var_wt1, var_21);
            var_27 = wp::sub(var_25, var_26);
            var_28 = wp::mul(var_27, var_stiffness_scale);
            // bid = flex_nodebodyid[gidx]                                                    <L 2799>
            var_29 = wp::address(var_flex_nodebodyid, var_23);
            var_31 = wp::load(var_29);
            var_30 = wp::copy(var_31);
            // node_pos = flexnode_xpos_in[worldid, gidx]                                     <L 2800>
            var_32 = wp::address(var_flexnode_xpos_in, var_worldid, var_23);
            var_34 = wp::load(var_32);
            var_33 = wp::copy(var_34);
            // body_xipos = xipos_in[worldid, bid]                                            <L 2802>
            var_35 = wp::address(var_xipos_in, var_worldid, var_30);
            var_37 = wp::load(var_35);
            var_36 = wp::copy(var_37);
            // offset = body_xipos - node_pos                                                 <L 2803>
            var_38 = wp::sub(var_36, var_33);
            // spatial_frc = wp.spatial_vector(frc, -wp.cross(offset, frc))                   <L 2804>
            var_39 = wp::cross(var_38, var_28);
            var_40 = wp::neg(var_39);
            var_41 = wp::vec_t<6, wp::float32>(var_28, var_40);
            // wp.atomic_add(body_force_out, worldid, bid, spatial_frc)                       <L 2805>
            var_42 = wp::atomic_add(var_body_force_out, var_worldid, var_30, var_41);
            // idx += 1                                                                       <L 2806>
            var_44 = wp::add(var_1, var_43);
            wp::assign(var_1, var_44);
            goto start_for_2;
        end_for_2:;
        goto start_for_0;
    end_for_0:;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2967
static CUDA_CALLABLE wp::float32 _flex_vert_mass_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_gv)
{
    //---------
    // primal vars
    wp::int32* var_0;
    wp::int32* var_1;
    wp::int32 var_2;
    wp::int32 var_3;
    wp::int32 var_4;
    wp::int32* var_5;
    const wp::int32 var_6 = 3;
    bool var_7;
    wp::int32 var_8;
    const wp::float32 var_9 = 0.0;
    wp::float32 var_10;
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
    wp::float32 var_22;
    wp::float32 var_23;
    //---------
    // forward
    // def _flex_vert_mass(                                                                   <L 2968>
    // b = body_weldid[flex_vertbodyid[gv]]                                                   <L 2982>
    var_0 = wp::address(var_flex_vertbodyid, var_gv);
    var_2 = wp::load(var_0);
    var_1 = wp::address(var_body_weldid, var_2);
    var_4 = wp::load(var_1);
    var_3 = wp::copy(var_4);
    // if body_dofnum[b] != 3:                                                                <L 2983>
    var_5 = wp::address(var_body_dofnum, var_3);
    var_8 = wp::load(var_5);
    var_7 = (var_8 != var_6);
    if (var_7) {
        // return float(0.0)                                                                  <L 2984>
        var_10 = wp::float(var_9);
        return var_10;
    }
    // da = body_dofadr[b]                                                                    <L 2985>
    var_11 = wp::address(var_body_dofadr, var_3);
    var_13 = wp::load(var_11);
    var_12 = wp::copy(var_13);
    // diag_idx = M_rowadr[da] + M_rownnz[da] - 1                                             <L 2986>
    var_14 = wp::address(var_M_rowadr, var_12);
    var_15 = wp::address(var_M_rownnz, var_12);
    var_17 = wp::load(var_14);
    var_18 = wp::load(var_15);
    var_16 = wp::add(var_17, var_18);
    var_20 = wp::sub(var_16, var_19);
    // return M_in[worldid, diag_idx]                                                         <L 2987>
    var_21 = wp::address(var_M_in, var_worldid, var_20);
    var_23 = wp::load(var_21);
    var_22 = wp::copy(var_23);
    return var_22;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2990
static CUDA_CALLABLE wp::float32 _flex_participant_mass_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_flex_id,
    wp::int32 var_elem_id,
    wp::int32 var_vert_id,
    wp::float32 var_mmin_in)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::int32 var_1 = 0;
    bool var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::float32 var_8;
    bool var_9;
    const wp::float32 var_10 = 0.0;
    bool var_11;
    bool var_12;
    const wp::float32 var_13 = 0.0;
    bool var_14;
    bool var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    wp::float32 var_18;
    const wp::int32 var_19 = 0;
    bool var_20;
    wp::int32* var_21;
    wp::int32 var_22;
    wp::int32 var_23;
    const wp::int32 var_24 = 1;
    wp::int32 var_25;
    wp::int32* var_26;
    wp::int32 var_27;
    wp::int32 var_28;
    wp::int32 var_29;
    wp::range_t var_30;
    wp::int32 var_31;
    wp::int32* var_32;
    wp::int32 var_33;
    wp::int32* var_34;
    wp::int32 var_35;
    wp::int32 var_36;
    wp::int32 var_37;
    wp::float32 var_38;
    bool var_39;
    const wp::float32 var_40 = 0.0;
    bool var_41;
    bool var_42;
    const wp::float32 var_43 = 0.0;
    bool var_44;
    bool var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    //---------
    // forward
    // def _flex_participant_mass(                                                            <L 2991>
    // mmin = mmin_in                                                                         <L 3012>
    var_0 = wp::copy(var_mmin_in);
    // if flex_id >= 0:                                                                       <L 3013>
    var_2 = (var_flex_id >= var_1);
    if (var_2) {
        // if vert_id >= 0:                                                                   <L 3014>
        var_4 = (var_vert_id >= var_3);
        if (var_4) {
            // gv = flex_vertadr[flex_id] + vert_id                                           <L 3015>
            var_5 = wp::address(var_flex_vertadr, var_flex_id);
            var_7 = wp::load(var_5);
            var_6 = wp::add(var_7, var_vert_id);
            // mv = _flex_vert_mass(body_weldid, body_dofnum, body_dofadr, flex_vertbodyid, M_rownnz, M_rowadr, M_in, worldid, gv)       <L 3016>
            var_8 = _flex_vert_mass_0(var_body_weldid, var_body_dofnum, var_body_dofadr, var_flex_vertbodyid, var_M_rownnz, var_M_rowadr, var_M_in, var_worldid, var_6);
            // if mv > 0.0 and (mmin == 0.0 or mv < mmin):                                    <L 3017>
            var_11 = (var_8 > var_10);
            var_9 = var_11;
            if (var_9) {
                var_14 = (var_0 == var_13);
                var_12 = var_14;
                if (!var_12) {
                    var_15 = (var_8 < var_0);
                    var_12 = var_12 || var_15;
                }
                var_9 = var_9 && var_12;
            }
            if (var_9) {
                // mmin = mv                                                                  <L 3018>
                var_16 = wp::copy(var_8);
            }
            var_17 = wp::where(var_9, var_16, var_0);
        }
        var_18 = wp::where(var_4, var_17, var_0);
        if (!var_4) {
            // elif elem_id >= 0:                                                             <L 3019>
            var_20 = (var_elem_id >= var_19);
            if (var_20) {
                // dim = flex_dim[flex_id]                                                    <L 3020>
                var_21 = wp::address(var_flex_dim, var_flex_id);
                var_23 = wp::load(var_21);
                var_22 = wp::copy(var_23);
                // nvrt = dim + 1                                                             <L 3021>
                var_25 = wp::add(var_22, var_24);
                // base_e = flex_elemdataadr[flex_id] + nvrt * elem_id                        <L 3022>
                var_26 = wp::address(var_flex_elemdataadr, var_flex_id);
                var_27 = wp::mul(var_25, var_elem_id);
                var_29 = wp::load(var_26);
                var_28 = wp::add(var_29, var_27);
                // for j in range(nvrt):                                                      <L 3023>
                var_30 = wp::range(var_25);
                start_for_0:;
                    if (iter_cmp(var_30) == 0) goto end_for_0;
                    var_31 = wp::iter_next(var_30);
                    // gv = flex_vertadr[flex_id] + flex_elem[base_e + j]                     <L 3024>
                    var_32 = wp::address(var_flex_vertadr, var_flex_id);
                    var_33 = wp::add(var_28, var_31);
                    var_34 = wp::address(var_flex_elem, var_33);
                    var_36 = wp::load(var_32);
                    var_37 = wp::load(var_34);
                    var_35 = wp::add(var_36, var_37);
                    // mv = _flex_vert_mass(body_weldid, body_dofnum, body_dofadr, flex_vertbodyid, M_rownnz, M_rowadr, M_in, worldid, gv)       <L 3025>
                    var_38 = _flex_vert_mass_0(var_body_weldid, var_body_dofnum, var_body_dofadr, var_flex_vertbodyid, var_M_rownnz, var_M_rowadr, var_M_in, var_worldid, var_35);
                    // if mv > 0.0 and (mmin == 0.0 or mv < mmin):                            <L 3026>
                    var_41 = (var_38 > var_40);
                    var_39 = var_41;
                    if (var_39) {
                        var_44 = (var_18 == var_43);
                        var_42 = var_44;
                        if (!var_42) {
                            var_45 = (var_38 < var_18);
                            var_42 = var_42 || var_45;
                        }
                        var_39 = var_39 && var_42;
                    }
                    if (var_39) {
                        // mmin = mv                                                          <L 3027>
                        var_46 = wp::copy(var_38);
                    }
                    var_47 = wp::where(var_39, var_46, var_18);
                    wp::assign(var_18, var_47);
                    wp::assign(var_6, var_35);
                    wp::assign(var_8, var_38);
                    goto start_for_0;
                end_for_0:;
            }
        }
    }
    var_48 = wp::where(var_2, var_18, var_0);
    // return mmin                                                                            <L 3028>
    return var_48;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:3031
static CUDA_CALLABLE wp::float32 _flex_contact_stiffness_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_flex0,
    wp::int32 var_flex1,
    wp::int32 var_elem0,
    wp::int32 var_elem1,
    wp::int32 var_vert0,
    wp::int32 var_vert1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::float32 var_3 = 50000000.0;
    wp::float32 var_4;
    //---------
    // forward
    // def _flex_contact_stiffness(                                                           <L 3032>
    // mmin = _flex_participant_mass(                                                         <L 3055>
    // body_weldid,                                                                           <L 3056>
    // body_dofnum,                                                                           <L 3057>
    // body_dofadr,                                                                           <L 3058>
    // flex_dim,                                                                              <L 3059>
    // flex_vertadr,                                                                          <L 3060>
    // flex_elemdataadr,                                                                      <L 3061>
    // flex_vertbodyid,                                                                       <L 3062>
    // flex_elem,                                                                             <L 3063>
    // M_rownnz,                                                                              <L 3064>
    // M_rowadr,                                                                              <L 3065>
    // M_in,                                                                                  <L 3066>
    // worldid,                                                                               <L 3067>
    // flex0,                                                                                 <L 3068>
    // elem0,                                                                                 <L 3069>
    // vert0,                                                                                 <L 3070>
    // 0.0,                                                                                   <L 3071>
    var_1 = _flex_participant_mass_0(var_body_weldid, var_body_dofnum, var_body_dofadr, var_flex_dim, var_flex_vertadr, var_flex_elemdataadr, var_flex_vertbodyid, var_flex_elem, var_M_rownnz, var_M_rowadr, var_M_in, var_worldid, var_flex0, var_elem0, var_vert0, var_0);
    // mmin = _flex_participant_mass(                                                         <L 3073>
    // body_weldid,                                                                           <L 3074>
    // body_dofnum,                                                                           <L 3075>
    // body_dofadr,                                                                           <L 3076>
    // flex_dim,                                                                              <L 3077>
    // flex_vertadr,                                                                          <L 3078>
    // flex_elemdataadr,                                                                      <L 3079>
    // flex_vertbodyid,                                                                       <L 3080>
    // flex_elem,                                                                             <L 3081>
    // M_rownnz,                                                                              <L 3082>
    // M_rowadr,                                                                              <L 3083>
    // M_in,                                                                                  <L 3084>
    // worldid,                                                                               <L 3085>
    // flex1,                                                                                 <L 3086>
    // elem1,                                                                                 <L 3087>
    // vert1,                                                                                 <L 3088>
    // mmin,                                                                                  <L 3089>
    var_2 = _flex_participant_mass_0(var_body_weldid, var_body_dofnum, var_body_dofadr, var_flex_dim, var_flex_vertadr, var_flex_elemdataadr, var_flex_vertbodyid, var_flex_elem, var_M_rownnz, var_M_rowadr, var_M_in, var_worldid, var_flex1, var_elem1, var_vert1, var_1);
    // return 5.0e7 * mmin                                                                    <L 3093>
    var_4 = wp::mul(var_3, var_2);
    return var_4;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:3096
static CUDA_CALLABLE wp::int32 _add_con_dof_0(
    wp::int32 var_cid,
    wp::int32 var_dof,
    wp::float32 var_val,
    wp::int32 var_nnz,
    wp::array_t<wp::int32> var_efm_con_dof_out,
    wp::array_t<wp::float32> var_efm_con_val_out)
{
    //---------
    // primal vars
    wp::range_t var_0;
    wp::int32 var_1;
    wp::int32* var_2;
    bool var_3;
    wp::int32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 1;
    wp::int32 var_7;
    //---------
    // forward
    // def _add_con_dof(                                                                      <L 3097>
    // for idx in range(nnz):                                                                 <L 3107>
    var_0 = wp::range(var_nnz);
    start_for_0:;
        if (iter_cmp(var_0) == 0) goto end_for_0;
        var_1 = wp::iter_next(var_0);
        // if efm_con_dof_out[cid, idx] == dof:                                               <L 3108>
        var_2 = wp::address(var_efm_con_dof_out, var_cid, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_4 == var_dof);
        if (var_3) {
            // efm_con_val_out[cid, idx] += val                                               <L 3109>
            var_5 = wp::atomic_add(var_efm_con_val_out, var_cid, var_1, var_val);
            // return nnz                                                                     <L 3110>
            return var_nnz;
        }
        goto start_for_0;
    end_for_0:;
    // efm_con_dof_out[cid, nnz] = dof                                                        <L 3111>
    wp::array_store(var_efm_con_dof_out, var_cid, var_nnz, var_dof);
    // efm_con_val_out[cid, nnz] = val                                                        <L 3112>
    wp::array_store(var_efm_con_val_out, var_cid, var_nnz, var_val);
    // return nnz + 1                                                                         <L 3113>
    var_7 = wp::add(var_nnz, var_6);
    return var_7;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:749
static CUDA_CALLABLE void _snh_volume_0(
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::float32 var_inv_det_dm,
    wp::float32 & ret_0,
    wp::mat_t<4, 3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    const wp::int32 var_3 = 2;
    wp::vec_t<3, wp::float32> var_4;
    const wp::int32 var_5 = 4;
    wp::vec_t<3, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::vec_t<3, wp::float32> var_16;
    wp::mat_t<4, 3, wp::float32> var_17;
    const wp::int32 var_18 = 0;
    const wp::int32 var_19 = 1;
    const wp::int32 var_20 = 2;
    const wp::int32 var_21 = 3;
    wp::float32 var_22;
    wp::float32 var_23;
    //---------
    // forward
    // def _snh_volume(edgevec: mat63, inv_det_dm: float) -> tuple[float, mat43]:             <L 750>
    // a = -edgevec[0]                                                                        <L 751>
    var_1 = wp::extract(var_edgevec, var_0);
    var_2 = wp::neg(var_1);
    // b = edgevec[2]                                                                         <L 752>
    var_4 = wp::extract(var_edgevec, var_3);
    // c = -edgevec[4]                                                                        <L 753>
    var_6 = wp::extract(var_edgevec, var_5);
    var_7 = wp::neg(var_6);
    // bxc = wp.cross(b, c)                                                                   <L 754>
    var_8 = wp::cross(var_4, var_7);
    // cxa = wp.cross(c, a)                                                                   <L 755>
    var_9 = wp::cross(var_7, var_2);
    // axb = wp.cross(a, b)                                                                   <L 756>
    var_10 = wp::cross(var_2, var_4);
    // g1 = bxc * inv_det_dm                                                                  <L 757>
    var_11 = wp::mul(var_8, var_inv_det_dm);
    // g2 = cxa * inv_det_dm                                                                  <L 758>
    var_12 = wp::mul(var_9, var_inv_det_dm);
    // g3 = axb * inv_det_dm                                                                  <L 759>
    var_13 = wp::mul(var_10, var_inv_det_dm);
    // g0 = -(g1 + g2 + g3)                                                                   <L 760>
    var_14 = wp::add(var_11, var_12);
    var_15 = wp::add(var_14, var_13);
    var_16 = wp::neg(var_15);
    // grad = mat43()                                                                         <L 761>
    var_17 = wp::mat_t<4, 3, wp::float32>();
    // grad[0] = g0                                                                           <L 762>
    wp::assign_inplace(var_17, var_18, var_16);
    // grad[1] = g1                                                                           <L 763>
    wp::assign_inplace(var_17, var_19, var_11);
    // grad[2] = g2                                                                           <L 764>
    wp::assign_inplace(var_17, var_20, var_12);
    // grad[3] = g3                                                                           <L 765>
    wp::assign_inplace(var_17, var_21, var_13);
    // return wp.dot(a, bxc) * inv_det_dm, grad                                               <L 766>
    var_22 = wp::dot(var_2, var_8);
    var_23 = wp::mul(var_22, var_inv_det_dm);
    ret_0 = var_23;
    ret_1 = var_17;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1622
static CUDA_CALLABLE void adj__flex_hessian_active_0(
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<bool> var_flex_rigid,
    wp::int32 var_f,
    wp::array_t<wp::int32> & adj_flex_dim,
    wp::array_t<wp::int32> & adj_flex_interp,
    wp::array_t<wp::int32> & adj_flex_stiffnessadr,
    wp::array_t<wp::float32> & adj_flex_stiffness,
    wp::array_t<bool> & adj_flex_rigid,
    wp::int32 & adj_f,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:53
static CUDA_CALLABLE void adj_geom_semiaxes_0(
    wp::vec_t<3, wp::float32> var_size,
    wp::int32 var_geom_type,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::int32 & adj_geom_type,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:42
static CUDA_CALLABLE void adj__pow2_0(
    wp::float32 var_val,
    wp::float32 & adj_val,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:47
static CUDA_CALLABLE void adj__pow4_0(
    wp::float32 var_val,
    wp::float32 & adj_val,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:73
static CUDA_CALLABLE void adj_ellipsoid_max_moment_0(
    wp::vec_t<3, wp::float32> var_size,
    wp::int32 var_dir,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::int32 & adj_dir,
    wp::float32 & adj_ret)
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:67
static CUDA_CALLABLE void adj_mat33_to_quat_polar_0(
    wp::mat_t<3, 3, wp::float32> var_F,
    wp::mat_t<3, 3, wp::float32> & adj_F,
    wp::quat_t<wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:101
static CUDA_CALLABLE void adj_compute_interp_cell_quat_0(
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::int32 var_order,
    wp::int32 var_ci,
    wp::int32 var_cj,
    wp::int32 var_ck,
    wp::int32 var_cy,
    wp::int32 var_cz,
    wp::int32 var_ny_g,
    wp::int32 var_nz_g,
    wp::int32 var_nstart,
    wp::int32 var_worldid,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_flexnode_xpos_in,
    wp::int32 & adj_order,
    wp::int32 & adj_ci,
    wp::int32 & adj_cj,
    wp::int32 & adj_ck,
    wp::int32 & adj_cy,
    wp::int32 & adj_cz,
    wp::int32 & adj_ny_g,
    wp::int32 & adj_nz_g,
    wp::int32 & adj_nstart,
    wp::int32 & adj_worldid,
    wp::quat_t<wp::float32> & adj_ret)
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1056
static CUDA_CALLABLE void adj_stretch_edge_endpoints_0(
    wp::int32 var_dim,
    wp::int32 & adj_dim,
    wp::mat_t<6, 2, wp::int32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1065
static CUDA_CALLABLE void adj_stretch_edge_vectors_0(
    wp::mat_t<4, 3, wp::float32> var_vert_xpos,
    wp::int32 var_dim,
    wp::mat_t<4, 3, wp::float32> & adj_vert_xpos,
    wp::int32 & adj_dim,
    wp::mat_t<6, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1075
static CUDA_CALLABLE void adj_stretch_elongation_0(
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::int32 var_worldid,
    wp::int32 var_elemedge_adr,
    wp::int32 var_edge_adr,
    wp::int32 var_nedge,
    wp::array_t<wp::int32> & adj_flex_elemedge,
    wp::array_t<wp::float32> & adj_flexedge_length0,
    wp::array_t<wp::float32> & adj_flexedge_length_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_elemedge_adr,
    wp::int32 & adj_edge_adr,
    wp::int32 & adj_nedge,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:955
static CUDA_CALLABLE void adj__snh_jacobi_rot_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::mat_t<3, 3, wp::float32> var_V,
    wp::int32 var_p,
    wp::int32 var_q,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    bool & ret_2,
    wp::mat_t<3, 3, wp::float32> & adj_A,
    wp::mat_t<3, 3, wp::float32> & adj_V,
    wp::int32 & adj_p,
    wp::int32 & adj_q,
    wp::mat_t<3, 3, wp::float32> & adj_ret_0,
    wp::mat_t<3, 3, wp::float32> & adj_ret_1,
    bool & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:978
static CUDA_CALLABLE void adj__snh_swap_cols_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::mat_t<3, 3, wp::float32> var_V,
    wp::vec_t<3, wp::float32> var_s,
    wp::int32 var_p,
    wp::int32 var_q,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::mat_t<3, 3, wp::float32> & adj_A,
    wp::mat_t<3, 3, wp::float32> & adj_V,
    wp::vec_t<3, wp::float32> & adj_s,
    wp::int32 & adj_p,
    wp::int32 & adj_q,
    wp::mat_t<3, 3, wp::float32> & adj_ret_0,
    wp::mat_t<3, 3, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:994
static CUDA_CALLABLE void adj__snh_svd_0(
    wp::mat_t<3, 3, wp::float32> var_F,
    wp::mat_t<3, 3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::mat_t<3, 3, wp::float32> & ret_2,
    wp::mat_t<3, 3, wp::float32> & adj_F,
    wp::mat_t<3, 3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::mat_t<3, 3, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1097
static CUDA_CALLABLE void adj_stretch_metric_0(
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::int32 var_nedge,
    wp::array_t<wp::float32> & adj_flex_stiffness,
    wp::int32 & adj_stiffness_adr,
    wp::int32 & adj_nedge,
    wp::mat_t<6, 6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1116
static CUDA_CALLABLE void adj_stretch_tension_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_elongation,
    wp::int32 var_nedge,
    wp::mat_t<6, 6, wp::float32> & adj_metric,
    wp::vec_t<6, wp::float32> & adj_elongation,
    wp::int32 & adj_nedge,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1127
static CUDA_CALLABLE void adj_stretch_elasticity_0(
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::vec_t<6, wp::float32> var_elongation,
    wp::int32 var_nedge,
    wp::mat_t<6, 6, wp::float32> & ret_0,
    wp::vec_t<6, wp::float32> & ret_1,
    wp::array_t<wp::float32> & adj_flex_stiffness,
    wp::int32 & adj_stiffness_adr,
    wp::vec_t<6, wp::float32> & adj_elongation,
    wp::int32 & adj_nedge,
    wp::mat_t<6, 6, wp::float32> & adj_ret_0,
    wp::vec_t<6, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:724
static CUDA_CALLABLE void adj__snh_cubic_0(
    wp::vec_t<6, wp::float32> var_tension,
    wp::vec_t<6, wp::float32> var_s,
    wp::float32 var_gamma,
    wp::vec_t<6, wp::float32> & adj_tension,
    wp::vec_t<6, wp::float32> & adj_s,
    wp::float32 & adj_gamma,
    wp::vec_t<6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:769
static CUDA_CALLABLE void adj__snh_cubic_metric_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_s,
    wp::float32 var_gamma,
    wp::mat_t<6, 6, wp::float32> & adj_metric,
    wp::vec_t<6, wp::float32> & adj_s,
    wp::float32 & adj_gamma,
    wp::mat_t<6, 6, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:840
static CUDA_CALLABLE void adj__snh_positive3_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::mat_t<3, 3, wp::float32> & adj_A,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:856
static CUDA_CALLABLE void adj__snh_eigen3_0(
    wp::mat_t<3, 3, wp::float32> var_A,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    wp::mat_t<3, 3, wp::float32> & adj_A,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::mat_t<3, 3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1202
static CUDA_CALLABLE void adj_snh_project_0(
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_vert0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::int32 var_stiffness_adr,
    wp::int32 var_vbase,
    wp::vec_t<4, wp::int32> var_vert,
    wp::vec_t<3, wp::float32> var_size,
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::vec_t<6, wp::float32> var_elongation,
    bool & ret_0,
    wp::mat_t<3, 3, wp::float32> & ret_1,
    wp::mat_t<4, 3, wp::float32> & ret_2,
    wp::mat_t<3, 3, wp::float32> & ret_3,
    wp::vec_t<3, wp::float32> & ret_4,
    wp::vec_t<3, wp::float32> & ret_5,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_flex_vert0,
    wp::array_t<wp::float32> & adj_flex_stiffness,
    wp::int32 & adj_stiffness_adr,
    wp::int32 & adj_vbase,
    wp::vec_t<4, wp::int32> & adj_vert,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::mat_t<6, 3, wp::float32> & adj_edgevec,
    wp::vec_t<6, wp::float32> & adj_elongation,
    bool & adj_ret_0,
    wp::mat_t<3, 3, wp::float32> & adj_ret_1,
    wp::mat_t<4, 3, wp::float32> & adj_ret_2,
    wp::mat_t<3, 3, wp::float32> & adj_ret_3,
    wp::vec_t<3, wp::float32> & adj_ret_4,
    wp::vec_t<3, wp::float32> & adj_ret_5)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1141
static CUDA_CALLABLE void adj_stretch_stiffness_0(
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::int32 var_worldid,
    wp::int32 var_stiffness_adr,
    wp::int32 var_elemedge_adr,
    wp::int32 var_edge_adr,
    wp::int32 var_nedge,
    wp::mat_t<6, 6, wp::float32> & ret_0,
    wp::vec_t<6, wp::float32> & ret_1,
    wp::array_t<wp::int32> & adj_flex_elemedge,
    wp::array_t<wp::float32> & adj_flexedge_length0,
    wp::array_t<wp::float32> & adj_flex_stiffness,
    wp::array_t<wp::float32> & adj_flexedge_length_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_stiffness_adr,
    wp::int32 & adj_elemedge_adr,
    wp::int32 & adj_edge_adr,
    wp::int32 & adj_nedge,
    wp::mat_t<6, 6, wp::float32> & adj_ret_0,
    wp::vec_t<6, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1338
static CUDA_CALLABLE void adj_snh_projected_block_0(
    wp::mat_t<3, 3, wp::float32> var_rotation,
    wp::mat_t<4, 3, wp::float32> var_gradient,
    wp::mat_t<3, 3, wp::float32> var_stretch,
    wp::vec_t<3, wp::float32> var_symmetric,
    wp::vec_t<3, wp::float32> var_skew,
    wp::int32 var_i,
    wp::int32 var_j,
    wp::mat_t<3, 3, wp::float32> & adj_rotation,
    wp::mat_t<4, 3, wp::float32> & adj_gradient,
    wp::mat_t<3, 3, wp::float32> & adj_stretch,
    wp::vec_t<3, wp::float32> & adj_symmetric,
    wp::vec_t<3, wp::float32> & adj_skew,
    wp::int32 & adj_i,
    wp::int32 & adj_j,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:1163
static CUDA_CALLABLE void adj_stretch_stiffness_block_0(
    wp::mat_t<6, 6, wp::float32> var_metric,
    wp::vec_t<6, wp::float32> var_tension,
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::int32 var_dim,
    wp::int32 var_i,
    wp::int32 var_j,
    wp::float32 var_scale,
    wp::mat_t<6, 6, wp::float32> & adj_metric,
    wp::vec_t<6, wp::float32> & adj_tension,
    wp::mat_t<6, 3, wp::float32> & adj_edgevec,
    wp::int32 & adj_dim,
    wp::int32 & adj_i,
    wp::int32 & adj_j,
    wp::float32 & adj_scale,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1277
static CUDA_CALLABLE void adj_flex_dphi_0(
    wp::float32 var_s,
    wp::int32 var_i,
    wp::int32 var_order,
    wp::float32 & adj_s,
    wp::int32 & adj_i,
    wp::int32 & adj_order,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1272
static CUDA_CALLABLE void adj_flex_phi_0(
    wp::float32 var_s,
    wp::int32 var_i,
    wp::int32 var_order,
    wp::float32 & adj_s,
    wp::int32 & adj_i,
    wp::int32 & adj_order,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\support.py:1282
static CUDA_CALLABLE void adj_dphi2D_0(
    wp::float32 var_s0,
    wp::int32 var_l0,
    wp::float32 var_s1,
    wp::int32 var_l1,
    wp::int32 var_order,
    wp::int32 var_direction,
    wp::float32 & adj_s0,
    wp::int32 & adj_l0,
    wp::float32 & adj_s1,
    wp::int32 & adj_l1,
    wp::int32 & adj_order,
    wp::int32 & adj_direction,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2766
static CUDA_CALLABLE void adj__apply_face_forces_0(
    wp::array_t<wp::int32> var_flex_nodebodyid,
    wp::array_t<wp::int32> var_flex_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::int32 var_face_id,
    wp::vec_t<2, wp::float32> var_local_coords,
    wp::vec_t<3, wp::float32> var_wt1,
    wp::vec_t<3, wp::float32> var_wt2,
    wp::float32 var_stiffness_scale,
    wp::int32 var_order_abs,
    wp::int32 var_worldid,
    wp::array_t<wp::vec_t<6, wp::float32>> var_body_force_out,
    wp::array_t<wp::int32> & adj_flex_nodebodyid,
    wp::array_t<wp::int32> & adj_flex_face,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_flexnode_xpos_in,
    wp::int32 & adj_face_id,
    wp::vec_t<2, wp::float32> & adj_local_coords,
    wp::vec_t<3, wp::float32> & adj_wt1,
    wp::vec_t<3, wp::float32> & adj_wt2,
    wp::float32 & adj_stiffness_scale,
    wp::int32 & adj_order_abs,
    wp::int32 & adj_worldid,
    wp::array_t<wp::vec_t<6, wp::float32>> & adj_body_force_out)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2967
static CUDA_CALLABLE void adj__flex_vert_mass_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_gv,
    wp::array_t<wp::int32> & adj_body_weldid,
    wp::array_t<wp::int32> & adj_body_dofnum,
    wp::array_t<wp::int32> & adj_body_dofadr,
    wp::array_t<wp::int32> & adj_flex_vertbodyid,
    wp::array_t<wp::int32> & adj_M_rownnz,
    wp::array_t<wp::int32> & adj_M_rowadr,
    wp::array_t<wp::float32> & adj_M_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_gv,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:2990
static CUDA_CALLABLE void adj__flex_participant_mass_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_flex_id,
    wp::int32 var_elem_id,
    wp::int32 var_vert_id,
    wp::float32 var_mmin_in,
    wp::array_t<wp::int32> & adj_body_weldid,
    wp::array_t<wp::int32> & adj_body_dofnum,
    wp::array_t<wp::int32> & adj_body_dofadr,
    wp::array_t<wp::int32> & adj_flex_dim,
    wp::array_t<wp::int32> & adj_flex_vertadr,
    wp::array_t<wp::int32> & adj_flex_elemdataadr,
    wp::array_t<wp::int32> & adj_flex_vertbodyid,
    wp::array_t<wp::int32> & adj_flex_elem,
    wp::array_t<wp::int32> & adj_M_rownnz,
    wp::array_t<wp::int32> & adj_M_rowadr,
    wp::array_t<wp::float32> & adj_M_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_flex_id,
    wp::int32 & adj_elem_id,
    wp::int32 & adj_vert_id,
    wp::float32 & adj_mmin_in,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:3031
static CUDA_CALLABLE void adj__flex_contact_stiffness_0(
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::float32> var_M_in,
    wp::int32 var_worldid,
    wp::int32 var_flex0,
    wp::int32 var_flex1,
    wp::int32 var_elem0,
    wp::int32 var_elem1,
    wp::int32 var_vert0,
    wp::int32 var_vert1,
    wp::array_t<wp::int32> & adj_body_weldid,
    wp::array_t<wp::int32> & adj_body_dofnum,
    wp::array_t<wp::int32> & adj_body_dofadr,
    wp::array_t<wp::int32> & adj_flex_dim,
    wp::array_t<wp::int32> & adj_flex_vertadr,
    wp::array_t<wp::int32> & adj_flex_elemdataadr,
    wp::array_t<wp::int32> & adj_flex_vertbodyid,
    wp::array_t<wp::int32> & adj_flex_elem,
    wp::array_t<wp::int32> & adj_M_rownnz,
    wp::array_t<wp::int32> & adj_M_rowadr,
    wp::array_t<wp::float32> & adj_M_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_flex0,
    wp::int32 & adj_flex1,
    wp::int32 & adj_elem0,
    wp::int32 & adj_elem1,
    wp::int32 & adj_vert0,
    wp::int32 & adj_vert1,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:3096
static CUDA_CALLABLE void adj__add_con_dof_0(
    wp::int32 var_cid,
    wp::int32 var_dof,
    wp::float32 var_val,
    wp::int32 var_nnz,
    wp::array_t<wp::int32> var_efm_con_dof_out,
    wp::array_t<wp::float32> var_efm_con_val_out,
    wp::int32 & adj_cid,
    wp::int32 & adj_dof,
    wp::float32 & adj_val,
    wp::int32 & adj_nnz,
    wp::array_t<wp::int32> & adj_efm_con_dof_out,
    wp::array_t<wp::float32> & adj_efm_con_val_out,
    wp::int32 & adj_ret)
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\passive.py:749
static CUDA_CALLABLE void adj__snh_volume_0(
    wp::mat_t<6, 3, wp::float32> var_edgevec,
    wp::float32 var_inv_det_dm,
    wp::float32 & ret_0,
    wp::mat_t<4, 3, wp::float32> & ret_1,
    wp::mat_t<6, 3, wp::float32> & adj_edgevec,
    wp::float32 & adj_inv_det_dm,
    wp::float32 & adj_ret_0,
    wp::mat_t<4, 3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _flex_hessian_diag_0362059a_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_edge,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<bool> var_flex_rigid,
    wp::array_t<wp::int32> var_flex_edgeflexid,
    wp::array_t<bool> var_flex_hessian_valid_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_flexedge_hessian_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flexvert_hessian_out)
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
        bool* var_5;
        bool var_6;
        bool var_7;
        bool var_8;
        bool var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        wp::vec_t<2, wp::int32>* var_13;
        wp::vec_t<2, wp::int32> var_14;
        wp::vec_t<2, wp::int32> var_15;
        wp::mat_t<3, 3, wp::float32>* var_16;
        wp::mat_t<3, 3, wp::float32> var_17;
        wp::mat_t<3, 3, wp::float32> var_18;
        const wp::int32 var_19 = 0;
        const wp::int32 var_20 = 0;
        wp::float32 var_21;
        wp::float32 var_22;
        const wp::int32 var_23 = 0;
        const wp::int32 var_24 = 1;
        wp::float32 var_25;
        wp::float32 var_26;
        const wp::int32 var_27 = 0;
        const wp::int32 var_28 = 2;
        wp::float32 var_29;
        wp::float32 var_30;
        const wp::int32 var_31 = 1;
        const wp::int32 var_32 = 1;
        wp::float32 var_33;
        wp::float32 var_34;
        const wp::int32 var_35 = 1;
        const wp::int32 var_36 = 2;
        wp::float32 var_37;
        wp::float32 var_38;
        const wp::int32 var_39 = 2;
        const wp::int32 var_40 = 2;
        wp::float32 var_41;
        wp::float32 var_42;
        wp::vec_t<6, wp::float32> var_43;
        const wp::int32 var_44 = 0;
        const wp::int32 var_45 = 0;
        wp::float32 var_46;
        wp::float32 var_47;
        const wp::int32 var_48 = 1;
        const wp::int32 var_49 = 0;
        wp::float32 var_50;
        wp::float32 var_51;
        const wp::int32 var_52 = 2;
        const wp::int32 var_53 = 0;
        wp::float32 var_54;
        wp::float32 var_55;
        const wp::int32 var_56 = 1;
        const wp::int32 var_57 = 1;
        wp::float32 var_58;
        wp::float32 var_59;
        const wp::int32 var_60 = 2;
        const wp::int32 var_61 = 1;
        wp::float32 var_62;
        wp::float32 var_63;
        const wp::int32 var_64 = 2;
        const wp::int32 var_65 = 2;
        wp::float32 var_66;
        wp::float32 var_67;
        wp::vec_t<6, wp::float32> var_68;
        const wp::int32 var_69 = 0;
        wp::int32 var_70;
        wp::int32 var_71;
        wp::vec_t<6, wp::float32> var_72;
        const wp::int32 var_73 = 1;
        wp::int32 var_74;
        wp::int32 var_75;
        wp::vec_t<6, wp::float32> var_76;
        //---------
        // forward
        // def _flex_hessian_diag(                                                                <L 1773>
        // worldid, edgeid = wp.tid()                                                             <L 1789>
        builtin_tid2d(var_0, var_1);
        // f = flex_edgeflexid[edgeid]                                                            <L 1790>
        var_2 = wp::address(var_flex_edgeflexid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if flex_hessian_valid_in[worldid, f]:                                                  <L 1791>
        var_5 = wp::address(var_flex_hessian_valid_in, var_0, var_3);
        var_6 = wp::load(var_5);
        if (var_6) {
            // return                                                                             <L 1792>
            continue;
        }
        var_7 = wp::load(var_5);
        // if not _flex_hessian_active(                                                           <L 1793>
        // flex_dim,                                                                              <L 1794>
        // flex_interp,                                                                           <L 1795>
        // flex_stiffnessadr,                                                                     <L 1796>
        // flex_stiffness,                                                                        <L 1797>
        // flex_rigid,                                                                            <L 1798>
        // f,                                                                                     <L 1799>
        var_8 = _flex_hessian_active_0(var_flex_dim, var_flex_interp, var_flex_stiffnessadr, var_flex_stiffness, var_flex_rigid, var_3);
        var_9 = wp::unot(var_8);
        if (var_9) {
            // return                                                                             <L 1801>
            continue;
        }
        // va = flex_vertadr[f]                                                                   <L 1803>
        var_10 = wp::address(var_flex_vertadr, var_3);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // vert = flex_edge[edgeid]                                                               <L 1804>
        var_13 = wp::address(var_flex_edge, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // block = flexedge_hessian_in[worldid, edgeid]                                           <L 1805>
        var_16 = wp::address(var_flexedge_hessian_in, var_0, var_1);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // d0 = vec6(-block[0, 0], -block[0, 1], -block[0, 2], -block[1, 1], -block[1, 2], -block[2, 2])       <L 1806>
        var_21 = wp::extract(var_17, var_19, var_20);
        var_22 = wp::neg(var_21);
        var_25 = wp::extract(var_17, var_23, var_24);
        var_26 = wp::neg(var_25);
        var_29 = wp::extract(var_17, var_27, var_28);
        var_30 = wp::neg(var_29);
        var_33 = wp::extract(var_17, var_31, var_32);
        var_34 = wp::neg(var_33);
        var_37 = wp::extract(var_17, var_35, var_36);
        var_38 = wp::neg(var_37);
        var_41 = wp::extract(var_17, var_39, var_40);
        var_42 = wp::neg(var_41);
        var_43 = wp::vec_t<6, wp::float32>({var_22, var_26, var_30, var_34, var_38, var_42});
        // d1 = vec6(-block[0, 0], -block[1, 0], -block[2, 0], -block[1, 1], -block[2, 1], -block[2, 2])       <L 1807>
        var_46 = wp::extract(var_17, var_44, var_45);
        var_47 = wp::neg(var_46);
        var_50 = wp::extract(var_17, var_48, var_49);
        var_51 = wp::neg(var_50);
        var_54 = wp::extract(var_17, var_52, var_53);
        var_55 = wp::neg(var_54);
        var_58 = wp::extract(var_17, var_56, var_57);
        var_59 = wp::neg(var_58);
        var_62 = wp::extract(var_17, var_60, var_61);
        var_63 = wp::neg(var_62);
        var_66 = wp::extract(var_17, var_64, var_65);
        var_67 = wp::neg(var_66);
        var_68 = wp::vec_t<6, wp::float32>({var_47, var_51, var_55, var_59, var_63, var_67});
        // wp.atomic_add(flexvert_hessian_out, worldid, va + vert[0], d0)                         <L 1808>
        var_70 = wp::extract(var_14, var_69);
        var_71 = wp::add(var_11, var_70);
        var_72 = wp::atomic_add(var_flexvert_hessian_out, var_0, var_71, var_43);
        // wp.atomic_add(flexvert_hessian_out, worldid, va + vert[1], d1)                         <L 1809>
        var_74 = wp::extract(var_14, var_73);
        var_75 = wp::add(var_11, var_74);
        var_76 = wp::atomic_add(var_flexvert_hessian_out, var_0, var_75, var_68);
    }
}



extern "C" __global__ void _fluid_force_83bf3546_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::vec_t<3, wp::float32>> var_opt_wind,
    wp::array_t<wp::float32> var_opt_density,
    wp::array_t<wp::float32> var_opt_viscosity,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_geomnum,
    wp::array_t<wp::int32> var_body_geomadr,
    wp::array_t<wp::float32> var_body_mass,
    wp::array_t<wp::vec_t<3, wp::float32>> var_body_inertia,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::float32> var_geom_fluid,
    wp::array_t<bool> var_body_fluid_ellipsoid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_ximat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_fluid_applied_out)
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
        wp::vec_t<3, wp::float32> var_3;
        const wp::float32 var_4 = 0.0;
        wp::vec_t<3, wp::float32> var_5;
        wp::vec_t<6, wp::float32> var_6;
        const wp::int32 var_7 = 0;
        bool var_8;
        wp::shape_t* var_9;
        const wp::int32 var_10 = 0;
        wp::int32 var_11;
        wp::shape_t var_12;
        wp::int32 var_13;
        wp::float32* var_14;
        wp::float32 var_15;
        wp::float32 var_16;
        const wp::float32 var_17 = 1e-15;
        bool var_18;
        wp::shape_t* var_19;
        const wp::int32 var_20 = 0;
        wp::int32 var_21;
        wp::shape_t var_22;
        wp::int32 var_23;
        wp::vec_t<3, wp::float32>* var_24;
        wp::vec_t<3, wp::float32> var_25;
        wp::vec_t<3, wp::float32> var_26;
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
        wp::vec_t<3, wp::float32>* var_43;
        wp::vec_t<3, wp::float32> var_44;
        wp::vec_t<3, wp::float32> var_45;
        wp::mat_t<3, 3, wp::float32>* var_46;
        wp::mat_t<3, 3, wp::float32> var_47;
        wp::mat_t<3, 3, wp::float32> var_48;
        wp::mat_t<3, 3, wp::float32> var_49;
        wp::vec_t<6, wp::float32>* var_50;
        wp::vec_t<6, wp::float32> var_51;
        wp::vec_t<6, wp::float32> var_52;
        wp::vec_t<3, wp::float32> var_53;
        wp::vec_t<3, wp::float32> var_54;
        wp::int32* var_55;
        wp::vec_t<3, wp::float32>* var_56;
        wp::int32 var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::vec_t<3, wp::float32> var_60;
        wp::vec_t<3, wp::float32> var_61;
        wp::vec_t<3, wp::float32> var_62;
        bool* var_63;
        bool var_64;
        const wp::float32 var_65 = 0.0;
        wp::vec_t<3, wp::float32> var_66;
        const wp::float32 var_67 = 0.0;
        wp::vec_t<3, wp::float32> var_68;
        wp::int32* var_69;
        wp::int32 var_70;
        wp::int32 var_71;
        wp::int32* var_72;
        wp::int32 var_73;
        wp::int32 var_74;
        wp::range_t var_75;
        wp::int32 var_76;
        wp::int32 var_77;
        const wp::int32 var_78 = 0;
        wp::float32* var_79;
        wp::float32 var_80;
        wp::float32 var_81;
        const wp::float32 var_82 = 0.0;
        bool var_83;
        wp::shape_t* var_84;
        const wp::int32 var_85 = 0;
        wp::int32 var_86;
        wp::shape_t var_87;
        wp::int32 var_88;
        wp::vec_t<3, wp::float32>* var_89;
        wp::vec_t<3, wp::float32> var_90;
        wp::vec_t<3, wp::float32> var_91;
        wp::int32* var_92;
        wp::vec_t<3, wp::float32> var_93;
        wp::int32 var_94;
        wp::mat_t<3, 3, wp::float32>* var_95;
        wp::mat_t<3, 3, wp::float32> var_96;
        wp::mat_t<3, 3, wp::float32> var_97;
        wp::mat_t<3, 3, wp::float32> var_98;
        wp::vec_t<3, wp::float32>* var_99;
        wp::vec_t<3, wp::float32> var_100;
        wp::vec_t<3, wp::float32> var_101;
        wp::vec_t<3, wp::float32> var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::vec_t<3, wp::float32> var_104;
        wp::vec_t<3, wp::float32> var_105;
        wp::vec_t<3, wp::float32> var_106;
        bool var_107;
        const wp::int32 var_108 = 0;
        wp::float32 var_109;
        const wp::int32 var_110 = 1;
        wp::float32 var_111;
        const wp::int32 var_112 = 2;
        wp::float32 var_113;
        wp::vec_t<3, wp::float32> var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::vec_t<3, wp::float32> var_116;
        const wp::float32 var_117 = 0.0;
        wp::vec_t<3, wp::float32> var_118;
        const wp::float32 var_119 = 0.0;
        wp::vec_t<3, wp::float32> var_120;
        const wp::float32 var_121 = 0.0;
        bool var_122;
        const wp::int32 var_123 = 6;
        wp::float32* var_124;
        const wp::int32 var_125 = 7;
        wp::float32* var_126;
        const wp::int32 var_127 = 8;
        wp::float32* var_128;
        wp::vec_t<3, wp::float32> var_129;
        wp::float32 var_130;
        wp::float32 var_131;
        wp::float32 var_132;
        const wp::int32 var_133 = 9;
        wp::float32* var_134;
        const wp::int32 var_135 = 10;
        wp::float32* var_136;
        const wp::int32 var_137 = 11;
        wp::float32* var_138;
        wp::vec_t<3, wp::float32> var_139;
        wp::float32 var_140;
        wp::float32 var_141;
        wp::float32 var_142;
        const wp::int32 var_143 = 0;
        wp::float32 var_144;
        wp::float32 var_145;
        const wp::int32 var_146 = 0;
        wp::float32 var_147;
        wp::float32 var_148;
        const wp::int32 var_149 = 1;
        wp::float32 var_150;
        wp::float32 var_151;
        const wp::int32 var_152 = 1;
        wp::float32 var_153;
        wp::float32 var_154;
        const wp::int32 var_155 = 2;
        wp::float32 var_156;
        wp::float32 var_157;
        const wp::int32 var_158 = 2;
        wp::float32 var_159;
        wp::float32 var_160;
        wp::vec_t<3, wp::float32> var_161;
        const wp::int32 var_162 = 0;
        wp::float32 var_163;
        wp::float32 var_164;
        const wp::int32 var_165 = 0;
        wp::float32 var_166;
        wp::float32 var_167;
        const wp::int32 var_168 = 1;
        wp::float32 var_169;
        wp::float32 var_170;
        const wp::int32 var_171 = 1;
        wp::float32 var_172;
        wp::float32 var_173;
        const wp::int32 var_174 = 2;
        wp::float32 var_175;
        wp::float32 var_176;
        const wp::int32 var_177 = 2;
        wp::float32 var_178;
        wp::float32 var_179;
        wp::vec_t<3, wp::float32> var_180;
        wp::vec_t<3, wp::float32> var_181;
        wp::vec_t<3, wp::float32> var_182;
        wp::vec_t<3, wp::float32> var_183;
        wp::vec_t<3, wp::float32> var_184;
        wp::vec_t<3, wp::float32> var_185;
        wp::vec_t<3, wp::float32> var_186;
        wp::vec_t<3, wp::float32> var_187;
        wp::vec_t<3, wp::float32> var_188;
        const wp::int32 var_189 = 5;
        wp::float32* var_190;
        wp::float32 var_191;
        wp::float32 var_192;
        const wp::int32 var_193 = 4;
        wp::float32* var_194;
        wp::float32 var_195;
        wp::float32 var_196;
        const wp::int32 var_197 = 1;
        wp::float32* var_198;
        wp::float32 var_199;
        wp::float32 var_200;
        const wp::int32 var_201 = 2;
        wp::float32* var_202;
        wp::float32 var_203;
        wp::float32 var_204;
        const wp::int32 var_205 = 3;
        wp::float32* var_206;
        wp::float32 var_207;
        wp::float32 var_208;
        const wp::float32 var_209 = 4.1887902047863905;
        const wp::int32 var_210 = 0;
        wp::float32 var_211;
        wp::float32 var_212;
        const wp::int32 var_213 = 1;
        wp::float32 var_214;
        wp::float32 var_215;
        const wp::int32 var_216 = 2;
        wp::float32 var_217;
        wp::float32 var_218;
        const wp::int32 var_219 = 0;
        wp::float32 var_220;
        const wp::int32 var_221 = 1;
        wp::float32 var_222;
        wp::float32 var_223;
        const wp::int32 var_224 = 2;
        wp::float32 var_225;
        wp::float32 var_226;
        const wp::int32 var_227 = 0;
        wp::float32 var_228;
        const wp::int32 var_229 = 1;
        wp::float32 var_230;
        wp::float32 var_231;
        const wp::int32 var_232 = 2;
        wp::float32 var_233;
        wp::float32 var_234;
        const wp::int32 var_235 = 0;
        wp::float32 var_236;
        const wp::int32 var_237 = 1;
        wp::float32 var_238;
        wp::float32 var_239;
        const wp::int32 var_240 = 2;
        wp::float32 var_241;
        wp::float32 var_242;
        wp::float32 var_243;
        wp::float32 var_244;
        const wp::float32 var_245 = 3.141592653589793;
        wp::float32 var_246;
        wp::float32 var_247;
        wp::float32 var_248;
        bool var_249;
        const wp::float32 var_250 = 1.0;
        wp::float32 var_251;
        const wp::float32 var_252 = 0.0;
        wp::float32 var_253;
        wp::vec_t<3, wp::float32> var_254;
        bool var_255;
        const wp::float32 var_256 = 1.0;
        wp::float32 var_257;
        const wp::float32 var_258 = 0.0;
        wp::float32 var_259;
        wp::vec_t<3, wp::float32> var_260;
        wp::vec_t<3, wp::float32> var_261;
        wp::float32 var_262;
        wp::float32 var_263;
        wp::vec_t<3, wp::float32> var_264;
        const wp::int32 var_265 = 1;
        wp::float32 var_266;
        const wp::int32 var_267 = 2;
        wp::float32 var_268;
        wp::float32 var_269;
        wp::float32 var_270;
        const wp::int32 var_271 = 2;
        wp::float32 var_272;
        const wp::int32 var_273 = 0;
        wp::float32 var_274;
        wp::float32 var_275;
        wp::float32 var_276;
        const wp::int32 var_277 = 0;
        wp::float32 var_278;
        const wp::int32 var_279 = 1;
        wp::float32 var_280;
        wp::float32 var_281;
        wp::float32 var_282;
        const wp::int32 var_283 = 0;
        wp::float32 var_284;
        wp::float32 var_285;
        wp::float32 var_286;
        const wp::int32 var_287 = 1;
        wp::float32 var_288;
        wp::float32 var_289;
        wp::float32 var_290;
        wp::float32 var_291;
        const wp::int32 var_292 = 2;
        wp::float32 var_293;
        wp::float32 var_294;
        wp::float32 var_295;
        wp::float32 var_296;
        bool var_297;
        const wp::float32 var_298 = 1.0;
        wp::float32 var_299;
        const wp::float32 var_300 = 0.0;
        wp::float32 var_301;
        wp::float32 var_302;
        wp::float32 var_303;
        wp::float32 var_304;
        wp::float32 var_305;
        const wp::int32 var_306 = 0;
        wp::float32 var_307;
        wp::float32 var_308;
        const wp::int32 var_309 = 0;
        wp::float32 var_310;
        wp::float32 var_311;
        wp::float32 var_312;
        const wp::int32 var_313 = 1;
        wp::float32 var_314;
        wp::float32 var_315;
        const wp::int32 var_316 = 1;
        wp::float32 var_317;
        wp::float32 var_318;
        wp::float32 var_319;
        wp::float32 var_320;
        const wp::int32 var_321 = 2;
        wp::float32 var_322;
        wp::float32 var_323;
        const wp::int32 var_324 = 2;
        wp::float32 var_325;
        wp::float32 var_326;
        wp::float32 var_327;
        const wp::float32 var_328 = 3.141592653589793;
        wp::float32 var_329;
        wp::float32 var_330;
        wp::float32 var_331;
        wp::float32 var_332;
        wp::float32 var_333;
        bool var_334;
        const wp::float32 var_335 = 1.0;
        wp::float32 var_336;
        const wp::float32 var_337 = 0.0;
        wp::float32 var_338;
        const wp::int32 var_339 = 0;
        wp::float32 var_340;
        wp::float32 var_341;
        const wp::int32 var_342 = 1;
        wp::float32 var_343;
        wp::float32 var_344;
        const wp::int32 var_345 = 2;
        wp::float32 var_346;
        wp::float32 var_347;
        wp::vec_t<3, wp::float32> var_348;
        const wp::float32 var_349 = 0.0;
        wp::vec_t<3, wp::float32> var_350;
        bool var_351;
        const wp::float32 var_352 = 0.0;
        bool var_353;
        const wp::float32 var_354 = 0.0;
        bool var_355;
        bool var_356;
        wp::vec_t<3, wp::float32> var_357;
        wp::float32 var_358;
        wp::float32 var_359;
        wp::float32 var_360;
        wp::vec_t<3, wp::float32> var_361;
        wp::vec_t<3, wp::float32> var_362;
        wp::vec_t<3, wp::float32> var_363;
        const wp::float32 var_364 = 0.6666666666666666;
        const wp::int32 var_365 = 0;
        wp::float32 var_366;
        const wp::int32 var_367 = 1;
        wp::float32 var_368;
        wp::float32 var_369;
        const wp::int32 var_370 = 2;
        wp::float32 var_371;
        wp::float32 var_372;
        wp::float32 var_373;
        const wp::float32 var_374 = 9.42477796076938;
        wp::float32 var_375;
        const wp::float32 var_376 = 3.141592653589793;
        wp::float32 var_377;
        wp::float32 var_378;
        wp::float32 var_379;
        const wp::float32 var_380 = 1.6755160819145563;
        wp::float32 var_381;
        wp::float32 var_382;
        wp::float32 var_383;
        const wp::int32 var_384 = 0;
        wp::float32 var_385;
        const wp::int32 var_386 = 1;
        wp::float32 var_387;
        const wp::int32 var_388 = 2;
        wp::float32 var_389;
        const wp::int32 var_390 = 0;
        wp::float32 var_391;
        wp::float32 var_392;
        wp::float32 var_393;
        wp::float32 var_394;
        wp::float32 var_395;
        wp::float32 var_396;
        const wp::int32 var_397 = 1;
        wp::float32 var_398;
        wp::float32 var_399;
        wp::float32 var_400;
        wp::float32 var_401;
        wp::float32 var_402;
        wp::float32 var_403;
        const wp::int32 var_404 = 2;
        wp::float32 var_405;
        wp::float32 var_406;
        wp::float32 var_407;
        wp::float32 var_408;
        wp::float32 var_409;
        wp::float32 var_410;
        wp::vec_t<3, wp::float32> var_411;
        wp::float32 var_412;
        wp::float32 var_413;
        wp::float32 var_414;
        wp::float32 var_415;
        wp::float32 var_416;
        wp::float32 var_417;
        wp::float32 var_418;
        wp::float32 var_419;
        wp::float32 var_420;
        wp::float32 var_421;
        wp::float32 var_422;
        wp::float32 var_423;
        wp::vec_t<3, wp::float32> var_424;
        wp::vec_t<3, wp::float32> var_425;
        wp::vec_t<3, wp::float32> var_426;
        wp::vec_t<3, wp::float32> var_427;
        wp::vec_t<3, wp::float32> var_428;
        wp::vec_t<3, wp::float32> var_429;
        wp::vec_t<3, wp::float32> var_430;
        wp::vec_t<3, wp::float32> var_431;
        wp::vec_t<3, wp::float32> var_432;
        wp::vec_t<3, wp::float32> var_433;
        wp::vec_t<3, wp::float32> var_434;
        wp::vec_t<3, wp::float32> var_435;
        wp::vec_t<6, wp::float32> var_436;
        bool var_437;
        wp::vec_t<3, wp::float32> var_438;
        wp::vec_t<3, wp::float32> var_439;
        bool var_440;
        const wp::int32 var_441 = 0;
        wp::float32 var_442;
        const wp::int32 var_443 = 1;
        wp::float32 var_444;
        const wp::int32 var_445 = 2;
        wp::float32 var_446;
        wp::vec_t<3, wp::float32> var_447;
        wp::vec_t<3, wp::float32> var_448;
        wp::vec_t<3, wp::float32> var_449;
        const wp::float32 var_450 = 0.0;
        wp::vec_t<3, wp::float32> var_451;
        const wp::float32 var_452 = 0.0;
        wp::vec_t<3, wp::float32> var_453;
        const wp::float32 var_454 = 0.0;
        bool var_455;
        const wp::float32 var_456 = 0.0;
        bool var_457;
        bool var_458;
        wp::shape_t* var_459;
        const wp::int32 var_460 = 0;
        wp::int32 var_461;
        wp::shape_t var_462;
        wp::int32 var_463;
        wp::vec_t<3, wp::float32>* var_464;
        wp::vec_t<3, wp::float32> var_465;
        wp::vec_t<3, wp::float32> var_466;
        wp::shape_t* var_467;
        const wp::int32 var_468 = 0;
        wp::int32 var_469;
        wp::shape_t var_470;
        wp::int32 var_471;
        wp::float32* var_472;
        wp::float32 var_473;
        wp::float32 var_474;
        const wp::float32 var_475 = 6.0;
        wp::float32 var_476;
        const wp::int32 var_477 = 1;
        wp::float32 var_478;
        const wp::int32 var_479 = 2;
        wp::float32 var_480;
        wp::float32 var_481;
        const wp::int32 var_482 = 0;
        wp::float32 var_483;
        wp::float32 var_484;
        wp::float32 var_485;
        wp::float32 var_486;
        wp::float32 var_487;
        const wp::int32 var_488 = 0;
        wp::float32 var_489;
        const wp::int32 var_490 = 2;
        wp::float32 var_491;
        wp::float32 var_492;
        const wp::int32 var_493 = 1;
        wp::float32 var_494;
        wp::float32 var_495;
        wp::float32 var_496;
        wp::float32 var_497;
        wp::float32 var_498;
        const wp::int32 var_499 = 0;
        wp::float32 var_500;
        const wp::int32 var_501 = 1;
        wp::float32 var_502;
        wp::float32 var_503;
        const wp::int32 var_504 = 2;
        wp::float32 var_505;
        wp::float32 var_506;
        wp::float32 var_507;
        wp::float32 var_508;
        wp::float32 var_509;
        wp::float32 var_510;
        wp::float32 var_511;
        wp::float32 var_512;
        const wp::float32 var_513 = 3.0;
        wp::float32 var_514;
        wp::vec_t<3, wp::float32> var_515;
        const wp::float32 var_516 = 3.0;
        wp::float32 var_517;
        wp::vec_t<3, wp::float32> var_518;
        const wp::float32 var_519 = 3.141592653589793;
        wp::vec_t<3, wp::float32> var_520;
        wp::vec_t<3, wp::float32> var_521;
        const wp::float32 var_522 = -3.0;
        wp::vec_t<3, wp::float32> var_523;
        wp::vec_t<3, wp::float32> var_524;
        const wp::float32 var_525 = 3.141592653589793;
        wp::vec_t<3, wp::float32> var_526;
        wp::vec_t<3, wp::float32> var_527;
        wp::vec_t<3, wp::float32> var_528;
        wp::vec_t<3, wp::float32> var_529;
        const wp::float32 var_530 = 0.5;
        wp::float32 var_531;
        wp::float32 var_532;
        wp::float32 var_533;
        const wp::int32 var_534 = 0;
        wp::float32 var_535;
        wp::float32 var_536;
        wp::float32 var_537;
        const wp::int32 var_538 = 0;
        wp::float32 var_539;
        wp::float32 var_540;
        const wp::float32 var_541 = 0.5;
        wp::float32 var_542;
        wp::float32 var_543;
        wp::float32 var_544;
        const wp::int32 var_545 = 1;
        wp::float32 var_546;
        wp::float32 var_547;
        wp::float32 var_548;
        const wp::int32 var_549 = 1;
        wp::float32 var_550;
        wp::float32 var_551;
        const wp::float32 var_552 = 0.5;
        wp::float32 var_553;
        wp::float32 var_554;
        wp::float32 var_555;
        const wp::int32 var_556 = 2;
        wp::float32 var_557;
        wp::float32 var_558;
        wp::float32 var_559;
        const wp::int32 var_560 = 2;
        wp::float32 var_561;
        wp::float32 var_562;
        wp::vec_t<3, wp::float32> var_563;
        wp::vec_t<3, wp::float32> var_564;
        const wp::float32 var_565 = 64.0;
        wp::float32 var_566;
        const wp::float32 var_567 = 4.0;
        wp::float32 var_568;
        const wp::float32 var_569 = 4.0;
        wp::float32 var_570;
        const wp::float32 var_571 = 4.0;
        wp::float32 var_572;
        wp::float32 var_573;
        wp::float32 var_574;
        const wp::int32 var_575 = 0;
        wp::float32 var_576;
        wp::float32 var_577;
        wp::float32 var_578;
        const wp::int32 var_579 = 0;
        wp::float32 var_580;
        wp::float32 var_581;
        wp::float32 var_582;
        wp::float32 var_583;
        wp::float32 var_584;
        const wp::int32 var_585 = 1;
        wp::float32 var_586;
        wp::float32 var_587;
        wp::float32 var_588;
        const wp::int32 var_589 = 1;
        wp::float32 var_590;
        wp::float32 var_591;
        wp::float32 var_592;
        wp::float32 var_593;
        wp::float32 var_594;
        const wp::int32 var_595 = 2;
        wp::float32 var_596;
        wp::float32 var_597;
        wp::float32 var_598;
        const wp::int32 var_599 = 2;
        wp::float32 var_600;
        wp::float32 var_601;
        wp::float32 var_602;
        wp::vec_t<3, wp::float32> var_603;
        wp::vec_t<3, wp::float32> var_604;
        wp::vec_t<3, wp::float32> var_605;
        wp::vec_t<3, wp::float32> var_606;
        wp::float32 var_607;
        wp::vec_t<3, wp::float32> var_608;
        wp::vec_t<3, wp::float32> var_609;
        wp::vec_t<6, wp::float32> var_610;
        //---------
        // forward
        // def _fluid_force(                                                                      <L 359>
        // worldid, bodyid = wp.tid()                                                             <L 384>
        builtin_tid2d(var_0, var_1);
        // zero_force = wp.spatial_vector(wp.vec3(0.0), wp.vec3(0.0))                             <L 385>
        var_3 = wp::vec_t<3, wp::float32>(var_2);
        var_5 = wp::vec_t<3, wp::float32>(var_4);
        var_6 = wp::vec_t<6, wp::float32>(var_3, var_5);
        // if bodyid == 0:                                                                        <L 387>
        var_8 = (var_1 == var_7);
        if (var_8) {
            // fluid_applied_out[worldid, bodyid] = zero_force                                    <L 388>
            wp::array_store(var_fluid_applied_out, var_0, var_1, var_6);
            // return                                                                             <L 389>
            continue;
        }
        // mass = body_mass[worldid % body_mass.shape[0], bodyid]                                 <L 392>
        var_9 = &(var_body_mass.shape);
        var_12 = wp::load(var_9);
        var_11 = wp::extract(var_12, var_10);
        var_13 = wp::mod(var_0, var_11);
        var_14 = wp::address(var_body_mass, var_13, var_1);
        var_16 = wp::load(var_14);
        var_15 = wp::copy(var_16);
        // if mass < MJ_MINVAL:                                                                   <L 393>
        var_18 = (var_15 < var_17);
        if (var_18) {
            // fluid_applied_out[worldid, bodyid] = zero_force                                    <L 394>
            wp::array_store(var_fluid_applied_out, var_0, var_1, var_6);
            // return                                                                             <L 395>
            continue;
        }
        // wind = opt_wind[worldid % opt_wind.shape[0]]                                           <L 397>
        var_19 = &(var_opt_wind.shape);
        var_22 = wp::load(var_19);
        var_21 = wp::extract(var_22, var_20);
        var_23 = wp::mod(var_0, var_21);
        var_24 = wp::address(var_opt_wind, var_23);
        var_26 = wp::load(var_24);
        var_25 = wp::copy(var_26);
        // density = opt_density[worldid % opt_density.shape[0]]                                  <L 398>
        var_27 = &(var_opt_density.shape);
        var_30 = wp::load(var_27);
        var_29 = wp::extract(var_30, var_28);
        var_31 = wp::mod(var_0, var_29);
        var_32 = wp::address(var_opt_density, var_31);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // viscosity = opt_viscosity[worldid % opt_viscosity.shape[0]]                            <L 399>
        var_35 = &(var_opt_viscosity.shape);
        var_38 = wp::load(var_35);
        var_37 = wp::extract(var_38, var_36);
        var_39 = wp::mod(var_0, var_37);
        var_40 = wp::address(var_opt_viscosity, var_39);
        var_42 = wp::load(var_40);
        var_41 = wp::copy(var_42);
        // xipos = xipos_in[worldid, bodyid]                                                      <L 402>
        var_43 = wp::address(var_xipos_in, var_0, var_1);
        var_45 = wp::load(var_43);
        var_44 = wp::copy(var_45);
        // rot = ximat_in[worldid, bodyid]                                                        <L 403>
        var_46 = wp::address(var_ximat_in, var_0, var_1);
        var_48 = wp::load(var_46);
        var_47 = wp::copy(var_48);
        // rotT = wp.transpose(rot)                                                               <L 404>
        var_49 = wp::transpose(var_47);
        // cvel = cvel_in[worldid, bodyid]                                                        <L 405>
        var_50 = wp::address(var_cvel_in, var_0, var_1);
        var_52 = wp::load(var_50);
        var_51 = wp::copy(var_52);
        // ang_global = wp.spatial_top(cvel)                                                      <L 406>
        var_53 = wp::spatial_top(var_51);
        // lin_global = wp.spatial_bottom(cvel)                                                   <L 407>
        var_54 = wp::spatial_bottom(var_51);
        // subtree_root = subtree_com_in[worldid, body_rootid[bodyid]]                            <L 408>
        var_55 = wp::address(var_body_rootid, var_1);
        var_57 = wp::load(var_55);
        var_56 = wp::address(var_subtree_com_in, var_0, var_57);
        var_59 = wp::load(var_56);
        var_58 = wp::copy(var_59);
        // lin_com = lin_global - wp.cross(xipos - subtree_root, ang_global)                      <L 409>
        var_60 = wp::sub(var_44, var_58);
        var_61 = wp::cross(var_60, var_53);
        var_62 = wp::sub(var_54, var_61);
        // if body_fluid_ellipsoid[bodyid]:                                                       <L 411>
        var_63 = wp::address(var_body_fluid_ellipsoid, var_1);
        var_64 = wp::load(var_63);
        if (var_64) {
            // force_global = wp.vec3(0.0)                                                        <L 412>
            var_66 = wp::vec_t<3, wp::float32>(var_65);
            // torque_global = wp.vec3(0.0)                                                       <L 413>
            var_68 = wp::vec_t<3, wp::float32>(var_67);
            // start = body_geomadr[bodyid]                                                       <L 415>
            var_69 = wp::address(var_body_geomadr, var_1);
            var_71 = wp::load(var_69);
            var_70 = wp::copy(var_71);
            // count = body_geomnum[bodyid]                                                       <L 416>
            var_72 = wp::address(var_body_geomnum, var_1);
            var_74 = wp::load(var_72);
            var_73 = wp::copy(var_74);
            // for i in range(count):                                                             <L 418>
            var_75 = wp::range(var_73);
            start_for_2:;
                if (iter_cmp(var_75) == 0) goto end_for_2;
                var_76 = wp::iter_next(var_75);
                // geomid = start + i                                                             <L 419>
                var_77 = wp::add(var_70, var_76);
                // coef = geom_fluid[geomid, 0]                                                   <L 420>
                var_79 = wp::address(var_geom_fluid, var_77, var_78);
                var_81 = wp::load(var_79);
                var_80 = wp::copy(var_81);
                // if coef <= 0.0:                                                                <L 421>
                var_83 = (var_80 <= var_82);
                if (var_83) {
                    // continue                                                                   <L 422>
                    goto start_for_2;
                }
                // size = geom_size[worldid % geom_size.shape[0], geomid]                         <L 424>
                var_84 = &(var_geom_size.shape);
                var_87 = wp::load(var_84);
                var_86 = wp::extract(var_87, var_85);
                var_88 = wp::mod(var_0, var_86);
                var_89 = wp::address(var_geom_size, var_88, var_77);
                var_91 = wp::load(var_89);
                var_90 = wp::copy(var_91);
                // semiaxes = geom_semiaxes(size, geom_type[geomid])                              <L 425>
                var_92 = wp::address(var_geom_type, var_77);
                var_94 = wp::load(var_92);
                var_93 = geom_semiaxes_0(var_90, var_94);
                // geom_rot = geom_xmat_in[worldid, geomid]                                       <L 426>
                var_95 = wp::address(var_geom_xmat_in, var_0, var_77);
                var_97 = wp::load(var_95);
                var_96 = wp::copy(var_97);
                // geom_rotT = wp.transpose(geom_rot)                                             <L 427>
                var_98 = wp::transpose(var_96);
                // geom_pos = geom_xpos_in[worldid, geomid]                                       <L 428>
                var_99 = wp::address(var_geom_xpos_in, var_0, var_77);
                var_101 = wp::load(var_99);
                var_100 = wp::copy(var_101);
                // lin_point = lin_com + wp.cross(ang_global, geom_pos - xipos)                   <L 430>
                var_102 = wp::sub(var_100, var_44);
                var_103 = wp::cross(var_53, var_102);
                var_104 = wp::add(var_62, var_103);
                // l_ang = geom_rotT @ ang_global                                                 <L 432>
                var_105 = wp::mul(var_98, var_53);
                // l_lin = geom_rotT @ lin_point                                                  <L 433>
                var_106 = wp::mul(var_98, var_104);
                // if wind[0] or wind[1] or wind[2]:                                              <L 435>
                var_109 = wp::extract(var_25, var_108);
                var_107 = var_109;
                if (!var_107) {
                    var_111 = wp::extract(var_25, var_110);
                    var_107 = var_107 || var_111;
                }
                if (!var_107) {
                    var_113 = wp::extract(var_25, var_112);
                    var_107 = var_107 || var_113;
                }
                if (var_107) {
                    // l_lin -= geom_rotT @ wind                                                  <L 436>
                    var_114 = wp::mul(var_98, var_25);
                    var_115 = wp::sub(var_106, var_114);
                }
                var_116 = wp::where(var_107, var_115, var_106);
                // lfrc_torque = wp.vec3(0.0)                                                     <L 438>
                var_118 = wp::vec_t<3, wp::float32>(var_117);
                // lfrc_force = wp.vec3(0.0)                                                      <L 439>
                var_120 = wp::vec_t<3, wp::float32>(var_119);
                // if density > 0.0:                                                              <L 441>
                var_122 = (var_33 > var_121);
                if (var_122) {
                    // virtual_mass = wp.vec3(geom_fluid[geomid, 6], geom_fluid[geomid, 7], geom_fluid[geomid, 8])       <L 443>
                    var_124 = wp::address(var_geom_fluid, var_77, var_123);
                    var_126 = wp::address(var_geom_fluid, var_77, var_125);
                    var_128 = wp::address(var_geom_fluid, var_77, var_127);
                    var_130 = wp::load(var_124);
                    var_131 = wp::load(var_126);
                    var_132 = wp::load(var_128);
                    var_129 = wp::vec_t<3, wp::float32>(var_130, var_131, var_132);
                    // virtual_inertia = wp.vec3(geom_fluid[geomid, 9], geom_fluid[geomid, 10], geom_fluid[geomid, 11])       <L 444>
                    var_134 = wp::address(var_geom_fluid, var_77, var_133);
                    var_136 = wp::address(var_geom_fluid, var_77, var_135);
                    var_138 = wp::address(var_geom_fluid, var_77, var_137);
                    var_140 = wp::load(var_134);
                    var_141 = wp::load(var_136);
                    var_142 = wp::load(var_138);
                    var_139 = wp::vec_t<3, wp::float32>(var_140, var_141, var_142);
                    // virtual_lin_mom = wp.vec3(                                                 <L 446>
                    // density * virtual_mass[0] * l_lin[0],                                      <L 447>
                    var_144 = wp::extract(var_129, var_143);
                    var_145 = wp::mul(var_33, var_144);
                    var_147 = wp::extract(var_116, var_146);
                    var_148 = wp::mul(var_145, var_147);
                    // density * virtual_mass[1] * l_lin[1],                                      <L 448>
                    var_150 = wp::extract(var_129, var_149);
                    var_151 = wp::mul(var_33, var_150);
                    var_153 = wp::extract(var_116, var_152);
                    var_154 = wp::mul(var_151, var_153);
                    // density * virtual_mass[2] * l_lin[2],                                      <L 449>
                    var_156 = wp::extract(var_129, var_155);
                    var_157 = wp::mul(var_33, var_156);
                    var_159 = wp::extract(var_116, var_158);
                    var_160 = wp::mul(var_157, var_159);
                    var_161 = wp::vec_t<3, wp::float32>(var_148, var_154, var_160);
                    // virtual_ang_mom = wp.vec3(                                                 <L 451>
                    // density * virtual_inertia[0] * l_ang[0],                                   <L 452>
                    var_163 = wp::extract(var_139, var_162);
                    var_164 = wp::mul(var_33, var_163);
                    var_166 = wp::extract(var_105, var_165);
                    var_167 = wp::mul(var_164, var_166);
                    // density * virtual_inertia[1] * l_ang[1],                                   <L 453>
                    var_169 = wp::extract(var_139, var_168);
                    var_170 = wp::mul(var_33, var_169);
                    var_172 = wp::extract(var_105, var_171);
                    var_173 = wp::mul(var_170, var_172);
                    // density * virtual_inertia[2] * l_ang[2],                                   <L 454>
                    var_175 = wp::extract(var_139, var_174);
                    var_176 = wp::mul(var_33, var_175);
                    var_178 = wp::extract(var_105, var_177);
                    var_179 = wp::mul(var_176, var_178);
                    var_180 = wp::vec_t<3, wp::float32>(var_167, var_173, var_179);
                    // added_mass_force = wp.cross(virtual_lin_mom, l_ang)                        <L 457>
                    var_181 = wp::cross(var_161, var_105);
                    // added_mass_torque = wp.cross(virtual_lin_mom, l_lin) + wp.cross(virtual_ang_mom, l_ang)       <L 458>
                    var_182 = wp::cross(var_161, var_116);
                    var_183 = wp::cross(var_180, var_105);
                    var_184 = wp::add(var_182, var_183);
                    // lfrc_force += added_mass_force                                             <L 460>
                    var_185 = wp::add(var_120, var_181);
                    // lfrc_torque += added_mass_torque                                           <L 461>
                    var_186 = wp::add(var_118, var_184);
                }
                var_187 = wp::where(var_122, var_186, var_118);
                var_188 = wp::where(var_122, var_185, var_120);
                // magnus_coef = geom_fluid[geomid, 5]                                            <L 464>
                var_190 = wp::address(var_geom_fluid, var_77, var_189);
                var_192 = wp::load(var_190);
                var_191 = wp::copy(var_192);
                // kutta_coef = geom_fluid[geomid, 4]                                             <L 465>
                var_194 = wp::address(var_geom_fluid, var_77, var_193);
                var_196 = wp::load(var_194);
                var_195 = wp::copy(var_196);
                // blunt_drag_coef = geom_fluid[geomid, 1]                                        <L 466>
                var_198 = wp::address(var_geom_fluid, var_77, var_197);
                var_200 = wp::load(var_198);
                var_199 = wp::copy(var_200);
                // slender_drag_coef = geom_fluid[geomid, 2]                                      <L 467>
                var_202 = wp::address(var_geom_fluid, var_77, var_201);
                var_204 = wp::load(var_202);
                var_203 = wp::copy(var_204);
                // ang_drag_coef = geom_fluid[geomid, 3]                                          <L 468>
                var_206 = wp::address(var_geom_fluid, var_77, var_205);
                var_208 = wp::load(var_206);
                var_207 = wp::copy(var_208);
                // volume = wp.static(4.0 / 3.0 * wp.pi) * semiaxes[0] * semiaxes[1] * semiaxes[2]       <L 470>
                var_211 = wp::extract(var_93, var_210);
                var_212 = wp::mul(var_209, var_211);
                var_214 = wp::extract(var_93, var_213);
                var_215 = wp::mul(var_212, var_214);
                var_217 = wp::extract(var_93, var_216);
                var_218 = wp::mul(var_215, var_217);
                // d_max = wp.max(wp.max(semiaxes[0], semiaxes[1]), semiaxes[2])                  <L 471>
                var_220 = wp::extract(var_93, var_219);
                var_222 = wp::extract(var_93, var_221);
                var_223 = wp::max(var_220, var_222);
                var_225 = wp::extract(var_93, var_224);
                var_226 = wp::max(var_223, var_225);
                // d_min = wp.min(wp.min(semiaxes[0], semiaxes[1]), semiaxes[2])                  <L 472>
                var_228 = wp::extract(var_93, var_227);
                var_230 = wp::extract(var_93, var_229);
                var_231 = wp::min(var_228, var_230);
                var_233 = wp::extract(var_93, var_232);
                var_234 = wp::min(var_231, var_233);
                // d_mid = semiaxes[0] + semiaxes[1] + semiaxes[2] - d_max - d_min                <L 473>
                var_236 = wp::extract(var_93, var_235);
                var_238 = wp::extract(var_93, var_237);
                var_239 = wp::add(var_236, var_238);
                var_241 = wp::extract(var_93, var_240);
                var_242 = wp::add(var_239, var_241);
                var_243 = wp::sub(var_242, var_226);
                var_244 = wp::sub(var_243, var_234);
                // A_max = wp.pi * d_max * d_mid                                                  <L 474>
                var_246 = wp::mul(var_245, var_226);
                var_247 = wp::mul(var_246, var_244);
                // lin_speed = wp.length(l_lin)                                                   <L 476>
                var_248 = wp::length(var_116);
                // inv_speed = wp.where(lin_speed > MJ_MINVAL, 1.0 / lin_speed, 0.0)              <L 477>
                var_249 = (var_248 > var_17);
                var_251 = wp::div(var_250, var_248);
                var_253 = wp::where(var_249, var_251, var_252);
                // v = l_lin * inv_speed                                                          <L 478>
                var_254 = wp::mul(var_116, var_253);
                // inv_dmax = wp.where(d_max > MJ_MINVAL, 1.0 / d_max, 0.0)                       <L 479>
                var_255 = (var_226 > var_17);
                var_257 = wp::div(var_256, var_226);
                var_259 = wp::where(var_255, var_257, var_258);
                // s = semiaxes * inv_dmax                                                        <L 480>
                var_260 = wp::mul(var_93, var_259);
                // magnus_force = wp.cross(l_ang, l_lin) * (magnus_coef * density * volume)       <L 482>
                var_261 = wp::cross(var_105, var_116);
                var_262 = wp::mul(var_191, var_33);
                var_263 = wp::mul(var_262, var_218);
                var_264 = wp::mul(var_261, var_263);
                // s12_sq = _pow2(s[1] * s[2])                                                    <L 484>
                var_266 = wp::extract(var_260, var_265);
                var_268 = wp::extract(var_260, var_267);
                var_269 = wp::mul(var_266, var_268);
                var_270 = _pow2_0(var_269);
                // s20_sq = _pow2(s[2] * s[0])                                                    <L 485>
                var_272 = wp::extract(var_260, var_271);
                var_274 = wp::extract(var_260, var_273);
                var_275 = wp::mul(var_272, var_274);
                var_276 = _pow2_0(var_275);
                // s01_sq = _pow2(s[0] * s[1])                                                    <L 486>
                var_278 = wp::extract(var_260, var_277);
                var_280 = wp::extract(var_260, var_279);
                var_281 = wp::mul(var_278, var_280);
                var_282 = _pow2_0(var_281);
                // proj_num = s12_sq * _pow2(v[0]) + s20_sq * _pow2(v[1]) + s01_sq * _pow2(v[2])       <L 488>
                var_284 = wp::extract(var_254, var_283);
                var_285 = _pow2_0(var_284);
                var_286 = wp::mul(var_270, var_285);
                var_288 = wp::extract(var_254, var_287);
                var_289 = _pow2_0(var_288);
                var_290 = wp::mul(var_276, var_289);
                var_291 = wp::add(var_286, var_290);
                var_293 = wp::extract(var_254, var_292);
                var_294 = _pow2_0(var_293);
                var_295 = wp::mul(var_282, var_294);
                var_296 = wp::add(var_291, var_295);
                // inv_proj_num = wp.where(proj_num > MJ_MINVAL, 1.0 / proj_num, 0.0)             <L 489>
                var_297 = (var_296 > var_17);
                var_299 = wp::div(var_298, var_296);
                var_301 = wp::where(var_297, var_299, var_300);
                // a = s12_sq * inv_proj_num                                                      <L 490>
                var_302 = wp::mul(var_270, var_301);
                // b = s20_sq * inv_proj_num                                                      <L 491>
                var_303 = wp::mul(var_276, var_301);
                // c = s01_sq * inv_proj_num                                                      <L 492>
                var_304 = wp::mul(var_282, var_301);
                // proj_denom = a * a * v[0] * v[0] + b * b * v[1] * v[1] + c * c * v[2] * v[2]       <L 493>
                var_305 = wp::mul(var_302, var_302);
                var_307 = wp::extract(var_254, var_306);
                var_308 = wp::mul(var_305, var_307);
                var_310 = wp::extract(var_254, var_309);
                var_311 = wp::mul(var_308, var_310);
                var_312 = wp::mul(var_303, var_303);
                var_314 = wp::extract(var_254, var_313);
                var_315 = wp::mul(var_312, var_314);
                var_317 = wp::extract(var_254, var_316);
                var_318 = wp::mul(var_315, var_317);
                var_319 = wp::add(var_311, var_318);
                var_320 = wp::mul(var_304, var_304);
                var_322 = wp::extract(var_254, var_321);
                var_323 = wp::mul(var_320, var_322);
                var_325 = wp::extract(var_254, var_324);
                var_326 = wp::mul(var_323, var_325);
                var_327 = wp::add(var_319, var_326);
                // A_proj = wp.pi * d_max * d_max * wp.sqrt(proj_num * proj_denom)                <L 495>
                var_329 = wp::mul(var_328, var_226);
                var_330 = wp::mul(var_329, var_226);
                var_331 = wp::mul(var_296, var_327);
                var_332 = wp::sqrt(var_331);
                var_333 = wp::mul(var_330, var_332);
                // cos_alpha = wp.where(proj_denom > MJ_MINVAL, 1.0 / proj_denom, 0.0)            <L 496>
                var_334 = (var_327 > var_17);
                var_336 = wp::div(var_335, var_327);
                var_338 = wp::where(var_334, var_336, var_337);
                // norm = wp.vec3(a * v[0], b * v[1], c * v[2])                                   <L 498>
                var_340 = wp::extract(var_254, var_339);
                var_341 = wp::mul(var_302, var_340);
                var_343 = wp::extract(var_254, var_342);
                var_344 = wp::mul(var_303, var_343);
                var_346 = wp::extract(var_254, var_345);
                var_347 = wp::mul(var_304, var_346);
                var_348 = wp::vec_t<3, wp::float32>(var_341, var_344, var_347);
                // kutta_force = wp.vec3(0.0)                                                     <L 500>
                var_350 = wp::vec_t<3, wp::float32>(var_349);
                // if density > 0.0 and kutta_coef != 0.0 and lin_speed > MJ_MINVAL:              <L 501>
                var_353 = (var_33 > var_352);
                var_351 = var_353;
                if (var_351) {
                    var_355 = (var_195 != var_354);
                    var_351 = var_351 && var_355;
                }
                if (var_351) {
                    var_356 = (var_248 > var_17);
                    var_351 = var_351 && var_356;
                }
                if (var_351) {
                    // kutta_circ = wp.cross(norm, l_lin) * (kutta_coef * density * cos_alpha * A_proj)       <L 502>
                    var_357 = wp::cross(var_348, var_116);
                    var_358 = wp::mul(var_195, var_33);
                    var_359 = wp::mul(var_358, var_338);
                    var_360 = wp::mul(var_359, var_333);
                    var_361 = wp::mul(var_357, var_360);
                    // kutta_force = wp.cross(kutta_circ, l_lin)                                  <L 503>
                    var_362 = wp::cross(var_361, var_116);
                }
                var_363 = wp::where(var_351, var_362, var_350);
                // eq_sphere_D = wp.static(2.0 / 3.0) * (semiaxes[0] + semiaxes[1] + semiaxes[2])       <L 505>
                var_366 = wp::extract(var_93, var_365);
                var_368 = wp::extract(var_93, var_367);
                var_369 = wp::add(var_366, var_368);
                var_371 = wp::extract(var_93, var_370);
                var_372 = wp::add(var_369, var_371);
                var_373 = wp::mul(var_364, var_372);
                // lin_visc_force_coef = wp.static(3.0 * wp.pi) * eq_sphere_D                     <L 506>
                var_375 = wp::mul(var_374, var_373);
                // lin_visc_torq_coef = wp.pi * eq_sphere_D * eq_sphere_D * eq_sphere_D           <L 507>
                var_377 = wp::mul(var_376, var_373);
                var_378 = wp::mul(var_377, var_373);
                var_379 = wp::mul(var_378, var_373);
                // I_max = wp.static(8.0 / 15.0 * wp.pi) * d_mid * _pow4(d_max)                   <L 509>
                var_381 = wp::mul(var_380, var_244);
                var_382 = _pow4_0(var_226);
                var_383 = wp::mul(var_381, var_382);
                // II0 = ellipsoid_max_moment(semiaxes, 0)                                        <L 510>
                var_385 = ellipsoid_max_moment_0(var_93, var_384);
                // II1 = ellipsoid_max_moment(semiaxes, 1)                                        <L 511>
                var_387 = ellipsoid_max_moment_0(var_93, var_386);
                // II2 = ellipsoid_max_moment(semiaxes, 2)                                        <L 512>
                var_389 = ellipsoid_max_moment_0(var_93, var_388);
                // mom_visc = wp.vec3(                                                            <L 514>
                // l_ang[0] * (ang_drag_coef * II0 + slender_drag_coef * (I_max - II0)),          <L 515>
                var_391 = wp::extract(var_105, var_390);
                var_392 = wp::mul(var_207, var_385);
                var_393 = wp::sub(var_383, var_385);
                var_394 = wp::mul(var_203, var_393);
                var_395 = wp::add(var_392, var_394);
                var_396 = wp::mul(var_391, var_395);
                // l_ang[1] * (ang_drag_coef * II1 + slender_drag_coef * (I_max - II1)),          <L 516>
                var_398 = wp::extract(var_105, var_397);
                var_399 = wp::mul(var_207, var_387);
                var_400 = wp::sub(var_383, var_387);
                var_401 = wp::mul(var_203, var_400);
                var_402 = wp::add(var_399, var_401);
                var_403 = wp::mul(var_398, var_402);
                // l_ang[2] * (ang_drag_coef * II2 + slender_drag_coef * (I_max - II2)),          <L 517>
                var_405 = wp::extract(var_105, var_404);
                var_406 = wp::mul(var_207, var_389);
                var_407 = wp::sub(var_383, var_389);
                var_408 = wp::mul(var_203, var_407);
                var_409 = wp::add(var_406, var_408);
                var_410 = wp::mul(var_405, var_409);
                var_411 = wp::vec_t<3, wp::float32>(var_396, var_403, var_410);
                // drag_lin_coef = viscosity * lin_visc_force_coef + density * lin_speed * (       <L 520>
                var_412 = wp::mul(var_41, var_375);
                var_413 = wp::mul(var_33, var_248);
                // A_proj * blunt_drag_coef + slender_drag_coef * (A_max - A_proj)                <L 521>
                var_414 = wp::mul(var_333, var_199);
                var_415 = wp::sub(var_247, var_333);
                var_416 = wp::mul(var_203, var_415);
                var_417 = wp::add(var_414, var_416);
                var_418 = wp::mul(var_413, var_417);
                var_419 = wp::add(var_412, var_418);
                // drag_ang_coef = viscosity * lin_visc_torq_coef + density * wp.length(mom_visc)       <L 523>
                var_420 = wp::mul(var_41, var_379);
                var_421 = wp::length(var_411);
                var_422 = wp::mul(var_33, var_421);
                var_423 = wp::add(var_420, var_422);
                // lfrc_torque -= drag_ang_coef * l_ang                                           <L 525>
                var_424 = wp::mul(var_423, var_105);
                var_425 = wp::sub(var_187, var_424);
                // lfrc_force += magnus_force + kutta_force - drag_lin_coef * l_lin               <L 526>
                var_426 = wp::add(var_264, var_363);
                var_427 = wp::mul(var_419, var_116);
                var_428 = wp::sub(var_426, var_427);
                var_429 = wp::add(var_188, var_428);
                // lfrc_torque *= coef                                                            <L 528>
                var_430 = wp::mul(var_425, var_80);
                // lfrc_force *= coef                                                             <L 529>
                var_431 = wp::mul(var_429, var_80);
                // torque_global += geom_rot @ lfrc_torque                                        <L 532>
                var_432 = wp::mul(var_96, var_430);
                var_433 = wp::add(var_68, var_432);
                // force_global += geom_rot @ lfrc_force                                          <L 533>
                var_434 = wp::mul(var_96, var_431);
                var_435 = wp::add(var_66, var_434);
                wp::assign(var_66, var_435);
                wp::assign(var_68, var_433);
                goto start_for_2;
            end_for_2:;
            // fluid_applied_out[worldid, bodyid] = wp.spatial_vector(force_global, torque_global)       <L 535>
            var_436 = wp::vec_t<6, wp::float32>(var_66, var_68);
            wp::array_store(var_fluid_applied_out, var_0, var_1, var_436);
            // return                                                                             <L 536>
            continue;
        }
        var_437 = wp::load(var_63);
        // l_ang = rotT @ ang_global                                                              <L 538>
        var_438 = wp::mul(var_49, var_53);
        // l_lin = rotT @ lin_com                                                                 <L 539>
        var_439 = wp::mul(var_49, var_62);
        // if wind[0] or wind[1] or wind[2]:                                                      <L 541>
        var_442 = wp::extract(var_25, var_441);
        var_440 = var_442;
        if (!var_440) {
            var_444 = wp::extract(var_25, var_443);
            var_440 = var_440 || var_444;
        }
        if (!var_440) {
            var_446 = wp::extract(var_25, var_445);
            var_440 = var_440 || var_446;
        }
        if (var_440) {
            // l_lin -= rotT @ wind                                                               <L 542>
            var_447 = wp::mul(var_49, var_25);
            var_448 = wp::sub(var_439, var_447);
        }
        var_449 = wp::where(var_440, var_448, var_439);
        // lfrc_torque = wp.vec3(0.0)                                                             <L 544>
        var_451 = wp::vec_t<3, wp::float32>(var_450);
        // lfrc_force = wp.vec3(0.0)                                                              <L 545>
        var_453 = wp::vec_t<3, wp::float32>(var_452);
        // has_viscosity = viscosity > 0.0                                                        <L 547>
        var_455 = (var_41 > var_454);
        // has_density = density > 0.0                                                            <L 548>
        var_457 = (var_33 > var_456);
        // if has_viscosity or has_density:                                                       <L 550>
        var_458 = var_455;
        if (!var_458) {
            var_458 = var_458 || var_457;
        }
        if (var_458) {
            // inertia = body_inertia[worldid % body_inertia.shape[0], bodyid]                    <L 551>
            var_459 = &(var_body_inertia.shape);
            var_462 = wp::load(var_459);
            var_461 = wp::extract(var_462, var_460);
            var_463 = wp::mod(var_0, var_461);
            var_464 = wp::address(var_body_inertia, var_463, var_1);
            var_466 = wp::load(var_464);
            var_465 = wp::copy(var_466);
            // mass = body_mass[worldid % body_mass.shape[0], bodyid]                             <L 552>
            var_467 = &(var_body_mass.shape);
            var_470 = wp::load(var_467);
            var_469 = wp::extract(var_470, var_468);
            var_471 = wp::mod(var_0, var_469);
            var_472 = wp::address(var_body_mass, var_471, var_1);
            var_474 = wp::load(var_472);
            var_473 = wp::copy(var_474);
            // scl = 6.0 / mass                                                                   <L 553>
            var_476 = wp::div(var_475, var_473);
            // box0 = wp.sqrt(wp.max(MJ_MINVAL, inertia[1] + inertia[2] - inertia[0]) * scl)       <L 554>
            var_478 = wp::extract(var_465, var_477);
            var_480 = wp::extract(var_465, var_479);
            var_481 = wp::add(var_478, var_480);
            var_483 = wp::extract(var_465, var_482);
            var_484 = wp::sub(var_481, var_483);
            var_485 = wp::max(var_17, var_484);
            var_486 = wp::mul(var_485, var_476);
            var_487 = wp::sqrt(var_486);
            // box1 = wp.sqrt(wp.max(MJ_MINVAL, inertia[0] + inertia[2] - inertia[1]) * scl)       <L 555>
            var_489 = wp::extract(var_465, var_488);
            var_491 = wp::extract(var_465, var_490);
            var_492 = wp::add(var_489, var_491);
            var_494 = wp::extract(var_465, var_493);
            var_495 = wp::sub(var_492, var_494);
            var_496 = wp::max(var_17, var_495);
            var_497 = wp::mul(var_496, var_476);
            var_498 = wp::sqrt(var_497);
            // box2 = wp.sqrt(wp.max(MJ_MINVAL, inertia[0] + inertia[1] - inertia[2]) * scl)       <L 556>
            var_500 = wp::extract(var_465, var_499);
            var_502 = wp::extract(var_465, var_501);
            var_503 = wp::add(var_500, var_502);
            var_505 = wp::extract(var_465, var_504);
            var_506 = wp::sub(var_503, var_505);
            var_507 = wp::max(var_17, var_506);
            var_508 = wp::mul(var_507, var_476);
            var_509 = wp::sqrt(var_508);
        }
        var_510 = wp::where(var_458, var_473, var_15);
        // if has_viscosity:                                                                      <L 558>
        if (var_455) {
            // diam = (box0 + box1 + box2) / 3.0                                                  <L 559>
            var_511 = wp::add(var_487, var_498);
            var_512 = wp::add(var_511, var_509);
            var_514 = wp::div(var_512, var_513);
            // lfrc_torque = -l_ang * wp.pow(diam, 3.0) * wp.pi * viscosity                       <L 560>
            var_515 = wp::neg(var_438);
            var_517 = wp::pow(var_514, var_516);
            var_518 = wp::mul(var_515, var_517);
            var_520 = wp::mul(var_518, var_519);
            var_521 = wp::mul(var_520, var_41);
            // lfrc_force = -3.0 * l_lin * diam * wp.pi * viscosity                               <L 561>
            var_523 = wp::mul(var_522, var_449);
            var_524 = wp::mul(var_523, var_514);
            var_526 = wp::mul(var_524, var_525);
            var_527 = wp::mul(var_526, var_41);
        }
        var_528 = wp::where(var_455, var_521, var_451);
        var_529 = wp::where(var_455, var_527, var_453);
        // if has_density:                                                                        <L 563>
        if (var_457) {
            // lfrc_force -= wp.vec3(                                                             <L 564>
            // 0.5 * density * box1 * box2 * wp.abs(l_lin[0]) * l_lin[0],                         <L 565>
            var_531 = wp::mul(var_530, var_33);
            var_532 = wp::mul(var_531, var_498);
            var_533 = wp::mul(var_532, var_509);
            var_535 = wp::extract(var_449, var_534);
            var_536 = wp::abs(var_535);
            var_537 = wp::mul(var_533, var_536);
            var_539 = wp::extract(var_449, var_538);
            var_540 = wp::mul(var_537, var_539);
            // 0.5 * density * box0 * box2 * wp.abs(l_lin[1]) * l_lin[1],                         <L 566>
            var_542 = wp::mul(var_541, var_33);
            var_543 = wp::mul(var_542, var_487);
            var_544 = wp::mul(var_543, var_509);
            var_546 = wp::extract(var_449, var_545);
            var_547 = wp::abs(var_546);
            var_548 = wp::mul(var_544, var_547);
            var_550 = wp::extract(var_449, var_549);
            var_551 = wp::mul(var_548, var_550);
            // 0.5 * density * box0 * box1 * wp.abs(l_lin[2]) * l_lin[2],                         <L 567>
            var_553 = wp::mul(var_552, var_33);
            var_554 = wp::mul(var_553, var_487);
            var_555 = wp::mul(var_554, var_498);
            var_557 = wp::extract(var_449, var_556);
            var_558 = wp::abs(var_557);
            var_559 = wp::mul(var_555, var_558);
            var_561 = wp::extract(var_449, var_560);
            var_562 = wp::mul(var_559, var_561);
            var_563 = wp::vec_t<3, wp::float32>(var_540, var_551, var_562);
            // lfrc_force -= wp.vec3(                                                             <L 564>
            var_564 = wp::sub(var_529, var_563);
            // scl = density / 64.0                                                               <L 570>
            var_566 = wp::div(var_33, var_565);
            // box0_pow4 = wp.pow(box0, 4.0)                                                      <L 571>
            var_568 = wp::pow(var_487, var_567);
            // box1_pow4 = wp.pow(box1, 4.0)                                                      <L 572>
            var_570 = wp::pow(var_498, var_569);
            // box2_pow4 = wp.pow(box2, 4.0)                                                      <L 573>
            var_572 = wp::pow(var_509, var_571);
            // lfrc_torque -= wp.vec3(                                                            <L 574>
            // box0 * (box1_pow4 + box2_pow4) * wp.abs(l_ang[0]) * l_ang[0] * scl,                <L 575>
            var_573 = wp::add(var_570, var_572);
            var_574 = wp::mul(var_487, var_573);
            var_576 = wp::extract(var_438, var_575);
            var_577 = wp::abs(var_576);
            var_578 = wp::mul(var_574, var_577);
            var_580 = wp::extract(var_438, var_579);
            var_581 = wp::mul(var_578, var_580);
            var_582 = wp::mul(var_581, var_566);
            // box1 * (box0_pow4 + box2_pow4) * wp.abs(l_ang[1]) * l_ang[1] * scl,                <L 576>
            var_583 = wp::add(var_568, var_572);
            var_584 = wp::mul(var_498, var_583);
            var_586 = wp::extract(var_438, var_585);
            var_587 = wp::abs(var_586);
            var_588 = wp::mul(var_584, var_587);
            var_590 = wp::extract(var_438, var_589);
            var_591 = wp::mul(var_588, var_590);
            var_592 = wp::mul(var_591, var_566);
            // box2 * (box0_pow4 + box1_pow4) * wp.abs(l_ang[2]) * l_ang[2] * scl,                <L 577>
            var_593 = wp::add(var_568, var_570);
            var_594 = wp::mul(var_509, var_593);
            var_596 = wp::extract(var_438, var_595);
            var_597 = wp::abs(var_596);
            var_598 = wp::mul(var_594, var_597);
            var_600 = wp::extract(var_438, var_599);
            var_601 = wp::mul(var_598, var_600);
            var_602 = wp::mul(var_601, var_566);
            var_603 = wp::vec_t<3, wp::float32>(var_582, var_592, var_602);
            // lfrc_torque -= wp.vec3(                                                            <L 574>
            var_604 = wp::sub(var_528, var_603);
        }
        var_605 = wp::where(var_457, var_604, var_528);
        var_606 = wp::where(var_457, var_564, var_529);
        var_607 = wp::where(var_457, var_566, var_476);
        // torque_global = rot @ lfrc_torque                                                      <L 580>
        var_608 = wp::mul(var_47, var_605);
        // force_global = rot @ lfrc_force                                                        <L 581>
        var_609 = wp::mul(var_47, var_606);
        // fluid_applied_out[worldid, bodyid] = wp.spatial_vector(force_global, torque_global)       <L 583>
        var_610 = wp::vec_t<6, wp::float32>(var_609, var_608);
        wp::array_store(var_fluid_applied_out, var_0, var_1, var_610);
    }
}



extern "C" __global__ void _flex_bending_77a8ccd9_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_edgeadr,
    wp::array_t<wp::int32> var_flex_bendingadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_edge,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_edgeflap,
    wp::array_t<wp::float32> var_flex_bending,
    wp::array_t<wp::float32> var_flex_damping,
    wp::array_t<wp::int32> var_flex_edgeflexid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_spring_body_force_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_damper_body_force_out)
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
        const wp::int32 var_11 = 0;
        bool var_12;
        bool var_13;
        wp::int32* var_14;
        const wp::int32 var_15 = 2;
        bool var_16;
        wp::int32 var_17;
        wp::int32* var_18;
        const wp::int32 var_19 = 0;
        bool var_20;
        wp::int32 var_21;
        wp::vec_t<2, wp::int32>* var_22;
        wp::vec_t<2, wp::int32> var_23;
        wp::vec_t<2, wp::int32> var_24;
        const wp::int32 var_25 = 1;
        wp::int32 var_26;
        const wp::int32 var_27 = -1;
        bool var_28;
        bool var_29;
        wp::float32* var_30;
        wp::float32 var_31;
        wp::float32 var_32;
        bool var_33;
        bool var_34;
        const wp::float32 var_35 = 0.0;
        bool var_36;
        bool var_37;
        bool var_38;
        bool var_39;
        wp::vec_t<2, wp::int32>* var_40;
        wp::vec_t<2, wp::int32> var_41;
        wp::vec_t<2, wp::int32> var_42;
        wp::int32* var_43;
        wp::int32 var_44;
        wp::int32 var_45;
        const wp::int32 var_46 = 0;
        wp::int32 var_47;
        wp::int32 var_48;
        const wp::int32 var_49 = 1;
        wp::int32 var_50;
        wp::int32 var_51;
        const wp::int32 var_52 = 0;
        wp::int32 var_53;
        wp::int32 var_54;
        const wp::int32 var_55 = 1;
        wp::int32 var_56;
        wp::int32 var_57;
        wp::vec_t<4, wp::int32> var_58;
        wp::mat_t<4, 3, wp::float32> var_59;
        wp::vec_t<4, wp::int32> var_60;
        const wp::int32 var_61 = 0;
        const wp::int32 var_62 = 0;
        const wp::int32 var_63 = 0;
        const wp::int32 var_64 = 0;
        wp::vec_t<4, wp::int32> var_65;
        const wp::int32 var_66 = 0;
        wp::int32 var_67;
        wp::vec_t<3, wp::float32>* var_68;
        wp::vec_t<3, wp::float32> var_69;
        wp::int32 var_70;
        wp::int32* var_71;
        wp::int32 var_72;
        wp::int32 var_73;
        const wp::int32 var_74 = 0;
        bool var_75;
        wp::int32* var_76;
        wp::int32* var_77;
        wp::int32 var_78;
        const wp::int32 var_79 = 0;
        bool var_80;
        wp::int32 var_81;
        const wp::int32 var_82 = 1;
        const wp::int32 var_83 = 1;
        wp::int32 var_84;
        wp::vec_t<3, wp::float32>* var_85;
        wp::vec_t<3, wp::float32> var_86;
        wp::int32 var_87;
        wp::int32* var_88;
        wp::int32 var_89;
        wp::int32 var_90;
        const wp::int32 var_91 = 0;
        bool var_92;
        wp::int32* var_93;
        wp::int32* var_94;
        wp::int32 var_95;
        const wp::int32 var_96 = 0;
        bool var_97;
        wp::int32 var_98;
        const wp::int32 var_99 = 1;
        const wp::int32 var_100 = 2;
        wp::int32 var_101;
        wp::vec_t<3, wp::float32>* var_102;
        wp::vec_t<3, wp::float32> var_103;
        wp::int32 var_104;
        wp::int32* var_105;
        wp::int32 var_106;
        wp::int32 var_107;
        const wp::int32 var_108 = 0;
        bool var_109;
        wp::int32* var_110;
        wp::int32* var_111;
        wp::int32 var_112;
        const wp::int32 var_113 = 0;
        bool var_114;
        wp::int32 var_115;
        const wp::int32 var_116 = 1;
        const wp::int32 var_117 = 3;
        wp::int32 var_118;
        wp::vec_t<3, wp::float32>* var_119;
        wp::vec_t<3, wp::float32> var_120;
        wp::int32 var_121;
        wp::int32* var_122;
        wp::int32 var_123;
        wp::int32 var_124;
        const wp::int32 var_125 = 0;
        bool var_126;
        wp::int32* var_127;
        wp::int32* var_128;
        wp::int32 var_129;
        const wp::int32 var_130 = 0;
        bool var_131;
        wp::int32 var_132;
        const wp::int32 var_133 = 1;
        const wp::int32 var_134 = 17;
        wp::int32 var_135;
        wp::int32 var_136;
        const wp::int32 var_137 = 16;
        wp::int32 var_138;
        wp::float32* var_139;
        wp::float32 var_140;
        wp::float32 var_141;
        wp::mat_t<4, 3, wp::float32> var_142;
        bool var_143;
        const wp::float32 var_144 = 0.0;
        bool var_145;
        const wp::int32 var_146 = 1;
        wp::vec_t<3, wp::float32> var_147;
        const wp::int32 var_148 = 0;
        wp::vec_t<3, wp::float32> var_149;
        wp::vec_t<3, wp::float32> var_150;
        const wp::int32 var_151 = 2;
        wp::vec_t<3, wp::float32> var_152;
        const wp::int32 var_153 = 0;
        wp::vec_t<3, wp::float32> var_154;
        wp::vec_t<3, wp::float32> var_155;
        const wp::int32 var_156 = 3;
        wp::vec_t<3, wp::float32> var_157;
        const wp::int32 var_158 = 0;
        wp::vec_t<3, wp::float32> var_159;
        wp::vec_t<3, wp::float32> var_160;
        wp::vec_t<3, wp::float32> var_161;
        const wp::int32 var_162 = 1;
        wp::vec_t<3, wp::float32> var_163;
        const wp::int32 var_164 = 2;
        wp::vec_t<3, wp::float32> var_165;
        const wp::int32 var_166 = 3;
        const wp::int32 var_167 = 1;
        wp::vec_t<3, wp::float32> var_168;
        const wp::int32 var_169 = 2;
        wp::vec_t<3, wp::float32> var_170;
        wp::vec_t<3, wp::float32> var_171;
        const wp::int32 var_172 = 3;
        wp::vec_t<3, wp::float32> var_173;
        wp::vec_t<3, wp::float32> var_174;
        wp::vec_t<3, wp::float32> var_175;
        const wp::int32 var_176 = 0;
        wp::mat_t<4, 3, wp::float32> var_177;
        const wp::int32 var_178 = 0;
        wp::int32 var_179;
        wp::int32 var_180;
        wp::vec_t<6, wp::float32>* var_181;
        wp::vec_t<6, wp::float32> var_182;
        wp::vec_t<6, wp::float32> var_183;
        wp::vec_t<3, wp::float32> var_184;
        wp::vec_t<3, wp::float32> var_185;
        wp::int32* var_186;
        wp::vec_t<3, wp::float32>* var_187;
        wp::int32 var_188;
        wp::vec_t<3, wp::float32> var_189;
        wp::vec_t<3, wp::float32> var_190;
        wp::vec_t<3, wp::float32> var_191;
        wp::vec_t<3, wp::float32> var_192;
        wp::vec_t<3, wp::float32> var_193;
        wp::vec_t<3, wp::float32> var_194;
        wp::int32 var_195;
        const wp::int32 var_196 = 1;
        wp::int32 var_197;
        wp::int32 var_198;
        wp::vec_t<6, wp::float32>* var_199;
        wp::vec_t<6, wp::float32> var_200;
        wp::vec_t<6, wp::float32> var_201;
        wp::vec_t<3, wp::float32> var_202;
        wp::vec_t<3, wp::float32> var_203;
        wp::int32* var_204;
        wp::vec_t<3, wp::float32>* var_205;
        wp::int32 var_206;
        wp::vec_t<3, wp::float32> var_207;
        wp::vec_t<3, wp::float32> var_208;
        wp::vec_t<3, wp::float32> var_209;
        wp::vec_t<3, wp::float32> var_210;
        wp::vec_t<3, wp::float32> var_211;
        wp::vec_t<3, wp::float32> var_212;
        wp::int32 var_213;
        wp::vec_t<6, wp::float32> var_214;
        wp::vec_t<3, wp::float32> var_215;
        wp::vec_t<3, wp::float32> var_216;
        wp::vec_t<3, wp::float32> var_217;
        wp::vec_t<3, wp::float32> var_218;
        const wp::int32 var_219 = 2;
        wp::int32 var_220;
        wp::int32 var_221;
        wp::vec_t<6, wp::float32>* var_222;
        wp::vec_t<6, wp::float32> var_223;
        wp::vec_t<6, wp::float32> var_224;
        wp::vec_t<3, wp::float32> var_225;
        wp::vec_t<3, wp::float32> var_226;
        wp::int32* var_227;
        wp::vec_t<3, wp::float32>* var_228;
        wp::int32 var_229;
        wp::vec_t<3, wp::float32> var_230;
        wp::vec_t<3, wp::float32> var_231;
        wp::vec_t<3, wp::float32> var_232;
        wp::vec_t<3, wp::float32> var_233;
        wp::vec_t<3, wp::float32> var_234;
        wp::vec_t<3, wp::float32> var_235;
        wp::int32 var_236;
        wp::vec_t<6, wp::float32> var_237;
        wp::vec_t<3, wp::float32> var_238;
        wp::vec_t<3, wp::float32> var_239;
        wp::vec_t<3, wp::float32> var_240;
        wp::vec_t<3, wp::float32> var_241;
        const wp::int32 var_242 = 3;
        wp::int32 var_243;
        wp::int32 var_244;
        wp::vec_t<6, wp::float32>* var_245;
        wp::vec_t<6, wp::float32> var_246;
        wp::vec_t<6, wp::float32> var_247;
        wp::vec_t<3, wp::float32> var_248;
        wp::vec_t<3, wp::float32> var_249;
        wp::int32* var_250;
        wp::vec_t<3, wp::float32>* var_251;
        wp::int32 var_252;
        wp::vec_t<3, wp::float32> var_253;
        wp::vec_t<3, wp::float32> var_254;
        wp::vec_t<3, wp::float32> var_255;
        wp::vec_t<3, wp::float32> var_256;
        wp::vec_t<3, wp::float32> var_257;
        wp::vec_t<3, wp::float32> var_258;
        wp::int32 var_259;
        wp::vec_t<6, wp::float32> var_260;
        wp::vec_t<3, wp::float32> var_261;
        wp::vec_t<3, wp::float32> var_262;
        wp::vec_t<3, wp::float32> var_263;
        wp::vec_t<3, wp::float32> var_264;
        wp::int32 var_265;
        wp::int32 var_266;
        const wp::int32 var_267 = 4;
        wp::range_t var_268;
        wp::int32 var_269;
        wp::int32 var_270;
        bool var_271;
        const wp::float32 var_272 = 0.0;
        wp::vec_t<3, wp::float32> var_273;
        const wp::float32 var_274 = 0.0;
        wp::vec_t<3, wp::float32> var_275;
        const wp::int32 var_276 = 0;
        const wp::int32 var_277 = 4;
        wp::int32 var_278;
        wp::int32 var_279;
        wp::int32 var_280;
        wp::float32* var_281;
        wp::float32 var_282;
        wp::float32 var_283;
        wp::vec_t<3, wp::float32> var_284;
        wp::vec_t<3, wp::float32> var_285;
        wp::vec_t<3, wp::float32> var_286;
        wp::vec_t<3, wp::float32> var_287;
        wp::vec_t<3, wp::float32> var_288;
        wp::vec_t<3, wp::float32> var_289;
        wp::vec_t<3, wp::float32> var_290;
        wp::vec_t<3, wp::float32> var_291;
        const wp::int32 var_292 = 1;
        const wp::int32 var_293 = 4;
        wp::int32 var_294;
        wp::int32 var_295;
        wp::int32 var_296;
        wp::float32* var_297;
        wp::float32 var_298;
        wp::float32 var_299;
        wp::vec_t<3, wp::float32> var_300;
        wp::vec_t<3, wp::float32> var_301;
        wp::vec_t<3, wp::float32> var_302;
        wp::vec_t<3, wp::float32> var_303;
        wp::vec_t<3, wp::float32> var_304;
        wp::vec_t<3, wp::float32> var_305;
        wp::vec_t<3, wp::float32> var_306;
        wp::vec_t<3, wp::float32> var_307;
        const wp::int32 var_308 = 2;
        const wp::int32 var_309 = 4;
        wp::int32 var_310;
        wp::int32 var_311;
        wp::int32 var_312;
        wp::float32* var_313;
        wp::float32 var_314;
        wp::float32 var_315;
        wp::vec_t<3, wp::float32> var_316;
        wp::vec_t<3, wp::float32> var_317;
        wp::vec_t<3, wp::float32> var_318;
        wp::vec_t<3, wp::float32> var_319;
        wp::vec_t<3, wp::float32> var_320;
        wp::vec_t<3, wp::float32> var_321;
        wp::vec_t<3, wp::float32> var_322;
        wp::vec_t<3, wp::float32> var_323;
        const wp::int32 var_324 = 3;
        const wp::int32 var_325 = 4;
        wp::int32 var_326;
        wp::int32 var_327;
        wp::int32 var_328;
        wp::float32* var_329;
        wp::float32 var_330;
        wp::float32 var_331;
        wp::vec_t<3, wp::float32> var_332;
        wp::vec_t<3, wp::float32> var_333;
        wp::vec_t<3, wp::float32> var_334;
        wp::vec_t<3, wp::float32> var_335;
        wp::vec_t<3, wp::float32> var_336;
        wp::vec_t<3, wp::float32> var_337;
        wp::vec_t<3, wp::float32> var_338;
        wp::vec_t<3, wp::float32> var_339;
        wp::int32 var_340;
        wp::vec_t<3, wp::float32> var_341;
        wp::vec_t<3, wp::float32>* var_342;
        wp::vec_t<3, wp::float32> var_343;
        wp::vec_t<3, wp::float32> var_344;
        wp::vec_t<3, wp::float32> var_345;
        wp::vec_t<3, wp::float32> var_346;
        wp::vec_t<3, wp::float32> var_347;
        wp::vec_t<3, wp::float32> var_348;
        wp::vec_t<3, wp::float32> var_349;
        wp::vec_t<3, wp::float32> var_350;
        wp::vec_t<3, wp::float32> var_351;
        wp::vec_t<6, wp::float32> var_352;
        wp::vec_t<6, wp::float32> var_353;
        wp::vec_t<3, wp::float32> var_354;
        wp::vec_t<3, wp::float32> var_355;
        wp::vec_t<3, wp::float32> var_356;
        wp::vec_t<3, wp::float32> var_357;
        wp::vec_t<6, wp::float32> var_358;
        wp::vec_t<6, wp::float32> var_359;
        //---------
        // forward
        // def _flex_bending(                                                                     <L 2483>
        // worldid, edgeid = wp.tid()                                                             <L 2511>
        builtin_tid2d(var_0, var_1);
        // f = flex_edgeflexid[edgeid]                                                            <L 2513>
        var_2 = wp::address(var_flex_edgeflexid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // eid = edgeid - flex_edgeadr[f]                                                         <L 2514>
        var_5 = wp::address(var_flex_edgeadr, var_3);
        var_7 = wp::load(var_5);
        var_6 = wp::sub(var_1, var_7);
        // bendingadr = flex_bendingadr[f]                                                        <L 2516>
        var_8 = wp::address(var_flex_bendingadr, var_3);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // if bendingadr < 0:                                                                     <L 2517>
        var_12 = (var_9 < var_11);
        if (var_12) {
            // return                                                                             <L 2518>
            continue;
        }
        // if flex_dim[f] != 2 or flex_interp[f] != 0:                                            <L 2520>
        var_14 = wp::address(var_flex_dim, var_3);
        var_17 = wp::load(var_14);
        var_16 = (var_17 != var_15);
        var_13 = var_16;
        if (!var_13) {
            var_18 = wp::address(var_flex_interp, var_3);
            var_21 = wp::load(var_18);
            var_20 = (var_21 != var_19);
            var_13 = var_13 || var_20;
        }
        if (var_13) {
            // return                                                                             <L 2521>
            continue;
        }
        // flap = flex_edgeflap[edgeid]                                                           <L 2523>
        var_22 = wp::address(var_flex_edgeflap, var_1);
        var_24 = wp::load(var_22);
        var_23 = wp::copy(var_24);
        // if flap[1] == -1:                                                                      <L 2524>
        var_26 = wp::extract(var_23, var_25);
        var_28 = (var_26 == var_27);
        if (var_28) {
            // return                                                                             <L 2525>
            continue;
        }
        // has_spring = not dsbl_spring                                                           <L 2527>
        var_29 = wp::unot(var_dsbl_spring);
        // damping = flex_damping[f]                                                              <L 2528>
        var_30 = wp::address(var_flex_damping, var_3);
        var_32 = wp::load(var_30);
        var_31 = wp::copy(var_32);
        // has_damper = not dsbl_damper and damping > 0.0                                         <L 2529>
        var_34 = wp::unot(var_dsbl_damper);
        var_33 = var_34;
        if (var_33) {
            var_36 = (var_31 > var_35);
            var_33 = var_33 && var_36;
        }
        // if not has_spring and not has_damper:                                                  <L 2530>
        var_38 = wp::unot(var_29);
        var_37 = var_38;
        if (var_37) {
            var_39 = wp::unot(var_33);
            var_37 = var_37 && var_39;
        }
        if (var_37) {
            // return                                                                             <L 2531>
            continue;
        }
        // edge = flex_edge[edgeid]                                                               <L 2533>
        var_40 = wp::address(var_flex_edge, var_1);
        var_42 = wp::load(var_40);
        var_41 = wp::copy(var_42);
        // vertadr = flex_vertadr[f]                                                              <L 2534>
        var_43 = wp::address(var_flex_vertadr, var_3);
        var_45 = wp::load(var_43);
        var_44 = wp::copy(var_45);
        // v = wp.vec4i(                                                                          <L 2535>
        // vertadr + edge[0],                                                                     <L 2536>
        var_47 = wp::extract(var_41, var_46);
        var_48 = wp::add(var_44, var_47);
        // vertadr + edge[1],                                                                     <L 2537>
        var_50 = wp::extract(var_41, var_49);
        var_51 = wp::add(var_44, var_50);
        // vertadr + flap[0],                                                                     <L 2538>
        var_53 = wp::extract(var_23, var_52);
        var_54 = wp::add(var_44, var_53);
        // vertadr + flap[1],                                                                     <L 2539>
        var_56 = wp::extract(var_23, var_55);
        var_57 = wp::add(var_44, var_56);
        var_58 = wp::vec_t<4, wp::int32>(var_48, var_51, var_54, var_57);
        // vpos = mat43()                                                                         <L 2542>
        var_59 = wp::mat_t<4, 3, wp::float32>();
        // bodyids = wp.vec4i()                                                                   <L 2543>
        var_60 = wp::vec_t<4, wp::int32>();
        // has_dof = wp.vec4i(0, 0, 0, 0)                                                         <L 2544>
        var_65 = wp::vec_t<4, wp::int32>(var_61, var_62, var_63, var_64);
        // for j in range(4):                                                                     <L 2545>
        // vpos[j] = flexvert_xpos_in[worldid, v[j]]                                              <L 2546>
        var_67 = wp::extract(var_58, var_66);
        var_68 = wp::address(var_flexvert_xpos_in, var_0, var_67);
        var_69 = wp::load(var_68);
        wp::assign_inplace(var_59, var_66, var_69);
        // bodyid_j = flex_vertbodyid[v[j]]                                                       <L 2547>
        var_70 = wp::extract(var_58, var_66);
        var_71 = wp::address(var_flex_vertbodyid, var_70);
        var_73 = wp::load(var_71);
        var_72 = wp::copy(var_73);
        // bodyids[j] = bodyid_j                                                                  <L 2548>
        wp::assign_inplace(var_60, var_66, var_72);
        // if bodyid_j >= 0:                                                                      <L 2549>
        var_75 = (var_72 >= var_74);
        if (var_75) {
            // if body_dofnum[body_weldid[bodyid_j]] > 0:                                         <L 2550>
            var_76 = wp::address(var_body_weldid, var_72);
            var_78 = wp::load(var_76);
            var_77 = wp::address(var_body_dofnum, var_78);
            var_81 = wp::load(var_77);
            var_80 = (var_81 > var_79);
            if (var_80) {
                // has_dof[j] = 1                                                                 <L 2551>
                wp::assign_inplace(var_65, var_66, var_82);
            }
        }
        // vpos[j] = flexvert_xpos_in[worldid, v[j]]                                              <L 2546>
        var_84 = wp::extract(var_58, var_83);
        var_85 = wp::address(var_flexvert_xpos_in, var_0, var_84);
        var_86 = wp::load(var_85);
        wp::assign_inplace(var_59, var_83, var_86);
        // bodyid_j = flex_vertbodyid[v[j]]                                                       <L 2547>
        var_87 = wp::extract(var_58, var_83);
        var_88 = wp::address(var_flex_vertbodyid, var_87);
        var_90 = wp::load(var_88);
        var_89 = wp::copy(var_90);
        // bodyids[j] = bodyid_j                                                                  <L 2548>
        wp::assign_inplace(var_60, var_83, var_89);
        // if bodyid_j >= 0:                                                                      <L 2549>
        var_92 = (var_89 >= var_91);
        if (var_92) {
            // if body_dofnum[body_weldid[bodyid_j]] > 0:                                         <L 2550>
            var_93 = wp::address(var_body_weldid, var_89);
            var_95 = wp::load(var_93);
            var_94 = wp::address(var_body_dofnum, var_95);
            var_98 = wp::load(var_94);
            var_97 = (var_98 > var_96);
            if (var_97) {
                // has_dof[j] = 1                                                                 <L 2551>
                wp::assign_inplace(var_65, var_83, var_99);
            }
        }
        // vpos[j] = flexvert_xpos_in[worldid, v[j]]                                              <L 2546>
        var_101 = wp::extract(var_58, var_100);
        var_102 = wp::address(var_flexvert_xpos_in, var_0, var_101);
        var_103 = wp::load(var_102);
        wp::assign_inplace(var_59, var_100, var_103);
        // bodyid_j = flex_vertbodyid[v[j]]                                                       <L 2547>
        var_104 = wp::extract(var_58, var_100);
        var_105 = wp::address(var_flex_vertbodyid, var_104);
        var_107 = wp::load(var_105);
        var_106 = wp::copy(var_107);
        // bodyids[j] = bodyid_j                                                                  <L 2548>
        wp::assign_inplace(var_60, var_100, var_106);
        // if bodyid_j >= 0:                                                                      <L 2549>
        var_109 = (var_106 >= var_108);
        if (var_109) {
            // if body_dofnum[body_weldid[bodyid_j]] > 0:                                         <L 2550>
            var_110 = wp::address(var_body_weldid, var_106);
            var_112 = wp::load(var_110);
            var_111 = wp::address(var_body_dofnum, var_112);
            var_115 = wp::load(var_111);
            var_114 = (var_115 > var_113);
            if (var_114) {
                // has_dof[j] = 1                                                                 <L 2551>
                wp::assign_inplace(var_65, var_100, var_116);
            }
        }
        // vpos[j] = flexvert_xpos_in[worldid, v[j]]                                              <L 2546>
        var_118 = wp::extract(var_58, var_117);
        var_119 = wp::address(var_flexvert_xpos_in, var_0, var_118);
        var_120 = wp::load(var_119);
        wp::assign_inplace(var_59, var_117, var_120);
        // bodyid_j = flex_vertbodyid[v[j]]                                                       <L 2547>
        var_121 = wp::extract(var_58, var_117);
        var_122 = wp::address(var_flex_vertbodyid, var_121);
        var_124 = wp::load(var_122);
        var_123 = wp::copy(var_124);
        // bodyids[j] = bodyid_j                                                                  <L 2548>
        wp::assign_inplace(var_60, var_117, var_123);
        // if bodyid_j >= 0:                                                                      <L 2549>
        var_126 = (var_123 >= var_125);
        if (var_126) {
            // if body_dofnum[body_weldid[bodyid_j]] > 0:                                         <L 2550>
            var_127 = wp::address(var_body_weldid, var_123);
            var_129 = wp::load(var_127);
            var_128 = wp::address(var_body_dofnum, var_129);
            var_132 = wp::load(var_128);
            var_131 = (var_132 > var_130);
            if (var_131) {
                // has_dof[j] = 1                                                                 <L 2551>
                wp::assign_inplace(var_65, var_117, var_133);
            }
        }
        // bbase = bendingadr + 17 * eid                                                          <L 2553>
        var_135 = wp::mul(var_134, var_6);
        var_136 = wp::add(var_9, var_135);
        // c16 = flex_bending[bbase + 16]                                                         <L 2554>
        var_138 = wp::add(var_136, var_137);
        var_139 = wp::address(var_flex_bending, var_138);
        var_141 = wp::load(var_139);
        var_140 = wp::copy(var_141);
        // frc = mat43()                                                                          <L 2555>
        var_142 = wp::mat_t<4, 3, wp::float32>();
        // if has_spring and c16 != 0.0:                                                          <L 2556>
        var_143 = var_29;
        if (var_143) {
            var_145 = (var_140 != var_144);
            var_143 = var_143 && var_145;
        }
        if (var_143) {
            // ed0 = vpos[1] - vpos[0]                                                            <L 2557>
            var_147 = wp::extract(var_59, var_146);
            var_149 = wp::extract(var_59, var_148);
            var_150 = wp::sub(var_147, var_149);
            // ed1 = vpos[2] - vpos[0]                                                            <L 2558>
            var_152 = wp::extract(var_59, var_151);
            var_154 = wp::extract(var_59, var_153);
            var_155 = wp::sub(var_152, var_154);
            // ed2 = vpos[3] - vpos[0]                                                            <L 2559>
            var_157 = wp::extract(var_59, var_156);
            var_159 = wp::extract(var_59, var_158);
            var_160 = wp::sub(var_157, var_159);
            // frc[1] = wp.cross(ed1, ed2)                                                        <L 2561>
            var_161 = wp::cross(var_155, var_160);
            wp::assign_inplace(var_142, var_162, var_161);
            // frc[2] = wp.cross(ed2, ed0)                                                        <L 2562>
            var_163 = wp::cross(var_160, var_150);
            wp::assign_inplace(var_142, var_164, var_163);
            // frc[3] = wp.cross(ed0, ed1)                                                        <L 2563>
            var_165 = wp::cross(var_150, var_155);
            wp::assign_inplace(var_142, var_166, var_165);
            // frc[0] = -(frc[1] + frc[2] + frc[3])                                               <L 2564>
            var_168 = wp::extract(var_142, var_167);
            var_170 = wp::extract(var_142, var_169);
            var_171 = wp::add(var_168, var_170);
            var_173 = wp::extract(var_142, var_172);
            var_174 = wp::add(var_171, var_173);
            var_175 = wp::neg(var_174);
            wp::assign_inplace(var_142, var_176, var_175);
        }
        // vel = mat43()                                                                          <L 2567>
        var_177 = wp::mat_t<4, 3, wp::float32>();
        // if has_damper:                                                                         <L 2568>
        if (var_33) {
            // for j in range(4):                                                                 <L 2569>
            // if has_dof[j]:                                                                     <L 2570>
            var_179 = wp::extract(var_65, var_178);
            if (var_179) {
                // bodyid_j = bodyids[j]                                                          <L 2571>
                var_180 = wp::extract(var_60, var_178);
                // cvel_j = cvel_in[worldid, bodyid_j]                                            <L 2572>
                var_181 = wp::address(var_cvel_in, var_0, var_180);
                var_183 = wp::load(var_181);
                var_182 = wp::copy(var_183);
                // omega_j = wp.spatial_top(cvel_j)                                               <L 2573>
                var_184 = wp::spatial_top(var_182);
                // vcom_j = wp.spatial_bottom(cvel_j)                                             <L 2574>
                var_185 = wp::spatial_bottom(var_182);
                // com_j = subtree_com_in[worldid, body_rootid[bodyid_j]]                         <L 2575>
                var_186 = wp::address(var_body_rootid, var_180);
                var_188 = wp::load(var_186);
                var_187 = wp::address(var_subtree_com_in, var_0, var_188);
                var_190 = wp::load(var_187);
                var_189 = wp::copy(var_190);
                // r_j = vpos[j] - com_j                                                          <L 2576>
                var_191 = wp::extract(var_59, var_178);
                var_192 = wp::sub(var_191, var_189);
                // vel[j] = vcom_j + wp.cross(omega_j, r_j)                                       <L 2577>
                var_193 = wp::cross(var_184, var_192);
                var_194 = wp::add(var_185, var_193);
                wp::assign_inplace(var_177, var_178, var_194);
            }
            var_195 = wp::where(var_179, var_180, var_123);
            // if has_dof[j]:                                                                     <L 2570>
            var_197 = wp::extract(var_65, var_196);
            if (var_197) {
                // bodyid_j = bodyids[j]                                                          <L 2571>
                var_198 = wp::extract(var_60, var_196);
                // cvel_j = cvel_in[worldid, bodyid_j]                                            <L 2572>
                var_199 = wp::address(var_cvel_in, var_0, var_198);
                var_201 = wp::load(var_199);
                var_200 = wp::copy(var_201);
                // omega_j = wp.spatial_top(cvel_j)                                               <L 2573>
                var_202 = wp::spatial_top(var_200);
                // vcom_j = wp.spatial_bottom(cvel_j)                                             <L 2574>
                var_203 = wp::spatial_bottom(var_200);
                // com_j = subtree_com_in[worldid, body_rootid[bodyid_j]]                         <L 2575>
                var_204 = wp::address(var_body_rootid, var_198);
                var_206 = wp::load(var_204);
                var_205 = wp::address(var_subtree_com_in, var_0, var_206);
                var_208 = wp::load(var_205);
                var_207 = wp::copy(var_208);
                // r_j = vpos[j] - com_j                                                          <L 2576>
                var_209 = wp::extract(var_59, var_196);
                var_210 = wp::sub(var_209, var_207);
                // vel[j] = vcom_j + wp.cross(omega_j, r_j)                                       <L 2577>
                var_211 = wp::cross(var_202, var_210);
                var_212 = wp::add(var_203, var_211);
                wp::assign_inplace(var_177, var_196, var_212);
            }
            var_213 = wp::where(var_197, var_198, var_195);
            var_214 = wp::where(var_197, var_200, var_182);
            var_215 = wp::where(var_197, var_202, var_184);
            var_216 = wp::where(var_197, var_203, var_185);
            var_217 = wp::where(var_197, var_207, var_189);
            var_218 = wp::where(var_197, var_210, var_192);
            // if has_dof[j]:                                                                     <L 2570>
            var_220 = wp::extract(var_65, var_219);
            if (var_220) {
                // bodyid_j = bodyids[j]                                                          <L 2571>
                var_221 = wp::extract(var_60, var_219);
                // cvel_j = cvel_in[worldid, bodyid_j]                                            <L 2572>
                var_222 = wp::address(var_cvel_in, var_0, var_221);
                var_224 = wp::load(var_222);
                var_223 = wp::copy(var_224);
                // omega_j = wp.spatial_top(cvel_j)                                               <L 2573>
                var_225 = wp::spatial_top(var_223);
                // vcom_j = wp.spatial_bottom(cvel_j)                                             <L 2574>
                var_226 = wp::spatial_bottom(var_223);
                // com_j = subtree_com_in[worldid, body_rootid[bodyid_j]]                         <L 2575>
                var_227 = wp::address(var_body_rootid, var_221);
                var_229 = wp::load(var_227);
                var_228 = wp::address(var_subtree_com_in, var_0, var_229);
                var_231 = wp::load(var_228);
                var_230 = wp::copy(var_231);
                // r_j = vpos[j] - com_j                                                          <L 2576>
                var_232 = wp::extract(var_59, var_219);
                var_233 = wp::sub(var_232, var_230);
                // vel[j] = vcom_j + wp.cross(omega_j, r_j)                                       <L 2577>
                var_234 = wp::cross(var_225, var_233);
                var_235 = wp::add(var_226, var_234);
                wp::assign_inplace(var_177, var_219, var_235);
            }
            var_236 = wp::where(var_220, var_221, var_213);
            var_237 = wp::where(var_220, var_223, var_214);
            var_238 = wp::where(var_220, var_225, var_215);
            var_239 = wp::where(var_220, var_226, var_216);
            var_240 = wp::where(var_220, var_230, var_217);
            var_241 = wp::where(var_220, var_233, var_218);
            // if has_dof[j]:                                                                     <L 2570>
            var_243 = wp::extract(var_65, var_242);
            if (var_243) {
                // bodyid_j = bodyids[j]                                                          <L 2571>
                var_244 = wp::extract(var_60, var_242);
                // cvel_j = cvel_in[worldid, bodyid_j]                                            <L 2572>
                var_245 = wp::address(var_cvel_in, var_0, var_244);
                var_247 = wp::load(var_245);
                var_246 = wp::copy(var_247);
                // omega_j = wp.spatial_top(cvel_j)                                               <L 2573>
                var_248 = wp::spatial_top(var_246);
                // vcom_j = wp.spatial_bottom(cvel_j)                                             <L 2574>
                var_249 = wp::spatial_bottom(var_246);
                // com_j = subtree_com_in[worldid, body_rootid[bodyid_j]]                         <L 2575>
                var_250 = wp::address(var_body_rootid, var_244);
                var_252 = wp::load(var_250);
                var_251 = wp::address(var_subtree_com_in, var_0, var_252);
                var_254 = wp::load(var_251);
                var_253 = wp::copy(var_254);
                // r_j = vpos[j] - com_j                                                          <L 2576>
                var_255 = wp::extract(var_59, var_242);
                var_256 = wp::sub(var_255, var_253);
                // vel[j] = vcom_j + wp.cross(omega_j, r_j)                                       <L 2577>
                var_257 = wp::cross(var_248, var_256);
                var_258 = wp::add(var_249, var_257);
                wp::assign_inplace(var_177, var_242, var_258);
            }
            var_259 = wp::where(var_243, var_244, var_236);
            var_260 = wp::where(var_243, var_246, var_237);
            var_261 = wp::where(var_243, var_248, var_238);
            var_262 = wp::where(var_243, var_249, var_239);
            var_263 = wp::where(var_243, var_253, var_240);
            var_264 = wp::where(var_243, var_256, var_241);
        }
        var_265 = wp::where(var_33, var_242, var_117);
        var_266 = wp::where(var_33, var_259, var_123);
        // for i in range(4):                                                                     <L 2579>
        var_268 = wp::range(var_267);
        start_for_4:;
            if (iter_cmp(var_268) == 0) goto end_for_4;
            var_269 = wp::iter_next(var_268);
            // if not has_dof[i]:                                                                 <L 2580>
            var_270 = wp::extract(var_65, var_269);
            var_271 = wp::unot(var_270);
            if (var_271) {
                // continue                                                                       <L 2581>
                goto start_for_4;
            }
            // acc_spring = wp.vec3(0.0)                                                          <L 2582>
            var_273 = wp::vec_t<3, wp::float32>(var_272);
            // acc_damper = wp.vec3(0.0)                                                          <L 2583>
            var_275 = wp::vec_t<3, wp::float32>(var_274);
            // for j in range(4):                                                                 <L 2584>
            // coeff = flex_bending[bbase + 4 * i + j]                                            <L 2585>
            var_278 = wp::mul(var_277, var_269);
            var_279 = wp::add(var_136, var_278);
            var_280 = wp::add(var_279, var_276);
            var_281 = wp::address(var_flex_bending, var_280);
            var_283 = wp::load(var_281);
            var_282 = wp::copy(var_283);
            // if has_spring:                                                                     <L 2586>
            if (var_29) {
                // acc_spring += coeff * vpos[j]                                                  <L 2587>
                var_284 = wp::extract(var_59, var_276);
                var_285 = wp::mul(var_282, var_284);
                var_286 = wp::add(var_273, var_285);
            }
            var_287 = wp::where(var_29, var_286, var_273);
            // if has_damper:                                                                     <L 2588>
            if (var_33) {
                // acc_damper += coeff * vel[j]                                                   <L 2589>
                var_288 = wp::extract(var_177, var_276);
                var_289 = wp::mul(var_282, var_288);
                var_290 = wp::add(var_275, var_289);
            }
            var_291 = wp::where(var_33, var_290, var_275);
            // coeff = flex_bending[bbase + 4 * i + j]                                            <L 2585>
            var_294 = wp::mul(var_293, var_269);
            var_295 = wp::add(var_136, var_294);
            var_296 = wp::add(var_295, var_292);
            var_297 = wp::address(var_flex_bending, var_296);
            var_299 = wp::load(var_297);
            var_298 = wp::copy(var_299);
            // if has_spring:                                                                     <L 2586>
            if (var_29) {
                // acc_spring += coeff * vpos[j]                                                  <L 2587>
                var_300 = wp::extract(var_59, var_292);
                var_301 = wp::mul(var_298, var_300);
                var_302 = wp::add(var_287, var_301);
            }
            var_303 = wp::where(var_29, var_302, var_287);
            // if has_damper:                                                                     <L 2588>
            if (var_33) {
                // acc_damper += coeff * vel[j]                                                   <L 2589>
                var_304 = wp::extract(var_177, var_292);
                var_305 = wp::mul(var_298, var_304);
                var_306 = wp::add(var_291, var_305);
            }
            var_307 = wp::where(var_33, var_306, var_291);
            // coeff = flex_bending[bbase + 4 * i + j]                                            <L 2585>
            var_310 = wp::mul(var_309, var_269);
            var_311 = wp::add(var_136, var_310);
            var_312 = wp::add(var_311, var_308);
            var_313 = wp::address(var_flex_bending, var_312);
            var_315 = wp::load(var_313);
            var_314 = wp::copy(var_315);
            // if has_spring:                                                                     <L 2586>
            if (var_29) {
                // acc_spring += coeff * vpos[j]                                                  <L 2587>
                var_316 = wp::extract(var_59, var_308);
                var_317 = wp::mul(var_314, var_316);
                var_318 = wp::add(var_303, var_317);
            }
            var_319 = wp::where(var_29, var_318, var_303);
            // if has_damper:                                                                     <L 2588>
            if (var_33) {
                // acc_damper += coeff * vel[j]                                                   <L 2589>
                var_320 = wp::extract(var_177, var_308);
                var_321 = wp::mul(var_314, var_320);
                var_322 = wp::add(var_307, var_321);
            }
            var_323 = wp::where(var_33, var_322, var_307);
            // coeff = flex_bending[bbase + 4 * i + j]                                            <L 2585>
            var_326 = wp::mul(var_325, var_269);
            var_327 = wp::add(var_136, var_326);
            var_328 = wp::add(var_327, var_324);
            var_329 = wp::address(var_flex_bending, var_328);
            var_331 = wp::load(var_329);
            var_330 = wp::copy(var_331);
            // if has_spring:                                                                     <L 2586>
            if (var_29) {
                // acc_spring += coeff * vpos[j]                                                  <L 2587>
                var_332 = wp::extract(var_59, var_324);
                var_333 = wp::mul(var_330, var_332);
                var_334 = wp::add(var_319, var_333);
            }
            var_335 = wp::where(var_29, var_334, var_319);
            // if has_damper:                                                                     <L 2588>
            if (var_33) {
                // acc_damper += coeff * vel[j]                                                   <L 2589>
                var_336 = wp::extract(var_177, var_324);
                var_337 = wp::mul(var_330, var_336);
                var_338 = wp::add(var_323, var_337);
            }
            var_339 = wp::where(var_33, var_338, var_323);
            // bodyid = bodyids[i]                                                                <L 2591>
            var_340 = wp::extract(var_60, var_269);
            // node_pos = vpos[i]                                                                 <L 2592>
            var_341 = wp::extract(var_59, var_269);
            // body_xipos = xipos_in[worldid, bodyid]                                             <L 2593>
            var_342 = wp::address(var_xipos_in, var_0, var_340);
            var_344 = wp::load(var_342);
            var_343 = wp::copy(var_344);
            // offset = body_xipos - node_pos                                                     <L 2594>
            var_345 = wp::sub(var_343, var_341);
            // if has_spring:                                                                     <L 2596>
            if (var_29) {
                // frc_s = -(acc_spring + c16 * frc[i])                                           <L 2597>
                var_346 = wp::extract(var_142, var_269);
                var_347 = wp::mul(var_140, var_346);
                var_348 = wp::add(var_335, var_347);
                var_349 = wp::neg(var_348);
                // spatial_frc_s = wp.spatial_vector(frc_s, -wp.cross(offset, frc_s))             <L 2598>
                var_350 = wp::cross(var_345, var_349);
                var_351 = wp::neg(var_350);
                var_352 = wp::vec_t<6, wp::float32>(var_349, var_351);
                // wp.atomic_add(flex_spring_body_force_out, worldid, bodyid, spatial_frc_s)       <L 2599>
                var_353 = wp::atomic_add(var_flex_spring_body_force_out, var_0, var_340, var_352);
            }
            // if has_damper:                                                                     <L 2601>
            if (var_33) {
                // frc_d = -acc_damper * damping                                                  <L 2602>
                var_354 = wp::neg(var_339);
                var_355 = wp::mul(var_354, var_31);
                // spatial_frc_d = wp.spatial_vector(frc_d, -wp.cross(offset, frc_d))             <L 2603>
                var_356 = wp::cross(var_345, var_355);
                var_357 = wp::neg(var_356);
                var_358 = wp::vec_t<6, wp::float32>(var_355, var_357);
                // wp.atomic_add(flex_damper_body_force_out, worldid, bodyid, spatial_frc_d)       <L 2604>
                var_359 = wp::atomic_add(var_flex_damper_body_force_out, var_0, var_340, var_358);
            }
            goto start_for_4;
        end_for_4:;
    }
}



extern "C" __global__ void _spring_damper_dof_passive_44d3399c_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_opt_disableflags,
    wp::array_t<wp::float32> var_qpos_spring,
    wp::array_t<wp::int32> var_jnt_type,
    wp::array_t<wp::int32> var_jnt_qposadr,
    wp::array_t<wp::int32> var_jnt_dofadr,
    wp::array_t<wp::float32> var_jnt_stiffness,
    wp::array_t<wp::vec_t<2, wp::float32>> var_jnt_stiffnesspoly,
    wp::array_t<wp::float32> var_dof_damping,
    wp::array_t<wp::vec_t<2, wp::float32>> var_dof_dampingpoly,
    wp::array_t<wp::float32> var_qpos_in,
    wp::array_t<wp::float32> var_qvel_in,
    wp::array_t<wp::float32> var_qfrc_spring_out,
    wp::array_t<wp::float32> var_qfrc_damper_out)
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
        wp::shape_t* var_8;
        const wp::int32 var_9 = 0;
        wp::int32 var_10;
        wp::shape_t var_11;
        wp::int32 var_12;
        wp::float32* var_13;
        wp::float32 var_14;
        wp::float32 var_15;
        wp::shape_t* var_16;
        const wp::int32 var_17 = 0;
        wp::int32 var_18;
        wp::shape_t var_19;
        wp::int32 var_20;
        wp::vec_t<2, wp::float32>* var_21;
        wp::vec_t<2, wp::float32> var_22;
        wp::vec_t<2, wp::float32> var_23;
        bool var_24;
        bool var_25;
        const wp::float32 var_26 = 0.0;
        bool var_27;
        const wp::int32 var_28 = 0;
        wp::float32 var_29;
        const wp::float32 var_30 = 0.0;
        bool var_31;
        const wp::int32 var_32 = 1;
        wp::float32 var_33;
        const wp::float32 var_34 = 0.0;
        bool var_35;
        const wp::int32 var_36 = 32;
        wp::int32 var_37;
        bool var_38;
        const wp::int32 var_39 = 1;
        const wp::int32 var_40 = 0;
        bool var_41;
        const wp::int32 var_42 = 6;
        wp::int32 var_43;
        const wp::int32 var_44 = 1;
        bool var_45;
        const wp::int32 var_46 = 3;
        wp::int32 var_47;
        wp::int32 var_48;
        const wp::int32 var_49 = 64;
        wp::int32 var_50;
        wp::range_t var_51;
        wp::int32 var_52;
        const wp::float32 var_53 = 0.0;
        wp::int32 var_54;
        wp::shape_t* var_55;
        const wp::int32 var_56 = 0;
        wp::int32 var_57;
        wp::shape_t var_58;
        wp::int32 var_59;
        wp::shape_t* var_60;
        const wp::int32 var_61 = 0;
        wp::int32 var_62;
        wp::shape_t var_63;
        wp::int32 var_64;
        wp::range_t var_65;
        wp::int32 var_66;
        wp::int32 var_67;
        wp::float32* var_68;
        wp::float32 var_69;
        wp::float32 var_70;
        wp::int32 var_71;
        wp::vec_t<2, wp::float32>* var_72;
        wp::vec_t<2, wp::float32> var_73;
        wp::vec_t<2, wp::float32> var_74;
        bool var_75;
        const wp::float32 var_76 = 0.0;
        bool var_77;
        const wp::int32 var_78 = 0;
        wp::float32 var_79;
        const wp::float32 var_80 = 0.0;
        bool var_81;
        const wp::int32 var_82 = 1;
        wp::float32 var_83;
        const wp::float32 var_84 = 0.0;
        bool var_85;
        wp::int32 var_86;
        wp::float32* var_87;
        wp::float32 var_88;
        wp::float32 var_89;
        wp::float32 var_90;
        const wp::int32 var_91 = 1;
        wp::float32 var_92;
        wp::float32 var_93;
        wp::int32 var_94;
        const wp::float32 var_95 = 0.0;
        wp::int32 var_96;
        wp::int32 var_97;
        bool var_98;
        wp::range_t var_99;
        wp::int32 var_100;
        const wp::float32 var_101 = 0.0;
        wp::int32 var_102;
        wp::int32 var_103;
        wp::int32* var_104;
        wp::int32 var_105;
        wp::int32 var_106;
        wp::shape_t* var_107;
        const wp::int32 var_108 = 0;
        wp::int32 var_109;
        wp::shape_t var_110;
        wp::int32 var_111;
        const wp::int32 var_112 = 0;
        bool var_113;
        const wp::int32 var_114 = 0;
        wp::int32 var_115;
        wp::float32* var_116;
        const wp::int32 var_117 = 0;
        wp::int32 var_118;
        wp::float32* var_119;
        wp::float32 var_120;
        wp::float32 var_121;
        wp::float32 var_122;
        const wp::int32 var_123 = 1;
        wp::int32 var_124;
        wp::float32* var_125;
        const wp::int32 var_126 = 1;
        wp::int32 var_127;
        wp::float32* var_128;
        wp::float32 var_129;
        wp::float32 var_130;
        wp::float32 var_131;
        const wp::int32 var_132 = 2;
        wp::int32 var_133;
        wp::float32* var_134;
        const wp::int32 var_135 = 2;
        wp::int32 var_136;
        wp::float32* var_137;
        wp::float32 var_138;
        wp::float32 var_139;
        wp::float32 var_140;
        wp::vec_t<3, wp::float32> var_141;
        wp::float32 var_142;
        const wp::int32 var_143 = 0;
        wp::float32 var_144;
        wp::float32 var_145;
        const wp::int32 var_146 = 0;
        wp::float32 var_147;
        wp::float32 var_148;
        const wp::int32 var_149 = 0;
        wp::int32 var_150;
        wp::float32 var_151;
        const wp::int32 var_152 = 1;
        wp::float32 var_153;
        wp::float32 var_154;
        const wp::int32 var_155 = 1;
        wp::int32 var_156;
        wp::float32 var_157;
        const wp::int32 var_158 = 2;
        wp::float32 var_159;
        wp::float32 var_160;
        const wp::int32 var_161 = 2;
        wp::int32 var_162;
        const wp::int32 var_163 = 3;
        wp::int32 var_164;
        wp::float32* var_165;
        const wp::int32 var_166 = 4;
        wp::int32 var_167;
        wp::float32* var_168;
        const wp::int32 var_169 = 5;
        wp::int32 var_170;
        wp::float32* var_171;
        const wp::int32 var_172 = 6;
        wp::int32 var_173;
        wp::float32* var_174;
        wp::quat_t<wp::float32> var_175;
        wp::float32 var_176;
        wp::float32 var_177;
        wp::float32 var_178;
        wp::float32 var_179;
        wp::quat_t<wp::float32> var_180;
        const wp::int32 var_181 = 3;
        wp::int32 var_182;
        wp::float32* var_183;
        const wp::int32 var_184 = 4;
        wp::int32 var_185;
        wp::float32* var_186;
        const wp::int32 var_187 = 5;
        wp::int32 var_188;
        wp::float32* var_189;
        const wp::int32 var_190 = 6;
        wp::int32 var_191;
        wp::float32* var_192;
        wp::quat_t<wp::float32> var_193;
        wp::float32 var_194;
        wp::float32 var_195;
        wp::float32 var_196;
        wp::float32 var_197;
        wp::vec_t<3, wp::float32> var_198;
        wp::float32 var_199;
        const wp::int32 var_200 = 0;
        wp::float32 var_201;
        wp::float32 var_202;
        const wp::int32 var_203 = 0;
        wp::float32 var_204;
        wp::float32 var_205;
        const wp::int32 var_206 = 3;
        wp::int32 var_207;
        wp::float32 var_208;
        const wp::int32 var_209 = 1;
        wp::float32 var_210;
        wp::float32 var_211;
        const wp::int32 var_212 = 4;
        wp::int32 var_213;
        wp::float32 var_214;
        const wp::int32 var_215 = 2;
        wp::float32 var_216;
        wp::float32 var_217;
        const wp::int32 var_218 = 5;
        wp::int32 var_219;
        const wp::int32 var_220 = 1;
        bool var_221;
        const wp::int32 var_222 = 0;
        wp::int32 var_223;
        wp::float32* var_224;
        const wp::int32 var_225 = 1;
        wp::int32 var_226;
        wp::float32* var_227;
        const wp::int32 var_228 = 2;
        wp::int32 var_229;
        wp::float32* var_230;
        const wp::int32 var_231 = 3;
        wp::int32 var_232;
        wp::float32* var_233;
        wp::quat_t<wp::float32> var_234;
        wp::float32 var_235;
        wp::float32 var_236;
        wp::float32 var_237;
        wp::float32 var_238;
        wp::quat_t<wp::float32> var_239;
        const wp::int32 var_240 = 0;
        wp::int32 var_241;
        wp::float32* var_242;
        const wp::int32 var_243 = 1;
        wp::int32 var_244;
        wp::float32* var_245;
        const wp::int32 var_246 = 2;
        wp::int32 var_247;
        wp::float32* var_248;
        const wp::int32 var_249 = 3;
        wp::int32 var_250;
        wp::float32* var_251;
        wp::quat_t<wp::float32> var_252;
        wp::float32 var_253;
        wp::float32 var_254;
        wp::float32 var_255;
        wp::float32 var_256;
        wp::vec_t<3, wp::float32> var_257;
        wp::float32 var_258;
        const wp::int32 var_259 = 0;
        wp::float32 var_260;
        wp::float32 var_261;
        const wp::int32 var_262 = 0;
        wp::float32 var_263;
        wp::float32 var_264;
        const wp::int32 var_265 = 0;
        wp::int32 var_266;
        wp::float32 var_267;
        const wp::int32 var_268 = 1;
        wp::float32 var_269;
        wp::float32 var_270;
        const wp::int32 var_271 = 1;
        wp::int32 var_272;
        wp::float32 var_273;
        const wp::int32 var_274 = 2;
        wp::float32 var_275;
        wp::float32 var_276;
        const wp::int32 var_277 = 2;
        wp::int32 var_278;
        wp::vec_t<3, wp::float32> var_279;
        wp::float32 var_280;
        wp::float32 var_281;
        wp::quat_t<wp::float32> var_282;
        wp::quat_t<wp::float32> var_283;
        wp::float32* var_284;
        wp::float32* var_285;
        wp::float32 var_286;
        wp::float32 var_287;
        wp::float32 var_288;
        wp::float32 var_289;
        const wp::int32 var_290 = 0;
        wp::float32 var_291;
        wp::float32 var_292;
        wp::vec_t<3, wp::float32> var_293;
        wp::float32 var_294;
        wp::float32 var_295;
        wp::quat_t<wp::float32> var_296;
        wp::quat_t<wp::float32> var_297;
        //---------
        // forward
        // def _spring_damper_dof_passive(                                                        <L 82>
        // worldid, jntid = wp.tid()                                                              <L 100>
        builtin_tid2d(var_0, var_1);
        // dofid = jnt_dofadr[jntid]                                                              <L 101>
        var_2 = wp::address(var_jnt_dofadr, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // jnttype = jnt_type[jntid]                                                              <L 102>
        var_5 = wp::address(var_jnt_type, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // stiffness = jnt_stiffness[worldid % jnt_stiffness.shape[0], jntid]                     <L 103>
        var_8 = &(var_jnt_stiffness.shape);
        var_11 = wp::load(var_8);
        var_10 = wp::extract(var_11, var_9);
        var_12 = wp::mod(var_0, var_10);
        var_13 = wp::address(var_jnt_stiffness, var_12, var_1);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // spoly = jnt_stiffnesspoly[worldid % jnt_stiffnesspoly.shape[0], jntid]                 <L 104>
        var_16 = &(var_jnt_stiffnesspoly.shape);
        var_19 = wp::load(var_16);
        var_18 = wp::extract(var_19, var_17);
        var_20 = wp::mod(var_0, var_18);
        var_21 = wp::address(var_jnt_stiffnesspoly, var_20, var_1);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // has_stiffness = (stiffness != 0.0 or spoly[0] != 0.0 or spoly[1] != 0.0) and not (opt_disableflags & DisableBit.SPRING)       <L 105>
        var_27 = (var_14 != var_26);
        var_25 = var_27;
        if (!var_25) {
            var_29 = wp::extract(var_22, var_28);
            var_31 = (var_29 != var_30);
            var_25 = var_25 || var_31;
        }
        if (!var_25) {
            var_33 = wp::extract(var_22, var_32);
            var_35 = (var_33 != var_34);
            var_25 = var_25 || var_35;
        }
        var_24 = var_25;
        if (var_24) {
            var_37 = wp::bit_and(var_opt_disableflags, var_36);
            var_38 = wp::unot(var_37);
            var_24 = var_24 && var_38;
        }
        // ndof = 1                                                                               <L 107>
        // if jnttype == JointType.FREE:                                                          <L 108>
        var_41 = (var_6 == var_40);
        if (var_41) {
            // ndof = 6                                                                           <L 109>
        }
        var_43 = wp::where(var_41, var_42, var_39);
        if (!var_41) {
            // elif jnttype == JointType.BALL:                                                    <L 110>
            var_45 = (var_6 == var_44);
            if (var_45) {
                // ndof = 3                                                                       <L 111>
            }
            var_47 = wp::where(var_45, var_46, var_43);
        }
        var_48 = wp::where(var_41, var_43, var_47);
        // if opt_disableflags & DisableBit.DAMPER:                                               <L 112>
        var_50 = wp::bit_and(var_opt_disableflags, var_49);
        if (var_50) {
            // for i in range(ndof):                                                              <L 113>
            var_51 = wp::range(var_48);
            start_for_0:;
                if (iter_cmp(var_51) == 0) goto end_for_0;
                var_52 = wp::iter_next(var_51);
                // qfrc_damper_out[worldid, dofid + i] = 0.0                                      <L 114>
                var_54 = wp::add(var_3, var_52);
                wp::array_store(var_qfrc_damper_out, var_0, var_54, var_53);
                goto start_for_0;
            end_for_0:;
        }
        if (!var_50) {
            // dof_damping_id = worldid % dof_damping.shape[0]                                    <L 116>
            var_55 = &(var_dof_damping.shape);
            var_58 = wp::load(var_55);
            var_57 = wp::extract(var_58, var_56);
            var_59 = wp::mod(var_0, var_57);
            // dof_dampingpoly_id = worldid % dof_dampingpoly.shape[0]                            <L 117>
            var_60 = &(var_dof_dampingpoly.shape);
            var_63 = wp::load(var_60);
            var_62 = wp::extract(var_63, var_61);
            var_64 = wp::mod(var_0, var_62);
            // for i in range(ndof):                                                              <L 118>
            var_65 = wp::range(var_48);
            start_for_2:;
                if (iter_cmp(var_65) == 0) goto end_for_2;
                var_66 = wp::iter_next(var_65);
                // damping = dof_damping[dof_damping_id, dofid + i]                               <L 119>
                var_67 = wp::add(var_3, var_66);
                var_68 = wp::address(var_dof_damping, var_59, var_67);
                var_70 = wp::load(var_68);
                var_69 = wp::copy(var_70);
                // dpoly = dof_dampingpoly[dof_dampingpoly_id, dofid + i]                         <L 120>
                var_71 = wp::add(var_3, var_66);
                var_72 = wp::address(var_dof_dampingpoly, var_64, var_71);
                var_74 = wp::load(var_72);
                var_73 = wp::copy(var_74);
                // if damping != 0.0 or dpoly[0] != 0.0 or dpoly[1] != 0.0:                       <L 121>
                var_77 = (var_69 != var_76);
                var_75 = var_77;
                if (!var_75) {
                    var_79 = wp::extract(var_73, var_78);
                    var_81 = (var_79 != var_80);
                    var_75 = var_75 || var_81;
                }
                if (!var_75) {
                    var_83 = wp::extract(var_73, var_82);
                    var_85 = (var_83 != var_84);
                    var_75 = var_75 || var_85;
                }
                if (var_75) {
                    // v = qvel_in[worldid, dofid + i]                                            <L 122>
                    var_86 = wp::add(var_3, var_66);
                    var_87 = wp::address(var_qvel_in, var_0, var_86);
                    var_89 = wp::load(var_87);
                    var_88 = wp::copy(var_89);
                    // qfrc_damper_out[worldid, dofid + i] = -v * util_misc._poly_force(damping, dpoly, v, 1)       <L 123>
                    var_90 = wp::neg(var_88);
                    var_92 = _poly_force_0(var_69, var_73, var_88, var_91);
                    var_93 = wp::mul(var_90, var_92);
                    var_94 = wp::add(var_3, var_66);
                    wp::array_store(var_qfrc_damper_out, var_0, var_94, var_93);
                }
                if (!var_75) {
                    // qfrc_damper_out[worldid, dofid + i] = 0.0                                  <L 125>
                    var_96 = wp::add(var_3, var_66);
                    wp::array_store(var_qfrc_damper_out, var_0, var_96, var_95);
                }
                goto start_for_2;
            end_for_2:;
        }
        var_97 = wp::where(var_50, var_52, var_66);
        // if not has_stiffness:                                                                  <L 127>
        var_98 = wp::unot(var_24);
        if (var_98) {
            // for i in range(ndof):                                                              <L 128>
            var_99 = wp::range(var_48);
            start_for_4:;
                if (iter_cmp(var_99) == 0) goto end_for_4;
                var_100 = wp::iter_next(var_99);
                // qfrc_spring_out[worldid, dofid + i] = 0.0                                      <L 129>
                var_102 = wp::add(var_3, var_100);
                wp::array_store(var_qfrc_spring_out, var_0, var_102, var_101);
                goto start_for_4;
            end_for_4:;
            // return                                                                             <L 130>
            continue;
        }
        var_103 = wp::where(var_98, var_100, var_97);
        // qposid = jnt_qposadr[jntid]                                                            <L 131>
        var_104 = wp::address(var_jnt_qposadr, var_1);
        var_106 = wp::load(var_104);
        var_105 = wp::copy(var_106);
        // qpos_spring_id = worldid % qpos_spring.shape[0]                                        <L 132>
        var_107 = &(var_qpos_spring.shape);
        var_110 = wp::load(var_107);
        var_109 = wp::extract(var_110, var_108);
        var_111 = wp::mod(var_0, var_109);
        // if jnttype == JointType.FREE:                                                          <L 134>
        var_113 = (var_6 == var_112);
        if (var_113) {
            // dif = wp.vec3(                                                                     <L 136>
            // qpos_in[worldid, qposid + 0] - qpos_spring[qpos_spring_id, qposid + 0],            <L 137>
            var_115 = wp::add(var_105, var_114);
            var_116 = wp::address(var_qpos_in, var_0, var_115);
            var_118 = wp::add(var_105, var_117);
            var_119 = wp::address(var_qpos_spring, var_111, var_118);
            var_121 = wp::load(var_116);
            var_122 = wp::load(var_119);
            var_120 = wp::sub(var_121, var_122);
            // qpos_in[worldid, qposid + 1] - qpos_spring[qpos_spring_id, qposid + 1],            <L 138>
            var_124 = wp::add(var_105, var_123);
            var_125 = wp::address(var_qpos_in, var_0, var_124);
            var_127 = wp::add(var_105, var_126);
            var_128 = wp::address(var_qpos_spring, var_111, var_127);
            var_130 = wp::load(var_125);
            var_131 = wp::load(var_128);
            var_129 = wp::sub(var_130, var_131);
            // qpos_in[worldid, qposid + 2] - qpos_spring[qpos_spring_id, qposid + 2],            <L 139>
            var_133 = wp::add(var_105, var_132);
            var_134 = wp::address(var_qpos_in, var_0, var_133);
            var_136 = wp::add(var_105, var_135);
            var_137 = wp::address(var_qpos_spring, var_111, var_136);
            var_139 = wp::load(var_134);
            var_140 = wp::load(var_137);
            var_138 = wp::sub(var_139, var_140);
            var_141 = wp::vec_t<3, wp::float32>(var_120, var_129, var_138);
            // r = wp.length(dif)                                                                 <L 141>
            var_142 = wp::length(var_141);
            // k = util_misc._poly_force(stiffness, spoly, r, 0)                                  <L 142>
            var_144 = _poly_force_0(var_14, var_22, var_142, var_143);
            // qfrc_spring_out[worldid, dofid + 0] = -k * dif[0]                                  <L 143>
            var_145 = wp::neg(var_144);
            var_147 = wp::extract(var_141, var_146);
            var_148 = wp::mul(var_145, var_147);
            var_150 = wp::add(var_3, var_149);
            wp::array_store(var_qfrc_spring_out, var_0, var_150, var_148);
            // qfrc_spring_out[worldid, dofid + 1] = -k * dif[1]                                  <L 144>
            var_151 = wp::neg(var_144);
            var_153 = wp::extract(var_141, var_152);
            var_154 = wp::mul(var_151, var_153);
            var_156 = wp::add(var_3, var_155);
            wp::array_store(var_qfrc_spring_out, var_0, var_156, var_154);
            // qfrc_spring_out[worldid, dofid + 2] = -k * dif[2]                                  <L 145>
            var_157 = wp::neg(var_144);
            var_159 = wp::extract(var_141, var_158);
            var_160 = wp::mul(var_157, var_159);
            var_162 = wp::add(var_3, var_161);
            wp::array_store(var_qfrc_spring_out, var_0, var_162, var_160);
            // rot = wp.quat(                                                                     <L 147>
            // qpos_in[worldid, qposid + 3],                                                      <L 148>
            var_164 = wp::add(var_105, var_163);
            var_165 = wp::address(var_qpos_in, var_0, var_164);
            // qpos_in[worldid, qposid + 4],                                                      <L 149>
            var_167 = wp::add(var_105, var_166);
            var_168 = wp::address(var_qpos_in, var_0, var_167);
            // qpos_in[worldid, qposid + 5],                                                      <L 150>
            var_170 = wp::add(var_105, var_169);
            var_171 = wp::address(var_qpos_in, var_0, var_170);
            // qpos_in[worldid, qposid + 6],                                                      <L 151>
            var_173 = wp::add(var_105, var_172);
            var_174 = wp::address(var_qpos_in, var_0, var_173);
            var_176 = wp::load(var_165);
            var_177 = wp::load(var_168);
            var_178 = wp::load(var_171);
            var_179 = wp::load(var_174);
            var_175 = wp::quat_t<wp::float32>(var_176, var_177, var_178, var_179);
            // rot = wp.normalize(rot)                                                            <L 153>
            var_180 = wp::normalize(var_175);
            // ref = wp.quat(                                                                     <L 154>
            // qpos_spring[qpos_spring_id, qposid + 3],                                           <L 155>
            var_182 = wp::add(var_105, var_181);
            var_183 = wp::address(var_qpos_spring, var_111, var_182);
            // qpos_spring[qpos_spring_id, qposid + 4],                                           <L 156>
            var_185 = wp::add(var_105, var_184);
            var_186 = wp::address(var_qpos_spring, var_111, var_185);
            // qpos_spring[qpos_spring_id, qposid + 5],                                           <L 157>
            var_188 = wp::add(var_105, var_187);
            var_189 = wp::address(var_qpos_spring, var_111, var_188);
            // qpos_spring[qpos_spring_id, qposid + 6],                                           <L 158>
            var_191 = wp::add(var_105, var_190);
            var_192 = wp::address(var_qpos_spring, var_111, var_191);
            var_194 = wp::load(var_183);
            var_195 = wp::load(var_186);
            var_196 = wp::load(var_189);
            var_197 = wp::load(var_192);
            var_193 = wp::quat_t<wp::float32>(var_194, var_195, var_196, var_197);
            // dif = math.quat_sub(rot, ref)                                                      <L 160>
            var_198 = quat_sub_0(var_180, var_193);
            // r_rot = wp.length(dif)                                                             <L 161>
            var_199 = wp::length(var_198);
            // k_rot = util_misc._poly_force(stiffness, spoly, r_rot, 0)                          <L 162>
            var_201 = _poly_force_0(var_14, var_22, var_199, var_200);
            // qfrc_spring_out[worldid, dofid + 3] = -k_rot * dif[0]                              <L 163>
            var_202 = wp::neg(var_201);
            var_204 = wp::extract(var_198, var_203);
            var_205 = wp::mul(var_202, var_204);
            var_207 = wp::add(var_3, var_206);
            wp::array_store(var_qfrc_spring_out, var_0, var_207, var_205);
            // qfrc_spring_out[worldid, dofid + 4] = -k_rot * dif[1]                              <L 164>
            var_208 = wp::neg(var_201);
            var_210 = wp::extract(var_198, var_209);
            var_211 = wp::mul(var_208, var_210);
            var_213 = wp::add(var_3, var_212);
            wp::array_store(var_qfrc_spring_out, var_0, var_213, var_211);
            // qfrc_spring_out[worldid, dofid + 5] = -k_rot * dif[2]                              <L 165>
            var_214 = wp::neg(var_201);
            var_216 = wp::extract(var_198, var_215);
            var_217 = wp::mul(var_214, var_216);
            var_219 = wp::add(var_3, var_218);
            wp::array_store(var_qfrc_spring_out, var_0, var_219, var_217);
        }
        if (!var_113) {
            // elif jnttype == JointType.BALL:                                                    <L 167>
            var_221 = (var_6 == var_220);
            if (var_221) {
                // rot = wp.quat(                                                                 <L 169>
                // qpos_in[worldid, qposid + 0],                                                  <L 170>
                var_223 = wp::add(var_105, var_222);
                var_224 = wp::address(var_qpos_in, var_0, var_223);
                // qpos_in[worldid, qposid + 1],                                                  <L 171>
                var_226 = wp::add(var_105, var_225);
                var_227 = wp::address(var_qpos_in, var_0, var_226);
                // qpos_in[worldid, qposid + 2],                                                  <L 172>
                var_229 = wp::add(var_105, var_228);
                var_230 = wp::address(var_qpos_in, var_0, var_229);
                // qpos_in[worldid, qposid + 3],                                                  <L 173>
                var_232 = wp::add(var_105, var_231);
                var_233 = wp::address(var_qpos_in, var_0, var_232);
                var_235 = wp::load(var_224);
                var_236 = wp::load(var_227);
                var_237 = wp::load(var_230);
                var_238 = wp::load(var_233);
                var_234 = wp::quat_t<wp::float32>(var_235, var_236, var_237, var_238);
                // rot = wp.normalize(rot)                                                        <L 175>
                var_239 = wp::normalize(var_234);
                // ref = wp.quat(                                                                 <L 176>
                // qpos_spring[qpos_spring_id, qposid + 0],                                       <L 177>
                var_241 = wp::add(var_105, var_240);
                var_242 = wp::address(var_qpos_spring, var_111, var_241);
                // qpos_spring[qpos_spring_id, qposid + 1],                                       <L 178>
                var_244 = wp::add(var_105, var_243);
                var_245 = wp::address(var_qpos_spring, var_111, var_244);
                // qpos_spring[qpos_spring_id, qposid + 2],                                       <L 179>
                var_247 = wp::add(var_105, var_246);
                var_248 = wp::address(var_qpos_spring, var_111, var_247);
                // qpos_spring[qpos_spring_id, qposid + 3],                                       <L 180>
                var_250 = wp::add(var_105, var_249);
                var_251 = wp::address(var_qpos_spring, var_111, var_250);
                var_253 = wp::load(var_242);
                var_254 = wp::load(var_245);
                var_255 = wp::load(var_248);
                var_256 = wp::load(var_251);
                var_252 = wp::quat_t<wp::float32>(var_253, var_254, var_255, var_256);
                // dif = math.quat_sub(rot, ref)                                                  <L 182>
                var_257 = quat_sub_0(var_239, var_252);
                // r = wp.length(dif)                                                             <L 183>
                var_258 = wp::length(var_257);
                // k = util_misc._poly_force(stiffness, spoly, r, 0)                              <L 184>
                var_260 = _poly_force_0(var_14, var_22, var_258, var_259);
                // qfrc_spring_out[worldid, dofid + 0] = -k * dif[0]                              <L 185>
                var_261 = wp::neg(var_260);
                var_263 = wp::extract(var_257, var_262);
                var_264 = wp::mul(var_261, var_263);
                var_266 = wp::add(var_3, var_265);
                wp::array_store(var_qfrc_spring_out, var_0, var_266, var_264);
                // qfrc_spring_out[worldid, dofid + 1] = -k * dif[1]                              <L 186>
                var_267 = wp::neg(var_260);
                var_269 = wp::extract(var_257, var_268);
                var_270 = wp::mul(var_267, var_269);
                var_272 = wp::add(var_3, var_271);
                wp::array_store(var_qfrc_spring_out, var_0, var_272, var_270);
                // qfrc_spring_out[worldid, dofid + 2] = -k * dif[2]                              <L 187>
                var_273 = wp::neg(var_260);
                var_275 = wp::extract(var_257, var_274);
                var_276 = wp::mul(var_273, var_275);
                var_278 = wp::add(var_3, var_277);
                wp::array_store(var_qfrc_spring_out, var_0, var_278, var_276);
            }
            var_279 = wp::where(var_221, var_257, var_198);
            var_280 = wp::where(var_221, var_258, var_142);
            var_281 = wp::where(var_221, var_260, var_144);
            var_282 = wp::where(var_221, var_239, var_180);
            var_283 = wp::where(var_221, var_252, var_193);
            if (!var_221) {
                // fdif = qpos_in[worldid, qposid] - qpos_spring[qpos_spring_id, qposid]          <L 191>
                var_284 = wp::address(var_qpos_in, var_0, var_105);
                var_285 = wp::address(var_qpos_spring, var_111, var_105);
                var_287 = wp::load(var_284);
                var_288 = wp::load(var_285);
                var_286 = wp::sub(var_287, var_288);
                // qfrc_spring_out[worldid, dofid] = -fdif * util_misc._poly_force(stiffness, spoly, fdif, 0)       <L 192>
                var_289 = wp::neg(var_286);
                var_291 = _poly_force_0(var_14, var_22, var_286, var_290);
                var_292 = wp::mul(var_289, var_291);
                wp::array_store(var_qfrc_spring_out, var_0, var_3, var_292);
            }
        }
        var_293 = wp::where(var_113, var_198, var_279);
        var_294 = wp::where(var_113, var_142, var_280);
        var_295 = wp::where(var_113, var_144, var_281);
        var_296 = wp::where(var_113, var_180, var_282);
        var_297 = wp::where(var_113, var_193, var_283);
    }
}



extern "C" __global__ void _flex_passive_interp_38c71a67_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nflex,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::vec_t<3, wp::int32>> var_flex_cellnum,
    wp::array_t<wp::int32> var_flex_nodeadr,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::int32> var_flex_nodebodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_node,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_node0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<wp::float32> var_flex_damping,
    wp::array_t<wp::int32> var_flex_edgeequality,
    wp::array_t<bool> var_flex_centered,
    wp::array_t<wp::vec_t<4, wp::int32>> var_flex_cell_map,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cvel_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_spring_body_force_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_damper_body_force_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_displ_scratch_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_vel_corot_scratch_out)
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
        wp::vec_t<4, wp::int32>* var_2;
        wp::vec_t<4, wp::int32> var_3;
        wp::vec_t<4, wp::int32> var_4;
        const wp::int32 var_5 = 0;
        wp::int32 var_6;
        const wp::int32 var_7 = 1;
        wp::int32 var_8;
        const wp::int32 var_9 = 2;
        wp::int32 var_10;
        const wp::int32 var_11 = 3;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        const wp::int32 var_16 = 0;
        bool var_17;
        const wp::int32 var_18 = 1;
        wp::int32 var_19;
        const wp::int32 var_20 = 1;
        wp::int32 var_21;
        wp::int32 var_22;
        const wp::int32 var_23 = 1;
        wp::int32 var_24;
        wp::int32 var_25;
        const wp::int32 var_26 = 3;
        wp::int32 var_27;
        wp::vec_t<3, wp::int32>* var_28;
        wp::vec_t<3, wp::int32> var_29;
        wp::vec_t<3, wp::int32> var_30;
        const wp::int32 var_31 = 1;
        wp::int32 var_32;
        const wp::int32 var_33 = 2;
        wp::int32 var_34;
        wp::int32* var_35;
        wp::int32 var_36;
        wp::int32 var_37;
        wp::int32 var_38;
        const wp::int32 var_39 = 1;
        wp::int32 var_40;
        wp::int32 var_41;
        const wp::int32 var_42 = 1;
        wp::int32 var_43;
        wp::int32* var_44;
        wp::int32 var_45;
        wp::int32 var_46;
        const wp::int32 var_47 = 0;
        bool var_48;
        wp::int32 var_49;
        wp::int32 var_50;
        wp::int32 var_51;
        wp::int32 var_52;
        wp::int32 var_53;
        wp::int32 var_54;
        wp::int32 var_55;
        wp::int32 var_56;
        wp::float32* var_57;
        const wp::float32 var_58 = 0.0;
        bool var_59;
        wp::float32 var_60;
        wp::quat_t<wp::float32> var_61;
        const wp::int32 var_62 = 0;
        wp::float32 var_63;
        wp::float32 var_64;
        const wp::int32 var_65 = 1;
        wp::float32 var_66;
        wp::float32 var_67;
        const wp::int32 var_68 = 2;
        wp::float32 var_69;
        wp::float32 var_70;
        const wp::int32 var_71 = 3;
        wp::float32 var_72;
        wp::quat_t<wp::float32> var_73;
        const wp::int32 var_74 = 0;
        wp::int32 var_75;
        const wp::int32 var_76 = 1;
        wp::int32 var_77;
        wp::range_t var_78;
        wp::int32 var_79;
        const wp::int32 var_80 = 1;
        wp::int32 var_81;
        wp::range_t var_82;
        wp::int32 var_83;
        const wp::int32 var_84 = 1;
        wp::int32 var_85;
        wp::range_t var_86;
        wp::int32 var_87;
        bool var_88;
        wp::int32 var_89;
        wp::int32 var_90;
        wp::int32 var_91;
        wp::int32 var_92;
        wp::int32 var_93;
        wp::int32 var_94;
        wp::int32 var_95;
        wp::int32 var_96;
        wp::int32 var_97;
        wp::int32 var_98;
        wp::int32 var_99;
        wp::int32 var_100;
        wp::vec_t<3, wp::float32>* var_101;
        wp::vec_t<3, wp::float32> var_102;
        wp::vec_t<3, wp::float32> var_103;
        bool var_104;
        wp::int32 var_105;
        wp::vec_t<3, wp::float32>* var_106;
        wp::vec_t<3, wp::float32> var_107;
        wp::vec_t<3, wp::float32> var_108;
        wp::vec_t<3, wp::float32> var_109;
        wp::vec_t<3, wp::float32> var_110;
        bool var_111;
        wp::int32 var_112;
        wp::int32* var_113;
        wp::int32 var_114;
        wp::int32 var_115;
        wp::vec_t<6, wp::float32>* var_116;
        wp::vec_t<6, wp::float32> var_117;
        wp::vec_t<6, wp::float32> var_118;
        wp::vec_t<3, wp::float32> var_119;
        wp::vec_t<3, wp::float32> var_120;
        wp::int32* var_121;
        wp::vec_t<3, wp::float32>* var_122;
        wp::int32 var_123;
        wp::vec_t<3, wp::float32> var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32> var_126;
        wp::vec_t<3, wp::float32> var_127;
        wp::vec_t<3, wp::float32> var_128;
        wp::vec_t<3, wp::float32> var_129;
        const wp::int32 var_130 = 1;
        wp::int32 var_131;
        wp::int32 var_132;
        const wp::int32 var_133 = 0;
        wp::int32 var_134;
        const wp::int32 var_135 = 1;
        wp::int32 var_136;
        wp::range_t var_137;
        wp::int32 var_138;
        const wp::int32 var_139 = 1;
        wp::int32 var_140;
        wp::range_t var_141;
        wp::int32 var_142;
        const wp::int32 var_143 = 1;
        wp::int32 var_144;
        wp::range_t var_145;
        wp::int32 var_146;
        bool var_147;
        wp::int32 var_148;
        wp::int32 var_149;
        wp::int32 var_150;
        wp::int32 var_151;
        wp::int32 var_152;
        wp::int32 var_153;
        wp::int32 var_154;
        wp::int32 var_155;
        wp::int32 var_156;
        wp::int32 var_157;
        wp::int32 var_158;
        wp::int32 var_159;
        wp::int32* var_160;
        wp::int32 var_161;
        wp::int32 var_162;
        const wp::float32 var_163 = 0.0;
        wp::vec_t<3, wp::float32> var_164;
        const wp::float32 var_165 = 0.0;
        wp::vec_t<3, wp::float32> var_166;
        const wp::int32 var_167 = 0;
        const wp::int32 var_168 = 3;
        wp::int32 var_169;
        wp::int32 var_170;
        const wp::float32 var_171 = 0.0;
        wp::float32 var_172;
        const wp::float32 var_173 = 0.0;
        wp::float32 var_174;
        wp::range_t var_175;
        wp::int32 var_176;
        const wp::int32 var_177 = 0;
        const wp::int32 var_178 = 3;
        wp::int32 var_179;
        wp::int32 var_180;
        wp::int32 var_181;
        wp::int32 var_182;
        wp::int32 var_183;
        wp::float32* var_184;
        wp::float32 var_185;
        wp::float32 var_186;
        bool var_187;
        wp::vec_t<3, wp::float32>* var_188;
        wp::float32 var_189;
        wp::vec_t<3, wp::float32> var_190;
        wp::float32 var_191;
        wp::float32 var_192;
        wp::float32 var_193;
        bool var_194;
        wp::vec_t<3, wp::float32>* var_195;
        wp::float32 var_196;
        wp::vec_t<3, wp::float32> var_197;
        wp::float32 var_198;
        wp::float32 var_199;
        wp::float32 var_200;
        const wp::int32 var_201 = 1;
        const wp::int32 var_202 = 3;
        wp::int32 var_203;
        wp::int32 var_204;
        wp::int32 var_205;
        wp::int32 var_206;
        wp::int32 var_207;
        wp::float32* var_208;
        wp::float32 var_209;
        wp::float32 var_210;
        bool var_211;
        wp::vec_t<3, wp::float32>* var_212;
        wp::float32 var_213;
        wp::vec_t<3, wp::float32> var_214;
        wp::float32 var_215;
        wp::float32 var_216;
        wp::float32 var_217;
        bool var_218;
        wp::vec_t<3, wp::float32>* var_219;
        wp::float32 var_220;
        wp::vec_t<3, wp::float32> var_221;
        wp::float32 var_222;
        wp::float32 var_223;
        wp::float32 var_224;
        const wp::int32 var_225 = 2;
        const wp::int32 var_226 = 3;
        wp::int32 var_227;
        wp::int32 var_228;
        wp::int32 var_229;
        wp::int32 var_230;
        wp::int32 var_231;
        wp::float32* var_232;
        wp::float32 var_233;
        wp::float32 var_234;
        bool var_235;
        wp::vec_t<3, wp::float32>* var_236;
        wp::float32 var_237;
        wp::vec_t<3, wp::float32> var_238;
        wp::float32 var_239;
        wp::float32 var_240;
        wp::float32 var_241;
        bool var_242;
        wp::vec_t<3, wp::float32>* var_243;
        wp::float32 var_244;
        wp::vec_t<3, wp::float32> var_245;
        wp::float32 var_246;
        wp::float32 var_247;
        wp::float32 var_248;
        const wp::int32 var_249 = 1;
        const wp::int32 var_250 = 3;
        wp::int32 var_251;
        wp::int32 var_252;
        const wp::float32 var_253 = 0.0;
        wp::float32 var_254;
        const wp::float32 var_255 = 0.0;
        wp::float32 var_256;
        wp::range_t var_257;
        wp::int32 var_258;
        const wp::int32 var_259 = 0;
        const wp::int32 var_260 = 3;
        wp::int32 var_261;
        wp::int32 var_262;
        wp::int32 var_263;
        wp::int32 var_264;
        wp::int32 var_265;
        wp::float32* var_266;
        wp::float32 var_267;
        wp::float32 var_268;
        bool var_269;
        wp::vec_t<3, wp::float32>* var_270;
        wp::float32 var_271;
        wp::vec_t<3, wp::float32> var_272;
        wp::float32 var_273;
        wp::float32 var_274;
        wp::float32 var_275;
        bool var_276;
        wp::vec_t<3, wp::float32>* var_277;
        wp::float32 var_278;
        wp::vec_t<3, wp::float32> var_279;
        wp::float32 var_280;
        wp::float32 var_281;
        wp::float32 var_282;
        const wp::int32 var_283 = 1;
        const wp::int32 var_284 = 3;
        wp::int32 var_285;
        wp::int32 var_286;
        wp::int32 var_287;
        wp::int32 var_288;
        wp::int32 var_289;
        wp::float32* var_290;
        wp::float32 var_291;
        wp::float32 var_292;
        bool var_293;
        wp::vec_t<3, wp::float32>* var_294;
        wp::float32 var_295;
        wp::vec_t<3, wp::float32> var_296;
        wp::float32 var_297;
        wp::float32 var_298;
        wp::float32 var_299;
        bool var_300;
        wp::vec_t<3, wp::float32>* var_301;
        wp::float32 var_302;
        wp::vec_t<3, wp::float32> var_303;
        wp::float32 var_304;
        wp::float32 var_305;
        wp::float32 var_306;
        const wp::int32 var_307 = 2;
        const wp::int32 var_308 = 3;
        wp::int32 var_309;
        wp::int32 var_310;
        wp::int32 var_311;
        wp::int32 var_312;
        wp::int32 var_313;
        wp::float32* var_314;
        wp::float32 var_315;
        wp::float32 var_316;
        bool var_317;
        wp::vec_t<3, wp::float32>* var_318;
        wp::float32 var_319;
        wp::vec_t<3, wp::float32> var_320;
        wp::float32 var_321;
        wp::float32 var_322;
        wp::float32 var_323;
        bool var_324;
        wp::vec_t<3, wp::float32>* var_325;
        wp::float32 var_326;
        wp::vec_t<3, wp::float32> var_327;
        wp::float32 var_328;
        wp::float32 var_329;
        wp::float32 var_330;
        const wp::int32 var_331 = 2;
        const wp::int32 var_332 = 3;
        wp::int32 var_333;
        wp::int32 var_334;
        const wp::float32 var_335 = 0.0;
        wp::float32 var_336;
        const wp::float32 var_337 = 0.0;
        wp::float32 var_338;
        wp::range_t var_339;
        wp::int32 var_340;
        const wp::int32 var_341 = 0;
        const wp::int32 var_342 = 3;
        wp::int32 var_343;
        wp::int32 var_344;
        wp::int32 var_345;
        wp::int32 var_346;
        wp::int32 var_347;
        wp::float32* var_348;
        wp::float32 var_349;
        wp::float32 var_350;
        bool var_351;
        wp::vec_t<3, wp::float32>* var_352;
        wp::float32 var_353;
        wp::vec_t<3, wp::float32> var_354;
        wp::float32 var_355;
        wp::float32 var_356;
        wp::float32 var_357;
        bool var_358;
        wp::vec_t<3, wp::float32>* var_359;
        wp::float32 var_360;
        wp::vec_t<3, wp::float32> var_361;
        wp::float32 var_362;
        wp::float32 var_363;
        wp::float32 var_364;
        const wp::int32 var_365 = 1;
        const wp::int32 var_366 = 3;
        wp::int32 var_367;
        wp::int32 var_368;
        wp::int32 var_369;
        wp::int32 var_370;
        wp::int32 var_371;
        wp::float32* var_372;
        wp::float32 var_373;
        wp::float32 var_374;
        bool var_375;
        wp::vec_t<3, wp::float32>* var_376;
        wp::float32 var_377;
        wp::vec_t<3, wp::float32> var_378;
        wp::float32 var_379;
        wp::float32 var_380;
        wp::float32 var_381;
        bool var_382;
        wp::vec_t<3, wp::float32>* var_383;
        wp::float32 var_384;
        wp::vec_t<3, wp::float32> var_385;
        wp::float32 var_386;
        wp::float32 var_387;
        wp::float32 var_388;
        const wp::int32 var_389 = 2;
        const wp::int32 var_390 = 3;
        wp::int32 var_391;
        wp::int32 var_392;
        wp::int32 var_393;
        wp::int32 var_394;
        wp::int32 var_395;
        wp::float32* var_396;
        wp::float32 var_397;
        wp::float32 var_398;
        bool var_399;
        wp::vec_t<3, wp::float32>* var_400;
        wp::float32 var_401;
        wp::vec_t<3, wp::float32> var_402;
        wp::float32 var_403;
        wp::float32 var_404;
        wp::float32 var_405;
        bool var_406;
        wp::vec_t<3, wp::float32>* var_407;
        wp::float32 var_408;
        wp::vec_t<3, wp::float32> var_409;
        wp::float32 var_410;
        wp::float32 var_411;
        wp::float32 var_412;
        wp::vec_t<3, wp::float32> var_413;
        wp::vec_t<3, wp::float32> var_414;
        wp::float32* var_415;
        wp::vec_t<3, wp::float32> var_416;
        wp::float32 var_417;
        wp::int32 var_418;
        wp::vec_t<3, wp::float32>* var_419;
        wp::vec_t<3, wp::float32> var_420;
        wp::vec_t<3, wp::float32> var_421;
        wp::vec_t<3, wp::float32>* var_422;
        wp::vec_t<3, wp::float32> var_423;
        wp::vec_t<3, wp::float32> var_424;
        wp::vec_t<3, wp::float32> var_425;
        bool var_426;
        wp::vec_t<3, wp::float32> var_427;
        wp::vec_t<3, wp::float32> var_428;
        wp::vec_t<6, wp::float32> var_429;
        wp::vec_t<6, wp::float32> var_430;
        bool var_431;
        wp::vec_t<3, wp::float32> var_432;
        wp::vec_t<3, wp::float32> var_433;
        wp::vec_t<6, wp::float32> var_434;
        wp::vec_t<6, wp::float32> var_435;
        const wp::int32 var_436 = 1;
        wp::int32 var_437;
        wp::int32 var_438;
        wp::int32 var_439;
        //---------
        // forward
        // def _flex_passive_interp(                                                              <L 2608>
        // worldid, cellid = wp.tid()                                                             <L 2639>
        builtin_tid2d(var_0, var_1);
        // mapping = flex_cell_map[cellid]                                                        <L 2641>
        var_2 = wp::address(var_flex_cell_map, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // f = mapping[0]                                                                         <L 2642>
        var_6 = wp::extract(var_3, var_5);
        // ci = mapping[1]                                                                        <L 2643>
        var_8 = wp::extract(var_3, var_7);
        // cj = mapping[2]                                                                        <L 2644>
        var_10 = wp::extract(var_3, var_9);
        // ck = mapping[3]                                                                        <L 2645>
        var_12 = wp::extract(var_3, var_11);
        // order = flex_interp[f]                                                                 <L 2647>
        var_13 = wp::address(var_flex_interp, var_6);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // if order <= 0:                                                                         <L 2648>
        var_17 = (var_14 <= var_16);
        if (var_17) {
            // return                                                                             <L 2649>
            continue;
        }
        // npc = (order + 1) * (order + 1) * (order + 1)                                          <L 2651>
        var_19 = wp::add(var_14, var_18);
        var_21 = wp::add(var_14, var_20);
        var_22 = wp::mul(var_19, var_21);
        var_24 = wp::add(var_14, var_23);
        var_25 = wp::mul(var_22, var_24);
        // ndof_cell = 3 * npc                                                                    <L 2652>
        var_27 = wp::mul(var_26, var_25);
        // cellnum = flex_cellnum[f]                                                              <L 2654>
        var_28 = wp::address(var_flex_cellnum, var_6);
        var_30 = wp::load(var_28);
        var_29 = wp::copy(var_30);
        // cy = cellnum[1]                                                                        <L 2655>
        var_32 = wp::extract(var_29, var_31);
        // cz = cellnum[2]                                                                        <L 2656>
        var_34 = wp::extract(var_29, var_33);
        // nstart = flex_nodeadr[f]                                                               <L 2657>
        var_35 = wp::address(var_flex_nodeadr, var_6);
        var_37 = wp::load(var_35);
        var_36 = wp::copy(var_37);
        // ny_g = cy * order + 1                                                                  <L 2658>
        var_38 = wp::mul(var_32, var_14);
        var_40 = wp::add(var_38, var_39);
        // nz_g = cz * order + 1                                                                  <L 2659>
        var_41 = wp::mul(var_34, var_14);
        var_43 = wp::add(var_41, var_42);
        // stiffness_adr_base = flex_stiffnessadr[f]                                              <L 2662>
        var_44 = wp::address(var_flex_stiffnessadr, var_6);
        var_46 = wp::load(var_44);
        var_45 = wp::copy(var_46);
        // if stiffness_adr_base < 0:                                                             <L 2663>
        var_48 = (var_45 < var_47);
        if (var_48) {
            // return                                                                             <L 2664>
            continue;
        }
        // cell_idx = ci * cy * cz + cj * cz + ck                                                 <L 2666>
        var_49 = wp::mul(var_8, var_32);
        var_50 = wp::mul(var_49, var_34);
        var_51 = wp::mul(var_10, var_34);
        var_52 = wp::add(var_50, var_51);
        var_53 = wp::add(var_52, var_12);
        // k_base = stiffness_adr_base + cell_idx * ndof_cell * ndof_cell                         <L 2667>
        var_54 = wp::mul(var_53, var_27);
        var_55 = wp::mul(var_54, var_27);
        var_56 = wp::add(var_45, var_55);
        // if flex_stiffness[k_base] == 0.0:                                                      <L 2670>
        var_57 = wp::address(var_flex_stiffness, var_56);
        var_60 = wp::load(var_57);
        var_59 = (var_60 == var_58);
        if (var_59) {
            // return                                                                             <L 2671>
            continue;
        }
        // cell_quat = support.compute_interp_cell_quat(flexnode_xpos_in, order, ci, cj, ck, cy, cz, ny_g, nz_g, nstart, worldid)       <L 2673>
        var_61 = compute_interp_cell_quat_0(var_flexnode_xpos_in, var_14, var_8, var_10, var_12, var_32, var_34, var_40, var_43, var_36, var_0);
        // cell_quat_inv = wp.quat(-cell_quat[0], -cell_quat[1], -cell_quat[2], cell_quat[3])       <L 2676>
        var_63 = wp::extract(var_61, var_62);
        var_64 = wp::neg(var_63);
        var_66 = wp::extract(var_61, var_65);
        var_67 = wp::neg(var_66);
        var_69 = wp::extract(var_61, var_68);
        var_70 = wp::neg(var_69);
        var_72 = wp::extract(var_61, var_71);
        var_73 = wp::quat_t<wp::float32>(var_64, var_67, var_70, var_72);
        // idx_j = int(0)                                                                         <L 2680>
        var_75 = wp::int(var_74);
        // for li_j in range(order + 1):                                                          <L 2681>
        var_77 = wp::add(var_14, var_76);
        var_78 = wp::range(var_77);
        start_for_3:;
            if (iter_cmp(var_78) == 0) goto end_for_3;
            var_79 = wp::iter_next(var_78);
            // for lj_j in range(order + 1):                                                      <L 2682>
            var_81 = wp::add(var_14, var_80);
            var_82 = wp::range(var_81);
            start_for_5:;
                if (iter_cmp(var_82) == 0) goto end_for_5;
                var_83 = wp::iter_next(var_82);
                // for lk_j in range(order + 1):                                                  <L 2683>
                var_85 = wp::add(var_14, var_84);
                var_86 = wp::range(var_85);
                start_for_7:;
                    if (iter_cmp(var_86) == 0) goto end_for_7;
                    var_87 = wp::iter_next(var_86);
                    // if idx_j < npc:                                                            <L 2684>
                    var_88 = (var_75 < var_25);
                    if (var_88) {
                        // gi_j = ci * order + li_j                                               <L 2685>
                        var_89 = wp::mul(var_8, var_14);
                        var_90 = wp::add(var_89, var_79);
                        // gj_j = cj * order + lj_j                                               <L 2686>
                        var_91 = wp::mul(var_10, var_14);
                        var_92 = wp::add(var_91, var_83);
                        // gk_j = ck * order + lk_j                                               <L 2687>
                        var_93 = wp::mul(var_12, var_14);
                        var_94 = wp::add(var_93, var_87);
                        // gidx_j = gi_j * ny_g * nz_g + gj_j * nz_g + gk_j                       <L 2688>
                        var_95 = wp::mul(var_90, var_40);
                        var_96 = wp::mul(var_95, var_43);
                        var_97 = wp::mul(var_92, var_43);
                        var_98 = wp::add(var_96, var_97);
                        var_99 = wp::add(var_98, var_94);
                        // xpos_j = flexnode_xpos_in[worldid, nstart + gidx_j]                    <L 2690>
                        var_100 = wp::add(var_36, var_99);
                        var_101 = wp::address(var_flexnode_xpos_in, var_0, var_100);
                        var_103 = wp::load(var_101);
                        var_102 = wp::copy(var_103);
                        // if not dsbl_spring:                                                    <L 2692>
                        var_104 = wp::unot(var_dsbl_spring);
                        if (var_104) {
                            // refpos_j = flex_node0[nstart + gidx_j]                             <L 2693>
                            var_105 = wp::add(var_36, var_99);
                            var_106 = wp::address(var_flex_node0, var_105);
                            var_108 = wp::load(var_106);
                            var_107 = wp::copy(var_108);
                            // xrot_j = wp.quat_rotate(cell_quat_inv, xpos_j)                     <L 2694>
                            var_109 = wp::quat_rotate(var_73, var_102);
                            // displ_scratch_out[worldid, cellid, idx_j] = xrot_j - refpos_j       <L 2695>
                            var_110 = wp::sub(var_109, var_107);
                            wp::array_store(var_displ_scratch_out, var_0, var_1, var_75, var_110);
                        }
                        // if not dsbl_damper:                                                    <L 2697>
                        var_111 = wp::unot(var_dsbl_damper);
                        if (var_111) {
                            // bodyid_j = flex_nodebodyid[nstart + gidx_j]                        <L 2698>
                            var_112 = wp::add(var_36, var_99);
                            var_113 = wp::address(var_flex_nodebodyid, var_112);
                            var_115 = wp::load(var_113);
                            var_114 = wp::copy(var_115);
                            // cvel_j = cvel_in[worldid, bodyid_j]                                <L 2699>
                            var_116 = wp::address(var_cvel_in, var_0, var_114);
                            var_118 = wp::load(var_116);
                            var_117 = wp::copy(var_118);
                            // omega_j = wp.spatial_top(cvel_j)                                   <L 2700>
                            var_119 = wp::spatial_top(var_117);
                            // vcom_j = wp.spatial_bottom(cvel_j)                                 <L 2701>
                            var_120 = wp::spatial_bottom(var_117);
                            // com_j = subtree_com_in[worldid, body_rootid[bodyid_j]]             <L 2702>
                            var_121 = wp::address(var_body_rootid, var_114);
                            var_123 = wp::load(var_121);
                            var_122 = wp::address(var_subtree_com_in, var_0, var_123);
                            var_125 = wp::load(var_122);
                            var_124 = wp::copy(var_125);
                            // r_j = xpos_j - com_j                                               <L 2703>
                            var_126 = wp::sub(var_102, var_124);
                            // vel_world_j = vcom_j + wp.cross(omega_j, r_j)                      <L 2704>
                            var_127 = wp::cross(var_119, var_126);
                            var_128 = wp::add(var_120, var_127);
                            // vel_corot_scratch_out[worldid, cellid, idx_j] = wp.quat_rotate(cell_quat_inv, vel_world_j)       <L 2705>
                            var_129 = wp::quat_rotate(var_73, var_128);
                            wp::array_store(var_vel_corot_scratch_out, var_0, var_1, var_75, var_129);
                        }
                        // idx_j += 1                                                             <L 2707>
                        var_131 = wp::add(var_75, var_130);
                    }
                    var_132 = wp::where(var_88, var_131, var_75);
                    wp::assign(var_75, var_132);
                    goto start_for_7;
                end_for_7:;
                goto start_for_5;
            end_for_5:;
            goto start_for_3;
        end_for_3:;
        // idx_i = int(0)                                                                         <L 2710>
        var_134 = wp::int(var_133);
        // for li_i in range(order + 1):                                                          <L 2711>
        var_136 = wp::add(var_14, var_135);
        var_137 = wp::range(var_136);
        start_for_9:;
            if (iter_cmp(var_137) == 0) goto end_for_9;
            var_138 = wp::iter_next(var_137);
            // for lj_i in range(order + 1):                                                      <L 2712>
            var_140 = wp::add(var_14, var_139);
            var_141 = wp::range(var_140);
            start_for_11:;
                if (iter_cmp(var_141) == 0) goto end_for_11;
                var_142 = wp::iter_next(var_141);
                // for lk_i in range(order + 1):                                                  <L 2713>
                var_144 = wp::add(var_14, var_143);
                var_145 = wp::range(var_144);
                start_for_13:;
                    if (iter_cmp(var_145) == 0) goto end_for_13;
                    var_146 = wp::iter_next(var_145);
                    // if idx_i < npc:                                                            <L 2714>
                    var_147 = (var_134 < var_25);
                    if (var_147) {
                        // gi_i = ci * order + li_i                                               <L 2715>
                        var_148 = wp::mul(var_8, var_14);
                        var_149 = wp::add(var_148, var_138);
                        // gj_i = cj * order + lj_i                                               <L 2716>
                        var_150 = wp::mul(var_10, var_14);
                        var_151 = wp::add(var_150, var_142);
                        // gk_i = ck * order + lk_i                                               <L 2717>
                        var_152 = wp::mul(var_12, var_14);
                        var_153 = wp::add(var_152, var_146);
                        // gidx_i = gi_i * ny_g * nz_g + gj_i * nz_g + gk_i                       <L 2718>
                        var_154 = wp::mul(var_149, var_40);
                        var_155 = wp::mul(var_154, var_43);
                        var_156 = wp::mul(var_151, var_43);
                        var_157 = wp::add(var_155, var_156);
                        var_158 = wp::add(var_157, var_153);
                        // bodyid_i = flex_nodebodyid[nstart + gidx_i]                            <L 2719>
                        var_159 = wp::add(var_36, var_158);
                        var_160 = wp::address(var_flex_nodebodyid, var_159);
                        var_162 = wp::load(var_160);
                        var_161 = wp::copy(var_162);
                        // frc_spring = wp.vec3(0.0)                                              <L 2721>
                        var_164 = wp::vec_t<3, wp::float32>(var_163);
                        // frc_damper = wp.vec3(0.0)                                              <L 2722>
                        var_166 = wp::vec_t<3, wp::float32>(var_165);
                        // for comp_i in range(3):                                                <L 2724>
                        // row = idx_i * 3 + comp_i                                               <L 2725>
                        var_169 = wp::mul(var_134, var_168);
                        var_170 = wp::add(var_169, var_167);
                        // val_spring = float(0.0)                                                <L 2726>
                        var_172 = wp::float(var_171);
                        // val_damper = float(0.0)                                                <L 2727>
                        var_174 = wp::float(var_173);
                        // for idx_j in range(npc):                                               <L 2729>
                        var_175 = wp::range(var_25);
                        start_for_15:;
                            if (iter_cmp(var_175) == 0) goto end_for_15;
                            var_176 = wp::iter_next(var_175);
                            // for comp_j in range(3):                                            <L 2730>
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_179 = wp::mul(var_176, var_178);
                            var_180 = wp::add(var_179, var_177);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_181 = wp::mul(var_170, var_27);
                            var_182 = wp::add(var_56, var_181);
                            var_183 = wp::add(var_182, var_180);
                            var_184 = wp::address(var_flex_stiffness, var_183);
                            var_186 = wp::load(var_184);
                            var_185 = wp::copy(var_186);
                            // if not dsbl_spring:                                                <L 2734>
                            var_187 = wp::unot(var_dsbl_spring);
                            if (var_187) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_188 = wp::address(var_displ_scratch_out, var_0, var_1, var_176);
                                var_190 = wp::load(var_188);
                                var_189 = wp::extract(var_190, var_177);
                                var_191 = wp::mul(var_185, var_189);
                                var_192 = wp::add(var_172, var_191);
                            }
                            var_193 = wp::where(var_187, var_192, var_172);
                            // if not dsbl_damper:                                                <L 2737>
                            var_194 = wp::unot(var_dsbl_damper);
                            if (var_194) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_195 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_176);
                                var_197 = wp::load(var_195);
                                var_196 = wp::extract(var_197, var_177);
                                var_198 = wp::mul(var_185, var_196);
                                var_199 = wp::add(var_174, var_198);
                            }
                            var_200 = wp::where(var_194, var_199, var_174);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_203 = wp::mul(var_176, var_202);
                            var_204 = wp::add(var_203, var_201);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_205 = wp::mul(var_170, var_27);
                            var_206 = wp::add(var_56, var_205);
                            var_207 = wp::add(var_206, var_204);
                            var_208 = wp::address(var_flex_stiffness, var_207);
                            var_210 = wp::load(var_208);
                            var_209 = wp::copy(var_210);
                            // if not dsbl_spring:                                                <L 2734>
                            var_211 = wp::unot(var_dsbl_spring);
                            if (var_211) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_212 = wp::address(var_displ_scratch_out, var_0, var_1, var_176);
                                var_214 = wp::load(var_212);
                                var_213 = wp::extract(var_214, var_201);
                                var_215 = wp::mul(var_209, var_213);
                                var_216 = wp::add(var_193, var_215);
                            }
                            var_217 = wp::where(var_211, var_216, var_193);
                            // if not dsbl_damper:                                                <L 2737>
                            var_218 = wp::unot(var_dsbl_damper);
                            if (var_218) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_219 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_176);
                                var_221 = wp::load(var_219);
                                var_220 = wp::extract(var_221, var_201);
                                var_222 = wp::mul(var_209, var_220);
                                var_223 = wp::add(var_200, var_222);
                            }
                            var_224 = wp::where(var_218, var_223, var_200);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_227 = wp::mul(var_176, var_226);
                            var_228 = wp::add(var_227, var_225);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_229 = wp::mul(var_170, var_27);
                            var_230 = wp::add(var_56, var_229);
                            var_231 = wp::add(var_230, var_228);
                            var_232 = wp::address(var_flex_stiffness, var_231);
                            var_234 = wp::load(var_232);
                            var_233 = wp::copy(var_234);
                            // if not dsbl_spring:                                                <L 2734>
                            var_235 = wp::unot(var_dsbl_spring);
                            if (var_235) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_236 = wp::address(var_displ_scratch_out, var_0, var_1, var_176);
                                var_238 = wp::load(var_236);
                                var_237 = wp::extract(var_238, var_225);
                                var_239 = wp::mul(var_233, var_237);
                                var_240 = wp::add(var_217, var_239);
                            }
                            var_241 = wp::where(var_235, var_240, var_217);
                            // if not dsbl_damper:                                                <L 2737>
                            var_242 = wp::unot(var_dsbl_damper);
                            if (var_242) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_243 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_176);
                                var_245 = wp::load(var_243);
                                var_244 = wp::extract(var_245, var_225);
                                var_246 = wp::mul(var_233, var_244);
                                var_247 = wp::add(var_224, var_246);
                            }
                            var_248 = wp::where(var_242, var_247, var_224);
                            wp::assign(var_172, var_241);
                            wp::assign(var_174, var_248);
                            goto start_for_15;
                        end_for_15:;
                        // frc_spring[comp_i] = val_spring                                        <L 2740>
                        wp::assign_inplace(var_164, var_167, var_172);
                        // frc_damper[comp_i] = val_damper                                        <L 2741>
                        wp::assign_inplace(var_166, var_167, var_174);
                        // row = idx_i * 3 + comp_i                                               <L 2725>
                        var_251 = wp::mul(var_134, var_250);
                        var_252 = wp::add(var_251, var_249);
                        // val_spring = float(0.0)                                                <L 2726>
                        var_254 = wp::float(var_253);
                        // val_damper = float(0.0)                                                <L 2727>
                        var_256 = wp::float(var_255);
                        // for idx_j in range(npc):                                               <L 2729>
                        var_257 = wp::range(var_25);
                        start_for_17:;
                            if (iter_cmp(var_257) == 0) goto end_for_17;
                            var_258 = wp::iter_next(var_257);
                            // for comp_j in range(3):                                            <L 2730>
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_261 = wp::mul(var_258, var_260);
                            var_262 = wp::add(var_261, var_259);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_263 = wp::mul(var_252, var_27);
                            var_264 = wp::add(var_56, var_263);
                            var_265 = wp::add(var_264, var_262);
                            var_266 = wp::address(var_flex_stiffness, var_265);
                            var_268 = wp::load(var_266);
                            var_267 = wp::copy(var_268);
                            // if not dsbl_spring:                                                <L 2734>
                            var_269 = wp::unot(var_dsbl_spring);
                            if (var_269) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_270 = wp::address(var_displ_scratch_out, var_0, var_1, var_258);
                                var_272 = wp::load(var_270);
                                var_271 = wp::extract(var_272, var_259);
                                var_273 = wp::mul(var_267, var_271);
                                var_274 = wp::add(var_254, var_273);
                            }
                            var_275 = wp::where(var_269, var_274, var_254);
                            // if not dsbl_damper:                                                <L 2737>
                            var_276 = wp::unot(var_dsbl_damper);
                            if (var_276) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_277 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_258);
                                var_279 = wp::load(var_277);
                                var_278 = wp::extract(var_279, var_259);
                                var_280 = wp::mul(var_267, var_278);
                                var_281 = wp::add(var_256, var_280);
                            }
                            var_282 = wp::where(var_276, var_281, var_256);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_285 = wp::mul(var_258, var_284);
                            var_286 = wp::add(var_285, var_283);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_287 = wp::mul(var_252, var_27);
                            var_288 = wp::add(var_56, var_287);
                            var_289 = wp::add(var_288, var_286);
                            var_290 = wp::address(var_flex_stiffness, var_289);
                            var_292 = wp::load(var_290);
                            var_291 = wp::copy(var_292);
                            // if not dsbl_spring:                                                <L 2734>
                            var_293 = wp::unot(var_dsbl_spring);
                            if (var_293) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_294 = wp::address(var_displ_scratch_out, var_0, var_1, var_258);
                                var_296 = wp::load(var_294);
                                var_295 = wp::extract(var_296, var_283);
                                var_297 = wp::mul(var_291, var_295);
                                var_298 = wp::add(var_275, var_297);
                            }
                            var_299 = wp::where(var_293, var_298, var_275);
                            // if not dsbl_damper:                                                <L 2737>
                            var_300 = wp::unot(var_dsbl_damper);
                            if (var_300) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_301 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_258);
                                var_303 = wp::load(var_301);
                                var_302 = wp::extract(var_303, var_283);
                                var_304 = wp::mul(var_291, var_302);
                                var_305 = wp::add(var_282, var_304);
                            }
                            var_306 = wp::where(var_300, var_305, var_282);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_309 = wp::mul(var_258, var_308);
                            var_310 = wp::add(var_309, var_307);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_311 = wp::mul(var_252, var_27);
                            var_312 = wp::add(var_56, var_311);
                            var_313 = wp::add(var_312, var_310);
                            var_314 = wp::address(var_flex_stiffness, var_313);
                            var_316 = wp::load(var_314);
                            var_315 = wp::copy(var_316);
                            // if not dsbl_spring:                                                <L 2734>
                            var_317 = wp::unot(var_dsbl_spring);
                            if (var_317) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_318 = wp::address(var_displ_scratch_out, var_0, var_1, var_258);
                                var_320 = wp::load(var_318);
                                var_319 = wp::extract(var_320, var_307);
                                var_321 = wp::mul(var_315, var_319);
                                var_322 = wp::add(var_299, var_321);
                            }
                            var_323 = wp::where(var_317, var_322, var_299);
                            // if not dsbl_damper:                                                <L 2737>
                            var_324 = wp::unot(var_dsbl_damper);
                            if (var_324) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_325 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_258);
                                var_327 = wp::load(var_325);
                                var_326 = wp::extract(var_327, var_307);
                                var_328 = wp::mul(var_315, var_326);
                                var_329 = wp::add(var_306, var_328);
                            }
                            var_330 = wp::where(var_324, var_329, var_306);
                            wp::assign(var_254, var_323);
                            wp::assign(var_256, var_330);
                            wp::assign(var_228, var_310);
                            wp::assign(var_233, var_315);
                            goto start_for_17;
                        end_for_17:;
                        // frc_spring[comp_i] = val_spring                                        <L 2740>
                        wp::assign_inplace(var_164, var_249, var_254);
                        // frc_damper[comp_i] = val_damper                                        <L 2741>
                        wp::assign_inplace(var_166, var_249, var_256);
                        // row = idx_i * 3 + comp_i                                               <L 2725>
                        var_333 = wp::mul(var_134, var_332);
                        var_334 = wp::add(var_333, var_331);
                        // val_spring = float(0.0)                                                <L 2726>
                        var_336 = wp::float(var_335);
                        // val_damper = float(0.0)                                                <L 2727>
                        var_338 = wp::float(var_337);
                        // for idx_j in range(npc):                                               <L 2729>
                        var_339 = wp::range(var_25);
                        start_for_19:;
                            if (iter_cmp(var_339) == 0) goto end_for_19;
                            var_340 = wp::iter_next(var_339);
                            // for comp_j in range(3):                                            <L 2730>
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_343 = wp::mul(var_340, var_342);
                            var_344 = wp::add(var_343, var_341);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_345 = wp::mul(var_334, var_27);
                            var_346 = wp::add(var_56, var_345);
                            var_347 = wp::add(var_346, var_344);
                            var_348 = wp::address(var_flex_stiffness, var_347);
                            var_350 = wp::load(var_348);
                            var_349 = wp::copy(var_350);
                            // if not dsbl_spring:                                                <L 2734>
                            var_351 = wp::unot(var_dsbl_spring);
                            if (var_351) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_352 = wp::address(var_displ_scratch_out, var_0, var_1, var_340);
                                var_354 = wp::load(var_352);
                                var_353 = wp::extract(var_354, var_341);
                                var_355 = wp::mul(var_349, var_353);
                                var_356 = wp::add(var_336, var_355);
                            }
                            var_357 = wp::where(var_351, var_356, var_336);
                            // if not dsbl_damper:                                                <L 2737>
                            var_358 = wp::unot(var_dsbl_damper);
                            if (var_358) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_359 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_340);
                                var_361 = wp::load(var_359);
                                var_360 = wp::extract(var_361, var_341);
                                var_362 = wp::mul(var_349, var_360);
                                var_363 = wp::add(var_338, var_362);
                            }
                            var_364 = wp::where(var_358, var_363, var_338);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_367 = wp::mul(var_340, var_366);
                            var_368 = wp::add(var_367, var_365);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_369 = wp::mul(var_334, var_27);
                            var_370 = wp::add(var_56, var_369);
                            var_371 = wp::add(var_370, var_368);
                            var_372 = wp::address(var_flex_stiffness, var_371);
                            var_374 = wp::load(var_372);
                            var_373 = wp::copy(var_374);
                            // if not dsbl_spring:                                                <L 2734>
                            var_375 = wp::unot(var_dsbl_spring);
                            if (var_375) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_376 = wp::address(var_displ_scratch_out, var_0, var_1, var_340);
                                var_378 = wp::load(var_376);
                                var_377 = wp::extract(var_378, var_365);
                                var_379 = wp::mul(var_373, var_377);
                                var_380 = wp::add(var_357, var_379);
                            }
                            var_381 = wp::where(var_375, var_380, var_357);
                            // if not dsbl_damper:                                                <L 2737>
                            var_382 = wp::unot(var_dsbl_damper);
                            if (var_382) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_383 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_340);
                                var_385 = wp::load(var_383);
                                var_384 = wp::extract(var_385, var_365);
                                var_386 = wp::mul(var_373, var_384);
                                var_387 = wp::add(var_364, var_386);
                            }
                            var_388 = wp::where(var_382, var_387, var_364);
                            // col = idx_j * 3 + comp_j                                           <L 2731>
                            var_391 = wp::mul(var_340, var_390);
                            var_392 = wp::add(var_391, var_389);
                            // K_ij = flex_stiffness[k_base + row * ndof_cell + col]              <L 2732>
                            var_393 = wp::mul(var_334, var_27);
                            var_394 = wp::add(var_56, var_393);
                            var_395 = wp::add(var_394, var_392);
                            var_396 = wp::address(var_flex_stiffness, var_395);
                            var_398 = wp::load(var_396);
                            var_397 = wp::copy(var_398);
                            // if not dsbl_spring:                                                <L 2734>
                            var_399 = wp::unot(var_dsbl_spring);
                            if (var_399) {
                                // val_spring += K_ij * displ_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2735>
                                var_400 = wp::address(var_displ_scratch_out, var_0, var_1, var_340);
                                var_402 = wp::load(var_400);
                                var_401 = wp::extract(var_402, var_389);
                                var_403 = wp::mul(var_397, var_401);
                                var_404 = wp::add(var_381, var_403);
                            }
                            var_405 = wp::where(var_399, var_404, var_381);
                            // if not dsbl_damper:                                                <L 2737>
                            var_406 = wp::unot(var_dsbl_damper);
                            if (var_406) {
                                // val_damper += K_ij * vel_corot_scratch_out[worldid, cellid, idx_j][comp_j]       <L 2738>
                                var_407 = wp::address(var_vel_corot_scratch_out, var_0, var_1, var_340);
                                var_409 = wp::load(var_407);
                                var_408 = wp::extract(var_409, var_389);
                                var_410 = wp::mul(var_397, var_408);
                                var_411 = wp::add(var_388, var_410);
                            }
                            var_412 = wp::where(var_406, var_411, var_388);
                            wp::assign(var_336, var_405);
                            wp::assign(var_338, var_412);
                            wp::assign(var_228, var_392);
                            wp::assign(var_233, var_397);
                            goto start_for_19;
                        end_for_19:;
                        // frc_spring[comp_i] = val_spring                                        <L 2740>
                        wp::assign_inplace(var_164, var_331, var_336);
                        // frc_damper[comp_i] = val_damper                                        <L 2741>
                        wp::assign_inplace(var_166, var_331, var_338);
                        // frc_spring_world = wp.quat_rotate(cell_quat, frc_spring)               <L 2744>
                        var_413 = wp::quat_rotate(var_61, var_164);
                        // frc_damper_world = wp.quat_rotate(cell_quat, frc_damper)               <L 2745>
                        var_414 = wp::quat_rotate(var_61, var_166);
                        // frc_damper_world = frc_damper_world * flex_damping[f]                  <L 2748>
                        var_415 = wp::address(var_flex_damping, var_6);
                        var_417 = wp::load(var_415);
                        var_416 = wp::mul(var_414, var_417);
                        // node_pos = flexnode_xpos_in[worldid, nstart + gidx_i]                  <L 2751>
                        var_418 = wp::add(var_36, var_158);
                        var_419 = wp::address(var_flexnode_xpos_in, var_0, var_418);
                        var_421 = wp::load(var_419);
                        var_420 = wp::copy(var_421);
                        // body_xipos = xipos_in[worldid, bodyid_i]                               <L 2752>
                        var_422 = wp::address(var_xipos_in, var_0, var_161);
                        var_424 = wp::load(var_422);
                        var_423 = wp::copy(var_424);
                        // offset = body_xipos - node_pos                                         <L 2754>
                        var_425 = wp::sub(var_423, var_420);
                        // if not dsbl_spring:                                                    <L 2755>
                        var_426 = wp::unot(var_dsbl_spring);
                        if (var_426) {
                            // spatial_frc_s = wp.spatial_vector(frc_spring_world, -wp.cross(offset, frc_spring_world))       <L 2756>
                            var_427 = wp::cross(var_425, var_413);
                            var_428 = wp::neg(var_427);
                            var_429 = wp::vec_t<6, wp::float32>(var_413, var_428);
                            // wp.atomic_add(flex_spring_body_force_out, worldid, bodyid_i, spatial_frc_s)       <L 2757>
                            var_430 = wp::atomic_add(var_flex_spring_body_force_out, var_0, var_161, var_429);
                        }
                        // if not dsbl_damper:                                                    <L 2759>
                        var_431 = wp::unot(var_dsbl_damper);
                        if (var_431) {
                            // spatial_frc_d = wp.spatial_vector(frc_damper_world, -wp.cross(offset, frc_damper_world))       <L 2760>
                            var_432 = wp::cross(var_425, var_416);
                            var_433 = wp::neg(var_432);
                            var_434 = wp::vec_t<6, wp::float32>(var_416, var_433);
                            // wp.atomic_add(flex_damper_body_force_out, worldid, bodyid_i, spatial_frc_d)       <L 2761>
                            var_435 = wp::atomic_add(var_flex_damper_body_force_out, var_0, var_161, var_434);
                        }
                        // idx_i += 1                                                             <L 2763>
                        var_437 = wp::add(var_134, var_436);
                    }
                    var_438 = wp::where(var_147, var_340, var_75);
                    var_439 = wp::where(var_147, var_437, var_134);
                    wp::assign(var_75, var_438);
                    wp::assign(var_134, var_439);
                    goto start_for_13;
                end_for_13:;
                goto start_for_11;
            end_for_11:;
            goto start_for_9;
        end_for_9:;
    }
}



extern "C" __global__ void _spring_damper_tendon_passive_d76f76d8_cuda_kernel_forward(
    wp::launch_bounds_t<3> dim,
    wp::array_t<wp::int32> var_ten_J_rownnz,
    wp::array_t<wp::int32> var_ten_J_rowadr,
    wp::array_t<wp::int32> var_ten_J_colind,
    wp::array_t<wp::float32> var_tendon_stiffness,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_stiffnesspoly,
    wp::array_t<wp::float32> var_tendon_damping,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_dampingpoly,
    wp::array_t<wp::vec_t<2, wp::float32>> var_tendon_lengthspring,
    wp::array_t<wp::float32> var_ten_J_in,
    wp::array_t<wp::float32> var_ten_length_in,
    wp::array_t<wp::float32> var_ten_velocity_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::float32> var_qfrc_spring_out,
    wp::array_t<wp::float32> var_qfrc_damper_out)
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
        wp::shape_t* var_11;
        const wp::int32 var_12 = 0;
        wp::int32 var_13;
        wp::shape_t var_14;
        wp::int32 var_15;
        wp::vec_t<2, wp::float32>* var_16;
        wp::vec_t<2, wp::float32> var_17;
        wp::vec_t<2, wp::float32> var_18;
        wp::shape_t* var_19;
        const wp::int32 var_20 = 0;
        wp::int32 var_21;
        wp::shape_t var_22;
        wp::int32 var_23;
        wp::float32* var_24;
        wp::float32 var_25;
        wp::float32 var_26;
        wp::shape_t* var_27;
        const wp::int32 var_28 = 0;
        wp::int32 var_29;
        wp::shape_t var_30;
        wp::int32 var_31;
        wp::vec_t<2, wp::float32>* var_32;
        wp::vec_t<2, wp::float32> var_33;
        wp::vec_t<2, wp::float32> var_34;
        bool var_35;
        bool var_36;
        const wp::float32 var_37 = 0.0;
        bool var_38;
        const wp::int32 var_39 = 0;
        wp::float32 var_40;
        const wp::float32 var_41 = 0.0;
        bool var_42;
        const wp::int32 var_43 = 1;
        wp::float32 var_44;
        const wp::float32 var_45 = 0.0;
        bool var_46;
        bool var_47;
        bool var_48;
        bool var_49;
        const wp::float32 var_50 = 0.0;
        bool var_51;
        const wp::int32 var_52 = 0;
        wp::float32 var_53;
        const wp::float32 var_54 = 0.0;
        bool var_55;
        const wp::int32 var_56 = 1;
        wp::float32 var_57;
        const wp::float32 var_58 = 0.0;
        bool var_59;
        bool var_60;
        bool var_61;
        bool var_62;
        bool var_63;
        wp::int32* var_64;
        wp::int32 var_65;
        wp::int32 var_66;
        bool var_67;
        wp::int32* var_68;
        wp::int32 var_69;
        wp::int32 var_70;
        wp::int32 var_71;
        wp::float32* var_72;
        wp::float32 var_73;
        wp::float32 var_74;
        wp::int32* var_75;
        wp::int32 var_76;
        wp::int32 var_77;
        wp::float32* var_78;
        wp::float32 var_79;
        wp::float32 var_80;
        wp::shape_t* var_81;
        const wp::int32 var_82 = 0;
        wp::int32 var_83;
        wp::shape_t var_84;
        wp::int32 var_85;
        wp::vec_t<2, wp::float32>* var_86;
        wp::vec_t<2, wp::float32> var_87;
        wp::vec_t<2, wp::float32> var_88;
        const wp::int32 var_89 = 0;
        wp::float32 var_90;
        const wp::int32 var_91 = 1;
        wp::float32 var_92;
        bool var_93;
        wp::float32 var_94;
        bool var_95;
        wp::float32 var_96;
        const wp::float32 var_97 = 0.0;
        wp::float32 var_98;
        wp::float32 var_99;
        wp::float32 var_100;
        const wp::int32 var_101 = 0;
        wp::float32 var_102;
        wp::float32 var_103;
        wp::slice_t var_104;
        const wp::int32 var_105 = 0;
        wp::array_t<wp::float32> var_106;
        wp::float32 var_107;
        wp::float32 var_108;
        wp::float32* var_109;
        wp::float32 var_110;
        wp::float32 var_111;
        wp::float32 var_112;
        const wp::int32 var_113 = 1;
        wp::float32 var_114;
        wp::float32 var_115;
        wp::slice_t var_116;
        const wp::int32 var_117 = 0;
        wp::array_t<wp::float32> var_118;
        wp::float32 var_119;
        wp::float32 var_120;
        //---------
        // forward
        // def _spring_damper_tendon_passive(                                                     <L 196>
        // worldid, tenid, dofid_sparse = wp.tid()                                                <L 217>
        builtin_tid3d(var_0, var_1, var_2);
        // stiffness = tendon_stiffness[worldid % tendon_stiffness.shape[0], tenid]               <L 219>
        var_3 = &(var_tendon_stiffness.shape);
        var_6 = wp::load(var_3);
        var_5 = wp::extract(var_6, var_4);
        var_7 = wp::mod(var_0, var_5);
        var_8 = wp::address(var_tendon_stiffness, var_7, var_1);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // spoly = tendon_stiffnesspoly[worldid % tendon_stiffnesspoly.shape[0], tenid]           <L 220>
        var_11 = &(var_tendon_stiffnesspoly.shape);
        var_14 = wp::load(var_11);
        var_13 = wp::extract(var_14, var_12);
        var_15 = wp::mod(var_0, var_13);
        var_16 = wp::address(var_tendon_stiffnesspoly, var_15, var_1);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // damping = tendon_damping[worldid % tendon_damping.shape[0], tenid]                     <L 221>
        var_19 = &(var_tendon_damping.shape);
        var_22 = wp::load(var_19);
        var_21 = wp::extract(var_22, var_20);
        var_23 = wp::mod(var_0, var_21);
        var_24 = wp::address(var_tendon_damping, var_23, var_1);
        var_26 = wp::load(var_24);
        var_25 = wp::copy(var_26);
        // dpoly = tendon_dampingpoly[worldid % tendon_dampingpoly.shape[0], tenid]               <L 222>
        var_27 = &(var_tendon_dampingpoly.shape);
        var_30 = wp::load(var_27);
        var_29 = wp::extract(var_30, var_28);
        var_31 = wp::mod(var_0, var_29);
        var_32 = wp::address(var_tendon_dampingpoly, var_31, var_1);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // has_stiffness = (stiffness != 0.0 or spoly[0] != 0.0 or spoly[1] != 0.0) and not dsbl_spring       <L 224>
        var_38 = (var_9 != var_37);
        var_36 = var_38;
        if (!var_36) {
            var_40 = wp::extract(var_17, var_39);
            var_42 = (var_40 != var_41);
            var_36 = var_36 || var_42;
        }
        if (!var_36) {
            var_44 = wp::extract(var_17, var_43);
            var_46 = (var_44 != var_45);
            var_36 = var_36 || var_46;
        }
        var_35 = var_36;
        if (var_35) {
            var_47 = wp::unot(var_dsbl_spring);
            var_35 = var_35 && var_47;
        }
        // has_damping = (damping != 0.0 or dpoly[0] != 0.0 or dpoly[1] != 0.0) and not dsbl_damper       <L 225>
        var_51 = (var_25 != var_50);
        var_49 = var_51;
        if (!var_49) {
            var_53 = wp::extract(var_33, var_52);
            var_55 = (var_53 != var_54);
            var_49 = var_49 || var_55;
        }
        if (!var_49) {
            var_57 = wp::extract(var_33, var_56);
            var_59 = (var_57 != var_58);
            var_49 = var_49 || var_59;
        }
        var_48 = var_49;
        if (var_48) {
            var_60 = wp::unot(var_dsbl_damper);
            var_48 = var_48 && var_60;
        }
        // if not has_stiffness and not has_damping:                                              <L 227>
        var_62 = wp::unot(var_35);
        var_61 = var_62;
        if (var_61) {
            var_63 = wp::unot(var_48);
            var_61 = var_61 && var_63;
        }
        if (var_61) {
            // return                                                                             <L 228>
            continue;
        }
        // rownnz = ten_J_rownnz[tenid]                                                           <L 230>
        var_64 = wp::address(var_ten_J_rownnz, var_1);
        var_66 = wp::load(var_64);
        var_65 = wp::copy(var_66);
        // if dofid_sparse >= rownnz:                                                             <L 231>
        var_67 = (var_2 >= var_65);
        if (var_67) {
            // return                                                                             <L 232>
            continue;
        }
        // rowadr = ten_J_rowadr[tenid]                                                           <L 233>
        var_68 = wp::address(var_ten_J_rowadr, var_1);
        var_70 = wp::load(var_68);
        var_69 = wp::copy(var_70);
        // sparseid = rowadr + dofid_sparse                                                       <L 234>
        var_71 = wp::add(var_69, var_2);
        // J = ten_J_in[worldid, sparseid]                                                        <L 235>
        var_72 = wp::address(var_ten_J_in, var_0, var_71);
        var_74 = wp::load(var_72);
        var_73 = wp::copy(var_74);
        // dofid = ten_J_colind[sparseid]                                                         <L 236>
        var_75 = wp::address(var_ten_J_colind, var_71);
        var_77 = wp::load(var_75);
        var_76 = wp::copy(var_77);
        // if has_stiffness:                                                                      <L 238>
        if (var_35) {
            // length = ten_length_in[worldid, tenid]                                             <L 240>
            var_78 = wp::address(var_ten_length_in, var_0, var_1);
            var_80 = wp::load(var_78);
            var_79 = wp::copy(var_80);
            // lengthspring = tendon_lengthspring[worldid % tendon_lengthspring.shape[0], tenid]       <L 241>
            var_81 = &(var_tendon_lengthspring.shape);
            var_84 = wp::load(var_81);
            var_83 = wp::extract(var_84, var_82);
            var_85 = wp::mod(var_0, var_83);
            var_86 = wp::address(var_tendon_lengthspring, var_85, var_1);
            var_88 = wp::load(var_86);
            var_87 = wp::copy(var_88);
            // lower = lengthspring[0]                                                            <L 242>
            var_90 = wp::extract(var_87, var_89);
            // upper = lengthspring[1]                                                            <L 243>
            var_92 = wp::extract(var_87, var_91);
            // x = wp.where(length > upper, length - upper, wp.where(length < lower, length - lower, 0.0))       <L 245>
            var_93 = (var_79 > var_92);
            var_94 = wp::sub(var_79, var_92);
            var_95 = (var_79 < var_90);
            var_96 = wp::sub(var_79, var_90);
            var_98 = wp::where(var_95, var_96, var_97);
            var_99 = wp::where(var_93, var_94, var_98);
            // frc_spring = -x * util_misc._poly_force(stiffness, spoly, x, 0)                    <L 246>
            var_100 = wp::neg(var_99);
            var_102 = _poly_force_0(var_9, var_17, var_99, var_101);
            var_103 = wp::mul(var_100, var_102);
            // wp.atomic_add(qfrc_spring_out[worldid], dofid, J * frc_spring)                     <L 249>
            var_104 = wp::slice_t(var_0, var_0, var_105);
            var_106 = wp::view(var_qfrc_spring_out, var_104);
            var_107 = wp::mul(var_73, var_103);
            var_108 = wp::atomic_add(var_106, var_76, var_107);
        }
        // if has_damping:                                                                        <L 251>
        if (var_48) {
            // v = ten_velocity_in[worldid, tenid]                                                <L 253>
            var_109 = wp::address(var_ten_velocity_in, var_0, var_1);
            var_111 = wp::load(var_109);
            var_110 = wp::copy(var_111);
            // frc_damper = -v * util_misc._poly_force(damping, dpoly, v, 1)                      <L 254>
            var_112 = wp::neg(var_110);
            var_114 = _poly_force_0(var_25, var_33, var_110, var_113);
            var_115 = wp::mul(var_112, var_114);
            // wp.atomic_add(qfrc_damper_out[worldid], dofid, J * frc_damper)                     <L 257>
            var_116 = wp::slice_t(var_0, var_0, var_117);
            var_118 = wp::view(var_qfrc_damper_out, var_116);
            var_119 = wp::mul(var_73, var_115);
            var_120 = wp::atomic_add(var_118, var_76, var_119);
        }
    }
}



extern "C" __global__ void _qfrc_adhesion_8d86bfc5_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_body_parentid,
    wp::array_t<wp::int32> var_body_rootid,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_dof_bodyid,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::int32> var_body_isdofancestor,
    wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_contact_pos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_geom_in,
    wp::array_t<wp::int32> var_contact_worldid_in,
    wp::array_t<wp::float32> var_contact_adhesion_in,
    wp::array_t<wp::int32> var_nacon_in,
    wp::array_t<wp::float32> var_qfrc_adhesion_out)
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
        wp::float32* var_5;
        wp::float32 var_6;
        wp::float32 var_7;
        const wp::float32 var_8 = 0.0;
        bool var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        wp::vec_t<2, wp::int32>* var_13;
        wp::vec_t<2, wp::int32> var_14;
        wp::vec_t<2, wp::int32> var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        const wp::int32 var_18 = 1;
        wp::int32 var_19;
        const wp::int32 var_20 = 0;
        bool var_21;
        wp::int32* var_22;
        wp::int32* var_23;
        wp::int32 var_24;
        wp::int32 var_25;
        const wp::int32 var_26 = -1;
        wp::int32 var_27;
        const wp::int32 var_28 = 0;
        bool var_29;
        wp::int32* var_30;
        wp::int32* var_31;
        wp::int32 var_32;
        wp::int32 var_33;
        const wp::int32 var_34 = -1;
        wp::int32 var_35;
        wp::vec_t<3, wp::float32>* var_36;
        wp::vec_t<3, wp::float32> var_37;
        wp::vec_t<3, wp::float32> var_38;
        wp::mat_t<3, 3, wp::float32>* var_39;
        wp::mat_t<3, 3, wp::float32> var_40;
        wp::mat_t<3, 3, wp::float32> var_41;
        const wp::int32 var_42 = 0;
        const wp::int32 var_43 = 0;
        wp::float32 var_44;
        const wp::int32 var_45 = 0;
        const wp::int32 var_46 = 1;
        wp::float32 var_47;
        const wp::int32 var_48 = 0;
        const wp::int32 var_49 = 2;
        wp::float32 var_50;
        wp::vec_t<3, wp::float32> var_51;
        wp::vec_t<3, wp::float32> var_52;
        const wp::int32 var_53 = 0;
        bool var_54;
        wp::int32 var_55;
        const wp::int32 var_56 = 0;
        bool var_57;
        wp::int32* var_58;
        wp::int32 var_59;
        wp::int32 var_60;
        wp::int32* var_61;
        wp::int32 var_62;
        wp::int32 var_63;
        wp::int32 var_64;
        wp::range_t var_65;
        wp::int32 var_66;
        wp::vec_t<3, wp::float32> var_67;
        wp::vec_t<3, wp::float32> var_68;
        wp::float32 var_69;
        wp::float32 var_70;
        wp::int32* var_71;
        wp::int32 var_72;
        wp::int32 var_73;
        const wp::int32 var_74 = 0;
        bool var_75;
        wp::int32 var_76;
        const wp::int32 var_77 = 0;
        bool var_78;
        wp::int32* var_79;
        wp::int32 var_80;
        wp::int32 var_81;
        wp::int32* var_82;
        wp::int32 var_83;
        wp::int32 var_84;
        wp::int32 var_85;
        wp::range_t var_86;
        wp::int32 var_87;
        wp::vec_t<3, wp::float32> var_88;
        wp::vec_t<3, wp::float32> var_89;
        wp::vec_t<3, wp::float32> var_90;
        wp::float32 var_91;
        wp::float32 var_92;
        wp::int32* var_93;
        wp::int32 var_94;
        wp::int32 var_95;
        wp::int32 var_96;
        //---------
        // forward
        // def _qfrc_adhesion(                                                                    <L 619>
        // cid = wp.tid()                                                                         <L 641>
        var_0 = builtin_tid1d();
        // if cid >= nacon_in[0]:                                                                 <L 642>
        var_2 = wp::address(var_nacon_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 643>
            continue;
        }
        // adhesion = contact_adhesion_in[cid]                                                    <L 645>
        var_5 = wp::address(var_contact_adhesion_in, var_0);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // if adhesion == 0.0:                                                                    <L 646>
        var_9 = (var_6 == var_8);
        if (var_9) {
            // return                                                                             <L 647>
            continue;
        }
        // worldid = contact_worldid_in[cid]                                                      <L 649>
        var_10 = wp::address(var_contact_worldid_in, var_0);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // geoms = contact_geom_in[cid]                                                           <L 650>
        var_13 = wp::address(var_contact_geom_in, var_0);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // g1 = geoms[0]                                                                          <L 651>
        var_17 = wp::extract(var_14, var_16);
        // g2 = geoms[1]                                                                          <L 652>
        var_19 = wp::extract(var_14, var_18);
        // body1 = body_weldid[geom_bodyid[g1]] if g1 >= 0 else -1                                <L 653>
        var_21 = (var_17 >= var_20);
        if (var_21) {
            var_22 = wp::address(var_geom_bodyid, var_17);
            var_24 = wp::load(var_22);
            var_23 = wp::address(var_body_weldid, var_24);
            var_25 = wp::load(var_23);
        }
        if (!var_21) {
        }
        var_27 = wp::where(var_21, var_25, var_26);
        // body2 = body_weldid[geom_bodyid[g2]] if g2 >= 0 else -1                                <L 654>
        var_29 = (var_19 >= var_28);
        if (var_29) {
            var_30 = wp::address(var_geom_bodyid, var_19);
            var_32 = wp::load(var_30);
            var_31 = wp::address(var_body_weldid, var_32);
            var_33 = wp::load(var_31);
        }
        if (!var_29) {
        }
        var_35 = wp::where(var_29, var_33, var_34);
        // pos = contact_pos_in[cid]                                                              <L 656>
        var_36 = wp::address(var_contact_pos_in, var_0);
        var_38 = wp::load(var_36);
        var_37 = wp::copy(var_38);
        // frame = contact_frame_in[cid]                                                          <L 657>
        var_39 = wp::address(var_contact_frame_in, var_0);
        var_41 = wp::load(var_39);
        var_40 = wp::copy(var_41);
        // normal = wp.vec3(frame[0, 0], frame[0, 1], frame[0, 2])                                <L 658>
        var_44 = wp::extract(var_40, var_42, var_43);
        var_47 = wp::extract(var_40, var_45, var_46);
        var_50 = wp::extract(var_40, var_48, var_49);
        var_51 = wp::vec_t<3, wp::float32>(var_44, var_47, var_50);
        // force = adhesion * normal                                                              <L 659>
        var_52 = wp::mul(var_6, var_51);
        // if body1 > 0:                                                                          <L 661>
        var_54 = (var_27 > var_53);
        if (var_54) {
            // b = body1                                                                          <L 662>
            var_55 = wp::copy(var_27);
            // while b > 0:                                                                       <L 663>
        start_while_2:;
            var_57 = (var_55 > var_56);
        if ((var_57) == false) goto end_while_2;
                // dofadr = body_dofadr[b]                                                        <L 664>
                var_58 = wp::address(var_body_dofadr, var_55);
                var_60 = wp::load(var_58);
                var_59 = wp::copy(var_60);
                // dofnum = body_dofnum[b]                                                        <L 665>
                var_61 = wp::address(var_body_dofnum, var_55);
                var_63 = wp::load(var_61);
                var_62 = wp::copy(var_63);
                // for dofid in range(dofadr, dofadr + dofnum):                                   <L 666>
                var_64 = wp::add(var_59, var_62);
                var_65 = wp::range(var_59, var_64);
                start_for_4:;
                    if (iter_cmp(var_65) == 0) goto end_for_4;
                    var_66 = wp::iter_next(var_65);
                    // jacp, _ = support.jac_dof(                                                 <L 667>
                    // body_parentid, body_rootid, dof_bodyid, body_isdofancestor, subtree_com_in, cdof_in, pos, body1, dofid, worldid       <L 668>
                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_37, var_27, var_66, var_11, var_67, var_68);
                    // wp.atomic_add(qfrc_adhesion_out, worldid, dofid, wp.dot(jacp, force))       <L 670>
                    var_69 = wp::dot(var_67, var_52);
                    var_70 = wp::atomic_add(var_qfrc_adhesion_out, var_11, var_66, var_69);
                    goto start_for_4;
                end_for_4:;
                // b = body_parentid[b]                                                           <L 671>
                var_71 = wp::address(var_body_parentid, var_55);
                var_73 = wp::load(var_71);
                var_72 = wp::copy(var_73);
                wp::assign(var_55, var_72);
        goto start_while_2;
        end_while_2:;
        }
        // if body2 > 0:                                                                          <L 673>
        var_75 = (var_35 > var_74);
        if (var_75) {
            // b = body2                                                                          <L 674>
            var_76 = wp::copy(var_35);
            // while b > 0:                                                                       <L 675>
        start_while_6:;
            var_78 = (var_76 > var_77);
        if ((var_78) == false) goto end_while_6;
                // dofadr = body_dofadr[b]                                                        <L 676>
                var_79 = wp::address(var_body_dofadr, var_76);
                var_81 = wp::load(var_79);
                var_80 = wp::copy(var_81);
                // dofnum = body_dofnum[b]                                                        <L 677>
                var_82 = wp::address(var_body_dofnum, var_76);
                var_84 = wp::load(var_82);
                var_83 = wp::copy(var_84);
                // for dofid in range(dofadr, dofadr + dofnum):                                   <L 678>
                var_85 = wp::add(var_80, var_83);
                var_86 = wp::range(var_80, var_85);
                start_for_8:;
                    if (iter_cmp(var_86) == 0) goto end_for_8;
                    var_87 = wp::iter_next(var_86);
                    // jacp, _ = support.jac_dof(                                                 <L 679>
                    // body_parentid, body_rootid, dof_bodyid, body_isdofancestor, subtree_com_in, cdof_in, pos, body2, dofid, worldid       <L 680>
                    jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_37, var_35, var_87, var_11, var_88, var_89);
                    // wp.atomic_add(qfrc_adhesion_out, worldid, dofid, wp.dot(jacp, -force))       <L 682>
                    var_90 = wp::neg(var_52);
                    var_91 = wp::dot(var_88, var_90);
                    var_92 = wp::atomic_add(var_qfrc_adhesion_out, var_11, var_87, var_91);
                    wp::assign(var_67, var_88);
                    wp::assign(var_68, var_89);
                    goto start_for_8;
                end_for_8:;
                // b = body_parentid[b]                                                           <L 683>
                var_93 = wp::address(var_body_parentid, var_76);
                var_95 = wp::load(var_93);
                var_94 = wp::copy(var_95);
                wp::assign(var_76, var_94);
                wp::assign(var_59, var_80);
                wp::assign(var_62, var_83);
                wp::assign(var_66, var_87);
        goto start_while_6;
        end_while_6:;
        }
        var_96 = wp::where(var_75, var_76, var_55);
    }
}



extern "C" __global__ void _flex_hessian_elem_e091c54e_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_edgeadr,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::int32> var_flex_elemedgeadr,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_edge,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_vert0,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_size,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<bool> var_flex_rigid,
    wp::array_t<wp::int32> var_flex_elemflexid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<bool> var_flex_hessian_valid_in,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_flexedge_hessian_out)
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
        bool* var_5;
        bool var_6;
        bool var_7;
        bool var_8;
        bool var_9;
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        const wp::int32 var_16 = 1;
        wp::int32 var_17;
        const wp::int32 var_18 = 2;
        bool var_19;
        const wp::int32 var_20 = 3;
        const wp::int32 var_21 = 6;
        wp::int32 var_22;
        wp::int32* var_23;
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
        wp::int32 var_36;
        wp::int32* var_37;
        const wp::int32 var_38 = 3;
        bool var_39;
        const wp::int32 var_40 = 24;
        const wp::int32 var_41 = 21;
        wp::int32 var_42;
        wp::int32 var_43;
        wp::int32 var_44;
        wp::int32 var_45;
        bool var_46;
        const bool var_47 = true;
        const wp::int32 var_48 = 3;
        bool var_49;
        const wp::int32 var_50 = 21;
        wp::int32 var_51;
        wp::float32* var_52;
        const wp::float32 var_53 = 0.0;
        bool var_54;
        wp::float32 var_55;
        const wp::int32 var_56 = -1;
        const wp::int32 var_57 = -1;
        const wp::int32 var_58 = -1;
        const wp::int32 var_59 = -1;
        wp::vec_t<4, wp::int32> var_60;
        wp::mat_t<4, 3, wp::float32> var_61;
        wp::range_t var_62;
        wp::int32 var_63;
        wp::int32 var_64;
        wp::int32* var_65;
        wp::int32 var_66;
        wp::int32 var_67;
        wp::int32 var_68;
        wp::vec_t<3, wp::float32>* var_69;
        wp::vec_t<3, wp::float32> var_70;
        wp::mat_t<6, 3, wp::float32> var_71;
        const wp::float32 var_72 = 0.0;
        wp::mat_t<6, 6, wp::float32> var_73;
        const wp::float32 var_74 = 0.0;
        const wp::float32 var_75 = 0.0;
        const wp::float32 var_76 = 0.0;
        const wp::float32 var_77 = 0.0;
        const wp::float32 var_78 = 0.0;
        const wp::float32 var_79 = 0.0;
        wp::vec_t<6, wp::float32> var_80;
        const wp::float32 var_81 = 0.0;
        const wp::float32 var_82 = 0.0;
        const wp::float32 var_83 = 0.0;
        const wp::float32 var_84 = 0.0;
        const wp::float32 var_85 = 0.0;
        const wp::float32 var_86 = 0.0;
        const wp::float32 var_87 = 0.0;
        const wp::float32 var_88 = 0.0;
        const wp::float32 var_89 = 0.0;
        wp::mat_t<3, 3, wp::float32> var_90;
        wp::mat_t<4, 3, wp::float32> var_91;
        const wp::float32 var_92 = 0.0;
        const wp::float32 var_93 = 0.0;
        const wp::float32 var_94 = 0.0;
        const wp::float32 var_95 = 0.0;
        const wp::float32 var_96 = 0.0;
        const wp::float32 var_97 = 0.0;
        const wp::float32 var_98 = 0.0;
        const wp::float32 var_99 = 0.0;
        const wp::float32 var_100 = 0.0;
        wp::mat_t<3, 3, wp::float32> var_101;
        const wp::float32 var_102 = 0.0;
        const wp::float32 var_103 = 0.0;
        const wp::float32 var_104 = 0.0;
        wp::vec_t<3, wp::float32> var_105;
        const wp::float32 var_106 = 0.0;
        const wp::float32 var_107 = 0.0;
        const wp::float32 var_108 = 0.0;
        wp::vec_t<3, wp::float32> var_109;
        const wp::int32 var_110 = 6;
        wp::vec_t<6, wp::float32> var_111;
        wp::vec_t<3, wp::float32>* var_112;
        bool var_113;
        wp::mat_t<3, 3, wp::float32> var_114;
        wp::mat_t<4, 3, wp::float32> var_115;
        wp::mat_t<3, 3, wp::float32> var_116;
        wp::vec_t<3, wp::float32> var_117;
        wp::vec_t<3, wp::float32> var_118;
        wp::vec_t<3, wp::float32> var_119;
        bool var_120;
        wp::mat_t<3, 3, wp::float32> var_121;
        wp::mat_t<4, 3, wp::float32> var_122;
        wp::mat_t<3, 3, wp::float32> var_123;
        wp::vec_t<3, wp::float32> var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::mat_t<6, 6, wp::float32> var_126;
        wp::vec_t<6, wp::float32> var_127;
        wp::mat_t<6, 6, wp::float32> var_128;
        wp::vec_t<6, wp::float32> var_129;
        wp::mat_t<6, 2, wp::int32> var_130;
        wp::range_t var_131;
        wp::int32 var_132;
        const wp::int32 var_133 = 0;
        wp::int32 var_134;
        const wp::int32 var_135 = 1;
        wp::int32 var_136;
        wp::int32 var_137;
        wp::int32* var_138;
        wp::int32 var_139;
        wp::int32 var_140;
        wp::vec_t<2, wp::int32>* var_141;
        wp::vec_t<2, wp::int32> var_142;
        wp::vec_t<2, wp::int32> var_143;
        const wp::int32 var_144 = 0;
        wp::int32 var_145;
        wp::int32 var_146;
        bool var_147;
        wp::int32 var_148;
        wp::int32 var_149;
        wp::int32 var_150;
        wp::int32 var_151;
        wp::int32 var_152;
        wp::mat_t<3, 3, wp::float32> var_153;
        const wp::float32 var_154 = 1.0;
        wp::mat_t<3, 3, wp::float32> var_155;
        wp::mat_t<3, 3, wp::float32> var_156;
        wp::mat_t<3, 3, wp::float32> var_157;
        //---------
        // forward
        // def _flex_hessian_elem(                                                                <L 1668>
        // worldid, elemid = wp.tid()                                                             <L 1694>
        builtin_tid2d(var_0, var_1);
        // f = flex_elemflexid[elemid]                                                            <L 1695>
        var_2 = wp::address(var_flex_elemflexid, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // if flex_hessian_valid_in[worldid, f]:                                                  <L 1696>
        var_5 = wp::address(var_flex_hessian_valid_in, var_0, var_3);
        var_6 = wp::load(var_5);
        if (var_6) {
            // return                                                                             <L 1697>
            continue;
        }
        var_7 = wp::load(var_5);
        // if not _flex_hessian_active(                                                           <L 1698>
        // flex_dim,                                                                              <L 1699>
        // flex_interp,                                                                           <L 1700>
        // flex_stiffnessadr,                                                                     <L 1701>
        // flex_stiffness,                                                                        <L 1702>
        // flex_rigid,                                                                            <L 1703>
        // f,                                                                                     <L 1704>
        var_8 = _flex_hessian_active_0(var_flex_dim, var_flex_interp, var_flex_stiffnessadr, var_flex_stiffness, var_flex_rigid, var_3);
        var_9 = wp::unot(var_8);
        if (var_9) {
            // return                                                                             <L 1706>
            continue;
        }
        // local_elemid = elemid - flex_elemadr[f]                                                <L 1708>
        var_10 = wp::address(var_flex_elemadr, var_3);
        var_12 = wp::load(var_10);
        var_11 = wp::sub(var_1, var_12);
        // dim = flex_dim[f]                                                                      <L 1709>
        var_13 = wp::address(var_flex_dim, var_3);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // nvrt = dim + 1                                                                         <L 1710>
        var_17 = wp::add(var_14, var_16);
        // nedge = 3 if dim == 2 else 6                                                           <L 1711>
        var_19 = (var_14 == var_18);
        if (var_19) {
        }
        if (!var_19) {
        }
        var_22 = wp::where(var_19, var_20, var_21);
        // elem_data_adr = flex_elemdataadr[f] + local_elemid * nvrt                              <L 1712>
        var_23 = wp::address(var_flex_elemdataadr, var_3);
        var_24 = wp::mul(var_11, var_17);
        var_26 = wp::load(var_23);
        var_25 = wp::add(var_26, var_24);
        // vbase = flex_vertadr[f]                                                                <L 1713>
        var_27 = wp::address(var_flex_vertadr, var_3);
        var_29 = wp::load(var_27);
        var_28 = wp::copy(var_29);
        // ebase = flex_edgeadr[f]                                                                <L 1714>
        var_30 = wp::address(var_flex_edgeadr, var_3);
        var_32 = wp::load(var_30);
        var_31 = wp::copy(var_32);
        // ee_base = flex_elemedgeadr[f] + local_elemid * nedge                                   <L 1715>
        var_33 = wp::address(var_flex_elemedgeadr, var_3);
        var_34 = wp::mul(var_11, var_22);
        var_36 = wp::load(var_33);
        var_35 = wp::add(var_36, var_34);
        // stiffness_adr = flex_stiffnessadr[f] + local_elemid * (FLEX_STIFFNESS_3D if dim == 3 else 21)       <L 1716>
        var_37 = wp::address(var_flex_stiffnessadr, var_3);
        var_39 = (var_14 == var_38);
        if (var_39) {
        }
        if (!var_39) {
        }
        var_42 = wp::where(var_39, var_40, var_41);
        var_43 = wp::mul(var_11, var_42);
        var_45 = wp::load(var_37);
        var_44 = wp::add(var_45, var_43);
        // snh = wp.static(FLEX_STIFFNESS_3D == 24) and dim == 3 and flex_stiffness[stiffness_adr + 21] != 0.0       <L 1717>
        var_46 = var_47;
        if (var_46) {
            var_49 = (var_14 == var_48);
            var_46 = var_46 && var_49;
        }
        if (var_46) {
            var_51 = wp::add(var_44, var_50);
            var_52 = wp::address(var_flex_stiffness, var_51);
            var_55 = wp::load(var_52);
            var_54 = (var_55 != var_53);
            var_46 = var_46 && var_54;
        }
        // elem_verts = wp.vec4i(-1, -1, -1, -1)                                                  <L 1719>
        var_60 = wp::vec_t<4, wp::int32>(var_56, var_57, var_58, var_59);
        // vert_xpos = mat43()                                                                    <L 1720>
        var_61 = wp::mat_t<4, 3, wp::float32>();
        // for v in range(nvrt):                                                                  <L 1721>
        var_62 = wp::range(var_17);
        start_for_2:;
            if (iter_cmp(var_62) == 0) goto end_for_2;
            var_63 = wp::iter_next(var_62);
            // vert_idx = flex_elem[elem_data_adr + v]                                            <L 1722>
            var_64 = wp::add(var_25, var_63);
            var_65 = wp::address(var_flex_elem, var_64);
            var_67 = wp::load(var_65);
            var_66 = wp::copy(var_67);
            // elem_verts[v] = vert_idx                                                           <L 1723>
            wp::assign_inplace(var_60, var_63, var_66);
            // vert_xpos[v] = flexvert_xpos_in[worldid, vbase + vert_idx]                         <L 1724>
            var_68 = wp::add(var_28, var_66);
            var_69 = wp::address(var_flexvert_xpos_in, var_0, var_68);
            var_70 = wp::load(var_69);
            wp::assign_inplace(var_61, var_63, var_70);
            goto start_for_2;
        end_for_2:;
        // edgevec = stretch_edge_vectors(vert_xpos, dim)                                         <L 1726>
        var_71 = stretch_edge_vectors_0(var_61, var_14);
        // metric = mat66(0.0)                                                                    <L 1728>
        var_73 = wp::mat_t<6, 6, wp::float32>(var_72);
        // tension = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                           <L 1729>
        var_80 = wp::vec_t<6, wp::float32>({var_74, var_75, var_76, var_77, var_78, var_79});
        // rotation = wp.mat33(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                       <L 1730>
        var_90 = wp::mat_t<3, 3, wp::float32>(var_81, var_82, var_83, var_84, var_85, var_86, var_87, var_88, var_89);
        // proj_grad = mat43()                                                                    <L 1731>
        var_91 = wp::mat_t<4, 3, wp::float32>();
        // stretch = wp.mat33(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                        <L 1732>
        var_101 = wp::mat_t<3, 3, wp::float32>(var_92, var_93, var_94, var_95, var_96, var_97, var_98, var_99, var_100);
        // symmetric = wp.vec3(0.0, 0.0, 0.0)                                                     <L 1733>
        var_105 = wp::vec_t<3, wp::float32>(var_102, var_103, var_104);
        // skew = wp.vec3(0.0, 0.0, 0.0)                                                          <L 1734>
        var_109 = wp::vec_t<3, wp::float32>(var_106, var_107, var_108);
        // if snh:                                                                                <L 1736>
        if (var_46) {
            // elongation = stretch_elongation(flex_elemedge, flexedge_length0, flexedge_length_in, worldid, ee_base, ebase, 6)       <L 1737>
            var_111 = stretch_elongation_0(var_flex_elemedge, var_flexedge_length0, var_flexedge_length_in, var_0, var_35, var_31, var_110);
            // active, rotation, proj_grad, stretch, symmetric, skew = snh_project(               <L 1738>
            // flex_vert0,                                                                        <L 1739>
            // flex_stiffness,                                                                    <L 1740>
            // stiffness_adr,                                                                     <L 1741>
            // vbase,                                                                             <L 1742>
            // elem_verts,                                                                        <L 1743>
            // flex_size[f],                                                                      <L 1744>
            var_112 = wp::address(var_flex_size, var_3);
            // edgevec,                                                                           <L 1745>
            // elongation,                                                                        <L 1746>
            var_119 = wp::load(var_112);
            snh_project_0(var_flex_vert0, var_flex_stiffness, var_44, var_28, var_60, var_119, var_71, var_111, var_113, var_114, var_115, var_116, var_117, var_118);
            // if not active:                                                                     <L 1748>
            var_120 = wp::unot(var_113);
            if (var_120) {
                // return                                                                         <L 1749>
                continue;
            }
        }
        var_121 = wp::where(var_46, var_114, var_90);
        var_122 = wp::where(var_46, var_115, var_91);
        var_123 = wp::where(var_46, var_116, var_101);
        var_124 = wp::where(var_46, var_117, var_105);
        var_125 = wp::where(var_46, var_118, var_109);
        if (!var_46) {
            // metric, tension = stretch_stiffness(                                               <L 1751>
            // flex_elemedge, flexedge_length0, flex_stiffness, flexedge_length_in, worldid, stiffness_adr, ee_base, ebase, nedge       <L 1752>
            stretch_stiffness_0(var_flex_elemedge, var_flexedge_length0, var_flex_stiffness, var_flexedge_length_in, var_0, var_44, var_35, var_31, var_22, var_126, var_127);
        }
        var_128 = wp::where(var_46, var_73, var_126);
        var_129 = wp::where(var_46, var_80, var_127);
        // edges = stretch_edge_endpoints(dim)                                                    <L 1755>
        var_130 = stretch_edge_endpoints_0(var_14);
        // for e in range(nedge):                                                                 <L 1756>
        var_131 = wp::range(var_22);
        start_for_5:;
            if (iter_cmp(var_131) == 0) goto end_for_5;
            var_132 = wp::iter_next(var_131);
            // i = edges[e, 0]                                                                    <L 1757>
            var_134 = wp::extract(var_130, var_132, var_133);
            // j = edges[e, 1]                                                                    <L 1758>
            var_136 = wp::extract(var_130, var_132, var_135);
            // edge_id = ebase + flex_elemedge[ee_base + e]                                       <L 1759>
            var_137 = wp::add(var_35, var_132);
            var_138 = wp::address(var_flex_elemedge, var_137);
            var_140 = wp::load(var_138);
            var_139 = wp::add(var_31, var_140);
            // endpoints = flex_edge[edge_id]                                                     <L 1760>
            var_141 = wp::address(var_flex_edge, var_139);
            var_143 = wp::load(var_141);
            var_142 = wp::copy(var_143);
            // if endpoints[0] != elem_verts[i]:                                                  <L 1761>
            var_145 = wp::extract(var_142, var_144);
            var_146 = wp::extract(var_60, var_134);
            var_147 = (var_145 != var_146);
            if (var_147) {
                // swap = i                                                                       <L 1762>
                var_148 = wp::copy(var_134);
                // i = j                                                                          <L 1763>
                var_149 = wp::copy(var_136);
                // j = swap                                                                       <L 1764>
                var_150 = wp::copy(var_148);
            }
            var_151 = wp::where(var_147, var_149, var_134);
            var_152 = wp::where(var_147, var_150, var_136);
            // if snh:                                                                            <L 1765>
            if (var_46) {
                // block = snh_projected_block(rotation, proj_grad, stretch, symmetric, skew, i, j)       <L 1766>
                var_153 = snh_projected_block_0(var_121, var_122, var_123, var_124, var_125, var_151, var_152);
            }
            if (!var_46) {
                // block = stretch_stiffness_block(metric, tension, edgevec, dim, i, j, 1.0)       <L 1768>
                var_155 = stretch_stiffness_block_0(var_128, var_129, var_71, var_14, var_151, var_152, var_154);
            }
            var_156 = wp::where(var_46, var_153, var_155);
            // wp.atomic_add(flexedge_hessian_out, worldid, edge_id, block)                       <L 1769>
            var_157 = wp::atomic_add(var_flexedge_hessian_out, var_0, var_139, var_156);
            goto start_for_5;
        end_for_5:;
    }
}



extern "C" __global__ void _flex_passive_bend_interp_6fd16e7b_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nflex,
    wp::array_t<wp::int32> var_flex_interp,
    wp::array_t<wp::vec_t<3, wp::int32>> var_flex_cellnum,
    wp::array_t<wp::int32> var_flex_nodeadr,
    wp::array_t<wp::int32> var_flex_nodenum,
    wp::array_t<wp::int32> var_flex_bendingadr,
    wp::array_t<wp::int32> var_flex_nodebodyid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_node,
    wp::array_t<wp::float32> var_flex_bending,
    wp::array_t<bool> var_flex_centered,
    wp::array_t<wp::int32> var_flex_faceadr,
    wp::array_t<wp::vec_t<2, wp::int32>> var_flex_bend_interp_map,
    wp::array_t<wp::int32> var_flex_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexnode_xpos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_xpos_in,
    wp::array_t<wp::quat_t<wp::float32>> var_face_quat_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_spring_body_force_out)
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
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        wp::vec_t<3, wp::int32>* var_16;
        wp::vec_t<3, wp::int32> var_17;
        wp::vec_t<3, wp::int32> var_18;
        const wp::int32 var_19 = 0;
        wp::int32 var_20;
        const wp::int32 var_21 = 1;
        wp::int32 var_22;
        const wp::int32 var_23 = 2;
        wp::int32 var_24;
        wp::int32* var_25;
        wp::int32 var_26;
        wp::int32 var_27;
        const wp::int32 var_28 = 1;
        wp::int32 var_29;
        const wp::int32 var_30 = 10;
        wp::int32 var_31;
        wp::int32 var_32;
        const wp::int32 var_33 = 0;
        wp::int32 var_34;
        wp::float32* var_35;
        wp::int32 var_36;
        wp::float32 var_37;
        const wp::int32 var_38 = 1;
        wp::int32 var_39;
        wp::float32* var_40;
        wp::int32 var_41;
        wp::float32 var_42;
        const wp::int32 var_43 = 2;
        wp::int32 var_44;
        wp::float32* var_45;
        const wp::int32 var_46 = 3;
        wp::int32 var_47;
        wp::float32* var_48;
        wp::vec_t<2, wp::float32> var_49;
        wp::float32 var_50;
        wp::float32 var_51;
        const wp::int32 var_52 = 4;
        wp::int32 var_53;
        wp::float32* var_54;
        const wp::int32 var_55 = 5;
        wp::int32 var_56;
        wp::float32* var_57;
        wp::vec_t<2, wp::float32> var_58;
        wp::float32 var_59;
        wp::float32 var_60;
        const wp::int32 var_61 = 6;
        wp::int32 var_62;
        wp::float32* var_63;
        wp::float32 var_64;
        wp::float32 var_65;
        const wp::int32 var_66 = 7;
        wp::int32 var_67;
        wp::float32* var_68;
        const wp::int32 var_69 = 8;
        wp::int32 var_70;
        wp::float32* var_71;
        const wp::int32 var_72 = 9;
        wp::int32 var_73;
        wp::float32* var_74;
        wp::vec_t<3, wp::float32> var_75;
        wp::float32 var_76;
        wp::float32 var_77;
        wp::float32 var_78;
        const wp::float32 var_79 = 0.0;
        bool var_80;
        wp::int32* var_81;
        wp::int32 var_82;
        wp::int32 var_83;
        wp::int32* var_84;
        wp::int32 var_85;
        wp::int32 var_86;
        wp::quat_t<wp::float32>* var_87;
        wp::quat_t<wp::float32> var_88;
        wp::quat_t<wp::float32> var_89;
        wp::quat_t<wp::float32>* var_90;
        wp::quat_t<wp::float32> var_91;
        wp::quat_t<wp::float32> var_92;
        const wp::float32 var_93 = 0.0;
        wp::vec_t<3, wp::float32> var_94;
        const wp::float32 var_95 = 0.0;
        wp::vec_t<3, wp::float32> var_96;
        const wp::float32 var_97 = 0.0;
        wp::vec_t<3, wp::float32> var_98;
        const wp::float32 var_99 = 0.0;
        wp::vec_t<3, wp::float32> var_100;
        const wp::int32 var_101 = 0;
        wp::int32 var_102;
        const wp::int32 var_103 = 3;
        wp::range_t var_104;
        wp::int32 var_105;
        bool var_106;
        const wp::int32 var_107 = 3;
        wp::range_t var_108;
        wp::int32 var_109;
        bool var_110;
        wp::vec_t<3, wp::float32>* var_111;
        wp::vec_t<3, wp::float32> var_112;
        wp::vec_t<3, wp::float32> var_113;
        wp::vec_t<3, wp::float32>* var_114;
        wp::vec_t<3, wp::float32> var_115;
        wp::vec_t<3, wp::float32> var_116;
        const wp::int32 var_117 = 0;
        wp::float32 var_118;
        wp::float32 var_119;
        const wp::int32 var_120 = 1;
        wp::float32 var_121;
        wp::float32 var_122;
        wp::float32 var_123;
        const wp::int32 var_124 = 0;
        wp::float32 var_125;
        wp::float32 var_126;
        const wp::int32 var_127 = 1;
        wp::float32 var_128;
        wp::float32 var_129;
        wp::float32 var_130;
        const wp::int32 var_131 = 0;
        wp::float32 var_132;
        wp::float32 var_133;
        const wp::int32 var_134 = 1;
        wp::float32 var_135;
        wp::float32 var_136;
        wp::float32 var_137;
        const wp::int32 var_138 = 0;
        wp::float32 var_139;
        wp::float32 var_140;
        const wp::int32 var_141 = 1;
        wp::float32 var_142;
        wp::float32 var_143;
        wp::float32 var_144;
        wp::vec_t<3, wp::float32> var_145;
        wp::vec_t<3, wp::float32> var_146;
        wp::vec_t<3, wp::float32> var_147;
        wp::vec_t<3, wp::float32> var_148;
        wp::vec_t<3, wp::float32> var_149;
        wp::vec_t<3, wp::float32> var_150;
        wp::vec_t<3, wp::float32> var_151;
        wp::vec_t<3, wp::float32> var_152;
        const wp::int32 var_153 = 1;
        wp::int32 var_154;
        wp::vec_t<3, wp::float32> var_155;
        wp::vec_t<3, wp::float32> var_156;
        wp::float32 var_157;
        wp::float32 var_158;
        bool var_159;
        const wp::float32 var_160 = 1e-15;
        bool var_161;
        bool var_162;
        const wp::float32 var_163 = 1.0;
        wp::float32 var_164;
        const wp::float32 var_165 = 1.0;
        wp::float32 var_166;
        wp::vec_t<3, wp::float32> var_167;
        wp::vec_t<3, wp::float32> var_168;
        wp::float32 var_169;
        const wp::float32 var_170 = 0.0;
        bool var_171;
        wp::quat_t<wp::float32> var_172;
        wp::quat_t<wp::float32> var_173;
        wp::quat_t<wp::float32> var_174;
        wp::quat_t<wp::float32> var_175;
        wp::vec_t<3, wp::float32> var_176;
        wp::vec_t<3, wp::float32> var_177;
        wp::vec_t<3, wp::float32> var_178;
        wp::float32 var_179;
        wp::vec_t<3, wp::float32> var_180;
        wp::vec_t<3, wp::float32> var_181;
        wp::vec_t<3, wp::float32> var_182;
        wp::float32 var_183;
        wp::vec_t<3, wp::float32> var_184;
        wp::vec_t<3, wp::float32> var_185;
        wp::vec_t<3, wp::float32> var_186;
        wp::vec_t<3, wp::float32> var_187;
        wp::vec_t<3, wp::float32> var_188;
        wp::vec_t<3, wp::float32> var_189;
        wp::vec_t<3, wp::float32> var_190;
        wp::float32 var_191;
        //---------
        // forward
        // def _flex_passive_bend_interp(                                                         <L 2810>
        // worldid, bend_edge_id = wp.tid()                                                       <L 2833>
        builtin_tid2d(var_0, var_1);
        // mapping = flex_bend_interp_map[bend_edge_id]                                           <L 2835>
        var_2 = wp::address(var_flex_bend_interp_map, var_1);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // f = mapping[0]                                                                         <L 2836>
        var_6 = wp::extract(var_3, var_5);
        // e = mapping[1]                                                                         <L 2837>
        var_8 = wp::extract(var_3, var_7);
        // order = flex_interp[f]                                                                 <L 2839>
        var_9 = wp::address(var_flex_interp, var_6);
        var_11 = wp::load(var_9);
        var_10 = wp::copy(var_11);
        // order_abs = -order                                                                     <L 2840>
        var_12 = wp::neg(var_10);
        // bendingadr = flex_bendingadr[f]                                                        <L 2841>
        var_13 = wp::address(var_flex_bendingadr, var_6);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // cellnum = flex_cellnum[f]                                                              <L 2843>
        var_16 = wp::address(var_flex_cellnum, var_6);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // cx = cellnum[0]                                                                        <L 2844>
        var_20 = wp::extract(var_17, var_19);
        // cy = cellnum[1]                                                                        <L 2845>
        var_22 = wp::extract(var_17, var_21);
        // cz = cellnum[2]                                                                        <L 2846>
        var_24 = wp::extract(var_17, var_23);
        // nstart = flex_nodeadr[f]                                                               <L 2847>
        var_25 = wp::address(var_flex_nodeadr, var_6);
        var_27 = wp::load(var_25);
        var_26 = wp::copy(var_27);
        // edata_base = bendingadr + 1 + e * 10                                                   <L 2849>
        var_29 = wp::add(var_14, var_28);
        var_31 = wp::mul(var_8, var_30);
        var_32 = wp::add(var_29, var_31);
        // fe_A = int(flex_bending[edata_base + 0])                                               <L 2850>
        var_34 = wp::add(var_32, var_33);
        var_35 = wp::address(var_flex_bending, var_34);
        var_37 = wp::load(var_35);
        var_36 = wp::int(var_37);
        // fe_B = int(flex_bending[edata_base + 1])                                               <L 2851>
        var_39 = wp::add(var_32, var_38);
        var_40 = wp::address(var_flex_bending, var_39);
        var_42 = wp::load(var_40);
        var_41 = wp::int(var_42);
        // local_A = wp.vec2(flex_bending[edata_base + 2], flex_bending[edata_base + 3])          <L 2852>
        var_44 = wp::add(var_32, var_43);
        var_45 = wp::address(var_flex_bending, var_44);
        var_47 = wp::add(var_32, var_46);
        var_48 = wp::address(var_flex_bending, var_47);
        var_50 = wp::load(var_45);
        var_51 = wp::load(var_48);
        var_49 = wp::vec_t<2, wp::float32>(var_50, var_51);
        // local_B = wp.vec2(flex_bending[edata_base + 4], flex_bending[edata_base + 5])          <L 2853>
        var_53 = wp::add(var_32, var_52);
        var_54 = wp::address(var_flex_bending, var_53);
        var_56 = wp::add(var_32, var_55);
        var_57 = wp::address(var_flex_bending, var_56);
        var_59 = wp::load(var_54);
        var_60 = wp::load(var_57);
        var_58 = wp::vec_t<2, wp::float32>(var_59, var_60);
        // stiffness = flex_bending[edata_base + 6]                                               <L 2854>
        var_62 = wp::add(var_32, var_61);
        var_63 = wp::address(var_flex_bending, var_62);
        var_65 = wp::load(var_63);
        var_64 = wp::copy(var_65);
        // dn0 = wp.vec3(flex_bending[edata_base + 7], flex_bending[edata_base + 8], flex_bending[edata_base + 9])       <L 2855>
        var_67 = wp::add(var_32, var_66);
        var_68 = wp::address(var_flex_bending, var_67);
        var_70 = wp::add(var_32, var_69);
        var_71 = wp::address(var_flex_bending, var_70);
        var_73 = wp::add(var_32, var_72);
        var_74 = wp::address(var_flex_bending, var_73);
        var_76 = wp::load(var_68);
        var_77 = wp::load(var_71);
        var_78 = wp::load(var_74);
        var_75 = wp::vec_t<3, wp::float32>(var_76, var_77, var_78);
        // if stiffness <= 0.0:                                                                   <L 2857>
        var_80 = (var_64 <= var_79);
        if (var_80) {
            // return                                                                             <L 2858>
            continue;
        }
        // face_id_A = flex_faceadr[f] + fe_A                                                     <L 2861>
        var_81 = wp::address(var_flex_faceadr, var_6);
        var_83 = wp::load(var_81);
        var_82 = wp::add(var_83, var_36);
        // face_id_B = flex_faceadr[f] + fe_B                                                     <L 2862>
        var_84 = wp::address(var_flex_faceadr, var_6);
        var_86 = wp::load(var_84);
        var_85 = wp::add(var_86, var_41);
        // quat_A = face_quat_in[worldid, face_id_A]                                              <L 2864>
        var_87 = wp::address(var_face_quat_in, var_0, var_82);
        var_89 = wp::load(var_87);
        var_88 = wp::copy(var_89);
        // quat_B = face_quat_in[worldid, face_id_B]                                              <L 2865>
        var_90 = wp::address(var_face_quat_in, var_0, var_85);
        var_92 = wp::load(var_90);
        var_91 = wp::copy(var_92);
        // t1_A = wp.vec3(0.0)                                                                    <L 2868>
        var_94 = wp::vec_t<3, wp::float32>(var_93);
        // t2_A = wp.vec3(0.0)                                                                    <L 2869>
        var_96 = wp::vec_t<3, wp::float32>(var_95);
        // t1_B = wp.vec3(0.0)                                                                    <L 2870>
        var_98 = wp::vec_t<3, wp::float32>(var_97);
        // t2_B = wp.vec3(0.0)                                                                    <L 2871>
        var_100 = wp::vec_t<3, wp::float32>(var_99);
        // idx = int(0)                                                                           <L 2873>
        var_102 = wp::int(var_101);
        // for l0 in range(3):                                                                    <L 2874>
        var_104 = wp::range(var_103);
        start_for_1:;
            if (iter_cmp(var_104) == 0) goto end_for_1;
            var_105 = wp::iter_next(var_104);
            // if l0 > order_abs:                                                                 <L 2875>
            var_106 = (var_105 > var_12);
            if (var_106) {
                // continue                                                                       <L 2876>
                goto start_for_1;
            }
            // for l1 in range(3):                                                                <L 2877>
            var_108 = wp::range(var_107);
            start_for_3:;
                if (iter_cmp(var_108) == 0) goto end_for_3;
                var_109 = wp::iter_next(var_108);
                // if l1 > order_abs:                                                             <L 2878>
                var_110 = (var_109 > var_12);
                if (var_110) {
                    // continue                                                                   <L 2879>
                    goto start_for_3;
                }
                // pos_A = face_xpos_in[worldid, face_id_A, idx]                                  <L 2880>
                var_111 = wp::address(var_face_xpos_in, var_0, var_82, var_102);
                var_113 = wp::load(var_111);
                var_112 = wp::copy(var_113);
                // pos_B = face_xpos_in[worldid, face_id_B, idx]                                  <L 2881>
                var_114 = wp::address(var_face_xpos_in, var_0, var_85, var_102);
                var_116 = wp::load(var_114);
                var_115 = wp::copy(var_116);
                // grad0_A = support.flex_dphi(local_A[0], l0, order_abs) * support.flex_phi(local_A[1], l1, order_abs)       <L 2883>
                var_118 = wp::extract(var_49, var_117);
                var_119 = flex_dphi_0(var_118, var_105, var_12);
                var_121 = wp::extract(var_49, var_120);
                var_122 = flex_phi_0(var_121, var_109, var_12);
                var_123 = wp::mul(var_119, var_122);
                // grad1_A = support.flex_phi(local_A[0], l0, order_abs) * support.flex_dphi(local_A[1], l1, order_abs)       <L 2884>
                var_125 = wp::extract(var_49, var_124);
                var_126 = flex_phi_0(var_125, var_105, var_12);
                var_128 = wp::extract(var_49, var_127);
                var_129 = flex_dphi_0(var_128, var_109, var_12);
                var_130 = wp::mul(var_126, var_129);
                // grad0_B = support.flex_dphi(local_B[0], l0, order_abs) * support.flex_phi(local_B[1], l1, order_abs)       <L 2886>
                var_132 = wp::extract(var_58, var_131);
                var_133 = flex_dphi_0(var_132, var_105, var_12);
                var_135 = wp::extract(var_58, var_134);
                var_136 = flex_phi_0(var_135, var_109, var_12);
                var_137 = wp::mul(var_133, var_136);
                // grad1_B = support.flex_phi(local_B[0], l0, order_abs) * support.flex_dphi(local_B[1], l1, order_abs)       <L 2887>
                var_139 = wp::extract(var_58, var_138);
                var_140 = flex_phi_0(var_139, var_105, var_12);
                var_142 = wp::extract(var_58, var_141);
                var_143 = flex_dphi_0(var_142, var_109, var_12);
                var_144 = wp::mul(var_140, var_143);
                // t1_A += pos_A * grad0_A                                                        <L 2889>
                var_145 = wp::mul(var_112, var_123);
                var_146 = wp::add(var_94, var_145);
                // t2_A += pos_A * grad1_A                                                        <L 2890>
                var_147 = wp::mul(var_112, var_130);
                var_148 = wp::add(var_96, var_147);
                // t1_B += pos_B * grad0_B                                                        <L 2892>
                var_149 = wp::mul(var_115, var_137);
                var_150 = wp::add(var_98, var_149);
                // t2_B += pos_B * grad1_B                                                        <L 2893>
                var_151 = wp::mul(var_115, var_144);
                var_152 = wp::add(var_100, var_151);
                // idx += 1                                                                       <L 2894>
                var_154 = wp::add(var_102, var_153);
                wp::assign(var_94, var_146);
                wp::assign(var_96, var_148);
                wp::assign(var_98, var_150);
                wp::assign(var_100, var_152);
                wp::assign(var_102, var_154);
                goto start_for_3;
            end_for_3:;
            goto start_for_1;
        end_for_1:;
        // n_A = wp.cross(t1_A, t2_A)                                                             <L 2896>
        var_155 = wp::cross(var_94, var_96);
        // n_B = wp.cross(t1_B, t2_B)                                                             <L 2897>
        var_156 = wp::cross(var_98, var_100);
        // len_A = wp.length(n_A)                                                                 <L 2899>
        var_157 = wp::length(var_155);
        // len_B = wp.length(n_B)                                                                 <L 2900>
        var_158 = wp::length(var_156);
        // if len_A < MJ_MINVAL or len_B < MJ_MINVAL:                                             <L 2902>
        var_161 = (var_157 < var_160);
        var_159 = var_161;
        if (!var_159) {
            var_162 = (var_158 < var_160);
            var_159 = var_159 || var_162;
        }
        if (var_159) {
            // return                                                                             <L 2903>
            continue;
        }
        // inv_A = 1.0 / len_A                                                                    <L 2905>
        var_164 = wp::div(var_163, var_157);
        // inv_B = 1.0 / len_B                                                                    <L 2906>
        var_166 = wp::div(var_165, var_158);
        // n_A_norm = n_A * inv_A                                                                 <L 2907>
        var_167 = wp::mul(var_155, var_164);
        // n_B_norm = n_B * inv_B                                                                 <L 2908>
        var_168 = wp::mul(var_156, var_166);
        // if wp.dot(quat_A, quat_B) < 0.0:                                                       <L 2911>
        var_169 = wp::dot(var_88, var_91);
        var_171 = (var_169 < var_170);
        if (var_171) {
            // quat_B = -quat_B                                                                   <L 2912>
            var_172 = wp::neg(var_91);
        }
        var_173 = wp::where(var_171, var_172, var_91);
        // quat_avg = quat_A + quat_B                                                             <L 2913>
        var_174 = wp::add(var_88, var_173);
        // quat_avg = wp.normalize(quat_avg)                                                      <L 2914>
        var_175 = wp::normalize(var_174);
        // dn0_rot = wp.quat_rotate(quat_avg, dn0)                                                <L 2917>
        var_176 = wp::quat_rotate(var_175, var_75);
        // r = n_A_norm - n_B_norm - dn0_rot                                                      <L 2920>
        var_177 = wp::sub(var_167, var_168);
        var_178 = wp::sub(var_177, var_176);
        // dot_A = wp.dot(n_A_norm, r)                                                            <L 2923>
        var_179 = wp::dot(var_167, var_178);
        // w_A = (r - n_A_norm * dot_A) * inv_A                                                   <L 2924>
        var_180 = wp::mul(var_167, var_179);
        var_181 = wp::sub(var_178, var_180);
        var_182 = wp::mul(var_181, var_164);
        // dot_B = wp.dot(n_B_norm, r)                                                            <L 2926>
        var_183 = wp::dot(var_168, var_178);
        // w_B = (r - n_B_norm * dot_B) * inv_B                                                   <L 2927>
        var_184 = wp::mul(var_168, var_183);
        var_185 = wp::sub(var_178, var_184);
        var_186 = wp::mul(var_185, var_166);
        // wAt2 = wp.cross(w_A, t2_A)                                                             <L 2929>
        var_187 = wp::cross(var_182, var_96);
        // wAt1 = wp.cross(w_A, t1_A)                                                             <L 2930>
        var_188 = wp::cross(var_182, var_94);
        // wBt2 = wp.cross(w_B, t2_B)                                                             <L 2931>
        var_189 = wp::cross(var_186, var_100);
        // wBt1 = wp.cross(w_B, t1_B)                                                             <L 2932>
        var_190 = wp::cross(var_186, var_98);
        // _apply_face_forces(                                                                    <L 2935>
        // flex_nodebodyid,                                                                       <L 2936>
        // flex_face,                                                                             <L 2937>
        // xipos_in,                                                                              <L 2938>
        // flexnode_xpos_in,                                                                      <L 2939>
        // face_id_A,                                                                             <L 2940>
        // local_A,                                                                               <L 2941>
        // wAt1,                                                                                  <L 2942>
        // wAt2,                                                                                  <L 2943>
        // stiffness,                                                                             <L 2944>
        // order_abs,                                                                             <L 2945>
        // worldid,                                                                               <L 2946>
        // flex_spring_body_force_out,                                                            <L 2947>
        _apply_face_forces_0(var_flex_nodebodyid, var_flex_face, var_xipos_in, var_flexnode_xpos_in, var_82, var_49, var_188, var_187, var_64, var_12, var_0, var_flex_spring_body_force_out);
        // _apply_face_forces(                                                                    <L 2951>
        // flex_nodebodyid,                                                                       <L 2952>
        // flex_face,                                                                             <L 2953>
        // xipos_in,                                                                              <L 2954>
        // flexnode_xpos_in,                                                                      <L 2955>
        // face_id_B,                                                                             <L 2956>
        // local_B,                                                                               <L 2957>
        // wBt1,                                                                                  <L 2958>
        // wBt2,                                                                                  <L 2959>
        // -stiffness,                                                                            <L 2960>
        var_191 = wp::neg(var_64);
        // order_abs,                                                                             <L 2961>
        // worldid,                                                                               <L 2962>
        // flex_spring_body_force_out,                                                            <L 2963>
        _apply_face_forces_0(var_flex_nodebodyid, var_flex_face, var_xipos_in, var_flexnode_xpos_in, var_85, var_58, var_190, var_189, var_191, var_12, var_0, var_flex_spring_body_force_out);
    }
}



extern "C" __global__ void _eff_contact_build_f1a6f426_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::float32> var_opt_timestep,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_body_dofnum,
    wp::array_t<wp::int32> var_body_dofadr,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_M_rownnz,
    wp::array_t<wp::int32> var_M_rowadr,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_xmat_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::float32> var_M_in,
    wp::array_t<wp::float32> var_contact_dist_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_contact_pos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_geom_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_flex_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_elem_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_vert_in,
    wp::array_t<wp::int32> var_contact_worldid_in,
    wp::array_t<wp::int32> var_contact_type_in,
    wp::array_t<wp::int32> var_nacon_in,
    wp::array_t<wp::int32> var_efm_con_dof_out,
    wp::array_t<wp::float32> var_efm_con_val_out,
    wp::array_t<wp::float32> var_efm_con_scale_out,
    wp::array_t<wp::float32> var_efm_con_force_out,
    wp::array_t<wp::int32> var_efm_con_nnz_out)
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
        wp::int32* var_5;
        const wp::int32 var_6 = 4;
        const wp::int32 var_7 = 4;
        wp::int32 var_8;
        wp::int32 var_9;
        wp::int32 var_10;
        bool var_11;
        bool var_12;
        const wp::int32 var_13 = 0;
        const wp::float32 var_14 = 0.0;
        wp::float32 var_15;
        const wp::float32 var_16 = 0.0;
        wp::float32 var_17;
        wp::int32* var_18;
        wp::int32 var_19;
        wp::int32 var_20;
        wp::shape_t* var_21;
        const wp::int32 var_22 = 0;
        wp::int32 var_23;
        wp::shape_t var_24;
        wp::int32 var_25;
        wp::float32* var_26;
        wp::float32 var_27;
        wp::float32 var_28;
        wp::vec_t<2, wp::int32>* var_29;
        const wp::int32 var_30 = 0;
        wp::int32 var_31;
        wp::vec_t<2, wp::int32> var_32;
        wp::vec_t<2, wp::int32>* var_33;
        const wp::int32 var_34 = 1;
        wp::int32 var_35;
        wp::vec_t<2, wp::int32> var_36;
        wp::vec_t<2, wp::int32>* var_37;
        const wp::int32 var_38 = 0;
        wp::int32 var_39;
        wp::vec_t<2, wp::int32> var_40;
        wp::vec_t<2, wp::int32>* var_41;
        const wp::int32 var_42 = 1;
        wp::int32 var_43;
        wp::vec_t<2, wp::int32> var_44;
        wp::vec_t<2, wp::int32>* var_45;
        const wp::int32 var_46 = 0;
        wp::int32 var_47;
        wp::vec_t<2, wp::int32> var_48;
        wp::vec_t<2, wp::int32>* var_49;
        const wp::int32 var_50 = 1;
        wp::int32 var_51;
        wp::vec_t<2, wp::int32> var_52;
        wp::float32 var_53;
        const wp::float32 var_54 = 0.0;
        wp::float32 var_55;
        bool var_56;
        const wp::int32 var_57 = 0;
        const wp::float32 var_58 = 0.0;
        wp::float32 var_59;
        const wp::float32 var_60 = 0.0;
        wp::float32 var_61;
        wp::float32* var_62;
        wp::float32 var_63;
        wp::float32 var_64;
        wp::vec_t<3, wp::float32>* var_65;
        wp::vec_t<3, wp::float32> var_66;
        wp::vec_t<3, wp::float32> var_67;
        wp::mat_t<3, 3, wp::float32>* var_68;
        const wp::int32 var_69 = 0;
        wp::vec_t<3, wp::float32> var_70;
        wp::mat_t<3, 3, wp::float32> var_71;
        const wp::int32 var_72 = 0;
        wp::int32 var_73;
        const wp::int32 var_74 = 0;
        bool var_75;
        const wp::int32 var_76 = 0;
        bool var_77;
        wp::int32* var_78;
        wp::int32 var_79;
        wp::int32 var_80;
        wp::int32* var_81;
        wp::int32* var_82;
        wp::int32 var_83;
        wp::int32 var_84;
        wp::int32 var_85;
        wp::int32* var_86;
        const wp::int32 var_87 = 3;
        bool var_88;
        wp::int32 var_89;
        wp::int32* var_90;
        wp::int32 var_91;
        wp::int32 var_92;
        wp::mat_t<3, 3, wp::float32>* var_93;
        wp::mat_t<3, 3, wp::float32> var_94;
        wp::mat_t<3, 3, wp::float32> var_95;
        wp::vec_t<3, wp::float32> var_96;
        const wp::int32 var_97 = 0;
        wp::int32 var_98;
        wp::float32 var_99;
        wp::float32 var_100;
        wp::int32 var_101;
        const wp::int32 var_102 = 1;
        wp::int32 var_103;
        wp::float32 var_104;
        wp::float32 var_105;
        wp::int32 var_106;
        const wp::int32 var_107 = 2;
        wp::int32 var_108;
        wp::float32 var_109;
        wp::float32 var_110;
        wp::int32 var_111;
        wp::int32 var_112;
        wp::int32 var_113;
        const wp::int32 var_114 = 0;
        bool var_115;
        wp::int32* var_116;
        wp::int32 var_117;
        wp::int32 var_118;
        const wp::int32 var_119 = 1;
        wp::int32 var_120;
        wp::int32* var_121;
        wp::int32 var_122;
        wp::int32 var_123;
        wp::int32 var_124;
        const wp::float32 var_125 = 0.0;
        const wp::float32 var_126 = 0.0;
        const wp::float32 var_127 = 0.0;
        const wp::float32 var_128 = 0.0;
        wp::vec_t<4, wp::float32> var_129;
        const wp::int32 var_130 = -1;
        const wp::int32 var_131 = -1;
        const wp::int32 var_132 = -1;
        const wp::int32 var_133 = -1;
        wp::vec_t<4, wp::int32> var_134;
        const wp::int32 var_135 = 0;
        wp::int32 var_136;
        const wp::float32 var_137 = 0.0;
        wp::float32 var_138;
        wp::range_t var_139;
        wp::int32 var_140;
        wp::int32 var_141;
        wp::int32* var_142;
        wp::int32 var_143;
        wp::int32 var_144;
        bool var_145;
        const wp::int32 var_146 = 0;
        bool var_147;
        bool var_148;
        wp::int32* var_149;
        wp::int32 var_150;
        wp::int32 var_151;
        wp::vec_t<3, wp::float32>* var_152;
        wp::vec_t<3, wp::float32> var_153;
        wp::vec_t<3, wp::float32> var_154;
        wp::vec_t<3, wp::float32> var_155;
        wp::float32 var_156;
        const wp::float32 var_157 = 1.0;
        wp::float32 var_158;
        wp::float32 var_159;
        const wp::int32 var_160 = 1;
        wp::int32 var_161;
        const wp::float32 var_162 = 0.0;
        wp::float32 var_163;
        bool var_164;
        const wp::float32 var_165 = 1.0;
        wp::float32 var_166;
        wp::float32 var_167;
        wp::range_t var_168;
        wp::int32 var_169;
        wp::float32 var_170;
        wp::float32 var_171;
        wp::int32 var_172;
        wp::int32* var_173;
        wp::int32* var_174;
        wp::int32 var_175;
        wp::int32 var_176;
        wp::int32 var_177;
        wp::int32* var_178;
        const wp::int32 var_179 = 3;
        bool var_180;
        wp::int32 var_181;
        wp::int32* var_182;
        wp::int32 var_183;
        wp::int32 var_184;
        wp::mat_t<3, 3, wp::float32>* var_185;
        wp::mat_t<3, 3, wp::float32> var_186;
        wp::mat_t<3, 3, wp::float32> var_187;
        wp::vec_t<3, wp::float32> var_188;
        const wp::int32 var_189 = 0;
        wp::int32 var_190;
        wp::float32 var_191;
        wp::float32 var_192;
        wp::float32 var_193;
        wp::int32 var_194;
        const wp::int32 var_195 = 1;
        wp::int32 var_196;
        wp::float32 var_197;
        wp::float32 var_198;
        wp::float32 var_199;
        wp::int32 var_200;
        const wp::int32 var_201 = 2;
        wp::int32 var_202;
        wp::float32 var_203;
        wp::float32 var_204;
        wp::float32 var_205;
        wp::int32 var_206;
        wp::int32 var_207;
        wp::int32 var_208;
        wp::vec_t<3, wp::float32> var_209;
        wp::int32 var_210;
        wp::int32 var_211;
        wp::int32 var_212;
        wp::int32 var_213;
        wp::int32 var_214;
        wp::int32 var_215;
        const wp::int32 var_216 = 0;
        bool var_217;
        const wp::int32 var_218 = 0;
        bool var_219;
        wp::int32* var_220;
        wp::int32 var_221;
        wp::int32 var_222;
        wp::int32* var_223;
        wp::int32* var_224;
        wp::int32 var_225;
        wp::int32 var_226;
        wp::int32 var_227;
        wp::int32* var_228;
        const wp::int32 var_229 = 3;
        bool var_230;
        wp::int32 var_231;
        wp::int32* var_232;
        wp::int32 var_233;
        wp::int32 var_234;
        wp::mat_t<3, 3, wp::float32>* var_235;
        wp::mat_t<3, 3, wp::float32> var_236;
        wp::mat_t<3, 3, wp::float32> var_237;
        wp::vec_t<3, wp::float32> var_238;
        const wp::int32 var_239 = 0;
        wp::int32 var_240;
        wp::float32 var_241;
        wp::int32 var_242;
        const wp::int32 var_243 = 1;
        wp::int32 var_244;
        wp::float32 var_245;
        wp::int32 var_246;
        const wp::int32 var_247 = 2;
        wp::int32 var_248;
        wp::float32 var_249;
        wp::int32 var_250;
        wp::int32 var_251;
        wp::int32 var_252;
        wp::vec_t<3, wp::float32> var_253;
        wp::int32 var_254;
        wp::int32 var_255;
        wp::int32 var_256;
        wp::int32 var_257;
        wp::int32 var_258;
        wp::vec_t<3, wp::float32> var_259;
        wp::int32 var_260;
        const wp::int32 var_261 = 0;
        bool var_262;
        wp::int32* var_263;
        wp::int32 var_264;
        wp::int32 var_265;
        const wp::int32 var_266 = 1;
        wp::int32 var_267;
        wp::int32* var_268;
        wp::int32 var_269;
        wp::int32 var_270;
        wp::int32 var_271;
        const wp::float32 var_272 = 0.0;
        const wp::float32 var_273 = 0.0;
        const wp::float32 var_274 = 0.0;
        const wp::float32 var_275 = 0.0;
        wp::vec_t<4, wp::float32> var_276;
        const wp::int32 var_277 = -1;
        const wp::int32 var_278 = -1;
        const wp::int32 var_279 = -1;
        const wp::int32 var_280 = -1;
        wp::vec_t<4, wp::int32> var_281;
        const wp::int32 var_282 = 0;
        wp::int32 var_283;
        const wp::float32 var_284 = 0.0;
        wp::float32 var_285;
        wp::range_t var_286;
        wp::int32 var_287;
        wp::int32 var_288;
        wp::int32* var_289;
        wp::int32 var_290;
        wp::int32 var_291;
        bool var_292;
        const wp::int32 var_293 = 0;
        bool var_294;
        bool var_295;
        wp::int32 var_296;
        wp::int32* var_297;
        wp::int32 var_298;
        wp::int32 var_299;
        wp::vec_t<3, wp::float32>* var_300;
        wp::vec_t<3, wp::float32> var_301;
        wp::vec_t<3, wp::float32> var_302;
        wp::vec_t<3, wp::float32> var_303;
        wp::float32 var_304;
        const wp::float32 var_305 = 1.0;
        wp::float32 var_306;
        wp::float32 var_307;
        const wp::int32 var_308 = 1;
        wp::int32 var_309;
        const wp::float32 var_310 = 0.0;
        wp::float32 var_311;
        bool var_312;
        const wp::float32 var_313 = 1.0;
        wp::float32 var_314;
        wp::float32 var_315;
        wp::range_t var_316;
        wp::int32 var_317;
        wp::float32 var_318;
        wp::float32 var_319;
        wp::int32 var_320;
        wp::int32* var_321;
        wp::int32* var_322;
        wp::int32 var_323;
        wp::int32 var_324;
        wp::int32 var_325;
        wp::int32* var_326;
        const wp::int32 var_327 = 3;
        bool var_328;
        wp::int32 var_329;
        wp::int32* var_330;
        wp::int32 var_331;
        wp::int32 var_332;
        wp::mat_t<3, 3, wp::float32>* var_333;
        wp::mat_t<3, 3, wp::float32> var_334;
        wp::mat_t<3, 3, wp::float32> var_335;
        wp::vec_t<3, wp::float32> var_336;
        const wp::int32 var_337 = 0;
        wp::int32 var_338;
        wp::float32 var_339;
        wp::float32 var_340;
        wp::int32 var_341;
        const wp::int32 var_342 = 1;
        wp::int32 var_343;
        wp::float32 var_344;
        wp::float32 var_345;
        wp::int32 var_346;
        const wp::int32 var_347 = 2;
        wp::int32 var_348;
        wp::float32 var_349;
        wp::float32 var_350;
        wp::int32 var_351;
        wp::int32 var_352;
        wp::int32 var_353;
        wp::vec_t<3, wp::float32> var_354;
        wp::int32 var_355;
        wp::int32 var_356;
        wp::int32 var_357;
        wp::int32 var_358;
        wp::int32 var_359;
        wp::int32 var_360;
        wp::int32 var_361;
        wp::int32 var_362;
        wp::int32 var_363;
        wp::int32 var_364;
        wp::int32 var_365;
        wp::vec_t<3, wp::float32> var_366;
        wp::int32 var_367;
        wp::int32 var_368;
        const wp::int32 var_369 = 0;
        wp::int32 var_370;
        wp::range_t var_371;
        wp::int32 var_372;
        wp::float32* var_373;
        wp::float32 var_374;
        wp::float32 var_375;
        const wp::float32 var_376 = 1e-12;
        bool var_377;
        bool var_378;
        wp::int32* var_379;
        wp::int32 var_380;
        wp::float32* var_381;
        wp::float32 var_382;
        const wp::int32 var_383 = 1;
        wp::int32 var_384;
        wp::int32 var_385;
        const wp::int32 var_386 = 0;
        bool var_387;
        wp::float32 var_388;
        const wp::float32 var_389 = 0.0;
        wp::float32 var_390;
        wp::float32 var_391;
        wp::float32 var_392;
        wp::float32 var_393;
        wp::float32 var_394;
        const wp::float32 var_395 = 0.0;
        wp::float32 var_396;
        const wp::float32 var_397 = 0.0;
        wp::float32 var_398;
        //---------
        // forward
        // def _eff_contact_build(                                                                <L 3117>
        // cid = wp.tid()                                                                         <L 3153>
        var_0 = builtin_tid1d();
        // if cid >= nacon_in[0]:                                                                 <L 3154>
        var_2 = wp::address(var_nacon_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 3155>
            continue;
        }
        // if not bool(contact_type_in[cid] & int(ContactType.PASSIVE)):                          <L 3157>
        var_5 = wp::address(var_contact_type_in, var_0);
        var_8 = wp::int(var_7);
        var_10 = wp::load(var_5);
        var_9 = wp::bit_and(var_10, var_8);
        var_11 = bool(var_9);
        var_12 = wp::unot(var_11);
        if (var_12) {
            // efm_con_nnz_out[cid] = 0                                                           <L 3158>
            wp::array_store(var_efm_con_nnz_out, var_0, var_13);
            // efm_con_force_out[cid] = float(0.0)                                                <L 3159>
            var_15 = wp::float(var_14);
            wp::array_store(var_efm_con_force_out, var_0, var_15);
            // efm_con_scale_out[cid] = float(0.0)                                                <L 3160>
            var_17 = wp::float(var_16);
            wp::array_store(var_efm_con_scale_out, var_0, var_17);
            // return                                                                             <L 3161>
            continue;
        }
        // worldid = contact_worldid_in[cid]                                                      <L 3163>
        var_18 = wp::address(var_contact_worldid_in, var_0);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // timestep = opt_timestep[worldid % opt_timestep.shape[0]]                               <L 3164>
        var_21 = &(var_opt_timestep.shape);
        var_24 = wp::load(var_21);
        var_23 = wp::extract(var_24, var_22);
        var_25 = wp::mod(var_19, var_23);
        var_26 = wp::address(var_opt_timestep, var_25);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // flex0 = contact_flex_in[cid][0]                                                        <L 3165>
        var_29 = wp::address(var_contact_flex_in, var_0);
        var_32 = wp::load(var_29);
        var_31 = wp::extract(var_32, var_30);
        // flex1 = contact_flex_in[cid][1]                                                        <L 3166>
        var_33 = wp::address(var_contact_flex_in, var_0);
        var_36 = wp::load(var_33);
        var_35 = wp::extract(var_36, var_34);
        // elem0 = contact_elem_in[cid][0]                                                        <L 3167>
        var_37 = wp::address(var_contact_elem_in, var_0);
        var_40 = wp::load(var_37);
        var_39 = wp::extract(var_40, var_38);
        // elem1 = contact_elem_in[cid][1]                                                        <L 3168>
        var_41 = wp::address(var_contact_elem_in, var_0);
        var_44 = wp::load(var_41);
        var_43 = wp::extract(var_44, var_42);
        // vert0 = contact_vert_in[cid][0]                                                        <L 3169>
        var_45 = wp::address(var_contact_vert_in, var_0);
        var_48 = wp::load(var_45);
        var_47 = wp::extract(var_48, var_46);
        // vert1 = contact_vert_in[cid][1]                                                        <L 3170>
        var_49 = wp::address(var_contact_vert_in, var_0);
        var_52 = wp::load(var_49);
        var_51 = wp::extract(var_52, var_50);
        // k = _flex_contact_stiffness(                                                           <L 3172>
        // body_weldid,                                                                           <L 3173>
        // body_dofnum,                                                                           <L 3174>
        // body_dofadr,                                                                           <L 3175>
        // flex_dim,                                                                              <L 3176>
        // flex_vertadr,                                                                          <L 3177>
        // flex_elemdataadr,                                                                      <L 3178>
        // flex_vertbodyid,                                                                       <L 3179>
        // flex_elem,                                                                             <L 3180>
        // M_rownnz,                                                                              <L 3181>
        // M_rowadr,                                                                              <L 3182>
        // M_in,                                                                                  <L 3183>
        // worldid,                                                                               <L 3184>
        // flex0,                                                                                 <L 3185>
        // flex1,                                                                                 <L 3186>
        // elem0,                                                                                 <L 3187>
        // elem1,                                                                                 <L 3188>
        // vert0,                                                                                 <L 3189>
        // vert1,                                                                                 <L 3190>
        var_53 = _flex_contact_stiffness_0(var_body_weldid, var_body_dofnum, var_body_dofadr, var_flex_dim, var_flex_vertadr, var_flex_elemdataadr, var_flex_vertbodyid, var_flex_elem, var_M_rownnz, var_M_rowadr, var_M_in, var_19, var_31, var_35, var_39, var_43, var_47, var_51);
        // if k <= float(0.0):                                                                    <L 3193>
        var_55 = wp::float(var_54);
        var_56 = (var_53 <= var_55);
        if (var_56) {
            // efm_con_nnz_out[cid] = 0                                                           <L 3194>
            wp::array_store(var_efm_con_nnz_out, var_0, var_57);
            // efm_con_force_out[cid] = float(0.0)                                                <L 3195>
            var_59 = wp::float(var_58);
            wp::array_store(var_efm_con_force_out, var_0, var_59);
            // efm_con_scale_out[cid] = float(0.0)                                                <L 3196>
            var_61 = wp::float(var_60);
            wp::array_store(var_efm_con_scale_out, var_0, var_61);
            // return                                                                             <L 3197>
            continue;
        }
        // gap = contact_dist_in[cid]                                                             <L 3199>
        var_62 = wp::address(var_contact_dist_in, var_0);
        var_64 = wp::load(var_62);
        var_63 = wp::copy(var_64);
        // pos = contact_pos_in[cid]                                                              <L 3200>
        var_65 = wp::address(var_contact_pos_in, var_0);
        var_67 = wp::load(var_65);
        var_66 = wp::copy(var_67);
        // normal = contact_frame_in[cid][0]                                                      <L 3201>
        var_68 = wp::address(var_contact_frame_in, var_0);
        var_71 = wp::load(var_68);
        var_70 = wp::extract(var_71, var_69);
        // nnz = int(0)                                                                           <L 3203>
        var_73 = wp::int(var_72);
        // if flex0 >= 0:                                                                         <L 3206>
        var_75 = (var_31 >= var_74);
        if (var_75) {
            // if vert0 >= 0:                                                                     <L 3207>
            var_77 = (var_47 >= var_76);
            if (var_77) {
                // gv = flex_vertadr[flex0] + vert0                                               <L 3208>
                var_78 = wp::address(var_flex_vertadr, var_31);
                var_80 = wp::load(var_78);
                var_79 = wp::add(var_80, var_47);
                // b = body_weldid[flex_vertbodyid[gv]]                                           <L 3209>
                var_81 = wp::address(var_flex_vertbodyid, var_79);
                var_83 = wp::load(var_81);
                var_82 = wp::address(var_body_weldid, var_83);
                var_85 = wp::load(var_82);
                var_84 = wp::copy(var_85);
                // if body_dofnum[b] == 3:                                                        <L 3210>
                var_86 = wp::address(var_body_dofnum, var_84);
                var_89 = wp::load(var_86);
                var_88 = (var_89 == var_87);
                if (var_88) {
                    // da = body_dofadr[b]                                                        <L 3211>
                    var_90 = wp::address(var_body_dofadr, var_84);
                    var_92 = wp::load(var_90);
                    var_91 = wp::copy(var_92);
                    // n_loc = wp.transpose(xmat_in[worldid, b]) @ normal                         <L 3212>
                    var_93 = wp::address(var_xmat_in, var_19, var_84);
                    var_95 = wp::load(var_93);
                    var_94 = wp::transpose(var_95);
                    var_96 = wp::mul(var_94, var_70);
                    // for ax in range(3):                                                        <L 3213>
                    // nnz = _add_con_dof(cid, da + ax, -n_loc[ax], nnz, efm_con_dof_out, efm_con_val_out)       <L 3214>
                    var_98 = wp::add(var_91, var_97);
                    var_99 = wp::extract(var_96, var_97);
                    var_100 = wp::neg(var_99);
                    var_101 = _add_con_dof_0(var_0, var_98, var_100, var_73, var_efm_con_dof_out, var_efm_con_val_out);
                    var_103 = wp::add(var_91, var_102);
                    var_104 = wp::extract(var_96, var_102);
                    var_105 = wp::neg(var_104);
                    var_106 = _add_con_dof_0(var_0, var_103, var_105, var_101, var_efm_con_dof_out, var_efm_con_val_out);
                    var_108 = wp::add(var_91, var_107);
                    var_109 = wp::extract(var_96, var_107);
                    var_110 = wp::neg(var_109);
                    var_111 = _add_con_dof_0(var_0, var_108, var_110, var_106, var_efm_con_dof_out, var_efm_con_val_out);
                }
                var_112 = wp::where(var_88, var_111, var_73);
            }
            var_113 = wp::where(var_77, var_112, var_73);
            if (!var_77) {
                // elif elem0 >= 0:                                                               <L 3215>
                var_115 = (var_39 >= var_114);
                if (var_115) {
                    // dim0 = flex_dim[flex0]                                                     <L 3216>
                    var_116 = wp::address(var_flex_dim, var_31);
                    var_118 = wp::load(var_116);
                    var_117 = wp::copy(var_118);
                    // nvrt0 = dim0 + 1                                                           <L 3217>
                    var_120 = wp::add(var_117, var_119);
                    // base0 = flex_elemdataadr[flex0] + nvrt0 * elem0                            <L 3218>
                    var_121 = wp::address(var_flex_elemdataadr, var_31);
                    var_122 = wp::mul(var_120, var_39);
                    var_124 = wp::load(var_121);
                    var_123 = wp::add(var_124, var_122);
                    // w0 = wp.vec4(0.0, 0.0, 0.0, 0.0)                                           <L 3219>
                    var_129 = wp::vec_t<4, wp::float32>(var_125, var_126, var_127, var_128);
                    // v0 = wp.vec4i(-1, -1, -1, -1)                                              <L 3220>
                    var_134 = wp::vec_t<4, wp::int32>(var_130, var_131, var_132, var_133);
                    // n0 = int(0)                                                                <L 3221>
                    var_136 = wp::int(var_135);
                    // sum_w0 = float(0.0)                                                        <L 3222>
                    var_138 = wp::float(var_137);
                    // for j in range(nvrt0):                                                     <L 3223>
                    var_139 = wp::range(var_120);
                    start_for_3:;
                        if (iter_cmp(var_139) == 0) goto end_for_3;
                        var_140 = wp::iter_next(var_139);
                        // v_local = flex_elem[base0 + j]                                         <L 3224>
                        var_141 = wp::add(var_123, var_140);
                        var_142 = wp::address(var_flex_elem, var_141);
                        var_144 = wp::load(var_142);
                        var_143 = wp::copy(var_144);
                        // if vert1 >= 0 and v_local == vert1:                                    <L 3225>
                        var_147 = (var_51 >= var_146);
                        var_145 = var_147;
                        if (var_145) {
                            var_148 = (var_143 == var_51);
                            var_145 = var_145 && var_148;
                        }
                        if (var_145) {
                            // continue                                                           <L 3226>
                            goto start_for_3;
                        }
                        // gv = flex_vertadr[flex0] + v_local                                     <L 3227>
                        var_149 = wp::address(var_flex_vertadr, var_31);
                        var_151 = wp::load(var_149);
                        var_150 = wp::add(var_151, var_143);
                        // v_pos = flexvert_xpos_in[worldid, gv]                                  <L 3228>
                        var_152 = wp::address(var_flexvert_xpos_in, var_19, var_150);
                        var_154 = wp::load(var_152);
                        var_153 = wp::copy(var_154);
                        // d = wp.length(pos - v_pos)                                             <L 3229>
                        var_155 = wp::sub(var_66, var_153);
                        var_156 = wp::length(var_155);
                        // weight = math.safe_div(1.0, d)                                         <L 3230>
                        var_158 = safe_div_0(var_157, var_156);
                        // w0[n0] = weight                                                        <L 3231>
                        wp::assign_inplace(var_129, var_136, var_158);
                        // v0[n0] = gv                                                            <L 3232>
                        wp::assign_inplace(var_134, var_136, var_150);
                        // sum_w0 += weight                                                       <L 3233>
                        var_159 = wp::add(var_138, var_158);
                        // n0 += 1                                                                <L 3234>
                        var_161 = wp::add(var_136, var_160);
                        wp::assign(var_79, var_150);
                        wp::assign(var_136, var_161);
                        wp::assign(var_138, var_159);
                        goto start_for_3;
                    end_for_3:;
                    // if sum_w0 > float(0.0):                                                    <L 3235>
                    var_163 = wp::float(var_162);
                    var_164 = (var_138 > var_163);
                    if (var_164) {
                        // inv_sum0 = float(1.0) / sum_w0                                         <L 3236>
                        var_166 = wp::float(var_165);
                        var_167 = wp::div(var_166, var_138);
                        // for j in range(n0):                                                    <L 3237>
                        var_168 = wp::range(var_136);
                        start_for_5:;
                            if (iter_cmp(var_168) == 0) goto end_for_5;
                            var_169 = wp::iter_next(var_168);
                            // wj = w0[j] * inv_sum0                                              <L 3238>
                            var_170 = wp::extract(var_129, var_169);
                            var_171 = wp::mul(var_170, var_167);
                            // gv = v0[j]                                                         <L 3239>
                            var_172 = wp::extract(var_134, var_169);
                            // b = body_weldid[flex_vertbodyid[gv]]                               <L 3240>
                            var_173 = wp::address(var_flex_vertbodyid, var_172);
                            var_175 = wp::load(var_173);
                            var_174 = wp::address(var_body_weldid, var_175);
                            var_177 = wp::load(var_174);
                            var_176 = wp::copy(var_177);
                            // if body_dofnum[b] == 3:                                            <L 3241>
                            var_178 = wp::address(var_body_dofnum, var_176);
                            var_181 = wp::load(var_178);
                            var_180 = (var_181 == var_179);
                            if (var_180) {
                                // da = body_dofadr[b]                                            <L 3242>
                                var_182 = wp::address(var_body_dofadr, var_176);
                                var_184 = wp::load(var_182);
                                var_183 = wp::copy(var_184);
                                // n_loc = wp.transpose(xmat_in[worldid, b]) @ normal             <L 3243>
                                var_185 = wp::address(var_xmat_in, var_19, var_176);
                                var_187 = wp::load(var_185);
                                var_186 = wp::transpose(var_187);
                                var_188 = wp::mul(var_186, var_70);
                                // for ax in range(3):                                            <L 3244>
                                // nnz = _add_con_dof(cid, da + ax, -wj * n_loc[ax], nnz, efm_con_dof_out, efm_con_val_out)       <L 3245>
                                var_190 = wp::add(var_183, var_189);
                                var_191 = wp::neg(var_171);
                                var_192 = wp::extract(var_188, var_189);
                                var_193 = wp::mul(var_191, var_192);
                                var_194 = _add_con_dof_0(var_0, var_190, var_193, var_113, var_efm_con_dof_out, var_efm_con_val_out);
                                var_196 = wp::add(var_183, var_195);
                                var_197 = wp::neg(var_171);
                                var_198 = wp::extract(var_188, var_195);
                                var_199 = wp::mul(var_197, var_198);
                                var_200 = _add_con_dof_0(var_0, var_196, var_199, var_194, var_efm_con_dof_out, var_efm_con_val_out);
                                var_202 = wp::add(var_183, var_201);
                                var_203 = wp::neg(var_171);
                                var_204 = wp::extract(var_188, var_201);
                                var_205 = wp::mul(var_203, var_204);
                                var_206 = _add_con_dof_0(var_0, var_202, var_205, var_200, var_efm_con_dof_out, var_efm_con_val_out);
                            }
                            var_207 = wp::where(var_180, var_206, var_113);
                            var_208 = wp::where(var_180, var_183, var_91);
                            var_209 = wp::where(var_180, var_188, var_96);
                            var_210 = wp::where(var_180, var_201, var_107);
                            wp::assign(var_113, var_207);
                            wp::assign(var_79, var_172);
                            wp::assign(var_84, var_176);
                            wp::assign(var_91, var_208);
                            wp::assign(var_96, var_209);
                            goto start_for_5;
                        end_for_5:;
                    }
                    var_211 = wp::where(var_164, var_210, var_107);
                    var_212 = wp::where(var_164, var_169, var_140);
                }
                var_213 = wp::where(var_115, var_211, var_107);
            }
            var_214 = wp::where(var_77, var_107, var_213);
        }
        var_215 = wp::where(var_75, var_113, var_73);
        // if flex1 >= 0:                                                                         <L 3248>
        var_217 = (var_35 >= var_216);
        if (var_217) {
            // if vert1 >= 0:                                                                     <L 3249>
            var_219 = (var_51 >= var_218);
            if (var_219) {
                // gv = flex_vertadr[flex1] + vert1                                               <L 3250>
                var_220 = wp::address(var_flex_vertadr, var_35);
                var_222 = wp::load(var_220);
                var_221 = wp::add(var_222, var_51);
                // b = body_weldid[flex_vertbodyid[gv]]                                           <L 3251>
                var_223 = wp::address(var_flex_vertbodyid, var_221);
                var_225 = wp::load(var_223);
                var_224 = wp::address(var_body_weldid, var_225);
                var_227 = wp::load(var_224);
                var_226 = wp::copy(var_227);
                // if body_dofnum[b] == 3:                                                        <L 3252>
                var_228 = wp::address(var_body_dofnum, var_226);
                var_231 = wp::load(var_228);
                var_230 = (var_231 == var_229);
                if (var_230) {
                    // da = body_dofadr[b]                                                        <L 3253>
                    var_232 = wp::address(var_body_dofadr, var_226);
                    var_234 = wp::load(var_232);
                    var_233 = wp::copy(var_234);
                    // n_loc = wp.transpose(xmat_in[worldid, b]) @ normal                         <L 3254>
                    var_235 = wp::address(var_xmat_in, var_19, var_226);
                    var_237 = wp::load(var_235);
                    var_236 = wp::transpose(var_237);
                    var_238 = wp::mul(var_236, var_70);
                    // for ax in range(3):                                                        <L 3255>
                    // nnz = _add_con_dof(cid, da + ax, n_loc[ax], nnz, efm_con_dof_out, efm_con_val_out)       <L 3256>
                    var_240 = wp::add(var_233, var_239);
                    var_241 = wp::extract(var_238, var_239);
                    var_242 = _add_con_dof_0(var_0, var_240, var_241, var_215, var_efm_con_dof_out, var_efm_con_val_out);
                    var_244 = wp::add(var_233, var_243);
                    var_245 = wp::extract(var_238, var_243);
                    var_246 = _add_con_dof_0(var_0, var_244, var_245, var_242, var_efm_con_dof_out, var_efm_con_val_out);
                    var_248 = wp::add(var_233, var_247);
                    var_249 = wp::extract(var_238, var_247);
                    var_250 = _add_con_dof_0(var_0, var_248, var_249, var_246, var_efm_con_dof_out, var_efm_con_val_out);
                }
                var_251 = wp::where(var_230, var_250, var_215);
                var_252 = wp::where(var_230, var_233, var_91);
                var_253 = wp::where(var_230, var_238, var_96);
                var_254 = wp::where(var_230, var_247, var_214);
            }
            var_255 = wp::where(var_219, var_251, var_215);
            var_256 = wp::where(var_219, var_221, var_79);
            var_257 = wp::where(var_219, var_226, var_84);
            var_258 = wp::where(var_219, var_252, var_91);
            var_259 = wp::where(var_219, var_253, var_96);
            var_260 = wp::where(var_219, var_254, var_214);
            if (!var_219) {
                // elif elem1 >= 0:                                                               <L 3257>
                var_262 = (var_43 >= var_261);
                if (var_262) {
                    // dim1 = flex_dim[flex1]                                                     <L 3258>
                    var_263 = wp::address(var_flex_dim, var_35);
                    var_265 = wp::load(var_263);
                    var_264 = wp::copy(var_265);
                    // nvrt1 = dim1 + 1                                                           <L 3259>
                    var_267 = wp::add(var_264, var_266);
                    // base1 = flex_elemdataadr[flex1] + nvrt1 * elem1                            <L 3260>
                    var_268 = wp::address(var_flex_elemdataadr, var_35);
                    var_269 = wp::mul(var_267, var_43);
                    var_271 = wp::load(var_268);
                    var_270 = wp::add(var_271, var_269);
                    // w1 = wp.vec4(0.0, 0.0, 0.0, 0.0)                                           <L 3261>
                    var_276 = wp::vec_t<4, wp::float32>(var_272, var_273, var_274, var_275);
                    // v1 = wp.vec4i(-1, -1, -1, -1)                                              <L 3262>
                    var_281 = wp::vec_t<4, wp::int32>(var_277, var_278, var_279, var_280);
                    // n1 = int(0)                                                                <L 3263>
                    var_283 = wp::int(var_282);
                    // sum_w1 = float(0.0)                                                        <L 3264>
                    var_285 = wp::float(var_284);
                    // for j in range(nvrt1):                                                     <L 3265>
                    var_286 = wp::range(var_267);
                    start_for_7:;
                        if (iter_cmp(var_286) == 0) goto end_for_7;
                        var_287 = wp::iter_next(var_286);
                        // v_local = flex_elem[base1 + j]                                         <L 3266>
                        var_288 = wp::add(var_270, var_287);
                        var_289 = wp::address(var_flex_elem, var_288);
                        var_291 = wp::load(var_289);
                        var_290 = wp::copy(var_291);
                        // if vert0 >= 0 and v_local == vert0:                                    <L 3267>
                        var_294 = (var_47 >= var_293);
                        var_292 = var_294;
                        if (var_292) {
                            var_295 = (var_290 == var_47);
                            var_292 = var_292 && var_295;
                        }
                        if (var_292) {
                            // continue                                                           <L 3268>
                            wp::assign(var_143, var_290);
                            goto start_for_7;
                        }
                        var_296 = wp::where(var_292, var_143, var_290);
                        // gv = flex_vertadr[flex1] + v_local                                     <L 3269>
                        var_297 = wp::address(var_flex_vertadr, var_35);
                        var_299 = wp::load(var_297);
                        var_298 = wp::add(var_299, var_296);
                        // v_pos = flexvert_xpos_in[worldid, gv]                                  <L 3270>
                        var_300 = wp::address(var_flexvert_xpos_in, var_19, var_298);
                        var_302 = wp::load(var_300);
                        var_301 = wp::copy(var_302);
                        // d = wp.length(pos - v_pos)                                             <L 3271>
                        var_303 = wp::sub(var_66, var_301);
                        var_304 = wp::length(var_303);
                        // weight = math.safe_div(1.0, d)                                         <L 3272>
                        var_306 = safe_div_0(var_305, var_304);
                        // w1[n1] = weight                                                        <L 3273>
                        wp::assign_inplace(var_276, var_283, var_306);
                        // v1[n1] = gv                                                            <L 3274>
                        wp::assign_inplace(var_281, var_283, var_298);
                        // sum_w1 += weight                                                       <L 3275>
                        var_307 = wp::add(var_285, var_306);
                        // n1 += 1                                                                <L 3276>
                        var_309 = wp::add(var_283, var_308);
                        wp::assign(var_256, var_298);
                        wp::assign(var_143, var_296);
                        wp::assign(var_153, var_301);
                        wp::assign(var_156, var_304);
                        wp::assign(var_158, var_306);
                        wp::assign(var_283, var_309);
                        wp::assign(var_285, var_307);
                        goto start_for_7;
                    end_for_7:;
                    // if sum_w1 > float(0.0):                                                    <L 3277>
                    var_311 = wp::float(var_310);
                    var_312 = (var_285 > var_311);
                    if (var_312) {
                        // inv_sum1 = float(1.0) / sum_w1                                         <L 3278>
                        var_314 = wp::float(var_313);
                        var_315 = wp::div(var_314, var_285);
                        // for j in range(n1):                                                    <L 3279>
                        var_316 = wp::range(var_283);
                        start_for_9:;
                            if (iter_cmp(var_316) == 0) goto end_for_9;
                            var_317 = wp::iter_next(var_316);
                            // wj = w1[j] * inv_sum1                                              <L 3280>
                            var_318 = wp::extract(var_276, var_317);
                            var_319 = wp::mul(var_318, var_315);
                            // gv = v1[j]                                                         <L 3281>
                            var_320 = wp::extract(var_281, var_317);
                            // b = body_weldid[flex_vertbodyid[gv]]                               <L 3282>
                            var_321 = wp::address(var_flex_vertbodyid, var_320);
                            var_323 = wp::load(var_321);
                            var_322 = wp::address(var_body_weldid, var_323);
                            var_325 = wp::load(var_322);
                            var_324 = wp::copy(var_325);
                            // if body_dofnum[b] == 3:                                            <L 3283>
                            var_326 = wp::address(var_body_dofnum, var_324);
                            var_329 = wp::load(var_326);
                            var_328 = (var_329 == var_327);
                            if (var_328) {
                                // da = body_dofadr[b]                                            <L 3284>
                                var_330 = wp::address(var_body_dofadr, var_324);
                                var_332 = wp::load(var_330);
                                var_331 = wp::copy(var_332);
                                // n_loc = wp.transpose(xmat_in[worldid, b]) @ normal             <L 3285>
                                var_333 = wp::address(var_xmat_in, var_19, var_324);
                                var_335 = wp::load(var_333);
                                var_334 = wp::transpose(var_335);
                                var_336 = wp::mul(var_334, var_70);
                                // for ax in range(3):                                            <L 3286>
                                // nnz = _add_con_dof(cid, da + ax, wj * n_loc[ax], nnz, efm_con_dof_out, efm_con_val_out)       <L 3287>
                                var_338 = wp::add(var_331, var_337);
                                var_339 = wp::extract(var_336, var_337);
                                var_340 = wp::mul(var_319, var_339);
                                var_341 = _add_con_dof_0(var_0, var_338, var_340, var_255, var_efm_con_dof_out, var_efm_con_val_out);
                                var_343 = wp::add(var_331, var_342);
                                var_344 = wp::extract(var_336, var_342);
                                var_345 = wp::mul(var_319, var_344);
                                var_346 = _add_con_dof_0(var_0, var_343, var_345, var_341, var_efm_con_dof_out, var_efm_con_val_out);
                                var_348 = wp::add(var_331, var_347);
                                var_349 = wp::extract(var_336, var_347);
                                var_350 = wp::mul(var_319, var_349);
                                var_351 = _add_con_dof_0(var_0, var_348, var_350, var_346, var_efm_con_dof_out, var_efm_con_val_out);
                            }
                            var_352 = wp::where(var_328, var_351, var_255);
                            var_353 = wp::where(var_328, var_331, var_258);
                            var_354 = wp::where(var_328, var_336, var_259);
                            var_355 = wp::where(var_328, var_347, var_260);
                            wp::assign(var_255, var_352);
                            wp::assign(var_256, var_320);
                            wp::assign(var_257, var_324);
                            wp::assign(var_258, var_353);
                            wp::assign(var_259, var_354);
                            wp::assign(var_171, var_319);
                            goto start_for_9;
                        end_for_9:;
                    }
                    var_356 = wp::where(var_312, var_355, var_260);
                    var_357 = wp::where(var_312, var_317, var_287);
                }
                var_358 = wp::where(var_262, var_356, var_260);
                var_359 = wp::where(var_262, var_357, var_212);
            }
            var_360 = wp::where(var_219, var_260, var_358);
            var_361 = wp::where(var_219, var_212, var_359);
        }
        var_362 = wp::where(var_217, var_255, var_215);
        var_363 = wp::where(var_217, var_256, var_79);
        var_364 = wp::where(var_217, var_257, var_84);
        var_365 = wp::where(var_217, var_258, var_91);
        var_366 = wp::where(var_217, var_259, var_96);
        var_367 = wp::where(var_217, var_360, var_214);
        var_368 = wp::where(var_217, var_361, var_212);
        // real_nnz = int(0)                                                                      <L 3290>
        var_370 = wp::int(var_369);
        // for idx in range(nnz):                                                                 <L 3291>
        var_371 = wp::range(var_362);
        start_for_11:;
            if (iter_cmp(var_371) == 0) goto end_for_11;
            var_372 = wp::iter_next(var_371);
            // if wp.abs(efm_con_val_out[cid, idx]) > 1e-12:                                      <L 3292>
            var_373 = wp::address(var_efm_con_val_out, var_0, var_372);
            var_375 = wp::load(var_373);
            var_374 = wp::abs(var_375);
            var_377 = (var_374 > var_376);
            if (var_377) {
                // if real_nnz != idx:                                                            <L 3293>
                var_378 = (var_370 != var_372);
                if (var_378) {
                    // efm_con_dof_out[cid, real_nnz] = efm_con_dof_out[cid, idx]                 <L 3294>
                    var_379 = wp::address(var_efm_con_dof_out, var_0, var_372);
                    var_380 = wp::load(var_379);
                    wp::array_store(var_efm_con_dof_out, var_0, var_370, var_380);
                    // efm_con_val_out[cid, real_nnz] = efm_con_val_out[cid, idx]                 <L 3295>
                    var_381 = wp::address(var_efm_con_val_out, var_0, var_372);
                    var_382 = wp::load(var_381);
                    wp::array_store(var_efm_con_val_out, var_0, var_370, var_382);
                }
                // real_nnz += 1                                                                  <L 3296>
                var_384 = wp::add(var_370, var_383);
            }
            var_385 = wp::where(var_377, var_384, var_370);
            wp::assign(var_370, var_385);
            goto start_for_11;
        end_for_11:;
        // efm_con_nnz_out[cid] = real_nnz                                                        <L 3298>
        wp::array_store(var_efm_con_nnz_out, var_0, var_370);
        // if real_nnz > 0:                                                                       <L 3299>
        var_387 = (var_370 > var_386);
        if (var_387) {
            // efm_con_force_out[cid] = -k * wp.min(gap, float(0.0))                              <L 3300>
            var_388 = wp::neg(var_53);
            var_390 = wp::float(var_389);
            var_391 = wp::min(var_63, var_390);
            var_392 = wp::mul(var_388, var_391);
            wp::array_store(var_efm_con_force_out, var_0, var_392);
            // efm_con_scale_out[cid] = timestep * timestep * k                                   <L 3301>
            var_393 = wp::mul(var_27, var_27);
            var_394 = wp::mul(var_393, var_53);
            wp::array_store(var_efm_con_scale_out, var_0, var_394);
        }
        if (!var_387) {
            // efm_con_force_out[cid] = float(0.0)                                                <L 3303>
            var_396 = wp::float(var_395);
            wp::array_store(var_efm_con_force_out, var_0, var_396);
            // efm_con_scale_out[cid] = float(0.0)                                                <L 3304>
            var_398 = wp::float(var_397);
            wp::array_store(var_efm_con_scale_out, var_0, var_398);
        }
    }
}



extern "C" __global__ void _spring_damper_flexedge_passive_ade4181f_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flex_edgestiffness,
    wp::array_t<wp::float32> var_flex_edgedamping,
    wp::array_t<bool> var_flexedge_rigid,
    wp::array_t<wp::int32> var_flexedge_J_rownnz,
    wp::array_t<wp::int32> var_flexedge_J_rowadr,
    wp::array_t<wp::int32> var_flexedge_J_colind,
    wp::array_t<wp::int32> var_flex_edgeflexid,
    wp::array_t<wp::float32> var_flexedge_J_in,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::array_t<wp::float32> var_flexedge_velocity_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::float32> var_qfrc_spring_out,
    wp::array_t<wp::float32> var_qfrc_damper_out)
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
        bool* var_2;
        bool var_3;
        bool var_4;
        wp::int32* var_5;
        wp::int32 var_6;
        wp::int32 var_7;
        const wp::float32 var_8 = 0.0;
        wp::float32 var_9;
        bool var_10;
        wp::float32* var_11;
        wp::float32 var_12;
        wp::float32 var_13;
        wp::float32 var_14;
        const wp::float32 var_15 = 0.0;
        wp::float32 var_16;
        bool var_17;
        wp::float32* var_18;
        wp::float32 var_19;
        wp::float32 var_20;
        wp::float32 var_21;
        bool var_22;
        const wp::float32 var_23 = 0.0;
        bool var_24;
        const wp::float32 var_25 = 0.0;
        bool var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        const wp::int32 var_30 = 0;
        bool var_31;
        const wp::float32 var_32 = 0.0;
        wp::float32 var_33;
        const wp::float32 var_34 = 0.0;
        bool var_35;
        wp::float32* var_36;
        wp::float32* var_37;
        wp::float32 var_38;
        wp::float32 var_39;
        wp::float32 var_40;
        wp::float32 var_41;
        wp::float32 var_42;
        const wp::float32 var_43 = 0.0;
        wp::float32 var_44;
        const wp::float32 var_45 = 0.0;
        bool var_46;
        wp::float32 var_47;
        wp::float32* var_48;
        wp::float32 var_49;
        wp::float32 var_50;
        wp::float32 var_51;
        bool var_52;
        const wp::float32 var_53 = 0.0;
        bool var_54;
        const wp::float32 var_55 = 0.0;
        bool var_56;
        wp::int32* var_57;
        wp::int32 var_58;
        wp::int32 var_59;
        wp::range_t var_60;
        wp::int32 var_61;
        wp::int32 var_62;
        wp::int32* var_63;
        wp::int32 var_64;
        wp::int32 var_65;
        wp::float32* var_66;
        wp::float32 var_67;
        wp::float32 var_68;
        const wp::float32 var_69 = 0.0;
        bool var_70;
        wp::slice_t var_71;
        const wp::int32 var_72 = 0;
        wp::array_t<wp::float32> var_73;
        wp::float32 var_74;
        wp::float32 var_75;
        const wp::float32 var_76 = 0.0;
        bool var_77;
        wp::slice_t var_78;
        const wp::int32 var_79 = 0;
        wp::array_t<wp::float32> var_80;
        wp::float32 var_81;
        wp::float32 var_82;
        //---------
        // forward
        // def _spring_damper_flexedge_passive(                                                   <L 261>
        // worldid, edgeid = wp.tid()                                                             <L 282>
        builtin_tid2d(var_0, var_1);
        // if flexedge_rigid[edgeid]:                                                             <L 284>
        var_2 = wp::address(var_flexedge_rigid, var_1);
        var_3 = wp::load(var_2);
        if (var_3) {
            // return                                                                             <L 285>
            continue;
        }
        var_4 = wp::load(var_2);
        // f = flex_edgeflexid[edgeid]                                                            <L 287>
        var_5 = wp::address(var_flex_edgeflexid, var_1);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // stiffness = float(0.0)                                                                 <L 289>
        var_9 = wp::float(var_8);
        // if not dsbl_spring:                                                                    <L 290>
        var_10 = wp::unot(var_dsbl_spring);
        if (var_10) {
            // stiffness = flex_edgestiffness[f]                                                  <L 291>
            var_11 = wp::address(var_flex_edgestiffness, var_6);
            var_13 = wp::load(var_11);
            var_12 = wp::copy(var_13);
        }
        var_14 = wp::where(var_10, var_12, var_9);
        // damping = float(0.0)                                                                   <L 293>
        var_16 = wp::float(var_15);
        // if not dsbl_damper:                                                                    <L 294>
        var_17 = wp::unot(var_dsbl_damper);
        if (var_17) {
            // damping = flex_edgedamping[f]                                                      <L 295>
            var_18 = wp::address(var_flex_edgedamping, var_6);
            var_20 = wp::load(var_18);
            var_19 = wp::copy(var_20);
        }
        var_21 = wp::where(var_17, var_19, var_16);
        // if stiffness == 0.0 and damping == 0.0:                                                <L 297>
        var_24 = (var_14 == var_23);
        var_22 = var_24;
        if (var_22) {
            var_26 = (var_21 == var_25);
            var_22 = var_22 && var_26;
        }
        if (var_22) {
            // return                                                                             <L 298>
            continue;
        }
        // rownnz = flexedge_J_rownnz[edgeid]                                                     <L 300>
        var_27 = wp::address(var_flexedge_J_rownnz, var_1);
        var_29 = wp::load(var_27);
        var_28 = wp::copy(var_29);
        // if rownnz == 0:                                                                        <L 301>
        var_31 = (var_28 == var_30);
        if (var_31) {
            // return                                                                             <L 302>
            continue;
        }
        // frc_spring = float(0.0)                                                                <L 304>
        var_33 = wp::float(var_32);
        // if stiffness != 0.0:                                                                   <L 305>
        var_35 = (var_14 != var_34);
        if (var_35) {
            // frc_spring = stiffness * (flexedge_length0[edgeid] - flexedge_length_in[worldid, edgeid])       <L 306>
            var_36 = wp::address(var_flexedge_length0, var_1);
            var_37 = wp::address(var_flexedge_length_in, var_0, var_1);
            var_39 = wp::load(var_36);
            var_40 = wp::load(var_37);
            var_38 = wp::sub(var_39, var_40);
            var_41 = wp::mul(var_14, var_38);
        }
        var_42 = wp::where(var_35, var_41, var_33);
        // frc_damper = float(0.0)                                                                <L 308>
        var_44 = wp::float(var_43);
        // if damping != 0.0:                                                                     <L 309>
        var_46 = (var_21 != var_45);
        if (var_46) {
            // frc_damper = -damping * flexedge_velocity_in[worldid, edgeid]                      <L 310>
            var_47 = wp::neg(var_21);
            var_48 = wp::address(var_flexedge_velocity_in, var_0, var_1);
            var_50 = wp::load(var_48);
            var_49 = wp::mul(var_47, var_50);
        }
        var_51 = wp::where(var_46, var_49, var_44);
        // if frc_spring == 0.0 and frc_damper == 0.0:                                            <L 312>
        var_54 = (var_42 == var_53);
        var_52 = var_54;
        if (var_52) {
            var_56 = (var_51 == var_55);
            var_52 = var_52 && var_56;
        }
        if (var_52) {
            // return                                                                             <L 313>
            continue;
        }
        // rowadr = flexedge_J_rowadr[edgeid]                                                     <L 315>
        var_57 = wp::address(var_flexedge_J_rowadr, var_1);
        var_59 = wp::load(var_57);
        var_58 = wp::copy(var_59);
        // for k in range(rownnz):                                                                <L 316>
        var_60 = wp::range(var_28);
        start_for_4:;
            if (iter_cmp(var_60) == 0) goto end_for_4;
            var_61 = wp::iter_next(var_60);
            // sparseid = rowadr + k                                                              <L 317>
            var_62 = wp::add(var_58, var_61);
            // colind = flexedge_J_colind[sparseid]                                               <L 318>
            var_63 = wp::address(var_flexedge_J_colind, var_62);
            var_65 = wp::load(var_63);
            var_64 = wp::copy(var_65);
            // J = flexedge_J_in[worldid, sparseid]                                               <L 319>
            var_66 = wp::address(var_flexedge_J_in, var_0, var_62);
            var_68 = wp::load(var_66);
            var_67 = wp::copy(var_68);
            // if frc_spring != 0.0:                                                              <L 320>
            var_70 = (var_42 != var_69);
            if (var_70) {
                // wp.atomic_add(qfrc_spring_out[worldid], colind, J * frc_spring)                <L 321>
                var_71 = wp::slice_t(var_0, var_0, var_72);
                var_73 = wp::view(var_qfrc_spring_out, var_71);
                var_74 = wp::mul(var_67, var_42);
                var_75 = wp::atomic_add(var_73, var_64, var_74);
            }
            // if frc_damper != 0.0:                                                              <L 322>
            var_77 = (var_51 != var_76);
            if (var_77) {
                // wp.atomic_add(qfrc_damper_out[worldid], colind, J * frc_damper)                <L 323>
                var_78 = wp::slice_t(var_0, var_0, var_79);
                var_80 = wp::view(var_qfrc_damper_out, var_78);
                var_81 = wp::mul(var_67, var_51);
                var_82 = wp::atomic_add(var_80, var_64, var_81);
            }
            goto start_for_4;
        end_for_4:;
    }
}



extern "C" __global__ void _eff_contact_force_3892154f_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_contact_worldid_in,
    wp::array_t<wp::int32> var_efm_con_dof_in,
    wp::array_t<wp::float32> var_efm_con_val_in,
    wp::array_t<wp::float32> var_efm_con_force_in,
    wp::array_t<wp::int32> var_efm_con_nnz_in,
    wp::array_t<wp::float32> var_qfrc_spring_out)
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
        wp::int32* var_1;
        wp::int32 var_2;
        wp::int32 var_3;
        const wp::int32 var_4 = 0;
        bool var_5;
        wp::float32* var_6;
        wp::float32 var_7;
        wp::float32 var_8;
        const wp::float32 var_9 = 0.0;
        wp::float32 var_10;
        bool var_11;
        wp::int32* var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        wp::range_t var_15;
        wp::int32 var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::float32* var_20;
        wp::float32 var_21;
        wp::float32 var_22;
        wp::float32 var_23;
        wp::float32 var_24;
        //---------
        // forward
        // def _eff_contact_force(                                                                <L 3363>
        // cid = wp.tid()                                                                         <L 3375>
        var_0 = builtin_tid1d();
        // nnz = efm_con_nnz_in[cid]                                                              <L 3376>
        var_1 = wp::address(var_efm_con_nnz_in, var_0);
        var_3 = wp::load(var_1);
        var_2 = wp::copy(var_3);
        // if nnz == 0:                                                                           <L 3377>
        var_5 = (var_2 == var_4);
        if (var_5) {
            // return                                                                             <L 3378>
            continue;
        }
        // f = efm_con_force_in[cid]                                                              <L 3379>
        var_6 = wp::address(var_efm_con_force_in, var_0);
        var_8 = wp::load(var_6);
        var_7 = wp::copy(var_8);
        // if f == float(0.0):                                                                    <L 3380>
        var_10 = wp::float(var_9);
        var_11 = (var_7 == var_10);
        if (var_11) {
            // return                                                                             <L 3381>
            continue;
        }
        // worldid = contact_worldid_in[cid]                                                      <L 3382>
        var_12 = wp::address(var_contact_worldid_in, var_0);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // for a in range(nnz):                                                                   <L 3383>
        var_15 = wp::range(var_2);
        start_for_2:;
            if (iter_cmp(var_15) == 0) goto end_for_2;
            var_16 = wp::iter_next(var_15);
            // dof = efm_con_dof_in[cid, a]                                                       <L 3384>
            var_17 = wp::address(var_efm_con_dof_in, var_0, var_16);
            var_19 = wp::load(var_17);
            var_18 = wp::copy(var_19);
            // val = efm_con_val_in[cid, a]                                                       <L 3385>
            var_20 = wp::address(var_efm_con_val_in, var_0, var_16);
            var_22 = wp::load(var_20);
            var_21 = wp::copy(var_22);
            // wp.atomic_add(qfrc_spring_out, worldid, dof, f * val)                              <L 3386>
            var_23 = wp::mul(var_7, var_21);
            var_24 = wp::atomic_add(var_qfrc_spring_out, var_13, var_18, var_23);
            goto start_for_2;
        end_for_2:;
    }
}



extern "C" __global__ void _flex_hessian_clear_93e33936_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nflexvert,
    wp::int32 var_nflexedge,
    wp::array_t<wp::int32> var_flex_edgeflexid,
    wp::array_t<wp::int32> var_flex_vertflexid,
    wp::array_t<bool> var_flex_hessian_valid_in,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flexvert_hessian_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_flexedge_hessian_out)
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
        bool var_2;
        wp::int32* var_3;
        wp::int32 var_4;
        wp::int32 var_5;
        bool var_6;
        const wp::int32 var_7 = 0;
        bool var_8;
        bool* var_9;
        bool var_10;
        bool var_11;
        const wp::float32 var_12 = 0.0;
        const wp::float32 var_13 = 0.0;
        const wp::float32 var_14 = 0.0;
        const wp::float32 var_15 = 0.0;
        const wp::float32 var_16 = 0.0;
        const wp::float32 var_17 = 0.0;
        wp::vec_t<6, wp::float32> var_18;
        bool var_19;
        wp::int32* var_20;
        wp::int32 var_21;
        wp::int32 var_22;
        bool var_23;
        const wp::int32 var_24 = 0;
        bool var_25;
        bool* var_26;
        bool var_27;
        bool var_28;
        const wp::float32 var_29 = 0.0;
        const wp::float32 var_30 = 0.0;
        const wp::float32 var_31 = 0.0;
        const wp::float32 var_32 = 0.0;
        const wp::float32 var_33 = 0.0;
        const wp::float32 var_34 = 0.0;
        const wp::float32 var_35 = 0.0;
        const wp::float32 var_36 = 0.0;
        const wp::float32 var_37 = 0.0;
        wp::mat_t<3, 3, wp::float32> var_38;
        //---------
        // forward
        // def _flex_hessian_clear(                                                               <L 1644>
        // worldid, tid = wp.tid()                                                                <L 1656>
        builtin_tid2d(var_0, var_1);
        // if tid < nflexvert:                                                                    <L 1657>
        var_2 = (var_1 < var_nflexvert);
        if (var_2) {
            // f_v = flex_vertflexid[tid]                                                         <L 1658>
            var_3 = wp::address(var_flex_vertflexid, var_1);
            var_5 = wp::load(var_3);
            var_4 = wp::copy(var_5);
            // if f_v >= 0 and not flex_hessian_valid_in[worldid, f_v]:                           <L 1659>
            var_8 = (var_4 >= var_7);
            var_6 = var_8;
            if (var_6) {
                var_9 = wp::address(var_flex_hessian_valid_in, var_0, var_4);
                var_11 = wp::load(var_9);
                var_10 = wp::unot(var_11);
                var_6 = var_6 && var_10;
            }
            if (var_6) {
                // flexvert_hessian_out[worldid, tid] = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)        <L 1660>
                var_18 = wp::vec_t<6, wp::float32>({var_12, var_13, var_14, var_15, var_16, var_17});
                wp::array_store(var_flexvert_hessian_out, var_0, var_1, var_18);
            }
        }
        // if tid < nflexedge:                                                                    <L 1661>
        var_19 = (var_1 < var_nflexedge);
        if (var_19) {
            // f_e = flex_edgeflexid[tid]                                                         <L 1662>
            var_20 = wp::address(var_flex_edgeflexid, var_1);
            var_22 = wp::load(var_20);
            var_21 = wp::copy(var_22);
            // if f_e >= 0 and not flex_hessian_valid_in[worldid, f_e]:                           <L 1663>
            var_25 = (var_21 >= var_24);
            var_23 = var_25;
            if (var_23) {
                var_26 = wp::address(var_flex_hessian_valid_in, var_0, var_21);
                var_28 = wp::load(var_26);
                var_27 = wp::unot(var_28);
                var_23 = var_23 && var_27;
            }
            if (var_23) {
                // flexedge_hessian_out[worldid, tid] = wp.mat33(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)       <L 1664>
                var_38 = wp::mat_t<3, 3, wp::float32>(var_29, var_30, var_31, var_32, var_33, var_34, var_35, var_36, var_37);
                wp::array_store(var_flexedge_hessian_out, var_0, var_1, var_38);
            }
        }
    }
}



extern "C" __global__ void _flex_elasticity_88cad0ef_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::array_t<wp::float32> var_opt_timestep,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_vertadr,
    wp::array_t<wp::int32> var_flex_edgeadr,
    wp::array_t<wp::int32> var_flex_elemadr,
    wp::array_t<wp::int32> var_flex_elemdataadr,
    wp::array_t<wp::int32> var_flex_stiffnessadr,
    wp::array_t<wp::int32> var_flex_elemedgeadr,
    wp::array_t<wp::int32> var_flex_vertbodyid,
    wp::array_t<wp::int32> var_flex_elem,
    wp::array_t<wp::int32> var_flex_elemedge,
    wp::array_t<wp::float32> var_flexedge_length0,
    wp::array_t<wp::float32> var_flex_stiffness,
    wp::array_t<wp::float32> var_flex_damping,
    wp::array_t<wp::int32> var_flex_elemflexid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flexvert_xpos_in,
    wp::array_t<wp::float32> var_flexedge_length_in,
    wp::array_t<wp::float32> var_flexedge_velocity_in,
    bool var_dsbl_spring,
    bool var_dsbl_damper,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_spring_body_force_out,
    wp::array_t<wp::vec_t<6, wp::float32>> var_flex_damper_body_force_out)
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
        wp::int32* var_10;
        wp::int32 var_11;
        wp::int32 var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        wp::int32* var_16;
        wp::int32 var_17;
        wp::int32 var_18;
        const wp::int32 var_19 = 0;
        bool var_20;
        wp::float32* var_21;
        const wp::float32 var_22 = 0.0;
        bool var_23;
        wp::float32 var_24;
        bool var_25;
        const wp::float32 var_26 = 0.0;
        bool var_27;
        bool var_28;
        wp::float32* var_29;
        wp::float32 var_30;
        wp::float32 var_31;
        const wp::float32 var_32 = 0.0;
        wp::float32 var_33;
        bool var_34;
        const wp::float32 var_35 = 0.0;
        bool var_36;
        wp::int32* var_37;
        wp::int32 var_38;
        wp::int32 var_39;
        const wp::int32 var_40 = 1;
        wp::int32 var_41;
        const wp::int32 var_42 = 2;
        bool var_43;
        const wp::int32 var_44 = 3;
        const wp::int32 var_45 = 6;
        wp::int32 var_46;
        const wp::int32 var_47 = 3;
        bool var_48;
        const wp::int32 var_49 = 24;
        const wp::int32 var_50 = 21;
        wp::int32 var_51;
        wp::int32 var_52;
        wp::int32 var_53;
        bool var_54;
        const bool var_55 = true;
        const wp::int32 var_56 = 3;
        bool var_57;
        const wp::int32 var_58 = 21;
        wp::int32 var_59;
        wp::float32* var_60;
        const wp::float32 var_61 = 0.0;
        bool var_62;
        wp::float32 var_63;
        bool var_64;
        const wp::int32 var_65 = 1;
        bool var_66;
        const wp::int32 var_67 = 0;
        const wp::int32 var_68 = 1;
        const wp::int32 var_69 = 0;
        const wp::int32 var_70 = 0;
        const wp::int32 var_71 = 0;
        const wp::int32 var_72 = 0;
        const wp::int32 var_73 = 0;
        const wp::int32 var_74 = 0;
        const wp::int32 var_75 = 0;
        const wp::int32 var_76 = 0;
        const wp::int32 var_77 = 0;
        const wp::int32 var_78 = 0;
        const wp::int32 var_79 = 6;
        const wp::int32 var_80 = 2;
        wp::tuple_t<wp::int32, wp::int32> var_81;
        wp::mat_t<6, 2, wp::int32> var_82;
        wp::mat_t<6, 2, wp::int32> var_83;
        wp::mat_t<6, 2, wp::int32> var_84;
        wp::int32* var_85;
        const wp::int32 var_86 = 1;
        wp::int32 var_87;
        wp::int32 var_88;
        wp::int32 var_89;
        wp::int32 var_90;
        wp::int32* var_91;
        wp::int32 var_92;
        wp::int32 var_93;
        const wp::int32 var_94 = -1;
        const wp::int32 var_95 = -1;
        const wp::int32 var_96 = -1;
        const wp::int32 var_97 = -1;
        wp::vec_t<4, wp::int32> var_98;
        wp::mat_t<4, 3, wp::float32> var_99;
        wp::range_t var_100;
        wp::int32 var_101;
        wp::int32 var_102;
        wp::int32* var_103;
        wp::int32 var_104;
        wp::int32 var_105;
        wp::int32 var_106;
        wp::int32* var_107;
        wp::int32 var_108;
        wp::int32 var_109;
        bool var_110;
        const wp::int32 var_111 = 0;
        bool var_112;
        const wp::int32 var_113 = 0;
        bool var_114;
        wp::vec_t<3, wp::float32>* var_115;
        wp::vec_t<3, wp::float32> var_116;
        wp::mat_t<6, 3, wp::float32> var_117;
        wp::range_t var_118;
        wp::int32 var_119;
        const wp::int32 var_120 = 0;
        wp::int32 var_121;
        wp::vec_t<3, wp::float32> var_122;
        const wp::int32 var_123 = 1;
        wp::int32 var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::vec_t<3, wp::float32> var_126;
        wp::int32* var_127;
        wp::int32 var_128;
        wp::int32 var_129;
        wp::int32 var_130;
        wp::int32* var_131;
        wp::int32 var_132;
        wp::int32 var_133;
        const wp::float32 var_134 = 0.0;
        const wp::float32 var_135 = 0.0;
        const wp::float32 var_136 = 0.0;
        const wp::float32 var_137 = 0.0;
        const wp::float32 var_138 = 0.0;
        const wp::float32 var_139 = 0.0;
        wp::vec_t<6, wp::float32> var_140;
        bool var_141;
        wp::vec_t<6, wp::float32> var_142;
        wp::vec_t<6, wp::float32> var_143;
        wp::mat_t<6, 6, wp::float32> var_144;
        wp::vec_t<6, wp::float32> var_145;
        const wp::int32 var_146 = 21;
        wp::int32 var_147;
        wp::float32* var_148;
        wp::vec_t<6, wp::float32> var_149;
        wp::float32 var_150;
        wp::vec_t<6, wp::float32> var_151;
        const wp::float32 var_152 = 0.0;
        const wp::float32 var_153 = 0.0;
        const wp::float32 var_154 = 0.0;
        const wp::float32 var_155 = 0.0;
        const wp::float32 var_156 = 0.0;
        const wp::float32 var_157 = 0.0;
        wp::vec_t<6, wp::float32> var_158;
        bool var_159;
        const wp::float32 var_160 = 0.0;
        bool var_161;
        bool var_162;
        const wp::float32 var_163 = 0.0;
        const wp::float32 var_164 = 0.0;
        const wp::float32 var_165 = 0.0;
        const wp::float32 var_166 = 0.0;
        const wp::float32 var_167 = 0.0;
        const wp::float32 var_168 = 0.0;
        wp::vec_t<6, wp::float32> var_169;
        wp::range_t var_170;
        wp::int32 var_171;
        wp::int32 var_172;
        wp::int32* var_173;
        wp::int32 var_174;
        wp::int32 var_175;
        wp::float32* var_176;
        wp::float32 var_177;
        wp::float32 var_178;
        wp::float32* var_179;
        wp::float32 var_180;
        wp::float32 var_181;
        wp::float32 var_182;
        const wp::float32 var_183 = 2.0;
        wp::float32 var_184;
        wp::float32 var_185;
        wp::float32 var_186;
        wp::float32 var_187;
        wp::vec_t<6, wp::float32> var_188;
        wp::int32 var_189;
        wp::vec_t<6, wp::float32> var_190;
        const wp::float32 var_191 = 0.0;
        const wp::int32 var_192 = 6;
        const wp::int32 var_193 = 3;
        wp::tuple_t<wp::int32, wp::int32> var_194;
        wp::mat_t<6, 3, wp::float32> var_195;
        const wp::float32 var_196 = 0.0;
        const wp::int32 var_197 = 6;
        const wp::int32 var_198 = 3;
        wp::tuple_t<wp::int32, wp::int32> var_199;
        wp::mat_t<6, 3, wp::float32> var_200;
        wp::range_t var_201;
        wp::int32 var_202;
        wp::float32 var_203;
        wp::float32 var_204;
        const wp::int32 var_205 = 0;
        wp::int32 var_206;
        const wp::int32 var_207 = 0;
        const wp::int32 var_208 = 0;
        bool var_209;
        wp::float32 var_210;
        wp::float32 var_211;
        wp::float32 var_212;
        wp::float32 var_213;
        bool var_214;
        wp::float32 var_215;
        bool var_216;
        const wp::float32 var_217 = 0.0;
        bool var_218;
        bool var_219;
        wp::float32 var_220;
        const wp::int32 var_221 = 1;
        const wp::int32 var_222 = 0;
        bool var_223;
        wp::float32 var_224;
        wp::float32 var_225;
        wp::float32 var_226;
        wp::float32 var_227;
        bool var_228;
        wp::float32 var_229;
        bool var_230;
        const wp::float32 var_231 = 0.0;
        bool var_232;
        bool var_233;
        wp::float32 var_234;
        const wp::int32 var_235 = 2;
        const wp::int32 var_236 = 0;
        bool var_237;
        wp::float32 var_238;
        wp::float32 var_239;
        wp::float32 var_240;
        wp::float32 var_241;
        bool var_242;
        wp::float32 var_243;
        bool var_244;
        const wp::float32 var_245 = 0.0;
        bool var_246;
        bool var_247;
        wp::float32 var_248;
        const wp::int32 var_249 = 1;
        wp::int32 var_250;
        const wp::int32 var_251 = 0;
        const wp::int32 var_252 = 0;
        bool var_253;
        wp::float32 var_254;
        wp::float32 var_255;
        wp::float32 var_256;
        wp::float32 var_257;
        bool var_258;
        wp::float32 var_259;
        bool var_260;
        const wp::float32 var_261 = 0.0;
        bool var_262;
        bool var_263;
        wp::float32 var_264;
        const wp::int32 var_265 = 1;
        const wp::int32 var_266 = 0;
        bool var_267;
        wp::float32 var_268;
        wp::float32 var_269;
        wp::float32 var_270;
        wp::float32 var_271;
        bool var_272;
        wp::float32 var_273;
        bool var_274;
        const wp::float32 var_275 = 0.0;
        bool var_276;
        bool var_277;
        wp::float32 var_278;
        const wp::int32 var_279 = 2;
        const wp::int32 var_280 = 0;
        bool var_281;
        wp::float32 var_282;
        wp::float32 var_283;
        wp::float32 var_284;
        wp::float32 var_285;
        bool var_286;
        wp::float32 var_287;
        bool var_288;
        const wp::float32 var_289 = 0.0;
        bool var_290;
        bool var_291;
        wp::float32 var_292;
        bool var_293;
        bool var_294;
        const wp::int32 var_295 = 23;
        wp::int32 var_296;
        wp::float32* var_297;
        wp::float32 var_298;
        wp::mat_t<4, 3, wp::float32> var_299;
        wp::float32 var_300;
        const wp::float32 var_301 = 2.0;
        const wp::int32 var_302 = 22;
        wp::int32 var_303;
        wp::float32* var_304;
        wp::float32 var_305;
        wp::float32 var_306;
        const wp::float32 var_307 = 1.0;
        wp::float32 var_308;
        wp::float32 var_309;
        const wp::int32 var_310 = 0;
        wp::vec_t<3, wp::float32> var_311;
        const wp::int32 var_312 = 0;
        wp::float32 var_313;
        wp::float32 var_314;
        const wp::int32 var_315 = 1;
        wp::float32 var_316;
        wp::float32 var_317;
        const wp::int32 var_318 = 2;
        wp::float32 var_319;
        wp::float32 var_320;
        const wp::int32 var_321 = 1;
        wp::vec_t<3, wp::float32> var_322;
        const wp::int32 var_323 = 0;
        wp::float32 var_324;
        wp::float32 var_325;
        const wp::int32 var_326 = 1;
        wp::float32 var_327;
        wp::float32 var_328;
        const wp::int32 var_329 = 2;
        wp::float32 var_330;
        wp::float32 var_331;
        const wp::int32 var_332 = 2;
        wp::vec_t<3, wp::float32> var_333;
        const wp::int32 var_334 = 0;
        wp::float32 var_335;
        wp::float32 var_336;
        const wp::int32 var_337 = 1;
        wp::float32 var_338;
        wp::float32 var_339;
        const wp::int32 var_340 = 2;
        wp::float32 var_341;
        wp::float32 var_342;
        const wp::int32 var_343 = 3;
        wp::vec_t<3, wp::float32> var_344;
        const wp::int32 var_345 = 0;
        wp::float32 var_346;
        wp::float32 var_347;
        const wp::int32 var_348 = 1;
        wp::float32 var_349;
        wp::float32 var_350;
        const wp::int32 var_351 = 2;
        wp::float32 var_352;
        wp::float32 var_353;
        wp::int32 var_354;
        wp::int32 var_355;
        wp::range_t var_356;
        wp::int32 var_357;
        wp::int32 var_358;
        wp::vec_t<3, wp::float32> var_359;
        wp::vec_t<3, wp::float32>* var_360;
        wp::vec_t<3, wp::float32> var_361;
        wp::vec_t<3, wp::float32> var_362;
        wp::vec_t<3, wp::float32> var_363;
        bool var_364;
        wp::vec_t<3, wp::float32> var_365;
        wp::vec_t<3, wp::float32> var_366;
        wp::vec_t<3, wp::float32> var_367;
        wp::vec_t<6, wp::float32> var_368;
        wp::vec_t<6, wp::float32> var_369;
        bool var_370;
        const wp::float32 var_371 = 0.0;
        bool var_372;
        bool var_373;
        wp::vec_t<3, wp::float32> var_374;
        wp::vec_t<3, wp::float32> var_375;
        wp::vec_t<3, wp::float32> var_376;
        wp::vec_t<6, wp::float32> var_377;
        wp::vec_t<6, wp::float32> var_378;
        //---------
        // forward
        // def _flex_elasticity(                                                                  <L 2340>
        // worldid, elemid = wp.tid()                                                             <L 2369>
        builtin_tid2d(var_0, var_1);
        // timestep = opt_timestep[worldid % opt_timestep.shape[0]]                               <L 2370>
        var_2 = &(var_opt_timestep.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_0, var_4);
        var_7 = wp::address(var_opt_timestep, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // f = flex_elemflexid[elemid]                                                            <L 2372>
        var_10 = wp::address(var_flex_elemflexid, var_1);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // local_elemid = elemid - flex_elemadr[f]                                                <L 2373>
        var_13 = wp::address(var_flex_elemadr, var_11);
        var_15 = wp::load(var_13);
        var_14 = wp::sub(var_1, var_15);
        // stiffness_adr_base = flex_stiffnessadr[f]                                              <L 2375>
        var_16 = wp::address(var_flex_stiffnessadr, var_11);
        var_18 = wp::load(var_16);
        var_17 = wp::copy(var_18);
        // if stiffness_adr_base < 0:                                                             <L 2376>
        var_20 = (var_17 < var_19);
        if (var_20) {
            // return                                                                             <L 2377>
            continue;
        }
        // if flex_stiffness[stiffness_adr_base] == 0.0:                                          <L 2378>
        var_21 = wp::address(var_flex_stiffness, var_17);
        var_24 = wp::load(var_21);
        var_23 = (var_24 == var_22);
        if (var_23) {
            // return                                                                             <L 2379>
            continue;
        }
        // if timestep > 0.0 and not dsbl_damper:                                                 <L 2381>
        var_27 = (var_8 > var_26);
        var_25 = var_27;
        if (var_25) {
            var_28 = wp::unot(var_dsbl_damper);
            var_25 = var_25 && var_28;
        }
        if (var_25) {
            // kD = flex_damping[f] / timestep                                                    <L 2382>
            var_29 = wp::address(var_flex_damping, var_11);
            var_31 = wp::load(var_29);
            var_30 = wp::div(var_31, var_8);
        }
        if (!var_25) {
            // kD = 0.0                                                                           <L 2384>
        }
        var_33 = wp::where(var_25, var_30, var_32);
        // if dsbl_spring and kD == 0.0:                                                          <L 2385>
        var_34 = var_dsbl_spring;
        if (var_34) {
            var_36 = (var_33 == var_35);
            var_34 = var_34 && var_36;
        }
        if (var_34) {
            // return                                                                             <L 2386>
            continue;
        }
        // dim = flex_dim[f]                                                                      <L 2388>
        var_37 = wp::address(var_flex_dim, var_11);
        var_39 = wp::load(var_37);
        var_38 = wp::copy(var_39);
        // nvert = dim + 1                                                                        <L 2389>
        var_41 = wp::add(var_38, var_40);
        // nedge = 3 if dim == 2 else 6                                                           <L 2390>
        var_43 = (var_38 == var_42);
        if (var_43) {
        }
        if (!var_43) {
        }
        var_46 = wp::where(var_43, var_44, var_45);
        // stiffness_adr = stiffness_adr_base + local_elemid * (FLEX_STIFFNESS_3D if dim == 3 else 21)       <L 2392>
        var_48 = (var_38 == var_47);
        if (var_48) {
        }
        if (!var_48) {
        }
        var_51 = wp::where(var_48, var_49, var_50);
        var_52 = wp::mul(var_14, var_51);
        var_53 = wp::add(var_17, var_52);
        // snh = wp.static(FLEX_STIFFNESS_3D == 24) and dim == 3 and flex_stiffness[stiffness_adr + 21] != 0.0       <L 2393>
        var_54 = var_55;
        if (var_54) {
            var_57 = (var_38 == var_56);
            var_54 = var_54 && var_57;
        }
        if (var_54) {
            var_59 = wp::add(var_53, var_58);
            var_60 = wp::address(var_flex_stiffness, var_59);
            var_63 = wp::load(var_60);
            var_62 = (var_63 != var_61);
            var_54 = var_54 && var_62;
        }
        // if snh and dsbl_spring:                                                                <L 2394>
        var_64 = var_54;
        if (var_64) {
            var_64 = var_64 && var_dsbl_spring;
        }
        if (var_64) {
            // return                                                                             <L 2395>
            continue;
        }
        // edges = wp.where(                                                                      <L 2397>
        // dim == 1,                                                                              <L 2398>
        var_66 = (var_38 == var_65);
        // wp.matrix(0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, shape=(6, 2), dtype=int),                <L 2399>
        var_81 = wp::tuple(var_79, var_80);
        var_82 = wp::mat_t<6, 2, wp::int32>({var_67, var_68, var_69, var_70, var_71, var_72, var_73, var_74, var_75, var_76, var_77, var_78});
        // stretch_edge_endpoints(dim),                                                           <L 2400>
        var_83 = stretch_edge_endpoints_0(var_38);
        var_84 = wp::where(var_66, var_82, var_83);
        // elem_data_adr = flex_elemdataadr[f] + local_elemid * (dim + 1)                         <L 2403>
        var_85 = wp::address(var_flex_elemdataadr, var_11);
        var_87 = wp::add(var_38, var_86);
        var_88 = wp::mul(var_14, var_87);
        var_90 = wp::load(var_85);
        var_89 = wp::add(var_90, var_88);
        // vbase = flex_vertadr[f]                                                                <L 2404>
        var_91 = wp::address(var_flex_vertadr, var_11);
        var_93 = wp::load(var_91);
        var_92 = wp::copy(var_93);
        // vert_body = wp.vec4i(-1, -1, -1, -1)                                                   <L 2406>
        var_98 = wp::vec_t<4, wp::int32>(var_94, var_95, var_96, var_97);
        // vert_xpos = mat43()                                                                    <L 2407>
        var_99 = wp::mat_t<4, 3, wp::float32>();
        // for v in range(nvert):                                                                 <L 2408>
        var_100 = wp::range(var_41);
        start_for_4:;
            if (iter_cmp(var_100) == 0) goto end_for_4;
            var_101 = wp::iter_next(var_100);
            // vert = flex_elem[elem_data_adr + v]                                                <L 2409>
            var_102 = wp::add(var_89, var_101);
            var_103 = wp::address(var_flex_elem, var_102);
            var_105 = wp::load(var_103);
            var_104 = wp::copy(var_105);
            // gvert = vbase + vert                                                               <L 2410>
            var_106 = wp::add(var_92, var_104);
            // bodyid = flex_vertbodyid[gvert]                                                    <L 2411>
            var_107 = wp::address(var_flex_vertbodyid, var_106);
            var_109 = wp::load(var_107);
            var_108 = wp::copy(var_109);
            // if v == 0 and bodyid < 0:                                                          <L 2412>
            var_112 = (var_101 == var_111);
            var_110 = var_112;
            if (var_110) {
                var_114 = (var_108 < var_113);
                var_110 = var_110 && var_114;
            }
            if (var_110) {
                // return                                                                         <L 2413>
                continue;
            }
            // vert_body[v] = bodyid                                                              <L 2414>
            wp::assign_inplace(var_98, var_101, var_108);
            // vert_xpos[v] = flexvert_xpos_in[worldid, gvert]                                    <L 2415>
            var_115 = wp::address(var_flexvert_xpos_in, var_0, var_106);
            var_116 = wp::load(var_115);
            wp::assign_inplace(var_99, var_101, var_116);
            goto start_for_4;
        end_for_4:;
        // edgevec = mat63()                                                                      <L 2417>
        var_117 = wp::mat_t<6, 3, wp::float32>();
        // for e in range(nedge):                                                                 <L 2418>
        var_118 = wp::range(var_46);
        start_for_7:;
            if (iter_cmp(var_118) == 0) goto end_for_7;
            var_119 = wp::iter_next(var_118);
            // edgevec[e] = vert_xpos[edges[e, 0]] - vert_xpos[edges[e, 1]]                       <L 2419>
            var_121 = wp::extract(var_84, var_119, var_120);
            var_122 = wp::extract(var_99, var_121);
            var_124 = wp::extract(var_84, var_119, var_123);
            var_125 = wp::extract(var_99, var_124);
            var_126 = wp::sub(var_122, var_125);
            wp::assign_inplace(var_117, var_119, var_126);
            goto start_for_7;
        end_for_7:;
        // elemedge_adr = flex_elemedgeadr[f] + local_elemid * nedge                              <L 2421>
        var_127 = wp::address(var_flex_elemedgeadr, var_11);
        var_128 = wp::mul(var_14, var_46);
        var_130 = wp::load(var_127);
        var_129 = wp::add(var_130, var_128);
        // edge_adr = flex_edgeadr[f]                                                             <L 2422>
        var_131 = wp::address(var_flex_edgeadr, var_11);
        var_133 = wp::load(var_131);
        var_132 = wp::copy(var_133);
        // elongation_spring = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                 <L 2424>
        var_140 = wp::vec_t<6, wp::float32>({var_134, var_135, var_136, var_137, var_138, var_139});
        // if not dsbl_spring:                                                                    <L 2425>
        var_141 = wp::unot(var_dsbl_spring);
        if (var_141) {
            // elongation_spring = stretch_elongation(                                            <L 2426>
            // flex_elemedge, flexedge_length0, flexedge_length_in, worldid, elemedge_adr, edge_adr, nedge       <L 2427>
            var_142 = stretch_elongation_0(var_flex_elemedge, var_flexedge_length0, var_flexedge_length_in, var_0, var_129, var_132, var_46);
        }
        var_143 = wp::where(var_141, var_142, var_140);
        // metric, tension_spring = stretch_elasticity(flex_stiffness, stiffness_adr, elongation_spring, nedge)       <L 2430>
        stretch_elasticity_0(var_flex_stiffness, var_53, var_143, var_46, var_144, var_145);
        // if snh:                                                                                <L 2431>
        if (var_54) {
            // tension_spring = _snh_cubic(tension_spring, elongation_spring, flex_stiffness[stiffness_adr + 21])       <L 2432>
            var_147 = wp::add(var_53, var_146);
            var_148 = wp::address(var_flex_stiffness, var_147);
            var_150 = wp::load(var_148);
            var_149 = _snh_cubic_0(var_145, var_143, var_150);
        }
        var_151 = wp::where(var_54, var_149, var_145);
        // tension_damper = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                                    <L 2434>
        var_158 = wp::vec_t<6, wp::float32>({var_152, var_153, var_154, var_155, var_156, var_157});
        // if kD > 0.0 and not snh:                                                               <L 2435>
        var_161 = (var_33 > var_160);
        var_159 = var_161;
        if (var_159) {
            var_162 = wp::unot(var_54);
            var_159 = var_159 && var_162;
        }
        if (var_159) {
            // elongation_damper = vec6(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)                             <L 2436>
            var_169 = wp::vec_t<6, wp::float32>({var_163, var_164, var_165, var_166, var_167, var_168});
            // for e in range(nedge):                                                             <L 2437>
            var_170 = wp::range(var_46);
            start_for_9:;
                if (iter_cmp(var_170) == 0) goto end_for_9;
                var_171 = wp::iter_next(var_170);
                // e_idx = edge_adr + flex_elemedge[elemedge_adr + e]                             <L 2438>
                var_172 = wp::add(var_129, var_171);
                var_173 = wp::address(var_flex_elemedge, var_172);
                var_175 = wp::load(var_173);
                var_174 = wp::add(var_132, var_175);
                // deformed = flexedge_length_in[worldid, e_idx]                                  <L 2439>
                var_176 = wp::address(var_flexedge_length_in, var_0, var_174);
                var_178 = wp::load(var_176);
                var_177 = wp::copy(var_178);
                // vel = flexedge_velocity_in[worldid, e_idx]                                     <L 2440>
                var_179 = wp::address(var_flexedge_velocity_in, var_0, var_174);
                var_181 = wp::load(var_179);
                var_180 = wp::copy(var_181);
                // dL = vel * timestep                                                            <L 2441>
                var_182 = wp::mul(var_180, var_8);
                // elongation_damper[e] = dL * (2.0 * deformed - dL) * kD                         <L 2442>
                var_184 = wp::mul(var_183, var_177);
                var_185 = wp::sub(var_184, var_182);
                var_186 = wp::mul(var_182, var_185);
                var_187 = wp::mul(var_186, var_33);
                wp::assign_inplace(var_169, var_171, var_187);
                goto start_for_9;
            end_for_9:;
            // tension_damper = stretch_tension(metric, elongation_damper, nedge)                 <L 2443>
            var_188 = stretch_tension_0(var_144, var_169, var_46);
        }
        var_189 = wp::where(var_159, var_171, var_119);
        var_190 = wp::where(var_159, var_188, var_158);
        // force_spring = wp.matrix(0.0, shape=(6, 3))                                            <L 2445>
        var_194 = wp::tuple(var_192, var_193);
        var_195 = wp::mat_t<6, 3, wp::float32>(var_191);
        // force_damper = wp.matrix(0.0, shape=(6, 3))                                            <L 2446>
        var_199 = wp::tuple(var_197, var_198);
        var_200 = wp::mat_t<6, 3, wp::float32>(var_196);
        // for ed2 in range(nedge):                                                               <L 2447>
        var_201 = wp::range(var_46);
        start_for_11:;
            if (iter_cmp(var_201) == 0) goto end_for_11;
            var_202 = wp::iter_next(var_201);
            // s_val = tension_spring[ed2]                                                        <L 2448>
            var_203 = wp::extract(var_151, var_202);
            // d_val = tension_damper[ed2]                                                        <L 2449>
            var_204 = wp::extract(var_190, var_202);
            // for i in range(2):                                                                 <L 2450>
            // vi = edges[ed2, i]                                                                 <L 2451>
            var_206 = wp::extract(var_84, var_202, var_205);
            // for x in range(3):                                                                 <L 2452>
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_209 = (var_205 == var_208);
            if (var_209) {
                var_210 = wp::extract(var_117, var_202, var_207);
            }
            if (!var_209) {
                var_211 = wp::extract(var_117, var_202, var_207);
                var_212 = wp::neg(var_211);
            }
            var_213 = wp::where(var_209, var_210, var_212);
            // if not dsbl_spring:                                                                <L 2454>
            var_214 = wp::unot(var_dsbl_spring);
            if (var_214) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_215 = wp::mul(var_203, var_213);
                wp::sub_inplace(var_195, var_206, var_207, var_215);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_218 = (var_33 > var_217);
            var_216 = var_218;
            if (var_216) {
                var_219 = wp::unot(var_54);
                var_216 = var_216 && var_219;
            }
            if (var_216) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_220 = wp::mul(var_204, var_213);
                wp::sub_inplace(var_200, var_206, var_207, var_220);
            }
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_223 = (var_205 == var_222);
            if (var_223) {
                var_224 = wp::extract(var_117, var_202, var_221);
            }
            if (!var_223) {
                var_225 = wp::extract(var_117, var_202, var_221);
                var_226 = wp::neg(var_225);
            }
            var_227 = wp::where(var_223, var_224, var_226);
            // if not dsbl_spring:                                                                <L 2454>
            var_228 = wp::unot(var_dsbl_spring);
            if (var_228) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_229 = wp::mul(var_203, var_227);
                wp::sub_inplace(var_195, var_206, var_221, var_229);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_232 = (var_33 > var_231);
            var_230 = var_232;
            if (var_230) {
                var_233 = wp::unot(var_54);
                var_230 = var_230 && var_233;
            }
            if (var_230) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_234 = wp::mul(var_204, var_227);
                wp::sub_inplace(var_200, var_206, var_221, var_234);
            }
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_237 = (var_205 == var_236);
            if (var_237) {
                var_238 = wp::extract(var_117, var_202, var_235);
            }
            if (!var_237) {
                var_239 = wp::extract(var_117, var_202, var_235);
                var_240 = wp::neg(var_239);
            }
            var_241 = wp::where(var_237, var_238, var_240);
            // if not dsbl_spring:                                                                <L 2454>
            var_242 = wp::unot(var_dsbl_spring);
            if (var_242) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_243 = wp::mul(var_203, var_241);
                wp::sub_inplace(var_195, var_206, var_235, var_243);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_246 = (var_33 > var_245);
            var_244 = var_246;
            if (var_244) {
                var_247 = wp::unot(var_54);
                var_244 = var_244 && var_247;
            }
            if (var_244) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_248 = wp::mul(var_204, var_241);
                wp::sub_inplace(var_200, var_206, var_235, var_248);
            }
            // vi = edges[ed2, i]                                                                 <L 2451>
            var_250 = wp::extract(var_84, var_202, var_249);
            // for x in range(3):                                                                 <L 2452>
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_253 = (var_249 == var_252);
            if (var_253) {
                var_254 = wp::extract(var_117, var_202, var_251);
            }
            if (!var_253) {
                var_255 = wp::extract(var_117, var_202, var_251);
                var_256 = wp::neg(var_255);
            }
            var_257 = wp::where(var_253, var_254, var_256);
            // if not dsbl_spring:                                                                <L 2454>
            var_258 = wp::unot(var_dsbl_spring);
            if (var_258) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_259 = wp::mul(var_203, var_257);
                wp::sub_inplace(var_195, var_250, var_251, var_259);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_262 = (var_33 > var_261);
            var_260 = var_262;
            if (var_260) {
                var_263 = wp::unot(var_54);
                var_260 = var_260 && var_263;
            }
            if (var_260) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_264 = wp::mul(var_204, var_257);
                wp::sub_inplace(var_200, var_250, var_251, var_264);
            }
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_267 = (var_249 == var_266);
            if (var_267) {
                var_268 = wp::extract(var_117, var_202, var_265);
            }
            if (!var_267) {
                var_269 = wp::extract(var_117, var_202, var_265);
                var_270 = wp::neg(var_269);
            }
            var_271 = wp::where(var_267, var_268, var_270);
            // if not dsbl_spring:                                                                <L 2454>
            var_272 = wp::unot(var_dsbl_spring);
            if (var_272) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_273 = wp::mul(var_203, var_271);
                wp::sub_inplace(var_195, var_250, var_265, var_273);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_276 = (var_33 > var_275);
            var_274 = var_276;
            if (var_274) {
                var_277 = wp::unot(var_54);
                var_274 = var_274 && var_277;
            }
            if (var_274) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_278 = wp::mul(var_204, var_271);
                wp::sub_inplace(var_200, var_250, var_265, var_278);
            }
            // g_val = edgevec[ed2, x] if i == 0 else -edgevec[ed2, x]                            <L 2453>
            var_281 = (var_249 == var_280);
            if (var_281) {
                var_282 = wp::extract(var_117, var_202, var_279);
            }
            if (!var_281) {
                var_283 = wp::extract(var_117, var_202, var_279);
                var_284 = wp::neg(var_283);
            }
            var_285 = wp::where(var_281, var_282, var_284);
            // if not dsbl_spring:                                                                <L 2454>
            var_286 = wp::unot(var_dsbl_spring);
            if (var_286) {
                // force_spring[vi, x] -= s_val * g_val                                           <L 2455>
                var_287 = wp::mul(var_203, var_285);
                wp::sub_inplace(var_195, var_250, var_279, var_287);
            }
            // if kD > 0.0 and not snh:                                                           <L 2456>
            var_290 = (var_33 > var_289);
            var_288 = var_290;
            if (var_288) {
                var_291 = wp::unot(var_54);
                var_288 = var_288 && var_291;
            }
            if (var_288) {
                // force_damper[vi, x] -= d_val * g_val                                           <L 2457>
                var_292 = wp::mul(var_204, var_285);
                wp::sub_inplace(var_200, var_250, var_279, var_292);
            }
            goto start_for_11;
        end_for_11:;
        // if snh and not dsbl_spring:                                                            <L 2459>
        var_293 = var_54;
        if (var_293) {
            var_294 = wp::unot(var_dsbl_spring);
            var_293 = var_293 && var_294;
        }
        if (var_293) {
            // J, vol_grad = _snh_volume(edgevec, flex_stiffness[stiffness_adr + 23])             <L 2460>
            var_296 = wp::add(var_53, var_295);
            var_297 = wp::address(var_flex_stiffness, var_296);
            var_300 = wp::load(var_297);
            _snh_volume_0(var_117, var_300, var_298, var_299);
            // pressure = 2.0 * flex_stiffness[stiffness_adr + 22] * (J - 1.0)                    <L 2461>
            var_303 = wp::add(var_53, var_302);
            var_304 = wp::address(var_flex_stiffness, var_303);
            var_306 = wp::load(var_304);
            var_305 = wp::mul(var_301, var_306);
            var_308 = wp::sub(var_298, var_307);
            var_309 = wp::mul(var_305, var_308);
            // for v in range(4):                                                                 <L 2462>
            // gv = vol_grad[v]                                                                   <L 2463>
            var_311 = wp::extract(var_299, var_310);
            // for x in range(3):                                                                 <L 2464>
            // force_spring[v, x] -= pressure * gv[x]                                             <L 2465>
            var_313 = wp::extract(var_311, var_312);
            var_314 = wp::mul(var_309, var_313);
            wp::sub_inplace(var_195, var_310, var_312, var_314);
            var_316 = wp::extract(var_311, var_315);
            var_317 = wp::mul(var_309, var_316);
            wp::sub_inplace(var_195, var_310, var_315, var_317);
            var_319 = wp::extract(var_311, var_318);
            var_320 = wp::mul(var_309, var_319);
            wp::sub_inplace(var_195, var_310, var_318, var_320);
            // gv = vol_grad[v]                                                                   <L 2463>
            var_322 = wp::extract(var_299, var_321);
            // for x in range(3):                                                                 <L 2464>
            // force_spring[v, x] -= pressure * gv[x]                                             <L 2465>
            var_324 = wp::extract(var_322, var_323);
            var_325 = wp::mul(var_309, var_324);
            wp::sub_inplace(var_195, var_321, var_323, var_325);
            var_327 = wp::extract(var_322, var_326);
            var_328 = wp::mul(var_309, var_327);
            wp::sub_inplace(var_195, var_321, var_326, var_328);
            var_330 = wp::extract(var_322, var_329);
            var_331 = wp::mul(var_309, var_330);
            wp::sub_inplace(var_195, var_321, var_329, var_331);
            // gv = vol_grad[v]                                                                   <L 2463>
            var_333 = wp::extract(var_299, var_332);
            // for x in range(3):                                                                 <L 2464>
            // force_spring[v, x] -= pressure * gv[x]                                             <L 2465>
            var_335 = wp::extract(var_333, var_334);
            var_336 = wp::mul(var_309, var_335);
            wp::sub_inplace(var_195, var_332, var_334, var_336);
            var_338 = wp::extract(var_333, var_337);
            var_339 = wp::mul(var_309, var_338);
            wp::sub_inplace(var_195, var_332, var_337, var_339);
            var_341 = wp::extract(var_333, var_340);
            var_342 = wp::mul(var_309, var_341);
            wp::sub_inplace(var_195, var_332, var_340, var_342);
            // gv = vol_grad[v]                                                                   <L 2463>
            var_344 = wp::extract(var_299, var_343);
            // for x in range(3):                                                                 <L 2464>
            // force_spring[v, x] -= pressure * gv[x]                                             <L 2465>
            var_346 = wp::extract(var_344, var_345);
            var_347 = wp::mul(var_309, var_346);
            wp::sub_inplace(var_195, var_343, var_345, var_347);
            var_349 = wp::extract(var_344, var_348);
            var_350 = wp::mul(var_309, var_349);
            wp::sub_inplace(var_195, var_343, var_348, var_350);
            var_352 = wp::extract(var_344, var_351);
            var_353 = wp::mul(var_309, var_352);
            wp::sub_inplace(var_195, var_343, var_351, var_353);
        }
        var_354 = wp::where(var_293, var_343, var_101);
        var_355 = wp::where(var_293, var_351, var_279);
        // for v in range(nvert):                                                                 <L 2467>
        var_356 = wp::range(var_41);
        start_for_13:;
            if (iter_cmp(var_356) == 0) goto end_for_13;
            var_357 = wp::iter_next(var_356);
            // bodyid = vert_body[v]                                                              <L 2468>
            var_358 = wp::extract(var_98, var_357);
            // node_pos = vert_xpos[v]                                                            <L 2469>
            var_359 = wp::extract(var_99, var_357);
            // body_xipos = xipos_in[worldid, bodyid]                                             <L 2470>
            var_360 = wp::address(var_xipos_in, var_0, var_358);
            var_362 = wp::load(var_360);
            var_361 = wp::copy(var_362);
            // offset = body_xipos - node_pos                                                     <L 2471>
            var_363 = wp::sub(var_361, var_359);
            // if not dsbl_spring:                                                                <L 2472>
            var_364 = wp::unot(var_dsbl_spring);
            if (var_364) {
                // frc_s = force_spring[v]                                                        <L 2473>
                var_365 = wp::extract(var_195, var_357);
                // spatial_frc_s = wp.spatial_vector(frc_s, -wp.cross(offset, frc_s))             <L 2474>
                var_366 = wp::cross(var_363, var_365);
                var_367 = wp::neg(var_366);
                var_368 = wp::vec_t<6, wp::float32>(var_365, var_367);
                // wp.atomic_add(flex_spring_body_force_out, worldid, bodyid, spatial_frc_s)       <L 2475>
                var_369 = wp::atomic_add(var_flex_spring_body_force_out, var_0, var_358, var_368);
            }
            // if kD > 0.0 and not snh:                                                           <L 2476>
            var_372 = (var_33 > var_371);
            var_370 = var_372;
            if (var_370) {
                var_373 = wp::unot(var_54);
                var_370 = var_370 && var_373;
            }
            if (var_370) {
                // frc_d = force_damper[v]                                                        <L 2477>
                var_374 = wp::extract(var_200, var_357);
                // spatial_frc_d = wp.spatial_vector(frc_d, -wp.cross(offset, frc_d))             <L 2478>
                var_375 = wp::cross(var_363, var_374);
                var_376 = wp::neg(var_375);
                var_377 = wp::vec_t<6, wp::float32>(var_374, var_376);
                // wp.atomic_add(flex_damper_body_force_out, worldid, bodyid, spatial_frc_d)       <L 2479>
                var_378 = wp::atomic_add(var_flex_damper_body_force_out, var_0, var_358, var_377);
            }
            wp::assign(var_108, var_358);
            goto start_for_13;
        end_for_13:;
    }
}



extern "C" __global__ void _gravity_force_7c63ac28_cuda_kernel_forward(
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
        const wp::int32 var_3 = 1;
        wp::int32 var_4;
        wp::shape_t* var_5;
        const wp::int32 var_6 = 0;
        wp::int32 var_7;
        wp::shape_t var_8;
        wp::int32 var_9;
        wp::float32* var_10;
        wp::float32 var_11;
        wp::float32 var_12;
        wp::shape_t* var_13;
        const wp::int32 var_14 = 0;
        wp::int32 var_15;
        wp::shape_t var_16;
        wp::int32 var_17;
        wp::vec_t<3, wp::float32>* var_18;
        wp::vec_t<3, wp::float32> var_19;
        wp::vec_t<3, wp::float32> var_20;
        wp::vec_t<3, wp::float32> var_21;
        wp::shape_t* var_22;
        const wp::int32 var_23 = 0;
        wp::int32 var_24;
        wp::shape_t var_25;
        wp::int32 var_26;
        wp::float32* var_27;
        wp::vec_t<3, wp::float32> var_28;
        wp::float32 var_29;
        wp::vec_t<3, wp::float32> var_30;
        wp::vec_t<3, wp::float32>* var_31;
        wp::vec_t<3, wp::float32> var_32;
        wp::vec_t<3, wp::float32> var_33;
        wp::vec_t<3, wp::float32> var_34;
        wp::vec_t<3, wp::float32> var_35;
        wp::slice_t var_36;
        const wp::int32 var_37 = 0;
        wp::array_t<wp::float32> var_38;
        wp::float32 var_39;
        wp::float32 var_40;
        //---------
        // forward
        // def _gravity_force(                                                                    <L 327>
        // worldid, bodyid, dofid = wp.tid()                                                      <L 343>
        builtin_tid3d(var_0, var_1, var_2);
        // bodyid += 1  # skip world body                                                         <L 344>
        var_4 = wp::add(var_1, var_3);
        // gravcomp = body_gravcomp[worldid % body_gravcomp.shape[0], bodyid]                     <L 345>
        var_5 = &(var_body_gravcomp.shape);
        var_8 = wp::load(var_5);
        var_7 = wp::extract(var_8, var_6);
        var_9 = wp::mod(var_0, var_7);
        var_10 = wp::address(var_body_gravcomp, var_9, var_4);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // gravity = opt_gravity[worldid % opt_gravity.shape[0]]                                  <L 346>
        var_13 = &(var_opt_gravity.shape);
        var_16 = wp::load(var_13);
        var_15 = wp::extract(var_16, var_14);
        var_17 = wp::mod(var_0, var_15);
        var_18 = wp::address(var_opt_gravity, var_17);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // if gravcomp:                                                                           <L 348>
        if (var_11) {
            // force = -gravity * body_mass[worldid % body_mass.shape[0], bodyid] * gravcomp       <L 349>
            var_21 = wp::neg(var_19);
            var_22 = &(var_body_mass.shape);
            var_25 = wp::load(var_22);
            var_24 = wp::extract(var_25, var_23);
            var_26 = wp::mod(var_0, var_24);
            var_27 = wp::address(var_body_mass, var_26, var_4);
            var_29 = wp::load(var_27);
            var_28 = wp::mul(var_21, var_29);
            var_30 = wp::mul(var_28, var_11);
            // pos = xipos_in[worldid, bodyid]                                                    <L 350>
            var_31 = wp::address(var_xipos_in, var_0, var_4);
            var_33 = wp::load(var_31);
            var_32 = wp::copy(var_33);
            // jac, _ = support.jac_dof(                                                          <L 351>
            // body_parentid, body_rootid, dof_bodyid, body_isdofancestor, subtree_com_in, cdof_in, pos, bodyid, dofid, worldid       <L 352>
            jac_dof_0(var_body_parentid, var_body_rootid, var_dof_bodyid, var_body_isdofancestor, var_subtree_com_in, var_cdof_in, var_32, var_4, var_2, var_0, var_34, var_35);
            // wp.atomic_add(qfrc_gravcomp_out[worldid], dofid, wp.dot(jac, force))               <L 355>
            var_36 = wp::slice_t(var_0, var_0, var_37);
            var_38 = wp::view(var_qfrc_gravcomp_out, var_36);
            var_39 = wp::dot(var_34, var_30);
            var_40 = wp::atomic_add(var_38, var_2, var_39);
        }
    }
}

