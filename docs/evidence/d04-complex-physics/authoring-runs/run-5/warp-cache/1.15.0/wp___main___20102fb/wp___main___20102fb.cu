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


struct VolumeData_53ac1a2d
{
    wp::vec_t<3, wp::float32> center;
    wp::vec_t<3, wp::float32> half_size;
    wp::array_t<wp::vec_t<3, wp::float32>> oct_aabb;
    wp::array_t<wp::vec_t<8, wp::int32>> oct_child;
    wp::array_t<wp::vec_t<8, wp::float32>> oct_coeff;
    wp::int32 root;
    bool valid;


    VolumeData_53ac1a2d() = default;
    CUDA_CALLABLE VolumeData_53ac1a2d(wp::vec_t<3, wp::float32> const& center,
    wp::vec_t<3, wp::float32> const& half_size = {},
    wp::array_t<wp::vec_t<3, wp::float32>> const& oct_aabb = {},
    wp::array_t<wp::vec_t<8, wp::int32>> const& oct_child = {},
    wp::array_t<wp::vec_t<8, wp::float32>> const& oct_coeff = {},
    wp::int32 const& root = {},
    bool const& valid = {})
        : center{center}
        , half_size{half_size}
        , oct_aabb{oct_aabb}
        , oct_child{oct_child}
        , oct_coeff{oct_coeff}
        , root{root}
        , valid{valid}

    {
    }

    CUDA_CALLABLE VolumeData_53ac1a2d& operator += (const VolumeData_53ac1a2d& rhs)
    {    center += rhs.center;
    half_size += rhs.half_size;
    root += rhs.root;

        return *this;}

};

static CUDA_CALLABLE void adj_VolumeData_53ac1a2d(wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::array_t<wp::vec_t<8, wp::int32>> const&,
    wp::array_t<wp::vec_t<8, wp::float32>> const&,
    wp::int32 const&,
    bool const&,
    wp::vec_t<3, wp::float32> & adj_center,
    wp::vec_t<3, wp::float32> & adj_half_size,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::int32>> & adj_oct_child,
    wp::array_t<wp::vec_t<8, wp::float32>> & adj_oct_coeff,
    wp::int32 & adj_root,
    bool & adj_valid,
    VolumeData_53ac1a2d & adj_ret)
{
    adj_center += adj_ret.center;
    adj_half_size += adj_ret.half_size;
    adj_oct_aabb = adj_ret.oct_aabb;
    adj_oct_child = adj_ret.oct_child;
    adj_oct_coeff = adj_ret.oct_coeff;
    adj_root += adj_ret.root;
    adj_valid += adj_ret.valid;
}

// Required when compiling adjoints.
CUDA_CALLABLE VolumeData_53ac1a2d add(const VolumeData_53ac1a2d& a, const VolumeData_53ac1a2d& b)
{
    return VolumeData_53ac1a2d();
}

CUDA_CALLABLE void adj_atomic_add(VolumeData_53ac1a2d* p, VolumeData_53ac1a2d t)
{
    wp::adj_atomic_add(&p->center, t.center);
    wp::adj_atomic_add(&p->half_size, t.half_size);
    wp::adj_atomic_add(&p->oct_aabb, t.oct_aabb);
    wp::adj_atomic_add(&p->oct_child, t.oct_child);
    wp::adj_atomic_add(&p->oct_coeff, t.oct_coeff);
    wp::adj_atomic_add(&p->root, t.root);
    wp::adj_atomic_add(&p->valid, t.valid);
}




// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:399
static CUDA_CALLABLE void box_project_0(
    wp::vec_t<3, wp::float32> var_center,
    wp::vec_t<3, wp::float32> var_half_size,
    wp::vec_t<3, wp::float32> var_xyz,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    wp::float32 var_3;
    const wp::int32 var_4 = 0;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::int32 var_16 = 2;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    bool var_20;
    const wp::int32 var_21 = 0;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    const wp::int32 var_25 = 1;
    wp::float32 var_26;
    const wp::float32 var_27 = 0.0;
    bool var_28;
    const wp::int32 var_29 = 2;
    wp::float32 var_30;
    const wp::float32 var_31 = 0.0;
    bool var_32;
    const wp::float32 var_33 = 0.0;
    const wp::float32 var_34 = 0.0;
    const wp::float32 var_35 = 0.0001;
    const wp::int32 var_36 = 0;
    wp::float32 var_37;
    const wp::int32 var_38 = 1;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    wp::vec_t<3, wp::float32> var_42;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    const wp::float32 var_45 = 0.0;
    bool var_46;
    const wp::int32 var_47 = 0;
    wp::float32 var_48;
    const wp::int32 var_49 = 0;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::int32 var_53 = 0;
    wp::float32 var_54;
    const wp::float32 var_55 = 0.0;
    bool var_56;
    const wp::int32 var_57 = 0;
    wp::float32 var_58;
    const wp::int32 var_59 = 0;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 1;
    wp::float32 var_64;
    const wp::int32 var_65 = 2;
    wp::float32 var_66;
    wp::vec_t<3, wp::float32> var_67;
    wp::vec_t<3, wp::float32> var_68;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    const wp::int32 var_71 = 0;
    wp::float32 var_72;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::int32 var_75 = 1;
    wp::float32 var_76;
    const wp::int32 var_77 = 2;
    wp::float32 var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::vec_t<3, wp::float32> var_80;
    wp::float32 var_81;
    wp::vec_t<3, wp::float32> var_82;
    const wp::int32 var_83 = 1;
    wp::float32 var_84;
    const wp::float32 var_85 = 0.0;
    bool var_86;
    const wp::int32 var_87 = 1;
    wp::float32 var_88;
    const wp::int32 var_89 = 1;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    const wp::int32 var_93 = 1;
    wp::float32 var_94;
    const wp::float32 var_95 = 0.0;
    bool var_96;
    const wp::int32 var_97 = 0;
    wp::float32 var_98;
    const wp::int32 var_99 = 1;
    wp::float32 var_100;
    const wp::int32 var_101 = 1;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    const wp::int32 var_105 = 2;
    wp::float32 var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    const wp::int32 var_109 = 0;
    wp::float32 var_110;
    const wp::int32 var_111 = 1;
    wp::float32 var_112;
    const wp::int32 var_113 = 1;
    wp::float32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    const wp::int32 var_117 = 2;
    wp::float32 var_118;
    wp::vec_t<3, wp::float32> var_119;
    wp::vec_t<3, wp::float32> var_120;
    wp::float32 var_121;
    wp::vec_t<3, wp::float32> var_122;
    const wp::int32 var_123 = 2;
    wp::float32 var_124;
    const wp::float32 var_125 = 0.0;
    bool var_126;
    const wp::int32 var_127 = 2;
    wp::float32 var_128;
    const wp::int32 var_129 = 2;
    wp::float32 var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    const wp::int32 var_133 = 2;
    wp::float32 var_134;
    const wp::float32 var_135 = 0.0;
    bool var_136;
    const wp::int32 var_137 = 0;
    wp::float32 var_138;
    const wp::int32 var_139 = 1;
    wp::float32 var_140;
    const wp::int32 var_141 = 2;
    wp::float32 var_142;
    const wp::int32 var_143 = 2;
    wp::float32 var_144;
    wp::float32 var_145;
    wp::float32 var_146;
    wp::vec_t<3, wp::float32> var_147;
    wp::vec_t<3, wp::float32> var_148;
    const wp::int32 var_149 = 0;
    wp::float32 var_150;
    const wp::int32 var_151 = 1;
    wp::float32 var_152;
    const wp::int32 var_153 = 2;
    wp::float32 var_154;
    const wp::int32 var_155 = 2;
    wp::float32 var_156;
    wp::float32 var_157;
    wp::float32 var_158;
    wp::vec_t<3, wp::float32> var_159;
    wp::vec_t<3, wp::float32> var_160;
    wp::float32 var_161;
    wp::vec_t<3, wp::float32> var_162;
    wp::float32 var_163;
    //---------
    // forward
    // def box_project(center: wp.vec3, half_size: wp.vec3, xyz: wp.vec3) -> Tuple[float, wp.vec3]:       <L 400>
    // r = xyz - center                                                                       <L 401>
    var_0 = wp::sub(var_xyz, var_center);
    // q = wp.vec3(wp.abs(r[0]) - half_size[0], wp.abs(r[1]) - half_size[1], wp.abs(r[2]) - half_size[2])       <L 402>
    var_2 = wp::extract(var_0, var_1);
    var_3 = wp::abs(var_2);
    var_5 = wp::extract(var_half_size, var_4);
    var_6 = wp::sub(var_3, var_5);
    var_8 = wp::extract(var_0, var_7);
    var_9 = wp::abs(var_8);
    var_11 = wp::extract(var_half_size, var_10);
    var_12 = wp::sub(var_9, var_11);
    var_14 = wp::extract(var_0, var_13);
    var_15 = wp::abs(var_14);
    var_17 = wp::extract(var_half_size, var_16);
    var_18 = wp::sub(var_15, var_17);
    var_19 = wp::vec_t<3, wp::float32>(var_6, var_12, var_18);
    // if q[0] <= 0.0 and q[1] <= 0.0 and q[2] <= 0.0:                                        <L 404>
    var_22 = wp::extract(var_19, var_21);
    var_24 = (var_22 <= var_23);
    var_20 = var_24;
    if (var_20) {
        var_26 = wp::extract(var_19, var_25);
        var_28 = (var_26 <= var_27);
        var_20 = var_20 && var_28;
    }
    if (var_20) {
        var_30 = wp::extract(var_19, var_29);
        var_32 = (var_30 <= var_31);
        var_20 = var_20 && var_32;
    }
    if (var_20) {
        // return 0.0, xyz                                                                    <L 405>
        ret_0 = var_33;
        ret_1 = var_xyz;
        return;
    }
    if (!var_20) {
        // dist_sqr = 0.0                                                                     <L 408>
        // eps = 1e-4                                                                         <L 409>
        // point = wp.vec3(xyz[0], xyz[1], xyz[2])                                            <L 410>
        var_37 = wp::extract(var_xyz, var_36);
        var_39 = wp::extract(var_xyz, var_38);
        var_41 = wp::extract(var_xyz, var_40);
        var_42 = wp::vec_t<3, wp::float32>(var_37, var_39, var_41);
        // if q[0] >= 0.0:                                                                    <L 412>
        var_44 = wp::extract(var_19, var_43);
        var_46 = (var_44 >= var_45);
        if (var_46) {
            // dist_sqr += q[0] * q[0]                                                        <L 413>
            var_48 = wp::extract(var_19, var_47);
            var_50 = wp::extract(var_19, var_49);
            var_51 = wp::mul(var_48, var_50);
            var_52 = wp::add(var_34, var_51);
            // if r[0] > 0.0:                                                                 <L 414>
            var_54 = wp::extract(var_0, var_53);
            var_56 = (var_54 > var_55);
            if (var_56) {
                // point = wp.vec3(point[0] - (q[0] + eps), point[1], point[2])               <L 415>
                var_58 = wp::extract(var_42, var_57);
                var_60 = wp::extract(var_19, var_59);
                var_61 = wp::add(var_60, var_35);
                var_62 = wp::sub(var_58, var_61);
                var_64 = wp::extract(var_42, var_63);
                var_66 = wp::extract(var_42, var_65);
                var_67 = wp::vec_t<3, wp::float32>(var_62, var_64, var_66);
            }
            var_68 = wp::where(var_56, var_67, var_42);
            if (!var_56) {
                // point = wp.vec3(point[0] + (q[0] + eps), point[1], point[2])               <L 417>
                var_70 = wp::extract(var_68, var_69);
                var_72 = wp::extract(var_19, var_71);
                var_73 = wp::add(var_72, var_35);
                var_74 = wp::add(var_70, var_73);
                var_76 = wp::extract(var_68, var_75);
                var_78 = wp::extract(var_68, var_77);
                var_79 = wp::vec_t<3, wp::float32>(var_74, var_76, var_78);
            }
            var_80 = wp::where(var_56, var_68, var_79);
        }
        var_81 = wp::where(var_46, var_52, var_34);
        var_82 = wp::where(var_46, var_80, var_42);
        // if q[1] >= 0.0:                                                                    <L 419>
        var_84 = wp::extract(var_19, var_83);
        var_86 = (var_84 >= var_85);
        if (var_86) {
            // dist_sqr += q[1] * q[1]                                                        <L 420>
            var_88 = wp::extract(var_19, var_87);
            var_90 = wp::extract(var_19, var_89);
            var_91 = wp::mul(var_88, var_90);
            var_92 = wp::add(var_81, var_91);
            // if r[1] > 0.0:                                                                 <L 421>
            var_94 = wp::extract(var_0, var_93);
            var_96 = (var_94 > var_95);
            if (var_96) {
                // point = wp.vec3(point[0], point[1] - (q[1] + eps), point[2])               <L 422>
                var_98 = wp::extract(var_82, var_97);
                var_100 = wp::extract(var_82, var_99);
                var_102 = wp::extract(var_19, var_101);
                var_103 = wp::add(var_102, var_35);
                var_104 = wp::sub(var_100, var_103);
                var_106 = wp::extract(var_82, var_105);
                var_107 = wp::vec_t<3, wp::float32>(var_98, var_104, var_106);
            }
            var_108 = wp::where(var_96, var_107, var_82);
            if (!var_96) {
                // point = wp.vec3(point[0], point[1] + (q[1] + eps), point[2])               <L 424>
                var_110 = wp::extract(var_108, var_109);
                var_112 = wp::extract(var_108, var_111);
                var_114 = wp::extract(var_19, var_113);
                var_115 = wp::add(var_114, var_35);
                var_116 = wp::add(var_112, var_115);
                var_118 = wp::extract(var_108, var_117);
                var_119 = wp::vec_t<3, wp::float32>(var_110, var_116, var_118);
            }
            var_120 = wp::where(var_96, var_108, var_119);
        }
        var_121 = wp::where(var_86, var_92, var_81);
        var_122 = wp::where(var_86, var_120, var_82);
        // if q[2] >= 0.0:                                                                    <L 426>
        var_124 = wp::extract(var_19, var_123);
        var_126 = (var_124 >= var_125);
        if (var_126) {
            // dist_sqr += q[2] * q[2]                                                        <L 427>
            var_128 = wp::extract(var_19, var_127);
            var_130 = wp::extract(var_19, var_129);
            var_131 = wp::mul(var_128, var_130);
            var_132 = wp::add(var_121, var_131);
            // if r[2] > 0.0:                                                                 <L 428>
            var_134 = wp::extract(var_0, var_133);
            var_136 = (var_134 > var_135);
            if (var_136) {
                // point = wp.vec3(point[0], point[1], point[2] - (q[2] + eps))               <L 429>
                var_138 = wp::extract(var_122, var_137);
                var_140 = wp::extract(var_122, var_139);
                var_142 = wp::extract(var_122, var_141);
                var_144 = wp::extract(var_19, var_143);
                var_145 = wp::add(var_144, var_35);
                var_146 = wp::sub(var_142, var_145);
                var_147 = wp::vec_t<3, wp::float32>(var_138, var_140, var_146);
            }
            var_148 = wp::where(var_136, var_147, var_122);
            if (!var_136) {
                // point = wp.vec3(point[0], point[1], point[2] + (q[2] + eps))               <L 431>
                var_150 = wp::extract(var_148, var_149);
                var_152 = wp::extract(var_148, var_151);
                var_154 = wp::extract(var_148, var_153);
                var_156 = wp::extract(var_19, var_155);
                var_157 = wp::add(var_156, var_35);
                var_158 = wp::add(var_154, var_157);
                var_159 = wp::vec_t<3, wp::float32>(var_150, var_152, var_158);
            }
            var_160 = wp::where(var_136, var_148, var_159);
        }
        var_161 = wp::where(var_126, var_132, var_121);
        var_162 = wp::where(var_126, var_160, var_122);
        // return wp.sqrt(dist_sqr), point                                                    <L 433>
        var_163 = wp::sqrt(var_161);
        ret_0 = var_163;
        ret_1 = var_162;
        return;
    }
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:318
static CUDA_CALLABLE void find_oct_0(
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::vec_t<3, wp::float32> var_p,
    bool var_grad,
    wp::int32 var_root,
    wp::int32 & ret_0,
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> & ret_1)
{
    //---------
    // primal vars
    wp::int32 var_0;
    const wp::int32 var_1 = 100;
    wp::int32 var_2;
    const wp::float32 var_3 = 0.0;
    wp::vec_t<8, wp::float32> var_4;
    const wp::float32 var_5 = 0.0;
    wp::vec_t<8, wp::float32> var_6;
    const wp::float32 var_7 = 0.0;
    wp::vec_t<8, wp::float32> var_8;
    const wp::float32 var_9 = 1e-06;
    const wp::int32 var_10 = 0;
    bool var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    wp::int32 var_14;
    const wp::int32 var_15 = -1;
    bool var_16;
    const wp::str var_17 = "ERROR: Invalid node number\n";
    const wp::int32 var_18 = -1;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_19;
    const wp::int32 var_20 = 0;
    wp::vec_t<3, wp::float32>* var_21;
    const wp::int32 var_22 = 1;
    wp::vec_t<3, wp::float32>* var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    const wp::int32 var_27 = 0;
    wp::vec_t<3, wp::float32>* var_28;
    const wp::int32 var_29 = 1;
    wp::vec_t<3, wp::float32>* var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    bool var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    wp::float32 var_37;
    const wp::int32 var_38 = 0;
    wp::float32 var_39;
    bool var_40;
    const wp::int32 var_41 = 0;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::int32 var_44 = 0;
    wp::float32 var_45;
    bool var_46;
    const wp::int32 var_47 = 1;
    wp::float32 var_48;
    wp::float32 var_49;
    const wp::int32 var_50 = 1;
    wp::float32 var_51;
    bool var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::int32 var_56 = 1;
    wp::float32 var_57;
    bool var_58;
    const wp::int32 var_59 = 2;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::int32 var_62 = 2;
    wp::float32 var_63;
    bool var_64;
    const wp::int32 var_65 = 2;
    wp::float32 var_66;
    wp::float32 var_67;
    const wp::int32 var_68 = 2;
    wp::float32 var_69;
    bool var_70;
    wp::int32 var_71;
    wp::vec_t<3, wp::float32> var_72;
    wp::vec_t<3, wp::float32> var_73;
    wp::vec_t<3, wp::float32> var_74;
    wp::vec_t<8, wp::int32>* var_75;
    const wp::int32 var_76 = 0;
    wp::int32 var_77;
    wp::vec_t<8, wp::int32> var_78;
    const wp::int32 var_79 = -1;
    bool var_80;
    wp::int32 var_81;
    wp::vec_t<8, wp::int32>* var_82;
    const wp::int32 var_83 = 1;
    wp::int32 var_84;
    wp::vec_t<8, wp::int32> var_85;
    const wp::int32 var_86 = -1;
    bool var_87;
    wp::int32 var_88;
    wp::int32 var_89;
    wp::vec_t<8, wp::int32>* var_90;
    const wp::int32 var_91 = 2;
    wp::int32 var_92;
    wp::vec_t<8, wp::int32> var_93;
    const wp::int32 var_94 = -1;
    bool var_95;
    wp::int32 var_96;
    wp::int32 var_97;
    wp::vec_t<8, wp::int32>* var_98;
    const wp::int32 var_99 = 3;
    wp::int32 var_100;
    wp::vec_t<8, wp::int32> var_101;
    const wp::int32 var_102 = -1;
    bool var_103;
    wp::int32 var_104;
    wp::int32 var_105;
    wp::vec_t<8, wp::int32>* var_106;
    const wp::int32 var_107 = 4;
    wp::int32 var_108;
    wp::vec_t<8, wp::int32> var_109;
    const wp::int32 var_110 = -1;
    bool var_111;
    wp::int32 var_112;
    wp::int32 var_113;
    wp::vec_t<8, wp::int32>* var_114;
    const wp::int32 var_115 = 5;
    wp::int32 var_116;
    wp::vec_t<8, wp::int32> var_117;
    const wp::int32 var_118 = -1;
    bool var_119;
    wp::int32 var_120;
    wp::int32 var_121;
    wp::vec_t<8, wp::int32>* var_122;
    const wp::int32 var_123 = 6;
    wp::int32 var_124;
    wp::vec_t<8, wp::int32> var_125;
    const wp::int32 var_126 = -1;
    bool var_127;
    wp::int32 var_128;
    wp::int32 var_129;
    wp::vec_t<8, wp::int32>* var_130;
    const wp::int32 var_131 = 7;
    wp::int32 var_132;
    wp::vec_t<8, wp::int32> var_133;
    const wp::int32 var_134 = -1;
    bool var_135;
    wp::int32 var_136;
    wp::int32 var_137;
    const wp::int32 var_138 = 0;
    bool var_139;
    bool var_140;
    const wp::int32 var_141 = 0;
    const wp::int32 var_142 = 1;
    wp::int32 var_143;
    const wp::int32 var_144 = 0;
    wp::float32 var_145;
    const wp::float32 var_146 = 1.0;
    const wp::int32 var_147 = 0;
    wp::float32 var_148;
    wp::float32 var_149;
    wp::float32 var_150;
    const wp::int32 var_151 = 2;
    wp::int32 var_152;
    const wp::int32 var_153 = 1;
    wp::float32 var_154;
    const wp::float32 var_155 = 1.0;
    const wp::int32 var_156 = 1;
    wp::float32 var_157;
    wp::float32 var_158;
    wp::float32 var_159;
    wp::float32 var_160;
    const wp::int32 var_161 = 4;
    wp::int32 var_162;
    const wp::int32 var_163 = 2;
    wp::float32 var_164;
    const wp::float32 var_165 = 1.0;
    const wp::int32 var_166 = 2;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    wp::float32 var_170;
    const wp::int32 var_171 = 1;
    const wp::int32 var_172 = 1;
    wp::int32 var_173;
    const wp::int32 var_174 = 0;
    wp::float32 var_175;
    const wp::float32 var_176 = 1.0;
    const wp::int32 var_177 = 0;
    wp::float32 var_178;
    wp::float32 var_179;
    wp::float32 var_180;
    const wp::int32 var_181 = 2;
    wp::int32 var_182;
    const wp::int32 var_183 = 1;
    wp::float32 var_184;
    const wp::float32 var_185 = 1.0;
    const wp::int32 var_186 = 1;
    wp::float32 var_187;
    wp::float32 var_188;
    wp::float32 var_189;
    wp::float32 var_190;
    const wp::int32 var_191 = 4;
    wp::int32 var_192;
    const wp::int32 var_193 = 2;
    wp::float32 var_194;
    const wp::float32 var_195 = 1.0;
    const wp::int32 var_196 = 2;
    wp::float32 var_197;
    wp::float32 var_198;
    wp::float32 var_199;
    wp::float32 var_200;
    const wp::int32 var_201 = 2;
    const wp::int32 var_202 = 1;
    wp::int32 var_203;
    const wp::int32 var_204 = 0;
    wp::float32 var_205;
    const wp::float32 var_206 = 1.0;
    const wp::int32 var_207 = 0;
    wp::float32 var_208;
    wp::float32 var_209;
    wp::float32 var_210;
    const wp::int32 var_211 = 2;
    wp::int32 var_212;
    const wp::int32 var_213 = 1;
    wp::float32 var_214;
    const wp::float32 var_215 = 1.0;
    const wp::int32 var_216 = 1;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    wp::float32 var_220;
    const wp::int32 var_221 = 4;
    wp::int32 var_222;
    const wp::int32 var_223 = 2;
    wp::float32 var_224;
    const wp::float32 var_225 = 1.0;
    const wp::int32 var_226 = 2;
    wp::float32 var_227;
    wp::float32 var_228;
    wp::float32 var_229;
    wp::float32 var_230;
    const wp::int32 var_231 = 3;
    const wp::int32 var_232 = 1;
    wp::int32 var_233;
    const wp::int32 var_234 = 0;
    wp::float32 var_235;
    const wp::float32 var_236 = 1.0;
    const wp::int32 var_237 = 0;
    wp::float32 var_238;
    wp::float32 var_239;
    wp::float32 var_240;
    const wp::int32 var_241 = 2;
    wp::int32 var_242;
    const wp::int32 var_243 = 1;
    wp::float32 var_244;
    const wp::float32 var_245 = 1.0;
    const wp::int32 var_246 = 1;
    wp::float32 var_247;
    wp::float32 var_248;
    wp::float32 var_249;
    wp::float32 var_250;
    const wp::int32 var_251 = 4;
    wp::int32 var_252;
    const wp::int32 var_253 = 2;
    wp::float32 var_254;
    const wp::float32 var_255 = 1.0;
    const wp::int32 var_256 = 2;
    wp::float32 var_257;
    wp::float32 var_258;
    wp::float32 var_259;
    wp::float32 var_260;
    const wp::int32 var_261 = 4;
    const wp::int32 var_262 = 1;
    wp::int32 var_263;
    const wp::int32 var_264 = 0;
    wp::float32 var_265;
    const wp::float32 var_266 = 1.0;
    const wp::int32 var_267 = 0;
    wp::float32 var_268;
    wp::float32 var_269;
    wp::float32 var_270;
    const wp::int32 var_271 = 2;
    wp::int32 var_272;
    const wp::int32 var_273 = 1;
    wp::float32 var_274;
    const wp::float32 var_275 = 1.0;
    const wp::int32 var_276 = 1;
    wp::float32 var_277;
    wp::float32 var_278;
    wp::float32 var_279;
    wp::float32 var_280;
    const wp::int32 var_281 = 4;
    wp::int32 var_282;
    const wp::int32 var_283 = 2;
    wp::float32 var_284;
    const wp::float32 var_285 = 1.0;
    const wp::int32 var_286 = 2;
    wp::float32 var_287;
    wp::float32 var_288;
    wp::float32 var_289;
    wp::float32 var_290;
    const wp::int32 var_291 = 5;
    const wp::int32 var_292 = 1;
    wp::int32 var_293;
    const wp::int32 var_294 = 0;
    wp::float32 var_295;
    const wp::float32 var_296 = 1.0;
    const wp::int32 var_297 = 0;
    wp::float32 var_298;
    wp::float32 var_299;
    wp::float32 var_300;
    const wp::int32 var_301 = 2;
    wp::int32 var_302;
    const wp::int32 var_303 = 1;
    wp::float32 var_304;
    const wp::float32 var_305 = 1.0;
    const wp::int32 var_306 = 1;
    wp::float32 var_307;
    wp::float32 var_308;
    wp::float32 var_309;
    wp::float32 var_310;
    const wp::int32 var_311 = 4;
    wp::int32 var_312;
    const wp::int32 var_313 = 2;
    wp::float32 var_314;
    const wp::float32 var_315 = 1.0;
    const wp::int32 var_316 = 2;
    wp::float32 var_317;
    wp::float32 var_318;
    wp::float32 var_319;
    wp::float32 var_320;
    const wp::int32 var_321 = 6;
    const wp::int32 var_322 = 1;
    wp::int32 var_323;
    const wp::int32 var_324 = 0;
    wp::float32 var_325;
    const wp::float32 var_326 = 1.0;
    const wp::int32 var_327 = 0;
    wp::float32 var_328;
    wp::float32 var_329;
    wp::float32 var_330;
    const wp::int32 var_331 = 2;
    wp::int32 var_332;
    const wp::int32 var_333 = 1;
    wp::float32 var_334;
    const wp::float32 var_335 = 1.0;
    const wp::int32 var_336 = 1;
    wp::float32 var_337;
    wp::float32 var_338;
    wp::float32 var_339;
    wp::float32 var_340;
    const wp::int32 var_341 = 4;
    wp::int32 var_342;
    const wp::int32 var_343 = 2;
    wp::float32 var_344;
    const wp::float32 var_345 = 1.0;
    const wp::int32 var_346 = 2;
    wp::float32 var_347;
    wp::float32 var_348;
    wp::float32 var_349;
    wp::float32 var_350;
    const wp::int32 var_351 = 7;
    const wp::int32 var_352 = 1;
    wp::int32 var_353;
    const wp::int32 var_354 = 0;
    wp::float32 var_355;
    const wp::float32 var_356 = 1.0;
    const wp::int32 var_357 = 0;
    wp::float32 var_358;
    wp::float32 var_359;
    wp::float32 var_360;
    const wp::int32 var_361 = 2;
    wp::int32 var_362;
    const wp::int32 var_363 = 1;
    wp::float32 var_364;
    const wp::float32 var_365 = 1.0;
    const wp::int32 var_366 = 1;
    wp::float32 var_367;
    wp::float32 var_368;
    wp::float32 var_369;
    wp::float32 var_370;
    const wp::int32 var_371 = 4;
    wp::int32 var_372;
    const wp::int32 var_373 = 2;
    wp::float32 var_374;
    const wp::float32 var_375 = 1.0;
    const wp::int32 var_376 = 2;
    wp::float32 var_377;
    wp::float32 var_378;
    wp::float32 var_379;
    wp::float32 var_380;
    const wp::float32 var_381 = 1.0;
    wp::vec_t<3, wp::float32> var_382;
    wp::vec_t<3, wp::float32> var_383;
    wp::vec_t<3, wp::float32> var_384;
    const wp::int32 var_385 = 0;
    const wp::int32 var_386 = 1;
    wp::int32 var_387;
    const wp::float32 var_388 = 1.0;
    const wp::float32 var_389 = -1.0;
    wp::float32 var_390;
    const wp::int32 var_391 = 2;
    wp::int32 var_392;
    const wp::int32 var_393 = 1;
    wp::float32 var_394;
    const wp::float32 var_395 = 1.0;
    const wp::int32 var_396 = 1;
    wp::float32 var_397;
    wp::float32 var_398;
    wp::float32 var_399;
    wp::float32 var_400;
    const wp::int32 var_401 = 4;
    wp::int32 var_402;
    const wp::int32 var_403 = 2;
    wp::float32 var_404;
    const wp::float32 var_405 = 1.0;
    const wp::int32 var_406 = 2;
    wp::float32 var_407;
    wp::float32 var_408;
    wp::float32 var_409;
    wp::float32 var_410;
    const wp::int32 var_411 = 0;
    wp::float32 var_412;
    wp::float32 var_413;
    const wp::int32 var_414 = 1;
    wp::int32 var_415;
    const wp::int32 var_416 = 0;
    wp::float32 var_417;
    const wp::float32 var_418 = 1.0;
    const wp::int32 var_419 = 0;
    wp::float32 var_420;
    wp::float32 var_421;
    wp::float32 var_422;
    const wp::int32 var_423 = 2;
    wp::int32 var_424;
    const wp::float32 var_425 = 1.0;
    const wp::float32 var_426 = -1.0;
    wp::float32 var_427;
    wp::float32 var_428;
    const wp::int32 var_429 = 4;
    wp::int32 var_430;
    const wp::int32 var_431 = 2;
    wp::float32 var_432;
    const wp::float32 var_433 = 1.0;
    const wp::int32 var_434 = 2;
    wp::float32 var_435;
    wp::float32 var_436;
    wp::float32 var_437;
    wp::float32 var_438;
    const wp::int32 var_439 = 1;
    wp::float32 var_440;
    wp::float32 var_441;
    const wp::int32 var_442 = 1;
    wp::int32 var_443;
    const wp::int32 var_444 = 0;
    wp::float32 var_445;
    const wp::float32 var_446 = 1.0;
    const wp::int32 var_447 = 0;
    wp::float32 var_448;
    wp::float32 var_449;
    wp::float32 var_450;
    const wp::int32 var_451 = 2;
    wp::int32 var_452;
    const wp::int32 var_453 = 1;
    wp::float32 var_454;
    const wp::float32 var_455 = 1.0;
    const wp::int32 var_456 = 1;
    wp::float32 var_457;
    wp::float32 var_458;
    wp::float32 var_459;
    wp::float32 var_460;
    const wp::int32 var_461 = 4;
    wp::int32 var_462;
    const wp::float32 var_463 = 1.0;
    const wp::float32 var_464 = -1.0;
    wp::float32 var_465;
    wp::float32 var_466;
    const wp::int32 var_467 = 2;
    wp::float32 var_468;
    wp::float32 var_469;
    const wp::int32 var_470 = 1;
    const wp::int32 var_471 = 1;
    wp::int32 var_472;
    const wp::float32 var_473 = 1.0;
    const wp::float32 var_474 = -1.0;
    wp::float32 var_475;
    const wp::int32 var_476 = 2;
    wp::int32 var_477;
    const wp::int32 var_478 = 1;
    wp::float32 var_479;
    const wp::float32 var_480 = 1.0;
    const wp::int32 var_481 = 1;
    wp::float32 var_482;
    wp::float32 var_483;
    wp::float32 var_484;
    wp::float32 var_485;
    const wp::int32 var_486 = 4;
    wp::int32 var_487;
    const wp::int32 var_488 = 2;
    wp::float32 var_489;
    const wp::float32 var_490 = 1.0;
    const wp::int32 var_491 = 2;
    wp::float32 var_492;
    wp::float32 var_493;
    wp::float32 var_494;
    wp::float32 var_495;
    const wp::int32 var_496 = 0;
    wp::float32 var_497;
    wp::float32 var_498;
    const wp::int32 var_499 = 1;
    wp::int32 var_500;
    const wp::int32 var_501 = 0;
    wp::float32 var_502;
    const wp::float32 var_503 = 1.0;
    const wp::int32 var_504 = 0;
    wp::float32 var_505;
    wp::float32 var_506;
    wp::float32 var_507;
    const wp::int32 var_508 = 2;
    wp::int32 var_509;
    const wp::float32 var_510 = 1.0;
    const wp::float32 var_511 = -1.0;
    wp::float32 var_512;
    wp::float32 var_513;
    const wp::int32 var_514 = 4;
    wp::int32 var_515;
    const wp::int32 var_516 = 2;
    wp::float32 var_517;
    const wp::float32 var_518 = 1.0;
    const wp::int32 var_519 = 2;
    wp::float32 var_520;
    wp::float32 var_521;
    wp::float32 var_522;
    wp::float32 var_523;
    const wp::int32 var_524 = 1;
    wp::float32 var_525;
    wp::float32 var_526;
    const wp::int32 var_527 = 1;
    wp::int32 var_528;
    const wp::int32 var_529 = 0;
    wp::float32 var_530;
    const wp::float32 var_531 = 1.0;
    const wp::int32 var_532 = 0;
    wp::float32 var_533;
    wp::float32 var_534;
    wp::float32 var_535;
    const wp::int32 var_536 = 2;
    wp::int32 var_537;
    const wp::int32 var_538 = 1;
    wp::float32 var_539;
    const wp::float32 var_540 = 1.0;
    const wp::int32 var_541 = 1;
    wp::float32 var_542;
    wp::float32 var_543;
    wp::float32 var_544;
    wp::float32 var_545;
    const wp::int32 var_546 = 4;
    wp::int32 var_547;
    const wp::float32 var_548 = 1.0;
    const wp::float32 var_549 = -1.0;
    wp::float32 var_550;
    wp::float32 var_551;
    const wp::int32 var_552 = 2;
    wp::float32 var_553;
    wp::float32 var_554;
    const wp::int32 var_555 = 2;
    const wp::int32 var_556 = 1;
    wp::int32 var_557;
    const wp::float32 var_558 = 1.0;
    const wp::float32 var_559 = -1.0;
    wp::float32 var_560;
    const wp::int32 var_561 = 2;
    wp::int32 var_562;
    const wp::int32 var_563 = 1;
    wp::float32 var_564;
    const wp::float32 var_565 = 1.0;
    const wp::int32 var_566 = 1;
    wp::float32 var_567;
    wp::float32 var_568;
    wp::float32 var_569;
    wp::float32 var_570;
    const wp::int32 var_571 = 4;
    wp::int32 var_572;
    const wp::int32 var_573 = 2;
    wp::float32 var_574;
    const wp::float32 var_575 = 1.0;
    const wp::int32 var_576 = 2;
    wp::float32 var_577;
    wp::float32 var_578;
    wp::float32 var_579;
    wp::float32 var_580;
    const wp::int32 var_581 = 0;
    wp::float32 var_582;
    wp::float32 var_583;
    const wp::int32 var_584 = 1;
    wp::int32 var_585;
    const wp::int32 var_586 = 0;
    wp::float32 var_587;
    const wp::float32 var_588 = 1.0;
    const wp::int32 var_589 = 0;
    wp::float32 var_590;
    wp::float32 var_591;
    wp::float32 var_592;
    const wp::int32 var_593 = 2;
    wp::int32 var_594;
    const wp::float32 var_595 = 1.0;
    const wp::float32 var_596 = -1.0;
    wp::float32 var_597;
    wp::float32 var_598;
    const wp::int32 var_599 = 4;
    wp::int32 var_600;
    const wp::int32 var_601 = 2;
    wp::float32 var_602;
    const wp::float32 var_603 = 1.0;
    const wp::int32 var_604 = 2;
    wp::float32 var_605;
    wp::float32 var_606;
    wp::float32 var_607;
    wp::float32 var_608;
    const wp::int32 var_609 = 1;
    wp::float32 var_610;
    wp::float32 var_611;
    const wp::int32 var_612 = 1;
    wp::int32 var_613;
    const wp::int32 var_614 = 0;
    wp::float32 var_615;
    const wp::float32 var_616 = 1.0;
    const wp::int32 var_617 = 0;
    wp::float32 var_618;
    wp::float32 var_619;
    wp::float32 var_620;
    const wp::int32 var_621 = 2;
    wp::int32 var_622;
    const wp::int32 var_623 = 1;
    wp::float32 var_624;
    const wp::float32 var_625 = 1.0;
    const wp::int32 var_626 = 1;
    wp::float32 var_627;
    wp::float32 var_628;
    wp::float32 var_629;
    wp::float32 var_630;
    const wp::int32 var_631 = 4;
    wp::int32 var_632;
    const wp::float32 var_633 = 1.0;
    const wp::float32 var_634 = -1.0;
    wp::float32 var_635;
    wp::float32 var_636;
    const wp::int32 var_637 = 2;
    wp::float32 var_638;
    wp::float32 var_639;
    const wp::int32 var_640 = 3;
    const wp::int32 var_641 = 1;
    wp::int32 var_642;
    const wp::float32 var_643 = 1.0;
    const wp::float32 var_644 = -1.0;
    wp::float32 var_645;
    const wp::int32 var_646 = 2;
    wp::int32 var_647;
    const wp::int32 var_648 = 1;
    wp::float32 var_649;
    const wp::float32 var_650 = 1.0;
    const wp::int32 var_651 = 1;
    wp::float32 var_652;
    wp::float32 var_653;
    wp::float32 var_654;
    wp::float32 var_655;
    const wp::int32 var_656 = 4;
    wp::int32 var_657;
    const wp::int32 var_658 = 2;
    wp::float32 var_659;
    const wp::float32 var_660 = 1.0;
    const wp::int32 var_661 = 2;
    wp::float32 var_662;
    wp::float32 var_663;
    wp::float32 var_664;
    wp::float32 var_665;
    const wp::int32 var_666 = 0;
    wp::float32 var_667;
    wp::float32 var_668;
    const wp::int32 var_669 = 1;
    wp::int32 var_670;
    const wp::int32 var_671 = 0;
    wp::float32 var_672;
    const wp::float32 var_673 = 1.0;
    const wp::int32 var_674 = 0;
    wp::float32 var_675;
    wp::float32 var_676;
    wp::float32 var_677;
    const wp::int32 var_678 = 2;
    wp::int32 var_679;
    const wp::float32 var_680 = 1.0;
    const wp::float32 var_681 = -1.0;
    wp::float32 var_682;
    wp::float32 var_683;
    const wp::int32 var_684 = 4;
    wp::int32 var_685;
    const wp::int32 var_686 = 2;
    wp::float32 var_687;
    const wp::float32 var_688 = 1.0;
    const wp::int32 var_689 = 2;
    wp::float32 var_690;
    wp::float32 var_691;
    wp::float32 var_692;
    wp::float32 var_693;
    const wp::int32 var_694 = 1;
    wp::float32 var_695;
    wp::float32 var_696;
    const wp::int32 var_697 = 1;
    wp::int32 var_698;
    const wp::int32 var_699 = 0;
    wp::float32 var_700;
    const wp::float32 var_701 = 1.0;
    const wp::int32 var_702 = 0;
    wp::float32 var_703;
    wp::float32 var_704;
    wp::float32 var_705;
    const wp::int32 var_706 = 2;
    wp::int32 var_707;
    const wp::int32 var_708 = 1;
    wp::float32 var_709;
    const wp::float32 var_710 = 1.0;
    const wp::int32 var_711 = 1;
    wp::float32 var_712;
    wp::float32 var_713;
    wp::float32 var_714;
    wp::float32 var_715;
    const wp::int32 var_716 = 4;
    wp::int32 var_717;
    const wp::float32 var_718 = 1.0;
    const wp::float32 var_719 = -1.0;
    wp::float32 var_720;
    wp::float32 var_721;
    const wp::int32 var_722 = 2;
    wp::float32 var_723;
    wp::float32 var_724;
    const wp::int32 var_725 = 4;
    const wp::int32 var_726 = 1;
    wp::int32 var_727;
    const wp::float32 var_728 = 1.0;
    const wp::float32 var_729 = -1.0;
    wp::float32 var_730;
    const wp::int32 var_731 = 2;
    wp::int32 var_732;
    const wp::int32 var_733 = 1;
    wp::float32 var_734;
    const wp::float32 var_735 = 1.0;
    const wp::int32 var_736 = 1;
    wp::float32 var_737;
    wp::float32 var_738;
    wp::float32 var_739;
    wp::float32 var_740;
    const wp::int32 var_741 = 4;
    wp::int32 var_742;
    const wp::int32 var_743 = 2;
    wp::float32 var_744;
    const wp::float32 var_745 = 1.0;
    const wp::int32 var_746 = 2;
    wp::float32 var_747;
    wp::float32 var_748;
    wp::float32 var_749;
    wp::float32 var_750;
    const wp::int32 var_751 = 0;
    wp::float32 var_752;
    wp::float32 var_753;
    const wp::int32 var_754 = 1;
    wp::int32 var_755;
    const wp::int32 var_756 = 0;
    wp::float32 var_757;
    const wp::float32 var_758 = 1.0;
    const wp::int32 var_759 = 0;
    wp::float32 var_760;
    wp::float32 var_761;
    wp::float32 var_762;
    const wp::int32 var_763 = 2;
    wp::int32 var_764;
    const wp::float32 var_765 = 1.0;
    const wp::float32 var_766 = -1.0;
    wp::float32 var_767;
    wp::float32 var_768;
    const wp::int32 var_769 = 4;
    wp::int32 var_770;
    const wp::int32 var_771 = 2;
    wp::float32 var_772;
    const wp::float32 var_773 = 1.0;
    const wp::int32 var_774 = 2;
    wp::float32 var_775;
    wp::float32 var_776;
    wp::float32 var_777;
    wp::float32 var_778;
    const wp::int32 var_779 = 1;
    wp::float32 var_780;
    wp::float32 var_781;
    const wp::int32 var_782 = 1;
    wp::int32 var_783;
    const wp::int32 var_784 = 0;
    wp::float32 var_785;
    const wp::float32 var_786 = 1.0;
    const wp::int32 var_787 = 0;
    wp::float32 var_788;
    wp::float32 var_789;
    wp::float32 var_790;
    const wp::int32 var_791 = 2;
    wp::int32 var_792;
    const wp::int32 var_793 = 1;
    wp::float32 var_794;
    const wp::float32 var_795 = 1.0;
    const wp::int32 var_796 = 1;
    wp::float32 var_797;
    wp::float32 var_798;
    wp::float32 var_799;
    wp::float32 var_800;
    const wp::int32 var_801 = 4;
    wp::int32 var_802;
    const wp::float32 var_803 = 1.0;
    const wp::float32 var_804 = -1.0;
    wp::float32 var_805;
    wp::float32 var_806;
    const wp::int32 var_807 = 2;
    wp::float32 var_808;
    wp::float32 var_809;
    const wp::int32 var_810 = 5;
    const wp::int32 var_811 = 1;
    wp::int32 var_812;
    const wp::float32 var_813 = 1.0;
    const wp::float32 var_814 = -1.0;
    wp::float32 var_815;
    const wp::int32 var_816 = 2;
    wp::int32 var_817;
    const wp::int32 var_818 = 1;
    wp::float32 var_819;
    const wp::float32 var_820 = 1.0;
    const wp::int32 var_821 = 1;
    wp::float32 var_822;
    wp::float32 var_823;
    wp::float32 var_824;
    wp::float32 var_825;
    const wp::int32 var_826 = 4;
    wp::int32 var_827;
    const wp::int32 var_828 = 2;
    wp::float32 var_829;
    const wp::float32 var_830 = 1.0;
    const wp::int32 var_831 = 2;
    wp::float32 var_832;
    wp::float32 var_833;
    wp::float32 var_834;
    wp::float32 var_835;
    const wp::int32 var_836 = 0;
    wp::float32 var_837;
    wp::float32 var_838;
    const wp::int32 var_839 = 1;
    wp::int32 var_840;
    const wp::int32 var_841 = 0;
    wp::float32 var_842;
    const wp::float32 var_843 = 1.0;
    const wp::int32 var_844 = 0;
    wp::float32 var_845;
    wp::float32 var_846;
    wp::float32 var_847;
    const wp::int32 var_848 = 2;
    wp::int32 var_849;
    const wp::float32 var_850 = 1.0;
    const wp::float32 var_851 = -1.0;
    wp::float32 var_852;
    wp::float32 var_853;
    const wp::int32 var_854 = 4;
    wp::int32 var_855;
    const wp::int32 var_856 = 2;
    wp::float32 var_857;
    const wp::float32 var_858 = 1.0;
    const wp::int32 var_859 = 2;
    wp::float32 var_860;
    wp::float32 var_861;
    wp::float32 var_862;
    wp::float32 var_863;
    const wp::int32 var_864 = 1;
    wp::float32 var_865;
    wp::float32 var_866;
    const wp::int32 var_867 = 1;
    wp::int32 var_868;
    const wp::int32 var_869 = 0;
    wp::float32 var_870;
    const wp::float32 var_871 = 1.0;
    const wp::int32 var_872 = 0;
    wp::float32 var_873;
    wp::float32 var_874;
    wp::float32 var_875;
    const wp::int32 var_876 = 2;
    wp::int32 var_877;
    const wp::int32 var_878 = 1;
    wp::float32 var_879;
    const wp::float32 var_880 = 1.0;
    const wp::int32 var_881 = 1;
    wp::float32 var_882;
    wp::float32 var_883;
    wp::float32 var_884;
    wp::float32 var_885;
    const wp::int32 var_886 = 4;
    wp::int32 var_887;
    const wp::float32 var_888 = 1.0;
    const wp::float32 var_889 = -1.0;
    wp::float32 var_890;
    wp::float32 var_891;
    const wp::int32 var_892 = 2;
    wp::float32 var_893;
    wp::float32 var_894;
    const wp::int32 var_895 = 6;
    const wp::int32 var_896 = 1;
    wp::int32 var_897;
    const wp::float32 var_898 = 1.0;
    const wp::float32 var_899 = -1.0;
    wp::float32 var_900;
    const wp::int32 var_901 = 2;
    wp::int32 var_902;
    const wp::int32 var_903 = 1;
    wp::float32 var_904;
    const wp::float32 var_905 = 1.0;
    const wp::int32 var_906 = 1;
    wp::float32 var_907;
    wp::float32 var_908;
    wp::float32 var_909;
    wp::float32 var_910;
    const wp::int32 var_911 = 4;
    wp::int32 var_912;
    const wp::int32 var_913 = 2;
    wp::float32 var_914;
    const wp::float32 var_915 = 1.0;
    const wp::int32 var_916 = 2;
    wp::float32 var_917;
    wp::float32 var_918;
    wp::float32 var_919;
    wp::float32 var_920;
    const wp::int32 var_921 = 0;
    wp::float32 var_922;
    wp::float32 var_923;
    const wp::int32 var_924 = 1;
    wp::int32 var_925;
    const wp::int32 var_926 = 0;
    wp::float32 var_927;
    const wp::float32 var_928 = 1.0;
    const wp::int32 var_929 = 0;
    wp::float32 var_930;
    wp::float32 var_931;
    wp::float32 var_932;
    const wp::int32 var_933 = 2;
    wp::int32 var_934;
    const wp::float32 var_935 = 1.0;
    const wp::float32 var_936 = -1.0;
    wp::float32 var_937;
    wp::float32 var_938;
    const wp::int32 var_939 = 4;
    wp::int32 var_940;
    const wp::int32 var_941 = 2;
    wp::float32 var_942;
    const wp::float32 var_943 = 1.0;
    const wp::int32 var_944 = 2;
    wp::float32 var_945;
    wp::float32 var_946;
    wp::float32 var_947;
    wp::float32 var_948;
    const wp::int32 var_949 = 1;
    wp::float32 var_950;
    wp::float32 var_951;
    const wp::int32 var_952 = 1;
    wp::int32 var_953;
    const wp::int32 var_954 = 0;
    wp::float32 var_955;
    const wp::float32 var_956 = 1.0;
    const wp::int32 var_957 = 0;
    wp::float32 var_958;
    wp::float32 var_959;
    wp::float32 var_960;
    const wp::int32 var_961 = 2;
    wp::int32 var_962;
    const wp::int32 var_963 = 1;
    wp::float32 var_964;
    const wp::float32 var_965 = 1.0;
    const wp::int32 var_966 = 1;
    wp::float32 var_967;
    wp::float32 var_968;
    wp::float32 var_969;
    wp::float32 var_970;
    const wp::int32 var_971 = 4;
    wp::int32 var_972;
    const wp::float32 var_973 = 1.0;
    const wp::float32 var_974 = -1.0;
    wp::float32 var_975;
    wp::float32 var_976;
    const wp::int32 var_977 = 2;
    wp::float32 var_978;
    wp::float32 var_979;
    const wp::int32 var_980 = 7;
    const wp::int32 var_981 = 1;
    wp::int32 var_982;
    const wp::float32 var_983 = 1.0;
    const wp::float32 var_984 = -1.0;
    wp::float32 var_985;
    const wp::int32 var_986 = 2;
    wp::int32 var_987;
    const wp::int32 var_988 = 1;
    wp::float32 var_989;
    const wp::float32 var_990 = 1.0;
    const wp::int32 var_991 = 1;
    wp::float32 var_992;
    wp::float32 var_993;
    wp::float32 var_994;
    wp::float32 var_995;
    const wp::int32 var_996 = 4;
    wp::int32 var_997;
    const wp::int32 var_998 = 2;
    wp::float32 var_999;
    const wp::float32 var_1000 = 1.0;
    const wp::int32 var_1001 = 2;
    wp::float32 var_1002;
    wp::float32 var_1003;
    wp::float32 var_1004;
    wp::float32 var_1005;
    const wp::int32 var_1006 = 0;
    wp::float32 var_1007;
    wp::float32 var_1008;
    const wp::int32 var_1009 = 1;
    wp::int32 var_1010;
    const wp::int32 var_1011 = 0;
    wp::float32 var_1012;
    const wp::float32 var_1013 = 1.0;
    const wp::int32 var_1014 = 0;
    wp::float32 var_1015;
    wp::float32 var_1016;
    wp::float32 var_1017;
    const wp::int32 var_1018 = 2;
    wp::int32 var_1019;
    const wp::float32 var_1020 = 1.0;
    const wp::float32 var_1021 = -1.0;
    wp::float32 var_1022;
    wp::float32 var_1023;
    const wp::int32 var_1024 = 4;
    wp::int32 var_1025;
    const wp::int32 var_1026 = 2;
    wp::float32 var_1027;
    const wp::float32 var_1028 = 1.0;
    const wp::int32 var_1029 = 2;
    wp::float32 var_1030;
    wp::float32 var_1031;
    wp::float32 var_1032;
    wp::float32 var_1033;
    const wp::int32 var_1034 = 1;
    wp::float32 var_1035;
    wp::float32 var_1036;
    const wp::int32 var_1037 = 1;
    wp::int32 var_1038;
    const wp::int32 var_1039 = 0;
    wp::float32 var_1040;
    const wp::float32 var_1041 = 1.0;
    const wp::int32 var_1042 = 0;
    wp::float32 var_1043;
    wp::float32 var_1044;
    wp::float32 var_1045;
    const wp::int32 var_1046 = 2;
    wp::int32 var_1047;
    const wp::int32 var_1048 = 1;
    wp::float32 var_1049;
    const wp::float32 var_1050 = 1.0;
    const wp::int32 var_1051 = 1;
    wp::float32 var_1052;
    wp::float32 var_1053;
    wp::float32 var_1054;
    wp::float32 var_1055;
    const wp::int32 var_1056 = 4;
    wp::int32 var_1057;
    const wp::float32 var_1058 = 1.0;
    const wp::float32 var_1059 = -1.0;
    wp::float32 var_1060;
    wp::float32 var_1061;
    const wp::int32 var_1062 = 2;
    wp::float32 var_1063;
    wp::float32 var_1064;
    wp::int32 var_1065;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_1066;
    const wp::int32 var_1067 = 0;
    wp::float32 var_1068;
    const wp::float32 var_1069 = 0.5;
    bool var_1070;
    const wp::int32 var_1071 = 0;
    const wp::int32 var_1072 = 1;
    wp::int32 var_1073;
    const wp::int32 var_1074 = 1;
    wp::float32 var_1075;
    const wp::float32 var_1076 = 0.5;
    bool var_1077;
    const wp::int32 var_1078 = 0;
    const wp::int32 var_1079 = 1;
    wp::int32 var_1080;
    const wp::int32 var_1081 = 2;
    wp::float32 var_1082;
    const wp::float32 var_1083 = 0.5;
    bool var_1084;
    const wp::int32 var_1085 = 0;
    const wp::int32 var_1086 = 1;
    wp::int32 var_1087;
    wp::vec_t<8, wp::int32>* var_1088;
    const wp::int32 var_1089 = 4;
    wp::int32 var_1090;
    const wp::int32 var_1091 = 2;
    wp::int32 var_1092;
    wp::int32 var_1093;
    wp::int32 var_1094;
    wp::int32 var_1095;
    wp::vec_t<8, wp::int32> var_1096;
    const wp::int32 var_1097 = -1;
    bool var_1098;
    wp::int32 var_1099;
    const wp::int32 var_1100 = -1;
    wp::int32 var_1101;
    const wp::str var_1102 = "ERROR: Node not found\n";
    const wp::int32 var_1103 = -1;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_1104;
    //---------
    // forward
    // def find_oct(                                                                          <L 319>
    // stack = root                                                                           <L 322>
    var_0 = wp::copy(var_root);
    // niter = int(100)                                                                       <L 323>
    var_2 = wp::int(var_1);
    // rx = vec8(0.0)                                                                         <L 324>
    var_4 = wp::vec_t<8, wp::float32>(var_3);
    // ry = vec8(0.0)                                                                         <L 325>
    var_6 = wp::vec_t<8, wp::float32>(var_5);
    // rz = vec8(0.0)                                                                         <L 326>
    var_8 = wp::vec_t<8, wp::float32>(var_7);
    // eps = 1e-6                                                                             <L 327>
    // while niter > 0:                                                                       <L 329>
    start_while_0:;
    var_11 = (var_2 > var_10);
    if ((var_11) == false) goto end_while_0;
        // niter -= 1                                                                         <L 330>
        var_13 = wp::sub(var_2, var_12);
        // node = stack                                                                       <L 331>
        var_14 = wp::copy(var_0);
        // if node == -1:                                                                     <L 333>
        var_16 = (var_14 == var_15);
        if (var_16) {
            // wp.printf("ERROR: Invalid node number\n")                                      <L 334>
            printf(var_17);
            // return -1, (rx, ry, rz)                                                        <L 335>
            var_19 = wp::tuple(var_4, var_6, var_8);
            ret_0 = var_18;
            ret_1 = var_19;
            return;
        }
        // vmin = oct_aabb[node, 0] - oct_aabb[node, 1]                                       <L 337>
        var_21 = wp::address(var_oct_aabb, var_14, var_20);
        var_23 = wp::address(var_oct_aabb, var_14, var_22);
        var_25 = wp::load(var_21);
        var_26 = wp::load(var_23);
        var_24 = wp::sub(var_25, var_26);
        // vmax = oct_aabb[node, 0] + oct_aabb[node, 1]                                       <L 338>
        var_28 = wp::address(var_oct_aabb, var_14, var_27);
        var_30 = wp::address(var_oct_aabb, var_14, var_29);
        var_32 = wp::load(var_28);
        var_33 = wp::load(var_30);
        var_31 = wp::add(var_32, var_33);
        // if (                                                                               <L 340>
        // p[0] + eps < vmin[0]                                                               <L 341>
        var_36 = wp::extract(var_p, var_35);
        var_37 = wp::add(var_36, var_9);
        var_39 = wp::extract(var_24, var_38);
        var_40 = (var_37 < var_39);
        var_34 = var_40;
        if (!var_34) {
            // or p[0] - eps > vmax[0]                                                        <L 342>
            var_42 = wp::extract(var_p, var_41);
            var_43 = wp::sub(var_42, var_9);
            var_45 = wp::extract(var_31, var_44);
            var_46 = (var_43 > var_45);
            var_34 = var_34 || var_46;
        }
        if (!var_34) {
            // or p[1] + eps < vmin[1]                                                        <L 343>
            var_48 = wp::extract(var_p, var_47);
            var_49 = wp::add(var_48, var_9);
            var_51 = wp::extract(var_24, var_50);
            var_52 = (var_49 < var_51);
            var_34 = var_34 || var_52;
        }
        if (!var_34) {
            // or p[1] - eps > vmax[1]                                                        <L 344>
            var_54 = wp::extract(var_p, var_53);
            var_55 = wp::sub(var_54, var_9);
            var_57 = wp::extract(var_31, var_56);
            var_58 = (var_55 > var_57);
            var_34 = var_34 || var_58;
        }
        if (!var_34) {
            // or p[2] + eps < vmin[2]                                                        <L 345>
            var_60 = wp::extract(var_p, var_59);
            var_61 = wp::add(var_60, var_9);
            var_63 = wp::extract(var_24, var_62);
            var_64 = (var_61 < var_63);
            var_34 = var_34 || var_64;
        }
        if (!var_34) {
            // or p[2] - eps > vmax[2]                                                        <L 346>
            var_66 = wp::extract(var_p, var_65);
            var_67 = wp::sub(var_66, var_9);
            var_69 = wp::extract(var_31, var_68);
            var_70 = (var_67 > var_69);
            var_34 = var_34 || var_70;
        }
        if (var_34) {
            // continue                                                                       <L 348>
            wp::assign(var_2, var_13);
            goto start_while_0;
        }
        var_71 = wp::where(var_34, var_2, var_13);
        // coord = wp.cw_div(p - vmin, vmax - vmin)                                           <L 350>
        var_72 = wp::sub(var_p, var_24);
        var_73 = wp::sub(var_31, var_24);
        var_74 = wp::cw_div(var_72, var_73);
        // child0 = oct_child[node][0]                                                        <L 354>
        var_75 = wp::address(var_oct_child, var_14);
        var_78 = wp::load(var_75);
        var_77 = wp::extract(var_78, var_76);
        // if (                                                                               <L 356>
        // int(child0 == -1)                                                                  <L 357>
        var_80 = (var_77 == var_79);
        var_81 = wp::int(var_80);
        // & int(oct_child[node][1] == -1)                                                    <L 358>
        var_82 = wp::address(var_oct_child, var_14);
        var_85 = wp::load(var_82);
        var_84 = wp::extract(var_85, var_83);
        var_87 = (var_84 == var_86);
        var_88 = wp::int(var_87);
        var_89 = wp::bit_and(var_81, var_88);
        // & int(oct_child[node][2] == -1)                                                    <L 359>
        var_90 = wp::address(var_oct_child, var_14);
        var_93 = wp::load(var_90);
        var_92 = wp::extract(var_93, var_91);
        var_95 = (var_92 == var_94);
        var_96 = wp::int(var_95);
        var_97 = wp::bit_and(var_89, var_96);
        // & int(oct_child[node][3] == -1)                                                    <L 360>
        var_98 = wp::address(var_oct_child, var_14);
        var_101 = wp::load(var_98);
        var_100 = wp::extract(var_101, var_99);
        var_103 = (var_100 == var_102);
        var_104 = wp::int(var_103);
        var_105 = wp::bit_and(var_97, var_104);
        // & int(oct_child[node][4] == -1)                                                    <L 361>
        var_106 = wp::address(var_oct_child, var_14);
        var_109 = wp::load(var_106);
        var_108 = wp::extract(var_109, var_107);
        var_111 = (var_108 == var_110);
        var_112 = wp::int(var_111);
        var_113 = wp::bit_and(var_105, var_112);
        // & int(oct_child[node][5] == -1)                                                    <L 362>
        var_114 = wp::address(var_oct_child, var_14);
        var_117 = wp::load(var_114);
        var_116 = wp::extract(var_117, var_115);
        var_119 = (var_116 == var_118);
        var_120 = wp::int(var_119);
        var_121 = wp::bit_and(var_113, var_120);
        // & int(oct_child[node][6] == -1)                                                    <L 363>
        var_122 = wp::address(var_oct_child, var_14);
        var_125 = wp::load(var_122);
        var_124 = wp::extract(var_125, var_123);
        var_127 = (var_124 == var_126);
        var_128 = wp::int(var_127);
        var_129 = wp::bit_and(var_121, var_128);
        // & int(oct_child[node][7] == -1)                                                    <L 364>
        var_130 = wp::address(var_oct_child, var_14);
        var_133 = wp::load(var_130);
        var_132 = wp::extract(var_133, var_131);
        var_135 = (var_132 == var_134);
        var_136 = wp::int(var_135);
        var_137 = wp::bit_and(var_129, var_136);
        // ) != 0:                                                                            <L 365>
        var_139 = (var_137 != var_138);
        if (var_139) {
            // if not grad:                                                                   <L 366>
            var_140 = wp::unot(var_grad);
            if (var_140) {
                // for j in range(8):                                                         <L 367>
                // rx[j] = (                                                                  <L 368>
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_143 = wp::bit_and(var_141, var_142);
                if (var_143) {
                    var_145 = wp::extract(var_74, var_144);
                }
                if (!var_143) {
                    var_148 = wp::extract(var_74, var_147);
                    var_149 = wp::sub(var_146, var_148);
                }
                var_150 = wp::where(var_143, var_145, var_149);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_152 = wp::bit_and(var_141, var_151);
                if (var_152) {
                    var_154 = wp::extract(var_74, var_153);
                }
                if (!var_152) {
                    var_157 = wp::extract(var_74, var_156);
                    var_158 = wp::sub(var_155, var_157);
                }
                var_159 = wp::where(var_152, var_154, var_158);
                var_160 = wp::mul(var_150, var_159);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_162 = wp::bit_and(var_141, var_161);
                if (var_162) {
                    var_164 = wp::extract(var_74, var_163);
                }
                if (!var_162) {
                    var_167 = wp::extract(var_74, var_166);
                    var_168 = wp::sub(var_165, var_167);
                }
                var_169 = wp::where(var_162, var_164, var_168);
                var_170 = wp::mul(var_160, var_169);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_141, var_170);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_173 = wp::bit_and(var_171, var_172);
                if (var_173) {
                    var_175 = wp::extract(var_74, var_174);
                }
                if (!var_173) {
                    var_178 = wp::extract(var_74, var_177);
                    var_179 = wp::sub(var_176, var_178);
                }
                var_180 = wp::where(var_173, var_175, var_179);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_182 = wp::bit_and(var_171, var_181);
                if (var_182) {
                    var_184 = wp::extract(var_74, var_183);
                }
                if (!var_182) {
                    var_187 = wp::extract(var_74, var_186);
                    var_188 = wp::sub(var_185, var_187);
                }
                var_189 = wp::where(var_182, var_184, var_188);
                var_190 = wp::mul(var_180, var_189);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_192 = wp::bit_and(var_171, var_191);
                if (var_192) {
                    var_194 = wp::extract(var_74, var_193);
                }
                if (!var_192) {
                    var_197 = wp::extract(var_74, var_196);
                    var_198 = wp::sub(var_195, var_197);
                }
                var_199 = wp::where(var_192, var_194, var_198);
                var_200 = wp::mul(var_190, var_199);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_171, var_200);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_203 = wp::bit_and(var_201, var_202);
                if (var_203) {
                    var_205 = wp::extract(var_74, var_204);
                }
                if (!var_203) {
                    var_208 = wp::extract(var_74, var_207);
                    var_209 = wp::sub(var_206, var_208);
                }
                var_210 = wp::where(var_203, var_205, var_209);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_212 = wp::bit_and(var_201, var_211);
                if (var_212) {
                    var_214 = wp::extract(var_74, var_213);
                }
                if (!var_212) {
                    var_217 = wp::extract(var_74, var_216);
                    var_218 = wp::sub(var_215, var_217);
                }
                var_219 = wp::where(var_212, var_214, var_218);
                var_220 = wp::mul(var_210, var_219);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_222 = wp::bit_and(var_201, var_221);
                if (var_222) {
                    var_224 = wp::extract(var_74, var_223);
                }
                if (!var_222) {
                    var_227 = wp::extract(var_74, var_226);
                    var_228 = wp::sub(var_225, var_227);
                }
                var_229 = wp::where(var_222, var_224, var_228);
                var_230 = wp::mul(var_220, var_229);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_201, var_230);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_233 = wp::bit_and(var_231, var_232);
                if (var_233) {
                    var_235 = wp::extract(var_74, var_234);
                }
                if (!var_233) {
                    var_238 = wp::extract(var_74, var_237);
                    var_239 = wp::sub(var_236, var_238);
                }
                var_240 = wp::where(var_233, var_235, var_239);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_242 = wp::bit_and(var_231, var_241);
                if (var_242) {
                    var_244 = wp::extract(var_74, var_243);
                }
                if (!var_242) {
                    var_247 = wp::extract(var_74, var_246);
                    var_248 = wp::sub(var_245, var_247);
                }
                var_249 = wp::where(var_242, var_244, var_248);
                var_250 = wp::mul(var_240, var_249);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_252 = wp::bit_and(var_231, var_251);
                if (var_252) {
                    var_254 = wp::extract(var_74, var_253);
                }
                if (!var_252) {
                    var_257 = wp::extract(var_74, var_256);
                    var_258 = wp::sub(var_255, var_257);
                }
                var_259 = wp::where(var_252, var_254, var_258);
                var_260 = wp::mul(var_250, var_259);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_231, var_260);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_263 = wp::bit_and(var_261, var_262);
                if (var_263) {
                    var_265 = wp::extract(var_74, var_264);
                }
                if (!var_263) {
                    var_268 = wp::extract(var_74, var_267);
                    var_269 = wp::sub(var_266, var_268);
                }
                var_270 = wp::where(var_263, var_265, var_269);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_272 = wp::bit_and(var_261, var_271);
                if (var_272) {
                    var_274 = wp::extract(var_74, var_273);
                }
                if (!var_272) {
                    var_277 = wp::extract(var_74, var_276);
                    var_278 = wp::sub(var_275, var_277);
                }
                var_279 = wp::where(var_272, var_274, var_278);
                var_280 = wp::mul(var_270, var_279);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_282 = wp::bit_and(var_261, var_281);
                if (var_282) {
                    var_284 = wp::extract(var_74, var_283);
                }
                if (!var_282) {
                    var_287 = wp::extract(var_74, var_286);
                    var_288 = wp::sub(var_285, var_287);
                }
                var_289 = wp::where(var_282, var_284, var_288);
                var_290 = wp::mul(var_280, var_289);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_261, var_290);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_293 = wp::bit_and(var_291, var_292);
                if (var_293) {
                    var_295 = wp::extract(var_74, var_294);
                }
                if (!var_293) {
                    var_298 = wp::extract(var_74, var_297);
                    var_299 = wp::sub(var_296, var_298);
                }
                var_300 = wp::where(var_293, var_295, var_299);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_302 = wp::bit_and(var_291, var_301);
                if (var_302) {
                    var_304 = wp::extract(var_74, var_303);
                }
                if (!var_302) {
                    var_307 = wp::extract(var_74, var_306);
                    var_308 = wp::sub(var_305, var_307);
                }
                var_309 = wp::where(var_302, var_304, var_308);
                var_310 = wp::mul(var_300, var_309);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_312 = wp::bit_and(var_291, var_311);
                if (var_312) {
                    var_314 = wp::extract(var_74, var_313);
                }
                if (!var_312) {
                    var_317 = wp::extract(var_74, var_316);
                    var_318 = wp::sub(var_315, var_317);
                }
                var_319 = wp::where(var_312, var_314, var_318);
                var_320 = wp::mul(var_310, var_319);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_291, var_320);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_323 = wp::bit_and(var_321, var_322);
                if (var_323) {
                    var_325 = wp::extract(var_74, var_324);
                }
                if (!var_323) {
                    var_328 = wp::extract(var_74, var_327);
                    var_329 = wp::sub(var_326, var_328);
                }
                var_330 = wp::where(var_323, var_325, var_329);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_332 = wp::bit_and(var_321, var_331);
                if (var_332) {
                    var_334 = wp::extract(var_74, var_333);
                }
                if (!var_332) {
                    var_337 = wp::extract(var_74, var_336);
                    var_338 = wp::sub(var_335, var_337);
                }
                var_339 = wp::where(var_332, var_334, var_338);
                var_340 = wp::mul(var_330, var_339);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_342 = wp::bit_and(var_321, var_341);
                if (var_342) {
                    var_344 = wp::extract(var_74, var_343);
                }
                if (!var_342) {
                    var_347 = wp::extract(var_74, var_346);
                    var_348 = wp::sub(var_345, var_347);
                }
                var_349 = wp::where(var_342, var_344, var_348);
                var_350 = wp::mul(var_340, var_349);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_321, var_350);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_353 = wp::bit_and(var_351, var_352);
                if (var_353) {
                    var_355 = wp::extract(var_74, var_354);
                }
                if (!var_353) {
                    var_358 = wp::extract(var_74, var_357);
                    var_359 = wp::sub(var_356, var_358);
                }
                var_360 = wp::where(var_353, var_355, var_359);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_362 = wp::bit_and(var_351, var_361);
                if (var_362) {
                    var_364 = wp::extract(var_74, var_363);
                }
                if (!var_362) {
                    var_367 = wp::extract(var_74, var_366);
                    var_368 = wp::sub(var_365, var_367);
                }
                var_369 = wp::where(var_362, var_364, var_368);
                var_370 = wp::mul(var_360, var_369);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_372 = wp::bit_and(var_351, var_371);
                if (var_372) {
                    var_374 = wp::extract(var_74, var_373);
                }
                if (!var_372) {
                    var_377 = wp::extract(var_74, var_376);
                    var_378 = wp::sub(var_375, var_377);
                }
                var_379 = wp::where(var_372, var_374, var_378);
                var_380 = wp::mul(var_370, var_379);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_351, var_380);
            }
            if (!var_140) {
                // inv_cell = wp.cw_div(wp.vec3(1.0), vmax - vmin)                            <L 375>
                var_382 = wp::vec_t<3, wp::float32>(var_381);
                var_383 = wp::sub(var_31, var_24);
                var_384 = wp::cw_div(var_382, var_383);
                // for j in range(8):                                                         <L 376>
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_387 = wp::bit_and(var_385, var_386);
                if (var_387) {
                }
                if (!var_387) {
                }
                var_390 = wp::where(var_387, var_388, var_389);
                var_392 = wp::bit_and(var_385, var_391);
                if (var_392) {
                    var_394 = wp::extract(var_74, var_393);
                }
                if (!var_392) {
                    var_397 = wp::extract(var_74, var_396);
                    var_398 = wp::sub(var_395, var_397);
                }
                var_399 = wp::where(var_392, var_394, var_398);
                var_400 = wp::mul(var_390, var_399);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_402 = wp::bit_and(var_385, var_401);
                if (var_402) {
                    var_404 = wp::extract(var_74, var_403);
                }
                if (!var_402) {
                    var_407 = wp::extract(var_74, var_406);
                    var_408 = wp::sub(var_405, var_407);
                }
                var_409 = wp::where(var_402, var_404, var_408);
                var_410 = wp::mul(var_400, var_409);
                var_412 = wp::extract(var_384, var_411);
                var_413 = wp::mul(var_410, var_412);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_385, var_413);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_415 = wp::bit_and(var_385, var_414);
                if (var_415) {
                    var_417 = wp::extract(var_74, var_416);
                }
                if (!var_415) {
                    var_420 = wp::extract(var_74, var_419);
                    var_421 = wp::sub(var_418, var_420);
                }
                var_422 = wp::where(var_415, var_417, var_421);
                var_424 = wp::bit_and(var_385, var_423);
                if (var_424) {
                }
                if (!var_424) {
                }
                var_427 = wp::where(var_424, var_425, var_426);
                var_428 = wp::mul(var_422, var_427);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_430 = wp::bit_and(var_385, var_429);
                if (var_430) {
                    var_432 = wp::extract(var_74, var_431);
                }
                if (!var_430) {
                    var_435 = wp::extract(var_74, var_434);
                    var_436 = wp::sub(var_433, var_435);
                }
                var_437 = wp::where(var_430, var_432, var_436);
                var_438 = wp::mul(var_428, var_437);
                var_440 = wp::extract(var_384, var_439);
                var_441 = wp::mul(var_438, var_440);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_385, var_441);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_443 = wp::bit_and(var_385, var_442);
                if (var_443) {
                    var_445 = wp::extract(var_74, var_444);
                }
                if (!var_443) {
                    var_448 = wp::extract(var_74, var_447);
                    var_449 = wp::sub(var_446, var_448);
                }
                var_450 = wp::where(var_443, var_445, var_449);
                var_452 = wp::bit_and(var_385, var_451);
                if (var_452) {
                    var_454 = wp::extract(var_74, var_453);
                }
                if (!var_452) {
                    var_457 = wp::extract(var_74, var_456);
                    var_458 = wp::sub(var_455, var_457);
                }
                var_459 = wp::where(var_452, var_454, var_458);
                var_460 = wp::mul(var_450, var_459);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_462 = wp::bit_and(var_385, var_461);
                if (var_462) {
                }
                if (!var_462) {
                }
                var_465 = wp::where(var_462, var_463, var_464);
                var_466 = wp::mul(var_460, var_465);
                var_468 = wp::extract(var_384, var_467);
                var_469 = wp::mul(var_466, var_468);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_385, var_469);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_472 = wp::bit_and(var_470, var_471);
                if (var_472) {
                }
                if (!var_472) {
                }
                var_475 = wp::where(var_472, var_473, var_474);
                var_477 = wp::bit_and(var_470, var_476);
                if (var_477) {
                    var_479 = wp::extract(var_74, var_478);
                }
                if (!var_477) {
                    var_482 = wp::extract(var_74, var_481);
                    var_483 = wp::sub(var_480, var_482);
                }
                var_484 = wp::where(var_477, var_479, var_483);
                var_485 = wp::mul(var_475, var_484);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_487 = wp::bit_and(var_470, var_486);
                if (var_487) {
                    var_489 = wp::extract(var_74, var_488);
                }
                if (!var_487) {
                    var_492 = wp::extract(var_74, var_491);
                    var_493 = wp::sub(var_490, var_492);
                }
                var_494 = wp::where(var_487, var_489, var_493);
                var_495 = wp::mul(var_485, var_494);
                var_497 = wp::extract(var_384, var_496);
                var_498 = wp::mul(var_495, var_497);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_470, var_498);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_500 = wp::bit_and(var_470, var_499);
                if (var_500) {
                    var_502 = wp::extract(var_74, var_501);
                }
                if (!var_500) {
                    var_505 = wp::extract(var_74, var_504);
                    var_506 = wp::sub(var_503, var_505);
                }
                var_507 = wp::where(var_500, var_502, var_506);
                var_509 = wp::bit_and(var_470, var_508);
                if (var_509) {
                }
                if (!var_509) {
                }
                var_512 = wp::where(var_509, var_510, var_511);
                var_513 = wp::mul(var_507, var_512);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_515 = wp::bit_and(var_470, var_514);
                if (var_515) {
                    var_517 = wp::extract(var_74, var_516);
                }
                if (!var_515) {
                    var_520 = wp::extract(var_74, var_519);
                    var_521 = wp::sub(var_518, var_520);
                }
                var_522 = wp::where(var_515, var_517, var_521);
                var_523 = wp::mul(var_513, var_522);
                var_525 = wp::extract(var_384, var_524);
                var_526 = wp::mul(var_523, var_525);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_470, var_526);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_528 = wp::bit_and(var_470, var_527);
                if (var_528) {
                    var_530 = wp::extract(var_74, var_529);
                }
                if (!var_528) {
                    var_533 = wp::extract(var_74, var_532);
                    var_534 = wp::sub(var_531, var_533);
                }
                var_535 = wp::where(var_528, var_530, var_534);
                var_537 = wp::bit_and(var_470, var_536);
                if (var_537) {
                    var_539 = wp::extract(var_74, var_538);
                }
                if (!var_537) {
                    var_542 = wp::extract(var_74, var_541);
                    var_543 = wp::sub(var_540, var_542);
                }
                var_544 = wp::where(var_537, var_539, var_543);
                var_545 = wp::mul(var_535, var_544);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_547 = wp::bit_and(var_470, var_546);
                if (var_547) {
                }
                if (!var_547) {
                }
                var_550 = wp::where(var_547, var_548, var_549);
                var_551 = wp::mul(var_545, var_550);
                var_553 = wp::extract(var_384, var_552);
                var_554 = wp::mul(var_551, var_553);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_470, var_554);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_557 = wp::bit_and(var_555, var_556);
                if (var_557) {
                }
                if (!var_557) {
                }
                var_560 = wp::where(var_557, var_558, var_559);
                var_562 = wp::bit_and(var_555, var_561);
                if (var_562) {
                    var_564 = wp::extract(var_74, var_563);
                }
                if (!var_562) {
                    var_567 = wp::extract(var_74, var_566);
                    var_568 = wp::sub(var_565, var_567);
                }
                var_569 = wp::where(var_562, var_564, var_568);
                var_570 = wp::mul(var_560, var_569);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_572 = wp::bit_and(var_555, var_571);
                if (var_572) {
                    var_574 = wp::extract(var_74, var_573);
                }
                if (!var_572) {
                    var_577 = wp::extract(var_74, var_576);
                    var_578 = wp::sub(var_575, var_577);
                }
                var_579 = wp::where(var_572, var_574, var_578);
                var_580 = wp::mul(var_570, var_579);
                var_582 = wp::extract(var_384, var_581);
                var_583 = wp::mul(var_580, var_582);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_555, var_583);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_585 = wp::bit_and(var_555, var_584);
                if (var_585) {
                    var_587 = wp::extract(var_74, var_586);
                }
                if (!var_585) {
                    var_590 = wp::extract(var_74, var_589);
                    var_591 = wp::sub(var_588, var_590);
                }
                var_592 = wp::where(var_585, var_587, var_591);
                var_594 = wp::bit_and(var_555, var_593);
                if (var_594) {
                }
                if (!var_594) {
                }
                var_597 = wp::where(var_594, var_595, var_596);
                var_598 = wp::mul(var_592, var_597);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_600 = wp::bit_and(var_555, var_599);
                if (var_600) {
                    var_602 = wp::extract(var_74, var_601);
                }
                if (!var_600) {
                    var_605 = wp::extract(var_74, var_604);
                    var_606 = wp::sub(var_603, var_605);
                }
                var_607 = wp::where(var_600, var_602, var_606);
                var_608 = wp::mul(var_598, var_607);
                var_610 = wp::extract(var_384, var_609);
                var_611 = wp::mul(var_608, var_610);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_555, var_611);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_613 = wp::bit_and(var_555, var_612);
                if (var_613) {
                    var_615 = wp::extract(var_74, var_614);
                }
                if (!var_613) {
                    var_618 = wp::extract(var_74, var_617);
                    var_619 = wp::sub(var_616, var_618);
                }
                var_620 = wp::where(var_613, var_615, var_619);
                var_622 = wp::bit_and(var_555, var_621);
                if (var_622) {
                    var_624 = wp::extract(var_74, var_623);
                }
                if (!var_622) {
                    var_627 = wp::extract(var_74, var_626);
                    var_628 = wp::sub(var_625, var_627);
                }
                var_629 = wp::where(var_622, var_624, var_628);
                var_630 = wp::mul(var_620, var_629);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_632 = wp::bit_and(var_555, var_631);
                if (var_632) {
                }
                if (!var_632) {
                }
                var_635 = wp::where(var_632, var_633, var_634);
                var_636 = wp::mul(var_630, var_635);
                var_638 = wp::extract(var_384, var_637);
                var_639 = wp::mul(var_636, var_638);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_555, var_639);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_642 = wp::bit_and(var_640, var_641);
                if (var_642) {
                }
                if (!var_642) {
                }
                var_645 = wp::where(var_642, var_643, var_644);
                var_647 = wp::bit_and(var_640, var_646);
                if (var_647) {
                    var_649 = wp::extract(var_74, var_648);
                }
                if (!var_647) {
                    var_652 = wp::extract(var_74, var_651);
                    var_653 = wp::sub(var_650, var_652);
                }
                var_654 = wp::where(var_647, var_649, var_653);
                var_655 = wp::mul(var_645, var_654);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_657 = wp::bit_and(var_640, var_656);
                if (var_657) {
                    var_659 = wp::extract(var_74, var_658);
                }
                if (!var_657) {
                    var_662 = wp::extract(var_74, var_661);
                    var_663 = wp::sub(var_660, var_662);
                }
                var_664 = wp::where(var_657, var_659, var_663);
                var_665 = wp::mul(var_655, var_664);
                var_667 = wp::extract(var_384, var_666);
                var_668 = wp::mul(var_665, var_667);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_640, var_668);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_670 = wp::bit_and(var_640, var_669);
                if (var_670) {
                    var_672 = wp::extract(var_74, var_671);
                }
                if (!var_670) {
                    var_675 = wp::extract(var_74, var_674);
                    var_676 = wp::sub(var_673, var_675);
                }
                var_677 = wp::where(var_670, var_672, var_676);
                var_679 = wp::bit_and(var_640, var_678);
                if (var_679) {
                }
                if (!var_679) {
                }
                var_682 = wp::where(var_679, var_680, var_681);
                var_683 = wp::mul(var_677, var_682);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_685 = wp::bit_and(var_640, var_684);
                if (var_685) {
                    var_687 = wp::extract(var_74, var_686);
                }
                if (!var_685) {
                    var_690 = wp::extract(var_74, var_689);
                    var_691 = wp::sub(var_688, var_690);
                }
                var_692 = wp::where(var_685, var_687, var_691);
                var_693 = wp::mul(var_683, var_692);
                var_695 = wp::extract(var_384, var_694);
                var_696 = wp::mul(var_693, var_695);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_640, var_696);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_698 = wp::bit_and(var_640, var_697);
                if (var_698) {
                    var_700 = wp::extract(var_74, var_699);
                }
                if (!var_698) {
                    var_703 = wp::extract(var_74, var_702);
                    var_704 = wp::sub(var_701, var_703);
                }
                var_705 = wp::where(var_698, var_700, var_704);
                var_707 = wp::bit_and(var_640, var_706);
                if (var_707) {
                    var_709 = wp::extract(var_74, var_708);
                }
                if (!var_707) {
                    var_712 = wp::extract(var_74, var_711);
                    var_713 = wp::sub(var_710, var_712);
                }
                var_714 = wp::where(var_707, var_709, var_713);
                var_715 = wp::mul(var_705, var_714);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_717 = wp::bit_and(var_640, var_716);
                if (var_717) {
                }
                if (!var_717) {
                }
                var_720 = wp::where(var_717, var_718, var_719);
                var_721 = wp::mul(var_715, var_720);
                var_723 = wp::extract(var_384, var_722);
                var_724 = wp::mul(var_721, var_723);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_640, var_724);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_727 = wp::bit_and(var_725, var_726);
                if (var_727) {
                }
                if (!var_727) {
                }
                var_730 = wp::where(var_727, var_728, var_729);
                var_732 = wp::bit_and(var_725, var_731);
                if (var_732) {
                    var_734 = wp::extract(var_74, var_733);
                }
                if (!var_732) {
                    var_737 = wp::extract(var_74, var_736);
                    var_738 = wp::sub(var_735, var_737);
                }
                var_739 = wp::where(var_732, var_734, var_738);
                var_740 = wp::mul(var_730, var_739);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_742 = wp::bit_and(var_725, var_741);
                if (var_742) {
                    var_744 = wp::extract(var_74, var_743);
                }
                if (!var_742) {
                    var_747 = wp::extract(var_74, var_746);
                    var_748 = wp::sub(var_745, var_747);
                }
                var_749 = wp::where(var_742, var_744, var_748);
                var_750 = wp::mul(var_740, var_749);
                var_752 = wp::extract(var_384, var_751);
                var_753 = wp::mul(var_750, var_752);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_725, var_753);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_755 = wp::bit_and(var_725, var_754);
                if (var_755) {
                    var_757 = wp::extract(var_74, var_756);
                }
                if (!var_755) {
                    var_760 = wp::extract(var_74, var_759);
                    var_761 = wp::sub(var_758, var_760);
                }
                var_762 = wp::where(var_755, var_757, var_761);
                var_764 = wp::bit_and(var_725, var_763);
                if (var_764) {
                }
                if (!var_764) {
                }
                var_767 = wp::where(var_764, var_765, var_766);
                var_768 = wp::mul(var_762, var_767);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_770 = wp::bit_and(var_725, var_769);
                if (var_770) {
                    var_772 = wp::extract(var_74, var_771);
                }
                if (!var_770) {
                    var_775 = wp::extract(var_74, var_774);
                    var_776 = wp::sub(var_773, var_775);
                }
                var_777 = wp::where(var_770, var_772, var_776);
                var_778 = wp::mul(var_768, var_777);
                var_780 = wp::extract(var_384, var_779);
                var_781 = wp::mul(var_778, var_780);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_725, var_781);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_783 = wp::bit_and(var_725, var_782);
                if (var_783) {
                    var_785 = wp::extract(var_74, var_784);
                }
                if (!var_783) {
                    var_788 = wp::extract(var_74, var_787);
                    var_789 = wp::sub(var_786, var_788);
                }
                var_790 = wp::where(var_783, var_785, var_789);
                var_792 = wp::bit_and(var_725, var_791);
                if (var_792) {
                    var_794 = wp::extract(var_74, var_793);
                }
                if (!var_792) {
                    var_797 = wp::extract(var_74, var_796);
                    var_798 = wp::sub(var_795, var_797);
                }
                var_799 = wp::where(var_792, var_794, var_798);
                var_800 = wp::mul(var_790, var_799);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_802 = wp::bit_and(var_725, var_801);
                if (var_802) {
                }
                if (!var_802) {
                }
                var_805 = wp::where(var_802, var_803, var_804);
                var_806 = wp::mul(var_800, var_805);
                var_808 = wp::extract(var_384, var_807);
                var_809 = wp::mul(var_806, var_808);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_725, var_809);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_812 = wp::bit_and(var_810, var_811);
                if (var_812) {
                }
                if (!var_812) {
                }
                var_815 = wp::where(var_812, var_813, var_814);
                var_817 = wp::bit_and(var_810, var_816);
                if (var_817) {
                    var_819 = wp::extract(var_74, var_818);
                }
                if (!var_817) {
                    var_822 = wp::extract(var_74, var_821);
                    var_823 = wp::sub(var_820, var_822);
                }
                var_824 = wp::where(var_817, var_819, var_823);
                var_825 = wp::mul(var_815, var_824);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_827 = wp::bit_and(var_810, var_826);
                if (var_827) {
                    var_829 = wp::extract(var_74, var_828);
                }
                if (!var_827) {
                    var_832 = wp::extract(var_74, var_831);
                    var_833 = wp::sub(var_830, var_832);
                }
                var_834 = wp::where(var_827, var_829, var_833);
                var_835 = wp::mul(var_825, var_834);
                var_837 = wp::extract(var_384, var_836);
                var_838 = wp::mul(var_835, var_837);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_810, var_838);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_840 = wp::bit_and(var_810, var_839);
                if (var_840) {
                    var_842 = wp::extract(var_74, var_841);
                }
                if (!var_840) {
                    var_845 = wp::extract(var_74, var_844);
                    var_846 = wp::sub(var_843, var_845);
                }
                var_847 = wp::where(var_840, var_842, var_846);
                var_849 = wp::bit_and(var_810, var_848);
                if (var_849) {
                }
                if (!var_849) {
                }
                var_852 = wp::where(var_849, var_850, var_851);
                var_853 = wp::mul(var_847, var_852);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_855 = wp::bit_and(var_810, var_854);
                if (var_855) {
                    var_857 = wp::extract(var_74, var_856);
                }
                if (!var_855) {
                    var_860 = wp::extract(var_74, var_859);
                    var_861 = wp::sub(var_858, var_860);
                }
                var_862 = wp::where(var_855, var_857, var_861);
                var_863 = wp::mul(var_853, var_862);
                var_865 = wp::extract(var_384, var_864);
                var_866 = wp::mul(var_863, var_865);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_810, var_866);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_868 = wp::bit_and(var_810, var_867);
                if (var_868) {
                    var_870 = wp::extract(var_74, var_869);
                }
                if (!var_868) {
                    var_873 = wp::extract(var_74, var_872);
                    var_874 = wp::sub(var_871, var_873);
                }
                var_875 = wp::where(var_868, var_870, var_874);
                var_877 = wp::bit_and(var_810, var_876);
                if (var_877) {
                    var_879 = wp::extract(var_74, var_878);
                }
                if (!var_877) {
                    var_882 = wp::extract(var_74, var_881);
                    var_883 = wp::sub(var_880, var_882);
                }
                var_884 = wp::where(var_877, var_879, var_883);
                var_885 = wp::mul(var_875, var_884);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_887 = wp::bit_and(var_810, var_886);
                if (var_887) {
                }
                if (!var_887) {
                }
                var_890 = wp::where(var_887, var_888, var_889);
                var_891 = wp::mul(var_885, var_890);
                var_893 = wp::extract(var_384, var_892);
                var_894 = wp::mul(var_891, var_893);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_810, var_894);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_897 = wp::bit_and(var_895, var_896);
                if (var_897) {
                }
                if (!var_897) {
                }
                var_900 = wp::where(var_897, var_898, var_899);
                var_902 = wp::bit_and(var_895, var_901);
                if (var_902) {
                    var_904 = wp::extract(var_74, var_903);
                }
                if (!var_902) {
                    var_907 = wp::extract(var_74, var_906);
                    var_908 = wp::sub(var_905, var_907);
                }
                var_909 = wp::where(var_902, var_904, var_908);
                var_910 = wp::mul(var_900, var_909);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_912 = wp::bit_and(var_895, var_911);
                if (var_912) {
                    var_914 = wp::extract(var_74, var_913);
                }
                if (!var_912) {
                    var_917 = wp::extract(var_74, var_916);
                    var_918 = wp::sub(var_915, var_917);
                }
                var_919 = wp::where(var_912, var_914, var_918);
                var_920 = wp::mul(var_910, var_919);
                var_922 = wp::extract(var_384, var_921);
                var_923 = wp::mul(var_920, var_922);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_895, var_923);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_925 = wp::bit_and(var_895, var_924);
                if (var_925) {
                    var_927 = wp::extract(var_74, var_926);
                }
                if (!var_925) {
                    var_930 = wp::extract(var_74, var_929);
                    var_931 = wp::sub(var_928, var_930);
                }
                var_932 = wp::where(var_925, var_927, var_931);
                var_934 = wp::bit_and(var_895, var_933);
                if (var_934) {
                }
                if (!var_934) {
                }
                var_937 = wp::where(var_934, var_935, var_936);
                var_938 = wp::mul(var_932, var_937);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_940 = wp::bit_and(var_895, var_939);
                if (var_940) {
                    var_942 = wp::extract(var_74, var_941);
                }
                if (!var_940) {
                    var_945 = wp::extract(var_74, var_944);
                    var_946 = wp::sub(var_943, var_945);
                }
                var_947 = wp::where(var_940, var_942, var_946);
                var_948 = wp::mul(var_938, var_947);
                var_950 = wp::extract(var_384, var_949);
                var_951 = wp::mul(var_948, var_950);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_895, var_951);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_953 = wp::bit_and(var_895, var_952);
                if (var_953) {
                    var_955 = wp::extract(var_74, var_954);
                }
                if (!var_953) {
                    var_958 = wp::extract(var_74, var_957);
                    var_959 = wp::sub(var_956, var_958);
                }
                var_960 = wp::where(var_953, var_955, var_959);
                var_962 = wp::bit_and(var_895, var_961);
                if (var_962) {
                    var_964 = wp::extract(var_74, var_963);
                }
                if (!var_962) {
                    var_967 = wp::extract(var_74, var_966);
                    var_968 = wp::sub(var_965, var_967);
                }
                var_969 = wp::where(var_962, var_964, var_968);
                var_970 = wp::mul(var_960, var_969);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_972 = wp::bit_and(var_895, var_971);
                if (var_972) {
                }
                if (!var_972) {
                }
                var_975 = wp::where(var_972, var_973, var_974);
                var_976 = wp::mul(var_970, var_975);
                var_978 = wp::extract(var_384, var_977);
                var_979 = wp::mul(var_976, var_978);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_895, var_979);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_982 = wp::bit_and(var_980, var_981);
                if (var_982) {
                }
                if (!var_982) {
                }
                var_985 = wp::where(var_982, var_983, var_984);
                var_987 = wp::bit_and(var_980, var_986);
                if (var_987) {
                    var_989 = wp::extract(var_74, var_988);
                }
                if (!var_987) {
                    var_992 = wp::extract(var_74, var_991);
                    var_993 = wp::sub(var_990, var_992);
                }
                var_994 = wp::where(var_987, var_989, var_993);
                var_995 = wp::mul(var_985, var_994);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_997 = wp::bit_and(var_980, var_996);
                if (var_997) {
                    var_999 = wp::extract(var_74, var_998);
                }
                if (!var_997) {
                    var_1002 = wp::extract(var_74, var_1001);
                    var_1003 = wp::sub(var_1000, var_1002);
                }
                var_1004 = wp::where(var_997, var_999, var_1003);
                var_1005 = wp::mul(var_995, var_1004);
                var_1007 = wp::extract(var_384, var_1006);
                var_1008 = wp::mul(var_1005, var_1007);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_980, var_1008);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_1010 = wp::bit_and(var_980, var_1009);
                if (var_1010) {
                    var_1012 = wp::extract(var_74, var_1011);
                }
                if (!var_1010) {
                    var_1015 = wp::extract(var_74, var_1014);
                    var_1016 = wp::sub(var_1013, var_1015);
                }
                var_1017 = wp::where(var_1010, var_1012, var_1016);
                var_1019 = wp::bit_and(var_980, var_1018);
                if (var_1019) {
                }
                if (!var_1019) {
                }
                var_1022 = wp::where(var_1019, var_1020, var_1021);
                var_1023 = wp::mul(var_1017, var_1022);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_1025 = wp::bit_and(var_980, var_1024);
                if (var_1025) {
                    var_1027 = wp::extract(var_74, var_1026);
                }
                if (!var_1025) {
                    var_1030 = wp::extract(var_74, var_1029);
                    var_1031 = wp::sub(var_1028, var_1030);
                }
                var_1032 = wp::where(var_1025, var_1027, var_1031);
                var_1033 = wp::mul(var_1023, var_1032);
                var_1035 = wp::extract(var_384, var_1034);
                var_1036 = wp::mul(var_1033, var_1035);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_980, var_1036);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_1038 = wp::bit_and(var_980, var_1037);
                if (var_1038) {
                    var_1040 = wp::extract(var_74, var_1039);
                }
                if (!var_1038) {
                    var_1043 = wp::extract(var_74, var_1042);
                    var_1044 = wp::sub(var_1041, var_1043);
                }
                var_1045 = wp::where(var_1038, var_1040, var_1044);
                var_1047 = wp::bit_and(var_980, var_1046);
                if (var_1047) {
                    var_1049 = wp::extract(var_74, var_1048);
                }
                if (!var_1047) {
                    var_1052 = wp::extract(var_74, var_1051);
                    var_1053 = wp::sub(var_1050, var_1052);
                }
                var_1054 = wp::where(var_1047, var_1049, var_1053);
                var_1055 = wp::mul(var_1045, var_1054);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_1057 = wp::bit_and(var_980, var_1056);
                if (var_1057) {
                }
                if (!var_1057) {
                }
                var_1060 = wp::where(var_1057, var_1058, var_1059);
                var_1061 = wp::mul(var_1055, var_1060);
                var_1063 = wp::extract(var_384, var_1062);
                var_1064 = wp::mul(var_1061, var_1063);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_980, var_1064);
            }
            var_1065 = wp::where(var_140, var_351, var_980);
            // return node, (rx, ry, rz)                                                      <L 385>
            var_1066 = wp::tuple(var_4, var_6, var_8);
            ret_0 = var_14;
            ret_1 = var_1066;
            return;
        }
        // x = 0 if coord[0] < 0.5 else 1                                                     <L 389>
        var_1068 = wp::extract(var_74, var_1067);
        var_1070 = (var_1068 < var_1069);
        if (var_1070) {
        }
        if (!var_1070) {
        }
        var_1073 = wp::where(var_1070, var_1071, var_1072);
        // y = 0 if coord[1] < 0.5 else 1                                                     <L 390>
        var_1075 = wp::extract(var_74, var_1074);
        var_1077 = (var_1075 < var_1076);
        if (var_1077) {
        }
        if (!var_1077) {
        }
        var_1080 = wp::where(var_1077, var_1078, var_1079);
        // z = 0 if coord[2] < 0.5 else 1                                                     <L 391>
        var_1082 = wp::extract(var_74, var_1081);
        var_1084 = (var_1082 < var_1083);
        if (var_1084) {
        }
        if (!var_1084) {
        }
        var_1087 = wp::where(var_1084, var_1085, var_1086);
        // child = oct_child[node][4 * z + 2 * y + x]                                         <L 392>
        var_1088 = wp::address(var_oct_child, var_14);
        var_1090 = wp::mul(var_1089, var_1087);
        var_1092 = wp::mul(var_1091, var_1080);
        var_1093 = wp::add(var_1090, var_1092);
        var_1094 = wp::add(var_1093, var_1073);
        var_1096 = wp::load(var_1088);
        var_1095 = wp::extract(var_1096, var_1094);
        // stack = child + root if child != -1 else -1                                        <L 393>
        var_1098 = (var_1095 != var_1097);
        if (var_1098) {
            var_1099 = wp::add(var_1095, var_root);
        }
        if (!var_1098) {
        }
        var_1101 = wp::where(var_1098, var_1099, var_1100);
        wp::assign(var_0, var_1101);
        wp::assign(var_2, var_71);
    goto start_while_0;
    end_while_0:;
    // wp.print("ERROR: Node not found\n")                                                    <L 395>
    wp::print(var_1102);
    // return -1, (rx, ry, rz)                                                                <L 396>
    var_1104 = wp::tuple(var_4, var_6, var_8);
    ret_0 = var_1103;
    ret_1 = var_1104;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:436
static CUDA_CALLABLE wp::float32 sample_volume_sdf_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32>* var_0;
    wp::vec_t<3, wp::float32>* var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::array_t<wp::vec_t<8, wp::int32>>* var_6;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_7;
    const bool var_8 = false;
    wp::int32* var_9;
    wp::int32 var_10;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_11;
    wp::array_t<wp::vec_t<8, wp::int32>> var_12;
    wp::array_t<wp::vec_t<3, wp::float32>> var_13;
    wp::int32 var_14;
    const wp::int32 var_15 = 0;
    wp::vec_t<8, wp::float32> var_16;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_17;
    wp::vec_t<8, wp::float32>* var_18;
    wp::array_t<wp::vec_t<8, wp::float32>> var_19;
    wp::float32 var_20;
    wp::vec_t<8, wp::float32> var_21;
    wp::float32 var_22;
    //---------
    // forward
    // def sample_volume_sdf(xyz: wp.vec3, volume_data: VolumeData) -> float:                 <L 437>
    // dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)             <L 438>
    var_0 = &((var_volume_data).center);
    var_1 = &((var_volume_data).half_size);
    var_4 = wp::load(var_0);
    var_5 = wp::load(var_1);
    box_project_0(var_4, var_5, var_xyz, var_2, var_3);
    // node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=False, root=volume_data.root)       <L 439>
    var_6 = &((var_volume_data).oct_child);
    var_7 = &((var_volume_data).oct_aabb);
    var_9 = &((var_volume_data).root);
    var_12 = wp::load(var_6);
    var_13 = wp::load(var_7);
    var_14 = wp::load(var_9);
    find_oct_0(var_12, var_13, var_3, var_8, var_14, var_10, var_11);
    // return dist0 + wp.dot(weights[0], volume_data.oct_coeff[node])                         <L 440>
    var_16 = wp::extract<0>(var_11);
    var_17 = &((var_volume_data).oct_coeff);
    var_19 = wp::load(var_17);
    var_18 = wp::address(var_19, var_10);
    var_21 = wp::load(var_18);
    var_20 = wp::dot(var_16, var_21);
    var_22 = wp::add(var_2, var_20);
    return var_22;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:443
static CUDA_CALLABLE wp::vec_t<3, wp::float32> sample_volume_grad_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32>* var_0;
    wp::vec_t<3, wp::float32>* var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    const wp::int32 var_6 = 0;
    bool var_7;
    const wp::float32 var_8 = 0.0001;
    const wp::float32 var_9 = 0.0;
    const wp::float32 var_10 = 0.0;
    wp::vec_t<3, wp::float32> var_11;
    const wp::float32 var_12 = 0.0;
    const wp::float32 var_13 = 0.0;
    wp::vec_t<3, wp::float32> var_14;
    const wp::float32 var_15 = 0.0;
    const wp::float32 var_16 = 0.0;
    wp::vec_t<3, wp::float32> var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::array_t<wp::vec_t<8, wp::int32>>* var_32;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_33;
    const bool var_34 = true;
    wp::int32* var_35;
    wp::int32 var_36;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_37;
    wp::array_t<wp::vec_t<8, wp::int32>> var_38;
    wp::array_t<wp::vec_t<3, wp::float32>> var_39;
    wp::int32 var_40;
    const wp::int32 var_41 = 0;
    wp::vec_t<8, wp::float32> var_42;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_43;
    wp::vec_t<8, wp::float32>* var_44;
    wp::array_t<wp::vec_t<8, wp::float32>> var_45;
    wp::float32 var_46;
    wp::vec_t<8, wp::float32> var_47;
    const wp::int32 var_48 = 1;
    wp::vec_t<8, wp::float32> var_49;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_50;
    wp::vec_t<8, wp::float32>* var_51;
    wp::array_t<wp::vec_t<8, wp::float32>> var_52;
    wp::float32 var_53;
    wp::vec_t<8, wp::float32> var_54;
    const wp::int32 var_55 = 2;
    wp::vec_t<8, wp::float32> var_56;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_57;
    wp::vec_t<8, wp::float32>* var_58;
    wp::array_t<wp::vec_t<8, wp::float32>> var_59;
    wp::float32 var_60;
    wp::vec_t<8, wp::float32> var_61;
    wp::vec_t<3, wp::float32> var_62;
    //---------
    // forward
    // def sample_volume_grad(xyz: wp.vec3, volume_data: VolumeData) -> wp.vec3:              <L 444>
    // dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)             <L 445>
    var_0 = &((var_volume_data).center);
    var_1 = &((var_volume_data).half_size);
    var_4 = wp::load(var_0);
    var_5 = wp::load(var_1);
    box_project_0(var_4, var_5, var_xyz, var_2, var_3);
    // if dist0 > 0:                                                                          <L 446>
    var_7 = (var_2 > var_6);
    if (var_7) {
        // h = 1e-4                                                                           <L 447>
        // dx = wp.vec3(h, 0.0, 0.0)                                                          <L 448>
        var_11 = wp::vec_t<3, wp::float32>(var_8, var_9, var_10);
        // dy = wp.vec3(0.0, h, 0.0)                                                          <L 449>
        var_14 = wp::vec_t<3, wp::float32>(var_12, var_8, var_13);
        // dz = wp.vec3(0.0, 0.0, h)                                                          <L 450>
        var_17 = wp::vec_t<3, wp::float32>(var_15, var_16, var_8);
        // f = sample_volume_sdf(xyz, volume_data)                                            <L 451>
        var_18 = sample_volume_sdf_0(var_xyz, var_volume_data);
        // grad_x = (sample_volume_sdf(xyz + dx, volume_data) - f) / h                        <L 452>
        var_19 = wp::add(var_xyz, var_11);
        var_20 = sample_volume_sdf_0(var_19, var_volume_data);
        var_21 = wp::sub(var_20, var_18);
        var_22 = wp::div(var_21, var_8);
        // grad_y = (sample_volume_sdf(xyz + dy, volume_data) - f) / h                        <L 453>
        var_23 = wp::add(var_xyz, var_14);
        var_24 = sample_volume_sdf_0(var_23, var_volume_data);
        var_25 = wp::sub(var_24, var_18);
        var_26 = wp::div(var_25, var_8);
        // grad_z = (sample_volume_sdf(xyz + dz, volume_data) - f) / h                        <L 454>
        var_27 = wp::add(var_xyz, var_17);
        var_28 = sample_volume_sdf_0(var_27, var_volume_data);
        var_29 = wp::sub(var_28, var_18);
        var_30 = wp::div(var_29, var_8);
        // return wp.vec3(grad_x, grad_y, grad_z)                                             <L 455>
        var_31 = wp::vec_t<3, wp::float32>(var_22, var_26, var_30);
        return var_31;
    }
    // node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=True, root=volume_data.root)       <L 456>
    var_32 = &((var_volume_data).oct_child);
    var_33 = &((var_volume_data).oct_aabb);
    var_35 = &((var_volume_data).root);
    var_38 = wp::load(var_32);
    var_39 = wp::load(var_33);
    var_40 = wp::load(var_35);
    find_oct_0(var_38, var_39, var_3, var_34, var_40, var_36, var_37);
    // grad_x = wp.dot(weights[0], volume_data.oct_coeff[node])                               <L 457>
    var_42 = wp::extract<0>(var_37);
    var_43 = &((var_volume_data).oct_coeff);
    var_45 = wp::load(var_43);
    var_44 = wp::address(var_45, var_36);
    var_47 = wp::load(var_44);
    var_46 = wp::dot(var_42, var_47);
    // grad_y = wp.dot(weights[1], volume_data.oct_coeff[node])                               <L 458>
    var_49 = wp::extract<1>(var_37);
    var_50 = &((var_volume_data).oct_coeff);
    var_52 = wp::load(var_50);
    var_51 = wp::address(var_52, var_36);
    var_54 = wp::load(var_51);
    var_53 = wp::dot(var_49, var_54);
    // grad_z = wp.dot(weights[2], volume_data.oct_coeff[node])                               <L 459>
    var_56 = wp::extract<2>(var_37);
    var_57 = &((var_volume_data).oct_coeff);
    var_59 = wp::load(var_57);
    var_58 = wp::address(var_59, var_36);
    var_61 = wp::load(var_58);
    var_60 = wp::dot(var_56, var_61);
    // return wp.vec3(grad_x, grad_y, grad_z)                                                 <L 460>
    var_62 = wp::vec_t<3, wp::float32>(var_46, var_53, var_60);
    return var_62;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:399
static CUDA_CALLABLE void adj_box_project_0(
    wp::vec_t<3, wp::float32> var_center,
    wp::vec_t<3, wp::float32> var_half_size,
    wp::vec_t<3, wp::float32> var_xyz,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_center,
    wp::vec_t<3, wp::float32> & adj_half_size,
    wp::vec_t<3, wp::float32> & adj_xyz,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    wp::float32 var_3;
    const wp::int32 var_4 = 0;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::int32 var_16 = 2;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    bool var_20;
    const wp::int32 var_21 = 0;
    wp::float32 var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    const wp::int32 var_25 = 1;
    wp::float32 var_26;
    const wp::float32 var_27 = 0.0;
    bool var_28;
    const wp::int32 var_29 = 2;
    wp::float32 var_30;
    const wp::float32 var_31 = 0.0;
    bool var_32;
    const wp::float32 var_33 = 0.0;
    const wp::float32 var_34 = 0.0;
    const wp::float32 var_35 = 0.0001;
    const wp::int32 var_36 = 0;
    wp::float32 var_37;
    const wp::int32 var_38 = 1;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    wp::vec_t<3, wp::float32> var_42;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    const wp::float32 var_45 = 0.0;
    bool var_46;
    const wp::int32 var_47 = 0;
    wp::float32 var_48;
    const wp::int32 var_49 = 0;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::int32 var_53 = 0;
    wp::float32 var_54;
    const wp::float32 var_55 = 0.0;
    bool var_56;
    const wp::int32 var_57 = 0;
    wp::float32 var_58;
    const wp::int32 var_59 = 0;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    const wp::int32 var_63 = 1;
    wp::float32 var_64;
    const wp::int32 var_65 = 2;
    wp::float32 var_66;
    wp::vec_t<3, wp::float32> var_67;
    wp::vec_t<3, wp::float32> var_68;
    const wp::int32 var_69 = 0;
    wp::float32 var_70;
    const wp::int32 var_71 = 0;
    wp::float32 var_72;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::int32 var_75 = 1;
    wp::float32 var_76;
    const wp::int32 var_77 = 2;
    wp::float32 var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::vec_t<3, wp::float32> var_80;
    wp::float32 var_81;
    wp::vec_t<3, wp::float32> var_82;
    const wp::int32 var_83 = 1;
    wp::float32 var_84;
    const wp::float32 var_85 = 0.0;
    bool var_86;
    const wp::int32 var_87 = 1;
    wp::float32 var_88;
    const wp::int32 var_89 = 1;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    const wp::int32 var_93 = 1;
    wp::float32 var_94;
    const wp::float32 var_95 = 0.0;
    bool var_96;
    const wp::int32 var_97 = 0;
    wp::float32 var_98;
    const wp::int32 var_99 = 1;
    wp::float32 var_100;
    const wp::int32 var_101 = 1;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    const wp::int32 var_105 = 2;
    wp::float32 var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    const wp::int32 var_109 = 0;
    wp::float32 var_110;
    const wp::int32 var_111 = 1;
    wp::float32 var_112;
    const wp::int32 var_113 = 1;
    wp::float32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    const wp::int32 var_117 = 2;
    wp::float32 var_118;
    wp::vec_t<3, wp::float32> var_119;
    wp::vec_t<3, wp::float32> var_120;
    wp::float32 var_121;
    wp::vec_t<3, wp::float32> var_122;
    const wp::int32 var_123 = 2;
    wp::float32 var_124;
    const wp::float32 var_125 = 0.0;
    bool var_126;
    const wp::int32 var_127 = 2;
    wp::float32 var_128;
    const wp::int32 var_129 = 2;
    wp::float32 var_130;
    wp::float32 var_131;
    wp::float32 var_132;
    const wp::int32 var_133 = 2;
    wp::float32 var_134;
    const wp::float32 var_135 = 0.0;
    bool var_136;
    const wp::int32 var_137 = 0;
    wp::float32 var_138;
    const wp::int32 var_139 = 1;
    wp::float32 var_140;
    const wp::int32 var_141 = 2;
    wp::float32 var_142;
    const wp::int32 var_143 = 2;
    wp::float32 var_144;
    wp::float32 var_145;
    wp::float32 var_146;
    wp::vec_t<3, wp::float32> var_147;
    wp::vec_t<3, wp::float32> var_148;
    const wp::int32 var_149 = 0;
    wp::float32 var_150;
    const wp::int32 var_151 = 1;
    wp::float32 var_152;
    const wp::int32 var_153 = 2;
    wp::float32 var_154;
    const wp::int32 var_155 = 2;
    wp::float32 var_156;
    wp::float32 var_157;
    wp::float32 var_158;
    wp::vec_t<3, wp::float32> var_159;
    wp::vec_t<3, wp::float32> var_160;
    wp::float32 var_161;
    wp::vec_t<3, wp::float32> var_162;
    wp::float32 var_163;
    //---------
    // dual vars
    wp::vec_t<3, wp::float32> adj_0 = {};
    wp::int32 adj_1 = {};
    wp::float32 adj_2 = {};
    wp::float32 adj_3 = {};
    wp::int32 adj_4 = {};
    wp::float32 adj_5 = {};
    wp::float32 adj_6 = {};
    wp::int32 adj_7 = {};
    wp::float32 adj_8 = {};
    wp::float32 adj_9 = {};
    wp::int32 adj_10 = {};
    wp::float32 adj_11 = {};
    wp::float32 adj_12 = {};
    wp::int32 adj_13 = {};
    wp::float32 adj_14 = {};
    wp::float32 adj_15 = {};
    wp::int32 adj_16 = {};
    wp::float32 adj_17 = {};
    wp::float32 adj_18 = {};
    wp::vec_t<3, wp::float32> adj_19 = {};
    bool adj_20 = {};
    wp::int32 adj_21 = {};
    wp::float32 adj_22 = {};
    wp::float32 adj_23 = {};
    bool adj_24 = {};
    wp::int32 adj_25 = {};
    wp::float32 adj_26 = {};
    wp::float32 adj_27 = {};
    bool adj_28 = {};
    wp::int32 adj_29 = {};
    wp::float32 adj_30 = {};
    wp::float32 adj_31 = {};
    bool adj_32 = {};
    wp::float32 adj_33 = {};
    wp::float32 adj_34 = {};
    wp::float32 adj_35 = {};
    wp::int32 adj_36 = {};
    wp::float32 adj_37 = {};
    wp::int32 adj_38 = {};
    wp::float32 adj_39 = {};
    wp::int32 adj_40 = {};
    wp::float32 adj_41 = {};
    wp::vec_t<3, wp::float32> adj_42 = {};
    wp::int32 adj_43 = {};
    wp::float32 adj_44 = {};
    wp::float32 adj_45 = {};
    bool adj_46 = {};
    wp::int32 adj_47 = {};
    wp::float32 adj_48 = {};
    wp::int32 adj_49 = {};
    wp::float32 adj_50 = {};
    wp::float32 adj_51 = {};
    wp::float32 adj_52 = {};
    wp::int32 adj_53 = {};
    wp::float32 adj_54 = {};
    wp::float32 adj_55 = {};
    bool adj_56 = {};
    wp::int32 adj_57 = {};
    wp::float32 adj_58 = {};
    wp::int32 adj_59 = {};
    wp::float32 adj_60 = {};
    wp::float32 adj_61 = {};
    wp::float32 adj_62 = {};
    wp::int32 adj_63 = {};
    wp::float32 adj_64 = {};
    wp::int32 adj_65 = {};
    wp::float32 adj_66 = {};
    wp::vec_t<3, wp::float32> adj_67 = {};
    wp::vec_t<3, wp::float32> adj_68 = {};
    wp::int32 adj_69 = {};
    wp::float32 adj_70 = {};
    wp::int32 adj_71 = {};
    wp::float32 adj_72 = {};
    wp::float32 adj_73 = {};
    wp::float32 adj_74 = {};
    wp::int32 adj_75 = {};
    wp::float32 adj_76 = {};
    wp::int32 adj_77 = {};
    wp::float32 adj_78 = {};
    wp::vec_t<3, wp::float32> adj_79 = {};
    wp::vec_t<3, wp::float32> adj_80 = {};
    wp::float32 adj_81 = {};
    wp::vec_t<3, wp::float32> adj_82 = {};
    wp::int32 adj_83 = {};
    wp::float32 adj_84 = {};
    wp::float32 adj_85 = {};
    bool adj_86 = {};
    wp::int32 adj_87 = {};
    wp::float32 adj_88 = {};
    wp::int32 adj_89 = {};
    wp::float32 adj_90 = {};
    wp::float32 adj_91 = {};
    wp::float32 adj_92 = {};
    wp::int32 adj_93 = {};
    wp::float32 adj_94 = {};
    wp::float32 adj_95 = {};
    bool adj_96 = {};
    wp::int32 adj_97 = {};
    wp::float32 adj_98 = {};
    wp::int32 adj_99 = {};
    wp::float32 adj_100 = {};
    wp::int32 adj_101 = {};
    wp::float32 adj_102 = {};
    wp::float32 adj_103 = {};
    wp::float32 adj_104 = {};
    wp::int32 adj_105 = {};
    wp::float32 adj_106 = {};
    wp::vec_t<3, wp::float32> adj_107 = {};
    wp::vec_t<3, wp::float32> adj_108 = {};
    wp::int32 adj_109 = {};
    wp::float32 adj_110 = {};
    wp::int32 adj_111 = {};
    wp::float32 adj_112 = {};
    wp::int32 adj_113 = {};
    wp::float32 adj_114 = {};
    wp::float32 adj_115 = {};
    wp::float32 adj_116 = {};
    wp::int32 adj_117 = {};
    wp::float32 adj_118 = {};
    wp::vec_t<3, wp::float32> adj_119 = {};
    wp::vec_t<3, wp::float32> adj_120 = {};
    wp::float32 adj_121 = {};
    wp::vec_t<3, wp::float32> adj_122 = {};
    wp::int32 adj_123 = {};
    wp::float32 adj_124 = {};
    wp::float32 adj_125 = {};
    bool adj_126 = {};
    wp::int32 adj_127 = {};
    wp::float32 adj_128 = {};
    wp::int32 adj_129 = {};
    wp::float32 adj_130 = {};
    wp::float32 adj_131 = {};
    wp::float32 adj_132 = {};
    wp::int32 adj_133 = {};
    wp::float32 adj_134 = {};
    wp::float32 adj_135 = {};
    bool adj_136 = {};
    wp::int32 adj_137 = {};
    wp::float32 adj_138 = {};
    wp::int32 adj_139 = {};
    wp::float32 adj_140 = {};
    wp::int32 adj_141 = {};
    wp::float32 adj_142 = {};
    wp::int32 adj_143 = {};
    wp::float32 adj_144 = {};
    wp::float32 adj_145 = {};
    wp::float32 adj_146 = {};
    wp::vec_t<3, wp::float32> adj_147 = {};
    wp::vec_t<3, wp::float32> adj_148 = {};
    wp::int32 adj_149 = {};
    wp::float32 adj_150 = {};
    wp::int32 adj_151 = {};
    wp::float32 adj_152 = {};
    wp::int32 adj_153 = {};
    wp::float32 adj_154 = {};
    wp::int32 adj_155 = {};
    wp::float32 adj_156 = {};
    wp::float32 adj_157 = {};
    wp::float32 adj_158 = {};
    wp::vec_t<3, wp::float32> adj_159 = {};
    wp::vec_t<3, wp::float32> adj_160 = {};
    wp::float32 adj_161 = {};
    wp::vec_t<3, wp::float32> adj_162 = {};
    wp::float32 adj_163 = {};
    //---------
    // forward
    // def box_project(center: wp.vec3, half_size: wp.vec3, xyz: wp.vec3) -> Tuple[float, wp.vec3]:       <L 400>
    // r = xyz - center                                                                       <L 401>
    var_0 = wp::sub(var_xyz, var_center);
    // q = wp.vec3(wp.abs(r[0]) - half_size[0], wp.abs(r[1]) - half_size[1], wp.abs(r[2]) - half_size[2])       <L 402>
    var_2 = wp::extract(var_0, var_1);
    var_3 = wp::abs(var_2);
    var_5 = wp::extract(var_half_size, var_4);
    var_6 = wp::sub(var_3, var_5);
    var_8 = wp::extract(var_0, var_7);
    var_9 = wp::abs(var_8);
    var_11 = wp::extract(var_half_size, var_10);
    var_12 = wp::sub(var_9, var_11);
    var_14 = wp::extract(var_0, var_13);
    var_15 = wp::abs(var_14);
    var_17 = wp::extract(var_half_size, var_16);
    var_18 = wp::sub(var_15, var_17);
    var_19 = wp::vec_t<3, wp::float32>(var_6, var_12, var_18);
    // if q[0] <= 0.0 and q[1] <= 0.0 and q[2] <= 0.0:                                        <L 404>
    var_22 = wp::extract(var_19, var_21);
    var_24 = (var_22 <= var_23);
    var_20 = var_24;
    if (var_20) {
        var_26 = wp::extract(var_19, var_25);
        var_28 = (var_26 <= var_27);
        var_20 = var_20 && var_28;
    }
    if (var_20) {
        var_30 = wp::extract(var_19, var_29);
        var_32 = (var_30 <= var_31);
        var_20 = var_20 && var_32;
    }
    if (var_20) {
        // return 0.0, xyz                                                                    <L 405>
        ret_0 = var_33;
        ret_1 = var_xyz;
        goto label0;
    }
    if (!var_20) {
        // dist_sqr = 0.0                                                                     <L 408>
        // eps = 1e-4                                                                         <L 409>
        // point = wp.vec3(xyz[0], xyz[1], xyz[2])                                            <L 410>
        var_37 = wp::extract(var_xyz, var_36);
        var_39 = wp::extract(var_xyz, var_38);
        var_41 = wp::extract(var_xyz, var_40);
        var_42 = wp::vec_t<3, wp::float32>(var_37, var_39, var_41);
        // if q[0] >= 0.0:                                                                    <L 412>
        var_44 = wp::extract(var_19, var_43);
        var_46 = (var_44 >= var_45);
        if (var_46) {
            // dist_sqr += q[0] * q[0]                                                        <L 413>
            var_48 = wp::extract(var_19, var_47);
            var_50 = wp::extract(var_19, var_49);
            var_51 = wp::mul(var_48, var_50);
            var_52 = wp::add(var_34, var_51);
            // if r[0] > 0.0:                                                                 <L 414>
            var_54 = wp::extract(var_0, var_53);
            var_56 = (var_54 > var_55);
            if (var_56) {
                // point = wp.vec3(point[0] - (q[0] + eps), point[1], point[2])               <L 415>
                var_58 = wp::extract(var_42, var_57);
                var_60 = wp::extract(var_19, var_59);
                var_61 = wp::add(var_60, var_35);
                var_62 = wp::sub(var_58, var_61);
                var_64 = wp::extract(var_42, var_63);
                var_66 = wp::extract(var_42, var_65);
                var_67 = wp::vec_t<3, wp::float32>(var_62, var_64, var_66);
            }
            var_68 = wp::where(var_56, var_67, var_42);
            if (!var_56) {
                // point = wp.vec3(point[0] + (q[0] + eps), point[1], point[2])               <L 417>
                var_70 = wp::extract(var_68, var_69);
                var_72 = wp::extract(var_19, var_71);
                var_73 = wp::add(var_72, var_35);
                var_74 = wp::add(var_70, var_73);
                var_76 = wp::extract(var_68, var_75);
                var_78 = wp::extract(var_68, var_77);
                var_79 = wp::vec_t<3, wp::float32>(var_74, var_76, var_78);
            }
            var_80 = wp::where(var_56, var_68, var_79);
        }
        var_81 = wp::where(var_46, var_52, var_34);
        var_82 = wp::where(var_46, var_80, var_42);
        // if q[1] >= 0.0:                                                                    <L 419>
        var_84 = wp::extract(var_19, var_83);
        var_86 = (var_84 >= var_85);
        if (var_86) {
            // dist_sqr += q[1] * q[1]                                                        <L 420>
            var_88 = wp::extract(var_19, var_87);
            var_90 = wp::extract(var_19, var_89);
            var_91 = wp::mul(var_88, var_90);
            var_92 = wp::add(var_81, var_91);
            // if r[1] > 0.0:                                                                 <L 421>
            var_94 = wp::extract(var_0, var_93);
            var_96 = (var_94 > var_95);
            if (var_96) {
                // point = wp.vec3(point[0], point[1] - (q[1] + eps), point[2])               <L 422>
                var_98 = wp::extract(var_82, var_97);
                var_100 = wp::extract(var_82, var_99);
                var_102 = wp::extract(var_19, var_101);
                var_103 = wp::add(var_102, var_35);
                var_104 = wp::sub(var_100, var_103);
                var_106 = wp::extract(var_82, var_105);
                var_107 = wp::vec_t<3, wp::float32>(var_98, var_104, var_106);
            }
            var_108 = wp::where(var_96, var_107, var_82);
            if (!var_96) {
                // point = wp.vec3(point[0], point[1] + (q[1] + eps), point[2])               <L 424>
                var_110 = wp::extract(var_108, var_109);
                var_112 = wp::extract(var_108, var_111);
                var_114 = wp::extract(var_19, var_113);
                var_115 = wp::add(var_114, var_35);
                var_116 = wp::add(var_112, var_115);
                var_118 = wp::extract(var_108, var_117);
                var_119 = wp::vec_t<3, wp::float32>(var_110, var_116, var_118);
            }
            var_120 = wp::where(var_96, var_108, var_119);
        }
        var_121 = wp::where(var_86, var_92, var_81);
        var_122 = wp::where(var_86, var_120, var_82);
        // if q[2] >= 0.0:                                                                    <L 426>
        var_124 = wp::extract(var_19, var_123);
        var_126 = (var_124 >= var_125);
        if (var_126) {
            // dist_sqr += q[2] * q[2]                                                        <L 427>
            var_128 = wp::extract(var_19, var_127);
            var_130 = wp::extract(var_19, var_129);
            var_131 = wp::mul(var_128, var_130);
            var_132 = wp::add(var_121, var_131);
            // if r[2] > 0.0:                                                                 <L 428>
            var_134 = wp::extract(var_0, var_133);
            var_136 = (var_134 > var_135);
            if (var_136) {
                // point = wp.vec3(point[0], point[1], point[2] - (q[2] + eps))               <L 429>
                var_138 = wp::extract(var_122, var_137);
                var_140 = wp::extract(var_122, var_139);
                var_142 = wp::extract(var_122, var_141);
                var_144 = wp::extract(var_19, var_143);
                var_145 = wp::add(var_144, var_35);
                var_146 = wp::sub(var_142, var_145);
                var_147 = wp::vec_t<3, wp::float32>(var_138, var_140, var_146);
            }
            var_148 = wp::where(var_136, var_147, var_122);
            if (!var_136) {
                // point = wp.vec3(point[0], point[1], point[2] + (q[2] + eps))               <L 431>
                var_150 = wp::extract(var_148, var_149);
                var_152 = wp::extract(var_148, var_151);
                var_154 = wp::extract(var_148, var_153);
                var_156 = wp::extract(var_19, var_155);
                var_157 = wp::add(var_156, var_35);
                var_158 = wp::add(var_154, var_157);
                var_159 = wp::vec_t<3, wp::float32>(var_150, var_152, var_158);
            }
            var_160 = wp::where(var_136, var_148, var_159);
        }
        var_161 = wp::where(var_126, var_132, var_121);
        var_162 = wp::where(var_126, var_160, var_122);
        // return wp.sqrt(dist_sqr), point                                                    <L 433>
        var_163 = wp::sqrt(var_161);
        ret_0 = var_163;
        ret_1 = var_162;
        goto label1;
    }
    //---------
    // reverse
    if (!var_20) {
        label1:;
        adj_162 += adj_ret_1;
        adj_163 += adj_ret_0;
        wp::adj_sqrt(var_161, var_163, adj_161, adj_163);
        // adj: return wp.sqrt(dist_sqr), point                                               <L 433>
        wp::adj_where(var_126, var_160, var_122, adj_126, adj_160, adj_122, adj_162);
        wp::adj_where(var_126, var_132, var_121, adj_126, adj_132, adj_121, adj_161);
        if (var_126) {
            wp::adj_where(var_136, var_148, var_159, adj_136, adj_148, adj_159, adj_160);
            if (!var_136) {
                wp::adj_vec_t(var_150, var_152, var_158, adj_150, adj_152, adj_158, adj_159);
                wp::adj_add(var_154, var_157, adj_154, adj_157, adj_158);
                wp::adj_add(var_156, var_35, adj_156, adj_35, adj_157);
                wp::adj_extract(var_19, var_155, adj_19, adj_155, adj_156);
                wp::adj_extract(var_148, var_153, adj_148, adj_153, adj_154);
                wp::adj_extract(var_148, var_151, adj_148, adj_151, adj_152);
                wp::adj_extract(var_148, var_149, adj_148, adj_149, adj_150);
                // adj: point = wp.vec3(point[0], point[1], point[2] + (q[2] + eps))          <L 431>
            }
            wp::adj_where(var_136, var_147, var_122, adj_136, adj_147, adj_122, adj_148);
            if (var_136) {
                wp::adj_vec_t(var_138, var_140, var_146, adj_138, adj_140, adj_146, adj_147);
                wp::adj_sub(var_142, var_145, adj_142, adj_145, adj_146);
                wp::adj_add(var_144, var_35, adj_144, adj_35, adj_145);
                wp::adj_extract(var_19, var_143, adj_19, adj_143, adj_144);
                wp::adj_extract(var_122, var_141, adj_122, adj_141, adj_142);
                wp::adj_extract(var_122, var_139, adj_122, adj_139, adj_140);
                wp::adj_extract(var_122, var_137, adj_122, adj_137, adj_138);
                // adj: point = wp.vec3(point[0], point[1], point[2] - (q[2] + eps))          <L 429>
            }
            wp::adj_extract(var_0, var_133, adj_0, adj_133, adj_134);
            // adj: if r[2] > 0.0:                                                            <L 428>
            wp::adj_add(var_121, var_131, adj_121, adj_131, adj_132);
            wp::adj_mul(var_128, var_130, adj_128, adj_130, adj_131);
            wp::adj_extract(var_19, var_129, adj_19, adj_129, adj_130);
            wp::adj_extract(var_19, var_127, adj_19, adj_127, adj_128);
            // adj: dist_sqr += q[2] * q[2]                                                   <L 427>
        }
        wp::adj_extract(var_19, var_123, adj_19, adj_123, adj_124);
        // adj: if q[2] >= 0.0:                                                               <L 426>
        wp::adj_where(var_86, var_120, var_82, adj_86, adj_120, adj_82, adj_122);
        wp::adj_where(var_86, var_92, var_81, adj_86, adj_92, adj_81, adj_121);
        if (var_86) {
            wp::adj_where(var_96, var_108, var_119, adj_96, adj_108, adj_119, adj_120);
            if (!var_96) {
                wp::adj_vec_t(var_110, var_116, var_118, adj_110, adj_116, adj_118, adj_119);
                wp::adj_extract(var_108, var_117, adj_108, adj_117, adj_118);
                wp::adj_add(var_112, var_115, adj_112, adj_115, adj_116);
                wp::adj_add(var_114, var_35, adj_114, adj_35, adj_115);
                wp::adj_extract(var_19, var_113, adj_19, adj_113, adj_114);
                wp::adj_extract(var_108, var_111, adj_108, adj_111, adj_112);
                wp::adj_extract(var_108, var_109, adj_108, adj_109, adj_110);
                // adj: point = wp.vec3(point[0], point[1] + (q[1] + eps), point[2])          <L 424>
            }
            wp::adj_where(var_96, var_107, var_82, adj_96, adj_107, adj_82, adj_108);
            if (var_96) {
                wp::adj_vec_t(var_98, var_104, var_106, adj_98, adj_104, adj_106, adj_107);
                wp::adj_extract(var_82, var_105, adj_82, adj_105, adj_106);
                wp::adj_sub(var_100, var_103, adj_100, adj_103, adj_104);
                wp::adj_add(var_102, var_35, adj_102, adj_35, adj_103);
                wp::adj_extract(var_19, var_101, adj_19, adj_101, adj_102);
                wp::adj_extract(var_82, var_99, adj_82, adj_99, adj_100);
                wp::adj_extract(var_82, var_97, adj_82, adj_97, adj_98);
                // adj: point = wp.vec3(point[0], point[1] - (q[1] + eps), point[2])          <L 422>
            }
            wp::adj_extract(var_0, var_93, adj_0, adj_93, adj_94);
            // adj: if r[1] > 0.0:                                                            <L 421>
            wp::adj_add(var_81, var_91, adj_81, adj_91, adj_92);
            wp::adj_mul(var_88, var_90, adj_88, adj_90, adj_91);
            wp::adj_extract(var_19, var_89, adj_19, adj_89, adj_90);
            wp::adj_extract(var_19, var_87, adj_19, adj_87, adj_88);
            // adj: dist_sqr += q[1] * q[1]                                                   <L 420>
        }
        wp::adj_extract(var_19, var_83, adj_19, adj_83, adj_84);
        // adj: if q[1] >= 0.0:                                                               <L 419>
        wp::adj_where(var_46, var_80, var_42, adj_46, adj_80, adj_42, adj_82);
        wp::adj_where(var_46, var_52, var_34, adj_46, adj_52, adj_34, adj_81);
        if (var_46) {
            wp::adj_where(var_56, var_68, var_79, adj_56, adj_68, adj_79, adj_80);
            if (!var_56) {
                wp::adj_vec_t(var_74, var_76, var_78, adj_74, adj_76, adj_78, adj_79);
                wp::adj_extract(var_68, var_77, adj_68, adj_77, adj_78);
                wp::adj_extract(var_68, var_75, adj_68, adj_75, adj_76);
                wp::adj_add(var_70, var_73, adj_70, adj_73, adj_74);
                wp::adj_add(var_72, var_35, adj_72, adj_35, adj_73);
                wp::adj_extract(var_19, var_71, adj_19, adj_71, adj_72);
                wp::adj_extract(var_68, var_69, adj_68, adj_69, adj_70);
                // adj: point = wp.vec3(point[0] + (q[0] + eps), point[1], point[2])          <L 417>
            }
            wp::adj_where(var_56, var_67, var_42, adj_56, adj_67, adj_42, adj_68);
            if (var_56) {
                wp::adj_vec_t(var_62, var_64, var_66, adj_62, adj_64, adj_66, adj_67);
                wp::adj_extract(var_42, var_65, adj_42, adj_65, adj_66);
                wp::adj_extract(var_42, var_63, adj_42, adj_63, adj_64);
                wp::adj_sub(var_58, var_61, adj_58, adj_61, adj_62);
                wp::adj_add(var_60, var_35, adj_60, adj_35, adj_61);
                wp::adj_extract(var_19, var_59, adj_19, adj_59, adj_60);
                wp::adj_extract(var_42, var_57, adj_42, adj_57, adj_58);
                // adj: point = wp.vec3(point[0] - (q[0] + eps), point[1], point[2])          <L 415>
            }
            wp::adj_extract(var_0, var_53, adj_0, adj_53, adj_54);
            // adj: if r[0] > 0.0:                                                            <L 414>
            wp::adj_add(var_34, var_51, adj_34, adj_51, adj_52);
            wp::adj_mul(var_48, var_50, adj_48, adj_50, adj_51);
            wp::adj_extract(var_19, var_49, adj_19, adj_49, adj_50);
            wp::adj_extract(var_19, var_47, adj_19, adj_47, adj_48);
            // adj: dist_sqr += q[0] * q[0]                                                   <L 413>
        }
        wp::adj_extract(var_19, var_43, adj_19, adj_43, adj_44);
        // adj: if q[0] >= 0.0:                                                               <L 412>
        wp::adj_vec_t(var_37, var_39, var_41, adj_37, adj_39, adj_41, adj_42);
        wp::adj_extract(var_xyz, var_40, adj_xyz, adj_40, adj_41);
        wp::adj_extract(var_xyz, var_38, adj_xyz, adj_38, adj_39);
        wp::adj_extract(var_xyz, var_36, adj_xyz, adj_36, adj_37);
        // adj: point = wp.vec3(xyz[0], xyz[1], xyz[2])                                       <L 410>
        // adj: eps = 1e-4                                                                    <L 409>
        // adj: dist_sqr = 0.0                                                                <L 408>
    }
    if (var_20) {
        label0:;
        adj_xyz += adj_ret_1;
        adj_33 += adj_ret_0;
        // adj: return 0.0, xyz                                                               <L 405>
    }
    if (var_20) {
        wp::adj_extract(var_19, var_29, adj_19, adj_29, adj_30);
    }
    if (var_20) {
        wp::adj_extract(var_19, var_25, adj_19, adj_25, adj_26);
    }
    wp::adj_extract(var_19, var_21, adj_19, adj_21, adj_22);
    // adj: if q[0] <= 0.0 and q[1] <= 0.0 and q[2] <= 0.0:                                   <L 404>
    wp::adj_vec_t(var_6, var_12, var_18, adj_6, adj_12, adj_18, adj_19);
    wp::adj_sub(var_15, var_17, adj_15, adj_17, adj_18);
    wp::adj_extract(var_half_size, var_16, adj_half_size, adj_16, adj_17);
    wp::adj_abs(var_14, adj_14, adj_15);
    wp::adj_extract(var_0, var_13, adj_0, adj_13, adj_14);
    wp::adj_sub(var_9, var_11, adj_9, adj_11, adj_12);
    wp::adj_extract(var_half_size, var_10, adj_half_size, adj_10, adj_11);
    wp::adj_abs(var_8, adj_8, adj_9);
    wp::adj_extract(var_0, var_7, adj_0, adj_7, adj_8);
    wp::adj_sub(var_3, var_5, adj_3, adj_5, adj_6);
    wp::adj_extract(var_half_size, var_4, adj_half_size, adj_4, adj_5);
    wp::adj_abs(var_2, adj_2, adj_3);
    wp::adj_extract(var_0, var_1, adj_0, adj_1, adj_2);
    // adj: q = wp.vec3(wp.abs(r[0]) - half_size[0], wp.abs(r[1]) - half_size[1], wp.abs(r[2]) - half_size[2])  <L 402>
    wp::adj_sub(var_xyz, var_center, adj_xyz, adj_center, adj_0);
    // adj: r = xyz - center                                                                  <L 401>
    // adj: def box_project(center: wp.vec3, half_size: wp.vec3, xyz: wp.vec3) -> Tuple[float, wp.vec3]:  <L 400>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:318
static CUDA_CALLABLE void adj_find_oct_0(
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::vec_t<3, wp::float32> var_p,
    bool var_grad,
    wp::int32 var_root,
    wp::int32 & ret_0,
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> & ret_1,
    wp::array_t<wp::vec_t<8, wp::int32>> & adj_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_oct_aabb,
    wp::vec_t<3, wp::float32> & adj_p,
    bool & adj_grad,
    wp::int32 & adj_root,
    wp::int32 & adj_ret_0,
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> & adj_ret_1)
{
    //---------
    // primal vars
    wp::int32 var_0;
    const wp::int32 var_1 = 100;
    wp::int32 var_2;
    const wp::float32 var_3 = 0.0;
    wp::vec_t<8, wp::float32> var_4;
    const wp::float32 var_5 = 0.0;
    wp::vec_t<8, wp::float32> var_6;
    const wp::float32 var_7 = 0.0;
    wp::vec_t<8, wp::float32> var_8;
    const wp::float32 var_9 = 1e-06;
    const wp::int32 var_10 = 0;
    bool var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    wp::int32 var_14;
    const wp::int32 var_15 = -1;
    bool var_16;
    const wp::str var_17 = "ERROR: Invalid node number\n";
    const wp::int32 var_18 = -1;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_19;
    const wp::int32 var_20 = 0;
    wp::vec_t<3, wp::float32>* var_21;
    const wp::int32 var_22 = 1;
    wp::vec_t<3, wp::float32>* var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    const wp::int32 var_27 = 0;
    wp::vec_t<3, wp::float32>* var_28;
    const wp::int32 var_29 = 1;
    wp::vec_t<3, wp::float32>* var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    bool var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    wp::float32 var_37;
    const wp::int32 var_38 = 0;
    wp::float32 var_39;
    bool var_40;
    const wp::int32 var_41 = 0;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::int32 var_44 = 0;
    wp::float32 var_45;
    bool var_46;
    const wp::int32 var_47 = 1;
    wp::float32 var_48;
    wp::float32 var_49;
    const wp::int32 var_50 = 1;
    wp::float32 var_51;
    bool var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    wp::float32 var_55;
    const wp::int32 var_56 = 1;
    wp::float32 var_57;
    bool var_58;
    const wp::int32 var_59 = 2;
    wp::float32 var_60;
    wp::float32 var_61;
    const wp::int32 var_62 = 2;
    wp::float32 var_63;
    bool var_64;
    const wp::int32 var_65 = 2;
    wp::float32 var_66;
    wp::float32 var_67;
    const wp::int32 var_68 = 2;
    wp::float32 var_69;
    bool var_70;
    wp::int32 var_71;
    wp::vec_t<3, wp::float32> var_72;
    wp::vec_t<3, wp::float32> var_73;
    wp::vec_t<3, wp::float32> var_74;
    wp::vec_t<8, wp::int32>* var_75;
    const wp::int32 var_76 = 0;
    wp::int32 var_77;
    wp::vec_t<8, wp::int32> var_78;
    const wp::int32 var_79 = -1;
    bool var_80;
    wp::int32 var_81;
    wp::vec_t<8, wp::int32>* var_82;
    const wp::int32 var_83 = 1;
    wp::int32 var_84;
    wp::vec_t<8, wp::int32> var_85;
    const wp::int32 var_86 = -1;
    bool var_87;
    wp::int32 var_88;
    wp::int32 var_89;
    wp::vec_t<8, wp::int32>* var_90;
    const wp::int32 var_91 = 2;
    wp::int32 var_92;
    wp::vec_t<8, wp::int32> var_93;
    const wp::int32 var_94 = -1;
    bool var_95;
    wp::int32 var_96;
    wp::int32 var_97;
    wp::vec_t<8, wp::int32>* var_98;
    const wp::int32 var_99 = 3;
    wp::int32 var_100;
    wp::vec_t<8, wp::int32> var_101;
    const wp::int32 var_102 = -1;
    bool var_103;
    wp::int32 var_104;
    wp::int32 var_105;
    wp::vec_t<8, wp::int32>* var_106;
    const wp::int32 var_107 = 4;
    wp::int32 var_108;
    wp::vec_t<8, wp::int32> var_109;
    const wp::int32 var_110 = -1;
    bool var_111;
    wp::int32 var_112;
    wp::int32 var_113;
    wp::vec_t<8, wp::int32>* var_114;
    const wp::int32 var_115 = 5;
    wp::int32 var_116;
    wp::vec_t<8, wp::int32> var_117;
    const wp::int32 var_118 = -1;
    bool var_119;
    wp::int32 var_120;
    wp::int32 var_121;
    wp::vec_t<8, wp::int32>* var_122;
    const wp::int32 var_123 = 6;
    wp::int32 var_124;
    wp::vec_t<8, wp::int32> var_125;
    const wp::int32 var_126 = -1;
    bool var_127;
    wp::int32 var_128;
    wp::int32 var_129;
    wp::vec_t<8, wp::int32>* var_130;
    const wp::int32 var_131 = 7;
    wp::int32 var_132;
    wp::vec_t<8, wp::int32> var_133;
    const wp::int32 var_134 = -1;
    bool var_135;
    wp::int32 var_136;
    wp::int32 var_137;
    const wp::int32 var_138 = 0;
    bool var_139;
    bool var_140;
    const wp::int32 var_141 = 0;
    const wp::int32 var_142 = 1;
    wp::int32 var_143;
    const wp::int32 var_144 = 0;
    wp::float32 var_145;
    const wp::float32 var_146 = 1.0;
    const wp::int32 var_147 = 0;
    wp::float32 var_148;
    wp::float32 var_149;
    wp::float32 var_150;
    const wp::int32 var_151 = 2;
    wp::int32 var_152;
    const wp::int32 var_153 = 1;
    wp::float32 var_154;
    const wp::float32 var_155 = 1.0;
    const wp::int32 var_156 = 1;
    wp::float32 var_157;
    wp::float32 var_158;
    wp::float32 var_159;
    wp::float32 var_160;
    const wp::int32 var_161 = 4;
    wp::int32 var_162;
    const wp::int32 var_163 = 2;
    wp::float32 var_164;
    const wp::float32 var_165 = 1.0;
    const wp::int32 var_166 = 2;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    wp::float32 var_170;
    const wp::int32 var_171 = 1;
    const wp::int32 var_172 = 1;
    wp::int32 var_173;
    const wp::int32 var_174 = 0;
    wp::float32 var_175;
    const wp::float32 var_176 = 1.0;
    const wp::int32 var_177 = 0;
    wp::float32 var_178;
    wp::float32 var_179;
    wp::float32 var_180;
    const wp::int32 var_181 = 2;
    wp::int32 var_182;
    const wp::int32 var_183 = 1;
    wp::float32 var_184;
    const wp::float32 var_185 = 1.0;
    const wp::int32 var_186 = 1;
    wp::float32 var_187;
    wp::float32 var_188;
    wp::float32 var_189;
    wp::float32 var_190;
    const wp::int32 var_191 = 4;
    wp::int32 var_192;
    const wp::int32 var_193 = 2;
    wp::float32 var_194;
    const wp::float32 var_195 = 1.0;
    const wp::int32 var_196 = 2;
    wp::float32 var_197;
    wp::float32 var_198;
    wp::float32 var_199;
    wp::float32 var_200;
    const wp::int32 var_201 = 2;
    const wp::int32 var_202 = 1;
    wp::int32 var_203;
    const wp::int32 var_204 = 0;
    wp::float32 var_205;
    const wp::float32 var_206 = 1.0;
    const wp::int32 var_207 = 0;
    wp::float32 var_208;
    wp::float32 var_209;
    wp::float32 var_210;
    const wp::int32 var_211 = 2;
    wp::int32 var_212;
    const wp::int32 var_213 = 1;
    wp::float32 var_214;
    const wp::float32 var_215 = 1.0;
    const wp::int32 var_216 = 1;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    wp::float32 var_220;
    const wp::int32 var_221 = 4;
    wp::int32 var_222;
    const wp::int32 var_223 = 2;
    wp::float32 var_224;
    const wp::float32 var_225 = 1.0;
    const wp::int32 var_226 = 2;
    wp::float32 var_227;
    wp::float32 var_228;
    wp::float32 var_229;
    wp::float32 var_230;
    const wp::int32 var_231 = 3;
    const wp::int32 var_232 = 1;
    wp::int32 var_233;
    const wp::int32 var_234 = 0;
    wp::float32 var_235;
    const wp::float32 var_236 = 1.0;
    const wp::int32 var_237 = 0;
    wp::float32 var_238;
    wp::float32 var_239;
    wp::float32 var_240;
    const wp::int32 var_241 = 2;
    wp::int32 var_242;
    const wp::int32 var_243 = 1;
    wp::float32 var_244;
    const wp::float32 var_245 = 1.0;
    const wp::int32 var_246 = 1;
    wp::float32 var_247;
    wp::float32 var_248;
    wp::float32 var_249;
    wp::float32 var_250;
    const wp::int32 var_251 = 4;
    wp::int32 var_252;
    const wp::int32 var_253 = 2;
    wp::float32 var_254;
    const wp::float32 var_255 = 1.0;
    const wp::int32 var_256 = 2;
    wp::float32 var_257;
    wp::float32 var_258;
    wp::float32 var_259;
    wp::float32 var_260;
    const wp::int32 var_261 = 4;
    const wp::int32 var_262 = 1;
    wp::int32 var_263;
    const wp::int32 var_264 = 0;
    wp::float32 var_265;
    const wp::float32 var_266 = 1.0;
    const wp::int32 var_267 = 0;
    wp::float32 var_268;
    wp::float32 var_269;
    wp::float32 var_270;
    const wp::int32 var_271 = 2;
    wp::int32 var_272;
    const wp::int32 var_273 = 1;
    wp::float32 var_274;
    const wp::float32 var_275 = 1.0;
    const wp::int32 var_276 = 1;
    wp::float32 var_277;
    wp::float32 var_278;
    wp::float32 var_279;
    wp::float32 var_280;
    const wp::int32 var_281 = 4;
    wp::int32 var_282;
    const wp::int32 var_283 = 2;
    wp::float32 var_284;
    const wp::float32 var_285 = 1.0;
    const wp::int32 var_286 = 2;
    wp::float32 var_287;
    wp::float32 var_288;
    wp::float32 var_289;
    wp::float32 var_290;
    const wp::int32 var_291 = 5;
    const wp::int32 var_292 = 1;
    wp::int32 var_293;
    const wp::int32 var_294 = 0;
    wp::float32 var_295;
    const wp::float32 var_296 = 1.0;
    const wp::int32 var_297 = 0;
    wp::float32 var_298;
    wp::float32 var_299;
    wp::float32 var_300;
    const wp::int32 var_301 = 2;
    wp::int32 var_302;
    const wp::int32 var_303 = 1;
    wp::float32 var_304;
    const wp::float32 var_305 = 1.0;
    const wp::int32 var_306 = 1;
    wp::float32 var_307;
    wp::float32 var_308;
    wp::float32 var_309;
    wp::float32 var_310;
    const wp::int32 var_311 = 4;
    wp::int32 var_312;
    const wp::int32 var_313 = 2;
    wp::float32 var_314;
    const wp::float32 var_315 = 1.0;
    const wp::int32 var_316 = 2;
    wp::float32 var_317;
    wp::float32 var_318;
    wp::float32 var_319;
    wp::float32 var_320;
    const wp::int32 var_321 = 6;
    const wp::int32 var_322 = 1;
    wp::int32 var_323;
    const wp::int32 var_324 = 0;
    wp::float32 var_325;
    const wp::float32 var_326 = 1.0;
    const wp::int32 var_327 = 0;
    wp::float32 var_328;
    wp::float32 var_329;
    wp::float32 var_330;
    const wp::int32 var_331 = 2;
    wp::int32 var_332;
    const wp::int32 var_333 = 1;
    wp::float32 var_334;
    const wp::float32 var_335 = 1.0;
    const wp::int32 var_336 = 1;
    wp::float32 var_337;
    wp::float32 var_338;
    wp::float32 var_339;
    wp::float32 var_340;
    const wp::int32 var_341 = 4;
    wp::int32 var_342;
    const wp::int32 var_343 = 2;
    wp::float32 var_344;
    const wp::float32 var_345 = 1.0;
    const wp::int32 var_346 = 2;
    wp::float32 var_347;
    wp::float32 var_348;
    wp::float32 var_349;
    wp::float32 var_350;
    const wp::int32 var_351 = 7;
    const wp::int32 var_352 = 1;
    wp::int32 var_353;
    const wp::int32 var_354 = 0;
    wp::float32 var_355;
    const wp::float32 var_356 = 1.0;
    const wp::int32 var_357 = 0;
    wp::float32 var_358;
    wp::float32 var_359;
    wp::float32 var_360;
    const wp::int32 var_361 = 2;
    wp::int32 var_362;
    const wp::int32 var_363 = 1;
    wp::float32 var_364;
    const wp::float32 var_365 = 1.0;
    const wp::int32 var_366 = 1;
    wp::float32 var_367;
    wp::float32 var_368;
    wp::float32 var_369;
    wp::float32 var_370;
    const wp::int32 var_371 = 4;
    wp::int32 var_372;
    const wp::int32 var_373 = 2;
    wp::float32 var_374;
    const wp::float32 var_375 = 1.0;
    const wp::int32 var_376 = 2;
    wp::float32 var_377;
    wp::float32 var_378;
    wp::float32 var_379;
    wp::float32 var_380;
    const wp::float32 var_381 = 1.0;
    wp::vec_t<3, wp::float32> var_382;
    wp::vec_t<3, wp::float32> var_383;
    wp::vec_t<3, wp::float32> var_384;
    const wp::int32 var_385 = 0;
    const wp::int32 var_386 = 1;
    wp::int32 var_387;
    const wp::float32 var_388 = 1.0;
    const wp::float32 var_389 = -1.0;
    wp::float32 var_390;
    const wp::int32 var_391 = 2;
    wp::int32 var_392;
    const wp::int32 var_393 = 1;
    wp::float32 var_394;
    const wp::float32 var_395 = 1.0;
    const wp::int32 var_396 = 1;
    wp::float32 var_397;
    wp::float32 var_398;
    wp::float32 var_399;
    wp::float32 var_400;
    const wp::int32 var_401 = 4;
    wp::int32 var_402;
    const wp::int32 var_403 = 2;
    wp::float32 var_404;
    const wp::float32 var_405 = 1.0;
    const wp::int32 var_406 = 2;
    wp::float32 var_407;
    wp::float32 var_408;
    wp::float32 var_409;
    wp::float32 var_410;
    const wp::int32 var_411 = 0;
    wp::float32 var_412;
    wp::float32 var_413;
    const wp::int32 var_414 = 1;
    wp::int32 var_415;
    const wp::int32 var_416 = 0;
    wp::float32 var_417;
    const wp::float32 var_418 = 1.0;
    const wp::int32 var_419 = 0;
    wp::float32 var_420;
    wp::float32 var_421;
    wp::float32 var_422;
    const wp::int32 var_423 = 2;
    wp::int32 var_424;
    const wp::float32 var_425 = 1.0;
    const wp::float32 var_426 = -1.0;
    wp::float32 var_427;
    wp::float32 var_428;
    const wp::int32 var_429 = 4;
    wp::int32 var_430;
    const wp::int32 var_431 = 2;
    wp::float32 var_432;
    const wp::float32 var_433 = 1.0;
    const wp::int32 var_434 = 2;
    wp::float32 var_435;
    wp::float32 var_436;
    wp::float32 var_437;
    wp::float32 var_438;
    const wp::int32 var_439 = 1;
    wp::float32 var_440;
    wp::float32 var_441;
    const wp::int32 var_442 = 1;
    wp::int32 var_443;
    const wp::int32 var_444 = 0;
    wp::float32 var_445;
    const wp::float32 var_446 = 1.0;
    const wp::int32 var_447 = 0;
    wp::float32 var_448;
    wp::float32 var_449;
    wp::float32 var_450;
    const wp::int32 var_451 = 2;
    wp::int32 var_452;
    const wp::int32 var_453 = 1;
    wp::float32 var_454;
    const wp::float32 var_455 = 1.0;
    const wp::int32 var_456 = 1;
    wp::float32 var_457;
    wp::float32 var_458;
    wp::float32 var_459;
    wp::float32 var_460;
    const wp::int32 var_461 = 4;
    wp::int32 var_462;
    const wp::float32 var_463 = 1.0;
    const wp::float32 var_464 = -1.0;
    wp::float32 var_465;
    wp::float32 var_466;
    const wp::int32 var_467 = 2;
    wp::float32 var_468;
    wp::float32 var_469;
    const wp::int32 var_470 = 1;
    const wp::int32 var_471 = 1;
    wp::int32 var_472;
    const wp::float32 var_473 = 1.0;
    const wp::float32 var_474 = -1.0;
    wp::float32 var_475;
    const wp::int32 var_476 = 2;
    wp::int32 var_477;
    const wp::int32 var_478 = 1;
    wp::float32 var_479;
    const wp::float32 var_480 = 1.0;
    const wp::int32 var_481 = 1;
    wp::float32 var_482;
    wp::float32 var_483;
    wp::float32 var_484;
    wp::float32 var_485;
    const wp::int32 var_486 = 4;
    wp::int32 var_487;
    const wp::int32 var_488 = 2;
    wp::float32 var_489;
    const wp::float32 var_490 = 1.0;
    const wp::int32 var_491 = 2;
    wp::float32 var_492;
    wp::float32 var_493;
    wp::float32 var_494;
    wp::float32 var_495;
    const wp::int32 var_496 = 0;
    wp::float32 var_497;
    wp::float32 var_498;
    const wp::int32 var_499 = 1;
    wp::int32 var_500;
    const wp::int32 var_501 = 0;
    wp::float32 var_502;
    const wp::float32 var_503 = 1.0;
    const wp::int32 var_504 = 0;
    wp::float32 var_505;
    wp::float32 var_506;
    wp::float32 var_507;
    const wp::int32 var_508 = 2;
    wp::int32 var_509;
    const wp::float32 var_510 = 1.0;
    const wp::float32 var_511 = -1.0;
    wp::float32 var_512;
    wp::float32 var_513;
    const wp::int32 var_514 = 4;
    wp::int32 var_515;
    const wp::int32 var_516 = 2;
    wp::float32 var_517;
    const wp::float32 var_518 = 1.0;
    const wp::int32 var_519 = 2;
    wp::float32 var_520;
    wp::float32 var_521;
    wp::float32 var_522;
    wp::float32 var_523;
    const wp::int32 var_524 = 1;
    wp::float32 var_525;
    wp::float32 var_526;
    const wp::int32 var_527 = 1;
    wp::int32 var_528;
    const wp::int32 var_529 = 0;
    wp::float32 var_530;
    const wp::float32 var_531 = 1.0;
    const wp::int32 var_532 = 0;
    wp::float32 var_533;
    wp::float32 var_534;
    wp::float32 var_535;
    const wp::int32 var_536 = 2;
    wp::int32 var_537;
    const wp::int32 var_538 = 1;
    wp::float32 var_539;
    const wp::float32 var_540 = 1.0;
    const wp::int32 var_541 = 1;
    wp::float32 var_542;
    wp::float32 var_543;
    wp::float32 var_544;
    wp::float32 var_545;
    const wp::int32 var_546 = 4;
    wp::int32 var_547;
    const wp::float32 var_548 = 1.0;
    const wp::float32 var_549 = -1.0;
    wp::float32 var_550;
    wp::float32 var_551;
    const wp::int32 var_552 = 2;
    wp::float32 var_553;
    wp::float32 var_554;
    const wp::int32 var_555 = 2;
    const wp::int32 var_556 = 1;
    wp::int32 var_557;
    const wp::float32 var_558 = 1.0;
    const wp::float32 var_559 = -1.0;
    wp::float32 var_560;
    const wp::int32 var_561 = 2;
    wp::int32 var_562;
    const wp::int32 var_563 = 1;
    wp::float32 var_564;
    const wp::float32 var_565 = 1.0;
    const wp::int32 var_566 = 1;
    wp::float32 var_567;
    wp::float32 var_568;
    wp::float32 var_569;
    wp::float32 var_570;
    const wp::int32 var_571 = 4;
    wp::int32 var_572;
    const wp::int32 var_573 = 2;
    wp::float32 var_574;
    const wp::float32 var_575 = 1.0;
    const wp::int32 var_576 = 2;
    wp::float32 var_577;
    wp::float32 var_578;
    wp::float32 var_579;
    wp::float32 var_580;
    const wp::int32 var_581 = 0;
    wp::float32 var_582;
    wp::float32 var_583;
    const wp::int32 var_584 = 1;
    wp::int32 var_585;
    const wp::int32 var_586 = 0;
    wp::float32 var_587;
    const wp::float32 var_588 = 1.0;
    const wp::int32 var_589 = 0;
    wp::float32 var_590;
    wp::float32 var_591;
    wp::float32 var_592;
    const wp::int32 var_593 = 2;
    wp::int32 var_594;
    const wp::float32 var_595 = 1.0;
    const wp::float32 var_596 = -1.0;
    wp::float32 var_597;
    wp::float32 var_598;
    const wp::int32 var_599 = 4;
    wp::int32 var_600;
    const wp::int32 var_601 = 2;
    wp::float32 var_602;
    const wp::float32 var_603 = 1.0;
    const wp::int32 var_604 = 2;
    wp::float32 var_605;
    wp::float32 var_606;
    wp::float32 var_607;
    wp::float32 var_608;
    const wp::int32 var_609 = 1;
    wp::float32 var_610;
    wp::float32 var_611;
    const wp::int32 var_612 = 1;
    wp::int32 var_613;
    const wp::int32 var_614 = 0;
    wp::float32 var_615;
    const wp::float32 var_616 = 1.0;
    const wp::int32 var_617 = 0;
    wp::float32 var_618;
    wp::float32 var_619;
    wp::float32 var_620;
    const wp::int32 var_621 = 2;
    wp::int32 var_622;
    const wp::int32 var_623 = 1;
    wp::float32 var_624;
    const wp::float32 var_625 = 1.0;
    const wp::int32 var_626 = 1;
    wp::float32 var_627;
    wp::float32 var_628;
    wp::float32 var_629;
    wp::float32 var_630;
    const wp::int32 var_631 = 4;
    wp::int32 var_632;
    const wp::float32 var_633 = 1.0;
    const wp::float32 var_634 = -1.0;
    wp::float32 var_635;
    wp::float32 var_636;
    const wp::int32 var_637 = 2;
    wp::float32 var_638;
    wp::float32 var_639;
    const wp::int32 var_640 = 3;
    const wp::int32 var_641 = 1;
    wp::int32 var_642;
    const wp::float32 var_643 = 1.0;
    const wp::float32 var_644 = -1.0;
    wp::float32 var_645;
    const wp::int32 var_646 = 2;
    wp::int32 var_647;
    const wp::int32 var_648 = 1;
    wp::float32 var_649;
    const wp::float32 var_650 = 1.0;
    const wp::int32 var_651 = 1;
    wp::float32 var_652;
    wp::float32 var_653;
    wp::float32 var_654;
    wp::float32 var_655;
    const wp::int32 var_656 = 4;
    wp::int32 var_657;
    const wp::int32 var_658 = 2;
    wp::float32 var_659;
    const wp::float32 var_660 = 1.0;
    const wp::int32 var_661 = 2;
    wp::float32 var_662;
    wp::float32 var_663;
    wp::float32 var_664;
    wp::float32 var_665;
    const wp::int32 var_666 = 0;
    wp::float32 var_667;
    wp::float32 var_668;
    const wp::int32 var_669 = 1;
    wp::int32 var_670;
    const wp::int32 var_671 = 0;
    wp::float32 var_672;
    const wp::float32 var_673 = 1.0;
    const wp::int32 var_674 = 0;
    wp::float32 var_675;
    wp::float32 var_676;
    wp::float32 var_677;
    const wp::int32 var_678 = 2;
    wp::int32 var_679;
    const wp::float32 var_680 = 1.0;
    const wp::float32 var_681 = -1.0;
    wp::float32 var_682;
    wp::float32 var_683;
    const wp::int32 var_684 = 4;
    wp::int32 var_685;
    const wp::int32 var_686 = 2;
    wp::float32 var_687;
    const wp::float32 var_688 = 1.0;
    const wp::int32 var_689 = 2;
    wp::float32 var_690;
    wp::float32 var_691;
    wp::float32 var_692;
    wp::float32 var_693;
    const wp::int32 var_694 = 1;
    wp::float32 var_695;
    wp::float32 var_696;
    const wp::int32 var_697 = 1;
    wp::int32 var_698;
    const wp::int32 var_699 = 0;
    wp::float32 var_700;
    const wp::float32 var_701 = 1.0;
    const wp::int32 var_702 = 0;
    wp::float32 var_703;
    wp::float32 var_704;
    wp::float32 var_705;
    const wp::int32 var_706 = 2;
    wp::int32 var_707;
    const wp::int32 var_708 = 1;
    wp::float32 var_709;
    const wp::float32 var_710 = 1.0;
    const wp::int32 var_711 = 1;
    wp::float32 var_712;
    wp::float32 var_713;
    wp::float32 var_714;
    wp::float32 var_715;
    const wp::int32 var_716 = 4;
    wp::int32 var_717;
    const wp::float32 var_718 = 1.0;
    const wp::float32 var_719 = -1.0;
    wp::float32 var_720;
    wp::float32 var_721;
    const wp::int32 var_722 = 2;
    wp::float32 var_723;
    wp::float32 var_724;
    const wp::int32 var_725 = 4;
    const wp::int32 var_726 = 1;
    wp::int32 var_727;
    const wp::float32 var_728 = 1.0;
    const wp::float32 var_729 = -1.0;
    wp::float32 var_730;
    const wp::int32 var_731 = 2;
    wp::int32 var_732;
    const wp::int32 var_733 = 1;
    wp::float32 var_734;
    const wp::float32 var_735 = 1.0;
    const wp::int32 var_736 = 1;
    wp::float32 var_737;
    wp::float32 var_738;
    wp::float32 var_739;
    wp::float32 var_740;
    const wp::int32 var_741 = 4;
    wp::int32 var_742;
    const wp::int32 var_743 = 2;
    wp::float32 var_744;
    const wp::float32 var_745 = 1.0;
    const wp::int32 var_746 = 2;
    wp::float32 var_747;
    wp::float32 var_748;
    wp::float32 var_749;
    wp::float32 var_750;
    const wp::int32 var_751 = 0;
    wp::float32 var_752;
    wp::float32 var_753;
    const wp::int32 var_754 = 1;
    wp::int32 var_755;
    const wp::int32 var_756 = 0;
    wp::float32 var_757;
    const wp::float32 var_758 = 1.0;
    const wp::int32 var_759 = 0;
    wp::float32 var_760;
    wp::float32 var_761;
    wp::float32 var_762;
    const wp::int32 var_763 = 2;
    wp::int32 var_764;
    const wp::float32 var_765 = 1.0;
    const wp::float32 var_766 = -1.0;
    wp::float32 var_767;
    wp::float32 var_768;
    const wp::int32 var_769 = 4;
    wp::int32 var_770;
    const wp::int32 var_771 = 2;
    wp::float32 var_772;
    const wp::float32 var_773 = 1.0;
    const wp::int32 var_774 = 2;
    wp::float32 var_775;
    wp::float32 var_776;
    wp::float32 var_777;
    wp::float32 var_778;
    const wp::int32 var_779 = 1;
    wp::float32 var_780;
    wp::float32 var_781;
    const wp::int32 var_782 = 1;
    wp::int32 var_783;
    const wp::int32 var_784 = 0;
    wp::float32 var_785;
    const wp::float32 var_786 = 1.0;
    const wp::int32 var_787 = 0;
    wp::float32 var_788;
    wp::float32 var_789;
    wp::float32 var_790;
    const wp::int32 var_791 = 2;
    wp::int32 var_792;
    const wp::int32 var_793 = 1;
    wp::float32 var_794;
    const wp::float32 var_795 = 1.0;
    const wp::int32 var_796 = 1;
    wp::float32 var_797;
    wp::float32 var_798;
    wp::float32 var_799;
    wp::float32 var_800;
    const wp::int32 var_801 = 4;
    wp::int32 var_802;
    const wp::float32 var_803 = 1.0;
    const wp::float32 var_804 = -1.0;
    wp::float32 var_805;
    wp::float32 var_806;
    const wp::int32 var_807 = 2;
    wp::float32 var_808;
    wp::float32 var_809;
    const wp::int32 var_810 = 5;
    const wp::int32 var_811 = 1;
    wp::int32 var_812;
    const wp::float32 var_813 = 1.0;
    const wp::float32 var_814 = -1.0;
    wp::float32 var_815;
    const wp::int32 var_816 = 2;
    wp::int32 var_817;
    const wp::int32 var_818 = 1;
    wp::float32 var_819;
    const wp::float32 var_820 = 1.0;
    const wp::int32 var_821 = 1;
    wp::float32 var_822;
    wp::float32 var_823;
    wp::float32 var_824;
    wp::float32 var_825;
    const wp::int32 var_826 = 4;
    wp::int32 var_827;
    const wp::int32 var_828 = 2;
    wp::float32 var_829;
    const wp::float32 var_830 = 1.0;
    const wp::int32 var_831 = 2;
    wp::float32 var_832;
    wp::float32 var_833;
    wp::float32 var_834;
    wp::float32 var_835;
    const wp::int32 var_836 = 0;
    wp::float32 var_837;
    wp::float32 var_838;
    const wp::int32 var_839 = 1;
    wp::int32 var_840;
    const wp::int32 var_841 = 0;
    wp::float32 var_842;
    const wp::float32 var_843 = 1.0;
    const wp::int32 var_844 = 0;
    wp::float32 var_845;
    wp::float32 var_846;
    wp::float32 var_847;
    const wp::int32 var_848 = 2;
    wp::int32 var_849;
    const wp::float32 var_850 = 1.0;
    const wp::float32 var_851 = -1.0;
    wp::float32 var_852;
    wp::float32 var_853;
    const wp::int32 var_854 = 4;
    wp::int32 var_855;
    const wp::int32 var_856 = 2;
    wp::float32 var_857;
    const wp::float32 var_858 = 1.0;
    const wp::int32 var_859 = 2;
    wp::float32 var_860;
    wp::float32 var_861;
    wp::float32 var_862;
    wp::float32 var_863;
    const wp::int32 var_864 = 1;
    wp::float32 var_865;
    wp::float32 var_866;
    const wp::int32 var_867 = 1;
    wp::int32 var_868;
    const wp::int32 var_869 = 0;
    wp::float32 var_870;
    const wp::float32 var_871 = 1.0;
    const wp::int32 var_872 = 0;
    wp::float32 var_873;
    wp::float32 var_874;
    wp::float32 var_875;
    const wp::int32 var_876 = 2;
    wp::int32 var_877;
    const wp::int32 var_878 = 1;
    wp::float32 var_879;
    const wp::float32 var_880 = 1.0;
    const wp::int32 var_881 = 1;
    wp::float32 var_882;
    wp::float32 var_883;
    wp::float32 var_884;
    wp::float32 var_885;
    const wp::int32 var_886 = 4;
    wp::int32 var_887;
    const wp::float32 var_888 = 1.0;
    const wp::float32 var_889 = -1.0;
    wp::float32 var_890;
    wp::float32 var_891;
    const wp::int32 var_892 = 2;
    wp::float32 var_893;
    wp::float32 var_894;
    const wp::int32 var_895 = 6;
    const wp::int32 var_896 = 1;
    wp::int32 var_897;
    const wp::float32 var_898 = 1.0;
    const wp::float32 var_899 = -1.0;
    wp::float32 var_900;
    const wp::int32 var_901 = 2;
    wp::int32 var_902;
    const wp::int32 var_903 = 1;
    wp::float32 var_904;
    const wp::float32 var_905 = 1.0;
    const wp::int32 var_906 = 1;
    wp::float32 var_907;
    wp::float32 var_908;
    wp::float32 var_909;
    wp::float32 var_910;
    const wp::int32 var_911 = 4;
    wp::int32 var_912;
    const wp::int32 var_913 = 2;
    wp::float32 var_914;
    const wp::float32 var_915 = 1.0;
    const wp::int32 var_916 = 2;
    wp::float32 var_917;
    wp::float32 var_918;
    wp::float32 var_919;
    wp::float32 var_920;
    const wp::int32 var_921 = 0;
    wp::float32 var_922;
    wp::float32 var_923;
    const wp::int32 var_924 = 1;
    wp::int32 var_925;
    const wp::int32 var_926 = 0;
    wp::float32 var_927;
    const wp::float32 var_928 = 1.0;
    const wp::int32 var_929 = 0;
    wp::float32 var_930;
    wp::float32 var_931;
    wp::float32 var_932;
    const wp::int32 var_933 = 2;
    wp::int32 var_934;
    const wp::float32 var_935 = 1.0;
    const wp::float32 var_936 = -1.0;
    wp::float32 var_937;
    wp::float32 var_938;
    const wp::int32 var_939 = 4;
    wp::int32 var_940;
    const wp::int32 var_941 = 2;
    wp::float32 var_942;
    const wp::float32 var_943 = 1.0;
    const wp::int32 var_944 = 2;
    wp::float32 var_945;
    wp::float32 var_946;
    wp::float32 var_947;
    wp::float32 var_948;
    const wp::int32 var_949 = 1;
    wp::float32 var_950;
    wp::float32 var_951;
    const wp::int32 var_952 = 1;
    wp::int32 var_953;
    const wp::int32 var_954 = 0;
    wp::float32 var_955;
    const wp::float32 var_956 = 1.0;
    const wp::int32 var_957 = 0;
    wp::float32 var_958;
    wp::float32 var_959;
    wp::float32 var_960;
    const wp::int32 var_961 = 2;
    wp::int32 var_962;
    const wp::int32 var_963 = 1;
    wp::float32 var_964;
    const wp::float32 var_965 = 1.0;
    const wp::int32 var_966 = 1;
    wp::float32 var_967;
    wp::float32 var_968;
    wp::float32 var_969;
    wp::float32 var_970;
    const wp::int32 var_971 = 4;
    wp::int32 var_972;
    const wp::float32 var_973 = 1.0;
    const wp::float32 var_974 = -1.0;
    wp::float32 var_975;
    wp::float32 var_976;
    const wp::int32 var_977 = 2;
    wp::float32 var_978;
    wp::float32 var_979;
    const wp::int32 var_980 = 7;
    const wp::int32 var_981 = 1;
    wp::int32 var_982;
    const wp::float32 var_983 = 1.0;
    const wp::float32 var_984 = -1.0;
    wp::float32 var_985;
    const wp::int32 var_986 = 2;
    wp::int32 var_987;
    const wp::int32 var_988 = 1;
    wp::float32 var_989;
    const wp::float32 var_990 = 1.0;
    const wp::int32 var_991 = 1;
    wp::float32 var_992;
    wp::float32 var_993;
    wp::float32 var_994;
    wp::float32 var_995;
    const wp::int32 var_996 = 4;
    wp::int32 var_997;
    const wp::int32 var_998 = 2;
    wp::float32 var_999;
    const wp::float32 var_1000 = 1.0;
    const wp::int32 var_1001 = 2;
    wp::float32 var_1002;
    wp::float32 var_1003;
    wp::float32 var_1004;
    wp::float32 var_1005;
    const wp::int32 var_1006 = 0;
    wp::float32 var_1007;
    wp::float32 var_1008;
    const wp::int32 var_1009 = 1;
    wp::int32 var_1010;
    const wp::int32 var_1011 = 0;
    wp::float32 var_1012;
    const wp::float32 var_1013 = 1.0;
    const wp::int32 var_1014 = 0;
    wp::float32 var_1015;
    wp::float32 var_1016;
    wp::float32 var_1017;
    const wp::int32 var_1018 = 2;
    wp::int32 var_1019;
    const wp::float32 var_1020 = 1.0;
    const wp::float32 var_1021 = -1.0;
    wp::float32 var_1022;
    wp::float32 var_1023;
    const wp::int32 var_1024 = 4;
    wp::int32 var_1025;
    const wp::int32 var_1026 = 2;
    wp::float32 var_1027;
    const wp::float32 var_1028 = 1.0;
    const wp::int32 var_1029 = 2;
    wp::float32 var_1030;
    wp::float32 var_1031;
    wp::float32 var_1032;
    wp::float32 var_1033;
    const wp::int32 var_1034 = 1;
    wp::float32 var_1035;
    wp::float32 var_1036;
    const wp::int32 var_1037 = 1;
    wp::int32 var_1038;
    const wp::int32 var_1039 = 0;
    wp::float32 var_1040;
    const wp::float32 var_1041 = 1.0;
    const wp::int32 var_1042 = 0;
    wp::float32 var_1043;
    wp::float32 var_1044;
    wp::float32 var_1045;
    const wp::int32 var_1046 = 2;
    wp::int32 var_1047;
    const wp::int32 var_1048 = 1;
    wp::float32 var_1049;
    const wp::float32 var_1050 = 1.0;
    const wp::int32 var_1051 = 1;
    wp::float32 var_1052;
    wp::float32 var_1053;
    wp::float32 var_1054;
    wp::float32 var_1055;
    const wp::int32 var_1056 = 4;
    wp::int32 var_1057;
    const wp::float32 var_1058 = 1.0;
    const wp::float32 var_1059 = -1.0;
    wp::float32 var_1060;
    wp::float32 var_1061;
    const wp::int32 var_1062 = 2;
    wp::float32 var_1063;
    wp::float32 var_1064;
    wp::int32 var_1065;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_1066;
    const wp::int32 var_1067 = 0;
    wp::float32 var_1068;
    const wp::float32 var_1069 = 0.5;
    bool var_1070;
    const wp::int32 var_1071 = 0;
    const wp::int32 var_1072 = 1;
    wp::int32 var_1073;
    const wp::int32 var_1074 = 1;
    wp::float32 var_1075;
    const wp::float32 var_1076 = 0.5;
    bool var_1077;
    const wp::int32 var_1078 = 0;
    const wp::int32 var_1079 = 1;
    wp::int32 var_1080;
    const wp::int32 var_1081 = 2;
    wp::float32 var_1082;
    const wp::float32 var_1083 = 0.5;
    bool var_1084;
    const wp::int32 var_1085 = 0;
    const wp::int32 var_1086 = 1;
    wp::int32 var_1087;
    wp::vec_t<8, wp::int32>* var_1088;
    const wp::int32 var_1089 = 4;
    wp::int32 var_1090;
    const wp::int32 var_1091 = 2;
    wp::int32 var_1092;
    wp::int32 var_1093;
    wp::int32 var_1094;
    wp::int32 var_1095;
    wp::vec_t<8, wp::int32> var_1096;
    const wp::int32 var_1097 = -1;
    bool var_1098;
    wp::int32 var_1099;
    const wp::int32 var_1100 = -1;
    wp::int32 var_1101;
    const wp::str var_1102 = "ERROR: Node not found\n";
    const wp::int32 var_1103 = -1;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_1104;
    //---------
    // dual vars
    wp::int32 adj_0 = {};
    wp::int32 adj_1 = {};
    wp::int32 adj_2 = {};
    wp::float32 adj_3 = {};
    wp::vec_t<8, wp::float32> adj_4 = {};
    wp::float32 adj_5 = {};
    wp::vec_t<8, wp::float32> adj_6 = {};
    wp::float32 adj_7 = {};
    wp::vec_t<8, wp::float32> adj_8 = {};
    wp::float32 adj_9 = {};
    wp::int32 adj_10 = {};
    bool adj_11 = {};
    wp::int32 adj_12 = {};
    wp::int32 adj_13 = {};
    wp::int32 adj_14 = {};
    wp::int32 adj_15 = {};
    bool adj_16 = {};
    wp::str adj_17 = {};
    wp::int32 adj_18 = {};
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> adj_19 = {};
    wp::int32 adj_20 = {};
    wp::vec_t<3, wp::float32> adj_21 = {};
    wp::int32 adj_22 = {};
    wp::vec_t<3, wp::float32> adj_23 = {};
    wp::vec_t<3, wp::float32> adj_24 = {};
    wp::vec_t<3, wp::float32> adj_25 = {};
    wp::vec_t<3, wp::float32> adj_26 = {};
    wp::int32 adj_27 = {};
    wp::vec_t<3, wp::float32> adj_28 = {};
    wp::int32 adj_29 = {};
    wp::vec_t<3, wp::float32> adj_30 = {};
    wp::vec_t<3, wp::float32> adj_31 = {};
    wp::vec_t<3, wp::float32> adj_32 = {};
    wp::vec_t<3, wp::float32> adj_33 = {};
    bool adj_34 = {};
    wp::int32 adj_35 = {};
    wp::float32 adj_36 = {};
    wp::float32 adj_37 = {};
    wp::int32 adj_38 = {};
    wp::float32 adj_39 = {};
    bool adj_40 = {};
    wp::int32 adj_41 = {};
    wp::float32 adj_42 = {};
    wp::float32 adj_43 = {};
    wp::int32 adj_44 = {};
    wp::float32 adj_45 = {};
    bool adj_46 = {};
    wp::int32 adj_47 = {};
    wp::float32 adj_48 = {};
    wp::float32 adj_49 = {};
    wp::int32 adj_50 = {};
    wp::float32 adj_51 = {};
    bool adj_52 = {};
    wp::int32 adj_53 = {};
    wp::float32 adj_54 = {};
    wp::float32 adj_55 = {};
    wp::int32 adj_56 = {};
    wp::float32 adj_57 = {};
    bool adj_58 = {};
    wp::int32 adj_59 = {};
    wp::float32 adj_60 = {};
    wp::float32 adj_61 = {};
    wp::int32 adj_62 = {};
    wp::float32 adj_63 = {};
    bool adj_64 = {};
    wp::int32 adj_65 = {};
    wp::float32 adj_66 = {};
    wp::float32 adj_67 = {};
    wp::int32 adj_68 = {};
    wp::float32 adj_69 = {};
    bool adj_70 = {};
    wp::int32 adj_71 = {};
    wp::vec_t<3, wp::float32> adj_72 = {};
    wp::vec_t<3, wp::float32> adj_73 = {};
    wp::vec_t<3, wp::float32> adj_74 = {};
    wp::vec_t<8, wp::int32> adj_75 = {};
    wp::int32 adj_76 = {};
    wp::int32 adj_77 = {};
    wp::vec_t<8, wp::int32> adj_78 = {};
    wp::int32 adj_79 = {};
    bool adj_80 = {};
    wp::int32 adj_81 = {};
    wp::vec_t<8, wp::int32> adj_82 = {};
    wp::int32 adj_83 = {};
    wp::int32 adj_84 = {};
    wp::vec_t<8, wp::int32> adj_85 = {};
    wp::int32 adj_86 = {};
    bool adj_87 = {};
    wp::int32 adj_88 = {};
    wp::int32 adj_89 = {};
    wp::vec_t<8, wp::int32> adj_90 = {};
    wp::int32 adj_91 = {};
    wp::int32 adj_92 = {};
    wp::vec_t<8, wp::int32> adj_93 = {};
    wp::int32 adj_94 = {};
    bool adj_95 = {};
    wp::int32 adj_96 = {};
    wp::int32 adj_97 = {};
    wp::vec_t<8, wp::int32> adj_98 = {};
    wp::int32 adj_99 = {};
    wp::int32 adj_100 = {};
    wp::vec_t<8, wp::int32> adj_101 = {};
    wp::int32 adj_102 = {};
    bool adj_103 = {};
    wp::int32 adj_104 = {};
    wp::int32 adj_105 = {};
    wp::vec_t<8, wp::int32> adj_106 = {};
    wp::int32 adj_107 = {};
    wp::int32 adj_108 = {};
    wp::vec_t<8, wp::int32> adj_109 = {};
    wp::int32 adj_110 = {};
    bool adj_111 = {};
    wp::int32 adj_112 = {};
    wp::int32 adj_113 = {};
    wp::vec_t<8, wp::int32> adj_114 = {};
    wp::int32 adj_115 = {};
    wp::int32 adj_116 = {};
    wp::vec_t<8, wp::int32> adj_117 = {};
    wp::int32 adj_118 = {};
    bool adj_119 = {};
    wp::int32 adj_120 = {};
    wp::int32 adj_121 = {};
    wp::vec_t<8, wp::int32> adj_122 = {};
    wp::int32 adj_123 = {};
    wp::int32 adj_124 = {};
    wp::vec_t<8, wp::int32> adj_125 = {};
    wp::int32 adj_126 = {};
    bool adj_127 = {};
    wp::int32 adj_128 = {};
    wp::int32 adj_129 = {};
    wp::vec_t<8, wp::int32> adj_130 = {};
    wp::int32 adj_131 = {};
    wp::int32 adj_132 = {};
    wp::vec_t<8, wp::int32> adj_133 = {};
    wp::int32 adj_134 = {};
    bool adj_135 = {};
    wp::int32 adj_136 = {};
    wp::int32 adj_137 = {};
    wp::int32 adj_138 = {};
    bool adj_139 = {};
    bool adj_140 = {};
    wp::int32 adj_141 = {};
    wp::int32 adj_142 = {};
    wp::int32 adj_143 = {};
    wp::int32 adj_144 = {};
    wp::float32 adj_145 = {};
    wp::float32 adj_146 = {};
    wp::int32 adj_147 = {};
    wp::float32 adj_148 = {};
    wp::float32 adj_149 = {};
    wp::float32 adj_150 = {};
    wp::int32 adj_151 = {};
    wp::int32 adj_152 = {};
    wp::int32 adj_153 = {};
    wp::float32 adj_154 = {};
    wp::float32 adj_155 = {};
    wp::int32 adj_156 = {};
    wp::float32 adj_157 = {};
    wp::float32 adj_158 = {};
    wp::float32 adj_159 = {};
    wp::float32 adj_160 = {};
    wp::int32 adj_161 = {};
    wp::int32 adj_162 = {};
    wp::int32 adj_163 = {};
    wp::float32 adj_164 = {};
    wp::float32 adj_165 = {};
    wp::int32 adj_166 = {};
    wp::float32 adj_167 = {};
    wp::float32 adj_168 = {};
    wp::float32 adj_169 = {};
    wp::float32 adj_170 = {};
    wp::int32 adj_171 = {};
    wp::int32 adj_172 = {};
    wp::int32 adj_173 = {};
    wp::int32 adj_174 = {};
    wp::float32 adj_175 = {};
    wp::float32 adj_176 = {};
    wp::int32 adj_177 = {};
    wp::float32 adj_178 = {};
    wp::float32 adj_179 = {};
    wp::float32 adj_180 = {};
    wp::int32 adj_181 = {};
    wp::int32 adj_182 = {};
    wp::int32 adj_183 = {};
    wp::float32 adj_184 = {};
    wp::float32 adj_185 = {};
    wp::int32 adj_186 = {};
    wp::float32 adj_187 = {};
    wp::float32 adj_188 = {};
    wp::float32 adj_189 = {};
    wp::float32 adj_190 = {};
    wp::int32 adj_191 = {};
    wp::int32 adj_192 = {};
    wp::int32 adj_193 = {};
    wp::float32 adj_194 = {};
    wp::float32 adj_195 = {};
    wp::int32 adj_196 = {};
    wp::float32 adj_197 = {};
    wp::float32 adj_198 = {};
    wp::float32 adj_199 = {};
    wp::float32 adj_200 = {};
    wp::int32 adj_201 = {};
    wp::int32 adj_202 = {};
    wp::int32 adj_203 = {};
    wp::int32 adj_204 = {};
    wp::float32 adj_205 = {};
    wp::float32 adj_206 = {};
    wp::int32 adj_207 = {};
    wp::float32 adj_208 = {};
    wp::float32 adj_209 = {};
    wp::float32 adj_210 = {};
    wp::int32 adj_211 = {};
    wp::int32 adj_212 = {};
    wp::int32 adj_213 = {};
    wp::float32 adj_214 = {};
    wp::float32 adj_215 = {};
    wp::int32 adj_216 = {};
    wp::float32 adj_217 = {};
    wp::float32 adj_218 = {};
    wp::float32 adj_219 = {};
    wp::float32 adj_220 = {};
    wp::int32 adj_221 = {};
    wp::int32 adj_222 = {};
    wp::int32 adj_223 = {};
    wp::float32 adj_224 = {};
    wp::float32 adj_225 = {};
    wp::int32 adj_226 = {};
    wp::float32 adj_227 = {};
    wp::float32 adj_228 = {};
    wp::float32 adj_229 = {};
    wp::float32 adj_230 = {};
    wp::int32 adj_231 = {};
    wp::int32 adj_232 = {};
    wp::int32 adj_233 = {};
    wp::int32 adj_234 = {};
    wp::float32 adj_235 = {};
    wp::float32 adj_236 = {};
    wp::int32 adj_237 = {};
    wp::float32 adj_238 = {};
    wp::float32 adj_239 = {};
    wp::float32 adj_240 = {};
    wp::int32 adj_241 = {};
    wp::int32 adj_242 = {};
    wp::int32 adj_243 = {};
    wp::float32 adj_244 = {};
    wp::float32 adj_245 = {};
    wp::int32 adj_246 = {};
    wp::float32 adj_247 = {};
    wp::float32 adj_248 = {};
    wp::float32 adj_249 = {};
    wp::float32 adj_250 = {};
    wp::int32 adj_251 = {};
    wp::int32 adj_252 = {};
    wp::int32 adj_253 = {};
    wp::float32 adj_254 = {};
    wp::float32 adj_255 = {};
    wp::int32 adj_256 = {};
    wp::float32 adj_257 = {};
    wp::float32 adj_258 = {};
    wp::float32 adj_259 = {};
    wp::float32 adj_260 = {};
    wp::int32 adj_261 = {};
    wp::int32 adj_262 = {};
    wp::int32 adj_263 = {};
    wp::int32 adj_264 = {};
    wp::float32 adj_265 = {};
    wp::float32 adj_266 = {};
    wp::int32 adj_267 = {};
    wp::float32 adj_268 = {};
    wp::float32 adj_269 = {};
    wp::float32 adj_270 = {};
    wp::int32 adj_271 = {};
    wp::int32 adj_272 = {};
    wp::int32 adj_273 = {};
    wp::float32 adj_274 = {};
    wp::float32 adj_275 = {};
    wp::int32 adj_276 = {};
    wp::float32 adj_277 = {};
    wp::float32 adj_278 = {};
    wp::float32 adj_279 = {};
    wp::float32 adj_280 = {};
    wp::int32 adj_281 = {};
    wp::int32 adj_282 = {};
    wp::int32 adj_283 = {};
    wp::float32 adj_284 = {};
    wp::float32 adj_285 = {};
    wp::int32 adj_286 = {};
    wp::float32 adj_287 = {};
    wp::float32 adj_288 = {};
    wp::float32 adj_289 = {};
    wp::float32 adj_290 = {};
    wp::int32 adj_291 = {};
    wp::int32 adj_292 = {};
    wp::int32 adj_293 = {};
    wp::int32 adj_294 = {};
    wp::float32 adj_295 = {};
    wp::float32 adj_296 = {};
    wp::int32 adj_297 = {};
    wp::float32 adj_298 = {};
    wp::float32 adj_299 = {};
    wp::float32 adj_300 = {};
    wp::int32 adj_301 = {};
    wp::int32 adj_302 = {};
    wp::int32 adj_303 = {};
    wp::float32 adj_304 = {};
    wp::float32 adj_305 = {};
    wp::int32 adj_306 = {};
    wp::float32 adj_307 = {};
    wp::float32 adj_308 = {};
    wp::float32 adj_309 = {};
    wp::float32 adj_310 = {};
    wp::int32 adj_311 = {};
    wp::int32 adj_312 = {};
    wp::int32 adj_313 = {};
    wp::float32 adj_314 = {};
    wp::float32 adj_315 = {};
    wp::int32 adj_316 = {};
    wp::float32 adj_317 = {};
    wp::float32 adj_318 = {};
    wp::float32 adj_319 = {};
    wp::float32 adj_320 = {};
    wp::int32 adj_321 = {};
    wp::int32 adj_322 = {};
    wp::int32 adj_323 = {};
    wp::int32 adj_324 = {};
    wp::float32 adj_325 = {};
    wp::float32 adj_326 = {};
    wp::int32 adj_327 = {};
    wp::float32 adj_328 = {};
    wp::float32 adj_329 = {};
    wp::float32 adj_330 = {};
    wp::int32 adj_331 = {};
    wp::int32 adj_332 = {};
    wp::int32 adj_333 = {};
    wp::float32 adj_334 = {};
    wp::float32 adj_335 = {};
    wp::int32 adj_336 = {};
    wp::float32 adj_337 = {};
    wp::float32 adj_338 = {};
    wp::float32 adj_339 = {};
    wp::float32 adj_340 = {};
    wp::int32 adj_341 = {};
    wp::int32 adj_342 = {};
    wp::int32 adj_343 = {};
    wp::float32 adj_344 = {};
    wp::float32 adj_345 = {};
    wp::int32 adj_346 = {};
    wp::float32 adj_347 = {};
    wp::float32 adj_348 = {};
    wp::float32 adj_349 = {};
    wp::float32 adj_350 = {};
    wp::int32 adj_351 = {};
    wp::int32 adj_352 = {};
    wp::int32 adj_353 = {};
    wp::int32 adj_354 = {};
    wp::float32 adj_355 = {};
    wp::float32 adj_356 = {};
    wp::int32 adj_357 = {};
    wp::float32 adj_358 = {};
    wp::float32 adj_359 = {};
    wp::float32 adj_360 = {};
    wp::int32 adj_361 = {};
    wp::int32 adj_362 = {};
    wp::int32 adj_363 = {};
    wp::float32 adj_364 = {};
    wp::float32 adj_365 = {};
    wp::int32 adj_366 = {};
    wp::float32 adj_367 = {};
    wp::float32 adj_368 = {};
    wp::float32 adj_369 = {};
    wp::float32 adj_370 = {};
    wp::int32 adj_371 = {};
    wp::int32 adj_372 = {};
    wp::int32 adj_373 = {};
    wp::float32 adj_374 = {};
    wp::float32 adj_375 = {};
    wp::int32 adj_376 = {};
    wp::float32 adj_377 = {};
    wp::float32 adj_378 = {};
    wp::float32 adj_379 = {};
    wp::float32 adj_380 = {};
    wp::float32 adj_381 = {};
    wp::vec_t<3, wp::float32> adj_382 = {};
    wp::vec_t<3, wp::float32> adj_383 = {};
    wp::vec_t<3, wp::float32> adj_384 = {};
    wp::int32 adj_385 = {};
    wp::int32 adj_386 = {};
    wp::int32 adj_387 = {};
    wp::float32 adj_388 = {};
    wp::float32 adj_389 = {};
    wp::float32 adj_390 = {};
    wp::int32 adj_391 = {};
    wp::int32 adj_392 = {};
    wp::int32 adj_393 = {};
    wp::float32 adj_394 = {};
    wp::float32 adj_395 = {};
    wp::int32 adj_396 = {};
    wp::float32 adj_397 = {};
    wp::float32 adj_398 = {};
    wp::float32 adj_399 = {};
    wp::float32 adj_400 = {};
    wp::int32 adj_401 = {};
    wp::int32 adj_402 = {};
    wp::int32 adj_403 = {};
    wp::float32 adj_404 = {};
    wp::float32 adj_405 = {};
    wp::int32 adj_406 = {};
    wp::float32 adj_407 = {};
    wp::float32 adj_408 = {};
    wp::float32 adj_409 = {};
    wp::float32 adj_410 = {};
    wp::int32 adj_411 = {};
    wp::float32 adj_412 = {};
    wp::float32 adj_413 = {};
    wp::int32 adj_414 = {};
    wp::int32 adj_415 = {};
    wp::int32 adj_416 = {};
    wp::float32 adj_417 = {};
    wp::float32 adj_418 = {};
    wp::int32 adj_419 = {};
    wp::float32 adj_420 = {};
    wp::float32 adj_421 = {};
    wp::float32 adj_422 = {};
    wp::int32 adj_423 = {};
    wp::int32 adj_424 = {};
    wp::float32 adj_425 = {};
    wp::float32 adj_426 = {};
    wp::float32 adj_427 = {};
    wp::float32 adj_428 = {};
    wp::int32 adj_429 = {};
    wp::int32 adj_430 = {};
    wp::int32 adj_431 = {};
    wp::float32 adj_432 = {};
    wp::float32 adj_433 = {};
    wp::int32 adj_434 = {};
    wp::float32 adj_435 = {};
    wp::float32 adj_436 = {};
    wp::float32 adj_437 = {};
    wp::float32 adj_438 = {};
    wp::int32 adj_439 = {};
    wp::float32 adj_440 = {};
    wp::float32 adj_441 = {};
    wp::int32 adj_442 = {};
    wp::int32 adj_443 = {};
    wp::int32 adj_444 = {};
    wp::float32 adj_445 = {};
    wp::float32 adj_446 = {};
    wp::int32 adj_447 = {};
    wp::float32 adj_448 = {};
    wp::float32 adj_449 = {};
    wp::float32 adj_450 = {};
    wp::int32 adj_451 = {};
    wp::int32 adj_452 = {};
    wp::int32 adj_453 = {};
    wp::float32 adj_454 = {};
    wp::float32 adj_455 = {};
    wp::int32 adj_456 = {};
    wp::float32 adj_457 = {};
    wp::float32 adj_458 = {};
    wp::float32 adj_459 = {};
    wp::float32 adj_460 = {};
    wp::int32 adj_461 = {};
    wp::int32 adj_462 = {};
    wp::float32 adj_463 = {};
    wp::float32 adj_464 = {};
    wp::float32 adj_465 = {};
    wp::float32 adj_466 = {};
    wp::int32 adj_467 = {};
    wp::float32 adj_468 = {};
    wp::float32 adj_469 = {};
    wp::int32 adj_470 = {};
    wp::int32 adj_471 = {};
    wp::int32 adj_472 = {};
    wp::float32 adj_473 = {};
    wp::float32 adj_474 = {};
    wp::float32 adj_475 = {};
    wp::int32 adj_476 = {};
    wp::int32 adj_477 = {};
    wp::int32 adj_478 = {};
    wp::float32 adj_479 = {};
    wp::float32 adj_480 = {};
    wp::int32 adj_481 = {};
    wp::float32 adj_482 = {};
    wp::float32 adj_483 = {};
    wp::float32 adj_484 = {};
    wp::float32 adj_485 = {};
    wp::int32 adj_486 = {};
    wp::int32 adj_487 = {};
    wp::int32 adj_488 = {};
    wp::float32 adj_489 = {};
    wp::float32 adj_490 = {};
    wp::int32 adj_491 = {};
    wp::float32 adj_492 = {};
    wp::float32 adj_493 = {};
    wp::float32 adj_494 = {};
    wp::float32 adj_495 = {};
    wp::int32 adj_496 = {};
    wp::float32 adj_497 = {};
    wp::float32 adj_498 = {};
    wp::int32 adj_499 = {};
    wp::int32 adj_500 = {};
    wp::int32 adj_501 = {};
    wp::float32 adj_502 = {};
    wp::float32 adj_503 = {};
    wp::int32 adj_504 = {};
    wp::float32 adj_505 = {};
    wp::float32 adj_506 = {};
    wp::float32 adj_507 = {};
    wp::int32 adj_508 = {};
    wp::int32 adj_509 = {};
    wp::float32 adj_510 = {};
    wp::float32 adj_511 = {};
    wp::float32 adj_512 = {};
    wp::float32 adj_513 = {};
    wp::int32 adj_514 = {};
    wp::int32 adj_515 = {};
    wp::int32 adj_516 = {};
    wp::float32 adj_517 = {};
    wp::float32 adj_518 = {};
    wp::int32 adj_519 = {};
    wp::float32 adj_520 = {};
    wp::float32 adj_521 = {};
    wp::float32 adj_522 = {};
    wp::float32 adj_523 = {};
    wp::int32 adj_524 = {};
    wp::float32 adj_525 = {};
    wp::float32 adj_526 = {};
    wp::int32 adj_527 = {};
    wp::int32 adj_528 = {};
    wp::int32 adj_529 = {};
    wp::float32 adj_530 = {};
    wp::float32 adj_531 = {};
    wp::int32 adj_532 = {};
    wp::float32 adj_533 = {};
    wp::float32 adj_534 = {};
    wp::float32 adj_535 = {};
    wp::int32 adj_536 = {};
    wp::int32 adj_537 = {};
    wp::int32 adj_538 = {};
    wp::float32 adj_539 = {};
    wp::float32 adj_540 = {};
    wp::int32 adj_541 = {};
    wp::float32 adj_542 = {};
    wp::float32 adj_543 = {};
    wp::float32 adj_544 = {};
    wp::float32 adj_545 = {};
    wp::int32 adj_546 = {};
    wp::int32 adj_547 = {};
    wp::float32 adj_548 = {};
    wp::float32 adj_549 = {};
    wp::float32 adj_550 = {};
    wp::float32 adj_551 = {};
    wp::int32 adj_552 = {};
    wp::float32 adj_553 = {};
    wp::float32 adj_554 = {};
    wp::int32 adj_555 = {};
    wp::int32 adj_556 = {};
    wp::int32 adj_557 = {};
    wp::float32 adj_558 = {};
    wp::float32 adj_559 = {};
    wp::float32 adj_560 = {};
    wp::int32 adj_561 = {};
    wp::int32 adj_562 = {};
    wp::int32 adj_563 = {};
    wp::float32 adj_564 = {};
    wp::float32 adj_565 = {};
    wp::int32 adj_566 = {};
    wp::float32 adj_567 = {};
    wp::float32 adj_568 = {};
    wp::float32 adj_569 = {};
    wp::float32 adj_570 = {};
    wp::int32 adj_571 = {};
    wp::int32 adj_572 = {};
    wp::int32 adj_573 = {};
    wp::float32 adj_574 = {};
    wp::float32 adj_575 = {};
    wp::int32 adj_576 = {};
    wp::float32 adj_577 = {};
    wp::float32 adj_578 = {};
    wp::float32 adj_579 = {};
    wp::float32 adj_580 = {};
    wp::int32 adj_581 = {};
    wp::float32 adj_582 = {};
    wp::float32 adj_583 = {};
    wp::int32 adj_584 = {};
    wp::int32 adj_585 = {};
    wp::int32 adj_586 = {};
    wp::float32 adj_587 = {};
    wp::float32 adj_588 = {};
    wp::int32 adj_589 = {};
    wp::float32 adj_590 = {};
    wp::float32 adj_591 = {};
    wp::float32 adj_592 = {};
    wp::int32 adj_593 = {};
    wp::int32 adj_594 = {};
    wp::float32 adj_595 = {};
    wp::float32 adj_596 = {};
    wp::float32 adj_597 = {};
    wp::float32 adj_598 = {};
    wp::int32 adj_599 = {};
    wp::int32 adj_600 = {};
    wp::int32 adj_601 = {};
    wp::float32 adj_602 = {};
    wp::float32 adj_603 = {};
    wp::int32 adj_604 = {};
    wp::float32 adj_605 = {};
    wp::float32 adj_606 = {};
    wp::float32 adj_607 = {};
    wp::float32 adj_608 = {};
    wp::int32 adj_609 = {};
    wp::float32 adj_610 = {};
    wp::float32 adj_611 = {};
    wp::int32 adj_612 = {};
    wp::int32 adj_613 = {};
    wp::int32 adj_614 = {};
    wp::float32 adj_615 = {};
    wp::float32 adj_616 = {};
    wp::int32 adj_617 = {};
    wp::float32 adj_618 = {};
    wp::float32 adj_619 = {};
    wp::float32 adj_620 = {};
    wp::int32 adj_621 = {};
    wp::int32 adj_622 = {};
    wp::int32 adj_623 = {};
    wp::float32 adj_624 = {};
    wp::float32 adj_625 = {};
    wp::int32 adj_626 = {};
    wp::float32 adj_627 = {};
    wp::float32 adj_628 = {};
    wp::float32 adj_629 = {};
    wp::float32 adj_630 = {};
    wp::int32 adj_631 = {};
    wp::int32 adj_632 = {};
    wp::float32 adj_633 = {};
    wp::float32 adj_634 = {};
    wp::float32 adj_635 = {};
    wp::float32 adj_636 = {};
    wp::int32 adj_637 = {};
    wp::float32 adj_638 = {};
    wp::float32 adj_639 = {};
    wp::int32 adj_640 = {};
    wp::int32 adj_641 = {};
    wp::int32 adj_642 = {};
    wp::float32 adj_643 = {};
    wp::float32 adj_644 = {};
    wp::float32 adj_645 = {};
    wp::int32 adj_646 = {};
    wp::int32 adj_647 = {};
    wp::int32 adj_648 = {};
    wp::float32 adj_649 = {};
    wp::float32 adj_650 = {};
    wp::int32 adj_651 = {};
    wp::float32 adj_652 = {};
    wp::float32 adj_653 = {};
    wp::float32 adj_654 = {};
    wp::float32 adj_655 = {};
    wp::int32 adj_656 = {};
    wp::int32 adj_657 = {};
    wp::int32 adj_658 = {};
    wp::float32 adj_659 = {};
    wp::float32 adj_660 = {};
    wp::int32 adj_661 = {};
    wp::float32 adj_662 = {};
    wp::float32 adj_663 = {};
    wp::float32 adj_664 = {};
    wp::float32 adj_665 = {};
    wp::int32 adj_666 = {};
    wp::float32 adj_667 = {};
    wp::float32 adj_668 = {};
    wp::int32 adj_669 = {};
    wp::int32 adj_670 = {};
    wp::int32 adj_671 = {};
    wp::float32 adj_672 = {};
    wp::float32 adj_673 = {};
    wp::int32 adj_674 = {};
    wp::float32 adj_675 = {};
    wp::float32 adj_676 = {};
    wp::float32 adj_677 = {};
    wp::int32 adj_678 = {};
    wp::int32 adj_679 = {};
    wp::float32 adj_680 = {};
    wp::float32 adj_681 = {};
    wp::float32 adj_682 = {};
    wp::float32 adj_683 = {};
    wp::int32 adj_684 = {};
    wp::int32 adj_685 = {};
    wp::int32 adj_686 = {};
    wp::float32 adj_687 = {};
    wp::float32 adj_688 = {};
    wp::int32 adj_689 = {};
    wp::float32 adj_690 = {};
    wp::float32 adj_691 = {};
    wp::float32 adj_692 = {};
    wp::float32 adj_693 = {};
    wp::int32 adj_694 = {};
    wp::float32 adj_695 = {};
    wp::float32 adj_696 = {};
    wp::int32 adj_697 = {};
    wp::int32 adj_698 = {};
    wp::int32 adj_699 = {};
    wp::float32 adj_700 = {};
    wp::float32 adj_701 = {};
    wp::int32 adj_702 = {};
    wp::float32 adj_703 = {};
    wp::float32 adj_704 = {};
    wp::float32 adj_705 = {};
    wp::int32 adj_706 = {};
    wp::int32 adj_707 = {};
    wp::int32 adj_708 = {};
    wp::float32 adj_709 = {};
    wp::float32 adj_710 = {};
    wp::int32 adj_711 = {};
    wp::float32 adj_712 = {};
    wp::float32 adj_713 = {};
    wp::float32 adj_714 = {};
    wp::float32 adj_715 = {};
    wp::int32 adj_716 = {};
    wp::int32 adj_717 = {};
    wp::float32 adj_718 = {};
    wp::float32 adj_719 = {};
    wp::float32 adj_720 = {};
    wp::float32 adj_721 = {};
    wp::int32 adj_722 = {};
    wp::float32 adj_723 = {};
    wp::float32 adj_724 = {};
    wp::int32 adj_725 = {};
    wp::int32 adj_726 = {};
    wp::int32 adj_727 = {};
    wp::float32 adj_728 = {};
    wp::float32 adj_729 = {};
    wp::float32 adj_730 = {};
    wp::int32 adj_731 = {};
    wp::int32 adj_732 = {};
    wp::int32 adj_733 = {};
    wp::float32 adj_734 = {};
    wp::float32 adj_735 = {};
    wp::int32 adj_736 = {};
    wp::float32 adj_737 = {};
    wp::float32 adj_738 = {};
    wp::float32 adj_739 = {};
    wp::float32 adj_740 = {};
    wp::int32 adj_741 = {};
    wp::int32 adj_742 = {};
    wp::int32 adj_743 = {};
    wp::float32 adj_744 = {};
    wp::float32 adj_745 = {};
    wp::int32 adj_746 = {};
    wp::float32 adj_747 = {};
    wp::float32 adj_748 = {};
    wp::float32 adj_749 = {};
    wp::float32 adj_750 = {};
    wp::int32 adj_751 = {};
    wp::float32 adj_752 = {};
    wp::float32 adj_753 = {};
    wp::int32 adj_754 = {};
    wp::int32 adj_755 = {};
    wp::int32 adj_756 = {};
    wp::float32 adj_757 = {};
    wp::float32 adj_758 = {};
    wp::int32 adj_759 = {};
    wp::float32 adj_760 = {};
    wp::float32 adj_761 = {};
    wp::float32 adj_762 = {};
    wp::int32 adj_763 = {};
    wp::int32 adj_764 = {};
    wp::float32 adj_765 = {};
    wp::float32 adj_766 = {};
    wp::float32 adj_767 = {};
    wp::float32 adj_768 = {};
    wp::int32 adj_769 = {};
    wp::int32 adj_770 = {};
    wp::int32 adj_771 = {};
    wp::float32 adj_772 = {};
    wp::float32 adj_773 = {};
    wp::int32 adj_774 = {};
    wp::float32 adj_775 = {};
    wp::float32 adj_776 = {};
    wp::float32 adj_777 = {};
    wp::float32 adj_778 = {};
    wp::int32 adj_779 = {};
    wp::float32 adj_780 = {};
    wp::float32 adj_781 = {};
    wp::int32 adj_782 = {};
    wp::int32 adj_783 = {};
    wp::int32 adj_784 = {};
    wp::float32 adj_785 = {};
    wp::float32 adj_786 = {};
    wp::int32 adj_787 = {};
    wp::float32 adj_788 = {};
    wp::float32 adj_789 = {};
    wp::float32 adj_790 = {};
    wp::int32 adj_791 = {};
    wp::int32 adj_792 = {};
    wp::int32 adj_793 = {};
    wp::float32 adj_794 = {};
    wp::float32 adj_795 = {};
    wp::int32 adj_796 = {};
    wp::float32 adj_797 = {};
    wp::float32 adj_798 = {};
    wp::float32 adj_799 = {};
    wp::float32 adj_800 = {};
    wp::int32 adj_801 = {};
    wp::int32 adj_802 = {};
    wp::float32 adj_803 = {};
    wp::float32 adj_804 = {};
    wp::float32 adj_805 = {};
    wp::float32 adj_806 = {};
    wp::int32 adj_807 = {};
    wp::float32 adj_808 = {};
    wp::float32 adj_809 = {};
    wp::int32 adj_810 = {};
    wp::int32 adj_811 = {};
    wp::int32 adj_812 = {};
    wp::float32 adj_813 = {};
    wp::float32 adj_814 = {};
    wp::float32 adj_815 = {};
    wp::int32 adj_816 = {};
    wp::int32 adj_817 = {};
    wp::int32 adj_818 = {};
    wp::float32 adj_819 = {};
    wp::float32 adj_820 = {};
    wp::int32 adj_821 = {};
    wp::float32 adj_822 = {};
    wp::float32 adj_823 = {};
    wp::float32 adj_824 = {};
    wp::float32 adj_825 = {};
    wp::int32 adj_826 = {};
    wp::int32 adj_827 = {};
    wp::int32 adj_828 = {};
    wp::float32 adj_829 = {};
    wp::float32 adj_830 = {};
    wp::int32 adj_831 = {};
    wp::float32 adj_832 = {};
    wp::float32 adj_833 = {};
    wp::float32 adj_834 = {};
    wp::float32 adj_835 = {};
    wp::int32 adj_836 = {};
    wp::float32 adj_837 = {};
    wp::float32 adj_838 = {};
    wp::int32 adj_839 = {};
    wp::int32 adj_840 = {};
    wp::int32 adj_841 = {};
    wp::float32 adj_842 = {};
    wp::float32 adj_843 = {};
    wp::int32 adj_844 = {};
    wp::float32 adj_845 = {};
    wp::float32 adj_846 = {};
    wp::float32 adj_847 = {};
    wp::int32 adj_848 = {};
    wp::int32 adj_849 = {};
    wp::float32 adj_850 = {};
    wp::float32 adj_851 = {};
    wp::float32 adj_852 = {};
    wp::float32 adj_853 = {};
    wp::int32 adj_854 = {};
    wp::int32 adj_855 = {};
    wp::int32 adj_856 = {};
    wp::float32 adj_857 = {};
    wp::float32 adj_858 = {};
    wp::int32 adj_859 = {};
    wp::float32 adj_860 = {};
    wp::float32 adj_861 = {};
    wp::float32 adj_862 = {};
    wp::float32 adj_863 = {};
    wp::int32 adj_864 = {};
    wp::float32 adj_865 = {};
    wp::float32 adj_866 = {};
    wp::int32 adj_867 = {};
    wp::int32 adj_868 = {};
    wp::int32 adj_869 = {};
    wp::float32 adj_870 = {};
    wp::float32 adj_871 = {};
    wp::int32 adj_872 = {};
    wp::float32 adj_873 = {};
    wp::float32 adj_874 = {};
    wp::float32 adj_875 = {};
    wp::int32 adj_876 = {};
    wp::int32 adj_877 = {};
    wp::int32 adj_878 = {};
    wp::float32 adj_879 = {};
    wp::float32 adj_880 = {};
    wp::int32 adj_881 = {};
    wp::float32 adj_882 = {};
    wp::float32 adj_883 = {};
    wp::float32 adj_884 = {};
    wp::float32 adj_885 = {};
    wp::int32 adj_886 = {};
    wp::int32 adj_887 = {};
    wp::float32 adj_888 = {};
    wp::float32 adj_889 = {};
    wp::float32 adj_890 = {};
    wp::float32 adj_891 = {};
    wp::int32 adj_892 = {};
    wp::float32 adj_893 = {};
    wp::float32 adj_894 = {};
    wp::int32 adj_895 = {};
    wp::int32 adj_896 = {};
    wp::int32 adj_897 = {};
    wp::float32 adj_898 = {};
    wp::float32 adj_899 = {};
    wp::float32 adj_900 = {};
    wp::int32 adj_901 = {};
    wp::int32 adj_902 = {};
    wp::int32 adj_903 = {};
    wp::float32 adj_904 = {};
    wp::float32 adj_905 = {};
    wp::int32 adj_906 = {};
    wp::float32 adj_907 = {};
    wp::float32 adj_908 = {};
    wp::float32 adj_909 = {};
    wp::float32 adj_910 = {};
    wp::int32 adj_911 = {};
    wp::int32 adj_912 = {};
    wp::int32 adj_913 = {};
    wp::float32 adj_914 = {};
    wp::float32 adj_915 = {};
    wp::int32 adj_916 = {};
    wp::float32 adj_917 = {};
    wp::float32 adj_918 = {};
    wp::float32 adj_919 = {};
    wp::float32 adj_920 = {};
    wp::int32 adj_921 = {};
    wp::float32 adj_922 = {};
    wp::float32 adj_923 = {};
    wp::int32 adj_924 = {};
    wp::int32 adj_925 = {};
    wp::int32 adj_926 = {};
    wp::float32 adj_927 = {};
    wp::float32 adj_928 = {};
    wp::int32 adj_929 = {};
    wp::float32 adj_930 = {};
    wp::float32 adj_931 = {};
    wp::float32 adj_932 = {};
    wp::int32 adj_933 = {};
    wp::int32 adj_934 = {};
    wp::float32 adj_935 = {};
    wp::float32 adj_936 = {};
    wp::float32 adj_937 = {};
    wp::float32 adj_938 = {};
    wp::int32 adj_939 = {};
    wp::int32 adj_940 = {};
    wp::int32 adj_941 = {};
    wp::float32 adj_942 = {};
    wp::float32 adj_943 = {};
    wp::int32 adj_944 = {};
    wp::float32 adj_945 = {};
    wp::float32 adj_946 = {};
    wp::float32 adj_947 = {};
    wp::float32 adj_948 = {};
    wp::int32 adj_949 = {};
    wp::float32 adj_950 = {};
    wp::float32 adj_951 = {};
    wp::int32 adj_952 = {};
    wp::int32 adj_953 = {};
    wp::int32 adj_954 = {};
    wp::float32 adj_955 = {};
    wp::float32 adj_956 = {};
    wp::int32 adj_957 = {};
    wp::float32 adj_958 = {};
    wp::float32 adj_959 = {};
    wp::float32 adj_960 = {};
    wp::int32 adj_961 = {};
    wp::int32 adj_962 = {};
    wp::int32 adj_963 = {};
    wp::float32 adj_964 = {};
    wp::float32 adj_965 = {};
    wp::int32 adj_966 = {};
    wp::float32 adj_967 = {};
    wp::float32 adj_968 = {};
    wp::float32 adj_969 = {};
    wp::float32 adj_970 = {};
    wp::int32 adj_971 = {};
    wp::int32 adj_972 = {};
    wp::float32 adj_973 = {};
    wp::float32 adj_974 = {};
    wp::float32 adj_975 = {};
    wp::float32 adj_976 = {};
    wp::int32 adj_977 = {};
    wp::float32 adj_978 = {};
    wp::float32 adj_979 = {};
    wp::int32 adj_980 = {};
    wp::int32 adj_981 = {};
    wp::int32 adj_982 = {};
    wp::float32 adj_983 = {};
    wp::float32 adj_984 = {};
    wp::float32 adj_985 = {};
    wp::int32 adj_986 = {};
    wp::int32 adj_987 = {};
    wp::int32 adj_988 = {};
    wp::float32 adj_989 = {};
    wp::float32 adj_990 = {};
    wp::int32 adj_991 = {};
    wp::float32 adj_992 = {};
    wp::float32 adj_993 = {};
    wp::float32 adj_994 = {};
    wp::float32 adj_995 = {};
    wp::int32 adj_996 = {};
    wp::int32 adj_997 = {};
    wp::int32 adj_998 = {};
    wp::float32 adj_999 = {};
    wp::float32 adj_1000 = {};
    wp::int32 adj_1001 = {};
    wp::float32 adj_1002 = {};
    wp::float32 adj_1003 = {};
    wp::float32 adj_1004 = {};
    wp::float32 adj_1005 = {};
    wp::int32 adj_1006 = {};
    wp::float32 adj_1007 = {};
    wp::float32 adj_1008 = {};
    wp::int32 adj_1009 = {};
    wp::int32 adj_1010 = {};
    wp::int32 adj_1011 = {};
    wp::float32 adj_1012 = {};
    wp::float32 adj_1013 = {};
    wp::int32 adj_1014 = {};
    wp::float32 adj_1015 = {};
    wp::float32 adj_1016 = {};
    wp::float32 adj_1017 = {};
    wp::int32 adj_1018 = {};
    wp::int32 adj_1019 = {};
    wp::float32 adj_1020 = {};
    wp::float32 adj_1021 = {};
    wp::float32 adj_1022 = {};
    wp::float32 adj_1023 = {};
    wp::int32 adj_1024 = {};
    wp::int32 adj_1025 = {};
    wp::int32 adj_1026 = {};
    wp::float32 adj_1027 = {};
    wp::float32 adj_1028 = {};
    wp::int32 adj_1029 = {};
    wp::float32 adj_1030 = {};
    wp::float32 adj_1031 = {};
    wp::float32 adj_1032 = {};
    wp::float32 adj_1033 = {};
    wp::int32 adj_1034 = {};
    wp::float32 adj_1035 = {};
    wp::float32 adj_1036 = {};
    wp::int32 adj_1037 = {};
    wp::int32 adj_1038 = {};
    wp::int32 adj_1039 = {};
    wp::float32 adj_1040 = {};
    wp::float32 adj_1041 = {};
    wp::int32 adj_1042 = {};
    wp::float32 adj_1043 = {};
    wp::float32 adj_1044 = {};
    wp::float32 adj_1045 = {};
    wp::int32 adj_1046 = {};
    wp::int32 adj_1047 = {};
    wp::int32 adj_1048 = {};
    wp::float32 adj_1049 = {};
    wp::float32 adj_1050 = {};
    wp::int32 adj_1051 = {};
    wp::float32 adj_1052 = {};
    wp::float32 adj_1053 = {};
    wp::float32 adj_1054 = {};
    wp::float32 adj_1055 = {};
    wp::int32 adj_1056 = {};
    wp::int32 adj_1057 = {};
    wp::float32 adj_1058 = {};
    wp::float32 adj_1059 = {};
    wp::float32 adj_1060 = {};
    wp::float32 adj_1061 = {};
    wp::int32 adj_1062 = {};
    wp::float32 adj_1063 = {};
    wp::float32 adj_1064 = {};
    wp::int32 adj_1065 = {};
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> adj_1066 = {};
    wp::int32 adj_1067 = {};
    wp::float32 adj_1068 = {};
    wp::float32 adj_1069 = {};
    bool adj_1070 = {};
    wp::int32 adj_1071 = {};
    wp::int32 adj_1072 = {};
    wp::int32 adj_1073 = {};
    wp::int32 adj_1074 = {};
    wp::float32 adj_1075 = {};
    wp::float32 adj_1076 = {};
    bool adj_1077 = {};
    wp::int32 adj_1078 = {};
    wp::int32 adj_1079 = {};
    wp::int32 adj_1080 = {};
    wp::int32 adj_1081 = {};
    wp::float32 adj_1082 = {};
    wp::float32 adj_1083 = {};
    bool adj_1084 = {};
    wp::int32 adj_1085 = {};
    wp::int32 adj_1086 = {};
    wp::int32 adj_1087 = {};
    wp::vec_t<8, wp::int32> adj_1088 = {};
    wp::int32 adj_1089 = {};
    wp::int32 adj_1090 = {};
    wp::int32 adj_1091 = {};
    wp::int32 adj_1092 = {};
    wp::int32 adj_1093 = {};
    wp::int32 adj_1094 = {};
    wp::int32 adj_1095 = {};
    wp::vec_t<8, wp::int32> adj_1096 = {};
    wp::int32 adj_1097 = {};
    bool adj_1098 = {};
    wp::int32 adj_1099 = {};
    wp::int32 adj_1100 = {};
    wp::int32 adj_1101 = {};
    wp::str adj_1102 = {};
    wp::int32 adj_1103 = {};
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> adj_1104 = {};
    //---------
    // forward
    // def find_oct(                                                                          <L 319>
    // stack = root                                                                           <L 322>
    var_0 = wp::copy(var_root);
    // niter = int(100)                                                                       <L 323>
    var_2 = wp::int(var_1);
    // rx = vec8(0.0)                                                                         <L 324>
    var_4 = wp::vec_t<8, wp::float32>(var_3);
    // ry = vec8(0.0)                                                                         <L 325>
    var_6 = wp::vec_t<8, wp::float32>(var_5);
    // rz = vec8(0.0)                                                                         <L 326>
    var_8 = wp::vec_t<8, wp::float32>(var_7);
    // eps = 1e-6                                                                             <L 327>
    // while niter > 0:                                                                       <L 329>
    // wp.print("ERROR: Node not found\n")                                                    <L 395>
    wp::print(var_1102);
    // return -1, (rx, ry, rz)                                                                <L 396>
    var_1104 = wp::tuple(var_4, var_6, var_8);
    ret_0 = var_1103;
    ret_1 = var_1104;
    goto label4;
    //---------
    // reverse
    label4:;
    adj_1104 += adj_ret_1;
    adj_1103 += adj_ret_0;
    // adj: return -1, (rx, ry, rz)                                                           <L 396>
    wp::adj_print(var_1102, adj_1102);
    // adj: wp.print("ERROR: Node not found\n")                                               <L 395>
    start_while_0:;
    var_11 = (var_2 > var_10);
    if ((var_11) == false) goto end_while_0;
    adj_12 = {};
    adj_13 = {};
    adj_14 = {};
    adj_15 = {};
    adj_16 = {};
    adj_17 = {};
    adj_18 = {};
    adj_19 = {};
    adj_20 = {};
    adj_21 = {};
    adj_22 = {};
    adj_23 = {};
    adj_24 = {};
    adj_25 = {};
    adj_26 = {};
    adj_27 = {};
    adj_28 = {};
    adj_29 = {};
    adj_30 = {};
    adj_31 = {};
    adj_32 = {};
    adj_33 = {};
    adj_34 = {};
    adj_35 = {};
    adj_36 = {};
    adj_37 = {};
    adj_38 = {};
    adj_39 = {};
    adj_40 = {};
    adj_41 = {};
    adj_42 = {};
    adj_43 = {};
    adj_44 = {};
    adj_45 = {};
    adj_46 = {};
    adj_47 = {};
    adj_48 = {};
    adj_49 = {};
    adj_50 = {};
    adj_51 = {};
    adj_52 = {};
    adj_53 = {};
    adj_54 = {};
    adj_55 = {};
    adj_56 = {};
    adj_57 = {};
    adj_58 = {};
    adj_59 = {};
    adj_60 = {};
    adj_61 = {};
    adj_62 = {};
    adj_63 = {};
    adj_64 = {};
    adj_65 = {};
    adj_66 = {};
    adj_67 = {};
    adj_68 = {};
    adj_69 = {};
    adj_70 = {};
    adj_71 = {};
    adj_72 = {};
    adj_73 = {};
    adj_74 = {};
    adj_75 = {};
    adj_76 = {};
    adj_77 = {};
    adj_78 = {};
    adj_79 = {};
    adj_80 = {};
    adj_81 = {};
    adj_82 = {};
    adj_83 = {};
    adj_84 = {};
    adj_85 = {};
    adj_86 = {};
    adj_87 = {};
    adj_88 = {};
    adj_89 = {};
    adj_90 = {};
    adj_91 = {};
    adj_92 = {};
    adj_93 = {};
    adj_94 = {};
    adj_95 = {};
    adj_96 = {};
    adj_97 = {};
    adj_98 = {};
    adj_99 = {};
    adj_100 = {};
    adj_101 = {};
    adj_102 = {};
    adj_103 = {};
    adj_104 = {};
    adj_105 = {};
    adj_106 = {};
    adj_107 = {};
    adj_108 = {};
    adj_109 = {};
    adj_110 = {};
    adj_111 = {};
    adj_112 = {};
    adj_113 = {};
    adj_114 = {};
    adj_115 = {};
    adj_116 = {};
    adj_117 = {};
    adj_118 = {};
    adj_119 = {};
    adj_120 = {};
    adj_121 = {};
    adj_122 = {};
    adj_123 = {};
    adj_124 = {};
    adj_125 = {};
    adj_126 = {};
    adj_127 = {};
    adj_128 = {};
    adj_129 = {};
    adj_130 = {};
    adj_131 = {};
    adj_132 = {};
    adj_133 = {};
    adj_134 = {};
    adj_135 = {};
    adj_136 = {};
    adj_137 = {};
    adj_138 = {};
    adj_139 = {};
    adj_140 = {};
    adj_141 = {};
    adj_142 = {};
    adj_143 = {};
    adj_144 = {};
    adj_145 = {};
    adj_146 = {};
    adj_147 = {};
    adj_148 = {};
    adj_149 = {};
    adj_150 = {};
    adj_151 = {};
    adj_152 = {};
    adj_153 = {};
    adj_154 = {};
    adj_155 = {};
    adj_156 = {};
    adj_157 = {};
    adj_158 = {};
    adj_159 = {};
    adj_160 = {};
    adj_161 = {};
    adj_162 = {};
    adj_163 = {};
    adj_164 = {};
    adj_165 = {};
    adj_166 = {};
    adj_167 = {};
    adj_168 = {};
    adj_169 = {};
    adj_170 = {};
    adj_171 = {};
    adj_172 = {};
    adj_173 = {};
    adj_174 = {};
    adj_175 = {};
    adj_176 = {};
    adj_177 = {};
    adj_178 = {};
    adj_179 = {};
    adj_180 = {};
    adj_181 = {};
    adj_182 = {};
    adj_183 = {};
    adj_184 = {};
    adj_185 = {};
    adj_186 = {};
    adj_187 = {};
    adj_188 = {};
    adj_189 = {};
    adj_190 = {};
    adj_191 = {};
    adj_192 = {};
    adj_193 = {};
    adj_194 = {};
    adj_195 = {};
    adj_196 = {};
    adj_197 = {};
    adj_198 = {};
    adj_199 = {};
    adj_200 = {};
    adj_201 = {};
    adj_202 = {};
    adj_203 = {};
    adj_204 = {};
    adj_205 = {};
    adj_206 = {};
    adj_207 = {};
    adj_208 = {};
    adj_209 = {};
    adj_210 = {};
    adj_211 = {};
    adj_212 = {};
    adj_213 = {};
    adj_214 = {};
    adj_215 = {};
    adj_216 = {};
    adj_217 = {};
    adj_218 = {};
    adj_219 = {};
    adj_220 = {};
    adj_221 = {};
    adj_222 = {};
    adj_223 = {};
    adj_224 = {};
    adj_225 = {};
    adj_226 = {};
    adj_227 = {};
    adj_228 = {};
    adj_229 = {};
    adj_230 = {};
    adj_231 = {};
    adj_232 = {};
    adj_233 = {};
    adj_234 = {};
    adj_235 = {};
    adj_236 = {};
    adj_237 = {};
    adj_238 = {};
    adj_239 = {};
    adj_240 = {};
    adj_241 = {};
    adj_242 = {};
    adj_243 = {};
    adj_244 = {};
    adj_245 = {};
    adj_246 = {};
    adj_247 = {};
    adj_248 = {};
    adj_249 = {};
    adj_250 = {};
    adj_251 = {};
    adj_252 = {};
    adj_253 = {};
    adj_254 = {};
    adj_255 = {};
    adj_256 = {};
    adj_257 = {};
    adj_258 = {};
    adj_259 = {};
    adj_260 = {};
    adj_261 = {};
    adj_262 = {};
    adj_263 = {};
    adj_264 = {};
    adj_265 = {};
    adj_266 = {};
    adj_267 = {};
    adj_268 = {};
    adj_269 = {};
    adj_270 = {};
    adj_271 = {};
    adj_272 = {};
    adj_273 = {};
    adj_274 = {};
    adj_275 = {};
    adj_276 = {};
    adj_277 = {};
    adj_278 = {};
    adj_279 = {};
    adj_280 = {};
    adj_281 = {};
    adj_282 = {};
    adj_283 = {};
    adj_284 = {};
    adj_285 = {};
    adj_286 = {};
    adj_287 = {};
    adj_288 = {};
    adj_289 = {};
    adj_290 = {};
    adj_291 = {};
    adj_292 = {};
    adj_293 = {};
    adj_294 = {};
    adj_295 = {};
    adj_296 = {};
    adj_297 = {};
    adj_298 = {};
    adj_299 = {};
    adj_300 = {};
    adj_301 = {};
    adj_302 = {};
    adj_303 = {};
    adj_304 = {};
    adj_305 = {};
    adj_306 = {};
    adj_307 = {};
    adj_308 = {};
    adj_309 = {};
    adj_310 = {};
    adj_311 = {};
    adj_312 = {};
    adj_313 = {};
    adj_314 = {};
    adj_315 = {};
    adj_316 = {};
    adj_317 = {};
    adj_318 = {};
    adj_319 = {};
    adj_320 = {};
    adj_321 = {};
    adj_322 = {};
    adj_323 = {};
    adj_324 = {};
    adj_325 = {};
    adj_326 = {};
    adj_327 = {};
    adj_328 = {};
    adj_329 = {};
    adj_330 = {};
    adj_331 = {};
    adj_332 = {};
    adj_333 = {};
    adj_334 = {};
    adj_335 = {};
    adj_336 = {};
    adj_337 = {};
    adj_338 = {};
    adj_339 = {};
    adj_340 = {};
    adj_341 = {};
    adj_342 = {};
    adj_343 = {};
    adj_344 = {};
    adj_345 = {};
    adj_346 = {};
    adj_347 = {};
    adj_348 = {};
    adj_349 = {};
    adj_350 = {};
    adj_351 = {};
    adj_352 = {};
    adj_353 = {};
    adj_354 = {};
    adj_355 = {};
    adj_356 = {};
    adj_357 = {};
    adj_358 = {};
    adj_359 = {};
    adj_360 = {};
    adj_361 = {};
    adj_362 = {};
    adj_363 = {};
    adj_364 = {};
    adj_365 = {};
    adj_366 = {};
    adj_367 = {};
    adj_368 = {};
    adj_369 = {};
    adj_370 = {};
    adj_371 = {};
    adj_372 = {};
    adj_373 = {};
    adj_374 = {};
    adj_375 = {};
    adj_376 = {};
    adj_377 = {};
    adj_378 = {};
    adj_379 = {};
    adj_380 = {};
    adj_381 = {};
    adj_382 = {};
    adj_383 = {};
    adj_384 = {};
    adj_385 = {};
    adj_386 = {};
    adj_387 = {};
    adj_388 = {};
    adj_389 = {};
    adj_390 = {};
    adj_391 = {};
    adj_392 = {};
    adj_393 = {};
    adj_394 = {};
    adj_395 = {};
    adj_396 = {};
    adj_397 = {};
    adj_398 = {};
    adj_399 = {};
    adj_400 = {};
    adj_401 = {};
    adj_402 = {};
    adj_403 = {};
    adj_404 = {};
    adj_405 = {};
    adj_406 = {};
    adj_407 = {};
    adj_408 = {};
    adj_409 = {};
    adj_410 = {};
    adj_411 = {};
    adj_412 = {};
    adj_413 = {};
    adj_414 = {};
    adj_415 = {};
    adj_416 = {};
    adj_417 = {};
    adj_418 = {};
    adj_419 = {};
    adj_420 = {};
    adj_421 = {};
    adj_422 = {};
    adj_423 = {};
    adj_424 = {};
    adj_425 = {};
    adj_426 = {};
    adj_427 = {};
    adj_428 = {};
    adj_429 = {};
    adj_430 = {};
    adj_431 = {};
    adj_432 = {};
    adj_433 = {};
    adj_434 = {};
    adj_435 = {};
    adj_436 = {};
    adj_437 = {};
    adj_438 = {};
    adj_439 = {};
    adj_440 = {};
    adj_441 = {};
    adj_442 = {};
    adj_443 = {};
    adj_444 = {};
    adj_445 = {};
    adj_446 = {};
    adj_447 = {};
    adj_448 = {};
    adj_449 = {};
    adj_450 = {};
    adj_451 = {};
    adj_452 = {};
    adj_453 = {};
    adj_454 = {};
    adj_455 = {};
    adj_456 = {};
    adj_457 = {};
    adj_458 = {};
    adj_459 = {};
    adj_460 = {};
    adj_461 = {};
    adj_462 = {};
    adj_463 = {};
    adj_464 = {};
    adj_465 = {};
    adj_466 = {};
    adj_467 = {};
    adj_468 = {};
    adj_469 = {};
    adj_470 = {};
    adj_471 = {};
    adj_472 = {};
    adj_473 = {};
    adj_474 = {};
    adj_475 = {};
    adj_476 = {};
    adj_477 = {};
    adj_478 = {};
    adj_479 = {};
    adj_480 = {};
    adj_481 = {};
    adj_482 = {};
    adj_483 = {};
    adj_484 = {};
    adj_485 = {};
    adj_486 = {};
    adj_487 = {};
    adj_488 = {};
    adj_489 = {};
    adj_490 = {};
    adj_491 = {};
    adj_492 = {};
    adj_493 = {};
    adj_494 = {};
    adj_495 = {};
    adj_496 = {};
    adj_497 = {};
    adj_498 = {};
    adj_499 = {};
    adj_500 = {};
    adj_501 = {};
    adj_502 = {};
    adj_503 = {};
    adj_504 = {};
    adj_505 = {};
    adj_506 = {};
    adj_507 = {};
    adj_508 = {};
    adj_509 = {};
    adj_510 = {};
    adj_511 = {};
    adj_512 = {};
    adj_513 = {};
    adj_514 = {};
    adj_515 = {};
    adj_516 = {};
    adj_517 = {};
    adj_518 = {};
    adj_519 = {};
    adj_520 = {};
    adj_521 = {};
    adj_522 = {};
    adj_523 = {};
    adj_524 = {};
    adj_525 = {};
    adj_526 = {};
    adj_527 = {};
    adj_528 = {};
    adj_529 = {};
    adj_530 = {};
    adj_531 = {};
    adj_532 = {};
    adj_533 = {};
    adj_534 = {};
    adj_535 = {};
    adj_536 = {};
    adj_537 = {};
    adj_538 = {};
    adj_539 = {};
    adj_540 = {};
    adj_541 = {};
    adj_542 = {};
    adj_543 = {};
    adj_544 = {};
    adj_545 = {};
    adj_546 = {};
    adj_547 = {};
    adj_548 = {};
    adj_549 = {};
    adj_550 = {};
    adj_551 = {};
    adj_552 = {};
    adj_553 = {};
    adj_554 = {};
    adj_555 = {};
    adj_556 = {};
    adj_557 = {};
    adj_558 = {};
    adj_559 = {};
    adj_560 = {};
    adj_561 = {};
    adj_562 = {};
    adj_563 = {};
    adj_564 = {};
    adj_565 = {};
    adj_566 = {};
    adj_567 = {};
    adj_568 = {};
    adj_569 = {};
    adj_570 = {};
    adj_571 = {};
    adj_572 = {};
    adj_573 = {};
    adj_574 = {};
    adj_575 = {};
    adj_576 = {};
    adj_577 = {};
    adj_578 = {};
    adj_579 = {};
    adj_580 = {};
    adj_581 = {};
    adj_582 = {};
    adj_583 = {};
    adj_584 = {};
    adj_585 = {};
    adj_586 = {};
    adj_587 = {};
    adj_588 = {};
    adj_589 = {};
    adj_590 = {};
    adj_591 = {};
    adj_592 = {};
    adj_593 = {};
    adj_594 = {};
    adj_595 = {};
    adj_596 = {};
    adj_597 = {};
    adj_598 = {};
    adj_599 = {};
    adj_600 = {};
    adj_601 = {};
    adj_602 = {};
    adj_603 = {};
    adj_604 = {};
    adj_605 = {};
    adj_606 = {};
    adj_607 = {};
    adj_608 = {};
    adj_609 = {};
    adj_610 = {};
    adj_611 = {};
    adj_612 = {};
    adj_613 = {};
    adj_614 = {};
    adj_615 = {};
    adj_616 = {};
    adj_617 = {};
    adj_618 = {};
    adj_619 = {};
    adj_620 = {};
    adj_621 = {};
    adj_622 = {};
    adj_623 = {};
    adj_624 = {};
    adj_625 = {};
    adj_626 = {};
    adj_627 = {};
    adj_628 = {};
    adj_629 = {};
    adj_630 = {};
    adj_631 = {};
    adj_632 = {};
    adj_633 = {};
    adj_634 = {};
    adj_635 = {};
    adj_636 = {};
    adj_637 = {};
    adj_638 = {};
    adj_639 = {};
    adj_640 = {};
    adj_641 = {};
    adj_642 = {};
    adj_643 = {};
    adj_644 = {};
    adj_645 = {};
    adj_646 = {};
    adj_647 = {};
    adj_648 = {};
    adj_649 = {};
    adj_650 = {};
    adj_651 = {};
    adj_652 = {};
    adj_653 = {};
    adj_654 = {};
    adj_655 = {};
    adj_656 = {};
    adj_657 = {};
    adj_658 = {};
    adj_659 = {};
    adj_660 = {};
    adj_661 = {};
    adj_662 = {};
    adj_663 = {};
    adj_664 = {};
    adj_665 = {};
    adj_666 = {};
    adj_667 = {};
    adj_668 = {};
    adj_669 = {};
    adj_670 = {};
    adj_671 = {};
    adj_672 = {};
    adj_673 = {};
    adj_674 = {};
    adj_675 = {};
    adj_676 = {};
    adj_677 = {};
    adj_678 = {};
    adj_679 = {};
    adj_680 = {};
    adj_681 = {};
    adj_682 = {};
    adj_683 = {};
    adj_684 = {};
    adj_685 = {};
    adj_686 = {};
    adj_687 = {};
    adj_688 = {};
    adj_689 = {};
    adj_690 = {};
    adj_691 = {};
    adj_692 = {};
    adj_693 = {};
    adj_694 = {};
    adj_695 = {};
    adj_696 = {};
    adj_697 = {};
    adj_698 = {};
    adj_699 = {};
    adj_700 = {};
    adj_701 = {};
    adj_702 = {};
    adj_703 = {};
    adj_704 = {};
    adj_705 = {};
    adj_706 = {};
    adj_707 = {};
    adj_708 = {};
    adj_709 = {};
    adj_710 = {};
    adj_711 = {};
    adj_712 = {};
    adj_713 = {};
    adj_714 = {};
    adj_715 = {};
    adj_716 = {};
    adj_717 = {};
    adj_718 = {};
    adj_719 = {};
    adj_720 = {};
    adj_721 = {};
    adj_722 = {};
    adj_723 = {};
    adj_724 = {};
    adj_725 = {};
    adj_726 = {};
    adj_727 = {};
    adj_728 = {};
    adj_729 = {};
    adj_730 = {};
    adj_731 = {};
    adj_732 = {};
    adj_733 = {};
    adj_734 = {};
    adj_735 = {};
    adj_736 = {};
    adj_737 = {};
    adj_738 = {};
    adj_739 = {};
    adj_740 = {};
    adj_741 = {};
    adj_742 = {};
    adj_743 = {};
    adj_744 = {};
    adj_745 = {};
    adj_746 = {};
    adj_747 = {};
    adj_748 = {};
    adj_749 = {};
    adj_750 = {};
    adj_751 = {};
    adj_752 = {};
    adj_753 = {};
    adj_754 = {};
    adj_755 = {};
    adj_756 = {};
    adj_757 = {};
    adj_758 = {};
    adj_759 = {};
    adj_760 = {};
    adj_761 = {};
    adj_762 = {};
    adj_763 = {};
    adj_764 = {};
    adj_765 = {};
    adj_766 = {};
    adj_767 = {};
    adj_768 = {};
    adj_769 = {};
    adj_770 = {};
    adj_771 = {};
    adj_772 = {};
    adj_773 = {};
    adj_774 = {};
    adj_775 = {};
    adj_776 = {};
    adj_777 = {};
    adj_778 = {};
    adj_779 = {};
    adj_780 = {};
    adj_781 = {};
    adj_782 = {};
    adj_783 = {};
    adj_784 = {};
    adj_785 = {};
    adj_786 = {};
    adj_787 = {};
    adj_788 = {};
    adj_789 = {};
    adj_790 = {};
    adj_791 = {};
    adj_792 = {};
    adj_793 = {};
    adj_794 = {};
    adj_795 = {};
    adj_796 = {};
    adj_797 = {};
    adj_798 = {};
    adj_799 = {};
    adj_800 = {};
    adj_801 = {};
    adj_802 = {};
    adj_803 = {};
    adj_804 = {};
    adj_805 = {};
    adj_806 = {};
    adj_807 = {};
    adj_808 = {};
    adj_809 = {};
    adj_810 = {};
    adj_811 = {};
    adj_812 = {};
    adj_813 = {};
    adj_814 = {};
    adj_815 = {};
    adj_816 = {};
    adj_817 = {};
    adj_818 = {};
    adj_819 = {};
    adj_820 = {};
    adj_821 = {};
    adj_822 = {};
    adj_823 = {};
    adj_824 = {};
    adj_825 = {};
    adj_826 = {};
    adj_827 = {};
    adj_828 = {};
    adj_829 = {};
    adj_830 = {};
    adj_831 = {};
    adj_832 = {};
    adj_833 = {};
    adj_834 = {};
    adj_835 = {};
    adj_836 = {};
    adj_837 = {};
    adj_838 = {};
    adj_839 = {};
    adj_840 = {};
    adj_841 = {};
    adj_842 = {};
    adj_843 = {};
    adj_844 = {};
    adj_845 = {};
    adj_846 = {};
    adj_847 = {};
    adj_848 = {};
    adj_849 = {};
    adj_850 = {};
    adj_851 = {};
    adj_852 = {};
    adj_853 = {};
    adj_854 = {};
    adj_855 = {};
    adj_856 = {};
    adj_857 = {};
    adj_858 = {};
    adj_859 = {};
    adj_860 = {};
    adj_861 = {};
    adj_862 = {};
    adj_863 = {};
    adj_864 = {};
    adj_865 = {};
    adj_866 = {};
    adj_867 = {};
    adj_868 = {};
    adj_869 = {};
    adj_870 = {};
    adj_871 = {};
    adj_872 = {};
    adj_873 = {};
    adj_874 = {};
    adj_875 = {};
    adj_876 = {};
    adj_877 = {};
    adj_878 = {};
    adj_879 = {};
    adj_880 = {};
    adj_881 = {};
    adj_882 = {};
    adj_883 = {};
    adj_884 = {};
    adj_885 = {};
    adj_886 = {};
    adj_887 = {};
    adj_888 = {};
    adj_889 = {};
    adj_890 = {};
    adj_891 = {};
    adj_892 = {};
    adj_893 = {};
    adj_894 = {};
    adj_895 = {};
    adj_896 = {};
    adj_897 = {};
    adj_898 = {};
    adj_899 = {};
    adj_900 = {};
    adj_901 = {};
    adj_902 = {};
    adj_903 = {};
    adj_904 = {};
    adj_905 = {};
    adj_906 = {};
    adj_907 = {};
    adj_908 = {};
    adj_909 = {};
    adj_910 = {};
    adj_911 = {};
    adj_912 = {};
    adj_913 = {};
    adj_914 = {};
    adj_915 = {};
    adj_916 = {};
    adj_917 = {};
    adj_918 = {};
    adj_919 = {};
    adj_920 = {};
    adj_921 = {};
    adj_922 = {};
    adj_923 = {};
    adj_924 = {};
    adj_925 = {};
    adj_926 = {};
    adj_927 = {};
    adj_928 = {};
    adj_929 = {};
    adj_930 = {};
    adj_931 = {};
    adj_932 = {};
    adj_933 = {};
    adj_934 = {};
    adj_935 = {};
    adj_936 = {};
    adj_937 = {};
    adj_938 = {};
    adj_939 = {};
    adj_940 = {};
    adj_941 = {};
    adj_942 = {};
    adj_943 = {};
    adj_944 = {};
    adj_945 = {};
    adj_946 = {};
    adj_947 = {};
    adj_948 = {};
    adj_949 = {};
    adj_950 = {};
    adj_951 = {};
    adj_952 = {};
    adj_953 = {};
    adj_954 = {};
    adj_955 = {};
    adj_956 = {};
    adj_957 = {};
    adj_958 = {};
    adj_959 = {};
    adj_960 = {};
    adj_961 = {};
    adj_962 = {};
    adj_963 = {};
    adj_964 = {};
    adj_965 = {};
    adj_966 = {};
    adj_967 = {};
    adj_968 = {};
    adj_969 = {};
    adj_970 = {};
    adj_971 = {};
    adj_972 = {};
    adj_973 = {};
    adj_974 = {};
    adj_975 = {};
    adj_976 = {};
    adj_977 = {};
    adj_978 = {};
    adj_979 = {};
    adj_980 = {};
    adj_981 = {};
    adj_982 = {};
    adj_983 = {};
    adj_984 = {};
    adj_985 = {};
    adj_986 = {};
    adj_987 = {};
    adj_988 = {};
    adj_989 = {};
    adj_990 = {};
    adj_991 = {};
    adj_992 = {};
    adj_993 = {};
    adj_994 = {};
    adj_995 = {};
    adj_996 = {};
    adj_997 = {};
    adj_998 = {};
    adj_999 = {};
    adj_1000 = {};
    adj_1001 = {};
    adj_1002 = {};
    adj_1003 = {};
    adj_1004 = {};
    adj_1005 = {};
    adj_1006 = {};
    adj_1007 = {};
    adj_1008 = {};
    adj_1009 = {};
    adj_1010 = {};
    adj_1011 = {};
    adj_1012 = {};
    adj_1013 = {};
    adj_1014 = {};
    adj_1015 = {};
    adj_1016 = {};
    adj_1017 = {};
    adj_1018 = {};
    adj_1019 = {};
    adj_1020 = {};
    adj_1021 = {};
    adj_1022 = {};
    adj_1023 = {};
    adj_1024 = {};
    adj_1025 = {};
    adj_1026 = {};
    adj_1027 = {};
    adj_1028 = {};
    adj_1029 = {};
    adj_1030 = {};
    adj_1031 = {};
    adj_1032 = {};
    adj_1033 = {};
    adj_1034 = {};
    adj_1035 = {};
    adj_1036 = {};
    adj_1037 = {};
    adj_1038 = {};
    adj_1039 = {};
    adj_1040 = {};
    adj_1041 = {};
    adj_1042 = {};
    adj_1043 = {};
    adj_1044 = {};
    adj_1045 = {};
    adj_1046 = {};
    adj_1047 = {};
    adj_1048 = {};
    adj_1049 = {};
    adj_1050 = {};
    adj_1051 = {};
    adj_1052 = {};
    adj_1053 = {};
    adj_1054 = {};
    adj_1055 = {};
    adj_1056 = {};
    adj_1057 = {};
    adj_1058 = {};
    adj_1059 = {};
    adj_1060 = {};
    adj_1061 = {};
    adj_1062 = {};
    adj_1063 = {};
    adj_1064 = {};
    adj_1065 = {};
    adj_1066 = {};
    adj_1067 = {};
    adj_1068 = {};
    adj_1069 = {};
    adj_1070 = {};
    adj_1071 = {};
    adj_1072 = {};
    adj_1073 = {};
    adj_1074 = {};
    adj_1075 = {};
    adj_1076 = {};
    adj_1077 = {};
    adj_1078 = {};
    adj_1079 = {};
    adj_1080 = {};
    adj_1081 = {};
    adj_1082 = {};
    adj_1083 = {};
    adj_1084 = {};
    adj_1085 = {};
    adj_1086 = {};
    adj_1087 = {};
    adj_1088 = {};
    adj_1089 = {};
    adj_1090 = {};
    adj_1091 = {};
    adj_1092 = {};
    adj_1093 = {};
    adj_1094 = {};
    adj_1095 = {};
    adj_1096 = {};
    adj_1097 = {};
    adj_1098 = {};
    adj_1099 = {};
    adj_1100 = {};
    adj_1101 = {};
        // niter -= 1                                                                         <L 330>
        var_13 = wp::sub(var_2, var_12);
        // node = stack                                                                       <L 331>
        var_14 = wp::copy(var_0);
        // if node == -1:                                                                     <L 333>
        var_16 = (var_14 == var_15);
        if (var_16) {
            // wp.printf("ERROR: Invalid node number\n")                                      <L 334>
            printf(var_17);
            // return -1, (rx, ry, rz)                                                        <L 335>
            var_19 = wp::tuple(var_4, var_6, var_8);
            ret_0 = var_18;
            ret_1 = var_19;
            goto label2;
        }
        // vmin = oct_aabb[node, 0] - oct_aabb[node, 1]                                       <L 337>
        var_21 = wp::address(var_oct_aabb, var_14, var_20);
        var_23 = wp::address(var_oct_aabb, var_14, var_22);
        var_25 = wp::load(var_21);
        var_26 = wp::load(var_23);
        var_24 = wp::sub(var_25, var_26);
        // vmax = oct_aabb[node, 0] + oct_aabb[node, 1]                                       <L 338>
        var_28 = wp::address(var_oct_aabb, var_14, var_27);
        var_30 = wp::address(var_oct_aabb, var_14, var_29);
        var_32 = wp::load(var_28);
        var_33 = wp::load(var_30);
        var_31 = wp::add(var_32, var_33);
        // if (                                                                               <L 340>
        // p[0] + eps < vmin[0]                                                               <L 341>
        var_36 = wp::extract(var_p, var_35);
        var_37 = wp::add(var_36, var_9);
        var_39 = wp::extract(var_24, var_38);
        var_40 = (var_37 < var_39);
        var_34 = var_40;
        if (!var_34) {
            // or p[0] - eps > vmax[0]                                                        <L 342>
            var_42 = wp::extract(var_p, var_41);
            var_43 = wp::sub(var_42, var_9);
            var_45 = wp::extract(var_31, var_44);
            var_46 = (var_43 > var_45);
            var_34 = var_34 || var_46;
        }
        if (!var_34) {
            // or p[1] + eps < vmin[1]                                                        <L 343>
            var_48 = wp::extract(var_p, var_47);
            var_49 = wp::add(var_48, var_9);
            var_51 = wp::extract(var_24, var_50);
            var_52 = (var_49 < var_51);
            var_34 = var_34 || var_52;
        }
        if (!var_34) {
            // or p[1] - eps > vmax[1]                                                        <L 344>
            var_54 = wp::extract(var_p, var_53);
            var_55 = wp::sub(var_54, var_9);
            var_57 = wp::extract(var_31, var_56);
            var_58 = (var_55 > var_57);
            var_34 = var_34 || var_58;
        }
        if (!var_34) {
            // or p[2] + eps < vmin[2]                                                        <L 345>
            var_60 = wp::extract(var_p, var_59);
            var_61 = wp::add(var_60, var_9);
            var_63 = wp::extract(var_24, var_62);
            var_64 = (var_61 < var_63);
            var_34 = var_34 || var_64;
        }
        if (!var_34) {
            // or p[2] - eps > vmax[2]                                                        <L 346>
            var_66 = wp::extract(var_p, var_65);
            var_67 = wp::sub(var_66, var_9);
            var_69 = wp::extract(var_31, var_68);
            var_70 = (var_67 > var_69);
            var_34 = var_34 || var_70;
        }
        if (var_34) {
            // continue                                                                       <L 348>
            wp::assign(var_2, var_13);
            goto start_while_0;
        }
        var_71 = wp::where(var_34, var_2, var_13);
        // coord = wp.cw_div(p - vmin, vmax - vmin)                                           <L 350>
        var_72 = wp::sub(var_p, var_24);
        var_73 = wp::sub(var_31, var_24);
        var_74 = wp::cw_div(var_72, var_73);
        // child0 = oct_child[node][0]                                                        <L 354>
        var_75 = wp::address(var_oct_child, var_14);
        var_78 = wp::load(var_75);
        var_77 = wp::extract(var_78, var_76);
        // if (                                                                               <L 356>
        // int(child0 == -1)                                                                  <L 357>
        var_80 = (var_77 == var_79);
        var_81 = wp::int(var_80);
        // & int(oct_child[node][1] == -1)                                                    <L 358>
        var_82 = wp::address(var_oct_child, var_14);
        var_85 = wp::load(var_82);
        var_84 = wp::extract(var_85, var_83);
        var_87 = (var_84 == var_86);
        var_88 = wp::int(var_87);
        var_89 = wp::bit_and(var_81, var_88);
        // & int(oct_child[node][2] == -1)                                                    <L 359>
        var_90 = wp::address(var_oct_child, var_14);
        var_93 = wp::load(var_90);
        var_92 = wp::extract(var_93, var_91);
        var_95 = (var_92 == var_94);
        var_96 = wp::int(var_95);
        var_97 = wp::bit_and(var_89, var_96);
        // & int(oct_child[node][3] == -1)                                                    <L 360>
        var_98 = wp::address(var_oct_child, var_14);
        var_101 = wp::load(var_98);
        var_100 = wp::extract(var_101, var_99);
        var_103 = (var_100 == var_102);
        var_104 = wp::int(var_103);
        var_105 = wp::bit_and(var_97, var_104);
        // & int(oct_child[node][4] == -1)                                                    <L 361>
        var_106 = wp::address(var_oct_child, var_14);
        var_109 = wp::load(var_106);
        var_108 = wp::extract(var_109, var_107);
        var_111 = (var_108 == var_110);
        var_112 = wp::int(var_111);
        var_113 = wp::bit_and(var_105, var_112);
        // & int(oct_child[node][5] == -1)                                                    <L 362>
        var_114 = wp::address(var_oct_child, var_14);
        var_117 = wp::load(var_114);
        var_116 = wp::extract(var_117, var_115);
        var_119 = (var_116 == var_118);
        var_120 = wp::int(var_119);
        var_121 = wp::bit_and(var_113, var_120);
        // & int(oct_child[node][6] == -1)                                                    <L 363>
        var_122 = wp::address(var_oct_child, var_14);
        var_125 = wp::load(var_122);
        var_124 = wp::extract(var_125, var_123);
        var_127 = (var_124 == var_126);
        var_128 = wp::int(var_127);
        var_129 = wp::bit_and(var_121, var_128);
        // & int(oct_child[node][7] == -1)                                                    <L 364>
        var_130 = wp::address(var_oct_child, var_14);
        var_133 = wp::load(var_130);
        var_132 = wp::extract(var_133, var_131);
        var_135 = (var_132 == var_134);
        var_136 = wp::int(var_135);
        var_137 = wp::bit_and(var_129, var_136);
        // ) != 0:                                                                            <L 365>
        var_139 = (var_137 != var_138);
        if (var_139) {
            // if not grad:                                                                   <L 366>
            var_140 = wp::unot(var_grad);
            if (var_140) {
                // for j in range(8):                                                         <L 367>
                // rx[j] = (                                                                  <L 368>
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_143 = wp::bit_and(var_141, var_142);
                if (var_143) {
                    var_145 = wp::extract(var_74, var_144);
                }
                if (!var_143) {
                    var_148 = wp::extract(var_74, var_147);
                    var_149 = wp::sub(var_146, var_148);
                }
                var_150 = wp::where(var_143, var_145, var_149);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_152 = wp::bit_and(var_141, var_151);
                if (var_152) {
                    var_154 = wp::extract(var_74, var_153);
                }
                if (!var_152) {
                    var_157 = wp::extract(var_74, var_156);
                    var_158 = wp::sub(var_155, var_157);
                }
                var_159 = wp::where(var_152, var_154, var_158);
                var_160 = wp::mul(var_150, var_159);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_162 = wp::bit_and(var_141, var_161);
                if (var_162) {
                    var_164 = wp::extract(var_74, var_163);
                }
                if (!var_162) {
                    var_167 = wp::extract(var_74, var_166);
                    var_168 = wp::sub(var_165, var_167);
                }
                var_169 = wp::where(var_162, var_164, var_168);
                var_170 = wp::mul(var_160, var_169);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_141, var_170);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_173 = wp::bit_and(var_171, var_172);
                if (var_173) {
                    var_175 = wp::extract(var_74, var_174);
                }
                if (!var_173) {
                    var_178 = wp::extract(var_74, var_177);
                    var_179 = wp::sub(var_176, var_178);
                }
                var_180 = wp::where(var_173, var_175, var_179);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_182 = wp::bit_and(var_171, var_181);
                if (var_182) {
                    var_184 = wp::extract(var_74, var_183);
                }
                if (!var_182) {
                    var_187 = wp::extract(var_74, var_186);
                    var_188 = wp::sub(var_185, var_187);
                }
                var_189 = wp::where(var_182, var_184, var_188);
                var_190 = wp::mul(var_180, var_189);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_192 = wp::bit_and(var_171, var_191);
                if (var_192) {
                    var_194 = wp::extract(var_74, var_193);
                }
                if (!var_192) {
                    var_197 = wp::extract(var_74, var_196);
                    var_198 = wp::sub(var_195, var_197);
                }
                var_199 = wp::where(var_192, var_194, var_198);
                var_200 = wp::mul(var_190, var_199);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_171, var_200);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_203 = wp::bit_and(var_201, var_202);
                if (var_203) {
                    var_205 = wp::extract(var_74, var_204);
                }
                if (!var_203) {
                    var_208 = wp::extract(var_74, var_207);
                    var_209 = wp::sub(var_206, var_208);
                }
                var_210 = wp::where(var_203, var_205, var_209);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_212 = wp::bit_and(var_201, var_211);
                if (var_212) {
                    var_214 = wp::extract(var_74, var_213);
                }
                if (!var_212) {
                    var_217 = wp::extract(var_74, var_216);
                    var_218 = wp::sub(var_215, var_217);
                }
                var_219 = wp::where(var_212, var_214, var_218);
                var_220 = wp::mul(var_210, var_219);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_222 = wp::bit_and(var_201, var_221);
                if (var_222) {
                    var_224 = wp::extract(var_74, var_223);
                }
                if (!var_222) {
                    var_227 = wp::extract(var_74, var_226);
                    var_228 = wp::sub(var_225, var_227);
                }
                var_229 = wp::where(var_222, var_224, var_228);
                var_230 = wp::mul(var_220, var_229);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_201, var_230);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_233 = wp::bit_and(var_231, var_232);
                if (var_233) {
                    var_235 = wp::extract(var_74, var_234);
                }
                if (!var_233) {
                    var_238 = wp::extract(var_74, var_237);
                    var_239 = wp::sub(var_236, var_238);
                }
                var_240 = wp::where(var_233, var_235, var_239);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_242 = wp::bit_and(var_231, var_241);
                if (var_242) {
                    var_244 = wp::extract(var_74, var_243);
                }
                if (!var_242) {
                    var_247 = wp::extract(var_74, var_246);
                    var_248 = wp::sub(var_245, var_247);
                }
                var_249 = wp::where(var_242, var_244, var_248);
                var_250 = wp::mul(var_240, var_249);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_252 = wp::bit_and(var_231, var_251);
                if (var_252) {
                    var_254 = wp::extract(var_74, var_253);
                }
                if (!var_252) {
                    var_257 = wp::extract(var_74, var_256);
                    var_258 = wp::sub(var_255, var_257);
                }
                var_259 = wp::where(var_252, var_254, var_258);
                var_260 = wp::mul(var_250, var_259);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_231, var_260);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_263 = wp::bit_and(var_261, var_262);
                if (var_263) {
                    var_265 = wp::extract(var_74, var_264);
                }
                if (!var_263) {
                    var_268 = wp::extract(var_74, var_267);
                    var_269 = wp::sub(var_266, var_268);
                }
                var_270 = wp::where(var_263, var_265, var_269);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_272 = wp::bit_and(var_261, var_271);
                if (var_272) {
                    var_274 = wp::extract(var_74, var_273);
                }
                if (!var_272) {
                    var_277 = wp::extract(var_74, var_276);
                    var_278 = wp::sub(var_275, var_277);
                }
                var_279 = wp::where(var_272, var_274, var_278);
                var_280 = wp::mul(var_270, var_279);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_282 = wp::bit_and(var_261, var_281);
                if (var_282) {
                    var_284 = wp::extract(var_74, var_283);
                }
                if (!var_282) {
                    var_287 = wp::extract(var_74, var_286);
                    var_288 = wp::sub(var_285, var_287);
                }
                var_289 = wp::where(var_282, var_284, var_288);
                var_290 = wp::mul(var_280, var_289);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_261, var_290);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_293 = wp::bit_and(var_291, var_292);
                if (var_293) {
                    var_295 = wp::extract(var_74, var_294);
                }
                if (!var_293) {
                    var_298 = wp::extract(var_74, var_297);
                    var_299 = wp::sub(var_296, var_298);
                }
                var_300 = wp::where(var_293, var_295, var_299);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_302 = wp::bit_and(var_291, var_301);
                if (var_302) {
                    var_304 = wp::extract(var_74, var_303);
                }
                if (!var_302) {
                    var_307 = wp::extract(var_74, var_306);
                    var_308 = wp::sub(var_305, var_307);
                }
                var_309 = wp::where(var_302, var_304, var_308);
                var_310 = wp::mul(var_300, var_309);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_312 = wp::bit_and(var_291, var_311);
                if (var_312) {
                    var_314 = wp::extract(var_74, var_313);
                }
                if (!var_312) {
                    var_317 = wp::extract(var_74, var_316);
                    var_318 = wp::sub(var_315, var_317);
                }
                var_319 = wp::where(var_312, var_314, var_318);
                var_320 = wp::mul(var_310, var_319);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_291, var_320);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_323 = wp::bit_and(var_321, var_322);
                if (var_323) {
                    var_325 = wp::extract(var_74, var_324);
                }
                if (!var_323) {
                    var_328 = wp::extract(var_74, var_327);
                    var_329 = wp::sub(var_326, var_328);
                }
                var_330 = wp::where(var_323, var_325, var_329);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_332 = wp::bit_and(var_321, var_331);
                if (var_332) {
                    var_334 = wp::extract(var_74, var_333);
                }
                if (!var_332) {
                    var_337 = wp::extract(var_74, var_336);
                    var_338 = wp::sub(var_335, var_337);
                }
                var_339 = wp::where(var_332, var_334, var_338);
                var_340 = wp::mul(var_330, var_339);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_342 = wp::bit_and(var_321, var_341);
                if (var_342) {
                    var_344 = wp::extract(var_74, var_343);
                }
                if (!var_342) {
                    var_347 = wp::extract(var_74, var_346);
                    var_348 = wp::sub(var_345, var_347);
                }
                var_349 = wp::where(var_342, var_344, var_348);
                var_350 = wp::mul(var_340, var_349);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_321, var_350);
                // (coord[0] if j & 1 else 1.0 - coord[0])                                    <L 369>
                var_353 = wp::bit_and(var_351, var_352);
                if (var_353) {
                    var_355 = wp::extract(var_74, var_354);
                }
                if (!var_353) {
                    var_358 = wp::extract(var_74, var_357);
                    var_359 = wp::sub(var_356, var_358);
                }
                var_360 = wp::where(var_353, var_355, var_359);
                // * (coord[1] if j & 2 else 1.0 - coord[1])                                  <L 370>
                var_362 = wp::bit_and(var_351, var_361);
                if (var_362) {
                    var_364 = wp::extract(var_74, var_363);
                }
                if (!var_362) {
                    var_367 = wp::extract(var_74, var_366);
                    var_368 = wp::sub(var_365, var_367);
                }
                var_369 = wp::where(var_362, var_364, var_368);
                var_370 = wp::mul(var_360, var_369);
                // * (coord[2] if j & 4 else 1.0 - coord[2])                                  <L 371>
                var_372 = wp::bit_and(var_351, var_371);
                if (var_372) {
                    var_374 = wp::extract(var_74, var_373);
                }
                if (!var_372) {
                    var_377 = wp::extract(var_74, var_376);
                    var_378 = wp::sub(var_375, var_377);
                }
                var_379 = wp::where(var_372, var_374, var_378);
                var_380 = wp::mul(var_370, var_379);
                // rx[j] = (                                                                  <L 368>
                wp::assign_inplace(var_4, var_351, var_380);
            }
            if (!var_140) {
                // inv_cell = wp.cw_div(wp.vec3(1.0), vmax - vmin)                            <L 375>
                var_382 = wp::vec_t<3, wp::float32>(var_381);
                var_383 = wp::sub(var_31, var_24);
                var_384 = wp::cw_div(var_382, var_383);
                // for j in range(8):                                                         <L 376>
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_387 = wp::bit_and(var_385, var_386);
                if (var_387) {
                }
                if (!var_387) {
                }
                var_390 = wp::where(var_387, var_388, var_389);
                var_392 = wp::bit_and(var_385, var_391);
                if (var_392) {
                    var_394 = wp::extract(var_74, var_393);
                }
                if (!var_392) {
                    var_397 = wp::extract(var_74, var_396);
                    var_398 = wp::sub(var_395, var_397);
                }
                var_399 = wp::where(var_392, var_394, var_398);
                var_400 = wp::mul(var_390, var_399);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_402 = wp::bit_and(var_385, var_401);
                if (var_402) {
                    var_404 = wp::extract(var_74, var_403);
                }
                if (!var_402) {
                    var_407 = wp::extract(var_74, var_406);
                    var_408 = wp::sub(var_405, var_407);
                }
                var_409 = wp::where(var_402, var_404, var_408);
                var_410 = wp::mul(var_400, var_409);
                var_412 = wp::extract(var_384, var_411);
                var_413 = wp::mul(var_410, var_412);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_385, var_413);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_415 = wp::bit_and(var_385, var_414);
                if (var_415) {
                    var_417 = wp::extract(var_74, var_416);
                }
                if (!var_415) {
                    var_420 = wp::extract(var_74, var_419);
                    var_421 = wp::sub(var_418, var_420);
                }
                var_422 = wp::where(var_415, var_417, var_421);
                var_424 = wp::bit_and(var_385, var_423);
                if (var_424) {
                }
                if (!var_424) {
                }
                var_427 = wp::where(var_424, var_425, var_426);
                var_428 = wp::mul(var_422, var_427);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_430 = wp::bit_and(var_385, var_429);
                if (var_430) {
                    var_432 = wp::extract(var_74, var_431);
                }
                if (!var_430) {
                    var_435 = wp::extract(var_74, var_434);
                    var_436 = wp::sub(var_433, var_435);
                }
                var_437 = wp::where(var_430, var_432, var_436);
                var_438 = wp::mul(var_428, var_437);
                var_440 = wp::extract(var_384, var_439);
                var_441 = wp::mul(var_438, var_440);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_385, var_441);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_443 = wp::bit_and(var_385, var_442);
                if (var_443) {
                    var_445 = wp::extract(var_74, var_444);
                }
                if (!var_443) {
                    var_448 = wp::extract(var_74, var_447);
                    var_449 = wp::sub(var_446, var_448);
                }
                var_450 = wp::where(var_443, var_445, var_449);
                var_452 = wp::bit_and(var_385, var_451);
                if (var_452) {
                    var_454 = wp::extract(var_74, var_453);
                }
                if (!var_452) {
                    var_457 = wp::extract(var_74, var_456);
                    var_458 = wp::sub(var_455, var_457);
                }
                var_459 = wp::where(var_452, var_454, var_458);
                var_460 = wp::mul(var_450, var_459);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_462 = wp::bit_and(var_385, var_461);
                if (var_462) {
                }
                if (!var_462) {
                }
                var_465 = wp::where(var_462, var_463, var_464);
                var_466 = wp::mul(var_460, var_465);
                var_468 = wp::extract(var_384, var_467);
                var_469 = wp::mul(var_466, var_468);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_385, var_469);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_472 = wp::bit_and(var_470, var_471);
                if (var_472) {
                }
                if (!var_472) {
                }
                var_475 = wp::where(var_472, var_473, var_474);
                var_477 = wp::bit_and(var_470, var_476);
                if (var_477) {
                    var_479 = wp::extract(var_74, var_478);
                }
                if (!var_477) {
                    var_482 = wp::extract(var_74, var_481);
                    var_483 = wp::sub(var_480, var_482);
                }
                var_484 = wp::where(var_477, var_479, var_483);
                var_485 = wp::mul(var_475, var_484);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_487 = wp::bit_and(var_470, var_486);
                if (var_487) {
                    var_489 = wp::extract(var_74, var_488);
                }
                if (!var_487) {
                    var_492 = wp::extract(var_74, var_491);
                    var_493 = wp::sub(var_490, var_492);
                }
                var_494 = wp::where(var_487, var_489, var_493);
                var_495 = wp::mul(var_485, var_494);
                var_497 = wp::extract(var_384, var_496);
                var_498 = wp::mul(var_495, var_497);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_470, var_498);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_500 = wp::bit_and(var_470, var_499);
                if (var_500) {
                    var_502 = wp::extract(var_74, var_501);
                }
                if (!var_500) {
                    var_505 = wp::extract(var_74, var_504);
                    var_506 = wp::sub(var_503, var_505);
                }
                var_507 = wp::where(var_500, var_502, var_506);
                var_509 = wp::bit_and(var_470, var_508);
                if (var_509) {
                }
                if (!var_509) {
                }
                var_512 = wp::where(var_509, var_510, var_511);
                var_513 = wp::mul(var_507, var_512);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_515 = wp::bit_and(var_470, var_514);
                if (var_515) {
                    var_517 = wp::extract(var_74, var_516);
                }
                if (!var_515) {
                    var_520 = wp::extract(var_74, var_519);
                    var_521 = wp::sub(var_518, var_520);
                }
                var_522 = wp::where(var_515, var_517, var_521);
                var_523 = wp::mul(var_513, var_522);
                var_525 = wp::extract(var_384, var_524);
                var_526 = wp::mul(var_523, var_525);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_470, var_526);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_528 = wp::bit_and(var_470, var_527);
                if (var_528) {
                    var_530 = wp::extract(var_74, var_529);
                }
                if (!var_528) {
                    var_533 = wp::extract(var_74, var_532);
                    var_534 = wp::sub(var_531, var_533);
                }
                var_535 = wp::where(var_528, var_530, var_534);
                var_537 = wp::bit_and(var_470, var_536);
                if (var_537) {
                    var_539 = wp::extract(var_74, var_538);
                }
                if (!var_537) {
                    var_542 = wp::extract(var_74, var_541);
                    var_543 = wp::sub(var_540, var_542);
                }
                var_544 = wp::where(var_537, var_539, var_543);
                var_545 = wp::mul(var_535, var_544);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_547 = wp::bit_and(var_470, var_546);
                if (var_547) {
                }
                if (!var_547) {
                }
                var_550 = wp::where(var_547, var_548, var_549);
                var_551 = wp::mul(var_545, var_550);
                var_553 = wp::extract(var_384, var_552);
                var_554 = wp::mul(var_551, var_553);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_470, var_554);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_557 = wp::bit_and(var_555, var_556);
                if (var_557) {
                }
                if (!var_557) {
                }
                var_560 = wp::where(var_557, var_558, var_559);
                var_562 = wp::bit_and(var_555, var_561);
                if (var_562) {
                    var_564 = wp::extract(var_74, var_563);
                }
                if (!var_562) {
                    var_567 = wp::extract(var_74, var_566);
                    var_568 = wp::sub(var_565, var_567);
                }
                var_569 = wp::where(var_562, var_564, var_568);
                var_570 = wp::mul(var_560, var_569);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_572 = wp::bit_and(var_555, var_571);
                if (var_572) {
                    var_574 = wp::extract(var_74, var_573);
                }
                if (!var_572) {
                    var_577 = wp::extract(var_74, var_576);
                    var_578 = wp::sub(var_575, var_577);
                }
                var_579 = wp::where(var_572, var_574, var_578);
                var_580 = wp::mul(var_570, var_579);
                var_582 = wp::extract(var_384, var_581);
                var_583 = wp::mul(var_580, var_582);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_555, var_583);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_585 = wp::bit_and(var_555, var_584);
                if (var_585) {
                    var_587 = wp::extract(var_74, var_586);
                }
                if (!var_585) {
                    var_590 = wp::extract(var_74, var_589);
                    var_591 = wp::sub(var_588, var_590);
                }
                var_592 = wp::where(var_585, var_587, var_591);
                var_594 = wp::bit_and(var_555, var_593);
                if (var_594) {
                }
                if (!var_594) {
                }
                var_597 = wp::where(var_594, var_595, var_596);
                var_598 = wp::mul(var_592, var_597);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_600 = wp::bit_and(var_555, var_599);
                if (var_600) {
                    var_602 = wp::extract(var_74, var_601);
                }
                if (!var_600) {
                    var_605 = wp::extract(var_74, var_604);
                    var_606 = wp::sub(var_603, var_605);
                }
                var_607 = wp::where(var_600, var_602, var_606);
                var_608 = wp::mul(var_598, var_607);
                var_610 = wp::extract(var_384, var_609);
                var_611 = wp::mul(var_608, var_610);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_555, var_611);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_613 = wp::bit_and(var_555, var_612);
                if (var_613) {
                    var_615 = wp::extract(var_74, var_614);
                }
                if (!var_613) {
                    var_618 = wp::extract(var_74, var_617);
                    var_619 = wp::sub(var_616, var_618);
                }
                var_620 = wp::where(var_613, var_615, var_619);
                var_622 = wp::bit_and(var_555, var_621);
                if (var_622) {
                    var_624 = wp::extract(var_74, var_623);
                }
                if (!var_622) {
                    var_627 = wp::extract(var_74, var_626);
                    var_628 = wp::sub(var_625, var_627);
                }
                var_629 = wp::where(var_622, var_624, var_628);
                var_630 = wp::mul(var_620, var_629);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_632 = wp::bit_and(var_555, var_631);
                if (var_632) {
                }
                if (!var_632) {
                }
                var_635 = wp::where(var_632, var_633, var_634);
                var_636 = wp::mul(var_630, var_635);
                var_638 = wp::extract(var_384, var_637);
                var_639 = wp::mul(var_636, var_638);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_555, var_639);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_642 = wp::bit_and(var_640, var_641);
                if (var_642) {
                }
                if (!var_642) {
                }
                var_645 = wp::where(var_642, var_643, var_644);
                var_647 = wp::bit_and(var_640, var_646);
                if (var_647) {
                    var_649 = wp::extract(var_74, var_648);
                }
                if (!var_647) {
                    var_652 = wp::extract(var_74, var_651);
                    var_653 = wp::sub(var_650, var_652);
                }
                var_654 = wp::where(var_647, var_649, var_653);
                var_655 = wp::mul(var_645, var_654);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_657 = wp::bit_and(var_640, var_656);
                if (var_657) {
                    var_659 = wp::extract(var_74, var_658);
                }
                if (!var_657) {
                    var_662 = wp::extract(var_74, var_661);
                    var_663 = wp::sub(var_660, var_662);
                }
                var_664 = wp::where(var_657, var_659, var_663);
                var_665 = wp::mul(var_655, var_664);
                var_667 = wp::extract(var_384, var_666);
                var_668 = wp::mul(var_665, var_667);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_640, var_668);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_670 = wp::bit_and(var_640, var_669);
                if (var_670) {
                    var_672 = wp::extract(var_74, var_671);
                }
                if (!var_670) {
                    var_675 = wp::extract(var_74, var_674);
                    var_676 = wp::sub(var_673, var_675);
                }
                var_677 = wp::where(var_670, var_672, var_676);
                var_679 = wp::bit_and(var_640, var_678);
                if (var_679) {
                }
                if (!var_679) {
                }
                var_682 = wp::where(var_679, var_680, var_681);
                var_683 = wp::mul(var_677, var_682);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_685 = wp::bit_and(var_640, var_684);
                if (var_685) {
                    var_687 = wp::extract(var_74, var_686);
                }
                if (!var_685) {
                    var_690 = wp::extract(var_74, var_689);
                    var_691 = wp::sub(var_688, var_690);
                }
                var_692 = wp::where(var_685, var_687, var_691);
                var_693 = wp::mul(var_683, var_692);
                var_695 = wp::extract(var_384, var_694);
                var_696 = wp::mul(var_693, var_695);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_640, var_696);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_698 = wp::bit_and(var_640, var_697);
                if (var_698) {
                    var_700 = wp::extract(var_74, var_699);
                }
                if (!var_698) {
                    var_703 = wp::extract(var_74, var_702);
                    var_704 = wp::sub(var_701, var_703);
                }
                var_705 = wp::where(var_698, var_700, var_704);
                var_707 = wp::bit_and(var_640, var_706);
                if (var_707) {
                    var_709 = wp::extract(var_74, var_708);
                }
                if (!var_707) {
                    var_712 = wp::extract(var_74, var_711);
                    var_713 = wp::sub(var_710, var_712);
                }
                var_714 = wp::where(var_707, var_709, var_713);
                var_715 = wp::mul(var_705, var_714);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_717 = wp::bit_and(var_640, var_716);
                if (var_717) {
                }
                if (!var_717) {
                }
                var_720 = wp::where(var_717, var_718, var_719);
                var_721 = wp::mul(var_715, var_720);
                var_723 = wp::extract(var_384, var_722);
                var_724 = wp::mul(var_721, var_723);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_640, var_724);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_727 = wp::bit_and(var_725, var_726);
                if (var_727) {
                }
                if (!var_727) {
                }
                var_730 = wp::where(var_727, var_728, var_729);
                var_732 = wp::bit_and(var_725, var_731);
                if (var_732) {
                    var_734 = wp::extract(var_74, var_733);
                }
                if (!var_732) {
                    var_737 = wp::extract(var_74, var_736);
                    var_738 = wp::sub(var_735, var_737);
                }
                var_739 = wp::where(var_732, var_734, var_738);
                var_740 = wp::mul(var_730, var_739);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_742 = wp::bit_and(var_725, var_741);
                if (var_742) {
                    var_744 = wp::extract(var_74, var_743);
                }
                if (!var_742) {
                    var_747 = wp::extract(var_74, var_746);
                    var_748 = wp::sub(var_745, var_747);
                }
                var_749 = wp::where(var_742, var_744, var_748);
                var_750 = wp::mul(var_740, var_749);
                var_752 = wp::extract(var_384, var_751);
                var_753 = wp::mul(var_750, var_752);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_725, var_753);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_755 = wp::bit_and(var_725, var_754);
                if (var_755) {
                    var_757 = wp::extract(var_74, var_756);
                }
                if (!var_755) {
                    var_760 = wp::extract(var_74, var_759);
                    var_761 = wp::sub(var_758, var_760);
                }
                var_762 = wp::where(var_755, var_757, var_761);
                var_764 = wp::bit_and(var_725, var_763);
                if (var_764) {
                }
                if (!var_764) {
                }
                var_767 = wp::where(var_764, var_765, var_766);
                var_768 = wp::mul(var_762, var_767);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_770 = wp::bit_and(var_725, var_769);
                if (var_770) {
                    var_772 = wp::extract(var_74, var_771);
                }
                if (!var_770) {
                    var_775 = wp::extract(var_74, var_774);
                    var_776 = wp::sub(var_773, var_775);
                }
                var_777 = wp::where(var_770, var_772, var_776);
                var_778 = wp::mul(var_768, var_777);
                var_780 = wp::extract(var_384, var_779);
                var_781 = wp::mul(var_778, var_780);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_725, var_781);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_783 = wp::bit_and(var_725, var_782);
                if (var_783) {
                    var_785 = wp::extract(var_74, var_784);
                }
                if (!var_783) {
                    var_788 = wp::extract(var_74, var_787);
                    var_789 = wp::sub(var_786, var_788);
                }
                var_790 = wp::where(var_783, var_785, var_789);
                var_792 = wp::bit_and(var_725, var_791);
                if (var_792) {
                    var_794 = wp::extract(var_74, var_793);
                }
                if (!var_792) {
                    var_797 = wp::extract(var_74, var_796);
                    var_798 = wp::sub(var_795, var_797);
                }
                var_799 = wp::where(var_792, var_794, var_798);
                var_800 = wp::mul(var_790, var_799);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_802 = wp::bit_and(var_725, var_801);
                if (var_802) {
                }
                if (!var_802) {
                }
                var_805 = wp::where(var_802, var_803, var_804);
                var_806 = wp::mul(var_800, var_805);
                var_808 = wp::extract(var_384, var_807);
                var_809 = wp::mul(var_806, var_808);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_725, var_809);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_812 = wp::bit_and(var_810, var_811);
                if (var_812) {
                }
                if (!var_812) {
                }
                var_815 = wp::where(var_812, var_813, var_814);
                var_817 = wp::bit_and(var_810, var_816);
                if (var_817) {
                    var_819 = wp::extract(var_74, var_818);
                }
                if (!var_817) {
                    var_822 = wp::extract(var_74, var_821);
                    var_823 = wp::sub(var_820, var_822);
                }
                var_824 = wp::where(var_817, var_819, var_823);
                var_825 = wp::mul(var_815, var_824);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_827 = wp::bit_and(var_810, var_826);
                if (var_827) {
                    var_829 = wp::extract(var_74, var_828);
                }
                if (!var_827) {
                    var_832 = wp::extract(var_74, var_831);
                    var_833 = wp::sub(var_830, var_832);
                }
                var_834 = wp::where(var_827, var_829, var_833);
                var_835 = wp::mul(var_825, var_834);
                var_837 = wp::extract(var_384, var_836);
                var_838 = wp::mul(var_835, var_837);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_810, var_838);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_840 = wp::bit_and(var_810, var_839);
                if (var_840) {
                    var_842 = wp::extract(var_74, var_841);
                }
                if (!var_840) {
                    var_845 = wp::extract(var_74, var_844);
                    var_846 = wp::sub(var_843, var_845);
                }
                var_847 = wp::where(var_840, var_842, var_846);
                var_849 = wp::bit_and(var_810, var_848);
                if (var_849) {
                }
                if (!var_849) {
                }
                var_852 = wp::where(var_849, var_850, var_851);
                var_853 = wp::mul(var_847, var_852);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_855 = wp::bit_and(var_810, var_854);
                if (var_855) {
                    var_857 = wp::extract(var_74, var_856);
                }
                if (!var_855) {
                    var_860 = wp::extract(var_74, var_859);
                    var_861 = wp::sub(var_858, var_860);
                }
                var_862 = wp::where(var_855, var_857, var_861);
                var_863 = wp::mul(var_853, var_862);
                var_865 = wp::extract(var_384, var_864);
                var_866 = wp::mul(var_863, var_865);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_810, var_866);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_868 = wp::bit_and(var_810, var_867);
                if (var_868) {
                    var_870 = wp::extract(var_74, var_869);
                }
                if (!var_868) {
                    var_873 = wp::extract(var_74, var_872);
                    var_874 = wp::sub(var_871, var_873);
                }
                var_875 = wp::where(var_868, var_870, var_874);
                var_877 = wp::bit_and(var_810, var_876);
                if (var_877) {
                    var_879 = wp::extract(var_74, var_878);
                }
                if (!var_877) {
                    var_882 = wp::extract(var_74, var_881);
                    var_883 = wp::sub(var_880, var_882);
                }
                var_884 = wp::where(var_877, var_879, var_883);
                var_885 = wp::mul(var_875, var_884);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_887 = wp::bit_and(var_810, var_886);
                if (var_887) {
                }
                if (!var_887) {
                }
                var_890 = wp::where(var_887, var_888, var_889);
                var_891 = wp::mul(var_885, var_890);
                var_893 = wp::extract(var_384, var_892);
                var_894 = wp::mul(var_891, var_893);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_810, var_894);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_897 = wp::bit_and(var_895, var_896);
                if (var_897) {
                }
                if (!var_897) {
                }
                var_900 = wp::where(var_897, var_898, var_899);
                var_902 = wp::bit_and(var_895, var_901);
                if (var_902) {
                    var_904 = wp::extract(var_74, var_903);
                }
                if (!var_902) {
                    var_907 = wp::extract(var_74, var_906);
                    var_908 = wp::sub(var_905, var_907);
                }
                var_909 = wp::where(var_902, var_904, var_908);
                var_910 = wp::mul(var_900, var_909);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_912 = wp::bit_and(var_895, var_911);
                if (var_912) {
                    var_914 = wp::extract(var_74, var_913);
                }
                if (!var_912) {
                    var_917 = wp::extract(var_74, var_916);
                    var_918 = wp::sub(var_915, var_917);
                }
                var_919 = wp::where(var_912, var_914, var_918);
                var_920 = wp::mul(var_910, var_919);
                var_922 = wp::extract(var_384, var_921);
                var_923 = wp::mul(var_920, var_922);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_895, var_923);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_925 = wp::bit_and(var_895, var_924);
                if (var_925) {
                    var_927 = wp::extract(var_74, var_926);
                }
                if (!var_925) {
                    var_930 = wp::extract(var_74, var_929);
                    var_931 = wp::sub(var_928, var_930);
                }
                var_932 = wp::where(var_925, var_927, var_931);
                var_934 = wp::bit_and(var_895, var_933);
                if (var_934) {
                }
                if (!var_934) {
                }
                var_937 = wp::where(var_934, var_935, var_936);
                var_938 = wp::mul(var_932, var_937);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_940 = wp::bit_and(var_895, var_939);
                if (var_940) {
                    var_942 = wp::extract(var_74, var_941);
                }
                if (!var_940) {
                    var_945 = wp::extract(var_74, var_944);
                    var_946 = wp::sub(var_943, var_945);
                }
                var_947 = wp::where(var_940, var_942, var_946);
                var_948 = wp::mul(var_938, var_947);
                var_950 = wp::extract(var_384, var_949);
                var_951 = wp::mul(var_948, var_950);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_895, var_951);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_953 = wp::bit_and(var_895, var_952);
                if (var_953) {
                    var_955 = wp::extract(var_74, var_954);
                }
                if (!var_953) {
                    var_958 = wp::extract(var_74, var_957);
                    var_959 = wp::sub(var_956, var_958);
                }
                var_960 = wp::where(var_953, var_955, var_959);
                var_962 = wp::bit_and(var_895, var_961);
                if (var_962) {
                    var_964 = wp::extract(var_74, var_963);
                }
                if (!var_962) {
                    var_967 = wp::extract(var_74, var_966);
                    var_968 = wp::sub(var_965, var_967);
                }
                var_969 = wp::where(var_962, var_964, var_968);
                var_970 = wp::mul(var_960, var_969);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_972 = wp::bit_and(var_895, var_971);
                if (var_972) {
                }
                if (!var_972) {
                }
                var_975 = wp::where(var_972, var_973, var_974);
                var_976 = wp::mul(var_970, var_975);
                var_978 = wp::extract(var_384, var_977);
                var_979 = wp::mul(var_976, var_978);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_895, var_979);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                var_982 = wp::bit_and(var_980, var_981);
                if (var_982) {
                }
                if (!var_982) {
                }
                var_985 = wp::where(var_982, var_983, var_984);
                var_987 = wp::bit_and(var_980, var_986);
                if (var_987) {
                    var_989 = wp::extract(var_74, var_988);
                }
                if (!var_987) {
                    var_992 = wp::extract(var_74, var_991);
                    var_993 = wp::sub(var_990, var_992);
                }
                var_994 = wp::where(var_987, var_989, var_993);
                var_995 = wp::mul(var_985, var_994);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])                   <L 379>
                var_997 = wp::bit_and(var_980, var_996);
                if (var_997) {
                    var_999 = wp::extract(var_74, var_998);
                }
                if (!var_997) {
                    var_1002 = wp::extract(var_74, var_1001);
                    var_1003 = wp::sub(var_1000, var_1002);
                }
                var_1004 = wp::where(var_997, var_999, var_1003);
                var_1005 = wp::mul(var_995, var_1004);
                var_1007 = wp::extract(var_384, var_1006);
                var_1008 = wp::mul(var_1005, var_1007);
                // rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 378>
                wp::assign_inplace(var_4, var_980, var_1008);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                var_1010 = wp::bit_and(var_980, var_1009);
                if (var_1010) {
                    var_1012 = wp::extract(var_74, var_1011);
                }
                if (!var_1010) {
                    var_1015 = wp::extract(var_74, var_1014);
                    var_1016 = wp::sub(var_1013, var_1015);
                }
                var_1017 = wp::where(var_1010, var_1012, var_1016);
                var_1019 = wp::bit_and(var_980, var_1018);
                if (var_1019) {
                }
                if (!var_1019) {
                }
                var_1022 = wp::where(var_1019, var_1020, var_1021);
                var_1023 = wp::mul(var_1017, var_1022);
                // * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])                   <L 381>
                var_1025 = wp::bit_and(var_980, var_1024);
                if (var_1025) {
                    var_1027 = wp::extract(var_74, var_1026);
                }
                if (!var_1025) {
                    var_1030 = wp::extract(var_74, var_1029);
                    var_1031 = wp::sub(var_1028, var_1030);
                }
                var_1032 = wp::where(var_1025, var_1027, var_1031);
                var_1033 = wp::mul(var_1023, var_1032);
                var_1035 = wp::extract(var_384, var_1034);
                var_1036 = wp::mul(var_1033, var_1035);
                // ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)       <L 380>
                wp::assign_inplace(var_6, var_980, var_1036);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                var_1038 = wp::bit_and(var_980, var_1037);
                if (var_1038) {
                    var_1040 = wp::extract(var_74, var_1039);
                }
                if (!var_1038) {
                    var_1043 = wp::extract(var_74, var_1042);
                    var_1044 = wp::sub(var_1041, var_1043);
                }
                var_1045 = wp::where(var_1038, var_1040, var_1044);
                var_1047 = wp::bit_and(var_980, var_1046);
                if (var_1047) {
                    var_1049 = wp::extract(var_74, var_1048);
                }
                if (!var_1047) {
                    var_1052 = wp::extract(var_74, var_1051);
                    var_1053 = wp::sub(var_1050, var_1052);
                }
                var_1054 = wp::where(var_1047, var_1049, var_1053);
                var_1055 = wp::mul(var_1045, var_1054);
                // * (1.0 if j & 4 else -1.0) * inv_cell[2])                                  <L 383>
                var_1057 = wp::bit_and(var_980, var_1056);
                if (var_1057) {
                }
                if (!var_1057) {
                }
                var_1060 = wp::where(var_1057, var_1058, var_1059);
                var_1061 = wp::mul(var_1055, var_1060);
                var_1063 = wp::extract(var_384, var_1062);
                var_1064 = wp::mul(var_1061, var_1063);
                // rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])       <L 382>
                wp::assign_inplace(var_8, var_980, var_1064);
            }
            var_1065 = wp::where(var_140, var_351, var_980);
            // return node, (rx, ry, rz)                                                      <L 385>
            var_1066 = wp::tuple(var_4, var_6, var_8);
            ret_0 = var_14;
            ret_1 = var_1066;
            goto label3;
        }
        // x = 0 if coord[0] < 0.5 else 1                                                     <L 389>
        var_1068 = wp::extract(var_74, var_1067);
        var_1070 = (var_1068 < var_1069);
        if (var_1070) {
        }
        if (!var_1070) {
        }
        var_1073 = wp::where(var_1070, var_1071, var_1072);
        // y = 0 if coord[1] < 0.5 else 1                                                     <L 390>
        var_1075 = wp::extract(var_74, var_1074);
        var_1077 = (var_1075 < var_1076);
        if (var_1077) {
        }
        if (!var_1077) {
        }
        var_1080 = wp::where(var_1077, var_1078, var_1079);
        // z = 0 if coord[2] < 0.5 else 1                                                     <L 391>
        var_1082 = wp::extract(var_74, var_1081);
        var_1084 = (var_1082 < var_1083);
        if (var_1084) {
        }
        if (!var_1084) {
        }
        var_1087 = wp::where(var_1084, var_1085, var_1086);
        // child = oct_child[node][4 * z + 2 * y + x]                                         <L 392>
        var_1088 = wp::address(var_oct_child, var_14);
        var_1090 = wp::mul(var_1089, var_1087);
        var_1092 = wp::mul(var_1091, var_1080);
        var_1093 = wp::add(var_1090, var_1092);
        var_1094 = wp::add(var_1093, var_1073);
        var_1096 = wp::load(var_1088);
        var_1095 = wp::extract(var_1096, var_1094);
        // stack = child + root if child != -1 else -1                                        <L 393>
        var_1098 = (var_1095 != var_1097);
        if (var_1098) {
            var_1099 = wp::add(var_1095, var_root);
        }
        if (!var_1098) {
        }
        var_1101 = wp::where(var_1098, var_1099, var_1100);
        wp::assign(var_0, var_1101);
        wp::assign(var_2, var_71);
        wp::adj_assign(var_2, var_71, adj_2, adj_71);
        wp::adj_assign(var_0, var_1101, adj_0, adj_1101);
        wp::adj_where(var_1098, var_1099, var_1100, adj_1098, adj_1099, adj_1100, adj_1101);
        if (!var_1098) {
        }
        if (var_1098) {
            wp::adj_add(var_1095, var_root, adj_1095, adj_root, adj_1099);
        }
        // adj: stack = child + root if child != -1 else -1                                   <L 393>
        wp::adj_extract(var_1096, var_1094, adj_1088, adj_1094, adj_1095);
        wp::adj_add(var_1093, var_1073, adj_1093, adj_1073, adj_1094);
        wp::adj_add(var_1090, var_1092, adj_1090, adj_1092, adj_1093);
        wp::adj_mul(var_1091, var_1080, adj_1091, adj_1080, adj_1092);
        wp::adj_mul(var_1089, var_1087, adj_1089, adj_1087, adj_1090);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_1088);
        // adj: child = oct_child[node][4 * z + 2 * y + x]                                    <L 392>
        wp::adj_where(var_1084, var_1085, var_1086, adj_1084, adj_1085, adj_1086, adj_1087);
        if (!var_1084) {
        }
        if (var_1084) {
        }
        wp::adj_extract(var_74, var_1081, adj_74, adj_1081, adj_1082);
        // adj: z = 0 if coord[2] < 0.5 else 1                                                <L 391>
        wp::adj_where(var_1077, var_1078, var_1079, adj_1077, adj_1078, adj_1079, adj_1080);
        if (!var_1077) {
        }
        if (var_1077) {
        }
        wp::adj_extract(var_74, var_1074, adj_74, adj_1074, adj_1075);
        // adj: y = 0 if coord[1] < 0.5 else 1                                                <L 390>
        wp::adj_where(var_1070, var_1071, var_1072, adj_1070, adj_1071, adj_1072, adj_1073);
        if (!var_1070) {
        }
        if (var_1070) {
        }
        wp::adj_extract(var_74, var_1067, adj_74, adj_1067, adj_1068);
        // adj: x = 0 if coord[0] < 0.5 else 1                                                <L 389>
        if (var_139) {
            label3:;
            adj_1066 += adj_ret_1;
            adj_14 += adj_ret_0;
            // adj: return node, (rx, ry, rz)                                                 <L 385>
            wp::adj_where(var_140, var_351, var_980, adj_140, adj_351, adj_980, adj_1065);
            if (!var_140) {
                wp::adj_assign_inplace(var_8, var_980, var_1064, adj_8, adj_980, adj_1064);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_1061, var_1063, adj_1061, adj_1063, adj_1064);
                wp::adj_extract(var_384, var_1062, adj_384, adj_1062, adj_1063);
                wp::adj_mul(var_1055, var_1060, adj_1055, adj_1060, adj_1061);
                wp::adj_where(var_1057, var_1058, var_1059, adj_1057, adj_1058, adj_1059, adj_1060);
                if (!var_1057) {
                }
                if (var_1057) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_1045, var_1054, adj_1045, adj_1054, adj_1055);
                wp::adj_where(var_1047, var_1049, var_1053, adj_1047, adj_1049, adj_1053, adj_1054);
                if (!var_1047) {
                    wp::adj_sub(var_1050, var_1052, adj_1050, adj_1052, adj_1053);
                    wp::adj_extract(var_74, var_1051, adj_74, adj_1051, adj_1052);
                }
                if (var_1047) {
                    wp::adj_extract(var_74, var_1048, adj_74, adj_1048, adj_1049);
                }
                wp::adj_where(var_1038, var_1040, var_1044, adj_1038, adj_1040, adj_1044, adj_1045);
                if (!var_1038) {
                    wp::adj_sub(var_1041, var_1043, adj_1041, adj_1043, adj_1044);
                    wp::adj_extract(var_74, var_1042, adj_74, adj_1042, adj_1043);
                }
                if (var_1038) {
                    wp::adj_extract(var_74, var_1039, adj_74, adj_1039, adj_1040);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_980, var_1036, adj_6, adj_980, adj_1036);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_1033, var_1035, adj_1033, adj_1035, adj_1036);
                wp::adj_extract(var_384, var_1034, adj_384, adj_1034, adj_1035);
                wp::adj_mul(var_1023, var_1032, adj_1023, adj_1032, adj_1033);
                wp::adj_where(var_1025, var_1027, var_1031, adj_1025, adj_1027, adj_1031, adj_1032);
                if (!var_1025) {
                    wp::adj_sub(var_1028, var_1030, adj_1028, adj_1030, adj_1031);
                    wp::adj_extract(var_74, var_1029, adj_74, adj_1029, adj_1030);
                }
                if (var_1025) {
                    wp::adj_extract(var_74, var_1026, adj_74, adj_1026, adj_1027);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_1017, var_1022, adj_1017, adj_1022, adj_1023);
                wp::adj_where(var_1019, var_1020, var_1021, adj_1019, adj_1020, adj_1021, adj_1022);
                if (!var_1019) {
                }
                if (var_1019) {
                }
                wp::adj_where(var_1010, var_1012, var_1016, adj_1010, adj_1012, adj_1016, adj_1017);
                if (!var_1010) {
                    wp::adj_sub(var_1013, var_1015, adj_1013, adj_1015, adj_1016);
                    wp::adj_extract(var_74, var_1014, adj_74, adj_1014, adj_1015);
                }
                if (var_1010) {
                    wp::adj_extract(var_74, var_1011, adj_74, adj_1011, adj_1012);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_980, var_1008, adj_4, adj_980, adj_1008);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_1005, var_1007, adj_1005, adj_1007, adj_1008);
                wp::adj_extract(var_384, var_1006, adj_384, adj_1006, adj_1007);
                wp::adj_mul(var_995, var_1004, adj_995, adj_1004, adj_1005);
                wp::adj_where(var_997, var_999, var_1003, adj_997, adj_999, adj_1003, adj_1004);
                if (!var_997) {
                    wp::adj_sub(var_1000, var_1002, adj_1000, adj_1002, adj_1003);
                    wp::adj_extract(var_74, var_1001, adj_74, adj_1001, adj_1002);
                }
                if (var_997) {
                    wp::adj_extract(var_74, var_998, adj_74, adj_998, adj_999);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_985, var_994, adj_985, adj_994, adj_995);
                wp::adj_where(var_987, var_989, var_993, adj_987, adj_989, adj_993, adj_994);
                if (!var_987) {
                    wp::adj_sub(var_990, var_992, adj_990, adj_992, adj_993);
                    wp::adj_extract(var_74, var_991, adj_74, adj_991, adj_992);
                }
                if (var_987) {
                    wp::adj_extract(var_74, var_988, adj_74, adj_988, adj_989);
                }
                wp::adj_where(var_982, var_983, var_984, adj_982, adj_983, adj_984, adj_985);
                if (!var_982) {
                }
                if (var_982) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_895, var_979, adj_8, adj_895, adj_979);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_976, var_978, adj_976, adj_978, adj_979);
                wp::adj_extract(var_384, var_977, adj_384, adj_977, adj_978);
                wp::adj_mul(var_970, var_975, adj_970, adj_975, adj_976);
                wp::adj_where(var_972, var_973, var_974, adj_972, adj_973, adj_974, adj_975);
                if (!var_972) {
                }
                if (var_972) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_960, var_969, adj_960, adj_969, adj_970);
                wp::adj_where(var_962, var_964, var_968, adj_962, adj_964, adj_968, adj_969);
                if (!var_962) {
                    wp::adj_sub(var_965, var_967, adj_965, adj_967, adj_968);
                    wp::adj_extract(var_74, var_966, adj_74, adj_966, adj_967);
                }
                if (var_962) {
                    wp::adj_extract(var_74, var_963, adj_74, adj_963, adj_964);
                }
                wp::adj_where(var_953, var_955, var_959, adj_953, adj_955, adj_959, adj_960);
                if (!var_953) {
                    wp::adj_sub(var_956, var_958, adj_956, adj_958, adj_959);
                    wp::adj_extract(var_74, var_957, adj_74, adj_957, adj_958);
                }
                if (var_953) {
                    wp::adj_extract(var_74, var_954, adj_74, adj_954, adj_955);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_895, var_951, adj_6, adj_895, adj_951);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_948, var_950, adj_948, adj_950, adj_951);
                wp::adj_extract(var_384, var_949, adj_384, adj_949, adj_950);
                wp::adj_mul(var_938, var_947, adj_938, adj_947, adj_948);
                wp::adj_where(var_940, var_942, var_946, adj_940, adj_942, adj_946, adj_947);
                if (!var_940) {
                    wp::adj_sub(var_943, var_945, adj_943, adj_945, adj_946);
                    wp::adj_extract(var_74, var_944, adj_74, adj_944, adj_945);
                }
                if (var_940) {
                    wp::adj_extract(var_74, var_941, adj_74, adj_941, adj_942);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_932, var_937, adj_932, adj_937, adj_938);
                wp::adj_where(var_934, var_935, var_936, adj_934, adj_935, adj_936, adj_937);
                if (!var_934) {
                }
                if (var_934) {
                }
                wp::adj_where(var_925, var_927, var_931, adj_925, adj_927, adj_931, adj_932);
                if (!var_925) {
                    wp::adj_sub(var_928, var_930, adj_928, adj_930, adj_931);
                    wp::adj_extract(var_74, var_929, adj_74, adj_929, adj_930);
                }
                if (var_925) {
                    wp::adj_extract(var_74, var_926, adj_74, adj_926, adj_927);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_895, var_923, adj_4, adj_895, adj_923);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_920, var_922, adj_920, adj_922, adj_923);
                wp::adj_extract(var_384, var_921, adj_384, adj_921, adj_922);
                wp::adj_mul(var_910, var_919, adj_910, adj_919, adj_920);
                wp::adj_where(var_912, var_914, var_918, adj_912, adj_914, adj_918, adj_919);
                if (!var_912) {
                    wp::adj_sub(var_915, var_917, adj_915, adj_917, adj_918);
                    wp::adj_extract(var_74, var_916, adj_74, adj_916, adj_917);
                }
                if (var_912) {
                    wp::adj_extract(var_74, var_913, adj_74, adj_913, adj_914);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_900, var_909, adj_900, adj_909, adj_910);
                wp::adj_where(var_902, var_904, var_908, adj_902, adj_904, adj_908, adj_909);
                if (!var_902) {
                    wp::adj_sub(var_905, var_907, adj_905, adj_907, adj_908);
                    wp::adj_extract(var_74, var_906, adj_74, adj_906, adj_907);
                }
                if (var_902) {
                    wp::adj_extract(var_74, var_903, adj_74, adj_903, adj_904);
                }
                wp::adj_where(var_897, var_898, var_899, adj_897, adj_898, adj_899, adj_900);
                if (!var_897) {
                }
                if (var_897) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_810, var_894, adj_8, adj_810, adj_894);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_891, var_893, adj_891, adj_893, adj_894);
                wp::adj_extract(var_384, var_892, adj_384, adj_892, adj_893);
                wp::adj_mul(var_885, var_890, adj_885, adj_890, adj_891);
                wp::adj_where(var_887, var_888, var_889, adj_887, adj_888, adj_889, adj_890);
                if (!var_887) {
                }
                if (var_887) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_875, var_884, adj_875, adj_884, adj_885);
                wp::adj_where(var_877, var_879, var_883, adj_877, adj_879, adj_883, adj_884);
                if (!var_877) {
                    wp::adj_sub(var_880, var_882, adj_880, adj_882, adj_883);
                    wp::adj_extract(var_74, var_881, adj_74, adj_881, adj_882);
                }
                if (var_877) {
                    wp::adj_extract(var_74, var_878, adj_74, adj_878, adj_879);
                }
                wp::adj_where(var_868, var_870, var_874, adj_868, adj_870, adj_874, adj_875);
                if (!var_868) {
                    wp::adj_sub(var_871, var_873, adj_871, adj_873, adj_874);
                    wp::adj_extract(var_74, var_872, adj_74, adj_872, adj_873);
                }
                if (var_868) {
                    wp::adj_extract(var_74, var_869, adj_74, adj_869, adj_870);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_810, var_866, adj_6, adj_810, adj_866);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_863, var_865, adj_863, adj_865, adj_866);
                wp::adj_extract(var_384, var_864, adj_384, adj_864, adj_865);
                wp::adj_mul(var_853, var_862, adj_853, adj_862, adj_863);
                wp::adj_where(var_855, var_857, var_861, adj_855, adj_857, adj_861, adj_862);
                if (!var_855) {
                    wp::adj_sub(var_858, var_860, adj_858, adj_860, adj_861);
                    wp::adj_extract(var_74, var_859, adj_74, adj_859, adj_860);
                }
                if (var_855) {
                    wp::adj_extract(var_74, var_856, adj_74, adj_856, adj_857);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_847, var_852, adj_847, adj_852, adj_853);
                wp::adj_where(var_849, var_850, var_851, adj_849, adj_850, adj_851, adj_852);
                if (!var_849) {
                }
                if (var_849) {
                }
                wp::adj_where(var_840, var_842, var_846, adj_840, adj_842, adj_846, adj_847);
                if (!var_840) {
                    wp::adj_sub(var_843, var_845, adj_843, adj_845, adj_846);
                    wp::adj_extract(var_74, var_844, adj_74, adj_844, adj_845);
                }
                if (var_840) {
                    wp::adj_extract(var_74, var_841, adj_74, adj_841, adj_842);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_810, var_838, adj_4, adj_810, adj_838);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_835, var_837, adj_835, adj_837, adj_838);
                wp::adj_extract(var_384, var_836, adj_384, adj_836, adj_837);
                wp::adj_mul(var_825, var_834, adj_825, adj_834, adj_835);
                wp::adj_where(var_827, var_829, var_833, adj_827, adj_829, adj_833, adj_834);
                if (!var_827) {
                    wp::adj_sub(var_830, var_832, adj_830, adj_832, adj_833);
                    wp::adj_extract(var_74, var_831, adj_74, adj_831, adj_832);
                }
                if (var_827) {
                    wp::adj_extract(var_74, var_828, adj_74, adj_828, adj_829);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_815, var_824, adj_815, adj_824, adj_825);
                wp::adj_where(var_817, var_819, var_823, adj_817, adj_819, adj_823, adj_824);
                if (!var_817) {
                    wp::adj_sub(var_820, var_822, adj_820, adj_822, adj_823);
                    wp::adj_extract(var_74, var_821, adj_74, adj_821, adj_822);
                }
                if (var_817) {
                    wp::adj_extract(var_74, var_818, adj_74, adj_818, adj_819);
                }
                wp::adj_where(var_812, var_813, var_814, adj_812, adj_813, adj_814, adj_815);
                if (!var_812) {
                }
                if (var_812) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_725, var_809, adj_8, adj_725, adj_809);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_806, var_808, adj_806, adj_808, adj_809);
                wp::adj_extract(var_384, var_807, adj_384, adj_807, adj_808);
                wp::adj_mul(var_800, var_805, adj_800, adj_805, adj_806);
                wp::adj_where(var_802, var_803, var_804, adj_802, adj_803, adj_804, adj_805);
                if (!var_802) {
                }
                if (var_802) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_790, var_799, adj_790, adj_799, adj_800);
                wp::adj_where(var_792, var_794, var_798, adj_792, adj_794, adj_798, adj_799);
                if (!var_792) {
                    wp::adj_sub(var_795, var_797, adj_795, adj_797, adj_798);
                    wp::adj_extract(var_74, var_796, adj_74, adj_796, adj_797);
                }
                if (var_792) {
                    wp::adj_extract(var_74, var_793, adj_74, adj_793, adj_794);
                }
                wp::adj_where(var_783, var_785, var_789, adj_783, adj_785, adj_789, adj_790);
                if (!var_783) {
                    wp::adj_sub(var_786, var_788, adj_786, adj_788, adj_789);
                    wp::adj_extract(var_74, var_787, adj_74, adj_787, adj_788);
                }
                if (var_783) {
                    wp::adj_extract(var_74, var_784, adj_74, adj_784, adj_785);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_725, var_781, adj_6, adj_725, adj_781);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_778, var_780, adj_778, adj_780, adj_781);
                wp::adj_extract(var_384, var_779, adj_384, adj_779, adj_780);
                wp::adj_mul(var_768, var_777, adj_768, adj_777, adj_778);
                wp::adj_where(var_770, var_772, var_776, adj_770, adj_772, adj_776, adj_777);
                if (!var_770) {
                    wp::adj_sub(var_773, var_775, adj_773, adj_775, adj_776);
                    wp::adj_extract(var_74, var_774, adj_74, adj_774, adj_775);
                }
                if (var_770) {
                    wp::adj_extract(var_74, var_771, adj_74, adj_771, adj_772);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_762, var_767, adj_762, adj_767, adj_768);
                wp::adj_where(var_764, var_765, var_766, adj_764, adj_765, adj_766, adj_767);
                if (!var_764) {
                }
                if (var_764) {
                }
                wp::adj_where(var_755, var_757, var_761, adj_755, adj_757, adj_761, adj_762);
                if (!var_755) {
                    wp::adj_sub(var_758, var_760, adj_758, adj_760, adj_761);
                    wp::adj_extract(var_74, var_759, adj_74, adj_759, adj_760);
                }
                if (var_755) {
                    wp::adj_extract(var_74, var_756, adj_74, adj_756, adj_757);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_725, var_753, adj_4, adj_725, adj_753);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_750, var_752, adj_750, adj_752, adj_753);
                wp::adj_extract(var_384, var_751, adj_384, adj_751, adj_752);
                wp::adj_mul(var_740, var_749, adj_740, adj_749, adj_750);
                wp::adj_where(var_742, var_744, var_748, adj_742, adj_744, adj_748, adj_749);
                if (!var_742) {
                    wp::adj_sub(var_745, var_747, adj_745, adj_747, adj_748);
                    wp::adj_extract(var_74, var_746, adj_74, adj_746, adj_747);
                }
                if (var_742) {
                    wp::adj_extract(var_74, var_743, adj_74, adj_743, adj_744);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_730, var_739, adj_730, adj_739, adj_740);
                wp::adj_where(var_732, var_734, var_738, adj_732, adj_734, adj_738, adj_739);
                if (!var_732) {
                    wp::adj_sub(var_735, var_737, adj_735, adj_737, adj_738);
                    wp::adj_extract(var_74, var_736, adj_74, adj_736, adj_737);
                }
                if (var_732) {
                    wp::adj_extract(var_74, var_733, adj_74, adj_733, adj_734);
                }
                wp::adj_where(var_727, var_728, var_729, adj_727, adj_728, adj_729, adj_730);
                if (!var_727) {
                }
                if (var_727) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_640, var_724, adj_8, adj_640, adj_724);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_721, var_723, adj_721, adj_723, adj_724);
                wp::adj_extract(var_384, var_722, adj_384, adj_722, adj_723);
                wp::adj_mul(var_715, var_720, adj_715, adj_720, adj_721);
                wp::adj_where(var_717, var_718, var_719, adj_717, adj_718, adj_719, adj_720);
                if (!var_717) {
                }
                if (var_717) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_705, var_714, adj_705, adj_714, adj_715);
                wp::adj_where(var_707, var_709, var_713, adj_707, adj_709, adj_713, adj_714);
                if (!var_707) {
                    wp::adj_sub(var_710, var_712, adj_710, adj_712, adj_713);
                    wp::adj_extract(var_74, var_711, adj_74, adj_711, adj_712);
                }
                if (var_707) {
                    wp::adj_extract(var_74, var_708, adj_74, adj_708, adj_709);
                }
                wp::adj_where(var_698, var_700, var_704, adj_698, adj_700, adj_704, adj_705);
                if (!var_698) {
                    wp::adj_sub(var_701, var_703, adj_701, adj_703, adj_704);
                    wp::adj_extract(var_74, var_702, adj_74, adj_702, adj_703);
                }
                if (var_698) {
                    wp::adj_extract(var_74, var_699, adj_74, adj_699, adj_700);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_640, var_696, adj_6, adj_640, adj_696);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_693, var_695, adj_693, adj_695, adj_696);
                wp::adj_extract(var_384, var_694, adj_384, adj_694, adj_695);
                wp::adj_mul(var_683, var_692, adj_683, adj_692, adj_693);
                wp::adj_where(var_685, var_687, var_691, adj_685, adj_687, adj_691, adj_692);
                if (!var_685) {
                    wp::adj_sub(var_688, var_690, adj_688, adj_690, adj_691);
                    wp::adj_extract(var_74, var_689, adj_74, adj_689, adj_690);
                }
                if (var_685) {
                    wp::adj_extract(var_74, var_686, adj_74, adj_686, adj_687);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_677, var_682, adj_677, adj_682, adj_683);
                wp::adj_where(var_679, var_680, var_681, adj_679, adj_680, adj_681, adj_682);
                if (!var_679) {
                }
                if (var_679) {
                }
                wp::adj_where(var_670, var_672, var_676, adj_670, adj_672, adj_676, adj_677);
                if (!var_670) {
                    wp::adj_sub(var_673, var_675, adj_673, adj_675, adj_676);
                    wp::adj_extract(var_74, var_674, adj_74, adj_674, adj_675);
                }
                if (var_670) {
                    wp::adj_extract(var_74, var_671, adj_74, adj_671, adj_672);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_640, var_668, adj_4, adj_640, adj_668);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_665, var_667, adj_665, adj_667, adj_668);
                wp::adj_extract(var_384, var_666, adj_384, adj_666, adj_667);
                wp::adj_mul(var_655, var_664, adj_655, adj_664, adj_665);
                wp::adj_where(var_657, var_659, var_663, adj_657, adj_659, adj_663, adj_664);
                if (!var_657) {
                    wp::adj_sub(var_660, var_662, adj_660, adj_662, adj_663);
                    wp::adj_extract(var_74, var_661, adj_74, adj_661, adj_662);
                }
                if (var_657) {
                    wp::adj_extract(var_74, var_658, adj_74, adj_658, adj_659);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_645, var_654, adj_645, adj_654, adj_655);
                wp::adj_where(var_647, var_649, var_653, adj_647, adj_649, adj_653, adj_654);
                if (!var_647) {
                    wp::adj_sub(var_650, var_652, adj_650, adj_652, adj_653);
                    wp::adj_extract(var_74, var_651, adj_74, adj_651, adj_652);
                }
                if (var_647) {
                    wp::adj_extract(var_74, var_648, adj_74, adj_648, adj_649);
                }
                wp::adj_where(var_642, var_643, var_644, adj_642, adj_643, adj_644, adj_645);
                if (!var_642) {
                }
                if (var_642) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_555, var_639, adj_8, adj_555, adj_639);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_636, var_638, adj_636, adj_638, adj_639);
                wp::adj_extract(var_384, var_637, adj_384, adj_637, adj_638);
                wp::adj_mul(var_630, var_635, adj_630, adj_635, adj_636);
                wp::adj_where(var_632, var_633, var_634, adj_632, adj_633, adj_634, adj_635);
                if (!var_632) {
                }
                if (var_632) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_620, var_629, adj_620, adj_629, adj_630);
                wp::adj_where(var_622, var_624, var_628, adj_622, adj_624, adj_628, adj_629);
                if (!var_622) {
                    wp::adj_sub(var_625, var_627, adj_625, adj_627, adj_628);
                    wp::adj_extract(var_74, var_626, adj_74, adj_626, adj_627);
                }
                if (var_622) {
                    wp::adj_extract(var_74, var_623, adj_74, adj_623, adj_624);
                }
                wp::adj_where(var_613, var_615, var_619, adj_613, adj_615, adj_619, adj_620);
                if (!var_613) {
                    wp::adj_sub(var_616, var_618, adj_616, adj_618, adj_619);
                    wp::adj_extract(var_74, var_617, adj_74, adj_617, adj_618);
                }
                if (var_613) {
                    wp::adj_extract(var_74, var_614, adj_74, adj_614, adj_615);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_555, var_611, adj_6, adj_555, adj_611);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_608, var_610, adj_608, adj_610, adj_611);
                wp::adj_extract(var_384, var_609, adj_384, adj_609, adj_610);
                wp::adj_mul(var_598, var_607, adj_598, adj_607, adj_608);
                wp::adj_where(var_600, var_602, var_606, adj_600, adj_602, adj_606, adj_607);
                if (!var_600) {
                    wp::adj_sub(var_603, var_605, adj_603, adj_605, adj_606);
                    wp::adj_extract(var_74, var_604, adj_74, adj_604, adj_605);
                }
                if (var_600) {
                    wp::adj_extract(var_74, var_601, adj_74, adj_601, adj_602);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_592, var_597, adj_592, adj_597, adj_598);
                wp::adj_where(var_594, var_595, var_596, adj_594, adj_595, adj_596, adj_597);
                if (!var_594) {
                }
                if (var_594) {
                }
                wp::adj_where(var_585, var_587, var_591, adj_585, adj_587, adj_591, adj_592);
                if (!var_585) {
                    wp::adj_sub(var_588, var_590, adj_588, adj_590, adj_591);
                    wp::adj_extract(var_74, var_589, adj_74, adj_589, adj_590);
                }
                if (var_585) {
                    wp::adj_extract(var_74, var_586, adj_74, adj_586, adj_587);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_555, var_583, adj_4, adj_555, adj_583);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_580, var_582, adj_580, adj_582, adj_583);
                wp::adj_extract(var_384, var_581, adj_384, adj_581, adj_582);
                wp::adj_mul(var_570, var_579, adj_570, adj_579, adj_580);
                wp::adj_where(var_572, var_574, var_578, adj_572, adj_574, adj_578, adj_579);
                if (!var_572) {
                    wp::adj_sub(var_575, var_577, adj_575, adj_577, adj_578);
                    wp::adj_extract(var_74, var_576, adj_74, adj_576, adj_577);
                }
                if (var_572) {
                    wp::adj_extract(var_74, var_573, adj_74, adj_573, adj_574);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_560, var_569, adj_560, adj_569, adj_570);
                wp::adj_where(var_562, var_564, var_568, adj_562, adj_564, adj_568, adj_569);
                if (!var_562) {
                    wp::adj_sub(var_565, var_567, adj_565, adj_567, adj_568);
                    wp::adj_extract(var_74, var_566, adj_74, adj_566, adj_567);
                }
                if (var_562) {
                    wp::adj_extract(var_74, var_563, adj_74, adj_563, adj_564);
                }
                wp::adj_where(var_557, var_558, var_559, adj_557, adj_558, adj_559, adj_560);
                if (!var_557) {
                }
                if (var_557) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_470, var_554, adj_8, adj_470, adj_554);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_551, var_553, adj_551, adj_553, adj_554);
                wp::adj_extract(var_384, var_552, adj_384, adj_552, adj_553);
                wp::adj_mul(var_545, var_550, adj_545, adj_550, adj_551);
                wp::adj_where(var_547, var_548, var_549, adj_547, adj_548, adj_549, adj_550);
                if (!var_547) {
                }
                if (var_547) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_535, var_544, adj_535, adj_544, adj_545);
                wp::adj_where(var_537, var_539, var_543, adj_537, adj_539, adj_543, adj_544);
                if (!var_537) {
                    wp::adj_sub(var_540, var_542, adj_540, adj_542, adj_543);
                    wp::adj_extract(var_74, var_541, adj_74, adj_541, adj_542);
                }
                if (var_537) {
                    wp::adj_extract(var_74, var_538, adj_74, adj_538, adj_539);
                }
                wp::adj_where(var_528, var_530, var_534, adj_528, adj_530, adj_534, adj_535);
                if (!var_528) {
                    wp::adj_sub(var_531, var_533, adj_531, adj_533, adj_534);
                    wp::adj_extract(var_74, var_532, adj_74, adj_532, adj_533);
                }
                if (var_528) {
                    wp::adj_extract(var_74, var_529, adj_74, adj_529, adj_530);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_470, var_526, adj_6, adj_470, adj_526);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_523, var_525, adj_523, adj_525, adj_526);
                wp::adj_extract(var_384, var_524, adj_384, adj_524, adj_525);
                wp::adj_mul(var_513, var_522, adj_513, adj_522, adj_523);
                wp::adj_where(var_515, var_517, var_521, adj_515, adj_517, adj_521, adj_522);
                if (!var_515) {
                    wp::adj_sub(var_518, var_520, adj_518, adj_520, adj_521);
                    wp::adj_extract(var_74, var_519, adj_74, adj_519, adj_520);
                }
                if (var_515) {
                    wp::adj_extract(var_74, var_516, adj_74, adj_516, adj_517);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_507, var_512, adj_507, adj_512, adj_513);
                wp::adj_where(var_509, var_510, var_511, adj_509, adj_510, adj_511, adj_512);
                if (!var_509) {
                }
                if (var_509) {
                }
                wp::adj_where(var_500, var_502, var_506, adj_500, adj_502, adj_506, adj_507);
                if (!var_500) {
                    wp::adj_sub(var_503, var_505, adj_503, adj_505, adj_506);
                    wp::adj_extract(var_74, var_504, adj_74, adj_504, adj_505);
                }
                if (var_500) {
                    wp::adj_extract(var_74, var_501, adj_74, adj_501, adj_502);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_470, var_498, adj_4, adj_470, adj_498);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_495, var_497, adj_495, adj_497, adj_498);
                wp::adj_extract(var_384, var_496, adj_384, adj_496, adj_497);
                wp::adj_mul(var_485, var_494, adj_485, adj_494, adj_495);
                wp::adj_where(var_487, var_489, var_493, adj_487, adj_489, adj_493, adj_494);
                if (!var_487) {
                    wp::adj_sub(var_490, var_492, adj_490, adj_492, adj_493);
                    wp::adj_extract(var_74, var_491, adj_74, adj_491, adj_492);
                }
                if (var_487) {
                    wp::adj_extract(var_74, var_488, adj_74, adj_488, adj_489);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_475, var_484, adj_475, adj_484, adj_485);
                wp::adj_where(var_477, var_479, var_483, adj_477, adj_479, adj_483, adj_484);
                if (!var_477) {
                    wp::adj_sub(var_480, var_482, adj_480, adj_482, adj_483);
                    wp::adj_extract(var_74, var_481, adj_74, adj_481, adj_482);
                }
                if (var_477) {
                    wp::adj_extract(var_74, var_478, adj_74, adj_478, adj_479);
                }
                wp::adj_where(var_472, var_473, var_474, adj_472, adj_473, adj_474, adj_475);
                if (!var_472) {
                }
                if (var_472) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_assign_inplace(var_8, var_385, var_469, adj_8, adj_385, adj_469);
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_mul(var_466, var_468, adj_466, adj_468, adj_469);
                wp::adj_extract(var_384, var_467, adj_384, adj_467, adj_468);
                wp::adj_mul(var_460, var_465, adj_460, adj_465, adj_466);
                wp::adj_where(var_462, var_463, var_464, adj_462, adj_463, adj_464, adj_465);
                if (!var_462) {
                }
                if (var_462) {
                }
                // adj: * (1.0 if j & 4 else -1.0) * inv_cell[2])                             <L 383>
                wp::adj_mul(var_450, var_459, adj_450, adj_459, adj_460);
                wp::adj_where(var_452, var_454, var_458, adj_452, adj_454, adj_458, adj_459);
                if (!var_452) {
                    wp::adj_sub(var_455, var_457, adj_455, adj_457, adj_458);
                    wp::adj_extract(var_74, var_456, adj_74, adj_456, adj_457);
                }
                if (var_452) {
                    wp::adj_extract(var_74, var_453, adj_74, adj_453, adj_454);
                }
                wp::adj_where(var_443, var_445, var_449, adj_443, adj_445, adj_449, adj_450);
                if (!var_443) {
                    wp::adj_sub(var_446, var_448, adj_446, adj_448, adj_449);
                    wp::adj_extract(var_74, var_447, adj_74, adj_447, adj_448);
                }
                if (var_443) {
                    wp::adj_extract(var_74, var_444, adj_74, adj_444, adj_445);
                }
                // adj: rz[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 382>
                wp::adj_assign_inplace(var_6, var_385, var_441, adj_6, adj_385, adj_441);
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_mul(var_438, var_440, adj_438, adj_440, adj_441);
                wp::adj_extract(var_384, var_439, adj_384, adj_439, adj_440);
                wp::adj_mul(var_428, var_437, adj_428, adj_437, adj_438);
                wp::adj_where(var_430, var_432, var_436, adj_430, adj_432, adj_436, adj_437);
                if (!var_430) {
                    wp::adj_sub(var_433, var_435, adj_433, adj_435, adj_436);
                    wp::adj_extract(var_74, var_434, adj_74, adj_434, adj_435);
                }
                if (var_430) {
                    wp::adj_extract(var_74, var_431, adj_74, adj_431, adj_432);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[1])              <L 381>
                wp::adj_mul(var_422, var_427, adj_422, adj_427, adj_428);
                wp::adj_where(var_424, var_425, var_426, adj_424, adj_425, adj_426, adj_427);
                if (!var_424) {
                }
                if (var_424) {
                }
                wp::adj_where(var_415, var_417, var_421, adj_415, adj_417, adj_421, adj_422);
                if (!var_415) {
                    wp::adj_sub(var_418, var_420, adj_418, adj_420, adj_421);
                    wp::adj_extract(var_74, var_419, adj_74, adj_419, adj_420);
                }
                if (var_415) {
                    wp::adj_extract(var_74, var_416, adj_74, adj_416, adj_417);
                }
                // adj: ry[j] = ((coord[0] if j & 1 else 1.0 - coord[0]) * (1.0 if j & 2 else -1.0)  <L 380>
                wp::adj_assign_inplace(var_4, var_385, var_413, adj_4, adj_385, adj_413);
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                wp::adj_mul(var_410, var_412, adj_410, adj_412, adj_413);
                wp::adj_extract(var_384, var_411, adj_384, adj_411, adj_412);
                wp::adj_mul(var_400, var_409, adj_400, adj_409, adj_410);
                wp::adj_where(var_402, var_404, var_408, adj_402, adj_404, adj_408, adj_409);
                if (!var_402) {
                    wp::adj_sub(var_405, var_407, adj_405, adj_407, adj_408);
                    wp::adj_extract(var_74, var_406, adj_74, adj_406, adj_407);
                }
                if (var_402) {
                    wp::adj_extract(var_74, var_403, adj_74, adj_403, adj_404);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2]) * inv_cell[0])              <L 379>
                wp::adj_mul(var_390, var_399, adj_390, adj_399, adj_400);
                wp::adj_where(var_392, var_394, var_398, adj_392, adj_394, adj_398, adj_399);
                if (!var_392) {
                    wp::adj_sub(var_395, var_397, adj_395, adj_397, adj_398);
                    wp::adj_extract(var_74, var_396, adj_74, adj_396, adj_397);
                }
                if (var_392) {
                    wp::adj_extract(var_74, var_393, adj_74, adj_393, adj_394);
                }
                wp::adj_where(var_387, var_388, var_389, adj_387, adj_388, adj_389, adj_390);
                if (!var_387) {
                }
                if (var_387) {
                }
                // adj: rx[j] = ((1.0 if j & 1 else -1.0) * (coord[1] if j & 2 else 1.0 - coord[1])  <L 378>
                // adj: for j in range(8):                                                    <L 376>
                wp::adj_cw_div(var_382, var_383, var_384, adj_382, adj_383, adj_384);
                wp::adj_sub(var_31, var_24, adj_31, adj_24, adj_383);
                wp::adj_vec_t(var_381, adj_381, adj_382);
                // adj: inv_cell = wp.cw_div(wp.vec3(1.0), vmax - vmin)                       <L 375>
            }
            if (var_140) {
                wp::adj_assign_inplace(var_4, var_351, var_380, adj_4, adj_351, adj_380);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_370, var_379, adj_370, adj_379, adj_380);
                wp::adj_where(var_372, var_374, var_378, adj_372, adj_374, adj_378, adj_379);
                if (!var_372) {
                    wp::adj_sub(var_375, var_377, adj_375, adj_377, adj_378);
                    wp::adj_extract(var_74, var_376, adj_74, adj_376, adj_377);
                }
                if (var_372) {
                    wp::adj_extract(var_74, var_373, adj_74, adj_373, adj_374);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_360, var_369, adj_360, adj_369, adj_370);
                wp::adj_where(var_362, var_364, var_368, adj_362, adj_364, adj_368, adj_369);
                if (!var_362) {
                    wp::adj_sub(var_365, var_367, adj_365, adj_367, adj_368);
                    wp::adj_extract(var_74, var_366, adj_74, adj_366, adj_367);
                }
                if (var_362) {
                    wp::adj_extract(var_74, var_363, adj_74, adj_363, adj_364);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_353, var_355, var_359, adj_353, adj_355, adj_359, adj_360);
                if (!var_353) {
                    wp::adj_sub(var_356, var_358, adj_356, adj_358, adj_359);
                    wp::adj_extract(var_74, var_357, adj_74, adj_357, adj_358);
                }
                if (var_353) {
                    wp::adj_extract(var_74, var_354, adj_74, adj_354, adj_355);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_321, var_350, adj_4, adj_321, adj_350);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_340, var_349, adj_340, adj_349, adj_350);
                wp::adj_where(var_342, var_344, var_348, adj_342, adj_344, adj_348, adj_349);
                if (!var_342) {
                    wp::adj_sub(var_345, var_347, adj_345, adj_347, adj_348);
                    wp::adj_extract(var_74, var_346, adj_74, adj_346, adj_347);
                }
                if (var_342) {
                    wp::adj_extract(var_74, var_343, adj_74, adj_343, adj_344);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_330, var_339, adj_330, adj_339, adj_340);
                wp::adj_where(var_332, var_334, var_338, adj_332, adj_334, adj_338, adj_339);
                if (!var_332) {
                    wp::adj_sub(var_335, var_337, adj_335, adj_337, adj_338);
                    wp::adj_extract(var_74, var_336, adj_74, adj_336, adj_337);
                }
                if (var_332) {
                    wp::adj_extract(var_74, var_333, adj_74, adj_333, adj_334);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_323, var_325, var_329, adj_323, adj_325, adj_329, adj_330);
                if (!var_323) {
                    wp::adj_sub(var_326, var_328, adj_326, adj_328, adj_329);
                    wp::adj_extract(var_74, var_327, adj_74, adj_327, adj_328);
                }
                if (var_323) {
                    wp::adj_extract(var_74, var_324, adj_74, adj_324, adj_325);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_291, var_320, adj_4, adj_291, adj_320);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_310, var_319, adj_310, adj_319, adj_320);
                wp::adj_where(var_312, var_314, var_318, adj_312, adj_314, adj_318, adj_319);
                if (!var_312) {
                    wp::adj_sub(var_315, var_317, adj_315, adj_317, adj_318);
                    wp::adj_extract(var_74, var_316, adj_74, adj_316, adj_317);
                }
                if (var_312) {
                    wp::adj_extract(var_74, var_313, adj_74, adj_313, adj_314);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_300, var_309, adj_300, adj_309, adj_310);
                wp::adj_where(var_302, var_304, var_308, adj_302, adj_304, adj_308, adj_309);
                if (!var_302) {
                    wp::adj_sub(var_305, var_307, adj_305, adj_307, adj_308);
                    wp::adj_extract(var_74, var_306, adj_74, adj_306, adj_307);
                }
                if (var_302) {
                    wp::adj_extract(var_74, var_303, adj_74, adj_303, adj_304);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_293, var_295, var_299, adj_293, adj_295, adj_299, adj_300);
                if (!var_293) {
                    wp::adj_sub(var_296, var_298, adj_296, adj_298, adj_299);
                    wp::adj_extract(var_74, var_297, adj_74, adj_297, adj_298);
                }
                if (var_293) {
                    wp::adj_extract(var_74, var_294, adj_74, adj_294, adj_295);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_261, var_290, adj_4, adj_261, adj_290);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_280, var_289, adj_280, adj_289, adj_290);
                wp::adj_where(var_282, var_284, var_288, adj_282, adj_284, adj_288, adj_289);
                if (!var_282) {
                    wp::adj_sub(var_285, var_287, adj_285, adj_287, adj_288);
                    wp::adj_extract(var_74, var_286, adj_74, adj_286, adj_287);
                }
                if (var_282) {
                    wp::adj_extract(var_74, var_283, adj_74, adj_283, adj_284);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_270, var_279, adj_270, adj_279, adj_280);
                wp::adj_where(var_272, var_274, var_278, adj_272, adj_274, adj_278, adj_279);
                if (!var_272) {
                    wp::adj_sub(var_275, var_277, adj_275, adj_277, adj_278);
                    wp::adj_extract(var_74, var_276, adj_74, adj_276, adj_277);
                }
                if (var_272) {
                    wp::adj_extract(var_74, var_273, adj_74, adj_273, adj_274);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_263, var_265, var_269, adj_263, adj_265, adj_269, adj_270);
                if (!var_263) {
                    wp::adj_sub(var_266, var_268, adj_266, adj_268, adj_269);
                    wp::adj_extract(var_74, var_267, adj_74, adj_267, adj_268);
                }
                if (var_263) {
                    wp::adj_extract(var_74, var_264, adj_74, adj_264, adj_265);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_231, var_260, adj_4, adj_231, adj_260);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_250, var_259, adj_250, adj_259, adj_260);
                wp::adj_where(var_252, var_254, var_258, adj_252, adj_254, adj_258, adj_259);
                if (!var_252) {
                    wp::adj_sub(var_255, var_257, adj_255, adj_257, adj_258);
                    wp::adj_extract(var_74, var_256, adj_74, adj_256, adj_257);
                }
                if (var_252) {
                    wp::adj_extract(var_74, var_253, adj_74, adj_253, adj_254);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_240, var_249, adj_240, adj_249, adj_250);
                wp::adj_where(var_242, var_244, var_248, adj_242, adj_244, adj_248, adj_249);
                if (!var_242) {
                    wp::adj_sub(var_245, var_247, adj_245, adj_247, adj_248);
                    wp::adj_extract(var_74, var_246, adj_74, adj_246, adj_247);
                }
                if (var_242) {
                    wp::adj_extract(var_74, var_243, adj_74, adj_243, adj_244);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_233, var_235, var_239, adj_233, adj_235, adj_239, adj_240);
                if (!var_233) {
                    wp::adj_sub(var_236, var_238, adj_236, adj_238, adj_239);
                    wp::adj_extract(var_74, var_237, adj_74, adj_237, adj_238);
                }
                if (var_233) {
                    wp::adj_extract(var_74, var_234, adj_74, adj_234, adj_235);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_201, var_230, adj_4, adj_201, adj_230);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_220, var_229, adj_220, adj_229, adj_230);
                wp::adj_where(var_222, var_224, var_228, adj_222, adj_224, adj_228, adj_229);
                if (!var_222) {
                    wp::adj_sub(var_225, var_227, adj_225, adj_227, adj_228);
                    wp::adj_extract(var_74, var_226, adj_74, adj_226, adj_227);
                }
                if (var_222) {
                    wp::adj_extract(var_74, var_223, adj_74, adj_223, adj_224);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_210, var_219, adj_210, adj_219, adj_220);
                wp::adj_where(var_212, var_214, var_218, adj_212, adj_214, adj_218, adj_219);
                if (!var_212) {
                    wp::adj_sub(var_215, var_217, adj_215, adj_217, adj_218);
                    wp::adj_extract(var_74, var_216, adj_74, adj_216, adj_217);
                }
                if (var_212) {
                    wp::adj_extract(var_74, var_213, adj_74, adj_213, adj_214);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_203, var_205, var_209, adj_203, adj_205, adj_209, adj_210);
                if (!var_203) {
                    wp::adj_sub(var_206, var_208, adj_206, adj_208, adj_209);
                    wp::adj_extract(var_74, var_207, adj_74, adj_207, adj_208);
                }
                if (var_203) {
                    wp::adj_extract(var_74, var_204, adj_74, adj_204, adj_205);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_171, var_200, adj_4, adj_171, adj_200);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_190, var_199, adj_190, adj_199, adj_200);
                wp::adj_where(var_192, var_194, var_198, adj_192, adj_194, adj_198, adj_199);
                if (!var_192) {
                    wp::adj_sub(var_195, var_197, adj_195, adj_197, adj_198);
                    wp::adj_extract(var_74, var_196, adj_74, adj_196, adj_197);
                }
                if (var_192) {
                    wp::adj_extract(var_74, var_193, adj_74, adj_193, adj_194);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_180, var_189, adj_180, adj_189, adj_190);
                wp::adj_where(var_182, var_184, var_188, adj_182, adj_184, adj_188, adj_189);
                if (!var_182) {
                    wp::adj_sub(var_185, var_187, adj_185, adj_187, adj_188);
                    wp::adj_extract(var_74, var_186, adj_74, adj_186, adj_187);
                }
                if (var_182) {
                    wp::adj_extract(var_74, var_183, adj_74, adj_183, adj_184);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_173, var_175, var_179, adj_173, adj_175, adj_179, adj_180);
                if (!var_173) {
                    wp::adj_sub(var_176, var_178, adj_176, adj_178, adj_179);
                    wp::adj_extract(var_74, var_177, adj_74, adj_177, adj_178);
                }
                if (var_173) {
                    wp::adj_extract(var_74, var_174, adj_74, adj_174, adj_175);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                wp::adj_assign_inplace(var_4, var_141, var_170, adj_4, adj_141, adj_170);
                // adj: rx[j] = (                                                             <L 368>
                wp::adj_mul(var_160, var_169, adj_160, adj_169, adj_170);
                wp::adj_where(var_162, var_164, var_168, adj_162, adj_164, adj_168, adj_169);
                if (!var_162) {
                    wp::adj_sub(var_165, var_167, adj_165, adj_167, adj_168);
                    wp::adj_extract(var_74, var_166, adj_74, adj_166, adj_167);
                }
                if (var_162) {
                    wp::adj_extract(var_74, var_163, adj_74, adj_163, adj_164);
                }
                // adj: * (coord[2] if j & 4 else 1.0 - coord[2])                             <L 371>
                wp::adj_mul(var_150, var_159, adj_150, adj_159, adj_160);
                wp::adj_where(var_152, var_154, var_158, adj_152, adj_154, adj_158, adj_159);
                if (!var_152) {
                    wp::adj_sub(var_155, var_157, adj_155, adj_157, adj_158);
                    wp::adj_extract(var_74, var_156, adj_74, adj_156, adj_157);
                }
                if (var_152) {
                    wp::adj_extract(var_74, var_153, adj_74, adj_153, adj_154);
                }
                // adj: * (coord[1] if j & 2 else 1.0 - coord[1])                             <L 370>
                wp::adj_where(var_143, var_145, var_149, adj_143, adj_145, adj_149, adj_150);
                if (!var_143) {
                    wp::adj_sub(var_146, var_148, adj_146, adj_148, adj_149);
                    wp::adj_extract(var_74, var_147, adj_74, adj_147, adj_148);
                }
                if (var_143) {
                    wp::adj_extract(var_74, var_144, adj_74, adj_144, adj_145);
                }
                // adj: (coord[0] if j & 1 else 1.0 - coord[0])                               <L 369>
                // adj: rx[j] = (                                                             <L 368>
                // adj: for j in range(8):                                                    <L 367>
            }
            // adj: if not grad:                                                              <L 366>
        }
        // adj: ) != 0:                                                                       <L 365>
        wp::adj_extract(var_133, var_131, adj_130, adj_131, adj_132);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_130);
        // adj: & int(oct_child[node][7] == -1)                                               <L 364>
        wp::adj_extract(var_125, var_123, adj_122, adj_123, adj_124);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_122);
        // adj: & int(oct_child[node][6] == -1)                                               <L 363>
        wp::adj_extract(var_117, var_115, adj_114, adj_115, adj_116);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_114);
        // adj: & int(oct_child[node][5] == -1)                                               <L 362>
        wp::adj_extract(var_109, var_107, adj_106, adj_107, adj_108);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_106);
        // adj: & int(oct_child[node][4] == -1)                                               <L 361>
        wp::adj_extract(var_101, var_99, adj_98, adj_99, adj_100);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_98);
        // adj: & int(oct_child[node][3] == -1)                                               <L 360>
        wp::adj_extract(var_93, var_91, adj_90, adj_91, adj_92);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_90);
        // adj: & int(oct_child[node][2] == -1)                                               <L 359>
        wp::adj_extract(var_85, var_83, adj_82, adj_83, adj_84);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_82);
        // adj: & int(oct_child[node][1] == -1)                                               <L 358>
        // adj: int(child0 == -1)                                                             <L 357>
        // adj: if (                                                                          <L 356>
        wp::adj_extract(var_78, var_76, adj_75, adj_76, adj_77);
        wp::adj_address(var_oct_child, var_14, adj_oct_child, adj_14, adj_75);
        // adj: child0 = oct_child[node][0]                                                   <L 354>
        wp::adj_cw_div(var_72, var_73, var_74, adj_72, adj_73, adj_74);
        wp::adj_sub(var_31, var_24, adj_31, adj_24, adj_73);
        wp::adj_sub(var_p, var_24, adj_p, adj_24, adj_72);
        // adj: coord = wp.cw_div(p - vmin, vmax - vmin)                                      <L 350>
        wp::adj_where(var_34, var_2, var_13, adj_34, adj_2, adj_13, adj_71);
        if (var_34) {
            wp::adj_assign(var_2, var_13, adj_2, adj_13);
            // adj: continue                                                                  <L 348>
        }
        if (!var_34) {
            wp::adj_extract(var_31, var_68, adj_31, adj_68, adj_69);
            wp::adj_sub(var_66, var_9, adj_66, adj_9, adj_67);
            wp::adj_extract(var_p, var_65, adj_p, adj_65, adj_66);
            // adj: or p[2] - eps > vmax[2]                                                   <L 346>
        }
        if (!var_34) {
            wp::adj_extract(var_24, var_62, adj_24, adj_62, adj_63);
            wp::adj_add(var_60, var_9, adj_60, adj_9, adj_61);
            wp::adj_extract(var_p, var_59, adj_p, adj_59, adj_60);
            // adj: or p[2] + eps < vmin[2]                                                   <L 345>
        }
        if (!var_34) {
            wp::adj_extract(var_31, var_56, adj_31, adj_56, adj_57);
            wp::adj_sub(var_54, var_9, adj_54, adj_9, adj_55);
            wp::adj_extract(var_p, var_53, adj_p, adj_53, adj_54);
            // adj: or p[1] - eps > vmax[1]                                                   <L 344>
        }
        if (!var_34) {
            wp::adj_extract(var_24, var_50, adj_24, adj_50, adj_51);
            wp::adj_add(var_48, var_9, adj_48, adj_9, adj_49);
            wp::adj_extract(var_p, var_47, adj_p, adj_47, adj_48);
            // adj: or p[1] + eps < vmin[1]                                                   <L 343>
        }
        if (!var_34) {
            wp::adj_extract(var_31, var_44, adj_31, adj_44, adj_45);
            wp::adj_sub(var_42, var_9, adj_42, adj_9, adj_43);
            wp::adj_extract(var_p, var_41, adj_p, adj_41, adj_42);
            // adj: or p[0] - eps > vmax[0]                                                   <L 342>
        }
        wp::adj_extract(var_24, var_38, adj_24, adj_38, adj_39);
        wp::adj_add(var_36, var_9, adj_36, adj_9, adj_37);
        wp::adj_extract(var_p, var_35, adj_p, adj_35, adj_36);
        // adj: p[0] + eps < vmin[0]                                                          <L 341>
        // adj: if (                                                                          <L 340>
        wp::adj_add(var_32, var_33, adj_28, adj_30, adj_31);
        wp::adj_address(var_oct_aabb, var_14, var_29, adj_oct_aabb, adj_14, adj_29, adj_30);
        wp::adj_address(var_oct_aabb, var_14, var_27, adj_oct_aabb, adj_14, adj_27, adj_28);
        // adj: vmax = oct_aabb[node, 0] + oct_aabb[node, 1]                                  <L 338>
        wp::adj_sub(var_25, var_26, adj_21, adj_23, adj_24);
        wp::adj_address(var_oct_aabb, var_14, var_22, adj_oct_aabb, adj_14, adj_22, adj_23);
        wp::adj_address(var_oct_aabb, var_14, var_20, adj_oct_aabb, adj_14, adj_20, adj_21);
        // adj: vmin = oct_aabb[node, 0] - oct_aabb[node, 1]                                  <L 337>
        if (var_16) {
            label2:;
            adj_19 += adj_ret_1;
            adj_18 += adj_ret_0;
            // adj: return -1, (rx, ry, rz)                                                   <L 335>
            // adj: wp.printf("ERROR: Invalid node number\n")                                 <L 334>
        }
        // adj: if node == -1:                                                                <L 333>
        wp::adj_copy(var_0, adj_0, adj_14);
        // adj: node = stack                                                                  <L 331>
        wp::adj_sub(var_2, var_12, adj_2, adj_12, adj_13);
        // adj: niter -= 1                                                                    <L 330>
    goto start_while_0;
    end_while_0:;
    // adj: while niter > 0:                                                                  <L 329>
    // adj: eps = 1e-6                                                                        <L 327>
    wp::adj_vec_t(var_7, adj_7, adj_8);
    // adj: rz = vec8(0.0)                                                                    <L 326>
    wp::adj_vec_t(var_5, adj_5, adj_6);
    // adj: ry = vec8(0.0)                                                                    <L 325>
    wp::adj_vec_t(var_3, adj_3, adj_4);
    // adj: rx = vec8(0.0)                                                                    <L 324>
    // adj: niter = int(100)                                                                  <L 323>
    wp::adj_copy(var_root, adj_root, adj_0);
    // adj: stack = root                                                                      <L 322>
    // adj: def find_oct(                                                                     <L 319>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:436
static CUDA_CALLABLE void adj_sample_volume_sdf_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data,
    wp::vec_t<3, wp::float32> & adj_xyz,
    VolumeData_53ac1a2d & adj_volume_data,
    wp::float32 & adj_ret)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32>* var_0;
    wp::vec_t<3, wp::float32>* var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::array_t<wp::vec_t<8, wp::int32>>* var_6;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_7;
    const bool var_8 = false;
    wp::int32* var_9;
    wp::int32 var_10;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_11;
    wp::array_t<wp::vec_t<8, wp::int32>> var_12;
    wp::array_t<wp::vec_t<3, wp::float32>> var_13;
    wp::int32 var_14;
    const wp::int32 var_15 = 0;
    wp::vec_t<8, wp::float32> var_16;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_17;
    wp::vec_t<8, wp::float32>* var_18;
    wp::array_t<wp::vec_t<8, wp::float32>> var_19;
    wp::float32 var_20;
    wp::vec_t<8, wp::float32> var_21;
    wp::float32 var_22;
    //---------
    // dual vars
    wp::vec_t<3, wp::float32> adj_0 = {};
    wp::vec_t<3, wp::float32> adj_1 = {};
    wp::float32 adj_2 = {};
    wp::vec_t<3, wp::float32> adj_3 = {};
    wp::vec_t<3, wp::float32> adj_4 = {};
    wp::vec_t<3, wp::float32> adj_5 = {};
    wp::array_t<wp::vec_t<8, wp::int32>> adj_6 = {};
    wp::array_t<wp::vec_t<3, wp::float32>> adj_7 = {};
    bool adj_8 = {};
    wp::int32 adj_9 = {};
    wp::int32 adj_10 = {};
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> adj_11 = {};
    wp::array_t<wp::vec_t<8, wp::int32>> adj_12 = {};
    wp::array_t<wp::vec_t<3, wp::float32>> adj_13 = {};
    wp::int32 adj_14 = {};
    wp::int32 adj_15 = {};
    wp::vec_t<8, wp::float32> adj_16 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_17 = {};
    wp::vec_t<8, wp::float32> adj_18 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_19 = {};
    wp::float32 adj_20 = {};
    wp::vec_t<8, wp::float32> adj_21 = {};
    wp::float32 adj_22 = {};
    //---------
    // forward
    // def sample_volume_sdf(xyz: wp.vec3, volume_data: VolumeData) -> float:                 <L 437>
    // dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)             <L 438>
    var_0 = &((var_volume_data).center);
    var_1 = &((var_volume_data).half_size);
    var_4 = wp::load(var_0);
    var_5 = wp::load(var_1);
    box_project_0(var_4, var_5, var_xyz, var_2, var_3);
    // node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=False, root=volume_data.root)       <L 439>
    var_6 = &((var_volume_data).oct_child);
    var_7 = &((var_volume_data).oct_aabb);
    var_9 = &((var_volume_data).root);
    var_12 = wp::load(var_6);
    var_13 = wp::load(var_7);
    var_14 = wp::load(var_9);
    find_oct_0(var_12, var_13, var_3, var_8, var_14, var_10, var_11);
    // return dist0 + wp.dot(weights[0], volume_data.oct_coeff[node])                         <L 440>
    var_16 = wp::extract<0>(var_11);
    var_17 = &((var_volume_data).oct_coeff);
    var_19 = wp::load(var_17);
    var_18 = wp::address(var_19, var_10);
    var_21 = wp::load(var_18);
    var_20 = wp::dot(var_16, var_21);
    var_22 = wp::add(var_2, var_20);
    goto label0;
    //---------
    // reverse
    label0:;
    adj_22 += adj_ret;
    wp::adj_add(var_2, var_20, adj_2, adj_20, adj_22);
    wp::adj_dot(var_16, var_21, adj_16, adj_18, adj_20);
    wp::adj_address(var_19, var_10, adj_17, adj_10, adj_18);
    adj_volume_data.oct_coeff = adj_17;
    // adj: return dist0 + wp.dot(weights[0], volume_data.oct_coeff[node])                    <L 440>
    adj_find_oct_0(var_12, var_13, var_3, var_8, var_14, var_10, var_11, adj_6, adj_7, adj_3, adj_8, adj_9, adj_10, adj_11);
    adj_volume_data.root = adj_9;
    adj_volume_data.oct_aabb = adj_7;
    adj_volume_data.oct_child = adj_6;
    // adj: node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=False, root=volume_data.root)  <L 439>
    adj_box_project_0(var_4, var_5, var_xyz, var_2, var_3, adj_0, adj_1, adj_xyz, adj_2, adj_3);
    adj_volume_data.half_size += adj_1;
    adj_volume_data.center += adj_0;
    // adj: dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)        <L 438>
    // adj: def sample_volume_sdf(xyz: wp.vec3, volume_data: VolumeData) -> float:            <L 437>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:443
static CUDA_CALLABLE void adj_sample_volume_grad_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data,
    wp::vec_t<3, wp::float32> & adj_xyz,
    VolumeData_53ac1a2d & adj_volume_data,
    wp::vec_t<3, wp::float32> & adj_ret)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32>* var_0;
    wp::vec_t<3, wp::float32>* var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    const wp::int32 var_6 = 0;
    bool var_7;
    const wp::float32 var_8 = 0.0001;
    const wp::float32 var_9 = 0.0;
    const wp::float32 var_10 = 0.0;
    wp::vec_t<3, wp::float32> var_11;
    const wp::float32 var_12 = 0.0;
    const wp::float32 var_13 = 0.0;
    wp::vec_t<3, wp::float32> var_14;
    const wp::float32 var_15 = 0.0;
    const wp::float32 var_16 = 0.0;
    wp::vec_t<3, wp::float32> var_17;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::array_t<wp::vec_t<8, wp::int32>>* var_32;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_33;
    const bool var_34 = true;
    wp::int32* var_35;
    wp::int32 var_36;
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> var_37;
    wp::array_t<wp::vec_t<8, wp::int32>> var_38;
    wp::array_t<wp::vec_t<3, wp::float32>> var_39;
    wp::int32 var_40;
    const wp::int32 var_41 = 0;
    wp::vec_t<8, wp::float32> var_42;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_43;
    wp::vec_t<8, wp::float32>* var_44;
    wp::array_t<wp::vec_t<8, wp::float32>> var_45;
    wp::float32 var_46;
    wp::vec_t<8, wp::float32> var_47;
    const wp::int32 var_48 = 1;
    wp::vec_t<8, wp::float32> var_49;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_50;
    wp::vec_t<8, wp::float32>* var_51;
    wp::array_t<wp::vec_t<8, wp::float32>> var_52;
    wp::float32 var_53;
    wp::vec_t<8, wp::float32> var_54;
    const wp::int32 var_55 = 2;
    wp::vec_t<8, wp::float32> var_56;
    wp::array_t<wp::vec_t<8, wp::float32>>* var_57;
    wp::vec_t<8, wp::float32>* var_58;
    wp::array_t<wp::vec_t<8, wp::float32>> var_59;
    wp::float32 var_60;
    wp::vec_t<8, wp::float32> var_61;
    wp::vec_t<3, wp::float32> var_62;
    //---------
    // dual vars
    wp::vec_t<3, wp::float32> adj_0 = {};
    wp::vec_t<3, wp::float32> adj_1 = {};
    wp::float32 adj_2 = {};
    wp::vec_t<3, wp::float32> adj_3 = {};
    wp::vec_t<3, wp::float32> adj_4 = {};
    wp::vec_t<3, wp::float32> adj_5 = {};
    wp::int32 adj_6 = {};
    bool adj_7 = {};
    wp::float32 adj_8 = {};
    wp::float32 adj_9 = {};
    wp::float32 adj_10 = {};
    wp::vec_t<3, wp::float32> adj_11 = {};
    wp::float32 adj_12 = {};
    wp::float32 adj_13 = {};
    wp::vec_t<3, wp::float32> adj_14 = {};
    wp::float32 adj_15 = {};
    wp::float32 adj_16 = {};
    wp::vec_t<3, wp::float32> adj_17 = {};
    wp::float32 adj_18 = {};
    wp::vec_t<3, wp::float32> adj_19 = {};
    wp::float32 adj_20 = {};
    wp::float32 adj_21 = {};
    wp::float32 adj_22 = {};
    wp::vec_t<3, wp::float32> adj_23 = {};
    wp::float32 adj_24 = {};
    wp::float32 adj_25 = {};
    wp::float32 adj_26 = {};
    wp::vec_t<3, wp::float32> adj_27 = {};
    wp::float32 adj_28 = {};
    wp::float32 adj_29 = {};
    wp::float32 adj_30 = {};
    wp::vec_t<3, wp::float32> adj_31 = {};
    wp::array_t<wp::vec_t<8, wp::int32>> adj_32 = {};
    wp::array_t<wp::vec_t<3, wp::float32>> adj_33 = {};
    bool adj_34 = {};
    wp::int32 adj_35 = {};
    wp::int32 adj_36 = {};
    wp::tuple_t<wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>, wp::vec_t<8, wp::float32>> adj_37 = {};
    wp::array_t<wp::vec_t<8, wp::int32>> adj_38 = {};
    wp::array_t<wp::vec_t<3, wp::float32>> adj_39 = {};
    wp::int32 adj_40 = {};
    wp::int32 adj_41 = {};
    wp::vec_t<8, wp::float32> adj_42 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_43 = {};
    wp::vec_t<8, wp::float32> adj_44 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_45 = {};
    wp::float32 adj_46 = {};
    wp::vec_t<8, wp::float32> adj_47 = {};
    wp::int32 adj_48 = {};
    wp::vec_t<8, wp::float32> adj_49 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_50 = {};
    wp::vec_t<8, wp::float32> adj_51 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_52 = {};
    wp::float32 adj_53 = {};
    wp::vec_t<8, wp::float32> adj_54 = {};
    wp::int32 adj_55 = {};
    wp::vec_t<8, wp::float32> adj_56 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_57 = {};
    wp::vec_t<8, wp::float32> adj_58 = {};
    wp::array_t<wp::vec_t<8, wp::float32>> adj_59 = {};
    wp::float32 adj_60 = {};
    wp::vec_t<8, wp::float32> adj_61 = {};
    wp::vec_t<3, wp::float32> adj_62 = {};
    //---------
    // forward
    // def sample_volume_grad(xyz: wp.vec3, volume_data: VolumeData) -> wp.vec3:              <L 444>
    // dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)             <L 445>
    var_0 = &((var_volume_data).center);
    var_1 = &((var_volume_data).half_size);
    var_4 = wp::load(var_0);
    var_5 = wp::load(var_1);
    box_project_0(var_4, var_5, var_xyz, var_2, var_3);
    // if dist0 > 0:                                                                          <L 446>
    var_7 = (var_2 > var_6);
    if (var_7) {
        // h = 1e-4                                                                           <L 447>
        // dx = wp.vec3(h, 0.0, 0.0)                                                          <L 448>
        var_11 = wp::vec_t<3, wp::float32>(var_8, var_9, var_10);
        // dy = wp.vec3(0.0, h, 0.0)                                                          <L 449>
        var_14 = wp::vec_t<3, wp::float32>(var_12, var_8, var_13);
        // dz = wp.vec3(0.0, 0.0, h)                                                          <L 450>
        var_17 = wp::vec_t<3, wp::float32>(var_15, var_16, var_8);
        // f = sample_volume_sdf(xyz, volume_data)                                            <L 451>
        var_18 = sample_volume_sdf_0(var_xyz, var_volume_data);
        // grad_x = (sample_volume_sdf(xyz + dx, volume_data) - f) / h                        <L 452>
        var_19 = wp::add(var_xyz, var_11);
        var_20 = sample_volume_sdf_0(var_19, var_volume_data);
        var_21 = wp::sub(var_20, var_18);
        var_22 = wp::div(var_21, var_8);
        // grad_y = (sample_volume_sdf(xyz + dy, volume_data) - f) / h                        <L 453>
        var_23 = wp::add(var_xyz, var_14);
        var_24 = sample_volume_sdf_0(var_23, var_volume_data);
        var_25 = wp::sub(var_24, var_18);
        var_26 = wp::div(var_25, var_8);
        // grad_z = (sample_volume_sdf(xyz + dz, volume_data) - f) / h                        <L 454>
        var_27 = wp::add(var_xyz, var_17);
        var_28 = sample_volume_sdf_0(var_27, var_volume_data);
        var_29 = wp::sub(var_28, var_18);
        var_30 = wp::div(var_29, var_8);
        // return wp.vec3(grad_x, grad_y, grad_z)                                             <L 455>
        var_31 = wp::vec_t<3, wp::float32>(var_22, var_26, var_30);
        goto label0;
    }
    // node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=True, root=volume_data.root)       <L 456>
    var_32 = &((var_volume_data).oct_child);
    var_33 = &((var_volume_data).oct_aabb);
    var_35 = &((var_volume_data).root);
    var_38 = wp::load(var_32);
    var_39 = wp::load(var_33);
    var_40 = wp::load(var_35);
    find_oct_0(var_38, var_39, var_3, var_34, var_40, var_36, var_37);
    // grad_x = wp.dot(weights[0], volume_data.oct_coeff[node])                               <L 457>
    var_42 = wp::extract<0>(var_37);
    var_43 = &((var_volume_data).oct_coeff);
    var_45 = wp::load(var_43);
    var_44 = wp::address(var_45, var_36);
    var_47 = wp::load(var_44);
    var_46 = wp::dot(var_42, var_47);
    // grad_y = wp.dot(weights[1], volume_data.oct_coeff[node])                               <L 458>
    var_49 = wp::extract<1>(var_37);
    var_50 = &((var_volume_data).oct_coeff);
    var_52 = wp::load(var_50);
    var_51 = wp::address(var_52, var_36);
    var_54 = wp::load(var_51);
    var_53 = wp::dot(var_49, var_54);
    // grad_z = wp.dot(weights[2], volume_data.oct_coeff[node])                               <L 459>
    var_56 = wp::extract<2>(var_37);
    var_57 = &((var_volume_data).oct_coeff);
    var_59 = wp::load(var_57);
    var_58 = wp::address(var_59, var_36);
    var_61 = wp::load(var_58);
    var_60 = wp::dot(var_56, var_61);
    // return wp.vec3(grad_x, grad_y, grad_z)                                                 <L 460>
    var_62 = wp::vec_t<3, wp::float32>(var_46, var_53, var_60);
    goto label1;
    //---------
    // reverse
    label1:;
    adj_62 += adj_ret;
    wp::adj_vec_t(var_46, var_53, var_60, adj_46, adj_53, adj_60, adj_62);
    // adj: return wp.vec3(grad_x, grad_y, grad_z)                                            <L 460>
    wp::adj_dot(var_56, var_61, adj_56, adj_58, adj_60);
    wp::adj_address(var_59, var_36, adj_57, adj_36, adj_58);
    adj_volume_data.oct_coeff = adj_57;
    // adj: grad_z = wp.dot(weights[2], volume_data.oct_coeff[node])                          <L 459>
    wp::adj_dot(var_49, var_54, adj_49, adj_51, adj_53);
    wp::adj_address(var_52, var_36, adj_50, adj_36, adj_51);
    adj_volume_data.oct_coeff = adj_50;
    // adj: grad_y = wp.dot(weights[1], volume_data.oct_coeff[node])                          <L 458>
    wp::adj_dot(var_42, var_47, adj_42, adj_44, adj_46);
    wp::adj_address(var_45, var_36, adj_43, adj_36, adj_44);
    adj_volume_data.oct_coeff = adj_43;
    // adj: grad_x = wp.dot(weights[0], volume_data.oct_coeff[node])                          <L 457>
    adj_find_oct_0(var_38, var_39, var_3, var_34, var_40, var_36, var_37, adj_32, adj_33, adj_3, adj_34, adj_35, adj_36, adj_37);
    adj_volume_data.root = adj_35;
    adj_volume_data.oct_aabb = adj_33;
    adj_volume_data.oct_child = adj_32;
    // adj: node, weights = find_oct(volume_data.oct_child, volume_data.oct_aabb, point, grad=True, root=volume_data.root)  <L 456>
    if (var_7) {
        label0:;
        adj_31 += adj_ret;
        wp::adj_vec_t(var_22, var_26, var_30, adj_22, adj_26, adj_30, adj_31);
        // adj: return wp.vec3(grad_x, grad_y, grad_z)                                        <L 455>
        wp::adj_div(var_29, var_8, var_30, adj_29, adj_8, adj_30);
        wp::adj_sub(var_28, var_18, adj_28, adj_18, adj_29);
        adj_sample_volume_sdf_0(var_27, var_volume_data, adj_27, adj_volume_data, adj_28);
        wp::adj_add(var_xyz, var_17, adj_xyz, adj_17, adj_27);
        // adj: grad_z = (sample_volume_sdf(xyz + dz, volume_data) - f) / h                   <L 454>
        wp::adj_div(var_25, var_8, var_26, adj_25, adj_8, adj_26);
        wp::adj_sub(var_24, var_18, adj_24, adj_18, adj_25);
        adj_sample_volume_sdf_0(var_23, var_volume_data, adj_23, adj_volume_data, adj_24);
        wp::adj_add(var_xyz, var_14, adj_xyz, adj_14, adj_23);
        // adj: grad_y = (sample_volume_sdf(xyz + dy, volume_data) - f) / h                   <L 453>
        wp::adj_div(var_21, var_8, var_22, adj_21, adj_8, adj_22);
        wp::adj_sub(var_20, var_18, adj_20, adj_18, adj_21);
        adj_sample_volume_sdf_0(var_19, var_volume_data, adj_19, adj_volume_data, adj_20);
        wp::adj_add(var_xyz, var_11, adj_xyz, adj_11, adj_19);
        // adj: grad_x = (sample_volume_sdf(xyz + dx, volume_data) - f) / h                   <L 452>
        adj_sample_volume_sdf_0(var_xyz, var_volume_data, adj_xyz, adj_volume_data, adj_18);
        // adj: f = sample_volume_sdf(xyz, volume_data)                                       <L 451>
        wp::adj_vec_t(var_15, var_16, var_8, adj_15, adj_16, adj_8, adj_17);
        // adj: dz = wp.vec3(0.0, 0.0, h)                                                     <L 450>
        wp::adj_vec_t(var_12, var_8, var_13, adj_12, adj_8, adj_13, adj_14);
        // adj: dy = wp.vec3(0.0, h, 0.0)                                                     <L 449>
        wp::adj_vec_t(var_8, var_9, var_10, adj_8, adj_9, adj_10, adj_11);
        // adj: dx = wp.vec3(h, 0.0, 0.0)                                                     <L 448>
        // adj: h = 1e-4                                                                      <L 447>
    }
    // adj: if dist0 > 0:                                                                     <L 446>
    adj_box_project_0(var_4, var_5, var_xyz, var_2, var_3, adj_0, adj_1, adj_xyz, adj_2, adj_3);
    adj_volume_data.half_size += adj_1;
    adj_volume_data.center += adj_0;
    // adj: dist0, point = box_project(volume_data.center, volume_data.half_size, xyz)        <L 445>
    // adj: def sample_volume_grad(xyz: wp.vec3, volume_data: VolumeData) -> wp.vec3:         <L 444>
    return;
}



extern "C" __global__ void volume_probe_kernel_b71204a5_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> var_oct_coeff,
    wp::int32 var_root,
    wp::array_t<wp::vec_t<3, wp::float32>> var_points,
    wp::array_t<wp::float32> var_distances,
    wp::array_t<wp::vec_t<3, wp::float32>> var_gradients)
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
        VolumeData_53ac1a2d var_1;
        const wp::int32 var_2 = 0;
        wp::vec_t<3, wp::float32>* var_3;
        wp::vec_t<3, wp::float32> var_4;
        wp::vec_t<3, wp::float32> var_5;
        const wp::int32 var_6 = 1;
        wp::vec_t<3, wp::float32>* var_7;
        wp::vec_t<3, wp::float32> var_8;
        wp::vec_t<3, wp::float32> var_9;
        const bool var_10 = true;
        wp::vec_t<3, wp::float32>* var_11;
        wp::float32 var_12;
        wp::vec_t<3, wp::float32> var_13;
        wp::vec_t<3, wp::float32>* var_14;
        wp::vec_t<3, wp::float32> var_15;
        wp::vec_t<3, wp::float32> var_16;
        //---------
        // forward
        // def volume_probe_kernel(                                                               <L 373>
        // tid = wp.tid()                                                                         <L 382>
        var_0 = builtin_tid1d();
        // volume = collision_sdf.VolumeData()                                                    <L 383>
        var_1 = VolumeData_53ac1a2d();
        // volume.center = oct_aabb[root, 0]                                                      <L 384>
        var_3 = wp::address(var_oct_aabb, var_root, var_2);
        var_5 = wp::load(var_3);
        var_4 = wp::copy(var_5);
        var_1.center = var_4;
        // volume.half_size = oct_aabb[root, 1]                                                   <L 385>
        var_7 = wp::address(var_oct_aabb, var_root, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        var_1.half_size = var_8;
        // volume.root = root                                                                     <L 386>
        var_1.root = var_root;
        // volume.oct_aabb = oct_aabb                                                             <L 387>
        var_1.oct_aabb = var_oct_aabb;
        // volume.oct_child = oct_child                                                           <L 388>
        var_1.oct_child = var_oct_child;
        // volume.oct_coeff = oct_coeff                                                           <L 389>
        var_1.oct_coeff = var_oct_coeff;
        // volume.valid = True                                                                    <L 390>
        var_1.valid = var_10;
        // distances[tid] = collision_sdf.sample_volume_sdf(points[tid], volume)                  <L 391>
        var_11 = wp::address(var_points, var_0);
        var_13 = wp::load(var_11);
        var_12 = sample_volume_sdf_0(var_13, var_1);
        wp::array_store(var_distances, var_0, var_12);
        // gradients[tid] = collision_sdf.sample_volume_grad(points[tid], volume)                 <L 392>
        var_14 = wp::address(var_points, var_0);
        var_16 = wp::load(var_14);
        var_15 = sample_volume_grad_0(var_16, var_1);
        wp::array_store(var_gradients, var_0, var_15);
    }
}



extern "C" __global__ void volume_probe_kernel_b71204a5_cuda_kernel_backward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> var_oct_coeff,
    wp::int32 var_root,
    wp::array_t<wp::vec_t<3, wp::float32>> var_points,
    wp::array_t<wp::float32> var_distances,
    wp::array_t<wp::vec_t<3, wp::float32>> var_gradients,
    wp::array_t<wp::vec_t<8, wp::int32>> adj_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> adj_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> adj_oct_coeff,
    wp::int32 adj_root,
    wp::array_t<wp::vec_t<3, wp::float32>> adj_points,
    wp::array_t<wp::float32> adj_distances,
    wp::array_t<wp::vec_t<3, wp::float32>> adj_gradients)
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
        VolumeData_53ac1a2d var_1;
        const wp::int32 var_2 = 0;
        wp::vec_t<3, wp::float32>* var_3;
        wp::vec_t<3, wp::float32> var_4;
        wp::vec_t<3, wp::float32> var_5;
        const wp::int32 var_6 = 1;
        wp::vec_t<3, wp::float32>* var_7;
        wp::vec_t<3, wp::float32> var_8;
        wp::vec_t<3, wp::float32> var_9;
        const bool var_10 = true;
        wp::vec_t<3, wp::float32>* var_11;
        wp::float32 var_12;
        wp::vec_t<3, wp::float32> var_13;
        wp::vec_t<3, wp::float32>* var_14;
        wp::vec_t<3, wp::float32> var_15;
        wp::vec_t<3, wp::float32> var_16;
        //---------
        // dual vars
        wp::int32 adj_0 = {};
        VolumeData_53ac1a2d adj_1 = {};
        wp::int32 adj_2 = {};
        wp::vec_t<3, wp::float32> adj_3 = {};
        wp::vec_t<3, wp::float32> adj_4 = {};
        wp::vec_t<3, wp::float32> adj_5 = {};
        wp::int32 adj_6 = {};
        wp::vec_t<3, wp::float32> adj_7 = {};
        wp::vec_t<3, wp::float32> adj_8 = {};
        wp::vec_t<3, wp::float32> adj_9 = {};
        bool adj_10 = {};
        wp::vec_t<3, wp::float32> adj_11 = {};
        wp::float32 adj_12 = {};
        wp::vec_t<3, wp::float32> adj_13 = {};
        wp::vec_t<3, wp::float32> adj_14 = {};
        wp::vec_t<3, wp::float32> adj_15 = {};
        wp::vec_t<3, wp::float32> adj_16 = {};
        //---------
        // forward
        // def volume_probe_kernel(                                                               <L 373>
        // tid = wp.tid()                                                                         <L 382>
        var_0 = builtin_tid1d();
        // volume = collision_sdf.VolumeData()                                                    <L 383>
        var_1 = VolumeData_53ac1a2d();
        // volume.center = oct_aabb[root, 0]                                                      <L 384>
        var_3 = wp::address(var_oct_aabb, var_root, var_2);
        var_5 = wp::load(var_3);
        var_4 = wp::copy(var_5);
        var_1.center = var_4;
        // volume.half_size = oct_aabb[root, 1]                                                   <L 385>
        var_7 = wp::address(var_oct_aabb, var_root, var_6);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        var_1.half_size = var_8;
        // volume.root = root                                                                     <L 386>
        var_1.root = var_root;
        // volume.oct_aabb = oct_aabb                                                             <L 387>
        var_1.oct_aabb = var_oct_aabb;
        // volume.oct_child = oct_child                                                           <L 388>
        var_1.oct_child = var_oct_child;
        // volume.oct_coeff = oct_coeff                                                           <L 389>
        var_1.oct_coeff = var_oct_coeff;
        // volume.valid = True                                                                    <L 390>
        var_1.valid = var_10;
        // distances[tid] = collision_sdf.sample_volume_sdf(points[tid], volume)                  <L 391>
        var_11 = wp::address(var_points, var_0);
        var_13 = wp::load(var_11);
        var_12 = sample_volume_sdf_0(var_13, var_1);
        // wp::array_store(var_distances, var_0, var_12);
        // gradients[tid] = collision_sdf.sample_volume_grad(points[tid], volume)                 <L 392>
        var_14 = wp::address(var_points, var_0);
        var_16 = wp::load(var_14);
        var_15 = sample_volume_grad_0(var_16, var_1);
        // wp::array_store(var_gradients, var_0, var_15);
        //---------
        // reverse
        wp::adj_array_store(var_gradients, var_0, var_15, adj_gradients, adj_0, adj_15);
        adj_sample_volume_grad_0(var_16, var_1, adj_14, adj_1, adj_15);
        wp::adj_address(var_points, var_0, adj_points, adj_0, adj_14);
        // adj: gradients[tid] = collision_sdf.sample_volume_grad(points[tid], volume)            <L 392>
        wp::adj_array_store(var_distances, var_0, var_12, adj_distances, adj_0, adj_12);
        adj_sample_volume_sdf_0(var_13, var_1, adj_11, adj_1, adj_12);
        wp::adj_address(var_points, var_0, adj_points, adj_0, adj_11);
        // adj: distances[tid] = collision_sdf.sample_volume_sdf(points[tid], volume)             <L 391>
        adj_1.valid = {};
        // adj: volume.valid = True                                                               <L 390>
        adj_1.oct_coeff = {};
        // adj: volume.oct_coeff = oct_coeff                                                      <L 389>
        adj_1.oct_child = {};
        // adj: volume.oct_child = oct_child                                                      <L 388>
        adj_1.oct_aabb = {};
        // adj: volume.oct_aabb = oct_aabb                                                        <L 387>
        adj_1.root = {};
        // adj: volume.root = root                                                                <L 386>
        adj_8 += adj_1.half_size;
        adj_1.half_size = {};
        wp::adj_copy(var_9, adj_7, adj_8);
        wp::adj_address(var_oct_aabb, var_root, var_6, adj_oct_aabb, adj_root, adj_6, adj_7);
        // adj: volume.half_size = oct_aabb[root, 1]                                              <L 385>
        adj_4 += adj_1.center;
        adj_1.center = {};
        wp::adj_copy(var_5, adj_3, adj_4);
        wp::adj_address(var_oct_aabb, var_root, var_2, adj_oct_aabb, adj_root, adj_2, adj_3);
        // adj: volume.center = oct_aabb[root, 0]                                                 <L 384>
        // adj: volume = collision_sdf.VolumeData()                                               <L 383>
        // adj: tid = wp.tid()                                                                    <L 382>
        // adj: def volume_probe_kernel(                                                          <L 373>
        continue;
    }
}

