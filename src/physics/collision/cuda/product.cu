// G10@P5 产品侧 CUDA 入口（NVRTC 运行时编译）。
//
// adapted from G5 独立探针 probe.cu（同 frozen 上游 revision 71da24d），Apache-2.0，
// 产品入口与探针共用同一批设备函数。
//
// 本文件只含产品入口需要的设备侧参数结构与三个 kernel：
//   - `mjwarp_collision_args`（原 `ProbeArgs`；字段名/顺序/类型完全不变）
//   - `mjwarp_collision_contact` / `mjwarp_collision_hfield` / `mjwarp_collision_sdf`
//     （原 `kernel_contact` / `kernel_hfield` / `kernel_sdf`；仅加 `extern "C"`）
// 辅助函数 `init_polytope` / `build_geom` / `build_params` / `sdf_type_of` 与
// `LocalPolytope` 原样保留，不改数值语义。
//
// 设备函数来自 `src/physics/collision/cuda/*.cu`（common→geom→support→simplex→
// gjk_core→polytope→polytope_build→epa→ccd→sdf_octree→sdf_math→sdf_narrowphase→
// hfield→contact），由 `runtime::collision_gpu::source` 按依赖顺序拼接，本文件最后
// 追加，故此处不再 `#include`；`d_contact_out` 由 `contact.cu` 定义，拼接后可见。
//
// 编译选项对齐 G5 探针的 nvcc `-O3 -arch=sm_89 --fmad=true -std=c++17`（无 fast-math）；
// NVRTC 的默认优化级别即 -O3。

// 设备侧指针集合（kernel 参数按值传递）。
struct mjwarp_collision_args {
  const float* geom_xpos;
  const float* geom_xmat;
  const float* geom_size;
  const float* geom_margin;
  const float* geom_gap;
  const float* geom_rbound;
  const float* geom_aabb;
  const int* geom_type;
  const int* geom_dataid;
  const int* mesh_vertadr;
  const int* mesh_vertnum;
  const int* mesh_graphadr;
  const int* mesh_octadr;
  const float* mesh_vert;
  const int* mesh_graph;
  const float* hfield_size;
  const int* hfield_nrow;
  const int* hfield_ncol;
  const int* hfield_adr;
  const float* hfield_data;
  const float* oct_aabb;
  const int* oct_child;
  const float* oct_coeff;

  int nworld;
  int ngeom;
  int g1;
  int g2;
  int type1;
  int type2;
  int geomgeomid;
  int naccdmax;
  int hfield_id;
  int oct_root;
  int noct;
  float tolerance;
  float margin;
  float gap;
  int gjk_iterations;
  int epa_iterations;
  int condim;
  int pairid0;
  int pairid1;
  float friction[5];
  float solref[2];
  float solreffriction[2];
  float solimp[5];
  float adhesion;
  int sdf_initpoints;
  int sdf_iterations;
  int naconmax;
  int efc_width;

  d_contact_out out;
  int* nacon;
  int* nccd;
  unsigned* overflow;
};

using LocalVert = v3;
constexpr int kEpaCap = 64;
// 上游 `types.MJ_MAX_EPAHORIZON`（host `gjk::MJ_MAX_EPAHORIZON = 24`）。
constexpr int kMaxHorizon = 24;

struct LocalPolytope {
  LocalVert vert[2 * (10 + 2 * kEpaCap)];
  int vert_index[2 * (10 + 2 * kEpaCap)];
  unsigned face[6 + 5 * kEpaCap];
  v3 face_pr[6 + 5 * kEpaCap];
  float face_norm2[6 + 5 * kEpaCap];
  int horizon[kMaxHorizon];
  d_polytope pt;
};

__device__ __forceinline__ void init_polytope(LocalPolytope* local) {
  local->pt.nvert = 0;
  local->pt.nface = 0;
  local->pt.nhorizon = 0;
  local->pt.status = 0;
  local->pt.vert = local->vert;
  local->pt.vert_index = local->vert_index;
  local->pt.face = local->face;
  local->pt.face_pr = local->face_pr;
  local->pt.face_norm2 = local->face_norm2;
  local->pt.horizon = local->horizon;
  local->pt.center = v3_make(0.0f, 0.0f, 0.0f);
  local->pt.max_face = 6 + 5 * kEpaCap;
  local->pt.max_horizon = kMaxHorizon;
}

__device__ __forceinline__ d_geom build_geom(const mjwarp_collision_args& a, int world,
                                            int geom_index) {
  d_geom geom;
  const int base = (world * a.ngeom + geom_index) * 3;
  geom.pos = v3_make(a.geom_xpos[base], a.geom_xpos[base + 1], a.geom_xpos[base + 2]);
  const int rot_base = (world * a.ngeom + geom_index) * 9;
  for (int index = 0; index < 9; ++index) geom.rot[index] = a.geom_xmat[rot_base + index];
  geom.size = v3_make(a.geom_size[geom_index * 3], a.geom_size[geom_index * 3 + 1],
                      a.geom_size[geom_index * 3 + 2]);
  geom.margin = 0.0f;
  for (int index = 0; index < 18; ++index) geom.polyvert[index] = 0.0f;
  geom.vertadr = -1;
  geom.vertnum = -1;
  geom.graphadr = -1;
  geom.index = -1;
  geom.mesh.vert = nullptr;
  geom.mesh.graph = nullptr;
  const int type = a.geom_type[geom_index];
  if (type == G5_MESH) {
    const int dataid = a.geom_dataid[geom_index];
    geom.vertadr = dataid >= 0 ? a.mesh_vertadr[dataid] : -1;
    geom.vertnum = dataid >= 0 ? a.mesh_vertnum[dataid] : -1;
    geom.graphadr = dataid >= 0 ? a.mesh_graphadr[dataid] : -1;
    geom.mesh.vert = a.mesh_vert;
    geom.mesh.graph = a.mesh_graph;
  }
  return geom;
}

__device__ __forceinline__ d_contact_params build_params(const mjwarp_collision_args& a) {
  d_contact_params params;
  params.margin = a.margin;
  params.gap = a.gap;
  params.condim = a.condim;
  for (int index = 0; index < 5; ++index) {
    params.friction[index] = a.friction[index];
    params.solimp[index] = a.solimp[index];
  }
  for (int index = 0; index < 2; ++index) {
    params.solref[index] = a.solref[index];
    params.solreffriction[index] = a.solreffriction[index];
  }
  params.adhesion = a.adhesion;
  params.geoms[0] = a.g1;
  params.geoms[1] = a.g2;
  params.pairid[0] = a.pairid0;
  params.pairid[1] = a.pairid1;
  params.worldid = 0;
  return params;
}

__device__ __forceinline__ int sdf_type_of(int geom_type) {
  switch (geom_type) {
    case G5_PLANE: return G5_SDF_PLANE;
    case G5_SPHERE: return G5_SDF_SPHERE;
    case G5_CAPSULE: return G5_SDF_CAPSULE;
    case G5_CYLINDER: return G5_SDF_CYLINDER;
    case G5_BOX: return G5_SDF_BOX;
    case G5_ELLIPSOID: return G5_SDF_ELLIPSOID;
    case G5_MESH: return G5_SDF_MESH;
    case G5_SDF: return G5_SDF_SDF;
    default: return G5_SDF_PLANE;
  }
}

// 上游 `eval_ccd_write_contact` 设备路径（contact kind；含 multiccd 未实装的限制）。
extern "C" __global__ void mjwarp_collision_contact(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  LocalPolytope local;
  init_polytope(&local);

  d_contact_params params = build_params(a);
  params.worldid = world;

  d_contact_pair_input in;
  in.geom1 = build_geom(a, world, a.g1);
  in.geom2 = build_geom(a, world, a.g2);
  in.geomtype1 = a.type1;
  in.geomtype2 = a.type2;
  in.tolerance = a.tolerance;
  in.gjk_iterations = a.gjk_iterations;
  in.epa_iterations = a.epa_iterations;
  in.collision_sensor = false;
  in.naccdmax = a.naccdmax;
  in.geomgeomid = a.geomgeomid;
  in.overflow_out = a.overflow;
  g5_ccd_pair_write(in, params, a.naconmax, false, a.nccd, &local.pt, &a.out);
}

// hfield 设备路径（上游 `ccd_hfield_kernel`）。
extern "C" __global__ void mjwarp_collision_hfield(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  LocalPolytope local;
  init_polytope(&local);

  d_contact_params params = build_params(a);
  params.worldid = world;

  d_hfield_data hfield;
  const int hf = a.hfield_id;
  for (int index = 0; index < 4; ++index) hfield.size[index] = a.hfield_size[hf * 4 + index];
  hfield.nrow = a.hfield_nrow[hf];
  hfield.ncol = a.hfield_ncol[hf];
  hfield.adr = a.hfield_adr[hf];
  hfield.data = a.hfield_data;

  const int pos_base = (world * a.ngeom + a.g1) * 3;
  const v3 hf_pos = v3_make(a.geom_xpos[pos_base], a.geom_xpos[pos_base + 1],
                            a.geom_xpos[pos_base + 2]);
  float hf_mat[9];
  const int rot_base = (world * a.ngeom + a.g1) * 9;
  for (int index = 0; index < 9; ++index) hf_mat[index] = a.geom_xmat[rot_base + index];

  d_geom geom2 = build_geom(a, world, a.g2);
  d_ccd_options options{a.tolerance, 0.0f, a.gjk_iterations, a.epa_iterations};
  const float other_rbound = a.geom_rbound[a.g2];
  d_hfield_result result;
  g5_hfield_pair_contacts(hfield, hf_pos, hf_mat, geom2, a.type2, other_rbound, a.margin, a.gap,
                          options, &local.pt, &result);
  g5_hfield_write_contacts(result, params, a.naconmax, a.overflow, &a.out);
}

// SDF 设备路径（上游 `_sdf_narrowphase` 的单 pair 子集：每初始点一次梯度下降 + 写接触）。
extern "C" __global__ void mjwarp_collision_sdf(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  d_contact_params params = build_params(a);
  params.worldid = world;

  d_volume_data volume;
  const int root = a.oct_root;
  volume.center = v3_make(a.oct_aabb[root * 6], a.oct_aabb[root * 6 + 1], a.oct_aabb[root * 6 + 2]);
  volume.half_size = v3_make(a.oct_aabb[root * 6 + 3], a.oct_aabb[root * 6 + 4],
                             a.oct_aabb[root * 6 + 5]);
  volume.oct_aabb = a.oct_aabb;
  volume.oct_child = a.oct_child;
  volume.oct_coeff = a.oct_coeff;
  volume.root = root;
  volume.noct = a.noct;

  d_sdf_shape shape1;
  shape1.geom = sdf_type_of(a.type1);
  shape1.size = v3_make(a.geom_size[a.g1 * 3], a.geom_size[a.g1 * 3 + 1],
                        a.geom_size[a.g1 * 3 + 2]);
  shape1.volume = nullptr;
  shape1.has_volume = false;

  d_sdf_shape shape2;
  shape2.geom = sdf_type_of(a.type2);
  shape2.size = v3_make(a.geom_size[a.g2 * 3], a.geom_size[a.g2 * 3 + 1],
                        a.geom_size[a.g2 * 3 + 2]);
  shape2.volume = &volume;
  shape2.has_volume = true;

  d_sdf_pair pair;
  pair.shape1 = shape1;
  pair.shape2 = shape2;
  const int p1 = (world * a.ngeom + a.g1) * 3;
  const int p2 = (world * a.ngeom + a.g2) * 3;
  pair.pos1 = v3_make(a.geom_xpos[p1], a.geom_xpos[p1 + 1], a.geom_xpos[p1 + 2]);
  pair.pos2 = v3_make(a.geom_xpos[p2], a.geom_xpos[p2 + 1], a.geom_xpos[p2 + 2]);
  const int r1 = (world * a.ngeom + a.g1) * 9;
  const int r2 = (world * a.ngeom + a.g2) * 9;
  for (int index = 0; index < 9; ++index) {
    pair.rot1[index] = a.geom_xmat[r1 + index];
    pair.rot2[index] = a.geom_xmat[r2 + index];
  }
  for (int index = 0; index < 6; ++index) {
    pair.aabb1[index] = a.geom_aabb[a.g1 * 6 + index];
    pair.aabb2[index] = a.geom_aabb[a.g2 * 6 + index];
  }
  pair.sdf_initpoints = a.sdf_initpoints;
  pair.sdf_iterations = a.sdf_iterations;

  d_sdf_contact contacts[64];
  const int capacity = a.sdf_initpoints < 64 ? a.sdf_initpoints : 64;
  const int count = g5_sdf_pair_contacts(&pair, contacts, capacity);
  for (int index = 0; index < count; ++index) {
    g5_write_contact(params, a.naconmax, 0, contacts[index].dist, contacts[index].pos,
                     contacts[index].frame, &a.out);
  }
}

// primitive narrowphase 设备路径（上游 `primitive_narrowphase` 的 dispatch 子集）：
// 只实现必做的 (SPHERE, BOX) 与 (CAPSULE, BOX)（兼容几何顺序互换）；其它组合不写接触。
extern "C" __global__ void mjwarp_collision_primitive(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  d_contact_params params = build_params(a);
  params.worldid = world;

  const d_geom geom1 = build_geom(a, world, a.g1);
  const d_geom geom2 = build_geom(a, world, a.g2);

  if (a.type1 == G5_SPHERE && a.type2 == G5_BOX) {
    g5_sphere_box_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (a.type1 == G5_BOX && a.type2 == G5_SPHERE) {
    g5_sphere_box_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (a.type1 == G5_CAPSULE && a.type2 == G5_BOX) {
    g5_capsule_box_wrapper(geom1, geom2, params, a.naconmax, &a.out);
  } else if (a.type1 == G5_BOX && a.type2 == G5_CAPSULE) {
    g5_capsule_box_wrapper(geom2, geom1, params, a.naconmax, &a.out);
  } else if (a.type1 == G5_PLANE || a.type2 == G5_PLANE) {
    // B1 plane 族：pinned `_PRIMITIVE_COLLISIONS` 的 6 组合（PLANE+SPHERE/CAPSULE/ELLIPSOID/
    // CYLINDER/BOX/MESH）；wrapper 第一个参数恒为 plane。mesh 只用 mesh 顶点/图（a.mesh_vert /
    // a.mesh_graph），不需要新 ABI 字段。写入门/顺序由各 wrapper 按 pinned 逐行实现，不在此加特判。
    const bool plane_first = a.type1 == G5_PLANE;
    const d_geom& plane = plane_first ? geom1 : geom2;
    const d_geom& other = plane_first ? geom2 : geom1;
    const int other_type = plane_first ? a.type2 : a.type1;
    if (other_type == G5_SPHERE) {
      g5_plane_sphere_wrapper(plane, other, params, a.naconmax, &a.out);
    } else if (other_type == G5_CAPSULE) {
      g5_plane_capsule_wrapper(plane, other, params, a.naconmax, &a.out);
    } else if (other_type == G5_ELLIPSOID) {
      g5_plane_ellipsoid_wrapper(plane, other, params, a.naconmax, &a.out);
    } else if (other_type == G5_CYLINDER) {
      g5_plane_cylinder_wrapper(plane, other, params, a.naconmax, &a.out);
    } else if (other_type == G5_BOX) {
      g5_plane_box_wrapper(plane, other, params, a.naconmax, &a.out);
    } else if (other_type == G5_MESH) {
      const g5_plane_mesh_view mesh = {a.mesh_vert, a.mesh_graph};
      g5_plane_convex_wrapper(plane, other, mesh, params, a.naconmax, &a.out);
    }
  }
}

// multicontact 设备路径：上游 `eval_ccd_write_contact` 的 `use_multiccd=true` 静态特化
// （box/mesh/cylinder 组合可生成 <=4 个接触；mesh 分支本片不实现，回退单接触）。
extern "C" __global__ void mjwarp_collision_contact_multi(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  LocalPolytope local;
  init_polytope(&local);

  d_contact_params params = build_params(a);
  params.worldid = world;

  d_contact_pair_input in;
  in.geom1 = build_geom(a, world, a.g1);
  in.geom2 = build_geom(a, world, a.g2);
  in.geomtype1 = a.type1;
  in.geomtype2 = a.type2;
  in.tolerance = a.tolerance;
  in.gjk_iterations = a.gjk_iterations;
  in.epa_iterations = a.epa_iterations;
  in.collision_sensor = false;
  in.naccdmax = a.naccdmax;
  in.geomgeomid = a.geomgeomid;
  in.overflow_out = a.overflow;
  g5_ccd_pair_write(in, params, a.naconmax, true, a.nccd, &local.pt, &a.out);
}

// primitive box-box 设备路径：上游 `primitive_narrowphase` 在 NATIVECCD 关闭时把
// `(BOX, BOX)` 改走 `box_box_wrapper`（8 个候选经 `detected` 过滤后写入，<=8 行）。
extern "C" __global__ void mjwarp_collision_boxbox(mjwarp_collision_args a) {
  const int world = blockIdx.x;
  if (world >= a.nworld) return;
  d_contact_params params = build_params(a);
  params.worldid = world;

  const d_geom geom1 = build_geom(a, world, a.g1);
  const d_geom geom2 = build_geom(a, world, a.g2);
  g5_box_box_wrapper(geom1, geom2, params, a.naconmax, &a.out);
}
