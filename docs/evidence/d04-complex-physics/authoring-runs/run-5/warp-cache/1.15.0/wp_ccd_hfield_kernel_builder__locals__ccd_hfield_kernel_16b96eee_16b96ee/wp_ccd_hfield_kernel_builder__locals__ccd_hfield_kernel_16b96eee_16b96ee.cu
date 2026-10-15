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




struct SupportPoint_e82efc60
{
    wp::vec_t<3, wp::float32> point;
    wp::int32 cached_index;
    wp::int32 vertex_index;


    SupportPoint_e82efc60() = default;
    CUDA_CALLABLE SupportPoint_e82efc60(wp::vec_t<3, wp::float32> const& point,
    wp::int32 const& cached_index = {},
    wp::int32 const& vertex_index = {})
        : point{point}
        , cached_index{cached_index}
        , vertex_index{vertex_index}

    {
    }

    CUDA_CALLABLE SupportPoint_e82efc60& operator += (const SupportPoint_e82efc60& rhs)
    {    point += rhs.point;
    cached_index += rhs.cached_index;
    vertex_index += rhs.vertex_index;

        return *this;}

};

static CUDA_CALLABLE void adj_SupportPoint_e82efc60(wp::vec_t<3, wp::float32> const&,
    wp::int32 const&,
    wp::int32 const&,
    wp::vec_t<3, wp::float32> & adj_point,
    wp::int32 & adj_cached_index,
    wp::int32 & adj_vertex_index,
    SupportPoint_e82efc60 & adj_ret)
{
    adj_point += adj_ret.point;
    adj_cached_index += adj_ret.cached_index;
    adj_vertex_index += adj_ret.vertex_index;
}

// Required when compiling adjoints.
CUDA_CALLABLE SupportPoint_e82efc60 add(const SupportPoint_e82efc60& a, const SupportPoint_e82efc60& b)
{
    return SupportPoint_e82efc60();
}

CUDA_CALLABLE void adj_atomic_add(SupportPoint_e82efc60* p, SupportPoint_e82efc60 t)
{
    wp::adj_atomic_add(&p->point, t.point);
    wp::adj_atomic_add(&p->cached_index, t.cached_index);
    wp::adj_atomic_add(&p->vertex_index, t.vertex_index);
}




struct GJKResult_8184e255
{
    bool separated;
    wp::float32 dist;
    wp::vec_t<3, wp::float32> x1;
    wp::vec_t<3, wp::float32> x2;
    wp::int32 dim;
    wp::mat_t<4, 3, wp::float32> simplex;
    wp::mat_t<4, 3, wp::float32> simplex1;
    wp::mat_t<4, 3, wp::float32> simplex2;
    wp::vec_t<4, wp::int32> simplex_index1;
    wp::vec_t<4, wp::int32> simplex_index2;
    wp::int32 index1;
    wp::int32 index2;


    GJKResult_8184e255() = default;
    CUDA_CALLABLE GJKResult_8184e255(bool const& separated,
    wp::float32 const& dist = {},
    wp::vec_t<3, wp::float32> const& x1 = {},
    wp::vec_t<3, wp::float32> const& x2 = {},
    wp::int32 const& dim = {},
    wp::mat_t<4, 3, wp::float32> const& simplex = {},
    wp::mat_t<4, 3, wp::float32> const& simplex1 = {},
    wp::mat_t<4, 3, wp::float32> const& simplex2 = {},
    wp::vec_t<4, wp::int32> const& simplex_index1 = {},
    wp::vec_t<4, wp::int32> const& simplex_index2 = {},
    wp::int32 const& index1 = {},
    wp::int32 const& index2 = {})
        : separated{separated}
        , dist{dist}
        , x1{x1}
        , x2{x2}
        , dim{dim}
        , simplex{simplex}
        , simplex1{simplex1}
        , simplex2{simplex2}
        , simplex_index1{simplex_index1}
        , simplex_index2{simplex_index2}
        , index1{index1}
        , index2{index2}

    {
    }

    CUDA_CALLABLE GJKResult_8184e255& operator += (const GJKResult_8184e255& rhs)
    {    dist += rhs.dist;
    x1 += rhs.x1;
    x2 += rhs.x2;
    dim += rhs.dim;
    simplex += rhs.simplex;
    simplex1 += rhs.simplex1;
    simplex2 += rhs.simplex2;
    simplex_index1 += rhs.simplex_index1;
    simplex_index2 += rhs.simplex_index2;
    index1 += rhs.index1;
    index2 += rhs.index2;

        return *this;}

};

static CUDA_CALLABLE void adj_GJKResult_8184e255(bool const&,
    wp::float32 const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::int32 const&,
    wp::mat_t<4, 3, wp::float32> const&,
    wp::mat_t<4, 3, wp::float32> const&,
    wp::mat_t<4, 3, wp::float32> const&,
    wp::vec_t<4, wp::int32> const&,
    wp::vec_t<4, wp::int32> const&,
    wp::int32 const&,
    wp::int32 const&,
    bool & adj_separated,
    wp::float32 & adj_dist,
    wp::vec_t<3, wp::float32> & adj_x1,
    wp::vec_t<3, wp::float32> & adj_x2,
    wp::int32 & adj_dim,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::mat_t<4, 3, wp::float32> & adj_simplex1,
    wp::mat_t<4, 3, wp::float32> & adj_simplex2,
    wp::vec_t<4, wp::int32> & adj_simplex_index1,
    wp::vec_t<4, wp::int32> & adj_simplex_index2,
    wp::int32 & adj_index1,
    wp::int32 & adj_index2,
    GJKResult_8184e255 & adj_ret)
{
    adj_separated += adj_ret.separated;
    adj_dist += adj_ret.dist;
    adj_x1 += adj_ret.x1;
    adj_x2 += adj_ret.x2;
    adj_dim += adj_ret.dim;
    adj_simplex += adj_ret.simplex;
    adj_simplex1 += adj_ret.simplex1;
    adj_simplex2 += adj_ret.simplex2;
    adj_simplex_index1 += adj_ret.simplex_index1;
    adj_simplex_index2 += adj_ret.simplex_index2;
    adj_index1 += adj_ret.index1;
    adj_index2 += adj_ret.index2;
}

// Required when compiling adjoints.
CUDA_CALLABLE GJKResult_8184e255 add(const GJKResult_8184e255& a, const GJKResult_8184e255& b)
{
    return GJKResult_8184e255();
}

CUDA_CALLABLE void adj_atomic_add(GJKResult_8184e255* p, GJKResult_8184e255 t)
{
    wp::adj_atomic_add(&p->separated, t.separated);
    wp::adj_atomic_add(&p->dist, t.dist);
    wp::adj_atomic_add(&p->x1, t.x1);
    wp::adj_atomic_add(&p->x2, t.x2);
    wp::adj_atomic_add(&p->dim, t.dim);
    wp::adj_atomic_add(&p->simplex, t.simplex);
    wp::adj_atomic_add(&p->simplex1, t.simplex1);
    wp::adj_atomic_add(&p->simplex2, t.simplex2);
    wp::adj_atomic_add(&p->simplex_index1, t.simplex_index1);
    wp::adj_atomic_add(&p->simplex_index2, t.simplex_index2);
    wp::adj_atomic_add(&p->index1, t.index1);
    wp::adj_atomic_add(&p->index2, t.index2);
}




struct Polytope_10582b13
{
    wp::int32 status;
    wp::array_t<wp::vec_t<3, wp::float32>> vert;
    wp::array_t<wp::int32> vert_index;
    wp::int32 nvert;
    wp::vec_t<3, wp::float32> center;
    wp::array_t<wp::int32> face;
    wp::array_t<wp::vec_t<3, wp::float32>> face_pr;
    wp::array_t<wp::float32> face_norm2;
    wp::int32 nface;
    wp::array_t<wp::int32> horizon;
    wp::int32 nhorizon;


    Polytope_10582b13() = default;
    CUDA_CALLABLE Polytope_10582b13(wp::int32 const& status,
    wp::array_t<wp::vec_t<3, wp::float32>> const& vert = {},
    wp::array_t<wp::int32> const& vert_index = {},
    wp::int32 const& nvert = {},
    wp::vec_t<3, wp::float32> const& center = {},
    wp::array_t<wp::int32> const& face = {},
    wp::array_t<wp::vec_t<3, wp::float32>> const& face_pr = {},
    wp::array_t<wp::float32> const& face_norm2 = {},
    wp::int32 const& nface = {},
    wp::array_t<wp::int32> const& horizon = {},
    wp::int32 const& nhorizon = {})
        : status{status}
        , vert{vert}
        , vert_index{vert_index}
        , nvert{nvert}
        , center{center}
        , face{face}
        , face_pr{face_pr}
        , face_norm2{face_norm2}
        , nface{nface}
        , horizon{horizon}
        , nhorizon{nhorizon}

    {
    }

    CUDA_CALLABLE Polytope_10582b13& operator += (const Polytope_10582b13& rhs)
    {    status += rhs.status;
    nvert += rhs.nvert;
    center += rhs.center;
    nface += rhs.nface;
    nhorizon += rhs.nhorizon;

        return *this;}

};

static CUDA_CALLABLE void adj_Polytope_10582b13(wp::int32 const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::array_t<wp::int32> const&,
    wp::int32 const&,
    wp::vec_t<3, wp::float32> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::array_t<wp::float32> const&,
    wp::int32 const&,
    wp::array_t<wp::int32> const&,
    wp::int32 const&,
    wp::int32 & adj_status,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_vert,
    wp::array_t<wp::int32> & adj_vert_index,
    wp::int32 & adj_nvert,
    wp::vec_t<3, wp::float32> & adj_center,
    wp::array_t<wp::int32> & adj_face,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_face_pr,
    wp::array_t<wp::float32> & adj_face_norm2,
    wp::int32 & adj_nface,
    wp::array_t<wp::int32> & adj_horizon,
    wp::int32 & adj_nhorizon,
    Polytope_10582b13 & adj_ret)
{
    adj_status += adj_ret.status;
    adj_vert = adj_ret.vert;
    adj_vert_index = adj_ret.vert_index;
    adj_nvert += adj_ret.nvert;
    adj_center += adj_ret.center;
    adj_face = adj_ret.face;
    adj_face_pr = adj_ret.face_pr;
    adj_face_norm2 = adj_ret.face_norm2;
    adj_nface += adj_ret.nface;
    adj_horizon = adj_ret.horizon;
    adj_nhorizon += adj_ret.nhorizon;
}

// Required when compiling adjoints.
CUDA_CALLABLE Polytope_10582b13 add(const Polytope_10582b13& a, const Polytope_10582b13& b)
{
    return Polytope_10582b13();
}

CUDA_CALLABLE void adj_atomic_add(Polytope_10582b13* p, Polytope_10582b13 t)
{
    wp::adj_atomic_add(&p->status, t.status);
    wp::adj_atomic_add(&p->vert, t.vert);
    wp::adj_atomic_add(&p->vert_index, t.vert_index);
    wp::adj_atomic_add(&p->nvert, t.nvert);
    wp::adj_atomic_add(&p->center, t.center);
    wp::adj_atomic_add(&p->face, t.face);
    wp::adj_atomic_add(&p->face_pr, t.face_pr);
    wp::adj_atomic_add(&p->face_norm2, t.face_norm2);
    wp::adj_atomic_add(&p->nface, t.nface);
    wp::adj_atomic_add(&p->horizon, t.horizon);
    wp::adj_atomic_add(&p->nhorizon, t.nhorizon);
}




// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:119
static CUDA_CALLABLE SupportPoint_e82efc60 support_0(
    Geom_4022bfa7 var_geom,
    wp::int32 var_geomtype,
    wp::vec_t<3, wp::float32> var_dir)
{
    //---------
    // primal vars
    SupportPoint_e82efc60 var_0;
    const wp::int32 var_1 = -1;
    const wp::int32 var_2 = -1;
    const wp::int32 var_3 = 2;
    bool var_4;
    wp::vec_t<3, wp::float32>* var_5;
    wp::vec_t<3, wp::float32>* var_6;
    const wp::int32 var_7 = 0;
    wp::float32 var_8;
    wp::vec_t<3, wp::float32> var_9;
    const wp::float32 var_10 = 0.5;
    wp::float32* var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::vec_t<3, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::mat_t<3, 3, wp::float32>* var_18;
    wp::mat_t<3, 3, wp::float32> var_19;
    wp::mat_t<3, 3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    const wp::int32 var_22 = 6;
    bool var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32>* var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::mat_t<3, 3, wp::float32>* var_28;
    wp::vec_t<3, wp::float32> var_29;
    wp::mat_t<3, 3, wp::float32> var_30;
    wp::vec_t<3, wp::float32>* var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    const wp::int32 var_34 = 0;
    wp::float32 var_35;
    const wp::float32 var_36 = 0.0;
    bool var_37;
    const wp::int32 var_38 = 1;
    const wp::int32 var_39 = 0;
    wp::int32 var_40;
    const wp::int32 var_41 = 1;
    wp::float32 var_42;
    const wp::float32 var_43 = 0.0;
    bool var_44;
    const wp::int32 var_45 = 2;
    const wp::int32 var_46 = 0;
    wp::int32 var_47;
    wp::int32* var_48;
    wp::int32 var_49;
    wp::int32 var_50;
    const wp::int32 var_51 = 2;
    wp::float32 var_52;
    const wp::float32 var_53 = 0.0;
    bool var_54;
    const wp::int32 var_55 = 4;
    const wp::int32 var_56 = 0;
    wp::int32 var_57;
    wp::int32* var_58;
    wp::int32 var_59;
    wp::int32 var_60;
    const wp::int32 var_61 = 3;
    bool var_62;
    wp::vec_t<3, wp::float32>* var_63;
    const wp::int32 var_64 = 0;
    wp::float32 var_65;
    wp::vec_t<3, wp::float32> var_66;
    wp::vec_t<3, wp::float32> var_67;
    const wp::int32 var_68 = 2;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::vec_t<3, wp::float32>* var_71;
    const wp::int32 var_72 = 1;
    wp::float32 var_73;
    wp::vec_t<3, wp::float32> var_74;
    wp::float32 var_75;
    const wp::int32 var_76 = 2;
    wp::mat_t<3, 3, wp::float32>* var_77;
    wp::vec_t<3, wp::float32> var_78;
    wp::mat_t<3, 3, wp::float32> var_79;
    wp::vec_t<3, wp::float32>* var_80;
    wp::vec_t<3, wp::float32> var_81;
    wp::vec_t<3, wp::float32> var_82;
    wp::vec_t<3, wp::float32> var_83;
    const wp::int32 var_84 = 4;
    bool var_85;
    wp::vec_t<3, wp::float32>* var_86;
    wp::vec_t<3, wp::float32> var_87;
    wp::vec_t<3, wp::float32> var_88;
    wp::vec_t<3, wp::float32> var_89;
    wp::vec_t<3, wp::float32>* var_90;
    wp::vec_t<3, wp::float32> var_91;
    wp::vec_t<3, wp::float32> var_92;
    wp::mat_t<3, 3, wp::float32>* var_93;
    wp::vec_t<3, wp::float32> var_94;
    wp::mat_t<3, 3, wp::float32> var_95;
    wp::vec_t<3, wp::float32>* var_96;
    wp::vec_t<3, wp::float32> var_97;
    wp::vec_t<3, wp::float32> var_98;
    wp::vec_t<3, wp::float32> var_99;
    const wp::int32 var_100 = 5;
    bool var_101;
    const wp::float32 var_102 = 0.0;
    const wp::float32 var_103 = 0.0;
    const wp::float32 var_104 = 0.0;
    wp::vec_t<3, wp::float32> var_105;
    const wp::int32 var_106 = 0;
    wp::float32 var_107;
    const wp::int32 var_108 = 0;
    wp::float32 var_109;
    wp::float32 var_110;
    const wp::int32 var_111 = 1;
    wp::float32 var_112;
    const wp::int32 var_113 = 1;
    wp::float32 var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    const wp::float32 var_118 = 1e-15;
    bool var_119;
    wp::vec_t<3, wp::float32>* var_120;
    const wp::int32 var_121 = 0;
    wp::float32 var_122;
    wp::vec_t<3, wp::float32> var_123;
    wp::float32 var_124;
    const wp::int32 var_125 = 0;
    wp::float32 var_126;
    wp::float32 var_127;
    const wp::int32 var_128 = 0;
    const wp::int32 var_129 = 1;
    wp::float32 var_130;
    wp::float32 var_131;
    const wp::int32 var_132 = 1;
    const wp::int32 var_133 = 2;
    wp::float32 var_134;
    const wp::float32 var_135 = 0.0;
    bool var_136;
    wp::vec_t<3, wp::float32>* var_137;
    const wp::int32 var_138 = 1;
    wp::float32 var_139;
    wp::vec_t<3, wp::float32> var_140;
    wp::vec_t<3, wp::float32>* var_141;
    const wp::int32 var_142 = 1;
    wp::float32 var_143;
    wp::vec_t<3, wp::float32> var_144;
    wp::float32 var_145;
    wp::float32 var_146;
    const wp::int32 var_147 = 2;
    const wp::int32 var_148 = 2;
    wp::float32 var_149;
    const wp::float32 var_150 = 0.0;
    bool var_151;
    const wp::int32 var_152 = 0;
    const wp::int32 var_153 = 1;
    wp::int32 var_154;
    wp::mat_t<3, 3, wp::float32>* var_155;
    wp::vec_t<3, wp::float32> var_156;
    wp::mat_t<3, 3, wp::float32> var_157;
    wp::vec_t<3, wp::float32>* var_158;
    wp::vec_t<3, wp::float32> var_159;
    wp::vec_t<3, wp::float32> var_160;
    wp::vec_t<3, wp::float32> var_161;
    const wp::int32 var_162 = 7;
    bool var_163;
    const wp::float32 var_164 = -1e+30;
    wp::float32 var_165;
    bool var_166;
    wp::int32* var_167;
    const wp::int32 var_168 = -1;
    bool var_169;
    wp::int32 var_170;
    wp::int32* var_171;
    const wp::int32 var_172 = 10;
    bool var_173;
    wp::int32 var_174;
    wp::int32* var_175;
    const wp::int32 var_176 = -1;
    bool var_177;
    wp::int32 var_178;
    wp::int32* var_179;
    wp::int32 var_180;
    wp::int32 var_181;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_182;
    wp::int32* var_183;
    wp::vec_t<3, wp::float32>* var_184;
    wp::array_t<wp::vec_t<3, wp::float32>> var_185;
    wp::int32 var_186;
    wp::float32 var_187;
    wp::vec_t<3, wp::float32> var_188;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_189;
    wp::int32* var_190;
    wp::vec_t<3, wp::float32>* var_191;
    wp::array_t<wp::vec_t<3, wp::float32>> var_192;
    wp::int32 var_193;
    wp::vec_t<3, wp::float32> var_194;
    wp::vec_t<3, wp::float32> var_195;
    wp::float32 var_196;
    wp::int32* var_197;
    wp::range_t var_198;
    wp::int32 var_199;
    wp::int32 var_200;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_201;
    wp::int32* var_202;
    wp::int32 var_203;
    wp::int32 var_204;
    wp::vec_t<3, wp::float32>* var_205;
    wp::array_t<wp::vec_t<3, wp::float32>> var_206;
    wp::vec_t<3, wp::float32> var_207;
    wp::vec_t<3, wp::float32> var_208;
    wp::float32 var_209;
    bool var_210;
    wp::float32 var_211;
    wp::int32* var_212;
    wp::int32 var_213;
    wp::int32 var_214;
    wp::float32 var_215;
    wp::int32* var_216;
    wp::int32* var_217;
    wp::int32 var_218;
    wp::int32 var_219;
    wp::int32 var_220;
    wp::float32 var_221;
    wp::array_t<wp::int32>* var_222;
    wp::int32* var_223;
    wp::int32* var_224;
    wp::array_t<wp::int32> var_225;
    wp::int32 var_226;
    wp::int32 var_227;
    wp::int32 var_228;
    wp::int32* var_229;
    const wp::int32 var_230 = 2;
    wp::int32 var_231;
    wp::int32 var_232;
    wp::int32* var_233;
    const wp::int32 var_234 = 2;
    wp::int32 var_235;
    wp::int32 var_236;
    wp::int32 var_237;
    wp::int32* var_238;
    const wp::int32 var_239 = 2;
    wp::int32 var_240;
    wp::int32 var_241;
    const wp::int32 var_242 = 2;
    wp::int32 var_243;
    wp::int32 var_244;
    const wp::int32 var_245 = -1;
    wp::int32 var_246;
    wp::int32* var_247;
    const wp::int32 var_248 = -1;
    bool var_249;
    wp::int32 var_250;
    wp::int32* var_251;
    const wp::int32 var_252 = 0;
    wp::int32 var_253;
    wp::int32 var_254;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_255;
    wp::int32* var_256;
    wp::array_t<wp::int32>* var_257;
    wp::int32 var_258;
    wp::int32* var_259;
    wp::array_t<wp::int32> var_260;
    wp::int32 var_261;
    wp::int32 var_262;
    wp::int32 var_263;
    wp::vec_t<3, wp::float32>* var_264;
    wp::array_t<wp::vec_t<3, wp::float32>> var_265;
    wp::float32 var_266;
    wp::vec_t<3, wp::float32> var_267;
    bool var_268;
    wp::int32 var_269;
    wp::array_t<wp::int32>* var_270;
    wp::int32 var_271;
    wp::int32* var_272;
    wp::array_t<wp::int32> var_273;
    wp::int32 var_274;
    wp::int32 var_275;
    wp::array_t<wp::int32>* var_276;
    wp::int32 var_277;
    wp::int32* var_278;
    wp::array_t<wp::int32> var_279;
    wp::int32 var_280;
    wp::int32 var_281;
    const wp::int32 var_282 = 0;
    bool var_283;
    wp::array_t<wp::int32>* var_284;
    wp::int32 var_285;
    wp::int32* var_286;
    wp::array_t<wp::int32> var_287;
    wp::int32 var_288;
    wp::int32 var_289;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_290;
    wp::int32* var_291;
    wp::int32 var_292;
    wp::int32 var_293;
    wp::vec_t<3, wp::float32>* var_294;
    wp::array_t<wp::vec_t<3, wp::float32>> var_295;
    wp::float32 var_296;
    wp::vec_t<3, wp::float32> var_297;
    bool var_298;
    wp::int32 var_299;
    bool var_300;
    wp::float32 var_301;
    const wp::int32 var_302 = 1;
    wp::int32 var_303;
    wp::array_t<wp::int32>* var_304;
    wp::int32 var_305;
    wp::int32* var_306;
    wp::array_t<wp::int32> var_307;
    wp::int32 var_308;
    wp::int32 var_309;
    wp::array_t<wp::int32>* var_310;
    wp::int32 var_311;
    wp::int32* var_312;
    wp::array_t<wp::int32> var_313;
    wp::int32 var_314;
    wp::int32 var_315;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_316;
    wp::int32* var_317;
    wp::int32* var_318;
    wp::int32 var_319;
    wp::int32 var_320;
    wp::int32 var_321;
    wp::vec_t<3, wp::float32>* var_322;
    wp::array_t<wp::vec_t<3, wp::float32>> var_323;
    wp::vec_t<3, wp::float32> var_324;
    wp::vec_t<3, wp::float32> var_325;
    wp::float32 var_326;
    wp::mat_t<3, 3, wp::float32>* var_327;
    wp::vec_t<3, wp::float32>* var_328;
    wp::vec_t<3, wp::float32> var_329;
    wp::mat_t<3, 3, wp::float32> var_330;
    wp::vec_t<3, wp::float32> var_331;
    wp::vec_t<3, wp::float32>* var_332;
    wp::vec_t<3, wp::float32> var_333;
    wp::vec_t<3, wp::float32> var_334;
    const wp::int32 var_335 = 1;
    bool var_336;
    wp::float32 var_337;
    const wp::int32 var_338 = 2;
    wp::float32 var_339;
    const wp::float32 var_340 = 0.0;
    bool var_341;
    const wp::int32 var_342 = -2;
    const wp::int32 var_343 = -3;
    wp::int32 var_344;
    const wp::int32 var_345 = 0;
    wp::mat_t<6, 3, wp::float32>* var_346;
    wp::vec_t<3, wp::float32> var_347;
    wp::mat_t<6, 3, wp::float32> var_348;
    wp::float32 var_349;
    bool var_350;
    wp::float32 var_351;
    wp::float32 var_352;
    const wp::int32 var_353 = 1;
    wp::mat_t<6, 3, wp::float32>* var_354;
    wp::vec_t<3, wp::float32> var_355;
    wp::mat_t<6, 3, wp::float32> var_356;
    wp::float32 var_357;
    bool var_358;
    wp::float32 var_359;
    wp::float32 var_360;
    const wp::int32 var_361 = 2;
    wp::mat_t<6, 3, wp::float32>* var_362;
    wp::vec_t<3, wp::float32> var_363;
    wp::mat_t<6, 3, wp::float32> var_364;
    wp::float32 var_365;
    bool var_366;
    wp::float32 var_367;
    wp::float32 var_368;
    const wp::int32 var_369 = 3;
    wp::mat_t<6, 3, wp::float32>* var_370;
    wp::vec_t<3, wp::float32> var_371;
    wp::mat_t<6, 3, wp::float32> var_372;
    wp::float32 var_373;
    bool var_374;
    wp::float32 var_375;
    wp::float32 var_376;
    const wp::int32 var_377 = 4;
    wp::mat_t<6, 3, wp::float32>* var_378;
    wp::vec_t<3, wp::float32> var_379;
    wp::mat_t<6, 3, wp::float32> var_380;
    wp::float32 var_381;
    bool var_382;
    wp::float32 var_383;
    wp::float32 var_384;
    const wp::int32 var_385 = 5;
    wp::mat_t<6, 3, wp::float32>* var_386;
    wp::vec_t<3, wp::float32> var_387;
    wp::mat_t<6, 3, wp::float32> var_388;
    wp::float32 var_389;
    bool var_390;
    wp::float32 var_391;
    wp::float32 var_392;
    wp::float32 var_393;
    wp::int32 var_394;
    wp::vec_t<3, wp::float32> var_395;
    wp::float32 var_396;
    const wp::int32 var_397 = 999;
    bool var_398;
    wp::mat_t<3, 3, wp::float32>* var_399;
    const wp::int32 var_400 = 0;
    wp::slice_t var_401;
    const wp::int32 var_402 = 0;
    const wp::int32 var_403 = 3;
    const wp::int32 var_404 = 1;
    wp::vec_t<3, wp::float32> var_405;
    wp::mat_t<3, 3, wp::float32> var_406;
    wp::mat_t<3, 3, wp::float32>* var_407;
    const wp::int32 var_408 = 1;
    wp::slice_t var_409;
    const wp::int32 var_410 = 0;
    const wp::int32 var_411 = 3;
    const wp::int32 var_412 = 1;
    wp::vec_t<3, wp::float32> var_413;
    wp::mat_t<3, 3, wp::float32> var_414;
    wp::mat_t<3, 3, wp::float32>* var_415;
    const wp::int32 var_416 = 2;
    wp::slice_t var_417;
    const wp::int32 var_418 = 0;
    const wp::int32 var_419 = 3;
    const wp::int32 var_420 = 1;
    wp::vec_t<3, wp::float32> var_421;
    wp::mat_t<3, 3, wp::float32> var_422;
    wp::float32 var_423;
    wp::float32 var_424;
    wp::float32 var_425;
    bool var_426;
    bool var_427;
    bool var_428;
    bool var_429;
    const wp::int32 var_430 = 105;
    bool var_431;
    wp::mat_t<6, 3, wp::float32>* var_432;
    const wp::int32 var_433 = 0;
    wp::vec_t<3, wp::float32> var_434;
    wp::mat_t<6, 3, wp::float32> var_435;
    wp::mat_t<6, 3, wp::float32>* var_436;
    const wp::int32 var_437 = 1;
    wp::vec_t<3, wp::float32> var_438;
    wp::mat_t<6, 3, wp::float32> var_439;
    wp::mat_t<6, 3, wp::float32>* var_440;
    const wp::int32 var_441 = 2;
    wp::vec_t<3, wp::float32> var_442;
    wp::mat_t<6, 3, wp::float32> var_443;
    wp::mat_t<6, 3, wp::float32>* var_444;
    const wp::int32 var_445 = 3;
    wp::vec_t<3, wp::float32> var_446;
    wp::mat_t<6, 3, wp::float32> var_447;
    wp::float32 var_448;
    wp::float32 var_449;
    wp::float32 var_450;
    wp::float32 var_451;
    bool var_452;
    bool var_453;
    bool var_454;
    bool var_455;
    bool var_456;
    bool var_457;
    bool var_458;
    bool var_459;
    wp::float32 var_460;
    wp::float32 var_461;
    wp::float32 var_462;
    wp::float32 var_463;
    wp::float32 var_464;
    wp::float32 var_465;
    wp::float32 var_466;
    wp::int32 var_467;
    wp::vec_t<3, wp::float32> var_468;
    wp::float32 var_469;
    wp::vec_t<3, wp::float32> var_470;
    wp::vec_t<3, wp::float32> var_471;
    wp::vec_t<3, wp::float32> var_472;
    wp::float32* var_473;
    const wp::float32 var_474 = 0.0;
    bool var_475;
    wp::float32 var_476;
    const wp::float32 var_477 = 0.5;
    wp::float32* var_478;
    wp::float32 var_479;
    wp::float32 var_480;
    wp::vec_t<3, wp::float32> var_481;
    wp::vec_t<3, wp::float32>* var_482;
    wp::vec_t<3, wp::float32> var_483;
    wp::vec_t<3, wp::float32> var_484;
    //---------
    // forward
    // def support(geom: Geom, geomtype: int, dir: wp.vec3) -> SupportPoint:                  <L 120>
    // sp = SupportPoint()                                                                    <L 121>
    var_0 = SupportPoint_e82efc60();
    // sp.cached_index = -1                                                                   <L 122>
    var_0.cached_index = var_1;
    // sp.vertex_index = -1                                                                   <L 123>
    var_0.vertex_index = var_2;
    // if geomtype == GeomType.SPHERE:                                                        <L 124>
    var_4 = (var_geomtype == var_3);
    if (var_4) {
        // sp.point = geom.pos + (geom.size[0] + 0.5 * geom.margin) * dir                     <L 125>
        var_5 = &((var_geom).pos);
        var_6 = &((var_geom).size);
        var_9 = wp::load(var_6);
        var_8 = wp::extract(var_9, var_7);
        var_11 = &((var_geom).margin);
        var_13 = wp::load(var_11);
        var_12 = wp::mul(var_10, var_13);
        var_14 = wp::add(var_8, var_12);
        var_15 = wp::mul(var_14, var_dir);
        var_17 = wp::load(var_5);
        var_16 = wp::add(var_17, var_15);
        var_0.point = var_16;
        // return sp                                                                          <L 126>
        return var_0;
    }
    // local_dir = wp.transpose(geom.rot) @ dir                                               <L 128>
    var_18 = &((var_geom).rot);
    var_20 = wp::load(var_18);
    var_19 = wp::transpose(var_20);
    var_21 = wp::mul(var_19, var_dir);
    // if geomtype == GeomType.BOX:                                                           <L 129>
    var_23 = (var_geomtype == var_22);
    if (var_23) {
        // tmp = wp.sign(local_dir)                                                           <L 130>
        var_24 = wp::sign(var_21);
        // res = wp.cw_mul(tmp, geom.size)                                                    <L 131>
        var_25 = &((var_geom).size);
        var_27 = wp::load(var_25);
        var_26 = wp::cw_mul(var_24, var_27);
        // sp.point = geom.rot @ res + geom.pos                                               <L 132>
        var_28 = &((var_geom).rot);
        var_30 = wp::load(var_28);
        var_29 = wp::mul(var_30, var_26);
        var_31 = &((var_geom).pos);
        var_33 = wp::load(var_31);
        var_32 = wp::add(var_29, var_33);
        var_0.point = var_32;
        // sp.vertex_index = wp.where(tmp[0] > 0.0, 1, 0)                                     <L 133>
        var_35 = wp::extract(var_24, var_34);
        var_37 = (var_35 > var_36);
        var_40 = wp::where(var_37, var_38, var_39);
        var_0.vertex_index = var_40;
        // sp.vertex_index += wp.where(tmp[1] > 0.0, 2, 0)                                    <L 134>
        var_42 = wp::extract(var_24, var_41);
        var_44 = (var_42 > var_43);
        var_47 = wp::where(var_44, var_45, var_46);
        var_48 = &((var_0).vertex_index);
        var_50 = wp::load(var_48);
        var_49 = wp::add(var_50, var_47);
        var_0.vertex_index = var_49;
        // sp.vertex_index += wp.where(tmp[2] > 0.0, 4, 0)                                    <L 135>
        var_52 = wp::extract(var_24, var_51);
        var_54 = (var_52 > var_53);
        var_57 = wp::where(var_54, var_55, var_56);
        var_58 = &((var_0).vertex_index);
        var_60 = wp::load(var_58);
        var_59 = wp::add(var_60, var_57);
        var_0.vertex_index = var_59;
    }
    if (!var_23) {
        // elif geomtype == GeomType.CAPSULE:                                                 <L 136>
        var_62 = (var_geomtype == var_61);
        if (var_62) {
            // res = local_dir * geom.size[0]                                                 <L 137>
            var_63 = &((var_geom).size);
            var_66 = wp::load(var_63);
            var_65 = wp::extract(var_66, var_64);
            var_67 = wp::mul(var_21, var_65);
            // res[2] += wp.sign(local_dir[2]) * geom.size[1]                                 <L 139>
            var_69 = wp::extract(var_21, var_68);
            var_70 = wp::sign(var_69);
            var_71 = &((var_geom).size);
            var_74 = wp::load(var_71);
            var_73 = wp::extract(var_74, var_72);
            var_75 = wp::mul(var_70, var_73);
            wp::add_inplace(var_67, var_76, var_75);
            // sp.point = geom.rot @ res + geom.pos                                           <L 140>
            var_77 = &((var_geom).rot);
            var_79 = wp::load(var_77);
            var_78 = wp::mul(var_79, var_67);
            var_80 = &((var_geom).pos);
            var_82 = wp::load(var_80);
            var_81 = wp::add(var_78, var_82);
            var_0.point = var_81;
        }
        var_83 = wp::where(var_62, var_67, var_26);
        if (!var_62) {
            // elif geomtype == GeomType.ELLIPSOID:                                           <L 141>
            var_85 = (var_geomtype == var_84);
            if (var_85) {
                // res = wp.cw_mul(local_dir, geom.size)                                      <L 142>
                var_86 = &((var_geom).size);
                var_88 = wp::load(var_86);
                var_87 = wp::cw_mul(var_21, var_88);
                // res = wp.normalize(res)                                                    <L 143>
                var_89 = wp::normalize(var_87);
                // res = wp.cw_mul(res, geom.size)                                            <L 145>
                var_90 = &((var_geom).size);
                var_92 = wp::load(var_90);
                var_91 = wp::cw_mul(var_89, var_92);
                // sp.point = geom.rot @ res + geom.pos                                       <L 146>
                var_93 = &((var_geom).rot);
                var_95 = wp::load(var_93);
                var_94 = wp::mul(var_95, var_91);
                var_96 = &((var_geom).pos);
                var_98 = wp::load(var_96);
                var_97 = wp::add(var_94, var_98);
                var_0.point = var_97;
            }
            var_99 = wp::where(var_85, var_91, var_83);
            if (!var_85) {
                // elif geomtype == GeomType.CYLINDER:                                        <L 147>
                var_101 = (var_geomtype == var_100);
                if (var_101) {
                    // res = wp.vec3(0.0, 0.0, 0.0)                                           <L 148>
                    var_105 = wp::vec_t<3, wp::float32>(var_102, var_103, var_104);
                    // d = wp.sqrt(local_dir[0] * local_dir[0] + local_dir[1] * local_dir[1])       <L 150>
                    var_107 = wp::extract(var_21, var_106);
                    var_109 = wp::extract(var_21, var_108);
                    var_110 = wp::mul(var_107, var_109);
                    var_112 = wp::extract(var_21, var_111);
                    var_114 = wp::extract(var_21, var_113);
                    var_115 = wp::mul(var_112, var_114);
                    var_116 = wp::add(var_110, var_115);
                    var_117 = wp::sqrt(var_116);
                    // if d > MINVAL:                                                         <L 151>
                    var_119 = (var_117 > var_118);
                    if (var_119) {
                        // scl = geom.size[0] / d                                             <L 152>
                        var_120 = &((var_geom).size);
                        var_123 = wp::load(var_120);
                        var_122 = wp::extract(var_123, var_121);
                        var_124 = wp::div(var_122, var_117);
                        // res[0] = local_dir[0] * scl                                        <L 153>
                        var_126 = wp::extract(var_21, var_125);
                        var_127 = wp::mul(var_126, var_124);
                        wp::assign_inplace(var_105, var_128, var_127);
                        // res[1] = local_dir[1] * scl                                        <L 154>
                        var_130 = wp::extract(var_21, var_129);
                        var_131 = wp::mul(var_130, var_124);
                        wp::assign_inplace(var_105, var_132, var_131);
                    }
                    // res[2] = wp.where(local_dir[2] >= 0.0, geom.size[1], -geom.size[1])       <L 156>
                    var_134 = wp::extract(var_21, var_133);
                    var_136 = (var_134 >= var_135);
                    var_137 = &((var_geom).size);
                    var_140 = wp::load(var_137);
                    var_139 = wp::extract(var_140, var_138);
                    var_141 = &((var_geom).size);
                    var_144 = wp::load(var_141);
                    var_143 = wp::extract(var_144, var_142);
                    var_145 = wp::neg(var_143);
                    var_146 = wp::where(var_136, var_139, var_145);
                    wp::assign_inplace(var_105, var_147, var_146);
                    // sp.vertex_index = wp.where(local_dir[2] >= 0.0, 0, 1)                  <L 157>
                    var_149 = wp::extract(var_21, var_148);
                    var_151 = (var_149 >= var_150);
                    var_154 = wp::where(var_151, var_152, var_153);
                    var_0.vertex_index = var_154;
                    // sp.point = geom.rot @ res + geom.pos                                   <L 158>
                    var_155 = &((var_geom).rot);
                    var_157 = wp::load(var_155);
                    var_156 = wp::mul(var_157, var_105);
                    var_158 = &((var_geom).pos);
                    var_160 = wp::load(var_158);
                    var_159 = wp::add(var_156, var_160);
                    var_0.point = var_159;
                }
                var_161 = wp::where(var_101, var_105, var_99);
                if (!var_101) {
                    // elif geomtype == GeomType.MESH:                                        <L 159>
                    var_163 = (var_geomtype == var_162);
                    if (var_163) {
                        // max_dist = float(FLOAT_MIN)                                        <L 160>
                        var_165 = wp::float(var_164);
                        // if geom.graphadr == -1 or geom.vertnum < 10:                       <L 161>
                        var_167 = &((var_geom).graphadr);
                        var_170 = wp::load(var_167);
                        var_169 = (var_170 == var_168);
                        var_166 = var_169;
                        if (!var_166) {
                            var_171 = &((var_geom).vertnum);
                            var_174 = wp::load(var_171);
                            var_173 = (var_174 < var_172);
                            var_166 = var_166 || var_173;
                        }
                        if (var_166) {
                            // if geom.index > -1:                                            <L 162>
                            var_175 = &((var_geom).index);
                            var_178 = wp::load(var_175);
                            var_177 = (var_178 > var_176);
                            if (var_177) {
                                // sp.cached_index = geom.index                               <L 163>
                                var_179 = &((var_geom).index);
                                var_181 = wp::load(var_179);
                                var_180 = wp::copy(var_181);
                                var_0.cached_index = var_180;
                                // max_dist = wp.dot(geom.vert[geom.index], local_dir)        <L 164>
                                var_182 = &((var_geom).vert);
                                var_183 = &((var_geom).index);
                                var_185 = wp::load(var_182);
                                var_186 = wp::load(var_183);
                                var_184 = wp::address(var_185, var_186);
                                var_188 = wp::load(var_184);
                                var_187 = wp::dot(var_188, var_21);
                                // sp.point = geom.vert[geom.index]                           <L 165>
                                var_189 = &((var_geom).vert);
                                var_190 = &((var_geom).index);
                                var_192 = wp::load(var_189);
                                var_193 = wp::load(var_190);
                                var_191 = wp::address(var_192, var_193);
                                var_195 = wp::load(var_191);
                                var_194 = wp::copy(var_195);
                                var_0.point = var_194;
                            }
                            var_196 = wp::where(var_177, var_187, var_165);
                            // for i in range(geom.vertnum):                                  <L 167>
                            var_197 = &((var_geom).vertnum);
                            var_199 = wp::load(var_197);
                            var_198 = wp::range(var_199);
                            start_for_1:;
                                if (iter_cmp(var_198) == 0) goto end_for_1;
                                var_200 = wp::iter_next(var_198);
                                // vert = geom.vert[geom.vertadr + i]                         <L 168>
                                var_201 = &((var_geom).vert);
                                var_202 = &((var_geom).vertadr);
                                var_204 = wp::load(var_202);
                                var_203 = wp::add(var_204, var_200);
                                var_206 = wp::load(var_201);
                                var_205 = wp::address(var_206, var_203);
                                var_208 = wp::load(var_205);
                                var_207 = wp::copy(var_208);
                                // dist = wp.dot(vert, local_dir)                             <L 169>
                                var_209 = wp::dot(var_207, var_21);
                                // if dist > max_dist:                                        <L 170>
                                var_210 = (var_209 > var_196);
                                if (var_210) {
                                    // max_dist = dist                                        <L 171>
                                    var_211 = wp::copy(var_209);
                                    // sp.point = vert                                        <L 172>
                                    var_0.point = var_207;
                                    // sp.cached_index = geom.vertadr + i                     <L 173>
                                    var_212 = &((var_geom).vertadr);
                                    var_214 = wp::load(var_212);
                                    var_213 = wp::add(var_214, var_200);
                                    var_0.cached_index = var_213;
                                }
                                var_215 = wp::where(var_210, var_211, var_196);
                                wp::assign(var_196, var_215);
                                goto start_for_1;
                            end_for_1:;
                            // sp.vertex_index = sp.cached_index - geom.vertadr               <L 174>
                            var_216 = &((var_0).cached_index);
                            var_217 = &((var_geom).vertadr);
                            var_219 = wp::load(var_216);
                            var_220 = wp::load(var_217);
                            var_218 = wp::sub(var_219, var_220);
                            var_0.vertex_index = var_218;
                        }
                        var_221 = wp::where(var_166, var_196, var_165);
                        if (!var_166) {
                            // numvert = geom.graph[geom.graphadr]                            <L 176>
                            var_222 = &((var_geom).graph);
                            var_223 = &((var_geom).graphadr);
                            var_225 = wp::load(var_222);
                            var_226 = wp::load(var_223);
                            var_224 = wp::address(var_225, var_226);
                            var_228 = wp::load(var_224);
                            var_227 = wp::copy(var_228);
                            // vert_edgeadr = geom.graphadr + 2                               <L 177>
                            var_229 = &((var_geom).graphadr);
                            var_232 = wp::load(var_229);
                            var_231 = wp::add(var_232, var_230);
                            // vert_globalid = geom.graphadr + 2 + numvert                    <L 178>
                            var_233 = &((var_geom).graphadr);
                            var_236 = wp::load(var_233);
                            var_235 = wp::add(var_236, var_234);
                            var_237 = wp::add(var_235, var_227);
                            // edge_localid = geom.graphadr + 2 + 2 * numvert                 <L 179>
                            var_238 = &((var_geom).graphadr);
                            var_241 = wp::load(var_238);
                            var_240 = wp::add(var_241, var_239);
                            var_243 = wp::mul(var_242, var_227);
                            var_244 = wp::add(var_240, var_243);
                            // prev = int(-1)                                                 <L 180>
                            var_246 = wp::int(var_245);
                            // imax = wp.where(geom.index > -1, geom.index, 0)                <L 181>
                            var_247 = &((var_geom).index);
                            var_250 = wp::load(var_247);
                            var_249 = (var_250 > var_248);
                            var_251 = &((var_geom).index);
                            var_254 = wp::load(var_251);
                            var_253 = wp::where(var_249, var_254, var_252);
                            // max_dist = wp.dot(local_dir, geom.vert[geom.vertadr + geom.graph[vert_globalid + imax]])       <L 182>
                            var_255 = &((var_geom).vert);
                            var_256 = &((var_geom).vertadr);
                            var_257 = &((var_geom).graph);
                            var_258 = wp::add(var_237, var_253);
                            var_260 = wp::load(var_257);
                            var_259 = wp::address(var_260, var_258);
                            var_262 = wp::load(var_256);
                            var_263 = wp::load(var_259);
                            var_261 = wp::add(var_262, var_263);
                            var_265 = wp::load(var_255);
                            var_264 = wp::address(var_265, var_261);
                            var_267 = wp::load(var_264);
                            var_266 = wp::dot(var_21, var_267);
                            // while imax != prev:                                            <L 185>
    start_while_3:;
                            var_268 = (var_253 != var_246);
    if ((var_268) == false) goto end_while_3;
                                // prev = imax                                                <L 186>
                                var_269 = wp::copy(var_253);
                                // i = geom.graph[vert_edgeadr + imax]                        <L 187>
                                var_270 = &((var_geom).graph);
                                var_271 = wp::add(var_231, var_253);
                                var_273 = wp::load(var_270);
                                var_272 = wp::address(var_273, var_271);
                                var_275 = wp::load(var_272);
                                var_274 = wp::copy(var_275);
                                // subidx = geom.graph[edge_localid + i]                      <L 188>
                                var_276 = &((var_geom).graph);
                                var_277 = wp::add(var_244, var_274);
                                var_279 = wp::load(var_276);
                                var_278 = wp::address(var_279, var_277);
                                var_281 = wp::load(var_278);
                                var_280 = wp::copy(var_281);
                                // while subidx >= 0:                                         <L 189>
    start_while_5:;
                                var_283 = (var_280 >= var_282);
    if ((var_283) == false) goto end_while_5;
                                    // idx = geom.graph[vert_globalid + subidx]               <L 190>
                                    var_284 = &((var_geom).graph);
                                    var_285 = wp::add(var_237, var_280);
                                    var_287 = wp::load(var_284);
                                    var_286 = wp::address(var_287, var_285);
                                    var_289 = wp::load(var_286);
                                    var_288 = wp::copy(var_289);
                                    // dist = wp.dot(local_dir, geom.vert[geom.vertadr + idx])       <L 191>
                                    var_290 = &((var_geom).vert);
                                    var_291 = &((var_geom).vertadr);
                                    var_293 = wp::load(var_291);
                                    var_292 = wp::add(var_293, var_288);
                                    var_295 = wp::load(var_290);
                                    var_294 = wp::address(var_295, var_292);
                                    var_297 = wp::load(var_294);
                                    var_296 = wp::dot(var_21, var_297);
                                    // imax = wp.where(dist > max_dist, subidx, imax)         <L 192>
                                    var_298 = (var_296 > var_266);
                                    var_299 = wp::where(var_298, var_280, var_253);
                                    // max_dist = wp.where(dist > max_dist, dist, max_dist)       <L 193>
                                    var_300 = (var_296 > var_266);
                                    var_301 = wp::where(var_300, var_296, var_266);
                                    // i += 1                                                 <L 194>
                                    var_303 = wp::add(var_274, var_302);
                                    // subidx = geom.graph[edge_localid + i]                  <L 195>
                                    var_304 = &((var_geom).graph);
                                    var_305 = wp::add(var_244, var_303);
                                    var_307 = wp::load(var_304);
                                    var_306 = wp::address(var_307, var_305);
                                    var_309 = wp::load(var_306);
                                    var_308 = wp::copy(var_309);
                                    wp::assign(var_266, var_301);
                                    wp::assign(var_274, var_303);
                                    wp::assign(var_209, var_296);
                                    wp::assign(var_253, var_299);
                                    wp::assign(var_280, var_308);
    goto start_while_5;
    end_while_5:;
                                wp::assign(var_200, var_274);
                                wp::assign(var_246, var_269);
    goto start_while_3;
    end_while_3:;
                            // sp.cached_index = imax                                         <L 197>
                            var_0.cached_index = var_253;
                            // sp.vertex_index = geom.graph[vert_globalid + imax]             <L 198>
                            var_310 = &((var_geom).graph);
                            var_311 = wp::add(var_237, var_253);
                            var_313 = wp::load(var_310);
                            var_312 = wp::address(var_313, var_311);
                            var_315 = wp::load(var_312);
                            var_314 = wp::copy(var_315);
                            var_0.vertex_index = var_314;
                            // sp.point = geom.vert[geom.vertadr + sp.vertex_index]           <L 199>
                            var_316 = &((var_geom).vert);
                            var_317 = &((var_geom).vertadr);
                            var_318 = &((var_0).vertex_index);
                            var_320 = wp::load(var_317);
                            var_321 = wp::load(var_318);
                            var_319 = wp::add(var_320, var_321);
                            var_323 = wp::load(var_316);
                            var_322 = wp::address(var_323, var_319);
                            var_325 = wp::load(var_322);
                            var_324 = wp::copy(var_325);
                            var_0.point = var_324;
                        }
                        var_326 = wp::where(var_166, var_221, var_266);
                        // sp.point = geom.rot @ sp.point + geom.pos                          <L 201>
                        var_327 = &((var_geom).rot);
                        var_328 = &((var_0).point);
                        var_330 = wp::load(var_327);
                        var_331 = wp::load(var_328);
                        var_329 = wp::mul(var_330, var_331);
                        var_332 = &((var_geom).pos);
                        var_334 = wp::load(var_332);
                        var_333 = wp::add(var_329, var_334);
                        var_0.point = var_333;
                    }
                    if (!var_163) {
                        // elif geomtype == GeomType.HFIELD:                                  <L 202>
                        var_336 = (var_geomtype == var_335);
                        if (var_336) {
                            // max_dist = float(FLOAT_MIN)                                    <L 203>
                            var_337 = wp::float(var_164);
                            // sp.vertex_index = wp.where(dir[2] < 0.0, -2, -3)               <L 205>
                            var_339 = wp::extract(var_dir, var_338);
                            var_341 = (var_339 < var_340);
                            var_344 = wp::where(var_341, var_342, var_343);
                            var_0.vertex_index = var_344;
                            // for i in range(6):                                             <L 206>
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_346 = &((var_geom).polyvert);
                            var_348 = wp::load(var_346);
                            var_347 = wp::extract(var_348, var_345);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_349 = wp::dot(var_347, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_350 = (var_349 > var_337);
                            if (var_350) {
                                // max_dist = dist                                            <L 210>
                                var_351 = wp::copy(var_349);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_347;
                            }
                            var_352 = wp::where(var_350, var_351, var_337);
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_354 = &((var_geom).polyvert);
                            var_356 = wp::load(var_354);
                            var_355 = wp::extract(var_356, var_353);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_357 = wp::dot(var_355, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_358 = (var_357 > var_352);
                            if (var_358) {
                                // max_dist = dist                                            <L 210>
                                var_359 = wp::copy(var_357);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_355;
                            }
                            var_360 = wp::where(var_358, var_359, var_352);
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_362 = &((var_geom).polyvert);
                            var_364 = wp::load(var_362);
                            var_363 = wp::extract(var_364, var_361);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_365 = wp::dot(var_363, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_366 = (var_365 > var_360);
                            if (var_366) {
                                // max_dist = dist                                            <L 210>
                                var_367 = wp::copy(var_365);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_363;
                            }
                            var_368 = wp::where(var_366, var_367, var_360);
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_370 = &((var_geom).polyvert);
                            var_372 = wp::load(var_370);
                            var_371 = wp::extract(var_372, var_369);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_373 = wp::dot(var_371, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_374 = (var_373 > var_368);
                            if (var_374) {
                                // max_dist = dist                                            <L 210>
                                var_375 = wp::copy(var_373);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_371;
                            }
                            var_376 = wp::where(var_374, var_375, var_368);
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_378 = &((var_geom).polyvert);
                            var_380 = wp::load(var_378);
                            var_379 = wp::extract(var_380, var_377);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_381 = wp::dot(var_379, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_382 = (var_381 > var_376);
                            if (var_382) {
                                // max_dist = dist                                            <L 210>
                                var_383 = wp::copy(var_381);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_379;
                            }
                            var_384 = wp::where(var_382, var_383, var_376);
                            // vert = geom.polyvert[i]                                        <L 207>
                            var_386 = &((var_geom).polyvert);
                            var_388 = wp::load(var_386);
                            var_387 = wp::extract(var_388, var_385);
                            // dist = wp.dot(vert, dir)                                       <L 208>
                            var_389 = wp::dot(var_387, var_dir);
                            // if dist > max_dist:                                            <L 209>
                            var_390 = (var_389 > var_384);
                            if (var_390) {
                                // max_dist = dist                                            <L 210>
                                var_391 = wp::copy(var_389);
                                // sp.point = vert                                            <L 211>
                                var_0.point = var_387;
                            }
                            var_392 = wp::where(var_390, var_391, var_384);
                        }
                        var_393 = wp::where(var_336, var_392, var_326);
                        var_394 = wp::where(var_336, var_385, var_200);
                        var_395 = wp::where(var_336, var_387, var_207);
                        var_396 = wp::where(var_336, var_389, var_209);
                        if (!var_336) {
                            // elif geomtype == GeomType.TRIANGLE:                            <L 212>
                            var_398 = (var_geomtype == var_397);
                            if (var_398) {
                                // t1 = geom.rot[0, :]                                        <L 213>
                                var_399 = &((var_geom).rot);
                                var_401 = wp::slice_t(var_402, var_403, var_404);
                                var_406 = wp::load(var_399);
                                var_405 = wp::extract<3>(var_406, var_400, var_401);
                                // t2 = geom.rot[1, :]                                        <L 214>
                                var_407 = &((var_geom).rot);
                                var_409 = wp::slice_t(var_410, var_411, var_412);
                                var_414 = wp::load(var_407);
                                var_413 = wp::extract<3>(var_414, var_408, var_409);
                                // t3 = geom.rot[2, :]                                        <L 215>
                                var_415 = &((var_geom).rot);
                                var_417 = wp::slice_t(var_418, var_419, var_420);
                                var_422 = wp::load(var_415);
                                var_421 = wp::extract<3>(var_422, var_416, var_417);
                                // d1 = wp.dot(t1, dir)                                       <L 216>
                                var_423 = wp::dot(var_405, var_dir);
                                // d2 = wp.dot(t2, dir)                                       <L 217>
                                var_424 = wp::dot(var_413, var_dir);
                                // d3 = wp.dot(t3, dir)                                       <L 218>
                                var_425 = wp::dot(var_421, var_dir);
                                // if d1 > d2 and d1 > d3:                                    <L 219>
                                var_427 = (var_423 > var_424);
                                var_426 = var_427;
                                if (var_426) {
                                    var_428 = (var_423 > var_425);
                                    var_426 = var_426 && var_428;
                                }
                                if (var_426) {
                                    // sp.point = t1                                          <L 220>
                                    var_0.point = var_405;
                                }
                                if (!var_426) {
                                    // elif d2 > d3:                                          <L 221>
                                    var_429 = (var_424 > var_425);
                                    if (var_429) {
                                        // sp.point = t2                                      <L 222>
                                        var_0.point = var_413;
                                    }
                                    if (!var_429) {
                                        // sp.point = t3                                      <L 224>
                                        var_0.point = var_421;
                                    }
                                }
                            }
                            if (!var_398) {
                                // elif geomtype == GeomType.FLEX:                            <L 225>
                                var_431 = (var_geomtype == var_430);
                                if (var_431) {
                                    // p0 = geom.polyvert[0]                                  <L 226>
                                    var_432 = &((var_geom).polyvert);
                                    var_435 = wp::load(var_432);
                                    var_434 = wp::extract(var_435, var_433);
                                    // p1 = geom.polyvert[1]                                  <L 227>
                                    var_436 = &((var_geom).polyvert);
                                    var_439 = wp::load(var_436);
                                    var_438 = wp::extract(var_439, var_437);
                                    // p2 = geom.polyvert[2]                                  <L 228>
                                    var_440 = &((var_geom).polyvert);
                                    var_443 = wp::load(var_440);
                                    var_442 = wp::extract(var_443, var_441);
                                    // p3 = geom.polyvert[3]                                  <L 229>
                                    var_444 = &((var_geom).polyvert);
                                    var_447 = wp::load(var_444);
                                    var_446 = wp::extract(var_447, var_445);
                                    // d0 = wp.dot(p0, dir)                                   <L 230>
                                    var_448 = wp::dot(var_434, var_dir);
                                    // d1 = wp.dot(p1, dir)                                   <L 231>
                                    var_449 = wp::dot(var_438, var_dir);
                                    // d2 = wp.dot(p2, dir)                                   <L 232>
                                    var_450 = wp::dot(var_442, var_dir);
                                    // d3 = wp.dot(p3, dir)                                   <L 233>
                                    var_451 = wp::dot(var_446, var_dir);
                                    // if d0 > d1 and d0 > d2 and d0 > d3:                    <L 234>
                                    var_453 = (var_448 > var_449);
                                    var_452 = var_453;
                                    if (var_452) {
                                        var_454 = (var_448 > var_450);
                                        var_452 = var_452 && var_454;
                                    }
                                    if (var_452) {
                                        var_455 = (var_448 > var_451);
                                        var_452 = var_452 && var_455;
                                    }
                                    if (var_452) {
                                        // sp.point = p0                                      <L 235>
                                        var_0.point = var_434;
                                    }
                                    if (!var_452) {
                                        // elif d1 > d2 and d1 > d3:                          <L 236>
                                        var_457 = (var_449 > var_450);
                                        var_456 = var_457;
                                        if (var_456) {
                                            var_458 = (var_449 > var_451);
                                            var_456 = var_456 && var_458;
                                        }
                                        if (var_456) {
                                            // sp.point = p1                                  <L 237>
                                            var_0.point = var_438;
                                        }
                                        if (!var_456) {
                                            // elif d2 > d3:                                  <L 238>
                                            var_459 = (var_450 > var_451);
                                            if (var_459) {
                                                // sp.point = p2                              <L 239>
                                                var_0.point = var_442;
                                            }
                                            if (!var_459) {
                                                // sp.point = p3                              <L 241>
                                                var_0.point = var_446;
                                            }
                                        }
                                    }
                                }
                                var_460 = wp::where(var_431, var_449, var_423);
                                var_461 = wp::where(var_431, var_450, var_424);
                                var_462 = wp::where(var_431, var_451, var_425);
                            }
                            var_463 = wp::where(var_398, var_423, var_460);
                            var_464 = wp::where(var_398, var_424, var_461);
                            var_465 = wp::where(var_398, var_425, var_462);
                        }
                    }
                    var_466 = wp::where(var_163, var_326, var_393);
                    var_467 = wp::where(var_163, var_200, var_394);
                    var_468 = wp::where(var_163, var_207, var_395);
                    var_469 = wp::where(var_163, var_209, var_396);
                }
            }
            var_470 = wp::where(var_85, var_99, var_161);
        }
        var_471 = wp::where(var_62, var_83, var_470);
    }
    var_472 = wp::where(var_23, var_26, var_471);
    // if geom.margin > 0.0:                                                                  <L 243>
    var_473 = &((var_geom).margin);
    var_476 = wp::load(var_473);
    var_475 = (var_476 > var_474);
    if (var_475) {
        // sp.point += dir * (0.5 * geom.margin)                                              <L 244>
        var_478 = &((var_geom).margin);
        var_480 = wp::load(var_478);
        var_479 = wp::mul(var_477, var_480);
        var_481 = wp::mul(var_dir, var_479);
        var_482 = &((var_0).point);
        var_484 = wp::load(var_482);
        var_483 = wp::add(var_484, var_481);
        var_0.point = var_483;
    }
    // return sp                                                                              <L 245>
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_convex.py:61
static CUDA_CALLABLE void _hfield_filter_0(
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::vec_t<4, wp::float32>> var_hfield_size,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_worldid,
    wp::int32 var_g1,
    wp::int32 var_g2,
    bool & ret_0,
    wp::float32 & ret_1,
    wp::float32 & ret_2,
    wp::float32 & ret_3,
    wp::float32 & ret_4,
    wp::float32 & ret_5,
    wp::float32 & ret_6)
{
    //---------
    // primal vars
    wp::shape_t* var_0;
    const wp::int32 var_1 = 0;
    wp::int32 var_2;
    wp::shape_t var_3;
    wp::int32 var_4;
    wp::int32* var_5;
    wp::int32 var_6;
    wp::int32 var_7;
    wp::vec_t<4, wp::float32>* var_8;
    wp::vec_t<4, wp::float32> var_9;
    wp::vec_t<4, wp::float32> var_10;
    wp::shape_t* var_11;
    const wp::int32 var_12 = 0;
    wp::int32 var_13;
    wp::shape_t var_14;
    wp::int32 var_15;
    wp::shape_t* var_16;
    const wp::int32 var_17 = 0;
    wp::int32 var_18;
    wp::shape_t var_19;
    wp::int32 var_20;
    wp::shape_t* var_21;
    const wp::int32 var_22 = 0;
    wp::int32 var_23;
    wp::shape_t var_24;
    wp::int32 var_25;
    wp::vec_t<3, wp::float32>* var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::mat_t<3, 3, wp::float32>* var_29;
    wp::mat_t<3, 3, wp::float32> var_30;
    wp::mat_t<3, 3, wp::float32> var_31;
    wp::mat_t<3, 3, wp::float32> var_32;
    wp::vec_t<3, wp::float32>* var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::float32* var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    wp::float32* var_41;
    wp::float32* var_42;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::int32 var_46 = 0;
    bool var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    bool var_52;
    wp::float32 var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::float32 var_57;
    bool var_58;
    const bool var_59 = true;
    const wp::float32 var_60 = 10000000000.0;
    const wp::int32 var_61 = 1;
    bool var_62;
    wp::float32 var_63;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::float32 var_66;
    bool var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    bool var_73;
    const bool var_74 = true;
    const wp::int32 var_75 = 2;
    wp::float32 var_76;
    const wp::int32 var_77 = 2;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::float32 var_80;
    bool var_81;
    const bool var_82 = true;
    const wp::int32 var_83 = 3;
    wp::float32 var_84;
    wp::float32 var_85;
    const wp::int32 var_86 = 2;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    bool var_90;
    const bool var_91 = true;
    wp::mat_t<3, 3, wp::float32>* var_92;
    wp::mat_t<3, 3, wp::float32> var_93;
    wp::mat_t<3, 3, wp::float32> var_94;
    wp::mat_t<3, 3, wp::float32> var_95;
    Geom_4022bfa7 var_96;
    wp::vec_t<3, wp::float32>* var_97;
    wp::vec_t<3, wp::float32> var_98;
    wp::vec_t<3, wp::float32> var_99;
    const wp::float32 var_100 = 0.0;
    const wp::int32 var_101 = -1;
    wp::int32* var_102;
    wp::int32 var_103;
    wp::int32 var_104;
    const wp::int32 var_105 = 7;
    bool var_106;
    wp::int32* var_107;
    wp::int32 var_108;
    wp::int32 var_109;
    const wp::int32 var_110 = 0;
    bool var_111;
    wp::int32* var_112;
    const wp::int32 var_113 = -1;
    wp::int32 var_114;
    wp::int32 var_115;
    const wp::int32 var_116 = 0;
    bool var_117;
    wp::int32* var_118;
    const wp::int32 var_119 = -1;
    wp::int32 var_120;
    wp::int32 var_121;
    const wp::int32 var_122 = 0;
    bool var_123;
    wp::int32* var_124;
    const wp::int32 var_125 = -1;
    wp::int32 var_126;
    wp::int32 var_127;
    const wp::float32 var_128 = 1.0;
    const wp::float32 var_129 = 0.0;
    const wp::float32 var_130 = 0.0;
    wp::vec_t<3, wp::float32> var_131;
    SupportPoint_e82efc60 var_132;
    wp::vec_t<3, wp::float32>* var_133;
    const wp::int32 var_134 = 0;
    wp::float32 var_135;
    wp::vec_t<3, wp::float32> var_136;
    const wp::float32 var_137 = -1.0;
    const wp::float32 var_138 = 0.0;
    const wp::float32 var_139 = 0.0;
    wp::vec_t<3, wp::float32> var_140;
    SupportPoint_e82efc60 var_141;
    wp::vec_t<3, wp::float32>* var_142;
    const wp::int32 var_143 = 0;
    wp::float32 var_144;
    wp::vec_t<3, wp::float32> var_145;
    const wp::float32 var_146 = 0.0;
    const wp::float32 var_147 = 1.0;
    const wp::float32 var_148 = 0.0;
    wp::vec_t<3, wp::float32> var_149;
    SupportPoint_e82efc60 var_150;
    wp::vec_t<3, wp::float32>* var_151;
    const wp::int32 var_152 = 1;
    wp::float32 var_153;
    wp::vec_t<3, wp::float32> var_154;
    const wp::float32 var_155 = 0.0;
    const wp::float32 var_156 = -1.0;
    const wp::float32 var_157 = 0.0;
    wp::vec_t<3, wp::float32> var_158;
    SupportPoint_e82efc60 var_159;
    wp::vec_t<3, wp::float32>* var_160;
    const wp::int32 var_161 = 1;
    wp::float32 var_162;
    wp::vec_t<3, wp::float32> var_163;
    const wp::float32 var_164 = 0.0;
    const wp::float32 var_165 = 0.0;
    const wp::float32 var_166 = 1.0;
    wp::vec_t<3, wp::float32> var_167;
    SupportPoint_e82efc60 var_168;
    wp::vec_t<3, wp::float32>* var_169;
    const wp::int32 var_170 = 2;
    wp::float32 var_171;
    wp::vec_t<3, wp::float32> var_172;
    const wp::float32 var_173 = 0.0;
    const wp::float32 var_174 = 0.0;
    const wp::float32 var_175 = -1.0;
    wp::vec_t<3, wp::float32> var_176;
    SupportPoint_e82efc60 var_177;
    wp::vec_t<3, wp::float32>* var_178;
    const wp::int32 var_179 = 2;
    wp::float32 var_180;
    wp::vec_t<3, wp::float32> var_181;
    bool var_182;
    wp::float32 var_183;
    const wp::int32 var_184 = 0;
    wp::float32 var_185;
    bool var_186;
    wp::float32 var_187;
    const wp::int32 var_188 = 0;
    wp::float32 var_189;
    wp::float32 var_190;
    bool var_191;
    wp::float32 var_192;
    const wp::int32 var_193 = 1;
    wp::float32 var_194;
    bool var_195;
    wp::float32 var_196;
    const wp::int32 var_197 = 1;
    wp::float32 var_198;
    wp::float32 var_199;
    bool var_200;
    wp::float32 var_201;
    const wp::int32 var_202 = 2;
    wp::float32 var_203;
    bool var_204;
    wp::float32 var_205;
    const wp::int32 var_206 = 3;
    wp::float32 var_207;
    wp::float32 var_208;
    bool var_209;
    const bool var_210 = true;
    const bool var_211 = false;
    //---------
    // forward
    // def _hfield_filter(                                                                    <L 62>
    // dataid_setid = worldid % geom_dataid.shape[0]                                          <L 88>
    var_0 = &(var_geom_dataid.shape);
    var_3 = wp::load(var_0);
    var_2 = wp::extract(var_3, var_1);
    var_4 = wp::mod(var_worldid, var_2);
    // hfdataid = geom_dataid[dataid_setid, g1]                                               <L 89>
    var_5 = wp::address(var_geom_dataid, var_4, var_g1);
    var_7 = wp::load(var_5);
    var_6 = wp::copy(var_7);
    // size1 = hfield_size[hfdataid]                                                          <L 90>
    var_8 = wp::address(var_hfield_size, var_6);
    var_10 = wp::load(var_8);
    var_9 = wp::copy(var_10);
    // rbound_id = worldid % geom_rbound.shape[0]                                             <L 93>
    var_11 = &(var_geom_rbound.shape);
    var_14 = wp::load(var_11);
    var_13 = wp::extract(var_14, var_12);
    var_15 = wp::mod(var_worldid, var_13);
    // margin_id = worldid % geom_margin.shape[0]                                             <L 94>
    var_16 = &(var_geom_margin.shape);
    var_19 = wp::load(var_16);
    var_18 = wp::extract(var_19, var_17);
    var_20 = wp::mod(var_worldid, var_18);
    // size_id = worldid % geom_size.shape[0]                                                 <L 95>
    var_21 = &(var_geom_size.shape);
    var_24 = wp::load(var_21);
    var_23 = wp::extract(var_24, var_22);
    var_25 = wp::mod(var_worldid, var_23);
    // pos1 = geom_xpos_in[worldid, g1]                                                       <L 97>
    var_26 = wp::address(var_geom_xpos_in, var_worldid, var_g1);
    var_28 = wp::load(var_26);
    var_27 = wp::copy(var_28);
    // mat1 = geom_xmat_in[worldid, g1]                                                       <L 98>
    var_29 = wp::address(var_geom_xmat_in, var_worldid, var_g1);
    var_31 = wp::load(var_29);
    var_30 = wp::copy(var_31);
    // mat1T = wp.transpose(mat1)                                                             <L 99>
    var_32 = wp::transpose(var_30);
    // pos2 = geom_xpos_in[worldid, g2]                                                       <L 100>
    var_33 = wp::address(var_geom_xpos_in, var_worldid, var_g2);
    var_35 = wp::load(var_33);
    var_34 = wp::copy(var_35);
    // pos = mat1T @ (pos2 - pos1)                                                            <L 101>
    var_36 = wp::sub(var_34, var_27);
    var_37 = wp::mul(var_32, var_36);
    // r2 = geom_rbound[rbound_id, g2]                                                        <L 102>
    var_38 = wp::address(var_geom_rbound, var_15, var_g2);
    var_40 = wp::load(var_38);
    var_39 = wp::copy(var_40);
    // margin = geom_margin[margin_id, g1] + geom_margin[margin_id, g2]                       <L 105>
    var_41 = wp::address(var_geom_margin, var_20, var_g1);
    var_42 = wp::address(var_geom_margin, var_20, var_g2);
    var_44 = wp::load(var_41);
    var_45 = wp::load(var_42);
    var_43 = wp::add(var_44, var_45);
    // for i in range(2):                                                                     <L 108>
    // if (size1[i] < pos[i] - r2 - margin) or (-size1[i] > pos[i] + r2 + margin):            <L 109>
    var_48 = wp::extract(var_9, var_46);
    var_49 = wp::extract(var_37, var_46);
    var_50 = wp::sub(var_49, var_39);
    var_51 = wp::sub(var_50, var_43);
    var_52 = (var_48 < var_51);
    var_47 = var_52;
    if (!var_47) {
        var_53 = wp::extract(var_9, var_46);
        var_54 = wp::neg(var_53);
        var_55 = wp::extract(var_37, var_46);
        var_56 = wp::add(var_55, var_39);
        var_57 = wp::add(var_56, var_43);
        var_58 = (var_54 > var_57);
        var_47 = var_47 || var_58;
    }
    if (var_47) {
        // return True, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL       <L 110>
        ret_0 = var_59;
        ret_1 = var_60;
        ret_2 = var_60;
        ret_3 = var_60;
        ret_4 = var_60;
        ret_5 = var_60;
        ret_6 = var_60;
        return;
    }
    // if (size1[i] < pos[i] - r2 - margin) or (-size1[i] > pos[i] + r2 + margin):            <L 109>
    var_63 = wp::extract(var_9, var_61);
    var_64 = wp::extract(var_37, var_61);
    var_65 = wp::sub(var_64, var_39);
    var_66 = wp::sub(var_65, var_43);
    var_67 = (var_63 < var_66);
    var_62 = var_67;
    if (!var_62) {
        var_68 = wp::extract(var_9, var_61);
        var_69 = wp::neg(var_68);
        var_70 = wp::extract(var_37, var_61);
        var_71 = wp::add(var_70, var_39);
        var_72 = wp::add(var_71, var_43);
        var_73 = (var_69 > var_72);
        var_62 = var_62 || var_73;
    }
    if (var_62) {
        // return True, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL       <L 110>
        ret_0 = var_74;
        ret_1 = var_60;
        ret_2 = var_60;
        ret_3 = var_60;
        ret_4 = var_60;
        ret_5 = var_60;
        ret_6 = var_60;
        return;
    }
    // if size1[2] < pos[2] - r2 - margin:  # up                                              <L 113>
    var_76 = wp::extract(var_9, var_75);
    var_78 = wp::extract(var_37, var_77);
    var_79 = wp::sub(var_78, var_39);
    var_80 = wp::sub(var_79, var_43);
    var_81 = (var_76 < var_80);
    if (var_81) {
        // return True, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL       <L 114>
        ret_0 = var_82;
        ret_1 = var_60;
        ret_2 = var_60;
        ret_3 = var_60;
        ret_4 = var_60;
        ret_5 = var_60;
        ret_6 = var_60;
        return;
    }
    // if -size1[3] > pos[2] + r2 + margin:  # down                                           <L 116>
    var_84 = wp::extract(var_9, var_83);
    var_85 = wp::neg(var_84);
    var_87 = wp::extract(var_37, var_86);
    var_88 = wp::add(var_87, var_39);
    var_89 = wp::add(var_88, var_43);
    var_90 = (var_85 > var_89);
    if (var_90) {
        // return True, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL       <L 117>
        ret_0 = var_91;
        ret_1 = var_60;
        ret_2 = var_60;
        ret_3 = var_60;
        ret_4 = var_60;
        ret_5 = var_60;
        ret_6 = var_60;
        return;
    }
    // mat2 = geom_xmat_in[worldid, g2]                                                       <L 119>
    var_92 = wp::address(var_geom_xmat_in, var_worldid, var_g2);
    var_94 = wp::load(var_92);
    var_93 = wp::copy(var_94);
    // mat = mat1T @ mat2                                                                     <L 120>
    var_95 = wp::mul(var_32, var_93);
    // geom2 = Geom()                                                                         <L 123>
    var_96 = Geom_4022bfa7();
    // geom2.pos = pos                                                                        <L 124>
    var_96.pos = var_37;
    // geom2.rot = mat                                                                        <L 125>
    var_96.rot = var_95;
    // geom2.size = geom_size[size_id, g2]                                                    <L 126>
    var_97 = wp::address(var_geom_size, var_25, var_g2);
    var_99 = wp::load(var_97);
    var_98 = wp::copy(var_99);
    var_96.size = var_98;
    // geom2.margin = 0.0  # margin handled separately                                        <L 127>
    var_96.margin = var_100;
    // geom2.index = -1                                                                       <L 128>
    var_96.index = var_101;
    // geomtype2 = geom_type[g2]                                                              <L 130>
    var_102 = wp::address(var_geom_type, var_g2);
    var_104 = wp::load(var_102);
    var_103 = wp::copy(var_104);
    // if geomtype2 == GeomType.MESH:                                                         <L 133>
    var_106 = (var_103 == var_105);
    if (var_106) {
        // dataid = geom_dataid[dataid_setid, g2]                                             <L 134>
        var_107 = wp::address(var_geom_dataid, var_4, var_g2);
        var_109 = wp::load(var_107);
        var_108 = wp::copy(var_109);
        // geom2.vertadr = wp.where(dataid >= 0, mesh_vertadr[dataid], -1)                    <L 135>
        var_111 = (var_108 >= var_110);
        var_112 = wp::address(var_mesh_vertadr, var_108);
        var_115 = wp::load(var_112);
        var_114 = wp::where(var_111, var_115, var_113);
        var_96.vertadr = var_114;
        // geom2.vertnum = wp.where(dataid >= 0, mesh_vertnum[dataid], -1)                    <L 136>
        var_117 = (var_108 >= var_116);
        var_118 = wp::address(var_mesh_vertnum, var_108);
        var_121 = wp::load(var_118);
        var_120 = wp::where(var_117, var_121, var_119);
        var_96.vertnum = var_120;
        // geom2.graphadr = wp.where(dataid >= 0, mesh_graphadr[dataid], -1)                  <L 137>
        var_123 = (var_108 >= var_122);
        var_124 = wp::address(var_mesh_graphadr, var_108);
        var_127 = wp::load(var_124);
        var_126 = wp::where(var_123, var_127, var_125);
        var_96.graphadr = var_126;
        // geom2.vert = mesh_vert                                                             <L 138>
        var_96.vert = var_mesh_vert;
        // geom2.graph = mesh_graph                                                           <L 139>
        var_96.graph = var_mesh_graph;
    }
    // xmax = support(geom2, geomtype2, wp.vec3(1.0, 0.0, 0.0)).point[0]                      <L 142>
    var_131 = wp::vec_t<3, wp::float32>(var_128, var_129, var_130);
    var_132 = support_0(var_96, var_103, var_131);
    var_133 = &(var_132.point);
    var_136 = wp::load(var_133);
    var_135 = wp::extract(var_136, var_134);
    // xmin = support(geom2, geomtype2, wp.vec3(-1.0, 0.0, 0.0)).point[0]                     <L 143>
    var_140 = wp::vec_t<3, wp::float32>(var_137, var_138, var_139);
    var_141 = support_0(var_96, var_103, var_140);
    var_142 = &(var_141.point);
    var_145 = wp::load(var_142);
    var_144 = wp::extract(var_145, var_143);
    // ymax = support(geom2, geomtype2, wp.vec3(0.0, 1.0, 0.0)).point[1]                      <L 144>
    var_149 = wp::vec_t<3, wp::float32>(var_146, var_147, var_148);
    var_150 = support_0(var_96, var_103, var_149);
    var_151 = &(var_150.point);
    var_154 = wp::load(var_151);
    var_153 = wp::extract(var_154, var_152);
    // ymin = support(geom2, geomtype2, wp.vec3(0.0, -1.0, 0.0)).point[1]                     <L 145>
    var_158 = wp::vec_t<3, wp::float32>(var_155, var_156, var_157);
    var_159 = support_0(var_96, var_103, var_158);
    var_160 = &(var_159.point);
    var_163 = wp::load(var_160);
    var_162 = wp::extract(var_163, var_161);
    // zmax = support(geom2, geomtype2, wp.vec3(0.0, 0.0, 1.0)).point[2]                      <L 146>
    var_167 = wp::vec_t<3, wp::float32>(var_164, var_165, var_166);
    var_168 = support_0(var_96, var_103, var_167);
    var_169 = &(var_168.point);
    var_172 = wp::load(var_169);
    var_171 = wp::extract(var_172, var_170);
    // zmin = support(geom2, geomtype2, wp.vec3(0.0, 0.0, -1.0)).point[2]                     <L 147>
    var_176 = wp::vec_t<3, wp::float32>(var_173, var_174, var_175);
    var_177 = support_0(var_96, var_103, var_176);
    var_178 = &(var_177.point);
    var_181 = wp::load(var_178);
    var_180 = wp::extract(var_181, var_179);
    // if (                                                                                   <L 150>
    // (xmin - margin > size1[0])                                                             <L 151>
    var_183 = wp::sub(var_144, var_43);
    var_185 = wp::extract(var_9, var_184);
    var_186 = (var_183 > var_185);
    var_182 = var_186;
    if (!var_182) {
        // or (xmax + margin < -size1[0])                                                     <L 152>
        var_187 = wp::add(var_135, var_43);
        var_189 = wp::extract(var_9, var_188);
        var_190 = wp::neg(var_189);
        var_191 = (var_187 < var_190);
        var_182 = var_182 || var_191;
    }
    if (!var_182) {
        // or (ymin - margin > size1[1])                                                      <L 153>
        var_192 = wp::sub(var_162, var_43);
        var_194 = wp::extract(var_9, var_193);
        var_195 = (var_192 > var_194);
        var_182 = var_182 || var_195;
    }
    if (!var_182) {
        // or (ymax + margin < -size1[1])                                                     <L 154>
        var_196 = wp::add(var_153, var_43);
        var_198 = wp::extract(var_9, var_197);
        var_199 = wp::neg(var_198);
        var_200 = (var_196 < var_199);
        var_182 = var_182 || var_200;
    }
    if (!var_182) {
        // or (zmin - margin > size1[2])                                                      <L 155>
        var_201 = wp::sub(var_180, var_43);
        var_203 = wp::extract(var_9, var_202);
        var_204 = (var_201 > var_203);
        var_182 = var_182 || var_204;
    }
    if (!var_182) {
        // or (zmax + margin < -size1[3])                                                     <L 156>
        var_205 = wp::add(var_171, var_43);
        var_207 = wp::extract(var_9, var_206);
        var_208 = wp::neg(var_207);
        var_209 = (var_205 < var_208);
        var_182 = var_182 || var_209;
    }
    if (var_182) {
        // return True, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL       <L 158>
        ret_0 = var_210;
        ret_1 = var_60;
        ret_2 = var_60;
        ret_3 = var_60;
        ret_4 = var_60;
        ret_5 = var_60;
        ret_6 = var_60;
        return;
    }
    if (!var_182) {
        // return False, xmin, xmax, ymin, ymax, zmin, zmax                                   <L 160>
        ret_0 = var_211;
        ret_1 = var_144;
        ret_2 = var_135;
        ret_3 = var_162;
        ret_4 = var_153;
        ret_5 = var_180;
        ret_6 = var_171;
        return;
    }
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:112
static CUDA_CALLABLE bool _discrete_geoms_0(
    wp::int32 var_g1,
    wp::int32 var_g2)
{
    //---------
    // primal vars
    bool var_0;
    bool var_1;
    const wp::int32 var_2 = 7;
    bool var_3;
    const wp::int32 var_4 = 6;
    bool var_5;
    const wp::int32 var_6 = 1;
    bool var_7;
    bool var_8;
    const wp::int32 var_9 = 7;
    bool var_10;
    const wp::int32 var_11 = 6;
    bool var_12;
    const wp::int32 var_13 = 1;
    bool var_14;
    //---------
    // forward
    // def _discrete_geoms(g1: int, g2: int) -> bool:                                         <L 113>
    // return (g1 == GeomType.MESH or g1 == GeomType.BOX or g1 == GeomType.HFIELD) and (       <L 114>
    var_3 = (var_g1 == var_2);
    var_1 = var_3;
    if (!var_1) {
        var_5 = (var_g1 == var_4);
        var_1 = var_1 || var_5;
    }
    if (!var_1) {
        var_7 = (var_g1 == var_6);
        var_1 = var_1 || var_7;
    }
    var_0 = var_1;
    if (var_0) {
        // g2 == GeomType.MESH or g2 == GeomType.BOX or g2 == GeomType.HFIELD                 <L 115>
        var_10 = (var_g2 == var_9);
        var_8 = var_10;
        if (!var_8) {
            var_12 = (var_g2 == var_11);
            var_8 = var_8 || var_12;
        }
        if (!var_8) {
            var_14 = (var_g2 == var_13);
            var_8 = var_8 || var_14;
        }
        var_0 = var_0 && var_8;
    }
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:617
static CUDA_CALLABLE void _gjk_support_0(
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_k,
    wp::float32 var_x_norm,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::int32 var_n,
    bool var_is_discrete,
    SupportPoint_e82efc60 & ret_0,
    SupportPoint_e82efc60 & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    bool var_1;
    const wp::float32 var_2 = 0.0001;
    bool var_3;
    const wp::int32 var_4 = 2;
    bool var_5;
    const wp::int32 var_6 = 1;
    wp::vec_t<3, wp::float32> var_7;
    const wp::int32 var_8 = 0;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::float32 var_11;
    const wp::float32 var_12 = 1e-30;
    bool var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::vec_t<3, wp::float32> var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::float32 var_18;
    const wp::float32 var_19 = 1e-15;
    bool var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::vec_t<3, wp::float32> var_24;
    const wp::int32 var_25 = 3;
    bool var_26;
    const wp::int32 var_27 = 1;
    wp::vec_t<3, wp::float32> var_28;
    const wp::int32 var_29 = 0;
    wp::vec_t<3, wp::float32> var_30;
    wp::vec_t<3, wp::float32> var_31;
    const wp::int32 var_32 = 2;
    wp::vec_t<3, wp::float32> var_33;
    const wp::int32 var_34 = 0;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::float32 var_38;
    bool var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::vec_t<3, wp::float32> var_42;
    wp::vec_t<3, wp::float32> var_43;
    wp::vec_t<3, wp::float32> var_44;
    wp::vec_t<3, wp::float32> var_45;
    wp::vec_t<3, wp::float32> var_46;
    wp::vec_t<3, wp::float32> var_47;
    wp::vec_t<3, wp::float32> var_48;
    SupportPoint_e82efc60 var_49;
    SupportPoint_e82efc60 var_50;
    //---------
    // forward
    // def _gjk_support(                                                                      <L 618>
    // dir_neg = x_k / x_norm                                                                 <L 630>
    var_0 = wp::div(var_x_k, var_x_norm);
    // if is_discrete and x_norm < 1e-4:                                                      <L 633>
    var_1 = var_is_discrete;
    if (var_1) {
        var_3 = (var_x_norm < var_2);
        var_1 = var_1 && var_3;
    }
    if (var_1) {
        // if n == 2:                                                                         <L 634>
        var_5 = (var_n == var_4);
        if (var_5) {
            // edge = simplex[1] - simplex[0]                                                 <L 635>
            var_7 = wp::extract(var_simplex, var_6);
            var_9 = wp::extract(var_simplex, var_8);
            var_10 = wp::sub(var_7, var_9);
            // edge_norm2 = wp.dot(edge, edge)                                                <L 636>
            var_11 = wp::dot(var_10, var_10);
            // if edge_norm2 > MINVAL2:                                                       <L 637>
            var_13 = (var_11 > var_12);
            if (var_13) {
                // proj = wp.dot(dir_neg, edge) / edge_norm2                                  <L 638>
                var_14 = wp::dot(var_0, var_10);
                var_15 = wp::div(var_14, var_11);
                // dir_neg = dir_neg - proj * edge                                            <L 639>
                var_16 = wp::mul(var_15, var_10);
                var_17 = wp::sub(var_0, var_16);
                // dir_norm = wp.length(dir_neg)                                              <L 640>
                var_18 = wp::length(var_17);
                // if dir_norm > MINVAL:                                                      <L 641>
                var_20 = (var_18 > var_19);
                if (var_20) {
                    // dir_neg = dir_neg / dir_norm                                           <L 642>
                    var_21 = wp::div(var_17, var_18);
                }
                var_22 = wp::where(var_20, var_21, var_17);
            }
            var_23 = wp::where(var_13, var_22, var_0);
        }
        var_24 = wp::where(var_5, var_23, var_0);
        if (!var_5) {
            // elif n == 3:                                                                   <L 643>
            var_26 = (var_n == var_25);
            if (var_26) {
                // e1 = simplex[1] - simplex[0]                                               <L 644>
                var_28 = wp::extract(var_simplex, var_27);
                var_30 = wp::extract(var_simplex, var_29);
                var_31 = wp::sub(var_28, var_30);
                // e2 = simplex[2] - simplex[0]                                               <L 645>
                var_33 = wp::extract(var_simplex, var_32);
                var_35 = wp::extract(var_simplex, var_34);
                var_36 = wp::sub(var_33, var_35);
                // normal = wp.cross(e1, e2)                                                  <L 646>
                var_37 = wp::cross(var_31, var_36);
                // normal_norm = wp.length(normal)                                            <L 647>
                var_38 = wp::length(var_37);
                // if normal_norm > MINVAL:                                                   <L 648>
                var_39 = (var_38 > var_19);
                if (var_39) {
                    // dir_neg = wp.sign(wp.dot(dir_neg, normal)) * normal / normal_norm       <L 649>
                    var_40 = wp::dot(var_24, var_37);
                    var_41 = wp::sign(var_40);
                    var_42 = wp::mul(var_41, var_37);
                    var_43 = wp::div(var_42, var_38);
                }
                var_44 = wp::where(var_39, var_43, var_24);
            }
            var_45 = wp::where(var_26, var_44, var_24);
        }
        var_46 = wp::where(var_5, var_24, var_45);
    }
    var_47 = wp::where(var_1, var_46, var_0);
    // sp1 = support(geom1, geomtype1, -dir_neg)                                              <L 651>
    var_48 = wp::neg(var_47);
    var_49 = support_0(var_geom1, var_geomtype1, var_48);
    // sp2 = support(geom2, geomtype2, dir_neg)                                               <L 652>
    var_50 = support_0(var_geom2, var_geomtype2, var_47);
    // return sp1, sp2                                                                        <L 653>
    ret_0 = var_49;
    ret_1 = var_50;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:315
static CUDA_CALLABLE wp::float32 _det3_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::float32 var_1;
    //---------
    // forward
    // def _det3(v1: wp.vec3, v2: wp.vec3, v3: wp.vec3) -> float:                             <L 316>
    // return wp.dot(v1, wp.cross(v2, v3))                                                    <L 317>
    var_0 = wp::cross(var_v2, var_v3);
    var_1 = wp::dot(var_v1, var_0);
    return var_1;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:320
static CUDA_CALLABLE wp::int32 _same_sign_0(
    wp::float32 var_a,
    wp::float32 var_b)
{
    //---------
    // primal vars
    bool var_0;
    const wp::float32 var_1 = 0.0;
    bool var_2;
    const wp::float32 var_3 = 0.0;
    bool var_4;
    const wp::int32 var_5 = 1;
    bool var_6;
    const wp::float32 var_7 = 0.0;
    bool var_8;
    const wp::float32 var_9 = 0.0;
    bool var_10;
    const wp::int32 var_11 = -1;
    const wp::int32 var_12 = 0;
    //---------
    // forward
    // def _same_sign(a: float, b: float) -> int:                                             <L 321>
    // if a > 0.0 and b > 0.0:                                                                <L 322>
    var_2 = (var_a > var_1);
    var_0 = var_2;
    if (var_0) {
        var_4 = (var_b > var_3);
        var_0 = var_0 && var_4;
    }
    if (var_0) {
        // return 1                                                                           <L 323>
        return var_5;
    }
    // if a < 0.0 and b < 0.0:                                                                <L 324>
    var_8 = (var_a < var_7);
    var_6 = var_8;
    if (var_6) {
        var_10 = (var_b < var_9);
        var_6 = var_6 && var_10;
    }
    if (var_6) {
        // return -1                                                                          <L 325>
        return var_11;
    }
    // return 0                                                                               <L 326>
    return var_12;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:336
static CUDA_CALLABLE void _project_origin_plane_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::int32 & ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    const wp::float32 var_8 = 0.0;
    bool var_9;
    const wp::int32 var_10 = 1;
    bool var_11;
    const wp::float32 var_12 = 0.0;
    bool var_13;
    const wp::float32 var_14 = 1e-15;
    bool var_15;
    wp::float32 var_16;
    wp::vec_t<3, wp::float32> var_17;
    const wp::int32 var_18 = 0;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::float32 var_22 = 0.0;
    bool var_23;
    const wp::int32 var_24 = 1;
    bool var_25;
    const wp::float32 var_26 = 0.0;
    bool var_27;
    bool var_28;
    wp::float32 var_29;
    wp::vec_t<3, wp::float32> var_30;
    const wp::int32 var_31 = 0;
    wp::vec_t<3, wp::float32> var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::vec_t<3, wp::float32> var_36;
    const wp::int32 var_37 = 0;
    //---------
    // forward
    // def _project_origin_plane(v1: wp.vec3, v2: wp.vec3, v3: wp.vec3) -> Tuple[wp.vec3, int]:       <L 337>
    // z = wp.vec3(0.0)                                                                       <L 338>
    var_1 = wp::vec_t<3, wp::float32>(var_0);
    // diff21 = v2 - v1                                                                       <L 339>
    var_2 = wp::sub(var_v2, var_v1);
    // diff31 = v3 - v1                                                                       <L 340>
    var_3 = wp::sub(var_v3, var_v1);
    // diff32 = v3 - v2                                                                       <L 341>
    var_4 = wp::sub(var_v3, var_v2);
    // n = wp.cross(diff32, diff21)                                                           <L 344>
    var_5 = wp::cross(var_4, var_2);
    // nv = wp.dot(n, v2)                                                                     <L 345>
    var_6 = wp::dot(var_5, var_v2);
    // nn = wp.dot(n, n)                                                                      <L 346>
    var_7 = wp::dot(var_5, var_5);
    // if nn == 0.0:                                                                          <L 347>
    var_9 = (var_7 == var_8);
    if (var_9) {
        // return z, 1                                                                        <L 348>
        ret_0 = var_1;
        ret_1 = var_10;
        return;
    }
    // if nv != 0.0 and nn > MINVAL:                                                          <L 349>
    var_13 = (var_6 != var_12);
    var_11 = var_13;
    if (var_11) {
        var_15 = (var_7 > var_14);
        var_11 = var_11 && var_15;
    }
    if (var_11) {
        // return (nv / nn) * n, 0                                                            <L 350>
        var_16 = wp::div(var_6, var_7);
        var_17 = wp::mul(var_16, var_5);
        ret_0 = var_17;
        ret_1 = var_18;
        return;
    }
    // n = wp.cross(diff21, diff31)                                                           <L 353>
    var_19 = wp::cross(var_2, var_3);
    // nv = wp.dot(n, v1)                                                                     <L 354>
    var_20 = wp::dot(var_19, var_v1);
    // nn = wp.dot(n, n)                                                                      <L 355>
    var_21 = wp::dot(var_19, var_19);
    // if nn == 0.0:                                                                          <L 356>
    var_23 = (var_21 == var_22);
    if (var_23) {
        // return z, 1                                                                        <L 357>
        ret_0 = var_1;
        ret_1 = var_24;
        return;
    }
    // if nv != 0.0 and nn > MINVAL:                                                          <L 358>
    var_27 = (var_20 != var_26);
    var_25 = var_27;
    if (var_25) {
        var_28 = (var_21 > var_14);
        var_25 = var_25 && var_28;
    }
    if (var_25) {
        // return (nv / nn) * n, 0                                                            <L 359>
        var_29 = wp::div(var_20, var_21);
        var_30 = wp::mul(var_29, var_19);
        ret_0 = var_30;
        ret_1 = var_31;
        return;
    }
    // n = wp.cross(diff31, diff32)                                                           <L 362>
    var_32 = wp::cross(var_3, var_4);
    // nv = wp.dot(n, v3)                                                                     <L 363>
    var_33 = wp::dot(var_32, var_v3);
    // nn = wp.dot(n, n)                                                                      <L 364>
    var_34 = wp::dot(var_32, var_32);
    // return (nv / nn) * n, 0                                                                <L 365>
    var_35 = wp::div(var_33, var_34);
    var_36 = wp::mul(var_35, var_32);
    ret_0 = var_36;
    ret_1 = var_37;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:329
static CUDA_CALLABLE wp::vec_t<3, wp::float32> _project_origin_line_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::vec_t<3, wp::float32> var_6;
    //---------
    // forward
    // def _project_origin_line(v1: wp.vec3, v2: wp.vec3) -> wp.vec3:                         <L 330>
    // diff = v2 - v1                                                                         <L 331>
    var_0 = wp::sub(var_v2, var_v1);
    // scl = -(wp.dot(v2, diff) / wp.dot(diff, diff))                                         <L 332>
    var_1 = wp::dot(var_v2, var_0);
    var_2 = wp::dot(var_0, var_0);
    var_3 = wp::div(var_1, var_2);
    var_4 = wp::neg(var_3);
    // return v2 + scl * diff                                                                 <L 333>
    var_5 = wp::mul(var_4, var_0);
    var_6 = wp::add(var_v2, var_5);
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:589
static CUDA_CALLABLE wp::vec_t<2, wp::float32> _S1D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 0;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    bool var_14;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::float32 var_17;
    wp::int32 var_18;
    const wp::int32 var_19 = 2;
    wp::float32 var_20;
    const wp::int32 var_21 = 2;
    wp::float32 var_22;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    bool var_26;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    wp::float32 var_29;
    wp::int32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    wp::float32 var_35;
    wp::float32 var_36;
    bool var_37;
    wp::int32 var_38;
    wp::int32 var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::vec_t<2, wp::float32> var_42;
    const wp::float32 var_43 = 0.0;
    const wp::float32 var_44 = 1.0;
    wp::vec_t<2, wp::float32> var_45;
    //---------
    // forward
    // def _S1D(s1: wp.vec3, s2: wp.vec3) -> wp.vec2:                                         <L 590>
    // p_o = _project_origin_line(s1, s2)                                                     <L 592>
    var_0 = _project_origin_line_0(var_s1, var_s2);
    // mu_max = s1[0] - s2[0]                                                                 <L 595>
    var_2 = wp::extract(var_s1, var_1);
    var_4 = wp::extract(var_s2, var_3);
    var_5 = wp::sub(var_2, var_4);
    // index = 0                                                                              <L 596>
    // mu = s1[1] - s2[1]                                                                     <L 598>
    var_8 = wp::extract(var_s1, var_7);
    var_10 = wp::extract(var_s2, var_9);
    var_11 = wp::sub(var_8, var_10);
    // if wp.abs(mu) >= wp.abs(mu_max):                                                       <L 599>
    var_12 = wp::abs(var_11);
    var_13 = wp::abs(var_5);
    var_14 = (var_12 >= var_13);
    if (var_14) {
        // mu_max = mu                                                                        <L 600>
        var_15 = wp::copy(var_11);
        // index = 1                                                                          <L 601>
    }
    var_17 = wp::where(var_14, var_15, var_5);
    var_18 = wp::where(var_14, var_16, var_6);
    // mu = s1[2] - s2[2]                                                                     <L 603>
    var_20 = wp::extract(var_s1, var_19);
    var_22 = wp::extract(var_s2, var_21);
    var_23 = wp::sub(var_20, var_22);
    // if wp.abs(mu) >= wp.abs(mu_max):                                                       <L 604>
    var_24 = wp::abs(var_23);
    var_25 = wp::abs(var_17);
    var_26 = (var_24 >= var_25);
    if (var_26) {
        // mu_max = mu                                                                        <L 605>
        var_27 = wp::copy(var_23);
        // index = 2                                                                          <L 606>
    }
    var_29 = wp::where(var_26, var_27, var_17);
    var_30 = wp::where(var_26, var_28, var_18);
    // C1 = p_o[index] - s2[index]                                                            <L 608>
    var_31 = wp::extract(var_0, var_30);
    var_32 = wp::extract(var_s2, var_30);
    var_33 = wp::sub(var_31, var_32);
    // C2 = s1[index] - p_o[index]                                                            <L 609>
    var_34 = wp::extract(var_s1, var_30);
    var_35 = wp::extract(var_0, var_30);
    var_36 = wp::sub(var_34, var_35);
    // if _same_sign(mu_max, C1) and _same_sign(mu_max, C2):                                  <L 612>
    var_38 = _same_sign_0(var_29, var_33);
    var_37 = var_38;
    if (var_37) {
        var_39 = _same_sign_0(var_29, var_36);
        var_37 = var_37 && var_39;
    }
    if (var_37) {
        // return wp.vec2(C1 / mu_max, C2 / mu_max)                                           <L 613>
        var_40 = wp::div(var_33, var_29);
        var_41 = wp::div(var_36, var_29);
        var_42 = wp::vec_t<2, wp::float32>(var_40, var_41);
        return var_42;
    }
    // return wp.vec2(0.0, 1.0)                                                               <L 614>
    var_45 = wp::vec_t<2, wp::float32>(var_43, var_44);
    return var_45;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:444
static CUDA_CALLABLE wp::vec_t<3, wp::float32> _S2D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2,
    wp::vec_t<3, wp::float32> var_s3)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::int32 var_1;
    wp::vec_t<2, wp::float32> var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 1;
    wp::float32 var_6;
    const wp::float32 var_7 = 0.0;
    wp::vec_t<3, wp::float32> var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::float32 var_13;
    const wp::int32 var_14 = 2;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::float32 var_17;
    wp::float32 var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 1;
    wp::float32 var_21;
    const wp::int32 var_22 = 2;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::int32 var_26 = 2;
    wp::float32 var_27;
    const wp::int32 var_28 = 1;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    const wp::int32 var_32 = 1;
    wp::float32 var_33;
    const wp::int32 var_34 = 2;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::float32 var_37;
    const wp::int32 var_38 = 2;
    wp::float32 var_39;
    const wp::int32 var_40 = 1;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    const wp::int32 var_44 = 0;
    wp::float32 var_45;
    const wp::int32 var_46 = 2;
    wp::float32 var_47;
    wp::float32 var_48;
    const wp::int32 var_49 = 2;
    wp::float32 var_50;
    const wp::int32 var_51 = 0;
    wp::float32 var_52;
    wp::float32 var_53;
    wp::float32 var_54;
    const wp::int32 var_55 = 0;
    wp::float32 var_56;
    const wp::int32 var_57 = 2;
    wp::float32 var_58;
    wp::float32 var_59;
    wp::float32 var_60;
    const wp::int32 var_61 = 2;
    wp::float32 var_62;
    const wp::int32 var_63 = 0;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::float32 var_66;
    const wp::int32 var_67 = 0;
    wp::float32 var_68;
    const wp::int32 var_69 = 2;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::int32 var_73 = 2;
    wp::float32 var_74;
    const wp::int32 var_75 = 0;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::int32 var_79 = 0;
    wp::float32 var_80;
    const wp::int32 var_81 = 1;
    wp::float32 var_82;
    wp::float32 var_83;
    const wp::int32 var_84 = 1;
    wp::float32 var_85;
    const wp::int32 var_86 = 0;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    const wp::int32 var_90 = 0;
    wp::float32 var_91;
    const wp::int32 var_92 = 1;
    wp::float32 var_93;
    wp::float32 var_94;
    wp::float32 var_95;
    const wp::int32 var_96 = 1;
    wp::float32 var_97;
    const wp::int32 var_98 = 0;
    wp::float32 var_99;
    wp::float32 var_100;
    wp::float32 var_101;
    const wp::int32 var_102 = 0;
    wp::float32 var_103;
    const wp::int32 var_104 = 1;
    wp::float32 var_105;
    wp::float32 var_106;
    wp::float32 var_107;
    const wp::int32 var_108 = 1;
    wp::float32 var_109;
    const wp::int32 var_110 = 0;
    wp::float32 var_111;
    wp::float32 var_112;
    wp::float32 var_113;
    const wp::float32 var_114 = 0.0;
    const wp::float32 var_115 = 0.0;
    wp::vec_t<2, wp::float32> var_116;
    const wp::float32 var_117 = 0.0;
    wp::vec_t<2, wp::float32> var_118;
    const wp::float32 var_119 = 0.0;
    wp::vec_t<2, wp::float32> var_120;
    const wp::float32 var_121 = 0.0;
    wp::vec_t<2, wp::float32> var_122;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    bool var_126;
    bool var_127;
    bool var_128;
    wp::float32 var_129;
    const wp::int32 var_130 = 1;
    wp::float32 var_131;
    const wp::int32 var_132 = 0;
    const wp::int32 var_133 = 2;
    wp::float32 var_134;
    const wp::int32 var_135 = 1;
    const wp::int32 var_136 = 1;
    wp::float32 var_137;
    const wp::int32 var_138 = 0;
    const wp::int32 var_139 = 2;
    wp::float32 var_140;
    const wp::int32 var_141 = 1;
    const wp::int32 var_142 = 1;
    wp::float32 var_143;
    const wp::int32 var_144 = 0;
    const wp::int32 var_145 = 2;
    wp::float32 var_146;
    const wp::int32 var_147 = 1;
    const wp::int32 var_148 = 1;
    wp::float32 var_149;
    const wp::int32 var_150 = 0;
    const wp::int32 var_151 = 2;
    wp::float32 var_152;
    const wp::int32 var_153 = 1;
    wp::float32 var_154;
    bool var_155;
    wp::float32 var_156;
    const wp::int32 var_157 = 0;
    wp::float32 var_158;
    const wp::int32 var_159 = 0;
    const wp::int32 var_160 = 2;
    wp::float32 var_161;
    const wp::int32 var_162 = 1;
    const wp::int32 var_163 = 0;
    wp::float32 var_164;
    const wp::int32 var_165 = 0;
    const wp::int32 var_166 = 2;
    wp::float32 var_167;
    const wp::int32 var_168 = 1;
    const wp::int32 var_169 = 0;
    wp::float32 var_170;
    const wp::int32 var_171 = 0;
    const wp::int32 var_172 = 2;
    wp::float32 var_173;
    const wp::int32 var_174 = 1;
    const wp::int32 var_175 = 0;
    wp::float32 var_176;
    const wp::int32 var_177 = 0;
    const wp::int32 var_178 = 2;
    wp::float32 var_179;
    const wp::int32 var_180 = 1;
    wp::float32 var_181;
    wp::float32 var_182;
    const wp::int32 var_183 = 0;
    wp::float32 var_184;
    const wp::int32 var_185 = 0;
    const wp::int32 var_186 = 1;
    wp::float32 var_187;
    const wp::int32 var_188 = 1;
    const wp::int32 var_189 = 0;
    wp::float32 var_190;
    const wp::int32 var_191 = 0;
    const wp::int32 var_192 = 1;
    wp::float32 var_193;
    const wp::int32 var_194 = 1;
    const wp::int32 var_195 = 0;
    wp::float32 var_196;
    const wp::int32 var_197 = 0;
    const wp::int32 var_198 = 1;
    wp::float32 var_199;
    const wp::int32 var_200 = 1;
    const wp::int32 var_201 = 0;
    wp::float32 var_202;
    const wp::int32 var_203 = 0;
    const wp::int32 var_204 = 1;
    wp::float32 var_205;
    const wp::int32 var_206 = 1;
    wp::float32 var_207;
    wp::float32 var_208;
    const wp::int32 var_209 = 0;
    wp::float32 var_210;
    const wp::int32 var_211 = 1;
    wp::float32 var_212;
    wp::float32 var_213;
    const wp::int32 var_214 = 1;
    wp::float32 var_215;
    const wp::int32 var_216 = 0;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    const wp::int32 var_220 = 0;
    wp::float32 var_221;
    const wp::int32 var_222 = 1;
    wp::float32 var_223;
    wp::float32 var_224;
    wp::float32 var_225;
    const wp::int32 var_226 = 0;
    wp::float32 var_227;
    const wp::int32 var_228 = 1;
    wp::float32 var_229;
    wp::float32 var_230;
    wp::float32 var_231;
    const wp::int32 var_232 = 1;
    wp::float32 var_233;
    const wp::int32 var_234 = 0;
    wp::float32 var_235;
    wp::float32 var_236;
    wp::float32 var_237;
    const wp::int32 var_238 = 0;
    wp::float32 var_239;
    const wp::int32 var_240 = 1;
    wp::float32 var_241;
    wp::float32 var_242;
    wp::float32 var_243;
    const wp::int32 var_244 = 0;
    wp::float32 var_245;
    const wp::int32 var_246 = 1;
    wp::float32 var_247;
    wp::float32 var_248;
    const wp::int32 var_249 = 1;
    wp::float32 var_250;
    const wp::int32 var_251 = 0;
    wp::float32 var_252;
    wp::float32 var_253;
    wp::float32 var_254;
    const wp::int32 var_255 = 0;
    wp::float32 var_256;
    const wp::int32 var_257 = 1;
    wp::float32 var_258;
    wp::float32 var_259;
    wp::float32 var_260;
    const wp::int32 var_261 = 0;
    wp::float32 var_262;
    const wp::int32 var_263 = 1;
    wp::float32 var_264;
    wp::float32 var_265;
    wp::float32 var_266;
    const wp::int32 var_267 = 1;
    wp::float32 var_268;
    const wp::int32 var_269 = 0;
    wp::float32 var_270;
    wp::float32 var_271;
    wp::float32 var_272;
    const wp::int32 var_273 = 0;
    wp::float32 var_274;
    const wp::int32 var_275 = 1;
    wp::float32 var_276;
    wp::float32 var_277;
    wp::float32 var_278;
    const wp::int32 var_279 = 0;
    wp::float32 var_280;
    const wp::int32 var_281 = 1;
    wp::float32 var_282;
    wp::float32 var_283;
    const wp::int32 var_284 = 1;
    wp::float32 var_285;
    const wp::int32 var_286 = 0;
    wp::float32 var_287;
    wp::float32 var_288;
    wp::float32 var_289;
    const wp::int32 var_290 = 0;
    wp::float32 var_291;
    const wp::int32 var_292 = 1;
    wp::float32 var_293;
    wp::float32 var_294;
    wp::float32 var_295;
    const wp::int32 var_296 = 0;
    wp::float32 var_297;
    const wp::int32 var_298 = 1;
    wp::float32 var_299;
    wp::float32 var_300;
    wp::float32 var_301;
    const wp::int32 var_302 = 1;
    wp::float32 var_303;
    const wp::int32 var_304 = 0;
    wp::float32 var_305;
    wp::float32 var_306;
    wp::float32 var_307;
    const wp::int32 var_308 = 0;
    wp::float32 var_309;
    const wp::int32 var_310 = 1;
    wp::float32 var_311;
    wp::float32 var_312;
    wp::float32 var_313;
    wp::int32 var_314;
    wp::int32 var_315;
    wp::int32 var_316;
    bool var_317;
    wp::float32 var_318;
    wp::float32 var_319;
    wp::float32 var_320;
    wp::vec_t<3, wp::float32> var_321;
    const wp::float32 var_322 = 1e+30;
    wp::float32 var_323;
    const wp::float32 var_324 = 0.0;
    const wp::float32 var_325 = 0.0;
    const wp::float32 var_326 = 0.0;
    wp::vec_t<3, wp::float32> var_327;
    bool var_328;
    wp::vec_t<2, wp::float32> var_329;
    const wp::int32 var_330 = 0;
    wp::float32 var_331;
    wp::vec_t<3, wp::float32> var_332;
    const wp::int32 var_333 = 1;
    wp::float32 var_334;
    wp::vec_t<3, wp::float32> var_335;
    wp::vec_t<3, wp::float32> var_336;
    wp::float32 var_337;
    const wp::float32 var_338 = 0.0;
    const wp::int32 var_339 = 0;
    const wp::int32 var_340 = 0;
    wp::float32 var_341;
    const wp::int32 var_342 = 1;
    const wp::int32 var_343 = 1;
    wp::float32 var_344;
    const wp::int32 var_345 = 2;
    wp::float32 var_346;
    wp::float32 var_347;
    bool var_348;
    wp::vec_t<2, wp::float32> var_349;
    const wp::int32 var_350 = 0;
    wp::float32 var_351;
    wp::vec_t<3, wp::float32> var_352;
    const wp::int32 var_353 = 1;
    wp::float32 var_354;
    wp::vec_t<3, wp::float32> var_355;
    wp::vec_t<3, wp::float32> var_356;
    wp::float32 var_357;
    bool var_358;
    const wp::int32 var_359 = 0;
    wp::float32 var_360;
    const wp::int32 var_361 = 0;
    const wp::float32 var_362 = 0.0;
    const wp::int32 var_363 = 1;
    const wp::int32 var_364 = 1;
    wp::float32 var_365;
    const wp::int32 var_366 = 2;
    wp::float32 var_367;
    wp::float32 var_368;
    wp::float32 var_369;
    wp::vec_t<2, wp::float32> var_370;
    wp::vec_t<3, wp::float32> var_371;
    wp::float32 var_372;
    bool var_373;
    wp::vec_t<2, wp::float32> var_374;
    const wp::int32 var_375 = 0;
    wp::float32 var_376;
    wp::vec_t<3, wp::float32> var_377;
    const wp::int32 var_378 = 1;
    wp::float32 var_379;
    wp::vec_t<3, wp::float32> var_380;
    wp::vec_t<3, wp::float32> var_381;
    wp::float32 var_382;
    bool var_383;
    const wp::int32 var_384 = 0;
    wp::float32 var_385;
    const wp::int32 var_386 = 0;
    const wp::int32 var_387 = 1;
    wp::float32 var_388;
    const wp::int32 var_389 = 1;
    const wp::float32 var_390 = 0.0;
    const wp::int32 var_391 = 2;
    wp::vec_t<2, wp::float32> var_392;
    wp::vec_t<3, wp::float32> var_393;
    wp::float32 var_394;
    //---------
    // forward
    // def _S2D(s1: wp.vec3, s2: wp.vec3, s3: wp.vec3) -> wp.vec3:                            <L 445>
    // p_o, ret = _project_origin_plane(s1, s2, s3)                                           <L 447>
    _project_origin_plane_0(var_s1, var_s2, var_s3, var_0, var_1);
    // if ret:                                                                                <L 448>
    if (var_1) {
        // v = _S1D(s1, s2)                                                                   <L 449>
        var_2 = _S1D_0(var_s1, var_s2);
        // return wp.vec3(v[0], v[1], 0.0)                                                    <L 450>
        var_4 = wp::extract(var_2, var_3);
        var_6 = wp::extract(var_2, var_5);
        var_8 = wp::vec_t<3, wp::float32>(var_4, var_6, var_7);
        return var_8;
    }
    // M_14 = s2[1] * s3[2] - s2[2] * s3[1] - s1[1] * s3[2] + s1[2] * s3[1] + s1[1] * s2[2] - s1[2] * s2[1]       <L 457>
    var_10 = wp::extract(var_s2, var_9);
    var_12 = wp::extract(var_s3, var_11);
    var_13 = wp::mul(var_10, var_12);
    var_15 = wp::extract(var_s2, var_14);
    var_17 = wp::extract(var_s3, var_16);
    var_18 = wp::mul(var_15, var_17);
    var_19 = wp::sub(var_13, var_18);
    var_21 = wp::extract(var_s1, var_20);
    var_23 = wp::extract(var_s3, var_22);
    var_24 = wp::mul(var_21, var_23);
    var_25 = wp::sub(var_19, var_24);
    var_27 = wp::extract(var_s1, var_26);
    var_29 = wp::extract(var_s3, var_28);
    var_30 = wp::mul(var_27, var_29);
    var_31 = wp::add(var_25, var_30);
    var_33 = wp::extract(var_s1, var_32);
    var_35 = wp::extract(var_s2, var_34);
    var_36 = wp::mul(var_33, var_35);
    var_37 = wp::add(var_31, var_36);
    var_39 = wp::extract(var_s1, var_38);
    var_41 = wp::extract(var_s2, var_40);
    var_42 = wp::mul(var_39, var_41);
    var_43 = wp::sub(var_37, var_42);
    // M_24 = s2[0] * s3[2] - s2[2] * s3[0] - s1[0] * s3[2] + s1[2] * s3[0] + s1[0] * s2[2] - s1[2] * s2[0]       <L 458>
    var_45 = wp::extract(var_s2, var_44);
    var_47 = wp::extract(var_s3, var_46);
    var_48 = wp::mul(var_45, var_47);
    var_50 = wp::extract(var_s2, var_49);
    var_52 = wp::extract(var_s3, var_51);
    var_53 = wp::mul(var_50, var_52);
    var_54 = wp::sub(var_48, var_53);
    var_56 = wp::extract(var_s1, var_55);
    var_58 = wp::extract(var_s3, var_57);
    var_59 = wp::mul(var_56, var_58);
    var_60 = wp::sub(var_54, var_59);
    var_62 = wp::extract(var_s1, var_61);
    var_64 = wp::extract(var_s3, var_63);
    var_65 = wp::mul(var_62, var_64);
    var_66 = wp::add(var_60, var_65);
    var_68 = wp::extract(var_s1, var_67);
    var_70 = wp::extract(var_s2, var_69);
    var_71 = wp::mul(var_68, var_70);
    var_72 = wp::add(var_66, var_71);
    var_74 = wp::extract(var_s1, var_73);
    var_76 = wp::extract(var_s2, var_75);
    var_77 = wp::mul(var_74, var_76);
    var_78 = wp::sub(var_72, var_77);
    // M_34 = s2[0] * s3[1] - s2[1] * s3[0] - s1[0] * s3[1] + s1[1] * s3[0] + s1[0] * s2[1] - s1[1] * s2[0]       <L 459>
    var_80 = wp::extract(var_s2, var_79);
    var_82 = wp::extract(var_s3, var_81);
    var_83 = wp::mul(var_80, var_82);
    var_85 = wp::extract(var_s2, var_84);
    var_87 = wp::extract(var_s3, var_86);
    var_88 = wp::mul(var_85, var_87);
    var_89 = wp::sub(var_83, var_88);
    var_91 = wp::extract(var_s1, var_90);
    var_93 = wp::extract(var_s3, var_92);
    var_94 = wp::mul(var_91, var_93);
    var_95 = wp::sub(var_89, var_94);
    var_97 = wp::extract(var_s1, var_96);
    var_99 = wp::extract(var_s3, var_98);
    var_100 = wp::mul(var_97, var_99);
    var_101 = wp::add(var_95, var_100);
    var_103 = wp::extract(var_s1, var_102);
    var_105 = wp::extract(var_s2, var_104);
    var_106 = wp::mul(var_103, var_105);
    var_107 = wp::add(var_101, var_106);
    var_109 = wp::extract(var_s1, var_108);
    var_111 = wp::extract(var_s2, var_110);
    var_112 = wp::mul(var_109, var_111);
    var_113 = wp::sub(var_107, var_112);
    // M_max = 0.0                                                                            <L 462>
    // s1_2D = wp.vec2(0.0)                                                                   <L 463>
    var_116 = wp::vec_t<2, wp::float32>(var_115);
    // s2_2D = wp.vec2(0.0)                                                                   <L 464>
    var_118 = wp::vec_t<2, wp::float32>(var_117);
    // s3_2D = wp.vec2(0.0)                                                                   <L 465>
    var_120 = wp::vec_t<2, wp::float32>(var_119);
    // p_o_2D = wp.vec2(0.0)                                                                  <L 466>
    var_122 = wp::vec_t<2, wp::float32>(var_121);
    // mu1 = wp.abs(M_14)                                                                     <L 468>
    var_123 = wp::abs(var_43);
    // mu2 = wp.abs(M_24)                                                                     <L 469>
    var_124 = wp::abs(var_78);
    // mu3 = wp.abs(M_34)                                                                     <L 470>
    var_125 = wp::abs(var_113);
    // if mu1 >= mu2 and mu1 >= mu3:                                                          <L 472>
    var_127 = (var_123 >= var_124);
    var_126 = var_127;
    if (var_126) {
        var_128 = (var_123 >= var_125);
        var_126 = var_126 && var_128;
    }
    if (var_126) {
        // M_max = M_14                                                                       <L 473>
        var_129 = wp::copy(var_43);
        // s1_2D[0] = s1[1]                                                                   <L 474>
        var_131 = wp::extract(var_s1, var_130);
        wp::assign_inplace(var_116, var_132, var_131);
        // s1_2D[1] = s1[2]                                                                   <L 475>
        var_134 = wp::extract(var_s1, var_133);
        wp::assign_inplace(var_116, var_135, var_134);
        // s2_2D[0] = s2[1]                                                                   <L 477>
        var_137 = wp::extract(var_s2, var_136);
        wp::assign_inplace(var_118, var_138, var_137);
        // s2_2D[1] = s2[2]                                                                   <L 478>
        var_140 = wp::extract(var_s2, var_139);
        wp::assign_inplace(var_118, var_141, var_140);
        // s3_2D[0] = s3[1]                                                                   <L 480>
        var_143 = wp::extract(var_s3, var_142);
        wp::assign_inplace(var_120, var_144, var_143);
        // s3_2D[1] = s3[2]                                                                   <L 481>
        var_146 = wp::extract(var_s3, var_145);
        wp::assign_inplace(var_120, var_147, var_146);
        // p_o_2D[0] = p_o[1]                                                                 <L 483>
        var_149 = wp::extract(var_0, var_148);
        wp::assign_inplace(var_122, var_150, var_149);
        // p_o_2D[1] = p_o[2]                                                                 <L 484>
        var_152 = wp::extract(var_0, var_151);
        wp::assign_inplace(var_122, var_153, var_152);
    }
    var_154 = wp::where(var_126, var_129, var_114);
    if (!var_126) {
        // elif mu2 >= mu3:                                                                   <L 485>
        var_155 = (var_124 >= var_125);
        if (var_155) {
            // M_max = M_24                                                                   <L 486>
            var_156 = wp::copy(var_78);
            // s1_2D[0] = s1[0]                                                               <L 487>
            var_158 = wp::extract(var_s1, var_157);
            wp::assign_inplace(var_116, var_159, var_158);
            // s1_2D[1] = s1[2]                                                               <L 488>
            var_161 = wp::extract(var_s1, var_160);
            wp::assign_inplace(var_116, var_162, var_161);
            // s2_2D[0] = s2[0]                                                               <L 490>
            var_164 = wp::extract(var_s2, var_163);
            wp::assign_inplace(var_118, var_165, var_164);
            // s2_2D[1] = s2[2]                                                               <L 491>
            var_167 = wp::extract(var_s2, var_166);
            wp::assign_inplace(var_118, var_168, var_167);
            // s3_2D[0] = s3[0]                                                               <L 493>
            var_170 = wp::extract(var_s3, var_169);
            wp::assign_inplace(var_120, var_171, var_170);
            // s3_2D[1] = s3[2]                                                               <L 494>
            var_173 = wp::extract(var_s3, var_172);
            wp::assign_inplace(var_120, var_174, var_173);
            // p_o_2D[0] = p_o[0]                                                             <L 496>
            var_176 = wp::extract(var_0, var_175);
            wp::assign_inplace(var_122, var_177, var_176);
            // p_o_2D[1] = p_o[2]                                                             <L 497>
            var_179 = wp::extract(var_0, var_178);
            wp::assign_inplace(var_122, var_180, var_179);
        }
        var_181 = wp::where(var_155, var_156, var_154);
        if (!var_155) {
            // M_max = M_34                                                                   <L 499>
            var_182 = wp::copy(var_113);
            // s1_2D[0] = s1[0]                                                               <L 500>
            var_184 = wp::extract(var_s1, var_183);
            wp::assign_inplace(var_116, var_185, var_184);
            // s1_2D[1] = s1[1]                                                               <L 501>
            var_187 = wp::extract(var_s1, var_186);
            wp::assign_inplace(var_116, var_188, var_187);
            // s2_2D[0] = s2[0]                                                               <L 503>
            var_190 = wp::extract(var_s2, var_189);
            wp::assign_inplace(var_118, var_191, var_190);
            // s2_2D[1] = s2[1]                                                               <L 504>
            var_193 = wp::extract(var_s2, var_192);
            wp::assign_inplace(var_118, var_194, var_193);
            // s3_2D[0] = s3[0]                                                               <L 506>
            var_196 = wp::extract(var_s3, var_195);
            wp::assign_inplace(var_120, var_197, var_196);
            // s3_2D[1] = s3[1]                                                               <L 507>
            var_199 = wp::extract(var_s3, var_198);
            wp::assign_inplace(var_120, var_200, var_199);
            // p_o_2D[0] = p_o[0]                                                             <L 509>
            var_202 = wp::extract(var_0, var_201);
            wp::assign_inplace(var_122, var_203, var_202);
            // p_o_2D[1] = p_o[1]                                                             <L 510>
            var_205 = wp::extract(var_0, var_204);
            wp::assign_inplace(var_122, var_206, var_205);
        }
        var_207 = wp::where(var_155, var_181, var_182);
    }
    var_208 = wp::where(var_126, var_154, var_207);
    // C31 = (                                                                                <L 518>
    // p_o_2D[0] * s2_2D[1]                                                                   <L 519>
    var_210 = wp::extract(var_122, var_209);
    var_212 = wp::extract(var_118, var_211);
    var_213 = wp::mul(var_210, var_212);
    // + p_o_2D[1] * s3_2D[0]                                                                 <L 520>
    var_215 = wp::extract(var_122, var_214);
    var_217 = wp::extract(var_120, var_216);
    var_218 = wp::mul(var_215, var_217);
    var_219 = wp::add(var_213, var_218);
    // + s2_2D[0] * s3_2D[1]                                                                  <L 521>
    var_221 = wp::extract(var_118, var_220);
    var_223 = wp::extract(var_120, var_222);
    var_224 = wp::mul(var_221, var_223);
    var_225 = wp::add(var_219, var_224);
    // - p_o_2D[0] * s3_2D[1]                                                                 <L 522>
    var_227 = wp::extract(var_122, var_226);
    var_229 = wp::extract(var_120, var_228);
    var_230 = wp::mul(var_227, var_229);
    var_231 = wp::sub(var_225, var_230);
    // - p_o_2D[1] * s2_2D[0]                                                                 <L 523>
    var_233 = wp::extract(var_122, var_232);
    var_235 = wp::extract(var_118, var_234);
    var_236 = wp::mul(var_233, var_235);
    var_237 = wp::sub(var_231, var_236);
    // - s3_2D[0] * s2_2D[1]                                                                  <L 524>
    var_239 = wp::extract(var_120, var_238);
    var_241 = wp::extract(var_118, var_240);
    var_242 = wp::mul(var_239, var_241);
    var_243 = wp::sub(var_237, var_242);
    // C32 = (                                                                                <L 528>
    // p_o_2D[0] * s3_2D[1]                                                                   <L 529>
    var_245 = wp::extract(var_122, var_244);
    var_247 = wp::extract(var_120, var_246);
    var_248 = wp::mul(var_245, var_247);
    // + p_o_2D[1] * s1_2D[0]                                                                 <L 530>
    var_250 = wp::extract(var_122, var_249);
    var_252 = wp::extract(var_116, var_251);
    var_253 = wp::mul(var_250, var_252);
    var_254 = wp::add(var_248, var_253);
    // + s3_2D[0] * s1_2D[1]                                                                  <L 531>
    var_256 = wp::extract(var_120, var_255);
    var_258 = wp::extract(var_116, var_257);
    var_259 = wp::mul(var_256, var_258);
    var_260 = wp::add(var_254, var_259);
    // - p_o_2D[0] * s1_2D[1]                                                                 <L 532>
    var_262 = wp::extract(var_122, var_261);
    var_264 = wp::extract(var_116, var_263);
    var_265 = wp::mul(var_262, var_264);
    var_266 = wp::sub(var_260, var_265);
    // - p_o_2D[1] * s3_2D[0]                                                                 <L 533>
    var_268 = wp::extract(var_122, var_267);
    var_270 = wp::extract(var_120, var_269);
    var_271 = wp::mul(var_268, var_270);
    var_272 = wp::sub(var_266, var_271);
    // - s1_2D[0] * s3_2D[1]                                                                  <L 534>
    var_274 = wp::extract(var_116, var_273);
    var_276 = wp::extract(var_120, var_275);
    var_277 = wp::mul(var_274, var_276);
    var_278 = wp::sub(var_272, var_277);
    // C33 = (                                                                                <L 538>
    // p_o_2D[0] * s1_2D[1]                                                                   <L 539>
    var_280 = wp::extract(var_122, var_279);
    var_282 = wp::extract(var_116, var_281);
    var_283 = wp::mul(var_280, var_282);
    // + p_o_2D[1] * s2_2D[0]                                                                 <L 540>
    var_285 = wp::extract(var_122, var_284);
    var_287 = wp::extract(var_118, var_286);
    var_288 = wp::mul(var_285, var_287);
    var_289 = wp::add(var_283, var_288);
    // + s1_2D[0] * s2_2D[1]                                                                  <L 541>
    var_291 = wp::extract(var_116, var_290);
    var_293 = wp::extract(var_118, var_292);
    var_294 = wp::mul(var_291, var_293);
    var_295 = wp::add(var_289, var_294);
    // - p_o_2D[0] * s2_2D[1]                                                                 <L 542>
    var_297 = wp::extract(var_122, var_296);
    var_299 = wp::extract(var_118, var_298);
    var_300 = wp::mul(var_297, var_299);
    var_301 = wp::sub(var_295, var_300);
    // - p_o_2D[1] * s1_2D[0]                                                                 <L 543>
    var_303 = wp::extract(var_122, var_302);
    var_305 = wp::extract(var_116, var_304);
    var_306 = wp::mul(var_303, var_305);
    var_307 = wp::sub(var_301, var_306);
    // - s2_2D[0] * s1_2D[1]                                                                  <L 544>
    var_309 = wp::extract(var_118, var_308);
    var_311 = wp::extract(var_116, var_310);
    var_312 = wp::mul(var_309, var_311);
    var_313 = wp::sub(var_307, var_312);
    // comp1 = _same_sign(M_max, C31)                                                         <L 547>
    var_314 = _same_sign_0(var_208, var_243);
    // comp2 = _same_sign(M_max, C32)                                                         <L 548>
    var_315 = _same_sign_0(var_208, var_278);
    // comp3 = _same_sign(M_max, C33)                                                         <L 549>
    var_316 = _same_sign_0(var_208, var_313);
    // if comp1 and comp2 and comp3:                                                          <L 552>
    var_317 = var_314;
    if (var_317) {
        var_317 = var_317 && var_315;
    }
    if (var_317) {
        var_317 = var_317 && var_316;
    }
    if (var_317) {
        // return wp.vec3(C31 / M_max, C32 / M_max, C33 / M_max)                              <L 553>
        var_318 = wp::div(var_243, var_208);
        var_319 = wp::div(var_278, var_208);
        var_320 = wp::div(var_313, var_208);
        var_321 = wp::vec_t<3, wp::float32>(var_318, var_319, var_320);
        return var_321;
    }
    // dmin = FLOAT_MAX                                                                       <L 556>
    var_323 = wp::copy(var_322);
    // lmbda = wp.vec3(0.0, 0.0, 0.0)                                                         <L 557>
    var_327 = wp::vec_t<3, wp::float32>(var_324, var_325, var_326);
    // if not comp1:                                                                          <L 559>
    var_328 = wp::unot(var_314);
    if (var_328) {
        // sublmbda = _S1D(s2, s3)                                                            <L 560>
        var_329 = _S1D_0(var_s2, var_s3);
        // x = sublmbda[0] * s2 + sublmbda[1] * s3                                            <L 561>
        var_331 = wp::extract(var_329, var_330);
        var_332 = wp::mul(var_331, var_s2);
        var_334 = wp::extract(var_329, var_333);
        var_335 = wp::mul(var_334, var_s3);
        var_336 = wp::add(var_332, var_335);
        // d = wp.dot(x, x)                                                                   <L 562>
        var_337 = wp::dot(var_336, var_336);
        // lmbda[0] = 0.0                                                                     <L 563>
        wp::assign_inplace(var_327, var_339, var_338);
        // lmbda[1] = sublmbda[0]                                                             <L 564>
        var_341 = wp::extract(var_329, var_340);
        wp::assign_inplace(var_327, var_342, var_341);
        // lmbda[2] = sublmbda[1]                                                             <L 565>
        var_344 = wp::extract(var_329, var_343);
        wp::assign_inplace(var_327, var_345, var_344);
        // dmin = d                                                                           <L 566>
        var_346 = wp::copy(var_337);
    }
    var_347 = wp::where(var_328, var_346, var_323);
    // if not comp2:                                                                          <L 568>
    var_348 = wp::unot(var_315);
    if (var_348) {
        // sublmbda = _S1D(s1, s3)                                                            <L 569>
        var_349 = _S1D_0(var_s1, var_s3);
        // x = sublmbda[0] * s1 + sublmbda[1] * s3                                            <L 570>
        var_351 = wp::extract(var_349, var_350);
        var_352 = wp::mul(var_351, var_s1);
        var_354 = wp::extract(var_349, var_353);
        var_355 = wp::mul(var_354, var_s3);
        var_356 = wp::add(var_352, var_355);
        // d = wp.dot(x, x)                                                                   <L 571>
        var_357 = wp::dot(var_356, var_356);
        // if d < dmin:                                                                       <L 572>
        var_358 = (var_357 < var_347);
        if (var_358) {
            // lmbda[0] = sublmbda[0]                                                         <L 573>
            var_360 = wp::extract(var_349, var_359);
            wp::assign_inplace(var_327, var_361, var_360);
            // lmbda[1] = 0.0                                                                 <L 574>
            wp::assign_inplace(var_327, var_363, var_362);
            // lmbda[2] = sublmbda[1]                                                         <L 575>
            var_365 = wp::extract(var_349, var_364);
            wp::assign_inplace(var_327, var_366, var_365);
            // dmin = d                                                                       <L 576>
            var_367 = wp::copy(var_357);
        }
        var_368 = wp::where(var_358, var_367, var_347);
    }
    var_369 = wp::where(var_348, var_368, var_347);
    var_370 = wp::where(var_348, var_349, var_329);
    var_371 = wp::where(var_348, var_356, var_336);
    var_372 = wp::where(var_348, var_357, var_337);
    // if not comp3:                                                                          <L 578>
    var_373 = wp::unot(var_316);
    if (var_373) {
        // sublmbda = _S1D(s1, s2)                                                            <L 579>
        var_374 = _S1D_0(var_s1, var_s2);
        // x = sublmbda[0] * s1 + sublmbda[1] * s2                                            <L 580>
        var_376 = wp::extract(var_374, var_375);
        var_377 = wp::mul(var_376, var_s1);
        var_379 = wp::extract(var_374, var_378);
        var_380 = wp::mul(var_379, var_s2);
        var_381 = wp::add(var_377, var_380);
        // d = wp.dot(x, x)                                                                   <L 581>
        var_382 = wp::dot(var_381, var_381);
        // if d < dmin:                                                                       <L 582>
        var_383 = (var_382 < var_369);
        if (var_383) {
            // lmbda[0] = sublmbda[0]                                                         <L 583>
            var_385 = wp::extract(var_374, var_384);
            wp::assign_inplace(var_327, var_386, var_385);
            // lmbda[1] = sublmbda[1]                                                         <L 584>
            var_388 = wp::extract(var_374, var_387);
            wp::assign_inplace(var_327, var_389, var_388);
            // lmbda[2] = 0.0                                                                 <L 585>
            wp::assign_inplace(var_327, var_391, var_390);
        }
    }
    var_392 = wp::where(var_373, var_374, var_370);
    var_393 = wp::where(var_373, var_381, var_371);
    var_394 = wp::where(var_373, var_382, var_372);
    // return lmbda                                                                           <L 586>
    return var_327;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:368
static CUDA_CALLABLE wp::vec_t<4, wp::float32> _S3D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2,
    wp::vec_t<3, wp::float32> var_s3,
    wp::vec_t<3, wp::float32> var_s4)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::int32 var_9;
    wp::int32 var_10;
    wp::int32 var_11;
    wp::int32 var_12;
    bool var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    wp::vec_t<4, wp::float32> var_18;
    const wp::float32 var_19 = 0.0;
    const wp::float32 var_20 = 0.0;
    const wp::float32 var_21 = 0.0;
    const wp::float32 var_22 = 0.0;
    wp::vec_t<4, wp::float32> var_23;
    const wp::float32 var_24 = 1e+30;
    wp::float32 var_25;
    bool var_26;
    wp::vec_t<3, wp::float32> var_27;
    const wp::int32 var_28 = 0;
    wp::float32 var_29;
    wp::vec_t<3, wp::float32> var_30;
    const wp::int32 var_31 = 1;
    wp::float32 var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    const wp::int32 var_35 = 2;
    wp::float32 var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::float32 var_39;
    const wp::float32 var_40 = 0.0;
    const wp::int32 var_41 = 0;
    const wp::int32 var_42 = 0;
    wp::float32 var_43;
    const wp::int32 var_44 = 1;
    const wp::int32 var_45 = 1;
    wp::float32 var_46;
    const wp::int32 var_47 = 2;
    const wp::int32 var_48 = 2;
    wp::float32 var_49;
    const wp::int32 var_50 = 3;
    wp::float32 var_51;
    wp::float32 var_52;
    bool var_53;
    wp::vec_t<3, wp::float32> var_54;
    const wp::int32 var_55 = 0;
    wp::float32 var_56;
    wp::vec_t<3, wp::float32> var_57;
    const wp::int32 var_58 = 1;
    wp::float32 var_59;
    wp::vec_t<3, wp::float32> var_60;
    wp::vec_t<3, wp::float32> var_61;
    const wp::int32 var_62 = 2;
    wp::float32 var_63;
    wp::vec_t<3, wp::float32> var_64;
    wp::vec_t<3, wp::float32> var_65;
    wp::float32 var_66;
    bool var_67;
    const wp::int32 var_68 = 0;
    wp::float32 var_69;
    const wp::int32 var_70 = 0;
    const wp::float32 var_71 = 0.0;
    const wp::int32 var_72 = 1;
    const wp::int32 var_73 = 1;
    wp::float32 var_74;
    const wp::int32 var_75 = 2;
    const wp::int32 var_76 = 2;
    wp::float32 var_77;
    const wp::int32 var_78 = 3;
    wp::float32 var_79;
    wp::float32 var_80;
    wp::float32 var_81;
    wp::vec_t<3, wp::float32> var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::float32 var_84;
    bool var_85;
    wp::vec_t<3, wp::float32> var_86;
    const wp::int32 var_87 = 0;
    wp::float32 var_88;
    wp::vec_t<3, wp::float32> var_89;
    const wp::int32 var_90 = 1;
    wp::float32 var_91;
    wp::vec_t<3, wp::float32> var_92;
    wp::vec_t<3, wp::float32> var_93;
    const wp::int32 var_94 = 2;
    wp::float32 var_95;
    wp::vec_t<3, wp::float32> var_96;
    wp::vec_t<3, wp::float32> var_97;
    wp::float32 var_98;
    bool var_99;
    const wp::int32 var_100 = 0;
    wp::float32 var_101;
    const wp::int32 var_102 = 0;
    const wp::int32 var_103 = 1;
    wp::float32 var_104;
    const wp::int32 var_105 = 1;
    const wp::float32 var_106 = 0.0;
    const wp::int32 var_107 = 2;
    const wp::int32 var_108 = 2;
    wp::float32 var_109;
    const wp::int32 var_110 = 3;
    wp::float32 var_111;
    wp::float32 var_112;
    wp::float32 var_113;
    wp::vec_t<3, wp::float32> var_114;
    wp::vec_t<3, wp::float32> var_115;
    wp::float32 var_116;
    bool var_117;
    wp::vec_t<3, wp::float32> var_118;
    const wp::int32 var_119 = 0;
    wp::float32 var_120;
    wp::vec_t<3, wp::float32> var_121;
    const wp::int32 var_122 = 1;
    wp::float32 var_123;
    wp::vec_t<3, wp::float32> var_124;
    wp::vec_t<3, wp::float32> var_125;
    const wp::int32 var_126 = 2;
    wp::float32 var_127;
    wp::vec_t<3, wp::float32> var_128;
    wp::vec_t<3, wp::float32> var_129;
    wp::float32 var_130;
    bool var_131;
    const wp::int32 var_132 = 0;
    wp::float32 var_133;
    const wp::int32 var_134 = 0;
    const wp::int32 var_135 = 1;
    wp::float32 var_136;
    const wp::int32 var_137 = 1;
    const wp::int32 var_138 = 2;
    wp::float32 var_139;
    const wp::int32 var_140 = 2;
    const wp::float32 var_141 = 0.0;
    const wp::int32 var_142 = 3;
    wp::vec_t<3, wp::float32> var_143;
    wp::vec_t<3, wp::float32> var_144;
    wp::float32 var_145;
    //---------
    // forward
    // def _S3D(s1: wp.vec3, s2: wp.vec3, s3: wp.vec3, s4: wp.vec3) -> wp.vec4:               <L 369>
    // C41 = -_det3(s2, s3, s4)                                                               <L 378>
    var_0 = _det3_0(var_s2, var_s3, var_s4);
    var_1 = wp::neg(var_0);
    // C42 = _det3(s1, s3, s4)                                                                <L 379>
    var_2 = _det3_0(var_s1, var_s3, var_s4);
    // C43 = -_det3(s1, s2, s4)                                                               <L 380>
    var_3 = _det3_0(var_s1, var_s2, var_s4);
    var_4 = wp::neg(var_3);
    // C44 = _det3(s1, s2, s3)                                                                <L 381>
    var_5 = _det3_0(var_s1, var_s2, var_s3);
    // m_det = C41 + C42 + C43 + C44                                                          <L 385>
    var_6 = wp::add(var_1, var_2);
    var_7 = wp::add(var_6, var_4);
    var_8 = wp::add(var_7, var_5);
    // comp1 = _same_sign(m_det, C41)                                                         <L 387>
    var_9 = _same_sign_0(var_8, var_1);
    // comp2 = _same_sign(m_det, C42)                                                         <L 388>
    var_10 = _same_sign_0(var_8, var_2);
    // comp3 = _same_sign(m_det, C43)                                                         <L 389>
    var_11 = _same_sign_0(var_8, var_4);
    // comp4 = _same_sign(m_det, C44)                                                         <L 390>
    var_12 = _same_sign_0(var_8, var_5);
    // if comp1 and comp2 and comp3 and comp4:                                                <L 393>
    var_13 = var_9;
    if (var_13) {
        var_13 = var_13 && var_10;
    }
    if (var_13) {
        var_13 = var_13 && var_11;
    }
    if (var_13) {
        var_13 = var_13 && var_12;
    }
    if (var_13) {
        // return wp.vec4(C41 / m_det, C42 / m_det, C43 / m_det, C44 / m_det)                 <L 394>
        var_14 = wp::div(var_1, var_8);
        var_15 = wp::div(var_2, var_8);
        var_16 = wp::div(var_4, var_8);
        var_17 = wp::div(var_5, var_8);
        var_18 = wp::vec_t<4, wp::float32>(var_14, var_15, var_16, var_17);
        return var_18;
    }
    // lmbda = wp.vec4(0.0, 0.0, 0.0, 0.0)                                                    <L 397>
    var_23 = wp::vec_t<4, wp::float32>(var_19, var_20, var_21, var_22);
    // dmin = FLOAT_MAX                                                                       <L 398>
    var_25 = wp::copy(var_24);
    // if not comp1:                                                                          <L 400>
    var_26 = wp::unot(var_9);
    if (var_26) {
        // sublmbda = _S2D(s2, s3, s4)                                                        <L 401>
        var_27 = _S2D_0(var_s2, var_s3, var_s4);
        // x = sublmbda[0] * s2 + sublmbda[1] * s3 + sublmbda[2] * s4                         <L 402>
        var_29 = wp::extract(var_27, var_28);
        var_30 = wp::mul(var_29, var_s2);
        var_32 = wp::extract(var_27, var_31);
        var_33 = wp::mul(var_32, var_s3);
        var_34 = wp::add(var_30, var_33);
        var_36 = wp::extract(var_27, var_35);
        var_37 = wp::mul(var_36, var_s4);
        var_38 = wp::add(var_34, var_37);
        // d = wp.dot(x, x)                                                                   <L 403>
        var_39 = wp::dot(var_38, var_38);
        // lmbda[0] = 0.0                                                                     <L 404>
        wp::assign_inplace(var_23, var_41, var_40);
        // lmbda[1] = sublmbda[0]                                                             <L 405>
        var_43 = wp::extract(var_27, var_42);
        wp::assign_inplace(var_23, var_44, var_43);
        // lmbda[2] = sublmbda[1]                                                             <L 406>
        var_46 = wp::extract(var_27, var_45);
        wp::assign_inplace(var_23, var_47, var_46);
        // lmbda[3] = sublmbda[2]                                                             <L 407>
        var_49 = wp::extract(var_27, var_48);
        wp::assign_inplace(var_23, var_50, var_49);
        // dmin = d                                                                           <L 408>
        var_51 = wp::copy(var_39);
    }
    var_52 = wp::where(var_26, var_51, var_25);
    // if not comp2:                                                                          <L 410>
    var_53 = wp::unot(var_10);
    if (var_53) {
        // sublmbda = _S2D(s1, s3, s4)                                                        <L 411>
        var_54 = _S2D_0(var_s1, var_s3, var_s4);
        // x = sublmbda[0] * s1 + sublmbda[1] * s3 + sublmbda[2] * s4                         <L 412>
        var_56 = wp::extract(var_54, var_55);
        var_57 = wp::mul(var_56, var_s1);
        var_59 = wp::extract(var_54, var_58);
        var_60 = wp::mul(var_59, var_s3);
        var_61 = wp::add(var_57, var_60);
        var_63 = wp::extract(var_54, var_62);
        var_64 = wp::mul(var_63, var_s4);
        var_65 = wp::add(var_61, var_64);
        // d = wp.dot(x, x)                                                                   <L 413>
        var_66 = wp::dot(var_65, var_65);
        // if d < dmin:                                                                       <L 414>
        var_67 = (var_66 < var_52);
        if (var_67) {
            // lmbda[0] = sublmbda[0]                                                         <L 415>
            var_69 = wp::extract(var_54, var_68);
            wp::assign_inplace(var_23, var_70, var_69);
            // lmbda[1] = 0.0                                                                 <L 416>
            wp::assign_inplace(var_23, var_72, var_71);
            // lmbda[2] = sublmbda[1]                                                         <L 417>
            var_74 = wp::extract(var_54, var_73);
            wp::assign_inplace(var_23, var_75, var_74);
            // lmbda[3] = sublmbda[2]                                                         <L 418>
            var_77 = wp::extract(var_54, var_76);
            wp::assign_inplace(var_23, var_78, var_77);
            // dmin = d                                                                       <L 419>
            var_79 = wp::copy(var_66);
        }
        var_80 = wp::where(var_67, var_79, var_52);
    }
    var_81 = wp::where(var_53, var_80, var_52);
    var_82 = wp::where(var_53, var_54, var_27);
    var_83 = wp::where(var_53, var_65, var_38);
    var_84 = wp::where(var_53, var_66, var_39);
    // if not comp3:                                                                          <L 421>
    var_85 = wp::unot(var_11);
    if (var_85) {
        // sublmbda = _S2D(s1, s2, s4)                                                        <L 422>
        var_86 = _S2D_0(var_s1, var_s2, var_s4);
        // x = sublmbda[0] * s1 + sublmbda[1] * s2 + sublmbda[2] * s4                         <L 423>
        var_88 = wp::extract(var_86, var_87);
        var_89 = wp::mul(var_88, var_s1);
        var_91 = wp::extract(var_86, var_90);
        var_92 = wp::mul(var_91, var_s2);
        var_93 = wp::add(var_89, var_92);
        var_95 = wp::extract(var_86, var_94);
        var_96 = wp::mul(var_95, var_s4);
        var_97 = wp::add(var_93, var_96);
        // d = wp.dot(x, x)                                                                   <L 424>
        var_98 = wp::dot(var_97, var_97);
        // if d < dmin:                                                                       <L 425>
        var_99 = (var_98 < var_81);
        if (var_99) {
            // lmbda[0] = sublmbda[0]                                                         <L 426>
            var_101 = wp::extract(var_86, var_100);
            wp::assign_inplace(var_23, var_102, var_101);
            // lmbda[1] = sublmbda[1]                                                         <L 427>
            var_104 = wp::extract(var_86, var_103);
            wp::assign_inplace(var_23, var_105, var_104);
            // lmbda[2] = 0.0                                                                 <L 428>
            wp::assign_inplace(var_23, var_107, var_106);
            // lmbda[3] = sublmbda[2]                                                         <L 429>
            var_109 = wp::extract(var_86, var_108);
            wp::assign_inplace(var_23, var_110, var_109);
            // dmin = d                                                                       <L 430>
            var_111 = wp::copy(var_98);
        }
        var_112 = wp::where(var_99, var_111, var_81);
    }
    var_113 = wp::where(var_85, var_112, var_81);
    var_114 = wp::where(var_85, var_86, var_82);
    var_115 = wp::where(var_85, var_97, var_83);
    var_116 = wp::where(var_85, var_98, var_84);
    // if not comp4:                                                                          <L 432>
    var_117 = wp::unot(var_12);
    if (var_117) {
        // sublmbda = _S2D(s1, s2, s3)                                                        <L 433>
        var_118 = _S2D_0(var_s1, var_s2, var_s3);
        // x = sublmbda[0] * s1 + sublmbda[1] * s2 + sublmbda[2] * s3                         <L 434>
        var_120 = wp::extract(var_118, var_119);
        var_121 = wp::mul(var_120, var_s1);
        var_123 = wp::extract(var_118, var_122);
        var_124 = wp::mul(var_123, var_s2);
        var_125 = wp::add(var_121, var_124);
        var_127 = wp::extract(var_118, var_126);
        var_128 = wp::mul(var_127, var_s3);
        var_129 = wp::add(var_125, var_128);
        // d = wp.dot(x, x)                                                                   <L 435>
        var_130 = wp::dot(var_129, var_129);
        // if d < dmin:                                                                       <L 436>
        var_131 = (var_130 < var_113);
        if (var_131) {
            // lmbda[0] = sublmbda[0]                                                         <L 437>
            var_133 = wp::extract(var_118, var_132);
            wp::assign_inplace(var_23, var_134, var_133);
            // lmbda[1] = sublmbda[1]                                                         <L 438>
            var_136 = wp::extract(var_118, var_135);
            wp::assign_inplace(var_23, var_137, var_136);
            // lmbda[2] = sublmbda[2]                                                         <L 439>
            var_139 = wp::extract(var_118, var_138);
            wp::assign_inplace(var_23, var_140, var_139);
            // lmbda[3] = 0.0                                                                 <L 440>
            wp::assign_inplace(var_23, var_142, var_141);
        }
    }
    var_143 = wp::where(var_117, var_118, var_114);
    var_144 = wp::where(var_117, var_129, var_115);
    var_145 = wp::where(var_117, var_130, var_116);
    // return lmbda                                                                           <L 441>
    return var_23;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:302
static CUDA_CALLABLE wp::vec_t<4, wp::float32> _subdistance_0(
    wp::int32 var_n,
    wp::mat_t<4, 3, wp::float32> var_simplex)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 4;
    bool var_1;
    const wp::int32 var_2 = 0;
    wp::vec_t<3, wp::float32> var_3;
    const wp::int32 var_4 = 1;
    wp::vec_t<3, wp::float32> var_5;
    const wp::int32 var_6 = 2;
    wp::vec_t<3, wp::float32> var_7;
    const wp::int32 var_8 = 3;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<4, wp::float32> var_10;
    const wp::int32 var_11 = 3;
    bool var_12;
    const wp::int32 var_13 = 0;
    wp::vec_t<3, wp::float32> var_14;
    const wp::int32 var_15 = 1;
    wp::vec_t<3, wp::float32> var_16;
    const wp::int32 var_17 = 2;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    const wp::int32 var_20 = 0;
    wp::float32 var_21;
    const wp::int32 var_22 = 1;
    wp::float32 var_23;
    const wp::int32 var_24 = 2;
    wp::float32 var_25;
    const wp::float32 var_26 = 0.0;
    wp::vec_t<4, wp::float32> var_27;
    const wp::int32 var_28 = 2;
    bool var_29;
    const wp::int32 var_30 = 0;
    wp::vec_t<3, wp::float32> var_31;
    const wp::int32 var_32 = 1;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<2, wp::float32> var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    const wp::int32 var_37 = 1;
    wp::float32 var_38;
    const wp::float32 var_39 = 0.0;
    const wp::float32 var_40 = 0.0;
    wp::vec_t<4, wp::float32> var_41;
    const wp::float32 var_42 = 1.0;
    const wp::float32 var_43 = 0.0;
    const wp::float32 var_44 = 0.0;
    const wp::float32 var_45 = 0.0;
    wp::vec_t<4, wp::float32> var_46;
    //---------
    // forward
    // def _subdistance(n: int, simplex: mat43) -> wp.vec4:                                   <L 303>
    // if n == 4:                                                                             <L 304>
    var_1 = (var_n == var_0);
    if (var_1) {
        // return _S3D(simplex[0], simplex[1], simplex[2], simplex[3])                        <L 305>
        var_3 = wp::extract(var_simplex, var_2);
        var_5 = wp::extract(var_simplex, var_4);
        var_7 = wp::extract(var_simplex, var_6);
        var_9 = wp::extract(var_simplex, var_8);
        var_10 = _S3D_0(var_3, var_5, var_7, var_9);
        return var_10;
    }
    // if n == 3:                                                                             <L 306>
    var_12 = (var_n == var_11);
    if (var_12) {
        // lmbda3 = _S2D(simplex[0], simplex[1], simplex[2])                                  <L 307>
        var_14 = wp::extract(var_simplex, var_13);
        var_16 = wp::extract(var_simplex, var_15);
        var_18 = wp::extract(var_simplex, var_17);
        var_19 = _S2D_0(var_14, var_16, var_18);
        // return wp.vec4(lmbda3[0], lmbda3[1], lmbda3[2], 0.0)                               <L 308>
        var_21 = wp::extract(var_19, var_20);
        var_23 = wp::extract(var_19, var_22);
        var_25 = wp::extract(var_19, var_24);
        var_27 = wp::vec_t<4, wp::float32>(var_21, var_23, var_25, var_26);
        return var_27;
    }
    // if n == 2:                                                                             <L 309>
    var_29 = (var_n == var_28);
    if (var_29) {
        // lmbda2 = _S1D(simplex[0], simplex[1])                                              <L 310>
        var_31 = wp::extract(var_simplex, var_30);
        var_33 = wp::extract(var_simplex, var_32);
        var_34 = _S1D_0(var_31, var_33);
        // return wp.vec4(lmbda2[0], lmbda2[1], 0.0, 0.0)                                     <L 311>
        var_36 = wp::extract(var_34, var_35);
        var_38 = wp::extract(var_34, var_37);
        var_41 = wp::vec_t<4, wp::float32>(var_36, var_38, var_39, var_40);
        return var_41;
    }
    // return wp.vec4(1.0, 0.0, 0.0, 0.0)                                                     <L 312>
    var_46 = wp::vec_t<4, wp::float32>(var_42, var_43, var_44, var_45);
    return var_46;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:291
static CUDA_CALLABLE wp::vec_t<3, wp::float32> _linear_combine_0(
    wp::int32 var_n,
    wp::vec_t<4, wp::float32> var_scl,
    wp::mat_t<4, 3, wp::float32> var_mat)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    bool var_1;
    const wp::int32 var_2 = 0;
    wp::float32 var_3;
    const wp::int32 var_4 = 0;
    wp::vec_t<3, wp::float32> var_5;
    wp::vec_t<3, wp::float32> var_6;
    const wp::int32 var_7 = 2;
    bool var_8;
    const wp::int32 var_9 = 0;
    wp::float32 var_10;
    const wp::int32 var_11 = 0;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    const wp::int32 var_14 = 1;
    wp::float32 var_15;
    const wp::int32 var_16 = 1;
    wp::vec_t<3, wp::float32> var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    const wp::int32 var_20 = 3;
    bool var_21;
    const wp::int32 var_22 = 0;
    wp::float32 var_23;
    const wp::int32 var_24 = 0;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    const wp::int32 var_27 = 1;
    wp::float32 var_28;
    const wp::int32 var_29 = 1;
    wp::vec_t<3, wp::float32> var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<3, wp::float32> var_32;
    const wp::int32 var_33 = 2;
    wp::float32 var_34;
    const wp::int32 var_35 = 2;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    const wp::int32 var_39 = 0;
    wp::float32 var_40;
    const wp::int32 var_41 = 0;
    wp::vec_t<3, wp::float32> var_42;
    wp::vec_t<3, wp::float32> var_43;
    const wp::int32 var_44 = 1;
    wp::float32 var_45;
    const wp::int32 var_46 = 1;
    wp::vec_t<3, wp::float32> var_47;
    wp::vec_t<3, wp::float32> var_48;
    wp::vec_t<3, wp::float32> var_49;
    const wp::int32 var_50 = 2;
    wp::float32 var_51;
    const wp::int32 var_52 = 2;
    wp::vec_t<3, wp::float32> var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::vec_t<3, wp::float32> var_55;
    const wp::int32 var_56 = 3;
    wp::float32 var_57;
    const wp::int32 var_58 = 3;
    wp::vec_t<3, wp::float32> var_59;
    wp::vec_t<3, wp::float32> var_60;
    wp::vec_t<3, wp::float32> var_61;
    //---------
    // forward
    // def _linear_combine(n: int, scl: wp.vec4, mat: mat43) -> wp.vec3:                      <L 292>
    // if n == 1:                                                                             <L 293>
    var_1 = (var_n == var_0);
    if (var_1) {
        // return scl[0] * mat[0]                                                             <L 294>
        var_3 = wp::extract(var_scl, var_2);
        var_5 = wp::extract(var_mat, var_4);
        var_6 = wp::mul(var_3, var_5);
        return var_6;
    }
    // if n == 2:                                                                             <L 295>
    var_8 = (var_n == var_7);
    if (var_8) {
        // return scl[0] * mat[0] + scl[1] * mat[1]                                           <L 296>
        var_10 = wp::extract(var_scl, var_9);
        var_12 = wp::extract(var_mat, var_11);
        var_13 = wp::mul(var_10, var_12);
        var_15 = wp::extract(var_scl, var_14);
        var_17 = wp::extract(var_mat, var_16);
        var_18 = wp::mul(var_15, var_17);
        var_19 = wp::add(var_13, var_18);
        return var_19;
    }
    // if n == 3:                                                                             <L 297>
    var_21 = (var_n == var_20);
    if (var_21) {
        // return scl[0] * mat[0] + scl[1] * mat[1] + scl[2] * mat[2]                         <L 298>
        var_23 = wp::extract(var_scl, var_22);
        var_25 = wp::extract(var_mat, var_24);
        var_26 = wp::mul(var_23, var_25);
        var_28 = wp::extract(var_scl, var_27);
        var_30 = wp::extract(var_mat, var_29);
        var_31 = wp::mul(var_28, var_30);
        var_32 = wp::add(var_26, var_31);
        var_34 = wp::extract(var_scl, var_33);
        var_36 = wp::extract(var_mat, var_35);
        var_37 = wp::mul(var_34, var_36);
        var_38 = wp::add(var_32, var_37);
        return var_38;
    }
    // return scl[0] * mat[0] + scl[1] * mat[1] + scl[2] * mat[2] + scl[3] * mat[3]           <L 299>
    var_40 = wp::extract(var_scl, var_39);
    var_42 = wp::extract(var_mat, var_41);
    var_43 = wp::mul(var_40, var_42);
    var_45 = wp::extract(var_scl, var_44);
    var_47 = wp::extract(var_mat, var_46);
    var_48 = wp::mul(var_45, var_47);
    var_49 = wp::add(var_43, var_48);
    var_51 = wp::extract(var_scl, var_50);
    var_53 = wp::extract(var_mat, var_52);
    var_54 = wp::mul(var_51, var_53);
    var_55 = wp::add(var_49, var_54);
    var_57 = wp::extract(var_scl, var_56);
    var_59 = wp::extract(var_mat, var_58);
    var_60 = wp::mul(var_57, var_59);
    var_61 = wp::add(var_55, var_60);
    return var_61;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:656
static CUDA_CALLABLE GJKResult_8184e255 gjk_0(
    wp::float32 var_tolerance,
    wp::int32 var_gjk_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::vec_t<3, wp::float32> var_x1_0,
    wp::vec_t<3, wp::float32> var_x2_0,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::float32 var_cutoff,
    bool var_is_discrete)
{
    //---------
    // primal vars
    wp::mat_t<4, 3, wp::float32> var_0;
    wp::mat_t<4, 3, wp::float32> var_1;
    wp::mat_t<4, 3, wp::float32> var_2;
    wp::vec_t<4, wp::int32> var_3;
    wp::vec_t<4, wp::int32> var_4;
    const wp::int32 var_5 = 0;
    wp::int32 var_6;
    const wp::float32 var_7 = 1.0;
    const wp::float32 var_8 = 0.0;
    const wp::float32 var_9 = 0.0;
    const wp::float32 var_10 = 0.0;
    wp::vec_t<4, wp::float32> var_11;
    const wp::float32 var_12 = 0.0;
    const wp::float32 var_13 = 0.5;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::float32 var_17 = 1e-15;
    wp::float32 var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::float32 var_22 = 0.0;
    wp::float32 var_23;
    wp::range_t var_24;
    wp::int32 var_25;
    bool var_26;
    bool var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    bool var_30;
    SupportPoint_e82efc60 var_31;
    SupportPoint_e82efc60 var_32;
    wp::vec_t<3, wp::float32>* var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::int32* var_35;
    wp::int32 var_36;
    wp::int32 var_37;
    wp::int32* var_38;
    wp::int32 var_39;
    wp::vec_t<3, wp::float32>* var_40;
    wp::vec_t<3, wp::float32> var_41;
    wp::int32* var_42;
    wp::int32 var_43;
    wp::int32 var_44;
    wp::int32* var_45;
    wp::int32 var_46;
    wp::vec_t<3, wp::float32> var_47;
    wp::vec_t<3, wp::float32> var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::vec_t<3, wp::float32> var_50;
    wp::vec_t<3, wp::float32> var_51;
    wp::float32 var_52;
    bool var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::float32 var_55;
    const wp::float32 var_56 = 0.0;
    bool var_57;
    const wp::float32 var_58 = 0.0;
    bool var_59;
    GJKResult_8184e255 var_60;
    const bool var_61 = true;
    const wp::int32 var_62 = 0;
    const wp::float32 var_63 = 1e+30;
    wp::int32* var_64;
    wp::int32 var_65;
    wp::int32 var_66;
    wp::int32* var_67;
    wp::int32 var_68;
    wp::int32 var_69;
    bool var_70;
    bool var_71;
    const wp::float32 var_72 = 0.0;
    bool var_73;
    wp::float32 var_74;
    bool var_75;
    GJKResult_8184e255 var_76;
    const bool var_77 = true;
    const wp::int32 var_78 = 0;
    wp::int32* var_79;
    wp::int32 var_80;
    wp::int32 var_81;
    wp::int32* var_82;
    wp::int32 var_83;
    wp::int32 var_84;
    GJKResult_8184e255 var_85;
    GJKResult_8184e255 var_86;
    GJKResult_8184e255 var_87;
    const wp::int32 var_88 = 1;
    wp::int32 var_89;
    wp::vec_t<4, wp::float32> var_90;
    const wp::int32 var_91 = 0;
    wp::int32 var_92;
    const wp::int32 var_93 = 4;
    wp::range_t var_94;
    wp::int32 var_95;
    wp::float32 var_96;
    const wp::float32 var_97 = 0.0;
    bool var_98;
    wp::vec_t<3, wp::float32> var_99;
    wp::vec_t<3, wp::float32> var_100;
    wp::vec_t<3, wp::float32> var_101;
    wp::int32 var_102;
    wp::int32 var_103;
    wp::float32 var_104;
    const wp::int32 var_105 = 1;
    wp::int32 var_106;
    wp::int32 var_107;
    const wp::int32 var_108 = 1;
    bool var_109;
    wp::int32 var_110;
    wp::vec_t<4, wp::float32> var_111;
    wp::vec_t<3, wp::float32> var_112;
    wp::float32 var_113;
    wp::float32 var_114;
    wp::float32 var_115;
    const wp::int32 var_116 = 4;
    bool var_117;
    wp::int32 var_118;
    wp::vec_t<4, wp::float32> var_119;
    wp::vec_t<3, wp::float32> var_120;
    wp::float32 var_121;
    wp::float32 var_122;
    GJKResult_8184e255 var_123;
    const bool var_124 = false;
    const wp::int32 var_125 = 0;
    bool var_126;
    wp::vec_t<3, wp::float32> var_127;
    wp::vec_t<3, wp::float32> var_128;
    const wp::int32 var_129 = 0;
    bool var_130;
    wp::vec_t<3, wp::float32> var_131;
    wp::vec_t<3, wp::float32> var_132;
    const wp::float32 var_133 = 0.0;
    bool var_134;
    wp::vec_t<3, wp::float32> var_135;
    wp::vec_t<3, wp::float32> var_136;
    SupportPoint_e82efc60 var_137;
    SupportPoint_e82efc60 var_138;
    wp::vec_t<3, wp::float32>* var_139;
    wp::vec_t<3, wp::float32>* var_140;
    wp::vec_t<3, wp::float32> var_141;
    wp::vec_t<3, wp::float32> var_142;
    wp::vec_t<3, wp::float32> var_143;
    wp::float32 var_144;
    const wp::float32 var_145 = 0.0;
    bool var_146;
    SupportPoint_e82efc60 var_147;
    SupportPoint_e82efc60 var_148;
    bool var_149;
    const wp::int32 var_150 = 4;
    bool var_151;
    bool* var_152;
    bool var_153;
    bool var_154;
    const wp::float32 var_155 = 0.0;
    wp::float32 var_156;
    wp::int32* var_157;
    wp::int32 var_158;
    wp::int32 var_159;
    wp::int32* var_160;
    wp::int32 var_161;
    wp::int32 var_162;
    //---------
    // forward
    // def gjk(                                                                               <L 657>
    // simplex = mat43()                                                                      <L 671>
    var_0 = wp::mat_t<4, 3, wp::float32>();
    // simplex1 = mat43()                                                                     <L 672>
    var_1 = wp::mat_t<4, 3, wp::float32>();
    // simplex2 = mat43()                                                                     <L 673>
    var_2 = wp::mat_t<4, 3, wp::float32>();
    // simplex_index1 = wp.vec4i()                                                            <L 674>
    var_3 = wp::vec_t<4, wp::int32>();
    // simplex_index2 = wp.vec4i()                                                            <L 675>
    var_4 = wp::vec_t<4, wp::int32>();
    // n = int(0)                                                                             <L 676>
    var_6 = wp::int(var_5);
    // lmbda = wp.vec4(1.0, 0.0, 0.0, 0.0)  # barycentric coordinates                         <L 677>
    var_11 = wp::vec_t<4, wp::float32>(var_7, var_8, var_9, var_10);
    // epsilon = wp.where(is_discrete, 0.0, 0.5 * tolerance * tolerance)                      <L 682>
    var_14 = wp::mul(var_13, var_tolerance);
    var_15 = wp::mul(var_14, var_tolerance);
    var_16 = wp::where(var_is_discrete, var_12, var_15);
    // min_norm = wp.where(is_discrete, MINVAL, tolerance)                                    <L 683>
    var_18 = wp::where(var_is_discrete, var_17, var_tolerance);
    // x_k = x1_0 - x2_0                                                                      <L 686>
    var_19 = wp::sub(var_x1_0, var_x2_0);
    // xnorm = wp.sqrt(wp.dot(x_k, x_k))                                                      <L 687>
    var_20 = wp::dot(var_19, var_19);
    var_21 = wp::sqrt(var_20);
    // xnorm_prev = float(0.0)                                                                <L 688>
    var_23 = wp::float(var_22);
    // for _ in range(gjk_iterations):                                                        <L 690>
    var_24 = wp::range(var_gjk_iterations);
    start_for_0:;
        if (iter_cmp(var_24) == 0) goto end_for_0;
        var_25 = wp::iter_next(var_24);
        // if xnorm < min_norm or wp.abs(xnorm_prev - xnorm) < MINVAL:                        <L 691>
        var_27 = (var_21 < var_18);
        var_26 = var_27;
        if (!var_26) {
            var_28 = wp::sub(var_23, var_21);
            var_29 = wp::abs(var_28);
            var_30 = (var_29 < var_17);
            var_26 = var_26 || var_30;
        }
        if (var_26) {
            // break                                                                          <L 692>
            goto end_for_0;
        }
        // sp1, sp2 = _gjk_support(geom1, geom2, geomtype1, geomtype2, x_k, xnorm, simplex, n, is_discrete)       <L 695>
        _gjk_support_0(var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_19, var_21, var_0, var_6, var_is_discrete, var_31, var_32);
        // simplex1[n] = sp1.point                                                            <L 696>
        var_33 = &((var_31).point);
        var_34 = wp::load(var_33);
        wp::assign_inplace(var_1, var_6, var_34);
        // geom1.index = sp1.cached_index                                                     <L 697>
        var_35 = &((var_31).cached_index);
        var_37 = wp::load(var_35);
        var_36 = wp::copy(var_37);
        var_geom1.index = var_36;
        // simplex_index1[n] = sp1.vertex_index                                               <L 698>
        var_38 = &((var_31).vertex_index);
        var_39 = wp::load(var_38);
        wp::assign_inplace(var_3, var_6, var_39);
        // simplex2[n] = sp2.point                                                            <L 700>
        var_40 = &((var_32).point);
        var_41 = wp::load(var_40);
        wp::assign_inplace(var_2, var_6, var_41);
        // geom2.index = sp2.cached_index                                                     <L 701>
        var_42 = &((var_32).cached_index);
        var_44 = wp::load(var_42);
        var_43 = wp::copy(var_44);
        var_geom2.index = var_43;
        // simplex_index2[n] = sp2.vertex_index                                               <L 702>
        var_45 = &((var_32).vertex_index);
        var_46 = wp::load(var_45);
        wp::assign_inplace(var_4, var_6, var_46);
        // simplex[n] = simplex1[n] - simplex2[n]                                             <L 705>
        var_47 = wp::extract(var_1, var_6);
        var_48 = wp::extract(var_2, var_6);
        var_49 = wp::sub(var_47, var_48);
        wp::assign_inplace(var_0, var_6, var_49);
        // if wp.dot(x_k, x_k - simplex[n]) < epsilon:                                        <L 709>
        var_50 = wp::extract(var_0, var_6);
        var_51 = wp::sub(var_19, var_50);
        var_52 = wp::dot(var_19, var_51);
        var_53 = (var_52 < var_16);
        if (var_53) {
            // break                                                                          <L 710>
            goto end_for_0;
        }
        // lower = wp.dot(x_k, simplex[n])                                                    <L 714>
        var_54 = wp::extract(var_0, var_6);
        var_55 = wp::dot(var_19, var_54);
        // if cutoff == 0.0:                                                                  <L 715>
        var_57 = (var_cutoff == var_56);
        if (var_57) {
            // if lower > 0.0:                                                                <L 716>
            var_59 = (var_55 > var_58);
            if (var_59) {
                // result = GJKResult()                                                       <L 717>
                var_60 = GJKResult_8184e255();
                // result.separated = True                                                    <L 718>
                var_60.separated = var_61;
                // result.dim = 0                                                             <L 719>
                var_60.dim = var_62;
                // result.dist = FLOAT_MAX                                                    <L 720>
                var_60.dist = var_63;
                // result.index1 = geom1.index                                                <L 721>
                var_64 = &((var_geom1).index);
                var_66 = wp::load(var_64);
                var_65 = wp::copy(var_66);
                var_60.index1 = var_65;
                // result.index2 = geom2.index                                                <L 722>
                var_67 = &((var_geom2).index);
                var_69 = wp::load(var_67);
                var_68 = wp::copy(var_69);
                var_60.index2 = var_68;
                // return result                                                              <L 723>
                return var_60;
            }
        }
        if (!var_57) {
            // elif cutoff < FLOAT_MAX:                                                       <L 724>
            var_70 = (var_cutoff < var_63);
            if (var_70) {
                // if lower > 0.0 and lower >= cutoff * xnorm:                                <L 725>
                var_73 = (var_55 > var_72);
                var_71 = var_73;
                if (var_71) {
                    var_74 = wp::mul(var_cutoff, var_21);
                    var_75 = (var_55 >= var_74);
                    var_71 = var_71 && var_75;
                }
                if (var_71) {
                    // result = GJKResult()                                                   <L 726>
                    var_76 = GJKResult_8184e255();
                    // result.separated = True                                                <L 727>
                    var_76.separated = var_77;
                    // result.dim = 0                                                         <L 728>
                    var_76.dim = var_78;
                    // result.dist = FLOAT_MAX                                                <L 729>
                    var_76.dist = var_63;
                    // result.index1 = geom1.index                                            <L 730>
                    var_79 = &((var_geom1).index);
                    var_81 = wp::load(var_79);
                    var_80 = wp::copy(var_81);
                    var_76.index1 = var_80;
                    // result.index2 = geom2.index                                            <L 731>
                    var_82 = &((var_geom2).index);
                    var_84 = wp::load(var_82);
                    var_83 = wp::copy(var_84);
                    var_76.index2 = var_83;
                    // return result                                                          <L 732>
                    return var_76;
                }
                var_85 = wp::where(var_71, var_76, var_60);
            }
            var_86 = wp::where(var_70, var_85, var_60);
        }
        var_87 = wp::where(var_57, var_60, var_86);
        // lmbda = _subdistance(n + 1, simplex)                                               <L 736>
        var_89 = wp::add(var_6, var_88);
        var_90 = _subdistance_0(var_89, var_0);
        // n = int(0)                                                                         <L 739>
        var_92 = wp::int(var_91);
        // for i in range(4):                                                                 <L 740>
        var_94 = wp::range(var_93);
        start_for_4:;
            if (iter_cmp(var_94) == 0) goto end_for_4;
            var_95 = wp::iter_next(var_94);
            // if lmbda[i] == 0.0:                                                            <L 741>
            var_96 = wp::extract(var_90, var_95);
            var_98 = (var_96 == var_97);
            if (var_98) {
                // continue                                                                   <L 742>
                goto start_for_4;
            }
            // simplex[n] = simplex[i]                                                        <L 744>
            var_99 = wp::extract(var_0, var_95);
            wp::assign_inplace(var_0, var_92, var_99);
            // simplex1[n] = simplex1[i]                                                      <L 745>
            var_100 = wp::extract(var_1, var_95);
            wp::assign_inplace(var_1, var_92, var_100);
            // simplex2[n] = simplex2[i]                                                      <L 746>
            var_101 = wp::extract(var_2, var_95);
            wp::assign_inplace(var_2, var_92, var_101);
            // simplex_index1[n] = simplex_index1[i]                                          <L 747>
            var_102 = wp::extract(var_3, var_95);
            wp::assign_inplace(var_3, var_92, var_102);
            // simplex_index2[n] = simplex_index2[i]                                          <L 748>
            var_103 = wp::extract(var_4, var_95);
            wp::assign_inplace(var_4, var_92, var_103);
            // lmbda[n] = lmbda[i]                                                            <L 749>
            var_104 = wp::extract(var_90, var_95);
            wp::assign_inplace(var_90, var_92, var_104);
            // n += int(1)                                                                    <L 750>
            var_106 = wp::int(var_105);
            var_107 = wp::add(var_92, var_106);
            wp::assign(var_92, var_107);
            goto start_for_4;
        end_for_4:;
        // if n < 1:                                                                          <L 753>
        var_109 = (var_92 < var_108);
        if (var_109) {
            // break                                                                          <L 754>
            wp::assign(var_6, var_92);
            wp::assign(var_11, var_90);
            goto end_for_0;
        }
        var_110 = wp::where(var_109, var_6, var_92);
        var_111 = wp::where(var_109, var_11, var_90);
        // x_k = _linear_combine(n, lmbda, simplex)                                           <L 757>
        var_112 = _linear_combine_0(var_110, var_111, var_0);
        // xnorm_prev = xnorm                                                                 <L 758>
        var_113 = wp::copy(var_21);
        // xnorm = wp.sqrt(wp.dot(x_k, x_k))                                                  <L 759>
        var_114 = wp::dot(var_112, var_112);
        var_115 = wp::sqrt(var_114);
        // if n == 4:                                                                         <L 762>
        var_117 = (var_110 == var_116);
        if (var_117) {
            // break                                                                          <L 763>
            wp::assign(var_6, var_110);
            wp::assign(var_11, var_111);
            wp::assign(var_19, var_112);
            wp::assign(var_21, var_115);
            wp::assign(var_23, var_113);
            goto end_for_0;
        }
        var_118 = wp::where(var_117, var_6, var_110);
        var_119 = wp::where(var_117, var_11, var_111);
        var_120 = wp::where(var_117, var_19, var_112);
        var_121 = wp::where(var_117, var_21, var_115);
        var_122 = wp::where(var_117, var_23, var_113);
        wp::assign(var_6, var_118);
        wp::assign(var_11, var_119);
        wp::assign(var_19, var_120);
        wp::assign(var_21, var_121);
        wp::assign(var_23, var_122);
        goto start_for_0;
    end_for_0:;
    // result = GJKResult()                                                                   <L 765>
    var_123 = GJKResult_8184e255();
    // result.separated = False                                                               <L 766>
    var_123.separated = var_124;
    // result.x1 = wp.where(n == 0, x1_0, _linear_combine(n, lmbda, simplex1))                <L 771>
    var_126 = (var_6 == var_125);
    var_127 = _linear_combine_0(var_6, var_11, var_1);
    var_128 = wp::where(var_126, var_x1_0, var_127);
    var_123.x1 = var_128;
    // result.x2 = wp.where(n == 0, x2_0, _linear_combine(n, lmbda, simplex2))                <L 772>
    var_130 = (var_6 == var_129);
    var_131 = _linear_combine_0(var_6, var_11, var_2);
    var_132 = wp::where(var_130, var_x2_0, var_131);
    var_123.x2 = var_132;
    // if xnorm > 0.0:                                                                        <L 774>
    var_134 = (var_21 > var_133);
    if (var_134) {
        // dir = x_k / xnorm                                                                  <L 775>
        var_135 = wp::div(var_19, var_21);
        // sp1 = support(geom1, geomtype1, -dir)                                              <L 776>
        var_136 = wp::neg(var_135);
        var_137 = support_0(var_geom1, var_geomtype1, var_136);
        // sp2 = support(geom2, geomtype2, dir)                                               <L 777>
        var_138 = support_0(var_geom2, var_geomtype2, var_135);
        // result.separated = wp.dot(x_k, sp1.point - sp2.point) > 0.0                        <L 778>
        var_139 = &((var_137).point);
        var_140 = &((var_138).point);
        var_142 = wp::load(var_139);
        var_143 = wp::load(var_140);
        var_141 = wp::sub(var_142, var_143);
        var_144 = wp::dot(var_19, var_141);
        var_146 = (var_144 > var_145);
        var_123.separated = var_146;
    }
    var_147 = wp::where(var_134, var_137, var_31);
    var_148 = wp::where(var_134, var_138, var_32);
    // result.dist = wp.where(n == 4 and not result.separated, 0.0, xnorm)                    <L 781>
    var_151 = (var_6 == var_150);
    var_149 = var_151;
    if (var_149) {
        var_152 = &((var_123).separated);
        var_154 = wp::load(var_152);
        var_153 = wp::unot(var_154);
        var_149 = var_149 && var_153;
    }
    var_156 = wp::where(var_149, var_155, var_21);
    var_123.dist = var_156;
    // result.dim = n                                                                         <L 783>
    var_123.dim = var_6;
    // result.simplex1 = simplex1                                                             <L 784>
    var_123.simplex1 = var_1;
    // result.simplex2 = simplex2                                                             <L 785>
    var_123.simplex2 = var_2;
    // result.simplex_index1 = simplex_index1                                                 <L 786>
    var_123.simplex_index1 = var_3;
    // result.simplex_index2 = simplex_index2                                                 <L 787>
    var_123.simplex_index2 = var_4;
    // result.simplex = simplex                                                               <L 788>
    var_123.simplex = var_0;
    // result.index1 = geom1.index                                                            <L 789>
    var_157 = &((var_geom1).index);
    var_159 = wp::load(var_157);
    var_158 = wp::copy(var_159);
    var_123.index1 = var_158;
    // result.index2 = geom2.index                                                            <L 790>
    var_160 = &((var_geom2).index);
    var_162 = wp::load(var_160);
    var_161 = wp::copy(var_162);
    var_123.index2 = var_161;
    // return result                                                                          <L 791>
    return var_123;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:807
static CUDA_CALLABLE wp::vec_t<3, wp::float32> _tri_affine_coord_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_p)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::float32 var_1;
    const wp::int32 var_2 = 2;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::int32 var_5 = 2;
    wp::float32 var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 1;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 2;
    wp::float32 var_18;
    const wp::int32 var_19 = 1;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 1;
    wp::float32 var_24;
    const wp::int32 var_25 = 2;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    const wp::int32 var_29 = 2;
    wp::float32 var_30;
    const wp::int32 var_31 = 1;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    const wp::int32 var_35 = 0;
    wp::float32 var_36;
    const wp::int32 var_37 = 2;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    const wp::int32 var_42 = 0;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    const wp::int32 var_46 = 0;
    wp::float32 var_47;
    const wp::int32 var_48 = 2;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    const wp::int32 var_52 = 2;
    wp::float32 var_53;
    const wp::int32 var_54 = 0;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::float32 var_57;
    const wp::int32 var_58 = 0;
    wp::float32 var_59;
    const wp::int32 var_60 = 2;
    wp::float32 var_61;
    wp::float32 var_62;
    wp::float32 var_63;
    const wp::int32 var_64 = 2;
    wp::float32 var_65;
    const wp::int32 var_66 = 0;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::float32 var_69;
    const wp::int32 var_70 = 0;
    wp::float32 var_71;
    const wp::int32 var_72 = 1;
    wp::float32 var_73;
    wp::float32 var_74;
    const wp::int32 var_75 = 1;
    wp::float32 var_76;
    const wp::int32 var_77 = 0;
    wp::float32 var_78;
    wp::float32 var_79;
    wp::float32 var_80;
    const wp::int32 var_81 = 0;
    wp::float32 var_82;
    const wp::int32 var_83 = 1;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::float32 var_86;
    const wp::int32 var_87 = 1;
    wp::float32 var_88;
    const wp::int32 var_89 = 0;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    const wp::int32 var_93 = 0;
    wp::float32 var_94;
    const wp::int32 var_95 = 1;
    wp::float32 var_96;
    wp::float32 var_97;
    wp::float32 var_98;
    const wp::int32 var_99 = 1;
    wp::float32 var_100;
    const wp::int32 var_101 = 0;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::float32 var_104;
    const wp::float32 var_105 = 0.0;
    const wp::int32 var_106 = 0;
    const wp::int32 var_107 = 0;
    wp::float32 var_108;
    wp::float32 var_109;
    wp::float32 var_110;
    bool var_111;
    bool var_112;
    bool var_113;
    wp::float32 var_114;
    const wp::int32 var_115 = 1;
    const wp::int32 var_116 = 2;
    wp::float32 var_117;
    wp::int32 var_118;
    wp::int32 var_119;
    bool var_120;
    wp::float32 var_121;
    const wp::int32 var_122 = 0;
    const wp::int32 var_123 = 2;
    wp::float32 var_124;
    wp::int32 var_125;
    wp::int32 var_126;
    wp::float32 var_127;
    const wp::int32 var_128 = 0;
    const wp::int32 var_129 = 1;
    wp::float32 var_130;
    wp::int32 var_131;
    wp::int32 var_132;
    wp::float32 var_133;
    wp::int32 var_134;
    wp::int32 var_135;
    wp::float32 var_136;
    wp::float32 var_137;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::float32 var_140;
    wp::float32 var_141;
    wp::float32 var_142;
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
    wp::float32 var_157;
    wp::float32 var_158;
    wp::float32 var_159;
    wp::float32 var_160;
    wp::float32 var_161;
    wp::float32 var_162;
    wp::float32 var_163;
    wp::float32 var_164;
    wp::float32 var_165;
    wp::float32 var_166;
    wp::float32 var_167;
    wp::float32 var_168;
    wp::float32 var_169;
    wp::float32 var_170;
    wp::float32 var_171;
    wp::float32 var_172;
    wp::float32 var_173;
    wp::float32 var_174;
    wp::float32 var_175;
    wp::float32 var_176;
    wp::float32 var_177;
    wp::float32 var_178;
    wp::float32 var_179;
    wp::float32 var_180;
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
    wp::vec_t<3, wp::float32> var_208;
    //---------
    // forward
    // def _tri_affine_coord(v1: wp.vec3, v2: wp.vec3, v3: wp.vec3, p: wp.vec3) -> wp.vec3:       <L 808>
    // M_14 = v2[1] * v3[2] - v2[2] * v3[1] - v1[1] * v3[2] + v1[2] * v3[1] + v1[1] * v2[2] - v1[2] * v2[1]       <L 810>
    var_1 = wp::extract(var_v2, var_0);
    var_3 = wp::extract(var_v3, var_2);
    var_4 = wp::mul(var_1, var_3);
    var_6 = wp::extract(var_v2, var_5);
    var_8 = wp::extract(var_v3, var_7);
    var_9 = wp::mul(var_6, var_8);
    var_10 = wp::sub(var_4, var_9);
    var_12 = wp::extract(var_v1, var_11);
    var_14 = wp::extract(var_v3, var_13);
    var_15 = wp::mul(var_12, var_14);
    var_16 = wp::sub(var_10, var_15);
    var_18 = wp::extract(var_v1, var_17);
    var_20 = wp::extract(var_v3, var_19);
    var_21 = wp::mul(var_18, var_20);
    var_22 = wp::add(var_16, var_21);
    var_24 = wp::extract(var_v1, var_23);
    var_26 = wp::extract(var_v2, var_25);
    var_27 = wp::mul(var_24, var_26);
    var_28 = wp::add(var_22, var_27);
    var_30 = wp::extract(var_v1, var_29);
    var_32 = wp::extract(var_v2, var_31);
    var_33 = wp::mul(var_30, var_32);
    var_34 = wp::sub(var_28, var_33);
    // M_24 = v2[0] * v3[2] - v2[2] * v3[0] - v1[0] * v3[2] + v1[2] * v3[0] + v1[0] * v2[2] - v1[2] * v2[0]       <L 811>
    var_36 = wp::extract(var_v2, var_35);
    var_38 = wp::extract(var_v3, var_37);
    var_39 = wp::mul(var_36, var_38);
    var_41 = wp::extract(var_v2, var_40);
    var_43 = wp::extract(var_v3, var_42);
    var_44 = wp::mul(var_41, var_43);
    var_45 = wp::sub(var_39, var_44);
    var_47 = wp::extract(var_v1, var_46);
    var_49 = wp::extract(var_v3, var_48);
    var_50 = wp::mul(var_47, var_49);
    var_51 = wp::sub(var_45, var_50);
    var_53 = wp::extract(var_v1, var_52);
    var_55 = wp::extract(var_v3, var_54);
    var_56 = wp::mul(var_53, var_55);
    var_57 = wp::add(var_51, var_56);
    var_59 = wp::extract(var_v1, var_58);
    var_61 = wp::extract(var_v2, var_60);
    var_62 = wp::mul(var_59, var_61);
    var_63 = wp::add(var_57, var_62);
    var_65 = wp::extract(var_v1, var_64);
    var_67 = wp::extract(var_v2, var_66);
    var_68 = wp::mul(var_65, var_67);
    var_69 = wp::sub(var_63, var_68);
    // M_34 = v2[0] * v3[1] - v2[1] * v3[0] - v1[0] * v3[1] + v1[1] * v3[0] + v1[0] * v2[1] - v1[1] * v2[0]       <L 812>
    var_71 = wp::extract(var_v2, var_70);
    var_73 = wp::extract(var_v3, var_72);
    var_74 = wp::mul(var_71, var_73);
    var_76 = wp::extract(var_v2, var_75);
    var_78 = wp::extract(var_v3, var_77);
    var_79 = wp::mul(var_76, var_78);
    var_80 = wp::sub(var_74, var_79);
    var_82 = wp::extract(var_v1, var_81);
    var_84 = wp::extract(var_v3, var_83);
    var_85 = wp::mul(var_82, var_84);
    var_86 = wp::sub(var_80, var_85);
    var_88 = wp::extract(var_v1, var_87);
    var_90 = wp::extract(var_v3, var_89);
    var_91 = wp::mul(var_88, var_90);
    var_92 = wp::add(var_86, var_91);
    var_94 = wp::extract(var_v1, var_93);
    var_96 = wp::extract(var_v2, var_95);
    var_97 = wp::mul(var_94, var_96);
    var_98 = wp::add(var_92, var_97);
    var_100 = wp::extract(var_v1, var_99);
    var_102 = wp::extract(var_v2, var_101);
    var_103 = wp::mul(var_100, var_102);
    var_104 = wp::sub(var_98, var_103);
    // M_max = 0.0                                                                            <L 816>
    // x = 0                                                                                  <L 817>
    // y = 0                                                                                  <L 818>
    // mu1 = wp.abs(M_14)                                                                     <L 820>
    var_108 = wp::abs(var_34);
    // mu2 = wp.abs(M_24)                                                                     <L 821>
    var_109 = wp::abs(var_69);
    // mu3 = wp.abs(M_34)                                                                     <L 822>
    var_110 = wp::abs(var_104);
    // if mu1 >= mu2 and mu1 >= mu3:                                                          <L 824>
    var_112 = (var_108 >= var_109);
    var_111 = var_112;
    if (var_111) {
        var_113 = (var_108 >= var_110);
        var_111 = var_111 && var_113;
    }
    if (var_111) {
        // M_max = M_14                                                                       <L 825>
        var_114 = wp::copy(var_34);
        // x = 1                                                                              <L 826>
        // y = 2                                                                              <L 827>
    }
    var_117 = wp::where(var_111, var_114, var_105);
    var_118 = wp::where(var_111, var_115, var_106);
    var_119 = wp::where(var_111, var_116, var_107);
    if (!var_111) {
        // elif mu2 >= mu3:                                                                   <L 828>
        var_120 = (var_109 >= var_110);
        if (var_120) {
            // M_max = M_24                                                                   <L 829>
            var_121 = wp::copy(var_69);
            // x = 0                                                                          <L 830>
            // y = 2                                                                          <L 831>
        }
        var_124 = wp::where(var_120, var_121, var_117);
        var_125 = wp::where(var_120, var_122, var_118);
        var_126 = wp::where(var_120, var_123, var_119);
        if (!var_120) {
            // M_max = M_34                                                                   <L 833>
            var_127 = wp::copy(var_104);
            // x = 0                                                                          <L 834>
            // y = 1                                                                          <L 835>
        }
        var_130 = wp::where(var_120, var_124, var_127);
        var_131 = wp::where(var_120, var_125, var_128);
        var_132 = wp::where(var_120, var_126, var_129);
    }
    var_133 = wp::where(var_111, var_117, var_130);
    var_134 = wp::where(var_111, var_118, var_131);
    var_135 = wp::where(var_111, var_119, var_132);
    // C31 = p[x] * v2[y] + p[y] * v3[x] + v2[x] * v3[y] - p[x] * v3[y] - p[y] * v2[x] - v3[x] * v2[y]       <L 838>
    var_136 = wp::extract(var_p, var_134);
    var_137 = wp::extract(var_v2, var_135);
    var_138 = wp::mul(var_136, var_137);
    var_139 = wp::extract(var_p, var_135);
    var_140 = wp::extract(var_v3, var_134);
    var_141 = wp::mul(var_139, var_140);
    var_142 = wp::add(var_138, var_141);
    var_143 = wp::extract(var_v2, var_134);
    var_144 = wp::extract(var_v3, var_135);
    var_145 = wp::mul(var_143, var_144);
    var_146 = wp::add(var_142, var_145);
    var_147 = wp::extract(var_p, var_134);
    var_148 = wp::extract(var_v3, var_135);
    var_149 = wp::mul(var_147, var_148);
    var_150 = wp::sub(var_146, var_149);
    var_151 = wp::extract(var_p, var_135);
    var_152 = wp::extract(var_v2, var_134);
    var_153 = wp::mul(var_151, var_152);
    var_154 = wp::sub(var_150, var_153);
    var_155 = wp::extract(var_v3, var_134);
    var_156 = wp::extract(var_v2, var_135);
    var_157 = wp::mul(var_155, var_156);
    var_158 = wp::sub(var_154, var_157);
    // C32 = p[x] * v3[y] + p[y] * v1[x] + v3[x] * v1[y] - p[x] * v1[y] - p[y] * v3[x] - v1[x] * v3[y]       <L 841>
    var_159 = wp::extract(var_p, var_134);
    var_160 = wp::extract(var_v3, var_135);
    var_161 = wp::mul(var_159, var_160);
    var_162 = wp::extract(var_p, var_135);
    var_163 = wp::extract(var_v1, var_134);
    var_164 = wp::mul(var_162, var_163);
    var_165 = wp::add(var_161, var_164);
    var_166 = wp::extract(var_v3, var_134);
    var_167 = wp::extract(var_v1, var_135);
    var_168 = wp::mul(var_166, var_167);
    var_169 = wp::add(var_165, var_168);
    var_170 = wp::extract(var_p, var_134);
    var_171 = wp::extract(var_v1, var_135);
    var_172 = wp::mul(var_170, var_171);
    var_173 = wp::sub(var_169, var_172);
    var_174 = wp::extract(var_p, var_135);
    var_175 = wp::extract(var_v3, var_134);
    var_176 = wp::mul(var_174, var_175);
    var_177 = wp::sub(var_173, var_176);
    var_178 = wp::extract(var_v1, var_134);
    var_179 = wp::extract(var_v3, var_135);
    var_180 = wp::mul(var_178, var_179);
    var_181 = wp::sub(var_177, var_180);
    // C33 = p[x] * v1[y] + p[y] * v2[x] + v1[x] * v2[y] - p[x] * v2[y] - p[y] * v1[x] - v2[x] * v1[y]       <L 844>
    var_182 = wp::extract(var_p, var_134);
    var_183 = wp::extract(var_v1, var_135);
    var_184 = wp::mul(var_182, var_183);
    var_185 = wp::extract(var_p, var_135);
    var_186 = wp::extract(var_v2, var_134);
    var_187 = wp::mul(var_185, var_186);
    var_188 = wp::add(var_184, var_187);
    var_189 = wp::extract(var_v1, var_134);
    var_190 = wp::extract(var_v2, var_135);
    var_191 = wp::mul(var_189, var_190);
    var_192 = wp::add(var_188, var_191);
    var_193 = wp::extract(var_p, var_134);
    var_194 = wp::extract(var_v2, var_135);
    var_195 = wp::mul(var_193, var_194);
    var_196 = wp::sub(var_192, var_195);
    var_197 = wp::extract(var_p, var_135);
    var_198 = wp::extract(var_v1, var_134);
    var_199 = wp::mul(var_197, var_198);
    var_200 = wp::sub(var_196, var_199);
    var_201 = wp::extract(var_v2, var_134);
    var_202 = wp::extract(var_v1, var_135);
    var_203 = wp::mul(var_201, var_202);
    var_204 = wp::sub(var_200, var_203);
    // return wp.vec3(C31 / M_max, C32 / M_max, C33 / M_max)                                  <L 847>
    var_205 = wp::div(var_158, var_133);
    var_206 = wp::div(var_181, var_133);
    var_207 = wp::div(var_204, var_133);
    var_208 = wp::vec_t<3, wp::float32>(var_205, var_206, var_207);
    return var_208;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2454
static CUDA_CALLABLE void _inflate_0(
    GJKResult_8184e255 var_result,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::float32 var_margin1,
    wp::float32 var_margin2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    wp::float32* var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::vec_t<3, wp::float32>* var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::vec_t<3, wp::float32>* var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    const wp::int32 var_9 = 1;
    bool var_10;
    wp::vec_t<4, wp::int32>* var_11;
    const wp::int32 var_12 = 0;
    wp::int32 var_13;
    wp::vec_t<4, wp::int32> var_14;
    const bool var_15 = false;
    bool var_16;
    wp::int32* var_17;
    wp::range_t var_18;
    wp::int32 var_19;
    wp::int32 var_20;
    wp::vec_t<4, wp::int32>* var_21;
    wp::int32 var_22;
    wp::vec_t<4, wp::int32> var_23;
    bool var_24;
    const bool var_25 = true;
    const wp::float32 var_26 = 0.0;
    const wp::float32 var_27 = 0.0;
    const wp::float32 var_28 = 1.0;
    wp::vec_t<3, wp::float32> var_29;
    SupportPoint_e82efc60 var_30;
    wp::vec_t<3, wp::float32>* var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::mat_t<6, 3, wp::float32>* var_35;
    const wp::int32 var_36 = 3;
    wp::vec_t<3, wp::float32> var_37;
    wp::mat_t<6, 3, wp::float32> var_38;
    wp::mat_t<6, 3, wp::float32>* var_39;
    const wp::int32 var_40 = 4;
    wp::vec_t<3, wp::float32> var_41;
    wp::mat_t<6, 3, wp::float32> var_42;
    wp::mat_t<6, 3, wp::float32>* var_43;
    const wp::int32 var_44 = 5;
    wp::vec_t<3, wp::float32> var_45;
    wp::mat_t<6, 3, wp::float32> var_46;
    wp::vec_t<3, wp::float32> var_47;
    bool var_48;
    const wp::int32 var_49 = 0;
    wp::float32 var_50;
    const wp::float32 var_51 = 0.0;
    bool var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    const wp::float32 var_55 = 0.0;
    bool var_56;
    const wp::int32 var_57 = 2;
    wp::float32 var_58;
    const wp::float32 var_59 = 0.0;
    bool var_60;
    const wp::int32 var_61 = 0;
    wp::float32 var_62;
    wp::vec_t<3, wp::float32> var_63;
    const wp::int32 var_64 = 1;
    wp::float32 var_65;
    wp::vec_t<3, wp::float32> var_66;
    wp::vec_t<3, wp::float32> var_67;
    const wp::int32 var_68 = 2;
    wp::float32 var_69;
    wp::vec_t<3, wp::float32> var_70;
    wp::vec_t<3, wp::float32> var_71;
    wp::vec_t<3, wp::float32> var_72;
    wp::vec_t<3, wp::float32> var_73;
    const wp::int32 var_74 = 1;
    wp::float32 var_75;
    const wp::float32 var_76 = 0.0;
    bool var_77;
    wp::vec_t<3, wp::float32> var_78;
    const wp::int32 var_79 = 0;
    wp::float32 var_80;
    const wp::float32 var_81 = 0.0;
    bool var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::vec_t<3, wp::float32> var_84;
    wp::float32 var_85;
    wp::vec_t<3, wp::float32> var_86;
    wp::vec_t<3, wp::float32> var_87;
    wp::vec_t<3, wp::float32> var_88;
    wp::vec_t<3, wp::float32> var_89;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::vec_t<3, wp::float32> var_93;
    wp::vec_t<3, wp::float32> var_94;
    wp::float32 var_95;
    wp::vec_t<3, wp::float32> var_96;
    wp::vec_t<3, wp::float32> var_97;
    wp::vec_t<3, wp::float32> var_98;
    wp::vec_t<3, wp::float32> var_99;
    const wp::float32 var_100 = 0.0;
    bool var_101;
    wp::vec_t<3, wp::float32> var_102;
    wp::vec_t<3, wp::float32> var_103;
    wp::vec_t<3, wp::float32> var_104;
    const wp::float32 var_105 = 0.0;
    bool var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::float32 var_110;
    wp::float32 var_111;
    //---------
    // forward
    // def _inflate(                                                                          <L 2455>
    // dist = result.dist                                                                     <L 2458>
    var_0 = &((var_result).dist);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // x1 = result.x1                                                                         <L 2459>
    var_3 = &((var_result).x1);
    var_5 = wp::load(var_3);
    var_4 = wp::copy(var_5);
    // x2 = result.x2                                                                         <L 2460>
    var_6 = &((var_result).x2);
    var_8 = wp::load(var_6);
    var_7 = wp::copy(var_8);
    // if geomtype1 == GeomType.HFIELD:                                                       <L 2462>
    var_10 = (var_geomtype1 == var_9);
    if (var_10) {
        // v = result.simplex_index1[0]                                                       <L 2463>
        var_11 = &((var_result).simplex_index1);
        var_14 = wp::load(var_11);
        var_13 = wp::extract(var_14, var_12);
        // is_side = bool(False)                                                              <L 2464>
        var_16 = bool(var_15);
        // for i in range(result.dim):                                                        <L 2465>
        var_17 = &((var_result).dim);
        var_19 = wp::load(var_17);
        var_18 = wp::range(var_19);
        start_for_0:;
            if (iter_cmp(var_18) == 0) goto end_for_0;
            var_20 = wp::iter_next(var_18);
            // if result.simplex_index1[i] != v:                                              <L 2466>
            var_21 = &((var_result).simplex_index1);
            var_23 = wp::load(var_21);
            var_22 = wp::extract(var_23, var_20);
            var_24 = (var_22 != var_13);
            if (var_24) {
                // is_side = True                                                             <L 2467>
                // break                                                                      <L 2468>
                wp::assign(var_16, var_25);
                goto end_for_0;
            }
            goto start_for_0;
        end_for_0:;
        // if is_side:                                                                        <L 2470>
        if (var_16) {
            // n = wp.vec3(0.0, 0.0, 1.0)                                                     <L 2471>
            var_29 = wp::vec_t<3, wp::float32>(var_26, var_27, var_28);
            // sp = support(geom2, geomtype2, x2)                                             <L 2472>
            var_30 = support_0(var_geom2, var_geomtype2, var_7);
            // x2 = sp.point - margin2 * n                                                    <L 2473>
            var_31 = &((var_30).point);
            var_32 = wp::mul(var_margin2, var_29);
            var_34 = wp::load(var_31);
            var_33 = wp::sub(var_34, var_32);
            // a = geom1.polyvert[3]                                                          <L 2476>
            var_35 = &((var_geom1).polyvert);
            var_38 = wp::load(var_35);
            var_37 = wp::extract(var_38, var_36);
            // b = geom1.polyvert[4]                                                          <L 2477>
            var_39 = &((var_geom1).polyvert);
            var_42 = wp::load(var_39);
            var_41 = wp::extract(var_42, var_40);
            // c = geom1.polyvert[5]                                                          <L 2478>
            var_43 = &((var_geom1).polyvert);
            var_46 = wp::load(var_43);
            var_45 = wp::extract(var_46, var_44);
            // coordinates = _tri_affine_coord(a, b, c, x2)                                   <L 2480>
            var_47 = _tri_affine_coord_0(var_37, var_41, var_45, var_33);
            // if coordinates[0] > 0.0 and coordinates[1] > 0.0 and coordinates[2] > 0.0:       <L 2481>
            var_50 = wp::extract(var_47, var_49);
            var_52 = (var_50 > var_51);
            var_48 = var_52;
            if (var_48) {
                var_54 = wp::extract(var_47, var_53);
                var_56 = (var_54 > var_55);
                var_48 = var_48 && var_56;
            }
            if (var_48) {
                var_58 = wp::extract(var_47, var_57);
                var_60 = (var_58 > var_59);
                var_48 = var_48 && var_60;
            }
            if (var_48) {
                // x1 = coordinates[0] * a + coordinates[1] * b + coordinates[2] * c          <L 2482>
                var_62 = wp::extract(var_47, var_61);
                var_63 = wp::mul(var_62, var_37);
                var_65 = wp::extract(var_47, var_64);
                var_66 = wp::mul(var_65, var_41);
                var_67 = wp::add(var_63, var_66);
                var_69 = wp::extract(var_47, var_68);
                var_70 = wp::mul(var_69, var_45);
                var_71 = wp::add(var_67, var_70);
            }
            var_72 = wp::where(var_48, var_71, var_4);
            if (!var_48) {
                // p = c                                                                      <L 2484>
                var_73 = wp::copy(var_45);
                // p = wp.where(coordinates[1] > 0.0, b, p)                                   <L 2485>
                var_75 = wp::extract(var_47, var_74);
                var_77 = (var_75 > var_76);
                var_78 = wp::where(var_77, var_41, var_73);
                // p = wp.where(coordinates[0] > 0.0, a, p)                                   <L 2486>
                var_80 = wp::extract(var_47, var_79);
                var_82 = (var_80 > var_81);
                var_83 = wp::where(var_82, var_37, var_78);
                // x1 = x2 - wp.dot(x2 - p, n) * n                                            <L 2487>
                var_84 = wp::sub(var_33, var_83);
                var_85 = wp::dot(var_84, var_29);
                var_86 = wp::mul(var_85, var_29);
                var_87 = wp::sub(var_33, var_86);
            }
            var_88 = wp::where(var_48, var_72, var_87);
            // dist = -wp.norm_l2(x1 - x2)                                                    <L 2488>
            var_89 = wp::sub(var_88, var_33);
            var_90 = norm_l2_0(var_89);
            var_91 = wp::neg(var_90);
            // return dist, x1, x2                                                            <L 2489>
            ret_0 = var_91;
            ret_1 = var_88;
            ret_2 = var_33;
            return;
        }
        var_92 = wp::where(var_16, var_91, var_1);
        var_93 = wp::where(var_16, var_88, var_4);
        var_94 = wp::where(var_16, var_33, var_7);
    }
    var_95 = wp::where(var_10, var_92, var_1);
    var_96 = wp::where(var_10, var_93, var_4);
    var_97 = wp::where(var_10, var_94, var_7);
    // n = wp.normalize(x2 - x1)                                                              <L 2491>
    var_98 = wp::sub(var_97, var_96);
    var_99 = wp::normalize(var_98);
    // if margin1 > 0.0:                                                                      <L 2492>
    var_101 = (var_margin1 > var_100);
    if (var_101) {
        // x1 += margin1 * n                                                                  <L 2493>
        var_102 = wp::mul(var_margin1, var_99);
        var_103 = wp::add(var_96, var_102);
    }
    var_104 = wp::where(var_101, var_103, var_96);
    // if margin2 > 0.0:                                                                      <L 2495>
    var_106 = (var_margin2 > var_105);
    if (var_106) {
        // x2 -= margin2 * n                                                                  <L 2496>
        var_107 = wp::mul(var_margin2, var_99);
        var_108 = wp::sub(var_97, var_107);
    }
    var_109 = wp::where(var_106, var_108, var_97);
    // dist -= margin1 + margin2                                                              <L 2497>
    var_110 = wp::add(var_margin1, var_margin2);
    var_111 = wp::sub(var_95, var_110);
    // return dist, x1, x2                                                                    <L 2498>
    ret_0 = var_111;
    ret_1 = var_104;
    ret_2 = var_109;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2501
static CUDA_CALLABLE void gjk_phase_0(
    wp::float32 var_tolerance,
    wp::float32 var_cutoff,
    wp::int32 var_gjk_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_1,
    wp::vec_t<3, wp::float32> var_x_2,
    bool & ret_0,
    wp::float32 & ret_1,
    wp::int32 & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::vec_t<3, wp::float32> & ret_4,
    GJKResult_8184e255 & ret_5,
    Geom_4022bfa7 & ret_6,
    Geom_4022bfa7 & ret_7)
{
    //---------
    // primal vars
    wp::float32* var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::float32* var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::vec_t<3, wp::float32>* var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::vec_t<3, wp::float32>* var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    const wp::float32 var_12 = 0.0;
    const wp::float32 var_13 = 0.0;
    const wp::float32 var_14 = 0.0;
    const wp::float32 var_15 = 0.0;
    GJKResult_8184e255 var_16;
    bool var_17;
    bool var_18;
    bool var_19;
    wp::float32* var_20;
    const wp::float32 var_21 = 0.0;
    bool var_22;
    wp::float32 var_23;
    wp::float32* var_24;
    const wp::float32 var_25 = 0.0;
    bool var_26;
    wp::float32 var_27;
    bool var_28;
    const wp::int32 var_29 = 2;
    bool var_30;
    const wp::int32 var_31 = 3;
    bool var_32;
    wp::vec_t<3, wp::float32>* var_33;
    const wp::int32 var_34 = 0;
    wp::float32 var_35;
    wp::vec_t<3, wp::float32> var_36;
    const wp::float32 var_37 = 0.5;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::float32 var_40 = 0.0;
    const wp::float32 var_41 = 0.0;
    wp::vec_t<3, wp::float32>* var_42;
    const wp::int32 var_43 = 1;
    wp::float32 var_44;
    wp::vec_t<3, wp::float32> var_45;
    wp::vec_t<3, wp::float32>* var_46;
    const wp::int32 var_47 = 2;
    wp::float32 var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::vec_t<3, wp::float32> var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    bool var_53;
    const wp::int32 var_54 = 2;
    bool var_55;
    const wp::int32 var_56 = 3;
    bool var_57;
    wp::vec_t<3, wp::float32>* var_58;
    const wp::int32 var_59 = 0;
    wp::float32 var_60;
    wp::vec_t<3, wp::float32> var_61;
    const wp::float32 var_62 = 0.5;
    wp::float32 var_63;
    wp::float32 var_64;
    const wp::float32 var_65 = 0.0;
    const wp::float32 var_66 = 0.0;
    wp::vec_t<3, wp::float32>* var_67;
    const wp::int32 var_68 = 1;
    wp::float32 var_69;
    wp::vec_t<3, wp::float32> var_70;
    wp::vec_t<3, wp::float32>* var_71;
    const wp::int32 var_72 = 2;
    wp::float32 var_73;
    wp::vec_t<3, wp::float32> var_74;
    wp::vec_t<3, wp::float32> var_75;
    wp::float32 var_76;
    wp::float32 var_77;
    wp::float32 var_78;
    const wp::float32 var_79 = 0.0;
    bool var_80;
    wp::float32 var_81;
    wp::float32 var_82;
    GJKResult_8184e255 var_83;
    wp::int32* var_84;
    wp::int32 var_85;
    wp::int32 var_86;
    wp::int32* var_87;
    wp::int32 var_88;
    wp::int32 var_89;
    wp::float32* var_90;
    bool var_91;
    wp::float32 var_92;
    wp::float32* var_93;
    const wp::float32 var_94 = 1e+30;
    bool var_95;
    wp::float32 var_96;
    const bool var_97 = false;
    wp::float32* var_98;
    const wp::int32 var_99 = 1;
    wp::vec_t<3, wp::float32>* var_100;
    wp::vec_t<3, wp::float32>* var_101;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::vec_t<3, wp::float32> var_104;
    wp::vec_t<3, wp::float32> var_105;
    wp::vec_t<3, wp::float32> var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::float32 var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::vec_t<3, wp::float32> var_110;
    const bool var_111 = false;
    const wp::int32 var_112 = 1;
    wp::float32 var_113;
    wp::float32 var_114;
    wp::float32 var_115;
    GJKResult_8184e255 var_116;
    wp::int32* var_117;
    wp::int32 var_118;
    wp::int32 var_119;
    wp::int32* var_120;
    wp::int32 var_121;
    wp::int32 var_122;
    bool var_123;
    wp::float32* var_124;
    bool var_125;
    wp::float32 var_126;
    wp::int32* var_127;
    const wp::int32 var_128 = 2;
    bool var_129;
    wp::int32 var_130;
    bool* var_131;
    bool var_132;
    const bool var_133 = false;
    wp::float32* var_134;
    const wp::int32 var_135 = 1;
    wp::vec_t<3, wp::float32>* var_136;
    wp::vec_t<3, wp::float32>* var_137;
    wp::float32 var_138;
    wp::float32 var_139;
    wp::vec_t<3, wp::float32> var_140;
    wp::vec_t<3, wp::float32> var_141;
    wp::vec_t<3, wp::float32> var_142;
    wp::vec_t<3, wp::float32> var_143;
    const bool var_144 = true;
    wp::float32* var_145;
    const wp::int32 var_146 = 1;
    wp::vec_t<3, wp::float32>* var_147;
    wp::vec_t<3, wp::float32>* var_148;
    wp::float32 var_149;
    wp::float32 var_150;
    wp::vec_t<3, wp::float32> var_151;
    wp::vec_t<3, wp::float32> var_152;
    wp::vec_t<3, wp::float32> var_153;
    wp::vec_t<3, wp::float32> var_154;
    //---------
    // forward
    // def gjk_phase(                                                                         <L 2502>
    // orig_margin1 = geom1.margin                                                            <L 2515>
    var_0 = &((var_geom1).margin);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // orig_margin2 = geom2.margin                                                            <L 2516>
    var_3 = &((var_geom2).margin);
    var_5 = wp::load(var_3);
    var_4 = wp::copy(var_5);
    // orig_size1 = geom1.size                                                                <L 2517>
    var_6 = &((var_geom1).size);
    var_8 = wp::load(var_6);
    var_7 = wp::copy(var_8);
    // orig_size2 = geom2.size                                                                <L 2518>
    var_9 = &((var_geom2).size);
    var_11 = wp::load(var_9);
    var_10 = wp::copy(var_11);
    // full_margin1 = 0.0                                                                     <L 2519>
    // full_margin2 = 0.0                                                                     <L 2520>
    // size1 = 0.0                                                                            <L 2521>
    // size2 = 0.0                                                                            <L 2522>
    // empty = GJKResult()                                                                    <L 2523>
    var_16 = GJKResult_8184e255();
    // is_discrete = _discrete_geoms(geomtype1, geomtype2) and (geom1.margin == 0.0 and geom2.margin == 0.0)       <L 2526>
    var_18 = _discrete_geoms_0(var_geomtype1, var_geomtype2);
    var_17 = var_18;
    if (var_17) {
        var_20 = &((var_geom1).margin);
        var_23 = wp::load(var_20);
        var_22 = (var_23 == var_21);
        var_19 = var_22;
        if (var_19) {
            var_24 = &((var_geom2).margin);
            var_27 = wp::load(var_24);
            var_26 = (var_27 == var_25);
            var_19 = var_19 && var_26;
        }
        var_17 = var_17 && var_19;
    }
    // if geomtype1 == GeomType.SPHERE or geomtype1 == GeomType.CAPSULE:                      <L 2529>
    var_30 = (var_geomtype1 == var_29);
    var_28 = var_30;
    if (!var_28) {
        var_32 = (var_geomtype1 == var_31);
        var_28 = var_28 || var_32;
    }
    if (var_28) {
        // size1 = geom1.size[0]                                                              <L 2530>
        var_33 = &((var_geom1).size);
        var_36 = wp::load(var_33);
        var_35 = wp::extract(var_36, var_34);
        // full_margin1 = size1 + 0.5 * orig_margin1                                          <L 2531>
        var_38 = wp::mul(var_37, var_1);
        var_39 = wp::add(var_35, var_38);
        // geom1.margin = 0.0                                                                 <L 2532>
        var_geom1.margin = var_40;
        // geom1.size = wp.vec3(0.0, geom1.size[1], geom1.size[2])                            <L 2533>
        var_42 = &((var_geom1).size);
        var_45 = wp::load(var_42);
        var_44 = wp::extract(var_45, var_43);
        var_46 = &((var_geom1).size);
        var_49 = wp::load(var_46);
        var_48 = wp::extract(var_49, var_47);
        var_50 = wp::vec_t<3, wp::float32>(var_41, var_44, var_48);
        var_geom1.size = var_50;
    }
    var_51 = wp::where(var_28, var_39, var_12);
    var_52 = wp::where(var_28, var_35, var_14);
    // if geomtype2 == GeomType.SPHERE or geomtype2 == GeomType.CAPSULE:                      <L 2535>
    var_55 = (var_geomtype2 == var_54);
    var_53 = var_55;
    if (!var_53) {
        var_57 = (var_geomtype2 == var_56);
        var_53 = var_53 || var_57;
    }
    if (var_53) {
        // size2 = geom2.size[0]                                                              <L 2536>
        var_58 = &((var_geom2).size);
        var_61 = wp::load(var_58);
        var_60 = wp::extract(var_61, var_59);
        // full_margin2 = size2 + 0.5 * orig_margin2                                          <L 2537>
        var_63 = wp::mul(var_62, var_4);
        var_64 = wp::add(var_60, var_63);
        // geom2.margin = 0.0                                                                 <L 2538>
        var_geom2.margin = var_65;
        // geom2.size = wp.vec3(0.0, geom2.size[1], geom2.size[2])                            <L 2539>
        var_67 = &((var_geom2).size);
        var_70 = wp::load(var_67);
        var_69 = wp::extract(var_70, var_68);
        var_71 = &((var_geom2).size);
        var_74 = wp::load(var_71);
        var_73 = wp::extract(var_74, var_72);
        var_75 = wp::vec_t<3, wp::float32>(var_66, var_69, var_73);
        var_geom2.size = var_75;
    }
    var_76 = wp::where(var_53, var_64, var_13);
    var_77 = wp::where(var_53, var_60, var_15);
    // if size1 + size2 > 0.0:                                                                <L 2541>
    var_78 = wp::add(var_52, var_77);
    var_80 = (var_78 > var_79);
    if (var_80) {
        // cutoff += full_margin1 + full_margin2                                              <L 2542>
        var_81 = wp::add(var_51, var_76);
        var_82 = wp::add(var_cutoff, var_81);
        // result = gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2, cutoff, is_discrete)       <L 2543>
        var_83 = gjk_0(var_tolerance, var_gjk_iterations, var_geom1, var_geom2, var_x_1, var_x_2, var_geomtype1, var_geomtype2, var_82, var_17);
        // geom1.index = result.index1                                                        <L 2544>
        var_84 = &((var_83).index1);
        var_86 = wp::load(var_84);
        var_85 = wp::copy(var_86);
        var_geom1.index = var_85;
        // geom2.index = result.index2                                                        <L 2545>
        var_87 = &((var_83).index2);
        var_89 = wp::load(var_87);
        var_88 = wp::copy(var_89);
        var_geom2.index = var_88;
        // if result.dist > tolerance:                                                        <L 2548>
        var_90 = &((var_83).dist);
        var_92 = wp::load(var_90);
        var_91 = (var_92 > var_tolerance);
        if (var_91) {
            // if result.dist == FLOAT_MAX:                                                   <L 2549>
            var_93 = &((var_83).dist);
            var_96 = wp::load(var_93);
            var_95 = (var_96 == var_94);
            if (var_95) {
                // return False, result.dist, 1, result.x1, result.x2, empty, geom1, geom2       <L 2550>
                var_98 = &((var_83).dist);
                var_100 = &((var_83).x1);
                var_101 = &((var_83).x2);
                var_103 = wp::load(var_98);
                var_102 = wp::copy(var_103);
                var_105 = wp::load(var_100);
                var_104 = wp::copy(var_105);
                var_107 = wp::load(var_101);
                var_106 = wp::copy(var_107);
                ret_0 = var_97;
                ret_1 = var_102;
                ret_2 = var_99;
                ret_3 = var_104;
                ret_4 = var_106;
                ret_5 = var_16;
                ret_6 = var_geom1;
                ret_7 = var_geom2;
                return;
            }
            // dist, x1, x2 = _inflate(result, geom1, geom2, geomtype1, geomtype2, full_margin1, full_margin2)       <L 2551>
            _inflate_0(var_83, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_51, var_76, var_108, var_109, var_110);
            // return False, dist, 1, x1, x2, empty, geom1, geom2                             <L 2552>
            ret_0 = var_111;
            ret_1 = var_108;
            ret_2 = var_112;
            ret_3 = var_109;
            ret_4 = var_110;
            ret_5 = var_16;
            ret_6 = var_geom1;
            ret_7 = var_geom2;
            return;
        }
        // geom1.margin = orig_margin1                                                        <L 2555>
        var_geom1.margin = var_1;
        // geom1.size = orig_size1                                                            <L 2556>
        var_geom1.size = var_7;
        // geom2.margin = orig_margin2                                                        <L 2557>
        var_geom2.margin = var_4;
        // geom2.size = orig_size2                                                            <L 2558>
        var_geom2.size = var_10;
        // cutoff -= full_margin1 + full_margin2                                              <L 2559>
        var_113 = wp::add(var_51, var_76);
        var_114 = wp::sub(var_82, var_113);
    }
    var_115 = wp::where(var_80, var_114, var_cutoff);
    // result = gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2, cutoff, is_discrete)       <L 2561>
    var_116 = gjk_0(var_tolerance, var_gjk_iterations, var_geom1, var_geom2, var_x_1, var_x_2, var_geomtype1, var_geomtype2, var_115, var_17);
    // geom1.index = result.index1                                                            <L 2562>
    var_117 = &((var_116).index1);
    var_119 = wp::load(var_117);
    var_118 = wp::copy(var_119);
    var_geom1.index = var_118;
    // geom2.index = result.index2                                                            <L 2563>
    var_120 = &((var_116).index2);
    var_122 = wp::load(var_120);
    var_121 = wp::copy(var_122);
    var_geom2.index = var_121;
    // if result.dist > tolerance or result.dim < 2 or result.separated:                      <L 2566>
    var_124 = &((var_116).dist);
    var_126 = wp::load(var_124);
    var_125 = (var_126 > var_tolerance);
    var_123 = var_125;
    if (!var_123) {
        var_127 = &((var_116).dim);
        var_130 = wp::load(var_127);
        var_129 = (var_130 < var_128);
        var_123 = var_123 || var_129;
    }
    if (!var_123) {
        var_131 = &((var_116).separated);
        var_132 = wp::load(var_131);
        var_123 = var_123 || var_132;
    }
    if (var_123) {
        // return False, result.dist, 1, result.x1, result.x2, empty, geom1, geom2            <L 2567>
        var_134 = &((var_116).dist);
        var_136 = &((var_116).x1);
        var_137 = &((var_116).x2);
        var_139 = wp::load(var_134);
        var_138 = wp::copy(var_139);
        var_141 = wp::load(var_136);
        var_140 = wp::copy(var_141);
        var_143 = wp::load(var_137);
        var_142 = wp::copy(var_143);
        ret_0 = var_133;
        ret_1 = var_138;
        ret_2 = var_135;
        ret_3 = var_140;
        ret_4 = var_142;
        ret_5 = var_16;
        ret_6 = var_geom1;
        ret_7 = var_geom2;
        return;
    }
    // return True, result.dist, 1, result.x1, result.x2, result, geom1, geom2                <L 2569>
    var_145 = &((var_116).dist);
    var_147 = &((var_116).x1);
    var_148 = &((var_116).x2);
    var_150 = wp::load(var_145);
    var_149 = wp::copy(var_150);
    var_152 = wp::load(var_147);
    var_151 = wp::copy(var_152);
    var_154 = wp::load(var_148);
    var_153 = wp::copy(var_154);
    ret_0 = var_144;
    ret_1 = var_149;
    ret_2 = var_146;
    ret_3 = var_151;
    ret_4 = var_153;
    ret_5 = var_116;
    ret_6 = var_geom1;
    ret_7 = var_geom2;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:906
static CUDA_CALLABLE wp::mat_t<3, 3, wp::float32> _rotmat_0(
    wp::vec_t<3, wp::float32> var_axis)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    wp::float32 var_3;
    const wp::int32 var_4 = 1;
    wp::float32 var_5;
    wp::float32 var_6;
    const wp::int32 var_7 = 2;
    wp::float32 var_8;
    wp::float32 var_9;
    const wp::float32 var_10 = 0.86602540378;
    const wp::float32 var_11 = -0.5;
    wp::mat_t<3, 3, wp::float32> var_12;
    wp::float32 var_13;
    const wp::float32 var_14 = 1.0;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::int32 var_18 = 0;
    const wp::int32 var_19 = 0;
    wp::float32 var_20;
    const wp::float32 var_21 = 1.0;
    wp::float32 var_22;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::int32 var_26 = 0;
    const wp::int32 var_27 = 1;
    wp::float32 var_28;
    const wp::float32 var_29 = 1.0;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::int32 var_34 = 0;
    const wp::int32 var_35 = 2;
    wp::float32 var_36;
    const wp::float32 var_37 = 1.0;
    wp::float32 var_38;
    wp::float32 var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    const wp::int32 var_42 = 1;
    const wp::int32 var_43 = 0;
    wp::float32 var_44;
    const wp::float32 var_45 = 1.0;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    const wp::int32 var_49 = 1;
    const wp::int32 var_50 = 1;
    wp::float32 var_51;
    const wp::float32 var_52 = 1.0;
    wp::float32 var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    const wp::int32 var_57 = 1;
    const wp::int32 var_58 = 2;
    wp::float32 var_59;
    const wp::float32 var_60 = 1.0;
    wp::float32 var_61;
    wp::float32 var_62;
    wp::float32 var_63;
    wp::float32 var_64;
    const wp::int32 var_65 = 2;
    const wp::int32 var_66 = 0;
    wp::float32 var_67;
    const wp::float32 var_68 = 1.0;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    wp::float32 var_72;
    const wp::int32 var_73 = 2;
    const wp::int32 var_74 = 1;
    wp::float32 var_75;
    const wp::float32 var_76 = 1.0;
    wp::float32 var_77;
    wp::float32 var_78;
    wp::float32 var_79;
    const wp::int32 var_80 = 2;
    const wp::int32 var_81 = 2;
    //---------
    // forward
    // def _rotmat(axis: wp.vec3) -> wp.mat33:                                                <L 907>
    // n = wp.norm_l2(axis)                                                                   <L 908>
    var_0 = norm_l2_0(var_axis);
    // u1 = axis[0] / n                                                                       <L 909>
    var_2 = wp::extract(var_axis, var_1);
    var_3 = wp::div(var_2, var_0);
    // u2 = axis[1] / n                                                                       <L 910>
    var_5 = wp::extract(var_axis, var_4);
    var_6 = wp::div(var_5, var_0);
    // u3 = axis[2] / n                                                                       <L 911>
    var_8 = wp::extract(var_axis, var_7);
    var_9 = wp::div(var_8, var_0);
    // sin = 0.86602540378  # sin(120 deg)                                                    <L 913>
    // cos = -0.5  # cos(120 deg)                                                             <L 914>
    // R = wp.mat33()                                                                         <L 915>
    var_12 = wp::mat_t<3, 3, wp::float32>();
    // R[0, 0] = cos + u1 * u1 * (1.0 - cos)                                                  <L 916>
    var_13 = wp::mul(var_3, var_3);
    var_15 = wp::sub(var_14, var_11);
    var_16 = wp::mul(var_13, var_15);
    var_17 = wp::add(var_11, var_16);
    wp::assign_inplace(var_12, var_18, var_19, var_17);
    // R[0, 1] = u1 * u2 * (1.0 - cos) - u3 * sin                                             <L 917>
    var_20 = wp::mul(var_3, var_6);
    var_22 = wp::sub(var_21, var_11);
    var_23 = wp::mul(var_20, var_22);
    var_24 = wp::mul(var_9, var_10);
    var_25 = wp::sub(var_23, var_24);
    wp::assign_inplace(var_12, var_26, var_27, var_25);
    // R[0, 2] = u1 * u3 * (1.0 - cos) + u2 * sin                                             <L 918>
    var_28 = wp::mul(var_3, var_9);
    var_30 = wp::sub(var_29, var_11);
    var_31 = wp::mul(var_28, var_30);
    var_32 = wp::mul(var_6, var_10);
    var_33 = wp::add(var_31, var_32);
    wp::assign_inplace(var_12, var_34, var_35, var_33);
    // R[1, 0] = u2 * u1 * (1.0 - cos) + u3 * sin                                             <L 919>
    var_36 = wp::mul(var_6, var_3);
    var_38 = wp::sub(var_37, var_11);
    var_39 = wp::mul(var_36, var_38);
    var_40 = wp::mul(var_9, var_10);
    var_41 = wp::add(var_39, var_40);
    wp::assign_inplace(var_12, var_42, var_43, var_41);
    // R[1, 1] = cos + u2 * u2 * (1.0 - cos)                                                  <L 920>
    var_44 = wp::mul(var_6, var_6);
    var_46 = wp::sub(var_45, var_11);
    var_47 = wp::mul(var_44, var_46);
    var_48 = wp::add(var_11, var_47);
    wp::assign_inplace(var_12, var_49, var_50, var_48);
    // R[1, 2] = u2 * u3 * (1.0 - cos) - u1 * sin                                             <L 921>
    var_51 = wp::mul(var_6, var_9);
    var_53 = wp::sub(var_52, var_11);
    var_54 = wp::mul(var_51, var_53);
    var_55 = wp::mul(var_3, var_10);
    var_56 = wp::sub(var_54, var_55);
    wp::assign_inplace(var_12, var_57, var_58, var_56);
    // R[2, 0] = u1 * u3 * (1.0 - cos) - u2 * sin                                             <L 922>
    var_59 = wp::mul(var_3, var_9);
    var_61 = wp::sub(var_60, var_11);
    var_62 = wp::mul(var_59, var_61);
    var_63 = wp::mul(var_6, var_10);
    var_64 = wp::sub(var_62, var_63);
    wp::assign_inplace(var_12, var_65, var_66, var_64);
    // R[2, 1] = u2 * u3 * (1.0 - cos) + u1 * sin                                             <L 923>
    var_67 = wp::mul(var_6, var_9);
    var_69 = wp::sub(var_68, var_11);
    var_70 = wp::mul(var_67, var_69);
    var_71 = wp::mul(var_3, var_10);
    var_72 = wp::add(var_70, var_71);
    wp::assign_inplace(var_12, var_73, var_74, var_72);
    // R[2, 2] = cos + u3 * u3 * (1.0 - cos)                                                  <L 924>
    var_75 = wp::mul(var_9, var_9);
    var_77 = wp::sub(var_76, var_11);
    var_78 = wp::mul(var_75, var_77);
    var_79 = wp::add(var_11, var_78);
    wp::assign_inplace(var_12, var_80, var_81, var_79);
    // return R                                                                               <L 925>
    return var_12;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:274
static CUDA_CALLABLE void _epa_support_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_idx,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geom1_type,
    wp::int32 var_geom2_type,
    wp::vec_t<3, wp::float32> var_dir,
    wp::int32 & ret_0,
    wp::int32 & ret_1)
{
    //---------
    // primal vars
    SupportPoint_e82efc60 var_0;
    wp::vec_t<3, wp::float32>* var_1;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_2;
    const wp::int32 var_3 = 2;
    wp::int32 var_4;
    wp::array_t<wp::vec_t<3, wp::float32>> var_5;
    wp::vec_t<3, wp::float32> var_6;
    wp::int32* var_7;
    wp::array_t<wp::int32>* var_8;
    const wp::int32 var_9 = 2;
    wp::int32 var_10;
    wp::array_t<wp::int32> var_11;
    wp::int32 var_12;
    wp::int32* var_13;
    wp::int32 var_14;
    wp::int32 var_15;
    wp::vec_t<3, wp::float32> var_16;
    SupportPoint_e82efc60 var_17;
    wp::vec_t<3, wp::float32>* var_18;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_19;
    const wp::int32 var_20 = 2;
    wp::int32 var_21;
    const wp::int32 var_22 = 1;
    wp::int32 var_23;
    wp::array_t<wp::vec_t<3, wp::float32>> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::int32* var_26;
    wp::array_t<wp::int32>* var_27;
    const wp::int32 var_28 = 2;
    wp::int32 var_29;
    const wp::int32 var_30 = 1;
    wp::int32 var_31;
    wp::array_t<wp::int32> var_32;
    wp::int32 var_33;
    wp::int32* var_34;
    wp::int32 var_35;
    wp::int32 var_36;
    //---------
    // forward
    // def _epa_support(                                                                      <L 275>
    // sp = support(geom1, geom1_type, dir)                                                   <L 278>
    var_0 = support_0(var_geom1, var_geom1_type, var_dir);
    // pt.vert[2 * idx] = sp.point                                                            <L 279>
    var_1 = &((var_0).point);
    var_2 = &((var_pt).vert);
    var_4 = wp::mul(var_3, var_idx);
    var_5 = wp::load(var_2);
    var_6 = wp::load(var_1);
    wp::array_store(var_5, var_4, var_6);
    // pt.vert_index[2 * idx] = sp.vertex_index                                               <L 280>
    var_7 = &((var_0).vertex_index);
    var_8 = &((var_pt).vert_index);
    var_10 = wp::mul(var_9, var_idx);
    var_11 = wp::load(var_8);
    var_12 = wp::load(var_7);
    wp::array_store(var_11, var_10, var_12);
    // index1 = sp.cached_index                                                               <L 281>
    var_13 = &((var_0).cached_index);
    var_15 = wp::load(var_13);
    var_14 = wp::copy(var_15);
    // sp = support(geom2, geom2_type, -dir)                                                  <L 283>
    var_16 = wp::neg(var_dir);
    var_17 = support_0(var_geom2, var_geom2_type, var_16);
    // pt.vert[2 * idx + 1] = sp.point                                                        <L 284>
    var_18 = &((var_17).point);
    var_19 = &((var_pt).vert);
    var_21 = wp::mul(var_20, var_idx);
    var_23 = wp::add(var_21, var_22);
    var_24 = wp::load(var_19);
    var_25 = wp::load(var_18);
    wp::array_store(var_24, var_23, var_25);
    // pt.vert_index[2 * idx + 1] = sp.vertex_index                                           <L 285>
    var_26 = &((var_17).vertex_index);
    var_27 = &((var_pt).vert_index);
    var_29 = wp::mul(var_28, var_idx);
    var_31 = wp::add(var_29, var_30);
    var_32 = wp::load(var_27);
    var_33 = wp::load(var_26);
    wp::array_store(var_32, var_31, var_33);
    // index2 = sp.cached_index                                                               <L 286>
    var_34 = &((var_17).cached_index);
    var_36 = wp::load(var_34);
    var_35 = wp::copy(var_36);
    // return index1, index2                                                                  <L 288>
    ret_0 = var_14;
    ret_1 = var_35;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:248
static CUDA_CALLABLE wp::float32 _attach_face_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_idx,
    wp::int32 var_v1,
    wp::int32 var_v2,
    wp::int32 var_v3)
{
    //---------
    // primal vars
    wp::int32* var_0;
    wp::array_t<wp::int32>* var_1;
    wp::shape_t* var_2;
    const wp::int32 var_3 = 0;
    wp::int32 var_4;
    wp::shape_t var_5;
    bool var_6;
    wp::int32 var_7;
    const wp::float32 var_8 = 0.0;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_9;
    const wp::int32 var_10 = 2;
    wp::int32 var_11;
    wp::vec_t<3, wp::float32>* var_12;
    wp::array_t<wp::vec_t<3, wp::float32>> var_13;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_14;
    const wp::int32 var_15 = 2;
    wp::int32 var_16;
    const wp::int32 var_17 = 1;
    wp::int32 var_18;
    wp::vec_t<3, wp::float32>* var_19;
    wp::array_t<wp::vec_t<3, wp::float32>> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_24;
    const wp::int32 var_25 = 2;
    wp::int32 var_26;
    wp::vec_t<3, wp::float32>* var_27;
    wp::array_t<wp::vec_t<3, wp::float32>> var_28;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_29;
    const wp::int32 var_30 = 2;
    wp::int32 var_31;
    const wp::int32 var_32 = 1;
    wp::int32 var_33;
    wp::vec_t<3, wp::float32>* var_34;
    wp::array_t<wp::vec_t<3, wp::float32>> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_39;
    const wp::int32 var_40 = 2;
    wp::int32 var_41;
    wp::vec_t<3, wp::float32>* var_42;
    wp::array_t<wp::vec_t<3, wp::float32>> var_43;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_44;
    const wp::int32 var_45 = 2;
    wp::int32 var_46;
    const wp::int32 var_47 = 1;
    wp::int32 var_48;
    wp::vec_t<3, wp::float32>* var_49;
    wp::array_t<wp::vec_t<3, wp::float32>> var_50;
    wp::vec_t<3, wp::float32> var_51;
    wp::vec_t<3, wp::float32> var_52;
    wp::vec_t<3, wp::float32> var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::int32 var_55;
    const wp::float32 var_56 = 0.0;
    wp::vec_t<3, wp::float32>* var_57;
    wp::vec_t<3, wp::float32> var_58;
    wp::vec_t<3, wp::float32> var_59;
    wp::float32 var_60;
    const wp::float32 var_61 = 0.0;
    bool var_62;
    wp::vec_t<3, wp::float32> var_63;
    wp::vec_t<3, wp::float32> var_64;
    const wp::int32 var_65 = 10;
    wp::int32 var_66;
    wp::int32 var_67;
    const wp::int32 var_68 = 20;
    wp::int32 var_69;
    wp::int32 var_70;
    wp::array_t<wp::int32>* var_71;
    wp::array_t<wp::int32> var_72;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_73;
    wp::array_t<wp::vec_t<3, wp::float32>> var_74;
    wp::float32 var_75;
    wp::array_t<wp::float32>* var_76;
    wp::array_t<wp::float32> var_77;
    wp::array_t<wp::float32>* var_78;
    wp::float32* var_79;
    wp::array_t<wp::float32> var_80;
    wp::float32 var_81;
    wp::float32 var_82;
    //---------
    // forward
    // def _attach_face(pt: Polytope, idx: int, v1: int, v2: int, v3: int) -> float:          <L 249>
    // if pt.nface == pt.face.shape[0]:                                                       <L 251>
    var_0 = &((var_pt).nface);
    var_1 = &((var_pt).face);
    var_2 = &(var_1->shape);
    var_5 = wp::load(var_2);
    var_4 = wp::extract(var_5, var_3);
    var_7 = wp::load(var_0);
    var_6 = (var_7 == var_4);
    if (var_6) {
        // return 0.0                                                                         <L 252>
        return var_8;
    }
    // p1 = pt.vert[2 * v1] - pt.vert[2 * v1 + 1]                                             <L 255>
    var_9 = &((var_pt).vert);
    var_11 = wp::mul(var_10, var_v1);
    var_13 = wp::load(var_9);
    var_12 = wp::address(var_13, var_11);
    var_14 = &((var_pt).vert);
    var_16 = wp::mul(var_15, var_v1);
    var_18 = wp::add(var_16, var_17);
    var_20 = wp::load(var_14);
    var_19 = wp::address(var_20, var_18);
    var_22 = wp::load(var_12);
    var_23 = wp::load(var_19);
    var_21 = wp::sub(var_22, var_23);
    // p2 = pt.vert[2 * v2] - pt.vert[2 * v2 + 1]                                             <L 256>
    var_24 = &((var_pt).vert);
    var_26 = wp::mul(var_25, var_v2);
    var_28 = wp::load(var_24);
    var_27 = wp::address(var_28, var_26);
    var_29 = &((var_pt).vert);
    var_31 = wp::mul(var_30, var_v2);
    var_33 = wp::add(var_31, var_32);
    var_35 = wp::load(var_29);
    var_34 = wp::address(var_35, var_33);
    var_37 = wp::load(var_27);
    var_38 = wp::load(var_34);
    var_36 = wp::sub(var_37, var_38);
    // p3 = pt.vert[2 * v3] - pt.vert[2 * v3 + 1]                                             <L 257>
    var_39 = &((var_pt).vert);
    var_41 = wp::mul(var_40, var_v3);
    var_43 = wp::load(var_39);
    var_42 = wp::address(var_43, var_41);
    var_44 = &((var_pt).vert);
    var_46 = wp::mul(var_45, var_v3);
    var_48 = wp::add(var_46, var_47);
    var_50 = wp::load(var_44);
    var_49 = wp::address(var_50, var_48);
    var_52 = wp::load(var_42);
    var_53 = wp::load(var_49);
    var_51 = wp::sub(var_52, var_53);
    // r, ret = _project_origin_plane(p3, p2, p1)                                             <L 258>
    _project_origin_plane_0(var_51, var_36, var_21, var_54, var_55);
    // if ret:                                                                                <L 259>
    if (var_55) {
        // return 0.0                                                                         <L 260>
        return var_56;
    }
    // if wp.dot(r, p1 - pt.center) < 0.0:                                                    <L 263>
    var_57 = &((var_pt).center);
    var_59 = wp::load(var_57);
    var_58 = wp::sub(var_21, var_59);
    var_60 = wp::dot(var_54, var_58);
    var_62 = (var_60 < var_61);
    if (var_62) {
        // r = -r                                                                             <L 264>
        var_63 = wp::neg(var_54);
    }
    var_64 = wp::where(var_62, var_63, var_54);
    // face = v1 + (v2 << 10) + (v3 << 20)                                                    <L 266>
    var_66 = wp::lshift(var_v2, var_65);
    var_67 = wp::add(var_v1, var_66);
    var_69 = wp::lshift(var_v3, var_68);
    var_70 = wp::add(var_67, var_69);
    // pt.face[idx] = face                                                                    <L 267>
    var_71 = &((var_pt).face);
    var_72 = wp::load(var_71);
    wp::array_store(var_72, var_idx, var_70);
    // pt.face_pr[idx] = r                                                                    <L 268>
    var_73 = &((var_pt).face_pr);
    var_74 = wp::load(var_73);
    wp::array_store(var_74, var_idx, var_64);
    // pt.face_norm2[idx] = wp.dot(r, r)                                                      <L 270>
    var_75 = wp::dot(var_64, var_64);
    var_76 = &((var_pt).face_norm2);
    var_77 = wp::load(var_76);
    wp::array_store(var_77, var_idx, var_75);
    // return pt.face_norm2[idx]                                                              <L 271>
    var_78 = &((var_pt).face_norm2);
    var_80 = wp::load(var_78);
    var_79 = wp::address(var_80, var_idx);
    var_82 = wp::load(var_79);
    var_81 = wp::copy(var_82);
    return var_81;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:867
static CUDA_CALLABLE GJKResult_8184e255 _replace_simplex3_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_v1,
    wp::int32 var_v2,
    wp::int32 var_v3)
{
    //---------
    // primal vars
    GJKResult_8184e255 var_0;
    wp::mat_t<4, 3, wp::float32> var_1;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_2;
    const wp::int32 var_3 = 2;
    wp::int32 var_4;
    wp::vec_t<3, wp::float32>* var_5;
    wp::array_t<wp::vec_t<3, wp::float32>> var_6;
    const wp::int32 var_7 = 0;
    wp::vec_t<3, wp::float32> var_8;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_9;
    const wp::int32 var_10 = 2;
    wp::int32 var_11;
    wp::vec_t<3, wp::float32>* var_12;
    wp::array_t<wp::vec_t<3, wp::float32>> var_13;
    const wp::int32 var_14 = 1;
    wp::vec_t<3, wp::float32> var_15;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_16;
    const wp::int32 var_17 = 2;
    wp::int32 var_18;
    wp::vec_t<3, wp::float32>* var_19;
    wp::array_t<wp::vec_t<3, wp::float32>> var_20;
    const wp::int32 var_21 = 2;
    wp::vec_t<3, wp::float32> var_22;
    wp::mat_t<4, 3, wp::float32> var_23;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_24;
    const wp::int32 var_25 = 2;
    wp::int32 var_26;
    const wp::int32 var_27 = 1;
    wp::int32 var_28;
    wp::vec_t<3, wp::float32>* var_29;
    wp::array_t<wp::vec_t<3, wp::float32>> var_30;
    const wp::int32 var_31 = 0;
    wp::vec_t<3, wp::float32> var_32;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_33;
    const wp::int32 var_34 = 2;
    wp::int32 var_35;
    const wp::int32 var_36 = 1;
    wp::int32 var_37;
    wp::vec_t<3, wp::float32>* var_38;
    wp::array_t<wp::vec_t<3, wp::float32>> var_39;
    const wp::int32 var_40 = 1;
    wp::vec_t<3, wp::float32> var_41;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_42;
    const wp::int32 var_43 = 2;
    wp::int32 var_44;
    const wp::int32 var_45 = 1;
    wp::int32 var_46;
    wp::vec_t<3, wp::float32>* var_47;
    wp::array_t<wp::vec_t<3, wp::float32>> var_48;
    const wp::int32 var_49 = 2;
    wp::vec_t<3, wp::float32> var_50;
    wp::mat_t<4, 3, wp::float32> var_51;
    const wp::int32 var_52 = 0;
    wp::vec_t<3, wp::float32> var_53;
    const wp::int32 var_54 = 0;
    wp::vec_t<3, wp::float32> var_55;
    wp::vec_t<3, wp::float32> var_56;
    const wp::int32 var_57 = 0;
    const wp::int32 var_58 = 1;
    wp::vec_t<3, wp::float32> var_59;
    const wp::int32 var_60 = 1;
    wp::vec_t<3, wp::float32> var_61;
    wp::vec_t<3, wp::float32> var_62;
    const wp::int32 var_63 = 1;
    const wp::int32 var_64 = 2;
    wp::vec_t<3, wp::float32> var_65;
    const wp::int32 var_66 = 2;
    wp::vec_t<3, wp::float32> var_67;
    wp::vec_t<3, wp::float32> var_68;
    const wp::int32 var_69 = 2;
    wp::vec_t<4, wp::int32> var_70;
    wp::array_t<wp::int32>* var_71;
    const wp::int32 var_72 = 2;
    wp::int32 var_73;
    wp::int32* var_74;
    wp::array_t<wp::int32> var_75;
    const wp::int32 var_76 = 0;
    wp::int32 var_77;
    wp::array_t<wp::int32>* var_78;
    const wp::int32 var_79 = 2;
    wp::int32 var_80;
    wp::int32* var_81;
    wp::array_t<wp::int32> var_82;
    const wp::int32 var_83 = 1;
    wp::int32 var_84;
    wp::array_t<wp::int32>* var_85;
    const wp::int32 var_86 = 2;
    wp::int32 var_87;
    wp::int32* var_88;
    wp::array_t<wp::int32> var_89;
    const wp::int32 var_90 = 2;
    wp::int32 var_91;
    wp::vec_t<4, wp::int32> var_92;
    wp::array_t<wp::int32>* var_93;
    const wp::int32 var_94 = 2;
    wp::int32 var_95;
    const wp::int32 var_96 = 1;
    wp::int32 var_97;
    wp::int32* var_98;
    wp::array_t<wp::int32> var_99;
    const wp::int32 var_100 = 0;
    wp::int32 var_101;
    wp::array_t<wp::int32>* var_102;
    const wp::int32 var_103 = 2;
    wp::int32 var_104;
    const wp::int32 var_105 = 1;
    wp::int32 var_106;
    wp::int32* var_107;
    wp::array_t<wp::int32> var_108;
    const wp::int32 var_109 = 1;
    wp::int32 var_110;
    wp::array_t<wp::int32>* var_111;
    const wp::int32 var_112 = 2;
    wp::int32 var_113;
    const wp::int32 var_114 = 1;
    wp::int32 var_115;
    wp::int32* var_116;
    wp::array_t<wp::int32> var_117;
    const wp::int32 var_118 = 2;
    wp::int32 var_119;
    //---------
    // forward
    // def _replace_simplex3(pt: Polytope, v1: int, v2: int, v3: int) -> GJKResult:           <L 868>
    // result = GJKResult()                                                                   <L 869>
    var_0 = GJKResult_8184e255();
    // simplex1 = mat43()                                                                     <L 872>
    var_1 = wp::mat_t<4, 3, wp::float32>();
    // simplex1[0] = pt.vert[2 * v1]                                                          <L 873>
    var_2 = &((var_pt).vert);
    var_4 = wp::mul(var_3, var_v1);
    var_6 = wp::load(var_2);
    var_5 = wp::address(var_6, var_4);
    var_8 = wp::load(var_5);
    wp::assign_inplace(var_1, var_7, var_8);
    // simplex1[1] = pt.vert[2 * v2]                                                          <L 874>
    var_9 = &((var_pt).vert);
    var_11 = wp::mul(var_10, var_v2);
    var_13 = wp::load(var_9);
    var_12 = wp::address(var_13, var_11);
    var_15 = wp::load(var_12);
    wp::assign_inplace(var_1, var_14, var_15);
    // simplex1[2] = pt.vert[2 * v3]                                                          <L 875>
    var_16 = &((var_pt).vert);
    var_18 = wp::mul(var_17, var_v3);
    var_20 = wp::load(var_16);
    var_19 = wp::address(var_20, var_18);
    var_22 = wp::load(var_19);
    wp::assign_inplace(var_1, var_21, var_22);
    // simplex2 = mat43()                                                                     <L 877>
    var_23 = wp::mat_t<4, 3, wp::float32>();
    // simplex2[0] = pt.vert[2 * v1 + 1]                                                      <L 878>
    var_24 = &((var_pt).vert);
    var_26 = wp::mul(var_25, var_v1);
    var_28 = wp::add(var_26, var_27);
    var_30 = wp::load(var_24);
    var_29 = wp::address(var_30, var_28);
    var_32 = wp::load(var_29);
    wp::assign_inplace(var_23, var_31, var_32);
    // simplex2[1] = pt.vert[2 * v2 + 1]                                                      <L 879>
    var_33 = &((var_pt).vert);
    var_35 = wp::mul(var_34, var_v2);
    var_37 = wp::add(var_35, var_36);
    var_39 = wp::load(var_33);
    var_38 = wp::address(var_39, var_37);
    var_41 = wp::load(var_38);
    wp::assign_inplace(var_23, var_40, var_41);
    // simplex2[2] = pt.vert[2 * v3 + 1]                                                      <L 880>
    var_42 = &((var_pt).vert);
    var_44 = wp::mul(var_43, var_v3);
    var_46 = wp::add(var_44, var_45);
    var_48 = wp::load(var_42);
    var_47 = wp::address(var_48, var_46);
    var_50 = wp::load(var_47);
    wp::assign_inplace(var_23, var_49, var_50);
    // simplex = mat43()                                                                      <L 882>
    var_51 = wp::mat_t<4, 3, wp::float32>();
    // simplex[0] = simplex1[0] - simplex2[0]                                                 <L 883>
    var_53 = wp::extract(var_1, var_52);
    var_55 = wp::extract(var_23, var_54);
    var_56 = wp::sub(var_53, var_55);
    wp::assign_inplace(var_51, var_57, var_56);
    // simplex[1] = simplex1[1] - simplex2[1]                                                 <L 884>
    var_59 = wp::extract(var_1, var_58);
    var_61 = wp::extract(var_23, var_60);
    var_62 = wp::sub(var_59, var_61);
    wp::assign_inplace(var_51, var_63, var_62);
    // simplex[2] = simplex1[2] - simplex2[2]                                                 <L 885>
    var_65 = wp::extract(var_1, var_64);
    var_67 = wp::extract(var_23, var_66);
    var_68 = wp::sub(var_65, var_67);
    wp::assign_inplace(var_51, var_69, var_68);
    // simplex_index1 = wp.vec4i()                                                            <L 887>
    var_70 = wp::vec_t<4, wp::int32>();
    // simplex_index1[0] = pt.vert_index[2 * v1]                                              <L 888>
    var_71 = &((var_pt).vert_index);
    var_73 = wp::mul(var_72, var_v1);
    var_75 = wp::load(var_71);
    var_74 = wp::address(var_75, var_73);
    var_77 = wp::load(var_74);
    wp::assign_inplace(var_70, var_76, var_77);
    // simplex_index1[1] = pt.vert_index[2 * v2]                                              <L 889>
    var_78 = &((var_pt).vert_index);
    var_80 = wp::mul(var_79, var_v2);
    var_82 = wp::load(var_78);
    var_81 = wp::address(var_82, var_80);
    var_84 = wp::load(var_81);
    wp::assign_inplace(var_70, var_83, var_84);
    // simplex_index1[2] = pt.vert_index[2 * v3]                                              <L 890>
    var_85 = &((var_pt).vert_index);
    var_87 = wp::mul(var_86, var_v3);
    var_89 = wp::load(var_85);
    var_88 = wp::address(var_89, var_87);
    var_91 = wp::load(var_88);
    wp::assign_inplace(var_70, var_90, var_91);
    // simplex_index2 = wp.vec4i()                                                            <L 892>
    var_92 = wp::vec_t<4, wp::int32>();
    // simplex_index2[0] = pt.vert_index[2 * v1 + 1]                                          <L 893>
    var_93 = &((var_pt).vert_index);
    var_95 = wp::mul(var_94, var_v1);
    var_97 = wp::add(var_95, var_96);
    var_99 = wp::load(var_93);
    var_98 = wp::address(var_99, var_97);
    var_101 = wp::load(var_98);
    wp::assign_inplace(var_92, var_100, var_101);
    // simplex_index2[1] = pt.vert_index[2 * v2 + 1]                                          <L 894>
    var_102 = &((var_pt).vert_index);
    var_104 = wp::mul(var_103, var_v2);
    var_106 = wp::add(var_104, var_105);
    var_108 = wp::load(var_102);
    var_107 = wp::address(var_108, var_106);
    var_110 = wp::load(var_107);
    wp::assign_inplace(var_92, var_109, var_110);
    // simplex_index2[2] = pt.vert_index[2 * v3 + 1]                                          <L 895>
    var_111 = &((var_pt).vert_index);
    var_113 = wp::mul(var_112, var_v3);
    var_115 = wp::add(var_113, var_114);
    var_117 = wp::load(var_111);
    var_116 = wp::address(var_117, var_115);
    var_119 = wp::load(var_116);
    wp::assign_inplace(var_92, var_118, var_119);
    // result.simplex = simplex                                                               <L 897>
    var_0.simplex = var_51;
    // result.simplex1 = simplex1                                                             <L 898>
    var_0.simplex1 = var_1;
    // result.simplex2 = simplex2                                                             <L 899>
    var_0.simplex2 = var_23;
    // result.simplex_index1 = simplex_index1                                                 <L 900>
    var_0.simplex_index1 = var_70;
    // result.simplex_index2 = simplex_index2                                                 <L 901>
    var_0.simplex_index2 = var_92;
    // return result                                                                          <L 903>
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:928
static CUDA_CALLABLE wp::int32 _ray_triangle_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_v4,
    wp::vec_t<3, wp::float32> var_v5)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::float32 var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::vec_t<3, wp::float32> var_6;
    wp::float32 var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::float32 var_11;
    bool var_12;
    const wp::float32 var_13 = 0.0;
    bool var_14;
    const wp::float32 var_15 = 0.0;
    bool var_16;
    const wp::float32 var_17 = 0.0;
    bool var_18;
    const wp::int32 var_19 = 1;
    bool var_20;
    const wp::float32 var_21 = 0.0;
    bool var_22;
    const wp::float32 var_23 = 0.0;
    bool var_24;
    const wp::float32 var_25 = 0.0;
    bool var_26;
    const wp::int32 var_27 = -1;
    const wp::int32 var_28 = 0;
    //---------
    // forward
    // def _ray_triangle(v1: wp.vec3, v2: wp.vec3, v3: wp.vec3, v4: wp.vec3, v5: wp.vec3) -> int:       <L 929>
    // vol1 = _det3(v3 - v1, v4 - v1, v2 - v1)                                                <L 930>
    var_0 = wp::sub(var_v3, var_v1);
    var_1 = wp::sub(var_v4, var_v1);
    var_2 = wp::sub(var_v2, var_v1);
    var_3 = _det3_0(var_0, var_1, var_2);
    // vol2 = _det3(v4 - v1, v5 - v1, v2 - v1)                                                <L 931>
    var_4 = wp::sub(var_v4, var_v1);
    var_5 = wp::sub(var_v5, var_v1);
    var_6 = wp::sub(var_v2, var_v1);
    var_7 = _det3_0(var_4, var_5, var_6);
    // vol3 = _det3(v5 - v1, v3 - v1, v2 - v1)                                                <L 932>
    var_8 = wp::sub(var_v5, var_v1);
    var_9 = wp::sub(var_v3, var_v1);
    var_10 = wp::sub(var_v2, var_v1);
    var_11 = _det3_0(var_8, var_9, var_10);
    // if vol1 >= 0.0 and vol2 >= 0.0 and vol3 >= 0.0:                                        <L 934>
    var_14 = (var_3 >= var_13);
    var_12 = var_14;
    if (var_12) {
        var_16 = (var_7 >= var_15);
        var_12 = var_12 && var_16;
    }
    if (var_12) {
        var_18 = (var_11 >= var_17);
        var_12 = var_12 && var_18;
    }
    if (var_12) {
        // return 1                                                                           <L 935>
        return var_19;
    }
    // if vol1 <= 0.0 and vol2 <= 0.0 and vol3 <= 0.0:                                        <L 936>
    var_22 = (var_3 <= var_21);
    var_20 = var_22;
    if (var_20) {
        var_24 = (var_7 <= var_23);
        var_20 = var_20 && var_24;
    }
    if (var_20) {
        var_26 = (var_11 <= var_25);
        var_20 = var_20 && var_26;
    }
    if (var_20) {
        // return -1                                                                          <L 937>
        return var_27;
    }
    // return 0                                                                               <L 938>
    return var_28;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1042
static CUDA_CALLABLE void _polytope2_0(
    Polytope_10582b13 var_pt,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    Polytope_10582b13 & ret_0,
    GJKResult_8184e255 & ret_1)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::vec_t<3, wp::float32> var_1;
    const wp::int32 var_2 = 0;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    const wp::float32 var_5 = 0.5;
    const wp::int32 var_6 = 0;
    wp::vec_t<3, wp::float32> var_7;
    const wp::int32 var_8 = 1;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    const wp::float32 var_12 = 1e+30;
    wp::float32 var_13;
    const wp::int32 var_14 = 0;
    const wp::int32 var_15 = 0;
    wp::float32 var_16;
    wp::float32 var_17;
    bool var_18;
    wp::float32 var_19;
    wp::float32 var_20;
    wp::int32 var_21;
    wp::float32 var_22;
    wp::int32 var_23;
    const wp::int32 var_24 = 1;
    wp::float32 var_25;
    wp::float32 var_26;
    bool var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    wp::int32 var_30;
    wp::float32 var_31;
    wp::int32 var_32;
    const wp::int32 var_33 = 2;
    wp::float32 var_34;
    wp::float32 var_35;
    bool var_36;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::int32 var_39;
    wp::float32 var_40;
    wp::int32 var_41;
    const wp::float32 var_42 = 0.0;
    const wp::float32 var_43 = 0.0;
    const wp::float32 var_44 = 0.0;
    wp::vec_t<3, wp::float32> var_45;
    const wp::float32 var_46 = 1.0;
    wp::vec_t<3, wp::float32> var_47;
    wp::mat_t<3, 3, wp::float32> var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::vec_t<3, wp::float32> var_50;
    const wp::int32 var_51 = 0;
    wp::vec_t<3, wp::float32> var_52;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_53;
    const wp::int32 var_54 = 0;
    wp::array_t<wp::vec_t<3, wp::float32>> var_55;
    const wp::int32 var_56 = 0;
    wp::vec_t<3, wp::float32> var_57;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_58;
    const wp::int32 var_59 = 1;
    wp::array_t<wp::vec_t<3, wp::float32>> var_60;
    const wp::int32 var_61 = 1;
    wp::vec_t<3, wp::float32> var_62;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_63;
    const wp::int32 var_64 = 2;
    wp::array_t<wp::vec_t<3, wp::float32>> var_65;
    const wp::int32 var_66 = 1;
    wp::vec_t<3, wp::float32> var_67;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_68;
    const wp::int32 var_69 = 3;
    wp::array_t<wp::vec_t<3, wp::float32>> var_70;
    const wp::int32 var_71 = 0;
    wp::int32 var_72;
    wp::array_t<wp::int32>* var_73;
    const wp::int32 var_74 = 0;
    wp::array_t<wp::int32> var_75;
    const wp::int32 var_76 = 0;
    wp::int32 var_77;
    wp::array_t<wp::int32>* var_78;
    const wp::int32 var_79 = 1;
    wp::array_t<wp::int32> var_80;
    const wp::int32 var_81 = 1;
    wp::int32 var_82;
    wp::array_t<wp::int32>* var_83;
    const wp::int32 var_84 = 2;
    wp::array_t<wp::int32> var_85;
    const wp::int32 var_86 = 1;
    wp::int32 var_87;
    wp::array_t<wp::int32>* var_88;
    const wp::int32 var_89 = 3;
    wp::array_t<wp::int32> var_90;
    const wp::int32 var_91 = 2;
    wp::float32 var_92;
    wp::vec_t<3, wp::float32> var_93;
    wp::int32 var_94;
    wp::int32 var_95;
    const wp::int32 var_96 = 3;
    wp::float32 var_97;
    wp::vec_t<3, wp::float32> var_98;
    wp::int32 var_99;
    wp::int32 var_100;
    const wp::int32 var_101 = 4;
    wp::float32 var_102;
    wp::vec_t<3, wp::float32> var_103;
    wp::int32 var_104;
    wp::int32 var_105;
    const wp::int32 var_106 = 0;
    const wp::int32 var_107 = 0;
    const wp::int32 var_108 = 2;
    const wp::int32 var_109 = 3;
    wp::float32 var_110;
    const wp::float32 var_111 = 1e-10;
    bool var_112;
    const wp::int32 var_113 = -1;
    const wp::int32 var_114 = 0;
    const wp::int32 var_115 = 2;
    const wp::int32 var_116 = 3;
    GJKResult_8184e255 var_117;
    const wp::int32 var_118 = 1;
    const wp::int32 var_119 = 0;
    const wp::int32 var_120 = 4;
    const wp::int32 var_121 = 2;
    wp::float32 var_122;
    bool var_123;
    const wp::int32 var_124 = -1;
    const wp::int32 var_125 = 0;
    const wp::int32 var_126 = 4;
    const wp::int32 var_127 = 2;
    GJKResult_8184e255 var_128;
    const wp::int32 var_129 = 2;
    const wp::int32 var_130 = 0;
    const wp::int32 var_131 = 3;
    const wp::int32 var_132 = 4;
    wp::float32 var_133;
    bool var_134;
    const wp::int32 var_135 = -1;
    const wp::int32 var_136 = 0;
    const wp::int32 var_137 = 3;
    const wp::int32 var_138 = 4;
    GJKResult_8184e255 var_139;
    const wp::int32 var_140 = 3;
    const wp::int32 var_141 = 1;
    const wp::int32 var_142 = 3;
    const wp::int32 var_143 = 2;
    wp::float32 var_144;
    bool var_145;
    const wp::int32 var_146 = -1;
    const wp::int32 var_147 = 1;
    const wp::int32 var_148 = 3;
    const wp::int32 var_149 = 2;
    GJKResult_8184e255 var_150;
    const wp::int32 var_151 = 4;
    const wp::int32 var_152 = 1;
    const wp::int32 var_153 = 2;
    const wp::int32 var_154 = 4;
    wp::float32 var_155;
    bool var_156;
    const wp::int32 var_157 = -1;
    const wp::int32 var_158 = 1;
    const wp::int32 var_159 = 2;
    const wp::int32 var_160 = 4;
    GJKResult_8184e255 var_161;
    const wp::int32 var_162 = 5;
    const wp::int32 var_163 = 1;
    const wp::int32 var_164 = 4;
    const wp::int32 var_165 = 3;
    wp::float32 var_166;
    bool var_167;
    const wp::int32 var_168 = -1;
    const wp::int32 var_169 = 1;
    const wp::int32 var_170 = 4;
    const wp::int32 var_171 = 3;
    GJKResult_8184e255 var_172;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_173;
    const wp::int32 var_174 = 4;
    wp::vec_t<3, wp::float32>* var_175;
    wp::array_t<wp::vec_t<3, wp::float32>> var_176;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_177;
    const wp::int32 var_178 = 5;
    wp::vec_t<3, wp::float32>* var_179;
    wp::array_t<wp::vec_t<3, wp::float32>> var_180;
    wp::vec_t<3, wp::float32> var_181;
    wp::vec_t<3, wp::float32> var_182;
    wp::vec_t<3, wp::float32> var_183;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_184;
    const wp::int32 var_185 = 6;
    wp::vec_t<3, wp::float32>* var_186;
    wp::array_t<wp::vec_t<3, wp::float32>> var_187;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_188;
    const wp::int32 var_189 = 7;
    wp::vec_t<3, wp::float32>* var_190;
    wp::array_t<wp::vec_t<3, wp::float32>> var_191;
    wp::vec_t<3, wp::float32> var_192;
    wp::vec_t<3, wp::float32> var_193;
    wp::vec_t<3, wp::float32> var_194;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_195;
    const wp::int32 var_196 = 8;
    wp::vec_t<3, wp::float32>* var_197;
    wp::array_t<wp::vec_t<3, wp::float32>> var_198;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_199;
    const wp::int32 var_200 = 9;
    wp::vec_t<3, wp::float32>* var_201;
    wp::array_t<wp::vec_t<3, wp::float32>> var_202;
    wp::vec_t<3, wp::float32> var_203;
    wp::vec_t<3, wp::float32> var_204;
    wp::vec_t<3, wp::float32> var_205;
    const wp::int32 var_206 = 0;
    wp::vec_t<3, wp::float32> var_207;
    const wp::int32 var_208 = 1;
    wp::vec_t<3, wp::float32> var_209;
    wp::int32 var_210;
    bool var_211;
    const wp::int32 var_212 = 1;
    GJKResult_8184e255 var_213;
    const wp::int32 var_214 = 5;
    const wp::int32 var_215 = 6;
    const wp::int32 var_216 = 0;
    GJKResult_8184e255 var_217;
    //---------
    // forward
    // def _polytope2(                                                                        <L 1043>
    // diff = simplex[1] - simplex[0]                                                         <L 1057>
    var_1 = wp::extract(var_simplex, var_0);
    var_3 = wp::extract(var_simplex, var_2);
    var_4 = wp::sub(var_1, var_3);
    // pt.center = 0.5 * (simplex[0] + simplex[1])                                            <L 1060>
    var_7 = wp::extract(var_simplex, var_6);
    var_9 = wp::extract(var_simplex, var_8);
    var_10 = wp::add(var_7, var_9);
    var_11 = wp::mul(var_5, var_10);
    var_pt.center = var_11;
    // value = FLOAT_MAX                                                                      <L 1063>
    var_13 = wp::copy(var_12);
    // index = 0                                                                              <L 1064>
    // for i in range(3):                                                                     <L 1065>
    // if wp.abs(diff[i]) < value:                                                            <L 1066>
    var_16 = wp::extract(var_4, var_15);
    var_17 = wp::abs(var_16);
    var_18 = (var_17 < var_13);
    if (var_18) {
        // value = wp.abs(diff[i])                                                            <L 1067>
        var_19 = wp::extract(var_4, var_15);
        var_20 = wp::abs(var_19);
        // index = i                                                                          <L 1068>
        var_21 = wp::copy(var_15);
    }
    var_22 = wp::where(var_18, var_20, var_13);
    var_23 = wp::where(var_18, var_21, var_14);
    // if wp.abs(diff[i]) < value:                                                            <L 1066>
    var_25 = wp::extract(var_4, var_24);
    var_26 = wp::abs(var_25);
    var_27 = (var_26 < var_22);
    if (var_27) {
        // value = wp.abs(diff[i])                                                            <L 1067>
        var_28 = wp::extract(var_4, var_24);
        var_29 = wp::abs(var_28);
        // index = i                                                                          <L 1068>
        var_30 = wp::copy(var_24);
    }
    var_31 = wp::where(var_27, var_29, var_22);
    var_32 = wp::where(var_27, var_30, var_23);
    // if wp.abs(diff[i]) < value:                                                            <L 1066>
    var_34 = wp::extract(var_4, var_33);
    var_35 = wp::abs(var_34);
    var_36 = (var_35 < var_31);
    if (var_36) {
        // value = wp.abs(diff[i])                                                            <L 1067>
        var_37 = wp::extract(var_4, var_33);
        var_38 = wp::abs(var_37);
        // index = i                                                                          <L 1068>
        var_39 = wp::copy(var_33);
    }
    var_40 = wp::where(var_36, var_38, var_31);
    var_41 = wp::where(var_36, var_39, var_32);
    // e = wp.vec3(0.0, 0.0, 0.0)                                                             <L 1071>
    var_45 = wp::vec_t<3, wp::float32>(var_42, var_43, var_44);
    // e[index] = 1.0                                                                         <L 1072>
    wp::assign_inplace(var_45, var_41, var_46);
    // d1 = wp.cross(e, diff)                                                                 <L 1073>
    var_47 = wp::cross(var_45, var_4);
    // R = _rotmat(diff)                                                                      <L 1076>
    var_48 = _rotmat_0(var_4);
    // d2 = R @ d1                                                                            <L 1077>
    var_49 = wp::mul(var_48, var_47);
    // d3 = R @ d2                                                                            <L 1078>
    var_50 = wp::mul(var_48, var_49);
    // pt.vert[0] = simplex1[0]                                                               <L 1081>
    var_52 = wp::extract(var_simplex1, var_51);
    var_53 = &((var_pt).vert);
    var_55 = wp::load(var_53);
    wp::array_store(var_55, var_54, var_52);
    // pt.vert[1] = simplex2[0]                                                               <L 1082>
    var_57 = wp::extract(var_simplex2, var_56);
    var_58 = &((var_pt).vert);
    var_60 = wp::load(var_58);
    wp::array_store(var_60, var_59, var_57);
    // pt.vert[2] = simplex1[1]                                                               <L 1083>
    var_62 = wp::extract(var_simplex1, var_61);
    var_63 = &((var_pt).vert);
    var_65 = wp::load(var_63);
    wp::array_store(var_65, var_64, var_62);
    // pt.vert[3] = simplex2[1]                                                               <L 1084>
    var_67 = wp::extract(var_simplex2, var_66);
    var_68 = &((var_pt).vert);
    var_70 = wp::load(var_68);
    wp::array_store(var_70, var_69, var_67);
    // pt.vert_index[0] = simplex_index1[0]                                                   <L 1086>
    var_72 = wp::extract(var_simplex_index1, var_71);
    var_73 = &((var_pt).vert_index);
    var_75 = wp::load(var_73);
    wp::array_store(var_75, var_74, var_72);
    // pt.vert_index[1] = simplex_index2[0]                                                   <L 1087>
    var_77 = wp::extract(var_simplex_index2, var_76);
    var_78 = &((var_pt).vert_index);
    var_80 = wp::load(var_78);
    wp::array_store(var_80, var_79, var_77);
    // pt.vert_index[2] = simplex_index1[1]                                                   <L 1088>
    var_82 = wp::extract(var_simplex_index1, var_81);
    var_83 = &((var_pt).vert_index);
    var_85 = wp::load(var_83);
    wp::array_store(var_85, var_84, var_82);
    // pt.vert_index[3] = simplex_index2[1]                                                   <L 1089>
    var_87 = wp::extract(var_simplex_index2, var_86);
    var_88 = &((var_pt).vert_index);
    var_90 = wp::load(var_88);
    wp::array_store(var_90, var_89, var_87);
    // _epa_support(pt, 2, geom1, geom2, geomtype1, geomtype2, d1 / wp.norm_l2(d1))           <L 1091>
    var_92 = norm_l2_0(var_47);
    var_93 = wp::div(var_47, var_92);
    _epa_support_0(var_pt, var_91, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_93, var_94, var_95);
    // _epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2, d2 / wp.norm_l2(d2))           <L 1092>
    var_97 = norm_l2_0(var_49);
    var_98 = wp::div(var_49, var_97);
    _epa_support_0(var_pt, var_96, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_98, var_99, var_100);
    // _epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2, d3 / wp.norm_l2(d3))           <L 1093>
    var_102 = norm_l2_0(var_50);
    var_103 = wp::div(var_50, var_102);
    _epa_support_0(var_pt, var_101, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_103, var_104, var_105);
    // if _attach_face(pt, 0, 0, 2, 3) < MIN_DIST2:                                           <L 1096>
    var_110 = _attach_face_0(var_pt, var_106, var_107, var_108, var_109);
    var_112 = (var_110 < var_111);
    if (var_112) {
        // pt.status = -1                                                                     <L 1097>
        var_pt.status = var_113;
        // return pt, _replace_simplex3(pt, 0, 2, 3)                                          <L 1098>
        var_117 = _replace_simplex3_0(var_pt, var_114, var_115, var_116);
        ret_0 = var_pt;
        ret_1 = var_117;
        return;
    }
    // if _attach_face(pt, 1, 0, 4, 2) < MIN_DIST2:                                           <L 1100>
    var_122 = _attach_face_0(var_pt, var_118, var_119, var_120, var_121);
    var_123 = (var_122 < var_111);
    if (var_123) {
        // pt.status = -1                                                                     <L 1101>
        var_pt.status = var_124;
        // return pt, _replace_simplex3(pt, 0, 4, 2)                                          <L 1102>
        var_128 = _replace_simplex3_0(var_pt, var_125, var_126, var_127);
        ret_0 = var_pt;
        ret_1 = var_128;
        return;
    }
    // if _attach_face(pt, 2, 0, 3, 4) < MIN_DIST2:                                           <L 1104>
    var_133 = _attach_face_0(var_pt, var_129, var_130, var_131, var_132);
    var_134 = (var_133 < var_111);
    if (var_134) {
        // pt.status = -1                                                                     <L 1105>
        var_pt.status = var_135;
        // return pt, _replace_simplex3(pt, 0, 3, 4)                                          <L 1106>
        var_139 = _replace_simplex3_0(var_pt, var_136, var_137, var_138);
        ret_0 = var_pt;
        ret_1 = var_139;
        return;
    }
    // if _attach_face(pt, 3, 1, 3, 2) < MIN_DIST2:                                           <L 1108>
    var_144 = _attach_face_0(var_pt, var_140, var_141, var_142, var_143);
    var_145 = (var_144 < var_111);
    if (var_145) {
        // pt.status = -1                                                                     <L 1109>
        var_pt.status = var_146;
        // return pt, _replace_simplex3(pt, 1, 3, 2)                                          <L 1110>
        var_150 = _replace_simplex3_0(var_pt, var_147, var_148, var_149);
        ret_0 = var_pt;
        ret_1 = var_150;
        return;
    }
    // if _attach_face(pt, 4, 1, 2, 4) < MIN_DIST2:                                           <L 1112>
    var_155 = _attach_face_0(var_pt, var_151, var_152, var_153, var_154);
    var_156 = (var_155 < var_111);
    if (var_156) {
        // pt.status = -1                                                                     <L 1113>
        var_pt.status = var_157;
        // return pt, _replace_simplex3(pt, 1, 2, 4)                                          <L 1114>
        var_161 = _replace_simplex3_0(var_pt, var_158, var_159, var_160);
        ret_0 = var_pt;
        ret_1 = var_161;
        return;
    }
    // if _attach_face(pt, 5, 1, 4, 3) < MIN_DIST2:                                           <L 1116>
    var_166 = _attach_face_0(var_pt, var_162, var_163, var_164, var_165);
    var_167 = (var_166 < var_111);
    if (var_167) {
        // pt.status = -1                                                                     <L 1117>
        var_pt.status = var_168;
        // return pt, _replace_simplex3(pt, 1, 4, 3)                                          <L 1118>
        var_172 = _replace_simplex3_0(var_pt, var_169, var_170, var_171);
        ret_0 = var_pt;
        ret_1 = var_172;
        return;
    }
    // v2 = pt.vert[4] - pt.vert[5]                                                           <L 1121>
    var_173 = &((var_pt).vert);
    var_176 = wp::load(var_173);
    var_175 = wp::address(var_176, var_174);
    var_177 = &((var_pt).vert);
    var_180 = wp::load(var_177);
    var_179 = wp::address(var_180, var_178);
    var_182 = wp::load(var_175);
    var_183 = wp::load(var_179);
    var_181 = wp::sub(var_182, var_183);
    // v3 = pt.vert[6] - pt.vert[7]                                                           <L 1122>
    var_184 = &((var_pt).vert);
    var_187 = wp::load(var_184);
    var_186 = wp::address(var_187, var_185);
    var_188 = &((var_pt).vert);
    var_191 = wp::load(var_188);
    var_190 = wp::address(var_191, var_189);
    var_193 = wp::load(var_186);
    var_194 = wp::load(var_190);
    var_192 = wp::sub(var_193, var_194);
    // v4 = pt.vert[8] - pt.vert[9]                                                           <L 1123>
    var_195 = &((var_pt).vert);
    var_198 = wp::load(var_195);
    var_197 = wp::address(var_198, var_196);
    var_199 = &((var_pt).vert);
    var_202 = wp::load(var_199);
    var_201 = wp::address(var_202, var_200);
    var_204 = wp::load(var_197);
    var_205 = wp::load(var_201);
    var_203 = wp::sub(var_204, var_205);
    // if not _ray_triangle(simplex[0], simplex[1], v2, v3, v4):                              <L 1124>
    var_207 = wp::extract(var_simplex, var_206);
    var_209 = wp::extract(var_simplex, var_208);
    var_210 = _ray_triangle_0(var_207, var_209, var_181, var_192, var_203);
    var_211 = wp::unot(var_210);
    if (var_211) {
        // pt.status = 1                                                                      <L 1125>
        var_pt.status = var_212;
        // return pt, GJKResult()                                                             <L 1126>
        var_213 = GJKResult_8184e255();
        ret_0 = var_pt;
        ret_1 = var_213;
        return;
    }
    // pt.nvert = 5                                                                           <L 1129>
    var_pt.nvert = var_214;
    // pt.nface = 6                                                                           <L 1130>
    var_pt.nface = var_215;
    // pt.status = 0                                                                          <L 1131>
    var_pt.status = var_216;
    // return pt, GJKResult()                                                                 <L 1132>
    var_217 = GJKResult_8184e255();
    ret_0 = var_pt;
    ret_1 = var_217;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:794
static CUDA_CALLABLE bool _same_side_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_p3)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::float32 var_4;
    wp::vec_t<3, wp::float32> var_5;
    wp::float32 var_6;
    bool var_7;
    bool var_8;
    const wp::float32 var_9 = 0.0;
    bool var_10;
    const wp::float32 var_11 = 0.0;
    bool var_12;
    bool var_13;
    const wp::float32 var_14 = 0.0;
    bool var_15;
    const wp::float32 var_16 = 0.0;
    bool var_17;
    //---------
    // forward
    // def _same_side(p0: wp.vec3, p1: wp.vec3, p2: wp.vec3, p3: wp.vec3) -> bool:            <L 795>
    // n = wp.cross(p1 - p0, p2 - p0)                                                         <L 796>
    var_0 = wp::sub(var_p1, var_p0);
    var_1 = wp::sub(var_p2, var_p0);
    var_2 = wp::cross(var_0, var_1);
    // dot1 = wp.dot(n, p3 - p0)                                                              <L 797>
    var_3 = wp::sub(var_p3, var_p0);
    var_4 = wp::dot(var_2, var_3);
    // dot2 = wp.dot(n, -p0)                                                                  <L 798>
    var_5 = wp::neg(var_p0);
    var_6 = wp::dot(var_2, var_5);
    // return (dot1 > 0.0 and dot2 > 0.0) or (dot1 < 0.0 and dot2 < 0.0)                      <L 799>
    var_10 = (var_4 > var_9);
    var_8 = var_10;
    if (var_8) {
        var_12 = (var_6 > var_11);
        var_8 = var_8 && var_12;
    }
    var_7 = var_8;
    if (!var_7) {
        var_15 = (var_4 < var_14);
        var_13 = var_15;
        if (var_13) {
            var_17 = (var_6 < var_16);
            var_13 = var_13 && var_17;
        }
        var_7 = var_7 || var_13;
    }
    return var_7;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:802
static CUDA_CALLABLE bool _test_tetra_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_p3)
{
    //---------
    // primal vars
    bool var_0;
    bool var_1;
    bool var_2;
    bool var_3;
    bool var_4;
    //---------
    // forward
    // def _test_tetra(p0: wp.vec3, p1: wp.vec3, p2: wp.vec3, p3: wp.vec3) -> bool:           <L 803>
    // return _same_side(p0, p1, p2, p3) and _same_side(p1, p2, p3, p0) and _same_side(p2, p3, p0, p1) and _same_side(p3, p0, p1, p2)       <L 804>
    var_1 = _same_side_0(var_p0, var_p1, var_p2, var_p3);
    var_0 = var_1;
    if (var_0) {
        var_2 = _same_side_0(var_p1, var_p2, var_p3, var_p0);
        var_0 = var_0 && var_2;
    }
    if (var_0) {
        var_3 = _same_side_0(var_p2, var_p3, var_p0, var_p1);
        var_0 = var_0 && var_3;
    }
    if (var_0) {
        var_4 = _same_side_0(var_p3, var_p0, var_p1, var_p2);
        var_0 = var_0 && var_4;
    }
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1228
static CUDA_CALLABLE void _polytope4_0(
    Polytope_10582b13 var_pt,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Polytope_10582b13 & ret_0,
    GJKResult_8184e255 & ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.25;
    const wp::int32 var_1 = 0;
    wp::vec_t<3, wp::float32> var_2;
    const wp::int32 var_3 = 1;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    const wp::int32 var_6 = 2;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    const wp::int32 var_9 = 3;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    const wp::int32 var_13 = 0;
    wp::vec_t<3, wp::float32> var_14;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_15;
    const wp::int32 var_16 = 0;
    wp::array_t<wp::vec_t<3, wp::float32>> var_17;
    const wp::int32 var_18 = 0;
    wp::vec_t<3, wp::float32> var_19;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_20;
    const wp::int32 var_21 = 1;
    wp::array_t<wp::vec_t<3, wp::float32>> var_22;
    const wp::int32 var_23 = 1;
    wp::vec_t<3, wp::float32> var_24;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_25;
    const wp::int32 var_26 = 2;
    wp::array_t<wp::vec_t<3, wp::float32>> var_27;
    const wp::int32 var_28 = 1;
    wp::vec_t<3, wp::float32> var_29;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_30;
    const wp::int32 var_31 = 3;
    wp::array_t<wp::vec_t<3, wp::float32>> var_32;
    const wp::int32 var_33 = 2;
    wp::vec_t<3, wp::float32> var_34;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_35;
    const wp::int32 var_36 = 4;
    wp::array_t<wp::vec_t<3, wp::float32>> var_37;
    const wp::int32 var_38 = 2;
    wp::vec_t<3, wp::float32> var_39;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_40;
    const wp::int32 var_41 = 5;
    wp::array_t<wp::vec_t<3, wp::float32>> var_42;
    const wp::int32 var_43 = 3;
    wp::vec_t<3, wp::float32> var_44;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_45;
    const wp::int32 var_46 = 6;
    wp::array_t<wp::vec_t<3, wp::float32>> var_47;
    const wp::int32 var_48 = 3;
    wp::vec_t<3, wp::float32> var_49;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_50;
    const wp::int32 var_51 = 7;
    wp::array_t<wp::vec_t<3, wp::float32>> var_52;
    const wp::int32 var_53 = 0;
    wp::int32 var_54;
    wp::array_t<wp::int32>* var_55;
    const wp::int32 var_56 = 0;
    wp::array_t<wp::int32> var_57;
    const wp::int32 var_58 = 0;
    wp::int32 var_59;
    wp::array_t<wp::int32>* var_60;
    const wp::int32 var_61 = 1;
    wp::array_t<wp::int32> var_62;
    const wp::int32 var_63 = 1;
    wp::int32 var_64;
    wp::array_t<wp::int32>* var_65;
    const wp::int32 var_66 = 2;
    wp::array_t<wp::int32> var_67;
    const wp::int32 var_68 = 1;
    wp::int32 var_69;
    wp::array_t<wp::int32>* var_70;
    const wp::int32 var_71 = 3;
    wp::array_t<wp::int32> var_72;
    const wp::int32 var_73 = 2;
    wp::int32 var_74;
    wp::array_t<wp::int32>* var_75;
    const wp::int32 var_76 = 4;
    wp::array_t<wp::int32> var_77;
    const wp::int32 var_78 = 2;
    wp::int32 var_79;
    wp::array_t<wp::int32>* var_80;
    const wp::int32 var_81 = 5;
    wp::array_t<wp::int32> var_82;
    const wp::int32 var_83 = 3;
    wp::int32 var_84;
    wp::array_t<wp::int32>* var_85;
    const wp::int32 var_86 = 6;
    wp::array_t<wp::int32> var_87;
    const wp::int32 var_88 = 3;
    wp::int32 var_89;
    wp::array_t<wp::int32>* var_90;
    const wp::int32 var_91 = 7;
    wp::array_t<wp::int32> var_92;
    wp::vec_t<4, wp::float32> var_93;
    const wp::int32 var_94 = 0;
    wp::int32 var_95;
    const wp::int32 var_96 = 0;
    const wp::int32 var_97 = 0;
    const wp::int32 var_98 = 1;
    const wp::int32 var_99 = 2;
    wp::float32 var_100;
    const wp::int32 var_101 = 0;
    const wp::int32 var_102 = 0;
    wp::float32 var_103;
    const wp::float32 var_104 = 1e-17;
    bool var_105;
    const wp::int32 var_106 = -1;
    const wp::int32 var_107 = 0;
    const wp::int32 var_108 = 1;
    const wp::int32 var_109 = 2;
    GJKResult_8184e255 var_110;
    const wp::int32 var_111 = 1;
    const wp::int32 var_112 = 0;
    const wp::int32 var_113 = 3;
    const wp::int32 var_114 = 1;
    wp::float32 var_115;
    const wp::int32 var_116 = 1;
    const wp::int32 var_117 = 1;
    wp::float32 var_118;
    bool var_119;
    const wp::int32 var_120 = -1;
    const wp::int32 var_121 = 0;
    const wp::int32 var_122 = 3;
    const wp::int32 var_123 = 1;
    GJKResult_8184e255 var_124;
    const wp::int32 var_125 = 0;
    wp::float32 var_126;
    const wp::int32 var_127 = 1;
    wp::float32 var_128;
    bool var_129;
    const wp::int32 var_130 = 0;
    const wp::int32 var_131 = 1;
    wp::int32 var_132;
    const wp::int32 var_133 = 2;
    const wp::int32 var_134 = 0;
    const wp::int32 var_135 = 2;
    const wp::int32 var_136 = 3;
    wp::float32 var_137;
    const wp::int32 var_138 = 2;
    const wp::int32 var_139 = 2;
    wp::float32 var_140;
    bool var_141;
    const wp::int32 var_142 = -1;
    const wp::int32 var_143 = 0;
    const wp::int32 var_144 = 2;
    const wp::int32 var_145 = 3;
    GJKResult_8184e255 var_146;
    const wp::int32 var_147 = 2;
    wp::float32 var_148;
    wp::float32 var_149;
    bool var_150;
    const wp::int32 var_151 = 2;
    wp::int32 var_152;
    const wp::int32 var_153 = 3;
    const wp::int32 var_154 = 3;
    const wp::int32 var_155 = 2;
    const wp::int32 var_156 = 1;
    wp::float32 var_157;
    const wp::int32 var_158 = 3;
    const wp::int32 var_159 = 3;
    wp::float32 var_160;
    bool var_161;
    const wp::int32 var_162 = -1;
    const wp::int32 var_163 = 3;
    const wp::int32 var_164 = 2;
    const wp::int32 var_165 = 1;
    GJKResult_8184e255 var_166;
    const wp::int32 var_167 = 3;
    wp::float32 var_168;
    wp::float32 var_169;
    bool var_170;
    const wp::int32 var_171 = 3;
    wp::int32 var_172;
    const wp::int32 var_173 = 0;
    wp::vec_t<3, wp::float32> var_174;
    const wp::int32 var_175 = 1;
    wp::vec_t<3, wp::float32> var_176;
    const wp::int32 var_177 = 2;
    wp::vec_t<3, wp::float32> var_178;
    const wp::int32 var_179 = 3;
    wp::vec_t<3, wp::float32> var_180;
    bool var_181;
    bool var_182;
    wp::float32 var_183;
    const wp::float32 var_184 = 1e-15;
    bool var_185;
    const wp::int32 var_186 = 12;
    GJKResult_8184e255 var_187;
    const wp::int32 var_188 = -1;
    const wp::int32 var_189 = 0;
    bool var_190;
    const wp::int32 var_191 = 0;
    const wp::int32 var_192 = 1;
    const wp::int32 var_193 = 2;
    GJKResult_8184e255 var_194;
    const wp::int32 var_195 = 1;
    bool var_196;
    const wp::int32 var_197 = 0;
    const wp::int32 var_198 = 3;
    const wp::int32 var_199 = 1;
    GJKResult_8184e255 var_200;
    const wp::int32 var_201 = 2;
    bool var_202;
    const wp::int32 var_203 = 0;
    const wp::int32 var_204 = 2;
    const wp::int32 var_205 = 3;
    GJKResult_8184e255 var_206;
    const wp::int32 var_207 = 3;
    const wp::int32 var_208 = 2;
    const wp::int32 var_209 = 1;
    GJKResult_8184e255 var_210;
    const wp::int32 var_211 = 4;
    const wp::int32 var_212 = 4;
    const wp::int32 var_213 = 0;
    GJKResult_8184e255 var_214;
    //---------
    // forward
    // def _polytope4(                                                                        <L 1229>
    // pt.center = 0.25 * (simplex[0] + simplex[1] + simplex[2] + simplex[3])                 <L 1240>
    var_2 = wp::extract(var_simplex, var_1);
    var_4 = wp::extract(var_simplex, var_3);
    var_5 = wp::add(var_2, var_4);
    var_7 = wp::extract(var_simplex, var_6);
    var_8 = wp::add(var_5, var_7);
    var_10 = wp::extract(var_simplex, var_9);
    var_11 = wp::add(var_8, var_10);
    var_12 = wp::mul(var_0, var_11);
    var_pt.center = var_12;
    // pt.vert[0] = simplex1[0]                                                               <L 1242>
    var_14 = wp::extract(var_simplex1, var_13);
    var_15 = &((var_pt).vert);
    var_17 = wp::load(var_15);
    wp::array_store(var_17, var_16, var_14);
    // pt.vert[1] = simplex2[0]                                                               <L 1243>
    var_19 = wp::extract(var_simplex2, var_18);
    var_20 = &((var_pt).vert);
    var_22 = wp::load(var_20);
    wp::array_store(var_22, var_21, var_19);
    // pt.vert[2] = simplex1[1]                                                               <L 1244>
    var_24 = wp::extract(var_simplex1, var_23);
    var_25 = &((var_pt).vert);
    var_27 = wp::load(var_25);
    wp::array_store(var_27, var_26, var_24);
    // pt.vert[3] = simplex2[1]                                                               <L 1245>
    var_29 = wp::extract(var_simplex2, var_28);
    var_30 = &((var_pt).vert);
    var_32 = wp::load(var_30);
    wp::array_store(var_32, var_31, var_29);
    // pt.vert[4] = simplex1[2]                                                               <L 1246>
    var_34 = wp::extract(var_simplex1, var_33);
    var_35 = &((var_pt).vert);
    var_37 = wp::load(var_35);
    wp::array_store(var_37, var_36, var_34);
    // pt.vert[5] = simplex2[2]                                                               <L 1247>
    var_39 = wp::extract(var_simplex2, var_38);
    var_40 = &((var_pt).vert);
    var_42 = wp::load(var_40);
    wp::array_store(var_42, var_41, var_39);
    // pt.vert[6] = simplex1[3]                                                               <L 1248>
    var_44 = wp::extract(var_simplex1, var_43);
    var_45 = &((var_pt).vert);
    var_47 = wp::load(var_45);
    wp::array_store(var_47, var_46, var_44);
    // pt.vert[7] = simplex2[3]                                                               <L 1249>
    var_49 = wp::extract(var_simplex2, var_48);
    var_50 = &((var_pt).vert);
    var_52 = wp::load(var_50);
    wp::array_store(var_52, var_51, var_49);
    // pt.vert_index[0] = simplex_index1[0]                                                   <L 1251>
    var_54 = wp::extract(var_simplex_index1, var_53);
    var_55 = &((var_pt).vert_index);
    var_57 = wp::load(var_55);
    wp::array_store(var_57, var_56, var_54);
    // pt.vert_index[1] = simplex_index2[0]                                                   <L 1252>
    var_59 = wp::extract(var_simplex_index2, var_58);
    var_60 = &((var_pt).vert_index);
    var_62 = wp::load(var_60);
    wp::array_store(var_62, var_61, var_59);
    // pt.vert_index[2] = simplex_index1[1]                                                   <L 1253>
    var_64 = wp::extract(var_simplex_index1, var_63);
    var_65 = &((var_pt).vert_index);
    var_67 = wp::load(var_65);
    wp::array_store(var_67, var_66, var_64);
    // pt.vert_index[3] = simplex_index2[1]                                                   <L 1254>
    var_69 = wp::extract(var_simplex_index2, var_68);
    var_70 = &((var_pt).vert_index);
    var_72 = wp::load(var_70);
    wp::array_store(var_72, var_71, var_69);
    // pt.vert_index[4] = simplex_index1[2]                                                   <L 1255>
    var_74 = wp::extract(var_simplex_index1, var_73);
    var_75 = &((var_pt).vert_index);
    var_77 = wp::load(var_75);
    wp::array_store(var_77, var_76, var_74);
    // pt.vert_index[5] = simplex_index2[2]                                                   <L 1256>
    var_79 = wp::extract(var_simplex_index2, var_78);
    var_80 = &((var_pt).vert_index);
    var_82 = wp::load(var_80);
    wp::array_store(var_82, var_81, var_79);
    // pt.vert_index[6] = simplex_index1[3]                                                   <L 1257>
    var_84 = wp::extract(var_simplex_index1, var_83);
    var_85 = &((var_pt).vert_index);
    var_87 = wp::load(var_85);
    wp::array_store(var_87, var_86, var_84);
    // pt.vert_index[7] = simplex_index2[3]                                                   <L 1258>
    var_89 = wp::extract(var_simplex_index2, var_88);
    var_90 = &((var_pt).vert_index);
    var_92 = wp::load(var_90);
    wp::array_store(var_92, var_91, var_89);
    // dist = wp.vec4()                                                                       <L 1260>
    var_93 = wp::vec_t<4, wp::float32>();
    // idx = int(0)                                                                           <L 1261>
    var_95 = wp::int(var_94);
    // dist[0] = _attach_face(pt, 0, 0, 1, 2)                                                 <L 1264>
    var_100 = _attach_face_0(var_pt, var_96, var_97, var_98, var_99);
    wp::assign_inplace(var_93, var_101, var_100);
    // if dist[0] < MIN_DIST4:                                                                <L 1265>
    var_103 = wp::extract(var_93, var_102);
    var_105 = (var_103 < var_104);
    if (var_105) {
        // pt.status = -1                                                                     <L 1266>
        var_pt.status = var_106;
        // return pt, _replace_simplex3(pt, 0, 1, 2)                                          <L 1267>
        var_110 = _replace_simplex3_0(var_pt, var_107, var_108, var_109);
        ret_0 = var_pt;
        ret_1 = var_110;
        return;
    }
    // dist[1] = _attach_face(pt, 1, 0, 3, 1)                                                 <L 1269>
    var_115 = _attach_face_0(var_pt, var_111, var_112, var_113, var_114);
    wp::assign_inplace(var_93, var_116, var_115);
    // if dist[1] < MIN_DIST4:                                                                <L 1270>
    var_118 = wp::extract(var_93, var_117);
    var_119 = (var_118 < var_104);
    if (var_119) {
        // pt.status = -1                                                                     <L 1271>
        var_pt.status = var_120;
        // return pt, _replace_simplex3(pt, 0, 3, 1)                                          <L 1272>
        var_124 = _replace_simplex3_0(var_pt, var_121, var_122, var_123);
        ret_0 = var_pt;
        ret_1 = var_124;
        return;
    }
    // idx = wp.where(dist[0] < dist[1], 0, 1)                                                <L 1273>
    var_126 = wp::extract(var_93, var_125);
    var_128 = wp::extract(var_93, var_127);
    var_129 = (var_126 < var_128);
    var_132 = wp::where(var_129, var_130, var_131);
    // dist[2] = _attach_face(pt, 2, 0, 2, 3)                                                 <L 1275>
    var_137 = _attach_face_0(var_pt, var_133, var_134, var_135, var_136);
    wp::assign_inplace(var_93, var_138, var_137);
    // if dist[2] < MIN_DIST4:                                                                <L 1276>
    var_140 = wp::extract(var_93, var_139);
    var_141 = (var_140 < var_104);
    if (var_141) {
        // pt.status = -1                                                                     <L 1277>
        var_pt.status = var_142;
        // return pt, _replace_simplex3(pt, 0, 2, 3)                                          <L 1278>
        var_146 = _replace_simplex3_0(var_pt, var_143, var_144, var_145);
        ret_0 = var_pt;
        ret_1 = var_146;
        return;
    }
    // idx = wp.where(dist[2] < dist[idx], 2, idx)                                            <L 1279>
    var_148 = wp::extract(var_93, var_147);
    var_149 = wp::extract(var_93, var_132);
    var_150 = (var_148 < var_149);
    var_152 = wp::where(var_150, var_151, var_132);
    // dist[3] = _attach_face(pt, 3, 3, 2, 1)                                                 <L 1281>
    var_157 = _attach_face_0(var_pt, var_153, var_154, var_155, var_156);
    wp::assign_inplace(var_93, var_158, var_157);
    // if dist[3] < MIN_DIST4:                                                                <L 1282>
    var_160 = wp::extract(var_93, var_159);
    var_161 = (var_160 < var_104);
    if (var_161) {
        // pt.status = -1                                                                     <L 1283>
        var_pt.status = var_162;
        // return pt, _replace_simplex3(pt, 3, 2, 1)                                          <L 1284>
        var_166 = _replace_simplex3_0(var_pt, var_163, var_164, var_165);
        ret_0 = var_pt;
        ret_1 = var_166;
        return;
    }
    // idx = wp.where(dist[3] < dist[idx], 3, idx)                                            <L 1285>
    var_168 = wp::extract(var_93, var_167);
    var_169 = wp::extract(var_93, var_152);
    var_170 = (var_168 < var_169);
    var_172 = wp::where(var_170, var_171, var_152);
    // if not _test_tetra(simplex[0], simplex[1], simplex[2], simplex[3]):                    <L 1287>
    var_174 = wp::extract(var_simplex, var_173);
    var_176 = wp::extract(var_simplex, var_175);
    var_178 = wp::extract(var_simplex, var_177);
    var_180 = wp::extract(var_simplex, var_179);
    var_181 = _test_tetra_0(var_174, var_176, var_178, var_180);
    var_182 = wp::unot(var_181);
    if (var_182) {
        // if dist[idx] > MINVAL:                                                             <L 1288>
        var_183 = wp::extract(var_93, var_172);
        var_185 = (var_183 > var_184);
        if (var_185) {
            // pt.status = 12                                                                 <L 1289>
            var_pt.status = var_186;
            // return pt, GJKResult()                                                         <L 1290>
            var_187 = GJKResult_8184e255();
            ret_0 = var_pt;
            ret_1 = var_187;
            return;
        }
        // pt.status = -1                                                                     <L 1293>
        var_pt.status = var_188;
        // if idx == 0:                                                                       <L 1294>
        var_190 = (var_172 == var_189);
        if (var_190) {
            // return pt, _replace_simplex3(pt, 0, 1, 2)                                      <L 1295>
            var_194 = _replace_simplex3_0(var_pt, var_191, var_192, var_193);
            ret_0 = var_pt;
            ret_1 = var_194;
            return;
        }
        if (!var_190) {
            // elif idx == 1:                                                                 <L 1296>
            var_196 = (var_172 == var_195);
            if (var_196) {
                // return pt, _replace_simplex3(pt, 0, 3, 1)                                  <L 1297>
                var_200 = _replace_simplex3_0(var_pt, var_197, var_198, var_199);
                ret_0 = var_pt;
                ret_1 = var_200;
                return;
            }
            if (!var_196) {
                // elif idx == 2:                                                             <L 1298>
                var_202 = (var_172 == var_201);
                if (var_202) {
                    // return pt, _replace_simplex3(pt, 0, 2, 3)                              <L 1299>
                    var_206 = _replace_simplex3_0(var_pt, var_203, var_204, var_205);
                    ret_0 = var_pt;
                    ret_1 = var_206;
                    return;
                }
                if (!var_202) {
                    // return pt, _replace_simplex3(pt, 3, 2, 1)                              <L 1301>
                    var_210 = _replace_simplex3_0(var_pt, var_207, var_208, var_209);
                    ret_0 = var_pt;
                    ret_1 = var_210;
                    return;
                }
            }
        }
    }
    // pt.nvert = 4                                                                           <L 1304>
    var_pt.nvert = var_211;
    // pt.nface = 4                                                                           <L 1305>
    var_pt.nface = var_212;
    // pt.status = 0                                                                          <L 1306>
    var_pt.status = var_213;
    // return pt, GJKResult()                                                                 <L 1307>
    var_214 = GJKResult_8184e255();
    ret_0 = var_pt;
    ret_1 = var_214;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:850
static CUDA_CALLABLE bool _tri_point_intersect_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_p)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    const wp::int32 var_3 = 1;
    wp::float32 var_4;
    const wp::int32 var_5 = 2;
    wp::float32 var_6;
    bool var_7;
    const wp::float32 var_8 = 0.0;
    bool var_9;
    const wp::float32 var_10 = 0.0;
    bool var_11;
    const wp::float32 var_12 = 0.0;
    bool var_13;
    const bool var_14 = false;
    wp::vec_t<3, wp::float32> var_15;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    wp::float32 var_18;
    const wp::int32 var_19 = 0;
    wp::float32 var_20;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 0;
    wp::float32 var_24;
    wp::float32 var_25;
    wp::float32 var_26;
    const wp::int32 var_27 = 0;
    const wp::int32 var_28 = 1;
    wp::float32 var_29;
    wp::float32 var_30;
    const wp::int32 var_31 = 1;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::float32 var_34;
    const wp::int32 var_35 = 1;
    wp::float32 var_36;
    wp::float32 var_37;
    wp::float32 var_38;
    const wp::int32 var_39 = 1;
    const wp::int32 var_40 = 2;
    wp::float32 var_41;
    wp::float32 var_42;
    const wp::int32 var_43 = 2;
    wp::float32 var_44;
    wp::float32 var_45;
    wp::float32 var_46;
    const wp::int32 var_47 = 2;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::int32 var_51 = 2;
    wp::vec_t<3, wp::float32> var_52;
    wp::float32 var_53;
    const wp::float32 var_54 = 1e-15;
    bool var_55;
    //---------
    // forward
    // def _tri_point_intersect(v1: wp.vec3, v2: wp.vec3, v3: wp.vec3, p: wp.vec3) -> bool:       <L 851>
    // coordinates = _tri_affine_coord(v1, v2, v3, p)                                         <L 852>
    var_0 = _tri_affine_coord_0(var_v1, var_v2, var_v3, var_p);
    // l1 = coordinates[0]                                                                    <L 853>
    var_2 = wp::extract(var_0, var_1);
    // l2 = coordinates[1]                                                                    <L 854>
    var_4 = wp::extract(var_0, var_3);
    // l3 = coordinates[2]                                                                    <L 855>
    var_6 = wp::extract(var_0, var_5);
    // if l1 < 0.0 or l2 < 0.0 or l3 < 0.0:                                                   <L 857>
    var_9 = (var_2 < var_8);
    var_7 = var_9;
    if (!var_7) {
        var_11 = (var_4 < var_10);
        var_7 = var_7 || var_11;
    }
    if (!var_7) {
        var_13 = (var_6 < var_12);
        var_7 = var_7 || var_13;
    }
    if (var_7) {
        // return False                                                                       <L 858>
        return var_14;
    }
    // pr = wp.vec3()                                                                         <L 860>
    var_15 = wp::vec_t<3, wp::float32>();
    // pr[0] = v1[0] * l1 + v2[0] * l2 + v3[0] * l3                                           <L 861>
    var_17 = wp::extract(var_v1, var_16);
    var_18 = wp::mul(var_17, var_2);
    var_20 = wp::extract(var_v2, var_19);
    var_21 = wp::mul(var_20, var_4);
    var_22 = wp::add(var_18, var_21);
    var_24 = wp::extract(var_v3, var_23);
    var_25 = wp::mul(var_24, var_6);
    var_26 = wp::add(var_22, var_25);
    wp::assign_inplace(var_15, var_27, var_26);
    // pr[1] = v1[1] * l1 + v2[1] * l2 + v3[1] * l3                                           <L 862>
    var_29 = wp::extract(var_v1, var_28);
    var_30 = wp::mul(var_29, var_2);
    var_32 = wp::extract(var_v2, var_31);
    var_33 = wp::mul(var_32, var_4);
    var_34 = wp::add(var_30, var_33);
    var_36 = wp::extract(var_v3, var_35);
    var_37 = wp::mul(var_36, var_6);
    var_38 = wp::add(var_34, var_37);
    wp::assign_inplace(var_15, var_39, var_38);
    // pr[2] = v1[2] * l1 + v2[2] * l2 + v3[2] * l3                                           <L 863>
    var_41 = wp::extract(var_v1, var_40);
    var_42 = wp::mul(var_41, var_2);
    var_44 = wp::extract(var_v2, var_43);
    var_45 = wp::mul(var_44, var_4);
    var_46 = wp::add(var_42, var_45);
    var_48 = wp::extract(var_v3, var_47);
    var_49 = wp::mul(var_48, var_6);
    var_50 = wp::add(var_46, var_49);
    wp::assign_inplace(var_15, var_51, var_50);
    // return wp.norm_l2(pr - p) < MINVAL                                                     <L 864>
    var_52 = wp::sub(var_15, var_p);
    var_53 = norm_l2_0(var_52);
    var_55 = (var_53 < var_54);
    return var_55;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1135
static CUDA_CALLABLE Polytope_10582b13 _polytope3_0(
    Polytope_10582b13 var_pt,
    wp::float32 var_dist,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::vec_t<3, wp::float32> var_1;
    const wp::int32 var_2 = 1;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    const wp::int32 var_5 = 2;
    wp::vec_t<3, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    const wp::float32 var_8 = 0.3333333333333333;
    wp::vec_t<3, wp::float32> var_9;
    const wp::int32 var_10 = 1;
    wp::vec_t<3, wp::float32> var_11;
    const wp::int32 var_12 = 0;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    const wp::int32 var_15 = 2;
    wp::vec_t<3, wp::float32> var_16;
    const wp::int32 var_17 = 0;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::float32 var_21;
    const wp::float32 var_22 = 1e-15;
    bool var_23;
    const wp::int32 var_24 = 2;
    wp::vec_t<3, wp::float32> var_25;
    const wp::int32 var_26 = 0;
    wp::vec_t<3, wp::float32> var_27;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_28;
    const wp::int32 var_29 = 0;
    wp::array_t<wp::vec_t<3, wp::float32>> var_30;
    const wp::int32 var_31 = 0;
    wp::vec_t<3, wp::float32> var_32;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_33;
    const wp::int32 var_34 = 1;
    wp::array_t<wp::vec_t<3, wp::float32>> var_35;
    const wp::int32 var_36 = 1;
    wp::vec_t<3, wp::float32> var_37;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_38;
    const wp::int32 var_39 = 2;
    wp::array_t<wp::vec_t<3, wp::float32>> var_40;
    const wp::int32 var_41 = 1;
    wp::vec_t<3, wp::float32> var_42;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_43;
    const wp::int32 var_44 = 3;
    wp::array_t<wp::vec_t<3, wp::float32>> var_45;
    const wp::int32 var_46 = 2;
    wp::vec_t<3, wp::float32> var_47;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_48;
    const wp::int32 var_49 = 4;
    wp::array_t<wp::vec_t<3, wp::float32>> var_50;
    const wp::int32 var_51 = 2;
    wp::vec_t<3, wp::float32> var_52;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_53;
    const wp::int32 var_54 = 5;
    wp::array_t<wp::vec_t<3, wp::float32>> var_55;
    const wp::int32 var_56 = 0;
    wp::int32 var_57;
    wp::array_t<wp::int32>* var_58;
    const wp::int32 var_59 = 0;
    wp::array_t<wp::int32> var_60;
    const wp::int32 var_61 = 0;
    wp::int32 var_62;
    wp::array_t<wp::int32>* var_63;
    const wp::int32 var_64 = 1;
    wp::array_t<wp::int32> var_65;
    const wp::int32 var_66 = 1;
    wp::int32 var_67;
    wp::array_t<wp::int32>* var_68;
    const wp::int32 var_69 = 2;
    wp::array_t<wp::int32> var_70;
    const wp::int32 var_71 = 1;
    wp::int32 var_72;
    wp::array_t<wp::int32>* var_73;
    const wp::int32 var_74 = 3;
    wp::array_t<wp::int32> var_75;
    const wp::int32 var_76 = 2;
    wp::int32 var_77;
    wp::array_t<wp::int32>* var_78;
    const wp::int32 var_79 = 4;
    wp::array_t<wp::int32> var_80;
    const wp::int32 var_81 = 2;
    wp::int32 var_82;
    wp::array_t<wp::int32>* var_83;
    const wp::int32 var_84 = 5;
    wp::array_t<wp::int32> var_85;
    const wp::int32 var_86 = 3;
    wp::vec_t<3, wp::float32> var_87;
    wp::int32 var_88;
    wp::int32 var_89;
    const wp::int32 var_90 = 4;
    wp::int32 var_91;
    wp::int32 var_92;
    const wp::int32 var_93 = 0;
    wp::vec_t<3, wp::float32> var_94;
    const wp::int32 var_95 = 1;
    wp::vec_t<3, wp::float32> var_96;
    const wp::int32 var_97 = 2;
    wp::vec_t<3, wp::float32> var_98;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_99;
    const wp::int32 var_100 = 6;
    wp::vec_t<3, wp::float32>* var_101;
    wp::array_t<wp::vec_t<3, wp::float32>> var_102;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_103;
    const wp::int32 var_104 = 7;
    wp::vec_t<3, wp::float32>* var_105;
    wp::array_t<wp::vec_t<3, wp::float32>> var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32> var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_110;
    const wp::int32 var_111 = 8;
    wp::vec_t<3, wp::float32>* var_112;
    wp::array_t<wp::vec_t<3, wp::float32>> var_113;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_114;
    const wp::int32 var_115 = 9;
    wp::vec_t<3, wp::float32>* var_116;
    wp::array_t<wp::vec_t<3, wp::float32>> var_117;
    wp::vec_t<3, wp::float32> var_118;
    wp::vec_t<3, wp::float32> var_119;
    wp::vec_t<3, wp::float32> var_120;
    bool var_121;
    const wp::int32 var_122 = 3;
    bool var_123;
    const wp::int32 var_124 = 4;
    bool var_125;
    const wp::float32 var_126 = 1e-05;
    bool var_127;
    bool var_128;
    bool var_129;
    bool var_130;
    bool var_131;
    const wp::int32 var_132 = 5;
    const wp::int32 var_133 = 0;
    const wp::int32 var_134 = 4;
    const wp::int32 var_135 = 0;
    const wp::int32 var_136 = 1;
    wp::float32 var_137;
    const wp::float32 var_138 = 1e-10;
    bool var_139;
    const wp::int32 var_140 = 6;
    const wp::int32 var_141 = 1;
    const wp::int32 var_142 = 4;
    const wp::int32 var_143 = 2;
    const wp::int32 var_144 = 0;
    wp::float32 var_145;
    bool var_146;
    const wp::int32 var_147 = 7;
    const wp::int32 var_148 = 2;
    const wp::int32 var_149 = 4;
    const wp::int32 var_150 = 1;
    const wp::int32 var_151 = 2;
    wp::float32 var_152;
    bool var_153;
    const wp::int32 var_154 = 8;
    const wp::int32 var_155 = 3;
    const wp::int32 var_156 = 3;
    const wp::int32 var_157 = 1;
    const wp::int32 var_158 = 0;
    wp::float32 var_159;
    bool var_160;
    const wp::int32 var_161 = 9;
    const wp::int32 var_162 = 4;
    const wp::int32 var_163 = 3;
    const wp::int32 var_164 = 0;
    const wp::int32 var_165 = 2;
    wp::float32 var_166;
    bool var_167;
    const wp::int32 var_168 = 10;
    const wp::int32 var_169 = 5;
    const wp::int32 var_170 = 3;
    const wp::int32 var_171 = 2;
    const wp::int32 var_172 = 1;
    wp::float32 var_173;
    bool var_174;
    const wp::int32 var_175 = 11;
    const wp::int32 var_176 = 5;
    const wp::int32 var_177 = 6;
    const wp::int32 var_178 = 0;
    //---------
    // forward
    // def _polytope3(                                                                        <L 1136>
    // pt.center = (simplex[0] + simplex[1] + simplex[2]) * wp.static(1.0 / 3.0)              <L 1152>
    var_1 = wp::extract(var_simplex, var_0);
    var_3 = wp::extract(var_simplex, var_2);
    var_4 = wp::add(var_1, var_3);
    var_6 = wp::extract(var_simplex, var_5);
    var_7 = wp::add(var_4, var_6);
    var_9 = wp::mul(var_7, var_8);
    var_pt.center = var_9;
    // n = wp.cross(simplex[1] - simplex[0], simplex[2] - simplex[0])                         <L 1155>
    var_11 = wp::extract(var_simplex, var_10);
    var_13 = wp::extract(var_simplex, var_12);
    var_14 = wp::sub(var_11, var_13);
    var_16 = wp::extract(var_simplex, var_15);
    var_18 = wp::extract(var_simplex, var_17);
    var_19 = wp::sub(var_16, var_18);
    var_20 = wp::cross(var_14, var_19);
    // norm = wp.norm_l2(n)                                                                   <L 1156>
    var_21 = norm_l2_0(var_20);
    // if norm < MINVAL:                                                                      <L 1157>
    var_23 = (var_21 < var_22);
    if (var_23) {
        // pt.status = 2                                                                      <L 1158>
        var_pt.status = var_24;
        // return pt                                                                          <L 1159>
        return var_pt;
    }
    // n = n / norm                                                                           <L 1160>
    var_25 = wp::div(var_20, var_21);
    // pt.vert[0] = simplex1[0]                                                               <L 1162>
    var_27 = wp::extract(var_simplex1, var_26);
    var_28 = &((var_pt).vert);
    var_30 = wp::load(var_28);
    wp::array_store(var_30, var_29, var_27);
    // pt.vert[1] = simplex2[0]                                                               <L 1163>
    var_32 = wp::extract(var_simplex2, var_31);
    var_33 = &((var_pt).vert);
    var_35 = wp::load(var_33);
    wp::array_store(var_35, var_34, var_32);
    // pt.vert[2] = simplex1[1]                                                               <L 1164>
    var_37 = wp::extract(var_simplex1, var_36);
    var_38 = &((var_pt).vert);
    var_40 = wp::load(var_38);
    wp::array_store(var_40, var_39, var_37);
    // pt.vert[3] = simplex2[1]                                                               <L 1165>
    var_42 = wp::extract(var_simplex2, var_41);
    var_43 = &((var_pt).vert);
    var_45 = wp::load(var_43);
    wp::array_store(var_45, var_44, var_42);
    // pt.vert[4] = simplex1[2]                                                               <L 1166>
    var_47 = wp::extract(var_simplex1, var_46);
    var_48 = &((var_pt).vert);
    var_50 = wp::load(var_48);
    wp::array_store(var_50, var_49, var_47);
    // pt.vert[5] = simplex2[2]                                                               <L 1167>
    var_52 = wp::extract(var_simplex2, var_51);
    var_53 = &((var_pt).vert);
    var_55 = wp::load(var_53);
    wp::array_store(var_55, var_54, var_52);
    // pt.vert_index[0] = simplex_index1[0]                                                   <L 1169>
    var_57 = wp::extract(var_simplex_index1, var_56);
    var_58 = &((var_pt).vert_index);
    var_60 = wp::load(var_58);
    wp::array_store(var_60, var_59, var_57);
    // pt.vert_index[1] = simplex_index2[0]                                                   <L 1170>
    var_62 = wp::extract(var_simplex_index2, var_61);
    var_63 = &((var_pt).vert_index);
    var_65 = wp::load(var_63);
    wp::array_store(var_65, var_64, var_62);
    // pt.vert_index[2] = simplex_index1[1]                                                   <L 1171>
    var_67 = wp::extract(var_simplex_index1, var_66);
    var_68 = &((var_pt).vert_index);
    var_70 = wp::load(var_68);
    wp::array_store(var_70, var_69, var_67);
    // pt.vert_index[3] = simplex_index2[1]                                                   <L 1172>
    var_72 = wp::extract(var_simplex_index2, var_71);
    var_73 = &((var_pt).vert_index);
    var_75 = wp::load(var_73);
    wp::array_store(var_75, var_74, var_72);
    // pt.vert_index[4] = simplex_index1[2]                                                   <L 1173>
    var_77 = wp::extract(var_simplex_index1, var_76);
    var_78 = &((var_pt).vert_index);
    var_80 = wp::load(var_78);
    wp::array_store(var_80, var_79, var_77);
    // pt.vert_index[5] = simplex_index2[2]                                                   <L 1174>
    var_82 = wp::extract(var_simplex_index2, var_81);
    var_83 = &((var_pt).vert_index);
    var_85 = wp::load(var_83);
    wp::array_store(var_85, var_84, var_82);
    // _epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2, -n)                            <L 1176>
    var_87 = wp::neg(var_25);
    _epa_support_0(var_pt, var_86, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_87, var_88, var_89);
    // _epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2, n)                             <L 1177>
    _epa_support_0(var_pt, var_90, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_25, var_91, var_92);
    // v1 = simplex[0]                                                                        <L 1179>
    var_94 = wp::extract(var_simplex, var_93);
    // v2 = simplex[1]                                                                        <L 1180>
    var_96 = wp::extract(var_simplex, var_95);
    // v3 = simplex[2]                                                                        <L 1181>
    var_98 = wp::extract(var_simplex, var_97);
    // v4 = pt.vert[6] - pt.vert[7]                                                           <L 1182>
    var_99 = &((var_pt).vert);
    var_102 = wp::load(var_99);
    var_101 = wp::address(var_102, var_100);
    var_103 = &((var_pt).vert);
    var_106 = wp::load(var_103);
    var_105 = wp::address(var_106, var_104);
    var_108 = wp::load(var_101);
    var_109 = wp::load(var_105);
    var_107 = wp::sub(var_108, var_109);
    // v5 = pt.vert[8] - pt.vert[9]                                                           <L 1183>
    var_110 = &((var_pt).vert);
    var_113 = wp::load(var_110);
    var_112 = wp::address(var_113, var_111);
    var_114 = &((var_pt).vert);
    var_117 = wp::load(var_114);
    var_116 = wp::address(var_117, var_115);
    var_119 = wp::load(var_112);
    var_120 = wp::load(var_116);
    var_118 = wp::sub(var_119, var_120);
    // if _tri_point_intersect(v1, v2, v3, v4):                                               <L 1186>
    var_121 = _tri_point_intersect_0(var_94, var_96, var_98, var_107);
    if (var_121) {
        // pt.status = 3                                                                      <L 1187>
        var_pt.status = var_122;
        // return pt                                                                          <L 1188>
        return var_pt;
    }
    // if _tri_point_intersect(v1, v2, v3, v5):                                               <L 1191>
    var_123 = _tri_point_intersect_0(var_94, var_96, var_98, var_118);
    if (var_123) {
        // pt.status = 4                                                                      <L 1192>
        var_pt.status = var_124;
        // return pt                                                                          <L 1193>
        return var_pt;
    }
    // if dist > 1e-5 and not _test_tetra(v1, v2, v3, v4) and not _test_tetra(v1, v2, v3, v5):       <L 1197>
    var_127 = (var_dist > var_126);
    var_125 = var_127;
    if (var_125) {
        var_128 = _test_tetra_0(var_94, var_96, var_98, var_107);
        var_129 = wp::unot(var_128);
        var_125 = var_125 && var_129;
    }
    if (var_125) {
        var_130 = _test_tetra_0(var_94, var_96, var_98, var_118);
        var_131 = wp::unot(var_130);
        var_125 = var_125 && var_131;
    }
    if (var_125) {
        // pt.status = 5                                                                      <L 1198>
        var_pt.status = var_132;
        // return pt                                                                          <L 1199>
        return var_pt;
    }
    // if _attach_face(pt, 0, 4, 0, 1) < MIN_DIST3:                                           <L 1202>
    var_137 = _attach_face_0(var_pt, var_133, var_134, var_135, var_136);
    var_139 = (var_137 < var_138);
    if (var_139) {
        // pt.status = 6                                                                      <L 1203>
        var_pt.status = var_140;
        // return pt                                                                          <L 1204>
        return var_pt;
    }
    // if _attach_face(pt, 1, 4, 2, 0) < MIN_DIST3:                                           <L 1205>
    var_145 = _attach_face_0(var_pt, var_141, var_142, var_143, var_144);
    var_146 = (var_145 < var_138);
    if (var_146) {
        // pt.status = 7                                                                      <L 1206>
        var_pt.status = var_147;
        // return pt                                                                          <L 1207>
        return var_pt;
    }
    // if _attach_face(pt, 2, 4, 1, 2) < MIN_DIST3:                                           <L 1208>
    var_152 = _attach_face_0(var_pt, var_148, var_149, var_150, var_151);
    var_153 = (var_152 < var_138);
    if (var_153) {
        // pt.status = 8                                                                      <L 1209>
        var_pt.status = var_154;
        // return pt                                                                          <L 1210>
        return var_pt;
    }
    // if _attach_face(pt, 3, 3, 1, 0) < MIN_DIST3:                                           <L 1211>
    var_159 = _attach_face_0(var_pt, var_155, var_156, var_157, var_158);
    var_160 = (var_159 < var_138);
    if (var_160) {
        // pt.status = 9                                                                      <L 1212>
        var_pt.status = var_161;
        // return pt                                                                          <L 1213>
        return var_pt;
    }
    // if _attach_face(pt, 4, 3, 0, 2) < MIN_DIST3:                                           <L 1214>
    var_166 = _attach_face_0(var_pt, var_162, var_163, var_164, var_165);
    var_167 = (var_166 < var_138);
    if (var_167) {
        // pt.status = 10                                                                     <L 1215>
        var_pt.status = var_168;
        // return pt                                                                          <L 1216>
        return var_pt;
    }
    // if _attach_face(pt, 5, 3, 2, 1) < MIN_DIST3:                                           <L 1217>
    var_173 = _attach_face_0(var_pt, var_169, var_170, var_171, var_172);
    var_174 = (var_173 < var_138);
    if (var_174) {
        // pt.status = 11                                                                     <L 1218>
        var_pt.status = var_175;
        // return pt                                                                          <L 1219>
        return var_pt;
    }
    // pt.nvert = 5                                                                           <L 1222>
    var_pt.nvert = var_176;
    // pt.nface = 6                                                                           <L 1223>
    var_pt.nface = var_177;
    // pt.status = 0                                                                          <L 1224>
    var_pt.status = var_178;
    // return pt                                                                              <L 1225>
    return var_pt;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1334
static CUDA_CALLABLE bool _is_invalid_face_0(
    wp::int32 var_face)
{
    //---------
    // primal vars
    wp::uint32 var_0;
    const wp::uint32 var_1 = 3221225472u;
    wp::uint32 var_2;
    bool var_3;
    //---------
    // forward
    // def _is_invalid_face(face: int) -> bool:                                               <L 1335>
    // return bool(wp.uint32(face) & _FACE_INVALID_OR_DELETED_MASK)                           <L 1337>
    var_0 = wp::uint32(var_face);
    var_2 = wp::bit_and(var_0, var_1);
    var_3 = bool(var_2);
    return var_3;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1316
static CUDA_CALLABLE wp::int32 _delete_face_0(
    wp::int32 var_face)
{
    //---------
    // primal vars
    wp::uint32 var_0;
    const wp::uint32 var_1 = 2147483648u;
    wp::uint32 var_2;
    wp::int32 var_3;
    //---------
    // forward
    // def _delete_face(face: int) -> int:                                                    <L 1317>
    // return int(wp.uint32(face) | _FACE_DELETED_BIT)                                        <L 1319>
    var_0 = wp::uint32(var_face);
    var_2 = wp::bit_or(var_0, var_1);
    var_3 = wp::int(var_2);
    return var_3;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1310
static CUDA_CALLABLE wp::vec_t<3, wp::int32> _get_face_verts_0(
    wp::int32 var_face)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1023;
    wp::int32 var_1;
    const wp::int32 var_2 = 10;
    wp::int32 var_3;
    const wp::int32 var_4 = 1023;
    wp::int32 var_5;
    const wp::int32 var_6 = 20;
    wp::int32 var_7;
    const wp::int32 var_8 = 1023;
    wp::int32 var_9;
    wp::vec_t<3, wp::int32> var_10;
    //---------
    // forward
    // def _get_face_verts(face: int) -> wp.vec3i:                                            <L 1311>
    // return wp.vec3i(face & 0x3FF, (face >> 10) & 0x3FF, (face >> 20) & 0x3FF)              <L 1313>
    var_1 = wp::bit_and(var_face, var_0);
    var_3 = wp::rshift(var_face, var_2);
    var_5 = wp::bit_and(var_3, var_4);
    var_7 = wp::rshift(var_face, var_6);
    var_9 = wp::bit_and(var_7, var_8);
    var_10 = wp::vec_t<3, wp::int32>(var_1, var_5, var_9);
    return var_10;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:946
static CUDA_CALLABLE wp::int32 _add_edge_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_e1,
    wp::int32 var_e2)
{
    //---------
    // primal vars
    wp::int32* var_0;
    wp::int32 var_1;
    wp::int32 var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    const wp::int32 var_5 = -1;
    wp::int32 var_6;
    const wp::int32 var_7 = 10;
    wp::int32 var_8;
    wp::int32 var_9;
    wp::int32 var_10;
    wp::range_t var_11;
    wp::int32 var_12;
    wp::array_t<wp::int32>* var_13;
    wp::int32* var_14;
    wp::array_t<wp::int32> var_15;
    bool var_16;
    wp::int32 var_17;
    wp::array_t<wp::int32>* var_18;
    const wp::int32 var_19 = 1;
    wp::int32 var_20;
    wp::int32* var_21;
    wp::array_t<wp::int32> var_22;
    wp::array_t<wp::int32>* var_23;
    wp::array_t<wp::int32> var_24;
    wp::int32 var_25;
    const wp::int32 var_26 = 1;
    wp::int32 var_27;
    wp::array_t<wp::int32>* var_28;
    wp::shape_t* var_29;
    const wp::int32 var_30 = 0;
    wp::int32 var_31;
    wp::shape_t var_32;
    bool var_33;
    const wp::int32 var_34 = -1;
    wp::array_t<wp::int32>* var_35;
    wp::array_t<wp::int32> var_36;
    const wp::int32 var_37 = 1;
    wp::int32 var_38;
    //---------
    // forward
    // def _add_edge(pt: Polytope, e1: int, e2: int) -> int:                                  <L 947>
    // n = pt.nhorizon                                                                        <L 948>
    var_0 = &((var_pt).nhorizon);
    var_2 = wp::load(var_0);
    var_1 = wp::copy(var_2);
    // if n < 0:                                                                              <L 950>
    var_4 = (var_1 < var_3);
    if (var_4) {
        // return -1                                                                          <L 951>
        return var_5;
    }
    // edge = (wp.min(e1, e2) << 10) | wp.max(e1, e2)                                         <L 953>
    var_6 = wp::min(var_e1, var_e2);
    var_8 = wp::lshift(var_6, var_7);
    var_9 = wp::max(var_e1, var_e2);
    var_10 = wp::bit_or(var_8, var_9);
    // for i in range(n):                                                                     <L 955>
    var_11 = wp::range(var_1);
    start_for_1:;
        if (iter_cmp(var_11) == 0) goto end_for_1;
        var_12 = wp::iter_next(var_11);
        // if edge == pt.horizon[i]:                                                          <L 956>
        var_13 = &((var_pt).horizon);
        var_15 = wp::load(var_13);
        var_14 = wp::address(var_15, var_12);
        var_17 = wp::load(var_14);
        var_16 = (var_10 == var_17);
        if (var_16) {
            // pt.horizon[i] = pt.horizon[n - 1]                                              <L 957>
            var_18 = &((var_pt).horizon);
            var_20 = wp::sub(var_1, var_19);
            var_22 = wp::load(var_18);
            var_21 = wp::address(var_22, var_20);
            var_23 = &((var_pt).horizon);
            var_24 = wp::load(var_23);
            var_25 = wp::load(var_21);
            wp::array_store(var_24, var_12, var_25);
            // return n - 1                                                                   <L 958>
            var_27 = wp::sub(var_1, var_26);
            return var_27;
        }
        goto start_for_1;
    end_for_1:;
    // if n == pt.horizon.shape[0]:                                                           <L 961>
    var_28 = &((var_pt).horizon);
    var_29 = &(var_28->shape);
    var_32 = wp::load(var_29);
    var_31 = wp::extract(var_32, var_30);
    var_33 = (var_1 == var_31);
    if (var_33) {
        // return -1                                                                          <L 962>
        return var_34;
    }
    // pt.horizon[n] = edge                                                                   <L 964>
    var_35 = &((var_pt).horizon);
    var_36 = wp::load(var_35);
    wp::array_store(var_36, var_1, var_10);
    // return n + 1                                                                           <L 965>
    var_38 = wp::add(var_1, var_37);
    return var_38;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1322
static CUDA_CALLABLE bool _is_face_deleted_0(
    wp::int32 var_face)
{
    //---------
    // primal vars
    wp::uint32 var_0;
    const wp::uint32 var_1 = 2147483648u;
    wp::uint32 var_2;
    bool var_3;
    //---------
    // forward
    // def _is_face_deleted(face: int) -> bool:                                               <L 1323>
    // return bool(wp.uint32(face) & _FACE_DELETED_BIT)                                       <L 1325>
    var_0 = wp::uint32(var_face);
    var_2 = wp::bit_and(var_0, var_1);
    var_3 = bool(var_2);
    return var_3;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:941
static CUDA_CALLABLE wp::vec_t<2, wp::int32> _get_edge_0(
    wp::int32 var_edge)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1023;
    wp::int32 var_1;
    const wp::int32 var_2 = 10;
    wp::int32 var_3;
    const wp::int32 var_4 = 1023;
    wp::int32 var_5;
    wp::vec_t<2, wp::int32> var_6;
    //---------
    // forward
    // def _get_edge(edge: int) -> wp.vec2i:                                                  <L 942>
    // return wp.vec2i(edge & 0x3FF, (edge >> 10) & 0x3FF)                                    <L 943>
    var_1 = wp::bit_and(var_edge, var_0);
    var_3 = wp::rshift(var_edge, var_2);
    var_5 = wp::bit_and(var_3, var_4);
    var_6 = wp::vec_t<2, wp::int32>(var_1, var_5);
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1328
static CUDA_CALLABLE wp::int32 _invalidate_face_0(
    wp::int32 var_face)
{
    //---------
    // primal vars
    wp::uint32 var_0;
    const wp::uint32 var_1 = 1073741824u;
    wp::uint32 var_2;
    wp::int32 var_3;
    //---------
    // forward
    // def _invalidate_face(face: int) -> int:                                                <L 1329>
    // return int(wp.uint32(face) | _FACE_INVALID_BIT)                                        <L 1331>
    var_0 = wp::uint32(var_face);
    var_2 = wp::bit_or(var_0, var_1);
    var_3 = wp::int(var_2);
    return var_3;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:968
static CUDA_CALLABLE void _epa_witness_0(
    Polytope_10582b13 var_pt,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::int32 var_face_idx,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::float32 & ret_2)
{
    //---------
    // primal vars
    wp::array_t<wp::int32>* var_0;
    wp::int32* var_1;
    wp::array_t<wp::int32> var_2;
    wp::vec_t<3, wp::int32> var_3;
    wp::int32 var_4;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_5;
    const wp::int32 var_6 = 2;
    const wp::int32 var_7 = 0;
    wp::int32 var_8;
    wp::int32 var_9;
    wp::vec_t<3, wp::float32>* var_10;
    wp::array_t<wp::vec_t<3, wp::float32>> var_11;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_12;
    const wp::int32 var_13 = 2;
    const wp::int32 var_14 = 0;
    wp::int32 var_15;
    wp::int32 var_16;
    const wp::int32 var_17 = 1;
    wp::int32 var_18;
    wp::vec_t<3, wp::float32>* var_19;
    wp::array_t<wp::vec_t<3, wp::float32>> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<3, wp::float32> var_23;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_24;
    const wp::int32 var_25 = 2;
    const wp::int32 var_26 = 1;
    wp::int32 var_27;
    wp::int32 var_28;
    wp::vec_t<3, wp::float32>* var_29;
    wp::array_t<wp::vec_t<3, wp::float32>> var_30;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_31;
    const wp::int32 var_32 = 2;
    const wp::int32 var_33 = 1;
    wp::int32 var_34;
    wp::int32 var_35;
    const wp::int32 var_36 = 1;
    wp::int32 var_37;
    wp::vec_t<3, wp::float32>* var_38;
    wp::array_t<wp::vec_t<3, wp::float32>> var_39;
    wp::vec_t<3, wp::float32> var_40;
    wp::vec_t<3, wp::float32> var_41;
    wp::vec_t<3, wp::float32> var_42;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_43;
    const wp::int32 var_44 = 2;
    const wp::int32 var_45 = 2;
    wp::int32 var_46;
    wp::int32 var_47;
    wp::vec_t<3, wp::float32>* var_48;
    wp::array_t<wp::vec_t<3, wp::float32>> var_49;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_50;
    const wp::int32 var_51 = 2;
    const wp::int32 var_52 = 2;
    wp::int32 var_53;
    wp::int32 var_54;
    const wp::int32 var_55 = 1;
    wp::int32 var_56;
    wp::vec_t<3, wp::float32>* var_57;
    wp::array_t<wp::vec_t<3, wp::float32>> var_58;
    wp::vec_t<3, wp::float32> var_59;
    wp::vec_t<3, wp::float32> var_60;
    wp::vec_t<3, wp::float32> var_61;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_62;
    wp::vec_t<3, wp::float32>* var_63;
    wp::array_t<wp::vec_t<3, wp::float32>> var_64;
    wp::vec_t<3, wp::float32> var_65;
    wp::vec_t<3, wp::float32> var_66;
    const wp::int32 var_67 = 0;
    wp::float32 var_68;
    const wp::int32 var_69 = 1;
    wp::float32 var_70;
    const wp::int32 var_71 = 2;
    wp::float32 var_72;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_73;
    const wp::int32 var_74 = 2;
    const wp::int32 var_75 = 0;
    wp::int32 var_76;
    wp::int32 var_77;
    const wp::int32 var_78 = 1;
    wp::int32 var_79;
    wp::vec_t<3, wp::float32>* var_80;
    wp::array_t<wp::vec_t<3, wp::float32>> var_81;
    wp::vec_t<3, wp::float32> var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_84;
    const wp::int32 var_85 = 2;
    const wp::int32 var_86 = 1;
    wp::int32 var_87;
    wp::int32 var_88;
    const wp::int32 var_89 = 1;
    wp::int32 var_90;
    wp::vec_t<3, wp::float32>* var_91;
    wp::array_t<wp::vec_t<3, wp::float32>> var_92;
    wp::vec_t<3, wp::float32> var_93;
    wp::vec_t<3, wp::float32> var_94;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_95;
    const wp::int32 var_96 = 2;
    const wp::int32 var_97 = 2;
    wp::int32 var_98;
    wp::int32 var_99;
    const wp::int32 var_100 = 1;
    wp::int32 var_101;
    wp::vec_t<3, wp::float32>* var_102;
    wp::array_t<wp::vec_t<3, wp::float32>> var_103;
    wp::vec_t<3, wp::float32> var_104;
    wp::vec_t<3, wp::float32> var_105;
    wp::vec_t<3, wp::float32> var_106;
    const wp::int32 var_107 = 0;
    wp::float32 var_108;
    wp::float32 var_109;
    const wp::int32 var_110 = 0;
    wp::float32 var_111;
    wp::float32 var_112;
    wp::float32 var_113;
    const wp::int32 var_114 = 0;
    wp::float32 var_115;
    wp::float32 var_116;
    wp::float32 var_117;
    const wp::int32 var_118 = 0;
    const wp::int32 var_119 = 1;
    wp::float32 var_120;
    wp::float32 var_121;
    const wp::int32 var_122 = 1;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::float32 var_125;
    const wp::int32 var_126 = 1;
    wp::float32 var_127;
    wp::float32 var_128;
    wp::float32 var_129;
    const wp::int32 var_130 = 1;
    const wp::int32 var_131 = 2;
    wp::float32 var_132;
    wp::float32 var_133;
    const wp::int32 var_134 = 2;
    wp::float32 var_135;
    wp::float32 var_136;
    wp::float32 var_137;
    const wp::int32 var_138 = 2;
    wp::float32 var_139;
    wp::float32 var_140;
    wp::float32 var_141;
    const wp::int32 var_142 = 2;
    wp::array_t<wp::int32>* var_143;
    const wp::int32 var_144 = 2;
    const wp::int32 var_145 = 0;
    wp::int32 var_146;
    wp::int32 var_147;
    wp::int32* var_148;
    wp::array_t<wp::int32> var_149;
    wp::int32 var_150;
    wp::int32 var_151;
    wp::array_t<wp::int32>* var_152;
    const wp::int32 var_153 = 2;
    const wp::int32 var_154 = 1;
    wp::int32 var_155;
    wp::int32 var_156;
    wp::int32* var_157;
    wp::array_t<wp::int32> var_158;
    wp::int32 var_159;
    wp::int32 var_160;
    wp::array_t<wp::int32>* var_161;
    const wp::int32 var_162 = 2;
    const wp::int32 var_163 = 2;
    wp::int32 var_164;
    wp::int32 var_165;
    wp::int32* var_166;
    wp::array_t<wp::int32> var_167;
    wp::int32 var_168;
    wp::int32 var_169;
    bool var_170;
    const wp::int32 var_171 = 1;
    bool var_172;
    bool var_173;
    bool var_174;
    bool var_175;
    const wp::float32 var_176 = 0.0;
    const wp::float32 var_177 = 0.0;
    const wp::float32 var_178 = 1.0;
    wp::vec_t<3, wp::float32> var_179;
    wp::mat_t<6, 3, wp::float32>* var_180;
    const wp::int32 var_181 = 3;
    wp::vec_t<3, wp::float32> var_182;
    wp::mat_t<6, 3, wp::float32> var_183;
    wp::mat_t<6, 3, wp::float32>* var_184;
    const wp::int32 var_185 = 4;
    wp::vec_t<3, wp::float32> var_186;
    wp::mat_t<6, 3, wp::float32> var_187;
    wp::mat_t<6, 3, wp::float32>* var_188;
    const wp::int32 var_189 = 5;
    wp::vec_t<3, wp::float32> var_190;
    wp::mat_t<6, 3, wp::float32> var_191;
    bool var_192;
    const wp::int32 var_193 = 3;
    bool var_194;
    const wp::int32 var_195 = 2;
    bool var_196;
    wp::vec_t<3, wp::float32>* var_197;
    const wp::int32 var_198 = 0;
    wp::float32 var_199;
    wp::vec_t<3, wp::float32> var_200;
    wp::float32* var_201;
    wp::float32 var_202;
    wp::float32 var_203;
    const wp::float32 var_204 = 0.0;
    const wp::float32 var_205 = 0.0;
    wp::vec_t<3, wp::float32>* var_206;
    const wp::int32 var_207 = 1;
    wp::float32 var_208;
    wp::vec_t<3, wp::float32> var_209;
    wp::vec_t<3, wp::float32>* var_210;
    const wp::int32 var_211 = 2;
    wp::float32 var_212;
    wp::vec_t<3, wp::float32> var_213;
    wp::vec_t<3, wp::float32> var_214;
    SupportPoint_e82efc60 var_215;
    wp::vec_t<3, wp::float32>* var_216;
    const wp::float32 var_217 = 0.5;
    wp::float32 var_218;
    wp::float32 var_219;
    wp::vec_t<3, wp::float32> var_220;
    wp::vec_t<3, wp::float32> var_221;
    wp::vec_t<3, wp::float32> var_222;
    wp::vec_t<3, wp::float32>* var_223;
    const wp::int32 var_224 = 0;
    wp::float32* var_225;
    wp::vec_t<3, wp::float32> var_226;
    wp::vec_t<3, wp::float32> var_227;
    SupportPoint_e82efc60 var_228;
    wp::vec_t<3, wp::float32>* var_229;
    wp::vec_t<3, wp::float32> var_230;
    wp::vec_t<3, wp::float32> var_231;
    wp::vec_t<3, wp::float32> var_232;
    SupportPoint_e82efc60 var_233;
    wp::vec_t<3, wp::float32> var_234;
    bool var_235;
    const wp::int32 var_236 = 0;
    wp::float32 var_237;
    const wp::float32 var_238 = 0.0;
    bool var_239;
    const wp::int32 var_240 = 1;
    wp::float32 var_241;
    const wp::float32 var_242 = 0.0;
    bool var_243;
    const wp::int32 var_244 = 2;
    wp::float32 var_245;
    const wp::float32 var_246 = 0.0;
    bool var_247;
    const wp::int32 var_248 = 0;
    wp::float32 var_249;
    wp::vec_t<3, wp::float32> var_250;
    const wp::int32 var_251 = 1;
    wp::float32 var_252;
    wp::vec_t<3, wp::float32> var_253;
    wp::vec_t<3, wp::float32> var_254;
    const wp::int32 var_255 = 2;
    wp::float32 var_256;
    wp::vec_t<3, wp::float32> var_257;
    wp::vec_t<3, wp::float32> var_258;
    wp::vec_t<3, wp::float32> var_259;
    const wp::int32 var_260 = 1;
    wp::float32 var_261;
    const wp::int32 var_262 = 0;
    bool var_263;
    wp::vec_t<3, wp::float32> var_264;
    const wp::int32 var_265 = 0;
    wp::float32 var_266;
    const wp::int32 var_267 = 0;
    bool var_268;
    wp::vec_t<3, wp::float32> var_269;
    wp::vec_t<3, wp::float32> var_270;
    wp::float32 var_271;
    wp::vec_t<3, wp::float32> var_272;
    wp::vec_t<3, wp::float32> var_273;
    wp::vec_t<3, wp::float32> var_274;
    wp::vec_t<3, wp::float32> var_275;
    wp::float32 var_276;
    wp::float32 var_277;
    wp::vec_t<3, wp::float32> var_278;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_279;
    const wp::int32 var_280 = 2;
    const wp::int32 var_281 = 0;
    wp::int32 var_282;
    wp::int32 var_283;
    wp::vec_t<3, wp::float32>* var_284;
    wp::array_t<wp::vec_t<3, wp::float32>> var_285;
    wp::vec_t<3, wp::float32> var_286;
    wp::vec_t<3, wp::float32> var_287;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_288;
    const wp::int32 var_289 = 2;
    const wp::int32 var_290 = 1;
    wp::int32 var_291;
    wp::int32 var_292;
    wp::vec_t<3, wp::float32>* var_293;
    wp::array_t<wp::vec_t<3, wp::float32>> var_294;
    wp::vec_t<3, wp::float32> var_295;
    wp::vec_t<3, wp::float32> var_296;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_297;
    const wp::int32 var_298 = 2;
    const wp::int32 var_299 = 2;
    wp::int32 var_300;
    wp::int32 var_301;
    wp::vec_t<3, wp::float32>* var_302;
    wp::array_t<wp::vec_t<3, wp::float32>> var_303;
    wp::vec_t<3, wp::float32> var_304;
    wp::vec_t<3, wp::float32> var_305;
    wp::vec_t<3, wp::float32> var_306;
    const wp::int32 var_307 = 0;
    wp::float32 var_308;
    wp::float32 var_309;
    const wp::int32 var_310 = 0;
    wp::float32 var_311;
    wp::float32 var_312;
    wp::float32 var_313;
    const wp::int32 var_314 = 0;
    wp::float32 var_315;
    wp::float32 var_316;
    wp::float32 var_317;
    const wp::int32 var_318 = 0;
    const wp::int32 var_319 = 1;
    wp::float32 var_320;
    wp::float32 var_321;
    const wp::int32 var_322 = 1;
    wp::float32 var_323;
    wp::float32 var_324;
    wp::float32 var_325;
    const wp::int32 var_326 = 1;
    wp::float32 var_327;
    wp::float32 var_328;
    wp::float32 var_329;
    const wp::int32 var_330 = 1;
    const wp::int32 var_331 = 2;
    wp::float32 var_332;
    wp::float32 var_333;
    const wp::int32 var_334 = 2;
    wp::float32 var_335;
    wp::float32 var_336;
    wp::float32 var_337;
    const wp::int32 var_338 = 2;
    wp::float32 var_339;
    wp::float32 var_340;
    wp::float32 var_341;
    const wp::int32 var_342 = 2;
    wp::array_t<wp::float32>* var_343;
    wp::float32* var_344;
    wp::array_t<wp::float32> var_345;
    wp::float32 var_346;
    wp::float32 var_347;
    wp::float32 var_348;
    //---------
    // forward
    // def _epa_witness(                                                                      <L 969>
    // face = _get_face_verts(pt.face[face_idx])                                              <L 972>
    var_0 = &((var_pt).face);
    var_2 = wp::load(var_0);
    var_1 = wp::address(var_2, var_face_idx);
    var_4 = wp::load(var_1);
    var_3 = _get_face_verts_0(var_4);
    // v1 = pt.vert[2 * face[0]] - pt.vert[2 * face[0] + 1]                                   <L 974>
    var_5 = &((var_pt).vert);
    var_8 = wp::extract(var_3, var_7);
    var_9 = wp::mul(var_6, var_8);
    var_11 = wp::load(var_5);
    var_10 = wp::address(var_11, var_9);
    var_12 = &((var_pt).vert);
    var_15 = wp::extract(var_3, var_14);
    var_16 = wp::mul(var_13, var_15);
    var_18 = wp::add(var_16, var_17);
    var_20 = wp::load(var_12);
    var_19 = wp::address(var_20, var_18);
    var_22 = wp::load(var_10);
    var_23 = wp::load(var_19);
    var_21 = wp::sub(var_22, var_23);
    // v2 = pt.vert[2 * face[1]] - pt.vert[2 * face[1] + 1]                                   <L 975>
    var_24 = &((var_pt).vert);
    var_27 = wp::extract(var_3, var_26);
    var_28 = wp::mul(var_25, var_27);
    var_30 = wp::load(var_24);
    var_29 = wp::address(var_30, var_28);
    var_31 = &((var_pt).vert);
    var_34 = wp::extract(var_3, var_33);
    var_35 = wp::mul(var_32, var_34);
    var_37 = wp::add(var_35, var_36);
    var_39 = wp::load(var_31);
    var_38 = wp::address(var_39, var_37);
    var_41 = wp::load(var_29);
    var_42 = wp::load(var_38);
    var_40 = wp::sub(var_41, var_42);
    // v3 = pt.vert[2 * face[2]] - pt.vert[2 * face[2] + 1]                                   <L 976>
    var_43 = &((var_pt).vert);
    var_46 = wp::extract(var_3, var_45);
    var_47 = wp::mul(var_44, var_46);
    var_49 = wp::load(var_43);
    var_48 = wp::address(var_49, var_47);
    var_50 = &((var_pt).vert);
    var_53 = wp::extract(var_3, var_52);
    var_54 = wp::mul(var_51, var_53);
    var_56 = wp::add(var_54, var_55);
    var_58 = wp::load(var_50);
    var_57 = wp::address(var_58, var_56);
    var_60 = wp::load(var_48);
    var_61 = wp::load(var_57);
    var_59 = wp::sub(var_60, var_61);
    // coordinates = _tri_affine_coord(v1, v2, v3, pt.face_pr[face_idx])                      <L 978>
    var_62 = &((var_pt).face_pr);
    var_64 = wp::load(var_62);
    var_63 = wp::address(var_64, var_face_idx);
    var_66 = wp::load(var_63);
    var_65 = _tri_affine_coord_0(var_21, var_40, var_59, var_66);
    // l1 = coordinates[0]                                                                    <L 979>
    var_68 = wp::extract(var_65, var_67);
    // l2 = coordinates[1]                                                                    <L 980>
    var_70 = wp::extract(var_65, var_69);
    // l3 = coordinates[2]                                                                    <L 981>
    var_72 = wp::extract(var_65, var_71);
    // v1 = pt.vert[2 * face[0] + 1]                                                          <L 984>
    var_73 = &((var_pt).vert);
    var_76 = wp::extract(var_3, var_75);
    var_77 = wp::mul(var_74, var_76);
    var_79 = wp::add(var_77, var_78);
    var_81 = wp::load(var_73);
    var_80 = wp::address(var_81, var_79);
    var_83 = wp::load(var_80);
    var_82 = wp::copy(var_83);
    // v2 = pt.vert[2 * face[1] + 1]                                                          <L 985>
    var_84 = &((var_pt).vert);
    var_87 = wp::extract(var_3, var_86);
    var_88 = wp::mul(var_85, var_87);
    var_90 = wp::add(var_88, var_89);
    var_92 = wp::load(var_84);
    var_91 = wp::address(var_92, var_90);
    var_94 = wp::load(var_91);
    var_93 = wp::copy(var_94);
    // v3 = pt.vert[2 * face[2] + 1]                                                          <L 986>
    var_95 = &((var_pt).vert);
    var_98 = wp::extract(var_3, var_97);
    var_99 = wp::mul(var_96, var_98);
    var_101 = wp::add(var_99, var_100);
    var_103 = wp::load(var_95);
    var_102 = wp::address(var_103, var_101);
    var_105 = wp::load(var_102);
    var_104 = wp::copy(var_105);
    // x2 = wp.vec3()                                                                         <L 987>
    var_106 = wp::vec_t<3, wp::float32>();
    // x2[0] = v1[0] * l1 + v2[0] * l2 + v3[0] * l3                                           <L 988>
    var_108 = wp::extract(var_82, var_107);
    var_109 = wp::mul(var_108, var_68);
    var_111 = wp::extract(var_93, var_110);
    var_112 = wp::mul(var_111, var_70);
    var_113 = wp::add(var_109, var_112);
    var_115 = wp::extract(var_104, var_114);
    var_116 = wp::mul(var_115, var_72);
    var_117 = wp::add(var_113, var_116);
    wp::assign_inplace(var_106, var_118, var_117);
    // x2[1] = v1[1] * l1 + v2[1] * l2 + v3[1] * l3                                           <L 989>
    var_120 = wp::extract(var_82, var_119);
    var_121 = wp::mul(var_120, var_68);
    var_123 = wp::extract(var_93, var_122);
    var_124 = wp::mul(var_123, var_70);
    var_125 = wp::add(var_121, var_124);
    var_127 = wp::extract(var_104, var_126);
    var_128 = wp::mul(var_127, var_72);
    var_129 = wp::add(var_125, var_128);
    wp::assign_inplace(var_106, var_130, var_129);
    // x2[2] = v1[2] * l1 + v2[2] * l2 + v3[2] * l3                                           <L 990>
    var_132 = wp::extract(var_82, var_131);
    var_133 = wp::mul(var_132, var_68);
    var_135 = wp::extract(var_93, var_134);
    var_136 = wp::mul(var_135, var_70);
    var_137 = wp::add(var_133, var_136);
    var_139 = wp::extract(var_104, var_138);
    var_140 = wp::mul(var_139, var_72);
    var_141 = wp::add(var_137, var_140);
    wp::assign_inplace(var_106, var_142, var_141);
    // i1 = pt.vert_index[2 * face[0]]                                                        <L 993>
    var_143 = &((var_pt).vert_index);
    var_146 = wp::extract(var_3, var_145);
    var_147 = wp::mul(var_144, var_146);
    var_149 = wp::load(var_143);
    var_148 = wp::address(var_149, var_147);
    var_151 = wp::load(var_148);
    var_150 = wp::copy(var_151);
    // i2 = pt.vert_index[2 * face[1]]                                                        <L 994>
    var_152 = &((var_pt).vert_index);
    var_155 = wp::extract(var_3, var_154);
    var_156 = wp::mul(var_153, var_155);
    var_158 = wp::load(var_152);
    var_157 = wp::address(var_158, var_156);
    var_160 = wp::load(var_157);
    var_159 = wp::copy(var_160);
    // i3 = pt.vert_index[2 * face[2]]                                                        <L 995>
    var_161 = &((var_pt).vert_index);
    var_164 = wp::extract(var_3, var_163);
    var_165 = wp::mul(var_162, var_164);
    var_167 = wp::load(var_161);
    var_166 = wp::address(var_167, var_165);
    var_169 = wp::load(var_166);
    var_168 = wp::copy(var_169);
    // if geomtype1 == GeomType.HFIELD and (i1 != i2 or i1 != i3):                            <L 996>
    var_172 = (var_geomtype1 == var_171);
    var_170 = var_172;
    if (var_170) {
        var_174 = (var_150 != var_159);
        var_173 = var_174;
        if (!var_173) {
            var_175 = (var_150 != var_168);
            var_173 = var_173 || var_175;
        }
        var_170 = var_170 && var_173;
    }
    if (var_170) {
        // n = wp.vec3(0.0, 0.0, 1.0)                                                         <L 998>
        var_179 = wp::vec_t<3, wp::float32>(var_176, var_177, var_178);
        // a = geom1.polyvert[3]                                                              <L 1001>
        var_180 = &((var_geom1).polyvert);
        var_183 = wp::load(var_180);
        var_182 = wp::extract(var_183, var_181);
        // b = geom1.polyvert[4]                                                              <L 1002>
        var_184 = &((var_geom1).polyvert);
        var_187 = wp::load(var_184);
        var_186 = wp::extract(var_187, var_185);
        // c = geom1.polyvert[5]                                                              <L 1003>
        var_188 = &((var_geom1).polyvert);
        var_191 = wp::load(var_188);
        var_190 = wp::extract(var_191, var_189);
        // if geomtype2 == GeomType.CAPSULE or geomtype2 == GeomType.SPHERE:                  <L 1006>
        var_194 = (var_geomtype2 == var_193);
        var_192 = var_194;
        if (!var_192) {
            var_196 = (var_geomtype2 == var_195);
            var_192 = var_192 || var_196;
        }
        if (var_192) {
            // radius = geom2.size[0]                                                         <L 1007>
            var_197 = &((var_geom2).size);
            var_200 = wp::load(var_197);
            var_199 = wp::extract(var_200, var_198);
            // margin = geom2.margin                                                          <L 1008>
            var_201 = &((var_geom2).margin);
            var_203 = wp::load(var_201);
            var_202 = wp::copy(var_203);
            // geom2.margin = 0.0                                                             <L 1009>
            var_geom2.margin = var_204;
            // geom2.size = wp.vec3(0.0, geom2.size[1], geom2.size[2])                        <L 1010>
            var_206 = &((var_geom2).size);
            var_209 = wp::load(var_206);
            var_208 = wp::extract(var_209, var_207);
            var_210 = &((var_geom2).size);
            var_213 = wp::load(var_210);
            var_212 = wp::extract(var_213, var_211);
            var_214 = wp::vec_t<3, wp::float32>(var_205, var_208, var_212);
            var_geom2.size = var_214;
            // sp = support(geom2, geomtype2, x2)                                             <L 1011>
            var_215 = support_0(var_geom2, var_geomtype2, var_106);
            // x2 = sp.point - (0.5 * margin + radius) * n                                    <L 1012>
            var_216 = &((var_215).point);
            var_218 = wp::mul(var_217, var_202);
            var_219 = wp::add(var_218, var_199);
            var_220 = wp::mul(var_219, var_179);
            var_222 = wp::load(var_216);
            var_221 = wp::sub(var_222, var_220);
            // geom2.size[0] = radius                                                         <L 1013>
            var_223 = &((var_geom2).size);
            var_225 = wp::indexref(var_223, var_224);
            wp::store(var_225, var_199);
            // geom2.margin = margin                                                          <L 1014>
            var_geom2.margin = var_202;
        }
        var_226 = wp::where(var_192, var_221, var_106);
        if (!var_192) {
            // x2 = wp.normalize(x2)                                                          <L 1016>
            var_227 = wp::normalize(var_226);
            // sp = support(geom2, geomtype2, x2)                                             <L 1017>
            var_228 = support_0(var_geom2, var_geomtype2, var_227);
            // x2 = sp.point                                                                  <L 1018>
            var_229 = &((var_228).point);
            var_231 = wp::load(var_229);
            var_230 = wp::copy(var_231);
        }
        var_232 = wp::where(var_192, var_226, var_230);
        var_233 = wp::where(var_192, var_215, var_228);
        // coordinates2 = _tri_affine_coord(a, b, c, x2)                                      <L 1020>
        var_234 = _tri_affine_coord_0(var_182, var_186, var_190, var_232);
        // if coordinates2[0] > 0.0 and coordinates2[1] > 0.0 and coordinates2[2] > 0.0:       <L 1021>
        var_237 = wp::extract(var_234, var_236);
        var_239 = (var_237 > var_238);
        var_235 = var_239;
        if (var_235) {
            var_241 = wp::extract(var_234, var_240);
            var_243 = (var_241 > var_242);
            var_235 = var_235 && var_243;
        }
        if (var_235) {
            var_245 = wp::extract(var_234, var_244);
            var_247 = (var_245 > var_246);
            var_235 = var_235 && var_247;
        }
        if (var_235) {
            // x1 = coordinates2[0] * a + coordinates2[1] * b + coordinates2[2] * c           <L 1022>
            var_249 = wp::extract(var_234, var_248);
            var_250 = wp::mul(var_249, var_182);
            var_252 = wp::extract(var_234, var_251);
            var_253 = wp::mul(var_252, var_186);
            var_254 = wp::add(var_250, var_253);
            var_256 = wp::extract(var_234, var_255);
            var_257 = wp::mul(var_256, var_190);
            var_258 = wp::add(var_254, var_257);
        }
        if (!var_235) {
            // p = c                                                                          <L 1024>
            var_259 = wp::copy(var_190);
            // p = wp.where(coordinates2[1] > 0, b, p)                                        <L 1025>
            var_261 = wp::extract(var_234, var_260);
            var_263 = (var_261 > var_262);
            var_264 = wp::where(var_263, var_186, var_259);
            // p = wp.where(coordinates2[0] > 0, a, p)                                        <L 1026>
            var_266 = wp::extract(var_234, var_265);
            var_268 = (var_266 > var_267);
            var_269 = wp::where(var_268, var_182, var_264);
            // x1 = x2 - wp.dot(x2 - p, n) * n                                                <L 1027>
            var_270 = wp::sub(var_232, var_269);
            var_271 = wp::dot(var_270, var_179);
            var_272 = wp::mul(var_271, var_179);
            var_273 = wp::sub(var_232, var_272);
        }
        var_274 = wp::where(var_235, var_258, var_273);
        // return x1, x2, -wp.norm_l2(x1 - x2)                                                <L 1028>
        var_275 = wp::sub(var_274, var_232);
        var_276 = norm_l2_0(var_275);
        var_277 = wp::neg(var_276);
        ret_0 = var_274;
        ret_1 = var_232;
        ret_2 = var_277;
        return;
    }
    var_278 = wp::where(var_170, var_232, var_106);
    // v1 = pt.vert[2 * face[0]]                                                              <L 1031>
    var_279 = &((var_pt).vert);
    var_282 = wp::extract(var_3, var_281);
    var_283 = wp::mul(var_280, var_282);
    var_285 = wp::load(var_279);
    var_284 = wp::address(var_285, var_283);
    var_287 = wp::load(var_284);
    var_286 = wp::copy(var_287);
    // v2 = pt.vert[2 * face[1]]                                                              <L 1032>
    var_288 = &((var_pt).vert);
    var_291 = wp::extract(var_3, var_290);
    var_292 = wp::mul(var_289, var_291);
    var_294 = wp::load(var_288);
    var_293 = wp::address(var_294, var_292);
    var_296 = wp::load(var_293);
    var_295 = wp::copy(var_296);
    // v3 = pt.vert[2 * face[2]]                                                              <L 1033>
    var_297 = &((var_pt).vert);
    var_300 = wp::extract(var_3, var_299);
    var_301 = wp::mul(var_298, var_300);
    var_303 = wp::load(var_297);
    var_302 = wp::address(var_303, var_301);
    var_305 = wp::load(var_302);
    var_304 = wp::copy(var_305);
    // x1 = wp.vec3()                                                                         <L 1034>
    var_306 = wp::vec_t<3, wp::float32>();
    // x1[0] = v1[0] * l1 + v2[0] * l2 + v3[0] * l3                                           <L 1035>
    var_308 = wp::extract(var_286, var_307);
    var_309 = wp::mul(var_308, var_68);
    var_311 = wp::extract(var_295, var_310);
    var_312 = wp::mul(var_311, var_70);
    var_313 = wp::add(var_309, var_312);
    var_315 = wp::extract(var_304, var_314);
    var_316 = wp::mul(var_315, var_72);
    var_317 = wp::add(var_313, var_316);
    wp::assign_inplace(var_306, var_318, var_317);
    // x1[1] = v1[1] * l1 + v2[1] * l2 + v3[1] * l3                                           <L 1036>
    var_320 = wp::extract(var_286, var_319);
    var_321 = wp::mul(var_320, var_68);
    var_323 = wp::extract(var_295, var_322);
    var_324 = wp::mul(var_323, var_70);
    var_325 = wp::add(var_321, var_324);
    var_327 = wp::extract(var_304, var_326);
    var_328 = wp::mul(var_327, var_72);
    var_329 = wp::add(var_325, var_328);
    wp::assign_inplace(var_306, var_330, var_329);
    // x1[2] = v1[2] * l1 + v2[2] * l2 + v3[2] * l3                                           <L 1037>
    var_332 = wp::extract(var_286, var_331);
    var_333 = wp::mul(var_332, var_68);
    var_335 = wp::extract(var_295, var_334);
    var_336 = wp::mul(var_335, var_70);
    var_337 = wp::add(var_333, var_336);
    var_339 = wp::extract(var_304, var_338);
    var_340 = wp::mul(var_339, var_72);
    var_341 = wp::add(var_337, var_340);
    wp::assign_inplace(var_306, var_342, var_341);
    // return x1, x2, -wp.sqrt(pt.face_norm2[face_idx])                                       <L 1039>
    var_343 = &((var_pt).face_norm2);
    var_345 = wp::load(var_343);
    var_344 = wp::address(var_345, var_face_idx);
    var_347 = wp::load(var_344);
    var_346 = wp::sqrt(var_347);
    var_348 = wp::neg(var_346);
    ret_0 = var_306;
    ret_1 = var_278;
    ret_2 = var_348;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1340
static CUDA_CALLABLE void _epa_0(
    wp::float32 var_tolerance,
    wp::int32 var_epa_iterations,
    Polytope_10582b13 var_pt,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    bool var_is_discrete,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::int32 & ret_3)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 1e+30;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::int32 var_3 = -1;
    wp::int32 var_4;
    const wp::int32 var_5 = -1;
    wp::int32 var_6;
    const wp::float32 var_7 = 1e-07;
    wp::float32 var_8;
    wp::int32* var_9;
    wp::int32 var_10;
    wp::int32 var_11;
    const wp::int32 var_12 = 1000;
    wp::int32 var_13;
    wp::range_t var_14;
    wp::int32 var_15;
    wp::int32 var_16;
    const wp::int32 var_17 = -1;
    wp::int32 var_18;
    wp::float32 var_19;
    wp::int32* var_20;
    wp::range_t var_21;
    wp::int32 var_22;
    wp::int32 var_23;
    bool var_24;
    wp::array_t<wp::int32>* var_25;
    wp::int32* var_26;
    wp::array_t<wp::int32> var_27;
    bool var_28;
    wp::int32 var_29;
    bool var_30;
    wp::array_t<wp::float32>* var_31;
    wp::float32* var_32;
    wp::array_t<wp::float32> var_33;
    bool var_34;
    wp::float32 var_35;
    wp::int32 var_36;
    wp::array_t<wp::float32>* var_37;
    wp::float32* var_38;
    wp::array_t<wp::float32> var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::int32 var_42;
    wp::float32 var_43;
    bool var_44;
    bool var_45;
    const wp::int32 var_46 = 0;
    bool var_47;
    wp::int32 var_48;
    wp::int32 var_49;
    wp::int32 var_50;
    const wp::float32 var_51 = 0.0;
    bool var_52;
    wp::int32 var_53;
    wp::int32 var_54;
    wp::float32 var_55;
    wp::int32* var_56;
    wp::int32 var_57;
    wp::int32 var_58;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_59;
    wp::vec_t<3, wp::float32>* var_60;
    wp::array_t<wp::vec_t<3, wp::float32>> var_61;
    wp::vec_t<3, wp::float32> var_62;
    wp::vec_t<3, wp::float32> var_63;
    wp::vec_t<3, wp::float32> var_64;
    wp::int32 var_65;
    wp::int32 var_66;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_67;
    const wp::int32 var_68 = 2;
    wp::int32 var_69;
    wp::vec_t<3, wp::float32>* var_70;
    wp::array_t<wp::vec_t<3, wp::float32>> var_71;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_72;
    const wp::int32 var_73 = 2;
    wp::int32 var_74;
    const wp::int32 var_75 = 1;
    wp::int32 var_76;
    wp::vec_t<3, wp::float32>* var_77;
    wp::array_t<wp::vec_t<3, wp::float32>> var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::vec_t<3, wp::float32> var_80;
    wp::vec_t<3, wp::float32> var_81;
    const wp::int32 var_82 = 1;
    wp::int32* var_83;
    wp::int32 var_84;
    wp::int32 var_85;
    wp::float32 var_86;
    wp::float32 var_87;
    bool var_88;
    wp::float32 var_89;
    wp::float32 var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    bool var_94;
    wp::float32 var_95;
    wp::float32 var_96;
    wp::int32 var_97;
    wp::int32 var_98;
    const bool var_99 = false;
    bool var_100;
    wp::int32* var_101;
    const wp::int32 var_102 = 1;
    wp::int32 var_103;
    wp::int32 var_104;
    wp::range_t var_105;
    wp::int32 var_106;
    bool var_107;
    wp::array_t<wp::int32>* var_108;
    const wp::int32 var_109 = 2;
    wp::int32 var_110;
    wp::int32* var_111;
    wp::array_t<wp::int32> var_112;
    wp::array_t<wp::int32>* var_113;
    const wp::int32 var_114 = 2;
    wp::int32 var_115;
    wp::int32* var_116;
    wp::array_t<wp::int32> var_117;
    bool var_118;
    wp::int32 var_119;
    wp::int32 var_120;
    wp::array_t<wp::int32>* var_121;
    const wp::int32 var_122 = 2;
    wp::int32 var_123;
    const wp::int32 var_124 = 1;
    wp::int32 var_125;
    wp::int32* var_126;
    wp::array_t<wp::int32> var_127;
    wp::array_t<wp::int32>* var_128;
    const wp::int32 var_129 = 2;
    wp::int32 var_130;
    const wp::int32 var_131 = 1;
    wp::int32 var_132;
    wp::int32* var_133;
    wp::array_t<wp::int32> var_134;
    bool var_135;
    wp::int32 var_136;
    wp::int32 var_137;
    const bool var_138 = true;
    wp::float32 var_139;
    wp::float32 var_140;
    wp::int32 var_141;
    wp::int32 var_142;
    wp::float32 var_143;
    wp::float32 var_144;
    wp::int32 var_145;
    wp::int32 var_146;
    wp::int32 var_147;
    const wp::int32 var_148 = 1;
    wp::int32 var_149;
    wp::array_t<wp::int32>* var_150;
    wp::int32* var_151;
    wp::array_t<wp::int32> var_152;
    wp::int32 var_153;
    wp::int32 var_154;
    wp::array_t<wp::int32>* var_155;
    wp::array_t<wp::int32> var_156;
    wp::array_t<wp::int32>* var_157;
    wp::int32* var_158;
    wp::array_t<wp::int32> var_159;
    wp::vec_t<3, wp::int32> var_160;
    wp::int32 var_161;
    const wp::int32 var_162 = 0;
    wp::int32 var_163;
    const wp::int32 var_164 = 1;
    wp::int32 var_165;
    wp::int32 var_166;
    const wp::int32 var_167 = 1;
    wp::int32 var_168;
    const wp::int32 var_169 = 2;
    wp::int32 var_170;
    wp::int32 var_171;
    const wp::int32 var_172 = 2;
    wp::int32 var_173;
    const wp::int32 var_174 = 0;
    wp::int32 var_175;
    wp::int32 var_176;
    wp::int32* var_177;
    const wp::int32 var_178 = -1;
    bool var_179;
    wp::int32 var_180;
    const wp::str var_181 = "Warning: EPA horizon = %d isn't large enough.\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.EPA_HORIZON (or = 0 for all)\n";
    wp::array_t<wp::int32>* var_182;
    wp::shape_t* var_183;
    const wp::int32 var_184 = 0;
    wp::int32 var_185;
    wp::shape_t var_186;
    const wp::int32 var_187 = 256;
    const wp::int32 var_188 = 256;
    wp::int32 var_189;
    const wp::int32 var_190 = -1;
    wp::float32 var_191;
    wp::float32 var_192;
    wp::int32 var_193;
    wp::int32 var_194;
    wp::int32 var_195;
    wp::int32* var_196;
    wp::range_t var_197;
    wp::int32 var_198;
    wp::int32 var_199;
    wp::array_t<wp::int32>* var_200;
    wp::int32* var_201;
    wp::array_t<wp::int32> var_202;
    bool var_203;
    wp::int32 var_204;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_205;
    wp::vec_t<3, wp::float32>* var_206;
    wp::array_t<wp::vec_t<3, wp::float32>> var_207;
    wp::float32 var_208;
    wp::vec_t<3, wp::float32> var_209;
    wp::array_t<wp::float32>* var_210;
    wp::float32* var_211;
    wp::array_t<wp::float32> var_212;
    wp::float32 var_213;
    wp::float32 var_214;
    const wp::float32 var_215 = 1e-10;
    bool var_216;
    wp::array_t<wp::int32>* var_217;
    wp::int32* var_218;
    wp::array_t<wp::int32> var_219;
    bool var_220;
    wp::int32 var_221;
    const wp::int32 var_222 = 1;
    wp::int32 var_223;
    wp::int32 var_224;
    wp::array_t<wp::int32>* var_225;
    wp::int32* var_226;
    wp::array_t<wp::int32> var_227;
    wp::int32 var_228;
    wp::int32 var_229;
    wp::array_t<wp::int32>* var_230;
    wp::array_t<wp::int32> var_231;
    wp::array_t<wp::int32>* var_232;
    wp::int32* var_233;
    wp::array_t<wp::int32> var_234;
    wp::vec_t<3, wp::int32> var_235;
    wp::int32 var_236;
    const wp::int32 var_237 = 0;
    wp::int32 var_238;
    const wp::int32 var_239 = 1;
    wp::int32 var_240;
    wp::int32 var_241;
    const wp::int32 var_242 = 1;
    wp::int32 var_243;
    const wp::int32 var_244 = 2;
    wp::int32 var_245;
    wp::int32 var_246;
    const wp::int32 var_247 = 2;
    wp::int32 var_248;
    const wp::int32 var_249 = 0;
    wp::int32 var_250;
    wp::int32 var_251;
    wp::int32* var_252;
    const wp::int32 var_253 = -1;
    bool var_254;
    wp::int32 var_255;
    const wp::str var_256 = "Warning: EPA horizon = %d isn't large enough.\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.EPA_HORIZON (or = 0 for all)\n";
    wp::array_t<wp::int32>* var_257;
    wp::shape_t* var_258;
    const wp::int32 var_259 = 0;
    wp::int32 var_260;
    wp::shape_t var_261;
    const wp::int32 var_262 = 256;
    const wp::int32 var_263 = 256;
    wp::int32 var_264;
    const wp::int32 var_265 = -1;
    wp::int32 var_266;
    wp::vec_t<3, wp::int32> var_267;
    wp::int32 var_268;
    wp::vec_t<3, wp::int32> var_269;
    wp::int32* var_270;
    wp::range_t var_271;
    wp::int32 var_272;
    wp::int32 var_273;
    wp::array_t<wp::int32>* var_274;
    wp::int32* var_275;
    wp::array_t<wp::int32> var_276;
    wp::vec_t<2, wp::int32> var_277;
    wp::int32 var_278;
    wp::int32* var_279;
    const wp::int32 var_280 = 0;
    wp::int32 var_281;
    const wp::int32 var_282 = 1;
    wp::int32 var_283;
    wp::float32 var_284;
    wp::int32 var_285;
    const wp::float32 var_286 = 0.0;
    bool var_287;
    const wp::int32 var_288 = -1;
    const wp::int32 var_289 = 1;
    wp::int32* var_290;
    wp::int32 var_291;
    wp::int32 var_292;
    bool var_293;
    bool var_294;
    bool var_295;
    const wp::int32 var_296 = 1;
    wp::int32 var_297;
    wp::int32 var_298;
    wp::array_t<wp::int32>* var_299;
    wp::int32* var_300;
    const wp::int32 var_301 = 1;
    wp::int32 var_302;
    wp::int32 var_303;
    wp::int32* var_304;
    wp::array_t<wp::int32> var_305;
    wp::int32 var_306;
    wp::int32 var_307;
    wp::array_t<wp::int32>* var_308;
    wp::int32* var_309;
    const wp::int32 var_310 = 1;
    wp::int32 var_311;
    wp::int32 var_312;
    wp::array_t<wp::int32> var_313;
    bool var_314;
    const wp::int32 var_315 = 0;
    bool var_316;
    const wp::int32 var_317 = -1;
    bool var_318;
    wp::float32 var_319;
    wp::float32 var_320;
    wp::int32 var_321;
    wp::int32 var_322;
    wp::int32 var_323;
    const wp::int32 var_324 = 0;
    const wp::int32 var_325 = -1;
    bool var_326;
    wp::vec_t<3, wp::float32> var_327;
    wp::vec_t<3, wp::float32> var_328;
    wp::float32 var_329;
    const wp::float32 var_330 = 0.0;
    wp::vec_t<3, wp::float32> var_331;
    wp::vec_t<3, wp::float32> var_332;
    const wp::int32 var_333 = -1;
    //---------
    // forward
    // def _epa(                                                                              <L 1341>
    // upper = FLOAT_MAX                                                                      <L 1357>
    var_1 = wp::copy(var_0);
    // upper2 = FLOAT_MAX                                                                     <L 1358>
    var_2 = wp::copy(var_0);
    // idx = int(-1)                                                                          <L 1359>
    var_4 = wp::int(var_3);
    // pidx = int(-1)                                                                         <L 1360>
    var_6 = wp::int(var_5);
    // epsilon = wp.where(is_discrete, MIN_EPATOL, tolerance)                                 <L 1361>
    var_8 = wp::where(var_is_discrete, var_7, var_tolerance);
    // nvalid = pt.nface  # number of potential faces for expanding the polytope              <L 1362>
    var_9 = &((var_pt).nface);
    var_11 = wp::load(var_9);
    var_10 = wp::copy(var_11);
    // epa_iterations = wp.min(epa_iterations, 1000)                                          <L 1367>
    var_13 = wp::min(var_epa_iterations, var_12);
    // for _ in range(epa_iterations):                                                        <L 1368>
    var_14 = wp::range(var_13);
    start_for_0:;
        if (iter_cmp(var_14) == 0) goto end_for_0;
        var_15 = wp::iter_next(var_14);
        // pidx = idx                                                                         <L 1369>
        var_16 = wp::copy(var_4);
        // idx = int(-1)                                                                      <L 1370>
        var_18 = wp::int(var_17);
        // lower2 = float(FLOAT_MAX)                                                          <L 1371>
        var_19 = wp::float(var_0);
        // for i in range(pt.nface):                                                          <L 1374>
        var_20 = &((var_pt).nface);
        var_22 = wp::load(var_20);
        var_21 = wp::range(var_22);
        start_for_2:;
            if (iter_cmp(var_21) == 0) goto end_for_2;
            var_23 = wp::iter_next(var_21);
            // if not _is_invalid_face(pt.face[i]) and pt.face_norm2[i] < lower2:             <L 1375>
            var_25 = &((var_pt).face);
            var_27 = wp::load(var_25);
            var_26 = wp::address(var_27, var_23);
            var_29 = wp::load(var_26);
            var_28 = _is_invalid_face_0(var_29);
            var_30 = wp::unot(var_28);
            var_24 = var_30;
            if (var_24) {
                var_31 = &((var_pt).face_norm2);
                var_33 = wp::load(var_31);
                var_32 = wp::address(var_33, var_23);
                var_35 = wp::load(var_32);
                var_34 = (var_35 < var_19);
                var_24 = var_24 && var_34;
            }
            if (var_24) {
                // idx = i                                                                    <L 1376>
                var_36 = wp::copy(var_23);
                // lower2 = pt.face_norm2[i]                                                  <L 1377>
                var_37 = &((var_pt).face_norm2);
                var_39 = wp::load(var_37);
                var_38 = wp::address(var_39, var_23);
                var_41 = wp::load(var_38);
                var_40 = wp::copy(var_41);
            }
            var_42 = wp::where(var_24, var_36, var_18);
            var_43 = wp::where(var_24, var_40, var_19);
            wp::assign(var_18, var_42);
            wp::assign(var_19, var_43);
            goto start_for_2;
        end_for_2:;
        // if lower2 > upper2 or idx < 0:                                                     <L 1380>
        var_45 = (var_19 > var_2);
        var_44 = var_45;
        if (!var_44) {
            var_47 = (var_18 < var_46);
            var_44 = var_44 || var_47;
        }
        if (var_44) {
            // idx = pidx                                                                     <L 1381>
            var_48 = wp::copy(var_16);
            // break                                                                          <L 1382>
            wp::assign(var_4, var_48);
            wp::assign(var_6, var_16);
            goto end_for_0;
        }
        var_49 = wp::where(var_44, var_4, var_18);
        var_50 = wp::where(var_44, var_6, var_16);
        // if lower2 <= 0.0:                                                                  <L 1385>
        var_52 = (var_19 <= var_51);
        if (var_52) {
            // break                                                                          <L 1386>
            wp::assign(var_4, var_49);
            wp::assign(var_6, var_50);
            goto end_for_0;
        }
        var_53 = wp::where(var_52, var_4, var_49);
        var_54 = wp::where(var_52, var_6, var_50);
        // lower = wp.sqrt(lower2)                                                            <L 1389>
        var_55 = wp::sqrt(var_19);
        // wi = pt.nvert                                                                      <L 1390>
        var_56 = &((var_pt).nvert);
        var_58 = wp::load(var_56);
        var_57 = wp::copy(var_58);
        // face_pr = pt.face_pr[idx]                                                          <L 1391>
        var_59 = &((var_pt).face_pr);
        var_61 = wp::load(var_59);
        var_60 = wp::address(var_61, var_53);
        var_63 = wp::load(var_60);
        var_62 = wp::copy(var_63);
        // i1, i2 = _epa_support(pt, wi, geom1, geom2, geomtype1, geomtype2, face_pr / lower)       <L 1392>
        var_64 = wp::div(var_62, var_55);
        _epa_support_0(var_pt, var_57, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_64, var_65, var_66);
        // w = pt.vert[2 * wi] - pt.vert[2 * wi + 1]                                          <L 1393>
        var_67 = &((var_pt).vert);
        var_69 = wp::mul(var_68, var_57);
        var_71 = wp::load(var_67);
        var_70 = wp::address(var_71, var_69);
        var_72 = &((var_pt).vert);
        var_74 = wp::mul(var_73, var_57);
        var_76 = wp::add(var_74, var_75);
        var_78 = wp::load(var_72);
        var_77 = wp::address(var_78, var_76);
        var_80 = wp::load(var_70);
        var_81 = wp::load(var_77);
        var_79 = wp::sub(var_80, var_81);
        // geom1.index = i1                                                                   <L 1394>
        var_geom1.index = var_65;
        // geom2.index = i2                                                                   <L 1395>
        var_geom2.index = var_66;
        // pt.nvert += 1                                                                      <L 1396>
        var_83 = &((var_pt).nvert);
        var_85 = wp::load(var_83);
        var_84 = wp::add(var_85, var_82);
        var_pt.nvert = var_84;
        // upper_k = wp.dot(face_pr, w) / lower                                               <L 1399>
        var_86 = wp::dot(var_62, var_79);
        var_87 = wp::div(var_86, var_55);
        // if upper_k < upper:                                                                <L 1400>
        var_88 = (var_87 < var_1);
        if (var_88) {
            // upper = upper_k                                                                <L 1401>
            var_89 = wp::copy(var_87);
            // upper2 = upper * upper                                                         <L 1402>
            var_90 = wp::mul(var_89, var_89);
        }
        var_91 = wp::where(var_88, var_89, var_1);
        var_92 = wp::where(var_88, var_90, var_2);
        // if upper - lower < epsilon:                                                        <L 1404>
        var_93 = wp::sub(var_91, var_55);
        var_94 = (var_93 < var_8);
        if (var_94) {
            // break                                                                          <L 1405>
            wp::assign(var_1, var_91);
            wp::assign(var_2, var_92);
            wp::assign(var_4, var_53);
            wp::assign(var_6, var_54);
            goto end_for_0;
        }
        var_95 = wp::where(var_94, var_1, var_91);
        var_96 = wp::where(var_94, var_2, var_92);
        var_97 = wp::where(var_94, var_4, var_53);
        var_98 = wp::where(var_94, var_6, var_54);
        // if is_discrete:                                                                    <L 1408>
        if (var_is_discrete) {
            // found_repeated = bool(False)                                                   <L 1409>
            var_100 = bool(var_99);
            // for i in range(pt.nvert - 1):                                                  <L 1410>
            var_101 = &((var_pt).nvert);
            var_104 = wp::load(var_101);
            var_103 = wp::sub(var_104, var_102);
            var_105 = wp::range(var_103);
            start_for_4:;
                if (iter_cmp(var_105) == 0) goto end_for_4;
                var_106 = wp::iter_next(var_105);
                // if pt.vert_index[2 * i] == pt.vert_index[2 * wi] and pt.vert_index[2 * i + 1] == pt.vert_index[2 * wi + 1]:       <L 1411>
                var_108 = &((var_pt).vert_index);
                var_110 = wp::mul(var_109, var_106);
                var_112 = wp::load(var_108);
                var_111 = wp::address(var_112, var_110);
                var_113 = &((var_pt).vert_index);
                var_115 = wp::mul(var_114, var_57);
                var_117 = wp::load(var_113);
                var_116 = wp::address(var_117, var_115);
                var_119 = wp::load(var_111);
                var_120 = wp::load(var_116);
                var_118 = (var_119 == var_120);
                var_107 = var_118;
                if (var_107) {
                    var_121 = &((var_pt).vert_index);
                    var_123 = wp::mul(var_122, var_106);
                    var_125 = wp::add(var_123, var_124);
                    var_127 = wp::load(var_121);
                    var_126 = wp::address(var_127, var_125);
                    var_128 = &((var_pt).vert_index);
                    var_130 = wp::mul(var_129, var_57);
                    var_132 = wp::add(var_130, var_131);
                    var_134 = wp::load(var_128);
                    var_133 = wp::address(var_134, var_132);
                    var_136 = wp::load(var_126);
                    var_137 = wp::load(var_133);
                    var_135 = (var_136 == var_137);
                    var_107 = var_107 && var_135;
                }
                if (var_107) {
                    // found_repeated = True                                                  <L 1412>
                    // break                                                                  <L 1413>
                    wp::assign(var_100, var_138);
                    goto end_for_4;
                }
                goto start_for_4;
            end_for_4:;
            // if found_repeated:                                                             <L 1414>
            if (var_100) {
                // break                                                                      <L 1415>
                wp::assign(var_1, var_95);
                wp::assign(var_2, var_96);
                wp::assign(var_4, var_97);
                wp::assign(var_6, var_98);
                goto end_for_0;
            }
            var_139 = wp::where(var_100, var_1, var_95);
            var_140 = wp::where(var_100, var_2, var_96);
            var_141 = wp::where(var_100, var_4, var_97);
            var_142 = wp::where(var_100, var_6, var_98);
        }
        var_143 = wp::where(var_is_discrete, var_139, var_95);
        var_144 = wp::where(var_is_discrete, var_140, var_96);
        var_145 = wp::where(var_is_discrete, var_141, var_97);
        var_146 = wp::where(var_is_discrete, var_142, var_98);
        var_147 = wp::where(var_is_discrete, var_106, var_23);
        // nvalid -= 1                                                                        <L 1417>
        var_149 = wp::sub(var_10, var_148);
        // pt.face[idx] = _delete_face(pt.face[idx])                                          <L 1418>
        var_150 = &((var_pt).face);
        var_152 = wp::load(var_150);
        var_151 = wp::address(var_152, var_145);
        var_154 = wp::load(var_151);
        var_153 = _delete_face_0(var_154);
        var_155 = &((var_pt).face);
        var_156 = wp::load(var_155);
        wp::array_store(var_156, var_145, var_153);
        // face = _get_face_verts(pt.face[idx])                                               <L 1419>
        var_157 = &((var_pt).face);
        var_159 = wp::load(var_157);
        var_158 = wp::address(var_159, var_145);
        var_161 = wp::load(var_158);
        var_160 = _get_face_verts_0(var_161);
        // pt.nhorizon = _add_edge(pt, face[0], face[1])                                      <L 1420>
        var_163 = wp::extract(var_160, var_162);
        var_165 = wp::extract(var_160, var_164);
        var_166 = _add_edge_0(var_pt, var_163, var_165);
        var_pt.nhorizon = var_166;
        // pt.nhorizon = _add_edge(pt, face[1], face[2])                                      <L 1421>
        var_168 = wp::extract(var_160, var_167);
        var_170 = wp::extract(var_160, var_169);
        var_171 = _add_edge_0(var_pt, var_168, var_170);
        var_pt.nhorizon = var_171;
        // pt.nhorizon = _add_edge(pt, face[2], face[0])                                      <L 1422>
        var_173 = wp::extract(var_160, var_172);
        var_175 = wp::extract(var_160, var_174);
        var_176 = _add_edge_0(var_pt, var_173, var_175);
        var_pt.nhorizon = var_176;
        // if pt.nhorizon == -1:                                                              <L 1423>
        var_177 = &((var_pt).nhorizon);
        var_180 = wp::load(var_177);
        var_179 = (var_180 == var_178);
        if (var_179) {
            // if warn_overflow:                                                              <L 1424>
            if (var_warn_overflow) {
                // wp.printf(                                                                 <L 1425>
                // "Warning: EPA horizon = %d isn't large enough.\n"                          <L 1426>
                // pt.horizon.shape[0],                                                       <L 1428>
                var_182 = &((var_pt).horizon);
                var_183 = &(var_182->shape);
                var_186 = wp::load(var_183);
                var_185 = wp::extract(var_186, var_184);
                printf(var_181, var_185);
            }
            // wp.atomic_or(overflow_out, worldid, OverflowType.EPA_HORIZON)                  <L 1430>
            var_189 = wp::atomic_or(var_overflow_out, var_worldid, var_188);
            // idx = -1                                                                       <L 1431>
            // break                                                                          <L 1432>
            wp::assign(var_1, var_143);
            wp::assign(var_2, var_144);
            wp::assign(var_4, var_190);
            wp::assign(var_6, var_146);
            wp::assign(var_10, var_149);
            goto end_for_0;
        }
        var_191 = wp::where(var_179, var_1, var_143);
        var_192 = wp::where(var_179, var_2, var_144);
        var_193 = wp::where(var_179, var_4, var_145);
        var_194 = wp::where(var_179, var_6, var_146);
        var_195 = wp::where(var_179, var_10, var_149);
        // for i in range(pt.nface):                                                          <L 1435>
        var_196 = &((var_pt).nface);
        var_198 = wp::load(var_196);
        var_197 = wp::range(var_198);
        start_for_6:;
            if (iter_cmp(var_197) == 0) goto end_for_6;
            var_199 = wp::iter_next(var_197);
            // if _is_face_deleted(pt.face[i]):                                               <L 1436>
            var_200 = &((var_pt).face);
            var_202 = wp::load(var_200);
            var_201 = wp::address(var_202, var_199);
            var_204 = wp::load(var_201);
            var_203 = _is_face_deleted_0(var_204);
            if (var_203) {
                // continue                                                                   <L 1437>
                goto start_for_6;
            }
            // if wp.dot(pt.face_pr[i], w) - pt.face_norm2[i] > 1e-10:                        <L 1439>
            var_205 = &((var_pt).face_pr);
            var_207 = wp::load(var_205);
            var_206 = wp::address(var_207, var_199);
            var_209 = wp::load(var_206);
            var_208 = wp::dot(var_209, var_79);
            var_210 = &((var_pt).face_norm2);
            var_212 = wp::load(var_210);
            var_211 = wp::address(var_212, var_199);
            var_214 = wp::load(var_211);
            var_213 = wp::sub(var_208, var_214);
            var_216 = (var_213 > var_215);
            if (var_216) {
                // nvalid = wp.where(_is_invalid_face(pt.face[i]), nvalid, nvalid - 1)        <L 1440>
                var_217 = &((var_pt).face);
                var_219 = wp::load(var_217);
                var_218 = wp::address(var_219, var_199);
                var_221 = wp::load(var_218);
                var_220 = _is_invalid_face_0(var_221);
                var_223 = wp::sub(var_195, var_222);
                var_224 = wp::where(var_220, var_195, var_223);
                // pt.face[i] = _delete_face(pt.face[i])                                      <L 1441>
                var_225 = &((var_pt).face);
                var_227 = wp::load(var_225);
                var_226 = wp::address(var_227, var_199);
                var_229 = wp::load(var_226);
                var_228 = _delete_face_0(var_229);
                var_230 = &((var_pt).face);
                var_231 = wp::load(var_230);
                wp::array_store(var_231, var_199, var_228);
                // face = _get_face_verts(pt.face[i])                                         <L 1442>
                var_232 = &((var_pt).face);
                var_234 = wp::load(var_232);
                var_233 = wp::address(var_234, var_199);
                var_236 = wp::load(var_233);
                var_235 = _get_face_verts_0(var_236);
                // pt.nhorizon = _add_edge(pt, face[0], face[1])                              <L 1443>
                var_238 = wp::extract(var_235, var_237);
                var_240 = wp::extract(var_235, var_239);
                var_241 = _add_edge_0(var_pt, var_238, var_240);
                var_pt.nhorizon = var_241;
                // pt.nhorizon = _add_edge(pt, face[1], face[2])                              <L 1444>
                var_243 = wp::extract(var_235, var_242);
                var_245 = wp::extract(var_235, var_244);
                var_246 = _add_edge_0(var_pt, var_243, var_245);
                var_pt.nhorizon = var_246;
                // pt.nhorizon = _add_edge(pt, face[2], face[0])                              <L 1445>
                var_248 = wp::extract(var_235, var_247);
                var_250 = wp::extract(var_235, var_249);
                var_251 = _add_edge_0(var_pt, var_248, var_250);
                var_pt.nhorizon = var_251;
                // if pt.nhorizon == -1:                                                      <L 1446>
                var_252 = &((var_pt).nhorizon);
                var_255 = wp::load(var_252);
                var_254 = (var_255 == var_253);
                if (var_254) {
                    // if warn_overflow:                                                      <L 1447>
                    if (var_warn_overflow) {
                        // wp.printf(                                                         <L 1448>
                        // "Warning: EPA horizon = %d isn't large enough.\n"                  <L 1449>
                        // pt.horizon.shape[0],                                               <L 1451>
                        var_257 = &((var_pt).horizon);
                        var_258 = &(var_257->shape);
                        var_261 = wp::load(var_258);
                        var_260 = wp::extract(var_261, var_259);
                        printf(var_256, var_260);
                    }
                    // wp.atomic_or(overflow_out, worldid, OverflowType.EPA_HORIZON)          <L 1453>
                    var_264 = wp::atomic_or(var_overflow_out, var_worldid, var_263);
                    // idx = -1                                                               <L 1454>
                    // break                                                                  <L 1455>
                    wp::assign(var_193, var_265);
                    wp::assign(var_195, var_224);
                    wp::assign(var_160, var_235);
                    goto end_for_6;
                }
                var_266 = wp::where(var_254, var_195, var_224);
                var_267 = wp::where(var_254, var_160, var_235);
            }
            var_268 = wp::where(var_216, var_266, var_195);
            var_269 = wp::where(var_216, var_267, var_160);
            wp::assign(var_195, var_268);
            wp::assign(var_160, var_269);
            goto start_for_6;
        end_for_6:;
        // for i in range(pt.nhorizon):                                                       <L 1458>
        var_270 = &((var_pt).nhorizon);
        var_272 = wp::load(var_270);
        var_271 = wp::range(var_272);
        start_for_8:;
            if (iter_cmp(var_271) == 0) goto end_for_8;
            var_273 = wp::iter_next(var_271);
            // edge = _get_edge(pt.horizon[i])                                                <L 1459>
            var_274 = &((var_pt).horizon);
            var_276 = wp::load(var_274);
            var_275 = wp::address(var_276, var_273);
            var_278 = wp::load(var_275);
            var_277 = _get_edge_0(var_278);
            // dist2 = _attach_face(pt, pt.nface, wi, edge[0], edge[1])                       <L 1460>
            var_279 = &((var_pt).nface);
            var_281 = wp::extract(var_277, var_280);
            var_283 = wp::extract(var_277, var_282);
            var_285 = wp::load(var_279);
            var_284 = _attach_face_0(var_pt, var_285, var_57, var_281, var_283);
            // if dist2 == 0.0:                                                               <L 1461>
            var_287 = (var_284 == var_286);
            if (var_287) {
                // idx = -1                                                                   <L 1462>
                // break                                                                      <L 1463>
                wp::assign(var_193, var_288);
                goto end_for_8;
            }
            // pt.nface += 1                                                                  <L 1465>
            var_290 = &((var_pt).nface);
            var_292 = wp::load(var_290);
            var_291 = wp::add(var_292, var_289);
            var_pt.nface = var_291;
            // if dist2 >= lower2 and dist2 <= upper2:                                        <L 1467>
            var_294 = (var_284 >= var_19);
            var_293 = var_294;
            if (var_293) {
                var_295 = (var_284 <= var_192);
                var_293 = var_293 && var_295;
            }
            if (var_293) {
                // nvalid += 1                                                                <L 1468>
                var_297 = wp::add(var_195, var_296);
            }
            var_298 = wp::where(var_293, var_297, var_195);
            if (!var_293) {
                // pt.face[pt.nface - 1] = _invalidate_face(pt.face[pt.nface - 1])            <L 1470>
                var_299 = &((var_pt).face);
                var_300 = &((var_pt).nface);
                var_303 = wp::load(var_300);
                var_302 = wp::sub(var_303, var_301);
                var_305 = wp::load(var_299);
                var_304 = wp::address(var_305, var_302);
                var_307 = wp::load(var_304);
                var_306 = _invalidate_face_0(var_307);
                var_308 = &((var_pt).face);
                var_309 = &((var_pt).nface);
                var_312 = wp::load(var_309);
                var_311 = wp::sub(var_312, var_310);
                var_313 = wp::load(var_308);
                wp::array_store(var_313, var_311, var_306);
            }
            wp::assign(var_195, var_298);
            goto start_for_8;
        end_for_8:;
        // if nvalid == 0 or idx == -1:                                                       <L 1473>
        var_316 = (var_195 == var_315);
        var_314 = var_316;
        if (!var_314) {
            var_318 = (var_193 == var_317);
            var_314 = var_314 || var_318;
        }
        if (var_314) {
            // break                                                                          <L 1474>
            wp::assign(var_1, var_191);
            wp::assign(var_2, var_192);
            wp::assign(var_4, var_193);
            wp::assign(var_6, var_194);
            wp::assign(var_10, var_195);
            goto end_for_0;
        }
        var_319 = wp::where(var_314, var_1, var_191);
        var_320 = wp::where(var_314, var_2, var_192);
        var_321 = wp::where(var_314, var_4, var_193);
        var_322 = wp::where(var_314, var_6, var_194);
        var_323 = wp::where(var_314, var_10, var_195);
        // pt.nhorizon = 0                                                                    <L 1477>
        var_pt.nhorizon = var_324;
        wp::assign(var_1, var_319);
        wp::assign(var_2, var_320);
        wp::assign(var_4, var_321);
        wp::assign(var_6, var_322);
        wp::assign(var_10, var_323);
        goto start_for_0;
    end_for_0:;
    // if idx > -1:                                                                           <L 1480>
    var_326 = (var_4 > var_325);
    if (var_326) {
        // x1, x2, dist = _epa_witness(pt, geom1, geom2, geomtype1, geomtype2, idx)           <L 1481>
        _epa_witness_0(var_pt, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_4, var_327, var_328, var_329);
        // return dist, x1, x2, idx                                                           <L 1482>
        ret_0 = var_329;
        ret_1 = var_327;
        ret_2 = var_328;
        ret_3 = var_4;
        return;
    }
    // return 0.0, wp.vec3(), wp.vec3(), -1                                                   <L 1483>
    var_331 = wp::vec_t<3, wp::float32>();
    var_332 = wp::vec_t<3, wp::float32>();
    ret_0 = var_330;
    ret_1 = var_331;
    ret_2 = var_332;
    ret_3 = var_333;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2572
static CUDA_CALLABLE void epa_phase_0(
    wp::float32 var_tolerance,
    wp::int32 var_epa_iterations,
    GJKResult_8184e255 var_result,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::array_t<wp::vec_t<3, wp::float32>> var_vert,
    wp::array_t<wp::int32> var_vert_index,
    wp::array_t<wp::int32> var_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_pr,
    wp::array_t<wp::float32> var_face_norm2,
    wp::array_t<wp::int32> var_horizon,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::int32 & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::int32 & ret_4)
{
    //---------
    // primal vars
    Polytope_10582b13 var_0;
    const wp::int32 var_1 = 0;
    const wp::int32 var_2 = 0;
    const wp::int32 var_3 = 0;
    wp::int32* var_4;
    const wp::int32 var_5 = 2;
    bool var_6;
    wp::int32 var_7;
    wp::mat_t<4, 3, wp::float32>* var_8;
    wp::mat_t<4, 3, wp::float32>* var_9;
    wp::mat_t<4, 3, wp::float32>* var_10;
    wp::vec_t<4, wp::int32>* var_11;
    wp::vec_t<4, wp::int32>* var_12;
    Polytope_10582b13 var_13;
    GJKResult_8184e255 var_14;
    wp::mat_t<4, 3, wp::float32> var_15;
    wp::mat_t<4, 3, wp::float32> var_16;
    wp::mat_t<4, 3, wp::float32> var_17;
    wp::vec_t<4, wp::int32> var_18;
    wp::vec_t<4, wp::int32> var_19;
    wp::int32* var_20;
    const wp::int32 var_21 = -1;
    bool var_22;
    wp::int32 var_23;
    wp::mat_t<4, 3, wp::float32>* var_24;
    wp::mat_t<4, 3, wp::float32> var_25;
    wp::mat_t<4, 3, wp::float32> var_26;
    wp::mat_t<4, 3, wp::float32>* var_27;
    wp::mat_t<4, 3, wp::float32> var_28;
    wp::mat_t<4, 3, wp::float32> var_29;
    wp::mat_t<4, 3, wp::float32>* var_30;
    wp::mat_t<4, 3, wp::float32> var_31;
    wp::mat_t<4, 3, wp::float32> var_32;
    wp::vec_t<4, wp::int32>* var_33;
    wp::vec_t<4, wp::int32> var_34;
    wp::vec_t<4, wp::int32> var_35;
    wp::vec_t<4, wp::int32>* var_36;
    wp::vec_t<4, wp::int32> var_37;
    wp::vec_t<4, wp::int32> var_38;
    const wp::int32 var_39 = 3;
    Polytope_10582b13 var_40;
    wp::int32* var_41;
    const wp::int32 var_42 = 4;
    bool var_43;
    wp::int32 var_44;
    wp::mat_t<4, 3, wp::float32>* var_45;
    wp::mat_t<4, 3, wp::float32>* var_46;
    wp::mat_t<4, 3, wp::float32>* var_47;
    wp::vec_t<4, wp::int32>* var_48;
    wp::vec_t<4, wp::int32>* var_49;
    Polytope_10582b13 var_50;
    GJKResult_8184e255 var_51;
    wp::mat_t<4, 3, wp::float32> var_52;
    wp::mat_t<4, 3, wp::float32> var_53;
    wp::mat_t<4, 3, wp::float32> var_54;
    wp::vec_t<4, wp::int32> var_55;
    wp::vec_t<4, wp::int32> var_56;
    wp::int32* var_57;
    const wp::int32 var_58 = -1;
    bool var_59;
    wp::int32 var_60;
    wp::mat_t<4, 3, wp::float32>* var_61;
    wp::mat_t<4, 3, wp::float32> var_62;
    wp::mat_t<4, 3, wp::float32> var_63;
    wp::mat_t<4, 3, wp::float32>* var_64;
    wp::mat_t<4, 3, wp::float32> var_65;
    wp::mat_t<4, 3, wp::float32> var_66;
    wp::mat_t<4, 3, wp::float32>* var_67;
    wp::mat_t<4, 3, wp::float32> var_68;
    wp::mat_t<4, 3, wp::float32> var_69;
    wp::vec_t<4, wp::int32>* var_70;
    wp::vec_t<4, wp::int32> var_71;
    wp::vec_t<4, wp::int32> var_72;
    wp::vec_t<4, wp::int32>* var_73;
    wp::vec_t<4, wp::int32> var_74;
    wp::vec_t<4, wp::int32> var_75;
    const wp::int32 var_76 = 3;
    Polytope_10582b13 var_77;
    GJKResult_8184e255 var_78;
    Polytope_10582b13 var_79;
    GJKResult_8184e255 var_80;
    wp::int32* var_81;
    const wp::int32 var_82 = 3;
    bool var_83;
    wp::int32 var_84;
    wp::float32* var_85;
    wp::mat_t<4, 3, wp::float32>* var_86;
    wp::mat_t<4, 3, wp::float32>* var_87;
    wp::mat_t<4, 3, wp::float32>* var_88;
    wp::vec_t<4, wp::int32>* var_89;
    wp::vec_t<4, wp::int32>* var_90;
    Polytope_10582b13 var_91;
    wp::float32 var_92;
    wp::mat_t<4, 3, wp::float32> var_93;
    wp::mat_t<4, 3, wp::float32> var_94;
    wp::mat_t<4, 3, wp::float32> var_95;
    wp::vec_t<4, wp::int32> var_96;
    wp::vec_t<4, wp::int32> var_97;
    Polytope_10582b13 var_98;
    wp::int32* var_99;
    wp::int32 var_100;
    wp::float32* var_101;
    const wp::int32 var_102 = 1;
    wp::vec_t<3, wp::float32>* var_103;
    wp::vec_t<3, wp::float32>* var_104;
    const wp::int32 var_105 = -1;
    wp::float32 var_106;
    wp::float32 var_107;
    wp::vec_t<3, wp::float32> var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::vec_t<3, wp::float32> var_110;
    wp::vec_t<3, wp::float32> var_111;
    wp::int32 var_112;
    bool var_113;
    bool var_114;
    bool var_115;
    wp::float32* var_116;
    const wp::float32 var_117 = 0.0;
    bool var_118;
    wp::float32 var_119;
    wp::float32* var_120;
    const wp::float32 var_121 = 0.0;
    bool var_122;
    wp::float32 var_123;
    wp::float32 var_124;
    wp::vec_t<3, wp::float32> var_125;
    wp::vec_t<3, wp::float32> var_126;
    wp::int32 var_127;
    const wp::int32 var_128 = -1;
    bool var_129;
    const wp::float32 var_130 = 1e+30;
    const wp::int32 var_131 = 0;
    wp::vec_t<3, wp::float32> var_132;
    wp::vec_t<3, wp::float32> var_133;
    const wp::int32 var_134 = -1;
    bool var_135;
    wp::float32* var_136;
    const wp::float32 var_137 = 0.0;
    bool var_138;
    wp::float32 var_139;
    wp::float32* var_140;
    const wp::float32 var_141 = 0.0;
    bool var_142;
    wp::float32 var_143;
    const wp::int32 var_144 = -1;
    wp::int32 var_145;
    bool var_146;
    bool var_147;
    const wp::int32 var_148 = 6;
    bool var_149;
    const wp::int32 var_150 = 7;
    bool var_151;
    const wp::int32 var_152 = 5;
    bool var_153;
    bool var_154;
    const wp::int32 var_155 = 6;
    bool var_156;
    const wp::int32 var_157 = 7;
    bool var_158;
    const wp::int32 var_159 = 5;
    bool var_160;
    const wp::int32 var_161 = -1;
    wp::int32 var_162;
    const wp::int32 var_163 = 1;
    //---------
    // forward
    // def epa_phase(                                                                         <L 2573>
    // pt = Polytope()                                                                        <L 2594>
    var_0 = Polytope_10582b13();
    // pt.nface = 0                                                                           <L 2595>
    var_0.nface = var_1;
    // pt.nvert = 0                                                                           <L 2596>
    var_0.nvert = var_2;
    // pt.nhorizon = 0                                                                        <L 2597>
    var_0.nhorizon = var_3;
    // pt.vert = vert                                                                         <L 2598>
    var_0.vert = var_vert;
    // pt.vert_index = vert_index                                                             <L 2599>
    var_0.vert_index = var_vert_index;
    // pt.face = face                                                                         <L 2600>
    var_0.face = var_face;
    // pt.face_pr = face_pr                                                                   <L 2601>
    var_0.face_pr = var_face_pr;
    // pt.face_norm2 = face_norm2                                                             <L 2602>
    var_0.face_norm2 = var_face_norm2;
    // pt.horizon = horizon                                                                   <L 2603>
    var_0.horizon = var_horizon;
    // if result.dim == 2:                                                                    <L 2605>
    var_4 = &((var_result).dim);
    var_7 = wp::load(var_4);
    var_6 = (var_7 == var_5);
    if (var_6) {
        // pt, new_result = _polytope2(                                                       <L 2606>
        // pt,                                                                                <L 2607>
        // result.simplex,                                                                    <L 2608>
        var_8 = &((var_result).simplex);
        // result.simplex1,                                                                   <L 2609>
        var_9 = &((var_result).simplex1);
        // result.simplex2,                                                                   <L 2610>
        var_10 = &((var_result).simplex2);
        // result.simplex_index1,                                                             <L 2611>
        var_11 = &((var_result).simplex_index1);
        // result.simplex_index2,                                                             <L 2612>
        var_12 = &((var_result).simplex_index2);
        // geom1,                                                                             <L 2613>
        // geom2,                                                                             <L 2614>
        // geomtype1,                                                                         <L 2615>
        // geomtype2,                                                                         <L 2616>
        var_15 = wp::load(var_8);
        var_16 = wp::load(var_9);
        var_17 = wp::load(var_10);
        var_18 = wp::load(var_11);
        var_19 = wp::load(var_12);
        _polytope2_0(var_0, var_15, var_16, var_17, var_18, var_19, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_13, var_14);
        // if pt.status == -1:                                                                <L 2618>
        var_20 = &((var_13).status);
        var_23 = wp::load(var_20);
        var_22 = (var_23 == var_21);
        if (var_22) {
            // result.simplex = new_result.simplex                                            <L 2619>
            var_24 = &((var_14).simplex);
            var_26 = wp::load(var_24);
            var_25 = wp::copy(var_26);
            var_result.simplex = var_25;
            // result.simplex1 = new_result.simplex1                                          <L 2620>
            var_27 = &((var_14).simplex1);
            var_29 = wp::load(var_27);
            var_28 = wp::copy(var_29);
            var_result.simplex1 = var_28;
            // result.simplex2 = new_result.simplex2                                          <L 2621>
            var_30 = &((var_14).simplex2);
            var_32 = wp::load(var_30);
            var_31 = wp::copy(var_32);
            var_result.simplex2 = var_31;
            // result.simplex_index1 = new_result.simplex_index1                              <L 2622>
            var_33 = &((var_14).simplex_index1);
            var_35 = wp::load(var_33);
            var_34 = wp::copy(var_35);
            var_result.simplex_index1 = var_34;
            // result.simplex_index2 = new_result.simplex_index2                              <L 2623>
            var_36 = &((var_14).simplex_index2);
            var_38 = wp::load(var_36);
            var_37 = wp::copy(var_38);
            var_result.simplex_index2 = var_37;
            // result.dim = 3                                                                 <L 2624>
            var_result.dim = var_39;
        }
    }
    var_40 = wp::where(var_6, var_13, var_0);
    if (!var_6) {
        // elif result.dim == 4:                                                              <L 2625>
        var_41 = &((var_result).dim);
        var_44 = wp::load(var_41);
        var_43 = (var_44 == var_42);
        if (var_43) {
            // pt, new_result = _polytope4(                                                   <L 2626>
            // pt,                                                                            <L 2627>
            // result.simplex,                                                                <L 2628>
            var_45 = &((var_result).simplex);
            // result.simplex1,                                                               <L 2629>
            var_46 = &((var_result).simplex1);
            // result.simplex2,                                                               <L 2630>
            var_47 = &((var_result).simplex2);
            // result.simplex_index1,                                                         <L 2631>
            var_48 = &((var_result).simplex_index1);
            // result.simplex_index2,                                                         <L 2632>
            var_49 = &((var_result).simplex_index2);
            var_52 = wp::load(var_45);
            var_53 = wp::load(var_46);
            var_54 = wp::load(var_47);
            var_55 = wp::load(var_48);
            var_56 = wp::load(var_49);
            _polytope4_0(var_40, var_52, var_53, var_54, var_55, var_56, var_50, var_51);
            // if pt.status == -1:                                                            <L 2634>
            var_57 = &((var_50).status);
            var_60 = wp::load(var_57);
            var_59 = (var_60 == var_58);
            if (var_59) {
                // result.simplex = new_result.simplex                                        <L 2635>
                var_61 = &((var_51).simplex);
                var_63 = wp::load(var_61);
                var_62 = wp::copy(var_63);
                var_result.simplex = var_62;
                // result.simplex1 = new_result.simplex1                                      <L 2636>
                var_64 = &((var_51).simplex1);
                var_66 = wp::load(var_64);
                var_65 = wp::copy(var_66);
                var_result.simplex1 = var_65;
                // result.simplex2 = new_result.simplex2                                      <L 2637>
                var_67 = &((var_51).simplex2);
                var_69 = wp::load(var_67);
                var_68 = wp::copy(var_69);
                var_result.simplex2 = var_68;
                // result.simplex_index1 = new_result.simplex_index1                          <L 2638>
                var_70 = &((var_51).simplex_index1);
                var_72 = wp::load(var_70);
                var_71 = wp::copy(var_72);
                var_result.simplex_index1 = var_71;
                // result.simplex_index2 = new_result.simplex_index2                          <L 2639>
                var_73 = &((var_51).simplex_index2);
                var_75 = wp::load(var_73);
                var_74 = wp::copy(var_75);
                var_result.simplex_index2 = var_74;
                // result.dim = 3                                                             <L 2640>
                var_result.dim = var_76;
            }
        }
        var_77 = wp::where(var_43, var_50, var_40);
        var_78 = wp::where(var_43, var_51, var_14);
    }
    var_79 = wp::where(var_6, var_40, var_77);
    var_80 = wp::where(var_6, var_14, var_78);
    // if result.dim == 3:                                                                    <L 2643>
    var_81 = &((var_result).dim);
    var_84 = wp::load(var_81);
    var_83 = (var_84 == var_82);
    if (var_83) {
        // pt = _polytope3(                                                                   <L 2644>
        // pt,                                                                                <L 2645>
        // result.dist,                                                                       <L 2646>
        var_85 = &((var_result).dist);
        // result.simplex,                                                                    <L 2647>
        var_86 = &((var_result).simplex);
        // result.simplex1,                                                                   <L 2648>
        var_87 = &((var_result).simplex1);
        // result.simplex2,                                                                   <L 2649>
        var_88 = &((var_result).simplex2);
        // result.simplex_index1,                                                             <L 2650>
        var_89 = &((var_result).simplex_index1);
        // result.simplex_index2,                                                             <L 2651>
        var_90 = &((var_result).simplex_index2);
        // geom1,                                                                             <L 2652>
        // geom2,                                                                             <L 2653>
        // geomtype1,                                                                         <L 2654>
        // geomtype2,                                                                         <L 2655>
        var_92 = wp::load(var_85);
        var_93 = wp::load(var_86);
        var_94 = wp::load(var_87);
        var_95 = wp::load(var_88);
        var_96 = wp::load(var_89);
        var_97 = wp::load(var_90);
        var_91 = _polytope3_0(var_79, var_92, var_93, var_94, var_95, var_96, var_97, var_geom1, var_geom2, var_geomtype1, var_geomtype2);
    }
    var_98 = wp::where(var_83, var_91, var_79);
    // if pt.status:                                                                          <L 2659>
    var_99 = &((var_98).status);
    var_100 = wp::load(var_99);
    if (var_100) {
        // return result.dist, 1, result.x1, result.x2, -1                                    <L 2660>
        var_101 = &((var_result).dist);
        var_103 = &((var_result).x1);
        var_104 = &((var_result).x2);
        var_107 = wp::load(var_101);
        var_106 = wp::copy(var_107);
        var_109 = wp::load(var_103);
        var_108 = wp::copy(var_109);
        var_111 = wp::load(var_104);
        var_110 = wp::copy(var_111);
        ret_0 = var_106;
        ret_1 = var_102;
        ret_2 = var_108;
        ret_3 = var_110;
        ret_4 = var_105;
        return;
    }
    var_112 = wp::load(var_99);
    // is_discrete = _discrete_geoms(geomtype1, geomtype2) and (geom1.margin == 0.0 and geom2.margin == 0.0)       <L 2662>
    var_114 = _discrete_geoms_0(var_geomtype1, var_geomtype2);
    var_113 = var_114;
    if (var_113) {
        var_116 = &((var_geom1).margin);
        var_119 = wp::load(var_116);
        var_118 = (var_119 == var_117);
        var_115 = var_118;
        if (var_115) {
            var_120 = &((var_geom2).margin);
            var_123 = wp::load(var_120);
            var_122 = (var_123 == var_121);
            var_115 = var_115 && var_122;
        }
        var_113 = var_113 && var_115;
    }
    // dist, x1, x2, idx = _epa(                                                              <L 2663>
    // tolerance, epa_iterations, pt, geom1, geom2, geomtype1, geomtype2, is_discrete, warn_overflow, worldid, overflow_out       <L 2664>
    _epa_0(var_tolerance, var_epa_iterations, var_98, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_113, var_warn_overflow, var_worldid, var_overflow_out, var_124, var_125, var_126, var_127);
    // if idx == -1:                                                                          <L 2666>
    var_129 = (var_127 == var_128);
    if (var_129) {
        // return FLOAT_MAX, 0, wp.vec3(), wp.vec3(), -1                                      <L 2667>
        var_132 = wp::vec_t<3, wp::float32>();
        var_133 = wp::vec_t<3, wp::float32>();
        ret_0 = var_130;
        ret_1 = var_131;
        ret_2 = var_132;
        ret_3 = var_133;
        ret_4 = var_134;
        return;
    }
    // if geom1.margin != 0.0 or geom2.margin != 0.0:                                         <L 2670>
    var_136 = &((var_geom1).margin);
    var_139 = wp::load(var_136);
    var_138 = (var_139 != var_137);
    var_135 = var_138;
    if (!var_135) {
        var_140 = &((var_geom2).margin);
        var_143 = wp::load(var_140);
        var_142 = (var_143 != var_141);
        var_135 = var_135 || var_142;
    }
    if (var_135) {
        // idx = -1                                                                           <L 2671>
    }
    var_145 = wp::where(var_135, var_144, var_127);
    // if (geomtype1 != GeomType.BOX and geomtype1 != GeomType.MESH and geomtype1 != GeomType.CYLINDER) or (       <L 2674>
    var_149 = (var_geomtype1 != var_148);
    var_147 = var_149;
    if (var_147) {
        var_151 = (var_geomtype1 != var_150);
        var_147 = var_147 && var_151;
    }
    if (var_147) {
        var_153 = (var_geomtype1 != var_152);
        var_147 = var_147 && var_153;
    }
    var_146 = var_147;
    if (!var_146) {
        // geomtype2 != GeomType.BOX and geomtype2 != GeomType.MESH and geomtype2 != GeomType.CYLINDER       <L 2675>
        var_156 = (var_geomtype2 != var_155);
        var_154 = var_156;
        if (var_154) {
            var_158 = (var_geomtype2 != var_157);
            var_154 = var_154 && var_158;
        }
        if (var_154) {
            var_160 = (var_geomtype2 != var_159);
            var_154 = var_154 && var_160;
        }
        var_146 = var_146 || var_154;
    }
    if (var_146) {
        // idx = -1                                                                           <L 2677>
    }
    var_162 = wp::where(var_146, var_161, var_145);
    // return dist, 1, x1, x2, idx                                                            <L 2679>
    ret_0 = var_124;
    ret_1 = var_163;
    ret_2 = var_125;
    ret_3 = var_126;
    ret_4 = var_162;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2682
static CUDA_CALLABLE void ccd_0(
    wp::float32 var_tolerance,
    wp::float32 var_cutoff,
    wp::int32 var_gjk_iterations,
    wp::int32 var_epa_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_1,
    wp::vec_t<3, wp::float32> var_x_2,
    wp::array_t<wp::vec_t<3, wp::float32>> var_vert,
    wp::array_t<wp::int32> var_vert_index,
    wp::array_t<wp::int32> var_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_pr,
    wp::array_t<wp::float32> var_face_norm2,
    wp::array_t<wp::int32> var_horizon,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::int32 & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::int32 & ret_4)
{
    //---------
    // primal vars
    bool var_0;
    wp::float32 var_1;
    wp::int32 var_2;
    wp::vec_t<3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    GJKResult_8184e255 var_5;
    Geom_4022bfa7 var_6;
    Geom_4022bfa7 var_7;
    bool var_8;
    const wp::int32 var_9 = -1;
    wp::float32 var_10;
    wp::int32 var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::int32 var_14;
    //---------
    // forward
    // def ccd(                                                                               <L 2683>
    // needs_epa, dist, ncontact, x1, x2, result, geom1, geom2 = gjk_phase(                   <L 2707>
    // tolerance, cutoff, gjk_iterations, geom1, geom2, geomtype1, geomtype2, x_1, x_2        <L 2708>
    gjk_phase_0(var_tolerance, var_cutoff, var_gjk_iterations, var_geom1, var_geom2, var_geomtype1, var_geomtype2, var_x_1, var_x_2, var_0, var_1, var_2, var_3, var_4, var_5, var_6, var_7);
    // if not needs_epa:                                                                      <L 2710>
    var_8 = wp::unot(var_0);
    if (var_8) {
        // return dist, ncontact, x1, x2, -1                                                  <L 2711>
        ret_0 = var_1;
        ret_1 = var_2;
        ret_2 = var_3;
        ret_3 = var_4;
        ret_4 = var_9;
        return;
    }
    // return epa_phase(                                                                      <L 2712>
    // tolerance,                                                                             <L 2713>
    // epa_iterations,                                                                        <L 2714>
    // result,                                                                                <L 2715>
    // geom1,                                                                                 <L 2716>
    // geom2,                                                                                 <L 2717>
    // geomtype1,                                                                             <L 2718>
    // geomtype2,                                                                             <L 2719>
    // vert,                                                                                  <L 2720>
    // vert_index,                                                                            <L 2721>
    // face,                                                                                  <L 2722>
    // face_pr,                                                                               <L 2723>
    // face_norm2,                                                                            <L 2724>
    // horizon,                                                                               <L 2725>
    // warn_overflow,                                                                         <L 2726>
    // worldid,                                                                               <L 2727>
    // overflow_out,                                                                          <L 2728>
    epa_phase_0(var_tolerance, var_epa_iterations, var_5, var_6, var_7, var_geomtype1, var_geomtype2, var_vert, var_vert_index, var_face, var_face_pr, var_face_norm2, var_horizon, var_warn_overflow, var_worldid, var_overflow_out, var_10, var_11, var_12, var_13, var_14);
    ret_0 = var_10;
    ret_1 = var_11;
    ret_2 = var_12;
    ret_3 = var_13;
    ret_4 = var_14;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:119
static CUDA_CALLABLE void adj_support_0(
    Geom_4022bfa7 var_geom,
    wp::int32 var_geomtype,
    wp::vec_t<3, wp::float32> var_dir,
    Geom_4022bfa7 & adj_geom,
    wp::int32 & adj_geomtype,
    wp::vec_t<3, wp::float32> & adj_dir,
    SupportPoint_e82efc60 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_convex.py:61
static CUDA_CALLABLE void adj__hfield_filter_0(
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::float32> var_geom_rbound,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::int32> var_mesh_graph,
    wp::array_t<wp::vec_t<4, wp::float32>> var_hfield_size,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_worldid,
    wp::int32 var_g1,
    wp::int32 var_g2,
    bool & ret_0,
    wp::float32 & ret_1,
    wp::float32 & ret_2,
    wp::float32 & ret_3,
    wp::float32 & ret_4,
    wp::float32 & ret_5,
    wp::float32 & ret_6,
    wp::array_t<wp::int32> & adj_geom_type,
    wp::array_t<wp::int32> & adj_geom_dataid,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_size,
    wp::array_t<wp::float32> & adj_geom_rbound,
    wp::array_t<wp::float32> & adj_geom_margin,
    wp::array_t<wp::int32> & adj_mesh_vertadr,
    wp::array_t<wp::int32> & adj_mesh_vertnum,
    wp::array_t<wp::int32> & adj_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_vert,
    wp::array_t<wp::int32> & adj_mesh_graph,
    wp::array_t<wp::vec_t<4, wp::float32>> & adj_hfield_size,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> & adj_geom_xmat_in,
    wp::int32 & adj_worldid,
    wp::int32 & adj_g1,
    wp::int32 & adj_g2,
    bool & adj_ret_0,
    wp::float32 & adj_ret_1,
    wp::float32 & adj_ret_2,
    wp::float32 & adj_ret_3,
    wp::float32 & adj_ret_4,
    wp::float32 & adj_ret_5,
    wp::float32 & adj_ret_6)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:112
static CUDA_CALLABLE void adj__discrete_geoms_0(
    wp::int32 var_g1,
    wp::int32 var_g2,
    wp::int32 & adj_g1,
    wp::int32 & adj_g2,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:617
static CUDA_CALLABLE void adj__gjk_support_0(
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_k,
    wp::float32 var_x_norm,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::int32 var_n,
    bool var_is_discrete,
    SupportPoint_e82efc60 & ret_0,
    SupportPoint_e82efc60 & ret_1,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::vec_t<3, wp::float32> & adj_x_k,
    wp::float32 & adj_x_norm,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::int32 & adj_n,
    bool & adj_is_discrete,
    SupportPoint_e82efc60 & adj_ret_0,
    SupportPoint_e82efc60 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:315
static CUDA_CALLABLE void adj__det3_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_v3,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:320
static CUDA_CALLABLE void adj__same_sign_0(
    wp::float32 var_a,
    wp::float32 var_b,
    wp::float32 & adj_a,
    wp::float32 & adj_b,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:336
static CUDA_CALLABLE void adj__project_origin_plane_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::int32 & ret_1,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_v3,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::int32 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:329
static CUDA_CALLABLE void adj__project_origin_line_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:589
static CUDA_CALLABLE void adj__S1D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2,
    wp::vec_t<3, wp::float32> & adj_s1,
    wp::vec_t<3, wp::float32> & adj_s2,
    wp::vec_t<2, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:444
static CUDA_CALLABLE void adj__S2D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2,
    wp::vec_t<3, wp::float32> var_s3,
    wp::vec_t<3, wp::float32> & adj_s1,
    wp::vec_t<3, wp::float32> & adj_s2,
    wp::vec_t<3, wp::float32> & adj_s3,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:368
static CUDA_CALLABLE void adj__S3D_0(
    wp::vec_t<3, wp::float32> var_s1,
    wp::vec_t<3, wp::float32> var_s2,
    wp::vec_t<3, wp::float32> var_s3,
    wp::vec_t<3, wp::float32> var_s4,
    wp::vec_t<3, wp::float32> & adj_s1,
    wp::vec_t<3, wp::float32> & adj_s2,
    wp::vec_t<3, wp::float32> & adj_s3,
    wp::vec_t<3, wp::float32> & adj_s4,
    wp::vec_t<4, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:302
static CUDA_CALLABLE void adj__subdistance_0(
    wp::int32 var_n,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::int32 & adj_n,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::vec_t<4, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:291
static CUDA_CALLABLE void adj__linear_combine_0(
    wp::int32 var_n,
    wp::vec_t<4, wp::float32> var_scl,
    wp::mat_t<4, 3, wp::float32> var_mat,
    wp::int32 & adj_n,
    wp::vec_t<4, wp::float32> & adj_scl,
    wp::mat_t<4, 3, wp::float32> & adj_mat,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:656
static CUDA_CALLABLE void adj_gjk_0(
    wp::float32 var_tolerance,
    wp::int32 var_gjk_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::vec_t<3, wp::float32> var_x1_0,
    wp::vec_t<3, wp::float32> var_x2_0,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::float32 var_cutoff,
    bool var_is_discrete,
    wp::float32 & adj_tolerance,
    wp::int32 & adj_gjk_iterations,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::vec_t<3, wp::float32> & adj_x1_0,
    wp::vec_t<3, wp::float32> & adj_x2_0,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::float32 & adj_cutoff,
    bool & adj_is_discrete,
    GJKResult_8184e255 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:807
static CUDA_CALLABLE void adj__tri_affine_coord_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_v3,
    wp::vec_t<3, wp::float32> & adj_p,
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
    //---------
    // primal vars
    wp::float32 var_0;
    //---------
    // dual vars
    wp::float32 adj_0 = {};
    //---------
    // forward
    // def norm_l2(v: Any) -> float:                                                          <L 1>
    // return wp.length(v)                                                                    <L 12>
    var_0 = wp::length(var_v);
    goto label0;
    //---------
    // reverse
    label0:;
    adj_0 += adj_ret;
    wp::adj_length(var_v, var_0, adj_v, adj_0);
    // adj: return wp.length(v)                                                               <L 12>
    // adj: def norm_l2(v: Any) -> float:                                                     <L 1>
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2454
static CUDA_CALLABLE void adj__inflate_0(
    GJKResult_8184e255 var_result,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::float32 var_margin1,
    wp::float32 var_margin2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    GJKResult_8184e255 & adj_result,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::float32 & adj_margin1,
    wp::float32 & adj_margin2,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2501
static CUDA_CALLABLE void adj_gjk_phase_0(
    wp::float32 var_tolerance,
    wp::float32 var_cutoff,
    wp::int32 var_gjk_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_1,
    wp::vec_t<3, wp::float32> var_x_2,
    bool & ret_0,
    wp::float32 & ret_1,
    wp::int32 & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::vec_t<3, wp::float32> & ret_4,
    GJKResult_8184e255 & ret_5,
    Geom_4022bfa7 & ret_6,
    Geom_4022bfa7 & ret_7,
    wp::float32 & adj_tolerance,
    wp::float32 & adj_cutoff,
    wp::int32 & adj_gjk_iterations,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::vec_t<3, wp::float32> & adj_x_1,
    wp::vec_t<3, wp::float32> & adj_x_2,
    bool & adj_ret_0,
    wp::float32 & adj_ret_1,
    wp::int32 & adj_ret_2,
    wp::vec_t<3, wp::float32> & adj_ret_3,
    wp::vec_t<3, wp::float32> & adj_ret_4,
    GJKResult_8184e255 & adj_ret_5,
    Geom_4022bfa7 & adj_ret_6,
    Geom_4022bfa7 & adj_ret_7)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:906
static CUDA_CALLABLE void adj__rotmat_0(
    wp::vec_t<3, wp::float32> var_axis,
    wp::vec_t<3, wp::float32> & adj_axis,
    wp::mat_t<3, 3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:274
static CUDA_CALLABLE void adj__epa_support_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_idx,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geom1_type,
    wp::int32 var_geom2_type,
    wp::vec_t<3, wp::float32> var_dir,
    wp::int32 & ret_0,
    wp::int32 & ret_1,
    Polytope_10582b13 & adj_pt,
    wp::int32 & adj_idx,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geom1_type,
    wp::int32 & adj_geom2_type,
    wp::vec_t<3, wp::float32> & adj_dir,
    wp::int32 & adj_ret_0,
    wp::int32 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:248
static CUDA_CALLABLE void adj__attach_face_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_idx,
    wp::int32 var_v1,
    wp::int32 var_v2,
    wp::int32 var_v3,
    Polytope_10582b13 & adj_pt,
    wp::int32 & adj_idx,
    wp::int32 & adj_v1,
    wp::int32 & adj_v2,
    wp::int32 & adj_v3,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:867
static CUDA_CALLABLE void adj__replace_simplex3_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_v1,
    wp::int32 var_v2,
    wp::int32 var_v3,
    Polytope_10582b13 & adj_pt,
    wp::int32 & adj_v1,
    wp::int32 & adj_v2,
    wp::int32 & adj_v3,
    GJKResult_8184e255 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:928
static CUDA_CALLABLE void adj__ray_triangle_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_v4,
    wp::vec_t<3, wp::float32> var_v5,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_v3,
    wp::vec_t<3, wp::float32> & adj_v4,
    wp::vec_t<3, wp::float32> & adj_v5,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1042
static CUDA_CALLABLE void adj__polytope2_0(
    Polytope_10582b13 var_pt,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    Polytope_10582b13 & ret_0,
    GJKResult_8184e255 & ret_1,
    Polytope_10582b13 & adj_pt,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::mat_t<4, 3, wp::float32> & adj_simplex1,
    wp::mat_t<4, 3, wp::float32> & adj_simplex2,
    wp::vec_t<4, wp::int32> & adj_simplex_index1,
    wp::vec_t<4, wp::int32> & adj_simplex_index2,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    Polytope_10582b13 & adj_ret_0,
    GJKResult_8184e255 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:794
static CUDA_CALLABLE void adj__same_side_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_p3,
    wp::vec_t<3, wp::float32> & adj_p0,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    wp::vec_t<3, wp::float32> & adj_p3,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:802
static CUDA_CALLABLE void adj__test_tetra_0(
    wp::vec_t<3, wp::float32> var_p0,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<3, wp::float32> var_p3,
    wp::vec_t<3, wp::float32> & adj_p0,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    wp::vec_t<3, wp::float32> & adj_p3,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1228
static CUDA_CALLABLE void adj__polytope4_0(
    Polytope_10582b13 var_pt,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Polytope_10582b13 & ret_0,
    GJKResult_8184e255 & ret_1,
    Polytope_10582b13 & adj_pt,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::mat_t<4, 3, wp::float32> & adj_simplex1,
    wp::mat_t<4, 3, wp::float32> & adj_simplex2,
    wp::vec_t<4, wp::int32> & adj_simplex_index1,
    wp::vec_t<4, wp::int32> & adj_simplex_index2,
    Polytope_10582b13 & adj_ret_0,
    GJKResult_8184e255 & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:850
static CUDA_CALLABLE void adj__tri_point_intersect_0(
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_v3,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_v3,
    wp::vec_t<3, wp::float32> & adj_p,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1135
static CUDA_CALLABLE void adj__polytope3_0(
    Polytope_10582b13 var_pt,
    wp::float32 var_dist,
    wp::mat_t<4, 3, wp::float32> var_simplex,
    wp::mat_t<4, 3, wp::float32> var_simplex1,
    wp::mat_t<4, 3, wp::float32> var_simplex2,
    wp::vec_t<4, wp::int32> var_simplex_index1,
    wp::vec_t<4, wp::int32> var_simplex_index2,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    Polytope_10582b13 & adj_pt,
    wp::float32 & adj_dist,
    wp::mat_t<4, 3, wp::float32> & adj_simplex,
    wp::mat_t<4, 3, wp::float32> & adj_simplex1,
    wp::mat_t<4, 3, wp::float32> & adj_simplex2,
    wp::vec_t<4, wp::int32> & adj_simplex_index1,
    wp::vec_t<4, wp::int32> & adj_simplex_index2,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    Polytope_10582b13 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1334
static CUDA_CALLABLE void adj__is_invalid_face_0(
    wp::int32 var_face,
    wp::int32 & adj_face,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1316
static CUDA_CALLABLE void adj__delete_face_0(
    wp::int32 var_face,
    wp::int32 & adj_face,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1310
static CUDA_CALLABLE void adj__get_face_verts_0(
    wp::int32 var_face,
    wp::int32 & adj_face,
    wp::vec_t<3, wp::int32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:946
static CUDA_CALLABLE void adj__add_edge_0(
    Polytope_10582b13 var_pt,
    wp::int32 var_e1,
    wp::int32 var_e2,
    Polytope_10582b13 & adj_pt,
    wp::int32 & adj_e1,
    wp::int32 & adj_e2,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1322
static CUDA_CALLABLE void adj__is_face_deleted_0(
    wp::int32 var_face,
    wp::int32 & adj_face,
    bool & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:941
static CUDA_CALLABLE void adj__get_edge_0(
    wp::int32 var_edge,
    wp::int32 & adj_edge,
    wp::vec_t<2, wp::int32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1328
static CUDA_CALLABLE void adj__invalidate_face_0(
    wp::int32 var_face,
    wp::int32 & adj_face,
    wp::int32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:968
static CUDA_CALLABLE void adj__epa_witness_0(
    Polytope_10582b13 var_pt,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::int32 var_face_idx,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::float32 & ret_2,
    Polytope_10582b13 & adj_pt,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::int32 & adj_face_idx,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::float32 & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:1340
static CUDA_CALLABLE void adj__epa_0(
    wp::float32 var_tolerance,
    wp::int32 var_epa_iterations,
    Polytope_10582b13 var_pt,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    bool var_is_discrete,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::int32 & ret_3,
    wp::float32 & adj_tolerance,
    wp::int32 & adj_epa_iterations,
    Polytope_10582b13 & adj_pt,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    bool & adj_is_discrete,
    bool & adj_warn_overflow,
    wp::int32 & adj_worldid,
    wp::array_t<wp::int32> & adj_overflow_out,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2,
    wp::int32 & adj_ret_3)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2572
static CUDA_CALLABLE void adj_epa_phase_0(
    wp::float32 var_tolerance,
    wp::int32 var_epa_iterations,
    GJKResult_8184e255 var_result,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::array_t<wp::vec_t<3, wp::float32>> var_vert,
    wp::array_t<wp::int32> var_vert_index,
    wp::array_t<wp::int32> var_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_pr,
    wp::array_t<wp::float32> var_face_norm2,
    wp::array_t<wp::int32> var_horizon,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::int32 & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::int32 & ret_4,
    wp::float32 & adj_tolerance,
    wp::int32 & adj_epa_iterations,
    GJKResult_8184e255 & adj_result,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_vert,
    wp::array_t<wp::int32> & adj_vert_index,
    wp::array_t<wp::int32> & adj_face,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_face_pr,
    wp::array_t<wp::float32> & adj_face_norm2,
    wp::array_t<wp::int32> & adj_horizon,
    bool & adj_warn_overflow,
    wp::int32 & adj_worldid,
    wp::array_t<wp::int32> & adj_overflow_out,
    wp::float32 & adj_ret_0,
    wp::int32 & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2,
    wp::vec_t<3, wp::float32> & adj_ret_3,
    wp::int32 & adj_ret_4)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_gjk.py:2682
static CUDA_CALLABLE void adj_ccd_0(
    wp::float32 var_tolerance,
    wp::float32 var_cutoff,
    wp::int32 var_gjk_iterations,
    wp::int32 var_epa_iterations,
    Geom_4022bfa7 var_geom1,
    Geom_4022bfa7 var_geom2,
    wp::int32 var_geomtype1,
    wp::int32 var_geomtype2,
    wp::vec_t<3, wp::float32> var_x_1,
    wp::vec_t<3, wp::float32> var_x_2,
    wp::array_t<wp::vec_t<3, wp::float32>> var_vert,
    wp::array_t<wp::int32> var_vert_index,
    wp::array_t<wp::int32> var_face,
    wp::array_t<wp::vec_t<3, wp::float32>> var_face_pr,
    wp::array_t<wp::float32> var_face_norm2,
    wp::array_t<wp::int32> var_horizon,
    bool var_warn_overflow,
    wp::int32 var_worldid,
    wp::array_t<wp::int32> var_overflow_out,
    wp::float32 & ret_0,
    wp::int32 & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & ret_3,
    wp::int32 & ret_4,
    wp::float32 & adj_tolerance,
    wp::float32 & adj_cutoff,
    wp::int32 & adj_gjk_iterations,
    wp::int32 & adj_epa_iterations,
    Geom_4022bfa7 & adj_geom1,
    Geom_4022bfa7 & adj_geom2,
    wp::int32 & adj_geomtype1,
    wp::int32 & adj_geomtype2,
    wp::vec_t<3, wp::float32> & adj_x_1,
    wp::vec_t<3, wp::float32> & adj_x_2,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_vert,
    wp::array_t<wp::int32> & adj_vert_index,
    wp::array_t<wp::int32> & adj_face,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_face_pr,
    wp::array_t<wp::float32> & adj_face_norm2,
    wp::array_t<wp::int32> & adj_horizon,
    bool & adj_warn_overflow,
    wp::int32 & adj_worldid,
    wp::array_t<wp::int32> & adj_overflow_out,
    wp::float32 & adj_ret_0,
    wp::int32 & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2,
    wp::vec_t<3, wp::float32> & adj_ret_3,
    wp::int32 & adj_ret_4)
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



extern "C" __global__ void ccd_hfield_kernel_builder__locals__ccd_hfield_kernel_7343d62c_cuda_kernel_forward(
    wp::launch_bounds_t<1> dim,
    wp::array_t<wp::float32> var_opt_ccd_tolerance,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::float32> var_geom_rbound,
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
    wp::array_t<wp::vec_t<4, wp::float32>> var_hfield_size,
    wp::array_t<wp::int32> var_hfield_nrow,
    wp::array_t<wp::int32> var_hfield_ncol,
    wp::array_t<wp::int32> var_hfield_adr,
    wp::array_t<wp::float32> var_hfield_data,
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
    wp::int32 var_naccdmax_in,
    wp::array_t<wp::int32> var_ncollision_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_in,
    wp::array_t<wp::int32> var_collision_worldid_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_epa_vert_in,
    wp::array_t<wp::int32> var_epa_vert_index_in,
    wp::array_t<wp::int32> var_epa_face_in,
    wp::array_t<wp::vec_t<3, wp::float32>> var_epa_pr_in,
    wp::array_t<wp::float32> var_epa_norm2_in,
    wp::array_t<wp::int32> var_epa_horizon_in,
    wp::array_t<wp::int32> var_nccd_in,
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
    wp::array_t<wp::int32> var_overflow_out)
{
    wp::tile_shared_storage_t tile_mem;

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
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
        const wp::int32 var_10 = 1;
        wp::int32 var_11;
        bool var_12;
        wp::int32* var_13;
        const wp::int32 var_14 = 1;
        bool var_15;
        wp::int32 var_16;
        wp::int32* var_17;
        const wp::int32 var_18 = 6;
        bool var_19;
        wp::int32 var_20;
        wp::int32* var_21;
        wp::int32 var_22;
        wp::int32 var_23;
        bool var_24;
        wp::float32 var_25;
        wp::float32 var_26;
        wp::float32 var_27;
        wp::float32 var_28;
        wp::float32 var_29;
        wp::float32 var_30;
        const wp::int32 var_31 = 16;
        const wp::int32 var_32 = 1;
        wp::int32 var_33;
        bool var_34;
        const bool var_35 = true;
        const wp::str var_36 = "CCD overflow - please increase naccdmax beyond %u\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.CCD (or = 0 for all)\n";
        const wp::int32 var_37 = 16;
        wp::int32 var_38;
        wp::vec_t<2, wp::int32> var_39;
        wp::float32 var_40;
        wp::float32 var_41;
        wp::int32 var_42;
        wp::vec_t<5, wp::float32> var_43;
        wp::vec_t<2, wp::float32> var_44;
        wp::vec_t<2, wp::float32> var_45;
        wp::vec_t<5, wp::float32> var_46;
        wp::float32 var_47;
        Geom_4022bfa7 var_48;
        Geom_4022bfa7 var_49;
        wp::vec_t<3, wp::float32>* var_50;
        wp::vec_t<3, wp::float32> var_51;
        wp::vec_t<3, wp::float32> var_52;
        wp::mat_t<3, 3, wp::float32>* var_53;
        wp::mat_t<3, 3, wp::float32> var_54;
        wp::mat_t<3, 3, wp::float32> var_55;
        wp::mat_t<3, 3, wp::float32> var_56;
        wp::vec_t<3, wp::float32>* var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::vec_t<3, wp::float32> var_60;
        wp::mat_t<3, 3, wp::float32>* var_61;
        wp::mat_t<3, 3, wp::float32> var_62;
        wp::mat_t<3, 3, wp::float32> var_63;
        const wp::float32 var_64 = 0.0;
        const wp::float32 var_65 = 0.0;
        const wp::float32 var_66 = 0.0;
        wp::vec_t<3, wp::float32> var_67;
        const wp::int32 var_68 = 3;
        wp::mat_t<3, 3, wp::float32> var_69;
        wp::shape_t* var_70;
        const wp::int32 var_71 = 0;
        wp::int32 var_72;
        wp::shape_t var_73;
        wp::int32 var_74;
        wp::int32* var_75;
        wp::int32 var_76;
        wp::int32 var_77;
        wp::int32* var_78;
        wp::int32 var_79;
        wp::int32 var_80;
        wp::int32* var_81;
        wp::int32 var_82;
        wp::int32 var_83;
        wp::vec_t<4, wp::float32>* var_84;
        wp::vec_t<4, wp::float32> var_85;
        wp::vec_t<4, wp::float32> var_86;
        const wp::float32 var_87 = 0.5;
        const wp::int32 var_88 = 1;
        wp::int32 var_89;
        wp::float32 var_90;
        wp::float32 var_91;
        const wp::int32 var_92 = 0;
        wp::float32 var_93;
        wp::float32 var_94;
        const wp::float32 var_95 = 0.5;
        const wp::int32 var_96 = 1;
        wp::int32 var_97;
        wp::float32 var_98;
        wp::float32 var_99;
        const wp::int32 var_100 = 1;
        wp::float32 var_101;
        wp::float32 var_102;
        const wp::int32 var_103 = 0;
        const wp::int32 var_104 = 0;
        wp::float32 var_105;
        wp::float32 var_106;
        wp::float32 var_107;
        wp::float32 var_108;
        wp::int32 var_109;
        wp::int32 var_110;
        const wp::int32 var_111 = 1;
        wp::int32 var_112;
        const wp::int32 var_113 = 0;
        wp::float32 var_114;
        wp::float32 var_115;
        wp::float32 var_116;
        wp::float32 var_117;
        wp::int32 var_118;
        wp::int32 var_119;
        const wp::int32 var_120 = 0;
        const wp::int32 var_121 = 1;
        wp::float32 var_122;
        wp::float32 var_123;
        wp::float32 var_124;
        wp::float32 var_125;
        wp::int32 var_126;
        wp::int32 var_127;
        const wp::int32 var_128 = 1;
        wp::int32 var_129;
        const wp::int32 var_130 = 1;
        wp::float32 var_131;
        wp::float32 var_132;
        wp::float32 var_133;
        wp::float32 var_134;
        wp::int32 var_135;
        wp::int32 var_136;
        const wp::float32 var_137 = 2.0;
        const wp::int32 var_138 = 0;
        wp::float32 var_139;
        wp::float32 var_140;
        const wp::int32 var_141 = 1;
        wp::int32 var_142;
        wp::float32 var_143;
        wp::float32 var_144;
        const wp::float32 var_145 = 2.0;
        const wp::int32 var_146 = 1;
        wp::float32 var_147;
        wp::float32 var_148;
        const wp::int32 var_149 = 1;
        wp::int32 var_150;
        wp::float32 var_151;
        wp::float32 var_152;
        const wp::int32 var_153 = 1;
        const wp::int32 var_154 = 0;
        wp::vec_t<2, wp::int32> var_155;
        wp::mat_t<6, 3, wp::float32> var_156;
        const wp::int32 var_157 = 3;
        wp::float32 var_158;
        wp::float32 var_159;
        const wp::int32 var_160 = 0;
        const wp::int32 var_161 = 2;
        const wp::int32 var_162 = 3;
        wp::float32 var_163;
        wp::float32 var_164;
        const wp::int32 var_165 = 1;
        const wp::int32 var_166 = 2;
        const wp::int32 var_167 = 3;
        wp::float32 var_168;
        wp::float32 var_169;
        const wp::int32 var_170 = 2;
        const wp::int32 var_171 = 2;
        wp::int32* var_172;
        wp::int32 var_173;
        wp::int32 var_174;
        wp::vec_t<50, wp::float32> var_175;
        wp::mat_t<50, 3, wp::float32> var_176;
        wp::mat_t<50, 3, wp::float32> var_177;
        const wp::float32 var_178 = 10000000000.0;
        wp::float32 var_179;
        wp::vec_t<3, wp::float32> var_180;
        wp::vec_t<3, wp::float32> var_181;
        const wp::int32 var_182 = -1;
        wp::int32 var_183;
        wp::slice_t var_184;
        const wp::int32 var_185 = 0;
        wp::array_t<wp::vec_t<3, wp::float32>> var_186;
        wp::slice_t var_187;
        const wp::int32 var_188 = 0;
        wp::array_t<wp::int32> var_189;
        wp::slice_t var_190;
        const wp::int32 var_191 = 0;
        wp::array_t<wp::int32> var_192;
        wp::slice_t var_193;
        const wp::int32 var_194 = 0;
        wp::array_t<wp::vec_t<3, wp::float32>> var_195;
        wp::slice_t var_196;
        const wp::int32 var_197 = 0;
        wp::array_t<wp::float32> var_198;
        wp::slice_t var_199;
        const wp::int32 var_200 = 0;
        wp::array_t<wp::int32> var_201;
        wp::vec_t<2, wp::int32>* var_202;
        wp::vec_t<2, wp::int32> var_203;
        wp::vec_t<2, wp::int32> var_204;
        const wp::int32 var_205 = 0;
        wp::int32 var_206;
        const wp::int32 var_207 = 0;
        wp::int32 var_208;
        wp::range_t var_209;
        wp::int32 var_210;
        const wp::int32 var_211 = 0;
        bool var_212;
        const wp::int32 var_213 = 0;
        wp::float32 var_214;
        wp::float32 var_215;
        const wp::int32 var_216 = 0;
        wp::float32 var_217;
        wp::float32 var_218;
        wp::int32 var_219;
        wp::int32 var_220;
        wp::float32 var_221;
        wp::float32 var_222;
        const wp::int32 var_223 = 1;
        wp::float32 var_224;
        wp::float32 var_225;
        wp::int32 var_226;
        wp::int32 var_227;
        wp::int32 var_228;
        wp::int32 var_229;
        wp::int32 var_230;
        wp::float32* var_231;
        const wp::int32 var_232 = 2;
        wp::float32 var_233;
        wp::float32 var_234;
        wp::float32 var_235;
        wp::float32 var_236;
        const wp::int32 var_237 = 1;
        wp::vec_t<3, wp::float32> var_238;
        const wp::int32 var_239 = 0;
        const wp::int32 var_240 = 2;
        wp::vec_t<3, wp::float32> var_241;
        const wp::int32 var_242 = 1;
        const wp::int32 var_243 = 4;
        wp::vec_t<3, wp::float32> var_244;
        const wp::int32 var_245 = 3;
        const wp::int32 var_246 = 5;
        wp::vec_t<3, wp::float32> var_247;
        const wp::int32 var_248 = 4;
        const wp::int32 var_249 = 2;
        const wp::int32 var_250 = 0;
        const wp::int32 var_251 = 5;
        const wp::int32 var_252 = 0;
        const wp::int32 var_253 = 2;
        const wp::int32 var_254 = 1;
        const wp::int32 var_255 = 5;
        const wp::int32 var_256 = 1;
        const wp::int32 var_257 = 5;
        const wp::int32 var_258 = 2;
        const wp::int32 var_259 = 1;
        wp::float32 var_260;
        wp::float32 var_261;
        const wp::int32 var_262 = 0;
        wp::float32 var_263;
        wp::float32 var_264;
        wp::int32 var_265;
        wp::int32 var_266;
        wp::float32 var_267;
        wp::float32 var_268;
        const wp::int32 var_269 = 1;
        wp::float32 var_270;
        wp::float32 var_271;
        wp::int32 var_272;
        wp::int32 var_273;
        wp::int32 var_274;
        wp::int32 var_275;
        wp::int32 var_276;
        wp::float32* var_277;
        const wp::int32 var_278 = 2;
        wp::float32 var_279;
        wp::float32 var_280;
        wp::float32 var_281;
        wp::float32 var_282;
        const wp::int32 var_283 = 1;
        wp::vec_t<3, wp::float32> var_284;
        const wp::int32 var_285 = 0;
        const wp::int32 var_286 = 2;
        wp::vec_t<3, wp::float32> var_287;
        const wp::int32 var_288 = 1;
        const wp::int32 var_289 = 4;
        wp::vec_t<3, wp::float32> var_290;
        const wp::int32 var_291 = 3;
        const wp::int32 var_292 = 5;
        wp::vec_t<3, wp::float32> var_293;
        const wp::int32 var_294 = 4;
        const wp::int32 var_295 = 2;
        const wp::int32 var_296 = 0;
        const wp::int32 var_297 = 5;
        const wp::int32 var_298 = 0;
        const wp::int32 var_299 = 2;
        const wp::int32 var_300 = 1;
        const wp::int32 var_301 = 5;
        const wp::int32 var_302 = 1;
        const wp::int32 var_303 = 5;
        const wp::int32 var_304 = 2;
        const wp::int32 var_305 = 1;
        wp::int32 var_306;
        const wp::int32 var_307 = 1;
        wp::int32 var_308;
        wp::range_t var_309;
        wp::int32 var_310;
        const wp::int32 var_311 = 0;
        bool var_312;
        const wp::int32 var_313 = 2;
        wp::range_t var_314;
        wp::int32 var_315;
        const wp::int32 var_316 = 50;
        bool var_317;
        const wp::int32 var_318 = 1;
        wp::float32 var_319;
        wp::float32 var_320;
        const wp::int32 var_321 = 0;
        wp::float32 var_322;
        wp::float32 var_323;
        wp::int32 var_324;
        wp::int32 var_325;
        wp::float32 var_326;
        wp::float32 var_327;
        const wp::int32 var_328 = 1;
        wp::float32 var_329;
        wp::float32 var_330;
        wp::int32 var_331;
        wp::int32 var_332;
        wp::int32 var_333;
        wp::int32 var_334;
        wp::int32 var_335;
        wp::float32* var_336;
        const wp::int32 var_337 = 2;
        wp::float32 var_338;
        wp::float32 var_339;
        wp::float32 var_340;
        wp::float32 var_341;
        const wp::int32 var_342 = 1;
        wp::vec_t<3, wp::float32> var_343;
        const wp::int32 var_344 = 0;
        const wp::int32 var_345 = 2;
        wp::vec_t<3, wp::float32> var_346;
        const wp::int32 var_347 = 1;
        const wp::int32 var_348 = 4;
        wp::vec_t<3, wp::float32> var_349;
        const wp::int32 var_350 = 3;
        const wp::int32 var_351 = 5;
        wp::vec_t<3, wp::float32> var_352;
        const wp::int32 var_353 = 4;
        const wp::int32 var_354 = 2;
        const wp::int32 var_355 = 0;
        const wp::int32 var_356 = 5;
        const wp::int32 var_357 = 0;
        const wp::int32 var_358 = 2;
        const wp::int32 var_359 = 1;
        const wp::int32 var_360 = 5;
        const wp::int32 var_361 = 1;
        const wp::int32 var_362 = 5;
        const wp::int32 var_363 = 2;
        bool var_364;
        const wp::int32 var_365 = 3;
        const wp::int32 var_366 = 2;
        wp::float32 var_367;
        bool var_368;
        const wp::int32 var_369 = 4;
        const wp::int32 var_370 = 2;
        wp::float32 var_371;
        bool var_372;
        const wp::int32 var_373 = 5;
        const wp::int32 var_374 = 2;
        wp::float32 var_375;
        bool var_376;
        wp::float32 var_377;
        wp::float32 var_378;
        wp::float32 var_379;
        wp::vec_t<3, wp::float32>* var_380;
        wp::mat_t<3, 3, wp::float32>* var_381;
        const wp::int32 var_382 = 0;
        wp::vec_t<3, wp::float32> var_383;
        const wp::int32 var_384 = 1;
        wp::vec_t<3, wp::float32> var_385;
        wp::vec_t<3, wp::float32> var_386;
        const wp::int32 var_387 = 2;
        wp::vec_t<3, wp::float32> var_388;
        wp::vec_t<3, wp::float32> var_389;
        const wp::int32 var_390 = 3;
        wp::vec_t<3, wp::float32> var_391;
        wp::vec_t<3, wp::float32> var_392;
        const wp::int32 var_393 = 4;
        wp::vec_t<3, wp::float32> var_394;
        wp::vec_t<3, wp::float32> var_395;
        const wp::int32 var_396 = 5;
        wp::vec_t<3, wp::float32> var_397;
        wp::vec_t<3, wp::float32> var_398;
        wp::vec_t<3, wp::float32> var_399;
        wp::mat_t<3, 3, wp::float32> var_400;
        const wp::float32 var_401 = 0.16666666666666666;
        wp::vec_t<3, wp::float32> var_402;
        wp::vec_t<3, wp::float32> var_403;
        wp::vec_t<3, wp::float32> var_404;
        wp::shape_t* var_405;
        const wp::int32 var_406 = 0;
        wp::int32 var_407;
        wp::shape_t var_408;
        wp::int32 var_409;
        wp::float32* var_410;
        const wp::float32 var_411 = 0.0;
        const wp::int32 var_412 = 35;
        const wp::int32 var_413 = 35;
        wp::vec_t<3, wp::float32>* var_414;
        const bool var_415 = true;
        wp::float32 var_416;
        wp::int32 var_417;
        wp::vec_t<3, wp::float32> var_418;
        wp::vec_t<3, wp::float32> var_419;
        wp::int32 var_420;
        wp::float32 var_421;
        wp::vec_t<3, wp::float32> var_422;
        bool var_423;
        const wp::int32 var_424 = 0;
        bool var_425;
        wp::float32 var_426;
        bool var_427;
        wp::float32 var_428;
        wp::float32 var_429;
        wp::float32 var_430;
        const wp::float32 var_431 = 0.5;
        wp::vec_t<3, wp::float32> var_432;
        wp::vec_t<3, wp::float32> var_433;
        wp::vec_t<3, wp::float32> var_434;
        wp::vec_t<3, wp::float32> var_435;
        const wp::int32 var_436 = 0;
        wp::float32 var_437;
        const wp::int32 var_438 = 0;
        const wp::int32 var_439 = 1;
        wp::float32 var_440;
        const wp::int32 var_441 = 1;
        const wp::int32 var_442 = 2;
        wp::float32 var_443;
        const wp::int32 var_444 = 2;
        wp::vec_t<3, wp::float32> var_445;
        wp::mat_t<3, 3, wp::float32> var_446;
        const wp::int32 var_447 = 0;
        const wp::int32 var_448 = 0;
        wp::float32 var_449;
        const wp::int32 var_450 = 0;
        const wp::int32 var_451 = 1;
        wp::float32 var_452;
        const wp::int32 var_453 = 0;
        const wp::int32 var_454 = 2;
        wp::float32 var_455;
        wp::vec_t<3, wp::float32> var_456;
        wp::vec_t<3, wp::float32> var_457;
        const wp::int32 var_458 = 0;
        wp::float32 var_459;
        const wp::int32 var_460 = 0;
        const wp::int32 var_461 = 1;
        wp::float32 var_462;
        const wp::int32 var_463 = 1;
        const wp::int32 var_464 = 2;
        wp::float32 var_465;
        const wp::int32 var_466 = 2;
        bool var_467;
        wp::float32 var_468;
        wp::vec_t<3, wp::float32> var_469;
        wp::vec_t<3, wp::float32> var_470;
        wp::int32 var_471;
        wp::float32 var_472;
        wp::vec_t<3, wp::float32> var_473;
        wp::vec_t<3, wp::float32> var_474;
        wp::int32 var_475;
        const wp::int32 var_476 = 1;
        wp::int32 var_477;
        const wp::int32 var_478 = 0;
        bool var_479;
        const bool var_480 = true;
        const wp::str var_481 = "height field collision overflow, number of collisions >= %u - please adjust resolution: \ndecrease the number of hfield rows/cols or modify size of colliding geom\nTo disable the print warning: m.opt.warn_overflow &= ~mjw.OverflowType.HFIELD (or = 0 for all)\n";
        const wp::int32 var_482 = 32;
        const wp::int32 var_483 = 32;
        wp::int32 var_484;
        const wp::int32 var_485 = 0;
        bool var_486;
        const wp::int32 var_487 = 0;
        wp::mat_t<3, 3, wp::float32> var_488;
        wp::int32 var_489;
        const bool var_490 = true;
        const wp::float32 var_491 = 0.001;
        const wp::int32 var_492 = -1;
        wp::int32 var_493;
        const wp::float32 var_494 = -10000000000.0;
        wp::float32 var_495;
        wp::range_t var_496;
        wp::int32 var_497;
        bool var_498;
        const wp::int32 var_499 = 0;
        wp::float32 var_500;
        const wp::int32 var_501 = 1;
        wp::float32 var_502;
        const wp::int32 var_503 = 2;
        wp::float32 var_504;
        wp::vec_t<3, wp::float32> var_505;
        wp::vec_t<3, wp::float32> var_506;
        wp::float32 var_507;
        bool var_508;
        wp::int32 var_509;
        wp::float32 var_510;
        wp::int32 var_511;
        wp::float32 var_512;
        bool var_513;
        const wp::int32 var_514 = -1;
        bool var_515;
        bool var_516;
        const wp::float32 var_517 = 0.0;
        bool var_518;
        bool var_519;
        const wp::int32 var_520 = 0;
        wp::float32 var_521;
        const wp::int32 var_522 = 1;
        wp::float32 var_523;
        const wp::int32 var_524 = 2;
        wp::float32 var_525;
        wp::vec_t<3, wp::float32> var_526;
        const wp::int32 var_527 = 0;
        wp::float32 var_528;
        const wp::int32 var_529 = 1;
        wp::float32 var_530;
        const wp::int32 var_531 = 2;
        wp::float32 var_532;
        wp::vec_t<3, wp::float32> var_533;
        const wp::int32 var_534 = 1;
        wp::float32 var_535;
        wp::mat_t<3, 3, wp::float32> var_536;
        wp::int32 var_537;
        wp::vec_t<3, wp::float32> var_538;
        wp::vec_t<3, wp::float32> var_539;
        const wp::int32 var_540 = -1;
        wp::int32 var_541;
        const wp::float32 var_542 = -10000000000.0;
        wp::float32 var_543;
        wp::range_t var_544;
        wp::int32 var_545;
        bool var_546;
        bool var_547;
        bool var_548;
        const wp::int32 var_549 = 0;
        wp::float32 var_550;
        const wp::int32 var_551 = 1;
        wp::float32 var_552;
        const wp::int32 var_553 = 2;
        wp::float32 var_554;
        wp::vec_t<3, wp::float32> var_555;
        wp::vec_t<3, wp::float32> var_556;
        wp::float32 var_557;
        wp::float32 var_558;
        bool var_559;
        wp::int32 var_560;
        wp::float32 var_561;
        wp::int32 var_562;
        wp::float32 var_563;
        bool var_564;
        const wp::int32 var_565 = -1;
        bool var_566;
        bool var_567;
        const wp::float32 var_568 = 0.0;
        bool var_569;
        bool var_570;
        const wp::int32 var_571 = 0;
        wp::float32 var_572;
        const wp::int32 var_573 = 1;
        wp::float32 var_574;
        const wp::int32 var_575 = 2;
        wp::float32 var_576;
        wp::vec_t<3, wp::float32> var_577;
        const wp::int32 var_578 = 0;
        wp::float32 var_579;
        const wp::int32 var_580 = 1;
        wp::float32 var_581;
        const wp::int32 var_582 = 2;
        wp::float32 var_583;
        wp::vec_t<3, wp::float32> var_584;
        const wp::int32 var_585 = 2;
        wp::float32 var_586;
        wp::mat_t<3, 3, wp::float32> var_587;
        wp::int32 var_588;
        wp::vec_t<3, wp::float32> var_589;
        wp::vec_t<3, wp::float32> var_590;
        wp::vec_t<3, wp::float32> var_591;
        wp::vec_t<3, wp::float32> var_592;
        const wp::int32 var_593 = -1;
        wp::int32 var_594;
        const wp::float32 var_595 = -10000000000.0;
        wp::float32 var_596;
        wp::range_t var_597;
        wp::int32 var_598;
        bool var_599;
        bool var_600;
        bool var_601;
        bool var_602;
        const wp::int32 var_603 = 0;
        wp::float32 var_604;
        const wp::int32 var_605 = 1;
        wp::float32 var_606;
        const wp::int32 var_607 = 2;
        wp::float32 var_608;
        wp::vec_t<3, wp::float32> var_609;
        wp::vec_t<3, wp::float32> var_610;
        wp::float32 var_611;
        wp::float32 var_612;
        wp::vec_t<3, wp::float32> var_613;
        wp::float32 var_614;
        wp::float32 var_615;
        wp::float32 var_616;
        bool var_617;
        wp::int32 var_618;
        wp::float32 var_619;
        wp::int32 var_620;
        wp::float32 var_621;
        bool var_622;
        const wp::int32 var_623 = -1;
        bool var_624;
        bool var_625;
        const wp::float32 var_626 = 0.0;
        bool var_627;
        bool var_628;
        const wp::int32 var_629 = 0;
        wp::float32 var_630;
        const wp::int32 var_631 = 1;
        wp::float32 var_632;
        const wp::int32 var_633 = 2;
        wp::float32 var_634;
        wp::vec_t<3, wp::float32> var_635;
        const wp::int32 var_636 = 0;
        wp::float32 var_637;
        const wp::int32 var_638 = 1;
        wp::float32 var_639;
        const wp::int32 var_640 = 2;
        wp::float32 var_641;
        wp::vec_t<3, wp::float32> var_642;
        const wp::int32 var_643 = 3;
        wp::float32 var_644;
        wp::mat_t<3, 3, wp::float32> var_645;
        wp::int32 var_646;
        //---------
        // forward
        // def ccd_hfield_kernel(                                                                 <L 176>
        // collisionid = wp.tid()                                                                 <L 256>
        var_0 = builtin_tid1d();
        // if collisionid >= ncollision_in[0]:                                                    <L 257>
        var_2 = wp::address(var_ncollision_in, var_1);
        var_4 = wp::load(var_2);
        var_3 = (var_0 >= var_4);
        if (var_3) {
            // return                                                                             <L 258>
            return;
        }
        // geoms = collision_pair_in[collisionid]                                                 <L 260>
        var_5 = wp::address(var_collision_pair_in, var_0);
        var_7 = wp::load(var_5);
        var_6 = wp::copy(var_7);
        // g1 = geoms[0]                                                                          <L 261>
        var_9 = wp::extract(var_6, var_8);
        // g2 = geoms[1]                                                                          <L 262>
        var_11 = wp::extract(var_6, var_10);
        // if geom_type[g1] != geomtype1 or geom_type[g2] != geomtype2:                           <L 264>
        var_13 = wp::address(var_geom_type, var_9);
        var_16 = wp::load(var_13);
        var_15 = (var_16 != var_14);
        var_12 = var_15;
        if (!var_12) {
            var_17 = wp::address(var_geom_type, var_11);
            var_20 = wp::load(var_17);
            var_19 = (var_20 != var_18);
            var_12 = var_12 || var_19;
        }
        if (var_12) {
            // return                                                                             <L 265>
            return;
        }
        // worldid = collision_worldid_in[collisionid]                                            <L 267>
        var_21 = wp::address(var_collision_worldid_in, var_0);
        var_23 = wp::load(var_21);
        var_22 = wp::copy(var_23);
        // no_hf_collision, xmin, xmax, ymin, ymax, zmin, zmax = _hfield_filter(                  <L 270>
        // geom_type,                                                                             <L 271>
        // geom_dataid,                                                                           <L 272>
        // geom_size,                                                                             <L 273>
        // geom_rbound,                                                                           <L 274>
        // geom_margin,                                                                           <L 275>
        // mesh_vertadr,                                                                          <L 276>
        // mesh_vertnum,                                                                          <L 277>
        // mesh_graphadr,                                                                         <L 278>
        // mesh_vert,                                                                             <L 279>
        // mesh_graph,                                                                            <L 280>
        // hfield_size,                                                                           <L 281>
        // geom_xpos_in,                                                                          <L 282>
        // geom_xmat_in,                                                                          <L 283>
        // worldid,                                                                               <L 284>
        // g1,                                                                                    <L 285>
        // g2,                                                                                    <L 286>
        _hfield_filter_0(var_geom_type, var_geom_dataid, var_geom_size, var_geom_rbound, var_geom_margin, var_mesh_vertadr, var_mesh_vertnum, var_mesh_graphadr, var_mesh_vert, var_mesh_graph, var_hfield_size, var_geom_xpos_in, var_geom_xmat_in, var_22, var_9, var_11, var_24, var_25, var_26, var_27, var_28, var_29, var_30);
        // if no_hf_collision:                                                                    <L 288>
        if (var_24) {
            // return                                                                             <L 289>
            return;
        }
        // ccdid = wp.atomic_add(nccd_in, wp.static(geomgeomid), 1)                               <L 291>
        var_33 = wp::atomic_add(var_nccd_in, var_31, var_32);
        // if ccdid >= naccdmax_in:                                                               <L 292>
        var_34 = (var_33 >= var_naccdmax_in);
        if (var_34) {
            // if wp.static(bool(warn_overflow & OverflowType.CCD)):                              <L 293>
            // wp.printf(                                                                         <L 294>
            // "CCD overflow - please increase naccdmax beyond %u\n"                              <L 295>
            // naccdmax_in,                                                                       <L 297>
            printf(var_36, var_naccdmax_in);
            // wp.atomic_or(overflow_out, worldid, wp.static(OverflowType.CCD))                   <L 299>
            var_38 = wp::atomic_or(var_overflow_out, var_22, var_37);
            // return                                                                             <L 300>
            return;
        }
        // _, margin, gap, condim, friction, solref, solreffriction, solimp, adhesion = contact_params(       <L 302>
        // geom_condim,                                                                           <L 303>
        // geom_priority,                                                                         <L 304>
        // geom_solmix,                                                                           <L 305>
        // geom_solref,                                                                           <L 306>
        // geom_solimp,                                                                           <L 307>
        // geom_friction,                                                                         <L 308>
        // geom_margin,                                                                           <L 309>
        // geom_gap,                                                                              <L 310>
        // geom_adhesion,                                                                         <L 311>
        // pair_dim,                                                                              <L 312>
        // pair_solref,                                                                           <L 313>
        // pair_solreffriction,                                                                   <L 314>
        // pair_solimp,                                                                           <L 315>
        // pair_margin,                                                                           <L 316>
        // pair_gap,                                                                              <L 317>
        // pair_adhesion,                                                                         <L 318>
        // pair_friction,                                                                         <L 319>
        // collision_pair_in,                                                                     <L 320>
        // collision_pairid_in,                                                                   <L 321>
        // collisionid,                                                                           <L 322>
        // worldid,                                                                               <L 323>
        contact_params_0(var_geom_condim, var_geom_priority, var_geom_solmix, var_geom_solref, var_geom_solimp, var_geom_friction, var_geom_margin, var_geom_gap, var_geom_adhesion, var_pair_dim, var_pair_solref, var_pair_solreffriction, var_pair_solimp, var_pair_margin, var_pair_gap, var_pair_adhesion, var_pair_friction, var_collision_pair_in, var_collision_pairid_in, var_0, var_22, var_39, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47);
        // geom1, geom2 = geom_collision_pair(                                                    <L 326>
        // geom_type,                                                                             <L 327>
        // geom_dataid,                                                                           <L 328>
        // geom_size,                                                                             <L 329>
        // mesh_vertadr,                                                                          <L 330>
        // mesh_vertnum,                                                                          <L 331>
        // mesh_graphadr,                                                                         <L 332>
        // mesh_vert,                                                                             <L 333>
        // mesh_graph,                                                                            <L 334>
        // mesh_polynum,                                                                          <L 335>
        // mesh_polyadr,                                                                          <L 336>
        // mesh_polynormal,                                                                       <L 337>
        // mesh_polyvertadr,                                                                      <L 338>
        // mesh_polyvertnum,                                                                      <L 339>
        // mesh_polyvert,                                                                         <L 340>
        // mesh_polymapadr,                                                                       <L 341>
        // mesh_polymapnum,                                                                       <L 342>
        // mesh_polymap,                                                                          <L 343>
        // geom_xpos_in,                                                                          <L 344>
        // geom_xmat_in,                                                                          <L 345>
        // geoms,                                                                                 <L 346>
        // worldid,                                                                               <L 347>
        geom_collision_pair_0(var_geom_type, var_geom_dataid, var_geom_size, var_mesh_vertadr, var_mesh_vertnum, var_mesh_graphadr, var_mesh_vert, var_mesh_graph, var_mesh_polynum, var_mesh_polyadr, var_mesh_polynormal, var_mesh_polyvertadr, var_mesh_polyvertnum, var_mesh_polyvert, var_mesh_polymapadr, var_mesh_polymapnum, var_mesh_polymap, var_geom_xpos_in, var_geom_xmat_in, var_6, var_22, var_48, var_49);
        // hf_pos = geom_xpos_in[worldid, g1]                                                     <L 351>
        var_50 = wp::address(var_geom_xpos_in, var_22, var_9);
        var_52 = wp::load(var_50);
        var_51 = wp::copy(var_52);
        // hf_mat = geom_xmat_in[worldid, g1]                                                     <L 352>
        var_53 = wp::address(var_geom_xmat_in, var_22, var_9);
        var_55 = wp::load(var_53);
        var_54 = wp::copy(var_55);
        // hf_matT = wp.transpose(hf_mat)                                                         <L 353>
        var_56 = wp::transpose(var_54);
        // geom2.pos = hf_matT @ (geom2.pos - hf_pos)                                             <L 355>
        var_57 = &((var_49).pos);
        var_59 = wp::load(var_57);
        var_58 = wp::sub(var_59, var_51);
        var_60 = wp::mul(var_56, var_58);
        var_49.pos = var_60;
        // geom2.rot = hf_matT @ geom2.rot                                                        <L 356>
        var_61 = &((var_49).rot);
        var_63 = wp::load(var_61);
        var_62 = wp::mul(var_56, var_63);
        var_49.rot = var_62;
        // geom1.pos = wp.vec3(0.0, 0.0, 0.0)                                                     <L 359>
        var_67 = wp::vec_t<3, wp::float32>(var_64, var_65, var_66);
        var_48.pos = var_67;
        // geom1.rot = wp.identity(n=3, dtype=float)                                              <L 360>
        var_69 = wp::identity<3, wp::float32>();
        var_48.rot = var_69;
        // geom1_dataid = geom_dataid[worldid % geom_dataid.shape[0], g1]                         <L 363>
        var_70 = &(var_geom_dataid.shape);
        var_73 = wp::load(var_70);
        var_72 = wp::extract(var_73, var_71);
        var_74 = wp::mod(var_22, var_72);
        var_75 = wp::address(var_geom_dataid, var_74, var_9);
        var_77 = wp::load(var_75);
        var_76 = wp::copy(var_77);
        // nrow = hfield_nrow[geom1_dataid]                                                       <L 366>
        var_78 = wp::address(var_hfield_nrow, var_76);
        var_80 = wp::load(var_78);
        var_79 = wp::copy(var_80);
        // ncol = hfield_ncol[geom1_dataid]                                                       <L 367>
        var_81 = wp::address(var_hfield_ncol, var_76);
        var_83 = wp::load(var_81);
        var_82 = wp::copy(var_83);
        // size = hfield_size[geom1_dataid]                                                       <L 368>
        var_84 = wp::address(var_hfield_size, var_76);
        var_86 = wp::load(var_84);
        var_85 = wp::copy(var_86);
        // x_scale = 0.5 * float(ncol - 1) / size[0]                                              <L 371>
        var_89 = wp::sub(var_82, var_88);
        var_90 = wp::float(var_89);
        var_91 = wp::mul(var_87, var_90);
        var_93 = wp::extract(var_85, var_92);
        var_94 = wp::div(var_91, var_93);
        // y_scale = 0.5 * float(nrow - 1) / size[1]                                              <L 372>
        var_97 = wp::sub(var_79, var_96);
        var_98 = wp::float(var_97);
        var_99 = wp::mul(var_95, var_98);
        var_101 = wp::extract(var_85, var_100);
        var_102 = wp::div(var_99, var_101);
        // cmin = wp.max(0, int(wp.floor((xmin + size[0]) * x_scale)))                            <L 373>
        var_105 = wp::extract(var_85, var_104);
        var_106 = wp::add(var_25, var_105);
        var_107 = wp::mul(var_106, var_94);
        var_108 = wp::floor(var_107);
        var_109 = wp::int(var_108);
        var_110 = wp::max(var_103, var_109);
        // cmax = wp.min(ncol - 1, int(wp.ceil((xmax + size[0]) * x_scale)))                      <L 374>
        var_112 = wp::sub(var_82, var_111);
        var_114 = wp::extract(var_85, var_113);
        var_115 = wp::add(var_26, var_114);
        var_116 = wp::mul(var_115, var_94);
        var_117 = wp::ceil(var_116);
        var_118 = wp::int(var_117);
        var_119 = wp::min(var_112, var_118);
        // rmin = wp.max(0, int(wp.floor((ymin + size[1]) * y_scale)))                            <L 375>
        var_122 = wp::extract(var_85, var_121);
        var_123 = wp::add(var_27, var_122);
        var_124 = wp::mul(var_123, var_102);
        var_125 = wp::floor(var_124);
        var_126 = wp::int(var_125);
        var_127 = wp::max(var_120, var_126);
        // rmax = wp.min(nrow - 1, int(wp.ceil((ymax + size[1]) * y_scale)))                      <L 376>
        var_129 = wp::sub(var_79, var_128);
        var_131 = wp::extract(var_85, var_130);
        var_132 = wp::add(var_28, var_131);
        var_133 = wp::mul(var_132, var_102);
        var_134 = wp::ceil(var_133);
        var_135 = wp::int(var_134);
        var_136 = wp::min(var_129, var_135);
        // dx = (2.0 * size[0]) / float(ncol - 1)                                                 <L 378>
        var_139 = wp::extract(var_85, var_138);
        var_140 = wp::mul(var_137, var_139);
        var_142 = wp::sub(var_82, var_141);
        var_143 = wp::float(var_142);
        var_144 = wp::div(var_140, var_143);
        // dy = (2.0 * size[1]) / float(nrow - 1)                                                 <L 379>
        var_147 = wp::extract(var_85, var_146);
        var_148 = wp::mul(var_145, var_147);
        var_150 = wp::sub(var_79, var_149);
        var_151 = wp::float(var_150);
        var_152 = wp::div(var_148, var_151);
        // dr = wp.vec2i(1, 0)                                                                    <L 380>
        var_155 = wp::vec_t<2, wp::int32>(var_153, var_154);
        // prism = mat63()                                                                        <L 382>
        var_156 = wp::mat_t<6, 3, wp::float32>();
        // prism[0, 2] = -size[3]                                                                 <L 385>
        var_158 = wp::extract(var_85, var_157);
        var_159 = wp::neg(var_158);
        wp::assign_inplace(var_156, var_160, var_161, var_159);
        // prism[1, 2] = -size[3]                                                                 <L 386>
        var_163 = wp::extract(var_85, var_162);
        var_164 = wp::neg(var_163);
        wp::assign_inplace(var_156, var_165, var_166, var_164);
        // prism[2, 2] = -size[3]                                                                 <L 387>
        var_168 = wp::extract(var_85, var_167);
        var_169 = wp::neg(var_168);
        wp::assign_inplace(var_156, var_170, var_171, var_169);
        // adr = hfield_adr[geom1_dataid]                                                         <L 389>
        var_172 = wp::address(var_hfield_adr, var_76);
        var_174 = wp::load(var_172);
        var_173 = wp::copy(var_174);
        // hfield_contact_dist = vec_maxconpair()                                                 <L 391>
        var_175 = wp::vec_t<50, wp::float32>();
        // hfield_contact_pos = mat_maxconpair()                                                  <L 392>
        var_176 = wp::mat_t<50, 3, wp::float32>();
        // hfield_contact_normal = mat_maxconpair()                                               <L 393>
        var_177 = wp::mat_t<50, 3, wp::float32>();
        // min_dist = float(MJ_MAXVAL)                                                            <L 394>
        var_179 = wp::float(var_178);
        // min_normal = wp.vec3(MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL)                                  <L 395>
        var_180 = wp::vec_t<3, wp::float32>(var_178, var_178, var_178);
        // min_pos = wp.vec3(MJ_MAXVAL, MJ_MAXVAL, MJ_MAXVAL)                                     <L 396>
        var_181 = wp::vec_t<3, wp::float32>(var_178, var_178, var_178);
        // min_id = int(-1)                                                                       <L 397>
        var_183 = wp::int(var_182);
        // geom2.margin = margin                                                                  <L 400>
        var_49.margin = var_40;
        // epa_vert = epa_vert_in[ccdid]                                                          <L 403>
        var_184 = wp::slice_t(var_33, var_33, var_185);
        var_186 = wp::view(var_epa_vert_in, var_184);
        // epa_vert_index = epa_vert_index_in[ccdid]                                              <L 404>
        var_187 = wp::slice_t(var_33, var_33, var_188);
        var_189 = wp::view(var_epa_vert_index_in, var_187);
        // epa_face = epa_face_in[ccdid]                                                          <L 405>
        var_190 = wp::slice_t(var_33, var_33, var_191);
        var_192 = wp::view(var_epa_face_in, var_190);
        // epa_pr = epa_pr_in[ccdid]                                                              <L 406>
        var_193 = wp::slice_t(var_33, var_33, var_194);
        var_195 = wp::view(var_epa_pr_in, var_193);
        // epa_norm2 = epa_norm2_in[ccdid]                                                        <L 407>
        var_196 = wp::slice_t(var_33, var_33, var_197);
        var_198 = wp::view(var_epa_norm2_in, var_196);
        // epa_horizon = epa_horizon_in[ccdid]                                                    <L 408>
        var_199 = wp::slice_t(var_33, var_33, var_200);
        var_201 = wp::view(var_epa_horizon_in, var_199);
        // collision_pairid = collision_pairid_in[collisionid]                                    <L 410>
        var_202 = wp::address(var_collision_pairid_in, var_0);
        var_204 = wp::load(var_202);
        var_203 = wp::copy(var_204);
        // count = int(0)                                                                         <L 413>
        var_206 = wp::int(var_205);
        // noverflow = int(0)                                                                     <L 414>
        var_208 = wp::int(var_207);
        // for r in range(rmin, rmax):                                                            <L 415>
        var_209 = wp::range(var_127, var_136);
        start_for_4:;
            if (iter_cmp(var_209) == 0) goto end_for_4;
            var_210 = wp::iter_next(var_209);
            // if noverflow > 0:                                                                  <L 416>
            var_212 = (var_208 > var_211);
            if (var_212) {
                // break                                                                          <L 417>
                goto end_for_4;
            }
            // for init_i in range(2):                                                            <L 420>
            // x = dx * float(cmin) - size[0]                                                     <L 421>
            var_214 = wp::float(var_110);
            var_215 = wp::mul(var_144, var_214);
            var_217 = wp::extract(var_85, var_216);
            var_218 = wp::sub(var_215, var_217);
            // y = dy * float(r + dr[init_i]) - size[1]                                           <L 422>
            var_219 = wp::extract(var_155, var_213);
            var_220 = wp::add(var_210, var_219);
            var_221 = wp::float(var_220);
            var_222 = wp::mul(var_152, var_221);
            var_224 = wp::extract(var_85, var_223);
            var_225 = wp::sub(var_222, var_224);
            // z = hfield_data[adr + (r + dr[init_i]) * ncol + cmin] * size[2] + margin           <L 423>
            var_226 = wp::extract(var_155, var_213);
            var_227 = wp::add(var_210, var_226);
            var_228 = wp::mul(var_227, var_82);
            var_229 = wp::add(var_173, var_228);
            var_230 = wp::add(var_229, var_110);
            var_231 = wp::address(var_hfield_data, var_230);
            var_233 = wp::extract(var_85, var_232);
            var_235 = wp::load(var_231);
            var_234 = wp::mul(var_235, var_233);
            var_236 = wp::add(var_234, var_40);
            // prism[0] = prism[1]                                                                <L 425>
            var_238 = wp::extract(var_156, var_237);
            wp::assign_inplace(var_156, var_239, var_238);
            // prism[1] = prism[2]                                                                <L 426>
            var_241 = wp::extract(var_156, var_240);
            wp::assign_inplace(var_156, var_242, var_241);
            // prism[3] = prism[4]                                                                <L 427>
            var_244 = wp::extract(var_156, var_243);
            wp::assign_inplace(var_156, var_245, var_244);
            // prism[4] = prism[5]                                                                <L 428>
            var_247 = wp::extract(var_156, var_246);
            wp::assign_inplace(var_156, var_248, var_247);
            // prism[2, 0] = x                                                                    <L 430>
            wp::assign_inplace(var_156, var_249, var_250, var_218);
            // prism[5, 0] = x                                                                    <L 431>
            wp::assign_inplace(var_156, var_251, var_252, var_218);
            // prism[2, 1] = y                                                                    <L 432>
            wp::assign_inplace(var_156, var_253, var_254, var_225);
            // prism[5, 1] = y                                                                    <L 433>
            wp::assign_inplace(var_156, var_255, var_256, var_225);
            // prism[5, 2] = z                                                                    <L 434>
            wp::assign_inplace(var_156, var_257, var_258, var_236);
            // x = dx * float(cmin) - size[0]                                                     <L 421>
            var_260 = wp::float(var_110);
            var_261 = wp::mul(var_144, var_260);
            var_263 = wp::extract(var_85, var_262);
            var_264 = wp::sub(var_261, var_263);
            // y = dy * float(r + dr[init_i]) - size[1]                                           <L 422>
            var_265 = wp::extract(var_155, var_259);
            var_266 = wp::add(var_210, var_265);
            var_267 = wp::float(var_266);
            var_268 = wp::mul(var_152, var_267);
            var_270 = wp::extract(var_85, var_269);
            var_271 = wp::sub(var_268, var_270);
            // z = hfield_data[adr + (r + dr[init_i]) * ncol + cmin] * size[2] + margin           <L 423>
            var_272 = wp::extract(var_155, var_259);
            var_273 = wp::add(var_210, var_272);
            var_274 = wp::mul(var_273, var_82);
            var_275 = wp::add(var_173, var_274);
            var_276 = wp::add(var_275, var_110);
            var_277 = wp::address(var_hfield_data, var_276);
            var_279 = wp::extract(var_85, var_278);
            var_281 = wp::load(var_277);
            var_280 = wp::mul(var_281, var_279);
            var_282 = wp::add(var_280, var_40);
            // prism[0] = prism[1]                                                                <L 425>
            var_284 = wp::extract(var_156, var_283);
            wp::assign_inplace(var_156, var_285, var_284);
            // prism[1] = prism[2]                                                                <L 426>
            var_287 = wp::extract(var_156, var_286);
            wp::assign_inplace(var_156, var_288, var_287);
            // prism[3] = prism[4]                                                                <L 427>
            var_290 = wp::extract(var_156, var_289);
            wp::assign_inplace(var_156, var_291, var_290);
            // prism[4] = prism[5]                                                                <L 428>
            var_293 = wp::extract(var_156, var_292);
            wp::assign_inplace(var_156, var_294, var_293);
            // prism[2, 0] = x                                                                    <L 430>
            wp::assign_inplace(var_156, var_295, var_296, var_264);
            // prism[5, 0] = x                                                                    <L 431>
            wp::assign_inplace(var_156, var_297, var_298, var_264);
            // prism[2, 1] = y                                                                    <L 432>
            wp::assign_inplace(var_156, var_299, var_300, var_271);
            // prism[5, 1] = y                                                                    <L 433>
            wp::assign_inplace(var_156, var_301, var_302, var_271);
            // prism[5, 2] = z                                                                    <L 434>
            wp::assign_inplace(var_156, var_303, var_304, var_282);
            // for c in range(cmin + 1, cmax + 1):                                                <L 436>
            var_306 = wp::add(var_110, var_305);
            var_308 = wp::add(var_119, var_307);
            var_309 = wp::range(var_306, var_308);
            start_for_6:;
                if (iter_cmp(var_309) == 0) goto end_for_6;
                var_310 = wp::iter_next(var_309);
                // if noverflow > 0:                                                              <L 437>
                var_312 = (var_208 > var_311);
                if (var_312) {
                    // break                                                                      <L 438>
                    goto end_for_6;
                }
                // for i in range(2):                                                             <L 441>
                var_314 = wp::range(var_313);
                start_for_8:;
                    if (iter_cmp(var_314) == 0) goto end_for_8;
                    var_315 = wp::iter_next(var_314);
                    // if count >= MJ_MAXCONPAIR:                                                 <L 442>
                    var_317 = (var_206 >= var_316);
                    if (var_317) {
                        // noverflow = 1                                                          <L 443>
                        // break                                                                  <L 444>
                        wp::assign(var_208, var_318);
                        goto end_for_8;
                    }
                    // x = dx * float(c) - size[0]                                                <L 447>
                    var_319 = wp::float(var_310);
                    var_320 = wp::mul(var_144, var_319);
                    var_322 = wp::extract(var_85, var_321);
                    var_323 = wp::sub(var_320, var_322);
                    // y = dy * float(r + dr[i]) - size[1]                                        <L 448>
                    var_324 = wp::extract(var_155, var_315);
                    var_325 = wp::add(var_210, var_324);
                    var_326 = wp::float(var_325);
                    var_327 = wp::mul(var_152, var_326);
                    var_329 = wp::extract(var_85, var_328);
                    var_330 = wp::sub(var_327, var_329);
                    // z = hfield_data[adr + (r + dr[i]) * ncol + c] * size[2] + margin           <L 449>
                    var_331 = wp::extract(var_155, var_315);
                    var_332 = wp::add(var_210, var_331);
                    var_333 = wp::mul(var_332, var_82);
                    var_334 = wp::add(var_173, var_333);
                    var_335 = wp::add(var_334, var_310);
                    var_336 = wp::address(var_hfield_data, var_335);
                    var_338 = wp::extract(var_85, var_337);
                    var_340 = wp::load(var_336);
                    var_339 = wp::mul(var_340, var_338);
                    var_341 = wp::add(var_339, var_40);
                    // prism[0] = prism[1]                                                        <L 451>
                    var_343 = wp::extract(var_156, var_342);
                    wp::assign_inplace(var_156, var_344, var_343);
                    // prism[1] = prism[2]                                                        <L 452>
                    var_346 = wp::extract(var_156, var_345);
                    wp::assign_inplace(var_156, var_347, var_346);
                    // prism[3] = prism[4]                                                        <L 453>
                    var_349 = wp::extract(var_156, var_348);
                    wp::assign_inplace(var_156, var_350, var_349);
                    // prism[4] = prism[5]                                                        <L 454>
                    var_352 = wp::extract(var_156, var_351);
                    wp::assign_inplace(var_156, var_353, var_352);
                    // prism[2, 0] = x                                                            <L 456>
                    wp::assign_inplace(var_156, var_354, var_355, var_323);
                    // prism[5, 0] = x                                                            <L 457>
                    wp::assign_inplace(var_156, var_356, var_357, var_323);
                    // prism[2, 1] = y                                                            <L 458>
                    wp::assign_inplace(var_156, var_358, var_359, var_330);
                    // prism[5, 1] = y                                                            <L 459>
                    wp::assign_inplace(var_156, var_360, var_361, var_330);
                    // prism[5, 2] = z                                                            <L 460>
                    wp::assign_inplace(var_156, var_362, var_363, var_341);
                    // if prism[3, 2] < zmin and prism[4, 2] < zmin and prism[5, 2] < zmin:       <L 463>
                    var_367 = wp::extract(var_156, var_365, var_366);
                    var_368 = (var_367 < var_29);
                    var_364 = var_368;
                    if (var_364) {
                        var_371 = wp::extract(var_156, var_369, var_370);
                        var_372 = (var_371 < var_29);
                        var_364 = var_364 && var_372;
                    }
                    if (var_364) {
                        var_375 = wp::extract(var_156, var_373, var_374);
                        var_376 = (var_375 < var_29);
                        var_364 = var_364 && var_376;
                    }
                    if (var_364) {
                        // continue                                                               <L 464>
                        wp::assign(var_264, var_323);
                        wp::assign(var_271, var_330);
                        wp::assign(var_282, var_341);
                        goto start_for_8;
                    }
                    var_377 = wp::where(var_364, var_264, var_323);
                    var_378 = wp::where(var_364, var_271, var_330);
                    var_379 = wp::where(var_364, var_282, var_341);
                    // geom1.polyvert = prism                                                     <L 466>
                    var_48.polyvert = var_156;
                    // x1 = geom1.pos + geom1.rot @ (prism[0] + prism[1] + prism[2] + prism[3] + prism[4] + prism[5]) * wp.static(1.0 / 6.0)       <L 469>
                    var_380 = &((var_48).pos);
                    var_381 = &((var_48).rot);
                    var_383 = wp::extract(var_156, var_382);
                    var_385 = wp::extract(var_156, var_384);
                    var_386 = wp::add(var_383, var_385);
                    var_388 = wp::extract(var_156, var_387);
                    var_389 = wp::add(var_386, var_388);
                    var_391 = wp::extract(var_156, var_390);
                    var_392 = wp::add(var_389, var_391);
                    var_394 = wp::extract(var_156, var_393);
                    var_395 = wp::add(var_392, var_394);
                    var_397 = wp::extract(var_156, var_396);
                    var_398 = wp::add(var_395, var_397);
                    var_400 = wp::load(var_381);
                    var_399 = wp::mul(var_400, var_398);
                    var_402 = wp::mul(var_399, var_401);
                    var_404 = wp::load(var_380);
                    var_403 = wp::add(var_404, var_402);
                    // dist, ncontact, w1, w2, idx = ccd(                                         <L 471>
                    // opt_ccd_tolerance[worldid % opt_ccd_tolerance.shape[0]],                   <L 472>
                    var_405 = &(var_opt_ccd_tolerance.shape);
                    var_408 = wp::load(var_405);
                    var_407 = wp::extract(var_408, var_406);
                    var_409 = wp::mod(var_22, var_407);
                    var_410 = wp::address(var_opt_ccd_tolerance, var_409);
                    // 0.0,                                                                       <L 473>
                    // gjk_iterations,                                                            <L 474>
                    // epa_iterations,                                                            <L 475>
                    // geom1,                                                                     <L 476>
                    // geom2,                                                                     <L 477>
                    // geomtype1,                                                                 <L 478>
                    // geomtype2,                                                                 <L 479>
                    // x1,                                                                        <L 480>
                    // geom2.pos,                                                                 <L 481>
                    var_414 = &((var_49).pos);
                    // epa_vert,                                                                  <L 482>
                    // epa_vert_index,                                                            <L 483>
                    // epa_face,                                                                  <L 484>
                    // epa_pr,                                                                    <L 485>
                    // epa_norm2,                                                                 <L 486>
                    // epa_horizon,                                                               <L 487>
                    // wp.static(bool(warn_overflow & OverflowType.EPA_HORIZON)),                 <L 488>
                    // worldid,                                                                   <L 489>
                    // overflow_out,                                                              <L 490>
                    var_421 = wp::load(var_410);
                    var_422 = wp::load(var_414);
                    ccd_0(var_421, var_411, var_412, var_413, var_48, var_49, var_14, var_18, var_403, var_422, var_186, var_189, var_192, var_195, var_198, var_201, var_415, var_22, var_overflow_out, var_416, var_417, var_418, var_419, var_420);
                    // if ncontact == 0 or dist >= margin + gap:                                  <L 494>
                    var_425 = (var_417 == var_424);
                    var_423 = var_425;
                    if (!var_423) {
                        var_426 = wp::add(var_40, var_41);
                        var_427 = (var_416 >= var_426);
                        var_423 = var_423 || var_427;
                    }
                    if (var_423) {
                        // continue                                                               <L 495>
                        wp::assign(var_264, var_377);
                        wp::assign(var_271, var_378);
                        wp::assign(var_282, var_379);
                        goto start_for_8;
                    }
                    var_428 = wp::where(var_423, var_264, var_377);
                    var_429 = wp::where(var_423, var_271, var_378);
                    var_430 = wp::where(var_423, var_282, var_379);
                    // hfield_contact_dist[count] = dist                                          <L 498>
                    wp::assign_inplace(var_175, var_206, var_416);
                    // pos_local = 0.5 * (w1 + w2)                                                <L 501>
                    var_432 = wp::add(var_418, var_419);
                    var_433 = wp::mul(var_431, var_432);
                    // pos = hf_mat @ pos_local + hf_pos                                          <L 502>
                    var_434 = wp::mul(var_54, var_433);
                    var_435 = wp::add(var_434, var_51);
                    // hfield_contact_pos[count, 0] = pos[0]                                      <L 503>
                    var_437 = wp::extract(var_435, var_436);
                    wp::assign_inplace(var_176, var_206, var_438, var_437);
                    // hfield_contact_pos[count, 1] = pos[1]                                      <L 504>
                    var_440 = wp::extract(var_435, var_439);
                    wp::assign_inplace(var_176, var_206, var_441, var_440);
                    // hfield_contact_pos[count, 2] = pos[2]                                      <L 505>
                    var_443 = wp::extract(var_435, var_442);
                    wp::assign_inplace(var_176, var_206, var_444, var_443);
                    // frame_local = make_frame(w1 - w2)                                          <L 507>
                    var_445 = wp::sub(var_418, var_419);
                    var_446 = make_frame_0(var_445);
                    // normal_local = wp.vec3(frame_local[0, 0], frame_local[0, 1], frame_local[0, 2])       <L 508>
                    var_449 = wp::extract(var_446, var_447, var_448);
                    var_452 = wp::extract(var_446, var_450, var_451);
                    var_455 = wp::extract(var_446, var_453, var_454);
                    var_456 = wp::vec_t<3, wp::float32>(var_449, var_452, var_455);
                    // normal = hf_mat @ normal_local                                             <L 509>
                    var_457 = wp::mul(var_54, var_456);
                    // hfield_contact_normal[count, 0] = normal[0]                                <L 510>
                    var_459 = wp::extract(var_457, var_458);
                    wp::assign_inplace(var_177, var_206, var_460, var_459);
                    // hfield_contact_normal[count, 1] = normal[1]                                <L 511>
                    var_462 = wp::extract(var_457, var_461);
                    wp::assign_inplace(var_177, var_206, var_463, var_462);
                    // hfield_contact_normal[count, 2] = normal[2]                                <L 512>
                    var_465 = wp::extract(var_457, var_464);
                    wp::assign_inplace(var_177, var_206, var_466, var_465);
                    // if dist < min_dist:                                                        <L 515>
                    var_467 = (var_416 < var_179);
                    if (var_467) {
                        // min_dist = dist                                                        <L 516>
                        var_468 = wp::copy(var_416);
                        // min_normal = normal                                                    <L 517>
                        var_469 = wp::copy(var_457);
                        // min_pos = pos                                                          <L 518>
                        var_470 = wp::copy(var_435);
                        // min_id = count                                                         <L 519>
                        var_471 = wp::copy(var_206);
                    }
                    var_472 = wp::where(var_467, var_468, var_179);
                    var_473 = wp::where(var_467, var_469, var_180);
                    var_474 = wp::where(var_467, var_470, var_181);
                    var_475 = wp::where(var_467, var_471, var_183);
                    // count += 1                                                                 <L 521>
                    var_477 = wp::add(var_206, var_476);
                    wp::assign(var_179, var_472);
                    wp::assign(var_180, var_473);
                    wp::assign(var_181, var_474);
                    wp::assign(var_183, var_475);
                    wp::assign(var_206, var_477);
                    wp::assign(var_264, var_428);
                    wp::assign(var_271, var_429);
                    wp::assign(var_282, var_430);
                    goto start_for_8;
                end_for_8:;
                goto start_for_6;
            end_for_6:;
            goto start_for_4;
        end_for_4:;
        // if noverflow > 0:                                                                      <L 524>
        var_479 = (var_208 > var_478);
        if (var_479) {
            // if wp.static(bool(warn_overflow & OverflowType.HFIELD)):                           <L 525>
            // wp.printf(                                                                         <L 526>
            // "height field collision overflow, number of collisions >= %u - please adjust resolution: \n"       <L 527>
            // MJ_MAXCONPAIR,                                                                     <L 530>
            printf(var_481, var_316);
            // wp.atomic_or(overflow_out, worldid, OverflowType.HFIELD)                           <L 532>
            var_484 = wp::atomic_or(var_overflow_out, var_22, var_483);
        }
        // if count == 0:                                                                         <L 535>
        var_486 = (var_206 == var_485);
        if (var_486) {
            // return                                                                             <L 536>
            return;
        }
        // write_contact(                                                                         <L 539>
        // naconmax_in,                                                                           <L 540>
        // 0,                                                                                     <L 541>
        // min_dist,                                                                              <L 542>
        // min_pos,                                                                               <L 543>
        // make_frame(min_normal),                                                                <L 544>
        var_488 = make_frame_0(var_180);
        // margin,                                                                                <L 545>
        // gap,                                                                                   <L 546>
        // condim,                                                                                <L 547>
        // friction,                                                                              <L 548>
        // solref,                                                                                <L 549>
        // solreffriction,                                                                        <L 550>
        // solimp,                                                                                <L 551>
        // adhesion,                                                                              <L 552>
        // geoms,                                                                                 <L 553>
        // collision_pairid,                                                                      <L 554>
        // worldid,                                                                               <L 555>
        // contact_dist_out,                                                                      <L 556>
        // contact_pos_out,                                                                       <L 557>
        // contact_frame_out,                                                                     <L 558>
        // contact_includemargin_out,                                                             <L 559>
        // contact_friction_out,                                                                  <L 560>
        // contact_solref_out,                                                                    <L 561>
        // contact_solreffriction_out,                                                            <L 562>
        // contact_solimp_out,                                                                    <L 563>
        // contact_dim_out,                                                                       <L 564>
        // contact_geom_out,                                                                      <L 565>
        // contact_efc_address_out,                                                               <L 566>
        // contact_worldid_out,                                                                   <L 567>
        // contact_type_out,                                                                      <L 568>
        // contact_geomcollisionid_out,                                                           <L 569>
        // contact_adhesion_out,                                                                  <L 570>
        // nacon_out,                                                                             <L 571>
        var_489 = write_contact_0(var_naconmax_in, var_487, var_179, var_181, var_488, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_6, var_203, var_22, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        // if wp.static(True):                                                                    <L 575>
        // MIN_DIST_TO_NEXT_CONTACT = 1.0e-3                                                      <L 576>
        // id1 = int(-1)                                                                          <L 579>
        var_493 = wp::int(var_492);
        // dist1 = float(-MJ_MAXVAL)                                                              <L 580>
        var_495 = wp::float(var_494);
        // for i in range(count):                                                                 <L 581>
        var_496 = wp::range(var_206);
        start_for_11:;
            if (iter_cmp(var_496) == 0) goto end_for_11;
            var_497 = wp::iter_next(var_496);
            // if i == min_id:                                                                    <L 582>
            var_498 = (var_497 == var_183);
            if (var_498) {
                // continue                                                                       <L 583>
                goto start_for_11;
            }
            // hf_pos = wp.vec3(hfield_contact_pos[i, 0], hfield_contact_pos[i, 1], hfield_contact_pos[i, 2])       <L 585>
            var_500 = wp::extract(var_176, var_497, var_499);
            var_502 = wp::extract(var_176, var_497, var_501);
            var_504 = wp::extract(var_176, var_497, var_503);
            var_505 = wp::vec_t<3, wp::float32>(var_500, var_502, var_504);
            // dist = wp.norm_l2(hf_pos - min_pos)                                                <L 586>
            var_506 = wp::sub(var_505, var_181);
            var_507 = norm_l2_0(var_506);
            // if dist > dist1:                                                                   <L 588>
            var_508 = (var_507 > var_495);
            if (var_508) {
                // id1 = i                                                                        <L 589>
                var_509 = wp::copy(var_497);
                // dist1 = dist                                                                   <L 590>
                var_510 = wp::copy(var_507);
            }
            var_511 = wp::where(var_508, var_509, var_493);
            var_512 = wp::where(var_508, var_510, var_495);
            wp::assign(var_51, var_505);
            wp::assign(var_416, var_507);
            wp::assign(var_493, var_511);
            wp::assign(var_495, var_512);
            goto start_for_11;
        end_for_11:;
        // if id1 == -1 or (0.0 < dist1 and dist1 < MIN_DIST_TO_NEXT_CONTACT):                    <L 592>
        var_515 = (var_493 == var_514);
        var_513 = var_515;
        if (!var_513) {
            var_518 = (var_517 < var_495);
            var_516 = var_518;
            if (var_516) {
                var_519 = (var_495 < var_491);
                var_516 = var_516 && var_519;
            }
            var_513 = var_513 || var_516;
        }
        if (var_513) {
            // return                                                                             <L 593>
            return;
        }
        // pos1 = wp.vec3(hfield_contact_pos[id1, 0], hfield_contact_pos[id1, 1], hfield_contact_pos[id1, 2])       <L 595>
        var_521 = wp::extract(var_176, var_493, var_520);
        var_523 = wp::extract(var_176, var_493, var_522);
        var_525 = wp::extract(var_176, var_493, var_524);
        var_526 = wp::vec_t<3, wp::float32>(var_521, var_523, var_525);
        // normal1 = wp.vec3(hfield_contact_normal[id1, 0], hfield_contact_normal[id1, 1], hfield_contact_normal[id1, 2])       <L 596>
        var_528 = wp::extract(var_177, var_493, var_527);
        var_530 = wp::extract(var_177, var_493, var_529);
        var_532 = wp::extract(var_177, var_493, var_531);
        var_533 = wp::vec_t<3, wp::float32>(var_528, var_530, var_532);
        // write_contact(                                                                         <L 598>
        // naconmax_in,                                                                           <L 599>
        // 1,                                                                                     <L 600>
        // hfield_contact_dist[id1],                                                              <L 601>
        var_535 = wp::extract(var_175, var_493);
        // pos1,                                                                                  <L 602>
        // make_frame(normal1),                                                                   <L 603>
        var_536 = make_frame_0(var_533);
        // margin,                                                                                <L 604>
        // gap,                                                                                   <L 605>
        // condim,                                                                                <L 606>
        // friction,                                                                              <L 607>
        // solref,                                                                                <L 608>
        // solreffriction,                                                                        <L 609>
        // solimp,                                                                                <L 610>
        // adhesion,                                                                              <L 611>
        // geoms,                                                                                 <L 612>
        // collision_pairid,                                                                      <L 613>
        // worldid,                                                                               <L 614>
        // contact_dist_out,                                                                      <L 615>
        // contact_pos_out,                                                                       <L 616>
        // contact_frame_out,                                                                     <L 617>
        // contact_includemargin_out,                                                             <L 618>
        // contact_friction_out,                                                                  <L 619>
        // contact_solref_out,                                                                    <L 620>
        // contact_solreffriction_out,                                                            <L 621>
        // contact_solimp_out,                                                                    <L 622>
        // contact_dim_out,                                                                       <L 623>
        // contact_geom_out,                                                                      <L 624>
        // contact_efc_address_out,                                                               <L 625>
        // contact_worldid_out,                                                                   <L 626>
        // contact_type_out,                                                                      <L 627>
        // contact_geomcollisionid_out,                                                           <L 628>
        // contact_adhesion_out,                                                                  <L 629>
        // nacon_out,                                                                             <L 630>
        var_537 = write_contact_0(var_naconmax_in, var_534, var_535, var_526, var_536, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_6, var_203, var_22, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        // dist_min1 = wp.cross(min_normal, min_pos - pos1)                                       <L 634>
        var_538 = wp::sub(var_181, var_526);
        var_539 = wp::cross(var_180, var_538);
        // id2 = int(-1)                                                                          <L 636>
        var_541 = wp::int(var_540);
        // dist_12 = float(-MJ_MAXVAL)                                                            <L 637>
        var_543 = wp::float(var_542);
        // for i in range(count):                                                                 <L 638>
        var_544 = wp::range(var_206);
        start_for_14:;
            if (iter_cmp(var_544) == 0) goto end_for_14;
            var_545 = wp::iter_next(var_544);
            // if i == min_id or i == id1:                                                        <L 639>
            var_547 = (var_545 == var_183);
            var_546 = var_547;
            if (!var_546) {
                var_548 = (var_545 == var_493);
                var_546 = var_546 || var_548;
            }
            if (var_546) {
                // continue                                                                       <L 640>
                goto start_for_14;
            }
            // hf_pos = wp.vec3(hfield_contact_pos[i, 0], hfield_contact_pos[i, 1], hfield_contact_pos[i, 2])       <L 642>
            var_550 = wp::extract(var_176, var_545, var_549);
            var_552 = wp::extract(var_176, var_545, var_551);
            var_554 = wp::extract(var_176, var_545, var_553);
            var_555 = wp::vec_t<3, wp::float32>(var_550, var_552, var_554);
            // dist = wp.abs(wp.dot(hf_pos - min_pos, dist_min1))                                 <L 643>
            var_556 = wp::sub(var_555, var_181);
            var_557 = wp::dot(var_556, var_539);
            var_558 = wp::abs(var_557);
            // if dist > dist_12:                                                                 <L 645>
            var_559 = (var_558 > var_543);
            if (var_559) {
                // id2 = i                                                                        <L 646>
                var_560 = wp::copy(var_545);
                // dist_12 = dist                                                                 <L 647>
                var_561 = wp::copy(var_558);
            }
            var_562 = wp::where(var_559, var_560, var_541);
            var_563 = wp::where(var_559, var_561, var_543);
            wp::assign(var_51, var_555);
            wp::assign(var_416, var_558);
            wp::assign(var_541, var_562);
            wp::assign(var_543, var_563);
            goto start_for_14;
        end_for_14:;
        // if id2 == -1 or (0.0 < dist_12 and dist_12 < MIN_DIST_TO_NEXT_CONTACT):                <L 649>
        var_566 = (var_541 == var_565);
        var_564 = var_566;
        if (!var_564) {
            var_569 = (var_568 < var_543);
            var_567 = var_569;
            if (var_567) {
                var_570 = (var_543 < var_491);
                var_567 = var_567 && var_570;
            }
            var_564 = var_564 || var_567;
        }
        if (var_564) {
            // return                                                                             <L 650>
            return;
        }
        // pos2 = wp.vec3(hfield_contact_pos[id2, 0], hfield_contact_pos[id2, 1], hfield_contact_pos[id2, 2])       <L 652>
        var_572 = wp::extract(var_176, var_541, var_571);
        var_574 = wp::extract(var_176, var_541, var_573);
        var_576 = wp::extract(var_176, var_541, var_575);
        var_577 = wp::vec_t<3, wp::float32>(var_572, var_574, var_576);
        // normal2 = wp.vec3(hfield_contact_normal[id2, 0], hfield_contact_normal[id2, 1], hfield_contact_normal[id2, 2])       <L 653>
        var_579 = wp::extract(var_177, var_541, var_578);
        var_581 = wp::extract(var_177, var_541, var_580);
        var_583 = wp::extract(var_177, var_541, var_582);
        var_584 = wp::vec_t<3, wp::float32>(var_579, var_581, var_583);
        // write_contact(                                                                         <L 655>
        // naconmax_in,                                                                           <L 656>
        // 2,                                                                                     <L 657>
        // hfield_contact_dist[id2],                                                              <L 658>
        var_586 = wp::extract(var_175, var_541);
        // pos2,                                                                                  <L 659>
        // make_frame(normal2),                                                                   <L 660>
        var_587 = make_frame_0(var_584);
        // margin,                                                                                <L 661>
        // gap,                                                                                   <L 662>
        // condim,                                                                                <L 663>
        // friction,                                                                              <L 664>
        // solref,                                                                                <L 665>
        // solreffriction,                                                                        <L 666>
        // solimp,                                                                                <L 667>
        // adhesion,                                                                              <L 668>
        // geoms,                                                                                 <L 669>
        // collision_pairid,                                                                      <L 670>
        // worldid,                                                                               <L 671>
        // contact_dist_out,                                                                      <L 672>
        // contact_pos_out,                                                                       <L 673>
        // contact_frame_out,                                                                     <L 674>
        // contact_includemargin_out,                                                             <L 675>
        // contact_friction_out,                                                                  <L 676>
        // contact_solref_out,                                                                    <L 677>
        // contact_solreffriction_out,                                                            <L 678>
        // contact_solimp_out,                                                                    <L 679>
        // contact_dim_out,                                                                       <L 680>
        // contact_geom_out,                                                                      <L 681>
        // contact_efc_address_out,                                                               <L 682>
        // contact_worldid_out,                                                                   <L 683>
        // contact_type_out,                                                                      <L 684>
        // contact_geomcollisionid_out,                                                           <L 685>
        // contact_adhesion_out,                                                                  <L 686>
        // nacon_out,                                                                             <L 687>
        var_588 = write_contact_0(var_naconmax_in, var_585, var_586, var_577, var_587, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_6, var_203, var_22, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
        // vec_min2 = wp.cross(min_normal, min_pos - pos2)                                        <L 691>
        var_589 = wp::sub(var_181, var_577);
        var_590 = wp::cross(var_180, var_589);
        // vec_12 = wp.cross(min_normal, pos1 - pos2)                                             <L 692>
        var_591 = wp::sub(var_526, var_577);
        var_592 = wp::cross(var_180, var_591);
        // id3 = int(-1)                                                                          <L 694>
        var_594 = wp::int(var_593);
        // dist3 = float(-MJ_MAXVAL)                                                              <L 695>
        var_596 = wp::float(var_595);
        // for i in range(count):                                                                 <L 696>
        var_597 = wp::range(var_206);
        start_for_17:;
            if (iter_cmp(var_597) == 0) goto end_for_17;
            var_598 = wp::iter_next(var_597);
            // if i == min_id or i == id1 or i == id2:                                            <L 697>
            var_600 = (var_598 == var_183);
            var_599 = var_600;
            if (!var_599) {
                var_601 = (var_598 == var_493);
                var_599 = var_599 || var_601;
            }
            if (!var_599) {
                var_602 = (var_598 == var_541);
                var_599 = var_599 || var_602;
            }
            if (var_599) {
                // continue                                                                       <L 698>
                goto start_for_17;
            }
            // hf_pos = wp.vec3(hfield_contact_pos[i, 0], hfield_contact_pos[i, 1], hfield_contact_pos[i, 2])       <L 700>
            var_604 = wp::extract(var_176, var_598, var_603);
            var_606 = wp::extract(var_176, var_598, var_605);
            var_608 = wp::extract(var_176, var_598, var_607);
            var_609 = wp::vec_t<3, wp::float32>(var_604, var_606, var_608);
            // dist = wp.abs(wp.dot(hf_pos - min_pos, vec_min2)) + wp.abs(wp.dot(pos1 - hf_pos, vec_12))       <L 701>
            var_610 = wp::sub(var_609, var_181);
            var_611 = wp::dot(var_610, var_590);
            var_612 = wp::abs(var_611);
            var_613 = wp::sub(var_526, var_609);
            var_614 = wp::dot(var_613, var_592);
            var_615 = wp::abs(var_614);
            var_616 = wp::add(var_612, var_615);
            // if dist > dist3:                                                                   <L 703>
            var_617 = (var_616 > var_596);
            if (var_617) {
                // id3 = i                                                                        <L 704>
                var_618 = wp::copy(var_598);
                // dist3 = dist                                                                   <L 705>
                var_619 = wp::copy(var_616);
            }
            var_620 = wp::where(var_617, var_618, var_594);
            var_621 = wp::where(var_617, var_619, var_596);
            wp::assign(var_51, var_609);
            wp::assign(var_416, var_616);
            wp::assign(var_594, var_620);
            wp::assign(var_596, var_621);
            goto start_for_17;
        end_for_17:;
        // if id3 == -1 or (0.0 < dist3 and dist3 < MIN_DIST_TO_NEXT_CONTACT):                    <L 707>
        var_624 = (var_594 == var_623);
        var_622 = var_624;
        if (!var_622) {
            var_627 = (var_626 < var_596);
            var_625 = var_627;
            if (var_625) {
                var_628 = (var_596 < var_491);
                var_625 = var_625 && var_628;
            }
            var_622 = var_622 || var_625;
        }
        if (var_622) {
            // return                                                                             <L 708>
            return;
        }
        // pos3 = wp.vec3(hfield_contact_pos[id3, 0], hfield_contact_pos[id3, 1], hfield_contact_pos[id3, 2])       <L 710>
        var_630 = wp::extract(var_176, var_594, var_629);
        var_632 = wp::extract(var_176, var_594, var_631);
        var_634 = wp::extract(var_176, var_594, var_633);
        var_635 = wp::vec_t<3, wp::float32>(var_630, var_632, var_634);
        // normal3 = wp.vec3(hfield_contact_normal[id3, 0], hfield_contact_normal[id3, 1], hfield_contact_normal[id3, 2])       <L 711>
        var_637 = wp::extract(var_177, var_594, var_636);
        var_639 = wp::extract(var_177, var_594, var_638);
        var_641 = wp::extract(var_177, var_594, var_640);
        var_642 = wp::vec_t<3, wp::float32>(var_637, var_639, var_641);
        // write_contact(                                                                         <L 713>
        // naconmax_in,                                                                           <L 714>
        // 3,                                                                                     <L 715>
        // hfield_contact_dist[id3],                                                              <L 716>
        var_644 = wp::extract(var_175, var_594);
        // pos3,                                                                                  <L 717>
        // make_frame(normal3),                                                                   <L 718>
        var_645 = make_frame_0(var_642);
        // margin,                                                                                <L 719>
        // gap,                                                                                   <L 720>
        // condim,                                                                                <L 721>
        // friction,                                                                              <L 722>
        // solref,                                                                                <L 723>
        // solreffriction,                                                                        <L 724>
        // solimp,                                                                                <L 725>
        // adhesion,                                                                              <L 726>
        // geoms,                                                                                 <L 727>
        // collision_pairid,                                                                      <L 728>
        // worldid,                                                                               <L 729>
        // contact_dist_out,                                                                      <L 730>
        // contact_pos_out,                                                                       <L 731>
        // contact_frame_out,                                                                     <L 732>
        // contact_includemargin_out,                                                             <L 733>
        // contact_friction_out,                                                                  <L 734>
        // contact_solref_out,                                                                    <L 735>
        // contact_solreffriction_out,                                                            <L 736>
        // contact_solimp_out,                                                                    <L 737>
        // contact_dim_out,                                                                       <L 738>
        // contact_geom_out,                                                                      <L 739>
        // contact_efc_address_out,                                                               <L 740>
        // contact_worldid_out,                                                                   <L 741>
        // contact_type_out,                                                                      <L 742>
        // contact_geomcollisionid_out,                                                           <L 743>
        // contact_adhesion_out,                                                                  <L 744>
        // nacon_out,                                                                             <L 745>
        var_646 = write_contact_0(var_naconmax_in, var_643, var_644, var_635, var_645, var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_6, var_203, var_22, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
}

