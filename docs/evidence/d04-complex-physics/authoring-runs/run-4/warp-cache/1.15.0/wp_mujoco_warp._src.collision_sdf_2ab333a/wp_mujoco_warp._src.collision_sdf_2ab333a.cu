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




struct AABB_886b22fe
{
    wp::vec_t<3, wp::float32> min;
    wp::vec_t<3, wp::float32> max;


    AABB_886b22fe() = default;
    CUDA_CALLABLE AABB_886b22fe(wp::vec_t<3, wp::float32> const& min,
    wp::vec_t<3, wp::float32> const& max = {})
        : min{min}
        , max{max}

    {
    }

    CUDA_CALLABLE AABB_886b22fe& operator += (const AABB_886b22fe& rhs)
    {    min += rhs.min;
    max += rhs.max;

        return *this;}

};

static CUDA_CALLABLE void adj_AABB_886b22fe(wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> & adj_min,
    wp::vec_t<3, wp::float32> & adj_max,
    AABB_886b22fe & adj_ret)
{
    adj_min += adj_ret.min;
    adj_max += adj_ret.max;
}

// Required when compiling adjoints.
CUDA_CALLABLE AABB_886b22fe add(const AABB_886b22fe& a, const AABB_886b22fe& b)
{
    return AABB_886b22fe();
}

CUDA_CALLABLE void adj_atomic_add(AABB_886b22fe* p, AABB_886b22fe t)
{
    wp::adj_atomic_add(&p->min, t.min);
    wp::adj_atomic_add(&p->max, t.max);
}




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




struct MeshData_52eaa0fa
{
    wp::int32 nmeshface;
    wp::array_t<wp::int32> mesh_vertadr;
    wp::array_t<wp::vec_t<3, wp::float32>> mesh_vert;
    wp::array_t<wp::int32> mesh_faceadr;
    wp::array_t<wp::vec_t<3, wp::int32>> mesh_face;
    wp::int32 data_id;
    wp::vec_t<3, wp::float32> pos;
    wp::mat_t<3, 3, wp::float32> mat;
    wp::vec_t<3, wp::float32> size;
    wp::vec_t<3, wp::float32> pnt;
    wp::vec_t<3, wp::float32> vec;
    bool valid;


    MeshData_52eaa0fa() = default;
    CUDA_CALLABLE MeshData_52eaa0fa(wp::int32 const& nmeshface,
    wp::array_t<wp::int32> const& mesh_vertadr = {},
    wp::array_t<wp::vec_t<3, wp::float32>> const& mesh_vert = {},
    wp::array_t<wp::int32> const& mesh_faceadr = {},
    wp::array_t<wp::vec_t<3, wp::int32>> const& mesh_face = {},
    wp::int32 const& data_id = {},
    wp::vec_t<3, wp::float32> const& pos = {},
    wp::mat_t<3, 3, wp::float32> const& mat = {},
    wp::vec_t<3, wp::float32> const& size = {},
    wp::vec_t<3, wp::float32> const& pnt = {},
    wp::vec_t<3, wp::float32> const& vec = {},
    bool const& valid = {})
        : nmeshface{nmeshface}
        , mesh_vertadr{mesh_vertadr}
        , mesh_vert{mesh_vert}
        , mesh_faceadr{mesh_faceadr}
        , mesh_face{mesh_face}
        , data_id{data_id}
        , pos{pos}
        , mat{mat}
        , size{size}
        , pnt{pnt}
        , vec{vec}
        , valid{valid}

    {
    }

    CUDA_CALLABLE MeshData_52eaa0fa& operator += (const MeshData_52eaa0fa& rhs)
    {    nmeshface += rhs.nmeshface;
    data_id += rhs.data_id;
    pos += rhs.pos;
    mat += rhs.mat;
    size += rhs.size;
    pnt += rhs.pnt;
    vec += rhs.vec;

        return *this;}

};

static CUDA_CALLABLE void adj_MeshData_52eaa0fa(wp::int32 const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::vec_t<3, wp::float32>> const&,
    wp::array_t<wp::int32> const&,
    wp::array_t<wp::vec_t<3, wp::int32>> const&,
    wp::int32 const&,
    wp::vec_t<3, wp::float32> const&,
    wp::mat_t<3, 3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    bool const&,
    wp::int32 & adj_nmeshface,
    wp::array_t<wp::int32> & adj_mesh_vertadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_vert,
    wp::array_t<wp::int32> & adj_mesh_faceadr,
    wp::array_t<wp::vec_t<3, wp::int32>> & adj_mesh_face,
    wp::int32 & adj_data_id,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_mat,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    bool & adj_valid,
    MeshData_52eaa0fa & adj_ret)
{
    adj_nmeshface += adj_ret.nmeshface;
    adj_mesh_vertadr = adj_ret.mesh_vertadr;
    adj_mesh_vert = adj_ret.mesh_vert;
    adj_mesh_faceadr = adj_ret.mesh_faceadr;
    adj_mesh_face = adj_ret.mesh_face;
    adj_data_id += adj_ret.data_id;
    adj_pos += adj_ret.pos;
    adj_mat += adj_ret.mat;
    adj_size += adj_ret.size;
    adj_pnt += adj_ret.pnt;
    adj_vec += adj_ret.vec;
    adj_valid += adj_ret.valid;
}

// Required when compiling adjoints.
CUDA_CALLABLE MeshData_52eaa0fa add(const MeshData_52eaa0fa& a, const MeshData_52eaa0fa& b)
{
    return MeshData_52eaa0fa();
}

CUDA_CALLABLE void adj_atomic_add(MeshData_52eaa0fa* p, MeshData_52eaa0fa t)
{
    wp::adj_atomic_add(&p->nmeshface, t.nmeshface);
    wp::adj_atomic_add(&p->mesh_vertadr, t.mesh_vertadr);
    wp::adj_atomic_add(&p->mesh_vert, t.mesh_vert);
    wp::adj_atomic_add(&p->mesh_faceadr, t.mesh_faceadr);
    wp::adj_atomic_add(&p->mesh_face, t.mesh_face);
    wp::adj_atomic_add(&p->data_id, t.data_id);
    wp::adj_atomic_add(&p->pos, t.pos);
    wp::adj_atomic_add(&p->mat, t.mat);
    wp::adj_atomic_add(&p->size, t.size);
    wp::adj_atomic_add(&p->pnt, t.pnt);
    wp::adj_atomic_add(&p->vec, t.vec);
    wp::adj_atomic_add(&p->valid, t.valid);
}




struct OptimizationParams_51b33fc1
{
    wp::mat_t<3, 3, wp::float32> rel_mat;
    wp::vec_t<3, wp::float32> rel_pos;
    wp::vec_t<128, wp::float32> attr1;
    wp::vec_t<128, wp::float32> attr2;


    OptimizationParams_51b33fc1() = default;
    CUDA_CALLABLE OptimizationParams_51b33fc1(wp::mat_t<3, 3, wp::float32> const& rel_mat,
    wp::vec_t<3, wp::float32> const& rel_pos = {},
    wp::vec_t<128, wp::float32> const& attr1 = {},
    wp::vec_t<128, wp::float32> const& attr2 = {})
        : rel_mat{rel_mat}
        , rel_pos{rel_pos}
        , attr1{attr1}
        , attr2{attr2}

    {
    }

    CUDA_CALLABLE OptimizationParams_51b33fc1& operator += (const OptimizationParams_51b33fc1& rhs)
    {    rel_mat += rhs.rel_mat;
    rel_pos += rhs.rel_pos;
    attr1 += rhs.attr1;
    attr2 += rhs.attr2;

        return *this;}

};

static CUDA_CALLABLE void adj_OptimizationParams_51b33fc1(wp::mat_t<3, 3, wp::float32> const&,
    wp::vec_t<3, wp::float32> const&,
    wp::vec_t<128, wp::float32> const&,
    wp::vec_t<128, wp::float32> const&,
    wp::mat_t<3, 3, wp::float32> & adj_rel_mat,
    wp::vec_t<3, wp::float32> & adj_rel_pos,
    wp::vec_t<128, wp::float32> & adj_attr1,
    wp::vec_t<128, wp::float32> & adj_attr2,
    OptimizationParams_51b33fc1 & adj_ret)
{
    adj_rel_mat += adj_ret.rel_mat;
    adj_rel_pos += adj_ret.rel_pos;
    adj_attr1 += adj_ret.attr1;
    adj_attr2 += adj_ret.attr2;
}

// Required when compiling adjoints.
CUDA_CALLABLE OptimizationParams_51b33fc1 add(const OptimizationParams_51b33fc1& a, const OptimizationParams_51b33fc1& b)
{
    return OptimizationParams_51b33fc1();
}

CUDA_CALLABLE void adj_atomic_add(OptimizationParams_51b33fc1* p, OptimizationParams_51b33fc1 t)
{
    wp::adj_atomic_add(&p->rel_mat, t.rel_mat);
    wp::adj_atomic_add(&p->rel_pos, t.rel_pos);
    wp::adj_atomic_add(&p->attr1, t.attr1);
    wp::adj_atomic_add(&p->attr2, t.attr2);
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:131
static CUDA_CALLABLE AABB_886b22fe transform_aabb_0(
    wp::vec_t<3, wp::float32> var_aabb_pos,
    wp::vec_t<3, wp::float32> var_aabb_size,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_ori)
{
    //---------
    // primal vars
    AABB_886b22fe var_0;
    const wp::float32 var_1 = -1000000000.0;
    const wp::float32 var_2 = -1000000000.0;
    const wp::float32 var_3 = -1000000000.0;
    wp::vec_t<3, wp::float32> var_4;
    const wp::float32 var_5 = 1000000000.0;
    const wp::float32 var_6 = 1000000000.0;
    const wp::float32 var_7 = 1000000000.0;
    wp::vec_t<3, wp::float32> var_8;
    const wp::int32 var_9 = 0;
    const wp::int32 var_10 = 0;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::int32 var_13;
    const wp::float32 var_14 = 1.0;
    const wp::float32 var_15 = -1.0;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::int32 var_18 = 1;
    wp::float32 var_19;
    const wp::int32 var_20 = 2;
    wp::int32 var_21;
    const wp::float32 var_22 = 1.0;
    const wp::float32 var_23 = -1.0;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::int32 var_26 = 2;
    wp::float32 var_27;
    const wp::int32 var_28 = 4;
    wp::int32 var_29;
    const wp::float32 var_30 = 1.0;
    const wp::float32 var_31 = -1.0;
    wp::float32 var_32;
    wp::float32 var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32>* var_38;
    wp::vec_t<3, wp::float32> var_39;
    wp::vec_t<3, wp::float32> var_40;
    wp::vec_t<3, wp::float32>* var_41;
    wp::vec_t<3, wp::float32> var_42;
    wp::vec_t<3, wp::float32> var_43;
    const wp::int32 var_44 = 1;
    const wp::int32 var_45 = 0;
    wp::float32 var_46;
    const wp::int32 var_47 = 1;
    wp::int32 var_48;
    const wp::float32 var_49 = 1.0;
    const wp::float32 var_50 = -1.0;
    wp::float32 var_51;
    wp::float32 var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    const wp::int32 var_55 = 2;
    wp::int32 var_56;
    const wp::float32 var_57 = 1.0;
    const wp::float32 var_58 = -1.0;
    wp::float32 var_59;
    wp::float32 var_60;
    const wp::int32 var_61 = 2;
    wp::float32 var_62;
    const wp::int32 var_63 = 4;
    wp::int32 var_64;
    const wp::float32 var_65 = 1.0;
    const wp::float32 var_66 = -1.0;
    wp::float32 var_67;
    wp::float32 var_68;
    wp::vec_t<3, wp::float32> var_69;
    wp::vec_t<3, wp::float32> var_70;
    wp::vec_t<3, wp::float32> var_71;
    wp::vec_t<3, wp::float32> var_72;
    wp::vec_t<3, wp::float32>* var_73;
    wp::vec_t<3, wp::float32> var_74;
    wp::vec_t<3, wp::float32> var_75;
    wp::vec_t<3, wp::float32>* var_76;
    wp::vec_t<3, wp::float32> var_77;
    wp::vec_t<3, wp::float32> var_78;
    const wp::int32 var_79 = 2;
    const wp::int32 var_80 = 0;
    wp::float32 var_81;
    const wp::int32 var_82 = 1;
    wp::int32 var_83;
    const wp::float32 var_84 = 1.0;
    const wp::float32 var_85 = -1.0;
    wp::float32 var_86;
    wp::float32 var_87;
    const wp::int32 var_88 = 1;
    wp::float32 var_89;
    const wp::int32 var_90 = 2;
    wp::int32 var_91;
    const wp::float32 var_92 = 1.0;
    const wp::float32 var_93 = -1.0;
    wp::float32 var_94;
    wp::float32 var_95;
    const wp::int32 var_96 = 2;
    wp::float32 var_97;
    const wp::int32 var_98 = 4;
    wp::int32 var_99;
    const wp::float32 var_100 = 1.0;
    const wp::float32 var_101 = -1.0;
    wp::float32 var_102;
    wp::float32 var_103;
    wp::vec_t<3, wp::float32> var_104;
    wp::vec_t<3, wp::float32> var_105;
    wp::vec_t<3, wp::float32> var_106;
    wp::vec_t<3, wp::float32> var_107;
    wp::vec_t<3, wp::float32>* var_108;
    wp::vec_t<3, wp::float32> var_109;
    wp::vec_t<3, wp::float32> var_110;
    wp::vec_t<3, wp::float32>* var_111;
    wp::vec_t<3, wp::float32> var_112;
    wp::vec_t<3, wp::float32> var_113;
    const wp::int32 var_114 = 3;
    const wp::int32 var_115 = 0;
    wp::float32 var_116;
    const wp::int32 var_117 = 1;
    wp::int32 var_118;
    const wp::float32 var_119 = 1.0;
    const wp::float32 var_120 = -1.0;
    wp::float32 var_121;
    wp::float32 var_122;
    const wp::int32 var_123 = 1;
    wp::float32 var_124;
    const wp::int32 var_125 = 2;
    wp::int32 var_126;
    const wp::float32 var_127 = 1.0;
    const wp::float32 var_128 = -1.0;
    wp::float32 var_129;
    wp::float32 var_130;
    const wp::int32 var_131 = 2;
    wp::float32 var_132;
    const wp::int32 var_133 = 4;
    wp::int32 var_134;
    const wp::float32 var_135 = 1.0;
    const wp::float32 var_136 = -1.0;
    wp::float32 var_137;
    wp::float32 var_138;
    wp::vec_t<3, wp::float32> var_139;
    wp::vec_t<3, wp::float32> var_140;
    wp::vec_t<3, wp::float32> var_141;
    wp::vec_t<3, wp::float32> var_142;
    wp::vec_t<3, wp::float32>* var_143;
    wp::vec_t<3, wp::float32> var_144;
    wp::vec_t<3, wp::float32> var_145;
    wp::vec_t<3, wp::float32>* var_146;
    wp::vec_t<3, wp::float32> var_147;
    wp::vec_t<3, wp::float32> var_148;
    const wp::int32 var_149 = 4;
    const wp::int32 var_150 = 0;
    wp::float32 var_151;
    const wp::int32 var_152 = 1;
    wp::int32 var_153;
    const wp::float32 var_154 = 1.0;
    const wp::float32 var_155 = -1.0;
    wp::float32 var_156;
    wp::float32 var_157;
    const wp::int32 var_158 = 1;
    wp::float32 var_159;
    const wp::int32 var_160 = 2;
    wp::int32 var_161;
    const wp::float32 var_162 = 1.0;
    const wp::float32 var_163 = -1.0;
    wp::float32 var_164;
    wp::float32 var_165;
    const wp::int32 var_166 = 2;
    wp::float32 var_167;
    const wp::int32 var_168 = 4;
    wp::int32 var_169;
    const wp::float32 var_170 = 1.0;
    const wp::float32 var_171 = -1.0;
    wp::float32 var_172;
    wp::float32 var_173;
    wp::vec_t<3, wp::float32> var_174;
    wp::vec_t<3, wp::float32> var_175;
    wp::vec_t<3, wp::float32> var_176;
    wp::vec_t<3, wp::float32> var_177;
    wp::vec_t<3, wp::float32>* var_178;
    wp::vec_t<3, wp::float32> var_179;
    wp::vec_t<3, wp::float32> var_180;
    wp::vec_t<3, wp::float32>* var_181;
    wp::vec_t<3, wp::float32> var_182;
    wp::vec_t<3, wp::float32> var_183;
    const wp::int32 var_184 = 5;
    const wp::int32 var_185 = 0;
    wp::float32 var_186;
    const wp::int32 var_187 = 1;
    wp::int32 var_188;
    const wp::float32 var_189 = 1.0;
    const wp::float32 var_190 = -1.0;
    wp::float32 var_191;
    wp::float32 var_192;
    const wp::int32 var_193 = 1;
    wp::float32 var_194;
    const wp::int32 var_195 = 2;
    wp::int32 var_196;
    const wp::float32 var_197 = 1.0;
    const wp::float32 var_198 = -1.0;
    wp::float32 var_199;
    wp::float32 var_200;
    const wp::int32 var_201 = 2;
    wp::float32 var_202;
    const wp::int32 var_203 = 4;
    wp::int32 var_204;
    const wp::float32 var_205 = 1.0;
    const wp::float32 var_206 = -1.0;
    wp::float32 var_207;
    wp::float32 var_208;
    wp::vec_t<3, wp::float32> var_209;
    wp::vec_t<3, wp::float32> var_210;
    wp::vec_t<3, wp::float32> var_211;
    wp::vec_t<3, wp::float32> var_212;
    wp::vec_t<3, wp::float32>* var_213;
    wp::vec_t<3, wp::float32> var_214;
    wp::vec_t<3, wp::float32> var_215;
    wp::vec_t<3, wp::float32>* var_216;
    wp::vec_t<3, wp::float32> var_217;
    wp::vec_t<3, wp::float32> var_218;
    const wp::int32 var_219 = 6;
    const wp::int32 var_220 = 0;
    wp::float32 var_221;
    const wp::int32 var_222 = 1;
    wp::int32 var_223;
    const wp::float32 var_224 = 1.0;
    const wp::float32 var_225 = -1.0;
    wp::float32 var_226;
    wp::float32 var_227;
    const wp::int32 var_228 = 1;
    wp::float32 var_229;
    const wp::int32 var_230 = 2;
    wp::int32 var_231;
    const wp::float32 var_232 = 1.0;
    const wp::float32 var_233 = -1.0;
    wp::float32 var_234;
    wp::float32 var_235;
    const wp::int32 var_236 = 2;
    wp::float32 var_237;
    const wp::int32 var_238 = 4;
    wp::int32 var_239;
    const wp::float32 var_240 = 1.0;
    const wp::float32 var_241 = -1.0;
    wp::float32 var_242;
    wp::float32 var_243;
    wp::vec_t<3, wp::float32> var_244;
    wp::vec_t<3, wp::float32> var_245;
    wp::vec_t<3, wp::float32> var_246;
    wp::vec_t<3, wp::float32> var_247;
    wp::vec_t<3, wp::float32>* var_248;
    wp::vec_t<3, wp::float32> var_249;
    wp::vec_t<3, wp::float32> var_250;
    wp::vec_t<3, wp::float32>* var_251;
    wp::vec_t<3, wp::float32> var_252;
    wp::vec_t<3, wp::float32> var_253;
    const wp::int32 var_254 = 7;
    const wp::int32 var_255 = 0;
    wp::float32 var_256;
    const wp::int32 var_257 = 1;
    wp::int32 var_258;
    const wp::float32 var_259 = 1.0;
    const wp::float32 var_260 = -1.0;
    wp::float32 var_261;
    wp::float32 var_262;
    const wp::int32 var_263 = 1;
    wp::float32 var_264;
    const wp::int32 var_265 = 2;
    wp::int32 var_266;
    const wp::float32 var_267 = 1.0;
    const wp::float32 var_268 = -1.0;
    wp::float32 var_269;
    wp::float32 var_270;
    const wp::int32 var_271 = 2;
    wp::float32 var_272;
    const wp::int32 var_273 = 4;
    wp::int32 var_274;
    const wp::float32 var_275 = 1.0;
    const wp::float32 var_276 = -1.0;
    wp::float32 var_277;
    wp::float32 var_278;
    wp::vec_t<3, wp::float32> var_279;
    wp::vec_t<3, wp::float32> var_280;
    wp::vec_t<3, wp::float32> var_281;
    wp::vec_t<3, wp::float32> var_282;
    wp::vec_t<3, wp::float32>* var_283;
    wp::vec_t<3, wp::float32> var_284;
    wp::vec_t<3, wp::float32> var_285;
    wp::vec_t<3, wp::float32>* var_286;
    wp::vec_t<3, wp::float32> var_287;
    wp::vec_t<3, wp::float32> var_288;
    //---------
    // forward
    // def transform_aabb(aabb_pos: wp.vec3, aabb_size: wp.vec3, pos: wp.vec3, ori: wp.mat33) -> AABB:       <L 132>
    // aabb = AABB()                                                                          <L 133>
    var_0 = AABB_886b22fe();
    // aabb.max = wp.vec3(-1000000000.0, -1000000000.0, -1000000000.0)                        <L 134>
    var_4 = wp::vec_t<3, wp::float32>(var_1, var_2, var_3);
    var_0.max = var_4;
    // aabb.min = wp.vec3(1000000000.0, 1000000000.0, 1000000000.0)                           <L 135>
    var_8 = wp::vec_t<3, wp::float32>(var_5, var_6, var_7);
    var_0.min = var_8;
    // for i in range(8):                                                                     <L 136>
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_11 = wp::extract(var_aabb_size, var_10);
    var_13 = wp::bit_and(var_9, var_12);
    if (var_13) {
    }
    if (!var_13) {
    }
    var_16 = wp::where(var_13, var_14, var_15);
    var_17 = wp::mul(var_11, var_16);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_19 = wp::extract(var_aabb_size, var_18);
    var_21 = wp::bit_and(var_9, var_20);
    if (var_21) {
    }
    if (!var_21) {
    }
    var_24 = wp::where(var_21, var_22, var_23);
    var_25 = wp::mul(var_19, var_24);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_27 = wp::extract(var_aabb_size, var_26);
    var_29 = wp::bit_and(var_9, var_28);
    if (var_29) {
    }
    if (!var_29) {
    }
    var_32 = wp::where(var_29, var_30, var_31);
    var_33 = wp::mul(var_27, var_32);
    var_34 = wp::vec_t<3, wp::float32>(var_17, var_25, var_33);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_35 = wp::add(var_34, var_aabb_pos);
    var_36 = wp::mul(var_ori, var_35);
    var_37 = wp::add(var_36, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_38 = &((var_0).min);
    var_40 = wp::load(var_38);
    var_39 = wp::min(var_40, var_37);
    var_0.min = var_39;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_41 = &((var_0).max);
    var_43 = wp::load(var_41);
    var_42 = wp::max(var_43, var_37);
    var_0.max = var_42;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_46 = wp::extract(var_aabb_size, var_45);
    var_48 = wp::bit_and(var_44, var_47);
    if (var_48) {
    }
    if (!var_48) {
    }
    var_51 = wp::where(var_48, var_49, var_50);
    var_52 = wp::mul(var_46, var_51);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_54 = wp::extract(var_aabb_size, var_53);
    var_56 = wp::bit_and(var_44, var_55);
    if (var_56) {
    }
    if (!var_56) {
    }
    var_59 = wp::where(var_56, var_57, var_58);
    var_60 = wp::mul(var_54, var_59);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_62 = wp::extract(var_aabb_size, var_61);
    var_64 = wp::bit_and(var_44, var_63);
    if (var_64) {
    }
    if (!var_64) {
    }
    var_67 = wp::where(var_64, var_65, var_66);
    var_68 = wp::mul(var_62, var_67);
    var_69 = wp::vec_t<3, wp::float32>(var_52, var_60, var_68);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_70 = wp::add(var_69, var_aabb_pos);
    var_71 = wp::mul(var_ori, var_70);
    var_72 = wp::add(var_71, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_73 = &((var_0).min);
    var_75 = wp::load(var_73);
    var_74 = wp::min(var_75, var_72);
    var_0.min = var_74;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_76 = &((var_0).max);
    var_78 = wp::load(var_76);
    var_77 = wp::max(var_78, var_72);
    var_0.max = var_77;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_81 = wp::extract(var_aabb_size, var_80);
    var_83 = wp::bit_and(var_79, var_82);
    if (var_83) {
    }
    if (!var_83) {
    }
    var_86 = wp::where(var_83, var_84, var_85);
    var_87 = wp::mul(var_81, var_86);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_89 = wp::extract(var_aabb_size, var_88);
    var_91 = wp::bit_and(var_79, var_90);
    if (var_91) {
    }
    if (!var_91) {
    }
    var_94 = wp::where(var_91, var_92, var_93);
    var_95 = wp::mul(var_89, var_94);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_97 = wp::extract(var_aabb_size, var_96);
    var_99 = wp::bit_and(var_79, var_98);
    if (var_99) {
    }
    if (!var_99) {
    }
    var_102 = wp::where(var_99, var_100, var_101);
    var_103 = wp::mul(var_97, var_102);
    var_104 = wp::vec_t<3, wp::float32>(var_87, var_95, var_103);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_105 = wp::add(var_104, var_aabb_pos);
    var_106 = wp::mul(var_ori, var_105);
    var_107 = wp::add(var_106, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_108 = &((var_0).min);
    var_110 = wp::load(var_108);
    var_109 = wp::min(var_110, var_107);
    var_0.min = var_109;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_111 = &((var_0).max);
    var_113 = wp::load(var_111);
    var_112 = wp::max(var_113, var_107);
    var_0.max = var_112;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_116 = wp::extract(var_aabb_size, var_115);
    var_118 = wp::bit_and(var_114, var_117);
    if (var_118) {
    }
    if (!var_118) {
    }
    var_121 = wp::where(var_118, var_119, var_120);
    var_122 = wp::mul(var_116, var_121);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_124 = wp::extract(var_aabb_size, var_123);
    var_126 = wp::bit_and(var_114, var_125);
    if (var_126) {
    }
    if (!var_126) {
    }
    var_129 = wp::where(var_126, var_127, var_128);
    var_130 = wp::mul(var_124, var_129);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_132 = wp::extract(var_aabb_size, var_131);
    var_134 = wp::bit_and(var_114, var_133);
    if (var_134) {
    }
    if (!var_134) {
    }
    var_137 = wp::where(var_134, var_135, var_136);
    var_138 = wp::mul(var_132, var_137);
    var_139 = wp::vec_t<3, wp::float32>(var_122, var_130, var_138);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_140 = wp::add(var_139, var_aabb_pos);
    var_141 = wp::mul(var_ori, var_140);
    var_142 = wp::add(var_141, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_143 = &((var_0).min);
    var_145 = wp::load(var_143);
    var_144 = wp::min(var_145, var_142);
    var_0.min = var_144;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_146 = &((var_0).max);
    var_148 = wp::load(var_146);
    var_147 = wp::max(var_148, var_142);
    var_0.max = var_147;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_151 = wp::extract(var_aabb_size, var_150);
    var_153 = wp::bit_and(var_149, var_152);
    if (var_153) {
    }
    if (!var_153) {
    }
    var_156 = wp::where(var_153, var_154, var_155);
    var_157 = wp::mul(var_151, var_156);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_159 = wp::extract(var_aabb_size, var_158);
    var_161 = wp::bit_and(var_149, var_160);
    if (var_161) {
    }
    if (!var_161) {
    }
    var_164 = wp::where(var_161, var_162, var_163);
    var_165 = wp::mul(var_159, var_164);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_167 = wp::extract(var_aabb_size, var_166);
    var_169 = wp::bit_and(var_149, var_168);
    if (var_169) {
    }
    if (!var_169) {
    }
    var_172 = wp::where(var_169, var_170, var_171);
    var_173 = wp::mul(var_167, var_172);
    var_174 = wp::vec_t<3, wp::float32>(var_157, var_165, var_173);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_175 = wp::add(var_174, var_aabb_pos);
    var_176 = wp::mul(var_ori, var_175);
    var_177 = wp::add(var_176, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_178 = &((var_0).min);
    var_180 = wp::load(var_178);
    var_179 = wp::min(var_180, var_177);
    var_0.min = var_179;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_181 = &((var_0).max);
    var_183 = wp::load(var_181);
    var_182 = wp::max(var_183, var_177);
    var_0.max = var_182;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_186 = wp::extract(var_aabb_size, var_185);
    var_188 = wp::bit_and(var_184, var_187);
    if (var_188) {
    }
    if (!var_188) {
    }
    var_191 = wp::where(var_188, var_189, var_190);
    var_192 = wp::mul(var_186, var_191);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_194 = wp::extract(var_aabb_size, var_193);
    var_196 = wp::bit_and(var_184, var_195);
    if (var_196) {
    }
    if (!var_196) {
    }
    var_199 = wp::where(var_196, var_197, var_198);
    var_200 = wp::mul(var_194, var_199);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_202 = wp::extract(var_aabb_size, var_201);
    var_204 = wp::bit_and(var_184, var_203);
    if (var_204) {
    }
    if (!var_204) {
    }
    var_207 = wp::where(var_204, var_205, var_206);
    var_208 = wp::mul(var_202, var_207);
    var_209 = wp::vec_t<3, wp::float32>(var_192, var_200, var_208);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_210 = wp::add(var_209, var_aabb_pos);
    var_211 = wp::mul(var_ori, var_210);
    var_212 = wp::add(var_211, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_213 = &((var_0).min);
    var_215 = wp::load(var_213);
    var_214 = wp::min(var_215, var_212);
    var_0.min = var_214;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_216 = &((var_0).max);
    var_218 = wp::load(var_216);
    var_217 = wp::max(var_218, var_212);
    var_0.max = var_217;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_221 = wp::extract(var_aabb_size, var_220);
    var_223 = wp::bit_and(var_219, var_222);
    if (var_223) {
    }
    if (!var_223) {
    }
    var_226 = wp::where(var_223, var_224, var_225);
    var_227 = wp::mul(var_221, var_226);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_229 = wp::extract(var_aabb_size, var_228);
    var_231 = wp::bit_and(var_219, var_230);
    if (var_231) {
    }
    if (!var_231) {
    }
    var_234 = wp::where(var_231, var_232, var_233);
    var_235 = wp::mul(var_229, var_234);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_237 = wp::extract(var_aabb_size, var_236);
    var_239 = wp::bit_and(var_219, var_238);
    if (var_239) {
    }
    if (!var_239) {
    }
    var_242 = wp::where(var_239, var_240, var_241);
    var_243 = wp::mul(var_237, var_242);
    var_244 = wp::vec_t<3, wp::float32>(var_227, var_235, var_243);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_245 = wp::add(var_244, var_aabb_pos);
    var_246 = wp::mul(var_ori, var_245);
    var_247 = wp::add(var_246, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_248 = &((var_0).min);
    var_250 = wp::load(var_248);
    var_249 = wp::min(var_250, var_247);
    var_0.min = var_249;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_251 = &((var_0).max);
    var_253 = wp::load(var_251);
    var_252 = wp::max(var_253, var_247);
    var_0.max = var_252;
    // vec = wp.vec3(                                                                         <L 137>
    // aabb_size.x * (1.0 if (i & 1) else -1.0),                                              <L 138>
    var_256 = wp::extract(var_aabb_size, var_255);
    var_258 = wp::bit_and(var_254, var_257);
    if (var_258) {
    }
    if (!var_258) {
    }
    var_261 = wp::where(var_258, var_259, var_260);
    var_262 = wp::mul(var_256, var_261);
    // aabb_size.y * (1.0 if (i & 2) else -1.0),                                              <L 139>
    var_264 = wp::extract(var_aabb_size, var_263);
    var_266 = wp::bit_and(var_254, var_265);
    if (var_266) {
    }
    if (!var_266) {
    }
    var_269 = wp::where(var_266, var_267, var_268);
    var_270 = wp::mul(var_264, var_269);
    // aabb_size.z * (1.0 if (i & 4) else -1.0),                                              <L 140>
    var_272 = wp::extract(var_aabb_size, var_271);
    var_274 = wp::bit_and(var_254, var_273);
    if (var_274) {
    }
    if (!var_274) {
    }
    var_277 = wp::where(var_274, var_275, var_276);
    var_278 = wp::mul(var_272, var_277);
    var_279 = wp::vec_t<3, wp::float32>(var_262, var_270, var_278);
    // frame_vec = ori * (vec + aabb_pos) + pos                                               <L 142>
    var_280 = wp::add(var_279, var_aabb_pos);
    var_281 = wp::mul(var_ori, var_280);
    var_282 = wp::add(var_281, var_pos);
    // aabb.min = wp.min(aabb.min, frame_vec)                                                 <L 143>
    var_283 = &((var_0).min);
    var_285 = wp::load(var_283);
    var_284 = wp::min(var_285, var_282);
    var_0.min = var_284;
    // aabb.max = wp.max(aabb.max, frame_vec)                                                 <L 144>
    var_286 = &((var_0).max);
    var_288 = wp::load(var_286);
    var_287 = wp::max(var_288, var_282);
    var_0.max = var_287;
    // return aabb                                                                            <L 145>
    return var_0;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:81
static CUDA_CALLABLE void get_sdf_params_0(
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> var_oct_coeff,
    wp::array_t<wp::int32> var_mesh_octadr,
    wp::array_t<wp::int32> var_plugin,
    wp::array_t<wp::vec_t<128, wp::float32>> var_plugin_attr,
    wp::int32 var_g_type,
    wp::vec_t<3, wp::float32> var_g_size,
    wp::int32 var_plugin_id,
    wp::int32 var_mesh_id,
    wp::vec_t<128, wp::float32> & ret_0,
    wp::int32 & ret_1,
    VolumeData_53ac1a2d & ret_2,
    MeshData_52eaa0fa & ret_3)
{
    //---------
    // primal vars
    wp::vec_t<128, wp::float32> var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    const wp::int32 var_3 = 0;
    const wp::int32 var_4 = 1;
    wp::float32 var_5;
    const wp::int32 var_6 = 1;
    const wp::int32 var_7 = 2;
    wp::float32 var_8;
    const wp::int32 var_9 = 2;
    const wp::int32 var_10 = -1;
    VolumeData_53ac1a2d var_11;
    bool var_12;
    const wp::int32 var_13 = 8;
    bool var_14;
    const wp::int32 var_15 = -1;
    bool var_16;
    wp::vec_t<128, wp::float32>* var_17;
    wp::vec_t<128, wp::float32> var_18;
    wp::vec_t<128, wp::float32> var_19;
    wp::int32* var_20;
    wp::int32 var_21;
    wp::int32 var_22;
    wp::vec_t<128, wp::float32> var_23;
    wp::int32 var_24;
    bool var_25;
    const wp::int32 var_26 = 8;
    bool var_27;
    const wp::int32 var_28 = -1;
    bool var_29;
    wp::int32* var_30;
    wp::int32 var_31;
    wp::int32 var_32;
    const wp::int32 var_33 = 0;
    wp::vec_t<3, wp::float32>* var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    const wp::int32 var_37 = 1;
    wp::vec_t<3, wp::float32>* var_38;
    wp::vec_t<3, wp::float32> var_39;
    wp::vec_t<3, wp::float32> var_40;
    const bool var_41 = true;
    bool var_42;
    const wp::int32 var_43 = 7;
    bool var_44;
    const wp::int32 var_45 = -1;
    bool var_46;
    wp::int32* var_47;
    const wp::int32 var_48 = -1;
    bool var_49;
    wp::int32 var_50;
    wp::int32* var_51;
    wp::int32 var_52;
    wp::int32 var_53;
    const wp::int32 var_54 = 0;
    wp::vec_t<3, wp::float32>* var_55;
    wp::vec_t<3, wp::float32> var_56;
    wp::vec_t<3, wp::float32> var_57;
    const wp::int32 var_58 = 1;
    wp::vec_t<3, wp::float32>* var_59;
    wp::vec_t<3, wp::float32> var_60;
    wp::vec_t<3, wp::float32> var_61;
    const bool var_62 = true;
    wp::int32 var_63;
    wp::int32 var_64;
    MeshData_52eaa0fa var_65;
    //---------
    // forward
    // def get_sdf_params(                                                                    <L 82>
    // attributes = vec_pluginattr()                                                          <L 97>
    var_0 = wp::vec_t<128, wp::float32>();
    // attributes[0] = g_size[0]                                                              <L 98>
    var_2 = wp::extract(var_g_size, var_1);
    wp::assign_inplace(var_0, var_3, var_2);
    // attributes[1] = g_size[1]                                                              <L 99>
    var_5 = wp::extract(var_g_size, var_4);
    wp::assign_inplace(var_0, var_6, var_5);
    // attributes[2] = g_size[2]                                                              <L 100>
    var_8 = wp::extract(var_g_size, var_7);
    wp::assign_inplace(var_0, var_9, var_8);
    // plugin_index = -1                                                                      <L 101>
    // volume_data = VolumeData()                                                             <L 102>
    var_11 = VolumeData_53ac1a2d();
    // if g_type == GeomType.SDF and plugin_id != -1:                                         <L 104>
    var_14 = (var_g_type == var_13);
    var_12 = var_14;
    if (var_12) {
        var_16 = (var_plugin_id != var_15);
        var_12 = var_12 && var_16;
    }
    if (var_12) {
        // attributes = plugin_attr[plugin_id]                                                <L 105>
        var_17 = wp::address(var_plugin_attr, var_plugin_id);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // plugin_index = plugin[plugin_id]                                                   <L 106>
        var_20 = wp::address(var_plugin, var_plugin_id);
        var_22 = wp::load(var_20);
        var_21 = wp::copy(var_22);
    }
    var_23 = wp::where(var_12, var_18, var_0);
    var_24 = wp::where(var_12, var_21, var_10);
    if (!var_12) {
        // elif g_type == GeomType.SDF and mesh_id != -1:                                     <L 108>
        var_27 = (var_g_type == var_26);
        var_25 = var_27;
        if (var_25) {
            var_29 = (var_mesh_id != var_28);
            var_25 = var_25 && var_29;
        }
        if (var_25) {
            // octadr = mesh_octadr[mesh_id]                                                  <L 109>
            var_30 = wp::address(var_mesh_octadr, var_mesh_id);
            var_32 = wp::load(var_30);
            var_31 = wp::copy(var_32);
            // volume_data.center = oct_aabb[octadr, 0]                                       <L 110>
            var_34 = wp::address(var_oct_aabb, var_31, var_33);
            var_36 = wp::load(var_34);
            var_35 = wp::copy(var_36);
            var_11.center = var_35;
            // volume_data.half_size = oct_aabb[octadr, 1]                                    <L 111>
            var_38 = wp::address(var_oct_aabb, var_31, var_37);
            var_40 = wp::load(var_38);
            var_39 = wp::copy(var_40);
            var_11.half_size = var_39;
            // volume_data.root = octadr                                                      <L 112>
            var_11.root = var_31;
            // volume_data.oct_aabb = oct_aabb                                                <L 113>
            var_11.oct_aabb = var_oct_aabb;
            // volume_data.oct_child = oct_child                                              <L 114>
            var_11.oct_child = var_oct_child;
            // volume_data.oct_coeff = oct_coeff                                              <L 115>
            var_11.oct_coeff = var_oct_coeff;
            // volume_data.valid = True                                                       <L 116>
            var_11.valid = var_41;
        }
        if (!var_25) {
            // elif g_type == GeomType.MESH and mesh_id != -1 and mesh_octadr[mesh_id] != -1:       <L 118>
            var_44 = (var_g_type == var_43);
            var_42 = var_44;
            if (var_42) {
                var_46 = (var_mesh_id != var_45);
                var_42 = var_42 && var_46;
            }
            if (var_42) {
                var_47 = wp::address(var_mesh_octadr, var_mesh_id);
                var_50 = wp::load(var_47);
                var_49 = (var_50 != var_48);
                var_42 = var_42 && var_49;
            }
            if (var_42) {
                // octadr = mesh_octadr[mesh_id]                                              <L 119>
                var_51 = wp::address(var_mesh_octadr, var_mesh_id);
                var_53 = wp::load(var_51);
                var_52 = wp::copy(var_53);
                // volume_data.center = oct_aabb[octadr, 0]                                   <L 120>
                var_55 = wp::address(var_oct_aabb, var_52, var_54);
                var_57 = wp::load(var_55);
                var_56 = wp::copy(var_57);
                var_11.center = var_56;
                // volume_data.half_size = oct_aabb[octadr, 1]                                <L 121>
                var_59 = wp::address(var_oct_aabb, var_52, var_58);
                var_61 = wp::load(var_59);
                var_60 = wp::copy(var_61);
                var_11.half_size = var_60;
                // volume_data.root = octadr                                                  <L 122>
                var_11.root = var_52;
                // volume_data.oct_aabb = oct_aabb                                            <L 123>
                var_11.oct_aabb = var_oct_aabb;
                // volume_data.oct_child = oct_child                                          <L 124>
                var_11.oct_child = var_oct_child;
                // volume_data.oct_coeff = oct_coeff                                          <L 125>
                var_11.oct_coeff = var_oct_coeff;
                // volume_data.valid = True                                                   <L 126>
                var_11.valid = var_62;
            }
            var_63 = wp::where(var_42, var_52, var_31);
        }
        var_64 = wp::where(var_25, var_31, var_63);
    }
    // return attributes, plugin_index, volume_data, MeshData()                               <L 128>
    var_65 = MeshData_52eaa0fa();
    ret_0 = var_23;
    ret_1 = var_24;
    ret_2 = var_11;
    ret_3 = var_65;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:60
static CUDA_CALLABLE wp::float32 halton_0(
    wp::int32 var_index,
    wp::int32 var_base)
{
    //---------
    // primal vars
    wp::int32 var_0;
    wp::float32 var_1;
    const wp::float32 var_2 = 1.0;
    wp::float32 var_3;
    wp::float32 var_4;
    const wp::float32 var_5 = 0.0;
    wp::float32 var_6;
    const wp::int32 var_7 = 0;
    bool var_8;
    wp::int32 var_9;
    wp::int32 var_10;
    wp::int32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::int32 var_16;
    //---------
    // forward
    // def halton(index: int, base: int) -> float:                                            <L 61>
    // n0 = index                                                                             <L 62>
    var_0 = wp::copy(var_index);
    // b = float(base)                                                                        <L 63>
    var_1 = wp::float(var_base);
    // f = float(1.0) / b                                                                     <L 64>
    var_3 = wp::float(var_2);
    var_4 = wp::div(var_3, var_1);
    // hn = float(0.0)                                                                        <L 65>
    var_6 = wp::float(var_5);
    // while n0 > 0:                                                                          <L 67>
    start_while_0:;
    var_8 = (var_0 > var_7);
    if ((var_8) == false) goto end_while_0;
        // n1 = n0 // base                                                                    <L 68>
        var_9 = wp::floordiv(var_0, var_base);
        // r = n0 - n1 * base                                                                 <L 69>
        var_10 = wp::mul(var_9, var_base);
        var_11 = wp::sub(var_0, var_10);
        // hn += f * float(r)                                                                 <L 70>
        var_12 = wp::float(var_11);
        var_13 = wp::mul(var_4, var_12);
        var_14 = wp::add(var_6, var_13);
        // f /= b                                                                             <L 71>
        var_15 = wp::div(var_4, var_1);
        // n0 = n1                                                                            <L 72>
        var_16 = wp::copy(var_9);
        wp::assign(var_0, var_16);
        wp::assign(var_4, var_15);
        wp::assign(var_6, var_14);
    goto start_while_0;
    end_while_0:;
    // return hn                                                                              <L 74>
    return var_6;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:158
static CUDA_CALLABLE wp::float32 sphere_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::int32 var_1 = 0;
    wp::float32 var_2;
    wp::float32 var_3;
    //---------
    // forward
    // def sphere(p: wp.vec3, size: wp.vec3) -> float:                                        <L 159>
    // return wp.length(p) - size[0]                                                          <L 160>
    var_0 = wp::length(var_p);
    var_2 = wp::extract(var_size, var_1);
    var_3 = wp::sub(var_0, var_2);
    return var_3;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:187
static CUDA_CALLABLE wp::float32 capsule_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 2;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    const wp::int32 var_8 = 0;
    wp::float32 var_9;
    const wp::int32 var_10 = 1;
    wp::float32 var_11;
    const wp::int32 var_12 = 2;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    //---------
    // forward
    // def capsule(p: wp.vec3, size: wp.vec3) -> float:                                       <L 188>
    // r = size[0]                                                                            <L 189>
    var_1 = wp::extract(var_size, var_0);
    // h = size[1]                                                                            <L 190>
    var_3 = wp::extract(var_size, var_2);
    // pz_clamped = wp.clamp(p[2], -h, h)                                                     <L 191>
    var_5 = wp::extract(var_p, var_4);
    var_6 = wp::neg(var_3);
    var_7 = wp::clamp(var_5, var_6, var_3);
    // diff = wp.vec3(p[0], p[1], p[2] - pz_clamped)                                          <L 192>
    var_9 = wp::extract(var_p, var_8);
    var_11 = wp::extract(var_p, var_10);
    var_13 = wp::extract(var_p, var_12);
    var_14 = wp::sub(var_13, var_7);
    var_15 = wp::vec_t<3, wp::float32>(var_9, var_11, var_14);
    // return wp.length(diff) - r                                                             <L 193>
    var_16 = wp::length(var_15);
    var_17 = wp::sub(var_16, var_1);
    return var_17;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:196
static CUDA_CALLABLE wp::float32 cylinder_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 0;
    wp::float32 var_5;
    const wp::int32 var_6 = 1;
    wp::float32 var_7;
    wp::vec_t<2, wp::float32> var_8;
    wp::float32 var_9;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    const wp::float32 var_16 = 0.0;
    wp::float32 var_17;
    const wp::float32 var_18 = 0.0;
    wp::float32 var_19;
    const wp::float32 var_20 = 0.0;
    wp::float32 var_21;
    wp::vec_t<2, wp::float32> var_22;
    wp::float32 var_23;
    wp::float32 var_24;
    //---------
    // forward
    // def cylinder(p: wp.vec3, size: wp.vec3) -> float:                                      <L 197>
    // r = size[0]                                                                            <L 198>
    var_1 = wp::extract(var_size, var_0);
    // h = size[1]                                                                            <L 199>
    var_3 = wp::extract(var_size, var_2);
    // dx = wp.length(wp.vec2(p[0], p[1])) - r                                                <L 200>
    var_5 = wp::extract(var_p, var_4);
    var_7 = wp::extract(var_p, var_6);
    var_8 = wp::vec_t<2, wp::float32>(var_5, var_7);
    var_9 = wp::length(var_8);
    var_10 = wp::sub(var_9, var_1);
    // dy = wp.abs(p[2]) - h                                                                  <L 201>
    var_12 = wp::extract(var_p, var_11);
    var_13 = wp::abs(var_12);
    var_14 = wp::sub(var_13, var_3);
    // return wp.min(wp.max(dx, dy), 0.0) + wp.length(wp.vec2(wp.max(dx, 0.0), wp.max(dy, 0.0)))       <L 202>
    var_15 = wp::max(var_10, var_14);
    var_17 = wp::min(var_15, var_16);
    var_19 = wp::max(var_10, var_18);
    var_21 = wp::max(var_14, var_20);
    var_22 = wp::vec_t<2, wp::float32>(var_19, var_21);
    var_23 = wp::length(var_22);
    var_24 = wp::add(var_17, var_23);
    return var_24;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:148
static CUDA_CALLABLE wp::vec_t<3, wp::float32> radial_field_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> var_x,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 0;
    wp::float32 var_7;
    wp::float32 var_8;
    const wp::int32 var_9 = 1;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 1;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 2;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::int32 var_18 = 2;
    wp::float32 var_19;
    wp::float32 var_20;
    //---------
    // forward
    // def radial_field(a: wp.vec3, x: wp.vec3, size: wp.vec3) -> wp.vec3:                    <L 149>
    // field = wp.cw_div(-size, a)                                                            <L 150>
    var_0 = wp::neg(var_size);
    var_1 = wp::cw_div(var_0, var_a);
    // field = wp.normalize(field)                                                            <L 151>
    var_2 = wp::normalize(var_1);
    // field[0] *= wp.sign(x[0])                                                              <L 152>
    var_4 = wp::extract(var_x, var_3);
    var_5 = wp::sign(var_4);
    var_7 = wp::extract(var_2, var_6);
    var_8 = wp::mul(var_7, var_5);
    wp::assign_inplace(var_2, var_6, var_8);
    // field[1] *= wp.sign(x[1])                                                              <L 153>
    var_10 = wp::extract(var_x, var_9);
    var_11 = wp::sign(var_10);
    var_13 = wp::extract(var_2, var_12);
    var_14 = wp::mul(var_13, var_11);
    wp::assign_inplace(var_2, var_12, var_14);
    // field[2] *= wp.sign(x[2])                                                              <L 154>
    var_16 = wp::extract(var_x, var_15);
    var_17 = wp::sign(var_16);
    var_19 = wp::extract(var_2, var_18);
    var_20 = wp::mul(var_19, var_17);
    wp::assign_inplace(var_2, var_18, var_20);
    // return field                                                                           <L 155>
    return var_2;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:163
static CUDA_CALLABLE wp::float32 box_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    bool var_2;
    const wp::int32 var_3 = 0;
    wp::float32 var_4;
    const wp::int32 var_5 = 0;
    bool var_6;
    const wp::int32 var_7 = 1;
    wp::float32 var_8;
    const wp::int32 var_9 = 0;
    bool var_10;
    const wp::int32 var_11 = 2;
    wp::float32 var_12;
    const wp::int32 var_13 = 0;
    bool var_14;
    const wp::float32 var_15 = 0.0;
    const wp::float32 var_16 = 0.0;
    const wp::float32 var_17 = 0.0;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::float32 var_22 = 0.0;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::vec_t<3, wp::float32> var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    //---------
    // forward
    // def box(p: wp.vec3, size: wp.vec3) -> float:                                           <L 164>
    // a = wp.abs(p) - size                                                                   <L 165>
    var_0 = wp::abs(var_p);
    var_1 = wp::sub(var_0, var_size);
    // if a[0] >= 0 or a[1] >= 0 or a[2] >= 0:                                                <L 166>
    var_4 = wp::extract(var_1, var_3);
    var_6 = (var_4 >= var_5);
    var_2 = var_6;
    if (!var_2) {
        var_8 = wp::extract(var_1, var_7);
        var_10 = (var_8 >= var_9);
        var_2 = var_2 || var_10;
    }
    if (!var_2) {
        var_12 = wp::extract(var_1, var_11);
        var_14 = (var_12 >= var_13);
        var_2 = var_2 || var_14;
    }
    if (var_2) {
        // z = wp.vec3(0.0, 0.0, 0.0)                                                         <L 167>
        var_18 = wp::vec_t<3, wp::float32>(var_15, var_16, var_17);
        // b = wp.max(a, z)                                                                   <L 168>
        var_19 = wp::max(var_1, var_18);
        // return wp.norm_l2(b) + wp.min(wp.max(a), 0.0)                                      <L 169>
        var_20 = norm_l2_0(var_19);
        var_21 = wp::max(var_1);
        var_23 = wp::min(var_21, var_22);
        var_24 = wp::add(var_20, var_23);
        return var_24;
    }
    // b = radial_field(a, p, size)                                                           <L 170>
    var_25 = radial_field_0(var_1, var_p, var_size);
    // t = -wp.cw_div(a, wp.abs(b))                                                           <L 171>
    var_26 = wp::abs(var_25);
    var_27 = wp::cw_div(var_1, var_26);
    var_28 = wp::neg(var_27);
    // return -wp.min(t) * wp.norm_l2(b)                                                      <L 172>
    var_29 = wp::min(var_28);
    var_30 = wp::neg(var_29);
    var_31 = norm_l2_0(var_25);
    var_32 = wp::mul(var_30, var_31);
    return var_32;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:175
static CUDA_CALLABLE wp::float32 ellipsoid_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
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
    const wp::int32 var_10 = 2;
    wp::float32 var_11;
    const wp::int32 var_12 = 2;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 0;
    wp::float32 var_18;
    const wp::int32 var_19 = 0;
    wp::float32 var_20;
    const wp::float32 var_21 = 2.0;
    wp::float32 var_22;
    wp::float32 var_23;
    const wp::int32 var_24 = 1;
    wp::float32 var_25;
    const wp::int32 var_26 = 1;
    wp::float32 var_27;
    const wp::float32 var_28 = 2.0;
    wp::float32 var_29;
    wp::float32 var_30;
    const wp::int32 var_31 = 2;
    wp::float32 var_32;
    const wp::int32 var_33 = 2;
    wp::float32 var_34;
    const wp::float32 var_35 = 2.0;
    wp::float32 var_36;
    wp::float32 var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::float32 var_39;
    const wp::float32 var_40 = 0.0;
    bool var_41;
    wp::float32 var_42;
    const wp::float32 var_43 = 1e-12;
    wp::float32 var_44;
    const wp::float32 var_45 = 1.0;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    //---------
    // forward
    // def ellipsoid(p: wp.vec3, size: wp.vec3) -> float:                                     <L 176>
    // scaled_p = wp.vec3(p[0] / size[0], p[1] / size[1], p[2] / size[2])                     <L 177>
    var_1 = wp::extract(var_p, var_0);
    var_3 = wp::extract(var_size, var_2);
    var_4 = wp::div(var_1, var_3);
    var_6 = wp::extract(var_p, var_5);
    var_8 = wp::extract(var_size, var_7);
    var_9 = wp::div(var_6, var_8);
    var_11 = wp::extract(var_p, var_10);
    var_13 = wp::extract(var_size, var_12);
    var_14 = wp::div(var_11, var_13);
    var_15 = wp::vec_t<3, wp::float32>(var_4, var_9, var_14);
    // k0 = wp.length(scaled_p)                                                               <L 178>
    var_16 = wp::length(var_15);
    // k1 = wp.length(wp.vec3(p[0] / (size[0] ** 2.0), p[1] / (size[1] ** 2.0), p[2] / (size[2] ** 2.0)))       <L 179>
    var_18 = wp::extract(var_p, var_17);
    var_20 = wp::extract(var_size, var_19);
    var_22 = wp::pow(var_20, var_21);
    var_23 = wp::div(var_18, var_22);
    var_25 = wp::extract(var_p, var_24);
    var_27 = wp::extract(var_size, var_26);
    var_29 = wp::pow(var_27, var_28);
    var_30 = wp::div(var_25, var_29);
    var_32 = wp::extract(var_p, var_31);
    var_34 = wp::extract(var_size, var_33);
    var_36 = wp::pow(var_34, var_35);
    var_37 = wp::div(var_32, var_36);
    var_38 = wp::vec_t<3, wp::float32>(var_23, var_30, var_37);
    var_39 = wp::length(var_38);
    // if k1 != 0.0:                                                                          <L 180>
    var_41 = (var_39 != var_40);
    if (var_41) {
        // denom = k1                                                                         <L 181>
        var_42 = wp::copy(var_39);
    }
    if (!var_41) {
        // denom = 1e-12                                                                      <L 183>
    }
    var_44 = wp::where(var_41, var_42, var_43);
    // return k0 * (k0 - 1.0) / denom                                                         <L 184>
    var_46 = wp::sub(var_16, var_45);
    var_47 = wp::mul(var_16, var_46);
    var_48 = wp::div(var_47, var_44);
    return var_48;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:109
static CUDA_CALLABLE void _ray_quad_0(
    wp::float32 var_a,
    wp::float32 var_b,
    wp::float32 var_c,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    const wp::float32 var_3 = 1e-15;
    bool var_4;
    const wp::float32 var_5 = -1.0;
    const wp::float32 var_6 = -1.0;
    const wp::float32 var_7 = -1.0;
    wp::vec_t<2, wp::float32> var_8;
    wp::float32 var_9;
    const wp::float32 var_10 = 1.0;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    wp::vec_t<2, wp::float32> var_18;
    const wp::float32 var_19 = 0.0;
    bool var_20;
    const wp::float32 var_21 = 0.0;
    bool var_22;
    const wp::float32 var_23 = -1.0;
    //---------
    // forward
    // def _ray_quad(a: float, b: float, c: float) -> Tuple[float, wp.vec2]:                  <L 110>
    // det = b * b - a * c                                                                    <L 112>
    var_0 = wp::mul(var_b, var_b);
    var_1 = wp::mul(var_a, var_c);
    var_2 = wp::sub(var_0, var_1);
    // if det < MJ_MINVAL:                                                                    <L 113>
    var_4 = (var_2 < var_3);
    if (var_4) {
        // return -1.0, wp.vec2(-1.0, -1.0)                                                   <L 114>
        var_8 = wp::vec_t<2, wp::float32>(var_6, var_7);
        ret_0 = var_5;
        ret_1 = var_8;
        return;
    }
    // det = wp.sqrt(det)                                                                     <L 115>
    var_9 = wp::sqrt(var_2);
    // den = safe_div(1.0, a)                                                                 <L 118>
    var_11 = safe_div_0(var_10, var_a);
    // x0 = (-b - det) * den                                                                  <L 119>
    var_12 = wp::neg(var_b);
    var_13 = wp::sub(var_12, var_9);
    var_14 = wp::mul(var_13, var_11);
    // x1 = (-b + det) * den                                                                  <L 120>
    var_15 = wp::neg(var_b);
    var_16 = wp::add(var_15, var_9);
    var_17 = wp::mul(var_16, var_11);
    // x = wp.vec2(x0, x1)                                                                    <L 121>
    var_18 = wp::vec_t<2, wp::float32>(var_14, var_17);
    // if x0 >= 0.0:                                                                          <L 124>
    var_20 = (var_14 >= var_19);
    if (var_20) {
        // return x0, x                                                                       <L 125>
        ret_0 = var_14;
        ret_1 = var_18;
        return;
    }
    if (!var_20) {
        // elif x1 >= 0.0:                                                                    <L 126>
        var_22 = (var_17 >= var_21);
        if (var_22) {
            // return x1, x                                                                   <L 127>
            ret_0 = var_17;
            ret_1 = var_18;
            return;
        }
        if (!var_22) {
            // return -1.0, x                                                                 <L 129>
            ret_0 = var_23;
            ret_1 = var_18;
            return;
        }
    }
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:241
static CUDA_CALLABLE void ray_sphere_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::float32 var_dist_sqr,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::float32 var_1;
    wp::float32 var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::vec_t<2, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    const wp::int32 var_8 = 0;
    bool var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::vec_t<3, wp::float32> var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    //---------
    // forward
    // def ray_sphere(pos: wp.vec3, dist_sqr: float, pnt: wp.vec3, vec: wp.vec3) -> Tuple[float, wp.vec3]:       <L 242>
    // dif = pnt - pos                                                                        <L 244>
    var_0 = wp::sub(var_pnt, var_pos);
    // a = wp.dot(vec, vec)                                                                   <L 246>
    var_1 = wp::dot(var_vec, var_vec);
    // b = wp.dot(vec, dif)                                                                   <L 247>
    var_2 = wp::dot(var_vec, var_0);
    // c = wp.dot(dif, dif) - dist_sqr                                                        <L 248>
    var_3 = wp::dot(var_0, var_0);
    var_4 = wp::sub(var_3, var_dist_sqr);
    // sol, _ = _ray_quad(a, b, c)                                                            <L 250>
    _ray_quad_0(var_1, var_2, var_4, var_5, var_6);
    // normal = wp.vec3()                                                                     <L 251>
    var_7 = wp::vec_t<3, wp::float32>();
    // if sol >= 0:                                                                           <L 252>
    var_9 = (var_5 >= var_8);
    if (var_9) {
        // s = pnt + vec * sol                                                                <L 253>
        var_10 = wp::mul(var_vec, var_5);
        var_11 = wp::add(var_pnt, var_10);
        // normal = wp.normalize(s - pos)                                                     <L 254>
        var_12 = wp::sub(var_11, var_pos);
        var_13 = wp::normalize(var_12);
    }
    var_14 = wp::where(var_9, var_13, var_7);
    // return sol, normal                                                                     <L 255>
    ret_0 = var_5;
    ret_1 = var_14;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:36
static CUDA_CALLABLE void _ray_map_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::mat_t<3, 3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::vec_t<3, wp::float32> var_3;
    //---------
    // forward
    // def _ray_map(pos: wp.vec3, mat: wp.mat33, pnt: wp.vec3, vec: wp.vec3) -> Tuple[wp.vec3, wp.vec3]:       <L 37>
    // matT = wp.transpose(mat)                                                               <L 49>
    var_0 = wp::transpose(var_mat);
    // lpnt = matT @ (pnt - pos)                                                              <L 50>
    var_1 = wp::sub(var_pnt, var_pos);
    var_2 = wp::mul(var_0, var_1);
    // lvec = matT @ vec                                                                      <L 51>
    var_3 = wp::mul(var_0, var_vec);
    // return lpnt, lvec                                                                      <L 53>
    ret_0 = var_2;
    ret_1 = var_3;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:424
static CUDA_CALLABLE void ray_box_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<6, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    const wp::float32 var_0 = -1.0;
    const wp::float32 var_1 = -1.0;
    const wp::float32 var_2 = -1.0;
    const wp::float32 var_3 = -1.0;
    const wp::float32 var_4 = -1.0;
    const wp::float32 var_5 = -1.0;
    wp::vec_t<6, wp::float32> var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    wp::vec_t<3, wp::float32> var_9;
    const wp::int32 var_10 = 0;
    bool var_11;
    const wp::float32 var_12 = -1.0;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<3, wp::float32> var_15;
    const wp::float32 var_16 = -1.0;
    wp::float32 var_17;
    const wp::int32 var_18 = -1;
    const wp::int32 var_19 = -1;
    const wp::int32 var_20 = 0;
    wp::float32 var_21;
    wp::float32 var_22;
    const wp::float32 var_23 = 1e-15;
    bool var_24;
    const wp::int32 var_25 = -1;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::float32 var_32;
    const wp::float32 var_33 = 0.0;
    bool var_34;
    const wp::int32 var_35 = 0;
    bool var_36;
    const wp::int32 var_37 = 1;
    const wp::int32 var_38 = 0;
    wp::int32 var_39;
    const wp::int32 var_40 = 2;
    bool var_41;
    const wp::int32 var_42 = 1;
    const wp::int32 var_43 = 2;
    wp::int32 var_44;
    wp::float32 var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    wp::float32 var_51;
    wp::float32 var_52;
    bool var_53;
    wp::float32 var_54;
    wp::float32 var_55;
    bool var_56;
    wp::float32 var_57;
    wp::float32 var_58;
    bool var_59;
    bool var_60;
    const wp::float32 var_61 = 0.0;
    bool var_62;
    bool var_63;
    wp::float32 var_64;
    wp::int32 var_65;
    wp::int32 var_66;
    wp::float32 var_67;
    wp::int32 var_68;
    wp::int32 var_69;
    const wp::int32 var_70 = 2;
    wp::int32 var_71;
    const wp::int32 var_72 = 1;
    wp::int32 var_73;
    const wp::int32 var_74 = 2;
    wp::int32 var_75;
    wp::int32 var_76;
    wp::float32 var_77;
    wp::int32 var_78;
    wp::int32 var_79;
    wp::float32 var_80;
    wp::int32 var_81;
    wp::int32 var_82;
    const wp::int32 var_83 = 1;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::float32 var_86;
    wp::float32 var_87;
    wp::float32 var_88;
    wp::float32 var_89;
    wp::float32 var_90;
    const wp::float32 var_91 = 0.0;
    bool var_92;
    const wp::int32 var_93 = 0;
    bool var_94;
    const wp::int32 var_95 = 1;
    const wp::int32 var_96 = 0;
    wp::int32 var_97;
    const wp::int32 var_98 = 2;
    bool var_99;
    const wp::int32 var_100 = 1;
    const wp::int32 var_101 = 2;
    wp::int32 var_102;
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
    wp::float32 var_113;
    bool var_114;
    wp::float32 var_115;
    wp::float32 var_116;
    bool var_117;
    bool var_118;
    const wp::float32 var_119 = 0.0;
    bool var_120;
    bool var_121;
    wp::float32 var_122;
    wp::int32 var_123;
    wp::int32 var_124;
    wp::float32 var_125;
    wp::int32 var_126;
    wp::int32 var_127;
    const wp::int32 var_128 = 2;
    wp::int32 var_129;
    const wp::int32 var_130 = 1;
    wp::int32 var_131;
    const wp::int32 var_132 = 2;
    wp::int32 var_133;
    wp::int32 var_134;
    wp::float32 var_135;
    wp::int32 var_136;
    wp::int32 var_137;
    wp::float32 var_138;
    wp::int32 var_139;
    wp::int32 var_140;
    wp::int32 var_141;
    wp::int32 var_142;
    wp::float32 var_143;
    wp::float32 var_144;
    wp::float32 var_145;
    wp::int32 var_146;
    wp::int32 var_147;
    const wp::int32 var_148 = 1;
    wp::float32 var_149;
    wp::float32 var_150;
    bool var_151;
    const wp::int32 var_152 = -1;
    wp::float32 var_153;
    wp::float32 var_154;
    wp::float32 var_155;
    wp::float32 var_156;
    wp::float32 var_157;
    wp::float32 var_158;
    wp::float32 var_159;
    const wp::float32 var_160 = 0.0;
    bool var_161;
    const wp::int32 var_162 = 0;
    bool var_163;
    const wp::int32 var_164 = 1;
    const wp::int32 var_165 = 0;
    wp::int32 var_166;
    const wp::int32 var_167 = 2;
    bool var_168;
    const wp::int32 var_169 = 1;
    const wp::int32 var_170 = 2;
    wp::int32 var_171;
    wp::float32 var_172;
    wp::float32 var_173;
    wp::float32 var_174;
    wp::float32 var_175;
    wp::float32 var_176;
    wp::float32 var_177;
    wp::float32 var_178;
    wp::float32 var_179;
    bool var_180;
    wp::float32 var_181;
    wp::float32 var_182;
    bool var_183;
    wp::float32 var_184;
    wp::float32 var_185;
    bool var_186;
    bool var_187;
    const wp::float32 var_188 = 0.0;
    bool var_189;
    bool var_190;
    wp::float32 var_191;
    wp::int32 var_192;
    wp::int32 var_193;
    wp::float32 var_194;
    wp::int32 var_195;
    wp::int32 var_196;
    const wp::int32 var_197 = 2;
    wp::int32 var_198;
    const wp::int32 var_199 = 1;
    wp::int32 var_200;
    const wp::int32 var_201 = 2;
    wp::int32 var_202;
    wp::int32 var_203;
    wp::float32 var_204;
    wp::int32 var_205;
    wp::int32 var_206;
    wp::float32 var_207;
    wp::int32 var_208;
    wp::int32 var_209;
    wp::int32 var_210;
    wp::int32 var_211;
    wp::float32 var_212;
    wp::float32 var_213;
    const wp::int32 var_214 = 1;
    wp::float32 var_215;
    wp::float32 var_216;
    wp::float32 var_217;
    wp::float32 var_218;
    wp::float32 var_219;
    wp::float32 var_220;
    wp::float32 var_221;
    const wp::float32 var_222 = 0.0;
    bool var_223;
    const wp::int32 var_224 = 0;
    bool var_225;
    const wp::int32 var_226 = 1;
    const wp::int32 var_227 = 0;
    wp::int32 var_228;
    const wp::int32 var_229 = 2;
    bool var_230;
    const wp::int32 var_231 = 1;
    const wp::int32 var_232 = 2;
    wp::int32 var_233;
    wp::float32 var_234;
    wp::float32 var_235;
    wp::float32 var_236;
    wp::float32 var_237;
    wp::float32 var_238;
    wp::float32 var_239;
    wp::float32 var_240;
    wp::float32 var_241;
    bool var_242;
    wp::float32 var_243;
    wp::float32 var_244;
    bool var_245;
    wp::float32 var_246;
    wp::float32 var_247;
    bool var_248;
    bool var_249;
    const wp::float32 var_250 = 0.0;
    bool var_251;
    bool var_252;
    wp::float32 var_253;
    wp::int32 var_254;
    wp::int32 var_255;
    wp::float32 var_256;
    wp::int32 var_257;
    wp::int32 var_258;
    const wp::int32 var_259 = 2;
    wp::int32 var_260;
    const wp::int32 var_261 = 1;
    wp::int32 var_262;
    const wp::int32 var_263 = 2;
    wp::int32 var_264;
    wp::int32 var_265;
    wp::float32 var_266;
    wp::int32 var_267;
    wp::int32 var_268;
    wp::float32 var_269;
    wp::int32 var_270;
    wp::int32 var_271;
    wp::int32 var_272;
    wp::int32 var_273;
    wp::float32 var_274;
    wp::float32 var_275;
    wp::float32 var_276;
    wp::int32 var_277;
    wp::int32 var_278;
    wp::int32 var_279;
    wp::float32 var_280;
    wp::int32 var_281;
    wp::int32 var_282;
    wp::float32 var_283;
    wp::float32 var_284;
    const wp::int32 var_285 = 2;
    wp::float32 var_286;
    wp::float32 var_287;
    bool var_288;
    const wp::int32 var_289 = -1;
    wp::float32 var_290;
    wp::float32 var_291;
    wp::float32 var_292;
    wp::float32 var_293;
    wp::float32 var_294;
    wp::float32 var_295;
    wp::float32 var_296;
    const wp::float32 var_297 = 0.0;
    bool var_298;
    const wp::int32 var_299 = 0;
    bool var_300;
    const wp::int32 var_301 = 1;
    const wp::int32 var_302 = 0;
    wp::int32 var_303;
    const wp::int32 var_304 = 2;
    bool var_305;
    const wp::int32 var_306 = 1;
    const wp::int32 var_307 = 2;
    wp::int32 var_308;
    wp::float32 var_309;
    wp::float32 var_310;
    wp::float32 var_311;
    wp::float32 var_312;
    wp::float32 var_313;
    wp::float32 var_314;
    wp::float32 var_315;
    wp::float32 var_316;
    bool var_317;
    wp::float32 var_318;
    wp::float32 var_319;
    bool var_320;
    wp::float32 var_321;
    wp::float32 var_322;
    bool var_323;
    bool var_324;
    const wp::float32 var_325 = 0.0;
    bool var_326;
    bool var_327;
    wp::float32 var_328;
    wp::int32 var_329;
    wp::int32 var_330;
    wp::float32 var_331;
    wp::int32 var_332;
    wp::int32 var_333;
    const wp::int32 var_334 = 2;
    wp::int32 var_335;
    const wp::int32 var_336 = 1;
    wp::int32 var_337;
    const wp::int32 var_338 = 2;
    wp::int32 var_339;
    wp::int32 var_340;
    wp::float32 var_341;
    wp::int32 var_342;
    wp::int32 var_343;
    wp::float32 var_344;
    wp::int32 var_345;
    wp::int32 var_346;
    wp::int32 var_347;
    wp::int32 var_348;
    wp::float32 var_349;
    wp::float32 var_350;
    const wp::int32 var_351 = 1;
    wp::float32 var_352;
    wp::float32 var_353;
    wp::float32 var_354;
    wp::float32 var_355;
    wp::float32 var_356;
    wp::float32 var_357;
    wp::float32 var_358;
    const wp::float32 var_359 = 0.0;
    bool var_360;
    const wp::int32 var_361 = 0;
    bool var_362;
    const wp::int32 var_363 = 1;
    const wp::int32 var_364 = 0;
    wp::int32 var_365;
    const wp::int32 var_366 = 2;
    bool var_367;
    const wp::int32 var_368 = 1;
    const wp::int32 var_369 = 2;
    wp::int32 var_370;
    wp::float32 var_371;
    wp::float32 var_372;
    wp::float32 var_373;
    wp::float32 var_374;
    wp::float32 var_375;
    wp::float32 var_376;
    wp::float32 var_377;
    wp::float32 var_378;
    bool var_379;
    wp::float32 var_380;
    wp::float32 var_381;
    bool var_382;
    wp::float32 var_383;
    wp::float32 var_384;
    bool var_385;
    bool var_386;
    const wp::float32 var_387 = 0.0;
    bool var_388;
    bool var_389;
    wp::float32 var_390;
    wp::int32 var_391;
    wp::int32 var_392;
    wp::float32 var_393;
    wp::int32 var_394;
    wp::int32 var_395;
    const wp::int32 var_396 = 2;
    wp::int32 var_397;
    const wp::int32 var_398 = 1;
    wp::int32 var_399;
    const wp::int32 var_400 = 2;
    wp::int32 var_401;
    wp::int32 var_402;
    wp::float32 var_403;
    wp::int32 var_404;
    wp::int32 var_405;
    wp::float32 var_406;
    wp::int32 var_407;
    wp::int32 var_408;
    wp::int32 var_409;
    wp::int32 var_410;
    wp::float32 var_411;
    wp::float32 var_412;
    wp::float32 var_413;
    wp::int32 var_414;
    wp::int32 var_415;
    wp::int32 var_416;
    wp::float32 var_417;
    wp::int32 var_418;
    wp::int32 var_419;
    wp::float32 var_420;
    wp::float32 var_421;
    wp::vec_t<3, wp::float32> var_422;
    const wp::int32 var_423 = 0;
    bool var_424;
    wp::float32 var_425;
    wp::vec_t<3, wp::float32> var_426;
    wp::vec_t<3, wp::float32> var_427;
    //---------
    // forward
    // def ray_box(pos: wp.vec3, mat: wp.mat33, size: wp.vec3, pnt: wp.vec3, vec: wp.vec3) -> Tuple[float, vec6, wp.vec3]:       <L 425>
    // all = vec6(-1.0, -1.0, -1.0, -1.0, -1.0, -1.0)                                         <L 427>
    var_6 = wp::vec_t<6, wp::float32>({var_0, var_1, var_2, var_3, var_4, var_5});
    // ssz = wp.dot(size, size)                                                               <L 430>
    var_7 = wp::dot(var_size, var_size);
    // dist_sphere, _ = ray_sphere(pos, ssz, pnt, vec)                                        <L 431>
    ray_sphere_0(var_pos, var_7, var_pnt, var_vec, var_8, var_9);
    // if dist_sphere < 0:                                                                    <L 432>
    var_11 = (var_8 < var_10);
    if (var_11) {
        // return -1.0, all, wp.vec3()                                                        <L 433>
        var_13 = wp::vec_t<3, wp::float32>();
        ret_0 = var_12;
        ret_1 = var_6;
        ret_2 = var_13;
        return;
    }
    // lpnt, lvec = _ray_map(pos, mat, pnt, vec)                                              <L 436>
    _ray_map_0(var_pos, var_mat, var_pnt, var_vec, var_14, var_15);
    // x = float(-1.0)                                                                        <L 439>
    var_17 = wp::float(var_16);
    // face_side = -1                                                                         <L 440>
    // face_axis = -1                                                                         <L 441>
    // for i in range(3):                                                                     <L 444>
    // if wp.abs(lvec[i]) > MJ_MINVAL:                                                        <L 445>
    var_21 = wp::extract(var_15, var_20);
    var_22 = wp::abs(var_21);
    var_24 = (var_22 > var_23);
    if (var_24) {
        // for side in range(-1, 2, 2):                                                       <L 446>
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_26 = wp::float(var_25);
        var_27 = wp::extract(var_size, var_20);
        var_28 = wp::mul(var_26, var_27);
        var_29 = wp::extract(var_14, var_20);
        var_30 = wp::sub(var_28, var_29);
        var_31 = wp::extract(var_15, var_20);
        var_32 = wp::div(var_30, var_31);
        // if sol >= 0.0:                                                                     <L 451>
        var_34 = (var_32 >= var_33);
        if (var_34) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_36 = (var_20 == var_35);
            if (var_36) {
            }
            if (!var_36) {
            }
            var_39 = wp::where(var_36, var_37, var_38);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_41 = (var_20 == var_40);
            if (var_41) {
            }
            if (!var_41) {
            }
            var_44 = wp::where(var_41, var_42, var_43);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_45 = wp::extract(var_14, var_39);
            var_46 = wp::extract(var_15, var_39);
            var_47 = wp::mul(var_32, var_46);
            var_48 = wp::add(var_45, var_47);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_49 = wp::extract(var_14, var_44);
            var_50 = wp::extract(var_15, var_44);
            var_51 = wp::mul(var_32, var_50);
            var_52 = wp::add(var_49, var_51);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_54 = wp::abs(var_48);
            var_55 = wp::extract(var_size, var_39);
            var_56 = (var_54 <= var_55);
            var_53 = var_56;
            if (var_53) {
                var_57 = wp::abs(var_52);
                var_58 = wp::extract(var_size, var_44);
                var_59 = (var_57 <= var_58);
                var_53 = var_53 && var_59;
            }
            if (var_53) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_62 = (var_17 < var_61);
                var_60 = var_62;
                if (!var_60) {
                    var_63 = (var_32 < var_17);
                    var_60 = var_60 || var_63;
                }
                if (var_60) {
                    // x = sol                                                                <L 463>
                    var_64 = wp::copy(var_32);
                    // face_axis = i                                                          <L 464>
                    var_65 = wp::copy(var_20);
                    // face_side = side                                                       <L 465>
                    var_66 = wp::copy(var_25);
                }
                var_67 = wp::where(var_60, var_64, var_17);
                var_68 = wp::where(var_60, var_66, var_18);
                var_69 = wp::where(var_60, var_65, var_19);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_71 = wp::mul(var_70, var_20);
                var_73 = wp::add(var_25, var_72);
                var_75 = wp::floordiv(var_73, var_74);
                var_76 = wp::add(var_71, var_75);
                wp::assign_inplace(var_6, var_76, var_32);
            }
            var_77 = wp::where(var_53, var_67, var_17);
            var_78 = wp::where(var_53, var_68, var_18);
            var_79 = wp::where(var_53, var_69, var_19);
        }
        var_80 = wp::where(var_34, var_77, var_17);
        var_81 = wp::where(var_34, var_78, var_18);
        var_82 = wp::where(var_34, var_79, var_19);
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_84 = wp::float(var_83);
        var_85 = wp::extract(var_size, var_20);
        var_86 = wp::mul(var_84, var_85);
        var_87 = wp::extract(var_14, var_20);
        var_88 = wp::sub(var_86, var_87);
        var_89 = wp::extract(var_15, var_20);
        var_90 = wp::div(var_88, var_89);
        // if sol >= 0.0:                                                                     <L 451>
        var_92 = (var_90 >= var_91);
        if (var_92) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_94 = (var_20 == var_93);
            if (var_94) {
            }
            if (!var_94) {
            }
            var_97 = wp::where(var_94, var_95, var_96);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_99 = (var_20 == var_98);
            if (var_99) {
            }
            if (!var_99) {
            }
            var_102 = wp::where(var_99, var_100, var_101);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_103 = wp::extract(var_14, var_97);
            var_104 = wp::extract(var_15, var_97);
            var_105 = wp::mul(var_90, var_104);
            var_106 = wp::add(var_103, var_105);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_107 = wp::extract(var_14, var_102);
            var_108 = wp::extract(var_15, var_102);
            var_109 = wp::mul(var_90, var_108);
            var_110 = wp::add(var_107, var_109);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_112 = wp::abs(var_106);
            var_113 = wp::extract(var_size, var_97);
            var_114 = (var_112 <= var_113);
            var_111 = var_114;
            if (var_111) {
                var_115 = wp::abs(var_110);
                var_116 = wp::extract(var_size, var_102);
                var_117 = (var_115 <= var_116);
                var_111 = var_111 && var_117;
            }
            if (var_111) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_120 = (var_80 < var_119);
                var_118 = var_120;
                if (!var_118) {
                    var_121 = (var_90 < var_80);
                    var_118 = var_118 || var_121;
                }
                if (var_118) {
                    // x = sol                                                                <L 463>
                    var_122 = wp::copy(var_90);
                    // face_axis = i                                                          <L 464>
                    var_123 = wp::copy(var_20);
                    // face_side = side                                                       <L 465>
                    var_124 = wp::copy(var_83);
                }
                var_125 = wp::where(var_118, var_122, var_80);
                var_126 = wp::where(var_118, var_124, var_81);
                var_127 = wp::where(var_118, var_123, var_82);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_129 = wp::mul(var_128, var_20);
                var_131 = wp::add(var_83, var_130);
                var_133 = wp::floordiv(var_131, var_132);
                var_134 = wp::add(var_129, var_133);
                wp::assign_inplace(var_6, var_134, var_90);
            }
            var_135 = wp::where(var_111, var_125, var_80);
            var_136 = wp::where(var_111, var_126, var_81);
            var_137 = wp::where(var_111, var_127, var_82);
        }
        var_138 = wp::where(var_92, var_135, var_80);
        var_139 = wp::where(var_92, var_136, var_81);
        var_140 = wp::where(var_92, var_137, var_82);
        var_141 = wp::where(var_92, var_97, var_39);
        var_142 = wp::where(var_92, var_102, var_44);
        var_143 = wp::where(var_92, var_106, var_48);
        var_144 = wp::where(var_92, var_110, var_52);
    }
    var_145 = wp::where(var_24, var_138, var_17);
    var_146 = wp::where(var_24, var_139, var_18);
    var_147 = wp::where(var_24, var_140, var_19);
    // if wp.abs(lvec[i]) > MJ_MINVAL:                                                        <L 445>
    var_149 = wp::extract(var_15, var_148);
    var_150 = wp::abs(var_149);
    var_151 = (var_150 > var_23);
    if (var_151) {
        // for side in range(-1, 2, 2):                                                       <L 446>
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_153 = wp::float(var_152);
        var_154 = wp::extract(var_size, var_148);
        var_155 = wp::mul(var_153, var_154);
        var_156 = wp::extract(var_14, var_148);
        var_157 = wp::sub(var_155, var_156);
        var_158 = wp::extract(var_15, var_148);
        var_159 = wp::div(var_157, var_158);
        // if sol >= 0.0:                                                                     <L 451>
        var_161 = (var_159 >= var_160);
        if (var_161) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_163 = (var_148 == var_162);
            if (var_163) {
            }
            if (!var_163) {
            }
            var_166 = wp::where(var_163, var_164, var_165);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_168 = (var_148 == var_167);
            if (var_168) {
            }
            if (!var_168) {
            }
            var_171 = wp::where(var_168, var_169, var_170);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_172 = wp::extract(var_14, var_166);
            var_173 = wp::extract(var_15, var_166);
            var_174 = wp::mul(var_159, var_173);
            var_175 = wp::add(var_172, var_174);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_176 = wp::extract(var_14, var_171);
            var_177 = wp::extract(var_15, var_171);
            var_178 = wp::mul(var_159, var_177);
            var_179 = wp::add(var_176, var_178);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_181 = wp::abs(var_175);
            var_182 = wp::extract(var_size, var_166);
            var_183 = (var_181 <= var_182);
            var_180 = var_183;
            if (var_180) {
                var_184 = wp::abs(var_179);
                var_185 = wp::extract(var_size, var_171);
                var_186 = (var_184 <= var_185);
                var_180 = var_180 && var_186;
            }
            if (var_180) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_189 = (var_145 < var_188);
                var_187 = var_189;
                if (!var_187) {
                    var_190 = (var_159 < var_145);
                    var_187 = var_187 || var_190;
                }
                if (var_187) {
                    // x = sol                                                                <L 463>
                    var_191 = wp::copy(var_159);
                    // face_axis = i                                                          <L 464>
                    var_192 = wp::copy(var_148);
                    // face_side = side                                                       <L 465>
                    var_193 = wp::copy(var_152);
                }
                var_194 = wp::where(var_187, var_191, var_145);
                var_195 = wp::where(var_187, var_193, var_146);
                var_196 = wp::where(var_187, var_192, var_147);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_198 = wp::mul(var_197, var_148);
                var_200 = wp::add(var_152, var_199);
                var_202 = wp::floordiv(var_200, var_201);
                var_203 = wp::add(var_198, var_202);
                wp::assign_inplace(var_6, var_203, var_159);
            }
            var_204 = wp::where(var_180, var_194, var_145);
            var_205 = wp::where(var_180, var_195, var_146);
            var_206 = wp::where(var_180, var_196, var_147);
        }
        var_207 = wp::where(var_161, var_204, var_145);
        var_208 = wp::where(var_161, var_205, var_146);
        var_209 = wp::where(var_161, var_206, var_147);
        var_210 = wp::where(var_161, var_166, var_141);
        var_211 = wp::where(var_161, var_171, var_142);
        var_212 = wp::where(var_161, var_175, var_143);
        var_213 = wp::where(var_161, var_179, var_144);
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_215 = wp::float(var_214);
        var_216 = wp::extract(var_size, var_148);
        var_217 = wp::mul(var_215, var_216);
        var_218 = wp::extract(var_14, var_148);
        var_219 = wp::sub(var_217, var_218);
        var_220 = wp::extract(var_15, var_148);
        var_221 = wp::div(var_219, var_220);
        // if sol >= 0.0:                                                                     <L 451>
        var_223 = (var_221 >= var_222);
        if (var_223) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_225 = (var_148 == var_224);
            if (var_225) {
            }
            if (!var_225) {
            }
            var_228 = wp::where(var_225, var_226, var_227);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_230 = (var_148 == var_229);
            if (var_230) {
            }
            if (!var_230) {
            }
            var_233 = wp::where(var_230, var_231, var_232);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_234 = wp::extract(var_14, var_228);
            var_235 = wp::extract(var_15, var_228);
            var_236 = wp::mul(var_221, var_235);
            var_237 = wp::add(var_234, var_236);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_238 = wp::extract(var_14, var_233);
            var_239 = wp::extract(var_15, var_233);
            var_240 = wp::mul(var_221, var_239);
            var_241 = wp::add(var_238, var_240);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_243 = wp::abs(var_237);
            var_244 = wp::extract(var_size, var_228);
            var_245 = (var_243 <= var_244);
            var_242 = var_245;
            if (var_242) {
                var_246 = wp::abs(var_241);
                var_247 = wp::extract(var_size, var_233);
                var_248 = (var_246 <= var_247);
                var_242 = var_242 && var_248;
            }
            if (var_242) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_251 = (var_207 < var_250);
                var_249 = var_251;
                if (!var_249) {
                    var_252 = (var_221 < var_207);
                    var_249 = var_249 || var_252;
                }
                if (var_249) {
                    // x = sol                                                                <L 463>
                    var_253 = wp::copy(var_221);
                    // face_axis = i                                                          <L 464>
                    var_254 = wp::copy(var_148);
                    // face_side = side                                                       <L 465>
                    var_255 = wp::copy(var_214);
                }
                var_256 = wp::where(var_249, var_253, var_207);
                var_257 = wp::where(var_249, var_255, var_208);
                var_258 = wp::where(var_249, var_254, var_209);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_260 = wp::mul(var_259, var_148);
                var_262 = wp::add(var_214, var_261);
                var_264 = wp::floordiv(var_262, var_263);
                var_265 = wp::add(var_260, var_264);
                wp::assign_inplace(var_6, var_265, var_221);
            }
            var_266 = wp::where(var_242, var_256, var_207);
            var_267 = wp::where(var_242, var_257, var_208);
            var_268 = wp::where(var_242, var_258, var_209);
        }
        var_269 = wp::where(var_223, var_266, var_207);
        var_270 = wp::where(var_223, var_267, var_208);
        var_271 = wp::where(var_223, var_268, var_209);
        var_272 = wp::where(var_223, var_228, var_210);
        var_273 = wp::where(var_223, var_233, var_211);
        var_274 = wp::where(var_223, var_237, var_212);
        var_275 = wp::where(var_223, var_241, var_213);
    }
    var_276 = wp::where(var_151, var_269, var_145);
    var_277 = wp::where(var_151, var_270, var_146);
    var_278 = wp::where(var_151, var_271, var_147);
    var_279 = wp::where(var_151, var_214, var_83);
    var_280 = wp::where(var_151, var_221, var_90);
    var_281 = wp::where(var_151, var_272, var_141);
    var_282 = wp::where(var_151, var_273, var_142);
    var_283 = wp::where(var_151, var_274, var_143);
    var_284 = wp::where(var_151, var_275, var_144);
    // if wp.abs(lvec[i]) > MJ_MINVAL:                                                        <L 445>
    var_286 = wp::extract(var_15, var_285);
    var_287 = wp::abs(var_286);
    var_288 = (var_287 > var_23);
    if (var_288) {
        // for side in range(-1, 2, 2):                                                       <L 446>
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_290 = wp::float(var_289);
        var_291 = wp::extract(var_size, var_285);
        var_292 = wp::mul(var_290, var_291);
        var_293 = wp::extract(var_14, var_285);
        var_294 = wp::sub(var_292, var_293);
        var_295 = wp::extract(var_15, var_285);
        var_296 = wp::div(var_294, var_295);
        // if sol >= 0.0:                                                                     <L 451>
        var_298 = (var_296 >= var_297);
        if (var_298) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_300 = (var_285 == var_299);
            if (var_300) {
            }
            if (!var_300) {
            }
            var_303 = wp::where(var_300, var_301, var_302);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_305 = (var_285 == var_304);
            if (var_305) {
            }
            if (!var_305) {
            }
            var_308 = wp::where(var_305, var_306, var_307);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_309 = wp::extract(var_14, var_303);
            var_310 = wp::extract(var_15, var_303);
            var_311 = wp::mul(var_296, var_310);
            var_312 = wp::add(var_309, var_311);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_313 = wp::extract(var_14, var_308);
            var_314 = wp::extract(var_15, var_308);
            var_315 = wp::mul(var_296, var_314);
            var_316 = wp::add(var_313, var_315);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_318 = wp::abs(var_312);
            var_319 = wp::extract(var_size, var_303);
            var_320 = (var_318 <= var_319);
            var_317 = var_320;
            if (var_317) {
                var_321 = wp::abs(var_316);
                var_322 = wp::extract(var_size, var_308);
                var_323 = (var_321 <= var_322);
                var_317 = var_317 && var_323;
            }
            if (var_317) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_326 = (var_276 < var_325);
                var_324 = var_326;
                if (!var_324) {
                    var_327 = (var_296 < var_276);
                    var_324 = var_324 || var_327;
                }
                if (var_324) {
                    // x = sol                                                                <L 463>
                    var_328 = wp::copy(var_296);
                    // face_axis = i                                                          <L 464>
                    var_329 = wp::copy(var_285);
                    // face_side = side                                                       <L 465>
                    var_330 = wp::copy(var_289);
                }
                var_331 = wp::where(var_324, var_328, var_276);
                var_332 = wp::where(var_324, var_330, var_277);
                var_333 = wp::where(var_324, var_329, var_278);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_335 = wp::mul(var_334, var_285);
                var_337 = wp::add(var_289, var_336);
                var_339 = wp::floordiv(var_337, var_338);
                var_340 = wp::add(var_335, var_339);
                wp::assign_inplace(var_6, var_340, var_296);
            }
            var_341 = wp::where(var_317, var_331, var_276);
            var_342 = wp::where(var_317, var_332, var_277);
            var_343 = wp::where(var_317, var_333, var_278);
        }
        var_344 = wp::where(var_298, var_341, var_276);
        var_345 = wp::where(var_298, var_342, var_277);
        var_346 = wp::where(var_298, var_343, var_278);
        var_347 = wp::where(var_298, var_303, var_281);
        var_348 = wp::where(var_298, var_308, var_282);
        var_349 = wp::where(var_298, var_312, var_283);
        var_350 = wp::where(var_298, var_316, var_284);
        // sol = (float(side) * size[i] - lpnt[i]) / lvec[i]                                  <L 448>
        var_352 = wp::float(var_351);
        var_353 = wp::extract(var_size, var_285);
        var_354 = wp::mul(var_352, var_353);
        var_355 = wp::extract(var_14, var_285);
        var_356 = wp::sub(var_354, var_355);
        var_357 = wp::extract(var_15, var_285);
        var_358 = wp::div(var_356, var_357);
        // if sol >= 0.0:                                                                     <L 451>
        var_360 = (var_358 >= var_359);
        if (var_360) {
            // id0 = 1 if i == 0 else 0                                                       <L 452>
            var_362 = (var_285 == var_361);
            if (var_362) {
            }
            if (!var_362) {
            }
            var_365 = wp::where(var_362, var_363, var_364);
            // id1 = 1 if i == 2 else 2                                                       <L 453>
            var_367 = (var_285 == var_366);
            if (var_367) {
            }
            if (!var_367) {
            }
            var_370 = wp::where(var_367, var_368, var_369);
            // p0 = lpnt[id0] + sol * lvec[id0]                                               <L 456>
            var_371 = wp::extract(var_14, var_365);
            var_372 = wp::extract(var_15, var_365);
            var_373 = wp::mul(var_358, var_372);
            var_374 = wp::add(var_371, var_373);
            // p1 = lpnt[id1] + sol * lvec[id1]                                               <L 457>
            var_375 = wp::extract(var_14, var_370);
            var_376 = wp::extract(var_15, var_370);
            var_377 = wp::mul(var_358, var_376);
            var_378 = wp::add(var_375, var_377);
            // if (wp.abs(p0) <= size[id0]) and (wp.abs(p1) <= size[id1]):                    <L 460>
            var_380 = wp::abs(var_374);
            var_381 = wp::extract(var_size, var_365);
            var_382 = (var_380 <= var_381);
            var_379 = var_382;
            if (var_379) {
                var_383 = wp::abs(var_378);
                var_384 = wp::extract(var_size, var_370);
                var_385 = (var_383 <= var_384);
                var_379 = var_379 && var_385;
            }
            if (var_379) {
                // if x < 0.0 or sol < x:                                                     <L 462>
                var_388 = (var_344 < var_387);
                var_386 = var_388;
                if (!var_386) {
                    var_389 = (var_358 < var_344);
                    var_386 = var_386 || var_389;
                }
                if (var_386) {
                    // x = sol                                                                <L 463>
                    var_390 = wp::copy(var_358);
                    // face_axis = i                                                          <L 464>
                    var_391 = wp::copy(var_285);
                    // face_side = side                                                       <L 465>
                    var_392 = wp::copy(var_351);
                }
                var_393 = wp::where(var_386, var_390, var_344);
                var_394 = wp::where(var_386, var_392, var_345);
                var_395 = wp::where(var_386, var_391, var_346);
                // all[2 * i + (side + 1) // 2] = sol                                         <L 468>
                var_397 = wp::mul(var_396, var_285);
                var_399 = wp::add(var_351, var_398);
                var_401 = wp::floordiv(var_399, var_400);
                var_402 = wp::add(var_397, var_401);
                wp::assign_inplace(var_6, var_402, var_358);
            }
            var_403 = wp::where(var_379, var_393, var_344);
            var_404 = wp::where(var_379, var_394, var_345);
            var_405 = wp::where(var_379, var_395, var_346);
        }
        var_406 = wp::where(var_360, var_403, var_344);
        var_407 = wp::where(var_360, var_404, var_345);
        var_408 = wp::where(var_360, var_405, var_346);
        var_409 = wp::where(var_360, var_365, var_347);
        var_410 = wp::where(var_360, var_370, var_348);
        var_411 = wp::where(var_360, var_374, var_349);
        var_412 = wp::where(var_360, var_378, var_350);
    }
    var_413 = wp::where(var_288, var_406, var_276);
    var_414 = wp::where(var_288, var_407, var_277);
    var_415 = wp::where(var_288, var_408, var_278);
    var_416 = wp::where(var_288, var_351, var_279);
    var_417 = wp::where(var_288, var_358, var_280);
    var_418 = wp::where(var_288, var_409, var_281);
    var_419 = wp::where(var_288, var_410, var_282);
    var_420 = wp::where(var_288, var_411, var_283);
    var_421 = wp::where(var_288, var_412, var_284);
    // normal = wp.vec3()                                                                     <L 470>
    var_422 = wp::vec_t<3, wp::float32>();
    // if x >= 0:                                                                             <L 471>
    var_424 = (var_413 >= var_423);
    if (var_424) {
        // normal[face_axis] = float(face_side)                                               <L 472>
        var_425 = wp::float(var_414);
        wp::assign_inplace(var_422, var_415, var_425);
        // normal = mat @ normal                                                              <L 473>
        var_426 = wp::mul(var_mat, var_422);
    }
    var_427 = wp::where(var_424, var_426, var_422);
    // return x, all, normal                                                                  <L 475>
    ret_0 = var_413;
    ret_1 = var_6;
    ret_2 = var_427;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:132
static CUDA_CALLABLE void _orthogonal_basis_0(
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 2;
    wp::float32 var_1;
    const wp::float32 var_2 = 0.0;
    bool var_3;
    const wp::float32 var_4 = 1.0;
    const wp::float32 var_5 = -1.0;
    wp::float32 var_6;
    const wp::float32 var_7 = -1.0;
    const wp::int32 var_8 = 2;
    wp::float32 var_9;
    wp::float32 var_10;
    wp::float32 var_11;
    const wp::int32 var_12 = 0;
    wp::float32 var_13;
    const wp::int32 var_14 = 1;
    wp::float32 var_15;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::float32 var_18 = 1.0;
    const wp::int32 var_19 = 0;
    wp::float32 var_20;
    wp::float32 var_21;
    const wp::int32 var_22 = 0;
    wp::float32 var_23;
    wp::float32 var_24;
    wp::float32 var_25;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::float32 var_28;
    const wp::int32 var_29 = 0;
    wp::float32 var_30;
    wp::float32 var_31;
    wp::vec_t<3, wp::float32> var_32;
    const wp::int32 var_33 = 1;
    wp::float32 var_34;
    const wp::int32 var_35 = 1;
    wp::float32 var_36;
    wp::float32 var_37;
    wp::float32 var_38;
    wp::float32 var_39;
    const wp::int32 var_40 = 1;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::vec_t<3, wp::float32> var_43;
    //---------
    // forward
    // def _orthogonal_basis(vec: wp.vec3) -> Tuple[wp.vec3, wp.vec3]:                        <L 133>
    // sign = wp.where(vec[2] >= 0.0, 1.0, -1.0)                                              <L 141>
    var_1 = wp::extract(var_vec, var_0);
    var_3 = (var_1 >= var_2);
    var_6 = wp::where(var_3, var_4, var_5);
    // a = -1.0 / (sign + vec[2])                                                             <L 142>
    var_9 = wp::extract(var_vec, var_8);
    var_10 = wp::add(var_6, var_9);
    var_11 = wp::div(var_7, var_10);
    // b = vec[0] * vec[1] * a                                                                <L 143>
    var_13 = wp::extract(var_vec, var_12);
    var_15 = wp::extract(var_vec, var_14);
    var_16 = wp::mul(var_13, var_15);
    var_17 = wp::mul(var_16, var_11);
    // b0 = wp.vec3(                                                                          <L 145>
    // 1.0 + sign * vec[0] * vec[0] * a,                                                      <L 146>
    var_20 = wp::extract(var_vec, var_19);
    var_21 = wp::mul(var_6, var_20);
    var_23 = wp::extract(var_vec, var_22);
    var_24 = wp::mul(var_21, var_23);
    var_25 = wp::mul(var_24, var_11);
    var_26 = wp::add(var_18, var_25);
    // sign * b,                                                                              <L 147>
    var_27 = wp::mul(var_6, var_17);
    // -sign * vec[0],                                                                        <L 148>
    var_28 = wp::neg(var_6);
    var_30 = wp::extract(var_vec, var_29);
    var_31 = wp::mul(var_28, var_30);
    var_32 = wp::vec_t<3, wp::float32>(var_26, var_27, var_31);
    // b1 = wp.vec3(                                                                          <L 150>
    // b,                                                                                     <L 151>
    // sign + vec[1] * vec[1] * a,                                                            <L 152>
    var_34 = wp::extract(var_vec, var_33);
    var_36 = wp::extract(var_vec, var_35);
    var_37 = wp::mul(var_34, var_36);
    var_38 = wp::mul(var_37, var_11);
    var_39 = wp::add(var_6, var_38);
    // -vec[1],                                                                               <L 153>
    var_41 = wp::extract(var_vec, var_40);
    var_42 = wp::neg(var_41);
    var_43 = wp::vec_t<3, wp::float32>(var_17, var_39, var_42);
    // return b0, b1                                                                          <L 155>
    ret_0 = var_32;
    ret_1 = var_43;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:158
static CUDA_CALLABLE void _ray_triangle_1(
    wp::vec_t<3, wp::float32> var_v0,
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> var_b0,
    wp::vec_t<3, wp::float32> var_b1,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    bool var_9;
    bool var_10;
    const wp::float32 var_11 = 0.0;
    bool var_12;
    const wp::float32 var_13 = 0.0;
    bool var_14;
    const wp::float32 var_15 = 0.0;
    bool var_16;
    bool var_17;
    const wp::float32 var_18 = 0.0;
    bool var_19;
    const wp::float32 var_20 = 0.0;
    bool var_21;
    const wp::float32 var_22 = 0.0;
    bool var_23;
    bool var_24;
    const wp::float32 var_25 = 0.0;
    bool var_26;
    const wp::float32 var_27 = 0.0;
    bool var_28;
    const wp::float32 var_29 = 0.0;
    bool var_30;
    bool var_31;
    const wp::float32 var_32 = 0.0;
    bool var_33;
    const wp::float32 var_34 = 0.0;
    bool var_35;
    const wp::float32 var_36 = 0.0;
    bool var_37;
    const wp::float32 var_38 = -1.0;
    wp::vec_t<3, wp::float32> var_39;
    wp::float32 var_40;
    wp::float32 var_41;
    wp::float32 var_42;
    wp::float32 var_43;
    wp::float32 var_44;
    wp::float32 var_45;
    wp::vec_t<2, wp::float32> var_46;
    wp::float32 var_47;
    wp::float32 var_48;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::float32 var_51 = 1e-15;
    bool var_52;
    const wp::float32 var_53 = -1.0;
    wp::vec_t<3, wp::float32> var_54;
    const wp::int32 var_55 = 0;
    wp::float32 var_56;
    wp::float32 var_57;
    const wp::int32 var_58 = 1;
    wp::float32 var_59;
    wp::float32 var_60;
    wp::float32 var_61;
    wp::float32 var_62;
    wp::float32 var_63;
    const wp::int32 var_64 = 0;
    wp::float32 var_65;
    wp::float32 var_66;
    const wp::int32 var_67 = 1;
    wp::float32 var_68;
    wp::float32 var_69;
    wp::float32 var_70;
    wp::float32 var_71;
    bool var_72;
    const wp::float32 var_73 = 0.0;
    bool var_74;
    const wp::float32 var_75 = 0.0;
    bool var_76;
    wp::float32 var_77;
    const wp::float32 var_78 = 1.0;
    bool var_79;
    const wp::float32 var_80 = -1.0;
    wp::vec_t<3, wp::float32> var_81;
    wp::vec_t<3, wp::float32> var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::vec_t<3, wp::float32> var_84;
    wp::vec_t<3, wp::float32> var_85;
    wp::float32 var_86;
    wp::float32 var_87;
    bool var_88;
    const wp::float32 var_89 = -1.0;
    wp::vec_t<3, wp::float32> var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    wp::float32 var_93;
    const wp::float32 var_94 = 0.0;
    bool var_95;
    const wp::float32 var_96 = -1.0;
    wp::float32 var_97;
    wp::vec_t<3, wp::float32> var_98;
    //---------
    // forward
    // def _ray_triangle(                                                                     <L 159>
    // dif0 = v0 - pnt                                                                        <L 163>
    var_0 = wp::sub(var_v0, var_pnt);
    // dif1 = v1 - pnt                                                                        <L 164>
    var_1 = wp::sub(var_v1, var_pnt);
    // dif2 = v2 - pnt                                                                        <L 165>
    var_2 = wp::sub(var_v2, var_pnt);
    // planar_00 = wp.dot(dif0, b0)                                                           <L 168>
    var_3 = wp::dot(var_0, var_b0);
    // planar_01 = wp.dot(dif0, b1)                                                           <L 169>
    var_4 = wp::dot(var_0, var_b1);
    // planar_10 = wp.dot(dif1, b0)                                                           <L 170>
    var_5 = wp::dot(var_1, var_b0);
    // planar_11 = wp.dot(dif1, b1)                                                           <L 171>
    var_6 = wp::dot(var_1, var_b1);
    // planar_20 = wp.dot(dif2, b0)                                                           <L 172>
    var_7 = wp::dot(var_2, var_b0);
    // planar_21 = wp.dot(dif2, b1)                                                           <L 173>
    var_8 = wp::dot(var_2, var_b1);
    // if (                                                                                   <L 176>
    // (planar_00 > 0.0 and planar_10 > 0.0 and planar_20 > 0.0)                              <L 177>
    var_12 = (var_3 > var_11);
    var_10 = var_12;
    if (var_10) {
        var_14 = (var_5 > var_13);
        var_10 = var_10 && var_14;
    }
    if (var_10) {
        var_16 = (var_7 > var_15);
        var_10 = var_10 && var_16;
    }
    var_9 = var_10;
    if (!var_9) {
        // or (planar_00 < 0.0 and planar_10 < 0.0 and planar_20 < 0.0)                       <L 178>
        var_19 = (var_3 < var_18);
        var_17 = var_19;
        if (var_17) {
            var_21 = (var_5 < var_20);
            var_17 = var_17 && var_21;
        }
        if (var_17) {
            var_23 = (var_7 < var_22);
            var_17 = var_17 && var_23;
        }
        var_9 = var_9 || var_17;
    }
    if (!var_9) {
        // or (planar_01 > 0.0 and planar_11 > 0.0 and planar_21 > 0.0)                       <L 179>
        var_26 = (var_4 > var_25);
        var_24 = var_26;
        if (var_24) {
            var_28 = (var_6 > var_27);
            var_24 = var_24 && var_28;
        }
        if (var_24) {
            var_30 = (var_8 > var_29);
            var_24 = var_24 && var_30;
        }
        var_9 = var_9 || var_24;
    }
    if (!var_9) {
        // or (planar_01 < 0.0 and planar_11 < 0.0 and planar_21 < 0.0)                       <L 180>
        var_33 = (var_4 < var_32);
        var_31 = var_33;
        if (var_31) {
            var_35 = (var_6 < var_34);
            var_31 = var_31 && var_35;
        }
        if (var_31) {
            var_37 = (var_8 < var_36);
            var_31 = var_31 && var_37;
        }
        var_9 = var_9 || var_31;
    }
    if (var_9) {
        // return -1.0, wp.vec3()                                                             <L 182>
        var_39 = wp::vec_t<3, wp::float32>();
        ret_0 = var_38;
        ret_1 = var_39;
        return;
    }
    // A00 = planar_00 - planar_20                                                            <L 186>
    var_40 = wp::sub(var_3, var_7);
    // A10 = planar_10 - planar_20                                                            <L 187>
    var_41 = wp::sub(var_5, var_7);
    // A01 = planar_01 - planar_21                                                            <L 188>
    var_42 = wp::sub(var_4, var_8);
    // A11 = planar_11 - planar_21                                                            <L 189>
    var_43 = wp::sub(var_6, var_8);
    // b = wp.vec2(-planar_20, -planar_21)                                                    <L 191>
    var_44 = wp::neg(var_7);
    var_45 = wp::neg(var_8);
    var_46 = wp::vec_t<2, wp::float32>(var_44, var_45);
    // det = A00 * A11 - A10 * A01                                                            <L 193>
    var_47 = wp::mul(var_40, var_43);
    var_48 = wp::mul(var_41, var_42);
    var_49 = wp::sub(var_47, var_48);
    // if wp.abs(det) < MJ_MINVAL:                                                            <L 194>
    var_50 = wp::abs(var_49);
    var_52 = (var_50 < var_51);
    if (var_52) {
        // return -1.0, wp.vec3()                                                             <L 195>
        var_54 = wp::vec_t<3, wp::float32>();
        ret_0 = var_53;
        ret_1 = var_54;
        return;
    }
    // t0 = (A11 * b[0] - A10 * b[1]) / det                                                   <L 197>
    var_56 = wp::extract(var_46, var_55);
    var_57 = wp::mul(var_43, var_56);
    var_59 = wp::extract(var_46, var_58);
    var_60 = wp::mul(var_41, var_59);
    var_61 = wp::sub(var_57, var_60);
    var_62 = wp::div(var_61, var_49);
    // t1 = (-A01 * b[0] + A00 * b[1]) / det                                                  <L 198>
    var_63 = wp::neg(var_42);
    var_65 = wp::extract(var_46, var_64);
    var_66 = wp::mul(var_63, var_65);
    var_68 = wp::extract(var_46, var_67);
    var_69 = wp::mul(var_40, var_68);
    var_70 = wp::add(var_66, var_69);
    var_71 = wp::div(var_70, var_49);
    // if t0 < 0.0 or t1 < 0.0 or t0 + t1 > 1.0:                                              <L 201>
    var_74 = (var_62 < var_73);
    var_72 = var_74;
    if (!var_72) {
        var_76 = (var_71 < var_75);
        var_72 = var_72 || var_76;
    }
    if (!var_72) {
        var_77 = wp::add(var_62, var_71);
        var_79 = (var_77 > var_78);
        var_72 = var_72 || var_79;
    }
    if (var_72) {
        // return -1.0, wp.vec3()                                                             <L 202>
        var_81 = wp::vec_t<3, wp::float32>();
        ret_0 = var_80;
        ret_1 = var_81;
        return;
    }
    // dif0 = v0 - v2                                                                         <L 205>
    var_82 = wp::sub(var_v0, var_v2);
    // dif1 = v1 - v2                                                                         <L 206>
    var_83 = wp::sub(var_v1, var_v2);
    // dif2 = pnt - v2                                                                        <L 207>
    var_84 = wp::sub(var_pnt, var_v2);
    // nrm = wp.cross(dif0, dif1)  # normal to triangle plane                                 <L 208>
    var_85 = wp::cross(var_82, var_83);
    // denom = wp.dot(vec, nrm)                                                               <L 209>
    var_86 = wp::dot(var_vec, var_85);
    // if wp.abs(denom) < MJ_MINVAL:                                                          <L 210>
    var_87 = wp::abs(var_86);
    var_88 = (var_87 < var_51);
    if (var_88) {
        // return -1.0, wp.vec3()                                                             <L 211>
        var_90 = wp::vec_t<3, wp::float32>();
        ret_0 = var_89;
        ret_1 = var_90;
        return;
    }
    // dist = -wp.dot(dif2, nrm) / denom                                                      <L 213>
    var_91 = wp::dot(var_84, var_85);
    var_92 = wp::neg(var_91);
    var_93 = wp::div(var_92, var_86);
    // return wp.where(dist >= 0.0, dist, -1.0), wp.normalize(nrm)                            <L 214>
    var_95 = (var_93 >= var_94);
    var_97 = wp::where(var_95, var_93, var_96);
    var_98 = wp::normalize(var_85);
    ret_0 = var_97;
    ret_1 = var_98;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:632
static CUDA_CALLABLE void ray_mesh_0(
    wp::int32 var_nmeshface,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_faceadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::vec_t<3, wp::int32>> var_mesh_face,
    wp::int32 var_data_id,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    wp::float32 var_0;
    wp::vec_t<6, wp::float32> var_1;
    wp::vec_t<3, wp::float32> var_2;
    const wp::float32 var_3 = 0.0;
    bool var_4;
    const wp::float32 var_5 = -1.0;
    wp::vec_t<3, wp::float32> var_6;
    wp::vec_t<3, wp::float32> var_7;
    wp::vec_t<3, wp::float32> var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    const wp::float32 var_11 = -1.0;
    wp::float32 var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::int32* var_14;
    wp::int32 var_15;
    wp::int32 var_16;
    wp::int32* var_17;
    wp::int32 var_18;
    wp::int32 var_19;
    const wp::int32 var_20 = 1;
    wp::int32 var_21;
    wp::shape_t* var_22;
    const wp::int32 var_23 = 0;
    wp::int32 var_24;
    wp::shape_t var_25;
    bool var_26;
    const wp::int32 var_27 = 1;
    wp::int32 var_28;
    wp::int32* var_29;
    wp::int32 var_30;
    wp::int32 var_31;
    wp::int32 var_32;
    wp::int32 var_33;
    wp::range_t var_34;
    wp::int32 var_35;
    wp::vec_t<3, wp::int32>* var_36;
    wp::vec_t<3, wp::int32> var_37;
    wp::vec_t<3, wp::int32> var_38;
    const wp::int32 var_39 = 0;
    wp::int32 var_40;
    wp::int32 var_41;
    wp::vec_t<3, wp::float32>* var_42;
    wp::vec_t<3, wp::float32> var_43;
    wp::vec_t<3, wp::float32> var_44;
    const wp::int32 var_45 = 1;
    wp::int32 var_46;
    wp::int32 var_47;
    wp::vec_t<3, wp::float32>* var_48;
    wp::vec_t<3, wp::float32> var_49;
    wp::vec_t<3, wp::float32> var_50;
    const wp::int32 var_51 = 2;
    wp::int32 var_52;
    wp::int32 var_53;
    wp::vec_t<3, wp::float32>* var_54;
    wp::vec_t<3, wp::float32> var_55;
    wp::vec_t<3, wp::float32> var_56;
    wp::float32 var_57;
    wp::vec_t<3, wp::float32> var_58;
    bool var_59;
    const wp::int32 var_60 = 0;
    bool var_61;
    bool var_62;
    const wp::int32 var_63 = 0;
    bool var_64;
    bool var_65;
    wp::float32 var_66;
    wp::vec_t<3, wp::float32> var_67;
    wp::float32 var_68;
    wp::vec_t<3, wp::float32> var_69;
    wp::vec_t<3, wp::float32> var_70;
    //---------
    // forward
    // def ray_mesh(                                                                          <L 633>
    // dist_box, _all, _normal = ray_box(pos, mat, size, pnt, vec)                            <L 650>
    ray_box_0(var_pos, var_mat, var_size, var_pnt, var_vec, var_0, var_1, var_2);
    // if dist_box < 0.0:                                                                     <L 651>
    var_4 = (var_0 < var_3);
    if (var_4) {
        // return -1.0, wp.vec3()                                                             <L 652>
        var_6 = wp::vec_t<3, wp::float32>();
        ret_0 = var_5;
        ret_1 = var_6;
        return;
    }
    // pnt, vec = _ray_map(pos, mat, pnt, vec)                                                <L 654>
    _ray_map_0(var_pos, var_mat, var_pnt, var_vec, var_7, var_8);
    // b0, b1 = _orthogonal_basis(vec)                                                        <L 657>
    _orthogonal_basis_0(var_8, var_9, var_10);
    // x = float(-1.0)                                                                        <L 659>
    var_12 = wp::float(var_11);
    // normal = wp.vec3()                                                                     <L 660>
    var_13 = wp::vec_t<3, wp::float32>();
    // vert_start = mesh_vertadr[data_id]                                                     <L 663>
    var_14 = wp::address(var_mesh_vertadr, var_data_id);
    var_16 = wp::load(var_14);
    var_15 = wp::copy(var_16);
    // face_start = mesh_faceadr[data_id]                                                     <L 666>
    var_17 = wp::address(var_mesh_faceadr, var_data_id);
    var_19 = wp::load(var_17);
    var_18 = wp::copy(var_19);
    // if data_id + 1 < mesh_faceadr.shape[0]:                                                <L 668>
    var_21 = wp::add(var_data_id, var_20);
    var_22 = &(var_mesh_faceadr.shape);
    var_25 = wp::load(var_22);
    var_24 = wp::extract(var_25, var_23);
    var_26 = (var_21 < var_24);
    if (var_26) {
        // face_end = mesh_faceadr[data_id + 1]                                               <L 669>
        var_28 = wp::add(var_data_id, var_27);
        var_29 = wp::address(var_mesh_faceadr, var_28);
        var_31 = wp::load(var_29);
        var_30 = wp::copy(var_31);
    }
    if (!var_26) {
        // face_end = nmeshface                                                               <L 671>
        var_32 = wp::copy(var_nmeshface);
    }
    var_33 = wp::where(var_26, var_30, var_32);
    // for i in range(face_start, face_end):                                                  <L 674>
    var_34 = wp::range(var_18, var_33);
    start_for_1:;
        if (iter_cmp(var_34) == 0) goto end_for_1;
        var_35 = wp::iter_next(var_34);
        // v_idx = mesh_face[i]                                                               <L 676>
        var_36 = wp::address(var_mesh_face, var_35);
        var_38 = wp::load(var_36);
        var_37 = wp::copy(var_38);
        // v0 = mesh_vert[vert_start + v_idx.x]                                               <L 679>
        var_40 = wp::extract(var_37, var_39);
        var_41 = wp::add(var_15, var_40);
        var_42 = wp::address(var_mesh_vert, var_41);
        var_44 = wp::load(var_42);
        var_43 = wp::copy(var_44);
        // v1 = mesh_vert[vert_start + v_idx.y]                                               <L 680>
        var_46 = wp::extract(var_37, var_45);
        var_47 = wp::add(var_15, var_46);
        var_48 = wp::address(var_mesh_vert, var_47);
        var_50 = wp::load(var_48);
        var_49 = wp::copy(var_50);
        // v2 = mesh_vert[vert_start + v_idx.z]                                               <L 681>
        var_52 = wp::extract(var_37, var_51);
        var_53 = wp::add(var_15, var_52);
        var_54 = wp::address(var_mesh_vert, var_53);
        var_56 = wp::load(var_54);
        var_55 = wp::copy(var_56);
        // dist, normal_tri = _ray_triangle(v0, v1, v2, pnt, vec, b0, b1)                     <L 684>
        _ray_triangle_1(var_43, var_49, var_55, var_7, var_8, var_9, var_10, var_57, var_58);
        // if dist >= 0 and (x < 0 or dist < x):                                              <L 685>
        var_61 = (var_57 >= var_60);
        var_59 = var_61;
        if (var_59) {
            var_64 = (var_12 < var_63);
            var_62 = var_64;
            if (!var_62) {
                var_65 = (var_57 < var_12);
                var_62 = var_62 || var_65;
            }
            var_59 = var_59 && var_62;
        }
        if (var_59) {
            // x = dist                                                                       <L 686>
            var_66 = wp::copy(var_57);
            // normal = normal_tri                                                            <L 687>
            var_67 = wp::copy(var_58);
        }
        var_68 = wp::where(var_59, var_66, var_12);
        var_69 = wp::where(var_59, var_67, var_13);
        wp::assign(var_12, var_68);
        wp::assign(var_13, var_69);
        goto start_for_1;
    end_for_1:;
    // normal = mat @ normal                                                                  <L 689>
    var_70 = wp::mul(var_mat, var_13);
    // return x, normal                                                                       <L 691>
    ret_0 = var_12;
    ret_1 = var_70;
    return;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:298
static CUDA_CALLABLE wp::float32 user_sdf_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type)
{
    //---------
    // primal vars
    const wp::str var_0 = "ERROR: user_sdf function must be implemented by user code\n";
    const wp::float32 var_1 = 0.0;
    //---------
    // forward
    // def user_sdf(p: wp.vec3, attr: vec_pluginattr, sdf_type: int) -> float:                <L 299>
    // wp.printf("ERROR: user_sdf function must be implemented by user code\n")               <L 304>
    printf(var_0);
    // return 0.0                                                                             <L 305>
    return var_1;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:463
static CUDA_CALLABLE wp::float32 sdf_0(
    wp::int32 var_type,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    VolumeData_53ac1a2d var_volume_data,
    MeshData_52eaa0fa var_mesh_data)
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
    const wp::int32 var_7 = 0;
    bool var_8;
    const wp::int32 var_9 = 2;
    wp::float32 var_10;
    const wp::int32 var_11 = 2;
    bool var_12;
    wp::float32 var_13;
    const wp::int32 var_14 = 3;
    bool var_15;
    wp::float32 var_16;
    const wp::int32 var_17 = 5;
    bool var_18;
    wp::float32 var_19;
    const wp::int32 var_20 = 6;
    bool var_21;
    wp::float32 var_22;
    const wp::int32 var_23 = 4;
    bool var_24;
    wp::float32 var_25;
    bool var_26;
    const wp::int32 var_27 = 7;
    bool var_28;
    bool* var_29;
    bool var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::int32* var_33;
    wp::array_t<wp::int32>* var_34;
    wp::array_t<wp::int32>* var_35;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_36;
    wp::array_t<wp::vec_t<3, wp::int32>>* var_37;
    wp::int32* var_38;
    wp::vec_t<3, wp::float32>* var_39;
    wp::mat_t<3, 3, wp::float32>* var_40;
    wp::vec_t<3, wp::float32>* var_41;
    wp::vec_t<3, wp::float32>* var_42;
    wp::vec_t<3, wp::float32>* var_43;
    wp::float32 var_44;
    wp::vec_t<3, wp::float32> var_45;
    wp::int32 var_46;
    wp::array_t<wp::int32> var_47;
    wp::array_t<wp::int32> var_48;
    wp::array_t<wp::vec_t<3, wp::float32>> var_49;
    wp::array_t<wp::vec_t<3, wp::int32>> var_50;
    wp::int32 var_51;
    wp::vec_t<3, wp::float32> var_52;
    wp::mat_t<3, 3, wp::float32> var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::vec_t<3, wp::float32> var_55;
    wp::vec_t<3, wp::float32> var_56;
    wp::float32 var_57;
    bool var_58;
    wp::int32* var_59;
    wp::array_t<wp::int32>* var_60;
    wp::array_t<wp::int32>* var_61;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_62;
    wp::array_t<wp::vec_t<3, wp::int32>>* var_63;
    wp::int32* var_64;
    wp::vec_t<3, wp::float32>* var_65;
    wp::mat_t<3, 3, wp::float32>* var_66;
    wp::vec_t<3, wp::float32>* var_67;
    wp::vec_t<3, wp::float32>* var_68;
    wp::vec_t<3, wp::float32>* var_69;
    wp::vec_t<3, wp::float32> var_70;
    wp::vec_t<3, wp::float32> var_71;
    wp::float32 var_72;
    wp::vec_t<3, wp::float32> var_73;
    wp::int32 var_74;
    wp::array_t<wp::int32> var_75;
    wp::array_t<wp::int32> var_76;
    wp::array_t<wp::vec_t<3, wp::float32>> var_77;
    wp::array_t<wp::vec_t<3, wp::int32>> var_78;
    wp::int32 var_79;
    wp::vec_t<3, wp::float32> var_80;
    wp::mat_t<3, 3, wp::float32> var_81;
    wp::vec_t<3, wp::float32> var_82;
    wp::vec_t<3, wp::float32> var_83;
    wp::float32 var_84;
    wp::float32 var_85;
    wp::vec_t<3, wp::float32> var_86;
    const wp::int32 var_87 = 8;
    bool var_88;
    const wp::int32 var_89 = -1;
    bool var_90;
    wp::float32 var_91;
    wp::float32 var_92;
    bool var_93;
    const wp::int32 var_94 = 7;
    bool var_95;
    bool* var_96;
    bool var_97;
    wp::float32 var_98;
    const wp::str var_99 = "ERROR: SDF type not implemented\n";
    const wp::float32 var_100 = 0.0;
    //---------
    // forward
    // def sdf(type: int, p: wp.vec3, attr: vec_pluginattr, sdf_type: int, volume_data: VolumeData, mesh_data: MeshData) -> float:       <L 464>
    // attr_vec3 = wp.vec3(attr[0], attr[1], attr[2])                                         <L 466>
    var_1 = wp::extract(var_attr, var_0);
    var_3 = wp::extract(var_attr, var_2);
    var_5 = wp::extract(var_attr, var_4);
    var_6 = wp::vec_t<3, wp::float32>(var_1, var_3, var_5);
    // if type == GeomType.PLANE:                                                             <L 467>
    var_8 = (var_type == var_7);
    if (var_8) {
        // return p[2]                                                                        <L 468>
        var_10 = wp::extract(var_p, var_9);
        return var_10;
    }
    if (!var_8) {
        // elif type == GeomType.SPHERE:                                                      <L 469>
        var_12 = (var_type == var_11);
        if (var_12) {
            // return sphere(p, attr_vec3)                                                    <L 470>
            var_13 = sphere_0(var_p, var_6);
            return var_13;
        }
        if (!var_12) {
            // elif type == GeomType.CAPSULE:                                                 <L 471>
            var_15 = (var_type == var_14);
            if (var_15) {
                // return capsule(p, attr_vec3)                                               <L 472>
                var_16 = capsule_0(var_p, var_6);
                return var_16;
            }
            if (!var_15) {
                // elif type == GeomType.CYLINDER:                                            <L 473>
                var_18 = (var_type == var_17);
                if (var_18) {
                    // return cylinder(p, attr_vec3)                                          <L 474>
                    var_19 = cylinder_0(var_p, var_6);
                    return var_19;
                }
                if (!var_18) {
                    // elif type == GeomType.BOX:                                             <L 475>
                    var_21 = (var_type == var_20);
                    if (var_21) {
                        // return box(p, attr_vec3)                                           <L 476>
                        var_22 = box_0(var_p, var_6);
                        return var_22;
                    }
                    if (!var_21) {
                        // elif type == GeomType.ELLIPSOID:                                   <L 477>
                        var_24 = (var_type == var_23);
                        if (var_24) {
                            // return ellipsoid(p, attr_vec3)                                 <L 478>
                            var_25 = ellipsoid_0(var_p, var_6);
                            return var_25;
                        }
                        if (!var_24) {
                            // elif type == GeomType.MESH and mesh_data.valid:                <L 479>
                            var_28 = (var_type == var_27);
                            var_26 = var_28;
                            if (var_26) {
                                var_29 = &((var_mesh_data).valid);
                                var_30 = wp::load(var_29);
                                var_26 = var_26 && var_30;
                            }
                            if (var_26) {
                                // mesh_data.pnt = p                                          <L 480>
                                var_mesh_data.pnt = var_p;
                                // mesh_data.vec = -wp.normalize(p)                           <L 481>
                                var_31 = wp::normalize(var_p);
                                var_32 = wp::neg(var_31);
                                var_mesh_data.vec = var_32;
                                // dist, normal = ray_mesh(                                   <L 482>
                                // mesh_data.nmeshface,                                       <L 483>
                                var_33 = &((var_mesh_data).nmeshface);
                                // mesh_data.mesh_vertadr,                                    <L 484>
                                var_34 = &((var_mesh_data).mesh_vertadr);
                                // mesh_data.mesh_faceadr,                                    <L 485>
                                var_35 = &((var_mesh_data).mesh_faceadr);
                                // mesh_data.mesh_vert,                                       <L 486>
                                var_36 = &((var_mesh_data).mesh_vert);
                                // mesh_data.mesh_face,                                       <L 487>
                                var_37 = &((var_mesh_data).mesh_face);
                                // mesh_data.data_id,                                         <L 488>
                                var_38 = &((var_mesh_data).data_id);
                                // mesh_data.pos,                                             <L 489>
                                var_39 = &((var_mesh_data).pos);
                                // mesh_data.mat,                                             <L 490>
                                var_40 = &((var_mesh_data).mat);
                                // mesh_data.size,                                            <L 491>
                                var_41 = &((var_mesh_data).size);
                                // mesh_data.pnt,                                             <L 492>
                                var_42 = &((var_mesh_data).pnt);
                                // mesh_data.vec,                                             <L 493>
                                var_43 = &((var_mesh_data).vec);
                                var_46 = wp::load(var_33);
                                var_47 = wp::load(var_34);
                                var_48 = wp::load(var_35);
                                var_49 = wp::load(var_36);
                                var_50 = wp::load(var_37);
                                var_51 = wp::load(var_38);
                                var_52 = wp::load(var_39);
                                var_53 = wp::load(var_40);
                                var_54 = wp::load(var_41);
                                var_55 = wp::load(var_42);
                                var_56 = wp::load(var_43);
                                ray_mesh_0(var_46, var_47, var_48, var_49, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_44, var_45);
                                // if dist > wp.norm_l2(p):                                   <L 495>
                                var_57 = norm_l2_0(var_p);
                                var_58 = (var_44 > var_57);
                                if (var_58) {
                                    // dist, normal = ray_mesh(                               <L 496>
                                    // mesh_data.nmeshface,                                   <L 497>
                                    var_59 = &((var_mesh_data).nmeshface);
                                    // mesh_data.mesh_vertadr,                                <L 498>
                                    var_60 = &((var_mesh_data).mesh_vertadr);
                                    // mesh_data.mesh_faceadr,                                <L 499>
                                    var_61 = &((var_mesh_data).mesh_faceadr);
                                    // mesh_data.mesh_vert,                                   <L 500>
                                    var_62 = &((var_mesh_data).mesh_vert);
                                    // mesh_data.mesh_face,                                   <L 501>
                                    var_63 = &((var_mesh_data).mesh_face);
                                    // mesh_data.data_id,                                     <L 502>
                                    var_64 = &((var_mesh_data).data_id);
                                    // mesh_data.pos,                                         <L 503>
                                    var_65 = &((var_mesh_data).pos);
                                    // mesh_data.mat,                                         <L 504>
                                    var_66 = &((var_mesh_data).mat);
                                    // mesh_data.size,                                        <L 505>
                                    var_67 = &((var_mesh_data).size);
                                    // mesh_data.pnt,                                         <L 506>
                                    var_68 = &((var_mesh_data).pnt);
                                    // -mesh_data.vec,                                        <L 507>
                                    var_69 = &((var_mesh_data).vec);
                                    var_71 = wp::load(var_69);
                                    var_70 = wp::neg(var_71);
                                    var_74 = wp::load(var_59);
                                    var_75 = wp::load(var_60);
                                    var_76 = wp::load(var_61);
                                    var_77 = wp::load(var_62);
                                    var_78 = wp::load(var_63);
                                    var_79 = wp::load(var_64);
                                    var_80 = wp::load(var_65);
                                    var_81 = wp::load(var_66);
                                    var_82 = wp::load(var_67);
                                    var_83 = wp::load(var_68);
                                    ray_mesh_0(var_74, var_75, var_76, var_77, var_78, var_79, var_80, var_81, var_82, var_83, var_70, var_72, var_73);
                                    // return -dist                                           <L 509>
                                    var_84 = wp::neg(var_72);
                                    return var_84;
                                }
                                var_85 = wp::where(var_58, var_72, var_44);
                                var_86 = wp::where(var_58, var_73, var_45);
                                // return dist                                                <L 510>
                                return var_85;
                            }
                            if (!var_26) {
                                // elif type == GeomType.SDF:                                 <L 511>
                                var_88 = (var_type == var_87);
                                if (var_88) {
                                    // if sdf_type == -1:                                     <L 512>
                                    var_90 = (var_sdf_type == var_89);
                                    if (var_90) {
                                        // return sample_volume_sdf(p, volume_data)           <L 513>
                                        var_91 = sample_volume_sdf_0(var_p, var_volume_data);
                                        return var_91;
                                    }
                                    if (!var_90) {
                                        // return user_sdf(p, attr, sdf_type)                 <L 515>
                                        var_92 = user_sdf_0(var_p, var_attr, var_sdf_type);
                                        return var_92;
                                    }
                                }
                                if (!var_88) {
                                    // elif type == GeomType.MESH and volume_data.valid:       <L 516>
                                    var_95 = (var_type == var_94);
                                    var_93 = var_95;
                                    if (var_93) {
                                        var_96 = &((var_volume_data).valid);
                                        var_97 = wp::load(var_96);
                                        var_93 = var_93 && var_97;
                                    }
                                    if (var_93) {
                                        // return sample_volume_sdf(p, volume_data)           <L 517>
                                        var_98 = sample_volume_sdf_0(var_p, var_volume_data);
                                        return var_98;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // wp.printf("ERROR: SDF type not implemented\n")                                         <L 518>
    printf(var_99);
    // return 0.0                                                                             <L 519>
    return var_100;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:205
static CUDA_CALLABLE wp::vec_t<3, wp::float32> grad_sphere_0(
    wp::vec_t<3, wp::float32> var_p)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::float32 var_1 = 1e-09;
    bool var_2;
    wp::vec_t<3, wp::float32> var_3;
    const wp::float32 var_4 = 0.0;
    wp::vec_t<3, wp::float32> var_5;
    //---------
    // forward
    // def grad_sphere(p: wp.vec3) -> wp.vec3:                                                <L 206>
    // c = wp.length(p)                                                                       <L 207>
    var_0 = wp::length(var_p);
    // if c > 1e-9:                                                                           <L 208>
    var_2 = (var_0 > var_1);
    if (var_2) {
        // return p / c                                                                       <L 209>
        var_3 = wp::div(var_p, var_0);
        return var_3;
    }
    if (!var_2) {
        // return wp.vec3(0.0)                                                                <L 211>
        var_5 = wp::vec_t<3, wp::float32>(var_4);
        return var_5;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:252
static CUDA_CALLABLE wp::vec_t<3, wp::float32> grad_capsule_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 1;
    wp::float32 var_1;
    const wp::int32 var_2 = 2;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    const wp::int32 var_6 = 0;
    wp::float32 var_7;
    const wp::int32 var_8 = 1;
    wp::float32 var_9;
    const wp::int32 var_10 = 2;
    wp::float32 var_11;
    wp::float32 var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::float32 var_14;
    const wp::float32 var_15 = 1e-15;
    bool var_16;
    wp::vec_t<3, wp::float32> var_17;
    const wp::float32 var_18 = 0.0;
    wp::vec_t<3, wp::float32> var_19;
    //---------
    // forward
    // def grad_capsule(p: wp.vec3, size: wp.vec3) -> wp.vec3:                                <L 253>
    // h = size[1]                                                                            <L 254>
    var_1 = wp::extract(var_size, var_0);
    // pz_clamped = wp.clamp(p[2], -h, h)                                                     <L 255>
    var_3 = wp::extract(var_p, var_2);
    var_4 = wp::neg(var_1);
    var_5 = wp::clamp(var_3, var_4, var_1);
    // diff = wp.vec3(p[0], p[1], p[2] - pz_clamped)                                          <L 256>
    var_7 = wp::extract(var_p, var_6);
    var_9 = wp::extract(var_p, var_8);
    var_11 = wp::extract(var_p, var_10);
    var_12 = wp::sub(var_11, var_5);
    var_13 = wp::vec_t<3, wp::float32>(var_7, var_9, var_12);
    // c = wp.length(diff)                                                                    <L 257>
    var_14 = wp::length(var_13);
    // if c > MJ_MINVAL:                                                                      <L 258>
    var_16 = (var_14 > var_15);
    if (var_16) {
        // return diff / c                                                                    <L 259>
        var_17 = wp::div(var_13, var_14);
        return var_17;
    }
    if (!var_16) {
        // return wp.vec3(0.0)                                                                <L 261>
        var_19 = wp::vec_t<3, wp::float32>(var_18);
        return var_19;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:264
static CUDA_CALLABLE wp::vec_t<3, wp::float32> grad_cylinder_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    const wp::int32 var_0 = 0;
    wp::float32 var_1;
    const wp::int32 var_2 = 1;
    wp::float32 var_3;
    const wp::int32 var_4 = 0;
    wp::float32 var_5;
    const wp::int32 var_6 = 1;
    wp::float32 var_7;
    wp::vec_t<2, wp::float32> var_8;
    wp::float32 var_9;
    const wp::float32 var_10 = 1e-15;
    bool var_11;
    const wp::int32 var_12 = 0;
    wp::float32 var_13;
    wp::float32 var_14;
    const wp::int32 var_15 = 1;
    wp::float32 var_16;
    wp::float32 var_17;
    const wp::float32 var_18 = 0.0;
    wp::vec_t<3, wp::float32> var_19;
    const wp::float32 var_20 = 0.0;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    const wp::float32 var_23 = 0.0;
    const wp::float32 var_24 = 0.0;
    const wp::int32 var_25 = 2;
    wp::float32 var_26;
    wp::float32 var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::float32 var_29;
    const wp::int32 var_30 = 2;
    wp::float32 var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    bool var_34;
    const wp::float32 var_35 = 0.0;
    bool var_36;
    const wp::float32 var_37 = 0.0;
    bool var_38;
    wp::vec_t<2, wp::float32> var_39;
    wp::float32 var_40;
    bool var_41;
    wp::float32 var_42;
    wp::vec_t<3, wp::float32> var_43;
    wp::float32 var_44;
    wp::vec_t<3, wp::float32> var_45;
    wp::vec_t<3, wp::float32> var_46;
    const wp::float32 var_47 = 0.0;
    wp::vec_t<3, wp::float32> var_48;
    const wp::float32 var_49 = 0.0;
    bool var_50;
    const wp::float32 var_51 = 0.0;
    bool var_52;
    bool var_53;
    //---------
    // forward
    // def grad_cylinder(p: wp.vec3, size: wp.vec3) -> wp.vec3:                               <L 265>
    // r = size[0]                                                                            <L 266>
    var_1 = wp::extract(var_size, var_0);
    // h = size[1]                                                                            <L 267>
    var_3 = wp::extract(var_size, var_2);
    // radial_dist = wp.length(wp.vec2(p[0], p[1]))                                           <L 269>
    var_5 = wp::extract(var_p, var_4);
    var_7 = wp::extract(var_p, var_6);
    var_8 = wp::vec_t<2, wp::float32>(var_5, var_7);
    var_9 = wp::length(var_8);
    // if radial_dist > MJ_MINVAL:                                                            <L 270>
    var_11 = (var_9 > var_10);
    if (var_11) {
        // u = wp.vec3(p[0] / radial_dist, p[1] / radial_dist, 0.0)                           <L 271>
        var_13 = wp::extract(var_p, var_12);
        var_14 = wp::div(var_13, var_9);
        var_16 = wp::extract(var_p, var_15);
        var_17 = wp::div(var_16, var_9);
        var_19 = wp::vec_t<3, wp::float32>(var_14, var_17, var_18);
    }
    if (!var_11) {
        // u = wp.vec3(0.0)                                                                   <L 273>
        var_21 = wp::vec_t<3, wp::float32>(var_20);
    }
    var_22 = wp::where(var_11, var_19, var_21);
    // w = wp.vec3(0.0, 0.0, wp.sign(p[2]))                                                   <L 275>
    var_26 = wp::extract(var_p, var_25);
    var_27 = wp::sign(var_26);
    var_28 = wp::vec_t<3, wp::float32>(var_23, var_24, var_27);
    // dx = radial_dist - r                                                                   <L 277>
    var_29 = wp::sub(var_9, var_1);
    // dy = wp.abs(p[2]) - h                                                                  <L 278>
    var_31 = wp::extract(var_p, var_30);
    var_32 = wp::abs(var_31);
    var_33 = wp::sub(var_32, var_3);
    // if dx > 0.0 and dy > 0.0:                                                              <L 280>
    var_36 = (var_29 > var_35);
    var_34 = var_36;
    if (var_34) {
        var_38 = (var_33 > var_37);
        var_34 = var_34 && var_38;
    }
    if (var_34) {
        // v = wp.vec2(dx, dy)                                                                <L 281>
        var_39 = wp::vec_t<2, wp::float32>(var_29, var_33);
        // len_v = wp.length(v)                                                               <L 282>
        var_40 = wp::length(var_39);
        // if len_v > MJ_MINVAL:                                                              <L 283>
        var_41 = (var_40 > var_10);
        if (var_41) {
            // return u * (dx / len_v) + w * (dy / len_v)                                     <L 284>
            var_42 = wp::div(var_29, var_40);
            var_43 = wp::mul(var_22, var_42);
            var_44 = wp::div(var_33, var_40);
            var_45 = wp::mul(var_28, var_44);
            var_46 = wp::add(var_43, var_45);
            return var_46;
        }
        if (!var_41) {
            // return wp.vec3(0.0)                                                            <L 286>
            var_48 = wp::vec_t<3, wp::float32>(var_47);
            return var_48;
        }
    }
    if (!var_34) {
        // elif dx > 0.0:                                                                     <L 287>
        var_50 = (var_29 > var_49);
        if (var_50) {
            // return u                                                                       <L 288>
            return var_22;
        }
        if (!var_50) {
            // elif dy > 0.0:                                                                 <L 289>
            var_52 = (var_33 > var_51);
            if (var_52) {
                // return w                                                                   <L 290>
                return var_28;
            }
            if (!var_52) {
                // if dx > dy:                                                                <L 292>
                var_53 = (var_29 > var_33);
                if (var_53) {
                    // return u                                                               <L 293>
                    return var_22;
                }
                if (!var_53) {
                    // return w                                                               <L 295>
                    return var_28;
                }
            }
        }
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:214
static CUDA_CALLABLE wp::vec_t<3, wp::float32> grad_box_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
{
    //---------
    // primal vars
    wp::vec_t<3, wp::float32> var_0;
    wp::vec_t<3, wp::float32> var_1;
    wp::float32 var_2;
    const wp::int32 var_3 = 0;
    bool var_4;
    wp::vec_t<3, wp::float32> var_5;
    const wp::float32 var_6 = 0.0;
    const wp::float32 var_7 = 0.0;
    const wp::float32 var_8 = 0.0;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<3, wp::float32> var_10;
    wp::float32 var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::vec_t<3, wp::float32> var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<3, wp::float32> var_15;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    const wp::int32 var_18 = 0;
    bool var_19;
    const wp::float32 var_20 = 0.0;
    const wp::int32 var_21 = 0;
    const wp::int32 var_22 = 1;
    wp::float32 var_23;
    const wp::int32 var_24 = 0;
    bool var_25;
    const wp::float32 var_26 = 0.0;
    const wp::int32 var_27 = 1;
    const wp::int32 var_28 = 2;
    wp::float32 var_29;
    const wp::int32 var_30 = 0;
    bool var_31;
    const wp::float32 var_32 = 0.0;
    const wp::int32 var_33 = 2;
    //---------
    // forward
    // def grad_box(p: wp.vec3, size: wp.vec3) -> wp.vec3:                                    <L 215>
    // a = wp.abs(p) - size                                                                   <L 216>
    var_0 = wp::abs(var_p);
    var_1 = wp::sub(var_0, var_size);
    // if wp.max(a) < 0:                                                                      <L 217>
    var_2 = wp::max(var_1);
    var_4 = (var_2 < var_3);
    if (var_4) {
        // return radial_field(a, p, size)                                                    <L 218>
        var_5 = radial_field_0(var_1, var_p, var_size);
        return var_5;
    }
    // z = wp.vec3(0.0, 0.0, 0.0)                                                             <L 219>
    var_9 = wp::vec_t<3, wp::float32>(var_6, var_7, var_8);
    // b = wp.max(a, z)                                                                       <L 220>
    var_10 = wp::max(var_1, var_9);
    // c = wp.norm_l2(b)                                                                      <L 221>
    var_11 = norm_l2_0(var_10);
    // g = wp.cw_mul(wp.div(b, c), wp.cw_div(p, wp.abs(p)))                                   <L 222>
    var_12 = wp::div(var_10, var_11);
    var_13 = wp::abs(var_p);
    var_14 = wp::cw_div(var_p, var_13);
    var_15 = wp::cw_mul(var_12, var_14);
    // if a[0] <= 0:                                                                          <L 223>
    var_17 = wp::extract(var_1, var_16);
    var_19 = (var_17 <= var_18);
    if (var_19) {
        // g[0] = 0.0                                                                         <L 224>
        wp::assign_inplace(var_15, var_21, var_20);
    }
    // if a[1] <= 0:                                                                          <L 225>
    var_23 = wp::extract(var_1, var_22);
    var_25 = (var_23 <= var_24);
    if (var_25) {
        // g[1] = 0.0                                                                         <L 226>
        wp::assign_inplace(var_15, var_27, var_26);
    }
    // if a[2] <= 0:                                                                          <L 227>
    var_29 = wp::extract(var_1, var_28);
    var_31 = (var_29 <= var_30);
    if (var_31) {
        // g[2] = 0.0                                                                         <L 228>
        wp::assign_inplace(var_15, var_33, var_32);
    }
    // return g                                                                               <L 229>
    return var_15;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:232
static CUDA_CALLABLE wp::vec_t<3, wp::float32> grad_ellipsoid_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size)
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
    const wp::int32 var_10 = 2;
    wp::float32 var_11;
    const wp::int32 var_12 = 2;
    wp::float32 var_13;
    wp::float32 var_14;
    wp::vec_t<3, wp::float32> var_15;
    const wp::int32 var_16 = 0;
    wp::float32 var_17;
    const wp::int32 var_18 = 0;
    wp::float32 var_19;
    wp::float32 var_20;
    const wp::int32 var_21 = 1;
    wp::float32 var_22;
    const wp::int32 var_23 = 1;
    wp::float32 var_24;
    wp::float32 var_25;
    const wp::int32 var_26 = 2;
    wp::float32 var_27;
    const wp::int32 var_28 = 2;
    wp::float32 var_29;
    wp::float32 var_30;
    wp::vec_t<3, wp::float32> var_31;
    wp::float32 var_32;
    wp::float32 var_33;
    const wp::float32 var_34 = 1.0;
    wp::float32 var_35;
    const wp::float32 var_36 = 1.0;
    wp::float32 var_37;
    wp::vec_t<3, wp::float32> var_38;
    const wp::int32 var_39 = 0;
    wp::float32 var_40;
    wp::float32 var_41;
    const wp::int32 var_42 = 0;
    wp::float32 var_43;
    const wp::int32 var_44 = 0;
    wp::float32 var_45;
    wp::float32 var_46;
    wp::float32 var_47;
    const wp::int32 var_48 = 1;
    wp::float32 var_49;
    wp::float32 var_50;
    const wp::int32 var_51 = 1;
    wp::float32 var_52;
    const wp::int32 var_53 = 1;
    wp::float32 var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    const wp::int32 var_57 = 2;
    wp::float32 var_58;
    wp::float32 var_59;
    const wp::int32 var_60 = 2;
    wp::float32 var_61;
    const wp::int32 var_62 = 2;
    wp::float32 var_63;
    wp::float32 var_64;
    wp::float32 var_65;
    wp::vec_t<3, wp::float32> var_66;
    const wp::float32 var_67 = 2.0;
    wp::float32 var_68;
    const wp::float32 var_69 = 1.0;
    wp::float32 var_70;
    wp::float32 var_71;
    const wp::float32 var_72 = 1.0;
    wp::float32 var_73;
    wp::float32 var_74;
    wp::float32 var_75;
    wp::float32 var_76;
    wp::vec_t<3, wp::float32> var_77;
    wp::vec_t<3, wp::float32> var_78;
    wp::vec_t<3, wp::float32> var_79;
    wp::float32 var_80;
    wp::vec_t<3, wp::float32> var_81;
    //---------
    // forward
    // def grad_ellipsoid(p: wp.vec3, size: wp.vec3) -> wp.vec3:                              <L 233>
    // a = wp.vec3(p[0] / size[0], p[1] / size[1], p[2] / size[2])                            <L 234>
    var_1 = wp::extract(var_p, var_0);
    var_3 = wp::extract(var_size, var_2);
    var_4 = wp::div(var_1, var_3);
    var_6 = wp::extract(var_p, var_5);
    var_8 = wp::extract(var_size, var_7);
    var_9 = wp::div(var_6, var_8);
    var_11 = wp::extract(var_p, var_10);
    var_13 = wp::extract(var_size, var_12);
    var_14 = wp::div(var_11, var_13);
    var_15 = wp::vec_t<3, wp::float32>(var_4, var_9, var_14);
    // b = wp.vec3(a[0] / size[0], a[1] / size[1], a[2] / size[2])                            <L 235>
    var_17 = wp::extract(var_15, var_16);
    var_19 = wp::extract(var_size, var_18);
    var_20 = wp::div(var_17, var_19);
    var_22 = wp::extract(var_15, var_21);
    var_24 = wp::extract(var_size, var_23);
    var_25 = wp::div(var_22, var_24);
    var_27 = wp::extract(var_15, var_26);
    var_29 = wp::extract(var_size, var_28);
    var_30 = wp::div(var_27, var_29);
    var_31 = wp::vec_t<3, wp::float32>(var_20, var_25, var_30);
    // k0 = wp.length(a)                                                                      <L 236>
    var_32 = wp::length(var_15);
    // k1 = wp.length(b)                                                                      <L 237>
    var_33 = wp::length(var_31);
    // invK0 = 1.0 / k0                                                                       <L 238>
    var_35 = wp::div(var_34, var_32);
    // invK1 = 1.0 / k1                                                                       <L 239>
    var_37 = wp::div(var_36, var_33);
    // gk0 = b * invK0                                                                        <L 240>
    var_38 = wp::mul(var_31, var_35);
    // gk1 = wp.vec3(                                                                         <L 241>
    // b[0] * invK1 / (size[0] * size[0]),                                                    <L 242>
    var_40 = wp::extract(var_31, var_39);
    var_41 = wp::mul(var_40, var_37);
    var_43 = wp::extract(var_size, var_42);
    var_45 = wp::extract(var_size, var_44);
    var_46 = wp::mul(var_43, var_45);
    var_47 = wp::div(var_41, var_46);
    // b[1] * invK1 / (size[1] * size[1]),                                                    <L 243>
    var_49 = wp::extract(var_31, var_48);
    var_50 = wp::mul(var_49, var_37);
    var_52 = wp::extract(var_size, var_51);
    var_54 = wp::extract(var_size, var_53);
    var_55 = wp::mul(var_52, var_54);
    var_56 = wp::div(var_50, var_55);
    // b[2] * invK1 / (size[2] * size[2]),                                                    <L 244>
    var_58 = wp::extract(var_31, var_57);
    var_59 = wp::mul(var_58, var_37);
    var_61 = wp::extract(var_size, var_60);
    var_63 = wp::extract(var_size, var_62);
    var_64 = wp::mul(var_61, var_63);
    var_65 = wp::div(var_59, var_64);
    var_66 = wp::vec_t<3, wp::float32>(var_47, var_56, var_65);
    // df_dk0 = (2.0 * k0 - 1.0) * invK1                                                      <L 246>
    var_68 = wp::mul(var_67, var_32);
    var_70 = wp::sub(var_68, var_69);
    var_71 = wp::mul(var_70, var_37);
    // df_dk1 = k0 * (k0 - 1.0) * invK1 * invK1                                               <L 247>
    var_73 = wp::sub(var_32, var_72);
    var_74 = wp::mul(var_32, var_73);
    var_75 = wp::mul(var_74, var_37);
    var_76 = wp::mul(var_75, var_37);
    // raw_grad = gk0 * df_dk0 - gk1 * df_dk1                                                 <L 248>
    var_77 = wp::mul(var_38, var_71);
    var_78 = wp::mul(var_66, var_76);
    var_79 = wp::sub(var_77, var_78);
    // return raw_grad / wp.length(raw_grad)                                                  <L 249>
    var_80 = wp::length(var_79);
    var_81 = wp::div(var_79, var_80);
    return var_81;
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:308
static CUDA_CALLABLE wp::vec_t<3, wp::float32> user_sdf_grad_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type)
{
    //---------
    // primal vars
    const wp::str var_0 = "ERROR: user_sdf_grad function must be implemented by user code\n";
    const wp::float32 var_1 = 0.0;
    wp::vec_t<3, wp::float32> var_2;
    //---------
    // forward
    // def user_sdf_grad(p: wp.vec3, attr: vec_pluginattr, sdf_type: int) -> wp.vec3:         <L 309>
    // wp.printf("ERROR: user_sdf_grad function must be implemented by user code\n")          <L 314>
    printf(var_0);
    // return wp.vec3(0.0)                                                                    <L 315>
    var_2 = wp::vec_t<3, wp::float32>(var_1);
    return var_2;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:522
static CUDA_CALLABLE wp::vec_t<3, wp::float32> sdf_grad_0(
    wp::int32 var_type,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    VolumeData_53ac1a2d var_volume_data,
    MeshData_52eaa0fa var_mesh_data)
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
    const wp::int32 var_7 = 0;
    bool var_8;
    const wp::float32 var_9 = 0.0;
    const wp::float32 var_10 = 0.0;
    const wp::float32 var_11 = 1.0;
    wp::vec_t<3, wp::float32> var_12;
    const wp::int32 var_13 = 2;
    bool var_14;
    wp::vec_t<3, wp::float32> var_15;
    const wp::int32 var_16 = 3;
    bool var_17;
    wp::vec_t<3, wp::float32> var_18;
    const wp::int32 var_19 = 5;
    bool var_20;
    wp::vec_t<3, wp::float32> var_21;
    const wp::int32 var_22 = 6;
    bool var_23;
    wp::vec_t<3, wp::float32> var_24;
    const wp::int32 var_25 = 4;
    bool var_26;
    wp::vec_t<3, wp::float32> var_27;
    bool var_28;
    const wp::int32 var_29 = 7;
    bool var_30;
    bool* var_31;
    bool var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::int32* var_35;
    wp::array_t<wp::int32>* var_36;
    wp::array_t<wp::int32>* var_37;
    wp::array_t<wp::vec_t<3, wp::float32>>* var_38;
    wp::array_t<wp::vec_t<3, wp::int32>>* var_39;
    wp::int32* var_40;
    wp::vec_t<3, wp::float32>* var_41;
    wp::mat_t<3, 3, wp::float32>* var_42;
    wp::vec_t<3, wp::float32>* var_43;
    wp::vec_t<3, wp::float32>* var_44;
    wp::vec_t<3, wp::float32>* var_45;
    wp::float32 var_46;
    wp::vec_t<3, wp::float32> var_47;
    wp::int32 var_48;
    wp::array_t<wp::int32> var_49;
    wp::array_t<wp::int32> var_50;
    wp::array_t<wp::vec_t<3, wp::float32>> var_51;
    wp::array_t<wp::vec_t<3, wp::int32>> var_52;
    wp::int32 var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::mat_t<3, 3, wp::float32> var_55;
    wp::vec_t<3, wp::float32> var_56;
    wp::vec_t<3, wp::float32> var_57;
    wp::vec_t<3, wp::float32> var_58;
    wp::float32 var_59;
    bool var_60;
    const wp::float32 var_61 = 1.0;
    wp::vec_t<3, wp::float32> var_62;
    const wp::float32 var_63 = -1.0;
    wp::vec_t<3, wp::float32> var_64;
    const wp::int32 var_65 = 8;
    bool var_66;
    const wp::int32 var_67 = -1;
    bool var_68;
    wp::vec_t<3, wp::float32> var_69;
    wp::vec_t<3, wp::float32> var_70;
    bool var_71;
    const wp::int32 var_72 = 7;
    bool var_73;
    bool* var_74;
    bool var_75;
    wp::vec_t<3, wp::float32> var_76;
    const wp::str var_77 = "ERROR: SDF grad type not implemented\n";
    const wp::float32 var_78 = 0.0;
    wp::vec_t<3, wp::float32> var_79;
    //---------
    // forward
    // def sdf_grad(                                                                          <L 523>
    // attr_vec3 = wp.vec3(attr[0], attr[1], attr[2])                                         <L 527>
    var_1 = wp::extract(var_attr, var_0);
    var_3 = wp::extract(var_attr, var_2);
    var_5 = wp::extract(var_attr, var_4);
    var_6 = wp::vec_t<3, wp::float32>(var_1, var_3, var_5);
    // if type == GeomType.PLANE:                                                             <L 528>
    var_8 = (var_type == var_7);
    if (var_8) {
        // grad = wp.vec3(0.0, 0.0, 1.0)                                                      <L 529>
        var_12 = wp::vec_t<3, wp::float32>(var_9, var_10, var_11);
        // return grad                                                                        <L 530>
        return var_12;
    }
    if (!var_8) {
        // elif type == GeomType.SPHERE:                                                      <L 531>
        var_14 = (var_type == var_13);
        if (var_14) {
            // return grad_sphere(p)                                                          <L 532>
            var_15 = grad_sphere_0(var_p);
            return var_15;
        }
        if (!var_14) {
            // elif type == GeomType.CAPSULE:                                                 <L 533>
            var_17 = (var_type == var_16);
            if (var_17) {
                // return grad_capsule(p, attr_vec3)                                          <L 534>
                var_18 = grad_capsule_0(var_p, var_6);
                return var_18;
            }
            if (!var_17) {
                // elif type == GeomType.CYLINDER:                                            <L 535>
                var_20 = (var_type == var_19);
                if (var_20) {
                    // return grad_cylinder(p, attr_vec3)                                     <L 536>
                    var_21 = grad_cylinder_0(var_p, var_6);
                    return var_21;
                }
                if (!var_20) {
                    // elif type == GeomType.BOX:                                             <L 537>
                    var_23 = (var_type == var_22);
                    if (var_23) {
                        // return grad_box(p, attr_vec3)                                      <L 538>
                        var_24 = grad_box_0(var_p, var_6);
                        return var_24;
                    }
                    if (!var_23) {
                        // elif type == GeomType.ELLIPSOID:                                   <L 539>
                        var_26 = (var_type == var_25);
                        if (var_26) {
                            // return grad_ellipsoid(p, attr_vec3)                            <L 540>
                            var_27 = grad_ellipsoid_0(var_p, var_6);
                            return var_27;
                        }
                        if (!var_26) {
                            // elif type == GeomType.MESH and mesh_data.valid:                <L 541>
                            var_30 = (var_type == var_29);
                            var_28 = var_30;
                            if (var_28) {
                                var_31 = &((var_mesh_data).valid);
                                var_32 = wp::load(var_31);
                                var_28 = var_28 && var_32;
                            }
                            if (var_28) {
                                // mesh_data.pnt = p                                          <L 542>
                                var_mesh_data.pnt = var_p;
                                // mesh_data.vec = -wp.normalize(p)                           <L 543>
                                var_33 = wp::normalize(var_p);
                                var_34 = wp::neg(var_33);
                                var_mesh_data.vec = var_34;
                                // dist, normal = ray_mesh(                                   <L 544>
                                // mesh_data.nmeshface,                                       <L 545>
                                var_35 = &((var_mesh_data).nmeshface);
                                // mesh_data.mesh_vertadr,                                    <L 546>
                                var_36 = &((var_mesh_data).mesh_vertadr);
                                // mesh_data.mesh_faceadr,                                    <L 547>
                                var_37 = &((var_mesh_data).mesh_faceadr);
                                // mesh_data.mesh_vert,                                       <L 548>
                                var_38 = &((var_mesh_data).mesh_vert);
                                // mesh_data.mesh_face,                                       <L 549>
                                var_39 = &((var_mesh_data).mesh_face);
                                // mesh_data.data_id,                                         <L 550>
                                var_40 = &((var_mesh_data).data_id);
                                // mesh_data.pos,                                             <L 551>
                                var_41 = &((var_mesh_data).pos);
                                // mesh_data.mat,                                             <L 552>
                                var_42 = &((var_mesh_data).mat);
                                // mesh_data.size,                                            <L 553>
                                var_43 = &((var_mesh_data).size);
                                // mesh_data.pnt,                                             <L 554>
                                var_44 = &((var_mesh_data).pnt);
                                // mesh_data.vec,                                             <L 555>
                                var_45 = &((var_mesh_data).vec);
                                var_48 = wp::load(var_35);
                                var_49 = wp::load(var_36);
                                var_50 = wp::load(var_37);
                                var_51 = wp::load(var_38);
                                var_52 = wp::load(var_39);
                                var_53 = wp::load(var_40);
                                var_54 = wp::load(var_41);
                                var_55 = wp::load(var_42);
                                var_56 = wp::load(var_43);
                                var_57 = wp::load(var_44);
                                var_58 = wp::load(var_45);
                                ray_mesh_0(var_48, var_49, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_57, var_58, var_46, var_47);
                                // if dist > wp.norm_l2(p):                                   <L 557>
                                var_59 = norm_l2_0(var_p);
                                var_60 = (var_46 > var_59);
                                if (var_60) {
                                    // return wp.vec3(1.0)                                    <L 558>
                                    var_62 = wp::vec_t<3, wp::float32>(var_61);
                                    return var_62;
                                }
                                if (!var_60) {
                                    // return wp.vec3(-1.0)                                   <L 560>
                                    var_64 = wp::vec_t<3, wp::float32>(var_63);
                                    return var_64;
                                }
                            }
                            if (!var_28) {
                                // elif type == GeomType.SDF:                                 <L 562>
                                var_66 = (var_type == var_65);
                                if (var_66) {
                                    // if sdf_type == -1:                                     <L 563>
                                    var_68 = (var_sdf_type == var_67);
                                    if (var_68) {
                                        // return sample_volume_grad(p, volume_data)          <L 564>
                                        var_69 = sample_volume_grad_0(var_p, var_volume_data);
                                        return var_69;
                                    }
                                    if (!var_68) {
                                        // return user_sdf_grad(p, attr, sdf_type)            <L 566>
                                        var_70 = user_sdf_grad_0(var_p, var_attr, var_sdf_type);
                                        return var_70;
                                    }
                                }
                                if (!var_66) {
                                    // elif type == GeomType.MESH and volume_data.valid:       <L 567>
                                    var_73 = (var_type == var_72);
                                    var_71 = var_73;
                                    if (var_71) {
                                        var_74 = &((var_volume_data).valid);
                                        var_75 = wp::load(var_74);
                                        var_71 = var_71 && var_75;
                                    }
                                    if (var_71) {
                                        // return sample_volume_grad(p, volume_data)          <L 568>
                                        var_76 = sample_volume_grad_0(var_p, var_volume_data);
                                        return var_76;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // wp.printf("ERROR: SDF grad type not implemented\n")                                    <L 569>
    printf(var_77);
    // return wp.vec3(0.0)                                                                    <L 570>
    var_79 = wp::vec_t<3, wp::float32>(var_78);
    return var_79;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:597
static CUDA_CALLABLE wp::vec_t<3, wp::float32> compute_grad_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    OptimizationParams_51b33fc1 var_params,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2)
{
    //---------
    // primal vars
    wp::vec_t<128, wp::float32>* var_0;
    wp::float32 var_1;
    wp::vec_t<128, wp::float32> var_2;
    const wp::int32 var_3 = 8;
    const wp::int32 var_4 = 8;
    wp::vec_t<128, wp::float32>* var_5;
    wp::float32 var_6;
    wp::vec_t<128, wp::float32> var_7;
    wp::vec_t<128, wp::float32>* var_8;
    wp::vec_t<3, wp::float32> var_9;
    wp::vec_t<128, wp::float32> var_10;
    const wp::int32 var_11 = 8;
    const wp::int32 var_12 = 8;
    wp::vec_t<128, wp::float32>* var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::vec_t<128, wp::float32> var_15;
    wp::mat_t<3, 3, wp::float32>* var_16;
    wp::mat_t<3, 3, wp::float32> var_17;
    wp::mat_t<3, 3, wp::float32> var_18;
    wp::vec_t<3, wp::float32> var_19;
    bool var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::float32 var_22;
    bool var_23;
    wp::vec_t<3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    wp::float32 var_27;
    wp::vec_t<3, wp::float32> var_28;
    wp::vec_t<3, wp::float32> var_29;
    //---------
    // forward
    // def compute_grad(                                                                      <L 598>
    // A = sdf(type1, p1, params.attr1, sdf_type1, volume_data1, mesh_data1)                  <L 612>
    var_0 = &((var_params).attr1);
    var_2 = wp::load(var_0);
    var_1 = sdf_0(var_type1, var_p1, var_2, var_sdf_type1, var_volume_data1, var_mesh_data1);
    // B = sdf(GeomType.SDF, p2, params.attr2, sdf_type2, volume_data2, mesh_data2)           <L 613>
    var_5 = &((var_params).attr2);
    var_7 = wp::load(var_5);
    var_6 = sdf_0(var_4, var_p2, var_7, var_sdf_type2, var_volume_data2, var_mesh_data2);
    // grad1 = sdf_grad(type1, p1, params.attr1, sdf_type1, volume_data1, mesh_data1)         <L 614>
    var_8 = &((var_params).attr1);
    var_10 = wp::load(var_8);
    var_9 = sdf_grad_0(var_type1, var_p1, var_10, var_sdf_type1, var_volume_data1, var_mesh_data1);
    // grad2 = sdf_grad(GeomType.SDF, p2, params.attr2, sdf_type2, volume_data2, mesh_data2)       <L 615>
    var_13 = &((var_params).attr2);
    var_15 = wp::load(var_13);
    var_14 = sdf_grad_0(var_12, var_p2, var_15, var_sdf_type2, var_volume_data2, var_mesh_data2);
    // grad1_transformed = wp.transpose(params.rel_mat) * grad1                               <L 616>
    var_16 = &((var_params).rel_mat);
    var_18 = wp::load(var_16);
    var_17 = wp::transpose(var_18);
    var_19 = wp::mul(var_17, var_9);
    // if sfd_intersection:                                                                   <L 617>
    if (var_sfd_intersection) {
        // if A > B:                                                                          <L 618>
        var_20 = (var_1 > var_6);
        if (var_20) {
            // return grad1_transformed                                                       <L 619>
            return var_19;
        }
        if (!var_20) {
            // return grad2                                                                   <L 621>
            return var_14;
        }
    }
    if (!var_sfd_intersection) {
        // gradient = grad2 + grad1_transformed                                               <L 623>
        var_21 = wp::add(var_14, var_19);
        // max_val = wp.max(A, B)                                                             <L 624>
        var_22 = wp::max(var_1, var_6);
        // if A > B:                                                                          <L 625>
        var_23 = (var_1 > var_6);
        if (var_23) {
            // max_grad = grad1_transformed                                                   <L 626>
            var_24 = wp::copy(var_19);
        }
        if (!var_23) {
            // max_grad = grad2                                                               <L 628>
            var_25 = wp::copy(var_14);
        }
        var_26 = wp::where(var_23, var_24, var_25);
        // sign = wp.sign(max_val)                                                            <L 629>
        var_27 = wp::sign(var_22);
        // gradient += max_grad * sign                                                        <L 630>
        var_28 = wp::mul(var_26, var_27);
        var_29 = wp::add(var_21, var_28);
        // return gradient                                                                    <L 631>
        return var_29;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:573
static CUDA_CALLABLE wp::float32 clearance_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<128, wp::float32> var_s1,
    wp::vec_t<128, wp::float32> var_s2,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2)
{
    //---------
    // primal vars
    wp::float32 var_0;
    const wp::int32 var_1 = 8;
    const wp::int32 var_2 = 8;
    wp::float32 var_3;
    wp::float32 var_4;
    wp::float32 var_5;
    wp::float32 var_6;
    wp::float32 var_7;
    wp::float32 var_8;
    //---------
    // forward
    // def clearance(                                                                         <L 574>
    // sdf1 = sdf(type1, p1, s1, sdf_type1, volume_data1, mesh_data1)                         <L 589>
    var_0 = sdf_0(var_type1, var_p1, var_s1, var_sdf_type1, var_volume_data1, var_mesh_data1);
    // sdf2 = sdf(GeomType.SDF, p2, s2, sdf_type2, volume_data2, mesh_data2)                  <L 590>
    var_3 = sdf_0(var_2, var_p2, var_s2, var_sdf_type2, var_volume_data2, var_mesh_data2);
    // if sfd_intersection:                                                                   <L 591>
    if (var_sfd_intersection) {
        // return wp.max(sdf1, sdf2)                                                          <L 592>
        var_4 = wp::max(var_0, var_3);
        return var_4;
    }
    if (!var_sfd_intersection) {
        // return sdf1 + sdf2 + wp.abs(wp.max(sdf1, sdf2))                                    <L 594>
        var_5 = wp::add(var_0, var_3);
        var_6 = wp::max(var_0, var_3);
        var_7 = wp::abs(var_6);
        var_8 = wp::add(var_5, var_7);
        return var_8;
    }
    return {};
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:634
static CUDA_CALLABLE void gradient_step_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_x,
    OptimizationParams_51b33fc1 var_params,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    wp::int32 var_niter,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1)
{
    //---------
    // primal vars
    const wp::float32 var_0 = 0.0001;
    const wp::float32 var_1 = 0.5;
    const wp::float32 var_2 = 0.1;
    const wp::float32 var_3 = 10000000000.0;
    wp::float32 var_4;
    wp::range_t var_5;
    wp::int32 var_6;
    const wp::float32 var_7 = 2.0;
    wp::float32 var_8;
    const wp::int32 var_9 = 0;
    wp::float32 var_10;
    const wp::int32 var_11 = 1;
    wp::float32 var_12;
    const wp::int32 var_13 = 2;
    wp::float32 var_14;
    wp::vec_t<3, wp::float32> var_15;
    wp::mat_t<3, 3, wp::float32>* var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::mat_t<3, 3, wp::float32> var_18;
    wp::vec_t<3, wp::float32>* var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<3, wp::float32> var_21;
    wp::vec_t<3, wp::float32> var_22;
    wp::vec_t<128, wp::float32>* var_23;
    wp::vec_t<128, wp::float32>* var_24;
    wp::float32 var_25;
    wp::vec_t<128, wp::float32> var_26;
    wp::vec_t<128, wp::float32> var_27;
    wp::float32 var_28;
    const wp::float32 var_29 = 1e-12;
    bool var_30;
    const wp::float32 var_31 = -0.1;
    wp::float32 var_32;
    wp::float32 var_33;
    const bool var_34 = true;
    wp::float32 var_35;
    wp::float32 var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    wp::mat_t<3, 3, wp::float32>* var_39;
    wp::vec_t<3, wp::float32> var_40;
    wp::mat_t<3, 3, wp::float32> var_41;
    wp::vec_t<3, wp::float32>* var_42;
    wp::vec_t<3, wp::float32> var_43;
    wp::vec_t<3, wp::float32> var_44;
    wp::vec_t<128, wp::float32>* var_45;
    wp::vec_t<128, wp::float32>* var_46;
    wp::float32 var_47;
    wp::vec_t<128, wp::float32> var_48;
    wp::vec_t<128, wp::float32> var_49;
    bool var_50;
    bool var_51;
    wp::float32 var_52;
    bool var_53;
    wp::vec_t<3, wp::float32> var_54;
    wp::float32 var_55;
    wp::float32 var_56;
    wp::vec_t<3, wp::float32> var_57;
    wp::float32 var_58;
    bool var_59;
    //---------
    // forward
    // def gradient_step(                                                                     <L 635>
    // amin = 1e-4                                                                            <L 649>
    // rho = 0.5                                                                              <L 650>
    // c = 0.1                                                                                <L 651>
    // dist = float(1e10)                                                                     <L 652>
    var_4 = wp::float(var_3);
    // for i in range(niter):                                                                 <L 653>
    var_5 = wp::range(var_niter);
    start_for_0:;
        if (iter_cmp(var_5) == 0) goto end_for_0;
        var_6 = wp::iter_next(var_5);
        // alpha = float(2.0)                                                                 <L 654>
        var_8 = wp::float(var_7);
        // x2 = wp.vec3(x[0], x[1], x[2])                                                     <L 655>
        var_10 = wp::extract(var_x, var_9);
        var_12 = wp::extract(var_x, var_11);
        var_14 = wp::extract(var_x, var_13);
        var_15 = wp::vec_t<3, wp::float32>(var_10, var_12, var_14);
        // x1 = params.rel_mat * x2 + params.rel_pos                                          <L 656>
        var_16 = &((var_params).rel_mat);
        var_18 = wp::load(var_16);
        var_17 = wp::mul(var_18, var_15);
        var_19 = &((var_params).rel_pos);
        var_21 = wp::load(var_19);
        var_20 = wp::add(var_17, var_21);
        // grad = compute_grad(                                                               <L 657>
        // type1, x1, x2, params, sdf_type1, sdf_type2, sfd_intersection, volume_data1, volume_data2, mesh_data1, mesh_data2       <L 658>
        var_22 = compute_grad_0(var_type1, var_20, var_15, var_params, var_sdf_type1, var_sdf_type2, var_sfd_intersection, var_volume_data1, var_volume_data2, var_mesh_data1, var_mesh_data2);
        // dist0 = clearance(                                                                 <L 660>
        // type1,                                                                             <L 661>
        // x1,                                                                                <L 662>
        // x,                                                                                 <L 663>
        // params.attr1,                                                                      <L 664>
        var_23 = &((var_params).attr1);
        // params.attr2,                                                                      <L 665>
        var_24 = &((var_params).attr2);
        // sdf_type1,                                                                         <L 666>
        // sdf_type2,                                                                         <L 667>
        // sfd_intersection,                                                                  <L 668>
        // volume_data1,                                                                      <L 669>
        // volume_data2,                                                                      <L 670>
        // mesh_data1,                                                                        <L 671>
        // mesh_data2,                                                                        <L 672>
        var_26 = wp::load(var_23);
        var_27 = wp::load(var_24);
        var_25 = clearance_0(var_type1, var_20, var_x, var_26, var_27, var_sdf_type1, var_sdf_type2, var_sfd_intersection, var_volume_data1, var_volume_data2, var_mesh_data1, var_mesh_data2);
        // grad_dot = wp.dot(grad, grad)                                                      <L 674>
        var_28 = wp::dot(var_22, var_22);
        // if grad_dot < 1e-12:                                                               <L 675>
        var_30 = (var_28 < var_29);
        if (var_30) {
            // return dist0, x                                                                <L 676>
            ret_0 = var_25;
            ret_1 = var_x;
            return;
        }
        // wolfe = -c * alpha * grad_dot                                                      <L 677>
        var_32 = wp::mul(var_31, var_8);
        var_33 = wp::mul(var_32, var_28);
        // while True:                                                                        <L 678>
    start_while_3:;
    if ((var_34) == false) goto end_while_3;
            // alpha *= rho                                                                   <L 679>
            var_35 = wp::mul(var_8, var_1);
            // wolfe *= rho                                                                   <L 680>
            var_36 = wp::mul(var_33, var_1);
            // x = x2 - grad * alpha                                                          <L 681>
            var_37 = wp::mul(var_22, var_35);
            var_38 = wp::sub(var_15, var_37);
            // x1 = params.rel_mat * x + params.rel_pos                                       <L 682>
            var_39 = &((var_params).rel_mat);
            var_41 = wp::load(var_39);
            var_40 = wp::mul(var_41, var_38);
            var_42 = &((var_params).rel_pos);
            var_44 = wp::load(var_42);
            var_43 = wp::add(var_40, var_44);
            // dist = clearance(                                                              <L 683>
            // type1,                                                                         <L 684>
            // x1,                                                                            <L 685>
            // x,                                                                             <L 686>
            // params.attr1,                                                                  <L 687>
            var_45 = &((var_params).attr1);
            // params.attr2,                                                                  <L 688>
            var_46 = &((var_params).attr2);
            // sdf_type1,                                                                     <L 689>
            // sdf_type2,                                                                     <L 690>
            // sfd_intersection,                                                              <L 691>
            // volume_data1,                                                                  <L 692>
            // volume_data2,                                                                  <L 693>
            // mesh_data1,                                                                    <L 694>
            // mesh_data2,                                                                    <L 695>
            var_48 = wp::load(var_45);
            var_49 = wp::load(var_46);
            var_47 = clearance_0(var_type1, var_43, var_38, var_48, var_49, var_sdf_type1, var_sdf_type2, var_sfd_intersection, var_volume_data1, var_volume_data2, var_mesh_data1, var_mesh_data2);
            // if alpha <= amin or (dist - dist0) <= wolfe:                                   <L 697>
            var_51 = (var_35 <= var_0);
            var_50 = var_51;
            if (!var_50) {
                var_52 = wp::sub(var_47, var_25);
                var_53 = (var_52 <= var_36);
                var_50 = var_50 || var_53;
            }
            if (var_50) {
                // break                                                                      <L 698>
                wp::assign(var_x, var_38);
                wp::assign(var_4, var_47);
                wp::assign(var_8, var_35);
                wp::assign(var_20, var_43);
                wp::assign(var_33, var_36);
                goto end_while_3;
            }
            var_54 = wp::where(var_50, var_x, var_38);
            var_55 = wp::where(var_50, var_4, var_47);
            var_56 = wp::where(var_50, var_8, var_35);
            var_57 = wp::where(var_50, var_20, var_43);
            var_58 = wp::where(var_50, var_33, var_36);
            wp::assign(var_x, var_54);
            wp::assign(var_4, var_55);
            wp::assign(var_8, var_56);
            wp::assign(var_20, var_57);
            wp::assign(var_33, var_58);
    goto start_while_3;
    end_while_3:;
        // if dist > dist0:                                                                   <L 699>
        var_59 = (var_4 > var_25);
        if (var_59) {
            // return dist0, x2                                                               <L 700>
            ret_0 = var_25;
            ret_1 = var_15;
            return;
        }
        goto start_for_0;
    end_for_0:;
    // return dist, x                                                                         <L 701>
    ret_0 = var_4;
    ret_1 = var_x;
    return;
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:704
static CUDA_CALLABLE void gradient_descent_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_x0_initial,
    wp::vec_t<128, wp::float32> var_attr1,
    wp::vec_t<128, wp::float32> var_attr2,
    wp::vec_t<3, wp::float32> var_pos1,
    wp::mat_t<3, 3, wp::float32> var_rot1,
    wp::vec_t<3, wp::float32> var_pos2,
    wp::mat_t<3, 3, wp::float32> var_rot2,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    wp::int32 var_sdf_iterations,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2)
{
    //---------
    // primal vars
    OptimizationParams_51b33fc1 var_0;
    wp::mat_t<3, 3, wp::float32> var_1;
    wp::mat_t<3, 3, wp::float32> var_2;
    wp::mat_t<3, 3, wp::float32> var_3;
    wp::vec_t<3, wp::float32> var_4;
    wp::vec_t<3, wp::float32> var_5;
    const bool var_6 = false;
    wp::float32 var_7;
    wp::vec_t<3, wp::float32> var_8;
    const wp::int32 var_9 = 1;
    const bool var_10 = true;
    wp::float32 var_11;
    wp::vec_t<3, wp::float32> var_12;
    wp::mat_t<3, 3, wp::float32>* var_13;
    wp::vec_t<3, wp::float32> var_14;
    wp::mat_t<3, 3, wp::float32> var_15;
    wp::vec_t<3, wp::float32>* var_16;
    wp::vec_t<3, wp::float32> var_17;
    wp::vec_t<3, wp::float32> var_18;
    wp::vec_t<128, wp::float32>* var_19;
    wp::vec_t<3, wp::float32> var_20;
    wp::vec_t<128, wp::float32> var_21;
    wp::mat_t<3, 3, wp::float32>* var_22;
    wp::mat_t<3, 3, wp::float32> var_23;
    wp::mat_t<3, 3, wp::float32> var_24;
    wp::vec_t<3, wp::float32> var_25;
    wp::vec_t<3, wp::float32> var_26;
    const wp::int32 var_27 = 8;
    const wp::int32 var_28 = 8;
    wp::vec_t<128, wp::float32>* var_29;
    wp::vec_t<3, wp::float32> var_30;
    wp::vec_t<128, wp::float32> var_31;
    wp::vec_t<3, wp::float32> var_32;
    wp::vec_t<3, wp::float32> var_33;
    wp::vec_t<3, wp::float32> var_34;
    wp::vec_t<3, wp::float32> var_35;
    wp::vec_t<3, wp::float32> var_36;
    wp::vec_t<3, wp::float32> var_37;
    wp::vec_t<3, wp::float32> var_38;
    const wp::float32 var_39 = 2.0;
    wp::vec_t<3, wp::float32> var_40;
    wp::vec_t<3, wp::float32> var_41;
    //---------
    // forward
    // def gradient_descent(                                                                  <L 705>
    // params = OptimizationParams()                                                          <L 723>
    var_0 = OptimizationParams_51b33fc1();
    // params.rel_mat = wp.transpose(rot1) * rot2                                             <L 724>
    var_1 = wp::transpose(var_rot1);
    var_2 = wp::mul(var_1, var_rot2);
    var_0.rel_mat = var_2;
    // params.rel_pos = wp.transpose(rot1) * (pos2 - pos1)                                    <L 725>
    var_3 = wp::transpose(var_rot1);
    var_4 = wp::sub(var_pos2, var_pos1);
    var_5 = wp::mul(var_3, var_4);
    var_0.rel_pos = var_5;
    // params.attr1 = attr1                                                                   <L 726>
    var_0.attr1 = var_attr1;
    // params.attr2 = attr2                                                                   <L 727>
    var_0.attr2 = var_attr2;
    // dist, x = gradient_step(                                                               <L 728>
    // type1, x0_initial, params, sdf_type1, sdf_type2, sdf_iterations, False, volume_data1, volume_data2, mesh_data1, mesh_data2       <L 729>
    gradient_step_0(var_type1, var_x0_initial, var_0, var_sdf_type1, var_sdf_type2, var_sdf_iterations, var_6, var_volume_data1, var_volume_data2, var_mesh_data1, var_mesh_data2, var_7, var_8);
    // dist, x = gradient_step(type1, x, params, sdf_type1, sdf_type2, 1, True, volume_data1, volume_data2, mesh_data1, mesh_data2)       <L 731>
    gradient_step_0(var_type1, var_8, var_0, var_sdf_type1, var_sdf_type2, var_9, var_10, var_volume_data1, var_volume_data2, var_mesh_data1, var_mesh_data2, var_11, var_12);
    // x_1 = params.rel_mat * x + params.rel_pos                                              <L 732>
    var_13 = &((var_0).rel_mat);
    var_15 = wp::load(var_13);
    var_14 = wp::mul(var_15, var_12);
    var_16 = &((var_0).rel_pos);
    var_18 = wp::load(var_16);
    var_17 = wp::add(var_14, var_18);
    // grad1 = sdf_grad(type1, x_1, params.attr1, sdf_type1, volume_data1, mesh_data1)        <L 733>
    var_19 = &((var_0).attr1);
    var_21 = wp::load(var_19);
    var_20 = sdf_grad_0(var_type1, var_17, var_21, var_sdf_type1, var_volume_data1, var_mesh_data1);
    // grad1 = wp.transpose(params.rel_mat) * grad1                                           <L 734>
    var_22 = &((var_0).rel_mat);
    var_24 = wp::load(var_22);
    var_23 = wp::transpose(var_24);
    var_25 = wp::mul(var_23, var_20);
    // grad1 = wp.normalize(grad1)                                                            <L 735>
    var_26 = wp::normalize(var_25);
    // grad2 = sdf_grad(GeomType.SDF, x, params.attr2, sdf_type2, volume_data2, mesh_data2)       <L 736>
    var_29 = &((var_0).attr2);
    var_31 = wp::load(var_29);
    var_30 = sdf_grad_0(var_28, var_12, var_31, var_sdf_type2, var_volume_data2, var_mesh_data2);
    // grad2 = wp.normalize(grad2)                                                            <L 737>
    var_32 = wp::normalize(var_30);
    // n = grad1 - grad2                                                                      <L 738>
    var_33 = wp::sub(var_26, var_32);
    // n = wp.normalize(n)                                                                    <L 739>
    var_34 = wp::normalize(var_33);
    // pos = rot2 * x + pos2                                                                  <L 740>
    var_35 = wp::mul(var_rot2, var_12);
    var_36 = wp::add(var_35, var_pos2);
    // n = rot2 * n                                                                           <L 741>
    var_37 = wp::mul(var_rot2, var_34);
    // pos3 = pos - n * dist / 2.0                                                            <L 742>
    var_38 = wp::mul(var_37, var_11);
    var_40 = wp::div(var_38, var_39);
    var_41 = wp::sub(var_36, var_40);
    // return dist, pos3, n                                                                   <L 743>
    ret_0 = var_11;
    ret_1 = var_41;
    ret_2 = var_37;
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
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:131
static CUDA_CALLABLE void adj_transform_aabb_0(
    wp::vec_t<3, wp::float32> var_aabb_pos,
    wp::vec_t<3, wp::float32> var_aabb_size,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_ori,
    wp::vec_t<3, wp::float32> & adj_aabb_pos,
    wp::vec_t<3, wp::float32> & adj_aabb_size,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_ori,
    AABB_886b22fe & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:81
static CUDA_CALLABLE void adj_get_sdf_params_0(
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> var_oct_coeff,
    wp::array_t<wp::int32> var_mesh_octadr,
    wp::array_t<wp::int32> var_plugin,
    wp::array_t<wp::vec_t<128, wp::float32>> var_plugin_attr,
    wp::int32 var_g_type,
    wp::vec_t<3, wp::float32> var_g_size,
    wp::int32 var_plugin_id,
    wp::int32 var_mesh_id,
    wp::vec_t<128, wp::float32> & ret_0,
    wp::int32 & ret_1,
    VolumeData_53ac1a2d & ret_2,
    MeshData_52eaa0fa & ret_3,
    wp::array_t<wp::vec_t<8, wp::int32>> & adj_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> & adj_oct_coeff,
    wp::array_t<wp::int32> & adj_mesh_octadr,
    wp::array_t<wp::int32> & adj_plugin,
    wp::array_t<wp::vec_t<128, wp::float32>> & adj_plugin_attr,
    wp::int32 & adj_g_type,
    wp::vec_t<3, wp::float32> & adj_g_size,
    wp::int32 & adj_plugin_id,
    wp::int32 & adj_mesh_id,
    wp::vec_t<128, wp::float32> & adj_ret_0,
    wp::int32 & adj_ret_1,
    VolumeData_53ac1a2d & adj_ret_2,
    MeshData_52eaa0fa & adj_ret_3)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\util_misc.py:60
static CUDA_CALLABLE void adj_halton_0(
    wp::int32 var_index,
    wp::int32 var_base,
    wp::int32 & adj_index,
    wp::int32 & adj_base,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:158
static CUDA_CALLABLE void adj_sphere_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:187
static CUDA_CALLABLE void adj_capsule_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:196
static CUDA_CALLABLE void adj_cylinder_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_ret)
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


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:148
static CUDA_CALLABLE void adj_radial_field_0(
    wp::vec_t<3, wp::float32> var_a,
    wp::vec_t<3, wp::float32> var_x,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_a,
    wp::vec_t<3, wp::float32> & adj_x,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:163
static CUDA_CALLABLE void adj_box_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:175
static CUDA_CALLABLE void adj_ellipsoid_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:109
static CUDA_CALLABLE void adj__ray_quad_0(
    wp::float32 var_a,
    wp::float32 var_b,
    wp::float32 var_c,
    wp::float32 & ret_0,
    wp::vec_t<2, wp::float32> & ret_1,
    wp::float32 & adj_a,
    wp::float32 & adj_b,
    wp::float32 & adj_c,
    wp::float32 & adj_ret_0,
    wp::vec_t<2, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:241
static CUDA_CALLABLE void adj_ray_sphere_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::float32 var_dist_sqr,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::float32 & adj_dist_sqr,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:36
static CUDA_CALLABLE void adj__ray_map_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_mat,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:424
static CUDA_CALLABLE void adj_ray_box_0(
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<6, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_mat,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::float32 & adj_ret_0,
    wp::vec_t<6, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:132
static CUDA_CALLABLE void adj__orthogonal_basis_0(
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::vec_t<3, wp::float32> & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:158
static CUDA_CALLABLE void adj__ray_triangle_1(
    wp::vec_t<3, wp::float32> var_v0,
    wp::vec_t<3, wp::float32> var_v1,
    wp::vec_t<3, wp::float32> var_v2,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::vec_t<3, wp::float32> var_b0,
    wp::vec_t<3, wp::float32> var_b1,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & adj_v0,
    wp::vec_t<3, wp::float32> & adj_v1,
    wp::vec_t<3, wp::float32> & adj_v2,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::vec_t<3, wp::float32> & adj_b0,
    wp::vec_t<3, wp::float32> & adj_b1,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\ray.py:632
static CUDA_CALLABLE void adj_ray_mesh_0(
    wp::int32 var_nmeshface,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_faceadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::vec_t<3, wp::int32>> var_mesh_face,
    wp::int32 var_data_id,
    wp::vec_t<3, wp::float32> var_pos,
    wp::mat_t<3, 3, wp::float32> var_mat,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> var_pnt,
    wp::vec_t<3, wp::float32> var_vec,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::int32 & adj_nmeshface,
    wp::array_t<wp::int32> & adj_mesh_vertadr,
    wp::array_t<wp::int32> & adj_mesh_faceadr,
    wp::array_t<wp::vec_t<3, wp::float32>> & adj_mesh_vert,
    wp::array_t<wp::vec_t<3, wp::int32>> & adj_mesh_face,
    wp::int32 & adj_data_id,
    wp::vec_t<3, wp::float32> & adj_pos,
    wp::mat_t<3, 3, wp::float32> & adj_mat,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_pnt,
    wp::vec_t<3, wp::float32> & adj_vec,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
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
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
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
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:436
static CUDA_CALLABLE void adj_sample_volume_sdf_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data,
    wp::vec_t<3, wp::float32> & adj_xyz,
    VolumeData_53ac1a2d & adj_volume_data,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:298
static CUDA_CALLABLE void adj_user_sdf_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<128, wp::float32> & adj_attr,
    wp::int32 & adj_sdf_type,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:463
static CUDA_CALLABLE void adj_sdf_0(
    wp::int32 var_type,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    VolumeData_53ac1a2d var_volume_data,
    MeshData_52eaa0fa var_mesh_data,
    wp::int32 & adj_type,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<128, wp::float32> & adj_attr,
    wp::int32 & adj_sdf_type,
    VolumeData_53ac1a2d & adj_volume_data,
    MeshData_52eaa0fa & adj_mesh_data,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:205
static CUDA_CALLABLE void adj_grad_sphere_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:252
static CUDA_CALLABLE void adj_grad_capsule_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:264
static CUDA_CALLABLE void adj_grad_cylinder_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:214
static CUDA_CALLABLE void adj_grad_box_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:232
static CUDA_CALLABLE void adj_grad_ellipsoid_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<3, wp::float32> var_size,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<3, wp::float32> & adj_size,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:443
static CUDA_CALLABLE void adj_sample_volume_grad_0(
    wp::vec_t<3, wp::float32> var_xyz,
    VolumeData_53ac1a2d var_volume_data,
    wp::vec_t<3, wp::float32> & adj_xyz,
    VolumeData_53ac1a2d & adj_volume_data,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:308
static CUDA_CALLABLE void adj_user_sdf_grad_0(
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<128, wp::float32> & adj_attr,
    wp::int32 & adj_sdf_type,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:522
static CUDA_CALLABLE void adj_sdf_grad_0(
    wp::int32 var_type,
    wp::vec_t<3, wp::float32> var_p,
    wp::vec_t<128, wp::float32> var_attr,
    wp::int32 var_sdf_type,
    VolumeData_53ac1a2d var_volume_data,
    MeshData_52eaa0fa var_mesh_data,
    wp::int32 & adj_type,
    wp::vec_t<3, wp::float32> & adj_p,
    wp::vec_t<128, wp::float32> & adj_attr,
    wp::int32 & adj_sdf_type,
    VolumeData_53ac1a2d & adj_volume_data,
    MeshData_52eaa0fa & adj_mesh_data,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:597
static CUDA_CALLABLE void adj_compute_grad_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    OptimizationParams_51b33fc1 var_params,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::int32 & adj_type1,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    OptimizationParams_51b33fc1 & adj_params,
    wp::int32 & adj_sdf_type1,
    wp::int32 & adj_sdf_type2,
    bool & adj_sfd_intersection,
    VolumeData_53ac1a2d & adj_volume_data1,
    VolumeData_53ac1a2d & adj_volume_data2,
    MeshData_52eaa0fa & adj_mesh_data1,
    MeshData_52eaa0fa & adj_mesh_data2,
    wp::vec_t<3, wp::float32> & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:573
static CUDA_CALLABLE void adj_clearance_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_p1,
    wp::vec_t<3, wp::float32> var_p2,
    wp::vec_t<128, wp::float32> var_s1,
    wp::vec_t<128, wp::float32> var_s2,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::int32 & adj_type1,
    wp::vec_t<3, wp::float32> & adj_p1,
    wp::vec_t<3, wp::float32> & adj_p2,
    wp::vec_t<128, wp::float32> & adj_s1,
    wp::vec_t<128, wp::float32> & adj_s2,
    wp::int32 & adj_sdf_type1,
    wp::int32 & adj_sdf_type2,
    bool & adj_sfd_intersection,
    VolumeData_53ac1a2d & adj_volume_data1,
    VolumeData_53ac1a2d & adj_volume_data2,
    MeshData_52eaa0fa & adj_mesh_data1,
    MeshData_52eaa0fa & adj_mesh_data2,
    wp::float32 & adj_ret)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:634
static CUDA_CALLABLE void adj_gradient_step_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_x,
    OptimizationParams_51b33fc1 var_params,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    wp::int32 var_niter,
    bool var_sfd_intersection,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::int32 & adj_type1,
    wp::vec_t<3, wp::float32> & adj_x,
    OptimizationParams_51b33fc1 & adj_params,
    wp::int32 & adj_sdf_type1,
    wp::int32 & adj_sdf_type2,
    wp::int32 & adj_niter,
    bool & adj_sfd_intersection,
    VolumeData_53ac1a2d & adj_volume_data1,
    VolumeData_53ac1a2d & adj_volume_data2,
    MeshData_52eaa0fa & adj_mesh_data1,
    MeshData_52eaa0fa & adj_mesh_data2,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1)
{
	// reverse mode disabled (module option "enable_backward" is False or no dependent kernel found with "enable_backward")
}


// C:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Lib\site-packages\mujoco_warp\_src\collision_sdf.py:704
static CUDA_CALLABLE void adj_gradient_descent_0(
    wp::int32 var_type1,
    wp::vec_t<3, wp::float32> var_x0_initial,
    wp::vec_t<128, wp::float32> var_attr1,
    wp::vec_t<128, wp::float32> var_attr2,
    wp::vec_t<3, wp::float32> var_pos1,
    wp::mat_t<3, 3, wp::float32> var_rot1,
    wp::vec_t<3, wp::float32> var_pos2,
    wp::mat_t<3, 3, wp::float32> var_rot2,
    wp::int32 var_sdf_type1,
    wp::int32 var_sdf_type2,
    wp::int32 var_sdf_iterations,
    VolumeData_53ac1a2d var_volume_data1,
    VolumeData_53ac1a2d var_volume_data2,
    MeshData_52eaa0fa var_mesh_data1,
    MeshData_52eaa0fa var_mesh_data2,
    wp::float32 & ret_0,
    wp::vec_t<3, wp::float32> & ret_1,
    wp::vec_t<3, wp::float32> & ret_2,
    wp::int32 & adj_type1,
    wp::vec_t<3, wp::float32> & adj_x0_initial,
    wp::vec_t<128, wp::float32> & adj_attr1,
    wp::vec_t<128, wp::float32> & adj_attr2,
    wp::vec_t<3, wp::float32> & adj_pos1,
    wp::mat_t<3, 3, wp::float32> & adj_rot1,
    wp::vec_t<3, wp::float32> & adj_pos2,
    wp::mat_t<3, 3, wp::float32> & adj_rot2,
    wp::int32 & adj_sdf_type1,
    wp::int32 & adj_sdf_type2,
    wp::int32 & adj_sdf_iterations,
    VolumeData_53ac1a2d & adj_volume_data1,
    VolumeData_53ac1a2d & adj_volume_data2,
    MeshData_52eaa0fa & adj_mesh_data1,
    MeshData_52eaa0fa & adj_mesh_data2,
    wp::float32 & adj_ret_0,
    wp::vec_t<3, wp::float32> & adj_ret_1,
    wp::vec_t<3, wp::float32> & adj_ret_2)
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



extern "C" __global__ void _sdf_narrowphase_c43bdb62_cuda_kernel_forward(
    wp::launch_bounds_t<2> dim,
    wp::int32 var_nmeshface,
    wp::array_t<wp::vec_t<8, wp::int32>> var_oct_child,
    wp::array_t<wp::vec_t<3, wp::float32>> var_oct_aabb,
    wp::array_t<wp::vec_t<8, wp::float32>> var_oct_coeff,
    wp::array_t<wp::int32> var_geom_type,
    wp::array_t<wp::int32> var_geom_condim,
    wp::array_t<wp::int32> var_geom_dataid,
    wp::array_t<wp::int32> var_geom_priority,
    wp::array_t<wp::float32> var_geom_solmix,
    wp::array_t<wp::vec_t<2, wp::float32>> var_geom_solref,
    wp::array_t<wp::vec_t<5, wp::float32>> var_geom_solimp,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_size,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_aabb,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_friction,
    wp::array_t<wp::float32> var_geom_margin,
    wp::array_t<wp::float32> var_geom_gap,
    wp::array_t<wp::float32> var_geom_adhesion,
    wp::array_t<wp::int32> var_mesh_vertadr,
    wp::array_t<wp::int32> var_mesh_vertnum,
    wp::array_t<wp::int32> var_mesh_faceadr,
    wp::array_t<wp::int32> var_mesh_octadr,
    wp::array_t<wp::int32> var_mesh_graphadr,
    wp::array_t<wp::vec_t<3, wp::float32>> var_mesh_vert,
    wp::array_t<wp::vec_t<3, wp::int32>> var_mesh_face,
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
    wp::array_t<wp::int32> var_plugin,
    wp::array_t<wp::vec_t<128, wp::float32>> var_plugin_attr,
    wp::array_t<wp::int32> var_geom_plugin_index,
    wp::array_t<wp::vec_t<3, wp::float32>> var_geom_xpos_in,
    wp::array_t<wp::mat_t<3, 3, wp::float32>> var_geom_xmat_in,
    wp::int32 var_naconmax_in,
    wp::array_t<wp::int32> var_ncollision_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pair_in,
    wp::array_t<wp::vec_t<2, wp::int32>> var_collision_pairid_in,
    wp::array_t<wp::int32> var_collision_worldid_in,
    wp::int32 var_sdf_initpoints,
    wp::int32 var_sdf_iterations,
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

    const size_t _idx = static_cast<size_t>(blockIdx.z * gridDim.y + blockIdx.y) * static_cast<size_t>(gridDim.x * blockDim.x) + static_cast<size_t>(blockIdx.x * blockDim.x + threadIdx.x);
    if (_idx >= dim.size) return;
            // reset shared memory allocator
    wp::tile_shared_storage_t::init();

        //---------
        // primal vars
        wp::int32 var_0;
        wp::int32 var_1;
        bool var_2;
        const wp::int32 var_3 = 0;
        wp::int32* var_4;
        bool var_5;
        wp::int32 var_6;
        wp::vec_t<2, wp::int32>* var_7;
        wp::vec_t<2, wp::int32> var_8;
        wp::vec_t<2, wp::int32> var_9;
        const wp::int32 var_10 = 1;
        wp::int32 var_11;
        wp::int32* var_12;
        wp::int32 var_13;
        wp::int32 var_14;
        const wp::int32 var_15 = 8;
        bool var_16;
        wp::int32* var_17;
        wp::int32 var_18;
        wp::int32 var_19;
        wp::vec_t<2, wp::int32> var_20;
        wp::float32 var_21;
        wp::float32 var_22;
        wp::int32 var_23;
        wp::vec_t<5, wp::float32> var_24;
        wp::vec_t<2, wp::float32> var_25;
        wp::vec_t<2, wp::float32> var_26;
        wp::vec_t<5, wp::float32> var_27;
        wp::float32 var_28;
        Geom_4022bfa7 var_29;
        Geom_4022bfa7 var_30;
        wp::shape_t* var_31;
        const wp::int32 var_32 = 0;
        wp::int32 var_33;
        wp::shape_t var_34;
        wp::int32 var_35;
        const wp::int32 var_36 = 0;
        wp::int32 var_37;
        wp::int32* var_38;
        wp::int32 var_39;
        wp::int32 var_40;
        wp::int32* var_41;
        wp::int32 var_42;
        wp::int32 var_43;
        wp::int32* var_44;
        wp::int32 var_45;
        wp::int32 var_46;
        wp::mat_t<3, 3, wp::float32>* var_47;
        wp::mat_t<3, 3, wp::float32> var_48;
        wp::mat_t<3, 3, wp::float32> var_49;
        wp::mat_t<3, 3, wp::float32>* var_50;
        wp::mat_t<3, 3, wp::float32> var_51;
        wp::mat_t<3, 3, wp::float32> var_52;
        wp::mat_t<3, 3, wp::float32>* var_53;
        wp::mat_t<3, 3, wp::float32> var_54;
        wp::mat_t<3, 3, wp::float32> var_55;
        wp::vec_t<3, wp::float32>* var_56;
        wp::vec_t<3, wp::float32>* var_57;
        wp::vec_t<3, wp::float32> var_58;
        wp::vec_t<3, wp::float32> var_59;
        wp::vec_t<3, wp::float32> var_60;
        wp::vec_t<3, wp::float32> var_61;
        const wp::int32 var_62 = 0;
        wp::vec_t<3, wp::float32>* var_63;
        wp::vec_t<3, wp::float32> var_64;
        wp::vec_t<3, wp::float32> var_65;
        const wp::int32 var_66 = 1;
        wp::vec_t<3, wp::float32>* var_67;
        wp::vec_t<3, wp::float32> var_68;
        wp::vec_t<3, wp::float32> var_69;
        const wp::int32 var_70 = 3;
        wp::mat_t<3, 3, wp::float32> var_71;
        const wp::float32 var_72 = 0.0;
        wp::vec_t<3, wp::float32> var_73;
        AABB_886b22fe var_74;
        const wp::int32 var_75 = 0;
        wp::vec_t<3, wp::float32>* var_76;
        wp::vec_t<3, wp::float32> var_77;
        wp::vec_t<3, wp::float32> var_78;
        const wp::int32 var_79 = 1;
        wp::vec_t<3, wp::float32>* var_80;
        wp::vec_t<3, wp::float32> var_81;
        wp::vec_t<3, wp::float32> var_82;
        AABB_886b22fe var_83;
        AABB_886b22fe var_84;
        wp::vec_t<3, wp::float32>* var_85;
        wp::vec_t<3, wp::float32>* var_86;
        wp::vec_t<3, wp::float32> var_87;
        wp::vec_t<3, wp::float32> var_88;
        wp::vec_t<3, wp::float32> var_89;
        wp::vec_t<3, wp::float32>* var_90;
        wp::vec_t<3, wp::float32>* var_91;
        wp::vec_t<3, wp::float32> var_92;
        wp::vec_t<3, wp::float32> var_93;
        wp::vec_t<3, wp::float32> var_94;
        bool var_95;
        wp::vec_t<3, wp::float32>* var_96;
        const wp::int32 var_97 = 0;
        wp::float32 var_98;
        wp::vec_t<3, wp::float32> var_99;
        wp::vec_t<3, wp::float32>* var_100;
        const wp::int32 var_101 = 0;
        wp::float32 var_102;
        wp::vec_t<3, wp::float32> var_103;
        bool var_104;
        wp::vec_t<3, wp::float32>* var_105;
        const wp::int32 var_106 = 1;
        wp::float32 var_107;
        wp::vec_t<3, wp::float32> var_108;
        wp::vec_t<3, wp::float32>* var_109;
        const wp::int32 var_110 = 1;
        wp::float32 var_111;
        wp::vec_t<3, wp::float32> var_112;
        bool var_113;
        wp::vec_t<3, wp::float32>* var_114;
        const wp::int32 var_115 = 2;
        wp::float32 var_116;
        wp::vec_t<3, wp::float32> var_117;
        wp::vec_t<3, wp::float32>* var_118;
        const wp::int32 var_119 = 2;
        wp::float32 var_120;
        wp::vec_t<3, wp::float32> var_121;
        bool var_122;
        wp::vec_t<3, wp::float32>* var_123;
        wp::vec_t<3, wp::float32> var_124;
        wp::vec_t<3, wp::float32> var_125;
        wp::mat_t<3, 3, wp::float32>* var_126;
        wp::mat_t<3, 3, wp::float32> var_127;
        wp::mat_t<3, 3, wp::float32> var_128;
        wp::vec_t<3, wp::float32>* var_129;
        wp::vec_t<3, wp::float32> var_130;
        wp::vec_t<3, wp::float32> var_131;
        wp::mat_t<3, 3, wp::float32>* var_132;
        wp::mat_t<3, 3, wp::float32> var_133;
        wp::mat_t<3, 3, wp::float32> var_134;
        wp::shape_t* var_135;
        const wp::int32 var_136 = 0;
        wp::int32 var_137;
        wp::shape_t var_138;
        wp::int32 var_139;
        wp::vec_t<3, wp::float32>* var_140;
        wp::int32* var_141;
        wp::vec_t<128, wp::float32> var_142;
        wp::int32 var_143;
        VolumeData_53ac1a2d var_144;
        MeshData_52eaa0fa var_145;
        wp::vec_t<3, wp::float32> var_146;
        wp::int32 var_147;
        wp::vec_t<3, wp::float32>* var_148;
        wp::int32* var_149;
        wp::vec_t<128, wp::float32> var_150;
        wp::int32 var_151;
        VolumeData_53ac1a2d var_152;
        MeshData_52eaa0fa var_153;
        wp::vec_t<3, wp::float32> var_154;
        wp::int32 var_155;
        wp::int32* var_156;
        wp::int32 var_157;
        wp::int32 var_158;
        wp::vec_t<3, wp::float32>* var_159;
        wp::vec_t<3, wp::float32> var_160;
        wp::vec_t<3, wp::float32> var_161;
        wp::mat_t<3, 3, wp::float32>* var_162;
        wp::mat_t<3, 3, wp::float32> var_163;
        wp::mat_t<3, 3, wp::float32> var_164;
        wp::vec_t<3, wp::float32>* var_165;
        wp::vec_t<3, wp::float32> var_166;
        wp::vec_t<3, wp::float32> var_167;
        const wp::float32 var_168 = -1.0;
        wp::vec_t<3, wp::float32> var_169;
        const wp::float32 var_170 = 0.0;
        wp::vec_t<3, wp::float32> var_171;
        const bool var_172 = true;
        wp::int32* var_173;
        wp::int32 var_174;
        wp::int32 var_175;
        wp::vec_t<3, wp::float32>* var_176;
        wp::vec_t<3, wp::float32> var_177;
        wp::vec_t<3, wp::float32> var_178;
        wp::mat_t<3, 3, wp::float32>* var_179;
        wp::mat_t<3, 3, wp::float32> var_180;
        wp::mat_t<3, 3, wp::float32> var_181;
        wp::vec_t<3, wp::float32>* var_182;
        wp::vec_t<3, wp::float32> var_183;
        wp::vec_t<3, wp::float32> var_184;
        const wp::float32 var_185 = -1.0;
        wp::vec_t<3, wp::float32> var_186;
        const wp::float32 var_187 = 0.0;
        wp::vec_t<3, wp::float32> var_188;
        const bool var_189 = true;
        wp::vec_t<3, wp::float32>* var_190;
        const wp::int32 var_191 = 0;
        wp::float32 var_192;
        wp::vec_t<3, wp::float32> var_193;
        wp::vec_t<3, wp::float32>* var_194;
        const wp::int32 var_195 = 0;
        wp::float32 var_196;
        wp::vec_t<3, wp::float32> var_197;
        wp::vec_t<3, wp::float32>* var_198;
        const wp::int32 var_199 = 0;
        wp::float32 var_200;
        wp::vec_t<3, wp::float32> var_201;
        wp::float32 var_202;
        const wp::int32 var_203 = 2;
        wp::float32 var_204;
        wp::float32 var_205;
        wp::float32 var_206;
        wp::vec_t<3, wp::float32>* var_207;
        const wp::int32 var_208 = 1;
        wp::float32 var_209;
        wp::vec_t<3, wp::float32> var_210;
        wp::vec_t<3, wp::float32>* var_211;
        const wp::int32 var_212 = 1;
        wp::float32 var_213;
        wp::vec_t<3, wp::float32> var_214;
        wp::vec_t<3, wp::float32>* var_215;
        const wp::int32 var_216 = 1;
        wp::float32 var_217;
        wp::vec_t<3, wp::float32> var_218;
        wp::float32 var_219;
        const wp::int32 var_220 = 3;
        wp::float32 var_221;
        wp::float32 var_222;
        wp::float32 var_223;
        wp::vec_t<3, wp::float32>* var_224;
        const wp::int32 var_225 = 2;
        wp::float32 var_226;
        wp::vec_t<3, wp::float32> var_227;
        wp::vec_t<3, wp::float32>* var_228;
        const wp::int32 var_229 = 2;
        wp::float32 var_230;
        wp::vec_t<3, wp::float32> var_231;
        wp::vec_t<3, wp::float32>* var_232;
        const wp::int32 var_233 = 2;
        wp::float32 var_234;
        wp::vec_t<3, wp::float32> var_235;
        wp::float32 var_236;
        const wp::int32 var_237 = 5;
        wp::float32 var_238;
        wp::float32 var_239;
        wp::float32 var_240;
        wp::vec_t<3, wp::float32> var_241;
        wp::mat_t<3, 3, wp::float32>* var_242;
        wp::vec_t<3, wp::float32> var_243;
        wp::mat_t<3, 3, wp::float32> var_244;
        wp::vec_t<3, wp::float32>* var_245;
        wp::vec_t<3, wp::float32> var_246;
        wp::vec_t<3, wp::float32> var_247;
        wp::mat_t<3, 3, wp::float32> var_248;
        wp::vec_t<3, wp::float32> var_249;
        wp::vec_t<3, wp::float32> var_250;
        wp::float32 var_251;
        wp::vec_t<3, wp::float32> var_252;
        wp::vec_t<3, wp::float32> var_253;
        const wp::int32 var_254 = 0;
        wp::mat_t<3, 3, wp::float32> var_255;
        wp::vec_t<2, wp::int32>* var_256;
        wp::int32 var_257;
        wp::vec_t<2, wp::int32> var_258;
        //---------
        // forward
        // def _sdf_narrowphase(                                                                  <L 747>
        // i, contact_tid = wp.tid()                                                              <L 823>
        builtin_tid2d(var_0, var_1);
        // if i >= sdf_initpoints:                                                                <L 824>
        var_2 = (var_0 >= var_sdf_initpoints);
        if (var_2) {
            // return                                                                             <L 825>
            return;
        }
        // if contact_tid >= ncollision_in[0]:                                                    <L 826>
        var_4 = wp::address(var_ncollision_in, var_3);
        var_6 = wp::load(var_4);
        var_5 = (var_1 >= var_6);
        if (var_5) {
            // return                                                                             <L 827>
            return;
        }
        // geoms = collision_pair_in[contact_tid]                                                 <L 828>
        var_7 = wp::address(var_collision_pair_in, var_1);
        var_9 = wp::load(var_7);
        var_8 = wp::copy(var_9);
        // g2 = geoms[1]                                                                          <L 829>
        var_11 = wp::extract(var_8, var_10);
        // type2 = geom_type[g2]                                                                  <L 830>
        var_12 = wp::address(var_geom_type, var_11);
        var_14 = wp::load(var_12);
        var_13 = wp::copy(var_14);
        // if type2 != GeomType.SDF:                                                              <L 831>
        var_16 = (var_13 != var_15);
        if (var_16) {
            // return                                                                             <L 832>
            return;
        }
        // worldid = collision_worldid_in[contact_tid]                                            <L 833>
        var_17 = wp::address(var_collision_worldid_in, var_1);
        var_19 = wp::load(var_17);
        var_18 = wp::copy(var_19);
        // _, margin, gap, condim, friction, solref, solreffriction, solimp, adhesion = contact_params(       <L 834>
        // geom_condim,                                                                           <L 835>
        // geom_priority,                                                                         <L 836>
        // geom_solmix,                                                                           <L 837>
        // geom_solref,                                                                           <L 838>
        // geom_solimp,                                                                           <L 839>
        // geom_friction,                                                                         <L 840>
        // geom_margin,                                                                           <L 841>
        // geom_gap,                                                                              <L 842>
        // geom_adhesion,                                                                         <L 843>
        // pair_dim,                                                                              <L 844>
        // pair_solref,                                                                           <L 845>
        // pair_solreffriction,                                                                   <L 846>
        // pair_solimp,                                                                           <L 847>
        // pair_margin,                                                                           <L 848>
        // pair_gap,                                                                              <L 849>
        // pair_adhesion,                                                                         <L 850>
        // pair_friction,                                                                         <L 851>
        // collision_pair_in,                                                                     <L 852>
        // collision_pairid_in,                                                                   <L 853>
        // contact_tid,                                                                           <L 854>
        // worldid,                                                                               <L 855>
        contact_params_0(var_geom_condim, var_geom_priority, var_geom_solmix, var_geom_solref, var_geom_solimp, var_geom_friction, var_geom_margin, var_geom_gap, var_geom_adhesion, var_pair_dim, var_pair_solref, var_pair_solreffriction, var_pair_solimp, var_pair_margin, var_pair_gap, var_pair_adhesion, var_pair_friction, var_collision_pair_in, var_collision_pairid_in, var_1, var_18, var_20, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28);
        // geom1, geom2 = geom_collision_pair(                                                    <L 858>
        // geom_type,                                                                             <L 859>
        // geom_dataid,                                                                           <L 860>
        // geom_size,                                                                             <L 861>
        // mesh_vertadr,                                                                          <L 862>
        // mesh_vertnum,                                                                          <L 863>
        // mesh_graphadr,                                                                         <L 864>
        // mesh_vert,                                                                             <L 865>
        // mesh_graph,                                                                            <L 866>
        // mesh_polynum,                                                                          <L 867>
        // mesh_polyadr,                                                                          <L 868>
        // mesh_polynormal,                                                                       <L 869>
        // mesh_polyvertadr,                                                                      <L 870>
        // mesh_polyvertnum,                                                                      <L 871>
        // mesh_polyvert,                                                                         <L 872>
        // mesh_polymapadr,                                                                       <L 873>
        // mesh_polymapnum,                                                                       <L 874>
        // mesh_polymap,                                                                          <L 875>
        // geom_xpos_in,                                                                          <L 876>
        // geom_xmat_in,                                                                          <L 877>
        // geoms,                                                                                 <L 878>
        // worldid,                                                                               <L 879>
        geom_collision_pair_0(var_geom_type, var_geom_dataid, var_geom_size, var_mesh_vertadr, var_mesh_vertnum, var_mesh_graphadr, var_mesh_vert, var_mesh_graph, var_mesh_polynum, var_mesh_polyadr, var_mesh_polynormal, var_mesh_polyvertadr, var_mesh_polyvertnum, var_mesh_polyvert, var_mesh_polymapadr, var_mesh_polymapnum, var_mesh_polymap, var_geom_xpos_in, var_geom_xmat_in, var_8, var_18, var_29, var_30);
        // aabb_id = worldid % geom_aabb.shape[0]                                                 <L 882>
        var_31 = &(var_geom_aabb.shape);
        var_34 = wp::load(var_31);
        var_33 = wp::extract(var_34, var_32);
        var_35 = wp::mod(var_18, var_33);
        // g1 = geoms[0]                                                                          <L 883>
        var_37 = wp::extract(var_8, var_36);
        // type1 = geom_type[g1]                                                                  <L 884>
        var_38 = wp::address(var_geom_type, var_37);
        var_40 = wp::load(var_38);
        var_39 = wp::copy(var_40);
        // g1_plugin = geom_plugin_index[g1]                                                      <L 886>
        var_41 = wp::address(var_geom_plugin_index, var_37);
        var_43 = wp::load(var_41);
        var_42 = wp::copy(var_43);
        // g2_plugin = geom_plugin_index[g2]                                                      <L 887>
        var_44 = wp::address(var_geom_plugin_index, var_11);
        var_46 = wp::load(var_44);
        var_45 = wp::copy(var_46);
        // g1_to_g2_rot = wp.transpose(geom1.rot) * geom2.rot                                     <L 889>
        var_47 = &((var_29).rot);
        var_49 = wp::load(var_47);
        var_48 = wp::transpose(var_49);
        var_50 = &((var_30).rot);
        var_52 = wp::load(var_50);
        var_51 = wp::mul(var_48, var_52);
        // g1_to_g2_pos = wp.transpose(geom1.rot) * (geom2.pos - geom1.pos)                       <L 890>
        var_53 = &((var_29).rot);
        var_55 = wp::load(var_53);
        var_54 = wp::transpose(var_55);
        var_56 = &((var_30).pos);
        var_57 = &((var_29).pos);
        var_59 = wp::load(var_56);
        var_60 = wp::load(var_57);
        var_58 = wp::sub(var_59, var_60);
        var_61 = wp::mul(var_54, var_58);
        // aabb_pos = geom_aabb[aabb_id, g1, 0]                                                   <L 891>
        var_63 = wp::address(var_geom_aabb, var_35, var_37, var_62);
        var_65 = wp::load(var_63);
        var_64 = wp::copy(var_65);
        // aabb_size = geom_aabb[aabb_id, g1, 1]                                                  <L 892>
        var_67 = wp::address(var_geom_aabb, var_35, var_37, var_66);
        var_69 = wp::load(var_67);
        var_68 = wp::copy(var_69);
        // identity = wp.identity(3, dtype=float)                                                 <L 893>
        var_71 = wp::identity<3, wp::float32>();
        // aabb1 = transform_aabb(aabb_pos, aabb_size, wp.vec3(0.0), identity)                    <L 894>
        var_73 = wp::vec_t<3, wp::float32>(var_72);
        var_74 = transform_aabb_0(var_64, var_68, var_73, var_71);
        // aabb_pos = geom_aabb[aabb_id, g2, 0]                                                   <L 895>
        var_76 = wp::address(var_geom_aabb, var_35, var_11, var_75);
        var_78 = wp::load(var_76);
        var_77 = wp::copy(var_78);
        // aabb_size = geom_aabb[aabb_id, g2, 1]                                                  <L 896>
        var_80 = wp::address(var_geom_aabb, var_35, var_11, var_79);
        var_82 = wp::load(var_80);
        var_81 = wp::copy(var_82);
        // aabb2 = transform_aabb(aabb_pos, aabb_size, g1_to_g2_pos, g1_to_g2_rot)                <L 897>
        var_83 = transform_aabb_0(var_77, var_81, var_61, var_51);
        // aabb_intersection = AABB()                                                             <L 898>
        var_84 = AABB_886b22fe();
        // aabb_intersection.min = wp.max(aabb1.min, aabb2.min)                                   <L 899>
        var_85 = &((var_74).min);
        var_86 = &((var_83).min);
        var_88 = wp::load(var_85);
        var_89 = wp::load(var_86);
        var_87 = wp::max(var_88, var_89);
        var_84.min = var_87;
        // aabb_intersection.max = wp.min(aabb1.max, aabb2.max)                                   <L 900>
        var_90 = &((var_74).max);
        var_91 = &((var_83).max);
        var_93 = wp::load(var_90);
        var_94 = wp::load(var_91);
        var_92 = wp::min(var_93, var_94);
        var_84.max = var_92;
        // if (                                                                                   <L 902>
        // aabb_intersection.min[0] > aabb_intersection.max[0]                                    <L 903>
        var_96 = &((var_84).min);
        var_99 = wp::load(var_96);
        var_98 = wp::extract(var_99, var_97);
        var_100 = &((var_84).max);
        var_103 = wp::load(var_100);
        var_102 = wp::extract(var_103, var_101);
        var_104 = (var_98 > var_102);
        var_95 = var_104;
        if (!var_95) {
            // or aabb_intersection.min[1] > aabb_intersection.max[1]                             <L 904>
            var_105 = &((var_84).min);
            var_108 = wp::load(var_105);
            var_107 = wp::extract(var_108, var_106);
            var_109 = &((var_84).max);
            var_112 = wp::load(var_109);
            var_111 = wp::extract(var_112, var_110);
            var_113 = (var_107 > var_111);
            var_95 = var_95 || var_113;
        }
        if (!var_95) {
            // or aabb_intersection.min[2] > aabb_intersection.max[2]                             <L 905>
            var_114 = &((var_84).min);
            var_117 = wp::load(var_114);
            var_116 = wp::extract(var_117, var_115);
            var_118 = &((var_84).max);
            var_121 = wp::load(var_118);
            var_120 = wp::extract(var_121, var_119);
            var_122 = (var_116 > var_120);
            var_95 = var_95 || var_122;
        }
        if (var_95) {
            // return                                                                             <L 907>
            return;
        }
        // pos2 = geom2.pos                                                                       <L 909>
        var_123 = &((var_30).pos);
        var_125 = wp::load(var_123);
        var_124 = wp::copy(var_125);
        // rot2 = geom2.rot                                                                       <L 910>
        var_126 = &((var_30).rot);
        var_128 = wp::load(var_126);
        var_127 = wp::copy(var_128);
        // pos1 = geom1.pos                                                                       <L 911>
        var_129 = &((var_29).pos);
        var_131 = wp::load(var_129);
        var_130 = wp::copy(var_131);
        // rot1 = geom1.rot                                                                       <L 912>
        var_132 = &((var_29).rot);
        var_134 = wp::load(var_132);
        var_133 = wp::copy(var_134);
        // dataid_setid = worldid % geom_dataid.shape[0]                                          <L 914>
        var_135 = &(var_geom_dataid.shape);
        var_138 = wp::load(var_135);
        var_137 = wp::extract(var_138, var_136);
        var_139 = wp::mod(var_18, var_137);
        // attr1, g1_plugin_id, volume_data1, mesh_data1 = get_sdf_params(                        <L 916>
        // oct_child,                                                                             <L 917>
        // oct_aabb,                                                                              <L 918>
        // oct_coeff,                                                                             <L 919>
        // mesh_octadr,                                                                           <L 920>
        // plugin,                                                                                <L 921>
        // plugin_attr,                                                                           <L 922>
        // type1,                                                                                 <L 923>
        // geom1.size,                                                                            <L 924>
        var_140 = &((var_29).size);
        // g1_plugin,                                                                             <L 925>
        // geom_dataid[dataid_setid, g1],                                                         <L 926>
        var_141 = wp::address(var_geom_dataid, var_139, var_37);
        var_146 = wp::load(var_140);
        var_147 = wp::load(var_141);
        get_sdf_params_0(var_oct_child, var_oct_aabb, var_oct_coeff, var_mesh_octadr, var_plugin, var_plugin_attr, var_39, var_146, var_42, var_147, var_142, var_143, var_144, var_145);
        // attr2, g2_plugin_id, volume_data2, mesh_data2 = get_sdf_params(                        <L 929>
        // oct_child,                                                                             <L 930>
        // oct_aabb,                                                                              <L 931>
        // oct_coeff,                                                                             <L 932>
        // mesh_octadr,                                                                           <L 933>
        // plugin,                                                                                <L 934>
        // plugin_attr,                                                                           <L 935>
        // type2,                                                                                 <L 936>
        // geom2.size,                                                                            <L 937>
        var_148 = &((var_30).size);
        // g2_plugin,                                                                             <L 938>
        // geom_dataid[dataid_setid, g2],                                                         <L 939>
        var_149 = wp::address(var_geom_dataid, var_139, var_11);
        var_154 = wp::load(var_148);
        var_155 = wp::load(var_149);
        get_sdf_params_0(var_oct_child, var_oct_aabb, var_oct_coeff, var_mesh_octadr, var_plugin, var_plugin_attr, var_13, var_154, var_45, var_155, var_150, var_151, var_152, var_153);
        // mesh_data1.nmeshface = nmeshface                                                       <L 942>
        var_145.nmeshface = var_nmeshface;
        // mesh_data1.mesh_vertadr = mesh_vertadr                                                 <L 943>
        var_145.mesh_vertadr = var_mesh_vertadr;
        // mesh_data1.mesh_vert = mesh_vert                                                       <L 944>
        var_145.mesh_vert = var_mesh_vert;
        // mesh_data1.mesh_faceadr = mesh_faceadr                                                 <L 945>
        var_145.mesh_faceadr = var_mesh_faceadr;
        // mesh_data1.mesh_face = mesh_face                                                       <L 946>
        var_145.mesh_face = var_mesh_face;
        // mesh_data1.data_id = geom_dataid[dataid_setid, g1]                                     <L 947>
        var_156 = wp::address(var_geom_dataid, var_139, var_37);
        var_158 = wp::load(var_156);
        var_157 = wp::copy(var_158);
        var_145.data_id = var_157;
        // mesh_data1.pos = geom1.pos                                                             <L 948>
        var_159 = &((var_29).pos);
        var_161 = wp::load(var_159);
        var_160 = wp::copy(var_161);
        var_145.pos = var_160;
        // mesh_data1.mat = geom1.rot                                                             <L 949>
        var_162 = &((var_29).rot);
        var_164 = wp::load(var_162);
        var_163 = wp::copy(var_164);
        var_145.mat = var_163;
        // mesh_data1.size = geom1.size                                                           <L 950>
        var_165 = &((var_29).size);
        var_167 = wp::load(var_165);
        var_166 = wp::copy(var_167);
        var_145.size = var_166;
        // mesh_data1.pnt = wp.vec3(-1.0)                                                         <L 951>
        var_169 = wp::vec_t<3, wp::float32>(var_168);
        var_145.pnt = var_169;
        // mesh_data1.vec = wp.vec3(0.0)                                                          <L 952>
        var_171 = wp::vec_t<3, wp::float32>(var_170);
        var_145.vec = var_171;
        // mesh_data1.valid = True                                                                <L 953>
        var_145.valid = var_172;
        // mesh_data2.nmeshface = nmeshface                                                       <L 955>
        var_153.nmeshface = var_nmeshface;
        // mesh_data2.mesh_vertadr = mesh_vertadr                                                 <L 956>
        var_153.mesh_vertadr = var_mesh_vertadr;
        // mesh_data2.mesh_vert = mesh_vert                                                       <L 957>
        var_153.mesh_vert = var_mesh_vert;
        // mesh_data2.mesh_faceadr = mesh_faceadr                                                 <L 958>
        var_153.mesh_faceadr = var_mesh_faceadr;
        // mesh_data2.mesh_face = mesh_face                                                       <L 959>
        var_153.mesh_face = var_mesh_face;
        // mesh_data2.data_id = geom_dataid[dataid_setid, g2]                                     <L 960>
        var_173 = wp::address(var_geom_dataid, var_139, var_11);
        var_175 = wp::load(var_173);
        var_174 = wp::copy(var_175);
        var_153.data_id = var_174;
        // mesh_data2.pos = geom2.pos                                                             <L 961>
        var_176 = &((var_30).pos);
        var_178 = wp::load(var_176);
        var_177 = wp::copy(var_178);
        var_153.pos = var_177;
        // mesh_data2.mat = geom2.rot                                                             <L 962>
        var_179 = &((var_30).rot);
        var_181 = wp::load(var_179);
        var_180 = wp::copy(var_181);
        var_153.mat = var_180;
        // mesh_data2.size = geom2.size                                                           <L 963>
        var_182 = &((var_30).size);
        var_184 = wp::load(var_182);
        var_183 = wp::copy(var_184);
        var_153.size = var_183;
        // mesh_data2.pnt = wp.vec3(-1.0)                                                         <L 964>
        var_186 = wp::vec_t<3, wp::float32>(var_185);
        var_153.pnt = var_186;
        // mesh_data2.vec = wp.vec3(0.0)                                                          <L 965>
        var_188 = wp::vec_t<3, wp::float32>(var_187);
        var_153.vec = var_188;
        // mesh_data2.valid = True                                                                <L 966>
        var_153.valid = var_189;
        // x_g2 = wp.vec3(                                                                        <L 968>
        // aabb_intersection.min[0] + (aabb_intersection.max[0] - aabb_intersection.min[0]) * halton(i, 2),       <L 969>
        var_190 = &((var_84).min);
        var_193 = wp::load(var_190);
        var_192 = wp::extract(var_193, var_191);
        var_194 = &((var_84).max);
        var_197 = wp::load(var_194);
        var_196 = wp::extract(var_197, var_195);
        var_198 = &((var_84).min);
        var_201 = wp::load(var_198);
        var_200 = wp::extract(var_201, var_199);
        var_202 = wp::sub(var_196, var_200);
        var_204 = halton_0(var_0, var_203);
        var_205 = wp::mul(var_202, var_204);
        var_206 = wp::add(var_192, var_205);
        // aabb_intersection.min[1] + (aabb_intersection.max[1] - aabb_intersection.min[1]) * halton(i, 3),       <L 970>
        var_207 = &((var_84).min);
        var_210 = wp::load(var_207);
        var_209 = wp::extract(var_210, var_208);
        var_211 = &((var_84).max);
        var_214 = wp::load(var_211);
        var_213 = wp::extract(var_214, var_212);
        var_215 = &((var_84).min);
        var_218 = wp::load(var_215);
        var_217 = wp::extract(var_218, var_216);
        var_219 = wp::sub(var_213, var_217);
        var_221 = halton_0(var_0, var_220);
        var_222 = wp::mul(var_219, var_221);
        var_223 = wp::add(var_209, var_222);
        // aabb_intersection.min[2] + (aabb_intersection.max[2] - aabb_intersection.min[2]) * halton(i, 5),       <L 971>
        var_224 = &((var_84).min);
        var_227 = wp::load(var_224);
        var_226 = wp::extract(var_227, var_225);
        var_228 = &((var_84).max);
        var_231 = wp::load(var_228);
        var_230 = wp::extract(var_231, var_229);
        var_232 = &((var_84).min);
        var_235 = wp::load(var_232);
        var_234 = wp::extract(var_235, var_233);
        var_236 = wp::sub(var_230, var_234);
        var_238 = halton_0(var_0, var_237);
        var_239 = wp::mul(var_236, var_238);
        var_240 = wp::add(var_226, var_239);
        var_241 = wp::vec_t<3, wp::float32>(var_206, var_223, var_240);
        // x = geom1.rot * x_g2 + geom1.pos                                                       <L 973>
        var_242 = &((var_29).rot);
        var_244 = wp::load(var_242);
        var_243 = wp::mul(var_244, var_241);
        var_245 = &((var_29).pos);
        var_247 = wp::load(var_245);
        var_246 = wp::add(var_243, var_247);
        // x0_initial = wp.transpose(rot2) * (x - pos2)                                           <L 974>
        var_248 = wp::transpose(var_127);
        var_249 = wp::sub(var_246, var_124);
        var_250 = wp::mul(var_248, var_249);
        // dist, pos, n = gradient_descent(                                                       <L 975>
        // type1,                                                                                 <L 976>
        // x0_initial,                                                                            <L 977>
        // attr1,                                                                                 <L 978>
        // attr2,                                                                                 <L 979>
        // pos1,                                                                                  <L 980>
        // rot1,                                                                                  <L 981>
        // pos2,                                                                                  <L 982>
        // rot2,                                                                                  <L 983>
        // g1_plugin_id,                                                                          <L 984>
        // g2_plugin_id,                                                                          <L 985>
        // sdf_iterations,                                                                        <L 986>
        // volume_data1,                                                                          <L 987>
        // volume_data2,                                                                          <L 988>
        // mesh_data1,                                                                            <L 989>
        // mesh_data2,                                                                            <L 990>
        gradient_descent_0(var_39, var_250, var_142, var_150, var_130, var_133, var_124, var_127, var_143, var_151, var_sdf_iterations, var_144, var_152, var_145, var_153, var_251, var_252, var_253);
        // write_contact(                                                                         <L 992>
        // naconmax_in,                                                                           <L 993>
        // 0,                                                                                     <L 994>
        // dist,                                                                                  <L 995>
        // pos,                                                                                   <L 996>
        // make_frame(n),                                                                         <L 997>
        var_255 = make_frame_0(var_253);
        // margin,                                                                                <L 998>
        // gap,                                                                                   <L 999>
        // condim,                                                                                <L 1000>
        // friction,                                                                              <L 1001>
        // solref,                                                                                <L 1002>
        // solreffriction,                                                                        <L 1003>
        // solimp,                                                                                <L 1004>
        // adhesion,                                                                              <L 1005>
        // geoms,                                                                                 <L 1006>
        // collision_pairid_in[contact_tid],                                                      <L 1007>
        var_256 = wp::address(var_collision_pairid_in, var_1);
        // worldid,                                                                               <L 1008>
        // contact_dist_out,                                                                      <L 1009>
        // contact_pos_out,                                                                       <L 1010>
        // contact_frame_out,                                                                     <L 1011>
        // contact_includemargin_out,                                                             <L 1012>
        // contact_friction_out,                                                                  <L 1013>
        // contact_solref_out,                                                                    <L 1014>
        // contact_solreffriction_out,                                                            <L 1015>
        // contact_solimp_out,                                                                    <L 1016>
        // contact_dim_out,                                                                       <L 1017>
        // contact_geom_out,                                                                      <L 1018>
        // contact_efc_address_out,                                                               <L 1019>
        // contact_worldid_out,                                                                   <L 1020>
        // contact_type_out,                                                                      <L 1021>
        // contact_geomcollisionid_out,                                                           <L 1022>
        // contact_adhesion_out,                                                                  <L 1023>
        // nacon_out,                                                                             <L 1024>
        var_258 = wp::load(var_256);
        var_257 = write_contact_0(var_naconmax_in, var_254, var_251, var_252, var_255, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_8, var_258, var_18, var_contact_dist_out, var_contact_pos_out, var_contact_frame_out, var_contact_includemargin_out, var_contact_friction_out, var_contact_solref_out, var_contact_solreffriction_out, var_contact_solimp_out, var_contact_dim_out, var_contact_geom_out, var_contact_efc_address_out, var_contact_worldid_out, var_contact_type_out, var_contact_geomcollisionid_out, var_contact_adhesion_out, var_nacon_out);
}

