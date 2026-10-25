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

int main(int argc, char** argv) {
  if (argc < 2) {
    std::fprintf(stderr, "usage: probe <case_dir> [repeats]\n");
    return 2;
  }
  const std::string directory = argv[1];
  const int repeats = argc > 2 ? std::atoi(argv[2]) : 2;
  const CaseMeta meta = read_meta(directory);

  DeviceArrays host;
  host.geom_xpos = load_bin<float>(directory, "geom_xpos");
  host.geom_xmat = load_bin<float>(directory, "geom_xmat");
  host.geom_size = load_bin<float>(directory, "geom_size");
  host.geom_margin = load_bin<float>(directory, "geom_margin");
  host.geom_gap = load_bin<float>(directory, "geom_gap");
  host.geom_rbound = load_bin<float>(directory, "geom_rbound");
  host.geom_aabb = load_bin<float>(directory, "geom_aabb");
  host.geom_type = load_bin<int>(directory, "geom_type");
  host.geom_dataid = load_bin<int>(directory, "geom_dataid");
  host.mesh_vertadr = load_bin<int>(directory, "mesh_vertadr");
  host.mesh_vertnum = load_bin<int>(directory, "mesh_vertnum");
  host.mesh_graphadr = load_bin<int>(directory, "mesh_graphadr");
  host.mesh_octadr = load_bin<int>(directory, "mesh_octadr");
  host.mesh_vert = load_bin<float>(directory, "mesh_vert");
  host.mesh_graph = load_bin<int>(directory, "mesh_graph");
  host.hfield_size = load_bin<float>(directory, "hfield_size");
  host.hfield_nrow = load_bin<int>(directory, "hfield_nrow");
  host.hfield_ncol = load_bin<int>(directory, "hfield_ncol");
  host.hfield_adr = load_bin<int>(directory, "hfield_adr");
  host.hfield_data = load_bin<float>(directory, "hfield_data");
  host.oct_aabb = load_bin<float>(directory, "oct_aabb");
  host.oct_child = load_bin<int>(directory, "oct_child");
  host.oct_coeff = load_bin<float>(directory, "oct_coeff");

  ProbeArgs args{};
  args.geom_xpos = to_device(host.geom_xpos);
  args.geom_xmat = to_device(host.geom_xmat);
  args.geom_size = to_device(host.geom_size);
  args.geom_margin = to_device(host.geom_margin);
  args.geom_gap = to_device(host.geom_gap);
  args.geom_rbound = to_device(host.geom_rbound);
  args.geom_aabb = to_device(host.geom_aabb);
  args.geom_type = to_device(host.geom_type);
  args.geom_dataid = to_device(host.geom_dataid);
  args.mesh_vertadr = to_device(host.mesh_vertadr);
  args.mesh_vertnum = to_device(host.mesh_vertnum);
  args.mesh_graphadr = to_device(host.mesh_graphadr);
  args.mesh_octadr = to_device(host.mesh_octadr);
  args.mesh_vert = to_device(host.mesh_vert);
  args.mesh_graph = to_device(host.mesh_graph);
  args.hfield_size = to_device(host.hfield_size);
  args.hfield_nrow = to_device(host.hfield_nrow);
  args.hfield_ncol = to_device(host.hfield_ncol);
  args.hfield_adr = to_device(host.hfield_adr);
  args.hfield_data = to_device(host.hfield_data);
  args.oct_aabb = to_device(host.oct_aabb);
  args.oct_child = to_device(host.oct_child);
  args.oct_coeff = to_device(host.oct_coeff);

  args.nworld = meta.i32("nworld");
  args.ngeom = meta.i32("ngeom");
  args.g1 = meta.i32("g1");
  args.g2 = meta.i32("g2");
  args.type1 = meta.i32("type1");
  args.type2 = meta.i32("type2");
  args.geomgeomid = 0;
  args.naccdmax = meta.i32("naccdmax");
  args.naconmax = meta.i32("naconmax");
  args.efc_width = meta.i32("efc_width");
  args.hfield_id = meta.i32("hfield_id");
  args.oct_root = meta.i32("oct_root");
  args.noct = meta.i32("noct");
  args.tolerance = meta.f32("tolerance");
  args.margin = meta.f32("margin");
  args.gap = meta.f32("gap");
  args.gjk_iterations = meta.i32("gjk_iterations");
  args.epa_iterations = meta.i32("epa_iterations");
  args.condim = meta.i32("condim");
  args.pairid0 = meta.i32("pairid0");
  args.pairid1 = meta.i32("pairid1");
  const auto friction = meta.f32s("friction");
  const auto solref = meta.f32s("solref");
  const auto solreffriction = meta.f32s("solreffriction");
  const auto solimp = meta.f32s("solimp");
  for (int index = 0; index < 5; ++index) {
    args.friction[index] = friction[index];
    args.solimp[index] = solimp[index];
  }
  for (int index = 0; index < 2; ++index) {
    args.solref[index] = solref[index];
    args.solreffriction[index] = solreffriction[index];
  }
  args.adhesion = meta.f32("adhesion");
  args.sdf_initpoints = meta.i32("sdf_initpoints");
  args.sdf_iterations = meta.i32("sdf_iterations");

  const int naconmax = args.naconmax;
  const int efc_width = args.efc_width;
  const int nworld = args.nworld;

  // 输出缓冲（设备端）。
  float* d_dist = nullptr;
  v3* d_pos = nullptr;
  float* d_frame = nullptr;
  float* d_includemargin = nullptr;
  float* d_friction = nullptr;
  float* d_solref = nullptr;
  float* d_solreffriction = nullptr;
  float* d_solimp = nullptr;
  float* d_adhesion = nullptr;
  int* d_dim = nullptr;
  int* d_geom = nullptr;
  int* d_efc = nullptr;
  int* d_worldid = nullptr;
  int* d_type = nullptr;
  int* d_geomcollisionid = nullptr;
  int* d_nacon = nullptr;
  int* d_nccd = nullptr;
  unsigned* d_overflow = nullptr;
  PROBE_CHECK(cudaMalloc(&d_dist, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_pos, naconmax * sizeof(v3)));
  PROBE_CHECK(cudaMalloc(&d_frame, naconmax * 9 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_includemargin, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_friction, naconmax * 5 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solref, naconmax * 2 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solreffriction, naconmax * 2 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solimp, naconmax * 5 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_adhesion, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_dim, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_geom, naconmax * 2 * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_efc, naconmax * efc_width * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_worldid, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_type, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_geomcollisionid, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_nacon, sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_nccd, sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_overflow, nworld * sizeof(unsigned)));

  d_contact_out out{};
  out.dist = d_dist;
  out.pos = d_pos;
  out.frame = d_frame;
  out.includemargin = d_includemargin;
  out.friction = d_friction;
  out.solref = d_solref;
  out.solreffriction = d_solreffriction;
  out.solimp = d_solimp;
  out.dim = d_dim;
  out.geom = d_geom;
  out.efc_address = d_efc;
  out.worldid = d_worldid;
  out.type = d_type;
  out.geomcollisionid = d_geomcollisionid;
  out.adhesion = d_adhesion;
  out.nacon = d_nacon;
  out.efc_address_width = efc_width;
  args.out = out;
  args.nacon = d_nacon;
  args.nccd = d_nccd;
  args.overflow = d_overflow;

  cudaDeviceProp properties{};
  PROBE_CHECK(cudaGetDeviceProperties(&properties, 0));
  size_t free_bytes = 0;
  size_t total_bytes = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_bytes, &total_bytes));
  std::printf("device=%s sm=%d.%d cudart=%d free=%zuMiB total=%zuMiB\n", properties.name,
              properties.major, properties.minor, CUDART_VERSION, free_bytes >> 20,
              total_bytes >> 20);

  const std::string kind = meta.text("kind");
  void (*kernel)(ProbeArgs) = kernel_contact;
  if (kind == "hfield") kernel = kernel_hfield;
  if (kind == "sdf") kernel = kernel_sdf;

  std::vector<double> times;
  std::vector<Outputs> runs;
  for (int repeat = 0; repeat < repeats; ++repeat) {
    PROBE_CHECK(cudaMemset(d_nacon, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(d_nccd, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(d_overflow, 0, nworld * sizeof(unsigned)));
    PROBE_CHECK(cudaMemset(d_efc, 0xff, naconmax * efc_width * sizeof(int)));
    const auto started = std::chrono::steady_clock::now();
    kernel<<<nworld, 1>>>(args);
    PROBE_CHECK(cudaGetLastError());
    PROBE_CHECK(cudaDeviceSynchronize());
    const auto elapsed =
        std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - started)
            .count();
    times.push_back(elapsed);

    Outputs run;
    std::vector<int> nacon_counter(1, 0);
    from_device(d_nacon, nacon_counter);
    run.nacon_counter = nacon_counter;
    run.nacon = nacon_counter[0];
    const int rows = naconmax;
    run.dist.resize(rows);
    run.pos.resize(rows * 3);
    run.frame.resize(rows * 9);
    run.includemargin.resize(rows);
    run.friction.resize(rows * 5);
    run.solref.resize(rows * 2);
    run.solreffriction.resize(rows * 2);
    run.solimp.resize(rows * 5);
    run.adhesion.resize(rows);
    run.dim.resize(rows);
    run.geom.resize(rows * 2);
    run.efc_address.resize(rows * efc_width);
    run.worldid.resize(rows);
    run.type.resize(rows);
    run.geomcollisionid.resize(rows);
    run.overflow.resize(nworld);
    run.nccd.resize(1);
    from_device(d_dist, run.dist);
    from_device(reinterpret_cast<const float*>(d_pos), run.pos);
    from_device(d_frame, run.frame);
    from_device(d_includemargin, run.includemargin);
    from_device(d_friction, run.friction);
    from_device(d_solref, run.solref);
    from_device(d_solreffriction, run.solreffriction);
    from_device(d_solimp, run.solimp);
    from_device(d_adhesion, run.adhesion);
    from_device(d_dim, run.dim);
    from_device(d_geom, run.geom);
    from_device(d_efc, run.efc_address);
    from_device(d_worldid, run.worldid);
    from_device(d_type, run.type);
    from_device(d_geomcollisionid, run.geomcollisionid);
    from_device(d_overflow, run.overflow);
    from_device(d_nccd, run.nccd);
    runs.push_back(run);
  }

  // 只落有意义的行（nacon 行）；同时落 nacon/overflow/nccd 汇总。
  const int nacon_rows = runs.front().nacon;
  for (size_t repeat = 0; repeat < runs.size(); ++repeat) {
    const std::string out_dir = directory + "/out" + std::to_string(repeat + 1);
    std::system(("mkdir \"" + out_dir + "\" 2>nul").c_str());
    const Outputs& run = runs[repeat];
    std::vector<float> dist(run.dist.begin(), run.dist.begin() + nacon_rows);
    std::vector<float> pos(run.pos.begin(), run.pos.begin() + nacon_rows * 3);
    std::vector<float> frame(run.frame.begin(), run.frame.begin() + nacon_rows * 9);
    std::vector<float> includemargin(run.includemargin.begin(),
                                    run.includemargin.begin() + nacon_rows);
    std::vector<float> friction(run.friction.begin(), run.friction.begin() + nacon_rows * 5);
    std::vector<float> solref(run.solref.begin(), run.solref.begin() + nacon_rows * 2);
    std::vector<float> solreffriction(run.solreffriction.begin(),
                                      run.solreffriction.begin() + nacon_rows * 2);
    std::vector<float> solimp(run.solimp.begin(), run.solimp.begin() + nacon_rows * 5);
    std::vector<float> adhesion(run.adhesion.begin(), run.adhesion.begin() + nacon_rows);
    std::vector<int> dim(run.dim.begin(), run.dim.begin() + nacon_rows);
    std::vector<int> geom(run.geom.begin(), run.geom.begin() + nacon_rows * 2);
    std::vector<int> efc(run.efc_address.begin(), run.efc_address.begin() + nacon_rows * efc_width);
    std::vector<int> worldid(run.worldid.begin(), run.worldid.begin() + nacon_rows);
    std::vector<int> type(run.type.begin(), run.type.begin() + nacon_rows);
    std::vector<int> geomcollisionid(run.geomcollisionid.begin(),
                                     run.geomcollisionid.begin() + nacon_rows);
    write_bin(out_dir, "dist", dist);
    write_bin(out_dir, "pos", pos);
    write_bin(out_dir, "frame", frame);
    write_bin(out_dir, "includemargin", includemargin);
    write_bin(out_dir, "friction", friction);
    write_bin(out_dir, "solref", solref);
    write_bin(out_dir, "solreffriction", solreffriction);
    write_bin(out_dir, "solimp", solimp);
    write_bin(out_dir, "adhesion", adhesion);
    write_bin(out_dir, "dim", dim);
    write_bin(out_dir, "geom", geom);
    write_bin(out_dir, "efc_address", efc);
    write_bin(out_dir, "worldid", worldid);
    write_bin(out_dir, "type", type);
    write_bin(out_dir, "geomcollisionid", geomcollisionid);
    write_bin(out_dir, "overflow", run.overflow);
    write_bin(out_dir, "nccd", run.nccd);
    write_bin(out_dir, "nacon", run.nacon_counter);
    // efc 宽度记录，Rust 侧据此解析 efc_address。
    std::ofstream efc_meta(out_dir + "/efc_width.txt");
    efc_meta << efc_width << "\n";
  }

  const bool raw_equal = runs.size() < 2 || [&]() {
    const Outputs& a = runs[0];
    const Outputs& b = runs[1];
    return a.nacon == b.nacon && a.dist == b.dist && a.pos == b.pos && a.frame == b.frame &&
           a.dim == b.dim && a.geom == b.geom && a.worldid == b.worldid && a.type == b.type &&
           a.geomcollisionid == b.geomcollisionid && a.overflow == b.overflow && a.nccd == b.nccd;
  }();
  std::printf("case=%s kind=%s nacon=%d overflow=", meta.text("case").c_str(), kind.c_str(),
              runs.front().nacon);
  for (unsigned value : runs.front().overflow) std::printf("0x%x,", value);
  std::printf(" nccd=%d raw_equal=%s times=", runs.front().nccd[0], raw_equal ? "true" : "false");
  for (double time : times) std::printf("%.3fms,", time);
  std::printf("\n");

  std::ofstream report(directory + "/probe-report.txt");
  report << "case " << meta.text("case") << "\n";
  report << "kind " << kind << "\n";
  report << "device " << properties.name << "\n";
  report << "compute_capability " << properties.major << "." << properties.minor << "\n";
  report << "cudart " << CUDART_VERSION << "\n";
  report << "nacon " << runs.front().nacon << "\n";
  report << "nccd " << runs.front().nccd[0] << "\n";
  for (size_t index = 0; index < runs.front().overflow.size(); ++index) {
    report << "overflow_world" << index << " " << runs.front().overflow[index] << "\n";
  }
  report << "two_runs_raw_bitwise_identical " << (raw_equal ? "true" : "false") << "\n";
  for (size_t index = 0; index < times.size(); ++index) {
    report << "run" << (index + 1) << "_ms " << times[index] << "\n";
  }
  report << "free_mib_before " << (free_bytes >> 20) << "\n";
  size_t free_after = 0;
  size_t total_after = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_after, &total_after));
  report << "free_mib_after " << (free_after >> 20) << "\n";
  return 0;
}
