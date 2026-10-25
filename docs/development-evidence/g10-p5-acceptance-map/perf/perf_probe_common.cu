// G6-2 性能探针 · 冻结 G5 探针源第 1-426 行（设备 include/结构体/kernel；authoring-only，不进产品构建）。
// 来源：docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu（sha256=4ad47bce54fa8f3c257f851fae12c8950d3d9d35d4d5ad3c71ef31daf784d6e2）。
// 由 split_perf_probe.ps1 机械拆分生成；除 perf_probe.cu 的 --perf 分支外未改内核与加载逻辑。

// G5 GPU 窗口 · nvcc 独立 GPU 探针（authoring-only，不进产品构建）。
//
// 目的：在 GPU 窗口内用**同一批 device 源**（`src/physics/collision/cuda/*.cu`，
// `#include` 而非复制）跑 contact / hfield / SDF 三条设备路径，把接触数组落盘，
// 由 Rust 侧（`tests/g10_p5_gpu_kernels.rs`）与上游 CUDA 参考逐字段比对。
//
// 编译：由 `build_probe.ps1` 调 nvcc，`-fmad=true`（默认）且加
// `--fmad=true` 显式声明，禁止 fast-math，`-O3`，`-arch=sm_89`。
// 输入：`cases/<case>/meta.txt` + `*.bin`（由 `prepare_cases.py` 生成）。
// 输出：`cases/<case>/out1/`、`out2/` 二进制 + `probe-report.txt`。
//
// 说明：本探针只是「本窗口验收用的独立 GPU 探针」，不是产品 NVRTC 入口；
// 产品 launcher（NVRTC 编译入口 + 批量 launch/回读）单列后续小片（G5 unmet）。

#include <cuda_runtime.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <chrono>
#include <fstream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#include "contact.cu"
#include "sdf_narrowphase.cu"

#define PROBE_CHECK(expr)                                                          \
  do {                                                                             \
    cudaError_t status = (expr);                                                   \
    if (status != cudaSuccess) {                                                   \
      std::fprintf(stderr, "CUDA error %s at %s:%d\n", cudaGetErrorString(status), \
                   __FILE__, __LINE__);                                            \
      std::exit(3);                                                                \
    }                                                                              \
  } while (0)

namespace {

struct CaseMeta {
  std::map<std::string, std::string> values;

  const std::string& raw(const std::string& key) const {
    auto it = values.find(key);
    if (it == values.end()) {
      std::fprintf(stderr, "meta missing key: %s\n", key.c_str());
      std::exit(2);
    }
    return it->second;
  }
  int i32(const std::string& key) const { return std::atoi(raw(key).c_str()); }
  float f32(const std::string& key) const { return std::strtof(raw(key).c_str(), nullptr); }
  bool has(const std::string& key) const { return values.count(key) != 0; }
  std::vector<float> f32s(const std::string& key) const {
    std::vector<float> out;
    std::istringstream stream(raw(key));
    float value = 0.0f;
    while (stream >> value) out.push_back(value);
    return out;
  }
  std::string text(const std::string& key) const { return raw(key); }
};

CaseMeta read_meta(const std::string& directory) {
  CaseMeta meta;
  std::ifstream input(directory + "/meta.txt");
  if (!input) {
    std::fprintf(stderr, "cannot open %s/meta.txt\n", directory.c_str());
    std::exit(2);
  }
  std::string line;
  while (std::getline(input, line)) {
    if (line.empty()) continue;
    const auto space = line.find(' ');
    if (space == std::string::npos) {
      meta.values[line] = "";
    } else {
      meta.values[line.substr(0, space)] = line.substr(space + 1);
    }
  }
  return meta;
}

std::vector<unsigned char> read_file(const std::string& path) {
  std::ifstream input(path, std::ios::binary);
  if (!input) {
    std::fprintf(stderr, "cannot open %s\n", path.c_str());
    std::exit(2);
  }
  return std::vector<unsigned char>((std::istreambuf_iterator<char>(input)),
                                    std::istreambuf_iterator<char>());
}

template <typename T>
std::vector<T> load_bin(const std::string& directory, const std::string& name) {
  const auto bytes = read_file(directory + "/" + name + ".bin");
  if (bytes.size() % sizeof(T) != 0) {
    std::fprintf(stderr, "%s size not multiple of %zu\n", name.c_str(), sizeof(T));
    std::exit(2);
  }
  std::vector<T> values(bytes.size() / sizeof(T));
  if (!bytes.empty()) std::memcpy(values.data(), bytes.data(), bytes.size());
  return values;
}

template <typename T>
void write_bin(const std::string& directory, const std::string& name,
               const std::vector<T>& values) {
  std::ofstream output(directory + "/" + name + ".bin", std::ios::binary);
  if (!values.empty()) {
    output.write(reinterpret_cast<const char*>(values.data()),
                 static_cast<std::streamsize>(values.size() * sizeof(T)));
  }
}

// 设备侧指针集合（kernel 参数按值传递）。
struct ProbeArgs {
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

__device__ __forceinline__ d_geom build_geom(const ProbeArgs& a, int world, int geom_index) {
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

__device__ __forceinline__ d_contact_params build_params(const ProbeArgs& a) {
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
__global__ void kernel_contact(ProbeArgs a) {
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
  g5_ccd_pair_write(in, params, a.naconmax, a.nccd, &local.pt, &a.out);
}

// hfield 设备路径（上游 `ccd_hfield_kernel`）。
__global__ void kernel_hfield(ProbeArgs a) {
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
__global__ void kernel_sdf(ProbeArgs a) {
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

struct DeviceArrays {
  std::vector<float> geom_xpos, geom_xmat, geom_size, geom_margin, geom_gap, geom_rbound,
      geom_aabb, mesh_vert, hfield_size, hfield_data, oct_aabb, oct_coeff;
  std::vector<int> geom_type, geom_dataid, mesh_vertadr, mesh_vertnum, mesh_graphadr,
      mesh_octadr, mesh_graph, hfield_nrow, hfield_ncol, hfield_adr, oct_child;
};

template <typename T>
T* to_device(const std::vector<T>& host) {
  T* device = nullptr;
  const size_t bytes = host.size() * sizeof(T);
  PROBE_CHECK(cudaMalloc(&device, bytes == 0 ? sizeof(T) : bytes));
  if (!host.empty()) PROBE_CHECK(cudaMemcpy(device, host.data(), bytes, cudaMemcpyHostToDevice));
  return device;
}

template <typename T>
void from_device(const T* device, std::vector<T>& host) {
  if (host.empty()) return;
  PROBE_CHECK(cudaMemcpy(host.data(), device, host.size() * sizeof(T), cudaMemcpyDeviceToHost));
}

struct Outputs {
  int nacon = 0;
  std::vector<float> dist, pos, frame, includemargin, friction, solref, solreffriction, solimp,
      adhesion;
  std::vector<int> dim, geom, efc_address, worldid, type, geomcollisionid;
  std::vector<unsigned> overflow;
  std::vector<int> nccd;
  std::vector<int> nacon_counter;
};

}  // namespace

