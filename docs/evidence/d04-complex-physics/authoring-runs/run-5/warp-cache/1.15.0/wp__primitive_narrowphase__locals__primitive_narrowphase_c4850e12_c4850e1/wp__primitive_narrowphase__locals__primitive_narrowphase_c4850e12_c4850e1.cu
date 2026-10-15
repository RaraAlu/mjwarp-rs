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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:562
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> _compute_rotmore_0(
    wp::int32 var_face_idx)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::mat_t<3, 3, wp::float32> var_1;
    const wp::int32 var_2 = 0;
    bool var_3;
    const wp::float32 var_4 = -1.0;
    const wp::int32 var_5 = 0;
    const wp::int32 var_6 = 2;
    const wp::float32 var_7 = 1.0;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    const wp::int32 var_10 = 1;
    const wp::float32 var_11 = 1.0;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    const wp::int32 var_14 = 0;
    const wp::int32 var_15 = 1;
    bool var_16;
    const wp::float32 var_17 = 1.0;
    wp::float32 var_18;
    const wp::int32 var_19 = 0;
    const wp::int32 var_20 = 0;
    const wp::float32 var_21 = -1.0;
    const wp::int32 var_22 = 1;
    const wp::int32 var_23 = 2;
    const wp::float32 var_24 = 1.0;
    wp::float32 var_25;
    const wp::int32 var_26 = 2;
    const wp::int32 var_27 = 1;
    const wp::int32 var_28 = 2;
    bool var_29;
    const wp::float32 var_30 = 1.0;
    wp::float32 var_31;
    const wp::int32 var_32 = 0;
    const wp::int32 var_33 = 0;
    const wp::float32 var_34 = 1.0;
    wp::float32 var_35;
    const wp::int32 var_36 = 1;
    const wp::int32 var_37 = 1;
    const wp::float32 var_38 = 1.0;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    const wp::int32 var_41 = 2;
    const wp::int32 var_42 = 3;
    bool var_43;
    const wp::float32 var_44 = 1.0;
    wp::float32 var_45;
    const wp::int32 var_46 = 0;
    const wp::int32 var_47 = 2;
    const wp::float32 var_48 = 1.0;
    wp::float32 var_49;
    const wp::int32 var_50 = 1;
    const wp::int32 var_51 = 1;
    const wp::float32 var_52 = -1.0;
    const wp::int32 var_53 = 2;
    const wp::int32 var_54 = 0;
    const wp::int32 var_55 = 4;
    bool var_56;
    const wp::float32 var_57 = 1.0;
    wp::float32 var_58;
    const wp::int32 var_59 = 0;
    const wp::int32 var_60 = 0;
    const wp::float32 var_61 = 1.0;
    wp::float32 var_62;
    const wp::int32 var_63 = 1;
    const wp::int32 var_64 = 2;
    const wp::float32 var_65 = -1.0;
    const wp::int32 var_66 = 2;
    const wp::int32 var_67 = 1;
    const wp::int32 var_68 = 5;
    bool var_69;
    const wp::float32 var_70 = -1.0;
    const wp::int32 var_71 = 0;
    const wp::int32 var_72 = 0;
    const wp::float32 var_73 = 1.0;
    wp::float32 var_74;
    const wp::int32 var_75 = 1;
    const wp::int32 var_76 = 1;
    const wp::float32 var_77 = -1.0;
    const wp::int32 var_78 = 2;
    const wp::int32 var_79 = 2;
    //---------
    // forward
    // def _compute_rotmore(face_idx: int) -> wp.mat33:                                       <L 563>
    // rotmore = wp.mat33(0.0)                                                                <L 564>
    var_1 = wp::mat_t<3, 3, wp::float32>(var_0);
    // if face_idx == 0:                                                                      <L 566>
    var_3 = (var_face_idx == var_2);
    if (var_3) {
        // rotmore[0, 2] = -1.0                                                               <L 567>
        wp::assign_inplace(var_1, var_5, var_6, var_4);
        // rotmore[1, 1] = +1.0                                                               <L 568>
        var_8 = wp::pos(var_7);
        wp::assign_inplace(var_1, var_9, var_10, var_8);
        // rotmore[2, 0] = +1.0                                                               <L 569>
        var_12 = wp::pos(var_11);
        wp::assign_inplace(var_1, var_13, var_14, var_12);
    }
    if (!var_3) {
        // elif face_idx == 1:                                                                <L 570>
        var_16 = (var_face_idx == var_15);
        if (var_16) {
            // rotmore[0, 0] = +1.0                                                           <L 571>
            var_18 = wp::pos(var_17);
            wp::assign_inplace(var_1, var_19, var_20, var_18);
            // rotmore[1, 2] = -1.0                                                           <L 572>
            wp::assign_inplace(var_1, var_22, var_23, var_21);
            // rotmore[2, 1] = +1.0                                                           <L 573>
            var_25 = wp::pos(var_24);
            wp::assign_inplace(var_1, var_26, var_27, var_25);
        }
        if (!var_16) {
            // elif face_idx == 2:                                                            <L 574>
            var_29 = (var_face_idx == var_28);
            if (var_29) {
                // rotmore[0, 0] = +1.0                                                       <L 575>
                var_31 = wp::pos(var_30);
                wp::assign_inplace(var_1, var_32, var_33, var_31);
                // rotmore[1, 1] = +1.0                                                       <L 576>
                var_35 = wp::pos(var_34);
                wp::assign_inplace(var_1, var_36, var_37, var_35);
                // rotmore[2, 2] = +1.0                                                       <L 577>
                var_39 = wp::pos(var_38);
                wp::assign_inplace(var_1, var_40, var_41, var_39);
            }
            if (!var_29) {
                // elif face_idx == 3:                                                        <L 578>
                var_43 = (var_face_idx == var_42);
                if (var_43) {
                    // rotmore[0, 2] = +1.0                                                   <L 579>
                    var_45 = wp::pos(var_44);
                    wp::assign_inplace(var_1, var_46, var_47, var_45);
                    // rotmore[1, 1] = +1.0                                                   <L 580>
                    var_49 = wp::pos(var_48);
                    wp::assign_inplace(var_1, var_50, var_51, var_49);
                    // rotmore[2, 0] = -1.0                                                   <L 581>
                    wp::assign_inplace(var_1, var_53, var_54, var_52);
                }
                if (!var_43) {
                    // elif face_idx == 4:                                                    <L 582>
                    var_56 = (var_face_idx == var_55);
                    if (var_56) {
                        // rotmore[0, 0] = +1.0                                               <L 583>
                        var_58 = wp::pos(var_57);
                        wp::assign_inplace(var_1, var_59, var_60, var_58);
                        // rotmore[1, 2] = +1.0                                               <L 584>
                        var_62 = wp::pos(var_61);
                        wp::assign_inplace(var_1, var_63, var_64, var_62);
                        // rotmore[2, 1] = -1.0                                               <L 585>
                        wp::assign_inplace(var_1, var_66, var_67, var_65);
                    }
                    if (!var_56) {
                        // elif face_idx == 5:                                                <L 586>
                        var_69 = (var_face_idx == var_68);
                        if (var_69) {
                            // rotmore[0, 0] = -1.0                                           <L 587>
                            wp::assign_inplace(var_1, var_71, var_72, var_70);
                            // rotmore[1, 1] = +1.0                                           <L 588>
                            var_74 = wp::pos(var_73);
                            wp::assign_inplace(var_1, var_75, var_76, var_74);
                            // rotmore[2, 2] = -1.0                                           <L 589>
                            wp::assign_inplace(var_1, var_78, var_79, var_77);
                        }
                    }
                }
            }
        }
    }
    // return rotmore                                                                         <L 591>
    return var_1;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:594
static CUDA_CALLABLE void box_box_0(
    wp::vec_t<3, wp::float32> var_box1_pos,
    wp::mat_t<3, 3, wp::float32> var_box1_rot,
    wp::vec_t<3, wp::float32> var_box1_size,
    wp::vec_t<3, wp::float32> var_box2_pos,
    wp::mat_t<3, 3, wp::float32> var_box2_rot,
    wp::vec_t<3, wp::float32> var_box2_size,
    wp::float32 var_margin,
    wp::vec_t<8, wp::float32> & ret_0,
    wp::mat_t<8, 3, wp::float32> & ret_1,
    wp::mat_t<8, 3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    wp::vec_t<8, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    const wp::float32 var_2 = 10000000000.0;
    const wp::int32 var_3 = 1;
    const wp::int32 var_4 = 2;
    const wp::int32 var_5 = 3;
    const wp::int32 var_6 = 4;
    const wp::int32 var_7 = 5;
    const wp::int32 var_8 = 6;
    const wp::int32 var_9 = 7;
    wp::mat_t<8, 3, wp::float32> var_10;
    wp::mat_t<8, 3, wp::float32> var_11;
    const wp::int32 var_12 = 0;
    wp::mat_t<3, 3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::mat_t<3, 3, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::mat_t<3, 3, wp::float32> var_19;
    wp::mat_t<3, 3, wp::float32> var_20;
    wp::mat_t<3, 3, wp::float32> var_21;
    const wp::int32 var_22 = 0;
    wp::vec_t<3, wp::float32> var_23;
    wp::vec_t<3, wp::float32> var_24;
    const wp::int32 var_25 = 1;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    const wp::int32 var_28 = 2;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    wp::mat_t<3, 3, wp::float32> var_31;
    wp::mat_t<3, 3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    const wp::float32 var_35 = 3.0;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    const wp::int32 var_38 = 0;
    wp::float32 var_39;
    wp::float32 var_40;
    const wp::int32 var_41 = 1;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::int32 var_44 = 2;
    wp::float32 var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    const wp::int32 var_48 = -1;
    wp::int32 var_49;
    const wp::int32 var_50 = 0;
    wp::float32 var_51;
    wp::float32 var_52;
    wp::float32 var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::float32 var_57;
    wp::float32 var_58;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    wp::float32 var_63;
    wp::float32 var_64;
    bool var_65;
    wp::float32 var_66;
    bool var_67;
    wp::float32 var_68;
    bool var_69;
    bool var_70;
    wp::float32 var_71;
    const wp::int32 var_72 = 3;
    wp::float32 var_73;
    const wp::int32 var_74 = 0;
    bool var_75;
    wp::int32 var_76;
    wp::int32 var_77;
    wp::int32 var_78;
    const wp::int32 var_79 = 0;
    wp::int32 var_80;
    wp::float32 var_81;
    wp::int32 var_82;
    bool var_83;
    wp::float32 var_84;
    const wp::int32 var_85 = 3;
    wp::float32 var_86;
    const wp::int32 var_87 = 0;
    bool var_88;
    wp::int32 var_89;
    wp::int32 var_90;
    wp::int32 var_91;
    const wp::int32 var_92 = 6;
    wp::int32 var_93;
    wp::float32 var_94;
    wp::int32 var_95;
    const wp::int32 var_96 = 1;
    wp::float32 var_97;
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
    bool var_111;
    wp::float32 var_112;
    bool var_113;
    wp::float32 var_114;
    bool var_115;
    bool var_116;
    wp::float32 var_117;
    const wp::int32 var_118 = 3;
    wp::float32 var_119;
    const wp::int32 var_120 = 0;
    bool var_121;
    wp::int32 var_122;
    wp::int32 var_123;
    wp::int32 var_124;
    const wp::int32 var_125 = 0;
    wp::int32 var_126;
    wp::float32 var_127;
    wp::int32 var_128;
    bool var_129;
    wp::float32 var_130;
    const wp::int32 var_131 = 3;
    wp::float32 var_132;
    const wp::int32 var_133 = 0;
    bool var_134;
    wp::int32 var_135;
    wp::int32 var_136;
    wp::int32 var_137;
    const wp::int32 var_138 = 6;
    wp::int32 var_139;
    wp::float32 var_140;
    wp::int32 var_141;
    const wp::int32 var_142 = 2;
    wp::float32 var_143;
    wp::float32 var_144;
    wp::float32 var_145;
    wp::float32 var_146;
    wp::float32 var_147;
    wp::float32 var_148;
    wp::float32 var_149;
    wp::float32 var_150;
    wp::float32 var_151;
    wp::float32 var_152;
    wp::float32 var_153;
    wp::float32 var_154;
    wp::float32 var_155;
    wp::float32 var_156;
    bool var_157;
    wp::float32 var_158;
    bool var_159;
    wp::float32 var_160;
    bool var_161;
    bool var_162;
    wp::float32 var_163;
    const wp::int32 var_164 = 3;
    wp::float32 var_165;
    const wp::int32 var_166 = 0;
    bool var_167;
    wp::int32 var_168;
    wp::int32 var_169;
    wp::int32 var_170;
    const wp::int32 var_171 = 0;
    wp::int32 var_172;
    wp::float32 var_173;
    wp::int32 var_174;
    bool var_175;
    wp::float32 var_176;
    const wp::int32 var_177 = 3;
    wp::float32 var_178;
    const wp::int32 var_179 = 0;
    bool var_180;
    wp::int32 var_181;
    wp::int32 var_182;
    wp::int32 var_183;
    const wp::int32 var_184 = 6;
    wp::int32 var_185;
    wp::float32 var_186;
    wp::int32 var_187;
    const wp::float32 var_188 = 0.0;
    wp::vec_t<3, wp::float32> var_189;
    const bool var_190 = false;
    bool var_191;
    const wp::int32 var_192 = 0;
    wp::int32 var_193;
    const wp::int32 var_194 = 0;
    wp::int32 var_195;
    const wp::int32 var_196 = 0;
    const wp::int32 var_197 = 3;
    wp::range_t var_198;
    wp::int32 var_199;
    const wp::int32 var_200 = 0;
    bool var_201;
    const wp::float32 var_202 = 0.0;
    const wp::int32 var_203 = 2;
    wp::float32 var_204;
    wp::float32 var_205;
    const wp::int32 var_206 = 1;
    wp::float32 var_207;
    wp::vec_t<3, wp::float32> var_208;
    const wp::int32 var_209 = 1;
    bool var_210;
    const wp::int32 var_211 = 2;
    wp::float32 var_212;
    const wp::float32 var_213 = 0.0;
    const wp::int32 var_214 = 0;
    wp::float32 var_215;
    wp::float32 var_216;
    wp::vec_t<3, wp::float32> var_217;
    wp::vec_t<3, wp::float32> var_218;
    const wp::int32 var_219 = 1;
    wp::float32 var_220;
    wp::float32 var_221;
    const wp::int32 var_222 = 0;
    wp::float32 var_223;
    const wp::float32 var_224 = 0.0;
    wp::vec_t<3, wp::float32> var_225;
    wp::vec_t<3, wp::float32> var_226;
    wp::vec_t<3, wp::float32> var_227;
    wp::float32 var_228;
    const wp::float32 var_229 = 1e-15;
    bool var_230;
    wp::vec_t<3, wp::float32> var_231;
    wp::float32 var_232;
    const wp::float32 var_233 = 0.0;
    wp::float32 var_234;
    const wp::int32 var_235 = 0;
    bool var_236;
    wp::float32 var_237;
    wp::float32 var_238;
    wp::float32 var_239;
    wp::float32 var_240;
    wp::float32 var_241;
    wp::float32 var_242;
    bool var_243;
    wp::float32 var_244;
    const wp::int32 var_245 = 3;
    wp::int32 var_246;
    wp::int32 var_247;
    wp::float32 var_248;
    wp::float32 var_249;
    wp::float32 var_250;
    wp::float32 var_251;
    wp::float32 var_252;
    const wp::int32 var_253 = 1;
    bool var_254;
    wp::float32 var_255;
    wp::float32 var_256;
    wp::float32 var_257;
    wp::float32 var_258;
    wp::float32 var_259;
    wp::float32 var_260;
    bool var_261;
    wp::float32 var_262;
    const wp::int32 var_263 = 3;
    wp::int32 var_264;
    wp::int32 var_265;
    wp::float32 var_266;
    wp::float32 var_267;
    wp::float32 var_268;
    wp::float32 var_269;
    wp::float32 var_270;
    const wp::int32 var_271 = 2;
    bool var_272;
    wp::float32 var_273;
    wp::float32 var_274;
    wp::float32 var_275;
    wp::float32 var_276;
    wp::float32 var_277;
    wp::float32 var_278;
    bool var_279;
    wp::float32 var_280;
    const wp::int32 var_281 = 3;
    wp::int32 var_282;
    wp::int32 var_283;
    wp::float32 var_284;
    wp::float32 var_285;
    wp::float32 var_286;
    wp::float32 var_287;
    wp::float32 var_288;
    wp::float32 var_289;
    wp::float32 var_290;
    wp::float32 var_291;
    bool var_292;
    const wp::float32 var_293 = 1.0;
    const wp::float32 var_294 = 1e-12;
    wp::float32 var_295;
    wp::float32 var_296;
    bool var_297;
    wp::float32 var_298;
    const wp::int32 var_299 = 0;
    const wp::int32 var_300 = 0;
    const wp::int32 var_301 = 0;
    bool var_302;
    bool var_303;
    wp::float32 var_304;
    const wp::int32 var_305 = 0;
    bool var_306;
    wp::int32 var_307;
    const wp::int32 var_308 = 0;
    bool var_309;
    wp::int32 var_310;
    wp::int32 var_311;
    const wp::int32 var_312 = 1;
    wp::int32 var_313;
    wp::int32 var_314;
    wp::int32 var_315;
    bool var_316;
    const wp::int32 var_317 = 3;
    wp::int32 var_318;
    wp::int32 var_319;
    wp::float32 var_320;
    const wp::int32 var_321 = 0;
    bool var_322;
    wp::int32 var_323;
    const wp::int32 var_324 = 0;
    bool var_325;
    wp::int32 var_326;
    wp::int32 var_327;
    wp::int32 var_328;
    const wp::int32 var_329 = 3;
    wp::int32 var_330;
    const wp::int32 var_331 = 3;
    wp::int32 var_332;
    const wp::int32 var_333 = 1;
    bool var_334;
    wp::int32 var_335;
    wp::int32 var_336;
    const wp::int32 var_337 = 1;
    wp::int32 var_338;
    wp::int32 var_339;
    wp::int32 var_340;
    wp::int32 var_341;
    const wp::int32 var_342 = 1;
    bool var_343;
    bool var_344;
    wp::float32 var_345;
    const wp::int32 var_346 = 0;
    bool var_347;
    wp::int32 var_348;
    const wp::int32 var_349 = 0;
    bool var_350;
    wp::int32 var_351;
    wp::int32 var_352;
    const wp::int32 var_353 = 1;
    wp::int32 var_354;
    wp::int32 var_355;
    wp::int32 var_356;
    bool var_357;
    const wp::int32 var_358 = 3;
    wp::int32 var_359;
    wp::int32 var_360;
    wp::float32 var_361;
    const wp::int32 var_362 = 0;
    bool var_363;
    wp::int32 var_364;
    const wp::int32 var_365 = 0;
    bool var_366;
    wp::int32 var_367;
    wp::int32 var_368;
    wp::int32 var_369;
    const wp::int32 var_370 = 3;
    wp::int32 var_371;
    const wp::int32 var_372 = 3;
    wp::int32 var_373;
    const wp::int32 var_374 = 1;
    bool var_375;
    wp::int32 var_376;
    wp::int32 var_377;
    const wp::int32 var_378 = 1;
    wp::int32 var_379;
    wp::int32 var_380;
    wp::int32 var_381;
    wp::int32 var_382;
    const wp::int32 var_383 = 2;
    bool var_384;
    bool var_385;
    wp::float32 var_386;
    const wp::int32 var_387 = 0;
    bool var_388;
    wp::int32 var_389;
    const wp::int32 var_390 = 0;
    bool var_391;
    wp::int32 var_392;
    wp::int32 var_393;
    const wp::int32 var_394 = 1;
    wp::int32 var_395;
    wp::int32 var_396;
    wp::int32 var_397;
    bool var_398;
    const wp::int32 var_399 = 3;
    wp::int32 var_400;
    wp::int32 var_401;
    wp::float32 var_402;
    const wp::int32 var_403 = 0;
    bool var_404;
    wp::int32 var_405;
    const wp::int32 var_406 = 0;
    bool var_407;
    wp::int32 var_408;
    wp::int32 var_409;
    wp::int32 var_410;
    const wp::int32 var_411 = 3;
    wp::int32 var_412;
    const wp::int32 var_413 = 3;
    wp::int32 var_414;
    const wp::int32 var_415 = 1;
    bool var_416;
    wp::int32 var_417;
    wp::int32 var_418;
    const wp::int32 var_419 = 1;
    wp::int32 var_420;
    wp::int32 var_421;
    wp::int32 var_422;
    wp::int32 var_423;
    const wp::int32 var_424 = 12;
    const wp::int32 var_425 = 3;
    wp::int32 var_426;
    wp::int32 var_427;
    wp::int32 var_428;
    wp::vec_t<3, wp::float32> var_429;
    const wp::int32 var_430 = 0;
    bool var_431;
    wp::float32 var_432;
    wp::int32 var_433;
    wp::vec_t<3, wp::float32> var_434;
    bool var_435;
    wp::int32 var_436;
    wp::int32 var_437;
    wp::int32 var_438;
    const wp::int32 var_439 = 1;
    const wp::int32 var_440 = 3;
    wp::range_t var_441;
    wp::int32 var_442;
    const wp::int32 var_443 = 0;
    bool var_444;
    const wp::float32 var_445 = 0.0;
    const wp::int32 var_446 = 2;
    wp::float32 var_447;
    wp::float32 var_448;
    const wp::int32 var_449 = 1;
    wp::float32 var_450;
    wp::vec_t<3, wp::float32> var_451;
    wp::vec_t<3, wp::float32> var_452;
    const wp::int32 var_453 = 1;
    bool var_454;
    const wp::int32 var_455 = 2;
    wp::float32 var_456;
    const wp::float32 var_457 = 0.0;
    const wp::int32 var_458 = 0;
    wp::float32 var_459;
    wp::float32 var_460;
    wp::vec_t<3, wp::float32> var_461;
    wp::vec_t<3, wp::float32> var_462;
    const wp::int32 var_463 = 1;
    wp::float32 var_464;
    wp::float32 var_465;
    const wp::int32 var_466 = 0;
    wp::float32 var_467;
    const wp::float32 var_468 = 0.0;
    wp::vec_t<3, wp::float32> var_469;
    wp::vec_t<3, wp::float32> var_470;
    wp::vec_t<3, wp::float32> var_471;
    wp::float32 var_472;
    bool var_473;
    wp::vec_t<3, wp::float32> var_474;
    wp::float32 var_475;
    wp::vec_t<3, wp::float32> var_476;
    wp::float32 var_477;
    const wp::float32 var_478 = 0.0;
    wp::float32 var_479;
    const wp::int32 var_480 = 0;
    bool var_481;
    wp::float32 var_482;
    wp::float32 var_483;
    wp::float32 var_484;
    wp::float32 var_485;
    wp::float32 var_486;
    wp::float32 var_487;
    bool var_488;
    wp::float32 var_489;
    const wp::int32 var_490 = 3;
    wp::int32 var_491;
    wp::int32 var_492;
    wp::float32 var_493;
    wp::float32 var_494;
    wp::float32 var_495;
    wp::float32 var_496;
    wp::float32 var_497;
    const wp::int32 var_498 = 1;
    bool var_499;
    wp::float32 var_500;
    wp::float32 var_501;
    wp::float32 var_502;
    wp::float32 var_503;
    wp::float32 var_504;
    wp::float32 var_505;
    bool var_506;
    wp::float32 var_507;
    const wp::int32 var_508 = 3;
    wp::int32 var_509;
    wp::int32 var_510;
    wp::float32 var_511;
    wp::float32 var_512;
    wp::float32 var_513;
    wp::float32 var_514;
    wp::float32 var_515;
    const wp::int32 var_516 = 2;
    bool var_517;
    wp::float32 var_518;
    wp::float32 var_519;
    wp::float32 var_520;
    wp::float32 var_521;
    wp::float32 var_522;
    wp::float32 var_523;
    bool var_524;
    wp::float32 var_525;
    const wp::int32 var_526 = 3;
    wp::int32 var_527;
    wp::int32 var_528;
    wp::float32 var_529;
    wp::float32 var_530;
    wp::float32 var_531;
    wp::float32 var_532;
    wp::float32 var_533;
    wp::float32 var_534;
    wp::float32 var_535;
    wp::float32 var_536;
    bool var_537;
    const wp::float32 var_538 = 1.0;
    const wp::float32 var_539 = 1e-12;
    wp::float32 var_540;
    wp::float32 var_541;
    bool var_542;
    wp::float32 var_543;
    const wp::int32 var_544 = 0;
    const wp::int32 var_545 = 0;
    const wp::int32 var_546 = 0;
    bool var_547;
    bool var_548;
    wp::float32 var_549;
    const wp::int32 var_550 = 0;
    bool var_551;
    wp::int32 var_552;
    const wp::int32 var_553 = 0;
    bool var_554;
    wp::int32 var_555;
    wp::int32 var_556;
    const wp::int32 var_557 = 1;
    wp::int32 var_558;
    wp::int32 var_559;
    wp::int32 var_560;
    bool var_561;
    const wp::int32 var_562 = 3;
    wp::int32 var_563;
    wp::int32 var_564;
    wp::float32 var_565;
    const wp::int32 var_566 = 0;
    bool var_567;
    wp::int32 var_568;
    const wp::int32 var_569 = 0;
    bool var_570;
    wp::int32 var_571;
    wp::int32 var_572;
    wp::int32 var_573;
    const wp::int32 var_574 = 3;
    wp::int32 var_575;
    const wp::int32 var_576 = 3;
    wp::int32 var_577;
    const wp::int32 var_578 = 1;
    bool var_579;
    wp::int32 var_580;
    wp::int32 var_581;
    const wp::int32 var_582 = 1;
    wp::int32 var_583;
    wp::int32 var_584;
    wp::int32 var_585;
    wp::int32 var_586;
    const wp::int32 var_587 = 1;
    bool var_588;
    bool var_589;
    wp::float32 var_590;
    const wp::int32 var_591 = 0;
    bool var_592;
    wp::int32 var_593;
    const wp::int32 var_594 = 0;
    bool var_595;
    wp::int32 var_596;
    wp::int32 var_597;
    const wp::int32 var_598 = 1;
    wp::int32 var_599;
    wp::int32 var_600;
    wp::int32 var_601;
    bool var_602;
    const wp::int32 var_603 = 3;
    wp::int32 var_604;
    wp::int32 var_605;
    wp::float32 var_606;
    const wp::int32 var_607 = 0;
    bool var_608;
    wp::int32 var_609;
    const wp::int32 var_610 = 0;
    bool var_611;
    wp::int32 var_612;
    wp::int32 var_613;
    wp::int32 var_614;
    const wp::int32 var_615 = 3;
    wp::int32 var_616;
    const wp::int32 var_617 = 3;
    wp::int32 var_618;
    const wp::int32 var_619 = 1;
    bool var_620;
    wp::int32 var_621;
    wp::int32 var_622;
    const wp::int32 var_623 = 1;
    wp::int32 var_624;
    wp::int32 var_625;
    wp::int32 var_626;
    wp::int32 var_627;
    const wp::int32 var_628 = 2;
    bool var_629;
    bool var_630;
    wp::float32 var_631;
    const wp::int32 var_632 = 0;
    bool var_633;
    wp::int32 var_634;
    const wp::int32 var_635 = 0;
    bool var_636;
    wp::int32 var_637;
    wp::int32 var_638;
    const wp::int32 var_639 = 1;
    wp::int32 var_640;
    wp::int32 var_641;
    wp::int32 var_642;
    bool var_643;
    const wp::int32 var_644 = 3;
    wp::int32 var_645;
    wp::int32 var_646;
    wp::float32 var_647;
    const wp::int32 var_648 = 0;
    bool var_649;
    wp::int32 var_650;
    const wp::int32 var_651 = 0;
    bool var_652;
    wp::int32 var_653;
    wp::int32 var_654;
    wp::int32 var_655;
    const wp::int32 var_656 = 3;
    wp::int32 var_657;
    const wp::int32 var_658 = 3;
    wp::int32 var_659;
    const wp::int32 var_660 = 1;
    bool var_661;
    wp::int32 var_662;
    wp::int32 var_663;
    const wp::int32 var_664 = 1;
    wp::int32 var_665;
    wp::int32 var_666;
    wp::int32 var_667;
    wp::int32 var_668;
    const wp::int32 var_669 = 12;
    const wp::int32 var_670 = 3;
    wp::int32 var_671;
    wp::int32 var_672;
    wp::int32 var_673;
    wp::vec_t<3, wp::float32> var_674;
    const wp::int32 var_675 = 0;
    bool var_676;
    wp::float32 var_677;
    wp::int32 var_678;
    wp::vec_t<3, wp::float32> var_679;
    bool var_680;
    wp::int32 var_681;
    wp::int32 var_682;
    wp::int32 var_683;
    const wp::int32 var_684 = 2;
    const wp::int32 var_685 = 3;
    wp::range_t var_686;
    wp::int32 var_687;
    const wp::int32 var_688 = 0;
    bool var_689;
    const wp::float32 var_690 = 0.0;
    const wp::int32 var_691 = 2;
    wp::float32 var_692;
    wp::float32 var_693;
    const wp::int32 var_694 = 1;
    wp::float32 var_695;
    wp::vec_t<3, wp::float32> var_696;
    wp::vec_t<3, wp::float32> var_697;
    const wp::int32 var_698 = 1;
    bool var_699;
    const wp::int32 var_700 = 2;
    wp::float32 var_701;
    const wp::float32 var_702 = 0.0;
    const wp::int32 var_703 = 0;
    wp::float32 var_704;
    wp::float32 var_705;
    wp::vec_t<3, wp::float32> var_706;
    wp::vec_t<3, wp::float32> var_707;
    const wp::int32 var_708 = 1;
    wp::float32 var_709;
    wp::float32 var_710;
    const wp::int32 var_711 = 0;
    wp::float32 var_712;
    const wp::float32 var_713 = 0.0;
    wp::vec_t<3, wp::float32> var_714;
    wp::vec_t<3, wp::float32> var_715;
    wp::vec_t<3, wp::float32> var_716;
    wp::float32 var_717;
    bool var_718;
    wp::vec_t<3, wp::float32> var_719;
    wp::float32 var_720;
    wp::vec_t<3, wp::float32> var_721;
    wp::float32 var_722;
    const wp::float32 var_723 = 0.0;
    wp::float32 var_724;
    const wp::int32 var_725 = 0;
    bool var_726;
    wp::float32 var_727;
    wp::float32 var_728;
    wp::float32 var_729;
    wp::float32 var_730;
    wp::float32 var_731;
    wp::float32 var_732;
    bool var_733;
    wp::float32 var_734;
    const wp::int32 var_735 = 3;
    wp::int32 var_736;
    wp::int32 var_737;
    wp::float32 var_738;
    wp::float32 var_739;
    wp::float32 var_740;
    wp::float32 var_741;
    wp::float32 var_742;
    const wp::int32 var_743 = 1;
    bool var_744;
    wp::float32 var_745;
    wp::float32 var_746;
    wp::float32 var_747;
    wp::float32 var_748;
    wp::float32 var_749;
    wp::float32 var_750;
    bool var_751;
    wp::float32 var_752;
    const wp::int32 var_753 = 3;
    wp::int32 var_754;
    wp::int32 var_755;
    wp::float32 var_756;
    wp::float32 var_757;
    wp::float32 var_758;
    wp::float32 var_759;
    wp::float32 var_760;
    const wp::int32 var_761 = 2;
    bool var_762;
    wp::float32 var_763;
    wp::float32 var_764;
    wp::float32 var_765;
    wp::float32 var_766;
    wp::float32 var_767;
    wp::float32 var_768;
    bool var_769;
    wp::float32 var_770;
    const wp::int32 var_771 = 3;
    wp::int32 var_772;
    wp::int32 var_773;
    wp::float32 var_774;
    wp::float32 var_775;
    wp::float32 var_776;
    wp::float32 var_777;
    wp::float32 var_778;
    wp::float32 var_779;
    wp::float32 var_780;
    wp::float32 var_781;
    bool var_782;
    const wp::float32 var_783 = 1.0;
    const wp::float32 var_784 = 1e-12;
    wp::float32 var_785;
    wp::float32 var_786;
    bool var_787;
    wp::float32 var_788;
    const wp::int32 var_789 = 0;
    const wp::int32 var_790 = 0;
    const wp::int32 var_791 = 0;
    bool var_792;
    bool var_793;
    wp::float32 var_794;
    const wp::int32 var_795 = 0;
    bool var_796;
    wp::int32 var_797;
    const wp::int32 var_798 = 0;
    bool var_799;
    wp::int32 var_800;
    wp::int32 var_801;
    const wp::int32 var_802 = 1;
    wp::int32 var_803;
    wp::int32 var_804;
    wp::int32 var_805;
    bool var_806;
    const wp::int32 var_807 = 3;
    wp::int32 var_808;
    wp::int32 var_809;
    wp::float32 var_810;
    const wp::int32 var_811 = 0;
    bool var_812;
    wp::int32 var_813;
    const wp::int32 var_814 = 0;
    bool var_815;
    wp::int32 var_816;
    wp::int32 var_817;
    wp::int32 var_818;
    const wp::int32 var_819 = 3;
    wp::int32 var_820;
    const wp::int32 var_821 = 3;
    wp::int32 var_822;
    const wp::int32 var_823 = 1;
    bool var_824;
    wp::int32 var_825;
    wp::int32 var_826;
    const wp::int32 var_827 = 1;
    wp::int32 var_828;
    wp::int32 var_829;
    wp::int32 var_830;
    wp::int32 var_831;
    const wp::int32 var_832 = 1;
    bool var_833;
    bool var_834;
    wp::float32 var_835;
    const wp::int32 var_836 = 0;
    bool var_837;
    wp::int32 var_838;
    const wp::int32 var_839 = 0;
    bool var_840;
    wp::int32 var_841;
    wp::int32 var_842;
    const wp::int32 var_843 = 1;
    wp::int32 var_844;
    wp::int32 var_845;
    wp::int32 var_846;
    bool var_847;
    const wp::int32 var_848 = 3;
    wp::int32 var_849;
    wp::int32 var_850;
    wp::float32 var_851;
    const wp::int32 var_852 = 0;
    bool var_853;
    wp::int32 var_854;
    const wp::int32 var_855 = 0;
    bool var_856;
    wp::int32 var_857;
    wp::int32 var_858;
    wp::int32 var_859;
    const wp::int32 var_860 = 3;
    wp::int32 var_861;
    const wp::int32 var_862 = 3;
    wp::int32 var_863;
    const wp::int32 var_864 = 1;
    bool var_865;
    wp::int32 var_866;
    wp::int32 var_867;
    const wp::int32 var_868 = 1;
    wp::int32 var_869;
    wp::int32 var_870;
    wp::int32 var_871;
    wp::int32 var_872;
    const wp::int32 var_873 = 2;
    bool var_874;
    bool var_875;
    wp::float32 var_876;
    const wp::int32 var_877 = 0;
    bool var_878;
    wp::int32 var_879;
    const wp::int32 var_880 = 0;
    bool var_881;
    wp::int32 var_882;
    wp::int32 var_883;
    const wp::int32 var_884 = 1;
    wp::int32 var_885;
    wp::int32 var_886;
    wp::int32 var_887;
    bool var_888;
    const wp::int32 var_889 = 3;
    wp::int32 var_890;
    wp::int32 var_891;
    wp::float32 var_892;
    const wp::int32 var_893 = 0;
    bool var_894;
    wp::int32 var_895;
    const wp::int32 var_896 = 0;
    bool var_897;
    wp::int32 var_898;
    wp::int32 var_899;
    wp::int32 var_900;
    const wp::int32 var_901 = 3;
    wp::int32 var_902;
    const wp::int32 var_903 = 3;
    wp::int32 var_904;
    const wp::int32 var_905 = 1;
    bool var_906;
    wp::int32 var_907;
    wp::int32 var_908;
    const wp::int32 var_909 = 1;
    wp::int32 var_910;
    wp::int32 var_911;
    wp::int32 var_912;
    wp::int32 var_913;
    const wp::int32 var_914 = 12;
    const wp::int32 var_915 = 3;
    wp::int32 var_916;
    wp::int32 var_917;
    wp::int32 var_918;
    wp::vec_t<3, wp::float32> var_919;
    const wp::int32 var_920 = 0;
    bool var_921;
    wp::float32 var_922;
    wp::int32 var_923;
    wp::vec_t<3, wp::float32> var_924;
    bool var_925;
    wp::int32 var_926;
    wp::int32 var_927;
    wp::int32 var_928;
    const wp::int32 var_929 = -1;
    bool var_930;
    wp::mat_t<8, 3, wp::float32> var_931;
    wp::vec_t<8, wp::float32> var_932;
    const wp::int32 var_933 = 8;
    const wp::int32 var_934 = 12;
    bool var_935;
    const wp::int32 var_936 = 6;
    wp::int32 var_937;
    const wp::int32 var_938 = 6;
    wp::int32 var_939;
    wp::mat_t<3, 3, wp::float32> var_940;
    wp::mat_t<3, 3, wp::float32> var_941;
    wp::mat_t<3, 3, wp::float32> var_942;
    wp::vec_t<3, wp::float32> var_943;
    wp::vec_t<3, wp::float32> var_944;
    wp::vec_t<3, wp::float32> var_945;
    wp::vec_t<3, wp::float32> var_946;
    wp::vec_t<3, wp::float32> var_947;
    wp::vec_t<3, wp::float32> var_948;
    wp::mat_t<3, 3, wp::float32> var_949;
    const wp::int32 var_950 = 0;
    wp::float32 var_951;
    const wp::int32 var_952 = 1;
    wp::float32 var_953;
    const wp::int32 var_954 = 2;
    wp::float32 var_955;
    const wp::int32 var_956 = 2;
    const wp::int32 var_957 = 0;
    wp::int32 var_958;
    const wp::int32 var_959 = 0;
    const wp::int32 var_960 = 2;
    wp::float32 var_961;
    const wp::int32 var_962 = 0;
    bool var_963;
    const wp::int32 var_964 = 1;
    wp::int32 var_965;
    wp::int32 var_966;
    wp::int32 var_967;
    const wp::int32 var_968 = 1;
    const wp::int32 var_969 = 2;
    wp::float32 var_970;
    const wp::int32 var_971 = 0;
    bool var_972;
    const wp::int32 var_973 = 1;
    wp::int32 var_974;
    wp::int32 var_975;
    wp::int32 var_976;
    const wp::int32 var_977 = 2;
    const wp::int32 var_978 = 2;
    wp::float32 var_979;
    const wp::int32 var_980 = 0;
    bool var_981;
    const wp::int32 var_982 = 1;
    wp::int32 var_983;
    wp::int32 var_984;
    wp::int32 var_985;
    wp::vec_t<3, wp::float32> var_986;
    const wp::int32 var_987 = 0;
    wp::vec_t<3, wp::float32> var_988;
    wp::float32 var_989;
    wp::vec_t<3, wp::float32> var_990;
    const wp::int32 var_991 = 1;
    wp::int32 var_992;
    wp::int32 var_993;
    const wp::float32 var_994 = 1.0;
    const wp::float32 var_995 = -1.0;
    wp::float32 var_996;
    wp::vec_t<3, wp::float32> var_997;
    wp::vec_t<3, wp::float32> var_998;
    const wp::int32 var_999 = 1;
    wp::vec_t<3, wp::float32> var_1000;
    wp::float32 var_1001;
    wp::vec_t<3, wp::float32> var_1002;
    const wp::int32 var_1003 = 1;
    wp::int32 var_1004;
    wp::int32 var_1005;
    const wp::float32 var_1006 = 1.0;
    const wp::float32 var_1007 = -1.0;
    wp::float32 var_1008;
    wp::vec_t<3, wp::float32> var_1009;
    wp::vec_t<3, wp::float32> var_1010;
    const wp::int32 var_1011 = 2;
    wp::vec_t<3, wp::float32> var_1012;
    wp::float32 var_1013;
    wp::vec_t<3, wp::float32> var_1014;
    const wp::int32 var_1015 = 1;
    wp::int32 var_1016;
    wp::int32 var_1017;
    const wp::float32 var_1018 = 1.0;
    const wp::float32 var_1019 = -1.0;
    wp::float32 var_1020;
    wp::vec_t<3, wp::float32> var_1021;
    wp::vec_t<3, wp::float32> var_1022;
    const wp::int32 var_1023 = 1;
    wp::int32 var_1024;
    const wp::int32 var_1025 = 0;
    wp::int32 var_1026;
    const wp::float32 var_1027 = 0.0;
    wp::vec_t<3, wp::float32> var_1028;
    const wp::float32 var_1029 = 0.0;
    wp::vec_t<3, wp::float32> var_1030;
    const wp::int32 var_1031 = 0;
    const wp::int32 var_1032 = 2;
    wp::float32 var_1033;
    wp::float32 var_1034;
    const wp::float32 var_1035 = 0.5;
    bool var_1036;
    bool var_1037;
    wp::vec_t<3, wp::float32> var_1038;
    wp::float32 var_1039;
    wp::vec_t<3, wp::float32> var_1040;
    const wp::int32 var_1041 = 1;
    wp::int32 var_1042;
    wp::int32 var_1043;
    const wp::float32 var_1044 = -2.0;
    const wp::float32 var_1045 = 2.0;
    wp::float32 var_1046;
    wp::vec_t<3, wp::float32> var_1047;
    wp::vec_t<3, wp::float32> var_1048;
    wp::vec_t<3, wp::float32> var_1049;
    wp::float32 var_1050;
    wp::vec_t<3, wp::float32> var_1051;
    const wp::int32 var_1052 = 1;
    wp::int32 var_1053;
    wp::int32 var_1054;
    const wp::float32 var_1055 = -2.0;
    const wp::float32 var_1056 = 2.0;
    wp::float32 var_1057;
    wp::vec_t<3, wp::float32> var_1058;
    wp::vec_t<3, wp::float32> var_1059;
    const wp::int32 var_1060 = 1;
    wp::int32 var_1061;
    wp::int32 var_1062;
    wp::vec_t<3, wp::float32> var_1063;
    wp::vec_t<3, wp::float32> var_1064;
    const wp::int32 var_1065 = 1;
    const wp::int32 var_1066 = 2;
    wp::float32 var_1067;
    wp::float32 var_1068;
    const wp::float32 var_1069 = 0.5;
    bool var_1070;
    bool var_1071;
    wp::vec_t<3, wp::float32> var_1072;
    wp::float32 var_1073;
    wp::vec_t<3, wp::float32> var_1074;
    const wp::int32 var_1075 = 1;
    wp::int32 var_1076;
    wp::int32 var_1077;
    const wp::float32 var_1078 = -2.0;
    const wp::float32 var_1079 = 2.0;
    wp::float32 var_1080;
    wp::vec_t<3, wp::float32> var_1081;
    wp::vec_t<3, wp::float32> var_1082;
    wp::vec_t<3, wp::float32> var_1083;
    wp::float32 var_1084;
    wp::vec_t<3, wp::float32> var_1085;
    const wp::int32 var_1086 = 1;
    wp::int32 var_1087;
    wp::int32 var_1088;
    const wp::float32 var_1089 = -2.0;
    const wp::float32 var_1090 = 2.0;
    wp::float32 var_1091;
    wp::vec_t<3, wp::float32> var_1092;
    wp::vec_t<3, wp::float32> var_1093;
    const wp::int32 var_1094 = 1;
    wp::int32 var_1095;
    wp::int32 var_1096;
    wp::vec_t<3, wp::float32> var_1097;
    wp::vec_t<3, wp::float32> var_1098;
    const wp::int32 var_1099 = 2;
    const wp::int32 var_1100 = 2;
    wp::float32 var_1101;
    wp::float32 var_1102;
    const wp::float32 var_1103 = 0.5;
    bool var_1104;
    bool var_1105;
    wp::vec_t<3, wp::float32> var_1106;
    wp::float32 var_1107;
    wp::vec_t<3, wp::float32> var_1108;
    const wp::int32 var_1109 = 1;
    wp::int32 var_1110;
    wp::int32 var_1111;
    const wp::float32 var_1112 = -2.0;
    const wp::float32 var_1113 = 2.0;
    wp::float32 var_1114;
    wp::vec_t<3, wp::float32> var_1115;
    wp::vec_t<3, wp::float32> var_1116;
    wp::vec_t<3, wp::float32> var_1117;
    wp::float32 var_1118;
    wp::vec_t<3, wp::float32> var_1119;
    const wp::int32 var_1120 = 1;
    wp::int32 var_1121;
    wp::int32 var_1122;
    const wp::float32 var_1123 = -2.0;
    const wp::float32 var_1124 = 2.0;
    wp::float32 var_1125;
    wp::vec_t<3, wp::float32> var_1126;
    wp::vec_t<3, wp::float32> var_1127;
    const wp::int32 var_1128 = 1;
    wp::int32 var_1129;
    wp::int32 var_1130;
    wp::vec_t<3, wp::float32> var_1131;
    wp::vec_t<3, wp::float32> var_1132;
    wp::int32 var_1133;
    const wp::int32 var_1134 = 0;
    wp::int32 var_1135;
    wp::range_t var_1136;
    wp::int32 var_1137;
    const wp::int32 var_1138 = 0;
    const wp::int32 var_1139 = 2;
    bool var_1140;
    const wp::float32 var_1141 = 0.0;
    wp::vec_t<3, wp::float32> var_1142;
    const wp::int32 var_1143 = 2;
    bool var_1144;
    wp::vec_t<3, wp::float32> var_1145;
    wp::vec_t<3, wp::float32> var_1146;
    wp::vec_t<3, wp::float32> var_1147;
    bool var_1148;
    const wp::int32 var_1149 = 0;
    bool var_1150;
    const wp::int32 var_1151 = 3;
    bool var_1152;
    wp::vec_t<3, wp::float32> var_1153;
    wp::float32 var_1154;
    wp::float32 var_1155;
    bool var_1156;
    const wp::float32 var_1157 = 1.0;
    wp::float32 var_1158;
    wp::float32 var_1159;
    const wp::int32 var_1160 = -1;
    const wp::int32 var_1161 = 2;
    const wp::int32 var_1162 = 2;
    wp::range_t var_1163;
    wp::int32 var_1164;
    wp::float32 var_1165;
    wp::float32 var_1166;
    wp::float32 var_1167;
    wp::float32 var_1168;
    wp::float32 var_1169;
    wp::float32 var_1170;
    bool var_1171;
    const wp::int32 var_1172 = 0;
    bool var_1173;
    const wp::int32 var_1174 = 1;
    bool var_1175;
    wp::float32 var_1176;
    const wp::int32 var_1177 = 1;
    wp::int32 var_1178;
    wp::float32 var_1179;
    const wp::int32 var_1180 = 1;
    wp::int32 var_1181;
    wp::float32 var_1182;
    wp::float32 var_1183;
    wp::float32 var_1184;
    wp::float32 var_1185;
    const wp::int32 var_1186 = 1;
    wp::int32 var_1187;
    wp::float32 var_1188;
    bool var_1189;
    wp::float32 var_1190;
    wp::float32 var_1191;
    wp::vec_t<3, wp::float32> var_1192;
    wp::vec_t<3, wp::float32> var_1193;
    const wp::int32 var_1194 = 1;
    wp::int32 var_1195;
    wp::int32 var_1196;
    const wp::int32 var_1197 = 1;
    const wp::int32 var_1198 = 2;
    bool var_1199;
    const wp::float32 var_1200 = 0.0;
    wp::vec_t<3, wp::float32> var_1201;
    const wp::int32 var_1202 = 2;
    bool var_1203;
    wp::vec_t<3, wp::float32> var_1204;
    wp::vec_t<3, wp::float32> var_1205;
    wp::vec_t<3, wp::float32> var_1206;
    bool var_1207;
    const wp::int32 var_1208 = 0;
    bool var_1209;
    const wp::int32 var_1210 = 3;
    bool var_1211;
    wp::vec_t<3, wp::float32> var_1212;
    wp::float32 var_1213;
    wp::float32 var_1214;
    bool var_1215;
    const wp::float32 var_1216 = 1.0;
    wp::float32 var_1217;
    wp::float32 var_1218;
    const wp::int32 var_1219 = -1;
    const wp::int32 var_1220 = 2;
    const wp::int32 var_1221 = 2;
    wp::range_t var_1222;
    wp::int32 var_1223;
    wp::float32 var_1224;
    wp::float32 var_1225;
    wp::float32 var_1226;
    wp::float32 var_1227;
    wp::float32 var_1228;
    wp::float32 var_1229;
    bool var_1230;
    const wp::int32 var_1231 = 0;
    bool var_1232;
    const wp::int32 var_1233 = 1;
    bool var_1234;
    wp::float32 var_1235;
    wp::float32 var_1236;
    const wp::int32 var_1237 = 1;
    wp::int32 var_1238;
    wp::float32 var_1239;
    const wp::int32 var_1240 = 1;
    wp::int32 var_1241;
    wp::float32 var_1242;
    wp::float32 var_1243;
    wp::float32 var_1244;
    wp::float32 var_1245;
    const wp::int32 var_1246 = 1;
    wp::int32 var_1247;
    wp::float32 var_1248;
    bool var_1249;
    wp::float32 var_1250;
    wp::float32 var_1251;
    wp::float32 var_1252;
    wp::vec_t<3, wp::float32> var_1253;
    wp::vec_t<3, wp::float32> var_1254;
    const wp::int32 var_1255 = 1;
    wp::int32 var_1256;
    wp::int32 var_1257;
    wp::float32 var_1258;
    const wp::int32 var_1259 = 2;
    bool var_1260;
    const wp::int32 var_1261 = 0;
    wp::float32 var_1262;
    const wp::int32 var_1263 = 0;
    wp::float32 var_1264;
    const wp::int32 var_1265 = 1;
    wp::float32 var_1266;
    const wp::int32 var_1267 = 1;
    wp::float32 var_1268;
    const wp::float32 var_1269 = 1.0;
    wp::float32 var_1270;
    wp::float32 var_1271;
    wp::float32 var_1272;
    wp::float32 var_1273;
    const wp::int32 var_1274 = 0;
    const wp::int32 var_1275 = 2;
    wp::int32 var_1276;
    wp::float32 var_1277;
    wp::float32 var_1278;
    const wp::int32 var_1279 = 2;
    wp::int32 var_1280;
    wp::float32 var_1281;
    wp::float32 var_1282;
    const wp::int32 var_1283 = 0;
    wp::float32 var_1284;
    wp::float32 var_1285;
    const wp::int32 var_1286 = 1;
    wp::float32 var_1287;
    wp::float32 var_1288;
    wp::float32 var_1289;
    wp::float32 var_1290;
    wp::float32 var_1291;
    wp::float32 var_1292;
    wp::float32 var_1293;
    wp::float32 var_1294;
    wp::float32 var_1295;
    wp::float32 var_1296;
    bool var_1297;
    const wp::int32 var_1298 = 0;
    bool var_1299;
    const wp::int32 var_1300 = 0;
    bool var_1301;
    const wp::int32 var_1302 = 1;
    bool var_1303;
    const wp::int32 var_1304 = 1;
    bool var_1305;
    const wp::int32 var_1306 = 2;
    wp::float32 var_1307;
    const wp::int32 var_1308 = 2;
    wp::float32 var_1309;
    wp::float32 var_1310;
    wp::float32 var_1311;
    const wp::int32 var_1312 = 2;
    wp::float32 var_1313;
    wp::float32 var_1314;
    wp::float32 var_1315;
    wp::vec_t<3, wp::float32> var_1316;
    const wp::int32 var_1317 = 1;
    wp::int32 var_1318;
    wp::int32 var_1319;
    const wp::int32 var_1320 = 1;
    const wp::int32 var_1321 = 2;
    wp::int32 var_1322;
    wp::float32 var_1323;
    wp::float32 var_1324;
    const wp::int32 var_1325 = 2;
    wp::int32 var_1326;
    wp::float32 var_1327;
    wp::float32 var_1328;
    const wp::int32 var_1329 = 0;
    wp::float32 var_1330;
    wp::float32 var_1331;
    const wp::int32 var_1332 = 1;
    wp::float32 var_1333;
    wp::float32 var_1334;
    wp::float32 var_1335;
    wp::float32 var_1336;
    wp::float32 var_1337;
    wp::float32 var_1338;
    wp::float32 var_1339;
    wp::float32 var_1340;
    wp::float32 var_1341;
    wp::float32 var_1342;
    bool var_1343;
    const wp::int32 var_1344 = 0;
    bool var_1345;
    const wp::int32 var_1346 = 0;
    bool var_1347;
    const wp::int32 var_1348 = 1;
    bool var_1349;
    const wp::int32 var_1350 = 1;
    bool var_1351;
    const wp::int32 var_1352 = 2;
    wp::float32 var_1353;
    const wp::int32 var_1354 = 2;
    wp::float32 var_1355;
    wp::float32 var_1356;
    wp::float32 var_1357;
    const wp::int32 var_1358 = 2;
    wp::float32 var_1359;
    wp::float32 var_1360;
    wp::float32 var_1361;
    wp::vec_t<3, wp::float32> var_1362;
    const wp::int32 var_1363 = 1;
    wp::int32 var_1364;
    wp::int32 var_1365;
    const wp::int32 var_1366 = 2;
    const wp::int32 var_1367 = 2;
    wp::int32 var_1368;
    wp::float32 var_1369;
    wp::float32 var_1370;
    const wp::int32 var_1371 = 2;
    wp::int32 var_1372;
    wp::float32 var_1373;
    wp::float32 var_1374;
    const wp::int32 var_1375 = 0;
    wp::float32 var_1376;
    wp::float32 var_1377;
    const wp::int32 var_1378 = 1;
    wp::float32 var_1379;
    wp::float32 var_1380;
    wp::float32 var_1381;
    wp::float32 var_1382;
    wp::float32 var_1383;
    wp::float32 var_1384;
    wp::float32 var_1385;
    wp::float32 var_1386;
    wp::float32 var_1387;
    wp::float32 var_1388;
    bool var_1389;
    const wp::int32 var_1390 = 0;
    bool var_1391;
    const wp::int32 var_1392 = 0;
    bool var_1393;
    const wp::int32 var_1394 = 1;
    bool var_1395;
    const wp::int32 var_1396 = 1;
    bool var_1397;
    const wp::int32 var_1398 = 2;
    wp::float32 var_1399;
    const wp::int32 var_1400 = 2;
    wp::float32 var_1401;
    wp::float32 var_1402;
    wp::float32 var_1403;
    const wp::int32 var_1404 = 2;
    wp::float32 var_1405;
    wp::float32 var_1406;
    wp::float32 var_1407;
    wp::vec_t<3, wp::float32> var_1408;
    const wp::int32 var_1409 = 1;
    wp::int32 var_1410;
    wp::int32 var_1411;
    const wp::int32 var_1412 = 3;
    const wp::int32 var_1413 = 2;
    wp::int32 var_1414;
    wp::float32 var_1415;
    wp::float32 var_1416;
    const wp::int32 var_1417 = 2;
    wp::int32 var_1418;
    wp::float32 var_1419;
    wp::float32 var_1420;
    const wp::int32 var_1421 = 0;
    wp::float32 var_1422;
    wp::float32 var_1423;
    const wp::int32 var_1424 = 1;
    wp::float32 var_1425;
    wp::float32 var_1426;
    wp::float32 var_1427;
    wp::float32 var_1428;
    wp::float32 var_1429;
    wp::float32 var_1430;
    wp::float32 var_1431;
    wp::float32 var_1432;
    wp::float32 var_1433;
    wp::float32 var_1434;
    bool var_1435;
    const wp::int32 var_1436 = 0;
    bool var_1437;
    const wp::int32 var_1438 = 0;
    bool var_1439;
    const wp::int32 var_1440 = 1;
    bool var_1441;
    const wp::int32 var_1442 = 1;
    bool var_1443;
    const wp::int32 var_1444 = 2;
    wp::float32 var_1445;
    const wp::int32 var_1446 = 2;
    wp::float32 var_1447;
    wp::float32 var_1448;
    wp::float32 var_1449;
    const wp::int32 var_1450 = 2;
    wp::float32 var_1451;
    wp::float32 var_1452;
    wp::float32 var_1453;
    wp::vec_t<3, wp::float32> var_1454;
    const wp::int32 var_1455 = 1;
    wp::int32 var_1456;
    wp::int32 var_1457;
    wp::int32 var_1458;
    wp::int32 var_1459;
    const wp::int32 var_1460 = 1;
    wp::int32 var_1461;
    wp::range_t var_1462;
    wp::int32 var_1463;
    const wp::int32 var_1464 = 1;
    wp::int32 var_1465;
    wp::float32 var_1466;
    wp::vec_t<3, wp::float32> var_1467;
    wp::vec_t<3, wp::float32> var_1468;
    const wp::int32 var_1469 = 2;
    wp::int32 var_1470;
    const wp::int32 var_1471 = 0;
    bool var_1472;
    wp::float32 var_1473;
    wp::vec_t<3, wp::float32> var_1474;
    wp::vec_t<3, wp::float32> var_1475;
    bool var_1476;
    const wp::int32 var_1477 = 0;
    wp::float32 var_1478;
    wp::float32 var_1479;
    bool var_1480;
    const wp::int32 var_1481 = 0;
    wp::float32 var_1482;
    bool var_1483;
    const wp::int32 var_1484 = 1;
    wp::float32 var_1485;
    wp::float32 var_1486;
    bool var_1487;
    const wp::int32 var_1488 = 1;
    wp::float32 var_1489;
    bool var_1490;
    const wp::int32 var_1491 = 1;
    wp::int32 var_1492;
    wp::int32 var_1493;
    wp::int32 var_1494;
    const wp::int32 var_1495 = 0;
    wp::int32 var_1496;
    wp::range_t var_1497;
    wp::int32 var_1498;
    const wp::int32 var_1499 = 2;
    wp::float32 var_1500;
    bool var_1501;
    bool var_1502;
    wp::vec_t<3, wp::float32> var_1503;
    const wp::int32 var_1504 = 2;
    wp::float32 var_1505;
    const wp::float32 var_1506 = 0.5;
    const wp::int32 var_1507 = 2;
    wp::float32 var_1508;
    wp::float32 var_1509;
    const wp::int32 var_1510 = 1;
    wp::int32 var_1511;
    wp::mat_t<3, 3, wp::float32> var_1512;
    wp::mat_t<3, 3, wp::float32> var_1513;
    wp::mat_t<3, 3, wp::float32> var_1514;
    wp::vec_t<3, wp::float32> var_1515;
    const wp::float32 var_1516 = -1.0;
    const wp::float32 var_1517 = 1.0;
    wp::float32 var_1518;
    wp::mat_t<3, 3, wp::float32> var_1519;
    const wp::int32 var_1520 = 2;
    wp::vec_t<3, wp::float32> var_1521;
    wp::vec_t<3, wp::float32> var_1522;
    wp::int32 var_1523;
    wp::int32 var_1524;
    const wp::int32 var_1525 = 12;
    wp::int32 var_1526;
    const wp::int32 var_1527 = 3;
    wp::int32 var_1528;
    const wp::int32 var_1529 = 12;
    wp::int32 var_1530;
    const wp::int32 var_1531 = 3;
    wp::int32 var_1532;
    const wp::int32 var_1533 = 1;
    const wp::int32 var_1534 = 1;
    wp::int32 var_1535;
    wp::int32 var_1536;
    wp::int32 var_1537;
    const wp::int32 var_1538 = 2;
    const wp::int32 var_1539 = 2;
    wp::int32 var_1540;
    wp::int32 var_1541;
    wp::int32 var_1542;
    const wp::int32 var_1543 = 1;
    const wp::int32 var_1544 = 1;
    wp::int32 var_1545;
    wp::int32 var_1546;
    wp::int32 var_1547;
    const wp::int32 var_1548 = 2;
    const wp::int32 var_1549 = 2;
    wp::int32 var_1550;
    wp::int32 var_1551;
    wp::int32 var_1552;
    wp::float32 var_1553;
    wp::float32 var_1554;
    bool var_1555;
    wp::int32 var_1556;
    wp::int32 var_1557;
    wp::float32 var_1558;
    wp::float32 var_1559;
    bool var_1560;
    wp::int32 var_1561;
    wp::int32 var_1562;
    const wp::int32 var_1563 = 1;
    wp::int32 var_1564;
    wp::int32 var_1565;
    const wp::int32 var_1566 = 3;
    wp::int32 var_1567;
    wp::int32 var_1568;
    wp::mat_t<3, 3, wp::float32> var_1569;
    wp::vec_t<3, wp::float32> var_1570;
    wp::vec_t<3, wp::float32> var_1571;
    wp::mat_t<3, 3, wp::float32> var_1572;
    wp::mat_t<3, 3, wp::float32> var_1573;
    wp::mat_t<3, 3, wp::float32> var_1574;
    wp::vec_t<3, wp::float32> var_1575;
    wp::vec_t<3, wp::float32> var_1576;
    const wp::int32 var_1577 = 0;
    wp::float32 var_1578;
    const wp::int32 var_1579 = 1;
    wp::float32 var_1580;
    const wp::int32 var_1581 = 2;
    wp::float32 var_1582;
    const wp::int32 var_1583 = 2;
    wp::vec_t<3, wp::float32> var_1584;
    wp::float32 var_1585;
    wp::vec_t<3, wp::float32> var_1586;
    const wp::int32 var_1587 = 1;
    wp::int32 var_1588;
    wp::int32 var_1589;
    const wp::float32 var_1590 = 1.0;
    const wp::float32 var_1591 = -1.0;
    wp::float32 var_1592;
    wp::vec_t<3, wp::float32> var_1593;
    wp::vec_t<3, wp::float32> var_1594;
    wp::vec_t<3, wp::float32> var_1595;
    wp::float32 var_1596;
    wp::vec_t<3, wp::float32> var_1597;
    const wp::int32 var_1598 = 1;
    wp::int32 var_1599;
    wp::int32 var_1600;
    const wp::float32 var_1601 = 1.0;
    const wp::float32 var_1602 = -1.0;
    wp::float32 var_1603;
    wp::vec_t<3, wp::float32> var_1604;
    wp::vec_t<3, wp::float32> var_1605;
    const wp::int32 var_1606 = 0;
    const wp::int32 var_1607 = 0;
    wp::vec_t<3, wp::float32> var_1608;
    wp::vec_t<3, wp::float32> var_1609;
    wp::float32 var_1610;
    wp::vec_t<3, wp::float32> var_1611;
    wp::vec_t<3, wp::float32> var_1612;
    const wp::int32 var_1613 = 1;
    wp::vec_t<3, wp::float32> var_1614;
    wp::float32 var_1615;
    wp::vec_t<3, wp::float32> var_1616;
    const wp::int32 var_1617 = 0;
    wp::vec_t<3, wp::float32> var_1618;
    wp::float32 var_1619;
    wp::vec_t<3, wp::float32> var_1620;
    const wp::int32 var_1621 = 1;
    wp::int32 var_1622;
    wp::int32 var_1623;
    const wp::float32 var_1624 = -1.0;
    const wp::float32 var_1625 = 1.0;
    wp::float32 var_1626;
    wp::vec_t<3, wp::float32> var_1627;
    wp::vec_t<3, wp::float32> var_1628;
    wp::vec_t<3, wp::float32> var_1629;
    wp::float32 var_1630;
    wp::vec_t<3, wp::float32> var_1631;
    const wp::int32 var_1632 = 1;
    wp::int32 var_1633;
    wp::int32 var_1634;
    const wp::float32 var_1635 = 1.0;
    const wp::float32 var_1636 = -1.0;
    wp::float32 var_1637;
    wp::vec_t<3, wp::float32> var_1638;
    wp::vec_t<3, wp::float32> var_1639;
    const wp::int32 var_1640 = 2;
    const wp::int32 var_1641 = 2;
    wp::vec_t<3, wp::float32> var_1642;
    wp::vec_t<3, wp::float32> var_1643;
    wp::float32 var_1644;
    wp::vec_t<3, wp::float32> var_1645;
    wp::vec_t<3, wp::float32> var_1646;
    const wp::int32 var_1647 = 3;
    wp::vec_t<3, wp::float32> var_1648;
    wp::float32 var_1649;
    wp::vec_t<3, wp::float32> var_1650;
    const wp::int32 var_1651 = 2;
    const wp::int32 var_1652 = 4;
    const wp::int32 var_1653 = 0;
    wp::vec_t<3, wp::float32> var_1654;
    const wp::int32 var_1655 = 1;
    wp::vec_t<3, wp::float32> var_1656;
    const wp::int32 var_1657 = 0;
    wp::vec_t<3, wp::float32> var_1658;
    wp::vec_t<3, wp::float32> var_1659;
    const wp::int32 var_1660 = 2;
    wp::vec_t<3, wp::float32> var_1661;
    const wp::int32 var_1662 = 0;
    wp::vec_t<3, wp::float32> var_1663;
    wp::vec_t<3, wp::float32> var_1664;
    const wp::int32 var_1665 = 2;
    wp::float32 var_1666;
    wp::float32 var_1667;
    bool var_1668;
    const wp::float32 var_1669 = -1.0;
    const wp::float32 var_1670 = 1.0;
    wp::float32 var_1671;
    const wp::int32 var_1672 = 2;
    wp::float32 var_1673;
    wp::float32 var_1674;
    wp::mat_t<4, 3, wp::float32> var_1675;
    const wp::int32 var_1676 = 0;
    wp::vec_t<3, wp::float32> var_1677;
    const wp::int32 var_1678 = 2;
    wp::float32 var_1679;
    const wp::float32 var_1680 = -1.0;
    const wp::float32 var_1681 = 1.0;
    wp::float32 var_1682;
    wp::float32 var_1683;
    wp::float32 var_1684;
    wp::vec_t<3, wp::float32> var_1685;
    const wp::int32 var_1686 = 1;
    wp::vec_t<3, wp::float32> var_1687;
    const wp::int32 var_1688 = 2;
    wp::float32 var_1689;
    const wp::float32 var_1690 = -1.0;
    const wp::float32 var_1691 = 1.0;
    wp::float32 var_1692;
    wp::float32 var_1693;
    wp::float32 var_1694;
    wp::vec_t<3, wp::float32> var_1695;
    const wp::int32 var_1696 = 2;
    wp::vec_t<3, wp::float32> var_1697;
    const wp::int32 var_1698 = 2;
    wp::float32 var_1699;
    const wp::float32 var_1700 = -1.0;
    const wp::float32 var_1701 = 1.0;
    wp::float32 var_1702;
    wp::float32 var_1703;
    wp::float32 var_1704;
    wp::vec_t<3, wp::float32> var_1705;
    const wp::int32 var_1706 = 3;
    wp::vec_t<3, wp::float32> var_1707;
    const wp::int32 var_1708 = 2;
    wp::float32 var_1709;
    const wp::float32 var_1710 = -1.0;
    const wp::float32 var_1711 = 1.0;
    wp::float32 var_1712;
    wp::float32 var_1713;
    wp::float32 var_1714;
    wp::vec_t<3, wp::float32> var_1715;
    const wp::int32 var_1716 = 0;
    wp::vec_t<3, wp::float32> var_1717;
    const wp::int32 var_1718 = 1;
    wp::vec_t<3, wp::float32> var_1719;
    const wp::int32 var_1720 = 0;
    wp::vec_t<3, wp::float32> var_1721;
    wp::vec_t<3, wp::float32> var_1722;
    const wp::int32 var_1723 = 2;
    wp::vec_t<3, wp::float32> var_1724;
    const wp::int32 var_1725 = 0;
    wp::vec_t<3, wp::float32> var_1726;
    wp::vec_t<3, wp::float32> var_1727;
    const wp::int32 var_1728 = 0;
    wp::int32 var_1729;
    const wp::int32 var_1730 = 0;
    const wp::int32 var_1731 = 0;
    wp::float32 var_1732;
    const wp::int32 var_1733 = 2;
    bool var_1734;
    const wp::float32 var_1735 = 0.0;
    const wp::int32 var_1736 = 2;
    bool var_1737;
    wp::float32 var_1738;
    wp::float32 var_1739;
    wp::float32 var_1740;
    wp::float32 var_1741;
    wp::float32 var_1742;
    bool var_1743;
    const wp::int32 var_1744 = 0;
    bool var_1745;
    const wp::int32 var_1746 = 3;
    bool var_1747;
    wp::float32 var_1748;
    wp::float32 var_1749;
    wp::float32 var_1750;
    const wp::int32 var_1751 = 1;
    wp::int32 var_1752;
    wp::float32 var_1753;
    const wp::int32 var_1754 = 2;
    bool var_1755;
    const wp::float32 var_1756 = 0.0;
    const wp::int32 var_1757 = 2;
    bool var_1758;
    const wp::int32 var_1759 = 1;
    wp::int32 var_1760;
    wp::float32 var_1761;
    const wp::int32 var_1762 = 1;
    wp::int32 var_1763;
    wp::float32 var_1764;
    wp::float32 var_1765;
    wp::float32 var_1766;
    wp::float32 var_1767;
    bool var_1768;
    const wp::int32 var_1769 = 0;
    bool var_1770;
    const wp::int32 var_1771 = 3;
    bool var_1772;
    const wp::int32 var_1773 = 1;
    wp::int32 var_1774;
    wp::float32 var_1775;
    const wp::int32 var_1776 = 1;
    wp::int32 var_1777;
    wp::float32 var_1778;
    wp::float32 var_1779;
    const wp::int32 var_1780 = 2;
    bool var_1781;
    const wp::float32 var_1782 = 0.0;
    wp::vec_t<3, wp::float32> var_1783;
    const wp::int32 var_1784 = 2;
    bool var_1785;
    wp::vec_t<3, wp::float32> var_1786;
    wp::vec_t<3, wp::float32> var_1787;
    wp::vec_t<3, wp::float32> var_1788;
    bool var_1789;
    const wp::int32 var_1790 = 0;
    bool var_1791;
    const wp::int32 var_1792 = 3;
    bool var_1793;
    wp::vec_t<3, wp::float32> var_1794;
    wp::float32 var_1795;
    bool var_1796;
    const wp::float32 var_1797 = 1.0;
    wp::float32 var_1798;
    const wp::int32 var_1799 = -1;
    const wp::int32 var_1800 = 2;
    const wp::int32 var_1801 = 2;
    wp::range_t var_1802;
    wp::int32 var_1803;
    bool var_1804;
    wp::float32 var_1805;
    wp::float32 var_1806;
    wp::float32 var_1807;
    wp::float32 var_1808;
    wp::float32 var_1809;
    bool var_1810;
    const wp::int32 var_1811 = 0;
    bool var_1812;
    const wp::int32 var_1813 = 1;
    bool var_1814;
    wp::float32 var_1815;
    wp::float32 var_1816;
    wp::float32 var_1817;
    wp::float32 var_1818;
    wp::float32 var_1819;
    const wp::int32 var_1820 = 1;
    wp::int32 var_1821;
    wp::float32 var_1822;
    bool var_1823;
    wp::float32 var_1824;
    wp::float32 var_1825;
    wp::float32 var_1826;
    const wp::int32 var_1827 = 2;
    wp::float32 var_1828;
    const wp::int32 var_1829 = 2;
    wp::float32 var_1830;
    wp::float32 var_1831;
    wp::float32 var_1832;
    wp::float32 var_1833;
    bool var_1834;
    wp::float32 var_1835;
    wp::float32 var_1836;
    wp::float32 var_1837;
    const wp::float32 var_1838 = 0.5;
    wp::vec_t<3, wp::float32> var_1839;
    wp::vec_t<3, wp::float32> var_1840;
    const wp::float32 var_1841 = 0.5;
    wp::vec_t<3, wp::float32> var_1842;
    wp::vec_t<3, wp::float32> var_1843;
    const wp::float32 var_1844 = 0.5;
    wp::float32 var_1845;
    const wp::float32 var_1846 = 0.5;
    wp::float32 var_1847;
    const wp::int32 var_1848 = 1;
    wp::int32 var_1849;
    const wp::int32 var_1850 = 2;
    wp::float32 var_1851;
    wp::float32 var_1852;
    const wp::float32 var_1853 = 2.0;
    wp::float32 var_1854;
    const wp::int32 var_1855 = 1;
    wp::int32 var_1856;
    wp::int32 var_1857;
    wp::float32 var_1858;
    const wp::int32 var_1859 = 1;
    wp::float32 var_1860;
    const wp::int32 var_1861 = 2;
    bool var_1862;
    const wp::float32 var_1863 = 0.0;
    const wp::int32 var_1864 = 2;
    bool var_1865;
    wp::float32 var_1866;
    wp::float32 var_1867;
    wp::float32 var_1868;
    wp::float32 var_1869;
    wp::float32 var_1870;
    bool var_1871;
    const wp::int32 var_1872 = 0;
    bool var_1873;
    const wp::int32 var_1874 = 3;
    bool var_1875;
    wp::float32 var_1876;
    wp::float32 var_1877;
    wp::float32 var_1878;
    const wp::int32 var_1879 = 1;
    wp::int32 var_1880;
    wp::float32 var_1881;
    const wp::int32 var_1882 = 2;
    bool var_1883;
    const wp::float32 var_1884 = 0.0;
    const wp::int32 var_1885 = 2;
    bool var_1886;
    const wp::int32 var_1887 = 1;
    wp::int32 var_1888;
    wp::float32 var_1889;
    const wp::int32 var_1890 = 1;
    wp::int32 var_1891;
    wp::float32 var_1892;
    wp::float32 var_1893;
    wp::float32 var_1894;
    wp::float32 var_1895;
    bool var_1896;
    const wp::int32 var_1897 = 0;
    bool var_1898;
    const wp::int32 var_1899 = 3;
    bool var_1900;
    const wp::int32 var_1901 = 1;
    wp::int32 var_1902;
    wp::float32 var_1903;
    const wp::int32 var_1904 = 1;
    wp::int32 var_1905;
    wp::float32 var_1906;
    wp::float32 var_1907;
    const wp::int32 var_1908 = 2;
    bool var_1909;
    const wp::float32 var_1910 = 0.0;
    wp::vec_t<3, wp::float32> var_1911;
    const wp::int32 var_1912 = 2;
    bool var_1913;
    wp::vec_t<3, wp::float32> var_1914;
    wp::vec_t<3, wp::float32> var_1915;
    wp::vec_t<3, wp::float32> var_1916;
    bool var_1917;
    const wp::int32 var_1918 = 0;
    bool var_1919;
    const wp::int32 var_1920 = 3;
    bool var_1921;
    wp::vec_t<3, wp::float32> var_1922;
    wp::float32 var_1923;
    bool var_1924;
    const wp::float32 var_1925 = 1.0;
    wp::float32 var_1926;
    const wp::int32 var_1927 = -1;
    const wp::int32 var_1928 = 2;
    const wp::int32 var_1929 = 2;
    wp::range_t var_1930;
    wp::int32 var_1931;
    bool var_1932;
    wp::float32 var_1933;
    wp::float32 var_1934;
    wp::float32 var_1935;
    wp::float32 var_1936;
    wp::float32 var_1937;
    bool var_1938;
    const wp::int32 var_1939 = 0;
    bool var_1940;
    const wp::int32 var_1941 = 1;
    bool var_1942;
    wp::float32 var_1943;
    wp::float32 var_1944;
    wp::float32 var_1945;
    wp::float32 var_1946;
    wp::float32 var_1947;
    const wp::int32 var_1948 = 1;
    wp::int32 var_1949;
    wp::float32 var_1950;
    bool var_1951;
    wp::float32 var_1952;
    wp::float32 var_1953;
    wp::float32 var_1954;
    const wp::int32 var_1955 = 2;
    wp::float32 var_1956;
    const wp::int32 var_1957 = 2;
    wp::float32 var_1958;
    wp::float32 var_1959;
    wp::float32 var_1960;
    wp::float32 var_1961;
    bool var_1962;
    wp::float32 var_1963;
    wp::float32 var_1964;
    wp::float32 var_1965;
    const wp::float32 var_1966 = 0.5;
    wp::vec_t<3, wp::float32> var_1967;
    wp::vec_t<3, wp::float32> var_1968;
    const wp::float32 var_1969 = 0.5;
    wp::vec_t<3, wp::float32> var_1970;
    wp::vec_t<3, wp::float32> var_1971;
    const wp::float32 var_1972 = 0.5;
    wp::float32 var_1973;
    const wp::float32 var_1974 = 0.5;
    wp::float32 var_1975;
    const wp::int32 var_1976 = 1;
    wp::int32 var_1977;
    const wp::int32 var_1978 = 2;
    wp::float32 var_1979;
    wp::float32 var_1980;
    const wp::float32 var_1981 = 2.0;
    wp::float32 var_1982;
    const wp::int32 var_1983 = 1;
    wp::int32 var_1984;
    wp::int32 var_1985;
    wp::float32 var_1986;
    const wp::int32 var_1987 = 1;
    const wp::int32 var_1988 = 0;
    wp::float32 var_1989;
    const wp::int32 var_1990 = 2;
    bool var_1991;
    const wp::float32 var_1992 = 0.0;
    const wp::int32 var_1993 = 2;
    bool var_1994;
    wp::float32 var_1995;
    wp::float32 var_1996;
    wp::float32 var_1997;
    wp::float32 var_1998;
    wp::float32 var_1999;
    bool var_2000;
    const wp::int32 var_2001 = 0;
    bool var_2002;
    const wp::int32 var_2003 = 3;
    bool var_2004;
    wp::float32 var_2005;
    wp::float32 var_2006;
    wp::float32 var_2007;
    const wp::int32 var_2008 = 1;
    wp::int32 var_2009;
    wp::float32 var_2010;
    const wp::int32 var_2011 = 2;
    bool var_2012;
    const wp::float32 var_2013 = 0.0;
    const wp::int32 var_2014 = 2;
    bool var_2015;
    const wp::int32 var_2016 = 1;
    wp::int32 var_2017;
    wp::float32 var_2018;
    const wp::int32 var_2019 = 1;
    wp::int32 var_2020;
    wp::float32 var_2021;
    wp::float32 var_2022;
    wp::float32 var_2023;
    wp::float32 var_2024;
    bool var_2025;
    const wp::int32 var_2026 = 0;
    bool var_2027;
    const wp::int32 var_2028 = 3;
    bool var_2029;
    const wp::int32 var_2030 = 1;
    wp::int32 var_2031;
    wp::float32 var_2032;
    const wp::int32 var_2033 = 1;
    wp::int32 var_2034;
    wp::float32 var_2035;
    wp::float32 var_2036;
    const wp::int32 var_2037 = 2;
    bool var_2038;
    const wp::float32 var_2039 = 0.0;
    wp::vec_t<3, wp::float32> var_2040;
    const wp::int32 var_2041 = 2;
    bool var_2042;
    wp::vec_t<3, wp::float32> var_2043;
    wp::vec_t<3, wp::float32> var_2044;
    wp::vec_t<3, wp::float32> var_2045;
    bool var_2046;
    const wp::int32 var_2047 = 0;
    bool var_2048;
    const wp::int32 var_2049 = 3;
    bool var_2050;
    wp::vec_t<3, wp::float32> var_2051;
    wp::float32 var_2052;
    bool var_2053;
    const wp::float32 var_2054 = 1.0;
    wp::float32 var_2055;
    const wp::int32 var_2056 = -1;
    const wp::int32 var_2057 = 2;
    const wp::int32 var_2058 = 2;
    wp::range_t var_2059;
    wp::int32 var_2060;
    bool var_2061;
    wp::float32 var_2062;
    wp::float32 var_2063;
    wp::float32 var_2064;
    wp::float32 var_2065;
    wp::float32 var_2066;
    bool var_2067;
    const wp::int32 var_2068 = 0;
    bool var_2069;
    const wp::int32 var_2070 = 1;
    bool var_2071;
    wp::float32 var_2072;
    wp::float32 var_2073;
    wp::float32 var_2074;
    wp::float32 var_2075;
    wp::float32 var_2076;
    const wp::int32 var_2077 = 1;
    wp::int32 var_2078;
    wp::float32 var_2079;
    bool var_2080;
    wp::float32 var_2081;
    wp::float32 var_2082;
    wp::float32 var_2083;
    const wp::int32 var_2084 = 2;
    wp::float32 var_2085;
    const wp::int32 var_2086 = 2;
    wp::float32 var_2087;
    wp::float32 var_2088;
    wp::float32 var_2089;
    wp::float32 var_2090;
    bool var_2091;
    wp::float32 var_2092;
    wp::float32 var_2093;
    wp::float32 var_2094;
    const wp::float32 var_2095 = 0.5;
    wp::vec_t<3, wp::float32> var_2096;
    wp::vec_t<3, wp::float32> var_2097;
    const wp::float32 var_2098 = 0.5;
    wp::vec_t<3, wp::float32> var_2099;
    wp::vec_t<3, wp::float32> var_2100;
    const wp::float32 var_2101 = 0.5;
    wp::float32 var_2102;
    const wp::float32 var_2103 = 0.5;
    wp::float32 var_2104;
    const wp::int32 var_2105 = 1;
    wp::int32 var_2106;
    const wp::int32 var_2107 = 2;
    wp::float32 var_2108;
    wp::float32 var_2109;
    const wp::float32 var_2110 = 2.0;
    wp::float32 var_2111;
    const wp::int32 var_2112 = 1;
    wp::int32 var_2113;
    wp::int32 var_2114;
    wp::float32 var_2115;
    const wp::int32 var_2116 = 1;
    wp::float32 var_2117;
    const wp::int32 var_2118 = 2;
    bool var_2119;
    const wp::float32 var_2120 = 0.0;
    const wp::int32 var_2121 = 2;
    bool var_2122;
    wp::float32 var_2123;
    wp::float32 var_2124;
    wp::float32 var_2125;
    wp::float32 var_2126;
    wp::float32 var_2127;
    bool var_2128;
    const wp::int32 var_2129 = 0;
    bool var_2130;
    const wp::int32 var_2131 = 3;
    bool var_2132;
    wp::float32 var_2133;
    wp::float32 var_2134;
    wp::float32 var_2135;
    const wp::int32 var_2136 = 1;
    wp::int32 var_2137;
    wp::float32 var_2138;
    const wp::int32 var_2139 = 2;
    bool var_2140;
    const wp::float32 var_2141 = 0.0;
    const wp::int32 var_2142 = 2;
    bool var_2143;
    const wp::int32 var_2144 = 1;
    wp::int32 var_2145;
    wp::float32 var_2146;
    const wp::int32 var_2147 = 1;
    wp::int32 var_2148;
    wp::float32 var_2149;
    wp::float32 var_2150;
    wp::float32 var_2151;
    wp::float32 var_2152;
    bool var_2153;
    const wp::int32 var_2154 = 0;
    bool var_2155;
    const wp::int32 var_2156 = 3;
    bool var_2157;
    const wp::int32 var_2158 = 1;
    wp::int32 var_2159;
    wp::float32 var_2160;
    const wp::int32 var_2161 = 1;
    wp::int32 var_2162;
    wp::float32 var_2163;
    wp::float32 var_2164;
    const wp::int32 var_2165 = 2;
    bool var_2166;
    const wp::float32 var_2167 = 0.0;
    wp::vec_t<3, wp::float32> var_2168;
    const wp::int32 var_2169 = 2;
    bool var_2170;
    wp::vec_t<3, wp::float32> var_2171;
    wp::vec_t<3, wp::float32> var_2172;
    wp::vec_t<3, wp::float32> var_2173;
    bool var_2174;
    const wp::int32 var_2175 = 0;
    bool var_2176;
    const wp::int32 var_2177 = 3;
    bool var_2178;
    wp::vec_t<3, wp::float32> var_2179;
    wp::float32 var_2180;
    bool var_2181;
    const wp::float32 var_2182 = 1.0;
    wp::float32 var_2183;
    const wp::int32 var_2184 = -1;
    const wp::int32 var_2185 = 2;
    const wp::int32 var_2186 = 2;
    wp::range_t var_2187;
    wp::int32 var_2188;
    bool var_2189;
    wp::float32 var_2190;
    wp::float32 var_2191;
    wp::float32 var_2192;
    wp::float32 var_2193;
    wp::float32 var_2194;
    bool var_2195;
    const wp::int32 var_2196 = 0;
    bool var_2197;
    const wp::int32 var_2198 = 1;
    bool var_2199;
    wp::float32 var_2200;
    wp::float32 var_2201;
    wp::float32 var_2202;
    wp::float32 var_2203;
    wp::float32 var_2204;
    const wp::int32 var_2205 = 1;
    wp::int32 var_2206;
    wp::float32 var_2207;
    bool var_2208;
    wp::float32 var_2209;
    wp::float32 var_2210;
    wp::float32 var_2211;
    const wp::int32 var_2212 = 2;
    wp::float32 var_2213;
    const wp::int32 var_2214 = 2;
    wp::float32 var_2215;
    wp::float32 var_2216;
    wp::float32 var_2217;
    wp::float32 var_2218;
    bool var_2219;
    wp::float32 var_2220;
    wp::float32 var_2221;
    wp::float32 var_2222;
    const wp::float32 var_2223 = 0.5;
    wp::vec_t<3, wp::float32> var_2224;
    wp::vec_t<3, wp::float32> var_2225;
    const wp::float32 var_2226 = 0.5;
    wp::vec_t<3, wp::float32> var_2227;
    wp::vec_t<3, wp::float32> var_2228;
    const wp::float32 var_2229 = 0.5;
    wp::float32 var_2230;
    const wp::float32 var_2231 = 0.5;
    wp::float32 var_2232;
    const wp::int32 var_2233 = 1;
    wp::int32 var_2234;
    const wp::int32 var_2235 = 2;
    wp::float32 var_2236;
    wp::float32 var_2237;
    const wp::float32 var_2238 = 2.0;
    wp::float32 var_2239;
    const wp::int32 var_2240 = 1;
    wp::int32 var_2241;
    wp::int32 var_2242;
    wp::float32 var_2243;
    const wp::int32 var_2244 = 2;
    const wp::int32 var_2245 = 0;
    wp::float32 var_2246;
    const wp::int32 var_2247 = 2;
    bool var_2248;
    const wp::float32 var_2249 = 0.0;
    const wp::int32 var_2250 = 2;
    bool var_2251;
    wp::float32 var_2252;
    wp::float32 var_2253;
    wp::float32 var_2254;
    wp::float32 var_2255;
    wp::float32 var_2256;
    bool var_2257;
    const wp::int32 var_2258 = 0;
    bool var_2259;
    const wp::int32 var_2260 = 3;
    bool var_2261;
    wp::float32 var_2262;
    wp::float32 var_2263;
    wp::float32 var_2264;
    const wp::int32 var_2265 = 1;
    wp::int32 var_2266;
    wp::float32 var_2267;
    const wp::int32 var_2268 = 2;
    bool var_2269;
    const wp::float32 var_2270 = 0.0;
    const wp::int32 var_2271 = 2;
    bool var_2272;
    const wp::int32 var_2273 = 1;
    wp::int32 var_2274;
    wp::float32 var_2275;
    const wp::int32 var_2276 = 1;
    wp::int32 var_2277;
    wp::float32 var_2278;
    wp::float32 var_2279;
    wp::float32 var_2280;
    wp::float32 var_2281;
    bool var_2282;
    const wp::int32 var_2283 = 0;
    bool var_2284;
    const wp::int32 var_2285 = 3;
    bool var_2286;
    const wp::int32 var_2287 = 1;
    wp::int32 var_2288;
    wp::float32 var_2289;
    const wp::int32 var_2290 = 1;
    wp::int32 var_2291;
    wp::float32 var_2292;
    wp::float32 var_2293;
    const wp::int32 var_2294 = 2;
    bool var_2295;
    const wp::float32 var_2296 = 0.0;
    wp::vec_t<3, wp::float32> var_2297;
    const wp::int32 var_2298 = 2;
    bool var_2299;
    wp::vec_t<3, wp::float32> var_2300;
    wp::vec_t<3, wp::float32> var_2301;
    wp::vec_t<3, wp::float32> var_2302;
    bool var_2303;
    const wp::int32 var_2304 = 0;
    bool var_2305;
    const wp::int32 var_2306 = 3;
    bool var_2307;
    wp::vec_t<3, wp::float32> var_2308;
    wp::float32 var_2309;
    bool var_2310;
    const wp::float32 var_2311 = 1.0;
    wp::float32 var_2312;
    const wp::int32 var_2313 = -1;
    const wp::int32 var_2314 = 2;
    const wp::int32 var_2315 = 2;
    wp::range_t var_2316;
    wp::int32 var_2317;
    bool var_2318;
    wp::float32 var_2319;
    wp::float32 var_2320;
    wp::float32 var_2321;
    wp::float32 var_2322;
    wp::float32 var_2323;
    bool var_2324;
    const wp::int32 var_2325 = 0;
    bool var_2326;
    const wp::int32 var_2327 = 1;
    bool var_2328;
    wp::float32 var_2329;
    wp::float32 var_2330;
    wp::float32 var_2331;
    wp::float32 var_2332;
    wp::float32 var_2333;
    const wp::int32 var_2334 = 1;
    wp::int32 var_2335;
    wp::float32 var_2336;
    bool var_2337;
    wp::float32 var_2338;
    wp::float32 var_2339;
    wp::float32 var_2340;
    const wp::int32 var_2341 = 2;
    wp::float32 var_2342;
    const wp::int32 var_2343 = 2;
    wp::float32 var_2344;
    wp::float32 var_2345;
    wp::float32 var_2346;
    wp::float32 var_2347;
    bool var_2348;
    wp::float32 var_2349;
    wp::float32 var_2350;
    wp::float32 var_2351;
    const wp::float32 var_2352 = 0.5;
    wp::vec_t<3, wp::float32> var_2353;
    wp::vec_t<3, wp::float32> var_2354;
    const wp::float32 var_2355 = 0.5;
    wp::vec_t<3, wp::float32> var_2356;
    wp::vec_t<3, wp::float32> var_2357;
    const wp::float32 var_2358 = 0.5;
    wp::float32 var_2359;
    const wp::float32 var_2360 = 0.5;
    wp::float32 var_2361;
    const wp::int32 var_2362 = 1;
    wp::int32 var_2363;
    const wp::int32 var_2364 = 2;
    wp::float32 var_2365;
    wp::float32 var_2366;
    const wp::float32 var_2367 = 2.0;
    wp::float32 var_2368;
    const wp::int32 var_2369 = 1;
    wp::int32 var_2370;
    wp::int32 var_2371;
    wp::float32 var_2372;
    const wp::int32 var_2373 = 1;
    wp::float32 var_2374;
    const wp::int32 var_2375 = 2;
    bool var_2376;
    const wp::float32 var_2377 = 0.0;
    const wp::int32 var_2378 = 2;
    bool var_2379;
    wp::float32 var_2380;
    wp::float32 var_2381;
    wp::float32 var_2382;
    wp::float32 var_2383;
    wp::float32 var_2384;
    bool var_2385;
    const wp::int32 var_2386 = 0;
    bool var_2387;
    const wp::int32 var_2388 = 3;
    bool var_2389;
    wp::float32 var_2390;
    wp::float32 var_2391;
    wp::float32 var_2392;
    const wp::int32 var_2393 = 1;
    wp::int32 var_2394;
    wp::float32 var_2395;
    const wp::int32 var_2396 = 2;
    bool var_2397;
    const wp::float32 var_2398 = 0.0;
    const wp::int32 var_2399 = 2;
    bool var_2400;
    const wp::int32 var_2401 = 1;
    wp::int32 var_2402;
    wp::float32 var_2403;
    const wp::int32 var_2404 = 1;
    wp::int32 var_2405;
    wp::float32 var_2406;
    wp::float32 var_2407;
    wp::float32 var_2408;
    wp::float32 var_2409;
    bool var_2410;
    const wp::int32 var_2411 = 0;
    bool var_2412;
    const wp::int32 var_2413 = 3;
    bool var_2414;
    const wp::int32 var_2415 = 1;
    wp::int32 var_2416;
    wp::float32 var_2417;
    const wp::int32 var_2418 = 1;
    wp::int32 var_2419;
    wp::float32 var_2420;
    wp::float32 var_2421;
    const wp::int32 var_2422 = 2;
    bool var_2423;
    const wp::float32 var_2424 = 0.0;
    wp::vec_t<3, wp::float32> var_2425;
    const wp::int32 var_2426 = 2;
    bool var_2427;
    wp::vec_t<3, wp::float32> var_2428;
    wp::vec_t<3, wp::float32> var_2429;
    wp::vec_t<3, wp::float32> var_2430;
    bool var_2431;
    const wp::int32 var_2432 = 0;
    bool var_2433;
    const wp::int32 var_2434 = 3;
    bool var_2435;
    wp::vec_t<3, wp::float32> var_2436;
    wp::float32 var_2437;
    bool var_2438;
    const wp::float32 var_2439 = 1.0;
    wp::float32 var_2440;
    const wp::int32 var_2441 = -1;
    const wp::int32 var_2442 = 2;
    const wp::int32 var_2443 = 2;
    wp::range_t var_2444;
    wp::int32 var_2445;
    bool var_2446;
    wp::float32 var_2447;
    wp::float32 var_2448;
    wp::float32 var_2449;
    wp::float32 var_2450;
    wp::float32 var_2451;
    bool var_2452;
    const wp::int32 var_2453 = 0;
    bool var_2454;
    const wp::int32 var_2455 = 1;
    bool var_2456;
    wp::float32 var_2457;
    wp::float32 var_2458;
    wp::float32 var_2459;
    wp::float32 var_2460;
    wp::float32 var_2461;
    const wp::int32 var_2462 = 1;
    wp::int32 var_2463;
    wp::float32 var_2464;
    bool var_2465;
    wp::float32 var_2466;
    wp::float32 var_2467;
    wp::float32 var_2468;
    const wp::int32 var_2469 = 2;
    wp::float32 var_2470;
    const wp::int32 var_2471 = 2;
    wp::float32 var_2472;
    wp::float32 var_2473;
    wp::float32 var_2474;
    wp::float32 var_2475;
    bool var_2476;
    wp::float32 var_2477;
    wp::float32 var_2478;
    wp::float32 var_2479;
    const wp::float32 var_2480 = 0.5;
    wp::vec_t<3, wp::float32> var_2481;
    wp::vec_t<3, wp::float32> var_2482;
    const wp::float32 var_2483 = 0.5;
    wp::vec_t<3, wp::float32> var_2484;
    wp::vec_t<3, wp::float32> var_2485;
    const wp::float32 var_2486 = 0.5;
    wp::float32 var_2487;
    const wp::float32 var_2488 = 0.5;
    wp::float32 var_2489;
    const wp::int32 var_2490 = 1;
    wp::int32 var_2491;
    const wp::int32 var_2492 = 2;
    wp::float32 var_2493;
    wp::float32 var_2494;
    const wp::float32 var_2495 = 2.0;
    wp::float32 var_2496;
    const wp::int32 var_2497 = 1;
    wp::int32 var_2498;
    wp::int32 var_2499;
    wp::float32 var_2500;
    const wp::int32 var_2501 = 3;
    const wp::int32 var_2502 = 0;
    wp::float32 var_2503;
    const wp::int32 var_2504 = 2;
    bool var_2505;
    const wp::float32 var_2506 = 0.0;
    const wp::int32 var_2507 = 2;
    bool var_2508;
    wp::float32 var_2509;
    wp::float32 var_2510;
    wp::float32 var_2511;
    wp::float32 var_2512;
    wp::float32 var_2513;
    bool var_2514;
    const wp::int32 var_2515 = 0;
    bool var_2516;
    const wp::int32 var_2517 = 3;
    bool var_2518;
    wp::float32 var_2519;
    wp::float32 var_2520;
    wp::float32 var_2521;
    const wp::int32 var_2522 = 1;
    wp::int32 var_2523;
    wp::float32 var_2524;
    const wp::int32 var_2525 = 2;
    bool var_2526;
    const wp::float32 var_2527 = 0.0;
    const wp::int32 var_2528 = 2;
    bool var_2529;
    const wp::int32 var_2530 = 1;
    wp::int32 var_2531;
    wp::float32 var_2532;
    const wp::int32 var_2533 = 1;
    wp::int32 var_2534;
    wp::float32 var_2535;
    wp::float32 var_2536;
    wp::float32 var_2537;
    wp::float32 var_2538;
    bool var_2539;
    const wp::int32 var_2540 = 0;
    bool var_2541;
    const wp::int32 var_2542 = 3;
    bool var_2543;
    const wp::int32 var_2544 = 1;
    wp::int32 var_2545;
    wp::float32 var_2546;
    const wp::int32 var_2547 = 1;
    wp::int32 var_2548;
    wp::float32 var_2549;
    wp::float32 var_2550;
    const wp::int32 var_2551 = 2;
    bool var_2552;
    const wp::float32 var_2553 = 0.0;
    wp::vec_t<3, wp::float32> var_2554;
    const wp::int32 var_2555 = 2;
    bool var_2556;
    wp::vec_t<3, wp::float32> var_2557;
    wp::vec_t<3, wp::float32> var_2558;
    wp::vec_t<3, wp::float32> var_2559;
    bool var_2560;
    const wp::int32 var_2561 = 0;
    bool var_2562;
    const wp::int32 var_2563 = 3;
    bool var_2564;
    wp::vec_t<3, wp::float32> var_2565;
    wp::float32 var_2566;
    bool var_2567;
    const wp::float32 var_2568 = 1.0;
    wp::float32 var_2569;
    const wp::int32 var_2570 = -1;
    const wp::int32 var_2571 = 2;
    const wp::int32 var_2572 = 2;
    wp::range_t var_2573;
    wp::int32 var_2574;
    bool var_2575;
    wp::float32 var_2576;
    wp::float32 var_2577;
    wp::float32 var_2578;
    wp::float32 var_2579;
    wp::float32 var_2580;
    bool var_2581;
    const wp::int32 var_2582 = 0;
    bool var_2583;
    const wp::int32 var_2584 = 1;
    bool var_2585;
    wp::float32 var_2586;
    wp::float32 var_2587;
    wp::float32 var_2588;
    wp::float32 var_2589;
    wp::float32 var_2590;
    const wp::int32 var_2591 = 1;
    wp::int32 var_2592;
    wp::float32 var_2593;
    bool var_2594;
    wp::float32 var_2595;
    wp::float32 var_2596;
    wp::float32 var_2597;
    const wp::int32 var_2598 = 2;
    wp::float32 var_2599;
    const wp::int32 var_2600 = 2;
    wp::float32 var_2601;
    wp::float32 var_2602;
    wp::float32 var_2603;
    wp::float32 var_2604;
    bool var_2605;
    wp::float32 var_2606;
    wp::float32 var_2607;
    wp::float32 var_2608;
    const wp::float32 var_2609 = 0.5;
    wp::vec_t<3, wp::float32> var_2610;
    wp::vec_t<3, wp::float32> var_2611;
    const wp::float32 var_2612 = 0.5;
    wp::vec_t<3, wp::float32> var_2613;
    wp::vec_t<3, wp::float32> var_2614;
    const wp::float32 var_2615 = 0.5;
    wp::float32 var_2616;
    const wp::float32 var_2617 = 0.5;
    wp::float32 var_2618;
    const wp::int32 var_2619 = 1;
    wp::int32 var_2620;
    const wp::int32 var_2621 = 2;
    wp::float32 var_2622;
    wp::float32 var_2623;
    const wp::float32 var_2624 = 2.0;
    wp::float32 var_2625;
    const wp::int32 var_2626 = 1;
    wp::int32 var_2627;
    wp::int32 var_2628;
    wp::float32 var_2629;
    const wp::int32 var_2630 = 1;
    wp::float32 var_2631;
    const wp::int32 var_2632 = 2;
    bool var_2633;
    const wp::float32 var_2634 = 0.0;
    const wp::int32 var_2635 = 2;
    bool var_2636;
    wp::float32 var_2637;
    wp::float32 var_2638;
    wp::float32 var_2639;
    wp::float32 var_2640;
    wp::float32 var_2641;
    bool var_2642;
    const wp::int32 var_2643 = 0;
    bool var_2644;
    const wp::int32 var_2645 = 3;
    bool var_2646;
    wp::float32 var_2647;
    wp::float32 var_2648;
    wp::float32 var_2649;
    const wp::int32 var_2650 = 1;
    wp::int32 var_2651;
    wp::float32 var_2652;
    const wp::int32 var_2653 = 2;
    bool var_2654;
    const wp::float32 var_2655 = 0.0;
    const wp::int32 var_2656 = 2;
    bool var_2657;
    const wp::int32 var_2658 = 1;
    wp::int32 var_2659;
    wp::float32 var_2660;
    const wp::int32 var_2661 = 1;
    wp::int32 var_2662;
    wp::float32 var_2663;
    wp::float32 var_2664;
    wp::float32 var_2665;
    wp::float32 var_2666;
    bool var_2667;
    const wp::int32 var_2668 = 0;
    bool var_2669;
    const wp::int32 var_2670 = 3;
    bool var_2671;
    const wp::int32 var_2672 = 1;
    wp::int32 var_2673;
    wp::float32 var_2674;
    const wp::int32 var_2675 = 1;
    wp::int32 var_2676;
    wp::float32 var_2677;
    wp::float32 var_2678;
    const wp::int32 var_2679 = 2;
    bool var_2680;
    const wp::float32 var_2681 = 0.0;
    wp::vec_t<3, wp::float32> var_2682;
    const wp::int32 var_2683 = 2;
    bool var_2684;
    wp::vec_t<3, wp::float32> var_2685;
    wp::vec_t<3, wp::float32> var_2686;
    wp::vec_t<3, wp::float32> var_2687;
    bool var_2688;
    const wp::int32 var_2689 = 0;
    bool var_2690;
    const wp::int32 var_2691 = 3;
    bool var_2692;
    wp::vec_t<3, wp::float32> var_2693;
    wp::float32 var_2694;
    bool var_2695;
    const wp::float32 var_2696 = 1.0;
    wp::float32 var_2697;
    const wp::int32 var_2698 = -1;
    const wp::int32 var_2699 = 2;
    const wp::int32 var_2700 = 2;
    wp::range_t var_2701;
    wp::int32 var_2702;
    bool var_2703;
    wp::float32 var_2704;
    wp::float32 var_2705;
    wp::float32 var_2706;
    wp::float32 var_2707;
    wp::float32 var_2708;
    bool var_2709;
    const wp::int32 var_2710 = 0;
    bool var_2711;
    const wp::int32 var_2712 = 1;
    bool var_2713;
    wp::float32 var_2714;
    wp::float32 var_2715;
    wp::float32 var_2716;
    wp::float32 var_2717;
    wp::float32 var_2718;
    const wp::int32 var_2719 = 1;
    wp::int32 var_2720;
    wp::float32 var_2721;
    bool var_2722;
    wp::float32 var_2723;
    wp::float32 var_2724;
    wp::float32 var_2725;
    const wp::int32 var_2726 = 2;
    wp::float32 var_2727;
    const wp::int32 var_2728 = 2;
    wp::float32 var_2729;
    wp::float32 var_2730;
    wp::float32 var_2731;
    wp::float32 var_2732;
    bool var_2733;
    wp::float32 var_2734;
    wp::float32 var_2735;
    wp::float32 var_2736;
    const wp::float32 var_2737 = 0.5;
    wp::vec_t<3, wp::float32> var_2738;
    wp::vec_t<3, wp::float32> var_2739;
    const wp::float32 var_2740 = 0.5;
    wp::vec_t<3, wp::float32> var_2741;
    wp::vec_t<3, wp::float32> var_2742;
    const wp::float32 var_2743 = 0.5;
    wp::float32 var_2744;
    const wp::float32 var_2745 = 0.5;
    wp::float32 var_2746;
    const wp::int32 var_2747 = 1;
    wp::int32 var_2748;
    const wp::int32 var_2749 = 2;
    wp::float32 var_2750;
    wp::float32 var_2751;
    const wp::float32 var_2752 = 2.0;
    wp::float32 var_2753;
    const wp::int32 var_2754 = 1;
    wp::int32 var_2755;
    wp::int32 var_2756;
    wp::float32 var_2757;
    wp::int32 var_2758;
    const wp::int32 var_2759 = 0;
    wp::float32 var_2760;
    const wp::int32 var_2761 = 0;
    wp::float32 var_2762;
    const wp::int32 var_2763 = 1;
    wp::float32 var_2764;
    const wp::int32 var_2765 = 1;
    wp::float32 var_2766;
    const wp::float32 var_2767 = 1.0;
    wp::float32 var_2768;
    wp::float32 var_2769;
    wp::float32 var_2770;
    wp::float32 var_2771;
    const wp::int32 var_2772 = 4;
    wp::range_t var_2773;
    wp::int32 var_2774;
    bool var_2775;
    const wp::int32 var_2776 = 2;
    wp::int32 var_2777;
    wp::float32 var_2778;
    wp::float32 var_2779;
    const wp::int32 var_2780 = 2;
    wp::int32 var_2781;
    wp::float32 var_2782;
    wp::float32 var_2783;
    const wp::int32 var_2784 = 0;
    wp::float32 var_2785;
    wp::float32 var_2786;
    const wp::int32 var_2787 = 1;
    wp::float32 var_2788;
    wp::float32 var_2789;
    wp::float32 var_2790;
    wp::float32 var_2791;
    wp::float32 var_2792;
    wp::float32 var_2793;
    wp::float32 var_2794;
    wp::float32 var_2795;
    wp::float32 var_2796;
    wp::float32 var_2797;
    const wp::int32 var_2798 = 0;
    bool var_2799;
    bool var_2800;
    bool var_2801;
    const wp::int32 var_2802 = 0;
    bool var_2803;
    const wp::int32 var_2804 = 1;
    bool var_2805;
    bool var_2806;
    const wp::int32 var_2807 = 0;
    bool var_2808;
    const wp::int32 var_2809 = 1;
    bool var_2810;
    wp::float32 var_2811;
    wp::float32 var_2812;
    wp::float32 var_2813;
    wp::float32 var_2814;
    wp::float32 var_2815;
    wp::float32 var_2816;
    wp::float32 var_2817;
    wp::float32 var_2818;
    wp::float32 var_2819;
    wp::float32 var_2820;
    wp::float32 var_2821;
    wp::float32 var_2822;
    bool var_2823;
    const wp::int32 var_2824 = 0;
    bool var_2825;
    const wp::int32 var_2826 = 0;
    bool var_2827;
    const wp::int32 var_2828 = 1;
    bool var_2829;
    const wp::int32 var_2830 = 1;
    bool var_2831;
    wp::float32 var_2832;
    wp::float32 var_2833;
    wp::float32 var_2834;
    wp::float32 var_2835;
    wp::float32 var_2836;
    wp::float32 var_2837;
    wp::float32 var_2838;
    wp::float32 var_2839;
    wp::float32 var_2840;
    wp::float32 var_2841;
    wp::float32 var_2842;
    wp::float32 var_2843;
    const wp::float32 var_2844 = 0.0;
    const wp::float32 var_2845 = 1.0;
    wp::float32 var_2846;
    const wp::float32 var_2847 = 0.0;
    const wp::float32 var_2848 = 1.0;
    wp::float32 var_2849;
    const wp::float32 var_2850 = 1.0;
    wp::float32 var_2851;
    wp::float32 var_2852;
    const wp::int32 var_2853 = 0;
    wp::vec_t<3, wp::float32> var_2854;
    wp::vec_t<3, wp::float32> var_2855;
    const wp::int32 var_2856 = 1;
    wp::vec_t<3, wp::float32> var_2857;
    wp::vec_t<3, wp::float32> var_2858;
    wp::vec_t<3, wp::float32> var_2859;
    const wp::int32 var_2860 = 2;
    wp::vec_t<3, wp::float32> var_2861;
    wp::vec_t<3, wp::float32> var_2862;
    wp::vec_t<3, wp::float32> var_2863;
    const wp::float32 var_2864 = 0.0;
    wp::vec_t<3, wp::float32> var_2865;
    wp::vec_t<3, wp::float32> var_2866;
    wp::vec_t<3, wp::float32> var_2867;
    wp::float32 var_2868;
    bool var_2869;
    const wp::int32 var_2870 = 2;
    wp::float32 var_2871;
    const wp::int32 var_2872 = 0;
    bool var_2873;
    wp::float32 var_2874;
    bool var_2875;
    wp::float32 var_2876;
    wp::float32 var_2877;
    wp::float32 var_2878;
    wp::float32 var_2879;
    wp::float32 var_2880;
    wp::float32 var_2881;
    const wp::float32 var_2882 = 0.5;
    wp::vec_t<3, wp::float32> var_2883;
    wp::vec_t<3, wp::float32> var_2884;
    wp::vec_t<3, wp::float32> var_2885;
    wp::float32 var_2886;
    const wp::int32 var_2887 = 2;
    wp::float32 var_2888;
    const wp::int32 var_2889 = 0;
    bool var_2890;
    const wp::float32 var_2891 = -1.0;
    const wp::float32 var_2892 = 1.0;
    wp::float32 var_2893;
    wp::float32 var_2894;
    const wp::int32 var_2895 = 1;
    wp::int32 var_2896;
    wp::int32 var_2897;
    const wp::int32 var_2898 = 4;
    wp::range_t var_2899;
    wp::int32 var_2900;
    bool var_2901;
    const wp::int32 var_2902 = 0;
    wp::float32 var_2903;
    const wp::int32 var_2904 = 1;
    wp::float32 var_2905;
    bool var_2906;
    const wp::int32 var_2907 = 0;
    bool var_2908;
    const wp::int32 var_2909 = 0;
    bool var_2910;
    bool var_2911;
    bool var_2912;
    wp::float32 var_2913;
    bool var_2914;
    bool var_2915;
    bool var_2916;
    wp::float32 var_2917;
    bool var_2918;
    bool var_2919;
    wp::float32 var_2920;
    wp::float32 var_2921;
    wp::float32 var_2922;
    wp::float32 var_2923;
    bool var_2924;
    wp::float32 var_2925;
    bool var_2926;
    bool var_2927;
    wp::float32 var_2928;
    bool var_2929;
    bool var_2930;
    wp::float32 var_2931;
    wp::float32 var_2932;
    wp::float32 var_2933;
    wp::float32 var_2934;
    const wp::int32 var_2935 = 0;
    wp::float32 var_2936;
    const wp::int32 var_2937 = 0;
    wp::float32 var_2938;
    wp::float32 var_2939;
    wp::float32 var_2940;
    bool var_2941;
    wp::float32 var_2942;
    wp::float32 var_2943;
    wp::float32 var_2944;
    wp::float32 var_2945;
    wp::float32 var_2946;
    wp::float32 var_2947;
    wp::float32 var_2948;
    wp::float32 var_2949;
    wp::float32 var_2950;
    wp::float32 var_2951;
    wp::float32 var_2952;
    bool var_2953;
    wp::float32 var_2954;
    wp::float32 var_2955;
    wp::float32 var_2956;
    wp::float32 var_2957;
    wp::float32 var_2958;
    wp::float32 var_2959;
    wp::float32 var_2960;
    wp::float32 var_2961;
    wp::float32 var_2962;
    wp::float32 var_2963;
    const wp::int32 var_2964 = 1;
    wp::float32 var_2965;
    wp::float32 var_2966;
    wp::float32 var_2967;
    bool var_2968;
    wp::float32 var_2969;
    wp::float32 var_2970;
    wp::float32 var_2971;
    wp::float32 var_2972;
    wp::float32 var_2973;
    wp::float32 var_2974;
    wp::float32 var_2975;
    wp::float32 var_2976;
    wp::float32 var_2977;
    wp::float32 var_2978;
    wp::float32 var_2979;
    bool var_2980;
    wp::float32 var_2981;
    wp::float32 var_2982;
    wp::float32 var_2983;
    wp::float32 var_2984;
    wp::float32 var_2985;
    wp::float32 var_2986;
    wp::float32 var_2987;
    wp::float32 var_2988;
    wp::float32 var_2989;
    wp::float32 var_2990;
    const wp::int32 var_2991 = 2;
    wp::float32 var_2992;
    wp::float32 var_2993;
    const wp::int32 var_2994 = 2;
    wp::float32 var_2995;
    wp::float32 var_2996;
    wp::float32 var_2997;
    wp::float32 var_2998;
    bool var_2999;
    const wp::int32 var_3000 = 2;
    wp::float32 var_3001;
    const wp::int32 var_3002 = 0;
    bool var_3003;
    wp::float32 var_3004;
    bool var_3005;
    wp::float32 var_3006;
    wp::float32 var_3007;
    wp::float32 var_3008;
    const wp::int32 var_3009 = 0;
    wp::float32 var_3010;
    const wp::int32 var_3011 = 1;
    wp::float32 var_3012;
    const wp::float32 var_3013 = 0.0;
    wp::vec_t<3, wp::float32> var_3014;
    const wp::int32 var_3015 = 0;
    wp::float32 var_3016;
    wp::float32 var_3017;
    wp::float32 var_3018;
    bool var_3019;
    wp::float32 var_3020;
    wp::float32 var_3021;
    const wp::float32 var_3022 = 0.5;
    wp::float32 var_3023;
    wp::float32 var_3024;
    wp::float32 var_3025;
    bool var_3026;
    wp::float32 var_3027;
    wp::float32 var_3028;
    const wp::float32 var_3029 = 0.5;
    wp::float32 var_3030;
    const wp::int32 var_3031 = 1;
    wp::float32 var_3032;
    wp::float32 var_3033;
    wp::float32 var_3034;
    bool var_3035;
    wp::float32 var_3036;
    wp::float32 var_3037;
    const wp::float32 var_3038 = 0.5;
    wp::float32 var_3039;
    wp::float32 var_3040;
    wp::float32 var_3041;
    bool var_3042;
    wp::float32 var_3043;
    wp::float32 var_3044;
    const wp::float32 var_3045 = 0.5;
    wp::float32 var_3046;
    wp::vec_t<3, wp::float32> var_3047;
    wp::vec_t<3, wp::float32> var_3048;
    const wp::float32 var_3049 = 0.5;
    wp::vec_t<3, wp::float32> var_3050;
    wp::float32 var_3051;
    const wp::int32 var_3052 = 2;
    wp::float32 var_3053;
    const wp::int32 var_3054 = 0;
    bool var_3055;
    const wp::float32 var_3056 = -1.0;
    const wp::float32 var_3057 = 1.0;
    wp::float32 var_3058;
    wp::float32 var_3059;
    const wp::int32 var_3060 = 1;
    wp::int32 var_3061;
    wp::mat_t<3, 3, wp::float32> var_3062;
    wp::mat_t<3, 3, wp::float32> var_3063;
    wp::vec_t<3, wp::float32> var_3064;
    const wp::float32 var_3065 = -1.0;
    const wp::float32 var_3066 = 1.0;
    wp::float32 var_3067;
    wp::mat_t<3, 3, wp::float32> var_3068;
    wp::vec_t<3, wp::float32> var_3069;
    wp::int32 var_3070;
    wp::int32 var_3071;
    wp::mat_t<3, 3, wp::float32> var_3072;
    wp::mat_t<3, 3, wp::float32> var_3073;
    wp::vec_t<3, wp::float32> var_3074;
    wp::vec_t<3, wp::float32> var_3075;
    wp::mat_t<3, 3, wp::float32> var_3076;
    wp::float32 var_3077;
    wp::float32 var_3078;
    wp::float32 var_3079;
    wp::int32 var_3080;
    wp::int32 var_3081;
    wp::float32 var_3082;
    wp::float32 var_3083;
    wp::float32 var_3084;
    wp::float32 var_3085;
    wp::float32 var_3086;
    wp::float32 var_3087;
    wp::mat_t<3, 3, wp::float32> var_3088;
    wp::vec_t<3, wp::float32> var_3089;
    wp::vec_t<3, wp::float32> var_3090;
    wp::int32 var_3091;
    wp::range_t var_3092;
    wp::int32 var_3093;
    const wp::int32 var_3094 = 2;
    wp::vec_t<3, wp::float32> var_3095;
    wp::vec_t<3, wp::float32> var_3096;
    wp::vec_t<3, wp::float32> var_3097;
    wp::float32 var_3098;
    //---------
    // forward
    // def box_box(                                                                           <L 595>
    // contact_dist = vec8f()                                                                 <L 622>
    var_0 = wp::vec_t<8, wp::float32>();
    // for i in range(8):                                                                     <L 623>
    // contact_dist[i] = MJ_MAXVAL                                                            <L 624>
    wp::assign_inplace(var_0, var_1, var_2);
    wp::assign_inplace(var_0, var_3, var_2);
    wp::assign_inplace(var_0, var_4, var_2);
    wp::assign_inplace(var_0, var_5, var_2);
    wp::assign_inplace(var_0, var_6, var_2);
    wp::assign_inplace(var_0, var_7, var_2);
    wp::assign_inplace(var_0, var_8, var_2);
    wp::assign_inplace(var_0, var_9, var_2);
    // contact_pos = mat83f()                                                                 <L 625>
    var_10 = wp::mat_t<8, 3, wp::float32>();
    // contact_normals = mat83f()                                                             <L 626>
    var_11 = wp::mat_t<8, 3, wp::float32>();
    // contact_count = 0                                                                      <L 627>
    // pos21 = wp.transpose(box1_rot) @ (box2_pos - box1_pos)                                 <L 630>
    var_13 = wp::transpose(var_box1_rot);
    var_14 = wp::sub(var_box2_pos, var_box1_pos);
    var_15 = wp::mul(var_13, var_14);
    // pos12 = wp.transpose(box2_rot) @ (box1_pos - box2_pos)                                 <L 631>
    var_16 = wp::transpose(var_box2_rot);
    var_17 = wp::sub(var_box1_pos, var_box2_pos);
    var_18 = wp::mul(var_16, var_17);
    // rot21 = wp.transpose(box1_rot) @ box2_rot                                              <L 633>
    var_19 = wp::transpose(var_box1_rot);
    var_20 = wp::mul(var_19, var_box2_rot);
    // rot12 = wp.transpose(rot21)                                                            <L 634>
    var_21 = wp::transpose(var_20);
    // rot21abs = wp.matrix_from_rows(wp.abs(rot21[0]), wp.abs(rot21[1]), wp.abs(rot21[2]))       <L 636>
    var_23 = wp::extract(var_20, var_22);
    var_24 = wp::abs(var_23);
    var_26 = wp::extract(var_20, var_25);
    var_27 = wp::abs(var_26);
    var_29 = wp::extract(var_20, var_28);
    var_30 = wp::abs(var_29);
    var_31 = wp::matrix_from_rows<wp::float32>(var_24, var_27, var_30);
    // rot12abs = wp.transpose(rot21abs)                                                      <L 637>
    var_32 = wp::transpose(var_31);
    // plen2 = rot21abs @ box2_size                                                           <L 639>
    var_33 = wp::mul(var_31, var_box2_size);
    // plen1 = rot12abs @ box1_size                                                           <L 640>
    var_34 = wp::mul(var_32, var_box1_size);
    // s_sum_3 = 3.0 * (box1_size + box2_size)                                                <L 643>
    var_36 = wp::add(var_box1_size, var_box2_size);
    var_37 = wp::mul(var_35, var_36);
    // separation = wp.float32(margin + s_sum_3[0] + s_sum_3[1] + s_sum_3[2])                 <L 644>
    var_39 = wp::extract(var_37, var_38);
    var_40 = wp::add(var_margin, var_39);
    var_42 = wp::extract(var_37, var_41);
    var_43 = wp::add(var_40, var_42);
    var_45 = wp::extract(var_37, var_44);
    var_46 = wp::add(var_43, var_45);
    var_47 = wp::float32(var_46);
    // axis_code = wp.int32(-1)                                                               <L 645>
    var_49 = wp::int32(var_48);
    // for i in range(3):                                                                     <L 648>
    // c1 = -wp.abs(pos21[i]) + box1_size[i] + plen2[i]                                       <L 649>
    var_51 = wp::extract(var_15, var_50);
    var_52 = wp::abs(var_51);
    var_53 = wp::neg(var_52);
    var_54 = wp::extract(var_box1_size, var_50);
    var_55 = wp::add(var_53, var_54);
    var_56 = wp::extract(var_33, var_50);
    var_57 = wp::add(var_55, var_56);
    // c2 = -wp.abs(pos12[i]) + box2_size[i] + plen1[i]                                       <L 651>
    var_58 = wp::extract(var_18, var_50);
    var_59 = wp::abs(var_58);
    var_60 = wp::neg(var_59);
    var_61 = wp::extract(var_box2_size, var_50);
    var_62 = wp::add(var_60, var_61);
    var_63 = wp::extract(var_34, var_50);
    var_64 = wp::add(var_62, var_63);
    // if c1 < -margin or c2 < -margin:                                                       <L 653>
    var_66 = wp::neg(var_margin);
    var_67 = (var_57 < var_66);
    var_65 = var_67;
    if (!var_65) {
        var_68 = wp::neg(var_margin);
        var_69 = (var_64 < var_68);
        var_65 = var_65 || var_69;
    }
    if (var_65) {
        // return contact_dist, contact_pos, contact_normals                                  <L 654>
        ret_0 = var_0;
        ret_1 = var_10;
        ret_2 = var_11;
        return;
    }
    // if c1 < separation:                                                                    <L 656>
    var_70 = (var_57 < var_47);
    if (var_70) {
        // separation = c1                                                                    <L 657>
        var_71 = wp::copy(var_57);
        // axis_code = i + 3 * wp.int32(pos21[i] < 0) + 0  # Face of box1                     <L 658>
        var_73 = wp::extract(var_15, var_50);
        var_75 = (var_73 < var_74);
        var_76 = wp::int32(var_75);
        var_77 = wp::mul(var_72, var_76);
        var_78 = wp::add(var_50, var_77);
        var_80 = wp::add(var_78, var_79);
    }
    var_81 = wp::where(var_70, var_71, var_47);
    var_82 = wp::where(var_70, var_80, var_49);
    // if c2 < separation:                                                                    <L 659>
    var_83 = (var_64 < var_81);
    if (var_83) {
        // separation = c2                                                                    <L 660>
        var_84 = wp::copy(var_64);
        // axis_code = i + 3 * wp.int32(pos12[i] < 0) + 6  # Face of box2                     <L 661>
        var_86 = wp::extract(var_18, var_50);
        var_88 = (var_86 < var_87);
        var_89 = wp::int32(var_88);
        var_90 = wp::mul(var_85, var_89);
        var_91 = wp::add(var_50, var_90);
        var_93 = wp::add(var_91, var_92);
    }
    var_94 = wp::where(var_83, var_84, var_81);
    var_95 = wp::where(var_83, var_93, var_82);
    // c1 = -wp.abs(pos21[i]) + box1_size[i] + plen2[i]                                       <L 649>
    var_97 = wp::extract(var_15, var_96);
    var_98 = wp::abs(var_97);
    var_99 = wp::neg(var_98);
    var_100 = wp::extract(var_box1_size, var_96);
    var_101 = wp::add(var_99, var_100);
    var_102 = wp::extract(var_33, var_96);
    var_103 = wp::add(var_101, var_102);
    // c2 = -wp.abs(pos12[i]) + box2_size[i] + plen1[i]                                       <L 651>
    var_104 = wp::extract(var_18, var_96);
    var_105 = wp::abs(var_104);
    var_106 = wp::neg(var_105);
    var_107 = wp::extract(var_box2_size, var_96);
    var_108 = wp::add(var_106, var_107);
    var_109 = wp::extract(var_34, var_96);
    var_110 = wp::add(var_108, var_109);
    // if c1 < -margin or c2 < -margin:                                                       <L 653>
    var_112 = wp::neg(var_margin);
    var_113 = (var_103 < var_112);
    var_111 = var_113;
    if (!var_111) {
        var_114 = wp::neg(var_margin);
        var_115 = (var_110 < var_114);
        var_111 = var_111 || var_115;
    }
    if (var_111) {
        // return contact_dist, contact_pos, contact_normals                                  <L 654>
        ret_0 = var_0;
        ret_1 = var_10;
        ret_2 = var_11;
        return;
    }
    // if c1 < separation:                                                                    <L 656>
    var_116 = (var_103 < var_94);
    if (var_116) {
        // separation = c1                                                                    <L 657>
        var_117 = wp::copy(var_103);
        // axis_code = i + 3 * wp.int32(pos21[i] < 0) + 0  # Face of box1                     <L 658>
        var_119 = wp::extract(var_15, var_96);
        var_121 = (var_119 < var_120);
        var_122 = wp::int32(var_121);
        var_123 = wp::mul(var_118, var_122);
        var_124 = wp::add(var_96, var_123);
        var_126 = wp::add(var_124, var_125);
    }
    var_127 = wp::where(var_116, var_117, var_94);
    var_128 = wp::where(var_116, var_126, var_95);
    // if c2 < separation:                                                                    <L 659>
    var_129 = (var_110 < var_127);
    if (var_129) {
        // separation = c2                                                                    <L 660>
        var_130 = wp::copy(var_110);
        // axis_code = i + 3 * wp.int32(pos12[i] < 0) + 6  # Face of box2                     <L 661>
        var_132 = wp::extract(var_18, var_96);
        var_134 = (var_132 < var_133);
        var_135 = wp::int32(var_134);
        var_136 = wp::mul(var_131, var_135);
        var_137 = wp::add(var_96, var_136);
        var_139 = wp::add(var_137, var_138);
    }
    var_140 = wp::where(var_129, var_130, var_127);
    var_141 = wp::where(var_129, var_139, var_128);
    // c1 = -wp.abs(pos21[i]) + box1_size[i] + plen2[i]                                       <L 649>
    var_143 = wp::extract(var_15, var_142);
    var_144 = wp::abs(var_143);
    var_145 = wp::neg(var_144);
    var_146 = wp::extract(var_box1_size, var_142);
    var_147 = wp::add(var_145, var_146);
    var_148 = wp::extract(var_33, var_142);
    var_149 = wp::add(var_147, var_148);
    // c2 = -wp.abs(pos12[i]) + box2_size[i] + plen1[i]                                       <L 651>
    var_150 = wp::extract(var_18, var_142);
    var_151 = wp::abs(var_150);
    var_152 = wp::neg(var_151);
    var_153 = wp::extract(var_box2_size, var_142);
    var_154 = wp::add(var_152, var_153);
    var_155 = wp::extract(var_34, var_142);
    var_156 = wp::add(var_154, var_155);
    // if c1 < -margin or c2 < -margin:                                                       <L 653>
    var_158 = wp::neg(var_margin);
    var_159 = (var_149 < var_158);
    var_157 = var_159;
    if (!var_157) {
        var_160 = wp::neg(var_margin);
        var_161 = (var_156 < var_160);
        var_157 = var_157 || var_161;
    }
    if (var_157) {
        // return contact_dist, contact_pos, contact_normals                                  <L 654>
        ret_0 = var_0;
        ret_1 = var_10;
        ret_2 = var_11;
        return;
    }
    // if c1 < separation:                                                                    <L 656>
    var_162 = (var_149 < var_140);
    if (var_162) {
        // separation = c1                                                                    <L 657>
        var_163 = wp::copy(var_149);
        // axis_code = i + 3 * wp.int32(pos21[i] < 0) + 0  # Face of box1                     <L 658>
        var_165 = wp::extract(var_15, var_142);
        var_167 = (var_165 < var_166);
        var_168 = wp::int32(var_167);
        var_169 = wp::mul(var_164, var_168);
        var_170 = wp::add(var_142, var_169);
        var_172 = wp::add(var_170, var_171);
    }
    var_173 = wp::where(var_162, var_163, var_140);
    var_174 = wp::where(var_162, var_172, var_141);
    // if c2 < separation:                                                                    <L 659>
    var_175 = (var_156 < var_173);
    if (var_175) {
        // separation = c2                                                                    <L 660>
        var_176 = wp::copy(var_156);
        // axis_code = i + 3 * wp.int32(pos12[i] < 0) + 6  # Face of box2                     <L 661>
        var_178 = wp::extract(var_18, var_142);
        var_180 = (var_178 < var_179);
        var_181 = wp::int32(var_180);
        var_182 = wp::mul(var_177, var_181);
        var_183 = wp::add(var_142, var_182);
        var_185 = wp::add(var_183, var_184);
    }
    var_186 = wp::where(var_175, var_176, var_173);
    var_187 = wp::where(var_175, var_185, var_174);
    // clnorm = wp.vec3(0.0)                                                                  <L 663>
    var_189 = wp::vec_t<3, wp::float32>(var_188);
    // inv = wp.bool(False)                                                                   <L 664>
    var_191 = bool(var_190);
    // cle1 = wp.int32(0)                                                                     <L 665>
    var_193 = wp::int32(var_192);
    // cle2 = wp.int32(0)                                                                     <L 666>
    var_195 = wp::int32(var_194);
    // for i in range(3):                                                                     <L 669>
    // for j in range(3):                                                                     <L 670>
    var_198 = wp::range(var_197);
    start_for_3:;
        if (iter_cmp(var_198) == 0) goto end_for_3;
        var_199 = wp::iter_next(var_198);
        // if i == 0:                                                                         <L 672>
        var_201 = (var_196 == var_200);
        if (var_201) {
            // cross_axis = wp.vec3(0.0, -rot12[j, 2], rot12[j, 1])                           <L 673>
            var_204 = wp::extract(var_21, var_199, var_203);
            var_205 = wp::neg(var_204);
            var_207 = wp::extract(var_21, var_199, var_206);
            var_208 = wp::vec_t<3, wp::float32>(var_202, var_205, var_207);
        }
        if (!var_201) {
            // elif i == 1:                                                                   <L 674>
            var_210 = (var_196 == var_209);
            if (var_210) {
                // cross_axis = wp.vec3(rot12[j, 2], 0.0, -rot12[j, 0])                       <L 675>
                var_212 = wp::extract(var_21, var_199, var_211);
                var_215 = wp::extract(var_21, var_199, var_214);
                var_216 = wp::neg(var_215);
                var_217 = wp::vec_t<3, wp::float32>(var_212, var_213, var_216);
            }
            var_218 = wp::where(var_210, var_217, var_208);
            if (!var_210) {
                // cross_axis = wp.vec3(-rot12[j, 1], rot12[j, 0], 0.0)                       <L 677>
                var_220 = wp::extract(var_21, var_199, var_219);
                var_221 = wp::neg(var_220);
                var_223 = wp::extract(var_21, var_199, var_222);
                var_225 = wp::vec_t<3, wp::float32>(var_221, var_223, var_224);
            }
            var_226 = wp::where(var_210, var_218, var_225);
        }
        var_227 = wp::where(var_201, var_208, var_226);
        // cross_length = wp.length(cross_axis)                                               <L 679>
        var_228 = wp::length(var_227);
        // if cross_length < MJ_MINVAL:                                                       <L 680>
        var_230 = (var_228 < var_229);
        if (var_230) {
            // continue                                                                       <L 681>
            goto start_for_3;
        }
        // cross_axis /= cross_length                                                         <L 683>
        var_231 = wp::div(var_227, var_228);
        // box_dist = wp.dot(pos21, cross_axis)                                               <L 685>
        var_232 = wp::dot(var_15, var_231);
        // c3 = wp.float32(0.0)                                                               <L 686>
        var_234 = wp::float32(var_233);
        // for k in range(3):                                                                 <L 689>
        // if k != i:                                                                         <L 690>
        var_236 = (var_235 != var_196);
        if (var_236) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_237 = wp::extract(var_box1_size, var_235);
            var_238 = wp::extract(var_231, var_235);
            var_239 = wp::abs(var_238);
            var_240 = wp::mul(var_237, var_239);
            var_241 = wp::add(var_234, var_240);
        }
        var_242 = wp::where(var_236, var_241, var_234);
        // if k != j:                                                                         <L 692>
        var_243 = (var_235 != var_199);
        if (var_243) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_244 = wp::extract(var_box2_size, var_235);
            var_246 = wp::sub(var_245, var_235);
            var_247 = wp::sub(var_246, var_199);
            var_248 = wp::extract(var_31, var_196, var_247);
            var_249 = wp::mul(var_244, var_248);
            var_250 = wp::div(var_249, var_228);
            var_251 = wp::add(var_242, var_250);
        }
        var_252 = wp::where(var_243, var_251, var_242);
        // if k != i:                                                                         <L 690>
        var_254 = (var_253 != var_196);
        if (var_254) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_255 = wp::extract(var_box1_size, var_253);
            var_256 = wp::extract(var_231, var_253);
            var_257 = wp::abs(var_256);
            var_258 = wp::mul(var_255, var_257);
            var_259 = wp::add(var_252, var_258);
        }
        var_260 = wp::where(var_254, var_259, var_252);
        // if k != j:                                                                         <L 692>
        var_261 = (var_253 != var_199);
        if (var_261) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_262 = wp::extract(var_box2_size, var_253);
            var_264 = wp::sub(var_263, var_253);
            var_265 = wp::sub(var_264, var_199);
            var_266 = wp::extract(var_31, var_196, var_265);
            var_267 = wp::mul(var_262, var_266);
            var_268 = wp::div(var_267, var_228);
            var_269 = wp::add(var_260, var_268);
        }
        var_270 = wp::where(var_261, var_269, var_260);
        // if k != i:                                                                         <L 690>
        var_272 = (var_271 != var_196);
        if (var_272) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_273 = wp::extract(var_box1_size, var_271);
            var_274 = wp::extract(var_231, var_271);
            var_275 = wp::abs(var_274);
            var_276 = wp::mul(var_273, var_275);
            var_277 = wp::add(var_270, var_276);
        }
        var_278 = wp::where(var_272, var_277, var_270);
        // if k != j:                                                                         <L 692>
        var_279 = (var_271 != var_199);
        if (var_279) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_280 = wp::extract(var_box2_size, var_271);
            var_282 = wp::sub(var_281, var_271);
            var_283 = wp::sub(var_282, var_199);
            var_284 = wp::extract(var_31, var_196, var_283);
            var_285 = wp::mul(var_280, var_284);
            var_286 = wp::div(var_285, var_228);
            var_287 = wp::add(var_278, var_286);
        }
        var_288 = wp::where(var_279, var_287, var_278);
        // c3 -= wp.abs(box_dist)                                                             <L 695>
        var_289 = wp::abs(var_232);
        var_290 = wp::sub(var_288, var_289);
        // if c3 < -margin:                                                                   <L 698>
        var_291 = wp::neg(var_margin);
        var_292 = (var_290 < var_291);
        if (var_292) {
            // return contact_dist, contact_pos, contact_normals                              <L 699>
            ret_0 = var_0;
            ret_1 = var_10;
            ret_2 = var_11;
            return;
        }
        // if c3 < separation * (1.0 - 1e-12):                                                <L 702>
        var_295 = wp::sub(var_293, var_294);
        var_296 = wp::mul(var_186, var_295);
        var_297 = (var_290 < var_296);
        if (var_297) {
            // separation = c3                                                                <L 703>
            var_298 = wp::copy(var_290);
            // cle1 = 0                                                                       <L 705>
            // cle2 = 0                                                                       <L 706>
            // for k in range(3):                                                             <L 708>
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_303 = (var_301 != var_196);
            var_302 = var_303;
            if (var_302) {
                var_304 = wp::extract(var_231, var_301);
                var_306 = (var_304 > var_305);
                var_307 = wp::int(var_306);
                var_309 = (var_232 < var_308);
                var_310 = wp::int(var_309);
                var_311 = wp::bit_xor(var_307, var_310);
                var_302 = var_302 && var_311;
            }
            if (var_302) {
                // cle1 += 1 << k                                                             <L 710>
                var_313 = wp::lshift(var_312, var_301);
                var_314 = wp::add(var_299, var_313);
            }
            var_315 = wp::where(var_302, var_314, var_299);
            // if k != j:                                                                     <L 711>
            var_316 = (var_301 != var_199);
            if (var_316) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_318 = wp::sub(var_317, var_301);
                var_319 = wp::sub(var_318, var_199);
                var_320 = wp::extract(var_20, var_196, var_319);
                var_322 = (var_320 > var_321);
                var_323 = wp::int(var_322);
                var_325 = (var_232 < var_324);
                var_326 = wp::int(var_325);
                var_327 = wp::bit_xor(var_323, var_326);
                var_328 = wp::sub(var_301, var_199);
                var_330 = wp::add(var_328, var_329);
                var_332 = wp::mod(var_330, var_331);
                var_334 = (var_332 == var_333);
                var_335 = wp::int(var_334);
                var_336 = wp::bit_xor(var_327, var_335);
                if (var_336) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_338 = wp::lshift(var_337, var_301);
                    var_339 = wp::add(var_300, var_338);
                }
                var_340 = wp::where(var_336, var_339, var_300);
            }
            var_341 = wp::where(var_316, var_340, var_300);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_344 = (var_342 != var_196);
            var_343 = var_344;
            if (var_343) {
                var_345 = wp::extract(var_231, var_342);
                var_347 = (var_345 > var_346);
                var_348 = wp::int(var_347);
                var_350 = (var_232 < var_349);
                var_351 = wp::int(var_350);
                var_352 = wp::bit_xor(var_348, var_351);
                var_343 = var_343 && var_352;
            }
            if (var_343) {
                // cle1 += 1 << k                                                             <L 710>
                var_354 = wp::lshift(var_353, var_342);
                var_355 = wp::add(var_315, var_354);
            }
            var_356 = wp::where(var_343, var_355, var_315);
            // if k != j:                                                                     <L 711>
            var_357 = (var_342 != var_199);
            if (var_357) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_359 = wp::sub(var_358, var_342);
                var_360 = wp::sub(var_359, var_199);
                var_361 = wp::extract(var_20, var_196, var_360);
                var_363 = (var_361 > var_362);
                var_364 = wp::int(var_363);
                var_366 = (var_232 < var_365);
                var_367 = wp::int(var_366);
                var_368 = wp::bit_xor(var_364, var_367);
                var_369 = wp::sub(var_342, var_199);
                var_371 = wp::add(var_369, var_370);
                var_373 = wp::mod(var_371, var_372);
                var_375 = (var_373 == var_374);
                var_376 = wp::int(var_375);
                var_377 = wp::bit_xor(var_368, var_376);
                if (var_377) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_379 = wp::lshift(var_378, var_342);
                    var_380 = wp::add(var_341, var_379);
                }
                var_381 = wp::where(var_377, var_380, var_341);
            }
            var_382 = wp::where(var_357, var_381, var_341);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_385 = (var_383 != var_196);
            var_384 = var_385;
            if (var_384) {
                var_386 = wp::extract(var_231, var_383);
                var_388 = (var_386 > var_387);
                var_389 = wp::int(var_388);
                var_391 = (var_232 < var_390);
                var_392 = wp::int(var_391);
                var_393 = wp::bit_xor(var_389, var_392);
                var_384 = var_384 && var_393;
            }
            if (var_384) {
                // cle1 += 1 << k                                                             <L 710>
                var_395 = wp::lshift(var_394, var_383);
                var_396 = wp::add(var_356, var_395);
            }
            var_397 = wp::where(var_384, var_396, var_356);
            // if k != j:                                                                     <L 711>
            var_398 = (var_383 != var_199);
            if (var_398) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_400 = wp::sub(var_399, var_383);
                var_401 = wp::sub(var_400, var_199);
                var_402 = wp::extract(var_20, var_196, var_401);
                var_404 = (var_402 > var_403);
                var_405 = wp::int(var_404);
                var_407 = (var_232 < var_406);
                var_408 = wp::int(var_407);
                var_409 = wp::bit_xor(var_405, var_408);
                var_410 = wp::sub(var_383, var_199);
                var_412 = wp::add(var_410, var_411);
                var_414 = wp::mod(var_412, var_413);
                var_416 = (var_414 == var_415);
                var_417 = wp::int(var_416);
                var_418 = wp::bit_xor(var_409, var_417);
                if (var_418) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_420 = wp::lshift(var_419, var_383);
                    var_421 = wp::add(var_382, var_420);
                }
                var_422 = wp::where(var_418, var_421, var_382);
            }
            var_423 = wp::where(var_398, var_422, var_382);
            // axis_code = 12 + i * 3 + j                                                     <L 715>
            var_426 = wp::mul(var_196, var_425);
            var_427 = wp::add(var_424, var_426);
            var_428 = wp::add(var_427, var_199);
            // clnorm = cross_axis                                                            <L 716>
            var_429 = wp::copy(var_231);
            // inv = box_dist < 0                                                             <L 717>
            var_431 = (var_232 < var_430);
        }
        var_432 = wp::where(var_297, var_298, var_186);
        var_433 = wp::where(var_297, var_428, var_187);
        var_434 = wp::where(var_297, var_429, var_189);
        var_435 = wp::where(var_297, var_431, var_191);
        var_436 = wp::where(var_297, var_397, var_193);
        var_437 = wp::where(var_297, var_423, var_195);
        var_438 = wp::where(var_297, var_383, var_271);
        wp::assign(var_186, var_432);
        wp::assign(var_187, var_433);
        wp::assign(var_189, var_434);
        wp::assign(var_191, var_435);
        wp::assign(var_193, var_436);
        wp::assign(var_195, var_437);
        goto start_for_3;
    end_for_3:;
    // for j in range(3):                                                                     <L 670>
    var_441 = wp::range(var_440);
    start_for_6:;
        if (iter_cmp(var_441) == 0) goto end_for_6;
        var_442 = wp::iter_next(var_441);
        // if i == 0:                                                                         <L 672>
        var_444 = (var_439 == var_443);
        if (var_444) {
            // cross_axis = wp.vec3(0.0, -rot12[j, 2], rot12[j, 1])                           <L 673>
            var_447 = wp::extract(var_21, var_442, var_446);
            var_448 = wp::neg(var_447);
            var_450 = wp::extract(var_21, var_442, var_449);
            var_451 = wp::vec_t<3, wp::float32>(var_445, var_448, var_450);
        }
        var_452 = wp::where(var_444, var_451, var_231);
        if (!var_444) {
            // elif i == 1:                                                                   <L 674>
            var_454 = (var_439 == var_453);
            if (var_454) {
                // cross_axis = wp.vec3(rot12[j, 2], 0.0, -rot12[j, 0])                       <L 675>
                var_456 = wp::extract(var_21, var_442, var_455);
                var_459 = wp::extract(var_21, var_442, var_458);
                var_460 = wp::neg(var_459);
                var_461 = wp::vec_t<3, wp::float32>(var_456, var_457, var_460);
            }
            var_462 = wp::where(var_454, var_461, var_452);
            if (!var_454) {
                // cross_axis = wp.vec3(-rot12[j, 1], rot12[j, 0], 0.0)                       <L 677>
                var_464 = wp::extract(var_21, var_442, var_463);
                var_465 = wp::neg(var_464);
                var_467 = wp::extract(var_21, var_442, var_466);
                var_469 = wp::vec_t<3, wp::float32>(var_465, var_467, var_468);
            }
            var_470 = wp::where(var_454, var_462, var_469);
        }
        var_471 = wp::where(var_444, var_452, var_470);
        // cross_length = wp.length(cross_axis)                                               <L 679>
        var_472 = wp::length(var_471);
        // if cross_length < MJ_MINVAL:                                                       <L 680>
        var_473 = (var_472 < var_229);
        if (var_473) {
            // continue                                                                       <L 681>
            wp::assign(var_231, var_471);
            wp::assign(var_228, var_472);
            goto start_for_6;
        }
        var_474 = wp::where(var_473, var_231, var_471);
        var_475 = wp::where(var_473, var_228, var_472);
        // cross_axis /= cross_length                                                         <L 683>
        var_476 = wp::div(var_474, var_475);
        // box_dist = wp.dot(pos21, cross_axis)                                               <L 685>
        var_477 = wp::dot(var_15, var_476);
        // c3 = wp.float32(0.0)                                                               <L 686>
        var_479 = wp::float32(var_478);
        // for k in range(3):                                                                 <L 689>
        // if k != i:                                                                         <L 690>
        var_481 = (var_480 != var_439);
        if (var_481) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_482 = wp::extract(var_box1_size, var_480);
            var_483 = wp::extract(var_476, var_480);
            var_484 = wp::abs(var_483);
            var_485 = wp::mul(var_482, var_484);
            var_486 = wp::add(var_479, var_485);
        }
        var_487 = wp::where(var_481, var_486, var_479);
        // if k != j:                                                                         <L 692>
        var_488 = (var_480 != var_442);
        if (var_488) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_489 = wp::extract(var_box2_size, var_480);
            var_491 = wp::sub(var_490, var_480);
            var_492 = wp::sub(var_491, var_442);
            var_493 = wp::extract(var_31, var_439, var_492);
            var_494 = wp::mul(var_489, var_493);
            var_495 = wp::div(var_494, var_475);
            var_496 = wp::add(var_487, var_495);
        }
        var_497 = wp::where(var_488, var_496, var_487);
        // if k != i:                                                                         <L 690>
        var_499 = (var_498 != var_439);
        if (var_499) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_500 = wp::extract(var_box1_size, var_498);
            var_501 = wp::extract(var_476, var_498);
            var_502 = wp::abs(var_501);
            var_503 = wp::mul(var_500, var_502);
            var_504 = wp::add(var_497, var_503);
        }
        var_505 = wp::where(var_499, var_504, var_497);
        // if k != j:                                                                         <L 692>
        var_506 = (var_498 != var_442);
        if (var_506) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_507 = wp::extract(var_box2_size, var_498);
            var_509 = wp::sub(var_508, var_498);
            var_510 = wp::sub(var_509, var_442);
            var_511 = wp::extract(var_31, var_439, var_510);
            var_512 = wp::mul(var_507, var_511);
            var_513 = wp::div(var_512, var_475);
            var_514 = wp::add(var_505, var_513);
        }
        var_515 = wp::where(var_506, var_514, var_505);
        // if k != i:                                                                         <L 690>
        var_517 = (var_516 != var_439);
        if (var_517) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_518 = wp::extract(var_box1_size, var_516);
            var_519 = wp::extract(var_476, var_516);
            var_520 = wp::abs(var_519);
            var_521 = wp::mul(var_518, var_520);
            var_522 = wp::add(var_515, var_521);
        }
        var_523 = wp::where(var_517, var_522, var_515);
        // if k != j:                                                                         <L 692>
        var_524 = (var_516 != var_442);
        if (var_524) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_525 = wp::extract(var_box2_size, var_516);
            var_527 = wp::sub(var_526, var_516);
            var_528 = wp::sub(var_527, var_442);
            var_529 = wp::extract(var_31, var_439, var_528);
            var_530 = wp::mul(var_525, var_529);
            var_531 = wp::div(var_530, var_475);
            var_532 = wp::add(var_523, var_531);
        }
        var_533 = wp::where(var_524, var_532, var_523);
        // c3 -= wp.abs(box_dist)                                                             <L 695>
        var_534 = wp::abs(var_477);
        var_535 = wp::sub(var_533, var_534);
        // if c3 < -margin:                                                                   <L 698>
        var_536 = wp::neg(var_margin);
        var_537 = (var_535 < var_536);
        if (var_537) {
            // return contact_dist, contact_pos, contact_normals                              <L 699>
            ret_0 = var_0;
            ret_1 = var_10;
            ret_2 = var_11;
            return;
        }
        // if c3 < separation * (1.0 - 1e-12):                                                <L 702>
        var_540 = wp::sub(var_538, var_539);
        var_541 = wp::mul(var_186, var_540);
        var_542 = (var_535 < var_541);
        if (var_542) {
            // separation = c3                                                                <L 703>
            var_543 = wp::copy(var_535);
            // cle1 = 0                                                                       <L 705>
            // cle2 = 0                                                                       <L 706>
            // for k in range(3):                                                             <L 708>
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_548 = (var_546 != var_439);
            var_547 = var_548;
            if (var_547) {
                var_549 = wp::extract(var_476, var_546);
                var_551 = (var_549 > var_550);
                var_552 = wp::int(var_551);
                var_554 = (var_477 < var_553);
                var_555 = wp::int(var_554);
                var_556 = wp::bit_xor(var_552, var_555);
                var_547 = var_547 && var_556;
            }
            if (var_547) {
                // cle1 += 1 << k                                                             <L 710>
                var_558 = wp::lshift(var_557, var_546);
                var_559 = wp::add(var_544, var_558);
            }
            var_560 = wp::where(var_547, var_559, var_544);
            // if k != j:                                                                     <L 711>
            var_561 = (var_546 != var_442);
            if (var_561) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_563 = wp::sub(var_562, var_546);
                var_564 = wp::sub(var_563, var_442);
                var_565 = wp::extract(var_20, var_439, var_564);
                var_567 = (var_565 > var_566);
                var_568 = wp::int(var_567);
                var_570 = (var_477 < var_569);
                var_571 = wp::int(var_570);
                var_572 = wp::bit_xor(var_568, var_571);
                var_573 = wp::sub(var_546, var_442);
                var_575 = wp::add(var_573, var_574);
                var_577 = wp::mod(var_575, var_576);
                var_579 = (var_577 == var_578);
                var_580 = wp::int(var_579);
                var_581 = wp::bit_xor(var_572, var_580);
                if (var_581) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_583 = wp::lshift(var_582, var_546);
                    var_584 = wp::add(var_545, var_583);
                }
                var_585 = wp::where(var_581, var_584, var_545);
            }
            var_586 = wp::where(var_561, var_585, var_545);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_589 = (var_587 != var_439);
            var_588 = var_589;
            if (var_588) {
                var_590 = wp::extract(var_476, var_587);
                var_592 = (var_590 > var_591);
                var_593 = wp::int(var_592);
                var_595 = (var_477 < var_594);
                var_596 = wp::int(var_595);
                var_597 = wp::bit_xor(var_593, var_596);
                var_588 = var_588 && var_597;
            }
            if (var_588) {
                // cle1 += 1 << k                                                             <L 710>
                var_599 = wp::lshift(var_598, var_587);
                var_600 = wp::add(var_560, var_599);
            }
            var_601 = wp::where(var_588, var_600, var_560);
            // if k != j:                                                                     <L 711>
            var_602 = (var_587 != var_442);
            if (var_602) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_604 = wp::sub(var_603, var_587);
                var_605 = wp::sub(var_604, var_442);
                var_606 = wp::extract(var_20, var_439, var_605);
                var_608 = (var_606 > var_607);
                var_609 = wp::int(var_608);
                var_611 = (var_477 < var_610);
                var_612 = wp::int(var_611);
                var_613 = wp::bit_xor(var_609, var_612);
                var_614 = wp::sub(var_587, var_442);
                var_616 = wp::add(var_614, var_615);
                var_618 = wp::mod(var_616, var_617);
                var_620 = (var_618 == var_619);
                var_621 = wp::int(var_620);
                var_622 = wp::bit_xor(var_613, var_621);
                if (var_622) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_624 = wp::lshift(var_623, var_587);
                    var_625 = wp::add(var_586, var_624);
                }
                var_626 = wp::where(var_622, var_625, var_586);
            }
            var_627 = wp::where(var_602, var_626, var_586);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_630 = (var_628 != var_439);
            var_629 = var_630;
            if (var_629) {
                var_631 = wp::extract(var_476, var_628);
                var_633 = (var_631 > var_632);
                var_634 = wp::int(var_633);
                var_636 = (var_477 < var_635);
                var_637 = wp::int(var_636);
                var_638 = wp::bit_xor(var_634, var_637);
                var_629 = var_629 && var_638;
            }
            if (var_629) {
                // cle1 += 1 << k                                                             <L 710>
                var_640 = wp::lshift(var_639, var_628);
                var_641 = wp::add(var_601, var_640);
            }
            var_642 = wp::where(var_629, var_641, var_601);
            // if k != j:                                                                     <L 711>
            var_643 = (var_628 != var_442);
            if (var_643) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_645 = wp::sub(var_644, var_628);
                var_646 = wp::sub(var_645, var_442);
                var_647 = wp::extract(var_20, var_439, var_646);
                var_649 = (var_647 > var_648);
                var_650 = wp::int(var_649);
                var_652 = (var_477 < var_651);
                var_653 = wp::int(var_652);
                var_654 = wp::bit_xor(var_650, var_653);
                var_655 = wp::sub(var_628, var_442);
                var_657 = wp::add(var_655, var_656);
                var_659 = wp::mod(var_657, var_658);
                var_661 = (var_659 == var_660);
                var_662 = wp::int(var_661);
                var_663 = wp::bit_xor(var_654, var_662);
                if (var_663) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_665 = wp::lshift(var_664, var_628);
                    var_666 = wp::add(var_627, var_665);
                }
                var_667 = wp::where(var_663, var_666, var_627);
            }
            var_668 = wp::where(var_643, var_667, var_627);
            // axis_code = 12 + i * 3 + j                                                     <L 715>
            var_671 = wp::mul(var_439, var_670);
            var_672 = wp::add(var_669, var_671);
            var_673 = wp::add(var_672, var_442);
            // clnorm = cross_axis                                                            <L 716>
            var_674 = wp::copy(var_476);
            // inv = box_dist < 0                                                             <L 717>
            var_676 = (var_477 < var_675);
        }
        var_677 = wp::where(var_542, var_543, var_186);
        var_678 = wp::where(var_542, var_673, var_187);
        var_679 = wp::where(var_542, var_674, var_189);
        var_680 = wp::where(var_542, var_676, var_191);
        var_681 = wp::where(var_542, var_642, var_193);
        var_682 = wp::where(var_542, var_668, var_195);
        var_683 = wp::where(var_542, var_628, var_516);
        wp::assign(var_186, var_677);
        wp::assign(var_187, var_678);
        wp::assign(var_189, var_679);
        wp::assign(var_191, var_680);
        wp::assign(var_193, var_681);
        wp::assign(var_195, var_682);
        wp::assign(var_231, var_476);
        wp::assign(var_228, var_475);
        wp::assign(var_232, var_477);
        wp::assign(var_290, var_535);
        goto start_for_6;
    end_for_6:;
    // for j in range(3):                                                                     <L 670>
    var_686 = wp::range(var_685);
    start_for_9:;
        if (iter_cmp(var_686) == 0) goto end_for_9;
        var_687 = wp::iter_next(var_686);
        // if i == 0:                                                                         <L 672>
        var_689 = (var_684 == var_688);
        if (var_689) {
            // cross_axis = wp.vec3(0.0, -rot12[j, 2], rot12[j, 1])                           <L 673>
            var_692 = wp::extract(var_21, var_687, var_691);
            var_693 = wp::neg(var_692);
            var_695 = wp::extract(var_21, var_687, var_694);
            var_696 = wp::vec_t<3, wp::float32>(var_690, var_693, var_695);
        }
        var_697 = wp::where(var_689, var_696, var_231);
        if (!var_689) {
            // elif i == 1:                                                                   <L 674>
            var_699 = (var_684 == var_698);
            if (var_699) {
                // cross_axis = wp.vec3(rot12[j, 2], 0.0, -rot12[j, 0])                       <L 675>
                var_701 = wp::extract(var_21, var_687, var_700);
                var_704 = wp::extract(var_21, var_687, var_703);
                var_705 = wp::neg(var_704);
                var_706 = wp::vec_t<3, wp::float32>(var_701, var_702, var_705);
            }
            var_707 = wp::where(var_699, var_706, var_697);
            if (!var_699) {
                // cross_axis = wp.vec3(-rot12[j, 1], rot12[j, 0], 0.0)                       <L 677>
                var_709 = wp::extract(var_21, var_687, var_708);
                var_710 = wp::neg(var_709);
                var_712 = wp::extract(var_21, var_687, var_711);
                var_714 = wp::vec_t<3, wp::float32>(var_710, var_712, var_713);
            }
            var_715 = wp::where(var_699, var_707, var_714);
        }
        var_716 = wp::where(var_689, var_697, var_715);
        // cross_length = wp.length(cross_axis)                                               <L 679>
        var_717 = wp::length(var_716);
        // if cross_length < MJ_MINVAL:                                                       <L 680>
        var_718 = (var_717 < var_229);
        if (var_718) {
            // continue                                                                       <L 681>
            wp::assign(var_231, var_716);
            wp::assign(var_228, var_717);
            goto start_for_9;
        }
        var_719 = wp::where(var_718, var_231, var_716);
        var_720 = wp::where(var_718, var_228, var_717);
        // cross_axis /= cross_length                                                         <L 683>
        var_721 = wp::div(var_719, var_720);
        // box_dist = wp.dot(pos21, cross_axis)                                               <L 685>
        var_722 = wp::dot(var_15, var_721);
        // c3 = wp.float32(0.0)                                                               <L 686>
        var_724 = wp::float32(var_723);
        // for k in range(3):                                                                 <L 689>
        // if k != i:                                                                         <L 690>
        var_726 = (var_725 != var_684);
        if (var_726) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_727 = wp::extract(var_box1_size, var_725);
            var_728 = wp::extract(var_721, var_725);
            var_729 = wp::abs(var_728);
            var_730 = wp::mul(var_727, var_729);
            var_731 = wp::add(var_724, var_730);
        }
        var_732 = wp::where(var_726, var_731, var_724);
        // if k != j:                                                                         <L 692>
        var_733 = (var_725 != var_687);
        if (var_733) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_734 = wp::extract(var_box2_size, var_725);
            var_736 = wp::sub(var_735, var_725);
            var_737 = wp::sub(var_736, var_687);
            var_738 = wp::extract(var_31, var_684, var_737);
            var_739 = wp::mul(var_734, var_738);
            var_740 = wp::div(var_739, var_720);
            var_741 = wp::add(var_732, var_740);
        }
        var_742 = wp::where(var_733, var_741, var_732);
        // if k != i:                                                                         <L 690>
        var_744 = (var_743 != var_684);
        if (var_744) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_745 = wp::extract(var_box1_size, var_743);
            var_746 = wp::extract(var_721, var_743);
            var_747 = wp::abs(var_746);
            var_748 = wp::mul(var_745, var_747);
            var_749 = wp::add(var_742, var_748);
        }
        var_750 = wp::where(var_744, var_749, var_742);
        // if k != j:                                                                         <L 692>
        var_751 = (var_743 != var_687);
        if (var_751) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_752 = wp::extract(var_box2_size, var_743);
            var_754 = wp::sub(var_753, var_743);
            var_755 = wp::sub(var_754, var_687);
            var_756 = wp::extract(var_31, var_684, var_755);
            var_757 = wp::mul(var_752, var_756);
            var_758 = wp::div(var_757, var_720);
            var_759 = wp::add(var_750, var_758);
        }
        var_760 = wp::where(var_751, var_759, var_750);
        // if k != i:                                                                         <L 690>
        var_762 = (var_761 != var_684);
        if (var_762) {
            // c3 += box1_size[k] * wp.abs(cross_axis[k])                                     <L 691>
            var_763 = wp::extract(var_box1_size, var_761);
            var_764 = wp::extract(var_721, var_761);
            var_765 = wp::abs(var_764);
            var_766 = wp::mul(var_763, var_765);
            var_767 = wp::add(var_760, var_766);
        }
        var_768 = wp::where(var_762, var_767, var_760);
        // if k != j:                                                                         <L 692>
        var_769 = (var_761 != var_687);
        if (var_769) {
            // c3 += box2_size[k] * rot21abs[i, 3 - k - j] / cross_length                     <L 693>
            var_770 = wp::extract(var_box2_size, var_761);
            var_772 = wp::sub(var_771, var_761);
            var_773 = wp::sub(var_772, var_687);
            var_774 = wp::extract(var_31, var_684, var_773);
            var_775 = wp::mul(var_770, var_774);
            var_776 = wp::div(var_775, var_720);
            var_777 = wp::add(var_768, var_776);
        }
        var_778 = wp::where(var_769, var_777, var_768);
        // c3 -= wp.abs(box_dist)                                                             <L 695>
        var_779 = wp::abs(var_722);
        var_780 = wp::sub(var_778, var_779);
        // if c3 < -margin:                                                                   <L 698>
        var_781 = wp::neg(var_margin);
        var_782 = (var_780 < var_781);
        if (var_782) {
            // return contact_dist, contact_pos, contact_normals                              <L 699>
            ret_0 = var_0;
            ret_1 = var_10;
            ret_2 = var_11;
            return;
        }
        // if c3 < separation * (1.0 - 1e-12):                                                <L 702>
        var_785 = wp::sub(var_783, var_784);
        var_786 = wp::mul(var_186, var_785);
        var_787 = (var_780 < var_786);
        if (var_787) {
            // separation = c3                                                                <L 703>
            var_788 = wp::copy(var_780);
            // cle1 = 0                                                                       <L 705>
            // cle2 = 0                                                                       <L 706>
            // for k in range(3):                                                             <L 708>
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_793 = (var_791 != var_684);
            var_792 = var_793;
            if (var_792) {
                var_794 = wp::extract(var_721, var_791);
                var_796 = (var_794 > var_795);
                var_797 = wp::int(var_796);
                var_799 = (var_722 < var_798);
                var_800 = wp::int(var_799);
                var_801 = wp::bit_xor(var_797, var_800);
                var_792 = var_792 && var_801;
            }
            if (var_792) {
                // cle1 += 1 << k                                                             <L 710>
                var_803 = wp::lshift(var_802, var_791);
                var_804 = wp::add(var_789, var_803);
            }
            var_805 = wp::where(var_792, var_804, var_789);
            // if k != j:                                                                     <L 711>
            var_806 = (var_791 != var_687);
            if (var_806) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_808 = wp::sub(var_807, var_791);
                var_809 = wp::sub(var_808, var_687);
                var_810 = wp::extract(var_20, var_684, var_809);
                var_812 = (var_810 > var_811);
                var_813 = wp::int(var_812);
                var_815 = (var_722 < var_814);
                var_816 = wp::int(var_815);
                var_817 = wp::bit_xor(var_813, var_816);
                var_818 = wp::sub(var_791, var_687);
                var_820 = wp::add(var_818, var_819);
                var_822 = wp::mod(var_820, var_821);
                var_824 = (var_822 == var_823);
                var_825 = wp::int(var_824);
                var_826 = wp::bit_xor(var_817, var_825);
                if (var_826) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_828 = wp::lshift(var_827, var_791);
                    var_829 = wp::add(var_790, var_828);
                }
                var_830 = wp::where(var_826, var_829, var_790);
            }
            var_831 = wp::where(var_806, var_830, var_790);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_834 = (var_832 != var_684);
            var_833 = var_834;
            if (var_833) {
                var_835 = wp::extract(var_721, var_832);
                var_837 = (var_835 > var_836);
                var_838 = wp::int(var_837);
                var_840 = (var_722 < var_839);
                var_841 = wp::int(var_840);
                var_842 = wp::bit_xor(var_838, var_841);
                var_833 = var_833 && var_842;
            }
            if (var_833) {
                // cle1 += 1 << k                                                             <L 710>
                var_844 = wp::lshift(var_843, var_832);
                var_845 = wp::add(var_805, var_844);
            }
            var_846 = wp::where(var_833, var_845, var_805);
            // if k != j:                                                                     <L 711>
            var_847 = (var_832 != var_687);
            if (var_847) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_849 = wp::sub(var_848, var_832);
                var_850 = wp::sub(var_849, var_687);
                var_851 = wp::extract(var_20, var_684, var_850);
                var_853 = (var_851 > var_852);
                var_854 = wp::int(var_853);
                var_856 = (var_722 < var_855);
                var_857 = wp::int(var_856);
                var_858 = wp::bit_xor(var_854, var_857);
                var_859 = wp::sub(var_832, var_687);
                var_861 = wp::add(var_859, var_860);
                var_863 = wp::mod(var_861, var_862);
                var_865 = (var_863 == var_864);
                var_866 = wp::int(var_865);
                var_867 = wp::bit_xor(var_858, var_866);
                if (var_867) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_869 = wp::lshift(var_868, var_832);
                    var_870 = wp::add(var_831, var_869);
                }
                var_871 = wp::where(var_867, var_870, var_831);
            }
            var_872 = wp::where(var_847, var_871, var_831);
            // if k != i and (int(cross_axis[k] > 0) ^ int(box_dist < 0)):                    <L 709>
            var_875 = (var_873 != var_684);
            var_874 = var_875;
            if (var_874) {
                var_876 = wp::extract(var_721, var_873);
                var_878 = (var_876 > var_877);
                var_879 = wp::int(var_878);
                var_881 = (var_722 < var_880);
                var_882 = wp::int(var_881);
                var_883 = wp::bit_xor(var_879, var_882);
                var_874 = var_874 && var_883;
            }
            if (var_874) {
                // cle1 += 1 << k                                                             <L 710>
                var_885 = wp::lshift(var_884, var_873);
                var_886 = wp::add(var_846, var_885);
            }
            var_887 = wp::where(var_874, var_886, var_846);
            // if k != j:                                                                     <L 711>
            var_888 = (var_873 != var_687);
            if (var_888) {
                // if int(rot21[i, 3 - k - j] > 0) ^ int(box_dist < 0) ^ int((k - j + 3) % 3 == 1):       <L 712>
                var_890 = wp::sub(var_889, var_873);
                var_891 = wp::sub(var_890, var_687);
                var_892 = wp::extract(var_20, var_684, var_891);
                var_894 = (var_892 > var_893);
                var_895 = wp::int(var_894);
                var_897 = (var_722 < var_896);
                var_898 = wp::int(var_897);
                var_899 = wp::bit_xor(var_895, var_898);
                var_900 = wp::sub(var_873, var_687);
                var_902 = wp::add(var_900, var_901);
                var_904 = wp::mod(var_902, var_903);
                var_906 = (var_904 == var_905);
                var_907 = wp::int(var_906);
                var_908 = wp::bit_xor(var_899, var_907);
                if (var_908) {
                    // cle2 += 1 << k                                                         <L 713>
                    var_910 = wp::lshift(var_909, var_873);
                    var_911 = wp::add(var_872, var_910);
                }
                var_912 = wp::where(var_908, var_911, var_872);
            }
            var_913 = wp::where(var_888, var_912, var_872);
            // axis_code = 12 + i * 3 + j                                                     <L 715>
            var_916 = wp::mul(var_684, var_915);
            var_917 = wp::add(var_914, var_916);
            var_918 = wp::add(var_917, var_687);
            // clnorm = cross_axis                                                            <L 716>
            var_919 = wp::copy(var_721);
            // inv = box_dist < 0                                                             <L 717>
            var_921 = (var_722 < var_920);
        }
        var_922 = wp::where(var_787, var_788, var_186);
        var_923 = wp::where(var_787, var_918, var_187);
        var_924 = wp::where(var_787, var_919, var_189);
        var_925 = wp::where(var_787, var_921, var_191);
        var_926 = wp::where(var_787, var_887, var_193);
        var_927 = wp::where(var_787, var_913, var_195);
        var_928 = wp::where(var_787, var_873, var_761);
        wp::assign(var_186, var_922);
        wp::assign(var_187, var_923);
        wp::assign(var_189, var_924);
        wp::assign(var_191, var_925);
        wp::assign(var_193, var_926);
        wp::assign(var_195, var_927);
        wp::assign(var_231, var_721);
        wp::assign(var_228, var_720);
        wp::assign(var_232, var_722);
        wp::assign(var_290, var_780);
        goto start_for_9;
    end_for_9:;
    // if axis_code == -1:                                                                    <L 720>
    var_930 = (var_187 == var_929);
    if (var_930) {
        // return contact_dist, contact_pos, contact_normals                                  <L 721>
        ret_0 = var_0;
        ret_1 = var_10;
        ret_2 = var_11;
        return;
    }
    // points = mat83f()                                                                      <L 723>
    var_931 = wp::mat_t<8, 3, wp::float32>();
    // depth = vec8f()                                                                        <L 724>
    var_932 = wp::vec_t<8, wp::float32>();
    // max_con_pair = 8                                                                       <L 725>
    // if axis_code < 12:                                                                     <L 728>
    var_935 = (var_187 < var_934);
    if (var_935) {
        // face_idx = axis_code % 6                                                           <L 730>
        var_937 = wp::mod(var_187, var_936);
        // box_idx = axis_code // 6                                                           <L 731>
        var_939 = wp::floordiv(var_187, var_938);
        // rotmore = _compute_rotmore(face_idx)                                               <L 732>
        var_940 = _compute_rotmore_0(var_937);
        // r = rotmore @ wp.where(box_idx, rot12, rot21)                                      <L 734>
        var_941 = wp::where(var_939, var_21, var_20);
        var_942 = wp::mul(var_940, var_941);
        // p = rotmore @ wp.where(box_idx, pos12, pos21)                                      <L 735>
        var_943 = wp::where(var_939, var_18, var_15);
        var_944 = wp::mul(var_940, var_943);
        // ss = wp.abs(rotmore @ wp.where(box_idx, box2_size, box1_size))                     <L 736>
        var_945 = wp::where(var_939, var_box2_size, var_box1_size);
        var_946 = wp::mul(var_940, var_945);
        var_947 = wp::abs(var_946);
        // s = wp.where(box_idx, box1_size, box2_size)                                        <L 737>
        var_948 = wp::where(var_939, var_box1_size, var_box2_size);
        // rt = wp.transpose(r)                                                               <L 738>
        var_949 = wp::transpose(var_942);
        // lx, ly, hz = ss[0], ss[1], ss[2]                                                   <L 740>
        var_951 = wp::extract(var_947, var_950);
        var_953 = wp::extract(var_947, var_952);
        var_955 = wp::extract(var_947, var_954);
        // p[2] -= hz                                                                         <L 741>
        wp::sub_inplace(var_944, var_956, var_955);
        // clcorner = wp.int32(0)  # corner of non-face box with least axis separation        <L 743>
        var_958 = wp::int32(var_957);
        // for i in range(3):                                                                 <L 745>
        // if r[2, i] < 0:                                                                    <L 746>
        var_961 = wp::extract(var_942, var_960, var_959);
        var_963 = (var_961 < var_962);
        if (var_963) {
            // clcorner += 1 << i                                                             <L 747>
            var_965 = wp::lshift(var_964, var_959);
            var_966 = wp::add(var_958, var_965);
        }
        var_967 = wp::where(var_963, var_966, var_958);
        // if r[2, i] < 0:                                                                    <L 746>
        var_970 = wp::extract(var_942, var_969, var_968);
        var_972 = (var_970 < var_971);
        if (var_972) {
            // clcorner += 1 << i                                                             <L 747>
            var_974 = wp::lshift(var_973, var_968);
            var_975 = wp::add(var_967, var_974);
        }
        var_976 = wp::where(var_972, var_975, var_967);
        // if r[2, i] < 0:                                                                    <L 746>
        var_979 = wp::extract(var_942, var_978, var_977);
        var_981 = (var_979 < var_980);
        if (var_981) {
            // clcorner += 1 << i                                                             <L 747>
            var_983 = wp::lshift(var_982, var_977);
            var_984 = wp::add(var_976, var_983);
        }
        var_985 = wp::where(var_981, var_984, var_976);
        // lp = p                                                                             <L 749>
        var_986 = wp::copy(var_944);
        // for i in range(wp.static(3)):                                                      <L 750>
        // lp += rt[i] * s[i] * wp.where(clcorner & (1 << i), 1.0, -1.0)                      <L 751>
        var_988 = wp::extract(var_949, var_987);
        var_989 = wp::extract(var_948, var_987);
        var_990 = wp::mul(var_988, var_989);
        var_992 = wp::lshift(var_991, var_987);
        var_993 = wp::bit_and(var_985, var_992);
        var_996 = wp::where(var_993, var_994, var_995);
        var_997 = wp::mul(var_990, var_996);
        var_998 = wp::add(var_986, var_997);
        var_1000 = wp::extract(var_949, var_999);
        var_1001 = wp::extract(var_948, var_999);
        var_1002 = wp::mul(var_1000, var_1001);
        var_1004 = wp::lshift(var_1003, var_999);
        var_1005 = wp::bit_and(var_985, var_1004);
        var_1008 = wp::where(var_1005, var_1006, var_1007);
        var_1009 = wp::mul(var_1002, var_1008);
        var_1010 = wp::add(var_998, var_1009);
        var_1012 = wp::extract(var_949, var_1011);
        var_1013 = wp::extract(var_948, var_1011);
        var_1014 = wp::mul(var_1012, var_1013);
        var_1016 = wp::lshift(var_1015, var_1011);
        var_1017 = wp::bit_and(var_985, var_1016);
        var_1020 = wp::where(var_1017, var_1018, var_1019);
        var_1021 = wp::mul(var_1014, var_1020);
        var_1022 = wp::add(var_1010, var_1021);
        // m = wp.int32(1)                                                                    <L 753>
        var_1024 = wp::int32(var_1023);
        // dirs = wp.int32(0)                                                                 <L 754>
        var_1026 = wp::int32(var_1025);
        // cn1 = wp.vec3(0.0)                                                                 <L 756>
        var_1028 = wp::vec_t<3, wp::float32>(var_1027);
        // cn2 = wp.vec3(0.0)                                                                 <L 757>
        var_1030 = wp::vec_t<3, wp::float32>(var_1029);
        // for i in range(3):                                                                 <L 759>
        // if wp.abs(r[2, i]) < 0.5:                                                          <L 760>
        var_1033 = wp::extract(var_942, var_1032, var_1031);
        var_1034 = wp::abs(var_1033);
        var_1036 = (var_1034 < var_1035);
        if (var_1036) {
            // if not dirs:                                                                   <L 761>
            var_1037 = wp::unot(var_1026);
            if (var_1037) {
                // cn1 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 762>
                var_1038 = wp::extract(var_949, var_1031);
                var_1039 = wp::extract(var_948, var_1031);
                var_1040 = wp::mul(var_1038, var_1039);
                var_1042 = wp::lshift(var_1041, var_1031);
                var_1043 = wp::bit_and(var_985, var_1042);
                var_1046 = wp::where(var_1043, var_1044, var_1045);
                var_1047 = wp::mul(var_1040, var_1046);
            }
            var_1048 = wp::where(var_1037, var_1047, var_1028);
            if (!var_1037) {
                // cn2 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 764>
                var_1049 = wp::extract(var_949, var_1031);
                var_1050 = wp::extract(var_948, var_1031);
                var_1051 = wp::mul(var_1049, var_1050);
                var_1053 = wp::lshift(var_1052, var_1031);
                var_1054 = wp::bit_and(var_985, var_1053);
                var_1057 = wp::where(var_1054, var_1055, var_1056);
                var_1058 = wp::mul(var_1051, var_1057);
            }
            var_1059 = wp::where(var_1037, var_1030, var_1058);
            // dirs += 1                                                                      <L 766>
            var_1061 = wp::add(var_1026, var_1060);
        }
        var_1062 = wp::where(var_1036, var_1061, var_1026);
        var_1063 = wp::where(var_1036, var_1048, var_1028);
        var_1064 = wp::where(var_1036, var_1059, var_1030);
        // if wp.abs(r[2, i]) < 0.5:                                                          <L 760>
        var_1067 = wp::extract(var_942, var_1066, var_1065);
        var_1068 = wp::abs(var_1067);
        var_1070 = (var_1068 < var_1069);
        if (var_1070) {
            // if not dirs:                                                                   <L 761>
            var_1071 = wp::unot(var_1062);
            if (var_1071) {
                // cn1 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 762>
                var_1072 = wp::extract(var_949, var_1065);
                var_1073 = wp::extract(var_948, var_1065);
                var_1074 = wp::mul(var_1072, var_1073);
                var_1076 = wp::lshift(var_1075, var_1065);
                var_1077 = wp::bit_and(var_985, var_1076);
                var_1080 = wp::where(var_1077, var_1078, var_1079);
                var_1081 = wp::mul(var_1074, var_1080);
            }
            var_1082 = wp::where(var_1071, var_1081, var_1063);
            if (!var_1071) {
                // cn2 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 764>
                var_1083 = wp::extract(var_949, var_1065);
                var_1084 = wp::extract(var_948, var_1065);
                var_1085 = wp::mul(var_1083, var_1084);
                var_1087 = wp::lshift(var_1086, var_1065);
                var_1088 = wp::bit_and(var_985, var_1087);
                var_1091 = wp::where(var_1088, var_1089, var_1090);
                var_1092 = wp::mul(var_1085, var_1091);
            }
            var_1093 = wp::where(var_1071, var_1064, var_1092);
            // dirs += 1                                                                      <L 766>
            var_1095 = wp::add(var_1062, var_1094);
        }
        var_1096 = wp::where(var_1070, var_1095, var_1062);
        var_1097 = wp::where(var_1070, var_1082, var_1063);
        var_1098 = wp::where(var_1070, var_1093, var_1064);
        // if wp.abs(r[2, i]) < 0.5:                                                          <L 760>
        var_1101 = wp::extract(var_942, var_1100, var_1099);
        var_1102 = wp::abs(var_1101);
        var_1104 = (var_1102 < var_1103);
        if (var_1104) {
            // if not dirs:                                                                   <L 761>
            var_1105 = wp::unot(var_1096);
            if (var_1105) {
                // cn1 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 762>
                var_1106 = wp::extract(var_949, var_1099);
                var_1107 = wp::extract(var_948, var_1099);
                var_1108 = wp::mul(var_1106, var_1107);
                var_1110 = wp::lshift(var_1109, var_1099);
                var_1111 = wp::bit_and(var_985, var_1110);
                var_1114 = wp::where(var_1111, var_1112, var_1113);
                var_1115 = wp::mul(var_1108, var_1114);
            }
            var_1116 = wp::where(var_1105, var_1115, var_1097);
            if (!var_1105) {
                // cn2 = rt[i] * s[i] * wp.where(clcorner & (1 << i), -2.0, 2.0)              <L 764>
                var_1117 = wp::extract(var_949, var_1099);
                var_1118 = wp::extract(var_948, var_1099);
                var_1119 = wp::mul(var_1117, var_1118);
                var_1121 = wp::lshift(var_1120, var_1099);
                var_1122 = wp::bit_and(var_985, var_1121);
                var_1125 = wp::where(var_1122, var_1123, var_1124);
                var_1126 = wp::mul(var_1119, var_1125);
            }
            var_1127 = wp::where(var_1105, var_1098, var_1126);
            // dirs += 1                                                                      <L 766>
            var_1129 = wp::add(var_1096, var_1128);
        }
        var_1130 = wp::where(var_1104, var_1129, var_1096);
        var_1131 = wp::where(var_1104, var_1116, var_1097);
        var_1132 = wp::where(var_1104, var_1127, var_1098);
        // k = dirs * dirs                                                                    <L 768>
        var_1133 = wp::mul(var_1130, var_1130);
        // n = wp.int32(0)                                                                    <L 772>
        var_1135 = wp::int32(var_1134);
        // for i in range(k):                                                                 <L 774>
        var_1136 = wp::range(var_1133);
        start_for_13:;
            if (iter_cmp(var_1136) == 0) goto end_for_13;
            var_1137 = wp::iter_next(var_1136);
            // for q in range(2):                                                             <L 775>
            // lav = lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, cn1, cn2))           <L 777>
            var_1140 = (var_1137 < var_1139);
            var_1142 = wp::vec_t<3, wp::float32>(var_1141);
            var_1144 = (var_1137 == var_1143);
            var_1145 = wp::where(var_1144, var_1131, var_1132);
            var_1146 = wp::where(var_1140, var_1142, var_1145);
            var_1147 = wp::add(var_1022, var_1146);
            // lbv = wp.where(i == 0 or i == 3, cn1, cn2)                                     <L 778>
            var_1150 = (var_1137 == var_1149);
            var_1148 = var_1150;
            if (!var_1148) {
                var_1152 = (var_1137 == var_1151);
                var_1148 = var_1148 || var_1152;
            }
            var_1153 = wp::where(var_1148, var_1131, var_1132);
            // if wp.abs(lbv[q]) > MJ_MINVAL:                                                 <L 780>
            var_1154 = wp::extract(var_1153, var_1138);
            var_1155 = wp::abs(var_1154);
            var_1156 = (var_1155 > var_229);
            if (var_1156) {
                // br = 1.0 / lbv[q]                                                          <L 781>
                var_1158 = wp::extract(var_1153, var_1138);
                var_1159 = wp::div(var_1157, var_1158);
                // for j in range(-1, 2, 2):                                                  <L 782>
                var_1163 = wp::range(var_1160, var_1161, var_1162);
                start_for_15:;
                    if (iter_cmp(var_1163) == 0) goto end_for_15;
                    var_1164 = wp::iter_next(var_1163);
                    // l = ss[q] * wp.float32(j)                                              <L 783>
                    var_1165 = wp::extract(var_947, var_1138);
                    var_1166 = wp::float32(var_1164);
                    var_1167 = wp::mul(var_1165, var_1166);
                    // c1 = (l - lav[q]) * br                                                 <L 784>
                    var_1168 = wp::extract(var_1147, var_1138);
                    var_1169 = wp::sub(var_1167, var_1168);
                    var_1170 = wp::mul(var_1169, var_1159);
                    // if c1 < 0 or c1 > 1:                                                   <L 785>
                    var_1173 = (var_1170 < var_1172);
                    var_1171 = var_1173;
                    if (!var_1171) {
                        var_1175 = (var_1170 > var_1174);
                        var_1171 = var_1171 || var_1175;
                    }
                    if (var_1171) {
                        // continue                                                           <L 786>
                        wp::assign(var_149, var_1170);
                        goto start_for_15;
                    }
                    var_1176 = wp::where(var_1171, var_149, var_1170);
                    // c2 = lav[1 - q] + lbv[1 - q] * c1                                      <L 787>
                    var_1178 = wp::sub(var_1177, var_1138);
                    var_1179 = wp::extract(var_1147, var_1178);
                    var_1181 = wp::sub(var_1180, var_1138);
                    var_1182 = wp::extract(var_1153, var_1181);
                    var_1183 = wp::mul(var_1182, var_1176);
                    var_1184 = wp::add(var_1179, var_1183);
                    // if wp.abs(c2) > ss[1 - q]:                                             <L 788>
                    var_1185 = wp::abs(var_1184);
                    var_1187 = wp::sub(var_1186, var_1138);
                    var_1188 = wp::extract(var_947, var_1187);
                    var_1189 = (var_1185 > var_1188);
                    if (var_1189) {
                        // continue                                                           <L 789>
                        wp::assign(var_149, var_1176);
                        wp::assign(var_156, var_1184);
                        goto start_for_15;
                    }
                    var_1190 = wp::where(var_1189, var_149, var_1176);
                    var_1191 = wp::where(var_1189, var_156, var_1184);
                    // points[n] = lav + c1 * lbv                                             <L 791>
                    var_1192 = wp::mul(var_1190, var_1153);
                    var_1193 = wp::add(var_1147, var_1192);
                    wp::assign_inplace(var_931, var_1135, var_1193);
                    // n += 1                                                                 <L 792>
                    var_1195 = wp::add(var_1135, var_1194);
                    wp::assign(var_149, var_1190);
                    wp::assign(var_156, var_1191);
                    wp::assign(var_1135, var_1195);
                    goto start_for_15;
                end_for_15:;
            }
            var_1196 = wp::where(var_1156, var_1164, var_687);
            // lav = lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, cn1, cn2))           <L 777>
            var_1199 = (var_1137 < var_1198);
            var_1201 = wp::vec_t<3, wp::float32>(var_1200);
            var_1203 = (var_1137 == var_1202);
            var_1204 = wp::where(var_1203, var_1131, var_1132);
            var_1205 = wp::where(var_1199, var_1201, var_1204);
            var_1206 = wp::add(var_1022, var_1205);
            // lbv = wp.where(i == 0 or i == 3, cn1, cn2)                                     <L 778>
            var_1209 = (var_1137 == var_1208);
            var_1207 = var_1209;
            if (!var_1207) {
                var_1211 = (var_1137 == var_1210);
                var_1207 = var_1207 || var_1211;
            }
            var_1212 = wp::where(var_1207, var_1131, var_1132);
            // if wp.abs(lbv[q]) > MJ_MINVAL:                                                 <L 780>
            var_1213 = wp::extract(var_1212, var_1197);
            var_1214 = wp::abs(var_1213);
            var_1215 = (var_1214 > var_229);
            if (var_1215) {
                // br = 1.0 / lbv[q]                                                          <L 781>
                var_1217 = wp::extract(var_1212, var_1197);
                var_1218 = wp::div(var_1216, var_1217);
                // for j in range(-1, 2, 2):                                                  <L 782>
                var_1222 = wp::range(var_1219, var_1220, var_1221);
                start_for_17:;
                    if (iter_cmp(var_1222) == 0) goto end_for_17;
                    var_1223 = wp::iter_next(var_1222);
                    // l = ss[q] * wp.float32(j)                                              <L 783>
                    var_1224 = wp::extract(var_947, var_1197);
                    var_1225 = wp::float32(var_1223);
                    var_1226 = wp::mul(var_1224, var_1225);
                    // c1 = (l - lav[q]) * br                                                 <L 784>
                    var_1227 = wp::extract(var_1206, var_1197);
                    var_1228 = wp::sub(var_1226, var_1227);
                    var_1229 = wp::mul(var_1228, var_1218);
                    // if c1 < 0 or c1 > 1:                                                   <L 785>
                    var_1232 = (var_1229 < var_1231);
                    var_1230 = var_1232;
                    if (!var_1230) {
                        var_1234 = (var_1229 > var_1233);
                        var_1230 = var_1230 || var_1234;
                    }
                    if (var_1230) {
                        // continue                                                           <L 786>
                        wp::assign(var_149, var_1229);
                        wp::assign(var_1167, var_1226);
                        goto start_for_17;
                    }
                    var_1235 = wp::where(var_1230, var_149, var_1229);
                    var_1236 = wp::where(var_1230, var_1167, var_1226);
                    // c2 = lav[1 - q] + lbv[1 - q] * c1                                      <L 787>
                    var_1238 = wp::sub(var_1237, var_1197);
                    var_1239 = wp::extract(var_1206, var_1238);
                    var_1241 = wp::sub(var_1240, var_1197);
                    var_1242 = wp::extract(var_1212, var_1241);
                    var_1243 = wp::mul(var_1242, var_1235);
                    var_1244 = wp::add(var_1239, var_1243);
                    // if wp.abs(c2) > ss[1 - q]:                                             <L 788>
                    var_1245 = wp::abs(var_1244);
                    var_1247 = wp::sub(var_1246, var_1197);
                    var_1248 = wp::extract(var_947, var_1247);
                    var_1249 = (var_1245 > var_1248);
                    if (var_1249) {
                        // continue                                                           <L 789>
                        wp::assign(var_149, var_1235);
                        wp::assign(var_156, var_1244);
                        wp::assign(var_1167, var_1236);
                        goto start_for_17;
                    }
                    var_1250 = wp::where(var_1249, var_149, var_1235);
                    var_1251 = wp::where(var_1249, var_156, var_1244);
                    var_1252 = wp::where(var_1249, var_1167, var_1236);
                    // points[n] = lav + c1 * lbv                                             <L 791>
                    var_1253 = wp::mul(var_1250, var_1212);
                    var_1254 = wp::add(var_1206, var_1253);
                    wp::assign_inplace(var_931, var_1135, var_1254);
                    // n += 1                                                                 <L 792>
                    var_1256 = wp::add(var_1135, var_1255);
                    wp::assign(var_149, var_1250);
                    wp::assign(var_156, var_1251);
                    wp::assign(var_1135, var_1256);
                    wp::assign(var_1167, var_1252);
                    goto start_for_17;
                end_for_17:;
            }
            var_1257 = wp::where(var_1215, var_1223, var_1196);
            var_1258 = wp::where(var_1215, var_1218, var_1159);
            wp::assign(var_687, var_1257);
            goto start_for_13;
        end_for_13:;
        // if dirs == 2:                                                                      <L 794>
        var_1260 = (var_1130 == var_1259);
        if (var_1260) {
            // ax = cn1[0]                                                                    <L 795>
            var_1262 = wp::extract(var_1131, var_1261);
            // bx = cn2[0]                                                                    <L 796>
            var_1264 = wp::extract(var_1132, var_1263);
            // ay = cn1[1]                                                                    <L 797>
            var_1266 = wp::extract(var_1131, var_1265);
            // by = cn2[1]                                                                    <L 798>
            var_1268 = wp::extract(var_1132, var_1267);
            // C = safe_div(1.0, ax * by - bx * ay)                                           <L 799>
            var_1270 = wp::mul(var_1262, var_1268);
            var_1271 = wp::mul(var_1264, var_1266);
            var_1272 = wp::sub(var_1270, var_1271);
            var_1273 = safe_div_0(var_1269, var_1272);
            // for i in range(4):                                                             <L 801>
            // llx = wp.where(i // 2, lx, -lx)                                                <L 802>
            var_1276 = wp::floordiv(var_1274, var_1275);
            var_1277 = wp::neg(var_951);
            var_1278 = wp::where(var_1276, var_951, var_1277);
            // lly = wp.where(i % 2, ly, -ly)                                                 <L 803>
            var_1280 = wp::mod(var_1274, var_1279);
            var_1281 = wp::neg(var_953);
            var_1282 = wp::where(var_1280, var_953, var_1281);
            // x = llx - lp[0]                                                                <L 805>
            var_1284 = wp::extract(var_1022, var_1283);
            var_1285 = wp::sub(var_1278, var_1284);
            // y = lly - lp[1]                                                                <L 806>
            var_1287 = wp::extract(var_1022, var_1286);
            var_1288 = wp::sub(var_1282, var_1287);
            // u = (x * by - y * bx) * C                                                      <L 808>
            var_1289 = wp::mul(var_1285, var_1268);
            var_1290 = wp::mul(var_1288, var_1264);
            var_1291 = wp::sub(var_1289, var_1290);
            var_1292 = wp::mul(var_1291, var_1273);
            // v = (y * ax - x * ay) * C                                                      <L 809>
            var_1293 = wp::mul(var_1288, var_1262);
            var_1294 = wp::mul(var_1285, var_1266);
            var_1295 = wp::sub(var_1293, var_1294);
            var_1296 = wp::mul(var_1295, var_1273);
            // if u > 0 and v > 0 and u < 1 and v < 1:                                        <L 811>
            var_1299 = (var_1292 > var_1298);
            var_1297 = var_1299;
            if (var_1297) {
                var_1301 = (var_1296 > var_1300);
                var_1297 = var_1297 && var_1301;
            }
            if (var_1297) {
                var_1303 = (var_1292 < var_1302);
                var_1297 = var_1297 && var_1303;
            }
            if (var_1297) {
                var_1305 = (var_1296 < var_1304);
                var_1297 = var_1297 && var_1305;
            }
            if (var_1297) {
                // points[n] = wp.vec3(llx, lly, lp[2] + u * cn1[2] + v * cn2[2])             <L 812>
                var_1307 = wp::extract(var_1022, var_1306);
                var_1309 = wp::extract(var_1131, var_1308);
                var_1310 = wp::mul(var_1292, var_1309);
                var_1311 = wp::add(var_1307, var_1310);
                var_1313 = wp::extract(var_1132, var_1312);
                var_1314 = wp::mul(var_1296, var_1313);
                var_1315 = wp::add(var_1311, var_1314);
                var_1316 = wp::vec_t<3, wp::float32>(var_1278, var_1282, var_1315);
                wp::assign_inplace(var_931, var_1135, var_1316);
                // n += 1                                                                     <L 813>
                var_1318 = wp::add(var_1135, var_1317);
            }
            var_1319 = wp::where(var_1297, var_1318, var_1135);
            // llx = wp.where(i // 2, lx, -lx)                                                <L 802>
            var_1322 = wp::floordiv(var_1320, var_1321);
            var_1323 = wp::neg(var_951);
            var_1324 = wp::where(var_1322, var_951, var_1323);
            // lly = wp.where(i % 2, ly, -ly)                                                 <L 803>
            var_1326 = wp::mod(var_1320, var_1325);
            var_1327 = wp::neg(var_953);
            var_1328 = wp::where(var_1326, var_953, var_1327);
            // x = llx - lp[0]                                                                <L 805>
            var_1330 = wp::extract(var_1022, var_1329);
            var_1331 = wp::sub(var_1324, var_1330);
            // y = lly - lp[1]                                                                <L 806>
            var_1333 = wp::extract(var_1022, var_1332);
            var_1334 = wp::sub(var_1328, var_1333);
            // u = (x * by - y * bx) * C                                                      <L 808>
            var_1335 = wp::mul(var_1331, var_1268);
            var_1336 = wp::mul(var_1334, var_1264);
            var_1337 = wp::sub(var_1335, var_1336);
            var_1338 = wp::mul(var_1337, var_1273);
            // v = (y * ax - x * ay) * C                                                      <L 809>
            var_1339 = wp::mul(var_1334, var_1262);
            var_1340 = wp::mul(var_1331, var_1266);
            var_1341 = wp::sub(var_1339, var_1340);
            var_1342 = wp::mul(var_1341, var_1273);
            // if u > 0 and v > 0 and u < 1 and v < 1:                                        <L 811>
            var_1345 = (var_1338 > var_1344);
            var_1343 = var_1345;
            if (var_1343) {
                var_1347 = (var_1342 > var_1346);
                var_1343 = var_1343 && var_1347;
            }
            if (var_1343) {
                var_1349 = (var_1338 < var_1348);
                var_1343 = var_1343 && var_1349;
            }
            if (var_1343) {
                var_1351 = (var_1342 < var_1350);
                var_1343 = var_1343 && var_1351;
            }
            if (var_1343) {
                // points[n] = wp.vec3(llx, lly, lp[2] + u * cn1[2] + v * cn2[2])             <L 812>
                var_1353 = wp::extract(var_1022, var_1352);
                var_1355 = wp::extract(var_1131, var_1354);
                var_1356 = wp::mul(var_1338, var_1355);
                var_1357 = wp::add(var_1353, var_1356);
                var_1359 = wp::extract(var_1132, var_1358);
                var_1360 = wp::mul(var_1342, var_1359);
                var_1361 = wp::add(var_1357, var_1360);
                var_1362 = wp::vec_t<3, wp::float32>(var_1324, var_1328, var_1361);
                wp::assign_inplace(var_931, var_1319, var_1362);
                // n += 1                                                                     <L 813>
                var_1364 = wp::add(var_1319, var_1363);
            }
            var_1365 = wp::where(var_1343, var_1364, var_1319);
            // llx = wp.where(i // 2, lx, -lx)                                                <L 802>
            var_1368 = wp::floordiv(var_1366, var_1367);
            var_1369 = wp::neg(var_951);
            var_1370 = wp::where(var_1368, var_951, var_1369);
            // lly = wp.where(i % 2, ly, -ly)                                                 <L 803>
            var_1372 = wp::mod(var_1366, var_1371);
            var_1373 = wp::neg(var_953);
            var_1374 = wp::where(var_1372, var_953, var_1373);
            // x = llx - lp[0]                                                                <L 805>
            var_1376 = wp::extract(var_1022, var_1375);
            var_1377 = wp::sub(var_1370, var_1376);
            // y = lly - lp[1]                                                                <L 806>
            var_1379 = wp::extract(var_1022, var_1378);
            var_1380 = wp::sub(var_1374, var_1379);
            // u = (x * by - y * bx) * C                                                      <L 808>
            var_1381 = wp::mul(var_1377, var_1268);
            var_1382 = wp::mul(var_1380, var_1264);
            var_1383 = wp::sub(var_1381, var_1382);
            var_1384 = wp::mul(var_1383, var_1273);
            // v = (y * ax - x * ay) * C                                                      <L 809>
            var_1385 = wp::mul(var_1380, var_1262);
            var_1386 = wp::mul(var_1377, var_1266);
            var_1387 = wp::sub(var_1385, var_1386);
            var_1388 = wp::mul(var_1387, var_1273);
            // if u > 0 and v > 0 and u < 1 and v < 1:                                        <L 811>
            var_1391 = (var_1384 > var_1390);
            var_1389 = var_1391;
            if (var_1389) {
                var_1393 = (var_1388 > var_1392);
                var_1389 = var_1389 && var_1393;
            }
            if (var_1389) {
                var_1395 = (var_1384 < var_1394);
                var_1389 = var_1389 && var_1395;
            }
            if (var_1389) {
                var_1397 = (var_1388 < var_1396);
                var_1389 = var_1389 && var_1397;
            }
            if (var_1389) {
                // points[n] = wp.vec3(llx, lly, lp[2] + u * cn1[2] + v * cn2[2])             <L 812>
                var_1399 = wp::extract(var_1022, var_1398);
                var_1401 = wp::extract(var_1131, var_1400);
                var_1402 = wp::mul(var_1384, var_1401);
                var_1403 = wp::add(var_1399, var_1402);
                var_1405 = wp::extract(var_1132, var_1404);
                var_1406 = wp::mul(var_1388, var_1405);
                var_1407 = wp::add(var_1403, var_1406);
                var_1408 = wp::vec_t<3, wp::float32>(var_1370, var_1374, var_1407);
                wp::assign_inplace(var_931, var_1365, var_1408);
                // n += 1                                                                     <L 813>
                var_1410 = wp::add(var_1365, var_1409);
            }
            var_1411 = wp::where(var_1389, var_1410, var_1365);
            // llx = wp.where(i // 2, lx, -lx)                                                <L 802>
            var_1414 = wp::floordiv(var_1412, var_1413);
            var_1415 = wp::neg(var_951);
            var_1416 = wp::where(var_1414, var_951, var_1415);
            // lly = wp.where(i % 2, ly, -ly)                                                 <L 803>
            var_1418 = wp::mod(var_1412, var_1417);
            var_1419 = wp::neg(var_953);
            var_1420 = wp::where(var_1418, var_953, var_1419);
            // x = llx - lp[0]                                                                <L 805>
            var_1422 = wp::extract(var_1022, var_1421);
            var_1423 = wp::sub(var_1416, var_1422);
            // y = lly - lp[1]                                                                <L 806>
            var_1425 = wp::extract(var_1022, var_1424);
            var_1426 = wp::sub(var_1420, var_1425);
            // u = (x * by - y * bx) * C                                                      <L 808>
            var_1427 = wp::mul(var_1423, var_1268);
            var_1428 = wp::mul(var_1426, var_1264);
            var_1429 = wp::sub(var_1427, var_1428);
            var_1430 = wp::mul(var_1429, var_1273);
            // v = (y * ax - x * ay) * C                                                      <L 809>
            var_1431 = wp::mul(var_1426, var_1262);
            var_1432 = wp::mul(var_1423, var_1266);
            var_1433 = wp::sub(var_1431, var_1432);
            var_1434 = wp::mul(var_1433, var_1273);
            // if u > 0 and v > 0 and u < 1 and v < 1:                                        <L 811>
            var_1437 = (var_1430 > var_1436);
            var_1435 = var_1437;
            if (var_1435) {
                var_1439 = (var_1434 > var_1438);
                var_1435 = var_1435 && var_1439;
            }
            if (var_1435) {
                var_1441 = (var_1430 < var_1440);
                var_1435 = var_1435 && var_1441;
            }
            if (var_1435) {
                var_1443 = (var_1434 < var_1442);
                var_1435 = var_1435 && var_1443;
            }
            if (var_1435) {
                // points[n] = wp.vec3(llx, lly, lp[2] + u * cn1[2] + v * cn2[2])             <L 812>
                var_1445 = wp::extract(var_1022, var_1444);
                var_1447 = wp::extract(var_1131, var_1446);
                var_1448 = wp::mul(var_1430, var_1447);
                var_1449 = wp::add(var_1445, var_1448);
                var_1451 = wp::extract(var_1132, var_1450);
                var_1452 = wp::mul(var_1434, var_1451);
                var_1453 = wp::add(var_1449, var_1452);
                var_1454 = wp::vec_t<3, wp::float32>(var_1416, var_1420, var_1453);
                wp::assign_inplace(var_931, var_1411, var_1454);
                // n += 1                                                                     <L 813>
                var_1456 = wp::add(var_1411, var_1455);
            }
            var_1457 = wp::where(var_1435, var_1456, var_1411);
        }
        var_1458 = wp::where(var_1260, var_1412, var_1137);
        var_1459 = wp::where(var_1260, var_1457, var_1135);
        // for i in range(1 << dirs):                                                         <L 815>
        var_1461 = wp::lshift(var_1460, var_1130);
        var_1462 = wp::range(var_1461);
        start_for_19:;
            if (iter_cmp(var_1462) == 0) goto end_for_19;
            var_1463 = wp::iter_next(var_1462);
            // tmpv = lp + wp.float32(i & 1) * cn1 + wp.float32((i & 2) != 0) * cn2           <L 816>
            var_1465 = wp::bit_and(var_1463, var_1464);
            var_1466 = wp::float32(var_1465);
            var_1467 = wp::mul(var_1466, var_1131);
            var_1468 = wp::add(var_1022, var_1467);
            var_1470 = wp::bit_and(var_1463, var_1469);
            var_1472 = (var_1470 != var_1471);
            var_1473 = wp::float32(var_1472);
            var_1474 = wp::mul(var_1473, var_1132);
            var_1475 = wp::add(var_1468, var_1474);
            // if tmpv[0] > -lx and tmpv[0] < lx and tmpv[1] > -ly and tmpv[1] < ly:          <L 817>
            var_1478 = wp::extract(var_1475, var_1477);
            var_1479 = wp::neg(var_951);
            var_1480 = (var_1478 > var_1479);
            var_1476 = var_1480;
            if (var_1476) {
                var_1482 = wp::extract(var_1475, var_1481);
                var_1483 = (var_1482 < var_951);
                var_1476 = var_1476 && var_1483;
            }
            if (var_1476) {
                var_1485 = wp::extract(var_1475, var_1484);
                var_1486 = wp::neg(var_953);
                var_1487 = (var_1485 > var_1486);
                var_1476 = var_1476 && var_1487;
            }
            if (var_1476) {
                var_1489 = wp::extract(var_1475, var_1488);
                var_1490 = (var_1489 < var_953);
                var_1476 = var_1476 && var_1490;
            }
            if (var_1476) {
                // points[n] = tmpv                                                           <L 818>
                wp::assign_inplace(var_931, var_1459, var_1475);
                // n += 1                                                                     <L 819>
                var_1492 = wp::add(var_1459, var_1491);
            }
            var_1493 = wp::where(var_1476, var_1492, var_1459);
            wp::assign(var_1459, var_1493);
            goto start_for_19;
        end_for_19:;
        // m = n                                                                              <L 821>
        var_1494 = wp::copy(var_1459);
        // n = wp.int32(0)                                                                    <L 822>
        var_1496 = wp::int32(var_1495);
        // for i in range(m):                                                                 <L 824>
        var_1497 = wp::range(var_1494);
        start_for_21:;
            if (iter_cmp(var_1497) == 0) goto end_for_21;
            var_1498 = wp::iter_next(var_1497);
            // if points[i][2] > margin:                                                      <L 825>
            var_1500 = wp::extract(var_931, var_1498, var_1499);
            var_1501 = (var_1500 > var_margin);
            if (var_1501) {
                // continue                                                                   <L 826>
                goto start_for_21;
            }
            // if i != n:                                                                     <L 827>
            var_1502 = (var_1498 != var_1496);
            if (var_1502) {
                // points[n] = points[i]                                                      <L 828>
                var_1503 = wp::extract(var_931, var_1498);
                wp::assign_inplace(var_931, var_1496, var_1503);
            }
            // depth[n] = points[n, 2]                                                        <L 830>
            var_1505 = wp::extract(var_931, var_1496, var_1504);
            wp::assign_inplace(var_932, var_1496, var_1505);
            // points[n, 2] *= 0.5                                                            <L 831>
            var_1508 = wp::extract(var_931, var_1496, var_1507);
            var_1509 = wp::mul(var_1508, var_1506);
            wp::assign_inplace(var_931, var_1496, var_1507, var_1509);
            // n += 1                                                                         <L 832>
            var_1511 = wp::add(var_1496, var_1510);
            wp::assign(var_1496, var_1511);
            goto start_for_21;
        end_for_21:;
        // rw = wp.where(box_idx, box2_rot, box1_rot) @ wp.transpose(rotmore)                 <L 835>
        var_1512 = wp::where(var_939, var_box2_rot, var_box1_rot);
        var_1513 = wp::transpose(var_940);
        var_1514 = wp::mul(var_1512, var_1513);
        // pw = wp.where(box_idx, box2_pos, box1_pos)                                         <L 836>
        var_1515 = wp::where(var_939, var_box2_pos, var_box1_pos);
        // normal = wp.where(box_idx, -1.0, 1.0) * wp.transpose(rw)[2]                        <L 837>
        var_1518 = wp::where(var_939, var_1516, var_1517);
        var_1519 = wp::transpose(var_1514);
        var_1521 = wp::extract(var_1519, var_1520);
        var_1522 = wp::mul(var_1518, var_1521);
    }
    var_1523 = wp::where(var_935, var_1498, var_684);
    var_1524 = wp::where(var_935, var_1133, var_928);
    if (!var_935) {
        // edge1 = (axis_code - 12) // 3                                                      <L 841>
        var_1526 = wp::sub(var_187, var_1525);
        var_1528 = wp::floordiv(var_1526, var_1527);
        // edge2 = (axis_code - 12) % 3                                                       <L 842>
        var_1530 = wp::sub(var_187, var_1529);
        var_1532 = wp::mod(var_1530, var_1531);
        // ax1 = wp.int(1 - (edge2 & 1))                                                      <L 845>
        var_1535 = wp::bit_and(var_1532, var_1534);
        var_1536 = wp::sub(var_1533, var_1535);
        var_1537 = wp::int(var_1536);
        // ax2 = wp.int(2 - (edge2 & 2))                                                      <L 846>
        var_1540 = wp::bit_and(var_1532, var_1539);
        var_1541 = wp::sub(var_1538, var_1540);
        var_1542 = wp::int(var_1541);
        // pax1 = wp.int(1 - (edge1 & 1))                                                     <L 848>
        var_1545 = wp::bit_and(var_1528, var_1544);
        var_1546 = wp::sub(var_1543, var_1545);
        var_1547 = wp::int(var_1546);
        // pax2 = wp.int(2 - (edge1 & 2))                                                     <L 849>
        var_1550 = wp::bit_and(var_1528, var_1549);
        var_1551 = wp::sub(var_1548, var_1550);
        var_1552 = wp::int(var_1551);
        // if rot21abs[edge1, ax1] < rot21abs[edge1, ax2]:                                    <L 851>
        var_1553 = wp::extract(var_31, var_1528, var_1537);
        var_1554 = wp::extract(var_31, var_1528, var_1542);
        var_1555 = (var_1553 < var_1554);
        if (var_1555) {
            // ax1, ax2 = ax2, ax1                                                            <L 852>
        }
        var_1556 = wp::where(var_1555, var_1542, var_1537);
        var_1557 = wp::where(var_1555, var_1537, var_1542);
        // if rot12abs[edge2, pax1] < rot12abs[edge2, pax2]:                                  <L 854>
        var_1558 = wp::extract(var_32, var_1532, var_1547);
        var_1559 = wp::extract(var_32, var_1532, var_1552);
        var_1560 = (var_1558 < var_1559);
        if (var_1560) {
            // pax1, pax2 = pax2, pax1                                                        <L 855>
        }
        var_1561 = wp::where(var_1560, var_1552, var_1547);
        var_1562 = wp::where(var_1560, var_1547, var_1552);
        // rotmore = _compute_rotmore(wp.where(cle1 & (1 << pax2), pax2, pax2 + 3))           <L 857>
        var_1564 = wp::lshift(var_1563, var_1562);
        var_1565 = wp::bit_and(var_193, var_1564);
        var_1567 = wp::add(var_1562, var_1566);
        var_1568 = wp::where(var_1565, var_1562, var_1567);
        var_1569 = _compute_rotmore_0(var_1568);
        // p = rotmore @ pos21                                                                <L 860>
        var_1570 = wp::mul(var_1569, var_15);
        // rnorm = rotmore @ clnorm                                                           <L 861>
        var_1571 = wp::mul(var_1569, var_189);
        // r = rotmore @ rot21                                                                <L 862>
        var_1572 = wp::mul(var_1569, var_20);
        // rt = wp.transpose(r)                                                               <L 863>
        var_1573 = wp::transpose(var_1572);
        // s = wp.abs(wp.transpose(rotmore) @ box1_size)                                      <L 864>
        var_1574 = wp::transpose(var_1569);
        var_1575 = wp::mul(var_1574, var_box1_size);
        var_1576 = wp::abs(var_1575);
        // lx, ly, hz = s[0], s[1], s[2]                                                      <L 866>
        var_1578 = wp::extract(var_1576, var_1577);
        var_1580 = wp::extract(var_1576, var_1579);
        var_1582 = wp::extract(var_1576, var_1581);
        // p[2] -= hz                                                                         <L 867>
        wp::sub_inplace(var_1570, var_1583, var_1582);
        // points[0] = (                                                                      <L 871>
        // p                                                                                  <L 872>
        // + rt[ax1] * box2_size[ax1] * wp.where(cle2 & (1 << ax1), 1.0, -1.0)                <L 873>
        var_1584 = wp::extract(var_1573, var_1556);
        var_1585 = wp::extract(var_box2_size, var_1556);
        var_1586 = wp::mul(var_1584, var_1585);
        var_1588 = wp::lshift(var_1587, var_1556);
        var_1589 = wp::bit_and(var_195, var_1588);
        var_1592 = wp::where(var_1589, var_1590, var_1591);
        var_1593 = wp::mul(var_1586, var_1592);
        var_1594 = wp::add(var_1570, var_1593);
        // + rt[ax2] * box2_size[ax2] * wp.where(cle2 & (1 << ax2), 1.0, -1.0)                <L 874>
        var_1595 = wp::extract(var_1573, var_1557);
        var_1596 = wp::extract(var_box2_size, var_1557);
        var_1597 = wp::mul(var_1595, var_1596);
        var_1599 = wp::lshift(var_1598, var_1557);
        var_1600 = wp::bit_and(var_195, var_1599);
        var_1603 = wp::where(var_1600, var_1601, var_1602);
        var_1604 = wp::mul(var_1597, var_1603);
        var_1605 = wp::add(var_1594, var_1604);
        // points[0] = (                                                                      <L 871>
        wp::assign_inplace(var_931, var_1606, var_1605);
        // points[1] = points[0] - rt[edge2] * box2_size[edge2]                               <L 876>
        var_1608 = wp::extract(var_931, var_1607);
        var_1609 = wp::extract(var_1573, var_1532);
        var_1610 = wp::extract(var_box2_size, var_1532);
        var_1611 = wp::mul(var_1609, var_1610);
        var_1612 = wp::sub(var_1608, var_1611);
        wp::assign_inplace(var_931, var_1613, var_1612);
        // points[0] += rt[edge2] * box2_size[edge2]                                          <L 877>
        var_1614 = wp::extract(var_1573, var_1532);
        var_1615 = wp::extract(var_box2_size, var_1532);
        var_1616 = wp::mul(var_1614, var_1615);
        wp::add_inplace(var_931, var_1617, var_1616);
        // points[2] = (                                                                      <L 879>
        // p                                                                                  <L 880>
        // + rt[ax1] * box2_size[ax1] * wp.where(cle2 & (1 << ax1), -1.0, 1.0)                <L 881>
        var_1618 = wp::extract(var_1573, var_1556);
        var_1619 = wp::extract(var_box2_size, var_1556);
        var_1620 = wp::mul(var_1618, var_1619);
        var_1622 = wp::lshift(var_1621, var_1556);
        var_1623 = wp::bit_and(var_195, var_1622);
        var_1626 = wp::where(var_1623, var_1624, var_1625);
        var_1627 = wp::mul(var_1620, var_1626);
        var_1628 = wp::add(var_1570, var_1627);
        // + rt[ax2] * box2_size[ax2] * wp.where(cle2 & (1 << ax2), 1.0, -1.0)                <L 882>
        var_1629 = wp::extract(var_1573, var_1557);
        var_1630 = wp::extract(var_box2_size, var_1557);
        var_1631 = wp::mul(var_1629, var_1630);
        var_1633 = wp::lshift(var_1632, var_1557);
        var_1634 = wp::bit_and(var_195, var_1633);
        var_1637 = wp::where(var_1634, var_1635, var_1636);
        var_1638 = wp::mul(var_1631, var_1637);
        var_1639 = wp::add(var_1628, var_1638);
        // points[2] = (                                                                      <L 879>
        wp::assign_inplace(var_931, var_1640, var_1639);
        // points[3] = points[2] - rt[edge2] * box2_size[edge2]                               <L 885>
        var_1642 = wp::extract(var_931, var_1641);
        var_1643 = wp::extract(var_1573, var_1532);
        var_1644 = wp::extract(var_box2_size, var_1532);
        var_1645 = wp::mul(var_1643, var_1644);
        var_1646 = wp::sub(var_1642, var_1645);
        wp::assign_inplace(var_931, var_1647, var_1646);
        // points[2] += rt[edge2] * box2_size[edge2]                                          <L 886>
        var_1648 = wp::extract(var_1573, var_1532);
        var_1649 = wp::extract(var_box2_size, var_1532);
        var_1650 = wp::mul(var_1648, var_1649);
        wp::add_inplace(var_931, var_1651, var_1650);
        // n = 4                                                                              <L 888>
        // axi_lp = points[0]                                                                 <L 891>
        var_1654 = wp::extract(var_931, var_1653);
        // axi_cn1 = points[1] - points[0]                                                    <L 892>
        var_1656 = wp::extract(var_931, var_1655);
        var_1658 = wp::extract(var_931, var_1657);
        var_1659 = wp::sub(var_1656, var_1658);
        // axi_cn2 = points[2] - points[0]                                                    <L 893>
        var_1661 = wp::extract(var_931, var_1660);
        var_1663 = wp::extract(var_931, var_1662);
        var_1664 = wp::sub(var_1661, var_1663);
        // if wp.abs(rnorm[2]) < MJ_MINVAL:                                                   <L 896>
        var_1666 = wp::extract(var_1571, var_1665);
        var_1667 = wp::abs(var_1666);
        var_1668 = (var_1667 < var_229);
        if (var_1668) {
            // return contact_dist, contact_pos, contact_normals  # Shouldn't happen          <L 897>
            ret_0 = var_0;
            ret_1 = var_10;
            ret_2 = var_11;
            return;
        }
        // innorm = wp.where(inv, -1.0, 1.0) / rnorm[2]                                       <L 900>
        var_1671 = wp::where(var_191, var_1669, var_1670);
        var_1673 = wp::extract(var_1571, var_1672);
        var_1674 = wp::div(var_1671, var_1673);
        // pu = mat43f()                                                                      <L 902>
        var_1675 = wp::mat_t<4, 3, wp::float32>();
        // for i in range(4):                                                                 <L 905>
        // pu[i] = points[i]                                                                  <L 906>
        var_1677 = wp::extract(var_931, var_1676);
        wp::assign_inplace(var_1675, var_1676, var_1677);
        // c_scl = points[i, 2] * wp.where(inv, -1.0, 1.0) * innorm                           <L 907>
        var_1679 = wp::extract(var_931, var_1676, var_1678);
        var_1682 = wp::where(var_191, var_1680, var_1681);
        var_1683 = wp::mul(var_1679, var_1682);
        var_1684 = wp::mul(var_1683, var_1674);
        // points[i] -= rnorm * c_scl                                                         <L 908>
        var_1685 = wp::mul(var_1571, var_1684);
        wp::sub_inplace(var_931, var_1676, var_1685);
        // pu[i] = points[i]                                                                  <L 906>
        var_1687 = wp::extract(var_931, var_1686);
        wp::assign_inplace(var_1675, var_1686, var_1687);
        // c_scl = points[i, 2] * wp.where(inv, -1.0, 1.0) * innorm                           <L 907>
        var_1689 = wp::extract(var_931, var_1686, var_1688);
        var_1692 = wp::where(var_191, var_1690, var_1691);
        var_1693 = wp::mul(var_1689, var_1692);
        var_1694 = wp::mul(var_1693, var_1674);
        // points[i] -= rnorm * c_scl                                                         <L 908>
        var_1695 = wp::mul(var_1571, var_1694);
        wp::sub_inplace(var_931, var_1686, var_1695);
        // pu[i] = points[i]                                                                  <L 906>
        var_1697 = wp::extract(var_931, var_1696);
        wp::assign_inplace(var_1675, var_1696, var_1697);
        // c_scl = points[i, 2] * wp.where(inv, -1.0, 1.0) * innorm                           <L 907>
        var_1699 = wp::extract(var_931, var_1696, var_1698);
        var_1702 = wp::where(var_191, var_1700, var_1701);
        var_1703 = wp::mul(var_1699, var_1702);
        var_1704 = wp::mul(var_1703, var_1674);
        // points[i] -= rnorm * c_scl                                                         <L 908>
        var_1705 = wp::mul(var_1571, var_1704);
        wp::sub_inplace(var_931, var_1696, var_1705);
        // pu[i] = points[i]                                                                  <L 906>
        var_1707 = wp::extract(var_931, var_1706);
        wp::assign_inplace(var_1675, var_1706, var_1707);
        // c_scl = points[i, 2] * wp.where(inv, -1.0, 1.0) * innorm                           <L 907>
        var_1709 = wp::extract(var_931, var_1706, var_1708);
        var_1712 = wp::where(var_191, var_1710, var_1711);
        var_1713 = wp::mul(var_1709, var_1712);
        var_1714 = wp::mul(var_1713, var_1674);
        // points[i] -= rnorm * c_scl                                                         <L 908>
        var_1715 = wp::mul(var_1571, var_1714);
        wp::sub_inplace(var_931, var_1706, var_1715);
        // pts_lp = points[0]                                                                 <L 910>
        var_1717 = wp::extract(var_931, var_1716);
        // pts_cn1 = points[1] - points[0]                                                    <L 911>
        var_1719 = wp::extract(var_931, var_1718);
        var_1721 = wp::extract(var_931, var_1720);
        var_1722 = wp::sub(var_1719, var_1721);
        // pts_cn2 = points[2] - points[0]                                                    <L 912>
        var_1724 = wp::extract(var_931, var_1723);
        var_1726 = wp::extract(var_931, var_1725);
        var_1727 = wp::sub(var_1724, var_1726);
        // n = wp.int32(0)                                                                    <L 914>
        var_1729 = wp::int32(var_1728);
        // for i in range(4):                                                                 <L 916>
        // for q in range(2):                                                                 <L 917>
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_1732 = wp::extract(var_1717, var_1731);
        var_1734 = (var_1730 < var_1733);
        var_1737 = (var_1730 == var_1736);
        var_1738 = wp::extract(var_1722, var_1731);
        var_1739 = wp::extract(var_1727, var_1731);
        var_1740 = wp::where(var_1737, var_1738, var_1739);
        var_1741 = wp::where(var_1734, var_1735, var_1740);
        var_1742 = wp::add(var_1732, var_1741);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_1745 = (var_1730 == var_1744);
        var_1743 = var_1745;
        if (!var_1743) {
            var_1747 = (var_1730 == var_1746);
            var_1743 = var_1743 || var_1747;
        }
        var_1748 = wp::extract(var_1722, var_1731);
        var_1749 = wp::extract(var_1727, var_1731);
        var_1750 = wp::where(var_1743, var_1748, var_1749);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_1752 = wp::sub(var_1751, var_1731);
        var_1753 = wp::extract(var_1717, var_1752);
        var_1755 = (var_1730 < var_1754);
        var_1758 = (var_1730 == var_1757);
        var_1760 = wp::sub(var_1759, var_1731);
        var_1761 = wp::extract(var_1722, var_1760);
        var_1763 = wp::sub(var_1762, var_1731);
        var_1764 = wp::extract(var_1727, var_1763);
        var_1765 = wp::where(var_1758, var_1761, var_1764);
        var_1766 = wp::where(var_1755, var_1756, var_1765);
        var_1767 = wp::add(var_1753, var_1766);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_1770 = (var_1730 == var_1769);
        var_1768 = var_1770;
        if (!var_1768) {
            var_1772 = (var_1730 == var_1771);
            var_1768 = var_1768 || var_1772;
        }
        var_1774 = wp::sub(var_1773, var_1731);
        var_1775 = wp::extract(var_1722, var_1774);
        var_1777 = wp::sub(var_1776, var_1731);
        var_1778 = wp::extract(var_1727, var_1777);
        var_1779 = wp::where(var_1768, var_1775, var_1778);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_1781 = (var_1730 < var_1780);
        var_1783 = wp::vec_t<3, wp::float32>(var_1782);
        var_1785 = (var_1730 == var_1784);
        var_1786 = wp::where(var_1785, var_1659, var_1664);
        var_1787 = wp::where(var_1781, var_1783, var_1786);
        var_1788 = wp::add(var_1654, var_1787);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_1791 = (var_1730 == var_1790);
        var_1789 = var_1791;
        if (!var_1789) {
            var_1793 = (var_1730 == var_1792);
            var_1789 = var_1789 || var_1793;
        }
        var_1794 = wp::where(var_1789, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_1795 = wp::abs(var_1750);
        var_1796 = (var_1795 > var_229);
        if (var_1796) {
            // br = 1.0 / lb                                                                  <L 928>
            var_1798 = wp::div(var_1797, var_1750);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_1802 = wp::range(var_1799, var_1800, var_1801);
            start_for_24:;
                if (iter_cmp(var_1802) == 0) goto end_for_24;
                var_1803 = wp::iter_next(var_1802);
                // if n == max_con_pair:                                                      <L 930>
                var_1804 = (var_1729 == var_933);
                if (var_1804) {
                    // break                                                                  <L 931>
                    goto end_for_24;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_1805 = wp::extract(var_1576, var_1731);
                var_1806 = wp::float32(var_1803);
                var_1807 = wp::mul(var_1805, var_1806);
                // c1 = (l - la) * br                                                         <L 933>
                var_1808 = wp::sub(var_1807, var_1742);
                var_1809 = wp::mul(var_1808, var_1798);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_1812 = (var_1809 < var_1811);
                var_1810 = var_1812;
                if (!var_1810) {
                    var_1814 = (var_1809 > var_1813);
                    var_1810 = var_1810 || var_1814;
                }
                if (var_1810) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_1809);
                    wp::assign(var_1167, var_1807);
                    goto start_for_24;
                }
                var_1815 = wp::where(var_1810, var_149, var_1809);
                var_1816 = wp::where(var_1810, var_1167, var_1807);
                // c2 = lc + ld * c1                                                          <L 936>
                var_1817 = wp::mul(var_1779, var_1815);
                var_1818 = wp::add(var_1767, var_1817);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_1819 = wp::abs(var_1818);
                var_1821 = wp::sub(var_1820, var_1731);
                var_1822 = wp::extract(var_1576, var_1821);
                var_1823 = (var_1819 > var_1822);
                if (var_1823) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_1815);
                    wp::assign(var_156, var_1818);
                    wp::assign(var_1167, var_1816);
                    goto start_for_24;
                }
                var_1824 = wp::where(var_1823, var_149, var_1815);
                var_1825 = wp::where(var_1823, var_156, var_1818);
                var_1826 = wp::where(var_1823, var_1167, var_1816);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_1828 = wp::extract(var_1788, var_1827);
                var_1830 = wp::extract(var_1794, var_1829);
                var_1831 = wp::mul(var_1830, var_1824);
                var_1832 = wp::add(var_1828, var_1831);
                var_1833 = wp::mul(var_1832, var_1674);
                var_1834 = (var_1833 > var_margin);
                if (var_1834) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_1824);
                    wp::assign(var_156, var_1825);
                    wp::assign(var_1167, var_1826);
                    goto start_for_24;
                }
                var_1835 = wp::where(var_1834, var_149, var_1824);
                var_1836 = wp::where(var_1834, var_156, var_1825);
                var_1837 = wp::where(var_1834, var_1167, var_1826);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_1839 = wp::mul(var_1788, var_1838);
                var_1840 = wp::mul(var_1835, var_1794);
                var_1842 = wp::mul(var_1840, var_1841);
                var_1843 = wp::add(var_1839, var_1842);
                wp::assign_inplace(var_931, var_1729, var_1843);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_1845 = wp::mul(var_1844, var_1837);
                wp::add_inplace(var_931, var_1729, var_1731, var_1845);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_1847 = wp::mul(var_1846, var_1836);
                var_1849 = wp::sub(var_1848, var_1731);
                wp::add_inplace(var_931, var_1729, var_1849, var_1847);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_1851 = wp::extract(var_931, var_1729, var_1850);
                var_1852 = wp::mul(var_1851, var_1674);
                var_1854 = wp::mul(var_1852, var_1853);
                wp::assign_inplace(var_932, var_1729, var_1854);
                // n += 1                                                                     <L 946>
                var_1856 = wp::add(var_1729, var_1855);
                wp::assign(var_149, var_1835);
                wp::assign(var_156, var_1836);
                wp::assign(var_1729, var_1856);
                wp::assign(var_1167, var_1837);
                goto start_for_24;
            end_for_24:;
        }
        var_1857 = wp::where(var_1796, var_1803, var_687);
        var_1858 = wp::where(var_1796, var_1798, var_1258);
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_1860 = wp::extract(var_1717, var_1859);
        var_1862 = (var_1730 < var_1861);
        var_1865 = (var_1730 == var_1864);
        var_1866 = wp::extract(var_1722, var_1859);
        var_1867 = wp::extract(var_1727, var_1859);
        var_1868 = wp::where(var_1865, var_1866, var_1867);
        var_1869 = wp::where(var_1862, var_1863, var_1868);
        var_1870 = wp::add(var_1860, var_1869);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_1873 = (var_1730 == var_1872);
        var_1871 = var_1873;
        if (!var_1871) {
            var_1875 = (var_1730 == var_1874);
            var_1871 = var_1871 || var_1875;
        }
        var_1876 = wp::extract(var_1722, var_1859);
        var_1877 = wp::extract(var_1727, var_1859);
        var_1878 = wp::where(var_1871, var_1876, var_1877);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_1880 = wp::sub(var_1879, var_1859);
        var_1881 = wp::extract(var_1717, var_1880);
        var_1883 = (var_1730 < var_1882);
        var_1886 = (var_1730 == var_1885);
        var_1888 = wp::sub(var_1887, var_1859);
        var_1889 = wp::extract(var_1722, var_1888);
        var_1891 = wp::sub(var_1890, var_1859);
        var_1892 = wp::extract(var_1727, var_1891);
        var_1893 = wp::where(var_1886, var_1889, var_1892);
        var_1894 = wp::where(var_1883, var_1884, var_1893);
        var_1895 = wp::add(var_1881, var_1894);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_1898 = (var_1730 == var_1897);
        var_1896 = var_1898;
        if (!var_1896) {
            var_1900 = (var_1730 == var_1899);
            var_1896 = var_1896 || var_1900;
        }
        var_1902 = wp::sub(var_1901, var_1859);
        var_1903 = wp::extract(var_1722, var_1902);
        var_1905 = wp::sub(var_1904, var_1859);
        var_1906 = wp::extract(var_1727, var_1905);
        var_1907 = wp::where(var_1896, var_1903, var_1906);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_1909 = (var_1730 < var_1908);
        var_1911 = wp::vec_t<3, wp::float32>(var_1910);
        var_1913 = (var_1730 == var_1912);
        var_1914 = wp::where(var_1913, var_1659, var_1664);
        var_1915 = wp::where(var_1909, var_1911, var_1914);
        var_1916 = wp::add(var_1654, var_1915);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_1919 = (var_1730 == var_1918);
        var_1917 = var_1919;
        if (!var_1917) {
            var_1921 = (var_1730 == var_1920);
            var_1917 = var_1917 || var_1921;
        }
        var_1922 = wp::where(var_1917, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_1923 = wp::abs(var_1878);
        var_1924 = (var_1923 > var_229);
        if (var_1924) {
            // br = 1.0 / lb                                                                  <L 928>
            var_1926 = wp::div(var_1925, var_1878);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_1930 = wp::range(var_1927, var_1928, var_1929);
            start_for_26:;
                if (iter_cmp(var_1930) == 0) goto end_for_26;
                var_1931 = wp::iter_next(var_1930);
                // if n == max_con_pair:                                                      <L 930>
                var_1932 = (var_1729 == var_933);
                if (var_1932) {
                    // break                                                                  <L 931>
                    goto end_for_26;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_1933 = wp::extract(var_1576, var_1859);
                var_1934 = wp::float32(var_1931);
                var_1935 = wp::mul(var_1933, var_1934);
                // c1 = (l - la) * br                                                         <L 933>
                var_1936 = wp::sub(var_1935, var_1870);
                var_1937 = wp::mul(var_1936, var_1926);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_1940 = (var_1937 < var_1939);
                var_1938 = var_1940;
                if (!var_1938) {
                    var_1942 = (var_1937 > var_1941);
                    var_1938 = var_1938 || var_1942;
                }
                if (var_1938) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_1937);
                    wp::assign(var_1167, var_1935);
                    goto start_for_26;
                }
                var_1943 = wp::where(var_1938, var_149, var_1937);
                var_1944 = wp::where(var_1938, var_1167, var_1935);
                // c2 = lc + ld * c1                                                          <L 936>
                var_1945 = wp::mul(var_1907, var_1943);
                var_1946 = wp::add(var_1895, var_1945);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_1947 = wp::abs(var_1946);
                var_1949 = wp::sub(var_1948, var_1859);
                var_1950 = wp::extract(var_1576, var_1949);
                var_1951 = (var_1947 > var_1950);
                if (var_1951) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_1943);
                    wp::assign(var_156, var_1946);
                    wp::assign(var_1167, var_1944);
                    goto start_for_26;
                }
                var_1952 = wp::where(var_1951, var_149, var_1943);
                var_1953 = wp::where(var_1951, var_156, var_1946);
                var_1954 = wp::where(var_1951, var_1167, var_1944);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_1956 = wp::extract(var_1916, var_1955);
                var_1958 = wp::extract(var_1922, var_1957);
                var_1959 = wp::mul(var_1958, var_1952);
                var_1960 = wp::add(var_1956, var_1959);
                var_1961 = wp::mul(var_1960, var_1674);
                var_1962 = (var_1961 > var_margin);
                if (var_1962) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_1952);
                    wp::assign(var_156, var_1953);
                    wp::assign(var_1167, var_1954);
                    goto start_for_26;
                }
                var_1963 = wp::where(var_1962, var_149, var_1952);
                var_1964 = wp::where(var_1962, var_156, var_1953);
                var_1965 = wp::where(var_1962, var_1167, var_1954);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_1967 = wp::mul(var_1916, var_1966);
                var_1968 = wp::mul(var_1963, var_1922);
                var_1970 = wp::mul(var_1968, var_1969);
                var_1971 = wp::add(var_1967, var_1970);
                wp::assign_inplace(var_931, var_1729, var_1971);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_1973 = wp::mul(var_1972, var_1965);
                wp::add_inplace(var_931, var_1729, var_1859, var_1973);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_1975 = wp::mul(var_1974, var_1964);
                var_1977 = wp::sub(var_1976, var_1859);
                wp::add_inplace(var_931, var_1729, var_1977, var_1975);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_1979 = wp::extract(var_931, var_1729, var_1978);
                var_1980 = wp::mul(var_1979, var_1674);
                var_1982 = wp::mul(var_1980, var_1981);
                wp::assign_inplace(var_932, var_1729, var_1982);
                // n += 1                                                                     <L 946>
                var_1984 = wp::add(var_1729, var_1983);
                wp::assign(var_149, var_1963);
                wp::assign(var_156, var_1964);
                wp::assign(var_1729, var_1984);
                wp::assign(var_1167, var_1965);
                goto start_for_26;
            end_for_26:;
        }
        var_1985 = wp::where(var_1924, var_1931, var_1857);
        var_1986 = wp::where(var_1924, var_1926, var_1858);
        // for q in range(2):                                                                 <L 917>
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_1989 = wp::extract(var_1717, var_1988);
        var_1991 = (var_1987 < var_1990);
        var_1994 = (var_1987 == var_1993);
        var_1995 = wp::extract(var_1722, var_1988);
        var_1996 = wp::extract(var_1727, var_1988);
        var_1997 = wp::where(var_1994, var_1995, var_1996);
        var_1998 = wp::where(var_1991, var_1992, var_1997);
        var_1999 = wp::add(var_1989, var_1998);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2002 = (var_1987 == var_2001);
        var_2000 = var_2002;
        if (!var_2000) {
            var_2004 = (var_1987 == var_2003);
            var_2000 = var_2000 || var_2004;
        }
        var_2005 = wp::extract(var_1722, var_1988);
        var_2006 = wp::extract(var_1727, var_1988);
        var_2007 = wp::where(var_2000, var_2005, var_2006);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2009 = wp::sub(var_2008, var_1988);
        var_2010 = wp::extract(var_1717, var_2009);
        var_2012 = (var_1987 < var_2011);
        var_2015 = (var_1987 == var_2014);
        var_2017 = wp::sub(var_2016, var_1988);
        var_2018 = wp::extract(var_1722, var_2017);
        var_2020 = wp::sub(var_2019, var_1988);
        var_2021 = wp::extract(var_1727, var_2020);
        var_2022 = wp::where(var_2015, var_2018, var_2021);
        var_2023 = wp::where(var_2012, var_2013, var_2022);
        var_2024 = wp::add(var_2010, var_2023);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2027 = (var_1987 == var_2026);
        var_2025 = var_2027;
        if (!var_2025) {
            var_2029 = (var_1987 == var_2028);
            var_2025 = var_2025 || var_2029;
        }
        var_2031 = wp::sub(var_2030, var_1988);
        var_2032 = wp::extract(var_1722, var_2031);
        var_2034 = wp::sub(var_2033, var_1988);
        var_2035 = wp::extract(var_1727, var_2034);
        var_2036 = wp::where(var_2025, var_2032, var_2035);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2038 = (var_1987 < var_2037);
        var_2040 = wp::vec_t<3, wp::float32>(var_2039);
        var_2042 = (var_1987 == var_2041);
        var_2043 = wp::where(var_2042, var_1659, var_1664);
        var_2044 = wp::where(var_2038, var_2040, var_2043);
        var_2045 = wp::add(var_1654, var_2044);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2048 = (var_1987 == var_2047);
        var_2046 = var_2048;
        if (!var_2046) {
            var_2050 = (var_1987 == var_2049);
            var_2046 = var_2046 || var_2050;
        }
        var_2051 = wp::where(var_2046, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2052 = wp::abs(var_2007);
        var_2053 = (var_2052 > var_229);
        if (var_2053) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2055 = wp::div(var_2054, var_2007);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2059 = wp::range(var_2056, var_2057, var_2058);
            start_for_28:;
                if (iter_cmp(var_2059) == 0) goto end_for_28;
                var_2060 = wp::iter_next(var_2059);
                // if n == max_con_pair:                                                      <L 930>
                var_2061 = (var_1729 == var_933);
                if (var_2061) {
                    // break                                                                  <L 931>
                    goto end_for_28;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2062 = wp::extract(var_1576, var_1988);
                var_2063 = wp::float32(var_2060);
                var_2064 = wp::mul(var_2062, var_2063);
                // c1 = (l - la) * br                                                         <L 933>
                var_2065 = wp::sub(var_2064, var_1999);
                var_2066 = wp::mul(var_2065, var_2055);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2069 = (var_2066 < var_2068);
                var_2067 = var_2069;
                if (!var_2067) {
                    var_2071 = (var_2066 > var_2070);
                    var_2067 = var_2067 || var_2071;
                }
                if (var_2067) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2066);
                    wp::assign(var_1167, var_2064);
                    goto start_for_28;
                }
                var_2072 = wp::where(var_2067, var_149, var_2066);
                var_2073 = wp::where(var_2067, var_1167, var_2064);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2074 = wp::mul(var_2036, var_2072);
                var_2075 = wp::add(var_2024, var_2074);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2076 = wp::abs(var_2075);
                var_2078 = wp::sub(var_2077, var_1988);
                var_2079 = wp::extract(var_1576, var_2078);
                var_2080 = (var_2076 > var_2079);
                if (var_2080) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2072);
                    wp::assign(var_156, var_2075);
                    wp::assign(var_1167, var_2073);
                    goto start_for_28;
                }
                var_2081 = wp::where(var_2080, var_149, var_2072);
                var_2082 = wp::where(var_2080, var_156, var_2075);
                var_2083 = wp::where(var_2080, var_1167, var_2073);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2085 = wp::extract(var_2045, var_2084);
                var_2087 = wp::extract(var_2051, var_2086);
                var_2088 = wp::mul(var_2087, var_2081);
                var_2089 = wp::add(var_2085, var_2088);
                var_2090 = wp::mul(var_2089, var_1674);
                var_2091 = (var_2090 > var_margin);
                if (var_2091) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2081);
                    wp::assign(var_156, var_2082);
                    wp::assign(var_1167, var_2083);
                    goto start_for_28;
                }
                var_2092 = wp::where(var_2091, var_149, var_2081);
                var_2093 = wp::where(var_2091, var_156, var_2082);
                var_2094 = wp::where(var_2091, var_1167, var_2083);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2096 = wp::mul(var_2045, var_2095);
                var_2097 = wp::mul(var_2092, var_2051);
                var_2099 = wp::mul(var_2097, var_2098);
                var_2100 = wp::add(var_2096, var_2099);
                wp::assign_inplace(var_931, var_1729, var_2100);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2102 = wp::mul(var_2101, var_2094);
                wp::add_inplace(var_931, var_1729, var_1988, var_2102);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2104 = wp::mul(var_2103, var_2093);
                var_2106 = wp::sub(var_2105, var_1988);
                wp::add_inplace(var_931, var_1729, var_2106, var_2104);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2108 = wp::extract(var_931, var_1729, var_2107);
                var_2109 = wp::mul(var_2108, var_1674);
                var_2111 = wp::mul(var_2109, var_2110);
                wp::assign_inplace(var_932, var_1729, var_2111);
                // n += 1                                                                     <L 946>
                var_2113 = wp::add(var_1729, var_2112);
                wp::assign(var_149, var_2092);
                wp::assign(var_156, var_2093);
                wp::assign(var_1729, var_2113);
                wp::assign(var_1167, var_2094);
                goto start_for_28;
            end_for_28:;
        }
        var_2114 = wp::where(var_2053, var_2060, var_1985);
        var_2115 = wp::where(var_2053, var_2055, var_1986);
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_2117 = wp::extract(var_1717, var_2116);
        var_2119 = (var_1987 < var_2118);
        var_2122 = (var_1987 == var_2121);
        var_2123 = wp::extract(var_1722, var_2116);
        var_2124 = wp::extract(var_1727, var_2116);
        var_2125 = wp::where(var_2122, var_2123, var_2124);
        var_2126 = wp::where(var_2119, var_2120, var_2125);
        var_2127 = wp::add(var_2117, var_2126);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2130 = (var_1987 == var_2129);
        var_2128 = var_2130;
        if (!var_2128) {
            var_2132 = (var_1987 == var_2131);
            var_2128 = var_2128 || var_2132;
        }
        var_2133 = wp::extract(var_1722, var_2116);
        var_2134 = wp::extract(var_1727, var_2116);
        var_2135 = wp::where(var_2128, var_2133, var_2134);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2137 = wp::sub(var_2136, var_2116);
        var_2138 = wp::extract(var_1717, var_2137);
        var_2140 = (var_1987 < var_2139);
        var_2143 = (var_1987 == var_2142);
        var_2145 = wp::sub(var_2144, var_2116);
        var_2146 = wp::extract(var_1722, var_2145);
        var_2148 = wp::sub(var_2147, var_2116);
        var_2149 = wp::extract(var_1727, var_2148);
        var_2150 = wp::where(var_2143, var_2146, var_2149);
        var_2151 = wp::where(var_2140, var_2141, var_2150);
        var_2152 = wp::add(var_2138, var_2151);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2155 = (var_1987 == var_2154);
        var_2153 = var_2155;
        if (!var_2153) {
            var_2157 = (var_1987 == var_2156);
            var_2153 = var_2153 || var_2157;
        }
        var_2159 = wp::sub(var_2158, var_2116);
        var_2160 = wp::extract(var_1722, var_2159);
        var_2162 = wp::sub(var_2161, var_2116);
        var_2163 = wp::extract(var_1727, var_2162);
        var_2164 = wp::where(var_2153, var_2160, var_2163);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2166 = (var_1987 < var_2165);
        var_2168 = wp::vec_t<3, wp::float32>(var_2167);
        var_2170 = (var_1987 == var_2169);
        var_2171 = wp::where(var_2170, var_1659, var_1664);
        var_2172 = wp::where(var_2166, var_2168, var_2171);
        var_2173 = wp::add(var_1654, var_2172);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2176 = (var_1987 == var_2175);
        var_2174 = var_2176;
        if (!var_2174) {
            var_2178 = (var_1987 == var_2177);
            var_2174 = var_2174 || var_2178;
        }
        var_2179 = wp::where(var_2174, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2180 = wp::abs(var_2135);
        var_2181 = (var_2180 > var_229);
        if (var_2181) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2183 = wp::div(var_2182, var_2135);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2187 = wp::range(var_2184, var_2185, var_2186);
            start_for_30:;
                if (iter_cmp(var_2187) == 0) goto end_for_30;
                var_2188 = wp::iter_next(var_2187);
                // if n == max_con_pair:                                                      <L 930>
                var_2189 = (var_1729 == var_933);
                if (var_2189) {
                    // break                                                                  <L 931>
                    goto end_for_30;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2190 = wp::extract(var_1576, var_2116);
                var_2191 = wp::float32(var_2188);
                var_2192 = wp::mul(var_2190, var_2191);
                // c1 = (l - la) * br                                                         <L 933>
                var_2193 = wp::sub(var_2192, var_2127);
                var_2194 = wp::mul(var_2193, var_2183);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2197 = (var_2194 < var_2196);
                var_2195 = var_2197;
                if (!var_2195) {
                    var_2199 = (var_2194 > var_2198);
                    var_2195 = var_2195 || var_2199;
                }
                if (var_2195) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2194);
                    wp::assign(var_1167, var_2192);
                    goto start_for_30;
                }
                var_2200 = wp::where(var_2195, var_149, var_2194);
                var_2201 = wp::where(var_2195, var_1167, var_2192);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2202 = wp::mul(var_2164, var_2200);
                var_2203 = wp::add(var_2152, var_2202);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2204 = wp::abs(var_2203);
                var_2206 = wp::sub(var_2205, var_2116);
                var_2207 = wp::extract(var_1576, var_2206);
                var_2208 = (var_2204 > var_2207);
                if (var_2208) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2200);
                    wp::assign(var_156, var_2203);
                    wp::assign(var_1167, var_2201);
                    goto start_for_30;
                }
                var_2209 = wp::where(var_2208, var_149, var_2200);
                var_2210 = wp::where(var_2208, var_156, var_2203);
                var_2211 = wp::where(var_2208, var_1167, var_2201);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2213 = wp::extract(var_2173, var_2212);
                var_2215 = wp::extract(var_2179, var_2214);
                var_2216 = wp::mul(var_2215, var_2209);
                var_2217 = wp::add(var_2213, var_2216);
                var_2218 = wp::mul(var_2217, var_1674);
                var_2219 = (var_2218 > var_margin);
                if (var_2219) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2209);
                    wp::assign(var_156, var_2210);
                    wp::assign(var_1167, var_2211);
                    goto start_for_30;
                }
                var_2220 = wp::where(var_2219, var_149, var_2209);
                var_2221 = wp::where(var_2219, var_156, var_2210);
                var_2222 = wp::where(var_2219, var_1167, var_2211);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2224 = wp::mul(var_2173, var_2223);
                var_2225 = wp::mul(var_2220, var_2179);
                var_2227 = wp::mul(var_2225, var_2226);
                var_2228 = wp::add(var_2224, var_2227);
                wp::assign_inplace(var_931, var_1729, var_2228);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2230 = wp::mul(var_2229, var_2222);
                wp::add_inplace(var_931, var_1729, var_2116, var_2230);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2232 = wp::mul(var_2231, var_2221);
                var_2234 = wp::sub(var_2233, var_2116);
                wp::add_inplace(var_931, var_1729, var_2234, var_2232);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2236 = wp::extract(var_931, var_1729, var_2235);
                var_2237 = wp::mul(var_2236, var_1674);
                var_2239 = wp::mul(var_2237, var_2238);
                wp::assign_inplace(var_932, var_1729, var_2239);
                // n += 1                                                                     <L 946>
                var_2241 = wp::add(var_1729, var_2240);
                wp::assign(var_149, var_2220);
                wp::assign(var_156, var_2221);
                wp::assign(var_1729, var_2241);
                wp::assign(var_1167, var_2222);
                goto start_for_30;
            end_for_30:;
        }
        var_2242 = wp::where(var_2181, var_2188, var_2114);
        var_2243 = wp::where(var_2181, var_2183, var_2115);
        // for q in range(2):                                                                 <L 917>
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_2246 = wp::extract(var_1717, var_2245);
        var_2248 = (var_2244 < var_2247);
        var_2251 = (var_2244 == var_2250);
        var_2252 = wp::extract(var_1722, var_2245);
        var_2253 = wp::extract(var_1727, var_2245);
        var_2254 = wp::where(var_2251, var_2252, var_2253);
        var_2255 = wp::where(var_2248, var_2249, var_2254);
        var_2256 = wp::add(var_2246, var_2255);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2259 = (var_2244 == var_2258);
        var_2257 = var_2259;
        if (!var_2257) {
            var_2261 = (var_2244 == var_2260);
            var_2257 = var_2257 || var_2261;
        }
        var_2262 = wp::extract(var_1722, var_2245);
        var_2263 = wp::extract(var_1727, var_2245);
        var_2264 = wp::where(var_2257, var_2262, var_2263);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2266 = wp::sub(var_2265, var_2245);
        var_2267 = wp::extract(var_1717, var_2266);
        var_2269 = (var_2244 < var_2268);
        var_2272 = (var_2244 == var_2271);
        var_2274 = wp::sub(var_2273, var_2245);
        var_2275 = wp::extract(var_1722, var_2274);
        var_2277 = wp::sub(var_2276, var_2245);
        var_2278 = wp::extract(var_1727, var_2277);
        var_2279 = wp::where(var_2272, var_2275, var_2278);
        var_2280 = wp::where(var_2269, var_2270, var_2279);
        var_2281 = wp::add(var_2267, var_2280);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2284 = (var_2244 == var_2283);
        var_2282 = var_2284;
        if (!var_2282) {
            var_2286 = (var_2244 == var_2285);
            var_2282 = var_2282 || var_2286;
        }
        var_2288 = wp::sub(var_2287, var_2245);
        var_2289 = wp::extract(var_1722, var_2288);
        var_2291 = wp::sub(var_2290, var_2245);
        var_2292 = wp::extract(var_1727, var_2291);
        var_2293 = wp::where(var_2282, var_2289, var_2292);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2295 = (var_2244 < var_2294);
        var_2297 = wp::vec_t<3, wp::float32>(var_2296);
        var_2299 = (var_2244 == var_2298);
        var_2300 = wp::where(var_2299, var_1659, var_1664);
        var_2301 = wp::where(var_2295, var_2297, var_2300);
        var_2302 = wp::add(var_1654, var_2301);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2305 = (var_2244 == var_2304);
        var_2303 = var_2305;
        if (!var_2303) {
            var_2307 = (var_2244 == var_2306);
            var_2303 = var_2303 || var_2307;
        }
        var_2308 = wp::where(var_2303, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2309 = wp::abs(var_2264);
        var_2310 = (var_2309 > var_229);
        if (var_2310) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2312 = wp::div(var_2311, var_2264);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2316 = wp::range(var_2313, var_2314, var_2315);
            start_for_32:;
                if (iter_cmp(var_2316) == 0) goto end_for_32;
                var_2317 = wp::iter_next(var_2316);
                // if n == max_con_pair:                                                      <L 930>
                var_2318 = (var_1729 == var_933);
                if (var_2318) {
                    // break                                                                  <L 931>
                    goto end_for_32;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2319 = wp::extract(var_1576, var_2245);
                var_2320 = wp::float32(var_2317);
                var_2321 = wp::mul(var_2319, var_2320);
                // c1 = (l - la) * br                                                         <L 933>
                var_2322 = wp::sub(var_2321, var_2256);
                var_2323 = wp::mul(var_2322, var_2312);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2326 = (var_2323 < var_2325);
                var_2324 = var_2326;
                if (!var_2324) {
                    var_2328 = (var_2323 > var_2327);
                    var_2324 = var_2324 || var_2328;
                }
                if (var_2324) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2323);
                    wp::assign(var_1167, var_2321);
                    goto start_for_32;
                }
                var_2329 = wp::where(var_2324, var_149, var_2323);
                var_2330 = wp::where(var_2324, var_1167, var_2321);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2331 = wp::mul(var_2293, var_2329);
                var_2332 = wp::add(var_2281, var_2331);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2333 = wp::abs(var_2332);
                var_2335 = wp::sub(var_2334, var_2245);
                var_2336 = wp::extract(var_1576, var_2335);
                var_2337 = (var_2333 > var_2336);
                if (var_2337) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2329);
                    wp::assign(var_156, var_2332);
                    wp::assign(var_1167, var_2330);
                    goto start_for_32;
                }
                var_2338 = wp::where(var_2337, var_149, var_2329);
                var_2339 = wp::where(var_2337, var_156, var_2332);
                var_2340 = wp::where(var_2337, var_1167, var_2330);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2342 = wp::extract(var_2302, var_2341);
                var_2344 = wp::extract(var_2308, var_2343);
                var_2345 = wp::mul(var_2344, var_2338);
                var_2346 = wp::add(var_2342, var_2345);
                var_2347 = wp::mul(var_2346, var_1674);
                var_2348 = (var_2347 > var_margin);
                if (var_2348) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2338);
                    wp::assign(var_156, var_2339);
                    wp::assign(var_1167, var_2340);
                    goto start_for_32;
                }
                var_2349 = wp::where(var_2348, var_149, var_2338);
                var_2350 = wp::where(var_2348, var_156, var_2339);
                var_2351 = wp::where(var_2348, var_1167, var_2340);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2353 = wp::mul(var_2302, var_2352);
                var_2354 = wp::mul(var_2349, var_2308);
                var_2356 = wp::mul(var_2354, var_2355);
                var_2357 = wp::add(var_2353, var_2356);
                wp::assign_inplace(var_931, var_1729, var_2357);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2359 = wp::mul(var_2358, var_2351);
                wp::add_inplace(var_931, var_1729, var_2245, var_2359);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2361 = wp::mul(var_2360, var_2350);
                var_2363 = wp::sub(var_2362, var_2245);
                wp::add_inplace(var_931, var_1729, var_2363, var_2361);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2365 = wp::extract(var_931, var_1729, var_2364);
                var_2366 = wp::mul(var_2365, var_1674);
                var_2368 = wp::mul(var_2366, var_2367);
                wp::assign_inplace(var_932, var_1729, var_2368);
                // n += 1                                                                     <L 946>
                var_2370 = wp::add(var_1729, var_2369);
                wp::assign(var_149, var_2349);
                wp::assign(var_156, var_2350);
                wp::assign(var_1729, var_2370);
                wp::assign(var_1167, var_2351);
                goto start_for_32;
            end_for_32:;
        }
        var_2371 = wp::where(var_2310, var_2317, var_2242);
        var_2372 = wp::where(var_2310, var_2312, var_2243);
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_2374 = wp::extract(var_1717, var_2373);
        var_2376 = (var_2244 < var_2375);
        var_2379 = (var_2244 == var_2378);
        var_2380 = wp::extract(var_1722, var_2373);
        var_2381 = wp::extract(var_1727, var_2373);
        var_2382 = wp::where(var_2379, var_2380, var_2381);
        var_2383 = wp::where(var_2376, var_2377, var_2382);
        var_2384 = wp::add(var_2374, var_2383);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2387 = (var_2244 == var_2386);
        var_2385 = var_2387;
        if (!var_2385) {
            var_2389 = (var_2244 == var_2388);
            var_2385 = var_2385 || var_2389;
        }
        var_2390 = wp::extract(var_1722, var_2373);
        var_2391 = wp::extract(var_1727, var_2373);
        var_2392 = wp::where(var_2385, var_2390, var_2391);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2394 = wp::sub(var_2393, var_2373);
        var_2395 = wp::extract(var_1717, var_2394);
        var_2397 = (var_2244 < var_2396);
        var_2400 = (var_2244 == var_2399);
        var_2402 = wp::sub(var_2401, var_2373);
        var_2403 = wp::extract(var_1722, var_2402);
        var_2405 = wp::sub(var_2404, var_2373);
        var_2406 = wp::extract(var_1727, var_2405);
        var_2407 = wp::where(var_2400, var_2403, var_2406);
        var_2408 = wp::where(var_2397, var_2398, var_2407);
        var_2409 = wp::add(var_2395, var_2408);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2412 = (var_2244 == var_2411);
        var_2410 = var_2412;
        if (!var_2410) {
            var_2414 = (var_2244 == var_2413);
            var_2410 = var_2410 || var_2414;
        }
        var_2416 = wp::sub(var_2415, var_2373);
        var_2417 = wp::extract(var_1722, var_2416);
        var_2419 = wp::sub(var_2418, var_2373);
        var_2420 = wp::extract(var_1727, var_2419);
        var_2421 = wp::where(var_2410, var_2417, var_2420);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2423 = (var_2244 < var_2422);
        var_2425 = wp::vec_t<3, wp::float32>(var_2424);
        var_2427 = (var_2244 == var_2426);
        var_2428 = wp::where(var_2427, var_1659, var_1664);
        var_2429 = wp::where(var_2423, var_2425, var_2428);
        var_2430 = wp::add(var_1654, var_2429);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2433 = (var_2244 == var_2432);
        var_2431 = var_2433;
        if (!var_2431) {
            var_2435 = (var_2244 == var_2434);
            var_2431 = var_2431 || var_2435;
        }
        var_2436 = wp::where(var_2431, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2437 = wp::abs(var_2392);
        var_2438 = (var_2437 > var_229);
        if (var_2438) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2440 = wp::div(var_2439, var_2392);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2444 = wp::range(var_2441, var_2442, var_2443);
            start_for_34:;
                if (iter_cmp(var_2444) == 0) goto end_for_34;
                var_2445 = wp::iter_next(var_2444);
                // if n == max_con_pair:                                                      <L 930>
                var_2446 = (var_1729 == var_933);
                if (var_2446) {
                    // break                                                                  <L 931>
                    goto end_for_34;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2447 = wp::extract(var_1576, var_2373);
                var_2448 = wp::float32(var_2445);
                var_2449 = wp::mul(var_2447, var_2448);
                // c1 = (l - la) * br                                                         <L 933>
                var_2450 = wp::sub(var_2449, var_2384);
                var_2451 = wp::mul(var_2450, var_2440);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2454 = (var_2451 < var_2453);
                var_2452 = var_2454;
                if (!var_2452) {
                    var_2456 = (var_2451 > var_2455);
                    var_2452 = var_2452 || var_2456;
                }
                if (var_2452) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2451);
                    wp::assign(var_1167, var_2449);
                    goto start_for_34;
                }
                var_2457 = wp::where(var_2452, var_149, var_2451);
                var_2458 = wp::where(var_2452, var_1167, var_2449);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2459 = wp::mul(var_2421, var_2457);
                var_2460 = wp::add(var_2409, var_2459);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2461 = wp::abs(var_2460);
                var_2463 = wp::sub(var_2462, var_2373);
                var_2464 = wp::extract(var_1576, var_2463);
                var_2465 = (var_2461 > var_2464);
                if (var_2465) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2457);
                    wp::assign(var_156, var_2460);
                    wp::assign(var_1167, var_2458);
                    goto start_for_34;
                }
                var_2466 = wp::where(var_2465, var_149, var_2457);
                var_2467 = wp::where(var_2465, var_156, var_2460);
                var_2468 = wp::where(var_2465, var_1167, var_2458);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2470 = wp::extract(var_2430, var_2469);
                var_2472 = wp::extract(var_2436, var_2471);
                var_2473 = wp::mul(var_2472, var_2466);
                var_2474 = wp::add(var_2470, var_2473);
                var_2475 = wp::mul(var_2474, var_1674);
                var_2476 = (var_2475 > var_margin);
                if (var_2476) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2466);
                    wp::assign(var_156, var_2467);
                    wp::assign(var_1167, var_2468);
                    goto start_for_34;
                }
                var_2477 = wp::where(var_2476, var_149, var_2466);
                var_2478 = wp::where(var_2476, var_156, var_2467);
                var_2479 = wp::where(var_2476, var_1167, var_2468);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2481 = wp::mul(var_2430, var_2480);
                var_2482 = wp::mul(var_2477, var_2436);
                var_2484 = wp::mul(var_2482, var_2483);
                var_2485 = wp::add(var_2481, var_2484);
                wp::assign_inplace(var_931, var_1729, var_2485);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2487 = wp::mul(var_2486, var_2479);
                wp::add_inplace(var_931, var_1729, var_2373, var_2487);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2489 = wp::mul(var_2488, var_2478);
                var_2491 = wp::sub(var_2490, var_2373);
                wp::add_inplace(var_931, var_1729, var_2491, var_2489);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2493 = wp::extract(var_931, var_1729, var_2492);
                var_2494 = wp::mul(var_2493, var_1674);
                var_2496 = wp::mul(var_2494, var_2495);
                wp::assign_inplace(var_932, var_1729, var_2496);
                // n += 1                                                                     <L 946>
                var_2498 = wp::add(var_1729, var_2497);
                wp::assign(var_149, var_2477);
                wp::assign(var_156, var_2478);
                wp::assign(var_1729, var_2498);
                wp::assign(var_1167, var_2479);
                goto start_for_34;
            end_for_34:;
        }
        var_2499 = wp::where(var_2438, var_2445, var_2371);
        var_2500 = wp::where(var_2438, var_2440, var_2372);
        // for q in range(2):                                                                 <L 917>
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_2503 = wp::extract(var_1717, var_2502);
        var_2505 = (var_2501 < var_2504);
        var_2508 = (var_2501 == var_2507);
        var_2509 = wp::extract(var_1722, var_2502);
        var_2510 = wp::extract(var_1727, var_2502);
        var_2511 = wp::where(var_2508, var_2509, var_2510);
        var_2512 = wp::where(var_2505, var_2506, var_2511);
        var_2513 = wp::add(var_2503, var_2512);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2516 = (var_2501 == var_2515);
        var_2514 = var_2516;
        if (!var_2514) {
            var_2518 = (var_2501 == var_2517);
            var_2514 = var_2514 || var_2518;
        }
        var_2519 = wp::extract(var_1722, var_2502);
        var_2520 = wp::extract(var_1727, var_2502);
        var_2521 = wp::where(var_2514, var_2519, var_2520);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2523 = wp::sub(var_2522, var_2502);
        var_2524 = wp::extract(var_1717, var_2523);
        var_2526 = (var_2501 < var_2525);
        var_2529 = (var_2501 == var_2528);
        var_2531 = wp::sub(var_2530, var_2502);
        var_2532 = wp::extract(var_1722, var_2531);
        var_2534 = wp::sub(var_2533, var_2502);
        var_2535 = wp::extract(var_1727, var_2534);
        var_2536 = wp::where(var_2529, var_2532, var_2535);
        var_2537 = wp::where(var_2526, var_2527, var_2536);
        var_2538 = wp::add(var_2524, var_2537);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2541 = (var_2501 == var_2540);
        var_2539 = var_2541;
        if (!var_2539) {
            var_2543 = (var_2501 == var_2542);
            var_2539 = var_2539 || var_2543;
        }
        var_2545 = wp::sub(var_2544, var_2502);
        var_2546 = wp::extract(var_1722, var_2545);
        var_2548 = wp::sub(var_2547, var_2502);
        var_2549 = wp::extract(var_1727, var_2548);
        var_2550 = wp::where(var_2539, var_2546, var_2549);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2552 = (var_2501 < var_2551);
        var_2554 = wp::vec_t<3, wp::float32>(var_2553);
        var_2556 = (var_2501 == var_2555);
        var_2557 = wp::where(var_2556, var_1659, var_1664);
        var_2558 = wp::where(var_2552, var_2554, var_2557);
        var_2559 = wp::add(var_1654, var_2558);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2562 = (var_2501 == var_2561);
        var_2560 = var_2562;
        if (!var_2560) {
            var_2564 = (var_2501 == var_2563);
            var_2560 = var_2560 || var_2564;
        }
        var_2565 = wp::where(var_2560, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2566 = wp::abs(var_2521);
        var_2567 = (var_2566 > var_229);
        if (var_2567) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2569 = wp::div(var_2568, var_2521);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2573 = wp::range(var_2570, var_2571, var_2572);
            start_for_36:;
                if (iter_cmp(var_2573) == 0) goto end_for_36;
                var_2574 = wp::iter_next(var_2573);
                // if n == max_con_pair:                                                      <L 930>
                var_2575 = (var_1729 == var_933);
                if (var_2575) {
                    // break                                                                  <L 931>
                    goto end_for_36;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2576 = wp::extract(var_1576, var_2502);
                var_2577 = wp::float32(var_2574);
                var_2578 = wp::mul(var_2576, var_2577);
                // c1 = (l - la) * br                                                         <L 933>
                var_2579 = wp::sub(var_2578, var_2513);
                var_2580 = wp::mul(var_2579, var_2569);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2583 = (var_2580 < var_2582);
                var_2581 = var_2583;
                if (!var_2581) {
                    var_2585 = (var_2580 > var_2584);
                    var_2581 = var_2581 || var_2585;
                }
                if (var_2581) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2580);
                    wp::assign(var_1167, var_2578);
                    goto start_for_36;
                }
                var_2586 = wp::where(var_2581, var_149, var_2580);
                var_2587 = wp::where(var_2581, var_1167, var_2578);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2588 = wp::mul(var_2550, var_2586);
                var_2589 = wp::add(var_2538, var_2588);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2590 = wp::abs(var_2589);
                var_2592 = wp::sub(var_2591, var_2502);
                var_2593 = wp::extract(var_1576, var_2592);
                var_2594 = (var_2590 > var_2593);
                if (var_2594) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2586);
                    wp::assign(var_156, var_2589);
                    wp::assign(var_1167, var_2587);
                    goto start_for_36;
                }
                var_2595 = wp::where(var_2594, var_149, var_2586);
                var_2596 = wp::where(var_2594, var_156, var_2589);
                var_2597 = wp::where(var_2594, var_1167, var_2587);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2599 = wp::extract(var_2559, var_2598);
                var_2601 = wp::extract(var_2565, var_2600);
                var_2602 = wp::mul(var_2601, var_2595);
                var_2603 = wp::add(var_2599, var_2602);
                var_2604 = wp::mul(var_2603, var_1674);
                var_2605 = (var_2604 > var_margin);
                if (var_2605) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2595);
                    wp::assign(var_156, var_2596);
                    wp::assign(var_1167, var_2597);
                    goto start_for_36;
                }
                var_2606 = wp::where(var_2605, var_149, var_2595);
                var_2607 = wp::where(var_2605, var_156, var_2596);
                var_2608 = wp::where(var_2605, var_1167, var_2597);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2610 = wp::mul(var_2559, var_2609);
                var_2611 = wp::mul(var_2606, var_2565);
                var_2613 = wp::mul(var_2611, var_2612);
                var_2614 = wp::add(var_2610, var_2613);
                wp::assign_inplace(var_931, var_1729, var_2614);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2616 = wp::mul(var_2615, var_2608);
                wp::add_inplace(var_931, var_1729, var_2502, var_2616);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2618 = wp::mul(var_2617, var_2607);
                var_2620 = wp::sub(var_2619, var_2502);
                wp::add_inplace(var_931, var_1729, var_2620, var_2618);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2622 = wp::extract(var_931, var_1729, var_2621);
                var_2623 = wp::mul(var_2622, var_1674);
                var_2625 = wp::mul(var_2623, var_2624);
                wp::assign_inplace(var_932, var_1729, var_2625);
                // n += 1                                                                     <L 946>
                var_2627 = wp::add(var_1729, var_2626);
                wp::assign(var_149, var_2606);
                wp::assign(var_156, var_2607);
                wp::assign(var_1729, var_2627);
                wp::assign(var_1167, var_2608);
                goto start_for_36;
            end_for_36:;
        }
        var_2628 = wp::where(var_2567, var_2574, var_2499);
        var_2629 = wp::where(var_2567, var_2569, var_2500);
        // la = pts_lp[q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[q], pts_cn2[q]))       <L 918>
        var_2631 = wp::extract(var_1717, var_2630);
        var_2633 = (var_2501 < var_2632);
        var_2636 = (var_2501 == var_2635);
        var_2637 = wp::extract(var_1722, var_2630);
        var_2638 = wp::extract(var_1727, var_2630);
        var_2639 = wp::where(var_2636, var_2637, var_2638);
        var_2640 = wp::where(var_2633, var_2634, var_2639);
        var_2641 = wp::add(var_2631, var_2640);
        // lb = wp.where(i == 0 or i == 3, pts_cn1[q], pts_cn2[q])                            <L 919>
        var_2644 = (var_2501 == var_2643);
        var_2642 = var_2644;
        if (!var_2642) {
            var_2646 = (var_2501 == var_2645);
            var_2642 = var_2642 || var_2646;
        }
        var_2647 = wp::extract(var_1722, var_2630);
        var_2648 = wp::extract(var_1727, var_2630);
        var_2649 = wp::where(var_2642, var_2647, var_2648);
        // lc = pts_lp[1 - q] + wp.where(i < 2, 0.0, wp.where(i == 2, pts_cn1[1 - q], pts_cn2[1 - q]))       <L 920>
        var_2651 = wp::sub(var_2650, var_2630);
        var_2652 = wp::extract(var_1717, var_2651);
        var_2654 = (var_2501 < var_2653);
        var_2657 = (var_2501 == var_2656);
        var_2659 = wp::sub(var_2658, var_2630);
        var_2660 = wp::extract(var_1722, var_2659);
        var_2662 = wp::sub(var_2661, var_2630);
        var_2663 = wp::extract(var_1727, var_2662);
        var_2664 = wp::where(var_2657, var_2660, var_2663);
        var_2665 = wp::where(var_2654, var_2655, var_2664);
        var_2666 = wp::add(var_2652, var_2665);
        // ld = wp.where(i == 0 or i == 3, pts_cn1[1 - q], pts_cn2[1 - q])                    <L 921>
        var_2669 = (var_2501 == var_2668);
        var_2667 = var_2669;
        if (!var_2667) {
            var_2671 = (var_2501 == var_2670);
            var_2667 = var_2667 || var_2671;
        }
        var_2673 = wp::sub(var_2672, var_2630);
        var_2674 = wp::extract(var_1722, var_2673);
        var_2676 = wp::sub(var_2675, var_2630);
        var_2677 = wp::extract(var_1727, var_2676);
        var_2678 = wp::where(var_2667, var_2674, var_2677);
        // lua = axi_lp + wp.where(i < 2, wp.vec3(0.0), wp.where(i == 2, axi_cn1, axi_cn2))       <L 924>
        var_2680 = (var_2501 < var_2679);
        var_2682 = wp::vec_t<3, wp::float32>(var_2681);
        var_2684 = (var_2501 == var_2683);
        var_2685 = wp::where(var_2684, var_1659, var_1664);
        var_2686 = wp::where(var_2680, var_2682, var_2685);
        var_2687 = wp::add(var_1654, var_2686);
        // lub = wp.where(i == 0 or i == 3, axi_cn1, axi_cn2)                                 <L 925>
        var_2690 = (var_2501 == var_2689);
        var_2688 = var_2690;
        if (!var_2688) {
            var_2692 = (var_2501 == var_2691);
            var_2688 = var_2688 || var_2692;
        }
        var_2693 = wp::where(var_2688, var_1659, var_1664);
        // if wp.abs(lb) > MJ_MINVAL:                                                         <L 927>
        var_2694 = wp::abs(var_2649);
        var_2695 = (var_2694 > var_229);
        if (var_2695) {
            // br = 1.0 / lb                                                                  <L 928>
            var_2697 = wp::div(var_2696, var_2649);
            // for j in range(-1, 2, 2):                                                      <L 929>
            var_2701 = wp::range(var_2698, var_2699, var_2700);
            start_for_38:;
                if (iter_cmp(var_2701) == 0) goto end_for_38;
                var_2702 = wp::iter_next(var_2701);
                // if n == max_con_pair:                                                      <L 930>
                var_2703 = (var_1729 == var_933);
                if (var_2703) {
                    // break                                                                  <L 931>
                    goto end_for_38;
                }
                // l = s[q] * wp.float32(j)                                                   <L 932>
                var_2704 = wp::extract(var_1576, var_2630);
                var_2705 = wp::float32(var_2702);
                var_2706 = wp::mul(var_2704, var_2705);
                // c1 = (l - la) * br                                                         <L 933>
                var_2707 = wp::sub(var_2706, var_2641);
                var_2708 = wp::mul(var_2707, var_2697);
                // if c1 < 0 or c1 > 1:                                                       <L 934>
                var_2711 = (var_2708 < var_2710);
                var_2709 = var_2711;
                if (!var_2709) {
                    var_2713 = (var_2708 > var_2712);
                    var_2709 = var_2709 || var_2713;
                }
                if (var_2709) {
                    // continue                                                               <L 935>
                    wp::assign(var_149, var_2708);
                    wp::assign(var_1167, var_2706);
                    goto start_for_38;
                }
                var_2714 = wp::where(var_2709, var_149, var_2708);
                var_2715 = wp::where(var_2709, var_1167, var_2706);
                // c2 = lc + ld * c1                                                          <L 936>
                var_2716 = wp::mul(var_2678, var_2714);
                var_2717 = wp::add(var_2666, var_2716);
                // if wp.abs(c2) > s[1 - q]:                                                  <L 937>
                var_2718 = wp::abs(var_2717);
                var_2720 = wp::sub(var_2719, var_2630);
                var_2721 = wp::extract(var_1576, var_2720);
                var_2722 = (var_2718 > var_2721);
                if (var_2722) {
                    // continue                                                               <L 938>
                    wp::assign(var_149, var_2714);
                    wp::assign(var_156, var_2717);
                    wp::assign(var_1167, var_2715);
                    goto start_for_38;
                }
                var_2723 = wp::where(var_2722, var_149, var_2714);
                var_2724 = wp::where(var_2722, var_156, var_2717);
                var_2725 = wp::where(var_2722, var_1167, var_2715);
                // if (lua[2] + lub[2] * c1) * innorm > margin:                               <L 939>
                var_2727 = wp::extract(var_2687, var_2726);
                var_2729 = wp::extract(var_2693, var_2728);
                var_2730 = wp::mul(var_2729, var_2723);
                var_2731 = wp::add(var_2727, var_2730);
                var_2732 = wp::mul(var_2731, var_1674);
                var_2733 = (var_2732 > var_margin);
                if (var_2733) {
                    // continue                                                               <L 940>
                    wp::assign(var_149, var_2723);
                    wp::assign(var_156, var_2724);
                    wp::assign(var_1167, var_2725);
                    goto start_for_38;
                }
                var_2734 = wp::where(var_2733, var_149, var_2723);
                var_2735 = wp::where(var_2733, var_156, var_2724);
                var_2736 = wp::where(var_2733, var_1167, var_2725);
                // points[n] = lua * 0.5 + c1 * lub * 0.5                                     <L 942>
                var_2738 = wp::mul(var_2687, var_2737);
                var_2739 = wp::mul(var_2734, var_2693);
                var_2741 = wp::mul(var_2739, var_2740);
                var_2742 = wp::add(var_2738, var_2741);
                wp::assign_inplace(var_931, var_1729, var_2742);
                // points[n, q] += 0.5 * l                                                    <L 943>
                var_2744 = wp::mul(var_2743, var_2736);
                wp::add_inplace(var_931, var_1729, var_2630, var_2744);
                // points[n, 1 - q] += 0.5 * c2                                               <L 944>
                var_2746 = wp::mul(var_2745, var_2735);
                var_2748 = wp::sub(var_2747, var_2630);
                wp::add_inplace(var_931, var_1729, var_2748, var_2746);
                // depth[n] = points[n, 2] * innorm * 2.0                                     <L 945>
                var_2750 = wp::extract(var_931, var_1729, var_2749);
                var_2751 = wp::mul(var_2750, var_1674);
                var_2753 = wp::mul(var_2751, var_2752);
                wp::assign_inplace(var_932, var_1729, var_2753);
                // n += 1                                                                     <L 946>
                var_2755 = wp::add(var_1729, var_2754);
                wp::assign(var_149, var_2734);
                wp::assign(var_156, var_2735);
                wp::assign(var_1729, var_2755);
                wp::assign(var_1167, var_2736);
                goto start_for_38;
            end_for_38:;
        }
        var_2756 = wp::where(var_2695, var_2702, var_2628);
        var_2757 = wp::where(var_2695, var_2697, var_2629);
        // nl = n                                                                             <L 948>
        var_2758 = wp::copy(var_1729);
        // ax = pts_cn1[0]                                                                    <L 950>
        var_2760 = wp::extract(var_1722, var_2759);
        // bx = pts_cn2[0]                                                                    <L 951>
        var_2762 = wp::extract(var_1727, var_2761);
        // ay = pts_cn1[1]                                                                    <L 952>
        var_2764 = wp::extract(var_1722, var_2763);
        // by = pts_cn2[1]                                                                    <L 953>
        var_2766 = wp::extract(var_1727, var_2765);
        // C = safe_div(1.0, ax * by - bx * ay)                                               <L 954>
        var_2768 = wp::mul(var_2760, var_2766);
        var_2769 = wp::mul(var_2762, var_2764);
        var_2770 = wp::sub(var_2768, var_2769);
        var_2771 = safe_div_0(var_2767, var_2770);
        // for i in range(4):                                                                 <L 956>
        var_2773 = wp::range(var_2772);
        start_for_40:;
            if (iter_cmp(var_2773) == 0) goto end_for_40;
            var_2774 = wp::iter_next(var_2773);
            // if n == max_con_pair:                                                          <L 957>
            var_2775 = (var_1729 == var_933);
            if (var_2775) {
                // break                                                                      <L 958>
                goto end_for_40;
            }
            // llx = wp.where(i // 2, lx, -lx)                                                <L 959>
            var_2777 = wp::floordiv(var_2774, var_2776);
            var_2778 = wp::neg(var_1578);
            var_2779 = wp::where(var_2777, var_1578, var_2778);
            // lly = wp.where(i % 2, ly, -ly)                                                 <L 960>
            var_2781 = wp::mod(var_2774, var_2780);
            var_2782 = wp::neg(var_1580);
            var_2783 = wp::where(var_2781, var_1580, var_2782);
            // x = llx - pts_lp[0]                                                            <L 962>
            var_2785 = wp::extract(var_1717, var_2784);
            var_2786 = wp::sub(var_2779, var_2785);
            // y = lly - pts_lp[1]                                                            <L 963>
            var_2788 = wp::extract(var_1717, var_2787);
            var_2789 = wp::sub(var_2783, var_2788);
            // u = (x * by - y * bx) * C                                                      <L 965>
            var_2790 = wp::mul(var_2786, var_2766);
            var_2791 = wp::mul(var_2789, var_2762);
            var_2792 = wp::sub(var_2790, var_2791);
            var_2793 = wp::mul(var_2792, var_2771);
            // v = (y * ax - x * ay) * C                                                      <L 966>
            var_2794 = wp::mul(var_2789, var_2760);
            var_2795 = wp::mul(var_2786, var_2764);
            var_2796 = wp::sub(var_2794, var_2795);
            var_2797 = wp::mul(var_2796, var_2771);
            // if nl == 0:                                                                    <L 968>
            var_2799 = (var_2758 == var_2798);
            if (var_2799) {
                // if (u < 0 or u > 1) and (v < 0 or v > 1):                                  <L 969>
                var_2803 = (var_2793 < var_2802);
                var_2801 = var_2803;
                if (!var_2801) {
                    var_2805 = (var_2793 > var_2804);
                    var_2801 = var_2801 || var_2805;
                }
                var_2800 = var_2801;
                if (var_2800) {
                    var_2808 = (var_2797 < var_2807);
                    var_2806 = var_2808;
                    if (!var_2806) {
                        var_2810 = (var_2797 > var_2809);
                        var_2806 = var_2806 || var_2810;
                    }
                    var_2800 = var_2800 && var_2806;
                }
                if (var_2800) {
                    // continue                                                               <L 970>
                    wp::assign(var_1416, var_2779);
                    wp::assign(var_1420, var_2783);
                    wp::assign(var_1423, var_2786);
                    wp::assign(var_1426, var_2789);
                    wp::assign(var_1430, var_2793);
                    wp::assign(var_1434, var_2797);
                    goto start_for_40;
                }
                var_2811 = wp::where(var_2800, var_1416, var_2779);
                var_2812 = wp::where(var_2800, var_1420, var_2783);
                var_2813 = wp::where(var_2800, var_1423, var_2786);
                var_2814 = wp::where(var_2800, var_1426, var_2789);
                var_2815 = wp::where(var_2800, var_1430, var_2793);
                var_2816 = wp::where(var_2800, var_1434, var_2797);
            }
            var_2817 = wp::where(var_2799, var_2811, var_2779);
            var_2818 = wp::where(var_2799, var_2812, var_2783);
            var_2819 = wp::where(var_2799, var_2813, var_2786);
            var_2820 = wp::where(var_2799, var_2814, var_2789);
            var_2821 = wp::where(var_2799, var_2815, var_2793);
            var_2822 = wp::where(var_2799, var_2816, var_2797);
            if (!var_2799) {
                // elif u < 0 or v < 0 or u > 1 or v > 1:                                     <L 971>
                var_2825 = (var_2821 < var_2824);
                var_2823 = var_2825;
                if (!var_2823) {
                    var_2827 = (var_2822 < var_2826);
                    var_2823 = var_2823 || var_2827;
                }
                if (!var_2823) {
                    var_2829 = (var_2821 > var_2828);
                    var_2823 = var_2823 || var_2829;
                }
                if (!var_2823) {
                    var_2831 = (var_2822 > var_2830);
                    var_2823 = var_2823 || var_2831;
                }
                if (var_2823) {
                    // continue                                                               <L 972>
                    wp::assign(var_1416, var_2817);
                    wp::assign(var_1420, var_2818);
                    wp::assign(var_1423, var_2819);
                    wp::assign(var_1426, var_2820);
                    wp::assign(var_1430, var_2821);
                    wp::assign(var_1434, var_2822);
                    goto start_for_40;
                }
                var_2832 = wp::where(var_2823, var_1416, var_2817);
                var_2833 = wp::where(var_2823, var_1420, var_2818);
                var_2834 = wp::where(var_2823, var_1423, var_2819);
                var_2835 = wp::where(var_2823, var_1426, var_2820);
                var_2836 = wp::where(var_2823, var_1430, var_2821);
                var_2837 = wp::where(var_2823, var_1434, var_2822);
            }
            var_2838 = wp::where(var_2799, var_2817, var_2832);
            var_2839 = wp::where(var_2799, var_2818, var_2833);
            var_2840 = wp::where(var_2799, var_2819, var_2834);
            var_2841 = wp::where(var_2799, var_2820, var_2835);
            var_2842 = wp::where(var_2799, var_2821, var_2836);
            var_2843 = wp::where(var_2799, var_2822, var_2837);
            // u = wp.clamp(u, 0.0, 1.0)                                                      <L 974>
            var_2846 = wp::clamp(var_2842, var_2844, var_2845);
            // v = wp.clamp(v, 0.0, 1.0)                                                      <L 975>
            var_2849 = wp::clamp(var_2843, var_2847, var_2848);
            // w = 1.0 - u - v                                                                <L 976>
            var_2851 = wp::sub(var_2850, var_2846);
            var_2852 = wp::sub(var_2851, var_2849);
            // vtmp = pu[0] * w + pu[1] * u + pu[2] * v                                       <L 977>
            var_2854 = wp::extract(var_1675, var_2853);
            var_2855 = wp::mul(var_2854, var_2852);
            var_2857 = wp::extract(var_1675, var_2856);
            var_2858 = wp::mul(var_2857, var_2846);
            var_2859 = wp::add(var_2855, var_2858);
            var_2861 = wp::extract(var_1675, var_2860);
            var_2862 = wp::mul(var_2861, var_2849);
            var_2863 = wp::add(var_2859, var_2862);
            // points[n] = wp.vec3(llx, lly, 0.0)                                             <L 979>
            var_2865 = wp::vec_t<3, wp::float32>(var_2838, var_2839, var_2864);
            wp::assign_inplace(var_931, var_1729, var_2865);
            // vtmp2 = points[n] - vtmp                                                       <L 981>
            var_2866 = wp::extract(var_931, var_1729);
            var_2867 = wp::sub(var_2866, var_2863);
            // tc1 = wp.length_sq(vtmp2)                                                      <L 982>
            var_2868 = wp::length_sq(var_2867);
            // if vtmp[2] > 0 and tc1 > margin * margin:                                      <L 983>
            var_2871 = wp::extract(var_2863, var_2870);
            var_2873 = (var_2871 > var_2872);
            var_2869 = var_2873;
            if (var_2869) {
                var_2874 = wp::mul(var_margin, var_margin);
                var_2875 = (var_2868 > var_2874);
                var_2869 = var_2869 && var_2875;
            }
            if (var_2869) {
                // continue                                                                   <L 984>
                wp::assign(var_1416, var_2838);
                wp::assign(var_1420, var_2839);
                wp::assign(var_1423, var_2840);
                wp::assign(var_1426, var_2841);
                wp::assign(var_1430, var_2846);
                wp::assign(var_1434, var_2849);
                goto start_for_40;
            }
            var_2876 = wp::where(var_2869, var_1416, var_2838);
            var_2877 = wp::where(var_2869, var_1420, var_2839);
            var_2878 = wp::where(var_2869, var_1423, var_2840);
            var_2879 = wp::where(var_2869, var_1426, var_2841);
            var_2880 = wp::where(var_2869, var_1430, var_2846);
            var_2881 = wp::where(var_2869, var_1434, var_2849);
            // points[n] = 0.5 * (points[n] + vtmp)                                           <L 986>
            var_2883 = wp::extract(var_931, var_1729);
            var_2884 = wp::add(var_2883, var_2863);
            var_2885 = wp::mul(var_2882, var_2884);
            wp::assign_inplace(var_931, var_1729, var_2885);
            // depth[n] = wp.sqrt(tc1) * wp.where(vtmp[2] < 0, -1.0, 1.0)                     <L 988>
            var_2886 = wp::sqrt(var_2868);
            var_2888 = wp::extract(var_2863, var_2887);
            var_2890 = (var_2888 < var_2889);
            var_2893 = wp::where(var_2890, var_2891, var_2892);
            var_2894 = wp::mul(var_2886, var_2893);
            wp::assign_inplace(var_932, var_1729, var_2894);
            // n += 1                                                                         <L 989>
            var_2896 = wp::add(var_1729, var_2895);
            wp::assign(var_1729, var_2896);
            wp::assign(var_1416, var_2876);
            wp::assign(var_1420, var_2877);
            wp::assign(var_1423, var_2878);
            wp::assign(var_1426, var_2879);
            wp::assign(var_1430, var_2880);
            wp::assign(var_1434, var_2881);
            goto start_for_40;
        end_for_40:;
        // nf = n                                                                             <L 991>
        var_2897 = wp::copy(var_1729);
        // for i in range(4):                                                                 <L 993>
        var_2899 = wp::range(var_2898);
        start_for_42:;
            if (iter_cmp(var_2899) == 0) goto end_for_42;
            var_2900 = wp::iter_next(var_2899);
            // if n >= max_con_pair:                                                          <L 994>
            var_2901 = (var_1729 >= var_933);
            if (var_2901) {
                // break                                                                      <L 995>
                goto end_for_42;
            }
            // x = pu[i, 0]                                                                   <L 996>
            var_2903 = wp::extract(var_1675, var_2900, var_2902);
            // y = pu[i, 1]                                                                   <L 997>
            var_2905 = wp::extract(var_1675, var_2900, var_2904);
            // if nl == 0 and nf != 0:                                                        <L 998>
            var_2908 = (var_2758 == var_2907);
            var_2906 = var_2908;
            if (var_2906) {
                var_2910 = (var_2897 != var_2909);
                var_2906 = var_2906 && var_2910;
            }
            if (var_2906) {
                // if (x < -lx or x > lx) and (y < -ly or y > ly):                            <L 999>
                var_2913 = wp::neg(var_1578);
                var_2914 = (var_2903 < var_2913);
                var_2912 = var_2914;
                if (!var_2912) {
                    var_2915 = (var_2903 > var_1578);
                    var_2912 = var_2912 || var_2915;
                }
                var_2911 = var_2912;
                if (var_2911) {
                    var_2917 = wp::neg(var_1580);
                    var_2918 = (var_2905 < var_2917);
                    var_2916 = var_2918;
                    if (!var_2916) {
                        var_2919 = (var_2905 > var_1580);
                        var_2916 = var_2916 || var_2919;
                    }
                    var_2911 = var_2911 && var_2916;
                }
                if (var_2911) {
                    // continue                                                               <L 1000>
                    wp::assign(var_1423, var_2903);
                    wp::assign(var_1426, var_2905);
                    goto start_for_42;
                }
                var_2920 = wp::where(var_2911, var_1423, var_2903);
                var_2921 = wp::where(var_2911, var_1426, var_2905);
            }
            var_2922 = wp::where(var_2906, var_2920, var_2903);
            var_2923 = wp::where(var_2906, var_2921, var_2905);
            if (!var_2906) {
                // elif x < -lx or x > lx or y < -ly or y > ly:                               <L 1001>
                var_2925 = wp::neg(var_1578);
                var_2926 = (var_2922 < var_2925);
                var_2924 = var_2926;
                if (!var_2924) {
                    var_2927 = (var_2922 > var_1578);
                    var_2924 = var_2924 || var_2927;
                }
                if (!var_2924) {
                    var_2928 = wp::neg(var_1580);
                    var_2929 = (var_2923 < var_2928);
                    var_2924 = var_2924 || var_2929;
                }
                if (!var_2924) {
                    var_2930 = (var_2923 > var_1580);
                    var_2924 = var_2924 || var_2930;
                }
                if (var_2924) {
                    // continue                                                               <L 1002>
                    wp::assign(var_1423, var_2922);
                    wp::assign(var_1426, var_2923);
                    goto start_for_42;
                }
                var_2931 = wp::where(var_2924, var_1423, var_2922);
                var_2932 = wp::where(var_2924, var_1426, var_2923);
            }
            var_2933 = wp::where(var_2906, var_2922, var_2931);
            var_2934 = wp::where(var_2906, var_2923, var_2932);
            // c1 = wp.float32(0)                                                             <L 1004>
            var_2936 = wp::float32(var_2935);
            // for j in range(2):                                                             <L 1006>
            // if pu[i, j] < -s[j]:                                                           <L 1007>
            var_2938 = wp::extract(var_1675, var_2900, var_2937);
            var_2939 = wp::extract(var_1576, var_2937);
            var_2940 = wp::neg(var_2939);
            var_2941 = (var_2938 < var_2940);
            if (var_2941) {
                // c1 += (pu[i, j] + s[j]) * (pu[i, j] + s[j])                                <L 1008>
                var_2942 = wp::extract(var_1675, var_2900, var_2937);
                var_2943 = wp::extract(var_1576, var_2937);
                var_2944 = wp::add(var_2942, var_2943);
                var_2945 = wp::extract(var_1675, var_2900, var_2937);
                var_2946 = wp::extract(var_1576, var_2937);
                var_2947 = wp::add(var_2945, var_2946);
                var_2948 = wp::mul(var_2944, var_2947);
                var_2949 = wp::add(var_2936, var_2948);
            }
            var_2950 = wp::where(var_2941, var_2949, var_2936);
            if (!var_2941) {
                // elif pu[i, j] > s[j]:                                                      <L 1009>
                var_2951 = wp::extract(var_1675, var_2900, var_2937);
                var_2952 = wp::extract(var_1576, var_2937);
                var_2953 = (var_2951 > var_2952);
                if (var_2953) {
                    // c1 += (pu[i, j] - s[j]) * (pu[i, j] - s[j])                            <L 1010>
                    var_2954 = wp::extract(var_1675, var_2900, var_2937);
                    var_2955 = wp::extract(var_1576, var_2937);
                    var_2956 = wp::sub(var_2954, var_2955);
                    var_2957 = wp::extract(var_1675, var_2900, var_2937);
                    var_2958 = wp::extract(var_1576, var_2937);
                    var_2959 = wp::sub(var_2957, var_2958);
                    var_2960 = wp::mul(var_2956, var_2959);
                    var_2961 = wp::add(var_2950, var_2960);
                }
                var_2962 = wp::where(var_2953, var_2961, var_2950);
            }
            var_2963 = wp::where(var_2941, var_2950, var_2962);
            // if pu[i, j] < -s[j]:                                                           <L 1007>
            var_2965 = wp::extract(var_1675, var_2900, var_2964);
            var_2966 = wp::extract(var_1576, var_2964);
            var_2967 = wp::neg(var_2966);
            var_2968 = (var_2965 < var_2967);
            if (var_2968) {
                // c1 += (pu[i, j] + s[j]) * (pu[i, j] + s[j])                                <L 1008>
                var_2969 = wp::extract(var_1675, var_2900, var_2964);
                var_2970 = wp::extract(var_1576, var_2964);
                var_2971 = wp::add(var_2969, var_2970);
                var_2972 = wp::extract(var_1675, var_2900, var_2964);
                var_2973 = wp::extract(var_1576, var_2964);
                var_2974 = wp::add(var_2972, var_2973);
                var_2975 = wp::mul(var_2971, var_2974);
                var_2976 = wp::add(var_2963, var_2975);
            }
            var_2977 = wp::where(var_2968, var_2976, var_2963);
            if (!var_2968) {
                // elif pu[i, j] > s[j]:                                                      <L 1009>
                var_2978 = wp::extract(var_1675, var_2900, var_2964);
                var_2979 = wp::extract(var_1576, var_2964);
                var_2980 = (var_2978 > var_2979);
                if (var_2980) {
                    // c1 += (pu[i, j] - s[j]) * (pu[i, j] - s[j])                            <L 1010>
                    var_2981 = wp::extract(var_1675, var_2900, var_2964);
                    var_2982 = wp::extract(var_1576, var_2964);
                    var_2983 = wp::sub(var_2981, var_2982);
                    var_2984 = wp::extract(var_1675, var_2900, var_2964);
                    var_2985 = wp::extract(var_1576, var_2964);
                    var_2986 = wp::sub(var_2984, var_2985);
                    var_2987 = wp::mul(var_2983, var_2986);
                    var_2988 = wp::add(var_2977, var_2987);
                }
                var_2989 = wp::where(var_2980, var_2988, var_2977);
            }
            var_2990 = wp::where(var_2968, var_2977, var_2989);
            // c1 += pu[i, 2] * innorm * pu[i, 2] * innorm                                    <L 1012>
            var_2992 = wp::extract(var_1675, var_2900, var_2991);
            var_2993 = wp::mul(var_2992, var_1674);
            var_2995 = wp::extract(var_1675, var_2900, var_2994);
            var_2996 = wp::mul(var_2993, var_2995);
            var_2997 = wp::mul(var_2996, var_1674);
            var_2998 = wp::add(var_2990, var_2997);
            // if pu[i, 2] > 0 and c1 > margin * margin:                                      <L 1014>
            var_3001 = wp::extract(var_1675, var_2900, var_3000);
            var_3003 = (var_3001 > var_3002);
            var_2999 = var_3003;
            if (var_2999) {
                var_3004 = wp::mul(var_margin, var_margin);
                var_3005 = (var_2998 > var_3004);
                var_2999 = var_2999 && var_3005;
            }
            if (var_2999) {
                // continue                                                                   <L 1015>
                wp::assign(var_149, var_2998);
                wp::assign(var_1423, var_2933);
                wp::assign(var_1426, var_2934);
                goto start_for_42;
            }
            var_3006 = wp::where(var_2999, var_149, var_2998);
            var_3007 = wp::where(var_2999, var_1423, var_2933);
            var_3008 = wp::where(var_2999, var_1426, var_2934);
            // tmp_p = wp.vec3(pu[i, 0], pu[i, 1], 0.0)                                       <L 1017>
            var_3010 = wp::extract(var_1675, var_2900, var_3009);
            var_3012 = wp::extract(var_1675, var_2900, var_3011);
            var_3014 = wp::vec_t<3, wp::float32>(var_3010, var_3012, var_3013);
            // for j in range(2):                                                             <L 1019>
            // if pu[i, j] < -s[j]:                                                           <L 1020>
            var_3016 = wp::extract(var_1675, var_2900, var_3015);
            var_3017 = wp::extract(var_1576, var_3015);
            var_3018 = wp::neg(var_3017);
            var_3019 = (var_3016 < var_3018);
            if (var_3019) {
                // tmp_p[j] = -s[j] * 0.5                                                     <L 1021>
                var_3020 = wp::extract(var_1576, var_3015);
                var_3021 = wp::neg(var_3020);
                var_3023 = wp::mul(var_3021, var_3022);
                wp::assign_inplace(var_3014, var_3015, var_3023);
            }
            if (!var_3019) {
                // elif pu[i, j] > s[j]:                                                      <L 1022>
                var_3024 = wp::extract(var_1675, var_2900, var_3015);
                var_3025 = wp::extract(var_1576, var_3015);
                var_3026 = (var_3024 > var_3025);
                if (var_3026) {
                    // tmp_p[j] = +s[j] * 0.5                                                 <L 1023>
                    var_3027 = wp::extract(var_1576, var_3015);
                    var_3028 = wp::pos(var_3027);
                    var_3030 = wp::mul(var_3028, var_3029);
                    wp::assign_inplace(var_3014, var_3015, var_3030);
                }
            }
            // if pu[i, j] < -s[j]:                                                           <L 1020>
            var_3032 = wp::extract(var_1675, var_2900, var_3031);
            var_3033 = wp::extract(var_1576, var_3031);
            var_3034 = wp::neg(var_3033);
            var_3035 = (var_3032 < var_3034);
            if (var_3035) {
                // tmp_p[j] = -s[j] * 0.5                                                     <L 1021>
                var_3036 = wp::extract(var_1576, var_3031);
                var_3037 = wp::neg(var_3036);
                var_3039 = wp::mul(var_3037, var_3038);
                wp::assign_inplace(var_3014, var_3031, var_3039);
            }
            if (!var_3035) {
                // elif pu[i, j] > s[j]:                                                      <L 1022>
                var_3040 = wp::extract(var_1675, var_2900, var_3031);
                var_3041 = wp::extract(var_1576, var_3031);
                var_3042 = (var_3040 > var_3041);
                if (var_3042) {
                    // tmp_p[j] = +s[j] * 0.5                                                 <L 1023>
                    var_3043 = wp::extract(var_1576, var_3031);
                    var_3044 = wp::pos(var_3043);
                    var_3046 = wp::mul(var_3044, var_3045);
                    wp::assign_inplace(var_3014, var_3031, var_3046);
                }
            }
            // tmp_p += pu[i]                                                                 <L 1025>
            var_3047 = wp::extract(var_1675, var_2900);
            var_3048 = wp::add(var_3014, var_3047);
            // points[n] = tmp_p * 0.5                                                        <L 1026>
            var_3050 = wp::mul(var_3048, var_3049);
            wp::assign_inplace(var_931, var_1729, var_3050);
            // depth[n] = wp.sqrt(c1) * wp.where(pu[i, 2] < 0, -1.0, 1.0)                     <L 1028>
            var_3051 = wp::sqrt(var_3006);
            var_3053 = wp::extract(var_1675, var_2900, var_3052);
            var_3055 = (var_3053 < var_3054);
            var_3058 = wp::where(var_3055, var_3056, var_3057);
            var_3059 = wp::mul(var_3051, var_3058);
            wp::assign_inplace(var_932, var_1729, var_3059);
            // n += 1                                                                         <L 1029>
            var_3061 = wp::add(var_1729, var_3060);
            wp::assign(var_149, var_3006);
            wp::assign(var_1729, var_3061);
            wp::assign(var_1423, var_3007);
            wp::assign(var_1426, var_3008);
            goto start_for_42;
        end_for_42:;
        // rw = box1_rot @ wp.transpose(rotmore)                                              <L 1032>
        var_3062 = wp::transpose(var_1569);
        var_3063 = wp::mul(var_box1_rot, var_3062);
        // pw = box1_pos                                                                      <L 1033>
        var_3064 = wp::copy(var_box1_pos);
        // normal = wp.where(inv, -1.0, 1.0) * rw @ rnorm                                     <L 1034>
        var_3067 = wp::where(var_191, var_3065, var_3066);
        var_3068 = wp::mul(var_3067, var_3063);
        var_3069 = wp::mul(var_3068, var_1571);
    }
    var_3070 = wp::where(var_935, var_1523, var_2900);
    var_3071 = wp::where(var_935, var_687, var_3031);
    var_3072 = wp::where(var_935, var_940, var_1569);
    var_3073 = wp::where(var_935, var_942, var_1572);
    var_3074 = wp::where(var_935, var_944, var_1570);
    var_3075 = wp::where(var_935, var_948, var_1576);
    var_3076 = wp::where(var_935, var_949, var_1573);
    var_3077 = wp::where(var_935, var_951, var_1578);
    var_3078 = wp::where(var_935, var_953, var_1580);
    var_3079 = wp::where(var_935, var_955, var_1582);
    var_3080 = wp::where(var_935, var_1496, var_1729);
    var_3081 = wp::where(var_935, var_1197, var_2630);
    var_3082 = wp::where(var_935, var_1258, var_2757);
    var_3083 = wp::where(var_935, var_1262, var_2760);
    var_3084 = wp::where(var_935, var_1264, var_2762);
    var_3085 = wp::where(var_935, var_1266, var_2764);
    var_3086 = wp::where(var_935, var_1268, var_2766);
    var_3087 = wp::where(var_935, var_1273, var_2771);
    var_3088 = wp::where(var_935, var_1514, var_3063);
    var_3089 = wp::where(var_935, var_1515, var_3064);
    var_3090 = wp::where(var_935, var_1522, var_3069);
    // contact_count = n                                                                      <L 1036>
    var_3091 = wp::copy(var_3080);
    // for i in range(contact_count):                                                         <L 1039>
    var_3092 = wp::range(var_3091);
    start_for_44:;
        if (iter_cmp(var_3092) == 0) goto end_for_44;
        var_3093 = wp::iter_next(var_3092);
        // points[i, 2] += hz                                                                 <L 1040>
        wp::add_inplace(var_931, var_3093, var_3094, var_3079);
        // pos = rw @ points[i] + pw                                                          <L 1041>
        var_3095 = wp::extract(var_931, var_3093);
        var_3096 = wp::mul(var_3088, var_3095);
        var_3097 = wp::add(var_3096, var_3089);
        // contact_dist[i] = depth[i]                                                         <L 1042>
        var_3098 = wp::extract(var_932, var_3093);
        wp::assign_inplace(var_0, var_3093, var_3098);
        // contact_pos[i] = pos                                                               <L 1043>
        wp::assign_inplace(var_10, var_3093, var_3097);
        // contact_normals[i] = normal                                                        <L 1044>
        wp::assign_inplace(var_11, var_3093, var_3090);
        goto start_for_44;
    end_for_44:;
    // return contact_dist, contact_pos, contact_normals                                      <L 1046>
    ret_0 = var_0;
    ret_1 = var_10;
    ret_2 = var_11;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive.py:1249
static CUDA_CALLABLE void box_box_wrapper_0(
    wp::int32 var_naconmax_in,
    Geom_4022bfa7 var_box1,
    Geom_4022bfa7 var_box2,
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
    wp::mat_t<3, 3, wp::float32>* var_1;
    wp::vec_t<3, wp::float32>* var_2;
    wp::vec_t<3, wp::float32>* var_3;
    wp::mat_t<3, 3, wp::float32>* var_4;
    wp::vec_t<3, wp::float32>* var_5;
    wp::vec_t<8, wp::float32> var_6;
    wp::mat_t<8, 3, wp::float32> var_7;
    wp::mat_t<8, 3, wp::float32> var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::mat_t<3, 3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::mat_t<3, 3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::mat_t<3, 3, wp::float32> var_19;
    wp::int32 var_20;
    const wp::int32 var_21 = 1;
    wp::float32 var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::mat_t<3, 3, wp::float32> var_25;
    wp::int32 var_26;
    const wp::int32 var_27 = 2;
    wp::float32 var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::vec_t<3, wp::float32> var_30;
    wp::mat_t<3, 3, wp::float32> var_31;
    wp::int32 var_32;
    const wp::int32 var_33 = 3;
    wp::float32 var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::mat_t<3, 3, wp::float32> var_37;
    wp::int32 var_38;
    const wp::int32 var_39 = 4;
    wp::float32 var_40;
    wp::vec_t<3, wp::float32> var_41;
    wp::vec_t<3, wp::float32> var_42;
    wp::mat_t<3, 3, wp::float32> var_43;
    wp::int32 var_44;
    const wp::int32 var_45 = 5;
    wp::float32 var_46;
    wp::vec_t<3, wp::float32> var_47;
    wp::vec_t<3, wp::float32> var_48;
    wp::mat_t<3, 3, wp::float32> var_49;
    wp::int32 var_50;
    const wp::int32 var_51 = 6;
    wp::float32 var_52;
    wp::vec_t<3, wp::float32> var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::mat_t<3, 3, wp::float32> var_55;
    wp::int32 var_56;
    const wp::int32 var_57 = 7;
    wp::float32 var_58;
    wp::vec_t<3, wp::float32> var_59;
    wp::vec_t<3, wp::float32> var_60;
    wp::mat_t<3, 3, wp::float32> var_61;
    wp::int32 var_62;
    //---------
    // forward
    // def box_box_wrapper(                                                                   <L 1250>
    // dist, pos, normal = box_box(                                                           <L 1287>
    // box1.pos,                                                                              <L 1288>
    var_0 = &((var_box1).pos);
    // box1.rot,                                                                              <L 1289>
    var_1 = &((var_box1).rot);
    // box1.size,                                                                             <L 1290>
    var_2 = &((var_box1).size);
    // box2.pos,                                                                              <L 1291>
    var_3 = &((var_box2).pos);
    // box2.rot,                                                                              <L 1292>
    var_4 = &((var_box2).rot);
    // box2.size,                                                                             <L 1293>
    var_5 = &((var_box2).size);
    // margin,                                                                                <L 1294>
    var_9 = wp::load(var_0);
    var_10 = wp::load(var_1);
    var_11 = wp::load(var_2);
    var_12 = wp::load(var_3);
    var_13 = wp::load(var_4);
    var_14 = wp::load(var_5);
    box_box_0(var_9, var_10, var_11, var_12, var_13, var_14, var_margin, var_6, var_7, var_8);
    // for i in range(8):                                                                     <L 1297>
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_16 = wp::extract(var_6, var_15);
    // pos[i],                                                                                <L 1302>
    var_17 = wp::extract(var_7, var_15);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_18 = wp::extract(var_8, var_15);
    var_19 = make_frame_0(var_18);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_20 = write_contact_0(var_naconmax_in, var_15, var_16, var_17, var_19, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_22 = wp::extract(var_6, var_21);
    // pos[i],                                                                                <L 1302>
    var_23 = wp::extract(var_7, var_21);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_24 = wp::extract(var_8, var_21);
    var_25 = make_frame_0(var_24);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_26 = write_contact_0(var_naconmax_in, var_21, var_22, var_23, var_25, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_28 = wp::extract(var_6, var_27);
    // pos[i],                                                                                <L 1302>
    var_29 = wp::extract(var_7, var_27);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_30 = wp::extract(var_8, var_27);
    var_31 = make_frame_0(var_30);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_32 = write_contact_0(var_naconmax_in, var_27, var_28, var_29, var_31, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_34 = wp::extract(var_6, var_33);
    // pos[i],                                                                                <L 1302>
    var_35 = wp::extract(var_7, var_33);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_36 = wp::extract(var_8, var_33);
    var_37 = make_frame_0(var_36);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_38 = write_contact_0(var_naconmax_in, var_33, var_34, var_35, var_37, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_40 = wp::extract(var_6, var_39);
    // pos[i],                                                                                <L 1302>
    var_41 = wp::extract(var_7, var_39);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_42 = wp::extract(var_8, var_39);
    var_43 = make_frame_0(var_42);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_44 = write_contact_0(var_naconmax_in, var_39, var_40, var_41, var_43, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_46 = wp::extract(var_6, var_45);
    // pos[i],                                                                                <L 1302>
    var_47 = wp::extract(var_7, var_45);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_48 = wp::extract(var_8, var_45);
    var_49 = make_frame_0(var_48);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_50 = write_contact_0(var_naconmax_in, var_45, var_46, var_47, var_49, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_52 = wp::extract(var_6, var_51);
    // pos[i],                                                                                <L 1302>
    var_53 = wp::extract(var_7, var_51);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_54 = wp::extract(var_8, var_51);
    var_55 = make_frame_0(var_54);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_56 = write_contact_0(var_naconmax_in, var_51, var_52, var_53, var_55, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
    // write_contact(                                                                         <L 1298>
    // naconmax_in,                                                                           <L 1299>
    // i,                                                                                     <L 1300>
    // dist[i],                                                                               <L 1301>
    var_58 = wp::extract(var_6, var_57);
    // pos[i],                                                                                <L 1302>
    var_59 = wp::extract(var_7, var_57);
    // make_frame(normal[i]),                                                                 <L 1303>
    var_60 = wp::extract(var_8, var_57);
    var_61 = make_frame_0(var_60);
    // margin,                                                                                <L 1304>
    // gap,                                                                                   <L 1305>
    // condim,                                                                                <L 1306>
    // friction,                                                                              <L 1307>
    // solref,                                                                                <L 1308>
    // solreffriction,                                                                        <L 1309>
    // solimp,                                                                                <L 1310>
    // adhesion,                                                                              <L 1311>
    // geoms,                                                                                 <L 1312>
    // pairid,                                                                                <L 1313>
    // worldid,                                                                               <L 1314>
    // contact_dist_out,                                                                      <L 1315>
    // contact_pos_out,                                                                       <L 1316>
    // contact_frame_out,                                                                     <L 1317>
    // contact_includemargin_out,                                                             <L 1318>
    // contact_friction_out,                                                                  <L 1319>
    // contact_solref_out,                                                                    <L 1320>
    // contact_solreffriction_out,                                                            <L 1321>
    // contact_solimp_out,                                                                    <L 1322>
    // contact_dim_out,                                                                       <L 1323>
    // contact_geom_out,                                                                      <L 1324>
    // contact_efc_address_out,                                                               <L 1325>
    // contact_worldid_out,                                                                   <L 1326>
    // contact_type_out,                                                                      <L 1327>
    // contact_geomcollisionid_out,                                                           <L 1328>
    // contact_adhesion_out,                                                                  <L 1329>
    // nacon_out,                                                                             <L 1330>
    var_62 = write_contact_0(var_naconmax_in, var_57, var_58, var_59, var_61, var_margin, var_gap, var_condim, var_friction, var_solref, var_solreffriction, var_solimp, var_adhesion, var_geoms, var_pairid, var_worldid, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:562
static CUDA_CALLABLE void adj__compute_rotmore_0(
    wp::int32 var_face_idx,
    wp::int32 & adj_face_idx,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive_core.py:594
static CUDA_CALLABLE void adj_box_box_0(
    wp::vec_t<3, wp::float32> var_box1_pos,
    wp::mat_t<3, 3, wp::float32> var_box1_rot,
    wp::vec_t<3, wp::float32> var_box1_size,
    wp::vec_t<3, wp::float32> var_box2_pos,
    wp::mat_t<3, 3, wp::float32> var_box2_rot,
    wp::vec_t<3, wp::float32> var_box2_size,
    wp::float32 var_margin,
    wp::vec_t<8, wp::float32> & ret_0,
    wp::mat_t<8, 3, wp::float32> & ret_1,
    wp::mat_t<8, 3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & adj_box1_pos,
    wp::mat_t<3, 3, wp::float32> & adj_box1_rot,
    wp::vec_t<3, wp::float32> & adj_box1_size,
    wp::vec_t<3, wp::float32> & adj_box2_pos,
    wp::mat_t<3, 3, wp::float32> & adj_box2_rot,
    wp::vec_t<3, wp::float32> & adj_box2_size,
    wp::float32 & adj_margin,
    wp::vec_t<8, wp::float32> & adj_ret_0,
    wp::mat_t<8, 3, wp::float32> & adj_ret_1,
    wp::mat_t<8, 3, wp::float32> & adj_ret_2)
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_primitive.py:1249
static CUDA_CALLABLE void adj_box_box_wrapper_0(
    wp::int32 var_naconmax_in,
    Geom_4022bfa7 var_box1,
    Geom_4022bfa7 var_box2,
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
    Geom_4022bfa7 & adj_box1,
    Geom_4022bfa7 & adj_box2,
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



extern "C" __global__ void _primitive_narrowphase__locals__primitive_narrowphase_a92306f2_cuda_kernel_forward(
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
        const wp::int32 var_23 = 6;
        const wp::int32 var_24 = 6;
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
        const wp::int32 var_40 = 1;
        const wp::int32 var_41 = 0;
        const wp::int32 var_42 = 2;
        const wp::int32 var_43 = 0;
        wp::int32 var_44;
        wp::int32* var_45;
        wp::int32 var_46;
        wp::int32 var_47;
        const wp::int32 var_48 = 1;
        wp::int32 var_49;
        wp::int32* var_50;
        wp::int32 var_51;
        wp::int32 var_52;
        bool var_53;
        bool var_54;
        bool var_55;
        wp::vec_t<2, wp::int32>* var_56;
        wp::vec_t<2, wp::int32> var_57;
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
            box_box_wrapper_0(var_naconmax_in, var_20, var_21, var_9, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_6, var_39, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        }
        // collision_type1 = wp.static(primitive_collisions_types[i][0])                          <L 1475>
        // collision_type2 = wp.static(primitive_collisions_types[i][1])                          <L 1476>
        // type1 = geom_type[geoms[0]]                                                            <L 1477>
        var_44 = wp::extract(var_6, var_43);
        var_45 = wp::address(var_geom_type, var_44);
        var_47 = wp::load(var_45);
        var_46 = wp::copy(var_47);
        // type2 = geom_type[geoms[1]]                                                            <L 1478>
        var_49 = wp::extract(var_6, var_48);
        var_50 = wp::address(var_geom_type, var_49);
        var_52 = wp::load(var_50);
        var_51 = wp::copy(var_52);
        // if collision_type1 == type1 and collision_type2 == type2:                              <L 1479>
        var_54 = (var_41 == var_46);
        var_53 = var_54;
        if (var_53) {
            var_55 = (var_42 == var_51);
            var_53 = var_53 && var_55;
        }
        if (var_53) {
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
            var_56 = wp::address(var_collision_pairid_in, var_0);
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
            var_57 = wp::load(var_56);
            plane_sphere_wrapper_0(var_naconmax_in, var_20, var_21, var_9, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_6, var_57, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        }
    }
}

