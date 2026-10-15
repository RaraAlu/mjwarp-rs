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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:279
static CUDA_CALLABLE void _mix_flex_contact_params_0(
    wp::int32 var_a_condim,
    wp::int32 var_a_priority,
    wp::float32 var_a_solmix,
    wp::vec_t<2, wp::float32> var_a_solref,
    wp::vec_t<5, wp::float32> var_a_solimp,
    wp::vec_t<3, wp::float32> var_a_friction,
    wp::float32 var_a_gap,
    wp::int32 var_b_condim,
    wp::int32 var_b_priority,
    wp::float32 var_b_solmix,
    wp::vec_t<2, wp::float32> var_b_solref,
    wp::vec_t<5, wp::float32> var_b_solimp,
    wp::vec_t<3, wp::float32> var_b_friction,
    wp::float32 var_b_gap,
    wp::int32 & ret_0,
    wp::float32 & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<5, wp::float32> & ret_3,
    wp::vec_t<5, wp::float32> & ret_4)
{
    //---------
    // primal vars
    wp::float32 var_0;
    bool var_1;
    wp::int32 var_2;
    wp::vec_t<2, wp::float32> var_3;
    wp::vec_t<5, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    bool var_6;
    wp::int32 var_7;
    wp::vec_t<2, wp::float32> var_8;
    wp::vec_t<5, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::int32 var_11;
    wp::vec_t<2, wp::float32> var_12;
    wp::vec_t<5, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::int32 var_15;
    bool var_16;
    const wp::float32 var_17 = 1e-15;
    bool var_18;
    bool var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    bool var_22;
    bool var_23;
    bool var_24;
    const wp::float32 var_25 = 0.5;
    wp::float32 var_26;
    bool var_27;
    const wp::float32 var_28 = 0.0;
    wp::float32 var_29;
    const wp::float32 var_30 = 1.0;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    bool var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    const wp::float32 var_37 = 0.0;
    bool var_38;
    const wp::int32 var_39 = 0;
    wp::float32 var_40;
    const wp::float32 var_41 = 0.0;
    bool var_42;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::float32 var_46 = 1.0;
    wp::float32 var_47;
    const wp::int32 var_48 = 0;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::int32 var_52 = 1;
    wp::float32 var_53;
    wp::float32 var_54;
    const wp::float32 var_55 = 1.0;
    wp::float32 var_56;
    const wp::int32 var_57 = 1;
    wp::float32 var_58;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::vec_t<2, wp::float32> var_61;
    wp::vec_t<2, wp::float32> var_62;
    const wp::int32 var_63 = 0;
    wp::float32 var_64;
    const wp::int32 var_65 = 0;
    wp::float32 var_66;
    wp::float32 var_67;
    const wp::int32 var_68 = 1;
    wp::float32 var_69;
    const wp::int32 var_70 = 1;
    wp::float32 var_71;
    wp::float32 var_72;
    wp::vec_t<2, wp::float32> var_73;
    wp::vec_t<2, wp::float32> var_74;
    const wp::int32 var_75 = 0;
    wp::float32 var_76;
    wp::float32 var_77;
    const wp::float32 var_78 = 1.0;
    wp::float32 var_79;
    const wp::int32 var_80 = 0;
    wp::float32 var_81;
    wp::float32 var_82;
    wp::float32 var_83;
    const wp::int32 var_84 = 1;
    wp::float32 var_85;
    wp::float32 var_86;
    const wp::float32 var_87 = 1.0;
    wp::float32 var_88;
    const wp::int32 var_89 = 1;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    const wp::int32 var_93 = 2;
    wp::float32 var_94;
    wp::float32 var_95;
    const wp::float32 var_96 = 1.0;
    wp::float32 var_97;
    const wp::int32 var_98 = 2;
    wp::float32 var_99;
    wp::float32 var_100;
    wp::float32 var_101;
    const wp::int32 var_102 = 3;
    wp::float32 var_103;
    wp::float32 var_104;
    const wp::float32 var_105 = 1.0;
    wp::float32 var_106;
    const wp::int32 var_107 = 3;
    wp::float32 var_108;
    wp::float32 var_109;
    wp::float32 var_110;
    const wp::int32 var_111 = 4;
    wp::float32 var_112;
    wp::float32 var_113;
    const wp::float32 var_114 = 1.0;
    wp::float32 var_115;
    const wp::int32 var_116 = 4;
    wp::float32 var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    wp::vec_t<5, wp::float32> var_120;
    const wp::int32 var_121 = 0;
    wp::float32 var_122;
    const wp::int32 var_123 = 0;
    wp::float32 var_124;
    wp::float32 var_125;
    const wp::int32 var_126 = 1;
    wp::float32 var_127;
    const wp::int32 var_128 = 1;
    wp::float32 var_129;
    wp::float32 var_130;
    const wp::int32 var_131 = 2;
    wp::float32 var_132;
    const wp::int32 var_133 = 2;
    wp::float32 var_134;
    wp::float32 var_135;
    wp::vec_t<3, wp::float32> var_136;
    wp::int32 var_137;
    wp::vec_t<2, wp::float32> var_138;
    wp::vec_t<5, wp::float32> var_139;
    wp::vec_t<3, wp::float32> var_140;
    wp::int32 var_141;
    wp::vec_t<2, wp::float32> var_142;
    wp::vec_t<5, wp::float32> var_143;
    wp::vec_t<3, wp::float32> var_144;
    const wp::float32 var_145 = 1e-05;
    const wp::int32 var_146 = 0;
    wp::float32 var_147;
    wp::float32 var_148;
    const wp::int32 var_149 = 0;
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
    wp::vec_t<5, wp::float32> var_161;
    //---------
    // forward
    // def _mix_flex_contact_params(                                                          <L 280>
    // gap = a_gap + b_gap                                                                    <L 298>
    var_0 = wp::add(var_a_gap, var_b_gap);
    // if a_priority > b_priority:                                                            <L 300>
    var_1 = (var_a_priority > var_b_priority);
    if (var_1) {
        // condim = a_condim                                                                  <L 301>
        var_2 = wp::copy(var_a_condim);
        // solref = a_solref                                                                  <L 302>
        var_3 = wp::copy(var_a_solref);
        // solimp = a_solimp                                                                  <L 303>
        var_4 = wp::copy(var_a_solimp);
        // fri = a_friction                                                                   <L 304>
        var_5 = wp::copy(var_a_friction);
    }
    if (!var_1) {
        // elif a_priority < b_priority:                                                      <L 305>
        var_6 = (var_a_priority < var_b_priority);
        if (var_6) {
            // condim = b_condim                                                              <L 306>
            var_7 = wp::copy(var_b_condim);
            // solref = b_solref                                                              <L 307>
            var_8 = wp::copy(var_b_solref);
            // solimp = b_solimp                                                              <L 308>
            var_9 = wp::copy(var_b_solimp);
            // fri = b_friction                                                               <L 309>
            var_10 = wp::copy(var_b_friction);
        }
        var_11 = wp::where(var_6, var_7, var_2);
        var_12 = wp::where(var_6, var_8, var_3);
        var_13 = wp::where(var_6, var_9, var_4);
        var_14 = wp::where(var_6, var_10, var_5);
        if (!var_6) {
            // condim = wp.max(a_condim, b_condim)                                            <L 312>
            var_15 = wp::max(var_a_condim, var_b_condim);
            // if a_solmix >= MJ_MINVAL and b_solmix >= MJ_MINVAL:                            <L 315>
            var_18 = (var_a_solmix >= var_17);
            var_16 = var_18;
            if (var_16) {
                var_19 = (var_b_solmix >= var_17);
                var_16 = var_16 && var_19;
            }
            if (var_16) {
                // mix = a_solmix / (a_solmix + b_solmix)                                     <L 316>
                var_20 = wp::add(var_a_solmix, var_b_solmix);
                var_21 = wp::div(var_a_solmix, var_20);
            }
            if (!var_16) {
                // elif a_solmix < MJ_MINVAL and b_solmix < MJ_MINVAL:                        <L 317>
                var_23 = (var_a_solmix < var_17);
                var_22 = var_23;
                if (var_22) {
                    var_24 = (var_b_solmix < var_17);
                    var_22 = var_22 && var_24;
                }
                if (var_22) {
                    // mix = 0.5                                                              <L 318>
                }
                var_26 = wp::where(var_22, var_25, var_21);
                if (!var_22) {
                    // elif a_solmix < MJ_MINVAL:                                             <L 319>
                    var_27 = (var_a_solmix < var_17);
                    if (var_27) {
                        // mix = 0.0                                                          <L 320>
                    }
                    var_29 = wp::where(var_27, var_28, var_26);
                    if (!var_27) {
                        // mix = 1.0                                                          <L 322>
                    }
                    var_31 = wp::where(var_27, var_29, var_30);
                }
                var_32 = wp::where(var_22, var_26, var_31);
            }
            var_33 = wp::where(var_16, var_21, var_32);
            // if a_solref[0] > 0.0 and b_solref[0] > 0.0:                                    <L 325>
            var_36 = wp::extract(var_a_solref, var_35);
            var_38 = (var_36 > var_37);
            var_34 = var_38;
            if (var_34) {
                var_40 = wp::extract(var_b_solref, var_39);
                var_42 = (var_40 > var_41);
                var_34 = var_34 && var_42;
            }
            if (var_34) {
                // solref = wp.vec2(                                                          <L 326>
                // mix * a_solref[0] + (1.0 - mix) * b_solref[0],                             <L 327>
                var_44 = wp::extract(var_a_solref, var_43);
                var_45 = wp::mul(var_33, var_44);
                var_47 = wp::sub(var_46, var_33);
                var_49 = wp::extract(var_b_solref, var_48);
                var_50 = wp::mul(var_47, var_49);
                var_51 = wp::add(var_45, var_50);
                // mix * a_solref[1] + (1.0 - mix) * b_solref[1],                             <L 328>
                var_53 = wp::extract(var_a_solref, var_52);
                var_54 = wp::mul(var_33, var_53);
                var_56 = wp::sub(var_55, var_33);
                var_58 = wp::extract(var_b_solref, var_57);
                var_59 = wp::mul(var_56, var_58);
                var_60 = wp::add(var_54, var_59);
                var_61 = wp::vec_t<2, wp::float32>(var_51, var_60);
            }
            var_62 = wp::where(var_34, var_61, var_12);
            if (!var_34) {
                // solref = wp.vec2(                                                          <L 331>
                // wp.min(a_solref[0], b_solref[0]),                                          <L 332>
                var_64 = wp::extract(var_a_solref, var_63);
                var_66 = wp::extract(var_b_solref, var_65);
                var_67 = wp::min(var_64, var_66);
                // wp.min(a_solref[1], b_solref[1]),                                          <L 333>
                var_69 = wp::extract(var_a_solref, var_68);
                var_71 = wp::extract(var_b_solref, var_70);
                var_72 = wp::min(var_69, var_71);
                var_73 = wp::vec_t<2, wp::float32>(var_67, var_72);
            }
            var_74 = wp::where(var_34, var_62, var_73);
            // solimp = vec5(                                                                 <L 337>
            // mix * a_solimp[0] + (1.0 - mix) * b_solimp[0],                                 <L 338>
            var_76 = wp::extract(var_a_solimp, var_75);
            var_77 = wp::mul(var_33, var_76);
            var_79 = wp::sub(var_78, var_33);
            var_81 = wp::extract(var_b_solimp, var_80);
            var_82 = wp::mul(var_79, var_81);
            var_83 = wp::add(var_77, var_82);
            // mix * a_solimp[1] + (1.0 - mix) * b_solimp[1],                                 <L 339>
            var_85 = wp::extract(var_a_solimp, var_84);
            var_86 = wp::mul(var_33, var_85);
            var_88 = wp::sub(var_87, var_33);
            var_90 = wp::extract(var_b_solimp, var_89);
            var_91 = wp::mul(var_88, var_90);
            var_92 = wp::add(var_86, var_91);
            // mix * a_solimp[2] + (1.0 - mix) * b_solimp[2],                                 <L 340>
            var_94 = wp::extract(var_a_solimp, var_93);
            var_95 = wp::mul(var_33, var_94);
            var_97 = wp::sub(var_96, var_33);
            var_99 = wp::extract(var_b_solimp, var_98);
            var_100 = wp::mul(var_97, var_99);
            var_101 = wp::add(var_95, var_100);
            // mix * a_solimp[3] + (1.0 - mix) * b_solimp[3],                                 <L 341>
            var_103 = wp::extract(var_a_solimp, var_102);
            var_104 = wp::mul(var_33, var_103);
            var_106 = wp::sub(var_105, var_33);
            var_108 = wp::extract(var_b_solimp, var_107);
            var_109 = wp::mul(var_106, var_108);
            var_110 = wp::add(var_104, var_109);
            // mix * a_solimp[4] + (1.0 - mix) * b_solimp[4],                                 <L 342>
            var_112 = wp::extract(var_a_solimp, var_111);
            var_113 = wp::mul(var_33, var_112);
            var_115 = wp::sub(var_114, var_33);
            var_117 = wp::extract(var_b_solimp, var_116);
            var_118 = wp::mul(var_115, var_117);
            var_119 = wp::add(var_113, var_118);
            var_120 = wp::vec_t<5, wp::float32>({var_83, var_92, var_101, var_110, var_119});
            // fri = wp.vec3(                                                                 <L 346>
            // wp.max(a_friction[0], b_friction[0]),                                          <L 347>
            var_122 = wp::extract(var_a_friction, var_121);
            var_124 = wp::extract(var_b_friction, var_123);
            var_125 = wp::max(var_122, var_124);
            // wp.max(a_friction[1], b_friction[1]),                                          <L 348>
            var_127 = wp::extract(var_a_friction, var_126);
            var_129 = wp::extract(var_b_friction, var_128);
            var_130 = wp::max(var_127, var_129);
            // wp.max(a_friction[2], b_friction[2]),                                          <L 349>
            var_132 = wp::extract(var_a_friction, var_131);
            var_134 = wp::extract(var_b_friction, var_133);
            var_135 = wp::max(var_132, var_134);
            var_136 = wp::vec_t<3, wp::float32>(var_125, var_130, var_135);
        }
        var_137 = wp::where(var_6, var_11, var_15);
        var_138 = wp::where(var_6, var_12, var_74);
        var_139 = wp::where(var_6, var_13, var_120);
        var_140 = wp::where(var_6, var_14, var_136);
    }
    var_141 = wp::where(var_1, var_2, var_137);
    var_142 = wp::where(var_1, var_3, var_138);
    var_143 = wp::where(var_1, var_4, var_139);
    var_144 = wp::where(var_1, var_5, var_140);
    // friction = vec5(                                                                       <L 353>
    // wp.max(MJ_MINMU, fri[0]),                                                              <L 354>
    var_147 = wp::extract(var_144, var_146);
    var_148 = wp::max(var_145, var_147);
    // wp.max(MJ_MINMU, fri[0]),                                                              <L 355>
    var_150 = wp::extract(var_144, var_149);
    var_151 = wp::max(var_145, var_150);
    // wp.max(MJ_MINMU, fri[1]),                                                              <L 356>
    var_153 = wp::extract(var_144, var_152);
    var_154 = wp::max(var_145, var_153);
    // wp.max(MJ_MINMU, fri[2]),                                                              <L 357>
    var_156 = wp::extract(var_144, var_155);
    var_157 = wp::max(var_145, var_156);
    // wp.max(MJ_MINMU, fri[2]),                                                              <L 358>
    var_159 = wp::extract(var_144, var_158);
    var_160 = wp::max(var_145, var_159);
    var_161 = wp::vec_t<5, wp::float32>({var_148, var_151, var_154, var_157, var_160});
    // return condim, gap, solref, solimp, friction                                           <L 361>
    ret_0 = var_141;
    ret_1 = var_0;
    ret_2 = var_142;
    ret_3 = var_143;
    ret_4 = var_161;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:202
static CUDA_CALLABLE void orthogonals_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    const wp::float32 var_1 = 1.0;
    const wp::float32 var_2 = 0.0;
    wp::vec_t<3, wp::float32> var_3;
    const wp::float32 var_4 = 0.0;
    const wp::float32 var_5 = 0.0;
    const wp::float32 var_6 = 1.0;
    wp::vec_t<3, wp::float32> var_7;
    bool var_8;
    const wp::float32 var_9 = -0.5;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    bool var_12;
    const wp::int32 var_13 = 1;
    wp::float32 var_14;
    const wp::float32 var_15 = 0.5;
    bool var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    const wp::float32 var_25 = 0.0;
    const wp::float32 var_26 = 0.0;
    const wp::float32 var_27 = 0.0;
    wp::vec_t<3, wp::float32> var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    //---------
    // forward
    // def orthogonals(a: wp.vec3):                                                           <L 203>
    // y = wp.vec3(0.0, 1.0, 0.0)                                                             <L 204>
    var_3 = wp::vec_t<3, wp::float32>(var_0, var_1, var_2);
    // z = wp.vec3(0.0, 0.0, 1.0)                                                             <L 205>
    var_7 = wp::vec_t<3, wp::float32>(var_4, var_5, var_6);
    // b = wp.where((-0.5 < a[1]) and (a[1] < 0.5), y, z)                                     <L 206>
    var_11 = wp::extract(var_a, var_10);
    var_12 = (var_9 < var_11);
    var_8 = var_12;
    if (var_8) {
        var_14 = wp::extract(var_a, var_13);
        var_16 = (var_14 < var_15);
        var_8 = var_8 && var_16;
    }
    var_17 = wp::where(var_8, var_3, var_7);
    // b = b - a * wp.dot(a, b)                                                               <L 207>
    var_18 = wp::dot(var_a, var_17);
    var_19 = wp::mul(var_a, var_18);
    var_20 = wp::sub(var_17, var_19);
    // b = wp.normalize(b)                                                                    <L 208>
    var_21 = wp::normalize(var_20);
    // if wp.length(a) == 0.0:                                                                <L 209>
    var_22 = wp::length(var_a);
    var_24 = (var_22 == var_23);
    if (var_24) {
        // b = wp.vec3(0.0, 0.0, 0.0)                                                         <L 210>
        var_28 = wp::vec_t<3, wp::float32>(var_25, var_26, var_27);
    }
    var_29 = wp::where(var_24, var_28, var_21);
    // c = wp.cross(a, b)                                                                     <L 211>
    var_30 = wp::cross(var_a, var_29);
    // return b, c                                                                            <L 213>
    ret_0 = var_29;
    ret_1 = var_30;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:246
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> make_frame_0(
    wp::vec_t<3, wp::float32> var_a)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 1;
    wp::float32 var_6;
    const wp::int32 var_7 = 2;
    wp::float32 var_8;
    const wp::int32 var_9 = 0;
    wp::float32 var_10;
    const wp::int32 var_11 = 1;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    const wp::int32 var_17 = 1;
    wp::float32 var_18;
    const wp::int32 var_19 = 2;
    wp::float32 var_20;
    wp::mat_t<3, 3, wp::float32> var_21;
    //---------
    // forward
    // def make_frame(a: wp.vec3):                                                            <L 247>
    // a = wp.normalize(a)                                                                    <L 248>
    var_0 = wp::normalize(var_a);
    // b, c = orthogonals(a)                                                                  <L 249>
    orthogonals_0(var_0, var_1, var_2);
    // return wp.mat33(                                                                       <L 252>
    // a.x, a.y, a.z,                                                                         <L 253>
    var_4 = wp::extract(var_0, var_3);
    var_6 = wp::extract(var_0, var_5);
    var_8 = wp::extract(var_0, var_7);
    // b.x, b.y, b.z,                                                                         <L 254>
    var_10 = wp::extract(var_1, var_9);
    var_12 = wp::extract(var_1, var_11);
    var_14 = wp::extract(var_1, var_13);
    // c.x, c.y, c.z                                                                          <L 255>
    var_16 = wp::extract(var_2, var_15);
    var_18 = wp::extract(var_2, var_17);
    var_20 = wp::extract(var_2, var_19);
    var_21 = wp::mat_t<3, 3, wp::float32>(var_4, var_6, var_8, var_10, var_12, var_14, var_16, var_18, var_20);
    return var_21;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_flex.py:279
static CUDA_CALLABLE void adj__mix_flex_contact_params_0(
    wp::int32 var_a_condim,
    wp::int32 var_a_priority,
    wp::float32 var_a_solmix,
    wp::vec_t<2, wp::float32> var_a_solref,
    wp::vec_t<5, wp::float32> var_a_solimp,
    wp::vec_t<3, wp::float32> var_a_friction,
    wp::float32 var_a_gap,
    wp::int32 var_b_condim,
    wp::int32 var_b_priority,
    wp::float32 var_b_solmix,
    wp::vec_t<2, wp::float32> var_b_solref,
    wp::vec_t<5, wp::float32> var_b_solimp,
    wp::vec_t<3, wp::float32> var_b_friction,
    wp::float32 var_b_gap,
    wp::int32 & ret_0,
    wp::float32 & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<5, wp::float32> & ret_3,
    wp::vec_t<5, wp::float32> & ret_4,
    wp::int32 & adj_a_condim,
    wp::int32 & adj_a_priority,
    wp::float32 & adj_a_solmix,
    wp::vec_t<2, wp::float32> & adj_a_solref,
    wp::vec_t<5, wp::float32> & adj_a_solimp,
    wp::vec_t<3, wp::float32> & adj_a_friction,
    wp::float32 & adj_a_gap,
    wp::int32 & adj_b_condim,
    wp::int32 & adj_b_priority,
    wp::float32 & adj_b_solmix,
    wp::vec_t<2, wp::float32> & adj_b_solref,
    wp::vec_t<5, wp::float32> & adj_b_solimp,
    wp::vec_t<3, wp::float32> & adj_b_friction,
    wp::float32 & adj_b_gap,
    wp::int32 & adj_ret_0,
    wp::float32 & adj_ret_1,
    wp::vec_t<2, wp::float32> & adj_ret_2,
    wp::vec_t<5, wp::float32> & adj_ret_3,
    wp::vec_t<5, wp::float32> & adj_ret_4)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:202
static CUDA_CALLABLE void adj_orthogonals_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_a,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    const wp::float32 var_1 = 1.0;
    const wp::float32 var_2 = 0.0;
    wp::vec_t<3, wp::float32> var_3;
    const wp::float32 var_4 = 0.0;
    const wp::float32 var_5 = 0.0;
    const wp::float32 var_6 = 1.0;
    wp::vec_t<3, wp::float32> var_7;
    bool var_8;
    const wp::float32 var_9 = -0.5;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    bool var_12;
    const wp::int32 var_13 = 1;
    wp::float32 var_14;
    const wp::float32 var_15 = 0.5;
    bool var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    const wp::float32 var_25 = 0.0;
    const wp::float32 var_26 = 0.0;
    const wp::float32 var_27 = 0.0;
    wp::vec_t<3, wp::float32> var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    //---------
    // dual vars
    wp::float32 adj_0 = {};
    wp::float32 adj_1 = {};
    wp::float32 adj_2 = {};
    wp::vec_t<3, wp::float32> adj_3 = {};
    wp::float32 adj_4 = {};
    wp::float32 adj_5 = {};
    wp::float32 adj_6 = {};
    wp::vec_t<3, wp::float32> adj_7 = {};
    bool adj_8 = {};
    wp::float32 adj_9 = {};
    wp::int32 adj_10 = {};
    wp::float32 adj_11 = {};
    bool adj_12 = {};
    wp::int32 adj_13 = {};
    wp::float32 adj_14 = {};
    wp::float32 adj_15 = {};
    bool adj_16 = {};
    wp::vec_t<3, wp::float32> adj_17 = {};
    wp::float32 adj_18 = {};
    wp::vec_t<3, wp::float32> adj_19 = {};
    wp::vec_t<3, wp::float32> adj_20 = {};
    wp::vec_t<3, wp::float32> adj_21 = {};
    wp::float32 adj_22 = {};
    wp::float32 adj_23 = {};
    bool adj_24 = {};
    wp::float32 adj_25 = {};
    wp::float32 adj_26 = {};
    wp::float32 adj_27 = {};
    wp::vec_t<3, wp::float32> adj_28 = {};
    wp::vec_t<3, wp::float32> adj_29 = {};
    wp::vec_t<3, wp::float32> adj_30 = {};
    //---------
    // forward
    // def orthogonals(a: wp.vec3):                                                           <L 203>
    // y = wp.vec3(0.0, 1.0, 0.0)                                                             <L 204>
    var_3 = wp::vec_t<3, wp::float32>(var_0, var_1, var_2);
    // z = wp.vec3(0.0, 0.0, 1.0)                                                             <L 205>
    var_7 = wp::vec_t<3, wp::float32>(var_4, var_5, var_6);
    // b = wp.where((-0.5 < a[1]) and (a[1] < 0.5), y, z)                                     <L 206>
    var_11 = wp::extract(var_a, var_10);
    var_12 = (var_9 < var_11);
    var_8 = var_12;
    if (var_8) {
        var_14 = wp::extract(var_a, var_13);
        var_16 = (var_14 < var_15);
        var_8 = var_8 && var_16;
    }
    var_17 = wp::where(var_8, var_3, var_7);
    // b = b - a * wp.dot(a, b)                                                               <L 207>
    var_18 = wp::dot(var_a, var_17);
    var_19 = wp::mul(var_a, var_18);
    var_20 = wp::sub(var_17, var_19);
    // b = wp.normalize(b)                                                                    <L 208>
    var_21 = wp::normalize(var_20);
    // if wp.length(a) == 0.0:                                                                <L 209>
    var_22 = wp::length(var_a);
    var_24 = (var_22 == var_23);
    if (var_24) {
        // b = wp.vec3(0.0, 0.0, 0.0)                                                         <L 210>
        var_28 = wp::vec_t<3, wp::float32>(var_25, var_26, var_27);
    }
    var_29 = wp::where(var_24, var_28, var_21);
    // c = wp.cross(a, b)                                                                     <L 211>
    var_30 = wp::cross(var_a, var_29);
    // return b, c                                                                            <L 213>
    ret_0 = var_29;
    ret_1 = var_30;
    goto label0;
    //---------
    // reverse
    label0:;
    adj_30 += adj_ret_1;
    adj_29 += adj_ret_0;
    // adj: return b, c                                                                       <L 213>
    wp::adj_cross(var_a, var_29, adj_a, adj_29, adj_30);
    // adj: c = wp.cross(a, b)                                                                <L 211>
    wp::adj_where(var_24, var_28, var_21, adj_24, adj_28, adj_21, adj_29);
    if (var_24) {
        wp::adj_vec_t(var_25, var_26, var_27, adj_25, adj_26, adj_27, adj_28);
        // adj: b = wp.vec3(0.0, 0.0, 0.0)                                                    <L 210>
    }
    wp::adj_length(var_a, var_22, adj_a, adj_22);
    // adj: if wp.length(a) == 0.0:                                                           <L 209>
    wp::adj_normalize(var_20, var_21, adj_20, adj_21);
    // adj: b = wp.normalize(b)                                                               <L 208>
    wp::adj_sub(var_17, var_19, adj_17, adj_19, adj_20);
    wp::adj_mul(var_a, var_18, adj_a, adj_18, adj_19);
    wp::adj_dot(var_a, var_17, adj_a, adj_17, adj_18);
    // adj: b = b - a * wp.dot(a, b)                                                          <L 207>
    wp::adj_where(var_8, var_3, var_7, adj_8, adj_3, adj_7, adj_17);
    if (var_8) {
        wp::adj_extract(var_a, var_13, adj_a, adj_13, adj_14);
    }
    wp::adj_extract(var_a, var_10, adj_a, adj_10, adj_11);
    // adj: b = wp.where((-0.5 < a[1]) and (a[1] < 0.5), y, z)                                <L 206>
    wp::adj_vec_t(var_4, var_5, var_6, adj_4, adj_5, adj_6, adj_7);
    // adj: z = wp.vec3(0.0, 0.0, 1.0)                                                        <L 205>
    wp::adj_vec_t(var_0, var_1, var_2, adj_0, adj_1, adj_2, adj_3);
    // adj: y = wp.vec3(0.0, 1.0, 0.0)                                                        <L 204>
    // adj: def orthogonals(a: wp.vec3):                                                      <L 203>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:246
static CUDA_CALLABLE void adj_make_frame_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> & adj_a,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 1;
    wp::float32 var_6;
    const wp::int32 var_7 = 2;
    wp::float32 var_8;
    const wp::int32 var_9 = 0;
    wp::float32 var_10;
    const wp::int32 var_11 = 1;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    const wp::int32 var_17 = 1;
    wp::float32 var_18;
    const wp::int32 var_19 = 2;
    wp::float32 var_20;
    wp::mat_t<3, 3, wp::float32> var_21;
    //---------
    // dual vars
    wp::vec_t<3, wp::float32> adj_0 = {};
    wp::vec_t<3, wp::float32> adj_1 = {};
    wp::vec_t<3, wp::float32> adj_2 = {};
    wp::int32 adj_3 = {};
    wp::float32 adj_4 = {};
    wp::int32 adj_5 = {};
    wp::float32 adj_6 = {};
    wp::int32 adj_7 = {};
    wp::float32 adj_8 = {};
    wp::int32 adj_9 = {};
    wp::float32 adj_10 = {};
    wp::int32 adj_11 = {};
    wp::float32 adj_12 = {};
    wp::int32 adj_13 = {};
    wp::float32 adj_14 = {};
    wp::int32 adj_15 = {};
    wp::float32 adj_16 = {};
    wp::int32 adj_17 = {};
    wp::float32 adj_18 = {};
    wp::int32 adj_19 = {};
    wp::float32 adj_20 = {};
    wp::mat_t<3, 3, wp::float32> adj_21 = {};
    //---------
    // forward
    // def make_frame(a: wp.vec3):                                                            <L 247>
    // a = wp.normalize(a)                                                                    <L 248>
    var_0 = wp::normalize(var_a);
    // b, c = orthogonals(a)                                                                  <L 249>
    orthogonals_0(var_0, var_1, var_2);
    // return wp.mat33(                                                                       <L 252>
    // a.x, a.y, a.z,                                                                         <L 253>
    var_4 = wp::extract(var_0, var_3);
    var_6 = wp::extract(var_0, var_5);
    var_8 = wp::extract(var_0, var_7);
    // b.x, b.y, b.z,                                                                         <L 254>
    var_10 = wp::extract(var_1, var_9);
    var_12 = wp::extract(var_1, var_11);
    var_14 = wp::extract(var_1, var_13);
    // c.x, c.y, c.z                                                                          <L 255>
    var_16 = wp::extract(var_2, var_15);
    var_18 = wp::extract(var_2, var_17);
    var_20 = wp::extract(var_2, var_19);
    var_21 = wp::mat_t<3, 3, wp::float32>(var_4, var_6, var_8, var_10, var_12, var_14, var_16, var_18, var_20);
    goto label0;
    //---------
    // reverse
    label0:;
    adj_21 += adj_ret;
    wp::adj_mat_t(var_4, var_6, var_8, var_10, var_12, var_14, var_16, var_18, var_20, adj_4, adj_6, adj_8, adj_10, adj_12, adj_14, adj_16, adj_18, adj_20, adj_21);
    wp::adj_extract(var_2, var_19, adj_2, adj_19, adj_20);
    wp::adj_extract(var_2, var_17, adj_2, adj_17, adj_18);
    wp::adj_extract(var_2, var_15, adj_2, adj_15, adj_16);
    // adj: c.x, c.y, c.z                                                                     <L 255>
    wp::adj_extract(var_1, var_13, adj_1, adj_13, adj_14);
    wp::adj_extract(var_1, var_11, adj_1, adj_11, adj_12);
    wp::adj_extract(var_1, var_9, adj_1, adj_9, adj_10);
    // adj: b.x, b.y, b.z,                                                                    <L 254>
    wp::adj_extract(var_0, var_7, adj_0, adj_7, adj_8);
    wp::adj_extract(var_0, var_5, adj_0, adj_5, adj_6);
    wp::adj_extract(var_0, var_3, adj_0, adj_3, adj_4);
    // adj: a.x, a.y, a.z,                                                                    <L 253>
    // adj: return wp.mat33(                                                                  <L 252>
    adj_orthogonals_0(var_0, var_1, var_2, adj_0, adj_1, adj_2);
    // adj: b, c = orthogonals(a)                                                             <L 249>
    wp::adj_normalize(var_a, var_0, adj_a, adj_0);
    // adj: a = wp.normalize(a)                                                               <L 248>
    // adj: def make_frame(a: wp.vec3):                                                       <L 247>
    return;
}



extern "C" __global__ void _write_filtered_contacts__locals__kernel_1fa09624_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::int32 var_opt_integrator,
    wp::array_t<wp::int32> var_body_weldid,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_bodyid,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::int32> var_flex_condim,
    wp::array_t<wp::int32> var_flex_priority,
    wp::array_t<wp::float32> var_flex_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_flex_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_flex_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_flex_friction,
    wp::array_t<wp::float32> var_flex_margin,
    wp::array_t<wp::float32> var_flex_gap,
    wp::array_t<wp::int32> var_flex_passive,
    wp::array_t<wp::int32> var_flex_dim,
    wp::array_t<wp::int32> var_flex_interp,
    wp::int32 var_naconmax_in,
    wp::array_t<wp::int32> var_ncand,
    wp::array_t<wp::float32> var_cand_dist,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_pos,
    wp::array_t<wp::vec_t<3, wp::float32>> var_cand_nrm,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_geom,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_flex,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_elem,
    wp::array_t<wp::vec_t<2, wp::int32>> var_cand_vert,
    wp::array_t<wp::int32> var_cand_worldid,
    wp::array_t<wp::int32> var_cand_active,
    wp::array_t<wp::float32> var_contact_dist_out,
    wp::array_t<wp::vec_t<3, wp::float32>> var_contact_pos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_contact_frame_out,
    wp::array_t<wp::float32> var_contact_includemargin_out,
    wp::array_t<wp::vec_t<5, wp::float32>> var_contact_friction_out,
    wp::array_t<wp::vec_t<2, wp::float32>> var_contact_solref_out,
    wp::array_t<wp::vec_t<2, wp::float32>> var_contact_solreffriction_out,
    wp::array_t<wp::vec_t<5, wp::float32>> var_contact_solimp_out,
    wp::array_t<wp::int32> var_contact_dim_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_geom_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_flex_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_elem_out,
    wp::array_t<wp::vec_t<2, wp::int32>> var_contact_vert_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::int32> var_nacon_out,
    wp::array_t<wp::int32> var_overflow_out)
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
        const wp::int32 var_6 = 0;
        bool var_7;
        wp::int32 var_8;
        wp::vec_t<2, wp::int32>* var_9;
        const wp::int32 var_10 = 0;
        wp::int32 var_11;
        wp::vec_t<2, wp::int32> var_12;
        wp::int32* var_13;
        wp::int32 var_14;
        wp::int32 var_15;
        const wp::int32 var_16 = 0;
        wp::int32 var_17;
        const wp::float32 var_18 = 0.0;
        wp::float32 var_19;
        const wp::float32 var_20 = 0.0;
        wp::float32 var_21;
        const wp::float32 var_22 = 0.0;
        const wp::float32 var_23 = 0.0;
        wp::vec_t<2, wp::float32> var_24;
        const wp::float32 var_25 = 0.0;
        const wp::float32 var_26 = 0.0;
        const wp::float32 var_27 = 0.0;
        const wp::float32 var_28 = 0.0;
        const wp::float32 var_29 = 0.0;
        wp::vec_t<5, wp::float32> var_30;
        const wp::float32 var_31 = 0.0;
        const wp::float32 var_32 = 0.0;
        const wp::float32 var_33 = 0.0;
        const wp::float32 var_34 = 0.0;
        const wp::float32 var_35 = 0.0;
        wp::vec_t<5, wp::float32> var_36;
        const wp::int32 var_37 = 0;
        bool var_38;
        wp::vec_t<2, wp::int32>* var_39;
        const wp::int32 var_40 = 1;
        wp::int32 var_41;
        wp::vec_t<2, wp::int32> var_42;
        wp::shape_t* var_43;
        const wp::int32 var_44 = 0;
        wp::int32 var_45;
        wp::shape_t var_46;
        wp::int32 var_47;
        wp::float32* var_48;
        wp::float32 var_49;
        wp::float32 var_50;
        wp::float32* var_51;
        wp::float32 var_52;
        wp::float32 var_53;
        wp::float32 var_54;
        wp::int32* var_55;
        wp::int32* var_56;
        wp::shape_t* var_57;
        const wp::int32 var_58 = 0;
        wp::int32 var_59;
        wp::shape_t var_60;
        wp::int32 var_61;
        wp::float32* var_62;
        wp::shape_t* var_63;
        const wp::int32 var_64 = 0;
        wp::int32 var_65;
        wp::shape_t var_66;
        wp::int32 var_67;
        wp::vec_t<2, wp::float32>* var_68;
        wp::shape_t* var_69;
        const wp::int32 var_70 = 0;
        wp::int32 var_71;
        wp::shape_t var_72;
        wp::int32 var_73;
        wp::vec_t<5, wp::float32>* var_74;
        wp::shape_t* var_75;
        const wp::int32 var_76 = 0;
        wp::int32 var_77;
        wp::shape_t var_78;
        wp::int32 var_79;
        wp::vec_t<3, wp::float32>* var_80;
        wp::shape_t* var_81;
        const wp::int32 var_82 = 0;
        wp::int32 var_83;
        wp::shape_t var_84;
        wp::int32 var_85;
        wp::float32* var_86;
        wp::int32* var_87;
        wp::int32* var_88;
        wp::float32* var_89;
        wp::vec_t<2, wp::float32>* var_90;
        wp::vec_t<5, wp::float32>* var_91;
        wp::vec_t<3, wp::float32>* var_92;
        wp::float32* var_93;
        wp::int32 var_94;
        wp::float32 var_95;
        wp::vec_t<2, wp::float32> var_96;
        wp::vec_t<5, wp::float32> var_97;
        wp::vec_t<5, wp::float32> var_98;
        wp::int32 var_99;
        wp::int32 var_100;
        wp::float32 var_101;
        wp::vec_t<2, wp::float32> var_102;
        wp::vec_t<5, wp::float32> var_103;
        wp::vec_t<3, wp::float32> var_104;
        wp::float32 var_105;
        wp::int32 var_106;
        wp::int32 var_107;
        wp::float32 var_108;
        wp::vec_t<2, wp::float32> var_109;
        wp::vec_t<5, wp::float32> var_110;
        wp::vec_t<3, wp::float32> var_111;
        wp::float32 var_112;
        wp::int32 var_113;
        wp::float32 var_114;
        wp::float32 var_115;
        wp::vec_t<2, wp::float32> var_116;
        wp::vec_t<5, wp::float32> var_117;
        wp::vec_t<5, wp::float32> var_118;
        wp::vec_t<2, wp::int32>* var_119;
        const wp::int32 var_120 = 0;
        wp::int32 var_121;
        wp::vec_t<2, wp::int32> var_122;
        wp::vec_t<2, wp::int32>* var_123;
        const wp::int32 var_124 = 1;
        wp::int32 var_125;
        wp::vec_t<2, wp::int32> var_126;
        bool var_127;
        const wp::float32 var_128 = 0.0;
        wp::float32 var_129;
        const wp::float32 var_130 = 0.0;
        wp::float32 var_131;
        const wp::float32 var_132 = 0.0;
        wp::float32 var_133;
        wp::float32 var_134;
        wp::float32* var_135;
        wp::float32* var_136;
        wp::float32 var_137;
        wp::float32 var_138;
        wp::float32 var_139;
        wp::float32* var_140;
        wp::float32 var_141;
        wp::float32 var_142;
        wp::float32* var_143;
        wp::float32 var_144;
        wp::float32 var_145;
        wp::float32 var_146;
        wp::float32 var_147;
        wp::float32 var_148;
        wp::int32* var_149;
        wp::int32* var_150;
        wp::float32* var_151;
        wp::vec_t<2, wp::float32>* var_152;
        wp::vec_t<5, wp::float32>* var_153;
        wp::vec_t<3, wp::float32>* var_154;
        wp::int32* var_155;
        wp::int32* var_156;
        wp::float32* var_157;
        wp::vec_t<2, wp::float32>* var_158;
        wp::vec_t<5, wp::float32>* var_159;
        wp::vec_t<3, wp::float32>* var_160;
        wp::int32 var_161;
        wp::float32 var_162;
        wp::vec_t<2, wp::float32> var_163;
        wp::vec_t<5, wp::float32> var_164;
        wp::vec_t<5, wp::float32> var_165;
        wp::int32 var_166;
        wp::int32 var_167;
        wp::float32 var_168;
        wp::vec_t<2, wp::float32> var_169;
        wp::vec_t<5, wp::float32> var_170;
        wp::vec_t<3, wp::float32> var_171;
        wp::int32 var_172;
        wp::int32 var_173;
        wp::float32 var_174;
        wp::vec_t<2, wp::float32> var_175;
        wp::vec_t<5, wp::float32> var_176;
        wp::vec_t<3, wp::float32> var_177;
        bool var_178;
        wp::vec_t<2, wp::int32>* var_179;
        const wp::int32 var_180 = 0;
        wp::int32 var_181;
        wp::vec_t<2, wp::int32> var_182;
        const wp::int32 var_183 = 0;
        bool var_184;
        wp::int32* var_185;
        const wp::int32 var_186 = 3;
        bool var_187;
        wp::int32 var_188;
        const wp::int32 var_189 = 1;
        wp::int32 var_190;
        wp::int32 var_191;
        wp::int32 var_192;
        wp::int32 var_193;
        wp::float32 var_194;
        wp::float32 var_195;
        wp::vec_t<2, wp::float32> var_196;
        wp::vec_t<5, wp::float32> var_197;
        wp::vec_t<5, wp::float32> var_198;
        wp::float32* var_199;
        bool var_200;
        wp::float32 var_201;
        const wp::int32 var_202 = 0;
        const wp::int32 var_203 = 1;
        wp::int32 var_204;
        bool var_205;
        const bool var_206 = true;
        const wp::str var_207 = "flex contact overflow - please increase naconmax beyond %u\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.NARROWPHASE (or = 0 for all)\n";
        const wp::int32 var_208 = 8;
        wp::int32 var_209;
        wp::float32* var_210;
        wp::float32 var_211;
        wp::vec_t<3, wp::float32>* var_212;
        wp::vec_t<3, wp::float32> var_213;
        wp::vec_t<3, wp::float32>* var_214;
        wp::mat_t<3, 3, wp::float32> var_215;
        wp::vec_t<3, wp::float32> var_216;
        bool var_217;
        const wp::int32 var_218 = 0;
        bool var_219;
        wp::int32* var_220;
        const wp::int32 var_221 = 0;
        const wp::int32 var_222 = 0;
        wp::int32 var_223;
        bool var_224;
        wp::int32 var_225;
        wp::float32 var_226;
        wp::vec_t<2, wp::int32>* var_227;
        wp::vec_t<2, wp::int32> var_228;
        wp::vec_t<2, wp::int32>* var_229;
        wp::vec_t<2, wp::int32> var_230;
        wp::vec_t<2, wp::int32>* var_231;
        wp::vec_t<2, wp::int32> var_232;
        wp::vec_t<2, wp::int32>* var_233;
        wp::vec_t<2, wp::int32> var_234;
        wp::int32* var_235;
        wp::int32 var_236;
        wp::vec_t<2, wp::int32>* var_237;
        const wp::int32 var_238 = 0;
        wp::int32 var_239;
        wp::vec_t<2, wp::int32> var_240;
        wp::vec_t<2, wp::int32>* var_241;
        const wp::int32 var_242 = 1;
        wp::int32 var_243;
        wp::vec_t<2, wp::int32> var_244;
        wp::vec_t<2, wp::int32>* var_245;
        const wp::int32 var_246 = 0;
        wp::int32 var_247;
        wp::vec_t<2, wp::int32> var_248;
        wp::vec_t<2, wp::int32>* var_249;
        const wp::int32 var_250 = 1;
        wp::int32 var_251;
        wp::vec_t<2, wp::int32> var_252;
        bool var_253;
        bool var_254;
        bool var_255;
        const wp::int32 var_256 = 0;
        bool var_257;
        wp::int32* var_258;
        const wp::int32 var_259 = 0;
        bool var_260;
        wp::int32 var_261;
        wp::int32* var_262;
        const wp::int32 var_263 = 0;
        bool var_264;
        wp::int32 var_265;
        wp::int32* var_266;
        const wp::int32 var_267 = 2;
        bool var_268;
        wp::int32 var_269;
        bool var_270;
        const wp::int32 var_271 = 0;
        bool var_272;
        wp::int32* var_273;
        const wp::int32 var_274 = 0;
        bool var_275;
        wp::int32 var_276;
        wp::int32* var_277;
        const wp::int32 var_278 = 0;
        bool var_279;
        wp::int32 var_280;
        wp::int32* var_281;
        const wp::int32 var_282 = 2;
        bool var_283;
        wp::int32 var_284;
        const wp::int32 var_285 = 4;
        const wp::int32 var_286 = 4;
        wp::int32 var_287;
        bool var_288;
        bool var_289;
        bool var_290;
        const wp::int32 var_291 = 0;
        bool var_292;
        bool var_293;
        const wp::int32 var_294 = 0;
        bool var_295;
        wp::int32* var_296;
        wp::int32* var_297;
        wp::int32 var_298;
        const wp::int32 var_299 = 0;
        bool var_300;
        wp::int32 var_301;
        bool var_302;
        const wp::int32 var_303 = 0;
        bool var_304;
        bool var_305;
        const wp::int32 var_306 = 0;
        bool var_307;
        wp::int32* var_308;
        wp::int32* var_309;
        wp::int32 var_310;
        const wp::int32 var_311 = 0;
        bool var_312;
        wp::int32 var_313;
        bool var_314;
        const wp::int32 var_315 = 4;
        const wp::int32 var_316 = 4;
        wp::int32 var_317;
        const wp::int32 var_318 = 1;
        const wp::int32 var_319 = 1;
        wp::int32 var_320;
        wp::int32 var_321;
        const wp::int32 var_322 = 0;
        //---------
        // forward
        // def kernel(                                                                            <L 2387>
        // i = wp.tid()                                                                           <L 2446>
        var_0 = builtin_tid1d();
        // if i >= ncand[0]:                                                                      <L 2447>
        var_2 = wp::address(var_ncand, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 2448>
            continue;
        }
        // if cand_active[i] == 0:                                                                <L 2450>
        var_5 = wp::address(var_cand_active, var_0);
        var_8 = wp::load(var_5);
        var_7 = (var_8 == var_6);
        if (var_7) {
            // return                                                                             <L 2451>
            continue;
        }
        // geomid = cand_geom[i][0]                                                               <L 2453>
        var_9 = wp::address(var_cand_geom, var_0);
        var_12 = wp::load(var_9);
        var_11 = wp::extract(var_12, var_10);
        // worldid = cand_worldid[i]                                                              <L 2454>
        var_13 = wp::address(var_cand_worldid, var_0);
        var_15 = wp::load(var_13);
        var_14 = wp::copy(var_15);
        // condim = int(0)                                                                        <L 2456>
        var_17 = wp::int(var_16);
        // margin = float(0.0)                                                                    <L 2457>
        var_19 = wp::float(var_18);
        // gap = float(0.0)                                                                       <L 2458>
        var_21 = wp::float(var_20);
        // solref = wp.vec2(0.0, 0.0)                                                             <L 2459>
        var_24 = wp::vec_t<2, wp::float32>(var_22, var_23);
        // solimp = vec5(0.0, 0.0, 0.0, 0.0, 0.0)                                                 <L 2460>
        var_30 = wp::vec_t<5, wp::float32>({var_25, var_26, var_27, var_28, var_29});
        // friction = vec5(0.0, 0.0, 0.0, 0.0, 0.0)                                               <L 2461>
        var_36 = wp::vec_t<5, wp::float32>({var_31, var_32, var_33, var_34, var_35});
        // if geomid >= 0:                                                                        <L 2463>
        var_38 = (var_11 >= var_37);
        if (var_38) {
            // flexid = cand_flex[i][1]                                                           <L 2464>
            var_39 = wp::address(var_cand_flex, var_0);
            var_42 = wp::load(var_39);
            var_41 = wp::extract(var_42, var_40);
            // geom_margin_val = geom_margin[worldid % geom_margin.shape[0], geomid]              <L 2465>
            var_43 = &(var_geom_margin.shape);
            var_46 = wp::load(var_43);
            var_45 = wp::extract(var_46, var_44);
            var_47 = wp::mod(var_14, var_45);
            var_48 = wp::address(var_geom_margin, var_47, var_11);
            var_50 = wp::load(var_48);
            var_49 = wp::copy(var_50);
            // tri_margin = flex_margin[flexid]                                                   <L 2466>
            var_51 = wp::address(var_flex_margin, var_41);
            var_53 = wp::load(var_51);
            var_52 = wp::copy(var_53);
            // margin = geom_margin_val + tri_margin                                              <L 2467>
            var_54 = wp::add(var_49, var_52);
            // condim, gap, solref, solimp, friction = _mix_flex_contact_params(                  <L 2469>
            // geom_condim[geomid],                                                               <L 2470>
            var_55 = wp::address(var_geom_condim, var_11);
            // geom_priority[geomid],                                                             <L 2471>
            var_56 = wp::address(var_geom_priority, var_11);
            // geom_solmix[worldid % geom_solmix.shape[0], geomid],                               <L 2472>
            var_57 = &(var_geom_solmix.shape);
            var_60 = wp::load(var_57);
            var_59 = wp::extract(var_60, var_58);
            var_61 = wp::mod(var_14, var_59);
            var_62 = wp::address(var_geom_solmix, var_61, var_11);
            // geom_solref[worldid % geom_solref.shape[0], geomid],                               <L 2473>
            var_63 = &(var_geom_solref.shape);
            var_66 = wp::load(var_63);
            var_65 = wp::extract(var_66, var_64);
            var_67 = wp::mod(var_14, var_65);
            var_68 = wp::address(var_geom_solref, var_67, var_11);
            // geom_solimp[worldid % geom_solimp.shape[0], geomid],                               <L 2474>
            var_69 = &(var_geom_solimp.shape);
            var_72 = wp::load(var_69);
            var_71 = wp::extract(var_72, var_70);
            var_73 = wp::mod(var_14, var_71);
            var_74 = wp::address(var_geom_solimp, var_73, var_11);
            // geom_friction[worldid % geom_friction.shape[0], geomid],                           <L 2475>
            var_75 = &(var_geom_friction.shape);
            var_78 = wp::load(var_75);
            var_77 = wp::extract(var_78, var_76);
            var_79 = wp::mod(var_14, var_77);
            var_80 = wp::address(var_geom_friction, var_79, var_11);
            // geom_gap[worldid % geom_gap.shape[0], geomid],                                     <L 2476>
            var_81 = &(var_geom_gap.shape);
            var_84 = wp::load(var_81);
            var_83 = wp::extract(var_84, var_82);
            var_85 = wp::mod(var_14, var_83);
            var_86 = wp::address(var_geom_gap, var_85, var_11);
            // flex_condim[flexid],                                                               <L 2477>
            var_87 = wp::address(var_flex_condim, var_41);
            // flex_priority[flexid],                                                             <L 2478>
            var_88 = wp::address(var_flex_priority, var_41);
            // flex_solmix[flexid],                                                               <L 2479>
            var_89 = wp::address(var_flex_solmix, var_41);
            // flex_solref[flexid],                                                               <L 2480>
            var_90 = wp::address(var_flex_solref, var_41);
            // flex_solimp[flexid],                                                               <L 2481>
            var_91 = wp::address(var_flex_solimp, var_41);
            // flex_friction[flexid],                                                             <L 2482>
            var_92 = wp::address(var_flex_friction, var_41);
            // flex_gap[flexid],                                                                  <L 2483>
            var_93 = wp::address(var_flex_gap, var_41);
            var_99 = wp::load(var_55);
            var_100 = wp::load(var_56);
            var_101 = wp::load(var_62);
            var_102 = wp::load(var_68);
            var_103 = wp::load(var_74);
            var_104 = wp::load(var_80);
            var_105 = wp::load(var_86);
            var_106 = wp::load(var_87);
            var_107 = wp::load(var_88);
            var_108 = wp::load(var_89);
            var_109 = wp::load(var_90);
            var_110 = wp::load(var_91);
            var_111 = wp::load(var_92);
            var_112 = wp::load(var_93);
            _mix_flex_contact_params_0(var_99, var_100, var_101, var_102, var_103, var_104, var_105, var_106, var_107, var_108, var_109, var_110, var_111, var_112, var_94, var_95, var_96, var_97, var_98);
        }
        var_113 = wp::where(var_38, var_94, var_17);
        var_114 = wp::where(var_38, var_54, var_19);
        var_115 = wp::where(var_38, var_95, var_21);
        var_116 = wp::where(var_38, var_96, var_24);
        var_117 = wp::where(var_38, var_97, var_30);
        var_118 = wp::where(var_38, var_98, var_36);
        if (!var_38) {
            // flex1 = cand_flex[i][0]                                                            <L 2486>
            var_119 = wp::address(var_cand_flex, var_0);
            var_122 = wp::load(var_119);
            var_121 = wp::extract(var_122, var_120);
            // flex2 = cand_flex[i][1]                                                            <L 2487>
            var_123 = wp::address(var_cand_flex, var_0);
            var_126 = wp::load(var_123);
            var_125 = wp::extract(var_126, var_124);
            // is_self = flex1 == flex2                                                           <L 2488>
            var_127 = (var_121 == var_125);
            // if is_self:                                                                        <L 2489>
            if (var_127) {
                // margin = float(0.0)                                                            <L 2490>
                var_129 = wp::float(var_128);
                // gap1 = float(0.0)                                                              <L 2491>
                var_131 = wp::float(var_130);
                // gap2 = float(0.0)                                                              <L 2492>
                var_133 = wp::float(var_132);
            }
            var_134 = wp::where(var_127, var_129, var_114);
            if (!var_127) {
                // margin = flex_margin[flex1] + flex_margin[flex2]                               <L 2494>
                var_135 = wp::address(var_flex_margin, var_121);
                var_136 = wp::address(var_flex_margin, var_125);
                var_138 = wp::load(var_135);
                var_139 = wp::load(var_136);
                var_137 = wp::add(var_138, var_139);
                // gap1 = flex_gap[flex1]                                                         <L 2495>
                var_140 = wp::address(var_flex_gap, var_121);
                var_142 = wp::load(var_140);
                var_141 = wp::copy(var_142);
                // gap2 = flex_gap[flex2]                                                         <L 2496>
                var_143 = wp::address(var_flex_gap, var_125);
                var_145 = wp::load(var_143);
                var_144 = wp::copy(var_145);
            }
            var_146 = wp::where(var_127, var_134, var_137);
            var_147 = wp::where(var_127, var_131, var_141);
            var_148 = wp::where(var_127, var_133, var_144);
            // mixed_condim, gap, solref, solimp, friction = _mix_flex_contact_params(            <L 2497>
            // flex_condim[flex1],                                                                <L 2498>
            var_149 = wp::address(var_flex_condim, var_121);
            // flex_priority[flex1],                                                              <L 2499>
            var_150 = wp::address(var_flex_priority, var_121);
            // flex_solmix[flex1],                                                                <L 2500>
            var_151 = wp::address(var_flex_solmix, var_121);
            // flex_solref[flex1],                                                                <L 2501>
            var_152 = wp::address(var_flex_solref, var_121);
            // flex_solimp[flex1],                                                                <L 2502>
            var_153 = wp::address(var_flex_solimp, var_121);
            // flex_friction[flex1],                                                              <L 2503>
            var_154 = wp::address(var_flex_friction, var_121);
            // gap1,                                                                              <L 2504>
            // flex_condim[flex2],                                                                <L 2505>
            var_155 = wp::address(var_flex_condim, var_125);
            // flex_priority[flex2],                                                              <L 2506>
            var_156 = wp::address(var_flex_priority, var_125);
            // flex_solmix[flex2],                                                                <L 2507>
            var_157 = wp::address(var_flex_solmix, var_125);
            // flex_solref[flex2],                                                                <L 2508>
            var_158 = wp::address(var_flex_solref, var_125);
            // flex_solimp[flex2],                                                                <L 2509>
            var_159 = wp::address(var_flex_solimp, var_125);
            // flex_friction[flex2],                                                              <L 2510>
            var_160 = wp::address(var_flex_friction, var_125);
            // gap2,                                                                              <L 2511>
            var_166 = wp::load(var_149);
            var_167 = wp::load(var_150);
            var_168 = wp::load(var_151);
            var_169 = wp::load(var_152);
            var_170 = wp::load(var_153);
            var_171 = wp::load(var_154);
            var_172 = wp::load(var_155);
            var_173 = wp::load(var_156);
            var_174 = wp::load(var_157);
            var_175 = wp::load(var_158);
            var_176 = wp::load(var_159);
            var_177 = wp::load(var_160);
            _mix_flex_contact_params_0(var_166, var_167, var_168, var_169, var_170, var_171, var_147, var_172, var_173, var_174, var_175, var_176, var_177, var_148, var_161, var_162, var_163, var_164, var_165);
            // if cand_vert[i][0] >= 0 and flex_dim[flex1] == 3:                                  <L 2514>
            var_179 = wp::address(var_cand_vert, var_0);
            var_182 = wp::load(var_179);
            var_181 = wp::extract(var_182, var_180);
            var_184 = (var_181 >= var_183);
            var_178 = var_184;
            if (var_178) {
                var_185 = wp::address(var_flex_dim, var_121);
                var_188 = wp::load(var_185);
                var_187 = (var_188 == var_186);
                var_178 = var_178 && var_187;
            }
            if (var_178) {
                // condim = 1                                                                     <L 2515>
            }
            var_190 = wp::where(var_178, var_189, var_113);
            if (!var_178) {
                // condim = mixed_condim                                                          <L 2517>
                var_191 = wp::copy(var_161);
            }
            var_192 = wp::where(var_178, var_190, var_191);
        }
        var_193 = wp::where(var_38, var_113, var_192);
        var_194 = wp::where(var_38, var_114, var_146);
        var_195 = wp::where(var_38, var_115, var_162);
        var_196 = wp::where(var_38, var_116, var_163);
        var_197 = wp::where(var_38, var_117, var_164);
        var_198 = wp::where(var_38, var_118, var_165);
        // if cand_dist[i] >= margin:                                                             <L 2519>
        var_199 = wp::address(var_cand_dist, var_0);
        var_201 = wp::load(var_199);
        var_200 = (var_201 >= var_194);
        if (var_200) {
            // return                                                                             <L 2520>
            continue;
        }
        // id_ = wp.atomic_add(nacon_out, 0, 1)                                                   <L 2522>
        var_204 = wp::atomic_add(var_nacon_out, var_202, var_203);
        // if id_ >= naconmax_in:                                                                 <L 2523>
        var_205 = (var_204 >= var_naconmax_in);
        if (var_205) {
            // if wp.static(bool(warn_overflow & OverflowType.NARROWPHASE)):                      <L 2524>
            // wp.printf(                                                                         <L 2525>
            // "flex contact overflow - please increase naconmax beyond %u\n"                     <L 2526>
            // naconmax_in,                                                                       <L 2528>
            printf(var_207, var_naconmax_in);
            // wp.atomic_or(overflow_out, worldid, wp.static(OverflowType.NARROWPHASE))           <L 2530>
            var_209 = wp::atomic_or(var_overflow_out, var_14, var_208);
            // return                                                                             <L 2531>
            continue;
        }
        // contact_dist_out[id_] = cand_dist[i]                                                   <L 2533>
        var_210 = wp::address(var_cand_dist, var_0);
        var_211 = wp::load(var_210);
        wp::array_store(var_contact_dist_out, var_204, var_211);
        // contact_pos_out[id_] = cand_pos[i]                                                     <L 2534>
        var_212 = wp::address(var_cand_pos, var_0);
        var_213 = wp::load(var_212);
        wp::array_store(var_contact_pos_out, var_204, var_213);
        // contact_frame_out[id_] = make_frame(cand_nrm[i])                                       <L 2535>
        var_214 = wp::address(var_cand_nrm, var_0);
        var_216 = wp::load(var_214);
        var_215 = make_frame_0(var_216);
        wp::array_store(var_contact_frame_out, var_204, var_215);
        // if geomid >= 0 and geom_type[geomid] == int(GeomType.PLANE):                           <L 2536>
        var_219 = (var_11 >= var_218);
        var_217 = var_219;
        if (var_217) {
            var_220 = wp::address(var_geom_type, var_11);
            var_223 = wp::int(var_222);
            var_225 = wp::load(var_220);
            var_224 = (var_225 == var_223);
            var_217 = var_217 && var_224;
        }
        if (var_217) {
            // contact_includemargin_out[id_] = margin - gap                                      <L 2537>
            var_226 = wp::sub(var_194, var_195);
            wp::array_store(var_contact_includemargin_out, var_204, var_226);
        }
        if (!var_217) {
            // contact_includemargin_out[id_] = margin                                            <L 2539>
            wp::array_store(var_contact_includemargin_out, var_204, var_194);
        }
        // contact_friction_out[id_] = friction                                                   <L 2540>
        wp::array_store(var_contact_friction_out, var_204, var_198);
        // contact_solref_out[id_] = solref                                                       <L 2541>
        wp::array_store(var_contact_solref_out, var_204, var_196);
        // contact_solreffriction_out[id_] = solref                                               <L 2542>
        wp::array_store(var_contact_solreffriction_out, var_204, var_196);
        // contact_solimp_out[id_] = solimp                                                       <L 2543>
        wp::array_store(var_contact_solimp_out, var_204, var_197);
        // contact_dim_out[id_] = condim                                                          <L 2544>
        wp::array_store(var_contact_dim_out, var_204, var_193);
        // contact_geom_out[id_] = cand_geom[i]                                                   <L 2545>
        var_227 = wp::address(var_cand_geom, var_0);
        var_228 = wp::load(var_227);
        wp::array_store(var_contact_geom_out, var_204, var_228);
        // contact_flex_out[id_] = cand_flex[i]                                                   <L 2546>
        var_229 = wp::address(var_cand_flex, var_0);
        var_230 = wp::load(var_229);
        wp::array_store(var_contact_flex_out, var_204, var_230);
        // contact_elem_out[id_] = cand_elem[i]                                                   <L 2547>
        var_231 = wp::address(var_cand_elem, var_0);
        var_232 = wp::load(var_231);
        wp::array_store(var_contact_elem_out, var_204, var_232);
        // contact_vert_out[id_] = cand_vert[i]                                                   <L 2548>
        var_233 = wp::address(var_cand_vert, var_0);
        var_234 = wp::load(var_233);
        wp::array_store(var_contact_vert_out, var_204, var_234);
        // contact_worldid_out[id_] = cand_worldid[i]                                             <L 2549>
        var_235 = wp::address(var_cand_worldid, var_0);
        var_236 = wp::load(var_235);
        wp::array_store(var_contact_worldid_out, var_204, var_236);
        // f0 = cand_flex[i][0]                                                                   <L 2550>
        var_237 = wp::address(var_cand_flex, var_0);
        var_240 = wp::load(var_237);
        var_239 = wp::extract(var_240, var_238);
        // f1 = cand_flex[i][1]                                                                   <L 2551>
        var_241 = wp::address(var_cand_flex, var_0);
        var_244 = wp::load(var_241);
        var_243 = wp::extract(var_244, var_242);
        // g0 = cand_geom[i][0]                                                                   <L 2552>
        var_245 = wp::address(var_cand_geom, var_0);
        var_248 = wp::load(var_245);
        var_247 = wp::extract(var_248, var_246);
        // g1 = cand_geom[i][1]                                                                   <L 2553>
        var_249 = wp::address(var_cand_geom, var_0);
        var_252 = wp::load(var_249);
        var_251 = wp::extract(var_252, var_250);
        // wants = (                                                                              <L 2554>
        // (f0 >= 0 and flex_passive[f0] != 0 and flex_interp[f0] == 0 and flex_dim[f0] >= 2)       <L 2555>
        var_257 = (var_239 >= var_256);
        var_255 = var_257;
        if (var_255) {
            var_258 = wp::address(var_flex_passive, var_239);
            var_261 = wp::load(var_258);
            var_260 = (var_261 != var_259);
            var_255 = var_255 && var_260;
        }
        if (var_255) {
            var_262 = wp::address(var_flex_interp, var_239);
            var_265 = wp::load(var_262);
            var_264 = (var_265 == var_263);
            var_255 = var_255 && var_264;
        }
        if (var_255) {
            var_266 = wp::address(var_flex_dim, var_239);
            var_269 = wp::load(var_266);
            var_268 = (var_269 >= var_267);
            var_255 = var_255 && var_268;
        }
        var_254 = var_255;
        if (!var_254) {
            // or (f1 >= 0 and flex_passive[f1] != 0 and flex_interp[f1] == 0 and flex_dim[f1] >= 2)       <L 2556>
            var_272 = (var_243 >= var_271);
            var_270 = var_272;
            if (var_270) {
                var_273 = wp::address(var_flex_passive, var_243);
                var_276 = wp::load(var_273);
                var_275 = (var_276 != var_274);
                var_270 = var_270 && var_275;
            }
            if (var_270) {
                var_277 = wp::address(var_flex_interp, var_243);
                var_280 = wp::load(var_277);
                var_279 = (var_280 == var_278);
                var_270 = var_270 && var_279;
            }
            if (var_270) {
                var_281 = wp::address(var_flex_dim, var_243);
                var_284 = wp::load(var_281);
                var_283 = (var_284 >= var_282);
                var_270 = var_270 && var_283;
            }
            var_254 = var_254 || var_270;
        }
        var_253 = var_254;
        if (var_253) {
            // ) and opt_integrator == int(IntegratorType.DISCRETE)                               <L 2557>
            var_287 = wp::int(var_286);
            var_288 = (var_opt_integrator == var_287);
            var_253 = var_253 && var_288;
        }
        // ok = (f0 >= 0 or (g0 >= 0 and body_weldid[geom_bodyid[g0]] == 0)) and (                <L 2558>
        var_292 = (var_239 >= var_291);
        var_290 = var_292;
        if (!var_290) {
            var_295 = (var_247 >= var_294);
            var_293 = var_295;
            if (var_293) {
                var_296 = wp::address(var_geom_bodyid, var_247);
                var_298 = wp::load(var_296);
                var_297 = wp::address(var_body_weldid, var_298);
                var_301 = wp::load(var_297);
                var_300 = (var_301 == var_299);
                var_293 = var_293 && var_300;
            }
            var_290 = var_290 || var_293;
        }
        var_289 = var_290;
        if (var_289) {
            // f1 >= 0 or (g1 >= 0 and body_weldid[geom_bodyid[g1]] == 0)                         <L 2559>
            var_304 = (var_243 >= var_303);
            var_302 = var_304;
            if (!var_302) {
                var_307 = (var_251 >= var_306);
                var_305 = var_307;
                if (var_305) {
                    var_308 = wp::address(var_geom_bodyid, var_251);
                    var_310 = wp::load(var_308);
                    var_309 = wp::address(var_body_weldid, var_310);
                    var_313 = wp::load(var_309);
                    var_312 = (var_313 == var_311);
                    var_305 = var_305 && var_312;
                }
                var_302 = var_302 || var_305;
            }
            var_289 = var_289 && var_302;
        }
        // is_passive = wants and ok                                                              <L 2561>
        var_314 = var_253;
        if (var_314) {
            var_314 = var_314 && var_289;
        }
        // contact_type_out[id_] = int(ContactType.PASSIVE) if is_passive else int(ContactType.CONSTRAINT)       <L 2562>
        if (var_314) {
            var_317 = wp::int(var_316);
        }
        if (!var_314) {
            var_320 = wp::int(var_319);
        }
        var_321 = wp::where(var_314, var_317, var_320);
        wp::array_store(var_contact_type_out, var_204, var_321);
        // contact_geomcollisionid_out[id_] = 0                                                   <L 2563>
        wp::array_store(var_contact_geomcollisionid_out, var_204, var_322);
    }
}

