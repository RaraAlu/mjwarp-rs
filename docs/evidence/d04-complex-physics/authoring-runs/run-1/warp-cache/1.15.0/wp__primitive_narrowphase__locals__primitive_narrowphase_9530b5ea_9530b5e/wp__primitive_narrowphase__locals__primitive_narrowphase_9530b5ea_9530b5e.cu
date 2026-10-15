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


struct Geom_4022bfa7
{
    wp::vec_t<3, wp::float32> pos;
    wp::mat_t<3, 3, wp::float32> rot;
    wp::vec_t<3, wp::float32> normal;
    wp::vec_t<3, wp::float32> size;
    wp::float32 margin;
    wp::mat_t<6, 3, wp::float32> polyvert;
    wp::int32 vertadr;
    wp::int32 vertnum;
    wp::array_t<wp::vec_t<3, wp::float32>> vert;
    wp::int32 graphadr;
    wp::array_t<wp::int32> graph;
    wp::int32 mesh_polynum;
    wp::int32 mesh_polyadr;
    wp::array_t<wp::vec_t<3, wp::float32>> mesh_polynormal;
    wp::array_t<wp::int32> mesh_polyvertadr;
    wp::array_t<wp::int32> mesh_polyvertnum;
    wp::array_t<wp::int32> mesh_polyvert;
    wp::array_t<wp::int32> mesh_polymapadr;
    wp::array_t<wp::int32> mesh_polymapnum;
    wp::array_t<wp::int32> mesh_polymap;
    wp::int32 index;


    Geom_4022bfa7() = default;
    CUDA_CALLABLE Geom_4022bfa7(wp::vec_t<3, wp::float32> const& pos,
    wp::mat_t<3, 3, wp::float32> const& rot = {},
    wp::vec_t<3, wp::float32> const& normal = {},
    wp::vec_t<3, wp::float32> const& size = {},
    wp::float32 const& margin = {},
    wp::mat_t<6, 3, wp::float32> const& polyvert = {},
    wp::int32 const& vertadr = {},
    wp::int32 const& vertnum = {},
    wp::array_t<wp::vec_t<3, wp::float32>> const& vert = {},
    wp::int32 const& graphadr = {},
    wp::array_t<wp::int32> const& graph = {},
    wp::int32 const& mesh_polynum = {},
    wp::int32 const& mesh_polyadr = {},
    wp::array_t<wp::vec_t<3, wp::float32>> const& mesh_polynormal = {},
    wp::array_t<wp::int32> const& mesh_polyvertadr = {},
    wp::array_t<wp::int32> const& mesh_polyvertnum = {},
    wp::array_t<wp::int32> const& mesh_polyvert = {},
    wp::array_t<wp::int32> const& mesh_polymapadr = {},
    wp::array_t<wp::int32> const& mesh_polymapnum = {},
    wp::array_t<wp::int32> const& mesh_polymap = {},
    wp::int32 const& index = {})
        : pos{pos}
        , rot{rot}
        , normal{normal}
        , size{size}
        , margin{margin}
        , polyvert{polyvert}
        , vertadr{vertadr}
        , vertnum{vertnum}
        , vert{vert}
        , graphadr{graphadr}
        , graph{graph}
        , mesh_polynum{mesh_polynum}
        , mesh_polyadr{mesh_polyadr}
        , mesh_polynormal{mesh_polynormal}
        , mesh_polyvertadr{mesh_polyvertadr}
        , mesh_polyvertnum{mesh_polyvertnum}
        , mesh_polyvert{mesh_polyvert}
        , mesh_polymapadr{mesh_polymapadr}
        , mesh_polymapnum{mesh_polymapnum}
        , mesh_polymap{mesh_polymap}
        , index{index}

    {
    }

    CUDA_CALLABLE Geom_4022bfa7& operator += (const Geom_4022bfa7& rhs)
    {    pos += rhs.pos;
    rot += rhs.rot;
    normal += rhs.normal;
    size += rhs.size;
    margin += rhs.margin;
    polyvert += rhs.polyvert;
    vertadr += rhs.vertadr;
    vertnum += rhs.vertnum;
    graphadr += rhs.graphadr;
    mesh_polynum += rhs.mesh_polynum;
    mesh_polyadr += rhs.mesh_polyadr;
    index += rhs.index;

        return *this;}

};

static CUDA_CALLABLE void adj_Geom_4022bfa7(wp::vec_t<3, wp::float32> const&,
    wp::mat_t<3, 3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::float32 const&,
    wp::mat_t<6, 3, wp::float32> const&,
    wp::int32 const&,
    wp::int32 const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::int32 const&,
    wp::array_t<wp::int32> const&,
    wp::int32 const&,
    wp::int32 const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::int32> const&,
    wp::int32 const&,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_rot,
    wp::vec_t<3, wp::float32> & adj_normal,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_margin,
    wp::mat_t<6, 3, wp::float32> & adj_polyvert,
    wp::int32 & adj_vertadr,
    wp::int32 & adj_vertnum,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_vert,
    wp::int32 & adj_graphadr,
    wp::array_t<wp::int32> & adj_graph,
    wp::int32 & adj_mesh_polynum,
    wp::int32 & adj_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_polynormal,
    wp::array_t<wp::int32> & adj_mesh_polyvertadr,
    wp::array_t<wp::int32> & adj_mesh_polyvertnum,
    wp::array_t<wp::int32> & adj_mesh_polyvert,
    wp::array_t<wp::int32> & adj_mesh_polymapadr,
    wp::array_t<wp::int32> & adj_mesh_polymapnum,
    wp::array_t<wp::int32> & adj_mesh_polymap,
    wp::int32 & adj_index,
    Geom_4022bfa7 & adj_ret)
{
    adj_pos += adj_ret.pos;
    adj_rot += adj_ret.rot;
    adj_normal += adj_ret.normal;
    adj_size += adj_ret.size;
    adj_margin += adj_ret.margin;
    adj_polyvert += adj_ret.polyvert;
    adj_vertadr += adj_ret.vertadr;
    adj_vertnum += adj_ret.vertnum;
    adj_vert = adj_ret.vert;
    adj_graphadr += adj_ret.graphadr;
    adj_graph = adj_ret.graph;
    adj_mesh_polynum += adj_ret.mesh_polynum;
    adj_mesh_polyadr += adj_ret.mesh_polyadr;
    adj_mesh_polynormal = adj_ret.mesh_polynormal;
    adj_mesh_polyvertadr = adj_ret.mesh_polyvertadr;
    adj_mesh_polyvertnum = adj_ret.mesh_polyvertnum;
    adj_mesh_polyvert = adj_ret.mesh_polyvert;
    adj_mesh_polymapadr = adj_ret.mesh_polymapadr;
    adj_mesh_polymapnum = adj_ret.mesh_polymapnum;
    adj_mesh_polymap = adj_ret.mesh_polymap;
    adj_index += adj_ret.index;
}

// Required when compiling adjoints.
CUDA_CALLABLE Geom_4022bfa7 add(const Geom_4022bfa7& a, const Geom_4022bfa7& b)
{
    return Geom_4022bfa7();
}

CUDA_CALLABLE void adj_atomic_add(Geom_4022bfa7* p, Geom_4022bfa7 t)
{
    wp::adj_atomic_add(&p->pos, t.pos);
    wp::adj_atomic_add(&p->rot, t.rot);
    wp::adj_atomic_add(&p->normal, t.normal);
    wp::adj_atomic_add(&p->size, t.size);
    wp::adj_atomic_add(&p->margin, t.margin);
    wp::adj_atomic_add(&p->polyvert, t.polyvert);
    wp::adj_atomic_add(&p->vertadr, t.vertadr);
    wp::adj_atomic_add(&p->vertnum, t.vertnum);
    wp::adj_atomic_add(&p->vert, t.vert);
    wp::adj_atomic_add(&p->graphadr, t.graphadr);
    wp::adj_atomic_add(&p->graph, t.graph);
    wp::adj_atomic_add(&p->mesh_polynum, t.mesh_polynum);
    wp::adj_atomic_add(&p->mesh_polyadr, t.mesh_polyadr);
    wp::adj_atomic_add(&p->mesh_polynormal, t.mesh_polynormal);
    wp::adj_atomic_add(&p->mesh_polyvertadr, t.mesh_polyvertadr);
    wp::adj_atomic_add(&p->mesh_polyvertnum, t.mesh_polyvertnum);
    wp::adj_atomic_add(&p->mesh_polyvert, t.mesh_polyvert);
    wp::adj_atomic_add(&p->mesh_polymapadr, t.mesh_polymapadr);
    wp::adj_atomic_add(&p->mesh_polymapnum, t.mesh_polymapnum);
    wp::adj_atomic_add(&p->mesh_polymap, t.mesh_polymap);
    wp::adj_atomic_add(&p->index, t.index);
}




// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:294
static CUDA_CALLABLE void contact_margin_gap_0(
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_pair_margin,
    wp::array_t<wp::float32> var_pair_gap,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_pairid,
    wp::int32 var_worldid,
    wp::float32 & ret_0,
    wp::float32 & ret_1)
{
    //---------
    // primal vars
    const wp::int32 var_0 = -1;
    bool var_1;
    wp::shape_t* var_2;
    const wp::int32 var_3 = 0;
    wp::int32 var_4;
    wp::shape_t var_5;
    wp::int32 var_6;
    wp::float32* var_7;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::shape_t* var_10;
    const wp::int32 var_11 = 0;
    wp::int32 var_12;
    wp::shape_t var_13;
    wp::int32 var_14;
    wp::float32* var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::int32 var_18 = 0;
    wp::int32 var_19;
    const wp::int32 var_20 = 1;
    wp::int32 var_21;
    wp::shape_t* var_22;
    const wp::int32 var_23 = 0;
    wp::int32 var_24;
    wp::shape_t var_25;
    wp::int32 var_26;
    wp::shape_t* var_27;
    const wp::int32 var_28 = 0;
    wp::int32 var_29;
    wp::shape_t var_30;
    wp::int32 var_31;
    wp::float32* var_32;
    wp::float32* var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::float32* var_37;
    wp::float32* var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    //---------
    // forward
    // def contact_margin_gap(                                                                <L 295>
    // if pairid > -1:                                                                        <L 306>
    var_1 = (var_pairid > var_0);
    if (var_1) {
        // margin = pair_margin[worldid % pair_margin.shape[0], pairid]                       <L 307>
        var_2 = &(var_pair_margin.shape);
        var_5 = wp::load(var_2);
        var_4 = wp::extract(var_5, var_3);
        var_6 = wp::mod(var_worldid, var_4);
        var_7 = wp::address(var_pair_margin, var_6, var_pairid);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // gap = pair_gap[worldid % pair_gap.shape[0], pairid]                                <L 308>
        var_10 = &(var_pair_gap.shape);
        var_13 = wp::load(var_10);
        var_12 = wp::extract(var_13, var_11);
        var_14 = wp::mod(var_worldid, var_12);
        var_15 = wp::address(var_pair_gap, var_14, var_pairid);
        var_17 = wp::load(var_15);
        var_16 = wp::copy(var_17);
    }
    if (!var_1) {
        // g1 = geoms[0]                                                                      <L 310>
        var_19 = wp::extract(var_geoms, var_18);
        // g2 = geoms[1]                                                                      <L 311>
        var_21 = wp::extract(var_geoms, var_20);
        // margin_id = worldid % geom_margin.shape[0]                                         <L 312>
        var_22 = &(var_geom_margin.shape);
        var_25 = wp::load(var_22);
        var_24 = wp::extract(var_25, var_23);
        var_26 = wp::mod(var_worldid, var_24);
        // gap_id = worldid % geom_gap.shape[0]                                               <L 313>
        var_27 = &(var_geom_gap.shape);
        var_30 = wp::load(var_27);
        var_29 = wp::extract(var_30, var_28);
        var_31 = wp::mod(var_worldid, var_29);
        // margin = geom_margin[margin_id, g1] + geom_margin[margin_id, g2]                   <L 314>
        var_32 = wp::address(var_geom_margin, var_26, var_19);
        var_33 = wp::address(var_geom_margin, var_26, var_21);
        var_35 = wp::load(var_32);
        var_36 = wp::load(var_33);
        var_34 = wp::add(var_35, var_36);
        // gap = geom_gap[gap_id, g1] + geom_gap[gap_id, g2]                                  <L 315>
        var_37 = wp::address(var_geom_gap, var_31, var_19);
        var_38 = wp::address(var_geom_gap, var_31, var_21);
        var_40 = wp::load(var_37);
        var_41 = wp::load(var_38);
        var_39 = wp::add(var_40, var_41);
    }
    var_42 = wp::where(var_1, var_8, var_34);
    var_43 = wp::where(var_1, var_16, var_39);
    // return margin, gap                                                                     <L 317>
    ret_0 = var_42;
    ret_1 = var_43;
    return;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:320
static CUDA_CALLABLE void contact_material_params_0(
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_solimp,
    wp::array_t<wp::float32> var_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_friction,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_pairid,
    wp::int32 var_worldid,
    wp::int32 & ret_0,
    wp::vec_t<5, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<2, wp::float32> & ret_3,
    wp::vec_t<5, wp::float32> & ret_4,
    wp::float32 & ret_5)
{
    //---------
    // primal vars
    const wp::int32 var_0 = -1;
    bool var_1;
    wp::int32* var_2;
    wp::int32 var_3;
    wp::int32 var_4;
    wp::shape_t* var_5;
    const wp::int32 var_6 = 0;
    wp::int32 var_7;
    wp::shape_t var_8;
    wp::int32 var_9;
    wp::vec_t<5, wp::float32>* var_10;
    wp::vec_t<5, wp::float32> var_11;
    wp::vec_t<5, wp::float32> var_12;
    wp::shape_t* var_13;
    const wp::int32 var_14 = 0;
    wp::int32 var_15;
    wp::shape_t var_16;
    wp::int32 var_17;
    wp::vec_t<2, wp::float32>* var_18;
    wp::vec_t<2, wp::float32> var_19;
    wp::vec_t<2, wp::float32> var_20;
    wp::shape_t* var_21;
    const wp::int32 var_22 = 0;
    wp::int32 var_23;
    wp::shape_t var_24;
    wp::int32 var_25;
    wp::vec_t<2, wp::float32>* var_26;
    wp::vec_t<2, wp::float32> var_27;
    wp::vec_t<2, wp::float32> var_28;
    wp::shape_t* var_29;
    const wp::int32 var_30 = 0;
    wp::int32 var_31;
    wp::shape_t var_32;
    wp::int32 var_33;
    wp::vec_t<5, wp::float32>* var_34;
    wp::vec_t<5, wp::float32> var_35;
    wp::vec_t<5, wp::float32> var_36;
    wp::shape_t* var_37;
    const wp::int32 var_38 = 0;
    wp::int32 var_39;
    wp::shape_t var_40;
    wp::int32 var_41;
    wp::float32* var_42;
    wp::float32 var_43;
    wp::float32 var_44;
    const wp::int32 var_45 = 0;
    wp::int32 var_46;
    const wp::int32 var_47 = 1;
    wp::int32 var_48;
    wp::shape_t* var_49;
    const wp::int32 var_50 = 0;
    wp::int32 var_51;
    wp::shape_t var_52;
    wp::int32 var_53;
    wp::shape_t* var_54;
    const wp::int32 var_55 = 0;
    wp::int32 var_56;
    wp::shape_t var_57;
    wp::int32 var_58;
    wp::shape_t* var_59;
    const wp::int32 var_60 = 0;
    wp::int32 var_61;
    wp::shape_t var_62;
    wp::int32 var_63;
    wp::shape_t* var_64;
    const wp::int32 var_65 = 0;
    wp::int32 var_66;
    wp::shape_t var_67;
    wp::int32 var_68;
    wp::shape_t* var_69;
    const wp::int32 var_70 = 0;
    wp::int32 var_71;
    wp::shape_t var_72;
    wp::int32 var_73;
    wp::float32* var_74;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::float32* var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::int32* var_80;
    wp::int32 var_81;
    wp::int32 var_82;
    wp::int32* var_83;
    wp::int32 var_84;
    wp::int32 var_85;
    wp::int32* var_86;
    wp::int32 var_87;
    wp::int32 var_88;
    wp::int32* var_89;
    wp::int32 var_90;
    wp::int32 var_91;
    wp::float32* var_92;
    wp::float32 var_93;
    wp::float32 var_94;
    wp::float32* var_95;
    wp::float32 var_96;
    wp::float32 var_97;
    bool var_98;
    const wp::float32 var_99 = 1.0;
    wp::int32 var_100;
    wp::vec_t<3, wp::float32>* var_101;
    wp::vec_t<3, wp::float32> var_102;
    wp::vec_t<3, wp::float32> var_103;
    wp::float32 var_104;
    wp::int32 var_105;
    wp::float32 var_106;
    bool var_107;
    const wp::float32 var_108 = 0.0;
    wp::int32 var_109;
    wp::vec_t<3, wp::float32>* var_110;
    wp::vec_t<3, wp::float32> var_111;
    wp::vec_t<3, wp::float32> var_112;
    wp::float32 var_113;
    wp::int32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::vec_t<3, wp::float32> var_117;
    wp::float32 var_118;
    wp::float32 var_119;
    bool var_120;
    const wp::float32 var_121 = 1e-15;
    bool var_122;
    bool var_123;
    const wp::float32 var_124 = 0.5;
    wp::float32 var_125;
    bool var_126;
    bool var_127;
    bool var_128;
    const wp::float32 var_129 = 0.0;
    wp::float32 var_130;
    bool var_131;
    bool var_132;
    bool var_133;
    const wp::float32 var_134 = 1.0;
    wp::float32 var_135;
    wp::int32 var_136;
    wp::vec_t<3, wp::float32>* var_137;
    wp::vec_t<3, wp::float32>* var_138;
    wp::vec_t<3, wp::float32> var_139;
    wp::vec_t<3, wp::float32> var_140;
    wp::vec_t<3, wp::float32> var_141;
    wp::float32 var_142;
    wp::int32 var_143;
    wp::float32 var_144;
    wp::float32 var_145;
    wp::vec_t<3, wp::float32> var_146;
    wp::int32 var_147;
    wp::float32 var_148;
    wp::float32 var_149;
    wp::vec_t<3, wp::float32> var_150;
    const wp::int32 var_151 = 0;
    wp::float32 var_152;
    const wp::int32 var_153 = 0;
    wp::float32 var_154;
    const wp::int32 var_155 = 1;
    wp::float32 var_156;
    const wp::int32 var_157 = 2;
    wp::float32 var_158;
    const wp::int32 var_159 = 2;
    wp::float32 var_160;
    wp::vec_t<5, wp::float32> var_161;
    bool var_162;
    wp::vec_t<2, wp::float32>* var_163;
    const wp::int32 var_164 = 0;
    wp::float32 var_165;
    wp::vec_t<2, wp::float32> var_166;
    const wp::float32 var_167 = 0.0;
    bool var_168;
    wp::vec_t<2, wp::float32>* var_169;
    const wp::int32 var_170 = 0;
    wp::float32 var_171;
    wp::vec_t<2, wp::float32> var_172;
    const wp::float32 var_173 = 0.0;
    bool var_174;
    wp::vec_t<2, wp::float32>* var_175;
    wp::vec_t<2, wp::float32> var_176;
    wp::vec_t<2, wp::float32> var_177;
    const wp::float32 var_178 = 1.0;
    wp::float32 var_179;
    wp::vec_t<2, wp::float32>* var_180;
    wp::vec_t<2, wp::float32> var_181;
    wp::vec_t<2, wp::float32> var_182;
    wp::vec_t<2, wp::float32> var_183;
    wp::vec_t<2, wp::float32> var_184;
    wp::vec_t<2, wp::float32>* var_185;
    wp::vec_t<2, wp::float32>* var_186;
    wp::vec_t<2, wp::float32> var_187;
    wp::vec_t<2, wp::float32> var_188;
    wp::vec_t<2, wp::float32> var_189;
    wp::vec_t<2, wp::float32> var_190;
    const wp::float32 var_191 = 0.0;
    const wp::float32 var_192 = 0.0;
    wp::vec_t<2, wp::float32> var_193;
    wp::vec_t<5, wp::float32>* var_194;
    wp::vec_t<5, wp::float32> var_195;
    wp::vec_t<5, wp::float32> var_196;
    const wp::float32 var_197 = 1.0;
    wp::float32 var_198;
    wp::vec_t<5, wp::float32>* var_199;
    wp::vec_t<5, wp::float32> var_200;
    wp::vec_t<5, wp::float32> var_201;
    wp::vec_t<5, wp::float32> var_202;
    wp::int32 var_203;
    wp::vec_t<5, wp::float32> var_204;
    wp::vec_t<2, wp::float32> var_205;
    wp::vec_t<2, wp::float32> var_206;
    wp::vec_t<5, wp::float32> var_207;
    wp::float32 var_208;
    const wp::float32 var_209 = 1e-05;
    const wp::int32 var_210 = 0;
    wp::float32 var_211;
    wp::float32 var_212;
    const wp::int32 var_213 = 1;
    wp::float32 var_214;
    wp::float32 var_215;
    const wp::int32 var_216 = 2;
    wp::float32 var_217;
    wp::float32 var_218;
    const wp::int32 var_219 = 3;
    wp::float32 var_220;
    wp::float32 var_221;
    const wp::int32 var_222 = 4;
    wp::float32 var_223;
    wp::float32 var_224;
    wp::vec_t<5, wp::float32> var_225;
    //---------
    // forward
    // def contact_material_params(                                                           <L 321>
    // if pairid > -1:                                                                        <L 341>
    var_1 = (var_pairid > var_0);
    if (var_1) {
        // condim = pair_dim[pairid]                                                          <L 342>
        var_2 = wp::address(var_pair_dim, var_pairid);
        var_4 = wp::load(var_2);
        var_3 = wp::copy(var_4);
        // friction = pair_friction[worldid % pair_friction.shape[0], pairid]                 <L 343>
        var_5 = &(var_pair_friction.shape);
        var_8 = wp::load(var_5);
        var_7 = wp::extract(var_8, var_6);
        var_9 = wp::mod(var_worldid, var_7);
        var_10 = wp::address(var_pair_friction, var_9, var_pairid);
        var_12 = wp::load(var_10);
        var_11 = wp::copy(var_12);
        // solref = pair_solref[worldid % pair_solref.shape[0], pairid]                       <L 344>
        var_13 = &(var_pair_solref.shape);
        var_16 = wp::load(var_13);
        var_15 = wp::extract(var_16, var_14);
        var_17 = wp::mod(var_worldid, var_15);
        var_18 = wp::address(var_pair_solref, var_17, var_pairid);
        var_20 = wp::load(var_18);
        var_19 = wp::copy(var_20);
        // solreffriction = pair_solreffriction[worldid % pair_solreffriction.shape[0], pairid]       <L 345>
        var_21 = &(var_pair_solreffriction.shape);
        var_24 = wp::load(var_21);
        var_23 = wp::extract(var_24, var_22);
        var_25 = wp::mod(var_worldid, var_23);
        var_26 = wp::address(var_pair_solreffriction, var_25, var_pairid);
        var_28 = wp::load(var_26);
        var_27 = wp::copy(var_28);
        // solimp = pair_solimp[worldid % pair_solimp.shape[0], pairid]                       <L 346>
        var_29 = &(var_pair_solimp.shape);
        var_32 = wp::load(var_29);
        var_31 = wp::extract(var_32, var_30);
        var_33 = wp::mod(var_worldid, var_31);
        var_34 = wp::address(var_pair_solimp, var_33, var_pairid);
        var_36 = wp::load(var_34);
        var_35 = wp::copy(var_36);
        // adhesion = pair_adhesion[worldid % pair_adhesion.shape[0], pairid]                 <L 347>
        var_37 = &(var_pair_adhesion.shape);
        var_40 = wp::load(var_37);
        var_39 = wp::extract(var_40, var_38);
        var_41 = wp::mod(var_worldid, var_39);
        var_42 = wp::address(var_pair_adhesion, var_41, var_pairid);
        var_44 = wp::load(var_42);
        var_43 = wp::copy(var_44);
    }
    if (!var_1) {
        // g1 = geoms[0]                                                                      <L 349>
        var_46 = wp::extract(var_geoms, var_45);
        // g2 = geoms[1]                                                                      <L 350>
        var_48 = wp::extract(var_geoms, var_47);
        // solmix_id = worldid % geom_solmix.shape[0]                                         <L 351>
        var_49 = &(var_geom_solmix.shape);
        var_52 = wp::load(var_49);
        var_51 = wp::extract(var_52, var_50);
        var_53 = wp::mod(var_worldid, var_51);
        // friction_id = worldid % geom_friction.shape[0]                                     <L 352>
        var_54 = &(var_geom_friction.shape);
        var_57 = wp::load(var_54);
        var_56 = wp::extract(var_57, var_55);
        var_58 = wp::mod(var_worldid, var_56);
        // solref_id = worldid % geom_solref.shape[0]                                         <L 353>
        var_59 = &(var_geom_solref.shape);
        var_62 = wp::load(var_59);
        var_61 = wp::extract(var_62, var_60);
        var_63 = wp::mod(var_worldid, var_61);
        // solimp_id = worldid % geom_solimp.shape[0]                                         <L 354>
        var_64 = &(var_geom_solimp.shape);
        var_67 = wp::load(var_64);
        var_66 = wp::extract(var_67, var_65);
        var_68 = wp::mod(var_worldid, var_66);
        // adhesion_id = worldid % geom_adhesion.shape[0]                                     <L 355>
        var_69 = &(var_geom_adhesion.shape);
        var_72 = wp::load(var_69);
        var_71 = wp::extract(var_72, var_70);
        var_73 = wp::mod(var_worldid, var_71);
        // solmix1 = geom_solmix[solmix_id, g1]                                               <L 357>
        var_74 = wp::address(var_geom_solmix, var_53, var_46);
        var_76 = wp::load(var_74);
        var_75 = wp::copy(var_76);
        // solmix2 = geom_solmix[solmix_id, g2]                                               <L 358>
        var_77 = wp::address(var_geom_solmix, var_53, var_48);
        var_79 = wp::load(var_77);
        var_78 = wp::copy(var_79);
        // condim1 = geom_condim[g1]                                                          <L 360>
        var_80 = wp::address(var_geom_condim, var_46);
        var_82 = wp::load(var_80);
        var_81 = wp::copy(var_82);
        // condim2 = geom_condim[g2]                                                          <L 361>
        var_83 = wp::address(var_geom_condim, var_48);
        var_85 = wp::load(var_83);
        var_84 = wp::copy(var_85);
        // p1 = geom_priority[g1]                                                             <L 364>
        var_86 = wp::address(var_geom_priority, var_46);
        var_88 = wp::load(var_86);
        var_87 = wp::copy(var_88);
        // p2 = geom_priority[g2]                                                             <L 365>
        var_89 = wp::address(var_geom_priority, var_48);
        var_91 = wp::load(var_89);
        var_90 = wp::copy(var_91);
        // a1 = geom_adhesion[adhesion_id, g1]                                                <L 366>
        var_92 = wp::address(var_geom_adhesion, var_73, var_46);
        var_94 = wp::load(var_92);
        var_93 = wp::copy(var_94);
        // a2 = geom_adhesion[adhesion_id, g2]                                                <L 367>
        var_95 = wp::address(var_geom_adhesion, var_73, var_48);
        var_97 = wp::load(var_95);
        var_96 = wp::copy(var_97);
        // if p1 > p2:                                                                        <L 369>
        var_98 = (var_87 > var_90);
        if (var_98) {
            // mix = 1.0                                                                      <L 370>
            // condim = condim1                                                               <L 371>
            var_100 = wp::copy(var_81);
            // max_geom_friction = geom_friction[friction_id, g1]                             <L 372>
            var_101 = wp::address(var_geom_friction, var_58, var_46);
            var_103 = wp::load(var_101);
            var_102 = wp::copy(var_103);
            // adhesion = a1                                                                  <L 373>
            var_104 = wp::copy(var_93);
        }
        var_105 = wp::where(var_98, var_100, var_3);
        var_106 = wp::where(var_98, var_104, var_43);
        if (!var_98) {
            // elif p2 > p1:                                                                  <L 374>
            var_107 = (var_90 > var_87);
            if (var_107) {
                // mix = 0.0                                                                  <L 375>
                // condim = condim2                                                           <L 376>
                var_109 = wp::copy(var_84);
                // max_geom_friction = geom_friction[friction_id, g2]                         <L 377>
                var_110 = wp::address(var_geom_friction, var_58, var_48);
                var_112 = wp::load(var_110);
                var_111 = wp::copy(var_112);
                // adhesion = a2                                                              <L 378>
                var_113 = wp::copy(var_96);
            }
            var_114 = wp::where(var_107, var_109, var_105);
            var_115 = wp::where(var_107, var_113, var_106);
            var_116 = wp::where(var_107, var_108, var_99);
            var_117 = wp::where(var_107, var_111, var_102);
            if (!var_107) {
                // mix = safe_div(solmix1, solmix1 + solmix2)                                 <L 380>
                var_118 = wp::add(var_75, var_78);
                var_119 = safe_div_0(var_75, var_118);
                // mix = wp.where((solmix1 < MJ_MINVAL) and (solmix2 < MJ_MINVAL), 0.5, mix)       <L 381>
                var_122 = (var_75 < var_121);
                var_120 = var_122;
                if (var_120) {
                    var_123 = (var_78 < var_121);
                    var_120 = var_120 && var_123;
                }
                var_125 = wp::where(var_120, var_124, var_119);
                // mix = wp.where((solmix1 < MJ_MINVAL) and (solmix2 >= MJ_MINVAL), 0.0, mix)       <L 382>
                var_127 = (var_75 < var_121);
                var_126 = var_127;
                if (var_126) {
                    var_128 = (var_78 >= var_121);
                    var_126 = var_126 && var_128;
                }
                var_130 = wp::where(var_126, var_129, var_125);
                // mix = wp.where((solmix1 >= MJ_MINVAL) and (solmix2 < MJ_MINVAL), 1.0, mix)       <L 383>
                var_132 = (var_75 >= var_121);
                var_131 = var_132;
                if (var_131) {
                    var_133 = (var_78 < var_121);
                    var_131 = var_131 && var_133;
                }
                var_135 = wp::where(var_131, var_134, var_130);
                // condim = wp.max(condim1, condim2)                                          <L 384>
                var_136 = wp::max(var_81, var_84);
                // max_geom_friction = wp.max(geom_friction[friction_id, g1], geom_friction[friction_id, g2])       <L 385>
                var_137 = wp::address(var_geom_friction, var_58, var_46);
                var_138 = wp::address(var_geom_friction, var_58, var_48);
                var_140 = wp::load(var_137);
                var_141 = wp::load(var_138);
                var_139 = wp::max(var_140, var_141);
                // adhesion = a1 + a2                                                         <L 386>
                var_142 = wp::add(var_93, var_96);
            }
            var_143 = wp::where(var_107, var_114, var_136);
            var_144 = wp::where(var_107, var_115, var_142);
            var_145 = wp::where(var_107, var_116, var_135);
            var_146 = wp::where(var_107, var_117, var_139);
        }
        var_147 = wp::where(var_98, var_105, var_143);
        var_148 = wp::where(var_98, var_106, var_144);
        var_149 = wp::where(var_98, var_99, var_145);
        var_150 = wp::where(var_98, var_102, var_146);
        // friction = vec5(                                                                   <L 388>
        // max_geom_friction[0],                                                              <L 389>
        var_152 = wp::extract(var_150, var_151);
        // max_geom_friction[0],                                                              <L 390>
        var_154 = wp::extract(var_150, var_153);
        // max_geom_friction[1],                                                              <L 391>
        var_156 = wp::extract(var_150, var_155);
        // max_geom_friction[2],                                                              <L 392>
        var_158 = wp::extract(var_150, var_157);
        // max_geom_friction[2],                                                              <L 393>
        var_160 = wp::extract(var_150, var_159);
        var_161 = wp::vec_t<5, wp::float32>({var_152, var_154, var_156, var_158, var_160});
        // if geom_solref[solref_id, g1][0] > 0.0 and geom_solref[solref_id, g2][0] > 0.0:       <L 396>
        var_163 = wp::address(var_geom_solref, var_63, var_46);
        var_166 = wp::load(var_163);
        var_165 = wp::extract(var_166, var_164);
        var_168 = (var_165 > var_167);
        var_162 = var_168;
        if (var_162) {
            var_169 = wp::address(var_geom_solref, var_63, var_48);
            var_172 = wp::load(var_169);
            var_171 = wp::extract(var_172, var_170);
            var_174 = (var_171 > var_173);
            var_162 = var_162 && var_174;
        }
        if (var_162) {
            // solref = mix * geom_solref[solref_id, g1] + (1.0 - mix) * geom_solref[solref_id, g2]       <L 397>
            var_175 = wp::address(var_geom_solref, var_63, var_46);
            var_177 = wp::load(var_175);
            var_176 = wp::mul(var_149, var_177);
            var_179 = wp::sub(var_178, var_149);
            var_180 = wp::address(var_geom_solref, var_63, var_48);
            var_182 = wp::load(var_180);
            var_181 = wp::mul(var_179, var_182);
            var_183 = wp::add(var_176, var_181);
        }
        var_184 = wp::where(var_162, var_183, var_19);
        if (!var_162) {
            // solref = wp.min(geom_solref[solref_id, g1], geom_solref[solref_id, g2])        <L 399>
            var_185 = wp::address(var_geom_solref, var_63, var_46);
            var_186 = wp::address(var_geom_solref, var_63, var_48);
            var_188 = wp::load(var_185);
            var_189 = wp::load(var_186);
            var_187 = wp::min(var_188, var_189);
        }
        var_190 = wp::where(var_162, var_184, var_187);
        // solreffriction = wp.vec2(0.0, 0.0)                                                 <L 401>
        var_193 = wp::vec_t<2, wp::float32>(var_191, var_192);
        // solimp = mix * geom_solimp[solimp_id, g1] + (1.0 - mix) * geom_solimp[solimp_id, g2]       <L 402>
        var_194 = wp::address(var_geom_solimp, var_68, var_46);
        var_196 = wp::load(var_194);
        var_195 = wp::mul(var_149, var_196);
        var_198 = wp::sub(var_197, var_149);
        var_199 = wp::address(var_geom_solimp, var_68, var_48);
        var_201 = wp::load(var_199);
        var_200 = wp::mul(var_198, var_201);
        var_202 = wp::add(var_195, var_200);
    }
    var_203 = wp::where(var_1, var_3, var_147);
    var_204 = wp::where(var_1, var_11, var_161);
    var_205 = wp::where(var_1, var_19, var_190);
    var_206 = wp::where(var_1, var_27, var_193);
    var_207 = wp::where(var_1, var_35, var_202);
    var_208 = wp::where(var_1, var_43, var_148);
    // friction = vec5(                                                                       <L 404>
    // wp.max(MJ_MINMU, friction[0]),                                                         <L 405>
    var_211 = wp::extract(var_204, var_210);
    var_212 = wp::max(var_209, var_211);
    // wp.max(MJ_MINMU, friction[1]),                                                         <L 406>
    var_214 = wp::extract(var_204, var_213);
    var_215 = wp::max(var_209, var_214);
    // wp.max(MJ_MINMU, friction[2]),                                                         <L 407>
    var_217 = wp::extract(var_204, var_216);
    var_218 = wp::max(var_209, var_217);
    // wp.max(MJ_MINMU, friction[3]),                                                         <L 408>
    var_220 = wp::extract(var_204, var_219);
    var_221 = wp::max(var_209, var_220);
    // wp.max(MJ_MINMU, friction[4]),                                                         <L 409>
    var_223 = wp::extract(var_204, var_222);
    var_224 = wp::max(var_209, var_223);
    var_225 = wp::vec_t<5, wp::float32>({var_212, var_215, var_218, var_221, var_224});
    // return condim, friction, solref, solreffriction, solimp, adhesion                      <L 412>
    ret_0 = var_203;
    ret_1 = var_225;
    ret_2 = var_205;
    ret_3 = var_206;
    ret_4 = var_207;
    ret_5 = var_208;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:416
static CUDA_CALLABLE void contact_params_0(
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_solimp,
    wp::array_t<wp::float32> var_pair_margin,
    wp::array_t<wp::float32> var_pair_gap,
    wp::array_t<wp::float32> var_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_friction,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_in,
    wp::int32 var_cid,
    wp::int32 var_worldid,
    wp::vec_t<2, wp::int32> & ret_0,
    wp::float32 & ret_1,
    wp::float32 & ret_2,
    wp::int32 & ret_3,
    wp::vec_t<5, wp::float32> & ret_4,
    wp::vec_t<2, wp::float32> & ret_5,
    wp::vec_t<2, wp::float32> & ret_6,
    wp::vec_t<5, wp::float32> & ret_7,
    wp::float32 & ret_8)
{
    //---------
    // primal vars
    wp::vec_t<2, wp::int32>* var_0;
    wp::vec_t<2, wp::int32> var_1;
    wp::vec_t<2, wp::int32> var_2;
    wp::vec_t<2, wp::int32>* var_3;
    const wp::int32 var_4 = 0;
    wp::int32 var_5;
    wp::vec_t<2, wp::int32> var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::int32 var_9;
    wp::vec_t<5, wp::float32> var_10;
    wp::vec_t<2, wp::float32> var_11;
    wp::vec_t<2, wp::float32> var_12;
    wp::vec_t<5, wp::float32> var_13;
    wp::float32 var_14;
    //---------
    // forward
    // def contact_params(                                                                    <L 417>
    // geoms = collision_pair_in[cid]                                                         <L 447>
    var_0 = wp::address(var_collision_pair_in, var_cid);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // pairid = collision_pairid_in[cid][0]                                                   <L 448>
    var_3 = wp::address(var_collision_pairid_in, var_cid);
    var_6 = wp::load(var_3);
    var_5 = wp::extract(var_6, var_4);
    // margin, gap = contact_margin_gap(geom_margin, geom_gap, pair_margin, pair_gap, geoms, pairid, worldid)       <L 450>
    contact_margin_gap_0(var_geom_margin, var_geom_gap, var_pair_margin, var_pair_gap, var_1, var_5, var_worldid, var_7, var_8);
    // condim, friction, solref, solreffriction, solimp, adhesion = contact_material_params(       <L 451>
    // geom_condim,                                                                           <L 452>
    // geom_priority,                                                                         <L 453>
    // geom_solmix,                                                                           <L 454>
    // geom_solref,                                                                           <L 455>
    // geom_solimp,                                                                           <L 456>
    // geom_friction,                                                                         <L 457>
    // geom_adhesion,                                                                         <L 458>
    // pair_dim,                                                                              <L 459>
    // pair_solref,                                                                           <L 460>
    // pair_solreffriction,                                                                   <L 461>
    // pair_solimp,                                                                           <L 462>
    // pair_adhesion,                                                                         <L 463>
    // pair_friction,                                                                         <L 464>
    // geoms,                                                                                 <L 465>
    // pairid,                                                                                <L 466>
    // worldid,                                                                               <L 467>
    contact_material_params_0(var_geom_condim, var_geom_priority, var_geom_solmix, var_geom_solref, var_geom_solimp, var_geom_friction, var_geom_adhesion, var_pair_dim, var_pair_solref, var_pair_solreffriction, var_pair_solimp, var_pair_adhesion, var_pair_friction, var_1, var_5, var_worldid, var_9, var_10, var_11, var_12, var_13, var_14);
    // return geoms, margin, gap, condim, friction, solref, solreffriction, solimp, adhesion       <L 470>
    ret_0 = var_1;
    ret_1 = var_7;
    ret_2 = var_8;
    ret_3 = var_9;
    ret_4 = var_10;
    ret_5 = var_11;
    ret_6 = var_12;
    ret_7 = var_13;
    ret_8 = var_14;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:65
static CUDA_CALLABLE void geom_collision_pair_from_types_0(
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::int32> var_mesh_polynum,
    wp::array_t<wp::int32> var_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_polynormal,
    wp::array_t<wp::int32> var_mesh_polyvertadr,
    wp::array_t<wp::int32> var_mesh_polyvertnum,
    wp::array_t<wp::int32> var_mesh_polyvert,
    wp::array_t<wp::int32> var_mesh_polymapadr,
    wp::array_t<wp::int32> var_mesh_polymapnum,
    wp::array_t<wp::int32> var_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_geom_type1,
    wp::int32 var_geom_type2,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_worldid,
    Geom_4022bfa7 & ret_0,
    Geom_4022bfa7 & ret_1)
{
    //---------
    // primal vars
    Geom_4022bfa7 var_0;
    Geom_4022bfa7 var_1;
    const wp::int32 var_2 = 0;
    wp::int32 var_3;
    const wp::int32 var_4 = 1;
    wp::int32 var_5;
    wp::vec_t<3, wp::float32>* var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::mat_t<3, 3, wp::float32>* var_9;
    wp::mat_t<3, 3, wp::float32> var_10;
    wp::mat_t<3, 3, wp::float32> var_11;
    wp::shape_t* var_12;
    const wp::int32 var_13 = 0;
    wp::int32 var_14;
    wp::shape_t var_15;
    wp::int32 var_16;
    wp::vec_t<3, wp::float32>* var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::mat_t<3, 3, wp::float32>* var_20;
    const wp::int32 var_21 = 0;
    const wp::int32 var_22 = 2;
    wp::float32 var_23;
    wp::mat_t<3, 3, wp::float32> var_24;
    wp::mat_t<3, 3, wp::float32>* var_25;
    const wp::int32 var_26 = 1;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    wp::mat_t<3, 3, wp::float32> var_29;
    wp::mat_t<3, 3, wp::float32>* var_30;
    const wp::int32 var_31 = 2;
    const wp::int32 var_32 = 2;
    wp::float32 var_33;
    wp::mat_t<3, 3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32>* var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::mat_t<3, 3, wp::float32>* var_39;
    wp::mat_t<3, 3, wp::float32> var_40;
    wp::mat_t<3, 3, wp::float32> var_41;
    wp::shape_t* var_42;
    const wp::int32 var_43 = 0;
    wp::int32 var_44;
    wp::shape_t var_45;
    wp::int32 var_46;
    wp::vec_t<3, wp::float32>* var_47;
    wp::vec_t<3, wp::float32> var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::mat_t<3, 3, wp::float32>* var_50;
    const wp::int32 var_51 = 0;
    const wp::int32 var_52 = 2;
    wp::float32 var_53;
    wp::mat_t<3, 3, wp::float32> var_54;
    wp::mat_t<3, 3, wp::float32>* var_55;
    const wp::int32 var_56 = 1;
    const wp::int32 var_57 = 2;
    wp::float32 var_58;
    wp::mat_t<3, 3, wp::float32> var_59;
    wp::mat_t<3, 3, wp::float32>* var_60;
    const wp::int32 var_61 = 2;
    const wp::int32 var_62 = 2;
    wp::float32 var_63;
    wp::mat_t<3, 3, wp::float32> var_64;
    wp::vec_t<3, wp::float32> var_65;
    wp::shape_t* var_66;
    const wp::int32 var_67 = 0;
    wp::int32 var_68;
    wp::shape_t var_69;
    wp::int32 var_70;
    const wp::int32 var_71 = 7;
    bool var_72;
    wp::int32* var_73;
    wp::int32 var_74;
    wp::int32 var_75;
    const wp::int32 var_76 = 0;
    bool var_77;
    wp::int32* var_78;
    const wp::int32 var_79 = -1;
    wp::int32 var_80;
    wp::int32 var_81;
    const wp::int32 var_82 = 0;
    bool var_83;
    wp::int32* var_84;
    const wp::int32 var_85 = -1;
    wp::int32 var_86;
    wp::int32 var_87;
    const wp::int32 var_88 = 0;
    bool var_89;
    wp::int32* var_90;
    const wp::int32 var_91 = -1;
    wp::int32 var_92;
    wp::int32 var_93;
    const wp::int32 var_94 = 0;
    bool var_95;
    wp::int32* var_96;
    const wp::int32 var_97 = -1;
    wp::int32 var_98;
    wp::int32 var_99;
    const wp::int32 var_100 = 0;
    bool var_101;
    wp::int32* var_102;
    const wp::int32 var_103 = -1;
    wp::int32 var_104;
    wp::int32 var_105;
    const wp::int32 var_106 = 7;
    bool var_107;
    wp::int32* var_108;
    wp::int32 var_109;
    wp::int32 var_110;
    const wp::int32 var_111 = 0;
    bool var_112;
    wp::int32* var_113;
    const wp::int32 var_114 = -1;
    wp::int32 var_115;
    wp::int32 var_116;
    const wp::int32 var_117 = 0;
    bool var_118;
    wp::int32* var_119;
    const wp::int32 var_120 = -1;
    wp::int32 var_121;
    wp::int32 var_122;
    const wp::int32 var_123 = 0;
    bool var_124;
    wp::int32* var_125;
    const wp::int32 var_126 = -1;
    wp::int32 var_127;
    wp::int32 var_128;
    const wp::int32 var_129 = 0;
    bool var_130;
    wp::int32* var_131;
    const wp::int32 var_132 = -1;
    wp::int32 var_133;
    wp::int32 var_134;
    const wp::int32 var_135 = 0;
    bool var_136;
    wp::int32* var_137;
    const wp::int32 var_138 = -1;
    wp::int32 var_139;
    wp::int32 var_140;
    wp::int32 var_141;
    const wp::int32 var_142 = -1;
    const wp::float32 var_143 = 0.0;
    const wp::int32 var_144 = -1;
    const wp::float32 var_145 = 0.0;
    //---------
    // forward
    // def geom_collision_pair_from_types(                                                    <L 66>
    // geom1 = Geom()                                                                         <L 93>
    var_0 = Geom_4022bfa7();
    // geom2 = Geom()                                                                         <L 94>
    var_1 = Geom_4022bfa7();
    // g1 = geoms[0]                                                                          <L 96>
    var_3 = wp::extract(var_geoms, var_2);
    // g2 = geoms[1]                                                                          <L 97>
    var_5 = wp::extract(var_geoms, var_4);
    // geom1.pos = geom_xpos_in[worldid, g1]                                                  <L 99>
    var_6 = wp::address(var_geom_xpos_in, var_worldid, var_3);
    var_8 = wp::load(var_6);
    var_7 = wp::copy(var_8);
    var_0.pos = var_7;
    // geom1.rot = geom_xmat_in[worldid, g1]                                                  <L 100>
    var_9 = wp::address(var_geom_xmat_in, var_worldid, var_3);
    var_11 = wp::load(var_9);
    var_10 = wp::copy(var_11);
    var_0.rot = var_10;
    // geom1.size = geom_size[worldid % geom_size.shape[0], g1]                               <L 101>
    var_12 = &(var_geom_size.shape);
    var_15 = wp::load(var_12);
    var_14 = wp::extract(var_15, var_13);
    var_16 = wp::mod(var_worldid, var_14);
    var_17 = wp::address(var_geom_size, var_16, var_3);
    var_19 = wp::load(var_17);
    var_18 = wp::copy(var_19);
    var_0.size = var_18;
    // geom1.normal = wp.vec3(geom1.rot[0, 2], geom1.rot[1, 2], geom1.rot[2, 2])              <L 103>
    var_20 = &((var_0).rot);
    var_24 = wp::load(var_20);
    var_23 = wp::extract(var_24, var_21, var_22);
    var_25 = &((var_0).rot);
    var_29 = wp::load(var_25);
    var_28 = wp::extract(var_29, var_26, var_27);
    var_30 = &((var_0).rot);
    var_34 = wp::load(var_30);
    var_33 = wp::extract(var_34, var_31, var_32);
    var_35 = wp::vec_t<3, wp::float32>(var_23, var_28, var_33);
    var_0.normal = var_35;
    // geom2.pos = geom_xpos_in[worldid, g2]                                                  <L 105>
    var_36 = wp::address(var_geom_xpos_in, var_worldid, var_5);
    var_38 = wp::load(var_36);
    var_37 = wp::copy(var_38);
    var_1.pos = var_37;
    // geom2.rot = geom_xmat_in[worldid, g2]                                                  <L 106>
    var_39 = wp::address(var_geom_xmat_in, var_worldid, var_5);
    var_41 = wp::load(var_39);
    var_40 = wp::copy(var_41);
    var_1.rot = var_40;
    // geom2.size = geom_size[worldid % geom_size.shape[0], g2]                               <L 107>
    var_42 = &(var_geom_size.shape);
    var_45 = wp::load(var_42);
    var_44 = wp::extract(var_45, var_43);
    var_46 = wp::mod(var_worldid, var_44);
    var_47 = wp::address(var_geom_size, var_46, var_5);
    var_49 = wp::load(var_47);
    var_48 = wp::copy(var_49);
    var_1.size = var_48;
    // geom2.normal = wp.vec3(geom2.rot[0, 2], geom2.rot[1, 2], geom2.rot[2, 2])              <L 109>
    var_50 = &((var_1).rot);
    var_54 = wp::load(var_50);
    var_53 = wp::extract(var_54, var_51, var_52);
    var_55 = &((var_1).rot);
    var_59 = wp::load(var_55);
    var_58 = wp::extract(var_59, var_56, var_57);
    var_60 = &((var_1).rot);
    var_64 = wp::load(var_60);
    var_63 = wp::extract(var_64, var_61, var_62);
    var_65 = wp::vec_t<3, wp::float32>(var_53, var_58, var_63);
    var_1.normal = var_65;
    // dataid_setid = worldid % geom_dataid.shape[0]                                          <L 111>
    var_66 = &(var_geom_dataid.shape);
    var_69 = wp::load(var_66);
    var_68 = wp::extract(var_69, var_67);
    var_70 = wp::mod(var_worldid, var_68);
    // if geom_type1 == GeomType.MESH:                                                        <L 113>
    var_72 = (var_geom_type1 == var_71);
    if (var_72) {
        // dataid = geom_dataid[dataid_setid, g1]                                             <L 114>
        var_73 = wp::address(var_geom_dataid, var_70, var_3);
        var_75 = wp::load(var_73);
        var_74 = wp::copy(var_75);
        // geom1.vertadr = wp.where(dataid >= 0, mesh_vertadr[dataid], -1)                    <L 115>
        var_77 = (var_74 >= var_76);
        var_78 = wp::address(var_mesh_vertadr, var_74);
        var_81 = wp::load(var_78);
        var_80 = wp::where(var_77, var_81, var_79);
        var_0.vertadr = var_80;
        // geom1.vertnum = wp.where(dataid >= 0, mesh_vertnum[dataid], -1)                    <L 116>
        var_83 = (var_74 >= var_82);
        var_84 = wp::address(var_mesh_vertnum, var_74);
        var_87 = wp::load(var_84);
        var_86 = wp::where(var_83, var_87, var_85);
        var_0.vertnum = var_86;
        // geom1.graphadr = wp.where(dataid >= 0, mesh_graphadr[dataid], -1)                  <L 117>
        var_89 = (var_74 >= var_88);
        var_90 = wp::address(var_mesh_graphadr, var_74);
        var_93 = wp::load(var_90);
        var_92 = wp::where(var_89, var_93, var_91);
        var_0.graphadr = var_92;
        // geom1.mesh_polynum = wp.where(dataid >= 0, mesh_polynum[dataid], -1)               <L 118>
        var_95 = (var_74 >= var_94);
        var_96 = wp::address(var_mesh_polynum, var_74);
        var_99 = wp::load(var_96);
        var_98 = wp::where(var_95, var_99, var_97);
        var_0.mesh_polynum = var_98;
        // geom1.mesh_polyadr = wp.where(dataid >= 0, mesh_polyadr[dataid], -1)               <L 119>
        var_101 = (var_74 >= var_100);
        var_102 = wp::address(var_mesh_polyadr, var_74);
        var_105 = wp::load(var_102);
        var_104 = wp::where(var_101, var_105, var_103);
        var_0.mesh_polyadr = var_104;
        // geom1.vert = mesh_vert                                                             <L 121>
        var_0.vert = var_mesh_vert;
        // geom1.graph = mesh_graph                                                           <L 122>
        var_0.graph = var_mesh_graph;
        // geom1.mesh_polynormal = mesh_polynormal                                            <L 123>
        var_0.mesh_polynormal = var_mesh_polynormal;
        // geom1.mesh_polyvertadr = mesh_polyvertadr                                          <L 124>
        var_0.mesh_polyvertadr = var_mesh_polyvertadr;
        // geom1.mesh_polyvertnum = mesh_polyvertnum                                          <L 125>
        var_0.mesh_polyvertnum = var_mesh_polyvertnum;
        // geom1.mesh_polyvert = mesh_polyvert                                                <L 126>
        var_0.mesh_polyvert = var_mesh_polyvert;
        // geom1.mesh_polymapadr = mesh_polymapadr                                            <L 127>
        var_0.mesh_polymapadr = var_mesh_polymapadr;
        // geom1.mesh_polymapnum = mesh_polymapnum                                            <L 128>
        var_0.mesh_polymapnum = var_mesh_polymapnum;
        // geom1.mesh_polymap = mesh_polymap                                                  <L 129>
        var_0.mesh_polymap = var_mesh_polymap;
    }
    // if geom_type2 == GeomType.MESH:                                                        <L 131>
    var_107 = (var_geom_type2 == var_106);
    if (var_107) {
        // dataid = geom_dataid[dataid_setid, g2]                                             <L 132>
        var_108 = wp::address(var_geom_dataid, var_70, var_5);
        var_110 = wp::load(var_108);
        var_109 = wp::copy(var_110);
        // geom2.vertadr = wp.where(dataid >= 0, mesh_vertadr[dataid], -1)                    <L 133>
        var_112 = (var_109 >= var_111);
        var_113 = wp::address(var_mesh_vertadr, var_109);
        var_116 = wp::load(var_113);
        var_115 = wp::where(var_112, var_116, var_114);
        var_1.vertadr = var_115;
        // geom2.vertnum = wp.where(dataid >= 0, mesh_vertnum[dataid], -1)                    <L 134>
        var_118 = (var_109 >= var_117);
        var_119 = wp::address(var_mesh_vertnum, var_109);
        var_122 = wp::load(var_119);
        var_121 = wp::where(var_118, var_122, var_120);
        var_1.vertnum = var_121;
        // geom2.graphadr = wp.where(dataid >= 0, mesh_graphadr[dataid], -1)                  <L 135>
        var_124 = (var_109 >= var_123);
        var_125 = wp::address(var_mesh_graphadr, var_109);
        var_128 = wp::load(var_125);
        var_127 = wp::where(var_124, var_128, var_126);
        var_1.graphadr = var_127;
        // geom2.mesh_polynum = wp.where(dataid >= 0, mesh_polynum[dataid], -1)               <L 136>
        var_130 = (var_109 >= var_129);
        var_131 = wp::address(var_mesh_polynum, var_109);
        var_134 = wp::load(var_131);
        var_133 = wp::where(var_130, var_134, var_132);
        var_1.mesh_polynum = var_133;
        // geom2.mesh_polyadr = wp.where(dataid >= 0, mesh_polyadr[dataid], -1)               <L 137>
        var_136 = (var_109 >= var_135);
        var_137 = wp::address(var_mesh_polyadr, var_109);
        var_140 = wp::load(var_137);
        var_139 = wp::where(var_136, var_140, var_138);
        var_1.mesh_polyadr = var_139;
        // geom2.vert = mesh_vert                                                             <L 139>
        var_1.vert = var_mesh_vert;
        // geom2.graph = mesh_graph                                                           <L 140>
        var_1.graph = var_mesh_graph;
        // geom2.mesh_polynormal = mesh_polynormal                                            <L 141>
        var_1.mesh_polynormal = var_mesh_polynormal;
        // geom2.mesh_polyvertadr = mesh_polyvertadr                                          <L 142>
        var_1.mesh_polyvertadr = var_mesh_polyvertadr;
        // geom2.mesh_polyvertnum = mesh_polyvertnum                                          <L 143>
        var_1.mesh_polyvertnum = var_mesh_polyvertnum;
        // geom2.mesh_polyvert = mesh_polyvert                                                <L 144>
        var_1.mesh_polyvert = var_mesh_polyvert;
        // geom2.mesh_polymapadr = mesh_polymapadr                                            <L 145>
        var_1.mesh_polymapadr = var_mesh_polymapadr;
        // geom2.mesh_polymapnum = mesh_polymapnum                                            <L 146>
        var_1.mesh_polymapnum = var_mesh_polymapnum;
        // geom2.mesh_polymap = mesh_polymap                                                  <L 147>
        var_1.mesh_polymap = var_mesh_polymap;
    }
    var_141 = wp::where(var_107, var_109, var_74);
    // geom1.index = -1                                                                       <L 149>
    var_0.index = var_142;
    // geom1.margin = 0.0                                                                     <L 150>
    var_0.margin = var_143;
    // geom2.index = -1                                                                       <L 152>
    var_1.index = var_144;
    // geom2.margin = 0.0                                                                     <L 153>
    var_1.margin = var_145;
    // return geom1, geom2                                                                    <L 155>
    ret_0 = var_0;
    ret_1 = var_1;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:158
static CUDA_CALLABLE void geom_collision_pair_0(
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::int32> var_mesh_polynum,
    wp::array_t<wp::int32> var_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_polynormal,
    wp::array_t<wp::int32> var_mesh_polyvertadr,
    wp::array_t<wp::int32> var_mesh_polyvertnum,
    wp::array_t<wp::int32> var_mesh_polyvert,
    wp::array_t<wp::int32> var_mesh_polymapadr,
    wp::array_t<wp::int32> var_mesh_polymapnum,
    wp::array_t<wp::int32> var_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_worldid,
    Geom_4022bfa7 & ret_0,
    Geom_4022bfa7 & ret_1)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::int32 var_1;
    wp::int32* var_2;
    wp::int32 var_3;
    wp::int32 var_4;
    const wp::int32 var_5 = 1;
    wp::int32 var_6;
    wp::int32* var_7;
    wp::int32 var_8;
    wp::int32 var_9;
    Geom_4022bfa7 var_10;
    Geom_4022bfa7 var_11;
    //---------
    // forward
    // def geom_collision_pair(                                                               <L 159>
    // geom_type1 = geom_type[geoms[0]]                                                       <L 185>
    var_1 = wp::extract(var_geoms, var_0);
    var_2 = wp::address(var_geom_type, var_1);
    var_4 = wp::load(var_2);
    var_3 = wp::copy(var_4);
    // geom_type2 = geom_type[geoms[1]]                                                       <L 186>
    var_6 = wp::extract(var_geoms, var_5);
    var_7 = wp::address(var_geom_type, var_6);
    var_9 = wp::load(var_7);
    var_8 = wp::copy(var_9);
    // return geom_collision_pair_from_types(                                                 <L 187>
    // geom_dataid,                                                                           <L 188>
    // geom_size,                                                                             <L 189>
    // mesh_vertadr,                                                                          <L 190>
    // mesh_vertnum,                                                                          <L 191>
    // mesh_graphadr,                                                                         <L 192>
    // mesh_vert,                                                                             <L 193>
    // mesh_graph,                                                                            <L 194>
    // mesh_polynum,                                                                          <L 195>
    // mesh_polyadr,                                                                          <L 196>
    // mesh_polynormal,                                                                       <L 197>
    // mesh_polyvertadr,                                                                      <L 198>
    // mesh_polyvertnum,                                                                      <L 199>
    // mesh_polyvert,                                                                         <L 200>
    // mesh_polymapadr,                                                                       <L 201>
    // mesh_polymapnum,                                                                       <L 202>
    // mesh_polymap,                                                                          <L 203>
    // geom_xpos_in,                                                                          <L 204>
    // geom_xmat_in,                                                                          <L 205>
    // geom_type1,                                                                            <L 206>
    // geom_type2,                                                                            <L 207>
    // geoms,                                                                                 <L 208>
    // worldid,                                                                               <L 209>
    geom_collision_pair_from_types_0(var_geom_dataid, var_geom_size, var_mesh_vertadr, var_mesh_vertnum, var_mesh_graphadr, var_mesh_vert, var_mesh_graph, var_mesh_polynum, var_mesh_polyadr, var_mesh_polynormal, var_mesh_polyvertadr, var_mesh_polyvertnum, var_mesh_polyvert, var_mesh_polymapadr, var_mesh_polymapnum, var_mesh_polymap, var_geom_xpos_in, var_geom_xmat_in, var_3, var_8, var_geoms, var_worldid, var_10, var_11);
    ret_0 = var_10;
    ret_1 = var_11;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:50
static CUDA_CALLABLE void plane_sphere_0(
    wp::vec_t<3, wp::float32> var_plane_normal,
    wp::vec_t<3, wp::float32> var_plane_pos,
    wp::vec_t<3, wp::float32> var_sphere_pos,
    wp::float32 var_sphere_radius,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::float32 var_3 = 0.5;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    //---------
    // forward
    // def plane_sphere(plane_normal: wp.vec3, plane_pos: wp.vec3, sphere_pos: wp.vec3, sphere_radius: float) -> Tuple[float, wp.vec3]:       <L 51>
    // dist = wp.dot(sphere_pos - plane_pos, plane_normal) - sphere_radius                    <L 53>
    var_0 = wp::sub(var_sphere_pos, var_plane_pos);
    var_1 = wp::dot(var_0, var_plane_normal);
    var_2 = wp::sub(var_1, var_sphere_radius);
    // pos = sphere_pos - plane_normal * (sphere_radius + 0.5 * dist)                         <L 54>
    var_4 = wp::mul(var_3, var_2);
    var_5 = wp::add(var_sphere_radius, var_4);
    var_6 = wp::mul(var_plane_normal, var_5);
    var_7 = wp::sub(var_sphere_pos, var_6);
    // return dist, pos                                                                       <L 55>
    ret_0 = var_2;
    ret_1 = var_7;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:213
static CUDA_CALLABLE wp::int32 write_contact_0(
    wp::int32 var_naconmax_in,
    wp::int32 var_id_,
    wp::float32 var_dist_in,
    wp::vec_t<3, wp::float32> var_pos_in,
    wp::mat_t<3, 3, wp::float32> var_frame_in,
    wp::float32 var_margin_in,
    wp::float32 var_gap_in,
    wp::int32 var_condim_in,
    wp::vec_t<5, wp::float32> var_friction_in,
    wp::vec_t<2, wp::float32> var_solref_in,
    wp::vec_t<2, wp::float32> var_solreffriction_in,
    wp::vec_t<5, wp::float32> var_solimp_in,
    wp::float32 var_adhesion_in,
    wp::vec_t<2, wp::int32> var_geoms_in,
    wp::vec_t<2, wp::int32> var_pairid_in,
    wp::int32 var_worldid_in,
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
    wp::array_t<wp::int32> var_contact_efc_address_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::float32> var_contact_adhesion_out,
    wp::array_t<wp::int32> var_nacon_out)
{
    //---------
    // primal vars
    bool var_0;
    bool var_1;
    bool var_2;
    const wp::float32 var_3 = 0.0;
    bool var_4;
    wp::float32 var_5;
    bool var_6;
    wp::float32 var_7;
    bool var_8;
    bool var_9;
    const wp::float32 var_10 = 0.0;
    bool var_11;
    bool var_12;
    const wp::int32 var_13 = 1;
    wp::int32 var_14;
    bool var_15;
    bool var_16;
    const wp::int32 var_17 = 0;
    wp::int32 var_18;
    const wp::int32 var_19 = -2;
    bool var_20;
    bool var_21;
    const wp::int32 var_22 = 1;
    wp::int32 var_23;
    const wp::int32 var_24 = -1;
    bool var_25;
    const wp::int32 var_26 = 0;
    const wp::int32 var_27 = 0;
    bool var_28;
    const wp::int32 var_29 = 0;
    wp::int32 var_30;
    const wp::int32 var_31 = -1;
    bool var_32;
    const wp::int32 var_33 = 1;
    wp::int32 var_34;
    wp::int32 var_35;
    const wp::int32 var_36 = 1;
    wp::int32 var_37;
    const wp::int32 var_38 = 0;
    bool var_39;
    const wp::int32 var_40 = 2;
    wp::int32 var_41;
    wp::int32 var_42;
    const wp::int32 var_43 = 0;
    const wp::int32 var_44 = 1;
    wp::int32 var_45;
    bool var_46;
    wp::float32 var_47;
    wp::shape_t* var_48;
    const wp::int32 var_49 = 1;
    wp::int32 var_50;
    wp::shape_t var_51;
    wp::range_t var_52;
    wp::int32 var_53;
    const wp::int32 var_54 = -1;
    wp::int32 var_55;
    const wp::int32 var_56 = 0;
    //---------
    // forward
    // def write_contact(                                                                     <L 214>
    // active = (dist_in < margin_in) or ((adhesion_in != 0.0) and (dist_in < margin_in + gap_in))       <L 255>
    var_1 = (var_dist_in < var_margin_in);
    var_0 = var_1;
    if (!var_0) {
        var_4 = (var_adhesion_in != var_3);
        var_2 = var_4;
        if (var_2) {
            var_5 = wp::add(var_margin_in, var_gap_in);
            var_6 = (var_dist_in < var_5);
            var_2 = var_2 && var_6;
        }
        var_0 = var_0 || var_2;
    }
    // detected = dist_in < margin_in + gap_in                                                <L 256>
    var_7 = wp::add(var_margin_in, var_gap_in);
    var_8 = (var_dist_in < var_7);
    // condim = wp.where((adhesion_in != 0.0) and (dist_in >= margin_in), 1, condim_in)       <L 257>
    var_11 = (var_adhesion_in != var_10);
    var_9 = var_11;
    if (var_9) {
        var_12 = (var_dist_in >= var_margin_in);
        var_9 = var_9 && var_12;
    }
    var_14 = wp::where(var_9, var_13, var_condim_in);
    // if (pairid_in[0] == -2 or not detected) and pairid_in[1] == -1:                        <L 260>
    var_18 = wp::extract(var_pairid_in, var_17);
    var_20 = (var_18 == var_19);
    var_16 = var_20;
    if (!var_16) {
        var_21 = wp::unot(var_8);
        var_16 = var_16 || var_21;
    }
    var_15 = var_16;
    if (var_15) {
        var_23 = wp::extract(var_pairid_in, var_22);
        var_25 = (var_23 == var_24);
        var_15 = var_15 && var_25;
    }
    if (var_15) {
        // return 0                                                                           <L 261>
        return var_26;
    }
    // contact_type = 0                                                                       <L 263>
    // if pairid_in[0] >= -1 and detected:                                                    <L 265>
    var_30 = wp::extract(var_pairid_in, var_29);
    var_32 = (var_30 >= var_31);
    var_28 = var_32;
    if (var_28) {
        var_28 = var_28 && var_8;
    }
    if (var_28) {
        // contact_type |= ContactType.CONSTRAINT                                             <L 266>
        var_34 = wp::bit_or(var_27, var_33);
    }
    var_35 = wp::where(var_28, var_34, var_27);
    // if pairid_in[1] >= 0:                                                                  <L 268>
    var_37 = wp::extract(var_pairid_in, var_36);
    var_39 = (var_37 >= var_38);
    if (var_39) {
        // contact_type |= ContactType.SENSOR                                                 <L 269>
        var_41 = wp::bit_or(var_35, var_40);
    }
    var_42 = wp::where(var_39, var_41, var_35);
    // cid = wp.atomic_add(nacon_out, 0, 1)                                                   <L 271>
    var_45 = wp::atomic_add(var_nacon_out, var_43, var_44);
    // if cid < naconmax_in:                                                                  <L 272>
    var_46 = (var_45 < var_naconmax_in);
    if (var_46) {
        // contact_dist_out[cid] = dist_in                                                    <L 273>
        wp::array_store(var_contact_dist_out, var_45, var_dist_in);
        // contact_pos_out[cid] = pos_in                                                      <L 274>
        wp::array_store(var_contact_pos_out, var_45, var_pos_in);
        // contact_frame_out[cid] = frame_in                                                  <L 275>
        wp::array_store(var_contact_frame_out, var_45, var_frame_in);
        // contact_geom_out[cid] = geoms_in                                                   <L 276>
        wp::array_store(var_contact_geom_out, var_45, var_geoms_in);
        // contact_worldid_out[cid] = worldid_in                                              <L 277>
        wp::array_store(var_contact_worldid_out, var_45, var_worldid_in);
        // includemargin = margin_in                                                          <L 278>
        var_47 = wp::copy(var_margin_in);
        // contact_includemargin_out[cid] = includemargin                                     <L 279>
        wp::array_store(var_contact_includemargin_out, var_45, var_47);
        // contact_dim_out[cid] = condim                                                      <L 280>
        wp::array_store(var_contact_dim_out, var_45, var_14);
        // contact_friction_out[cid] = friction_in                                            <L 281>
        wp::array_store(var_contact_friction_out, var_45, var_friction_in);
        // contact_solref_out[cid] = solref_in                                                <L 282>
        wp::array_store(var_contact_solref_out, var_45, var_solref_in);
        // contact_solreffriction_out[cid] = solreffriction_in                                <L 283>
        wp::array_store(var_contact_solreffriction_out, var_45, var_solreffriction_in);
        // contact_solimp_out[cid] = solimp_in                                                <L 284>
        wp::array_store(var_contact_solimp_out, var_45, var_solimp_in);
        // contact_adhesion_out[cid] = adhesion_in                                            <L 285>
        wp::array_store(var_contact_adhesion_out, var_45, var_adhesion_in);
        // contact_type_out[cid] = contact_type                                               <L 286>
        wp::array_store(var_contact_type_out, var_45, var_42);
        // contact_geomcollisionid_out[cid] = id_                                             <L 287>
        wp::array_store(var_contact_geomcollisionid_out, var_45, var_id_);
        // for i in range(contact_efc_address_out.shape[1]):                                  <L 288>
        var_48 = &(var_contact_efc_address_out.shape);
        var_51 = wp::load(var_48);
        var_50 = wp::extract(var_51, var_49);
        var_52 = wp::range(var_50);
        start_for_1:;
            if (iter_cmp(var_52) == 0) goto end_for_1;
            var_53 = wp::iter_next(var_52);
            // contact_efc_address_out[cid, i] = -1                                           <L 289>
            wp::array_store(var_contact_efc_address_out, var_45, var_53, var_54);
            goto start_for_1;
        end_for_1:;
        // return int(active)                                                                 <L 290>
        var_55 = wp::int(var_0);
        return var_55;
    }
    // return 0                                                                               <L 291>
    return var_56;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive.py:280
static CUDA_CALLABLE void plane_sphere_wrapper_0(
    wp::int32 var_naconmax_in,
    Geom_4022bfa7 var_plane,
    Geom_4022bfa7 var_sphere,
    wp::int32 var_worldid,
    wp::float32 var_margin,
    wp::float32 var_gap,
    wp::int32 var_condim,
    wp::vec_t<5, wp::float32> var_friction,
    wp::vec_t<2, wp::float32> var_solref,
    wp::vec_t<2, wp::float32> var_solreffriction,
    wp::vec_t<5, wp::float32> var_solimp,
    wp::float32 var_adhesion,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::vec_t<2, wp::int32> var_pairid,
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
    wp::array_t<wp::int32> var_contact_efc_address_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::float32> var_contact_adhesion_out,
    wp::array_t<wp::int32> var_nacon_out)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32>* var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::vec_t<3, wp::float32>* var_3;
    wp::vec_t<3, wp::float32>* var_4;
    wp::vec_t<3, wp::float32>* var_5;
    const wp::int32 var_6 = 0;
    wp::float32 var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::float32 var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    const wp::int32 var_13 = 0;
    wp::mat_t<3, 3, wp::float32> var_14;
    wp::int32 var_15;
    //---------
    // forward
    // def plane_sphere_wrapper(                                                              <L 281>
    // normal = plane.normal                                                                  <L 317>
    var_0 = &((var_plane).normal);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // dist, pos = plane_sphere(normal, plane.pos, sphere.pos, sphere.size[0])                <L 318>
    var_3 = &((var_plane).pos);
    var_4 = &((var_sphere).pos);
    var_5 = &((var_sphere).size);
    var_8 = wp::load(var_5);
    var_7 = wp::extract(var_8, var_6);
    var_11 = wp::load(var_3);
    var_12 = wp::load(var_4);
    plane_sphere_0(var_1, var_11, var_12, var_7, var_9, var_10);
    // write_contact(                                                                         <L 320>
    // naconmax_in,                                                                           <L 321>
    // 0,                                                                                     <L 322>
    // dist,                                                                                  <L 323>
    // pos,                                                                                   <L 324>
    // make_frame(normal),                                                                    <L 325>
    var_14 = make_frame_0(var_1);
    // margin,                                                                                <L 326>
    // gap,                                                                                   <L 327>
    // condim,                                                                                <L 328>
    // friction,                                                                              <L 329>
    // solref,                                                                                <L 330>
    // solreffriction,                                                                        <L 331>
    // solimp,                                                                                <L 332>
    // adhesion,                                                                              <L 333>
    // geoms,                                                                                 <L 334>
    // pairid,                                                                                <L 335>
    // worldid,                                                                               <L 336>
    // contact_dist_out,                                                                      <L 337>
    // contact_pos_out,                                                                       <L 338>
    // contact_frame_out,                                                                     <L 339>
    // contact_includemargin_out,                                                             <L 340>
    // contact_friction_out,                                                                  <L 341>
    // contact_solref_out,                                                                    <L 342>
    // contact_solreffriction_out,                                                            <L 343>
    // contact_solimp_out,                                                                    <L 344>
    // contact_dim_out,                                                                       <L 345>
    // contact_geom_out,                                                                      <L 346>
    // contact_efc_address_out,                                                               <L 347>
    // contact_worldid_out,                                                                   <L 348>
    // contact_type_out,                                                                      <L 349>
    // contact_geomcollisionid_out,                                                           <L 350>
    // contact_adhesion_out,                                                                  <L 351>
    // nacon_out,                                                                             <L 352>
    var_15 = write_contact_0(var_naconmax_in, var_13, var_9, var_10, var_14, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:294
static CUDA_CALLABLE void adj_contact_margin_gap_0(
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_pair_margin,
    wp::array_t<wp::float32> var_pair_gap,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_pairid,
    wp::int32 var_worldid,
    wp::float32 & ret_0,
    wp::float32 & ret_1,
    wp::array_t<wp::float32> & adj_geom_margin,
    wp::array_t<wp::float32> & adj_geom_gap,
    wp::array_t<wp::float32> & adj_pair_margin,
    wp::array_t<wp::float32> & adj_pair_gap,
    wp::vec_t<2, wp::int32> & adj_geoms,
    wp::int32 & adj_pairid,
    wp::int32 & adj_worldid,
    wp::float32 & adj_ret_0,
    wp::float32 & adj_ret_1)
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
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    bool var_1;
    const wp::float32 var_2 = 1e-15;
    const wp::float32 var_3 = 1e-15;
    wp::float32 var_4;
    wp::float32 var_5;
    //---------
    // dual vars
    wp::float32 adj_0 = {};
    bool adj_1 = {};
    wp::float32 adj_2 = {};
    wp::float32 adj_3 = {};
    wp::float32 adj_4 = {};
    wp::float32 adj_5 = {};
    //---------
    // forward
    // def safe_div(x: Any, y: Any) -> Any:                                                   <L 1>
    // return x / wp.where(y != 0.0, y, types.MJ_MINVAL)                                      <L 2>
    var_1 = (var_y != var_0);
    var_4 = wp::where(var_1, var_y, var_3);
    var_5 = wp::div(var_x, var_4);
    goto label0;
    //---------
    // reverse
    label0:;
    adj_5 += adj_ret;
    wp::adj_div(var_x, var_4, var_5, adj_x, adj_4, adj_5);
    wp::adj_where(var_1, var_y, var_3, adj_1, adj_y, adj_3, adj_4);
    // adj: return x / wp.where(y != 0.0, y, types.MJ_MINVAL)                                 <L 2>
    // adj: def safe_div(x: Any, y: Any) -> Any:                                              <L 1>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:320
static CUDA_CALLABLE void adj_contact_material_params_0(
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_solimp,
    wp::array_t<wp::float32> var_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_friction,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_pairid,
    wp::int32 var_worldid,
    wp::int32 & ret_0,
    wp::vec_t<5, wp::float32> & ret_1,
    wp::vec_t<2, wp::float32> & ret_2,
    wp::vec_t<2, wp::float32> & ret_3,
    wp::vec_t<5, wp::float32> & ret_4,
    wp::float32 & ret_5,
    wp::array_t<wp::int32> & adj_geom_condim,
    wp::array_t<wp::int32> & adj_geom_priority,
    wp::array_t<wp::float32> & adj_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_friction,
    wp::array_t<wp::float32> & adj_geom_adhesion,
    wp::array_t<wp::int32> & adj_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_pair_solimp,
    wp::array_t<wp::float32> & adj_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_pair_friction,
    wp::vec_t<2, wp::int32> & adj_geoms,
    wp::int32 & adj_pairid,
    wp::int32 & adj_worldid,
    wp::int32 & adj_ret_0,
    wp::vec_t<5, wp::float32> & adj_ret_1,
    wp::vec_t<2, wp::float32> & adj_ret_2,
    wp::vec_t<2, wp::float32> & adj_ret_3,
    wp::vec_t<5, wp::float32> & adj_ret_4,
    wp::float32 & adj_ret_5)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:416
static CUDA_CALLABLE void adj_contact_params_0(
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_solimp,
    wp::array_t<wp::float32> var_pair_margin,
    wp::array_t<wp::float32> var_pair_gap,
    wp::array_t<wp::float32> var_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_friction,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_in,
    wp::int32 var_cid,
    wp::int32 var_worldid,
    wp::vec_t<2, wp::int32> & ret_0,
    wp::float32 & ret_1,
    wp::float32 & ret_2,
    wp::int32 & ret_3,
    wp::vec_t<5, wp::float32> & ret_4,
    wp::vec_t<2, wp::float32> & ret_5,
    wp::vec_t<2, wp::float32> & ret_6,
    wp::vec_t<5, wp::float32> & ret_7,
    wp::float32 & ret_8,
    wp::array_t<wp::int32> & adj_geom_condim,
    wp::array_t<wp::int32> & adj_geom_priority,
    wp::array_t<wp::float32> & adj_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_friction,
    wp::array_t<wp::float32> & adj_geom_margin,
    wp::array_t<wp::float32> & adj_geom_gap,
    wp::array_t<wp::float32> & adj_geom_adhesion,
    wp::array_t<wp::int32> & adj_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_pair_solimp,
    wp::array_t<wp::float32> & adj_pair_margin,
    wp::array_t<wp::float32> & adj_pair_gap,
    wp::array_t<wp::float32> & adj_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_pair_friction,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_collision_pairid_in,
    wp::int32 & adj_cid,
    wp::int32 & adj_worldid,
    wp::vec_t<2, wp::int32> & adj_ret_0,
    wp::float32 & adj_ret_1,
    wp::float32 & adj_ret_2,
    wp::int32 & adj_ret_3,
    wp::vec_t<5, wp::float32> & adj_ret_4,
    wp::vec_t<2, wp::float32> & adj_ret_5,
    wp::vec_t<2, wp::float32> & adj_ret_6,
    wp::vec_t<5, wp::float32> & adj_ret_7,
    wp::float32 & adj_ret_8)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:65
static CUDA_CALLABLE void adj_geom_collision_pair_from_types_0(
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::int32> var_mesh_polynum,
    wp::array_t<wp::int32> var_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_polynormal,
    wp::array_t<wp::int32> var_mesh_polyvertadr,
    wp::array_t<wp::int32> var_mesh_polyvertnum,
    wp::array_t<wp::int32> var_mesh_polyvert,
    wp::array_t<wp::int32> var_mesh_polymapadr,
    wp::array_t<wp::int32> var_mesh_polymapnum,
    wp::array_t<wp::int32> var_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_geom_type1,
    wp::int32 var_geom_type2,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_worldid,
    Geom_4022bfa7 & ret_0,
    Geom_4022bfa7 & ret_1,
    wp::array_t<wp::int32> & adj_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_size,
    wp::array_t<wp::int32> & adj_mesh_vertadr,
    wp::array_t<wp::int32> & adj_mesh_vertnum,
    wp::array_t<wp::int32> & adj_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_vert,
    wp::array_t<wp::int32> & adj_mesh_graph,
    wp::array_t<wp::int32> & adj_mesh_polynum,
    wp::array_t<wp::int32> & adj_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_polynormal,
    wp::array_t<wp::int32> & adj_mesh_polyvertadr,
    wp::array_t<wp::int32> & adj_mesh_polyvertnum,
    wp::array_t<wp::int32> & adj_mesh_polyvert,
    wp::array_t<wp::int32> & adj_mesh_polymapadr,
    wp::array_t<wp::int32> & adj_mesh_polymapnum,
    wp::array_t<wp::int32> & adj_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_geom_xmat_in,
    wp::int32 & adj_geom_type1,
    wp::int32 & adj_geom_type2,
    wp::vec_t<2, wp::int32> & adj_geoms,
    wp::int32 & adj_worldid,
    Geom_4022bfa7 & adj_ret_0,
    Geom_4022bfa7 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:158
static CUDA_CALLABLE void adj_geom_collision_pair_0(
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::int32> var_mesh_polynum,
    wp::array_t<wp::int32> var_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_polynormal,
    wp::array_t<wp::int32> var_mesh_polyvertadr,
    wp::array_t<wp::int32> var_mesh_polyvertnum,
    wp::array_t<wp::int32> var_mesh_polyvert,
    wp::array_t<wp::int32> var_mesh_polymapadr,
    wp::array_t<wp::int32> var_mesh_polymapnum,
    wp::array_t<wp::int32> var_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::int32 var_worldid,
    Geom_4022bfa7 & ret_0,
    Geom_4022bfa7 & ret_1,
    wp::array_t<wp::int32> & adj_geom_type,
    wp::array_t<wp::int32> & adj_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_size,
    wp::array_t<wp::int32> & adj_mesh_vertadr,
    wp::array_t<wp::int32> & adj_mesh_vertnum,
    wp::array_t<wp::int32> & adj_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_vert,
    wp::array_t<wp::int32> & adj_mesh_graph,
    wp::array_t<wp::int32> & adj_mesh_polynum,
    wp::array_t<wp::int32> & adj_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_polynormal,
    wp::array_t<wp::int32> & adj_mesh_polyvertadr,
    wp::array_t<wp::int32> & adj_mesh_polyvertnum,
    wp::array_t<wp::int32> & adj_mesh_polyvert,
    wp::array_t<wp::int32> & adj_mesh_polymapadr,
    wp::array_t<wp::int32> & adj_mesh_polymapnum,
    wp::array_t<wp::int32> & adj_mesh_polymap,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_geom_xmat_in,
    wp::vec_t<2, wp::int32> & adj_geoms,
    wp::int32 & adj_worldid,
    Geom_4022bfa7 & adj_ret_0,
    Geom_4022bfa7 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:50
static CUDA_CALLABLE void adj_plane_sphere_0(
    wp::vec_t<3, wp::float32> var_plane_normal,
    wp::vec_t<3, wp::float32> var_plane_pos,
    wp::vec_t<3, wp::float32> var_sphere_pos,
    wp::float32 var_sphere_radius,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_plane_normal,
    wp::vec_t<3, wp::float32> & adj_plane_pos,
    wp::vec_t<3, wp::float32> & adj_sphere_pos,
    wp::float32 & adj_sphere_radius,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
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
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\math.py:246
static CUDA_CALLABLE void adj_make_frame_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> & adj_a,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_core.py:213
static CUDA_CALLABLE void adj_write_contact_0(
    wp::int32 var_naconmax_in,
    wp::int32 var_id_,
    wp::float32 var_dist_in,
    wp::vec_t<3, wp::float32> var_pos_in,
    wp::mat_t<3, 3, wp::float32> var_frame_in,
    wp::float32 var_margin_in,
    wp::float32 var_gap_in,
    wp::int32 var_condim_in,
    wp::vec_t<5, wp::float32> var_friction_in,
    wp::vec_t<2, wp::float32> var_solref_in,
    wp::vec_t<2, wp::float32> var_solreffriction_in,
    wp::vec_t<5, wp::float32> var_solimp_in,
    wp::float32 var_adhesion_in,
    wp::vec_t<2, wp::int32> var_geoms_in,
    wp::vec_t<2, wp::int32> var_pairid_in,
    wp::int32 var_worldid_in,
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
    wp::array_t<wp::int32> var_contact_efc_address_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::float32> var_contact_adhesion_out,
    wp::array_t<wp::int32> var_nacon_out,
    wp::int32 & adj_naconmax_in,
    wp::int32 & adj_id_,
    wp::float32 & adj_dist_in,
    wp::vec_t<3, wp::float32> & adj_pos_in,
    wp::mat_t<3, 3, wp::float32> & adj_frame_in,
    wp::float32 & adj_margin_in,
    wp::float32 & adj_gap_in,
    wp::int32 & adj_condim_in,
    wp::vec_t<5, wp::float32> & adj_friction_in,
    wp::vec_t<2, wp::float32> & adj_solref_in,
    wp::vec_t<2, wp::float32> & adj_solreffriction_in,
    wp::vec_t<5, wp::float32> & adj_solimp_in,
    wp::float32 & adj_adhesion_in,
    wp::vec_t<2, wp::int32> & adj_geoms_in,
    wp::vec_t<2, wp::int32> & adj_pairid_in,
    wp::int32 & adj_worldid_in,
    wp::array_t<wp::float32> & adj_contact_dist_out,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_contact_pos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_contact_frame_out,
    wp::array_t<wp::float32> & adj_contact_includemargin_out,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_contact_friction_out,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_contact_solref_out,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_contact_solreffriction_out,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_contact_solimp_out,
    wp::array_t<wp::int32> & adj_contact_dim_out,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_contact_geom_out,
    wp::array_t<wp::int32> & adj_contact_efc_address_out,
    wp::array_t<wp::int32> & adj_contact_worldid_out,
    wp::array_t<wp::int32> & adj_contact_type_out,
    wp::array_t<wp::int32> & adj_contact_geomcollisionid_out,
    wp::array_t<wp::float32> & adj_contact_adhesion_out,
    wp::array_t<wp::int32> & adj_nacon_out,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive.py:280
static CUDA_CALLABLE void adj_plane_sphere_wrapper_0(
    wp::int32 var_naconmax_in,
    Geom_4022bfa7 var_plane,
    Geom_4022bfa7 var_sphere,
    wp::int32 var_worldid,
    wp::float32 var_margin,
    wp::float32 var_gap,
    wp::int32 var_condim,
    wp::vec_t<5, wp::float32> var_friction,
    wp::vec_t<2, wp::float32> var_solref,
    wp::vec_t<2, wp::float32> var_solreffriction,
    wp::vec_t<5, wp::float32> var_solimp,
    wp::float32 var_adhesion,
    wp::vec_t<2, wp::int32> var_geoms,
    wp::vec_t<2, wp::int32> var_pairid,
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
    wp::array_t<wp::int32> var_contact_efc_address_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::float32> var_contact_adhesion_out,
    wp::array_t<wp::int32> var_nacon_out,
    wp::int32 & adj_naconmax_in,
    Geom_4022bfa7 & adj_plane,
    Geom_4022bfa7 & adj_sphere,
    wp::int32 & adj_worldid,
    wp::float32 & adj_margin,
    wp::float32 & adj_gap,
    wp::int32 & adj_condim,
    wp::vec_t<5, wp::float32> & adj_friction,
    wp::vec_t<2, wp::float32> & adj_solref,
    wp::vec_t<2, wp::float32> & adj_solreffriction,
    wp::vec_t<5, wp::float32> & adj_solimp,
    wp::float32 & adj_adhesion,
    wp::vec_t<2, wp::int32> & adj_geoms,
    wp::vec_t<2, wp::int32> & adj_pairid,
    wp::array_t<wp::float32> & adj_contact_dist_out,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_contact_pos_out,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_contact_frame_out,
    wp::array_t<wp::float32> & adj_contact_includemargin_out,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_contact_friction_out,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_contact_solref_out,
    wp::array_t<wp::vec_t<2, wp::float32>> & adj_contact_solreffriction_out,
    wp::array_t<wp::vec_t<5, wp::float32>> & adj_contact_solimp_out,
    wp::array_t<wp::int32> & adj_contact_dim_out,
    wp::array_t<wp::vec_t<2, wp::int32>> & adj_contact_geom_out,
    wp::array_t<wp::int32> & adj_contact_efc_address_out,
    wp::array_t<wp::int32> & adj_contact_worldid_out,
    wp::array_t<wp::int32> & adj_contact_type_out,
    wp::array_t<wp::int32> & adj_contact_geomcollisionid_out,
    wp::array_t<wp::float32> & adj_contact_adhesion_out,
    wp::array_t<wp::int32> & adj_nacon_out)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}



extern "C" __global__ void _primitive_narrowphase__locals__primitive_narrowphase_a1733669_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::int32> var_mesh_polynum,
    wp::array_t<wp::int32> var_mesh_polyadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_polynormal,
    wp::array_t<wp::int32> var_mesh_polyvertadr,
    wp::array_t<wp::int32> var_mesh_polyvertnum,
    wp::array_t<wp::int32> var_mesh_polyvert,
    wp::array_t<wp::int32> var_mesh_polymapadr,
    wp::array_t<wp::int32> var_mesh_polymapnum,
    wp::array_t<wp::int32> var_mesh_polymap,
    wp::array_t<wp::int32> var_pair_dim,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solref,
    wp::array_t<wp::vec_t<2, wp::float32>> var_pair_solreffriction,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_solimp,
    wp::array_t<wp::float32> var_pair_margin,
    wp::array_t<wp::float32> var_pair_gap,
    wp::array_t<wp::float32> var_pair_adhesion,
    wp::array_t<wp::vec_t<5, wp::float32>> var_pair_friction,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_naconmax_in,
    wp::array_t<wp::int32> var_ncollision_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_in,
    wp::array_t<wp::int32> var_collision_worldid_in,
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
    wp::array_t<wp::int32> var_contact_efc_address_out,
    wp::array_t<wp::int32> var_contact_worldid_out,
    wp::array_t<wp::int32> var_contact_type_out,
    wp::array_t<wp::int32> var_contact_geomcollisionid_out,
    wp::array_t<wp::float32> var_contact_adhesion_out,
    wp::array_t<wp::int32> var_nacon_out)
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
        wp::int32* var_8;
        wp::int32 var_9;
        wp::int32 var_10;
        wp::vec_t<2, wp::int32> var_11;
        wp::float32 var_12;
        wp::float32 var_13;
        wp::int32 var_14;
        wp::vec_t<5, wp::float32> var_15;
        wp::vec_t<2, wp::float32> var_16;
        wp::vec_t<2, wp::float32> var_17;
        wp::vec_t<5, wp::float32> var_18;
        wp::float32 var_19;
        Geom_4022bfa7 var_20;
        Geom_4022bfa7 var_21;
        const wp::int32 var_22 = 0;
        const wp::int32 var_23 = 0;
        const wp::int32 var_24 = 2;
        const wp::int32 var_25 = 0;
        wp::int32 var_26;
        wp::int32* var_27;
        wp::int32 var_28;
        wp::int32 var_29;
        const wp::int32 var_30 = 1;
        wp::int32 var_31;
        wp::int32* var_32;
        wp::int32 var_33;
        wp::int32 var_34;
        bool var_35;
        bool var_36;
        bool var_37;
        wp::vec_t<2, wp::int32>* var_38;
        wp::vec_t<2, wp::int32> var_39;
        //---------
        // forward
        // def primitive_narrowphase(                                                             <L 1355>
        // tid = wp.tid()                                                                         <L 1418>
        var_0 = builtin_tid1d();
        // if tid >= ncollision_in[0]:                                                            <L 1420>
        var_2 = wp::address(var_ncollision_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 1421>
            continue;
        }
        // geoms = collision_pair_in[tid]                                                         <L 1423>
        var_5 = wp::address(var_collision_pair_in, var_0);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // worldid = collision_worldid_in[tid]                                                    <L 1424>
        var_8 = wp::address(var_collision_worldid_in, var_0);
        var_10 = wp::load(var_8);
        var_9 = wp::copy(var_10);
        // _, margin, gap, condim, friction, solref, solreffriction, solimp, adhesion = contact_params(       <L 1426>
        // geom_condim,                                                                           <L 1427>
        // geom_priority,                                                                         <L 1428>
        // geom_solmix,                                                                           <L 1429>
        // geom_solref,                                                                           <L 1430>
        // geom_solimp,                                                                           <L 1431>
        // geom_friction,                                                                         <L 1432>
        // geom_margin,                                                                           <L 1433>
        // geom_gap,                                                                              <L 1434>
        // geom_adhesion,                                                                         <L 1435>
        // pair_dim,                                                                              <L 1436>
        // pair_solref,                                                                           <L 1437>
        // pair_solreffriction,                                                                   <L 1438>
        // pair_solimp,                                                                           <L 1439>
        // pair_margin,                                                                           <L 1440>
        // pair_gap,                                                                              <L 1441>
        // pair_adhesion,                                                                         <L 1442>
        // pair_friction,                                                                         <L 1443>
        // collision_pair_in,                                                                     <L 1444>
        // collision_pairid_in,                                                                   <L 1445>
        // tid,                                                                                   <L 1446>
        // worldid,                                                                               <L 1447>
        contact_params_0(var_geom_condim, var_geom_priority, var_geom_solmix, var_geom_solref, var_geom_solimp, var_geom_friction, var_geom_margin, var_geom_gap, var_geom_adhesion, var_pair_dim, var_pair_solref, var_pair_solreffriction, var_pair_solimp, var_pair_margin, var_pair_gap, var_pair_adhesion, var_pair_friction, var_collision_pair_in, var_collision_pairid_in, var_0, var_9, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19);
        // geom1, geom2 = geom_collision_pair(                                                    <L 1450>
        // geom_type,                                                                             <L 1451>
        // geom_dataid,                                                                           <L 1452>
        // geom_size,                                                                             <L 1453>
        // mesh_vertadr,                                                                          <L 1454>
        // mesh_vertnum,                                                                          <L 1455>
        // mesh_graphadr,                                                                         <L 1456>
        // mesh_vert,                                                                             <L 1457>
        // mesh_graph,                                                                            <L 1458>
        // mesh_polynum,                                                                          <L 1459>
        // mesh_polyadr,                                                                          <L 1460>
        // mesh_polynormal,                                                                       <L 1461>
        // mesh_polyvertadr,                                                                      <L 1462>
        // mesh_polyvertnum,                                                                      <L 1463>
        // mesh_polyvert,                                                                         <L 1464>
        // mesh_polymapadr,                                                                       <L 1465>
        // mesh_polymapnum,                                                                       <L 1466>
        // mesh_polymap,                                                                          <L 1467>
        // geom_xpos_in,                                                                          <L 1468>
        // geom_xmat_in,                                                                          <L 1469>
        // geoms,                                                                                 <L 1470>
        // worldid,                                                                               <L 1471>
        geom_collision_pair_0(var_geom_type, var_geom_dataid, var_geom_size, var_mesh_vertadr, var_mesh_vertnum, var_mesh_graphadr, var_mesh_vert, var_mesh_graph, var_mesh_polynum, var_mesh_polyadr, var_mesh_polynormal, var_mesh_polyvertadr, var_mesh_polyvertnum, var_mesh_polyvert, var_mesh_polymapadr, var_mesh_polymapnum, var_mesh_polymap, var_geom_xpos_in, var_geom_xmat_in, var_6, var_9, var_20, var_21);
        // for i in range(wp.static(len(primitive_collisions_func))):                             <L 1474>
        // collision_type1 = wp.static(primitive_collisions_types[i][0])                          <L 1475>
        // collision_type2 = wp.static(primitive_collisions_types[i][1])                          <L 1476>
        // type1 = geom_type[geoms[0]]                                                            <L 1477>
        var_26 = wp::extract(var_6, var_25);
        var_27 = wp::address(var_geom_type, var_26);
        var_29 = wp::load(var_27);
        var_28 = wp::copy(var_29);
        // type2 = geom_type[geoms[1]]                                                            <L 1478>
        var_31 = wp::extract(var_6, var_30);
        var_32 = wp::address(var_geom_type, var_31);
        var_34 = wp::load(var_32);
        var_33 = wp::copy(var_34);
        // if collision_type1 == type1 and collision_type2 == type2:                              <L 1479>
        var_36 = (var_23 == var_28);
        var_35 = var_36;
        if (var_35) {
            var_37 = (var_24 == var_33);
            var_35 = var_35 && var_37;
        }
        if (var_35) {
            // wp.static(primitive_collisions_func[i])(                                           <L 1480>
            // naconmax_in,                                                                       <L 1481>
            // geom1,                                                                             <L 1482>
            // geom2,                                                                             <L 1483>
            // worldid,                                                                           <L 1484>
            // margin,                                                                            <L 1485>
            // gap,                                                                               <L 1486>
            // condim,                                                                            <L 1487>
            // friction,                                                                          <L 1488>
            // solref,                                                                            <L 1489>
            // solreffriction,                                                                    <L 1490>
            // solimp,                                                                            <L 1491>
            // adhesion,                                                                          <L 1492>
            // geoms,                                                                             <L 1493>
            // collision_pairid_in[tid],                                                          <L 1494>
            var_38 = wp::address(var_collision_pairid_in, var_0);
            // contact_dist_out,                                                                  <L 1495>
            // contact_pos_out,                                                                   <L 1496>
            // contact_frame_out,                                                                 <L 1497>
            // contact_includemargin_out,                                                         <L 1498>
            // contact_friction_out,                                                              <L 1499>
            // contact_solref_out,                                                                <L 1500>
            // contact_solreffriction_out,                                                        <L 1501>
            // contact_solimp_out,                                                                <L 1502>
            // contact_dim_out,                                                                   <L 1503>
            // contact_geom_out,                                                                  <L 1504>
            // contact_efc_address_out,                                                           <L 1505>
            // contact_worldid_out,                                                               <L 1506>
            // contact_type_out,                                                                  <L 1507>
            // contact_geomcollisionid_out,                                                       <L 1508>
            // contact_adhesion_out,                                                              <L 1509>
            // nacon_out,                                                                         <L 1510>
            var_39 = wp::load(var_38);
            plane_sphere_wrapper_0(var_naconmax_in, var_20, var_21, var_9, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_6, var_39, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        }
    }
}

